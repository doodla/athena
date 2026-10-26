//! `close`, `intervene`, `restart`, `stop`, `start`, and `check` (design §3.1, §3.5, §7.4).

use crate::capability::JobConfig;
use crate::commits::{NewFile, message};
use crate::invocation::{PartialUsage, interrupted_record};
use crate::scoring::{Closing, rescore, score, totals};
use crate::store::{JobRecords, JobState, Records};
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_data::records::{
    AnyRecord, Intervention, InterventionCategory, InterventionSource, Invocation, Outcome,
    OutcomeState, Restart, Role, Score,
};
use athena_data::{JobId, Quantity, RecordId, Unit};
use athena_gate::history::check_history;
use athena_gate::reconcile;
use athena_gate::{Change, Rule, Scope, Violation, gate};
use std::collections::{BTreeMap, BTreeSet};

/// How `close` ended a job.
#[derive(Debug, Clone)]
pub struct Closed {
    pub job: JobId,
    pub state: OutcomeState,
    pub commit: String,
}

/// The arguments of `athena intervene`.
#[derive(Debug, Clone)]
pub struct InterveneArgs {
    pub what: String,
    pub why: String,
    pub class: String,
    pub category: String,
    pub job: Option<String>,
    pub with_staged: bool,
}

/// What `intervene` recorded.
#[derive(Debug, Clone)]
pub struct Intervened {
    pub intervention: Intervention,
    pub commit: String,
    pub rescore: Option<Score>,
}

/// The arguments of `athena restart`.
#[derive(Debug, Clone, Default)]
pub struct RestartArgs {
    pub cost: String,
    pub minutes: Option<f64>,
    pub tokens: Option<f64>,
    pub measured: bool,
    pub lost: Vec<String>,
    pub via: Vec<String>,
}

/// What `check` found.
#[derive(Debug, Default)]
pub struct CheckReport {
    pub commits: usize,
    pub records: usize,
    pub violations: Vec<Violation>,
    /// Things to show that do not fail the check, such as a cutover still waiting for its import.
    pub notes: Vec<String>,
}

fn short(id: &str) -> &str {
    &id[..id.len().min(12)]
}

impl Athena {
    /// `athena close`: closes a job that will not run or finish.
    pub fn close(&self, job_ref: &str, why: &str, ui: &dyn Ui) -> Result<Closed> {
        self.close_with(job_ref, why, false, ui)
    }

