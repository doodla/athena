//! `ask` and `correct`: the reader's reading, with forecasts, committed before any work (design §3.2–3.3).

use crate::capability::{JobConfig, ToolGrant};
use crate::checks::tail;
use crate::commits::{NewFile, message};
use crate::invocation::InvocationFacts;
use crate::plan::Priority;
use crate::store::{JobRecords, JobState, Records};
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_agent::AgentInvocation;
use athena_agent::env::{EnvSpec, Side};
use athena_agent::policy::Policy;
use athena_context::briefs::{
    EarlierAttempt, EarlierCase, ReaderInputs, RetryCause, reader_brief, render_catalog,
};
use athena_context::outputs::{
    CriterionKindOutput, DeliverableActionOutput, ReaderChoices, ReadingOutput, reading_schema,
};
use athena_context::{CONTRACT, Component, Composed, READER_FRAME, compose};
use athena_data::records::{
    AnyRecord, Correction, CorrectionKind, Criterion, CriterionKind, CritiqueVerdict,
    DefaultApplied, Deliverable, DeliverableAction, Finding, Forecast, Forecasts, Intervention,
    InterventionCategory, InterventionSource, IntrospectionRecord, IntrospectionStatus,
    IntrospectionTrigger, Invocation, OutcomeState, PreventionKind, PriorityRecord, Quantiles,
    Reading, Request, Role,
};
use athena_data::{JobId, RecordId};
use athena_gate::redact::Redactor;
use athena_git::snapshot::{Snapshot, SnapshotRepo};
use std::collections::BTreeMap;
use std::time::Duration;

/// Whether two readings promise the same work: the same capability, the same class of work, and the same
/// deliverables (path and action). Probabilities, wording, criteria and forecasts may differ. A reading
/// with no capability (a decline) is never the same scope as anything.
pub fn same_scope(old: &Reading, new: &Reading) -> bool {
    let deliverables = |r: &Reading| -> Vec<(String, String)> {
        let mut list: Vec<(String, String)> = r
            .deliverables
            .iter()
            .map(|d| (d.path.clone(), format!("{:?}", d.action)))
            .collect();
        list.sort();
        list
    };
    old.capability.is_some()
        && old.capability == new.capability
        && old.class_of_work == new.class_of_work
        && deliverables(old) == deliverables(new)
}

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

/// The path every change reading must list, so that the deliverables list is the one statement of scope.
const CHANGELOG_PATH: &str = "CHANGELOG.md";

/// Adds `CHANGELOG.md` to a change reading's deliverables, with a `changelog` default that cites BRIEF.md.
/// Nothing is added when the file is already listed, or when the reading is not a change.
pub(crate) fn ensure_changelog(reading: &mut Reading) {
    if reading.capability.as_deref() != Some("change") {
        return;
    }
    if !reading
        .deliverables
        .iter()
        .any(|d| d.path == CHANGELOG_PATH)
    {
        reading.deliverables.push(Deliverable {
            path: CHANGELOG_PATH.into(),
            action: DeliverableAction::Modify,
            p: 0.9,
        });
    }
    if !reading.defaults_applied.iter().any(|d| d.id == "changelog") {
        reading.defaults_applied.push(DefaultApplied {
            id: "changelog".into(),
            how: "BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md."
                .into(),
        });
    }
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
            effort: jc.config.effort_for(role).to_string(),
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
            out.hard_reason.clone().unwrap_or_default(),
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
        let title = reading_title(&text[0]);
        // A blank reason is no reason, so the job is not hard. A hard job gets the configured hard producer
        // setting, and the reason stays on the reading.
        let hard_reason = Some(text[5].trim().to_string()).filter(|r| !r.is_empty());
        let (producer_model, producer_effort) = config
            .models
            .producer_setting(&out.producer_model, hard_reason.is_some());
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
            producer_model,
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
            // A blank reason is no reason, so the request is not marked urgent.
            urgent_reason: out
                .urgent_reason
                .as_deref()
                .map(str::trim)
                .filter(|r| !r.is_empty())
                .map(str::to_string),
            producer_effort: hard_reason.is_some().then_some(producer_effort),
            hard_reason,
        };
        let mut reading = reading;
        ensure_changelog(&mut reading);
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

    /// `athena priority <job> <level>`: appends a priority record for a queued request. The request record
    /// is not touched; plan and status prefer the latest priority record over the request's words. The level
    /// is checked and the job must be queued before anything is committed.
    pub fn set_priority(&self, job_ref: &str, level: &str) -> Result<(JobId, Priority)> {
        let Some(priority) = Priority::parse(level) else {
            bail!(
                "unknown priority {level:?}; use one of {}",
                athena_data::records::PRIORITY_LEVELS.join(", ")
            );
        };
        let head = self.head()?;
        let records = crate::store::Records::load(&self.git, &head)?;
        let job_id = records.resolve(job_ref)?;
        let job = records.jobs.get(job_id.as_str()).expect("resolved");
        if job.state() != JobState::Queued {
            bail!(
                "job {job_id} is {}; only a queued request can be given a priority",
                job.state().name()
            );
        }
        let n = job.priorities.iter().map(|p| p.n).max().unwrap_or(0) + 1;
        self.commit_records(
            &[NewFile::record(&AnyRecord::Priority(PriorityRecord {
                job: job_id.clone(),
                n,
                at: crate::now(),
                priority: priority.as_str().to_string(),
            }))],
            &message(
                &format!("Set priority {}: {job_id}", priority.as_str()),
                "A priority record for a queued request. The request is unchanged; plan and status use the latest priority record.",
                &[("Athena-Job", job_id.as_str().to_string())],
            ),
        )?;
        Ok((job_id, priority))
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
        let attempt =
            earlier_attempt_with(records, &request.text, &redactor, &|j| self.running(j))?;
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

    /// `athena retry <job>`: reads a retry of a rejected, failed or interrupted job (dispatched with no
    /// process working on it, for example after a restart), seeded from that job's
    /// introspection record when it has one. The request recorded is "try again on <job id>", so a later
    /// retry of this job finds the original; nothing is run. The flag says whether the introspection
    /// record seeded the reading (false: the plain "try again on" behavior).
    pub fn retry(&self, job_ref: &str, ui: &dyn Ui) -> Result<(Proposal, bool)> {
        let head = self.head()?;
        let records = crate::store::Records::load(&self.git, &head)?;
        let id = records.resolve(job_ref)?;
        let job = records.jobs.get(id.as_str()).expect("resolved");
        // A paused line is refused before any model is called (`scope_guard`).
        if let Some(pause) = self.line_pause(&records, id.as_str()) {
            bail!("{}", self.pause_refusal(&pause));
        }
        let seeded = has_retry_introspection(&records, job);
        // `ask` refuses a job that is not rejected, failed or interrupted before any model is called.
        self.state.registry().reap();
        let proposal = self.ask(&format!("{RETRY_PREFIX}{id}"), ui)?;
        Ok((proposal, seeded))
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
        let attempt =
            earlier_attempt_with(&records, &request.text, &redactor, &|j| self.running(j))?;
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
        // A correction is an event: it is introspected at once; this never blocks or fails the command.
        if let Some(id) = &intervention_id {
            self.introspect_intervention(id, ui);
        }
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
/// rejected or failed job's request, final verification and (for a failed or unverified job) outcome
/// reason. Returns `None` when the request is not that form.
/// The carried text goes through `redactor`.
pub fn earlier_attempt(
    records: &Records,
    request: &str,
    redactor: &Redactor,
) -> Result<Option<EarlierAttempt>> {
    earlier_attempt_with(records, request, redactor, &|_| false)
}

/// Like [`earlier_attempt`], with `running` saying whether a process still works on a job. A dispatched job
/// that no process works on was interrupted (for example by a restart) and can be tried again, though it
/// may have no verification or outcome record; one that is running is refused.
pub fn earlier_attempt_with(
    records: &Records,
    request: &str,
    redactor: &Redactor,
    running: &dyn Fn(&str) -> bool,
) -> Result<Option<EarlierAttempt>> {
    let Some(reference) = retry_reference(request)? else {
        return Ok(None);
    };
    let id = records.resolve(reference)?;
    let job = records.jobs.get(id.as_str()).expect("resolved");
    let state = job.state();
    let interrupted = state == JobState::Dispatched && !running(id.as_str());
    if state == JobState::Dispatched && !interrupted {
        bail!("job {id} is running; wait for it to finish before trying it again");
    }
    if !interrupted
        && !matches!(
            state,
            JobState::Closed(
                OutcomeState::Rejected | OutcomeState::Failed | OutcomeState::TimedOut
            )
        )
    {
        bail!(
            "job {id} is {}; only a rejected or failed job can be tried again (so can a timed out one), or one that was interrupted",
            state.name()
        );
    }
    if job.request.is_none() {
        bail!("job {id} is {} but has no request record", state.name());
    }
    if !interrupted && job.verification.is_none() && job.outcome.is_none() {
        bail!(
            "job {id} is {} but has no verification or outcome record",
            state.name()
        );
    }
    let mut attempt = carry(job, redactor)?.expect("has a request");
    // When the retried job was itself a retry, carry the original request instead of the retry phrase,
    // and the corrections given to the jobs it retried (oldest first) ahead of its own.
    let earlier = earlier_jobs(records, job);
    if let Some(original) = earlier.last() {
        let mut texts = vec![original.request.as_ref().expect("checked").text.clone()];
        let inherited: Vec<&str> = earlier
            .iter()
            .rev()
            .flat_map(|j| j.corrections.iter().map(|c| c.text.as_str()))
            .collect();
        texts.extend(inherited.iter().map(|t| t.to_string()));
        redactor.redact_all(&mut texts)?;
        attempt.request = texts.remove(0);
        texts.append(&mut attempt.corrections);
        attempt.corrections = texts;
    }
    attempt.introspection = retry_cause(records, job, redactor)?;
    Ok(Some(attempt))
}

/// The job's own introspection of its rejection or failure that can seed a retry: the latest recorded
/// one with a proposed request.
fn retry_introspection<'a>(
    records: &'a Records,
    job: &JobRecords,
) -> Option<&'a IntrospectionRecord> {
    records.introspections.iter().rev().find(|i| {
        i.job.as_ref() == Some(job.id())
            && matches!(
                i.trigger,
                IntrospectionTrigger::Rejection | IntrospectionTrigger::Failure
            )
            && i.status == IntrospectionStatus::Recorded
            && i.proposed_request.is_some()
            && i.cause.is_some()
            && i.prevention.is_some()
    })
}

