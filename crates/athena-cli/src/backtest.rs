//! The read-only backtest of decisions D87 and D88 (`docs/design/reviews/decision-sets-review-1.md`, section 5;
//! `docs/design/decision-sets.md` section 8, step 0).
//!
//! It does four things, all by code and from stored fields only:
//!
//! 1. Defines the typed header for D87 and D88 (and no other decision) and parses the supersession phrases of
//!    `docs/decisions.md`, then checks the hits against the user-confirmed list `docs/decision-links.toml`
//!    (see [`crate::decisions`]).
//! 2. Reads `records/` (job outcomes and interventions) without an agent.
//! 3. Runs the exact-match signals of those headers over the records. The intervention signal with an exact
//!    match is D87's: a hand change (`did-work-themselves`) on a landed job, class of work `code`, threshold
//!    two jobs (`docs/design/decision-sets.md` section 9). D87's host-error signal runs over the observed
//!    `athena.host-error/1` records and over the derived ones: records that code derives, in memory, from past
//!    interventions whose text states a host refusal ([`athena_core::watcher::derive_host_error`]). A derived
//!    record is marked derived, cites its source intervention, and is listed apart from the observed ones; the
//!    client did not observe it.
//! 4. Builds one introspection record for the earliest event that fired a signal naming D87. The record holds
//!    only the fields code writes; the fields a language model writes are listed as pending.
//!
//! The result is a [`Report`] with the review's pass conditions applied. A condition that does not hold is a
//! [`Finding`] with its reason; the report does not pass, and nothing is retried or loosened. The backtest writes
//! nothing: the introspection record is returned, not committed.

use crate::decisions::{ConfirmedLinks, DecisionView, mark_under_review, view};
use anyhow::{Context, Result};
use athena_core::watcher::{self, D87_SIGNAL, HostErrorSignal};
use athena_data::records::{AnyRecord, HostError, InterventionCategory, OutcomeState};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

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
/// from the records this backtest reads (history checks have no record yet), so it is listed and not run.
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

/// The typed headers the backtest knows: D87 and D88, as `decision-sets.md` section 4 gives them. D87 stays
/// `active` with `amended_by = D88`, not `superseded`: it is superseded for its landing shape only, and its two
/// rerun fixes stand (review 1, section 3). Its enforcement is `brief` (finding F4).
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
// Supersession phrases of docs/decisions.md (decision-sets.md section 6, step 3).
// ---------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HitKind {
    /// "Superseded by Dnn", written inside the older entry.
    SupersededBy,
    /// "Supersedes Dnn", written inside the newer entry.
    Supersedes,
    /// "Amends Dnn", written inside the newer entry.
    Amends,
}

/// One phrase with a target id. `older` is the entry that is superseded or amended; `newer` is the one doing it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hit {
    pub line: usize,
    pub entry: String,
    pub kind: HitKind,
    pub older: String,
    pub newer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Parsed {
    pub entries: Vec<String>,
    pub hits: Vec<Hit>,
    /// Lines where a supersession phrase stands as a statement but names no entry.
    pub lacking_target: Vec<usize>,
}

const PHRASES: [(&str, HitKind); 3] = [
    ("superseded by", HitKind::SupersededBy),
    ("supersedes", HitKind::Supersedes),
    ("amends", HitKind::Amends),
];

/// An entry id (`D` and at least two digits) at the start of `text`, and the rest after it.
fn take_id(text: &str) -> Option<(String, &str)> {
    let rest = text.strip_prefix('D')?;
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    if digits < 2 {
        return None;
    }
    Some((format!("D{}", &rest[..digits]), &rest[digits..]))
}

/// The ids named after a phrase. A phrase names one id, except "Amends", which may list several joined by
/// commas or "and" ("Amends D06 and D14"). The list stops at the first id that is followed by anything else.
fn target_ids(after: &str, list: bool) -> Vec<String> {
    let mut ids = Vec::new();
    let mut rest = after.trim_start();
    while let Some((id, tail)) = take_id(rest) {
        ids.push(id);
        if !list {
            break;
        }
        let next = [", and ", " and ", ", "]
            .iter()
            .find_map(|sep| tail.strip_prefix(sep));
        match next {
            Some(next) => rest = next,
            None => break,
        }
    }
    ids
}

/// A phrase that opens a statement: after `**`, `(`, "this", or at the start of the text. Descriptive prose
/// ("each supersedes or amends an accepted entry") is not a statement and is not counted as lacking a target.
fn opens_statement(before: &str) -> bool {
    let before = before.trim_end();
    before.is_empty() || before.ends_with("**") || before.ends_with('(') || before.ends_with("this")
}

fn entry_heading(line: &str) -> Option<String> {
    let rest = line.strip_prefix("## ")?;
    take_id(rest).map(|(id, _)| id)
}

