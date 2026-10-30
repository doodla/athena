//! The evidence snapshot: a read-only copy of the records and of main's first-parent history, taken at dispatch.
//!
//! A job's working copy has no `records/` and a single commit, by design. A producer or verifier that must check
//! a change against the real records or main's history reads this snapshot instead. It lives under the job's
//! own directory, outside the working copy, and holds:
//!
//! - `records/`: every file under `records/` at the base commit, byte for byte, in the same layout;
//! - `landings/`: the landing files [`Records::load`] reads at that commit (the tree's and the records branch's);
//! - `history.json`: an array of [`HistoryCommit`], newest first, the first-parent log of the base commit,
//!   at most [`HISTORY_LIMIT`] commits. `committed_at` is RFC 3339 (the committer date); `trailers` are the
//!   commit message's trailers in order.
//!
//! [`write`] fills a directory; [`lock`] then makes it read-only, with mode bits and the immutable attribute
//! (mode bits alone do not stop a process that runs as root), and proves the lock with write probes.
//! `athena status --snapshot <dir>` reads it back through [`load_records`] and [`read_history`].

use crate::state::StateDir;
use crate::store::{Records, landing_files, parse_landings};
use anyhow::{Context, Result, bail};
use athena_data::JobId;
use athena_data::records::check_record_file;
use athena_gate::records::{RecordFile, RecordSet};
use athena_git::Git;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// The history file at the root of a snapshot.
pub const HISTORY_FILE: &str = "history.json";

/// The most commits `history.json` holds.
pub const HISTORY_LIMIT: usize = 200;

/// One trailer of a commit message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trailer {
    pub key: String,
    pub value: String,
}

/// One commit of main's first-parent history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryCommit {
    pub hash: String,
    /// The committer date, RFC 3339.
    pub committed_at: String,
    pub subject: String,
    pub trailers: Vec<Trailer>,
}

/// Where the evidence snapshot of `job` lives: beside the working copy, not in it.
pub fn evidence_dir(state: &StateDir, job: &JobId) -> PathBuf {
    state.job_dir(job).join("evidence")
}

/// The first-parent log of `head`, newest first, at most [`HISTORY_LIMIT`] commits.
pub fn history_of(git: &Git, head: &str) -> Result<Vec<HistoryCommit>> {
    let out = git.run([
        "log".to_string(),
        "--first-parent".into(),
        format!("-n{HISTORY_LIMIT}"),
        "--format=%H%x1f%cI%x1f%s%x1f%(trailers:only,unfold)%x1e".into(),
        head.to_string(),
    ])?;
    let mut commits = Vec::new();
    for entry in out.split('\x1e') {
        let entry = entry.trim_matches(['\n', '\r']);
        if entry.is_empty() {
            continue;
        }
        let mut parts = entry.splitn(4, '\x1f');
        let (Some(hash), Some(committed_at), Some(subject), Some(trailers)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            bail!("unreadable git log entry: {entry:?}");
        };
        let trailers = trailers
            .lines()
            .filter_map(|line| line.split_once(':'))
            .map(|(key, value)| Trailer {
                key: key.trim().to_string(),
                value: value.trim().to_string(),
            })
            .collect();
        commits.push(HistoryCommit {
            hash: hash.to_string(),
            committed_at: committed_at.to_string(),
            subject: subject.to_string(),
            trailers,
        });
    }
    Ok(commits)
}

/// Writes the snapshot of `head` into `dir`, which must not exist yet. The files are plain and writable until
/// [`lock`] is called.
pub fn write(git: &Git, head: &str, dir: &Path) -> Result<()> {
    if dir.exists() {
        bail!("{} already exists", dir.display());
    }
    std::fs::create_dir_all(dir)?;
    let entries: Vec<_> = git
        .ls_tree_under(head, "records")?
        .into_iter()
        .filter(|e| e.kind == "blob")
        .collect();
    let oids: Vec<String> = entries.iter().map(|e| e.oid.clone()).collect();
    let blobs = git.cat_blobs(&oids)?;
    for (entry, bytes) in entries.iter().zip(blobs) {
        write_file(dir, &entry.path, &bytes)?;
    }
    // The records directory exists even when the repository holds no record yet.
    std::fs::create_dir_all(dir.join("records"))?;
    for (path, bytes) in landing_files(git, head) {
        let name = Path::new(&path)
            .file_name()
            .context("a landing file without a name")?;
        let target = dir.join("landings").join(name);
        if !target.exists() {
            write_file(dir, &format!("landings/{}", name.to_string_lossy()), &bytes)?;
        }
    }
    let history = history_of(git, head)?;
    std::fs::write(
        dir.join(HISTORY_FILE),
        serde_json::to_string_pretty(&history)? + "\n",
    )?;
    Ok(())
}