/// The longest one-line account of an earlier case that is carried, in characters.
const CASE_WHAT_CHARS: usize = 160;

/// The cause, prevention, proposed request and earlier cases from `job`'s introspection record, redacted;
/// `None` when the job has no such record. Earlier cases are the ids the record stored; classes are not
/// matched again.
fn retry_cause(
    records: &Records,
    job: &JobRecords,
    redactor: &Redactor,
) -> Result<Option<RetryCause>> {
    let Some(record) = retry_introspection(records, job) else {
        return Ok(None);
    };
    let kind_and_text = |kind: Option<PreventionKind>, text: &str| match kind {
        Some(kind) => format!("{}: {text}", kind.as_str()),
        None => text.to_string(),
    };
    let class = record.failure_class.as_ref().map(|id| {
        let name = records
            .classes
            .iter()
            .find(|c| &c.id == id)
            .map_or("unknown class", |c| c.name.as_str());
        format!("{name} ({id})")
    });
    // Everything is redacted in one pass: the cause fields first, then each case's text.
    let mut texts = vec![
        record.cause.clone().unwrap_or_default(),
        kind_and_text(
            record.prevention_kind,
            record.prevention.as_deref().unwrap_or_default(),
        ),
        record.proposed_request.clone().unwrap_or_default(),
    ];
    let mut cases = Vec::new();
    for id in &record.earlier_cases {
        let Some(case) = records.introspections.iter().find(|c| &c.id == id) else {
            continue;
        };
        let what = case
            .what_happened
            .as_deref()
            .and_then(|w| w.lines().find(|l| !l.trim().is_empty()))
            .unwrap_or("(no account recorded)")
            .trim()
            .chars()
            .take(CASE_WHAT_CHARS)
            .collect::<String>();
        let prevention = case
            .prevention
            .as_deref()
            .map(|p| kind_and_text(case.prevention_kind, p));
        texts.push(what);
        texts.push(prevention.clone().unwrap_or_default());
        cases.push((case, prevention.is_some()));
    }
    redactor.redact_all(&mut texts)?;
    let mut texts = texts.into_iter();
    let cause = texts.next().expect("three leading texts");
    let prevention = texts.next().expect("three leading texts");
    let proposed_request = texts.next().expect("three leading texts");
    let earlier_cases = cases
        .into_iter()
        .map(|(case, has_prevention)| {
            let what = texts.next().expect("two texts per case");
            let text = texts.next().expect("two texts per case");
            EarlierCase {
                id: case.id.to_string(),
                job: case.job.as_ref().map(ToString::to_string),
                what,
                prevention: has_prevention.then_some(text),
            }
        })
        .collect();
    Ok(Some(RetryCause {
        introspection: record.id.to_string(),
        class,
        cause,
        prevention,
        proposed_request,
        earlier_cases,
    }))
}

/// Whether `job` has an introspection record that `athena retry` can seed a retry from.
pub fn has_retry_introspection(records: &Records, job: &JobRecords) -> bool {
    retry_introspection(records, job).is_some()
}

/// The job a request of the form "try again on <job>" retries; `None` when it is not that form or names no
/// job.
pub fn retried_job(records: &Records, request: &str) -> Option<JobId> {
    let reference = retry_reference(request).ok().flatten()?;
    records.resolve(reference).ok()
}

/// Whether a retry of a job may start its producer from the job's rejected candidate instead of the base.
///
/// True only when all of these hold, read from the job's records and never from prose:
/// - the job was rejected (not failed, interrupted, blocked, landed or dropped);
/// - it has a candidate with a commit and a tree and no admission violations;
/// - its verification recorded at least one check, every check passed, and the verifier found no mismatch
///   between the request and the reading;
/// - no behavior criterion is unmet or cites a command that was not seen running; and
/// - either at least one criterion is unmet and every unmet criterion is of kind content, or no criterion is
///   unmet, at least one finding is a blocker or major, and every such blocking finding cites only non-code
///   files (Markdown, a CHANGELOG, or anything under `docs/`; see
///   [`finding_is_content_only`](crate::verify::finding_is_content_only)). Minor and nit findings are ignored.
///
/// Otherwise a rejection that names no unmet criterion and no blocking finding has an unknown cause, so it
/// does not qualify, and a blocking finding that cites a code file (or no recognizable path) restarts from
/// the base. The new candidate is verified in full either way.
pub fn qualifies_for_tree_retry(
    state: JobState,
    candidate: Option<&athena_data::records::Candidate>,
    verification: Option<&athena_data::records::Verification>,
) -> bool {
    if state != JobState::Closed(OutcomeState::Rejected) {
        return false;
    }
    let (Some(candidate), Some(verification)) = (candidate, verification) else {
        return false;
    };
    if candidate.candidate_commit.is_none()
        || candidate.candidate_tree.is_none()
        || !candidate.violations.is_empty()
    {
        return false;
    }
    if verification.checks.is_empty()
        || !verification.checks.iter().all(|c| c.passed)
        || verification.request_reading_mismatch.is_some()
    {
        return false;
    }
    let behavior_trouble = verification
        .criteria
        .iter()
        .any(|c| c.kind == CriterionKind::Behavior && (!c.met || !c.commands_verified));
    if behavior_trouble {
        return false;
    }
    let mut unmet = verification.criteria.iter().filter(|c| !c.met).peekable();
    if unmet.peek().is_some() {
        return unmet.all(|c| c.kind == CriterionKind::Content);
    }
    let mut blocking = verification
        .findings
        .iter()
        .filter(|f| crate::verify::finding_blocks(f))
        .peekable();
    blocking.peek().is_some() && blocking.all(crate::verify::finding_is_content_only)
}

/// What a producer retry seeded from a rejected candidate starts from and is told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeRetrySeed {
    /// The rejected job.
    pub job: JobId,
    /// The base commit the rejected job's candidate was made against.
    pub base_commit: String,
    /// The rejected candidate's commit, whose visible files seed the working directory.
    pub candidate_commit: String,
    /// The unmet criteria with the verifier's evidence, redacted.
    pub unmet: Vec<String>,
    /// The verification's findings, redacted.
    pub findings: Vec<String>,
    /// The blocker and major findings among `findings`: those the retry must address.
    pub must_address: Vec<String>,
}

/// The seed for a request of the form "try again on <job>" when [`qualifies_for_tree_retry`] holds for that
/// job; `None` for any other request or job. The caller must still check that the visible base has not
/// changed since `base_commit` before it seeds from the candidate.
pub fn tree_retry_seed(
    records: &Records,
    request: &str,
    redactor: &Redactor,
) -> Result<Option<TreeRetrySeed>> {
    let Some(id) = retried_job(records, request) else {
        return Ok(None);
    };
    let Some(job) = records.jobs.get(id.as_str()) else {
        return Ok(None);
    };
    if !qualifies_for_tree_retry(
        job.state(),
        job.candidate.as_ref(),
        job.verification.as_ref(),
    ) {
        return Ok(None);
    }
    let (Some(candidate), Some(verification)) = (&job.candidate, &job.verification) else {
        return Ok(None);
    };
    let Some(candidate_commit) = candidate.candidate_commit.clone() else {
        return Ok(None);
    };
    let mut texts: Vec<String> = verification
        .criteria
        .iter()
        .filter(|c| !c.met)
        .map(|c| format!("{}: {}", c.criterion, c.evidence))
        .collect();
    let unmet_count = texts.len();
    texts.extend(verification.findings.iter().map(finding_text));
    redactor.redact_all(&mut texts)?;
    let findings = texts.split_off(unmet_count);
    // The blocking findings are picked from the redacted list by position, so both retry paths show the
    // same text for the same finding.
    let must_address = verification
        .findings
        .iter()
        .zip(&findings)
        .filter(|(f, _)| crate::verify::finding_blocks(f))
        .map(|(_, text)| text.clone())
        .collect();
    Ok(Some(TreeRetrySeed {
        job: id,
        base_commit: candidate.base_commit.clone(),
        candidate_commit,
        unmet: texts,
        findings,
        must_address,
    }))
}

/// One finding as a retry brief shows it: severity, location, summary and evidence.
fn finding_text(f: &Finding) -> String {
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
}

/// The job a request "try again on <job>" retries, and every blocker and major finding of that job's final
/// verification, redacted, in the form [`tree_retry_seed`] uses. Findings have no resolved state, so none is
/// left out because a later round might have fixed it. `None` when the request is not a retry, the job has
/// no verification, or no finding blocks.
pub fn retry_findings(
    records: &Records,
    request: &str,
    redactor: &Redactor,
) -> Result<Option<(JobId, Vec<String>)>> {
    let Some(id) = retried_job(records, request) else {
        return Ok(None);
    };
    let Some(verification) = records
        .jobs
        .get(id.as_str())
        .and_then(|j| j.verification.as_ref())
    else {
        return Ok(None);
    };
    let mut texts: Vec<String> = verification
        .findings
        .iter()
        .filter(|f| crate::verify::finding_blocks(f))
        .map(finding_text)
        .collect();
    if texts.is_empty() {
        return Ok(None);
    }
    redactor.redact_all(&mut texts)?;
    Ok(Some((id, texts)))
}

