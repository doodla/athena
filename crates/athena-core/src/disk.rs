//! Free disk space: the check before a job is dispatched, before its checks run and before its verifier runs.
//!
//! A full disk makes builds fail in ways that look like faults in the work. So `[disk] min_free_gb` (default
//! [`athena_data::config::DEFAULT_MIN_FREE_GB`]) is enforced in code at three places:
//!
//! - At dispatch, before anything is committed. If space is low, the build directories (`target/`) of idle jobs
//!   are removed; if it is still low, the job is held: it stays queued, a local marker records the amounts,
//!   and `go` returns [`LowDisk`]. A later drain picks up the same job.
//! - Before the checks, once the candidate is built. If space is still low after the same cleanup, the job is
//!   held, not failed: its records (candidate, invocations, plans) are committed with no outcome and no
//!   score, a marker records the amounts, and `go` returns [`LowDisk`]. The job stays dispatched with no
//!   process, which reads as interrupted, so it is no work failure for introspection, failure classes or the
//!   scope guard.
//! - Before the verifier agent runs, after the checks. The same gate runs again, and so does a write probe in
//!   the verifier's workspace ([`verifier_environment_problem`]). Failing either holds the job the same way,
//!   with no verification verdict. A workspace that takes no write has no usable space, so a failed probe
//!   is held with 0 bytes free, and the probe's error is printed first.
//!
//! Hold and resume. A held job needs no `athena retry` from a person. The drain lists a dispatched job with
//! no process as interrupted and retries it by itself on a later pass, once space is free; the retry resumes
//! from the kept candidate ([`crate::state::ResumeMark`]) with no planner or producer run. That skips the
//! rebuild only while the files agents see on the main branch are those the job's base had (only `records/`
//! differs); if another job landed in between, the retry plans and produces again, as any retry does. Space
//! that stays short leaves the job held again, with no attempt used.
//!
//! Why the third place exists. The first two guards ran before the checks, but the checks are what fill the
//! disk: their builds write into the verifier's snapshot and the build directories. The disk guard did not
//! cover the point between the checks and the verifier, so the verifier started in an environment that could
//! not build or write, its failure read as a failure of the work, and the cause was found only by reading
//! logs. Now a verifier is never started in such an environment.
//!
//! A free amount that cannot be read never holds a job. A failing write probe always does. Jobs that closed
//! failed with an `environment: low disk` reason before this change are still told apart from failures of
//! the work ([`is_environment_reason`]) and `athena retry` still resumes them ([`resumable_stop`]).
//!
//! A fourth environment stop sits beside these: a check killed at its deadline, when no check failed for
//! real. A near-limit run on a loaded machine says nothing about the work, so the job closes `environment:
//! check timed out` with the measured duration and the limit ([`check_timeout_reason`]), and `athena retry`
//! resumes it. A check that fails with its own exit code is still a failure of the work.
//!
//! Freeing and resuming. The producer's build directory (`target/produce`) is idle once Athena's checks have
//! passed, since the checks build in their own directory; it is removed then, before the gate that runs
//! before the verifier, and kept when a check fails (the send-back needs it). A job that stops for an
//! environment cause after its candidate is built, or that is interrupted, leaves a resume mark
//! ([`crate::state::ResumeMark`]) outside its job directory; [`resumable_stop`] says which stopped jobs
//! `athena retry` resumes from that candidate, with no new planner or producer run.
//!
//! Sizing the disk needs numbers, and a restricted producer cannot take them. `athena measure-build`
//! ([`Athena::measure_build`]) builds the workspace cold in two separate git worktrees with different target
//! directories and commits an `athena.build-measurement/1` record: T (one cold target directory), S (one
//! worktree's tracked files), the wall time of each build, and the `sccache` hit and miss counts of each build
//! or that `sccache` is not installed. The cargo and `sccache` work sits behind [`CargoBuilder`], so tests give
//! it a fake. The Hestia sweep runs the measurement when there is no record and again when `Cargo.lock`
//! changes ([`crate::hestia::measure_build_due`]), and `athena status` shows the latest.
//!
//! The in-flight bound (design `async-drain.md` section 5, build job 4). Each job's producer has its own
//! worktree and build directory ([`crate::state::JobWorktree`]), so several jobs in flight multiply the disk
//! they use. With `[drain.pipeline] max_in_flight` above 1, a job's start waits when the disk fits fewer jobs:
//! `floor((free - reserve_disk_gb - max_check_slots x T) / (T + S))` ([`in_flight_limit`]), with T and S from
//! the latest build measurement or, with none, the design's estimates ([`Figures`]). A wait is the existing
//! hold ([`LowDisk`]): the job stays queued and a later drain starts it ([`start_decision`]). With
//! `max_in_flight` 1, or with no other job in flight, nothing is held by the bound.

use crate::commits::{NewFile, message};
use crate::state::{StateDir, format_bytes};
use crate::store::{JobState, Records};
use crate::{Athena, Ui};
use athena_data::records::{
    AnyRecord, BuildMeasurement, OutcomeState, SccacheCounts, SccacheFigures,
};
use athena_data::{JobId, RecordId};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The status text of a job held for lack of disk space.
pub const LOW_DISK: &str = "environment: low disk";

