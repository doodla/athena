//! The landing decision (design §7.3; M7).
//!
//! ```text
//! H := tip of the user's current branch
//! require P is an ancestor of H                          else stale
//! require the athena-* classification of every path in P ∪ H ∪ C is the same under P's and H's rules
//! if H == P:                                     tree := tree(C)
//! elif fingerprint(H; P) == fingerprint(P; P):   tree := merge-tree --merge-base P H C
//! else:                                          stale (governed base content changed)
//! assert fingerprint(tree; P) == F               (the landed content is the verified content)
//! ```
//!
//! The caller adds the closing records to the tree, gates the commit, and fast-forwards; a refused
//! fast-forward makes the job `blocked`.

use crate::verify::Verified;
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_data::JobId;
use athena_git::Git;
use athena_git::attrs::{classification_changes, fingerprint};
use std::collections::BTreeSet;

/// What landing on the current tip would do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    /// Land this tree (the candidate's, or the candidate replayed over records-only changes).
    Tree(String),
    /// The job cannot land on this tip; it must be verified again.
    Stale(String),
}

/// Decides how `verified` lands on `head`.
pub fn plan(git: &Git, verified: &Verified, head: &str) -> Result<Plan> {
    let p = verified.base();
    let c = verified.candidate();
    if !git.is_ancestor(p, head)? {
        return Ok(Plan::Stale(
            "the job's base commit is no longer an ancestor of the branch".into(),
        ));
    }
    let mut paths: BTreeSet<String> = BTreeSet::new();
    for treeish in [p, head, c] {
        paths.extend(git.ls_tree(treeish)?.into_iter().map(|e| e.path));
    }
    let paths: Vec<String> = paths.into_iter().collect();
    let moved = classification_changes(git, p, head, &paths)?;
    if !moved.is_empty() {
        return Ok(Plan::Stale(format!(
            "exclusion rules changed since dispatch (first: {})",
            moved[0]
        )));
    }
    let tree = if head == p {
        git.tree_of(c)?
    } else if fingerprint(git, head, p)? == fingerprint(git, p, p)? {
        match git.merge_tree(p, head, c)? {
            Some(tree) => tree,
            None => {
                return Ok(Plan::Stale(
                    "replaying the candidate over the records added since dispatch conflicted"
                        .into(),
                ));
            }
        }
    } else {
        let by = match first_governed_change(git, p, head)? {
            Some((commit, subject)) => format!(" (first by {} \"{subject}\")", &commit[..12]),
            None => String::new(),
        };
        return Ok(Plan::Stale(format!(
            "governed content changed on the branch since dispatch{by}; nothing landed, and that \
             change is untouched"
        )));
    };
    let landed = fingerprint(git, &tree, p)?;
    if landed != verified.fingerprint() {
        bail!(
            "the content to land ({landed}) differs from the verified content ({}); refusing to land",
            verified.fingerprint()
        );
    }
    Ok(Plan::Tree(tree))
}

/// Where the local branch should move after the host merged a job's pull request, or `None` when it need not.
///
/// A squash (or a rebase) merge makes a commit that does not descend from the branch's commits, so the job's
/// landed commit `landed` stays on the local branch but never reaches the base branch's history, and the next
/// pull request cut from the branch conflicts with it. The fix is to rebuild: the commits after `landed` (later
/// landed jobs, records, hand work) are replayed, in order and with their messages, on top of `merged`, the
/// commit the host made. The result holds everything the branch held. A conflict is an error and nothing moves.
///
/// `None` when `merged` is already part of `head` (a merge commit that was pulled, or an earlier resync), or
/// when `landed` is not on `head` (there is nothing of the job to replace).
pub fn resync_tip(
    git: &Git,
    head: &str,
    landed: &str,
    merged: &str,
    sign: bool,
) -> Result<Option<String>> {
    if git.is_ancestor(merged, head)? || !git.is_ancestor(landed, head)? {
        return Ok(None);
    }
    Ok(Some(git.replay(merged, landed, head, sign)?))
}