/// The kept plan of a job that stopped at the planning round cap, and the critic's findings on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanningResumeSeed {
    /// The job that stopped at the cap.
    pub job: JobId,
    /// Its last plan, from its `job-plan-<round>.json`.
    pub plan: athena_data::records::JobPlan,
    /// Every finding of the critique of that plan, from its `critique-<round>.json`.
    pub findings: Vec<Finding>,
}

/// The seed for a request of the form "try again on <job>" when that job failed in planning because its plan
/// still needed revising in the last round; `None` for any other request, or a job that stopped for any other
/// reason (a reject, a misread reading, a timeout, a failed verification, an agent failure). The plan and
/// findings were redacted when they were recorded.
pub fn planning_resume_seed(records: &Records, request: &str) -> Option<PlanningResumeSeed> {
    let id = retried_job(records, request)?;
    let job = records.jobs.get(id.as_str())?;
    let outcome = job.outcome.as_ref()?;
    if outcome.state != OutcomeState::Failed {
        return None;
    }
    let why = outcome
        .reason
        .strip_prefix(athena_data::records::PLANNING_STOPPED_PREFIX)?;
    if !why
        .trim_start()
        .starts_with(athena_data::records::PLANNING_CAP_REASON)
    {
        return None;
    }
    let plan = job.plans.iter().max_by_key(|p| p.round)?;
    let critique = job.critiques.iter().find(|c| c.round == plan.round)?;
    if critique.final_verdict != CritiqueVerdict::Revise
        || critique.request_reading_mismatch.is_some()
    {
        return None;
    }
    Some(PlanningResumeSeed {
        job: id,
        plan: plan.clone(),
        findings: critique.findings.clone(),
    })
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

/// The jobs `job` is a retry of, nearest first, back to the first request that is not a retry. Empty
/// when `job` is not a retry; when the chain breaks (unknown job, no request, a cycle, or too deep),
/// the jobs reached so far are returned.
fn earlier_jobs<'a>(records: &'a Records, job: &'a JobRecords) -> Vec<&'a JobRecords> {
    let mut current = job;
    let mut seen = vec![current.id().as_str().to_string()];
    let mut chain = Vec::new();
    for _ in 0..MAX_RETRY_DEPTH {
        let Some(request) = current.request.as_ref() else {
            break;
        };
        let Some(reference) = retry_reference(&request.text).ok().flatten() else {
            break;
        };
        let Some(next) = records
            .resolve(reference)
            .ok()
            .and_then(|id| records.jobs.get(id.as_str()))
        else {
            break;
        };
        if next.request.is_none() || seen.iter().any(|s| s == next.id().as_str()) {
            break;
        }
        seen.push(next.id().as_str().to_string());
        chain.push(next);
        current = next;
    }
    chain
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
        texts.extend(verification.findings.iter().map(finding_text));
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
    // A job stopped in planning has no verification: the findings of the critique that stopped it say what
    // was wrong with the plan, so a retry carries them and not only the outcome reason (review F11).
    if let Some(critique) = job
        .critiques
        .last()
        .filter(|c| c.final_verdict != CritiqueVerdict::Accept)
    {
        if let Some(how) = &critique.request_reading_mismatch {
            texts.push(format!(
                "plan critique, round {}: the reading misread the request: {how}",
                critique.round
            ));
        }
        texts.extend(critique.findings.iter().map(|f| {
            let location = f
                .location
                .as_deref()
                .map_or(String::new(), |l| format!(" ({l})"));
            format!(
                "plan critique, round {}, {}{location}: {} {}",
                critique.round,
                format!("{:?}", f.severity).to_lowercase(),
                f.summary,
                f.evidence
            )
            .trim_end()
            .to_string()
        }));
    }
    // A job closed before verification has only its outcome reason to say why; a failed job's reason
    // (for example an admission refusal) is carried even when it was verified.
    let failed = job
        .outcome
        .as_ref()
        .is_some_and(|o| matches!(o.state, OutcomeState::Failed | OutcomeState::TimedOut));
    if let Some(outcome) = &job.outcome
        && (failed || job.verification.is_none())
        && !outcome.reason.trim().is_empty()
    {
        texts.push(format!("outcome: {}", outcome.reason));
    }
    redactor.redact_all(&mut texts)?;
    let request = texts.remove(0);
    // The final reading and the user's corrections and answers, redacted like the rest.
    let final_reading = job.latest_reading();
    let criteria = final_reading.map_or(0, |r| r.acceptance_criteria.len());
    let mut extra: Vec<String> = Vec::new();
    if let Some(reading) = final_reading {
        extra.push(reading.summary.clone());
        extra.extend(reading.acceptance_criteria.iter().map(|c| {
            let kind = match c.kind {
                CriterionKind::Content => "content",
                CriterionKind::Behavior => "behavior",
            };
            format!("[{kind}] {}", c.text)
        }));
        extra.extend(reading.deliverables.iter().map(|d| {
            let action = match d.action {
                DeliverableAction::Create => "create",
                DeliverableAction::Modify => "modify",
                DeliverableAction::Delete => "delete",
            };
            format!("{action} {}", d.path)
        }));
    }
    let lead = extra.len();
    extra.extend(job.corrections.iter().map(|c| c.text.clone()));
    redactor.redact_all(&mut extra)?;
    let corrections = extra.split_off(lead);
    let mut extra = extra.into_iter();
    let reading_summary = final_reading.and_then(|_| extra.next());
    let reading_criteria: Vec<String> = extra.by_ref().take(criteria).collect();
    Ok(Some(EarlierAttempt {
        introspection: None,
        job: job.id().as_str().to_string(),
        request,
        failed,
        findings: texts,
        reading_summary,
        reading_criteria,
        reading_deliverables: extra.collect(),
        corrections,
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

/// Whether a word of a request looks like a repository path: it has a directory part or a file extension.
pub(crate) fn looks_like_path(word: &str) -> bool {
    let w = word.trim_matches(|c: char| !(c.is_ascii_alphanumeric() || "/._-".contains(c)));
    !w.is_empty()
        && !w.starts_with('/')
        && !w.contains("..")
        && (w.contains('/')
            || [".rs", ".md", ".toml", ".json", ".yml"]
                .iter()
                .any(|ext| w.ends_with(ext)))
}

/// What planning looks up in the repository and its records, so `plan_entry` stays free of I/O.
pub struct Lookup<'a> {
    /// Whether a job with this full id is among the records.
    pub job_exists: &'a dyn Fn(&str) -> bool,
    /// Whether this path (a directory ends with `/`) exists at the base commit.
    pub path_exists: &'a dyn Fn(&str) -> bool,
    /// What BRIEF.md says the owner wants (see [`brief_topics`]); empty when there is no brief.
    pub brief: &'a [BriefTopic],
}

impl std::fmt::Debug for Lookup<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lookup").finish_non_exhaustive()
    }
}

/// One section of BRIEF.md that a request can serve: a requirement or an owner addendum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BriefTopic {
    /// The `###` heading.
    pub heading: String,
    /// The section's body. Kept only for addenda; a requirement is matched by its heading alone, because its
    /// long body would match nearly any request.
    pub text: String,
}

/// The sections of `brief` (the text of BRIEF.md) that a request can serve: every `###` heading, and for those
/// under `## Addenda` the body too.
pub fn brief_topics(brief: &str) -> Vec<BriefTopic> {
    let mut topics: Vec<BriefTopic> = Vec::new();
    let mut in_addenda = false;
    let mut in_topic = false;
    for line in brief.lines() {
        if let Some(h) = line.strip_prefix("### ") {
            topics.push(BriefTopic {
                heading: h.trim().to_string(),
                text: String::new(),
            });
            in_topic = true;
        } else if line.starts_with('#') {
            in_addenda = line.trim_end() == "## Addenda";
            in_topic = false;
        } else if in_topic
            && in_addenda
            && let Some(t) = topics.last_mut()
        {
            t.text.push_str(line);
            t.text.push('\n');
        }
    }
    topics
}

/// Words too common in BRIEF.md to say which section a request serves.
const COMMON_WORDS: [&str; 30] = [
    "that", "this", "with", "from", "have", "they", "them", "will", "should", "would", "could",
    "about", "which", "there", "their", "athena", "system", "what", "when", "also", "into", "make",
    "work", "then", "than", "each", "only", "some", "just", "need",
];

/// The words of `text` that can tie a request to a brief section: lowercase, a trailing `s` dropped, at least
/// four letters (or `ci`), and neither common nor one of [`IMPLEMENTATION_WORDS`].
fn topic_words(text: &str) -> std::collections::BTreeSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .map(|w| w.strip_suffix('s').unwrap_or(w))
        .filter(|w| {
            (w.len() >= 4 || *w == "ci")
                && !w.chars().all(|c| c.is_ascii_digit())
                && !COMMON_WORDS.contains(w)
                && !IMPLEMENTATION_WORDS.contains(w)
        })
        .map(String::from)
        .collect()
}

/// The topic of `topics` that `hay` shares the most words with, when it shares at least two.
fn serves_topic<'t>(hay: &str, topics: &'t [BriefTopic]) -> Option<&'t BriefTopic> {
    let words = topic_words(hay);
    topics
        .iter()
        .map(|t| {
            let theirs = topic_words(&format!("{} {}", t.heading, t.text));
            (words.intersection(&theirs).count(), t)
        })
        .filter(|(shared, _)| *shared >= 2)
        .max_by_key(|(shared, _)| *shared)
        .map(|(_, t)| t)
}

