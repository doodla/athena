//! Commits Athena builds with plumbing (design §2.3, §5.5, §7.3).
//!
//! Athena writes to the user's repository only here. It writes blobs and trees through a temporary index
//! (the user's index and work tree are untouched), creates the commit with `commit-tree` (signed when
//! `commit.gpgsign` is set), runs the gate on it in process (plumbing commits run no hooks), and advances
//! the current branch with `git merge --ff-only`, which refuses rather than overwrite local edits. All of
//! it happens under the repository lock.

use crate::Athena;
use crate::state::StateDir;
use anyhow::{Result, bail};
use athena_data::records::AnyRecord;
use athena_gate::{Change, Scope, gate};
use athena_git::Git;
use athena_git::index::{IndexEdit, TempIndex};

/// A file to add in a commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

impl NewFile {
    /// A record at its canonical path.
    pub fn record(record: &AnyRecord) -> NewFile {
        NewFile {
            path: record.canonical_path(),
            bytes: record.to_json().into_bytes(),
        }
    }
    pub fn text(path: impl Into<String>, text: impl Into<String>) -> NewFile {
        NewFile {
            path: path.into(),
            bytes: text.into().into_bytes(),
        }
    }
}

/// A commit message: subject, body, and trailers in git's trailer convention.
pub fn message(subject: &str, body: &str, trailers: &[(&str, String)]) -> String {
    let mut text = subject.trim().to_string();
    text.push_str("\n\n");
    if !body.trim().is_empty() {
        text.push_str(body.trim());
        text.push_str("\n\n");
    }
    for (key, value) in trailers {
        text.push_str(&format!("{key}: {value}\n"));
    }
    text
}

/// Why a fast-forward did not happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The branch moved while the commit was built; building again on the new tip may succeed.
    Moved,
    /// Local edits in the checkout conflict with the commit.
    LocalEdits(String),
}

/// The part of [`Athena`] that writes records: the repository and its state directory, nothing else. It is
/// what a background role holds, because the agent driver in [`Athena`] is not shared between threads.
#[derive(Debug, Clone, Copy)]
pub struct Committer<'a> {
    pub git: &'a Git,
    pub state: &'a StateDir,
}

impl Athena {
    fn committer(&self) -> Committer<'_> {
        Committer {
            git: &self.git,
            state: &self.state,
        }
    }

    /// Builds a commit of `tree` plus `files` on parent `parent`, and gates it. The commit is not yet on
    /// any branch.
    pub fn build_commit(
        &self,
        tree: &str,
        parent: &str,
        files: &[NewFile],
        message: &str,
    ) -> Result<String> {
        self.committer().build_commit(tree, parent, files, message)
    }

    /// Fast-forwards the current branch to `commit`, which must descend from `expected_head`.
    pub fn advance(
        &self,
        commit: &str,
        expected_head: &str,
    ) -> Result<std::result::Result<(), Refusal>> {
        self.committer().advance(commit, expected_head)
    }

    /// Commits `files` (records only) on top of the current branch, gated, and fast-forwards to it.
    /// Returns the new commit.
    pub fn commit_records(&self, files: &[NewFile], message: &str) -> Result<String> {
        self.committer().commit_records(files, message)
    }

    /// Like [`Athena::commit_records`], for records that depend on the commit they are built on, such as a
    /// score that reads the interventions there. `files` is called again whenever the branch moved.
    pub fn commit_records_with(
        &self,
        files: &dyn Fn(&str) -> Result<Vec<NewFile>>,
        message: &str,
    ) -> Result<String> {
        self.committer().commit_records_with(files, message)
    }

    /// [`Athena::commit_records_with`], gated under `scope`.
    pub fn commit_records_in(
        &self,
        files: &dyn Fn(&str) -> Result<Vec<NewFile>>,
        message: &str,
        scope: Scope,
    ) -> Result<String> {
        self.committer().commit_records_in(files, message, scope)
    }
}