    /// [`Athena::close`]. With `allow_legacy_usage`, invocation records already committed that fail
    /// "usage records reconcile" (written before a usage fix) do not block the closing commit; each is
    /// named on `ui`. The records are not modified, and the closing outcome's totals are summed from
    /// them, so the outcome itself reconciles. New invocation records are still reconciled.
    pub fn close_with(
        &self,
        job_ref: &str,
        why: &str,
        allow_legacy_usage: bool,
        ui: &dyn Ui,
    ) -> Result<Closed> {
        if why.trim().is_empty() {
            bail!("say why the job is being closed with --why");
        }
        self.state.registry().reap();
        let head = self.head()?;
        let records = Records::load(&self.git, &head)?;
        let id = records.resolve(job_ref)?;
        let Some(_lock) = self.state.try_lock_job(&id)? else {
            bail!("job {id} is running in another athena process; `athena stop` terminates it");
        };
        let job = records.jobs.get(id.as_str()).expect("resolved").clone();
        let config = JobConfig::at(&self.git, &head)?.config;
        let redactor = self.redactor(&config, Some(&id))?;
        let why = redactor.redact(why.trim())?;
        let (state, after_blocked, files, invocations) = match job.state() {
            JobState::Queued | JobState::Proposed => (
                OutcomeState::Dropped,
                false,
                Vec::new(),
                job.invocations.clone(),
            ),
            JobState::Declined => (
                OutcomeState::Declined,
                false,
                Vec::new(),
                job.invocations.clone(),
            ),
            JobState::Blocked => {
                let outcome = job
                    .outcome
                    .clone()
                    .context("a blocked job has an outcome")?;
                let abandoned = Outcome {
                    state: OutcomeState::Abandoned,
                    reason: format!("closed while blocked: {why}"),
                    at: crate::now(),
                    after_blocked: true,
                    ..outcome
                };
                let files = vec![NewFile::record(&AnyRecord::Outcome(Box::new(abandoned)))];
                let commit = self.commit_records(
                    &files,
                    &message(
                        &format!("Abandoned after blocked: {}", job.title()),
                        &why,
                        &[("Athena-Job", id.as_str().to_string())],
                    ),
                )?;
                ui.line(&format!("abandoned  {id} · {}", short(&commit)));
                crate::state::report_clean(ui, &self.state.clean_job(&id));
                return Ok(Closed {
                    job: id,
                    state: OutcomeState::Abandoned,
                    commit,
                });
            }
            JobState::Dispatched => {
                if self
                    .state
                    .registry()
                    .entries()
                    .iter()
                    .any(|e| e.job == id.as_str())
                {
                    bail!("job {id} is running; `athena stop` terminates it first");
                }
                let (files, invocations) = self.recover_interrupted(&job, &config)?;
                (OutcomeState::Abandoned, false, files, invocations)
            }
            JobState::Closed(state) => bail!("job {id} is already closed ({})", state.as_str()),
        };
        let outcome = Outcome {
            job: id.clone(),
            state,
            reason: format!("closed: {why}"),
            at: crate::now(),
            totals: Some(totals(&invocations, None)),
            branch: self.git.current_branch().ok().flatten(),
            base_commit: job.job.as_ref().and_then(|j| j.base_commit.clone()),
            candidate_commit: None,
            after_blocked,
            through_commit: None,
        };
        let mut trailers = vec![("Athena-Job", id.as_str().to_string())];
        if let Some(n) = job.job.as_ref().and_then(|j| j.accepted_reading) {
            let path = format!("{}/reading-{n}.json", id.dir());
            if let Some(prediction) = crate::run::introducing_commit(self, "HEAD", &path)? {
                trailers.push(("Athena-Prediction", prediction));
            }
        }
        // The score reads the interventions on the commit it is built on (M27).
        let scored_at = crate::now();
        let scope = if allow_legacy_usage {
            for v in reconcile::committed_invocation_violations(&self.git, &head)? {
                ui.line(&format!(
                    "letting a legacy usage record through: {}",
                    v.detail
                ));
            }
            Scope::CommitLegacyUsage
        } else {
            Scope::Commit
        };
        let commit = self.commit_records_in(
            &|head: &str| {
                let records = Records::load(&self.git, head)?;
                let score = score(
                    &job,
                    &records,
                    &invocations,
                    Closing::default(),
                    None,
                    scored_at,
                );
                let mut all = files.clone();
                all.push(NewFile::record(&AnyRecord::Score(Box::new(score))));
                all.push(NewFile::record(&AnyRecord::Outcome(Box::new(
                    outcome.clone(),
                ))));
                Ok(all)
            },
            &message(
                &crate::run::closing_subject(state, &job.title()),
                &why,
                &trailers,
            ),
            scope,
        )
        .map_err(|e| {
            if !allow_legacy_usage && e.to_string().contains("usage records reconcile") {
                e.context(
                    "if the failing records were written before a usage fix, run `athena close` again with --allow-legacy-usage",
                )
            } else {
                e
            }
        })?;
        ui.line(&format!("{:<10} {id} · {}", state.as_str(), short(&commit)));
        crate::state::report_clean(ui, &self.state.clean_job(&id));
        Ok(Closed {
            job: id,
            state,
            commit,
        })
    }

