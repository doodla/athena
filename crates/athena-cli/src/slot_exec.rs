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
//! While it holds a slot the wrapper links the job's `CARGO_TARGET_DIR` path to the slot's warm target
//! directory ([`link_slot_target`]), as the checks do, and removes the link when cargo exits.
//!
//! A wait past the limit prints [`wait_message`], writes the wait note the run reads after the producer ends
//! (the job is then held, not failed) and exits [`EXIT_WAIT`]. The real `cargo` is never started.

use athena_agent::env::{BuildLink, CargoShim};
use athena_agent::policy::cargo_subcommand;
use athena_core::disk::SlotWait;
use athena_core::slots::{self, Class, SlotQueue};
use athena_core::state::StateDir;
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
    let held = if takes_slot {
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
    // The link lives until the child has exited and goes before the slot is freed (locals drop in reverse order).
    let _link = held
        .as_ref()
        .and_then(|slot| link_slot_target(&shim, slot, lookup));
    match Command::new(&real).args(args).env("PATH", &path).status() {
        Ok(status) => exit_code(status),
        Err(error) => {
            eprintln!("cargo: running {}: {error}", real.display());
            EXIT_NOT_FOUND
        }
    }
}

/// Links the job's `CARGO_TARGET_DIR` path to the warm target directory of the slot held, as the checks do, so
/// the producer's build fills the slot's directory and not a private one. The job directory is the parent of the
/// shim directory. The slot directories sit in the state directory beside the slot lock files
/// ([`StateDir::slots_dir`] and [`StateDir::slot_target`]). With no `CARGO_TARGET_DIR`, or when the slot's
/// directory cannot be made or the path is refused (it must lie inside the job directory, D46), a warning goes
/// to stderr and cargo runs unlinked, in whatever `CARGO_TARGET_DIR` names.
fn link_slot_target(
    shim: &CargoShim,
    slot: &slots::Slot,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<BuildLink> {
    let path = lookup("CARGO_TARGET_DIR").filter(|p| !p.is_empty())?;
    let (Some(job_dir), Some(state_root)) = (shim.dir.parent(), shim.slots_dir.parent()) else {
        eprintln!("cargo: warning: no build slot link: the shim settings name no parent directory");
        return None;
    };
    // No size limit here: the wrapper has no such setting, so nothing is deleted.
    let target =
        match StateDir::new(state_root.to_path_buf()).prepare_slot_target(slot.index(), u64::MAX) {
            Ok(prepared) => prepared.dir,
            Err(error) => {
                eprintln!(
                    "cargo: warning: build slot {} has no usable target directory: {error:#}",
                    slot.index()
                );
                return None;
            }
        };
    match BuildLink::create(job_dir, Path::new(&path), &target) {
        Ok(link) => Some(link),
        Err(error) => {
            eprintln!("cargo: warning: could not link the build slot: {error}");
            None
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
        std::fs::write(&path, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(mode)).unwrap();
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

    /// A state directory with a fake `cargo` that records what its job's target path is while it runs.
    struct Linked {
        temp: tempfile::TempDir,
        fake_dir: PathBuf,
    }

    impl Linked {
        fn new() -> Linked {
            let temp = tempfile::tempdir().unwrap();
            let fake_dir = temp.path().join("fake");
            std::fs::create_dir_all(&fake_dir).unwrap();
            Linked { temp, fake_dir }
        }

        fn job_dir(&self, name: &str) -> PathBuf {
            self.temp.path().join("jobs").join(name)
        }

        fn slot_target(&self) -> PathBuf {
            self.temp.path().join("build-slots/0/target")
        }

        /// Installs a fake cargo that records, for `target`, whether it is a link and where it resolves, and
        /// writes `built-by-<tag>` into it. The records go to `<temp>/seen-<tag>`.
        fn fake(&self, tag: &str, target: &Path) {
            let seen = self.temp.path().join(format!("seen-{tag}"));
            let body = format!(
                "#!/bin/sh\nT='{t}'\nif [ -L \"$T\" ]; then echo link > '{s}'; else echo plain >> '{s}'; fi\n\
                 (cd -P \"$T\" 2>/dev/null && pwd -P) >> '{s}'\nif [ -d \"$T\" ]; then echo x > \"$T/built-by-{tag}\"; fi\nexit 0\n",
                t = target.display(),
                s = seen.display()
            );
            std::fs::write(self.fake_dir.join("cargo"), body).unwrap();
            std::fs::set_permissions(
                self.fake_dir.join("cargo"),
                std::fs::Permissions::from_mode(0o755),
            )
            .unwrap();
        }

        fn seen(&self, tag: &str) -> Vec<String> {
            std::fs::read_to_string(self.temp.path().join(format!("seen-{tag}")))
                .unwrap()
                .lines()
                .map(String::from)
                .collect()
        }

        /// Runs the wrapper for the job directory `job` with `CARGO_TARGET_DIR` as given.
        fn run(&self, job: &Path, target: Option<&Path>, args: &[&str]) -> u8 {
            let values = std::collections::BTreeMap::from([
                (
                    "ATHENA_CARGO_SHIM_DIR",
                    job.join("shim").display().to_string(),
                ),
                (
                    "ATHENA_SLOTS_DIR",
                    self.temp.path().join("slots").display().to_string(),
                ),
                ("ATHENA_SLOTS", "1".to_string()),
                ("ATHENA_SLOT_WAIT_S", "5".to_string()),
                ("ATHENA_SLOT_PROMOTE_S", "600".to_string()),
                (
                    "ATHENA_SLOT_WAIT_NOTE",
                    job.join("slot-wait.json").display().to_string(),
                ),
                ("PATH", self.fake_dir.display().to_string()),
            ]);
            let lookup = |key: &str| {
                if key == "CARGO_TARGET_DIR" {
                    return target.map(|t| t.display().to_string());
                }
                values.get(key).cloned()
            };
            let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
            run(&args, &lookup)
        }
    }

    #[test]
    fn a_slotted_build_runs_in_the_slots_target_directory_through_a_link_that_is_gone_after() {
        let state = Linked::new();
        let job = state.job_dir("a");
        let target = job.join("target/produce");
        state.fake("a", &target);
        assert_eq!(state.run(&job, Some(&target), &["build"]), 0);
        let seen = state.seen("a");
        let slot = state.slot_target().canonicalize().unwrap();
        assert_eq!(seen, ["link".to_string(), slot.display().to_string()]);
        assert!(slot.join("built-by-a").exists(), "built in the slot");
        assert!(slot.join("CACHEDIR.TAG").exists());
        assert!(
            std::fs::symlink_metadata(&target).is_err(),
            "the link is gone after"
        );
        assert!(slot.is_dir(), "the slot directory stays");
    }

    #[test]
    fn two_jobs_one_after_the_other_reuse_the_same_slot_directory() {
        let state = Linked::new();
        let (first, second) = (state.job_dir("a"), state.job_dir("b"));
        let (t1, t2) = (first.join("target/produce"), second.join("target/produce"));
        state.fake("a", &t1);
        assert_eq!(state.run(&first, Some(&t1), &["test"]), 0);
        state.fake("b", &t2);
        assert_eq!(state.run(&second, Some(&t2), &["check"]), 0);
        assert_eq!(state.seen("a")[1], state.seen("b")[1]);
        let slot = state.slot_target();
        assert!(
            slot.join("built-by-a").exists(),
            "the first run's file stays"
        );
        assert!(slot.join("built-by-b").exists());
        assert_eq!(state.seen("b")[0], "link");
    }

    #[test]
    fn unslotted_subcommands_neither_link_nor_touch_the_slot_directory() {
        let state = Linked::new();
        let job = state.job_dir("a");
        let target = job.join("target/produce");
        std::fs::create_dir_all(&target).unwrap();
        state.fake("a", &target);
        for sub in ["fmt", "metadata"] {
            assert_eq!(state.run(&job, Some(&target), &[sub]), 0);
            // The path is still the plain directory the job made, and the fake wrote into it.
            assert_eq!(state.seen("a")[0], "plain", "{sub}");
            assert!(!state.temp.path().join("build-slots").exists(), "{sub}");
            assert!(!state.temp.path().join("slots").exists(), "{sub}");
        }
        assert!(target.join("built-by-a").exists());
        assert!(!std::fs::symlink_metadata(&target).unwrap().is_symlink());
    }

    #[test]
    fn a_target_path_outside_the_job_is_refused_and_cargo_runs_unlinked() {
        let state = Linked::new();
        let job = state.job_dir("a");
        let outside = state.temp.path().join("elsewhere/target");
        std::fs::create_dir_all(&outside).unwrap();
        state.fake("a", &outside);
        assert_eq!(state.run(&job, Some(&outside), &["build"]), 0);
        // D46: not linked; cargo ran and wrote into the directory it was given.
        assert_eq!(state.seen("a")[0], "plain");
        assert!(outside.join("built-by-a").exists());
        assert!(!std::fs::symlink_metadata(&outside).unwrap().is_symlink());
        assert!(!state.slot_target().join("built-by-a").exists());
        // The same wrapper links a path inside the job.
        let inside = job.join("target/produce");
        state.fake("b", &inside);
        assert_eq!(state.run(&job, Some(&inside), &["build"]), 0);
        assert_eq!(state.seen("b")[0], "link");
        // No CARGO_TARGET_DIR at all: nothing to link, cargo still runs.
        state.fake("c", &inside);
        assert_eq!(state.run(&job, None, &["build"]), 0);
        assert_eq!(state.seen("c")[0], "plain");
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
