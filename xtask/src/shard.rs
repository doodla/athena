//! `test-shard <k> <n> [--dry-run]`: runs one of `n` disjoint slices of the workspace's tests, so CI can
//! spread `cargo test --workspace` over several runners and a developer can run the same slice locally.
//!
//! The unit of sharding is a workspace test target: each library, binary and integration-test file, plus one
//! `--doc` unit per library with doctests. `cargo metadata` lists them, the list is sorted by
//! (package, selector), and unit `i` belongs to shard `i % n`. Shards are numbered from 0, so `k` is in `0..n`.
//! The `n` shards together run exactly what `cargo test --workspace` runs.
//!
//! Buy-first call (docs/conventions): `cargo metadata` and `cargo test` already meet the need, so no sharding
//! tool is added. `cargo-nextest` was rejected: it is a new pinned tool and changes how doctests run.
//!
//! Known difference, for the throughput review: cargo cannot attach a different target selector to each `-p`,
//! so a shard runs one `cargo test --locked -p <package> <selector>` per unit. Cargo resolves features over the
//! packages of one invocation, so a per-unit run can compile a dependency with a different feature set than
//! one `--workspace` run would. Grouping a shard's units by package is the follow-up if the compile cost shows.

use anyhow::{Context, Result, anyhow, bail};
use serde_json::Value;
use std::path::Path;
use std::process::Command;

pub const USAGE: &str = "cargo run -p xtask -- test-shard <k> <n> [--dry-run]\n       \
     runs shard k (0-based, k < n) of n of the workspace's test targets; --dry-run only lists them";

/// One slice of `cargo test --workspace`: a package and the cargo selector for one of its test targets.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Unit {
    pub package: String,
    /// `--lib`, `--doc`, `--bin <name>` or `--test <name>`.
    pub selector: String,
}

impl Unit {
    /// The line `--dry-run` prints for this unit.
    pub fn line(&self) -> String {
        format!("{} {}", self.package, self.selector)
    }
}