/// The status text of a job closed because a check was killed at its deadline.
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
/// write, else the hold that says what is wrong. A workspace that takes no write has no usable space, so a
/// failed probe is held with 0 bytes free (the probe error is written to `ui` first). `job` keeps its own
/// build directory.
pub(crate) fn verifier_environment_problem(
    state: &StateDir,
    required_gb: u64,
    job: &JobId,
    workspace: &Path,
    ui: &dyn Ui,
) -> Option<LowDisk> {
    if let Some(low) = gate(state, required_gb, job, ui) {
        return Some(low);
    }
    write_probe(workspace).err().map(|why| {
        ui.line(&format!(
            "disk       the verifier's workspace cannot be written to: {why}"
        ));
        LowDisk {
            free: 0,
            required: required_gb.saturating_mul(GB),
        }
    })
}

/// The closing reason for a job whose check was killed at its deadline and whose other checks did not fail
/// for real. It names the check, the measured duration against the limit, and the retry command. A slow or
/// loaded machine is the usual cause, so it is held as the environment, not rejected as the work.
pub fn check_timeout_reason(name: &str, took_s: u64, limit_s: u64, job: &JobId) -> String {
    format!(
        "{CHECK_TIMEOUT}: {name} ran {took_s}s of {limit_s}s limit; this is the environment, not the work. Free the machine or rerun, then `athena retry {job}`"
    )
}

/// True when a job's closing `reason` records a failure of the environment (low disk, an unwritable
/// workspace or a check killed at its deadline), not a failure of the work. It holds for any job.
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

// ---------------------------------------------------------------------------------------------------------
// The in-flight bound (docs/design/async-drain.md section 5)

/// T when no build measurement exists: the design's estimate of one cold target directory, 6 GB.
pub const ESTIMATE_T_BYTES: u64 = 6 * GB;

/// S when no build measurement exists: the design's estimate of one worktree's tracked files, 0.2 GB.
pub const ESTIMATE_S_BYTES: u64 = GB / 5;

/// Where the figures of the bound came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FigureSource {
    /// The latest `athena.build-measurement/1` record, by its id (`athena measure-build` writes it).
    Measured(String),
    /// No record exists: the design's estimates ([`ESTIMATE_T_BYTES`], [`ESTIMATE_S_BYTES`]).
    DesignEstimate,
}

impl FigureSource {
    /// "build measurement 20261029-090000-5abc" or "design estimate".
    pub fn text(&self) -> String {
        match self {
            FigureSource::Measured(id) => format!("build measurement {id}"),
            FigureSource::DesignEstimate => "design estimate".to_string(),
        }
    }
}

/// The two sizes the in-flight bound is computed from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Figures {
    /// T: bytes of one cold target directory.
    pub t: u64,
    /// S: bytes of one worktree's tracked files.
    pub s: u64,
    pub source: FigureSource,
}

impl Figures {
    /// The figures of the latest measurement (see [`BuildMeasurement::latest`]), or the design estimates when
    /// there is none.
    pub fn from_latest(latest: Option<&BuildMeasurement>) -> Figures {
        match latest {
            Some(m) => Figures {
                t: m.target_bytes,
                s: m.source_bytes,
                source: FigureSource::Measured(m.id.to_string()),
            },
            None => Figures {
                t: ESTIMATE_T_BYTES,
                s: ESTIMATE_S_BYTES,
                source: FigureSource::DesignEstimate,
            },
        }
    }
}

/// How many jobs the disk holds at once (design section 5):
/// `floor((free - reserve_disk_gb - max_check_slots x T) / (T + S))`, with `free` the free bytes before any
/// of those jobs is placed. A disk below the reserve and the check slots holds none, so it is 0, never
/// negative. With T + S = 0 a job costs nothing and the count is unbounded.
pub fn in_flight_limit(
    free_bytes: u64,
    reserve_gb: u64,
    check_slots: u32,
    figures: &Figures,
) -> u64 {
    let kept = reserve_gb
        .saturating_mul(GB)
        .saturating_add(u64::from(check_slots).saturating_mul(figures.t));
    free_bytes
        .saturating_sub(kept)
        .checked_div(figures.t.saturating_add(figures.s))
        .unwrap_or(u64::MAX)
}

/// Whether a job must wait to start, given the pipeline settings and what the disk holds. `None` starts it.
///
/// - With `max_in_flight` 1 nothing is checked: today's one-job-at-a-time run, which `[disk] min_free_gb`
///   already guards.
/// - With no other job in flight nothing is checked either: waiting would free nothing, so a held lone job
///   could never start.
/// - Otherwise the job waits when the disk fits fewer jobs than `max_in_flight` ([`in_flight_limit`]) and
///   `others` jobs are already in flight, the bound or more.
///
/// `free` is the free bytes now and `held` the bytes the other jobs' worktrees and build directories
/// already use, which the bound counts as free because it assumes no job is placed yet. The returned
/// [`LowDisk`] names the free bytes now and the free bytes `others + 1` jobs need.
pub fn start_decision(
    pipeline: &athena_data::config::PipelineConfig,
    figures: &Figures,
    free: u64,
    held: u64,
    others: usize,
) -> Option<LowDisk> {
    if pipeline.max_in_flight <= 1 || others == 0 {
        return None;
    }
    let limit = in_flight_limit(
        free.saturating_add(held),
        pipeline.reserve_disk_gb,
        pipeline.max_check_slots,
        figures,
    );
    if limit >= u64::from(pipeline.max_in_flight) || (others as u64) < limit {
        return None;
    }
    let per_job = figures.t.saturating_add(figures.s);
    let needed = pipeline
        .reserve_disk_gb
        .saturating_mul(GB)
        .saturating_add(u64::from(pipeline.max_check_slots).saturating_mul(figures.t))
        .saturating_add((others as u64 + 1).saturating_mul(per_job));
    Some(LowDisk {
        free,
        required: needed.saturating_sub(held),
    })
}

