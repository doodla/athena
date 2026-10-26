//! `athena review` on fixture records in a temporary repository: the verdict is committed as a class-label
//! record (and a class record for a new class), the introspection file is never touched, `athena status`
//! counts the unreviewed introspections, and every error writes nothing.

mod common;

use athena_core::introspect::{ClassRow, JEV_CLASS_KEY, JEV_NEW_OPTION, JEV_REPEAT_KEY, ask_jev};
use athena_core::jev::{Answer, Evaluation, FakeJev, JevError, Usage};
use athena_core::labeled::{build, read_inputs};
use athena_data::records::{
    AnyRecord, ClassStatus, FailureClass, IntrospectionJev, IntrospectionRecord,
    IntrospectionStatus, IntrospectionTrigger, Labeler, PreventionKind,
};
use athena_data::{JobId, RecordId};
use common::*;
use std::collections::BTreeMap;
use std::process::Command;

const CLASS: &str = "20261001-000000-c001";
const OTHER: &str = "20261001-000000-c002";

fn at(i: u32) -> jiff::Timestamp {
    format!("2026-10-01T{:02}:{:02}:00Z", i / 60, i % 60)
        .parse()
        .unwrap()
}

fn rid(i: u32, n: u32) -> RecordId {
    RecordId::from_parts(at(i), n)
}

fn registry() -> Vec<ClassRow> {
    vec![
        ClassRow {
            id: CLASS.into(),
            name: "Field with no use".into(),
            description: "A new field has no reader.".into(),
        },
        ClassRow {
            id: OTHER.into(),
            name: "Missing test".into(),
            description: "A change lands without a test.".into(),
        },
    ]
}

fn class_record(id: &str, name: &str) -> AnyRecord {
    AnyRecord::FailureClass(FailureClass {
        id: RecordId::parse(id).unwrap(),
        at: at(0),
        name: name.into(),
        description: "A class.".into(),
        status: ClassStatus::Proposed,
        proposed_by: rid(0, 1),
    })
}

