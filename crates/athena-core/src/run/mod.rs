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
    ReserveStage, TreeRetrySeed, planning_resume_seed, replay, replay_cases, replay_declaration_of,
    resumable_job, retried_job, retry_findings, same_scope, tree_retry_seed,
};
use crate::scoring::{self, Happened, score, totals};
use crate::state::{NoStages, ResumeMark, ResumeStage, Stage, StageSink};
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

mod candidate;
mod checks;
mod close;
mod pin;
mod planning;
mod producer;
mod send_back;
mod verification;

use candidate::Admitted;
use checks::SnapExec;
use producer::{Flow, Reported};
use verification::Checked;

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
        self.go_with_stages(job_ref, accepted_by, ui, &NoStages)
    }

    /// [`go`](Athena::go), reporting each stage the run enters to `stages`: the producer (again on each
    /// send-back), the checks, the verifier, and the landing of a blocked job.
    pub fn go_with_stages(
        &self,
        job_ref: &str,
        accepted_by: AcceptedBy,
        ui: &dyn Ui,
        stages: &dyn StageSink,
    ) -> Result<GoResult> {
        let result = self.go_job(job_ref, accepted_by, ui, stages)?;
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

    fn go_job(
        &self,
        job_ref: &str,
        accepted_by: AcceptedBy,
        ui: &dyn Ui,
        stages: &dyn StageSink,
    ) -> Result<GoResult> {
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
            JobState::Blocked => {
                stages.enter(&id, Stage::Landing);
                self.retry_landing(&records, &job, ui)
                    .map(|f| GoResult::Finished(Box::new(f)))
            }
            JobState::Proposed => match job.latest_reading() {
                Some(reading) if reading.capability.is_some() => self.dispatch(
                    &head,
                    &records,
                    &job,
                    reading.clone(),
                    accepted_by,
                    Outputs { ui, stages },
                ),
                Some(_) => unreachable!("a reading without a capability is declined"),
                // The first read gave no reading (the reader failed): read it again, as for a queued job.
                None => self
                    .read_unread(&job, ui)
                    .map(|p| GoResult::Reread(Box::new(p))),
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
                if let Some(low) = crate::disk::read_hold(&self.state, &id) {
                    bail!(
                        "job {id} is held: {}. A drain resumes it from its kept candidate once space is free; `athena retry {id}` does the same by hand, or `athena close {id} --why \"...\"` records what happened",
                        low.text()
                    )
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

    /// Spends the resume marks when a job closes: the mark (and any hold marker) of the job this one resumed
    /// from, and this job's own unless it closed failed for an environment cause (which `athena retry`
    /// resumes from).
    fn settle_resume_marks(
        &self,
        id: &JobId,
        consumed: Option<&JobId>,
        state: OutcomeState,
        reason: &str,
    ) {
        if let Some(old) = consumed {
            self.state.remove_resume(old);
            crate::disk::clear_hold(&self.state, old);
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
        Outputs { ui, stages }: Outputs<'_>,
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
                Outputs { ui, stages },
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
        // With several jobs allowed in flight, each with its own worktree and build directory, the disk may
        // fit fewer than are allowed (design section 5). The extra start waits the same way: held, queued,
        // not failed. With one job at a time, or no other job running, this never holds.
        if let Some(low) = self.start_hold(&jc, records, &id, ui) {
            crate::disk::write_hold(&self.state, &id, &low);
            ui.line(&format!("held       {}", low.text()));
            return Err(low.into());
        }
        crate::disk::clear_hold(&self.state, &id);
        let capability_id = reading.capability.clone().expect("checked by the caller");
        let capability = self.capability(&capability_id)?;
        // In `hard` mode only a reading marked hard is planned; the marking is the one that escalates the
        // producer (job-planning.md section 9, D143).
        let planned = jc
            .config
            .job_plan
            .as_ref()
            .is_some_and(|p| p.plans(reading.hard_reason.is_some()))
            && !capability.pins_repo();
        // The whole path still to run (every planning round, the producer and the verifier) must be
        // affordable before anything is committed.
        self.preflight(&jc, job.spent_usd(), ReserveStage::Dispatch { planned })?;
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
            stages,
            pin_dir: pinned.map(|(_, dir)| dir),
            seed_commit: retry_seed.map(|s| s.candidate_commit),
            resume,
        };
        // Job planning (docs/design/job-planning.md): one accepted round, before the producer. The round
        // seeds the closing with its invocations, plan and critique, so the producer's invocation is
        // numbered after them and the budget check below counts what planning cost (review F1).
        if let Some(seed) = resume_from {
            let run = self.resume_candidate(&ctx, &accepted, seed, Closing::new());
            return self.finish_dispatched(&ctx, run);
        }
        let mut closing = Closing::new();
        let mut producer_prompt = brief.clone();
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
            if let Err(e) = self.preflight(&jc, spent, ReserveStage::Run) {
                self.stop_planning(&ctx, &mut closing, &format!("{e}"));
                return self.close_dispatched(&ctx, closing);
            }
        }
        let run = self.produce_and_verify(&ctx, &producer_prompt, &accepted, closing);
        self.finish_dispatched(&ctx, run)
    }

    /// Closes a job whose run can have pushed its CI branch (the resume path and the produce-and-verify
    /// path), then deletes that branch. The delete runs whatever the run ended in: a verdict of any outcome,
    /// a stop, or an error; a failed delete is a warning line only (see [`Athena::delete_ci_branch`]).
    fn finish_dispatched(&self, ctx: &RunContext<'_>, run: Result<Closing>) -> Result<GoResult> {
        let closed = run.and_then(|closing| self.close_dispatched(ctx, closing));
        self.delete_ci_branch(ctx.id(), ctx.ui);
        closed
    }
}

/// Where a run says what it is doing: progress lines, and the stage it enters.
#[derive(Clone, Copy)]
struct Outputs<'a> {
    ui: &'a dyn Ui,
    stages: &'a dyn StageSink,
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
    /// Told when the run enters the producer, the checks and the verifier.
    stages: &'a dyn StageSink,
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

/// The seam tests use to give a routed check a ready-made result in place of a process. It is read first in
/// `SnapExec::exec`, so classification, the closing reason and the job state all stay in production code. It
/// is per thread, so parallel tests do not see each other's results.
#[cfg(test)]
pub(crate) mod check_seam {
    use crate::capability::CheckSpec;
    use athena_data::records::CheckRun;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Builds the result of one check from its spec.
    pub type Fake = Rc<dyn Fn(&CheckSpec) -> CheckRun>;

    thread_local! {
        static SEAM: RefCell<Option<Fake>> = const { RefCell::new(None) };
    }

    /// Clears the seam when dropped.
    pub struct Guard;

    impl Drop for Guard {
        fn drop(&mut self) {
            SEAM.with(|s| *s.borrow_mut() = None);
        }
    }

    pub fn install(fake: Fake) -> Guard {
        SEAM.with(|s| *s.borrow_mut() = Some(fake));
        Guard
    }

    pub fn current() -> Option<Fake> {
        SEAM.with(|s| s.borrow().clone())
    }
}

/// The seam tests use to give the CI read a fake `gh` runner and a sleeper that spends no time. With nothing
/// installed, a test build reads no CI result (`routing::NoCi`), so a test that gives the repository a GitHub
/// origin never reaches the real `gh` or waits. Production builds do not have it. It is per thread.
#[cfg(test)]
pub(crate) mod ci_seam {
    use crate::gh::Runner;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::time::Duration;

    /// Called in place of a sleep between two reads of the CI run.
    pub type Sleeper = Rc<dyn Fn(Duration)>;

    type Pair = (Rc<dyn Runner>, Sleeper);

    thread_local! {
        static SEAM: RefCell<Option<Pair>> = const { RefCell::new(None) };
    }

    /// Clears the seam when dropped.
    pub struct Guard;

    impl Drop for Guard {
        fn drop(&mut self) {
            SEAM.with(|s| *s.borrow_mut() = None);
        }
    }

    pub fn install(runner: Rc<dyn Runner>, sleeper: Sleeper) -> Guard {
        SEAM.with(|s| *s.borrow_mut() = Some((runner, sleeper)));
        Guard
    }

    pub fn current() -> Option<Pair> {
        SEAM.with(|s| s.borrow().clone())
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
    /// Picks where the CI result of job `job` is read from and how long to wait for it, together, and runs
    /// `act` with both. The source and the wait are one choice, so a repository with no GitHub origin can
    /// never get a wait:
    ///
    /// - a nested run, or an origin that is not an https URL on github.com (none, a local path, another
    ///   host): [`crate::routing::NoCi`] with one read and no wait;
    /// - a GitHub origin: the push run on the job's CI branch, read through `gh`, once at first and then
    ///   every [`crate::routing::CI_POLL`] up to `wait_s` seconds. In a test build the runner and the sleeper
    ///   come from [`ci_seam`], and with none installed the choice is `NoCi`.
    ///
    /// The source borrows the client, which borrows the runner, so they live only inside this call.
    fn with_ci_source<T>(
        &self,
        job: &JobId,
        wait_s: u64,
        act: impl FnOnce(&dyn crate::ci_result::CiSource, crate::ci_result::Wait) -> T,
    ) -> T {
        use crate::ci_result::Wait;
        let one_read = Wait {
            polls: 1,
            pause: std::time::Duration::ZERO,
        };
        let repository = if self.nested.is_some() {
            None
        } else {
            self.git
                .run(["remote", "get-url", "origin"])
                .ok()
                .and_then(|url| crate::gh::GhClient::origin_repository(&url))
        };
        let Some(repository) = repository else {
            return act(&crate::routing::NoCi, one_read);
        };
        let wait = Wait {
            polls: crate::routing::ci_polls(wait_s),
            pause: std::time::Duration::ZERO,
        };
        let branch = crate::gh::ci_branch(job);
        #[cfg(test)]
        {
            let Some((runner, sleeper)) = ci_seam::current() else {
                return act(&crate::routing::NoCi, one_read);
            };
            let client = crate::gh::GhClient::new(&*runner, repository);
            let source = client.ci_source(&branch);
            let sleep = move |pause: std::time::Duration| (*sleeper)(pause);
            let paced = crate::routing::Paced::new(&source, &sleep, crate::routing::CI_POLL);
            act(&paced, wait)
        }
        #[cfg(not(test))]
        {
            let runner = crate::gh::GhCli;
            let client = crate::gh::GhClient::new(&runner, repository);
            let source = client.ci_source(&branch);
            let paced =
                crate::routing::Paced::new(&source, &std::thread::sleep, crate::routing::CI_POLL);
            act(&paced, wait)
        }
    }

    /// The stages of a job after the reading is accepted, in order:
    ///
    /// 1. `baseline_stage` takes the baseline checks before the producer runs. Then the disk gate: low space
    ///    holds the job here, before the producer is started.
    /// 2. `producer_snapshot` and `producer_loop` run the producer and collect its report; a stop in
    ///    the loop closes the job here. The producer's worktree and build directory are a
    ///    [`crate::state::JobWorktree`], held until this function returns by any path.
    /// 3. `candidate_stage` admits the producer's work as a candidate commit; a refusal closes the job
    ///    here.
    /// 4. The producer's work tree is removed and the resume mark is written, so a job that stops for
    ///    the environment from here on is resumed from the candidate.
    /// 5. `check_and_verify` runs Athena's checks (see `checks.rs`), then the verifier stages (see
    ///    `verification.rs`), and returns the closing.
    fn produce_and_verify(
        &self,
        ctx: &RunContext<'_>,
        brief: &str,
        accepted: &AcceptedReading,
        mut closing: Closing,
    ) -> Result<Closing> {
        let id = ctx.id().clone();
        let job_dir = self.state.job_dir(&id);
        let p = ctx.prediction.clone();
        // After any planning invocations the closing already holds.
        let seq = ctx.next_seq(&closing);

        self.baseline_stage(ctx, &job_dir, seq, &mut closing);
        // Free disk space, before the producer. The baseline builds on the base commit and the planner ran, so
        // the disk can be short now though it was not at dispatch. After the idle build directories are removed,
        // space that is still low holds the job as an environment stop, not a failure: no producer is started,
        // there is no candidate and no resume mark, and a later drain retries the job from the start.
        if let Some(low) = self.disk_gate(ctx.jc.config.min_free_gb(), &id, ctx.ui) {
            return Err(self.hold_for_disk(ctx, closing, low));
        }
        // The producer's worktree and build directory belong to this call: cleared now, removed on every way
        // out of the function (a verdict, a stop or an error), so a finished job leaves neither behind.
        let _worktree = self.state.job_worktree(&id);
        let prepared = self.producer_snapshot(ctx)?;
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

        let Admitted {
            commit: c,
            fingerprint: f,
        } = match self.candidate_stage(
            ctx,
            &mut closing,
            report,
            &report_gaps,
            &report_kept_in,
            &snapshot,
        )? {
            Flow::Go(admitted) => admitted,
            Flow::Stop => return Ok(closing),
        };
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
        let recorded_checks = built.recorded_checks.take();

        // Free disk space, before the checks. Low space makes builds fail in ways that look like faults in the
        // work, so after the idle build directories are removed, space that is still low holds the job: it is
        // committed with its records and no outcome, stays dispatched, and the next drain resumes it from the
        // kept candidate. It is not a failure of the work and no one needs to run `athena retry`.
        if recorded_checks.is_none()
            && let Some(low) = self.disk_gate(jc.config.min_free_gb(), &id, ctx.ui)
        {
            return Err(self.hold_for_disk(ctx, closing, low));
        }

        // The admitted candidate goes to the job's CI branch, so the repository's CI runs on it. The branch is
        // deleted when the job closes (`finish_dispatched`). The checks below read its push run, and a failure to
        // push is a warning line only: the checks then run here once the wait bound has passed.
        self.push_ci_branch(&id, &c, ctx.ui);

        // Verification starts: open or update the job's pull request now, so the host's checks run on the
        // candidate while the verifier works. It never merges, and a failure is a warning line only.
        self.early_pr(&id, &c, &ctx.reading.title, ctx.ui);

        // The verifier's snapshot: its own two-commit repository, base then candidate, with the base's
        // check-tool configuration in the working tree.
        let verify_dir = job_dir.join("verify");
        let (vsnap, tool_config_from_base, prepared) =
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
        // The statement for the verifier when the check results came from the CI run for the candidate. A
        // resumed job takes its recorded checks and a nested run runs none, so neither carries one.
        let mut reuse: Option<String> = None;
        let mut cleanup: Vec<String> = Vec::new();
        let check_started = Instant::now();
        let changed_for_skip: Vec<String> = closing
            .candidate
            .as_ref()
            .map(|c| c.changes.iter().map(|e| e.path.clone()).collect())
            .unwrap_or_default();
        let resumed_checks = recorded_checks.is_some();
        if !resumed_checks {
            ctx.stages.enter(&id, Stage::Checks);
        }
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
            // The candidate was pushed to the job's CI branch above: a GitHub origin gives its push run, read
            // up to `ci_wait_s`; any other repository gives no CI result, with one read and no wait. A green
            // run stands in for the checks, a red run fails them, and anything else runs them here.
            let (runs, ci_record) =
                self.with_ci_source(&id, jc.config.budget.ci_wait_s, |ci, wait| {
                    crate::routing::run_routed_with_record(
                        &route,
                        &c,
                        &changed_for_skip,
                        ci,
                        &exec,
                        wait,
                    )
                })?;
            check_runs = runs;
            if let Some(record) = &ci_record {
                ctx.ui.line(&format!(
                    "ci         checks reused from {} for {c}",
                    record.run_address.as_deref().unwrap_or("the CI run")
                ));
                reuse = crate::verify::reuse_statement(record, &c);
            }
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
                    failed_tests: Vec::new(),
                    reused: false,
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
            // A check killed at its deadline, with no new failure of any other kind, says nothing about the
            // work: a loaded machine does the same. The job closes as an environment stop that `athena
            // retry` resumes, and the verifier is not started. A real failure in another check still goes
            // on to be judged and rejected.
            if let Some(slow) = crate::checks::held_for_timeout(&check_runs, usable_baseline) {
                let took = slow.duration.value().unwrap_or(0.0).round() as u64;
                let reason = crate::disk::check_timeout_reason(
                    &slow.name,
                    took,
                    jc.config.budget.check_timeout_s,
                    &id,
                );
                let changed_paths = std::mem::take(&mut closing.happened.changed_paths);
                closing.happened = self.failure(ctx, &reason);
                closing.happened.changed_paths = changed_paths;
                closing.happened.verification_passed = None;
                let _ = crate::pin::remove_tree(&verify_dir);
                return Ok(closing);
            }
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

        self.verification_stage(
            ctx,
            accepted,
            closing,
            &built,
            Checked {
                vsnap,
                prepared,
                tool_config_from_base,
                check_runs,
                cleanup,
                replay_record,
                verify_dir,
                capability_env,
                reuse,
            },
        )
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
mod tests;
