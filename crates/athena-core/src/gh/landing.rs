//! The landing row: the work branch, the landed commit, the pull request flow of `land_pr`, and
//! `Athena::land_pr` with what follows a merge.

use super::early::{replace_early, wait_for_checks};
use super::{
    Checks, GhClient, GitLandingRecords, GitRemote, HostErrorLog, LandingFacts, LandingRecords,
    NewPr, PAGES_WAIT_DELAY, PAGES_WAIT_LOOKS, PagesHost, PrDraft, PrHost, Remote, Runner,
    cutover_job_for, parse_remote, watch_pages,
};
use crate::pr::{
    Ancestry, Content, GitContent, Host, LinkState, PrLink, PushRequest, check_push, clear_early,
    load_early, load_link, load_link_or_adopt_early, observe_merge, store_link,
};
use crate::state::StateDir;
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_data::JobId;
use athena_data::records::{Verdict, WorkPhase};
use athena_git::{CommitText, Git};
use serde::Serialize;
use std::time::Duration;

/// The branch Athena pushes for a job: the only one the push guard allows.
pub fn work_branch(job: &JobId) -> String {
    format!("athena/{}", job.as_str())
}

/// The commit pushed as the work branch: the job's locally landed commit. It is found from the job's landing
/// record: the newest commit reachable from `head` that carries the job's `Athena-Job` trailer and the
/// verified content's `Athena-Verified-Content` trailer. It holds the verified change, the trailers and the
/// records. A job that is not landed locally has no such commit, and that is an error: nothing is pushed.
pub fn landed_commit(git: &Git, head: &str, job: &JobId, fingerprint: &str) -> Result<String> {
    let candidates = git.log_grep(head, &format!("^Athena-Verified-Content: {fingerprint}$"))?;
    for commit in candidates {
        let trailers = git.trailers(&git.message(&commit)?)?;
        let for_job = trailers
            .iter()
            .any(|(k, v)| k == "Athena-Job" && v == job.as_str());
        let verified = trailers
            .iter()
            .any(|(k, v)| k == "Athena-Verified-Content" && v == fingerprint);
        if for_job && verified {
            return Ok(commit);
        }
    }
    bail!(
        "job {job} is not landed on this branch, so there is no landed commit to push; run `athena go {job}` first"
    )
}

/// [`landed_commit`] for a job landed after C0 (D111), whose commits carry no trailers. The commit is found
/// from the job's records: the one that introduced `outcome-landed.json` (a retried landing), else
/// `outcome.json`, and only when it is reachable from `head` and its `verification.json` holds the
/// verified fingerprint.
pub fn landed_commit_from_records(
    git: &Git,
    head: &str,
    job: &JobId,
    fingerprint: &str,
) -> Result<String> {
    for name in ["outcome-landed.json", "outcome.json"] {
        let path = format!("{}/{name}", job.dir());
        let out = git.run(["log", "--format=%H", "--diff-filter=A", head, "--", &path])?;
        let Some(commit) = out.lines().last() else {
            continue;
        };
        let verification = git.show_file(commit, &format!("{}/verification.json", job.dir()))?;
        if verification.is_some_and(|b| String::from_utf8_lossy(&b).contains(fingerprint)) {
            return Ok(commit.to_string());
        }
    }
    bail!(
        "job {job} is not landed on this branch, so there is no landed commit to push; run `athena go {job}` first"
    )
}

/// The landed commit of `job`: by trailers before C0, by the job's records after it.
pub fn landed_commit_for(git: &Git, head: &str, job: &JobId, fingerprint: &str) -> Result<String> {
    if cutover_job_for(git, head).is_some() {
        landed_commit_from_records(git, head, job, fingerprint)
    } else {
        landed_commit(git, head, job, fingerprint)
    }
}

/// The squash text of `commit`: strict before C0, relaxed after it (no trailers are written).
pub fn squash_text_for(git: &Git, head: &str, commit: &str) -> Result<CommitText> {
    Ok(if cutover_job_for(git, head).is_some() {
        git.squash_text_relaxed(commit)?
    } else {
        git.squash_text(commit)?
    })
}

/// How long to wait for the checks.
pub struct Poll<'a> {
    /// How many times to ask, at least once.
    pub attempts: u32,
    pub delay: Duration,
    pub sleep: &'a dyn Fn(Duration),
}

impl std::fmt::Debug for Poll<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Poll")
            .field("attempts", &self.attempts)
            .field("delay", &self.delay)
            .finish_non_exhaustive()
    }
}

/// Everything `land_pr` needs for one job.
pub struct LandPr<'a> {
    pub state: &'a StateDir,
    pub job: &'a JobId,
    /// The branch the caller asks to push; the guard allows only [`work_branch`].
    pub branch: &'a str,
    /// The commit to push as the pull request's head: the job's locally landed commit (see [`landed_commit`]).
    pub commit: &'a str,
    pub verified_fingerprint: &'a str,
    /// The pull request's text. Its base is the host's default branch, read when the request is opened.
    pub pr: PrDraft<'a>,
    /// The message a squash merge carries: the job's title, and its trailers in the last paragraph.
    pub squash: &'a CommitText,
    pub host_name: &'a str,
    pub repository: &'a str,
    /// The `[landing] merge_via_rest` setting.
    pub merge_via_rest: bool,
    pub poll: Poll<'a>,
    pub ancestry: &'a dyn Ancestry,
    pub remote: &'a dyn Remote,
    pub host: &'a dyn PrHost,
    pub content: &'a dyn Content,
    /// Where the landing-intent and landing records are written (§5.2, §5.3 of
    /// `docs/design/records-history.md`). `Athena::land_pr` always passes one, built by
    /// [`GitLandingRecords::for_remote`]; `None` skips the records and is for tests of the rest of the flow.
    pub records: Option<&'a dyn LandingRecords>,
}

impl std::fmt::Debug for LandPr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LandPr")
            .field("job", &self.job)
            .field("branch", &self.branch)
            .field("repository", &self.repository)
            .field("merge_via_rest", &self.merge_via_rest)
            .finish_non_exhaustive()
    }
}

