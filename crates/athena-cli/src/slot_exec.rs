//! The producer's `cargo`: the `athena` binary linked as `cargo` in a directory first on the producer's `PATH`,
//! so that the producer's `cargo build`, `test`, `clippy` and `check` take a build slot like a check does
//! (`docs/design/agent-and-build-slots.md` section 6, step 4).
//!
//! The wrapper finds the subcommand after the global options the tool policy allows
//! ([`athena_agent::policy::cargo_subcommand`], the policy's own list and walk). For a subcommand that builds it
//! takes a [`Class::Producer`] slot, runs the real `cargo` and frees the slot when cargo exits. Every other
//! subcommand and every form it does not recognise runs the real `cargo` at once with no slot.
//!
//! The real `cargo` runs as a child, not by `exec`: the slot is a file lock this process holds, and the workspace
//! denies the `unsafe` that passing the lock on would need. The child's `PATH` lacks the shim directory, so a
//! `cargo` that the real `cargo` starts (a subcommand, a build script) is the real one and cannot wait for the
//! slot its parent holds.
//!
//! A wait past the limit prints [`wait_message`], writes the wait note the run reads after the producer ends
//! (the job is then held, not failed) and exits [`EXIT_WAIT`]. The real `cargo` is never started.

use athena_agent::env::CargoShim;
use athena_agent::policy::cargo_subcommand;
use athena_core::disk::SlotWait;
use athena_core::slots::{self, Class, SlotQueue};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::time::Duration;

/// The subcommands that take a build slot: the ones that compile.
pub const CARGO_SLOT_SUBCOMMANDS: [&str; 4] = ["build", "test", "clippy", "check"];

/// The exit code of a wrapper that gave up waiting for a slot: `EX_TEMPFAIL`, not a failure of the command.
pub const EXIT_WAIT: u8 = 75;
/// The exit code when the wrapper's settings are unusable (`EX_SOFTWARE`).
pub const EXIT_SETTINGS: u8 = 70;
/// The exit code when no real `cargo` is on the `PATH` (the shell's "command not found").
pub const EXIT_NOT_FOUND: u8 = 127;

/// What the wrapper tells the producer when no slot came free.
pub fn wait_message(limit_s: u64) -> String {
    format!("no build slot free for {limit_s} s; try again")
}

/// Runs the producer's `cargo` with `args` (the words after the program name). `lookup` reads the environment.
/// Returns the exit code.
pub fn run(args: &[String], lookup: &dyn Fn(&str) -> Option<String>) -> u8 {
    let shim = match CargoShim::from_env(lookup) {
        Ok(shim) => shim,
        Err(why) => {
            eprintln!("cargo: the build-slot wrapper has no settings ({why})");
            return EXIT_SETTINGS;
        }
    };
    let path = shim.without_shim(&lookup("PATH").unwrap_or_default());
    let Some(real) = find_program("cargo", &path) else {
        eprintln!("cargo: not found on PATH");
        return EXIT_NOT_FOUND;
    };
    let words: Vec<&str> = args.iter().map(String::as_str).collect();
    let takes_slot =
        cargo_subcommand(&words).is_some_and(|sub| CARGO_SLOT_SUBCOMMANDS.contains(&sub));
    // The slot, when there is one, is held until after the child has exited.
    let _slot = if takes_slot {
        let queue = SlotQueue::new(
            shim.slots_dir.clone(),
            shim.slots,
            Duration::from_secs(shim.wait_s),
            Duration::from_secs(shim.promote_s),
        );
        match queue.acquire(Class::Producer, &|| Ok(())) {
            Ok(slot) => {
                // A slot after an earlier give-up: the producer tried again, so it is not held for that wait.
                slots::clear_wait_note(&shim.note);
                Some(slot)
            }
            Err(error) => match error.downcast_ref::<SlotWait>() {
                Some(wait) => {
                    eprintln!("{}", wait_message(wait.limit_s));
                    if let Err(why) = slots::write_wait_note(&shim.note, wait) {
                        eprintln!("cargo: {why:#}");
                    }
                    return EXIT_WAIT;
                }
                None => {
                    eprintln!("cargo: taking a build slot failed: {error:#}");
                    return EXIT_SETTINGS;
                }
            },
        }
    } else {
        None
    };
    let mut cmd = Command::new(&real);
    cmd.args(args).env("PATH", &path);
    match athena_agent::procs::retry_busy(|| cmd.status()) {
        Ok(status) => exit_code(status),
        Err(error) => {
            eprintln!("cargo: running {}: {error}", real.display());
            EXIT_NOT_FOUND
        }
    }
}

/// The exit code to return for a child's status: its own code, or 128 plus the signal that ended it.
pub fn exit_code(status: ExitStatus) -> u8 {
    match (status.code(), status.signal()) {
        (Some(code), _) => u8::try_from(code & 0xff).unwrap_or(1),
        (None, Some(signal)) => u8::try_from(128 + signal).unwrap_or(255),
        (None, None) => 1,
    }
}

/// The first executable file called `name` in the directories of `path`.
pub fn find_program(name: &str, path: &str) -> Option<PathBuf> {
    path.split(':')
        .filter(|dir| !dir.is_empty())
        .map(|dir| Path::new(dir).join(name))
        .find(|candidate| {
            candidate
                .metadata()
                .is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn script(dir: &Path, name: &str, mode: u32) {
        let path = dir.join(name);
        let body = "#!/bin/sh\nexit 0\n";
        if mode & 0o111 != 0 {
            athena_testkit::write_executable(&path, body);
        } else {
            // A newly written file is not executable, so a non-executable stand-in needs no mode change.
            std::fs::write(&path, body).unwrap();
        }
    }

    #[test]
    fn the_real_cargo_is_the_first_executable_on_the_path() {
        let temp = tempfile::tempdir().unwrap();
        let (a, b, c) = (
            temp.path().join("a"),
            temp.path().join("b"),
            temp.path().join("c"),
        );
        for dir in [&a, &b, &c] {
            std::fs::create_dir_all(dir).unwrap();
        }
        // `a` holds a file that is not executable; `b` and `c` hold real ones.
        script(&a, "cargo", 0o644);
        script(&b, "cargo", 0o755);
        script(&c, "cargo", 0o755);
        let path = format!(
            "{}::{}:{}:/nonexistent",
            a.display(),
            b.display(),
            c.display()
        );
        assert_eq!(find_program("cargo", &path), Some(b.join("cargo")));
        assert_eq!(find_program("cargo", &a.display().to_string()), None);
        assert_eq!(find_program("cargo", ""), None);
    }

    #[test]
    fn exit_codes_follow_the_child() {
        let status = |code: i32| ExitStatus::from_raw(code << 8);
        assert_eq!(exit_code(status(0)), 0);
        assert_eq!(exit_code(status(101)), 101);
        // Ended by a signal (9): 128 + 9, as a shell reports it.
        assert_eq!(exit_code(ExitStatus::from_raw(9)), 137);
    }

    #[test]
    fn the_message_names_the_limit() {
        assert_eq!(wait_message(300), "no build slot free for 300 s; try again");
    }

    #[test]
    fn only_the_building_subcommands_take_a_slot() {
        assert_eq!(CARGO_SLOT_SUBCOMMANDS, ["build", "test", "clippy", "check"]);
        for sub in ["fmt", "metadata", "tree", "doc", "run"] {
            assert!(!CARGO_SLOT_SUBCOMMANDS.contains(&sub), "{sub}");
        }
    }
}
