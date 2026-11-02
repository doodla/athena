//! Guard: the lists that say what the athena binary is built from are checked against what it really compiles in.
//!
//! Root cause this test closes (D154): a hand-kept list (the `BUILD_PATHS` constant, the `athena-bin` cache key in
//! `.github/workflows/ci.yml`) was checked for its shape but never against the files the build includes, so a
//! prompt or `docs/system-facts.md` could be left out and only a late verdict would notice.
//!
//! The real input set is derived here from the source: the target of every production `include_str!` and
//! `include_bytes!` under `crates/` and `xtask/`, plus the manifests, `rust-toolchain.toml` and `setup.sh`. Two
//! declared lists must cover it: `athena_core::status::BUILD_PATHS` (the stale-build rule) and every `athena-bin`
//! `hashFiles` line of the real `ci.yml`.
//!
//! The ci.yml half overlaps `athena_bin_keys_cover_every_compiled_in_file` in `xtask/tests/policy.rs`, which scans
//! the same way and must agree on the known-target floor below. A change to the scan or the floor in one belongs
//! in the other. This test also covers `BUILD_PATHS`, which that one does not.
//!
//! The limit: an include after an early `#[cfg(test)]` line, or a path built with `concat!`, is missed by the
//! scan; `crates/**` and `xtask/**` still cover whatever lies under those directories.

use athena_core::status::{BUILD_PATHS, list_touches};
use regex::Regex;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The workspace root, found from this crate's manifest directory (`crates/athena-core`).
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the workspace root")
        .to_path_buf()
}

