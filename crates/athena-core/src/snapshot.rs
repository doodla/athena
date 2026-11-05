//! The evidence snapshot: a read-only copy of `records/`, `landings/` and main's first-parent history,
//! written at dispatch outside the producer's working copy, so a producer or verifier can check a claim about
//! real records without the working copy holding any.
//!
//! The snapshot is runtime state under the job's state directory (`<state>/jobs/<id>/evidence`), not a record
//! file. It holds:
//! - `records/`: every file under `records/` at the dispatch base commit;
//! - `landings/`: the landing records from the tree and from the local records branch's tip, the two sources
//!   [`Records::load`] reads;
//! - `history.json`: the base commit and the newest [`HISTORY_LIMIT`] commits on its first-parent chain, each
//!   with hash, commit time, subject and trailers.
//!
//! [`status`] reads such a directory with no repository (`athena status --snapshot <dir>`). It assembles the
//! records through [`Records::load_dir`], which shares its assembly with [`Records::load`], and prints the
//! unchanged [`Throughput::text`], so the same records and history give the same throughput section.

use crate::state::StateDir;
use crate::status::Throughput;
use crate::store::{Records, landing_files};
use anyhow::{Context, Result, bail};
use athena_data::JobId;
use athena_git::Git;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// How many first-parent commits `history.json` keeps.
pub const HISTORY_LIMIT: usize = 200;

/// The file name of the history inside a snapshot.
pub const HISTORY_FILE: &str = "history.json";

/// Where a job's evidence snapshot lives.
pub fn evidence_dir(state: &StateDir, job: &JobId) -> PathBuf {
    state.job_dir(job).join("evidence")
}

/// One commit of `history.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub hash: String,
    /// Committer time, strict ISO 8601.
    pub committed_at: String,
    pub subject: String,
    /// Trailer lines as `Key: value`.
    pub trailers: Vec<String>,
}

/// `history.json`: the commit the snapshot was taken at and the commits before it, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryFile {
    pub base: String,
    pub commits: Vec<HistoryEntry>,
}

/// Writes the snapshot of `treeish` into `dest` and locks it read-only. Anything already at `dest` (an
/// earlier dispatch of the same job) is replaced. Fails, leaving the directory unlocked, when the lock does
/// not hold; a snapshot that a briefing calls read-only is never left writable.
pub fn write(git: &Git, treeish: &str, dest: &Path) -> Result<()> {
    crate::pin::remove_tree(dest)
        .with_context(|| format!("removing the earlier evidence snapshot {}", dest.display()))?;
    std::fs::create_dir_all(dest.join("records"))?;
    std::fs::create_dir_all(dest.join("landings"))?;

    let entries: Vec<_> = git
        .ls_tree_under(treeish, "records")?
        .into_iter()
        .filter(|e| e.kind == "blob")
        .collect();
    let oids: Vec<String> = entries.iter().map(|e| e.oid.clone()).collect();
    for (entry, bytes) in entries.iter().zip(git.cat_blobs(&oids)?) {
        put(dest, &entry.path, &bytes)?;
    }

    // The tree and the records branch can name one landing file twice. A repeat with the same bytes is
    // dropped; one with other bytes is kept under a numbered name, so nothing `Records::load` reads is lost.
    let mut written: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for (path, bytes) in landing_files(git, treeish) {
        if written.get(&path).is_some_and(|b| *b == bytes) {
            continue;
        }
        let mut name = path.clone();
        let mut n = 1;
        while written.contains_key(&name) {
            n += 1;
            name = match path.rsplit_once('.') {
                Some((stem, ext)) => format!("{stem}-{n}.{ext}"),
                None => format!("{path}-{n}"),
            };
        }
        put(dest, &name, &bytes)?;
        written.insert(name, bytes);
    }

    let base = git
        .rev_parse(treeish)?
        .unwrap_or_else(|| treeish.to_string());
    let history = HistoryFile {
        base,
        commits: git
            .log_first_parent(treeish, HISTORY_LIMIT)?
            .into_iter()
            .map(|c| HistoryEntry {
                hash: c.hash,
                committed_at: c.committed_at,
                subject: c.subject,
                trailers: c.trailers,
            })
            .collect(),
    };
    put(
        dest,
        HISTORY_FILE,
        serde_json::to_string_pretty(&history)?.as_bytes(),
    )?;
    lock(dest)
}

fn put(root: &Path, relative: &str, bytes: &[u8]) -> Result<()> {
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))
}

/// Makes the snapshot refuse writes, even from root.
///
/// Write bits come off every file and directory, which stops every user but root. Where the process may set
/// it (root, on a file system that supports it) the immutable attribute goes on too, which stops root. The lock
/// is then probed: a file is created and an existing one opened for writing. If either works the snapshot is
/// unlocked again and the call fails, so a lock that does not hold is never taken for one that does.
pub fn lock(dir: &Path) -> Result<()> {
    set_write_bits(dir, false)?;
    // Best effort: without the privilege or file system support the probe below decides.
    let _ = set_immutable(dir, true);
    if let Some(how) = probe_write(dir) {
        let _ = unlock(dir);
        bail!(
            "the evidence snapshot lock does not hold: {how} succeeded in {}; run as a user the permission \
             bits bind, or on a file system that supports the immutable attribute",
            dir.display()
        );
    }
    Ok(())
}

