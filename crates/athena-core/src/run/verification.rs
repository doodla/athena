//! The verification stages of a job: the verifier's inputs, the verifier run and its one rerun, and the
//! redaction and assembly of the verification (structure-pass §3.3, stages S8 to S10).

use super::*;

/// What the checks stage hands to the verification stage: the verifier's snapshot, the results of the
/// checks and the replay record.
pub(super) struct Checked {
    pub(super) vsnap: Snapshot,
    pub(super) prepared: String,
    pub(super) tool_config_from_base: Vec<String>,
    pub(super) check_runs: Vec<CheckRun>,
    pub(super) cleanup: Vec<String>,
    pub(super) replay_record: Replay,
    pub(super) verify_dir: PathBuf,
    pub(super) capability_env: std::collections::BTreeMap<String, String>,
    /// [`crate::verify::reuse_statement`] for the candidate, when the check results came from its CI run.
    pub(super) reuse: Option<String>,
}

impl Athena {
    /// The last part of a job: the verifier's inputs, the verifier run (once more if it changed its
    /// snapshot), the redaction of its free text and the verification with its verdict.
    pub(super) fn verification_stage(
        &self,
        ctx: &RunContext<'_>,
        accepted: &AcceptedReading,
        mut closing: Closing,
        built: &Built,
        checked: Checked,
    ) -> Result<Closing> {
        let id = ctx.id().clone();
        let jc = ctx.jc;
        let p = built.base.clone();
        let c = built.commit.clone();
        let f = built.fingerprint.clone();
        let mut seq = built.seq;
        let report_gaps = built.report_gaps.clone();
        let left_criteria = built.left_criteria.clone();
        let Checked {
            mut vsnap,
            prepared,
            tool_config_from_base,
            check_runs,
            cleanup,
            replay_record,
            verify_dir,
            capability_env,
            reuse,
        } = checked;
        let usable_baseline = closing
            .baseline
            .as_ref()
            .filter(|b| b.base_commit == p.as_str());

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
        // Check results taken from the CI run for this candidate are said to be so, with the run, the SHA and
        // each check's conclusion. It is not a check and fails nothing: the recorded check results above
        // decide. A job resumed after its checks passed, or a nested run, carries none.
        if let Some(statement) = reuse {
            inputs.checks.push(CheckResultView {
                name: "ci-result".into(),
                command: "not a check".into(),
                passed: true,
                detail: statement,
            });
        }
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
        // is written and removed in the verifier's snapshot. A problem holds the job: it keeps its candidate
        // and check results, has no verdict and no outcome, and the next drain resumes it at this stage. The
        // verifier is not started and its brief is not recorded.
        if let Some(low) = crate::disk::verifier_environment_problem(
            &self.state,
            jc.config.min_free_gb(),
            &id,
            vsnap.dir(),
            ctx.ui,
        ) {
            let _ = crate::pin::remove_tree(&verify_dir);
            return Err(self.hold_for_disk(ctx, closing, low));
        }
        let mut verifier_invocation = None;
        let mut verdict_output: Option<VerdictOutput> = None;
        let mut commands_run = Vec::new();
        let mut verifier_failure = None;
        // Set when the verifier changed its snapshot: the system failed to keep it still, so the work is
        // not at fault. The verdict is discarded and verification reruns once on a fresh snapshot.
        let mut system_fault: Option<String> = None;
        // The verifier always runs, so a failed check never leaves the criteria unjudged; the check still
        // fails the final verdict unless it is pre-existing: it failed on the base too and, where both runs
        // name their failing tests, every failing test now also failed there (`verify::is_pre_existing`).
        {
            ctx.stages.enter(&id, Stage::Verifier);
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

    /// Creates the verifier's snapshot at `dir` (removing what is there): base then candidate as two commits,
    /// the pinned tree mounted if there is one, and the base's check-tool configuration in the work tree.
    /// Returns it with the tool-config paths and the work tree's tree as prepared.
    pub(super) fn verifier_snapshot(
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
}
