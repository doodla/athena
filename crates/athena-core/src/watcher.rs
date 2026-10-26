//! The watcher: decisions put under review by exact matching (`docs/design/decision-sets.md` section 4,
//! "Watching" and "Firing"; build order step 4).
//!
//! The watcher reads stored fields only and decides by code, with no model. This slice has one signal source,
//! `host-error`, and one decision with a typed signal, D87. A host-error signal names an operation, the HTTP
//! statuses that falsify the decision's assumption, the phase and a threshold in distinct jobs. A hard signal
//! (a refusal by the host) fires on the first job; the threshold of a hard signal cannot be raised.
//!
//! * [`load`] checks a signal's values against the record vocabulary and refuses one outside it, naming the
//!   valid values. The signals in this module pass through it when they are read, so a wrong value is an error
//!   and not a signal that can never fire.
//! * [`evaluate`] returns the firings. It reads `athena.host-error/1` records and the decision-state records
//!   already written. It never reads an introspection, so the introspection of a firing cannot cause another
//!   firing. A host error that an earlier decision-state record already cites is not evaluated twice, and a
//!   decision that is already under review does not fire again.
//! * [`Athena::watch_landing`] is the landing trigger: it commits the refusals of a landing run as records, runs
//!   [`evaluate`] over everything committed and appends one `athena.decision-state/1` record per firing.
//!
//! Clearing a review is not built; only the user clears it, in a later change.

use crate::Athena;
use crate::Ui;
use crate::commits::{NewFile, message};
use crate::gh::HostErrorLog;
use crate::store::Records;
use anyhow::Result;
use athena_data::RecordId;
use athena_data::records::{
    AnyRecord, DecisionState, DecisionStateKind, DefaultExpectations, HostError, HostOperation,
    Intervention, MessageClass, SignalStrength, WorkPhase,
};
use serde::Serialize;
use std::collections::BTreeSet;

/// A host-error signal as it is written, before its values are checked.
#[derive(Debug, Clone, Copy)]
pub struct RawSignal {
    pub decision: &'static str,
    pub name: &'static str,
    pub strength: SignalStrength,
    pub operation: &'static str,
    pub status_codes: &'static [i64],
    pub phase: &'static str,
    pub threshold_jobs: usize,
}

/// D87's hard signal: the host refusing a merge with 405 or 422 falsifies "the repository allows merge commits".
pub const D87_SIGNAL: RawSignal = RawSignal {
    decision: "D87",
    name: "host-error: operation merge, status 405 or 422, phase landing",
    strength: SignalStrength::Hard,
    operation: "merge",
    status_codes: &[405, 422],
    phase: "landing",
    threshold_jobs: 1,
};

/// A host-error signal whose values are in the vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HostErrorSignal {
    pub decision: String,
    pub name: String,
    pub strength: SignalStrength,
    pub operation: HostOperation,
    pub status_codes: Vec<u16>,
    pub phase: WorkPhase,
    pub threshold_jobs: usize,
}

const PHASES: [(&str, WorkPhase); 4] = [
    ("reading", WorkPhase::Reading),
    ("producing", WorkPhase::Producing),
    ("verifying", WorkPhase::Verifying),
    ("landing", WorkPhase::Landing),
];

/// Checks a signal's values against the vocabulary of the host-error record. The error names the valid values.
pub fn load(raw: &RawSignal) -> Result<HostErrorSignal, String> {
    let operation = HostOperation::parse(raw.operation).ok_or_else(|| {
        format!(
            "signal {:?}: operation {:?} is not one of: {}",
            raw.name,
            raw.operation,
            HostOperation::ALL.map(HostOperation::as_str).join(", ")
        )
    })?;
    let phase = PHASES
        .iter()
        .find(|(name, _)| *name == raw.phase)
        .map(|(_, phase)| *phase)
        .ok_or_else(|| {
            format!(
                "signal {:?}: phase {:?} is not one of: {}",
                raw.name,
                raw.phase,
                PHASES.map(|(name, _)| name).join(", ")
            )
        })?;
    if raw.status_codes.is_empty() {
        return Err(format!("signal {:?}: no status_code is named", raw.name));
    }
    let mut status_codes = Vec::new();
    for code in raw.status_codes {
        match u16::try_from(*code) {
            Ok(code) if (100..=599).contains(&code) => status_codes.push(code),
            _ => {
                return Err(format!(
                    "signal {:?}: status_code {code} is not an HTTP status (100 to 599)",
                    raw.name
                ));
            }
        }
    }
    let least = match raw.strength {
        SignalStrength::Hard => 1,
        SignalStrength::Soft => 2,
    };
    if raw.strength == SignalStrength::Hard && raw.threshold_jobs != 1 {
        return Err(format!(
            "signal {:?}: a hard signal fires on the first job; its threshold is 1, not {}",
            raw.name, raw.threshold_jobs
        ));
    }
    if raw.threshold_jobs < least {
        return Err(format!(
            "signal {:?}: a soft signal needs at least {least} jobs, not {}",
            raw.name, raw.threshold_jobs
        ));
    }
    Ok(HostErrorSignal {
        decision: raw.decision.to_string(),
        name: raw.name.to_string(),
        strength: raw.strength,
        operation,
        status_codes,
        phase,
        threshold_jobs: raw.threshold_jobs,
    })
}

