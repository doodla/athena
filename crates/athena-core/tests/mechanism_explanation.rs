//! OLY-B-S4-01: each mechanism explanation cites a prompt, code and run source, and, for Athena, its own records.
//!
//! The mechanism is `athena_core::mechanism_explanation::check`, a pure function. These tests drive it with
//! complete and defective explanations and check the real `docs/spec.toml` entry.
//!
//! Limits: a record source stands in for "primary" evidence; a mechanism with no explanation is not detected;
//! nothing yet feeds `check` a real study of Athena, so the measure ends `status: not shown`.

use athena_core::hestia::{SpecRequirement, measure_problem, parse_spec, spec_gaps};
use athena_core::mechanism_explanation::{Evidence, EvidenceKind, Explanation, Gap, check};
use std::path::{Path, PathBuf};

const REQUIREMENT: &str = "OLY-B-S4-01";
const TEST_COMMAND: &str = "cargo test -p athena-core --test mechanism_explanation";

fn repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    let path: PathBuf = path;
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn owner_problem(r: &SpecRequirement) -> Option<String> {
    (r.owner != "Athena").then(|| format!("the owner is `{}`, not Athena", r.owner))
}

fn measure_label_problem(r: &SpecRequirement) -> Option<String> {
    if !r.measure.trim_end().ends_with("status: not shown") {
        return Some("the measure does not end with `status: not shown`".into());
    }
    (!r.measure.contains(TEST_COMMAND))
        .then(|| format!("the measure does not name `{TEST_COMMAND}`"))
}

fn real_requirement() -> SpecRequirement {
    parse_spec(&repo_file("docs/spec.toml"))
        .expect("docs/spec.toml parses")
        .into_iter()
        .find(|r| r.id == REQUIREMENT)
        .expect("docs/spec.toml has OLY-B-S4-01")
}

fn ev(kind: EvidenceKind, source: &str) -> Evidence {
    Evidence {
        kind,
        source: source.into(),
    }
}

fn explanation(instance: &str, mechanism: &str, evidence: Vec<Evidence>) -> Explanation {
    Explanation {
        instance: instance.into(),
        mechanism: mechanism.into(),
        evidence,
    }
}

fn missing(instance: &str, mechanism: &str, kind: EvidenceKind) -> Gap {
    Gap::MissingEvidence {
        instance: instance.into(),
        mechanism: mechanism.into(),
        kind,
    }
}

fn base() -> Vec<Evidence> {
    vec![
        ev(EvidenceKind::Prompt, "prompts/a.md"),
        ev(EvidenceKind::Code, "src/a.rs"),
        ev(EvidenceKind::Run, "run-1"),
    ]
}

fn without(kind: EvidenceKind) -> Vec<Evidence> {
    base().into_iter().filter(|e| e.kind != kind).collect()
}

#[test]
fn complete_explanation_has_no_gaps() {
    let mut full = base();
    full.push(ev(EvidenceKind::Record, "records/jobs/x.json"));
    assert!(check(&[explanation("athena", "m", full)]).is_empty());
    assert!(check(&[explanation("hermes", "m", base())]).is_empty());
}

#[test]
fn missing_prompt_is_reported() {
    let got = check(&[explanation("hermes", "m", without(EvidenceKind::Prompt))]);
    assert_eq!(got, vec![missing("hermes", "m", EvidenceKind::Prompt)]);
}

#[test]
fn missing_code_is_reported() {
    let got = check(&[explanation("hermes", "m", without(EvidenceKind::Code))]);
    assert_eq!(got, vec![missing("hermes", "m", EvidenceKind::Code)]);
}

#[test]
fn missing_run_is_reported() {
    let got = check(&[explanation("hermes", "m", without(EvidenceKind::Run))]);
    assert_eq!(got, vec![missing("hermes", "m", EvidenceKind::Run)]);
}

