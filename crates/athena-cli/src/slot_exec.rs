//! The producer's `cargo` wrapper (`docs/design/agent-and-build-slots.md` sections 4 and 8, build step 4).
//!
//! A producer's `PATH` starts with a directory whose `cargo` is a link to the `athena` binary
//! ([`athena_agent::env::install_cargo_wrapper`]). `main` sees the name `cargo` and calls [`run_cargo`] here.
//! The wrapper finds the subcommand after the global options, using [`CARGO_GLOBAL_FLAGS`], the list the command
//! policy skips too, so `cargo -q build` and `cargo --locked test` cannot pass the queue by hiding the
//! subcommand behind a flag. For `build`, `test`, `clippy` and `check` it takes a build slot from the queue the
//! drain uses ([`SlotQueue`]), runs the real `cargo` as a child and frees the slot when the child ends. Every
//! other subcommand runs with no slot.
//!
//! When no slot frees within the queue's limit the real `cargo` is not started: the wrapper writes the hold
//! marker the run reads ([`SlotWait`]), prints [`wait_message`] and exits 75, an environment result and not a
//! policy denial. The run then holds the job instead of failing it.

use athena_agent::env::{
    SLOT_COUNT_ENV, SLOT_DIR_ENV, SLOT_HOLD_ENV, SLOT_PROMOTE_ENV, SLOT_WAIT_ENV, WRAPPER_DIR_ENV,
};
use athena_agent::policy::CARGO_GLOBAL_FLAGS;
use athena_core::disk::SlotWait;
use athena_core::slots::{Class, SlotQueue};
use std::ffi::{OsStr, OsString};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// The subcommands that build, and so take a slot.
pub const SLOTTED: [&str; 4] = ["build", "test", "clippy", "check"];

/// The exit status when no slot came free (`EX_TEMPFAIL`).
pub const EXIT_NO_SLOT: i32 = 75;

/// The first word after the leading global options, which is cargo's subcommand. `None` when the arguments end
/// first, or when an option that is not in [`CARGO_GLOBAL_FLAGS`] comes first (cargo reads that one and the
/// policy refuses it, so the wrapper runs it without a slot).
pub fn subcommand(args: &[OsString]) -> Option<&str> {
    for arg in args {
        let word = arg.to_str()?;
        if word.starts_with('-') {
            if !CARGO_GLOBAL_FLAGS.contains(&word) {
                return None;
            }
            continue;
        }
        return Some(word);
    }
    None
}

/// Whether `cargo <args>` takes a build slot.
pub fn takes_slot(args: &[OsString]) -> bool {
    subcommand(args).is_some_and(|sub| SLOTTED.contains(&sub))
}

/// What the wrapper prints when no slot freed.
pub fn wait_message(limit_s: u64) -> String {
    format!("no build slot free for {limit_s} s; try again")
}

/// The queue and hold file the producer's environment names, read from `get`. `None` when the environment has
/// no queue (the wrapper then runs everything unslotted).
fn queue_from(get: &dyn Fn(&str) -> Option<String>) -> Option<(SlotQueue, Option<PathBuf>)> {
    let dir = PathBuf::from(get(SLOT_DIR_ENV)?);
    let number = |key: &str, default: u64| get(key).and_then(|v| v.parse().ok()).unwrap_or(default);
    let queue = SlotQueue::new(
        dir,
        number(SLOT_COUNT_ENV, 1) as usize,
        Duration::from_secs(number(SLOT_WAIT_ENV, 1800)),
        Duration::from_secs(number(SLOT_PROMOTE_ENV, 600)),
    );
    Some((queue, get(SLOT_HOLD_ENV).map(PathBuf::from)))
}