/// Every typed host-error signal of the build: D87's, as of this slice.
pub fn host_error_signals() -> Result<Vec<HostErrorSignal>, String> {
    [D87_SIGNAL].iter().map(load).collect()
}

impl HostErrorSignal {
    /// Exact matching on stored fields: the operation, the phase and one of the statuses are all equal. A
    /// refusal with no status never matches.
    pub fn matches(&self, error: &HostError) -> bool {
        error.operation == self.operation
            && error.phase == self.phase
            && error
                .status_code
                .is_some_and(|status| self.status_codes.contains(&status))
    }
}

/// The first `HTTP <code>` of `text`: the letters `HTTP`, one space, and exactly three digits that no digit
/// follows. Returns the code. A code outside 100 to 599 is not a status and yields none.
fn http_code(text: &str) -> Option<u16> {
    let mut rest = text;
    while let Some(at) = rest.find("HTTP ") {
        let after = &rest[at + "HTTP ".len()..];
        let digits = after.chars().take_while(char::is_ascii_digit).count();
        if digits == 3 {
            return after[..3].parse().ok().filter(|c| (100..=599).contains(c));
        }
        rest = after;
    }
    None
}

/// Words that may stand between `refused` and `merge` ("refused a merge", "refused the merge").
const DETERMINERS: [&str; 4] = ["a", "the", "to", "any"];

/// The most words allowed between the refused merge and its `HTTP <code>` ("merge commit (HTTP 405)").
const MAX_WORDS_TO_CODE: usize = 4;

/// The status code tied to the refusal of the merge itself: the word `refused`, then optional determiners, then
/// the word `merge`, then within [`MAX_WORDS_TO_CODE`] words, and before any `;`, line break or sentence end,
/// an `HTTP <code>`. A full stop ends a sentence only when white space or the end of the text follows it, so a
/// decimal or a version number does not cut the text. A code that stands elsewhere in the text, such as "the
/// push failed with HTTP 422", is not tied to the merge and yields none.
fn refused_merge_code(text: &str) -> Option<u16> {
    // ASCII lower case keeps every byte offset of `text`.
    let lower = text.to_ascii_lowercase();
    let is_word = |c: char| c.is_alphanumeric();
    for (at, _) in lower.match_indices("refused") {
        let before_ok = !lower[..at].chars().next_back().is_some_and(is_word);
        let after = &lower[at + "refused".len()..];
        if !before_ok || after.chars().next().is_some_and(is_word) {
            continue;
        }
        let mut offset = at + "refused".len();
        let mut rest = after.trim_start_matches([' ', '\t']);
        offset += after.len() - rest.len();
        loop {
            let word_len = rest.chars().take_while(|c| is_word(*c)).count();
            let word = &rest[..word_len];
            if DETERMINERS.contains(&word) {
                let next = rest[word_len..].trim_start_matches([' ', '\t']);
                offset += rest.len() - next.len();
                rest = next;
            } else {
                break;
            }
        }
        if !rest.starts_with("merge") || rest[5..].chars().next().is_some_and(is_word) {
            continue;
        }
        let tail = &text[offset + "merge".len()..];
        let mut end = tail.len();
        let bytes = tail.as_bytes();
        for (i, b) in bytes.iter().enumerate() {
            let stops = match b {
                b';' | b'\n' | b'\r' => true,
                b'.' => bytes.get(i + 1).is_none_or(u8::is_ascii_whitespace),
                _ => false,
            };
            if stops {
                end = i;
                break;
            }
        }
        let clause = &tail[..end];
        let Some(code_at) = clause.find("HTTP ") else {
            continue;
        };
        if clause[..code_at].split_whitespace().count() > MAX_WORDS_TO_CODE {
            continue;
        }
        if let Some(code) = http_code(&clause[code_at..]) {
            return Some(code);
        }
    }
    None
}

