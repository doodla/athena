//! A writer for the records branch `athena/records` (§8.2 of `docs/design/records-history.md`, D111).
//!
//! The records history is a branch with its own root that is never merged into main. Several jobs may write
//! to it at once, so every update is a **compare-and-swap**: a commit is built on the tip the writer last read
//! and published with `git update-ref <ref> <new> <expected>`, which git applies only if the branch still holds
//! `<expected>` (the all-zero id when the branch does not exist yet). Each commit has the tip it swapped from
//! as its only parent, so every update is a fast-forward and no commit is lost.
//!
//! When the swap is rejected, another writer moved the tip. The writer re-reads the tip, checks its files
//! against it again, rebuilds the commit on it and retries, up to a bounded number of attempts
//! ([`DEFAULT_MAX_ATTEMPTS`]). After that it returns [`RecordsError::Contended`] rather than loop.
//!
//! The branch is **append-only**. Files are opaque bytes keyed by path; this module does not read them.
//!
//! - A [`Change::Put`] at a new path adds the file.
//! - A `Put` at an existing path with *different* bytes is refused.
//! - A `Put` at an existing path with the *same* bytes is a no-op: no commit is made and the current tip is
//!   returned. This keeps a replayed write idempotent, for example after a lost reply.
//! - [`Change::Delete`] and [`Change::Rename`] are always refused: they would modify or remove a record.
//!
//! A refused batch changes nothing: the whole batch is checked before any commit is made. Trees are built in
//! a temporary index, so the user's index and work tree are never touched. The importer in `athena-core`
//! (`records_import`) uses [`RecordsWriter::append`]; landing (`athena-core` `gh`) uses
//! [`RecordsWriter::append_to_remote`], which builds on the remote tip and publishes to the remote (§8.2).

use crate::index::{IndexEdit, TempIndex};
use crate::{Git, GitError, null_oid};
use std::collections::BTreeMap;
use std::time::Duration;

/// The full name of the records branch.
pub const RECORDS_REF: &str = "refs/heads/athena/records";

/// The records branch's short name, as it is spelled on a remote.
pub const RECORDS_BRANCH: &str = "athena/records";

/// How many times a write is tried against a moving tip before it gives up.
pub const DEFAULT_MAX_ATTEMPTS: usize = 8;

/// One change to the records branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change {
    /// Add `bytes` at `path`. Same bytes at an existing path is a no-op; other bytes there are refused.
    Put { path: String, bytes: Vec<u8> },
    /// Always refused: records are never deleted.
    Delete { path: String },
    /// Always refused: records are never renamed.
    Rename { from: String, to: String },
}

impl Change {
    /// A [`Change::Put`].
    pub fn put(path: impl Into<String>, bytes: impl Into<Vec<u8>>) -> Self {
        Change::Put {
            path: path.into(),
            bytes: bytes.into(),
        }
    }
}

/// Why a write to the records branch did not happen.
#[derive(Debug, thiserror::Error)]
pub enum RecordsError {
    #[error(transparent)]
    Git(#[from] GitError),
    /// The batch breaks the append-only rule or names a bad path. Nothing was written.
    #[error("records branch refused {path:?}: {reason}")]
    Refused { path: String, reason: String },
    /// Every attempt lost the compare-and-swap, or could not take the branch's lock.
    #[error(
        "records branch update gave up after {attempts} attempts: the tip kept moving (last failure: {last})"
    )]
    Contended { attempts: usize, last: String },
}

/// Appends files to the records branch of one repository.
#[derive(Debug, Clone)]
pub struct RecordsWriter<'a> {
    git: &'a Git,
    max_attempts: usize,
}

impl<'a> RecordsWriter<'a> {
    /// A writer with [`DEFAULT_MAX_ATTEMPTS`].
    pub fn new(git: &'a Git) -> Self {
        RecordsWriter {
            git,
            max_attempts: DEFAULT_MAX_ATTEMPTS,
        }
    }

