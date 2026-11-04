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

/// The history file's name, at the snapshot root (see [`SnapshotRepo::PlainWithHistory`]).
pub const HISTORY_FILE: &str = "HISTORY.txt";

/// How many commits the history file lists at most.
pub const HISTORY_LIMIT: usize = 200;

/// How many first-parent commits are scanned to find [`HISTORY_LIMIT`] visible ones; bounds the time on
/// large histories.
const HISTORY_SCAN_LIMIT: usize = 2000;

/// The commit-view file's name, at the snapshot root (see [`Snapshot::add_commit_views`]).
pub const COMMITS_FILE: &str = "ATHENA-COMMITS.txt";

/// How many requested commits get a view at most; later hashes are ignored, with a note in the file.
pub const MAX_REQUESTED_COMMITS: usize = 8;

/// How many bytes of diff one commit's view holds at most.
const COMMIT_DIFF_LIMIT: usize = 200 * 1024;

/// What [`Snapshot::add_commit_views`] did, by the hashes as requested.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommitViews {
    /// The full hashes of the commits shown.
    pub shown: Vec<String>,
    /// The requested hashes that got no view: not a commit, not in the base commit's history, or a commit that
    /// changed only paths hidden from agents. The file says so in one shared note, which does not tell these
    /// apart.
    pub omitted: Vec<String>,
}

/// What repository a snapshot gets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotRepo {
    /// A plain directory (the reader, which has no Bash).
    None,
    /// A plain directory, no `.git`, plus [`HISTORY_FILE`] at its root: the newest [`HISTORY_LIMIT`] commits
    /// of the source commit's first-parent history (the planner and critic, which read with Read and Grep).
    PlainWithHistory,
    /// One commit, "base" (the producer).
    Base,
    /// Two commits, "base" then "candidate" (the verifier); the work tree holds the candidate.
    BaseAndCandidate { candidate_commit: String },
    /// One commit, "base" (as [`SnapshotRepo::Base`]), but the work tree holds the visible files of
    /// `tree_commit`, uncommitted (a producer retry seeded from a rejected candidate). The change list is
    /// still against the base, and [`Snapshot::unchanged_since_seed`] tells whether the work tree is as seeded.
    BaseThenTree { tree_commit: String },
    /// One commit, "base", but the work tree holds a copy of the files in `dir` (the kept work of a producer
    /// that hit its deadline), `.git` left out. The change list is against the base, as for
    /// [`SnapshotRepo::BaseThenTree`].
    BaseThenDir { dir: PathBuf },
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
        if repo == SnapshotRepo::PlainWithHistory {
            let text = history_text(user, source_commit, rules_commit)?;
            std::fs::write(dir.join(HISTORY_FILE), text)?;
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
            SnapshotRepo::BaseThenDir { dir: kept } => {
                clear_work_tree(dir)?;
                copy_files(&kept, dir)?;
                snapshot.seed_tree = Some(snapshot.work_tree_tree()?);
            }
            SnapshotRepo::None | SnapshotRepo::PlainWithHistory | SnapshotRepo::Base => {}
        }
        Ok(snapshot)
    }

    /// Adds [`COMMITS_FILE`] to a snapshot that has a repository: a read-only text view of the commits in
    /// `requested` (hashes or abbreviations the job's request names, at most [`MAX_REQUESTED_COMMITS`]).
    ///
    /// A commit is shown only if it resolves to a commit in `user` and is an ancestor of (or is)
    /// `source_commit`. For each, the file holds the full hash, the date, the subject, the changed paths and a
    /// unified diff of those paths; paths hidden from agents under `rules_commit`'s rules (`records/`,
    /// `athena-exclude`) and submodule entries are dropped before anything is read, and commit bodies and
    /// authors are never written. A commit left with no visible path is not shown.
    ///
    /// Anything else, a mistyped or unknown hash, a commit outside the base's history or one that changed only
    /// hidden paths, is omitted, not refused: the call still returns `Ok`, and the file carries one shared note
    /// that does not say which reason applied, so it does not confirm that a hidden commit exists.
    ///
    /// No git object, ref, alternate or remote enters the snapshot; the user's repository is only read. The
    /// file is listed in the snapshot's `.git/info/exclude`, so it is not part of any candidate, change list or
    /// work-tree tree. An empty `requested` changes nothing.
    pub fn add_commit_views(
        &self,
        user: &Git,
        source_commit: &str,
        rules_commit: &str,
        requested: &[String],
    ) -> Result<CommitViews> {
        if requested.is_empty() {
            return Ok(CommitViews::default());
        }
        let git_dir = self.dir.join(".git");
        if !git_dir.is_dir() {
            return Err(crate::GitError::Refused(
                "this snapshot has no repository, so it cannot carry a commit view".into(),
            ));
        }
        let git = self.git();
        let tracked = [self.base.as_deref(), self.candidate.as_deref()]
            .into_iter()
            .flatten()
            .try_fold(false, |seen, c| {
                git.path_exists(c, COMMITS_FILE).map(|here| seen || here)
            })?;
        if tracked || self.dir.join(COMMITS_FILE).symlink_metadata().is_ok() {
            return Err(crate::GitError::Refused(format!(
                "the snapshot already has a file named {COMMITS_FILE}"
            )));
        }
        let (used, ignored) = requested.split_at(requested.len().min(MAX_REQUESTED_COMMITS));
        let mut views = CommitViews::default();
        let mut blocks = String::new();
        for token in used {
            let resolved = match resolve_requested(user, source_commit, token)? {
                Some(commit) => commit,
                None => {
                    views.omitted.push(token.clone());
                    continue;
                }
            };
            if views.shown.contains(&resolved) {
                continue;
            }
            match commit_block(user, rules_commit, &resolved)? {
                Some(block) => {
                    blocks.push_str(&block);
                    views.shown.push(resolved);
                }
                None => views.omitted.push(token.clone()),
            }
        }
        let mut out = "# Read-only view of the commits the request names, built by Athena from the user's repository.\n\
             # Paths hidden from agents are left out. Commit bodies and authors are not shown.\n\
             # Only commits in the base commit's history are shown.\n\n"
            .to_string();
        if !views.omitted.is_empty() {
            out.push_str(&format!(
                "Not available: {}. Each is not a known commit, is not in the base commit's history, or changed only \
                 paths hidden from agents.\n\n",
                views.omitted.join(", ")
            ));
        }
        if !ignored.is_empty() {
            out.push_str(&format!(
                "Ignored: {} more requested hash(es) beyond the first {MAX_REQUESTED_COMMITS}.\n\n",
                ignored.len()
            ));
        }
        out.push_str(&blocks);
        // The exclusion goes in before the file does, so no add can pick the file up.
        let info = git_dir.join("info");
        std::fs::create_dir_all(&info)?;
        let exclude = info.join("exclude");
        let mut rules = std::fs::read_to_string(&exclude).unwrap_or_default();
        if !rules.is_empty() && !rules.ends_with('\n') {
            rules.push('\n');
        }
        rules.push_str(&format!("/{COMMITS_FILE}\n"));
        std::fs::write(&exclude, rules)?;
        std::fs::write(self.dir.join(COMMITS_FILE), out)?;
        Ok(views)
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

/// The full hash `token` names, if it is a commit in `user` that is `source_commit` or one of its ancestors.
/// A token that is not 4 to 40 hexadecimal characters is never given to git.
fn resolve_requested(user: &Git, source_commit: &str, token: &str) -> Result<Option<String>> {
    if !(4..=40).contains(&token.len()) || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Ok(None);
    }
    let Some(commit) = user.rev_parse(token)? else {
        return Ok(None);
    };
    Ok(user.is_ancestor(&commit, source_commit)?.then_some(commit))
}