impl Athena {
    /// Whether the start of `id` must wait for a running job to end, by the in-flight bound
    /// ([`start_decision`]). T and S come from the latest build measurement in `records`, or the design
    /// estimates; the reserve, the check slots and `max_in_flight` come from `[drain.pipeline]`. `None` starts
    /// the job. A free amount that cannot be read never holds a job. On a hold the reason is written to `ui`.
    ///
    /// This is a gate, not a lock. Another job counts as in flight when its job lock is held and it has a
    /// worktree or build directory ([`StateDir::other_jobs_in_flight`]), and the probe locks that job for
    /// microseconds. Two jobs that start at the same instant can both pass.
    pub(crate) fn start_hold(
        &self,
        jc: &crate::capability::JobConfig,
        records: &Records,
        id: &JobId,
        ui: &dyn Ui,
    ) -> Option<LowDisk> {
        let pipeline = jc.config.pipeline();
        if pipeline.max_in_flight <= 1 {
            return None;
        }
        let others = self.state.other_jobs_in_flight(id);
        if others.is_empty() {
            return None;
        }
        let free = self.state.free_bytes()?;
        let held: u64 = others
            .iter()
            .map(|other| {
                dir_bytes(&self.state.work_dir(other)) + dir_bytes(&self.state.produce_dir(other))
            })
            .sum();
        let figures = Figures::from_latest(BuildMeasurement::latest(&records.build_measurements));
        let low = start_decision(&pipeline, &figures, free, held, others.len())?;
        ui.line(&format!(
            "disk       start held: {} other job(s) in flight, which uses {} of the disk; with {} free, \
             {} GB reserved, {} check slot(s), T {} and S {} ({}), fewer than {} jobs fit",
            others.len(),
            format_bytes(held),
            format_bytes(free),
            pipeline.reserve_disk_gb,
            pipeline.max_check_slots,
            format_bytes(figures.t),
            format_bytes(figures.s),
            figures.source.text(),
            pipeline.max_in_flight,
        ));
        Some(low)
    }
}

// ---------------------------------------------------------------------------------------------------------
// The build measurement (`athena measure-build`)

/// What one cold build took, as the builder saw it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuildRun {
    /// Wall time of the build, in seconds.
    pub seconds: f64,
    /// The compile-cache counts of this build alone; `None` when no cache was in use.
    pub sccache: Option<SccacheCounts>,
}

/// The part of a measurement that runs cargo and `sccache`. [`Athena::measure_build`] does everything else
/// (the worktrees, the sizes, the record), so a test can supply a builder that only returns fixed numbers.
pub trait CargoBuilder {
    /// Whether `sccache` is installed. When it is not, both builds run without a compile cache.
    fn sccache_installed(&self) -> bool;
    /// Builds the whole workspace cold in `worktree` with `target_dir` as the build directory, and reports
    /// the wall time and, when `sccache` is installed, the cache counts of this build alone.
    fn build(&self, worktree: &Path, target_dir: &Path) -> anyhow::Result<BuildRun>;
}

/// The real builder: `cargo build --locked --workspace --all-targets`, through `sccache` when it is installed.
#[derive(Debug, Clone, Copy, Default)]
pub struct CargoBuild;

impl CargoBuilder for CargoBuild {
    fn sccache_installed(&self) -> bool {
        std::process::Command::new("sccache")
            .arg("--version")
            .output()
            .is_ok_and(|o| o.status.success())
    }

    fn build(&self, worktree: &Path, target_dir: &Path) -> anyhow::Result<BuildRun> {
        use anyhow::Context;
        let cached = self.sccache_installed();
        let mut cargo = std::process::Command::new("cargo");
        cargo
            .args(["build", "--locked", "--workspace", "--all-targets"])
            .current_dir(worktree)
            .env("CARGO_TARGET_DIR", target_dir);
        if cached {
            // Counts start from zero, so they belong to this build alone; incremental builds are not cached.
            let zeroed = std::process::Command::new("sccache")
                .arg("--zero-stats")
                .output()
                .context("running `sccache --zero-stats`")?;
            if !zeroed.status.success() {
                anyhow::bail!("`sccache --zero-stats` failed");
            }
            cargo
                .env("RUSTC_WRAPPER", "sccache")
                .env("CARGO_INCREMENTAL", "0");
        } else {
            // No cache is installed, so no wrapper the environment names may stand in for one.
            cargo
                .env_remove("RUSTC_WRAPPER")
                .env_remove("CARGO_BUILD_RUSTC_WRAPPER");
        }
        let started = std::time::Instant::now();
        let out = cargo.output().context("running `cargo build`")?;
        let seconds = started.elapsed().as_secs_f64();
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            let tail: Vec<&str> = stderr.lines().rev().take(20).collect();
            anyhow::bail!(
                "`cargo build` failed in {}:\n{}",
                worktree.display(),
                tail.into_iter().rev().collect::<Vec<_>>().join("\n")
            );
        }
        let sccache = if cached {
            let stats = std::process::Command::new("sccache")
                .arg("--show-stats")
                .output()
                .context("running `sccache --show-stats`")?;
            let text = String::from_utf8_lossy(&stats.stdout);
            Some(parse_sccache_stats(&text).ok_or_else(|| {
                anyhow::anyhow!(
                    "sccache is installed but the hit and miss counts in `sccache --show-stats` could not be read"
                )
            })?)
        } else {
            None
        };
        Ok(BuildRun { seconds, sccache })
    }
}

