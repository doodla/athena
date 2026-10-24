//! `athena go`: dispatch, produce, verify independently, and land (design §2.3, §3.4, §7).

use crate::admission::{AdmissionInput, Changed, admit, is_governance, plan_findings};
use crate::capability::{Capability, ChangeList, JobConfig};
use crate::chain::latest_landed_job;
use crate::checks::{CheckContext, run_check};
use crate::commits::{NewFile, Refusal, message};
use crate::invocation::InvocationFacts;
use crate::land::{Plan, plan};
use crate::pin::{is_full_sha, mount};
use crate::reading::{Proposal, same_scope};
use crate::scoring::{self, Happened, score, totals};
use crate::store::{JobRecords, JobState, Records};
use crate::verify::{MAP_PATH, VerdictInputs, Verified, judge, mark_pre_existing, verified};
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_agent::env::{EnvSpec, Side};
use athena_agent::policy::Policy;
use athena_context::briefs::{
    AcceptedReading, CandidateView, CheckResultView, CheckView, CriticInputs, DeliverableFact,
    PlanRevision, PlannerInputs, ProducerInputs, VerifierInputs, VerifierPlan, critic_brief,
    planner_brief, producer_brief, producer_prompt_with_plan, render_plan_section,
    verifier_brief_with_plan,
};
use athena_context::outputs::{
    CriterionStatusOutput, CriticOutput, CritiqueVerdictOutput, DeliverableActionOutput,
    PlanOutput, ProducerOutput, SeverityOutput, VerdictOutput, critic_schema, plan_schema,
    producer_schema, verdict_schema,
};
use athena_context::{
    CONTRACT, CRITIC_FRAME, Component, PLANNER_FRAME, PRODUCER_FRAME, VERIFIER_FRAME, compose,
};
use athena_data::records::{
    AcceptedBy, AnyRecord, Baseline, Build, Candidate, Caps, ChangeEntry, ChangeStatus, CheckRun,
    Critique, CritiqueVerdict, DeliverableAction, Demonstration, Finding, FindingSource,
    Invocation, InvocationOutcome, Job, JobKind, JobPlan, Outcome, OutcomeState, Owner, PinnedRepo,
    PlanDecision, PlanFile, PlanRisk, PolicyViolation, ProducerReport, Reading, RejectedOption,
    ReportedCheck, ReportedCriterion, Role, Score, Severity, Totals, Verdict, Verification,
};
use athena_data::{JobId, sha256_hex};
use athena_gate::redact::Redactor;
use athena_git::attrs::fingerprint;
use athena_git::index::{IndexEdit, TempIndex};
use athena_git::snapshot::{Snapshot, SnapshotRepo};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Instant;

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
}

