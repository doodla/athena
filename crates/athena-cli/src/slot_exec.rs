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

use athena_agent::env::{BuildLink, CargoShim};
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
    // The link lives until the child has exited; dropping it removes the link only.
    let _link = _slot
        .as_ref()
        .and_then(|slot| link_slot_target(&shim, slot.index(), lookup));
    match Command::new(&real).args(args).env("PATH", &path).status() {
        Ok(status) => exit_code(status),
        Err(error) => {
            eprintln!("cargo: running {}: {error}", real.display());
            EXIT_NOT_FOUND
        }
    }
}

/// The directory name, beside the slot lock directory, that holds the slots' warm target directories
/// (`StateDir::build_slots_dir`).
const BUILD_SLOTS_DIR_NAME: &str = "build-slots";

/// Links the job's `CARGO_TARGET_DIR` to slot `index`'s warm target directory, the one the checks use. A
/// failure, or no build path inside the job's directory (D46), is a warning and cargo runs unlinked.
fn link_slot_target(
    shim: &CargoShim,
    index: usize,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Option<BuildLink> {
    let path = lookup("CARGO_TARGET_DIR")?;
    let job_dir = shim.dir.parent()?;
    let base = shim.slots_dir.parent()?.join(BUILD_SLOTS_DIR_NAME);
    let target = base.join(index.to_string()).join("target");
    if let Err(e) = std::fs::create_dir_all(&target) {
        eprintln!(
            "cargo: warning: build slot {index} has no usable target directory: {}: {e}",
            target.display()
        );
        return None;
    }
    match BuildLink::create(job_dir, Path::new(&path), &target) {
        Ok(link) => Some(link),
        Err(e) => {
            eprintln!("cargo: warning: could not link the build slot: {e}");
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

    /// A throwaway state root with a fake `cargo` and a job directory per call of [`Fixture::job`].
    struct Fixture {
        temp: tempfile::TempDir,
    }

    /// One job: its directory, the build path the environment names, and the file the fake cargo reports to.
    struct Job {
        dir: PathBuf,
        target: PathBuf,
        report: PathBuf,
    }

    impl Fixture {
        fn new() -> Fixture {
            let temp = tempfile::tempdir().unwrap();
            std::fs::create_dir_all(temp.path().join("bin")).unwrap();
            std::fs::create_dir_all(temp.path().join("state/slots")).unwrap();
            Fixture { temp }
        }

        fn slots_dir(&self) -> PathBuf {
            self.temp.path().join("state/slots")
        }

        fn slot_target(&self, n: usize) -> PathBuf {
            self.temp
                .path()
                .join(format!("state/build-slots/{n}/target"))
        }

        /// A job whose fake cargo reports, while it runs, whether the build path is a link, where it
        /// resolves, and writes `marker` into the build path.
        fn job(&self, name: &str, target: Option<&Path>) -> Job {
            let dir = self.temp.path().join("jobs").join(name);
            std::fs::create_dir_all(dir.join("shim")).unwrap();
            let target = target.map_or_else(|| dir.join("target/produce"), Path::to_path_buf);
            let report = self.temp.path().join(format!("{name}.report"));
            let script = format!(
                "#!/bin/sh\n{{\nif test -L '{t}'; then echo link; else echo nolink; fi\nreadlink -f '{t}'\n}} > '{r}'\nif test -d '{t}'; then echo {name} > '{t}/{name}.txt'; fi\nexit 0\n",
                t = target.display(),
                r = report.display(),
            );
            let path = self.temp.path().join("bin/cargo");
            std::fs::write(&path, script).unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
            Job {
                dir,
                target,
                report,
            }
        }

        fn run(&self, job: &Job, args: &[&str]) -> u8 {
            let env = std::collections::BTreeMap::from([
                (
                    "ATHENA_CARGO_SHIM_DIR",
                    job.dir.join("shim").display().to_string(),
                ),
                ("ATHENA_SLOTS_DIR", self.slots_dir().display().to_string()),
                ("ATHENA_SLOTS", "2".to_string()),
                ("ATHENA_SLOT_WAIT_S", "5".to_string()),
                ("ATHENA_SLOT_PROMOTE_S", "60".to_string()),
                (
                    "ATHENA_SLOT_WAIT_NOTE",
                    job.dir.join("slot-wait.json").display().to_string(),
                ),
                (
                    "PATH",
                    format!(
                        "{}:{}:/usr/bin:/bin",
                        job.dir.join("shim").display(),
                        self.temp.path().join("bin").display()
                    ),
                ),
                ("CARGO_TARGET_DIR", job.target.display().to_string()),
            ]);
            let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
            run(&args, &|key| env.get(key).cloned())
        }
    }

    fn report(job: &Job) -> Vec<String> {
        std::fs::read_to_string(&job.report)
            .unwrap()
            .lines()
            .map(String::from)
            .collect()
    }

    #[test]
    fn a_slotted_build_sees_its_target_dir_linked_to_the_slot_and_the_link_goes_after() {
        let fx = Fixture::new();
        let job = fx.job("one", None);
        assert_eq!(fx.run(&job, &["build"]), 0);
        let seen = report(&job);
        assert_eq!(seen[0], "link");
        let resolved = fx.slot_target(0).canonicalize().unwrap();
        assert_eq!(Path::new(&seen[1]), resolved);
        assert!(
            std::fs::symlink_metadata(&job.target).is_err(),
            "the link is gone after the run"
        );
        assert!(fx.slot_target(0).join("one.txt").is_file());
    }

    #[test]
    fn two_runs_in_different_jobs_share_the_slot_target_and_its_files() {
        let fx = Fixture::new();
        // The fake cargo is rewritten for each job, so a job is made just before it runs.
        let a = fx.job("a", None);
        assert_eq!(fx.run(&a, &["test"]), 0);
        let b = fx.job("b", None);
        assert_eq!(fx.run(&b, &["clippy"]), 0);
        let resolved = fx.slot_target(0).canonicalize().unwrap();
        assert_eq!(Path::new(&report(&a)[1]), resolved);
        assert_eq!(Path::new(&report(&b)[1]), resolved);
        // The first run's file is still there for the second.
        assert!(fx.slot_target(0).join("a.txt").is_file());
        assert!(fx.slot_target(0).join("b.txt").is_file());
    }

    #[test]
    fn unslotted_subcommands_link_nothing_and_touch_no_slot_directory() {
        let fx = Fixture::new();
        let job = fx.job("u", None);
        for sub in ["fmt", "metadata"] {
            assert_eq!(fx.run(&job, &[sub]), 0);
            assert_eq!(report(&job)[0], "nolink", "{sub}");
            assert!(!job.target.exists(), "{sub}: CARGO_TARGET_DIR is unchanged");
            assert!(!fx.temp.path().join("state/build-slots").exists(), "{sub}");
            assert_eq!(
                std::fs::read_dir(fx.slots_dir()).unwrap().count(),
                0,
                "{sub}"
            );
        }
    }

    #[test]
    fn a_target_dir_outside_the_job_is_refused_and_one_inside_is_linked() {
        let fx = Fixture::new();
        let outside = fx.temp.path().join("elsewhere/target");
        std::fs::create_dir_all(&outside).unwrap();
        let job = fx.job("o", Some(&outside));
        assert_eq!(fx.run(&job, &["check"]), 0, "cargo still runs");
        assert_eq!(report(&job)[0], "nolink");
        assert!(outside.is_dir(), "the outside directory is left alone");
        assert!(!fx.slot_target(0).join("o.txt").exists());

        let inside = fx.job("i", None);
        assert_eq!(fx.run(&inside, &["check"]), 0);
        assert_eq!(report(&inside)[0], "link");
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