    /// Sets the attempt bound (at least one attempt is always made).
    pub fn with_max_attempts(mut self, max_attempts: usize) -> Self {
        self.max_attempts = max_attempts.max(1);
        self
    }

    /// The current tip of the records branch, or `None` before the first write.
    pub fn tip(&self) -> Result<Option<String>, RecordsError> {
        Ok(self.git.rev_parse(RECORDS_REF)?)
    }

    /// Applies `changes` as one commit with `message` and returns the new tip. When every change is a
    /// same-bytes `Put` at an existing path, nothing is committed and the current tip is returned.
    ///
    /// The first write creates the branch as an orphan: its commit has no parent.
    pub fn append(&self, message: &str, changes: &[Change]) -> Result<String, RecordsError> {
        self.append_with_hook(message, changes, |_| {})
    }

    /// As [`append`](Self::append), calling `before_swap(attempt)` (counting from 1) after the tip is read
    /// and before the swap. Tests use it to move the tip at that moment.
    fn append_with_hook(
        &self,
        message: &str,
        changes: &[Change],
        mut before_swap: impl FnMut(usize),
    ) -> Result<String, RecordsError> {
        let puts = self.validate(changes)?;
        let sign = self.git.signs_commits()?;
        let mut last = String::new();
        for attempt in 1..=self.max_attempts {
            let tip = self.tip()?;
            let new_files = self.plan(tip.as_deref(), &puts)?;
            if new_files.is_empty() {
                // Only reachable with a tip: a first write with nothing to add was refused in `validate`.
                return tip.ok_or_else(|| refused("", "nothing to write"));
            }
            before_swap(attempt);

            let commit = self.build_commit(tip.as_deref(), new_files, message, sign, &[])?;

            let expected = tip.as_deref().unwrap_or_else(|| null_oid());
            let swap = self
                .git
                .exit(["update-ref", RECORDS_REF, &commit, expected], None, &[])?;
            if swap.code == Some(0) {
                return Ok(commit);
            }
            last = swap.stderr;
            if attempt < self.max_attempts {
                std::thread::sleep(Duration::from_millis(5 * attempt as u64));
            }
        }
        Err(RecordsError::Contended {
            attempts: self.max_attempts,
            last,
        })
    }

    /// Builds (but does not publish) a commit holding `base`'s tree plus `new_files`. Its parents are `base`
    /// (when there is one) and then `extra_parents`, which only carry reachability (§6.1).
    fn build_commit(
        &self,
        base: Option<&str>,
        new_files: Vec<(String, String)>,
        message: &str,
        sign: bool,
        extra_parents: &[&str],
    ) -> Result<String, RecordsError> {
        let base_tree = match base {
            Some(tip) => tip.to_string(),
            None => self.git.empty_tree()?,
        };
        let index = TempIndex::from_tree(self.git, &base_tree)?;
        let edits: Vec<IndexEdit> = new_files
            .into_iter()
            .map(|(path, oid)| IndexEdit::Put {
                mode: "100644".into(),
                oid,
                path,
            })
            .collect();
        index.apply(&edits)?;
        let tree = index.write_tree()?;
        let mut parents: Vec<&str> = base.into_iter().collect();
        parents.extend(extra_parents.iter().copied());
        Ok(self.git.commit_tree(&tree, &parents, message, sign)?)
    }

    /// Resets the local records branch to what `remote` holds (§8.2, the recover path; D111). A local
    /// branch that is ahead of the remote, or that has diverged from it, loses the commits the remote
    /// does not have: the remote is the source of truth, so such a commit is a leftover of a write that
    /// never reached it. Returns the remote tip, which is now also the local tip. When the remote has no
    /// records branch the local branch is left alone and the local tip (possibly `None`) is returned.
    pub fn sync_from(&self, remote: &str) -> Result<Option<String>, RecordsError> {
        match self.git.fetch_remote_branch(remote, RECORDS_BRANCH)? {
            Some(tip) => {
                if self.tip()?.as_deref() != Some(tip.as_str()) {
                    self.git.set_ref(RECORDS_REF, &tip)?;
                }
                Ok(Some(tip))
            }
            None => self.tip(),
        }
    }

