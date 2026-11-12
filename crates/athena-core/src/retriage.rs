//! Re-triage of the proposals queued before Olympus (requirement OLY-A-09).
//!
//! Each open work-item proposal queued before 2026-10-17 is kept if the steward still raises its finding key,
//! and otherwise closed with a rejection that gives the reason. Queued job requests are not work items and are
//! not covered here. The function is pure: it returns the decisions to append and writes nothing.

use crate::store::Records;
use crate::work_decisions::{open_proposals, rejection};
use athena_data::RecordId;
use athena_data::records::WorkDecision;
use jiff::Timestamp;
use std::collections::HashSet;

/// The start of the Olympus day. A proposal queued earlier is re-triaged.
pub const OLYMPUS_START: &str = "2026-10-17T00:00:00Z";

/// The reason recorded when a pre-Olympus proposal is closed.
pub const CLOSED_REASON: &str = "Re-triaged against Olympus: the steward no longer raises this finding, so it no longer matters.";

/// The rejections that close every open proposal queued before Olympus whose key is not in `still_raised`.
/// A proposal that is kept, queued on or after Olympus, or already decided gets no decision, so running it
/// again on the records with these decisions added returns nothing. `next_id` supplies a fresh record id for
/// each decision.
pub fn retriage(
    records: &Records,
    still_raised: &HashSet<String>,
    mut next_id: impl FnMut() -> RecordId,
    now: Timestamp,
) -> Vec<WorkDecision> {
    let cutoff: Timestamp = OLYMPUS_START.parse().expect("the Olympus start is a time");
    open_proposals(records)
        .into_iter()
        .filter(|w| w.at < cutoff && !still_raised.contains(&w.key))
        .filter_map(|w| rejection(w, CLOSED_REASON, next_id(), now).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::records::{Reference, WorkDecisionState, WorkItem, WorkItemKind};
    use athena_data::roles::Ladder;

    fn proposal(id: &str, at: &str, key: &str) -> WorkItem {
        WorkItem {
            id: RecordId::parse(id).unwrap(),
            at: at.parse().unwrap(),
            role: "steward".into(),
            kind: WorkItemKind::Proposal,
            key: key.into(),
            title: "Change the thing".into(),
            rung: Ladder::DesignDecisions,
            source: Reference::path("records/jobs"),
            evidence: vec![],
            brief: Some(athena_data::records::OwnerBrief {
                question: "Q?".into(),
                if_yes: "Yes.".into(),
                if_no: "No.".into(),
                cost: "Little.".into(),
                recommendation: "Yes.".into(),
            }),
        }
    }

    fn ids() -> impl FnMut() -> RecordId {
        let mut n = 0;
        move || {
            n += 1;
            RecordId::parse(&format!("20261101-00000{n}-dec{n}")).unwrap()
        }
    }

    fn now() -> Timestamp {
        "2026-11-01T00:00:00Z".parse().unwrap()
    }

    fn records() -> Records {
        let mut records = Records::default();
        records.work_items.extend([
            proposal("20261010-000001-aaaa", "2026-10-10T00:00:00Z", "kept"),
            proposal("20261011-000001-bbbb", "2026-10-11T00:00:00Z", "gone"),
            proposal("20261020-000001-cccc", "2026-10-20T00:00:00Z", "gone"),
        ]);
        records
    }

    #[test]
    fn retriage_closes_unraised_pre_olympus_proposals_and_keeps_the_rest() {
        let mut records = records();
        let raised: HashSet<String> = ["kept".to_string()].into();
        let decisions = retriage(&records, &raised, ids(), now());
        assert_eq!(decisions.len(), 1, "only the unraised pre-Olympus one");
        assert_eq!(decisions[0].work_item.to_string(), "20261011-000001-bbbb");
        match &decisions[0].decision {
            WorkDecisionState::Rejected { reason } => assert!(!reason.trim().is_empty()),
            other => panic!("expected a rejection, got {other:?}"),
        }
        records.work_decisions.extend(decisions);
        let open: Vec<String> = open_proposals(&records)
            .iter()
            .map(|w| w.id.to_string())
            .collect();
        assert_eq!(open, ["20261010-000001-aaaa", "20261020-000001-cccc"]);
    }

    #[test]
    fn retriage_is_idempotent_and_an_empty_queue_closes_nothing() {
        let mut records = records();
        let raised = HashSet::new();
        let first = retriage(&records, &raised, ids(), now());
        assert_eq!(first.len(), 2, "both pre-Olympus proposals are closed");
        records.work_decisions.extend(first);
        let again = retriage(&records, &raised, ids(), now());
        assert!(again.is_empty(), "decided items are not decided again");

        let empty = Records::default();
        let none = retriage(&empty, &raised, ids(), now());
        assert!(none.is_empty(), "an empty queue closes nothing");
    }
}