/// One commit of a pull request, with the job its `Athena-Job` trailer names (§5.3 of
/// `docs/design/records-history.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrCommit {
    pub sha: String,
    pub tree: String,
    pub job: JobId,
}

/// The commits of a pull request, oldest first: every commit in `remote_base..head` (§5.3). Each must carry
/// exactly one `Athena-Job` trailer, which names the job whose verification covers it. A commit without one
/// is an error naming its sha, and nothing is recorded: a commit nothing verified must not be written into a
/// landing record. An empty range is an error too.
pub fn pr_commits(git: &Git, remote_base: &str, head: &str) -> Result<Vec<PrCommit>> {
    let listing = git.run(["rev-list", "--reverse", &format!("{remote_base}..{head}")])?;
    let mut commits = Vec::new();
    for sha in listing.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let trailers = git.trailers(&git.message(sha)?)?;
        let jobs: Vec<&str> = trailers
            .iter()
            .filter(|(key, _)| key == "Athena-Job")
            .map(|(_, value)| value.as_str())
            .collect();
        let [job] = jobs.as_slice() else {
            bail!(
                "commit {sha} lies between the remote base and the PR head (base {remote_base}, head {head}) \
                 and has {} Athena-Job trailers (it needs exactly one), so no verification covers it; \
                 nothing was recorded",
                jobs.len()
            );
        };
        let job = JobId::parse(job).with_context(|| {
            format!(
                "commit {sha} lies between the remote base and the PR head (base {remote_base}, head {head}) \
                 and its Athena-Job trailer {job:?} is not a job id"
            )
        })?;
        commits.push(PrCommit {
            sha: sha.to_string(),
            tree: git.tree_of(sha)?,
            job,
        });
    }
    if commits.is_empty() {
        bail!(
            "the PR head {head} adds no commit to the remote base {remote_base}; nothing to record"
        );
    }
    Ok(commits)
}

impl Athena {
    /// Moves the current branch onto the commit the host made for a merged pull request (see [`resync_tip`]).
    /// Local edits in the checkout are kept; if one is in the way of the move, the branch stays as it was and
    /// the error says so.
    pub fn resync_onto_merge(&self, landed: &str, merged: &str, ui: &dyn Ui) -> Result<()> {
        let _lock = self.state.lock_repository()?;
        let head = self.head()?;
        let sign = self.git.signs_commits()?;
        let Some(tip) = resync_tip(&self.git, &head, landed, merged, sign).with_context(|| {
            format!(
                "could not rebuild the branch on the merged commit {merged}; the branch is unchanged \
                 (it still holds {head})"
            )
        })?
        else {
            return Ok(());
        };
        match self.git.reset_keep(&tip)? {
            Ok(()) => {
                ui.line(&format!(
                    "the branch was rebuilt on the merged commit {}; it held {} before",
                    &merged[..merged.len().min(12)],
                    &head[..head.len().min(12)]
                ));
                Ok(())
            }
            Err(stderr) => bail!(
                "could not move the branch onto the merged commit: {}. The branch is unchanged; move \
                 the local edits aside and run the command again",
                stderr.trim()
            ),
        }
    }
}

