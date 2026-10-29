//! Free disk space: the check before a job is dispatched, before its checks run and before its verifier runs.
//!
//! A full disk makes builds fail in ways that look like faults in the work. So `[disk] min_free_gb` (default
//! [`athena_data::config::DEFAULT_MIN_FREE_GB`]) is enforced in code at three places:
//!
//! - At dispatch, before anything is committed. If space is low, the build directories (`target/`) of idle jobs
//!   are removed; if it is still low, the job is held: it stays queued, a local marker records the amounts,
//!   and `go` returns [`LowDisk`]. A later drain picks up the same job.
//! - Before the checks. If space is still low after the same cleanup, the job fails with an `environment: low
//!   disk` reason, which `athena retry` handles. Nothing is left dispatched with a marker.
//! - Before the verifier agent runs, after the checks. The same gate runs again, and so does a write probe in
//!   the verifier's workspace ([`verifier_environment_problem`]). Failing either closes the job with the same
//!   `environment: low disk` reason and no verification verdict.
//!
//! Why the third place exists. The first two guards ran before the checks, but the checks are what fill the
//! disk: their builds write into the verifier's snapshot and the build directories. The disk guard did not
//! cover the point between the checks and the verifier, so the verifier started in an environment that could
//! not build or write, its failure read as a failure of the work, and the cause was found only by reading
//! logs. An environment failure there was not separated from a work failure. Now a verifier is never started
//! in such an environment, and the closing reason says it is the environment ([`is_environment_reason`]).
//!
//! A free amount that cannot be read never holds or fails a job. A failing write probe always does.
//!
//! A check killed at its time limit (`[budget] check_timeout_s`) is an environment stop too: a busy or slow
//! machine can push a long suite past the limit, and that says nothing about the work. The job closes with an
//! `environment: check timed out` reason ([`check_timeout_reason`]) that names the measured time and the
//! limit, and `athena retry` resumes it. A check that fails with an exit code is still a failure of the work.
//!
//! Freeing and resuming. The producer's build directory (`target/produce`) is idle once Athena's checks have
//! passed, since the checks build in their own directory; it is removed then, before the gate that runs
//! before the verifier, and kept when a check fails (the send-back needs it). A job that stops for an
//! environment cause after its candidate is built, or that is interrupted, leaves a resume mark
//! ([`crate::state::ResumeMark`]) outside its job directory; [`resumable_stop`] says which stopped jobs
//! `athena retry` resumes from that candidate, with no new planner or producer run.

use crate::state::{StateDir, format_bytes};
use crate::store::JobState;
use crate::{Athena, Ui};
use athena_data::JobId;
use athena_data::records::OutcomeState;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The status text of a job held for lack of disk space.
pub const LOW_DISK: &str = "environment: low disk";

/// The status text of a job stopped because one of its checks was killed at its time limit.
pub const CHECK_TIMEOUT: &str = "environment: check timed out";

/// Bytes in one gigabyte as `[disk] min_free_gb` counts them.
pub const GB: u64 = 1_000_000_000;

/// Dispatch was held: free disk space stayed below the minimum after cleanup. The job stays queued.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LowDisk {
    pub free: u64,
    pub required: u64,
}

impl LowDisk {
    /// "environment: low disk: 3.0 GB free, 10.0 GB required".
    pub fn text(&self) -> String {
        format!(
            "{LOW_DISK}: {} free, {} required",
            format_bytes(self.free),
            format_bytes(self.required)
        )
    }
}

impl std::fmt::Display for LowDisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}; the job stays queued and is not rejected: free disk space, then run it again",
            self.text()
        )
    }
}

impl std::error::Error for LowDisk {}

/// Test hook: when this environment variable holds a whole number, [`df_free_bytes`] returns it instead of
/// reading the disk. Tests set it on a spawned `athena` binary to show a run under simulated free space.
pub const FREE_BYTES_ENV: &str = "ATHENA_FREE_BYTES";

/// The simulated free bytes in `text` (the value of [`FREE_BYTES_ENV`]); anything else is ignored.
fn simulated_bytes(text: Option<&str>) -> Option<u64> {
    text?.trim().parse().ok()
}

