//! Job planning: the planner and critic rounds, the plan record and the planning tool grant (design §3.4).

use super::*;

impl Athena {
    /// Ends the job in planning: it fails before any code was written. Its verification result is unknown,
    /// not false: no verification could have failed (review F11).
    pub(super) fn stop_planning(&self, ctx: &RunContext<'_>, closing: &mut Closing, reason: &str) {
        closing.happened = self.failure(
            ctx,
            &format!("{} {reason}", athena_data::records::PLANNING_STOPPED_PREFIX),
        );
        closing.happened.verification_passed = None;
    }

    /// Job planning: up to `max_rounds` rounds of planner, Athena's own checks and critic, in a read-only
    /// snapshot of the prediction commit. Only critic reviews count toward `max_rounds`: a round is the
    /// critic's look at one plan. When Athena's own plan rules find a blocker or major problem in a plan,
    /// the plan goes straight back to the planner, with no critic call and without using the round, up to
    /// [`MAX_MECHANICAL_PASSES`] times per round; a plan that still breaks a rule after that stops the job.
    /// A critic's revise sends the planner a revision brief that quotes the previous plan and every
    /// finding. On accept, the invocations, plans and critiques go into `closing` and the rendered
    /// producer section is returned. Reject, a reading the critic judged a misread, revise in the last
    /// round, or a failure of either agent stops the job: it is failed in `closing` and `None` is
    /// returned.
    pub(super) fn plan_round(
        &self,
        ctx: &RunContext<'_>,
        accepted: &AcceptedReading,
        checks: &[CheckView],
        closing: &mut Closing,
        seed: Option<PlanRevision>,
    ) -> Result<Option<String>> {
        let cfg = ctx
            .jc
            .config
            .job_plan
            .expect("planning runs only with [job_plan]");
        let resumed_from = seed.as_ref().and_then(|s| s.resumed_from.clone());
        let mut revision: Option<PlanRevision> = seed;
        for round in 1..=cfg.max_rounds {
            let mut mechanical_passes = 0;
            loop {
                let attempt = Attempt {
                    round,
                    mechanical_passes,
                    // A kept plan from an earlier job is revised in the first round only.
                    resumed_from: if round == 1 {
                        resumed_from.clone()
                    } else {
                        None
                    },
                };
                match self.planning_round(
                    ctx,
                    accepted,
                    checks,
                    closing,
                    attempt,
                    revision.take(),
                )? {
                    RoundEnd::Accepted(section) => return Ok(Some(section)),
                    RoundEnd::Stopped => return Ok(None),
                    RoundEnd::Revise(next) => {
                        revision = Some(*next);
                        break;
                    }
                    RoundEnd::Mechanical(next) => {
                        mechanical_passes += 1;
                        revision = Some(*next);
                    }
                }
            }
        }
        unreachable!("the last round either accepts or stops the job")
    }

