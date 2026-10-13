//! Pull-request landing, owned mode: the core logic with no network and no host client
//! (`docs/design/native-pr-landing.md`, slice 1).
//!
//! Three pieces:
//! - [`PrLink`]: the pull request number, host, repository, pushed head and job id, kept in the job's state
//!   directory (never in the repository's `records/`, design §5.1 step 3);
//! - [`check_push`]: the push guard. It refuses any branch other than the job's work branch and any push that
//!   is not a fast-forward. It has no force option;
//! - [`check_merged`]: the merged-content check. The merge is found by the stored pull request number, never
//!   by reading trailers from the merged commit, and the merged content must have the verified fingerprint.
//!
//! The host and git sit behind small traits so the logic is tested without a network.

use anyhow::{Context, Result};
use athena_data::JobId;
use athena_git::Git;
use athena_git::attrs::fingerprint;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::state::StateDir;

/// Where a job's pull request stands, as far as Athena has observed it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkState {
    /// Opened, not yet seen merged.
    #[default]
    Open,
    /// Merged, and the merged content has the verified fingerprint.
    Landed,
    /// Merged, but the merged content is not the verified content. The job is not recorded as landed. The
    /// user moves it out by re-verifying the job on the new tip as a new job (design §5.4).
    MergedUnverified,
}

/// The link between a job and its pull request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrLink {
    pub pr_number: u64,
    pub host: String,
    pub repository: String,
    /// The commit id that was pushed as the pull request's head.
    pub head: String,
    pub job: String,
    #[serde(default)]
    pub state: LinkState,
    /// The merged commit id the host reported, once seen.
    #[serde(default)]
    pub merged_commit: Option<String>,
}

/// The file that holds a job's link, under the job's state directory.
pub fn link_path(state: &StateDir, job: &JobId) -> PathBuf {
    state.job_dir(job).join("pr-link.json")
}

/// Writes the link for `job`. The file is in the state directory, outside the repository.
pub fn store_link(state: &StateDir, job: &JobId, link: &PrLink) -> Result<()> {
    let path = link_path(state, job);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut text = serde_json::to_string_pretty(link)?;
    text.push('\n');
    std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))
}

/// Reads the link for `job`, or `None` when no pull request was opened for it.
pub fn load_link(state: &StateDir, job: &JobId) -> Result<Option<PrLink>> {
    let path = link_path(state, job);
    match std::fs::read(&path) {
        Ok(bytes) => Ok(Some(
            serde_json::from_slice(&bytes)
                .with_context(|| format!("parsing {}", path.display()))?,
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e).with_context(|| format!("reading {}", path.display())),
    }
}

/// The ancestry question the push guard asks of git.
pub trait Ancestry {
    fn is_ancestor(&self, ancestor: &str, descendant: &str) -> Result<bool>;
}

impl Ancestry for Git {
    fn is_ancestor(&self, ancestor: &str, descendant: &str) -> Result<bool> {
        Ok(Git::is_ancestor(self, ancestor, descendant)?)
    }
}

/// A push Athena is about to make.
#[derive(Debug, Clone, Copy)]
pub struct PushRequest<'a> {
    /// The only branch Athena may push for this job: its work branch (or, in stealth mode, the neutral
    /// name chosen for it).
    pub allowed_branch: &'a str,
    /// The branch the caller wants to push.
    pub branch: &'a str,
    /// The commit to push.
    pub local_head: &'a str,
    /// The commit the remote branch holds now, or `None` when it does not exist yet.
    pub remote_head: Option<&'a str>,
}

/// Why the guard refused a push.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PushRefusal {
    WrongBranch { wanted: String, allowed: String },
    NotFastForward { remote: String, local: String },
}

impl std::fmt::Display for PushRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PushRefusal::WrongBranch { wanted, allowed } => write!(
                f,
                "refusing to push {wanted:?}: Athena pushes only the job's own branch, {allowed:?}"
            ),
            PushRefusal::NotFastForward { remote, local } => write!(
                f,
                "refusing to push {local}: the remote branch holds {remote}, which it does not contain; \
                 Athena never forces a push"
            ),
        }
    }
}

