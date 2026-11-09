//! The landing decision (design §7.3; M7).
//!
//! ```text
//! H := tip of the user's current branch
//! require the current branch is the branch the job was dispatched from   else stale
//! require P is an ancestor of H                          else stale
//! require the athena-* classification of every path in P ∪ H ∪ C is the same under P's and H's rules
//! if H == P:                                     tree := tree(C)
//! elif fingerprint(H; P) == fingerprint(P; P):   tree := merge-tree --merge-base P H C
//! else:                                          stale (governed base content changed)
//! assert fingerprint(tree; P) == F               (the landed content is the verified content)
//! ```
//!
//! A job lands only on the branch it was dispatched from, even when its base commit is an ancestor of
//! another branch's tip. The dispatch branch is recorded in the job record (`dispatch_branch`). A job
//! dispatched on a detached HEAD has the empty string there; it names no branch, so it never lands. A
//! branch job finished on a detached HEAD does not land either. A job record made before the field
//! existed has no dispatch branch, and that job keeps the older rule: only the ancestor check applies.
//!
//! Before any of this, landing needs the working copy on a branch: [`refuse_detached`] stops a detached
//! HEAD with an error naming the commit and the branch the job needs, so nothing is recorded or committed.
//!
//! The caller adds the closing records to the tree, gates the commit, and fast-forwards; a refused
//! fast-forward makes the job `blocked`.
//!
//! A stale job is not lost. `athena retry <job>` merges the candidate the job kept onto the branch tip
//! (`Athena::stale_seed` in `run`): a clean merge is checked and verified again as a new job, with no
//! planner or producer run, and a conflict goes to a producer run seeded from the conflicted merge. The new
//! job lands through this same rule, so the last line above still holds for it: its landed content must equal
//! the content verified on the merged tree, and a landing whose fingerprint differs is refused. What makes a
//! job stale is unchanged.

use crate::gh::{GitRemote, Remote, verification_id};
use crate::pr::{PushRequest, check_push};
use crate::store::Records;
use crate::verify::Verified;
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_data::history::LandingKind;
use athena_data::records::AnyRecord;
use athena_data::{JobId, RecordId};
use athena_gate::invariant::classify_pre_merge;
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
    /// The checked-out branch is not the one the job was dispatched from; nothing lands here. The text
    /// names the dispatch branch.
    OtherBranch(String),
}

/// The branch a job was dispatched from, as its job record holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dispatched {
    /// The record has no dispatch branch (made before the field existed): no branch check.
    Unknown,
    /// HEAD was on this branch at dispatch.
    Branch(String),
    /// HEAD was detached at dispatch (recorded as an empty branch name).
    Detached,
}

impl Dispatched {
    /// Reads the record's `dispatch_branch`: absent is `Unknown`, empty is `Detached`.
    pub fn from_record(recorded: Option<&str>) -> Self {
        match recorded {
            None => Dispatched::Unknown,
            Some("") => Dispatched::Detached,
            Some(name) => Dispatched::Branch(name.to_string()),
        }
    }
}

/// The working-copy precondition of landing: HEAD must be on a branch. A detached HEAD stops landing with
/// an error that names the commit HEAD is at and the branch the job needs, before any record or commit is
/// built. Without this check a detached HEAD surfaced further on as a stale outcome committed onto the
/// detached commit, or as a job without a verification.
///
/// A different checked-out branch is not refused here: `plan` handles it with its own text.
pub fn refuse_detached(git: &Git, expected: &Dispatched) -> Result<()> {
    if git.current_branch()?.is_some() {
        return Ok(());
    }
    let at = git.head()?.unwrap_or_default();
    let at = &at[..at.len().min(12)];
    let wanted = match expected {
        Dispatched::Branch(name) => format!("branch `{name}` (the job's dispatch branch)"),
        Dispatched::Detached | Dispatched::Unknown => "a checked-out branch".to_string(),
    };
    bail!(
        "the working copy's HEAD is detached at {at}; landing needs {wanted} checked out; nothing \
         landed. Check the branch out, then run again"
    )
}

/// The branch `id` was dispatched from, read from its job record at `head`. A `head` that lacks the job
/// (a HEAD detached on an older commit) falls back to the first local branch, in name order, whose tip
/// holds a job record with a dispatch branch. The result is `Unknown` when no record names one.
pub fn dispatched_from(git: &Git, head: &str, id: &JobId) -> Result<Dispatched> {
    let records = Records::load(git, head)?;
    if let Some(job) = records.jobs.get(id.as_str()).and_then(|j| j.job.as_ref()) {
        return Ok(Dispatched::from_record(job.dispatch_branch.as_deref()));
    }
    let path = format!("{}/job.json", id.dir());
    let listed = git.run(["for-each-ref", "--format=%(refname:short)", "refs/heads"])?;
    let mut branches: Vec<&str> = listed.lines().filter(|b| !b.is_empty()).collect();
    branches.sort_unstable();
    for branch in branches {
        let Some(bytes) = git.show_file(branch, &path)? else {
            continue;
        };
        if let Ok(AnyRecord::Job(job)) = AnyRecord::parse(&path, &bytes)
            && job
                .dispatch_branch
                .as_deref()
                .is_some_and(|b| !b.is_empty())
        {
            return Ok(Dispatched::from_record(job.dispatch_branch.as_deref()));
        }
    }
    Ok(Dispatched::Unknown)
}

