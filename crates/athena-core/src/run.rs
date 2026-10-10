//! `athena go`: dispatch, produce, verify independently, and land (design §2.3, §3.4, §7).

use crate::admission::{AdmissionInput, Changed, admit, is_governance};
use crate::capability::{Capability, ChangeList, JobConfig};
use crate::checks::{CheckContext, run_check};
use crate::commits::{NewFile, Refusal, message};
use crate::invocation::InvocationFacts;
use crate::land::{Plan, plan};
use crate::reading::Proposal;
use crate::scoring::{self, Happened, score, totals};
use crate::store::{JobRecords, JobState, Records};
use crate::verify::{VerdictInputs, Verified, judge, verified};
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_agent::env::{EnvSpec, Side};
use athena_context::briefs::{
    AcceptedReading, CandidateView, CheckResultView, CheckView, ProducerInputs, VerifierInputs,
    producer_brief, verifier_brief,
};
use athena_context::outputs::{ProducerOutput, VerdictOutput, producer_schema, verdict_schema};
use athena_context::{CONTRACT, Component, PRODUCER_FRAME, VERIFIER_FRAME, compose};
use athena_data::records::{
    AcceptedBy, AnyRecord, Build, Candidate, Caps, ChangeEntry, ChangeStatus, CheckRun, Invocation,
    InvocationOutcome, Job, JobKind, Outcome, OutcomeState, Owner, PolicyViolation, ProducerReport,
    Reading, ReportedCheck, Role, Score, Totals, Verdict, Verification,
};
use athena_data::{JobId, sha256_hex};
use athena_gate::redact::Redactor;
use athena_git::attrs::fingerprint;
use athena_git::index::{IndexEdit, TempIndex};
use athena_git::snapshot::{Snapshot, SnapshotRepo};
use std::collections::BTreeSet;
use std::time::Instant;

/// How `go` ended.
#[derive(Debug, Clone)]
pub enum GoResult {
    /// The configuration changed since the reading: Athena read again, and the new reading needs accepting.
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
}

