//! OLY-B-STD-03: Athena adopts good, maintained options and builds only for needs they do not meet.
//!
//! The mechanism ties the external crates in the root `[workspace.dependencies]` to the Decision bullets of the
//! "Buy before build" entries in `docs/decisions.md` (D36 and the entry appended after it). A crate counts as
//! recorded only as an exact backticked name in such a Decision bullet. Limits: only the root manifest is read,
//! the check is one-way, and whether a maintained option exists stays a verifier judgement.

use athena_core::hestia::{SpecRequirement, measure_problem, parse_spec, spec_gaps};
use std::path::Path;

const TEST_COMMAND: &str = "cargo test -p athena-core --test buy_first";
const REQUIREMENT: &str = "OLY-B-STD-03";

fn repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// The `[workspace.dependencies]` keys that have no `path` key, in file order.
fn external_crates(cargo_toml: &str) -> Vec<String> {
    let doc: toml_edit::DocumentMut = cargo_toml.parse().expect("Cargo.toml parses");
    let deps = doc
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(toml_edit::Item::as_table_like)
        .expect("Cargo.toml has [workspace.dependencies]");
    deps.iter()
        .filter(|(_, value)| {
            value
                .as_table_like()
                .map(|t| !t.contains_key("path"))
                .unwrap_or(true)
        })
        .map(|(name, _)| name.to_string())
        .collect()
}

/// The Decision bullets of every entry whose heading contains "Buy before build".
fn adopted_record(decisions: &str) -> String {
    let mut out = String::new();
    let mut in_entry = false;
    let mut in_decision = false;
    for line in decisions.lines() {
        if line.starts_with("## D") {
            in_entry = line.contains("Buy before build");
            in_decision = false;
            continue;
        }
        if !in_entry {
            continue;
        }
        if line.starts_with("- **Decision:**") {
            in_decision = true;
        } else if line.starts_with("- **") {
            in_decision = false;
        }
        if in_decision {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// External crates with no exact backticked name in the adopted record.
fn unrecorded(cargo_toml: &str, decisions: &str) -> Vec<String> {
    let record = adopted_record(decisions);
    external_crates(cargo_toml)
        .into_iter()
        .filter(|name| !record.contains(&format!("`{name}`")))
        .collect()
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
        .expect("docs/spec.toml has OLY-B-STD-03")
}

fn d36_section(decisions: &str) -> String {
    let start = decisions.find("## D36 · ").expect("the log has D36");
    let rest = &decisions[start..];
    let end = rest[3..].find("\n## D").map_or(rest.len(), |i| i + 3);
    rest[..end].to_string()
}

#[test]
fn every_external_workspace_crate_is_recorded_in_the_buy_first_decisions() {
    let missing = unrecorded(&repo_file("Cargo.toml"), &repo_file("docs/decisions.md"));
    assert!(
        missing.is_empty(),
        "external crates with no Buy before build decision: {missing:?}"
    );
}

#[test]
fn a_stray_dependency_is_refused() {
    let cargo = repo_file("Cargo.toml");
    let stray = cargo.replacen(
        "[workspace.dependencies]\n",
        "[workspace.dependencies]\nleft-pad-rs = \"1\"\n",
        1,
    );
    assert_ne!(cargo, stray);
    let decisions = repo_file("docs/decisions.md");
    assert!(unrecorded(&cargo, &decisions).is_empty());
    assert_eq!(unrecorded(&stray, &decisions), vec!["left-pad-rs"]);
}

#[test]
fn a_crate_named_only_as_a_rejected_alternative_is_refused() {
    let cargo = repo_file("Cargo.toml");
    let decisions = repo_file("docs/decisions.md");
    assert!(
        decisions.contains("`proptest`"),
        "D36 lists proptest as turned down"
    );
    let stray = cargo.replacen(
        "[workspace.dependencies]\n",
        "[workspace.dependencies]\nproptest = \"1\"\n",
        1,
    );
    assert_eq!(unrecorded(&stray, &decisions), vec!["proptest"]);
}

#[test]
fn d36_alone_leaves_proc_macro2_syn_and_toml_edit_unrecorded() {
    let manifest = "[workspace.dependencies]\n\
        athena-data = { path = \"crates/athena-data\" }\n\
        serde = \"1\"\ntoml = \"0.9\"\ntoml_edit = \"0.23\"\n\
        syn = { version = \"2\", features = [\"full\"] }\n\
        proc-macro2 = \"1\"\n";
    let d36 = d36_section(&repo_file("docs/decisions.md"));
    assert_eq!(
        unrecorded(manifest, &d36),
        vec!["toml_edit", "syn", "proc-macro2"]
    );
}

#[test]
fn a_buy_first_heading_without_a_decision_bullet_records_nothing() {
    let manifest = "[workspace.dependencies]\nfoo = \"1\"\n";
    let log =
        "## D1 · Buy before build: foo\n\n- **Alternatives:**\n  - `foo` instead of nothing.\n";
    assert_eq!(unrecorded(manifest, log), vec!["foo"]);
    let with = "## D1 · Buy before build: foo\n\n- **Decision:** adopt `foo`.\n- **Alternatives:** none.\n";
    assert!(unrecorded(manifest, with).is_empty());
    let other = "## D1 · Something else\n\n- **Decision:** adopt `foo`.\n";
    assert_eq!(unrecorded(manifest, other), vec!["foo"]);
}

#[test]
fn path_crates_are_not_external() {
    let inline = "[workspace.dependencies]\na = { path = \"crates/a\" }\nb = \"1\"\n";
    assert_eq!(external_crates(inline), vec!["b"]);
    let table = "[workspace.dependencies.a]\npath = \"crates/a\"\n[workspace.dependencies.c]\nversion = \"2\"\n";
    assert_eq!(external_crates(table), vec!["c"]);
}

#[test]
fn spec_entry_has_a_real_mechanism() {
    let text = repo_file("docs/spec.toml");
    let requirements = parse_spec(&text).expect("docs/spec.toml parses");
    let r = real_requirement();
    assert_ne!(r.mechanism, "none");
    assert!(r.mechanism.contains("buy_first"), "{}", r.mechanism);
    assert!(r.mechanism.contains("D36"), "{}", r.mechanism);
    assert_eq!(owner_problem(&r), None);
    assert_eq!(measure_problem(&r.measure), None, "{}", r.measure);
    assert_eq!(measure_label_problem(&r), None, "{}", r.measure);
    assert!(
        !spec_gaps(&requirements).iter().any(|g| g.id == REQUIREMENT),
        "OLY-B-STD-03 is still a spec gap"
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
    // The production check accepts this label; the end-label check is what refuses it.
    assert_eq!(measure_problem(&r.measure), None);
    assert!(measure_label_problem(&r).is_some());
}

#[test]
fn spec_check_refuses_an_invalid_label() {
    let mut r = real_requirement();
    r.measure = format!("{TEST_COMMAND}; status: bogus");
    assert!(measure_problem(&r.measure).is_some());
    assert!(measure_label_problem(&r).is_some());
}

#[test]
fn spec_check_refuses_a_wrong_owner() {
    let mut r = real_requirement();
    r.owner = "Hestia".into();
    assert!(owner_problem(&r).is_some());
}

#[test]
fn spec_check_refuses_a_measure_without_the_test_command() {
    let mut r = real_requirement();
    r.measure = "The dependency list; status: not shown".into();
    assert_eq!(measure_problem(&r.measure), None);
    assert!(measure_label_problem(&r).is_some());
}