/// Decides how `verified` lands on `head`, the tip of the checked-out branch. A job dispatched from a
/// branch lands only while that branch is checked out.
pub fn plan(git: &Git, verified: &Verified, head: &str, dispatched: &Dispatched) -> Result<Plan> {
    let p = verified.base();
    let c = verified.candidate();
    match dispatched {
        Dispatched::Unknown => {}
        Dispatched::Detached => {
            return Ok(Plan::OtherBranch(
                "the job was dispatched on a detached HEAD (recorded as an empty branch name), which \
                 names no branch to land on; nothing landed"
                    .into(),
            ));
        }
        Dispatched::Branch(name) => match git.current_branch()? {
            Some(current) if current == *name => {}
            Some(current) => {
                return Ok(Plan::OtherBranch(format!(
                    "the job was dispatched on branch `{name}`, but branch `{current}` is checked out; \
                     nothing landed"
                )));
            }
            None => {
                return Ok(Plan::OtherBranch(format!(
                    "the job was dispatched on branch `{name}`, but HEAD is detached; nothing landed"
                )));
            }
        },
    }
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
/// pull request cut from the branch conflicts with it. The fix adds to the branch and rewrites nothing: the
/// result is a merge commit whose parents are `head` and `merged`, the commit the host made. Every commit the
/// branch held, and the commit its remote copy holds, stays an ancestor, so a plain push of the new tip is
/// accepted.
///
/// The merge takes `landed` as its merge base, so only what the host's commit holds beyond the landed job
/// comes in; later edits of the lines the job landed do not conflict. The merge commit must hold the same
/// governed content as `head` (the host's commit equals the landed job when the branch was up to date at the
/// merge), so it classifies as records only. A conflict, or a governed difference from `head`, is an error and
/// nothing moves.
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
    let Some(tree) = git.merge_tree(landed, head, merged)? else {
        bail!(
            "the host's commit {merged} conflicts with work on the branch after the landed commit \
             {landed}; merge it by hand"
        );
    };
    if fingerprint(git, &tree, head)? != fingerprint(git, head, head)? {
        let first = git
            .diff_tree(head, &tree)?
            .into_iter()
            .next()
            .map(|e| e.path)
            .unwrap_or_default();
        bail!(
            "the host's commit {merged} changes governed content the branch does not hold (first: \
             {first}); merge it by hand"
        );
    }
    let short = &merged[..merged.len().min(12)];
    let message = format!("Merge the host's commit {short} into the branch\n");
    Ok(Some(git.commit_tree(
        &tree,
        &[head, merged],
        &message,
        sign,
    )?))
}

/// One commit of a pull request, classified by what it contains (§5.3 of
/// `docs/design/records-history.md`). `job` and `verification` are set exactly for
/// [`LandingKind::JobLanding`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrCommit {
    pub sha: String,
    pub tree: String,
    pub kind: LandingKind,
    /// The job whose closing records the commit adds.
    pub job: Option<JobId>,
    /// The record that verifies that job's work.
    pub verification: Option<RecordId>,
}

/// The commits of a pull request, oldest first: every commit in `remote_base..head` (§5.3), each classified
/// from its content alone. Messages are never read. A commit is judged against its first parent, under the
/// first parent's exclusion rules (the gate's own view of it), by the first rule that applies:
///
/// 1. It adds the closing records of a job (`athena_gate::invariant::closing_jobs`, the single source of
///    truth for "this commit lands a job"): [`LandingKind::JobLanding`], carrying that job and its
///    verification record, which must be in the commit's tree. A job whose recorded outcome is not landed
///    does not count, so the commit that adds a blocked outcome is records only, and the later commit that
///    adds `outcome-landed.json` is the landing. The records of a held job (a candidate with no
///    verification and no outcome) do not count either: they claim no landing. Closing records of more
///    than one job are an error.
/// 2. It adds an intervention record (`records/interventions/<month>/<id>.json`):
///    [`LandingKind::DeclaredIntervention`].
/// 3. Otherwise, if the governed fingerprint changed, nothing covers the change: an error.
/// 4. Otherwise it touched only records and paths the rules exclude: [`LandingKind::RecordsOnly`]. An owner
///    path counts only where the rules exclude it, so one without the exclusion falls under rule 3.
///
/// An error names the commit's sha, the base and the head, and nothing is recorded: a commit nothing
/// verified must not be written into a landing record. An empty range is an error too.
///
/// The head is always the landing commit: the candidate that `go` pushes early, when verification starts, is
/// replaced by the landing commit before the head is judged (`gh::land_pr`), so it never appears here.
pub fn pr_commits(git: &Git, remote_base: &str, head: &str) -> Result<Vec<PrCommit>> {
    pr_commits_excluding(git, remote_base, head, &[])
}

/// [`pr_commits`] without the commits reachable from any of `landed_heads`: the heads of pull requests
/// that already landed. The working branch keeps a landed job's commits after the host squashed them
/// (the resync adds a merge commit and rewrites nothing), so they stay in `remote_base..head`; a landing
/// record must not list them a second time. A head this repository does not hold is ignored.
pub fn pr_commits_excluding(
    git: &Git,
    remote_base: &str,
    head: &str,
    landed_heads: &[String],
) -> Result<Vec<PrCommit>> {
    let mut args = vec![
        "rev-list".to_string(),
        "--reverse".to_string(),
        format!("{remote_base}..{head}"),
    ];
    for landed in landed_heads {
        if git.commit_exists(landed)? {
            args.push(format!("^{landed}"));
        }
    }
    let listing = git.run(args)?;
    let mut commits = Vec::new();
    for sha in listing.lines().map(str::trim).filter(|l| !l.is_empty()) {
        commits.push(classify_commit(git, remote_base, head, sha)?);
    }
    if commits.is_empty() {
        bail!(
            "the PR head {head} adds no commit to the remote base {remote_base}; nothing to record"
        );
    }
    Ok(commits)
}

