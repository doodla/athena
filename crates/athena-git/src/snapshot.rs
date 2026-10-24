//! Detached snapshots: the directories agents work in (design §4.1, D08).
//!
//! The core writes the visible files of a commit (everything except `records/` and `athena-exclude` paths,
//! under a stated commit's rules) into a fresh directory with a temporary index and `checkout-index`. The
//! snapshot then gets a throwaway repository of its own, which shares no objects, refs, configuration, or
//! hooks with the user's: from inside it, no git command reaches the user's history or hidden content.

use crate::attrs::{PathClass, classify};
use crate::index::{IndexEdit, TempIndex};
use crate::{DiffEntry, Git, Result, parse_raw_diff};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What repository a snapshot gets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotRepo {
    /// A plain directory (the reader, which has no Bash).
    None,
    /// One commit, "base" (the producer).
    Base,
    /// Two commits, "base" then "candidate" (the verifier); the work tree holds the candidate.
    BaseAndCandidate { candidate_commit: String },
    /// One commit, "base" (as [`SnapshotRepo::Base`]), but the work tree holds the visible files of
    /// `tree_commit`, uncommitted (a producer retry seeded from a rejected candidate). The change list is
    /// still against the base, and [`Snapshot::unchanged_since_seed`] tells whether the work tree is as seeded.
    BaseThenTree { tree_commit: String },
}

/// A materialized snapshot.
#[derive(Debug)]
pub struct Snapshot {
    dir: PathBuf,
    base: Option<String>,
    candidate: Option<String>,
    /// The work tree's tree right after seeding (`BaseThenTree` only).
    seed_tree: Option<String>,
}

impl Snapshot {
    /// Builds a snapshot of `source_commit` in `dir`, which must not exist yet, hiding what the rules of
    /// `rules_commit` mark private and everything under `records/`.
    pub fn create(
        user: &Git,
        source_commit: &str,
        rules_commit: &str,
        dir: &Path,
        repo: SnapshotRepo,
    ) -> Result<Self> {
        if dir.exists() {
            return Err(crate::GitError::Refused(format!(
                "snapshot directory {} already exists",
                dir.display()
            )));
        }
        std::fs::create_dir_all(dir)?;
        materialize(user, source_commit, rules_commit, dir)?;
        let mut snapshot = Snapshot {
            dir: dir.to_path_buf(),
            base: None,
            candidate: None,
            seed_tree: None,
        };
        if repo == SnapshotRepo::None {
            return Ok(snapshot);
        }
        let git = snapshot.git();
        git.run(["init", "-q", "-b", "athena"])?;
        snapshot.base = Some(snapshot.commit_all("base")?);
        match repo {
            SnapshotRepo::BaseAndCandidate { candidate_commit } => {
                clear_work_tree(dir)?;
                materialize(user, &candidate_commit, rules_commit, dir)?;
                snapshot.candidate = Some(snapshot.commit_all("candidate")?);
            }
            SnapshotRepo::BaseThenTree { tree_commit } => {
                clear_work_tree(dir)?;
                materialize(user, &tree_commit, rules_commit, dir)?;
                snapshot.seed_tree = Some(snapshot.work_tree_tree()?);
            }
            SnapshotRepo::None | SnapshotRepo::Base => {}
        }
        Ok(snapshot)
    }

    /// Whether the work tree is exactly as seeded by [`SnapshotRepo::BaseThenTree`]. Always false for any
    /// other snapshot, which has no seed.
    pub fn unchanged_since_seed(&self) -> Result<bool> {
        match &self.seed_tree {
            Some(seed) => Ok(*seed == self.work_tree_tree()?),
            None => Ok(false),
        }
    }

