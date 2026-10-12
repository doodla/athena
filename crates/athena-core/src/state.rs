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
}

/// A held lock; released on drop.
#[derive(Debug)]
pub struct Lock {
    _file: File,
}

impl StateDir {
    pub fn new(root: PathBuf) -> StateDir {
        StateDir { root }
    }

    /// The state directory for the repository whose common git directory is `common_dir`.
    pub fn for_repository(common_dir: &Path) -> StateDir {
        StateDir {
            root: athena_data::paths::repo_state_dir(common_dir),
        }
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

    /// Removes a closed job's snapshots and build directories, keeping nothing that matters.
    pub fn clean_job(&self, job: &JobId) {
        // A study's pinned mounts are read-only, so removal first restores write access.
        let _ = crate::pin::remove_tree(&self.job_dir(job));
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
