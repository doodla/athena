//! OLY-B-O1-01: the v0 walking skeleton was the only work done outside Athena, recorded as such.
//!
//! The mechanism is the bootstrap range (`athena_gate::invariant::bootstrap_range`, rule M6). These tests check the
//! spec entry names it, and that the outcome record closes the range.

use athena_gate::invariant::{BootstrapRange, bootstrap_range};
use athena_git::Git;
use std::path::Path;

const GATE: &str = "athena_gate::invariant::bootstrap_range";

fn repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

#[derive(serde::Deserialize, Default)]
struct Req {
    #[serde(default)]
    id: String,
    #[serde(default)]
    mechanism: String,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    measure: String,
}

#[derive(serde::Deserialize)]
struct Doc {
    #[serde(default)]
    requirement: Vec<Req>,
}

/// Checks the OLY-B-O1-01 entry of a spec text.
fn check_entry(text: &str) -> Result<(), String> {
    let doc: Doc = toml::from_str(text).map_err(|e| format!("spec does not parse: {e}"))?;
    let r = doc
        .requirement
        .iter()
        .find(|r| r.id == "OLY-B-O1-01")
        .ok_or("OLY-B-O1-01 is missing")?;
    let mechanism = r.mechanism.trim();
    if mechanism.is_empty() || mechanism.eq_ignore_ascii_case("none") {
        return Err(format!("mechanism is blank or none: {mechanism:?}"));
    }
    if !mechanism.contains(GATE) {
        return Err(format!("mechanism does not name {GATE}: {mechanism:?}"));
    }
    if r.owner != "Athena" {
        return Err(format!("owner is not Athena: {:?}", r.owner));
    }
    let measure = r.measure.trim();
    if !measure.ends_with("status: not shown") {
        return Err(format!(
            "measure does not end `status: not shown`: {measure:?}"
        ));
    }
    if !measure.contains("bootstrap") {
        return Err("measure does not name the bootstrap test".to_string());
    }
    if measure.matches("status:").count() != 1 {
        return Err("measure must contain `status:` exactly once".to_string());
    }
    Ok(())
}

#[test]
fn spec_entry_names_the_bootstrap_range_gate() {
    check_entry(&repo_file("docs/spec.toml")).unwrap();
}

#[test]
fn base_entry_is_rejected() {
    let base = r#"
[[requirement]]
id = "OLY-B-O1-01"
mechanism = "none"
owner = "Athena"
measure = "The spec gap list; status: requirement"
"#;
    let err = check_entry(base).unwrap_err();
    assert!(err.contains("mechanism"), "{err}");
}

#[test]
fn bootstrap_range_is_open_then_closed_by_the_outcome_record() {
    let dir = tempfile::tempdir().unwrap();
    let git = Git::isolated(dir.path());
    git.run(["init", "-q", "-b", "main"]).unwrap();
    std::fs::write(dir.path().join("a.txt"), "a\n").unwrap();
    git.run(["add", "-A"]).unwrap();
    git.run([
        "commit",
        "-q",
        "-m",
        "bootstrap work\n\nAthena-Job: bootstrap\n",
    ])
    .unwrap();
    let first = git.run(["rev-parse", "HEAD"]).unwrap().trim().to_string();
    assert_eq!(bootstrap_range(&git, "HEAD").unwrap(), BootstrapRange::Open);

    let outcome = format!(
        "{{\n  \"schema\": \"athena.outcome/1\",\n  \"job\": \"bootstrap\",\n  \"state\": \"closed\",\n  \"reason\": \"accepted\",\n  \"at\": \"2026-10-10T18:00:00Z\",\n  \"totals\": null,\n  \"branch\": null,\n  \"base_commit\": null,\n  \"candidate_commit\": null,\n  \"after_blocked\": false,\n  \"through_commit\": \"{first}\"\n}}\n"
    );
    let path = dir.path().join("records/jobs/bootstrap/outcome.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, outcome).unwrap();
    git.run(["add", "-A"]).unwrap();
    git.run(["commit", "-q", "-m", "close the bootstrap"])
        .unwrap();
    assert_eq!(
        bootstrap_range(&git, "HEAD").unwrap(),
        BootstrapRange::ClosedThrough(first)
    );
}
