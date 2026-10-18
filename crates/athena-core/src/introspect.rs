//! Introspection (`docs/design/defaults-loop.md` sections 3.1 to 3.4; build order step 1).
//!
//! This module holds two things that share one set of definitions:
//!
//! 1. **Shared reading code.** The reader of `records/` ([`read_records`]), the typed decision headers and
//!    their signals, the evidence-after-decision rule ([`after_start`]) and the citation validation
//!    ([`validate_citations`]). The read-only `athena backtest` and the introspection step both use these; the
//!    backtest re-exports them and keeps no copy. The `athena.introspection/1` schema is defined once, in
//!    `athena_data::records` ([`IntrospectionRecord`], [`INTROSPECTION_SCHEMA`]); [`Introspection`] is the view
//!    of its code-written fields that the backtest builds.
//! 2. **The introspection step.** After each of these events, Athena records one introspection with no request
//!    from the user: a job that ends `rejected` or `failed`, a correction of a reading (or an answer to its
//!    question), and any other intervention ([`Athena::introspect_job`], [`Athena::introspect_intervention`]).
//!    Host errors and landings are later builds. When Athena has a Jev (`Athena::jev`; the fake in tests), it
//!    is asked the failure-class and is-repeat questions in one call, in shadow ([`ask_jev`]): the answers
//!    are stored in the record's `jev` field so they can be scored later, and never change the class, the
//!    status or any outcome. A failed Jev call is stored as an error and blocks nothing.
//!
//! The reading code (section 3.3) is a function of Athena itself. It reads only the stored fields it lists:
//! `what`, `why`, `class_of_work`, `category`, job id and date of the event; for a rejected or failed job,
//! `what` also carries the job's stored `verification.json` failures: each unmet criterion's text, each blocker
//! or major finding (severity, summary, location), and, under a separate "Failed checks" heading, each failed
//! check's name and the end of its output tail, capped at [`DETAIL_CAP`] entries of [`TEXT_CAP`] and
//! [`TAIL_CAP`] characters (a verification with no failure detail says so; a job with none adds nothing).
//! This text is not redacted again; the check output was redacted when it was stored. For earlier cases it
//! reads the record
//! id, job id, date, the one-line `what`, the class and the `cause`. It never reads transcripts, diffs,
//! standard-error text or files of other jobs. It narrows earlier cases by `class_of_work` and `category`,
//! caps them at [`CASE_CAP`], and passes only those fields to the model. On return, any id that was not on a
//! list passed in is dropped and logged, not repaired.
//!
//! A failed introspection (model error, invalid answer, a cause that restates the event) is recorded as
//! failed, never blocks the job that caused it, and is not itself introspected: only the events above start
//! one.

use crate::capability::JobConfig;
use crate::commits::{NewFile, message};
use crate::jev::{Answer as JevAnswer, Evaluate, Question};
use crate::store::Records as GitRecords;
use crate::watcher::{self, D87_SIGNAL, HostErrorSignal};
use crate::{Athena, Ui};
use anyhow::{Context, Result};
use athena_agent::env::{EnvSpec, Side};
use athena_agent::{AgentInvocation, NoSink};
use athena_data::records::{
    AnyRecord, ClassStatus, FailureClass, HostError, INTROSPECTION_PROSE_FIELDS,
    INTROSPECTION_SCHEMA, InterventionCategory, IntrospectionEvidence, IntrospectionJev,
    IntrospectionRecord, IntrospectionStatus, IntrospectionTrigger, JevClassAnswer, OutcomeState,
    PreventionKind, ProposalRecord, Role,
};
use athena_data::records::{
    CheckRun, CriterionResult, Finding as VerificationFinding, Severity, Verdict, Verification,
};
use athena_data::{JobId, RecordId};
use jiff::Timestamp;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The most earlier cases passed to the model (an estimate; `defaults-loop.md` section 3.3).
pub const CASE_CAP: usize = 5;

// ---------------------------------------------------------------------------------------------------------
// The typed header (decision-sets.md section 2), for D87 and D88 only.
// ---------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    Requirement,
    Expectation,
    Design,
    Process,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Active,
    Proposed,
    Superseded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Enforcement {
    CodeCheck,
    Brief,
    ProseOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phase {
    Reading,
    Producing,
    Verifying,
    Landing,
}

/// An exact match on a stored intervention: the category and class of work are equal, and the job it names
/// landed. Nothing here is free text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InterventionMatch {
    pub category: InterventionCategory,
    pub class_of_work: &'static str,
    pub on_landed_job: bool,
}

/// A watched condition. It is run when it has an exact match: on an intervention (`exact`) or on a host-error
/// record (`host_error`, the same signal the landing watcher evaluates). A signal with neither cannot be decided
/// from the records this reader reads (history checks have no record yet), so it is listed and not run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Signal {
    pub name: &'static str,
    pub threshold_jobs: usize,
    pub exact: Option<InterventionMatch>,
    pub host_error: Option<HostErrorSignal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecisionHeader {
    pub id: &'static str,
    pub kind: Kind,
    pub classes: Vec<&'static str>,
    pub paths: Vec<&'static str>,
    pub phases: Vec<Phase>,
    pub status: Status,
    pub superseded_by: Option<&'static str>,
    pub amended_by: Vec<&'static str>,
    pub enforcement: Enforcement,
    pub assumptions: Vec<&'static str>,
    pub signals: Vec<Signal>,
}

/// The typed headers known: D87 and D88, as `decision-sets.md` section 4 gives them. D87 stays `active` with
/// `amended_by = D88`, not `superseded`: it is superseded for its landing shape only, and its two rerun fixes
/// stand (review 1, section 3). Its enforcement is `brief` (finding F4).
pub fn headers() -> Vec<DecisionHeader> {
    vec![
        DecisionHeader {
            id: "D87",
            kind: Kind::Process,
            classes: vec!["*"],
            paths: vec!["crates/athena-core/src/gh.rs"],
            phases: vec![Phase::Landing],
            status: Status::Active,
            superseded_by: None,
            amended_by: vec!["D88"],
            enforcement: Enforcement::Brief,
            assumptions: vec![
                "The repository allows merge commits (allow_merge_commit is true).",
                "A user who merges by hand follows the recommendation.",
            ],
            signals: vec![
                // The hard signal: the host refusing a merge falsifies the first assumption. Its values are
                // the watcher's own, validated against the record vocabulary when they are loaded.
                Signal {
                    name: D87_SIGNAL.name,
                    threshold_jobs: D87_SIGNAL.threshold_jobs,
                    exact: None,
                    host_error: Some(
                        watcher::load(&D87_SIGNAL).expect("D87's signal is in the vocabulary"),
                    ),
                },
                Signal {
                    name: "check-failure: history violation after a pull request merge",
                    threshold_jobs: 1,
                    exact: None,
                    host_error: None,
                },
                Signal {
                    name: "intervention: did-work-themselves on a landed branch, class code",
                    threshold_jobs: 2,
                    exact: Some(InterventionMatch {
                        category: InterventionCategory::DidWorkThemselves,
                        class_of_work: "code",
                        on_landed_job: true,
                    }),
                    host_error: None,
                },
            ],
        },
        DecisionHeader {
            id: "D88",
            kind: Kind::Process,
            classes: vec!["*"],
            paths: vec!["crates/athena-core/src/gh.rs"],
            phases: vec![Phase::Landing],
            status: Status::Active,
            superseded_by: None,
            amended_by: vec![],
            enforcement: Enforcement::Brief,
            assumptions: vec![
                "The host reports the three allow flags accurately.",
                "The squash text is accepted as given (D88 says this was not observed).",
            ],
            signals: vec![Signal {
                name: "check-failure: history violation after a squash",
                threshold_jobs: 1,
                exact: None,
                host_error: None,
            }],
        },
    ]
}