    /// One planner call in a planning round, then Athena's own checks, then (if they pass) the critic; see
    /// [`Self::plan_round`].
    fn planning_round(
        &self,
        ctx: &RunContext<'_>,
        accepted: &AcceptedReading,
        checks: &[CheckView],
        closing: &mut Closing,
        attempt: Attempt,
        revision: Option<PlanRevision>,
    ) -> Result<RoundEnd> {
        let Attempt {
            round,
            mechanical_passes,
            resumed_from,
        } = attempt;
        let jc = ctx.jc;
        let cfg = jc
            .config
            .job_plan
            .expect("planning runs only with [job_plan]");
        let last_round = round >= cfg.max_rounds;
        let id = ctx.id().clone();
        let p = ctx.prediction.clone();
        let request = ctx
            .job
            .request
            .as_ref()
            .map(|r| r.text.clone())
            .unwrap_or_default();
        let grant = planning_grant();
        let snap_dir = self.state.job_dir(&id).join(format!("plan-{round}"));
        let no_env = std::collections::BTreeMap::new();
        let verifier_commands = ctx.capability.verifier_grant(jc).commands;

        // The planner.
        if let Err(e) = self.preflight(jc, ctx.spent(closing), ReserveStage::Planner { round }) {
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
        Snapshot::create(&self.git, &p, &p, &snap_dir, SnapshotRepo::PlainWithHistory)?;
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
        plan.resumed_from = resumed_from;
        let section = render_plan_section(&plan);
        plan.producer_section_sha256 = sha256_hex(section.as_bytes());
        if let Err(e) = plan.validate() {
            self.stop_planning(ctx, closing, &format!("the plan is invalid: {e}"));
            return Ok(RoundEnd::Stopped);
        }

        // Athena's own checks, at no model cost. A major finding sends the plan back to the planner without a
        // critic call and without using the round, up to MAX_MECHANICAL_PASSES times in a round.
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
            if mechanical_passes < MAX_MECHANICAL_PASSES {
                ctx.ui.line(&format!(
                    "athena     plan round {round}: {} finding(s); no critic call; revising (pass {} of {MAX_MECHANICAL_PASSES})",
                    athena_findings.len(),
                    mechanical_passes + 1
                ));
                return Ok(RoundEnd::Mechanical(Box::new(PlanRevision {
                    previous: plan,
                    findings: athena_findings,
                    resumed_from: None,
                })));
            }
            ctx.ui.line(&format!(
                "athena     plan round {round}: {} finding(s); no critic call; stopping",
                athena_findings.len()
            ));
            let findings = athena_findings;
            let reason = format!(
                "the plan still broke Athena's own plan rules after {MAX_MECHANICAL_PASSES} mechanical revisions (round {round} of {}). Findings: {}",
                cfg.max_rounds,
                findings_line(&findings)
            );
            let critique = Critique {
                job: id.clone(),
                round,
                at: crate::now(),
                invocation_seq: None,
                agent_verdict: None,
                final_verdict: CritiqueVerdict::Revise,
                findings,
                request_reading_mismatch: None,
                summary: Some(
                    "Athena's own checks found a major problem in the plan; the critic was not called."
                        .into(),
                ),
            };
            // The plan and its Athena-only critique stand as the round's records, as evidence of the stop.
            closing.plans.push(plan);
            closing.critiques.push(critique);
            self.stop_planning(ctx, closing, &reason);
            return Ok(RoundEnd::Stopped);
        }

        // The critic.
        if let Err(e) = self.preflight(jc, ctx.spent(closing), ReserveStage::Critic { round }) {
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
        Snapshot::create(&self.git, &p, &p, &snap_dir, SnapshotRepo::PlainWithHistory)?;
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
        let findings = critic_findings(athena_findings, &critic_output);
        let verdict = plan_verdict(agent_verdict, &findings);
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
                stop(athena_data::records::PLANNING_CAP_REASON.into())
            }
            CritiqueVerdict::Revise => Ok(RoundEnd::Revise(Box::new(PlanRevision {
                previous: plan,
                findings: findings.clone(),
                resumed_from: None,
            }))),
        }
    }
}

/// How many times in one planning round the planner may revise a plan that breaks Athena's own plan rules
/// before the critic sees it. These passes call no critic and do not count toward `max_rounds`. A plan that
/// still breaks a rule after this many passes stops the job in planning.
pub(super) const MAX_MECHANICAL_PASSES: u32 = 2;

/// Which planner call of which planning round is being made.
struct Attempt {
    /// The critic round this plan is for, counted from 1; only critic reviews advance it.
    round: u32,
    /// How many mechanical revisions this round has already used.
    mechanical_passes: u32,
    /// The earlier job whose kept plan the round revises (a retry's first round only).
    resumed_from: Option<JobId>,
}

/// How one planning round ended.
enum RoundEnd {
    /// The plan was accepted; the producer's section, rendered from it.
    Accepted(String),
    /// The critic's review asks for a revision: the planner is asked again in the next round, with the
    /// previous plan and every finding.
    Revise(Box<PlanRevision>),
    /// Athena's own plan rules found a blocker or major problem: the planner is asked again in the same
    /// round, with no critic call. Nothing is filed for this plan.
    Mechanical(Box<PlanRevision>),
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
        resumed_from: None,
    })
}

/// What the planner and the critic may do: read, search and list files in their snapshot. No Bash, no git
/// command, no writes. The history they see is a generated file, not a repository.
fn planning_grant() -> crate::capability::ToolGrant {
    crate::capability::ToolGrant {
        tools: vec!["Read".into(), "Grep".into(), "Glob".into()],
        commands: std::collections::BTreeMap::new(),
        writes: false,
        permission_mode: "dontAsk".into(),
    }
}

#[cfg(test)]
mod planning_grant_tests {
    use super::*;

    #[test]
    fn the_planner_and_critic_grant_has_no_bash_no_git_and_no_writes() {
        let grant = planning_grant();
        assert_eq!(grant.tools, ["Read", "Grep", "Glob"]);
        assert!(!grant.tools.iter().any(|t| t == "Bash"));
        assert!(grant.commands.is_empty(), "no command, so no git command");
        assert!(!grant.writes);
        assert_eq!(grant.permission_mode, "dontAsk");
    }

    #[test]
    fn the_briefs_and_the_snapshot_agree_on_the_history_file() {
        assert_eq!(
            athena_context::briefs::HISTORY_FILE,
            athena_git::snapshot::HISTORY_FILE
        );
        assert_eq!(
            athena_context::briefs::HISTORY_LIMIT,
            athena_git::snapshot::HISTORY_LIMIT
        );
    }
}