/// The hit and miss counts in the text of `sccache --show-stats`: the lines `Cache hits` and `Cache misses`
/// followed by a whole number. The per-language lines (`Cache hits (Rust)`) are not the totals and are
/// skipped. `None` when either total is missing.
pub fn parse_sccache_stats(text: &str) -> Option<SccacheCounts> {
    let count = |label: &str| -> Option<u64> {
        text.lines().find_map(|line| {
            let rest = line.trim().strip_prefix(label)?;
            // The label must end here: "Cache hits (Rust)" and "Cache hits rate" are other lines.
            if !rest.starts_with(char::is_whitespace) {
                return None;
            }
            rest.trim().parse().ok()
        })
    };
    Some(SccacheCounts {
        hits: count("Cache hits")?,
        misses: count("Cache misses")?,
    })
}

/// The bytes in the files under `dir`, counting each file's length and following no symbolic link. A
/// directory that cannot be read counts nothing.
pub fn dir_bytes(dir: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| match entry.file_type() {
            Ok(kind) if kind.is_dir() => dir_bytes(&entry.path()),
            Ok(kind) if kind.is_file() => entry.metadata().map(|m| m.len()).unwrap_or(0),
            _ => 0,
        })
        .sum()
}

/// The bytes of the files git tracks in the work tree at `worktree`.
fn source_bytes(worktree: &Path) -> anyhow::Result<u64> {
    let tracked = athena_git::Git::at(worktree).ls_files()?;
    Ok(tracked
        .iter()
        .filter_map(|path| std::fs::symlink_metadata(worktree.join(path)).ok())
        .filter(|meta| meta.is_file())
        .map(|meta| meta.len())
        .sum())
}

/// The git blob id of `Cargo.lock` at `commit`; `None` when that commit has no `Cargo.lock`.
pub(crate) fn cargo_lock_blob(
    git: &athena_git::Git,
    commit: &str,
) -> anyhow::Result<Option<String>> {
    Ok(git
        .ls_tree_under(commit, "Cargo.lock")?
        .into_iter()
        .find(|e| e.kind == "blob" && e.path == "Cargo.lock")
        .map(|e| e.oid))
}

/// The scratch worktrees of one measurement. Dropping it removes them and the build directories, so every
/// way out of a measurement (success, a failed build, an early error) leaves nothing behind.
struct Scratch<'a> {
    git: &'a athena_git::Git,
    root: std::path::PathBuf,
}

impl Scratch<'_> {
    fn worktree(&self, name: &str) -> std::path::PathBuf {
        self.root.join(name)
    }

    fn target(&self, name: &str) -> std::path::PathBuf {
        self.root.join(format!("target-{name}"))
    }

    fn remove(&self) {
        use std::ffi::OsStr;
        for name in ["a", "b"] {
            let dir = self.worktree(name);
            if dir.exists() {
                let _ = self.git.run([
                    OsStr::new("worktree"),
                    OsStr::new("remove"),
                    OsStr::new("--force"),
                    dir.as_os_str(),
                ]);
            }
        }
        let _ = std::fs::remove_dir_all(&self.root);
        let _ = self.git.run(["worktree", "prune"]);
    }
}

impl Drop for Scratch<'_> {
    fn drop(&mut self) {
        self.remove();
    }
}