impl Closing {
    fn new() -> Closing {
        Closing {
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
    pub fn go(&self, job_ref: &str, accepted_by: AcceptedBy, ui: &dyn Ui) -> Result<GoResult> {
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
                if self
                    .state
                    .registry()
                    .entries()
                    .iter()
                    .any(|e| e.job == id.as_str())
                {
                    bail!("job {id} is running");
                }
                bail!(
                    "job {id} was interrupted: its process died before it closed. `athena close {id} --why \"...\"` records what happened"
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
            ui.line("reading    athena.toml or defaults.toml changed since this reading; reading again under the new configuration");
            return self
                .reread_after_configuration_change(&jc, records, job, ui)
                .map(|p| GoResult::Reread(Box::new(p)));
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

        // The prediction commit: the accepted reading's job record and the producer brief, before any work.
        let producer_grant = capability.producer_grant(&jc);
        let checks = capability.checks(&jc);
        let accepted = AcceptedReading::new(&reading, &jc.defaults, is_governance);
        let brief = producer_brief(&ProducerInputs {
            job: id.as_str().to_string(),
            request: job
                .request
                .as_ref()
                .map(|r| r.text.clone())
                .unwrap_or_default(),
            reading: accepted.clone(),
            checks: checks
                .iter()
                .map(|c| CheckView {
                    name: c.name.clone(),
                    command: c.run.join(" "),
                })
                .collect(),
            commands: athena_agent::policy::describe_commands(&producer_grant.commands),
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
        };
        let closing = self.produce_and_verify(&ctx, &brief, &accepted)?;
        self.close_dispatched(&ctx, closing)
    }

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
        let (reading, invocation, error) = self.read_for(
            jc,
            records,
            &id,
            &request,
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
}

impl Athena {
    fn produce_and_verify(
        &self,
        ctx: &RunContext<'_>,
        brief: &str,
        accepted: &AcceptedReading,
    ) -> Result<Closing> {
        let mut closing = Closing::new();
        let id = ctx.id().clone();
        let jc = ctx.jc;
        let job_dir = self.state.job_dir(&id);
        let p = ctx.prediction.clone();
        let mut seq = ctx.job.next_seq();

        // The producer, in a detached snapshot of the base with a one-commit repository of its own.
        let work = job_dir.join("work");
        if work.exists() {
            std::fs::remove_dir_all(&work)?;
        }
        let snapshot = Snapshot::create(&self.git, &p, &p, &work, SnapshotRepo::Base)?;
        let grant = ctx.capability.producer_grant(jc);
        let composed = compose(&[
            CONTRACT.into(),
            PRODUCER_FRAME.into(),
            Component::from(ctx.capability.producer_section()),
        ]);
        let capability_env = ctx.capability.env(jc);
        let model = ctx.reading.producer_model.clone();
        let spec = self.agent_spec(
            jc,
            &id,
            seq,
            Role::Producer,
            &model,
            &composed,
            brief.to_string(),
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
        let report = match output
            .structured
            .clone()
            .map(serde_json::from_value::<ProducerOutput>)
        {
            Some(Ok(report)) => Some(self.redact_report(report, ctx.redactor)?),
            _ => None,
        };
        if report.is_none() {
            closing.happened = self.failure(ctx, "the producer returned no valid report");
            return Ok(closing);
        }

        // The candidate: the snapshot's change list, applied to the base's full tree in the user's repository.
        let changes = snapshot.change_list()?;
        let mut candidate = Candidate {
            job: id.clone(),
            base_commit: p.clone(),
            candidate_commit: None,
            candidate_tree: None,
            changes: Vec::new(),
            fingerprint: None,
            violations: Vec::new(),
            producer_report: report,
        };
        if changes.is_empty() {
            closing.candidate = Some(candidate);
            closing.happened = self.failure(ctx, "the producer made no change");
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
        let _ = std::fs::remove_dir_all(&work);

        // The verifier's snapshot: its own two-commit repository, base then candidate, with the base's
        // check-tool configuration in the working tree.
        let verify_dir = job_dir.join("verify");
        if verify_dir.exists() {
            std::fs::remove_dir_all(&verify_dir)?;
        }
        let vsnap = Snapshot::create(
            &self.git,
            &p,
            &p,
            &verify_dir,
            SnapshotRepo::BaseAndCandidate {
                candidate_commit: c.clone(),
            },
        )?;
        let tool_config_from_base = crate::toolconfig::apply_base_tool_config(&vsnap)?;
        let prepared = vsnap.work_tree_tree()?;
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
        for spec in ctx.capability.checks(jc) {
            if self.nested.is_some() {
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
                continue;
            }
            self.refuse_if_stopped()?;
            let run = run_check(
                &spec,
                &CheckContext {
                    dir: vsnap.dir(),
                    env: &check_env,
                    timeout: std::time::Duration::from_secs(jc.config.budget.check_timeout_s),
                    registry: &registry,
                    job: id.as_str(),
                    redactor: ctx.redactor,
                },
            )?;
            cleanup.extend(vsnap.restore_to(&prepared)?);
            let failed = !run.passed;
            check_runs.push(run);
            if failed {
                break;
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
        let all_passed = check_runs.iter().all(|r| r.passed);

        let criteria = ctx.reading.acceptance_criteria.clone();
        let verifier_grant = ctx.capability.verifier_grant(jc);
        let candidate_view = CandidateView {
            base: p.clone(),
            candidate: c.clone(),
            stat: vsnap.git().run(["diff", "--stat", "HEAD~1", "HEAD"])?,
            diff: vsnap.git().run(["diff", "HEAD~1", "HEAD"])?,
        };
        let inputs = VerifierInputs {
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
        };
        let mut verifier_invocation = None;
        let mut verdict_output: Option<VerdictOutput> = None;
        let mut commands_run = Vec::new();
        let mut verifier_failure = None;
        let mut snapshot_changed = None;
        if all_passed {
            let spent = ctx.spent(&closing);
            if let Err(e) = self.check_budget(jc, spent, jc.config.budget.verifier_usd, "verifier")
            {
                verifier_failure = Some(format!("{e}"));
            } else {
                let vbrief = verifier_brief(&inputs, true);
                closing.files.push(NewFile::text(
                    format!("{}/brief-verifier.md", id.dir()),
                    verifier_brief(&inputs, false),
                ));
                let vcomposed = compose(&[
                    CONTRACT.into(),
                    VERIFIER_FRAME.into(),
                    Component::from(ctx.capability.verifier_section()),
                ]);
                let vmodel = ctx.reading.verifier_model.clone();
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
                match self.invoke_and_record(&vfacts, ctx.redactor)? {
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
                // A verifier that changed its snapshot invalidates its own verdict.
                let head_now = vsnap.git().head()?;
                let tree_now = vsnap.work_tree_tree()?;
                if head_now.as_deref() != vsnap.candidate() || tree_now != prepared {
                    snapshot_changed = Some(format!(
                        "snapshot HEAD {:?}, work tree {} (expected {})",
                        head_now,
                        short(&tree_now),
                        short(&prepared)
                    ));
                }
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
        let judgment = judge(&VerdictInputs {
            criteria: &criteria,
            checks: &check_runs,
            output: verdict_output.as_ref(),
            commands_run: &redacted_commands,
            snapshot_changed: snapshot_changed.clone(),
            verifier_failure: verifier_failure.clone(),
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
            inputs: inputs.names(),
            summary: verdict_output.as_ref().map(|v| v.summary.clone()),
        };
        let _ = std::fs::remove_dir_all(&verify_dir);
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
        Ok(ProducerReport {
            summary,
            files: report.files,
            checks_run,
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
        if let Some(candidate) = &closing.candidate {
            closing
                .files
                .push(NewFile::record(&AnyRecord::Candidate(Box::new(
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
        self.state.clean_job(&id);
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
            self.state.clean_job(&id);
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
                            self.state.clean_job(&id);
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