/// What `land_pr` found or did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrReport {
    pub link: PrLink,
    /// The checks, when the pull request was still open and they were read.
    pub checks: Option<Checks>,
    /// Whether this run asked the host to merge.
    pub merge_requested: bool,
    /// How many times the failed jobs were re-run: 0 or 1.
    pub reruns: u32,
    /// The first failing test's name, read from the failed job's log, when the checks were red.
    pub failing_test: Option<String>,
    /// Things the caller must be told that are not failures: a merge whose landing record was not written,
    /// naming the landing-intent record that reconciliation will use (§8.1).
    pub warnings: Vec<String>,
}

/// Runs the flow. With no stored link: guard and push the branch, open the pull request, store the link.
/// Then, while the pull request is open: look for a merge, else read the checks, and merge over REST only if
/// the setting is on and the checks are green. Checks are green only when no check run on the head is still
/// queued or in progress. A merge the host refuses with "Repository rule violations" while the checks are
/// pending again is a wait: the report has `checks` pending, `merge_requested` set and a warning, and it is
/// not an error. The same refusal with the checks settled stays an error. The client logs that refusal with the
/// pending mark on the merge call's own entry, so the watcher does not read it as a refused merge method.
pub fn land_pr(job: &LandPr) -> Result<PrReport> {
    // The commit an early push left on the branch at verification start, if any.
    let early = load_early(job.state, job.job)?;
    let mut warnings = Vec::new();
    let mut link = match load_link_or_adopt_early(job.state, job.job)? {
        Some(link) => link,
        None => {
            let (link, note) = publish(job, early.as_deref())?;
            warnings.extend(note);
            link
        }
    };
    let mut checks = None;
    let mut merge_requested = false;
    let mut reruns = 0;
    let mut failing_test = None;
    observe_merge(job.host, job.content, &mut link, job.verified_fingerprint)?;
    if link.state == LinkState::Open {
        // The pull request's head is whatever its branch holds now, not the job's commit: a fix may have
        // been pushed since. Everything below judges and guards that head, and nothing replaces it.
        let Some(head) = job.remote.remote_head(job.branch)? else {
            bail!(
                "the pull request's branch {} is not on the remote, so its head cannot be judged",
                job.branch
            );
        };
        // The branch still holds the job's own early push: the landing commit replaces it, pinned to that
        // head, and the pull request opened early is kept. Any other head is judged below as before.
        let head = match early.as_deref() {
            Some(early_head) if head == early_head && head != job.commit => {
                warnings.push(replace_early(job, early_head)?);
                job.commit.to_string()
            }
            _ => head,
        };
        if early.is_some() && head == job.commit {
            clear_early(job.state, job.job)?;
        }
        link.head = head.clone();
        store_link(job.state, job.job, &link)?;
        if head != job.commit {
            let seen = job.host.checks(&head);
            let extras = job
                .remote
                .commits_after(job.commit, &head)
                .unwrap_or_default();
            let text = refusal_text(link.pr_number, job.commit, &head, &seen, &extras);
            bail!("{text}");
        }
    }
    if link.state == LinkState::Open {
        let waited = wait_for_checks(job, &link.head)?;
        let seen = waited.checks;
        reruns = waited.reruns;
        failing_test = waited.failing_test;
        warnings.extend(waited.notes);
        checks = Some(seen);
        // The merge gate: the setting is on and every check is green. Nothing else reaches the host's
        // merge endpoint.
        if job.merge_via_rest && seen == Checks::Green {
            // The landing-intent record is on the records branch of the remote before the host is asked to
            // merge (§5.2). An error here, such as a commit nothing verified, stops the landing before
            // anything is merged.
            let intent = match job.records {
                Some(records) => {
                    let base = records.base_tip(&job.host.default_branch()?)?;
                    let id = records.intent(link.pr_number, &link.head, &base)?;
                    Some((records, base, id))
                }
                None => None,
            };
            let method = match job.host.merge(link.pr_number, &link.head, job.squash) {
                Ok(method) => {
                    merge_requested = true;
                    Some(method)
                }
                // The host refused the merge by its repository rules. While a check is still running that
                // is a wait, not a failure: the merge is possible once every check has finished. Any other
                // refusal, or one with the checks settled, is returned as it came.
                Err(refusal) if is_rule_violation(&refusal) => {
                    if job.host.check_report(&link.head)?.checks != Checks::Pending {
                        return Err(refusal);
                    }
                    let record = intent
                        .as_ref()
                        .map(|(_, _, id)| {
                            format!("; landing-intent record {id} is on the records branch")
                        })
                        .unwrap_or_default();
                    warnings.push(format!(
                        "the host refused the merge by its repository rules while checks were not finished \
                         ({refusal:#}); waiting to merge{record}"
                    ));
                    checks = Some(Checks::Pending);
                    merge_requested = true;
                    None
                }
                Err(other) => return Err(other),
            };
            if let Some(method) = method {
                observe_merge(job.host, job.content, &mut link, job.verified_fingerprint)?;
                if let Some((records, expected_base, intent)) = intent {
                    match (&link.state, &link.merged_commit) {
                    (LinkState::Landed, Some(host_commit)) => {
                        let facts = LandingFacts {
                            intent: intent.clone(),
                            expected_base,
                            pr_number: link.pr_number,
                            head: link.head.clone(),
                            host_commit: host_commit.clone(),
                            merge_method: method,
                        };
                        // The merge is done and stays done: a landing record that cannot be written is a
                        // warning, and reconciliation (§8.1) writes it from the intent.
                        match records.landing(&facts) {
                            Err(problem) => warnings.push(format!(
                                "the pull request was merged but its landing record could not be written \
                                 ({problem:#}); landing-intent record {intent} is on the records branch for reconciliation"
                            )),
                            // The cutover runs after the landing record, and a failure of it is a warning
                            // too: the merge and the record stay done, and the reconcile step of the next
                            // `athena land` or `athena status` finishes the import.
                            Ok(_) => match records.cutover(&facts.host_commit) {
                                Ok(Some(line)) => warnings.push(line),
                                Ok(None) => {}
                                Err(problem) => warnings.push(format!(
                                    "the cutover import did not finish ({problem:#}); the next `athena land` \
                                     or `athena status` tries it again"
                                )),
                            },
                        }
                    }
                    _ => warnings.push(format!(
                        "a merge was requested but not confirmed, so no landing record was written; \
                         landing-intent record {intent} is on the records branch for reconciliation"
                    )),
                }
                }
            }
        }
    }
    store_link(job.state, job.job, &link)?;
    Ok(PrReport {
        link,
        checks,
        merge_requested,
        reruns,
        failing_test,
        warnings,
    })
}