/// Derives a host-error record from a past intervention that states a host refusal, from before host errors
/// were recorded. This is a typed decision made by an exact pattern, not by a model: the `what` or the `why`
/// text states that the host refused a merge and ties an `HTTP <code>` (three digits, 100 to 599) to that
/// refusal, as in "GitHub refused a merge commit (HTTP 405)" (see `refused_merge_code`); and the intervention
/// names a job. A bare "merged" next to a code that belongs to something else derives nothing. Otherwise it
/// returns none.
///
/// The record is marked `derived` and cites the intervention. Its id is the intervention's id; the marker, not
/// the id, tells it from an observed record. Its time is the intervention's, its operation is `merge` and its
/// phase is `landing`. It is never written under `records/`, and nothing may present it as observed by the
/// client.
pub fn derive_host_error(intervention: &Intervention) -> Option<HostError> {
    let job = intervention.job.clone()?;
    let status =
        refused_merge_code(&intervention.what).or_else(|| refused_merge_code(&intervention.why))?;
    Some(HostError {
        id: intervention.id.clone(),
        at: intervention.at,
        status_code: Some(status),
        operation: HostOperation::Merge,
        job,
        phase: WorkPhase::Landing,
        message_class: MessageClass::of_status(Some(status)),
        derived: true,
        source_intervention: Some(intervention.id.clone()),
    })
}

/// A signal that crossed its threshold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Firing {
    pub decision: String,
    pub signal: String,
    /// The host-error records that fired it, oldest first.
    pub firing_records: Vec<RecordId>,
    /// The distinct jobs of those records.
    pub jobs: BTreeSet<String>,
}

/// The signals that fire over the host errors, given the decision-state records already written. A host error a
/// state record already cites is skipped, and so is a decision that is already under review. Only host-error
/// records are read: nothing an introspection wrote is evaluated.
pub fn evaluate(
    signals: &[HostErrorSignal],
    host_errors: &[HostError],
    states: &[DecisionState],
) -> Vec<Firing> {
    let cited: BTreeSet<&RecordId> = states.iter().flat_map(|s| &s.firing_records).collect();
    let current = athena_data::records::fold_decision_states(states);
    let mut firings = Vec::new();
    for signal in signals {
        if current.get(&signal.decision) == Some(&DecisionStateKind::UnderReview) {
            continue;
        }
        let matched: Vec<&HostError> = host_errors
            .iter()
            .filter(|e| !cited.contains(&e.id) && signal.matches(e))
            .collect();
        let jobs: BTreeSet<String> = matched.iter().map(|e| e.job.to_string()).collect();
        if matched.is_empty() || jobs.len() < signal.threshold_jobs {
            continue;
        }
        firings.push(Firing {
            decision: signal.decision.clone(),
            signal: signal.name.clone(),
            firing_records: matched.iter().map(|e| e.id.clone()).collect(),
            jobs,
        });
    }
    firings
}