/// The first commit after `base` on the way to `head` that changed governed content, with its subject.
fn first_governed_change(git: &Git, base: &str, head: &str) -> Result<Option<(String, String)>> {
    let before = fingerprint(git, base, base)?;
    let commits = git.run([
        "rev-list",
        "--reverse",
        "--first-parent",
        &format!("{base}..{head}"),
    ])?;
    for commit in commits.lines() {
        if fingerprint(git, commit, base)? != before {
            let subject = git.run(["log", "-1", "--format=%s", commit])?;
            return Ok(Some((commit.to_string(), subject.trim().to_string())));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verify::{Judgment, verified};
    use athena_data::records::Verdict;

    fn repo() -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        std::fs::write(
            dir.path().join(".gitattributes"),
            "/PRIVATE.md athena-exclude\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("a.txt"), "a\n").unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "base"]).unwrap();
        (dir, git)
    }

    fn commit(git: &Git, dir: &std::path::Path, path: &str, text: &str) -> String {
        std::fs::write(dir.join(path), text).unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", path]).unwrap();
        git.head().unwrap().unwrap()
    }

    fn pass(base: &str, candidate: &str, fingerprint: &str) -> Verified {
        let judgment = Judgment {
            criteria: vec![],
            findings: vec![],
            agent_verdict: Some(Verdict::Pass),
            final_verdict: Verdict::Pass,
            reasons: vec![],
        };
        verified(&judgment, base, candidate, fingerprint).unwrap()
    }

    /// A base P and a candidate C on a side branch, with C's fingerprint.
    fn base_and_candidate() -> (tempfile::TempDir, Git, String, String, String) {
        let (dir, git) = repo();
        let p = git.head().unwrap().unwrap();
        git.run(["checkout", "-q", "-b", "candidate"]).unwrap();
        let c = commit(&git, dir.path(), "a.txt", "changed\n");
        git.run(["checkout", "-q", "main"]).unwrap();
        let f = fingerprint(&git, &c, &p).unwrap();
        (dir, git, p, c, f)
    }

    fn merge_base(git: &Git, a: &str, b: &str) -> String {
        git.run(["merge-base", a, b]).unwrap().trim().to_string()
    }

    fn commit_message(
        git: &Git,
        dir: &std::path::Path,
        path: &str,
        text: &str,
        message: &str,
    ) -> String {
        let full = dir.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, text).unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", message]).unwrap();
        git.head().unwrap().unwrap()
    }

    #[test]
    fn after_a_squash_the_branch_is_rebuilt_on_it_and_the_next_pull_request_is_clean() {
        let (dir, git) = repo();
        let root = dir.path();
        std::fs::write(root.join("a.txt"), "1\n2\n3\n").unwrap();
        git.run(["commit", "-q", "-am", "lines"]).unwrap();
        let start = git.head().unwrap().unwrap();
        // The branch: job A lands, then job B (which edits A's line again), then B's records, then a hand edit.
        let landed = commit_message(
            &git,
            root,
            "a.txt",
            "A\n2\n3\n",
            "Job A\n\nAthena-Job: ja\n",
        );
        commit_message(
            &git,
            root,
            "a.txt",
            "B\n2\n3\n",
            "Job B\n\nAthena-Job: jb\n",
        );
        commit_message(
            &git,
            root,
            "records/b.json",
            "{}\n",
            "Records\n\nAthena-Job: jb\n",
        );
        let head = commit_message(&git, root, "notes.txt", "hand\n", "Hand edit\n");

        // The host squashes job A's pull request onto main, which also gained a commit of its own.
        git.run(["checkout", "-q", "-b", "origin-main", &start])
            .unwrap();
        commit_message(&git, root, "other.txt", "o\n", "Other work");
        let moved = git.head().unwrap().unwrap();
        let text = git.squash_text(&landed).unwrap();
        let squash_tree = git.merge_tree(&start, &moved, &landed).unwrap().unwrap();
        let squash = git
            .commit_tree(
                &squash_tree,
                &[&moved],
                &format!("{}\n\n{}", text.title, text.body),
                false,
            )
            .unwrap();
        git.run(["checkout", "-q", "main"]).unwrap();
        assert!(!git.is_ancestor(&squash, &head).unwrap());

        // Without the rebuild, a pull request cut from the branch conflicts with the squash.
        let before = merge_base(&git, &squash, &head);
        assert_eq!(git.merge_tree(&before, &squash, &head).unwrap(), None);

        let tip = resync_tip(&git, &head, &landed, &squash, false)
            .unwrap()
            .expect("the branch must move");
        assert!(git.is_ancestor(&squash, &tip).unwrap());
        // Later landed jobs, records and hand work are kept, in order, with their messages.
        let subjects = git
            .run(["log", "--format=%s", &format!("{squash}..{tip}")])
            .unwrap();
        assert_eq!(
            subjects.lines().collect::<Vec<_>>(),
            ["Hand edit", "Records", "Job B"]
        );
        let show =
            |path: &str| String::from_utf8(git.show_file(&tip, path).unwrap().unwrap()).unwrap();
        assert_eq!(show("a.txt"), "B\n2\n3\n");
        assert_eq!(show("records/b.json"), "{}\n");
        assert_eq!(show("notes.txt"), "hand\n");
        assert_eq!(show("other.txt"), "o\n");
        let job_b = git.message(&format!("{tip}~2")).unwrap();
        assert!(
            git.trailers(&job_b)
                .unwrap()
                .contains(&("Athena-Job".into(), "jb".into()))
        );

        // Move the branch there; the next job's pull request branch merges with main without a conflict.
        git.run(["reset", "-q", "--hard", &tip]).unwrap();
        commit_message(&git, root, "next.txt", "n\n", "Job C\n\nAthena-Job: jc\n");
        let next = git.head().unwrap().unwrap();
        let base_of_next = merge_base(&git, &squash, &next);
        assert_eq!(base_of_next, squash);
        assert!(
            git.merge_tree(&base_of_next, &squash, &next)
                .unwrap()
                .is_some()
        );

        // A second run changes nothing, and a branch that does not hold the job's commit is left alone.
        assert_eq!(
            resync_tip(&git, &next, &landed, &squash, false).unwrap(),
            None
        );
        assert_eq!(
            resync_tip(&git, &squash, &landed, &squash, false).unwrap(),
            None
        );
    }

    #[test]
    fn a_squash_the_later_work_cannot_be_carried_over_is_an_error_that_moves_nothing() {
        let (dir, git) = repo();
        let root = dir.path();
        let landed = commit_message(&git, root, "a.txt", "A\n", "Job A\n\nAthena-Job: ja\n");
        let head = commit_message(&git, root, "a.txt", "B\n", "Job B\n\nAthena-Job: jb\n");
        // The host's squash differs from the landed commit in the very line job B changes.
        git.run(["checkout", "-q", "-b", "side", "HEAD~2"]).unwrap();
        let squash = commit_message(&git, root, "a.txt", "squash differs\n", "Job A (#1)\n");
        git.run(["checkout", "-q", "main"]).unwrap();
        let err = resync_tip(&git, &head, &landed, &squash, false).unwrap_err();
        assert!(err.to_string().contains("conflicts"), "{err}");
        assert_eq!(git.head().unwrap().unwrap(), head);
    }

    #[test]
    fn a_verified_candidate_lands_on_its_base() {
        // M7 (allows): a verified candidate lands.
        let (_dir, git, p, c, f) = base_and_candidate();
        assert_eq!(
            plan(&git, &pass(&p, &c, &f), &p).unwrap(),
            Plan::Tree(git.tree_of(&c).unwrap())
        );
    }

    #[test]
    fn landing_refuses_content_that_differs_from_the_verified_content() {
        // M7 (blocks): landing after the candidate changed.
        let (_dir, git, p, c, _f) = base_and_candidate();
        let wrong = fingerprint(&git, &p, &p).unwrap();
        assert!(plan(&git, &pass(&p, &c, &wrong), &p).is_err());
    }

    #[test]
    fn a_base_that_is_no_longer_an_ancestor_makes_the_job_stale() {
        // M7 (blocks): landing when the base is no longer an ancestor.
        let (dir, git, p, c, f) = base_and_candidate();
        git.run(["checkout", "-q", "--orphan", "rewritten"])
            .unwrap();
        let rewritten = commit(&git, dir.path(), "b.txt", "history was rewritten\n");
        assert!(matches!(
            plan(&git, &pass(&p, &c, &f), &rewritten).unwrap(),
            Plan::Stale(_)
        ));
    }

    #[test]
    fn a_changed_exclusion_rule_makes_the_job_stale() {
        // M7 (blocks): landing after an exclusion rule changed.
        let (dir, git, p, c, f) = base_and_candidate();
        let head = commit(
            &git,
            dir.path(),
            ".gitattributes",
            "/PRIVATE.md athena-exclude\n/a.txt athena-exclude\n",
        );
        assert!(matches!(
            plan(&git, &pass(&p, &c, &f), &head).unwrap(),
            Plan::Stale(_)
        ));
    }
}