impl Closing {
    fn new() -> Closing {
        Closing {
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
        };
        // Job planning (docs/design/job-planning.md): one accepted round, before the producer. The round
        // seeds the closing with its invocations, plan and critique, so the producer's invocation is
        // numbered after them and the budget check below counts what planning cost (review F1).
        let mut closing = Closing::new();
        let mut producer_prompt = brief.clone();
        let planned =
            jc.config.job_plan.as_ref().is_some_and(|p| p.enabled) && !ctx.capability.pins_repo();
        if planned {
            let Some(section) = self.plan_round(&ctx, &accepted, &check_views, &mut closing)?
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

    /// Ends the job in planning: it fails before any code was written. Its verification result is unknown,
    /// not false: no verification could have failed (review F11).
    fn stop_planning(&self, ctx: &RunContext<'_>, closing: &mut Closing, reason: &str) {
        closing.happened = self.failure(
            ctx,
            &format!("planning stopped before any code was written: {reason}"),
        );
        closing.happened.verification_passed = None;
    }

    /// Job planning: up to `max_rounds` rounds of planner, Athena's own checks and critic, in a read-only
    /// snapshot of the prediction commit. A revise sends the planner a revision brief that quotes the previous
    /// plan and every finding. On accept, the invocations, plans and critiques go into `closing` and the
    /// rendered producer section is returned. Reject, a reading the critic judged a misread, revise in the
    /// last round, or a failure of either agent stops the job: it is failed in `closing` and `None` is
    /// returned.
    fn plan_round(
        &self,
        ctx: &RunContext<'_>,
        accepted: &AcceptedReading,
        checks: &[CheckView],
        closing: &mut Closing,
    ) -> Result<Option<String>> {
        let cfg = ctx
            .jc
            .config
            .job_plan
            .expect("planning runs only with [job_plan]");
        let mut revision: Option<PlanRevision> = None;
        for round in 1..=cfg.max_rounds {
            match self.planning_round(ctx, accepted, checks, closing, round, revision.take())? {
                RoundEnd::Accepted(section) => return Ok(Some(section)),
                RoundEnd::Stopped => return Ok(None),
                RoundEnd::Revise(next) => revision = Some(*next),
            }
        }
        unreachable!("the last round either accepts or stops the job")
    }

    /// One planning round; see [`Self::plan_round`].
    fn planning_round(
        &self,
        ctx: &RunContext<'_>,
        accepted: &AcceptedReading,
        checks: &[CheckView],
        closing: &mut Closing,
        round: u32,
        revision: Option<PlanRevision>,
    ) -> Result<RoundEnd> {
        let jc = ctx.jc;
        let cfg = jc
            .config
            .job_plan
            .expect("planning runs only with [job_plan]");
        let last_round = round >= cfg.max_rounds;
        let id = ctx.id().clone();
        let p = ctx.prediction.clone();
        let budget = &jc.config.budget;
        let request = ctx
            .job
            .request
            .as_ref()
            .map(|r| r.text.clone())
            .unwrap_or_default();
        let after = budget.producer_usd + budget.verifier_usd;
        let grant = crate::capability::ToolGrant {
            tools: vec!["Read".into(), "Grep".into(), "Glob".into()],
            commands: std::collections::BTreeMap::new(),
            writes: false,
            permission_mode: "dontAsk".into(),
        };
        let snap_dir = self.state.job_dir(&id).join(format!("plan-{round}"));
        let no_env = std::collections::BTreeMap::new();
        let verifier_commands = ctx.capability.verifier_grant(jc).commands;

        // The planner.
        if let Err(e) = self.check_budget(
            jc,
            ctx.spent(closing),
            cfg.planner_usd + after,
            "planner, with the producer and verifier",
        ) {
            self.stop_planning(ctx, closing, &format!("{e}"));
            return Ok(RoundEnd::Stopped);
        }
        let mut facts = Vec::new();
        for d in &accepted.deliverables {
            let bytes = self.git.show_file(&p, &d.0)?.map(|b| b.len() as u64);
            facts.push(DeliverableFact {
                path: d.0.clone(),
                exists: bytes.is_some(),
                bytes,
            });
        }
        let planner_brief_text = planner_brief(&PlannerInputs {
            job: id.as_str().to_string(),
            request: request.clone(),
            reading: accepted.clone(),
            checks: checks.to_vec(),
            verifier_commands: athena_agent::policy::describe_commands(&verifier_commands),
            deliverable_facts: facts,
            revision,
        });
        let composed = compose(&[CONTRACT.into(), PLANNER_FRAME.into()]);
        crate::pin::remove_tree(&snap_dir)?;
        Snapshot::create(&self.git, &p, &p, &snap_dir, SnapshotRepo::None)?;
        let model = ctx.reading.producer_model.clone();
        let seq = ctx.next_seq(closing);
        let spec = self.agent_spec(
            jc,
            &id,
            seq,
            Role::Planner,
            &model,
            &composed,
            planner_brief_text,
            &grant,
            &snap_dir,
            Side::Read,
            &no_env,
            cfg.planner_usd,
            cfg.planner_timeout_s,
            plan_schema(),
        )?;
        let invoked = self.invoke_and_record(
            &InvocationFacts {
                job: &id,
                spec: &spec,
                composed: &composed,
                config: &jc.config,
                fake: self.is_fake(),
            },
            ctx.redactor,
        )?;
        let _ = crate::pin::remove_tree(&snap_dir);
        let (output, record) = match invoked {
            Ok(done) => done,
            Err(error) => {
                self.stop_planning(ctx, closing, &format!("the planner could not run: {error}"));
                return Ok(RoundEnd::Stopped);
            }
        };
        ctx.ui.line(&format!(
            "planner    {model} · {:.0}s · {}",
            output.wall_s,
            cost_text(&record)
        ));
        closing.new_invocations.push(record.clone());
        if output.outcome != InvocationOutcome::Success {
            let why = record.detail.clone().unwrap_or_default();
            self.stop_planning(
                ctx,
                closing,
                &format!("the planner did not complete: {why}"),
            );
            return Ok(RoundEnd::Stopped);
        }
        let plan_output = match output
            .structured
            .clone()
            .map(serde_json::from_value::<PlanOutput>)
        {
            Some(Ok(raw)) => redacted(raw, ctx.redactor)?,
            _ => {
                self.stop_planning(ctx, closing, "the planner returned no valid plan");
                return Ok(RoundEnd::Stopped);
            }
        };
        let mut plan = plan_record(&id, round, seq, plan_output, &|path| {
            Ok(self.git.show_file(&p, path)?.is_some())
        })?;
        let section = render_plan_section(&plan);
        plan.producer_section_sha256 = sha256_hex(section.as_bytes());
        if let Err(e) = plan.validate() {
            self.stop_planning(ctx, closing, &format!("the plan is invalid: {e}"));
            return Ok(RoundEnd::Stopped);
        }

        // Athena's own checks, at no model cost. A major finding sends the plan back without a critic call.
        let athena_findings = plan_findings(
            &plan,
            ctx.reading,
            &Policy {
                role: Role::Verifier,
                root: self.state.job_dir(&id),
                commands: verifier_commands,
                writes: false,
            },
            &|path| self.git.path_is_tree(&p, path).unwrap_or(false),
        );
        if athena_findings
            .iter()
            .any(|f| matches!(f.severity, Severity::Blocker | Severity::Major))
        {
            ctx.ui.line(&format!(
                "athena     plan round {round}: {} finding(s); no critic call",
                athena_findings.len()
            ));
            let critique = Critique {
                job: id.clone(),
                round,
                at: crate::now(),
                invocation_seq: None,
                agent_verdict: None,
                final_verdict: CritiqueVerdict::Revise,
                findings: athena_findings,
                request_reading_mismatch: None,
                summary: Some(
                    "Athena's own checks found a major problem in the plan; the critic was not called."
                        .into(),
                ),
            };
            return self.end_round(
                ctx,
                closing,
                plan,
                critique,
                section,
                last_round,
                cfg.max_rounds,
            );
        }

        // The critic.
        if let Err(e) = self.check_budget(
            jc,
            ctx.spent(closing),
            cfg.critic_usd + after,
            "critic, with the producer and verifier",
        ) {
            self.stop_planning(ctx, closing, &format!("{e}"));
            // The plan stands as a record even though no critic reviewed it.
            closing.plans.push(plan);
            return Ok(RoundEnd::Stopped);
        }
        let critic_brief_text = critic_brief(&CriticInputs {
            job: id.as_str().to_string(),
            request,
            reading: accepted.clone(),
            plan: plan.clone(),
            athena_findings: athena_findings.clone(),
        });
        let composed = compose(&[CONTRACT.into(), CRITIC_FRAME.into()]);
        Snapshot::create(&self.git, &p, &p, &snap_dir, SnapshotRepo::None)?;
        let model = ctx.reading.verifier_model.clone();
        let critic_seq = ctx.next_seq(closing);
        let spec = self.agent_spec(
            jc,
            &id,
            critic_seq,
            Role::Critic,
            &model,
            &composed,
            critic_brief_text,
            &grant,
            &snap_dir,
            Side::Read,
            &no_env,
            cfg.critic_usd,
            cfg.critic_timeout_s,
            critic_schema(),
        )?;
        let invoked = self.invoke_and_record(
            &InvocationFacts {
                job: &id,
                spec: &spec,
                composed: &composed,
                config: &jc.config,
                fake: self.is_fake(),
            },
            ctx.redactor,
        )?;
        let _ = crate::pin::remove_tree(&snap_dir);
        let (output, record) = match invoked {
            Ok(done) => done,
            Err(error) => {
                self.stop_planning(ctx, closing, &format!("the critic could not run: {error}"));
                return Ok(RoundEnd::Stopped);
            }
        };
        ctx.ui.line(&format!(
            "critic     {model} · {:.0}s · {}",
            output.wall_s,
            cost_text(&record)
        ));
        closing.new_invocations.push(record.clone());
        if output.outcome != InvocationOutcome::Success {
            let why = record.detail.clone().unwrap_or_default();
            self.stop_planning(ctx, closing, &format!("the critic did not complete: {why}"));
            return Ok(RoundEnd::Stopped);
        }
        let critic_output = match output
            .structured
            .clone()
            .map(serde_json::from_value::<CriticOutput>)
        {
            Some(Ok(raw)) => redacted(raw, ctx.redactor)?,
            _ => {
                self.stop_planning(ctx, closing, "the critic returned no valid critique");
                return Ok(RoundEnd::Stopped);
            }
        };
        let agent_verdict = match critic_output.verdict {
            CritiqueVerdictOutput::Accept => CritiqueVerdict::Accept,
            CritiqueVerdictOutput::Revise => CritiqueVerdict::Revise,
            CritiqueVerdictOutput::Reject => CritiqueVerdict::Reject,
        };
        // Athena's own findings (minor ones only, here) come first, then the critic's.
        let mut findings = athena_findings;
        findings.extend(critic_output.findings.iter().map(|f| Finding {
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
        let verdict = if agent_verdict == CritiqueVerdict::Accept
            && findings
                .iter()
                .any(|f| matches!(f.severity, Severity::Blocker | Severity::Major))
        {
            CritiqueVerdict::Revise
        } else {
            agent_verdict
        };
        let critique = Critique {
            job: id.clone(),
            round,
            at: crate::now(),
            invocation_seq: Some(critic_seq),
            agent_verdict: Some(agent_verdict),
            final_verdict: verdict,
            findings,
            request_reading_mismatch: critic_output.request_reading_mismatch.clone(),
            summary: Some(critic_output.summary.clone()),
        };
        if let Err(e) = critique.validate() {
            self.stop_planning(ctx, closing, &format!("the critique is invalid: {e}"));
            return Ok(RoundEnd::Stopped);
        }
        self.end_round(
            ctx,
            closing,
            plan,
            critique,
            section,
            last_round,
            cfg.max_rounds,
        )
    }

    /// Files a round's plan and critique and decides what follows. Athena decides the verdict as
    /// `verify::judge` does: the plan is accepted only when the critic accepts and neither source holds a
    /// blocker or major finding. Reject, a misread reading, or revise in the last round stops the job.
    #[allow(clippy::too_many_arguments)]
    fn end_round(
        &self,
        ctx: &RunContext<'_>,
        closing: &mut Closing,
        plan: JobPlan,
        critique: Critique,
        section: String,
        last_round: bool,
        max_rounds: u32,
    ) -> Result<RoundEnd> {
        let round = plan.round;
        let verdict = critique.final_verdict;
        let misread = critique.request_reading_mismatch.clone();
        let findings = critique.findings.clone();
        let summary = critique.summary.clone().unwrap_or_default();
        closing.plans.push(plan.clone());
        closing.critiques.push(critique);
        let mut stop = |why: String| -> Result<RoundEnd> {
            self.stop_planning(
                ctx,
                closing,
                &format!(
                    "{why} (round {round} of {max_rounds}). {summary} Findings: {}",
                    findings_line(&findings)
                ),
            );
            Ok(RoundEnd::Stopped)
        };
        if let Some(how) = misread {
            return stop(format!(
                "the critic judged that the reading misread the request: {how}"
            ));
        }
        match verdict {
            CritiqueVerdict::Accept => Ok(RoundEnd::Accepted(section)),
            CritiqueVerdict::Reject => stop("the critic rejected the plan".into()),
            CritiqueVerdict::Revise if last_round => {
                stop("the plan still needed revising in the last round".into())
            }
            CritiqueVerdict::Revise => Ok(RoundEnd::Revise(Box::new(PlanRevision {
                previous: plan,
                findings: findings.clone(),
            }))),
        }
    }

    /// For a capability that pins, resolves the commit to read and checks its tree out under the job's
    /// directory. Nothing is pinned for other capabilities.
    ///
    /// The rule: an existing study's `studies/<id>/pin.json` overrides `repo_ref`. When the committed head holds
    /// a study whose `repository_url` equals the configured `repo_url`, the job pins exactly that study's
    /// `sha`; athena.toml's `repo_ref` (a branch such as `main`) is used only for a new study, one with no
    /// pin.json for the repository. With several studies for the repository, the one whose `study_id` appears
    /// in the request text is used; if none or more than one is named, the job stops rather than guess.
    /// A pin.json whose `sha` is not a full 40-hex id, or a pin that does not resolve to that same sha, stops
    /// the job before dispatch with a message naming the study; it never falls back to `repo_ref`.
    ///
    /// The cause this fixes: job 20261015-201650-cyvb mounted fc8972cd (the tip of `main`) while the study's
    /// pin.json said 6407052f, so the producer cited lines of newer code under the older label.
    ///
    /// A pinning capability whose configuration names no repository, or a ref that does not resolve, also
    /// stops the job before dispatch.
    fn pin_repo(
        &self,
        capability: &dyn Capability,
        jc: &JobConfig,
        id: &JobId,
        head: &str,
        request: &str,
    ) -> Result<Option<(PinnedRepo, PathBuf)>> {
        if !capability.pins_repo() {
            return Ok(None);
        }
        let section = jc.config.capability(capability.id());
        let (Some(url), Some(git_ref)) = (
            section.and_then(|c| c.repo_url.clone()),
            section.and_then(|c| c.repo_ref.clone()),
        ) else {
            bail!(
                "capability {} studies a repository, but athena.toml [capabilities.{}] sets no repo_url and repo_ref to pin",
                capability.id(),
                capability.id()
            );
        };
        if let Some((_, rest)) = url.split_once("://")
            && rest.split('/').next().is_some_and(|a| a.contains('@'))
        {
            bail!("repo_url must not carry credentials; it is recorded in the job record");
        }
        let dir = self.state.job_dir(id).join("pin");
        crate::pin::remove_tree(&dir)?;
        if let Some((study, wanted)) = self.study_pin(head, &url, request)? {
            let sha = self
                .pinner
                .pin(&url, &wanted, &dir)
                .with_context(|| format!("pinning study {study}'s commit {wanted} of {url}"))?;
            if sha != wanted {
                bail!(
                    "study {study}: its pin.json names {wanted}, but the pin resolved to {sha}; the job would read a different commit"
                );
            }
            return Ok(Some((
                PinnedRepo {
                    url,
                    git_ref: wanted,
                    sha,
                },
                dir,
            )));
        }
        let sha = self
            .pinner
            .pin(&url, &git_ref, &dir)
            .with_context(|| format!("pinning {git_ref} of {url}"))?;
        if !is_full_sha(&sha) {
            bail!("the pinner returned {sha:?}, which is not a full 40-hex commit id");
        }
        Ok(Some((PinnedRepo { url, git_ref, sha }, dir)))
    }

    /// The `(study id, sha)` an existing study of the repository at `url` pins, read from `studies/*/pin.json`
    /// in the commit `head`; `None` when no study has a pin for that repository. See [`Self::pin_repo`].
    fn study_pin(&self, head: &str, url: &str, request: &str) -> Result<Option<(String, String)>> {
        let mut found: Vec<(String, String)> = Vec::new();
        for entry in self.git.ls_tree_under(head, "studies/")? {
            let Some(dir) = entry
                .path
                .strip_prefix("studies/")
                .and_then(|p| p.strip_suffix("/pin.json"))
                .filter(|d| !d.contains('/'))
            else {
                continue;
            };
            let bytes = self
                .git
                .show_file(head, &entry.path)?
                .with_context(|| format!("{} vanished", entry.path))?;
            let value: serde_json::Value = serde_json::from_slice(&bytes)
                .with_context(|| format!("study {dir}: {} is not valid JSON", entry.path))?;
            if value["repository_url"].as_str() != Some(url) {
                continue;
            }
            let study = value["study_id"].as_str().unwrap_or(dir).to_string();
            let sha = value["sha"].as_str().unwrap_or_default();
            if !is_full_sha(sha) {
                bail!(
                    "study {study}: pin.json sha {sha:?} is not a full 40-hex commit id, so the job cannot pin it; fix the pin"
                );
            }
            found.push((study, sha.to_string()));
        }
        match found.len() {
            0 => Ok(None),
            1 => Ok(found.pop()),
            _ => {
                let mut named: Vec<_> = found
                    .into_iter()
                    .filter(|(study, _)| request.contains(study.as_str()))
                    .collect();
                if named.len() == 1 {
                    Ok(named.pop())
                } else {
                    bail!(
                        "{url} has several studies and the request names {}; name exactly one study id so the job reads that study's pinned commit",
                        if named.is_empty() {
                            "none of them".to_string()
                        } else {
                            "more than one of them".to_string()
                        }
                    )
                }
            }
        }
    }

    /// The reading-validity rule (D38): a reading is valid only for the configuration inputs it was made
    /// under, which are the parsed `athena.toml` settings that bear on a reading
    /// ([`athena_data::config::Config::reading_inputs`]; `[parallel]` and `[landing]` do not) and the bytes
    /// of `defaults.toml`. When they differ, the request is read again here, and [`Athena::dispatch`]
    /// accepts the new reading for a flag-accepted caller when its scope is unchanged.
    fn reread_after_configuration_change(
        &self,
        jc: &JobConfig,
        records: &Records,
        job: &JobRecords,
        ui: &dyn Ui,
    ) -> Result<Proposal> {
        let id = job.id().clone();
        let n = job.readings.iter().map(|r| r.n).max().unwrap_or(0);
        if n >= jc.config.budget.max_readings {
            bail!("job {id} has {n} readings, the most a job may have; close it and ask again");
        }
        let redactor = self.redactor(&jc.config, Some(&id))?;
        let request = job.request.clone().context("the job has no request")?;
        let history: Vec<(Reading, Option<String>)> =
            job.readings.iter().map(|r| (r.clone(), None)).collect();
        let spent = job.spent_usd();
        let attempt = crate::reading::earlier_attempt(records, &request.text, &redactor)?;
        let (reading, invocation, error) = self.read_for(
            jc,
            records,
            &id,
            &request,
            attempt.as_ref(),
            &history,
            n + 1,
            job.next_seq(),
            spent,
            &redactor,
            ui,
        )?;
        let mut files = vec![NewFile::record(&AnyRecord::Invocation(Box::new(
            invocation.clone(),
        )))];
        if let Some(r) = &reading {
            files.push(NewFile::record(&AnyRecord::Reading(Box::new(r.clone()))));
        }
        let commit = self.commit_records(
            &files,
            &message(
                "Read again: the configuration changed before dispatch",
                "A reading is valid only for the configuration it was made under (D38).",
                &[("Athena-Job", id.as_str().to_string())],
            ),
        )?;
        Ok(Proposal {
            job: id,
            reading,
            reader_error: error,
            spent_usd: spent + invocation.cost.best_lower(),
            invocation: Some(invocation),
            commit,
        })
    }
}

/// How one planning round ended.
enum RoundEnd {
    /// The plan was accepted; the producer's section, rendered from it.
    Accepted(String),
    /// The planner is asked again, with the previous plan and every finding.
    Revise(Box<PlanRevision>),
    /// The job failed in planning; `closing` holds the failure.
    Stopped,
}

/// Findings on one line each, joined by `; `, for a failure reason.
fn findings_line(findings: &[Finding]) -> String {
    if findings.is_empty() {
        return "(none recorded)".into();
    }
    findings
        .iter()
        .map(|f| {
            let location = f
                .location
                .as_deref()
                .map_or(String::new(), |l| format!(" ({l})"));
            format!(
                "{}: {}{location}",
                format!("{:?}", f.severity).to_lowercase(),
                f.summary.trim()
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
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

/// Redacts every string of an agent's structured output, by redacting its JSON text and reading it back.
fn redacted<T: serde::Serialize + serde::de::DeserializeOwned>(
    value: T,
    redactor: &Redactor,
) -> Result<T> {
    let mut text = vec![serde_json::to_string(&value)?];
    redactor.redact_all(&mut text)?;
    serde_json::from_str(&text[0]).context("the redacted output is no longer valid")
}

/// The `athena.job-plan/1` record of a planner's output. Athena sets whether each file is governance and
/// whether it exists. The caller sets `producer_section_sha256` once the section is rendered.
fn plan_record(
    job: &JobId,
    round: u32,
    invocation_seq: u32,
    out: PlanOutput,
    exists: &dyn Fn(&str) -> Result<bool>,
) -> Result<JobPlan> {
    let mut files = Vec::new();
    for f in out.files {
        files.push(PlanFile {
            governance: is_governance(&f.path),
            exists: exists(&f.path)?,
            action: match f.action {
                DeliverableActionOutput::Create => DeliverableAction::Create,
                DeliverableActionOutput::Modify => DeliverableAction::Modify,
                DeliverableActionOutput::Delete => DeliverableAction::Delete,
            },
            path: f.path,
            purpose: f.purpose,
        });
    }
    Ok(JobPlan {
        job: job.clone(),
        round,
        at: crate::now(),
        invocation_seq,
        approach: out.approach,
        decisions: out
            .decisions
            .into_iter()
            .map(|d| PlanDecision {
                choice: d.choice,
                reason: d.reason,
                rejected: d
                    .rejected
                    .into_iter()
                    .map(|r| RejectedOption {
                        option: r.option,
                        why: r.why,
                    })
                    .collect(),
            })
            .collect(),
        files,
        steps: out.steps,
        risks: out
            .risks
            .into_iter()
            .map(|r| PlanRisk {
                risk: r.risk,
                mitigation: r.mitigation,
            })
            .collect(),
        demonstrations: out
            .demonstrations
            .into_iter()
            .map(|d| Demonstration {
                criterion: d.criterion,
                method: d.method,
                command: d.command,
                observes: d.observes,
                fails_without: d.fails_without,
            })
            .collect(),
        open_questions: out.open_questions,
        producer_section_sha256: "0".repeat(64),
    })
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

/// The accepted criteria the report does not show as met: marked unmet, or missing from `criteria_check`.
/// A criterion matches an entry by its text with numbering, kind tag, case, spacing and trailing
/// punctuation ignored (see [`criterion_key`]); any other difference in words is a different criterion.
pub(crate) fn unmet_criteria(report: &ProducerOutput, accepted: &AcceptedReading) -> Vec<String> {
    accepted
        .criteria
        .iter()
        .map(|(text, _)| text)
        .filter(|text| {
            let key = criterion_key(text);
            !report.criteria_check.iter().any(|c| {
                c.status == CriterionStatusOutput::Met && criterion_key(&c.criterion) == key
            })
        })
        .cloned()
        .collect()
}

/// A criterion's text reduced for comparison: no leading number (`1.`, `2)`, `3:`), no leading `[kind]`
/// tag, lowercase, single spaces, no trailing punctuation.
fn criterion_key(text: &str) -> String {
    let mut rest = text.trim();
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    if digits > 0
        && let Some(after) = rest[digits..].strip_prefix(['.', ')', ':'])
    {
        rest = after.trim_start();
    }
    if let Some(inner) = rest.strip_prefix('[')
        && let Some(end) = inner.find(']')
        && inner[..end].chars().all(|c| c.is_ascii_alphabetic())
    {
        rest = inner[end + 1..].trim_start();
    }
    let words = rest.split_whitespace().collect::<Vec<_>>().join(" ");
    words
        .trim_end_matches(|c: char| c.is_ascii_punctuation() && c != '`' && c != ')')
        .trim_end()
        .to_lowercase()
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
        let mut seq = ctx.next_seq(&closing);

        // The baseline: the configured checks on the base commit, before the producer starts. It is
        // recorded only and never stops the job.
        let baseline = self.take_baseline(ctx, &job_dir, seq);
        let baseline_line: Vec<String> = baseline
            .checks
            .iter()
            .map(|r| format!("{} {}", r.name, if r.passed { "ok" } else { "FAILED" }))
            .collect();
        ctx.ui.line(&format!(
            "baseline   {}{}",
            if baseline_line.is_empty() {
                "no checks".to_string()
            } else {
                baseline_line.join(" · ")
            },
            baseline
                .note
                .as_ref()
                .map(|n| format!(" ({n})"))
                .unwrap_or_default()
        ));
        closing.baseline = Some(baseline);

        // The producer, in a detached snapshot of the base with a one-commit repository of its own.
        let work = job_dir.join("work");
        if work.exists() {
            crate::pin::remove_tree(&work)?;
        }
        let snapshot = Snapshot::create(&self.git, &p, &p, &work, SnapshotRepo::Base)?;
        if let Some(pin_dir) = &ctx.pin_dir {
            mount(pin_dir, snapshot.dir())?;
        }
        let grant = ctx.capability.producer_grant(jc);
        let composed = compose(&[
            CONTRACT.into(),
            PRODUCER_FRAME.into(),
            Component::from(ctx.capability.producer_section()),
        ]);
        let capability_env = ctx.capability.env(jc);
        let model = ctx.reading.producer_model.clone();
        // A report that leaves acceptance criteria unmet or unreported is sent back to the producer, in
        // the same working directory, up to `PRODUCER_SEND_BACKS` times.
        let mut attempt_brief = brief.to_string();
        let mut send_backs = 0;
        // What the last report still lacked after the send-back, and where that report is kept.
        let mut report_gaps: Vec<String> = Vec::new();
        let mut report_kept_in = String::new();
        let report = loop {
            let spec = self.agent_spec(
                jc,
                &id,
                seq,
                Role::Producer,
                &model,
                &composed,
                attempt_brief.clone(),
                &grant,
                snapshot.dir(),
                Side::Produce,
                &capability_env,
                jc.config.budget.producer_usd,
                jc.config.budget.producer_timeout_s,
                producer_schema(),
            )?;
            seq += 1;
            let facts = InvocationFacts {
                job: &id,
                spec: &spec,
                composed: &composed,
                config: &jc.config,
                fake: self.is_fake(),
            };
            let (output, record) = match self.invoke_and_record(&facts, ctx.redactor)? {
                Ok(done) => done,
                Err(error) => {
                    closing.happened =
                        self.failure(ctx, &format!("the producer could not run: {error}"));
                    return Ok(closing);
                }
            };
            ctx.ui.line(&format!(
                "producer   {model} · {:.0}s · {} · {} commands run",
                output.wall_s,
                cost_text(&record),
                record.commands_run.len()
            ));
            closing.new_invocations.push(record.clone());
            if output.outcome != InvocationOutcome::Success {
                closing.happened = self.failure(
                    ctx,
                    &format!(
                        "the producer did not complete: {}",
                        record.detail.clone().unwrap_or_default()
                    ),
                );
                return Ok(closing);
            }
            let raw = match output
                .structured
                .clone()
                .map(serde_json::from_value::<ProducerOutput>)
            {
                Some(Ok(raw)) => raw,
                _ => {
                    closing.happened = self.failure(ctx, "the producer returned no valid report");
                    return Ok(closing);
                }
            };
            let unmet = unmet_criteria(&raw, accepted);
            let report = self.redact_report(raw, ctx.redactor)?;
            if unmet.is_empty() {
                break Some(report);
            }
            let problems = vec![format!(
                "acceptance criteria not met or not reported: {}",
                unmet
                    .iter()
                    .map(|c| format!("\"{c}\""))
                    .collect::<Vec<_>>()
                    .join("; ")
            )];
            // Keep the redacted report that failed the check: the attempt's invocation record carries it
            // in `detail`, so each attempt's report stays with the job's records.
            let kept = {
                let attempt = closing
                    .new_invocations
                    .last_mut()
                    .expect("the attempt's invocation was just recorded");
                attempt.detail = Some(format!(
                    "self-check failed: {}; redacted report: {}",
                    problems.join("; "),
                    serde_json::to_string(&report)?
                ));
                format!(
                    "the detail of {}",
                    AnyRecord::Invocation(Box::new(attempt.clone())).canonical_path()
                )
            };
            if send_backs >= PRODUCER_SEND_BACKS {
                // The work may be finished even though the report is not: the verifier judges it. A run
                // that changed nothing still fails, below, after the loop.
                ctx.ui.line(&format!(
                    "producer   report still incomplete after the send-back: {}; the verifier will judge it",
                    problems.join("; ")
                ));
                report_gaps = problems;
                report_kept_in = kept;
                break Some(report);
            }
            send_backs += 1;
            ctx.ui
                .line(&format!("producer   sent back: {}", problems.join("; ")));
            let mut sent_back = format!("{brief}\n\n## Sent back\n\n");
            sent_back.push_str(
                "Your previous report did not show every acceptance criterion as met in criteria_check. \
                 Your edits are still in your working directory. Fix the work for each criterion below, \
                 then report every acceptance criterion in criteria_check, with its text exactly as given \
                 (without its number or kind tag), its status and your evidence. This is the only send-back: \
                 if your changes are still in place afterwards, they go to the verifier with the criteria \
                 your report still leaves unmet or unreported flagged, and the verifier judges them. \
                 A run that leaves no change fails.\n\nCriteria not met or not reported:\n\n",
            );
            for criterion in &unmet {
                sent_back.push_str(&format!("- {criterion}\n"));
            }
            attempt_brief = sent_back;
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
        if self.nested.is_none() {
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
        // A check that failed on this job's base commit and fails again is marked pre-existing. A baseline
        // taken on any other commit says nothing about this base.
        let usable_baseline = closing
            .baseline
            .as_ref()
            .filter(|b| b.base_commit == p.as_str());
        mark_pre_existing(&mut check_runs, usable_baseline);

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
                     by it; judge them yourself from the recorded check results: {}",
                    report_gaps.join("; ")
                ),
            });
        }
        // The verifier sees the accepted plan's demonstrations and planned files, never its approach or
        // reasons (review F5). Without a plan the brief is as it always was.
        let verifier_plan = closing.plans.last().map(VerifierPlan::from_plan);
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
            system_fault: system_fault.clone(),
            verifier_failure: verifier_failure.clone(),
            map_base: map_base.as_deref(),
            map_candidate: map_candidate.as_deref(),
            baseline: usable_baseline,
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

    /// Commits the closing records of a dispatched job, landing the change when it was verified.
    fn close_dispatched(&self, ctx: &RunContext<'_>, mut closing: Closing) -> Result<GoResult> {
        let id = ctx.id().clone();
        let mut all_invocations = ctx.job.invocations.clone();
        all_invocations.extend(closing.new_invocations.iter().cloned());
        let totals = totals(&all_invocations, closing.happened.wall_time_execution_s);
        // The score reads the interventions on the commit the closing commit is built on, so it is
        // computed for each attempt; the gate recomputes it from that commit (M27).
        let scored_at = crate::now();
        let candidate_record = closing.candidate.clone();
        let verification_record = closing.verification.clone();
        let wall_time = closing.happened.wall_time_execution_s;
        let score_on = |head: &str| -> Result<Score> {
            let records = Records::load(&self.git, head)?;
            Ok(score(
                ctx.job,
                &records,
                &all_invocations,
                scoring::Closing {
                    candidate: candidate_record.as_ref(),
                    verification: verification_record.as_ref(),
                    wall_time_execution_s: wall_time,
                },
                None,
                scored_at,
            ))
        };
        for inv in &closing.new_invocations {
            closing
                .files
                .push(NewFile::record(&AnyRecord::Invocation(Box::new(
                    inv.clone(),
                ))));
        }
        for plan in &closing.plans {
            closing
                .files
                .push(NewFile::record(&AnyRecord::JobPlan(Box::new(plan.clone()))));
        }
        for critique in &closing.critiques {
            closing
                .files
                .push(NewFile::record(&AnyRecord::Critique(Box::new(
                    critique.clone(),
                ))));
        }
        if let Some(baseline) = &closing.baseline {
            closing
                .files
                .push(NewFile::record(&AnyRecord::Baseline(Box::new(
                    baseline.clone(),
                ))));
        }
        if let Some(candidate) = &closing.candidate {
            closing
                .files
                .push(NewFile::record(&AnyRecord::Candidate2(Box::new(
                    candidate.clone(),
                ))));
        }
        if let Some(verification) = &closing.verification {
            closing
                .files
                .push(NewFile::record(&AnyRecord::Verification(Box::new(
                    verification.clone(),
                ))));
        }
        let reading_path = format!("{}/reading-{}.json", id.dir(), ctx.reading.n);
        let prediction = introducing_commit(self, "HEAD", &reading_path)?
            .unwrap_or_else(|| ctx.prediction.clone());
        let mut state = closing.happened.state.unwrap_or(OutcomeState::Failed);
        if state == OutcomeState::Landed && closing.verified.is_none() {
            state = OutcomeState::Rejected;
        }
        let reason =
            closing
                .happened
                .reason
                .clone()
                .unwrap_or_else(|| match &closing.verification {
                    Some(v) if v.final_verdict == Verdict::Pass => "verified".into(),
                    Some(v) => format!("the final verdict is fail: {}", v.reasons.join("; ")),
                    None => "failed before verification".into(),
                });
        let branch = self.git.current_branch().ok().flatten();
        let totals_for_outcome = totals.clone();
        let candidate_commit = closing
            .candidate
            .as_ref()
            .and_then(|c| c.candidate_commit.clone());
        let base_commit = ctx.prediction.clone();
        let outcome_id = id.clone();
        let outcome = move |state: OutcomeState, reason: String, after_blocked: bool| Outcome {
            job: outcome_id.clone(),
            state,
            reason,
            at: crate::now(),
            totals: Some(totals_for_outcome.clone()),
            branch: branch.clone(),
            base_commit: Some(base_commit.clone()),
            candidate_commit: candidate_commit.clone(),
            after_blocked,
            through_commit: None,
        };
        let mut trailers = vec![
            ("Athena-Job", id.as_str().to_string()),
            ("Athena-Prediction", prediction.clone()),
            ("Athena-Class", ctx.reading.class_of_work.clone()),
            ("Athena-Capability", ctx.capability.id().to_string()),
            ("Athena-Producer", ctx.reading.producer_model.clone()),
        ];
        let verdict_word = closing.verification.as_ref().map(|v| {
            if v.final_verdict == Verdict::Pass {
                "pass"
            } else {
                "fail"
            }
        });
        if let Some(word) = verdict_word {
            trailers.push((
                "Athena-Verifier",
                format!("{} {word}", ctx.reading.verifier_model),
            ));
        }
        trailers.push((
            "Athena-Cost-USD",
            format!(
                "{:.4} {}",
                totals.cost_all.best_lower(),
                provenance_word(&totals.cost_all)
            ),
        ));

        if state == OutcomeState::Landed
            && let (Some(v), Some(verification)) =
                (closing.verified.clone(), closing.verification.clone())
        {
            let mut landing_trailers = trailers.clone();
            landing_trailers.push(("Athena-Verified-Content", v.fingerprint().to_string()));
            let co_author = format!(
                "{} <noreply@anthropic.com>",
                ctx.jc.config.model_name(&ctx.reading.producer_model)
            );
            landing_trailers.push(("Co-Authored-By", co_author));
            let body = landing_body(ctx.reading, &verification);
            let landing_message = message(&ctx.reading.title, &body, &landing_trailers);
            return self.land_or_block(
                ctx,
                &v,
                &closing,
                &landing_message,
                &trailers,
                outcome,
                totals,
                &score_on,
                verification,
            );
        }
        let last_score: std::cell::RefCell<Option<Score>> = std::cell::RefCell::new(None);
        let subject = closing_subject(state, &ctx.reading.title);
        let commit = self.commit_records_with(
            &|head: &str| {
                let score = score_on(head)?;
                let mut files = closing.files.clone();
                files.push(NewFile::record(&AnyRecord::Score(Box::new(score.clone()))));
                files.push(NewFile::record(&AnyRecord::Outcome(Box::new(outcome(
                    state,
                    reason.clone(),
                    false,
                )))));
                *last_score.borrow_mut() = Some(score);
                Ok(files)
            },
            &message(&subject, &reason, &trailers),
        )?;
        let score = last_score
            .into_inner()
            .expect("a committed closing has a score");
        ctx.ui.line(&format!("{:<10} {reason}", state.as_str()));
        crate::state::report_clean(ctx.ui, &self.state.clean_job(&id));
        Ok(GoResult::Finished(Box::new(Finished {
            job: id,
            state,
            reason,
            reading: ctx.reading.clone(),
            prediction_commit: Some(prediction),
            closing_commit: commit,
            landed_commit: None,
            totals: Some(totals),
            score: Some(score),
            verification: closing.verification,
        })))
    }

    #[allow(clippy::too_many_arguments)]
    fn land_or_block(
        &self,
        ctx: &RunContext<'_>,
        verified: &Verified,
        closing: &Closing,
        landing_message: &str,
        trailers: &[(&str, String)],
        outcome: impl Fn(OutcomeState, String, bool) -> Outcome,
        totals: Totals,
        score_on: &dyn Fn(&str) -> Result<Score>,
        verification: Verification,
    ) -> Result<GoResult> {
        let id = ctx.id().clone();
        let _lock = self.state.lock_repository()?;
        let mut result: Option<(OutcomeState, String, String, Option<String>)> = None;
        let mut score = None;
        for _attempt in 0..3 {
            let head = self.head()?;
            let attempt_score = score_on(&head)?;
            let mut closing_files = closing.files.clone();
            closing_files.push(NewFile::record(&AnyRecord::Score(Box::new(
                attempt_score.clone(),
            ))));
            score = Some(attempt_score);
            match plan(&self.git, verified, &head)? {
                Plan::Stale(reason) => {
                    let mut files = closing_files.clone();
                    files.push(NewFile::record(&AnyRecord::Outcome(Box::new(outcome(
                        OutcomeState::Stale,
                        reason.clone(),
                        false,
                    )))));
                    let commit = self.build_commit(
                        &head,
                        &head,
                        &files,
                        &message(
                            &closing_subject(OutcomeState::Stale, &ctx.reading.title),
                            &reason,
                            trailers,
                        ),
                    )?;
                    match self.advance(&commit, &head)? {
                        Ok(()) => {
                            result = Some((OutcomeState::Stale, reason, commit, None));
                            break;
                        }
                        Err(Refusal::Moved) => continue,
                        Err(Refusal::LocalEdits(e)) => {
                            bail!("could not record the stale outcome: {e}")
                        }
                    }
                }
                Plan::Tree(tree) => {
                    let mut files = closing_files.clone();
                    files.push(NewFile::record(&AnyRecord::Outcome(Box::new(outcome(
                        OutcomeState::Landed,
                        "verified and landed".into(),
                        false,
                    )))));
                    let commit = match self.build_commit(&tree, &head, &files, landing_message) {
                        Ok(commit) => commit,
                        Err(refused) => {
                            // The in-process gate refused the landing commit: the job fails, with its records.
                            let reason =
                                format!("the gate refused the landing commit: {refused:#}");
                            let mut files = closing_files.clone();
                            files.push(NewFile::record(&AnyRecord::Outcome(Box::new(outcome(
                                OutcomeState::Failed,
                                reason.clone(),
                                false,
                            )))));
                            let records_commit = self.build_commit(
                                &head,
                                &head,
                                &files,
                                &message(
                                    &closing_subject(OutcomeState::Failed, &ctx.reading.title),
                                    &reason,
                                    trailers,
                                ),
                            )?;
                            match self.advance(&records_commit, &head)? {
                                Ok(()) => {
                                    result =
                                        Some((OutcomeState::Failed, reason, records_commit, None));
                                    break;
                                }
                                Err(Refusal::Moved) => continue,
                                Err(Refusal::LocalEdits(e)) => {
                                    bail!("could not record the failed outcome: {e}")
                                }
                            }
                        }
                    };
                    match self.advance(&commit, &head)? {
                        Ok(()) => {
                            result = Some((
                                OutcomeState::Landed,
                                "verified and landed".into(),
                                commit.clone(),
                                Some(commit),
                            ));
                            break;
                        }
                        Err(Refusal::Moved) => continue,
                        Err(Refusal::LocalEdits(stderr)) => {
                            let reason = format!(
                                "verified, but local edits in the checkout conflict with the change: {}",
                                stderr.trim()
                            );
                            let mut files = closing_files.clone();
                            files.push(NewFile::record(&AnyRecord::Outcome(Box::new(outcome(
                                OutcomeState::Blocked,
                                reason.clone(),
                                false,
                            )))));
                            let records_commit = self.build_commit(
                                &head,
                                &head,
                                &files,
                                &message(
                                    &closing_subject(OutcomeState::Blocked, &ctx.reading.title),
                                    &reason,
                                    trailers,
                                ),
                            )?;
                            match self.advance(&records_commit, &head)? {
                                Ok(()) => {
                                    result =
                                        Some((OutcomeState::Blocked, reason, records_commit, None));
                                    break;
                                }
                                Err(Refusal::Moved) => continue,
                                Err(Refusal::LocalEdits(e)) => {
                                    bail!("could not record the blocked outcome: {e}")
                                }
                            }
                        }
                    }
                }
            }
        }
        let Some((state, reason, commit, landed)) = result else {
            bail!("the branch kept moving while Athena landed; run `athena go {id}` again")
        };
        match state {
            OutcomeState::Landed => ctx.ui.line(&format!(
                "landed     {} \"{}\"{}",
                short(&commit),
                ctx.reading.title,
                branch_suffix(self)
            )),
            OutcomeState::Blocked => ctx.ui.line(&format!(
                "blocked    {reason}. Resolve the local edits, then `athena go {id}`"
            )),
            other => ctx.ui.line(&format!("{:<10} {reason}", other.as_str())),
        }
        if state != OutcomeState::Blocked {
            crate::state::report_clean(ctx.ui, &self.state.clean_job(&id));
        }
        Ok(GoResult::Finished(Box::new(Finished {
            job: id,
            state,
            reason,
            reading: ctx.reading.clone(),
            prediction_commit: trailers
                .iter()
                .find(|(k, _)| *k == "Athena-Prediction")
                .map(|(_, v)| v.clone()),
            closing_commit: commit,
            landed_commit: landed,
            totals: Some(totals),
            score,
            verification: Some(verification),
        })))
    }

    /// `go` on a blocked job: re-runs the landing algorithm and lands with `outcome-landed.json`.
    fn retry_landing(&self, _records: &Records, job: &JobRecords, ui: &dyn Ui) -> Result<Finished> {
        let id = job.id().clone();
        let verification = job
            .verification
            .clone()
            .context("a blocked job has a verification record")?;
        let candidate = job
            .candidate
            .clone()
            .context("a blocked job has a candidate record")?;
        let reading = job
            .job
            .as_ref()
            .and_then(|j| j.accepted_reading)
            .and_then(|n| job.reading(n))
            .cloned()
            .context("the accepted reading")?;
        let base = candidate.base_commit.clone();
        let f_now = fingerprint(&self.git, &verification.candidate_commit, &base)?;
        let Some(v) = Verified::from_record(&verification, &base, &f_now) else {
            bail!("job {id}'s verification does not hold for its candidate any more; ask again");
        };
        let outcome = job
            .outcome
            .clone()
            .context("a blocked job has an outcome")?;
        let mut trailers_found = Vec::new();
        for commit in self
            .git
            .log_grep("HEAD", &format!("^Athena-Job: {}$", id.as_str()))?
        {
            let found = self.git.trailers(&self.git.message(&commit)?)?;
            if found.iter().any(|(k, _)| k == "Athena-Prediction") {
                trailers_found = found;
                break;
            }
        }
        if trailers_found.is_empty() {
            bail!("the blocked closing commit of job {id} was not found");
        }
        let mut trailers: Vec<(&str, String)> = Vec::new();
        for (k, v) in &trailers_found {
            let key: &str = match k.as_str() {
                "Athena-Job" => "Athena-Job",
                "Athena-Prediction" => "Athena-Prediction",
                "Athena-Class" => "Athena-Class",
                "Athena-Capability" => "Athena-Capability",
                "Athena-Producer" => "Athena-Producer",
                "Athena-Verifier" => "Athena-Verifier",
                "Athena-Cost-USD" => "Athena-Cost-USD",
                _ => continue,
            };
            trailers.push((key, v.clone()));
        }
        trailers.push(("Athena-Verified-Content", v.fingerprint().to_string()));
        let config = athena_gate::config_at(&self.git, Some(&base))?;
        trailers.push((
            "Co-Authored-By",
            format!(
                "{} <noreply@anthropic.com>",
                config.model_name(&reading.producer_model)
            ),
        ));
        let landed_outcome = Outcome {
            job: id.clone(),
            state: OutcomeState::Landed,
            reason: "verified, blocked by local edits, and landed on retry".into(),
            at: crate::now(),
            totals: outcome.totals.clone(),
            branch: self.git.current_branch().ok().flatten(),
            base_commit: Some(base.clone()),
            candidate_commit: Some(verification.candidate_commit.clone()),
            after_blocked: true,
            through_commit: None,
        };
        let files = vec![NewFile::record(&AnyRecord::Outcome(Box::new(
            landed_outcome,
        )))];
        let body = landing_body(&reading, &verification);
        let landing_message = message(&reading.title, &body, &trailers);
        let _lock = self.state.lock_repository()?;
        for _attempt in 0..3 {
            let head = self.head()?;
            match plan(&self.git, &v, &head)? {
                Plan::Stale(reason) => bail!(
                    "job {id} can no longer land: {reason}. It stays blocked; `athena close {id} --why \"...\"` abandons it, and a new ask redoes the work"
                ),
                Plan::Tree(tree) => {
                    let commit = self.build_commit(&tree, &head, &files, &landing_message)?;
                    match self.advance(&commit, &head)? {
                        Ok(()) => {
                            ui.line(&format!(
                                "landed     {} \"{}\"{}",
                                short(&commit),
                                reading.title,
                                branch_suffix(self)
                            ));
                            crate::state::report_clean(ui, &self.state.clean_job(&id));
                            return Ok(Finished {
                                job: id,
                                state: OutcomeState::Landed,
                                reason: "landed on retry".into(),
                                reading,
                                prediction_commit: trailers
                                    .iter()
                                    .find(|(k, _)| *k == "Athena-Prediction")
                                    .map(|(_, v)| v.clone()),
                                closing_commit: commit.clone(),
                                landed_commit: Some(commit),
                                totals: outcome.totals,
                                score: job.latest_score().cloned(),
                                verification: Some(verification),
                            });
                        }
                        Err(Refusal::Moved) => continue,
                        Err(Refusal::LocalEdits(stderr)) => bail!(
                            "job {id} is still blocked: local edits conflict with it: {}. Resolve them, then `athena go {id}`",
                            stderr.trim()
                        ),
                    }
                }
            }
        }
        bail!("the branch kept moving while Athena landed; run `athena go {id}` again")
    }
}

impl Happened {
    fn with_reason(mut self, reason: &str) -> Happened {
        self.reason = Some(reason.to_string());
        self
    }
}

/// The subject of a closing commit: the outcome, then the job's title (the job id is in the trailer).
pub fn closing_subject(state: OutcomeState, title: &str) -> String {
    let word = state.as_str();
    let mut chars = word.chars();
    let capitalized = chars
        .next()
        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default();
    format!("{capitalized}: {title}")
}

fn provenance_word(q: &athena_data::Quantity) -> &'static str {
    match q.provenance() {
        athena_data::Provenance::Measured => "measured",
        athena_data::Provenance::Reported => "reported",
        athena_data::Provenance::Estimated => "estimated",
        athena_data::Provenance::Pending => "pending",
    }
}

fn cost_text(record: &Invocation) -> String {
    match record.cost.value() {
        Some(v) => format!("${v:.2}"),
        None => format!("≤ ${:.2} (estimated)", record.cost.upper().unwrap_or(0.0)),
    }
}

fn branch_suffix(athena: &Athena) -> String {
    match athena.git.current_branch().ok().flatten() {
        Some(branch) => format!(" on {branch}"),
        None => " (detached HEAD)".into(),
    }
}

fn landing_body(reading: &Reading, verification: &Verification) -> String {
    let mut body = format!("{}\n\nCriteria met:\n", reading.summary.trim());
    for (i, c) in verification.criteria.iter().enumerate() {
        body.push_str(&format!(
            "{}. [{}] {}\n",
            i + 1,
            if c.met { "met" } else { "unmet" },
            c.criterion
        ));
    }
    let minor = verification.findings.len();
    body.push_str(&format!(
        "\nVerified independently: final verdict pass; {} checks passed; {minor} finding(s) of minor or lower severity.",
        verification.checks.len()
    ));
    if let Some(summary) = &verification.summary {
        body.push_str(&format!("\n\nVerifier: {}", summary.trim()));
    }
    body
}

/// Re-exported for the CLI: the build that recorded an invocation.
pub fn build_text(build: &Build) -> String {
    format!(
        "{} ({}{})",
        build.version,
        short(&build.source_commit),
        if build.dirty { ", dirty" } else { "" }
    )
}