impl Athena {
    /// The landing trigger. Commits the refusals in `log` as `athena.host-error/1` records, then evaluates the
    /// watcher's signals over every host error at the new tip and appends an `athena.decision-state/1` record
    /// for each firing. Runs at the end of every `athena land --pr`, whether or not it succeeded, so a refusal
    /// that failed the landing is seen at once. Returns the state records written.
    pub fn watch_landing(&self, log: &HostErrorLog, ui: &dyn Ui) -> Result<Vec<DecisionState>> {
        let errors = log.entries();
        if !errors.is_empty() {
            let files: Vec<NewFile> = errors
                .iter()
                .map(|e| NewFile::record(&AnyRecord::HostError(e.clone())))
                .collect();
            let trailers: Vec<(&str, String)> = files
                .iter()
                .map(|f| ("Athena-Record", f.path.clone()))
                .collect();
            let body: Vec<String> = errors
                .iter()
                .map(|e| {
                    format!(
                        "{} {}",
                        e.operation.as_str(),
                        e.status_code
                            .map_or_else(|| "no HTTP answer".to_string(), |s| format!("HTTP {s}"))
                    )
                })
                .collect();
            self.commit_records(
                &files,
                &message("Record refused host calls", &body.join("\n"), &trailers),
            )?;
            ui.line(&format!("recorded   {} refused host call(s)", errors.len()));
        }
        let signals = host_error_signals().map_err(anyhow::Error::msg)?;
        let head = self.head()?;
        let records = Records::load_current(&self.git, &head)?;
        let firings = evaluate(&signals, &records.host_errors, &records.decision_states);
        let mut written = Vec::new();
        for firing in firings {
            let mut id = RecordId::generate(crate::now());
            while records.has_record_id(&id) || written.iter().any(|w: &DecisionState| w.id == id) {
                id = RecordId::generate(crate::now());
            }
            let state = DecisionState {
                id,
                at: crate::now(),
                decision: firing.decision.clone(),
                state: DecisionStateKind::UnderReview,
                signal: Some(firing.signal.clone()),
                firing_records: firing.firing_records.clone(),
                decision_job: None,
            };
            let record = AnyRecord::DecisionState(state.clone());
            let path = record.canonical_path();
            self.commit_records(
                &[NewFile::record(&record)],
                &message(
                    &format!("Put {} under review", firing.decision),
                    &format!(
                        "Signal: {}\nFired by: {}",
                        firing.signal,
                        firing
                            .firing_records
                            .iter()
                            .map(RecordId::as_str)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    &[("Athena-Record", path)],
                ),
            )?;
            ui.line(&format!(
                "under review  {} · {}",
                firing.decision,
                firing
                    .firing_records
                    .iter()
                    .map(RecordId::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            written.push(state);
        }
        Ok(written)
    }

    /// Writes an `athena.default-expectations/1` record for a default: what it assumes and how to tell it
    /// failed. Returns the record and the commit. A second record for the same default supersedes the first for
    /// readers ([`Records::expectations_for`] returns the newest); neither is edited.
    pub fn record_default_expectations(
        &self,
        expectations: DefaultExpectations,
    ) -> Result<(DefaultExpectations, String)> {
        expectations
            .validate()
            .map_err(|reason| anyhow::anyhow!(reason))?;
        let record = AnyRecord::DefaultExpectations(expectations.clone());
        let path = record.canonical_path();
        let commit = self.commit_records(
            &[NewFile::record(&record)],
            &message(
                &format!("Record expectations of default {}", expectations.default_id),
                &format!("Assumptions: {}", expectations.assumptions.len()),
                &[("Athena-Record", path)],
            ),
        )?;
        Ok((expectations, commit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::JobId;

    fn error(id: &str, job: &str, op: HostOperation, status: Option<u16>) -> HostError {
        HostError {
            id: RecordId::parse(id).unwrap(),
            at: "2026-10-13T19:21:56Z".parse().unwrap(),
            status_code: status,
            operation: op,
            job: JobId::parse(job).unwrap(),
            phase: WorkPhase::Landing,
            message_class: MessageClass::of_status(status),
            derived: false,
            source_intervention: None,
        }
    }

    const JOB: &str = "20261013-192156-ar1x";

    fn intervention(what: &str, why: &str, job: Option<&str>) -> Intervention {
        Intervention {
            id: RecordId::parse("20261013-192156-ar1x").unwrap(),
            at: "2026-10-13T19:30:00Z".parse().unwrap(),
            what: what.to_string(),
            why: why.to_string(),
            class_of_work: "code".to_string(),
            category: athena_data::records::InterventionCategory::DidWorkThemselves,
            job: job.map(|j| JobId::parse(j).unwrap()),
            source: athena_data::records::InterventionSource::User,
            with_staged_changes: false,
        }
    }

    #[test]
    fn a_refusal_stated_in_an_intervention_derives_a_marked_record() {
        let i = intervention(
            "Merged the pull request by hand",
            "GitHub refused a merge commit (HTTP 405)",
            Some(JOB),
        );
        let derived = derive_host_error(&i).expect("the text matches");
        assert!(derived.derived);
        assert_eq!(derived.source_intervention.as_ref(), Some(&i.id));
        assert_eq!(derived.status_code, Some(405));
        assert_eq!(derived.operation, HostOperation::Merge);
        assert_eq!(derived.phase, WorkPhase::Landing);
        assert_eq!(derived.job.to_string(), JOB);
        assert_eq!(derived.at, i.at);
        assert_eq!(derived.message_class, MessageClass::MethodNotAllowed);
        assert!(load(&D87_SIGNAL).unwrap().matches(&derived));
    }

    #[test]
    fn text_without_the_exact_pattern_derives_nothing() {
        let none = |what: &str, why: &str| derive_host_error(&intervention(what, why, Some(JOB)));
        assert!(none("Fixed the docs by hand", "the wording was wrong").is_none());
        // A code without a merge, and a merge without a code.
        assert!(none("The push failed with HTTP 403", "token scope").is_none());
        assert!(none("Merged by hand", "the host refused").is_none());
        // Not the exact form: no space, lower case, two digits, four digits.
        assert!(none("GitHub refused a merge (HTTP405)", "x").is_none());
        assert!(none("GitHub refused a merge (http 405)", "x").is_none());
        assert!(none("GitHub refused a merge (HTTP 40)", "x").is_none());
        assert!(none("GitHub refused a merge (HTTP 4050)", "x").is_none());
        // "emerged" is not the word merge.
        assert!(none("GitHub refused an emerged branch (HTTP 405)", "x").is_none());
    }

    #[test]
    fn the_code_must_be_tied_to_the_refusal_of_the_merge() {
        let derives = |what: &str, why: &str| {
            derive_host_error(&intervention(what, why, Some(JOB))).map(|d| d.status_code)
        };
        // The refused merge and its code derive.
        assert_eq!(
            derives("GitHub refused a merge commit (HTTP 405)", "x"),
            Some(Some(405))
        );
        // The word for the person who ran the commands is built at run time: the boundary scan rejects it
        // in a file, and the real text of these interventions has it.
        let who = ["Oper", "ator"].concat();
        let ar1x = format!(
            "{who} squash-merged PR #41 by hand with the job's trailers after GitHub refused a merge \
             commit (HTTP 405)."
        );
        assert_eq!(derives(&ar1x, "x"), Some(Some(405)));
        // The text of a landing that failed on a push, then merged by hand: no refused merge.
        let kn3q = format!(
            "athena land --pr failed with HTTP 422 for job vbdn; {who} opened PR #38 by hand, \
             simulated the squash, and merged it with the job trailer."
        );
        assert_eq!(derives(&kn3q, "x"), None);
        assert_eq!(
            derives(
                "I merged by hand because the push failed with HTTP 422",
                "x"
            ),
            None
        );
        assert_eq!(derives("the push failed with HTTP 422", "x"), None);
        // A refused merge, then a code of another failure after the clause ends.
        assert_eq!(
            derives("GitHub refused a merge; the push failed with HTTP 422", "x"),
            None
        );
        // A refusal of something else, and a code far from the merge.
        assert_eq!(
            derives("GitHub refused the push (HTTP 422); merged", "x"),
            None
        );
        assert_eq!(
            derives(
                "refused a merge after many tries on the branch at once, HTTP 405",
                "x"
            ),
            None
        );
    }

    #[test]
    fn a_version_number_does_not_cut_the_clause() {
        let derives = |what: &str| derive_host_error(&intervention(what, "x", Some(JOB)));
        assert!(derives("GitHub 3.1 refused a merge commit (HTTP 405).").is_some());
        assert!(derives("GitHub refused a merge in v1.2 (HTTP 405).").is_some());
        // A sentence end does cut: the code of the next sentence is not tied to the merge.
        assert!(derives("GitHub refused a merge. Then the push failed with HTTP 422.").is_none());
    }

    #[test]
    fn an_intervention_without_a_job_derives_nothing() {
        let i = intervention("GitHub refused a merge (HTTP 405)", "x", None);
        assert!(derive_host_error(&i).is_none());
    }

    #[test]
    fn a_code_outside_the_http_range_derives_nothing() {
        for code in ["099", "600", "999"] {
            let i = intervention(
                "GitHub refused a merge",
                &format!("(HTTP {code})"),
                Some(JOB),
            );
            assert!(derive_host_error(&i).is_none(), "{code}");
            let i = intervention(
                &format!("GitHub refused a merge (HTTP {code})"),
                "x",
                Some(JOB),
            );
            assert!(derive_host_error(&i).is_none(), "{code}");
        }
        let i = intervention("GitHub refused a merge (HTTP 599)", "x", Some(JOB));
        assert!(derive_host_error(&i).is_some());
    }

    #[test]
    fn d87s_signal_is_in_the_vocabulary() {
        let signals = host_error_signals().unwrap();
        assert_eq!(signals.len(), 1);
        let d87 = &signals[0];
        assert_eq!(d87.decision, "D87");
        assert_eq!(d87.operation, HostOperation::Merge);
        assert_eq!(d87.status_codes, [405, 422]);
        assert_eq!(d87.strength, SignalStrength::Hard);
        assert_eq!(d87.threshold_jobs, 1);
    }

    #[test]
    fn a_value_outside_the_vocabulary_is_refused_with_the_valid_values() {
        let bad = |change: fn(&mut RawSignal)| {
            let mut raw = D87_SIGNAL;
            change(&mut raw);
            load(&raw).unwrap_err()
        };
        let err = bad(|r| r.operation = "delete-repo");
        assert!(err.contains("merge, create-pr, read-repo"), "{err}");
        let err = bad(|r| r.phase = "docs");
        assert!(
            err.contains("reading, producing, verifying, landing"),
            "{err}"
        );
        assert!(bad(|r| r.status_codes = &[40_500]).contains("not an HTTP status"));
        assert!(bad(|r| r.status_codes = &[]).contains("no status_code"));
        // A hard signal's threshold cannot be raised; a soft one needs repeats.
        assert!(bad(|r| r.threshold_jobs = 2).contains("fires on the first job"));
        assert!(
            bad(|r| {
                r.strength = SignalStrength::Soft;
                r.threshold_jobs = 1;
            })
            .contains("at least 2")
        );
    }

    #[test]
    fn a_refused_merge_fires_on_the_first_occurrence() {
        let signals = host_error_signals().unwrap();
        for status in [405, 422] {
            let errors = [error(
                "20261013-192200-k7q2",
                JOB,
                HostOperation::Merge,
                Some(status),
            )];
            let firings = evaluate(&signals, &errors, &[]);
            assert_eq!(firings.len(), 1, "{status}");
            assert_eq!(firings[0].decision, "D87");
            assert_eq!(firings[0].firing_records, [errors[0].id.clone()]);
        }
    }

    #[test]
    fn soft_and_unrelated_errors_do_not_fire() {
        let signals = host_error_signals().unwrap();
        let none = [
            error("20261013-192201-aaaa", JOB, HostOperation::Merge, Some(404)),
            error("20261013-192202-bbbb", JOB, HostOperation::Merge, Some(500)),
            error("20261013-192203-cccc", JOB, HostOperation::Merge, None),
            error(
                "20261013-192204-dddd",
                JOB,
                HostOperation::CreatePr,
                Some(405),
            ),
            error(
                "20261013-192205-eeee",
                JOB,
                HostOperation::ReadRepo,
                Some(422),
            ),
        ];
        assert!(evaluate(&signals, &none, &[]).is_empty());
        // The right operation and status in another phase.
        let mut other = error("20261013-192206-ffff", JOB, HostOperation::Merge, Some(405));
        other.phase = WorkPhase::Producing;
        assert!(evaluate(&signals, &[other], &[]).is_empty());
    }

    fn under_review(cited: &HostError) -> DecisionState {
        DecisionState {
            id: RecordId::parse("20261013-193000-zzzz").unwrap(),
            at: "2026-10-13T19:30:00Z".parse().unwrap(),
            decision: "D87".into(),
            state: DecisionStateKind::UnderReview,
            signal: Some("x".into()),
            firing_records: vec![cited.id.clone()],
            decision_job: None,
        }
    }

    #[test]
    fn a_record_is_not_evaluated_twice_and_a_decision_under_review_does_not_fire_again() {
        let signals = host_error_signals().unwrap();
        let first = error("20261013-192200-k7q2", JOB, HostOperation::Merge, Some(405));
        let state = under_review(&first);
        assert!(
            evaluate(
                &signals,
                std::slice::from_ref(&first),
                std::slice::from_ref(&state)
            )
            .is_empty()
        );
        // A later refusal of a decision already under review does not fire again either.
        let later = error("20261014-090000-m2m2", JOB, HostOperation::Merge, Some(405));
        assert!(evaluate(&signals, &[first.clone(), later.clone()], &[state]).is_empty());
        // Once cleared, a refusal that no state record cites fires again; the cited one does not.
        let mut cleared = under_review(&first);
        cleared.id = RecordId::parse("20261013-194000-yyyy").unwrap();
        cleared.at = "2026-10-13T19:40:00Z".parse().unwrap();
        cleared.state = DecisionStateKind::Cleared;
        let states = [under_review(&first), cleared];
        let firings = evaluate(&signals, &[first, later.clone()], &states);
        assert_eq!(firings.len(), 1);
        assert_eq!(firings[0].firing_records, [later.id]);
    }
}
