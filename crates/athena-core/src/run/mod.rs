//! `athena go`: dispatch, produce, verify independently, and land (design §2.3, §3.4, §7).

use crate::admission::{AdmissionInput, Changed, admit, is_governance, plan_findings};
use crate::capability::{Capability, ChangeList, JobConfig};
use crate::chain::latest_landed_job;
use crate::checks::{CheckContext, run_check};
use crate::commits::{NewFile, Refusal, message};
use crate::invocation::InvocationFacts;
use crate::land::{Plan, plan};
use crate::pin::{is_full_sha, mount};
use crate::reading::{
    BinaryChecker, NO_HISTORY_REASON, Proposal, ReadingChecker, ReplayCheckers, ReplayRun,
    TreeRetrySeed, planning_resume_seed, replay, replay_cases, replay_declaration_of,
    resumable_job, retried_job, retry_findings, same_scope, tree_retry_seed,
};
use crate::scoring::{self, Happened, score, totals};
use crate::state::{ResumeMark, ResumeStage};
use crate::store::{JobRecords, JobState, Records};
use crate::verify::{MAP_PATH, VerdictInputs, Verified, judge, mark_pre_existing, verified};
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_agent::env::{EnvSpec, Side};
use athena_agent::policy::Policy;
use athena_context::briefs::{
    AcceptedReading, CandidateView, CheckResultView, CheckView, CriticInputs, DeliverableFact,
    PlanRevision, PlannerInputs, ProducerInputs, ReplayView, RetryFindings, RetrySeed,
    VerifierInputs, VerifierPlan, class_summaries, critic_brief, planner_brief, producer_brief,
    producer_prompt_with_plan, render_plan_section, verifier_brief_with_plan,
};
use athena_context::outputs::{
    CriticOutput, CritiqueVerdictOutput, DeliverableActionOutput, PlanOutput, ProducerOutput,
    SeverityOutput, VerdictOutput, critic_schema, plan_schema, producer_schema, verdict_schema,
};
use athena_context::{
    CONTRACT, CRITIC_FRAME, Component, PLANNER_FRAME, PRODUCER_FRAME, VERIFIER_FRAME, compose,
};
use athena_data::records::{
    AcceptedBy, AnyRecord, Baseline, Build, Candidate, Caps, ChangeEntry, ChangeStatus, CheckRun,
    Critique, CritiqueVerdict, DeliverableAction, Demonstration, Finding, FindingSource,
    Invocation, InvocationOutcome, Job, JobKind, JobPlan, Outcome, OutcomeState, Owner, PinnedRepo,
    PlanDecision, PlanFile, PlanRisk, PolicyViolation, ProducerReport, Reading, RejectedOption,
    Replay, ReportedCheck, ReportedCriterion, Role, Score, Severity, Totals, Verdict, Verification,
};
use athena_data::{JobId, sha256_hex};
use athena_gate::redact::Redactor;
use athena_git::attrs::fingerprint;
use athena_git::index::{IndexEdit, TempIndex};
use athena_git::snapshot::{Snapshot, SnapshotRepo};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Instant;

mod close;
mod pin;
mod planning;
mod producer;
mod send_back;

use producer::{Flow, Reported};

pub use close::{build_text, closing_subject};
pub(crate) use send_back::{left_to_athena, send_back_text, unmet_criteria};

/// How `go` ended.
#[derive(Debug, Clone)]
pub enum GoResult {
    /// Athena read the request, and the reading needs accepting: a queued request read for the first
    /// time, or a reading made again because the configuration changed since it.
    ///
    /// A reading stays valid across a configuration change unless the change bears on a reading: it is
    /// judged by [`athena_data::config::Config::reading_inputs`] (every `athena.toml` setting but
    /// `[parallel]` and `[landing]`, compared as parsed values) and by the bytes of `defaults.toml`
    /// (D38). When a relevant change forces a new reading, a caller that accepts by flag
    /// ([`AcceptedBy::Flag`], which `athena drain` uses) accepts it at once and runs the job if it has the
    /// same scope as the reading it replaces ([`crate::reading::same_scope`]: capability, class of work
    /// and deliverable paths and actions). A reading whose scope changed is returned here for the user to
    /// accept, as is any reread for [`AcceptedBy::User`].
    Reread(Box<Proposal>),
    /// The job reached a terminal state, or `blocked`.
    Finished(Box<Finished>),
}

/// A job that reached a terminal state (or `blocked`).
#[derive(Debug, Clone)]
pub struct Finished {
    pub job: JobId,
    pub state: OutcomeState,
    pub reason: String,
    pub reading: Reading,
    pub prediction_commit: Option<String>,
    pub closing_commit: String,
    pub landed_commit: Option<String>,
    pub totals: Option<Totals>,
    pub score: Option<Score>,
    pub verification: Option<Verification>,
}

/// Everything a dispatched job accumulates before its closing commit.
struct Closing {
    files: Vec<NewFile>,
    new_invocations: Vec<Invocation>,
    candidate: Option<Candidate>,
    verification: Option<Verification>,
    /// Set only on a final verdict of pass.
    verified: Option<Verified>,
    happened: Happened,
    /// The accepted plans of a planned job, written with the invocations.
    plans: Vec<JobPlan>,
    /// The critiques of those plans.
    critiques: Vec<Critique>,
    /// The checks run on the base commit before the producer. Recorded only.
    baseline: Option<Baseline>,
    /// The earlier job whose kept candidate this job resumed from; its resume mark is spent when this job closes.
    consumed_resume: Option<JobId>,
}

/// A built candidate on its way through the checks and the verifier: what [`Athena::check_and_verify`] takes
/// from the producer half, or from a resume.
struct Built {
    /// The base (prediction) commit.
    base: String,
    /// The candidate commit, on `base`.
    commit: String,
    /// The candidate's fingerprint against `base`.
    fingerprint: String,
    /// The sequence number the next invocation gets.
    seq: u32,
    report_gaps: Vec<String>,
    left_criteria: Vec<String>,
    /// The check results of an earlier run on this very candidate: the checks do not run again.
    recorded_checks: Option<Vec<CheckRun>>,
}

/// A stopped job's kept candidate that a retry resumes from.
struct ResumeSeed {
    job: JobId,
    mark: ResumeMark,
}

impl Closing {
    fn new() -> Closing {
        Closing {
            consumed_resume: None,
            baseline: None,
            plans: Vec::new(),
            critiques: Vec::new(),
            files: Vec::new(),
            new_invocations: Vec::new(),
            candidate: None,
            verification: None,
            verified: None,
            happened: Happened::default(),
        }
    }
}

fn change_status(status: char) -> ChangeStatus {
    match status {
        'A' => ChangeStatus::Added,
        'D' => ChangeStatus::Deleted,
        'T' => ChangeStatus::TypeChanged,
        _ => ChangeStatus::Modified,
    }
}

/// What the verifier brief shows of a replay record and of what the reading declares about it. The declared
/// share and its reason are read from the reading by the same function the gate uses, so the verifier sees
/// the share that was applied. A share above 25% with no reason is shown as unsupported, and the 10% limit
/// applied to it.
fn replay_view(replay: &Replay, reading: &Reading) -> ReplayView {
    let declaration = replay_declaration_of(reading);
    ReplayView {
        total: replay.total as usize,
        newly_refused: replay
            .newly_refused
            .iter()
            .map(|r| (r.job.as_str().to_string(), r.reason.clone()))
            .collect(),
        held: replay.held,
        declared: declaration.effective_share(),
        reason: declaration
            .effective_share()
            .and(declaration.reason.clone()),
        unsupported: declaration.unsupported_share(),
        no_replay: replay.no_replay.clone(),
    }
}

/// The findings on a plan: Athena's own (minor ones only, here) first, then the critic's. A soundness finding
/// is an ordinary finding of the critic, with its own severity.
fn critic_findings(athena_findings: Vec<Finding>, critic: &CriticOutput) -> Vec<Finding> {
    let mut findings = athena_findings;
    findings.extend(critic.findings.iter().map(|f| Finding {
        severity: match f.severity {
            SeverityOutput::Blocker => Severity::Blocker,
            SeverityOutput::Major => Severity::Major,
            SeverityOutput::Minor => Severity::Minor,
            SeverityOutput::Nit => Severity::Nit,
        },
        summary: f.summary.clone(),
        evidence: format!("{} Fix: {}", f.evidence.trim(), f.fix.trim()),
        location: Some(f.location.clone()),
        source: FindingSource::Critic,
    }));
    findings
}

/// The plan's verdict: the critic's own, except that an accept with a blocker or major finding on the plan
/// (from either source) becomes revise.
fn plan_verdict(agent_verdict: CritiqueVerdict, findings: &[Finding]) -> CritiqueVerdict {
    if agent_verdict == CritiqueVerdict::Accept
        && findings
            .iter()
            .any(|f| matches!(f.severity, Severity::Blocker | Severity::Major))
    {
        CritiqueVerdict::Revise
    } else {
        agent_verdict
    }
}

/// Runs `command` to the end, killing it after `timeout`. An error says how it ended when it did not succeed.
fn run_bounded(
    command: &mut std::process::Command,
    timeout: std::time::Duration,
) -> std::result::Result<(), String> {
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(status)) => return Err(format!("it exited with {status}")),
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("it ran past {}s", timeout.as_secs()));
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(200)),
            Err(e) => return Err(e.to_string()),
        }
    }
}

fn short(id: &str) -> &str {
    &id[..id.len().min(12)]
}

/// The commit that added a path, searching back from `rev`.
pub fn introducing_commit(athena: &Athena, rev: &str, path: &str) -> Result<Option<String>> {
    let out = athena
        .git
        .run(["log", "--format=%H", "--diff-filter=A", rev, "--", path])?;
    Ok(out.lines().last().map(str::to_string))
}

impl Athena {
    /// `athena go`: runs a proposed job to its terminal state, or retries the landing of a blocked one.
    ///
    /// A job that ends `rejected` or `failed` is introspected at once, with no request from the user. The
    /// introspection never changes the result and never blocks `go` (`defaults-loop.md` section 3.1).
    pub fn go(&self, job_ref: &str, accepted_by: AcceptedBy, ui: &dyn Ui) -> Result<GoResult> {
        let result = self.go_job(job_ref, accepted_by, ui)?;
        if let GoResult::Finished(finished) = &result {
            if matches!(
                finished.state,
                OutcomeState::Rejected | OutcomeState::Failed
            ) {
                self.introspect_job(&finished.job, ui);
            }
            // The steward runs beside the main flow: this only counts the task and may start it.
            self.note_finished(finished.state, ui);
            // The line watcher records a flag or a pause for the line; it never changes the result.
            self.watch_line(&finished.job, ui);
        }
        Ok(result)
    }

    fn go_job(&self, job_ref: &str, accepted_by: AcceptedBy, ui: &dyn Ui) -> Result<GoResult> {
        self.refuse_if_stopped()?;
        self.state.registry().reap();
        let head = self.head()?;
        let records = Records::load(&self.git, &head)?;
        let id = records.resolve(job_ref)?;
        let Some(_job_lock) = self.state.try_lock_job(&id)? else {
            bail!("job {id} is already running in another athena process");
        };
        let job = records.jobs.get(id.as_str()).expect("resolved").clone();
        match job.state() {
            JobState::Queued => self
                .read_queued(&job, ui)
                .map(|p| GoResult::Reread(Box::new(p))),
            JobState::Blocked => self
                .retry_landing(&records, &job, ui)
                .map(|f| GoResult::Finished(Box::new(f))),
            JobState::Proposed => match job.latest_reading() {
                Some(reading) if reading.capability.is_some() => {
                    self.dispatch(&head, &records, &job, reading.clone(), accepted_by, ui)
                }
                Some(_) => unreachable!("a reading without a capability is declined"),
                None => bail!(
                    "job {id} has no reading yet (the reader failed); `athena correct {id} \"<line>\"` reads it again"
                ),
            },
            JobState::Declined => bail!(
                "job {id} was declined: {}. Correct it with `athena correct {id} \"<line>\"`, or `athena close {id} --why \"...\"`",
                job.latest_reading()
                    .and_then(|r| r.decline_reason.clone())
                    .unwrap_or_default()
            ),
            JobState::Dispatched => {
                if self.running(id.as_str()) {
                    bail!("job {id} is running");
                }
                bail!(
                    "job {id} was interrupted: its process died before it closed. `athena retry {id}` reads it again to resume, or `athena close {id} --why \"...\"` records what happened"
                )
            }
            JobState::Closed(state) => bail!("job {id} is already closed ({})", state.as_str()),
        }
    }

    /// The seed for a producer retry that starts from a rejected candidate, or `None` when the retry starts
    /// from the base. The request must be "try again on <job>" for a job that
    /// [`qualifies_for_tree_retry`](crate::reading::qualifies_for_tree_retry) (unmet content criteria, or
    /// blocking findings that cite only documentation files); the rejected candidate must
    /// still be in the repository; and the files agents see at `head` must be those the rejected job's base
    /// had (only `records/` differs), so that starting from the old candidate cannot undo later work.
    fn tree_seed(
        &self,
        records: &Records,
        request: &str,
        redactor: &Redactor,
        head: &str,
    ) -> Result<Option<TreeRetrySeed>> {
        let Some(seed) = tree_retry_seed(records, request, redactor)? else {
            return Ok(None);
        };
        if self.git.rev_parse(&seed.candidate_commit)?.is_none()
            || self.git.rev_parse(&seed.base_commit)?.is_none()
        {
            return Ok(None);
        }
        let changed = self.git.diff_tree(&seed.base_commit, head)?;
        if changed.iter().any(|c| !c.path.starts_with("records/")) {
            return Ok(None);
        }
        Ok(Some(seed))
    }

    /// The kept candidate of a stopped job that a retry resumes from. The request must be "try again on <job>"
    /// for a job that failed for an environment cause or was interrupted
    /// ([`resumable_job`]); its resume mark must exist, and the candidate and base commits must still be in
    /// the repository; and the files agents see at `head` must be those the job's base had (only `records/`
    /// differs), so that resuming cannot undo later work. Otherwise `None`: the retry plans and produces.
    fn resume_seed(
        &self,
        records: &Records,
        request: &str,
        head: &str,
    ) -> Result<Option<ResumeSeed>> {
        let Some(job) = resumable_job(records, request, &|j| self.running(j)) else {
            return Ok(None);
        };
        let Some(mark) = self.state.read_resume(&job) else {
            return Ok(None);
        };
        let Some(commit) = mark.candidate.candidate_commit.clone() else {
            return Ok(None);
        };
        if !mark.candidate.violations.is_empty()
            || self.git.rev_parse(&commit)?.is_none()
            || self.git.rev_parse(&mark.base_commit)?.is_none()
        {
            return Ok(None);
        }
        let changed = self.git.diff_tree(&mark.base_commit, head)?;
        if changed.iter().any(|c| !c.path.starts_with("records/")) {
            return Ok(None);
        }
        Ok(Some(ResumeSeed { job, mark }))
    }

