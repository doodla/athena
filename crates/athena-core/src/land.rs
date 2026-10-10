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
        return Ok(Plan::Stale(
            "governed content changed on the branch since dispatch".into(),
        ));
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
