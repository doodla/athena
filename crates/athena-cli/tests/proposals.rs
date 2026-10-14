//! Defaults proposals (`docs/design/defaults-loop.md` sections 8 and 9, build order step 3), on fixture
//! records in temporary repositories: `athena propose` writes a pending proposal when one class repeats across
//! enough distinct jobs; `athena status` lists open proposals and omits rejected and accepted ones; accepting
//! one submits an ordinary job request that lists `defaults.toml` as a deliverable and edits nothing; rejecting
//! one is recorded and suppresses the class. No model is called.

mod common;

use athena_core::proposals::{self, REPEAT_THRESHOLD};
use athena_core::store::{JobState, Records};
use athena_core::{Athena, Quiet};
use athena_data::records::{
    AnyRecord, ClassStatus, FailureClass, IntrospectionRecord, IntrospectionStatus,
    IntrospectionTrigger, PreventionKind, ProposalStatus,
};
use athena_data::{JobId, RecordId};
use common::*;
use std::process::Command;

const CLASS: &str = "20261001-000000-c001";

fn at(text: &str) -> jiff::Timestamp {
    text.parse().unwrap()
}

fn rid(day: u32, n: u32) -> RecordId {
    RecordId::from_parts(at(&format!("2026-10-{day:02}T00:00:00Z")), n)
}

fn class_record() -> AnyRecord {
    AnyRecord::FailureClass(FailureClass {
        id: RecordId::parse(CLASS).unwrap(),
        at: at("2026-10-01T00:00:00Z"),
        name: "Field with no use".into(),
        description: "A new field has no reader.".into(),
        status: ClassStatus::Proposed,
        proposed_by: rid(1, 1),
    })
}

/// A recorded introspection of the class, for the job numbered `job`, written by a correction on `day`.
fn case_record(day: u32, job: u32) -> AnyRecord {
    AnyRecord::Introspection(Box::new(IntrospectionRecord {
        id: rid(day, 1),
        at: at(&format!("2026-10-{day:02}T00:00:00Z")),
        trigger: IntrospectionTrigger::Correction,
        job: Some(JobId::parse(&format!("202610{day:02}-000000-j{job:03}")).unwrap()),
        intervention: Some(rid(day, 501)),
        class_of_work: "code".into(),
        category: "corrected-output".into(),
        what_happened: Some("Added a field with no reader.".into()),
        failure_class: Some(RecordId::parse(CLASS).unwrap()),
        new_class: false,
        earlier_cases: vec![],
        decisions_matched: vec![],
        cause: Some("Nothing ties a field to its first use.".into()),
        prevention_kind: Some(PreventionKind::Default),
        prevention: Some("Add the field with its reader.".into()),
        proposed_request: Some("Add the reader.".into()),
        evidence: None,
        status: IntrospectionStatus::Recorded,
        failure: None,
        flags: vec![],
        dropped_ids: vec![],
        from_jev: false,
        jev: None,
    }))
}

/// Commits the class and `cases` (days of the month, one job each) as the repository's records.
fn commit_cases(fx: &Fixture, days: &[u32]) {
    let mut records = vec![class_record()];
    records.extend(days.iter().map(|d| case_record(*d, *d)));
    for record in &records {
        write(fx.root(), &[(&record.canonical_path(), &record.to_json())]);
    }
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run(["commit", "-q", "-m", "Add fixture introspections"])
        .unwrap();
}

fn athena(fx: &Fixture) -> Athena {
    fx.athena(Scenario::default().script())
}

fn open_ids(athena: &Athena) -> Vec<String> {
    athena
        .status()
        .unwrap()
        .proposals
        .iter()
        .map(|p| p.id.clone())
        .collect()
}

#[test]
fn status_lists_open_proposals_with_their_sources_and_omits_decided_ones() {
    let fx = Fixture::new();
    commit_cases(&fx, &[2, 3, 4]);
    let athena = athena(&fx);
    let generated = athena.propose(&Quiet).unwrap();
    assert_eq!(generated.proposals.len(), 1);

    let view = athena.status().unwrap();
    let [p] = view.proposals.as_slice() else {
        panic!("one open proposal")
    };
    assert_eq!(
        (p.id.as_str(), p.kind.as_str()),
        ("field-with-no-use", "add")
    );
    assert_eq!((p.version_from, p.version_to), (1, 2));
    assert_eq!(p.sources.len(), REPEAT_THRESHOLD);
    assert!(p.sources[0].contains("introspection 20261002-000000-"));
    assert!(p.sources[0].contains("(job 20261002-000000-j002)"));

    let text = athena_cli::display::status(&view);
    for needle in [
        "field-with-no-use",
        "[add]",
        "version 1 -> 2",
        "Nothing ties a field to its first use.",
        "source: introspection 20261003-000000-",
        "athena accept-proposal field-with-no-use",
    ] {
        assert!(text.contains(needle), "{needle:?} missing from:\n{text}");
    }

    // `athena status --json` prints exactly this view.
    let json = serde_json::to_value(&view).unwrap();
    let shown = &json["proposals"][0];
    assert_eq!(shown["id"], "field-with-no-use");
    assert_eq!(shown["kind"], "add");
    assert_eq!(shown["version_from"], 1);
    assert_eq!(shown["version_to"], 2);
    assert!(
        shown["statement"]
            .as_str()
            .unwrap()
            .contains("Field with no use")
    );
    assert_eq!(shown["sources"].as_array().unwrap().len(), 3);

    // A rejected proposal is not listed.
    athena
        .reject_proposal("field-with-no-use", "Too broad.")
        .unwrap();
    assert!(open_ids(&athena).is_empty());
    let text = athena_cli::display::status(&athena.status().unwrap());
    assert!(!text.contains("proposals"), "{text}");
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    assert_eq!(
        records.proposals.len(),
        2,
        "a status change is a new record"
    );
    assert!(matches!(
        proposals::current(&records.proposals)[0].status,
        ProposalStatus::Rejected { ref reason } if reason == "Too broad."
    ));
}