/// Free bytes on the disk that holds `path` (or its nearest existing parent), read with `df -Pk`, unless
/// [`FREE_BYTES_ENV`] gives a simulated reading.
pub fn df_free_bytes(path: &Path) -> std::io::Result<u64> {
    if let Some(bytes) = simulated_bytes(std::env::var(FREE_BYTES_ENV).ok().as_deref()) {
        return Ok(bytes);
    }
    let mut at = path;
    while !at.exists() {
        match at.parent() {
            Some(parent) => at = parent,
            None => break,
        }
    }
    let out = std::process::Command::new("df")
        .arg("-Pk")
        .arg(at)
        .output()?;
    if !out.status.success() {
        return Err(std::io::Error::other("df failed"));
    }
    parse_df(&String::from_utf8_lossy(&out.stdout))
        .ok_or_else(|| std::io::Error::other("df output not understood"))
}

/// The available bytes in POSIX `df -Pk` output: the fourth column of the last line, in 1024-byte blocks.
fn parse_df(text: &str) -> Option<u64> {
    let line = text.lines().rfind(|l| !l.trim().is_empty())?;
    let kib: u64 = line.split_whitespace().nth(3)?.parse().ok()?;
    kib.checked_mul(1024)
}

/// Removes the `target/` of every job directory that no process holds the job lock of, except `keep`. Source
/// files, snapshots and records stay. Returns the bytes removed.
pub fn sweep_idle_targets(state: &StateDir, keep: &JobId, ui: &dyn Ui) -> u64 {
    let mut total = 0;
    for id in state.job_ids() {
        if &id == keep {
            continue;
        }
        // Held by a running command: its build directory is in use.
        let Ok(Some(_lock)) = state.try_lock_job(&id) else {
            continue;
        };
        match state.remove_target(&id) {
            Ok(0) => {}
            Ok(bytes) => {
                total += bytes;
                ui.line(&format!(
                    "disk       removed build directory of job {id}: {}",
                    format_bytes(bytes)
                ));
            }
            Err(why) => ui.line(&format!(
                "warning: could not remove the build directory of job {id}: {why}"
            )),
        }
    }
    total
}

/// Records that dispatch of `job` was held, with the amounts.
pub fn write_hold(state: &StateDir, job: &JobId, hold: &LowDisk) {
    let path = state.hold_file(job);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string(hold) {
        let _ = std::fs::write(path, text);
    }
}

/// The hold recorded for `job`, if any.
pub fn read_hold(state: &StateDir, job: &JobId) -> Option<LowDisk> {
    let text = std::fs::read_to_string(state.hold_file(job)).ok()?;
    serde_json::from_str(&text).ok()
}

/// Removes the hold marker of `job`.
pub fn clear_hold(state: &StateDir, job: &JobId) {
    let _ = std::fs::remove_file(state.hold_file(job));
}

/// The name of the small file [`write_probe`] writes and removes.
pub const PROBE_FILE: &str = ".athena-write-probe";

/// Writes a small file in `dir`, flushes it to the disk and removes it. A full disk, a missing or read-only
/// directory and a broken mount all fail here, with the error the system gave. The file is gone afterwards,
/// so a snapshot's tree is unchanged by a probe that succeeds.
pub fn write_probe(dir: &Path) -> std::io::Result<()> {
    use std::io::Write;
    let path = dir.join(PROBE_FILE);
    let mut file = std::fs::File::create(&path)?;
    let written = file
        .write_all(b"athena write probe\n")
        .and_then(|()| file.sync_all());
    drop(file);
    let removed = std::fs::remove_file(&path);
    written.and(removed)
}

/// The check before the verifier agent runs: the shared disk gate (with the idle-job cleanup) and then a
/// write probe in `workspace`, the verifier's snapshot. Returns `None` when the environment can build and
/// write, else the `environment: low disk` text that says what is wrong. `job` keeps its own build directory.
pub(crate) fn verifier_environment_problem(
    state: &StateDir,
    required_gb: u64,
    job: &JobId,
    workspace: &Path,
    ui: &dyn Ui,
) -> Option<String> {
    if let Some(low) = gate(state, required_gb, job, ui) {
        return Some(low.text());
    }
    write_probe(workspace)
        .err()
        .map(|why| format!("{LOW_DISK}: the verifier's workspace cannot be written to ({why})"))
}