// ---------------------------------------------------------------------------------------------------------
// Reading records/ (stored fields only).
// ---------------------------------------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct OutcomeRow {
    job: String,
    state: OutcomeState,
}

/// The stored fields of an `athena.intervention/1` record that the signals use.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct InterventionRow {
    pub id: String,
    pub at: String,
    pub class_of_work: String,
    pub category: InterventionCategory,
    pub job: Option<String>,
}

/// The stored fields of an earlier introspection that section 3.3 lets the reading pass on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CaseRow {
    pub id: String,
    pub job: Option<String>,
    pub at: String,
    pub class_of_work: String,
    pub category: String,
    /// The first line of `what_happened`.
    pub what: String,
    pub failure_class: String,
    pub cause: String,
}

/// One entry of the class registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClassRow {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Default)]
pub struct Records {
    /// Every job that has an outcome record.
    pub jobs: BTreeSet<String>,
    /// Jobs with an outcome whose state is `landed` (including the retried landing of a blocked job).
    pub landed: BTreeSet<String>,
    pub interventions: Vec<InterventionRow>,
    /// Every observed `athena.host-error/1` record, oldest first.
    pub host_errors: Vec<HostError>,
    /// Host-error records derived from interventions, oldest first. Each is marked derived and cites its
    /// source intervention. They are held in memory and never written to `records/`.
    pub derived_host_errors: Vec<HostError>,
    /// Recorded introspections (a failed one has no class and is not a case), oldest first.
    pub cases: Vec<CaseRow>,
    /// The class registry, oldest first.
    pub classes: Vec<ClassRow>,
    /// For each case (by introspection id), the intervention its event recorded, if it has one.
    pub case_interventions: BTreeMap<String, String>,
    /// Every `athena.defaults-proposal/1` record, oldest first (`crate::proposals`).
    pub proposals: Vec<ProposalRecord>,
}

/// The children of a directory, sorted; a directory that does not exist has none.
fn children(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
        paths.push(entry?.path());
    }
    paths.sort();
    Ok(paths)
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("parsing {}", path.display()))
}

fn is_json(path: &Path) -> bool {
    path.extension().is_some_and(|e| e == "json")
}

/// The parsed records of the files in `records/<kind>/<month>/`, oldest path first.
pub(crate) fn month_files(root: &Path, kind: &str) -> Result<Vec<(String, Vec<u8>)>> {
    let mut files = Vec::new();
    for month in children(&root.join("records").join(kind))? {
        for file in children(&month)? {
            if !is_json(&file) {
                continue;
            }
            let bytes =
                std::fs::read(&file).with_context(|| format!("reading {}", file.display()))?;
            files.push((file.display().to_string(), bytes));
        }
    }
    Ok(files)
}

/// Read job outcomes (`records/jobs/<month>/<job>/outcome*.json`), interventions
/// (`records/interventions/<month>/<id>.json`), host errors, introspections and the class registry. A missing
/// `records/` reads as empty.
pub fn read_records(root: &Path) -> Result<Records> {
    let mut records = Records::default();
    for month in children(&root.join("records/jobs"))? {
        for job_dir in children(&month)? {
            if !job_dir.is_dir() {
                continue;
            }
            for file in children(&job_dir)? {
                let is_outcome = file
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("outcome"));
                if !is_outcome || !is_json(&file) {
                    continue;
                }
                let row: OutcomeRow = read_json(&file)?;
                if row.state == OutcomeState::Landed {
                    records.landed.insert(row.job.clone());
                }
                records.jobs.insert(row.job);
            }
        }
    }
    for month in children(&root.join("records/interventions"))? {
        for file in children(&month)? {
            if !is_json(&file) {
                continue;
            }
            records.interventions.push(read_json(&file)?);
            // The derivation reads the whole record (its `what` and `why`). A file that is not a valid
            // intervention record derives nothing; the row above is read as before.
            let bytes =
                std::fs::read(&file).with_context(|| format!("reading {}", file.display()))?;
            if let Ok(AnyRecord::Intervention(intervention)) =
                AnyRecord::parse(&file.display().to_string(), &bytes)
                && let Some(derived) = watcher::derive_host_error(&intervention)
            {
                records.derived_host_errors.push(derived);
            }
        }
    }
    records
        .interventions
        .sort_by(|a, b| (&a.at, &a.id).cmp(&(&b.at, &b.id)));
    for month in children(&root.join("records/host-errors"))? {
        for file in children(&month)? {
            if !is_json(&file) {
                continue;
            }
            let bytes =
                std::fs::read(&file).with_context(|| format!("reading {}", file.display()))?;
            let path = file.display().to_string();
            match AnyRecord::parse(&path, &bytes)? {
                // A record on disk that is marked derived is kept with the derived ones, never as observed.
                AnyRecord::HostError(error) if error.derived => {
                    records.derived_host_errors.push(error)
                }
                AnyRecord::HostError(error) => records.host_errors.push(error),
                _ => anyhow::bail!("{path}: not an athena.host-error/1 record"),
            }
        }
    }
    records
        .host_errors
        .sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
    records
        .derived_host_errors
        .sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
    for (path, bytes) in month_files(root, "introspections")? {
        match AnyRecord::parse(&path, &bytes)? {
            AnyRecord::Introspection(i) => {
                // Only a written record in a class is a case; the fields kept are the listed ones.
                if let (IntrospectionStatus::Recorded, Some(class), Some(what), Some(cause)) = (
                    i.status,
                    i.failure_class.as_ref(),
                    i.what_happened.as_deref(),
                    i.cause.as_deref(),
                ) {
                    if let Some(intervention) = &i.intervention {
                        records
                            .case_interventions
                            .insert(i.id.to_string(), intervention.to_string());
                    }
                    records.cases.push(CaseRow {
                        id: i.id.to_string(),
                        job: i.job.as_ref().map(ToString::to_string),
                        at: i.at.to_string(),
                        class_of_work: i.class_of_work.clone(),
                        category: i.category.clone(),
                        what: what.lines().next().unwrap_or("").to_string(),
                        failure_class: class.to_string(),
                        cause: cause.to_string(),
                    });
                }
            }
            _ => anyhow::bail!("{path}: not an athena.introspection/1 record"),
        }
    }
    records
        .cases
        .sort_by(|a, b| (&a.at, &a.id).cmp(&(&b.at, &b.id)));
    for (path, bytes) in month_files(root, "failure-classes")? {
        match AnyRecord::parse(&path, &bytes)? {
            AnyRecord::FailureClass(c) => records.classes.push(ClassRow {
                id: c.id.to_string(),
                name: c.name,
                description: c.description,
            }),
            _ => anyhow::bail!("{path}: not an athena.failure-class/1 record"),
        }
    }
    records.classes.sort_by(|a, b| a.id.cmp(&b.id));
    for (path, bytes) in month_files(root, "defaults-proposals")? {
        match AnyRecord::parse(&path, &bytes)? {
            AnyRecord::DefaultsProposal(p) => records.proposals.push(*p),
            _ => anyhow::bail!("{path}: not an athena.defaults-proposal/1 record"),
        }
    }
    records
        .proposals
        .sort_by(|a, b| (a.at, &a.record).cmp(&(b.at, &b.record)));
    Ok(records)
}

