//! The state directory of one repository (design §2.4).
//!
//! It holds what lives on one machine while work runs: snapshots, live copies of records not yet
//! committed, partial usage of running invocations, the process-group registry, locks, the kill switch,
//! and per-job build directories. Once a job's records are committed, nothing here matters.

use anyhow::{Context, Result};
use athena_agent::procs::Registry;
use athena_data::JobId;
use athena_data::records::{Candidate, CheckRun};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};

/// One repository's state directory.
#[derive(Debug, Clone)]
pub struct StateDir {
    root: PathBuf,
    remover: Remover,
    free_space: FreeSpace,
}

/// A held lock; released on drop.
#[derive(Debug)]
pub struct Lock {
    _file: File,
}

impl StateDir {
    pub fn new(root: PathBuf) -> StateDir {
        StateDir {
            root,
            remover: crate::pin::remove_tree,
            free_space: crate::disk::df_free_bytes,
        }
    }

    /// The state directory for the repository whose common git directory is `common_dir`.
    pub fn for_repository(common_dir: &Path) -> StateDir {
        StateDir::new(athena_data::paths::repo_state_dir(common_dir))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn registry(&self) -> Registry {
        Registry::new(&self.root)
    }

    /// The job's own directory: snapshots, live records, build directories.
    pub fn job_dir(&self, job: &JobId) -> PathBuf {
        self.root.join("jobs").join(job.as_str())
    }

    /// Where the work tree of a job whose producer hit its deadline is kept. It lies outside the job's own
    /// directory, which closing removes, so a retry can start from it.
    pub fn kept_work_dir(&self, job: &JobId) -> PathBuf {
        self.root.join("kept").join(job.as_str()).join("work")
    }

    /// Removes the kept work of `job`, if any, and its now empty parent.
    pub fn remove_kept_work(&self, job: &JobId) {
        let dir = self.root.join("kept").join(job.as_str());
        let _ = (self.remover)(&dir);
    }

    /// Where records of a job are mirrored until they are committed.
    pub fn live_dir(&self, job: &JobId) -> PathBuf {
        self.job_dir(job).join("live")
    }

    fn lock_file(&self, path: PathBuf) -> Result<File> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .with_context(|| format!("opening {}", path.display()))
    }

    /// The repository lock, held while Athena writes to the user's repository.
    pub fn lock_repository(&self) -> Result<Lock> {
        let file = self.lock_file(self.root.join("repository.lock"))?;
        file.lock()?;
        Ok(Lock { _file: file })
    }

    /// The job lock, held by the one command that is running a job. Fails at once if another holds it.
    pub fn try_lock_job(&self, job: &JobId) -> Result<Option<Lock>> {
        let file = self.lock_file(self.job_dir(job).join("job.lock"))?;
        match file.try_lock() {
            Ok(()) => Ok(Some(Lock { _file: file })),
            Err(std::fs::TryLockError::WouldBlock) => Ok(None),
            Err(std::fs::TryLockError::Error(e)) => Err(e.into()),
        }
    }

    /// Writes a record's live copy, so `athena close` can commit it if this process dies first.
    pub fn write_live(&self, job: &JobId, path: &str, bytes: &[u8]) -> Result<()> {
        let full = self.live_dir(job).join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(full, bytes)?;
        Ok(())
    }

    /// Every live record of a job: (repository path, bytes).
    pub fn live_records(&self, job: &JobId) -> Result<Vec<(String, Vec<u8>)>> {
        let base = self.live_dir(job);
        let mut out = Vec::new();
        if base.exists() {
            collect(&base, &base, &mut out)?;
        }
        out.sort();
        Ok(out)
    }

    /// The file where an invocation's running usage is persisted.
    pub fn partial_usage_file(&self, job: &JobId, seq: u32) -> PathBuf {
        self.job_dir(job)
            .join("usage")
            .join(format!("{seq:02}.json"))
    }

    /// Replaces how a job directory is removed. Tests use it to make removal fail after the job's closing
    /// commit; the default restores write access to read-only mounts and removes the tree.
    pub fn with_remover(mut self, remover: Remover) -> StateDir {
        self.remover = remover;
        self
    }