/// Classifies the one commit `sha` of the range `remote_base..head` by the rules of [`pr_commits`], which
/// calls this for each commit. It is public so a caller can judge every commit and report all the bad ones
/// rather than only the first.
pub fn classify_commit(git: &Git, remote_base: &str, head: &str, sha: &str) -> Result<PrCommit> {
    let between = || {
        format!(
            "commit {sha} lies between the remote base and the PR head (base {remote_base}, head {head})"
        )
    };
    let covered = match classify_pre_merge(git, sha)? {
        Ok(covered) => covered,
        Err(uncovered) => bail!("{} and {}", between(), uncovered.clause()),
    };
    let verification = match &covered.job {
        Some(job) => Some(verification_id(job, sha)?),
        None => None,
    };
    Ok(PrCommit {
        sha: sha.to_string(),
        tree: covered.tree,
        kind: covered.kind,
        job: covered.job,
        verification,
    })
}

impl Athena {
    /// Merges the commit the host made for a merged pull request into the current branch (see
    /// [`resync_tip`]): the branch moves to a new merge commit that has its old tip as first parent, so no
    /// commit is rewritten and the branch stays a descendant of its remote copy. Local edits in the checkout
    /// are kept; if one is in the way of the move, the branch stays as it was and the error says so. A
    /// conflicting merge is an error and the branch is unchanged too.
    pub fn resync_onto_merge(&self, landed: &str, merged: &str, ui: &dyn Ui) -> Result<()> {
        let _lock = self.state.lock_repository()?;
        let head = self.head()?;
        let sign = self.git.signs_commits()?;
        let Some(tip) = resync_tip(&self.git, &head, landed, merged, sign).with_context(|| {
            format!(
                "could not merge the host's commit {merged} into the branch; the branch is unchanged \
                 (it still holds {head})"
            )
        })?
        else {
            return Ok(());
        };
        match self.git.reset_keep(&tip)? {
            Ok(()) => {
                ui.line(&format!(
                    "the branch merged the host's commit {}; it held {} before",
                    &merged[..merged.len().min(12)],
                    &head[..head.len().min(12)]
                ));
                Ok(())
            }
            Err(stderr) => bail!(
                "could not move the branch onto the merge of the host's commit: {}. The branch is \
                 unchanged; move the local edits aside and run the command again",
                stderr.trim()
            ),
        }
    }