    /// Resumes a stopped job's kept candidate in this job, with no planner and no producer run. The candidate's
    /// changes are applied again to this job's base, so the verifier's diff and the landing carry no records
    /// noise from the older base. Admission runs again on it. At the stage `Verifier` the check results
    /// of the earlier job are used, when the rebuilt candidate has the fingerprint the checks ran on; at any
    /// other stage, or when it differs, the checks run.
    fn resume_candidate(
        &self,
        ctx: &RunContext<'_>,
        accepted: &AcceptedReading,
        seed: ResumeSeed,
        mut closing: Closing,
    ) -> Result<Closing> {
        let id = ctx.id().clone();
        let p = ctx.prediction.clone();
        let ResumeSeed { job: old_job, mark } = seed;
        let old_commit = mark
            .candidate
            .candidate_commit
            .clone()
            .context("the resume mark has no candidate commit")?;
        ctx.ui.line(&format!(
            "resume     job {old_job} kept its candidate; resuming it at the {} stage, with no planner or producer run",
            match mark.stage {
                ResumeStage::Checks => "checks",
                ResumeStage::Verifier => "verifier",
            }
        ));
        let changes = self.git.diff_tree(&mark.base_commit, &old_commit)?;
        let new_blobs: Vec<String> = changes
            .iter()
            .filter(|c| c.status != 'D')
            .map(|c| c.new_oid.clone())
            .collect();
        let sizes = self.git.blob_sizes(&new_blobs)?;
        let mut edits = Vec::new();
        let mut changed = Vec::new();
        for change in &changes {
            if change.status == 'D' {
                edits.push(IndexEdit::Remove {
                    path: change.path.clone(),
                });
            } else {
                edits.push(IndexEdit::Put {
                    mode: change.new_mode.clone(),
                    oid: change.new_oid.clone(),
                    path: change.path.clone(),
                });
            }
            changed.push(Changed {
                path: change.path.clone(),
                status: change_status(change.status),
                mode: change.new_mode.clone(),
                size: sizes.get(&change.new_oid).copied().unwrap_or(0),
            });
        }
        let index = TempIndex::from_tree(&self.git, &p)?;
        index.apply(&edits)?;
        let tree = index.write_tree()?;
        let c = self.git.commit_tree(
            &tree,
            &[&p],
            &format!(
                "Candidate for job {id}\n\nNot on any branch; landing builds its own commit.\n"
            ),
            false,
        )?;
        let f = fingerprint(&self.git, &c, &p)?;
        let mut candidate = mark.candidate.clone();
        candidate.job = id.clone();
        candidate.base_commit = p.clone();
        candidate.base_fingerprint = Some(fingerprint(&self.git, &p, &p)?);
        candidate.previous_job = latest_landed_job(&self.git, &p)?;
        candidate.candidate_commit = Some(c.clone());
        candidate.candidate_tree = Some(tree);
        candidate.fingerprint = Some(f.clone());
        let jc = ctx.jc;
        let mut violations: Vec<PolicyViolation> = admit(&AdmissionInput {
            git: &self.git,
            base: &p,
            candidate: &c,
            changes: &changed,
            reading: ctx.reading,
            max_file_bytes: jc.config.records.max_file_bytes,
            personal_data: &jc.config.personal_data,
            config: &jc.config,
        })?;
        violations.extend(ctx.capability.admit(
            &ChangeList {
                entries: candidate.changes.clone(),
            },
            jc,
        ));
        ctx.ui.line(&format!(
            "candidate  {} · fingerprint {}…",
            candidate
                .changes
                .iter()
                .map(|e| e.path.clone())
                .collect::<Vec<_>>()
                .join(", "),
            &f[..18.min(f.len())]
        ));
        closing.happened.changed_paths = changes.iter().map(|c| c.path.clone()).collect();
        closing.consumed_resume = Some(old_job);
        if !violations.is_empty() {
            let summary = violations
                .iter()
                .map(|v| v.detail.clone())
                .collect::<Vec<_>>()
                .join("; ");
            candidate.violations = violations;
            closing.candidate = Some(candidate);
            let changed_paths = std::mem::take(&mut closing.happened.changed_paths);
            closing.happened =
                self.failure(ctx, &format!("admission refused the change: {summary}"));
            closing.happened.changed_paths = changed_paths;
            return Ok(closing);
        }
        closing.candidate = Some(candidate);
        let same_checks = mark.candidate.fingerprint.as_deref() == Some(f.as_str());
        let built = Built {
            base: p,
            commit: c,
            fingerprint: f,
            seq: ctx.next_seq(&closing),
            report_gaps: mark.report_gaps,
            left_criteria: mark.left_criteria,
            recorded_checks: (mark.stage == ResumeStage::Verifier && same_checks)
                .then_some(mark.check_runs),
        };
        self.write_resume_mark(ctx, &closing, &built);
        self.check_and_verify(ctx, accepted, closing, built)
    }

    /// Records where `built` stands, so that a job that stops (by the environment, or when its process dies)
    /// can be resumed from this candidate. A failure to write it is a warning: it costs the resume, not the job.
    fn write_resume_mark(&self, ctx: &RunContext<'_>, closing: &Closing, built: &Built) {
        let Some(candidate) = closing.candidate.clone() else {
            return;
        };
        let (stage, check_runs) = match &built.recorded_checks {
            Some(runs) => (ResumeStage::Verifier, runs.clone()),
            None => (ResumeStage::Checks, Vec::new()),
        };
        let mark = ResumeMark {
            stage,
            base_commit: built.base.clone(),
            candidate,
            check_runs,
            report_gaps: built.report_gaps.clone(),
            left_criteria: built.left_criteria.clone(),
        };
        if let Err(e) = self.state.write_resume(ctx.id(), &mark) {
            ctx.ui.line(&format!(
                "warning: could not record where this job stands, so a retry cannot resume it: {e:#}"
            ));
        }
    }

    /// Spends the resume marks when a job closes: the mark of the job this one resumed from, and this job's
    /// own unless it closed failed for an environment cause (which `athena retry` resumes from).
    fn settle_resume_marks(
        &self,
        id: &JobId,
        consumed: Option<&JobId>,
        state: OutcomeState,
        reason: &str,
    ) {
        if let Some(old) = consumed {
            self.state.remove_resume(old);
        }
        if !(state == OutcomeState::Failed && crate::disk::is_environment_reason(reason)) {
            self.state.remove_resume(id);
        }
    }

    /// The kept work of a timed out job that a retry resumes from: the request must be "try again on <job>"
    /// for a job that closed `timed_out`; its kept work tree and progress note must still exist; and the
    /// files agents see at `head` must be those the job's base had (only `records/` differs), so that
    /// resuming cannot undo later work. Otherwise `None`: the retry starts from the base.
    fn timeout_seed(
        &self,
        records: &Records,
        request: &str,
        head: &str,
    ) -> Result<Option<TimeoutSeed>> {
        let Some(id) = retried_job(records, request) else {
            return Ok(None);
        };
        let Some(job) = records.jobs.get(id.as_str()) else {
            return Ok(None);
        };
        if job.state() != JobState::Closed(OutcomeState::TimedOut) {
            return Ok(None);
        }
        let dir = self.state.kept_work_dir(&id);
        let (Some(note), Some(base)) = (
            job.briefs.get("progress-note"),
            job.job.as_ref().and_then(|j| j.base_commit.clone()),
        ) else {
            return Ok(None);
        };
        if !dir.is_dir() || self.git.rev_parse(&base)?.is_none() {
            return Ok(None);
        }
        let changed = self.git.diff_tree(&base, head)?;
        if changed.iter().any(|c| !c.path.starts_with("records/")) {
            return Ok(None);
        }
        Ok(Some(TimeoutSeed {
            job: id,
            note: note.clone(),
            dir,
        }))
    }

    /// Ends a producer that hit its deadline. The work tree moves out of the job's directory, which closing
    /// removes, and the progress note listing the changed files goes into the closing. A study's pinned
    /// checkout is mounted inside the work tree, so a pinned job keeps nothing.
    fn keep_timed_out_work(
        &self,
        ctx: &RunContext<'_>,
        snapshot: &Snapshot,
        reason: &str,
        closing: &mut Closing,
    ) -> Result<()> {
        let id = ctx.id().clone();
        let changes = snapshot.change_list().unwrap_or_default();
        let kept_dir = self.state.kept_work_dir(&id);
        let kept = if ctx.pin_dir.is_some() {
            Err("a pinned study keeps no work tree".to_string())
        } else {
            std::fs::create_dir_all(kept_dir.parent().expect("has a parent"))
                .and_then(|_| std::fs::rename(snapshot.dir(), &kept_dir))
                .map_err(|e| e.to_string())
        };
        let mut texts = vec![format!(
            "# Progress note\n\nThe producer of job {id} stopped: {reason}.\n\n"
        )];
        let mut listing = if changes.is_empty() {
            "No file had been changed.\n".to_string()
        } else {
            let mut s = format!(
                "Changed files ({}), against the base commit:\n\n",
                changes.len()
            );
            for c in &changes {
                s.push_str(&format!("- {} {}\n", c.status, c.path));
            }
            s
        };
        match &kept {
            Ok(()) => listing.push_str(
                "\nThe edits were kept; `athena retry` starts the next producer from them.\n",
            ),
            Err(why) => listing.push_str(&format!(
                "\nThe edits could not be kept ({why}); a retry starts from the base.\n"
            )),
        }
        texts[0].push_str(&listing);
        ctx.redactor.redact_all(&mut texts)?;
        closing.files.push(NewFile::text(
            format!("{}/progress-note.md", id.dir()),
            texts.remove(0),
        ));
        closing.happened = self.failure(ctx, &format!("the producer did not complete: {reason}"));
        closing.happened.state = Some(OutcomeState::TimedOut);
        Ok(())
    }

