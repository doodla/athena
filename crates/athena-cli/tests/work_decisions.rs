//! Deciding a steward proposal by its id (`athena approve-work`, `athena reject-work`), on a seeded work item
//! in a temporary repository: approving queues an ordinary job whose request copies the work item's content
//! and the content of what it cites, because readers cannot see `records/`; rejecting records the reason; a
//! second decision, an unknown id and an empty reason are refused. No model call is made beyond the scripted
//! reading.

mod common;

use athena_core::Quiet;
use athena_core::store::{JobState, Records};
use athena_data::records::{
    AnyRecord, Reference, Request, WorkDecisionState, WorkItem, WorkItemKind,
};
use athena_data::roles::Ladder;
use athena_data::{JobId, RecordId};
use common::*;
use std::process::Command;

const ITEM: &str = "20261015-003903-khz6";
const CITED_JOB: &str = "20261010-153012-k7q2";

fn at() -> jiff::Timestamp {
    "2026-10-15T00:39:03Z".parse().unwrap()
}

fn commit(fx: &Fixture, records: &[AnyRecord], message: &str) {
    for record in records {
        write(fx.root(), &[(&record.canonical_path(), &record.to_json())]);
    }
    fx.git.run(["add", "-A"]).unwrap();
    fx.git.run(["commit", "-q", "-m", message]).unwrap();
}

fn work_item(id: &str, kind: WorkItemKind) -> AnyRecord {
    AnyRecord::WorkItem(Box::new(WorkItem {
        id: RecordId::parse(id).unwrap(),
        at: at(),
        role: "steward".into(),
        kind,
        key: format!("proposal:{id}"),
        title: "Make readers see the cited records".into(),
        rung: Ladder::DesignDecisions,
        source: Reference::path("src/lib.rs"),
        evidence: vec![
            Reference::job(&JobId::parse(CITED_JOB).unwrap()),
            Reference::path("records/jobs"),
        ],
    }))
}

/// A repository with a proposal-kind work item, the job it cites, and a second item that is a blind spot.
fn seeded() -> Fixture {
    let fx = Fixture::new();
    let cited = AnyRecord::Request(Request {
        job: JobId::parse(CITED_JOB).unwrap(),
        text: "Add the frobnicate option to the exporter.".into(),
        received_at: at(),
        channel: "cli".into(),
    });
    commit(
        &fx,
        &[
            work_item(ITEM, WorkItemKind::Proposal),
            work_item("20261015-003904-aaaa", WorkItemKind::BlindSpot),
            cited,
        ],
        "Seed a steward proposal",
    );
    fx
}

#[test]
fn status_lists_the_open_proposal_with_its_id_and_nothing_else() {
    let fx = seeded();
    let athena = fx.athena(Scenario::default().script());
    let view = athena.status().unwrap();
    let [w] = view.work_proposals.as_slice() else {
        panic!("one open proposal, got {:?}", view.work_proposals)
    };
    assert_eq!(w.id, ITEM);
    assert_eq!(w.rung, "design-decisions");
    let text = athena_cli::display::status(&view);
    for needle in [
        ITEM,
        "Make readers see the cited records",
        "athena approve-work",
        "athena reject-work",
    ] {
        assert!(text.contains(needle), "{needle:?} missing from:\n{text}");
    }
    assert!(!text.contains("20261015-003904-aaaa"), "{text}");
}

#[test]
fn approving_queues_a_job_whose_request_carries_the_cited_content() {
    let fx = seeded();
    let athena = fx.athena(Scenario::default().script());
    let reading = athena.approve_work(ITEM, &Quiet).unwrap();

    let records = Records::load(&fx.git, &fx.head()).unwrap();
    let job = &records.jobs[reading.job.as_str()];
    assert_eq!(job.state(), JobState::Proposed, "read, not run");
    let request = &job.request.as_ref().unwrap().text;
    // The item's own content.
    assert!(request.starts_with("Make readers see the cited records"));
    assert!(request.contains("design-decisions"));
    assert!(request.contains(ITEM));
    // The source file, inlined.
    assert!(request.contains("pub fn one() -> u32"), "{request}");
    // The cited job's request, inlined.
    assert!(
        request.contains("Add the frobnicate option to the exporter."),
        "{request}"
    );
    // A cited directory is listed, not left as a bare path.
    assert!(request.contains("A directory of"), "{request}");
    assert!(!request.contains("records/work-items/"), "{request}");

    // The decision is an appended record that links the item to the job.
    let [decision] = records.work_decisions.as_slice() else {
        panic!("one decision")
    };
    assert_eq!(decision.work_item.as_str(), ITEM);
    assert_eq!(
        decision.decision,
        WorkDecisionState::Approved {
            job: reading.job.clone()
        }
    );
    assert_eq!(records.work_items.len(), 2, "work items are not edited");
    assert!(athena.status().unwrap().work_proposals.is_empty());

    // A second decision is refused, either way.
    let again = athena.approve_work(ITEM, &Quiet).unwrap_err();
    assert!(again.to_string().contains("already approved"), "{again}");
    assert!(athena.reject_work(ITEM, "Changed my mind.").is_err());
}

#[test]
fn rejecting_records_the_reason_and_closes_the_proposal() {
    let fx = seeded();
    let athena = fx.athena(Scenario::default().script());
    let decision = athena.reject_work(ITEM, "Too broad.").unwrap();
    assert_eq!(
        decision.decision,
        WorkDecisionState::Rejected {
            reason: "Too broad.".into()
        }
    );
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    assert_eq!(records.work_decisions, vec![decision]);
    assert!(records.invalid.is_empty());
    assert!(athena.status().unwrap().work_proposals.is_empty());
    assert!(athena.approve_work(ITEM, &Quiet).is_err());
    assert!(athena.reject_work(ITEM, "Again.").is_err());
}

#[test]
fn an_unknown_id_a_blind_spot_and_an_empty_reason_are_refused() {
    let fx = seeded();
    let athena = fx.athena(Scenario::default().script());
    let head = fx.head();
    assert!(athena.approve_work("20261015-009999-zzzz", &Quiet).is_err());
    assert!(athena.reject_work("20261015-009999-zzzz", "No.").is_err());
    assert!(athena.reject_work("not an id", "No.").is_err());
    assert!(athena.approve_work("20261015-003904-aaaa", &Quiet).is_err());
    assert!(athena.reject_work(ITEM, "  ").is_err());
    assert_eq!(fx.head(), head, "a refusal commits nothing");
}

#[test]
fn the_work_commands_exist() {
    for command in ["approve-work", "reject-work"] {
        let out = Command::new(env!("CARGO_BIN_EXE_athena"))
            .env_remove("ATHENA_NESTED")
            .args([command, "--help"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{command}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