    /// Collects what an interrupted job left in the state directory: completed invocation records, and
    /// lower-bound records for invocations whose process died. If nothing shows the producer ran, and the job
    /// did not die in planning, its invocation is recorded as missing, with both bounds from its cap.
    fn recover_interrupted(
        &self,
        job: &JobRecords,
        config: &athena_data::Config,
    ) -> Result<(Vec<NewFile>, Vec<Invocation>)> {
        let id = job.id();
        let committed: BTreeSet<u32> = job.invocations.iter().map(|i| i.seq).collect();
        let mut invocations = job.invocations.clone();
        let mut files = Vec::new();
        for (path, bytes) in self.state.live_records(id)? {
            if let Ok(AnyRecord::Invocation(inv)) = AnyRecord::parse(&path, &bytes)
                && !committed.contains(&inv.seq)
                && !invocations.iter().any(|i| i.seq == inv.seq)
            {
                files.push(NewFile {
                    path: path.clone(),
                    bytes,
                });
                invocations.push(*inv);
            }
        }
        let usage_dir = self.state.job_dir(id).join("usage");
        if let Ok(entries) = std::fs::read_dir(&usage_dir) {
            for entry in entries.filter_map(Result::ok) {
                let Ok(text) = std::fs::read_to_string(entry.path()) else {
                    continue;
                };
                let Ok(partial) = serde_json::from_str::<PartialUsage>(&text) else {
                    continue;
                };
                if invocations.iter().any(|i| i.seq == partial.seq) {
                    continue;
                }
                let record = interrupted_record(self, id, &partial, config);
                files.push(NewFile::record(&AnyRecord::Invocation(Box::new(
                    record.clone(),
                ))));
                invocations.push(record);
            }
        }
        // A job that died while planning has planner or critic invocations and no producer yet: the producer
        // never started, so there is nothing to bound and no missing producer to record.
        let planned = invocations
            .iter()
            .any(|i| matches!(i.role, Role::Planner | Role::Critic));
        if !planned && !invocations.iter().any(|i| i.role == Role::Producer) {
            let seq = invocations.iter().map(|i| i.seq).max().unwrap_or(0) + 1;
            let model = job
                .latest_reading()
                .map(|r| r.producer_model.clone())
                .unwrap_or_default();
            let partial = PartialUsage {
                seq,
                role: "producer".into(),
                model,
                cap_usd: config.budget.producer_usd,
                timeout_s: config.budget.producer_timeout_s,
                started_at: job.job.as_ref().and_then(|j| j.dispatched_at),
                ..PartialUsage::default()
            };
            let mut record = interrupted_record(self, id, &partial, config);
            record.outcome = athena_data::records::InvocationOutcome::Missing;
            record.detail = Some("recorded by `athena close`: no trace of the producer survived; its cost is bounded by its cap".into());
            files.push(NewFile::record(&AnyRecord::Invocation(Box::new(
                record.clone(),
            ))));
            invocations.push(record);
        }
        invocations.sort_by_key(|i| i.seq);
        Ok((files, invocations))
    }

    /// `athena intervene`: records an intervention, with the user's staged changes when asked.
    pub fn intervene(&self, args: &InterveneArgs, ui: &dyn Ui) -> Result<Intervened> {
        let head = self.head()?;
        let records = Records::load(&self.git, &head)?;
        let config = JobConfig::at(&self.git, &head)?.config;
        if !config.is_class(&args.class) {
            bail!(
                "class {:?} is not in the closed list: {}",
                args.class,
                config.classes.list.join(", ")
            );
        }
        let Some(category) = InterventionCategory::parse(&args.category) else {
            bail!(
                "category {:?} is not one of: {}",
                args.category,
                InterventionCategory::ALL
                    .iter()
                    .map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        };
        let job = match &args.job {
            Some(reference) => Some(records.resolve(reference)?),
            None => None,
        };
        if args.with_staged && self.git.staged_changes()?.is_empty() {
            bail!(
                "--with-staged was given, but nothing is staged; `git add` your hand changes first"
            );
        }
        let redactor = self.redactor(&config, None)?;
        let mut text = vec![args.what.trim().to_string(), args.why.trim().to_string()];
        redactor.redact_all(&mut text)?;
        let mut id = RecordId::generate(crate::now());
        while records.has_record_id(&id) {
            id = RecordId::generate(crate::now());
        }
        let intervention = Intervention {
            id: id.clone(),
            at: crate::now(),
            what: text[0].clone(),
            why: text[1].clone(),
            class_of_work: args.class.clone(),
            category,
            job: job.clone(),
            source: InterventionSource::User,
            with_staged_changes: args.with_staged,
        };
        let mut files = vec![NewFile::record(&AnyRecord::Intervention(
            intervention.clone(),
        ))];
        let mut rescored = None;
        if let Some(job_id) = &job {
            let record = records.jobs.get(job_id.as_str()).expect("resolved");
            if record.outcome.is_some()
                && record.job.is_some()
                && let Some(previous) = record.latest_score()
            {
                let s = rescore(previous, id.clone(), crate::now());
                files.push(NewFile::record(&AnyRecord::Score(Box::new(s.clone()))));
                rescored = Some(s);
            }
        }
        let subject = intervention_subject(&intervention.what);
        let body = format!(
            "Why: {}\nClass: {}. Category: {}.",
            intervention.why,
            intervention.class_of_work,
            category.as_str()
        );
        let mut trailers = vec![("Athena-Intervention", id.as_str().to_string())];
        if let Some(job_id) = &job {
            trailers.push(("Athena-Job", job_id.as_str().to_string()));
        }
        let msg = message(&subject, &body, &trailers);
        let commit = if args.with_staged {
            self.commit_with_staged(&files, &msg)?
        } else {
            self.commit_records(&files, &msg)?
        };
        ui.line(&format!(
            "recorded   intervention {} · {}",
            id,
            short(&commit)
        ));
        // Every intervention is introspected at once; this never blocks or fails the command.
        self.introspect_intervention(&id, ui);
        Ok(Intervened {
            intervention,
            commit,
            rescore: rescored,
        })
    }

    /// Commits the user's staged changes together with `files`, through git itself, after gating it.
    fn commit_with_staged(&self, files: &[NewFile], msg: &str) -> Result<String> {
        let _lock = self.state.lock_repository()?;
        // After C0 (D111) Athena writes no trailers; the message is judged as it ships.
        let msg = crate::commits::message_for_parent(&self.git, &self.head()?, msg);
        let msg = msg.as_str();
        let root = self.git.dir().to_path_buf();
        for file in files {
            let full = root.join(&file.path);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full, &file.bytes)?;
            self.git.run(["add", "--", &file.path])?;
        }
        let change = Change::staged(&self.git, Some(msg.to_string()))?;
        let mut violations = gate(&self.git, &change, Scope::PreCommit)?;
        violations.extend(gate(&self.git, &change, Scope::CommitMsg)?);
        if !violations.is_empty() {
            let list = violations
                .iter()
                .map(|v| format!("  - {v}"))
                .collect::<Vec<_>>()
                .join("\n");
            bail!("the gate refused the commit (the record stays staged):\n{list}");
        }
        let sign = self.git.signs_commits()?;
        let mut args = vec!["commit", "-q", "-F", "-"];
        if sign {
            args.push("-S");
        }
        self.git.run_bytes(&args, Some(msg.as_bytes()), &[])?;
        self.head()
    }

