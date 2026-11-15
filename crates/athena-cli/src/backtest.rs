//! The read-only backtest of decisions D87 and D88 (`docs/design/reviews/decision-sets-review-1.md`, section 5;
//! `docs/design/decision-sets.md` section 8, step 0).
//!
//! It does four things, all by code and from stored fields only:
//!
//! 1. Defines the typed header for D87 and D88 (and no other decision) and parses the supersession phrases of
//!    `docs/decisions.md` (the parse is `athena_data::decision_links::parse_supersessions`, re-exported here),
//!    then checks the hits against the user-confirmed list `docs/decision-links.toml`, whose links are keyed on
//!    the older and newer entry ids (see [`crate::decisions`]).
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
    headers, read_records, run_signals, starts_from_history, validate_citations,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

// ---------------------------------------------------------------------------------------------------------
// Supersession phrases of docs/decisions.md (decision-sets.md section 6, step 3).
// ---------------------------------------------------------------------------------------------------------

// The parse itself (`HitKind`, `Hit`, `Parsed`, `parse_supersessions`) lives in `athena_data::decision_links`,
// so that the producer-side link check and this backtest read the log through one parser.
pub use athena_data::decision_links::{Hit, HitKind, Parsed, parse_supersessions};

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
                "listed but not found in the log: {} and {}",
                link.older, link.newer
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

/// Condition (a), for the `[[governs]]` list: every governed decision is an entry of the log. A governed id with
/// no entry could never be amended, so a change to its paths could never reconcile it.
fn check_governs(parsed: &Parsed, confirmed: &ConfirmedLinks, findings: &mut Vec<Finding>) {
    for governed in confirmed.unknown_governed(&parsed.entries) {
        findings.push(Finding {
            condition: "a",
            reason: format!(
                "[[governs]] names {}, which is not an entry of the log (paths: {})",
                governed.decision,
                governed.paths.join(", ")
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
    check_governs(&parsed, confirmed, &mut findings);
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
        &read_records(root)?,
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
    fn a_governed_decision_that_is_not_an_entry_of_the_log_is_a_condition_a_finding() {
        let log = "## D01 · x\n\n- **Provenance:** p\n\n## D02 · y\n\n- **Provenance:** q\n";
        let links = ConfirmedLinks::parse(
            "[[governs]]\ndecision = \"D02\"\npaths = [\"src/\"]\n\n\
             [[governs]]\ndecision = \"D9999\"\npaths = [\"docs/a.md\", \"b/\"]\n",
        )
        .unwrap();
        let report = run_on(log, &links, &Records::default());
        let governs: Vec<&Finding> = report
            .findings
            .iter()
            .filter(|f| f.reason.contains("[[governs]]"))
            .collect();
        assert_eq!(governs.len(), 1, "{:?}", report.findings);
        assert_eq!(governs[0].condition, "a");
        assert!(governs[0].reason.contains("D9999"), "{}", governs[0].reason);
        assert!(governs[0].reason.contains("docs/a.md, b/"));
        // With the entry in the log there is nothing to report.
        let log = format!("{log}\n## D9999 · z\n\n- **Provenance:** r\n");
        let report = run_on(&log, &links, &Records::default());
        assert!(
            report
                .findings
                .iter()
                .all(|f| !f.reason.contains("[[governs]]")),
            "{:?}",
            report.findings
        );
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