    /// Replaces how free disk space is read. Tests use it to simulate a full or a recovered disk; the default
    /// asks `df`.
    pub fn with_free_space(mut self, free_space: FreeSpace) -> StateDir {
        self.free_space = free_space;
        self
    }

    /// Free bytes on the disk that holds the state directory, or `None` when they cannot be read. An unknown
    /// amount never holds or fails a job.
    pub fn free_bytes(&self) -> Option<u64> {
        (self.free_space)(&self.root).ok()
    }

    /// The directories of the jobs that have one here, in name order.
    pub fn job_ids(&self) -> Vec<JobId> {
        let Ok(entries) = std::fs::read_dir(self.root.join("jobs")) else {
            return Vec::new();
        };
        let mut ids: Vec<JobId> = entries
            .flatten()
            .filter(|e| e.path().is_dir())
            .filter_map(|e| JobId::parse(&e.file_name().to_string_lossy()).ok())
            .collect();
        ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        ids
    }

    /// Removes only the build directory (`target/`) of `job`, keeping its source, snapshots and records.
    /// Returns the allocated bytes it held, or the system's message when removal failed.
    pub fn remove_target(&self, job: &JobId) -> std::result::Result<u64, String> {
        let dir = self.job_dir(job).join("target");
        if !dir.exists() {
            return Ok(0);
        }
        let bytes = allocated_bytes(&dir);
        (self.remover)(&dir)
            .map(|()| bytes)
            .map_err(|e| e.to_string())
    }

    /// The producer's build directory of `job` (`CARGO_TARGET_DIR` is `<job dir>/target/produce`).
    pub fn produce_dir(&self, job: &JobId) -> PathBuf {
        self.job_dir(job).join("target").join("produce")
    }

    /// Removes only the producer's build directory of `job`, keeping the rest of `target/` (the checks'
    /// own build directory). Returns the allocated bytes it held, or the system's message when removal failed.
    pub fn remove_produce(&self, job: &JobId) -> std::result::Result<u64, String> {
        let dir = self.produce_dir(job);
        if !dir.exists() {
            return Ok(0);
        }
        let bytes = allocated_bytes(&dir);
        (self.remover)(&dir)
            .map(|()| bytes)
            .map_err(|e| e.to_string())
    }

    /// Where the resume mark of `job` is kept. It lies outside the job's own directory, which closing
    /// removes, so `athena retry` can resume from the built candidate after the job stopped.
    pub fn resume_file(&self, job: &JobId) -> PathBuf {
        self.root
            .join("kept")
            .join(job.as_str())
            .join("resume.json")
    }

    /// Writes the resume mark of `job`, replacing an earlier one.
    pub fn write_resume(&self, job: &JobId, mark: &ResumeMark) -> Result<()> {
        let path = self.resume_file(job);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_vec(mark)?)
            .with_context(|| format!("writing {}", path.display()))
    }

    /// The resume mark of `job`; `None` when there is none or it cannot be read.
    pub fn read_resume(&self, job: &JobId) -> Option<ResumeMark> {
        let bytes = std::fs::read(self.resume_file(job)).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// Removes the resume mark of `job`, and its directory when nothing else is kept there.
    pub fn remove_resume(&self, job: &JobId) {
        let file = self.resume_file(job);
        let _ = std::fs::remove_file(&file);
        // `remove_dir` removes only an empty directory: other kept work stays.
        if let Some(dir) = file.parent() {
            let _ = std::fs::remove_dir(dir);
            if let Some(kept) = dir.parent() {
                let _ = std::fs::remove_dir(kept);
            }
        }
    }

    /// The in-flight table of this repository's drain, in the state root.
    pub fn in_flight_table(&self) -> InFlightTable {
        InFlightTable::at(self.root.join("in-flight.json"))
    }

    /// The local marker that says dispatch of `job` was held for lack of disk space.
    pub fn hold_file(&self, job: &JobId) -> PathBuf {
        self.root
            .join("holds")
            .join(format!("{}.json", job.as_str()))
    }

    /// Removes a closed job's snapshots and build directories, keeping nothing that matters. Call it only
    /// after the closing commit has succeeded. It measures the directory's allocated size just before
    /// removal and returns it with the error, if any; the caller reports both with [`report_clean`].
    pub fn clean_job(&self, job: &JobId) -> CleanReport {
        let dir = self.job_dir(job);
        let bytes = allocated_bytes(&dir);
        match (self.remover)(&dir) {
            Ok(()) => CleanReport { bytes, error: None },
            Err(e) => CleanReport {
                bytes: 0,
                error: Some((dir, e.to_string())),
            },
        }
    }
}