    fn dispatch(
        &self,
        head: &str,
        records: &Records,
        job: &JobRecords,
        reading: Reading,
        accepted_by: AcceptedBy,
        ui: &dyn Ui,
    ) -> Result<GoResult> {
        let id = job.id().clone();
        // A paused line gets no new attempt until the owner lifts the pause (`scope_guard`).
        if let Some(pause) = self.line_pause(records, id.as_str()) {
            bail!("{}", self.pause_refusal(&pause));
        }
        let jc = JobConfig::at(&self.git, head)?;
        if reading.config_sha256 != jc.config_sha256
            || reading.defaults_sha256 != jc.defaults_sha256
        {
            ui.line("reading    the configuration that bears on a reading changed since this reading; reading again under the new configuration");
            let proposal = self.reread_after_configuration_change(&jc, records, job, ui)?;
            // A non-interactive caller (`--yes`, drain) accepts a reading that promises the same work; any
            // other reread waits for the user.
            let same = proposal.reader_error.is_none()
                && proposal
                    .reading
                    .as_ref()
                    .is_some_and(|new| same_scope(&reading, new));
            if accepted_by != AcceptedBy::Flag || !same {
                return Ok(GoResult::Reread(Box::new(proposal)));
            }
            ui.line(
                "reading    the new reading has the same scope as the one it replaces; accepted",
            );
            let new_head = self.head()?;
            let new_records = Records::load(&self.git, &new_head)?;
            let new_job = new_records
                .jobs
                .get(id.as_str())
                .context("the job vanished after it was read again")?
                .clone();
            let new_reading = proposal
                .reading
                .clone()
                .context("the new reading vanished")?;
            return self.dispatch(
                &new_head,
                &new_records,
                &new_job,
                new_reading,
                accepted_by,
                ui,
            );
        }
        // Free disk space, before anything is committed or any agent runs. When it stays low after the idle
        // build directories are removed, the job is held: it stays queued, not rejected, and a later drain
        // picks up the same job. A hold is an error from `go`; no state changed.
        if let Some(low) = self.disk_gate(jc.config.min_free_gb(), &id, ui) {
            crate::disk::write_hold(&self.state, &id, &low);
            ui.line(&format!("held       {}", low.text()));
            return Err(low.into());
        }
        crate::disk::clear_hold(&self.state, &id);
        let capability_id = reading.capability.clone().expect("checked by the caller");
        let capability = self.capability(&capability_id)?;
        let spent = job.spent_usd();
        self.check_budget(
            &jc,
            spent,
            jc.config.budget.producer_usd + jc.config.budget.verifier_usd,
            "producer and verifier together",
        )?;
        let redactor = self.redactor(&jc.config, Some(&id))?;

        // A study's repository is pinned by the core, here, before anything is committed or any agent runs.
        let request_text = job.request.as_ref().map(|r| r.text.as_str()).unwrap_or("");
        let pinned = self.pin_repo(capability, &jc, &id, head, request_text)?;
        let retry_seed = self.tree_seed(records, request_text, &redactor, head)?;
        let resume = if retry_seed.is_none() && pinned.is_none() {
            self.timeout_seed(records, request_text, head)?
        } else {
            None
        };
        // A job that stopped for the environment, or was interrupted, after its candidate was built resumes from
        // that candidate: no planner and no producer run (`resume_candidate`).
        let resume_from = if pinned.is_none() {
            self.resume_seed(records, request_text, head)?
        } else {
            None
        };
        // A retry that starts from the base still carries the blocker and major findings of the job it retries.
        let earlier_findings = if retry_seed.is_none() && resume.is_none() {
            retry_findings(records, request_text, &redactor)?
        } else {
            None
        };

        // The prediction commit: the accepted reading's job record and the producer brief, before any work.
        let producer_grant = capability.producer_grant(&jc);
        let checks = capability.checks(&jc);
        let accepted = AcceptedReading::new(&reading, &jc.defaults, is_governance);
        let check_views: Vec<CheckView> = checks
            .iter()
            .map(|c| CheckView {
                name: c.name.clone(),
                command: c.run.join(" "),
            })
            .collect();
        let brief = producer_brief(&ProducerInputs {
            job: id.as_str().to_string(),
            request: job
                .request
                .as_ref()
                .map(|r| r.text.clone())
                .unwrap_or_default(),
            reading: accepted.clone(),
            checks: check_views.clone(),
            commands: athena_agent::policy::describe_commands(&producer_grant.commands),
            pin: pinned.as_ref().map(|(record, _)| record.clone()),
            retry_from: match (&retry_seed, &resume) {
                (Some(s), _) => Some(RetrySeed {
                    job: s.job.as_str().to_string(),
                    unmet: s.unmet.clone(),
                    findings: s.findings.clone(),
                    must_address: s.must_address.clone(),
                    progress_note: None,
                }),
                (None, Some(r)) => Some(RetrySeed {
                    job: r.job.as_str().to_string(),
                    unmet: Vec::new(),
                    findings: Vec::new(),
                    must_address: Vec::new(),
                    progress_note: Some(r.note.clone()),
                }),
                (None, None) => None,
            },
            retry_findings: earlier_findings.map(|(job, findings)| RetryFindings {
                job: job.as_str().to_string(),
                findings,
            }),
            failure_classes: class_summaries(&records.introspections, &records.classes),
        });
        let dispatched_at = crate::now();
        let clock = Instant::now();
        let budget = &jc.config.budget;
        let job_record = Job {
            id: id.clone(),
            kind: JobKind::Request,
            title: reading.title.clone(),
            class_of_work: Some(reading.class_of_work.clone()),
            capability: Some(capability_id.clone()),
            accepted_reading: Some(reading.n),
            accepted_by: Some(accepted_by),
            owner: Owner {
                role: "producer".into(),
                capability: Some(capability_id.clone()),
                model: Some(reading.producer_model.clone()),
                note: None,
            },
            verifier_model: Some(reading.verifier_model.clone()),
            caps: Some(Caps {
                job_usd: budget.job_usd,
                reader_usd: budget.reader_usd,
                producer_usd: budget.producer_usd,
                verifier_usd: budget.verifier_usd,
                producer_timeout_s: budget.producer_timeout_s,
                verifier_timeout_s: budget.verifier_timeout_s,
                check_timeout_s: budget.check_timeout_s,
            }),
            base_commit: Some(head.to_string()),
            config_sha256: Some(jc.config_sha256.clone()),
            defaults_sha256: Some(jc.defaults_sha256.clone()),
            athena: Some(self.build.clone()),
            driver: Some(self.driver.info()),
            brief_sha256: Some(sha256_hex(brief.as_bytes())),
            dispatched_at: Some(dispatched_at),
            request_path: None,
            request_sha256: None,
            cost: None,
            tokens: None,
            note: None,
            pin: pinned.as_ref().map(|(record, _)| record.clone()),
        };
        let files = vec![
            NewFile::record(&AnyRecord::Job(Box::new(job_record.clone()))),
            NewFile::text(format!("{}/brief-producer.md", id.dir()), brief.clone()),
        ];
        let prediction = self.commit_records(
            &files,
            &message(
                &format!("Dispatch: {}", reading.title),
                "The accepted reading's job record and the producer brief, committed before any work starts. The \
                 reading and its forecasts were committed earlier, when they were made.",
                &[("Athena-Job", id.as_str().to_string())],
            ),
        )?;
        ui.line(&format!(
            "dispatched {id} · prediction commit {}",
            short(&prediction)
        ));

        let mut job = job.clone();
        job.job = Some(job_record);
        let ctx = RunContext {
            jc: &jc,
            job: &job,
            reading: &reading,
            capability,
            redactor: &redactor,
            prediction: prediction.clone(),
            clock,
            ui,
            pin_dir: pinned.map(|(_, dir)| dir),
            seed_commit: retry_seed.map(|s| s.candidate_commit),
            resume,
        };
        // Job planning (docs/design/job-planning.md): one accepted round, before the producer. The round
        // seeds the closing with its invocations, plan and critique, so the producer's invocation is
        // numbered after them and the budget check below counts what planning cost (review F1).
        if let Some(seed) = resume_from {
            let closing = self.resume_candidate(&ctx, &accepted, seed, Closing::new())?;
            return self.close_dispatched(&ctx, closing);
        }
        let mut closing = Closing::new();
        let mut producer_prompt = brief.clone();
        // In `hard` mode only a reading marked hard is planned; the marking is the one that escalates the
        // producer (job-planning.md section 9, D143).
        let planned = jc
            .config
            .job_plan
            .as_ref()
            .is_some_and(|p| p.plans(ctx.reading.hard_reason.is_some()))
            && !ctx.capability.pins_repo();
        if planned {
            let seed = planning_resume_seed(records, request_text).map(|s| PlanRevision {
                previous: s.plan,
                findings: s.findings,
                resumed_from: Some(s.job),
            });
            let Some(section) =
                self.plan_round(&ctx, &accepted, &check_views, &mut closing, seed)?
            else {
                return self.close_dispatched(&ctx, closing);
            };
            producer_prompt = producer_prompt_with_plan(&brief, &section);
            let spent = ctx.spent(&closing);
            if let Err(e) = self.check_budget(
                &jc,
                spent,
                jc.config.budget.producer_usd + jc.config.budget.verifier_usd,
                "producer and verifier together",
            ) {
                self.stop_planning(&ctx, &mut closing, &format!("{e}"));
                return self.close_dispatched(&ctx, closing);
            }
        }
        let closing = self.produce_and_verify(&ctx, &producer_prompt, &accepted, closing)?;
        self.close_dispatched(&ctx, closing)
    }
}

/// What every phase of a dispatched job needs.
struct RunContext<'a> {
    jc: &'a JobConfig,
    job: &'a JobRecords,
    reading: &'a Reading,
    capability: &'a dyn Capability,
    redactor: &'a Redactor,
    prediction: String,
    clock: Instant,
    ui: &'a dyn Ui,
    /// The pinned tree's checkout, mounted read-only into both agents' working directories.
    pin_dir: Option<PathBuf>,
    /// The rejected candidate commit the producer's working directory is seeded from, for a tree-seeded retry.
    seed_commit: Option<String>,
    /// The timed out job whose kept work tree seeds the producer's working directory, for a resumed retry.
    resume: Option<TimeoutSeed>,
}

/// The kept work of a timed out job and the progress note it left.
struct TimeoutSeed {
    job: JobId,
    note: String,
    dir: PathBuf,
}

impl RunContext<'_> {
    fn id(&self) -> &JobId {
        self.job.id()
    }
    fn spent(&self, closing: &Closing) -> f64 {
        self.job.spent_usd()
            + closing
                .new_invocations
                .iter()
                .map(|i| i.cost.best_lower())
                .sum::<f64>()
    }
    /// The number of the next invocation: after the job's recorded ones and those the closing holds.
    fn next_seq(&self, closing: &Closing) -> u32 {
        self.job.next_seq() + closing.new_invocations.len() as u32
    }
}

/// Runs one routed check in the verifier's snapshot, then puts the snapshot back as it was prepared.
struct SnapExec<'a> {
    athena: &'a Athena,
    snapshot: &'a Snapshot,
    prepared: &'a str,
    env: &'a std::collections::BTreeMap<String, String>,
    registry: &'a athena_agent::procs::Registry,
    job: &'a str,
    timeout: std::time::Duration,
    redactor: &'a Redactor,
    cleanup: std::cell::RefCell<Vec<String>>,
}

impl crate::routing::Exec for SnapExec<'_> {
    fn exec(&self, spec: &crate::capability::CheckSpec) -> Result<CheckRun> {
        self.athena.refuse_if_stopped()?;
        let run = run_check(
            spec,
            &CheckContext {
                dir: self.snapshot.dir(),
                env: self.env,
                timeout: self.timeout,
                registry: self.registry,
                job: self.job,
                redactor: self.redactor,
            },
        )?;
        self.cleanup
            .borrow_mut()
            .extend(self.snapshot.restore_to(self.prepared)?);
        Ok(run)
    }
}

/// How many times a report with unmet criteria goes back to the producer before the verifier judges it.
const PRODUCER_SEND_BACKS: u32 = 1;

/// The paths the candidate is expected to change: the accepted plan's files (when the job was planned)
/// and the accepted reading's deliverables, each once, in that order. A path written `./x` is `x`.
pub(crate) fn planned_paths(plan: Option<&JobPlan>, accepted: &AcceptedReading) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    let listed = plan
        .into_iter()
        .flat_map(|p| p.files.iter().map(|f| f.path.as_str()))
        .chain(
            accepted
                .deliverables
                .iter()
                .map(|(path, _, _)| path.as_str()),
        );
    for path in listed {
        let path = normal_path(path);
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    paths
}

/// The planned paths the candidate left unchanged without saying why: not in `changed` (the snapshot's
/// change list, which includes deleted files), and with no entry in the report's `unchanged_planned_files`
/// that names the path and gives a reason. A path is flagged once, in the order of `planned`.
///
/// This is the check that sends a candidate with a forgotten planned file back to the producer before
/// the verifier starts. It reads only paths, never file content.
pub(crate) fn unchanged_planned_files(
    report: &ProducerOutput,
    planned: &[String],
    changed: &[String],
) -> Vec<String> {
    let changed: BTreeSet<String> = changed.iter().map(|p| normal_path(p)).collect();
    planned
        .iter()
        .map(|p| normal_path(p))
        .filter(|path| !changed.contains(path))
        .filter(|path| {
            !report
                .unchanged_planned_files
                .iter()
                .any(|u| normal_path(&u.path) == *path && !u.reason.trim().is_empty())
        })
        .collect()
}

/// A path as compared: surrounding spaces and a leading `./` removed.
fn normal_path(path: &str) -> String {
    let path = path.trim();
    path.strip_prefix("./").unwrap_or(path).to_string()
}

impl Athena {
    /// Runs the capability's configured checks on a fresh snapshot of the base commit and returns the
    /// baseline record. It never fails: a snapshot or environment error, or a nested run, gives a record
    /// with no checks and a note saying why. `before_invocation` is the sequence number the first
    /// producer invocation will get. Verification reads the result to mark pre-existing failures.
    fn take_baseline(&self, ctx: &RunContext<'_>, job_dir: &std::path::Path, seq: u32) -> Baseline {
        let mut baseline = Baseline {
            job: ctx.id().clone(),
            base_commit: ctx.prediction.clone(),
            before_invocation: seq,
            checks: Vec::new(),
            note: None,
        };
        if self.nested.is_some() {
            baseline.note =
                Some("not run: a nested run skips checks; the outer job runs its own".into());
            return baseline;
        }
        let dir = job_dir.join("baseline");
        let outcome = self.baseline_checks(ctx, job_dir, &dir);
        let _ = crate::pin::remove_tree(&dir);
        match outcome {
            Ok(checks) => baseline.checks = checks,
            Err(e) => baseline.note = Some(format!("the baseline could not run: {e:#}")),
        }
        baseline
    }

    fn baseline_checks(
        &self,
        ctx: &RunContext<'_>,
        job_dir: &std::path::Path,
        dir: &std::path::Path,
    ) -> Result<Vec<CheckRun>> {
        let jc = ctx.jc;
        if dir.exists() {
            crate::pin::remove_tree(dir)?;
        }
        let p = ctx.prediction.clone();
        let snapshot = Snapshot::create(&self.git, &p, &p, dir, SnapshotRepo::Base)?;
        if let Some(pin_dir) = &ctx.pin_dir {
            mount(pin_dir, snapshot.dir())?;
        }
        let env = athena_agent::env::build(
            &EnvSpec {
                agents: &jc.config.agents,
                job: ctx.id().as_str(),
                job_dir,
                snapshot: snapshot.dir(),
                side: Side::Verify,
                capability_env: &ctx.capability.env(jc),
            },
            |key| std::env::var(key).ok(),
        )
        .map_err(anyhow::Error::msg)?;
        let registry = self.state.registry();
        Ok(crate::checks::baseline_runs(
            &ctx.capability.checks(jc),
            &CheckContext {
                dir: snapshot.dir(),
                env: &env,
                timeout: std::time::Duration::from_secs(jc.config.budget.check_timeout_s),
                registry: &registry,
                job: ctx.id().as_str(),
                redactor: ctx.redactor,
            },
        ))
    }

