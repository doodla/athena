//! OLY-B-S1-02: reusable study work lives in the studying capability, instance-specific work stays with its
//! instance.
//!
//! The mechanism is this test, which checks the split on the real files:
//! - every `studies/<id>/` directory holds no code file (at any depth) and has a `pin.json`;
//! - nothing under `crates/athena-cap-study` names an instance, by the path `studies/<id>` or by the quoted
//!   literal `"<id>"`;
//! - the spec entry has a real mechanism, an owner and a measure ending `status: not shown`.
//!
//! It complements the checks inside the capability, and shares with them the rule that instance work is data
//! under one `studies/<id>/` directory with a pin: `StudyCapability::admit`
//! (`crates/athena-cap-study/src/lib.rs`) refuses a study change outside one such directory, and the test
//! `every_study_follows_the_conventions_and_the_matching_one_validates` (`crates/athena-cap-study/src/rules.rs`)
//! checks every instance in full, including the pin content. `athena-core` cannot depend on the capability
//! crate, so those are cited and not called. The capability accepts nested paths and any file type, so this
//! test forbids only code files.
//!
//! Limits: a fresh context for each later instance and the fix-and-re-check of gaps in the capability are not
//! visible in static files, so the measure ends `status: not shown`. Only one instance is studied so far; the
//! checks iterate over every `studies/*/` so later instances are covered. The capability's own tests use the
//! fixture ids `a`, `b` and `s1`; a real instance with one of those ids would need those fixtures renamed.

use athena_core::hestia::{SpecRequirement, measure_problem, parse_spec, spec_gaps};
use std::path::{Path, PathBuf};

const REQUIREMENT: &str = "OLY-B-S1-02";
const TEST_COMMAND: &str = "cargo test -p athena-core --test study_instance_split";
const CODE_EXTENSIONS: &[&str] = &[
    "rs", "sh", "bash", "py", "js", "ts", "rb", "go", "c", "h", "cpp", "java", "pl",
];
const CODE_NAMES: &[&str] = &["Cargo.toml", "build.rs"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn repo_file(relative: &str) -> String {
    let path = repo_root().join(relative);
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
        .expect("docs/spec.toml has OLY-B-S1-02")
}

/// Problems with one instance. `files` are paths relative to `studies/<id>/`, with `/` separators.
fn instance_problems(id: &str, files: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for f in files {
        let name = f.rsplit('/').next().unwrap_or(f);
        let ext = name.rsplit_once('.').map(|(_, e)| e);
        let is_code =
            CODE_NAMES.contains(&name) || ext.is_some_and(|e| CODE_EXTENSIONS.contains(&e));
        if is_code {
            out.push(format!(
                "studies/{id}/{f} is code; instance work is data, and code belongs in the capability"
            ));
        }
    }
    // The minimum shared with the capability's loader; the pin content is checked in rules.rs.
    if !files.iter().any(|f| f == "pin.json") {
        out.push(format!("studies/{id}/pin.json is missing"));
    }
    out
}

fn is_id_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_'
}

/// Problems with capability sources: a source that names an instance by `studies/<id>` or `"<id>"`.
fn capability_problems(sources: &[(String, String)], ids: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for (file, text) in sources {
        for id in ids {
            let path = format!("studies/{id}");
            let by_path = text.match_indices(&path).any(|(i, _)| {
                !text[i + path.len()..]
                    .chars()
                    .next()
                    .is_some_and(is_id_char)
            });
            let by_literal = text.contains(&format!("\"{id}\""));
            if by_path || by_literal {
                out.push(format!(
                    "{file} names the instance `{id}`; reusable study logic must not"
                ));
            }
        }
    }
    out
}

fn walk(dir: &Path, base: &Path, out: &mut Vec<String>) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|e| e.expect("directory entry").path())
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            walk(&p, base, out);
        } else {
            let rel = p.strip_prefix(base).expect("under base");
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

fn instance_ids() -> Vec<String> {
    let mut ids: Vec<String> = std::fs::read_dir(repo_root().join("studies"))
        .expect("studies/ exists")
        .map(|e| e.expect("directory entry").path())
        .filter(|p| p.is_dir())
        .map(|p| p.file_name().expect("name").to_string_lossy().into_owned())
        .collect();
    ids.sort();
    ids
}

fn instance_files(id: &str) -> Vec<String> {
    let base = repo_root().join("studies").join(id);
    let mut files = Vec::new();
    walk(&base, &base, &mut files);
    files
}

fn capability_sources() -> Vec<(String, String)> {
    let base = repo_root().join("crates/athena-cap-study");
    let mut files = Vec::new();
    walk(&base, &base, &mut files);
    files
        .into_iter()
        .filter(|f| f.starts_with("src/") || f.starts_with("prompts/"))
        .map(|f| {
            let text = std::fs::read_to_string(base.join(&f))
                .unwrap_or_else(|e| panic!("cannot read {f}: {e}"));
            (format!("crates/athena-cap-study/{f}"), text)
        })
        .collect()
}

fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_string()).collect()
}