    /// Appends `changes` as one commit on the **remote** tip and publishes it to `remote` (§8.2; D111).
    ///
    /// The commit is built on the remote's records tip and pushed with an expected-tip guard, so it lands
    /// only if the remote branch still holds that tip. The local branch is reset to the remote first
    /// ([`sync_from`](Self::sync_from)) and moves to the new commit only after the push succeeded. When the
    /// push is rejected because another writer moved the remote, the tip is fetched again, the commit is
    /// rebuilt on it and the push retried, up to the attempt bound; then [`RecordsError::Contended`] is
    /// returned. A push that fails while the remote tip has not moved is a plain error, not a retry.
    ///
    /// `extra_parents` become the second and later parents of the commit (§6.1: a landing record carries the
    /// pull request's head so its commits stay reachable). When every change is a same-bytes `Put` at an
    /// existing path, nothing is pushed and the remote tip is returned.
    pub fn append_to_remote(
        &self,
        remote: &str,
        message: &str,
        changes: &[Change],
        extra_parents: &[&str],
    ) -> Result<String, RecordsError> {
        self.append_to_remote_with_hook(remote, message, changes, extra_parents, |_| {})
    }

    /// As [`append_to_remote`](Self::append_to_remote), calling `before_push(attempt)` (from 1) after the
    /// commit is built and before it is pushed. Tests use it to move the remote at that moment.
    fn append_to_remote_with_hook(
        &self,
        remote: &str,
        message: &str,
        changes: &[Change],
        extra_parents: &[&str],
        mut before_push: impl FnMut(usize),
    ) -> Result<String, RecordsError> {
        let puts = self.validate(changes)?;
        let sign = self.git.signs_commits()?;
        self.sync_from(remote)?;
        let mut last = String::new();
        for attempt in 1..=self.max_attempts {
            let remote_tip = self.git.fetch_remote_branch(remote, RECORDS_BRANCH)?;
            // With no remote branch yet, a local one (a first write that never got out) is the base.
            let base = match &remote_tip {
                Some(tip) => Some(tip.clone()),
                None => self.tip()?,
            };
            let new_files = self.plan(base.as_deref(), &puts)?;
            if new_files.is_empty() {
                return base.ok_or_else(|| refused("", "nothing to write"));
            }
            let commit =
                self.build_commit(base.as_deref(), new_files, message, sign, extra_parents)?;
            before_push(attempt);
            let pushed = match &remote_tip {
                Some(expected) => {
                    self.git
                        .push_if_remote_at(remote, RECORDS_BRANCH, &commit, expected)
                }
                None => self.git.push_branch(remote, RECORDS_BRANCH, &commit),
            };
            match pushed {
                Ok(()) => {
                    self.git.set_ref(RECORDS_REF, &commit)?;
                    return Ok(commit);
                }
                Err(error) => {
                    let now = self.git.fetch_remote_branch(remote, RECORDS_BRANCH)?;
                    if now == remote_tip {
                        return Err(error.into());
                    }
                    last = error.to_string();
                }
            }
            if attempt < self.max_attempts {
                std::thread::sleep(Duration::from_millis(5 * attempt as u64));
            }
        }
        Err(RecordsError::Contended {
            attempts: self.max_attempts,
            last,
        })
    }

    /// Checks paths and refuses deletes and renames. Returns the puts with their blob ids, de-duplicated.
    fn validate(&self, changes: &[Change]) -> Result<Vec<(String, String)>, RecordsError> {
        let mut puts: Vec<(String, String)> = Vec::new();
        for change in changes {
            match change {
                Change::Put { path, bytes } => {
                    check_path(path)?;
                    let oid = self.git.hash_object_w(bytes)?;
                    match puts.iter().find(|(p, _)| p == path) {
                        Some((_, other)) if *other != oid => {
                            return Err(refused(path, "the batch writes this path twice"));
                        }
                        Some(_) => {}
                        None => puts.push((path.clone(), oid)),
                    }
                }
                Change::Delete { path } => {
                    return Err(refused(
                        path,
                        "records are append-only; a path is never deleted",
                    ));
                }
                Change::Rename { from, .. } => {
                    return Err(refused(
                        from,
                        "records are append-only; a path is never renamed",
                    ));
                }
            }
        }
        if puts.is_empty() {
            return Err(refused("", "nothing to write"));
        }
        Ok(puts)
    }

