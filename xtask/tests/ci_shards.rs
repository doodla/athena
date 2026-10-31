//! CI's parallel layout (docs/design/ci-routing.md §4.2): `.github/workflows/ci.yml` runs fmt, clippy and the
//! tests as separate jobs, the tests in shards, and a final `ci` job says whether a commit is green.
//!
//! The central rule is that the shards form a partition: the test command each shard runs, taken together,
//! covers every workspace test target exactly once. It is checked in three layers:
//!
//! 1. `parse_workflow` reads the shard list and the `test-shard` command out of ci.yml (a shape rule);
//! 2. each listed shard is asked what it runs, with the built `xtask test-shard <k> <n> --dry-run`;
//! 3. the pure `check_cover` compares those sets with an oracle built here, from `cargo metadata`, without any
//!    code from the splitter, and cross-checked against the source files on disk.
//!
//! `check_cover` is separate so tests can feed it well-formed but wrong shards: a text change to ci.yml always
//! changes the matrix shape and is rejected by layer 1, so it would never reach the comparison.
//!
//! Why the oracle does not call `cargo test --workspace --no-run`: it compiles the whole workspace and prints
//! artifact paths, not stable target names.

use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn ci_yml() -> String {
    std::fs::read_to_string(root().join(".github/workflows/ci.yml")).unwrap()
}

fn xtask(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(args)
        .current_dir(root())
        .output()
        .unwrap()
}

// --------------------------------------------------------------------------------------------------------
// Reading ci.yml

/// The jobs of the workflow: job id to its lines (comment lines left out), in file order.
fn jobs(text: &str) -> BTreeMap<String, Vec<String>> {
    let key = Regex::new(r"^  ([A-Za-z][A-Za-z0-9_-]*):\s*$").unwrap();
    let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut current: Option<String> = None;
    let mut in_jobs = false;
    for line in text.lines() {
        if line == "jobs:" {
            in_jobs = true;
            continue;
        }
        if !in_jobs {
            continue;
        }
        if !line.starts_with(' ') && !line.is_empty() && !line.starts_with('#') {
            break; // another top-level key
        }
        if let Some(c) = key.captures(line) {
            current = Some(c[1].to_string());
            out.insert(c[1].to_string(), Vec::new());
        } else if !line.trim_start().starts_with('#')
            && let Some(job) = &current
        {
            out.get_mut(job).unwrap().push(line.to_string());
        }
    }
    out
}

/// The matrix list and the shard count `n` that ci.yml's `test` job runs, or why they cannot be read.
fn parse_workflow(text: &str) -> Result<(Vec<u32>, u32), String> {
    let all = jobs(text);
    let test = all.get("test").ok_or("ci.yml has no job `test`")?;
    let block = test.join("\n");
    let list = Regex::new(r"(?m)^        shard: \[([0-9, ]*)\]\s*$").unwrap();
    let lists: Vec<_> = list.captures_iter(&block).collect();
    let [list] = lists.as_slice() else {
        return Err(format!(
            "the test job needs exactly one `shard: [..]` matrix list, found {}",
            lists.len()
        ));
    };
    let shards: Vec<u32> = list[1]
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().map_err(|_| format!("bad shard number {s:?}")))
        .collect::<Result<_, _>>()?;
    let command = Regex::new(r"test-shard \$\{\{ matrix\.shard \}\} ([0-9]+)\s*$").unwrap();
    let counts: Vec<u32> = block
        .lines()
        .filter_map(|l| command.captures(l))
        .map(|c| c[1].parse().unwrap())
        .collect();
    let [n] = counts.as_slice() else {
        return Err(format!(
            "the test job needs exactly one `test-shard ${{{{ matrix.shard }}}} <n>` command, found {}",
            counts.len()
        ));
    };
    let want: Vec<u32> = (0..*n).collect();
    let mut sorted = shards.clone();
    sorted.sort_unstable();
    if sorted != want {
        return Err(format!(
            "the matrix list {shards:?} must be exactly 0..{n} (each shard once), the command's n is {n}"
        ));
    }
    Ok((shards, *n))
}