/// The real `cargo`: the first `cargo` on `path` that is not in `skip` (the wrapper's own directory).
fn real_cargo(path: &OsStr, skip: Option<&Path>) -> Option<PathBuf> {
    std::env::split_paths(path)
        .filter(|dir| skip != Some(dir.as_path()))
        .map(|dir| dir.join("cargo"))
        .find(|candidate| {
            use std::os::unix::fs::PermissionsExt;
            candidate
                .metadata()
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
}

/// Runs `cargo <args>` as the producer's wrapper does and returns the exit status for the process.
pub fn run_cargo(args: &[OsString]) -> i32 {
    let get = |key: &str| std::env::var(key).ok();
    let skip = get(WRAPPER_DIR_ENV).map(PathBuf::from);
    let path = std::env::var_os("PATH").unwrap_or_default();
    let Some(cargo) = real_cargo(&path, skip.as_deref()) else {
        eprintln!("athena cargo wrapper: no cargo found on PATH");
        return 127;
    };
    // Held until the child ends; dropped on return, or freed by the system if this process dies.
    let _slot = if takes_slot(args) {
        match queue_from(&get) {
            Some((queue, hold_file)) => match queue.acquire(Class::Producer, &|| Ok(())) {
                Ok(slot) => Some(slot),
                Err(error) => return no_slot(&error, hold_file.as_deref()),
            },
            None => None,
        }
    } else {
        None
    };
    match Command::new(&cargo).args(args).status() {
        Ok(status) => status
            .code()
            .unwrap_or_else(|| 128 + status.signal().unwrap_or(0)),
        Err(error) => {
            eprintln!(
                "athena cargo wrapper: cannot run {}: {error}",
                cargo.display()
            );
            126
        }
    }
}

/// Ends the command for a slot that never came: the message, the hold marker, exit 75.
fn no_slot(error: &anyhow::Error, hold_file: Option<&Path>) -> i32 {
    let Some(wait) = error.downcast_ref::<SlotWait>() else {
        eprintln!("athena cargo wrapper: the build-slot queue failed: {error:#}");
        return 1;
    };
    if let Some(file) = hold_file {
        if let Some(parent) = file.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(text) = serde_json::to_string(wait) {
            let _ = std::fs::write(file, text);
        }
    }
    eprintln!("{}", wait_message(wait.limit_s));
    EXIT_NO_SLOT
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(words: &[&str]) -> Vec<OsString> {
        words.iter().map(OsString::from).collect()
    }

    #[test]
    fn the_subcommand_is_found_after_the_global_flags() {
        assert_eq!(subcommand(&args(&["build"])), Some("build"));
        assert_eq!(subcommand(&args(&["-q", "build"])), Some("build"));
        assert_eq!(
            subcommand(&args(&["--locked", "test", "-p", "x"])),
            Some("test")
        );
        assert_eq!(
            subcommand(&args(&["-q", "--offline", "-vv", "check"])),
            Some("check")
        );
        assert_eq!(subcommand(&args(&["-q"])), None);
        assert_eq!(subcommand(&args(&["--config", "x=1", "build"])), None);
        assert_eq!(subcommand(&args(&[])), None);
    }

    #[test]
    fn every_policy_global_flag_is_skipped() {
        for flag in CARGO_GLOBAL_FLAGS {
            assert_eq!(
                subcommand(&args(&[flag, "clippy"])),
                Some("clippy"),
                "{flag}"
            );
        }
    }

    #[test]
    fn only_build_test_clippy_and_check_take_a_slot() {
        for sub in ["build", "test", "clippy", "check"] {
            assert!(takes_slot(&args(&[sub])), "{sub}");
            assert!(takes_slot(&args(&["--locked", sub])), "{sub}");
        }
        for sub in ["fmt", "metadata", "tree", "doc", "run"] {
            assert!(!takes_slot(&args(&[sub])), "{sub}");
        }
        assert!(
            !takes_slot(&args(&["fmt", "build"])),
            "a later word is an argument"
        );
    }

    #[test]
    fn the_wait_message_names_the_limit() {
        assert_eq!(
            wait_message(1800),
            "no build slot free for 1800 s; try again"
        );
    }
}
