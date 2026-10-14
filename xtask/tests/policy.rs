//! Repository-policy tests: rules about the repository itself, enforced by test (design §8).
//!
//! - M19: nothing in Athena refers to who runs it or the particular machine it runs on;
//! - M20: no tracked file is code in another language (by extension or shebang), beyond the short
//!   allow-list described in `docs/non-rust-files.md`; data and documents are allowed anywhere;
//! - M21: the crate dependency graph matches `docs/architecture.md`;
//! - M24: every test named in `docs/enforcement.md` exists.

use regex::Regex;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn git(args: &[&str], stdin: Option<&str>) -> String {
    use std::io::Write;
    let mut child = Command::new("git")
        .arg("-C")
        .arg(root())
        .args(["-c", "core.attributesFile=/dev/null"])
        .args(args)
        .env("GIT_ATTR_NOSYSTEM", "1")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    // `xtask` may not depend on `athena-git` (architecture policy), so this mirrors
    // `athena_git::output_with_stdin`: write stdin on its own thread while stdout is drained, or both
    // sides block once the output passes the pipe buffer.
    let mut pipe = child.stdin.take().unwrap();
    let input = stdin.unwrap_or("").as_bytes();
    let out = std::thread::scope(|scope| {
        let writer = scope.spawn(move || {
            // A broken pipe means git stopped reading; its exit status is checked below.
            let _ = pipe.write_all(input);
        });
        let out = child.wait_with_output().unwrap();
        writer.join().unwrap();
        out
    });
    assert!(out.status.success(), "git {args:?} failed");
    String::from_utf8(out.stdout).unwrap()
}

/// Tracked files, minus the paths the owner keeps to themselves (`athena-exclude`) or maintains
/// (`athena-readonly`), by the committed rules at `HEAD`.
fn athena_files() -> Vec<String> {
    let files: Vec<String> = git(&["ls-files", "-z"], None)
        .split('\0')
        .filter(|f| !f.is_empty())
        .map(String::from)
        .collect();
    let input: String = files.iter().map(|f| format!("{f}\0")).collect();
    let attrs = git(
        &[
            "check-attr",
            "--source=HEAD",
            "-z",
            "--stdin",
            "athena-exclude",
            "athena-readonly",
        ],
        Some(&input),
    );
    let fields: Vec<&str> = attrs.split('\0').collect();
    let mut owner = BTreeSet::new();
    for triple in fields.chunks(3) {
        if let [path, _attr, value] = triple
            && !matches!(*value, "unspecified" | "unset" | "")
        {
            owner.insert(path.to_string());
        }
    }
    files.into_iter().filter(|f| !owner.contains(f)).collect()
}

// --------------------------------------------------------------------------------------------------------
// M19: the boundary scan

/// Words that would name who runs Athena or the machine it runs on, built from fragments so this file
/// does not contain them. Matched as whole words, case-insensitively.
fn boundary_words() -> Vec<String> {
    [
        ["oper", "ator"],
        ["oper", "ators"],
        ["op", "ctl"],
        ["c", "cr"],
        ["sand", "box"],
        ["sand", "boxes"],
        ["sand", "boxed"],
        ["lea", "se"],
        ["lea", "ses"],
        ["hand", "off"],
        ["hand", "offs"],
        ["hand-", "off"],
        ["hand-", "offs"],
    ]
    .iter()
    .map(|parts| parts.concat())
    .chain(
        [
            ["cloud", " session"],
            ["cloud", " sessions"],
            ["remote", " session"],
            ["remote", " sessions"],
        ]
        .iter()
        .map(|p| p.concat()),
    )
    .collect()
}

/// Fragments of this machine's own paths and identifiers, matched anywhere, case-insensitively.
fn boundary_fragments() -> Vec<String> {
    [
        ["/home", "/user"],
        ["claude", "-0/"],
        ["/root", "/."],
        [".c", "cr/"],
        ["claude.ai", "/code"],
        ["session", "_01"],
    ]
    .iter()
    .map(|parts| parts.concat())
    .collect()
}