#[test]
fn blank_source_does_not_count() {
    let mut e = without(EvidenceKind::Code);
    e.push(ev(EvidenceKind::Code, "  "));
    let got = check(&[explanation("hermes", "m", e)]);
    assert_eq!(got, vec![missing("hermes", "m", EvidenceKind::Code)]);
}

#[test]
fn athena_with_only_prompt_code_run_is_reported() {
    let got = check(&[explanation("athena", "m", base())]);
    assert_eq!(
        got,
        vec![Gap::NoOwnRecords {
            instance: "athena".into(),
            mechanism: "m".into()
        }]
    );
}

#[test]
fn athena_with_a_record_is_complete() {
    let mut e = base();
    e.push(ev(EvidenceKind::Record, "records/jobs/x.json"));
    assert!(check(&[explanation("athena", "m", e)]).is_empty());
}

#[test]
fn athena_match_ignores_case() {
    let got = check(&[explanation("Athena", "m", base())]);
    assert_eq!(got.len(), 1);
    assert!(matches!(got[0], Gap::NoOwnRecords { .. }));
}

#[test]
fn non_athena_without_records_is_not_flagged() {
    assert!(check(&[explanation("firstmate", "m", base())]).is_empty());
}

#[test]
fn several_defects_give_gaps_in_order() {
    let got = check(&[
        explanation("hermes", "one", vec![]),
        explanation("athena", "two", vec![ev(EvidenceKind::Prompt, "p")]),
    ]);
    assert_eq!(
        got,
        vec![
            missing("hermes", "one", EvidenceKind::Prompt),
            missing("hermes", "one", EvidenceKind::Code),
            missing("hermes", "one", EvidenceKind::Run),
            missing("athena", "two", EvidenceKind::Code),
            missing("athena", "two", EvidenceKind::Run),
            Gap::NoOwnRecords {
                instance: "athena".into(),
                mechanism: "two".into()
            },
        ]
    );
}

#[test]
fn empty_input_gives_no_gaps() {
    // The stated limit: a mechanism with no explanation at all is not detected.
    assert!(check(&[]).is_empty());
}

#[test]
fn spec_entry_has_a_real_mechanism() {
    let requirements = parse_spec(&repo_file("docs/spec.toml")).expect("docs/spec.toml parses");
    let r = real_requirement();
    assert_ne!(r.mechanism, "none");
    for part in [
        "crates/athena-core/src/mechanism_explanation.rs",
        "crates/athena-core/tests/mechanism_explanation.rs",
        "check",
    ] {
        assert!(
            r.mechanism.contains(part),
            "{part} missing: {}",
            r.mechanism
        );
    }
    assert_eq!(owner_problem(&r), None);
    assert_eq!(measure_problem(&r.measure), None, "{}", r.measure);
    assert_eq!(measure_label_problem(&r), None, "{}", r.measure);
    assert!(r.measure.contains("not detected"), "{}", r.measure);
    assert!(
        !spec_gaps(&requirements).iter().any(|g| g.id == REQUIREMENT),
        "OLY-B-S4-01 is still a spec gap"
    );
}

#[test]
fn spec_check_refuses_a_none_mechanism() {
    let mut r = real_requirement();
    r.mechanism = "none".into();
    let gaps = spec_gaps(&[r]);
    assert_eq!(gaps.len(), 1);
    assert!(gaps[0].no_mechanism);
}

#[test]
fn spec_check_refuses_a_wrong_status_label() {
    let mut r = real_requirement();
    r.measure = format!("The spec gap list; {TEST_COMMAND}; status: requirement");
    assert!(measure_label_problem(&r).is_some());
}

#[test]
fn spec_check_refuses_a_measure_without_the_command() {
    let mut r = real_requirement();
    r.measure = "Checked on the real files; status: not shown".into();
    assert!(measure_label_problem(&r).is_some());
}

#[test]
fn spec_check_refuses_a_wrong_owner() {
    let mut r = real_requirement();
    r.owner = "Hestia".into();
    assert!(owner_problem(&r).is_some());
}
