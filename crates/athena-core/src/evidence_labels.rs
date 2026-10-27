//! Class labels derived from evidence (`BRIEF.md`: the labels come from what happened afterwards, not from
//! anyone's opinion; `docs/design/defaults-loop.md` section 10.2).
//!
//! `athena score-jev` needs labeled introspections, and a person's `athena review` was the only way to get
//! one. This module derives a label from three kinds of later event, each recorded and each cited by id:
//!
//! 1. **Retry outcome** ([`EvidenceKind::RetryOutcome`]): a later job whose request is the introspection's
//!    proposed request (or `athena retry` of the introspection's job, when it is the record that seeds that
//!    retry) landed, or was rejected or failed and its own introspection has the same failure class (it failed
//!    for the same cause). A retry that failed for another class says nothing about this one.
//! 2. **Recurrence after prevention** ([`EvidenceKind::RecurrenceAfterPrevention`]): an accepted defaults
//!    proposal for the class was applied by a job that landed after the introspection, and then the class either
//!    kept recurring (a later recorded introspection in the class) or stopped (none, over at least
//!    [`STOPPED_MIN_JOBS`] jobs that closed after the landing).
//! 3. **Same-cause findings** ([`EvidenceKind::SameCauseFindings`]): a finding of the job's plan critic, or of
//!    its verifier, shares enough words with the introspection's cause to name the same cause. When the
//!    introspection was written for a rejection or a failure the model was shown the verifier's blocker and
//!    major findings, so those do not count as independent evidence; critic findings and the verifier's minor
//!    findings do.
//!
//! **What a label says.** Each kind can only support the class the introspection recorded. The label names
//! that class and cites the records; it never invents another class and never says `none`. If the kinds ever
//! named different classes ([`resolve`]), the case would be left unlabeled. An introspection with no evidence is
//! left unlabeled (it stays *unreviewed* in `athena score-jev`).
//!
//! **Records, not edits.** Derivation reads the records at the branch tip and appends new
//! `athena.class-label/1` records with `labeled_by: evidence` and a `basis`. No record is edited or deleted.
//! An introspection that already has a person's or a stronger model's label is skipped, because that label wins
//! whichever is newer ([`crate::labeled::newest_labels`]). One that already has an evidence label with the
//! same class and basis is skipped too, so a rerun writes nothing.

use crate::Athena;
use crate::commits::{NewFile, message};
use crate::labeled::newest_labels;
use crate::proposals;
use crate::reading::{retried_job, retry_introspection};
use crate::store::{JobRecords, JobState, Records};
use anyhow::Result;
use athena_data::RecordId;
use athena_data::records::{
    AnyRecord, ClassLabel, EvidenceKind, Finding, FindingSource, IntrospectionRecord,
    IntrospectionStatus, IntrospectionTrigger, LabelBasis, Labeler, OutcomeState, ProposalStatus,
    Severity,
};
use jiff::Timestamp;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// The fewest jobs that must have closed after a prevention landed before "the class stopped" counts as
/// evidence. An **estimate**: with fewer jobs the class may simply not have had a chance to recur.
pub const STOPPED_MIN_JOBS: usize = 5;

/// A finding names the introspection's cause when it shares at least this many words (of five letters or more)
/// with the cause text and at least [`CAUSE_MIN_SHARE`] of the cause's words. Estimates; word overlap is a
/// plain stand-in for "names the same cause", and the basis shows the finding so a reader can judge it.
const CAUSE_MIN_SHARED_WORDS: usize = 3;
const CAUSE_MIN_SHARE: f64 = 0.25;

/// The longest finding summary quoted in a basis, in characters.
const QUOTE_CHARS: usize = 100;

/// The most later introspections cited by a "kept recurring" basis.
const CITED_RECURRENCES: usize = 3;

/// What one derivation run found, over the recorded introspections that name a class. Each such introspection
/// is counted once: `recorded == reviewed + already_derived + derived + no_evidence + conflicting`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Summary {
    /// Recorded introspections that name a failure class (only these can be labeled).
    pub recorded: usize,
    /// Skipped: a person or a stronger model labeled it, and that label wins.
    pub reviewed: usize,
    /// Skipped: an evidence label with the same class and basis already exists.
    pub already_derived: usize,
    /// Labels written by this run.
    pub derived: usize,
    /// Of those, how many cite each kind of evidence (a label citing two kinds counts in both).
    pub by_kind: BTreeMap<String, usize>,
    /// Left unlabeled: no evidence of any kind.
    pub no_evidence: usize,
    /// Left unlabeled: the kinds of evidence named different classes.
    pub conflicting: usize,
    /// The commit that holds the new labels; none when nothing was written.
    pub commit: Option<String>,
}

/// One kind of evidence pointing at one class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vote {
    pub class: RecordId,
    pub basis: LabelBasis,
}