/// The boundary violations in one text: (line number, term).
fn boundary_scan(text: &str) -> Vec<(usize, String)> {
    let words: Vec<(String, Regex)> = boundary_words()
        .into_iter()
        .map(|w| {
            let re = Regex::new(&format!(r"(?i)\b{}\b", regex::escape(&w))).unwrap();
            (w, re)
        })
        .collect();
    let fragments: Vec<String> = boundary_fragments()
        .into_iter()
        .map(|f| f.to_lowercase())
        .collect();
    let mut found = Vec::new();
    for (i, line) in text.lines().enumerate() {
        for (word, re) in &words {
            if re.is_match(line) {
                found.push((i + 1, word.clone()));
            }
        }
        let lower = line.to_lowercase();
        for fragment in &fragments {
            if lower.contains(fragment.as_str()) {
                found.push((i + 1, fragment.clone()));
            }
        }
    }
    found
}

#[test]
fn athena_refers_to_no_one_who_runs_it_or_where() {
    // M19 (allows): Athena's tree passes. Records hold verbatim inputs, and owner paths are the owner's.
    let mut violations = Vec::new();
    for path in athena_files() {
        if path.starts_with("records/") || path == "LICENSE" {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(root().join(&path)) else {
            continue;
        };
        for (line, term) in boundary_scan(&text) {
            violations.push(format!("{path}:{line}: {term}"));
        }
    }
    assert!(
        violations.is_empty(),
        "Athena must not refer to who runs it or where:\n{}",
        violations.join("\n")
    );
}

#[test]
fn the_boundary_scanner_flags_a_planted_term() {
    // M19 (blocks): the scanner flags a planted term.
    let planted = format!(
        "Ask the {} first.\nIt ran in a {} on {}.",
        ["oper", "ator"].concat(),
        ["sand", "box"].concat(),
        ["/home", "/user/x"].concat()
    );
    let found = boundary_scan(&planted);
    assert_eq!(found.len(), 3, "{found:?}");
    assert!(
        boundary_scan("Cooperators accrue the opcode.").is_empty(),
        "whole words only"
    );
    // All four subjects the bootstrap brief rules out, not only the first two (acceptance
    // verification 1, m3).
    let planted = format!(
        "Renew the {} at each {}.\nTwo {} and one {}.",
        ["lea", "se"].concat(),
        ["hand", "off"].concat(),
        ["lea", "ses"].concat(),
        ["hand-", "off"].concat()
    );
    assert_eq!(boundary_scan(&planted).len(), 4, "{planted}");
    assert!(
        boundary_scan("The release hands off to cargo.").is_empty(),
        "whole words only"
    );
}

#[test]
fn the_producer_brief_lists_exactly_the_scanners_terms() {
    // M19 (blocks): the producer brief's banned list cannot drift from the scanner's. The dependency is
    // dev-only, so the documented graph (M21) is unchanged.
    assert_eq!(
        athena_context::briefs::boundary_words(),
        boundary_words(),
        "the brief's words differ from the scanner's"
    );
    assert_eq!(
        athena_context::briefs::boundary_fragments(),
        boundary_fragments(),
        "the brief's fragments differ from the scanner's"
    );
}

// --------------------------------------------------------------------------------------------------------
// M20: no code in another language

/// Extensions of code in a language other than Rust. Data and documents (json, toml, yaml, md, txt) are
/// not code and are not listed.
const OTHER_LANGUAGE_EXTENSIONS: &[&str] = &[
    "py", "js", "mjs", "cjs", "jsx", "ts", "tsx", "sh", "bash", "zsh", "rb", "go", "pl", "php",
    "java", "c", "cpp", "h", "lua", "ps1",
];

/// The non-Rust code that must exist, each with its reason. Mirrored in `docs/non-rust-files.md`.
const ALLOWED_OTHER_LANGUAGE_CODE: &[(&str, &str)] = &[
    (
        "setup.sh",
        "runs before any Rust toolchain exists: it installs a pinned rustup and passes control to Rust",
    ),
    (
        ".github/workflows/ci.yml",
        "GitHub Actions reads only YAML workflows; it is a thin call into Rust",
    ),
    (
        ".github/workflows/pages.yml",
        "GitHub Actions reads only YAML workflows; it renders the explainer page with a Rust xtask command and deploys it",
    ),
];

/// Whether a tracked file is code in a language other than Rust: by extension, or by a `#!` first line
/// (`#![` opens a Rust inner attribute, not a shebang). `head` is the start of the file's content.
fn is_other_language_code(path: &str, head: &[u8]) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let by_extension = name
        .rsplit_once('.')
        .is_some_and(|(_, ext)| OTHER_LANGUAGE_EXTENSIONS.contains(&ext.to_lowercase().as_str()));
    by_extension || (head.starts_with(b"#!") && !head.starts_with(b"#!["))
}

