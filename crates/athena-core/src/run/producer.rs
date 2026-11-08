//! The producer stages of a job: the baseline checks on the base commit, the producer's snapshot, and the
//! producer loop with its send-back (design §2.3, structure-pass §3.3, stages S1 to S3).

use super::*;
use crate::capability::ToolGrant;
use crate::mirrors::{self, Drift};
use athena_agent::env::CargoShim;
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
    /// The replay the loop measured on the last report, when it ran. The hold at verification reuses it for
    /// the same candidate tree and accepted readings.
    pub(super) loop_replay: Option<LoopReplay>,
    /// The sequence number the next invocation gets.
    pub(super) seq: u32,
    pub(super) snapshot: Snapshot,
    pub(super) work: PathBuf,
}

/// The send-back problem a held replay adds: the count, the measured percent, the limit and the job ids
/// (at most [`REPLAY_BRIEF_MAX_LISTED`] of them, then a count of the rest).
fn replay_problem(replay: &LoopReplay) -> String {
    let newly = &replay.run.newly_refused;
    let mut ids: Vec<&str> = newly
        .iter()
        .take(REPLAY_BRIEF_MAX_LISTED)
        .map(|r| r.job.as_str())
        .collect();
    let more = newly.len().saturating_sub(ids.len());
    let rest = format!("and {more} more");
    if more > 0 {
        ids.push(&rest);
    }
    format!(
        "replay of accepted readings: {} of {} ({}%) newly refused, over the {}% limit: {}",
        newly.len(),
        replay.run.total,
        newly.len() * 100 / replay.run.total.max(1),
        replay.limit,
        ids.join(", ")
    )
}

