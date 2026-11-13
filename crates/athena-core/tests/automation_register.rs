//! OLY-B-A9-01: each automation is registered with its savings, and those that cost more than they save are
//! listed for retirement. The mechanism is `crates/athena-core/src/automation.rs`; this test shows it.

use athena_core::automation::{Automation, register, registration_problem, review};
use athena_core::hestia::{measure_problem, parse_spec, spec_gaps};
use std::path::Path;

const TEST_COMMAND: &str = "cargo test -p athena-core --test automation_register";
const REQUIREMENT: &str = "OLY-B-A9-01";

fn entry(name: &str, cost: u64, savings: Option<u64>) -> Automation {
    Automation {
        name: name.into(),
        owner: "Athena".into(),
        cost_minutes: cost,
        savings_minutes: savings,
        source: "a repeated manual step".into(),
    }
}

#[test]
fn an_automation_costing_more_than_it_saves_is_listed_for_retirement() {
    let entries = vec![
        entry("costly", 120, Some(30)),
        entry("worth-it", 30, Some(120)),
    ];
    let r = review(&entries);
    assert_eq!(r.retire.len(), 1);
    assert_eq!(r.retire[0].name, "costly");
    assert!(r.unregistered.is_empty());
}

#[test]
fn an_automation_saving_more_than_it_costs_is_kept() {
    let entries = vec![
        entry("worth-it", 30, Some(120)),
        entry("even", 50, Some(50)),
    ];
    let r = review(&entries);
    assert!(r.retire.is_empty(), "{r:?}");
}

#[test]
fn an_entry_with_no_savings_is_refused() {
    let mut reg = Vec::new();
    let err = register(&mut reg, entry("unmeasured", 10, None)).unwrap_err();
    assert!(err.contains("no savings registered"), "{err}");
    assert!(reg.is_empty());
    assert!(registration_problem(&entry("ok", 10, Some(20))).is_none());
    register(&mut reg, entry("ok", 10, Some(20))).unwrap();
    assert_eq!(reg.len(), 1);
    // One that reached the list anyway is reported as unregistered, not kept.
    let unmeasured = [entry("unmeasured", 10, None)];
    let r = review(&unmeasured);
    assert_eq!(r.unregistered.len(), 1);
    assert!(r.retire.is_empty());
}

#[test]
fn an_entry_without_a_source_is_refused() {
    let mut e = entry("no-source", 10, Some(20));
    e.source = " ".into();
    assert!(register(&mut Vec::new(), e).is_err());
}

#[test]
fn spec_entry_has_a_real_mechanism() {
    let text =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec.toml"))
            .expect("docs/spec.toml reads");
    let requirements = parse_spec(&text).expect("docs/spec.toml parses");
    let r = requirements
        .iter()
        .find(|r| r.id == REQUIREMENT)
        .expect("docs/spec.toml has OLY-B-A9-01");
    assert_ne!(r.mechanism, "none");
    assert!(
        r.mechanism.contains("crates/athena-core/src/automation.rs"),
        "{}",
        r.mechanism
    );
    assert!(
        r.mechanism
            .contains("crates/athena-core/tests/automation_register.rs"),
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

#[test]
fn spec_check_refuses_a_none_mechanism() {
    let text =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec.toml"))
            .expect("docs/spec.toml reads");
    let mut r = parse_spec(&text)
        .unwrap()
        .into_iter()
        .find(|r| r.id == REQUIREMENT)
        .unwrap();
    r.mechanism = "none".into();
    let gaps = spec_gaps(&[r]);
    assert_eq!(gaps.len(), 1);
    assert!(gaps[0].no_mechanism);
}
