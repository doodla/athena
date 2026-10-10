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
use anyhow::{Result, bail};
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
