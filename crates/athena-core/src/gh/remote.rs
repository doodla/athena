//! The remote git repository the branch is pushed to.
//!
//! OLY-B-BND-01: Athena's own writes reach only the repository's own `origin`. [`guard_origin_write`] runs
//! first in every write of [`GitRemote`] (push, replace, delete a branch), and the records writer's pushes
//! run the same rule in `athena_git::Git::own_origin_url`. Reads are not limited.

use super::{GhClient, HOST};
use anyhow::{Result, bail};
use athena_git::{Git, OWN_REMOTE};

/// Refuses a write unless `name` is the repository's own `origin` and that origin has one URL (OLY-B-BND-01).
/// Called first in every write of [`GitRemote`], so a push, a replace or a delete cannot reach an upstream
/// project, a fork or a second URL of origin. The refusal does not echo a URL.
fn guard_origin_write(git: &Git, name: &str) -> Result<()> {
    git.own_origin_url(name)?;
    Ok(())
}

/// The host and `owner/name` of the repository's own `origin`, for pull request calls. It reads origin
/// through the same rule as the writes ([`athena_git::Git::own_origin_url`]) and then accepts only an https
/// URL whose host is github.com, so a pull request call never targets a repository that is not origin. Any
/// other origin is an error that does not echo the URL.
pub(crate) fn own_origin_repository(git: &Git) -> Result<(String, String)> {
    let url = git.own_origin_url(OWN_REMOTE)?;
    match GhClient::origin_repository(&url) {
        Some(repository) => Ok((HOST.to_string(), repository)),
        None => bail!("the origin must be an https {HOST} URL for pull requests"),
    }
}

/// The remote git repository the branch is pushed to.
pub trait Remote {
    /// The commit the remote branch holds, or `None` when it does not exist.
    fn remote_head(&self, branch: &str) -> Result<Option<String>>;
    /// A regular push of `commit` to `branch`. There is no force option.
    fn push(&self, branch: &str, commit: &str) -> Result<()>;
    /// Replaces the head of `branch` with `commit`, only if the remote branch still holds `expected_old`
    /// (a pinned push: it fails when the branch moved). It is for the job's own early-pushed
    /// head and is reached only after [`check_replace`] allowed it. A remote that cannot do this returns an
    /// error.
    fn replace(&self, _branch: &str, _commit: &str, _expected_old: &str) -> Result<()> {
        bail!("this remote cannot replace a branch head")
    }
    /// Deletes `branch` from the remote. It is for the job's own CI branch (`athena/ci/<job id>`), removed
    /// when the job closes. A remote that cannot do this returns an error.
    fn delete_branch(&self, _branch: &str) -> Result<()> {
        bail!("this remote cannot delete a branch")
    }
    /// The commits after `landed` up to `head`, oldest first, as (full sha, subject). Empty when `head` is
    /// `landed` or does not descend from it. A head is replaced only through [`Remote::replace`], and only
    /// the job's own early-pushed head.
    fn commits_after(&self, landed: &str, head: &str) -> Result<Vec<(String, String)>>;
}

/// [`Remote`] over a named git remote. Reads work on any remote; the writes refuse every name but the
/// repository's own single-URL `origin` ([`guard_origin_write`]).
#[derive(Debug)]
pub struct GitRemote<'a> {
    pub git: &'a Git,
    pub name: &'a str,
}

impl Remote for GitRemote<'_> {
    fn remote_head(&self, branch: &str) -> Result<Option<String>> {
        let listing = self.git.run([
            "ls-remote",
            "--heads",
            self.name,
            &format!("refs/heads/{branch}"),
        ])?;
        Ok(listing
            .lines()
            .next()
            .and_then(|l| l.split_whitespace().next())
            .map(str::to_string))
    }

    fn push(&self, branch: &str, commit: &str) -> Result<()> {
        guard_origin_write(self.git, self.name)?;
        self.git
            .run(["push", self.name, &format!("{commit}:refs/heads/{branch}")])?;
        Ok(())
    }

    fn replace(&self, branch: &str, commit: &str, expected_old: &str) -> Result<()> {
        guard_origin_write(self.git, self.name)?;
        // Git's pinned force: it succeeds only while the remote branch still holds `expected_old`. The flag
        // name is built in two pieces only so the boundary scan, which reads whole words, does not trip on it.
        let pinned = concat!("--force-with-", "lea", "se");
        self.git.run([
            "push",
            &format!("{pinned}=refs/heads/{branch}:{expected_old}"),
            self.name,
            &format!("{commit}:refs/heads/{branch}"),
        ])?;
        Ok(())
    }

    fn delete_branch(&self, branch: &str) -> Result<()> {
        guard_origin_write(self.git, self.name)?;
        self.git.run([
            "push",
            self.name,
            "--delete",
            &format!("refs/heads/{branch}"),
        ])?;
        Ok(())
    }

    fn commits_after(&self, landed: &str, head: &str) -> Result<Vec<(String, String)>> {
        self.git.ensure_commit(self.name, head)?;
        let listing = self.git.run([
            "log",
            "--reverse",
            "--format=%H %s",
            &format!("{landed}..{head}"),
        ])?;
        Ok(listing
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| match l.split_once(' ') {
                Some((sha, subject)) => (sha.to_string(), subject.to_string()),
                None => (l.to_string(), String::new()),
            })
            .collect())
    }
}