/// The test units of every workspace member in `cargo metadata --no-deps` output, sorted.
///
/// Targets cargo does not test (examples, benches, and anything with `test = false`) are left out; a library
/// whose doctests are enabled adds a `--doc` unit, which cargo runs once per package.
pub fn plan_units(metadata: &Value) -> Result<Vec<Unit>> {
    let packages = metadata["packages"]
        .as_array()
        .ok_or_else(|| anyhow!("cargo metadata has no packages list"))?;
    let mut units = Vec::new();
    for package in packages {
        let name = package["name"]
            .as_str()
            .ok_or_else(|| anyhow!("a package has no name"))?;
        let targets = package["targets"]
            .as_array()
            .ok_or_else(|| anyhow!("package {name} has no targets list"))?;
        for target in targets {
            let target_name = target["name"]
                .as_str()
                .ok_or_else(|| anyhow!("a target of {name} has no name"))?;
            let kinds: Vec<&str> = target["kind"]
                .as_array()
                .map(|k| k.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default();
            if target["test"].as_bool() != Some(true) {
                continue;
            }
            let is_lib = kinds
                .iter()
                .any(|k| matches!(*k, "lib" | "rlib" | "proc-macro"));
            let selector = if is_lib {
                Some("--lib".to_string())
            } else if kinds.contains(&"bin") {
                Some(format!("--bin {target_name}"))
            } else if kinds.contains(&"test") {
                Some(format!("--test {target_name}"))
            } else {
                None
            };
            if let Some(selector) = selector {
                units.push(Unit {
                    package: name.to_string(),
                    selector,
                });
            }
            if is_lib && target["doctest"].as_bool() == Some(true) {
                units.push(Unit {
                    package: name.to_string(),
                    selector: "--doc".to_string(),
                });
            }
        }
    }
    units.sort();
    units.dedup();
    Ok(units)
}

/// The units of shard `k` of `n`: those at sorted position `i` with `i % n == k`.
pub fn select(units: &[Unit], k: u32, n: u32) -> Vec<Unit> {
    units
        .iter()
        .enumerate()
        .filter(|(i, _)| (*i as u64) % u64::from(n) == u64::from(k))
        .map(|(_, unit)| unit.clone())
        .collect()
}

/// The parsed command line.
#[derive(Debug, PartialEq, Eq)]
pub struct Args {
    pub k: u32,
    pub n: u32,
    pub dry_run: bool,
}

pub fn parse_args(args: &[String]) -> Result<Args> {
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let rest: Vec<&String> = args.iter().filter(|a| *a != "--dry-run").collect();
    let [k, n] = rest.as_slice() else {
        bail!("test-shard needs <k> and <n>\n{USAGE}");
    };
    let k: u32 = k
        .parse()
        .map_err(|_| anyhow!("<k> must be a whole number, got {k:?}\n{USAGE}"))?;
    let n: u32 = n
        .parse()
        .map_err(|_| anyhow!("<n> must be a whole number, got {n:?}\n{USAGE}"))?;
    if n == 0 {
        bail!("<n> must be at least 1\n{USAGE}");
    }
    if k >= n {
        bail!("<k> must be below <n> (shards are numbered from 0): got k={k}, n={n}\n{USAGE}");
    }
    Ok(Args { k, n, dry_run })
}

/// `test-shard <k> <n> [--dry-run]`.
pub fn command(args: &[String], root: &Path) -> Result<()> {
    let Args { k, n, dry_run } = parse_args(args)?;
    let out = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1", "--locked"])
        .current_dir(root)
        .output()
        .context("could not run cargo metadata")?;
    if !out.status.success() {
        bail!(
            "cargo metadata failed with {}: {}",
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let metadata: Value =
        serde_json::from_slice(&out.stdout).context("cargo metadata did not print JSON")?;
    let chosen = select(&plan_units(&metadata)?, k, n);
    if dry_run {
        for unit in &chosen {
            println!("{}", unit.line());
        }
        return Ok(());
    }
    if chosen.is_empty() {
        println!("shard {k} of {n} has no test targets; nothing to run");
        return Ok(());
    }
    let mut failed = Vec::new();
    for unit in &chosen {
        crate::step(&format!("shard {k}/{n}: {}", unit.line()));
        let status = Command::new("cargo")
            .args(["test", "--locked", "-p", &unit.package])
            .args(unit.selector.split_whitespace())
            .current_dir(root)
            .status()
            .context("could not run cargo test")?;
        if !status.success() {
            failed.push(unit.line());
        }
    }
    if failed.is_empty() {
        println!("\nshard {k} of {n}: {} units passed", chosen.len());
        Ok(())
    } else {
        bail!(
            "shard {k} of {n}: {} of {} units failed: {}",
            failed.len(),
            chosen.len(),
            failed.join("; ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> Value {
        json!({"packages": [
            {"name": "b-pkg", "targets": [
                {"name": "b_pkg", "kind": ["lib"], "test": true, "doctest": true},
                {"name": "bee", "kind": ["bin"], "test": true, "doctest": false},
                {"name": "demo", "kind": ["example"], "test": false, "doctest": false},
                {"name": "speed", "kind": ["bench"], "test": false, "doctest": false},
                {"name": "it", "kind": ["test"], "test": true, "doctest": false}
            ]},
            {"name": "a-pkg", "targets": [
                {"name": "a_pkg", "kind": ["lib"], "test": true, "doctest": false},
                {"name": "off", "kind": ["bin"], "test": false, "doctest": false}
            ]}
        ]})
    }

    fn lines(units: &[Unit]) -> Vec<String> {
        units.iter().map(Unit::line).collect()
    }

    #[test]
    fn plan_lists_test_targets_sorted_and_leaves_out_what_cargo_does_not_test() {
        let units = plan_units(&fixture()).unwrap();
        assert_eq!(
            lines(&units),
            [
                "a-pkg --lib",
                "b-pkg --bin bee",
                "b-pkg --doc",
                "b-pkg --lib",
                "b-pkg --test it"
            ]
        );
    }

    #[test]
    fn a_library_with_doctests_off_has_no_doc_unit() {
        let units = plan_units(&fixture()).unwrap();
        assert!(!lines(&units).contains(&"a-pkg --doc".to_string()));
    }

    #[test]
    fn plan_rejects_metadata_without_packages() {
        assert!(plan_units(&json!({})).is_err());
    }

    #[test]
    fn select_is_disjoint_and_complete_for_any_n() {
        let units = plan_units(&fixture()).unwrap();
        for n in [1u32, 2, 3, 5, 6, 9] {
            let mut seen: Vec<Unit> = Vec::new();
            for k in 0..n {
                seen.extend(select(&units, k, n));
            }
            assert_eq!(seen.len(), units.len(), "n={n}: no unit twice");
            seen.sort();
            assert_eq!(seen, units, "n={n}: no unit missing");
        }
    }

    #[test]
    fn more_shards_than_units_leaves_some_shards_empty() {
        let units = plan_units(&fixture()).unwrap();
        assert!(select(&units, 7, 9).is_empty());
    }

    #[test]
    fn arguments_are_checked() {
        let a = |s: &[&str]| s.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        assert_eq!(
            parse_args(&a(&["2", "6"])).unwrap(),
            Args {
                k: 2,
                n: 6,
                dry_run: false
            }
        );
        assert!(parse_args(&a(&["0", "1", "--dry-run"])).unwrap().dry_run);
        for bad in [
            &["6", "6"][..],
            &["0", "0"],
            &["3", "2"],
            &["x", "6"],
            &["1", "y"],
            &["1"],
            &[],
            &["-1", "6"],
        ] {
            assert!(parse_args(&a(bad)).is_err(), "{bad:?}");
        }
    }
}
