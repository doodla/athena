//! Per-gate counters: how many jobs a gate ended, and how often a later try overruled it.
//!
//! A gate is a check or rule that can end a job: a failed check of a rejected job (`fmt`, `clippy`, `test`),
//! the verifier when a job was rejected with no failed check, or an admission rule that a rejected or failed
//! job violated. The counts come from the job records alone (`request`, `candidate`, `verification` and
//! `outcome`); nothing is stored or backfilled.
//!
//! - A gate **ended** a job once for each job it ended (a gate counts once per job, however many of its
//!   checks failed).
//! - A gate was **overruled** by a job when a later request job "try again on <job>" landed the same change
//!   the gate had ended. At most one overruling is counted for each ended job, so `disagreements <= ends`.
//!
//! "The same change" is judged without the whole-tree fingerprint when the base moved, because that
//! fingerprint covers the whole governed tree and differs as soon as any other job lands in between: when
//! both candidates were made on the same base fingerprint, their candidate fingerprints must be equal;
//! otherwise their sorted, non-empty change lists (path, status, mode, bytes) must be equal. The change list
//! has no content hash, so two different edits of the same size to the same files read as the same change on
//! a moved base. That is a known weakness of this signal.

use crate::reading::retried_job;
use crate::store::{JobRecords, JobState, Records};
use athena_data::records::{Candidate, OutcomeState, PRE_EXISTING_NOTE};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// The gate named when a job was rejected and no check failed: the verifier's own verdict.
pub const VERIFIER_GATE: &str = "verifier";

/// One gate's counters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GateCount {
    pub gate: String,
    /// Jobs this gate ended.
    pub ends: u32,
    /// Ended jobs whose next attempt (a landed "try again on <job>") landed the same change.
    pub disagreements: u32,
}

impl GateCount {
    /// `ended 5 job(s), overruled 5 time(s)`: the figures Hestia's request and the introspection prompt cite.
    pub fn describe(&self) -> String {
        format!(
            "ended {} job(s), overruled {} time(s)",
            self.ends, self.disagreements
        )
    }
}

/// A gate name that can be written into a record line and read back: not empty, no `]`, no line break.
fn usable(name: &str) -> bool {
    !name.trim().is_empty() && name == name.trim() && !name.contains([']', '\n', '\r'])
}

/// The gates that ended `job`: empty unless it was rejected (failed checks, else the verifier; plus any
/// admission rule it violated) or failed (admission rules it violated).
fn gates_of(job: &JobRecords) -> BTreeSet<String> {
    let mut gates = BTreeSet::new();
    let Some(outcome) = &job.outcome else {
        return gates;
    };
    if outcome.state == OutcomeState::Rejected
        && let Some(verification) = &job.verification
    {
        let failed: Vec<&str> = verification
            .checks
            .iter()
            .filter(|c| {
                !c.passed
                    && c.revert_for.is_empty()
                    && !c
                        .note
                        .as_deref()
                        .is_some_and(|n| n.starts_with(PRE_EXISTING_NOTE))
            })
            .map(|c| c.name.as_str())
            .collect();
        if failed.is_empty() {
            gates.insert(VERIFIER_GATE.to_string());
        } else {
            gates.extend(failed.into_iter().map(str::to_string));
        }
    }
    if matches!(outcome.state, OutcomeState::Rejected | OutcomeState::Failed)
        && let Some(candidate) = &job.candidate
    {
        gates.extend(candidate.violations.iter().map(|v| v.rule.clone()));
    }
    gates.retain(|g| usable(g));
    gates
}

/// Whether two candidates are the same change; see the module comment for the rule.
fn same_change(earlier: &Candidate, later: &Candidate) -> bool {
    if let (Some(a), Some(b)) = (&earlier.base_fingerprint, &later.base_fingerprint)
        && a == b
    {
        return matches!((&earlier.fingerprint, &later.fingerprint), (Some(x), Some(y)) if x == y);
    }
    let sorted = |c: &Candidate| {
        let mut changes = c.changes.clone();
        changes.sort_by(|x, y| x.path.cmp(&y.path));
        changes
    };
    let (a, b) = (sorted(earlier), sorted(later));
    !a.is_empty() && a == b
}