/// Whether the file is flagged: other-language code that is not on the allow-list.
fn flagged(path: &str, head: &[u8]) -> bool {
    is_other_language_code(path, head)
        && !ALLOWED_OTHER_LANGUAGE_CODE
            .iter()
            .any(|(allowed, _)| *allowed == path)
}

#[test]
fn every_tracked_file_is_rust_or_data_or_an_allowed_exception() {
    // M20 (allows): the tree holds no code in another language beyond the allow-list.
    let stray: Vec<String> = athena_files()
        .into_iter()
        .filter(|p| {
            let head = std::fs::read(root().join(p)).unwrap_or_default();
            flagged(p, &head)
        })
        .collect();
    assert!(
        stray.is_empty(),
        "code in a language other than Rust (rewrite it in Rust, or allow-list it with its reason in \
         xtask/tests/policy.rs and docs/non-rust-files.md): {stray:?}"
    );
    for (path, reason) in ALLOWED_OTHER_LANGUAGE_CODE {
        assert!(root().join(path).exists(), "allow-listed {path} is missing");
        assert!(!reason.is_empty(), "{path} needs a reason");
    }
}

#[test]
fn a_stray_script_is_flagged() {
    // M20 (blocks): code in another language, by extension or by shebang.
    assert!(flagged("tools/build.py", b"print('x')\n"));
    assert!(flagged("scripts/run.sh", b"echo hi\n"));
    assert!(flagged("web/app.ts", b""));
    assert!(flagged("bin/tool", b"#!/bin/sh\necho hi\n"));
    assert!(
        flagged("sub/setup.sh", b"#!/bin/sh\n"),
        "only the root setup.sh is allowed"
    );
    // Data and documents pass anywhere, with no ledger entry.
    assert!(!flagged("crates/x/tests/fixtures/case.json", b"{}\n"));
    assert!(!flagged("docs/notes.md", b"# Notes\n"));
    assert!(!flagged("config/x.toml", b"[a]\n"));
    assert!(!flagged("data/x.yaml", b"a: 1\n"));
    assert!(!flagged("data/x.txt", b"text\n"));
    assert!(!flagged(
        "crates/x/src/lib.rs",
        b"#![forbid(unsafe_code)]\n"
    ));
    // The named exceptions are allowed.
    assert!(!flagged("setup.sh", b"#!/bin/sh\n"));
    assert!(!flagged(".github/workflows/ci.yml", b"name: ci\n"));
    assert!(!flagged(".github/workflows/pages.yml", b"name: pages\n"));
}

// --------------------------------------------------------------------------------------------------------
// M21: dependency direction

fn short(name: &str) -> String {
    name.strip_prefix("athena-").unwrap_or(name).to_string()
}

/// The documented graph: crate → the Athena crates it depends on, from `docs/architecture.md`.
fn documented() -> BTreeMap<String, BTreeSet<String>> {
    let text = std::fs::read_to_string(root().join("docs/architecture.md")).unwrap();
    let mut graph = BTreeMap::new();
    for line in text.lines().filter(|l| l.starts_with("| `")) {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        let name = cells[1].trim_matches('`').to_string();
        let deps: BTreeSet<String> = match cells[3] {
            "none" => BTreeSet::new(),
            list => list.split(',').map(|d| d.trim().to_string()).collect(),
        };
        graph.insert(short(&name), deps);
    }
    graph
}