    /// Pushes the checked-out working branch to `origin` with a plain push, after a landing. It returns whether
    /// the remote holds the branch's tip afterwards.
    ///
    /// It never forces and never pushes the default branch (changes reach that through pull requests): the
    /// default branch is the one `origin/HEAD` names, or `main` or `master` when that is not set. A branch
    /// the remote holds at a commit that is not an ancestor of the local tip is not pushed (the same
    /// fast-forward test as [`crate::pr::check_push`]). A refusal or a failed push is a `warning:` line, not
    /// an error: the landing stands. With no current branch or no `origin` it does nothing and says nothing.
    pub fn push_working_branch(&self, ui: &dyn Ui) -> bool {
        let Ok(Some(branch)) = self.git.current_branch() else {
            return false;
        };
        if self.git.run(["remote", "get-url", "origin"]).is_err() {
            return false;
        }
        let default = crate::reconcile::local_default_branch(&self.git, "origin");
        let is_default = match &default {
            Some(name) => *name == branch,
            None => branch == "main" || branch == "master",
        };
        if is_default {
            ui.line(&format!(
                "not pushing {branch} to origin: it is the default branch; changes reach it through pull \
                 requests"
            ));
            return false;
        }
        let Ok(Some(head)) = self.git.head() else {
            return false;
        };
        let warn = |why: &dyn std::fmt::Display| {
            ui.line(&format!(
                "warning: could not push {branch} to origin: {why}; the landing stands"
            ));
        };
        let remote = GitRemote {
            git: &self.git,
            name: "origin",
        };
        let remote_head = match remote.remote_head(&branch) {
            Ok(found) => found,
            Err(problem) => {
                warn(&format!("{problem:#}"));
                return false;
            }
        };
        if remote_head.as_deref() == Some(head.as_str()) {
            return true;
        }
        let request = PushRequest {
            allowed_branch: &branch,
            branch: &branch,
            local_head: &head,
            remote_head: remote_head.as_deref(),
        };
        if let Err(refusal) = check_push(&self.git, &request) {
            warn(&refusal);
            return false;
        }
        match remote.push(&branch, &head) {
            Ok(()) => {
                ui.line(&format!(
                    "pushed {branch} to origin at {}",
                    &head[..head.len().min(12)]
                ));
                true
            }
            Err(problem) => {
                warn(&format!("{problem:#}"));
                false
            }
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

/// Job records for tests that need a pull request holding a real job landing.
#[cfg(test)]
pub(crate) mod fixtures {
    use athena_data::records::{
        AnyRecord, Candidate, Outcome, OutcomeState, Totals, Verdict, Verification,
    };
    use athena_data::{JobId, Quantity, Unit};
    use athena_git::Git;
    use std::collections::BTreeMap;

    /// A file to commit: its path and its text.
    pub type File = (String, String);

    fn file(record: AnyRecord) -> File {
        (record.canonical_path(), record.to_json())
    }

    pub fn candidate(job: &JobId) -> File {
        file(AnyRecord::Candidate(Box::new(Candidate {
            job: job.clone(),
            base_commit: "0".repeat(40),
            base_fingerprint: None,
            previous_job: None,
            candidate_commit: Some("c".repeat(40)),
            candidate_tree: None,
            changes: vec![],
            fingerprint: Some("f".repeat(64)),
            violations: vec![],
            producer_report: None,
        })))
    }

    pub fn verification(job: &JobId) -> File {
        file(AnyRecord::Verification(Box::new(Verification {
            job: job.clone(),
            candidate_commit: "c".repeat(40),
            verified_fingerprint: "f".repeat(64),
            tool_config_from_base: vec![],
            checks: vec![],
            check_cleanup: vec![],
            verifier_invocation: None,
            criteria: vec![],
            findings: vec![],
            request_reading_mismatch: None,
            agent_verdict: Some(Verdict::Pass),
            final_verdict: Verdict::Pass,
            reasons: vec![],
            inputs: vec![],
            summary: None,
        })))
    }

    /// `outcome.json`, or `outcome-<state>.json` when `after_blocked`.
    pub fn outcome(job: &JobId, state: OutcomeState, after_blocked: bool) -> File {
        let zero = || Quantity::reported(0.0, Unit::Usd, "sum");
        file(AnyRecord::Outcome(Box::new(Outcome {
            job: job.clone(),
            state,
            reason: "test".into(),
            at: "2026-10-24T12:00:00Z".parse().unwrap(),
            totals: Some(Totals {
                cost_all: zero(),
                cost_execution: zero(),
                wall_time_execution: None,
                tokens: BTreeMap::new(),
                invocations: 0,
            }),
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked,
            through_commit: None,
        })))
    }

    /// The closing records of a job that landed in one commit.
    pub fn landed(job: &JobId) -> Vec<File> {
        vec![
            candidate(job),
            verification(job),
            outcome(job, OutcomeState::Landed, false),
        ]
    }

    /// Writes `files` under the work tree of `git`, stages everything and commits with `message`; returns
    /// the new commit.
    pub fn commit(git: &Git, files: &[File], message: &str) -> String {
        for (path, text) in files {
            let full = git.dir().join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, text).unwrap();
        }
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", message]).unwrap();
        git.head().unwrap().unwrap()
    }

    /// A file with a fixed text, for governed or owner changes.
    pub fn plain(path: &str, text: &str) -> File {
        (path.to_string(), text.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verify::{Judgment, verified};
    use athena_data::records::{OutcomeState, Verdict};
    use fixtures::plain;

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

    /// A branch `work` with job A landed, then job B (which edits A's line again), B's records and a hand
    /// edit, and the host's squash of job A on top of the starting commit (its tree equals the landed
    /// tree). Returns the repository with `work` checked out, the starting commit, the landed commit, the
    /// branch tip and the squash.
    fn squashed_branch() -> (tempfile::TempDir, Git, String, String, String, String) {
        let (dir, git) = repo();
        let root = dir.path();
        std::fs::write(root.join("a.txt"), "1\n2\n3\n").unwrap();
        git.run(["commit", "-q", "-am", "lines"]).unwrap();
        let start = git.head().unwrap().unwrap();
        git.run(["checkout", "-q", "-b", "work"]).unwrap();
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
        // The host squashes job A's pull request onto the starting commit.
        let text = git.squash_text(&landed).unwrap();
        let squash = git
            .commit_tree(
                &git.tree_of(&landed).unwrap(),
                &[&start],
                &format!("{} (#1)\n\n{}", text.title, text.body),
                false,
            )
            .unwrap();
        assert!(!git.is_ancestor(&squash, &head).unwrap());
        (dir, git, start, landed, head, squash)
    }

    /// A bare repository added to `git` as `origin`.
    fn add_origin(git: &Git) -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        git.run(["init", "--bare", "-q", path]).unwrap();
        git.run(["remote", "add", "origin", path]).unwrap();
        let bare = Git::isolated(dir.path());
        (dir, bare)
    }

    #[test]
    fn after_a_squash_the_branch_is_merged_with_it_and_a_plain_push_succeeds() {
        let (_dir, git, _start, landed, head, squash) = squashed_branch();
        let (_origin, bare) = add_origin(&git);
        // The remote copy holds the branch as it was before the landing resynced.
        git.push_branch("origin", "work", &head).unwrap();
        let previous_remote_head = bare.rev_parse("refs/heads/work").unwrap().unwrap();
        assert_eq!(previous_remote_head, head);

        let tip = resync_tip(&git, &head, &landed, &squash, false)
            .unwrap()
            .expect("the branch must move");
        if let Err(stderr) = git.reset_keep(&tip).unwrap() {
            panic!("{stderr}");
        }
        assert_eq!(git.head().unwrap().unwrap(), tip);

        // The branch descends from its previous remote head and from the host's commit.
        assert!(git.is_ancestor(&previous_remote_head, &tip).unwrap());
        assert!(git.is_ancestor(&squash, &tip).unwrap());
        // So a plain push is accepted, and the remote copy follows.
        git.push_branch("origin", "work", &tip).unwrap();
        assert_eq!(bare.rev_parse("refs/heads/work").unwrap().unwrap(), tip);

        // The rewrite this replaces is refused by the remote: its result left the remote copy's history.
        let rewritten = git.replay(&squash, &landed, &head, false).unwrap();
        assert!(!git.is_ancestor(&previous_remote_head, &rewritten).unwrap());
        assert!(git.push_branch("origin", "work", &rewritten).is_err());
    }

    #[test]
    fn resync_tip_adds_a_merge_commit_and_rewrites_nothing() {
        let (_dir, git, start, landed, head, squash) = squashed_branch();
        let tip = resync_tip(&git, &head, &landed, &squash, false)
            .unwrap()
            .expect("the branch must move");
        // A merge commit: the old tip first, the host's commit second.
        assert_eq!(git.parents(&tip).unwrap(), [head.clone(), squash.clone()]);
        // Every commit the branch held is still an ancestor, unchanged.
        let held = git.run(["rev-list", &head]).unwrap();
        for commit in held.lines() {
            assert!(git.is_ancestor(commit, &tip).unwrap(), "{commit}");
        }
        assert!(git.is_ancestor(&start, &tip).unwrap());
        // The tree is the old tip's, and the message carries no Athena- trailer.
        assert_eq!(git.tree_of(&tip).unwrap(), git.tree_of(&head).unwrap());
        let message = git.message(&tip).unwrap();
        assert!(!message.contains("Athena-"), "{message}");
        // The gate takes the merge as records only: its tree equals its first parent's.
        let judged = classify_commit(&git, &start, &tip, &tip).unwrap();
        assert_eq!(judged.kind, LandingKind::RecordsOnly);

        // The next job's pull request branch merges with the host's commit without a conflict.
        git.reset_keep(&tip).unwrap().unwrap();
        commit_message(
            &git,
            _dir.path(),
            "next.txt",
            "n\n",
            "Job C\n\nAthena-Job: jc\n",
        );
        let next = git.head().unwrap().unwrap();
        assert_eq!(merge_base(&git, &squash, &next), squash);
        assert!(git.merge_tree(&squash, &squash, &next).unwrap().is_some());
    }

    #[test]
    fn commits_of_a_landed_head_are_left_out_of_a_pull_request_range() {
        let (_dir, git, start, landed, ..) = squashed_branch();
        // Job A's commit is not a landing (it adds no closing records), so judging it is an error ...
        let judged = pr_commits(&git, &start, &landed).unwrap_err().to_string();
        assert!(!judged.contains("adds no commit"), "{judged}");
        // ... but once the head of its landed pull request is excluded, the range is empty.
        let empty = pr_commits_excluding(&git, &start, &landed, std::slice::from_ref(&landed))
            .unwrap_err()
            .to_string();
        assert!(empty.contains("adds no commit"), "{empty}");
        // A head this repository does not hold is ignored.
        let missing = "0".repeat(40);
        let unknown = pr_commits_excluding(&git, &start, &landed, &[missing])
            .unwrap_err()
            .to_string();
        assert_eq!(unknown, judged);
    }

    #[test]
    fn resync_tip_leaves_the_no_op_cases_alone() {
        let (_dir, git, start, landed, head, squash) = squashed_branch();
        let tip = resync_tip(&git, &head, &landed, &squash, false)
            .unwrap()
            .unwrap();
        // The host's commit is already part of the branch: a second run, and an older commit of the base.
        assert_eq!(
            resync_tip(&git, &tip, &landed, &squash, false).unwrap(),
            None
        );
        assert_eq!(
            resync_tip(&git, &head, &landed, &start, false).unwrap(),
            None
        );
        // The branch does not hold the job's commit.
        assert_eq!(
            resync_tip(&git, &squash, &landed, &squash, false).unwrap(),
            None
        );
        assert_eq!(
            resync_tip(&git, &start, &landed, &squash, false).unwrap(),
            None
        );
    }

    #[test]
    fn a_squash_the_later_work_conflicts_with_is_an_error_that_moves_nothing() {
        let (dir, git) = repo();
        let root = dir.path();
        let start = git.head().unwrap().unwrap();
        git.run(["checkout", "-q", "-b", "work"]).unwrap();
        let landed = commit_message(&git, root, "a.txt", "A\n", "Job A\n\nAthena-Job: ja\n");
        let head = commit_message(&git, root, "a.txt", "B\n", "Job B\n\nAthena-Job: jb\n");
        // The host's squash differs from the landed commit in the very line job B changes.
        let tree = {
            git.run(["checkout", "-q", "-b", "side", &start]).unwrap();
            commit_message(&git, root, "a.txt", "squash differs\n", "Job A (#1)\n");
            git.tree_of("HEAD").unwrap()
        };
        let squash = git
            .commit_tree(&tree, &[&start], "Job A (#1)\n", false)
            .unwrap();
        git.run(["checkout", "-q", "work"]).unwrap();
        let err = resync_tip(&git, &head, &landed, &squash, false).unwrap_err();
        assert!(err.to_string().contains("conflicts"), "{err}");
        assert_eq!(git.head().unwrap().unwrap(), head);
    }

    #[test]
    fn a_squash_with_governed_content_the_branch_lacks_is_an_error_that_moves_nothing() {
        let (dir, git, start, landed, head, _squash) = squashed_branch();
        // The host's commit also carries a governed file the branch does not hold.
        git.run(["checkout", "-q", "-b", "side", &start]).unwrap();
        std::fs::write(dir.path().join("a.txt"), "A\n2\n3\n").unwrap();
        commit_message(&git, dir.path(), "other.txt", "o\n", "Other work");
        let squash = git.head().unwrap().unwrap();
        git.run(["checkout", "-q", "work"]).unwrap();
        let err = resync_tip(&git, &head, &landed, &squash, false).unwrap_err();
        assert!(err.to_string().contains("other.txt"), "{err}");
        assert_eq!(git.head().unwrap().unwrap(), head);
    }

    #[test]
    fn a_hand_made_merge_that_takes_older_governed_content_is_not_records_only() {
        // The merge the resync makes is judged as records only; a merge that reverts governed content to an
        // older commit's version is not, so the gate still refuses it.
        let (dir, git) = repo();
        let start = git.head().unwrap().unwrap();
        git.run(["checkout", "-q", "-b", "work"]).unwrap();
        let head = commit_message(&git, dir.path(), "a.txt", "changed\n", "Edit\n");
        let reverted = git
            .commit_tree(
                &git.tree_of(&start).unwrap(),
                &[&head, &start],
                "Merge\n",
                false,
            )
            .unwrap();
        let err = classify_commit(&git, &start, &reverted, &reverted).unwrap_err();
        assert!(err.to_string().contains(&reverted), "{err}");
    }

    #[test]
    fn resync_onto_merge_reports_a_conflict_and_leaves_the_branch_unchanged() {
        let (dir, _state, athena) = crate::code_health::tests::set_up();
        let root = dir.path();
        let git = &athena.git;
        let start = git.head().unwrap().unwrap();
        git.run(["checkout", "-q", "-b", "work"]).unwrap();
        let landed = commit_message(git, root, "a.txt", "A\n", "Job A\n\nAthena-Job: ja\n");
        let head = commit_message(git, root, "a.txt", "B\n", "Job B\n\nAthena-Job: jb\n");
        git.run(["checkout", "-q", "-b", "side", &start]).unwrap();
        commit_message(git, root, "a.txt", "squash differs\n", "Job A (#1)\n");
        let squash = git.head().unwrap().unwrap();
        git.run(["checkout", "-q", "work"]).unwrap();
        let err = athena
            .resync_onto_merge(&landed, &squash, &crate::Quiet)
            .unwrap_err();
        assert!(
            format!("{err:#}").contains("the branch is unchanged"),
            "{err:#}"
        );
        assert_eq!(athena.head().unwrap(), head);
    }

    #[test]
    fn a_verified_candidate_lands_on_its_base() {
        // M7 (allows): a verified candidate lands.
        let (_dir, git, p, c, f) = base_and_candidate();
        assert_eq!(
            plan(&git, &pass(&p, &c, &f), &p, &Dispatched::Unknown).unwrap(),
            Plan::Tree(git.tree_of(&c).unwrap())
        );
    }

    #[test]
    fn a_job_lands_when_the_dispatch_branch_is_checked_out() {
        // The allowing side of the branch rule: on the dispatch branch the job lands, with the head at
        // the base and after a commit that changed only an excluded path (the replay case).
        let (dir, git, p, c, f) = base_and_candidate();
        let main = Dispatched::Branch("main".into());
        assert_eq!(
            plan(&git, &pass(&p, &c, &f), &p, &main).unwrap(),
            Plan::Tree(git.tree_of(&c).unwrap())
        );
        let head = commit(&git, dir.path(), "PRIVATE.md", "private\n");
        let Plan::Tree(tree) = plan(&git, &pass(&p, &c, &f), &head, &main).unwrap() else {
            panic!("the job must land over the excluded change")
        };
        assert_eq!(fingerprint(&git, &tree, &p).unwrap(), f);
        // The replay kept the excluded file next to the candidate's change.
        let at = |path: &str| {
            git.run(["cat-file", "-p", &format!("{tree}:{path}")])
                .unwrap()
        };
        assert_eq!(at("PRIVATE.md"), "private\n");
        assert_eq!(at("a.txt"), "changed\n");
    }

    #[test]
    fn a_job_dispatched_elsewhere_is_refused_on_another_branch() {
        // The base is an ancestor of the other branch's tip, so only the branch rule refuses.
        let (_dir, git, p, c, f) = base_and_candidate();
        git.run(["checkout", "-q", "-b", "other"]).unwrap();
        assert!(git.is_ancestor(&p, &git.head().unwrap().unwrap()).unwrap());
        let Plan::OtherBranch(reason) = plan(
            &git,
            &pass(&p, &c, &f),
            &p,
            &Dispatched::Branch("main".into()),
        )
        .unwrap() else {
            panic!("the job must not land on another branch")
        };
        assert!(
            reason.contains("`main`") && reason.contains("`other`"),
            "{reason}"
        );
        // The same call without the recorded branch lands, which is what the rule changes.
        assert!(matches!(
            plan(&git, &pass(&p, &c, &f), &p, &Dispatched::Unknown).unwrap(),
            Plan::Tree(_)
        ));
    }

    #[test]
    fn the_branch_rule_is_checked_before_the_ancestor_rule() {
        let (dir, git, p, c, f) = base_and_candidate();
        git.run(["checkout", "-q", "--orphan", "rewritten"])
            .unwrap();
        let head = commit(&git, dir.path(), "b.txt", "unrelated\n");
        let decided = plan(
            &git,
            &pass(&p, &c, &f),
            &head,
            &Dispatched::Branch("main".into()),
        )
        .unwrap();
        assert!(matches!(decided, Plan::OtherBranch(_)), "{decided:?}");
    }

    #[test]
    fn a_working_copy_on_a_branch_passes_the_precondition() {
        let (_dir, git, ..) = base_and_candidate();
        for expected in [
            Dispatched::Branch("main".into()),
            Dispatched::Detached,
            Dispatched::Unknown,
        ] {
            refuse_detached(&git, &expected).unwrap();
        }
    }

    #[test]
    fn a_detached_head_is_refused_naming_the_commit_and_the_branch() {
        let (_dir, git, p, ..) = base_and_candidate();
        git.run(["checkout", "-q", "--detach"]).unwrap();
        let error = refuse_detached(&git, &Dispatched::Branch("main".into())).unwrap_err();
        let text = format!("{error:#}");
        assert!(text.contains("detached"), "{text}");
        assert!(text.contains(&p[..12]), "{text}");
        assert!(text.contains("`main`"), "{text}");
        assert!(text.contains("nothing landed"), "{text}");
        assert!(!text.contains("verification"), "{text}");
    }

    #[test]
    fn a_detached_head_is_refused_when_the_dispatch_branch_is_not_known() {
        let (_dir, git, p, ..) = base_and_candidate();
        git.run(["checkout", "-q", "--detach"]).unwrap();
        for expected in [Dispatched::Detached, Dispatched::Unknown] {
            let text = format!("{:#}", refuse_detached(&git, &expected).unwrap_err());
            assert!(
                text.contains("detached") && text.contains(&p[..12]),
                "{text}"
            );
            assert!(text.contains("a checked-out branch"), "{text}");
            assert!(!text.contains("verification"), "{text}");
        }
    }

    #[test]
    fn a_job_with_no_record_anywhere_has_an_unknown_dispatch_branch() {
        let (_dir, git, p, ..) = base_and_candidate();
        let id = JobId::parse("20261010-120000-abcd").unwrap();
        assert_eq!(dispatched_from(&git, &p, &id).unwrap(), Dispatched::Unknown);
    }

    #[test]
    fn a_job_dispatched_on_a_detached_head_never_lands() {
        let (_dir, git, p, c, f) = base_and_candidate();
        let Plan::OtherBranch(reason) =
            plan(&git, &pass(&p, &c, &f), &p, &Dispatched::Detached).unwrap()
        else {
            panic!("a detached dispatch names no branch to land on")
        };
        assert!(reason.contains("detached HEAD"), "{reason}");
    }

    #[test]
    fn a_branch_job_finished_on_a_detached_head_is_refused() {
        let (_dir, git, p, c, f) = base_and_candidate();
        git.run(["checkout", "-q", "--detach"]).unwrap();
        let Plan::OtherBranch(reason) = plan(
            &git,
            &pass(&p, &c, &f),
            &p,
            &Dispatched::Branch("main".into()),
        )
        .unwrap() else {
            panic!("a detached HEAD is not the dispatch branch")
        };
        assert!(
            reason.contains("`main`") && reason.contains("detached"),
            "{reason}"
        );
    }

    #[test]
    fn the_dispatch_branch_is_read_from_the_record() {
        assert_eq!(Dispatched::from_record(None), Dispatched::Unknown);
        assert_eq!(Dispatched::from_record(Some("")), Dispatched::Detached);
        assert_eq!(
            Dispatched::from_record(Some("main")),
            Dispatched::Branch("main".into())
        );
    }

    #[test]
    fn landing_refuses_content_that_differs_from_the_verified_content() {
        // M7 (blocks): landing after the candidate changed.
        let (_dir, git, p, c, _f) = base_and_candidate();
        let wrong = fingerprint(&git, &p, &p).unwrap();
        assert!(plan(&git, &pass(&p, &c, &wrong), &p, &Dispatched::Unknown).is_err());
    }

    #[test]
    fn a_base_that_is_no_longer_an_ancestor_makes_the_job_stale() {
        // M7 (blocks): landing when the base is no longer an ancestor.
        let (dir, git, p, c, f) = base_and_candidate();
        git.run(["checkout", "-q", "--orphan", "rewritten"])
            .unwrap();
        let rewritten = commit(&git, dir.path(), "b.txt", "history was rewritten\n");
        assert!(matches!(
            plan(&git, &pass(&p, &c, &f), &rewritten, &Dispatched::Unknown).unwrap(),
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
            plan(&git, &pass(&p, &c, &f), &head, &Dispatched::Unknown).unwrap(),
            Plan::Stale(_)
        ));
    }

    // ---- pr_commits: classification by content, with no trailers anywhere ----

    const PR_JOB: &str = "20261024-115923-02gf";

    /// A repository with `owner/` excluded by a nested rule and `lacking/` carrying a rules file that
    /// excludes nothing there; returns the base commit the pull request is cut from.
    fn pr_repo() -> (tempfile::TempDir, Git, String) {
        let (dir, git) = repo();
        fixtures::commit(
            &git,
            &[
                plain("owner/.gitattributes", "* athena-exclude\n"),
                plain("lacking/.gitattributes", "*.tmp athena-exclude\n"),
            ],
            "rules",
        );
        let base = git.head().unwrap().unwrap();
        (dir, git, base)
    }

    fn kinds(commits: &[PrCommit]) -> Vec<LandingKind> {
        commits.iter().map(|c| c.kind).collect()
    }

    /// The error `pr_commits` gives for `base..HEAD`; it must name `sha`.
    fn refusal(git: &Git, base: &str, sha: &str) -> String {
        let head = git.head().unwrap().unwrap();
        let error = pr_commits(git, base, &head).unwrap_err().to_string();
        assert!(error.contains(sha), "{error}");
        error
    }

    #[test]
    fn a_realistic_range_is_classified_from_content_alone() {
        let (_dir, git, base) = pr_repo();
        let job = JobId::parse(PR_JOB).unwrap();
        let job_dir = job.dir();
        let rid = |text: &str| athena_data::RecordId::parse(text).unwrap();
        let mut shas = vec![
            fixtures::commit(
                &git,
                &[plain(&format!("{job_dir}/reading-1.json"), "{}\n")],
                "Reading",
            ),
            fixtures::commit(
                &git,
                &[plain(&format!("{job_dir}/job.json"), "{}\n")],
                "Dispatch",
            ),
            fixtures::commit(
                &git,
                &[plain(
                    "records/introspections/2026-10/20261024-120000-abcd.json",
                    "{}\n",
                )],
                "Introspection",
            ),
            fixtures::commit(&git, &[plain("owner/notes.md", "mine\n")], "Owner path"),
        ];
        let intervention = rid("20261024-121000-abcd");
        shas.push(fixtures::commit(
            &git,
            &[
                plain("a.txt", "by hand\n"),
                plain(
                    &format!(
                        "records/interventions/{}/{intervention}.json",
                        intervention.month()
                    ),
                    "{}\n",
                ),
            ],
            "Intervention",
        ));
        let mut landing = fixtures::landed(&job);
        landing.push(plain("a.txt", "landed by a job\n"));
        shas.push(fixtures::commit(&git, &landing, "Landing"));
        let head = git.head().unwrap().unwrap();

        // The landing's records really parse, so the classification below is not an accident.
        let files = athena_gate::jobs::JobFiles::load(&git, &head, &job).unwrap();
        assert!(
            files.candidate.is_some() && files.verification.is_some() && files.outcome.is_some()
        );

        let commits = pr_commits(&git, &base, &head).unwrap();
        assert_eq!(
            commits.iter().map(|c| c.sha.clone()).collect::<Vec<_>>(),
            shas
        );
        assert_eq!(
            kinds(&commits),
            [
                LandingKind::RecordsOnly,
                LandingKind::RecordsOnly,
                LandingKind::RecordsOnly,
                LandingKind::RecordsOnly,
                LandingKind::DeclaredIntervention,
                LandingKind::JobLanding,
            ]
        );
        for c in &commits[..5] {
            assert_eq!((&c.job, &c.verification), (&None, &None), "{}", c.sha);
        }
        let last = &commits[5];
        assert_eq!(last.job.as_ref(), Some(&job));
        assert_eq!(last.verification.as_ref(), Some(&rid(PR_JOB)));
        assert_eq!(last.tree, git.tree_of(&last.sha).unwrap());
    }

    #[test]
    fn a_blocked_outcome_is_records_only_and_the_later_landed_outcome_is_the_landing() {
        let (_dir, git, base) = pr_repo();
        let job = JobId::parse(PR_JOB).unwrap();
        let blocked = fixtures::commit(
            &git,
            &[
                fixtures::candidate(&job),
                fixtures::verification(&job),
                fixtures::outcome(&job, OutcomeState::Blocked, false),
            ],
            "Blocked",
        );
        let landed = fixtures::commit(
            &git,
            &[
                fixtures::outcome(&job, OutcomeState::Landed, true),
                plain("a.txt", "landed on retry\n"),
            ],
            "Retry",
        );
        let commits = pr_commits(&git, &base, &landed).unwrap();
        assert_eq!(
            kinds(&commits),
            [LandingKind::RecordsOnly, LandingKind::JobLanding]
        );
        assert_eq!(commits[0].sha, blocked);
        assert_eq!((&commits[0].job, &commits[0].verification), (&None, &None));
        assert_eq!(commits[1].job.as_ref(), Some(&job));
        assert!(commits[1].verification.is_some());
    }

    #[test]
    fn governed_change_that_is_neither_an_intervention_nor_closing_records_is_refused() {
        let (_dir, git, base) = pr_repo();
        let bad = fixtures::commit(&git, &[plain("a.txt", "hand edit\n")], "Hand edit");
        let error = refusal(&git, &base, &bad);
        assert!(error.contains("changes governed content"), "{error}");
    }

    #[test]
    fn a_landing_without_a_verification_is_refused() {
        let (_dir, git, base) = pr_repo();
        let job = JobId::parse(PR_JOB).unwrap();
        let bad = fixtures::commit(
            &git,
            &[
                fixtures::candidate(&job),
                fixtures::outcome(&job, OutcomeState::Landed, false),
                plain("a.txt", "landed, unverified\n"),
            ],
            "Landing",
        );
        let error = refusal(&git, &base, &bad);
        assert!(error.contains("without its verification"), "{error}");
    }

    #[test]
    fn a_landing_whose_outcome_is_blocked_is_refused_not_counted() {
        let (_dir, git, base) = pr_repo();
        let job = JobId::parse(PR_JOB).unwrap();
        let bad = fixtures::commit(
            &git,
            &[
                fixtures::candidate(&job),
                fixtures::verification(&job),
                fixtures::outcome(&job, OutcomeState::Blocked, false),
                plain("a.txt", "work that never landed\n"),
            ],
            "Blocked with work",
        );
        let error = refusal(&git, &base, &bad);
        assert!(error.contains("changes governed content"), "{error}");
    }

    #[test]
    fn an_owner_path_without_the_exclusion_is_refused() {
        let (_dir, git, base) = pr_repo();
        fixtures::commit(&git, &[plain("owner/notes.md", "excluded\n")], "Excluded");
        let bad = fixtures::commit(
            &git,
            &[plain("lacking/notes.md", "not excluded\n")],
            "Not excluded",
        );
        refusal(&git, &base, &bad);
    }

    #[test]
    fn an_empty_range_is_an_error() {
        let (_dir, git, base) = pr_repo();
        let error = pr_commits(&git, &base, &base).unwrap_err().to_string();
        assert!(error.contains("nothing to record"), "{error}");
    }
}