    /// The snapshot's own, isolated git.
    pub fn git(&self) -> Git {
        Git::isolated(&self.dir)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// The snapshot repository's base commit, if it has one.
    pub fn base(&self) -> Option<&str> {
        self.base.as_deref()
    }

    /// The snapshot repository's candidate commit, if it has one.
    pub fn candidate(&self) -> Option<&str> {
        self.candidate.as_deref()
    }

    fn commit_all(&self, message: &str) -> Result<String> {
        let git = self.git();
        git.run(["add", "-A"])?;
        git.run([
            "commit",
            "-q",
            "--allow-empty",
            "--no-verify",
            "-m",
            message,
        ])?;
        Ok(git.head()?.expect("a commit was just made"))
    }

    /// Makes the whole snapshot, `.git` included, refuse writes, even from root.
    ///
    /// Write bits come off every file and directory, which stops every user but root. Where the process may
    /// set it (root, on a file system that supports it), the immutable attribute goes on as well, which
    /// stops root too. The lock is then probed: a file is created and an existing one opened for writing.
    /// If either probe succeeds the snapshot is unlocked again and the call fails, so a lock that does not
    /// hold is never taken for one that does.
    pub fn lock(&self) -> Result<()> {
        set_tree_writable(&self.dir, false)?;
        // Best effort: without the privilege or file system support the probe below decides.
        let _ = set_immutable(&self.dir, true);
        if let Some(how) = self.probe_write() {
            let _ = self.unlock();
            return Err(crate::GitError::Refused(format!(
                "the snapshot lock does not hold: {how} succeeded in {}; run as a user the permission bits \
                 bind, or on a file system that supports the immutable attribute",
                self.dir.display()
            )));
        }
        Ok(())
    }

    /// Undoes [`Snapshot::lock`], so the snapshot can be changed and removed again.
    pub fn unlock(&self) -> Result<()> {
        unlock_dir(&self.dir)
    }

    /// The first write that works on a locked snapshot, if any (what it did is undone).
    fn probe_write(&self) -> Option<&'static str> {
        let fresh = self.dir.join(".athena-lock-probe");
        if std::fs::write(&fresh, b"x").is_ok() {
            let _ = std::fs::remove_file(&fresh);
            return Some("creating a file");
        }
        let existing = self.dir.join(".git").join("HEAD");
        if std::fs::OpenOptions::new()
            .append(true)
            .open(existing)
            .is_ok()
        {
            return Some("opening a git file for writing");
        }
        None
    }

    /// Everything that changed in the work tree since the base commit, without rename detection.
    pub fn change_list(&self) -> Result<Vec<DiffEntry>> {
        let base = self
            .base
            .as_deref()
            .ok_or_else(|| crate::GitError::Refused("this snapshot has no base commit".into()))?;
        let tree = self.work_tree_tree()?;
        self.git().diff_tree(base, &tree)
    }

    /// The tree of the work tree's current state, computed in a temporary index (respects `.gitignore`).
    pub fn work_tree_tree(&self) -> Result<String> {
        let git = self.git();
        let file = tempfile::NamedTempFile::new()?.into_temp_path();
        std::fs::remove_file(&file)?;
        let env: [(&str, &OsStr); 1] = [("GIT_INDEX_FILE", file.as_os_str())];
        git.run_bytes(["add", "-A"], None, &env)?;
        let out = git.run_bytes(["write-tree"], None, &env)?;
        Ok(String::from_utf8_lossy(&out).trim().to_string())
    }

    /// The bytes of a blob in the snapshot's own object store.
    pub fn blob(&self, oid: &str) -> Result<Vec<u8>> {
        self.git().cat_blob(oid)
    }

    /// Restores the work tree to exactly `tree` (a tree in the snapshot's repository), removing extra files.
    /// Returns the paths that differed.
    pub fn restore_to(&self, tree: &str) -> Result<Vec<String>> {
        let current = self.work_tree_tree()?;
        if current == tree {
            return Ok(Vec::new());
        }
        let git = self.git();
        let differences: Vec<DiffEntry> = git.diff_tree(tree, &current)?;
        for change in &differences {
            let path = self.dir.join(&change.path);
            match change.status {
                // Present now but not in the target: remove it.
                'A' => {
                    let _ = std::fs::remove_file(&path);
                }
                // Missing now, or different: write the target's version back.
                _ => {
                    let bytes = git.cat_blob(&change.old_oid)?;
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&path, bytes)?;
                    set_mode(&path, &change.old_mode)?;
                }
            }
        }
        Ok(differences.into_iter().map(|d| d.path).collect())
    }
}

/// Writes the files of `commit` that agents may see (under `rules_commit`'s rules) into `dir`.
pub fn materialize(
    user: &Git,
    commit: &str,
    rules_commit: &str,
    dir: &Path,
) -> Result<Vec<String>> {
    let entries = user.ls_tree(commit)?;
    let paths: Vec<String> = entries.iter().map(|e| e.path.clone()).collect();
    let classes = classify(user, rules_commit, &paths)?;
    let index = TempIndex::from_tree(user, commit)?;
    let mut removals = Vec::new();
    let mut visible = Vec::new();
    for entry in &entries {
        let class = classes
            .get(&entry.path)
            .copied()
            .unwrap_or(PathClass::Governed);
        if !class.visible_to_agents() || entry.mode == "160000" {
            removals.push(IndexEdit::Remove {
                path: entry.path.clone(),
            });
        } else {
            visible.push(entry.path.clone());
        }
    }
    index.apply(&removals)?;
    index.checkout_all(dir)?;
    Ok(visible)
}

