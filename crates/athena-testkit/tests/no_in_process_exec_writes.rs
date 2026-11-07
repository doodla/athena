//! A scan that keeps in-process executable writes out of the workspace's test code.
//!
//! A test that makes a file executable from inside its own process holds a write handle for a moment, and a
//! fork in another test thread can copy that handle, so the first test's run of the file fails with `ETXTBSY`.
//! Stand-ins must be written with `athena_testkit::write_executable`, which does the write in a child.
//!
//! [`violations`] is one pure function over a path and its text. The tree scan feeds it every Rust file under
//! `crates/*/{src,tests}` and `xtask/{src,tests}`; the planted-violation tests feed it strings.
//!
//! What counts as test code: every file under a `tests/` directory; files named `tests.rs`, `*_tests.rs` or
//! `test_support.rs`; and, in other files under `src/`, the text from the first `#[cfg(test)]` or
//! `#[cfg(all(test` line to the end (the convention `xtask/tests/policy.rs` uses). Production code above that
//! line may set permissions freely.

use std::path::{Path, PathBuf};

/// Files that hold a flagged line on purpose, each with the reason. The guard test fails when an entry no
/// longer exists or no longer has a hit, so the list cannot go stale.
const EXEMPT: &[(&str, &str)] = &[
    (
        "crates/athena-testkit/tests/no_in_process_exec_writes.rs",
        "holds the planted violations as string literals",
    ),
    (
        "crates/athena-testkit/tests/stand_in_stress.rs",
        "its in-process variant writes the bad way on purpose, as the control",
    ),
    (
        "crates/athena-cli/src/slot_exec.rs",
        "its unit-test helper writes scripts with a chosen mode, some not executable, and never runs them",
    ),
];