/// Words that say a request is about code, tests or docs, not which requirement it serves. They are never a
/// rung on their own: the rung comes from what the request serves in BRIEF.md and its addenda. They are
/// ignored when a request is matched to a brief section, since they are everywhere in a brief about software.
const IMPLEMENTATION_WORDS: [&str; 18] = [
    "readme",
    "changelog",
    "docs",
    "test",
    "tests",
    "code",
    "bug",
    "fix",
    "typo",
    "refactor",
    "rename",
    "function",
    "command",
    "crate",
    "module",
    "clippy",
    "lint",
    "cli",
];

/// The rung a request serves, found from what it serves in BRIEF.md and the owner's addenda; `None` when
/// nothing says. Deterministic and cheap, and a heuristic, in this order:
///
/// 1. A phrase naming a higher rung (an owner goal, a brief change, a design decision, a user default).
/// 2. The request or its reading shares at least two significant words with a section of `brief` (a
///    requirement heading or an addendum, see [`brief_topics`]): it serves the brief, so
///    BriefRequirements. This is how the owner's latest priorities rank above plain implementation work.
/// 3. A touched path or a reading for the `change` capability: Implementation, as plain work that serves no
///    particular brief section.
///
/// The [`IMPLEMENTATION_WORDS`] are not evidence. It is never the answer by default.
pub fn infer_rung(
    text: &str,
    reading: Option<&Reading>,
    touches: &[String],
    brief: &[BriefTopic],
) -> Option<athena_data::roles::Ladder> {
    use athena_data::roles::Ladder;
    let mut hay = text.to_lowercase();
    if let Some(r) = reading {
        hay.push(' ');
        hay.push_str(&r.summary.to_lowercase());
    }
    let has = |phrases: &[&str]| phrases.iter().any(|p| hay.contains(p));
    if has(&["owner goal", "owner's goal"]) {
        Some(Ladder::OwnerGoals)
    } else if has(&[
        "brief requirement",
        "change the brief",
        "amend the brief",
        "update the brief",
    ]) {
        Some(Ladder::BriefRequirements)
    } else if has(&["design decision", "record a decision", "new decision"]) {
        Some(Ladder::DesignDecisions)
    } else if has(&["user default", "defaults-conventions"]) {
        Some(Ladder::DefaultsConventions)
    } else if serves_topic(&hay, brief).is_some() {
        Some(Ladder::BriefRequirements)
    } else if !touches.is_empty()
        || reading.is_some_and(|r| r.capability.as_deref() == Some("change"))
    {
        Some(Ladder::Implementation)
    } else {
        None
    }
}

/// Whether a line of a request is an explicit acceptance bullet: it starts with `-`, `*` or a number and a dot.
fn is_bullet(line: &str) -> bool {
    line.starts_with("- ")
        || line.starts_with("* ")
        || line
            .split_once(". ")
            .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// The acceptance of a request that has not been read: its lines that start with `-`, `*` or a number and a
/// dot. This is the parse `athena plan` uses.
pub fn request_text_acceptance(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| is_bullet(l))
        .map(String::from)
        .collect()
}

/// What a request that has not been read touches: its path-like words, without repeats. This is the parse
/// `athena plan` uses.
pub fn request_text_touches(text: &str) -> Vec<String> {
    let mut touches: Vec<String> = Vec::new();
    for word in text.split_whitespace().filter(|w| looks_like_path(w)) {
        let path = word
            .trim_matches(|c: char| !(c.is_ascii_alphanumeric() || "/._-".contains(c)))
            .to_string();
        if !touches.contains(&path) {
            touches.push(path);
        }
    }
    touches
}