#[test]
fn a_rejected_class_is_proposed_again_only_after_enough_newer_jobs() {
    let fx = Fixture::new();
    commit_cases(&fx, &[2, 3, 4]);
    let athena = athena(&fx);
    athena.propose(&Quiet).unwrap();
    athena
        .reject_proposal("field-with-no-use", "Not now.")
        .unwrap();
    assert!(athena.propose(&Quiet).unwrap().proposals.is_empty());
    // Cases dated after the rejection (it is dated now, so use a far-future month).
    let mut later = Vec::new();
    for (n, job) in [(1u32, 21u32), (2, 22), (3, 23)] {
        let id = RecordId::from_parts(at("2099-01-01T00:00:00Z"), n);
        let AnyRecord::Introspection(mut record) = case_record(5, job) else {
            unreachable!()
        };
        record.id = id;
        record.at = at("2099-01-01T00:00:00Z");
        record.job = Some(JobId::parse(&format!("20990101-000000-j{job:03}")).unwrap());
        later.push(AnyRecord::Introspection(record));
    }
    for record in later.iter().take(2) {
        write(fx.root(), &[(&record.canonical_path(), &record.to_json())]);
    }
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run(["commit", "-q", "-m", "Two newer cases"])
        .unwrap();
    assert!(athena.propose(&Quiet).unwrap().proposals.is_empty());
    let last = &later[2];
    write(fx.root(), &[(&last.canonical_path(), &last.to_json())]);
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run(["commit", "-q", "-m", "A third newer case"])
        .unwrap();
    assert_eq!(athena.propose(&Quiet).unwrap().proposals.len(), 1);
    assert_eq!(open_ids(&athena), ["field-with-no-use"]);
}

#[test]
fn accepting_submits_a_job_that_lists_defaults_toml_and_edits_nothing() {
    let fx = Fixture::new();
    commit_cases(&fx, &[2, 3, 4]);
    let athena = athena(&fx);
    athena.propose(&Quiet).unwrap();
    let defaults_before = fx.file("defaults.toml").unwrap();

    let reading = athena.accept_proposal("field-with-no-use", &Quiet).unwrap();
    // Not edited: the file is as it was, in the working tree and at HEAD.
    assert_eq!(fx.file("defaults.toml").unwrap(), defaults_before);
    let at_head = fx
        .git
        .show_file(&fx.head(), "defaults.toml")
        .unwrap()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&at_head), defaults_before);

    let records = Records::load(&fx.git, &fx.head()).unwrap();
    let job = &records.jobs[reading.job.as_str()];
    assert_eq!(job.state(), JobState::Proposed, "read, not run");
    let request = &job.request.as_ref().unwrap().text;
    assert!(request.starts_with("apply defaults proposal field-with-no-use"));
    assert!(request.contains("Deliverable: defaults.toml (modify)"));
    let current = proposals::current(&records.proposals);
    assert_eq!(
        current[0].status,
        ProposalStatus::Accepted {
            job: reading.job.clone()
        }
    );
    assert!(open_ids(&athena).is_empty());
    // The proposal can only be accepted once.
    assert!(athena.accept_proposal("field-with-no-use", &Quiet).is_err());
}

#[test]
fn checking_a_candidate_applies_the_proposal_check_to_defaults_at_head() {
    let fx = Fixture::new();
    commit_cases(&fx, &[2, 3, 4]);
    let athena = athena(&fx);
    let p = athena.propose(&Quiet).unwrap().proposals.remove(0);
    let base = fx.file("defaults.toml").unwrap();
    let good = proposals::build_candidate(&base, &p).unwrap();
    assert!(athena.check_proposal(&p.id, &good).unwrap().is_empty());
    let bad = good.replace(&p.id, "another-id");
    assert!(!athena.check_proposal(&p.id, &bad).unwrap().is_empty());
}

#[test]
fn the_proposal_commands_exist() {
    for command in [
        "propose",
        "accept-proposal",
        "reject-proposal",
        "check-proposal",
    ] {
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