/// The actual graph, from `cargo metadata`.
fn actual() -> BTreeMap<String, BTreeSet<String>> {
    let out = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .current_dir(root())
        .output()
        .unwrap();
    assert!(out.status.success());
    let meta: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let members: BTreeSet<String> = meta["packages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap().to_string())
        .collect();
    let mut graph = BTreeMap::new();
    for package in meta["packages"].as_array().unwrap() {
        let deps: BTreeSet<String> = package["dependencies"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d["kind"].is_null())
            .map(|d| d["name"].as_str().unwrap().to_string())
            .filter(|d| members.contains(d))
            .map(|d| short(&d))
            .collect();
        graph.insert(short(package["name"].as_str().unwrap()), deps);
    }
    graph
}

/// Differences between the graphs, plus the rules beyond the table.
fn graph_problems(
    documented: &BTreeMap<String, BTreeSet<String>>,
    actual: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<String> {
    let mut problems = Vec::new();
    for (name, deps) in actual {
        match documented.get(name) {
            None => problems.push(format!("{name} is not in docs/architecture.md")),
            Some(doc) => {
                for dep in deps.difference(doc) {
                    problems.push(format!("{name} → {dep} is not documented"));
                }
                for dep in doc.difference(deps) {
                    problems.push(format!("{name} → {dep} is documented but absent"));
                }
            }
        }
    }
    if actual
        .get("core")
        .is_some_and(|d| d.iter().any(|x| x.starts_with("cap-")))
    {
        problems.push("athena-core depends on a capability".into());
    }
    for (name, deps) in actual.iter().filter(|(n, _)| n.starts_with("cap-")) {
        if deps.iter().any(|d| d.starts_with("cap-")) {
            problems.push(format!("{name} depends on another capability"));
        }
    }
    for leaf in ["data", "git"] {
        if actual.get(leaf).is_some_and(|d| !d.is_empty()) {
            problems.push(format!("athena-{leaf} must be a leaf"));
        }
    }
    problems
}

#[test]
fn dependency_direction_matches_the_architecture() {
    // M21 (allows): the documented graph.
    let problems = graph_problems(&documented(), &actual());
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn an_undocumented_edge_is_flagged() {
    // M21 (blocks): an undocumented edge; a core-to-capability edge.
    let documented = documented();
    let mut actual = actual();
    actual.get_mut("data").unwrap().insert("git".into());
    actual.get_mut("core").unwrap().insert("cap-change".into());
    let problems = graph_problems(&documented, &actual);
    assert!(problems.iter().any(|p| p.contains("data → git")));
    assert!(
        problems
            .iter()
            .any(|p| p.contains("athena-core depends on a capability"))
    );
    assert!(
        problems
            .iter()
            .any(|p| p.contains("athena-data must be a leaf"))
    );
}

// --------------------------------------------------------------------------------------------------------
// M24: the enforcement map names real tests

fn named_tests(map: &str) -> BTreeSet<String> {
    let name = Regex::new(r"`([a-z][a-z0-9]*(?:_[a-z0-9]+)+)`").unwrap();
    map.lines()
        .filter(|l| l.starts_with("| M"))
        .flat_map(|l| {
            name.captures_iter(l)
                .map(|c| c[1].to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn defined_tests() -> BTreeSet<String> {
    let def = Regex::new(r"fn ([a-z][a-z0-9_]*)\s*\(").unwrap();
    let mut names = BTreeSet::new();
    for path in athena_files().into_iter().filter(|p| p.ends_with(".rs")) {
        let text = std::fs::read_to_string(root().join(&path)).unwrap_or_default();
        names.extend(def.captures_iter(&text).map(|c| c[1].to_string()));
    }
    names
}

#[test]
fn every_test_named_in_the_enforcement_map_exists() {
    // M24 (allows): the map.
    let map = std::fs::read_to_string(root().join("docs/enforcement.md")).unwrap();
    let named = named_tests(&map);
    assert!(named.len() > 50, "the map parses: {} names", named.len());
    let defined = defined_tests();
    let missing: Vec<&String> = named.iter().filter(|n| !defined.contains(*n)).collect();
    assert!(
        missing.is_empty(),
        "tests named in docs/enforcement.md that do not exist: {missing:?}"
    );
}

#[test]
fn a_missing_test_name_is_flagged() {
    // M24 (blocks): a missing test.
    let map = "| M99 | rule | mechanism | `a_test_that_was_never_written` | `every_test_named_in_the_enforcement_map_exists` |";
    let named = named_tests(map);
    let defined = defined_tests();
    assert!(named.iter().any(|n| !defined.contains(n)));
}
