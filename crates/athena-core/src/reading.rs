//! `ask` and `correct`: the reader's reading, with forecasts, committed before any work (design §3.2–3.3).

use crate::admission::{governance_change_is_listed, is_governance};
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
    Reading, Replay, ReplayRefusal, Request, Role,
};
use athena_data::{JobId, RecordId};
use athena_gate::redact::Redactor;
use athena_git::snapshot::{Snapshot, SnapshotRepo};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
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

/// Why one run of the reader made no reading.
enum ReadError {
    /// The reader did not complete or returned nothing. Another run would not repair it.
    Failed(String),
    /// The reader returned a reading that a reading rule refused. The text says what to fix.
    Refused(String),
}

impl ReadError {
    fn into_text(self) -> String {
        match self {
            ReadError::Failed(text) | ReadError::Refused(text) => text,
        }
    }
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

/// The token headroom of every prompt set, for the reader's brief: the reader's own defaults and catalog
/// and the enabled capabilities' sections.
fn reader_headroom(athena: &Athena, jc: &JobConfig) -> Vec<athena_context::HeadroomRow> {
    let enabled: Vec<_> = athena
        .capabilities
        .iter()
        .filter(|c| jc.config.capability(c.id()).is_some())
        .collect();
    let cards: Vec<_> = enabled.iter().map(|c| c.card()).collect();
    let sections: Vec<_> = enabled
        .iter()
        .map(|c| {
            (
                c.id().to_string(),
                c.producer_section(),
                c.verifier_section(),
            )
        })
        .collect();
    athena_context::headroom_rows(
        Component::new("defaults", jc.defaults.version, jc.defaults.render()),
        Component::new("catalog", 1, render_catalog(&cards)),
        &sections,
    )
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

/// Refuses a reading whose own forecast median wall time is beyond the producer or verifier deadline.
///
/// Root cause: reading validation never compared the reading's scope with the producer and verifier deadlines
/// (`budget.producer_timeout_s`, `budget.verifier_timeout_s`), so a source with many items produced a reading
/// that no single run could finish. The scope shows up as the reader's own `wall_time_s.p50`, so the fix sits
/// in `build_reading`, the one validation step that holds both the forecast and the configured timeouts.
///
/// The rule is strict: a p50 equal to the smaller timeout is allowed. The error names the smaller of the two
/// timeouts and tells the reader to narrow the scope; `read_with_correction` feeds it back once.
fn check_reading_size(
    forecast: &Forecast,
    producer_timeout_s: u64,
    verifier_timeout_s: u64,
) -> Result<()> {
    let (name, limit) = if verifier_timeout_s < producer_timeout_s {
        ("verifier_timeout_s", verifier_timeout_s)
    } else {
        ("producer_timeout_s", producer_timeout_s)
    };
    let p50 = forecast.wall_time_s.p50();
    if p50 > limit as f64 {
        bail!(
            "the reading forecasts a median wall time of {p50:.0}s, beyond the configured {name} of {limit}s, \
             so no single run can finish it: pick one deliverable and name the rest as follow-up requests in \
             out_of_scope, then forecast the smaller scope"
        );
    }
    Ok(())
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

    /// Runs the reader and builds the reading. When a reading rule refuses the reading (it is invalid, or
    /// acceptance refuses it), the reader runs once more at once with the refusal text as a correction in its
    /// brief; only a second refusal is the failure returned. Every invocation made is returned, in order, so
    /// the caller records each and the job's spend is right; the last one made the returned reading or error.
    ///
    /// A reader that did not run, did not finish or returned nothing is not a refusal and is not read again.
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
    ) -> Result<(Option<Reading>, Vec<Invocation>, Option<String>)> {
        let (reading, first, error) = self.read_once(
            jc, records, job, request, attempt, history, n, seq, spent, None, redactor, ui,
        )?;
        let Some(ReadError::Refused(refusal)) = error else {
            return Ok((reading, vec![first], error.map(ReadError::into_text)));
        };
        ui.line(&format!(
            "reading    {} · the reading was refused; reading again once with the refusal as a correction",
            job.as_str()
        ));
        let spent_after = spent + first.cost.best_lower();
        match self.read_once(
            jc,
            records,
            job,
            request,
            attempt,
            history,
            n,
            seq + 1,
            spent_after,
            Some(&refusal),
            redactor,
            ui,
        ) {
            Ok((reading, second, error)) => Ok((
                reading,
                vec![first, second],
                error.map(ReadError::into_text),
            )),
            // The first invocation was made and must be recorded, so a failure to start the second is the
            // failure of the read, not an error that drops the first record.
            Err(error) => Ok((
                None,
                vec![first],
                Some(format!(
                    "{refusal}; the reader could not be run again with this refusal: {error:#}"
                )),
            )),
        }
    }

    /// One run of the reader: the invocation, and the reading it made or why it made none. `refusal` is the
    /// text of the refusal of the reading made just before, given to the reader as a correction.
    #[allow(clippy::too_many_arguments)]
    fn read_once(
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
        refusal: Option<&str>,
        redactor: &Redactor,
        ui: &dyn Ui,
    ) -> Result<(Option<Reading>, Invocation, Option<ReadError>)> {
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
            referenced_jobs: referenced_jobs.clone(),
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
            headroom: reader_headroom(self, jc),
            refusal: refusal.map(str::to_string),
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
            return Ok((None, record, Some(ReadError::Failed(why))));
        }
        let Some(structured) = output.structured.clone() else {
            return Ok((
                None,
                record,
                Some(ReadError::Failed(
                    "the reader returned no structured reading".into(),
                )),
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
            Ok(reading) => {
                // A retry or follow-up must reflect the previous result and state the change scope. Only the
                // retried job and the jobs the request says it follows up count; jobs it cites as evidence
                // (such as the introspections in a root-cause request) are shown to the reader, not checked.
                // Its deliverables are also compared with the files at the base commit.
                let repo_files: BTreeSet<String> = self
                    .git
                    .ls_tree(&jc.commit)?
                    .into_iter()
                    .map(|entry| entry.path)
                    .collect();
                match check_reading_acceptance(
                    &reading,
                    &request.text,
                    attempt,
                    &referenced_jobs,
                    Some(&repo_files),
                ) {
                    Ok(()) => Ok((Some(reading), record, None)),
                    Err(e) => Ok((None, record, Some(ReadError::Refused(format!("{e:#}"))))),
                }
            }
            Err(e) => Ok((
                None,
                record,
                Some(ReadError::Refused(format!(
                    "the reader's output was invalid: {e:#}"
                ))),
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
        // A decline starts no run, so only a reading that names a capability has a deadline to meet.
        if out.capability.is_some() {
            check_reading_size(
                &reader,
                config.budget.producer_timeout_s,
                config.budget.verifier_timeout_s,
            )?;
        }
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
        refuse_banned_input(&self.git, &head, text)?;
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
        refuse_banned_input(&self.git, &head, text)?;
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
        let (reading, invocations, error) = self.read_for(
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
        files.extend(invocation_files(&invocations));
        let spent_usd = total_cost(&invocations);
        let invocation = invocations.last().cloned().expect("a read makes one");
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
            spent_usd,
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
        // A job that stopped for the environment, or was interrupted, after its candidate was built resumes
        // from that candidate: `go` runs no planner or producer. The reader still runs below.
        let request = format!("{RETRY_PREFIX}{id}");
        if resumable_job(&records, &request, &|j| self.running(j)).is_some()
            && let Some(mark) = self.state.read_resume(&id)
        {
            ui.line(&format!(
                "resume     job {id} kept its candidate (stage: {}); `athena go` resumes from it with no new planner or producer run",
                match mark.stage {
                    crate::state::ResumeStage::Checks => "checks",
                    crate::state::ResumeStage::Verifier => "verifier",
                }
            ));
        }
        let proposal = self.ask(&request, ui)?;
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
        let (reading, invocations, error) = self.read_for(
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
        files.extend(invocation_files(&invocations));
        let invocation = invocations.last().cloned().expect("a read makes one");
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
            spent_usd: spent + total_cost(&invocations),
            invocation: Some(invocation),
            commit,
        })
    }
}

/// The record files of the invocations one read made, in order.
pub(crate) fn invocation_files(invocations: &[Invocation]) -> Vec<NewFile> {
    invocations
        .iter()
        .map(|i| NewFile::record(&AnyRecord::Invocation(Box::new(i.clone()))))
        .collect()
}

/// What the invocations of one read cost.
pub(crate) fn total_cost(invocations: &[Invocation]) -> f64 {
    invocations.iter().map(|i| i.cost.best_lower()).sum()
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
pub(crate) fn retry_introspection<'a>(
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

/// The job a request of the form "try again on <job>" may resume from its kept candidate: one that failed for
/// an environment cause, or was interrupted (dispatched, and `running` says no process works on it). See
/// [`crate::disk::resumable_stop`]. Whether a candidate was kept is for the caller to find out.
pub fn resumable_job(
    records: &Records,
    request: &str,
    running: &dyn Fn(&str) -> bool,
) -> Option<JobId> {
    let id = retried_job(records, request)?;
    let job = records.jobs.get(id.as_str())?;
    let state = job.state();
    if state == JobState::Dispatched && running(id.as_str()) {
        return None;
    }
    let reason = job.outcome.as_ref().map(|o| o.reason.as_str());
    crate::disk::resumable_stop(state, reason).then_some(id)
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

/// The most changed files of one earlier job that are carried; the rest are counted, not named.
const CARRIED_CHANGED_FILES: usize = 30;

/// The most characters of the producer's account that are carried; the start is kept.
const CARRIED_APPROACH_CHARS: usize = 800;

/// The most measured-check lines that are carried.
const CARRIED_CHECK_LINES: usize = 20;

/// The parts of [`EarlierAttempt`] that say what the earlier job did.
struct CarriedWork {
    changed_files: Vec<String>,
    changed_files_omitted: usize,
    approach: Option<String>,
    checks: Vec<String>,
}

/// What the earlier job did, from the records it already has: the files its candidate changed (`status
/// path`, at most [`CARRIED_CHANGED_FILES`]), the producer's own summary (at most
/// [`CARRIED_APPROACH_CHARS`] characters) and one line per measured check (at most
/// [`CARRIED_CHECK_LINES`], no output text). All of it is redacted. A file whose name redaction altered is
/// dropped from the list and counted in the returned number, with the files past the cap, so no reading is
/// ever asked to repeat a redaction marker.
fn carried_work(job: &JobRecords, redactor: &Redactor) -> Result<CarriedWork> {
    use athena_data::records::ChangeStatus;
    let raw_files: Vec<String> = job
        .candidate
        .iter()
        .flat_map(|c| &c.changes)
        .map(|c| {
            let status = match c.status {
                ChangeStatus::Added => "added",
                ChangeStatus::Modified => "modified",
                ChangeStatus::Deleted => "deleted",
                ChangeStatus::TypeChanged => "type-changed",
            };
            format!("{status} {}", c.path)
        })
        .collect();
    let summary = job
        .candidate
        .as_ref()
        .and_then(|c| c.producer_report.as_ref())
        .map(|r| r.summary.trim().to_string())
        .filter(|s| !s.is_empty());
    let checks: Vec<String> = job
        .verification
        .iter()
        .flat_map(|v| &v.checks)
        .take(CARRIED_CHECK_LINES)
        .map(|c| {
            let exit = c.exit_code.map_or("no exit code".to_string(), |code| {
                format!("exit code {code}")
            });
            let tests = c
                .tests_ran
                .map_or(String::new(), |n| format!(", {n} tests ran"));
            let result = if c.passed { "passed" } else { "failed" };
            format!("{}: {result} ({exit}{tests})", c.name)
        })
        .collect();
    // One redaction pass over the files, the summary and the check lines, so a secret that spans them is
    // found; the summary is shortened after redaction so a cut never leaves half a secret.
    let files = raw_files.len();
    let mut texts = raw_files.clone();
    texts.extend(summary.clone());
    texts.extend(checks);
    redactor.redact_all(&mut texts)?;
    let mut rest = texts.split_off(files);
    let approach = summary.map(|_| rest.remove(0));
    let approach = approach.map(|s| crate::truncate_at_word(&s, CARRIED_APPROACH_CHARS));
    let kept: Vec<String> = texts
        .into_iter()
        .zip(&raw_files)
        .filter(|(redacted, raw)| redacted == *raw)
        .map(|(_, raw)| raw.clone())
        .take(CARRIED_CHANGED_FILES)
        .collect();
    let omitted = files - kept.len();
    Ok(CarriedWork {
        changed_files: kept,
        changed_files_omitted: omitted,
        approach,
        checks: rest,
    })
}

/// The job's request, final verification findings and reasons, and each failed check's name, command,
/// exit code and output tail, redacted; the files its candidate changed, the producer's account and the
/// measured checks (see [`carried_work`]). `None` without a request.
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
        extra.extend(reading.out_of_scope.iter().cloned());
    }
    let deliverable_count = final_reading.map_or(0, |r| r.deliverables.len());
    let lead = extra.len();
    extra.extend(job.corrections.iter().map(|c| c.text.clone()));
    redactor.redact_all(&mut extra)?;
    let corrections = extra.split_off(lead);
    let mut extra = extra.into_iter();
    let reading_summary = final_reading.and_then(|_| extra.next());
    let reading_criteria: Vec<String> = extra.by_ref().take(criteria).collect();
    let work = carried_work(job, redactor)?;
    Ok(Some(EarlierAttempt {
        introspection: None,
        job: job.id().as_str().to_string(),
        request,
        failed,
        findings: texts,
        reading_summary,
        reading_criteria,
        reading_deliverables: extra.by_ref().take(deliverable_count).collect(),
        reading_out_of_scope: extra.collect(),
        end_state: job.outcome.as_ref().map_or_else(
            || job.state().name().to_string(),
            |o| o.state.as_str().to_string(),
        ),
        verdict: job
            .verification
            .as_ref()
            .map(|v| format!("{:?}", v.final_verdict).to_lowercase()),
        corrections,
        changed_files: work.changed_files,
        changed_files_omitted: work.changed_files_omitted,
        approach: work.approach,
        checks: work.checks,
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
    /// The priority the owner gave the section: from a `Priority: <level>` line in an addendum's body, or
    /// high when the body says its work ranks above all other work or is a top priority. Normal otherwise.
    pub priority: Priority,
}

/// The priority an addendum's body states (see [`BriefTopic::priority`]).
fn addendum_priority(body: &str) -> Priority {
    let stated = Priority::from_text(body);
    if stated != Priority::Normal {
        return stated;
    }
    let lower = body.to_lowercase();
    if [
        "ranks above all other work",
        "top priority",
        "highest priority",
    ]
    .iter()
    .any(|p| lower.contains(p))
    {
        Priority::High
    } else {
        Priority::Normal
    }
}

/// The sections of `brief` (the text of BRIEF.md) that a request can serve: every `###` heading, and for those
/// under `## Addenda` the body too. An addendum that states a priority carries it, so planning can rank the
/// work that serves it ahead of default order.
pub fn brief_topics(brief: &str) -> Vec<BriefTopic> {
    let mut topics: Vec<BriefTopic> = Vec::new();
    let mut in_addenda = false;
    let mut in_topic = false;
    for line in brief.lines() {
        if let Some(h) = line.strip_prefix("### ") {
            topics.push(BriefTopic {
                heading: h.trim().to_string(),
                text: String::new(),
                priority: Priority::Normal,
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
    for t in &mut topics {
        t.priority = addendum_priority(&t.text);
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

/// One banned boundary word found in a request or in a file the request names, before the job is admitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputFinding {
    /// `request` for the request text, else the repository path of the named file.
    pub source: String,
    pub line: usize,
    pub term: String,
}

impl std::fmt::Display for InputFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.source, self.line, self.term)
    }
}

/// The boundary scan (the same one `cargo test -p xtask --test policy` runs, from the same word lists) over
/// a request's text and over each file the request names that exists at `treeish`. A path is "named" when
/// [`request_text_touches`] finds it in the text. Files under `records/` and the license are skipped, as the
/// policy scan skips them; a file that is missing or is not text is skipped. Findings come request first,
/// then files in the order they are named.
pub fn scan_request_input(
    git: &athena_git::Git,
    treeish: &str,
    text: &str,
) -> Result<Vec<InputFinding>> {
    let mut findings: Vec<InputFinding> = athena_context::briefs::boundary_scan(text)
        .into_iter()
        .map(|hit| InputFinding {
            source: "request".to_string(),
            line: hit.line,
            term: hit.term,
        })
        .collect();
    for path in request_text_touches(text) {
        if path.starts_with("records/") || path == "LICENSE" {
            continue;
        }
        let Some(bytes) = git.show_file(treeish, &path)? else {
            continue;
        };
        let Ok(content) = String::from_utf8(bytes) else {
            continue;
        };
        for hit in athena_context::briefs::boundary_scan(&content) {
            findings.push(InputFinding {
                source: path.clone(),
                line: hit.line,
                term: hit.term,
            });
        }
    }
    Ok(findings)
}

/// Refuses a request whose text, or a file it names, holds a banned boundary word, so the words are
/// reworded before a job exists. Nothing is edited: the message lists each source, line and word.
fn refuse_banned_input(git: &athena_git::Git, treeish: &str, text: &str) -> Result<()> {
    let findings = scan_request_input(git, treeish, text)?;
    if findings.is_empty() {
        return Ok(());
    }
    let list: Vec<String> = findings.iter().map(|f| format!("  {f}")).collect();
    bail!(
        "the request was not admitted: it, or a file it names, holds words Athena's boundary scan rejects \
         (source:line: word). Reword them and ask again:\n{}",
        list.join("\n")
    );
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
/// - A read request whose design names no existing code it changes or reuses, and does not list only files it
///   creates, gets `code_problem` set ([`crate::plan::code_reference_problem`]), so the plan refuses it before
///   production. This is the early form of the check the verifier made only at the final verdict.
/// - A read request whose design modifies `athena.toml` or `defaults.toml` without a path under
///   `crates/athena-data/` gets `schema_problem` set ([`crate::plan::schema_reference_problem`]), so the plan
///   refuses it before production instead of admission refusing an unknown field after it.
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
    // The owner's stated priority reaches the plan: a request serving an addendum that states one is ranked
    // at least that high. A priority record from `athena priority` is the user's word and is left alone.
    if job.priorities.is_empty() {
        let mut hay = text.to_string();
        if let Some(r) = job.latest_reading() {
            hay.push(' ');
            hay.push_str(&r.summary);
        }
        if let Some(topic) = serves_topic(&hay, lookup.brief)
            && topic.priority < entry.priority
        {
            entry.priority = topic.priority;
        }
    }
    for path in &entry.touches {
        if !creates.contains(path) && !(lookup.path_exists)(path) {
            entry.open_questions.push(format!(
                "the touched path {path} does not exist at the base commit"
            ));
        }
    }
    // The early check that the design names the existing code it changes or reuses (or says it is wholly
    // new): it runs here, when the plan is made, not at the final verdict. A declined reading has no design.
    if let Some(reading) = job.latest_reading()
        && reading.capability.is_some()
    {
        entry.code_problem =
            crate::plan::code_reference_problem(&entry.touches, &creates, lookup.path_exists);
        // The same stage checks that a configuration edit comes with the schema that must accept it.
        let changes: Vec<(String, bool)> = reading
            .deliverables
            .iter()
            .map(|d| (d.path.clone(), d.action == DeliverableAction::Delete))
            .collect();
        entry.schema_problem = crate::plan::schema_reference_problem(&changes);
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

/// Words that may sit between a follow-up phrase and the job id it names ("follows up on job <id>").
const FOLLOW_UP_FILLER: [&str; 6] = ["on", "of", "to", "job", "the", "earlier"];

/// The phrases that say a request continues a named job.
const FOLLOW_UP_PHRASES: [&str; 5] = [
    "follows up",
    "follow up",
    "follow-up",
    "following up",
    "followup",
];

/// The job ids a request says it follows up: the id that comes right after a phrase such as "follows up
/// job <id>", "follow-up of <id>" or "following up on <id>" (case-insensitive; only the words `on`, `of`,
/// `to`, `job`, `the` and `earlier` may sit between the phrase and the id). Every other id in the text is
/// evidence the request cites, not a job it continues. In order, without repeats.
pub fn followed_up_ids(text: &str) -> Vec<RecordId> {
    // ASCII lowercasing keeps byte offsets, so positions in `lower` index `text` too.
    let lower = text.to_ascii_lowercase();
    let mut found: Vec<(usize, RecordId)> = Vec::new();
    for phrase in FOLLOW_UP_PHRASES {
        for (at, _) in lower.match_indices(phrase) {
            let rest = &text[at + phrase.len()..];
            let words = rest.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'));
            for word in words.filter(|w| !w.is_empty()) {
                if FOLLOW_UP_FILLER.contains(&word.to_ascii_lowercase().as_str()) {
                    continue;
                }
                if let Ok(id) = RecordId::parse(word) {
                    found.push((at, id));
                }
                break;
            }
        }
    }
    found.sort_by_key(|(at, _)| *at);
    let mut ids: Vec<RecordId> = Vec::new();
    for (_, id) in found {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

/// The earlier jobs a reading must account for: the retried job (`attempt`, from the job's `previous_job`
/// link) and those of the carried `referenced` jobs that the request says it follows up (see
/// [`followed_up_ids`]). Jobs the request only cites as evidence are left out, so their results need not be
/// restated; the reader still sees them in its brief.
pub fn continued_jobs<'a>(
    request: &str,
    attempt: Option<&'a EarlierAttempt>,
    referenced: &'a [EarlierAttempt],
) -> Vec<&'a EarlierAttempt> {
    let followed = followed_up_ids(request);
    attempt
        .into_iter()
        .chain(
            referenced
                .iter()
                .filter(|r| followed.iter().any(|id| id.as_str() == r.job)),
        )
        .collect()
}

/// The jobs a request names by full id, other than `skip` (the retried job): each one's request and
/// final findings, redacted. Ids that match no job (or a job without a request) are ignored. At most
/// [`MAX_REFERENCED_JOBS`] are carried; jobs the request says it follows up come first so the cap never
/// drops one. Carrying a job only shows it to the reader: [`continued_jobs`] decides which of them the
/// reading must account for.
pub fn referenced_jobs(
    records: &Records,
    request: &str,
    skip: Option<&str>,
    redactor: &Redactor,
) -> Result<Vec<EarlierAttempt>> {
    let mut out = Vec::new();
    let mut ids = followed_up_ids(request);
    for id in job_ids_in(request) {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    for id in ids {
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

/// Refuses a reading for a retried or follow-up job that leaves out the previous result or the change
/// scope. `priors` are the retried job and the jobs the request says it follows up (see
/// [`continued_jobs`]), never jobs it only cites as evidence; with none (a first attempt, or a request
/// that cites jobs as evidence) every reading passes, and so does one that declines the request or asks a question, which has no change to
/// scope yet.
///
/// - The previous result is reflected when the reading's own text (title, summary, assumptions,
///   departures and criteria) names each prior job by its id, so each finding is addressed or set aside
///   in words the user can read. The brief asks for exactly this.
/// - The change scope is stated when the reading lists at least one deliverable and at least one
///   out-of-scope item.
/// - The earlier job's files are accounted for when each file its candidate changed (the carried
///   `changed_files`) is covered by a deliverable (the same path, or a directory deliverable above it) or is
///   named in an out-of-scope item, as its own path or as a directory above it (see [`names_path_token`]).
///   Only deliverables and out-of-scope items count: the survey and other prose of a reading do not say
///   whether earlier work is kept or dropped. A prior with no recorded files asks for nothing.
///
/// The error names each missing part: "previous result", "change scope" and "the earlier job's files".
///
/// Root cause this part removes: the carry-over of an earlier job left out the files its candidate changed,
/// so a retry's reading could name the job and still ignore (keep or drop without saying) the work it did.
///
/// This check refuses more readings; the higher refusal rate is intended. A replay of accepted readings
/// (`replay`) holds a job that newly refuses more than 10% unless the job's criteria declare a share, as
/// "expected N% of newly refused readings", and give a reason when it is above 25%. The replay passes no
/// earlier attempts to the checks (see `check_reading_text`), so it cannot observe this part: the share a
/// job declares is an expectation, not a measurement.
pub fn check_prior_result_and_scope(reading: &Reading, priors: &[&EarlierAttempt]) -> Result<()> {
    if priors.is_empty() || reading.capability.is_none() || reading.question.is_some() {
        return Ok(());
    }
    let mut text = vec![
        reading.title.as_str(),
        reading.summary.as_str(),
        reading.departures.as_deref().unwrap_or_default(),
    ];
    text.extend(reading.assumptions.iter().map(String::as_str));
    text.extend(reading.acceptance_criteria.iter().map(|c| c.text.as_str()));
    let text = text.join("\n").to_lowercase();
    let unnamed: Vec<&str> = priors
        .iter()
        .map(|p| p.job.as_str())
        .filter(|id| !text.contains(&id.to_lowercase()))
        .collect();
    let mut missing: Vec<String> = Vec::new();
    if !unnamed.is_empty() {
        missing.push(format!(
            "the previous result: it does not name job {} in its summary, assumptions or criteria, \
             so it does not say how the earlier findings and end state are addressed or why they no longer apply",
            unnamed.join(", job ")
        ));
    }
    let mut scope: Vec<&str> = Vec::new();
    if reading.deliverables.is_empty() {
        scope.push("deliverables");
    }
    if reading.out_of_scope.iter().all(|i| i.trim().is_empty()) {
        scope.push("out_of_scope items");
    }
    if !scope.is_empty() {
        missing.push(format!(
            "the change scope: it lists no {}",
            scope.join(" and no ")
        ));
    }
    let ignored: Vec<String> = priors
        .iter()
        .filter_map(|p| {
            let files = unaccounted_files(p, reading);
            if files.is_empty() {
                return None;
            }
            let shown = files
                .iter()
                .take(MAX_FILES_NAMED)
                .copied()
                .collect::<Vec<_>>();
            let more = match files.len() - shown.len() {
                0 => String::new(),
                n => format!(" and {n} more"),
            };
            Some(format!("job {} changed {}{more}", p.job, shown.join(", ")))
        })
        .collect();
    if !ignored.is_empty() {
        missing.push(format!(
            "the earlier job's files: {}, but the reading neither lists them as deliverables nor excludes them \
             in out_of_scope, so it does not say whether that work is kept or dropped",
            ignored.join("; ")
        ));
    }
    if missing.is_empty() {
        return Ok(());
    }
    bail!(
        "the reading was refused because it leaves out {}; this job retries or follows up an earlier job, \
         so the reading must reflect that job's result and state the allowed change scope",
        missing.join("; and ")
    )
}

/// The most files of one earlier job that a refusal names.
const MAX_FILES_NAMED: usize = 5;

/// The paths of the files `prior`'s candidate changed that the reading neither lists nor excludes: a file is
/// accounted for when a deliverable is the file itself or a directory above it (compared by whole path
/// components), or when an out-of-scope item names the file or a directory above it as a whole path token
/// ([`names_path_token`]). Case is ignored. Free text such as the summary or the assumptions never counts.
fn unaccounted_files<'a>(prior: &'a EarlierAttempt, reading: &Reading) -> Vec<&'a str> {
    let deliverables: Vec<String> = reading
        .deliverables
        .iter()
        .map(|d| d.path.trim().trim_end_matches('/').to_lowercase())
        .collect();
    let out_of_scope: Vec<String> = reading
        .out_of_scope
        .iter()
        .map(|item| item.to_lowercase())
        .collect();
    prior
        .changed_files
        .iter()
        // A carried entry is `status path`; the path may itself hold spaces.
        .filter_map(|entry| entry.split_once(' ').map(|(_, path)| path))
        .filter(|path| {
            let file = path.to_lowercase();
            let covered = deliverables.iter().any(|d| {
                file == *d
                    || file
                        .strip_prefix(d.as_str())
                        .is_some_and(|r| r.starts_with('/'))
            });
            let mut names = vec![file.clone()];
            let mut ancestor = file.as_str();
            while let Some((parent, _)) = ancestor.rsplit_once('/') {
                names.push(parent.to_string());
                names.push(format!("{parent}/"));
                ancestor = parent;
            }
            let excluded = out_of_scope
                .iter()
                .any(|item| names.iter().any(|name| names_path_token(item, name)));
            !(covered || excluded)
        })
        .collect()
}

/// Whether `item` holds `token` as a whole path token (both lowercase). A token starts at the start of the
/// text or after white space, a backtick, a quote, `(`, `[`, `{`, `,` or `;`. It ends at the end of the text, or
/// before white space, a closing quote, a backtick, `)`, `]`, `}`, `,`, `;`, `:` or `*`, or before a `.`, `!` or `?`
/// that is itself followed by the end of the text or white space. A `/` or a letter or digit next to the token
/// never matches, so `src/a.rs` is not found in `crates/x/src/a.rs`, and `src/a` is not found in `src/a.rs`.
fn names_path_token(item: &str, token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    let starts = |c: char| c.is_whitespace() || "`\"'([{,;\u{2018}\u{201c}".contains(c);
    let ends = |c: char| c.is_whitespace() || "`\"')]},;:*\u{2019}\u{201d}".contains(c);
    item.match_indices(token).any(|(at, _)| {
        let before_ok = item[..at].chars().next_back().is_none_or(starts);
        let rest = &item[at + token.len()..];
        let mut after = rest.chars();
        let after_ok = match after.next() {
            None => true,
            Some(c) if ends(c) => true,
            Some('.' | '!' | '?') => after.next().is_none_or(char::is_whitespace),
            Some(_) => false,
        };
        before_ok && after_ok
    })
}

/// The free-text fields of a reading, each with the label a refusal names it by. The deliverable paths are
/// not here: [`check_boundary_and_repo_files`] scans them as paths.
fn reading_text_fields(reading: &Reading) -> Vec<(String, &str)> {
    let mut fields: Vec<(String, &str)> = vec![
        ("title".into(), reading.title.as_str()),
        ("summary".into(), reading.summary.as_str()),
    ];
    let optional = [
        ("decline_reason", &reading.decline_reason),
        ("question", &reading.question),
        ("departures", &reading.departures),
        ("hard_reason", &reading.hard_reason),
        ("urgent_reason", &reading.urgent_reason),
    ];
    for (label, value) in optional {
        if let Some(text) = value {
            fields.push((label.into(), text.as_str()));
        }
    }
    for (i, text) in reading.assumptions.iter().enumerate() {
        fields.push((format!("assumptions[{i}]"), text.as_str()));
    }
    for (i, text) in reading.out_of_scope.iter().enumerate() {
        fields.push((format!("out_of_scope[{i}]"), text.as_str()));
    }
    for (i, criterion) in reading.acceptance_criteria.iter().enumerate() {
        fields.push((format!("acceptance_criteria[{i}]"), criterion.text.as_str()));
    }
    for (i, default) in reading.defaults_applied.iter().enumerate() {
        fields.push((format!("defaults_applied[{i}]"), default.how.as_str()));
    }
    fields
}

/// Whether `path` is a file in `files`, or a directory with at least one file under it.
fn path_exists_in(files: &BTreeSet<String>, path: &str) -> bool {
    if files.contains(path) {
        return true;
    }
    let prefix = format!("{}/", path.trim_end_matches('/'));
    files
        .range(prefix.clone()..)
        .next()
        .is_some_and(|f| f.starts_with(&prefix))
}

/// Refuses a reading whose text holds a word the boundary scan rejects, or whose deliverables disagree with
/// the files at the base commit.
///
/// Root cause: a reading was checked only for the previous result and change scope, the evidence each
/// criterion rests on, the code survey and the governance files. Nothing compared its text with the boundary
/// word list or its deliverables with the repository's files at the base commit. A reading that named a
/// banned word, a path that does not exist, or a `create` of a file that already exists was accepted, and the
/// fault showed up only later, in the producer's run or at verification. The check is deterministic and
/// calls no model; it runs last in [`check_reading_acceptance`], so a refusal reaches the reader as its one
/// correction, as the other refusals do.
///
/// Every free-text field and every deliverable path goes through `athena_context::briefs::boundary_scan`.
/// `repo_files` is the set of file paths at the base commit. When it is `Some`
/// and the reading is a change with no open question, a `modify` or `delete` of a path that is neither a file
/// nor a directory holding files is refused, and so is a `create` of a path that is already a file. A
/// `modify` of `CHANGELOG.md` is allowed when the file is absent, because Athena adds that deliverable
/// itself ([`ensure_changelog`]) and the reader cannot repair it. With `None` (a replay with no repository)
/// only the words are checked.
pub fn check_boundary_and_repo_files(
    reading: &Reading,
    repo_files: Option<&BTreeSet<String>>,
) -> Result<()> {
    let mut problems: Vec<String> = Vec::new();
    for (label, text) in reading_text_fields(reading) {
        for hit in athena_context::briefs::boundary_scan(text) {
            problems.push(format!("{label}: the banned word {:?}", hit.term));
        }
    }
    for d in &reading.deliverables {
        for hit in athena_context::briefs::boundary_scan(&d.path) {
            problems.push(format!(
                "deliverable path {}: the banned word {:?}",
                d.path, hit.term
            ));
        }
    }
    let mut path_problems = 0;
    if let Some(files) = repo_files
        && reading.capability.is_some()
        && reading.question.is_none()
    {
        for d in &reading.deliverables {
            let message = match d.action {
                DeliverableAction::Create if files.contains(&d.path) => Some(format!(
                    "deliverable {} is a create, but the file already exists at the base commit \
                     (use modify)",
                    d.path
                )),
                DeliverableAction::Modify | DeliverableAction::Delete
                    if !path_exists_in(files, &d.path)
                        && !(d.action == DeliverableAction::Modify && d.path == CHANGELOG_PATH) =>
                {
                    let verb = if d.action == DeliverableAction::Modify {
                        "modify"
                    } else {
                        "delete"
                    };
                    Some(format!(
                        "deliverable {} is a {verb}, but no such file exists at the base commit \
                         (use create for a new file, or name the path that exists)",
                        d.path
                    ))
                }
                _ => None,
            };
            if let Some(message) = message {
                problems.push(message);
                path_problems += 1;
            }
        }
    }
    if problems.is_empty() {
        return Ok(());
    }
    let word_problems = problems.len() - path_problems;
    let repair = match (word_problems > 0, path_problems > 0) {
        (true, true) => {
            "reword the text without the banned words, and make each deliverable's action match the files"
        }
        (true, false) => "reword the text without the banned words",
        _ => "make each deliverable's action match the files at the base commit",
    };
    bail!(
        "the reading was refused because its text or deliverables fail a check Athena runs on every \
         reading: {}; {repair}",
        problems.join("; ")
    )
}

/// Every check a built reading must pass before it is accepted: the previous result and change scope of a
/// retried or followed-up job ([`check_prior_result_and_scope`]), then the evidence each criterion rests on
/// ([`check_criteria_evidence`]), then the code survey a code or tests reading with a behavior criterion
/// rests on ([`check_code_survey`]), then the governance files the outcome needs
/// ([`check_governance_coverage`]), then the reading's text against the boundary words and its deliverables
/// against the files at the base commit ([`check_boundary_and_repo_files`]; `repo_files` is `None` when
/// there is no repository to compare with). It applies to a first attempt as well as a retry.
pub fn check_reading_acceptance(
    reading: &Reading,
    request: &str,
    attempt: Option<&EarlierAttempt>,
    referenced: &[EarlierAttempt],
    repo_files: Option<&BTreeSet<String>>,
) -> Result<()> {
    check_prior_result_and_scope(reading, &continued_jobs(request, attempt, referenced))?;
    check_criteria_evidence(reading)?;
    check_code_survey(reading)?;
    check_governance_coverage(reading, request)?;
    check_boundary_and_repo_files(reading, repo_files)
}

/// Governance paths a request can name, and the whole-word lowercase phrase that names each. A path ending in
/// `/` stands for any file under that directory; its phrase has no trailing `/`, so a file path under the
/// directory (`.github/workflows/ci.yml`) matches as well as the directory. A request word such as "deploy"
/// or "config" names no path and requires nothing.
const GOVERNANCE_BY_NAMED_PATH: [(&str, &str); 5] = [
    (".github/workflows/", ".github/workflows"),
    ("athena.toml", "athena.toml"),
    ("defaults.toml", "defaults.toml"),
    ("rust-toolchain.toml", "rust-toolchain.toml"),
    ("setup.sh", "setup.sh"),
];

/// Deliverables whose change implies a governance file changes with them, and why.
const GOVERNANCE_BY_DELIVERABLE: [(&str, &str, &str); 1] = [(
    "crates/athena-data/src/config.rs",
    "athena.toml",
    "a new or changed configuration section must be set in athena.toml, or its schema test fails",
)];

/// Whether the reading covers the governance `path` (a directory when it ends in `/`): it lists a governance
/// deliverable there and names it in a criterion, or it names the path in `out_of_scope`.
fn governance_is_covered(reading: &Reading, path: &str) -> bool {
    let out_of_scope = reading
        .out_of_scope
        .iter()
        .any(|item| item.contains(path.trim_end_matches('/')));
    if out_of_scope {
        return true;
    }
    if path.ends_with('/') {
        reading
            .deliverables
            .iter()
            .any(|d| d.path.starts_with(path) && governance_change_is_listed(reading, &d.path))
    } else {
        governance_change_is_listed(reading, path)
    }
}

/// Refuses a reading that leaves out a governance file its outcome needs, or lists one without naming it.
///
/// Root cause this check removes: reading acceptance had no mapping from the request's outcome to governance
/// files, so a reading for "publish to GitHub Pages" (which needs a workflow), a new configuration section
/// (which needs `athena.toml`) or CI routing (which needs `athena.toml`) was accepted without them, and the
/// omission was found only at admission, after the producer had finished. Jobs 20261014-082328-g6vy,
/// 20261015-014012-akwq, 20261017-044557-3htd and 20261017-114155-ksq0 each failed that way. Admission has one
/// definition of a governance file (`is_governance`) and of a listed one (`governance_change_is_listed`); this
/// check applies the same two to the reading, where the reader can still repair it, for every reading with a
/// capability and a deliverable, first attempt or retry. It costs no model call.
///
/// It refuses when:
/// - a deliverable is a governance file that no criterion names;
/// - a deliverable in `GOVERNANCE_BY_DELIVERABLE` is listed without its governance file; or
/// - the request names a path of `GOVERNANCE_BY_NAMED_PATH` (the file itself, or a file under the directory)
///   and the reading neither lists and names that governance file nor names it in `out_of_scope`.
///
/// A request word that only suggests a governance file (deploy, config, CI, gate) requires nothing: the
/// earlier word table refused ordinary requests that merely used a common word, and the reader was not told
/// why, so the next read failed the same way (`Athena::read_for` now reads once more with the refusal text).
/// The error names each missing path and why, and says to list it as a deliverable and name it in a
/// criterion, or to name it in `out_of_scope`.
pub fn check_governance_coverage(reading: &Reading, request: &str) -> Result<()> {
    if reading.capability.is_none() || reading.question.is_some() || reading.deliverables.is_empty()
    {
        return Ok(());
    }
    let mut missing: Vec<(String, String)> = Vec::new();
    let mut need = |path: &str, why: String| {
        if !missing.iter().any(|(p, _)| p == path) {
            missing.push((path.to_string(), why));
        }
    };
    for d in &reading.deliverables {
        if is_governance(&d.path) && !governance_change_is_listed(reading, &d.path) {
            need(
                &d.path,
                "it is listed as a deliverable but no criterion names it".into(),
            );
        }
        for (trigger, path, why) in GOVERNANCE_BY_DELIVERABLE {
            if d.path == trigger && !governance_is_covered(reading, path) {
                need(path, format!("{trigger} is a deliverable, and {why}"));
            }
        }
    }
    // An underscore joins a word (`athena_gate` is one identifier), so it is not a boundary.
    let words = request.to_lowercase().replace('_', "x");
    for (path, phrase) in GOVERNANCE_BY_NAMED_PATH {
        if has_phrase(&words, phrase) && !governance_is_covered(reading, path) {
            need(
                path,
                format!(
                    "the request names {phrase:?}, so the reading must say whether the job changes it"
                ),
            );
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    let paths: Vec<String> = missing
        .iter()
        .map(|(path, why)| format!("{path} ({why})"))
        .collect();
    bail!(
        "the reading was refused because the outcome needs governance files it does not cover: {}; \
         list each missing path as a deliverable and name it in an acceptance criterion, or, if this job must \
         not change it, name it in out_of_scope; admission refuses a governance change that is not both listed \
         and named, and only after the producer has finished",
        paths.join("; ")
    )
}

/// Phrases that make the producer's own account the evidence for a criterion. Each is a whole-word
/// phrase, lowercase, with a straight apostrophe.
const REPORT_NOUNS: [&str; 5] = ["report", "summary", "claims", "statement", "account"];
const PRODUCER_VERBS: [&str; 12] = [
    "states",
    "confirms",
    "says",
    "lists",
    "reports",
    "claims",
    "attests",
    "declares",
    "notes",
    "describes",
    "asserts",
    "writes",
];
const REPORT_VERBS: [&str; 13] = [
    "lists",
    "states",
    "confirms",
    "says",
    "shows",
    "names",
    "records",
    "claims",
    "mentions",
    "notes",
    "describes",
    "cites",
    "includes",
];

/// Whether `text` contains `phrase` as whole words.
fn has_phrase(text: &str, phrase: &str) -> bool {
    text.match_indices(phrase).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + phrase.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

/// The phrase that makes a criterion rest on the producer's own account, if it has one. Matching is on
/// lowercase text with curly apostrophes made straight and runs of white space made single spaces.
fn producer_evidence_phrase(criterion: &str) -> Option<String> {
    let text = criterion
        .to_lowercase()
        .replace(['\u{2019}', '\u{2018}'], "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let mut phrases: Vec<String> = Vec::new();
    for noun in REPORT_NOUNS {
        phrases.push(format!("producer's {noun}"));
        phrases.push(format!("producers' {noun}"));
        phrases.push(format!("producer {noun}"));
        phrases.push(format!("{noun} from the producer"));
        phrases.push(format!("{noun} of the producer"));
        phrases.push(format!("{noun} by the producer"));
    }
    for verb in PRODUCER_VERBS {
        phrases.push(format!("the producer {verb}"));
        phrases.push(format!("producer {verb}"));
    }
    for verb in REPORT_VERBS {
        phrases.push(format!("the report {verb}"));
    }
    phrases.push("according to the producer".into());
    phrases.push("reported by the producer".into());
    phrases.into_iter().find(|p| has_phrase(&text, p))
}

/// Refuses a reading whose acceptance criterion rests on the producer's own report, summary, claims or
/// statement as its evidence.
///
/// Root cause this check removes: criteria were written without a defined list of what the verifier can
/// observe (the candidate diff, the repository snapshot, the recorded check output, and the commands it is
/// allowed to run), and nothing compared the evidence a criterion requires with that list. A criterion such
/// as "the producer's report lists the caller-check Grep" can only be satisfied by something the verifier
/// never sees, so it was accepted and then could not be judged. Jobs 20261027-004517-gk9e,
/// 20261027-011733-vrsk, 20261027-012938-yckz, 20261026-232402-6qme and 20261026-232423-hvyb each failed that
/// way. The list is now the reader's brief ("What the verifier can run") and this check enforces it where
/// criteria are first fixed, for every reading with a capability, first attempt or retry.
///
/// The error names each refused criterion by number and text and says to restate it as a command or Grep the
/// verifier can run, or a file in the snapshot.
pub fn check_criteria_evidence(reading: &Reading) -> Result<()> {
    if reading.capability.is_none() {
        return Ok(());
    }
    let refused: Vec<String> = reading
        .acceptance_criteria
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            producer_evidence_phrase(&c.text)
                .map(|phrase| format!("criterion {} ({:?}) cites {phrase:?}", i + 1, c.text))
        })
        .collect();
    if refused.is_empty() {
        return Ok(());
    }
    bail!(
        "the reading was refused because {}; the verifier sees the candidate diff, the repository snapshot and \
         the recorded check output, never the producer's report, summary, claims or statement, so a criterion \
         that rests on them cannot be judged; restate each as a command or Grep the verifier can run, or a \
         file in the snapshot",
        refused.join("; and ")
    )
}

/// The `Code survey:` assumptions of a reading: those whose trimmed text starts with that label, in any case.
fn code_surveys(reading: &Reading) -> Vec<&str> {
    reading
        .assumptions
        .iter()
        .map(|a| a.trim())
        .filter(|a| a.to_lowercase().starts_with("code survey:"))
        .collect()
}

/// Whether `text` names a path under `crates/`: `crates/` followed by a letter or digit.
fn names_crates_path(text: &str) -> bool {
    text.match_indices("crates/").any(|(at, marker)| {
        text[at + marker.len()..]
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric)
    })
}

/// Whether the deliverable adds or changes a record or data file under `crates/`: it is created there with a
/// data extension (not a `Cargo.toml`) or under a `records` or `data` directory, or it is a `records.rs`.
/// Only the deliverable list is read, never a criterion, so a criterion that says `records/` is not changed
/// triggers nothing.
fn is_record_or_data_deliverable(d: &Deliverable) -> bool {
    if !d.path.starts_with("crates/") {
        return false;
    }
    let mut parts = d.path.split('/');
    let file = parts.next_back().unwrap_or("");
    if file == "records.rs" {
        return true;
    }
    if d.action != DeliverableAction::Create || file == "Cargo.toml" {
        return false;
    }
    let data_extension = file.rsplit_once('.').is_some_and(|(_, ext)| {
        ["json", "jsonl", "ndjson", "toml", "yaml", "yml", "csv"].contains(&ext)
    });
    data_extension
        || d.path
            .split('/')
            .any(|part| part == "records" || part == "data")
}

/// Whether a survey names evidence that a new record or data file is accepted: `check_record_file`,
/// `AnyRecord`, or a round-trip test (an identifier with `round_trip` or `roundtrip`, or a path with
/// `round-trip`). A bare word such as "allow-list" or "round-trip" in prose is not evidence.
fn names_record_evidence(survey: &str) -> bool {
    survey
        .split(|c: char| !(c.is_alphanumeric() || matches!(c, '_' | '-' | '/')))
        .any(|token| {
            let lower = token.to_lowercase();
            token == "check_record_file"
                || token == "AnyRecord"
                || lower.contains("round_trip")
                || lower.contains("roundtrip")
                || (lower.contains("round-trip") && token.contains('/'))
        })
}

/// Refuses a reading that changes code or tests with a behavior criterion and carries no code survey: an
/// assumption that starts `Code survey:` and names a `crates/` path. A reading that also adds a record or data
/// file under `crates/` (or changes a `records.rs`) must have a survey that names `check_record_file`,
/// `AnyRecord`, or a round-trip test.
///
/// Root cause this check removes: the reading stage designs storage, override and test seams from a general
/// idea of the feature, without surveying what the records validation accepts or where resolution and probing
/// happen in the code. Jobs 20261028-055601-hg0n and 20261028-082151-67fa each began from a reading that
/// fixed its seams that way, and introspections 20261028-055651-dhh0 and 20261028-082240-77as traced both to
/// the missing survey. Reader briefs now ask for the survey (`CODE_SURVEY_GUIDE` in athena-context), and this
/// check enforces it where readings are accepted. It costs no model call, and it reads the assumptions and
/// the deliverable list, never the repository.
///
/// It applies when the reading has a capability and no question, its class of work is `code` or `tests`, and
/// at least one criterion is a behavior criterion. A docs reading, a decline, a question and a reading with
/// only content criteria need no survey.
///
/// This check refuses more readings; the higher refusal rate is intended. A replay of accepted readings
/// (`replay`) holds a job that newly refuses more than 10% unless the job's criteria declare a share, as
/// "expected N% of newly refused readings", and give a reason when it is above 25%.
///
/// The error names the missing survey or the missing evidence and tells the reader what to write.
pub fn check_code_survey(reading: &Reading) -> Result<()> {
    if reading.capability.is_none()
        || reading.question.is_some()
        || !matches!(reading.class_of_work.as_str(), "code" | "tests")
        || !reading
            .acceptance_criteria
            .iter()
            .any(|c| c.kind == CriterionKind::Behavior)
    {
        return Ok(());
    }
    let surveys: Vec<&str> = code_surveys(reading)
        .into_iter()
        .filter(|s| names_crates_path(s))
        .collect();
    if surveys.is_empty() {
        bail!(
            "the reading was refused because it changes {} with a behavior criterion but has no `Code survey:` \
             assumption that names a crates/ path; the criteria were designed from a general idea of the feature \
             and not from the code. Read the code, then add an assumption that starts `Code survey:` and names \
             the crates/ paths you read: the entry point and the order of resolution and probing, what the \
             records validation allow-list accepts, and how the property is measured directly",
            reading.class_of_work
        );
    }
    let new_records: Vec<&str> = reading
        .deliverables
        .iter()
        .filter(|d| is_record_or_data_deliverable(d))
        .map(|d| d.path.as_str())
        .collect();
    if !new_records.is_empty() && !surveys.iter().any(|s| names_record_evidence(s)) {
        bail!(
            "the reading was refused because it adds or changes a record or data file ({}) but its `Code survey:` \
             assumption does not name what accepts it; name `check_record_file`, `AnyRecord` or a round-trip test \
             by its function name or path, as read in the records validation, not the word allow-list alone",
            new_records.join(", ")
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------------------
// Replay of accepted readings through the base and candidate reading rules

/// The most accepted readings one replay uses: the newest ones.
pub const REPLAY_WINDOW: usize = 50;

/// A candidate that newly refuses more than this share of the replayed readings is held, unless it declares a
/// share of its own ([`replay_declaration`]). Exactly this share is not held.
pub const REPLAY_HOLD_PERCENT: usize = 10;

/// Why a replay of an empty history did not run.
pub const NO_HISTORY_REASON: &str = "no accepted readings in the history";

/// What the reading rules say about one reading and its request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadingCheck {
    Accept,
    /// Refused, with the reason on one line.
    Refuse(String),
}

impl ReadingCheck {
    /// The one line `athena check-reading` prints: `accept`, or `refuse: <reason>`.
    pub fn line(&self) -> String {
        match self {
            ReadingCheck::Accept => "accept".to_string(),
            ReadingCheck::Refuse(reason) => format!("refuse: {reason}"),
        }
    }

    /// Parses the first line of what `athena check-reading` printed.
    pub fn parse(output: &str) -> Result<ReadingCheck> {
        let line = output.lines().next().unwrap_or("").trim();
        if line == "accept" {
            return Ok(ReadingCheck::Accept);
        }
        match line.strip_prefix("refuse:") {
            Some(reason) if !reason.trim().is_empty() => {
                Ok(ReadingCheck::Refuse(reason.trim().to_string()))
            }
            _ => bail!("check-reading printed {line:?}, not `accept` or `refuse: <reason>`"),
        }
    }
}

/// Runs the reading rules ([`check_reading_acceptance`]) on one stored reading and request. There are no
/// earlier attempts, so the retry rule (`check_prior_result_and_scope`) sees a first attempt. This is what
/// `athena check-reading` runs, and no model is called. There is no repository to compare with, so the
/// words are checked and the deliverable paths are not.
pub fn check_reading_text(reading: &Reading, request: &str) -> ReadingCheck {
    match check_reading_acceptance(reading, request, None, &[], None) {
        Ok(()) => ReadingCheck::Accept,
        Err(error) => ReadingCheck::Refuse(
            error
                .to_string()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "),
        ),
    }
}

/// Something that applies one build's reading rules to a stored reading. It is given the request and the
/// reading and nothing else, so it cannot reach a model.
pub trait ReadingChecker {
    fn check(&self, request: &str, reading: &Reading) -> Result<ReadingCheck>;
}

impl<F> ReadingChecker for F
where
    F: Fn(&str, &Reading) -> Result<ReadingCheck>,
{
    fn check(&self, request: &str, reading: &Reading) -> Result<ReadingCheck> {
        self(request, reading)
    }
}

/// The base checker and the candidate checker of one replay.
pub type ReplayCheckers = (
    std::rc::Rc<dyn ReadingChecker>,
    std::rc::Rc<dyn ReadingChecker>,
);

/// A build of Athena, run as `<bin> check-reading --request <file> --reading <file>`. It holds a path and
/// nothing else: no driver, no agent handle.
#[derive(Debug, Clone)]
pub struct BinaryChecker {
    bin: PathBuf,
}

impl BinaryChecker {
    pub fn new(bin: PathBuf) -> BinaryChecker {
        BinaryChecker { bin }
    }
}

impl ReadingChecker for BinaryChecker {
    fn check(&self, request: &str, reading: &Reading) -> Result<ReadingCheck> {
        let dir = tempfile::tempdir().context("making a scratch directory for check-reading")?;
        let request_file = dir.path().join("request.txt");
        let reading_file = dir.path().join("reading.json");
        std::fs::write(&request_file, request)?;
        std::fs::write(
            &reading_file,
            AnyRecord::Reading(Box::new(reading.clone())).to_json(),
        )?;
        let output = std::process::Command::new(&self.bin)
            .arg("check-reading")
            .arg("--request")
            .arg(&request_file)
            .arg("--reading")
            .arg(&reading_file)
            .output()
            .with_context(|| format!("running {} check-reading", self.bin.display()))?;
        // Exit 0 is accept and 2 is refuse; both print the one line. Anything else is a failure to check.
        if !matches!(output.status.code(), Some(0 | 2)) {
            bail!(
                "{} check-reading exited with {}: {}",
                self.bin.display(),
                output.status,
                tail(&String::from_utf8_lossy(&output.stderr), 300)
            );
        }
        ReadingCheck::parse(&String::from_utf8_lossy(&output.stdout))
    }
}

/// One stored request and its accepted reading.
#[derive(Debug, Clone)]
pub struct ReplayCase {
    pub job: JobId,
    pub request: String,
    pub reading: Reading,
}

/// The newest [`REPLAY_WINDOW`] jobs that have an accepted reading, newest first, with their request and
/// that reading. `exclude` (the running job) is left out, so a history of only that job is empty.
pub fn replay_cases(records: &Records, exclude: Option<&JobId>) -> Vec<ReplayCase> {
    records
        .request_jobs()
        .into_iter()
        .filter(|j| Some(j.id()) != exclude)
        .filter_map(|j| {
            let n = j.job.as_ref()?.accepted_reading?;
            Some(ReplayCase {
                job: j.id().clone(),
                request: j.request.as_ref()?.text.clone(),
                reading: j.reading(n)?.clone(),
            })
        })
        .take(REPLAY_WINDOW)
        .collect()
}

/// How many percentage points above its declared share a candidate may newly refuse before it is held.
/// Exactly declared plus this many points is not held.
pub const REPLAY_DECLARED_MARGIN_POINTS: usize = 5;

/// A declared share above this percentage counts only with a stated reason (see [`replay_declaration`]).
/// Exactly this share needs no reason.
pub const REPLAY_REASON_ABOVE_PERCENT: u32 = 25;

/// The most characters of a stated reason that the replay keeps for the verifier.
const REPLAY_REASON_CHARS: usize = 300;

/// Words, lowercase, one of which a criterion or out_of_scope item must contain to count as a reason for a
/// declared share above [`REPLAY_REASON_ABOVE_PERCENT`] percent. Whole phrases, matched by `has_phrase`.
const REPLAY_REASON_MARKERS: [&str; 3] = ["because", "now asks", "new required"];

/// Words, lowercase, one of which the same item must also contain, so the sentence is about past readings or
/// refusals and not about something else. Whole phrases, matched by `has_phrase`.
const REPLAY_REASON_ANCHORS: [&str; 9] = [
    "past readings",
    "previous readings",
    "earlier readings",
    "older readings",
    "refuse",
    "refuses",
    "refused",
    "refusal",
    "refusals",
];

/// What a reading declares about the replay: the share of newly refused readings it expects, and the reason
/// it gives for a share above [`REPLAY_REASON_ABOVE_PERCENT`] percent.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplayDeclaration {
    /// The declared whole percent, 0 to 100, when a criterion states one.
    pub share: Option<u32>,
    /// The first criterion or out_of_scope item that gives a reason, cut to a fixed length.
    pub reason: Option<String>,
}

impl ReplayDeclaration {
    /// The share the replay holds the candidate to: the declared share when it is at most
    /// [`REPLAY_REASON_ABOVE_PERCENT`] percent or has a reason; otherwise none, so the
    /// [`REPLAY_HOLD_PERCENT`] limit applies.
    pub fn effective_share(&self) -> Option<u32> {
        self.share
            .filter(|s| *s <= REPLAY_REASON_ABOVE_PERCENT || self.reason.is_some())
    }

    /// The declared share when it is above [`REPLAY_REASON_ABOVE_PERCENT`] percent and has no reason, so it
    /// was not applied.
    pub fn unsupported_share(&self) -> Option<u32> {
        self.share
            .filter(|s| *s > REPLAY_REASON_ABOVE_PERCENT && self.reason.is_none())
    }
}

/// Lowercase text with runs of white space made single spaces.
fn normalised(text: &str) -> String {
    text.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The percent that `text` (normalised) declares for newly refused readings: a whole number from 0 to 100
/// and `%`, then optionally `of` and `the`, then `newly refused` ("expected 30% of newly refused readings").
/// A decimal, a number inside a word, or a percentage that is not tied to newly refused readings declares
/// nothing, so a quoted example does not count. The first match wins.
fn declared_percent(text: &str) -> Option<u32> {
    for (at, _) in text.match_indices('%') {
        let digits_start = at
            - text[..at]
                .bytes()
                .rev()
                .take_while(u8::is_ascii_digit)
                .count();
        let digits = &text[digits_start..at];
        if digits.is_empty() || digits.len() > 3 {
            continue;
        }
        let before = text[..digits_start].chars().next_back();
        if before.is_some_and(|c| c.is_alphanumeric() || c == '.') {
            continue;
        }
        let mut rest = text[at + 1..].trim_start();
        for word in ["of ", "the "] {
            rest = rest.strip_prefix(word).unwrap_or(rest);
        }
        let tied = rest
            .strip_prefix("newly refused")
            .is_some_and(|after| !after.chars().next().is_some_and(char::is_alphanumeric));
        match digits.parse::<u32>() {
            Ok(percent) if tied && percent <= 100 => return Some(percent),
            _ => {}
        }
    }
    None
}

/// What the criteria and out_of_scope items declare about the replay. A share is declared only as
/// "N% of newly refused readings" ([`declared_percent`]), the first in criteria order. The reason is the
/// first criterion, then out_of_scope item, that has one of [`REPLAY_REASON_MARKERS`] and one of
/// [`REPLAY_REASON_ANCHORS`]. This is a text floor, not proof of a good reason: the verifier is shown the
/// sentence beside the replay list and judges it.
pub fn replay_declaration(criteria: &[Criterion], out_of_scope: &[String]) -> ReplayDeclaration {
    let share = criteria
        .iter()
        .find_map(|c| declared_percent(&normalised(&c.text)));
    let reason = criteria
        .iter()
        .map(|c| c.text.as_str())
        .chain(out_of_scope.iter().map(String::as_str))
        .find(|item| {
            let text = normalised(item);
            REPLAY_REASON_MARKERS.iter().any(|m| has_phrase(&text, m))
                && REPLAY_REASON_ANCHORS.iter().any(|a| has_phrase(&text, a))
        })
        .map(|item| {
            item.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .chars()
                .take(REPLAY_REASON_CHARS)
                .collect()
        });
    ReplayDeclaration { share, reason }
}

/// What the reading declares about the replay, from its acceptance criteria and out_of_scope. The gate and the
/// verifier's view both read it from here.
pub fn replay_declaration_of(reading: &Reading) -> ReplayDeclaration {
    replay_declaration(&reading.acceptance_criteria, &reading.out_of_scope)
}

/// Whether `newly` refusals among `total` replayed readings hold the job. With a declared share, more than
/// that share plus [`REPLAY_DECLARED_MARGIN_POINTS`] points holds it; with none, more than
/// [`REPLAY_HOLD_PERCENT`] percent does. Integer math, so exactly the limit is not held.
pub fn replay_is_held(total: usize, newly: usize, declared: Option<u32>) -> bool {
    let limit = declared.map_or(REPLAY_HOLD_PERCENT, |d| {
        d as usize + REPLAY_DECLARED_MARGIN_POINTS
    });
    newly * 100 > total * limit
}

/// The result of one replay.
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayRun {
    pub total: usize,
    /// Refused by the candidate and accepted by the base, newest request first.
    pub newly_refused: Vec<ReplayRefusal>,
    /// Whether a declared share applied to this replay and the job is not held for it.
    pub intended: bool,
    pub held: bool,
    /// Why no replay ran (`no replay: <reason>`). Never a pass.
    pub no_replay: Option<String>,
}

impl ReplayRun {
    /// A replay that did not run, for `reason`.
    pub fn none(reason: &str) -> ReplayRun {
        ReplayRun {
            total: 0,
            newly_refused: vec![],
            intended: false,
            held: false,
            no_replay: Some(format!("no replay: {reason}")),
        }
    }

    /// The record of this replay.
    pub fn record(&self, job: &JobId, base_commit: &str, candidate_commit: &str) -> Replay {
        Replay {
            job: job.clone(),
            base_commit: base_commit.to_string(),
            candidate_commit: candidate_commit.to_string(),
            window: REPLAY_WINDOW as u32,
            total: self.total as u32,
            newly_refused: self.newly_refused.clone(),
            held: self.held,
            intended: self.intended,
            no_replay: self.no_replay.clone(),
        }
    }
}

/// Replays `cases` through the base and the candidate rules and lists the requests the candidate refuses and
/// the base accepted. `declared` is the share of newly refused readings the running job declares
/// ([`ReplayDeclaration::effective_share`]); see [`replay_is_held`]. No cases, or a checker that fails, gives
/// a replay that did not run, never a pass.
pub fn replay(
    cases: &[ReplayCase],
    base: &dyn ReadingChecker,
    candidate: &dyn ReadingChecker,
    declared: Option<u32>,
) -> ReplayRun {
    if cases.is_empty() {
        return ReplayRun::none(NO_HISTORY_REASON);
    }
    let mut newly_refused = Vec::new();
    for case in cases {
        let before = match base.check(&case.request, &case.reading) {
            Ok(check) => check,
            Err(error) => {
                return ReplayRun::none(&format!(
                    "the base build failed on {}: {error:#}",
                    case.job
                ));
            }
        };
        let after = match candidate.check(&case.request, &case.reading) {
            Ok(check) => check,
            Err(error) => {
                return ReplayRun::none(&format!(
                    "the candidate build failed on {}: {error:#}",
                    case.job
                ));
            }
        };
        if let (ReadingCheck::Accept, ReadingCheck::Refuse(reason)) = (before, after) {
            newly_refused.push(ReplayRefusal {
                job: case.job.clone(),
                reason,
            });
        }
    }
    // `Replay::validate` rejects a record that is held and intended, so a declared share that is exceeded is held only.
    let held = replay_is_held(cases.len(), newly_refused.len(), declared);
    ReplayRun {
        total: cases.len(),
        held,
        newly_refused,
        intended: declared.is_some() && !held,
        no_replay: None,
    }
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
    use athena_data::records::{ChangeEntry, ChangeStatus, ProducerReport};

    #[test]
    fn the_reader_is_told_a_job_is_hard_when_it_touches_a_refusing_rule_landing_the_history_check_records_the_pipeline_or_the_drain()
     {
        // Hard marking decides which jobs are planned (`[job_plan] mode = "hard"`), so the definition the reader
        // follows is part of the behavior. It is the description of the `hard_reason` field in the reader's schema.
        let schema = reading_schema(&ReaderChoices {
            classes: vec!["change".into()],
            capabilities: vec!["change".into()],
            producer_models: vec!["m".into()],
        });
        let description = schema["properties"]["hard_reason"]["description"]
            .as_str()
            .expect("hard_reason has a description")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        for area in [
            "rule that can refuse or hold work",
            "landing",
            "the history check",
            "records",
            "the run pipeline",
            "the drain",
        ] {
            assert!(description.contains(area), "{area}: {description}");
        }
    }

    /// A valid reading of one README deliverable, with the given forecast median wall time (seconds).
    fn sized_reading(p50: f64, capability: Option<&str>) -> serde_json::Value {
        serde_json::json!({
            "title": "Describe the demo",
            "summary": "Describe the demo in README.md.",
            "class_of_work": "docs",
            "capability": capability,
            "decline_reason": if capability.is_none() { Some("not a request") } else { None },
            "producer_model": "claude-haiku-5-5",
            "defaults_applied": [{ "id": "strangers", "how": "plain words" }],
            "assumptions": [],
            "question": null,
            "acceptance_criteria": [{ "text": "README.md describes the demo.", "kind": "content" }],
            "deliverables": [{ "path": "README.md", "action": "modify", "p": 0.9 }],
            "out_of_scope": ["Other files"],
            "forecast": {
                "p_reading_accepted": 0.8,
                "p_verification_passes": 0.7,
                "p_user_intervenes": 0.1,
                "cost_usd": { "p10": 0.05, "p50": 0.2, "p90": 0.8 },
                "wall_time_s": { "p10": 5.0, "p50": p50, "p90": p50 * 2.0 },
                "departures": null
            }
        })
    }

    /// Runs the real `build_reading` on `reading` with the shipped configuration, except for the two timeouts.
    fn build_sized(
        reading: serde_json::Value,
        producer_timeout_s: u64,
        verifier_timeout_s: u64,
    ) -> Result<Reading> {
        let repo = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        // `build_reading` never runs an agent, so the script is never called.
        let athena = crate::test_support::make_athena(
            &athena_git::Git::at(repo.path()),
            state.path(),
            vec![],
            Box::new(|_| unreachable!("build_reading runs no agent")),
        );
        let mut jc = JobConfig {
            config: athena_data::Config::parse(include_str!("../../../athena.toml")).unwrap(),
            defaults: athena_data::DefaultsModel::parse(include_str!("../../../defaults.toml"))
                .unwrap(),
            commit: "0".repeat(40),
            config_sha256: String::new(),
            defaults_sha256: String::new(),
        };
        jc.config.budget.producer_timeout_s = producer_timeout_s;
        jc.config.budget.verifier_timeout_s = verifier_timeout_s;
        athena.build_reading(
            &jc,
            &Records::default(),
            &JobId::parse("20261012-084936-n1p0").unwrap(),
            1,
            1,
            reading,
            &["study".to_string()],
            &Redactor::new(&athena_gate::builtin_config()).unwrap(),
        )
    }

    #[test]
    fn a_reading_forecasting_more_than_the_producer_timeout_is_refused_with_advice_to_pick_one_deliverable()
     {
        // Only the producer deadline (3000s) is exceeded by the 5000s median.
        let error = build_sized(sized_reading(5000.0, Some("study")), 3000, 10_000)
            .unwrap_err()
            .to_string();
        assert!(error.contains("pick one deliverable"), "{error}");
        assert!(error.contains("follow-up request"), "{error}");
        assert!(error.contains("producer_timeout_s of 3000s"), "{error}");
    }

    #[test]
    fn a_reading_forecasting_more_than_only_the_verifier_timeout_is_refused() {
        // 2700s is within the producer deadline (3000s) and beyond the verifier deadline (2400s).
        let error = build_sized(sized_reading(2700.0, Some("study")), 3000, 2400)
            .unwrap_err()
            .to_string();
        assert!(error.contains("pick one deliverable"), "{error}");
        assert!(error.contains("verifier_timeout_s of 2400s"), "{error}");
    }

    #[test]
    fn a_reading_within_both_timeouts_is_accepted() {
        for p50 in [30.0, 2400.0] {
            let reading = build_sized(sized_reading(p50, Some("study")), 3000, 2400)
                .unwrap_or_else(|e| panic!("p50 {p50} refused: {e:#}"));
            assert_eq!(reading.forecasts.reader.wall_time_s.p50(), p50);
        }
    }

    #[test]
    fn a_decline_is_not_held_to_the_deadlines() {
        // A decline starts no run, so an oversize forecast does not refuse it.
        let reading = build_sized(sized_reading(5000.0, None), 3000, 2400).unwrap();
        assert!(reading.capability.is_none());
    }

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
    fn an_addendum_priority_puts_the_work_serving_it_in_the_first_plan_wave() {
        use crate::plan::{Plan, Priority};
        use athena_data::roles::Ladder;
        let brief = brief_topics(
            "## Addenda\n\n### 2026-01-01: Invoice export\n\nPriority: urgent\nMake invoice export retries reliable.\n\n\
             ### 2026-01-02: Colour theme\n\nPick a calmer colour theme for reports.\n",
        );
        assert_eq!(brief[0].priority, Priority::Urgent);
        assert_eq!(brief[1].priority, Priority::Normal);
        let entry = |id: &str, text: &str, rung: Ladder| {
            let mut job = job_with_checks(vec![]);
            job.request.as_mut().unwrap().text = format!("{text}\n- it is done");
            let lookup = Lookup {
                job_exists: &|_| false,
                path_exists: &|_| true,
                brief: &brief,
            };
            let mut e = plan_entry(&job, Some(rung), &lookup);
            e.id = id.into();
            e
        };
        // The older items sit on a higher rung; the newest, on the lowest rung, serves the stated priority.
        let entries = vec![
            entry("old-a", "Pick a calmer colour theme", Ladder::OwnerGoals),
            entry(
                "old-b",
                "Pick a calmer colour theme again",
                Ladder::OwnerGoals,
            ),
            entry(
                "new",
                "Retry the invoice export reliably",
                Ladder::Implementation,
            ),
        ];
        assert_eq!(entries[2].priority, Priority::Urgent);
        assert_eq!(entries[0].priority, Priority::Normal);
        let plan = Plan::build(&entries);
        assert_eq!(plan.groups[0], ["new"], "{plan:?}");
        assert!(plan.priority_violations(&entries).is_empty());
        // An explicit priority record is the user's word and is not raised.
        let mut job = job_with_checks(vec![]);
        job.request.as_mut().unwrap().text = "Retry the invoice export reliably".into();
        job.priorities = vec![PriorityRecord {
            job: job.id().clone(),
            n: 1,
            at: crate::now(),
            priority: "low".into(),
        }];
        let lookup = Lookup {
            job_exists: &|_| false,
            path_exists: &|_| true,
            brief: &brief,
        };
        assert_eq!(plan_entry(&job, None, &lookup).priority, Priority::Low);
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

    /// A read job whose reading lists `deliverables` (path, action) and is judged against `exists`.
    fn read_entry(
        deliverables: &[(&str, DeliverableAction)],
        exists: &dyn Fn(&str) -> bool,
    ) -> crate::plan::PlanEntry {
        let mut job = job_with_checks(vec![]);
        job.request.as_mut().unwrap().text = "Change the greeting".into();
        let mut reading = final_reading(&job);
        reading.deliverables = deliverables
            .iter()
            .map(|(path, action)| Deliverable {
                path: (*path).into(),
                action: *action,
                p: 0.9,
            })
            .collect();
        job.readings = vec![reading];
        let lookup = Lookup {
            job_exists: &|_| true,
            path_exists: exists,
            brief: &[],
        };
        plan_entry(
            &job,
            Some(athena_data::roles::Ladder::Implementation),
            &lookup,
        )
    }

    #[test]
    fn a_reading_whose_design_names_no_existing_code_is_refused_at_plan_time() {
        let there = |p: &str| p == "crates/a/src/lib.rs";
        for (what, deliverables) in [
            ("no file at all", vec![]),
            (
                "a path that is not in the repository",
                vec![("crates/gone.rs", DeliverableAction::Modify)],
            ),
        ] {
            let entry = read_entry(&deliverables, &there);
            assert_eq!(entry.missing(), [crate::plan::MISSING_CODE], "{what}");
            let plan = crate::plan::Plan::build(&[entry]);
            assert!(plan.order.is_empty(), "{what}: {plan:?}");
            let reason = &plan.refused[0].reason;
            assert!(reason.contains("existing code"), "{what}: {reason}");
            assert!(reason.contains("the design"), "{what}: {reason}");
        }
    }

    #[test]
    fn a_reading_that_names_existing_code_or_is_wholly_new_passes_the_plan_check() {
        let there = |p: &str| p == "crates/a/src/lib.rs";
        let existing = read_entry(
            &[("crates/a/src/lib.rs", DeliverableAction::Modify)],
            &there,
        );
        assert!(existing.missing().is_empty(), "{:?}", existing.code_problem);
        let new_only = read_entry(&[("crates/b/new.rs", DeliverableAction::Create)], &there);
        assert!(new_only.missing().is_empty(), "{:?}", new_only.code_problem);
        let both = read_entry(
            &[
                ("crates/a/src/lib.rs", DeliverableAction::Modify),
                ("crates/b/new.rs", DeliverableAction::Create),
            ],
            &there,
        );
        assert_eq!(crate::plan::Plan::build(&[both]).order.len(), 1);
    }

    #[test]
    fn a_reading_that_edits_configuration_without_the_schema_is_refused_at_plan_time() {
        use DeliverableAction::{Delete, Modify};
        let all = |_: &str| true;
        let schema = "crates/athena-data/src/config.rs";
        // Config alone, and config beside the wrong crate (the shape of jobs 20261016-071202-q0xy and
        // 20261016-185517-qcrz), are refused with the schema file named.
        for (what, deliverables) in [
            ("athena.toml alone", vec![("athena.toml", Modify)]),
            ("defaults.toml alone", vec![("defaults.toml", Modify)]),
            (
                "the wrong crate",
                vec![
                    ("athena.toml", Modify),
                    ("crates/athena-core/src/config.rs", Modify),
                ],
            ),
            (
                "a deleted schema file",
                vec![("athena.toml", Modify), (schema, Delete)],
            ),
        ] {
            let entry = read_entry(&deliverables, &all);
            assert_eq!(entry.missing(), [crate::plan::MISSING_SCHEMA], "{what}");
            let plan = crate::plan::Plan::build(&[entry]);
            assert!(plan.order.is_empty(), "{what}: {plan:?}");
            let reason = &plan.refused[0].reason;
            assert!(
                reason.contains("crates/athena-data/src/"),
                "{what}: {reason}"
            );
        }
        // With a modified schema path, or with no configuration edit, the plan is accepted.
        for (what, deliverables) in [
            (
                "config and schema",
                vec![("athena.toml", Modify), (schema, Modify)],
            ),
            (
                "config and the schema test",
                vec![
                    ("athena.toml", Modify),
                    ("crates/athena-data/tests/repo_config.rs", Modify),
                ],
            ),
            (
                "no config",
                vec![("crates/athena-core/src/plan.rs", Modify)],
            ),
            (
                "config deleted",
                vec![("athena.toml", Delete), ("a.rs", Modify)],
            ),
        ] {
            let entry = read_entry(&deliverables, &all);
            assert!(
                entry.missing().is_empty(),
                "{what}: {:?}",
                entry.schema_problem
            );
        }
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

    const PRIOR_JOB: &str = "20261012-084936-n1p0";

    /// A reading of a retry: summary and scope are the parts under test.
    fn scoped_reading(
        summary: &str,
        deliverables: &[&str],
        out_of_scope: &[&str],
    ) -> athena_data::records::Reading {
        let forecast = Forecast {
            p_reading_accepted: 0.8,
            p_verification_passes: 0.7,
            p_user_intervenes: 0.1,
            cost_usd: Quantiles::new(0.2, 0.5, 1.5).unwrap(),
            wall_time_s: Quantiles::new(100.0, 300.0, 900.0).unwrap(),
        };
        Reading {
            job: JobId::parse("20261027-054151-ace0").unwrap(),
            n: 1,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: 1,
            base_commit: "a".repeat(40),
            config_sha256: "b".repeat(64),
            defaults_sha256: "c".repeat(64),
            defaults_version: 1,
            title: "Add a greeting".into(),
            summary: summary.into(),
            class_of_work: "feature".into(),
            capability: Some("change".into()),
            decline_reason: None,
            producer_model: "producer".into(),
            verifier_model: "verifier".into(),
            defaults_applied: vec![],
            assumptions: vec![],
            question: None,
            acceptance_criteria: vec![Criterion {
                text: "the greeting prints".into(),
                kind: CriterionKind::Content,
            }],
            deliverables: deliverables
                .iter()
                .map(|path| Deliverable {
                    path: (*path).into(),
                    action: DeliverableAction::Modify,
                    p: 0.9,
                })
                .collect(),
            out_of_scope: out_of_scope.iter().map(|s| s.to_string()).collect(),
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

    fn prior() -> EarlierAttempt {
        EarlierAttempt {
            job: PRIOR_JOB.into(),
            request: "add a greeting".into(),
            findings: vec!["major: the greeting is missing".into()],
            end_state: "rejected".into(),
            verdict: Some("fail".into()),
            ..Default::default()
        }
    }

    #[test]
    fn carry_records_the_end_state_verdict_and_out_of_scope_of_the_earlier_job() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let mut job = job_with_checks(vec![]);
        job.outcome = Some(outcome(&job, OutcomeState::Rejected, "unmet criterion"));
        job.readings.push(scoped_reading(
            "Add a greeting.",
            &["src/greeting.rs", "README.md"],
            &["the changelog"],
        ));
        let carried = carry(&job, &redactor).unwrap().unwrap();
        assert_eq!(carried.end_state, "rejected");
        assert_eq!(carried.verdict.as_deref(), Some("fail"));
        assert_eq!(
            carried.reading_deliverables,
            ["modify src/greeting.rs", "modify README.md"]
        );
        assert_eq!(carried.reading_out_of_scope, ["the changelog"]);
        // A job with no reading carries an empty scope, and the brief still states one.
        let mut bare = job_with_checks(vec![]);
        bare.outcome = Some(outcome(&bare, OutcomeState::Failed, "admission refused"));
        let carried = carry(&bare, &redactor).unwrap().unwrap();
        assert_eq!(carried.end_state, "failed");
        assert!(carried.reading_deliverables.is_empty());
        assert!(carried.reading_out_of_scope.is_empty());
    }

    #[test]
    fn a_complete_reading_of_a_retry_is_accepted() {
        let reading = scoped_reading(
            &format!("Retry {PRIOR_JOB}: fix the missing greeting its verification found."),
            &["src/greeting.rs"],
            &["the changelog"],
        );
        check_prior_result_and_scope(&reading, &[&prior()]).unwrap();
    }

    #[test]
    fn a_reading_that_leaves_out_the_previous_result_is_refused() {
        let reading = scoped_reading("Add a greeting.", &["src/greeting.rs"], &["the changelog"]);
        let error = check_prior_result_and_scope(&reading, &[&prior()])
            .unwrap_err()
            .to_string();
        assert!(error.contains("the previous result"), "{error}");
        assert!(error.contains(PRIOR_JOB), "{error}");
        assert!(!error.contains("the change scope"), "{error}");
    }

    #[test]
    fn a_reading_that_leaves_out_the_change_scope_is_refused() {
        let summary = format!("Retry {PRIOR_JOB}: fix the missing greeting.");
        for (deliverables, out_of_scope, named) in [
            (&[][..], &["the changelog"][..], "no deliverables"),
            (&["src/greeting.rs"][..], &[][..], "no out_of_scope items"),
            (
                &["src/greeting.rs"][..],
                &["  "][..],
                "no out_of_scope items",
            ),
        ] {
            let reading = scoped_reading(&summary, deliverables, out_of_scope);
            let error = check_prior_result_and_scope(&reading, &[&prior()])
                .unwrap_err()
                .to_string();
            assert!(error.contains("the change scope"), "{error}");
            assert!(error.contains(named), "{error}");
            assert!(!error.contains("the previous result"), "{error}");
        }
    }

    #[test]
    fn a_reading_missing_both_names_both() {
        let reading = scoped_reading("Add a greeting.", &[], &[]);
        let error = check_prior_result_and_scope(&reading, &[&prior()])
            .unwrap_err()
            .to_string();
        assert!(error.contains("the previous result"), "{error}");
        assert!(error.contains("the change scope"), "{error}");
    }

    #[test]
    fn a_follow_up_reading_must_name_every_job_it_follows() {
        let mut other = prior();
        other.job = "20261013-000000-abcd".into();
        let reading = scoped_reading(
            &format!("Follow up on {PRIOR_JOB}."),
            &["src/greeting.rs"],
            &["the changelog"],
        );
        let error = check_prior_result_and_scope(&reading, &[&prior(), &other])
            .unwrap_err()
            .to_string();
        assert!(error.contains("20261013-000000-abcd"), "{error}");
        assert!(!error.contains(&format!("job {PRIOR_JOB}")), "{error}");
        let named_both = scoped_reading(
            &format!("Follow up on {PRIOR_JOB} and 20261013-000000-ABCD."),
            &["src/greeting.rs"],
            &["the changelog"],
        );
        check_prior_result_and_scope(&named_both, &[&prior(), &other]).unwrap();
    }

    fn change(path: &str, status: ChangeStatus) -> ChangeEntry {
        ChangeEntry {
            path: path.into(),
            status,
            mode: None,
            bytes: None,
        }
    }

    /// A candidate that changed `paths` (each modified), with the producer's summary when given.
    fn candidate_changing(
        job: &JobRecords,
        paths: &[&str],
        summary: Option<&str>,
    ) -> athena_data::records::Candidate {
        let mut candidate = candidate_of(job);
        candidate.changes = paths
            .iter()
            .map(|p| change(p, ChangeStatus::Modified))
            .collect();
        candidate.producer_report = summary.map(|s| ProducerReport {
            summary: s.into(),
            files: vec![],
            checks_run: vec![],
            criteria_check: vec![],
            open_issues: vec![],
            mistakes: vec![],
            learnings: vec![],
            desires: vec![],
        });
        candidate
    }

    /// Failure shape (a): a job dispatched and abandoned before verification. It has a candidate and no
    /// verification, no outcome and no findings.
    fn abandoned_job(paths: &[&str]) -> JobRecords {
        let mut job = job_with_checks(vec![]);
        job.verification = None;
        job.candidate = Some(candidate_changing(&job, paths, Some("Wrote the parser.")));
        job
    }

    /// Failure shape (b): a job rejected with a finding and a failed check.
    fn rejected_job(paths: &[&str]) -> JobRecords {
        let mut job = job_with_checks(vec![check("test", false, "assertion failed")]);
        job.outcome = Some(outcome(&job, OutcomeState::Rejected, "a check failed"));
        job.candidate = Some(candidate_changing(&job, paths, Some("Wrote the parser.")));
        job
    }

    fn carry_of(job: &JobRecords) -> EarlierAttempt {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        carry(job, &redactor).unwrap().unwrap()
    }

    #[test]
    fn carry_includes_changed_files_approach_and_measured_checks_redacted_and_bounded() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let token = format!(
            "{}{}",
            ["gh", "p_"].concat(),
            "Zq8rT2vK9mW4xN7bP3cH6jL1sD5fG0aYuE2i"
        );
        let mut passed = check("fmt", true, "PASSING-OUTPUT");
        passed.tests_ran = Some(12);
        let mut job = job_with_checks(vec![
            passed,
            check("test", false, "FAILED-OUTPUT-TEXT assertion failed"),
        ]);
        // 35 plain files and one whose name holds a token: 36 in all, one past the cap by 6.
        let mut changes = vec![change("src/new.rs", ChangeStatus::Added)];
        changes.push(change(&format!("src/{token}.rs"), ChangeStatus::Modified));
        changes.push(change("old.md", ChangeStatus::Deleted));
        changes.extend((0..33).map(|i| change(&format!("src/f{i}.rs"), ChangeStatus::Modified)));
        let mut candidate = candidate_changing(
            &job,
            &[],
            Some(&format!(
                "START-OF-SUMMARY key {token}. {}",
                "words ".repeat(400)
            )),
        );
        candidate.changes = changes;
        job.candidate = Some(candidate);

        let carried = carry(&job, &redactor).unwrap().unwrap();
        // The cap: exactly 30 entries, each `status path`, in the order recorded.
        assert_eq!(carried.changed_files.len(), CARRIED_CHANGED_FILES);
        assert_eq!(carried.changed_files[0], "added src/new.rs");
        assert_eq!(carried.changed_files[1], "deleted old.md");
        assert_eq!(carried.changed_files[2], "modified src/f0.rs");
        // The path with the token is dropped, not carried redacted; with the 5 past the cap it is counted.
        assert!(
            !carried.changed_files.iter().any(|f| f.contains("[secret:")),
            "{:?}",
            carried.changed_files
        );
        assert_eq!(carried.changed_files_omitted, 36 - 30);
        // The approach keeps its start, is bounded, and holds no token.
        let approach = carried.approach.as_deref().unwrap();
        assert!(approach.starts_with("START-OF-SUMMARY key "), "{approach}");
        assert!(approach.contains("[secret:github-pat]"), "{approach}");
        assert!(!approach.contains(&token), "{approach}");
        assert!(approach.chars().count() <= CARRIED_APPROACH_CHARS);
        assert!(approach.len() > 600, "the bound keeps most of the text");
        // The measured checks: one line each, pass or fail, exit code and tests, and no output text.
        assert_eq!(
            carried.checks,
            [
                "fmt: passed (exit code 0, 12 tests ran)",
                "test: failed (exit code 101)"
            ]
        );
        let everything = format!("{carried:?}");
        assert!(!everything.contains(&token), "{everything}");
        assert!(!carried.checks.iter().any(|c| c.contains("OUTPUT")));
    }

    #[test]
    fn carry_without_a_candidate_has_no_files_or_approach_but_keeps_the_checks() {
        let job = job_with_checks(vec![check("fmt", true, ""), check("test", false, "x")]);
        assert!(job.candidate.is_none());
        let carried = carry_of(&job);
        assert!(carried.changed_files.is_empty());
        assert_eq!(carried.changed_files_omitted, 0);
        assert_eq!(carried.approach, None);
        assert_eq!(carried.checks.len(), 2);
        // A candidate with no report has files but no approach, and one with a blank summary has none.
        let mut job = job;
        job.candidate = Some(candidate_changing(&job, &["src/a.rs"], None));
        let carried = carry_of(&job);
        assert_eq!(carried.changed_files, ["modified src/a.rs"]);
        assert_eq!(carried.approach, None);
        job.candidate = Some(candidate_changing(&job, &["src/a.rs"], Some("  \n")));
        assert_eq!(carry_of(&job).approach, None);
    }

    #[test]
    fn carry_bounds_the_check_lines() {
        let job = job_with_checks((0..25).map(|i| check(&format!("c{i}"), true, "")).collect());
        let carried = carry_of(&job);
        assert_eq!(carried.checks.len(), CARRIED_CHECK_LINES);
        assert_eq!(carried.checks[0], "c0: passed (exit code 0)");
    }

    const GREETING_SUMMARY: &str = "Retry n1p0 follows";

    /// A reading of the retried job that lists README.md and excludes the changelog, and cites no earlier
    /// file; `PRIOR_JOB` is the id `job_with_checks` gives its jobs.
    fn retry_reading(deliverables: &[&str], out_of_scope: &[&str]) -> Reading {
        let summary = format!("{GREETING_SUMMARY} {PRIOR_JOB}: the parser is rewritten.");
        scoped_reading(&summary, deliverables, out_of_scope)
    }

    #[test]
    fn check_prior_result_refuses_files_the_reading_neither_lists_nor_excludes_for_both_failure_shapes()
     {
        for (shape, job) in [
            (
                "abandoned",
                abandoned_job(&["src/a.rs", "crates/x/src/a.rs"]),
            ),
            ("rejected", rejected_job(&["src/a.rs", "crates/x/src/a.rs"])),
        ] {
            let prior = carry_of(&job);
            assert_eq!(prior.changed_files.len(), 2, "{shape}");
            // The reading names the job id and gives a scope, but says nothing of the two files.
            let reading = retry_reading(&["README.md"], &["the changelog"]);
            let error = check_prior_result_and_scope(&reading, &[&prior])
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("the earlier job's files"),
                "{shape}: {error}"
            );
            assert!(error.contains(PRIOR_JOB), "{shape}: {error}");
            assert!(error.contains("src/a.rs"), "{shape}: {error}");
            assert!(error.contains("crates/x/src/a.rs"), "{shape}: {error}");
            assert!(!error.contains("the previous result"), "{shape}: {error}");
            assert!(!error.contains("the change scope"), "{shape}: {error}");
            // Accepted when the reading lists the files, or a directory deliverable covers them, or an
            // out_of_scope item names the paths or a directory above them.
            for (deliverables, out_of_scope) in [
                (
                    &["README.md", "src/a.rs", "crates/x/src/a.rs"][..],
                    &["the changelog"][..],
                ),
                (
                    &["README.md", "src", "crates/x/"][..],
                    &["the changelog"][..],
                ),
                (
                    &["README.md"][..],
                    &["Leave `src/a.rs` and crates/x/src/a.rs. alone."][..],
                ),
                (
                    &["README.md"][..],
                    &["Everything under `src/` and crates/x/ stays as the job left it."][..],
                ),
                (
                    &["README.md", "src/a.rs"][..],
                    &["no change to (crates/x/src/a.rs)"][..],
                ),
            ] {
                let reading = retry_reading(deliverables, out_of_scope);
                check_prior_result_and_scope(&reading, &[&prior])
                    .unwrap_or_else(|e| panic!("{shape}: {deliverables:?} {out_of_scope:?}: {e}"));
            }
        }
    }

    #[test]
    fn check_prior_result_does_not_count_survey_prose_or_a_sibling_path() {
        let prior = carry_of(&rejected_job(&["crates/athena-core/src/a.rs", "src/a.rs"]));
        // A survey in the assumptions and the summary cites crates/ and a sibling file; neither accounts for
        // the unlisted file.
        let mut reading = retry_reading(&["README.md"], &["the changelog"]);
        reading
            .summary
            .push_str(" See crates/ and crates/athena-core/src/lib.rs.");
        reading.assumptions = vec![
            "Code survey: read crates/athena-core/src/lib.rs, src/a.rs and crates/ in full.".into(),
        ];
        let error = check_prior_result_and_scope(&reading, &[&prior])
            .unwrap_err()
            .to_string();
        assert!(error.contains("crates/athena-core/src/a.rs"), "{error}");
        assert!(error.contains("src/a.rs"), "{error}");
        // A sibling file in out_of_scope does not account for the other file.
        let reading = retry_reading(
            &["README.md"],
            &["crates/athena-core/src/lib.rs stays; so does crates/x/src/a.rs"],
        );
        let error = check_prior_result_and_scope(&reading, &[&prior])
            .unwrap_err()
            .to_string();
        assert!(error.contains("crates/athena-core/src/a.rs"), "{error}");
        assert!(error.contains("src/a.rs"), "{error}");
        // `crates/x/src/a.rs` does not account for `src/a.rs`: a path inside a longer path is not a token.
        let only = carry_of(&rejected_job(&["src/a.rs"]));
        let reading = retry_reading(&["README.md"], &["crates/x/src/a.rs stays"]);
        assert!(check_prior_result_and_scope(&reading, &[&only]).is_err());
        // A criterion naming the file does not account for it either: only deliverables and out_of_scope do.
        let mut reading = retry_reading(&["README.md"], &["the changelog"]);
        reading.acceptance_criteria[0].text = "src/a.rs is rewritten".into();
        assert!(check_prior_result_and_scope(&reading, &[&only]).is_err());
    }

    #[test]
    fn path_tokens_match_only_on_whole_path_boundaries() {
        for (item, token, found) in [
            ("src/a.rs", "src/a.rs", true),
            ("keep src/a.rs.", "src/a.rs", true),
            ("keep src/a.rs. Next", "src/a.rs", true),
            ("keep src/a.rs!", "src/a.rs", true),
            ("keep `src/a.rs`, too", "src/a.rs", true),
            ("keep \"src/a.rs\"", "src/a.rs", true),
            ("keep 'src/a.rs'", "src/a.rs", true),
            ("keep (src/a.rs)", "src/a.rs", true),
            ("keep [src/a.rs]", "src/a.rs", true),
            ("keep {src/a.rs}", "src/a.rs", true),
            ("keep a,src/a.rs;b", "src/a.rs", true),
            ("src/a.rs: kept", "src/a.rs", true),
            ("everything in src/* stays", "src/", true),
            ("everything in src/ stays", "src/", true),
            ("src/a", "src/a.rs", false),
            ("src/a.rs", "src/a", false),
            ("src/a.rsx", "src/a.rs", false),
            ("src/a.rs.bak", "src/a.rs", false),
            ("crates/x/src/a.rs", "src/a.rs", false),
            ("xsrc/a.rs", "src/a.rs", false),
            ("src/a.rs/more", "src/a.rs", false),
            ("keep/src/a.rs", "src/a.rs", false),
            ("src/a.rs", "", false),
        ] {
            assert_eq!(names_path_token(item, token), found, "{item:?} {token:?}");
        }
        // The same token twice: the second, well-bounded one is found after the first fails.
        assert!(names_path_token("xsrc/a.rs and src/a.rs", "src/a.rs"));
    }

    #[test]
    fn check_prior_result_ignores_a_file_whose_name_redaction_altered() {
        let token = format!(
            "{}{}",
            ["gh", "p_"].concat(),
            "Zq8rT2vK9mW4xN7bP3cH6jL1sD5fG0aYuE2i"
        );
        let prior = carry_of(&rejected_job(&[&format!("src/{token}.rs")]));
        assert!(prior.changed_files.is_empty());
        assert_eq!(prior.changed_files_omitted, 1);
        let reading = retry_reading(&["README.md"], &["the changelog"]);
        check_prior_result_and_scope(&reading, &[&prior]).unwrap();
    }

    #[test]
    fn check_prior_result_names_only_the_prior_with_unaccounted_files() {
        let mut other = carry_of(&abandoned_job(&["docs/b.md"]));
        other.job = "20261013-000000-abcd".into();
        let listed = carry_of(&rejected_job(&["src/a.rs"]));
        let reading = scoped_reading(
            &format!("Follow up on {PRIOR_JOB} and 20261013-000000-abcd."),
            &["src/a.rs"],
            &["the changelog"],
        );
        let error = check_prior_result_and_scope(&reading, &[&listed, &other])
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("job 20261013-000000-abcd changed docs/b.md"),
            "{error}"
        );
        assert!(
            !error.contains("job 20261012-084936-n1p0 changed"),
            "{error}"
        );
        // More than five files are named five at a time, with the count of the rest.
        let many: Vec<String> = (0..8).map(|i| format!("lib/f{i}.rs")).collect();
        let many: Vec<&str> = many.iter().map(String::as_str).collect();
        let prior = carry_of(&rejected_job(&many));
        let reading = retry_reading(&["README.md"], &["the changelog"]);
        let error = check_prior_result_and_scope(&reading, &[&prior])
            .unwrap_err()
            .to_string();
        assert!(error.contains("lib/f4.rs and 3 more"), "{error}");
        assert!(!error.contains("lib/f5.rs"), "{error}");
    }

    #[test]
    fn check_prior_result_refuses_only_priors_with_recorded_files_the_reading_ignores() {
        // One fixed reading: it names the job, lists README.md, excludes CHANGELOG.md and cites no other file.
        let reading = retry_reading(&["README.md"], &["CHANGELOG.md stays as it is"]);
        let mut no_changes = abandoned_job(&[]);
        no_changes.candidate = Some(candidate_changing(&no_changes, &[], None));
        let mut no_candidate = abandoned_job(&["src/a.rs"]);
        no_candidate.candidate = None;
        let rows: [(&str, JobRecords, bool); 6] = [
            ("no candidate", no_candidate, false),
            ("a candidate with no changes", no_changes, false),
            (
                "files the reading lists",
                abandoned_job(&["README.md"]),
                false,
            ),
            ("files it excludes", rejected_job(&["CHANGELOG.md"]), false),
            (
                "files it ignores, abandoned",
                abandoned_job(&["src/a.rs"]),
                true,
            ),
            (
                "files it ignores, rejected",
                rejected_job(&["src/a.rs"]),
                true,
            ),
        ];
        for (row, job, refused) in rows {
            let prior = carry_of(&job);
            let result = check_prior_result_and_scope(&reading, &[&prior]);
            assert_eq!(result.is_err(), refused, "{row}: {result:?}");
            if let Err(error) = result {
                assert!(
                    error.to_string().contains("the earlier job's files"),
                    "{row}"
                );
            }
        }
    }

    #[test]
    fn prior_result_refusal_is_declared_intended() {
        // The share parses as 5, below the 10% a replay holds an undeclared check at, so no reason is needed.
        let criterion = Criterion {
            text: "The change refuses more readings: retries whose prior job changed files that the reading \
                   neither lists nor excludes are refused, and the higher refusal rate is intended: expected 5% \
                   of newly refused readings."
                .into(),
            kind: CriterionKind::Content,
        };
        let declaration = replay_declaration(&[criterion], &[]);
        assert_eq!(declaration.share, Some(5));
        // The check's doc comment and the changelog carry the same sentences.
        let source = include_str!("reading.rs");
        let doc = source
            .split("pub fn check_prior_result_and_scope")
            .next()
            .unwrap()
            .rsplit("\n\n")
            .next()
            .unwrap();
        assert!(
            doc.contains("This check refuses more readings; the higher refusal rate is intended.")
        );
        assert!(
            doc.replace("\n/// ", " ")
                .contains("The replay passes no earlier attempts"),
            "the replay cannot see this check"
        );
        let changelog = include_str!("../../../CHANGELOG.md")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        for phrase in [
            "refuses more readings",
            "higher refusal rate is intended",
            "expected 5% of newly refused readings",
            "20261027-054209-z7sj",
            "check_prior_result_and_scope",
            "prior_result_section",
        ] {
            assert!(changelog.contains(phrase), "CHANGELOG.md lacks {phrase:?}");
        }
    }

    const EVIDENCE_A: &str = "20261020-101010-aaaa";
    const EVIDENCE_B: &str = "20261021-111111-bbbb";

    fn earlier(job: &str) -> EarlierAttempt {
        EarlierAttempt {
            job: job.into(),
            request: "an earlier request".into(),
            findings: vec!["major: something was found".into()],
            end_state: "rejected".into(),
            ..Default::default()
        }
    }

    /// What `read_for` checks for a request: the jobs it continues, then the reading against them.
    fn check_reading(
        request: &str,
        attempt: Option<&EarlierAttempt>,
        referenced: &[EarlierAttempt],
        reading: &Reading,
    ) -> Result<()> {
        check_reading_acceptance(reading, request, attempt, referenced, None)
    }

    #[test]
    fn a_root_cause_request_citing_past_jobs_as_evidence_is_accepted() {
        let request = format!(
            "Root cause of the repeated refusal. Evidence: the introspection of {EVIDENCE_A} \
             and the introspection of {EVIDENCE_B} both record it."
        );
        let referenced = [earlier(EVIDENCE_A), earlier(EVIDENCE_B)];
        // No previous_job and no follow-up wording: neither cited job is continued.
        assert!(followed_up_ids(&request).is_empty());
        assert!(continued_jobs(&request, None, &referenced).is_empty());
        // The reading restates neither job's result, nor does it state a change scope.
        let reading = scoped_reading("Scope the check to the continued job.", &[], &[]);
        check_reading(&request, None, &referenced, &reading).unwrap();
        // The cited jobs would have been refused when checked as continued jobs.
        let error = check_prior_result_and_scope(&reading, &[&referenced[0], &referenced[1]])
            .unwrap_err()
            .to_string();
        assert!(error.contains("the previous result"), "{error}");
    }

    #[test]
    fn a_retry_whose_reading_omits_the_previous_result_is_refused() {
        let request = "add a greeting";
        let previous = prior();
        let reading = scoped_reading("Add a greeting.", &["src/greeting.rs"], &["the changelog"]);
        let error = check_reading(request, Some(&previous), &[], &reading)
            .unwrap_err()
            .to_string();
        assert!(error.contains("the previous result"), "{error}");
        assert!(error.contains(PRIOR_JOB), "{error}");
    }

    #[test]
    fn a_retry_whose_reading_names_the_previous_job_is_accepted() {
        let request = "add a greeting";
        let previous = prior();
        let reading = scoped_reading(
            &format!("Retry {PRIOR_JOB}: fix the missing greeting its verification found."),
            &["src/greeting.rs"],
            &["the changelog"],
        );
        check_reading(request, Some(&previous), &[], &reading).unwrap();
    }

    #[test]
    fn a_retry_is_checked_even_when_the_request_also_cites_evidence() {
        let request = format!("Retry, and note that {EVIDENCE_A} saw the same fault.");
        let previous = prior();
        let referenced = [earlier(EVIDENCE_A)];
        let reading = scoped_reading(
            &format!("Retry {PRIOR_JOB}: fix the missing greeting."),
            &["src/greeting.rs"],
            &["the changelog"],
        );
        check_reading(&request, Some(&previous), &referenced, &reading).unwrap();
        let bare = scoped_reading("Add a greeting.", &["src/greeting.rs"], &["the changelog"]);
        let error = check_reading(&request, Some(&previous), &referenced, &bare)
            .unwrap_err()
            .to_string();
        assert!(error.contains(PRIOR_JOB), "{error}");
        assert!(!error.contains(EVIDENCE_A), "{error}");
    }

    #[test]
    fn a_follow_up_request_is_checked_for_the_followed_job_only() {
        let request = format!(
            "This follows up job {PRIOR_JOB}. Evidence: {EVIDENCE_A} recorded the same fault."
        );
        assert_eq!(
            followed_up_ids(&request)
                .iter()
                .map(RecordId::as_str)
                .collect::<Vec<_>>(),
            [PRIOR_JOB]
        );
        let referenced = [earlier(EVIDENCE_A), prior()];
        let jobs = continued_jobs(&request, None, &referenced);
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].job, PRIOR_JOB);
        // A reading that omits the followed-up job is refused, and names only that job.
        let omits = scoped_reading("Add a greeting.", &["src/greeting.rs"], &["the changelog"]);
        let error = check_reading(&request, None, &referenced, &omits)
            .unwrap_err()
            .to_string();
        assert!(error.contains("the previous result"), "{error}");
        assert!(error.contains(PRIOR_JOB), "{error}");
        assert!(!error.contains(EVIDENCE_A), "{error}");
        // Naming the followed-up job is enough; the evidence job need not be named.
        let names = scoped_reading(
            &format!("Follow up on {PRIOR_JOB}: add the missing greeting."),
            &["src/greeting.rs"],
            &["the changelog"],
        );
        check_reading(&request, None, &referenced, &names).unwrap();
    }

    #[test]
    fn follow_up_wording_is_matched_narrowly() {
        let id = PRIOR_JOB;
        for text in [
            format!("follows up {id}"),
            format!("Follow-up of {id}"),
            format!("a follow up on job {id}"),
            format!("Following up on the earlier job {id}."),
        ] {
            assert_eq!(followed_up_ids(&text).len(), 1, "{text}");
        }
        // An id that merely follows other words, or a phrase that names no id, is not a follow-up.
        for text in [
            format!("see {id} and follow the evidence"),
            format!("follows up the cause; see {id}"),
            format!("Follow up carefully, citing {id}"),
            "follows up".to_string(),
        ] {
            assert!(followed_up_ids(&text).is_empty(), "{text}");
        }
    }

    #[test]
    fn a_first_attempt_reading_is_not_checked() {
        let reading = scoped_reading("Add a greeting.", &[], &[]);
        check_prior_result_and_scope(&reading, &[]).unwrap();
    }

    #[test]
    fn a_declined_reading_or_a_question_has_no_scope_to_check() {
        let mut declined = scoped_reading("Add a greeting.", &[], &[]);
        declined.capability = None;
        declined.decline_reason = Some("no capability fits".into());
        check_prior_result_and_scope(&declined, &[&prior()]).unwrap();
        let mut asking = scoped_reading("Add a greeting.", &[], &[]);
        asking.question = Some("which file?".into());
        check_prior_result_and_scope(&asking, &[&prior()]).unwrap();
    }

    /// A reading with a single acceptance criterion of this text.
    fn reading_with_criterion(text: &str) -> Reading {
        let mut reading =
            scoped_reading("Add a greeting.", &["src/greeting.rs"], &["the changelog"]);
        reading.acceptance_criteria = vec![Criterion {
            text: text.into(),
            kind: CriterionKind::Behavior,
        }];
        reading
    }

    #[test]
    fn a_criterion_resting_on_the_producers_report_is_refused_and_named() {
        let text = "the producer's report lists the caller-check Grep";
        let error = check_criteria_evidence(&reading_with_criterion(text))
            .unwrap_err()
            .to_string();
        assert!(error.contains(text), "{error}");
        assert!(error.contains("criterion 1"), "{error}");
        assert!(
            error.contains("a command or Grep the verifier can run"),
            "{error}"
        );
        assert!(error.contains("file in the snapshot"), "{error}");
    }

    #[test]
    fn every_phrase_family_that_cites_the_producers_own_account_is_refused() {
        for text in [
            "the report lists the caller-check Grep",
            "The producer states that no caller uses the old path",
            "the producer confirms the tests were run",
            "the producer\u{2019}s summary names every changed file",
            "the Producer's claims match the diff",
            "a producer report names the Grep it ran",
            "the producer\nsays the Grep returned no hits",
            "the criteria check is reported by the producer as met",
            "the old module is gone, according to the producer",
        ] {
            let error = check_criteria_evidence(&reading_with_criterion(text))
                .unwrap_err()
                .to_string();
            assert!(error.contains("was refused"), "{text}: {error}");
        }
    }

    #[test]
    fn every_refused_criterion_is_named_and_the_others_are_not() {
        let mut reading = reading_with_criterion("cargo test --locked passes");
        reading.acceptance_criteria.extend([
            Criterion {
                text: "the report shows the Grep".into(),
                kind: CriterionKind::Content,
            },
            Criterion {
                text: "a Grep of crates/ for `old::` returns no hits".into(),
                kind: CriterionKind::Behavior,
            },
            Criterion {
                text: "the producer confirms the move".into(),
                kind: CriterionKind::Content,
            },
        ]);
        let error = check_criteria_evidence(&reading).unwrap_err().to_string();
        assert!(error.contains("criterion 2"), "{error}");
        assert!(error.contains("criterion 4"), "{error}");
        assert!(!error.contains("criterion 1"), "{error}");
        assert!(!error.contains("criterion 3"), "{error}");
    }

    #[test]
    fn criteria_naming_a_command_a_grep_or_a_file_are_accepted() {
        for text in [
            "a Grep of crates/ for `gh::` outside gh/ returns no hits",
            "cargo test --locked passes",
            "Run `cargo test --locked -p athena-core reading`: the new test fails without the change",
            "CHANGELOG.md names the root cause",
            "README.md has a Quick start section",
            "the doc comment on the check names the cause",
            "the error message names the criterion and tells the reader to restate it",
            "the producer model is on the allowed list",
            "the summary section of docs/design/requirements-map.md is unchanged",
            "the report page from `cargo run -p xtask -- site` contains the glossary",
        ] {
            check_criteria_evidence(&reading_with_criterion(text))
                .unwrap_or_else(|e| panic!("{text}: {e}"));
        }
    }

    #[test]
    fn a_reading_that_declines_has_no_criterion_to_check() {
        let mut reading = reading_with_criterion("the producer's report lists the Grep");
        reading.capability = None;
        reading.decline_reason = Some("no capability fits".into());
        check_criteria_evidence(&reading).unwrap();
    }

    /// A reading that lists these deliverables, with one criterion per text and these out-of-scope items.
    fn governed_reading(
        deliverables: &[&str],
        criteria: &[&str],
        out_of_scope: &[&str],
    ) -> Reading {
        let mut reading = scoped_reading("Do the work.", deliverables, out_of_scope);
        reading.acceptance_criteria = criteria
            .iter()
            .map(|text| Criterion {
                text: (*text).into(),
                kind: CriterionKind::Content,
            })
            .collect();
        reading
    }

    /// A request phrasing that names a governance path, and the path the reading must cover.
    const NAMED_PATH_REQUESTS: [(&str, &str); 9] = [
        (
            "Publish to GitHub Pages from .github/workflows/pages.yml",
            ".github/workflows/",
        ),
        (
            "Run the tests in .github/workflows/ci.yml",
            ".github/workflows/",
        ),
        ("Add a job under .github/workflows", ".github/workflows/"),
        ("Change the routing in athena.toml", "athena.toml"),
        ("Raise the budgets in `athena.toml`.", "athena.toml"),
        ("Add a [gate] table to ATHENA.TOML", "athena.toml"),
        ("Change defaults.toml to add a rule", "defaults.toml"),
        (
            "Pin the version in rust-toolchain.toml",
            "rust-toolchain.toml",
        ),
        ("Change what setup.sh installs", "setup.sh"),
    ];

    /// Requests that use the common words of the old keyword table and name no governance path.
    const COMMON_WORD_REQUESTS: [&str; 14] = [
        "Publish to GitHub Pages",
        "Deploy the site on every push",
        "Add a workflow that builds the docs",
        "Run the tests in CI",
        "Add a CI job for the book",
        "Change the routing of small items",
        "Add a gate before merging",
        "Raise the budgets for retries",
        "Add new check classes for docs",
        "Make the checks configurable",
        "Add a configuration section for audiences",
        "Add a config option for the deployment gate",
        "Pin the version in rust-toolchain",
        "Explain how the gates and budgets work",
    ];

    /// A concrete governance file for a keyword path.
    fn concrete(path: &str) -> String {
        if path.ends_with('/') {
            format!("{path}pages.yml")
        } else {
            path.to_string()
        }
    }

    #[test]
    fn a_request_naming_a_governance_path_is_refused_until_it_is_covered() {
        for (request, path) in NAMED_PATH_REQUESTS {
            let file = concrete(path);
            // Without the file listed, the reading is refused, naming the path and the repair.
            let bare = governed_reading(&["src/site.rs"], &["the site builds"], &["the logo"]);
            let error = check_reading_acceptance(&bare, request, None, &[], None)
                .unwrap_err()
                .to_string();
            assert!(error.contains(path), "{request}: {error}");
            assert!(error.contains("as a deliverable"), "{request}: {error}");
            assert!(
                error.contains("in an acceptance criterion"),
                "{request}: {error}"
            );
            // Listed but not named in a criterion: still refused.
            let unnamed = governed_reading(&["src/site.rs", &file], &["the site builds"], &["x"]);
            assert!(
                check_reading_acceptance(&unnamed, request, None, &[], None).is_err(),
                "{request}: listed but unnamed"
            );
            // Listed and named: accepted.
            let named = governed_reading(
                &["src/site.rs", &file],
                &["the site builds", &format!("{file} runs the build")],
                &["the logo"],
            );
            check_reading_acceptance(&named, request, None, &[], None)
                .unwrap_or_else(|e| panic!("{request}: {e}"));
            // Put out of scope by name: accepted without the file.
            let excluded = governed_reading(
                &["src/site.rs"],
                &["the site builds"],
                &[&format!("Editing {}", path.trim_end_matches('/'))],
            );
            check_reading_acceptance(&excluded, request, None, &[], None)
                .unwrap_or_else(|e| panic!("{request}: {e}"));
        }
    }

    #[test]
    fn a_request_with_only_common_words_and_no_governance_path_is_accepted() {
        // None of these words names a path, so none requires a governance file; the reading lists none.
        let bare = governed_reading(&["src/site.rs"], &["the site builds"], &["the logo"]);
        for request in COMMON_WORD_REQUESTS {
            check_governance_coverage(&bare, request).unwrap_or_else(|e| panic!("{request}: {e}"));
            check_reading_acceptance(&bare, request, None, &[], None)
                .unwrap_or_else(|e| panic!("{request}: {e}"));
        }
    }

    #[test]
    fn a_request_naming_a_workflow_file_needs_it_listed_and_named() {
        let request = "Publish to GitHub Pages from .github/workflows/pages.yml";
        let bare = governed_reading(&["docs/index.md"], &["the page renders"], &["other pages"]);
        let error = check_reading_acceptance(&bare, request, None, &[], None)
            .unwrap_err()
            .to_string();
        assert!(error.contains(".github/workflows"), "{error}");
        assert!(error.contains("names"), "{error}");
        let with_workflow = governed_reading(
            &["docs/index.md", ".github/workflows/pages.yml"],
            &["pages.yml deploys the site"],
            &[],
        );
        check_reading_acceptance(&with_workflow, request, None, &[], None).unwrap();
    }

    #[test]
    fn a_governance_path_that_only_looks_like_a_name_is_not_named() {
        let bare = governed_reading(&["src/site.rs"], &["the site builds"], &["x"]);
        for request in [
            "rename my_athena.toml_helper",
            "fix xsetup.sh",
            "the athena.tomls file",
            "see .github/workflowsx",
        ] {
            check_governance_coverage(&bare, request).unwrap_or_else(|e| panic!("{request}: {e}"));
        }
    }

    #[test]
    fn a_deliverable_that_is_a_governance_file_is_still_refused_without_a_criterion_naming_it() {
        // Whatever the request says, and for the deliverable table too: these two refusals remain.
        let request = "add a greeting";
        for path in [".github/workflows/ci.yml", "defaults.toml", "setup.sh"] {
            let unnamed = governed_reading(&[path], &["the greeting prints"], &[]);
            let error = check_governance_coverage(&unnamed, request)
                .unwrap_err()
                .to_string();
            assert!(error.contains(path), "{error}");
            assert!(error.contains("no criterion names it"), "{error}");
        }
        let config = "crates/athena-data/src/config.rs";
        let bare = governed_reading(&[config], &["the setting parses"], &[]);
        let error = check_governance_coverage(&bare, request)
            .unwrap_err()
            .to_string();
        assert!(error.contains("athena.toml"), "{error}");
        assert_eq!(
            GOVERNANCE_BY_DELIVERABLE,
            [(
                "crates/athena-data/src/config.rs",
                "athena.toml",
                "a new or changed configuration section must be set in athena.toml, or its schema test fails",
            )]
        );
    }

    #[test]
    fn a_listed_governance_file_must_be_named_by_a_criterion_whatever_the_request_says() {
        let request = "add a greeting";
        let unnamed = governed_reading(&["athena.toml"], &["the greeting prints"], &[]);
        let error = check_reading_acceptance(&unnamed, request, None, &[], None)
            .unwrap_err()
            .to_string();
        assert!(error.contains("athena.toml"), "{error}");
        assert!(error.contains("no criterion names it"), "{error}");
        let named = governed_reading(&["athena.toml"], &["athena.toml sets the greeting"], &[]);
        check_reading_acceptance(&named, request, None, &[], None).unwrap();
    }

    #[test]
    fn a_new_config_section_needs_athena_toml_listed_and_named() {
        let request = "add an audience setting";
        let config = "crates/athena-data/src/config.rs";
        let without = governed_reading(&[config], &["the setting parses"], &[]);
        let error = check_reading_acceptance(&without, request, None, &[], None)
            .unwrap_err()
            .to_string();
        assert!(error.contains("athena.toml"), "{error}");
        assert!(error.contains(config), "{error}");
        let with = governed_reading(
            &[config, "athena.toml"],
            &["the setting parses", "athena.toml sets audience"],
            &[],
        );
        check_reading_acceptance(&with, request, None, &[], None).unwrap();
        let excluded = governed_reading(&[config], &["the setting parses"], &["athena.toml"]);
        check_reading_acceptance(&excluded, request, None, &[], None).unwrap();
    }

    #[test]
    fn a_request_without_governance_keywords_or_deliverables_is_accepted_unchanged() {
        let reading = governed_reading(&["src/greeting.rs"], &["the greeting prints"], &["x"]);
        check_reading_acceptance(&reading, "add a greeting", None, &[], None).unwrap();
        // A word that only contains a keyword, or an identifier that joins it, is not a match.
        for request in [
            "fix the circle area",
            "rename athena_gate_helper",
            "the pagesize",
        ] {
            check_reading_acceptance(&reading, request, None, &[], None)
                .unwrap_or_else(|e| panic!("{request}: {e}"));
        }
        // A reading that declines, asks, or changes nothing has no governance file to cover.
        let mut declined = governed_reading(&["src/site.rs"], &[], &[]);
        declined.capability = None;
        declined.decline_reason = Some("no capability fits".into());
        let request = "Publish to GitHub Pages from .github/workflows/pages.yml";
        check_reading_acceptance(&declined, request, None, &[], None).unwrap();
        let mut asking = governed_reading(&["src/site.rs"], &[], &[]);
        asking.question = Some("which file?".into());
        check_reading_acceptance(&asking, request, None, &[], None).unwrap();
        let empty = governed_reading(&[], &["x"], &[]);
        check_reading_acceptance(&empty, request, None, &[], None).unwrap();
    }

    #[test]
    fn the_governance_check_applies_to_a_retry_as_to_a_first_attempt() {
        let request = "Publish to GitHub Pages from .github/workflows/pages.yml";
        let previous = prior();
        let mut retry = governed_reading(&["docs/index.md"], &["the page renders"], &["x"]);
        retry.summary = format!("Retry {PRIOR_JOB}: add the missing workflow.");
        let error = check_reading_acceptance(&retry, request, Some(&previous), &[], None)
            .unwrap_err()
            .to_string();
        assert!(error.contains(".github/workflows"), "{error}");
        assert!(!error.contains("the previous result"), "{error}");
    }

    #[test]
    fn the_acceptance_check_refuses_a_first_attempt_and_a_retry_alike() {
        let bad = "the producer's report lists the caller-check Grep";
        // A first attempt: no earlier job, so the previous-result check passes and the evidence check decides.
        let reading = reading_with_criterion(bad);
        let error = check_reading_acceptance(&reading, "add a greeting", None, &[], None)
            .unwrap_err()
            .to_string();
        assert!(error.contains(bad), "{error}");
        // A retry whose reading names the previous job and states its scope is still refused for the criterion.
        let previous = prior();
        let mut retry = reading_with_criterion(bad);
        retry.summary = format!("Retry {PRIOR_JOB}: fix the missing greeting.");
        let error = check_reading_acceptance(&retry, "add a greeting", Some(&previous), &[], None)
            .unwrap_err()
            .to_string();
        assert!(error.contains(bad), "{error}");
        assert!(!error.contains("the previous result"), "{error}");
        // The same two readings with a criterion the verifier can run are accepted.
        let good = "a Grep of crates/ for `old::` returns no hits";
        check_reading_acceptance(
            &reading_with_criterion(good),
            "add a greeting",
            None,
            &[],
            None,
        )
        .unwrap();
        let mut retry = reading_with_criterion(good);
        retry.summary = format!("Retry {PRIOR_JOB}: fix the missing greeting.");
        check_reading_acceptance(&retry, "add a greeting", Some(&previous), &[], None).unwrap();
    }

    // -- The boundary words and the repository's files

    /// A banned word, built from pieces so this file passes the boundary scan it tests.
    fn banned_word() -> String {
        ["oper", "ator"].concat()
    }

    /// A banned path fragment, built from pieces for the same reason.
    fn banned_fragment() -> String {
        ["/ho", "me/user"].concat()
    }

    fn files_at_head() -> BTreeSet<String> {
        ["README.md", "src/lib.rs", "src/main.rs"]
            .iter()
            .map(|p| p.to_string())
            .collect()
    }

    /// A clean change reading with one deliverable, whose action and path the caller sets.
    fn reading_with_deliverable(path: &str, action: DeliverableAction) -> Reading {
        let mut reading = governed_reading(&[path], &["the greeting prints"], &["the logo"]);
        reading.deliverables[0].action = action;
        reading
    }

    #[test]
    fn a_reading_whose_summary_holds_a_boundary_word_is_refused_and_the_word_is_named() {
        let word = banned_word();
        let mut reading = governed_reading(&["README.md"], &["the greeting prints"], &["x"]);
        reading.summary = format!("Ask the {word} to approve it.");
        let error = check_reading_acceptance(&reading, "add a greeting", None, &[], None)
            .unwrap_err()
            .to_string();
        assert!(error.contains(&word), "{error}");
        assert!(error.contains("summary"), "{error}");
        // The same on the replay path and with a file set: the words do not depend on the repository.
        assert!(matches!(
            check_reading_text(&reading, "add a greeting"),
            ReadingCheck::Refuse(_)
        ));
        let files = files_at_head();
        let error = check_reading_acceptance(&reading, "add a greeting", None, &[], Some(&files))
            .unwrap_err()
            .to_string();
        assert!(error.contains(&word), "{error}");
        // Without the word the same reading is accepted.
        reading.summary = "Ask the owner to approve it.".into();
        check_reading_acceptance(&reading, "add a greeting", None, &[], Some(&files)).unwrap();
        assert!(matches!(
            check_reading_text(&reading, "add a greeting"),
            ReadingCheck::Accept
        ));
    }

    #[test]
    fn a_boundary_word_in_any_text_field_or_path_is_refused_with_the_field_named() {
        let word = banned_word();
        let fragment = banned_fragment();
        type Edit = fn(&mut Reading, String);
        let edits: [(&str, Edit); 9] = [
            ("title", |r, t| r.title = t),
            ("summary", |r, t| r.summary = t),
            ("decline_reason", |r, t| r.decline_reason = Some(t)),
            ("question", |r, t| r.question = Some(t)),
            ("departures", |r, t| r.departures = Some(t)),
            ("assumptions[0]", |r, t| r.assumptions = vec![t]),
            ("out_of_scope[0]", |r, t| r.out_of_scope = vec![t]),
            ("acceptance_criteria[0]", |r, t| {
                r.acceptance_criteria[0].text = t
            }),
            ("defaults_applied[0]", |r, t| {
                r.defaults_applied = vec![DefaultApplied {
                    id: "tested".into(),
                    how: t,
                }]
            }),
        ];
        for (field, edit) in edits {
            for text in [format!("The {word} decides."), format!("See {fragment}/x.")] {
                let mut reading =
                    governed_reading(&["README.md"], &["the greeting prints"], &["x"]);
                edit(&mut reading, text.clone());
                let error = check_reading_acceptance(&reading, "add a greeting", None, &[], None)
                    .unwrap_err()
                    .to_string();
                assert!(error.contains(field), "{field}: {error}");
                let term = if text.contains(&word) {
                    &word
                } else {
                    &fragment
                };
                assert!(error.contains(term.as_str()), "{field}: {error}");
            }
        }
        // A deliverable path is scanned too, and the error names it.
        let path = format!("docs/{word}.md");
        let reading = reading_with_deliverable(&path, DeliverableAction::Create);
        let error = check_reading_acceptance(&reading, "add a greeting", None, &[], None)
            .unwrap_err()
            .to_string();
        assert!(error.contains(&path) && error.contains(&word), "{error}");
        // Words that only contain a banned word are not hits.
        let mut clean = governed_reading(&["README.md"], &["the greeting prints"], &["x"]);
        clean.summary = "Cooperators accrue the opcode.".into();
        check_reading_acceptance(&clean, "add a greeting", None, &[], None).unwrap();
    }

    #[test]
    fn a_deliverable_that_disagrees_with_the_files_at_head_is_refused() {
        let files = files_at_head();
        let check = |reading: &Reading| {
            check_reading_acceptance(reading, "add a greeting", None, &[], Some(&files))
        };
        // Modify and delete need a file at HEAD.
        for (action, verb) in [
            (DeliverableAction::Modify, "modify"),
            (DeliverableAction::Delete, "delete"),
        ] {
            let reading = reading_with_deliverable("docs/missing.md", action);
            let error = check(&reading).unwrap_err().to_string();
            assert!(error.contains("docs/missing.md"), "{error}");
            assert!(error.contains(&format!("is a {verb}")), "{error}");
            assert!(error.contains("no such file"), "{error}");
            // A file that is there is accepted.
            check(&reading_with_deliverable("README.md", action)).unwrap();
        }
        // Create needs an absent path.
        let reading = reading_with_deliverable("README.md", DeliverableAction::Create);
        let error = check(&reading).unwrap_err().to_string();
        assert!(error.contains("README.md"), "{error}");
        assert!(error.contains("is a create"), "{error}");
        assert!(error.contains("already exists"), "{error}");
        check(&reading_with_deliverable(
            "docs/new.md",
            DeliverableAction::Create,
        ))
        .unwrap();
        // Valid paths and clean text: one modify, one create, accepted.
        let mut mixed = governed_reading(
            &["README.md", "docs/new.md"],
            &["the greeting prints"],
            &["the logo"],
        );
        mixed.deliverables[1].action = DeliverableAction::Create;
        check(&mixed).unwrap();
        // One bad path among good ones is refused and only it is named.
        mixed.deliverables.push(Deliverable {
            path: "src/gone.rs".into(),
            action: DeliverableAction::Delete,
            p: 0.5,
        });
        let error = check(&mixed).unwrap_err().to_string();
        assert!(error.contains("src/gone.rs"), "{error}");
        assert!(!error.contains("docs/new.md"), "{error}");
        assert!(!error.contains("deliverable README.md"), "{error}");
    }

    #[test]
    fn the_path_rule_allows_a_directory_and_an_absent_changelog_and_skips_without_a_file_set() {
        let files = files_at_head();
        let check = |reading: &Reading, set: Option<&BTreeSet<String>>| {
            check_reading_acceptance(reading, "add a greeting", None, &[], set)
        };
        // A directory with files under it is present; a name that only starts like one is not.
        let dir = reading_with_deliverable("src", DeliverableAction::Modify);
        check(&dir, Some(&files)).unwrap();
        let prefix = reading_with_deliverable("sr", DeliverableAction::Modify);
        assert!(check(&prefix, Some(&files)).is_err());
        // Athena adds CHANGELOG.md itself, so its modify is not refused when the file is absent.
        let mut change = governed_reading(&["README.md"], &["the greeting prints"], &["x"]);
        ensure_changelog(&mut change);
        assert!(change.deliverables.iter().any(|d| d.path == "CHANGELOG.md"));
        check(&change, Some(&files)).unwrap();
        // A delete of an absent CHANGELOG.md has no such exemption.
        let delete = reading_with_deliverable("CHANGELOG.md", DeliverableAction::Delete);
        assert!(check(&delete, Some(&files)).is_err());
        // With no file set (the replay), the paths are not judged.
        let absent = reading_with_deliverable("docs/missing.md", DeliverableAction::Modify);
        check(&absent, None).unwrap();
        assert!(matches!(
            check_reading_text(&absent, "add a greeting"),
            ReadingCheck::Accept
        ));
        // A reading that declines or asks names no change, so its paths are not judged.
        let mut asking = reading_with_deliverable("docs/missing.md", DeliverableAction::Modify);
        asking.question = Some("which file?".into());
        check(&asking, Some(&files)).unwrap();
        let mut declined = reading_with_deliverable("docs/missing.md", DeliverableAction::Modify);
        declined.capability = None;
        declined.decline_reason = Some("no capability fits".into());
        check(&declined, Some(&files)).unwrap();
    }

    #[test]
    fn the_text_and_path_problems_are_reported_together() {
        let files = files_at_head();
        let word = banned_word();
        let mut reading = reading_with_deliverable("docs/missing.md", DeliverableAction::Modify);
        reading.summary = format!("The {word} decides.");
        let error = check_reading_acceptance(&reading, "add a greeting", None, &[], Some(&files))
            .unwrap_err()
            .to_string();
        assert!(error.contains(&word), "{error}");
        assert!(error.contains("docs/missing.md"), "{error}");
    }

    // -- The replay of accepted readings

    const REFUSED_CRITERION: &str = "the producer's report lists the caller-check Grep";
    const ACCEPTED_CRITERION: &str = "cargo test --locked passes";
    const MARKER: &str = "MARKER";

    /// A stored request and accepted reading for `id` with this title and one criterion.
    fn replay_case(id: &str, title: &str, criterion: &str) -> ReplayCase {
        let mut reading = reading_with_criterion(criterion);
        reading.job = JobId::parse(id).unwrap();
        reading.title = title.into();
        ReplayCase {
            job: reading.job.clone(),
            request: "add a greeting".into(),
            reading,
        }
    }

    /// The base rules: the real reading rules.
    fn base_checker() -> impl ReadingChecker {
        |request: &str, reading: &Reading| Ok(check_reading_text(reading, request))
    }

    /// A candidate-like rule change: the real rules plus one more refusal, for a marker in the title.
    fn marker_candidate() -> impl ReadingChecker {
        |request: &str, reading: &Reading| match check_reading_text(reading, request) {
            ReadingCheck::Accept if reading.title.contains(MARKER) => {
                Ok(ReadingCheck::Refuse("the marker rule refuses this".into()))
            }
            other => Ok(other),
        }
    }

    fn case_id(n: usize) -> String {
        format!("20201001-{n:06}-{n:04}")
    }

    /// `total` cases of which the first `marked` carry the marker.
    fn marked_cases(total: usize, marked: usize) -> Vec<ReplayCase> {
        (0..total)
            .map(|n| {
                let title = if n < marked { MARKER } else { "Add a greeting" };
                replay_case(&case_id(n), title, ACCEPTED_CRITERION)
            })
            .collect()
    }

    fn criterion(text: &str) -> Criterion {
        Criterion {
            text: text.into(),
            kind: CriterionKind::Behavior,
        }
    }

    #[test]
    fn replay_lists_only_the_requests_the_candidate_newly_refuses() {
        let cases = vec![
            replay_case(&case_id(1), "Add a greeting", REFUSED_CRITERION),
            replay_case(&case_id(2), "Add a greeting", REFUSED_CRITERION),
            replay_case(&case_id(3), "Add a greeting", ACCEPTED_CRITERION),
            replay_case(&case_id(4), MARKER, ACCEPTED_CRITERION),
            replay_case(&case_id(5), MARKER, ACCEPTED_CRITERION),
        ];
        let run = replay(&cases, &base_checker(), &marker_candidate(), None);
        assert_eq!(run.no_replay, None);
        assert_eq!(run.total, 5);
        assert_eq!(run.newly_refused.len(), 2, "{:?}", run.newly_refused);
        let ids: Vec<&str> = run.newly_refused.iter().map(|r| r.job.as_str()).collect();
        assert_eq!(ids, [case_id(4), case_id(5)]);
        assert!(
            run.newly_refused
                .iter()
                .all(|r| r.reason == "the marker rule refuses this"),
            "{:?}",
            run.newly_refused
        );
        // Refused by both is not new, and a candidate equal to the base lists nothing.
        let same = replay(&cases, &base_checker(), &base_checker(), None);
        assert!(same.newly_refused.is_empty());
        assert!(!same.held);
    }

    #[test]
    fn a_checker_that_fails_gives_no_replay_not_a_pass() {
        let cases = marked_cases(3, 0);
        let broken = |_: &str, _: &Reading| -> Result<ReadingCheck> { bail!("exec format error") };
        let run = replay(&cases, &base_checker(), &broken, None);
        let reason = run.no_replay.expect("no replay");
        assert!(
            reason.starts_with("no replay: the candidate build failed on"),
            "{reason}"
        );
        assert!(reason.contains("exec format error"), "{reason}");
        assert_eq!((run.total, run.held), (0, false));
    }

    fn history_job(id: &str, accepted: bool) -> JobRecords {
        let case = replay_case(id, "Add a greeting", ACCEPTED_CRITERION);
        let mut job = dispatched_job_stub();
        job.id = case.job.clone();
        job.accepted_reading = accepted.then_some(1);
        JobRecords {
            id: Some(case.job.clone()),
            request: Some(Request {
                job: case.job,
                text: case.request,
                received_at: jiff::Timestamp::UNIX_EPOCH,
                channel: "cli".into(),
            }),
            readings: vec![case.reading],
            job: Some(job),
            ..Default::default()
        }
    }

    #[test]
    fn replay_window_is_the_newest_fifty_accepted_readings() {
        let mut records = Records::default();
        let mut add = |job: JobRecords| {
            records.jobs.insert(job.id().as_str().to_string(), job);
        };
        // 60 accepted readings, oldest first by id.
        for n in 0..60 {
            add(history_job(&case_id(n), true));
        }
        // Five newer jobs whose reading was never accepted, and the running job, newest of all.
        for n in 0..5 {
            add(history_job(&format!("20210101-{n:06}-{n:04}"), false));
        }
        let running = JobId::parse("20220101-000000-0000").unwrap();
        add(history_job(running.as_str(), true));

        let cases = replay_cases(&records, Some(&running));
        assert_eq!(cases.len(), REPLAY_WINDOW);
        let ids: Vec<&str> = cases.iter().map(|c| c.job.as_str()).collect();
        let expected: Vec<String> = (10..60).rev().map(case_id).collect();
        assert_eq!(ids, expected.iter().map(String::as_str).collect::<Vec<_>>());
        // Without the exclusion the running job would be the newest case.
        assert_eq!(
            replay_cases(&records, None)[0].job.as_str(),
            running.as_str()
        );
    }

    #[test]
    fn replay_window_with_an_empty_history_is_no_replay_not_a_pass() {
        let empty = replay_cases(&Records::default(), None);
        assert!(empty.is_empty());
        let run = replay(&empty, &base_checker(), &marker_candidate(), None);
        assert_eq!(
            run.no_replay.as_deref(),
            Some("no replay: no accepted readings in the history")
        );
        assert_eq!((run.total, run.held), (0, false));
        let record = run.record(&JobId::parse(PRIOR_JOB).unwrap(), "a", "b");
        record.validate().unwrap();
        // A history of only the running job is empty too.
        let running = JobId::parse(PRIOR_JOB).unwrap();
        let mut records = Records::default();
        records
            .jobs
            .insert(PRIOR_JOB.into(), history_job(PRIOR_JOB, true));
        assert!(replay_cases(&records, Some(&running)).is_empty());
    }

    #[test]
    fn replay_hold_is_over_ten_percent_without_a_declared_share() {
        let marked = |n| {
            let cases = marked_cases(10, n);
            replay(&cases, &base_checker(), &marker_candidate(), None)
        };
        // 2 of 10: held, with the list.
        let held = marked(2);
        assert!(held.held && !held.intended);
        assert_eq!(held.newly_refused.len(), 2);
        // Exactly 10% (1 of 10) is not held; the list is still recorded.
        let at = marked(1);
        assert!(!at.held);
        assert_eq!(at.newly_refused.len(), 1);
        // A declared 20% makes 2 of 10 not held, and the replay is still recorded in full.
        let cases = marked_cases(10, 2);
        let declared = replay(&cases, &base_checker(), &marker_candidate(), Some(20));
        assert!(declared.intended && !declared.held);
        assert_eq!(declared.newly_refused.len(), 2);
        assert_eq!(declared.total, 10);
        declared
            .record(&JobId::parse(PRIOR_JOB).unwrap(), "a", "b")
            .validate()
            .unwrap();
        // Criteria that only mention refusal declare nothing.
        let mentions = [criterion("readings that already refuse stay refused")];
        let none = replay_declaration(&mentions, &[]);
        let still_held = replay(
            &cases,
            &base_checker(),
            &marker_candidate(),
            none.effective_share(),
        );
        assert!(!still_held.intended && still_held.held);
    }

    #[test]
    fn replay_hold_boundary_uses_integer_percent() {
        for (total, newly, held) in [
            (0, 0, false),
            (10, 0, false),
            (10, 1, false),
            (10, 2, true),
            (50, 5, false),
            (50, 6, true),
            (3, 1, true),
        ] {
            assert_eq!(replay_is_held(total, newly, None), held, "{newly}/{total}");
        }
    }

    /// Replays `marked` newly refused readings of `total` under what `criteria` and `out_of_scope` declare.
    fn declared_run(
        criteria: &[&str],
        out_of_scope: &[&str],
        total: usize,
        marked: usize,
    ) -> (ReplayDeclaration, ReplayRun) {
        let criteria: Vec<Criterion> = criteria.iter().map(|t| criterion(t)).collect();
        let out_of_scope: Vec<String> = out_of_scope.iter().map(|s| s.to_string()).collect();
        let declaration = replay_declaration(&criteria, &out_of_scope);
        let run = replay(
            &marked_cases(total, marked),
            &base_checker(),
            &marker_candidate(),
            declaration.effective_share(),
        );
        (declaration, run)
    }

    const REASON: &str =
        "readers now ask for a new required section, because past readings lack it";

    #[test]
    fn replay_declared_share_of_30_with_80_measured_is_held_and_80_declared_passes() {
        let job = JobId::parse(PRIOR_JOB).unwrap();
        // 40 of 50 (80%) newly refused against a declared 30%: held, and the record is valid.
        let (declaration, held) = declared_run(
            &[ACCEPTED_CRITERION, "expected 30% of newly refused readings"],
            &[],
            50,
            40,
        );
        assert_eq!(declaration.share, Some(30));
        assert_eq!((held.total, held.newly_refused.len()), (50, 40));
        assert!(held.held && !held.intended);
        held.record(&job, "a", "b").validate().unwrap();
        // The same 80% against a declared 80% with a stated reason passes, and the list is kept.
        let (declaration, passed) = declared_run(
            &[ACCEPTED_CRITERION, "expected 80% of newly refused readings"],
            &[REASON],
            50,
            40,
        );
        assert_eq!(declaration.share, Some(80));
        assert_eq!(declaration.effective_share(), Some(80));
        assert!(!passed.held && passed.intended);
        assert_eq!(passed.newly_refused.len(), 40);
        passed.record(&job, "a", "b").validate().unwrap();
    }

    #[test]
    fn replay_phrase_without_a_number_does_not_exempt() {
        for text in [
            "the higher refusal rate is intended",
            "the change refuses more readings than the base",
        ] {
            let (declaration, run) = declared_run(&[ACCEPTED_CRITERION, text], &[], 50, 40);
            assert_eq!(declaration, ReplayDeclaration::default(), "{text}");
            assert!(run.held && !run.intended, "{text}");
            // Exactly 10% (1 of 10) is still not held.
            let (_, at) = declared_run(&[ACCEPTED_CRITERION, text], &[], 10, 1);
            assert!(!at.held, "{text}");
            assert_eq!(at.newly_refused.len(), 1);
        }
    }

    #[test]
    fn replay_phrase_only_candidate_is_now_held() {
        // The 2026-10-28 case: 40 of 50 newly refused and only a phrase saying the higher rate is intended.
        let (_, run) = declared_run(
            &[
                ACCEPTED_CRITERION,
                "the change refuses more readings than the base, and the higher refusal rate is intended",
            ],
            &[],
            50,
            40,
        );
        assert_eq!(run.newly_refused.len(), 40);
        assert!(run.held && !run.intended);
        assert!(replay_is_held(50, 40, None));
    }

    #[test]
    fn replay_declared_margin_edge_is_five_points_inclusive() {
        let declared = "expected 30% of newly refused readings";
        // Declared 30% with 35 of 100 passes; 36 of 100 is held.
        // A share above 25% needs a reason to apply.
        let share = replay_declaration(&[criterion(declared)], &[REASON.to_string()]);
        let share = share.effective_share();
        assert_eq!(share, Some(30));
        let hundred = |marked: usize| {
            let cases: Vec<ReplayCase> = (0..100)
                .map(|n| {
                    let title = if n < marked { MARKER } else { "Add a greeting" };
                    replay_case(
                        &format!("20201001-000000-{n:04}"),
                        title,
                        ACCEPTED_CRITERION,
                    )
                })
                .collect();
            replay(&cases, &base_checker(), &marker_candidate(), share)
        };
        let (at, over) = (hundred(35), hundred(36));
        assert_eq!((at.newly_refused.len(), over.newly_refused.len()), (35, 36));
        assert!(!at.held && at.intended, "35 of 100");
        assert!(over.held && !over.intended, "36 of 100");
        for (total, newly, declared, held) in [
            (50, 17, Some(30), false),
            (50, 18, Some(30), true),
            (100, 5, Some(0), false),
            (100, 6, Some(0), true),
            (10, 1, None, false),
            (10, 2, None, true),
            (50, 40, Some(80), false),
            (50, 43, Some(80), true),
            (50, 42, Some(80), false),
            (100, 100, Some(100), false),
        ] {
            assert_eq!(
                replay_is_held(total, newly, declared),
                held,
                "{newly}/{total} declared {declared:?}"
            );
        }
    }

    #[test]
    fn replay_reason_is_needed_above_25_percent() {
        // 25% needs no reason; 26% does, and without one the 10% limit applies.
        let at = replay_declaration(&[criterion("expected 25% of newly refused readings")], &[]);
        assert_eq!((at.share, at.effective_share()), (Some(25), Some(25)));
        assert_eq!(at.unsupported_share(), None);
        let above = replay_declaration(&[criterion("expected 26% of newly refused readings")], &[]);
        assert_eq!(above.share, Some(26));
        assert_eq!(
            (above.effective_share(), above.unsupported_share()),
            (None, Some(26))
        );
        // 80% declared without a reason: 2 of 10 is held at the 10% limit, 1 of 10 is not.
        let declared = "expected 80% of newly refused readings";
        let (_, held) = declared_run(&[declared], &[], 10, 2);
        assert!(held.held && !held.intended);
        let (_, kept) = declared_run(&[declared], &[], 10, 1);
        assert!(!kept.held);
        // An out_of_scope item with a marker and no anchor does not unlock it.
        let (declaration, run) = declared_run(
            &[declared],
            &["records.rs is out of scope because it needs a migration"],
            10,
            2,
        );
        assert_eq!(declaration.reason, None);
        assert!(run.held);
        // An anchor and no marker does not unlock it either.
        let (declaration, run) = declared_run(&[declared], &["past readings are refused"], 10, 2);
        assert_eq!(declaration.reason, None);
        assert!(run.held);
        // One item with a marker and an anchor does, from out_of_scope or from a criterion.
        let (declaration, run) = declared_run(&[declared], &[REASON], 10, 8);
        assert_eq!(declaration.reason.as_deref(), Some(REASON));
        assert!(!run.held && run.intended);
        let (declaration, _) = declared_run(
            &[
                declared,
                "the reader brief now asks for a new section, so earlier readings are refused",
            ],
            &[],
            10,
            8,
        );
        assert!(declaration.reason.is_some());
        // A long reason is cut, with white space made single spaces.
        let long = format!("because  past readings\nlack it {}", "x".repeat(400));
        let cut = replay_declaration(&[criterion(declared)], &[long]);
        let reason = cut.reason.unwrap();
        assert_eq!(reason.chars().count(), REPLAY_REASON_CHARS);
        assert!(reason.starts_with("because past readings lack it "));
    }

    #[test]
    fn replay_declared_percent_ignores_quoted_examples_and_other_numbers() {
        for text in [
            "a candidate declaring 'expected 30%' is held",
            "the log shows 30% progress",
            "expected 2.5% of newly refused readings",
            "coverage expected 90%",
            "expected 101% of newly refused readings",
            "expected 3000% of newly refused readings",
            "v30% of newly refused readings",
            "expected 30 percent of newly refused readings",
        ] {
            assert_eq!(
                replay_declaration(&[criterion(text)], &[]).share,
                None,
                "{text}"
            );
        }
        // The forms that do declare, and the first one wins.
        for (text, share) in [
            ("expected 30% of newly refused readings", Some(30)),
            ("Expected 30%  of the newly refused readings.", Some(30)),
            ("at most 0% of newly refused readings", Some(0)),
            ("100% of newly refused", Some(100)),
            ("(12% of newly refused readings)", Some(12)),
        ] {
            assert_eq!(
                replay_declaration(&[criterion(text)], &[]).share,
                share,
                "{text}"
            );
        }
        let two = replay_declaration(
            &[
                criterion("coverage expected 90%"),
                criterion("expected 20% of newly refused readings"),
                criterion("expected 40% of newly refused readings"),
            ],
            &[],
        );
        assert_eq!(two.share, Some(20));
    }

    #[test]
    fn replay_declaration_of_reads_the_reading_criteria_and_out_of_scope() {
        let mut reading = reading_with_criterion("expected 60% of newly refused readings");
        reading.out_of_scope = vec![REASON.to_string()];
        let declaration = replay_declaration_of(&reading);
        assert_eq!(declaration.share, Some(60));
        assert_eq!(declaration.reason.as_deref(), Some(REASON));
        assert_eq!(
            replay_declaration_of(&reading_with_criterion(ACCEPTED_CRITERION)),
            ReplayDeclaration::default()
        );
    }

    #[test]
    fn changelog_describes_the_declared_share_rule() {
        let changelog = include_str!("../../../CHANGELOG.md")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let unreleased = changelog
            .split("## [Unreleased]")
            .nth(1)
            .expect("an Unreleased section")
            .split(" ## [")
            .next()
            .unwrap();
        for needle in [
            "of newly refused readings",
            "5 points",
            "10%",
            "25%",
            "reason",
        ] {
            assert!(unreleased.contains(needle), "{needle}");
        }
    }

    #[test]
    fn check_lines_round_trip_and_odd_output_is_an_error() {
        for check in [
            ReadingCheck::Accept,
            ReadingCheck::Refuse("criterion 1 cites it".into()),
        ] {
            assert_eq!(ReadingCheck::parse(&check.line()).unwrap(), check);
        }
        for odd in ["", "ok", "refuse:", "refuse:   ", "error: boom"] {
            assert!(ReadingCheck::parse(odd).is_err(), "{odd:?}");
        }
        let one_line =
            check_reading_text(&reading_with_criterion(REFUSED_CRITERION), "add a greeting").line();
        assert!(one_line.starts_with("refuse: "), "{one_line}");
        assert!(!one_line.contains('\n'), "{one_line}");
    }

    /// An executable script that stands in for a build of `athena`.
    #[cfg(unix)]
    fn stub_binary(dir: &std::path::Path, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join("athena-stub");
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    /// Runs a checker, trying again while the freshly written stub is busy (a parallel test's fork can hold it).
    #[cfg(unix)]
    fn check_stub(bin: PathBuf, reading: &Reading) -> Result<ReadingCheck> {
        let checker = BinaryChecker::new(bin);
        let mut result = checker.check("add a greeting", reading);
        for _ in 0..20 {
            if !result
                .as_ref()
                .is_err_and(|e| format!("{e:#}").contains("Text file busy"))
            {
                break;
            }
            std::thread::sleep(Duration::from_millis(50));
            result = checker.check("add a greeting", reading);
        }
        result
    }

    #[cfg(unix)]
    #[test]
    fn replay_binary_checker_runs_a_build_from_its_path_alone_and_parses_its_line() {
        let dir = tempfile::tempdir().unwrap();
        let reading = reading_with_criterion(ACCEPTED_CRITERION);
        // The stub also proves the arguments: it exits 3 unless both files exist.
        let body = |line: &str, code: u8| {
            format!(
                "[ \"$1\" = check-reading ] && [ -f \"$3\" ] && [ -f \"$5\" ] || exit 3\n\
                 echo '{line}'\nexit {code}"
            )
        };
        let refusing = stub_binary(dir.path(), &body("refuse: the stub says no", 2));
        assert_eq!(
            check_stub(refusing, &reading).unwrap(),
            ReadingCheck::Refuse("the stub says no".into())
        );
        let accepting = stub_binary(dir.path(), &body("accept", 0));
        assert_eq!(
            check_stub(accepting, &reading).unwrap(),
            ReadingCheck::Accept
        );
        let failing = stub_binary(dir.path(), "echo oops >&2\nexit 1");
        let error = check_stub(failing, &reading).unwrap_err().to_string();
        assert!(error.contains("oops"), "{error}");
        let missing = BinaryChecker::new(dir.path().join("no-such-binary"));
        assert!(missing.check("add a greeting", &reading).is_err());
    }

    const SURVEY: &str = "Code survey: crates/athena-core/src/reading.rs check_reading_acceptance is the entry point; \
         the records validation allow-list was read in crates/athena-data/src/records.rs";

    /// A change reading of class code with one behavior criterion that cites a command, these assumptions
    /// and these deliverables.
    fn code_reading(assumptions: &[&str], deliverables: &[(&str, DeliverableAction)]) -> Reading {
        let mut reading =
            scoped_reading("Change the reading rules.", &[], &["the planner prompts"]);
        reading.class_of_work = "code".into();
        reading.assumptions = assumptions.iter().map(|a| a.to_string()).collect();
        reading.acceptance_criteria = vec![Criterion {
            text: "Run `cargo test -p athena-core`: the new unit test passes".into(),
            kind: CriterionKind::Behavior,
        }];
        reading.deliverables = deliverables
            .iter()
            .map(|(path, action)| Deliverable {
                path: (*path).into(),
                action: *action,
                p: 0.9,
            })
            .collect();
        reading
    }

    fn accept(reading: &Reading) -> Result<()> {
        check_reading_acceptance(reading, "change the reading rules", None, &[], None)
    }

    const MODIFY_READING: (&str, DeliverableAction) = (
        "crates/athena-core/src/reading.rs",
        DeliverableAction::Modify,
    );

    #[test]
    fn code_survey_missing_is_refused_and_present_is_accepted() {
        let error = accept(&code_reading(&[], &[MODIFY_READING]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("`Code survey:`"), "{error}");
        assert!(error.contains("crates/"), "{error}");
        assert!(error.contains("was refused"), "{error}");
        accept(&code_reading(&[SURVEY], &[MODIFY_READING])).unwrap();
        // The label is matched after trimming, in any case.
        accept(&code_reading(
            &["  code SURVEY: crates/athena-core/src/reading.rs"],
            &[MODIFY_READING],
        ))
        .unwrap();
    }

    #[test]
    fn code_survey_tests_class_is_refused_too() {
        let mut reading = code_reading(&[], &[MODIFY_READING]);
        reading.class_of_work = "tests".into();
        assert!(accept(&reading).is_err());
        reading.assumptions = vec![SURVEY.into()];
        accept(&reading).unwrap();
    }

    #[test]
    fn code_survey_must_start_with_the_label_and_name_a_crates_path() {
        // The label is not at the start.
        let error = accept(&code_reading(
            &["The Code survey: crates/athena-core/src/reading.rs was read"],
            &[MODIFY_READING],
        ))
        .unwrap_err()
        .to_string();
        assert!(error.contains("`Code survey:`"), "{error}");
        // A survey with no crates/ path, or a bare `crates/`.
        for text in [
            "Code survey: I read the reading rules",
            "Code survey: crates/ was read",
        ] {
            assert!(
                accept(&code_reading(&[text], &[MODIFY_READING])).is_err(),
                "{text}"
            );
        }
    }

    #[test]
    fn code_survey_new_record_file_needs_named_evidence() {
        let new_file = (
            "crates/athena-data/src/example.json",
            DeliverableAction::Create,
        );
        let plain =
            "Code survey: crates/athena-data/src/records.rs was read; the allow-list is wide";
        let error = accept(&code_reading(&[plain], &[new_file]))
            .unwrap_err()
            .to_string();
        assert!(error.contains("check_record_file"), "{error}");
        assert!(error.contains("AnyRecord"), "{error}");
        assert!(error.contains("round-trip"), "{error}");
        assert!(
            error.contains("crates/athena-data/src/example.json"),
            "{error}"
        );
        // A bare round-trip in prose is not a named test.
        let prose =
            "Code survey: crates/athena-data/src/records.rs was read; a round-trip is needed";
        assert!(accept(&code_reading(&[prose], &[new_file])).is_err());
        // A change to records.rs needs the same evidence.
        let records_rs = (
            "crates/athena-data/src/records.rs",
            DeliverableAction::Modify,
        );
        assert!(accept(&code_reading(&[plain], &[records_rs])).is_err());
        for evidence in [
            "Code survey: crates/athena-data/src/records.rs: `check_record_file` accepts it",
            "Code survey: crates/athena-data/src/records.rs: AnyRecord has a variant for it",
            "Code survey: crates/athena-data/src/records.rs: `records_round_trip_every_kind` covers it",
            "Code survey: crates/athena-data/src/records.rs: crates/athena-data/tests/round-trip.rs covers it",
        ] {
            accept(&code_reading(&[evidence], &[new_file]))
                .unwrap_or_else(|e| panic!("{evidence}: {e}"));
            accept(&code_reading(&[evidence], &[records_rs]))
                .unwrap_or_else(|e| panic!("{evidence}: {e}"));
        }
        // The evidence may sit in a second survey assumption.
        accept(&code_reading(
            &[
                SURVEY,
                "Code survey: crates/athena-data/src/records.rs: `AnyRecord`",
            ],
            &[new_file],
        ))
        .unwrap();
    }

    #[test]
    fn code_survey_record_file_requirement_ignores_ordinary_deliverables_and_criteria() {
        // A new crate's manifest, a changed existing json and a new source file are not new record files.
        for deliverable in [
            ("crates/foo/Cargo.toml", DeliverableAction::Create),
            ("crates/foo/src/lib.rs", DeliverableAction::Create),
            ("crates/foo/data.json", DeliverableAction::Modify),
            ("docs/example.json", DeliverableAction::Create),
        ] {
            accept(&code_reading(&[SURVEY], &[deliverable]))
                .unwrap_or_else(|e| panic!("{deliverable:?}: {e}"));
        }
        // A criterion that mentions records/ only to exclude it triggers nothing.
        let mut reading = code_reading(&[SURVEY], &[MODIFY_READING]);
        reading.acceptance_criteria.push(Criterion {
            text:
                "records/ is not changed; Run `git status --short`: no file under records/ appears"
                    .into(),
            kind: CriterionKind::Behavior,
        });
        accept(&reading).unwrap();
        // A new file in a records or data directory is one.
        for path in ["crates/foo/records/one.rs", "crates/foo/data/x"] {
            assert!(
                accept(&code_reading(
                    &[SURVEY],
                    &[(path, DeliverableAction::Create)]
                ))
                .is_err(),
                "{path}"
            );
        }
    }

    #[test]
    fn code_survey_does_not_apply_outside_its_gate() {
        let none: &[&str] = &[];
        // Docs and other classes.
        let mut docs = code_reading(none, &[MODIFY_READING]);
        docs.class_of_work = "docs".into();
        accept(&docs).unwrap();
        // No capability (a decline).
        let mut declined = code_reading(none, &[MODIFY_READING]);
        declined.capability = None;
        declined.decline_reason = Some("no capability fits".into());
        accept(&declined).unwrap();
        // A question.
        let mut asking = code_reading(none, &[MODIFY_READING]);
        asking.question = Some("which file?".into());
        accept(&asking).unwrap();
        // Content criteria only.
        let mut content = code_reading(none, &[MODIFY_READING]);
        content.acceptance_criteria = vec![Criterion {
            text: "the doc comment names the cause".into(),
            kind: CriterionKind::Content,
        }];
        accept(&content).unwrap();
        // The check itself agrees on each.
        for reading in [&docs, &declined, &asking, &content] {
            check_code_survey(reading).unwrap();
        }
    }

    #[test]
    fn code_survey_refusal_is_declared_intended() {
        // The sentence alone declares no share, so a replay would hold a check like this at 10%.
        let criterion = Criterion {
            text: "The change refuses more readings: change readings of class code or tests with a behavior \
                   criterion and no 'Code survey:' assumption naming a crates/ path are refused, and a higher \
                   refusal rate is intended."
                .into(),
            kind: CriterionKind::Content,
        };
        assert_eq!(replay_declaration(&[criterion], &[]).share, None);
        // The change does refuse the reading it names.
        assert!(accept(&code_reading(&[], &[MODIFY_READING])).is_err());
        // The check's own doc comment and the changelog carry the same sentence.
        let source = include_str!("reading.rs");
        let doc = source
            .split("pub fn check_code_survey")
            .next()
            .unwrap()
            .rsplit("\n\n")
            .next()
            .unwrap();
        assert!(
            doc.contains("This check refuses more readings; the higher refusal rate is intended.")
        );
        // The changelog wraps lines, so compare with runs of white space made single spaces.
        let changelog = include_str!("../../../CHANGELOG.md")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(changelog.contains("refuses more readings"));
        assert!(changelog.contains("higher refusal rate is intended"));
    }
}