/// The stage a job had reached when it stopped, which `athena retry` resumes at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResumeStage {
    /// The candidate was built; the checks had not all passed.
    Checks,
    /// Every check passed; the verifier had not run.
    Verifier,
}

/// What a retry needs to resume a job from its built candidate without a planner or a producer run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResumeMark {
    pub stage: ResumeStage,
    /// The job's base (prediction) commit, which the candidate's commit sits on.
    pub base_commit: String,
    pub candidate: Candidate,
    /// The check results, recorded once the stage is [`ResumeStage::Verifier`]; empty before.
    pub check_runs: Vec<CheckRun>,
    /// What the producer's report still lacked after its send-back, as the verifier is told.
    pub report_gaps: Vec<String>,
    /// The criteria the producer left to Athena's checks.
    pub left_criteria: Vec<String>,
}

/// The stage an in-flight job has reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InFlightStage {
    /// The producer is building the candidate.
    Producer,
    /// The checks are running on the candidate.
    Checks,
    /// The candidate is being landed.
    Landing,
}

impl InFlightStage {
    /// The stage's name as the file and the drain's notes spell it.
    pub fn as_str(self) -> &'static str {
        match self {
            InFlightStage::Producer => "producer",
            InFlightStage::Checks => "checks",
            InFlightStage::Landing => "landing",
        }
    }
}

/// One row of the in-flight table: a job the drain has started and not yet closed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InFlightEntry {
    /// The job's id.
    pub job: String,
    /// The stage the job was last known to be in.
    pub stage: InFlightStage,
    /// The job's work tree, as a path.
    pub worktree: String,
}

/// A local file listing the jobs a drain has in flight: job id, stage and work tree. The drain rewrites it at
/// each stage change and removes a job's row when the job closes, so a file left behind by a drain that died
/// names the jobs it had started. The file is local state, never committed. A missing file is an empty table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InFlightTable {
    path: PathBuf,
}

impl InFlightTable {
    /// The table kept in `path`.
    pub fn at(path: PathBuf) -> InFlightTable {
        InFlightTable { path }
    }

    /// The file this table is kept in.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Every row. A missing file is an empty table; a file that cannot be read or parsed is an error that
    /// names the file.
    pub fn read(&self) -> Result<Vec<InFlightEntry>> {
        let bytes = match std::fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => {
                return Err(e).with_context(|| format!("reading {}", self.path.display()));
            }
        };
        serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", self.path.display()))
    }

    /// Records that `job` is in `stage` in `worktree`: updates the job's row in place, or adds one.
    pub fn set(&self, job: &str, stage: InFlightStage, worktree: &Path) -> Result<()> {
        // A file that cannot be read is replaced rather than blocking the new row.
        let mut rows = self.read().unwrap_or_default();
        let row = InFlightEntry {
            job: job.to_string(),
            stage,
            worktree: worktree.display().to_string(),
        };
        match rows.iter_mut().find(|r| r.job == job) {
            Some(existing) => *existing = row,
            None => rows.push(row),
        }
        self.write(&rows)
    }

    /// Removes the row of `job`, if any.
    pub fn remove(&self, job: &str) -> Result<()> {
        let mut rows = self.read().unwrap_or_default();
        let before = rows.len();
        rows.retain(|r| r.job != job);
        if rows.len() == before {
            return Ok(());
        }
        self.write(&rows)
    }

    /// Writes `rows` to a temporary file beside the table and renames it over the table, so a reader sees
    /// the old file or the new one, never half of one.
    fn write(&self, rows: &[InFlightEntry]) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut temp = self.path.clone().into_os_string();
        temp.push(".tmp");
        let temp = PathBuf::from(temp);
        std::fs::write(&temp, serde_json::to_vec_pretty(rows)?)
            .with_context(|| format!("writing {}", temp.display()))?;
        std::fs::rename(&temp, &self.path)
            .with_context(|| format!("replacing {}", self.path.display()))
    }
}