    /// `athena restart`: records a restart of the user's own context.
    pub fn restart(&self, args: &RestartArgs, ui: &dyn Ui) -> Result<(Restart, String)> {
        if args.cost.trim().is_empty() {
            bail!("say what the restart cost with --cost");
        }
        let head = self.head()?;
        let records = Records::load(&self.git, &head)?;
        let config = JobConfig::at(&self.git, &head)?.config;
        let redactor = self.redactor(&config, None)?;
        let mut text = vec![args.cost.trim().to_string()];
        text.extend(args.lost.iter().cloned());
        text.extend(args.via.iter().cloned());
        redactor.redact_all(&mut text)?;
        let quantity = |value: Option<f64>, unit: Unit| -> Result<Option<Quantity>> {
            value
                .map(|v| {
                    if args.measured {
                        if !(v.is_finite() && v >= 0.0) {
                            bail!("quantities must be finite and not negative");
                        }
                        Ok(Quantity::measured(v, unit, "measured by the user"))
                    } else {
                        Quantity::estimated(Some(v), unit, "the user's recollection", None, None)
                            .map_err(anyhow::Error::from)
                    }
                })
                .transpose()
        };
        let mut id = RecordId::generate(crate::now());
        while records.has_record_id(&id) {
            id = RecordId::generate(crate::now());
        }
        let restart = Restart {
            id: id.clone(),
            at: crate::now(),
            cost: text[0].clone(),
            minutes: quantity(args.minutes, Unit::Minutes)?,
            tokens: quantity(args.tokens, Unit::Tokens)?,
            lost: text[1..1 + args.lost.len()].to_vec(),
            via: text[1 + args.lost.len()..].to_vec(),
        };
        let record = AnyRecord::Restart(restart.clone());
        let path = record.canonical_path();
        let commit = self.commit_records(
            &[NewFile::record(&record)],
            &message(
                "Record a restart",
                &format!("Cost: {}", restart.cost),
                &[("Athena-Record", path)],
            ),
        )?;
        ui.line(&format!("recorded   restart {} · {}", id, short(&commit)));
        Ok((restart, commit))
    }

    /// `athena stop`: the kill switch.
    pub fn stop(&self) -> Result<usize> {
        Ok(self.state.registry().stop_all()?.len())
    }

    /// `athena start`: turns the kill switch off. Returns whether it was on.
    pub fn start(&self) -> Result<bool> {
        Ok(self.state.registry().start()?)
    }

    /// `athena check`: every record, reconciliation, and resolution at `HEAD`, plus the gate over staged
    /// changes when there are any.
    pub fn check_working(&self) -> Result<CheckReport> {
        let head = self.head()?;
        let mut report = CheckReport::default();
        let set = athena_gate::records::RecordSet::load(&self.git, &head)?;
        report.records = set.files.len();
        for error in set.errors() {
            report
                .violations
                .push(Violation::new(Rule::ValidRecords, error.to_string(), ""));
        }
        report.violations.extend(reconcile::check_records(&set));
        report.violations.extend(check_resolutions(&set));
        if !self.git.staged_changes()?.is_empty() {
            report.violations.extend(gate(
                &self.git,
                &Change::staged(&self.git, None)?,
                Scope::PreCommit,
            )?);
        }
        Ok(report)
    }

