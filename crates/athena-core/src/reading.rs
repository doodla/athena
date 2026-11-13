//! `ask` and `correct`: the reader's reading, with forecasts, committed before any work (design §3.2–3.3).
//!
//! # Where a reading is checked for boundary words
//!
//! [`check_boundary_and_repo_files`] runs the boundary scan on two places only: every deliverable path, and
//! every acceptance criterion of kind `content` (text that says what a file must hold). It does not scan the
//! title, summary, assumptions, out-of-scope list, defaults applied, the decline, hard and urgent reasons,
//! the question, the departures or `behavior` criteria.
//!
//! Why: the risk the check guards against is a word reaching a tracked file. A reading is a record, records
//! are outside the boundary scan the policy test runs, and the writer already gets the word list and runs
//! that scan on its own files (D73). A word in prose therefore cannot fail a file check, and refusing it only
//! sent the reader round again (D153 cites the refusals). A deliverable path becomes a file name and a
//! `content` criterion tells the writer what to write, so a word there can reach a file and is still refused.
//!
//! Limits, stated so no one assumes more cover than there is: criterion kind is an imperfect signal (a
//! `content` criterion that only describes a document is still scanned; a `behavior` criterion that quotes
//! output is not, and the writer's scan and the policy scan at verification catch a word that reaches a
//! tracked file). The title, summary and criterion texts also reach landing commit messages and the pull
//! request title, and no scan covers those; D153 records that gap and the proposed follow-up.
//!
//! The scan of a user's own request text is a different check and is unchanged.

use crate::admission::{governance_change_is_listed, is_governance};
use crate::capability::{JobConfig, ToolGrant};
use crate::checks::tail;
use crate::commits::{NewFile, message};
use crate::invocation::InvocationFacts;
use crate::plan::{Ending, JobEnd, Priority};
use crate::store::{JobRecords, JobState, Records};
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_agent::AgentInvocation;
use athena_agent::env::{EnvSpec, Side};
use athena_agent::policy::Policy;
use athena_context::briefs::{
    CHANGELOG_PATH, EarlierAttempt, EarlierCase, ReaderInputs, RetryCause, reader_brief,
    render_catalog,
};
use athena_context::outputs::{
    CriterionKindOutput, DeliverableActionOutput, ReaderChoices, ReadingOutput, reading_schema,
};
use athena_context::{CONTRACT, Component, Composed, READER_FRAME, compose};
use athena_data::records::{
    AnyRecord, Correction, CorrectionKind, Criterion, CriterionKind, CritiqueVerdict,
    DefaultApplied, Deliverable, DeliverableAction, Finding, FocusRecord, Forecast, Forecasts,
    HoldRecord, HoldState, Intervention, InterventionCategory, InterventionSource,
    IntrospectionRecord, IntrospectionStatus, IntrospectionTrigger, Invocation, OTHER_READING_RULE,
    OutcomeState, PreventionKind, PriorityRecord, Quantiles, Reading, ReadingAttempt,
    ReadingRefusal, Replay, ReplayRefusal, Request, Role,
};
use athena_data::{JobId, RecordId};
use athena_gate::redact::Redactor;
use athena_git::snapshot::{Snapshot, SnapshotRepo};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::Duration;

mod checks;
mod outcome;
mod replay;
mod retry;
mod surveys;
mod terms;
mod topics;

pub use checks::*;
pub use outcome::*;
pub use replay::*;
pub use retry::*;
pub use surveys::*;
pub use terms::*;
pub use topics::*;

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

/// Where a job is when its budget is checked against everything still to run (see [`budget_reserve`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReserveStage {
    /// Before the reader: the whole path, the reader included, with every planning round when `[job_plan]`
    /// is enabled (the reading's `hard` flag and capability are not known yet, so the planning terms are
    /// counted; over-reserving is the safe side).
    Admission,
    /// Before anything is committed for a dispatched job. `planned` is whether this job will be planned.
    Dispatch { planned: bool },
    /// Before the planner of round `round` (1-based): this round and every later one, then the producer and
    /// the verifier.
    Planner { round: u32 },
    /// Before the critic of round `round` (1-based): this critic, every later round, then the producer and
    /// the verifier.
    Critic { round: u32 },
    /// Planning is over (or there was none): the producer and the verifier.
    Run,
}

/// The one computation of what a job must still be able to afford, in dollars, from `stage` on. Every cap is
/// read from `config`, so a changed cap in athena.toml changes the reserve with no code change. Each planning
/// round is a planner call and a critic call, up to `[job_plan] max_rounds` of them.
pub fn budget_reserve(config: &athena_data::Config, stage: ReserveStage) -> f64 {
    // Every sum of caps comes from `Config`, which `Config::check_budget` also uses: one place to change.
    let max_rounds = config.job_plan.map_or(0, |p| p.max_rounds);
    let after = config.run_usd();
    match stage {
        ReserveStage::Admission => config.budget_required(),
        ReserveStage::Dispatch { planned } => {
            let planning = if planned {
                config.planning_usd(max_rounds)
            } else {
                0.0
            };
            planning + after
        }
        ReserveStage::Planner { round } => {
            config.planning_usd(max_rounds.saturating_sub(round) + 1) + after
        }
        ReserveStage::Critic { round } => {
            config.critic_usd() + config.planning_usd(max_rounds.saturating_sub(round)) + after
        }
        ReserveStage::Run => after,
    }
}

/// A job whose budget does not cover what still has to run. It is returned before the money is spent, so
/// nothing was spent and no state changed; the drain holds such a job instead of counting a failure.
#[derive(Debug, Clone, Copy)]
pub struct BudgetShort {
    /// What is left of the job's budget.
    pub left: f64,
    /// The job's whole budget, `[budget] job_usd`.
    pub job_usd: f64,
    /// What the stage needs.
    pub reserve: f64,
    pub stage: ReserveStage,
}

impl BudgetShort {
    /// How much more budget the job needs.
    pub fn shortfall(&self) -> f64 {
        (self.reserve - self.left).max(0.0)
    }
}

impl std::fmt::Display for BudgetShort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let what = match self.stage {
            ReserveStage::Admission => {
                "the reader, every planning round, the producer and the verifier"
            }
            ReserveStage::Dispatch { planned: true } => {
                "every planning round, the producer and the verifier"
            }
            ReserveStage::Dispatch { planned: false } | ReserveStage::Run => {
                "the producer and the verifier"
            }
            ReserveStage::Planner { .. } => {
                "this planner call, every planning round after it, the producer and the verifier"
            }
            ReserveStage::Critic { .. } => {
                "this critic call, every planning round after it, the producer and the verifier"
            }
        };
        write!(
            f,
            "the job has ${:.2} of its ${:.2} budget left, ${:.2} short of the ${:.2} it needs for {what}; \
             raise `[budget] job_usd` in athena.toml through a job, or close this one",
            self.left,
            self.job_usd,
            self.shortfall(),
            self.reserve
        )
    }
}

impl std::error::Error for BudgetShort {}

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
    /// The reader returned a reading that a reading rule refused. The text says what to fix; the rule that
    /// refused it is kept in the invocation record's `reading_attempt`.
    Refused(String),
}

impl ReadError {
    fn into_text(self) -> String {
        match self {
            ReadError::Failed(text) | ReadError::Refused(text) => text,
        }
    }
}

/// A refusal raised while the reader's output is built into a reading, with the name of the rule that
/// raised it. Its text is the rule's own text, unchanged.
#[derive(Debug)]
struct NamedRefusal {
    rule: &'static str,
    text: String,
}

impl std::fmt::Display for NamedRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

impl std::error::Error for NamedRefusal {}

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