/// Whether the host refused a merge by its repository rules (GitHub: "Repository rule violations found").
/// The client uses the same test to mark such a refusal in the host-error log when checks were pending.
pub(super) fn is_rule_violation(refusal: &anyhow::Error) -> bool {
    format!("{refusal:#}").contains("Repository rule violations")
}

/// For a job whose landed commit is not on the current branch: when the job's pull request link already
/// exists, asks the host whether it merged and runs the merged-content check. Returns the report when the
/// link is no longer open (landed, or merged but not verified), and stores it. Returns `None` when there is
/// no link or the pull request is still open, so the caller keeps its "not landed" error. Nothing is pushed.
pub fn confirm_existing_merge(
    state: &StateDir,
    job: &JobId,
    host: &dyn Host,
    content: &dyn Content,
    verified_fingerprint: &str,
) -> Result<Option<PrReport>> {
    let Some(mut link) = load_link(state, job)? else {
        return Ok(None);
    };
    observe_merge(host, content, &mut link, verified_fingerprint)?;
    if link.state == LinkState::Open {
        return Ok(None);
    }
    store_link(state, job, &link)?;
    Ok(Some(PrReport {
        link,
        checks: None,
        merge_requested: false,
        reruns: 0,
        failing_test: None,
        warnings: Vec::new(),
    }))
}

/// The refusal for a pull request whose head is not the job's landing commit. It names the head, how its
/// checks stand (one look, so the old commit's result is never what is reported), each extra commit, and why
/// the head is refused. Nothing is merged or pushed.
fn refusal_text(
    number: u64,
    landed: &str,
    head: &str,
    checks: &Result<Checks>,
    extras: &[(String, String)],
) -> String {
    let short = |sha: &str| sha.chars().take(12).collect::<String>();
    let standing = match checks {
        Ok(Checks::Green) => "green".to_string(),
        Ok(Checks::Red) => "red".to_string(),
        Ok(Checks::Pending) => "pending".to_string(),
        Err(e) => format!("unreadable ({e:#})"),
    };
    let mut text = format!(
        "pull request #{number} is not merged: its current head {} is not the job's landing commit {}. \
         The checks of that head are {standing}.",
        short(head),
        short(landed)
    );
    if extras.is_empty() {
        text.push_str(&format!(
            "\nRefused head: {} (it adds no commit after the landing commit, so it is older than it or \
             unrelated to it).",
            short(head)
        ));
    } else {
        text.push_str("\nExtra commits after the landing commit:");
        for (sha, subject) in extras {
            text.push_str(&format!("\n  {} {subject}", short(sha)));
        }
    }
    text.push_str(
        "\nWhy it is refused: until the records-history build (D111) lands, a squash of a job's landing \
         plus any later commit is not a valid landing (it breaks the history check). This holds for a \
         declared intervention too. Athena did not merge it and did not replace the head with the job's \
         commit.",
    );
    text
}

fn publish(job: &LandPr, early: Option<&str>) -> Result<(PrLink, Option<String>)> {
    let allowed = work_branch(job.job);
    let commit = job.commit.to_string();
    let remote_head = job.remote.remote_head(job.branch)?;
    let mut note = None;
    let base = match (early, remote_head.as_deref()) {
        // An early push reached the branch but its pull request was not opened: replace that head.
        (Some(early_head), Some(head)) if head == early_head && head != commit => {
            let base = job.host.default_branch()?;
            note = Some(replace_early(job, early_head)?);
            base
        }
        _ => {
            check_push(
                job.ancestry,
                &PushRequest {
                    allowed_branch: &allowed,
                    branch: job.branch,
                    local_head: &commit,
                    remote_head: remote_head.as_deref(),
                },
            )?;
            let base = job.host.default_branch()?;
            job.remote.push(job.branch, &commit)?;
            base
        }
    };
    let pr_number = job.host.open_pr(&NewPr {
        title: job.pr.title,
        body: job.pr.body,
        head: job.pr.head,
        base: &base,
    })?;
    let link = PrLink {
        pr_number,
        host: job.host_name.to_string(),
        repository: job.repository.to_string(),
        head: commit,
        job: job.job.as_str().to_string(),
        state: LinkState::Open,
        merged_commit: None,
    };
    store_link(job.state, job.job, &link)?;
    Ok((link, note))
}

/// The start of the message line that marks a commit kept as records only (see [`plan_rebase`]).
const RECORDS_ONLY_MARKER: &str = "Athena-Records-Only: ";

/// Whether `message` has a line that marks its commit as kept as records only.
fn has_records_only_marker(message: &str) -> bool {
    message
        .lines()
        .any(|line| line.starts_with(RECORDS_ONLY_MARKER))
}

