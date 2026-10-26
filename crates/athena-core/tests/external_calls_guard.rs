//! Guard: every call to GitHub goes through the one wrapper in the directory `crates/athena-core/src/gh/`. Code under
//! `crates/` that runs the `gh` program, or names an HTTP client, anywhere else fails this test.

use regex::Regex;
use std::path::{Path, PathBuf};

/// The only directory whose files may run `gh`: a path prefix.
const WRAPPER: &str = "crates/athena-core/src/gh/";
/// This file names the patterns it looks for, so it is not scanned.
const SELF: &str = "crates/athena-core/tests/external_calls_guard.rs";

/// What counts as a call outside the wrapper: running `gh`, or an HTTP client crate or program.
fn forbidden() -> Regex {
    Regex::new(
        r#"Command::new\(\s*"gh"\s*\)|\b(reqwest|ureq|hyper|isahc|attohttpc|surf|curl|minreq|http_req)\b"#,
    )
    .unwrap()
}

/// The `(path, line number, line)` of every match in `files` (path, text), except in the wrapper itself.
fn violations(files: &[(String, String)]) -> Vec<(String, usize, String)> {
    let pattern = forbidden();
    let mut found = Vec::new();
    for (path, text) in files {
        if path.starts_with(WRAPPER) || path == SELF {
            continue;
        }
        for (n, line) in text.lines().enumerate() {
            if pattern.is_match(line) {
                found.push((path.clone(), n + 1, line.trim().to_string()));
            }
        }
    }
    found
}

fn collect(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect(&path, root, out);
        } else if path.extension().is_some_and(|e| e == "rs")
            || path.file_name().is_some_and(|n| n == "Cargo.toml")
        {
            let relative = path.strip_prefix(root).unwrap();
            let text = std::fs::read_to_string(&path).unwrap();
            out.push((relative.to_string_lossy().replace('\\', "/"), text));
        }
    }
}

#[test]
fn only_the_wrapper_calls_gh_or_an_http_client() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap();
    let mut files = Vec::new();
    collect(&root.join("crates"), root, &mut files);
    assert!(
        files.iter().any(|(p, _)| p.starts_with(WRAPPER)),
        "no wrapper file was found; the scan is looking in the wrong place"
    );
    let found = violations(&files);
    assert!(
        found.is_empty(),
        "external calls outside the wrapper in {WRAPPER}: {found:#?}"
    );
}

#[test]
fn the_scan_fails_on_a_fixture_outside_the_wrapper() {
    let gh = format!("let out = std::process::Command::new({:?}).output();", "gh");
    let http = format!("use {}::blocking::Client;", "reqwest");
    let files = vec![
        ("crates/other/src/lib.rs".to_string(), gh.clone()),
        ("crates/other/src/http.rs".to_string(), http),
        ("crates/other/Cargo.toml".to_string(), "ureq = \"2\"".into()),
        ("crates/other/src/ok.rs".to_string(), "fn fine() {}".into()),
        // Any file under the wrapper directory is allowed.
        (format!("{WRAPPER}mod.rs"), gh.clone()),
        (format!("{WRAPPER}sub/part.rs"), gh),
    ];
    let found = violations(&files);
    let paths: Vec<&str> = found.iter().map(|(p, _, _)| p.as_str()).collect();
    assert_eq!(
        paths,
        [
            "crates/other/src/lib.rs",
            "crates/other/src/http.rs",
            "crates/other/Cargo.toml"
        ]
    );
}