impl Athena {
    /// Builds the workspace cold in two separate git worktrees of HEAD, each with its own target directory,
    /// and commits the result as an `athena.build-measurement/1` record: T (the bytes of one cold target
    /// directory), S (the bytes of one worktree's tracked files), the wall time of each build and the
    /// `sccache` hit and miss counts of each build, or that `sccache` is not installed. The second build
    /// shows whether a compile cache answers across worktrees. The worktrees and target directories live
    /// under the state directory and are removed on every path. Fails, writing no record, when HEAD has no
    /// `Cargo.lock`, when a build fails, or when `sccache` is installed but gave no counts.
    pub fn measure_build(
        &self,
        builder: &dyn CargoBuilder,
        ui: &dyn Ui,
    ) -> anyhow::Result<BuildMeasurement> {
        use std::ffi::OsStr;
        let head = self.head()?;
        let Some(lock_hash) = cargo_lock_blob(&self.git, &head)? else {
            anyhow::bail!(
                "{} has no Cargo.lock, so there is no workspace to build",
                &head[..12]
            );
        };
        let cached = builder.sccache_installed();
        let scratch = Scratch {
            git: &self.git,
            root: self.state.root().join("measure-build"),
        };
        // A run that was killed may have left its worktrees behind.
        scratch.remove();
        std::fs::create_dir_all(&scratch.root)?;

        let mut runs = Vec::new();
        for name in ["a", "b"] {
            let worktree = scratch.worktree(name);
            self.git.run([
                OsStr::new("worktree"),
                OsStr::new("add"),
                OsStr::new("--detach"),
                worktree.as_os_str(),
                OsStr::new(&head),
            ])?;
            ui.line(&format!(
                "measure    cold build {} of 2 in its own worktree and target directory",
                runs.len() + 1
            ));
            runs.push(builder.build(&worktree, &scratch.target(name))?);
        }
        let target_bytes = dir_bytes(&scratch.target("a"));
        let source_bytes = source_bytes(&scratch.worktree("a"))?;
        let (first, second) = (runs[0], runs[1]);
        let sccache = if cached {
            match (first.sccache, second.sccache) {
                (Some(first), Some(second)) => SccacheFigures::Measured { first, second },
                _ => anyhow::bail!(
                    "sccache is installed but a build gave no hit and miss counts; no record was written"
                ),
            }
        } else {
            SccacheFigures::NotInstalled
        };
        drop(scratch);

        let now = crate::now();
        let known = Records::load(&self.git, &self.head()?)?;
        let mut id = RecordId::generate(now);
        while known.has_record_id(&id) {
            id = RecordId::generate(now);
        }
        let measurement = BuildMeasurement {
            id,
            at: now,
            commit: head,
            cargo_lock_hash: lock_hash,
            target_bytes,
            source_bytes,
            first_build_seconds: first.seconds,
            second_build_seconds: second.seconds,
            sccache,
        };
        measurement.validate().map_err(|e| anyhow::anyhow!(e))?;
        let record = AnyRecord::BuildMeasurement(Box::new(measurement.clone()));
        self.commit_records(
            &[NewFile::record(&record)],
            &message(
                &format!(
                    "Build measurement: target {}, source {}",
                    format_bytes(measurement.target_bytes),
                    format_bytes(measurement.source_bytes)
                ),
                "A cold workspace build measured in two separate worktrees with different target directories: \
                 the size of one target directory, the size of the sources, the wall time of each build and the \
                 compile-cache counts, or that no compile cache is installed.",
                &[("Athena-Record", record.canonical_path())],
            ),
        )?;
        ui.line(&crate::status::build_measurement_line(Some(&measurement)));
        Ok(measurement)
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
    fn the_verifier_environment_check_holds_the_job_for_any_job() {
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

            // Too little space after cleanup: a hold with the amounts.
            let low = verifier_environment_problem(&tight, 10, &job, &workspace, &ui)
                .expect("low space is a problem");
            assert_eq!(
                low,
                LowDisk {
                    free: 3 * GB,
                    required: 10 * GB
                }
            );
            assert_eq!(
                low.text(),
                "environment: low disk: 3.0 GB free, 10.0 GB required"
            );

            // An unwritable workspace is held too, whatever the free space says: it has 0 usable bytes, and
            // the probe error is said first.
            let broken = temp.path().join("not-there");
            let before = ui.0.lock().unwrap().len();
            let unwritable = verifier_environment_problem(&roomy, 10, &job, &broken, &ui)
                .expect("an unwritable workspace is a problem");
            assert_eq!(
                unwritable,
                LowDisk {
                    free: 0,
                    required: 10 * GB
                }
            );
            let said = ui.0.lock().unwrap();
            assert!(
                said[before..]
                    .iter()
                    .any(|l| l.contains("workspace cannot be written to")),
                "{said:?}"
            );
            assert!(unwritable.text().starts_with(LOW_DISK));
        }
        // A work failure is not mistaken for an environment one.
        assert!(!is_environment_reason("admission refused the change: x"));
        assert!(!is_environment_reason(
            "the verifier returned no valid verdict"
        ));
    }

    #[test]
    fn the_check_timeout_reason_is_an_environment_reason() {
        let job = id(1);
        let reason = check_timeout_reason("test", 1805, 1800, &job);
        assert!(is_environment_reason(&reason), "{reason}");
        assert!(is_environment_reason(CHECK_TIMEOUT));
        assert!(CHECK_TIMEOUT.starts_with(crate::scope_guard::ENVIRONMENT_PREFIX));
        assert!(!is_environment_reason(
            "the verifier returned no valid verdict"
        ));
        assert!(resumable_stop(
            JobState::Closed(OutcomeState::Failed),
            Some(&reason)
        ));
        assert!(!resumable_stop(
            JobState::Closed(OutcomeState::Rejected),
            Some(&reason)
        ));
    }

    #[test]
    fn check_timeout_reason_names_the_duration_and_the_limit() {
        let job = id(1);
        let reason = check_timeout_reason("test", 905, 900, &job);
        assert!(reason.contains("905s of 900s limit"), "{reason}");
        assert!(reason.contains("test ran"), "{reason}");
        assert!(reason.contains(&format!("athena retry {job}")), "{reason}");
        assert!(reason.contains("not the work"), "{reason}");
    }