/// The result of rebasing a job's landing commit onto the latest main (see [`plan_rebase`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rebase {
    /// Main is already part of the landing commit's history, or the landing commit is already part of
    /// main's: nothing moves.
    Current,
    /// The stack was replayed onto `onto`. `tip` is the new head of the whole stack, `landed` the new hash of
    /// the job's own landing commit, and `records_only` the old hashes of earlier commits that conflicted and
    /// were kept as records only.
    Moved {
        onto: String,
        landed: String,
        tip: String,
        records_only: Vec<String>,
    },
    /// The job is held back: its own landing commit conflicts with main (or is itself a records-only commit,
    /// so its code is not on the branch). Nothing is landed and nothing moves. `commit` is the commit at
    /// fault.
    Conflict { commit: String, text: String },
    /// The rebase could not be judged (main could not be fetched, the work branch is already published, or no
    /// landing commit was found). Nothing moves, and this is not a statement about the job.
    Unchecked(String),
}

/// The message a commit kept as records only carries: the old subject and body without the `Athena-*`
/// trailers (so the pre-C0 lookup by `Athena-Job` cannot select it), a line saying the code was held back, and
/// `Athena-Records-Only: <old commit>`. A message that already has the marker (a second rebase) keeps it and
/// gets no second one.
fn records_only_message(git: &Git, commit: &str) -> Result<String> {
    let message = git.message(commit)?;
    let marked = has_records_only_marker(&message);
    let kept: Vec<&str> = message
        .lines()
        .filter(|line| line.starts_with(RECORDS_ONLY_MARKER) || !line.starts_with("Athena-"))
        .collect();
    let mut text = kept.join("\n").trim_end().to_string();
    if !marked {
        text.push_str(&format!(
            "\n\nThe code of this commit was held back by a rebase conflict; only its records are kept.\n\n{RECORDS_ONLY_MARKER}{commit}"
        ));
    }
    text.push('\n');
    Ok(text)
}

/// Plans the rebase of the stack of landing commits ending at `head` onto `onto` (origin's main), for the
/// job whose own landing commit is `landed`. Nothing is moved: the new tip is returned.
///
/// Landing commits are stacked on one local branch, so the first-parent commits between the fork with `onto`
/// and `head` are replayed oldest first with a three-way merge each, keeping their messages. Then:
///
/// - A `landed` commit that is itself a records-only commit has no code to land: [`Rebase::Conflict`].
/// - `onto` is already an ancestor of `landed` (or `landed` of `onto`): [`Rebase::Current`].
/// - A conflict in `landed` itself: [`Rebase::Conflict`]. The job is held back.
/// - A conflict in any other commit: that commit is kept as records only. Its `records/` paths are carried
///   onto the new tip and its code paths are dropped, so no job's records leave the branch. A conflict in
///   that records-only merge is a [`Rebase::Conflict`] naming the commit.
///
/// Commits that replay cleanly, including earlier unlanded ones, are carried as they are. A replayed result
/// may hold governed content that differs from what was verified: deciding whether to re-check is for the
/// caller (build job 7); this function does not.
pub fn plan_rebase(git: &Git, onto: &str, head: &str, landed: &str, sign: bool) -> Result<Rebase> {
    if has_records_only_marker(&git.message(landed)?) {
        return Ok(Rebase::Conflict {
            commit: landed.to_string(),
            text: format!(
                "commit {landed} holds only the records of its job (its code was held back by an \
                 earlier rebase conflict), so there is no code to land"
            ),
        });
    }
    if git.is_ancestor(onto, landed)? || git.is_ancestor(landed, onto)? {
        return Ok(Rebase::Current);
    }
    if !git.is_ancestor(landed, head)? {
        bail!("the landing commit {landed} is not part of the branch head {head}");
    }
    let fork = git.run(["merge-base", onto, head])?.trim().to_string();
    let commits = git.run([
        "rev-list",
        "--reverse",
        "--first-parent",
        &format!("{fork}..{head}"),
    ])?;
    let conflict_text = |commit: &str| -> Result<String> {
        let subject = git.run(["log", "-1", "--format=%s", commit])?;
        Ok(format!(
            "commit {commit} (\"{}\") conflicts with {onto}",
            subject.trim()
        ))
    };
    let mut tip = onto.to_string();
    let mut new_landed = None;
    let mut records_only = Vec::new();
    for commit in commits.lines() {
        let parent =
            git.parents(commit)?.into_iter().next().with_context(|| {
                format!("commit {commit} has no parent, so it cannot be replayed")
            })?;
        match git.merge_tree(&parent, &tip, commit)? {
            Some(tree) => {
                if tree != git.tree_of(&tip)? {
                    // `%B` ends with a blank line; one newline keeps the message as it was written.
                    let message = format!("{}\n", git.message(commit)?.trim_end());
                    tip = git.commit_tree(&tree, &[&tip], &message, sign)?;
                }
            }
            None if commit == landed => {
                return Ok(Rebase::Conflict {
                    commit: commit.to_string(),
                    text: conflict_text(commit)?,
                });
            }
            None => {
                let note = records_only_message(git, commit)?;
                let image = git.commit_with_records(&parent, commit, &note, sign)?;
                if image != parent {
                    let Some(tree) = git.merge_tree(&parent, &tip, &image)? else {
                        return Ok(Rebase::Conflict {
                            commit: commit.to_string(),
                            text: format!(
                                "the records of an earlier commit conflict: {}",
                                conflict_text(commit)?
                            ),
                        });
                    };
                    if tree != git.tree_of(&tip)? {
                        tip = git.commit_tree(&tree, &[&tip], &note, sign)?;
                    }
                }
                records_only.push(commit.to_string());
            }
        }
        if commit == landed {
            new_landed = Some(tip.clone());
        }
    }
    let Some(new_landed) = new_landed else {
        bail!("the landing commit {landed} is not on the first-parent chain of {head}");
    };
    Ok(Rebase::Moved {
        onto: onto.to_string(),
        landed: new_landed,
        tip,
        records_only,
    })
}