    /// The puts that still need writing on `tip`. Refuses a put that would change an existing file or clash
    /// with a directory. A same-bytes put at an existing path is dropped.
    fn plan(
        &self,
        tip: Option<&str>,
        puts: &[(String, String)],
    ) -> Result<Vec<(String, String)>, RecordsError> {
        let existing: BTreeMap<String, String> = match tip {
            Some(tip) => self
                .git
                .ls_tree(tip)?
                .into_iter()
                .map(|e| (e.path, e.oid))
                .collect(),
            None => BTreeMap::new(),
        };
        let clashes = |a: &str, b: &str| {
            a.strip_prefix(b).is_some_and(|rest| rest.starts_with('/'))
                || b.strip_prefix(a).is_some_and(|rest| rest.starts_with('/'))
        };
        let mut out: Vec<(String, String)> = Vec::new();
        for (path, oid) in puts {
            if let Some(have) = existing.get(path) {
                if have == oid {
                    continue;
                }
                return Err(refused(
                    path,
                    "records are append-only; this path exists with different bytes",
                ));
            }
            let clash = existing
                .keys()
                .map(String::as_str)
                .chain(puts.iter().map(|(p, _)| p.as_str()))
                .any(|other| other != path && clashes(path, other));
            if clash {
                return Err(refused(
                    path,
                    "the path is a file where another record needs a directory, or the reverse",
                ));
            }
            out.push((path.clone(), oid.clone()));
        }
        Ok(out)
    }
}

fn refused(path: &str, reason: &str) -> RecordsError {
    RecordsError::Refused {
        path: path.to_string(),
        reason: reason.to_string(),
    }
}