    fn produce_and_verify(
        &self,
        ctx: &RunContext<'_>,
        brief: &str,
        accepted: &AcceptedReading,
        mut closing: Closing,
    ) -> Result<Closing> {
        let id = ctx.id().clone();
        let jc = ctx.jc;
        let job_dir = self.state.job_dir(&id);
        let p = ctx.prediction.clone();
        // After any planning invocations the closing already holds.
        let seq = ctx.next_seq(&closing);

        self.baseline_stage(ctx, &job_dir, seq, &mut closing);
        let prepared = self.producer_snapshot(ctx, &job_dir)?;
        let Reported {
            report,
            report_gaps,
            report_kept_in,
            left_criteria,
            seq,
            snapshot,
            work,
        } = match self.producer_loop(ctx, brief, accepted, &mut closing, seq, prepared)? {
            Flow::Go(reported) => reported,
            Flow::Stop => return Ok(closing),
        };

        // The candidate: the snapshot's change list, applied to the base's full tree in the user's repository.
        let changes = snapshot.change_list()?;
        // The chain links (athena.candidate/2): the base's own fingerprint, under its own rules, and the
        // latest job landed in the base's history.
        let base_fingerprint = fingerprint(&self.git, &p, &p)?;
        let previous_job = latest_landed_job(&self.git, &p)?;
        let mut candidate = Candidate {
            job: id.clone(),
            base_commit: p.clone(),
            base_fingerprint: Some(base_fingerprint),
            previous_job,
            candidate_commit: None,
            candidate_tree: None,
            changes: Vec::new(),
            fingerprint: None,
            violations: Vec::new(),
            producer_report: report,
        };
        // The change list is against the base, so it is empty only when the candidate does nothing at all.
        // A retry seeded from a kept candidate already differs from the base: if its producer adds nothing
        // (the kept candidate did the work and only evidence was missing), the seeded candidate is the
        // candidate and goes on to admission, checks and the verifier. Only a run with no change against the
        // base fails as an idle producer.
        if changes.is_empty() {
            closing.candidate = Some(candidate);
            let reason = if report_gaps.is_empty() {
                "the producer made no change".to_string()
            } else {
                format!(
                    "the producer made no change, and its report still had problems when sent back: {}; the report is kept in {report_kept_in}",
                    report_gaps.join("; ")
                )
            };
            closing.happened = self.failure(ctx, &reason);
            return Ok(closing);
        }
        let new_blobs: Vec<String> = changes
            .iter()
            .filter(|c| c.status != 'D')
            .map(|c| c.new_oid.clone())
            .collect();
        let sizes = snapshot.git().blob_sizes(&new_blobs)?;
        let mut edits = Vec::new();
        let mut changed = Vec::new();
        for change in &changes {
            let size = sizes.get(&change.new_oid).copied().unwrap_or(0);
            if change.status == 'D' {
                edits.push(IndexEdit::Remove {
                    path: change.path.clone(),
                });
            } else {
                if change.new_mode != "160000" {
                    let oid = self.git.hash_object_w(&snapshot.blob(&change.new_oid)?)?;
                    debug_assert_eq!(oid, change.new_oid, "blob ids carry over unchanged (E15)");
                }
                edits.push(IndexEdit::Put {
                    mode: change.new_mode.clone(),
                    oid: change.new_oid.clone(),
                    path: change.path.clone(),
                });
            }
            candidate.changes.push(ChangeEntry {
                path: change.path.clone(),
                status: change_status(change.status),
                mode: (change.status != 'D').then(|| change.new_mode.clone()),
                bytes: (change.status != 'D').then_some(size),
            });
            changed.push(Changed {
                path: change.path.clone(),
                status: change_status(change.status),
                mode: change.new_mode.clone(),
                size,
            });
        }
        closing.happened.changed_paths = changes
            .iter()
            .map(|c| c.path.clone())
            .collect::<BTreeSet<_>>();
        let index = TempIndex::from_tree(&self.git, &p)?;
        index.apply(&edits)?;
        let tree = index.write_tree()?;
        let c = self.git.commit_tree(
            &tree,
            &[&p],
            &format!(
                "Candidate for job {id}\n\nNot on any branch; landing builds its own commit.\n"
            ),
            false,
        )?;
        candidate.candidate_commit = Some(c.clone());
        candidate.candidate_tree = Some(tree.clone());

        // Admission: the core's rules, then the capability's.
        let mut violations: Vec<PolicyViolation> = admit(&AdmissionInput {
            git: &self.git,
            base: &p,
            candidate: &c,
            changes: &changed,
            reading: ctx.reading,
            max_file_bytes: jc.config.records.max_file_bytes,
            personal_data: &jc.config.personal_data,
            config: &jc.config,
        })?;
        violations.extend(ctx.capability.admit(
            &ChangeList {
                entries: candidate.changes.clone(),
            },
            jc,
        ));
        let f = fingerprint(&self.git, &c, &p)?;
        candidate.fingerprint = Some(f.clone());
        let stat: Vec<String> = candidate
            .changes
            .iter()
            .map(|e| format!("{} ({})", e.path, format!("{:?}", e.status).to_lowercase()))
            .collect();
        ctx.ui.line(&format!(
            "candidate  {} · fingerprint {}…",
            stat.join(", "),
            &f[..18.min(f.len())]
        ));
        if !violations.is_empty() {
            let summary = violations
                .iter()
                .map(|v| v.detail.clone())
                .collect::<Vec<_>>()
                .join("; ");
            candidate.violations = violations;
            closing.candidate = Some(candidate);
            let changed_paths = std::mem::take(&mut closing.happened.changed_paths);
            closing.happened =
                self.failure(ctx, &format!("admission refused the change: {summary}"));
            closing.happened.changed_paths = changed_paths;
            return Ok(closing);
        }
        closing.candidate = Some(candidate);
        let _ = crate::pin::remove_tree(&work);
        let built = Built {
            base: p,
            commit: c,
            fingerprint: f,
            seq,
            report_gaps,
            left_criteria,
            recorded_checks: None,
        };
        // The candidate is built: from here a job that stops for the environment, or whose process dies, is
        // resumed from it by `athena retry`, with no planner and no producer run.
        self.write_resume_mark(ctx, &closing, &built);
        self.check_and_verify(ctx, accepted, closing, built)
    }

    /// The second half of a job: the disk gate, Athena's own checks, the gate before the verifier, the
    /// verifier and the verdict, for a built candidate. A resumed job at the verifier stage skips the gate
    /// and the checks and uses the recorded results.
    fn check_and_verify(
        &self,
        ctx: &RunContext<'_>,
        accepted: &AcceptedReading,
        mut closing: Closing,
        mut built: Built,
    ) -> Result<Closing> {
        let id = ctx.id().clone();
        let jc = ctx.jc;
        let job_dir = self.state.job_dir(&id);
        let capability_env = ctx.capability.env(jc);
        let p = built.base.clone();
        let c = built.commit.clone();
        let f = built.fingerprint.clone();
        let mut seq = built.seq;
        let report_gaps = built.report_gaps.clone();
        let left_criteria = built.left_criteria.clone();
        let recorded_checks = built.recorded_checks.take();

        // Free disk space, before the checks. Low space makes builds fail in ways that look like faults in the
        // work, so after the idle build directories are removed, space that is still low fails the job with a
        // retryable environment reason. Nothing is held: the job closes, and `athena retry` reads it again.
        if recorded_checks.is_none()
            && let Some(low) = self.disk_gate(jc.config.min_free_gb(), &id, ctx.ui)
        {
            let changed_paths = std::mem::take(&mut closing.happened.changed_paths);
            closing.happened = self.failure(
                ctx,
                &format!(
                    "{} before the checks; this is the environment, not the work. Free disk space, then `athena retry {id}`",
                    low.text()
                ),
            );
            closing.happened.changed_paths = changed_paths;
            closing.happened.verification_passed = None;
            return Ok(closing);
        }

        // Verification starts: open or update the job's pull request now, so the host's checks run on the
        // candidate while the verifier works. It never merges, and a failure is a warning line only.
        self.early_pr(&id, &c, &ctx.reading.title, ctx.ui);

        // The verifier's snapshot: its own two-commit repository, base then candidate, with the base's
        // check-tool configuration in the working tree.
        let verify_dir = job_dir.join("verify");
        let (mut vsnap, tool_config_from_base, prepared) =
            self.verifier_snapshot(&verify_dir, &p, &c, ctx.pin_dir.as_deref())?;
        let check_env = athena_agent::env::build(
            &EnvSpec {
                agents: &jc.config.agents,
                job: id.as_str(),
                job_dir: &job_dir,
                snapshot: vsnap.dir(),
                side: Side::Verify,
                capability_env: &capability_env,
            },
            |key| std::env::var(key).ok(),
        )
        .map_err(anyhow::Error::msg)?;
        let registry = self.state.registry();
        let mut check_runs: Vec<CheckRun> = Vec::new();
        let mut cleanup: Vec<String> = Vec::new();
        let check_started = Instant::now();
        let changed_for_skip: Vec<String> = closing
            .candidate
            .as_ref()
            .map(|c| c.changes.iter().map(|e| e.path.clone()).collect())
            .unwrap_or_default();
        let resumed_checks = recorded_checks.is_some();
        if let Some(runs) = recorded_checks {
            ctx.ui.line(
                "resume     the checks passed on this candidate in the job it resumes; they do not run again",
            );
            check_runs = runs;
        } else if self.nested.is_none() {
            // The plan's kind picks the checks and where each result comes from (`routing`). A study is
            // routed by its capability. Skipping is the path rule, which the route applies.
            let kind = if ctx.capability.id() == "study" {
                "study"
            } else {
                ctx.reading.class_of_work.as_str()
            };
            let route =
                crate::routing::route(Some(kind), ctx.capability.checks(jc), &changed_for_skip);
            if route.workflow_run {
                ctx.ui.line(
                    "routing    build job: the workflow's own run is required and not yet performed",
                );
            }
            let exec = SnapExec {
                athena: self,
                snapshot: &vsnap,
                prepared: &prepared,
                env: &check_env,
                registry: &registry,
                job: id.as_str(),
                timeout: std::time::Duration::from_secs(jc.config.budget.check_timeout_s),
                redactor: ctx.redactor,
                cleanup: std::cell::RefCell::new(Vec::new()),
            };
            // No candidate commit is pushed yet, so no CI result exists to read: one read, no wait.
            check_runs = crate::routing::run_routed(
                &route,
                &c,
                &changed_for_skip,
                &crate::routing::NoCi,
                &exec,
                crate::ci_result::Wait {
                    polls: 1,
                    pause: std::time::Duration::ZERO,
                },
            )?;
            cleanup.extend(exec.cleanup.into_inner());
        } else {
            for spec in ctx.capability.checks(jc) {
                check_runs.push(CheckRun {
                    name: spec.name.clone(),
                    command: spec.run.clone(),
                    exit_code: None,
                    passed: true,
                    tests_ran: None,
                    duration: athena_data::Quantity::measured(
                        0.0,
                        athena_data::Unit::Seconds,
                        "not run",
                    ),
                    output_tail: String::new(),
                    note: Some(
                        "not run: a nested run skips checks; the outer job runs its own".into(),
                    ),
                });
            }
        }
        // A check that failed on this job's base commit and fails again is marked pre-existing. A baseline
        // taken on any other commit says nothing about this base.
        let usable_baseline = closing
            .baseline
            .as_ref()
            .filter(|b| b.base_commit == p.as_str());
        if !resumed_checks {
            let checks_line: Vec<String> = check_runs
                .iter()
                .map(|r| match (r.passed, r.tests_ran) {
                    (true, Some(n)) => format!("{} ok ({n} tests)", r.name),
                    (true, None) => format!("{} ok", r.name),
                    (false, _) => format!("{} FAILED", r.name),
                })
                .collect();
            ctx.ui.line(&format!(
                "checks     {} ({:.0}s)",
                checks_line.join(" · "),
                check_started.elapsed().as_secs_f64()
            ));
            mark_pre_existing(&mut check_runs, usable_baseline);
            // Every check passed: the producer's build directory is idle, and it is the largest thing on the
            // disk, so it goes now, before the gate that runs before the verifier. When a check failed it
            // stays, since the work may be sent back to the producer.
            if check_runs.iter().all(|r| r.passed) {
                match self.state.remove_produce(&id) {
                    Ok(0) => {}
                    Ok(bytes) => ctx.ui.line(&format!(
                        "disk       removed the producer's build directory: {}",
                        crate::state::format_bytes(bytes)
                    )),
                    Err(why) => ctx.ui.line(&format!(
                        "warning: could not remove the producer's build directory: {why}"
                    )),
                }
                // The checks passed: a stop from here resumes at the verifier, with these results.
                built.recorded_checks = Some(check_runs.clone());
                self.write_resume_mark(ctx, &closing, &built);
            }
        }

        // The replay of the newest accepted readings through the base and candidate reading rules. It runs
        // after the checks and the removal of the producer's build directory, and before the gate that
        // measures the disk for the verifier, because it builds twice and removes what it built. It calls no
        // model. A held replay fails the verdict (`judge`); the record is kept either way.
        let replay_record = self.replay_stage(ctx, &id, &p, &c, &check_runs);
        closing
            .files
            .push(NewFile::record(&AnyRecord::Replay(Box::new(
                replay_record.clone(),
            ))));

        let criteria = ctx.reading.acceptance_criteria.clone();
        let verifier_grant = ctx.capability.verifier_grant(jc);
        let candidate_view = CandidateView {
            base: p.clone(),
            candidate: c.clone(),
            stat: vsnap.git().run(["diff", "--stat", "HEAD~1", "HEAD"])?,
            diff: vsnap.git().run(["diff", "HEAD~1", "HEAD"])?,
        };
        let mut inputs = VerifierInputs {
            job: id.as_str().to_string(),
            request: ctx
                .job
                .request
                .as_ref()
                .map(|r| r.text.clone())
                .unwrap_or_default(),
            reading: accepted.clone(),
            candidate: candidate_view,
            checks: check_runs
                .iter()
                .map(|r| CheckResultView {
                    name: r.name.clone(),
                    command: r.command.join(" "),
                    passed: r.passed,
                    detail: match (r.tests_ran, &r.note) {
                        (_, Some(note)) => note.clone(),
                        (Some(n), None) => format!("{n} tests ran"),
                        (None, None) => String::new(),
                    },
                })
                .collect(),
            commands: athena_agent::policy::describe_commands(&verifier_grant.commands),
            tool_config_from_base: tool_config_from_base.clone(),
            pin: ctx.job.job.as_ref().and_then(|j| j.pin.clone()),
            replay: Some(replay_view(&replay_record, ctx.reading)),
            // Change verification asks the soundness question; a study's verifier brief does not.
            soundness: ctx.capability.id() != "study",
        };
        // What the producer's report still lacked after the send-back goes to the verifier as one more
        // entry under the checks. It is not a check and fails nothing; it names what the self-check
        // could not confirm, so the verifier judges those items itself. Only the gaps are passed, not the report.
        if !report_gaps.is_empty() {
            inputs.checks.push(CheckResultView {
                name: "producer-report-gaps".into(),
                command: "not a check".into(),
                passed: true,
                detail: format!(
                    "the producer's report was still incomplete after its send-back, so these were not confirmed \
                     by it; judge them yourself from the candidate and the recorded check results: {}",
                    report_gaps.join("; ")
                ),
            });
        }
        // Criteria the producer left to Athena's checks go the same way: not a check, fails nothing, and
        // the recorded check results still decide the verdict.
        if !left_criteria.is_empty() {
            inputs.checks.push(CheckResultView {
                name: "criteria-left-to-athena-checks".into(),
                command: "not a check".into(),
                passed: true,
                detail: format!(
                    "the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test \
                     suite); judge them yourself from the recorded check results: {}",
                    left_criteria.join("; ")
                ),
            });
        }
        // The verifier sees the accepted plan's demonstrations and planned files, never its approach or
        // reasons (review F5). Without a plan the brief is as it always was.
        let verifier_plan = closing.plans.last().map(VerifierPlan::from_plan);

        // The environment, immediately before the verifier agent runs. The checks above build and fill the
        // disk after the earlier gate passed, and a verifier started on a full disk fails in ways that read
        // as faults in the work. So the same gate (with its idle-build cleanup) runs again, and a small file
        // is written and removed in the verifier's snapshot. A problem closes the job with the retryable
        // `environment: low disk` reason and no verdict; the verifier is not started and its brief is not
        // recorded.
        if let Some(problem) = crate::disk::verifier_environment_problem(
            &self.state,
            jc.config.min_free_gb(),
            &id,
            vsnap.dir(),
            ctx.ui,
        ) {
            let changed_paths = std::mem::take(&mut closing.happened.changed_paths);
            closing.happened =
                self.failure(ctx, &crate::disk::before_verifier_reason(&problem, &id));
            closing.happened.changed_paths = changed_paths;
            closing.happened.verification_passed = None;
            let _ = crate::pin::remove_tree(&verify_dir);
            return Ok(closing);
        }
        let mut verifier_invocation = None;
        let mut verdict_output: Option<VerdictOutput> = None;
        let mut commands_run = Vec::new();
        let mut verifier_failure = None;
        // Set when the verifier changed its snapshot: the system failed to keep it still, so the work is
        // not at fault. The verdict is discarded and verification reruns once on a fresh snapshot.
        let mut system_fault: Option<String> = None;
        // The verifier always runs, so a failed check never leaves the criteria unjudged; the check still
        // fails the final verdict unless it is pre-existing.
        {
            closing.files.push(NewFile::text(
                format!("{}/brief-verifier.md", id.dir()),
                verifier_brief_with_plan(&inputs, verifier_plan.as_ref(), false),
            ));
            let vcomposed = compose(&[
                CONTRACT.into(),
                VERIFIER_FRAME.into(),
                Component::from(ctx.capability.verifier_section()),
            ]);
            let vmodel = ctx.reading.verifier_model.clone();
            let mut verifier_runs = 0;
            loop {
                verifier_runs += 1;
                let spent = ctx.spent(&closing);
                if let Err(e) =
                    self.check_budget(jc, spent, jc.config.budget.verifier_usd, "verifier")
                {
                    verifier_failure = Some(format!("{e}"));
                    break;
                }
                let vbrief = verifier_brief_with_plan(&inputs, verifier_plan.as_ref(), true);
                let vspec = self.agent_spec(
                    jc,
                    &id,
                    seq,
                    Role::Verifier,
                    &vmodel,
                    &vcomposed,
                    vbrief,
                    &verifier_grant,
                    vsnap.dir(),
                    Side::Verify,
                    &capability_env,
                    jc.config.budget.verifier_usd,
                    jc.config.budget.verifier_timeout_s,
                    verdict_schema(),
                )?;
                let vfacts = InvocationFacts {
                    job: &id,
                    spec: &vspec,
                    composed: &vcomposed,
                    config: &jc.config,
                    fake: self.is_fake(),
                };
                seq += 1;
                // The snapshot refuses writes while the verifier runs, even from root; the grant's write
                // deny stops its tools, and the lock stops everything else, such as a test it runs.
                if let Err(e) = vsnap.lock() {
                    let _ = vsnap.unlock();
                    verifier_failure =
                        Some(format!("the verifier's snapshot could not be locked: {e}"));
                    break;
                }
                let invoked = self.invoke_and_record(&vfacts, ctx.redactor);
                vsnap.unlock()?;
                match invoked? {
                    Err(error) => {
                        verifier_failure = Some(format!("the verifier could not run: {error}"))
                    }
                    Ok((voutput, vrecord)) => {
                        verifier_invocation = Some(vrecord.seq);
                        commands_run = voutput.commands_run.clone();
                        if voutput.outcome != InvocationOutcome::Success {
                            verifier_failure = Some(
                                vrecord
                                    .detail
                                    .clone()
                                    .unwrap_or_else(|| "the verifier did not complete".into()),
                            );
                        }
                        verdict_output = voutput
                            .structured
                            .clone()
                            .and_then(|v| serde_json::from_value(v).ok());
                        if verdict_output.is_none() && verifier_failure.is_none() {
                            verifier_failure =
                                Some("the verifier returned no valid verdict".into());
                        }
                        ctx.ui.line(&format!(
                            "verifier   {vmodel} · {:.0}s · {} · {}",
                            voutput.wall_s,
                            cost_text(&vrecord),
                            verdict_output
                                .as_ref()
                                .map(|v| format!("{:?}", v.verdict).to_lowercase())
                                .unwrap_or_else(|| "no verdict".into())
                        ));
                        closing.new_invocations.push(vrecord);
                    }
                }
                // A verifier that changed its snapshot invalidates its own verdict. That is a fault of the
                // system, which should have kept the snapshot still, not of the work: discard the verdict
                // and run the verifier once more on a fresh snapshot.
                let head_now = vsnap.git().head()?;
                let tree_now = vsnap.work_tree_tree()?;
                if head_now.as_deref() == vsnap.candidate() && tree_now == prepared {
                    break;
                }
                let change = format!(
                    "snapshot HEAD {:?}, work tree {} (expected {})",
                    head_now,
                    short(&tree_now),
                    short(&prepared)
                );
                verdict_output = None;
                verifier_failure = None;
                commands_run.clear();
                verifier_invocation = None;
                system_fault = Some(change.clone());
                if verifier_runs >= 2 {
                    verifier_failure = Some(format!(
                        "the verifier's snapshot changed on both runs, so no verdict could be trusted ({change})"
                    ));
                    break;
                }
                ctx.ui.line("fault      the verifier's snapshot changed; its verdict is discarded and verification reruns once");
                let (fresh, _, fresh_tree) =
                    self.verifier_snapshot(&verify_dir, &p, &c, ctx.pin_dir.as_deref())?;
                if fresh_tree != prepared {
                    bail!("a fresh verifier snapshot differs from the first one");
                }
                vsnap = fresh;
            }
        }

        // Redact the verifier's free text before it is recorded.
        if let Some(output) = verdict_output.as_mut() {
            let mut text: Vec<String> = vec![
                output.summary.clone(),
                output.request_reading_mismatch.clone().unwrap_or_default(),
            ];
            for c in &output.criteria {
                text.push(c.evidence.clone());
            }
            for f in &output.findings {
                text.push(f.summary.clone());
                text.push(f.evidence.clone());
                text.push(f.location.clone().unwrap_or_default());
            }
            ctx.redactor.redact_all(&mut text)?;
            let mut it = text.into_iter();
            output.summary = it.next().unwrap_or_default();
            let mismatch = it.next().unwrap_or_default();
            output.request_reading_mismatch = (!mismatch.trim().is_empty()).then_some(mismatch);
            for c in output.criteria.iter_mut() {
                c.evidence = it.next().unwrap_or_default();
            }
            for f in output.findings.iter_mut() {
                f.summary = it.next().unwrap_or_default();
                f.evidence = it.next().unwrap_or_default();
                let location = it.next().unwrap_or_default();
                f.location = (!location.trim().is_empty()).then_some(location);
            }
        }
        let mut redacted_commands = commands_run.clone();
        ctx.redactor.redact_all(&mut redacted_commands)?;
        // The verifier's snapshot holds the base, then the candidate, as two commits.
        let map_at = |rev: &str| -> Result<Option<String>> {
            Ok(vsnap
                .git()
                .show_file(rev, MAP_PATH)?
                .map(|b| String::from_utf8_lossy(&b).into_owned()))
        };
        let map_base = map_at("HEAD~1")?;
        let map_candidate = map_at("HEAD")?;
        let judgment = judge(&VerdictInputs {
            criteria: &criteria,
            checks: &check_runs,
            output: verdict_output.as_ref(),
            commands_run: &redacted_commands,
            candidate_tree: Some(c.as_str()),
            // A nested run records "not run" placeholders, not results.
            checks_tree: self.nested.is_none().then_some(c.as_str()),
            system_fault: system_fault.clone(),
            verifier_failure: verifier_failure.clone(),
            map_base: map_base.as_deref(),
            map_candidate: map_candidate.as_deref(),
            baseline: usable_baseline,
            replay: Some(&replay_record),
        });
        let verification = Verification {
            job: id.clone(),
            candidate_commit: c.clone(),
            verified_fingerprint: f.clone(),
            tool_config_from_base,
            checks: check_runs,
            check_cleanup: cleanup,
            verifier_invocation,
            criteria: judgment.criteria.clone(),
            findings: judgment.findings.clone(),
            request_reading_mismatch: verdict_output
                .as_ref()
                .and_then(|v| v.request_reading_mismatch.clone()),
            agent_verdict: judgment.agent_verdict,
            final_verdict: judgment.final_verdict,
            reasons: judgment.reasons.clone(),
            inputs: inputs.names_with_plan(verifier_plan.as_ref()),
            summary: verdict_output.as_ref().map(|v| v.summary.clone()),
        };
        let _ = crate::pin::remove_tree(&verify_dir);
        closing.happened.verification_passed = Some(judgment.final_verdict == Verdict::Pass);
        closing.happened.wall_time_execution_s = Some(ctx.clock.elapsed().as_secs_f64());
        closing.happened.state = Some(if judgment.final_verdict == Verdict::Pass {
            OutcomeState::Landed
        } else {
            OutcomeState::Rejected
        });
        closing.verified = verified(&judgment, &p, &c, &f);
        closing.verification = Some(verification);
        Ok(closing)
    }