impl std::error::Error for PushRefusal {}

/// The push guard. `Ok` only for the job's own branch and a fast-forward (or a new branch). A remote head
/// that git cannot place in the local history is refused too, because a fast-forward cannot be shown.
pub fn check_push(git: &dyn Ancestry, request: &PushRequest) -> Result<(), PushRefusal> {
    if request.branch != request.allowed_branch || request.branch.is_empty() {
        return Err(PushRefusal::WrongBranch {
            wanted: request.branch.to_string(),
            allowed: request.allowed_branch.to_string(),
        });
    }
    if let Some(remote) = request.remote_head
        && !git.is_ancestor(remote, request.local_head).unwrap_or(false)
    {
        return Err(PushRefusal::NotFastForward {
            remote: remote.to_string(),
            local: request.local_head.to_string(),
        });
    }
    Ok(())
}

/// A replacement of the job's own branch head that Athena is about to make.
#[derive(Debug, Clone, Copy)]
pub struct ReplaceRequest<'a> {
    /// The only branch Athena may replace for this job: its work branch.
    pub allowed_branch: &'a str,
    /// The branch the caller wants to replace.
    pub branch: &'a str,
    /// The commit that will become the branch head.
    pub local_head: &'a str,
    /// The commit the remote branch holds now, as just observed, or `None` when it could not be read.
    pub remote_head: Option<&'a str>,
}

/// The guard for replacing a branch head. `Ok` only for the job's own branch, with a remote head that was
/// observed and that the local object store knows. Any other case is an error and nothing is pushed:
/// a remote head git cannot place locally, or an ancestry check that fails, is reported, never forced over.
/// The ancestry answer itself does not matter (the job's own branch may be rewritten), but it must be
/// answerable, because an unanswerable one means the remote head is not understood.
pub fn check_replace(git: &dyn Ancestry, request: &ReplaceRequest) -> Result<()> {
    if request.branch != request.allowed_branch || request.branch.is_empty() {
        return Err(PushRefusal::WrongBranch {
            wanted: request.branch.to_string(),
            allowed: request.allowed_branch.to_string(),
        }
        .into());
    }
    let Some(remote) = request.remote_head else {
        anyhow::bail!(
            "the remote has no branch {:?} to replace; nothing was pushed",
            request.branch
        );
    };
    git.is_ancestor(remote, request.local_head)
        .with_context(|| {
            format!(
                "cannot tell how the remote head {remote} relates to {}; nothing was pushed",
                request.local_head
            )
        })?;
    Ok(())
}

/// What the host says about a pull request, asked by number.
pub trait Host {
    /// The commit the pull request was merged as (merge commit, squash or rebase), or `None` if it is not
    /// merged.
    fn merged_commit(&self, link: &PrLink) -> Result<Option<String>>;
}

/// The governed-content fingerprint of a commit, as the landing check defines it.
pub trait Content {
    fn fingerprint_of(&self, commit: &str) -> Result<String>;
}

/// [`Content`] over a real repository: the fingerprint of a commit under the rules at `rules_commit`
/// (the job's base). A commit missing locally is fetched from `remote` first.
#[derive(Debug)]
pub struct GitContent<'a> {
    pub git: &'a Git,
    pub rules_commit: &'a str,
    /// The remote to fetch a missing commit from.
    pub remote: &'a str,
}

impl Content for GitContent<'_> {
    fn fingerprint_of(&self, commit: &str) -> Result<String> {
        // The host made the merged commit, so it may not be in the local object store yet. Fetch the
        // object (no ref moves) before reading its tree.
        self.git.ensure_commit(self.remote, commit)?;
        Ok(fingerprint(self.git, commit, self.rules_commit)?)
    }
}

/// The result of looking at a pull request after the user may have merged it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeOutcome {
    NotMerged,
    /// Merged, and the merged content is the verified content.
    Verified {
        commit: String,
    },
    /// Merged, but the content is not the verified content.
    MergedButNotVerified {
        commit: String,
        merged_fingerprint: String,
        verified_fingerprint: String,
    },
}