/// Rebases the job's landing commit, with the stack under and over it, onto the latest `default_branch` of
/// `remote_name`, and moves the local branch to the result. It never pushes. Commits are signed when the
/// repository's configuration says so.
///
/// It is [`Rebase::Unchecked`], with nothing moved, when the job's work branch on the remote holds anything
/// but the job's own early push (a published head is not rewritten here), when main cannot be fetched, or
/// when the job has no landing commit on the branch. Otherwise it takes the repository lock, runs
/// [`plan_rebase`] and, for [`Rebase::Moved`], moves the branch with a keep-edits reset; a reset git refuses is
/// an error and the branch stays as it was.
pub fn rebase_job_branch(
    git: &Git,
    state: &StateDir,
    remote_name: &str,
    default_branch: &str,
    job: &JobId,
    verified_fingerprint: &str,
    ui: &dyn Ui,
) -> Result<Rebase> {
    let remote = GitRemote {
        git,
        name: remote_name,
    };
    let published = match remote.remote_head(&work_branch(job)) {
        Ok(published) => published,
        Err(problem) => {
            return Ok(Rebase::Unchecked(format!(
                "could not read the work branch on {remote_name}: {problem:#}"
            )));
        }
    };
    if let Some(published) = published
        && load_early(state, job)?.as_deref() != Some(published.as_str())
    {
        return Ok(Rebase::Unchecked(format!(
            "the work branch {} is already published at {published}; it is not rewritten",
            work_branch(job)
        )));
    }
    let onto = match git.fetch_remote_branch(remote_name, default_branch) {
        Ok(Some(onto)) => onto,
        Ok(None) => {
            return Ok(Rebase::Unchecked(format!(
                "{remote_name} has no branch {default_branch}"
            )));
        }
        Err(problem) => {
            return Ok(Rebase::Unchecked(format!(
                "could not fetch {default_branch} from {remote_name}: {problem}"
            )));
        }
    };
    let _lock = state.lock_repository()?;
    let Some(head) = git.head()? else {
        return Ok(Rebase::Unchecked("the branch has no commit".to_string()));
    };
    let landed = match landed_commit_for(git, &head, job, verified_fingerprint) {
        Ok(landed) => landed,
        Err(problem) => return Ok(Rebase::Unchecked(format!("{problem:#}"))),
    };
    let plan = plan_rebase(git, &onto, &head, &landed, git.signs_commits()?)?;
    if let Rebase::Moved { tip, .. } = &plan {
        match git.reset_keep(tip)? {
            Ok(()) => ui.line(&format!(
                "the branch was rebased onto {remote_name}/{default_branch} ({}); it held {} before",
                &onto[..onto.len().min(12)],
                &head[..head.len().min(12)]
            )),
            Err(stderr) => bail!(
                "could not move the branch onto {onto}: {}. The branch is unchanged; move the local \
                 edits aside and run the command again",
                stderr.trim()
            ),
        }
    }
    Ok(plan)
}

impl Athena {
    /// `athena land --pr <job>`: takes a verified job to an open pull request, or, when one exists, looks at
    /// it again (see the module text).
    pub fn land_pr(
        &self,
        job_ref: &str,
        runner: &dyn Runner,
        poll: Poll<'_>,
        ui: &dyn Ui,
    ) -> Result<PrReport> {
        self.refuse_if_stopped()?;
        // Reconcile first (§8.1): a landing record missing from an earlier crash is written, or the commit is
        // reported unattributed. It never fails the landing.
        for line in self.reconcile(runner, ui).lines() {
            ui.line(&line);
        }
        let head = self.head()?;
        let records = crate::store::Records::load(&self.git, &head)?;
        let id = records.resolve(job_ref)?;
        let Some(_lock) = self.state.try_lock_job(&id)? else {
            bail!("job {id} is busy in another athena process");
        };
        let job = records.jobs.get(id.as_str()).expect("resolved");
        let verification = job
            .verification
            .as_ref()
            .with_context(|| format!("job {id} has no verification; it cannot be landed"))?;
        if verification.final_verdict != Verdict::Pass {
            bail!("job {id} did not pass verification; it cannot be landed");
        }
        let candidate = job
            .candidate
            .as_ref()
            .with_context(|| format!("job {id} has no candidate record"))?;
        let remote_name = "origin";
        let url = self.git.run(["remote", "get-url", remote_name])?;
        let (host_name, repository) = parse_remote(&url)?;
        let config = athena_gate::config_at(&self.git, Some(&head))?;
        let branch = work_branch(&id);
        let title = job.title();
        let body = format!(
            "{}\n\nVerified job {}.",
            verification.summary.as_deref().unwrap_or(&title),
            id.as_str()
        );
        let content = GitContent {
            git: &self.git,
            rules_commit: &candidate.base_commit,
            remote: remote_name,
        };
        let log = HostErrorLog::new(id.clone(), WorkPhase::Landing);
        let client = GhClient {
            log: Some(&log),
            ui: Some(ui),
            ..GhClient::new(runner, repository.clone())
        };
        let remote = GitRemote {
            git: &self.git,
            name: remote_name,
        };
        // The records sink is built here, on the production path, and its error stops the landing: a
        // landing never goes ahead without somewhere to write its records (§5.2).
        let sink = GitLandingRecords::for_remote(&self.git, remote_name)?;
        let landed = landed_commit_for(&self.git, &head, &id, &verification.verified_fingerprint);
        let landed_id = landed.as_ref().ok().cloned();
        let attempt = || -> Result<PrReport> {
            match landed {
                Ok(landed) => land_pr(&LandPr {
                    state: &self.state,
                    job: &id,
                    branch: &branch,
                    commit: &landed,
                    verified_fingerprint: &verification.verified_fingerprint,
                    pr: PrDraft {
                        title: &title,
                        body: &body,
                        head: &branch,
                    },
                    squash: &squash_text_for(&self.git, &head, &landed)?,
                    host_name: &host_name,
                    repository: &repository,
                    merge_via_rest: config.landing.merge_via_rest,
                    poll,
                    ancestry: &self.git,
                    remote: &remote,
                    host: &client,
                    content: &content,
                    records: Some(&sink),
                }),
                // The landed commit is not on this branch. If the job's pull request already exists, the
                // host may have merged it, and that confirms the landing without the local commit.
                Err(not_landed) => match confirm_existing_merge(
                    &self.state,
                    &id,
                    &client,
                    &content,
                    &verification.verified_fingerprint,
                )? {
                    Some(report) => Ok(report),
                    None => Err(not_landed),
                },
            }
        };
        let outcome = attempt();
        // The watcher runs at landing, after a host error or not: it records every refused call of this run
        // and puts a decision under review when a signal fires. It runs before the outcome is returned, so a
        // refusal that failed the landing is seen too. A failure of the watcher is shown, not allowed to hide
        // the landing's own result.
        if let Err(problem) = self.watch_landing(&log, ui) {
            ui.line(&format!(
                "warning: could not record host errors or watch the decisions: {problem:#}"
            ));
        }
        let report = outcome?;
        ui.line(&format!(
            "pull request #{} on {}/{}: {:?}",
            report.link.pr_number, report.link.host, report.link.repository, report.link.state
        ));
        let pages_poll = Poll {
            attempts: PAGES_WAIT_LOOKS,
            delay: PAGES_WAIT_DELAY,
            sleep: &std::thread::sleep,
        };
        self.after_merge(&report, landed_id.as_deref(), &client, &pages_poll, ui)?;
        Ok(report)
    }