    /// The replay record of this job: the newest accepted readings run through the base and candidate builds'
    /// reading rules. It never fails the job and never passes silently: when the replay cannot run, the record
    /// says `no replay: <reason>`. Everything the replay builds is removed before this returns.
    fn replay_stage(
        &self,
        ctx: &RunContext<'_>,
        id: &JobId,
        base: &str,
        candidate: &str,
        checks: &[CheckRun],
    ) -> Replay {
        let run = self.replay_run(ctx, id, base, candidate, checks);
        ctx.ui.line(&match &run.no_replay {
            Some(reason) => format!("replay     {reason}"),
            None => format!(
                "replay     {} of {} accepted readings newly refused{}",
                run.newly_refused.len(),
                run.total,
                if run.held { " (held)" } else { "" }
            ),
        });
        run.record(id, base, candidate)
    }

    fn replay_run(
        &self,
        ctx: &RunContext<'_>,
        id: &JobId,
        base: &str,
        candidate: &str,
        checks: &[CheckRun],
    ) -> ReplayRun {
        if !checks.iter().all(|r| r.passed) {
            return ReplayRun::none("a check failed");
        }
        let records = match self.head().and_then(|head| Records::load(&self.git, &head)) {
            Ok(records) => records,
            Err(error) => {
                return ReplayRun::none(&format!("the records could not be read: {error:#}"));
            }
        };
        let cases = replay_cases(&records, Some(id));
        if cases.is_empty() {
            return ReplayRun::none(NO_HISTORY_REASON);
        }
        let dir = self.state.job_dir(id).join("replay");
        let timeout = std::time::Duration::from_secs(ctx.jc.config.budget.check_timeout_s);
        let run = match self.replay_checkers(base, candidate, &dir, timeout) {
            Ok((base_checker, candidate_checker)) => replay(
                &cases,
                &*base_checker,
                &*candidate_checker,
                replay_declaration_of(ctx.reading).effective_share(),
            ),
            Err(reason) => ReplayRun::none(&reason),
        };
        let _ = crate::pin::remove_tree(&dir);
        run
    }

    /// The base and the candidate reading rules as checkers. In tests the seam comes first, before any other
    /// step. Otherwise each is a build of `athena` from that commit's visible files, in `dir`, probed for the
    /// `check-reading` subcommand. A nested run, a fake driver, a missing package, a failed build or a missing
    /// subcommand is an `Err` with the reason: `no replay`, never a pass.
    fn replay_checkers(
        &self,
        base: &str,
        candidate: &str,
        dir: &std::path::Path,
        timeout: std::time::Duration,
    ) -> std::result::Result<ReplayCheckers, String> {
        #[cfg(test)]
        if let Some(pair) = crate::replay_seam::current() {
            return Ok(pair);
        }
        if self.nested.is_some() {
            return Err("a nested run builds nothing".into());
        }
        if self.is_fake() {
            return Err("the driver is a fake, so nothing is built".into());
        }
        let target = dir.join("target");
        let mut checkers: Vec<std::rc::Rc<dyn ReadingChecker>> = Vec::new();
        for (side, rev) in [("base", base), ("candidate", candidate)] {
            let tree = dir.join(format!("{side}-tree"));
            crate::pin::remove_tree(&tree)
                .map_err(|e| format!("the {side} tree could not be cleared: {e}"))?;
            std::fs::create_dir_all(&tree)
                .map_err(|e| format!("the {side} tree could not be made: {e}"))?;
            athena_git::snapshot::materialize(&self.git, rev, rev, &tree)
                .map_err(|e| format!("the {side} tree could not be written: {e:#}"))?;
            if !tree.join("crates/athena-cli/Cargo.toml").is_file() {
                return Err(format!(
                    "the {side} tree has no athena-cli package to build"
                ));
            }
            let mut build = std::process::Command::new("cargo");
            build
                .args(["build", "-p", "athena-cli", "--bin", "athena", "--locked"])
                .current_dir(&tree)
                .env("CARGO_TARGET_DIR", &target)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
            run_bounded(&mut build, timeout)
                .map_err(|e| format!("the {side} build failed: {e}"))?;
            let bin = dir.join(format!("athena-{side}"));
            std::fs::copy(target.join("debug/athena"), &bin)
                .map_err(|e| format!("the {side} build left no binary: {e}"))?;
            let mut probe = std::process::Command::new(&bin);
            probe
                .args(["check-reading", "--help"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null());
            run_bounded(&mut probe, timeout)
                .map_err(|_| format!("the {side} build has no check-reading subcommand"))?;
            checkers.push(std::rc::Rc::new(BinaryChecker::new(bin)));
        }
        let candidate_checker = checkers.pop().expect("two checkers");
        let base_checker = checkers.pop().expect("two checkers");
        Ok((base_checker, candidate_checker))
    }

    /// Creates the verifier's snapshot at `dir` (removing what is there): base then candidate as two commits,
    /// the pinned tree mounted if there is one, and the base's check-tool configuration in the work tree.
    /// Returns it with the tool-config paths and the work tree's tree as prepared.
    fn verifier_snapshot(
        &self,
        dir: &std::path::Path,
        base: &str,
        candidate: &str,
        pin_dir: Option<&std::path::Path>,
    ) -> Result<(Snapshot, Vec<String>, String)> {
        crate::pin::remove_tree(dir)?;
        let snapshot = Snapshot::create(
            &self.git,
            base,
            base,
            dir,
            SnapshotRepo::BaseAndCandidate {
                candidate_commit: candidate.to_string(),
            },
        )?;
        if let Some(pin_dir) = pin_dir {
            mount(pin_dir, snapshot.dir())?;
        }
        let tool_config = crate::toolconfig::apply_base_tool_config(&snapshot)?;
        let prepared = snapshot.work_tree_tree()?;
        Ok((snapshot, tool_config, prepared))
    }

    /// Marks a failure before verification.
    fn failure(&self, ctx: &RunContext<'_>, reason: &str) -> Happened {
        ctx.ui.line(&format!("failed     {reason}"));
        Happened {
            verification_passed: Some(false),
            changed_paths: BTreeSet::new(),
            wall_time_execution_s: Some(ctx.clock.elapsed().as_secs_f64()),
            state: Some(OutcomeState::Failed),
            reason: None,
        }
        .with_reason(reason)
    }

    fn redact_report(&self, report: ProducerOutput, redactor: &Redactor) -> Result<ProducerReport> {
        let mut text = vec![report.summary.clone()];
        let lists = [
            &report.open_issues,
            &report.mistakes,
            &report.learnings,
            &report.desires,
        ];
        for list in lists {
            text.extend(list.iter().cloned());
        }
        for check in &report.checks_run {
            text.push(check.command.clone());
            text.push(check.note.clone().unwrap_or_default());
        }
        for check in &report.criteria_check {
            text.push(check.criterion.clone());
            text.push(check.evidence.clone());
        }
        redactor.redact_all(&mut text)?;
        let mut it = text.into_iter();
        let summary = it.next().unwrap_or_default();
        let mut take = |n: usize| -> Vec<String> { it.by_ref().take(n).collect() };
        let open_issues = take(report.open_issues.len());
        let mistakes = take(report.mistakes.len());
        let learnings = take(report.learnings.len());
        let desires = take(report.desires.len());
        let mut checks_run = Vec::new();
        for check in &report.checks_run {
            let command = take(1).pop().unwrap_or_default();
            let note = take(1).pop().unwrap_or_default();
            checks_run.push(ReportedCheck {
                command,
                result: check.result.as_str().into(),
                note: (!note.trim().is_empty()).then_some(note),
            });
        }
        let mut criteria_check = Vec::new();
        for check in &report.criteria_check {
            let criterion = take(1).pop().unwrap_or_default();
            let evidence = take(1).pop().unwrap_or_default();
            criteria_check.push(ReportedCriterion {
                criterion,
                status: check.status.as_str().into(),
                evidence,
            });
        }
        Ok(ProducerReport {
            summary,
            files: report.files,
            checks_run,
            criteria_check,
            open_issues,
            mistakes,
            learnings,
            desires,
        })
    }
}

impl Happened {
    fn with_reason(mut self, reason: &str) -> Happened {
        self.reason = Some(reason.to_string());
        self
    }
}

fn cost_text(record: &Invocation) -> String {
    match record.cost.value() {
        Some(v) => format!("${v:.2}"),
        None => format!("≤ ${:.2} (estimated)", record.cost.upper().unwrap_or(0.0)),
    }
}

#[cfg(test)]
mod critic_soundness_tests {
    use super::*;
    use serde_json::json;