impl Committer<'_> {
    fn head(&self) -> Result<String> {
        match self.git.head()? {
            Some(head) => Ok(head),
            None => bail!(
                "the repository has no commits yet; commit athena.toml and defaults.toml first"
            ),
        }
    }

    /// Builds a commit of `tree` plus `files` on parent `parent`, and gates it. The commit is not yet on
    /// any branch.
    pub fn build_commit(
        &self,
        tree: &str,
        parent: &str,
        files: &[NewFile],
        message: &str,
    ) -> Result<String> {
        self.build_commit_in(tree, parent, files, message, Scope::Commit)
    }

    /// [`Committer::build_commit`], gated under `scope`.
    fn build_commit_in(
        &self,
        tree: &str,
        parent: &str,
        files: &[NewFile],
        message: &str,
        scope: Scope,
    ) -> Result<String> {
        let index = TempIndex::from_tree(self.git, tree)?;
        let mut edits = Vec::new();
        for file in files {
            let oid = self.git.hash_object_w(&file.bytes)?;
            edits.push(IndexEdit::Put {
                mode: "100644".into(),
                oid,
                path: file.path.clone(),
            });
        }
        index.apply(&edits)?;
        let new_tree = index.write_tree()?;
        let sign = self.git.signs_commits()?;
        let commit = self.git.commit_tree(&new_tree, &[parent], message, sign)?;
        let violations = gate(self.git, &Change::of_commit(self.git, &commit)?, scope)?;
        if !violations.is_empty() {
            let list = violations
                .iter()
                .map(|v| format!("  - {v}"))
                .collect::<Vec<_>>()
                .join("\n");
            bail!(
                "the gate refused Athena's own commit {}:\n{list}",
                &commit[..12]
            );
        }
        Ok(commit)
    }

    /// Fast-forwards the current branch to `commit`, which must descend from `expected_head`.
    pub fn advance(
        &self,
        commit: &str,
        expected_head: &str,
    ) -> Result<std::result::Result<(), Refusal>> {
        if self.head()? != expected_head {
            return Ok(Err(Refusal::Moved));
        }
        match self.git.merge_ff_only(commit)? {
            Ok(()) => Ok(Ok(())),
            Err(stderr) => {
                if self.head()? != expected_head {
                    Ok(Err(Refusal::Moved))
                } else {
                    Ok(Err(Refusal::LocalEdits(stderr)))
                }
            }
        }
    }

    /// Commits `files` (records only) on top of the current branch, gated, and fast-forwards to it.
    /// Returns the new commit.
    pub fn commit_records(&self, files: &[NewFile], message: &str) -> Result<String> {
        self.commit_records_with(&|_head: &str| Ok(files.to_vec()), message)
    }

    /// Like [`Athena::commit_records`], for records that depend on the commit they are built on, such as a
    /// score that reads the interventions there. `files` is called again whenever the branch moved.
    pub fn commit_records_with(
        &self,
        files: &dyn Fn(&str) -> Result<Vec<NewFile>>,
        message: &str,
    ) -> Result<String> {
        self.commit_records_in(files, message, Scope::Commit)
    }

    /// [`Committer::commit_records_with`], gated under `scope`.
    pub fn commit_records_in(
        &self,
        files: &dyn Fn(&str) -> Result<Vec<NewFile>>,
        message: &str,
        scope: Scope,
    ) -> Result<String> {
        let _lock = self.state.lock_repository()?;
        for _attempt in 0..3 {
            let head = self.head()?;
            let files = files(&head)?;
            let commit = self.build_commit_in(&head, &head, &files, message, scope)?;
            match self.advance(&commit, &head)? {
                Ok(()) => return Ok(commit),
                Err(Refusal::Moved) => continue,
                Err(Refusal::LocalEdits(stderr)) => bail!(
                    "could not add records to the branch: {stderr}. Records live only under records/; \
                     move any local edits there aside and run the command again"
                ),
            }
        }
        bail!("the branch kept moving while Athena committed its records; try again")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_follow_the_trailer_convention() {
        let text = message(
            "Add a changelog",
            "Body text.",
            &[("Athena-Job", "x".into()), ("Athena-Class", "docs".into())],
        );
        assert_eq!(
            text,
            "Add a changelog\n\nBody text.\n\nAthena-Job: x\nAthena-Class: docs\n"
        );
    }
}
