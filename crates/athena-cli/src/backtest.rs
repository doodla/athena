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
//! 4. Builds one introspection record from the earliest valid citation of whichever D87 signal fired: an
//!    intervention, or for a derived host-error record its source intervention. A decision under review with no
//!    such record is a finding (condition `d`). The record holds
//!    only the fields code writes; the fields a language model writes are listed as pending.
//!
//! Only records later than a decision's start count. The start is the first commit that introduced the
//! decision's id heading ([`decision_start`]). Every record a fired signal cites is validated against the
//! signal's fields and that window ([`validate_citations`]); an invalid citation is a finding.
//!
//! The typed headers, the records reader, the evidence-after-decision rule, the citation validation and the
//! introspection schema are defined once, in [`athena_core::introspect`], which the introspection step uses
//! as well; this module re-exports them and keeps no copy.
//!
//! The result is a [`Report`] with the review's pass conditions applied. A condition that does not hold is a
//! [`Finding`] with its reason; the report does not pass, and nothing is retried or loosened. The backtest writes
//! nothing: the introspection record is returned, not committed.

use crate::decisions::{ConfirmedLinks, DecisionView, mark_under_review, view};
use anyhow::{Context, Result};
pub use athena_core::introspect::{
    DecisionHeader, DerivedCite, Enforcement, Finding, InterventionMatch, InterventionRow,
    Introspection, Kind, Phase, Records, Signal, SignalRun, Starts, Status, decision_start,
    headers, read_records, read_records_current, run_signals, starts_from_history,
    validate_citations,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

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
// The introspection record (defaults-loop.md section 3.2): the code-written fields.
// ---------------------------------------------------------------------------------------------------------

/// The interventions a fired run cites that can serve as the introspection's event, each with a valid
/// citation, earliest first. An intervention event is its own source; a derived host-error record is
/// represented by its source intervention. An observed host-error record has no intervention row, so it
/// is not a source. A citation that fails [`validate_citations`] is skipped, as is a source with no job.
fn introspection_sources(
    run: &SignalRun,
    headers: &[DecisionHeader],
    records: &Records,
    starts: &Starts,
) -> Vec<InterventionRow> {
    let valid =
        |single: SignalRun| validate_citations(headers, &[single], records, starts).is_empty();
    let bare = SignalRun {
        events: Vec::new(),
        host_error_records: Vec::new(),
        derived_records: Vec::new(),
        ..run.clone()
    };
    let mut ids: BTreeSet<&str> = BTreeSet::new();
    for event in &run.events {
        let single = SignalRun {
            events: vec![event.clone()],
            ..bare.clone()
        };
        if valid(single) {
            ids.insert(event.id.as_str());
        }
    }
    for cite in &run.derived_records {
        let single = SignalRun {
            derived_records: vec![cite.clone()],
            ..bare.clone()
        };
        if valid(single) {
            ids.insert(cite.source_intervention.as_str());
        }
    }
    let mut rows: Vec<InterventionRow> = records
        .interventions
        .iter()
        .filter(|row| ids.contains(row.id.as_str()) && row.job.is_some())
        .cloned()
        .collect();
    rows.sort_by(|a, b| (&a.at, &a.id).cmp(&(&b.at, &b.id)));
    rows
}

/// The introspection record for a fired run, from the earliest valid citation of whichever signal fired.
fn introspection_for(
    run: &SignalRun,
    headers: &[DecisionHeader],
    records: &Records,
    starts: &Starts,
) -> Option<Introspection> {
    let sources = introspection_sources(run, headers, records, starts);
    let first = sources.first()?;
    Some(Introspection::code_fields(
        first,
        first.job.clone()?,
        sources
            .iter()
            .skip(1)
            .take_while(|e| (&e.at, &e.id) < (&first.at, &first.id))
            .map(|e| e.id.clone())
            .collect(),
        vec![run.decision.clone()],
    ))
}

// ---------------------------------------------------------------------------------------------------------
// The report and the review's pass conditions (review 1, section 5).
// ---------------------------------------------------------------------------------------------------------

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

/// Run the backtest with no decision start known: every record counts. Use [`run_on_from`] to count only the
/// records after each decision's start.
pub fn run_on(decisions: &str, confirmed: &ConfirmedLinks, records: &Records) -> Report {
    run_on_from(decisions, confirmed, records, &Starts::new())
}

/// Run the backtest on the text of `docs/decisions.md` and the records, counting only the records later than
/// the start of the decision a signal belongs to, and failing on any invalid citation.
pub fn run_on_from(
    decisions: &str,
    confirmed: &ConfirmedLinks,
    records: &Records,
    starts: &Starts,
) -> Report {
    let parsed = parse_supersessions(decisions);
    let headers = headers();
    let runs = run_signals(&headers, records, starts);
    let mut findings = Vec::new();
    findings.extend(validate_citations(&headers, &runs, records, starts));
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
        .filter(|r| r.decision == "D87" && r.fired)
        .find_map(|r| introspection_for(r, &headers, records, starts));
    if let Some(record) = &introspection {
        check_introspection(record, records, &mut findings);
    } else {
        for decision in &under_review {
            findings.push(Finding {
                condition: "d",
                reason: format!(
                    "{decision} is under review but no introspection record could be built: \
                     no fired run cites a valid intervention, or a derived record's source intervention"
                ),
            });
        }
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

/// Read `docs/decisions.md`, the confirmed list `docs/decision-links.toml` and `records/` under `root`, date
/// each typed decision from git history ([`starts_from_history`]) and run the backtest.
pub fn run(root: &Path) -> Result<Report> {
    let path = root.join("docs/decisions.md");
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    Ok(run_on_from(
        &text,
        &ConfirmedLinks::load(root)?,
        // The records are the current ones (the `athena/records` worktree when the repository has one, D111);
        // the decisions and their dates are in the checkout `root`.
        &read_records_current(root)?,
        &starts_from_history(root)?,
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
    use athena_core::introspect::{after_start, row_at};

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

    #[test]
    fn the_evidence_after_decision_rule_is_the_shared_one() {
        let mut starts = Starts::new();
        starts.insert("D87".into(), "2026-10-01T00:00:00Z".parse().unwrap());
        assert!(after_start(
            &starts,
            "D87",
            "2026-10-02T00:00:00Z".parse().ok()
        ));
        assert!(!after_start(
            &starts,
            "D87",
            "2026-09-30T00:00:00Z".parse().ok()
        ));
        let row = InterventionRow {
            id: "x".into(),
            at: "not a time".into(),
            class_of_work: "code".into(),
            category: athena_data::records::InterventionCategory::Other,
            job: None,
        };
        assert!(row_at(&row).is_none());
    }
}