    const SOUNDNESS_SUMMARY: &str =
        "Soundness: the refusal rule matches common words, so it has false positives";

    /// A critic output as the model returns it, parsed as the production round parses it, with one
    /// soundness finding at `severity`.
    fn critic_output(verdict: &str, severity: &str) -> CriticOutput {
        serde_json::from_value(json!({
            "verdict": verdict,
            "findings": [{
                "severity": severity,
                "summary": SOUNDNESS_SUMMARY,
                "location": "Approach",
                "evidence": "The rule matches words that common requests contain.",
                "fix": "Read the request's structure instead of matching words."
            }],
            "request_reading_mismatch": null,
            "summary": "A crude heuristic."
        }))
        .unwrap()
    }

    fn verdict_for(verdict: &str, severity: &str) -> CritiqueVerdict {
        let output = critic_output(verdict, severity);
        let findings = critic_findings(Vec::new(), &output);
        plan_verdict(CritiqueVerdict::Accept, &findings)
    }

    #[test]
    fn soundness_finding_blocker_makes_the_plan_revise() {
        let output = critic_output("accept", "blocker");
        let findings = critic_findings(Vec::new(), &output);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Blocker);
        assert_eq!(findings[0].source, FindingSource::Critic);
        assert!(findings[0].summary.starts_with("Soundness:"));
        // The critic said accept; the blocker finding overrides it.
        assert_eq!(
            plan_verdict(CritiqueVerdict::Accept, &findings),
            CritiqueVerdict::Revise
        );
        assert_eq!(verdict_for("accept", "major"), CritiqueVerdict::Revise);
    }

    #[test]
    fn soundness_finding_minor_or_nit_leaves_the_plan_accepted() {
        assert_eq!(verdict_for("accept", "minor"), CritiqueVerdict::Accept);
        assert_eq!(verdict_for("accept", "nit"), CritiqueVerdict::Accept);
    }

    #[test]
    fn soundness_finding_does_not_soften_the_critics_own_verdict() {
        let findings = critic_findings(Vec::new(), &critic_output("revise", "minor"));
        assert_eq!(
            plan_verdict(CritiqueVerdict::Revise, &findings),
            CritiqueVerdict::Revise
        );
        assert_eq!(
            plan_verdict(CritiqueVerdict::Reject, &findings),
            CritiqueVerdict::Reject
        );
    }
}

/// Characterization of the order `produce_and_verify` works in (structure pass, R0): a fake job goes through
/// `Athena::go` and the test reads what the UI said and which invocation numbers were recorded. It guards the
/// moves of the produce and verify code against a change of order.
#[cfg(test)]
mod produce_and_verify_order_tests {
    use super::*;
    use crate::Quiet;
    use crate::capability::{CapabilityCard, CheckSpec, ToolGrant};
    use athena_agent::fake::{FakeAgentDriver, FakeTurn};
    use athena_agent::{AgentInvocation, Usage};
    use athena_data::config::CheckKind;
    use athena_data::records::PolicyViolation;
    use athena_git::Git;
    use serde_json::json;
    use std::collections::BTreeMap;
    use std::sync::{Arc, Mutex};

    const CRITERION: &str = "README.md describes the demo";

