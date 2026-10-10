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
}

/// A materialized snapshot.
#[derive(Debug)]
pub struct Snapshot {
    dir: PathBuf,
    base: Option<String>,
    candidate: Option<String>,
}

impl Snapshot {
    /// Builds a snapshot of `source_commit` in `dir`, which must not exist yet, hiding what the rules of
    /// `rules_commit` mark private and everything under `records/`.
    pub fn create(user: &Git, source_commit: &str, rules_commit: &str, dir: &Path, repo: SnapshotRepo) -> Result<Self> {
        if dir.exists() {
            return Err(crate::GitError::Refused(format!("snapshot directory {} already exists", dir.display())));
        }
        std::fs::create_dir_all(dir)?;
        materialize(user, source_commit, rules_commit, dir)?;
        let mut snapshot = Snapshot { dir: dir.to_path_buf(), base: None, candidate: None };
        if repo == SnapshotRepo::None {
            return Ok(snapshot);
        }
        let git = snapshot.git();
        git.run(["init", "-q", "-b", "athena"])?;
        snapshot.base = Some(snapshot.commit_all("base")?);
        if let SnapshotRepo::BaseAndCandidate { candidate_commit } = repo {
            clear_work_tree(dir)?;
            materialize(user, &candidate_commit, rules_commit, dir)?;
            snapshot.candidate = Some(snapshot.commit_all("candidate")?);
        }
        Ok(snapshot)
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
        git.run(["commit", "-q", "--allow-empty", "--no-verify", "-m", message])?;
        Ok(git.head()?.expect("a commit was just made"))
    }

    /// Everything that changed in the work tree since the base commit, without rename detection.
    pub fn change_list(&self) -> Result<Vec<DiffEntry>> {
        let base = self.base.as_deref().ok_or_else(|| crate::GitError::Refused("this snapshot has no base commit".into()))?;
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
pub fn materialize(user: &Git, commit: &str, rules_commit: &str, dir: &Path) -> Result<Vec<String>> {
    let entries = user.ls_tree(commit)?;
    let paths: Vec<String> = entries.iter().map(|e| e.path.clone()).collect();
    let classes = classify(user, rules_commit, &paths)?;
    let index = TempIndex::from_tree(user, commit)?;
    let mut removals = Vec::new();
    let mut visible = Vec::new();
    for entry in &entries {
        let class = classes.get(&entry.path).copied().unwrap_or(PathClass::Governed);
        if !class.visible_to_agents() || entry.mode == "160000" {
            removals.push(IndexEdit::Remove { path: entry.path.clone() });
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
            (".gitattributes", "/FRONT.md athena-exclude\n/SPEC.md athena-readonly\n"),
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
            std::fs::set_permissions(dir.path().join("src/run.sh"), std::fs::Permissions::from_mode(0o755)).unwrap();
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
            let mode = std::fs::metadata(snap.dir().join("src/run.sh")).unwrap().permissions().mode();
            assert_eq!(mode & 0o111, 0o111, "the executable bit survives");
            assert!(std::fs::symlink_metadata(snap.dir().join("src/link.rs")).unwrap().file_type().is_symlink());
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
        assert!(inner.show_file("HEAD", "private/notes.md").unwrap().is_none());
        assert!(inner.rev_parse(&head).unwrap().is_none(), "the user's commit is not in the snapshot");
        assert_eq!(inner.rev_list_reverse("--all").unwrap().len(), 1);
        assert_eq!(std::fs::read(dir.path().join(".git/config")).unwrap(), config_before);
    }

    #[test]
    fn producer_edits_become_a_change_list() {
        let (_dir, git, head) = user_repo();
        let parent = tempfile::tempdir().unwrap();
        let snap_dir = parent.path().join("snap-edits");
        let snap = Snapshot::create(&git, &head, &head, &snap_dir, SnapshotRepo::Base).unwrap();
        write_files(snap.dir(), &[("src/main.rs", "fn main() { /* edited */ }\n"), ("src/new.rs", "pub fn f() {}\n")]);
        std::fs::remove_file(snap.dir().join("SPEC.md")).unwrap();
        let changes = snap.change_list().unwrap();
        let summary: Vec<(char, &str)> = changes.iter().map(|c| (c.status, c.path.as_str())).collect();
        assert_eq!(summary, vec![('D', "SPEC.md"), ('M', "src/main.rs"), ('A', "src/new.rs")]);
    }

    #[test]
    fn verifier_snapshots_carry_base_then_candidate() {
        let (dir, git, head) = user_repo();
        write_files(dir.path(), &[("src/main.rs", "fn main() { println!(\"candidate\"); }\n")]);
        let candidate = commit_all(&git, "candidate");
        let parent = tempfile::tempdir().unwrap();
        let snap_dir = parent.path().join("snap-verify");
        let snap = Snapshot::create(
            &git,
            &head,
            &head,
            &snap_dir,
            SnapshotRepo::BaseAndCandidate { candidate_commit: candidate.clone() },
        )
        .unwrap();
        let inner = snap.git();
        assert_eq!(inner.rev_list_reverse("HEAD").unwrap().len(), 2);
        let diff = inner.diff_tree("HEAD~1", "HEAD").unwrap();
        assert_eq!(diff.len(), 1);
        assert_eq!(diff[0].path, "src/main.rs");
        assert!(std::fs::read_to_string(snap.dir().join("src/main.rs")).unwrap().contains("candidate"));
    }

    #[test]
    fn restore_returns_a_work_tree_to_a_tree() {
        let (_dir, git, head) = user_repo();
        let parent = tempfile::tempdir().unwrap();
        let snap_dir = parent.path().join("snap-restore");
        let snap = Snapshot::create(&git, &head, &head, &snap_dir, SnapshotRepo::Base).unwrap();
        let tree = snap.work_tree_tree().unwrap();
        write_files(snap.dir(), &[("junk.txt", "left by a check"), ("src/main.rs", "changed")]);
        let restored = snap.restore_to(&tree).unwrap();
        assert_eq!(restored, vec!["junk.txt".to_string(), "src/main.rs".to_string()]);
        assert_eq!(snap.work_tree_tree().unwrap(), tree);
    }
}