/// The closing reason for a job whose verifier did not start because of `problem`
/// ([`verifier_environment_problem`]). It names the environment, not the work, and the retry command.
pub fn before_verifier_reason(problem: &str, job: &JobId) -> String {
    format!(
        "{problem} before the verifier; this is the environment, not the work. Free disk space, then `athena retry {job}`"
    )
}

/// The closing reason for a job whose check `name` was killed at its time limit: it ran `duration_s`
/// seconds against a limit of `limit_s` (`[budget] check_timeout_s`). A check cut off by the clock says
/// nothing about the work, so the reason names the environment, the measured time and the way forward.
pub fn check_timeout_reason(name: &str, duration_s: f64, limit_s: u64, job: &JobId) -> String {
    format!(
        "{CHECK_TIMEOUT}: {name} killed at {duration_s:.0}s of {limit_s}s limit; this is the environment, not the work. `athena retry {job}`, or raise [budget] check_timeout_s"
    )
}

/// True when a job's closing `reason` records a failure of the environment (low disk, an unwritable
/// workspace, or a check killed at its time limit), not a failure of the work. It holds for any job.
pub fn is_environment_reason(reason: &str) -> bool {
    reason.starts_with(LOW_DISK) || reason.starts_with(CHECK_TIMEOUT)
}

/// Whether a job that stopped is one `athena retry` may resume from its kept candidate: it closed failed
/// with an environment reason (`reason`, see [`is_environment_reason`]), or it is dispatched with no process
/// working on it (interrupted; the caller has checked that no process runs it). Whether a candidate was built
/// is a separate question: only a resume mark says so. A rejected job, a landed one and a failure of the work
/// are never resumed.
pub fn resumable_stop(state: JobState, reason: Option<&str>) -> bool {
    match state {
        JobState::Closed(OutcomeState::Failed) => reason.is_some_and(is_environment_reason),
        JobState::Dispatched => true,
        _ => false,
    }
}

/// Checks free disk space against `required_gb`. When low, it removes idle jobs' build directories and
/// reads again. Returns `None` when there is enough space (or it cannot be read), else the amounts that
/// are still short. `job` is the job asking; its own build directory is kept.
fn gate(state: &StateDir, required_gb: u64, job: &JobId, ui: &dyn Ui) -> Option<LowDisk> {
    let required = required_gb.saturating_mul(GB);
    let free = state.free_bytes()?;
    if free >= required {
        return None;
    }
    ui.line(&format!(
        "disk       {} free, {} required; removing build directories of closed and idle jobs",
        format_bytes(free),
        format_bytes(required)
    ));
    sweep_idle_targets(state, job, ui);
    let free = state.free_bytes()?;
    if free >= required {
        ui.line(&format!(
            "disk       {} free after cleanup",
            format_bytes(free)
        ));
        return None;
    }
    Some(LowDisk { free, required })
}