    /// A capability that edits files and runs one configured check, so the baseline has something to say.
    /// The check is the command `check` (`true` passes, `false` fails).
    struct Docs {
        check: &'static [&'static str],
    }

    /// A check that passes on any tree.
    const PASSES: &[&str] = &["true"];
    /// A check that passes on the base README and fails once the README is edited.
    const FAILS_ON_EDIT: &[&str] = &["sh", "-c", "test \"$(cat README.md)\" = '# Demo'"];

    impl Capability for Docs {
        fn id(&self) -> &'static str {
            "study"
        }
        fn card(&self) -> CapabilityCard {
            CapabilityCard {
                id: "study",
                purpose: "describe a demo.",
                when: "for a request to describe a demo.",
                produces: "files describing the demo.",
            }
        }
        fn producer_section(&self) -> athena_context::PromptAsset {
            PRODUCER_FRAME
        }
        fn verifier_section(&self) -> athena_context::PromptAsset {
            VERIFIER_FRAME
        }
        fn producer_grant(&self, _: &JobConfig) -> ToolGrant {
            ToolGrant {
                tools: vec!["Read".into()],
                commands: BTreeMap::new(),
                writes: true,
                permission_mode: "acceptEdits".into(),
            }
        }
        fn verifier_grant(&self, _: &JobConfig) -> ToolGrant {
            ToolGrant {
                tools: vec!["Read".into()],
                commands: BTreeMap::new(),
                writes: false,
                permission_mode: "dontAsk".into(),
            }
        }
        fn checks(&self, _: &JobConfig) -> Vec<CheckSpec> {
            vec![CheckSpec {
                name: "probe".into(),
                run: self.check.iter().map(|part| part.to_string()).collect(),
                kind: CheckKind::Command,
                skip_when_only: None,
            }]
        }
        fn env(&self, _: &JobConfig) -> BTreeMap<String, String> {
            BTreeMap::new()
        }
        fn admit(&self, _: &ChangeList, _: &JobConfig) -> Vec<PolicyViolation> {
            Vec::new()
        }
    }

    /// A UI that keeps every line, in order.
    #[derive(Default)]
    struct Lines(Mutex<Vec<String>>);

    impl Ui for Lines {
        fn line(&self, text: &str) {
            self.0.lock().unwrap().push(text.to_string());
        }
    }

    fn athena(repo: &tempfile::TempDir, state: &tempfile::TempDir) -> Athena {
        athena_with(
            repo,
            state,
            PASSES,
            crate::test_support::plenty_of_space,
            base_script(),
        )
    }

    /// An Athena over `repo` whose capability checks with the command `check`, whose free space is read by
    /// `free_space`, and whose agents follow `script`.
    fn athena_with(
        repo: &tempfile::TempDir,
        state: &tempfile::TempDir,
        check: &'static [&'static str],
        free_space: crate::state::FreeSpace,
        script: athena_agent::fake::Script,
    ) -> Athena {
        let state =
            crate::state::StateDir::new(state.path().join("repo")).with_free_space(free_space);
        let driver = FakeAgentDriver::new(script).with_registry(state.registry());
        Athena {
            git: Git::at(repo.path()),
            state,
            driver: std::sync::Arc::new(driver),
            capabilities: vec![Box::new(Docs { check })],
            build: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            athena_bin: "/bin/false".into(),
            nested: None,
            pinner: Box::new(crate::fake::CountingPinner::new(&"a".repeat(40))),
            jev: None,
        }
    }

    /// The fake agents of this repository: a reader, a producer that edits the README, and a verifier that
    /// passes it.
    fn base_script() -> athena_agent::fake::Script {
        Box::new(|spec: &AgentInvocation| {
            let mut turn = match spec.role {
                Role::Reader => FakeTurn::output(json!({
                    "title": "Describe the demo",
                    "summary": "Describe the demo in README.md.",
                    "class_of_work": "docs",
                    "capability": "study",
                    "decline_reason": null,
                    "producer_model": "claude-haiku-5-5",
                    "defaults_applied": [{ "id": "strangers", "how": "plain words" }],
                    "assumptions": [],
                    "question": null,
                    "acceptance_criteria": [{ "text": CRITERION, "kind": "content" }],
                    "deliverables": [{ "path": "README.md", "action": "modify", "p": 0.9 }],
                    "out_of_scope": ["Other files"],
                    "forecast": {
                        "p_reading_accepted": 0.8,
                        "p_verification_passes": 0.7,
                        "p_user_intervenes": 0.1,
                        "cost_usd": { "p10": 0.05, "p50": 0.2, "p90": 0.8 },
                        "wall_time_s": { "p10": 5.0, "p50": 30.0, "p90": 300.0 },
                        "departures": null
                    }
                })),
                Role::Producer => {
                    let mut t = FakeTurn::output(json!({
                        "summary": "Described the demo.",
                        "files": ["README.md"],
                        "checks_run": [],
                        "criteria_check": [
                            { "criterion": CRITERION, "status": "met", "evidence": "read it" }
                        ],
                        "open_issues": [], "mistakes": [], "learnings": [], "desires": []
                    }));
                    t.edits = vec![("README.md".into(), Some("# Demo\n\nA demo.\n".into()))];
                    t
                }
                Role::Verifier => FakeTurn::output(json!({
                    "verdict": "pass",
                    "criteria": [
                        { "criterion": CRITERION, "met": true, "evidence": "checked", "commands": [] }
                    ],
                    "findings": [],
                    "request_reading_mismatch": null,
                    "summary": "Checked."
                })),
                Role::Planner | Role::Critic => unreachable!("planning is off in this repository"),
                Role::Steward => FakeTurn::output(json!({ "items": [] })),
                Role::Audience => FakeTurn::output(json!({ "rating": 3, "findings": [] })),
                Role::Auditor => FakeTurn::output(json!({ "summary": "Fine.", "findings": [] })),
            };
            turn.cost_usd = 0.01;
            turn.usage = Usage {
                input: 1,
                output: 1,
                cache_read: 1,
                cache_write: 1,
            };
            turn
        })
    }

    fn repo() -> tempfile::TempDir {
        // Without `[job_plan]` (a job with no planning round) and without a pinned repository.
        let raw = include_str!("../../../../athena.toml");
        let mut in_job_plan = false;
        let config: String = raw
            .split_inclusive('\n')
            .filter(|line| {
                let key = line.trim_start();
                if key.starts_with('[') {
                    in_job_plan = key.trim_end() == "[job_plan]";
                }
                !(in_job_plan || key.starts_with("repo_url") || key.starts_with("repo_ref"))
            })
            .collect();
        let dir = tempfile::tempdir().unwrap();
        let git = Git::at(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value]).unwrap();
        }
        for (path, text) in [
            ("athena.toml", config.as_str()),
            ("defaults.toml", include_str!("../../../../defaults.toml")),
            ("README.md", "# Demo\n"),
        ] {
            std::fs::write(dir.path().join(path), text).unwrap();
        }
        git.run(["add", "-A"]).unwrap();
        git.run([
            "commit",
            "-q",
            "-m",
            "Start a demo repository\n\nAthena-Job: bootstrap",
        ])
        .unwrap();
        dir
    }

    /// Commits `text` at `path` in the test repository.
    fn commit_file(repo: &tempfile::TempDir, path: &str, text: &str) {
        let git = Git::at(repo.path());
        let full = repo.path().join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, text).unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "Add a file\n\nAthena-Job: bootstrap"])
            .unwrap();
    }

    #[test]
    fn a_request_with_a_banned_word_is_refused_before_admission_and_a_clean_one_is_admitted() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let athena = athena(&repo, &state);
        let before = athena.head().unwrap();
        // Built from fragments so this file passes the scan it tests.
        let banned = ["oper", "ator"].concat();
        let request = format!("describe the demo\nthen ask the {banned} to check it");
        for refused in [
            athena.ask(&request, &Quiet).unwrap_err().to_string(),
            athena.queue(&request).unwrap_err().to_string(),
        ] {
            assert!(refused.contains("not admitted"), "{refused}");
            assert!(
                refused.contains(&format!("request:2: {banned}")),
                "{refused}"
            );
        }
        // Nothing was committed and no model was asked: no job exists.
        assert_eq!(athena.head().unwrap(), before);

        // The same request, reworded, passes the scan and is admitted.
        let proposal = athena
            .ask("describe the demo\nthen check it", &Quiet)
            .unwrap();
        assert!(proposal.reading.is_some());
        assert_ne!(athena.head().unwrap(), before);
    }

    #[test]
    fn a_named_file_with_a_banned_word_is_flagged_with_path_and_line_and_a_clean_one_is_not() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let athena = athena(&repo, &state);
        let banned = ["sand", "box"].concat();
        commit_file(
            &repo,
            "docs/dirty.md",
            &format!("# Notes\n\nIt ran in a {banned}.\n"),
        );
        commit_file(&repo, "docs/clean.md", "# Notes\n\nNothing to see.\n");
        let before = athena.head().unwrap();

        let err = athena
            .ask("tidy up docs/dirty.md", &Quiet)
            .unwrap_err()
            .to_string();
        assert!(err.contains("not admitted"), "{err}");
        assert!(err.contains(&format!("docs/dirty.md:3: {banned}")), "{err}");
        assert_eq!(athena.head().unwrap(), before);

        // The scan itself: request text first, then each named file; a path that does not exist is skipped.
        let findings = crate::reading::scan_request_input(
            &athena.git,
            &before,
            "tidy docs/clean.md and docs/dirty.md and docs/missing.md",
        )
        .unwrap();
        assert_eq!(
            findings,
            vec![crate::reading::InputFinding {
                source: "docs/dirty.md".into(),
                line: 3,
                term: banned
            }]
        );

        // A clean named file is admitted.
        let proposal = athena.ask("tidy up docs/clean.md", &Quiet).unwrap();
        assert!(proposal.reading.is_some());
    }

    #[test]
    fn a_fake_job_takes_its_baseline_before_the_producer_and_numbers_the_invocations_in_order() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let athena = athena(&repo, &state);
        let proposal = athena.ask("describe the demo", &Quiet).unwrap();
        let ui = Lines::default();
        let GoResult::Finished(finished) = athena
            .go(proposal.job.as_str(), AcceptedBy::User, &ui)
            .unwrap()
        else {
            panic!("expected the job to finish");
        };
        assert_eq!(finished.state, OutcomeState::Landed, "{}", finished.reason);

        // The lines the three stages say, in the order they said them; the rest (dispatch, landing) is
        // not the order under test.
        let lines = ui.0.lock().unwrap().clone();
        let stages: Vec<&str> = lines
            .iter()
            .map(String::as_str)
            .filter(|l| l.starts_with("dispatched ") || l.starts_with("landed "))
            .map(|l| l.split_once(' ').unwrap().0)
            .collect();
        assert_eq!(stages, ["dispatched", "landed"]);
        let first = lines
            .iter()
            .position(|l| l.starts_with("dispatched "))
            .unwrap();
        let said: Vec<&str> = lines[first..]
            .iter()
            .map(String::as_str)
            .take_while(|l| !l.starts_with("landed "))
            .collect();
        assert_eq!(said.len(), 7, "{said:#?}");
        assert!(said[0].starts_with("dispatched "), "{said:#?}");
        assert_eq!(said[1], "baseline   probe ok");
        // The producer and verifier lines carry a model, a wall time, a cost and a count or verdict.
        assert!(
            said[2].starts_with("producer   claude-haiku-5-5 · ")
                && said[2].ends_with(" · $0.01 · 0 commands run"),
            "{said:#?}"
        );
        assert!(
            said[3].starts_with("candidate  README.md (modified) · "),
            "{said:#?}"
        );
        assert!(said[4].starts_with("checks     probe ok ("), "{said:#?}");
        // The replay has no history to replay here, and a fake driver builds nothing: it says why.
        assert!(said[5].starts_with("replay     no replay: "), "{said:#?}");
        assert!(
            said[6].starts_with("verifier   ") && said[6].ends_with(" · $0.01 · pass"),
            "{said:#?}"
        );

        // The invocation numbers: the reading is 1, the producer 2, the verifier 3, and the baseline was
        // taken before the producer, so it names the producer's number.
        let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        let job = &records.jobs[proposal.job.as_str()];
        let mut invocations: Vec<(u32, Role)> =
            job.invocations.iter().map(|i| (i.seq, i.role)).collect();
        invocations.sort();
        assert_eq!(
            invocations,
            [(1, Role::Reader), (2, Role::Producer), (3, Role::Verifier)]
        );
        let baseline = job
            .baseline
            .as_ref()
            .expect("the job recorded its baseline");
        assert_eq!(baseline.before_invocation, 2);
        assert_eq!(baseline.checks.len(), 1);
    }

    const MARKER: &str = "MARKER";

    /// Commits `total` jobs, each with a request, an accepted reading and a job record, as an older history.
    /// The first `marked` readings carry the marker in their title. The ids are dated 2020, so they are older
    /// than any job the test runs.
    fn fixture_reading(id: &JobId, title: &str) -> Reading {
        use athena_data::records::{
            Criterion, CriterionKind, Deliverable, Forecast, Forecasts, Quantiles,
        };
        let forecast = Forecast {
            p_reading_accepted: 0.8,
            p_verification_passes: 0.7,
            p_user_intervenes: 0.1,
            cost_usd: Quantiles::new(0.2, 0.5, 1.5).unwrap(),
            wall_time_s: Quantiles::new(100.0, 300.0, 900.0).unwrap(),
        };
        Reading {
            job: id.clone(),
            n: 1,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: 1,
            base_commit: "a".repeat(40),
            config_sha256: "b".repeat(64),
            defaults_sha256: "c".repeat(64),
            defaults_version: 1,
            title: title.into(),
            summary: "Add a greeting.".into(),
            class_of_work: "feature".into(),
            capability: Some("study".into()),
            decline_reason: None,
            producer_model: "producer".into(),
            verifier_model: "verifier".into(),
            defaults_applied: vec![],
            assumptions: vec![],
            question: None,
            acceptance_criteria: vec![Criterion {
                text: "cargo test --locked passes".into(),
                kind: CriterionKind::Behavior,
            }],
            deliverables: vec![Deliverable {
                path: "src/greeting.rs".into(),
                action: DeliverableAction::Modify,
                p: 0.9,
            }],
            out_of_scope: vec!["the changelog".into()],
            forecasts: Forecasts {
                reader: forecast.clone(),
                reference: forecast,
            },
            departures: None,
            urgent_reason: None,
            hard_reason: None,
            producer_effort: None,
        }
    }

    fn seed_accepted_jobs(athena: &Athena, total: usize, marked: usize) {
        use athena_data::records::Request;
        let mut files = Vec::new();
        for n in 0..total {
            let id = JobId::parse(&format!("20200101-{n:06}-{n:04}")).unwrap();
            let request = Request {
                job: id.clone(),
                text: "add a greeting".into(),
                received_at: jiff::Timestamp::UNIX_EPOCH,
                channel: "cli".into(),
            };
            let title = if n < marked { MARKER } else { "Add a greeting" };
            let reading = fixture_reading(&id, title);
            let job: Job = serde_json::from_value(json!({
                "id": id.as_str(), "kind": "request", "title": "Add a greeting",
                "class_of_work": null, "capability": "study", "accepted_reading": 1,
                "accepted_by": null,
                "owner": { "role": "producer", "capability": null, "model": null, "note": null },
                "verifier_model": null, "caps": null, "base_commit": null, "config_sha256": null,
                "defaults_sha256": null, "athena": null, "driver": null, "brief_sha256": null,
                "dispatched_at": null, "request_path": null, "request_sha256": null, "cost": null,
                "tokens": null, "note": null, "pin": null
            }))
            .unwrap();
            files.push(NewFile::record(&AnyRecord::Request(request)));
            files.push(NewFile::record(&AnyRecord::Reading(Box::new(reading))));
            files.push(NewFile::record(&AnyRecord::Job(Box::new(job))));
        }
        athena
            .commit_records(
                &files,
                &message(
                    "Seed accepted readings",
                    "Fixture history for the replay.",
                    &[],
                ),
            )
            .unwrap();
    }

    /// The base rules: the real reading rules.
    fn real_rules() -> std::rc::Rc<dyn ReadingChecker> {
        std::rc::Rc::new(|request: &str, reading: &Reading| {
            Ok(crate::reading::check_reading_text(reading, request))
        })
    }

    /// A candidate-like rule change: the real rules plus a refusal for the marker in the title.
    fn rules_refusing_the_marker() -> std::rc::Rc<dyn ReadingChecker> {
        std::rc::Rc::new(|request: &str, reading: &Reading| {
            Ok(match crate::reading::check_reading_text(reading, request) {
                crate::reading::ReadingCheck::Accept if reading.title.contains(MARKER) => {
                    crate::reading::ReadingCheck::Refuse("the marker rule refuses this".into())
                }
                other => other,
            })
        })
    }

    fn finished_job(athena: &Athena) -> (JobId, Box<Finished>) {
        let proposal = athena.ask("describe the demo", &Quiet).unwrap();
        let GoResult::Finished(finished) = athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Quiet)
            .unwrap()
        else {
            panic!("expected the job to finish");
        };
        (proposal.job, finished)
    }

    #[test]
    fn replay_holds_over_threshold_rejects_the_job_and_records_the_list() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let athena = athena(&repo, &state);
        seed_accepted_jobs(&athena, 10, 3);
        // The seam is read before tree resolution, the builds and the subcommand probe, so the fake
        // driver's "no replay" skip is not reached.
        let _seam = crate::replay_seam::install(real_rules(), rules_refusing_the_marker());
        let (job, finished) = finished_job(&athena);
        assert_eq!(
            finished.state,
            OutcomeState::Rejected,
            "{}",
            finished.reason
        );

        let head = athena.head().unwrap();
        let records = Records::load(&athena.git, &head).unwrap();
        let jr = &records.jobs[job.as_str()];
        let replay = jr.replay.as_ref().expect("replay.json is recorded");
        assert_eq!(replay.no_replay, None);
        assert_eq!(replay.total, 10);
        assert_eq!(replay.newly_refused.len(), 3, "{:?}", replay.newly_refused);
        assert!(replay.held && !replay.intended);
        let listed: Vec<&str> = replay
            .newly_refused
            .iter()
            .map(|r| r.job.as_str())
            .collect();
        assert_eq!(
            listed,
            [
                "20200101-000002-0002",
                "20200101-000001-0001",
                "20200101-000000-0000"
            ]
        );
        // The Athena blocker finding names the same list.
        let verification = jr.verification.as_ref().expect("verification.json");
        let blocker = verification
            .findings
            .iter()
            .find(|f| f.severity == Severity::Blocker)
            .expect("a blocker finding");
        assert_eq!(blocker.source, FindingSource::Athena);
        let text = format!("{} {}", blocker.summary, blocker.evidence);
        for id in &listed {
            assert!(text.contains(id), "{id}: {text}");
        }
        assert_eq!(verification.final_verdict, Verdict::Fail);
        // The records check accepts every file of the job directory, and no free-form replay.md exists.
        let dir = job.dir();
        let names = athena
            .git
            .run([
                "ls-tree",
                "-r",
                "--name-only",
                head.as_str(),
                "--",
                dir.as_str(),
            ])
            .unwrap();
        let paths: Vec<&str> = names.lines().collect();
        assert!(
            paths.iter().any(|p| p.ends_with("/replay.json")),
            "{paths:?}"
        );
        assert!(!paths.iter().any(|p| p.ends_with("replay.md")), "{paths:?}");
        for path in &paths {
            let bytes = athena.git.show_file(&head, path).unwrap().unwrap();
            athena_data::records::check_record_file(path, &bytes)
                .unwrap_or_else(|e| panic!("{path}: {e}"));
        }
    }

    #[test]
    fn replay_view_carries_the_declared_share_and_reason() {
        let id = JobId::parse("20200101-000000-0000").unwrap();
        let record = ReplayRun {
            total: 50,
            newly_refused: vec![athena_data::records::ReplayRefusal {
                job: id.clone(),
                reason: "the marker rule refuses this".into(),
            }],
            intended: true,
            held: false,
            no_replay: None,
        }
        .record(&id, "a", "b");
        let reason = "readers now ask for a new required section, because past readings lack it";
        let mut reading = fixture_reading(&id, "Add a greeting");
        reading.acceptance_criteria[0].text = "expected 80% of newly refused readings".into();
        reading.out_of_scope = vec!["the changelog".into(), reason.into()];
        // A share above 25% with a reason: the view carries both, as the verifier brief site passes them.
        let view = replay_view(&record, &reading);
        assert_eq!(view.declared, Some(80));
        assert_eq!(view.reason.as_deref(), Some(reason));
        assert_eq!(view.unsupported, None);
        assert_eq!((view.total, view.newly_refused.len()), (50, 1));
        // The same share with no reason is unsupported, and the 10% limit applied.
        reading.out_of_scope = vec!["the changelog".into()];
        let view = replay_view(&record, &reading);
        assert_eq!(
            (view.declared, view.reason, view.unsupported),
            (None, None, Some(80))
        );
        // A reading that declares nothing leaves all three empty.
        let plain = fixture_reading(&id, "Add a greeting");
        let view = replay_view(&record, &plain);
        assert_eq!(
            (view.declared, view.reason, view.unsupported),
            (None, None, None)
        );
    }

    #[test]
    fn replay_within_threshold_records_the_list_and_lands() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let athena = athena(&repo, &state);
        seed_accepted_jobs(&athena, 10, 1);
        let _seam = crate::replay_seam::install(real_rules(), rules_refusing_the_marker());
        let (job, finished) = finished_job(&athena);
        assert_eq!(finished.state, OutcomeState::Landed, "{}", finished.reason);
        let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        let replay = records.jobs[job.as_str()].replay.clone().expect("a replay");
        assert_eq!((replay.total, replay.newly_refused.len()), (10, 1));
        assert!(!replay.held);
    }

    /// Wraps `script` to log the role of every invocation it receives, in order.
    fn role_log(
        script: athena_agent::fake::Script,
        log: Arc<Mutex<Vec<Role>>>,
    ) -> athena_agent::fake::Script {
        Box::new(move |spec: &AgentInvocation| {
            log.lock().unwrap().push(spec.role);
            script(spec)
        })
    }

    /// Runs one job over a seeded history with this candidate rule; returns the roles invoked up to the
    /// verifier, the (seq, role) invocation records up to the verifier, and the replay record.
    fn run_with_candidate(
        candidate: std::rc::Rc<dyn ReadingChecker>,
    ) -> (Vec<Role>, Vec<(u32, Role)>, Replay) {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let log = Arc::new(Mutex::new(Vec::new()));
        let athena = athena_with(
            &repo,
            &state,
            PASSES,
            crate::test_support::plenty_of_space,
            role_log(base_script(), log.clone()),
        );
        seed_accepted_jobs(&athena, 10, 3);
        let _seam = crate::replay_seam::install(real_rules(), candidate);
        let (job, _) = finished_job(&athena);
        let roles = log.lock().unwrap().clone();
        let upto = roles
            .iter()
            .position(|r| *r == Role::Verifier)
            .expect("the verifier ran");
        let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        let jr = &records.jobs[job.as_str()];
        let mut invocations: Vec<(u32, Role)> =
            jr.invocations.iter().map(|i| (i.seq, i.role)).collect();
        invocations.sort();
        invocations.truncate(upto + 1);
        (
            roles[..=upto].to_vec(),
            invocations,
            jr.replay.clone().expect("a replay"),
        )
    }

    #[test]
    fn replay_makes_no_model_call_with_or_without_a_candidate_rule_change() {
        // Run one: the candidate rules equal the base. Run two: the candidate adds a refusal, so the job is held.
        let (plain_roles, plain_records, plain) = run_with_candidate(real_rules());
        let (changed_roles, changed_records, changed) =
            run_with_candidate(rules_refusing_the_marker());
        assert!(!plain.held && plain.newly_refused.is_empty());
        assert!(changed.held && changed.newly_refused.len() == 3);
        // The replay stage runs before the verifier, so a model call it made would put a role here.
        assert_eq!(
            plain_roles,
            [Role::Reader, Role::Producer, Role::Verifier],
            "{plain_roles:?}"
        );
        assert_eq!(plain_roles, changed_roles);
        assert_eq!(plain_records, changed_records);
        assert_eq!(
            plain_records.iter().map(|(_, r)| *r).collect::<Vec<_>>(),
            [Role::Reader, Role::Producer, Role::Verifier]
        );
    }

    #[test]
    fn replay_makes_no_model_call_because_the_checker_is_built_from_a_path_alone() {
        // `BinaryChecker::new` takes a path: there is no driver or agent handle to call. Run on a path that
        // is no program, it fails to spawn, which is all it can do.
        let checker = BinaryChecker::new(PathBuf::from("/nonexistent/athena"));
        let id = JobId::parse("20200101-000000-0000").unwrap();
        let reading = fixture_reading(&id, "Add a greeting");
        assert!(checker.check("add a greeting", &reading).is_err());
    }

    /// How many times each agent role was invoked.
    #[derive(Default)]
    struct Counts {
        planner: std::sync::atomic::AtomicUsize,
        producer: std::sync::atomic::AtomicUsize,
        reader: std::sync::atomic::AtomicUsize,
        verifier: std::sync::atomic::AtomicUsize,
    }

    impl Counts {
        fn counter(&self, role: Role) -> Option<&std::sync::atomic::AtomicUsize> {
            match role {
                Role::Planner => Some(&self.planner),
                Role::Producer => Some(&self.producer),
                Role::Reader => Some(&self.reader),
                Role::Verifier => Some(&self.verifier),
                _ => None,
            }
        }

        /// The (planner, producer, reader, verifier) counts so far.
        fn all(&self) -> (usize, usize, usize, usize) {
            let get =
                |c: &std::sync::atomic::AtomicUsize| c.load(std::sync::atomic::Ordering::SeqCst);
            (
                get(&self.planner),
                get(&self.producer),
                get(&self.reader),
                get(&self.verifier),
            )
        }
    }

    /// The base script, counting each planner, producer, reader and verifier invocation. When `state_root` is
    /// given, the producer also leaves a file in its job's build directory (`target/produce`), as a build would.
    fn counting(counts: Arc<Counts>, state_root: Option<PathBuf>) -> athena_agent::fake::Script {
        let inner = base_script();
        Box::new(move |spec: &AgentInvocation| {
            if let Some(counter) = counts.counter(spec.role) {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
            if spec.role == Role::Producer
                && let Some(root) = &state_root
            {
                let dir = root
                    .join("jobs")
                    .join(&spec.job)
                    .join("target/produce/debug");
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::write(dir.join("big"), vec![7u8; 20_000]).unwrap();
            }
            let mut turn = inner(spec);
            // A reading of a retry must name the job it retries, as a reader that read the request would.
            if spec.role == Role::Reader
                && let Some((_, after)) = spec.user_prompt.split_once("try again on ")
                && let Some(structured) = turn.structured.as_mut()
            {
                let earlier: String = after
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                    .collect();
                structured["summary"] = json!(format!(
                    "Describe the demo in README.md, as job {earlier} did."
                ));
            }
            turn
        })
    }

    fn finished(result: GoResult) -> Finished {
        match result {
            GoResult::Finished(finished) => *finished,
            GoResult::Reread(_) => panic!("expected the job to finish"),
        }
    }

    fn readme_at_head(athena: &Athena) -> String {
        let bytes = athena.git.show_file("HEAD", "README.md").unwrap().unwrap();
        String::from_utf8(bytes).unwrap()
    }

    #[test]
    fn produce_dir_is_removed_after_checks_pass_and_kept_when_a_check_fails() {
        for (check, passes) in [(PASSES, true), (FAILS_ON_EDIT, false)] {
            let repo = repo();
            let state = tempfile::tempdir().unwrap();
            let root = state.path().join("repo");
            let counts = Arc::new(Counts::default());
            let athena = athena_with(
                &repo,
                &state,
                check,
                crate::test_support::recording_plenty_space,
                counting(counts, Some(root.clone())),
            );
            let proposal = athena.ask("describe the demo", &Quiet).unwrap();
            let ui = Lines::default();
            let done = finished(
                athena
                    .go(proposal.job.as_str(), AcceptedBy::User, &ui)
                    .unwrap(),
            );

            // Every read of the free space saw the job directories as they were at that moment. The first read
            // with the verifier's snapshot present is the gate before the verifier; the one before it is the
            // gate before the checks.
            let probes = crate::test_support::probes_of(&root);
            let gate = probes
                .iter()
                .position(|p| p.verifying)
                .expect("the gate before the verifier read the free space");
            assert!(gate > 0, "{probes:?}");
            assert!(
                probes[gate - 1].produce_present,
                "the producer's build directory existed when the checks started: {probes:?}"
            );
            if passes {
                assert!(
                    !probes[gate].produce_present,
                    "the checks passed, so target/produce is gone before the verifier gate: {probes:?}"
                );
                assert_eq!(done.state, OutcomeState::Landed, "{}", done.reason);
            } else {
                assert!(
                    probes[gate].produce_present,
                    "a check failed, so target/produce is kept: {probes:?}"
                );
                assert_eq!(done.state, OutcomeState::Rejected, "{}", done.reason);
            }
        }
    }

    #[test]
    fn low_disk_before_the_verifier_resumes_without_planner_or_producer() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let root = state.path().join("repo");
        let counts = Arc::new(Counts::default());
        // Enough free space for the gate before the checks; too little for the gate before the verifier.
        let athena = athena_with(
            &repo,
            &state,
            PASSES,
            crate::test_support::low_once_verifying,
            counting(counts.clone(), Some(root.clone())),
        );
        let proposal = athena.ask("describe the demo", &Quiet).unwrap();
        let first = proposal.job.clone();
        let ui = Lines::default();
        let stopped = finished(athena.go(first.as_str(), AcceptedBy::User, &ui).unwrap());
        assert_eq!(stopped.state, OutcomeState::Failed, "{}", stopped.reason);
        assert!(
            crate::disk::is_environment_reason(&stopped.reason),
            "{}",
            stopped.reason
        );
        assert!(
            stopped.reason.contains("before the verifier"),
            "{}",
            stopped.reason
        );
        let said = ui.0.lock().unwrap().clone();
        assert!(
            said.iter().any(|l| l.starts_with("checks     ")),
            "{said:#?}"
        );
        // The producer ran once; no planner, and no verifier.
        let (planner, producer, read_before, verifier) = counts.all();
        assert_eq!((planner, producer, verifier), (0, 1, 0));
        assert!(read_before >= 1);
        let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        let candidate_fingerprint = records.jobs[first.as_str()]
            .candidate
            .as_ref()
            .and_then(|c| c.fingerprint.clone())
            .expect("the stopped job recorded its candidate");
        // The job's directory is gone, but its candidate is kept, at the stage that stopped.
        assert!(!athena.state.job_dir(&first).exists());
        assert_eq!(
            athena.state.read_resume(&first).map(|m| m.stage),
            Some(crate::state::ResumeStage::Verifier)
        );

        // Space is back. The retry reads again (the reader runs), then resumes.
        std::fs::write(root.join("freed"), b"").unwrap();
        let ui = Lines::default();
        let (retry, _) = athena.retry(first.as_str(), &ui).unwrap();
        assert_eq!(retry.reader_error, None);
        assert!(
            ui.0.lock()
                .unwrap()
                .iter()
                .any(|l| l.starts_with("resume     ")),
            "{:#?}",
            ui.0.lock().unwrap()
        );
        let (planner, producer, read_after, verifier) = counts.all();
        assert_eq!((planner, producer, verifier), (0, 1, 0));
        assert!(read_after > read_before, "the reader ran again");
        let ui = Lines::default();
        let done = finished(
            athena
                .go(retry.job.as_str(), AcceptedBy::User, &ui)
                .unwrap(),
        );
        assert_eq!(done.state, OutcomeState::Landed, "{}", done.reason);
        // No planner and no producer ran in the retry; only the verifier did.
        assert_eq!(counts.all(), (0, 1, read_after, 1));
        let said = ui.0.lock().unwrap().clone();
        assert!(
            !said.iter().any(|l| l.starts_with("checks     ")),
            "the retry resumed at the verifier: {said:#?}"
        );
        assert!(
            said.iter().any(|l| l.starts_with("verifier   ")),
            "{said:#?}"
        );
        assert_eq!(
            done.verification.as_ref().unwrap().verified_fingerprint,
            candidate_fingerprint,
            "the verifier judged the kept candidate"
        );
        assert!(readme_at_head(&athena).contains("A demo."));
        assert_eq!(athena.state.read_resume(&first), None, "the mark is spent");
        assert_eq!(athena.state.read_resume(&retry.job), None);
    }

    #[test]
    fn a_retry_reader_brief_carries_the_earlier_jobs_files_approach_and_checks() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        // The base script, keeping every prompt the reader is given.
        let prompts: Arc<Mutex<Vec<String>>> = Arc::default();
        let inner = counting(Arc::new(Counts::default()), None);
        let kept = prompts.clone();
        let script: athena_agent::fake::Script = Box::new(move |spec: &AgentInvocation| {
            if spec.role == Role::Reader {
                kept.lock().unwrap().push(spec.user_prompt.clone());
            }
            inner(spec)
        });
        let athena = athena_with(
            &repo,
            &state,
            FAILS_ON_EDIT,
            crate::test_support::plenty_of_space,
            script,
        );
        let proposal = athena.ask("describe the demo", &Quiet).unwrap();
        let first = proposal.job.clone();
        let rejected = finished(
            athena
                .go(first.as_str(), AcceptedBy::User, &Lines::default())
                .unwrap(),
        );
        assert_eq!(
            rejected.state,
            OutcomeState::Rejected,
            "{}",
            rejected.reason
        );

        // The retry's reader is shown what the rejected job did, from its candidate and verification.
        let (retry, _) = athena.retry(first.as_str(), &Quiet).unwrap();
        assert_eq!(retry.reader_error, None);
        let prompts = prompts.lock().unwrap();
        let brief = prompts.last().unwrap();
        let section = brief
            .split_once(athena_context::briefs::PRIOR_RESULT_HEADING)
            .unwrap_or_else(|| panic!("no section:\n{brief}"))
            .1;
        for needle in [
            "Files its candidate changed (1)",
            "\n> modified README.md\n",
            "\n> Described the demo.\n",
            "Measured checks:\n> probe: failed (exit code 1",
        ] {
            assert!(section.contains(needle), "{needle:?}:\n{section}");
        }
    }

    /// A UI that dies, as a process does, when the checks start.
    struct DiesAtChecks;

    impl Ui for DiesAtChecks {
        fn line(&self, text: &str) {
            if text.starts_with("checks     ") {
                panic!("the process died");
            }
        }
    }

    #[test]
    fn interrupted_job_with_a_candidate_resumes_without_planner_or_producer() {
        let repo = repo();
        let state = tempfile::tempdir().unwrap();
        let counts = Arc::new(Counts::default());
        let athena = athena_with(
            &repo,
            &state,
            PASSES,
            crate::test_support::plenty_of_space,
            counting(counts.clone(), None),
        );
        let proposal = athena.ask("describe the demo", &Quiet).unwrap();
        let first = proposal.job.clone();
        // The process dies after the candidate is built, as the checks start: the job is left dispatched.
        let died = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            athena.go(first.as_str(), AcceptedBy::User, &DiesAtChecks)
        }));
        assert!(died.is_err(), "the job was cut short");
        let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        assert_eq!(records.jobs[first.as_str()].state(), JobState::Dispatched);
        assert!(!athena.running(first.as_str()));
        let (planner, producer, read_before, verifier) = counts.all();
        assert_eq!((planner, producer, verifier), (0, 1, 0));
        let candidate_fingerprint = athena
            .state
            .read_resume(&first)
            .and_then(|m| m.candidate.fingerprint)
            .expect("the candidate was kept");

        let (retry, _) = athena.retry(first.as_str(), &Quiet).unwrap();
        assert_eq!(retry.reader_error, None);
        let (planner, producer, read_after, verifier) = counts.all();
        assert_eq!((planner, producer, verifier), (0, 1, 0));
        assert!(read_after > read_before, "the reader ran again");
        let ui = Lines::default();
        let done = finished(
            athena
                .go(retry.job.as_str(), AcceptedBy::User, &ui)
                .unwrap(),
        );
        assert_eq!(done.state, OutcomeState::Landed, "{}", done.reason);
        // No planner and no producer ran in the retry. The checks had not finished, so they ran in it.
        assert_eq!(counts.all(), (0, 1, read_after, 1));
        let said = ui.0.lock().unwrap().clone();
        assert!(
            said.iter().any(|l| l.starts_with("resume     ")),
            "{said:#?}"
        );
        assert!(
            said.iter().any(|l| l.starts_with("checks     ")),
            "{said:#?}"
        );
        assert_eq!(
            done.verification.as_ref().unwrap().verified_fingerprint,
            candidate_fingerprint
        );
        assert!(readme_at_head(&athena).contains("A demo."));
        assert_eq!(athena.state.read_resume(&first), None, "the mark is spent");
    }
}
