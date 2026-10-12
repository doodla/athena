//! `ask` and `correct`: the reader's reading, with forecasts, committed before any work (design §3.2–3.3).

use crate::capability::{JobConfig, ToolGrant};
use crate::checks::tail;
use crate::commits::{NewFile, message};
use crate::invocation::InvocationFacts;
use crate::store::{JobRecords, JobState, Records};
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_agent::AgentInvocation;
use athena_agent::env::{EnvSpec, Side};
use athena_agent::policy::Policy;
use athena_context::briefs::{EarlierAttempt, ReaderInputs, reader_brief, render_catalog};
use athena_context::outputs::{
    CriterionKindOutput, DeliverableActionOutput, ReaderChoices, ReadingOutput, reading_schema,
};
use athena_context::{CONTRACT, Component, Composed, READER_FRAME, compose};
use athena_data::records::{
    AnyRecord, Correction, CorrectionKind, Criterion, CriterionKind, DefaultApplied, Deliverable,
    DeliverableAction, Forecast, Forecasts, Intervention, InterventionCategory, InterventionSource,
    Invocation, OutcomeState, Quantiles, Reading, Request, Role,
};
use athena_data::{JobId, RecordId};
use athena_gate::redact::Redactor;
use athena_git::snapshot::{Snapshot, SnapshotRepo};
use std::collections::BTreeMap;
use std::time::Duration;

/// What `ask` and `correct` leave behind: a proposed (or declined) job, committed.
#[derive(Debug, Clone)]
pub struct Proposal {
    pub job: JobId,
    pub reading: Option<Reading>,
    /// Why there is no reading, when the reader failed.
    pub reader_error: Option<String>,
    pub invocation: Option<Invocation>,
    /// The proposal commit.
    pub commit: String,
    /// The job's total spend so far.
    pub spent_usd: f64,
}

/// The reader's system prompt: contract, frame, defaults model, capability catalog.
pub fn reader_system_prompt(athena: &Athena, jc: &JobConfig) -> Composed {
    let cards: Vec<_> = athena
        .capabilities
        .iter()
        .filter(|c| jc.config.capability(c.id()).is_some())
        .map(|c| c.card())
        .collect();
    compose(&[
        CONTRACT.into(),
        READER_FRAME.into(),
        Component::new("defaults", jc.defaults.version, jc.defaults.render()),
        Component::new("catalog", 1, render_catalog(&cards)),
    ])
}