/// The view of one commit (header, visible paths, diff of those paths), or `None` if it changed paths and none
/// of them is visible to agents under `rules_commit`'s rules.
fn commit_block(user: &Git, rules_commit: &str, commit: &str) -> Result<Option<String>> {
    let from = match user.parents(commit)?.first() {
        Some(parent) => parent.clone(),
        None => user.empty_tree()?,
    };
    let changes = user.diff_tree(&from, commit)?;
    let all: Vec<String> = changes
        .iter()
        .filter(|c| c.old_mode != "160000" && c.new_mode != "160000")
        .map(|c| c.path.clone())
        .collect();
    let classes = classify(user, rules_commit, &all)?;
    let paths: Vec<String> = all
        .iter()
        .filter(|p| {
            classes
                .get(*p)
                .copied()
                .unwrap_or(PathClass::Governed)
                .visible_to_agents()
        })
        .cloned()
        .collect();
    if paths.is_empty() && !changes.is_empty() {
        return Ok(None);
    }
    let (date, subject) = user.date_and_subject(commit)?;
    let mut block = format!("commit {commit}\ndate {date}\nsubject {subject}\n");
    if paths.is_empty() {
        block.push_str("paths\n    (no paths)\n");
    } else {
        block.push_str("paths\n");
        for p in &paths {
            block.push_str(&format!("    {p}\n"));
        }
        let mut diff = user.diff_text(&from, commit, &paths)?;
        if diff.len() > COMMIT_DIFF_LIMIT {
            let mut cut = COMMIT_DIFF_LIMIT;
            while !diff.is_char_boundary(cut) {
                cut -= 1;
            }
            diff.truncate(cut);
            diff.push_str(&format!(
                "\n[diff cut at {} KB; the path list above is complete]\n",
                COMMIT_DIFF_LIMIT / 1024
            ));
        }
        block.push('\n');
        block.push_str(&diff);
        if !block.ends_with('\n') {
            block.push('\n');
        }
    }
    block.push_str("\n----\n\n");
    Ok(Some(block))
}

