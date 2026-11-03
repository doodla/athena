//! `athena-core --lib` is split into parts by test-name prefix so its 800-odd tests spread over the CI shards
//! (see `xtask/src/shard.rs`). The rule checked here: every test name the library lists is kept by exactly
//! one part's filter, so each test runs once across the shards.
//!
//! The parts come from the built `xtask test-shard <k> 6 --dry-run`, and the matching below follows libtest's
//! documented filter rules without any code from the splitter: a positional argument keeps a test whose name
//! contains it (several arguments: any of them), and `--skip <s>` drops a test whose name contains `s`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

/// One part of the split binary: the fragments it keeps (empty = all) and the fragments it drops.
#[derive(Clone, Debug)]
struct Part {
    only: Vec<String>,
    skip: Vec<String>,
}

impl Part {
    /// libtest's decision for one test name.
    fn keeps(&self, name: &str) -> bool {
        (self.only.is_empty() || self.only.iter().any(|f| name.contains(f.as_str())))
            && !self.skip.iter().any(|f| name.contains(f.as_str()))
    }
}

const PREFIX: &str = "athena-core --lib -- ";

/// The parts every shard prints, from `test-shard <k> 6 --dry-run`.
fn parts_from_dry_run() -> Vec<Part> {
    let mut parts = Vec::new();
    for k in 0..6 {
        let out = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .args(["test-shard", &k.to_string(), "6", "--dry-run"])
            .current_dir(root())
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        for line in String::from_utf8(out.stdout).unwrap().lines() {
            if let Some(args) = line.strip_prefix(PREFIX) {
                parts.push(parse_args(args));
            }
        }
    }
    parts
}

fn parse_args(args: &str) -> Part {
    let mut part = Part {
        only: Vec::new(),
        skip: Vec::new(),
    };
    let mut words = args.split_whitespace();
    while let Some(w) = words.next() {
        if w == "--skip" {
            part.skip
                .push(words.next().expect("--skip needs a value").to_string());
        } else {
            part.only.push(w.to_string());
        }
    }
    part
}

/// Every test name in athena-core's library, from `cargo test -p athena-core --lib -- --list`.
fn lib_test_names() -> Vec<String> {
    let out = Command::new("cargo")
        .args(["test", "--locked", "-p", "athena-core", "--lib", "--"])
        .args(["--list", "--format", "terse"])
        .current_dir(root())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .filter_map(|l| l.strip_suffix(": test"))
        .map(String::from)
        .collect()
}

/// Passes only if each name is kept by exactly one part.
fn check_exactly_once(names: &[String], parts: &[Part]) -> Result<(), String> {
    for name in names {
        let hits = parts.iter().filter(|p| p.keeps(name)).count();
        if hits != 1 {
            return Err(format!("{name} is kept by {hits} parts, expected 1"));
        }
    }
    Ok(())
}

#[test]
fn the_library_is_split_into_at_least_two_parts() {
    assert!(parts_from_dry_run().len() >= 2);
}

#[test]
fn every_athena_core_lib_test_is_kept_by_exactly_one_part() {
    let names = lib_test_names();
    assert!(names.len() > 700, "only {} tests listed", names.len());
    check_exactly_once(&names, &parts_from_dry_run()).unwrap();
}

#[test]
fn a_dropped_prefix_is_rejected() {
    let names = lib_test_names();
    let parts = parts_from_dry_run();
    // Take the first prefix off the first part that has one; the catch-all still skips it, so tests vanish.
    let mut dropped = parts.clone();
    let part = dropped.iter_mut().find(|p| !p.only.is_empty()).unwrap();
    let prefix = part.only.remove(0);
    let err = check_exactly_once(&names, &dropped).unwrap_err();
    assert!(err.contains("kept by 0 parts"), "{prefix}: {err}");
}

#[test]
fn a_duplicated_prefix_is_rejected() {
    let names = lib_test_names();
    let parts = parts_from_dry_run();
    let mut doubled = parts.clone();
    let prefix = doubled.iter().find(|p| !p.only.is_empty()).unwrap().only[0].clone();
    // Add the same prefix to a second part that keeps specific prefixes.
    let other = doubled
        .iter_mut()
        .filter(|p| !p.only.is_empty() && !p.only.contains(&prefix))
        .next()
        .unwrap();
    other.only.push(prefix.clone());
    let err = check_exactly_once(&names, &doubled).unwrap_err();
    assert!(err.contains("kept by 2 parts"), "{prefix}: {err}");
}

#[test]
fn a_name_in_a_new_module_goes_to_the_catch_all_only() {
    let parts = parts_from_dry_run();
    let names = vec!["brand_new_module::tests::works".to_string()];
    check_exactly_once(&names, &parts).unwrap();
}
