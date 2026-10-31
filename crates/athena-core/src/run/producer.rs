//! The producer stages of a job: the baseline checks on the base commit, the producer's snapshot, and the
//! producer loop with its send-back (design §2.3, structure-pass §3.3, stages S1 to S3).

use super::*;
use crate::capability::ToolGrant;
use athena_context::Composed;
use std::collections::BTreeMap;

/// What a stage tells the driver: carry on with the stage's value, or end the job with the `closing` the
/// stage has already filled in.
pub(super) enum Flow<T> {
    Go(T),
    Stop,
}

/// The producer's snapshot and what its invocations are built from (stage S2).
pub(super) struct ProducerWork {
    /// The producer's working directory.
    pub(super) work: PathBuf,
    pub(super) snapshot: Snapshot,
    pub(super) grant: ToolGrant,
    pub(super) composed: Composed,
    pub(super) capability_env: BTreeMap<String, String>,
    pub(super) model: String,
}

/// A producer report that ends the loop (stage S3), with the values the later stages read.
pub(super) struct Reported {
    pub(super) report: Option<ProducerReport>,
    /// What the last report still lacked after the send-back.
    pub(super) report_gaps: Vec<String>,
    /// Where that report is kept.
    pub(super) report_kept_in: String,
    /// The criteria the last report left to Athena's checks.
    pub(super) left_criteria: Vec<String>,
    /// The sequence number the next invocation gets.
    pub(super) seq: u32,
    pub(super) snapshot: Snapshot,
    pub(super) work: PathBuf,
}

impl Athena {
    /// Stage S1: the configured checks on the base commit, before the producer starts.
    pub(super) fn baseline_stage(
        &self,
        ctx: &RunContext<'_>,
        job_dir: &std::path::Path,
        seq: u32,
        closing: &mut Closing,
    ) {
        // The baseline: the configured checks on the base commit, before the producer starts. It is
        // recorded only and never stops the job.
        let baseline = self.take_baseline(ctx, job_dir, seq);
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

    /// Stage S2: the producer's detached snapshot of the base, with the grant, brief frame and environment
    /// its invocations use. The caller holds the [`crate::state::JobWorktree`] that cleared a stale work tree
    /// before this runs and removes the work tree and build directory when the job's run ends.
    pub(super) fn producer_snapshot(&self, ctx: &RunContext<'_>) -> Result<ProducerWork> {
        let jc = ctx.jc;
        let p = ctx.prediction.clone();
        // The producer, in a detached snapshot of the base with a one-commit repository of its own. A
        // tree-seeded retry's work tree holds the rejected candidate's files on top of that base commit, so
        // the change list below is still against the base.
        let work = self.state.work_dir(ctx.id());
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
        Ok(ProducerWork {
            work,
            snapshot,
            grant,
            composed,
            capability_env,
            model,
        })
    }

    /// Stage S3: the producer loop. `Flow::Stop` means the job ended here and `closing` says how.
    pub(super) fn producer_loop(
        &self,
        ctx: &RunContext<'_>,
        brief: &str,
        accepted: &AcceptedReading,
        closing: &mut Closing,
        mut seq: u32,
        prepared: ProducerWork,
    ) -> Result<Flow<Reported>> {
        let id = ctx.id().clone();
        let jc = ctx.jc;
        let ProducerWork {
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
            // Every pass enters the producer stage, so a send-back reports it again.
            ctx.stages.enter(&id, Stage::Producer);
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
            report_gaps,
            report_kept_in,
            left_criteria,
            seq,
            snapshot,
            work,
        }))
    }
}
