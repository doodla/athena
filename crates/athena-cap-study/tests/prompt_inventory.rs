//! OLY-B-S2-01: every studied instance has a prompt-surface inventory built by code, reproducibly.
//!
//! The test runs `athena_cap_study::prompts_inventory::inventory` twice over every real `studies/<id>/`
//! directory and asserts the two results are equal and not empty, and checks the spec entry. It sits in the
//! capability crate because `athena-core` cannot depend on it.
//!
//! Limit: that each later instance is actually inventoried is not visible in static files, so the measure
//! ends `status: not shown`.

use athena_cap_study::prompts_inventory::{PromptSurface, inventory};
use athena_cap_study::records::{Claim, Rationale};
use athena_core::hestia::{SpecRequirement, measure_problem, parse_spec, spec_gaps};
use std::path::{Path, PathBuf};

const REQUIREMENT: &str = "OLY-B-S2-01";
const TEST_COMMAND: &str = "cargo test -p athena-cap-study --test prompt_inventory";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn instance_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(repo_root().join("studies"))
        .expect("studies/ exists")
        .map(|e| e.expect("entry").path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    dirs
}

fn inventory_of(dir: &Path) -> Vec<PromptSurface> {
    let claims: Vec<Claim> = serde_json::from_str(&read(&dir.join("claims.json"))).expect("claims");
    let cards: Vec<Rationale> =
        serde_json::from_str(&read(&dir.join("rationales.json"))).expect("rationales");
    inventory(&claims, &cards)
}

fn spec_problem(r: &SpecRequirement) -> Option<String> {
    if r.mechanism.trim() == "none" || r.mechanism.trim().is_empty() {
        return Some("the mechanism is none".into());
    }
    for part in [
        "crates/athena-cap-study/src/prompts_inventory.rs",
        "crates/athena-cap-study/tests/prompt_inventory.rs",
    ] {
        if !r.mechanism.contains(part) {
            return Some(format!("the mechanism does not cite {part}"));
        }
        if !repo_root().join(part).is_file() {
            return Some(format!("{part} does not exist"));
        }
    }
    if r.owner != "Athena" {
        return Some("the owner is not Athena".into());
    }
    if !r.measure.trim_end().ends_with("status: not shown") || !r.measure.contains(TEST_COMMAND) {
        return Some("the measure lacks the test command or `status: not shown`".into());
    }
    measure_problem(&r.measure)
}

fn real_requirement() -> SpecRequirement {
    parse_spec(&read(&repo_root().join("docs/spec.toml")))
        .expect("docs/spec.toml parses")
        .into_iter()
        .find(|r| r.id == REQUIREMENT)
        .expect("docs/spec.toml has OLY-B-S2-01")
}

#[test]
fn every_studied_instance_has_a_reproducible_non_empty_inventory() {
    let dirs = instance_dirs();
    assert!(!dirs.is_empty(), "no studies/<id>/ directory found");
    for dir in dirs {
        let first = inventory_of(&dir);
        let second = inventory_of(&dir);
        assert!(!first.is_empty(), "{} has no prompt surface", dir.display());
        assert_eq!(first, second, "{} is not reproducible", dir.display());
        let paths: Vec<&str> = first.iter().map(|s| s.path.as_str()).collect();
        let mut sorted = paths.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(paths, sorted, "inventory is sorted and unique");
    }
}

#[test]
fn the_spec_entry_has_the_mechanism_owner_and_measure() {
    let r = real_requirement();
    assert_eq!(spec_problem(&r), None, "{r:?}");
    assert!(!spec_gaps(&[r]).iter().any(|g| g.id == REQUIREMENT));
}

#[test]
fn a_none_mechanism_is_refused() {
    let mut r = real_requirement();
    r.mechanism = "none".into();
    assert!(spec_problem(&r).is_some());
    assert!(spec_gaps(&[r]).iter().any(|g| g.no_mechanism));
}

#[test]
fn a_wrong_owner_or_measure_is_refused() {
    let mut r = real_requirement();
    r.owner = "Hestia".into();
    assert!(spec_problem(&r).is_some());
    let mut r = real_requirement();
    r.measure = format!("{TEST_COMMAND}; status: requirement");
    assert!(spec_problem(&r).is_some());
}

#[test]
fn an_empty_inventory_is_detected() {
    // A study with no claims has no prompt surface, so the check above would fail on it.
    assert!(inventory(&[], &[]).is_empty());
}