/// Whether a change reading asks for `CHANGELOG.md` (a modify deliverable plus the `changelog` default that
/// [`ensure_changelog`] adds) and `changed`, the paths the candidate touches, does not include it.
///
/// Root cause of the failure class this answers: the `changelog` default was a rule the producer had to
/// remember, and no code compared the diff's file list with the reading's deliverables before the verdict.
/// For a docs-only design change the changelog feels optional, so it was skipped, and jobs 20261108-183842-fsaz
/// and 20261028-000831-rkv5 were failed only by the final verdict. Admission and the producer loop both call
/// this predicate (through `admission::changelog_required`), so the two cannot disagree. Any status counts as
/// touching the file; only the root `CHANGELOG.md` counts, with a leading `./` ignored.
pub(crate) fn changelog_unproduced<'a>(
    reading: &Reading,
    changed: impl IntoIterator<Item = &'a str>,
) -> bool {
    reading.capability.as_deref() == Some("change")
        && reading
            .deliverables
            .iter()
            .any(|d| d.path == CHANGELOG_PATH && d.action == DeliverableAction::Modify)
        && reading.defaults_applied.iter().any(|d| d.id == "changelog")
        && !changed
            .into_iter()
            .any(|p| p.trim_start_matches("./") == CHANGELOG_PATH)
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
        // The producer and the verifier may read the job's evidence snapshot, once dispatch has written it.
        let evidence = Some(crate::snapshot::evidence_dir(&self.state, job))
            .filter(|dir| matches!(role, Role::Producer | Role::Verifier) && dir.is_dir());
        let policy = Policy {
            role,
            root: snapshot.to_path_buf(),
            commands: grant.commands.clone(),
            writes: grant.writes,
            evidence: evidence.clone(),
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
                read_dirs: evidence.into_iter().collect(),
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

    /// Refuses, before any money is spent on the next stage, a job whose budget cannot cover everything that
    /// still has to run from `stage` on (see [`budget_reserve`]). `spent` is what the job has spent so far.
    /// The error is a [`BudgetShort`], so a caller can tell a hold made before spend from a failed run.
    pub(crate) fn preflight(&self, jc: &JobConfig, spent: f64, stage: ReserveStage) -> Result<()> {
        let reserve = budget_reserve(&jc.config, stage);
        let left = jc.config.budget.job_usd - spent;
        if reserve > left + 1e-9 {
            return Err(BudgetShort {
                left: left.max(0.0),
                job_usd: jc.config.budget.job_usd,
                reserve,
                stage,
            }
            .into());
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
            enabled_roles: jc.config.enabled_roles(),
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
        let (output, mut record) = match self.invoke_and_record(&facts, redactor)? {
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
        let judged = match self.build_reading(
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
                match refusing_check(
                    &reading,
                    &request.text,
                    attempt,
                    &referenced_jobs,
                    Some(&repo_files),
                ) {
                    None => Ok(reading),
                    Some((rule, e)) => Err((rule, format!("{e:#}"))),
                }
            }
            Err(e) => {
                let rule = e
                    .downcast_ref::<NamedRefusal>()
                    .map_or(OTHER_READING_RULE, |named| named.rule);
                Err((rule, format!("the reader's output was invalid: {e:#}")))
            }
        };
        // The second reading of a run is the one made with a refusal as its correction.
        let attempt_number = if refusal.is_some() { 2 } else { 1 };
        match judged {
            Ok(reading) => {
                record.reading_attempt = Some(ReadingAttempt {
                    attempt: attempt_number,
                    refusal: None,
                });
                Ok((Some(reading), record, None))
            }
            Err((rule, text)) => {
                // The text can quote the request or the reading, so the stored copy is redacted.
                let mut stored = vec![text.clone()];
                redactor.redact_all(&mut stored)?;
                record.reading_attempt = Some(ReadingAttempt {
                    attempt: attempt_number,
                    refusal: Some(ReadingRefusal {
                        rule: rule.to_string(),
                        text: stored.remove(0),
                    }),
                });
                Ok((None, record, Some(ReadError::Refused(text))))
            }
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
            )
            .map_err(|e| NamedRefusal {
                rule: "check_reading_size",
                text: format!("{e:#}"),
            })?;
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
        refuse_banned_input(text)?;
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
        self.first_reading(&jc, &records, &request, true, None, ui)
    }

    /// `athena ask --later`: records a request to be read and run later. Nothing is read and no model is
    /// called; `athena status` lists it as open work, and `athena go` reads it.
    pub fn queue(&self, text: &str) -> Result<(JobId, String)> {
        if text.trim().is_empty() {
            bail!("the request is empty; say what you want in a line or two");
        }
        let head = self.head()?;
        refuse_banned_input(text)?;
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
        let subject = queue_subject(&request.text);
        let commit = self.commit_records(
            &[NewFile::record(&AnyRecord::Request(request))],
            &message(
                &subject,
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

    /// `athena hold <job>`: appends a hold record for an open request, so `athena drain` does not dispatch it
    /// whatever its priority, until `athena release`. The request record is not touched. The job must be
    /// queued, read (proposed) or interrupted, must not be running, and must not already be on hold; all of
    /// that is checked before anything is committed. Like [`Athena::set_priority`], it reads the records at
    /// `HEAD`.
    pub fn set_hold(&self, job_ref: &str) -> Result<JobId> {
        let head = self.head()?;
        let records = crate::store::Records::load(&self.git, &head)?;
        let job_id = records.resolve(job_ref)?;
        let job = records.jobs.get(job_id.as_str()).expect("resolved");
        if !matches!(
            job.state(),
            JobState::Queued | JobState::Proposed | JobState::Dispatched
        ) {
            bail!(
                "job {job_id} is {}; only an open request can be held",
                job.state().name()
            );
        }
        if self.running(job_id.as_str()) {
            bail!("job {job_id} is running now; it can be held once it stops");
        }
        if job
            .latest_hold()
            .is_some_and(|h| h.state == HoldState::OnHold)
        {
            bail!("job {job_id} is already on hold; `athena release {job_id}` lifts it");
        }
        self.append_hold(&job_id, job, HoldState::OnHold)?;
        Ok(job_id)
    }

    /// `athena release <job>`: appends a later hold record that lifts the job's hold, so drain works it in its
    /// priority order again. Only the job's latest hold record is looked at, so a hold is released whatever
    /// state the job is in; a job that is not on hold is refused before anything is committed.
    pub fn release_hold(&self, job_ref: &str) -> Result<JobId> {
        let head = self.head()?;
        let records = crate::store::Records::load(&self.git, &head)?;
        let job_id = records.resolve(job_ref)?;
        let job = records.jobs.get(job_id.as_str()).expect("resolved");
        if !job
            .latest_hold()
            .is_some_and(|h| h.state == HoldState::OnHold)
        {
            bail!("job {job_id} is not on hold");
        }
        self.append_hold(&job_id, job, HoldState::Released)?;
        Ok(job_id)
    }

    /// Commits the next hold record of `job` with `state`, as one records-only commit.
    fn append_hold(&self, job_id: &JobId, job: &JobRecords, state: HoldState) -> Result<()> {
        let n = job.holds.iter().map(|h| h.n).max().unwrap_or(0) + 1;
        let (subject, body) = match state {
            HoldState::OnHold => (
                format!("Hold: {job_id}"),
                "A hold record for an open request. The request is unchanged; drain does not dispatch a job whose latest hold record puts it on hold.",
            ),
            HoldState::Released => (
                format!("Release: {job_id}"),
                "A later hold record that lifts the hold. The request is unchanged; drain works the job in its priority order again.",
            ),
        };
        self.commit_records(
            &[NewFile::record(&AnyRecord::Hold(HoldRecord {
                job: job_id.clone(),
                n,
                at: crate::now(),
                state,
            }))],
            &message(
                &subject,
                body,
                &[("Athena-Job", job_id.as_str().to_string())],
            ),
        )?;
        Ok(())
    }

    /// `athena focus <text>`: appends a focus record, what the user wants worked on now. The text and the git
    /// user name are checked before anything is committed; earlier focus records are never touched, and the
    /// highest `n` is the current focus.
    pub fn set_focus(&self, text: &str) -> Result<FocusRecord> {
        let text = text.trim();
        if text.is_empty() {
            bail!(
                "the focus is empty; say what to focus on, for example: athena focus \"speed first\""
            );
        }
        let Some(set_by) = self
            .git
            .config_get("user.name")?
            .map(|name| name.trim().to_string())
            .filter(|name| !name.is_empty())
        else {
            bail!("git user.name is not set; run git config user.name \"Your Name\" and try again");
        };
        let head = self.head()?;
        let records = crate::store::Records::load(&self.git, &head)?;
        let n = records.focus.iter().map(|f| f.n).max().unwrap_or(0) + 1;
        let focus = FocusRecord {
            n,
            at: crate::now(),
            text: text.to_string(),
            set_by,
        };
        focus.validate().map_err(|e| anyhow::anyhow!(e))?;
        let record = AnyRecord::Focus(focus.clone());
        self.commit_records(
            &[NewFile::record(&record)],
            &message(
                &format!("Set focus: {}", crate::truncate_at_word(text.lines().next().unwrap_or(""), 60)),
                "A focus record: what the user wants worked on now. The latest focus record is the current focus.",
                &[("Athena-Record", record.canonical_path())],
            ),
        )?;
        Ok(focus)
    }

    /// The current focus (`athena focus`): the focus record with the highest `n`, if any was set.
    pub fn focus(&self) -> Result<Option<FocusRecord>> {
        let head = self.head()?;
        let records = crate::store::Records::load(&self.git, &head)?;
        Ok(records.current_focus().cloned())
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
        self.first_reading(&jc, &records, &request, false, None, ui)
    }

    /// Reads again a job whose first read gave no reading (the reader failed): the request is already
    /// recorded, so only the new invocation and its reading are committed. The invocation is numbered after
    /// the ones already made, and what they cost counts against the job's budget.
    pub(crate) fn read_unread(&self, job: &JobRecords, ui: &dyn Ui) -> Result<Proposal> {
        self.refuse_if_stopped()?;
        let head = self.head()?;
        let jc = JobConfig::at(&self.git, &head)?;
        let records = crate::store::Records::load(&self.git, &head)?;
        let request = job
            .request
            .clone()
            .ok_or_else(|| anyhow::anyhow!("job {} has no request record", job.id()))?;
        self.first_reading(&jc, &records, &request, false, Some(job), ui)
    }

    /// Reads `request` and commits the reading, with the request record when `with_request` is set. `prior`
    /// is the job's records when an earlier read gave no reading and this one reads it again.
    fn first_reading(
        &self,
        jc: &JobConfig,
        records: &Records,
        request: &Request,
        with_request: bool,
        prior: Option<&JobRecords>,
        ui: &dyn Ui,
    ) -> Result<Proposal> {
        let job = request.job.clone();
        let redactor = self.redactor(&jc.config, Some(&job))?;
        let attempt =
            earlier_attempt_with(records, &request.text, &redactor, &|j| self.running(j))?;
        let seq = prior.map_or(1, JobRecords::next_seq);
        let before = prior.map_or(0.0, JobRecords::spent_usd);
        // The whole path must be affordable before the reader spends anything.
        self.preflight(jc, before, ReserveStage::Admission)?;
        let (reading, invocations, error) = self.read_for(
            jc,
            records,
            &job,
            request,
            attempt.as_ref(),
            &[],
            1,
            seq,
            before,
            &redactor,
            ui,
        )?;
        let mut files = Vec::new();
        if with_request {
            files.push(NewFile::record(&AnyRecord::Request(request.clone())));
        }
        files.extend(invocation_files(&invocations));
        // What this read cost; an earlier read's cost was counted when it was made.
        let spent_usd = total_cost(&invocations);
        let invocation = invocations.last().cloned().expect("a read makes one");
        if let Some(r) = &reading {
            files.push(NewFile::record(&AnyRecord::Reading(Box::new(r.clone()))));
        }
        let again = if prior.is_some() { " again" } else { "" };
        let subject = match &reading {
            Some(r) => format!("Read a request{again}: {}", r.title),
            None => format!("Read a request{again} (the reader failed)"),
        };
        let body = failed_reading_body(
            "The request, the reader's reading with its forecasts, and the reader's invocation, committed before any work.",
            reading.is_none().then_some(error.as_deref()).flatten(),
            &redactor,
        );
        let commit = self.commit_records(
            &files,
            &message(&subject, &body, &[("Athena-Job", job.as_str().to_string())]),
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

    /// `athena retry <job>`: reads a retry of a rejected, failed, timed out, stale or interrupted job
    /// (dispatched with no process working on it, for example after a restart, or held for low disk space).
    /// A stale job keeps its candidate: `go` merges it onto the branch tip and re-verifies it, or starts a
    /// producer run seeded from the conflicted merge, or starts from the base when it cannot be merged. The
    /// reading is seeded from that job's
    /// introspection record when it has one. The request recorded is "try again on <job id>", so a later
    /// retry of this job finds the original; nothing is run. The flag says whether the introspection
    /// record seeded the reading (false: the plain "try again on" behavior). An interrupted job that already
    /// has a retry standing for it (see [`retries_by_job`]) is refused, naming that retry; a retry closed
    /// without running does not count.
    pub fn retry(&self, job_ref: &str, ui: &dyn Ui) -> Result<(Proposal, bool)> {
        let head = self.head()?;
        let records = crate::store::Records::load(&self.git, &head)?;
        let id = records.resolve(job_ref)?;
        let job = records.jobs.get(id.as_str()).expect("resolved");
        // A paused line is refused before any model is called (`scope_guard`).
        if let Some(pause) = self.line_pause(&records, id.as_str()) {
            bail!("{}", self.pause_refusal(&pause));
        }
        // A process that died (for example in a restart) no longer works on its job: drop its entry first, so
        // the job counts as interrupted below and in `ask`.
        self.state.registry().reap();
        // An interrupted job stays dispatched after it is retried, so a second retry would do the same work
        // twice. It is refused here, naming the first retry. Other states are left to `ask`.
        if job.state() == JobState::Dispatched
            && !self.running(id.as_str())
            && let Some(again) = retries_by_job(&records).get(id.as_str())
        {
            bail!(
                "job {id} was interrupted and was already retried as job {again} ({}); {}. Nothing was started.",
                records
                    .jobs
                    .get(again.as_str())
                    .map_or("unknown", |j| j.state().name()),
                retry_follow_up(&records, again, self.running(again))
            );
        }
        // A job the capability preflight stopped did no work. When it was itself a retry, a retry of it retries
        // the job it retried, so the stored request and every reader of that link see the real chain. The
        // pause check and the interrupted-retry guard above keep the job the user named.
        let target = crate::preflight::retry_target(&records, &id);
        if target != id {
            ui.line(&format!(
                "retry      job {id} stopped at the capability preflight without doing work; this retries job {target}, the job it retried"
            ));
        }
        let target_job = records.jobs.get(target.as_str()).unwrap_or(job);
        let seeded = has_retry_introspection(&records, target_job);
        // A retry is a new job, judged on its own spend, which starts at 0. The check is made here too so a
        // retry that cannot be afforded prints no resume note; `ask` makes it again before the reader runs.
        let jc = JobConfig::at(&self.git, &head)?;
        self.preflight(&jc, 0.0, ReserveStage::Admission)?;
        // `ask` refuses a job that is not rejected, failed, timed out, stale or interrupted before any model is called.
        // A job that stopped for the environment, or was interrupted, after its candidate was built resumes
        // from that candidate: `go` runs no planner or producer. The reader still runs below.
        let request = format!("{RETRY_PREFIX}{target}");
        if resumable_job(&records, &request, &|j| self.running(j)).is_some()
            && let Some(mark) = self.state.read_resume(&target)
        {
            ui.line(&format!(
                "resume     job {target} kept its candidate (stage: {}); `athena go` resumes from it with no new planner or producer run",
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
        let body = failed_reading_body(
            "The user's one-line correction, the intervention it implies, and the new reading with its forecasts.",
            reading.is_none().then_some(error.as_deref()).flatten(),
            &redactor,
        );
        let commit = self.commit_records(&files, &message(&subject, &body, &trailers))?;
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

/// The longest reader error a commit body carries, in characters.
const READER_ERROR_MAX: usize = 1000;

/// The body of a reading commit. When the reader gave no usable reading, `error` is why, and the body names
/// it, so the failed-reading commit says what went wrong. The text is redacted, put on one line and cut
/// to [`READER_ERROR_MAX`] characters; if redaction fails, the body says the text was withheld, so the
/// commit is still made.
fn failed_reading_body(base: &str, error: Option<&str>, redactor: &Redactor) -> String {
    let Some(error) = error else {
        return base.to_string();
    };
    let one_line = error.split_whitespace().collect::<Vec<_>>().join(" ");
    let shown = match redactor.redact(&one_line) {
        Ok(text) => crate::truncate_at_word(&text, READER_ERROR_MAX),
        Err(_) => "(withheld: it could not be checked for secrets)".to_string(),
    };
    format!("{base}\n\nThe reader gave no usable reading: {shown}")
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

/// The subject of a queued request's commit: the line that names the request (not a `Priority:` line), cut to
/// 60 characters.
fn queue_subject(text: &str) -> String {
    let line: String = crate::plan::title_line(text).chars().take(60).collect();
    format!("Queue a request: {line}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::records::{ChangeEntry, ChangeStatus, ProducerReport};

    #[test]
    fn a_queued_request_is_titled_by_its_request_not_its_priority() {
        let subject = queue_subject("Priority: urgent\n\nFix the widget\nmore detail");
        assert_eq!(subject, "Queue a request: Fix the widget");
        assert!(!subject.contains("Priority"), "{subject}");
        assert_eq!(
            queue_subject("Priority: urgent"),
            "Queue a request: Priority: urgent"
        );
    }

    #[test]
    fn a_queued_request_commit_names_the_request_not_its_priority() {
        let repo = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let git = athena_git::Git::at(repo.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value]).unwrap();
        }
        for (path, text) in [
            ("athena.toml", include_str!("../../../athena.toml")),
            ("defaults.toml", include_str!("../../../defaults.toml")),
            ("README.md", "# Demo\n"),
        ] {
            std::fs::write(repo.path().join(path), text).unwrap();
        }
        git.run(["add", "-A"]).unwrap();
        git.run([
            "commit",
            "-q",
            "-m",
            "Start a demo repository\n\nAthena-Job: bootstrap",
        ])
        .unwrap();
        // Queuing runs no agent, so the script is never called.
        let athena = crate::test_support::make_athena(
            &git,
            state.path(),
            vec![],
            Box::new(|_| unreachable!("queue runs no agent")),
        );
        let (_, commit) = athena.queue("Priority: urgent\n\nFix the widget").unwrap();
        let subject = git.run(["log", "-1", "--format=%s", &commit]).unwrap();
        assert_eq!(subject.trim(), "Queue a request: Fix the widget");
        assert!(!subject.contains("Priority"), "{subject}");
    }

    /// The text of this file and of its child modules, which tests that read the source search.
    fn reading_source() -> &'static str {
        static SOURCE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        SOURCE.get_or_init(|| {
            [
                include_str!("reading/retry.rs"),
                include_str!("reading/topics.rs"),
                include_str!("reading/checks.rs"),
                include_str!("reading/outcome.rs"),
                include_str!("reading/terms.rs"),
                include_str!("reading/surveys.rs"),
                include_str!("reading/replay.rs"),
                include_str!("reading.rs"),
            ]
            .join("\n")
        })
    }

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

    fn shipped_config() -> athena_data::Config {
        athena_data::Config::parse(include_str!("../../../athena.toml")).unwrap()
    }

    #[test]
    fn the_reader_input_roles_come_from_the_config_job_plan_enabled_value() {
        let mut config = shipped_config();
        config.job_plan.as_mut().unwrap().enabled = true;
        assert!(config.enabled_roles().contains(&"planner".to_string()));
        assert!(config.enabled_roles().contains(&"critic".to_string()));
        config.job_plan.as_mut().unwrap().enabled = false;
        assert_eq!(config.enabled_roles(), ["reader", "producer", "verifier"]);
        config.job_plan = None;
        assert_eq!(config.enabled_roles(), ["reader", "producer", "verifier"]);
        // read_once passes the config's roles, not a fixed list.
        assert!(reading_source().contains("enabled_roles: jc.config.enabled_roles(),"));
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn shipped_budget_covers_the_reserve() {
        // Shipped caps: reader 2.00, planner and critic 3.00 over 5 rounds, producer 10.00, verifier 5.00.
        let config = shipped_config();
        let reserve = budget_reserve(&config, ReserveStage::Admission);
        assert!(close(reserve, 47.0), "{reserve}");
        // The shipped job budget of 50.00 covers it; a recorded 8.00 is $39.00 short.
        assert!(config.budget.job_usd >= reserve);
        // The config-load check and the runtime reserve are one sum, and the shipped file passes the check.
        assert!(close(config.budget_required(), reserve));
        assert!(config.check_budget().is_ok());
        let short = BudgetShort {
            left: 8.0,
            job_usd: 8.0,
            reserve,
            stage: ReserveStage::Admission,
        };
        assert!(close(short.shortfall(), 39.0));
        let text = short.to_string();
        assert!(text.contains("$39.00 short"), "{text}");
        assert!(text.contains("[budget] job_usd"), "{text}");
        assert!(text.contains("budget left"), "{text}");
        // A reserve that left out the critic and later rounds would be smaller than the real one.
        let plan = config.job_plan.expect("the shipped config plans");
        assert!(reserve > config.budget.reader_usd + plan.planner_usd + 5.0);
    }

    #[test]
    fn every_stage_counts_only_what_is_still_to_run_and_takes_its_caps_from_config() {
        let mut config = shipped_config();
        let plan = config.job_plan.as_mut().expect("the shipped config plans");
        plan.enabled = true;
        for max_rounds in 1..=4 {
            config.job_plan.as_mut().unwrap().max_rounds = max_rounds;
            let at = |stage| budget_reserve(&config, stage);
            let plan = config.job_plan.unwrap();
            let after = config.budget.producer_usd + config.budget.verifier_usd;
            let round = plan.planner_usd + plan.critic_usd;
            assert!(close(at(ReserveStage::Run), after));
            assert!(close(at(ReserveStage::Dispatch { planned: false }), after));
            assert!(close(
                at(ReserveStage::Dispatch { planned: true }),
                f64::from(max_rounds) * round + after
            ));
            assert!(close(
                at(ReserveStage::Admission),
                config.budget.reader_usd + f64::from(max_rounds) * round + after
            ));
            // The first planner is the whole planning path; the first critic is that less the planner.
            assert!(close(
                at(ReserveStage::Planner { round: 1 }),
                at(ReserveStage::Dispatch { planned: true })
            ));
            assert!(close(
                at(ReserveStage::Critic { round: 1 }),
                at(ReserveStage::Admission) - config.budget.reader_usd - plan.planner_usd
            ));
            // The last critic needs only itself, the producer and the verifier.
            assert!(close(
                at(ReserveStage::Critic { round: max_rounds }),
                plan.critic_usd + after
            ));
        }
        // A changed cap changes the reserve with no code change.
        config.budget.verifier_usd += 1.0;
        let wider = budget_reserve(&config, ReserveStage::Run);
        assert!(close(wider, 16.0), "{wider}");
        // With planning off, admission is the reader, the producer and the verifier.
        config.job_plan = None;
        assert!(close(
            budget_reserve(&config, ReserveStage::Admission),
            18.0
        ));
    }

    #[test]
    fn required_budget_follows_the_configured_caps() {
        let base = shipped_config();
        let required = |c: &athena_data::Config| {
            let need = c.budget_required();
            assert!(close(need, budget_reserve(c, ReserveStage::Admission)));
            need
        };
        let before = required(&base);
        let rounds = f64::from(base.job_plan.unwrap().max_rounds);

        // A dearer planner raises the requirement by one dollar a round, with no other change.
        let mut dearer = base.clone();
        dearer.job_plan.as_mut().unwrap().planner_usd += 1.0;
        assert!(close(required(&dearer), before + rounds));

        // One more round adds one planner call and one critic call.
        let mut longer = base.clone();
        longer.job_plan.as_mut().unwrap().max_rounds += 1;
        let pair = base.job_plan.unwrap().planner_usd + base.job_plan.unwrap().critic_usd;
        assert!(close(required(&longer), before + pair));

        // The check follows: a budget a cent under the new requirement is refused, exactly it is accepted.
        for changed in [&mut dearer, &mut longer] {
            let need = required(changed);
            changed.budget.job_usd = need - 0.01;
            assert!(changed.check_budget().is_err());
            changed.budget.job_usd = need;
            assert!(changed.check_budget().is_ok());
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
    fn a_size_refusal_carries_its_rule_name_and_any_other_invalid_output_does_not() {
        // The rule is read from the error by `read_once`; the text is the rule's own, unchanged.
        let size = build_sized(sized_reading(5000.0, Some("study")), 3000, 10_000).unwrap_err();
        let named = size
            .downcast_ref::<NamedRefusal>()
            .expect("a named refusal");
        assert_eq!(named.rule, "check_reading_size");
        assert!(
            named.text.contains("pick one deliverable"),
            "{}",
            named.text
        );
        assert_eq!(size.to_string(), named.text);
        // A reading with an unknown class fails earlier and names no rule: `read_once` files it as `other`.
        let mut bad_class = sized_reading(30.0, Some("study"));
        bad_class["class_of_work"] = serde_json::json!("not-a-class");
        let other = build_sized(bad_class, 3000, 2400).unwrap_err();
        assert!(other.downcast_ref::<NamedRefusal>().is_none(), "{other:#}");
        assert_eq!(OTHER_READING_RULE, "other");
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
    fn a_reading_commits_the_reader_forecast_and_a_reference_before_dispatch() {
        let reading = build_sized(sized_reading(30.0, Some("study")), 3000, 2400).unwrap();
        let reader = &reading.forecasts.reader;
        assert_eq!(reader.p_reading_accepted, 0.8);
        assert_eq!(reader.p_verification_passes, 0.7);
        assert_eq!(reader.p_user_intervenes, 0.1);
        assert_eq!(reader.cost_usd.p50(), 0.2);
        assert_eq!(reader.wall_time_s.p50(), 30.0);
        assert!(reading.forecasts.reference.validate().is_ok());
        // A reading without a forecast is refused: no work is dispatched unpredicted.
        let mut bare = sized_reading(30.0, Some("study"));
        bare.as_object_mut().unwrap().remove("forecast");
        let error = format!("{:#}", build_sized(bare, 3000, 2400).unwrap_err());
        assert!(error.contains("parsing the reading"), "{error}");
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
            failed_tests: Vec::new(),
            reused: false,
            revert_for: Vec::new(),
            queued_s: None,
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

    fn introspection_of(
        job: &JobRecords,
        n: u32,
        kind: PreventionKind,
        prevention: &str,
    ) -> IntrospectionRecord {
        IntrospectionRecord {
            id: athena_data::RecordId::parse(&format!("20261012-0900{n:02}-1abc")).unwrap(),
            at: format!("2026-10-12T09:00:{n:02}Z").parse().unwrap(),
            trigger: IntrospectionTrigger::Rejection,
            job: Some(job.id().clone()),
            intervention: None,
            class_of_work: "code".into(),
            category: "rejected".into(),
            what_happened: Some("The job was rejected.".into()),
            failure_class: None,
            new_class: false,
            earlier_cases: Vec::new(),
            decisions_matched: Vec::new(),
            cause: Some("A cause.".into()),
            prevention_kind: Some(kind),
            prevention: Some(prevention.into()),
            proposed_request: Some("Do the thing.".into()),
            evidence: None,
            status: IntrospectionStatus::Recorded,
            failure: None,
            flags: Vec::new(),
            other_causes: Vec::new(),
            open_hypotheses: Vec::new(),
            contradictions: Vec::new(),
            dropped_ids: Vec::new(),
            planning_stopped: false,
            from_jev: false,
            jev: None,
        }
    }

    #[test]
    fn retry_seed_skips_relax_and_remove_introspections() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let job = job_with_checks(vec![check("test", false, "failed")]);
        let mut records = records_with(job.clone());

        // A relax or a remove record alone seeds nothing.
        for kind in [PreventionKind::Relax, PreventionKind::Remove] {
            records.introspections = vec![introspection_of(
                &job,
                1,
                kind,
                "[gate: test] Loosen the check.",
            )];
            assert!(retry_introspection(&records, &job).is_none(), "{kind:?}");
            assert!(!has_retry_introspection(&records, &job), "{kind:?}");
            assert!(retry_cause(&records, &job, &redactor).unwrap().is_none());
        }

        // An older code-check record still seeds, and the newer relax record is not what is carried.
        let older = introspection_of(&job, 1, PreventionKind::CodeCheck, "Add a check.");
        let newer = introspection_of(&job, 2, PreventionKind::Relax, "[gate: test] Loosen it.");
        records.introspections = vec![older.clone(), newer.clone()];
        assert_eq!(retry_introspection(&records, &job).unwrap().id, older.id);
        let cause = retry_cause(&records, &job, &redactor).unwrap().unwrap();
        assert_eq!(cause.introspection, older.id.to_string());
        assert!(!cause.prevention.contains("Loosen"), "{}", cause.prevention);

        // A relax record cited as an earlier case of the seed is not carried either.
        let mut seed = older.clone();
        seed.earlier_cases = vec![newer.id.clone()];
        records.introspections = vec![seed, newer];
        let cause = retry_cause(&records, &job, &redactor).unwrap().unwrap();
        assert!(cause.earlier_cases.is_empty(), "{:?}", cause.earlier_cases);
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
            dependents: vec![],
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
        for state in [OutcomeState::Landed, OutcomeState::Abandoned] {
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

    #[test]
    fn a_stale_job_is_accepted_and_its_reason_is_carried() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let mut job = job_with_checks(vec![]);
        job.outcome = Some(outcome(
            &job,
            OutcomeState::Stale,
            "the branch moved: a governed file changed",
        ));
        let id = job.id().as_str().to_string();
        let attempt = earlier_attempt(&records_with(job), &format!("try again on {id}"), &redactor)
            .unwrap()
            .expect("a stale job can be tried again");
        assert_eq!(attempt.end_state, "stale");
        assert!(!attempt.failed);
        assert!(
            attempt
                .findings
                .iter()
                .any(|f| f == "outcome: the branch moved: a governed file changed"),
            "{:?}",
            attempt.findings
        );
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

    #[test]
    fn changelog_unproduced_asks_for_the_root_changelog_on_a_change_reading() {
        let mut reading = change_reading();
        ensure_changelog(&mut reading);
        // A docs-only candidate skipped the changelog.
        assert!(changelog_unproduced(&reading, ["docs/a.md"]));
        assert!(changelog_unproduced(&reading, []));
        // Touching the root file, in any form, satisfies it.
        assert!(!changelog_unproduced(
            &reading,
            ["docs/a.md", "CHANGELOG.md"]
        ));
        assert!(!changelog_unproduced(&reading, ["./CHANGELOG.md"]));
        // Another file of that name is not the deliverable.
        assert!(changelog_unproduced(&reading, ["docs/CHANGELOG.md"]));

        let mut study = reading.clone();
        study.capability = Some("study".into());
        assert!(!changelog_unproduced(&study, ["docs/a.md"]));

        // Without the default the rule is not asked for.
        let mut plain = reading.clone();
        plain.defaults_applied.retain(|d| d.id != "changelog");
        assert!(!changelog_unproduced(&plain, ["docs/a.md"]));

        // Only a modify deliverable asks for an edit.
        let mut created = reading.clone();
        for d in &mut created.deliverables {
            if d.path == "CHANGELOG.md" {
                d.action = DeliverableAction::Create;
            }
        }
        assert!(!changelog_unproduced(&created, ["docs/a.md"]));
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
        assert!(
            entry.depends_on.is_empty(),
            "an id after 'after' is a reference, not a dependency"
        );
        assert_eq!(
            entry.rung,
            Some(Ladder::Implementation),
            "inferred from the path"
        );
        assert!(entry.missing().is_empty());

        job.request.as_mut().unwrap().text = format!(
            "{}\nDepends on 20261012-075824-1x28",
            job.request.as_ref().unwrap().text
        );
        let entry = plan_entry(&job, None, &everything);
        assert_eq!(entry.depends_on, ["20261012-075824-1x28"]);
        job.request.as_mut().unwrap().text =
            "Add a greeting to crates/cli/src/main.rs after 20261012-075824-1x28\n- it prints hello\n- cargo test passes"
                .into();

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
    fn the_plan_entry_title_skips_a_leading_priority_line() {
        for (text, title) in [
            ("Priority: urgent\n\nFix the widget", "Fix the widget"),
            ("  priority: high\nFix the widget", "Fix the widget"),
            ("Fix the widget\nmore", "Fix the widget"),
            ("Priority: high\nPriority: low", "Priority: high"),
        ] {
            let mut job = job_with_checks(vec![]);
            job.request.as_mut().unwrap().text = text.into();
            job.readings.clear();
            assert_eq!(
                plan_entry(&job, None, &nothing_known()).title,
                title,
                "{text:?}"
            );
        }
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
            "Update crates/gone/src/x.rs and docs/here.md\nDepends on 20261012-075824-1x28\n- it works"
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

    const REJECTED: &str = "20261012-075824-1x28";

    /// The plan entry of an unread request with `text`, planned as an implementation request.
    fn entry_of_text(text: &str) -> crate::plan::PlanEntry {
        let mut job = job_with_checks(vec![]);
        job.request.as_mut().unwrap().text = text.into();
        let everything = Lookup {
            job_exists: &|_| true,
            path_exists: &|_| true,
            brief: &[],
        };
        let entry = plan_entry(
            &job,
            Some(athena_data::roles::Ladder::Implementation),
            &everything,
        );
        assert!(entry.missing().is_empty(), "{:?}", entry.missing());
        entry
    }

    fn rejected_history() -> Vec<crate::plan::JobEnd> {
        vec![crate::plan::JobEnd {
            id: REJECTED.into(),
            ending: crate::plan::Ending::Failed("rejected".into()),
            retry_of: None,
        }]
    }

    #[test]
    fn a_request_that_only_cites_a_rejected_job_is_planned_not_held() {
        let entry = entry_of_text(&format!(
            "Add a greeting to crates/cli/src/main.rs\nJob {REJECTED} was rejected, see its findings.\n- it prints hello"
        ));
        assert_ne!(entry.id, REJECTED);
        assert!(entry.depends_on.is_empty(), "{:?}", entry.depends_on);
        let plan = crate::plan::Plan::build_with(std::slice::from_ref(&entry), &rejected_history());
        assert_eq!(plan.order, [entry.id.as_str()], "{plan:?}");
        assert!(plan.refused.is_empty(), "{:?}", plan.refused);
    }

    #[test]
    fn a_depends_on_line_naming_a_rejected_job_with_no_landed_retry_still_holds_the_item() {
        let entry = entry_of_text(&format!(
            "Add a greeting to crates/cli/src/main.rs\nDepends on {REJECTED}\n- it prints hello"
        ));
        let plan = crate::plan::Plan::build_with(std::slice::from_ref(&entry), &rejected_history());
        assert!(plan.order.is_empty(), "{plan:?}");
        let held = plan.refused.iter().find(|r| r.id == entry.id).unwrap();
        assert!(held.missing.is_empty());
        assert!(
            held.reason.contains(&format!("depends on {REJECTED}"))
                && held.reason.contains("rejected"),
            "{}",
            held.reason
        );

        // A landed retry of the rejected job releases the item.
        let mut history = rejected_history();
        history.push(crate::plan::JobEnd {
            id: "20261013-000000-abcd".into(),
            ending: crate::plan::Ending::Landed,
            retry_of: Some(REJECTED.into()),
        });
        let plan = crate::plan::Plan::build_with(std::slice::from_ref(&entry), &history);
        assert_eq!(plan.order, [entry.id.as_str()], "{plan:?}");
        assert!(plan.refused.is_empty());
    }

    #[test]
    fn dependencies_are_only_depends_on_ids_and_the_retry_link() {
        let other = "20261011-000000-abcd";
        let depends = |text: &str| -> Vec<String> {
            dependency_ids_in(text)
                .iter()
                .map(|id| id.as_str().to_string())
                .collect()
        };

        // The retry link is a dependency, and a retry is not held by the job it retries.
        let retry = format!("try again on {REJECTED}");
        assert_eq!(depends(&retry), [REJECTED]);
        let retry_with_dot = format!("Try again on {REJECTED}.");
        assert_eq!(depends(&retry_with_dot), [REJECTED]);
        assert!(
            depends("try again on 1x28").is_empty(),
            "short ids never parse"
        );

        // An id on a 'Depends on' line is a dependency, once, wherever else it is cited.
        let both =
            format!("See {REJECTED} for why.\nDepends on {REJECTED}\nIt cites {REJECTED} again.");
        assert_eq!(depends(&both), [REJECTED]);

        // The same id elsewhere in the text is not; the one on the line is.
        let mixed = format!("Evidence is {REJECTED}.\n> - Depends on: {other}");
        assert_eq!(depends(&mixed), [other]);

        // Shapes of a 'Depends on' line.
        for line in [
            format!("Depends on {other}"),
            format!("depends on {other}"),
            format!("DEPENDS ON: {other}"),
            format!("  > Depends on {other}"),
            format!("- Depends on {other}"),
            format!("* **Depends on:** {other}"),
            format!("2. Depends on {other}"),
            format!("3) Depends on {other}."),
        ] {
            assert_eq!(depends(&format!("Goal\n{line}\n- done")), [other], "{line}");
        }
        let two = format!("Depends on {other} and {REJECTED}");
        assert_eq!(depends(&two), [other, REJECTED]);

        // Other wordings are references.
        for text in [
            format!("Add a greeting after {REJECTED}"),
            format!("Follows up on {REJECTED}"),
            format!("The fix depends on the evidence in {REJECTED}"),
            format!("Dependsonx {REJECTED}"),
            format!("Depends on\n{REJECTED}"),
        ] {
            assert!(depends(&text).is_empty(), "{text}");
        }

        // Through the plan: a retry of a rejected job is ordered, not held by it.
        let entry = entry_of_text(&format!("{retry}\n- it works"));
        assert_eq!(entry.depends_on, [REJECTED]);
        let history = vec![
            rejected_history().remove(0),
            crate::plan::JobEnd {
                id: entry.id.clone(),
                ending: crate::plan::Ending::Other,
                retry_of: Some(REJECTED.into()),
            },
        ];
        let plan = crate::plan::Plan::build_with(std::slice::from_ref(&entry), &history);
        assert_eq!(plan.order, [entry.id.as_str()], "{plan:?}");

        // The job's own id is never a dependency.
        let own = job_with_checks(vec![]).id().as_str().to_string();
        let entry = entry_of_text(&format!("Add a thing to a.rs\nDepends on {own}\n- done"));
        assert!(entry.depends_on.is_empty());
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
                    candidate: None,
                    conflicts: Vec::new(),
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

    #[test]
    fn retry_findings_carry_the_reason_of_a_job_closed_by_the_decision_gate() {
        use athena_data::decision_links::GATE_REASON_PREFIX;
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        // A job the decision check closed has no verification: it never reached the verifier.
        let gated = |state: OutcomeState, reason: &str| {
            let mut job = two_majors_and_a_minor();
            job.verification = None;
            job.outcome = Some(outcome(&job, state, reason));
            job
        };
        let reason = format!(
            "{GATE_REASON_PREFIX} touches D02 (README.md) and D12 (src/a.rs); add an entry"
        );
        let job = gated(OutcomeState::Failed, &reason);
        let id = job.id().clone();
        let request = format!("try again on {id}");
        let found = retry_findings(&records_with(job), &request, &redactor)
            .unwrap()
            .expect("the gate's reason is a finding");
        assert_eq!(found, (id.clone(), vec![reason.clone()]));

        // The same words without the prefix, or a job that did not close failed, carry nothing.
        for job in [
            gated(
                OutcomeState::Failed,
                "the verifier returned no valid verdict",
            ),
            gated(OutcomeState::Failed, &reason[GATE_REASON_PREFIX.len()..]),
            gated(OutcomeState::Landed, &reason),
        ] {
            let request = format!("try again on {}", job.id());
            assert_eq!(
                retry_findings(&records_with(job), &request, &redactor).unwrap(),
                None
            );
        }

        // A job that has a verification still answers from its findings, whatever its reason says.
        let mut verified = two_majors_and_a_minor();
        verified.outcome = Some(outcome(&verified, OutcomeState::Failed, &reason));
        let request = format!("try again on {}", verified.id());
        let (_, texts) = retry_findings(&records_with(verified), &request, &redactor)
            .unwrap()
            .expect("findings");
        assert_eq!(texts.len(), 2, "{texts:?}");
        assert!(texts.iter().all(|t| !t.starts_with(GATE_REASON_PREFIX)));
    }

    #[test]
    fn retry_findings_carry_the_reason_of_a_job_closed_by_the_allowed_paths_check() {
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let scoped = |state: OutcomeState, reason: &str| {
            let mut job = two_majors_and_a_minor();
            job.verification = None;
            job.outcome = Some(outcome(&job, state, reason));
            job
        };
        let reason = format!(
            "{SCOPE_REASON_PREFIX} NOTES.md, src/extra.rs (criterion \"No file outside X changed\"); revert each"
        );
        let job = scoped(OutcomeState::Failed, &reason);
        let id = job.id().clone();
        let request = format!("try again on {id}");
        let found = retry_findings(&records_with(job), &request, &redactor)
            .unwrap()
            .expect("the scope reason is a finding");
        assert_eq!(found, (id, vec![reason.clone()]));

        // The same words without the prefix, or a job that did not close failed, carry nothing.
        for job in [
            scoped(OutcomeState::Failed, &reason[SCOPE_REASON_PREFIX.len()..]),
            scoped(OutcomeState::Landed, &reason),
        ] {
            let request = format!("try again on {}", job.id());
            assert_eq!(
                retry_findings(&records_with(job), &request, &redactor).unwrap(),
                None
            );
        }
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
            no_change: None,
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

    #[test]
    fn carry_shows_a_revert_run_as_a_revert_run_not_a_pass_or_a_failure() {
        let mut revert = check("fails-without-1", true, "test demo ... FAILED");
        revert.exit_code = Some(101);
        revert.revert_for = vec![1];
        let job = job_with_checks(vec![check("fmt", true, ""), revert]);
        let carried = carry_of(&job);
        assert_eq!(
            carried.checks[1],
            "fails-without-1: revert run (exit code 101)"
        );
        assert!(
            !carried.checks.iter().any(|c| c.contains("failed")),
            "{:?}",
            carried.checks
        );
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
        let source = reading_source();
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
            let mut named = governed_reading(
                &["src/site.rs", &file],
                &["the site builds", &format!("{file} runs the build")],
                &["the logo"],
            );
            add_behavior(&mut named, SERVED_STATE);
            named.assumptions.push(CI_ENVIRONMENT.into());
            check_reading_acceptance(&named, request, None, &[], None)
                .unwrap_or_else(|e| panic!("{request}: {e}"));
            // Put out of scope by name: accepted without the file.
            let mut excluded = governed_reading(
                &["src/site.rs"],
                &["the site builds"],
                &[&format!("Editing {}", path.trim_end_matches('/'))],
            );
            add_behavior(&mut excluded, SERVED_STATE);
            check_reading_acceptance(&excluded, request, None, &[], None)
                .unwrap_or_else(|e| panic!("{request}: {e}"));
        }
    }

    #[test]
    fn a_request_with_only_common_words_and_no_governance_path_is_accepted() {
        // None of these words names a path, so none requires a governance file; the reading lists none.
        let mut bare = governed_reading(&["src/site.rs"], &["the site builds"], &["the logo"]);
        add_behavior(&mut bare, SERVED_STATE);
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
        let mut with_workflow = governed_reading(
            &["docs/index.md", ".github/workflows/pages.yml"],
            &["pages.yml deploys the site"],
            &[],
        );
        add_behavior(&mut with_workflow, SERVED_STATE);
        with_workflow.assumptions.push(CI_ENVIRONMENT.into());
        check_reading_acceptance(&with_workflow, request, None, &[], None).unwrap();
    }

    /// A complete `Environment:` assumption for a CI site.
    const CI_ENVIRONMENT: &str =
        "Environment: the CI job has write permission to refs on the GitHub origin";

    /// A behavior criterion that names an observable state.
    const SERVED_STATE: &str = "the site is served at a reachable URL";

    /// Adds a criterion of kind behavior with this text to the reading.
    fn add_behavior(reading: &mut Reading, text: &str) {
        reading.acceptance_criteria.push(Criterion {
            text: text.into(),
            kind: CriterionKind::Behavior,
        });
    }

    /// Adds a criterion of kind content with this text to the reading.
    fn add_content(reading: &mut Reading, text: &str) {
        reading.acceptance_criteria.push(Criterion {
            text: text.into(),
            kind: CriterionKind::Content,
        });
    }

    /// The request of job 20261014-082328-g6vy, as recorded.
    const G6VY_REQUEST: &str = "Publish a first explainer page on how firstmate handles work.";

    #[test]
    fn check_outcome_state_refuses_the_g6vy_misread() {
        // The reading of job 20261014-082328-g6vy: a page file exists, nothing says anyone can reach it.
        let reading = governed_reading(&["docs/explainer.md"], &["docs/explainer.md exists"], &[]);
        let error = check_outcome_state(&reading, G6VY_REQUEST)
            .unwrap_err()
            .to_string();
        assert!(error.contains("reachable URL"), "{error}");
        assert!(error.contains("served page"), "{error}");
        assert!(error.contains("publish"), "{error}");
        assert!(!error.contains('\n'), "{error}");
        assert!(
            error.starts_with("the reading was refused because"),
            "{error}"
        );
        // The gate runs it and names it as the check that refused.
        let gate = check_reading_acceptance(&reading, G6VY_REQUEST, None, &[], None)
            .unwrap_err()
            .to_string();
        assert_eq!(gate, error);
        let (name, _) = refusing_check(&reading, G6VY_REQUEST, None, &[], None).unwrap();
        assert_eq!(name, "check_outcome_state");
        // With a behavior criterion that names the state, it is accepted.
        let mut stated = reading.clone();
        add_behavior(&mut stated, "The page is reachable at its public URL");
        check_reading_acceptance(&stated, G6VY_REQUEST, None, &[], None).unwrap();
    }

    #[test]
    fn check_outcome_state_accepts_a_behavior_state_for_each_delivery_verb() {
        let pairs = [
            (
                "Publish the explainer page",
                "The page is reachable at its public URL",
            ),
            (
                "Deploy the site on every push",
                "The site is served at the deployed address",
            ),
            (
                "Share the report with the team",
                "The shared link is reachable by the recipient",
            ),
            ("Release v1", "The v1 release assets are downloadable"),
        ];
        for (request, state) in pairs {
            let file_only = governed_reading(&["docs/x.md"], &["docs/x.md exists"], &[]);
            let error = check_reading_acceptance(&file_only, request, None, &[], None)
                .unwrap_err()
                .to_string();
            assert!(error.contains("reachable URL"), "{request}: {error}");
            // A content criterion that mentions the state still checks only a file.
            let mut content = file_only.clone();
            add_content(&mut content, state);
            assert!(
                check_reading_acceptance(&content, request, None, &[], None).is_err(),
                "{request}: a content criterion must not satisfy the check"
            );
            // The same text as a behavior criterion does.
            let mut behavior = file_only.clone();
            add_behavior(&mut behavior, state);
            check_reading_acceptance(&behavior, request, None, &[], None)
                .unwrap_or_else(|e| panic!("{request}: {e}"));
        }
        // A request with no delivery verb is not touched, whatever the criteria say.
        let file_only = governed_reading(&["src/config.rs"], &["the setting parses"], &[]);
        for request in [
            "Add a timeout setting",
            "Add release notes for 0.4 and a share button.",
        ] {
            check_reading_acceptance(&file_only, request, None, &[], None)
                .unwrap_or_else(|e| panic!("{request}: {e}"));
        }
    }

    #[test]
    fn delivery_verbs_are_found_after_lead_ins_and_markers() {
        let found: [(&str, &str); 12] = [
            ("Build the docs and publish them", "publish"),
            ("Can you deploy the site", "deploy"),
            ("I want to release v1", "release"),
            ("* Publish the page", "publish"),
            ("- Publish the page", "publish"),
            ("1. Share the report", "share"),
            ("2) Deploy the site", "deploy"),
            ("(a) Release v1", "release"),
            ("Priority: high\n\nPublish the page", "publish"),
            ("Publish the crate to crates.io", "publish"),
            ("Release the binary as v1", "release"),
            ("Update it. Please share the report with the team.", "share"),
        ];
        for (request, verb) in found {
            let verbs = delivery_verbs(request);
            assert!(
                verbs.iter().any(|(v, _)| *v == verb),
                "{request:?}: found {verbs:?}"
            );
        }
        let not_found = [
            "Add release notes",
            "Add a share button",
            "Rename shared helpers",
            "Add tests for share and release.",
            "We deployed it",
            "Add a timeout setting",
            "Release the lock before awaiting",
            "Workers should share the HTTP client",
            "Publish the event to the bus",
            "Share the parsing code between crates",
            "Workers should share the package cache",
            "Release the image buffer before returning",
            "and release the slot on failure",
            "Fix the page, then see the release process",
            "Bump the version to v1.2 and tag the release",
        ];
        for request in not_found {
            let verbs = delivery_verbs(request);
            assert!(verbs.is_empty(), "{request:?}: found {verbs:?}");
        }
        // A dot inside a name or a version does not end a segment; a full stop does.
        let words = request_words("Use crates.io v1.2. Share it");
        let texts: Vec<&str> = words.iter().map(|w| w.text.as_str()).collect();
        assert_eq!(texts, ["use", "crates.io", "v1.2", "share", "it"]);
        let starts: Vec<bool> = words.iter().map(|w| w.segment_start).collect();
        assert_eq!(starts, [true, false, false, true, false]);
    }

    #[test]
    fn out_of_scope_and_incidental_words_do_not_satisfy_the_check() {
        // A substring or a noun use in out_of_scope is not the delivery put out of scope.
        for (request, item) in [
            (
                "Share the report with the team",
                "Changes to shared helpers",
            ),
            ("Release v1", "Release notes are unchanged"),
        ] {
            let reading = governed_reading(&["docs/x.md"], &["docs/x.md exists"], &[item]);
            assert!(
                check_reading_acceptance(&reading, request, None, &[], None).is_err(),
                "{request}: {item}"
            );
        }
        // The delivery named as a whole word, or in its -ing form, is.
        for (request, item) in [
            (
                "Share the report with the team",
                "Sharing the report is out of scope",
            ),
            ("Release v1", "Releasing is out of scope: no tag is cut"),
            ("Publish the explainer page", "Do not publish it"),
        ] {
            let reading = governed_reading(&["docs/x.md"], &["docs/x.md exists"], &[item]);
            check_reading_acceptance(&reading, request, None, &[], None)
                .unwrap_or_else(|e| panic!("{request}: {item}: {e}"));
        }
        // With two verbs, each needs naming.
        let reading = governed_reading(
            &["docs/x.md"],
            &["docs/x.md exists"],
            &["Publishing is out of scope"],
        );
        let error = check_outcome_state(&reading, "Publish the page. Deploy the site.")
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("deploy") && !error.contains("publish"),
            "{error}"
        );
        // Words that occur in file-only criteria do not name a state, even in a behavior criterion.
        for text in [
            "the <title> tag names the page",
            "The URL field is documented",
            "A reader can open docs/explainer.md",
            "docs/explainer.md is available at docs/",
            "The link in the page serves as a title",
        ] {
            let mut reading = governed_reading(&["docs/x.md"], &["docs/x.md exists"], &[]);
            add_behavior(&mut reading, text);
            assert!(
                check_reading_acceptance(&reading, "Publish the explainer page", None, &[], None)
                    .is_err(),
                "{text}"
            );
        }
    }

    #[test]
    fn the_outcome_state_check_skips_readings_that_deliver_nothing() {
        let request = "Publish the explainer page";
        let mut declined = governed_reading(&["docs/x.md"], &["docs/x.md exists"], &[]);
        declined.capability = None;
        declined.decline_reason = Some("no capability fits".into());
        check_outcome_state(&declined, request).unwrap();
        let mut asking = governed_reading(&["docs/x.md"], &["docs/x.md exists"], &[]);
        asking.question = Some("Which page?".into());
        check_outcome_state(&asking, request).unwrap();
        let none = governed_reading(&[], &["docs/x.md exists"], &[]);
        check_outcome_state(&none, request).unwrap();
    }

    #[test]
    fn check_outcome_state_doc_names_root_cause_and_scope() {
        let source = reading_source();
        let body = source
            .split("pub fn refusing_check(")
            .nth(1)
            .and_then(|rest| rest.split("\n}\n").next())
            .unwrap();
        let steps: Vec<&str> = body
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with("step!(check_"))
            .collect();
        let governance = steps
            .iter()
            .position(|l| l.starts_with("step!(check_governance_coverage("))
            .unwrap();
        assert_eq!(
            steps[governance + 1],
            "step!(check_outcome_state(reading, request));"
        );
        let doc = source
            .split("pub fn check_outcome_state(")
            .next()
            .unwrap()
            .rsplit("\n\n")
            .next()
            .unwrap();
        assert!(
            doc.lines().all(|l| l.starts_with("///") && l != "///"),
            "{doc}"
        );
        let doc = doc
            .lines()
            .map(|l| l.trim_start_matches('/').trim())
            .collect::<Vec<_>>()
            .join(" ");
        for needle in [
            "delivery verb was read as a narrow deliverable",
            "no check against the state the user observes",
            "covers 1 of the 3 cited jobs",
            "20261014-082328-g6vy",
            "20261014-174815-7bf8",
            "20261017-173508-0wtc",
            "outside the check",
            "check_defined_terms",
        ] {
            assert!(doc.contains(needle), "the doc comment lacks {needle:?}");
        }
        assert!(!doc.contains("which does not catch them"), "{doc}");
        let acceptance_doc = source
            .split("pub fn check_reading_acceptance(")
            .next()
            .unwrap();
        assert!(acceptance_doc.contains("[`check_outcome_state`]"));
    }

    #[test]
    fn outcome_state_refusal_is_declared_intended() {
        let declared = replay_declaration(
            &[Criterion {
                text: "The change refuses more readings: expected 5% of newly refused readings"
                    .into(),
                kind: CriterionKind::Behavior,
            }],
            &[],
        );
        assert_eq!(declared.share, Some(5));
        let source = reading_source();
        let doc = source
            .split("pub fn check_outcome_state(")
            .next()
            .unwrap()
            .rsplit("\n\n")
            .next()
            .unwrap()
            .lines()
            .map(|l| l.trim_start_matches('/').trim())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            doc.contains("This check refuses more readings; the higher refusal rate is intended.")
        );
        assert!(doc.contains("estimate"));
        let changelog = include_str!("../../../CHANGELOG.md")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        for needle in [
            "check_outcome_state",
            "refuses more readings",
            "higher refusal rate is intended",
            "expected 5% of newly refused readings",
            "estimate",
        ] {
            assert!(changelog.contains(needle), "the changelog lacks {needle:?}");
        }
    }

    /// A complete `Environment:` assumption for a local working copy.
    const LOCAL_ENVIRONMENT: &str = "Environment: the local Athena working copy has write access to the repo; \
         with no origin or a non-GitHub origin it skips the push; with a GitHub origin it can push refs";

    /// The reading with this `Environment:`-style assumption added.
    fn with_environment(mut reading: Reading, text: &str) -> Reading {
        reading.assumptions.push(text.into());
        reading
    }

    /// A docs reading with these behavior criteria.
    fn behavior_reading(texts: &[&str]) -> Reading {
        let mut reading = governed_reading(&["docs/guide.md"], &["docs/guide.md exists"], &[]);
        for text in texts {
            add_behavior(&mut reading, text);
        }
        reading
    }

    /// A docs reading whose summary is this text.
    fn summary_reading(summary: &str) -> Reading {
        let mut reading = behavior_reading(&[]);
        reading.summary = summary.into();
        reading
    }

    /// A docs reading with a workflow deliverable of this action.
    fn workflow_reading(action: DeliverableAction) -> Reading {
        let mut reading = governed_reading(
            &["docs/guide.md", ".github/workflows/lint.yml"],
            &[
                "docs/guide.md exists",
                ".github/workflows/lint.yml runs lint",
            ],
            &[],
        );
        reading.deliverables[1].action = action;
        reading
    }

    /// The name of the check that refuses the reading, or `None`.
    fn refuser(reading: &Reading) -> Option<&'static str> {
        refusing_check(&with_design(reading), "Review the guide", None, &[], None)
            .map(|(name, _)| name)
    }

    #[test]
    fn environment_reach_refuses_a_workflow_deliverable_without_an_environment() {
        for action in [DeliverableAction::Create, DeliverableAction::Modify] {
            let reading = workflow_reading(action);
            assert_eq!(refuser(&reading), Some("check_environment_reach"));
            let error = check_environment_reach(&reading).unwrap_err().to_string();
            assert!(
                error.starts_with("the reading was refused because"),
                "{error}"
            );
            assert!(error.contains("`Environment:`"), "{error}");
            assert!(error.contains(".github/workflows/lint.yml"), "{error}");
            let fixed = with_environment(reading, CI_ENVIRONMENT);
            assert_eq!(refuser(&fixed), None);
        }
        // Deleting a workflow assigns no action.
        assert_eq!(refuser(&workflow_reading(DeliverableAction::Delete)), None);
    }

    #[test]
    fn environment_reach_refuses_each_host_action_without_an_environment() {
        for text in [
            "Athena pushes the job branch to origin",
            "The job branch is pushed to the remote",
            "A pull request is opened on the remote",
            "The pull request is opened with `gh pr create`",
            "Run `git push`: the branch is published",
            "Athena opens the job's PR at verification start",
            "Open the job's PR early at verification start",
            "The early PR is reused by land",
            "The PR was merged by land",
            "Land is opening a pull request",
            "A pull request is raised for the job",
            "land calls the GitHub API to reuse the early PR",
            "The check calls the GitHub API for the run status",
            "The CI job writes the records",
            "CI commits the records",
        ] {
            let reading = behavior_reading(&[text]);
            assert_eq!(refuser(&reading), Some("check_environment_reach"), "{text}");
            let error = check_environment_reach(&reading).unwrap_err().to_string();
            assert!(
                error.starts_with("the reading was refused because"),
                "{error}"
            );
            assert!(error.contains("`Environment:`"), "{text}: {error}");
            let fixed = with_environment(reading, CI_ENVIRONMENT);
            assert_eq!(refuser(&fixed), None, "{text}");
        }
    }

    #[test]
    fn environment_reach_needs_a_site_a_permission_and_the_data() {
        let reading = behavior_reading(&["Athena pushes the job branch to origin"]);
        // No permission word: refused, and the error names the permissions.
        let no_permission = with_environment(
            reading.clone(),
            "Environment: the CI job reaches refs on the GitHub origin",
        );
        let error = check_environment_reach(&no_permission)
            .unwrap_err()
            .to_string();
        assert!(error.contains("permissions"), "{error}");
        // No site word.
        let no_site = with_environment(
            reading.clone(),
            "Environment: it has write access to refs on the origin",
        );
        let error = check_environment_reach(&no_site).unwrap_err().to_string();
        assert!(error.contains("the site"), "{error}");
        // No data word.
        let no_data = with_environment(reading.clone(), "Environment: the CI job may write");
        let error = check_environment_reach(&no_data).unwrap_err().to_string();
        assert!(error.contains("refs or data"), "{error}");
        // A local site must also state the case with no origin.
        let local_only = with_environment(
            reading.clone(),
            "Environment: the local working copy has write access to the repo",
        );
        let error = check_environment_reach(&local_only)
            .unwrap_err()
            .to_string();
        assert!(error.contains("no origin"), "{error}");
        assert_eq!(
            refuser(&with_environment(reading.clone(), LOCAL_ENVIRONMENT)),
            None
        );
        // A CI site always has an origin, so it needs no such case.
        assert_eq!(
            refuser(&with_environment(reading.clone(), CI_ENVIRONMENT)),
            None
        );
        // The label matches after trimming, in any case; the word alone does not count.
        let spaced = with_environment(
            reading.clone(),
            "  ENVIRONMENT: the CI job has write permission to refs on the GitHub origin",
        );
        assert_eq!(refuser(&spaced), None);
        let not_a_label = with_environment(
            reading,
            "The environment is a CI job with write permission to refs on the GitHub origin",
        );
        assert_eq!(refuser(&not_a_label), Some("check_environment_reach"));
    }

    #[test]
    fn environment_reach_is_skipped_for_a_decline_and_a_question() {
        let mut declined = behavior_reading(&["Athena pushes the job branch to origin"]);
        declined.capability = None;
        declined.decline_reason = Some("no capability fits".into());
        check_environment_reach(&declined).unwrap();
        let mut asking = behavior_reading(&["Athena pushes the job branch to origin"]);
        asking.question = Some("Which remote?".into());
        check_environment_reach(&asking).unwrap();
    }

    /// The Behavior criterion of the shape of job 20261024-220048-engm: code that reads the records branch.
    const ENGM_CRITERION: &str = "The history check reads the job records on the records branch";

    #[test]
    fn environment_reach_source_refuses_a_records_branch_read_in_the_engm_shape() {
        let mut reading = code_reading(&[SURVEY], &[MODIFY_READING]);
        add_behavior(&mut reading, ENGM_CRITERION);
        // The criterion names the history check, so it also needs the realistic-branch fixture; with that
        // criterion added the environment check is the one that refuses it.
        assert_eq!(refuser(&reading), Some("check_history_fixture"));
        add_behavior(&mut reading, HISTORY_FIXTURE_CRITERION);
        assert_eq!(refuser(&reading), Some("check_environment_reach"));
        // Quoted figures say nothing about where the code runs: an assumption missing the permission fails.
        let no_permission = with_environment(
            reading.clone(),
            "Environment: the CI job reaches the records branch",
        );
        assert_eq!(refuser(&no_permission), Some("check_environment_reach"));
        let figures = with_environment(reading.clone(), "Based on 3 jobs and 40% of readings");
        assert_eq!(refuser(&figures), Some("check_environment_reach"));
        let fixed = with_environment(
            reading,
            "Environment: the CI job has read-only access to contents and cannot write the records branch; \
             the producer snapshot has no records/",
        );
        assert_eq!(refuser(&fixed), None);
    }

    #[test]
    fn environment_reach_source_refuses_the_push_and_pull_request_shape_of_8ra0() {
        let reading = behavior_reading(&[
            "Athena opens the job's PR early at verification start",
            "Run `cargo test -p athena-core`: with no origin the push is skipped",
        ]);
        assert_eq!(refuser(&reading), Some("check_environment_reach"));
        let local_only = with_environment(
            reading.clone(),
            "Environment: the producer working copy has write access to the repo",
        );
        assert_eq!(refuser(&local_only), Some("check_environment_reach"));
        assert_eq!(refuser(&with_environment(reading, LOCAL_ENVIRONMENT)), None);
    }

    #[test]
    fn environment_reach_source_refuses_records_cited_in_the_summary_in_the_nm0k_shape() {
        let reading = summary_reading("Use the figures from job records and history");
        assert_eq!(refuser(&reading), Some("check_environment_reach"));
        let error = check_environment_reach(&reading).unwrap_err().to_string();
        assert!(
            error.starts_with("the reading was refused because"),
            "{error}"
        );
        assert!(error.contains("counted figures"), "{error}");
        // A records path, a code file and a year are not a reachable file or a figure.
        for env in [
            "Environment: records/foo.json holds them",
            "Environment: athena/records/x.json holds them",
            "Environment: crates/athena-core/src/reading.rs holds them",
            "Environment: they date from 2026",
            "Environment: the CI job reads 12345678 refs",
        ] {
            let reading = with_environment(reading.clone(), env);
            assert_eq!(refuser(&reading), Some("check_environment_reach"), "{env}");
        }
        // A counted figure in any assumption, or a reachable file in an Environment assumption, repairs it.
        for text in [
            "The figures cover 3 jobs",
            "About 40% of readings are affected",
            "Environment: docs/design/roadmap.md lists them",
            "Environment: see `docs/design/roadmap.md`.",
        ] {
            let fixed = with_environment(reading.clone(), text);
            assert_eq!(refuser(&fixed), None, "{text}");
        }
        // A file named outside an Environment assumption does not count.
        let outside = with_environment(reading.clone(), "See docs/design/roadmap.md");
        assert_eq!(refuser(&outside), Some("check_environment_reach"));
        assert_eq!(
            refuser(&summary_reading("Use the figures from history")),
            Some("check_environment_reach")
        );
        assert_eq!(
            refuser(&summary_reading("Fix the history of the page")),
            None
        );
    }

    #[test]
    fn environment_reach_source_covers_every_located_source_phrase() {
        for phrase in [
            "the records branch",
            "athena/records",
            "`athena/records`",
            "git history",
            "the git log",
            "commit history",
        ] {
            let reading = behavior_reading(&[&format!("The tool reads {phrase}")]);
            assert_eq!(
                refuser(&reading),
                Some("check_environment_reach"),
                "{phrase}"
            );
            let fixed = with_environment(reading, CI_ENVIRONMENT);
            assert_eq!(refuser(&fixed), None, "{phrase}");
        }
        for phrase in [
            "job records",
            "job history",
            "commit history",
            "git history",
            "the git log",
            "the records branch",
            "athena/records",
            "`athena/records`",
            "history",
        ] {
            let reading = summary_reading(&format!("Use the figures from {phrase}"));
            assert_eq!(
                refuser(&reading),
                Some("check_environment_reach"),
                "{phrase}"
            );
            let fixed = with_environment(reading, "Counts cover 3 jobs");
            assert_eq!(refuser(&fixed), None, "{phrase}");
        }
    }

    #[test]
    fn environment_reach_accepts_ordinary_wording_about_records() {
        // The wording of the status and scoring code: no place is named, so nothing is assigned to one.
        for text in [
            "Run `athena status`: rounds per landed job are computed from the job records",
            "Scores a job from its records",
            "The tool reads the records",
            "The audit reads records",
            "The record is written in the records branch",
            "A test shows a reading citing job records is refused",
        ] {
            let reading = behavior_reading(&[text]);
            check_environment_reach(&reading).unwrap_or_else(|e| panic!("{text}: {e}"));
        }
        let summary = summary_reading("The audit counts readings in records");
        check_environment_reach(&summary).unwrap();
    }

    #[test]
    fn environment_reach_accepts_docs_readings_and_words_inside_other_words() {
        let plain = governed_reading(&["docs/x.md"], &["docs/x.md exists"], &[]);
        check_environment_reach(&plain).unwrap();
        assert_eq!(refuser(&plain), None);
        // Content criteria, assumptions and out_of_scope are never read.
        let mut content = governed_reading(
            &["docs/guide.md"],
            &[
                "Athena pushes the job branch to origin",
                "The PR is opened by land and uses the GitHub API",
                "the history of the page",
                "gh/early.rs calls parse_remote",
            ],
            &["Opening a PR from CI"],
        );
        content.assumptions = vec!["The CI job writes the records branch via git push".into()];
        check_environment_reach(&content).unwrap();
        // Behavior criteria with the words only inside other words, paths and identifiers.
        for text in [
            "Fixed where it originates",
            "The ghost entry is removed",
            "The pushdown rule applies",
            "It runs remotely",
            "A Vec push appends the entry",
            "Entries are pushed onto the list",
            "Run `cargo test`: pr.rs parses the list",
            "gh/early.rs is not changed",
            "`parse_remote` returns None for a local path",
            "The pr is opened in lowercase prose",
            "An express route",
            "The CIty map renders",
            "CI checks each commit",
            "CI tests the commits",
            "The push/pull-request/gh wording is kept",
        ] {
            let reading = behavior_reading(&[text]);
            check_environment_reach(&reading).unwrap_or_else(|e| panic!("{text}: {e}"));
        }
    }

    #[test]
    fn environment_reach_accepts_the_criteria_and_summary_of_its_own_job() {
        let mut reading = summary_reading(
            "Root cause (jobs 20261024-220048-engm, 20261025-070538-8ra0, 20261025-034247-nm0k): the reading \
             stage never asks who performs an action or measurement, where, with what permissions and what \
             data it can reach, so plans assumed CI writes, a missing GitHub origin, or unreachable records. \
             Fix it at the reading gate with a new `check_environment_reach` step in refusing_check \
             (reading.rs) that refuses readings that need an external site or unreachable source but lack \
             an `Environment:` assumption.",
        );
        for text in [
            "Run `cargo test -p athena-core`: unit tests show a reading that adds a CI workflow deliverable, \
             or a push/pull-request/gh action, without an `Environment:` assumption naming site, permissions \
             and reachable data is refused by `check_environment_reach`, and the same reading with such an \
             assumption is accepted. These tests fail when the new step is removed.",
            "Run `cargo test -p athena-core`: a test shows a reading citing job records, history or the \
             records branch as a source is refused unless the figures are quoted inline or an \
             `Environment:` assumption lists a reachable file, and that it is accepted when they are. Tests \
             cover the class (separate readings shaped like each of the three jobs), not one job.",
            "Run `cargo test -p athena-core --test refusal_corpus`: the corpus has a refuse case and an \
             accept case for `check_environment_reach`, and the test that every `step!` has a refusing \
             corpus case passes.",
            "Docs-only readings and readings without these triggers are still accepted: a test with a plain \
             docs reading passes `check_environment_reach`; the change refuses more readings and the higher \
             refusal rate is intended, expected 8% of newly refused readings.",
            "Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets` and `cargo test \
             --workspace`: all pass.",
        ] {
            add_behavior(&mut reading, text);
        }
        check_environment_reach(&reading).unwrap();
        let declared = replay_declaration_of(&reading);
        assert_eq!(declared.share, Some(8));
    }

    #[test]
    fn environment_reach_has_one_step_line() {
        let source = reading_source();
        let body = source
            .split("pub fn refusing_check(")
            .nth(1)
            .and_then(|rest| rest.split("\n}\n").next())
            .unwrap();
        let steps: Vec<&str> = body
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with("step!(check_"))
            .collect();
        let line = "step!(check_environment_reach(reading));";
        assert_eq!(steps.iter().filter(|l| **l == line).count(), 1, "{steps:?}");
        let outcome = steps
            .iter()
            .position(|l| *l == "step!(check_outcome_state(reading, request));")
            .unwrap();
        assert_eq!(steps[outcome + 1], line);
        let acceptance_doc = source
            .split("pub fn check_reading_acceptance(")
            .next()
            .unwrap();
        assert!(acceptance_doc.contains("[`check_environment_reach`]"));
    }

    #[test]
    fn environment_reach_doc_and_changelog_name_the_cause() {
        fn collapse(text: &str) -> String {
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        }
        const CAUSE: &str = "no step asking who performs each action, where, with what permissions, and what \
                             data it reaches";
        let source = reading_source();
        let before = source
            .split("pub fn check_environment_reach(")
            .next()
            .unwrap();
        let mut doc_lines: Vec<&str> = before
            .lines()
            .rev()
            .take_while(|l| l.trim_start().starts_with("///"))
            .map(|l| l.trim_start().trim_start_matches('/'))
            .collect();
        doc_lines.reverse();
        let doc = collapse(&doc_lines.join(" "));
        let changelog = include_str!("../../../CHANGELOG.md");
        let unreleased = changelog
            .split("## [Unreleased]")
            .nth(1)
            .unwrap()
            .split("\n## [")
            .next()
            .unwrap();
        let mut items: Vec<String> = Vec::new();
        for line in unreleased.lines() {
            if line.starts_with("- ") {
                items.push(line.to_string());
            } else if let Some(last) = items.last_mut() {
                last.push(' ');
                last.push_str(line);
            }
        }
        let items: Vec<String> = items.iter().map(|i| collapse(i)).collect();
        let item = items
            .iter()
            .find(|i| i.contains(CAUSE))
            .expect("a changelog item names the cause");
        for (label, text) in [("doc comment", &doc), ("changelog item", item)] {
            for needle in [
                CAUSE,
                "20261024-220048-engm",
                "20261025-070538-8ra0",
                "20261025-034247-nm0k",
                "higher refusal rate is intended",
                "expected 8% of newly refused readings",
                "estimate",
                "not measured",
                "`Environment:`",
            ] {
                assert!(text.contains(needle), "the {label} lacks {needle:?}");
            }
        }
    }

    /// A complete cache statement: save scope, restore scope, write rights and the ref scope.
    const CACHE_SCOPE: &str = "Environment: the CI cache is saved by the main branch push run, restored by \
         pull request runs of the same branch and by main, written only by main";

    /// Cache statements that each lack one part, with the phrase the error must use for the missing part.
    const CACHE_SCOPE_GAPS: [(&str, &str); 5] = [
        (
            "Environment: the CI cache is restored by pull request runs of the same branch and by main, \
             written only by main",
            "which ref and event saves it",
        ),
        (
            "Environment: the CI cache is saved by the main branch push run, written only by main",
            "which runs restore it",
        ),
        (
            "Environment: the CI cache is saved by the main branch push run and restored by pull request \
             runs of the same branch and by main",
            "which runs may write shared entries",
        ),
        (
            "Environment: the CI cache is saved by default, restored by default and written by default",
            "the ref scope",
        ),
        (
            "Environment: it is saved by the main branch push run, restored by pull request runs of the \
             same branch and by main, written only by main",
            "no `Environment:` assumption names",
        ),
    ];

    /// The phrases the error uses for each part a cache statement can lack.
    const CACHE_SCOPE_PARTS: [&str; 4] = [
        "which ref and event saves it",
        "which runs restore it",
        "which runs may write shared entries",
        "the ref scope",
    ];

    /// Asserts that every gap in [`CACHE_SCOPE_GAPS`] is refused by `check_environment_reach` with an error that
    /// names the missing part, and that the complete statement is accepted. `base` carries the assumptions every
    /// variant needs for the place statement.
    fn assert_cache_scope_is_required(reading: &Reading, base: &[&str], label: &str) {
        let with_base = |extra: &str| {
            let mut r = reading.clone();
            for text in base {
                r.assumptions.push((*text).into());
            }
            r.assumptions.push(extra.into());
            r
        };
        let bare = {
            let mut r = reading.clone();
            for text in base {
                r.assumptions.push((*text).into());
            }
            r
        };
        let error = check_environment_reach(&bare).unwrap_err().to_string();
        assert!(
            error.contains("no `Environment:` assumption names"),
            "{label}: {error}"
        );
        assert!(!error.contains("who performs it"), "{label}: {error}");
        assert_eq!(refuser(&bare), Some("check_environment_reach"), "{label}");
        for (text, phrase) in CACHE_SCOPE_GAPS {
            let gap = with_base(text);
            let error = check_environment_reach(&gap).unwrap_err().to_string();
            assert!(error.contains(phrase), "{label}: {text}: {error}");
            if phrase != "no `Environment:` assumption names" {
                // Only the missing part is named in the list after "lacks".
                let lacks = error.split(" lacks ").nth(1).unwrap_or("");
                let lacks = lacks.split(". Repair").next().unwrap();
                assert_eq!(lacks, phrase, "{label}: {text}: {error}");
                for other in CACHE_SCOPE_PARTS.iter().filter(|p| **p != phrase) {
                    assert!(!lacks.contains(other), "{label}: {text}: {error}");
                }
            }
            assert_eq!(
                refuser(&gap),
                Some("check_environment_reach"),
                "{label}: {text}"
            );
        }
        let fixed = with_base(CACHE_SCOPE);
        check_environment_reach(&fixed).unwrap_or_else(|e| panic!("{label}: {e}"));
        assert_eq!(refuser(&fixed), None, "{label}");
    }

    #[test]
    fn environment_reach_cache_refuses_a_workflow_with_a_cache_without_scope() {
        for action in [DeliverableAction::Create, DeliverableAction::Modify] {
            let mut reading = governed_reading(
                &["docs/guide.md", ".github/workflows/ci.yml"],
                &[
                    "docs/guide.md exists",
                    ".github/workflows/ci.yml runs the build",
                ],
                &[],
            );
            reading.deliverables[1].action = action;
            // The workflow alone, with the place statement, is accepted: no cache is mentioned.
            let plain = with_environment(reading.clone(), CI_ENVIRONMENT);
            check_environment_reach(&plain).unwrap();
            assert_eq!(refuser(&plain), None);
            // A cache word in a behavior criterion makes the cache scope a requirement (a path is no trigger).
            add_behavior(
                &mut reading,
                "ci.yml saves a result cache keyed by the build fingerprint",
            );
            assert_cache_scope_is_required(&reading, &[CI_ENVIRONMENT], "workflow deliverable");
            // The same for a cache named only in the summary.
            let mut summary = governed_reading(
                &["docs/guide.md", ".github/workflows/ci.yml"],
                &[
                    "docs/guide.md exists",
                    ".github/workflows/ci.yml runs the build",
                ],
                &[],
            );
            summary.deliverables[1].action = action;
            summary.summary = "Save an artifact between runs".into();
            assert_cache_scope_is_required(&summary, &[CI_ENVIRONMENT], "summary");
        }
        // Deleting a workflow adds no cache, so a cache word beside it triggers nothing.
        let mut deleted = workflow_reading(DeliverableAction::Delete);
        add_behavior(
            &mut deleted,
            "ci.yml saves a result cache keyed by the build fingerprint",
        );
        check_environment_reach(&deleted).unwrap();
    }

    #[test]
    fn environment_reach_cache_refuses_a_behavior_cache_criterion_without_scope() {
        for text in [
            "The CI job restores a result cache keyed by the build fingerprint",
            "CI uploads the build artifact and later runs download it",
            "The build uses `actions/cache@v4` keyed by the lock file",
            "The job calls `actions/upload-artifact` for the report",
            "The GitHub runner restores a cache",
        ] {
            let reading = behavior_reading(&[text]);
            assert_cache_scope_is_required(&reading, &[], text);
        }
        // A CI block beside a cache sentence without write rights: the CI block's write word is not borrowed.
        let reading =
            behavior_reading(&["The CI job restores a result cache keyed by the fingerprint"]);
        let mut mixed = reading.clone();
        mixed.assumptions.push(CI_ENVIRONMENT.into());
        mixed.assumptions.push(
            "Environment: the CI cache is saved by the main branch push run and restored by pull request \
             runs of the same branch"
                .into(),
        );
        let error = check_environment_reach(&mixed).unwrap_err().to_string();
        assert!(
            error.contains("which runs may write shared entries"),
            "{error}"
        );
        // The cache-only refusal names the cache questions in its tail, not the place questions.
        assert!(
            error.contains("which ref saves the cache or artifact"),
            "{error}"
        );
        assert!(!error.contains("who performs it"), "{error}");
        assert!(
            error.starts_with("the reading was refused because"),
            "{error}"
        );
        // The best assumption is judged: a second, complete one repairs it.
        mixed.assumptions.push(CACHE_SCOPE.into());
        check_environment_reach(&mixed).unwrap();
        // A host action and a cache together need both statements; the error carries both tails.
        let both = behavior_reading(&[
            "Athena pushes the job branch to origin",
            "The CI job restores a result cache keyed by the fingerprint",
        ]);
        let error = check_environment_reach(&both).unwrap_err().to_string();
        assert!(error.contains("who performs it"), "{error}");
        assert!(
            error.contains("which ref saves the cache or artifact"),
            "{error}"
        );
        let fixed = with_environment(with_environment(both, CI_ENVIRONMENT), CACHE_SCOPE);
        check_environment_reach(&fixed).unwrap();
    }

    #[test]
    fn environment_reach_cache_scope_covers_the_class() {
        // Job 20261025-034731-2am0, rebuilt from the job brief: a result cache keyed by the fingerprint.
        let fingerprint = behavior_reading(&[
            "CI restores a result cache keyed by the fingerprint of the build paths",
        ]);
        assert_cache_scope_is_required(&fingerprint, &[CI_ENVIRONMENT], "2am0");
        // Job 20261025-055626-er96, rebuilt from the job brief: no cache word and no workflow deliverable.
        let early_push = behavior_reading(&["An early push whose CI run is reused at landing"]);
        assert!(
            check_environment_reach(&early_push)
                .unwrap_err()
                .to_string()
                .contains("a CI result reused from another ref")
        );
        assert_eq!(refuser(&early_push), Some("check_environment_reach"));
        let only_ci = with_environment(early_push.clone(), CI_ENVIRONMENT);
        assert_eq!(refuser(&only_ci), Some("check_environment_reach"));
        let fixed = with_environment(
            only_ci,
            "Environment: the CI result is saved by the early push run on the job branch, restored by the \
             landing run of the same branch and by main, written only by main",
        );
        check_environment_reach(&fixed).unwrap();
        assert_eq!(refuser(&fixed), None);
        // No cache, no artifact, no workflow and no CI result reuse: accepted unchanged.
        for text in [
            "Fixed where it originates",
            "The reader caches the survey in memory",
            "cache.rs is not changed",
            "`cache_dir` is kept",
            "The GitHub API response is cached in memory",
            "The reader caches its actions",
            "CI runs the tests",
            "The corpus runner caches the readings",
            "The workflow engine caches the plan",
            "ci restores the cache",
            "A green CI result is reused as the job baseline when the fingerprint matches",
            "A green CI run for the candidate SHA is reused",
            "CI reuses the run matched by the commit",
        ] {
            let reading = behavior_reading(&[text]);
            check_environment_reach(&reading).unwrap_or_else(|e| panic!("{text}: {e}"));
            assert_eq!(refuser(&reading), None, "{text}");
        }
        let plain = behavior_reading(&[SERVED_STATE]);
        check_environment_reach(&plain).unwrap();
        // A content criterion with a cache word, beside a workflow deliverable, triggers nothing.
        let mut content = workflow_reading(DeliverableAction::Create);
        add_content(&mut content, "The cache design is documented in the guide");
        check_environment_reach(&with_environment(content, CI_ENVIRONMENT)).unwrap();
        // A workflow deliverable makes a bare workflow or runner a CI place.
        let mut runner = workflow_reading(DeliverableAction::Create);
        add_behavior(&mut runner, "The runner restores the cache");
        assert_cache_scope_is_required(&runner, &[CI_ENVIRONMENT], "runner beside a workflow");
    }

    #[test]
    fn environment_reach_cache_scope_accepts_the_criteria_and_summary_of_its_own_job() {
        let mut reading = summary_reading(
            "Root cause: check_environment_reach makes the reader state site, permissions and data but never \
             asks which ref saves each CI cache or artifact, which runs restore it, and which may write shared \
             entries. Extend that check so a CI workflow deliverable or a cache or artifact behavior criterion \
             needs an Environment: assumption stating save scope, restore scope and write rights.",
        );
        for text in [
            "Run cargo test -p athena-core: a new unit test shows a reading with a created \
             .github/workflows/ci.yml deliverable or a behavior criterion that keys a CI cache or artifact is \
             refused when its Environment: assumption lacks cache save scope, restore scope or write rights, \
             and is accepted when the assumption states them (for example saved by the main branch push run, \
             restored by pull request runs of the same branch and by main, written only by main).",
            "Run cargo test -p athena-core: a test covers the class, not one job. It uses wording from both \
             referenced jobs (a fingerprint-keyed result cache; an early push whose CI run is reused at \
             landing), plus a reading with no cache or artifact and no workflow, which stays accepted.",
            "Run cargo test -p athena-core: the corpus includes the new check in the acceptance-check name \
             list and the replay declaration test passes: the change refuses more readings, expected 3% of newly \
             refused readings.",
        ] {
            add_behavior(&mut reading, text);
        }
        // Without the assumption the job's own text is refused; this is the rule it asks for.
        assert!(check_environment_reach(&reading).is_err());
        reading.assumptions.push(
            "Environment: the check runs locally in the reader stage over the in-memory Reading with \
             read-only access to the request and snapshot; it reaches no refs, origin or CI. The rule it \
             enforces concerns CI (GitHub Actions) caches: a cache is saved by the ref and event that ran \
             it, restored by runs of the same ref or its base, and shared entries are written only by the \
             default branch."
                .into(),
        );
        check_environment_reach(&reading).unwrap();
        let declared = replay_declaration_of(&reading);
        assert_eq!(declared.share, Some(3));
    }

    #[test]
    fn environment_reach_cache_scope_doc_and_changelog_name_the_cause() {
        fn collapse(text: &str) -> String {
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        }
        const CAUSE: &str = "no step asking, for each CI cache or stored artifact, which ref saves it, which \
                             runs restore it, and which runs may write shared entries";
        const OLD_CAUSE: &str = "no step asking who performs each action, where, with what permissions, and \
                                 what data it reaches";
        let before = reading_source()
            .split("pub fn check_environment_reach(")
            .next()
            .unwrap();
        let mut doc_lines: Vec<&str> = before
            .lines()
            .rev()
            .take_while(|l| l.trim_start().starts_with("///"))
            .map(|l| l.trim_start().trim_start_matches('/'))
            .collect();
        doc_lines.reverse();
        let doc = collapse(&doc_lines.join(" "));
        let unreleased = include_str!("../../../CHANGELOG.md")
            .split("## [Unreleased]")
            .nth(1)
            .unwrap()
            .split("\n## [")
            .next()
            .unwrap();
        let mut items: Vec<String> = Vec::new();
        for line in unreleased.lines() {
            if line.starts_with("- ") {
                items.push(line.to_string());
            } else if let Some(last) = items.last_mut() {
                last.push(' ');
                last.push_str(line);
            }
        }
        let items: Vec<String> = items.iter().map(|i| collapse(i)).collect();
        let item = items
            .iter()
            .find(|i| i.contains(CAUSE))
            .expect("a changelog item names the cache-scope cause");
        assert!(
            !item.contains(OLD_CAUSE),
            "the new item is separate from the first one"
        );
        for (label, text) in [("doc comment", &doc), ("changelog item", item)] {
            for needle in [
                CAUSE,
                "20261025-034731-2am0",
                "20261025-055626-er96",
                "20261025-050603-wrv7",
                "20261025-065849-n99a",
                "This check refuses more readings; the higher refusal rate is intended, expected 3% of newly \
                 refused readings (an estimate, not measured)",
                "`Environment:`",
                "`actions/cache`",
                "SHA",
            ] {
                let needle = collapse(needle);
                assert!(text.contains(&needle), "the {label} lacks {needle:?}");
            }
        }
        for needle in [
            "save scope",
            "restore scope",
            "write rights",
            "`check_environment_reach`",
            "briefs.rs",
        ] {
            assert!(item.contains(needle), "the changelog item lacks {needle:?}");
        }
        assert!(doc.contains("lowercase `ci`"), "{doc}");
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
    fn a_boundary_word_in_prose_fields_is_accepted() {
        let word = banned_word();
        let mut reading = governed_reading(&["README.md"], &["the greeting prints"], &["x"]);
        reading.summary = format!("Ask the {word} to approve it.");
        reading.assumptions = vec![format!("The {word} reads the logo.")];
        check_reading_acceptance(&reading, "add a greeting", None, &[], None).unwrap();
        // The same with a file set, and on the replay path: the words do not depend on the repository.
        let files = files_at_head();
        check_reading_acceptance(&reading, "add a greeting", None, &[], Some(&files)).unwrap();
        assert!(matches!(
            check_reading_text(&reading, "add a greeting"),
            ReadingCheck::Accept
        ));
    }

    #[test]
    fn a_boundary_word_in_a_path_or_content_criterion_is_still_refused() {
        let word = banned_word();
        let fragment = banned_fragment();
        for term in [&word, &fragment] {
            // A deliverable path is scanned, and the error names the path and the word.
            let path = format!("docs/{term}.md");
            let reading = reading_with_deliverable(&path, DeliverableAction::Create);
            let error = check_reading_acceptance(&reading, "add a greeting", None, &[], None)
                .unwrap_err()
                .to_string();
            assert!(
                error.contains(&path) && error.contains(term.as_str()),
                "{error}"
            );
            assert!(
                error.contains("deliverable path or content acceptance criterion"),
                "{error}"
            );
            // A content criterion is text to write into a file: scanned, and named by its label.
            let mut reading = governed_reading(&["README.md"], &["the greeting prints"], &["x"]);
            reading.acceptance_criteria[0].text = format!("README.md says: the {term} decides.");
            assert_eq!(reading.acceptance_criteria[0].kind, CriterionKind::Content);
            let error = check_reading_acceptance(&reading, "add a greeting", None, &[], None)
                .unwrap_err()
                .to_string();
            assert!(error.contains("acceptance_criteria[0]"), "{error}");
            assert!(error.contains(term.as_str()), "{error}");
            assert!(
                error.contains("deliverable path or content acceptance criterion"),
                "{error}"
            );
            // The replay path refuses it too.
            assert!(matches!(
                check_reading_text(&reading, "add a greeting"),
                ReadingCheck::Refuse(_)
            ));
        }
    }

    #[test]
    fn boundary_words_outside_paths_and_content_criteria_are_not_scanned() {
        let word = banned_word();
        let fragment = banned_fragment();
        type Edit = fn(&mut Reading, String);
        let edits: [(&str, Edit); 8] = [
            ("title", |r, t| r.title = t),
            ("out_of_scope[0]", |r, t| r.out_of_scope = vec![t]),
            ("defaults_applied[0]", |r, t| {
                r.defaults_applied = vec![DefaultApplied {
                    id: "tested".into(),
                    how: t,
                }]
            }),
            ("departures", |r, t| r.departures = Some(t)),
            ("hard_reason", |r, t| r.hard_reason = Some(t)),
            ("urgent_reason", |r, t| r.urgent_reason = Some(t)),
            ("behavior criterion", |r, t| {
                r.acceptance_criteria[0].text = t;
                r.acceptance_criteria[0].kind = CriterionKind::Behavior;
            }),
            ("assumptions[0]", |r, t| r.assumptions = vec![t]),
        ];
        for (field, edit) in edits {
            for text in [format!("The {word} decides."), format!("See {fragment}/x.")] {
                let mut reading =
                    governed_reading(&["README.md"], &["the greeting prints"], &["x"]);
                edit(&mut reading, text);
                if let Err(error) =
                    check_reading_acceptance(&reading, "add a greeting", None, &[], None)
                {
                    panic!("{field} was refused: {error}");
                }
            }
        }
        // A question and a decline name no change, so each is its own reading.
        let mut asking = governed_reading(&["README.md"], &["the greeting prints"], &["x"]);
        asking.question = Some(format!("Which {word} do you mean?"));
        check_reading_acceptance(&asking, "add a greeting", None, &[], None).unwrap();
        let mut declined = governed_reading(&["README.md"], &["the greeting prints"], &["x"]);
        declined.capability = None;
        declined.decline_reason = Some(format!("The {word} owns it."));
        check_reading_acceptance(&declined, "add a greeting", None, &[], None).unwrap();
        // Words that only contain a banned word are not hits in a scanned field either.
        let mut clean = governed_reading(&["README.md"], &["the greeting prints"], &["x"]);
        clean.acceptance_criteria[0].text = "README.md says: cooperators accrue the opcode.".into();
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
        reading.acceptance_criteria[0].text = format!("README.md says: the {word} decides.");
        let error = check_reading_acceptance(&reading, "add a greeting", None, &[], Some(&files))
            .unwrap_err()
            .to_string();
        assert!(error.contains(&word), "{error}");
        assert!(error.contains("docs/missing.md"), "{error}");
    }

    // -- The repo paths a reading relies on

    const CORE_FILE: &str = "crates/athena-core/src/reading.rs";
    const ABSENT_FILE: &str = "crates/athena-core/src/design_review.rs";

    /// A snapshot that holds the core reading file and a few others, but not `ABSENT_FILE`.
    fn files_with_core() -> BTreeSet<String> {
        [
            CORE_FILE,
            "crates/athena-core/src/lib.rs",
            "docs/decisions.md",
        ]
        .iter()
        .map(|p| p.to_string())
        .collect()
    }

    /// A clean change reading (a modify of the core file) whose assumption holds `text`.
    fn reading_assuming(text: &str) -> Reading {
        let mut reading = governed_reading(&[CORE_FILE], &["the greeting prints"], &["the logo"]);
        reading.assumptions = vec![text.into()];
        reading
    }

    fn check_refs(reading: &Reading, files: Option<&BTreeSet<String>>) -> Result<()> {
        check_reading_acceptance(reading, "add a greeting", None, &[], files)
    }

    #[test]
    fn a_reading_that_relies_on_an_absent_path_is_refused_and_the_path_is_named() {
        let files = files_with_core();
        let claim = format!("The role in {ABSENT_FILE} already exists and is reused.");
        // In an assumption, the summary and a criterion: each is refused with the path and field named.
        type Edit = fn(&mut Reading, String);
        let edits: [(&str, Edit); 3] = [
            ("assumptions[0]", |r, t| r.assumptions = vec![t]),
            ("summary", |r, t| r.summary = t),
            ("acceptance_criteria[0]", |r, t| {
                r.acceptance_criteria[0].text = t
            }),
        ];
        for (field, edit) in edits {
            let mut reading = reading_assuming("none");
            edit(&mut reading, claim.clone());
            let error = check_refs(&reading, Some(&files)).unwrap_err().to_string();
            assert!(error.contains(ABSENT_FILE), "{field}: {error}");
            assert!(error.contains(field), "{field}: {error}");
        }
        // The same reading passes once the path is in the snapshot.
        let mut with_it = files.clone();
        with_it.insert(ABSENT_FILE.into());
        check_refs(&reading_assuming(&claim), Some(&with_it)).unwrap();
    }

    #[test]
    fn the_reference_rule_accepts_a_present_created_or_declared_absent_path_and_no_file_set() {
        let files = files_with_core();
        let claim = format!("The role in {ABSENT_FILE} already exists and is reused.");
        // (a) a file in the snapshot, a directory above one, with trailing punctuation, backticks, parentheses
        // and a line suffix
        check_refs(
            &reading_assuming(&format!(
                "See `{CORE_FILE}`, ({CORE_FILE}:2849), crates/athena-core/src/ and docs/decisions.md."
            )),
            Some(&files),
        )
        .unwrap();
        // (b) a create deliverable, and a directory above one
        let mut created = reading_assuming(&claim);
        created.deliverables.push(Deliverable {
            path: ABSENT_FILE.into(),
            action: DeliverableAction::Create,
            p: 0.9,
        });
        check_refs(&created, Some(&files)).unwrap();
        let mut dir_created = reading_assuming("Files go under crates/athena-core/src/new_dir.");
        dir_created.deliverables.push(Deliverable {
            path: "crates/athena-core/src/new_dir/a.rs".into(),
            action: DeliverableAction::Create,
            p: 0.9,
        });
        check_refs(&dir_created, Some(&files)).unwrap();
        // (c) a sentence that says it does not exist; a second absent path in another sentence is refused
        let says_absent = format!("{ABSENT_FILE} does not exist yet.");
        check_refs(&reading_assuming(&says_absent), Some(&files)).unwrap();
        let both = format!("{says_absent} Also xtask/src/other.rs is reused.");
        let error = check_refs(&reading_assuming(&both), Some(&files))
            .unwrap_err()
            .to_string();
        assert!(error.contains("xtask/src/other.rs"), "{error}");
        assert!(!error.contains(ABSENT_FILE), "{error}");
        // (d) no file set: a replay judges nothing
        check_refs(&reading_assuming(&claim), None).unwrap();
        // (f) glob, placeholder, URL, absolute, relative, ellipsis and slash-less forms are not paths
        for text in [
            "Every file in crates/*/src is read.",
            "A file docs/<name>.md is written.",
            "See https://example.com/docs/guide.md for more.",
            "The file /etc/docs/x is not ours.",
            "Run ./docs/run.sh now.",
            "Paths like crates/athena-core/... are meant.",
            "The file design_review.rs already exists.",
            "Use the and/or form.",
            "CHANGELOG.md gets an entry, and so does docs/decisions.md.",
        ] {
            check_refs(&reading_assuming(text), Some(&files))
                .unwrap_or_else(|e| panic!("{text}: {e}"));
        }
        // (g) a reading that asks or declines names no change
        let mut asking = reading_assuming(&claim);
        asking.question = Some("which file?".into());
        check_refs(&asking, Some(&files)).unwrap();
        let mut declined = reading_assuming(&claim);
        declined.capability = None;
        declined.decline_reason = Some("no capability fits".into());
        check_refs(&declined, Some(&files)).unwrap();
    }

    #[test]
    fn the_reference_rule_refuses_any_absent_path_not_one_job_or_file() {
        let files = files_with_core();
        for path in [
            "docs/design/other.md",
            "xtask/src/new.rs",
            ".github/workflows/x.yml",
            "studies/a/b.md",
            "crates/other-crate/src/lib.rs",
            "crates/athena-core/src",
            "crates/athena-core/sr",
            "prompts/reader.md",
        ] {
            let text = format!("Job 20261026-000000-abcd added {path}, so it already exists.");
            let result = check_refs(&reading_assuming(&text), Some(&files));
            if path == "crates/athena-core/src" {
                // A directory that holds files is present.
                result.unwrap();
            } else if path == "prompts/reader.md" {
                // Not a top-level directory of the snapshot, so it is not judged.
                result.unwrap();
            } else {
                let error = result.unwrap_err().to_string();
                assert!(error.contains(path), "{path}: {error}");
            }
        }
    }

    #[test]
    fn the_reference_refusal_tells_the_reader_how_to_repair_it() {
        let files = files_with_core();
        let reading = reading_assuming(&format!("{ABSENT_FILE} already exists."));
        let error = check_refs(&reading, Some(&files)).unwrap_err().to_string();
        assert!(error.contains(ABSENT_FILE), "{error}");
        assert!(error.contains("create deliverable"), "{error}");
        assert!(error.contains("does not exist"), "{error}");
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

    #[test]
    fn replay_limit_is_declared_share_plus_five_or_ten_and_exactly_the_limit_passes() {
        assert_eq!(replay_limit_percent(None), REPLAY_HOLD_PERCENT);
        assert_eq!(replay_limit_percent(None), 10);
        assert_eq!(replay_limit_percent(Some(20)), 25);
        assert_eq!(replay_limit_percent(Some(0)), 5);
        // Exactly the limit is not held; one over is. The producer's loop sends back on `replay_is_held`.
        assert!(!replay_is_held(50, 5, None));
        assert!(replay_is_held(50, 6, None));
        assert!(!replay_is_held(20, 5, Some(20)));
        assert!(replay_is_held(20, 6, Some(20)));
        // The same boundary through a measured replay over closure checkers.
        for (marked, held) in [(5, false), (6, true)] {
            let run = replay(
                &marked_cases(50, marked),
                &base_checker(),
                &marker_candidate(),
                None,
            );
            assert_eq!(run.held, held, "{marked} of 50");
            assert_eq!(run.newly_refused.len(), marked);
        }
    }

    #[test]
    fn touches_reading_rules_is_true_only_for_code_and_cargo_files() {
        let paths = |list: &[&str]| -> Vec<String> { list.iter().map(|p| p.to_string()).collect() };
        assert!(!touches_reading_rules(&[]));
        assert!(!touches_reading_rules(&paths(&["README.md"])));
        assert!(!touches_reading_rules(&paths(&[
            "docs/x.md",
            "CHANGELOG.md"
        ])));
        assert!(!touches_reading_rules(&paths(&["docs/crates/x.md"])));
        assert!(touches_reading_rules(&paths(&[
            "crates/athena-core/src/reading.rs"
        ])));
        assert!(touches_reading_rules(&paths(&["README.md", "Cargo.lock"])));
        assert!(touches_reading_rules(&paths(&["Cargo.toml"])));
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
        let path = dir.join("athena-stub");
        athena_testkit::write_executable(&path, format!("#!/bin/sh\n{body}\n"));
        path
    }

    /// Runs a checker over a stub written by [`stub_binary`].
    #[cfg(unix)]
    fn check_stub(bin: PathBuf, reading: &Reading) -> Result<ReadingCheck> {
        BinaryChecker::new(bin).check("add a greeting", reading)
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

    /// The complete test-design survey for this reading: entry point, gate order, stop gate, reuse, output and
    /// one direct check for each non-routine behavior criterion, numbered by position. It names no crates/
    /// path, so it never stands in for the survey `check_code_survey` reads.
    fn complete_survey_for(reading: &Reading) -> String {
        let record = reading
            .deliverables
            .iter()
            .any(is_record_or_data_deliverable);
        let mut survey = format!(
            "Code survey: Entry point: `refusing_check`. Gate order: `check_criteria_evidence`, \
             `check_code_survey`, `check_history_fixture`. Stop gate: `check_code_survey`. \
             Reuses: `names_record_evidence`; no existing function checks gate order. Output: {}.",
            if record {
                "`AnyRecord::FailureClass`"
            } else {
                "no new output"
            }
        );
        for (i, c) in reading.acceptance_criteria.iter().enumerate() {
            if c.kind == CriterionKind::Behavior && !is_routine_criterion(&c.text) {
                survey.push_str(&format!(
                    " Direct check: criterion {} counts the calls `refusing_check` makes to \
                     `check_code_survey` and asserts the count is 1.",
                    i + 1
                ));
            }
        }
        survey
    }

    /// The reading with its complete test-design survey added as a further `Code survey:` assumption, when it
    /// has a survey at all. Whole-gate fixtures go through this, so they are judged on the rules they test.
    fn with_design(reading: &Reading) -> Reading {
        let mut reading = reading.clone();
        if !code_surveys(&reading).is_empty() {
            let survey = complete_survey_for(&reading);
            reading.assumptions.push(survey);
        }
        reading
    }

    fn accept(reading: &Reading) -> Result<()> {
        check_reading_acceptance(
            &with_design(reading),
            "change the reading rules",
            None,
            &[],
            None,
        )
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
        let source = reading_source();
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

    // -- The test design of a code survey (`check_behavior_test_design`)

    mod design_tests {
        use super::*;

        const D_ENTRY: &str = "Entry point: `refusing_check`.";
        const D_GATES: &str = "Gate order: `check_criteria_evidence`, `check_code_survey`.";
        const D_STOP: &str = "Stop gate: `check_code_survey`.";
        const D_REUSE: &str = "Reuses: `names_record_evidence`.";
        const D_OUTPUT: &str = "Output: no new output.";
        const D_DIRECT: &str = "Direct check: criterion 1 counts the calls `refusing_check` makes to \
         `check_code_survey` and asserts the count is 1.";

        fn design_survey(parts: &[&str]) -> String {
            format!(
                "Code survey: crates/athena-core/src/reading/surveys.rs. {}",
                parts.join(" ")
            )
        }

        fn complete_parts() -> [&'static str; 6] {
            [D_ENTRY, D_GATES, D_STOP, D_REUSE, D_OUTPUT, D_DIRECT]
        }

        fn design_reading(survey: &str) -> Reading {
            code_reading(&[survey], &[MODIFY_READING])
        }

        fn design_error(survey: &str) -> Option<String> {
            check_behavior_test_design(&design_reading(survey))
                .err()
                .map(|e| e.to_string())
        }

        /// The complete survey with its direct check replaced.
        fn with_direct(text: &str) -> String {
            let direct = format!("Direct check: criterion 1 {text}");
            let parts: Vec<&str> = complete_parts()[..5]
                .iter()
                .copied()
                .chain(std::iter::once(direct.as_str()))
                .collect();
            design_survey(&parts)
        }

        fn assert_design_ok(survey: &str) {
            if let Some(error) = design_error(survey) {
                panic!("{survey}\n{error}");
            }
        }

        fn assert_design_refused(survey: &str, needles: &[&str]) {
            let error = design_error(survey).unwrap_or_else(|| panic!("accepted: {survey}"));
            assert!(error.contains("check_behavior_test_design"), "{error}");
            for needle in needles {
                assert!(error.contains(needle), "{needle}: {error}");
            }
        }

        #[test]
        fn design_check_has_one_step_line_after_the_survey_step() {
            let body = reading_source()
                .split("pub fn refusing_check(")
                .nth(1)
                .and_then(|rest| rest.split("\n}\n").next())
                .unwrap();
            let lines: Vec<&str> = body.lines().map(str::trim).collect();
            let at = lines
                .iter()
                .position(|l| *l == "step!(check_code_survey(reading));")
                .unwrap();
            assert_eq!(lines[at + 1], "step!(check_behavior_test_design(reading));");
            assert_eq!(
                lines
                    .iter()
                    .filter(|l| l.contains("check_behavior_test_design"))
                    .count(),
                1
            );
        }

        #[test]
        fn design_check_does_not_apply_outside_its_gate() {
            let none: &[&str] = &[];
            let mut docs = code_reading(none, &[MODIFY_READING]);
            docs.class_of_work = "docs".into();
            let mut declined = code_reading(none, &[MODIFY_READING]);
            declined.capability = None;
            declined.decline_reason = Some("no capability fits".into());
            let mut asking = code_reading(none, &[MODIFY_READING]);
            asking.question = Some("which file?".into());
            let mut content = code_reading(none, &[MODIFY_READING]);
            content.acceptance_criteria = vec![Criterion {
                text: "the doc comment names the cause".into(),
                kind: CriterionKind::Content,
            }];
            for reading in [&docs, &declined, &asking, &content] {
                check_behavior_test_design(reading).unwrap();
            }
        }

        #[test]
        fn design_check_refuses_each_omitted_part() {
            let [entry, gates, stop, reuse, output, direct] = complete_parts();
            assert_design_ok(&design_survey(&complete_parts()));
            let cases: [(&str, Vec<&str>); 4] = [
                ("(a)", vec![entry, reuse, output, direct]),
                ("(b)", vec![entry, gates, stop, output, direct]),
                ("(c)", vec![entry, gates, stop, reuse, direct]),
                ("(d)", vec![entry, gates, stop, reuse, output]),
            ];
            for (missing, parts) in cases {
                let survey = design_survey(&parts);
                let error = design_error(&survey).unwrap_or_else(|| panic!("accepted: {survey}"));
                for letter in ["(a)", "(b)", "(c)", "(d)"] {
                    assert_eq!(
                        error.contains(letter),
                        letter == missing,
                        "{letter}: {error}"
                    );
                }
                // The whole gate names the check, and so does the stored-reading seam.
                let reading = design_reading(&survey);
                assert_eq!(
                    gate_refuser_raw(&reading),
                    Some("check_behavior_test_design")
                );
                match check_reading_text(&reading, "change the reading rules") {
                    ReadingCheck::Refuse(reason) => {
                        assert!(reason.contains("check_behavior_test_design"), "{reason}");
                    }
                    ReadingCheck::Accept => panic!("accepted: {survey}"),
                }
            }
            // The refusal ends with the filled-in example.
            let error = design_error(&design_survey(&[entry])).unwrap();
            assert!(error.ends_with(athena_context::briefs::CODE_SURVEY_EXAMPLE));
        }

        /// The first refusing check of a reading exactly as built, with no survey added.
        fn gate_refuser_raw(reading: &Reading) -> Option<&'static str> {
            refusing_check(reading, "change the reading rules", None, &[], None)
                .map(|(name, _)| name)
        }

        #[test]
        fn design_check_accepts_a_complete_reading_in_each_form() {
            let [entry, gates, stop, reuse, output, direct] = complete_parts();
            assert_design_ok(&design_survey(&complete_parts()));
            assert_eq!(
                gate_refuser_raw(&design_reading(&design_survey(&complete_parts()))),
                None
            );
            // Labels in other case and with extra spaces.
            assert_design_ok(
                "Code survey: crates/a/src/b.rs.   ENTRY POINT:   `refusing_check`.  gate ORDER: `check_a`,\n`check_b`. \
             STOP gate: `check_b`. REUSES: `names_record_evidence`. output: no new output. \
             direct CHECK: criterion 1 counts the calls `refusing_check` makes to `check_b`.",
            );
            // Parts spread over two surveys.
            let mut reading = code_reading(&[], &[MODIFY_READING]);
            reading.assumptions = vec![
                design_survey(&[entry, gates, stop]),
                format!("Code survey: {reuse} {output} {direct}"),
            ];
            check_behavior_test_design(&reading).unwrap();
            // An `Existing code:` assumption or clause satisfies the reuse part; partial reuse and none both pass.
            let mut existing =
                design_reading(&design_survey(&[entry, gates, stop, output, direct]));
            existing
                .assumptions
                .push("Existing code: `names_record_evidence` in surveys.rs".into());
            check_behavior_test_design(&existing).unwrap();
            assert_design_ok(&design_survey(&[
                entry,
                gates,
                stop,
                "Existing code: `names_record_evidence`.",
                output,
                direct,
            ]));
            assert_design_ok(&design_survey(&[
                entry,
                gates,
                stop,
                "Reuses: `has_phrase`; no existing function checks gate order.",
                output,
                direct,
            ]));
            // Identifier shapes: `f()`, `a::b`, a single backticked word, several entry points.
            assert_design_ok(&design_survey(&[
                "Entry point: `check_reading_command`, reader_brief() and `replay`.",
                "Gate order: `replay`, a::b.",
                "Stop gate: a::b.",
                "Reuses: `check_reading_text`.",
                output,
                "Direct check: criterion 1 asserts the calls `replay` makes to a::b.",
            ]));
            // Other measurement verbs and subjects.
            for text in [
                "asserts `refusing_check` returns Err",
                "checks `check_code_survey` is the stop gate",
                "verifies the planner, producer and model calls it makes: counts 0",
                "compares the output of `refusing_check` with the base",
                "observes `refusing_check` stop at `check_criteria_evidence`",
                "expects the count of model calls to be 1",
            ] {
                assert_design_ok(&with_direct(text));
            }
        }

        #[test]
        fn design_check_refuses_a_gate_order_that_is_prose_or_contradicts() {
            let [entry, gates, stop, reuse, output, direct] = complete_parts();
            // A prose stop gate is a missing part (a); a stop gate outside the order is a contradiction.
            assert_design_refused(
                &design_survey(&[
                    entry,
                    gates,
                    "Stop gate: the survey check.",
                    reuse,
                    output,
                    direct,
                ]),
                &["(a)", "prose"],
            );
            assert_design_refused(
                &design_survey(&[
                    entry,
                    gates,
                    "Stop gate: `check_other`.",
                    reuse,
                    output,
                    direct,
                ]),
                &[
                    "contradiction",
                    "`check_other`",
                    "not in the stated gate order",
                ],
            );
            // One gate is not an order.
            assert_design_refused(
                &design_survey(&[
                    entry,
                    "Gate order: `check_code_survey`.",
                    stop,
                    reuse,
                    output,
                    direct,
                ]),
                &["(a)"],
            );
            // A path is not a function name, in the entry point or the reuse.
            assert_design_refused(
                &design_survey(&[
                    "Entry point: crates/a/src/b.rs.",
                    gates,
                    stop,
                    reuse,
                    output,
                    direct,
                ]),
                &["(a)"],
            );
            assert_design_refused(
                &design_survey(&[
                    entry,
                    gates,
                    stop,
                    "Reuses: crates/athena-core/src/run_pin_tests.rs.",
                    output,
                    direct,
                ]),
                &["(b)"],
            );
        }

        #[test]
        fn design_check_numbers_criteria_by_position_and_wants_one_clause_each() {
            let [entry, gates, stop, reuse, output, _] = complete_parts();
            let content = |text: &str| Criterion {
                text: text.into(),
                kind: CriterionKind::Content,
            };
            let mut reading = code_reading(&[], &[MODIFY_READING]);
            reading
                .acceptance_criteria
                .insert(0, content("The doc names the cause"));
            reading
                .acceptance_criteria
                .insert(0, content("The changelog has an entry"));
            // The behavior criterion is the third.
            let at = |n: usize| {
                format!(
                    "Direct check: criterion {n} counts the calls `refusing_check` makes to `check_code_survey`."
                )
            };
            reading.assumptions = vec![design_survey(&[entry, gates, stop, reuse, output, &at(1)])];
            let error = check_behavior_test_design(&reading)
                .unwrap_err()
                .to_string();
            assert!(error.contains("behavior criterion 3"), "{error}");
            reading.assumptions = vec![design_survey(&[entry, gates, stop, reuse, output, &at(3)])];
            check_behavior_test_design(&reading).unwrap();
            // A clause with no number covers the only behavior criterion.
            reading.assumptions = vec![design_survey(&[
                entry,
                gates,
                stop,
                reuse,
                output,
                "Direct check: counts the calls `refusing_check` makes to `check_code_survey`.",
            ])];
            check_behavior_test_design(&reading).unwrap();
            // One clause for many criteria is refused.
            add_behavior(
                &mut reading,
                "Run `cargo test -p athena-cli`: the new case prints refuse",
            );
            for cited in [
                "criteria 3-4",
                "criteria 3 and 4",
                "criteria 3, 4",
                "criterion 3 and criterion 4",
            ] {
                reading.assumptions = vec![design_survey(&[
                    entry,
                    gates,
                    stop,
                    reuse,
                    output,
                    &format!(
                        "Direct check: {cited} count the calls `refusing_check` makes to `check_code_survey`."
                    ),
                ])];
                let error = check_behavior_test_design(&reading)
                    .unwrap_err()
                    .to_string();
                assert!(
                    error.contains("one direct check covers 2 criteria"),
                    "{cited}: {error}"
                );
            }
            // Two clauses, one each.
            reading.assumptions = vec![design_survey(&[
                entry,
                gates,
                stop,
                reuse,
                output,
                &at(3),
                &at(4),
            ])];
            check_behavior_test_design(&reading).unwrap();
            assert_eq!(cited_criteria("criteria 1-4"), [1, 2, 3, 4]);
            assert_eq!(cited_criteria("Criterion 2 counts 3"), [2]);
            assert_eq!(cited_criteria("no number"), Vec::<usize>::new());
        }

        #[test]
        fn design_check_wants_a_measurement_and_a_subject_in_each_direct_check() {
            // Words about the code are not a measurement; a name alone is not one either.
            assert_design_refused(
                &with_direct("directly calls `refusing_check` through the gates"),
                &["(d)", "states no measurement"],
            );
            assert_design_refused(
                &with_direct("`check_code_survey`"),
                &["(d)", "states no measurement"],
            );
            assert_design_refused(
                &with_direct("asserts `names_record_evidence` returns true"),
                &["(d)", "names no entry point, gate or model call"],
            );
            // A name that merely contains an entry point name is not that name.
            assert_design_refused(
                &with_direct("asserts `refusing_check_twice` returns true"),
                &["names no entry point"],
            );
        }

        #[test]
        fn design_check_routine_criteria_need_no_direct_check() {
            let [entry, gates, stop, reuse, output, direct] = complete_parts();
            let survey = design_survey(&[entry, gates, stop, reuse, output, direct]);
            for routine in [
                "cargo fmt, clippy and test pass",
                "existing tests pass",
                "cargo test -p athena-cli passes",
                "Run `cargo test -p athena-core`; existing tests pass",
                "No unrelated behavior changes: cargo fmt --check, cargo clippy and cargo test pass on the final tree, as a recorded Athena check on the same tree.",
                "Run `cargo test --locked --test refusal_corpus`: all tests pass",
            ] {
                assert!(is_routine_criterion(routine), "{routine}");
                let mut reading = design_reading(&survey);
                add_behavior(&mut reading, routine);
                check_behavior_test_design(&reading).unwrap_or_else(|e| panic!("{routine}: {e}"));
            }
            // A criterion with an added property is not routine and is named when it has no clause.
            for other in [
                "cargo test passes and the greeting prints hello",
                "Run `cargo test`: the new unit test passes",
            ] {
                assert!(!is_routine_criterion(other), "{other}");
                let mut reading = design_reading(&survey);
                add_behavior(&mut reading, other);
                let error = check_behavior_test_design(&reading)
                    .unwrap_err()
                    .to_string();
                assert!(error.contains("behavior criterion 2"), "{error}");
            }
            // A reading whose only behavior criterion is routine needs no Direct check at all.
            let mut only = design_reading(&design_survey(&[entry, gates, stop, reuse, output]));
            only.acceptance_criteria = vec![Criterion {
                text: "cargo fmt, clippy and test pass".into(),
                kind: CriterionKind::Behavior,
            }];
            check_behavior_test_design(&only).unwrap();
        }

        #[test]
        fn design_check_record_output_needs_an_accepted_kind_or_a_round_trip_deliverable() {
            let [entry, gates, stop, reuse, _, direct] = complete_parts();
            let with_output =
                |output: &str| design_survey(&[entry, gates, stop, reuse, output, direct]);
            for output in [
                "Output: `athena.failure-class/1`.",
                "Output: a `athena.landing/1` record.",
                "Output: `AnyRecord::FailureClass`.",
                "Output: `AnyRecord::Candidate2`.",
                "Output: `HistoryRecord::LandingIntent`.",
                "Output: `check_record_file` accepts it.",
            ] {
                assert_design_ok(&with_output(output));
            }
            for (output, needle) in [
                ("Output: `athena.madeup/1`.", "athena.madeup/1"),
                ("Output: `AnyRecord::Nonsense`.", "AnyRecord::Nonsense"),
                (
                    "Output: `HistoryRecord::Request`.",
                    "HistoryRecord::Request",
                ),
                ("Output: `AnyRecord`.", "names no accepted record kind"),
                (
                    "Output: a new typed record.",
                    "names no accepted record kind",
                ),
                ("Output: `AnyRecord::<Variant>`.", "AnyRecord::<Variant>"),
            ] {
                assert_design_refused(&with_output(output), &["(c)", needle]);
            }
            // A round-trip test counts when a criterion names it and a deliverable can hold it.
            let round_trip = "Output: the `records_round_trip_every_kind` test.";
            let mut reading = code_reading(&[&with_output(round_trip)], &[MODIFY_READING]);
            let error = check_behavior_test_design(&reading)
                .unwrap_err()
                .to_string();
            assert!(
                error.contains("(c)") && error.contains("round-trip test"),
                "{error}"
            );
            add_behavior(
                &mut reading,
                "Run `cargo test`: `records_round_trip_every_kind` passes",
            );
            reading.deliverables.push(Deliverable {
                path: "crates/athena-data/tests/records.rs".into(),
                action: DeliverableAction::Create,
                p: 0.9,
            });
            reading.assumptions = vec![format!(
                "{} Direct check: criterion 2 asserts `refusing_check` runs `records_round_trip_every_kind`.",
                with_output(round_trip)
            )];
            check_behavior_test_design(&reading).unwrap_or_else(|e| panic!("{e}"));
            // `no new output` beside a record deliverable contradicts the deliverables.
            let mut contradiction = design_reading(&with_output(D_OUTPUT));
            contradiction.deliverables.push(Deliverable {
                path: "crates/foo/src/records.rs".into(),
                action: DeliverableAction::Modify,
                p: 0.9,
            });
            let error = check_behavior_test_design(&contradiction)
                .unwrap_err()
                .to_string();
            assert!(error.contains("says no new output"), "{error}");
        }

        #[test]
        fn record_kinds_match_the_enums() {
            fn parse(source: &str, name: &str) -> Vec<(String, String)> {
                let body = source
                    .split(&format!("pub enum {name} {{"))
                    .nth(1)
                    .and_then(|rest| rest.split("\n}\n").next())
                    .unwrap();
                let mut out = Vec::new();
                let mut lines = body.lines().map(str::trim);
                while let Some(line) = lines.next() {
                    if let Some(rest) = line.strip_prefix("#[serde(rename = \"") {
                        let tag = rest.split('"').next().unwrap().to_string();
                        let variant = lines
                            .by_ref()
                            .find(|l| !l.starts_with("///") && !l.starts_with("#["))
                            .unwrap();
                        out.push((variant.split('(').next().unwrap().to_string(), tag));
                    }
                }
                out
            }
            let any = parse(
                include_str!("../../athena-data/src/records.rs"),
                "AnyRecord",
            );
            let history = parse(
                include_str!("../../athena-data/src/history.rs"),
                "HistoryRecord",
            );
            assert!(!any.is_empty() && !history.is_empty());
            for (name, parsed) in [("AnyRecord", &any), ("HistoryRecord", &history)] {
                let listed: Vec<(String, String)> = RECORD_KINDS
                    .iter()
                    .filter(|(e, _, _)| *e == name)
                    .map(|(_, v, t)| (v.to_string(), t.to_string()))
                    .collect();
                assert_eq!(&listed, parsed, "{name}");
            }
            assert_eq!(any.len() + history.len(), RECORD_KINDS.len());
            let unknown = |error: serde_json::Error| error.to_string().contains("unknown variant");
            for (enum_name, _, tag) in RECORD_KINDS {
                let json = format!("{{\"schema\":\"{tag}\"}}");
                let rejected = match enum_name {
                    "AnyRecord" => serde_json::from_str::<AnyRecord>(&json)
                        .err()
                        .is_some_and(unknown),
                    _ => serde_json::from_str::<athena_data::history::HistoryRecord>(&json)
                        .err()
                        .is_some_and(unknown),
                };
                assert!(!rejected, "{tag}");
            }
            let json = "{\"schema\":\"athena.madeup/1\"}";
            assert!(
                serde_json::from_str::<AnyRecord>(json)
                    .err()
                    .is_some_and(unknown)
            );
            assert!(
                serde_json::from_str::<athena_data::history::HistoryRecord>(json)
                    .err()
                    .is_some_and(unknown)
            );
        }

        #[test]
        fn design_check_refuses_a_skip_switch_or_a_helper_read_as_the_measurement() {
            for text in [
                "asserts `refusing_check` is run through a skip switch",
                "asserts `refusing_check` refuses when `skip_probe` is set",
                "asserts `refusing_check` with disable_probe on",
                "asserts `refusing_check` stops with --no-probe",
                "counts the calls `refusing_check` makes with ATHENA_NO_PROBE=1",
                "asserts `refusing_check` reads an internal helper",
                "checks `refusing_check` by reading an internal helper",
                // A negator that governs another word does not carry.
                "asserts `refusing_check`: the probe does not run when ATHENA_NO_PROBE=1 is set",
                "counts no model calls using `skip_probe`",
                "asserts the probe is not run with ATHENA_NO_PROBE and `refusing_check` stops",
                "asserts `refusing_check` is not `skip_probe` but ATHENA_NO_PROBE=1",
            ] {
                assert_design_refused(
                    &with_direct(text),
                    &["(d)", "skip switch or an internal helper"],
                );
            }
            // A skip switch hidden in the reuse or the gate order is caught when the direct check cites it.
            let [entry, gates, stop, _, output, _] = complete_parts();
            assert_design_refused(
                &design_survey(&[
                    entry,
                    gates,
                    stop,
                    "Reuses: `skip_probe`.",
                    output,
                    "Direct check: criterion 1 counts the calls `refusing_check` makes with `skip_probe` set.",
                ]),
                &["skip_probe"],
            );
            let skip_gate = design_survey(&[
                entry,
                "Gate order: `skip_probe`, `check_code_survey`.",
                stop,
                "Reuses: `has_phrase`.",
                output,
                "Direct check: criterion 1 counts the calls to `skip_probe`.",
            ]);
            assert_design_refused(&skip_gate, &["skip_probe"]);
            // The same gate in the order is fine when the direct check cites the entry point.
            assert_design_ok(&skip_gate.replace(
                "counts the calls to `skip_probe`",
                "counts the calls `refusing_check` makes to `check_code_survey`",
            ));
            // A behavior criterion that measures through a skip switch or a helper is refused; the negated one is not.
            for text in [
                "Run `cargo test`: the greeting is measured through a skip switch",
                "Run `cargo test`: the rule is checked by reading an internal helper",
            ] {
                let mut reading = design_reading(&design_survey(&complete_parts()));
                reading.acceptance_criteria[0].text = text.into();
                let error = check_behavior_test_design(&reading)
                    .unwrap_err()
                    .to_string();
                assert!(
                    error.contains("criterion 1 measures through"),
                    "{text}: {error}"
                );
            }
        }

        #[test]
        fn design_check_accepts_a_negated_mention_and_names_that_look_like_switches() {
            for text in [
                "asserts `refusing_check` is not a skip switch or a read of an internal helper",
                "counts calls to `refusing_check`, not a skip switch, `skip_probe` or ATHENA_NO_PROBE",
                "asserts `refusing_check` never by reading an internal helper or adding a skip switch",
                "counts calls to `refusing_check`, not `skip_probe`",
                "counts calls to `refusing_check` without ATHENA_NO_PROBE",
                "counting planner, producer or model calls, not reads of an internal helper and not a skip switch added just for the test",
                // Names that merely look like switches.
                "asserts `refusing_check` reads INPUT_FORMS and MAX_ROUNDS and line_no",
                "asserts `refusing_check` reports `no_change` and no_change_reason",
            ] {
                assert_design_ok(&with_direct(text));
            }
            let [entry, _, stop, reuse, output, direct] = complete_parts();
            // A skip-named gate that the direct check does not cite.
            assert_design_ok(&design_survey(&[
                entry,
                "Gate order: `skip_probe`, `check_code_survey`.",
                stop,
                reuse,
                output,
                direct,
            ]));
            // A behavior criterion that says it does not measure through a switch.
            let mut reading = design_reading(&design_survey(&complete_parts()));
            reading.acceptance_criteria[0].text =
            "Run `cargo test`: the greeting is not measured through a skip switch or an internal helper".into();
            check_behavior_test_design(&reading).unwrap();
            // The identifier rule: leading part only, field names exempt.
            for yes in [
                "skip_probe",
                "ATHENA_NO_PROBE=1",
                "--no-probe",
                "disable_probe",
                "`bypass_gate`",
            ] {
                assert!(skip_identifier(yes), "{yes}");
            }
            for no in [
                "INPUT_FORMS",
                "MAX_ROUNDS",
                "line_no",
                "no_change",
                "no_change_reason",
                "no",
                "skip",
                "check_code_survey",
            ] {
                assert!(!skip_identifier(no), "{no}");
            }
        }

        #[test]
        fn design_check_accepts_the_guide_example_and_the_template_of_the_survey_error() {
            use athena_context::briefs::CODE_SURVEY_EXAMPLE;
            let reading = design_reading(CODE_SURVEY_EXAMPLE);
            check_code_survey(&reading).unwrap();
            check_behavior_test_design(&reading).unwrap();
            assert_eq!(gate_refuser_raw(&reading), None);
            // The error of `check_code_survey` ends its instruction with the same example, and following it passes both steps.
            let error = check_code_survey(&code_reading(&[], &[MODIFY_READING]))
                .unwrap_err()
                .to_string();
            let template = error.split("Example: ").nth(1).unwrap();
            assert_eq!(template, CODE_SURVEY_EXAMPLE);
            let followed = design_reading(template);
            check_code_survey(&followed).unwrap();
            check_behavior_test_design(&followed).unwrap();
        }

        #[test]
        fn guide_labels_match_the_parser() {
            use athena_context::briefs::{CODE_SURVEY_EXAMPLE, CODE_SURVEY_GUIDE};
            let example = CODE_SURVEY_EXAMPLE.to_lowercase();
            let guide = CODE_SURVEY_GUIDE.to_lowercase();
            for label in SURVEY_LABELS {
                // `existing code:` is an alias that the example and the guide leave out.
                if label == "existing code:" {
                    continue;
                }
                assert!(example.contains(label), "{label}");
                assert!(guide.contains(label.trim_end_matches(':')), "{label}");
            }
            assert!(CODE_SURVEY_GUIDE.contains("check_behavior_test_design"));
        }

        #[test]
        fn design_check_replay_share_is_within_the_declared_range() {
            // Ten stored readings: three code readings with the short survey (newly refused), three with the complete
            // survey, four docs readings. Both builds run the real rules; the base is what accepted the short survey.
            let case = |n: usize, kind: &str| {
                let mut reading = match kind {
                    "short" => code_reading(&[SURVEY], &[MODIFY_READING]),
                    "complete" => with_design(&code_reading(&[SURVEY], &[MODIFY_READING])),
                    _ => {
                        let mut docs = code_reading(&[], &[MODIFY_READING]);
                        docs.class_of_work = "docs".into();
                        docs
                    }
                };
                reading.job = JobId::parse(&case_id(n)).unwrap();
                ReplayCase {
                    job: reading.job.clone(),
                    request: "change the reading rules".into(),
                    reading,
                }
            };
            let kinds = [
                "short", "short", "short", "complete", "complete", "complete", "docs", "docs",
                "docs", "docs",
            ];
            let cases: Vec<ReplayCase> =
                kinds.iter().enumerate().map(|(n, k)| case(n, k)).collect();
            // The base build did not have the new step: it judges each reading with the complete survey added.
            let base = |request: &str, reading: &Reading| {
                Ok(check_reading_text(&with_design(reading), request))
            };
            let candidate =
                |request: &str, reading: &Reading| Ok(check_reading_text(reading, request));
            let mut reading = reading_with_criterion(
                "The change refuses more readings; expected 30% of newly refused readings, because the check now \
             asks for new required labels that past readings never used",
            );
            reading.out_of_scope = vec![];
            let declaration = replay_declaration_of(&reading);
            assert_eq!(declaration.share, Some(30));
            assert!(declaration.reason.is_some());
            let run = replay(&cases, &base, &candidate, declaration.effective_share());
            assert_eq!(run.no_replay, None);
            assert!(!run.held, "{:?}", run.newly_refused);
            assert_eq!(run.total, 10);
            assert!(!run.newly_refused.is_empty());
            assert!(
                run.newly_refused.len() * 100 <= run.total * replay_limit_percent(Some(30)),
                "{:?}",
                run.newly_refused
            );
            // Every newly refused reading is accepted once the complete survey is added: nothing else refuses it.
            for refusal in &run.newly_refused {
                let case = cases.iter().find(|c| c.job == refusal.job).unwrap();
                assert!(
                    refusal.reason.contains("check_behavior_test_design"),
                    "{}",
                    refusal.reason
                );
                assert_eq!(
                    check_reading_text(&with_design(&case.reading), &case.request),
                    ReadingCheck::Accept
                );
            }
            // A candidate that refuses every code reading is held.
            let strict =
                |request: &str, reading: &Reading| match check_reading_text(reading, request) {
                    ReadingCheck::Accept if reading.class_of_work == "code" => {
                        Ok(ReadingCheck::Refuse("too broad".into()))
                    }
                    other => Ok(other),
                };
            let held = replay(&cases, &base, &strict, declaration.effective_share());
            assert!(held.held);
        }

        #[test]
        fn design_check_refusal_is_declared_intended() {
            let source = reading_source();
            let doc = source
                .split("pub fn check_behavior_test_design")
                .next()
                .unwrap()
                .rsplit("\n\n")
                .next()
                .unwrap();
            for needle in [
                "This check refuses more readings; the higher refusal rate is intended.",
                "expected 30% of newly refused readings",
                "20261028-055601-hg0n",
                "20261028-082151-67fa",
                "20261028-055651-dhh0",
                "20261028-082240-77as",
                "SCHEMA_TAGS",
                "skip_whitespace",
                "no_change",
            ] {
                assert!(doc.contains(needle), "{needle}");
            }
        }

        #[test]
        fn design_check_changelog_entry_states_the_rule() {
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
                "check_behavior_test_design",
                "refuses more readings",
                "the higher refusal rate is intended",
                "expected 30% of newly refused readings",
                "queued proposed readings may need a re-read",
                "CODE_SURVEY_EXAMPLE",
                "SCHEMA_TAGS",
                "no_change",
                "skip_whitespace",
                "local negation",
                "routine",
            ] {
                assert!(unreleased.contains(needle), "{needle}");
            }
        }
    }

    /// A behavior criterion that names a realistic-branch fixture with the three shapes the check requires.
    const HISTORY_FIXTURE_CRITERION: &str = "Run `cargo test -p athena-cli --test reconcile`: on a \
         realistic-branch fixture with an intervention commit, a records-only commit and the job landing, the \
         rule classifies each as intended";

    /// A code reading with the survey, these deliverables and one more behavior criterion.
    fn history_reading(paths: &[&str], criterion: &str) -> Reading {
        let deliverables: Vec<(&str, DeliverableAction)> = paths
            .iter()
            .map(|p| (*p, DeliverableAction::Modify))
            .collect();
        let mut reading = code_reading(&[SURVEY], &deliverables);
        add_behavior(&mut reading, criterion);
        reading
    }

    /// The first refusing check of the reading, as the gate runs them.
    fn gate_refuser(reading: &Reading) -> Option<&'static str> {
        refusing_check(
            &with_design(reading),
            "change the reading rules",
            None,
            &[],
            None,
        )
        .map(|(name, _)| name)
    }

    /// The first refusing check of the reading for this request, as the gate runs them.
    fn refuser_for(reading: &Reading, request: &str) -> Option<&'static str> {
        refusing_check(&with_design(reading), request, None, &[], None).map(|(name, _)| name)
    }

    /// A reading that passes every check for a docs change, plus these assumptions.
    fn terms_reading(assumptions: &[&str]) -> Reading {
        let mut reading = governed_reading(&["docs/x.md"], &["docs/x.md exists"], &[]);
        reading.assumptions = assumptions.iter().map(|a| a.to_string()).collect();
        reading
    }

    /// The error of `check_defined_terms` for this request and these assumptions, if it refuses.
    fn terms_error(request: &str, assumptions: &[&str]) -> Option<String> {
        check_defined_terms(&terms_reading(assumptions), request)
            .err()
            .map(|e| e.to_string())
    }

    const LADDER_DEFINED: &str =
        "Term ladder: the ordered levels of requirements, from the design down to the code";

    /// The request of job 20261014-174815-7bf8, whose plain prose uses the ladder.
    const LADDER_REQUEST: &str = "Build what the BRIEF.md addendum '2026-10-14: A loop that steers Athena, \
         roles, and a ladder of requirements' asks for, starting small. Use the ladder to decide what it may \
         change itself.";

    #[test]
    fn defined_terms_refuses_each_undefined_structural_term() {
        for (request, term) in [
            ("Add a ladder of requirements.", "ladder"),
            ("Rename the rung.", "rung"),
            ("Show the top tier first.", "tier"),
            ("Add a caching layer.", "layer"),
            ("The ladders are long.", "ladder"),
            ("The rungs are fixed.", "rung"),
            ("Move the ladder's order.", "ladder"),
            ("Fix the TIER.", "tier"),
            (LADDER_REQUEST, "ladder"),
        ] {
            let error = terms_error(request, &[]).unwrap_or_else(|| panic!("{request}: accepted"));
            assert!(
                error.starts_with("the reading was refused because"),
                "{error}"
            );
            assert!(error.contains(term), "{request}: {error}");
            assert!(error.contains("`Term "), "{error}");
            assert!(!error.contains('\n'), "{error}");
        }
        // The gate runs it and names it as the check that refused.
        let reading = terms_reading(&[]);
        assert_eq!(
            refuser_for(&reading, LADDER_REQUEST),
            Some("check_defined_terms")
        );
        let gate = check_reading_acceptance(&reading, LADDER_REQUEST, None, &[], None)
            .unwrap_err()
            .to_string();
        assert_eq!(Some(gate), terms_error(LADDER_REQUEST, &[]));
        // A definition, in the singular or the plural, with at least three words of meaning, passes.
        assert_eq!(terms_error(LADDER_REQUEST, &[LADDER_DEFINED]), None);
        assert_eq!(
            terms_error(
                "The ladders are long.",
                &["  TERM Ladders: what the file docs/x.md calls levels"]
            ),
            None
        );
        assert_eq!(
            terms_error(
                "Add a caching layer.",
                &["Term layers: the cache between the store and the reader"]
            ),
            None
        );
        assert_eq!(
            refuser_for(&terms_reading(&[LADDER_DEFINED]), LADDER_REQUEST),
            None
        );
        // A definition of one term leaves the other undefined, and the error names only that one.
        let error = terms_error("The ladder has a tier.", &[LADDER_DEFINED]).unwrap();
        assert!(error.contains("uses tier as"), "{error}");
        assert!(!error.contains("ladder"), "{error}");
        let both = terms_error("The ladder has a tier.", &[]).unwrap();
        assert!(both.contains("ladder and tier"), "{both}");
        assert!(both.contains("define them"), "{both}");
        // Too short a meaning, another word, or another label does not define it.
        for weak in [
            "Term ladder: levels",
            "Term rung: the step of a ladder on the wall",
            "Ladder: the ordered levels of requirements",
            "Terms ladder: the ordered levels of requirements",
        ] {
            assert!(terms_error(LADDER_REQUEST, &[weak]).is_some(), "{weak}");
        }
    }

    #[test]
    fn defined_terms_ignores_generated_lines_quotes_and_parentheses() {
        // The shape of hestia's root-cause request: earlier introspections' words are cited, not used.
        let root_cause = "Priority: high\n\n\
             Fix the root cause of a repeated failure: a ladder has no rung\n\n\
             The same cause has produced failures in more than one job. Find the cause where it originates and \
             fix it there, so the whole class is prevented; do not work around one instance.\n\n\
             Failure class: Tier problem (20261001-000000-abcd)\n\
             Cause: the layer is undefined\n\
             Seen 2 time(s) in 2 jobs. The introspections that record it:\n\
             Introspection 20261029-181811-bs86 (job 20261029-111202-2bpc); prevention it proposed: add a ladder\n\
             Introspection 20261030-170739-hq09 (job 20261030-060751-5389); prevention it proposed: a tier list\n\n\
             Acceptance:\n- The root cause is named\n\nFiles: crates/\n";
        assert_eq!(terms_error(root_cause, &[]), None);
        // The shape of the approved-work request: the rung line, the proposal line and the copied sections.
        let approval = "Change the thing\n\n\
             A person approved this proposal from the steward role. Carry the work out as an ordinary change.\n\
             Everything the proposal cites is copied below; nothing here points at a file you cannot open.\n\n\
             Proposal: 20261015-003903-khz6 (key k), queued at 2026-10-15T00:00:00Z by the steward role.\n\
             Rung of the requirements ladder: design decisions.\n\n\
             ## Source\n\nThe ladder has five rungs in this layer.\n\n## Evidence\n\n### 1. a tier\n";
        assert_eq!(terms_error(approval, &[]), None);
        // Quoted spans: double, curly, backtick and single.
        for quoted in [
            "Rename \"ladder\" to `tier` and 'layer' in the guide.",
            "Rename \u{201C}ladder\u{201D} and \u{2018}rung\u{2019} in the guide.",
            "The addendum '2026-10-14: A ladder of requirements' asks for roles.",
            "Rename `the ladder of tiers` in the guide.",
            "Keep the code block:\n```\nlet ladder = tier;\n```\nOnly.",
        ] {
            assert_eq!(terms_error(quoted, &[]), None, "{quoted}");
        }
        // Parenthesised example lists: two or more commas, or an example opening.
        for listed in [
            "Define the terms (ladder, layer, tier) in the guide.",
            "Define the terms (e.g. a tier) in the guide.",
            "Define the terms (such as a rung) in the guide.",
            "Define the terms (for example the layer) in the guide.",
        ] {
            assert_eq!(terms_error(listed, &[]), None, "{listed}");
        }
        // Paths and identifiers are not uses.
        assert_eq!(
            terms_error(
                "Edit crates/x/layer/mod.rs, layer_count, Ladder::new and ladder.md.",
                &[]
            ),
            None
        );
    }

    #[test]
    fn defined_terms_keeps_hand_written_lines_and_single_comma_parentheticals() {
        for (request, term) in [
            ("Introspection of the ladder is needed.", "ladder"),
            ("Cause: the ladder is undefined", "ladder"),
            ("Failure class: the tier is undefined", "tier"),
            (
                "Update the guide.\n\nRung of the requirements ladder: x.\n\n## Source\n\nthe tier",
                "ladder",
            ),
            (
                "Define it (the ladder, which ranks requirements).",
                "ladder",
            ),
            ("Fix the 'ladder", "ladder"),
            ("Sri's note: don't rename the ladder", "ladder"),
            ("Define the ladder (a list, of two).", "ladder"),
        ] {
            let error = terms_error(request, &[]).unwrap_or_else(|| panic!("{request}: accepted"));
            assert!(error.contains(term), "{request}: {error}");
        }
        // The anchor sentences skip only the exact generated shapes; the title and the sentences between count.
        let anchored_root_cause = "Introspection of the ladder is needed.\n\
             The same cause has produced failures in more than one job.\nCause: the rung";
        let error = terms_error(anchored_root_cause, &[]).unwrap();
        assert!(
            error.contains("ladder") && !error.contains("rung"),
            "{error}"
        );
        let approval = "Fix the ladder ordering\n\n\
             A person approved this proposal from the steward role. The tier matters.\n\n\
             Rung of the requirements ladder: design decisions.\n\n## Source\n\nthe layer";
        let error = terms_error(approval, &[]).unwrap();
        assert!(error.contains("ladder and tier"), "{error}");
        assert!(!error.contains("layer"), "{error}");
    }

    #[test]
    fn defined_terms_matches_whole_words_only() {
        for request in [
            "Rename the stepladder helper and remove the delayer module.",
            "Add a multilayered cache and a laddered scale.",
            "Update the guide.",
            "Edit ladder.md and layer.rs.",
            "The tiers_list field is renamed.",
            "Avoid the word 'layer' here.",
        ] {
            assert_eq!(terms_error(request, &[]), None, "{request}");
            assert_eq!(refuser_for(&terms_reading(&[]), request), None, "{request}");
        }
        // Positive controls: the plural is a use.
        assert!(terms_error("Document the layers.", &[]).is_some());
        assert!(terms_error("Count the rungs.", &[]).is_some());
    }

    #[test]
    fn defined_terms_skips_declines_questions_and_no_deliverables() {
        let request = "Add a ladder of requirements.";
        assert!(terms_error(request, &[]).is_some());
        let mut declined = terms_reading(&[]);
        declined.capability = None;
        declined.decline_reason = Some("no capability fits".into());
        check_defined_terms(&declined, request).unwrap();
        let mut asking = terms_reading(&[]);
        asking.question = Some("Which ladder?".into());
        check_defined_terms(&asking, request).unwrap();
        let mut none = terms_reading(&[]);
        none.deliverables.clear();
        check_defined_terms(&none, request).unwrap();
    }

    #[test]
    fn defined_terms_has_one_step_line_after_environment_reach() {
        let source = reading_source();
        let body = source
            .split("pub fn refusing_check(")
            .nth(1)
            .and_then(|rest| rest.split("\n}\n").next())
            .unwrap();
        let steps: Vec<&str> = body
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with("step!(check_"))
            .collect();
        let line = "step!(check_defined_terms(reading, request));";
        assert_eq!(steps.iter().filter(|l| **l == line).count(), 1, "{steps:?}");
        let reach = steps
            .iter()
            .position(|l| *l == "step!(check_environment_reach(reading));")
            .unwrap();
        assert_eq!(steps[reach + 1], line);
        let acceptance_doc = source
            .split("pub fn check_reading_acceptance(")
            .next()
            .unwrap();
        assert!(acceptance_doc.contains("[`check_defined_terms`]"));
    }

    #[test]
    fn defined_terms_refusal_needs_the_step_line() {
        // The refusal comes from the gate and not from a direct call: the gate refuses the undefined-ladder
        // reading under the check's own name, and the step line is in its body.
        let reading = terms_reading(&[]);
        assert_eq!(
            refuser_for(&reading, LADDER_REQUEST),
            Some("check_defined_terms")
        );
        assert!(check_reading_acceptance(&reading, LADDER_REQUEST, None, &[], None).is_err());
        let body = reading_source()
            .split("pub fn refusing_check(")
            .nth(1)
            .and_then(|rest| rest.split("\n}\n").next())
            .unwrap();
        assert!(body.contains("step!(check_defined_terms(reading, request));"));
    }

    /// The text of the `///` block above `pub fn check_defined_terms(`, as one line.
    fn defined_terms_doc() -> String {
        let before = reading_source()
            .split("pub fn check_defined_terms(")
            .next()
            .unwrap();
        let mut doc_lines: Vec<&str> = before
            .lines()
            .rev()
            .take_while(|l| l.trim_start().starts_with("///"))
            .map(|l| l.trim_start().trim_start_matches('/'))
            .collect();
        doc_lines.reverse();
        doc_lines
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn defined_terms_doc_and_changelog_name_the_cause() {
        let before = reading_source()
            .split("pub fn check_defined_terms(")
            .next()
            .unwrap();
        let doc_block = before.rsplit("\n\n").next().unwrap();
        assert!(
            doc_block
                .lines()
                .all(|l| l.starts_with("///") && l != "///"),
            "{doc_block}"
        );
        let doc = defined_terms_doc();
        for needle in [
            "readings are not checked against the outcome the user observes",
            "structural terms",
            "20261014-174815-7bf8",
            "20261017-173508-0wtc",
            "quoted",
            "terms outside the list",
            "Hermes",
            "no deliverable",
        ] {
            assert!(doc.contains(needle), "the doc comment lacks {needle:?}");
        }
        let changelog = include_str!("../../../CHANGELOG.md")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        for needle in [
            "reading::check_defined_terms",
            "readings are not checked against the outcome the user observes",
            "structural terms",
            "20261014-174815-7bf8",
        ] {
            assert!(changelog.contains(needle), "the changelog lacks {needle:?}");
        }
    }

    #[test]
    fn defined_terms_refusal_is_declared_intended() {
        let declared = replay_declaration(
            &[Criterion {
                text: "The change refuses more readings: expected 2% of newly refused readings"
                    .into(),
                kind: CriterionKind::Behavior,
            }],
            &[],
        );
        assert_eq!(declared.share, Some(2));
        let doc = defined_terms_doc();
        let changelog = include_str!("../../../CHANGELOG.md")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let decisions = include_str!("../../../docs/decisions.md")
            .split("## D164 ·")
            .nth(1)
            .expect("D164 is in the decision log")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        // D164 keeps the share it declared; the doc comment and the changelog carry the current one (D165).
        assert!(decisions.contains("expected 3% of newly refused readings"));
        // The share is judged in the defined-terms item only: another check (`check_environment_reach`, see
        // D165) now declares 3% in the same Unreleased section.
        let unreleased = include_str!("../../../CHANGELOG.md")
            .split("## [Unreleased]")
            .nth(1)
            .unwrap()
            .split("\n## [")
            .next()
            .unwrap();
        let mut items: Vec<String> = Vec::new();
        for line in unreleased.lines() {
            if line.starts_with("- ") {
                items.push(line.to_string());
            } else if let Some(last) = items.last_mut() {
                last.push(' ');
                last.push_str(line);
            }
        }
        let item = items
            .iter()
            .map(|i| i.split_whitespace().collect::<Vec<_>>().join(" "))
            .find(|i| {
                i.contains("`reading::check_defined_terms`")
                    && i.contains("expected 2% of newly refused readings")
            })
            .expect("a changelog item declares the share of the defined-terms check");
        for (label, text) in [("the doc comment", &doc), ("the changelog item", &item)] {
            assert!(
                text.contains("expected 2% of newly refused readings"),
                "{label} lacks the current share"
            );
            assert!(
                !text.contains("expected 3% of newly refused readings"),
                "{label} still holds the share of D164"
            );
        }
        for (label, text) in [
            ("the doc comment", &doc),
            ("the changelog", &changelog),
            ("D164", &decisions),
        ] {
            for needle in [
                "This check refuses more readings; the higher refusal rate is intended",
                "estimate",
                "hand-written requests",
                "(ladder, rung) are refused until the reading restates them",
                "priority tiers",
                "caching layer",
                "replay record at verification",
                "held until a person runs `athena go`",
            ] {
                assert!(text.contains(needle), "{label} lacks {needle:?}");
            }
        }
        assert!(
            doc.contains("This check refuses more readings; the higher refusal rate is intended."),
            "{doc}"
        );
        for needle in [
            "decide whether to narrow",
            "approve_request",
            "accept_request",
            "shared delimiter",
        ] {
            assert!(decisions.contains(needle), "D164 lacks {needle:?}");
        }
        // The pinned false refusals: priority tiers and a caching layer need a definition.
        for request in [
            "Order the work items by priority tier.",
            "Add a caching layer.",
        ] {
            assert_eq!(
                refuser_for(&terms_reading(&[]), request),
                Some("check_defined_terms"),
                "{request}"
            );
        }
        // The check_outcome_state comment no longer says the other two jobs are outside any check.
        let outcome_doc = reading_source()
            .split("pub fn check_outcome_state(")
            .next()
            .unwrap()
            .rsplit("\n\n")
            .next()
            .unwrap()
            .lines()
            .map(|l| l.trim_start_matches('/').trim())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(outcome_doc.contains("check_defined_terms"), "{outcome_doc}");
        assert!(
            !outcome_doc.contains("which does not catch them"),
            "{outcome_doc}"
        );
    }

    const HERMES_DEFINED: &str = "Term Hermes: a studied agent, not Athena";

    #[test]
    fn defined_terms_refuses_an_ambiguous_ontology_name() {
        // A plain-prose use of the name is refused: the exact word, the possessive, another case, a bullet.
        for request in [
            "Compare Hermes with Athena.",
            "Hermes's skills differ.",
            "Hermes\u{2019}s skills differ.",
            "Fix the hermes note.",
            "Fix the HERMES note.",
            "Acceptance:\n- Hermes Agent is described in the guide",
            "Hermes",
        ] {
            let error =
                terms_error(request, &[]).unwrap_or_else(|| panic!("{request:?}: accepted"));
            assert!(
                error.starts_with("the reading was refused because"),
                "{error}"
            );
            assert!(error.contains("uses Hermes as"), "{request:?}: {error}");
            assert!(error.contains("`Term Hermes:"), "{error}");
            assert!(error.contains("ambiguous name"), "{error}");
            assert!(!error.contains('\n'), "{error}");
            assert_eq!(
                refuser_for(&terms_reading(&[]), request),
                Some("check_defined_terms"),
                "{request:?}"
            );
        }
        // Quotes, backticks, example lists, fenced code, paths, identifiers and other words are not uses, and
        // neither is a request with no listed name.
        for request in [
            "Rename \"Hermes\" in the guide.",
            "Rename `Hermes` in the guide.",
            "Rename \u{201C}Hermes\u{201D} in the guide.",
            "Rename 'Hermes' in the guide.",
            "Compare the studies (firstmate, Hermes, Athena) in the guide.",
            "Compare the studies (e.g. Hermes) in the guide.",
            "Keep the code block:\n```\nlet agent = Hermes;\n```\nOnly.",
            "Edit docs/hermes.md and hermes_notes.",
            "Call Hermes::new.",
            "Rename the hermeshub helper.",
            "Fix the Hermesian note.",
            "Update the guide so that it lists the status flag.",
        ] {
            assert_eq!(terms_error(request, &[]), None, "{request:?}");
        }
        // A `Term Hermes:` line of three or more words defines it, in any case; fewer words do not.
        let request = "Compare Hermes with Athena.";
        for defined in [
            HERMES_DEFINED,
            "term hermes: a studied agent, not Athena",
            "  TERM HERMES: the Orchestration god of the Olympus Pantheon, per docs/ontology.toml",
        ] {
            assert_eq!(terms_error(request, &[defined]), None, "{defined}");
        }
        for weak in [
            "Term Hermes: agent",
            "Term Hermes: a studied",
            "Hermes: a studied agent here",
            "Terms Hermes: a studied agent here",
            "Term Zeus: a studied agent here",
        ] {
            assert!(terms_error(request, &[weak]).is_some(), "{weak}");
        }
        // A definition of the name leaves a structural term undefined, and the reverse.
        let error = terms_error("Compare the ladder with Hermes.", &[HERMES_DEFINED]).unwrap();
        assert!(error.contains("uses ladder as"), "{error}");
        assert!(!error.contains("Hermes"), "{error}");
        let error = terms_error("Compare the ladder with Hermes.", &[LADDER_DEFINED]).unwrap();
        assert!(error.contains("uses Hermes as"), "{error}");
        assert!(!error.contains("ladder"), "{error}");
        let both = terms_error("Compare the ladder with Hermes.", &[]).unwrap();
        assert!(both.contains("ladder and Hermes"), "{both}");
        assert!(both.contains("define them"), "{both}");
        assert_eq!(
            terms_error(
                "Compare the ladder with Hermes.",
                &[LADDER_DEFINED, HERMES_DEFINED]
            ),
            None
        );
        // The gate gives the same text.
        let gate = check_reading_acceptance(&terms_reading(&[]), request, None, &[], None)
            .unwrap_err()
            .to_string();
        assert_eq!(Some(gate), terms_error(request, &[]));
        // A reading with no capability, a question or no deliverable is skipped, as for a structural term.
        let mut asking = terms_reading(&[]);
        asking.question = Some("Which Hermes?".into());
        check_defined_terms(&asking, request).unwrap();
    }

    /// Problems in how the two lists classify the ontology terms: a name in neither list, in both, a listed name
    /// that is not an ontology term, an empty list or an ungated entry with a reason of fewer than three words.
    /// Names are compared exactly, with case.
    fn ontology_problems_for(
        listed: &[&str],
        ungated: &[(&str, &str)],
        ontology: &[String],
    ) -> Vec<String> {
        let mut found = Vec::new();
        if listed.is_empty() {
            found.push("AMBIGUOUS_NAMES is empty".to_string());
        }
        for name in ontology {
            let gated = listed.contains(&name.as_str());
            let free = ungated.iter().any(|(n, _)| n == name);
            if !gated && !free {
                found.push(format!(
                    "{name} is a term of docs/ontology.toml in neither AMBIGUOUS_NAMES nor UNGATED_TERMS; add it \
                     to AMBIGUOUS_NAMES if a reading must define it, or to UNGATED_TERMS with a reason"
                ));
            }
            if gated && free {
                found.push(format!(
                    "{name} is in both AMBIGUOUS_NAMES and UNGATED_TERMS"
                ));
            }
        }
        for name in listed {
            if !ontology.iter().any(|n| n == name) {
                found.push(format!(
                    "{name} is in AMBIGUOUS_NAMES but is not a [[term]] name of docs/ontology.toml"
                ));
            }
        }
        for (name, reason) in ungated {
            if !ontology.iter().any(|n| n == name) {
                found.push(format!(
                    "{name} is in UNGATED_TERMS but is not a [[term]] name of docs/ontology.toml"
                ));
            }
            if reason.split_whitespace().count() < 3 {
                found.push(format!("the reason for {name} is shorter than three words"));
            }
        }
        found
    }

    #[test]
    fn ambiguous_and_ungated_terms_cover_the_ontology() {
        let terms =
            crate::mirrors::parse_ontology(include_str!("../../../docs/ontology.toml")).unwrap();
        let ontology: Vec<String> = terms.into_iter().map(|t| t.name).collect();
        let listed: Vec<&str> = AMBIGUOUS_NAMES.to_vec();
        let ungated: Vec<(&str, &str)> = UNGATED_TERMS.to_vec();
        assert_eq!(
            ontology_problems_for(&listed, &ungated, &ontology),
            Vec::<String>::new()
        );
        assert!(listed.contains(&"Hermes"));
        // Every gated name is one word, so the word matcher can see it, and it comes back as written.
        for name in &listed {
            assert!(name.chars().all(char::is_alphabetic), "{name}");
            assert_eq!(named_term(&name.to_lowercase()), Some(*name));
        }
        // Controls: each fault is reported.
        let with = |extra: &str| -> Vec<String> {
            let mut names = ontology.clone();
            names.push(extra.to_string());
            names
        };
        let unclassified = ontology_problems_for(&listed, &ungated, &with("Zeus"));
        assert_eq!(unclassified.len(), 1, "{unclassified:?}");
        assert!(
            unclassified[0].contains("Zeus") && unclassified[0].contains("UNGATED_TERMS"),
            "{unclassified:?}"
        );
        let mut more = listed.clone();
        more.push("Zeus");
        let missing = ontology_problems_for(&more, &ungated, &ontology);
        assert_eq!(missing.len(), 1, "{missing:?}");
        assert!(missing[0].contains("Zeus") && missing[0].contains("not a [[term]]"));
        let mut more = ungated.clone();
        more.push(("Zeus", "a god of the Olympus"));
        let missing = ontology_problems_for(&listed, &more, &ontology);
        assert_eq!(missing.len(), 1, "{missing:?}");
        let mut both = ungated.clone();
        both.push(("Hermes", "also named in the other list"));
        let twice = ontology_problems_for(&listed, &both, &ontology);
        assert_eq!(twice.len(), 1, "{twice:?}");
        assert!(twice[0].contains("both"), "{twice:?}");
        let mut short = ungated.clone();
        short[0].1 = "common";
        assert_eq!(ontology_problems_for(&listed, &short, &ontology).len(), 1);
        // The comparison is exact: a name in another case is a different name.
        let lowercase = ontology_problems_for(&["hermes"], &ungated, &ontology);
        assert_eq!(lowercase.len(), 2, "{lowercase:?}");
        assert_eq!(ontology_problems_for(&[], &ungated, &ontology).len(), 2);
    }

    #[test]
    fn ambiguous_name_doc_and_error_text() {
        let doc = defined_terms_doc();
        for needle in [
            "readings are not checked against the outcome the user observes",
            "20261014-082328-g6vy",
            "20261014-174815-7bf8",
            "20261017-173508-0wtc",
            "covers 2 of the 3 cited jobs",
            "expected 2% of newly refused readings",
            "AMBIGUOUS_NAMES",
            "docs/ontology.toml",
            "cannot verify that the restatement matches the recorded senses",
            "This check refuses more readings; the higher refusal rate is intended.",
        ] {
            assert!(doc.contains(needle), "the doc comment lacks {needle:?}");
        }
        for stale in [
            "whose recorded request holds no word of the list",
            "covers 1 of the 3 cited jobs",
            "no check catches",
        ] {
            assert!(!doc.contains(stale), "the doc comment still says {stale:?}");
        }
        let error = terms_error("Compare Hermes with Athena.", &[]).unwrap();
        for needle in [
            "ambiguous name",
            "`Term Hermes:",
            "which of its recorded senses applies",
            "cite docs/ontology.toml",
            "at least three words",
        ] {
            assert!(error.contains(needle), "{needle:?}: {error}");
        }
        // The error of a structural term alone names no ambiguous name.
        let error = terms_error("Add a caching layer.", &[]).unwrap();
        assert!(!error.contains("Hermes"), "{error}");
    }

    /// Asserts the reading is refused by the history fixture check, then accepted once the fixture criterion
    /// is added, and returns the refusal text.
    fn refused_then_accepted(reading: Reading) -> String {
        assert_eq!(gate_refuser(&reading), Some("check_history_fixture"));
        let error = accept(&reading).unwrap_err().to_string();
        let mut fixed = reading;
        add_behavior(&mut fixed, HISTORY_FIXTURE_CRITERION);
        assert_eq!(gate_refuser(&fixed), None);
        accept(&fixed).unwrap();
        error
    }

    #[test]
    fn history_fixture_refuses_a_rule_tested_only_on_job_commits() {
        let reading = history_reading(
            &["crates/athena-core/src/land.rs"],
            "Run `cargo test -p athena-core land::`: pr_commits accepts the job commits",
        );
        let error = refused_then_accepted(reading.clone());
        for needle in [
            "was refused",
            "\"pr_commits\"",
            "realistic-branch fixture",
            "intervention commit",
            "records-only commit",
            "job landing",
            "retry-after-block",
        ] {
            assert!(error.contains(needle), "{needle}: {error}");
        }
        // The check refuses it on its own, with the same text.
        assert_eq!(
            check_history_fixture(&reading).unwrap_err().to_string(),
            error
        );
        // Spellings of the fixture shapes that are accepted.
        for criterion in [
            "Run `cargo test`: a realistic fixture holds an intervention commit, a records only commit and \
             the job's landing",
            "Run `cargo test`: a realistic fixture holds an intervention commit, a records-only commit and \
             the job\u{2019}s landing",
            "Run `cargo test`: the  realistic\n fixtures hold intervention commits, records-only commits and \
             the job landing",
        ] {
            let mut spelled = reading.clone();
            add_behavior(&mut spelled, criterion);
            assert_eq!(gate_refuser(&spelled), None, "{criterion}");
        }
        // Near misses stay refused: a content criterion, a missing shape, a missing word.
        let mut content = reading.clone();
        add_content(&mut content, HISTORY_FIXTURE_CRITERION);
        assert_eq!(gate_refuser(&content), Some("check_history_fixture"));
        for criterion in [
            "Run `cargo test`: a realistic fixture holds a records-only commit and the job landing",
            "Run `cargo test`: a realistic fixture holds an intervention commit and the job landing",
            "Run `cargo test`: a realistic fixture holds an intervention commit and a records-only commit",
            "Run `cargo test`: an intervention commit, a records-only commit and the job landing are tested",
            "Run `cargo test`: a fixture holds an intervention commit, a records-only commit and the job \
             landing",
            "Run `cargo test`: a realistic branch holds an intervention commit, a records-only commit and \
             the job landing",
        ] {
            let mut near = reading.clone();
            add_behavior(&mut near, criterion);
            assert_eq!(
                gate_refuser(&near),
                Some("check_history_fixture"),
                "{criterion}"
            );
        }
        // Every trigger phrase refuses, and its own text is named in the error.
        for phrase in HISTORY_RULE_PHRASES {
            let reading = history_reading(
                &["crates/athena-core/src/land.rs"],
                &format!("Run `cargo test`: the {phrase} is covered"),
            );
            let error = check_history_fixture(&reading).unwrap_err().to_string();
            assert!(
                error.contains(&format!("\"{phrase}\"")),
                "{phrase}: {error}"
            );
        }
    }

    #[test]
    fn history_fixture_job_1rve_shape() {
        // Job 20261024-153908-1rve changed the pr_commits content classification in land.rs, gh.rs and
        // invariant.rs, with a note in the records-history design doc and the changelog. Rebuilt from the
        // job's deliverables.
        let reading = history_reading(
            &[
                "crates/athena-core/src/land.rs",
                "crates/athena-core/src/gh.rs",
                "crates/athena-gate/src/invariant.rs",
                "docs/design/records-history.md",
                "CHANGELOG.md",
            ],
            "Run `cargo test -p athena-core land::`: pr_commits classifies each pull request commit by its \
             content",
        );
        refused_then_accepted(reading);
    }

    #[test]
    fn history_fixture_job_e65v_shape() {
        // Job 20261024-154552-e65v made the reconcile merge method follow the commit shape.
        let reading = history_reading(
            &[
                "crates/athena-core/src/land.rs",
                "crates/athena-gate/src/invariant.rs",
            ],
            "Run `cargo test -p athena-core land::`: the merge method follows the commit shape of the branch",
        );
        refused_then_accepted(reading);
    }

    #[test]
    fn history_fixture_job_hac3_shape() {
        // Job 20261025-012657-hac3 stopped writing Athena trailers after the cutover commit.
        let reading = history_reading(
            &[
                "crates/athena-core/src/commits.rs",
                "crates/athena-core/src/manage.rs",
                "crates/athena-git/src/attrs.rs",
                "crates/athena-core/src/status.rs",
                "crates/athena-core/src/auditor.rs",
            ],
            "Run `cargo test -p athena-cli --test lifecycle`: no Athena trailers are written after C0",
        );
        refused_then_accepted(reading);
        // A trigger in a criterion alone is enough, whatever files the job lists.
        let criterion_only = history_reading(
            &[
                "crates/athena-core/src/manage.rs",
                "crates/athena-core/src/status.rs",
            ],
            "Run `cargo test`: status prints the trailer of the last landing",
        );
        refused_then_accepted(criterion_only);
        // A content criterion can trigger it too.
        let mut content_trigger = code_reading(&[SURVEY], &[MODIFY_READING]);
        add_content(&mut content_trigger, "The doc names the history check");
        assert_eq!(
            gate_refuser(&content_trigger),
            Some("check_history_fixture")
        );
    }

    #[test]
    fn history_fixture_leaves_unrelated_readings_alone() {
        let neutral = "Run `cargo test`: the new unit test passes";
        let mut cases: Vec<(&str, Reading)> = Vec::new();
        // A docs reading that mentions the history check.
        let mut docs = history_reading(
            &["docs/guide.md"],
            "The history check reads the job records on the records branch",
        );
        docs.class_of_work = "docs".into();
        cases.push(("docs", docs));
        cases.push((
            "code touching none of it",
            history_reading(&["crates/athena-core/src/reading.rs"], neutral),
        ));
        // Job fe1df4f: a green CI result is reused as the baseline when the fingerprint matches.
        cases.push((
            "ci baseline",
            history_reading(
                &[
                    "crates/athena-core/src/reconcile.rs",
                    "crates/athena-core/src/checks.rs",
                ],
                "Run `cargo test -p athena-core`: a green CI result is reused as the job baseline when the \
                 fingerprint matches",
            ),
        ));
        // Job 1bc7736: a job lands only on the branch it was dispatched from.
        cases.push((
            "dispatch branch",
            history_reading(
                &["crates/athena-core/src/land.rs"],
                "Run `cargo test -p athena-core land::`: a job lands only on the branch it was dispatched from",
            ),
        ));
        cases.push((
            "landing queue",
            history_reading(
                &["crates/athena-core/src/gh.rs", "crates/athena-core/src/gh/landing.rs"],
                "Run `cargo test -p athena-core gh::`: landing is first in, first out; a job waits for the \
                 early CI run and the landing time is recorded",
            ),
        ));
        cases.push((
            "listed path alone",
            history_reading(&["crates/athena-core/src/land.rs"], neutral),
        ));
        cases.push((
            "athena-data history",
            history_reading(&["crates/athena-data/src/history.rs"], neutral),
        ));
        cases.push(("tests class", {
            let mut tests = history_reading(&["crates/athena-core/tests/reconcile.rs"], neutral);
            tests.class_of_work = "tests".into();
            tests
        }));
        // A decline, a question, and content criteria only.
        let mut declined =
            history_reading(&["crates/athena-core/src/land.rs"], "pr_commits is fixed");
        declined.capability = None;
        declined.decline_reason = Some("no capability fits".into());
        cases.push(("decline", declined));
        let mut asking =
            history_reading(&["crates/athena-core/src/land.rs"], "pr_commits is fixed");
        asking.question = Some("which file?".into());
        cases.push(("question", asking));
        let mut content = code_reading(&[SURVEY], &[MODIFY_READING]);
        content.acceptance_criteria = vec![Criterion {
            text: "The doc names the history check".into(),
            kind: CriterionKind::Content,
        }];
        cases.push(("content only", content));
        // The survey assumption is not searched: it may name a function the change only reads.
        cases.push(("survey names classify_commit", {
            let mut surveyed = history_reading(&["crates/athena-core/src/land.rs"], neutral);
            surveyed.assumptions = vec![format!("{SURVEY}; classify_commit in land.rs was read")];
            surveyed
        }));
        for (label, reading) in &cases {
            check_history_fixture(reading).unwrap_or_else(|e| panic!("{label}: {e}"));
            assert_ne!(
                gate_refuser(reading),
                Some("check_history_fixture"),
                "{label}"
            );
        }
        // The word boundaries are those of `has_phrase`: an underscore or a hyphen is a boundary.
        assert!(!has_phrase("the trailing newline", "trailer"));
        assert!(!has_phrase("a trailered job", "trailer"));
        assert!(has_phrase("a trailer_x field", "trailer"));
        assert!(has_phrase("a trailer-x field", "trailer"));
    }

    #[test]
    fn history_fixture_has_one_step_line() {
        let source = reading_source();
        let body = source
            .split("pub fn refusing_check(")
            .nth(1)
            .and_then(|rest| rest.split("\n}\n").next())
            .unwrap();
        let steps: Vec<&str> = body
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with("step!(check_"))
            .collect();
        let line = "step!(check_history_fixture(reading));";
        assert_eq!(steps.iter().filter(|l| **l == line).count(), 1, "{steps:?}");
        let survey = steps
            .iter()
            .position(|l| *l == "step!(check_code_survey(reading));")
            .unwrap();
        assert_eq!(
            steps[survey + 1],
            "step!(check_behavior_test_design(reading));"
        );
        assert_eq!(steps[survey + 2], line);
        let acceptance_doc = source
            .split("pub fn check_reading_acceptance(")
            .next()
            .unwrap();
        assert!(acceptance_doc.contains("[`check_history_fixture`]"));
    }

    #[test]
    fn history_fixture_refusal_needs_the_step_line() {
        // The refusal comes from the gate and not from a direct call: a reading the check refuses is refused
        // by `refusing_check` under the check's own name, and the step line is in its body.
        let reading = history_reading(
            &["crates/athena-core/src/land.rs"],
            "Run `cargo test`: pr_commits accepts the job commits",
        );
        assert_eq!(gate_refuser(&reading), Some("check_history_fixture"));
        assert!(accept(&reading).is_err());
        let body = reading_source()
            .split("pub fn refusing_check(")
            .nth(1)
            .and_then(|rest| rest.split("\n}\n").next())
            .unwrap();
        assert!(body.contains("step!(check_history_fixture(reading));"));
    }

    #[test]
    fn history_fixture_doc_and_changelog_name_the_cause() {
        fn collapse(text: &str) -> String {
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        }
        const CAUSE: &str = "derived from the happy-path commit shape";
        let source = reading_source();
        assert!(source.contains("pub fn check_history_fixture(reading: &Reading) -> Result<()>"));
        let before = source
            .split("pub fn check_history_fixture(")
            .next()
            .unwrap();
        let mut doc_lines: Vec<&str> = before
            .lines()
            .rev()
            .take_while(|l| l.trim_start().starts_with("///"))
            .map(|l| l.trim_start().trim_start_matches('/'))
            .collect();
        doc_lines.reverse();
        let doc = collapse(&doc_lines.join(" "));
        let changelog = include_str!("../../../CHANGELOG.md");
        let unreleased = changelog
            .split("## [Unreleased]")
            .nth(1)
            .unwrap()
            .split("\n## [")
            .next()
            .unwrap();
        let mut items: Vec<String> = Vec::new();
        for line in unreleased.lines() {
            if line.starts_with("- ") {
                items.push(line.to_string());
            } else if let Some(last) = items.last_mut() {
                last.push(' ');
                last.push_str(line);
            }
        }
        let items: Vec<String> = items.iter().map(|i| collapse(i)).collect();
        let item = items
            .iter()
            .find(|i| i.contains("reading::check_history_fixture"))
            .expect("a changelog item names the check");
        for (label, text) in [("doc comment", &doc), ("changelog item", item)] {
            for needle in [
                CAUSE,
                "20261024-153908-1rve",
                "20261024-154552-e65v",
                "20261025-012657-hac3",
                "where readings are accepted",
                "retry-after-block",
                "is not enforced",
                "no reader rule line",
                "higher refusal rate is intended",
                "expected 4% of newly refused readings",
                "estimate",
            ] {
                assert!(text.contains(needle), "the {label} lacks {needle:?}");
            }
        }
        let declared = replay_declaration(
            &[Criterion {
                text: "This check refuses more readings; expected 4% of newly refused readings."
                    .into(),
                kind: CriterionKind::Content,
            }],
            &[],
        );
        assert_eq!(declared.share, Some(4));
        // The doc's own declaration parses to the same share.
        assert_eq!(
            replay_declaration(
                &[Criterion {
                    text: doc.clone(),
                    kind: CriterionKind::Content
                }],
                &[]
            )
            .share,
            Some(4)
        );
    }

    /// A criterion that runs a landing-time measure on fixture records only.
    const FIXTURE_ONLY_CRITERION: &str = "Run `cargo test -p athena-core status::`: over a fixture of three \
         landings the landing time is 4 minutes";

    /// A criterion that runs the measure on the real records and takes the range from commit times in
    /// `history.json`, a source the change does not compute.
    const REAL_RECORDS_CRITERION: &str = "Run `cargo run -p athena-cli -- status --snapshot evidence`: on the \
         real records the landing time it prints is within the expected range taken from the commit times in \
         history.json";

    /// A code reading with the survey, these deliverables, this summary and one more behavior criterion.
    fn measure_reading(summary: &str, paths: &[&str], criterion: &str) -> Reading {
        let mut reading = history_reading(paths, criterion);
        reading.summary = summary.into();
        reading
    }

    const LANDING_TIME_SUMMARY: &str = "Fix the landing time shown by status";

    #[test]
    fn records_measure_without_real_records_criterion_is_refused() {
        // Criterion 3: the landing-time reading of 2026-10-29, verified on fixtures only.
        let reading = measure_reading(
            LANDING_TIME_SUMMARY,
            &["crates/athena-core/src/status.rs"],
            FIXTURE_ONLY_CRITERION,
        );
        assert_eq!(gate_refuser(&reading), Some("check_records_measure"));
        let error = accept(&reading).unwrap_err().to_string();
        for needle in [
            "was refused",
            "\"landing time\"",
            "`real records`",
            "independent source",
            "`history.json`",
            "`git log`",
            "`expected range`",
            "0.1 minutes",
            "273.8 minutes",
            "behavior criterion",
            "split or move",
        ] {
            assert!(error.contains(needle), "{needle}: {error}");
        }
        // The check refuses it on its own, with the same text.
        assert_eq!(
            check_records_measure(&reading).unwrap_err().to_string(),
            error
        );
        // The stored-reading seam (drain, plan, steward and `athena check-reading`) refuses it too.
        match check_reading_text(&with_design(&reading), "change the reading rules") {
            ReadingCheck::Refuse(reason) => {
                assert!(reason.contains("real records"), "{reason}");
                assert!(reason.contains("\"landing time\""), "{reason}");
            }
            ReadingCheck::Accept => {
                panic!("a stored measure reading with no real-records criterion")
            }
        }
    }

    #[test]
    fn records_measure_refusal_names_what_is_missing() {
        let landing = "crates/athena-core/src/status.rs";
        // Real records and a source, no range.
        let no_range = measure_reading(
            LANDING_TIME_SUMMARY,
            &[landing],
            "Run `cargo run -p athena-cli -- status --snapshot evidence`: on the real records the landing \
             time matches the commit times in history.json",
        );
        let error = check_records_measure(&no_range).unwrap_err().to_string();
        assert!(
            error.contains("missing from the nearest behavior criterion: an `expected range`"),
            "{error}"
        );
        // Real records and a range, no independent source.
        let no_source = measure_reading(
            LANDING_TIME_SUMMARY,
            &[landing],
            "Run `cargo run -p athena-cli -- status --snapshot evidence`: on the real records the landing \
             time is within the expected range of 2 to 10 minutes",
        );
        let error = check_records_measure(&no_source).unwrap_err().to_string();
        assert!(
            error.contains("missing from the nearest behavior criterion: an independent source"),
            "{error}"
        );
        // A source and a range, no real records.
        let no_real = measure_reading(
            LANDING_TIME_SUMMARY,
            &[landing],
            "Run `cargo test`: the landing time is within the expected range taken from the commit times in \
             history.json",
        );
        let error = check_records_measure(&no_real).unwrap_err().to_string();
        assert!(
            error.contains("missing from the nearest behavior criterion: `real records`"),
            "{error}"
        );
        // The three parts in a content criterion only: a content criterion runs nothing.
        let mut content = measure_reading(LANDING_TIME_SUMMARY, &[landing], FIXTURE_ONLY_CRITERION);
        add_content(&mut content, REAL_RECORDS_CRITERION);
        assert_eq!(gate_refuser(&content), Some("check_records_measure"));
        // The parts spread over several behavior criteria are not one criterion.
        let mut spread = measure_reading(LANDING_TIME_SUMMARY, &[landing], FIXTURE_ONLY_CRITERION);
        add_behavior(
            &mut spread,
            "Run `status` on the real records and show the landing time",
        );
        add_behavior(
            &mut spread,
            "The landing time is within the expected range from the commit times in history.json",
        );
        assert_eq!(gate_refuser(&spread), Some("check_records_measure"));
    }

    #[test]
    fn records_measure_words_and_paths_each_trigger_the_rule() {
        let status = "crates/athena-core/src/status.rs";
        let throughput = "crates/athena-core/src/status/throughput.rs";
        let planning = "crates/athena-core/src/status/planning.rs";
        // Each word in the summary, in the spellings a reader writes.
        for summary in [
            "Fix the landing-time shown by status",
            "Fix the landing_time shown by status",
            "Fix the Landing Times shown by status",
            "Report per-rule reading refusal counts in status",
            "Lower the refusal rates of the reading rules",
            "Show wasted work in the status output",
            "Count the time spent per job",
            "Count the unlanded time spent",
            "Round time_spent to whole minutes",
            "Add a line to the throughput review",
            "Change the throughput figures",
            "Change the keep rule",
            "Lower the planning cost",
            "Fix the check durations",
            "Fix the check_duration column",
        ] {
            let reading = measure_reading(summary, &[status], FIXTURE_ONLY_CRITERION);
            assert_eq!(
                gate_refuser(&reading),
                Some("check_records_measure"),
                "{summary}"
            );
        }
        // A word in a behavior criterion or a content criterion triggers it too.
        let in_criterion = measure_reading(
            "Change the status header",
            &[status],
            "Run `cargo test`: the keep rule keeps jobs that landed",
        );
        assert_eq!(gate_refuser(&in_criterion), Some("check_records_measure"));
        let mut in_content =
            measure_reading("Change the status header", &[status], "Run `cargo test`");
        add_content(&mut in_content, "The doc names the check durations");
        assert_eq!(gate_refuser(&in_content), Some("check_records_measure"));
        // The two modules that compute the figures trigger it with no measure word.
        for path in [throughput, planning] {
            let reading =
                measure_reading("Tidy the module", &[path], "Run `cargo test`: it builds");
            let error = check_records_measure(&reading).unwrap_err().to_string();
            assert!(error.contains(path), "{error}");
            assert_eq!(gate_refuser(&reading), Some("check_records_measure"));
        }
        // A split of a figure module with no behavior change still needs the criterion.
        let split = measure_reading(
            "Split the module into submodules; this moves code with no behavior change",
            &[throughput],
            "Run `cargo test -p athena-core status::`: the existing tests pass",
        );
        assert_eq!(gate_refuser(&split), Some("check_records_measure"));
        // A reading that names a measure only to say it is unchanged is still refused.
        let unchanged = measure_reading(
            "Change the status header; the landing time is unchanged",
            &[status],
            "Run `cargo test`: the header is bold",
        );
        assert_eq!(gate_refuser(&unchanged), Some("check_records_measure"));
    }

    #[test]
    fn records_measure_with_a_real_records_criterion_is_accepted() {
        // Criterion 4: the same reading, once a behavior criterion names real records, an independent source
        // and an expected range. `history.json` needs no `Environment:` assumption.
        let landing = "crates/athena-core/src/status.rs";
        let mut reading = measure_reading(LANDING_TIME_SUMMARY, &[landing], FIXTURE_ONLY_CRITERION);
        assert_eq!(gate_refuser(&reading), Some("check_records_measure"));
        add_behavior(&mut reading, REAL_RECORDS_CRITERION);
        assert_eq!(gate_refuser(&reading), None);
        accept(&reading).unwrap();
        assert_eq!(
            check_reading_text(&with_design(&reading), "change the reading rules"),
            ReadingCheck::Accept
        );
        // The example in the refusal text satisfies the check it comes from.
        let refused = measure_reading(LANDING_TIME_SUMMARY, &[landing], FIXTURE_ONLY_CRITERION);
        let error = check_records_measure(&refused).unwrap_err().to_string();
        let example = error
            .split("for example: \"")
            .nth(1)
            .and_then(|rest| rest.split("\". ").next())
            .unwrap();
        let mut with_example = refused.clone();
        add_behavior(&mut with_example, example);
        assert_eq!(gate_refuser(&with_example), None, "{example}");
        // Each independent source is accepted.
        for source in [
            "history.json",
            "the commit times",
            "the recorded check durations",
            "the recorded durations",
            "the landing record timestamps",
        ] {
            let mut sourced = refused.clone();
            add_behavior(
                &mut sourced,
                &format!(
                    "Run `status --snapshot evidence`: on the real records the landing time is within the \
                     expected range taken from {source}"
                ),
            );
            assert_eq!(gate_refuser(&sourced), None, "{source}");
        }
        // `git log` is accepted with the `Environment:` statement that reading it asks for.
        let mut git_log = refused.clone();
        add_behavior(
            &mut git_log,
            "Run `status --snapshot evidence`: on the real records the landing time is within the expected \
             range taken from git log commit times",
        );
        git_log.assumptions.push(
            "Environment: the verifier reads the git log and the records in the snapshot, with read \
             permission only; history is reached through the snapshot files"
                .into(),
        );
        assert_eq!(gate_refuser(&git_log), None);
        // The path trigger is met by the same criterion.
        let mut by_path = measure_reading(
            "Tidy the module",
            &["crates/athena-core/src/status/throughput.rs"],
            "Run `cargo test`: it builds",
        );
        assert_eq!(gate_refuser(&by_path), Some("check_records_measure"));
        add_behavior(&mut by_path, REAL_RECORDS_CRITERION);
        assert_eq!(gate_refuser(&by_path), None);
        // Spellings of the parts: hyphens, case and wrapped lines.
        let mut spelled = refused;
        add_behavior(
            &mut spelled,
            "Run `status --snapshot evidence`: on the Real-Records the landing time is within the\n  Expected \
             Range taken from the commit times in history.json",
        );
        assert_eq!(gate_refuser(&spelled), None);
    }

    #[test]
    fn records_measure_leaves_unrelated_readings_alone() {
        let status = "crates/athena-core/src/status.rs";
        let neutral = "Run `cargo test`: the new unit test passes";
        let mut cases: Vec<(&str, Reading)> = Vec::new();
        cases.push((
            "default reading",
            code_reading(&[SURVEY], &[MODIFY_READING]),
        ));
        let mut docs = measure_reading(
            LANDING_TIME_SUMMARY,
            &["docs/guide.md"],
            FIXTURE_ONLY_CRITERION,
        );
        docs.class_of_work = "docs".into();
        cases.push(("docs naming landing time", docs));
        let mut tests_class = measure_reading("Add a test", &[status], neutral);
        tests_class.class_of_work = "tests".into();
        cases.push(("tests class without a measure", tests_class));
        let mut declined = measure_reading(LANDING_TIME_SUMMARY, &[status], FIXTURE_ONLY_CRITERION);
        declined.capability = None;
        declined.decline_reason = Some("no capability fits".into());
        cases.push(("decline", declined));
        let mut asking = measure_reading(LANDING_TIME_SUMMARY, &[status], FIXTURE_ONLY_CRITERION);
        asking.question = Some("which file?".into());
        cases.push(("question", asking));
        let mut content = code_reading(&[SURVEY], &[MODIFY_READING]);
        content.summary = LANDING_TIME_SUMMARY.into();
        content.acceptance_criteria = vec![Criterion {
            text: "The doc names the landing time".into(),
            kind: CriterionKind::Content,
        }];
        cases.push(("content criteria only", content));
        cases.push((
            "drain throughput of the CI queue",
            measure_reading(
                "Raise the throughput of the drain queue",
                &["crates/athena-cli/src/drain.rs"],
                "Run `cargo test -p athena-cli --test drain`: two jobs drain in one pass",
            ),
        ));
        cases.push((
            "time spent waiting for CI",
            measure_reading(
                "Reduce time spent waiting for CI",
                &["crates/athena-core/src/land.rs"],
                neutral,
            ),
        ));
        cases.push((
            "record types module without a measure word",
            measure_reading(
                "Add a field to the record type",
                &["crates/athena-data/src/records/throughput.rs"],
                neutral,
            ),
        ));
        cases.push(("deleted figure module", {
            let mut deleted = measure_reading("Remove the unused file", &[], neutral);
            deleted.deliverables.push(Deliverable {
                path: "crates/athena-core/src/status/planning.rs".into(),
                action: DeliverableAction::Delete,
                p: 0.9,
            });
            deleted
        }));
        // The declaration a refusal rule carries by design is not a measure change, even when wrapped.
        cases.push((
            "wrapped declaration",
            measure_reading(
                "Refuse more readings",
                &[status],
                "Run `cargo test`: the check refuses more readings and the higher\n   refusal  rate is\n intended",
            ),
        ));
        cases.push((
            "declared share",
            measure_reading(
                "Refuse more readings",
                &[status],
                "Run `cargo test`: expected 3% of newly refused readings, a higher refusal rate than before",
            ),
        ));
        for (label, reading) in &cases {
            check_records_measure(reading).unwrap_or_else(|e| panic!("{label}: {e}"));
            assert_ne!(
                gate_refuser(reading),
                Some("check_records_measure"),
                "{label}"
            );
        }
        // The identifier is kept apart from the English phrase: only a hyphen reads as a space.
        assert!(!has_phrase("time spent waiting", "time_spent"));
        assert!(has_phrase("round time_spent now", "time_spent"));
    }

    #[test]
    fn records_measure_has_one_step_line() {
        let source = reading_source();
        let body = source
            .split("pub fn refusing_check(")
            .nth(1)
            .and_then(|rest| rest.split("\n}\n").next())
            .unwrap();
        let steps: Vec<&str> = body
            .lines()
            .map(str::trim)
            .filter(|l| l.starts_with("step!(check_"))
            .collect();
        let line = "step!(check_records_measure(reading));";
        assert_eq!(steps.iter().filter(|l| **l == line).count(), 1, "{steps:?}");
        let history = steps
            .iter()
            .position(|l| *l == "step!(check_history_fixture(reading));")
            .unwrap();
        assert_eq!(steps[history + 1], line);
        let acceptance_doc = source
            .split("pub fn check_reading_acceptance(")
            .next()
            .unwrap();
        assert!(acceptance_doc.contains("[`check_records_measure`]"));
    }

    /// A change reading as the gate sees it in production: `CHANGELOG.md` is always a deliverable.
    fn limited(deliverables: &[&str], criterion: &str) -> Reading {
        let mut paths = deliverables.to_vec();
        if !paths.contains(&"CHANGELOG.md") {
            paths.push("CHANGELOG.md");
        }
        governed_reading(&paths, &[criterion], &[])
    }

    #[test]
    fn scope_limit_changelog() {
        // A limit that does not let the job write CHANGELOG.md is refused, with or without the deliverable.
        let bare_list = governed_reading(&["docs/x.md"], &["Only docs/x.md may change"], &[]);
        assert_eq!(gate_refuser(&bare_list), Some("check_scope_limit"));
        for criterion in [
            "Only docs/x.md may change",
            "Only docs/x.md may change; CHANGELOG.md is not changed",
            "Only docs/x.md may change and CHANGELOG.md stays as it is",
            "No file outside docs/x.md changed",
            "Test-only change",
            "Only the docs may change",
            "Only the deliverable src/a.rs may change",
        ] {
            let reading = limited(&["docs/x.md"], criterion);
            assert_eq!(
                gate_refuser(&reading),
                Some("check_scope_limit"),
                "should refuse: {criterion}"
            );
        }
        for criterion in [
            "Only docs/x.md and CHANGELOG.md may change",
            "Only docs/x.md and CHANGELOG.md may change, keeping one entry",
            "Only docs/x.md and `CHANGELOG.md` may change; src is not changed",
            "No file outside docs/x.md and CHANGELOG.md changed",
            "Only the deliverables may change",
            "Test-only change that also edits CHANGELOG.md",
            // A bare form names no list of allowed files.
            "No other file is changed",
            "No other file is changed beyond formatting",
            "Only maybe tidy things; no file outside docs/x.md and CHANGELOG.md changed",
            "Production code is unchanged",
            // Not a limit at all.
            "docs/x.md explains the check",
        ] {
            let reading = limited(&["docs/x.md"], criterion);
            assert_eq!(gate_refuser(&reading), None, "should accept: {criterion}");
        }
        // The limit is read from criteria only, as verification reads it, so an out-of-scope item is no limit.
        let reading = governed_reading(
            &["docs/x.md", "CHANGELOG.md"],
            &["docs/x.md explains the check"],
            &["Only docs/x.md may change"],
        );
        assert_eq!(gate_refuser(&reading), None);
        // A reading with a question or another capability has no change to scope.
        let mut reading = limited(&["docs/x.md"], "Only docs/x.md may change");
        reading.question = Some("Which file?".into());
        assert!(check_scope_limit(&reading).is_ok());
        let mut reading = limited(&["docs/x.md"], "Only docs/x.md may change");
        reading.capability = Some("research".into());
        assert!(check_scope_limit(&reading).is_ok());
    }

    #[test]
    fn scope_limit_error_names_the_file_the_reason_and_the_repair() {
        let reading = limited(&["docs/x.md"], "Only docs/x.md may change");
        let error = check_reading_acceptance(&reading, "change the reading rules", None, &[], None)
            .unwrap_err()
            .to_string();
        for needle in [
            "CHANGELOG.md",
            "Athena always adds it",
            "the deliverables",
            "verification fails a diff outside it",
        ] {
            assert!(
                error.contains(needle),
                "the error lacks {needle:?}: {error}"
            );
        }
    }

    #[test]
    fn scope_limit_refusal_reaches_a_stored_reading_and_the_repair_clears_it() {
        // `queued_work` drops a Proposed job whose latest stored reading `check_reading_text` no longer accepts.
        let request = "change the reading rules";
        let reading = limited(&["docs/x.md"], "Only docs/x.md may change");
        match check_reading_text(&reading, request) {
            ReadingCheck::Refuse(reason) => assert!(reason.contains("CHANGELOG.md"), "{reason}"),
            ReadingCheck::Accept => panic!("a limit without CHANGELOG.md was accepted"),
        }
        let repaired = limited(&["docs/x.md"], "Only docs/x.md and CHANGELOG.md may change");
        assert_eq!(check_reading_text(&repaired, request), ReadingCheck::Accept);
    }

    /// A limited reading with these deliverables and, when given, a `Consumers:` assumption.
    fn config_reading(deliverables: &[&str], consumers: Option<&str>) -> Reading {
        let mut reading = limited(
            deliverables,
            "Only athena.toml, defaults.toml, docs/x.md and CHANGELOG.md may change",
        );
        reading.assumptions = consumers.iter().map(|c| c.to_string()).collect();
        reading
    }

    #[test]
    fn scope_limit_config_consumers() {
        for config in ["athena.toml", "defaults.toml"] {
            let base = [config, "docs/x.md"];
            let with = |extra: &[&'static str]| -> Vec<&'static str> {
                base.iter().chain(extra).copied().collect::<Vec<_>>()
            };
            let consumer = "crates/athena-cli/tests/common/mod.rs";
            let refused: Vec<(Vec<&str>, Option<String>)> = vec![
                (with(&[]), None),
                // An unrelated test deliverable is no answer.
                (with(&["crates/athena-core/tests/refusal_corpus.rs"]), None),
                (
                    with(&[]),
                    Some(format!("Consumers: {consumer} loads {config}")),
                ),
                (
                    with(&[]),
                    Some(format!(
                        "Consumers: {consumer} must be updated; others unchanged"
                    )),
                ),
                (with(&[]), Some("Consumers: crates/a.rs unchanged".into())),
                (
                    with(&["crates/a.rs"]),
                    Some("Consumers: crates/a.rs (deliverable), xtask/tests/policy.rs".into()),
                ),
                (with(&[]), Some("Consumers: they load it".into())),
                (with(&[]), Some(format!("Consumers: {config}"))),
            ];
            for (deliverables, consumers) in refused {
                let reading = config_reading(&deliverables, consumers.as_deref());
                assert_eq!(
                    gate_refuser(&reading),
                    Some("check_scope_limit"),
                    "should refuse {deliverables:?} with {consumers:?}"
                );
            }
            let accepted: Vec<(Vec<&str>, String)> = vec![
                (
                    with(&[consumer]),
                    format!("Consumers: {consumer} loads {config}"),
                ),
                (
                    with(&["xtask/tests/policy.rs"]),
                    "Consumers: xtask/tests/policy.rs".into(),
                ),
                (
                    with(&["crates/athena-cli/tests/"]),
                    "Consumers: crates/athena-cli/tests/lifecycle.rs".into(),
                ),
                (
                    with(&["xtask/tests/policy.rs"]),
                    "consumers: `./xtask/tests/policy.rs`, and tests that load it.".into(),
                ),
                (
                    with(&[]),
                    format!("Consumers: tests that load {config} unchanged"),
                ),
                (
                    with(&[]),
                    format!("Consumers: none found, searched the repo for {config} reads"),
                ),
                (with(&[]), "Consumers: no change to any test".into()),
            ];
            for (deliverables, consumers) in accepted {
                let reading = config_reading(&deliverables, Some(&consumers));
                assert_eq!(
                    gate_refuser(&reading),
                    None,
                    "should accept {deliverables:?} with {consumers:?}"
                );
            }
        }
        // The error names the config, the label and the repair.
        let reading = config_reading(&["athena.toml", "docs/x.md"], None);
        let error = check_scope_limit(&reading).unwrap_err().to_string();
        for needle in ["athena.toml", "Consumers:", "none found", "unchanged"] {
            assert!(
                error.contains(needle),
                "the error lacks {needle:?}: {error}"
            );
        }
    }

    #[test]
    fn scope_limit_unaffected_readings() {
        // No limit: a config deliverable asks for nothing.
        let reading = governed_reading(
            &["athena.toml", "CHANGELOG.md"],
            &["athena.toml has the key"],
            &[],
        );
        assert_eq!(gate_refuser(&reading), None);
        // A limit that names CHANGELOG.md and no config deliverable asks for no Consumers line.
        let reading = limited(&["docs/x.md"], "Only docs/x.md and CHANGELOG.md may change");
        assert_eq!(gate_refuser(&reading), None);
        // A bare limit without a config deliverable asks for nothing.
        let reading = limited(&["docs/x.md"], "No other file is changed");
        assert_eq!(gate_refuser(&reading), None);
        // A bare limit with a config deliverable needs the Consumers line, not the CHANGELOG.md words.
        let mut reading = governed_reading(
            &["defaults.toml", "CHANGELOG.md"],
            &["defaults.toml has the entry", "No other file is changed"],
            &[],
        );
        assert_eq!(gate_refuser(&reading), Some("check_scope_limit"));
        reading.assumptions = vec!["Consumers: none found".into()];
        assert_eq!(gate_refuser(&reading), None);
    }

    /// The request line the defaults-proposal job carries (see `accept_request` in proposals.rs).
    const PROPOSAL_REQUEST: &str = "apply defaults proposal 20261027-054151-ace0\n\n\
         Deliverable: defaults.toml (modify). Make one change to it and touch no other file.\n\
         - Add the default with id = \"x\".\n";

    #[test]
    fn scope_limit_proposal_apply_shape() {
        // The request line is still the one `accept_request` writes.
        assert!(include_str!("proposals.rs").contains(
            "Deliverable: defaults.toml (modify). Make one change to it and touch no other file."
        ));
        let mut reading = governed_reading(
            &["defaults.toml", "CHANGELOG.md"],
            &["defaults.toml has the entry", "No other file is changed"],
            &[],
        );
        assert_eq!(
            refuser_for(&reading, PROPOSAL_REQUEST),
            Some("check_scope_limit")
        );
        let error = check_reading_acceptance(&reading, PROPOSAL_REQUEST, None, &[], None)
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("Consumers:") && error.contains("none found"),
            "{error}"
        );
        for consumers in [
            "Consumers: none found",
            "Consumers: tests that load defaults.toml unchanged",
        ] {
            reading.assumptions = vec![consumers.into()];
            assert_eq!(refuser_for(&reading, PROPOSAL_REQUEST), None, "{consumers}");
        }
    }

    #[test]
    fn scope_limit_has_one_step_line() {
        let body = reading_source()
            .split("pub fn refusing_check(")
            .nth(1)
            .and_then(|rest| rest.split("\n}\n").next())
            .unwrap();
        assert_eq!(
            body.lines()
                .filter(|l| l.trim() == "step!(check_scope_limit(reading));")
                .count(),
            1
        );
    }

    #[test]
    fn scope_limit_doc_and_changelog_name_the_cause() {
        fn collapse(text: &str) -> String {
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        }
        let source = include_str!("reading/checks.rs");
        assert!(source.contains("pub fn check_scope_limit(reading: &Reading) -> Result<()>"));
        let before = source.split("pub fn check_scope_limit(").next().unwrap();
        let mut doc_lines: Vec<&str> = before
            .lines()
            .rev()
            .take_while(|l| l.trim_start().starts_with("///"))
            .map(|l| l.trim_start().trim_start_matches('/'))
            .collect();
        doc_lines.reverse();
        let doc = collapse(&doc_lines.join(" "));
        let changelog = include_str!("../../../CHANGELOG.md");
        let unreleased = changelog
            .split("## [Unreleased]")
            .nth(1)
            .unwrap()
            .split("\n## [")
            .next()
            .unwrap();
        let mut items: Vec<String> = Vec::new();
        for line in unreleased.lines() {
            if line.starts_with("- ") {
                items.push(line.to_string());
            } else if let Some(last) = items.last_mut() {
                last.push(' ');
                last.push_str(line);
            }
        }
        let items: Vec<String> = items.iter().map(|i| collapse(i)).collect();
        let item = items
            .iter()
            .find(|i| i.contains("reading::check_scope_limit"))
            .expect("a changelog item names the check");
        for (label, text) in [("doc comment", &doc), ("changelog item", item)] {
            for needle in [
                "a scope limit written from the named files without listing default-written files and config consumers",
                "scope-criterion-too-narrow-misses-dependent-tests",
                "20261018-045437-7z81",
                "20261023-052543-9bw7",
                "higher refusal rate is intended",
                "expected 3% of newly refused readings",
                "estimate",
            ] {
                assert!(text.contains(needle), "the {label} lacks {needle:?}");
            }
        }
        for needle in [
            "ensure_changelog",
            "named in the limit",
            "Consumers:",
            "leave the drain queue",
            "run/tests.rs",
        ] {
            assert!(item.contains(needle), "the changelog item lacks {needle:?}");
        }
        let decisions = include_str!("../../../docs/decisions.md");
        let d166 = decisions.split("## D166").nth(1).expect("D166 is recorded");
        for needle in [
            "scope-criterion-too-narrow-misses-dependent-tests",
            "check_scope_limit",
            "mechanism",
        ] {
            assert!(d166.contains(needle), "D166 lacks {needle:?}");
        }
    }

    const EXISTING: &str =
        "Existing code: crates/athena-core/src/reading.rs and docs/design/jev-code-health.md";
    const EXISTING_CRITERION: &str =
        "The design has an Existing code section that reuses or extends check_code_survey";

    /// A design reading with these assumptions, criteria and deliverables.
    fn design_reading(
        assumptions: &[&str],
        criteria: &[&str],
        deliverables: &[(&str, DeliverableAction)],
    ) -> Reading {
        let mut reading = code_reading(assumptions, deliverables);
        reading.class_of_work = "design".into();
        reading.acceptance_criteria = criteria
            .iter()
            .map(|text| Criterion {
                text: (*text).into(),
                kind: CriterionKind::Content,
            })
            .collect();
        reading
    }

    const NEW_DOC: (&str, DeliverableAction) = ("docs/design/new.md", DeliverableAction::Create);

    #[test]
    fn design_existing_code_compliant_reading_is_accepted() {
        for criterion in [
            EXISTING_CRITERION,
            "The design has an existing code section that reuses check_code_survey",
            "An Existing code section extends the gate and does not rewrite it",
            "Existing code section: it is reusing the survey check",
        ] {
            let reading = design_reading(&[EXISTING], &[criterion], &[NEW_DOC]);
            accept(&reading).unwrap_or_else(|e| panic!("{criterion}: {e}"));
        }
        // The label is matched after trimming, in any case; a path may sit in backticks.
        let lower = design_reading(
            &["  existing CODE: `crates/athena-core/src/reading.rs`."],
            &[EXISTING_CRITERION],
            &[NEW_DOC],
        );
        accept(&lower).unwrap();
        // Without the assumption the same reading is refused by the new step.
        let bare = design_reading(&[], &[EXISTING_CRITERION], &[NEW_DOC]);
        let refused = refusing_check(&bare, "change the reading rules", None, &[], None).unwrap();
        assert_eq!(refused.0, "check_design_existing_code");
    }

    #[test]
    fn design_existing_code_opposite_criteria_are_refused() {
        for criterion in [
            "no survey needed",
            "the existing code in section 3 is unchanged",
            "no existing code section is needed, nothing to reuse or extend",
            "The design doesn't need an existing code section that reuses anything or extends it",
            "The design has an existing code section",
            "The design has an existing code section, one two three four five six seven eight nine ten \
             eleven twelve that reuses it",
        ] {
            let reading = design_reading(&[EXISTING], &[criterion], &[NEW_DOC]);
            let error = accept(&reading).unwrap_err().to_string();
            assert!(error.contains("Existing code"), "{criterion}: {error}");
            assert!(
                error.contains("existing code section"),
                "{criterion}: {error}"
            );
        }
        // A reuse word exactly 12 words after the phrase is accepted.
        let edge = "An existing code section one two three four five six seven eight nine ten eleven reuses it";
        accept(&design_reading(&[EXISTING], &[edge], &[NEW_DOC])).unwrap();
    }

    #[test]
    fn design_existing_code_names_a_path_the_job_does_not_create() {
        // Only the file the job creates.
        let only_created = design_reading(
            &["Existing code: docs/design/new.md"],
            &[EXISTING_CRITERION],
            &[NEW_DOC],
        );
        let error = accept(&only_created).unwrap_err().to_string();
        assert!(error.contains("`Existing code:`"), "{error}");
        assert!(error.contains("docs/design/new.md"), "{error}");
        assert!(error.contains("does not create"), "{error}");
        // The same path as a Modify deliverable is existing code.
        let modified = design_reading(
            &["Existing code: docs/design/new.md"],
            &[EXISTING_CRITERION],
            &[("docs/design/new.md", DeliverableAction::Modify)],
        );
        accept(&modified).unwrap();
        // No such assumption; a code survey does not count.
        let survey = design_reading(&[SURVEY], &[EXISTING_CRITERION], &[NEW_DOC]);
        let error = accept(&survey).unwrap_err().to_string();
        assert!(error.contains("`Existing code:`"), "{error}");
        // The label with no path, with a bare root, and with another root.
        for text in [
            "Existing code: the reading module",
            "Existing code: crates/ and docs/ were read",
            "Existing code: xtask/src/main.rs",
            "The Existing code: crates/athena-core/src/reading.rs",
        ] {
            let reading = design_reading(&[text], &[EXISTING_CRITERION], &[NEW_DOC]);
            assert!(accept(&reading).is_err(), "{text}");
        }
    }

    #[test]
    fn design_existing_code_does_not_apply_outside_its_gate() {
        let none: &[&str] = &[];
        // Other classes, with no Existing code assumption.
        for class in ["code", "tests", "docs", "config"] {
            let mut reading = code_reading(&[SURVEY], &[MODIFY_READING]);
            reading.class_of_work = class.into();
            accept(&reading).unwrap_or_else(|e| panic!("{class}: {e}"));
            check_design_existing_code(&reading).unwrap();
        }
        // A design reading with a question or a decline.
        let mut asking = design_reading(none, &["the doc names the cause"], &[NEW_DOC]);
        asking.question = Some("which file?".into());
        let mut declined = design_reading(none, &["the doc names the cause"], &[NEW_DOC]);
        declined.capability = None;
        declined.decline_reason = Some("no capability fits".into());
        for reading in [&asking, &declined] {
            accept(reading).unwrap();
            check_design_existing_code(reading).unwrap();
        }
        // The same design reading without a question or decline is refused.
        let plain = design_reading(none, &["the doc names the cause"], &[NEW_DOC]);
        assert!(check_design_existing_code(&plain).is_err());
    }

    #[test]
    fn design_existing_code_refusal_is_declared_intended() {
        let criterion = Criterion {
            text: "The change refuses more readings: expected 5% of newly refused readings".into(),
            kind: CriterionKind::Content,
        };
        assert_eq!(replay_declaration(&[criterion], &[]).share, Some(5));
        let source = reading_source();
        let doc = source
            .split("pub fn check_design_existing_code(")
            .next()
            .unwrap()
            .rsplit("\n\n")
            .next()
            .unwrap();
        assert!(
            doc.lines().all(|l| l.starts_with("///") && l != "///"),
            "the doc comment is one paragraph: {doc}"
        );
        let doc = doc
            .lines()
            .map(|l| l.trim_start_matches('/').trim())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            doc.contains("This check refuses more readings; the higher refusal rate is intended.")
        );
        for needle in [
            "no inventory of existing code",
            "20261026-043137-zc0v",
            "20261026-065046-0c1d",
            "`Existing code:`",
        ] {
            assert!(doc.contains(needle), "{needle}");
        }
    }
}
