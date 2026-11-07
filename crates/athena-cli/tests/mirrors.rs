//! The hand-kept mirrors registry (`athena_core::mirrors`), run over the real files of this repository.
//!
//! A mirror is a file kept by hand in step with a governing source. Its comparison rule reads both sides, so a
//! file that exists and has the right shape but the wrong content fails here, under `cargo test`, and not at
//! the final verdict. `every_registry_entry_catches_its_drifted_fixture` walks `REGISTRY`, so a new entry that
//! uses an existing rule is covered with no other edit, and a new rule does not compile until it has a fixture.

use athena_core::mirrors::{
    self, REGISTRY, Rule, check_all, check_entry, git_base, hash_glob_problems, status_build_paths,
    test_units,
};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_real_repo_has_no_mirror_drift() {
    let root = repo_root();
    let found = check_all(&root, &git_base(&root), None);
    let lines: Vec<String> = found.iter().map(|d| d.message.clone()).collect();
    assert!(
        found.is_empty(),
        "hand-kept files disagree with the source they summarize:\n{}",
        lines.join("\n")
    );
}

fn put(root: &Path, path: &str, text: &str) {
    let full = root.join(path);
    std::fs::create_dir_all(full.parent().unwrap()).unwrap();
    std::fs::write(full, text).unwrap();
}

/// The prompt file the `prompt structure` fixture writes and reads the base text of.
const PROMPT_FIXTURE: &str = "crates/demo/prompts/reader.md";

const KEY: &str =
    "          key: athena-bin-${{ runner.os }}-${{ hashFiles('crates/**', 'athena.toml') }}\n";

/// Writes the files `entry` compares, following its rule when `drifted` is false and breaking it in one way
/// when true. Returns the text the base commit held, for the rule that reads one. The match is exhaustive: a
/// new rule needs a fixture here.
fn fixture(entry: &mirrors::Mirror, root: &Path, drifted: bool) -> Option<Vec<u8>> {
    match entry.rule {
        Rule::TimingUnits => {
            put(root, "crates/demo/tests/a.rs", "");
            put(root, entry.mirror, "demo --test a 1.00\n");
            if drifted {
                put(root, "crates/demo/tests/b.rs", "");
            }
            None
        }
        Rule::AppendOnly => {
            put(
                root,
                entry.mirror,
                if drifted {
                    "## D01 rewritten\n"
                } else {
                    "## D01\n## D02\n"
                },
            );
            Some(b"## D01\n".to_vec())
        }
        Rule::HashFilesCover => {
            let mirrors::Source::Path(ci) = entry.source else {
                panic!(
                    "{}: the build paths rule names its workflow file",
                    entry.name
                );
            };
            put(
                root,
                entry.mirror,
                "pub const BUILD_PATHS: [&str; 2] = [\"crates/\", \"athena.toml\"];\n",
            );
            let key = if drifted {
                KEY.replace(", 'athena.toml'", "")
            } else {
                KEY.to_string()
            };
            put(root, ci, &key);
            None
        }
        Rule::PromptStructure => {
            // The mirror is a glob: the fixture writes one concrete prompt file.
            let base = "- acceptance_criteria: each is a behavior; titles alone do not suffice.\n\
                 - deliverables: every path the change lists.\n";
            let new = if drifted {
                "- acceptance_criteria: each is a behavior; titles alone do not suffice.\
                 - deliverables: every path the change lists.\n"
            } else {
                "- acceptance_criteria: each is a behavior; titles alone do not suffice.\n\
                 - deliverables: every path the change lists, with its owner.\n"
            };
            put(root, PROMPT_FIXTURE, new);
            Some(base.as_bytes().to_vec())
        }
        Rule::DecisionEntries => {
            put(
                root,
                entry.mirror,
                if drifted {
                    "## D01 · a\n\n- **Provenance:** x\n\n## D02 · b\n\n- **Decision:** y\n"
                } else {
                    "## D01 · a\n\n- **Provenance:** x\n\n## D02 · b\n\n- **Provenance:** y\n"
                },
            );
            Some("## D01 · a\n\n- **Provenance:** x\n".as_bytes().to_vec())
        }
        Rule::OntologyTerms => {
            put(
                root,
                entry.mirror,
                "[[term]]\nname = \"Hermes\"\nsources = [\"BRIEF.md\"]\nsenses = [{ in = \"BRIEF.md\", marker = \"a tool\", says = \"x\" }]\n",
            );
            put(
                root,
                "BRIEF.md",
                if drifted {
                    "* Hermes: a tool\n* Hermes: a named role\n"
                } else {
                    "* Hermes: a tool\n"
                },
            );
            None
        }
    }
}