// --------------------------------------------------------------------------------------------------------
// The oracle: the workspace's test targets, from cargo metadata

fn metadata() -> Value {
    let out = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
        .current_dir(root())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

/// What `cargo test --workspace` runs for one target of one package, by cargo's documented rules: a target
/// is tested only if its `test` flag is set; libraries run as `--lib`, binaries and integration tests by
/// name; examples and benches are not tested.
fn target_unit(package: &str, target: &Value) -> Option<String> {
    if target["test"] != Value::Bool(true) {
        return None;
    }
    let name = target["name"].as_str().unwrap();
    let kind = target["kind"][0].as_str().unwrap();
    match kind {
        "lib" | "rlib" | "proc-macro" => Some(format!("{package} --lib")),
        "bin" => Some(format!("{package} --bin {name}")),
        "test" => Some(format!("{package} --test {name}")),
        _ => None,
    }
}

/// Every unit `cargo test --workspace` runs: each tested target, plus one doctest run per library with
/// doctests on.
fn oracle() -> BTreeSet<String> {
    let meta = metadata();
    let mut units = BTreeSet::new();
    for package in meta["packages"].as_array().unwrap() {
        let name = package["name"].as_str().unwrap();
        for target in package["targets"].as_array().unwrap() {
            if let Some(unit) = target_unit(name, target) {
                if unit.ends_with(" --lib") && target["doctest"] == Value::Bool(true) {
                    units.insert(format!("{name} --doc"));
                }
                units.insert(unit);
            }
        }
    }
    units
}

/// Source files cargo discovers as targets by default, in one package directory.
fn discovered_sources(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for single in ["src/lib.rs", "src/main.rs"] {
        if dir.join(single).is_file() {
            files.push(dir.join(single));
        }
    }
    for sub in ["src/bin", "tests"] {
        let Ok(entries) = std::fs::read_dir(dir.join(sub)) else {
            continue;
        };
        for entry in entries.map(|e| e.unwrap().path()) {
            if entry.is_file() && entry.extension().is_some_and(|e| e == "rs") {
                files.push(entry);
            } else if entry.join("main.rs").is_file() {
                files.push(entry.join("main.rs"));
            }
        }
    }
    files
}

#[test]
fn oracle_matches_files_on_disk() {
    // Every target source file in a member is, by its path, a metadata target that cargo tests; a renamed
    // target (athena-cli's `[[bin]] name = "athena"`) is found by `src_path`, not by a derived name.
    let meta = metadata();
    let oracle = oracle();
    let mut seen_units = BTreeSet::new();
    for package in meta["packages"].as_array().unwrap() {
        let name = package["name"].as_str().unwrap();
        let manifest = Path::new(package["manifest_path"].as_str().unwrap());
        let dir = manifest.parent().unwrap();
        let by_path: BTreeMap<PathBuf, &Value> = package["targets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| {
                (
                    Path::new(t["src_path"].as_str().unwrap())
                        .canonicalize()
                        .unwrap(),
                    t,
                )
            })
            .collect();
        for file in discovered_sources(dir) {
            let canonical = file.canonicalize().unwrap();
            let target = by_path
                .get(&canonical)
                .unwrap_or_else(|| panic!("{} is not a target of {name}", file.display()));
            let unit = target_unit(name, target).unwrap_or_else(|| {
                panic!(
                    "{} is a target of {name} that cargo does not test",
                    file.display()
                )
            });
            assert!(oracle.contains(&unit), "{unit} (from {})", file.display());
            seen_units.insert(unit);
        }
    }
    // The renamed binary is covered, by its name in Cargo.toml.
    assert!(seen_units.contains("athena-cli --bin athena"));
    // Every non-doctest oracle unit came from a file on disk (so the oracle holds nothing invented).
    for unit in &oracle {
        assert!(
            unit.ends_with(" --doc") || seen_units.contains(unit),
            "{unit} has no source file"
        );
    }
}

// --------------------------------------------------------------------------------------------------------
// The partition rule