/// Planning, the first stage: fleshes out one open request into a plan entry (design: planning first).
/// Deterministic code; it calls no model and fills nothing in by default.
///
/// - Once the request has been read, the goal is the reading's summary, the acceptance is its criteria (only
///   when the reading chose a capability: a declined reading has no acceptance), the open question is its
///   question and what it touches is its deliverables.
/// - Before it is read, the request's own words stand in: the first line is the goal; only the lines that
///   start with `-`, `*` or a number are the acceptance (the first line is **not** copied into it); path-like
///   words are what it touches.
/// - The rung is `rung_hint` when the caller knows it (a steward proposal carries it), else [`infer_rung`]
///   from what the request and the reading serve in `lookup.brief`, else `None`, which the plan flags.
/// - A request depends on every other job whose full id it names (a retry, or "after <job>").
/// - Open questions are also filled by lookup: every touched path that does not exist (a file a reading
///   creates is excepted), and every referenced job that is not among the records.
pub fn plan_entry(
    job: &JobRecords,
    rung_hint: Option<athena_data::roles::Ladder>,
    lookup: &Lookup<'_>,
) -> crate::plan::PlanEntry {
    let text = job.request.as_ref().map_or("", |r| r.text.as_str());
    let first = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let mut entry = crate::plan::PlanEntry::bare(job.id().as_str(), &job.title());
    entry.priority = job.priority();
    entry.depends_on = job_ids_in(text)
        .into_iter()
        .map(|id| id.as_str().to_string())
        .filter(|id| id != job.id().as_str())
        .collect();
    let mut creates: Vec<String> = Vec::new();
    match job.latest_reading() {
        Some(reading) => {
            entry.goal = if reading.summary.trim().is_empty() {
                first.trim().to_string()
            } else {
                reading.summary.clone()
            };
            if reading.capability.is_some() {
                entry.acceptance = reading
                    .acceptance_criteria
                    .iter()
                    .map(|c| c.text.clone())
                    .collect();
            }
            entry.open_questions = reading.question.iter().cloned().collect();
            entry.touches = reading
                .deliverables
                .iter()
                .map(|d| d.path.clone())
                .collect();
            creates = reading
                .deliverables
                .iter()
                .filter(|d| d.action == DeliverableAction::Create)
                .map(|d| d.path.clone())
                .collect();
        }
        None => {
            entry.goal = first.trim().to_string();
            entry.acceptance = request_text_acceptance(text);
            entry.touches = request_text_touches(text);
        }
    }
    entry.rung =
        rung_hint.or_else(|| infer_rung(text, job.latest_reading(), &entry.touches, lookup.brief));
    for path in &entry.touches {
        if !creates.contains(path) && !(lookup.path_exists)(path) {
            entry.open_questions.push(format!(
                "the touched path {path} does not exist at the base commit"
            ));
        }
    }
    for id in &entry.depends_on {
        if !(lookup.job_exists)(id) {
            entry
                .open_questions
                .push(format!("the referenced job {id} is not among the records"));
        }
    }
    entry
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

/// A reading's title: the first line of the reader's text, cut at a word boundary.
fn reading_title(text: &str) -> String {
    crate::truncate_at_word(text.lines().next().unwrap_or(""), crate::SUBJECT_MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_long_reading_title_is_cut_at_a_word_boundary() {
        let text = "Cut generated commit subjects at a word boundary and not in the middle of a word, \
                    so that the log reads cleanly\nsecond line";
        let title = reading_title(text);
        assert!(title.chars().count() <= 72, "{title:?}");
        assert!(text.starts_with(&title));
        assert!(
            text[title.len()..].starts_with(' '),
            "ends mid-word: {title:?}"
        );
    }

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

    fn outcome(
        job: &JobRecords,
        state: OutcomeState,
        reason: &str,
    ) -> athena_data::records::Outcome {
        athena_data::records::Outcome {
            job: job.id().clone(),
            state,
            reason: reason.into(),
            at: jiff::Timestamp::UNIX_EPOCH,
            totals: None,
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        }
    }

    fn records_with(job: JobRecords) -> Records {
        let mut records = Records::default();
        records.jobs.insert(job.id().as_str().to_string(), job);
        records
    }

    fn kept_plan(job: &JobRecords, round: u32) -> athena_data::records::JobPlan {
        athena_data::records::JobPlan {
            job: job.id().clone(),
            round,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: 1,
            approach: format!("KEPT-APPROACH-{round}"),
            decisions: vec![],
            files: vec![],
            steps: vec![],
            risks: vec![],
            demonstrations: vec![],
            open_questions: vec![],
            producer_section_sha256: "0".repeat(64),
            resumed_from: None,
        }
    }

    fn kept_critique(
        job: &JobRecords,
        round: u32,
        verdict: CritiqueVerdict,
    ) -> athena_data::records::Critique {
        athena_data::records::Critique {
            job: job.id().clone(),
            round,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: Some(2),
            agent_verdict: Some(verdict),
            final_verdict: verdict,
            findings: vec![Finding {
                severity: athena_data::records::Severity::Major,
                summary: "SMALL-FIX".into(),
                evidence: "step 3".into(),
                location: None,
                source: athena_data::records::FindingSource::Critic,
            }],
            request_reading_mismatch: None,
            summary: None,
        }
    }

    /// A job closed as failed in planning with `reason` after `rounds` rounds whose last verdict is `verdict`.
    fn planning_stopped(reason: &str, verdict: CritiqueVerdict) -> (Records, String) {
        let mut job = job_with_checks(vec![]);
        job.verification = None;
        for round in 1..=3 {
            job.plans.push(kept_plan(&job, round));
            job.critiques.push(kept_critique(&job, round, verdict));
        }
        job.outcome = Some(outcome(&job, OutcomeState::Failed, reason));
        let id = job.id().as_str().to_string();
        (records_with(job), format!("try again on {id}"))
    }

    #[test]
    fn a_planning_cap_stop_gives_its_round_three_plan_and_findings() {
        // The reason is copied from the stop of job 20261026-145955-s7e2, recorded before the change.
        let old = "planning stopped before any code was written: the plan still needed revising in the last round \
                   (round 3 of 3). The plan is close. Findings: major: SMALL-FIX";
        let (records, request) = planning_stopped(old, CritiqueVerdict::Revise);
        let seed = planning_resume_seed(&records, &request).expect("a seed");
        assert_eq!(seed.plan.round, 3);
        assert_eq!(seed.plan.approach, "KEPT-APPROACH-3");
        assert_eq!(seed.findings.len(), 1);
        assert_eq!(seed.findings[0].summary, "SMALL-FIX");
        assert_eq!(
            seed.job.as_str(),
            request.trim_start_matches("try again on ")
        );
    }

    #[test]
    fn a_stop_for_any_other_reason_or_a_new_request_gives_no_seed() {
        let prefix = athena_data::records::PLANNING_STOPPED_PREFIX;
        for reason in [
            format!("{prefix} the critic rejected the plan (round 3 of 3). Findings: none"),
            format!("{prefix} the critic judged that the reading misread the request: x"),
            format!("{prefix} the planner could not run: boom"),
            "the producer reached its deadline".to_string(),
            "verification failed: a check failed".to_string(),
        ] {
            let (records, request) = planning_stopped(&reason, CritiqueVerdict::Revise);
            assert_eq!(planning_resume_seed(&records, &request), None, "{reason}");
        }
        let cap = format!(
            "{prefix} {} (round 3 of 3).",
            athena_data::records::PLANNING_CAP_REASON
        );
        let (records, request) = planning_stopped(&cap, CritiqueVerdict::Revise);
        assert!(planning_resume_seed(&records, &request).is_some());
        // Not a retry request.
        assert_eq!(planning_resume_seed(&records, "add a greeting"), None);
        // A kept critique that is not a revise.
        let (records, request) = planning_stopped(&cap, CritiqueVerdict::Reject);
        assert_eq!(planning_resume_seed(&records, &request), None);
    }

    #[test]
    fn a_failed_job_without_verification_can_be_tried_again_and_carries_its_reason() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let mut job = job_with_checks(vec![]);
        job.verification = None;
        job.outcome = Some(outcome(
            &job,
            OutcomeState::Failed,
            "admission refused the change: governance file changed",
        ));
        let id = job.id().as_str().to_string();
        let records = records_with(job);
        let attempt = earlier_attempt(&records, &format!("try again on {id}"), &redactor)
            .unwrap()
            .unwrap();
        assert!(attempt.failed);
        assert_eq!(attempt.request, "add a greeting");
        assert_eq!(
            attempt.findings,
            ["outcome: admission refused the change: governance file changed"]
        );
    }

    #[test]
    fn a_carried_outcome_reason_is_redacted() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let token = format!(
            "{}{}",
            ["gh", "p_"].concat(),
            "Zq8rT2vK9mW4xN7bP3cH6jL1sD5fG0aYuE2i"
        );
        let mut job = job_with_checks(vec![]);
        job.verification = None;
        job.outcome = Some(outcome(&job, OutcomeState::Failed, &format!("key {token}")));
        let carried = carry(&job, &redactor).unwrap().unwrap();
        assert_eq!(carried.findings.len(), 1);
        assert!(!carried.findings[0].contains(&token));
        assert!(carried.findings[0].contains("[secret:github-pat]"));
    }

    #[test]
    fn only_rejected_or_failed_jobs_can_be_tried_again() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        for state in [
            OutcomeState::Landed,
            OutcomeState::Stale,
            OutcomeState::Abandoned,
        ] {
            let mut job = job_with_checks(vec![]);
            job.outcome = Some(outcome(&job, state, "done"));
            let id = job.id().as_str().to_string();
            let error =
                earlier_attempt(&records_with(job), &format!("try again on {id}"), &redactor)
                    .unwrap_err()
                    .to_string();
            assert!(error.contains(state.as_str()), "{error}");
            assert!(error.contains("only a rejected or failed job"), "{error}");
        }
        let open = job_with_checks(vec![]);
        let id = open.id().as_str().to_string();
        let error = earlier_attempt(
            &records_with(open),
            &format!("try again on {id}"),
            &redactor,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("queued"), "{error}");
        assert!(error.contains("only a rejected or failed job"), "{error}");
    }

    fn final_reading(job: &JobRecords) -> Reading {
        let forecast = Forecast {
            p_reading_accepted: 0.8,
            p_verification_passes: 0.7,
            p_user_intervenes: 0.1,
            cost_usd: Quantiles::new(0.2, 0.5, 1.5).unwrap(),
            wall_time_s: Quantiles::new(100.0, 300.0, 900.0).unwrap(),
        };
        Reading {
            job: job.id().clone(),
            n: 1,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: 1,
            base_commit: "a".repeat(40),
            config_sha256: "b".repeat(64),
            defaults_sha256: "c".repeat(64),
            defaults_version: 1,
            title: "Add a greeting".into(),
            summary: "Print a greeting.".into(),
            class_of_work: "feature".into(),
            capability: Some("change".into()),
            decline_reason: None,
            producer_model: "producer".into(),
            verifier_model: "verifier".into(),
            defaults_applied: vec![],
            assumptions: vec![],
            question: None,
            acceptance_criteria: vec![
                Criterion {
                    text: "greeting.rs exists".into(),
                    kind: CriterionKind::Content,
                },
                Criterion {
                    text: "cargo test passes".into(),
                    kind: CriterionKind::Behavior,
                },
            ],
            deliverables: vec![Deliverable {
                path: "greeting.rs".into(),
                action: DeliverableAction::Create,
                p: 0.9,
            }],
            out_of_scope: vec![],
            forecasts: Forecasts {
                reader: forecast.clone(),
                reference: forecast,
            },
            departures: None,
            urgent_reason: None,
            hard_reason: None,
            producer_effort: None,
        }
    }

    fn change_reading() -> Reading {
        let job = job_with_checks(vec![]);
        final_reading(&job)
    }

    #[test]
    fn a_change_reading_gains_the_changelog_deliverable_and_default() {
        let mut reading = change_reading();
        ensure_changelog(&mut reading);
        let changelog: Vec<_> = reading
            .deliverables
            .iter()
            .filter(|d| d.path == "CHANGELOG.md")
            .collect();
        assert_eq!(changelog.len(), 1);
        assert_eq!(changelog[0].action, DeliverableAction::Modify);
        let defaults: Vec<_> = reading
            .defaults_applied
            .iter()
            .filter(|d| d.id == "changelog")
            .collect();
        assert_eq!(defaults.len(), 1);
        assert!(defaults[0].how.contains("BRIEF.md"));
        assert!(defaults[0].how.contains("tagged releases and a changelog"));
    }

    #[test]
    fn ensure_changelog_adds_nothing_twice() {
        let mut reading = change_reading();
        ensure_changelog(&mut reading);
        let once = reading.clone();
        ensure_changelog(&mut reading);
        assert_eq!(reading, once);

        let mut listed = change_reading();
        listed.deliverables.push(Deliverable {
            path: "CHANGELOG.md".into(),
            action: DeliverableAction::Modify,
            p: 0.5,
        });
        listed.defaults_applied.push(DefaultApplied {
            id: "changelog".into(),
            how: "the reader's own words".into(),
        });
        let before = listed.clone();
        ensure_changelog(&mut listed);
        assert_eq!(listed, before);
    }

    #[test]
    fn ensure_changelog_leaves_study_and_declined_readings_alone() {
        let mut study = change_reading();
        study.capability = Some("study".into());
        let before = study.clone();
        ensure_changelog(&mut study);
        assert_eq!(study, before);

        let mut declined = change_reading();
        declined.capability = None;
        declined.decline_reason = Some("not possible".into());
        let before = declined.clone();
        ensure_changelog(&mut declined);
        assert_eq!(declined, before);
    }

    fn correction(job: &JobRecords, text: &str, kind: CorrectionKind) -> Correction {
        Correction {
            job: job.id().clone(),
            n: 1,
            at: jiff::Timestamp::UNIX_EPOCH,
            kind,
            text: text.into(),
            intervention: RecordId::parse("20261012-090000-abcd").unwrap(),
        }
    }

    #[test]
    fn a_retry_carries_the_final_reading_and_the_corrections_redacted() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let token = format!(
            "{}{}",
            ["gh", "p_"].concat(),
            "Zq8rT2vK9mW4xN7bP3cH6jL1sD5fG0aYuE2i"
        );
        let mut job = job_with_checks(vec![]);
        job.readings = vec![final_reading(&job)];
        job.corrections = vec![
            correction(&job, "name it greeting.rs", CorrectionKind::Correction),
            correction(&job, &format!("yes, key {token}"), CorrectionKind::Answer),
        ];
        job.outcome = Some(outcome(&job, OutcomeState::Rejected, "rejected"));
        let id = job.id().as_str().to_string();
        let attempt = earlier_attempt(&records_with(job), &format!("try again on {id}"), &redactor)
            .unwrap()
            .unwrap();
        assert_eq!(
            attempt.reading_summary.as_deref(),
            Some("Print a greeting.")
        );
        assert_eq!(
            attempt.reading_criteria,
            [
                "[content] greeting.rs exists",
                "[behavior] cargo test passes"
            ]
        );
        assert_eq!(attempt.reading_deliverables, ["create greeting.rs"]);
        assert_eq!(attempt.corrections.len(), 2);
        assert_eq!(attempt.corrections[0], "name it greeting.rs");
        assert!(!attempt.corrections[1].contains(&token));
        assert!(attempt.corrections[1].contains("[secret:github-pat]"));
    }

    #[test]
    fn a_job_without_a_reading_or_corrections_carries_none() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let carried = carry(&job_with_checks(vec![]), &redactor).unwrap().unwrap();
        assert_eq!(carried.reading_summary, None);
        assert!(carried.reading_criteria.is_empty());
        assert!(carried.reading_deliverables.is_empty());
        assert!(carried.corrections.is_empty());
    }

    #[test]
    fn a_retry_of_a_retry_keeps_the_original_request_and_the_retried_jobs_reading() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let mut first = job_with_checks(vec![]);
        first.outcome = Some(outcome(&first, OutcomeState::Rejected, "rejected"));
        let first_id = first.id().as_str().to_string();
        let mut second = job_with_checks(vec![]);
        let second_id = JobId::parse("20261012-090000-b2b2").unwrap();
        second.id = Some(second_id.clone());
        second.request.as_mut().unwrap().job = second_id.clone();
        second.request.as_mut().unwrap().text = format!("try again on {first_id}");
        second.readings = vec![final_reading(&second)];
        second.corrections = vec![correction(
            &second,
            "keep it short",
            CorrectionKind::Correction,
        )];
        second.outcome = Some(outcome(&second, OutcomeState::Rejected, "rejected"));
        let mut records = records_with(first);
        records.jobs.insert(second_id.as_str().to_string(), second);
        let attempt = earlier_attempt(
            &records,
            &format!("try again on {}", second_id.as_str()),
            &redactor,
        )
        .unwrap()
        .unwrap();
        assert_eq!(attempt.job, second_id.as_str());
        assert_eq!(attempt.request, "add a greeting");
        assert_eq!(
            attempt.reading_summary.as_deref(),
            Some("Print a greeting.")
        );
        assert_eq!(attempt.corrections, ["keep it short"]);
    }

    #[test]
    fn a_failed_job_carries_its_corrections_in_order_and_redacted() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let token = format!(
            "{}{}",
            ["gh", "p_"].concat(),
            "Zq8rT2vK9mW4xN7bP3cH6jL1sD5fG0aYuE2i"
        );
        let mut job = job_with_checks(vec![]);
        job.readings = vec![final_reading(&job)];
        job.corrections = vec![
            correction(&job, "merge only when opted in", CorrectionKind::Correction),
            correction(&job, &format!("key {token}"), CorrectionKind::Answer),
        ];
        job.outcome = Some(outcome(&job, OutcomeState::Failed, "failed"));
        let id = job.id().as_str().to_string();
        let attempt = earlier_attempt(&records_with(job), &format!("try again on {id}"), &redactor)
            .unwrap()
            .unwrap();
        assert!(attempt.failed);
        assert_eq!(
            attempt.reading_summary.as_deref(),
            Some("Print a greeting.")
        );
        assert_eq!(attempt.corrections.len(), 2);
        assert_eq!(attempt.corrections[0], "merge only when opted in");
        assert!(!attempt.corrections[1].contains(&token));
        assert!(attempt.corrections[1].contains("[secret:github-pat]"));
    }

    #[test]
    fn a_failed_job_without_a_verification_carries_its_corrections() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let mut job = job_with_checks(vec![]);
        job.verification = None;
        job.readings = vec![final_reading(&job)];
        job.corrections = vec![correction(
            &job,
            "keep it short",
            CorrectionKind::Correction,
        )];
        job.outcome = Some(outcome(&job, OutcomeState::Failed, "admission refused"));
        let id = job.id().as_str().to_string();
        let attempt = earlier_attempt(&records_with(job), &format!("try again on {id}"), &redactor)
            .unwrap()
            .unwrap();
        assert_eq!(attempt.corrections, ["keep it short"]);
        assert_eq!(attempt.findings, ["outcome: admission refused"]);
    }

    #[test]
    fn a_failed_retry_carries_the_corrections_of_the_jobs_it_retried() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let mut first = job_with_checks(vec![]);
        first.readings = vec![final_reading(&first)];
        first.corrections = vec![correction(
            &first,
            "merge only when opted in",
            CorrectionKind::Correction,
        )];
        first.outcome = Some(outcome(&first, OutcomeState::Failed, "failed"));
        let first_id = first.id().as_str().to_string();
        let mut second = job_with_checks(vec![]);
        let second_id = JobId::parse("20261012-090000-b2b2").unwrap();
        second.id = Some(second_id.clone());
        second.request.as_mut().unwrap().job = second_id.clone();
        second.request.as_mut().unwrap().text = format!("try again on {first_id}");
        second.readings = vec![final_reading(&second)];
        second.corrections = vec![correction(&second, "keep it short", CorrectionKind::Answer)];
        second.outcome = Some(outcome(&second, OutcomeState::Failed, "failed"));
        let mut records = records_with(first);
        records.jobs.insert(second_id.as_str().to_string(), second);
        let attempt = earlier_attempt(
            &records,
            &format!("try again on {}", second_id.as_str()),
            &redactor,
        )
        .unwrap()
        .unwrap();
        assert_eq!(attempt.request, "add a greeting");
        assert_eq!(
            attempt.corrections,
            ["merge only when opted in", "keep it short"]
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
    fn an_unread_request_is_planned_from_its_own_words_and_a_read_one_from_its_reading() {
        use athena_data::roles::Ladder;
        let mut job = job_with_checks(vec![]);
        job.request.as_mut().unwrap().text =
            "Add a greeting to crates/cli/src/main.rs after 20261012-075824-1x28\n- it prints hello\n- cargo test passes"
                .into();
        let everything = Lookup {
            job_exists: &|_| true,
            path_exists: &|_| true,
            brief: &[],
        };
        let entry = plan_entry(&job, None, &everything);
        assert_eq!(
            entry.goal,
            job.request.as_ref().unwrap().text.lines().next().unwrap()
        );
        assert_eq!(
            entry.acceptance,
            ["- it prints hello", "- cargo test passes"]
        );
        assert_eq!(entry.touches, ["crates/cli/src/main.rs"]);
        assert_eq!(entry.depends_on, ["20261012-075824-1x28"]);
        assert_eq!(
            entry.rung,
            Some(Ladder::Implementation),
            "inferred from the path"
        );
        assert!(entry.missing().is_empty());

        job.readings = vec![final_reading(&job)];
        let entry = plan_entry(&job, Some(Ladder::DesignDecisions), &everything);
        assert_eq!(entry.goal, "Print a greeting.");
        assert_eq!(
            entry.acceptance,
            ["greeting.rs exists", "cargo test passes"]
        );
        assert_eq!(entry.touches, ["greeting.rs"]);
        assert_eq!(entry.rung, Some(Ladder::DesignDecisions), "the hint wins");

        job.request.as_mut().unwrap().text = "  ".into();
        job.readings.clear();
        assert_eq!(
            plan_entry(&job, None, &everything).missing(),
            ["goal", "ladder rung", "acceptance"],
            "an empty request has nothing to plan"
        );
    }

    fn nothing_known() -> Lookup<'static> {
        Lookup {
            job_exists: &|_| false,
            path_exists: &|_| false,
            brief: &[],
        }
    }

    /// Lookups that know the real BRIEF.md, as `athena plan` does.
    fn real_brief() -> Vec<BriefTopic> {
        brief_topics(include_str!("../../../BRIEF.md"))
    }

    fn rung_with_brief(text: &str, brief: &[BriefTopic]) -> Option<athena_data::roles::Ladder> {
        let mut job = job_with_checks(vec![]);
        job.request.as_mut().unwrap().text = text.into();
        let lookup = Lookup {
            job_exists: &|_| false,
            path_exists: &|_| false,
            brief,
        };
        plan_entry(&job, None, &lookup).rung
    }

    #[test]
    fn a_request_serving_the_speed_addendum_ranks_above_implementation() {
        use athena_data::roles::Ladder;
        let brief = real_brief();
        assert!(
            brief.iter().any(|t| t.heading
                == "2026-10-16: Speed, planning first, and initiatives in parallel"
                && t.text.contains("intelligent routing")),
            "the speed addendum is read from BRIEF.md"
        );
        for text in [
            "Move compute to CI with intelligent routing of jobs",
            "Run parallel jobs for initiatives",
        ] {
            let rung = rung_with_brief(text, &brief);
            assert_eq!(rung, Some(Ladder::BriefRequirements), "{text}");
            // The derived order puts the top rung first, so a smaller rung is a higher one.
            assert!(rung < Some(Ladder::Implementation), "{text}");
        }
        // Even with a touched path, serving the brief wins over plain implementation.
        assert_eq!(
            rung_with_brief(
                "Add intelligent routing of CI jobs in .github/workflows/ci.yml",
                &brief
            ),
            Some(Ladder::BriefRequirements)
        );
    }

    #[test]
    fn a_plan_orders_a_speed_request_before_an_older_typo_fix() {
        let brief = real_brief();
        let entry = |id: &str, text: &str| {
            let mut job = job_with_checks(vec![]);
            job.request.as_mut().unwrap().text = format!("{text}\n- it is done");
            let lookup = Lookup {
                job_exists: &|_| false,
                path_exists: &|_| true,
                brief: &brief,
            };
            let mut e = plan_entry(&job, None, &lookup);
            e.id = id.into();
            e
        };
        let old_typo = entry("old", "Fix a typo in README.md");
        let new_speed = entry("new", "Route CI jobs with intelligent routing");
        assert_eq!(
            old_typo.rung,
            Some(athena_data::roles::Ladder::Implementation)
        );
        // Oldest first in the input: without the brief the old item would lead.
        let plan = crate::plan::Plan::build(&[old_typo, new_speed]);
        assert_eq!(plan.order, ["new", "old"], "{plan:?}");
    }

    #[test]
    fn a_word_like_fix_or_bug_alone_is_no_rung() {
        let brief = real_brief();
        assert_eq!(rung_with_brief("fix a bug", &brief), None);
        assert_eq!(rung_with_brief("fix the typo in the intro", &brief), None);
        assert_eq!(rung_with_brief("update the docs and tests", &brief), None);
    }

    #[test]
    fn a_vague_request_after_reading_is_flagged_and_names_what_is_missing() {
        let mut job = job_with_checks(vec![]);
        job.request.as_mut().unwrap().text = "make it better".into();
        // The reader read it and declined: no capability, no criteria.
        let mut reading = final_reading(&job);
        reading.capability = None;
        reading.decline_reason = Some("it does not say what to improve".into());
        reading.acceptance_criteria.clear();
        reading.deliverables.clear();
        reading.summary = "Make something better.".into();
        job.readings = vec![reading];
        let entry = plan_entry(&job, None, &nothing_known());
        assert_eq!(entry.missing(), ["ladder rung", "acceptance"]);
        let plan = crate::plan::Plan::build(&[entry]);
        assert!(plan.order.is_empty(), "{plan:?}");
        assert_eq!(plan.refused[0].missing, ["ladder rung", "acceptance"]);
    }

    #[test]
    fn a_priority_line_in_the_request_sets_the_plan_priority() {
        use crate::plan::Priority;
        let mut job = job_with_checks(vec![]);
        job.request.as_mut().unwrap().text = "Fix the typo in README.md".into();
        assert_eq!(
            plan_entry(&job, None, &nothing_known()).priority,
            Priority::Normal
        );
        job.request.as_mut().unwrap().text = "Fix the typo in README.md\nPriority: HIGH".into();
        assert_eq!(
            plan_entry(&job, None, &nothing_known()).priority,
            Priority::High
        );
    }

    #[test]
    fn the_first_line_is_not_copied_into_acceptance() {
        let mut job = job_with_checks(vec![]);
        job.request.as_mut().unwrap().text = "Fix the typo in README.md".into();
        let entry = plan_entry(&job, None, &nothing_known());
        assert!(entry.acceptance.is_empty(), "{:?}", entry.acceptance);
        assert_eq!(entry.missing(), ["acceptance"]);
        // Explicit bullets fill it, and a reading's criteria fill it once read.
        job.request.as_mut().unwrap().text = "Fix the typo in README.md\n* it reads 'the'".into();
        assert_eq!(
            plan_entry(&job, None, &nothing_known()).acceptance,
            ["* it reads 'the'"]
        );
        job.readings = vec![final_reading(&job)];
        assert_eq!(
            plan_entry(&job, None, &nothing_known()).acceptance,
            ["greeting.rs exists", "cargo test passes"]
        );
    }

    #[test]
    fn a_rung_is_inferred_from_evidence_or_flagged_never_defaulted_to_implementation() {
        use athena_data::roles::Ladder;
        let rung_of = |text: &str| {
            let mut job = job_with_checks(vec![]);
            job.request.as_mut().unwrap().text = text.into();
            plan_entry(&job, None, &nothing_known()).rung
        };
        assert_eq!(rung_of("make it better"), None);
        assert_eq!(rung_of("tidy things up a bit"), None);
        // A word such as "fix" or "typo" is not evidence by itself; a touched path is.
        assert_eq!(rung_of("fix the typo in the intro"), None);
        assert_eq!(rung_of("fix a bug"), None);
        assert_eq!(
            rung_of("fix the typo in README.md"),
            Some(Ladder::Implementation)
        );
        assert_eq!(
            rung_of("record a decision about retries"),
            Some(Ladder::DesignDecisions)
        );
        assert_eq!(
            rung_of("amend the brief to require a changelog"),
            Some(Ladder::BriefRequirements)
        );
        // A reading for the `change` capability is evidence too.
        let mut job = job_with_checks(vec![]);
        job.request.as_mut().unwrap().text = "make it better".into();
        let mut reading = final_reading(&job);
        reading.deliverables.clear();
        job.readings = vec![reading];
        assert_eq!(
            plan_entry(&job, None, &nothing_known()).rung,
            Some(Ladder::Implementation)
        );
        // No hint and no evidence: the plan flags the rung, so the item is refused.
        let mut job = job_with_checks(vec![]);
        job.request.as_mut().unwrap().text = "make it better\n- it is better".into();
        let entry = plan_entry(&job, None, &nothing_known());
        assert_eq!(entry.missing(), ["ladder rung"]);
        assert!(crate::plan::Plan::build(&[entry]).order.is_empty());
    }

    #[test]
    fn open_questions_name_a_missing_path_and_an_unresolved_job_for_read_and_unread_requests() {
        let mut job = job_with_checks(vec![]);
        job.request.as_mut().unwrap().text =
            "Update crates/gone/src/x.rs and docs/here.md after 20261012-075824-1x28\n- it works"
                .into();
        let lookup = Lookup {
            job_exists: &|_| false,
            path_exists: &|p| p == "docs/here.md",
            brief: &[],
        };
        let unread = plan_entry(&job, None, &lookup);
        assert_eq!(
            unread.open_questions,
            [
                "the touched path crates/gone/src/x.rs does not exist at the base commit",
                "the referenced job 20261012-075824-1x28 is not among the records"
            ]
        );
        // Read: a deliverable that is created may be absent; one that is modified may not.
        let mut reading = final_reading(&job);
        reading.deliverables = vec![
            Deliverable {
                path: "new.rs".into(),
                action: DeliverableAction::Create,
                p: 0.9,
            },
            Deliverable {
                path: "old.rs".into(),
                action: DeliverableAction::Modify,
                p: 0.9,
            },
        ];
        reading.question = Some("Which greeting?".into());
        job.readings = vec![reading];
        let read = plan_entry(&job, None, &lookup);
        assert_eq!(
            read.open_questions,
            [
                "Which greeting?",
                "the touched path old.rs does not exist at the base commit",
                "the referenced job 20261012-075824-1x28 is not among the records"
            ]
        );
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

    fn criterion_result(
        kind: CriterionKind,
        met: bool,
        commands_verified: bool,
    ) -> athena_data::records::CriterionResult {
        athena_data::records::CriterionResult {
            criterion: "a criterion".into(),
            kind,
            met,
            evidence: "UNMET-EVIDENCE the description lists no callers".into(),
            commands: vec![],
            commands_verified,
        }
    }

    fn candidate_of(job: &JobRecords) -> athena_data::records::Candidate {
        athena_data::records::Candidate {
            job: job.id().clone(),
            base_commit: "a".repeat(40),
            base_fingerprint: None,
            previous_job: None,
            candidate_commit: Some("b".repeat(40)),
            candidate_tree: Some("c".repeat(40)),
            changes: vec![],
            fingerprint: None,
            violations: vec![],
            producer_report: None,
        }
    }

    /// A rejected job whose checks passed and whose only unmet criterion is of kind content.
    fn content_only_rejection() -> JobRecords {
        let mut job = job_with_checks(vec![check("fmt", true, ""), check("test", true, "")]);
        job.candidate = Some(candidate_of(&job));
        job.outcome = Some(outcome(&job, OutcomeState::Rejected, "rejected"));
        let verification = job.verification.as_mut().unwrap();
        verification.criteria = vec![
            criterion_result(CriterionKind::Content, false, true),
            criterion_result(CriterionKind::Behavior, true, true),
        ];
        verification.findings = vec![athena_data::records::Finding {
            severity: athena_data::records::Severity::Major,
            summary: "no callers listed".into(),
            evidence: "the entry is silent".into(),
            location: Some("CHANGELOG.md".into()),
            source: athena_data::records::FindingSource::Verifier,
        }];
        job
    }

    fn qualifies(job: &JobRecords) -> bool {
        qualifies_for_tree_retry(
            job.state(),
            job.candidate.as_ref(),
            job.verification.as_ref(),
        )
    }

    #[test]
    fn only_a_rejection_with_every_check_passed_and_content_criteria_unmet_qualifies() {
        assert!(qualifies(&content_only_rejection()));

        type Change = fn(&mut JobRecords);
        let cases: [(&str, Change); 11] = [
            ("a check failed", |j| {
                j.verification.as_mut().unwrap().checks[1].passed = false
            }),
            ("a behavior criterion is unmet", |j| {
                j.verification.as_mut().unwrap().criteria[1].met = false
            }),
            ("a behavior criterion cites commands never seen", |j| {
                j.verification.as_mut().unwrap().criteria[1].commands_verified = false
            }),
            ("no candidate tree", |j| {
                j.candidate.as_mut().unwrap().candidate_tree = None
            }),
            ("no candidate commit", |j| {
                j.candidate.as_mut().unwrap().candidate_commit = None
            }),
            ("no candidate record", |j| j.candidate = None),
            ("admission violations", |j| {
                j.candidate.as_mut().unwrap().violations =
                    vec![athena_data::records::PolicyViolation {
                        rule: "governance".into(),
                        path: None,
                        detail: "x".into(),
                    }]
            }),
            ("the job failed", |j| {
                j.outcome.as_mut().unwrap().state = OutcomeState::Failed
            }),
            ("the job was interrupted", |j| j.outcome = None),
            // The fixture's finding sits in CHANGELOG.md and would now qualify, so clear it: with no
            // unmet criterion and no finding the cause is unknown.
            ("no criterion is unmet and no finding", |j| {
                let v = j.verification.as_mut().unwrap();
                v.criteria[0].met = true;
                v.findings.clear();
            }),
            ("no check was recorded", |j| {
                j.verification.as_mut().unwrap().checks.clear()
            }),
        ];
        for (name, change) in cases {
            let mut job = content_only_rejection();
            // An interrupted job is dispatched and has no outcome.
            if name == "the job was interrupted" {
                job.job = Some(athena_data::records::Job {
                    id: job.id().clone(),
                    ..dispatched_job_stub()
                });
            }
            change(&mut job);
            assert!(!qualifies(&job), "{name}: must restart from the base");
        }

        let mut mismatch = content_only_rejection();
        mismatch
            .verification
            .as_mut()
            .unwrap()
            .request_reading_mismatch = Some("misread".into());
        assert!(
            !qualifies(&mismatch),
            "a reading mismatch restarts from the base"
        );
    }

    fn finding(
        severity: athena_data::records::Severity,
        location: Option<&str>,
    ) -> athena_data::records::Finding {
        athena_data::records::Finding {
            severity,
            summary: "decision D122 is inserted inside D121".into(),
            evidence: "the entry splits D121".into(),
            location: location.map(String::from),
            source: athena_data::records::FindingSource::Verifier,
        }
    }

    /// A rejected job whose checks all passed, with no unmet criterion and one blocking docs finding.
    fn docs_only_rejection() -> JobRecords {
        use athena_data::records::Severity;
        let mut job = content_only_rejection();
        let verification = job.verification.as_mut().unwrap();
        verification.criteria = vec![criterion_result(CriterionKind::Behavior, true, true)];
        verification.findings = vec![finding(
            Severity::Major,
            Some("docs/decisions.md:3005-3053"),
        )];
        job
    }

    #[test]
    fn docs_only_blocking_finding_retries_from_the_candidate_and_anything_else_from_the_base() {
        use athena_data::records::Severity;
        assert!(qualifies(&docs_only_rejection()));

        // A minor code finding does not block; a minor finding beside a docs major still qualifies.
        let mut job = docs_only_rejection();
        job.verification
            .as_mut()
            .unwrap()
            .findings
            .push(finding(Severity::Minor, Some("src/lib.rs:1")));
        assert!(qualifies(&job), "minor findings are ignored");

        // Only minor findings: nothing blocks, so the cause is unknown.
        let mut job = docs_only_rejection();
        job.verification.as_mut().unwrap().findings =
            vec![finding(Severity::Minor, Some("docs/a.md"))];
        assert!(!qualifies(&job), "no blocking finding");

        type Change = fn(&mut JobRecords);
        let cases: [(&str, Change); 7] = [
            ("a major finding cites a .rs file", |j| {
                j.verification.as_mut().unwrap().findings = vec![finding(
                    Severity::Major,
                    Some("crates/athena-core/src/run.rs:10"),
                )]
            }),
            ("one finding cites a docs path and a code path", |j| {
                j.verification.as_mut().unwrap().findings = vec![finding(
                    Severity::Major,
                    Some("docs/a.md crates/x/src/lib.rs"),
                )]
            }),
            ("a docs major beside a code blocker", |j| {
                j.verification
                    .as_mut()
                    .unwrap()
                    .findings
                    .push(finding(Severity::Blocker, Some("Cargo.toml")))
            }),
            ("a check failed", |j| {
                j.verification.as_mut().unwrap().checks[0].passed = false
            }),
            ("a behavior criterion is unmet", |j| {
                j.verification.as_mut().unwrap().criteria[0].met = false
            }),
            ("the verifier reported a mismatch", |j| {
                j.verification.as_mut().unwrap().request_reading_mismatch = Some("misread".into())
            }),
            ("the candidate has admission violations", |j| {
                j.candidate.as_mut().unwrap().violations =
                    vec![athena_data::records::PolicyViolation {
                        rule: "governance".into(),
                        path: None,
                        detail: "x".into(),
                    }]
            }),
        ];
        for (name, change) in cases {
            let mut job = docs_only_rejection();
            change(&mut job);
            assert!(!qualifies(&job), "{name}: must restart from the base");
        }

        let mut job = docs_only_rejection();
        job.verification.as_mut().unwrap().findings.clear();
        assert!(!qualifies(&job), "no finding and no unmet criterion");
    }

    #[test]
    fn tree_retry_seed_docs_only_carries_the_docs_finding() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let job = docs_only_rejection();
        let id = job.id().as_str().to_string();
        let records = records_with(job);
        let seed = tree_retry_seed(&records, &format!("try again on {id}"), &redactor)
            .unwrap()
            .expect("a seed");
        assert!(seed.unmet.is_empty());
        assert_eq!(seed.findings.len(), 1);
        assert!(
            seed.findings[0].contains(
                "major (docs/decisions.md:3005-3053): decision D122 is inserted inside D121"
            ),
            "{:?}",
            seed.findings
        );
    }

    /// Two major findings (at docs paths so the job qualifies for a tree retry) and a minor one.
    fn two_majors_and_a_minor() -> JobRecords {
        use athena_data::records::Severity;
        let mut job = docs_only_rejection();
        let mut first = finding(Severity::Major, Some("docs/a.md:1"));
        first.summary = "FIRST-SUMMARY".into();
        first.evidence = "FIRST-EVIDENCE".into();
        let mut second = finding(Severity::Major, Some("docs/b.md:2"));
        second.summary = "SECOND-SUMMARY".into();
        second.evidence = "SECOND-EVIDENCE".into();
        let mut minor = finding(Severity::Minor, Some("docs/c.md"));
        minor.summary = "MINOR-SUMMARY".into();
        job.verification.as_mut().unwrap().findings = vec![first, minor, second];
        job
    }

    #[test]
    fn both_retry_paths_carry_every_major_finding_in_the_same_form() {
        use athena_context::briefs::{
            AcceptedReading, ProducerInputs, RetryFindings, RetrySeed, producer_brief,
        };
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let job = two_majors_and_a_minor();
        let id = job.id().clone();
        let records = records_with(job);
        let request = format!("try again on {id}");

        let seed = tree_retry_seed(&records, &request, &redactor)
            .unwrap()
            .expect("a seed");
        let (retried, unseeded) = retry_findings(&records, &request, &redactor)
            .unwrap()
            .expect("findings");
        assert_eq!(retried, id);
        assert_eq!(seed.must_address, unseeded);
        assert_eq!(unseeded.len(), 2, "{unseeded:?}");
        for (text, parts) in unseeded.iter().zip([
            ["major (docs/a.md:1)", "FIRST-SUMMARY", "FIRST-EVIDENCE"],
            ["major (docs/b.md:2)", "SECOND-SUMMARY", "SECOND-EVIDENCE"],
        ]) {
            for part in parts {
                assert!(text.contains(part), "{part} missing from {text}");
            }
        }
        assert!(seed.findings.iter().any(|f| f.contains("MINOR-SUMMARY")));

        let brief_of = |seeded: bool| {
            let mut inputs = ProducerInputs {
                job: "20261026-000000-test".into(),
                request: request.clone(),
                reading: AcceptedReading {
                    n: 1,
                    title: "t".into(),
                    summary: "s".into(),
                    assumptions: vec![],
                    criteria: vec![],
                    deliverables: vec![],
                    out_of_scope: vec![],
                    defaults: vec![],
                },
                checks: vec![],
                commands: vec![],
                pin: None,
                retry_from: None,
                retry_findings: None,
                failure_classes: Vec::new(),
            };
            if seeded {
                inputs.retry_from = Some(RetrySeed {
                    job: id.as_str().to_string(),
                    unmet: seed.unmet.clone(),
                    findings: seed.findings.clone(),
                    must_address: seed.must_address.clone(),
                    progress_note: None,
                });
            } else {
                inputs.retry_findings = Some(RetryFindings {
                    job: id.as_str().to_string(),
                    findings: unseeded.clone(),
                });
            }
            producer_brief(&inputs)
        };
        for seeded in [true, false] {
            let brief = brief_of(seeded);
            let start = brief.find("## Findings you must address").expect("a block");
            let rest = &brief[start..];
            let end = ["\n\n## ", "\n\nAll of the"]
                .iter()
                .filter_map(|stop| rest.find(stop))
                .min()
                .unwrap_or(rest.len());
            let block = &rest[..end];
            for text in &unseeded {
                assert!(
                    block.contains(&format!("> {text}")),
                    "seeded={seeded}: {block}"
                );
            }
            assert!(!block.contains("MINOR-SUMMARY"), "seeded={seeded}: {block}");
        }

        // A job without a blocking finding has nothing to carry.
        let mut quiet = two_majors_and_a_minor();
        quiet.verification.as_mut().unwrap().findings =
            vec![finding(athena_data::records::Severity::Minor, None)];
        let id = quiet.id().clone();
        let records = records_with(quiet);
        assert_eq!(
            retry_findings(&records, &format!("try again on {id}"), &redactor).unwrap(),
            None
        );
    }

    fn dispatched_job_stub() -> athena_data::records::Job {
        serde_json::from_value(serde_json::json!({
            "id": "20261012-084936-n1p0", "kind": "request", "title": "t", "class_of_work": null,
            "capability": null, "accepted_reading": null, "accepted_by": null,
            "owner": { "role": "producer", "capability": null, "model": null, "note": null },
            "verifier_model": null, "caps": null, "base_commit": null, "config_sha256": null,
            "defaults_sha256": null, "athena": null, "driver": null, "brief_sha256": null,
            "dispatched_at": null, "request_path": null, "request_sha256": null, "cost": null,
            "tokens": null, "note": null, "pin": null
        }))
        .unwrap()
    }

    #[test]
    fn the_seed_is_found_from_the_request_and_carries_redacted_findings() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let job = content_only_rejection();
        let id = job.id().as_str().to_string();
        let records = records_with(job);
        let seed = tree_retry_seed(&records, &format!("try again on {id}"), &redactor)
            .unwrap()
            .expect("a seed");
        assert_eq!(seed.job.as_str(), id);
        assert_eq!(seed.candidate_commit, "b".repeat(40));
        assert_eq!(seed.base_commit, "a".repeat(40));
        assert_eq!(seed.unmet.len(), 1);
        assert!(seed.unmet[0].contains("UNMET-EVIDENCE"), "{:?}", seed.unmet);
        assert_eq!(seed.findings.len(), 1);
        assert!(seed.findings[0].contains("major (CHANGELOG.md): no callers listed"));
        // Any other request, or a job that does not qualify, has no seed.
        assert_eq!(
            tree_retry_seed(&records, "add a greeting", &redactor).unwrap(),
            None
        );
        let mut failed = content_only_rejection();
        failed.verification.as_mut().unwrap().checks[0].passed = false;
        let records = records_with(failed);
        assert_eq!(
            tree_retry_seed(&records, &format!("try again on {id}"), &redactor).unwrap(),
            None
        );
    }
}