#[test]
fn every_registry_entry_catches_its_drifted_fixture() {
    assert!(!REGISTRY.is_empty());
    for entry in REGISTRY {
        let clean = tempfile::tempdir().unwrap();
        let base = fixture(entry, clean.path(), false);
        let base_of = |p: &str| base.clone().filter(|_| entry.covers(p));
        let found = check_entry(entry, clean.path(), &base_of);
        assert!(
            found.is_empty(),
            "{}: a clean fixture drifts: {found:?}",
            entry.name
        );

        let broken = tempfile::tempdir().unwrap();
        let base = fixture(entry, broken.path(), true);
        let base_of = |p: &str| base.clone().filter(|_| entry.covers(p));
        let found = check_entry(entry, broken.path(), &base_of);
        assert!(
            !found.is_empty(),
            "{}: a drifted fixture passes",
            entry.name
        );
        for drift in &found {
            // A glob entry names the concrete file that drifted.
            assert!(
                entry.covers(&drift.mirror),
                "{}: {} is no file of the entry",
                entry.name,
                drift.mirror
            );
            assert!(
                drift.message.contains(&drift.mirror),
                "{}: the message names the file: {drift}",
                entry.name
            );
            assert!(
                ["add ", "remove ", "put the old text back"]
                    .iter()
                    .any(|fix| drift.message.contains(fix)),
                "{}: the message names the fix: {drift}",
                entry.name
            );
        }
    }
}

/// The `--lib` and `--test` units of the targets cargo tests, from `cargo metadata`.
fn cargo_units() -> BTreeSet<String> {
    let out = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
        .current_dir(repo_root())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let meta: Value = serde_json::from_slice(&out.stdout).unwrap();
    let mut units = BTreeSet::new();
    for package in meta["packages"].as_array().unwrap() {
        let name = package["name"].as_str().unwrap();
        for target in package["targets"].as_array().unwrap() {
            if target["test"] != Value::Bool(true) {
                continue;
            }
            let target_name = target["name"].as_str().unwrap();
            match target["kind"][0].as_str().unwrap() {
                "lib" | "rlib" | "proc-macro" => units.insert(format!("{name} --lib")),
                "test" => units.insert(format!("{name} --test {target_name}")),
                _ => false,
            };
        }
    }
    units
}

#[test]
fn derived_units_agree_with_cargo_metadata() {
    let derived = test_units(&repo_root());
    let cargo = cargo_units();
    assert_eq!(
        derived, cargo,
        "the test units derived from the files on disk differ from cargo's; fix the derivation in \
         athena_core::mirrors::test_units, not the timings file"
    );
}

#[test]
fn the_ci_key_and_build_paths_cover_each_other() {
    let root = repo_root();
    let status = std::fs::read_to_string(root.join("crates/athena-core/src/status.rs")).unwrap();
    let ci = std::fs::read_to_string(root.join(".github/workflows/ci.yml")).unwrap();
    let parsed = status_build_paths(&status).expect("BUILD_PATHS parses");
    let compiled: Vec<String> = athena_core::status::BUILD_PATHS
        .iter()
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        parsed, compiled,
        "the parse of status.rs reads another list than the constant"
    );
    let found = hash_glob_problems(&ci, &parsed);
    assert!(found.is_empty(), "{}", found.join("\n"));
}