// ---------------------------------------------------------------------------------------------------------
// The signal.
// ---------------------------------------------------------------------------------------------------------

/// One run of one exact-match signal over the records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignalRun {
    pub decision: String,
    pub signal: String,
    pub threshold_jobs: usize,
    /// The matching interventions, earliest first.
    pub events: Vec<InterventionRow>,
    /// The observed `athena.host-error/1` records a host-error signal matched, oldest first.
    pub host_error_records: Vec<String>,
    /// The derived host-error records the signal matched, oldest first. The client did not observe these.
    pub derived_records: Vec<DerivedCite>,
    pub jobs: BTreeSet<String>,
    pub fired: bool,
}

/// A derived host-error record that fired a signal, with the intervention it was derived from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DerivedCite {
    pub record: String,
    pub source_intervention: String,
}

impl DerivedCite {
    pub fn of(error: &HostError) -> Self {
        DerivedCite {
            record: error.id.to_string(),
            source_intervention: error
                .source_intervention
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
        }
    }

    /// How a decision's review cites the record: as derived, and through it the source intervention.
    pub fn citation(&self) -> String {
        format!(
            "{} (derived, not observed; from intervention {})",
            self.record, self.source_intervention
        )
    }
}

pub fn matches(m: &InterventionMatch, row: &InterventionRow, records: &Records) -> bool {
    row.category == m.category
        && row.class_of_work == m.class_of_work
        && match &row.job {
            Some(job) => !m.on_landed_job || records.landed.contains(job),
            None => false,
        }
}

/// The time each decision started: the first commit that introduced its id heading in `docs/decisions.md`. A
/// decision with no start counts every record.
pub type Starts = BTreeMap<String, Timestamp>;

/// The evidence-after-decision rule: true when a record at `at` is later than the start of `decision`. A
/// decision with no start counts every record; a record with no usable time is not later than any start.
pub fn after_start(starts: &Starts, decision: &str, at: Option<Timestamp>) -> bool {
    match starts.get(decision) {
        None => true,
        Some(start) => at.is_some_and(|at| at > *start),
    }
}

pub fn row_at(row: &InterventionRow) -> Option<Timestamp> {
    row.at.parse().ok()
}

/// The time of the first commit that introduced the id heading `## <id> ` in `docs/decisions.md` of the
/// repository at `root`, from `git log -S`. The heading is found by its id and the space after it, not by its
/// full text, so a heading that was renamed later still dates from the commit that first added the id. `None`
/// when no commit adds it.
pub fn decision_start(root: &Path, id: &str) -> Result<Option<Timestamp>> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "--reverse", "--format=%cI", "-S"])
        .arg(format!("## {id} "))
        .args(["--", "docs/decisions.md"])
        .output()
        .context("running git log to date a decision")?;
    if !output.status.success() {
        anyhow::bail!(
            "git log failed while dating {id}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let Some(first) = text.lines().find(|l| !l.trim().is_empty()) else {
        return Ok(None);
    };
    let at = first
        .trim()
        .parse()
        .with_context(|| format!("reading the commit time {first:?} of {id}"))?;
    Ok(Some(at))
}

/// The start of every decision with a typed header, from the history of the repository at `root`. A decision
/// whose heading no commit adds is an error: its records cannot be dated.
pub fn starts_from_history(root: &Path) -> Result<Starts> {
    let mut starts = Starts::new();
    for header in headers() {
        let start = decision_start(root, header.id)?.with_context(|| {
            format!(
                "no commit in git history adds the heading `## {} `",
                header.id
            )
        })?;
        starts.insert(header.id.to_string(), start);
    }
    Ok(starts)
}

pub fn run_signals(
    headers: &[DecisionHeader],
    records: &Records,
    starts: &Starts,
) -> Vec<SignalRun> {
    let mut runs = Vec::new();
    for header in headers {
        let later = |at: Option<Timestamp>| after_start(starts, header.id, at);
        for signal in &header.signals {
            if let Some(host_error) = &signal.host_error {
                let matched: Vec<&HostError> = records
                    .host_errors
                    .iter()
                    .filter(|e| host_error.matches(e) && later(Some(e.at)))
                    .collect();
                let derived: Vec<&HostError> = records
                    .derived_host_errors
                    .iter()
                    .filter(|e| host_error.matches(e) && later(Some(e.at)))
                    .collect();
                let jobs: BTreeSet<String> = matched
                    .iter()
                    .chain(&derived)
                    .map(|e| e.job.to_string())
                    .collect();
                runs.push(SignalRun {
                    decision: header.id.to_string(),
                    signal: signal.name.to_string(),
                    threshold_jobs: signal.threshold_jobs,
                    fired: !jobs.is_empty() && jobs.len() >= signal.threshold_jobs,
                    events: Vec::new(),
                    host_error_records: matched.iter().map(|e| e.id.to_string()).collect(),
                    derived_records: derived.iter().map(|e| DerivedCite::of(e)).collect(),
                    jobs,
                });
                continue;
            }
            let Some(exact) = &signal.exact else { continue };
            let events: Vec<InterventionRow> = records
                .interventions
                .iter()
                .filter(|row| matches(exact, row, records) && later(row_at(row)))
                .cloned()
                .collect();
            let jobs: BTreeSet<String> = events.iter().filter_map(|e| e.job.clone()).collect();
            runs.push(SignalRun {
                decision: header.id.to_string(),
                signal: signal.name.to_string(),
                threshold_jobs: signal.threshold_jobs,
                fired: jobs.len() >= signal.threshold_jobs,
                events,
                host_error_records: Vec::new(),
                derived_records: Vec::new(),
                jobs,
            });
        }
    }
    runs
}

// ---------------------------------------------------------------------------------------------------------
// The code-written view of the introspection record, and the citation validation.
// ---------------------------------------------------------------------------------------------------------

/// The code-written fields of an `athena.introspection/1` record (`defaults-loop.md` section 3.2), as the
/// backtest builds them. The stored record is [`IntrospectionRecord`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Introspection {
    pub schema: &'static str,
    /// The intervention of the triggering event.
    pub intervention: String,
    pub job: String,
    pub class_of_work: String,
    pub at: String,
    /// Ids of earlier events of the same signal, by code.
    pub earlier_cases: Vec<String>,
    /// Typed decisions whose signal the event matched, by code.
    pub decisions_matched: Vec<String>,
    /// The fields a language model or a typed decision writes; not written by the backtest.
    pub pending_fields: Vec<&'static str>,
}

impl Introspection {
    /// The code-written fields of an event, with the model-written ones listed as pending.
    pub fn code_fields(
        intervention: &InterventionRow,
        job: String,
        earlier_cases: Vec<String>,
        decisions_matched: Vec<String>,
    ) -> Introspection {
        Introspection {
            schema: INTROSPECTION_SCHEMA,
            intervention: intervention.id.clone(),
            job,
            class_of_work: intervention.class_of_work.clone(),
            at: intervention.at.clone(),
            earlier_cases,
            decisions_matched,
            pending_fields: INTROSPECTION_PROSE_FIELDS.to_vec(),
        }
    }
}

/// A pass condition that did not hold, with the reason. A finding is reported as it is; nothing is retried or
/// relaxed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    /// The review's condition: `a`, `b`, `c`, `d` or `e`.
    pub condition: &'static str,
    pub reason: String,
}