    #[test]
    fn only_an_environment_failure_or_an_interrupted_job_is_resumable() {
        let low = format!("{LOW_DISK}: 3.0 GB free, 10.0 GB required before the checks");
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

    /// A builder that runs no cargo: it checks the worktree it is handed, writes a file of known length into
    /// the target directory and returns fixed times and counts.
    struct FakeBuilder {
        installed: bool,
        counts: [Option<SccacheCounts>; 2],
        fail_second: bool,
        calls: std::cell::RefCell<Vec<(std::path::PathBuf, std::path::PathBuf)>>,
    }

    /// Bytes the fake writes into the first and the second target directory.
    const TARGET_WRITTEN: [usize; 2] = [3000, 7000];
    const SECONDS: [f64; 2] = [61.5, 12.25];

    impl FakeBuilder {
        fn new(installed: bool, counts: [Option<SccacheCounts>; 2]) -> FakeBuilder {
            FakeBuilder {
                installed,
                counts,
                fail_second: false,
                calls: Default::default(),
            }
        }

        fn with_sccache() -> FakeBuilder {
            FakeBuilder::new(
                true,
                [
                    Some(SccacheCounts { hits: 0, misses: 9 }),
                    Some(SccacheCounts { hits: 4, misses: 5 }),
                ],
            )
        }
    }

    impl CargoBuilder for FakeBuilder {
        fn sccache_installed(&self) -> bool {
            self.installed
        }

        fn build(&self, worktree: &Path, target_dir: &Path) -> anyhow::Result<BuildRun> {
            let n = self.calls.borrow().len();
            self.calls
                .borrow_mut()
                .push((worktree.to_path_buf(), target_dir.to_path_buf()));
            assert!(
                worktree.join("Cargo.lock").is_file(),
                "the worktree holds HEAD's files"
            );
            if n == 1 && self.fail_second {
                anyhow::bail!("the second build failed");
            }
            std::fs::create_dir_all(target_dir.join("debug"))?;
            std::fs::write(target_dir.join("debug/out"), vec![7u8; TARGET_WRITTEN[n]])?;
            Ok(BuildRun {
                seconds: SECONDS[n],
                sccache: self.counts[n],
            })
        }
    }

    /// An Athena over a temp repository whose HEAD holds a `Cargo.lock` and two sources.
    fn repo_with_lock() -> (tempfile::TempDir, tempfile::TempDir, Athena) {
        let (dir, state, athena) = crate::code_health::tests::set_up();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("Cargo.lock"), "version = 4\n").unwrap();
        std::fs::write(dir.path().join("src/a.rs"), vec![b'a'; 100]).unwrap();
        std::fs::write(dir.path().join("src/b.rs"), vec![b'b'; 250]).unwrap();
        athena.git.run(["add", "-A"]).unwrap();
        athena
            .git
            .run(["commit", "-q", "-m", "Add a lock file and sources"])
            .unwrap();
        (dir, state, athena)
    }

    fn measurements(athena: &Athena) -> Vec<BuildMeasurement> {
        Records::load(&athena.git, &athena.head().unwrap())
            .unwrap()
            .build_measurements
    }

    /// The bytes of every file git tracks in the repository: what S must equal.
    fn tracked_bytes(athena: &Athena) -> u64 {
        athena
            .git
            .ls_files()
            .unwrap()
            .iter()
            .map(|p| std::fs::metadata(athena.git.dir().join(p)).unwrap().len())
            .sum()
    }

    fn scratch_is_gone(athena: &Athena, builder: &FakeBuilder) {
        for (worktree, target) in builder.calls.borrow().iter() {
            assert!(!worktree.exists(), "{}", worktree.display());
            assert!(!target.exists(), "{}", target.display());
        }
        assert!(!athena.state.root().join("measure-build").exists());
        let listed = athena.git.run(["worktree", "list"]).unwrap();
        assert_eq!(listed.lines().count(), 1, "{listed}");
    }

    #[test]
    fn measure_build_writes_t_s_both_times_and_the_cache_counts() {
        let (_dir, _state, athena) = repo_with_lock();
        let builder = FakeBuilder::with_sccache();
        let ui = Said::default();
        let head = athena.head().unwrap();
        // Counted before the run: the record it commits is not part of the tree it measured.
        let source_before = tracked_bytes(&athena);

        let measured = athena.measure_build(&builder, &ui).unwrap();

        let records = measurements(&athena);
        assert_eq!(records, vec![measured.clone()]);
        // T is one target directory, the first, as the fake wrote it: not the sum of both.
        assert_eq!(measured.target_bytes, TARGET_WRITTEN[0] as u64);
        // S is the tracked files of one worktree: every file at HEAD, counted from the real checkout.
        assert_eq!(measured.source_bytes, source_before);
        assert!(measured.source_bytes >= 100 + 250 + 12);
        assert_eq!(measured.first_build_seconds, 61.5);
        assert_eq!(measured.second_build_seconds, 12.25);
        assert_eq!(
            measured.sccache,
            SccacheFigures::Measured {
                first: SccacheCounts { hits: 0, misses: 9 },
                second: SccacheCounts { hits: 4, misses: 5 },
            }
        );
        assert_eq!(measured.commit, head);
        assert_eq!(
            Some(measured.cargo_lock_hash.clone()),
            cargo_lock_blob(&athena.git, &head).unwrap()
        );
        // Two builds, in two distinct worktrees with two distinct target directories.
        let calls = builder.calls.borrow();
        assert_eq!(calls.len(), 2);
        assert_ne!(calls[0].0, calls[1].0);
        assert_ne!(calls[0].1, calls[1].1);
        drop(calls);
        scratch_is_gone(&athena, &builder);
        let said = ui.0.lock().unwrap().join("\n");
        assert!(said.contains("61.5"), "{said}");
    }

