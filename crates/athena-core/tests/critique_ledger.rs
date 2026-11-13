//! OLY-B-S6-01: a living, machine-readable ledger holds critiques, improvements and open questions, and Athena
//! is critiqued to the same standard as the others.
//!
//! The mechanism is `docs/critique-ledger.toml` and the strict parser in
//! `crates/athena-core/src/critique_ledger.rs`. This test parses the real ledger and the real spec entry, and
//! shows each refusal on a mutated copy. Limit: nothing in `athena status` reports the ledger.

use athena_core::critique_ledger::{Kind, check_standard, parse_ledger};
use athena_core::hestia::{SpecRequirement, measure_problem, parse_spec, spec_gaps};
use std::path::Path;

const TEST_COMMAND: &str = "cargo test -p athena-core --test critique_ledger";
const REQUIREMENT: &str = "OLY-B-S6-01";

fn repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn real_requirement() -> SpecRequirement {
    parse_spec(&repo_file("docs/spec.toml"))
        .expect("docs/spec.toml parses")
        .into_iter()
        .find(|r| r.id == REQUIREMENT)
        .expect("docs/spec.toml has OLY-B-S6-01")
}

fn entry(id: &str, kind: &str, subject: &str) -> String {
    format!(
        "[[entry]]\nid = \"{id}\"\nkind = \"{kind}\"\nsubject = \"{subject}\"\nstatus = \"open\"\ntext = \"t\"\nsource = \"s\"\n\n"
    )
}

fn complete() -> String {
    entry("a", "critique", "athena")
        + &entry("b", "improvement", "x")
        + &entry("c", "open_question", "x")
}

fn refusal(text: &str) -> String {
    parse_ledger(text).expect_err("must be refused").to_string()
}

#[test]
fn the_real_ledger_parses_and_meets_the_standard() {
    let entries =
        parse_ledger(&repo_file("docs/critique-ledger.toml")).expect("real ledger parses");
    assert_eq!(check_standard(&entries), Vec::<String>::new());
    for kind in Kind::ALL {
        assert!(entries.iter().any(|e| e.kind == kind), "{kind:?}");
    }
    assert!(entries.iter().any(|e| e.subject == "athena"));
}

#[test]
fn a_complete_ledger_passes() {
    let entries = parse_ledger(&complete()).unwrap();
    assert_eq!(entries.len(), 3);
    assert!(check_standard(&entries).is_empty());
}

#[test]
fn a_repeated_id_is_refused() {
    let text = complete() + &entry("a", "critique", "x");
    assert!(refusal(&text).contains("`a` twice"));
}

#[test]
fn a_bad_kind_is_refused() {
    let text = complete().replacen("\"critique\"", "\"praise\"", 1);
    assert!(refusal(&text).contains("kind `praise`"));
}

#[test]
fn a_bad_status_is_refused() {
    let text = complete().replacen("\"open\"", "\"done\"", 1);
    assert!(refusal(&text).contains("status `done`"));
}

#[test]
fn blank_text_is_refused() {
    let text = complete().replacen("text = \"t\"", "text = \"  \"", 1);
    assert!(refusal(&text).contains("blank `text`"));
}

#[test]
fn an_unknown_key_is_refused() {
    let text = complete().replacen("source = \"s\"", "source = \"s\"\nseverity = \"high\"", 1);
    assert!(refusal(&text).contains("`severity`"));
    assert!(refusal(&format!("note = \"x\"\n{}", complete())).contains("`note`"));
}

#[test]
fn a_missing_field_is_refused() {
    let text = complete().replacen("source = \"s\"\n", "", 1);
    assert!(refusal(&text).contains("`source`"));
}

#[test]
fn a_ledger_without_an_athena_entry_fails_the_standard() {
    let text = complete().replacen("\"athena\"", "\"other\"", 1);
    let gaps = check_standard(&parse_ledger(&text).unwrap());
    assert_eq!(gaps.len(), 1);
    assert!(gaps[0].contains("athena"), "{gaps:?}");
}

#[test]
fn a_ledger_missing_a_kind_fails_the_standard() {
    let text = entry("a", "critique", "athena") + &entry("b", "improvement", "x");
    let gaps = check_standard(&parse_ledger(&text).unwrap());
    assert_eq!(gaps.len(), 1);
    assert!(gaps[0].contains("OpenQuestion"), "{gaps:?}");
    assert_eq!(check_standard(&[]).len(), 4);
}

#[test]
fn spec_entry_has_a_real_mechanism() {
    let requirements = parse_spec(&repo_file("docs/spec.toml")).expect("docs/spec.toml parses");
    let r = real_requirement();
    assert_ne!(r.mechanism, "none");
    assert!(
        r.mechanism
            .contains("crates/athena-core/src/critique_ledger.rs"),
        "{}",
        r.mechanism
    );
    assert!(r.mechanism.contains("docs/critique-ledger.toml"));
    assert_eq!(r.owner, "Athena");
    assert_eq!(measure_problem(&r.measure), None, "{}", r.measure);
    assert!(r.measure.trim_end().ends_with("status: not shown"));
    assert!(r.measure.contains(TEST_COMMAND));
    assert!(!spec_gaps(&requirements).iter().any(|g| g.id == REQUIREMENT));
}

#[test]
fn spec_check_refuses_a_none_mechanism() {
    let mut r = real_requirement();
    r.mechanism = "none".into();
    let gaps = spec_gaps(&[r]);
    assert_eq!(gaps.len(), 1);
    assert!(gaps[0].no_mechanism);
}
