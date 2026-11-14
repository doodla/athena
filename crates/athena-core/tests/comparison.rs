//! OLY-B-S7-01: the study maps all three systems onto shared concepts, compares them mechanism by mechanism,
//! and closes with a short essay on building orchestration from prompts.
//!
//! The mechanism is `athena_core::comparison::check`, a pure function. These tests drive it with complete and
//! defective comparisons and check the real `docs/spec.toml` entry.
//!
//! Limits: a concept or mechanism left out is not detected; nothing yet feeds `check` a real three-system
//! study, so the measure ends `status: not shown`.

use athena_core::comparison::{
    Comparison, Concept, ESSAY_WORD_CAP, Entry, Essay, Gap, MechanismComparison, check,
};
use athena_core::hestia::{SpecRequirement, measure_problem, parse_spec, spec_gaps};
use std::path::{Path, PathBuf};

const REQUIREMENT: &str = "OLY-B-S7-01";
const TEST_COMMAND: &str = "cargo test -p athena-core --test comparison";
const SYSTEMS: [&str; 3] = ["athena", "firstmate", "hermes"];

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
        .expect("docs/spec.toml has OLY-B-S7-01")
}

fn entry(system: &str, text: &str) -> Entry {
    Entry {
        system: system.into(),
        text: text.into(),
    }
}

fn entries_without(skip: Option<&str>) -> Vec<Entry> {
    SYSTEMS
        .iter()
        .filter(|s| Some(**s) != skip)
        .map(|s| entry(s, "does it"))
        .collect()
}

fn concept(entries: Vec<Entry>) -> Concept {
    Concept {
        name: "c".into(),
        entries,
    }
}

fn mech(entries: Vec<Entry>) -> MechanismComparison {
    MechanismComparison {
        mechanism: "m".into(),
        entries,
    }
}

fn essay(text: &str) -> Option<Essay> {
    Some(Essay { text: text.into() })
}

const GOOD_ESSAY: &str = "Building orchestration from prompts works when each prompt is small.";

fn complete() -> Comparison {
    Comparison {
        concepts: vec![concept(entries_without(None))],
        comparisons: vec![mech(entries_without(None))],
        essay: essay(GOOD_ESSAY),
    }
}

fn words(n: usize) -> String {
    let mut s = String::from("prompt orchestration");
    for _ in 2..n {
        s.push_str(" word");
    }
    s
}

#[test]
fn complete_comparison_has_no_gaps() {
    assert!(check(&complete()).is_empty());
}

#[test]
fn each_missing_system_in_a_concept_is_reported() {
    for system in SYSTEMS {
        let mut c = complete();
        c.concepts = vec![concept(entries_without(Some(system)))];
        assert_eq!(
            check(&c),
            vec![Gap::ConceptMissingSystem {
                concept: "c".into(),
                system: system.into()
            }]
        );
    }
}

#[test]
fn each_missing_system_in_a_mechanism_comparison_is_reported() {
    for system in SYSTEMS {
        let mut c = complete();
        c.comparisons = vec![mech(entries_without(Some(system)))];
        assert_eq!(
            check(&c),
            vec![Gap::ComparisonMissingSystem {
                mechanism: "m".into(),
                system: system.into()
            }]
        );
    }
}

#[test]
fn no_concepts_is_reported() {
    let mut c = complete();
    c.concepts.clear();
    assert_eq!(check(&c), vec![Gap::NoConcepts]);
}

#[test]
fn no_comparisons_is_reported() {
    let mut c = complete();
    c.comparisons.clear();
    assert_eq!(check(&c), vec![Gap::NoComparisons]);
}

#[test]
fn absent_essay_is_reported() {
    let mut c = complete();
    c.essay = None;
    assert_eq!(check(&c), vec![Gap::NoEssay]);
}

#[test]
fn blank_essay_is_reported() {
    let mut c = complete();
    c.essay = essay("  \n ");
    assert_eq!(check(&c), vec![Gap::NoEssay]);
}

#[test]
fn essay_at_the_cap_passes_and_one_over_fails() {
    let mut c = complete();
    c.essay = essay(&words(ESSAY_WORD_CAP));
    assert!(check(&c).is_empty());
    c.essay = essay(&words(ESSAY_WORD_CAP + 1));
    assert_eq!(
        check(&c),
        vec![Gap::EssayTooLong {
            words: ESSAY_WORD_CAP + 1,
            cap: ESSAY_WORD_CAP
        }]
    );
}

#[test]
fn essay_must_mention_both_terms() {
    let mut c = complete();
    c.essay = essay("Orchestration is hard.");
    assert_eq!(check(&c), vec![Gap::EssayOffTopic]);
    c.essay = essay("Prompts are useful.");
    assert_eq!(check(&c), vec![Gap::EssayOffTopic]);
    c.essay = essay("PROMPTS and ORCHESTRATING them.");
    assert!(check(&c).is_empty());
}

#[test]
fn blank_entry_text_does_not_count() {
    let mut e = entries_without(Some("hermes"));
    e.push(entry("hermes", "   "));
    let mut c = complete();
    c.concepts = vec![concept(e)];
    assert_eq!(
        check(&c),
        vec![Gap::ConceptMissingSystem {
            concept: "c".into(),
            system: "hermes".into()
        }]
    );
}

#[test]
fn unknown_system_name_does_not_count() {
    let mut e = entries_without(Some("hermes"));
    e.push(entry("other", "does it"));
    let mut c = complete();
    c.comparisons = vec![mech(e)];
    assert_eq!(
        check(&c),
        vec![Gap::ComparisonMissingSystem {
            mechanism: "m".into(),
            system: "hermes".into()
        }]
    );
}

#[test]
fn extra_entries_and_case_are_tolerated() {
    let mut e = entries_without(None);
    e.push(entry("other", "x"));
    e[0].system = "ATHENA".into();
    let mut c = complete();
    c.concepts = vec![concept(e)];
    assert!(check(&c).is_empty());
}

#[test]
fn several_defects_give_gaps_in_order() {
    let c = Comparison {
        concepts: vec![],
        comparisons: vec![mech(entries_without(Some("athena")))],
        essay: None,
    };
    assert_eq!(
        check(&c),
        vec![
            Gap::NoConcepts,
            Gap::ComparisonMissingSystem {
                mechanism: "m".into(),
                system: "athena".into()
            },
            Gap::NoEssay,
        ]
    );
}

#[test]
fn module_doc_lists_its_limits() {
    let src = repo_file("crates/athena-core/src/comparison.rs");
    assert!(src.starts_with("//! OLY-B-S7-01"));
    assert!(src.contains("//! Limits:"));
}

#[test]
fn spec_entry_has_a_real_mechanism() {
    let requirements = parse_spec(&repo_file("docs/spec.toml")).expect("docs/spec.toml parses");
    let r = real_requirement();
    assert_ne!(r.mechanism, "none");
    for part in [
        "crates/athena-core/src/comparison.rs",
        "crates/athena-core/tests/comparison.rs",
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
        "OLY-B-S7-01 is still a spec gap"
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