impl Athena {
    /// Checks free disk space against `required_gb`: the one gate that dispatch, the checks and the verifier
    /// all use. See [`gate`].
    pub(crate) fn disk_gate(&self, required_gb: u64, job: &JobId, ui: &dyn Ui) -> Option<LowDisk> {
        gate(&self.state, required_gb, job, ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct Said(Mutex<Vec<String>>);

    impl Ui for Said {
        fn line(&self, text: &str) {
            self.0.lock().unwrap().push(text.to_string());
        }
    }

    fn id(n: u32) -> JobId {
        JobId::parse(&format!("20261026-00{n:02}00-aaa{n}")).unwrap()
    }

    #[test]
    fn df_output_is_read_as_available_kibibytes() {
        let text = "Filesystem 1024-blocks Used Available Capacity Mounted on\n/dev/x 1000 400 600 40% /\n";
        assert_eq!(parse_df(text), Some(600 * 1024));
        assert_eq!(parse_df(""), None);
        assert_eq!(parse_df("header only\n/dev/x 1 2\n"), None);
    }

    #[test]
    fn a_simulated_reading_is_used_only_when_it_is_a_whole_number() {
        assert_eq!(simulated_bytes(Some("9000000000")), Some(9 * GB));
        assert_eq!(simulated_bytes(Some(" 0\n")), Some(0));
        assert_eq!(simulated_bytes(Some("lots")), None);
        assert_eq!(simulated_bytes(Some("")), None);
        assert_eq!(simulated_bytes(None), None);
    }

    #[test]
    fn the_real_reader_reports_some_space_for_a_real_directory() {
        let temp = tempfile::tempdir().unwrap();
        // A path that does not exist yet reads the disk of its nearest existing parent.
        assert!(df_free_bytes(&temp.path().join("not/yet")).unwrap() > 0);
    }

    #[test]
    fn the_sweep_removes_target_keeps_source_and_skips_locked_jobs() {
        let temp = tempfile::tempdir().unwrap();
        let state = StateDir::new(temp.path().to_path_buf());
        let (idle, locked, bare, own) = (id(1), id(2), id(3), id(4));
        for job in [&idle, &locked, &own] {
            let dir = state.job_dir(job);
            std::fs::create_dir_all(dir.join("target/debug")).unwrap();
            std::fs::write(dir.join("target/debug/big"), vec![1u8; 50_000]).unwrap();
            std::fs::create_dir_all(dir.join("work")).unwrap();
            std::fs::write(dir.join("work/main.rs"), b"fn main() {}").unwrap();
        }
        std::fs::create_dir_all(state.job_dir(&bare).join("work")).unwrap();
        std::fs::write(state.job_dir(&bare).join("work/lib.rs"), b"x").unwrap();

        let _held = state.try_lock_job(&locked).unwrap().expect("lock is free");
        let ui = Said::default();
        let removed = sweep_idle_targets(&state, &own, &ui);

        assert!(removed >= 50_000, "{removed}");
        assert!(!state.job_dir(&idle).join("target").exists());
        assert_eq!(
            std::fs::read(state.job_dir(&idle).join("work/main.rs")).unwrap(),
            b"fn main() {}",
            "source is kept"
        );
        assert!(
            state.job_dir(&locked).join("target/debug/big").exists(),
            "a locked job's build directory is kept"
        );
        assert!(
            state.job_dir(&own).join("target").exists(),
            "the asking job's own build directory is kept"
        );
        assert!(state.job_dir(&bare).join("work/lib.rs").exists());
        let said = ui.0.lock().unwrap().join("\n");
        assert!(said.contains(idle.as_str()), "{said}");
        assert!(!said.contains(locked.as_str()), "{said}");
    }

    fn plenty(_: &Path) -> std::io::Result<u64> {
        Ok(500 * GB)
    }

    fn short(_: &Path) -> std::io::Result<u64> {
        Ok(3 * GB)
    }

    #[test]
    fn the_write_probe_succeeds_in_a_normal_workspace_and_leaves_nothing_behind() {
        let temp = tempfile::tempdir().unwrap();
        write_probe(temp.path()).unwrap();
        assert!(!temp.path().join(PROBE_FILE).exists());
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn the_write_probe_fails_where_nothing_can_be_written() {
        let temp = tempfile::tempdir().unwrap();
        // A workspace that is gone, and one that is a file: neither can hold a new file, even for root.
        assert!(write_probe(&temp.path().join("gone")).is_err());
        let file = temp.path().join("plain");
        std::fs::write(&file, b"x").unwrap();
        assert!(write_probe(&file).is_err());
        assert_eq!(
            std::fs::read(&file).unwrap(),
            b"x",
            "nothing else is touched"
        );
    }

    #[test]
    fn the_verifier_environment_check_fails_as_the_environment_for_any_job() {
        let temp = tempfile::tempdir().unwrap();
        let workspace = temp.path().join("verify");
        std::fs::create_dir_all(&workspace).unwrap();
        let roomy = StateDir::new(temp.path().to_path_buf()).with_free_space(plenty);
        let tight = StateDir::new(temp.path().to_path_buf()).with_free_space(short);

        for n in 1..=3 {
            let job = id(n);
            let ui = Said::default();
            // Enough space and a writable workspace: the verifier may start.
            assert_eq!(
                verifier_environment_problem(&roomy, 10, &job, &workspace, &ui),
                None
            );

            // Too little space after cleanup: the environment's fault, with the amounts.
            let low = verifier_environment_problem(&tight, 10, &job, &workspace, &ui)
                .expect("low space is a problem");
            assert_eq!(low, "environment: low disk: 3.0 GB free, 10.0 GB required");

            // An unwritable workspace is the environment's fault too, whatever the free space says.
            let broken = temp.path().join("not-there");
            let unwritable = verifier_environment_problem(&roomy, 10, &job, &broken, &ui)
                .expect("an unwritable workspace is a problem");
            assert!(unwritable.starts_with(LOW_DISK), "{unwritable}");

            for problem in [low, unwritable] {
                let reason = before_verifier_reason(&problem, &job);
                assert!(is_environment_reason(&reason), "{reason}");
                assert!(reason.contains(&format!("athena retry {job}")), "{reason}");
                assert!(reason.contains("not the work"), "{reason}");
            }
        }
        // A work failure is not mistaken for an environment one.
        assert!(!is_environment_reason("admission refused the change: x"));
        assert!(!is_environment_reason(
            "the verifier returned no valid verdict"
        ));
    }

    #[test]
    fn a_check_timeout_reason_is_an_environment_reason() {
        let job = id(1);
        let reason = check_timeout_reason("test", 905.0, 900, &job);
        assert!(reason.starts_with(CHECK_TIMEOUT), "{reason}");
        assert!(is_environment_reason(&reason), "{reason}");
        assert!(
            reason.contains("test killed at 905s of 900s limit"),
            "{reason}"
        );
        assert!(reason.contains("not the work"), "{reason}");
        assert!(reason.contains(&format!("athena retry {job}")), "{reason}");
        assert!(resumable_stop(
            JobState::Closed(OutcomeState::Failed),
            Some(&reason)
        ));
        assert!(!resumable_stop(
            JobState::Closed(OutcomeState::Rejected),
            Some(&reason)
        ));
        // A failed check is a failure of the work, not the environment.
        assert!(!is_environment_reason("check test failed"));
        assert!(!is_environment_reason("admission refused the change: x"));
    }

    #[test]
    fn only_an_environment_failure_or_an_interrupted_job_is_resumable() {
        let low = before_verifier_reason(LOW_DISK, &id(1));
        let failed = JobState::Closed(OutcomeState::Failed);
        assert!(resumable_stop(failed, Some(&low)));
        assert!(!resumable_stop(
            failed,
            Some("admission refused the change: x")
        ));
        assert!(!resumable_stop(failed, None));
        // An interrupted job has no outcome and so no reason.
        assert!(resumable_stop(JobState::Dispatched, None));
        for state in [
            OutcomeState::Rejected,
            OutcomeState::Landed,
            OutcomeState::TimedOut,
        ] {
            assert!(!resumable_stop(JobState::Closed(state), Some(&low)));
        }
        assert!(!resumable_stop(JobState::Proposed, None));
    }

    #[test]
    fn a_hold_marker_round_trips_and_clears() {
        let temp = tempfile::tempdir().unwrap();
        let state = StateDir::new(temp.path().to_path_buf());
        let job = id(1);
        assert_eq!(read_hold(&state, &job), None);
        let hold = LowDisk {
            free: 3 * GB,
            required: 10 * GB,
        };
        write_hold(&state, &job, &hold);
        assert_eq!(read_hold(&state, &job), Some(hold));
        clear_hold(&state, &job);
        assert_eq!(read_hold(&state, &job), None);
        let text = hold.text();
        assert_eq!(text, "environment: low disk: 3.0 GB free, 10.0 GB required");
    }
}