/// What the votes for one introspection come to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Label the introspection with this class, citing every vote.
    Label {
        class: RecordId,
        basis: Vec<LabelBasis>,
    },
    /// No kind of evidence: leave it unlabeled.
    NoEvidence,
    /// The votes named different classes: leave it unlabeled rather than guess.
    Conflict,
}

/// Combines the votes on one introspection. Votes that agree on the class are all cited, one entry per kind;
/// votes that name different classes leave the case unlabeled.
pub fn resolve(mut votes: Vec<Vote>) -> Decision {
    let Some(first) = votes.first() else {
        return Decision::NoEvidence;
    };
    let class = first.class.clone();
    if votes.iter().any(|v| v.class != class) {
        return Decision::Conflict;
    }
    votes.sort_by_key(|v| v.basis.kind);
    Decision::Label {
        class,
        basis: votes.into_iter().map(|v| v.basis).collect(),
    }
}

/// The result of reading the records: the labels to append and what was found.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Derivation {
    pub labels: Vec<ClassLabel>,
    pub summary: Summary,
}

fn one_line(text: &str, max: usize) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    crate::truncate_at_word(&flat, max)
}

fn words(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 5)
        .map(str::to_lowercase)
        .collect()
}

fn closed_state(job: &JobRecords) -> Option<OutcomeState> {
    match job.state() {
        JobState::Closed(state) => Some(state),
        _ => None,
    }
}

// ---------------------------------------------------------------------------------------------------------
// The three kinds of evidence
// ---------------------------------------------------------------------------------------------------------

/// Evidence 1: the first later job that carried the proposed request and closed as landed, rejected or failed.
fn retry_vote(records: &Records, i: &IntrospectionRecord, class: &RecordId) -> Option<Vote> {
    let source = i.job.as_ref()?;
    let seeds_retry = records
        .jobs
        .get(source.as_str())
        .and_then(|j| retry_introspection(records, j))
        .is_some_and(|seed| seed.id == i.id);
    let proposed = i.proposed_request.as_deref().map(str::trim);
    let mut retries: Vec<&JobRecords> = records
        .jobs
        .values()
        .filter(|j| j.id() != source)
        .filter(|j| {
            let Some(request) = &j.request else {
                return false;
            };
            request.received_at >= i.at
                && ((seeds_retry && retried_job(records, &request.text).as_ref() == Some(source))
                    || proposed.is_some_and(|p| !p.is_empty() && request.text.trim() == p))
        })
        .collect();
    retries.sort_by(|a, b| a.id().as_str().cmp(b.id().as_str()));
    for retry in retries {
        let cited = |extra: Option<&IntrospectionRecord>| {
            let mut ids = vec![i.id.to_string(), retry.id().to_string()];
            ids.extend(extra.map(|e| e.id.to_string()));
            ids
        };
        let Some(state) = closed_state(retry) else {
            continue;
        };
        match state {
            OutcomeState::Landed => {
                return Some(Vote {
                    class: class.clone(),
                    basis: LabelBasis {
                        kind: EvidenceKind::RetryOutcome,
                        records: cited(None),
                        detail: format!(
                            "The retry {} that followed the proposed request and prevention landed.",
                            retry.id()
                        ),
                    },
                });
            }
            OutcomeState::Rejected | OutcomeState::Failed => {
                let again = records.introspections.iter().find(|r| {
                    r.job.as_ref() == Some(retry.id())
                        && r.status == IntrospectionStatus::Recorded
                        && matches!(
                            r.trigger,
                            IntrospectionTrigger::Rejection | IntrospectionTrigger::Failure
                        )
                        && r.failure_class.as_ref() == Some(class)
                });
                if let Some(again) = again {
                    return Some(Vote {
                        class: class.clone(),
                        basis: LabelBasis {
                            kind: EvidenceKind::RetryOutcome,
                            records: cited(Some(again)),
                            detail: format!(
                                "The retry {} failed for the same cause: its introspection {} names the same class.",
                                retry.id(),
                                again.id
                            ),
                        },
                    });
                }
            }
            _ => {}
        }
    }
    None
}

/// When a job's landing closed, if it landed.
fn landed_at(job: &JobRecords) -> Option<Timestamp> {
    (closed_state(job)? == OutcomeState::Landed)
        .then(|| {
            job.later_outcome
                .as_ref()
                .or(job.outcome.as_ref())
                .map(|o| o.at)
        })
        .flatten()
}

