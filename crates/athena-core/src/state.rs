//! The state directory of one repository (design §2.4).
//!
//! It holds what lives on one machine while work runs: snapshots, live copies of records not yet
//! committed, partial usage of running invocations, the process-group registry, locks, the kill switch,
//! and per-job build directories. Once a job's records are committed, nothing here matters.

use anyhow::{Context, Result};
use athena_agent::procs::Registry;
use athena_data::JobId;
use std::fs::File;
use std::path::{Path, PathBuf};

/// One repository's state directory.
#[derive(Debug, Clone)]
pub struct StateDir {
    root: PathBuf,
    remover: Remover,
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

/// How a job directory is removed.
pub type Remover = fn(&Path) -> std::io::Result<()>;

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
}