/// Parse the phrases of the log. The text before the first entry heading is the log's own header and is not
/// read: it describes the rule and names no entry. Matching is case-insensitive (D79 writes "This supersedes").
pub fn parse_supersessions(text: &str) -> Parsed {
    let mut parsed = Parsed {
        entries: Vec::new(),
        hits: Vec::new(),
        lacking_target: Vec::new(),
    };
    let mut current: Option<String> = None;
    for (index, line) in text.lines().enumerate() {
        if let Some(id) = entry_heading(line) {
            parsed.entries.push(id.clone());
            current = Some(id);
            continue;
        }
        let Some(entry) = &current else { continue };
        let lower = line.to_ascii_lowercase();
        for (phrase, kind) in PHRASES {
            for (at, _) in lower.match_indices(phrase) {
                if lower[..at]
                    .chars()
                    .next_back()
                    .is_some_and(char::is_alphabetic)
                {
                    continue;
                }
                let ids = target_ids(&line[at + phrase.len()..], kind == HitKind::Amends);
                if ids.is_empty() {
                    if opens_statement(&lower[..at]) {
                        parsed.lacking_target.push(index + 1);
                    }
                    continue;
                }
                for id in ids {
                    let (older, newer) = match kind {
                        HitKind::SupersededBy => (entry.clone(), id),
                        HitKind::Supersedes | HitKind::Amends => (id, entry.clone()),
                    };
                    parsed.hits.push(Hit {
                        line: index + 1,
                        entry: entry.clone(),
                        kind,
                        older,
                        newer,
                    });
                }
            }
        }
    }
    parsed
}

/// What the parse says about each older entry: the entry that supersedes it, and the entries that amend it.
/// This is the literal reading of the phrases; a partial supersession (D87) is for the user to confirm, and the
/// typed header records that reading.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Standing {
    pub superseded_by: BTreeSet<String>,
    pub amended_by: BTreeSet<String>,
}