/// Finds the merge by the stored pull request number and compares the merged content's fingerprint with the
/// verified one. The merged commit's message is never read.
pub fn check_merged(
    host: &dyn Host,
    content: &dyn Content,
    link: &PrLink,
    verified_fingerprint: &str,
) -> Result<MergeOutcome> {
    let Some(commit) = host.merged_commit(link)? else {
        return Ok(MergeOutcome::NotMerged);
    };
    let merged_fingerprint = content.fingerprint_of(&commit)?;
    if merged_fingerprint == verified_fingerprint {
        Ok(MergeOutcome::Verified { commit })
    } else {
        Ok(MergeOutcome::MergedButNotVerified {
            commit,
            merged_fingerprint,
            verified_fingerprint: verified_fingerprint.to_string(),
        })
    }
}

/// Records an outcome on the link: a merged-but-refused job gets the named state, and is never `Landed`.
pub fn apply_outcome(link: &mut PrLink, outcome: &MergeOutcome) {
    match outcome {
        MergeOutcome::NotMerged => {}
        MergeOutcome::Verified { commit } => {
            link.state = LinkState::Landed;
            link.merged_commit = Some(commit.clone());
        }
        MergeOutcome::MergedButNotVerified { commit, .. } => {
            link.state = LinkState::MergedUnverified;
            link.merged_commit = Some(commit.clone());
        }
    }
}

