//! `athena --version` names the package version and the source commit.

use athena_cli::wiring::version_string;
use athena_data::records::Build;
use std::process::Command;

fn build(commit: &str, dirty: bool) -> Build {
    Build {
        version: "1.2.3".into(),
        source_commit: commit.into(),
        dirty,
    }
}

#[test]
fn clean_build_names_its_commit() {
    assert_eq!(
        version_string(&build("abc123", false)),
        "1.2.3 (commit abc123)"
    );
}

#[test]
fn dirty_build_is_marked() {
    assert_eq!(
        version_string(&build("abc123", true)),
        "1.2.3 (commit abc123, dirty)"
    );
}

#[test]
fn unknown_commit_is_marked_dirty() {
    assert_eq!(
        version_string(&build("unknown", true)),
        "1.2.3 (commit unknown, dirty)"
    );
}

#[test]
fn binary_prints_the_build_info() {
    let out = Command::new(env!("CARGO_BIN_EXE_athena"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    let expected = athena_cli::wiring::build_info();
    assert_eq!(text.trim(), format!("athena {}", version_string(&expected)));
}
