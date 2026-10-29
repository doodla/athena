//! The producer stages of a dispatched job: the baseline on the base commit, the producer's snapshot and the
//! producer's send-back loop (design §2.3).

use super::*;
use crate::capability::ToolGrant;
use athena_context::Composed;
use std::collections::BTreeMap;

/// What a stage tells the driver: stop, with the job's closing already set, or go on with the stage's result.
pub(super) enum Flow<T> {
    /// The job ends here. The stage has set what happened on the closing, and the driver returns it.
    Stop,
    /// The stage finished and hands its result on.
    Go(T),
}

/// What the snapshot stage hands to the producer loop.
pub(super) struct ProducerSnapshot {
    work: std::path::PathBuf,
    snapshot: Snapshot,
    grant: ToolGrant,
    composed: Composed,
    capability_env: BTreeMap<String, String>,
    model: String,
}

/// What the producer loop hands to the candidate stage.
pub(super) struct Reported {
    /// The producer's redacted report.
    pub(super) report: Option<ProducerReport>,
    /// The sequence number the next invocation gets.
    pub(super) seq: u32,
    /// What the last report still lacked after the send-back.
    pub(super) report_gaps: Vec<String>,
    /// Where that report is kept.
    pub(super) report_kept_in: String,
    /// The criteria the last report left to Athena's checks.
    pub(super) left_criteria: Vec<String>,
    /// The producer's working snapshot.
    pub(super) snapshot: Snapshot,
    /// The snapshot's directory.
    pub(super) work: std::path::PathBuf,
}

impl Athena {
    /// The baseline: the configured checks on the base commit, before the producer starts.
    pub(super) fn run_baseline(&self, ctx: &RunContext<'_>, closing: &mut Closing, seq: u32) {
        let id = ctx.id().clone();
        let job_dir = self.state.job_dir(&id);

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
    }

    /// The producer's detached snapshot of the base, with the grant, the composed context and the
    /// environment the producer runs under.
    pub(super) fn prepare_producer(&self, ctx: &RunContext<'_>) -> Result<ProducerSnapshot> {
        let id = ctx.id().clone();
        let jc = ctx.jc;
        let job_dir = self.state.job_dir(&id);
        let p = ctx.prediction.clone();

        // The producer, in a detached snapshot of the base with a one-commit repository of its own. A
        // tree-seeded retry's work tree holds the rejected candidate's files on top of that base commit, so
        // the change list below is still against the base.
        let work = job_dir.join("work");
        if work.exists() {
            crate::pin::remove_tree(&work)?;
        }
        let repo = match (&ctx.seed_commit, &ctx.resume) {
            (Some(tree_commit), _) => SnapshotRepo::BaseThenTree {
                tree_commit: tree_commit.clone(),
            },
            (None, Some(resume)) => SnapshotRepo::BaseThenDir {
                dir: resume.dir.clone(),
            },
            (None, None) => SnapshotRepo::Base,
        };
        let snapshot = Snapshot::create(&self.git, &p, &p, &work, repo)?;
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
        Ok(ProducerSnapshot {
            work,
            snapshot,
            grant,
            composed,
            capability_env,
            model,
        })
    }

    /// The producer's runs in its snapshot, with the send-back of an incomplete report. Stops the job when
    /// the producer cannot run, times out, does not complete or returns no valid report.
    pub(super) fn run_producer(
        &self,
        ctx: &RunContext<'_>,
        brief: &str,
        accepted: &AcceptedReading,
        closing: &mut Closing,
        mut seq: u32,
        prepared: ProducerSnapshot,
    ) -> Result<Flow<Reported>> {
        let id = ctx.id().clone();
        let jc = ctx.jc;
        let ProducerSnapshot {
            work,
            snapshot,
            grant,
            composed,
            capability_env,
            model,
        } = prepared;
        // A report that leaves acceptance criteria unmet or unreported is sent back to the producer, in
        // the same working directory, up to `PRODUCER_SEND_BACKS` times.
        let mut attempt_brief = brief.to_string();
        let mut send_backs = 0;
        // The files the plan and the reading's deliverables expect the candidate to change; the same send-back
        // also covers any of them left unchanged without a reason in the report.
        let planned = planned_paths(closing.plans.last(), accepted);
        // What the last report still lacked after the send-back, and where that report is kept.
        let mut report_gaps: Vec<String> = Vec::new();
        let mut report_kept_in = String::new();
        // The criteria the last report left to Athena's checks; the verifier judges them from the checks.
        let mut left_criteria: Vec<String>;
        let report = loop {
            let mut spec = self.agent_spec(
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
            // A hard job's reading carries the escalated effort; an ordinary one keeps the configured one.
            if let Some(effort) = &ctx.reading.producer_effort {
                spec.effort = effort.clone();
            }
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
                    return Ok(Flow::Stop);
                }
            };
            ctx.ui.line(&format!(
                "producer   {model} · {:.0}s · {} · {} commands run",
                output.wall_s,
                cost_text(&record),
                record.commands_run.len()
            ));
            closing.new_invocations.push(record.clone());
            if output.outcome == InvocationOutcome::TimedOut {
                let reason = record.detail.clone().unwrap_or_default();
                self.keep_timed_out_work(ctx, &snapshot, &reason, closing)?;
                return Ok(Flow::Stop);
            }
            if output.outcome != InvocationOutcome::Success {
                closing.happened = self.failure(
                    ctx,
                    &format!(
                        "the producer did not complete: {}",
                        record.detail.clone().unwrap_or_default()
                    ),
                );
                return Ok(Flow::Stop);
            }
            let raw = match output
                .structured
                .clone()
                .map(serde_json::from_value::<ProducerOutput>)
            {
                Some(Ok(raw)) => raw,
                _ => {
                    closing.happened = self.failure(ctx, "the producer returned no valid report");
                    return Ok(Flow::Stop);
                }
            };
            let unmet = unmet_criteria(&raw, accepted);
            // Planned files and expected deliverables the work tree does not change, with no reason in the
            // report. A run with no change at all fails below as an idle producer, so it is not listed here.
            let unchanged = {
                let changed: Vec<String> = snapshot
                    .change_list()?
                    .into_iter()
                    .map(|c| c.path)
                    .collect();
                if changed.is_empty() {
                    Vec::new()
                } else {
                    unchanged_planned_files(&raw, &planned, &changed)
                }
            };
            left_criteria = left_to_athena(&raw, accepted);
            let report = self.redact_report(raw, ctx.redactor)?;
            if unmet.is_empty() && unchanged.is_empty() {
                break Some(report);
            }
            let mut problems = Vec::new();
            if !unmet.is_empty() {
                problems.push(format!(
                    "acceptance criteria not met or not reported: {}",
                    unmet
                        .iter()
                        .map(|c| format!("\"{c}\""))
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
            }
            if !unchanged.is_empty() {
                problems.push(format!(
                    "planned files left unchanged without a reason: {}",
                    unchanged.join(", ")
                ));
            }
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
            attempt_brief = send_back_text(brief, &unmet, &unchanged);
        };
        Ok(Flow::Go(Reported {
            report,
            seq,
            report_gaps,
            report_kept_in,
            left_criteria,
            snapshot,
            work,
        }))
    }
}