/// How a job directory is removed.
pub type Remover = fn(&Path) -> std::io::Result<()>;

/// How free disk space, in bytes, is read for the disk that holds a path.
pub type FreeSpace = fn(&Path) -> std::io::Result<u64>;

/// What removing a closed job's directory freed, or why it could not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanReport {
    /// Allocated bytes of the removed directory; 0 when removal failed.
    pub bytes: u64,
    /// The path that could not be removed, and the system's message.
    pub error: Option<(PathBuf, String)>,
}

/// Tells the user what removing a closed job's directory freed, or that it could not be removed. A failed
/// removal never claims space.
pub fn report_clean(ui: &dyn crate::Ui, report: &CleanReport) {
    match &report.error {
        None => ui.line(&format!("reclaimed {}", format_bytes(report.bytes))),
        Some((path, why)) => ui.line(&format!(
            "warning: could not remove {}: {why}",
            path.display()
        )),
    }
}

/// `bytes` in decimal units with one decimal place: B, KB, MB, GB, TB.
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if bytes < 1000 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

/// The space `path` occupies on disk: every directory, file and symlink under it (symlinks are not
/// followed) counted by allocated blocks, with each hard-linked file counted once. Unreadable entries add
/// nothing. A missing path is 0.
pub fn allocated_bytes(path: &Path) -> u64 {
    let mut seen = std::collections::HashSet::new();
    let mut total = 0;
    walk_allocated(path, &mut seen, &mut total);
    total
}

fn walk_allocated(path: &Path, seen: &mut std::collections::HashSet<(u64, u64)>, total: &mut u64) {
    use std::os::unix::fs::MetadataExt;
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return;
    };
    if seen.insert((meta.dev(), meta.ino())) {
        *total += meta.blocks() * 512;
    }
    if meta.is_dir()
        && let Ok(entries) = std::fs::read_dir(path)
    {
        for entry in entries.flatten() {
            walk_allocated(&entry.path(), seen, total);
        }
    }
}