    #[test]
    fn measure_build_says_so_when_sccache_is_not_installed() {
        let (_dir, _state, athena) = repo_with_lock();
        let builder = FakeBuilder::new(false, [None, None]);

        let measured = athena.measure_build(&builder, &Said::default()).unwrap();

        assert_eq!(measured.sccache, SccacheFigures::NotInstalled);
        assert_eq!(measured.target_bytes, TARGET_WRITTEN[0] as u64);
        assert_eq!(measurements(&athena).len(), 1);
        scratch_is_gone(&athena, &builder);
    }

    #[test]
    fn measure_build_records_nothing_and_cleans_up_when_a_build_fails() {
        let (_dir, _state, athena) = repo_with_lock();
        let mut builder = FakeBuilder::with_sccache();
        builder.fail_second = true;

        let err = athena
            .measure_build(&builder, &Said::default())
            .unwrap_err();

        assert!(
            format!("{err:#}").contains("second build failed"),
            "{err:#}"
        );
        assert!(measurements(&athena).is_empty());
        assert_eq!(builder.calls.borrow().len(), 2);
        scratch_is_gone(&athena, &builder);
    }

    #[test]
    fn measure_build_refuses_installed_sccache_without_counts_and_a_repository_without_a_lock() {
        let (_dir, _state, athena) = repo_with_lock();
        // Installed, but the builder could not read the counts: a zero must not be recorded in their place.
        let builder = FakeBuilder::new(true, [None, None]);
        let err = athena
            .measure_build(&builder, &Said::default())
            .unwrap_err();
        assert!(
            format!("{err:#}").contains("no hit and miss counts"),
            "{err:#}"
        );
        assert!(measurements(&athena).is_empty());
        scratch_is_gone(&athena, &builder);

        let (_dir, _state, bare) = crate::code_health::tests::set_up();
        let builder = FakeBuilder::with_sccache();
        let err = bare.measure_build(&builder, &Said::default()).unwrap_err();
        assert!(format!("{err:#}").contains("no Cargo.lock"), "{err:#}");
        assert!(builder.calls.borrow().is_empty());
        assert!(measurements(&bare).is_empty());
    }

    #[test]
    fn sccache_totals_are_read_from_its_text_and_garbage_is_refused() {
        let text = "Compile requests                      14\n\
                    Compile requests executed             12\n\
                    Cache hits                             5\n\
                    Cache hits (Rust)                      4\n\
                    Cache misses                           7\n\
                    Cache misses (Rust)                    6\n\
                    Cache hits rate                    41.67 %\n";
        assert_eq!(
            parse_sccache_stats(text),
            Some(SccacheCounts { hits: 5, misses: 7 })
        );
        assert_eq!(
            parse_sccache_stats("Cache hits 0\nCache misses 0\n"),
            Some(SccacheCounts { hits: 0, misses: 0 })
        );
        assert_eq!(parse_sccache_stats(""), None);
        assert_eq!(parse_sccache_stats("sccache: command not found"), None);
        assert_eq!(parse_sccache_stats("Cache hits  3\n"), None);
        assert_eq!(
            parse_sccache_stats("Cache hits  x\nCache misses  2\n"),
            None
        );
        assert_eq!(
            parse_sccache_stats("Cache hits (Rust)  3\nCache misses (Rust)  2\n"),
            None
        );
    }

