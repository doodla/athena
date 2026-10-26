//! Free disk space: the check before a job is dispatched and before its checks run.
//!
//! A full disk makes builds fail in ways that look like faults in the work. So `[disk] min_free_gb` (default
//! [`athena_data::config::DEFAULT_MIN_FREE_GB`]) is enforced in code at two places:
//!
//! - At dispatch, before anything is committed. If space is low, the build directories (`target/`) of idle jobs
//!   are removed; if it is still low, the job is held: it stays queued, a local marker records the amounts,
//!   and `go` returns [`LowDisk`]. A later drain picks up the same job.
//! - Before the checks. If space is still low after the same cleanup, the job fails with an `environment: low
//!   disk` reason, which `athena retry` handles. Nothing is left dispatched with a marker.
//!
//! A free amount that cannot be read never holds or fails a job.

use crate::state::{StateDir, format_bytes};
use crate::{Athena, Ui};
use athena_data::JobId;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The status text of a job held for lack of disk space.
pub const LOW_DISK: &str = "environment: low disk";

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

impl Athena {
    /// Checks free disk space against `required_gb`. When low, it removes idle jobs' build directories and
    /// reads again. Returns `None` when there is enough space (or it cannot be read), else the amounts that
    /// are still short. `job` is the job asking; its own build directory is kept.
    pub(crate) fn disk_gate(&self, required_gb: u64, job: &JobId, ui: &dyn Ui) -> Option<LowDisk> {
        let required = required_gb.saturating_mul(GB);
        let free = self.state.free_bytes()?;
        if free >= required {
            return None;
        }
        ui.line(&format!(
            "disk       {} free, {} required; removing build directories of closed and idle jobs",
            format_bytes(free),
            format_bytes(required)
        ));
        sweep_idle_targets(&self.state, job, ui);
        let free = self.state.free_bytes()?;
        if free >= required {
            ui.line(&format!(
                "disk       {} free after cleanup",
                format_bytes(free)
            ));
            return None;
        }
        Some(LowDisk { free, required })
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