/// Looks at the pull request once: finds the merge by the stored number, checks the merged content, and
/// records the outcome on `link`. A link that is no longer open is left as it is.
pub fn observe_merge(
    host: &dyn Host,
    content: &dyn Content,
    link: &mut PrLink,
    verified_fingerprint: &str,
) -> Result<MergeOutcome> {
    if link.state != LinkState::Open {
        return Ok(MergeOutcome::NotMerged);
    }
    let outcome = check_merged(host, content, link, verified_fingerprint)?;
    apply_outcome(link, &outcome);
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn link(job: &str) -> PrLink {
        PrLink {
            pr_number: 42,
            host: "github.com".into(),
            repository: "example/project".into(),
            head: "abc123".into(),
            job: job.into(),
            state: LinkState::Open,
            merged_commit: None,
        }
    }

    #[test]
    fn a_link_is_stored_in_the_job_state_directory_and_loaded_back() {
        let dir = tempfile::tempdir().unwrap();
        let state = StateDir::new(dir.path().to_path_buf());
        let job = JobId::parse("bootstrap").unwrap();
        assert_eq!(load_link(&state, &job).unwrap(), None);
        let l = link("bootstrap");
        store_link(&state, &job, &l).unwrap();
        assert!(link_path(&state, &job).starts_with(dir.path()));
        assert_eq!(load_link(&state, &job).unwrap(), Some(l));
    }

    struct Fixed(bool);
    impl Ancestry for Fixed {
        fn is_ancestor(&self, _: &str, _: &str) -> Result<bool> {
            Ok(self.0)
        }
    }

    fn request<'a>(branch: &'a str, allowed: &'a str, remote: Option<&'a str>) -> PushRequest<'a> {
        PushRequest {
            allowed_branch: allowed,
            branch,
            local_head: "new",
            remote_head: remote,
        }
    }

    #[test]
    fn the_guard_refuses_a_branch_that_is_not_the_jobs_own() {
        // Push only the job's work branch (design §8, row 1).
        let refusal = check_push(&Fixed(true), &request("main", "work/x", None)).unwrap_err();
        assert!(matches!(refusal, PushRefusal::WrongBranch { .. }));
        assert!(check_push(&Fixed(true), &request("work/x", "work/x", None)).is_ok());
    }

    #[test]
    fn the_guard_refuses_a_non_fast_forward_push() {
        // No force push (design §8, row 2), with real git: the remote head is not an ancestor.
        let (_dir, git) = {
            let dir = tempfile::tempdir().unwrap();
            let git = Git::isolated(dir.path());
            git.run(["init", "-q", "-b", "main"]).unwrap();
            (dir, git)
        };
        let commit = |msg: &str| {
            git.run(["commit", "-q", "--allow-empty", "-m", msg])
                .unwrap();
            git.head().unwrap().unwrap()
        };
        let first = commit("first");
        let second = commit("second");
        git.run(["checkout", "-q", "-b", "other", &first]).unwrap();
        let rewritten = commit("rewritten");

        let ok = PushRequest {
            allowed_branch: "work/x",
            branch: "work/x",
            local_head: &second,
            remote_head: Some(&first),
        };
        assert!(check_push(&git, &ok).is_ok());
        let bad = PushRequest {
            local_head: &rewritten,
            remote_head: Some(&second),
            ..ok
        };
        assert!(matches!(
            check_push(&git, &bad),
            Err(PushRefusal::NotFastForward { .. })
        ));
        // A remote head git does not know cannot be shown to be an ancestor, so it is refused.
        let unknown = PushRequest {
            remote_head: Some("0123456789abcdef0123456789abcdef01234567"),
            ..ok
        };
        assert!(check_push(&git, &unknown).is_err());
    }

    struct Failing;
    impl Ancestry for Failing {
        fn is_ancestor(&self, _: &str, _: &str) -> Result<bool> {
            anyhow::bail!("ancestry failed")
        }
    }

    #[test]
    fn the_replace_guard_allows_only_the_own_branch_and_an_understood_remote_head() {
        let ask = |branch, remote| ReplaceRequest {
            allowed_branch: "work/x",
            branch,
            local_head: "new",
            remote_head: remote,
        };
        assert!(check_replace(&Fixed(false), &ask("work/x", Some("old"))).is_ok());
        assert!(check_replace(&Fixed(true), &ask("work/x", Some("old"))).is_ok());
        let wrong = check_replace(&Fixed(true), &ask("main", Some("old"))).unwrap_err();
        assert!(wrong.downcast_ref::<PushRefusal>().is_some());
        assert!(check_replace(&Fixed(true), &ask("", Some("old"))).is_err());
        assert!(check_replace(&Fixed(true), &ask("work/x", None)).is_err());
        let failed = check_replace(&Failing, &ask("work/x", Some("old"))).unwrap_err();
        assert!(format!("{failed:#}").contains("ancestry failed"));
    }

    struct FakeHost {
        merged: Option<String>,
        asked: RefCell<Vec<u64>>,
    }
    impl Host for FakeHost {
        fn merged_commit(&self, link: &PrLink) -> Result<Option<String>> {
            self.asked.borrow_mut().push(link.pr_number);
            Ok(self.merged.clone())
        }
    }

    fn repo() -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        std::fs::write(dir.path().join("a.txt"), "a\n").unwrap();
        std::fs::write(dir.path().join("b.txt"), "b\n").unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "base"]).unwrap();
        (dir, git)
    }

    fn commit_file(
        git: &Git,
        dir: &std::path::Path,
        path: &str,
        text: &str,
        message: &str,
    ) -> String {
        std::fs::write(dir.join(path), text).unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", message]).unwrap();
        git.head().unwrap().unwrap()
    }

    #[test]
    fn the_merge_is_found_by_number_even_with_a_trailing_co_authored_by_paragraph() {
        // F6/F1: no Athena trailer on the merged commit; the lookup uses the stored number only.
        let (dir, git) = repo();
        let p = git.head().unwrap().unwrap();
        git.run(["checkout", "-q", "-b", "candidate"]).unwrap();
        let c = commit_file(&git, dir.path(), "a.txt", "changed\n", "change a");
        git.run(["checkout", "-q", "main"]).unwrap();
        let verified = fingerprint(&git, &c, &p).unwrap();
        // A squash-like commit with the candidate's tree and the host's trailing paragraph.
        let tree = git.tree_of(&c).unwrap();
        let merged = git
            .commit_tree(
                &tree,
                &[&p],
                "Change a (#42)\n\nCo-authored-by: Someone <s@example.com>\n",
                false,
            )
            .unwrap();

        let host = FakeHost {
            merged: Some(merged.clone()),
            asked: RefCell::new(vec![]),
        };
        let content = GitContent {
            git: &git,
            rules_commit: &p,
            remote: "origin",
        };
        let outcome = check_merged(&host, &content, &link("j"), &verified).unwrap();
        assert_eq!(outcome, MergeOutcome::Verified { commit: merged });
        assert_eq!(*host.asked.borrow(), vec![42]);

        let none = FakeHost {
            merged: None,
            asked: RefCell::new(vec![]),
        };
        assert_eq!(
            check_merged(&none, &content, &link("j"), &verified).unwrap(),
            MergeOutcome::NotMerged
        );
    }

    #[test]
    fn a_merged_commit_missing_locally_is_fetched_before_it_is_fingerprinted() {
        // The host merged the pull request after the local clone last fetched, so the merged commit is on
        // the remote only.
        let (origin_dir, origin) = repo();
        let p = origin.head().unwrap().unwrap();
        let parent = tempfile::tempdir().unwrap();
        let clone_path = parent.path().join("clone");
        Git::isolated(parent.path())
            .run([
                "clone",
                "-q",
                origin_dir.path().to_str().unwrap(),
                clone_path.to_str().unwrap(),
            ])
            .unwrap();
        let local = Git::isolated(&clone_path);
        let merged = commit_file(
            &origin,
            origin_dir.path(),
            "a.txt",
            "changed\n",
            "Merge pull request #42",
        );
        assert_eq!(local.rev_parse(&merged).unwrap(), None);
        let refs_before = local.run(["for-each-ref"]).unwrap();

        let content = GitContent {
            git: &local,
            rules_commit: &p,
            remote: "origin",
        };
        let got = content.fingerprint_of(&merged).unwrap();
        assert_eq!(got, fingerprint(&origin, &merged, &p).unwrap());
        // Only the object arrived: no ref moved.
        assert_eq!(local.run(["for-each-ref"]).unwrap(), refs_before);
        assert_eq!(local.head().unwrap().unwrap(), p);
    }

    #[test]
    fn disjoint_file_merges_are_merged_but_not_verified() {
        // F4/N1: two jobs from one base touch different files; the second merge holds both changes, which
        // is the content of neither verification.
        let (dir, git) = repo();
        let p = git.head().unwrap().unwrap();
        git.run(["checkout", "-q", "-b", "job-a"]).unwrap();
        let a = commit_file(&git, dir.path(), "a.txt", "from a\n", "a");
        git.run(["checkout", "-q", "main"]).unwrap();
        git.run(["checkout", "-q", "-b", "job-b"]).unwrap();
        let b = commit_file(&git, dir.path(), "b.txt", "from b\n", "b");
        git.run(["checkout", "-q", "main"]).unwrap();
        let verified_b = fingerprint(&git, &b, &p).unwrap();
        // Job a lands first; then b's pull request merges cleanly on top of it.
        git.run(["merge", "-q", "--ff-only", &a]).unwrap();
        git.run(["merge", "-q", "--no-ff", "-m", "Merge b (#42)", &b])
            .unwrap();
        let merged = git.head().unwrap().unwrap();

        let host = FakeHost {
            merged: Some(merged.clone()),
            asked: RefCell::new(vec![]),
        };
        let content = GitContent {
            git: &git,
            rules_commit: &p,
            remote: "origin",
        };
        let mut l = link("j");
        let outcome = check_merged(&host, &content, &l, &verified_b).unwrap();
        match &outcome {
            MergeOutcome::MergedButNotVerified {
                commit,
                verified_fingerprint,
                merged_fingerprint,
            } => {
                assert_eq!(commit, &merged);
                assert_eq!(verified_fingerprint, &verified_b);
                assert_ne!(merged_fingerprint, verified_fingerprint);
            }
            other => panic!("expected a refusal, got {other:?}"),
        }
        apply_outcome(&mut l, &outcome);
        assert_eq!(l.state, LinkState::MergedUnverified);
        assert_eq!(l.merged_commit, Some(merged));
    }
}