/// Undoes [`lock`], so the snapshot can be replaced or removed.
pub fn unlock(dir: &Path) -> Result<()> {
    athena_git::snapshot::unlock_dir(dir)?;
    Ok(())
}

/// The first write that works on a locked snapshot, if any (what it did is undone).
fn probe_write(dir: &Path) -> Option<&'static str> {
    let fresh = dir.join(".athena-lock-probe");
    if std::fs::write(&fresh, b"x").is_ok() {
        let _ = std::fs::remove_file(&fresh);
        return Some("creating a file");
    }
    if std::fs::OpenOptions::new()
        .append(true)
        .open(dir.join(HISTORY_FILE))
        .is_ok()
    {
        return Some("opening a snapshot file for writing");
    }
    None
}

/// Clears every write bit under `dir` (the files first, so a directory is still reachable while it is
/// walked). Symlinks are left alone.
fn set_write_bits(path: &Path, writable: bool) -> Result<()> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        return Ok(());
    }
    if meta.is_dir() {
        for child in std::fs::read_dir(path)? {
            set_write_bits(&child?.path(), writable)?;
        }
    }
    let mut permissions = meta.permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = permissions.mode();
        permissions.set_mode(if writable {
            mode | 0o200
        } else {
            mode & !0o222
        });
    }
    #[cfg(not(unix))]
    permissions.set_readonly(!writable);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

/// Sets or clears the immutable attribute under `dir` with the system's `chattr`.
fn set_immutable(dir: &Path, on: bool) -> std::io::Result<()> {
    let status = std::process::Command::new("chattr")
        .arg("-R")
        .arg(if on { "+i" } else { "-i" })
        .arg(dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("chattr failed"))
    }
}