/// The text of the history file: one block per commit, `<short hash> <date> <subject>` then one indented line
/// per changed path. Paths hidden from agents (`records/`, `athena-exclude`) are dropped, with `rules_commit`'s
/// rules; a commit left with no visible path is omitted, and one that changed no path at all is listed as
/// `(no paths)`. The limit counts commits after that omission.
fn history_text(user: &Git, source_commit: &str, rules_commit: &str) -> Result<String> {
    let commits = user.log_history(source_commit, HISTORY_SCAN_LIMIT)?;
    let mut all: Vec<String> = commits.iter().flat_map(|c| c.paths.clone()).collect();
    all.sort();
    all.dedup();
    let classes = classify(user, rules_commit, &all)?;
    let visible = |p: &String| {
        classes
            .get(p)
            .copied()
            .unwrap_or(PathClass::Governed)
            .visible_to_agents()
    };
    let mut out = format!(
        "# Last {HISTORY_LIMIT} commits (newest first): short hash, date, subject, changed paths.\n\
         # Paths hidden from agents are left out; a commit that changed only such paths is not listed.\n\n"
    );
    let mut listed = 0;
    for c in &commits {
        let paths: Vec<&String> = c.paths.iter().filter(|p| visible(p)).collect();
        if paths.is_empty() && !c.paths.is_empty() {
            continue;
        }
        if listed == HISTORY_LIMIT {
            break;
        }
        listed += 1;
        out.push_str(&format!("{} {} {}\n", c.hash, c.date, c.subject));
        if paths.is_empty() {
            out.push_str("    (no paths)\n");
        }
        for p in paths {
            out.push_str(&format!("    {p}\n"));
        }
    }
    if listed == 0 {
        out.push_str("(no commits)\n");
    }
    Ok(out)
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

/// Copies every file under `from` into `to`, keeping modes and symlinks and leaving the top-level `.git` out.
fn copy_files(from: &Path, to: &Path) -> Result<()> {
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        // A kept directory's own commit view is never carried into a new snapshot.
        if entry.file_name() == ".git" || entry.file_name() == COMMITS_FILE {
            continue;
        }
        let target = to.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            std::fs::create_dir_all(&target)?;
            copy_files_below(&entry.path(), &target)?;
        } else if kind.is_symlink() {
            std::os::unix::fs::symlink(std::fs::read_link(entry.path())?, &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Like [`copy_files`], for a directory below the top level, where a `.git` is an ordinary name.
fn copy_files_below(from: &Path, to: &Path) -> Result<()> {
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            std::fs::create_dir_all(&target)?;
            copy_files_below(&entry.path(), &target)?;
        } else if kind.is_symlink() {
            std::os::unix::fs::symlink(std::fs::read_link(entry.path())?, &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
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
    fn a_snapshot_seeded_from_a_kept_directory_copies_its_files_but_not_its_git() {
        let (_dir, git, head) = user_repo();
        let parent = tempfile::tempdir().unwrap();
        let kept = parent.path().join("kept");
        write_files(
            &kept,
            &[
                ("src/added.rs", "// kept\n"),
                ("src/main.rs", "// edited\n"),
            ],
        );
        std::fs::create_dir_all(kept.join(".git")).unwrap();
        std::fs::write(kept.join(".git/marker"), "old").unwrap();
        let snap = Snapshot::create(
            &git,
            &head,
            &head,
            &parent.path().join("snap-dir"),
            SnapshotRepo::BaseThenDir { dir: kept },
        )
        .unwrap();
        assert_eq!(snap.git().rev_list_reverse("HEAD").unwrap().len(), 1);
        assert!(!snap.dir().join(".git/marker").exists());
        assert_eq!(
            std::fs::read_to_string(snap.dir().join("src/added.rs")).unwrap(),
            "// kept\n"
        );
        let mut paths: Vec<String> = snap
            .change_list()
            .unwrap()
            .into_iter()
            .map(|c| c.path)
            .collect();
        paths.sort();
        assert!(paths.contains(&"src/added.rs".to_string()));
        assert!(snap.unchanged_since_seed().unwrap());
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

    /// The user repository's observable state: status, HEAD, refs and configuration.
    fn repo_state(dir: &Path, git: &Git) -> (String, Option<String>, String, Vec<u8>) {
        (
            git.run(["status", "--porcelain"]).unwrap(),
            git.head().unwrap(),
            git.run(["for-each-ref"]).unwrap(),
            std::fs::read(dir.join(".git/config")).unwrap(),
        )
    }

    #[test]
    fn a_history_snapshot_lists_visible_commits_without_hidden_paths_or_a_repository() {
        let (dir, git, _) = user_repo();
        write_files(dir.path(), &[("records/jobs/y.json", "{}")]);
        commit_all(&git, "records only");
        write_files(dir.path(), &[("private/more.md", "x")]);
        commit_all(&git, "private only");
        write_files(
            dir.path(),
            &[("src/lib.rs", "// lib\n"), ("private/again.md", "y")],
        );
        commit_all(&git, "mixed");
        write_files(dir.path(), &[("src/main.rs", "fn main() { 1; }\n")]);
        let head = commit_all(&git, "visible");
        let before = repo_state(dir.path(), &git);
        let parent = tempfile::tempdir().unwrap();
        let snap = Snapshot::create(
            &git,
            &head,
            &head,
            &parent.path().join("snap-history"),
            SnapshotRepo::PlainWithHistory,
        )
        .unwrap();
        let text = std::fs::read_to_string(snap.dir().join(HISTORY_FILE)).unwrap();
        let blocks: Vec<&str> = text
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with([' ', '#']))
            .collect();
        let subjects: Vec<&str> = blocks
            .iter()
            .map(|l| l.splitn(3, ' ').nth(2).unwrap())
            .collect();
        assert_eq!(
            subjects,
            ["visible", "mixed", "exec bit and symlink", "init"]
        );
        // Each entry has a short hash, a date, and the visible paths only.
        let date = blocks[0].split(' ').nth(1).unwrap();
        assert_eq!(date.len(), 10, "{date}");
        assert!(text.contains("\n    src/main.rs\n"));
        assert!(text.contains("\n    src/lib.rs\n"));
        assert!(text.contains("\n    SPEC.md\n"));
        assert!(!text.contains("records"), "{text}");
        assert!(!text.contains("private"), "{text}");
        assert!(!text.contains("FRONT.md"), "{text}");
        assert!(!text.contains("again.md"), "{text}");
        // No repository, and the user's repository is as it was.
        assert!(!snap.dir().join(".git").exists());
        assert!(snap.dir().join("src/main.rs").exists());
        assert_eq!(repo_state(dir.path(), &git), before);
    }

    #[test]
    fn a_history_snapshot_is_capped_at_the_limit_counted_after_exclusion() {
        let (dir, git, _) = user_repo();
        for i in 0..(HISTORY_LIMIT + 5) {
            write_files(dir.path(), &[("src/n.txt", &format!("{i}\n"))]);
            commit_all(&git, &format!("change {i}"));
            if i % 50 == 0 {
                write_files(dir.path(), &[("records/n.txt", &format!("{i}\n"))]);
                commit_all(&git, &format!("records {i}"));
            }
        }
        let head = git.head().unwrap().unwrap();
        let parent = tempfile::tempdir().unwrap();
        let snap = Snapshot::create(
            &git,
            &head,
            &head,
            &parent.path().join("snap-cap"),
            SnapshotRepo::PlainWithHistory,
        )
        .unwrap();
        let text = std::fs::read_to_string(snap.dir().join(HISTORY_FILE)).unwrap();
        let entries: Vec<&str> = text
            .lines()
            .filter(|l| !l.is_empty() && !l.starts_with([' ', '#']))
            .collect();
        assert_eq!(entries.len(), HISTORY_LIMIT);
        assert!(entries[0].ends_with("change 204"), "{}", entries[0]);
        assert!(!text.contains("records"));
    }

    #[test]
    fn plain_snapshots_have_no_history_file() {
        let (_dir, git, head) = user_repo();
        let parent = tempfile::tempdir().unwrap();
        let snap = Snapshot::create(
            &git,
            &head,
            &head,
            &parent.path().join("snap-plain"),
            SnapshotRepo::None,
        )
        .unwrap();
        assert!(!snap.dir().join(HISTORY_FILE).exists());
    }

    /// Every file under `dir` (relative path, bytes), `.git` included; symlinks are listed without content.
    fn all_files(dir: &Path) -> Vec<(String, Vec<u8>)> {
        fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                let rel = path.strip_prefix(root).unwrap().display().to_string();
                let kind = entry.file_type().unwrap();
                if kind.is_dir() {
                    out.push((rel, Vec::new()));
                    walk(root, &path, out);
                } else if kind.is_symlink() {
                    out.push((rel, Vec::new()));
                } else {
                    out.push((rel, std::fs::read(&path).unwrap()));
                }
            }
        }
        let mut out = Vec::new();
        walk(dir, dir, &mut out);
        out
    }

    /// Everything a snapshot holds that an agent or its git could print: the file names and bytes, the log of
    /// every ref with patches, and every object in the snapshot's store.
    fn everything_readable(snap: &Snapshot) -> Vec<u8> {
        let mut all = Vec::new();
        for (name, bytes) in all_files(snap.dir()) {
            all.extend_from_slice(name.as_bytes());
            all.push(b'\n');
            all.extend_from_slice(&bytes);
        }
        let git = snap.git();
        for args in [
            vec!["log", "--all", "-p", "--format=fuller"],
            vec!["cat-file", "--batch-all-objects", "--batch"],
        ] {
            all.extend(git.run_bytes(args, None, &[]).unwrap());
        }
        all
    }

    fn contains(haystack: &[u8], needle: &str) -> bool {
        haystack
            .windows(needle.len())
            .any(|w| w == needle.as_bytes())
    }

    fn commit_view(snap: &Snapshot) -> String {
        std::fs::read_to_string(snap.dir().join(COMMITS_FILE)).unwrap()
    }

    fn create(
        git: &Git,
        head: &str,
        name: &str,
        repo: SnapshotRepo,
    ) -> (tempfile::TempDir, Snapshot) {
        let parent = tempfile::tempdir().unwrap();
        let snap = Snapshot::create(git, head, head, &parent.path().join(name), repo).unwrap();
        (parent, snap)
    }

    #[test]
    fn named_commit_is_readable_in_a_producer_snapshot() {
        let (dir, git, _) = user_repo();
        write_files(
            dir.path(),
            &[
                ("src/feature.rs", "pub fn feature() -> u32 { 42 }\n"),
                ("src/main.rs", "fn main() { feature(); }\n"),
            ],
        );
        let extra = commit_all(&git, "Add the feature");
        write_files(dir.path(), &[("src/later.rs", "// later\n")]);
        let later = commit_all(&git, "A later change");
        let (_p, snap) = create(&git, &extra, "snap-named", SnapshotRepo::Base);
        let views = snap
            .add_commit_views(&git, &extra, &extra, std::slice::from_ref(&extra))
            .unwrap();
        assert_eq!(views.shown, vec![extra.clone()]);
        assert!(views.omitted.is_empty());
        let text = commit_view(&snap);
        assert!(text.contains(&format!("commit {extra}\n")), "{text}");
        assert!(text.contains("subject Add the feature\n"), "{text}");
        assert!(text.contains("\n    src/feature.rs\n"), "{text}");
        assert!(text.contains("\n    src/main.rs\n"), "{text}");
        assert!(text.contains("+++ b/src/feature.rs"), "{text}");
        assert!(text.contains("+pub fn feature() -> u32 { 42 }"), "{text}");
        assert!(text.contains("-fn main() {}"), "{text}");
        let date = text
            .lines()
            .find_map(|l| l.strip_prefix("date "))
            .expect("a date line");
        assert_eq!(date.len(), 10, "{date}");
        // An abbreviation of a commit that is an ancestor of the base, not the base itself.
        let (_p2, later_snap) = create(&git, &later, "snap-abbrev", SnapshotRepo::Base);
        let short = extra[..7].to_string();
        let views = later_snap
            .add_commit_views(&git, &later, &later, &[short])
            .unwrap();
        assert_eq!(views.shown, vec![extra.clone()]);
        assert!(commit_view(&later_snap).contains("+++ b/src/feature.rs"));
        assert!(!commit_view(&later_snap).contains("src/later.rs"));
    }

    #[test]
    fn hidden_paths_never_appear_in_the_commit_view() {
        let (dir, git, _) = user_repo();
        write_files(
            dir.path(),
            &[
                ("src/main.rs", "fn main() { /* MARK_VISIBLE_22b8 */ }\n"),
                ("FRONT.md", "MARK_FRONT_7f3a"),
                ("private/notes.md", "MARK_PRIV_91c2"),
                ("records/jobs/y.json", "{\"MARK_REC_5d10\":1}"),
            ],
        );
        let requested = commit_all(&git, "Change main with hidden files beside it");
        write_files(
            dir.path(),
            &[
                ("records/jobs/z.json", "{\"MARK_ONLYREC_0e44\":1}"),
                ("private/q.md", "MARK_ONLYPRIV_6b19"),
            ],
        );
        let hidden_only = commit_all(&git, "Only hidden subject xyz");
        write_files(
            dir.path(),
            &[("src/main.rs", "fn main() { /* later */ }\n")],
        );
        let base = commit_all(&git, "later");
        let markers = [
            "MARK_FRONT_7f3a",
            "MARK_PRIV_91c2",
            "MARK_REC_5d10",
            "MARK_ONLYREC_0e44",
            "MARK_ONLYPRIV_6b19",
        ];
        let names = [
            "private/notes.md",
            "private/q.md",
            "records/jobs/y.json",
            "records/jobs/z.json",
        ];
        // The control: git itself prints all of it for the same commits, so the assertions below can fail.
        let unfiltered: Vec<u8> = [&requested, &hidden_only]
            .iter()
            .flat_map(|c| git.run_bytes(["show", c.as_str()], None, &[]).unwrap())
            .collect();
        for needle in markers.iter().chain(names.iter()) {
            assert!(contains(&unfiltered, needle), "control lacks {needle}");
        }
        assert!(contains(&unfiltered, "FRONT.md"));
        let asked = vec![requested.clone(), hidden_only.clone()];
        let candidate = SnapshotRepo::BaseAndCandidate {
            candidate_commit: base.clone(),
        };
        for (name, repo) in [("snap-base", SnapshotRepo::Base), ("snap-two", candidate)] {
            let (_p, snap) = create(&git, &base, name, repo);
            let views = snap.add_commit_views(&git, &base, &base, &asked).unwrap();
            // (f) The commit that touched only hidden paths is under the note, with no subject.
            assert_eq!(views.shown, vec![requested.clone()]);
            assert_eq!(views.omitted, vec![hidden_only.clone()]);
            let text = commit_view(&snap);
            assert!(text.contains("Not available:"), "{text}");
            assert!(!text.contains("Only hidden subject"), "{text}");
            // The visible part of the same commit is there, so the checks are not vacuous.
            assert!(text.contains("MARK_VISIBLE_22b8"), "{text}");
            assert!(text.contains("subject Change main with hidden files"));
            let seen = everything_readable(&snap);
            // (a) no hidden content anywhere, (b) no hidden path name anywhere.
            for needle in markers.iter().chain(names.iter()) {
                assert!(!contains(&seen, needle), "{name}: {needle} is readable");
            }
            // (c) FRONT.md is named by the visible root .gitattributes, as in any snapshot; the view is clean.
            assert!(!text.contains("FRONT.md"), "{text}");
            assert!(!text.contains("records"), "{text}");
            assert!(!text.contains("private"), "{text}");
        }
        // (e) the history view of the same repository leaves the hidden paths out too.
        let (_p, plain) = create(&git, &base, "snap-history", SnapshotRepo::PlainWithHistory);
        let history = std::fs::read(plain.dir().join(HISTORY_FILE)).unwrap();
        for needle in markers.iter().chain(names.iter()) {
            assert!(!contains(&history, needle), "history has {needle}");
        }
    }

    #[test]
    fn a_commit_that_edits_gitattributes_shows_the_rule_it_adds() {
        // The limit: a visible .gitattributes names the paths it hides, in the snapshot and in the view.
        let (dir, git, _) = user_repo();
        write_files(
            dir.path(),
            &[(
                ".gitattributes",
                "/FRONT.md athena-exclude\n/SPEC.md athena-readonly\n/OTHER.md athena-exclude\n",
            )],
        );
        let edited = commit_all(&git, "Hide another file");
        let (_p, snap) = create(&git, &edited, "snap-attrs", SnapshotRepo::Base);
        snap.add_commit_views(&git, &edited, &edited, std::slice::from_ref(&edited))
            .unwrap();
        let text = commit_view(&snap);
        assert!(text.contains("+/OTHER.md athena-exclude"), "{text}");
    }

    #[test]
    fn commit_view_gives_the_snapshot_no_path_to_the_users_repository() {
        let (dir, git, head) = user_repo();
        write_files(dir.path(), &[("src/main.rs", "fn main() { 2; }\n")]);
        let extra = commit_all(&git, "Change main");
        let user_state = |git: &Git| {
            (
                git.run_bytes(["for-each-ref"], None, &[]).unwrap(),
                git.run_bytes(
                    ["cat-file", "--batch-all-objects", "--batch-check"],
                    None,
                    &[],
                )
                .unwrap(),
                git.head().unwrap(),
                std::fs::read(dir.path().join(".git/config")).unwrap(),
            )
        };
        let before = user_state(&git);
        let (_p, snap) = create(&git, &extra, "snap-isolated", SnapshotRepo::Base);
        let views = snap
            .add_commit_views(&git, &extra, &extra, std::slice::from_ref(&extra))
            .unwrap();
        assert_eq!(views.shown, vec![extra.clone()]);
        let git_dir = snap.dir().join(".git");
        assert!(!git_dir.join("objects/info/alternates").exists());
        assert!(!git_dir.join("objects/info/http-alternates").exists());
        assert_eq!(snap.git().run(["remote"]).unwrap().trim(), "");
        let config = std::fs::read_to_string(git_dir.join("config")).unwrap();
        assert!(
            !config.contains("url") && !config.contains("alternate"),
            "{config}"
        );
        let user_path = dir.path().display().to_string();
        for (name, bytes) in all_files(snap.dir()) {
            assert!(
                !contains(&bytes, &user_path),
                "{name} names the user's repository"
            );
        }
        // The snapshot's git cannot resolve the real commit; it has only its own one commit.
        assert!(snap.git().rev_parse(&extra).unwrap().is_none());
        assert!(snap.git().rev_parse(&head).unwrap().is_none());
        assert_eq!(snap.git().rev_list_reverse("--all").unwrap().len(), 1);
        assert_eq!(user_state(&git), before);
    }

    #[test]
    fn no_requested_commit_leaves_snapshots_unchanged() {
        let (dir, git, head) = user_repo();
        write_files(dir.path(), &[("src/main.rs", "fn main() { 3; }\n")]);
        let candidate = commit_all(&git, "candidate");
        let (_p, producer) = create(&git, &head, "snap-none-p", SnapshotRepo::Base);
        let (_q, verifier) = create(
            &git,
            &head,
            "snap-none-v",
            SnapshotRepo::BaseAndCandidate {
                candidate_commit: candidate,
            },
        );
        for (snap, commits) in [(&producer, "1"), (&verifier, "2")] {
            let exclude = std::fs::read(snap.dir().join(".git/info/exclude")).unwrap();
            let views = snap.add_commit_views(&git, &head, &head, &[]).unwrap();
            assert_eq!(views, CommitViews::default());
            assert!(!snap.dir().join(COMMITS_FILE).exists());
            assert_eq!(
                std::fs::read(snap.dir().join(".git/info/exclude")).unwrap(),
                exclude,
                "the exclude file is untouched"
            );
            let count = snap.git().run(["rev-list", "--count", "HEAD"]).unwrap();
            assert_eq!(count.trim(), commits);
            assert_eq!(snap.git().run(["status", "--porcelain"]).unwrap(), "");
        }
        assert!(
            !contains(&everything_readable(&producer), COMMITS_FILE),
            "nothing names the view file"
        );
    }

    #[test]
    fn unknown_or_unreachable_hashes_are_omitted_with_a_note() {
        // The chosen behavior is omission with a note, never a refusal and never a panic.
        let (dir, git, _) = user_repo();
        git.run(["checkout", "-q", "-b", "side"]).unwrap();
        write_files(dir.path(), &[("src/side.rs", "// side only\n")]);
        let side = commit_all(&git, "side only change");
        git.run(["checkout", "-q", "--orphan", "orph"]).unwrap();
        write_files(dir.path(), &[("orph.txt", "orphan\n")]);
        let orphan = commit_all(&git, "orphan only change");
        git.run(["checkout", "-q", "-f", "main"]).unwrap();
        let head = git.head().unwrap().unwrap();
        let blob = git.run(["rev-parse", "HEAD:FRONT.md"]).unwrap();
        let asked = vec![
            "deadbeef12".to_string(),
            side[..10].to_string(),
            orphan[..10].to_string(),
            blob.trim()[..10].to_string(),
            "not-a-hash".to_string(),
        ];
        let (_p, snap) = create(&git, &head, "snap-unknown", SnapshotRepo::Base);
        let views = snap.add_commit_views(&git, &head, &head, &asked).unwrap();
        assert!(views.shown.is_empty());
        assert_eq!(views.omitted, asked);
        let text = commit_view(&snap);
        assert!(text.contains("Not available:"), "{text}");
        assert!(!text.contains("diff --git"), "{text}");
        assert!(!text.contains("side only change"), "{text}");
        assert!(!text.contains("orphan only change"), "{text}");
        assert_eq!(snap.git().run(["status", "--porcelain"]).unwrap(), "");
    }

    #[test]
    fn only_the_first_requested_commits_are_used() {
        let (dir, git, _) = user_repo();
        let mut hashes = Vec::new();
        for i in 0..(MAX_REQUESTED_COMMITS + 1) {
            write_files(dir.path(), &[("src/n.txt", &format!("{i}\n"))]);
            hashes.push(commit_all(&git, &format!("change {i}")));
        }
        let head = hashes.last().unwrap().clone();
        let (_p, snap) = create(&git, &head, "snap-many", SnapshotRepo::Base);
        let views = snap.add_commit_views(&git, &head, &head, &hashes).unwrap();
        assert_eq!(views.shown, hashes[..MAX_REQUESTED_COMMITS]);
        let text = commit_view(&snap);
        assert!(text.contains("Ignored: 1 more"), "{text}");
        assert!(!text.contains("change 8"), "{text}");
    }

    #[test]
    fn a_long_diff_is_cut_with_a_visible_line_and_the_paths_stay_complete() {
        let (dir, git, _) = user_repo();
        let big = "x".repeat(100) + "\n";
        write_files(dir.path(), &[("src/big.txt", &big.repeat(3000))]);
        write_files(dir.path(), &[("src/zz.txt", "tail\n")]);
        let head = commit_all(&git, "big");
        let (_p, snap) = create(&git, &head, "snap-big", SnapshotRepo::Base);
        snap.add_commit_views(&git, &head, &head, std::slice::from_ref(&head))
            .unwrap();
        let text = commit_view(&snap);
        assert!(text.contains("[diff cut at 200 KB"), "len {}", text.len());
        assert!(text.contains("\n    src/zz.txt\n"));
        assert!(text.len() < COMMIT_DIFF_LIMIT + 4096);
    }

    #[test]
    fn commit_view_does_not_change_the_work_tree_tree() {
        let (dir, git, head) = user_repo();
        write_files(dir.path(), &[("src/main.rs", "fn main() { 4; }\n")]);
        let candidate = commit_all(&git, "candidate");
        let repos = [
            SnapshotRepo::Base,
            SnapshotRepo::BaseAndCandidate {
                candidate_commit: candidate,
            },
        ];
        for (i, repo) in repos.into_iter().enumerate() {
            let (_p, snap) = create(&git, &head, &format!("snap-tree-{i}"), repo);
            let tree = snap.work_tree_tree().unwrap();
            snap.add_commit_views(&git, &head, &head, std::slice::from_ref(&head))
                .unwrap();
            assert!(snap.dir().join(COMMITS_FILE).exists());
            assert_eq!(snap.work_tree_tree().unwrap(), tree);
            assert_eq!(snap.git().run(["status", "--porcelain"]).unwrap(), "");
            if i == 0 {
                assert!(snap.change_list().unwrap().is_empty());
                write_files(snap.dir(), &[("src/new.rs", "// new\n")]);
                let paths: Vec<String> = snap
                    .change_list()
                    .unwrap()
                    .into_iter()
                    .map(|c| c.path)
                    .collect();
                assert_eq!(paths, ["src/new.rs"]);
            }
        }
    }

    #[test]
    fn a_kept_directory_with_a_stale_commit_view_does_not_carry_it_into_the_new_snapshot() {
        let (_dir, git, head) = user_repo();
        let parent = tempfile::tempdir().unwrap();
        let kept = parent.path().join("kept");
        write_files(
            &kept,
            &[
                (COMMITS_FILE, "an old view\n"),
                ("src/added.rs", "// kept\n"),
            ],
        );
        let snap = Snapshot::create(
            &git,
            &head,
            &head,
            &parent.path().join("snap-resumed"),
            SnapshotRepo::BaseThenDir { dir: kept },
        )
        .unwrap();
        assert!(!snap.dir().join(COMMITS_FILE).exists());
        let paths: Vec<String> = snap
            .change_list()
            .unwrap()
            .into_iter()
            .map(|c| c.path)
            .collect();
        // The kept files are the new work tree, so the base's other files show as deleted; the stale view is
        // neither a change nor a file.
        assert!(paths.contains(&"src/added.rs".to_string()), "{paths:?}");
        assert!(!paths.iter().any(|p| p == COMMITS_FILE), "{paths:?}");
    }

    #[test]
    fn a_snapshot_that_cannot_carry_the_view_refuses_instead_of_overwriting() {
        let (dir, git, head) = user_repo();
        let (_p, plain) = create(&git, &head, "snap-plain-view", SnapshotRepo::None);
        let err = plain
            .add_commit_views(&git, &head, &head, std::slice::from_ref(&head))
            .unwrap_err();
        assert!(err.to_string().contains("no repository"), "{err}");
        write_files(dir.path(), &[(COMMITS_FILE, "tracked by the user\n")]);
        let tracked = commit_all(&git, "track a file with the view's name");
        let (_q, snap) = create(&git, &tracked, "snap-tracked-view", SnapshotRepo::Base);
        let err = snap
            .add_commit_views(&git, &tracked, &tracked, std::slice::from_ref(&tracked))
            .unwrap_err();
        assert!(err.to_string().contains("already has a file"), "{err}");
        assert_eq!(
            std::fs::read_to_string(snap.dir().join(COMMITS_FILE)).unwrap(),
            "tracked by the user\n"
        );
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