pub fn standing(hits: &[Hit]) -> BTreeMap<String, Standing> {
    let mut map: BTreeMap<String, Standing> = BTreeMap::new();
    for hit in hits {
        let entry = map.entry(hit.older.clone()).or_default();
        if hit.kind == HitKind::Amends {
            entry.amended_by.insert(hit.newer.clone());
        } else {
            entry.superseded_by.insert(hit.newer.clone());
        }
    }
    map
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

/// Read job outcomes (`records/jobs/<month>/<job>/outcome*.json`) and interventions
/// (`records/interventions/<month>/<id>.json`). A missing `records/` reads as empty.
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
    fn of(error: &HostError) -> Self {
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

fn matches(m: &InterventionMatch, row: &InterventionRow, records: &Records) -> bool {
    row.category == m.category
        && row.class_of_work == m.class_of_work
        && match &row.job {
            Some(job) => !m.on_landed_job || records.landed.contains(job),
            None => false,
        }
}

pub fn run_signals(headers: &[DecisionHeader], records: &Records) -> Vec<SignalRun> {
    let mut runs = Vec::new();
    for header in headers {
        for signal in &header.signals {
            if let Some(host_error) = &signal.host_error {
                let matched: Vec<&HostError> = records
                    .host_errors
                    .iter()
                    .filter(|e| host_error.matches(e))
                    .collect();
                let derived: Vec<&HostError> = records
                    .derived_host_errors
                    .iter()
                    .filter(|e| host_error.matches(e))
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
                .filter(|row| matches(exact, row, records))
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
// The introspection record (defaults-loop.md section 3.2): the code-written fields.
// ---------------------------------------------------------------------------------------------------------

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
    /// The fields a language model or a typed decision writes; not written by this backtest.
    pub pending_fields: Vec<&'static str>,
}

fn introspection_for(run: &SignalRun) -> Option<Introspection> {
    let first = run.events.first()?;
    Some(Introspection {
        schema: "athena.introspection/1",
        intervention: first.id.clone(),
        job: first.job.clone()?,
        class_of_work: first.class_of_work.clone(),
        at: first.at.clone(),
        earlier_cases: run
            .events
            .iter()
            .take_while(|e| (&e.at, &e.id) < (&first.at, &first.id))
            .map(|e| e.id.clone())
            .collect(),
        decisions_matched: vec![run.decision.clone()],
        pending_fields: vec![
            "what_happened",
            "failure_class",
            "cause",
            "prevention_kind",
            "prevention",
            "proposed_request",
        ],
    })
}

// ---------------------------------------------------------------------------------------------------------
// The report and the review's pass conditions (review 1, section 5).
// ---------------------------------------------------------------------------------------------------------

/// A pass condition that did not hold, with the reason. A finding is reported as it is; the backtest does not
/// retry or relax a condition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    /// The review's condition: `a`, `b`, `c` or `d`.
    pub condition: &'static str,
    pub reason: String,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub entries: usize,
    pub hits: Vec<Hit>,
    pub lacking_target: Vec<usize>,
    /// The typed view of every entry of the log, from the parse and the confirmed list.
    pub decision_view: Vec<DecisionView>,
    pub headers: Vec<DecisionHeader>,
    pub jobs_read: usize,
    pub landed_jobs: usize,
    pub interventions_read: usize,
    /// Observed host-error records read from `records/`.
    pub host_errors_read: usize,
    /// Host-error records derived from interventions, listed apart from the observed ones. Each cites its
    /// source intervention; the client did not observe them.
    pub derived_host_errors: Vec<DerivedCite>,
    pub runs: Vec<SignalRun>,
    pub under_review: Vec<String>,
    pub introspection: Option<Introspection>,
    pub findings: Vec<Finding>,
}

impl Report {
    pub fn passed(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Condition (a): every parsed hit is in the user-confirmed list, and every listed hit is found by the parse.
fn check_parse(parsed: &Parsed, confirmed: &ConfirmedLinks, findings: &mut Vec<Finding>) {
    for hit in confirmed.unlisted(&parsed.hits) {
        findings.push(Finding {
            condition: "a",
            reason: format!(
                "hit not in the confirmed list: {} {} {} (line {}, in {})",
                hit.older,
                match hit.kind {
                    HitKind::Amends => "amended by",
                    _ => "superseded by",
                },
                hit.newer,
                hit.line,
                hit.entry
            ),
        });
    }
    for link in confirmed.not_found(&parsed.hits) {
        findings.push(Finding {
            condition: "a",
            reason: format!(
                "listed but not found in the log: {} and {} at line {}",
                link.older, link.newer, link.line
            ),
        });
    }
    if !parsed.lacking_target.is_empty() {
        findings.push(Finding {
            condition: "a",
            reason: format!(
                "supersession phrases without a target id at lines {:?}",
                parsed.lacking_target
            ),
        });
    }
}

fn check_signals(
    runs: &[SignalRun],
    records: &Records,
    findings: &mut Vec<Finding>,
) -> Vec<String> {
    let under_review: Vec<String> = runs
        .iter()
        .filter(|r| r.fired)
        .map(|r| r.decision.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    // D87 passes when any of its signals fired. A host-error signal cites its own records, which exist by
    // construction; an intervention signal cites interventions and jobs that must exist in records/.
    let d87: Vec<&SignalRun> = runs.iter().filter(|r| r.decision == "D87").collect();
    match d87
        .iter()
        .find(|r| r.fired)
        .or_else(|| d87.last())
        .copied()
    {
        Some(run) if run.fired => {
            let known: BTreeSet<&str> = records.interventions.iter().map(|i| i.id.as_str()).collect();
            for event in &run.events {
                let job_exists = event.job.as_ref().is_some_and(|j| records.jobs.contains(j));
                if !known.contains(event.id.as_str()) || !job_exists {
                    findings.push(Finding {
                        condition: "b",
                        reason: format!(
                            "cited intervention {} or its job does not exist in records/",
                            event.id
                        ),
                    });
                }
            }
        }
        Some(run) => findings.push(Finding {
            condition: "b",
            reason: format!(
                "the signal found {} distinct job(s), fewer than the threshold of {}; no athena.host-error/1 \
                 record of a refused merge (405 or 422) is in records/, no intervention states one, and the \
                 interventions were not recorded with the fields the signal needs (finding F3)",
                run.jobs.len(),
                run.threshold_jobs
            ),
        }),
        None => findings.push(Finding {
            condition: "b",
            reason: "D87 has no exact-match signal to run".to_string(),
        }),
    }
    let others: Vec<&String> = under_review.iter().filter(|d| *d != "D87").collect();
    if !others.is_empty() {
        findings.push(Finding {
            condition: "c",
            reason: format!("the signal marked other typed decisions under review: {others:?}"),
        });
    }
    under_review
}

fn check_introspection(
    introspection: &Introspection,
    records: &Records,
    findings: &mut Vec<Finding>,
) {
    for case in &introspection.earlier_cases {
        match records.interventions.iter().find(|i| &i.id == case) {
            None => findings.push(Finding {
                condition: "d",
                reason: format!("earlier case {case} does not exist in records/"),
            }),
            Some(row) if row.class_of_work != introspection.class_of_work => {
                findings.push(Finding {
                    condition: "d",
                    reason: format!(
                        "earlier case {case} is in class {}, not {}",
                        row.class_of_work, introspection.class_of_work
                    ),
                })
            }
            Some(_) => {}
        }
    }
}

/// Run the backtest on the text of `docs/decisions.md` and the records under `root`.
pub fn run_on(decisions: &str, confirmed: &ConfirmedLinks, records: &Records) -> Report {
    let parsed = parse_supersessions(decisions);
    let headers = headers();
    let runs = run_signals(&headers, records);
    let mut findings = Vec::new();
    check_parse(&parsed, confirmed, &mut findings);
    let mut decision_view = view(&parsed, confirmed);
    let under_review = check_signals(&runs, records, &mut findings);
    for run in runs.iter().filter(|r| r.fired) {
        let cited: Vec<String> = run
            .host_error_records
            .iter()
            .cloned()
            .chain(run.derived_records.iter().map(DerivedCite::citation))
            .chain(run.events.iter().map(|e| e.id.clone()))
            .collect();
        mark_under_review(&mut decision_view, &run.decision, &cited);
    }
    let introspection = runs
        .iter()
        .find(|r| r.decision == "D87" && r.fired && !r.events.is_empty())
        .and_then(introspection_for);
    if let Some(record) = &introspection {
        check_introspection(record, records, &mut findings);
    }
    Report {
        entries: parsed.entries.len(),
        hits: parsed.hits,
        lacking_target: parsed.lacking_target,
        decision_view,
        headers,
        jobs_read: records.jobs.len(),
        landed_jobs: records.landed.len(),
        interventions_read: records.interventions.len(),
        host_errors_read: records.host_errors.len(),
        derived_host_errors: records
            .derived_host_errors
            .iter()
            .map(DerivedCite::of)
            .collect(),
        runs,
        under_review,
        introspection,
        findings,
    }
}

/// Read `docs/decisions.md`, the confirmed list `docs/decision-links.toml` and `records/` under `root`, and run
/// the backtest.
pub fn run(root: &Path) -> Result<Report> {
    let path = root.join("docs/decisions.md");
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    Ok(run_on(
        &text,
        &ConfirmedLinks::load(root)?,
        &read_records(root)?,
    ))
}

/// A short text view of a report.
pub fn render(report: &Report) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "decisions: {} entries, {} supersession/amend hits, {} phrase(s) lacking a target\n",
        report.entries,
        report.hits.len(),
        report.lacking_target.len()
    ));
    out.push_str(&format!(
        "records: {} job(s) with outcomes ({} landed), {} intervention(s)\n",
        report.jobs_read, report.landed_jobs, report.interventions_read
    ));
    out.push_str(&format!(
        "host errors: {} observed by the client\n",
        report.host_errors_read
    ));
    out.push_str(&format!(
        "host errors derived from interventions (not observed by the client): {}\n",
        report.derived_host_errors.len()
    ));
    for cite in &report.derived_host_errors {
        out.push_str(&format!("  - derived {}\n", cite.citation()));
    }
    for run in &report.runs {
        out.push_str(&format!(
            "signal {} [{}]: {} job(s) of {} needed: {}\n",
            run.decision,
            run.signal,
            run.jobs.len(),
            run.threshold_jobs,
            if run.fired { "fired" } else { "did not fire" }
        ));
    }
    for view in report
        .decision_view
        .iter()
        .filter(|v| v.under_review.is_some())
    {
        out.push_str(&format!(
            "{} is under review: cited record(s) {}\n",
            view.id,
            view.under_review.as_deref().unwrap_or_default().join(", ")
        ));
    }
    if let Some(record) = &report.introspection {
        out.push_str(&format!(
            "introspection: intervention {} (job {}) names {}\n",
            record.intervention,
            record.job,
            record.decisions_matched.join(", ")
        ));
    }
    if report.passed() {
        out.push_str("backtest: pass\n");
    } else {
        out.push_str("backtest: fail (a finding, not a retry)\n");
        for finding in &report.findings {
            out.push_str(&format!("  - ({}) {}\n", finding.condition, finding.reason));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_id_needs_two_digits() {
        assert_eq!(take_id("D88**"), Some(("D88".to_string(), "**")));
        assert_eq!(take_id("D8 x"), None);
        assert_eq!(take_id("Dx"), None);
    }

    #[test]
    fn amends_lists_split_on_and_and_commas() {
        assert_eq!(target_ids(" D06 and D14. The gate", true), ["D06", "D14"]);
        assert_eq!(target_ids(" D39's exemption list and D06.", true), ["D39"]);
        assert_eq!(target_ids(" D87's", false), ["D87"]);
    }

    #[test]
    fn descriptive_prose_is_not_a_missing_target() {
        let text = "## D01 · x\n\nEach supersedes or amends an accepted entry\n(active, superseded by a named decision)\n";
        let parsed = parse_supersessions(text);
        assert!(parsed.hits.is_empty());
        assert!(parsed.lacking_target.is_empty());
    }

    #[test]
    fn a_statement_without_an_id_is_reported_with_its_line() {
        let text = "## D01 · x\n- **Superseded by** nothing\n";
        assert_eq!(parse_supersessions(text).lacking_target, [2]);
    }
}