/// The gate counters of `records`: most overruled first, then most ended, then by name.
pub fn gate_counts(records: &Records) -> Vec<GateCount> {
    // For each job, the request jobs that landed and retry it ("try again on <job>").
    let mut landed_retries: BTreeMap<String, Vec<&JobRecords>> = BTreeMap::new();
    for job in records.request_jobs() {
        if job.state() != JobState::Closed(OutcomeState::Landed) {
            continue;
        }
        let Some(request) = &job.request else {
            continue;
        };
        if let Some(retried) = retried_job(records, &request.text)
            && &retried != job.id()
        {
            landed_retries
                .entry(retried.as_str().to_string())
                .or_default()
                .push(job);
        }
    }
    let mut counts: BTreeMap<String, GateCount> = BTreeMap::new();
    for job in records.request_jobs() {
        let gates = gates_of(job);
        if gates.is_empty() {
            continue;
        }
        let overruled = match &job.candidate {
            Some(ended) => landed_retries
                .get(job.id().as_str())
                .into_iter()
                .flatten()
                .filter_map(|retry| retry.candidate.as_ref())
                .any(|landed| same_change(ended, landed)),
            None => false,
        };
        for gate in gates {
            let entry = counts.entry(gate.clone()).or_insert(GateCount {
                gate,
                ends: 0,
                disagreements: 0,
            });
            entry.ends += 1;
            entry.disagreements += u32::from(overruled);
        }
    }
    let mut counts: Vec<GateCount> = counts.into_values().collect();
    counts.sort_by(|a, b| {
        b.disagreements
            .cmp(&a.disagreements)
            .then(b.ends.cmp(&a.ends))
            .then(a.gate.cmp(&b.gate))
    });
    counts
}

/// The start of a prevention text that names a gate.
const GATE_PREFIX: &str = "[gate: ";

/// `text` with the gate it concerns written at its start: `[gate: NAME] text`. This is a stopgap until an
/// introspection record has a typed field for the gate.
pub fn mark_prevention(gate: &str, text: &str) -> String {
    format!("{GATE_PREFIX}{gate}] {text}")
}

/// The gate named at the start of a prevention text written by [`mark_prevention`]; `None` for any other
/// text. Anchored at the start and needs the closing `] `.
pub fn gate_of_prevention(text: &str) -> Option<&str> {
    let rest = text.strip_prefix(GATE_PREFIX)?;
    let (gate, after) = rest.split_once("] ")?;
    (usable(gate) && !after.trim().is_empty()).then_some(gate)
}

/// `text` without the gate prefix [`mark_prevention`] wrote; the text unchanged when it has none.
pub fn prevention_without_gate(text: &str) -> &str {
    match gate_of_prevention(text) {
        Some(_) => text.split_once("] ").map_or(text, |(_, rest)| rest),
        None => text,
    }
}