    /// What follows a pull request's merge, in this order: the local branch is moved onto the merged commit,
    /// the Pages deploy is watched, and the code-health check runs (`crate::code_health`). The merge is done
    /// and stays done: a failed Pages watch or code-health check is shown, never returned as an error.
    pub(crate) fn after_merge(
        &self,
        report: &PrReport,
        landed: Option<&str>,
        pages: &dyn PagesHost,
        pages_poll: &Poll<'_>,
        ui: &dyn Ui,
    ) -> Result<()> {
        if report.link.state != LinkState::Landed {
            return Ok(());
        }
        // A merge the host made without the branch's commits (a squash or a rebase) leaves the local branch
        // beside main; bring it onto the merged commit so the next pull request opens clean.
        if let (Some(landed), Some(merged)) = (landed, &report.link.merged_commit) {
            self.resync_onto_merge(landed, merged, ui)?;
        }
        if let Some(merged) = &report.link.merged_commit
            && let Err(problem) = watch_pages(pages, merged, pages_poll, ui)
        {
            ui.line(&format!(
                "warning: could not watch the Pages deploy: {problem:#}"
            ));
        }
        self.code_health_after_landing(ui);
        Ok(())
    }
}

#[cfg(test)]
mod rebase_tests {
    use super::*;
    use crate::Quiet;
    use crate::land::fixtures::{self, File, plain};
    use crate::store::Records;

    const JOB_A: &str = "20261024-115923-02gf";
    const JOB_B: &str = "20261024-120000-0b0b";
    const JOB_C: &str = "20261024-121000-0c0c";
    const BASE: &str = "one\ntwo\nthree\n";
    const JOB_EDIT: &str = "one\nJOB\nthree\n";
    const MAIN_EDIT: &str = "one\nMAIN\nthree\n";

    fn fingerprint() -> String {
        "f".repeat(64)
    }

    fn id(text: &str) -> JobId {
        JobId::parse(text).unwrap()
    }