/// The hand-kept files that disagree with their source in the producer's work tree (`crate::mirrors`), for the
/// entries the changed paths can affect. The base text of a file is read from the snapshot's base commit; with no
/// base commit the rules that compare with it (append-only, prompt structure, decision entries) are skipped.
/// Prompt files and new decision entries are among the files checked. Empty unless the work tree is Athena's own source: the registry
/// holds Athena's conventions, which another repository need not follow.
fn mirror_drift_in(snapshot: &Snapshot, changed: &[String]) -> Vec<Drift> {
    if !mirrors::is_athena_source(snapshot.dir()) {
        return Vec::new();
    }
    let git = snapshot.git();
    let base = snapshot.base().map(str::to_string);
    let read = move |path: &str| {
        base.as_deref()
            .and_then(|b| git.show_file(b, path).ok().flatten())
    };
    mirrors::check_all(snapshot.dir(), &read, Some(changed))
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
        // The kept candidate of a timed out job is `seed_commit` too; its kept directory is the fallback.
        let repo = match (
            &ctx.seed_commit,
            ctx.resume.as_ref().and_then(|r| r.dir.as_ref()),
        ) {
            (Some(tree_commit), _) => SnapshotRepo::BaseThenTree {
                tree_commit: tree_commit.clone(),
            },
            (None, Some(dir)) => SnapshotRepo::BaseThenDir { dir: dir.clone() },
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

    /// Formats the producer's work tree after a producer pass (see the `format` module). Returns the reason when
    /// the job must stop (the kill switch is set or a run hit its deadline), and `None` otherwise. It prints a
    /// `format` line when the formatter could not fix the tree, when it rewrote the tree, and when it added
    /// files to the change list. A pinned study mounts a checkout inside the work tree, so it is not formatted.
    fn format_stage(
        &self,
        ctx: &RunContext<'_>,
        job_dir: &std::path::Path,
        snapshot: &Snapshot,
        capability_env: &BTreeMap<String, String>,
    ) -> Result<Option<String>> {
        if ctx.pin_dir.is_some() || !snapshot.dir().join("Cargo.toml").is_file() {
            return Ok(None);
        }
        let jc = ctx.jc;
        let env = athena_agent::env::build(
            &EnvSpec {
                agents: &jc.config.agents,
                job: ctx.id().as_str(),
                job_dir,
                snapshot: snapshot.dir(),
                side: Side::Produce,
                capability_env,
            },
            |key| std::env::var(key).ok(),
        )
        .map_err(anyhow::Error::msg)?;
        let registry = self.state.registry();
        let check_ctx = CheckContext {
            dir: snapshot.dir(),
            env: &env,
            timeout: std::time::Duration::from_secs(jc.config.budget.check_timeout_s),
            registry: &registry,
            job: ctx.id().as_str(),
            redactor: ctx.redactor,
        };
        let before = snapshot.work_tree_tree()?;
        let pre = snapshot.change_list()?;
        match format::format_work_tree(snapshot.dir(), &pre, &check_ctx)? {
            format::Formatted::Stopped(note) => return Ok(Some(note)),
            format::Formatted::Unfixed { output, note } => ctx.ui.line(&format!(
                "format     cargo fmt --check still fails: {}; the fmt check will report it",
                format::unfixed_summary(&output, note.as_deref())
            )),
            format::Formatted::Skipped(_) | format::Formatted::Done => {}
        }
        if snapshot.work_tree_tree()? != before {
            ctx.ui
                .line("format     cargo fmt rewrote the work tree before the checks");
            let added = format::paths_added_by_format(&pre, &snapshot.change_list()?);
            if !added.is_empty() {
                ctx.ui.line(&format!(
                    "format     cargo fmt also changed {} beyond the producer's files",
                    added.join(", ")
                ));
            }
        }
        Ok(None)
    }

    /// Stage S3: the producer loop. `Flow::Stop` means the job ended here and `closing` says how.
    ///
    /// Root cause of a repeated failure it answers: nothing made the producer measure a new or tightened
    /// refusal rule against past accepted readings while it worked. The replay ran once, after the producer
    /// finished, as a hold at verification, so the producer never saw the newly refused list or the share. Now
    /// each parsed report is followed by [`Athena::loop_replay`], and a measured share over the limit joins the
    /// unmet criteria and unchanged files in the one send-back, with the list and the share in the brief. The
    /// hold at verification stays as the backstop.
    ///
    /// The decision gate ([`Athena::unreconciled_in_snapshot`]) runs on the same work tree after each report.
    /// A change that touches code a decision governs without reconciling it earns the same one send-back,
    /// with the gate's message in the brief; the send-back is shared with the unmet criteria, so a producer
    /// already sent back for those gets none for the gate. A change still unreconciled at the limit leaves the
    /// loop, and the gate at the candidate (`check_and_verify`) closes the job with the gate's message.
    ///
    /// Files changed outside the job's allowed paths ([`outside_allowed_paths`]) are sent back the same way,
    /// with the files and the limiting criterion in the brief. They have a send-back of their own after the
    /// shared one ([`SCOPE_SEND_BACKS`]), so a change made outside the list after the first send-back is still
    /// sent back. A change still outside afterwards leaves the loop, and the check at the candidate closes the
    /// job before the verifier runs. A job with no allowed paths, and a nested run, are not checked.
    ///
    /// Hand-kept files that disagree with their source ([`crate::mirrors`]: shard timings, the decision log, the
    /// CI cache key, the ontology) are sent back the same way, for the registry entries the changed paths can
    /// affect, in Athena's own source tree only. The same registry compares the finished diff of a prompt file
    /// and the entries a change appends to the decision log with the structure they must keep (a bullet joined
    /// onto another line, a decision inserted inside another). The root cause: such a file was tested for its
    /// shape, so content that contradicted the source was found only at the final verdict. The send-back is shared with
    /// the unmet criteria. Drift left at the limit does not stop the job here: the repo test `mirrors`, run by
    /// the configured test check, fails on it at verification.
    ///
    /// The work tree is formatted by `Athena::format_stage` (see the `format` module). The format step belongs
    /// at the end of producer_loop, before the change list is taken: it is the last producer-stage act of every
    /// pass, right after the producer returned and its report was parsed. "After the producer" means after each
    /// producer invocation returns. The loop's exit, the replay it measures, the candidate, the checks and the
    /// verifier all come after it, so they all see the formatted tree. A job whose only defect is rustfmt
    /// layout therefore reaches the verifier formatted; an error the formatter cannot fix stays in the tree for
    /// the configured `fmt` check to report.
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
        // Send-backs for files outside the allowed paths, counted apart from the shared one above.
        let mut scope_send_backs = 0;
        // The job's allowed paths and the criterion that limits them, when an accepted criterion limits which
        // files may change. A nested run is judged by the outer job, whose check sees the combined change.
        let scope = match (
            ctx.job
                .job
                .as_ref()
                .and_then(|j| j.allowed_paths.as_deref()),
            accepted.limiting_criterion(),
        ) {
            (Some(allowed), Some(criterion)) if self.nested.is_none() => Some((allowed, criterion)),
            _ => None,
        };
        // The files the plan and the reading's deliverables expect the candidate to change; the same send-back
        // also covers any of them left unchanged without a reason in the report.
        let planned = planned_paths(closing.plans.last(), accepted);
        // What the last report still lacked after the send-back, and where that report is kept.
        let mut report_gaps: Vec<String> = Vec::new();
        let mut report_kept_in = String::new();
        // The criteria the last report left to Athena's checks; the verifier judges them from the checks.
        let mut left_criteria: Vec<String>;
        // The replay of accepted readings measured on the last report, when it ran.
        let mut loop_replay: Option<LoopReplay>;
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
            // The producer's `cargo build`, `test`, `clippy` and `check` take a build slot: a `cargo` wrapper
            // first on its PATH. Only this spec gets it, so the verifier and the checks run as before. A note
            // left by an earlier invocation is cleared, so only the wrapper of this one can hold the job.
            let job_dir = self.state.job_dir(&id);
            crate::slots::clear_wait_note(&crate::slots::wait_note_path(&job_dir));
            let shim = CargoShim::new(
                &job_dir,
                self.state.slots_dir(),
                &jc.config.pipeline(),
                &jc.config.agents,
            );
            shim.install(&self.athena_bin)
                .map_err(|e| anyhow::anyhow!("installing the producer's cargo wrapper: {e}"))?;
            shim.apply(&mut spec.env);
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
                self.keep_timed_out_work(ctx, &snapshot, accepted, &reason, closing)?;
                return Ok(Flow::Stop);
            }
            if output.outcome != InvocationOutcome::Success {
                closing.happened = self.failure(
                    ctx,
                    // An expired login or a usage limit is closed as an environment stop, not as work.
                    &crate::disk::producer_stop_reason(&record.detail.clone().unwrap_or_default()),
                );
                return Ok(Flow::Stop);
            }
            // The producer ended while its last cargo command had waited out the build slot: its work could not
            // be built, so the job is held (the way a wait at the checks holds it), not judged on unbuilt work.
            // A producer that tried again and got a slot cleared the note, so it is not held.
            let note = crate::slots::wait_note_path(&job_dir);
            if let Some(wait) = crate::slots::read_wait_note(&note) {
                crate::slots::clear_wait_note(&note);
                return Err(wait.into());
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
            // The last producer-stage act of the pass: Athena formats the work tree before the change list is
            // taken, so everything below and everything after the loop sees the formatted tree.
            if let Some(note) = self.format_stage(ctx, &job_dir, &snapshot, &capability_env)? {
                closing.happened = self.failure(ctx, &format!("the format step stopped: {note}"));
                return Ok(Flow::Stop);
            }
            let unmet = unmet_criteria(&raw, accepted);
            // Planned files and expected deliverables the work tree does not change, with no reason in the
            // report. A run with no change at all fails below as an idle producer, so it is not listed here.
            let changes = snapshot.change_list()?;
            let changed: Vec<String> = changes.iter().map(|c| c.path.clone()).collect();
            let unchanged = if changed.is_empty() {
                Vec::new()
            } else {
                unchanged_planned_files(&raw, &planned, &changed)
            };
            // Changed files, deleted ones included, outside the job's allowed paths. Empty for a job with no
            // limit, so every branch below that reads it behaves as before.
            let outside = scope
                .map(|(allowed, _)| outside_allowed_paths(allowed, &changed))
                .unwrap_or_default();
            // The decision gate's finding on the work tree as it stands: the same message the gate at the
            // candidate would close the job with. A nested run is judged by the outer job.
            let gate = if self.nested.is_none() && !changes.is_empty() {
                self.unreconciled_in_snapshot(&ctx.prediction, &snapshot, &changes)
            } else {
                None
            };
            // Hand-kept files the producer's change can leave disagreeing with their source (shard timings, the
            // decision log, the CI cache key, the ontology), and prompt files and new decision entries whose
            // structure the change broke. Empty in a nested run and outside Athena's source.
            let mirror_drift = if self.nested.is_none() && !changes.is_empty() {
                mirror_drift_in(&snapshot, &changed)
            } else {
                Vec::new()
            };
            left_criteria = left_to_athena(&raw, accepted);
            let report = self.redact_report(raw, ctx.redactor)?;
            // The replay of accepted readings through the base and the work as it stands. A measured share
            // over the limit is a problem like an unmet criterion; a replay that did not run is not.
            loop_replay = self.loop_replay(ctx, &snapshot)?;
            let held = loop_replay.as_ref().filter(|l| l.run.held);
            if unmet.is_empty()
                && unchanged.is_empty()
                && held.is_none()
                && gate.is_none()
                && outside.is_empty()
                && mirror_drift.is_empty()
            {
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
            if let Some(replay) = held {
                problems.push(replay_problem(replay));
            }
            // What the verifier is told the report still lacked at the limit; the gate's finding is not part
            // of it, because the job ends at the gate and no verifier runs.
            let report_problems = problems.clone();
            if gate.is_some() {
                problems.push("decisions not reconciled".to_string());
            }
            // Like the gate's finding, hand-kept files that disagree with their source are not part of what the
            // verifier is told the report lacked: the configured test check fails on them at verification.
            if !mirror_drift.is_empty() {
                problems.push(format!(
                    "files kept by hand disagree with their source: {}",
                    mirror_drift
                        .iter()
                        .map(|d| d.message.as_str())
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
            }
            // Like the gate's finding, files outside the allowed paths are not part of what the verifier is
            // told the report lacked: the job ends before the verifier when they remain.
            if !outside.is_empty() {
                problems.push(format!(
                    "files changed outside the job's allowed paths: {}",
                    outside.join(", ")
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
            // The shared send-back is used first; a change outside the allowed paths gets one more after it.
            let scope_slot_free = !outside.is_empty() && scope_send_backs < SCOPE_SEND_BACKS;
            if send_backs >= PRODUCER_SEND_BACKS && !scope_slot_free {
                // The work may be finished even though the report is not: the verifier judges it. A run
                // that changed nothing still fails, below, after the loop.
                if !report_problems.is_empty() {
                    ctx.ui.line(&format!(
                        "producer   report still incomplete after the send-back: {}; the verifier will judge it",
                        report_problems.join("; ")
                    ));
                    report_gaps = report_problems;
                    report_kept_in = kept;
                }
                if gate.is_some() {
                    // The gate at the candidate closes the job with the gate's message.
                    ctx.ui.line(
                        "producer   decisions still not reconciled after the send-back; the job ends at the decision gate",
                    );
                }
                if !mirror_drift.is_empty() {
                    ctx.ui.line(&format!(
                        "producer   hand-kept files still disagree with their source after the send-back: {}",
                        mirror_drift
                            .iter()
                            .map(|d| d.mirror.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if !outside.is_empty() {
                    // The job ends before the verifier, at the check on the candidate.
                    ctx.ui.line(
                        "producer   files still outside the allowed paths after the send-back; the job ends before the verifier",
                    );
                }
                break Some(report);
            }
            if send_backs < PRODUCER_SEND_BACKS {
                send_backs += 1;
            } else {
                scope_send_backs += 1;
            }
            ctx.ui
                .line(&format!("producer   sent back: {}", problems.join("; ")));
            let replay = held.map(|replay| (&replay.view, replay.limit));
            let next_brief = match (&gate, replay, scope.filter(|_| !outside.is_empty())) {
                (gate, replay, Some((_, criterion))) => send_back_text_scoped(
                    brief,
                    &unmet,
                    &unchanged,
                    replay,
                    gate.as_deref(),
                    Some((&outside, criterion)),
                ),
                (Some(message), replay, None) => {
                    send_back_text_full(brief, &unmet, &unchanged, replay, Some(message))
                }
                (None, Some(_), None) => {
                    send_back_text_with_replay(brief, &unmet, &unchanged, replay)
                }
                (None, None, None) => send_back_text(brief, &unmet, &unchanged),
            };
            attempt_brief = append_mirror_drift(next_brief, &mirror_drift);
        };
        Ok(Flow::Go(Reported {
            report,
            report_gaps,
            report_kept_in,
            left_criteria,
            loop_replay,
            seq,
            snapshot,
            work,
        }))
    }
}