/// Validates every record a signal run cites, against the signal's own fields and the decision's time window
/// (condition `e`). A cited record is valid when it exists in `records`, matches the signal (a host error: the
/// operation, phase and status; an intervention: the category, class of work and landed job), and is later than
/// the start of the decision. A derived record must also cite a source intervention that exists and is later
/// than the start. Each invalid citation is a finding.
pub fn validate_citations(
    headers: &[DecisionHeader],
    runs: &[SignalRun],
    records: &Records,
    starts: &Starts,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut invalid = |decision: &str, record: &str, why: &str| {
        findings.push(Finding {
            condition: "e",
            reason: format!("{decision} cites {record}, which is not valid: {why}"),
        });
    };
    for run in runs {
        let signal = headers
            .iter()
            .filter(|h| h.id == run.decision)
            .flat_map(|h| &h.signals)
            .find(|s| s.name == run.signal);
        let Some(signal) = signal else {
            invalid(&run.decision, &run.signal, "no such signal on the decision");
            continue;
        };
        let later = |at: Option<Timestamp>| after_start(starts, &run.decision, at);
        for id in &run.host_error_records {
            let found = records.host_errors.iter().find(|e| e.id.as_str() == id);
            match (found, &signal.host_error) {
                (None, _) => invalid(
                    &run.decision,
                    id,
                    "no observed host-error record has this id",
                ),
                (Some(_), None) => {
                    invalid(&run.decision, id, "the signal is not a host-error signal")
                }
                (Some(e), Some(s)) if !s.matches(e) => invalid(
                    &run.decision,
                    id,
                    "it does not match the signal's operation, phase and status",
                ),
                (Some(e), Some(_)) if !later(Some(e.at)) => invalid(
                    &run.decision,
                    id,
                    "it is not later than the decision's start",
                ),
                _ => {}
            }
        }
        for cite in &run.derived_records {
            let id = cite.record.as_str();
            let found = records
                .derived_host_errors
                .iter()
                .find(|e| e.id.as_str() == id);
            let source = records
                .interventions
                .iter()
                .find(|i| i.id == cite.source_intervention);
            match (found, &signal.host_error) {
                (None, _) => invalid(
                    &run.decision,
                    id,
                    "no derived host-error record has this id",
                ),
                (Some(_), None) => {
                    invalid(&run.decision, id, "the signal is not a host-error signal")
                }
                (Some(e), Some(s)) if !s.matches(e) => invalid(
                    &run.decision,
                    id,
                    "it does not match the signal's operation, phase and status",
                ),
                (Some(e), Some(_)) if !later(Some(e.at)) => invalid(
                    &run.decision,
                    id,
                    "it is not later than the decision's start",
                ),
                _ => match source {
                    None => invalid(&run.decision, id, "its source intervention does not exist"),
                    Some(row) if !later(row_at(row)) => invalid(
                        &run.decision,
                        id,
                        "its source intervention is not later than the decision's start",
                    ),
                    Some(_) => {}
                },
            }
        }
        for event in &run.events {
            let found = records.interventions.iter().find(|i| i.id == event.id);
            match (found, &signal.exact) {
                (None, _) => invalid(&run.decision, &event.id, "no intervention has this id"),
                (Some(_), None) => invalid(
                    &run.decision,
                    &event.id,
                    "the signal has no intervention match",
                ),
                (Some(row), Some(m)) if !matches(m, row, records) => invalid(
                    &run.decision,
                    &event.id,
                    "it does not match the signal's category, class of work and landed job",
                ),
                (Some(row), Some(_)) if !later(row_at(row)) => invalid(
                    &run.decision,
                    &event.id,
                    "it is not later than the decision's start",
                ),
                _ => {}
            }
        }
    }
    findings
}

// ---------------------------------------------------------------------------------------------------------
// The introspection step.
// ---------------------------------------------------------------------------------------------------------

/// The stored fields of the event that started an introspection.
#[derive(Debug, Clone)]
pub struct Event {
    pub trigger: IntrospectionTrigger,
    pub job: Option<JobId>,
    pub intervention: Option<RecordId>,
    pub class_of_work: String,
    /// The intervention category, or `rejected` or `failed`.
    pub category: String,
    pub what: String,
    pub why: String,
    pub at: Timestamp,
}

/// Decisions whose fired, valid signal cites the intervention `intervention`. The citation must be later than
/// the decision's start (the evidence-after-decision rule, through [`validate_citations`]). When the decisions
/// cannot be dated, none match.
pub fn decisions_matched(records: &Records, starts: &Starts, intervention: &str) -> Vec<String> {
    let headers = headers();
    let mut matched = BTreeSet::new();
    for run in run_signals(&headers, records, starts)
        .into_iter()
        .filter(|r| r.fired)
    {
        let cites = run.events.iter().any(|e| e.id == intervention)
            || run
                .derived_records
                .iter()
                .any(|d| d.source_intervention == intervention);
        if cites
            && validate_citations(&headers, std::slice::from_ref(&run), records, starts).is_empty()
        {
            matched.insert(run.decision);
        }
    }
    matched.into_iter().collect()
}

/// Earlier cases for an event: narrowed by `class_of_work` and `category`, newest first, at most
/// [`CASE_CAP`]. Only listed stored fields are in a [`CaseRow`].
pub fn narrow_cases<'a>(records: &'a Records, event: &Event) -> Vec<&'a CaseRow> {
    let mut cases: Vec<&CaseRow> = records
        .cases
        .iter()
        .filter(|c| c.class_of_work == event.class_of_work && c.category == event.category)
        .collect();
    cases.reverse();
    cases.truncate(CASE_CAP);
    cases
}

/// Lower case, letters and digits only, words joined by one space.
pub fn normalise(text: &str) -> String {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect::<Vec<_>>()
        .join(" ")
}

/// The model's answer, before code checks it.
#[derive(Debug, Deserialize)]
struct Answer {
    what_happened: String,
    failure_class: String,
    new_class: Option<NewClass>,
    earlier_cases: Vec<String>,
    decisions_matched: Vec<String>,
    cause: String,
    prevention_kind: String,
    prevention: String,
    proposed_request: String,
}

#[derive(Debug, Deserialize)]
struct NewClass {
    name: String,
    description: String,
}

/// The most unmet criteria, blocker or major findings, and failed checks each listed in `what`.
pub const DETAIL_CAP: usize = 8;
/// The most characters of one criterion, finding summary or location in `what`.
pub const TEXT_CAP: usize = 400;
/// The most characters kept from the end of one failed check's output tail.
pub const TAIL_CAP: usize = 600;

/// `text` cut to at most `cap` characters, marked when cut.
fn clip(text: &str, cap: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= cap {
        return text.to_string();
    }
    let cut: String = text.chars().take(cap).collect();
    format!("{cut}...")
}

/// The last `cap` characters of `text`, marked when the start was cut.
fn clip_tail(text: &str, cap: usize) -> String {
    let text = text.trim();
    let count = text.chars().count();
    if count <= cap {
        return text.to_string();
    }
    let kept: String = text.chars().skip(count - cap).collect();
    format!("...{kept}")
}

fn more(total: usize) -> String {
    if total > DETAIL_CAP {
        format!("- ({} more not shown)\n", total - DETAIL_CAP)
    } else {
        String::new()
    }
}