    /// A repository on `main` with one base commit holding `code.txt`.
    fn repo() -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        fixtures::commit(&git, &[plain("code.txt", BASE)], "base");
        (dir, git)
    }

    /// The landing commit of `job`: its closing records, `extra` files, and the job trailers.
    fn job_commit(git: &Git, job: &str, extra: &[File]) -> String {
        let mut files = fixtures::landed(&id(job));
        files.extend(extra.iter().cloned());
        let message = format!(
            "Job {job}\n\nAthena-Job: {job}\nAthena-Verified-Content: {}\n",
            fingerprint()
        );
        fixtures::commit(git, &files, &message)
    }

    /// A commit that changes `path` on a side branch from `base`; `main` is checked out again afterwards.
    fn outside(git: &Git, base: &str, path: &str, text: &str) -> String {
        git.run(["checkout", "-q", "-b", "outside", base]).unwrap();
        let onto = fixtures::commit(git, &[plain(path, text)], "outside change");
        git.run(["checkout", "-q", "main"]).unwrap();
        onto
    }

    fn show(git: &Git, commit: &str, path: &str) -> Option<String> {
        git.show_file(commit, path)
            .unwrap()
            .map(|b| String::from_utf8(b).unwrap())
    }

    #[test]
    fn plan_rebase_clean_move() {
        let (_dir, git) = repo();
        let base = git.head().unwrap().unwrap();
        let job = job_commit(&git, JOB_A, &[plain("a.txt", "a\n")]);
        let onto = outside(&git, &base, "outside.txt", "o\n");
        let plan = plan_rebase(&git, &onto, &job, &job, false).unwrap();
        let Rebase::Moved {
            onto: moved_onto,
            landed,
            tip,
            records_only,
        } = plan
        else {
            panic!("expected Moved, got {plan:?}");
        };
        assert_eq!(moved_onto, onto);
        assert_eq!(landed, tip);
        assert!(records_only.is_empty());
        assert_ne!(landed, job);
        assert_eq!(git.parents(&landed).unwrap(), vec![onto.clone()]);
        assert_eq!(git.message(&landed).unwrap(), git.message(&job).unwrap());
        assert_eq!(show(&git, &tip, "outside.txt").as_deref(), Some("o\n"));
        assert_eq!(show(&git, &tip, "a.txt").as_deref(), Some("a\n"));
        // The plan moves nothing.
        assert_eq!(git.head().unwrap().unwrap(), job);
    }

    #[test]
    fn plan_rebase_already_current() {
        let (_dir, git) = repo();
        let base = git.head().unwrap().unwrap();
        let job = job_commit(&git, JOB_A, &[plain("a.txt", "a\n")]);
        assert_eq!(
            plan_rebase(&git, &base, &job, &job, false).unwrap(),
            Rebase::Current
        );
        // Main already holding the landing commit is current too.
        assert_eq!(
            plan_rebase(&git, &job, &job, &job, false).unwrap(),
            Rebase::Current
        );
    }

    #[test]
    fn plan_rebase_conflict_in_own_commit() {
        let (_dir, git) = repo();
        let base = git.head().unwrap().unwrap();
        let job = job_commit(&git, JOB_A, &[plain("code.txt", JOB_EDIT)]);
        let onto = outside(&git, &base, "code.txt", MAIN_EDIT);
        match plan_rebase(&git, &onto, &job, &job, false).unwrap() {
            Rebase::Conflict { commit, text } => {
                assert_eq!(commit, job);
                assert!(text.contains(&job), "{text}");
            }
            other => panic!("expected Conflict, got {other:?}"),
        }
    }

    /// Two stacked jobs: `b` edits `code.txt` (conflicting with the moved main when `b_conflicts`), `c` adds
    /// a disjoint file. Returns the repository, main's new tip, `b`'s and `c`'s landing commits.
    fn stack(b_conflicts: bool) -> (tempfile::TempDir, Git, String, String, String) {
        let (dir, git) = repo();
        let base = git.head().unwrap().unwrap();
        let b_code = if b_conflicts {
            plain("code.txt", JOB_EDIT)
        } else {
            plain("b.txt", "b\n")
        };
        let b = job_commit(&git, JOB_B, &[b_code]);
        let c = job_commit(&git, JOB_C, &[plain("c.txt", "c\n")]);
        let onto = outside(&git, &base, "code.txt", MAIN_EDIT);
        (dir, git, onto, b, c)
    }

    #[test]
    fn plan_rebase_earlier_conflict_is_records_only() {
        let (_dir, git, onto, b, c) = stack(true);
        let plan = plan_rebase(&git, &onto, &c, &c, false).unwrap();
        let Rebase::Moved {
            landed,
            tip,
            records_only,
            ..
        } = plan
        else {
            panic!("expected Moved, got {plan:?}");
        };
        assert_eq!(records_only, vec![b.clone()]);
        assert_eq!(landed, tip);
        // The chain is main, the records-only commit of b, then c.
        let carried = git.parents(&tip).unwrap().remove(0);
        assert_eq!(git.parents(&carried).unwrap(), vec![onto.clone()]);
        let message = git.message(&carried).unwrap();
        assert!(
            message.contains(&format!("Athena-Records-Only: {b}")),
            "{message}"
        );
        // b's code is dropped: main's text stands, and b's records are all there.
        assert_eq!(show(&git, &tip, "code.txt").as_deref(), Some(MAIN_EDIT));
        for (path, text) in fixtures::landed(&id(JOB_B)) {
            assert_eq!(show(&git, &tip, &path), Some(text), "{path}");
        }
        assert_eq!(show(&git, &tip, "c.txt").as_deref(), Some("c\n"));
        // The production loader still sees both jobs, and b's closing records.
        let records = Records::load(&git, &tip).unwrap();
        assert!(records.invalid.is_empty(), "{:?}", records.invalid);
        let held = records.jobs.get(JOB_B).expect("the held-back job loads");
        assert!(held.candidate.is_some());
        assert!(held.verification.is_some());
        assert!(held.outcome.is_some());
        assert!(records.jobs.contains_key(JOB_C));
    }

    #[test]
    fn plan_rebase_records_only_commit_is_a_hold() {
        let (_dir, git, onto, _b, c) = stack(true);
        let Rebase::Moved { tip, .. } = plan_rebase(&git, &onto, &c, &c, false).unwrap() else {
            panic!("expected Moved");
        };
        // The lookup by records, as after C0, finds the records-only commit as the job's landing commit.
        let carried = landed_commit_from_records(&git, &tip, &id(JOB_B), &fingerprint()).unwrap();
        assert!(git.is_ancestor(&onto, &carried).unwrap());
        let message = git.message(&carried).unwrap();
        assert!(message.contains("Athena-Records-Only: "), "{message}");
        assert!(!message.contains("Athena-Job"), "{message}");
        // It has no code to land: a hold, not Current, although main is its ancestor.
        match plan_rebase(&git, &onto, &tip, &carried, false).unwrap() {
            Rebase::Conflict { commit, .. } => assert_eq!(commit, carried),
            other => panic!("expected Conflict, got {other:?}"),
        }
        // The pre-C0 lookup by trailers does not find it either.
        assert!(landed_commit(&git, &tip, &id(JOB_B), &fingerprint()).is_err());
    }

    #[test]
    fn plan_rebase_clean_earlier_commit_is_carried() {
        let (_dir, git, onto, b, c) = stack(false);
        let Rebase::Moved {
            landed,
            tip,
            records_only,
            ..
        } = plan_rebase(&git, &onto, &c, &c, false).unwrap()
        else {
            panic!("expected Moved");
        };
        assert!(records_only.is_empty());
        assert_eq!(landed, tip);
        let rebased_b = git.parents(&tip).unwrap().remove(0);
        assert_ne!(rebased_b, b);
        assert_eq!(git.parents(&rebased_b).unwrap(), vec![onto]);
        assert_eq!(git.message(&rebased_b).unwrap(), git.message(&b).unwrap());
        assert_eq!(show(&git, &tip, "b.txt").as_deref(), Some("b\n"));
        assert_eq!(show(&git, &tip, "c.txt").as_deref(), Some("c\n"));
        assert_eq!(show(&git, &tip, "code.txt").as_deref(), Some(MAIN_EDIT));
    }

    #[test]
    fn plan_rebase_unrelated_history_is_an_error() {
        let (_dir, git) = repo();
        let base = git.head().unwrap().unwrap();
        git.run(["checkout", "-q", "--orphan", "other"]).unwrap();
        let job = job_commit(&git, JOB_A, &[plain("a.txt", "a\n")]);
        assert!(plan_rebase(&git, &base, &job, &job, false).is_err());
    }

    /// A bare origin, the work repository cloned from nothing but wired to it, and a state directory.
    struct World {
        tmp: tempfile::TempDir,
        bare: Git,
        work: Git,
        state: StateDir,
    }

    fn world() -> World {
        let tmp = tempfile::tempdir().unwrap();
        let dir = |name: &str| {
            let path = tmp.path().join(name);
            std::fs::create_dir_all(&path).unwrap();
            path
        };
        let bare = Git::isolated(dir("origin.git"));
        bare.run(["init", "-q", "--bare", "-b", "main"]).unwrap();
        let work = Git::isolated(dir("work"));
        work.run(["init", "-q", "-b", "main"]).unwrap();
        let url = bare.dir().to_str().unwrap().to_string();
        work.run(["remote", "add", "origin", &url]).unwrap();
        fixtures::commit(&work, &[plain("code.txt", BASE)], "base");
        work.run(["push", "-q", "origin", "main"]).unwrap();
        let state = StateDir::new(dir("state"));
        World {
            tmp,
            bare,
            work,
            state,
        }
    }

    impl World {
        /// Another clone pushes a commit to origin's main; returns it.
        fn move_main(&self, path: &str, text: &str) -> String {
            let path_dir = self.tmp.path().join("other");
            std::fs::create_dir_all(&path_dir).unwrap();
            let other = Git::isolated(path_dir);
            if other.head().unwrap().is_none() {
                let url = self.bare.dir().to_str().unwrap().to_string();
                other.run(["clone", "-q", &url, "."]).unwrap();
            }
            fixtures::commit(&other, &[plain(path, text)], "outside change");
            other.run(["push", "-q", "origin", "main"]).unwrap();
            other.head().unwrap().unwrap()
        }

        fn rebase(&self, job: &str) -> Rebase {
            rebase_job_branch(
                &self.work,
                &self.state,
                "origin",
                "main",
                &id(job),
                &fingerprint(),
                &Quiet,
            )
            .unwrap()
        }
    }

    #[test]
    fn rebase_job_branch_lands_second_job_on_the_moved_main() {
        let w = world();
        job_commit(&w.work, JOB_A, &[plain("a.txt", "a\n")]);
        let second = job_commit(&w.work, JOB_B, &[plain("b.txt", "b\n")]);
        let outside = w.move_main("outside.txt", "o\n");
        assert!(!w.work.commit_exists(&outside).unwrap(), "not fetched yet");
        let Rebase::Moved {
            onto,
            landed,
            tip,
            records_only,
        } = w.rebase(JOB_B)
        else {
            panic!("expected Moved");
        };
        assert_eq!(onto, outside);
        assert!(records_only.is_empty());
        assert_ne!(landed, second);
        // The local branch is at the new tip, and the second job's commit sits on main.
        assert_eq!(w.work.head().unwrap().unwrap(), tip);
        assert_eq!(landed, tip);
        assert!(w.work.is_ancestor(&outside, &landed).unwrap());
        // History is linear: no merge, one parent per new commit, the first job riding along.
        let range = format!("{outside}..{tip}");
        assert_eq!(
            w.work.run(["rev-list", "--merges", &range]).unwrap().trim(),
            ""
        );
        let commits = w.work.run(["rev-list", &range]).unwrap();
        assert_eq!(commits.lines().count(), 2);
        for commit in commits.lines() {
            assert_eq!(w.work.parents(commit).unwrap().len(), 1, "{commit}");
        }
        let subjects = w.work.run(["log", "--format=%s", &range]).unwrap();
        assert_eq!(
            subjects.lines().collect::<Vec<_>>(),
            [format!("Job {JOB_B}"), format!("Job {JOB_A}")]
        );
        assert_eq!(show(&w.work, &tip, "outside.txt").as_deref(), Some("o\n"));
        // Nothing was pushed.
        assert_eq!(w.bare.rev_parse("main").unwrap().unwrap(), outside);
        // Run again: the branch is current.
        assert_eq!(w.rebase(JOB_B), Rebase::Current);
    }

    #[test]
    fn rebase_job_branch_conflict_changes_nothing() {
        let w = world();
        job_commit(&w.work, JOB_A, &[plain("a.txt", "a\n")]);
        let second = job_commit(&w.work, JOB_B, &[plain("code.txt", JOB_EDIT)]);
        let outside = w.move_main("code.txt", MAIN_EDIT);
        match w.rebase(JOB_B) {
            Rebase::Conflict { commit, .. } => assert_eq!(commit, second),
            other => panic!("expected Conflict, got {other:?}"),
        }
        assert_eq!(w.work.head().unwrap().unwrap(), second);
        assert_eq!(w.bare.rev_parse("main").unwrap().unwrap(), outside);
        assert_eq!(show(&w.work, "HEAD", "code.txt").as_deref(), Some(JOB_EDIT));
    }

    #[test]
    fn rebase_job_branch_unchecked_when_origin_is_unreachable() {
        let w = world();
        let job = job_commit(&w.work, JOB_A, &[plain("a.txt", "a\n")]);
        let missing = w.tmp.path().join("missing.git");
        let missing = missing.to_str().unwrap();
        w.work
            .run(["remote", "set-url", "origin", missing])
            .unwrap();
        assert!(matches!(w.rebase(JOB_A), Rebase::Unchecked(_)));
        assert_eq!(w.work.head().unwrap().unwrap(), job);
    }

    #[test]
    fn rebase_job_branch_unchecked_for_a_published_head() {
        let w = world();
        let job = job_commit(&w.work, JOB_A, &[plain("a.txt", "a\n")]);
        let refname = format!("HEAD:refs/heads/{}", work_branch(&id(JOB_A)));
        w.work.run(["push", "-q", "origin", &refname]).unwrap();
        w.move_main("outside.txt", "o\n");
        match w.rebase(JOB_A) {
            Rebase::Unchecked(reason) => assert!(reason.contains("published"), "{reason}"),
            other => panic!("expected Unchecked, got {other:?}"),
        }
        assert_eq!(w.work.head().unwrap().unwrap(), job);
    }
}