/// Evidence 2: the first accepted proposal for the class that landed after the introspection, and whether the
/// class then stopped or kept recurring.
fn recurrence_vote(records: &Records, i: &IntrospectionRecord, class: &RecordId) -> Option<Vote> {
    let (proposal, applied, landed) = proposals::current(&records.proposals)
        .into_iter()
        .filter(|p| &p.failure_class == class)
        .filter_map(|p| match &p.status {
            ProposalStatus::Accepted { job } => {
                let at = landed_at(records.jobs.get(job.as_str())?)?;
                Some((p, job, at))
            }
            _ => None,
        })
        .filter(|(_, _, at)| *at > i.at)
        .min_by_key(|(p, _, at)| (*at, p.record.clone()))?;
    let later: Vec<&IntrospectionRecord> = records
        .introspections
        .iter()
        .filter(|r| {
            r.id != i.id
                && r.at > landed
                && r.status == IntrospectionStatus::Recorded
                && r.failure_class.as_ref() == Some(class)
        })
        .collect();
    let mut cited = vec![
        i.id.to_string(),
        proposal.record.to_string(),
        applied.to_string(),
    ];
    let detail = if later.is_empty() {
        let closed = records
            .jobs
            .values()
            .filter(|j| j.id().is_request() && closed_state(j).is_some())
            .filter(|j| j.outcome.as_ref().is_some_and(|o| o.at > landed))
            .count();
        if closed < STOPPED_MIN_JOBS {
            return None;
        }
        format!(
            "After the prevention in proposal {} landed (job {applied}), the class did not recur over at least {STOPPED_MIN_JOBS} closed jobs.",
            proposal.record
        )
    } else {
        cited.extend(
            later
                .iter()
                .take(CITED_RECURRENCES)
                .map(|r| r.id.to_string()),
        );
        format!(
            "After the prevention in proposal {} landed (job {applied}), the class recurred in {} later introspections.",
            proposal.record,
            later.len()
        )
    };
    Some(Vote {
        class: class.clone(),
        basis: LabelBasis {
            kind: EvidenceKind::RecurrenceAfterPrevention,
            records: cited,
            detail,
        },
    })
}

/// Evidence 3: a critic finding, or a verifier finding the introspection was not shown, that names the cause.
fn findings_vote(records: &Records, i: &IntrospectionRecord, class: &RecordId) -> Option<Vote> {
    let job = records.jobs.get(i.job.as_ref()?.as_str())?;
    let cause = words(i.cause.as_deref()?);
    if cause.len() < CAUSE_MIN_SHARED_WORDS {
        return None;
    }
    let was_shown_blockers = matches!(
        i.trigger,
        IntrospectionTrigger::Rejection | IntrospectionTrigger::Failure
    );
    let usable = |f: &Finding| f.severity != Severity::Nit && f.source != FindingSource::Athena;
    let critic = job.critiques.iter().flat_map(|c| {
        c.findings
            .iter()
            .filter(|f| f.source == FindingSource::Critic)
            .map(move |f| (format!("critique round {}", c.round), f))
    });
    let verifier = job.verification.iter().flat_map(|v| {
        v.findings
            .iter()
            .filter(|f| f.source == FindingSource::Verifier)
            .filter(|f| !(was_shown_blockers && crate::verify::finding_blocks(f)))
            .map(|f| ("verification".to_string(), f))
    });
    let best = critic
        .chain(verifier)
        .filter(|(_, f)| usable(f))
        .map(|(whence, f)| {
            let shared = words(&format!("{} {}", f.summary, f.evidence))
                .intersection(&cause)
                .count();
            (shared, whence, f)
        })
        .filter(|(shared, _, _)| {
            *shared >= CAUSE_MIN_SHARED_WORDS
                && *shared as f64 / cause.len() as f64 >= CAUSE_MIN_SHARE
        })
        .max_by_key(|(shared, whence, _)| (*shared, std::cmp::Reverse(whence.clone())))?;
    let (shared, whence, finding) = best;
    Some(Vote {
        class: class.clone(),
        basis: LabelBasis {
            kind: EvidenceKind::SameCauseFindings,
            records: vec![i.id.to_string(), job.id().to_string()],
            detail: format!(
                "The {whence} {:?} finding names the same cause ({shared} of {} cause words): {}",
                finding.severity,
                cause.len(),
                one_line(&finding.summary, QUOTE_CHARS)
            ),
        },
    })
}

// ---------------------------------------------------------------------------------------------------------
// Deriving
// ---------------------------------------------------------------------------------------------------------