/// Writes `bytes` at `relative` under `dir`; a path that climbs out of `dir` is refused.
fn write_file(dir: &Path, relative: &str, bytes: &[u8]) -> Result<()> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        bail!("{relative:?} is not a path inside the snapshot");
    }
    let target = dir.join(path);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&target, bytes).with_context(|| format!("writing {}", target.display()))
}

/// Makes the snapshot at `dir` read-only: files 0444, directories 0555, then the immutable attribute, then
/// two probes that must fail (creating a file, and opening an existing one for writing). A process that runs
/// as root ignores mode bits, so the probes, not the modes, decide; if either succeeds the snapshot is
/// unlocked again and the call fails.
pub fn lock(dir: &Path) -> Result<()> {
    set_modes(dir)?;
    // Where `chattr` or its privilege is missing, the probes below say whether the modes are enough.
    let _ = std::process::Command::new("chattr")
        .arg("-R")
        .arg("+i")
        .arg(dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    let created = dir.join(".athena-write-probe");
    let create_worked = std::fs::write(&created, b"x").is_ok();
    let existing = dir.join(HISTORY_FILE);
    let open_worked = std::fs::OpenOptions::new()
        .write(true)
        .open(&existing)
        .is_ok();
    if create_worked || open_worked {
        let _ = std::fs::remove_file(&created);
        let _ = athena_git::snapshot::unlock_dir(dir);
        bail!(
            "the evidence snapshot at {} could not be made read-only (a write succeeded after locking)",
            dir.display()
        );
    }
    Ok(())
}

#[cfg(unix)]
fn set_modes(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let meta = std::fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        return Ok(());
    }
    if meta.is_dir() {
        for child in std::fs::read_dir(path)? {
            set_modes(&child?.path())?;
        }
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o555))?;
    } else {
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o444))?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn set_modes(path: &Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.is_dir() {
        for child in std::fs::read_dir(path)? {
            set_modes(&child?.path())?;
        }
    }
    let mut permissions = meta.permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

/// Every file of the snapshot's `records/` as a [`RecordSet`], checked as `RecordSet::load` checks a commit's.
pub fn record_set(dir: &Path) -> Result<RecordSet> {
    let mut files = Vec::new();
    collect(dir, &dir.join("records"), &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(RecordSet {
        files: files
            .into_iter()
            .map(|(path, bytes)| match check_record_file(&path, &bytes) {
                Ok(record) => RecordFile {
                    path,
                    record,
                    error: None,
                    bytes,
                },
                Err(error) => RecordFile {
                    path,
                    record: None,
                    error: Some(error),
                    bytes,
                },
            })
            .collect(),
    })
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e).with_context(|| format!("reading {}", dir.display())),
    };
    for entry in entries {
        let path = entry?.path();
        if path.is_dir() {
            collect(root, &path, out)?;
        } else {
            let relative = path
                .strip_prefix(root)
                .expect("below the snapshot root")
                .to_string_lossy()
                .into_owned();
            out.push((relative, std::fs::read(&path)?));
        }
    }
    Ok(())
}

/// The records of the snapshot at `dir`, loaded as [`Records::load`] loads a commit's.
pub fn load_records(dir: &Path) -> Result<Records> {
    let set = record_set(dir)?;
    let mut landing_bytes = Vec::new();
    collect(dir, &dir.join("landings"), &mut landing_bytes)?;
    let landings = parse_landings(landing_bytes.iter().map(|(_, b)| b.as_slice()));
    Ok(Records::from_set(&set, landings))
}