/// Test fixtures: job records for a gate that ended jobs and the tries that followed, shared with the
/// tests of other modules.
#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;
    use athena_data::records::{
        AnyRecord, ChangeEntry, ChangeStatus, CheckRun, Outcome, PolicyViolation, Request, Verdict,
        Verification,
    };
    use athena_data::{JobId, Quantity, Unit};

    /// The id of test job `n`.
    pub fn job_id(n: u32) -> JobId {
        JobId::parse(&format!("20261001-10{:02}{:02}-k7q2", n / 60, n % 60)).unwrap()
    }

    fn at(n: u32) -> jiff::Timestamp {
        format!("2026-10-01T10:{:02}:{:02}Z", n / 60, n % 60)
            .parse()
            .unwrap()
    }

    pub fn request(job: &JobId, text: &str, n: u32) -> AnyRecord {
        AnyRecord::Request(Request {
            job: job.clone(),
            text: text.into(),
            received_at: at(n),
            channel: "cli".into(),
        })
    }

    /// A candidate made on `base` (its fingerprint) that has fingerprint `tree` and these changes (path and
    /// size of each added file) and violates these admission rules.
    pub fn candidate(
        job: &JobId,
        base: &str,
        tree: &str,
        changes: &[(&str, u64)],
        rules: &[&str],
    ) -> AnyRecord {
        AnyRecord::Candidate2(Box::new(Candidate {
            job: job.clone(),
            base_commit: "0".repeat(40),
            base_fingerprint: Some(base.repeat(64).chars().take(64).collect()),
            previous_job: None,
            candidate_commit: None,
            candidate_tree: None,
            changes: changes
                .iter()
                .map(|(path, bytes)| ChangeEntry {
                    path: (*path).into(),
                    status: ChangeStatus::Added,
                    mode: Some("100644".into()),
                    bytes: Some(*bytes),
                })
                .collect(),
            fingerprint: Some(tree.repeat(64).chars().take(64).collect()),
            violations: rules
                .iter()
                .map(|rule| PolicyViolation {
                    rule: (*rule).into(),
                    path: None,
                    detail: "violated".into(),
                })
                .collect(),
            producer_report: None,
        }))
    }

    pub fn check(name: &str, passed: bool, note: Option<&str>) -> CheckRun {
        CheckRun {
            name: name.into(),
            command: vec!["cargo".into(), name.into()],
            exit_code: Some(if passed { 0 } else { 101 }),
            passed,
            tests_ran: None,
            duration: Quantity::measured(1.0, Unit::Seconds, "test"),
            output_tail: String::new(),
            note: note.map(String::from),
            failed_tests: Vec::new(),
            reused: false,
            revert_for: Vec::new(),
            queued_s: None,
        }
    }

    /// A failed verification whose checks are these (name, passed, note).
    pub fn verification(job: &JobId, checks: Vec<CheckRun>) -> AnyRecord {
        AnyRecord::Verification(Box::new(Verification {
            job: job.clone(),
            candidate_commit: "0".repeat(40),
            verified_fingerprint: "f".repeat(64),
            tool_config_from_base: Vec::new(),
            checks,
            check_cleanup: Vec::new(),
            verifier_invocation: None,
            criteria: Vec::new(),
            findings: Vec::new(),
            request_reading_mismatch: None,
            agent_verdict: None,
            final_verdict: Verdict::Fail,
            reasons: Vec::new(),
            inputs: Vec::new(),
            summary: None,
        }))
    }

    pub fn outcome(job: &JobId, state: OutcomeState, n: u32) -> AnyRecord {
        AnyRecord::Outcome(Box::new(Outcome {
            job: job.clone(),
            state,
            reason: "ended".into(),
            at: at(n),
            totals: Some(crate::scoring::totals(&[], None)),
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        }))
    }

    /// A job `n` that `gate` (a failed check) rejected, with its candidate on `base` with fingerprint `tree`
    /// and the one change `(path, bytes)`.
    pub fn rejected_by(
        n: u32,
        gate: &str,
        base: &str,
        tree: &str,
        change: (&str, u64),
    ) -> Vec<AnyRecord> {
        let job = job_id(n);
        vec![
            request(&job, "Add a thing", n),
            candidate(&job, base, tree, &[change], &[]),
            verification(&job, vec![check(gate, false, None)]),
            outcome(&job, OutcomeState::Rejected, n),
        ]
    }

    /// A job `n` that retried job `of` ("try again on <of>"), ended in `state`, with its candidate on `base`
    /// with fingerprint `tree` and the one change `(path, bytes)`.
    pub fn retry(
        n: u32,
        of: u32,
        state: OutcomeState,
        base: &str,
        tree: &str,
        change: (&str, u64),
    ) -> Vec<AnyRecord> {
        let job = job_id(n);
        let mut records = vec![
            request(&job, &format!("try again on {}", job_id(of)), n),
            candidate(&job, base, tree, &[change], &[]),
        ];
        if state == OutcomeState::Rejected {
            records.push(verification(&job, vec![check("test", false, None)]));
        }
        records.push(outcome(&job, state, n));
        records
    }

    /// Loads the records through the same assembly the commands use.
    pub fn load(records: &[AnyRecord]) -> Records {
        let dir = tempfile::tempdir().unwrap();
        for record in records {
            let path = dir.path().join(record.canonical_path());
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, record.to_json()).unwrap();
        }
        let loaded = Records::load_dir(dir.path()).unwrap();
        assert!(loaded.invalid.is_empty(), "{:?}", loaded.invalid);
        loaded
    }

    /// Jobs 1.. : `gate` ended `ends` jobs, and each of the first `overruled` of them was retried by a
    /// landed job on a moved base with a different tree and the same change; the others were retried by a
    /// job that failed. Returns the records.
    pub fn chain(gate: &str, first_job: u32, ends: u32, overruled: u32) -> Vec<AnyRecord> {
        let mut records = Vec::new();
        for i in 0..ends {
            let n = first_job + 2 * i;
            let file = format!("src/{gate}-{i}.rs");
            records.extend(rejected_by(n, gate, "a", "b", (&file, 10 + u64::from(i))));
            if i < overruled {
                records.extend(retry(
                    n + 1,
                    n,
                    OutcomeState::Landed,
                    "a",
                    "b",
                    (&file, 10 + u64::from(i)),
                ));
            } else {
                // Failed with no admission rule: it is not a gate of its own.
                records.extend(retry(
                    n + 1,
                    n,
                    OutcomeState::Failed,
                    "a",
                    "c",
                    (&file, 10 + u64::from(i)),
                ));
            }
        }
        records
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::*;
    use super::*;

    fn counts_of(records: &[athena_data::records::AnyRecord]) -> Vec<(String, u32, u32)> {
        gate_counts(&load(records))
            .into_iter()
            .map(|g| (g.gate, g.ends, g.disagreements))
            .collect()
    }

    fn row(gate: &str, ends: u32, disagreements: u32) -> (String, u32, u32) {
        (gate.to_string(), ends, disagreements)
    }

    #[test]
    fn a_gate_that_ended_five_jobs_each_retried_unchanged_has_five_ends_and_five_disagreements() {
        // Same base, same tree: the retry landed exactly what the gate had ended.
        let mut records = Vec::new();
        for i in 0..5 {
            let n = 2 * i + 1;
            let file = format!("src/f{i}.rs");
            records.extend(rejected_by(n, "test", "a", "b", (&file, 5)));
            records.extend(retry(n + 1, n, OutcomeState::Landed, "a", "b", (&file, 5)));
        }
        assert_eq!(counts_of(&records), [row("test", 5, 5)]);
    }

    #[test]
    fn a_gate_whose_ends_were_not_followed_by_an_unchanged_landing_has_no_disagreements() {
        let mut records = Vec::new();
        for i in 0..5 {
            let n = 2 * i + 1;
            let file = format!("src/f{i}.rs");
            records.extend(rejected_by(n, "fmt", "a", "b", (&file, 5)));
            if i < 3 {
                // Landed on the same base but a different tree: a changed landing.
                records.extend(retry(n + 1, n, OutcomeState::Landed, "a", "c", (&file, 5)));
            } else {
                // Rejected again: no landing at all.
                records.extend(retry(
                    n + 1,
                    n,
                    OutcomeState::Rejected,
                    "a",
                    "b",
                    (&file, 5),
                ));
            }
        }
        assert_eq!(counts_of(&records), [row("fmt", 5, 0), row("test", 2, 0)]);
    }

    #[test]
    fn a_moved_base_with_a_different_tree_and_the_same_change_list_is_a_disagreement() {
        let mut records = rejected_by(1, "clippy", "a", "b", ("src/x.rs", 7));
        records.extend(retry(2, 1, OutcomeState::Landed, "d", "e", ("src/x.rs", 7)));
        assert_eq!(counts_of(&records), [row("clippy", 1, 1)]);
    }

    #[test]
    fn a_moved_base_with_a_different_change_list_is_not_a_disagreement() {
        let mut records = rejected_by(1, "clippy", "a", "b", ("src/x.rs", 7));
        records.extend(retry(2, 1, OutcomeState::Landed, "d", "e", ("src/x.rs", 9)));
        let mut other_path = rejected_by(3, "fmt", "a", "b", ("src/x.rs", 7));
        other_path.extend(retry(4, 3, OutcomeState::Landed, "d", "e", ("src/y.rs", 7)));
        records.extend(other_path);
        assert_eq!(counts_of(&records), [row("clippy", 1, 0), row("fmt", 1, 0)]);
    }

    #[test]
    fn the_same_base_with_a_different_tree_is_not_a_disagreement_even_with_equal_change_lists() {
        let mut records = rejected_by(1, "test", "a", "b", ("src/x.rs", 7));
        records.extend(retry(2, 1, OutcomeState::Landed, "a", "c", ("src/x.rs", 7)));
        assert_eq!(counts_of(&records), [row("test", 1, 0)]);
    }

    #[test]
    fn a_rejected_job_with_no_failed_check_counts_under_the_verifier() {
        let job = job_id(1);
        let records = vec![
            request(&job, "Add a thing", 1),
            candidate(&job, "a", "b", &[("src/x.rs", 1)], &[]),
            verification(&job, vec![check("fmt", true, None)]),
            outcome(&job, athena_data::records::OutcomeState::Rejected, 1),
        ];
        assert_eq!(counts_of(&records), [row(VERIFIER_GATE, 1, 0)]);
    }

    #[test]
    fn a_pre_existing_failure_and_a_revert_demonstration_are_not_gates() {
        let job = job_id(1);
        let mut demo = check("demo", false, None);
        demo.revert_for = vec![1];
        let records = vec![
            request(&job, "Add a thing", 1),
            candidate(&job, "a", "b", &[("src/x.rs", 1)], &[]),
            verification(
                &job,
                vec![
                    check("test", false, Some(PRE_EXISTING_NOTE)),
                    demo,
                    check("clippy", false, None),
                ],
            ),
            outcome(&job, athena_data::records::OutcomeState::Rejected, 1),
        ];
        assert_eq!(counts_of(&records), [row("clippy", 1, 0)]);
    }

    #[test]
    fn an_admission_rule_that_ended_a_rejected_or_failed_job_is_a_gate() {
        let one = job_id(1);
        let two = job_id(2);
        let records = vec![
            request(&one, "Add a thing", 1),
            candidate(&one, "a", "b", &[], &["scope"]),
            outcome(&one, athena_data::records::OutcomeState::Rejected, 1),
            request(&two, "Add another thing", 2),
            candidate(&two, "a", "c", &[], &["scope", "banned-word"]),
            outcome(&two, athena_data::records::OutcomeState::Failed, 2),
        ];
        assert_eq!(
            counts_of(&records),
            [row("scope", 2, 0), row("banned-word", 1, 0)]
        );
    }

    #[test]
    fn the_most_overruled_gate_comes_first_then_the_one_that_ended_most_jobs() {
        let mut records = chain("fmt", 1, 4, 0);
        records.extend(chain("test", 20, 3, 2));
        records.extend(chain("clippy", 40, 2, 2));
        assert_eq!(
            counts_of(&records),
            [row("test", 3, 2), row("clippy", 2, 2), row("fmt", 4, 0)]
        );
    }

    #[test]
    fn a_gate_is_counted_once_per_job_and_a_job_with_two_retries_is_overruled_once() {
        let job = job_id(1);
        let mut records = vec![
            request(&job, "Add a thing", 1),
            candidate(&job, "a", "b", &[("src/x.rs", 1)], &[]),
            verification(
                &job,
                vec![check("test", false, None), check("test", false, None)],
            ),
            outcome(&job, athena_data::records::OutcomeState::Rejected, 1),
        ];
        records.extend(retry(2, 1, OutcomeState::Landed, "a", "b", ("src/x.rs", 1)));
        records.extend(retry(3, 1, OutcomeState::Landed, "a", "b", ("src/x.rs", 1)));
        assert_eq!(counts_of(&records), [row("test", 1, 1)]);
    }

    #[test]
    fn a_name_that_cannot_be_written_into_a_prefix_is_left_out() {
        let job = job_id(1);
        let records = vec![
            request(&job, "Add a thing", 1),
            candidate(&job, "a", "b", &[("src/x.rs", 1)], &[]),
            verification(
                &job,
                vec![check("a]b", false, None), check("line\nbreak", false, None)],
            ),
            outcome(&job, athena_data::records::OutcomeState::Rejected, 1),
        ];
        assert!(counts_of(&records).is_empty());
    }

    #[test]
    fn the_prevention_prefix_round_trips_and_only_the_start_counts() {
        let marked = mark_prevention("test", "Loosen it.");
        assert_eq!(marked, "[gate: test] Loosen it.");
        assert_eq!(gate_of_prevention(&marked), Some("test"));
        assert_eq!(gate_of_prevention("Loosen it. [gate: test] later"), None);
        assert_eq!(gate_of_prevention("[gate: test]no space"), None);
        assert_eq!(gate_of_prevention("[gate: ] x"), None);
        assert_eq!(gate_of_prevention("no prefix"), None);
        assert_eq!(prevention_without_gate(&marked), "Loosen it.");
        assert_eq!(prevention_without_gate("no prefix"), "no prefix");
    }

    #[test]
    fn describe_cites_both_counts() {
        let count = GateCount {
            gate: "test".into(),
            ends: 5,
            disagreements: 3,
        };
        assert_eq!(count.describe(), "ended 5 job(s), overruled 3 time(s)");
    }
}