/// What a failed verification adds to an event's `what`: each unmet criterion, each blocker or major finding,
/// and, under their own heading, each failed check. Counts and text are capped. Nothing is added when the
/// verification passed. The text comes from the stored `verification.json` only; check output was redacted
/// when it was stored, and this text is not redacted again.
fn failure_details(v: &Verification) -> String {
    if v.final_verdict == Verdict::Pass {
        return String::new();
    }
    let unmet: Vec<&CriterionResult> = v.criteria.iter().filter(|c| !c.met).collect();
    let findings: Vec<&VerificationFinding> = v
        .findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Blocker | Severity::Major))
        .collect();
    // A pre-existing failure was already on the base and did not cause this verdict.
    let checks: Vec<&CheckRun> = v
        .checks
        .iter()
        .filter(|c| {
            !c.passed
                && !c
                    .note
                    .as_deref()
                    .is_some_and(|n| n.starts_with(athena_data::records::PRE_EXISTING_NOTE))
        })
        .collect();
    if unmet.is_empty() && findings.is_empty() && checks.is_empty() {
        return "\n\nThe stored verification lists no unmet criterion, blocker or major finding, or \
                failed check, so the cause is not in the stored result."
            .to_string();
    }
    let mut out = String::new();
    if !unmet.is_empty() {
        out.push_str(&format!(
            "\n\nUnmet acceptance criteria ({}):\n",
            unmet.len()
        ));
        for c in unmet.iter().take(DETAIL_CAP) {
            out.push_str(&format!("- {}\n", clip(&c.criterion, TEXT_CAP)));
        }
        out.push_str(&more(unmet.len()));
    }
    if !findings.is_empty() {
        out.push_str(&format!(
            "\n\nBlocker and major findings ({}):\n",
            findings.len()
        ));
        for f in findings.iter().take(DETAIL_CAP) {
            let severity = if f.severity == Severity::Blocker {
                "blocker"
            } else {
                "major"
            };
            let location = f
                .location
                .as_deref()
                .map_or(String::new(), |l| format!(" at {}", clip(l, TEXT_CAP)));
            out.push_str(&format!(
                "- {severity}: {}{location}\n",
                clip(&f.summary, TEXT_CAP)
            ));
        }
        out.push_str(&more(findings.len()));
    }
    if !checks.is_empty() {
        out.push_str(&format!(
            "\n\nFailed checks (these are checks that ran, not acceptance criteria) ({}):\n",
            checks.len()
        ));
        for c in checks.iter().take(DETAIL_CAP) {
            out.push_str(&format!(
                "- {}: {}\n",
                clip(&c.name, TEXT_CAP),
                clip_tail(&c.output_tail, TAIL_CAP)
            ));
        }
        out.push_str(&more(checks.len()));
    }
    out.truncate(out.trim_end().len());
    out
}

const SYSTEM_PROMPT: &str = "You write one introspection for the Athena work system. You are given the \
stored fields of one event (a rejected or failed job, a correction, or an intervention), the class \
registry, and up to five earlier cases. You have no tools and cannot read anything else. Say what happened \
in plain words. For a rejected or failed job, the event may list unmet acceptance criteria, blocker and \
major findings, and failed checks; criteria and checks are different things, so do not call a failed check \
an unmet criterion. Choose the failure class: the id of a registry class that fits, or `new` with a name \
and a one-line description when none fits. List the ids of earlier cases that are in the same class, and \
the ids of listed decisions the event matched; use only ids you were given. Give the cause beyond this \
instance: why the system allowed it, which must not restate what happened. Choose the prevention kind \
(default, code-check or design-change), say exactly what would have prevented the whole class, and write a \
complete next request, ready to submit.";

fn answer_schema() -> serde_json::Value {
    let text = json!({ "type": "string" });
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["what_happened", "failure_class", "new_class", "earlier_cases", "decisions_matched",
            "cause", "prevention_kind", "prevention", "proposed_request"],
        "properties": {
            "what_happened": text,
            "failure_class": text,
            "new_class": {
                "type": ["object", "null"],
                "additionalProperties": false,
                "required": ["name", "description"],
                "properties": { "name": text, "description": text }
            },
            "earlier_cases": { "type": "array", "items": text },
            "decisions_matched": { "type": "array", "items": text },
            "cause": text,
            "prevention_kind": { "type": "string", "enum": ["default", "code-check", "design-change"] },
            "prevention": text,
            "proposed_request": text
        }
    })
}

/// What the model is given: the listed stored fields and nothing else.
fn user_prompt(
    event: &Event,
    classes: &[ClassRow],
    cases: &[&CaseRow],
    decisions: &[String],
) -> String {
    let value = json!({
        "event": {
            "trigger": event.trigger.as_str(),
            "what": event.what,
            "why": event.why,
            "class_of_work": event.class_of_work,
            "category": event.category,
            "job": event.job.as_ref().map(ToString::to_string),
            "date": event.at.to_string(),
        },
        "class_registry": classes,
        "earlier_cases": cases,
        "decisions_that_may_be_matched": decisions,
    });
    serde_json::to_string_pretty(&value).expect("a JSON value serializes")
}

/// The key of the failure-class question in the call to Jev.
pub const JEV_CLASS_KEY: &str = "failure_class";
/// The key of the is-repeat question in the call to Jev.
pub const JEV_REPEAT_KEY: &str = "is_repeat";
/// The option of the failure-class question that means no registry class fits.
pub const JEV_NEW_OPTION: &str = "new";

/// The two introspection questions: the failure class (a choice over the registry ids, plus `new`) and whether
/// the event repeats an earlier case (a boolean).
pub fn jev_questions(classes: &[ClassRow]) -> BTreeMap<String, Question> {
    let mut criteria: BTreeMap<String, String> = classes
        .iter()
        .map(|c| (c.id.clone(), format!("{}: {}", c.name, c.description)))
        .collect();
    criteria.insert(
        JEV_NEW_OPTION.to_string(),
        "None of the registry classes fits this event; it is a new class.".to_string(),
    );
    BTreeMap::from([
        (
            JEV_CLASS_KEY.to_string(),
            Question::Choice {
                instructions: "Which failure class does the event belong to? Choose the id of the \
                               registry class that fits, or `new` when none does."
                    .to_string(),
                criteria,
            },
        ),
        (
            JEV_REPEAT_KEY.to_string(),
            Question::Boolean {
                instructions:
                    "Is this event a repeat of one of the earlier cases listed, with the \
                               same underlying cause?"
                        .to_string(),
            },
        ),
    ])
}

/// Asks Jev both questions in one call over `state` (the text the model is given) and turns the reply into
/// the stored form. This never fails: a call that failed becomes a record with its error and no answers. The
/// result is stored and read by nothing that decides.
pub fn ask_jev(jev: &dyn Evaluate, state: &str, classes: &[ClassRow]) -> IntrospectionJev {
    let mut stored = IntrospectionJev {
        model: crate::jev::MODEL.to_string(),
        class: None,
        repeat_probability: None,
        input_tokens: 0,
        output_tokens: 0,
        cost_usd: None,
        error: None,
    };
    match jev.evaluate(state, &jev_questions(classes)) {
        Err(error) => stored.error = Some(error.to_string()),
        Ok(evaluation) => {
            stored.input_tokens = evaluation.usage.input_tokens;
            stored.output_tokens = evaluation.usage.output_tokens;
            stored.cost_usd = evaluation.cost_usd;
            match (
                evaluation.answers.get(JEV_CLASS_KEY),
                evaluation.answers.get(JEV_REPEAT_KEY),
            ) {
                (
                    Some(JevAnswer::Choice {
                        choice,
                        probabilities,
                        confidence,
                    }),
                    Some(JevAnswer::Boolean { probability }),
                ) => {
                    stored.class = Some(JevClassAnswer {
                        choice: choice.clone(),
                        probabilities: probabilities.clone(),
                        confidence: *confidence,
                    });
                    stored.repeat_probability = Some(*probability);
                }
                _ => stored.error = Some("Jev did not answer both questions".to_string()),
            }
        }
    }
    stored
}