/// A record path is relative, uses `/` between non-empty names, and has no `.`, `..`, `.git` or control
/// characters.
fn check_path(path: &str) -> Result<(), RecordsError> {
    if path.is_empty() {
        return Err(refused(path, "the path is empty"));
    }
    if path.chars().any(|c| c.is_control()) {
        return Err(refused(path, "the path has a control character"));
    }
    for name in path.split('/') {
        if name.is_empty() || name == "." || name == ".." || name.eq_ignore_ascii_case(".git") {
            return Err(refused(
                path,
                "the path must be relative, with plain names between single slashes",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::repo_with;
    use std::cell::Cell;

    fn files(git: &Git, tip: &str) -> Vec<(String, Vec<u8>)> {
        git.ls_tree(tip)
            .unwrap()
            .into_iter()
            .map(|e| {
                let bytes = git.cat_blob(&e.oid).unwrap();
                (e.path, bytes)
            })
            .collect()
    }

    fn paths(git: &Git, tip: &str) -> Vec<String> {
        files(git, tip).into_iter().map(|(p, _)| p).collect()
    }

    #[test]
    fn two_writers_on_threads_both_land() {
        let (_dir, git) = repo_with(&[("a.txt", "a")]);
        let tips: Vec<String> = std::thread::scope(|scope| {
            let handles: Vec<_> = ["jobs/one.json", "jobs/two.json"]
                .into_iter()
                .map(|path| {
                    let git = &git;
                    scope.spawn(move || {
                        RecordsWriter::new(git)
                            .append("write", &[Change::put(path, path.as_bytes())])
                            .unwrap()
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });

        let tip = git.rev_parse(RECORDS_REF).unwrap().unwrap();
        assert_eq!(paths(&git, &tip), vec!["jobs/one.json", "jobs/two.json"]);
        let chain = git.rev_list_reverse(&tip).unwrap();
        assert_eq!(chain.len(), 2, "no commit lost");
        assert!(git.parents(&chain[0]).unwrap().is_empty());
        assert_eq!(git.parents(&chain[1]).unwrap(), vec![chain[0].clone()]);
        assert!(git.is_ancestor(&chain[0], &chain[1]).unwrap());
        let mut returned = tips;
        returned.sort();
        let mut expected = chain.clone();
        expected.sort();
        assert_eq!(returned, expected, "each call returned its own commit");
    }

    #[test]
    fn stale_tip_is_rejected_then_replayed() {
        let (_dir, git) = repo_with(&[("a.txt", "a")]);
        let writer = RecordsWriter::new(&git);
        let first = writer
            .append("first", &[Change::put("jobs/early.json", "early")])
            .unwrap();

        let attempts = Cell::new(0);
        let competing = Cell::new(None);
        let tip = writer
            .append_with_hook("mine", &[Change::put("jobs/mine.json", "mine")], |n| {
                attempts.set(n);
                if n == 1 {
                    // Another writer lands after this writer read `first`, so its swap is stale.
                    competing.set(Some(
                        writer
                            .append("other", &[Change::put("jobs/other.json", "other")])
                            .unwrap(),
                    ));
                }
            })
            .unwrap();

        let competing = competing.take().unwrap();
        assert_eq!(attempts.get(), 2, "first swap rejected, second succeeded");
        assert_ne!(competing, first);
        assert_eq!(git.parents(&tip).unwrap(), vec![competing.clone()]);
        assert_eq!(git.rev_parse(RECORDS_REF).unwrap().unwrap(), tip);
        assert_eq!(
            paths(&git, &tip),
            vec!["jobs/early.json", "jobs/mine.json", "jobs/other.json"]
        );
        assert!(git.is_ancestor(&first, &competing).unwrap());
        assert!(git.is_ancestor(&competing, &tip).unwrap());
        assert_eq!(git.rev_list_reverse(&tip).unwrap().len(), 3);
    }

    #[test]
    fn a_replayed_file_that_another_writer_already_landed_is_not_duplicated() {
        let (_dir, git) = repo_with(&[("a.txt", "a")]);
        let writer = RecordsWriter::new(&git);
        writer
            .append("seed", &[Change::put("s.json", "s")])
            .unwrap();
        let landed = Cell::new(None);
        let tip = writer
            .append_with_hook("mine", &[Change::put("x.json", "x")], |n| {
                if n == 1 {
                    landed.set(Some(
                        writer
                            .append("same", &[Change::put("x.json", "x")])
                            .unwrap(),
                    ));
                }
            })
            .unwrap();
        assert_eq!(
            Some(tip),
            landed.take(),
            "the replay found nothing left to write"
        );
    }

    #[test]
    fn existing_paths_cannot_be_modified_deleted_or_renamed() {
        let (_dir, git) = repo_with(&[("a.txt", "a")]);
        let writer = RecordsWriter::new(&git);
        let tip = writer
            .append("first", &[Change::put("jobs/a.json", "one")])
            .unwrap();

        let refusals = [
            Change::put("jobs/a.json", "two"),
            Change::Delete {
                path: "jobs/a.json".into(),
            },
            Change::Rename {
                from: "jobs/a.json".into(),
                to: "jobs/b.json".into(),
            },
        ];
        for change in refusals {
            // Alongside a valid new file: the whole batch is refused.
            let err = writer
                .append("bad", &[Change::put("jobs/new.json", "n"), change.clone()])
                .unwrap_err();
            assert!(
                matches!(err, RecordsError::Refused { .. }),
                "{change:?}: {err}"
            );
            assert_eq!(
                writer.tip().unwrap().as_deref(),
                Some(tip.as_str()),
                "{change:?}"
            );
        }
        assert_eq!(
            files(&git, &tip),
            vec![("jobs/a.json".to_string(), b"one".to_vec())]
        );
        // A file cannot become a directory or the reverse.
        for path in ["jobs/a.json/deeper.json", "jobs"] {
            let err = writer
                .append("clash", &[Change::put(path, "x")])
                .unwrap_err();
            assert!(matches!(err, RecordsError::Refused { .. }), "{path}: {err}");
        }
        assert_eq!(writer.tip().unwrap().as_deref(), Some(tip.as_str()));
    }

    #[test]
    fn writing_the_same_bytes_again_is_a_no_op_that_returns_the_tip() {
        let (_dir, git) = repo_with(&[("a.txt", "a")]);
        let writer = RecordsWriter::new(&git);
        let tip = writer
            .append("first", &[Change::put("jobs/a.json", "one")])
            .unwrap();
        let again = writer
            .append("again", &[Change::put("jobs/a.json", "one")])
            .unwrap();
        assert_eq!(again, tip);
        assert_eq!(writer.tip().unwrap().as_deref(), Some(tip.as_str()));
        assert_eq!(git.rev_list_reverse(&tip).unwrap().len(), 1);
        // Mixed with a new file, only the new file is written.
        let next = writer
            .append(
                "mixed",
                &[
                    Change::put("jobs/a.json", "one"),
                    Change::put("jobs/b.json", "b"),
                ],
            )
            .unwrap();
        assert_eq!(git.parents(&next).unwrap(), vec![tip]);
        assert_eq!(paths(&git, &next), vec!["jobs/a.json", "jobs/b.json"]);
    }

    #[test]
    fn the_first_commit_is_an_orphan_unrelated_to_main() {
        let (_dir, git) = repo_with(&[("a.txt", "a")]);
        assert_eq!(RecordsWriter::new(&git).tip().unwrap(), None);
        let first = RecordsWriter::new(&git)
            .append("first", &[Change::put("jobs/a.json", "one")])
            .unwrap();
        assert!(git.parents(&first).unwrap().is_empty());
        let merge_base = git
            .exit(["merge-base", "main", "athena/records"], None, &[])
            .unwrap();
        assert_eq!(merge_base.code, Some(1), "{}", merge_base.stderr);
        assert!(merge_base.stdout.is_empty());
        // The records tree holds only records; main's files are not in it.
        assert_eq!(paths(&git, &first), vec!["jobs/a.json"]);
        // main and the work tree are untouched.
        assert_eq!(git.current_branch().unwrap().as_deref(), Some("main"));
        assert_eq!(git.run(["status", "--porcelain"]).unwrap(), "");
    }

    #[test]
    fn when_every_attempt_is_rejected_the_writer_stops_with_a_clear_error() {
        let (_dir, git) = repo_with(&[("a.txt", "a")]);
        let writer = RecordsWriter::new(&git).with_max_attempts(3);
        writer
            .append("seed", &[Change::put("seed.json", "s")])
            .unwrap();
        let calls = Cell::new(0);
        let err = writer
            .append_with_hook("mine", &[Change::put("mine.json", "m")], |n| {
                calls.set(n);
                // Another writer wins the race on every attempt.
                RecordsWriter::new(&git)
                    .append("rival", &[Change::put(format!("rival/{n}.json"), "r")])
                    .unwrap();
            })
            .unwrap_err();
        assert!(
            matches!(err, RecordsError::Contended { attempts: 3, .. }),
            "{err}"
        );
        assert!(err.to_string().contains("3 attempts"), "{err}");
        assert_eq!(calls.get(), 3, "bounded: exactly three attempts");
        let tip = writer.tip().unwrap().unwrap();
        assert_eq!(
            paths(&git, &tip),
            vec!["rival/1.json", "rival/2.json", "rival/3.json", "seed.json"]
        );
    }

    /// A bare remote, and a repository (with one commit on main) whose `origin` is that remote.
    fn bare_remote() -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let bare = Git::isolated(dir.path());
        bare.run(["init", "-q", "--bare", "-b", "main"]).unwrap();
        (dir, bare)
    }

    fn clone_of(bare: &Git) -> (tempfile::TempDir, Git) {
        let (dir, git) = repo_with(&[("a.txt", "a")]);
        let url = bare.dir().to_str().unwrap().to_string();
        git.run(["remote", "add", "origin", &url]).unwrap();
        (dir, git)
    }

    fn remote_tip(bare: &Git) -> Option<String> {
        bare.rev_parse(RECORDS_REF).unwrap()
    }

    #[test]
    fn a_remote_append_publishes_first_then_moves_the_local_branch_and_keeps_extra_parents() {
        let (_b, bare) = bare_remote();
        let (_d, git) = clone_of(&bare);
        let writer = RecordsWriter::new(&git);
        let first = writer
            .append_to_remote("origin", "first", &[Change::put("one.json", "1")], &[])
            .unwrap();
        assert_eq!(remote_tip(&bare).as_deref(), Some(first.as_str()));
        assert_eq!(writer.tip().unwrap().as_deref(), Some(first.as_str()));
        assert!(git.parents(&first).unwrap().is_empty());

        let work = git.head().unwrap().unwrap();
        let second = writer
            .append_to_remote(
                "origin",
                "second",
                &[Change::put("two.json", "2")],
                &[&work],
            )
            .unwrap();
        assert_eq!(git.parents(&second).unwrap(), vec![first.clone(), work]);
        assert_eq!(remote_tip(&bare).as_deref(), Some(second.as_str()));
        assert_eq!(writer.tip().unwrap().as_deref(), Some(second.as_str()));
        // First-parent order is unchanged by the extra parent.
        assert_eq!(paths(&bare, &second), vec!["one.json", "two.json"]);
        // The same bytes again is a no-op that pushes nothing.
        let again = writer
            .append_to_remote("origin", "again", &[Change::put("two.json", "2")], &[])
            .unwrap();
        assert_eq!(again, second);
    }

    #[test]
    fn a_rejected_push_is_rebuilt_on_the_fetched_remote_tip_and_retried() {
        let (_b, bare) = bare_remote();
        let (_d, git) = clone_of(&bare);
        let (_d2, rival) = clone_of(&bare);
        let writer = RecordsWriter::new(&git);
        let seed = writer
            .append_to_remote("origin", "seed", &[Change::put("seed.json", "s")], &[])
            .unwrap();
        let attempts = Cell::new(0);
        let rival_tip = Cell::new(None);
        let local_during = Cell::new(None);
        let tip = writer
            .append_to_remote_with_hook(
                "origin",
                "mine",
                &[Change::put("mine.json", "m")],
                &[],
                |n| {
                    attempts.set(n);
                    if n == 1 {
                        // Another clone moves the remote after this commit was built and before it is pushed.
                        rival_tip.set(Some(
                            RecordsWriter::new(&rival)
                                .append_to_remote(
                                    "origin",
                                    "rival",
                                    &[Change::put("rival.json", "r")],
                                    &[],
                                )
                                .unwrap(),
                        ));
                        local_during.set(writer.tip().unwrap());
                    }
                },
            )
            .unwrap();
        let rival_tip = rival_tip.take().unwrap();
        assert_eq!(attempts.get(), 2, "first push rejected, second accepted");
        assert_eq!(
            local_during.take().as_deref(),
            Some(seed.as_str()),
            "the local branch did not move while the push was pending or rejected"
        );
        assert_eq!(git.parents(&tip).unwrap(), vec![rival_tip]);
        assert_eq!(remote_tip(&bare).as_deref(), Some(tip.as_str()));
        assert_eq!(writer.tip().unwrap().as_deref(), Some(tip.as_str()));
        assert_eq!(
            paths(&bare, &tip),
            vec!["mine.json", "rival.json", "seed.json"],
            "the other writer's record is kept"
        );
    }

    #[test]
    fn a_remote_that_moves_on_every_attempt_ends_in_a_bounded_error_and_the_local_branch_stays() {
        let (_b, bare) = bare_remote();
        let (_d, git) = clone_of(&bare);
        let (_d2, rival) = clone_of(&bare);
        let writer = RecordsWriter::new(&git).with_max_attempts(3);
        let seed = writer
            .append_to_remote("origin", "seed", &[Change::put("seed.json", "s")], &[])
            .unwrap();
        let calls = Cell::new(0);
        let err = writer
            .append_to_remote_with_hook(
                "origin",
                "mine",
                &[Change::put("mine.json", "m")],
                &[],
                |n| {
                    calls.set(n);
                    RecordsWriter::new(&rival)
                        .append_to_remote(
                            "origin",
                            "rival",
                            &[Change::put(format!("rival/{n}.json"), "r")],
                            &[],
                        )
                        .unwrap();
                },
            )
            .unwrap_err();
        assert!(
            matches!(err, RecordsError::Contended { attempts: 3, .. }),
            "{err}"
        );
        assert_eq!(calls.get(), 3, "bounded: exactly three attempts");
        assert_eq!(writer.tip().unwrap().as_deref(), Some(seed.as_str()));
        let tip = remote_tip(&bare).unwrap();
        assert!(!paths(&bare, &tip).contains(&"mine.json".to_string()));
    }

    #[test]
    fn sync_from_resets_a_local_branch_that_is_ahead_or_diverged_and_the_next_write_succeeds() {
        let (_b, bare) = bare_remote();
        let (_d, git) = clone_of(&bare);
        let writer = RecordsWriter::new(&git);
        let base = writer
            .append_to_remote("origin", "base", &[Change::put("base.json", "b")], &[])
            .unwrap();

        // Ahead: a local-only commit on top of the remote tip.
        let leftover = writer
            .append("leftover", &[Change::put("leftover.json", "l")])
            .unwrap();
        assert_eq!(git.parents(&leftover).unwrap(), vec![base.clone()]);
        assert_eq!(
            writer.sync_from("origin").unwrap().as_deref(),
            Some(base.as_str())
        );
        assert_eq!(writer.tip().unwrap().as_deref(), Some(base.as_str()));

        // Diverged: the remote moves on through another clone while this one holds its own local commit.
        let local_only = writer
            .append("local only", &[Change::put("local-only.json", "x")])
            .unwrap();
        let (_d2, rival) = clone_of(&bare);
        let moved = RecordsWriter::new(&rival)
            .append_to_remote("origin", "rival", &[Change::put("rival.json", "r")], &[])
            .unwrap();
        assert_ne!(local_only, moved);
        assert_eq!(bare.rev_parse(&local_only).unwrap(), None);
        let tip = writer
            .append_to_remote("origin", "mine", &[Change::put("mine.json", "m")], &[])
            .unwrap();
        assert_eq!(git.parents(&tip).unwrap(), vec![moved]);
        assert_eq!(remote_tip(&bare).as_deref(), Some(tip.as_str()));
        assert_eq!(writer.tip().unwrap().as_deref(), Some(tip.as_str()));
        assert_eq!(
            paths(&bare, &tip),
            vec!["base.json", "mine.json", "rival.json"],
            "neither leftover commit reached the remote"
        );
    }

    #[test]
    fn bad_paths_and_empty_batches_are_refused_before_anything_is_written() {
        let (_dir, git) = repo_with(&[("a.txt", "a")]);
        let writer = RecordsWriter::new(&git);
        for path in [
            "",
            "/abs.json",
            "a//b.json",
            "../up.json",
            "a/./b",
            "dir/",
            ".git/x",
            "a\nb",
        ] {
            let err = writer.append("bad", &[Change::put(path, "x")]).unwrap_err();
            assert!(
                matches!(err, RecordsError::Refused { .. }),
                "{path:?}: {err}"
            );
        }
        let err = writer.append("empty", &[]).unwrap_err();
        assert!(matches!(err, RecordsError::Refused { .. }), "{err}");
        let err = writer
            .append(
                "twice",
                &[Change::put("x.json", "1"), Change::put("x.json", "2")],
            )
            .unwrap_err();
        assert!(matches!(err, RecordsError::Refused { .. }), "{err}");
        assert_eq!(writer.tip().unwrap(), None, "no branch was created");
    }
}