/// A reason is reported for each flagged line.
type Hit = (usize, &'static str);

fn is_test_file(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    rel.starts_with("tests/")
        || rel.contains("/tests/")
        || name == "tests.rs"
        || name == "test_support.rs"
        || name.ends_with("_tests.rs")
}

/// The 0-based index of the first line that counts as test code, or `None` when none does.
fn test_region_start(rel: &str, text: &str) -> Option<usize> {
    if is_test_file(rel) {
        return Some(0);
    }
    if !(rel.starts_with("src/") || rel.contains("/src/")) {
        return None;
    }
    text.lines().position(|l| {
        let l = l.trim_start();
        l.starts_with("#[cfg(test)]") || l.starts_with("#[cfg(all(test")
    })
}

/// A `.mode(` call that takes an argument (it sets a mode); `.mode()` with none only reads one.
fn sets_mode(line: &str) -> bool {
    let mut rest = line;
    while let Some(i) = rest.find(".mode(") {
        rest = &rest[i + ".mode(".len()..];
        if !rest.starts_with(')') {
            return true;
        }
    }
    false
}

/// Lines of test code that make a file executable from inside the process, as (1-based line, reason).
fn violations(rel: &str, text: &str) -> Vec<Hit> {
    let Some(start) = test_region_start(rel, text) else {
        return Vec::new();
    };
    let mut hits = Vec::new();
    for (i, line) in text.lines().enumerate().skip(start) {
        if line.trim_start().starts_with("//") {
            continue;
        }
        let reason = if line.contains("set_executable(") {
            "set_executable wrapper"
        } else if line.contains("set_permissions(") {
            "set_permissions"
        } else if line.contains("from_mode(") {
            "Permissions::from_mode"
        } else if line.contains(".set_mode(") {
            "set_mode"
        } else if sets_mode(line) {
            "OpenOptions mode"
        } else if line.contains("Command::new(") && line.contains("chmod\"") {
            "chmod child"
        } else {
            continue;
        };
        hits.push((i + 1, reason));
    }
    hits
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every scanned file as (path relative to the repository root, text).
fn tree() -> Vec<(String, String)> {
    let root = root();
    let mut dirs = vec![root.join("xtask/src"), root.join("xtask/tests")];
    for entry in std::fs::read_dir(root.join("crates")).unwrap() {
        let krate = entry.unwrap().path();
        dirs.push(krate.join("src"));
        dirs.push(krate.join("tests"));
    }
    let mut files = Vec::new();
    for dir in dirs {
        rust_files(&dir, &mut files);
    }
    files.sort();
    files
        .into_iter()
        .map(|f| {
            let rel = f
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let text = std::fs::read_to_string(&f).unwrap();
            (rel, text)
        })
        .collect()
}

#[test]
fn no_in_process_exec_writes() {
    let files = tree();
    assert!(files.len() > 100, "only {} files scanned", files.len());
    for must in ["xtask/src/main.rs", "crates/athena-cli/tests/slot_exec.rs"] {
        assert!(files.iter().any(|(r, _)| r == must), "{must} not scanned");
    }
    let mut found = Vec::new();
    for (rel, text) in &files {
        if EXEMPT.iter().any(|(p, _)| p == rel) {
            continue;
        }
        for (line, reason) in violations(rel, text) {
            found.push(format!("{rel}:{line}: {reason}"));
        }
    }
    assert!(
        found.is_empty(),
        "in-process executable writes in test code; use athena_testkit::write_executable:\n{}",
        found.join("\n")
    );
}

#[test]
fn every_exemption_still_exists_and_still_has_a_hit() {
    let files = tree();
    for (path, reason) in EXEMPT {
        let (_, text) = files
            .iter()
            .find(|(r, _)| r == path)
            .unwrap_or_else(|| panic!("exempt file {path} is gone; drop it from EXEMPT"));
        assert!(
            !violations(path, text).is_empty(),
            "{path} ({reason}) has no flagged line now; drop it from EXEMPT"
        );
    }
}

#[test]
fn a_set_executable_wrapper_call_is_flagged() {
    let text = "#[test]\nfn t() {\n    set_executable(&fake).unwrap();\n}\n";
    assert_eq!(
        violations("crates/x/tests/t.rs", text),
        vec![(3, "set_executable wrapper")]
    );
}

#[test]
fn set_permissions_with_0o755_is_flagged() {
    let text = "fn t() {\n    std::fs::set_permissions(&p, perms(0o755)).unwrap();\n}\n";
    let hits = violations("crates/x/tests/t.rs", text);
    assert_eq!(hits, vec![(2, "set_permissions")]);
}

#[test]
fn from_mode_is_flagged() {
    let text = "let p = std::fs::Permissions::from_mode(0o755);\n";
    assert_eq!(
        violations("crates/x/tests/t.rs", text),
        vec![(1, "Permissions::from_mode")]
    );
}

#[test]
fn set_mode_and_open_options_mode_are_flagged() {
    let text = "perms.set_mode(0o755);\nOpenOptions::new().mode(0o755).open(p);\n";
    assert_eq!(
        violations("crates/x/tests/t.rs", text),
        vec![(1, "set_mode"), (2, "OpenOptions mode")]
    );
}

#[test]
fn a_chmod_child_is_flagged() {
    let text = "Command::new(\"chmod\").arg(\"755\").arg(&p).status().unwrap();\n";
    assert_eq!(
        violations("crates/x/tests/t.rs", text),
        vec![(1, "chmod child")]
    );
}

#[test]
fn a_violation_after_the_cfg_test_marker_in_src_is_flagged() {
    let text = "fn prod() {}\n#[cfg(test)]\nmod t {\n    fn f() { set_executable(&p); }\n}\n";
    assert_eq!(
        violations("crates/x/src/lib.rs", text),
        vec![(4, "set_executable wrapper")]
    );
    let all = "#[cfg(all(test, unix))]\nmod t { fn f() { std::fs::set_permissions(a, b); } }\n";
    assert_eq!(violations("crates/x/src/lib.rs", all).len(), 1);
}

#[test]
fn a_test_support_file_is_scanned_whole() {
    let text = "fn f() { std::fs::set_permissions(a, b); }\n";
    assert_eq!(violations("crates/x/src/test_support.rs", text).len(), 1);
    assert_eq!(violations("crates/x/src/foo_tests.rs", text).len(), 1);
}

#[test]
fn production_code_comments_reads_and_the_helper_are_not_flagged() {
    let production = "fn set_executable(p: &Path) {\n    std::fs::set_permissions(p, perms).unwrap();\n}\n#[cfg(test)]\nmod t {}\n";
    assert!(violations("crates/x/src/lib.rs", production).is_empty());
    let no_marker = "fn f() { std::fs::set_permissions(a, b); }\n";
    assert!(violations("crates/x/src/lib.rs", no_marker).is_empty());
    let clean = "// set_permissions(a, b) is what not to do\n\
        let m = std::fs::metadata(&p).unwrap().permissions().mode();\n\
        athena_testkit::write_executable(&p, \"#!/bin/sh\\n\");\n\
        Command::new(\"sh\").arg(\"-c\").status().unwrap();\n";
    assert!(violations("crates/x/tests/t.rs", clean).is_empty());
}