fn jev_says(says: &str) -> IntrospectionJev {
    let probabilities = BTreeMap::from([
        (CLASS.to_string(), 0.7),
        (OTHER.to_string(), 0.2),
        (JEV_NEW_OPTION.to_string(), 0.1),
    ]);
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

/// A recorded introspection of class `class`; without a class, a failed one (the model could not run), which is
/// the only kind that has none.
fn introspection(i: u32, class: Option<&str>, jev: Option<IntrospectionJev>) -> AnyRecord {
    let recorded = class.is_some();
    let text = |s: &str| recorded.then(|| s.to_string());
    AnyRecord::Introspection(Box::new(IntrospectionRecord {
        id: rid(i, 1),
        at: at(i),
        trigger: IntrospectionTrigger::Correction,
        job: Some(JobId::parse(&format!("20261001-000000-j{i:03}")).unwrap()),
        intervention: Some(rid(i, 501)),
        class_of_work: "code".into(),
        category: "corrected-output".into(),
        what_happened: text("Added a field with no reader."),
        failure_class: class.map(|c| RecordId::parse(c).unwrap()),
        new_class: false,
        earlier_cases: vec![],
        decisions_matched: vec![],
        cause: text("Nothing ties a field to its first use."),
        prevention_kind: recorded.then_some(PreventionKind::Default),
        prevention: text("Add the field with its reader."),
        proposed_request: text("Add the reader."),
        evidence: None,
        status: if recorded {
            IntrospectionStatus::Recorded
        } else {
            IntrospectionStatus::Failed
        },
        failure: (!recorded).then(|| "the model could not run".to_string()),
        flags: vec![],
        dropped_ids: vec![],
        planning_stopped: false,
        from_jev: false,
        jev,
    }))
}

fn commit(fx: &Fixture, records: &[AnyRecord]) {
    for record in records {
        write(fx.root(), &[(&record.canonical_path(), &record.to_json())]);
    }
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run(["commit", "-q", "-m", "Add fixture records"])
        .unwrap();
}

/// Two classes, introspection 1 (class CLASS, Jev answered), introspection 2 (no class, Jev not asked) and
/// introspection 3 (class CLASS, the Jev call failed).
fn fixture() -> Fixture {
    let fx = Fixture::new();
    let failed = ask_jev(
        &FakeJev::new().then(Err(JevError::Transport("down".into()))),
        "state",
        &registry(),
    );
    commit(
        &fx,
        &[
            class_record(CLASS, "Field with no use"),
            class_record(OTHER, "Missing test"),
            introspection(1, Some(CLASS), Some(jev_says(CLASS))),
            introspection(2, None, None),
            introspection(3, Some(CLASS), Some(failed)),
        ],
    );
    fx
}

fn id(i: u32) -> String {
    rid(i, 1).to_string()
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

fn ok(out: &std::process::Output) -> String {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn json(out: &std::process::Output) -> serde_json::Value {
    serde_json::from_str(&ok(out)).unwrap()
}

fn introspection_bytes(fx: &Fixture, i: u32) -> Vec<u8> {
    let path = introspection(i, None, None).canonical_path();
    std::fs::read(fx.root().join(path)).unwrap()
}

fn labels(fx: &Fixture) -> Vec<athena_data::records::ClassLabel> {
    read_inputs(fx.root()).unwrap().labels
}

fn unreviewed(fx: &Fixture) -> usize {
    build(&read_inputs(fx.root()).unwrap()).excluded.unreviewed
}

#[test]
fn right_records_the_introspections_own_class_with_a_note_and_leaves_the_introspection_alone() {
    let fx = fixture();
    let before = introspection_bytes(&fx, 1);
    assert_eq!(unreviewed(&fx), 3);
    let out = athena(
        &fx,
        &[
            "review",
            &id(1),
            "--right",
            "--note",
            "Clear case.",
            "--json",
        ],
    );
    let review = json(&out);
    assert_eq!(review["label"]["class"], CLASS);
    let [label] = labels(&fx).try_into().unwrap();
    assert_eq!(label.class.as_ref().map(|c| c.as_str()), Some(CLASS));
    assert_eq!(label.labeled_by, Labeler::Person);
    assert_eq!(label.note.as_deref(), Some("Clear case."));
    assert_eq!(label.introspection.to_string(), id(1));
    assert_eq!(introspection_bytes(&fx, 1), before);
    // The commit is clean and holds the label.
    assert_eq!(fx.git.run(["status", "--porcelain"]).unwrap().trim(), "");
    // `labeled::build` no longer counts it as unreviewed.
    assert_eq!(unreviewed(&fx), 2);
}

#[test]
fn another_existing_class_is_recorded_without_a_note() {
    let fx = fixture();
    let before = introspection_bytes(&fx, 1);
    ok(&athena(&fx, &["review", &id(1), "--class", OTHER]));
    let [label] = labels(&fx).try_into().unwrap();
    assert_eq!(label.class.as_ref().map(|c| c.as_str()), Some(OTHER));
    assert_eq!(label.labeled_by, Labeler::Person);
    assert_eq!(label.note, None);
    assert_eq!(introspection_bytes(&fx, 1), before);
    // No note key in the file.
    let path = AnyRecord::ClassLabel(label).canonical_path();
    assert!(!fx.file(&path).unwrap().contains("note"));
    assert_eq!(unreviewed(&fx), 2);
}

#[test]
fn a_new_class_is_added_as_a_proposed_class_in_the_same_commit() {
    let fx = fixture();
    let before = introspection_bytes(&fx, 2);
    let head = fx.head();
    let out = athena(
        &fx,
        &[
            "review",
            &id(2),
            "--new-class",
            "Stale cache",
            "--description",
            "A cache outlives its source.",
            "--note",
            "Nothing in the registry fits.",
            "--json",
        ],
    );
    let review = json(&out);
    let inputs = read_inputs(fx.root()).unwrap();
    assert_eq!(inputs.classes.len(), 3);
    let class = inputs
        .classes
        .iter()
        .find(|c| c.name == "Stale cache")
        .expect("the new class is in the registry");
    assert_eq!(class.description, "A cache outlives its source.");
    let [label] = inputs.labels.as_slice() else {
        panic!("one label")
    };
    assert_eq!(
        label.class.as_ref().map(|c| c.to_string()),
        Some(class.id.clone())
    );
    assert_eq!(label.note.as_deref(), Some("Nothing in the registry fits."));
    assert_eq!(review["new_class"]["status"], "proposed");
    assert_eq!(review["new_class"]["proposed_by"], id(2));
    assert_eq!(introspection_bytes(&fx, 2), before);
    // One commit on top of the fixture carries both records.
    let files = fx.git.run(["diff", "--name-only", &head, "HEAD"]).unwrap();
    assert_eq!(files.lines().count(), 2, "{files}");
    assert_eq!(
        fx.git
            .run(["rev-list", "--count", &format!("{head}..HEAD")])
            .unwrap()
            .trim(),
        "1"
    );
}

#[test]
fn the_output_shows_the_record_and_jev_s_shadow_answer() {
    let fx = fixture();
    let text = ok(&athena(&fx, &["review", &id(1), "--right"]));
    for needle in [
        "Added a field with no reader.",
        "Field with no use (20261001-000000-c001)",
        "Nothing ties a field to its first use.",
        "default: Add the field with its reader.",
        "chose 20261001-000000-c001 (confidence 0.90)",
        "0.70  20261001-000000-c001 (Field with no use)",
        "0.20  20261001-000000-c002 (Missing test)",
        "0.10  new",
        "recorded: class Field with no use",
    ] {
        assert!(text.contains(needle), "{needle:?} not in {text}");
    }
    let not_asked = ok(&athena(&fx, &["review", &id(2), "--class", CLASS]));
    assert!(not_asked.contains("Jev was not asked"), "{not_asked}");
    let failed = ok(&athena(&fx, &["review", &id(3), "--right"]));
    assert!(failed.contains("Jev failed:"), "{failed}");
}

#[test]
fn reviewing_again_adds_a_newer_label_and_says_it_replaces_the_earlier_one() {
    let fx = fixture();
    ok(&athena(&fx, &["review", &id(1), "--right"]));
    let text = ok(&athena(&fx, &["review", &id(1), "--class", OTHER]));
    assert!(text.contains("replaces"), "{text}");
    assert_eq!(labels(&fx).len(), 2);
    let inputs = read_inputs(fx.root()).unwrap();
    let newest = athena_core::labeled::newest_labels(&inputs.labels);
    assert_eq!(
        newest[&id(1)].class.as_ref().map(|c| c.as_str()),
        Some(OTHER)
    );
    assert_eq!(unreviewed(&fx), 2);
}

#[test]
fn a_unique_prefix_names_the_introspection() {
    let fx = fixture();
    let prefix = &id(2)[..15];
    ok(&athena(&fx, &["review", prefix, "--class", CLASS]));
    assert_eq!(labels(&fx).len(), 1);
}

#[test]
fn status_counts_unreviewed_introspections_and_the_count_falls_after_a_review() {
    let fx = fixture();
    let status = json(&athena(&fx, &["status", "--json"]));
    assert_eq!(status["unreviewed_introspections"], 3);
    let next: Vec<&str> = status["next"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap())
        .collect();
    assert!(
        next.iter()
            .any(|n| n.starts_with(&format!("athena review {}", id(1)))),
        "{next:?}"
    );
    let text = ok(&athena(&fx, &["status"]));
    assert!(
        text.contains("unreviewed  3 introspection(s) with no class label"),
        "{text}"
    );
    ok(&athena(&fx, &["review", &id(1), "--right"]));
    let status = json(&athena(&fx, &["status", "--json"]));
    assert_eq!(status["unreviewed_introspections"], 2);
    ok(&athena(&fx, &["review", &id(2), "--class", CLASS]));
    ok(&athena(&fx, &["review", &id(3), "--right"]));
    let status = json(&athena(&fx, &["status", "--json"]));
    assert_eq!(status["unreviewed_introspections"], 0);
    let next = status["next"].as_array().unwrap();
    assert!(
        next.iter()
            .all(|n| !n.as_str().unwrap().starts_with("athena review")),
        "{next:?}"
    );
    let text = ok(&athena(&fx, &["status"]));
    assert!(text.contains("unreviewed  0 introspection(s)"), "{text}");
}

/// Runs `args` and checks that it failed with `needle` in the error and wrote nothing: no commit, no changed
/// or new file.
fn refused(fx: &Fixture, args: &[&str], needle: &str) {
    let head = fx.head();
    let out = athena(fx, args);
    assert!(!out.status.success(), "{args:?} succeeded");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(needle), "{needle:?} not in {stderr}");
    assert_eq!(fx.head(), head);
    assert_eq!(fx.git.run(["status", "--porcelain"]).unwrap().trim(), "");
    assert!(labels(fx).is_empty());
    assert_eq!(read_inputs(fx.root()).unwrap().classes.len(), 2);
}

#[test]
fn errors_write_nothing() {
    let fx = fixture();
    refused(
        &fx,
        &["review", "20261001-999999-zzzz", "--right"],
        "no introspection",
    );
    refused(
        &fx,
        &["review", &id(1), "--class", "20261001-000000-c999"],
        "no class 20261001-000000-c999 in the registry",
    );
    refused(
        &fx,
        &["review", &id(1), "--class", "nonsense"],
        "not a class id",
    );
    refused(&fx, &["review", &id(1)], "say what the class is");
    refused(
        &fx,
        &["review", &id(1), "--right", "--class", OTHER],
        "exactly one",
    );
    refused(
        &fx,
        &[
            "review",
            &id(1),
            "--right",
            "--new-class",
            "X",
            "--description",
            "Y",
        ],
        "exactly one",
    );
    refused(&fx, &["review", &id(2), "--right"], "records no class");
    refused(
        &fx,
        &["review", &id(1), "--right", "--note", "one\ntwo"],
        "single line",
    );
    refused(
        &fx,
        &["review", &id(1), "--new-class", "X"],
        "needs --description",
    );
    refused(
        &fx,
        &[
            "review",
            &id(1),
            "--new-class",
            "Missing test",
            "--description",
            "d",
        ],
        "already has a class named",
    );
    // A prefix shared by every introspection is ambiguous.
    refused(
        &fx,
        &["review", "20261001", "--right"],
        "names 3 introspections",
    );
}