#[test]
fn every_study_instance_holds_no_code_and_a_pin() {
    let ids = instance_ids();
    assert!(!ids.is_empty(), "no studies/<id>/ directory found");
    for id in &ids {
        let problems = instance_problems(id, &instance_files(id));
        assert!(problems.is_empty(), "{problems:?}");
    }
}

#[test]
fn the_study_capability_names_no_instance() {
    let sources = capability_sources();
    assert!(
        sources.iter().any(|(f, _)| f.ends_with("src/lib.rs")),
        "capability sources were not found"
    );
    let mut ids = instance_ids();
    ids.extend(strings(&["athena", "hermes"]));
    let problems = capability_problems(&sources, &ids);
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn an_instance_with_code_is_reported() {
    let clean = strings(&["pin.json", "claims.json", "notes.md"]);
    assert!(instance_problems("x", &clean).is_empty());
    let mut with_rs = clean.clone();
    with_rs.push("helper.rs".into());
    let p = instance_problems("x", &with_rs);
    assert_eq!(p.len(), 1);
    assert!(p[0].contains("studies/x/helper.rs"), "{p:?}");
    let mut nested = clean.clone();
    nested.push("sub/run.sh".into());
    let p = instance_problems("x", &nested);
    assert_eq!(p.len(), 1);
    assert!(p[0].contains("sub/run.sh"), "{p:?}");
    let mut manifest = clean;
    manifest.push("Cargo.toml".into());
    assert_eq!(instance_problems("x", &manifest).len(), 1);
}

#[test]
fn nested_data_files_are_allowed() {
    let files = strings(&["pin.json", "sub/dir/x.json", "notes.md", "sub/readme.txt"]);
    assert!(instance_problems("x", &files).is_empty());
}

#[test]
fn an_instance_without_a_pin_is_reported() {
    assert!(instance_problems("x", &strings(&["pin.json"])).is_empty());
    let p = instance_problems("x", &strings(&["claims.json", "sub/pin.json"]));
    assert_eq!(p.len(), 1);
    assert!(p[0].contains("pin.json is missing"), "{p:?}");
}

fn source(text: &str) -> Vec<(String, String)> {
    vec![("cap/src/lib.rs".to_string(), text.to_string())]
}

#[test]
fn a_capability_naming_an_instance_is_reported() {
    let ids = strings(&["firstmate-work-handling"]);
    assert!(capability_problems(&source("fn admit() {}"), &ids).is_empty());
    let by_path = capability_problems(
        &source("read(\"studies/firstmate-work-handling/x.json\")"),
        &ids,
    );
    assert_eq!(by_path.len(), 1);
    assert!(
        by_path[0].contains("firstmate-work-handling"),
        "{by_path:?}"
    );
    let by_literal = capability_problems(&source("let id = \"firstmate-work-handling\";"), &ids);
    assert_eq!(by_literal.len(), 1);
}

#[test]
fn short_ids_do_not_match_ordinary_text() {
    let ids = strings(&["athena"]);
    let text = "use athena_core::x; // athena-cap-study, Athena studies/athena-extra";
    assert!(capability_problems(&source(text), &ids).is_empty());
    assert_eq!(
        capability_problems(&source("studies/athena/x.json"), &ids).len(),
        1
    );
    assert_eq!(
        capability_problems(&source("studies/athena"), &ids).len(),
        1
    );
    assert_eq!(
        capability_problems(&source("let s = \"athena\";"), &ids).len(),
        1
    );
}

#[test]
fn spec_entry_has_a_real_mechanism() {
    let requirements = parse_spec(&repo_file("docs/spec.toml")).expect("docs/spec.toml parses");
    let r = real_requirement();
    assert_ne!(r.mechanism, "none");
    for part in [
        "crates/athena-core/tests/study_instance_split.rs",
        "StudyCapability::admit",
        "every_study_follows_the_conventions",
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
    assert!(
        !spec_gaps(&requirements).iter().any(|g| g.id == REQUIREMENT),
        "OLY-B-S1-02 is still a spec gap"
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
