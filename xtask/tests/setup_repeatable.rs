//! OLY-B-RUN-02: environment changes are repeatable. The mechanism is `setup.sh` followed by `xtask setup`;
//! this test reads the committed files and checks the properties the spec entry claims: a pinned and
//! checksummed rustup-init, an exact toolchain channel, one locked command, and skip-if-present guards.
//! It reads files; it does not run a fresh-machine setup.

use serde::Deserialize;
use std::path::{Path, PathBuf};

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
        .find(|r| r.id == "OLY-B-RUN-02")
        .expect("OLY-B-RUN-02 is in the spec")
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

fn is_exact_version(v: &str) -> bool {
    let parts: Vec<&str> = v.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

fn is_sha256(v: &str) -> bool {
    v.len() == 64 && v.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
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
fn the_spec_entry_names_the_setup_path_owner_and_test() {
    let r = entry();
    assert_eq!(r.status, "active");
    assert!(
        !is_blank_or_none(&r.mechanism),
        "mechanism is blank or none"
    );
    let m = r.mechanism.unwrap();
    assert!(m.contains("setup.sh"), "{m}");
    assert!(m.contains("xtask/src/main.rs"), "{m}");
    let tokens = file_tokens(&m);
    assert!(!tokens.is_empty());
    for t in tokens {
        assert!(
            root().join(&t).is_file(),
            "mechanism names a missing file: {t}"
        );
    }
    assert_eq!(r.owner, "Athena");
    assert!(r.measure.contains("setup_repeatable"), "{}", r.measure);
    assert!(
        r.measure
            .contains("oly_b_run_02_no_longer_appears_in_the_spec_gap_list"),
        "{}",
        r.measure
    );
    assert!(r.measure.ends_with("status: not shown"), "{}", r.measure);
}

#[test]
fn setup_sh_pins_rustup_and_checks_its_sha256() {
    let text = read("setup.sh");
    let version = setup_lines()
        .iter()
        .find_map(|l| l.strip_prefix("RUSTUP_VERSION=").map(str::to_string))
        .expect("setup.sh pins RUSTUP_VERSION");
    assert!(is_exact_version(&version), "RUSTUP_VERSION={version}");
    let sums: Vec<&str> = text
        .split_whitespace()
        .filter_map(|w| w.strip_prefix("sum="))
        .map(|w| w.trim_end_matches(';'))
        .collect();
    assert_eq!(sums.len(), 4, "one checksum per platform: {sums:?}");
    for s in sums {
        assert!(is_sha256(s), "not a sha256: {s}");
    }
    assert!(
        text.contains("rustup/archive/$RUSTUP_VERSION/"),
        "URL is not pinned"
    );
    assert!(
        text.contains(r#"[ "$actual" = "$sum" ] ||"#),
        "no checksum comparison"
    );
    assert!(
        text.contains("refusing to run it"),
        "no refusal on mismatch"
    );
    assert!(!text.contains("latest"));
    assert!(
        !text.lines().any(|l| l.trim_end().ends_with("| sh")),
        "pipe-to-shell install"
    );
}

#[test]
fn the_toolchain_channel_is_an_exact_version() {
    let v: toml::Value = toml::from_str(&read("rust-toolchain.toml")).unwrap();
    let channel = v["toolchain"]["channel"]
        .as_str()
        .expect("channel is a string");
    assert!(is_exact_version(channel), "channel {channel}");
}

#[test]
fn setup_sh_ends_in_one_locked_cargo_run() {
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
    assert!(last.contains("-p xtask"), "{last}");
    assert!(last.ends_with(r#"-- setup "$@""#), "{last}");
}

#[test]
fn the_rustup_install_runs_only_when_cargo_is_missing() {
    let lines = setup_lines();
    let guard = lines
        .iter()
        .position(|l| l.starts_with("if ! command -v cargo"))
        .expect("guard on cargo");
    for needle in ["curl ", "rustup-init\" -y"] {
        let at = lines
            .iter()
            .position(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("no line with {needle}"));
        assert!(at > guard, "{needle} runs before the cargo guard");
    }
}

#[test]
fn the_other_pins_are_exact() {
    let v: toml::Value = toml::from_str(&read("athena.toml")).unwrap();
    let g = &v["tools"]["gitleaks"];
    let version = g["version"].as_str().expect("gitleaks version");
    assert!(is_exact_version(version), "gitleaks {version}");
    let sums = g["sha256"].as_table().expect("gitleaks sha256 table");
    assert_eq!(sums.len(), 4);
    for (k, s) in sums {
        assert!(is_sha256(s.as_str().unwrap_or("")), "{k} is not a sha256");
    }
    let main = read("xtask/src/main.rs");
    assert!(main.contains("fn gitleaks_present"));
    assert!(main.contains("already installed"));
}

#[test]
fn blanking_the_mechanism_makes_the_entry_a_gap() {
    let real = entry();
    assert!(!is_blank_or_none(&real.mechanism));
    for blank in [Some("none".to_string()), Some(String::new()), None] {
        let mut copy = real.clone();
        copy.mechanism = blank;
        assert!(is_blank_or_none(&copy.mechanism));
    }
}