    #[test]
    fn directory_bytes_count_files_below_it() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("a/b")).unwrap();
        std::fs::write(temp.path().join("x"), vec![0u8; 10]).unwrap();
        std::fs::write(temp.path().join("a/y"), vec![0u8; 20]).unwrap();
        std::fs::write(temp.path().join("a/b/z"), vec![0u8; 400]).unwrap();
        assert_eq!(dir_bytes(temp.path()), 430);
        assert_eq!(dir_bytes(&temp.path().join("missing")), 0);
    }

    fn measurement(id: &str, at_secs: i64, t: u64, s: u64) -> BuildMeasurement {
        BuildMeasurement {
            id: RecordId::parse(id).unwrap(),
            at: jiff::Timestamp::from_second(1_790_000_000 + at_secs).unwrap(),
            commit: "0123456789abcdef0123456789abcdef01234567".into(),
            cargo_lock_hash: "89abcdef0123456789abcdef0123456789abcdef".into(),
            target_bytes: t,
            source_bytes: s,
            first_build_seconds: 60.0,
            second_build_seconds: 10.0,
            sccache: SccacheFigures::NotInstalled,
        }
    }

    fn estimates() -> Figures {
        Figures::from_latest(None)
    }

    #[test]
    fn with_no_record_the_design_estimates_are_used() {
        let figures = estimates();
        assert_eq!((figures.t, figures.s), (6_000_000_000, 200_000_000));
        assert_eq!(figures.source, FigureSource::DesignEstimate);
        assert_eq!(figures.source.text(), "design estimate");
    }

    #[test]
    fn the_figures_are_those_of_the_latest_record_not_the_first() {
        let older = measurement("20261029-090000-5abc", 0, 9 * GB, 3 * GB);
        let newer = measurement("20261029-100000-6abc", 60, 4 * GB, GB);
        for list in [
            vec![older.clone(), newer.clone()],
            vec![newer.clone(), older.clone()],
        ] {
            let figures = Figures::from_latest(BuildMeasurement::latest(&list));
            assert_eq!((figures.t, figures.s), (4 * GB, GB));
            assert_eq!(
                figures.source,
                FigureSource::Measured("20261029-100000-6abc".into())
            );
            assert_eq!(
                figures.source.text(),
                "build measurement 20261029-100000-6abc"
            );
        }
    }

    #[test]
    fn the_limit_is_floor_of_free_less_reserve_and_check_slots_over_t_plus_s() {
        // The design's example: (60 - 10 - 1 x 6) / (6 + 0.2) = 7.09.
        assert_eq!(in_flight_limit(60 * GB, 10, 1, &estimates()), 7);
        // Two check slots hold two T: (60 - 10 - 12) / 6.2 = 6.13.
        assert_eq!(in_flight_limit(60 * GB, 10, 2, &estimates()), 6);
        // A measured T = 4 GB and S = 1 GB: (60 - 10 - 4) / 5 = 9.2.
        let measured =
            Figures::from_latest(Some(&measurement("20261029-100000-6abc", 0, 4 * GB, GB)));
        assert_eq!(in_flight_limit(60 * GB, 10, 1, &measured), 9);
        // Exactly one job fits: (22.2 - 10 - 6) / 6.2 = 1; a byte less fits none.
        assert_eq!(in_flight_limit(22_200_000_000, 10, 1, &estimates()), 1);
        assert_eq!(in_flight_limit(22_199_999_999, 10, 1, &estimates()), 0);
    }

    #[test]
    fn a_disk_below_the_reserve_and_check_slots_fits_none_and_zero_cost_is_unbounded() {
        // 15 GB free is below reserve 10 + 1 slot x 6 = 16: saturates at 0, never wraps.
        assert_eq!(in_flight_limit(15 * GB, 10, 1, &estimates()), 0);
        assert_eq!(in_flight_limit(0, 10, 1, &estimates()), 0);
        let free_jobs = Figures {
            t: 0,
            s: 0,
            source: FigureSource::DesignEstimate,
        };
        assert_eq!(in_flight_limit(60 * GB, 10, 1, &free_jobs), u64::MAX);
    }

    fn pipeline(max_in_flight: u32) -> athena_data::config::PipelineConfig {
        athena_data::config::PipelineConfig {
            max_in_flight,
            ..Default::default()
        }
    }

    #[test]
    fn a_start_waits_only_when_the_disk_fits_fewer_than_max_in_flight_and_others_fill_it() {
        let figures = estimates();
        // 25 GB free, nothing held by the other job: limit floor(9 / 6.2) = 1 of 2, one is running.
        let low = start_decision(&pipeline(2), &figures, 25 * GB, 0, 1).expect("waits");
        assert_eq!(low.free, 25 * GB);
        // The free bytes two jobs need: 10 + 6 + 2 x 6.2 = 28.4 GB.
        assert_eq!(low.required, 28_400_000_000);
        assert!(low.free < low.required);
        // The bytes the running job already holds count as free for the bound: 27 GB + 3 GB held gives 30.
        assert_eq!(
            start_decision(&pipeline(2), &figures, 27 * GB, 3 * GB, 1),
            None
        );
        let low = start_decision(&pipeline(2), &figures, 20 * GB, 3 * GB, 1).expect("waits");
        assert_eq!(low.required, 25_400_000_000, "28.4 less the 3 GB it holds");
        // Room for three jobs, two allowed: not the bound's business.
        assert_eq!(start_decision(&pipeline(2), &figures, 40 * GB, 0, 1), None);
        // Nothing in flight: nothing to wait for, even on a disk that fits none.
        assert_eq!(start_decision(&pipeline(2), &figures, 12 * GB, 0, 0), None);
        // One job at a time never consults the bound.
        assert_eq!(start_decision(&pipeline(1), &figures, 12 * GB, 0, 3), None);
        // Three allowed, a disk of two (30 GB: floor(14 / 6.2) = 2), two running: the third waits.
        assert!(start_decision(&pipeline(3), &figures, 30 * GB, 0, 2).is_some());
        assert_eq!(start_decision(&pipeline(3), &figures, 30 * GB, 0, 1), None);
    }

    #[test]
    fn a_probe_failure_hold_has_no_free_bytes_and_a_marker_names_the_amounts() {
        let temp = tempfile::tempdir().unwrap();
        let state = StateDir::new(temp.path().to_path_buf()).with_free_space(plenty);
        let job = id(2);
        let ui = Said::default();
        let low = verifier_environment_problem(&state, 10, &job, &temp.path().join("gone"), &ui)
            .expect("an unwritable workspace holds");
        write_hold(&state, &job, &low);
        let read = read_hold(&state, &job).expect("the marker is written");
        assert_eq!(read.free, 0);
        assert_eq!(read.required, 10 * GB);
        assert!(read.text().contains("0 B free"), "{}", read.text());
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