/// Reads `records` and returns the labels to append, with `now` as their time. Nothing is written, and no
/// existing record is touched. A rerun over the records plus the returned labels returns none.
pub fn derive(records: &Records, now: Timestamp) -> Derivation {
    let counting = newest_labels(&records.class_labels);
    let mut taken: BTreeSet<RecordId> = records.class_labels.iter().map(|l| l.id.clone()).collect();
    let mut out = Derivation::default();
    for i in &records.introspections {
        let (IntrospectionStatus::Recorded, Some(class)) = (i.status, &i.failure_class) else {
            continue;
        };
        out.summary.recorded += 1;
        let existing = counting.get(i.id.as_str());
        if existing.is_some_and(|l| l.labeled_by.is_judgement()) {
            out.summary.reviewed += 1;
            continue;
        }
        let votes: Vec<Vote> = [
            retry_vote(records, i, class),
            recurrence_vote(records, i, class),
            findings_vote(records, i, class),
        ]
        .into_iter()
        .flatten()
        .collect();
        match resolve(votes) {
            Decision::NoEvidence if existing.is_some() => out.summary.already_derived += 1,
            Decision::NoEvidence => out.summary.no_evidence += 1,
            Decision::Conflict if existing.is_some() => out.summary.already_derived += 1,
            Decision::Conflict => out.summary.conflicting += 1,
            Decision::Label { class, basis } => {
                if existing.is_some_and(|l| l.class.as_ref() == Some(&class) && l.basis == basis) {
                    out.summary.already_derived += 1;
                    continue;
                }
                let id = loop {
                    let id = RecordId::generate(now);
                    if !records.has_record_id(&id) && !taken.contains(&id) {
                        break id;
                    }
                };
                taken.insert(id.clone());
                for kind in basis.iter().map(|b| b.kind).collect::<BTreeSet<_>>() {
                    *out.summary
                        .by_kind
                        .entry(kind.as_str().to_string())
                        .or_default() += 1;
                }
                out.summary.derived += 1;
                out.labels.push(ClassLabel {
                    id,
                    at: now,
                    introspection: i.id.clone(),
                    class: Some(class),
                    labeled_by: Labeler::Evidence,
                    is_repeat: None,
                    note: None,
                    basis,
                });
            }
        }
    }
    out
}