/// Reads `history.json` from a snapshot directory.
pub fn read_history(dir: &Path) -> Result<HistoryFile> {
    let path = dir.join(HISTORY_FILE);
    let text = std::fs::read_to_string(&path)
        .with_context(|| format!("reading {}; is it an evidence snapshot?", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

/// The short account of the history in a snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HistorySummary {
    pub commits: usize,
    pub newest: Option<HistoryEntry>,
    pub oldest: Option<HistoryEntry>,
}

/// What `athena status --snapshot` shows: the records and history of a snapshot, with no repository.
#[derive(Debug, Clone, Serialize)]
pub struct SnapshotView {
    /// The snapshot directory as given.
    pub dir: String,
    /// The commit the snapshot was taken at.
    pub base: String,
    pub history: HistorySummary,
    /// Request jobs by state name, as `athena status` counts them.
    pub jobs_by_state: BTreeMap<String, usize>,
    /// Landing records found (`landings/`).
    pub landing_records: usize,
    /// Throughput over the snapshot's records; none when no request job landed.
    pub throughput: Option<Throughput>,
    /// Files under `records/` that are not acceptable records.
    pub invalid_records: Vec<String>,
}

/// Reads the snapshot at `dir`. Needs no repository and no `records/` in the current directory.
pub fn status(dir: &Path) -> Result<SnapshotView> {
    if !dir.is_dir() {
        bail!(
            "no evidence snapshot at {}; it is written at dispatch, under the job's state directory",
            dir.display()
        );
    }
    let history = read_history(dir)?;
    let records = Records::load_dir(dir)?;
    let mut jobs_by_state: BTreeMap<String, usize> = BTreeMap::new();
    for job in records.request_jobs() {
        *jobs_by_state
            .entry(job.state().name().to_string())
            .or_default() += 1;
    }
    Ok(SnapshotView {
        dir: dir.display().to_string(),
        base: history.base.clone(),
        history: HistorySummary {
            commits: history.commits.len(),
            newest: history.commits.first().cloned(),
            oldest: history.commits.last().cloned(),
        },
        jobs_by_state,
        landing_records: records.landings.len(),
        throughput: Throughput::compute(&records),
        invalid_records: records.invalid.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_git::Git;

    /// Unlocks a snapshot when a test ends, even by a panic, so its temporary directory can be removed.
    struct Unlock(PathBuf);
    impl Drop for Unlock {
        fn drop(&mut self) {
            let _ = unlock(&self.0);
        }
    }

    fn repo(files: &[(&str, &str)]) -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (path, content) in files {
            let full = dir.path().join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, content).unwrap();
        }
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "first"]).unwrap();
        (dir, git)
    }

    fn commit(git: &Git, message: &str) {
        git.run(["commit", "-q", "--allow-empty", "-m", message])
            .unwrap();
    }

    #[test]
    fn an_unlocked_copy_accepts_a_write_and_the_locked_one_refuses_it() {
        let (_repo, git) = repo(&[
            ("records/a/one.md", "one"),
            ("src/x.rs", "not evidence"),
            ("landings/l.json", "{}"),
        ]);
        let out = tempfile::tempdir().unwrap();
        let dest = out.path().join("evidence");
        let _guard = Unlock(dest.clone());

        // The same writes, on the copy before the lock: they work, so the refusal below is the lock's.
        write(&git, "HEAD", &dest).unwrap();
        unlock(&dest).unwrap();
        std::fs::write(dest.join("new.txt"), "x").expect("an unlocked copy accepts a new file");
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(dest.join(HISTORY_FILE))
            .expect("an unlocked copy accepts a write to an existing file");
        std::io::Write::write_all(&mut file, b" ").unwrap();
        drop(file);
        std::fs::remove_file(dest.join("new.txt")).unwrap();

        lock(&dest).unwrap();
        assert!(
            std::fs::write(dest.join("new.txt"), "x").is_err(),
            "a new file in the locked snapshot"
        );
        assert!(
            std::fs::OpenOptions::new()
                .append(true)
                .open(dest.join("records/a/one.md"))
                .is_err(),
            "a write to a record in the locked snapshot"
        );
        assert!(
            std::fs::remove_file(dest.join(HISTORY_FILE)).is_err(),
            "removing a file from the locked snapshot"
        );
        assert_eq!(
            std::fs::read_to_string(dest.join("records/a/one.md")).unwrap(),
            "one"
        );
        assert!(!dest.join("src").exists(), "only evidence is copied");
    }

    #[test]
    fn the_snapshot_holds_the_records_the_landings_and_the_first_parent_history() {
        let (repo_dir, git) = repo(&[
            ("records/jobs/x.md", "brief"),
            ("landings/l1.json", "{\"a\":1}"),
        ]);
        std::fs::write(repo_dir.path().join("f"), "1").unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run([
            "commit",
            "-q",
            "-m",
            "second",
            "-m",
            "Athena-Job: 20261010-153012-k7q2",
        ])
        .unwrap();
        let head = git.head().unwrap().unwrap();
        let out = tempfile::tempdir().unwrap();
        let dest = out.path().join("evidence");
        let _guard = Unlock(dest.clone());
        write(&git, &head, &dest).unwrap();

        assert_eq!(
            std::fs::read_to_string(dest.join("records/jobs/x.md")).unwrap(),
            "brief"
        );
        assert_eq!(
            std::fs::read_to_string(dest.join("landings/l1.json")).unwrap(),
            "{\"a\":1}"
        );
        let history = read_history(&dest).unwrap();
        assert_eq!(history.base, head);
        assert_eq!(history.commits.len(), 2);
        assert_eq!(history.commits[0].hash, head);
        assert_eq!(history.commits[0].subject, "second");
        assert_eq!(
            history.commits[0].trailers,
            ["Athena-Job: 20261010-153012-k7q2"]
        );
        assert!(history.commits[0].committed_at.contains('T'));
        assert_eq!(history.commits[1].subject, "first");
    }

    #[test]
    fn the_history_keeps_the_newest_two_hundred_commits() {
        let (_repo, git) = repo(&[("records/a.md", "a")]);
        for n in 0..HISTORY_LIMIT + 5 {
            commit(&git, &format!("commit {n}"));
        }
        let out = tempfile::tempdir().unwrap();
        let dest = out.path().join("evidence");
        let _guard = Unlock(dest.clone());
        write(&git, "HEAD", &dest).unwrap();
        let history = read_history(&dest).unwrap();
        assert_eq!(history.commits.len(), HISTORY_LIMIT);
        assert_eq!(
            history.commits[0].subject,
            format!("commit {}", HISTORY_LIMIT + 4)
        );
    }

    #[test]
    fn a_second_dispatch_replaces_a_locked_snapshot() {
        let (repo_dir, git) = repo(&[("records/a.md", "old")]);
        let out = tempfile::tempdir().unwrap();
        let dest = out.path().join("evidence");
        let _guard = Unlock(dest.clone());
        write(&git, "HEAD", &dest).unwrap();
        std::fs::write(repo_dir.path().join("records/a.md"), "new").unwrap();
        git.run(["commit", "-q", "-a", "-m", "again"]).unwrap();
        write(&git, "HEAD", &dest).unwrap();
        assert_eq!(
            std::fs::read_to_string(dest.join("records/a.md")).unwrap(),
            "new"
        );
    }

    #[test]
    fn a_missing_snapshot_is_an_error_naming_the_directory() {
        let out = tempfile::tempdir().unwrap();
        let missing = out.path().join("nothing-here");
        let message = status(&missing).unwrap_err().to_string();
        assert!(
            message.contains(&missing.display().to_string()),
            "{message}"
        );
    }

    #[test]
    fn a_directory_without_history_is_not_a_snapshot() {
        let out = tempfile::tempdir().unwrap();
        let message = format!("{:#}", status(out.path()).unwrap_err());
        assert!(message.contains("history.json"), "{message}");
    }
}