impl Athena {
    /// Introspects a job that ended `rejected` or `failed`. A job in any other state is not an event. Never
    /// returns an error: a problem is a line to the user, since an introspection never blocks a job.
    pub fn introspect_job(&self, job: &JobId, ui: &dyn Ui) -> Option<IntrospectionRecord> {
        let result = self.event_of_job(job).and_then(|event| match event {
            Some(event) => self.introspect(event, ui).map(Some),
            None => Ok(None),
        });
        self.settle(result, ui)
    }

    /// Introspects an intervention (a correction, an answer, or any other). Never returns an error.
    pub fn introspect_intervention(
        &self,
        id: &RecordId,
        ui: &dyn Ui,
    ) -> Option<IntrospectionRecord> {
        let result = self
            .event_of_intervention(id)
            .and_then(|event| self.introspect(event, ui).map(Some));
        self.settle(result, ui)
    }

    fn settle(
        &self,
        result: Result<Option<IntrospectionRecord>>,
        ui: &dyn Ui,
    ) -> Option<IntrospectionRecord> {
        match result {
            Ok(record) => record,
            Err(error) => {
                ui.line(&format!("introspection could not be recorded: {error:#}"));
                None
            }
        }
    }

    fn event_of_job(&self, job: &JobId) -> Result<Option<Event>> {
        let records = GitRecords::load(&self.git, &self.head()?)?;
        let Some(job_records) = records.jobs.get(job.as_str()) else {
            return Ok(None);
        };
        let Some(outcome) = &job_records.outcome else {
            return Ok(None);
        };
        let (trigger, category) = match outcome.state {
            OutcomeState::Rejected => (IntrospectionTrigger::Rejection, "rejected"),
            OutcomeState::Failed => (IntrospectionTrigger::Failure, "failed"),
            _ => return Ok(None),
        };
        Ok(Some(Event {
            trigger,
            job: Some(job.clone()),
            intervention: None,
            class_of_work: job_records.class().unwrap_or("unknown").to_string(),
            category: category.to_string(),
            what: format!(
                "{}: {}{}",
                job_records.title(),
                outcome.reason,
                job_records
                    .verification
                    .as_ref()
                    .map(failure_details)
                    .unwrap_or_default()
            ),
            why: format!("the job ended {category}"),
            at: outcome.at,
        }))
    }

    fn event_of_intervention(&self, id: &RecordId) -> Result<Event> {
        let records = GitRecords::load(&self.git, &self.head()?)?;
        let i = records
            .interventions
            .iter()
            .find(|i| &i.id == id)
            .with_context(|| format!("no intervention {id}"))?;
        let trigger = match i.category {
            InterventionCategory::CorrectedOutput | InterventionCategory::AnsweredQuestion => {
                IntrospectionTrigger::Correction
            }
            _ => IntrospectionTrigger::Intervention,
        };
        Ok(Event {
            trigger,
            job: i.job.clone(),
            intervention: Some(i.id.clone()),
            class_of_work: i.class_of_work.clone(),
            category: i.category.as_str().to_string(),
            what: i.what.clone(),
            why: i.why.clone(),
            at: i.at,
        })
    }

    /// Reads the past records, asks the model for the prose, checks the answer, and commits the introspection
    /// (and a `proposed` class when the model named a new one). An answer that cannot be used is committed as a
    /// failed introspection. Only a failure to commit is an error.
    pub fn introspect(&self, event: Event, ui: &dyn Ui) -> Result<IntrospectionRecord> {
        let head = self.head()?;
        let root = self.git.dir().to_path_buf();
        let known = GitRecords::load(&self.git, &head)?;
        let past = read_records(&root)?;
        let cases = narrow_cases(&past, &event);
        let candidates = match &event.intervention {
            Some(id) => match starts_from_history(&root) {
                Ok(starts) => decisions_matched(&past, &starts, id.as_str()),
                Err(_) => Vec::new(),
            },
            None => Vec::new(),
        };
        let mut id = RecordId::generate(crate::now());
        while known.has_record_id(&id) {
            id = RecordId::generate(crate::now());
        }
        let config = JobConfig::at(&self.git, &head)?.config;
        let redactor = self.redactor(&config, event.job.as_ref())?;
        let mut record = IntrospectionRecord {
            id: id.clone(),
            at: crate::now(),
            trigger: event.trigger,
            job: event.job.clone(),
            intervention: event.intervention.clone(),
            class_of_work: event.class_of_work.clone(),
            category: event.category.clone(),
            what_happened: None,
            failure_class: None,
            new_class: false,
            earlier_cases: Vec::new(),
            decisions_matched: Vec::new(),
            cause: None,
            prevention_kind: None,
            prevention: None,
            proposed_request: None,
            evidence: None,
            status: IntrospectionStatus::Failed,
            failure: None,
            flags: Vec::new(),
            dropped_ids: Vec::new(),
            from_jev: false,
            jev: None,
        };
        let prompt = user_prompt(&event, &past.classes, &cases, &candidates);
        // Shadow: Jev sees the same fields the model sees. Its answers are stored beside the model's and are
        // read by nothing in this function.
        let jev_answers = self
            .jev
            .as_deref()
            .map(|jev| ask_jev(jev, &prompt, &past.classes));
        let scratch = self.state.root().join("introspect").join(id.as_str());
        std::fs::create_dir_all(&scratch)?;
        let spec = self.introspection_spec(&config, &event, &id, prompt, &scratch)?;
        let model = spec.model.clone();
        let mut new_class = None;
        match self.driver.invoke(&spec, &mut NoSink) {
            Err(error) => record.failure = Some(format!("the model could not run: {error}")),
            Ok(output) => {
                record.evidence = Some(IntrospectionEvidence {
                    model,
                    cost_usd: output.cost_usd.unwrap_or(0.0),
                    input_tokens: output.usage.input,
                    output_tokens: output.usage.output,
                });
                match output.structured.map(serde_json::from_value::<Answer>) {
                    _ if output.outcome != athena_data::records::InvocationOutcome::Success => {
                        record.failure = Some(
                            output
                                .detail
                                .unwrap_or_else(|| "the model did not complete".into()),
                        )
                    }
                    None => record.failure = Some("the model returned no answer".into()),
                    Some(Err(error)) => {
                        record.failure = Some(format!("the model's answer is invalid: {error}"))
                    }
                    Some(Ok(answer)) => {
                        let passed: Vec<String> = cases.iter().map(|c| c.id.clone()).collect();
                        new_class = self.apply_answer(
                            &mut record,
                            answer,
                            &past,
                            &passed,
                            &candidates,
                            &redactor,
                            ui,
                        );
                    }
                }
            }
        }
        let _ = std::fs::remove_dir_all(&scratch);
        record.jev = jev_answers;
        let mut files = Vec::new();
        if let Some(class) = new_class {
            files.push(NewFile::record(&AnyRecord::FailureClass(class)));
        }
        let path = AnyRecord::Introspection(Box::new(record.clone())).canonical_path();
        files.push(NewFile::record(&AnyRecord::Introspection(Box::new(
            record.clone(),
        ))));
        if record.status == IntrospectionStatus::Failed {
            ui.line(&format!(
                "introspect {} failed: {}",
                id,
                record.failure.as_deref().unwrap_or("")
            ));
        } else {
            ui.line(&format!("introspect {id} recorded"));
        }
        let subject = format!(
            "Record an introspection of a {} ({})",
            event.trigger.as_str(),
            record.status.as_str()
        );
        let body = "Why the event happened, the class it belongs to and what would have stopped the class. \
                    Written by a language model from the stored fields of the event and earlier cases. Jev, when asked, \
                    answered in shadow and decided nothing.";
        self.commit_records(&files, &message(&subject, body, &[("Athena-Record", path)]))?;
        Ok(record)
    }

