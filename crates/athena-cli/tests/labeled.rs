//! The labeled set and Jev scoring (`docs/design/defaults-loop.md` sections 10.2 to 10.4, build order step 6),
//! on fixture records in a temporary repository with the fake Jev: `athena score-jev` reads the records, gives
//! a score and verdict when enough reviewed cases exist, says "too few labeled cases" and gives no verdict
//! when not, counts what it leaves out, and writes nothing.

mod common;

use athena_core::introspect::{ClassRow, JEV_CLASS_KEY, JEV_NEW_OPTION, JEV_REPEAT_KEY, ask_jev};
use athena_core::jev::{Answer, Evaluation, FakeJev, JevError, Usage};
use athena_core::labeled::MIN_TEST_CASES;
use athena_data::records::{
    AnyRecord, ClassLabel, ClassStatus, FailureClass, IntrospectionJev, IntrospectionRecord,
    IntrospectionStatus, IntrospectionTrigger, Labeler, PreventionKind,
};
use athena_data::{JobId, RecordId};
use common::*;
use std::collections::BTreeMap;
use std::process::Command;

const CLASS: &str = "20261001-000000-c001";

fn at(i: u32) -> jiff::Timestamp {
    format!("2026-10-01T{:02}:{:02}:00Z", i / 60, i % 60)
        .parse()
        .unwrap()
}

fn rid(i: u32, n: u32) -> RecordId {
    RecordId::from_parts(at(i), n)
}

fn registry() -> Vec<ClassRow> {
    vec![ClassRow {
        id: CLASS.into(),
        name: "Field with no use".into(),
        description: "A new field has no reader.".into(),
    }]
}

fn class_record() -> AnyRecord {
    AnyRecord::FailureClass(FailureClass {
        id: RecordId::parse(CLASS).unwrap(),
        at: at(0),
        name: "Field with no use".into(),
        description: "A new field has no reader.".into(),
        status: ClassStatus::Proposed,
        proposed_by: rid(0, 1),
    })
}

/// What the fake Jev answers: `says` with probability `p`, the rest on the other option.
fn jev_says(says: &str, p: f64) -> IntrospectionJev {
    let other = if says == CLASS { JEV_NEW_OPTION } else { CLASS };
    let probabilities = BTreeMap::from([(says.to_string(), p), (other.to_string(), 1.0 - p)]);
    let evaluation = Evaluation {
        answers: BTreeMap::from([
            (
                JEV_CLASS_KEY.to_string(),
                Answer::Choice {
                    choice: says.to_string(),
                    probabilities,
                    confidence: 0.9,
                },
            ),
            (
                JEV_REPEAT_KEY.to_string(),
                Answer::Boolean { probability: 0.3 },
            ),
        ]),
        usage: Usage {
            input_tokens: 10,
            output_tokens: 2,
        },
        cost_usd: None,
    };
    ask_jev(&FakeJev::new().then(Ok(evaluation)), "state", &registry())
}

fn introspection(i: u32, jev: Option<IntrospectionJev>) -> AnyRecord {
    AnyRecord::Introspection(Box::new(IntrospectionRecord {
        id: rid(i, 1),
        at: at(i),
        trigger: IntrospectionTrigger::Correction,
        job: Some(JobId::parse(&format!("20261001-000000-j{i:03}")).unwrap()),
        intervention: Some(rid(i, 501)),
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
        planning_stopped: false,
        from_jev: false,
        jev,
    }))
}

fn label(i: u32, class: Option<&str>, by: Labeler) -> AnyRecord {
    AnyRecord::ClassLabel(ClassLabel {
        id: rid(i, 900),
        at: at(i),
        introspection: rid(i, 1),
        class: class.map(|c| RecordId::parse(c).unwrap()),
        labeled_by: by,
        is_repeat: None,
        note: None,
    })
}

/// Commits `records` as the repository's records.
fn commit(fx: &Fixture, records: &[AnyRecord]) {
    for record in records {
        write(fx.root(), &[(&record.canonical_path(), &record.to_json())]);
    }
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run(["commit", "-q", "-m", "Add fixture records"])
        .unwrap();
}

/// A reviewed case Jev answers with `says`.
fn reviewed(i: u32, class: Option<&str>, says: &str, by: Labeler) -> [AnyRecord; 2] {
    [
        introspection(i, Some(jev_says(says, 0.9))),
        label(i, class, by),
    ]
}

fn score_jev(fx: &Fixture, extra: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_athena"))
        .current_dir(fx.root())
        .env("ATHENA_STATE_DIR", fx.state.path())
        .env_remove("ATHENA_NESTED")
        .env_remove("ATHENA_JEV_SHADOW")
        .arg("score-jev")
        .args(extra)
        .output()
        .unwrap()
}