/// The history of the snapshot at `dir`, newest first.
pub fn read_history(dir: &Path) -> Result<Vec<HistoryCommit>> {
    let path = dir.join(HISTORY_FILE);
    let text = std::fs::read_to_string(&path).with_context(|| {
        format!(
            "{} is not an evidence snapshot: {} is missing or unreadable",
            dir.display(),
            HISTORY_FILE
        )
    })?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::at(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value]).unwrap();
        }
        std::fs::create_dir_all(dir.path().join("records/jobs/x")).unwrap();
        std::fs::write(
            dir.path().join("records/jobs/x/brief-producer.md"),
            "brief\n",
        )
        .unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "First\n\nAthena-Job: bootstrap"])
            .unwrap();
        std::fs::write(dir.path().join("note.txt"), "n\n").unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run([
            "commit",
            "-q",
            "-m",
            "Second: a subject\n\nBody text.\n\nAthena-Job: 20261030-070002-d103\nReviewed-by: someone",
        ])
        .unwrap();
        (dir, git)
    }

    /// Unlocks a snapshot so the temporary directory can be removed.
    struct Unlock<'a>(&'a Path);
    impl Drop for Unlock<'_> {
        fn drop(&mut self) {
            let _ = athena_git::snapshot::unlock_dir(self.0);
        }
    }

    #[test]
    fn history_holds_hash_time_subject_and_trailers_newest_first() {
        let (_repo, git) = repo();
        let head = git.head().unwrap().unwrap();
        let history = history_of(&git, &head).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].hash, head);
        assert_eq!(history[0].subject, "Second: a subject");
        assert_eq!(
            history[0].trailers,
            [
                Trailer {
                    key: "Athena-Job".into(),
                    value: "20261030-070002-d103".into()
                },
                Trailer {
                    key: "Reviewed-by".into(),
                    value: "someone".into()
                },
            ]
        );
        let stamp = git.run(["log", "-1", "--format=%cI", &head]).unwrap();
        assert_eq!(history[0].committed_at, stamp.trim());
        assert!(history[0].committed_at.contains('T'));
        assert_eq!(history[1].subject, "First");
    }

    #[test]
    fn write_copies_the_records_and_the_history_and_load_reads_them_back() {
        let (_repo, git) = repo();
        let head = git.head().unwrap().unwrap();
        let out = tempfile::tempdir().unwrap();
        let dir = out.path().join("evidence");
        write(&git, &head, &dir).unwrap();
        assert_eq!(
            std::fs::read(dir.join("records/jobs/x/brief-producer.md")).unwrap(),
            b"brief\n"
        );
        let set = record_set(&dir).unwrap();
        assert_eq!(set.files.len(), 1);
        assert_eq!(
            read_history(&dir).unwrap(),
            history_of(&git, &head).unwrap()
        );
        assert!(!dir.join("note.txt").exists(), "only records/ is copied");
        assert!(
            write(&git, &head, &dir).is_err(),
            "an existing directory is not overwritten"
        );
    }

    #[test]
    fn a_directory_without_a_history_file_is_named_in_the_error() {
        let out = tempfile::tempdir().unwrap();
        let error = read_history(out.path()).unwrap_err().to_string();
        assert!(error.contains(HISTORY_FILE), "{error}");
    }

    #[test]
    fn lock_refuses_writes_even_as_root_and_an_unlocked_copy_does_not() {
        let (_repo, git) = repo();
        let head = git.head().unwrap().unwrap();
        let out = tempfile::tempdir().unwrap();
        let control = out.path().join("control");
        let locked = out.path().join("locked");
        write(&git, &head, &control).unwrap();
        write(&git, &head, &locked).unwrap();
        let _unlock_control = Unlock(&control);
        let _unlock_locked = Unlock(&locked);
        // Control: without the lock a created file and a rewrite both succeed, in this same environment.
        std::fs::write(control.join("created"), b"x")
            .expect("the unlocked copy accepts a new file");
        std::fs::write(control.join(HISTORY_FILE), b"[]")
            .expect("the unlocked copy accepts a rewrite");

        lock(&locked).unwrap();
        assert!(std::fs::write(locked.join("created"), b"x").is_err());
        assert!(std::fs::write(locked.join("records/jobs/x/brief-producer.md"), b"x").is_err());
        assert!(std::fs::write(locked.join(HISTORY_FILE), b"[]").is_err());
        assert!(std::fs::remove_file(locked.join(HISTORY_FILE)).is_err());
        assert_eq!(
            std::fs::read(locked.join("records/jobs/x/brief-producer.md")).unwrap(),
            b"brief\n",
            "reading still works"
        );
    }
}