    /// Checks the model's answer and fills the record. Returns the class to add when the model named a new one.
    #[allow(clippy::too_many_arguments)]
    fn apply_answer(
        &self,
        record: &mut IntrospectionRecord,
        answer: Answer,
        past: &Records,
        passed_cases: &[String],
        passed_decisions: &[String],
        redactor: &athena_gate::redact::Redactor,
        ui: &dyn Ui,
    ) -> Option<FailureClass> {
        let mut text = vec![
            answer.what_happened.trim().to_string(),
            answer.cause.trim().to_string(),
            answer.prevention.trim().to_string(),
            answer.proposed_request.trim().to_string(),
        ];
        if redactor.redact_all(&mut text).is_err() {
            record.failure = Some("the answer could not be redacted".into());
            return None;
        }
        let Some(kind) = PreventionKind::parse(&answer.prevention_kind) else {
            record.failure = Some(format!(
                "the model chose prevention_kind {:?}, which is not default, code-check or design-change",
                answer.prevention_kind
            ));
            return None;
        };
        if text.iter().any(String::is_empty) {
            record.failure = Some("the answer leaves a field empty".into());
            return None;
        }
        record.what_happened = Some(text[0].clone());
        record.cause = Some(text[1].clone());
        record.prevention = Some(text[2].clone());
        record.proposed_request = Some(text[3].clone());
        record.prevention_kind = Some(kind);
        // Ids the model returned that code did not pass in are dropped and logged, not repaired.
        let class_id = RecordId::parse(&answer.failure_class).ok();
        let class_known = class_id
            .as_ref()
            .is_some_and(|c| past.classes.iter().any(|row| row.id == c.as_str()));
        for case in &answer.earlier_cases {
            let row = past.cases.iter().find(|c| &c.id == case);
            let in_class = class_known
                && row.is_some_and(|r| {
                    Some(r.failure_class.as_str()) == class_id.as_ref().map(RecordId::as_str)
                });
            match RecordId::parse(case) {
                Ok(id) if passed_cases.contains(case) && in_class => record.earlier_cases.push(id),
                _ => {
                    ui.line(&format!("introspect dropped earlier case {case}: not on the list passed in, or not in the class"));
                    record.dropped_ids.push(case.clone());
                }
            }
        }
        for decision in &answer.decisions_matched {
            if passed_decisions.contains(decision) {
                record.decisions_matched.push(decision.clone());
            } else {
                ui.line(&format!(
                    "introspect dropped decision {decision}: not on the list passed in"
                ));
                record.dropped_ids.push(decision.clone());
            }
        }
        if normalise(&text[0]) == normalise(&text[1]) {
            record.flags.push("the cause restates what_happened".into());
            record.failure =
                Some("the cause equals what happened after normalising; rated failed".into());
            return None;
        }
        let mut added = None;
        if answer.failure_class == "new" {
            match answer.new_class {
                Some(new) if !new.name.trim().is_empty() => {
                    let class = FailureClass {
                        id: RecordId::generate(crate::now()),
                        at: crate::now(),
                        name: new.name.trim().to_string(),
                        description: new.description.trim().to_string(),
                        status: ClassStatus::Proposed,
                        proposed_by: record.id.clone(),
                    };
                    record.failure_class = Some(class.id.clone());
                    record.new_class = true;
                    added = Some(class);
                }
                _ => {
                    record.failure = Some("the model chose a new class without naming it".into());
                    return None;
                }
            }
        } else if let (true, Some(id)) = (class_known, class_id) {
            record.failure_class = Some(id);
        } else {
            record.failure = Some(format!(
                "the model chose class {:?}, which is not in the registry",
                answer.failure_class
            ));
            return None;
        }
        record.status = IntrospectionStatus::Recorded;
        added
    }