    /// `athena check --history`: what CI runs over the full history.
    pub fn check_history(&self) -> Result<CheckReport> {
        let head = self.head()?;
        let history = check_history(&self.git, &head)?;
        let set = athena_gate::records::RecordSet::load(&self.git, &head)?;
        let mut violations = history.violations;
        violations.extend(check_resolutions(&set));
        Ok(CheckReport {
            commits: history.commits,
            records: history.records,
            violations,
            notes: history.notes,
        })
    }
}

/// The commit subject for a recorded intervention, cut at a word boundary.
fn intervention_subject(what: &str) -> String {
    crate::truncate_at_word(
        &format!(
            "Record an intervention: {}",
            what.lines().next().unwrap_or("")
        ),
        crate::SUBJECT_MAX,
    )
}

/// Resolutions must target an existing pending quantity, one resolution per target (design §5.3).
pub fn check_resolutions(set: &athena_gate::records::RecordSet) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut seen: BTreeMap<(String, String), String> = BTreeMap::new();
    for (path, record) in set.records() {
        let AnyRecord::Resolution(resolution) = record else {
            continue;
        };
        let key = (
            resolution.target_path.clone(),
            resolution.target_field.clone(),
        );
        if let Some(first) = seen.insert(key.clone(), path.to_string()) {
            violations.push(Violation::new(
                Rule::ValidRecords,
                format!(
                    "{path} resolves {}#{}, which {first} already resolves",
                    key.0, key.1
                ),
                "a pending quantity is resolved once",
            ));
        }
        let pending = set
            .bytes(&resolution.target_path)
            .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(bytes).ok())
            .and_then(|value| {
                value
                    .pointer(&format!("/{}", resolution.target_field.replace('.', "/")))
                    .cloned()
            })
            .is_some_and(|field| {
                field.get("provenance").and_then(|p| p.as_str()) == Some("pending")
            });
        if !pending {
            violations.push(Violation::new(
                Rule::ValidRecords,
                format!(
                    "{path} targets {}#{}, which is not a pending quantity",
                    resolution.target_path, resolution.target_field
                ),
                "point the resolution at a record field whose provenance is pending",
            ));
        }
    }
    violations
}

/// Every pending quantity in the records, as (record path, field path, basis).
pub fn pending_quantities(set: &athena_gate::records::RecordSet) -> Vec<(String, String, String)> {
    fn walk(value: &serde_json::Value, path: &str, out: &mut Vec<(String, String)>) {
        match value {
            serde_json::Value::Object(map) => {
                if map.get("provenance").and_then(|p| p.as_str()) == Some("pending") {
                    out.push((
                        path.trim_start_matches('.').to_string(),
                        map.get("basis")
                            .and_then(|b| b.as_str())
                            .unwrap_or("")
                            .to_string(),
                    ));
                    return;
                }
                for (key, child) in map {
                    walk(child, &format!("{path}.{key}"), out);
                }
            }
            serde_json::Value::Array(items) => {
                for (i, child) in items.iter().enumerate() {
                    walk(child, &format!("{path}.{i}"), out);
                }
            }
            _ => {}
        }
    }
    let mut found = Vec::new();
    for file in &set.files {
        if !file.path.ends_with(".json") {
            continue;
        }
        let Ok(value) = serde_json::from_slice::<serde_json::Value>(&file.bytes) else {
            continue;
        };
        let mut fields = Vec::new();
        walk(&value, "", &mut fields);
        for (field, basis) in fields {
            found.push((file.path.clone(), field, basis));
        }
    }
    found
}

#[cfg(test)]
mod subject_tests {
    use super::intervention_subject;

    #[test]
    fn a_long_intervention_subject_is_cut_at_a_word_boundary() {
        let what =
            "I rewrote the generated subject so that it never tells outside readers a half word";
        let subject = intervention_subject(what);
        let full = format!("Record an intervention: {what}");
        assert!(subject.chars().count() <= 72, "{subject:?}");
        assert!(full.starts_with(&subject));
        assert!(
            full[subject.len()..].starts_with(' '),
            "ends mid-word: {subject:?}"
        );
    }
}