/// The snapshot's top-level listing, for the reader's brief.
fn listing(dir: &std::path::Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|e| {
                    let name = e.file_name().to_string_lossy().into_owned();
                    if e.file_type().is_ok_and(|t| t.is_dir()) {
                        format!("{name}/")
                    } else {
                        name
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

fn normalize_path(path: &str) -> Option<String> {
    let trimmed = path.trim().trim_start_matches("./");
    if trimmed.is_empty() || trimmed.starts_with('/') || trimmed.split('/').any(|p| p == "..") {
        return None;
    }
    Some(trimmed.to_string())
}

impl Athena {
    /// Builds the invocation spec common to every role.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn agent_spec(
        &self,
        jc: &JobConfig,
        job: &JobId,
        seq: u32,
        role: Role,
        model: &str,
        composed: &Composed,
        user_prompt: String,
        grant: &ToolGrant,
        snapshot: &std::path::Path,
        side: Side,
        capability_env: &BTreeMap<String, String>,
        cap_usd: f64,
        timeout_s: u64,
        schema: serde_json::Value,
    ) -> Result<AgentInvocation> {
        let job_dir = self.state.job_dir(job);
        let scratch = job_dir.join("agent");
        std::fs::create_dir_all(&scratch)?;
        let policy = Policy {
            role,
            root: snapshot.to_path_buf(),
            commands: grant.commands.clone(),
            writes: grant.writes,
        };
        let policy_file = scratch.join(format!("policy-{seq:02}-{}.json", role.as_str()));
        std::fs::write(&policy_file, serde_json::to_string_pretty(&policy)?)?;
        let env = athena_agent::env::build(
            &EnvSpec {
                agents: &jc.config.agents,
                job: job.as_str(),
                job_dir: &job_dir,
                snapshot,
                side,
                capability_env,
            },
            |key| std::env::var(key).ok(),
        )
        .map_err(anyhow::Error::msg)?;
        Ok(AgentInvocation {
            job: job.as_str().to_string(),
            seq,
            role,
            model: model.to_string(),
            system_prompt: composed.text.clone(),
            user_prompt,
            tools: grant.tools.clone(),
            allowed_tools: athena_agent::policy::allowed_tool_rules(&grant.commands),
            permission_mode: grant.permission_mode.clone(),
            workdir: snapshot.to_path_buf(),
            env,
            hook: Some(athena_agent::HookSpec {
                athena_bin: self.athena_bin.clone(),
                policy_file,
            }),
            max_budget_usd: cap_usd,
            timeout: Duration::from_secs(timeout_s),
            json_schema: schema,
            scratch,
        })
    }

    pub(crate) fn is_fake(&self) -> bool {
        self.driver.info().name == "fake"
    }

    /// Refuses an invocation whose cap would exceed what is left of the job's budget (design §10.4).
    pub(crate) fn check_budget(
        &self,
        jc: &JobConfig,
        spent: f64,
        cap: f64,
        what: &str,
    ) -> Result<()> {
        let left = jc.config.budget.job_usd - spent;
        if cap > left + 1e-9 {
            bail!(
                "the job has ${:.2} of its ${:.2} budget left, less than the {what}'s cap of ${cap:.2}; \
                 raise [budget] in athena.toml through a job, or close this one",
                left.max(0.0),
                jc.config.budget.job_usd
            );
        }
        Ok(())
    }

    /// Runs the reader once and builds the reading.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn read_for(
        &self,
        jc: &JobConfig,
        records: &Records,
        job: &JobId,
        request: &Request,
        attempt: Option<&EarlierAttempt>,
        history: &[(Reading, Option<String>)],
        n: u32,
        seq: u32,
        spent: f64,
        redactor: &Redactor,
        ui: &dyn Ui,
    ) -> Result<(Option<Reading>, Invocation, Option<String>)> {
        self.check_budget(jc, spent, jc.config.budget.reader_usd, "reader")?;
        let snapshot_dir = self.state.job_dir(job).join(format!("read-{n}"));
        if snapshot_dir.exists() {
            std::fs::remove_dir_all(&snapshot_dir)?;
        }
        Snapshot::create(
            &self.git,
            &jc.commit,
            &jc.commit,
            &snapshot_dir,
            SnapshotRepo::None,
        )?;
        let (global_reference, basis) =
            crate::scoring::reference(&jc.config.forecast.reference, "", records);
        let composed = reader_system_prompt(self, jc);
        let referenced_jobs = referenced_jobs(
            records,
            &request.text,
            attempt.map(|a| a.job.as_str()),
            redactor,
        )?;
        let prompt = reader_brief(&ReaderInputs {
            request: request.text.clone(),
            earlier_attempt: attempt.cloned(),
            referenced_jobs,
            history: history.to_vec(),
            reference: global_reference,
            reference_basis: basis,
            classes: jc.config.classes.list.clone(),
            producer_models: jc.config.models.producer_allowed.clone(),
            producer_default: jc.config.models.producer_default.clone(),
            listing: listing(&snapshot_dir),
            verifier_can_run: self
                .capabilities
                .iter()
                .filter(|c| jc.config.capability(c.id()).is_some())
                .map(|c| {
                    let mut lines =
                        athena_agent::policy::describe_commands(&c.verifier_grant(jc).commands);
                    lines.push("cargo fmt only with --check".into());
                    (c.id().to_string(), lines)
                })
                .collect(),
        });
        let capability_ids: Vec<String> = self
            .capabilities
            .iter()
            .map(|c| c.id().to_string())
            .filter(|id| jc.config.capability(id).is_some())
            .collect();
        let schema = reading_schema(&ReaderChoices {
            classes: jc.config.classes.list.clone(),
            capabilities: capability_ids.clone(),
            producer_models: jc.config.models.producer_allowed.clone(),
        });
        let grant = ToolGrant {
            tools: vec!["Read".into(), "Grep".into(), "Glob".into()],
            commands: BTreeMap::new(),
            writes: false,
            permission_mode: "dontAsk".into(),
        };
        let model = jc.config.models.reader.clone();
        let spec = self.agent_spec(
            jc,
            job,
            seq,
            Role::Reader,
            &model,
            &composed,
            prompt,
            &grant,
            &snapshot_dir,
            Side::Read,
            &BTreeMap::new(),
            jc.config.budget.reader_usd,
            jc.config.budget.reader_timeout_s,
            schema,
        )?;
        ui.line(&format!(
            "reading    {} · {model} reads the request (reading {n})",
            job.as_str()
        ));
        let facts = InvocationFacts {
            job,
            spec: &spec,
            composed: &composed,
            config: &jc.config,
            fake: self.is_fake(),
        };
        let (output, record) = match self.invoke_and_record(&facts, redactor)? {
            Ok(done) => done,
            Err(error) => bail!("the reader could not run: {error}"),
        };
        let _ = std::fs::remove_dir_all(&snapshot_dir);
        if output.outcome != athena_data::records::InvocationOutcome::Success {
            let why = record
                .detail
                .clone()
                .unwrap_or_else(|| "the reader did not complete".into());
            return Ok((None, record, Some(why)));
        }
        let Some(structured) = output.structured.clone() else {
            return Ok((
                None,
                record,
                Some("the reader returned no structured reading".into()),
            ));
        };
        match self.build_reading(
            jc,
            records,
            job,
            n,
            seq,
            structured,
            &capability_ids,
            redactor,
        ) {
            Ok(reading) => Ok((Some(reading), record, None)),
            Err(e) => Ok((
                None,
                record,
                Some(format!("the reader's output was invalid: {e:#}")),
            )),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_reading(
        &self,
        jc: &JobConfig,
        records: &Records,
        job: &JobId,
        n: u32,
        seq: u32,
        structured: serde_json::Value,
        capabilities: &[String],
        redactor: &Redactor,
    ) -> Result<Reading> {
        let out: ReadingOutput =
            serde_json::from_value(structured).context("parsing the reading")?;
        let config = &jc.config;
        if !config.is_class(&out.class_of_work) {
            bail!("class {:?} is not in the closed list", out.class_of_work);
        }
        if let Some(cap) = &out.capability
            && !capabilities.contains(cap)
        {
            bail!("capability {cap:?} is not in the catalog");
        }
        if out.capability.is_none()
            && out
                .decline_reason
                .as_deref()
                .is_none_or(|r| r.trim().is_empty())
        {
            bail!("a reading without a capability must say why");
        }
        if !config.models.producer_allowed.contains(&out.producer_model) {
            bail!("producer model {:?} is not allowed", out.producer_model);
        }
        let f = &out.forecast;
        let reader = Forecast {
            p_reading_accepted: f.p_reading_accepted,
            p_verification_passes: f.p_verification_passes,
            p_user_intervenes: f.p_user_intervenes,
            cost_usd: Quantiles::new(f.cost_usd.p10, f.cost_usd.p50, f.cost_usd.p90)
                .map_err(anyhow::Error::msg)?,
            wall_time_s: Quantiles::new(f.wall_time_s.p10, f.wall_time_s.p50, f.wall_time_s.p90)
                .map_err(anyhow::Error::msg)?,
        };
        reader.validate().map_err(anyhow::Error::msg)?;
        let (reference, _) =
            crate::scoring::reference(&config.forecast.reference, &out.class_of_work, records);

        // Agent free text is redacted before it is recorded.
        let mut text: Vec<String> = vec![
            out.title.clone(),
            out.summary.clone(),
            out.decline_reason.clone().unwrap_or_default(),
            out.question.clone().unwrap_or_default(),
            f.departures.clone().unwrap_or_default(),
        ];
        let fixed = text.len();
        text.extend(out.assumptions.iter().cloned());
        text.extend(out.out_of_scope.iter().cloned());
        text.extend(out.acceptance_criteria.iter().map(|c| c.text.clone()));
        text.extend(out.defaults_applied.iter().map(|d| d.how.clone()));
        redactor.redact_all(&mut text)?;
        let mut rest = text.split_off(fixed).into_iter();
        let opt = |s: &String| (!s.trim().is_empty()).then(|| s.clone());
        let assumptions: Vec<String> = rest.by_ref().take(out.assumptions.len()).collect();
        let out_of_scope: Vec<String> = rest.by_ref().take(out.out_of_scope.len()).collect();
        let criteria_text: Vec<String> =
            rest.by_ref().take(out.acceptance_criteria.len()).collect();
        let hows: Vec<String> = rest.collect();

        let mut deliverables: Vec<Deliverable> = Vec::new();
        for d in &out.deliverables {
            let Some(path) = normalize_path(&d.path) else {
                bail!("deliverable path {:?} is not a repository path", d.path)
            };
            if deliverables.iter().any(|x| x.path == path) {
                continue;
            }
            deliverables.push(Deliverable {
                path,
                action: match d.action {
                    DeliverableActionOutput::Create => DeliverableAction::Create,
                    DeliverableActionOutput::Modify => DeliverableAction::Modify,
                    DeliverableActionOutput::Delete => DeliverableAction::Delete,
                },
                p: d.p,
            });
        }
        let title: String = text[0]
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(72)
            .collect();
        let reading = Reading {
            job: job.clone(),
            n,
            at: crate::now(),
            invocation_seq: seq,
            base_commit: jc.commit.clone(),
            config_sha256: jc.config_sha256.clone(),
            defaults_sha256: jc.defaults_sha256.clone(),
            defaults_version: jc.defaults.version,
            title,
            summary: text[1].clone(),
            class_of_work: out.class_of_work.clone(),
            capability: out.capability.clone(),
            decline_reason: opt(&text[2]),
            producer_model: out.producer_model.clone(),
            verifier_model: config.models.verifier.clone(),
            defaults_applied: out
                .defaults_applied
                .iter()
                .zip(hows)
                .map(|(d, how)| DefaultApplied {
                    id: d.id.clone(),
                    how,
                })
                .collect(),
            assumptions,
            question: opt(&text[3]),
            acceptance_criteria: out
                .acceptance_criteria
                .iter()
                .zip(criteria_text)
                .map(|(c, text)| Criterion {
                    text,
                    kind: match c.kind {
                        CriterionKindOutput::Content => CriterionKind::Content,
                        CriterionKindOutput::Behavior => CriterionKind::Behavior,
                    },
                })
                .collect(),
            deliverables,
            out_of_scope,
            forecasts: Forecasts { reader, reference },
            departures: opt(&text[4]),
        };
        reading.validate().map_err(anyhow::Error::msg)?;
        if reading.capability.is_some() && reading.acceptance_criteria.is_empty() {
            bail!("a reading needs at least one acceptance criterion");
        }
        Ok(reading)
    }

    /// `athena ask`: reads a request and commits the proposal.
    pub fn ask(&self, text: &str, ui: &dyn Ui) -> Result<Proposal> {
        self.refuse_if_stopped()?;
        self.state.registry().reap();
        if text.trim().is_empty() {
            bail!("the request is empty; say what you want in a line or two");
        }
        let head = self.head()?;
        let jc = JobConfig::at(&self.git, &head)?;
        let records = crate::store::Records::load(&self.git, &head)?;
        let mut id = RecordId::generate(crate::now());
        while records.jobs.contains_key(id.as_str()) {
            id = RecordId::generate(crate::now());
        }
        let job = JobId::Id(id);
        let redactor = self.redactor(&jc.config, Some(&job))?;
        let request = Request {
            job: job.clone(),
            text: redactor.redact(text.trim())?,
            received_at: crate::now(),
            channel: "cli".into(),
        };
        self.first_reading(&jc, &records, &request, true, ui)
    }

    /// `athena ask --later`: records a request to be read and run later. Nothing is read and no model is
    /// called; `athena status` lists it as open work, and `athena go` reads it.
    pub fn queue(&self, text: &str) -> Result<(JobId, String)> {
        if text.trim().is_empty() {
            bail!("the request is empty; say what you want in a line or two");
        }
        let head = self.head()?;
        let jc = JobConfig::at(&self.git, &head)?;
        let records = crate::store::Records::load(&self.git, &head)?;
        let mut id = RecordId::generate(crate::now());
        while records.jobs.contains_key(id.as_str()) {
            id = RecordId::generate(crate::now());
        }
        let job = JobId::Id(id);
        let redactor = self.redactor(&jc.config, Some(&job))?;
        let request = Request {
            job: job.clone(),
            text: redactor.redact(text.trim())?,
            received_at: crate::now(),
            channel: "cli".into(),
        };
        let first_line: String = request
            .text
            .lines()
            .next()
            .unwrap_or("")
            .chars()
            .take(60)
            .collect();
        let commit = self.commit_records(
            &[NewFile::record(&AnyRecord::Request(request))],
            &message(
                &format!("Queue a request: {first_line}"),
                "The request, recorded to be read and run later. Nothing was read and no model was called.",
                &[("Athena-Job", job.as_str().to_string())],
            ),
        )?;
        Ok((job, commit))
    }

    /// Reads a queued request for the first time, as `ask` would have.
    pub(crate) fn read_queued(&self, job: &JobRecords, ui: &dyn Ui) -> Result<Proposal> {
        self.refuse_if_stopped()?;
        let head = self.head()?;
        let jc = JobConfig::at(&self.git, &head)?;
        let records = crate::store::Records::load(&self.git, &head)?;
        let request = job
            .request
            .clone()
            .ok_or_else(|| anyhow::anyhow!("job {} has no request record", job.id()))?;
        self.first_reading(&jc, &records, &request, false, ui)
    }

    /// Reads `request` for the first time and commits the reading, with the request record when
    /// `with_request` is set.
    fn first_reading(
        &self,
        jc: &JobConfig,
        records: &Records,
        request: &Request,
        with_request: bool,
        ui: &dyn Ui,
    ) -> Result<Proposal> {
        let job = request.job.clone();
        let redactor = self.redactor(&jc.config, Some(&job))?;
        let attempt = earlier_attempt(records, &request.text, &redactor)?;
        let (reading, invocation, error) = self.read_for(
            jc,
            records,
            &job,
            request,
            attempt.as_ref(),
            &[],
            1,
            1,
            0.0,
            &redactor,
            ui,
        )?;
        let mut files = Vec::new();
        if with_request {
            files.push(NewFile::record(&AnyRecord::Request(request.clone())));
        }
        files.push(NewFile::record(&AnyRecord::Invocation(Box::new(
            invocation.clone(),
        ))));
        if let Some(r) = &reading {
            files.push(NewFile::record(&AnyRecord::Reading(Box::new(r.clone()))));
        }
        let subject = match &reading {
            Some(r) => format!("Read a request: {}", r.title),
            None => "Read a request (the reader failed)".to_string(),
        };
        let body = "The request, the reader's reading with its forecasts, and the reader's invocation, committed before any work.";
        let commit = self.commit_records(
            &files,
            &message(&subject, body, &[("Athena-Job", job.as_str().to_string())]),
        )?;
        Ok(Proposal {
            job,
            reading,
            reader_error: error,
            spent_usd: invocation.cost.best_lower(),
            invocation: Some(invocation),
            commit,
        })
    }

    /// `athena correct`: records a one-line correction (or an answer), and reads again.
    pub fn correct(&self, job_ref: &str, line: &str, ui: &dyn Ui) -> Result<Proposal> {
        self.refuse_if_stopped()?;
        if line.trim().is_empty() {
            bail!("the correction is empty");
        }
        let head = self.head()?;
        let records = crate::store::Records::load(&self.git, &head)?;
        let job_id = records.resolve(job_ref)?;
        let job = records.jobs.get(job_id.as_str()).expect("resolved").clone();
        match job.state() {
            JobState::Proposed | JobState::Declined => {}
            other => bail!(
                "job {} is {}; only a proposed or declined job can be corrected",
                job_id,
                other.name()
            ),
        }
        let jc = JobConfig::at(&self.git, &head)?;
        let max = jc.config.budget.max_readings;
        let n = job.readings.iter().map(|r| r.n).max().unwrap_or(0);
        if n >= max {
            bail!(
                "job {job_id} already has {n} readings, the most a job may have; close it and ask again with a fuller request"
            );
        }
        let redactor = self.redactor(&jc.config, Some(&job_id))?;
        let request = job
            .request
            .clone()
            .ok_or_else(|| anyhow::anyhow!("job {job_id} has no request record"))?;
        let line = redactor.redact(line.trim())?;
        let mut files = Vec::new();
        let mut intervention_id = None;
        if let Some(previous) = job.latest_reading() {
            let answer = previous.question.is_some();
            let id = RecordId::generate(crate::now());
            let intervention = Intervention {
                id: id.clone(),
                at: crate::now(),
                what: if answer {
                    format!(
                        "Answered the reader's question on reading {}: {line}",
                        previous.n
                    )
                } else {
                    format!("Corrected reading {}: {line}", previous.n)
                },
                why: if answer {
                    "The reader asked a question that the repository and the defaults did not settle.".into()
                } else {
                    "The reading did not match what the user meant.".into()
                },
                class_of_work: previous.class_of_work.clone(),
                category: if answer {
                    InterventionCategory::AnsweredQuestion
                } else {
                    InterventionCategory::CorrectedOutput
                },
                job: Some(job_id.clone()),
                source: InterventionSource::Observed,
                with_staged_changes: false,
            };
            files.push(NewFile::record(&AnyRecord::Correction(Correction {
                job: job_id.clone(),
                n: previous.n,
                at: crate::now(),
                kind: if answer {
                    CorrectionKind::Answer
                } else {
                    CorrectionKind::Correction
                },
                text: line.clone(),
                intervention: id.clone(),
            })));
            files.push(NewFile::record(&AnyRecord::Intervention(intervention)));
            intervention_id = Some(id);
        }
        // Earlier readings, each followed by the line that answered or corrected it.
        let mut corrections: BTreeMap<u32, String> = job
            .corrections
            .iter()
            .map(|c| (c.n, c.text.clone()))
            .collect();
        if let Some(previous) = job.latest_reading() {
            corrections.insert(previous.n, line.clone());
        }
        let history: Vec<(Reading, Option<String>)> = job
            .readings
            .iter()
            .map(|r| (r.clone(), corrections.get(&r.n).cloned()))
            .collect();
        let seq = job.next_seq();
        let spent = job.spent_usd();
        let attempt = earlier_attempt(&records, &request.text, &redactor)?;
        let (reading, invocation, error) = self.read_for(
            &jc,
            &records,
            &job_id,
            &request,
            attempt.as_ref(),
            &history,
            n + 1,
            seq,
            spent,
            &redactor,
            ui,
        )?;
        files.push(NewFile::record(&AnyRecord::Invocation(Box::new(
            invocation.clone(),
        ))));
        if let Some(r) = &reading {
            files.push(NewFile::record(&AnyRecord::Reading(Box::new(r.clone()))));
        }
        let subject = match &reading {
            Some(r) => format!("Read again after a correction: {}", r.title),
            None => "Read again after a correction (the reader failed)".to_string(),
        };
        let mut trailers = vec![("Athena-Job", job_id.as_str().to_string())];
        if let Some(id) = &intervention_id {
            trailers.push(("Athena-Intervention", id.as_str().to_string()));
        }
        let body = "The user's one-line correction, the intervention it implies, and the new reading with its forecasts.";
        let commit = self.commit_records(&files, &message(&subject, body, &trailers))?;
        Ok(Proposal {
            job: job_id,
            reading,
            reader_error: error,
            spent_usd: spent + invocation.cost.best_lower(),
            invocation: Some(invocation),
            commit,
        })
    }
}

const RETRY_PREFIX: &str = "try again on ";

/// Detects "try again on <job>" at the start of `request` and builds the earlier attempt from the
/// rejected job's request and final verification. Returns `None` when the request is not that form.
/// The carried text goes through `redactor`.
pub fn earlier_attempt(
    records: &Records,
    request: &str,
    redactor: &Redactor,
) -> Result<Option<EarlierAttempt>> {
    let Some(reference) = retry_reference(request)? else {
        return Ok(None);
    };
    let id = records.resolve(reference)?;
    let job = records.jobs.get(id.as_str()).expect("resolved");
    let state = job.state();
    if state != JobState::Closed(OutcomeState::Rejected) {
        bail!(
            "job {id} is {}; only a rejected job can be tried again",
            state.name()
        );
    }
    if job.request.is_none() {
        bail!("job {id} is {} but has no request record", state.name());
    }
    if job.verification.is_none() {
        bail!(
            "job {id} is {} but has no verification record",
            state.name()
        );
    }
    let mut attempt = carry(job, redactor)?.expect("has a request");
    // When the retried job was itself a retry, carry the original request instead of the retry phrase.
    if let Some(original) = original_request(records, job) {
        let mut texts = vec![original];
        redactor.redact_all(&mut texts)?;
        attempt.request = texts.remove(0);
    }
    Ok(Some(attempt))
}

/// The reference after "try again on " at the start of `request`; `None` when it is not that form.
fn retry_reference(request: &str) -> Result<Option<&str>> {
    let text = request.trim();
    if text.eq_ignore_ascii_case(RETRY_PREFIX.trim_end()) {
        bail!("say which job to try again on: 'try again on <job>'");
    }
    let Some(head) = text.get(..RETRY_PREFIX.len()) else {
        return Ok(None);
    };
    if !head.eq_ignore_ascii_case(RETRY_PREFIX) {
        return Ok(None);
    }
    let reference = text[RETRY_PREFIX.len()..]
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_end_matches(['.', ',', ';', ':', '!', '?']);
    if reference.is_empty() {
        bail!("say which job to try again on: 'try again on <job>'");
    }
    Ok(Some(reference))
}

/// The most retries followed back to an original request.
const MAX_RETRY_DEPTH: usize = 16;

/// Follows `job`'s "try again on <job>" request back to the first request that is not a retry.
/// `None` when `job` is not a retry; when the chain breaks (unknown job, no request, a cycle, or too
/// deep), the last request reached is returned. The text is not yet redacted.
fn original_request(records: &Records, job: &JobRecords) -> Option<String> {
    let mut current = job;
    let mut seen = vec![current.id().as_str().to_string()];
    let mut original = None;
    for _ in 0..MAX_RETRY_DEPTH {
        let text = &current.request.as_ref()?.text;
        let Some(reference) = retry_reference(text).ok().flatten() else {
            break;
        };
        let Some(next) = records
            .resolve(reference)
            .ok()
            .and_then(|id| records.jobs.get(id.as_str()))
        else {
            break;
        };
        let Some(next_request) = &next.request else {
            break;
        };
        if seen.iter().any(|s| s == next.id().as_str()) {
            break;
        }
        seen.push(next.id().as_str().to_string());
        original = Some(next_request.text.clone());
        current = next;
    }
    original
}

/// The most bytes of one failed check's output that are carried; the end of the output is kept.
const CARRIED_CHECK_OUTPUT_BYTES: usize = 1024;

/// The job's request, final verification findings and reasons, and each failed check's name, command,
/// exit code and output tail, redacted. `None` without a request.
fn carry(job: &JobRecords, redactor: &Redactor) -> Result<Option<EarlierAttempt>> {
    let Some(earlier) = &job.request else {
        return Ok(None);
    };
    let mut texts = vec![earlier.text.clone()];
    if let Some(verification) = &job.verification {
        texts.extend(verification.findings.iter().map(|f| {
            let location = f
                .location
                .as_deref()
                .map_or(String::new(), |l| format!(" ({l})"));
            format!(
                "{}{location}: {} {}",
                format!("{:?}", f.severity).to_lowercase(),
                f.summary,
                f.evidence
            )
            .trim_end()
            .to_string()
        }));
        texts.extend(verification.reasons.iter().map(|r| format!("reason: {r}")));
        texts.extend(verification.checks.iter().filter(|c| !c.passed).map(|c| {
            let exit = c.exit_code.map_or("no exit code".to_string(), |code| {
                format!("exit code {code}")
            });
            format!(
                "failed check {} (`{}`, {exit}), output:\n{}",
                c.name,
                c.command.join(" "),
                tail(&c.output_tail, CARRIED_CHECK_OUTPUT_BYTES)
            )
        }));
    }
    redactor.redact_all(&mut texts)?;
    let request = texts.remove(0);
    Ok(Some(EarlierAttempt {
        job: job.id().as_str().to_string(),
        request,
        findings: texts,
    }))
}

/// The most jobs a request may reference; more would bloat the reader brief.
pub const MAX_REFERENCED_JOBS: usize = 3;

/// Full job ids (`YYYYMMDD-HHMMSS-xxxx`) named anywhere in `text`, in order, without repeats.
pub fn job_ids_in(text: &str) -> Vec<RecordId> {
    let mut ids: Vec<RecordId> = Vec::new();
    for word in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
        if let Ok(id) = RecordId::parse(word)
            && !ids.contains(&id)
        {
            ids.push(id);
        }
    }
    ids
}

/// The jobs a request names by full id, other than `skip` (the retried job): each one's request and
/// final findings, redacted. Ids that match no job (or a job without a request) are ignored. At most
/// [`MAX_REFERENCED_JOBS`] are carried.
pub fn referenced_jobs(
    records: &Records,
    request: &str,
    skip: Option<&str>,
    redactor: &Redactor,
) -> Result<Vec<EarlierAttempt>> {
    let mut out = Vec::new();
    for id in job_ids_in(request) {
        if out.len() >= MAX_REFERENCED_JOBS {
            break;
        }
        if skip == Some(id.as_str()) {
            continue;
        }
        if let Some(job) = records.jobs.get(id.as_str())
            && let Some(carried) = carry(job, redactor)?
        {
            out.push(carried);
        }
    }
    Ok(out)
}

/// The records of a job at the current commit.
pub fn job_records(athena: &Athena, job_ref: &str) -> Result<(Records, JobRecords)> {
    let head = athena.head()?;
    let records = Records::load(&athena.git, &head)?;
    let id = records.resolve(job_ref)?;
    let job = records.jobs.get(id.as_str()).expect("resolved").clone();
    Ok((records, job))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_try_again_on_is_refused_and_a_named_job_is_found() {
        for bare in ["try again on", "Try again on   ", "try again on ."] {
            let error = retry_reference(bare).unwrap_err().to_string();
            assert!(error.contains("say which job"), "{bare:?}: {error}");
        }
        assert_eq!(
            retry_reference("Try again on 20261012-075010-86ja, please").unwrap(),
            Some("20261012-075010-86ja")
        );
        assert_eq!(retry_reference("add a greeting").unwrap(), None);
    }

    fn check(name: &str, passed: bool, output: &str) -> athena_data::records::CheckRun {
        athena_data::records::CheckRun {
            name: name.into(),
            command: vec!["cargo".into(), name.into()],
            exit_code: Some(if passed { 0 } else { 101 }),
            passed,
            tests_ran: None,
            duration: athena_data::Quantity::measured(1.0, athena_data::Unit::Seconds, "clock"),
            output_tail: output.into(),
            note: None,
        }
    }

    fn job_with_checks(checks: Vec<athena_data::records::CheckRun>) -> JobRecords {
        let id = JobId::parse("20261012-084936-n1p0").unwrap();
        JobRecords {
            id: Some(id.clone()),
            request: Some(athena_data::records::Request {
                job: id.clone(),
                text: "add a greeting".into(),
                received_at: jiff::Timestamp::UNIX_EPOCH,
                channel: "cli".into(),
            }),
            verification: Some(athena_data::records::Verification {
                job: id,
                candidate_commit: "0".repeat(40),
                verified_fingerprint: "0".repeat(64),
                tool_config_from_base: vec![],
                checks,
                check_cleanup: vec![],
                verifier_invocation: None,
                criteria: vec![],
                findings: vec![],
                request_reading_mismatch: None,
                agent_verdict: None,
                final_verdict: athena_data::records::Verdict::Fail,
                reasons: vec!["a check failed".into()],
                inputs: vec![],
                summary: None,
            }),
            ..JobRecords::default()
        }
    }

    #[test]
    fn carry_adds_only_failed_checks_after_the_reasons_redacted_and_bounded() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let token = format!(
            "{}{}",
            ["gh", "p_"].concat(),
            "Zq8rT2vK9mW4xN7bP3cH6jL1sD5fG0aYuE2i"
        );
        let noisy = format!("HEAD-NOISE {}TAIL-CANARY", "x".repeat(5000));
        let job = job_with_checks(vec![
            check("fmt", true, "PASSING-OUTPUT"),
            check("test", false, &format!("assertion failed\nkey {token}")),
            check("clippy", false, &noisy),
        ]);
        let carried = carry(&job, &redactor).unwrap().unwrap();
        assert_eq!(carried.findings.len(), 3, "{:?}", carried.findings);
        assert_eq!(carried.findings[0], "reason: a check failed");
        let test = &carried.findings[1];
        assert!(test.contains("failed check test"), "{test}");
        assert!(test.contains("cargo test"), "{test}");
        assert!(test.contains("exit code 101"), "{test}");
        assert!(test.contains("assertion failed"), "{test}");
        assert!(!test.contains(&token), "{test}");
        assert!(test.contains("[secret:github-pat]"), "{test}");
        let clippy = &carried.findings[2];
        assert!(clippy.contains("TAIL-CANARY"), "the end is kept");
        assert!(!clippy.contains("HEAD-NOISE"), "the head is dropped");
        assert!(clippy.len() < 1300, "{}", clippy.len());
        assert!(
            !carried
                .findings
                .iter()
                .any(|f| f.contains("PASSING-OUTPUT"))
        );
    }

    #[test]
    fn carry_without_failed_checks_adds_nothing() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let job = job_with_checks(vec![check("fmt", true, "PASSING-OUTPUT")]);
        let carried = carry(&job, &redactor).unwrap().unwrap();
        assert_eq!(carried.findings, ["reason: a check failed"]);
    }

    #[test]
    fn job_ids_are_found_mid_sentence_and_only_when_whole() {
        let ids = job_ids_in(
            "compare with 20261012-075824-1x28, (and 20261011-000000-abcd.) but not 20261012-075824-1x28 again",
        );
        let names: Vec<&str> = ids.iter().map(|i| i.as_str()).collect();
        assert_eq!(names, ["20261012-075824-1x28", "20261011-000000-abcd"]);
        assert!(job_ids_in("add 1x28 and 2026-10-12 and latest").is_empty());
        assert!(job_ids_in("x120261012-075824-1x28 or 20261012-075824-1x28z").is_empty());
    }
}