/// `a/b/../c` as `a/c`, for a path relative to the repository root.
fn normalize(path: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for c in path.components() {
        match c {
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::Normal(p) => parts.push(p.to_string_lossy().into_owned()),
            _ => {}
        }
    }
    parts.join("/")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
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

/// The target of every production `include_str!` and `include_bytes!` literal, relative to the repository root:
/// a file under a `src/` directory, not `tests.rs` or `*_tests.rs`, and the text before its first
/// `#[cfg(test)]` line. Test-only includes (the README, the changelog, design documents) never reach the binary.
fn production_includes() -> BTreeSet<String> {
    let include = Regex::new(r#"include_(?:str|bytes)!\(\s*"([^"]+)"\s*\)"#).unwrap();
    let root = root();
    let mut files = Vec::new();
    for top in ["crates", "xtask"] {
        rust_files(&root.join(top), &mut files);
    }
    let mut found = BTreeSet::new();
    for file in files {
        let rel = file.strip_prefix(&root).unwrap();
        let name = rel.file_name().unwrap().to_string_lossy();
        let production = rel.components().any(|c| c.as_os_str() == "src")
            && name != "tests.rs"
            && !name.ends_with("_tests.rs");
        if !production {
            continue;
        }
        let text = std::fs::read_to_string(&file).unwrap();
        let production_text = text.split("#[cfg(test)]").next().unwrap();
        for cap in include.captures_iter(production_text) {
            found.insert(normalize(&rel.parent().unwrap().join(&cap[1])));
        }
    }
    found
}

/// Every file the athena binary is built from: the include targets plus the manifests, the toolchain pin and
/// the install script.
fn compiled_in_files() -> BTreeSet<String> {
    let root = root();
    let mut files = production_includes();
    for fixed in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "setup.sh",
    ] {
        files.insert(fixed.to_string());
    }
    for entry in std::fs::read_dir(root.join("crates")).unwrap() {
        let manifest = entry.unwrap().path().join("Cargo.toml");
        if manifest.is_file() {
            files.insert(normalize(manifest.strip_prefix(&root).unwrap()));
        }
    }
    files.insert("xtask/Cargo.toml".to_string());
    files
}

/// Files the scan must find, or it checks nothing. The same floor as `xtask/tests/policy.rs`.
const KNOWN_TARGETS: [&str; 8] = [
    "crates/athena-context/prompts/producer.md",
    "crates/athena-cap-change/prompts/verifier.md",
    "crates/athena-cap-study/prompts/producer.md",
    "docs/system-facts.md",
    "athena.toml",
    "crates/athena-core/Cargo.toml",
    "xtask/Cargo.toml",
    "setup.sh",
];

/// Whether the `hashFiles` glob `glob` names `path`. Three forms are understood: an exact path, `dir/**` (every
/// file under `dir`) and `dir/**/*.ext`. Any other form is an Err naming it, so a new form fails loudly instead
/// of silently covering nothing.
fn glob_covers(glob: &str, path: &str) -> Result<bool, String> {
    if !glob.contains('*') {
        return Ok(glob == path);
    }
    if let Some(dir) = glob.strip_suffix("/**")
        && !dir.contains('*')
    {
        return Ok(path.starts_with(&format!("{dir}/")));
    }
    if let Some((dir, ext)) = glob.split_once("/**/*.")
        && !dir.contains('*')
        && !ext.contains('*')
    {
        return Ok(path.starts_with(&format!("{dir}/")) && path.ends_with(&format!(".{ext}")));
    }
    Err(format!(
        "the glob '{glob}' is a form this check cannot read"
    ))
}

/// The globs inside the `hashFiles(...)` of one `key:` line.
fn hash_files_globs(line: &str) -> Vec<String> {
    let args = Regex::new(r"hashFiles\(([^)]*)\)").unwrap();
    let glob = Regex::new(r"'([^']*)'").unwrap();
    args.captures_iter(line)
        .flat_map(|c| {
            glob.captures_iter(&c[1])
                .map(|g| g[1].to_string())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// What is wrong with the `athena-bin` cache keys in the workflow text `yml`: every key line (restore and save,
/// in every job) must cover every path in `required`. Err names each missing path and each unreadable glob; Ok
/// needs at least one key.
fn check_keys(yml: &str, required: &BTreeSet<String>) -> Result<(), String> {
    let keys: Vec<(usize, &str)> = yml
        .lines()
        .enumerate()
        .filter(|(_, l)| l.trim_start().starts_with("key: athena-bin-"))
        .collect();
    if keys.is_empty() {
        return Err("ci.yml has no athena-bin cache key".to_string());
    }
    let mut problems = Vec::new();
    for (n, line) in keys {
        let globs = hash_files_globs(line);
        if globs.is_empty() {
            problems.push(format!("line {}: the key hashes no files", n + 1));
        }
        for path in required {
            let mut covered = false;
            for glob in &globs {
                match glob_covers(glob, path) {
                    Ok(true) => covered = true,
                    Ok(false) => {}
                    Err(e) => {
                        let problem = format!("line {}: {e}", n + 1);
                        if !problems.contains(&problem) {
                            problems.push(problem);
                        }
                    }
                }
            }
            if !covered {
                problems.push(format!("line {}: the key does not cover {path}", n + 1));
            }
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

/// What is wrong with the path list `list` (the shape of `BUILD_PATHS`): Err names each path of `required` it
/// does not touch. It uses the production matcher, so the check and the stale-build rule agree.
fn check_build_paths(list: &[&str], required: &BTreeSet<String>) -> Result<(), String> {
    let missing: Vec<&String> = required
        .iter()
        .filter(|path| !list_touches(list, std::slice::from_ref(*path)))
        .collect();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!("the list does not touch {missing:?}"))
    }
}

fn ci_yml() -> String {
    std::fs::read_to_string(root().join(".github/workflows/ci.yml")).expect("read ci.yml")
}

/// The `athena-bin` key text of the workflow before job kq9v fixed it: only Rust sources and the root manifest.
fn with_pre_fix_globs(yml: &str) -> String {
    let args = Regex::new(r"hashFiles\([^)]*\)").unwrap();
    let out = args
        .replace_all(yml, "hashFiles('crates/**/*.rs', 'Cargo.toml')")
        .into_owned();
    assert_ne!(out, yml);
    out
}

#[test]
fn the_scan_finds_the_known_targets_and_each_exists() {
    let required = compiled_in_files();
    for path in KNOWN_TARGETS {
        assert!(
            required.contains(path),
            "the scan misses {path}: {required:?}"
        );
    }
    for path in &required {
        assert!(
            root().join(path).exists(),
            "{path} is named by an include or manifest but does not exist"
        );
    }
}

#[test]
fn declared_inputs_cover_the_real_compiled_in_inputs() {
    let required = compiled_in_files();
    assert_eq!(check_build_paths(&BUILD_PATHS, &required), Ok(()));
    assert_eq!(check_keys(&ci_yml(), &required), Ok(()));
}

#[test]
fn every_athena_bin_key_line_is_read() {
    // Six restore steps (one per job that sets up) and three save steps, the count `xtask/tests/policy.rs` holds.
    let keys = ci_yml()
        .lines()
        .filter(|l| l.trim_start().starts_with("key: athena-bin-"))
        .count();
    assert_eq!(keys, 9);
}

#[test]
fn a_key_missing_a_compiled_in_file_is_reported_by_name() {
    let required = compiled_in_files();
    let yml = ci_yml();
    // A: one key without docs/system-facts.md.
    let no_facts = yml.replacen(", 'docs/system-facts.md'", "", 1);
    assert_ne!(no_facts, yml);
    let err = check_keys(&no_facts, &required).unwrap_err();
    assert!(err.contains("docs/system-facts.md"), "{err}");
    assert_eq!(check_keys(&yml, &required), Ok(()));
}

#[test]
fn the_key_from_before_the_fix_is_reported_for_a_prompt_and_the_facts_file() {
    let required = compiled_in_files();
    let yml = ci_yml();
    // B: only Rust sources and Cargo.toml.
    let err = check_keys(&with_pre_fix_globs(&yml), &required).unwrap_err();
    for path in [
        "crates/athena-context/prompts/producer.md",
        "crates/athena-cap-change/prompts/verifier.md",
        "docs/system-facts.md",
        "athena.toml",
    ] {
        assert!(err.contains(path), "{path} not named: {err}");
    }
    assert!(!err.contains("cannot read"), "{err}");
    assert_eq!(check_keys(&yml, &required), Ok(()));
}

#[test]
fn a_glob_form_the_check_cannot_read_fails_and_is_named() {
    let required = compiled_in_files();
    let yml = ci_yml();
    // C: a pattern form the matcher does not support.
    let odd = yml.replacen("'crates/**'", "'crates/*/prompts/*.md'", 1);
    assert_ne!(odd, yml);
    let err = check_keys(&odd, &required).unwrap_err();
    assert!(err.contains("crates/*/prompts/*.md"), "{err}");
    assert_eq!(glob_covers("crates/**", "crates/a/b.md"), Ok(true));
    assert_eq!(glob_covers("crates/**/*.rs", "crates/a/b.md"), Ok(false));
    assert_eq!(glob_covers("crates/**/*.rs", "crates/a/b.rs"), Ok(true));
    assert_eq!(glob_covers("setup.sh", "setup.sh"), Ok(true));
    assert!(glob_covers("a*b", "ab").is_err());
}

#[test]
fn a_new_compiled_in_file_is_covered_only_where_a_glob_names_it() {
    let required = compiled_in_files();
    let yml = ci_yml();
    // D: under crates/ it is covered by `crates/**`; a new document outside is not.
    let mut inside = required.clone();
    inside.insert("crates/x/prompts/new.md".to_string());
    assert_eq!(check_keys(&yml, &inside), Ok(()));
    assert_eq!(check_build_paths(&BUILD_PATHS, &inside), Ok(()));
    let mut outside = required;
    outside.insert("docs/new-compiled-in.md".to_string());
    let err = check_keys(&yml, &outside).unwrap_err();
    assert!(err.contains("docs/new-compiled-in.md"), "{err}");
    let err = check_build_paths(&BUILD_PATHS, &outside).unwrap_err();
    assert!(err.contains("docs/new-compiled-in.md"), "{err}");
    assert!(check_keys("jobs:\n", &outside).is_err());
}

#[test]
fn the_list_from_before_the_fix_is_reported_for_the_files_it_left_out() {
    let required = compiled_in_files();
    // E: the five paths `BUILD_PATHS` held before this change.
    let old_five = ["crates/", "xtask/", "Cargo.toml", "Cargo.lock", "setup.sh"];
    let err = check_build_paths(&old_five, &required).unwrap_err();
    for path in ["athena.toml", "docs/system-facts.md", "rust-toolchain.toml"] {
        assert!(err.contains(path), "{path} not named: {err}");
    }
    assert_eq!(check_build_paths(&BUILD_PATHS, &required), Ok(()));
}

#[test]
fn unreleased_changelog_names_the_cause_and_jobs() {
    let text = std::fs::read_to_string(root().join("CHANGELOG.md")).expect("read CHANGELOG.md");
    let after = text
        .split("## [Unreleased]")
        .nth(1)
        .expect("an Unreleased section");
    let unreleased = after.split("\n## [").next().unwrap();
    for needle in [
        "20261031-172040-kq9v",
        "20261017-173508-0wtc",
        "20261025-214509-y179",
        "compiled-in",
    ] {
        assert!(
            unreleased.contains(needle),
            "the Unreleased section does not mention {needle}"
        );
    }
}