fn collect(base: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            collect(base, &path, out)?;
        } else {
            let relative = path
                .strip_prefix(base)
                .expect("under base")
                .to_string_lossy()
                .replace('\\', "/");
            out.push((relative, std::fs::read(&path)?));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::MetadataExt;
    use std::sync::Mutex;

    #[derive(Default)]
    struct Said(Mutex<Vec<String>>);

    impl crate::Ui for Said {
        fn line(&self, text: &str) {
            self.0.lock().unwrap().push(text.to_string());
        }
    }

    fn job() -> JobId {
        JobId::parse("20261023-181804-j42y").unwrap()
    }

    fn blocks(path: &Path) -> u64 {
        std::fs::symlink_metadata(path).unwrap().blocks() * 512
    }

    #[test]
    fn the_in_flight_table_round_trips_updates_in_place_and_removes() {
        let temp = tempfile::tempdir().unwrap();
        let table = StateDir::new(temp.path().join("repo")).in_flight_table();
        assert!(table.read().unwrap().is_empty(), "a missing file is empty");
        table
            .set("job-a", InFlightStage::Producer, Path::new("/w/a"))
            .unwrap();
        table
            .set("job-b", InFlightStage::Landing, Path::new("/w/b"))
            .unwrap();
        let rows = table.read().unwrap();
        assert_eq!(
            rows,
            [
                InFlightEntry {
                    job: "job-a".into(),
                    stage: InFlightStage::Producer,
                    worktree: "/w/a".into()
                },
                InFlightEntry {
                    job: "job-b".into(),
                    stage: InFlightStage::Landing,
                    worktree: "/w/b".into()
                },
            ]
        );
        table
            .set("job-a", InFlightStage::Checks, Path::new("/w/a"))
            .unwrap();
        let rows = table.read().unwrap();
        assert_eq!(rows.len(), 2, "updated in place, not added");
        assert_eq!(rows[0].stage, InFlightStage::Checks);
        table.remove("job-a").unwrap();
        let rows = table.read().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].job, "job-b");
    }

    #[test]
    fn the_in_flight_table_is_written_through_a_rename_and_leaves_no_temporary_file() {
        let temp = tempfile::tempdir().unwrap();
        let table = InFlightTable::at(temp.path().join("in-flight.json"));
        table
            .set("j", InFlightStage::Producer, Path::new("/w"))
            .unwrap();
        table
            .set("j", InFlightStage::Landing, Path::new("/w"))
            .unwrap();
        let names: Vec<_> = std::fs::read_dir(temp.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        assert_eq!(
            names,
            ["in-flight.json"],
            "the temporary file was renamed away"
        );
    }

    #[test]
    fn a_corrupt_in_flight_table_is_an_error_naming_the_file_and_the_next_write_replaces_it() {
        let temp = tempfile::tempdir().unwrap();
        let table = InFlightTable::at(temp.path().join("in-flight.json"));
        std::fs::write(table.path(), b"{not json").unwrap();
        let error = format!("{:#}", table.read().unwrap_err());
        assert!(error.contains("in-flight.json"), "{error}");
        table
            .set("j", InFlightStage::Producer, Path::new("/w"))
            .unwrap();
        assert_eq!(table.read().unwrap().len(), 1);
    }

    #[test]
    fn format_bytes_uses_decimal_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(999), "999 B");
        assert_eq!(format_bytes(4096), "4.1 KB");
        assert_eq!(format_bytes(5_500_000), "5.5 MB");
        assert_eq!(format_bytes(11_400_000_000), "11.4 GB");
        assert_eq!(format_bytes(2_000_000_000_000), "2.0 TB");
        assert_eq!(format_bytes(5_000_000_000_000_000), "5000.0 TB");
    }

    #[test]
    fn cleaning_removes_the_directory_and_reports_its_allocated_size_once_per_hard_link() {
        let temp = tempfile::tempdir().unwrap();
        let state = StateDir::new(temp.path().to_path_buf());
        let dir = state.job_dir(&job());
        std::fs::create_dir_all(dir.join("target/debug")).unwrap();
        std::fs::create_dir_all(dir.join("live")).unwrap();
        std::fs::write(dir.join("job.lock"), b"").unwrap();
        std::fs::write(dir.join("live/x"), vec![1u8; 20_000]).unwrap();
        std::fs::write(dir.join("target/debug/a"), vec![2u8; 100_000]).unwrap();
        std::fs::hard_link(dir.join("target/debug/a"), dir.join("target/debug/b")).unwrap();
        std::os::unix::fs::symlink("/", dir.join("target/link")).unwrap();

        let expected = [
            dir.clone(),
            dir.join("target"),
            dir.join("target/debug"),
            dir.join("live"),
            dir.join("job.lock"),
            dir.join("live/x"),
            dir.join("target/debug/a"),
            dir.join("target/link"),
        ]
        .iter()
        .map(|p| blocks(p))
        .sum::<u64>();
        assert!(expected >= 120_000, "the planted files are counted");

        let report = state.clean_job(&job());
        assert_eq!(report.error, None);
        assert_eq!(report.bytes, expected);
        assert!(!dir.exists());
    }

    #[test]
    fn a_failed_removal_names_the_path_and_claims_no_space() {
        let temp = tempfile::tempdir().unwrap();
        let state = StateDir::new(temp.path().to_path_buf());
        // A regular file where the directory should be cannot be removed as a tree.
        let path = state.job_dir(&job());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, vec![0u8; 10_000]).unwrap();
        let report = state.clean_job(&job());
        let (failed, _) = report.error.clone().expect("removal fails");
        assert_eq!(failed, path);
        assert_eq!(report.bytes, 0);
        let ui = Said::default();
        report_clean(&ui, &report);
        let said = ui.0.lock().unwrap().clone();
        assert_eq!(said.len(), 1, "{said:?}");
        assert!(
            said[0].starts_with("warning: could not remove "),
            "{said:?}"
        );
        assert!(said[0].contains(&path.display().to_string()), "{said:?}");
        assert!(!said[0].contains("reclaimed"));
    }

    #[test]
    fn an_injected_remover_failure_is_reported_and_a_success_reports_reclaimed() {
        let temp = tempfile::tempdir().unwrap();
        let state = StateDir::new(temp.path().to_path_buf()).with_remover(|_| {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "injected",
            ))
        });
        std::fs::create_dir_all(state.job_dir(&job())).unwrap();
        let report = state.clean_job(&job());
        assert!(report.error.as_ref().unwrap().1.contains("injected"));
        assert!(state.job_dir(&job()).exists());

        let ok = StateDir::new(temp.path().to_path_buf());
        let report = ok.clean_job(&job());
        let ui = Said::default();
        report_clean(&ui, &report);
        let said = ui.0.lock().unwrap().clone();
        assert_eq!(
            said,
            vec![format!("reclaimed {}", format_bytes(report.bytes))]
        );
        assert!(!ok.job_dir(&job()).exists());
    }

    #[test]
    fn removing_the_producer_build_directory_keeps_the_rest_of_target() {
        let temp = tempfile::tempdir().unwrap();
        let state = StateDir::new(temp.path().to_path_buf());
        let dir = state.job_dir(&job());
        std::fs::create_dir_all(dir.join("target/produce/debug")).unwrap();
        std::fs::create_dir_all(dir.join("target/verify")).unwrap();
        std::fs::write(dir.join("target/produce/debug/big"), vec![1u8; 50_000]).unwrap();
        std::fs::write(dir.join("target/verify/keep"), b"x").unwrap();

        let freed = state.remove_produce(&job()).unwrap();
        assert!(freed >= 50_000, "{freed}");
        assert!(!state.produce_dir(&job()).exists());
        assert!(dir.join("target/verify/keep").exists());
        // Nothing to remove is not an error.
        assert_eq!(state.remove_produce(&job()), Ok(0));
    }

    fn mark(stage: ResumeStage) -> ResumeMark {
        ResumeMark {
            stage,
            base_commit: "a".repeat(40),
            candidate: Candidate {
                job: job(),
                base_commit: "a".repeat(40),
                base_fingerprint: None,
                previous_job: None,
                candidate_commit: Some("b".repeat(40)),
                candidate_tree: Some("c".repeat(40)),
                changes: Vec::new(),
                fingerprint: Some("f".repeat(64)),
                violations: Vec::new(),
                producer_report: None,
            },
            check_runs: Vec::new(),
            report_gaps: vec!["a gap".into()],
            left_criteria: Vec::new(),
        }
    }

    #[test]
    fn a_resume_mark_round_trips_and_removing_it_removes_its_directory() {
        let temp = tempfile::tempdir().unwrap();
        let state = StateDir::new(temp.path().to_path_buf());
        assert_eq!(state.read_resume(&job()), None);
        state
            .write_resume(&job(), &mark(ResumeStage::Checks))
            .unwrap();
        assert_eq!(state.read_resume(&job()), Some(mark(ResumeStage::Checks)));
        // A later write replaces it.
        state
            .write_resume(&job(), &mark(ResumeStage::Verifier))
            .unwrap();
        assert_eq!(
            state.read_resume(&job()).unwrap().stage,
            ResumeStage::Verifier
        );
        // It lies outside the job directory, so cleaning the job keeps it.
        std::fs::create_dir_all(state.job_dir(&job())).unwrap();
        assert_eq!(state.clean_job(&job()).error, None);
        assert!(state.read_resume(&job()).is_some());
        state.remove_resume(&job());
        assert_eq!(state.read_resume(&job()), None);
        assert!(!state.resume_file(&job()).parent().unwrap().exists());
        assert!(!temp.path().join("kept").exists(), "nothing else is kept");
    }
}
