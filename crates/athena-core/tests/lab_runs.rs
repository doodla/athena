//! OLY-B-S3-01: a lab run is tied to a recorded configuration with a spend cap for each of the three systems.

use athena_core::lab::{LabConfig, LabError};
use std::path::Path;

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

fn config_text(commit_line: &str, caps: &str) -> String {
    format!("task = \"spec gap probe\"\n{commit_line}\n[caps_usd]\n{caps}\n")
}

fn valid() -> String {
    config_text(
        &format!("commit = \"{COMMIT}\""),
        "athena = 5.0\nfirstmate = 4\nhermes = 2.5",
    )
}

#[test]
fn a_valid_config_is_accepted() {
    let config = LabConfig::parse(&valid()).expect("valid config");
    assert_eq!(config.caps_usd.len(), 3);
    assert_eq!(config.caps_usd["firstmate"], 4.0);
    assert_eq!(config.commit, COMMIT);
}

#[test]
fn a_missing_system_is_refused() {
    let text = config_text(
        &format!("commit = \"{COMMIT}\""),
        "athena = 5.0\nfirstmate = 4.0",
    );
    assert_eq!(
        LabConfig::parse(&text).unwrap_err(),
        LabError::MissingSystem("hermes")
    );
}

#[test]
fn a_zero_or_negative_cap_is_refused() {
    for bad in ["0.0", "-1.0"] {
        let text = config_text(
            &format!("commit = \"{COMMIT}\""),
            &format!("athena = {bad}\nfirstmate = 4.0\nhermes = 2.0"),
        );
        assert_eq!(
            LabConfig::parse(&text).unwrap_err(),
            LabError::BadCap("athena".into()),
            "cap {bad}"
        );
    }
}

#[test]
fn a_missing_or_unpinned_commit_is_refused() {
    let caps = "athena = 1.0\nfirstmate = 1.0\nhermes = 1.0";
    assert_eq!(
        LabConfig::parse(&config_text("", caps)).unwrap_err(),
        LabError::NoCommit
    );
    assert_eq!(
        LabConfig::parse(&config_text("commit = \"main\"", caps)).unwrap_err(),
        LabError::NoCommit
    );
}

#[test]
fn a_run_at_its_cap_is_refused() {
    let config = LabConfig::parse(&valid()).unwrap();
    config.check_run("athena", 4.99).expect("below the cap");
    assert!(matches!(
        config.check_run("athena", 5.0),
        Err(LabError::CapSpent { .. })
    ));
    assert!(matches!(
        config.check_run("hermes", 9.0),
        Err(LabError::CapSpent { .. })
    ));
}

#[test]
fn the_fingerprint_follows_the_configuration() {
    let a = LabConfig::parse(&valid()).unwrap();
    let same = LabConfig::parse(&valid()).unwrap();
    assert_eq!(a.fingerprint(), same.fingerprint());
    assert_eq!(a.fingerprint().len(), 16);
    let changed = LabConfig::parse(&valid().replace("athena = 5.0", "athena = 6.0")).unwrap();
    assert_ne!(a.fingerprint(), changed.fingerprint());
}

#[test]
fn spec_entry_has_a_real_mechanism() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec.toml");
    let spec = std::fs::read_to_string(&path).expect("docs/spec.toml");
    let start = spec
        .find("id = \"OLY-B-S3-01\"")
        .expect("docs/spec.toml has OLY-B-S3-01");
    let rest = &spec[start..];
    let entry = &rest[..rest.find("[[requirement]]").unwrap_or(rest.len())];
    assert!(!entry.contains("mechanism = \"none\""), "{entry}");
    assert!(entry.contains("crates/athena-core/src/lab.rs"), "{entry}");
    assert!(entry.contains("owner = \"Athena\""), "{entry}");
    let measure = entry
        .lines()
        .find_map(|line| line.strip_prefix("measure = "))
        .expect("the entry has a measure line")
        .trim()
        .trim_end_matches('"');
    assert!(measure.ends_with("status: not shown"), "{measure}");
    assert!(
        measure.contains("crates/athena-core/tests/lab_runs.rs"),
        "{measure}"
    );
}