    fn introspection_spec(
        &self,
        config: &athena_data::Config,
        event: &Event,
        id: &RecordId,
        prompt: String,
        scratch: &Path,
    ) -> Result<AgentInvocation> {
        let job_dir = scratch.to_path_buf();
        let env = athena_agent::env::build(
            &EnvSpec {
                agents: &config.agents,
                job: event.job.as_ref().map_or(id.as_str(), JobId::as_str),
                job_dir: &job_dir,
                snapshot: scratch,
                side: Side::Read,
                capability_env: &BTreeMap::new(),
            },
            |key| std::env::var(key).ok(),
        )
        .map_err(anyhow::Error::msg)?;
        Ok(AgentInvocation {
            job: event
                .job
                .as_ref()
                .map_or(id.as_str(), JobId::as_str)
                .to_string(),
            seq: 0,
            role: Role::Reader,
            model: config.models.reader.clone(),
            system_prompt: SYSTEM_PROMPT.to_string(),
            user_prompt: prompt,
            tools: Vec::new(),
            allowed_tools: Vec::new(),
            permission_mode: "dontAsk".into(),
            workdir: scratch.to_path_buf(),
            env,
            hook: None,
            max_budget_usd: config.budget.reader_usd,
            timeout: Duration::from_secs(config.budget.reader_timeout_s),
            json_schema: answer_schema(),
            scratch: scratch.to_path_buf(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::fake::CountingPinner;
    use crate::state::StateDir;
    use athena_agent::fake::{FakeAgentDriver, FakeTurn};
    use athena_data::records::{Build, CriterionKind, FindingSource, Outcome};
    use athena_data::{Quantity, Unit};
    use athena_git::Git;

    const JOB: &str = "20261014-170031-2r9b";

    fn verification(
        criteria: Vec<CriterionResult>,
        findings: Vec<VerificationFinding>,
        checks: Vec<CheckRun>,
        verdict: Verdict,
    ) -> Verification {
        Verification {
            job: JobId::parse(JOB).unwrap(),
            candidate_commit: "0".repeat(40),
            verified_fingerprint: "f".repeat(8),
            tool_config_from_base: Vec::new(),
            checks,
            check_cleanup: Vec::new(),
            verifier_invocation: None,
            criteria,
            findings,
            request_reading_mismatch: None,
            agent_verdict: None,
            final_verdict: verdict,
            reasons: Vec::new(),
            inputs: Vec::new(),
            summary: None,
        }
    }

    fn criterion(text: &str, met: bool) -> CriterionResult {
        CriterionResult {
            criterion: text.into(),
            kind: CriterionKind::Behavior,
            met,
            evidence: "evidence that must not be copied".into(),
            commands: Vec::new(),
            commands_verified: true,
        }
    }

    fn finding(severity: Severity, summary: &str, location: Option<&str>) -> VerificationFinding {
        VerificationFinding {
            severity,
            summary: summary.into(),
            evidence: "evidence that must not be copied".into(),
            location: location.map(String::from),
            source: FindingSource::Verifier,
        }
    }

    fn check(name: &str, passed: bool, tail: &str) -> CheckRun {
        CheckRun {
            name: name.into(),
            command: vec!["cargo".into(), name.into()],
            exit_code: Some(if passed { 0 } else { 101 }),
            passed,
            tests_ran: None,
            duration: Quantity::measured(1.0, Unit::Seconds, "test"),
            output_tail: tail.into(),
            note: None,
        }
    }

    /// An Athena over a new repository that holds the outcome (rejected) and, when given, the verification
    /// of the job, committed as stored records.
    fn athena_with(
        verification: Option<Verification>,
    ) -> (tempfile::TempDir, tempfile::TempDir, Athena) {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let git = Git::at(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value]).unwrap();
        }
        std::fs::write(dir.path().join("README.md"), "# Demo\n").unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "Start a demo repository"])
            .unwrap();
        let outcome = Outcome {
            job: JobId::parse(JOB).unwrap(),
            state: OutcomeState::Rejected,
            reason: "verification failed".into(),
            at: "2026-10-14T17:30:00Z".parse().unwrap(),
            totals: Some(crate::scoring::totals(&[], None)),
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        };
        let mut files = vec![NewFile::record(&AnyRecord::Outcome(Box::new(outcome)))];
        if let Some(v) = verification {
            files.push(NewFile::record(&AnyRecord::Verification(Box::new(v))));
        }
        for file in &files {
            let path = dir.path().join(&file.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, &file.bytes).unwrap();
        }
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "Record the job"]).unwrap();
        let driver = FakeAgentDriver::new(Box::new(|_| FakeTurn::output(json!({}))));
        let athena = Athena {
            git,
            state: StateDir::new(state.path().join("repo")),
            driver: std::sync::Arc::new(driver),
            capabilities: Vec::new(),
            build: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            athena_bin: "/bin/false".into(),
            nested: None,
            pinner: Box::new(CountingPinner::new(&"0".repeat(40))),
            jev: None,
        };
        (dir, state, athena)
    }

    fn what_of(verification: Option<Verification>) -> String {
        let (_dir, _state, athena) = athena_with(verification);
        athena
            .event_of_job(&JobId::parse(JOB).unwrap())
            .unwrap()
            .unwrap()
            .what
    }

    #[test]
    fn an_event_carries_the_unmet_criterion_and_the_major_finding() {
        let what = what_of(Some(verification(
            vec![
                criterion("Tests call event_of_job itself", false),
                criterion("The changelog has an entry", true),
            ],
            vec![
                finding(
                    Severity::Major,
                    "No test reaches event_of_job",
                    Some("crates/athena-core/src/introspect.rs"),
                ),
                finding(Severity::Minor, "A small wording slip", None),
            ],
            vec![check("test", true, "all fine")],
            Verdict::Fail,
        )));
        assert!(what.starts_with(": verification failed"), "{what}");
        assert!(what.contains("Unmet acceptance criteria (1):"), "{what}");
        assert!(what.contains("- Tests call event_of_job itself"), "{what}");
        assert!(!what.contains("changelog has an entry"), "{what}");
        assert!(
            what.contains(
                "major: No test reaches event_of_job at crates/athena-core/src/introspect.rs"
            ),
            "{what}"
        );
        assert!(!what.contains("wording slip"), "{what}");
        assert!(!what.contains("must not be copied"), "{what}");
        assert!(!what.contains("Failed checks"), "{what}");
    }

    #[test]
    fn a_failed_check_is_listed_apart_and_is_not_an_unmet_criterion() {
        let what = what_of(Some(verification(
            vec![criterion("The tests pass", true)],
            Vec::new(),
            vec![
                check("fmt", true, "clean"),
                check("test", false, "thread panicked: assertion failed"),
            ],
            Verdict::Fail,
        )));
        assert!(what.contains("Failed checks"), "{what}");
        assert!(
            what.contains("- test: thread panicked: assertion failed"),
            "{what}"
        );
        assert!(!what.contains("Unmet acceptance criteria"), "{what}");
        assert!(!what.contains("fmt"), "{what}");
    }

    #[test]
    fn a_pre_existing_failed_check_is_not_listed_as_a_cause() {
        let mut old = check("fmt", false, "already broken");
        old.note = Some(athena_data::records::PRE_EXISTING_NOTE.to_string());
        let what = what_of(Some(verification(
            vec![criterion("The tests pass", false)],
            Vec::new(),
            vec![old, check("test", false, "thread panicked")],
            Verdict::Fail,
        )));
        assert!(what.contains("- test: thread panicked"), "{what}");
        assert!(!what.contains("already broken"), "{what}");
    }

    #[test]
    fn a_job_without_verification_or_without_failures_keeps_the_old_what() {
        assert_eq!(what_of(None), ": verification failed");
        let passing = verification(
            vec![criterion("Fine", true)],
            Vec::new(),
            vec![check("test", true, "ok")],
            Verdict::Pass,
        );
        assert_eq!(what_of(Some(passing)), ": verification failed");
    }

    #[test]
    fn a_failed_verdict_without_details_says_so() {
        let what = what_of(Some(verification(
            Vec::new(),
            vec![finding(Severity::Nit, "style", None)],
            Vec::new(),
            Verdict::Fail,
        )));
        assert!(what.contains("lists no unmet criterion"), "{what}");
    }

    #[test]
    fn counts_and_text_are_capped() {
        let long = "x".repeat(TEXT_CAP * 3);
        let criteria = (0..DETAIL_CAP + 3)
            .map(|n| criterion(&format!("criterion {n} {long}"), false))
            .collect();
        let tail = format!("{}END", "y".repeat(TAIL_CAP * 3));
        let what = what_of(Some(verification(
            criteria,
            Vec::new(),
            vec![check("test", false, &tail)],
            Verdict::Fail,
        )));
        assert!(what.contains("- (3 more not shown)"), "{what}");
        assert!(!what.contains(&"x".repeat(TEXT_CAP + 1)), "{what}");
        assert!(what.contains("yEND"), "{what}");
        assert!(!what.contains(&"y".repeat(TAIL_CAP + 1)), "{what}");
    }

    #[test]
    fn normalising_ignores_case_punctuation_and_spacing() {
        assert_eq!(normalise("The  job FAILED, twice."), "the job failed twice");
        assert_eq!(normalise("  "), "");
    }

    #[test]
    fn decisions_are_matched_only_by_evidence_after_the_decision_started() {
        let mut records = Records::default();
        records.landed.insert("20261001-000000-aaaa".into());
        records.landed.insert("20261001-000000-bbbb".into());
        for (id, at, job) in [
            (
                "20261001-000000-i001",
                "2026-10-01T00:00:00Z",
                "20261001-000000-aaaa",
            ),
            (
                "20261001-000100-i002",
                "2026-10-01T00:01:00Z",
                "20261001-000000-bbbb",
            ),
        ] {
            records.interventions.push(InterventionRow {
                id: id.into(),
                at: at.into(),
                class_of_work: "code".into(),
                category: InterventionCategory::DidWorkThemselves,
                job: Some(job.into()),
            });
        }
        let mut starts = Starts::new();
        starts.insert("D87".into(), "2026-09-30T00:00:00Z".parse().unwrap());
        assert_eq!(
            decisions_matched(&records, &starts, "20261001-000000-i001"),
            ["D87"]
        );
        // The decision started after the evidence: no match.
        starts.insert("D87".into(), "2026-10-02T00:00:00Z".parse().unwrap());
        assert!(decisions_matched(&records, &starts, "20261001-000000-i001").is_empty());
    }
}
