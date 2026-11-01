//! Close and land: the closing records of a dispatched job, the landing algorithm, the retry of a blocked
//! landing and the text of the closing and landing commits (design §2.3, §7).

use super::*;

impl Athena {
    /// Commits the closing records of a dispatched job, landing the change when it was verified.
    pub(super) fn close_dispatched(
        &self,
        ctx: &RunContext<'_>,
        mut closing: Closing,
    ) -> Result<GoResult> {
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
        push_closing_records(&mut closing);
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
        if let Some(resumed) = &ctx.resume {
            self.state.remove_kept_work(&resumed.job);
        }
        self.settle_resume_marks(&id, closing.consumed_resume.as_ref(), state, &reason);
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

    /// Holds a job because free disk space stayed low at a late gate (before the producer, before the
    /// checks, or before the verifier). The attempt's records are committed, with no score and no outcome, so
    /// the job stays dispatched with no process: it reads as interrupted, which no failure, introspection or
    /// scope-guard count looks at. A hold marker records the amounts. With a built candidate (the last two
    /// callers) the job's own resume mark stays, and the next drain resumes the job from its kept candidate
    /// ([`Athena::retry`]). A hold before the producer has no candidate and no resume mark, so the next drain
    /// reads the job again, and that retry spends the marker (`release_superseded_hold`). The error returned
    /// is the [`LowDisk`] hold; the caller returns it from `go`.
    pub(super) fn hold_for_disk(
        &self,
        ctx: &RunContext<'_>,
        mut closing: Closing,
        low: crate::disk::LowDisk,
    ) -> anyhow::Error {
        let id = ctx.id().clone();
        push_closing_records(&mut closing);
        let trailers = [("Athena-Job", id.as_str().to_string())];
        let subject = format!("Held: {}", ctx.reading.title);
        let committed =
            self.commit_records(&closing.files, &message(&subject, &low.text(), &trailers));
        if let Err(why) = committed {
            return why.context(format!(
                "job {id} was held for low disk space, but its records could not be committed"
            ));
        }
        crate::disk::write_hold(&self.state, &id, &low);
        ctx.ui.line(&format!("held       {}", low.text()));
        if let Some(old) = closing.consumed_resume.as_ref() {
            self.state.remove_resume(old);
            crate::disk::clear_hold(&self.state, old);
        }
        low.into()
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
        self.settle_resume_marks(&id, closing.consumed_resume.as_ref(), state, &reason);
        if state != OutcomeState::Blocked {
            crate::state::report_clean(ctx.ui, &self.state.clean_job(&id));
            if let Some(resumed) = &ctx.resume {
                self.state.remove_kept_work(&resumed.job);
            }
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
    pub(super) fn retry_landing(
        &self,
        _records: &Records,
        job: &JobRecords,
        ui: &dyn Ui,
    ) -> Result<Finished> {
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
        // After C0 (D111) no commit carries trailers: nothing is searched for, and no
        // Athena-Verified-Content is written. The prediction is the commit that recorded the reading.
        let stopped = crate::gh::cutover_job_for(&self.git, "HEAD").is_some();
        let mut trailers: Vec<(&str, String)> = Vec::new();
        let mut post_c0_prediction = None;
        if stopped {
            let reading_path = AnyRecord::Reading(Box::new(reading.clone())).canonical_path();
            post_c0_prediction = introducing_commit(self, "HEAD", &reading_path)?;
        } else {
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
        }
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
                                    .map(|(_, v)| v.clone())
                                    .or(post_c0_prediction),
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

/// Adds the record files every attempt commits, whether it closes or is held: its invocations, plans,
/// critiques, baseline, candidate and verification.
fn push_closing_records(closing: &mut Closing) {
    let mut records: Vec<AnyRecord> = Vec::new();
    records.extend(
        closing
            .new_invocations
            .iter()
            .map(|inv| AnyRecord::Invocation(Box::new(inv.clone()))),
    );
    records.extend(
        closing
            .plans
            .iter()
            .map(|plan| AnyRecord::JobPlan(Box::new(plan.clone()))),
    );
    records.extend(
        closing
            .critiques
            .iter()
            .map(|critique| AnyRecord::Critique(Box::new(critique.clone()))),
    );
    if let Some(baseline) = &closing.baseline {
        records.push(AnyRecord::Baseline(Box::new(baseline.clone())));
    }
    if let Some(candidate) = &closing.candidate {
        records.push(AnyRecord::Candidate2(Box::new(candidate.clone())));
    }
    if let Some(verification) = &closing.verification {
        records.push(AnyRecord::Verification(Box::new(verification.clone())));
    }
    closing.files.extend(records.iter().map(NewFile::record));
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