fn json(out: &std::process::Output) -> serde_json::Value {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

#[test]
fn the_help_lists_the_command() {
    let out = Command::new(env!("CARGO_BIN_EXE_athena"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("score-jev"));
}

#[test]
fn an_empty_store_reports_too_few_labeled_cases_and_exits_cleanly() {
    let fx = Fixture::new();
    let out = score_jev(&fx, &["--json"]);
    let report = json(&out);
    assert!(
        report["insufficient"]
            .as_str()
            .unwrap()
            .contains("too few labeled cases")
    );
    assert!(report["score"].is_null() && report["verdict"].is_null());
    assert_eq!(report["excluded"]["introspections"], 0);
    let text = score_jev(&fx, &[]);
    assert!(text.status.success());
    let text = String::from_utf8_lossy(&text.stdout);
    assert!(text.contains("too few labeled cases"), "{text}");
    assert!(!text.contains("Promotion"), "{text}");
}

#[test]
fn reviewed_cases_with_fake_jev_answers_give_a_score_a_verdict_and_change_nothing() {
    let fx = Fixture::new();
    let mut records = vec![class_record()];
    // Twenty calibration cases and then twelve test cases: nine of the class that Jev gets right, three
    // `none` cases that it also gets right (so the false-match rate is 0 of 3).
    for i in 1..=20 {
        records.extend(reviewed(i, Some(CLASS), CLASS, Labeler::Person));
    }
    for i in 101..110 {
        records.extend(reviewed(i, Some(CLASS), CLASS, Labeler::Person));
    }
    for i in 110..113 {
        records.extend(reviewed(i, None, JEV_NEW_OPTION, Labeler::StrongerModel));
    }
    // Left out: unreviewed, no Jev answer, and a failed Jev call.
    records.push(introspection(200, Some(jev_says(CLASS, 0.9))));
    records.push(introspection(201, None));
    records.push(label(201, Some(CLASS), Labeler::Person));
    let failed = ask_jev(
        &FakeJev::new().then(Err(JevError::Transport("down".into()))),
        "state",
        &registry(),
    );
    records.push(introspection(202, Some(failed)));
    records.push(label(202, Some(CLASS), Labeler::Person));
    commit(&fx, &records);
    let head = fx.head();

    let report = json(&score_jev(
        &fx,
        &["--json", "--cut", "2026-10-01T01:40:00Z"],
    ));
    let e = &report["excluded"];
    assert_eq!(
        (
            e["introspections"].as_u64(),
            e["labeled"].as_u64(),
            e["unreviewed"].as_u64(),
            e["no_jev_answer"].as_u64(),
            e["jev_error"].as_u64(),
        ),
        (Some(35), Some(32), Some(1), Some(1), Some(1))
    );
    assert_eq!(
        (report["calibration"].as_u64(), report["test"].as_u64()),
        (Some(20), Some(12))
    );
    assert!(report["test"].as_u64().unwrap() >= MIN_TEST_CASES as u64);
    assert!(report["insufficient"].is_null());
    let score = &report["score"];
    assert_eq!(score["n"], 12);
    assert_eq!(score["none_labels"], 3);
    assert_eq!(score["false_match_rate"], 0.0);
    assert!(score["skill"].as_f64().unwrap() > 0.0);
    assert_eq!(score["per_record"].as_array().unwrap().len(), 12);
    let items = report["verdict"]["items"].as_array().unwrap();
    assert_eq!(items.len(), 7);
    let status = |n: usize| items[n - 1]["status"].as_str().unwrap().to_string();
    assert_eq!(status(2), "met");
    assert_eq!(status(3), "met");
    assert_eq!(status(4), "not-evaluated");
    // The labeler is kept on each row.
    let by: Vec<&str> = report["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["labeled_by"].as_str().unwrap())
        .collect();
    assert!(by.contains(&"person") && by.contains(&"stronger-model"));

    let text = String::from_utf8_lossy(&score_jev(&fx, &["--cut", "2026-10-01T01:40:00Z"]).stdout)
        .into_owned();
    for needle in [
        "n = 12",
        "Promotion (section 10.4)",
        "Per record",
        "skill over it",
    ] {
        assert!(text.contains(needle), "{needle:?} not in {text}");
    }
    assert!(
        text.contains("95% interval") || text.contains("no bound was computed"),
        "{text}"
    );

    // Read-only: no commit, no changed or new file.
    assert_eq!(fx.head(), head);
    assert_eq!(fx.git.run(["status", "--porcelain"]).unwrap().trim(), "");
}

#[test]
fn a_few_reviewed_cases_give_no_score_and_no_verdict() {
    let fx = Fixture::new();
    let mut records = vec![class_record()];
    for i in 1..=5 {
        records.extend(reviewed(i, Some(CLASS), CLASS, Labeler::Person));
    }
    commit(&fx, &records);
    let report = json(&score_jev(&fx, &["--json"]));
    assert!(
        report["insufficient"]
            .as_str()
            .unwrap()
            .contains("too few labeled cases")
    );
    assert!(report["score"].is_null() && report["verdict"].is_null());
    assert_eq!(report["excluded"]["labeled"], 5);
}

#[test]
fn a_bad_cut_is_refused() {
    let fx = Fixture::new();
    let out = score_jev(&fx, &["--cut", "yesterday"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("RFC 3339"));
}
