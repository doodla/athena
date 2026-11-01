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
/// the setting is on and the checks are green.
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
            let method = job.host.merge(link.pr_number, &link.head, job.squash)?;
            merge_requested = true;
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
