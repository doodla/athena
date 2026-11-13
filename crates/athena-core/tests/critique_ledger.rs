//! OLY-B-S6-01: a living, machine-readable critique ledger. The mechanism is
//! `crates/athena-core/src/critique_ledger.rs` and `docs/critique-ledger.toml`; this test shows it.

use athena_core::critique_ledger::{LedgerError, check_standard, parse, parse_entries};
use athena_core::hestia::{measure_problem, parse_spec, spec_gaps};
use std::path::Path;

const TEST_COMMAND: &str = "cargo test -p athena-core --test critique_ledger";
const REQUIREMENT: &str = "OLY-B-S6-01";

fn read(rel: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
    .unwrap_or_else(|e| panic!("{rel} reads: {e}"))
}

fn entry(id: &str, kind: &str, subject: &str) -> String {
    format!(
        "[[entry]]\nid = \"{id}\"\nkind = \"{kind}\"\nsubject = \"{subject}\"\nstatus = \"open\"\ntext = \"t\"\nsource = \"s\"\n\n"
    )
}

fn full() -> String {
    entry("a", "critique", "athena")
        + &entry("b", "improvement", "hermes")
        + &entry("c", "open_question", "firstmate")
}

#[test]
fn the_real_ledger_parses_and_covers_every_kind_and_athena() {
    let entries = parse(&read("docs/critique-ledger.toml")).expect("the ledger is valid");
    for kind in ["critique", "improvement", "open_question"] {
        assert!(entries.iter().any(|e| e.kind == kind), "{kind}");
    }
    assert!(entries.iter().any(|e| e.subject == "athena"));
}

#[test]
fn a_valid_small_ledger_passes() {
    assert_eq!(parse(&full()).unwrap().len(), 3);
}

#[test]
fn a_duplicate_id_is_refused() {
    let text = full() + &entry("a", "critique", "hermes");
    assert_eq!(parse(&text), Err(LedgerError::DuplicateId("a".into())));
}

#[test]
fn an_unknown_kind_or_status_is_refused() {
    let bad_kind = entry("x", "praise", "athena");
    assert!(matches!(
        parse_entries(&bad_kind),
        Err(LedgerError::UnknownKind { .. })
    ));
    let bad_status = entry("x", "critique", "athena").replace("open", "pending");
    assert!(matches!(
        parse_entries(&bad_status),
        Err(LedgerError::UnknownStatus { .. })
    ));
}

#[test]
fn an_unknown_key_or_blank_text_is_refused() {
    let extra = full().replacen("source = \"s\"", "source = \"s\"\nseverity = \"high\"", 1);
    assert!(matches!(
        parse_entries(&extra),
        Err(LedgerError::UnknownKey { .. })
    ));
    let blank = entry("x", "critique", "athena").replace("text = \"t\"", "text = \"  \"");
    assert_eq!(
        parse_entries(&blank),
        Err(LedgerError::BlankField {
            index: 0,
            key: "text"
        })
    );
    assert!(matches!(
        parse_entries("[[entry]]\nid = \"x\"\n"),
        Err(LedgerError::BlankField { .. })
    ));
}

#[test]
fn a_ledger_without_an_athena_entry_is_refused() {
    let text = entry("a", "critique", "hermes")
        + &entry("b", "improvement", "hermes")
        + &entry("c", "open_question", "hermes");
    assert_eq!(parse(&text), Err(LedgerError::NoAthenaEntry));
}

#[test]
fn a_ledger_missing_a_kind_is_refused() {
    let text = entry("a", "critique", "athena") + &entry("b", "improvement", "athena");
    assert_eq!(parse(&text), Err(LedgerError::MissingKind("open_question")));
    let entries = parse_entries(&text).unwrap();
    assert!(check_standard(&entries).is_err());
}

#[test]
fn an_empty_or_malformed_ledger_is_refused() {
    assert_eq!(parse(""), Err(LedgerError::Empty));
    assert!(matches!(parse("[[entry"), Err(LedgerError::Syntax(_))));
    assert!(matches!(
        parse("other = 1\n"),
        Err(LedgerError::UnknownTopKey(_))
    ));
}

#[test]
fn spec_entry_has_a_real_mechanism() {
    let requirements = parse_spec(&read("docs/spec.toml")).expect("docs/spec.toml parses");
    let r = requirements
        .iter()
        .find(|r| r.id == REQUIREMENT)
        .expect("docs/spec.toml has OLY-B-S6-01");
    assert_ne!(r.mechanism, "none");
    assert!(
        r.mechanism
            .contains("crates/athena-core/src/critique_ledger.rs"),
        "{}",
        r.mechanism
    );
    assert!(
        r.mechanism.contains("docs/critique-ledger.toml"),
        "{}",
        r.mechanism
    );
    assert_eq!(r.owner, "Athena");
    assert_eq!(measure_problem(&r.measure), None, "{}", r.measure);
    assert!(r.measure.contains(TEST_COMMAND), "{}", r.measure);
    assert!(
        r.measure.trim_end().ends_with("status: not shown"),
        "{}",
        r.measure
    );
    assert!(!spec_gaps(&requirements).iter().any(|g| g.id == REQUIREMENT));
}