fn clear_work_tree(dir: &Path) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_name() == ".git" {
            continue;
        }
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            std::fs::remove_dir_all(path)?;
        } else {
            std::fs::remove_file(path)?;
        }
    }
    Ok(())
}

/// Clears the immutable attribute under `dir` and gives every file and directory its owner write bit back.
/// Harmless on a directory that was never locked.
pub fn unlock_dir(dir: &Path) -> Result<()> {
    let _ = set_immutable(dir, false);
    set_tree_writable(dir, true)
}

/// Adds or removes the owner write bit (all write bits, when removing) on `path` and everything under it.
/// Symlinks are left alone.
///
/// An entry that vanishes during the walk (git's own transient lock files and background housekeeping
/// leftovers in `.git`) is already gone, so there is nothing left to change; any other error propagates.
fn set_tree_writable(path: &Path, writable: bool) -> Result<()> {
    walk_tree_writable(path, writable, &mut |_| {})
}

/// `set_tree_writable` with a hook called on each child path after its directory was listed and before the
/// child is examined, so a test can remove entries in that window.
fn walk_tree_writable(path: &Path, writable: bool, between: &mut dyn FnMut(&Path)) -> Result<()> {
    match walk_one(path, writable, between) {
        Err(crate::GitError::Spawn(e)) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

fn walk_one(path: &Path, writable: bool, between: &mut dyn FnMut(&Path)) -> Result<()> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        return Ok(());
    }
    // A directory must be writable again before its children can be reached for changing.
    if writable {
        set_write_bits(path, true)?;
    }
    if meta.is_dir() {
        let children = std::fs::read_dir(path)?
            .map(|e| e.map(|e| e.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        for child in children {
            between(&child);
            walk_tree_writable(&child, writable, between)?;
        }
    }
    if !writable {
        set_write_bits(path, false)?;
    }
    Ok(())
}

#[cfg(unix)]
fn set_write_bits(path: &Path, writable: bool) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mode = std::fs::metadata(path)?.permissions().mode();
    let mode = if writable {
        mode | 0o200
    } else {
        mode & !0o222
    };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_write_bits(path: &Path, writable: bool) -> Result<()> {
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_readonly(!writable);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

/// Sets or clears the immutable attribute under `dir` with the system's `chattr`. Fails where the
/// program, the privilege, or file system support is missing.
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

#[cfg(unix)]
fn set_mode(path: &Path, mode: &str) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perm = if mode == "100755" { 0o755 } else { 0o644 };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(perm))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: &str) -> Result<()> {
    Ok(())
}

/// Parses raw diff output; re-exported for snapshot callers.
pub fn parse_raw(out: &[u8]) -> Result<Vec<DiffEntry>> {
    parse_raw_diff(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    fn user_repo() -> (tempfile::TempDir, Git, String) {
        let (dir, git) = repo_with(&[
            (
                ".gitattributes",
                "/FRONT.md athena-exclude\n/SPEC.md athena-readonly\n",
            ),
            ("private/.gitattributes", "* athena-exclude\n"),
            ("private/notes.md", "owner notes"),
            ("FRONT.md", "front door"),
            ("SPEC.md", "the spec"),
            ("records/jobs/x.json", "{}"),
            ("src/main.rs", "fn main() {}\n"),
            ("src/run.sh", "#!/bin/sh\necho hi\n"),
        ]);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(
                dir.path().join("src/run.sh"),
                std::fs::Permissions::from_mode(0o755),
            )
            .unwrap();
            std::os::unix::fs::symlink("main.rs", dir.path().join("src/link.rs")).unwrap();
        }
        let head = commit_all(&git, "exec bit and symlink");
        (dir, git, head)
    }

    #[test]
    fn snapshots_hold_only_what_agents_may_see() {
        // M11/M12 (blocks): records and owner-private files are absent on disk; read-only files are present.
        let (_dir, git, head) = user_repo();
        let parent = tempfile::tempdir().unwrap();
        let snap_dir = parent.path().join("snap-visible");
        let snap = Snapshot::create(&git, &head, &head, &snap_dir, SnapshotRepo::Base).unwrap();
        assert!(!snap.dir().join("records").exists());
        assert!(!snap.dir().join("FRONT.md").exists());
        assert!(!snap.dir().join("private").exists());
        assert!(snap.dir().join("SPEC.md").exists());
        assert!(snap.dir().join("src/main.rs").exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(snap.dir().join("src/run.sh"))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o111, 0o111, "the executable bit survives");
            assert!(
                std::fs::symlink_metadata(snap.dir().join("src/link.rs"))
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
        }
    }

    #[test]
    fn snapshot_git_cannot_reach_the_users_history_or_hidden_content() {
        // M11 (blocks): from inside a snapshot, git finds nothing of the user's.
        let (dir, git, head) = user_repo();
        let config_before = std::fs::read(dir.path().join(".git/config")).unwrap();
        let parent = tempfile::tempdir().unwrap();
        let snap_dir = parent.path().join("snap-escape");
        let snap = Snapshot::create(&git, &head, &head, &snap_dir, SnapshotRepo::Base).unwrap();
        let inner = snap.git();
        assert!(inner.show_file("HEAD", "FRONT.md").unwrap().is_none());
        assert!(
            inner
                .show_file("HEAD", "private/notes.md")
                .unwrap()
                .is_none()
        );
        assert!(
            inner.rev_parse(&head).unwrap().is_none(),
            "the user's commit is not in the snapshot"
        );
        assert_eq!(inner.rev_list_reverse("--all").unwrap().len(), 1);
        assert_eq!(
            std::fs::read(dir.path().join(".git/config")).unwrap(),
            config_before
        );
    }

    #[test]
    fn producer_edits_become_a_change_list() {
        let (_dir, git, head) = user_repo();
        let parent = tempfile::tempdir().unwrap();
        let snap_dir = parent.path().join("snap-edits");
        let snap = Snapshot::create(&git, &head, &head, &snap_dir, SnapshotRepo::Base).unwrap();
        write_files(
            snap.dir(),
            &[
                ("src/main.rs", "fn main() { /* edited */ }\n"),
                ("src/new.rs", "pub fn f() {}\n"),
            ],
        );
        std::fs::remove_file(snap.dir().join("SPEC.md")).unwrap();
        let changes = snap.change_list().unwrap();
        let summary: Vec<(char, &str)> = changes
            .iter()
            .map(|c| (c.status, c.path.as_str()))
            .collect();
        assert_eq!(
            summary,
            vec![('D', "SPEC.md"), ('M', "src/main.rs"), ('A', "src/new.rs")]
        );
    }

    #[test]
    fn verifier_snapshots_carry_base_then_candidate() {
        let (dir, git, head) = user_repo();
        write_files(
            dir.path(),
            &[("src/main.rs", "fn main() { println!(\"candidate\"); }\n")],
        );
        let candidate = commit_all(&git, "candidate");
        let parent = tempfile::tempdir().unwrap();
        let snap_dir = parent.path().join("snap-verify");
        let snap = Snapshot::create(
            &git,
            &head,
            &head,
            &snap_dir,
            SnapshotRepo::BaseAndCandidate {
                candidate_commit: candidate.clone(),
            },
        )
        .unwrap();
        let inner = snap.git();
        assert_eq!(inner.rev_list_reverse("HEAD").unwrap().len(), 2);
        let diff = inner.diff_tree("HEAD~1", "HEAD").unwrap();
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].path, "src/main.rs");
        assert!(
            std::fs::read_to_string(snap.dir().join("src/main.rs"))
                .unwrap()
                .contains("candidate")
        );
    }

    #[test]
    fn a_seeded_snapshot_diffs_against_the_base_and_notices_an_untouched_seed() {
        let (dir, git, head) = user_repo();
        write_files(
            dir.path(),
            &[
                ("src/main.rs", "fn main() { println!(\"candidate\"); }\n"),
                ("src/added.rs", "// added\n"),
            ],
        );
        let candidate = commit_all(&git, "candidate");
        let parent = tempfile::tempdir().unwrap();
        let snap = Snapshot::create(
            &git,
            &head,
            &head,
            &parent.path().join("snap-seed"),
            SnapshotRepo::BaseThenTree {
                tree_commit: candidate,
            },
        )
        .unwrap();
        assert_eq!(snap.git().rev_list_reverse("HEAD").unwrap().len(), 1);
        assert!(snap.dir().join("src/added.rs").exists());
        let mut paths: Vec<String> = snap
            .change_list()
            .unwrap()
            .into_iter()
            .map(|c| c.path)
            .collect();
        paths.sort();
        assert_eq!(paths, ["src/added.rs", "src/main.rs"]);
        assert!(snap.unchanged_since_seed().unwrap());
        std::fs::write(snap.dir().join("src/added.rs"), "// edited\n").unwrap();
        assert!(!snap.unchanged_since_seed().unwrap());
    }

    #[test]
    fn a_snapshot_without_a_seed_is_never_unchanged_since_seed() {
        let (_dir, git, head) = user_repo();
        let parent = tempfile::tempdir().unwrap();
        let snap = Snapshot::create(
            &git,
            &head,
            &head,
            &parent.path().join("snap-base"),
            SnapshotRepo::Base,
        )
        .unwrap();
        assert!(!snap.unchanged_since_seed().unwrap());
    }

    /// A verifier-shaped snapshot of the test repository.
    fn verifier_snapshot(name: &str) -> (tempfile::TempDir, Snapshot) {
        let (_dir, git, head) = user_repo();
        let parent = tempfile::tempdir().unwrap();
        let snap = Snapshot::create(
            &git,
            &head,
            &head,
            &parent.path().join(name),
            SnapshotRepo::BaseAndCandidate {
                candidate_commit: head.clone(),
            },
        )
        .unwrap();
        (parent, snap)
    }

    /// Tries a file edit, a new file, and a git commit in the snapshot; returns whether each worked.
    fn try_writes(snap: &Snapshot) -> (bool, bool, bool) {
        let edit =
            std::fs::write(snap.dir().join("src/main.rs"), "fn main() { /* x */ }\n").is_ok();
        let create = std::fs::write(snap.dir().join("new.txt"), "new").is_ok();
        let git = snap.git();
        let commit = git
            .run(["add", "-A"])
            .and_then(|_| git.run(["commit", "-q", "--allow-empty", "--no-verify", "-m", "x"]))
            .is_ok();
        (edit, create, commit)
    }

    #[test]
    fn an_unlocked_snapshot_takes_edits_and_commits() {
        // The control for the lock test: the same writes succeed when nothing locks the snapshot.
        let (_parent, snap) = verifier_snapshot("snap-open");
        let head = snap.git().head().unwrap();
        assert_eq!(try_writes(&snap), (true, true, true));
        assert_ne!(snap.git().head().unwrap(), head, "the commit moved HEAD");
    }

    #[test]
    fn a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree() {
        // Fails (never skips) when the lock does not hold: `lock` itself errors then.
        let (_parent, snap) = verifier_snapshot("snap-locked");
        let head = snap.git().head().unwrap();
        let tree = snap.work_tree_tree().unwrap();
        snap.lock().expect("the snapshot lock must hold");
        assert_eq!(try_writes(&snap), (false, false, false));
        // Reading the tree needs git to touch object files, so it waits for the unlock.
        snap.unlock().unwrap();
        assert_eq!(snap.git().head().unwrap(), head, "HEAD is unchanged");
        assert_eq!(
            snap.work_tree_tree().unwrap(),
            tree,
            "the tree is unchanged"
        );
        assert_eq!(
            try_writes(&snap),
            (true, true, true),
            "unlock restores writes"
        );
        // A locked snapshot can still be removed after unlocking.
        snap.lock().unwrap();
        snap.unlock().unwrap();
        std::fs::remove_dir_all(snap.dir()).unwrap();
    }

    #[test]
    fn a_walk_survives_entries_removed_after_the_directory_was_listed() {
        let dir = tempfile::tempdir().unwrap();
        write_files(
            dir.path(),
            &[("a/one", "1"), ("a/two", "2"), ("b/three", "3"), ("c", "4")],
        );
        // Remove every file and the directory "b" just before the walk reaches them, as git does with its
        // transient lock files.
        let mut removed = 0;
        walk_tree_writable(dir.path(), false, &mut |child| {
            if child.is_file() || child.ends_with("b") {
                let _ = std::fs::remove_file(child).or_else(|_| std::fs::remove_dir_all(child));
                removed += 1;
            }
        })
        .unwrap();
        assert!(removed >= 3, "the hook removed entries mid-walk");
        // A path that is already gone is nothing to do.
        assert!(set_tree_writable(&dir.path().join("a/one/x"), true).is_ok());
        unlock_dir(dir.path()).unwrap();
    }

    #[test]
    fn restore_returns_a_work_tree_to_a_tree() {
        let (_dir, git, head) = user_repo();
        let parent = tempfile::tempdir().unwrap();
        let snap_dir = parent.path().join("snap-restore");
        let snap = Snapshot::create(&git, &head, &head, &snap_dir, SnapshotRepo::Base).unwrap();
        let tree = snap.work_tree_tree().unwrap();
        write_files(
            snap.dir(),
            &[("junk.txt", "left by a check"), ("src/main.rs", "changed")],
        );
        let restored = snap.restore_to(&tree).unwrap();
        assert_eq!(
            restored,
            vec!["junk.txt".to_string(), "src/main.rs".to_string()]
        );
        assert_eq!(snap.work_tree_tree().unwrap(), tree);
    }
}