impl Athena {
    /// The evidence-label derivation. Athena runs it at the start of every drain and in each sweep, and
    /// `athena score-jev --derive` runs it on request. It derives the evidence labels from the records at the branch tip and commits
    /// them as new `athena.class-label/1` records, in one commit. Nothing is committed when there is nothing
    /// new, and no existing record is edited.
    pub fn derive_labels(&self) -> Result<Summary> {
        let head = self.head()?;
        let records = Records::load(&self.git, &head)?;
        let Derivation {
            labels,
            mut summary,
        } = derive(&records, crate::now());
        if labels.is_empty() {
            return Ok(summary);
        }
        let mut files = Vec::new();
        let mut trailers = Vec::new();
        for label in &labels {
            let record = AnyRecord::ClassLabel(label.clone());
            trailers.push(("Athena-Record", record.canonical_path()));
            files.push(NewFile::record(&record));
        }
        let commit = self.commit_records(
            &files,
            &message(
                &format!("Derive {} class labels from evidence", labels.len()),
                "Class labels derived from what happened after each introspection (the retry that followed, recurrence after a prevention landed, findings on the same job). Each cites the records it rests on. They are new records: nothing is edited, and a label from a person outranks them.",
                &trailers,
            ),
        )?;
        summary.commit = Some(commit);
        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::JobId;
    use athena_data::records::{
        Critique, CritiqueVerdict, Outcome, PreventionKind, ProposalKind, ProposalRecord,
        ProposalSource, ProposalVersion, Request, Verdict, Verification,
    };

    const C: &str = "20261001-000000-c001";
    const D: &str = "20261001-000000-c002";

    fn ts(minute: u32) -> Timestamp {
        format!("2026-10-01T{:02}:{:02}:00Z", minute / 60, minute % 60)
            .parse()
            .unwrap()
    }

    fn job_id(name: &str) -> JobId {
        JobId::parse(&format!("20261001-000000-{name}")).unwrap()
    }

    fn rid(minute: u32, n: u32) -> RecordId {
        RecordId::from_parts(ts(minute), n)
    }

    /// A recorded introspection `n` of `class` for job `job`, made at `minute`.
    fn intro(
        n: u32,
        minute: u32,
        job: &str,
        trigger: IntrospectionTrigger,
        class: &str,
    ) -> IntrospectionRecord {
        IntrospectionRecord {
            id: rid(minute, n),
            at: ts(minute),
            trigger,
            job: Some(job_id(job)),
            intervention: (!matches!(
                trigger,
                IntrospectionTrigger::Rejection | IntrospectionTrigger::Failure
            ))
            .then(|| rid(minute, 500 + n)),
            class_of_work: "code".into(),
            category: "rejected".into(),
            what_happened: Some("A field was added with no reader.".into()),
            failure_class: Some(RecordId::parse(class).unwrap()),
            new_class: false,
            earlier_cases: vec![],
            decisions_matched: vec![],
            cause: Some("Nothing connects a new configuration field to its reader module.".into()),
            prevention_kind: Some(PreventionKind::Default),
            prevention: Some("Add the reader.".into()),
            proposed_request: Some(format!("Add the reader for case {n}.")),
            evidence: None,
            status: IntrospectionStatus::Recorded,
            failure: None,
            flags: vec![],
            dropped_ids: vec![],
            planning_stopped: false,
            from_jev: false,
            jev: None,
        }
    }

    fn outcome(job: &str, state: OutcomeState, minute: u32) -> Outcome {
        Outcome {
            job: job_id(job),
            state,
            reason: "test".into(),
            at: ts(minute),
            totals: None,
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        }
    }

    /// Adds job `name` with a request of `text` at `minute`, closed as `state` when given.
    fn add_job(
        records: &mut Records,
        name: &str,
        text: &str,
        minute: u32,
        state: Option<OutcomeState>,
    ) {
        let id = job_id(name);
        let job = records.jobs.entry(id.as_str().to_string()).or_default();
        job.id = Some(id.clone());
        job.request = Some(Request {
            job: id.clone(),
            text: text.into(),
            received_at: ts(minute),
            channel: "cli".into(),
        });
        job.outcome = state.map(|s| outcome(name, s, minute + 1));
    }

    fn finding(source: FindingSource, severity: Severity, summary: &str) -> Finding {
        Finding {
            severity,
            summary: summary.into(),
            evidence: "Read the diff.".into(),
            location: None,
            source,
        }
    }

    fn proposal(class: &str, job: &str) -> ProposalRecord {
        ProposalRecord {
            record: rid(20, 77),
            at: ts(20),
            id: "d-reader".into(),
            kind: ProposalKind::Add,
            statement: "Add readers with fields.".into(),
            applies_to: vec!["code".into()],
            version: ProposalVersion { from: 1, to: 2 },
            sources: vec![ProposalSource {
                introspection: rid(1, 1),
                intervention: None,
                job: job_id("j001"),
                at: ts(1),
            }],
            failure_class: RecordId::parse(class).unwrap(),
            status: ProposalStatus::Accepted { job: job_id(job) },
            candidate_sha256: "a".repeat(64),
            notes: String::new(),
        }
    }

    fn basis_kinds(label: &ClassLabel) -> Vec<EvidenceKind> {
        label.basis.iter().map(|b| b.kind).collect()
    }

    fn label_of(d: &Derivation, n: u32, minute: u32) -> Option<&ClassLabel> {
        d.labels.iter().find(|l| l.introspection == rid(minute, n))
    }

    #[test]
    fn a_retry_that_followed_the_proposed_request_and_landed_gives_a_label_citing_it() {
        let mut records = Records::default();
        let case = intro(1, 1, "j001", IntrospectionTrigger::Rejection, C);
        let proposed = case.proposed_request.clone().unwrap();
        records.introspections.push(case);
        add_job(
            &mut records,
            "j001",
            "Add a field.",
            0,
            Some(OutcomeState::Rejected),
        );
        // The user sent the proposed request; it landed.
        add_job(
            &mut records,
            "j002",
            &proposed,
            5,
            Some(OutcomeState::Landed),
        );
        let d = derive(&records, ts(100));
        let label = label_of(&d, 1, 1).expect("a label");
        assert_eq!(label.labeled_by, Labeler::Evidence);
        assert_eq!(label.class.as_ref().map(|c| c.as_str()), Some(C));
        assert_eq!(basis_kinds(label), [EvidenceKind::RetryOutcome]);
        let cited = &label.basis[0].records;
        assert!(
            cited.contains(&rid(1, 1).to_string()) && cited.contains(&job_id("j002").to_string())
        );
        assert!(label.validate().is_ok());
        assert_eq!(d.summary.by_kind["retry-outcome"], 1);
    }

    #[test]
    fn athena_retry_of_the_job_counts_and_a_retry_failing_for_the_same_cause_gives_a_label() {
        let mut records = Records::default();
        records
            .introspections
            .push(intro(1, 1, "j001", IntrospectionTrigger::Rejection, C));
        add_job(
            &mut records,
            "j001",
            "Add a field.",
            0,
            Some(OutcomeState::Rejected),
        );
        // `athena retry j001` records this request, and the retry was rejected for the same class.
        add_job(
            &mut records,
            "j002",
            &format!("try again on {}", job_id("j001")),
            5,
            Some(OutcomeState::Rejected),
        );
        records
            .introspections
            .push(intro(2, 9, "j002", IntrospectionTrigger::Rejection, C));
        let d = derive(&records, ts(100));
        let label = label_of(&d, 1, 1).expect("a label for the first case");
        assert_eq!(basis_kinds(label), [EvidenceKind::RetryOutcome]);
        assert!(label.basis[0].detail.contains("same cause"));
        assert!(label.basis[0].records.contains(&rid(9, 2).to_string()));
        // The retry's own introspection has a retry of nothing: no evidence for it.
        assert!(label_of(&d, 2, 9).is_none());
    }

    #[test]
    fn a_retry_that_failed_for_another_class_or_is_still_open_gives_no_label() {
        let mut records = Records::default();
        records
            .introspections
            .push(intro(1, 1, "j001", IntrospectionTrigger::Rejection, C));
        add_job(
            &mut records,
            "j001",
            "Add a field.",
            0,
            Some(OutcomeState::Rejected),
        );
        add_job(
            &mut records,
            "j002",
            &format!("try again on {}", job_id("j001")),
            5,
            Some(OutcomeState::Failed),
        );
        records
            .introspections
            .push(intro(2, 9, "j002", IntrospectionTrigger::Failure, D));
        add_job(
            &mut records,
            "j003",
            &format!("try again on {}", job_id("j001")),
            12,
            None,
        );
        let d = derive(&records, ts(100));
        assert!(label_of(&d, 1, 1).is_none());
        assert_eq!(d.summary.no_evidence, 2);
    }

    #[test]
    fn a_class_that_kept_recurring_or_stopped_after_a_prevention_landed_gives_labels() {
        let mut records = Records::default();
        // Class C: the case at minute 1, a prevention that landed at minute 21, then a recurrence at minute 40.
        records
            .introspections
            .push(intro(1, 1, "j001", IntrospectionTrigger::Intervention, C));
        records
            .introspections
            .push(intro(2, 40, "j009", IntrospectionTrigger::Intervention, C));
        // Class D: the case at minute 2 and no recurrence over the jobs that closed after the landing.
        records
            .introspections
            .push(intro(3, 2, "j002", IntrospectionTrigger::Intervention, D));
        add_job(
            &mut records,
            "jp01",
            "Apply the default.",
            19,
            Some(OutcomeState::Landed),
        );
        let mut for_d = proposal(D, "jp01");
        for_d.id = "d-other".into();
        for_d.record = rid(20, 78);
        records.proposals.push(proposal(C, "jp01"));
        records.proposals.push(for_d);
        for n in 0..STOPPED_MIN_JOBS as u32 {
            add_job(
                &mut records,
                &format!("k{n:03}"),
                "Other work.",
                30 + n,
                Some(OutcomeState::Landed),
            );
        }
        let d = derive(&records, ts(100));
        let recurring = label_of(&d, 1, 1).expect("C recurred after the prevention");
        assert_eq!(
            basis_kinds(recurring),
            [EvidenceKind::RecurrenceAfterPrevention]
        );
        assert!(recurring.basis[0].detail.contains("recurred in 1 later"));
        for cited in [
            rid(20, 77).to_string(),
            job_id("jp01").to_string(),
            rid(40, 2).to_string(),
        ] {
            assert!(recurring.basis[0].records.contains(&cited), "{cited}");
        }
        let stopped = label_of(&d, 3, 2).expect("D stopped after the prevention");
        assert_eq!(stopped.class.as_ref().map(|c| c.as_str()), Some(D));
        assert!(stopped.basis[0].detail.contains("did not recur"));
        // The recurrence itself came after the landing, so no prevention follows it: unlabeled.
        assert!(label_of(&d, 2, 40).is_none());
        assert_eq!(d.summary.by_kind["recurrence-after-prevention"], 2);
    }

    #[test]
    fn a_stopped_after_prevention_label_is_not_rewritten_when_more_jobs_close() {
        let mut records = Records::default();
        records
            .introspections
            .push(intro(3, 2, "j002", IntrospectionTrigger::Intervention, D));
        add_job(
            &mut records,
            "jp01",
            "Apply the default.",
            19,
            Some(OutcomeState::Landed),
        );
        records.proposals.push(proposal(D, "jp01"));
        for n in 0..STOPPED_MIN_JOBS as u32 {
            add_job(
                &mut records,
                &format!("k{n:03}"),
                "Other work.",
                30 + n,
                Some(OutcomeState::Landed),
            );
        }
        let first = derive(&records, ts(100));
        assert_eq!(first.labels.len(), 1);
        assert!(
            !first.labels[0].basis[0]
                .detail
                .contains(&format!("over {} jobs", STOPPED_MIN_JOBS))
        );
        records.class_labels.extend(first.labels);
        // More jobs close; the evidence about the class is otherwise unchanged.
        for n in 0..3u32 {
            add_job(
                &mut records,
                &format!("m{n:03}"),
                "More work.",
                50 + n,
                Some(OutcomeState::Landed),
            );
        }
        let second = derive(&records, ts(200));
        assert!(second.labels.is_empty(), "{:?}", second.labels);
        assert_eq!(second.summary.already_derived, 1);
    }

    #[test]
    fn too_few_jobs_after_a_landing_do_not_show_that_the_class_stopped() {
        let mut records = Records::default();
        records
            .introspections
            .push(intro(1, 1, "j001", IntrospectionTrigger::Intervention, C));
        add_job(
            &mut records,
            "jp01",
            "Apply the default.",
            19,
            Some(OutcomeState::Landed),
        );
        records.proposals.push(proposal(C, "jp01"));
        add_job(
            &mut records,
            "k001",
            "Other work.",
            30,
            Some(OutcomeState::Landed),
        );
        assert!(derive(&records, ts(100)).labels.is_empty());
    }

    #[test]
    fn critic_and_independent_verifier_findings_that_name_the_cause_give_a_label() {
        let mut records = Records::default();
        // A correction: the verifier's finding was not shown to the introspection, so it counts.
        records
            .introspections
            .push(intro(1, 1, "j001", IntrospectionTrigger::Correction, C));
        add_job(&mut records, "j001", "Add a field.", 0, None);
        records
            .jobs
            .get_mut(job_id("j001").as_str())
            .unwrap()
            .verification = Some(Verification {
            job: job_id("j001"),
            candidate_commit: "c".repeat(40),
            verified_fingerprint: "f".repeat(8),
            tool_config_from_base: vec![],
            checks: vec![],
            check_cleanup: vec![],
            verifier_invocation: None,
            criteria: vec![],
            findings: vec![finding(
                FindingSource::Verifier,
                Severity::Major,
                "The new configuration field has no reader module connected to it.",
            )],
            request_reading_mismatch: None,
            agent_verdict: Some(Verdict::Fail),
            final_verdict: Verdict::Fail,
            reasons: vec![],
            inputs: vec![],
            summary: None,
        });
        // A rejection: only the critic's finding counts.
        records
            .introspections
            .push(intro(2, 2, "j002", IntrospectionTrigger::Rejection, C));
        add_job(&mut records, "j002", "Add another field.", 0, None);
        records
            .jobs
            .get_mut(job_id("j002").as_str())
            .unwrap()
            .critiques
            .push(Critique {
                job: job_id("j002"),
                round: 1,
                at: ts(3),
                invocation_seq: Some(2),
                agent_verdict: Some(CritiqueVerdict::Revise),
                final_verdict: CritiqueVerdict::Revise,
                findings: vec![finding(
                    FindingSource::Critic,
                    Severity::Major,
                    "Nothing connects the configuration field to its reader module.",
                )],
                request_reading_mismatch: None,
                summary: None,
            });
        let d = derive(&records, ts(100));
        let verifier = label_of(&d, 1, 1).expect("verifier finding");
        assert_eq!(basis_kinds(verifier), [EvidenceKind::SameCauseFindings]);
        assert!(verifier.basis[0].detail.contains("verification"));
        assert!(
            verifier.basis[0]
                .records
                .contains(&job_id("j001").to_string())
        );
        let critic = label_of(&d, 2, 2).expect("critic finding");
        assert!(critic.basis[0].detail.contains("critique round 1"));
    }

    #[test]
    fn a_blocking_verifier_finding_is_not_evidence_for_the_rejection_it_was_shown_to() {
        let mut records = Records::default();
        records
            .introspections
            .push(intro(1, 1, "j001", IntrospectionTrigger::Rejection, C));
        add_job(&mut records, "j001", "Add a field.", 0, None);
        records
            .jobs
            .get_mut(job_id("j001").as_str())
            .unwrap()
            .verification = Some(Verification {
            job: job_id("j001"),
            candidate_commit: "c".repeat(40),
            verified_fingerprint: "f".repeat(8),
            tool_config_from_base: vec![],
            checks: vec![],
            check_cleanup: vec![],
            verifier_invocation: None,
            criteria: vec![],
            findings: vec![
                finding(
                    FindingSource::Verifier,
                    Severity::Blocker,
                    "Nothing connects the configuration field to its reader module.",
                ),
                // Not about the cause at all.
                finding(
                    FindingSource::Verifier,
                    Severity::Minor,
                    "A comment is misspelled.",
                ),
            ],
            request_reading_mismatch: None,
            agent_verdict: Some(Verdict::Fail),
            final_verdict: Verdict::Fail,
            reasons: vec![],
            inputs: vec![],
            summary: None,
        });
        assert!(derive(&records, ts(100)).labels.is_empty());
    }

    #[test]
    fn a_case_with_no_evidence_gets_no_label_and_is_counted_as_unlabeled() {
        let mut records = Records::default();
        records
            .introspections
            .push(intro(1, 1, "j001", IntrospectionTrigger::Intervention, C));
        // A failed introspection names no class and cannot be labeled at all.
        let mut failed = intro(2, 2, "j002", IntrospectionTrigger::Failure, C);
        failed.status = IntrospectionStatus::Failed;
        failed.failure_class = None;
        records.introspections.push(failed);
        let d = derive(&records, ts(100));
        assert!(d.labels.is_empty());
        assert_eq!(
            (d.summary.recorded, d.summary.no_evidence, d.summary.derived),
            (1, 1, 0)
        );
        // And the labeled-set code agrees: nothing counts as labeled.
        let inputs = crate::labeled::Inputs {
            introspections: records.introspections.clone(),
            labels: d.labels.clone(),
            classes: vec![],
        };
        assert_eq!(crate::labeled::build(&inputs).excluded.unreviewed, 2);
    }

    #[test]
    fn evidence_that_names_different_classes_leaves_the_case_unlabeled() {
        let vote = |class: &str, kind: EvidenceKind| Vote {
            class: RecordId::parse(class).unwrap(),
            basis: LabelBasis {
                kind,
                records: vec!["x".into()],
                detail: "d".into(),
            },
        };
        assert_eq!(resolve(vec![]), Decision::NoEvidence);
        assert_eq!(
            resolve(vec![
                vote(C, EvidenceKind::RetryOutcome),
                vote(D, EvidenceKind::SameCauseFindings)
            ]),
            Decision::Conflict
        );
        let Decision::Label { class, basis } = resolve(vec![
            vote(C, EvidenceKind::SameCauseFindings),
            vote(C, EvidenceKind::RetryOutcome),
        ]) else {
            panic!("agreeing votes label");
        };
        assert_eq!(class.as_str(), C);
        // Every kind is cited, in a fixed order.
        assert_eq!(
            basis.iter().map(|b| b.kind).collect::<Vec<_>>(),
            [EvidenceKind::RetryOutcome, EvidenceKind::SameCauseFindings]
        );
    }

    #[test]
    fn several_kinds_that_agree_are_all_cited_in_one_label() {
        let mut records = Records::default();
        let case = intro(1, 1, "j001", IntrospectionTrigger::Correction, C);
        let proposed = case.proposed_request.clone().unwrap();
        records.introspections.push(case);
        add_job(&mut records, "j001", "Add a field.", 0, None);
        add_job(
            &mut records,
            "j002",
            &proposed,
            5,
            Some(OutcomeState::Landed),
        );
        records
            .jobs
            .get_mut(job_id("j001").as_str())
            .unwrap()
            .critiques
            .push(Critique {
                job: job_id("j001"),
                round: 1,
                at: ts(2),
                invocation_seq: None,
                agent_verdict: None,
                final_verdict: CritiqueVerdict::Revise,
                findings: vec![finding(
                    FindingSource::Critic,
                    Severity::Minor,
                    "Nothing connects the configuration field to its reader module.",
                )],
                request_reading_mismatch: None,
                summary: None,
            });
        let d = derive(&records, ts(100));
        let [label] = d.labels.as_slice() else {
            panic!("one label");
        };
        assert_eq!(
            basis_kinds(label),
            [EvidenceKind::RetryOutcome, EvidenceKind::SameCauseFindings]
        );
        assert_eq!(d.summary.derived, 1);
        assert_eq!(d.summary.by_kind.values().sum::<usize>(), 2);
    }

    #[test]
    fn a_person_s_label_is_skipped_and_a_rerun_writes_nothing_and_edits_nothing() {
        let mut records = Records::default();
        for n in 1..=3 {
            let case = intro(n, n, &format!("j00{n}"), IntrospectionTrigger::Rejection, C);
            let proposed = case.proposed_request.clone().unwrap();
            records.introspections.push(case);
            add_job(
                &mut records,
                &format!("j00{n}"),
                "Add a field.",
                0,
                Some(OutcomeState::Rejected),
            );
            add_job(
                &mut records,
                &format!("r00{n}"),
                &proposed,
                10 + n,
                Some(OutcomeState::Landed),
            );
        }
        // A person labeled case 2 (class D) before any derivation.
        let person = ClassLabel {
            id: rid(50, 900),
            at: ts(50),
            introspection: rid(2, 2),
            class: Some(RecordId::parse(D).unwrap()),
            labeled_by: Labeler::Person,
            is_repeat: None,
            note: None,
            basis: vec![],
        };
        records.class_labels.push(person.clone());
        let before = records.class_labels.clone();
        let first = derive(&records, ts(100));
        assert_eq!(first.labels.len(), 2);
        assert!(label_of(&first, 2, 2).is_none());
        let s = &first.summary;
        assert_eq!((s.recorded, s.reviewed, s.derived), (3, 1, 2));
        assert_eq!(
            s.recorded,
            s.reviewed + s.already_derived + s.derived + s.no_evidence + s.conflicting
        );
        // The derivation did not touch the records it read.
        assert_eq!(records.class_labels, before);
        // Once appended, a rerun finds them already derived and writes nothing.
        records.class_labels.extend(first.labels.clone());
        let second = derive(&records, ts(200));
        assert!(second.labels.is_empty());
        assert_eq!(
            (
                second.summary.derived,
                second.summary.already_derived,
                second.summary.reviewed
            ),
            (0, 2, 1)
        );
        // The person's label still counts for case 2.
        let counting = newest_labels(&records.class_labels);
        assert_eq!(counting[&rid(2, 2).to_string()], &person);
        // Ids are unique across the labels.
        let ids: BTreeSet<_> = records.class_labels.iter().map(|l| &l.id).collect();
        assert_eq!(ids.len(), records.class_labels.len());
    }

    #[test]
    fn no_records_give_no_labels() {
        let d = derive(&Records::default(), ts(1));
        assert!(d.labels.is_empty());
        assert_eq!(d.summary, Summary::default());
    }
}
