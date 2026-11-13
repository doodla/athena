//! OLY-B-S3-02: every divergence between documented and observed behavior is a finding.

use athena_core::divergence::{Claim, Divergence, Observation, reconcile};
use std::path::Path;

fn claim(key: &str, expected: &str) -> Claim {
    Claim {
        key: key.into(),
        source: "docs/example.md".into(),
        text: format!("{key} is {expected}"),
        expected: expected.into(),
    }
}

fn seen(key: &str, observed: &str) -> Observation {
    Observation {
        key: key.into(),
        observed: observed.into(),
    }
}

#[test]
fn matching_input_gives_no_findings() {
    let claims = [claim("a", "1"), claim("b", "2")];
    let observations = [seen("a", "1"), seen("b", "2")];
    assert!(reconcile(&claims, &observations).is_empty());
}

#[test]
fn a_differing_value_is_a_mismatch() {
    let found = reconcile(&[claim("a", "1")], &[seen("a", "9")]);
    assert_eq!(
        found,
        vec![Divergence::Mismatch {
            claim: claim("a", "1"),
            observed: "9".into()
        }]
    );
}

#[test]
fn a_claim_with_no_observation_is_unobserved() {
    let found = reconcile(&[claim("a", "1")], &[]);
    assert_eq!(
        found,
        vec![Divergence::Unobserved {
            claim: claim("a", "1")
        }]
    );
}

#[test]
fn an_observation_with_no_claim_is_undocumented() {
    let found = reconcile(&[], &[seen("z", "7")]);
    assert_eq!(
        found,
        vec![Divergence::Undocumented {
            key: "z".into(),
            observed: "7".into()
        }]
    );
}

#[test]
fn all_three_kinds_are_reported_together_and_none_is_dropped() {
    let claims = [claim("ok", "1"), claim("bad", "1"), claim("lost", "1")];
    let observations = [seen("ok", "1"), seen("bad", "2"), seen("extra", "3")];
    let found = reconcile(&claims, &observations);
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(matches!(found[0], Divergence::Mismatch { .. }));
    assert!(matches!(found[1], Divergence::Unobserved { .. }));
    assert!(matches!(found[2], Divergence::Undocumented { .. }));
    let keys: Vec<&str> = found.iter().map(Divergence::key).collect();
    assert_eq!(keys, ["bad", "lost", "extra"]);
}

#[test]
fn spec_entry_has_a_real_mechanism() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/spec.toml");
    let spec = std::fs::read_to_string(&path).expect("docs/spec.toml");
    let start = spec
        .find("id = \"OLY-B-S3-02\"")
        .expect("docs/spec.toml has OLY-B-S3-02");
    let rest = &spec[start..];
    let entry = &rest[..rest.find("[[requirement]]").unwrap_or(rest.len())];
    assert!(!entry.contains("mechanism = \"none\""), "{entry}");
    assert!(
        entry.contains("crates/athena-core/src/divergence.rs"),
        "{entry}"
    );
    assert!(entry.contains("reconcile"), "{entry}");
    assert!(entry.contains("owner = \"Athena\""), "{entry}");
    let measure = entry
        .lines()
        .find_map(|line| line.strip_prefix("measure = "))
        .expect("the entry has a measure line")
        .trim()
        .trim_end_matches('"');
    assert!(measure.ends_with("status: not shown"), "{measure}");
    assert!(
        measure.contains("crates/athena-core/tests/divergence.rs"),
        "{measure}"
    );
}