/// What shard `k` of `n` runs, per `xtask test-shard <k> <n> --dry-run`.
fn shard_units(k: u32, n: u32) -> Result<Vec<String>, String> {
    let out = xtask(&["test-shard", &k.to_string(), &n.to_string(), "--dry-run"]);
    if !out.status.success() {
        return Err(format!(
            "test-shard {k} {n} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(String::from)
        .collect())
}

/// Passes only if every oracle unit is in exactly one shard and no shard has anything else.
fn check_cover(shards: &[Vec<String>], oracle: &BTreeSet<String>) -> Result<(), String> {
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for unit in shards.iter().flatten() {
        *count.entry(unit.as_str()).or_default() += 1;
    }
    for (unit, times) in &count {
        if !oracle.contains(*unit) {
            return Err(format!(
                "{unit} is in a shard but is not a workspace test target"
            ));
        }
        if *times > 1 {
            return Err(format!(
                "{unit} runs in more than one shard ({times} times)"
            ));
        }
    }
    for unit in oracle {
        if !count.contains_key(unit.as_str()) {
            return Err(format!("{unit} is in no shard"));
        }
    }
    Ok(())
}

/// ci.yml's shards, run through the splitter and compared with the oracle.
fn check_partition(text: &str) -> Result<(), String> {
    let (list, n) = parse_workflow(text)?;
    let shards: Vec<Vec<String>> = list
        .iter()
        .map(|k| shard_units(*k, n))
        .collect::<Result<_, _>>()?;
    check_cover(&shards, &oracle())
}

fn real_shards() -> Vec<Vec<String>> {
    (0..6).map(|k| shard_units(k, 6).unwrap()).collect()
}

#[test]
fn shards_in_ci_yml_partition_every_workspace_test_target() {
    check_partition(&ci_yml()).unwrap();
}

#[test]
fn partition_rejects_a_changed_shard_list() {
    let text = ci_yml();
    assert!(text.contains("shard: [0, 1, 2, 3, 4, 5]"));
    assert!(text.contains("test-shard ${{ matrix.shard }} 6"));
    let mutations = [
        // a dropped shard
        text.replacen("shard: [0, 1, 2, 3, 4, 5]", "shard: [0, 1, 2, 3, 4]", 1),
        // a duplicated shard
        text.replacen("shard: [0, 1, 2, 3, 4, 5]", "shard: [0, 1, 2, 3, 4, 4]", 1),
        // a shard past n
        text.replacen("shard: [0, 1, 2, 3, 4, 5]", "shard: [0, 1, 2, 3, 4, 6]", 1),
        // a count that no longer matches the list
        text.replacen(
            "test-shard ${{ matrix.shard }} 6",
            "test-shard ${{ matrix.shard }} 5",
            1,
        ),
        // a command that no longer takes the shard from the matrix
        text.replacen("test-shard ${{ matrix.shard }} 6", "test-shard 0 6", 1),
        // no matrix list at all
        text.replacen("shard: [0, 1, 2, 3, 4, 5]", "shard: ${{ fromJSON(x) }}", 1),
    ];
    for mutated in mutations {
        assert_ne!(mutated, text, "the mutation changed nothing");
        assert!(check_partition(&mutated).is_err(), "accepted:\n{mutated}");
    }
}

#[test]
fn cover_accepts_the_real_shards() {
    check_cover(&real_shards(), &oracle()).unwrap();
}

#[test]
fn cover_rejects_a_unit_in_no_shard() {
    let mut shards = real_shards();
    let removed = shards[0].remove(0);
    let err = check_cover(&shards, &oracle()).unwrap_err();
    assert!(err.contains(&removed) && err.contains("no shard"), "{err}");
}

#[test]
fn cover_rejects_a_unit_in_two_shards() {
    let mut shards = real_shards();
    let copied = shards[0][0].clone();
    shards[1].push(copied.clone());
    let err = check_cover(&shards, &oracle()).unwrap_err();
    assert!(
        err.contains(&copied) && err.contains("more than one"),
        "{err}"
    );
}

#[test]
fn cover_rejects_a_unit_twice_in_one_shard() {
    let mut shards = real_shards();
    let copied = shards[2][0].clone();
    shards[2].push(copied.clone());
    let err = check_cover(&shards, &oracle()).unwrap_err();
    assert!(err.contains(&copied), "{err}");
}

#[test]
fn cover_rejects_an_oracle_unit_the_shards_lack() {
    let mut oracle = oracle();
    oracle.insert("synthetic-package --test extra".to_string());
    let err = check_cover(&real_shards(), &oracle).unwrap_err();
    assert!(
        err.contains("synthetic-package --test extra") && err.contains("no shard"),
        "{err}"
    );
}

#[test]
fn cover_rejects_a_shard_unit_that_is_not_a_target() {
    let mut shards = real_shards();
    shards[2].push("ghost --lib".to_string());
    let err = check_cover(&shards, &oracle()).unwrap_err();
    assert!(
        err.contains("ghost --lib") && err.contains("not a workspace test target"),
        "{err}"
    );
}

// --------------------------------------------------------------------------------------------------------
// The splitter command

#[test]
fn test_shard_6_is_disjoint_and_equals_the_workspace_target_set() {
    let shards = real_shards();
    let mut union = BTreeSet::new();
    for (k, shard) in shards.iter().enumerate() {
        for unit in shard {
            assert!(
                union.insert(unit.clone()),
                "{unit} in shard {k} and earlier"
            );
        }
    }
    assert_eq!(union, oracle());
}

#[test]
fn test_shard_1_equals_the_workspace() {
    let one: BTreeSet<String> = shard_units(0, 1).unwrap().into_iter().collect();
    assert_eq!(one, oracle());
}

#[test]
fn test_shard_output_is_stable_and_sorted() {
    let first = shard_units(0, 1).unwrap();
    assert_eq!(first, shard_units(0, 1).unwrap());
    let mut sorted = first.clone();
    sorted.sort();
    assert_eq!(first, sorted);
}

#[test]
fn test_shard_rejects_bad_arguments() {
    for args in [
        &["test-shard", "6", "6"][..],
        &["test-shard", "0", "0"],
        &["test-shard", "3", "2"],
        &["test-shard", "x", "6"],
        &["test-shard", "0"],
        &["test-shard"],
    ] {
        let out = xtask(args);
        assert!(!out.status.success(), "{args:?} was accepted");
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("error:"),
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

// --------------------------------------------------------------------------------------------------------
// The workflow's jobs

/// Why ci.yml's job layout is wrong, if it is.
fn check_jobs(text: &str) -> Result<(), String> {
    let all = jobs(text);
    for id in ["gate", "fmt", "clippy", "test", "pass", "ci"] {
        if !all.contains_key(id) {
            return Err(format!("no job `{id}`"));
        }
    }
    let (list, n) = parse_workflow(text)?;
    if list.len() != 6 || n != 6 {
        return Err(format!("expected 6 shards, found {} (n={n})", list.len()));
    }
    let timeout = Regex::new(r"^    timeout-minutes: [0-9]+$").unwrap();
    for (id, lines) in &all {
        if !lines.iter().any(|l| timeout.is_match(l)) {
            return Err(format!("job `{id}` has no timeout-minutes"));
        }
    }
    check_required_name(text)?;
    let ci = &all["ci"];
    if !ci.iter().any(|l| l == "    if: always()") {
        return Err("the ci job lacks `if: always()`".into());
    }
    let needs = ci
        .iter()
        .find_map(|l| l.strip_prefix("    needs: ["))
        .and_then(|l| l.strip_suffix(']'))
        .ok_or("the ci job has no `needs: [..]` list")?;
    let needed: BTreeSet<&str> = needs.split(',').map(str::trim).collect();
    for id in ["gate", "fmt", "clippy", "test"] {
        if !needed.contains(id) {
            return Err(format!("the ci job does not need `{id}`"));
        }
    }
    Ok(())
}

/// The check name branch protection requires. The job that reports it has the id `ci`.
const REQUIRED_CHECK_NAME: &str = "fmt, clippy, tests";

/// Why the required check name is not carried by exactly one job (the `ci` job) that needs every check job.
fn check_required_name(text: &str) -> Result<(), String> {
    let line = format!("    name: {REQUIRED_CHECK_NAME}");
    let named: Vec<String> = jobs(text)
        .into_iter()
        .filter(|(_, lines)| lines.contains(&line))
        .map(|(id, _)| id)
        .collect();
    if named.len() != 1 {
        return Err(format!(
            "expected exactly one job named `{REQUIRED_CHECK_NAME}`, found {named:?}"
        ));
    }
    if named[0] != "ci" {
        return Err(format!(
            "the job named `{REQUIRED_CHECK_NAME}` is `{}`, not `ci`",
            named[0]
        ));
    }
    let all = jobs(text);
    let needs = all["ci"]
        .iter()
        .find_map(|l| l.strip_prefix("    needs: ["))
        .and_then(|l| l.strip_suffix(']'))
        .ok_or("the required-check job has no `needs: [..]` list")?;
    let needed: BTreeSet<&str> = needs.split(',').map(str::trim).collect();
    for id in ["gate", "fmt", "clippy", "test"] {
        if !needed.contains(id) {
            return Err(format!("the required-check job does not need `{id}`"));
        }
    }
    Ok(())
}

#[test]
fn workflow_required_check_name_is_on_exactly_one_job_that_needs_every_check() {
    let text = ci_yml();
    check_required_name(&text).unwrap();
    let named = format!("    name: {REQUIRED_CHECK_NAME}\n");
    let pass_name = jobs(&text)["pass"]
        .iter()
        .find(|l| l.starts_with("    name: "))
        .cloned()
        .expect("the pass job has a name");
    let mutations = [
        // The final job reverts to the name `ci`.
        text.replacen(&named, "    name: ci\n", 1),
        // Another job takes the required name as well.
        text.replacen(
            &format!("{pass_name}\n"),
            &format!("    name: {REQUIRED_CHECK_NAME}\n"),
            1,
        ),
        // The named job stops needing a check job.
        text.replacen(
            "    needs: [gate, fmt, clippy, test]\n    if: always()",
            "    needs: [gate, clippy, test]\n    if: always()",
            1,
        ),
    ];
    for mutated in mutations {
        assert_ne!(mutated, text, "the mutation changed nothing");
        assert!(
            check_required_name(&mutated).is_err(),
            "accepted:\n{mutated}"
        );
    }
}

#[test]
fn workflow_jobs_are_separate_sharded_and_gated_by_ci() {
    check_jobs(&ci_yml()).unwrap();
}

#[test]
fn workflow_jobs_reject_a_missing_gate_condition_need_or_timeout() {
    let text = ci_yml();
    let mutations = [
        text.replacen("    if: always()\n", "", 1),
        text.replacen(
            "    needs: [gate, fmt, clippy, test]\n    if: always()",
            "    needs: [gate, clippy, test]\n    if: always()",
            1,
        ),
        text.replacen("    name: fmt\n", "    name: fmt\n    # x\n", 1)
            .replacen("  fmt:\n", "  format:\n", 1),
        text.replacen("    timeout-minutes: 15\n", "", 1),
        text.replacen("    timeout-minutes: 5\n", "", 1),
        text.replacen("    timeout-minutes: 30\n", "", 1),
        text.replacen("shard: [0, 1, 2, 3, 4, 5]", "shard: [0, 1, 2, 3]", 1)
            .replacen("matrix.shard }} 6", "matrix.shard }} 4", 1),
    ];
    for mutated in mutations {
        assert_ne!(mutated, text, "the mutation changed nothing");
        assert!(check_jobs(&mutated).is_err(), "accepted:\n{mutated}");
    }
}

/// The shell script of the ci job's only `run` step, dedented.
fn ci_script() -> String {
    let all = jobs(&ci_yml());
    let lines = &all["ci"];
    let start = lines
        .iter()
        .position(|l| l == "        run: |")
        .expect("the ci job has a run step");
    lines[start + 1..]
        .iter()
        .take_while(|l| l.starts_with("          ") || l.is_empty())
        .map(|l| l.get(10..).unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

fn ci_exit(gate: &str, skip: &str, fmt: &str, clippy: &str, test: &str) -> bool {
    Command::new("sh")
        .args(["-c", &ci_script()])
        .env("GATE", gate)
        .env("SKIP", skip)
        .env("FMT", fmt)
        .env("CLIPPY", clippy)
        .env("TEST", test)
        .output()
        .unwrap()
        .status
        .success()
}

#[test]
fn workflow_ci_job_is_green_only_when_every_check_succeeded() {
    let ok = "success";
    assert!(ci_exit(ok, "false", ok, ok, ok), "all succeeded");
    assert!(ci_exit(ok, "", ok, ok, ok), "an empty skip output is false");
    // A reused pass: the three checks are skipped on purpose.
    assert!(ci_exit(ok, "true", "skipped", "skipped", "skipped"));
    // Any other result is red.
    for bad in ["failure", "cancelled", "skipped"] {
        assert!(!ci_exit(ok, "false", bad, ok, ok), "fmt {bad}");
        assert!(!ci_exit(ok, "false", ok, bad, ok), "clippy {bad}");
        assert!(!ci_exit(ok, "false", ok, ok, bad), "test {bad}");
    }
    for bad in ["failure", "cancelled", "skipped", ""] {
        assert!(!ci_exit(bad, "false", ok, ok, ok), "gate {bad:?}");
    }
    // A reused pass does not excuse a check that ran and failed, or one that was cancelled.
    assert!(!ci_exit(ok, "true", "failure", "skipped", "skipped"));
    assert!(!ci_exit(ok, "true", "skipped", "skipped", "cancelled"));
    assert!(!ci_exit(ok, "true", ok, ok, ok));
    // A failed gate is red even if the others look skipped.
    assert!(!ci_exit("failure", "true", "skipped", "skipped", "skipped"));
}

#[test]
fn workflow_actions_are_pinned() {
    let text = ci_yml();
    let any = Regex::new(r"^\s*(- )?uses:").unwrap();
    let pinned = Regex::new(r"uses:\s+[A-Za-z0-9_./-]+@[0-9a-f]{40}\s+# v").unwrap();
    let mut uses = 0;
    for line in text.lines().filter(|l| any.is_match(l)) {
        uses += 1;
        assert!(pinned.is_match(line), "not pinned by commit SHA: {line}");
    }
    assert!(uses >= 20, "found only {uses} uses: lines");
}

#[test]
fn workflow_caches_keep_three_layers_in_each_check_job() {
    let all = jobs(&ci_yml());
    for (id, target) in [
        ("fmt", "target-fmt-"),
        ("clippy", "target-clippy-"),
        ("test", "target-test-"),
    ] {
        let text = all[id].join("\n");
        for needle in [
            "actions/cache/restore@",
            "actions/cache/save@",
            "key: rustup-${{ hashFiles('rust-toolchain.toml') }}",
            "key: registry-${{ hashFiles('Cargo.lock') }}",
            &format!(
                "key: {target}${{{{ hashFiles('rust-toolchain.toml') }}}}-${{{{ hashFiles('Cargo.lock') }}}}"
            ),
            &format!("restore-keys: {target}${{{{ hashFiles('rust-toolchain.toml') }}}}-"),
            "default_branch",
        ] {
            assert!(text.contains(needle), "{id}: {needle}");
        }
    }
    // The six shards share their cache keys, so only shard 0 saves them.
    let test = all["test"].join("\n");
    assert_eq!(
        test.matches("if: matrix.shard == 0 && ").count(),
        test.matches("actions/cache/save@").count()
    );
    // The earlier-pass lookup and marker are still there.
    assert!(all["gate"].join("\n").contains("path: ci-pass"));
    assert!(all["pass"].join("\n").contains("ci-cache mark ci-pass"));
}
