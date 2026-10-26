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
    IntrospectionStatus, IntrospectionTrigger, Labeler, Outcome, OutcomeState, PreventionKind,
    Request, Totals,
};
use athena_data::{JobId, Quantity, RecordId, Unit};
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
        basis: vec![],
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

/// A rejected case whose proposed request is `Add the reader {i}.`.
fn rejected_case(i: u32, jev: Option<IntrospectionJev>) -> AnyRecord {
    let AnyRecord::Introspection(mut record) = introspection(i, jev) else {
        unreachable!("introspection() builds an introspection");
    };
    record.proposed_request = Some(format!("Add the reader {i}."));
    AnyRecord::Introspection(record)
}

/// The retry of case `i`: a job whose request is the proposed request, closed as landed.
fn landed_retry(i: u32) -> [AnyRecord; 2] {
    let job = JobId::parse(&format!("20261001-000000-r{i:03}")).unwrap();
    let zero = || Quantity::reported(0.0, Unit::Usd, "sum");
    [
        AnyRecord::Request(Request {
            job: job.clone(),
            text: format!("Add the reader {i}."),
            received_at: at(i),
            channel: "cli".into(),
        }),
        AnyRecord::Outcome(Box::new(Outcome {
            job,
            state: OutcomeState::Landed,
            reason: "landed".into(),
            at: at(i),
            totals: Some(Totals {
                cost_all: zero(),
                cost_execution: zero(),
                wall_time_execution: None,
                tokens: BTreeMap::new(),
                invocations: 0,
            }),
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        })),
    ]
}

fn athena(fx: &Fixture, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_athena"))
        .current_dir(fx.root())
        .env("ATHENA_STATE_DIR", fx.state.path())
        .env_remove("ATHENA_NESTED")
        .env_remove("ATHENA_JEV_SHADOW")
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn derive_writes_evidence_labels_with_their_basis_once_and_scores_them() {
    let fx = Fixture::new();
    let mut records = vec![class_record()];
    // Thirty-two cases whose proposed request was sent and landed: evidence for each.
    for i in 1..=32 {
        records.push(rejected_case(i, Some(jev_says(CLASS, 0.9))));
        records.extend(landed_retry(i));
    }
    // A case with no evidence, and a case a person labeled `none` although its retry landed.
    records.push(rejected_case(200, Some(jev_says(CLASS, 0.9))));
    records.push(rejected_case(40, Some(jev_says(CLASS, 0.9))));
    records.extend(landed_retry(40));
    records.push(label(40, None, Labeler::Person));
    commit(&fx, &records);
    let head = fx.head();

    let first = json(&score_jev(&fx, &["--derive", "--json"]));
    let derivation = &first["derivation"];
    assert_eq!(derivation["recorded"], 34);
    assert_eq!(derivation["derived"], 32);
    assert_eq!(derivation["by_kind"]["retry-outcome"], 32);
    assert_eq!(derivation["reviewed"], 1);
    assert_eq!(derivation["no_evidence"], 1);
    assert_eq!(first["sources"]["evidence"], 32);
    assert_eq!(first["sources"]["reviewed"], 1);
    // The unlabeled case is still unreviewed; the others are scored.
    assert_eq!(first["excluded"]["unreviewed"], 1);
    assert_eq!(first["excluded"]["labeled"], 33);
    assert!(first["test"].as_u64().unwrap() >= MIN_TEST_CASES as u64);
    assert!(first["score"].is_object() && first["verdict"].is_object());

    // One new commit that only adds class-label records; no existing file changed or went away.
    let after = fx.head();
    assert_ne!(after, head);
    let changes = fx
        .git
        .run(["diff", "--name-status", &head, &after])
        .unwrap();
    let lines: Vec<&str> = changes.lines().collect();
    assert_eq!(lines.len(), 32);
    assert!(
        lines
            .iter()
            .all(|l| l.starts_with("A\trecords/class-labels/")),
        "{changes}"
    );
    assert_eq!(fx.git.run(["status", "--porcelain"]).unwrap().trim(), "");
    let labels = athena_core::labeled::read_inputs(fx.root()).unwrap().labels;
    let derived: Vec<&ClassLabel> = labels
        .iter()
        .filter(|l| l.labeled_by == Labeler::Evidence)
        .collect();
    assert_eq!(derived.len(), 32);
    for l in derived {
        let [basis] = l.basis.as_slice() else {
            panic!("one piece of evidence: {l:?}");
        };
        assert_eq!(basis.kind.as_str(), "retry-outcome");
        assert!(basis.records.contains(&l.introspection.to_string()));
        assert!(basis.records.iter().any(|r| r.contains("-r0")));
        assert_eq!(l.class.as_ref().map(|c| c.as_str()), Some(CLASS));
    }

    // A second run finds them already derived and commits nothing.
    let second = json(&score_jev(&fx, &["--derive", "--json"]));
    assert_eq!(second["derivation"]["derived"], 0);
    assert_eq!(second["derivation"]["already_derived"], 32);
    assert!(second["derivation"]["commit"].is_null());
    assert_eq!(fx.head(), after);

    // A person's review of a derived case wins and is reported as overriding the evidence label.
    let reviewed = athena(&fx, &["review", &rid(2, 1).to_string(), "--right"]);
    assert!(reviewed.status.success());
    let third = json(&score_jev(&fx, &["--json"]));
    assert_eq!(third["sources"]["evidence"], 31);
    assert_eq!(third["sources"]["reviewed"], 2);
    assert_eq!(third["sources"]["evidence_overridden"], 1);
    let text = String::from_utf8_lossy(&score_jev(&fx, &[]).stdout).into_owned();
    assert!(
        text.contains("labels derived from evidence  31 (by kind: retry-outcome 31)"),
        "{text}"
    );
}

#[test]
fn derive_with_no_evidence_reports_zero_labels_and_no_verdict_and_commits_nothing() {
    let fx = Fixture::new();
    let mut records = vec![class_record()];
    for i in 1..=3 {
        records.push(rejected_case(i, Some(jev_says(CLASS, 0.9))));
    }
    commit(&fx, &records);
    let head = fx.head();
    let report = json(&score_jev(&fx, &["--derive", "--json"]));
    assert_eq!(report["derivation"]["derived"], 0);
    assert_eq!(report["derivation"]["no_evidence"], 3);
    assert_eq!(report["sources"]["evidence"], 0);
    assert!(report["score"].is_null() && report["verdict"].is_null());
    assert!(
        report["insufficient"]
            .as_str()
            .unwrap()
            .contains("at least 10 test cases are needed")
    );
    assert_eq!(fx.head(), head);
    let text = String::from_utf8_lossy(&score_jev(&fx, &["--derive"]).stdout).into_owned();
    assert!(text.contains("0 new labels written"), "{text}");
    assert!(text.contains("labels derived from evidence  0"), "{text}");
}

#[test]
fn a_bad_cut_is_refused() {
    let fx = Fixture::new();
    let out = score_jev(&fx, &["--cut", "yesterday"]);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("RFC 3339"));
}
