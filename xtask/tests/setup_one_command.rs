//! OLY-B-A4-04: a clean machine reaches a working Athena with one command. The mechanism is `setup.sh`
//! handing off to `xtask setup`. This test reads the committed files and checks the spec entry and the
//! properties it claims. It reads files; it does not run a fresh-machine setup.

use serde::Deserialize;
use std::path::{Path, PathBuf};

const ID: &str = "OLY-B-A4-04";

#[derive(Debug, Deserialize)]
struct Spec {
    #[serde(default)]
    requirement: Vec<Req>,
}

#[derive(Debug, Clone, Deserialize)]
struct Req {
    id: String,
    status: String,
    #[serde(default)]
    mechanism: Option<String>,
    #[serde(default)]
    owner: String,
    #[serde(default)]
    measure: String,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn entry() -> Req {
    let spec: Spec = toml::from_str(&read("docs/spec.toml")).unwrap();
    spec.requirement
        .into_iter()
        .find(|r| r.id == ID)
        .expect("the requirement is in the spec")
}

/// The same rule as the spec gap list: a blank mechanism or the literal `none` is a gap.
fn is_blank_or_none(m: &Option<String>) -> bool {
    match m {
        None => true,
        Some(s) => {
            let t = s.trim();
            t.is_empty() || t.eq_ignore_ascii_case("none")
        }
    }
}

/// Every path-like token ending in .sh, .rs or .toml in `text`.
fn file_tokens(text: &str) -> Vec<String> {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-')))
        .map(|t| t.trim_end_matches('.'))
        .filter(|t| t.ends_with(".sh") || t.ends_with(".rs") || t.ends_with(".toml"))
        .map(str::to_string)
        .collect()
}

fn setup_lines() -> Vec<String> {
    read("setup.sh").lines().map(str::to_string).collect()
}

#[test]
fn the_spec_entry_has_a_mechanism_owner_and_measure() {
    let r = entry();
    assert_eq!(r.status, "active");
    assert!(
        !is_blank_or_none(&r.mechanism),
        "mechanism is blank or none"
    );
    let m = r.mechanism.unwrap();
    for named in [
        "setup.sh",
        "xtask/src/main.rs",
        "xtask/tests/setup_one_command.rs",
    ] {
        assert!(m.contains(named), "mechanism does not name {named}: {m}");
    }
    for t in file_tokens(&m) {
        assert!(
            root().join(&t).is_file(),
            "mechanism names a missing file: {t}"
        );
    }
    assert_eq!(r.owner, "Athena");
    assert!(
        r.measure
            .contains("cargo test -p xtask --test setup_one_command"),
        "{}",
        r.measure
    );
    assert!(r.measure.ends_with("status: not shown"), "{}", r.measure);
}

#[test]
fn setup_sh_has_one_locked_exec_of_xtask_setup() {
    let lines = setup_lines();
    let execs: Vec<&String> = lines.iter().filter(|l| l.starts_with("exec ")).collect();
    assert_eq!(execs.len(), 1, "{execs:?}");
    let last = lines
        .iter()
        .rev()
        .find(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .unwrap();
    assert_eq!(last, execs[0], "the exec line is not the last command");
    assert!(last.contains("cargo run"), "{last}");
    assert!(last.contains("--locked"), "{last}");
    assert!(last.contains("-p xtask -- setup"), "{last}");
}

#[test]
fn setup_sh_installs_a_pinned_checksummed_rustup_only_when_cargo_is_missing() {
    let text = read("setup.sh");
    assert!(
        text.lines().any(|l| l.starts_with("RUSTUP_VERSION=")),
        "no pinned RUSTUP_VERSION"
    );
    assert!(text.contains("rustup/archive/$RUSTUP_VERSION/"));
    let sums = text
        .split_whitespace()
        .filter(|w| w.starts_with("sum="))
        .count();
    assert_eq!(sums, 4, "one checksum per platform");
    assert!(text.contains(r#"[ "$actual" = "$sum" ] ||"#));
    let lines = setup_lines();
    let guard = lines
        .iter()
        .position(|l| l.starts_with("if ! command -v cargo"))
        .expect("guard on cargo");
    let curl = lines
        .iter()
        .position(|l| l.contains("curl "))
        .expect("a download line");
    assert!(curl > guard, "the download runs before the cargo guard");
}

#[test]
fn the_toolchain_is_pinned_and_the_readme_documents_the_one_command() {
    let v: toml::Value = toml::from_str(&read("rust-toolchain.toml")).unwrap();
    let channel = v["toolchain"]["channel"].as_str().expect("channel");
    assert!(
        channel.chars().next().unwrap().is_ascii_digit(),
        "{channel}"
    );
    let readme = read("README.md");
    assert!(
        readme
            .lines()
            .any(|l| l.trim_start().starts_with("./setup.sh")),
        "README has no ./setup.sh command line"
    );
    let main = read("xtask/src/main.rs");
    assert!(main.contains("fn gitleaks_present"));
    assert!(main.contains("already installed"));
}

#[test]
fn reverting_the_mechanism_to_none_is_a_gap() {
    let real = entry();
    assert!(!is_blank_or_none(&real.mechanism));
    for blank in [Some("none".to_string()), Some(String::new()), None] {
        let mut copy = real.clone();
        copy.mechanism = blank;
        assert!(is_blank_or_none(&copy.mechanism));
    }
}
