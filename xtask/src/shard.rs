//! `test-shard <k> <n> [--dry-run]`: runs one of `n` disjoint slices of the workspace's tests, so CI can
//! spread `cargo test --workspace` over several runners and a developer can run the same slice locally.
//!
//! The unit of sharding is a workspace test target: each library, binary and integration-test file, plus one
//! `--doc` unit per library with doctests. `cargo metadata` lists them, the list is sorted by
//! (package, selector). Shards are numbered from 0, so `k` is in `0..n`. The `n` shards together run exactly
//! what `cargo test --workspace` runs.
//!
//! One target is too large for a single shard: `athena-core --lib` (821 tests). It is split into parts by
//! test-name module prefix (`CORE_LIB_PARTS`), each run as `cargo test --locked -p athena-core --lib -- <prefixes>`,
//! plus a catch-all part that runs `--skip <prefix>` for every listed prefix, so a test in a new module still
//! runs once. The filter is part of the unit's `--dry-run` line and of its key in the timings file.
//!
//! Balance: `xtask/shard-timings.txt` records how many seconds each unit takes. Timed units are sorted longest
//! first and each goes to the shard with the least total so far, so the slowest shard stays near the mean.
//! A unit with no recorded time (a new test file, say) falls back to the sorted-position rule: among the
//! untimed units, the one at position `i` goes to shard `i % n`. Either way each unit is in exactly one shard.
//! After adding or reshaping tests, refresh the file with `cargo run -p xtask -- shard-timings` (it runs every
//! unit, so it takes about as long as the whole suite) and commit the result.
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
use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub const USAGE: &str = "cargo run -p xtask -- test-shard <k> <n> [--dry-run]\n       \
     runs shard k (0-based, k < n) of n of the workspace's test targets, balanced by xtask/shard-timings.txt; --dry-run only lists them\n       \
     cargo run -p xtask -- shard-timings\n       \
     runs every test unit once and rewrites xtask/shard-timings.txt with the measured seconds";

/// A test-name filter that narrows one test binary to a part of its tests, so one large binary can be spread
/// over several shards. Test names are module paths such as `gh::tests::records_flow::x`. libtest keeps a test
/// whose name contains any `only` prefix (or every test, when `only` is empty) and drops a test whose name
/// contains any `skip` prefix. The parts of one binary must together match each test exactly once.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Filter {
    /// Name fragments to keep; empty means keep everything (the catch-all part).
    pub only: Vec<String>,
    /// Name fragments to drop, passed as `--skip <fragment>`.
    pub skip: Vec<String>,
}

impl Filter {
    /// The arguments after `--` in `cargo test ... -- <args>`; also what `--dry-run` prints after `--`.
    pub fn args(&self) -> Vec<String> {
        let mut args: Vec<String> = self.only.clone();
        for s in &self.skip {
            args.push("--skip".to_string());
            args.push(s.clone());
        }
        args
    }
}

/// The module prefixes of the parts `athena-core --lib` is split into. Each inner list is one part; a final
/// catch-all part takes every test none of them keep, so a new module is still run exactly once. Prefixes are
/// listed in sorted order and grouped so the parts have about the same number of tests (about 137 each, in
/// the 821-test run 41469096430). When a test module is added or renamed, the coverage test in
/// `xtask/tests/shard_parts.rs` shows whether every test still matches exactly one part.
pub const CORE_LIB_PARTS: &[&[&str]] = &[
    &["admission::", "gh::", "land::"],
    &["evidence_labels::", "reading::"],
    &["hestia::", "run::", "run_pin_tests::"],
    &[
        "disk::",
        "labeled::",
        "plan::",
        "routing::",
        "state::",
        "verify::",
    ],
    &[
        "checks::",
        "ci_result::",
        "code_health::",
        "scope_guard::",
        "status::",
        "steward::",
    ],
];

/// The package and selector `CORE_LIB_PARTS` applies to.
const SPLIT_PACKAGE: &str = "athena-core";
const SPLIT_SELECTOR: &str = "--lib";

/// The filters of the split binary: one per entry of `CORE_LIB_PARTS`, then the catch-all that skips them all.
pub fn core_lib_filters() -> Vec<Filter> {
    let mut filters: Vec<Filter> = CORE_LIB_PARTS
        .iter()
        .map(|part| Filter {
            only: part.iter().map(|p| p.to_string()).collect(),
            skip: Vec::new(),
        })
        .collect();
    filters.push(Filter {
        only: Vec::new(),
        skip: CORE_LIB_PARTS
            .iter()
            .flat_map(|part| part.iter().map(|p| p.to_string()))
            .collect(),
    });
    filters
}

/// One slice of `cargo test --workspace`: a package and the cargo selector for one of its test targets,
/// optionally narrowed to a part of that binary's tests.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Unit {
    pub package: String,
    /// `--lib`, `--doc`, `--bin <name>` or `--test <name>`.
    pub selector: String,
    /// `None` runs the whole target.
    pub filter: Option<Filter>,
}

impl Unit {
    /// A unit that runs a whole target.
    pub fn whole(package: &str, selector: &str) -> Unit {
        Unit {
            package: package.to_string(),
            selector: selector.to_string(),
            filter: None,
        }
    }

    /// The line `--dry-run` prints for this unit, and its key in the timings file. A filtered unit adds
    /// ` -- <filter arguments>`, exactly what follows `--` on the `cargo test` command line.
    pub fn line(&self) -> String {
        match &self.filter {
            None => format!("{} {}", self.package, self.selector),
            Some(f) => format!(
                "{} {} -- {}",
                self.package,
                self.selector,
                f.args().join(" ")
            ),
        }
    }

    /// The arguments after `cargo test --locked`: `-p <package> <selector> [-- <filter>]`.
    pub fn cargo_args(&self) -> Vec<String> {
        let mut args = vec!["-p".to_string(), self.package.clone()];
        args.extend(self.selector.split_whitespace().map(String::from));
        if let Some(f) = &self.filter {
            args.push("--".to_string());
            args.extend(f.args());
        }
        args
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
                if name == SPLIT_PACKAGE && selector == SPLIT_SELECTOR {
                    for filter in core_lib_filters() {
                        units.push(Unit {
                            filter: Some(filter),
                            ..Unit::whole(name, &selector)
                        });
                    }
                } else {
                    units.push(Unit::whole(name, &selector));
                }
            }
            if is_lib && target["doctest"].as_bool() == Some(true) {
                units.push(Unit::whole(name, "--doc"));
            }
        }
    }
    units.sort();
    units.dedup();
    Ok(units)
}

/// Recorded seconds per unit, keyed by `Unit::line()`.
pub type Timings = BTreeMap<String, f64>;

/// Where the recorded timings live, relative to the repository root.
pub const TIMINGS_FILE: &str = "xtask/shard-timings.txt";

/// Parses the timings file: one `<package> <selector> <seconds>` line per unit (the unit's `--dry-run` line,
/// then its duration). Blank lines and lines starting with `#` are skipped. A malformed line is an error that
/// names the line number and its text.
pub fn parse_timings(text: &str) -> Result<Timings> {
    let mut timings = Timings::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let bad = |why: &str| {
            anyhow!(
                "{TIMINGS_FILE} line {}: {why}: {raw:?} (expected `<package> <selector> <seconds>`)",
                i + 1
            )
        };
        let (unit, seconds) = line
            .rsplit_once(char::is_whitespace)
            .ok_or_else(|| bad("no duration"))?;
        let seconds: f64 = seconds
            .parse()
            .map_err(|_| bad("the duration is not a number"))?;
        if !seconds.is_finite() || seconds < 0.0 {
            return Err(bad("the duration must be a finite number of seconds >= 0"));
        }
        let key = unit.split_whitespace().collect::<Vec<_>>().join(" ");
        if !key.contains(' ') {
            return Err(bad("no selector"));
        }
        if timings.insert(key, seconds).is_some() {
            return Err(bad("the unit appears twice"));
        }
    }
    Ok(timings)
}

/// Splits `units` into `n` shards. Units with a recorded timing are sorted by descending duration (ties by unit
/// order) and each goes to the shard with the least total so far (ties to the lowest shard). Units with no
/// timing keep the old rule: sorted among themselves, the one at position `i` goes to shard `i % n`. Every unit
/// lands in exactly one shard; the result depends only on the inputs.
pub fn assign(units: &[Unit], timings: &Timings, n: u32) -> Vec<Vec<Unit>> {
    let n = n as usize;
    let mut shards: Vec<Vec<Unit>> = vec![Vec::new(); n];
    let mut load = vec![0.0f64; n];
    let (mut timed, mut untimed): (Vec<&Unit>, Vec<&Unit>) = units
        .iter()
        .partition(|unit| timings.contains_key(&unit.line()));
    untimed.sort();
    for (i, unit) in untimed.into_iter().enumerate() {
        shards[i % n].push(unit.clone());
    }
    timed.sort_by(|a, b| {
        timings[&b.line()]
            .total_cmp(&timings[&a.line()])
            .then_with(|| a.cmp(b))
    });
    for unit in timed {
        let lightest = (0..n)
            .min_by(|a, b| load[*a].total_cmp(&load[*b]).then(a.cmp(b)))
            .expect("n is at least 1");
        load[lightest] += timings[&unit.line()];
        shards[lightest].push(unit.clone());
    }
    for shard in &mut shards {
        shard.sort();
    }
    shards
}

/// The units of shard `k` of `n`, sorted.
pub fn select(units: &[Unit], timings: &Timings, k: u32, n: u32) -> Vec<Unit> {
    assign(units, timings, n).swap_remove(k as usize)
}

/// The timings file text for `timings`, with a header that says how to refresh it.
pub fn render_timings(timings: &Timings) -> String {
    let mut out = String::from(
        "# Seconds one `cargo test` run of each workspace test unit takes, used by `test-shard` to balance shards.\n\
         # Line format: <package> <selector> <seconds>. A unit missing here is still sharded, by sorted position.\n\
         # Refresh from a local run with: cargo run -p xtask -- shard-timings\n",
    );
    for (unit, seconds) in timings {
        out.push_str(&format!("{unit} {seconds:.2}\n"));
    }
    out
}

/// Reads the timings file under `root`; a missing file means no timings, so every unit falls back.
fn load_timings(root: &Path) -> Result<Timings> {
    match std::fs::read_to_string(root.join(TIMINGS_FILE)) {
        Ok(text) => parse_timings(&text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Timings::new()),
        Err(e) => Err(e).with_context(|| format!("could not read {TIMINGS_FILE}")),
    }
}

fn workspace_units(root: &Path) -> Result<Vec<Unit>> {
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
    plan_units(&metadata)
}

/// `shard-timings`: builds every test target, runs each unit alone, and rewrites the timings file with the
/// wall time of each run. Run it on a quiet machine. The file is written even if some units fail (the command then
/// exits with an error naming them), so a red unit never leaves the file short of a unit.
pub fn timings_command(root: &Path) -> Result<()> {
    let units = workspace_units(root)?;
    crate::step("shard-timings: building every test target");
    crate::run(
        Command::new("cargo")
            .args(["test", "--locked", "--workspace", "--no-run"])
            .current_dir(root),
    )?;
    let mut timings = Timings::new();
    let mut failed = Vec::new();
    for unit in &units {
        crate::step(&format!("shard-timings: {}", unit.line()));
        let start = Instant::now();
        let status = Command::new("cargo")
            .args(["test", "--locked"])
            .args(unit.cargo_args())
            .current_dir(root)
            .status()
            .context("could not run cargo test")?;
        if !status.success() {
            failed.push(unit.line());
        }
        timings.insert(unit.line(), start.elapsed().as_secs_f64());
    }
    std::fs::write(root.join(TIMINGS_FILE), render_timings(&timings))
        .with_context(|| format!("could not write {TIMINGS_FILE}"))?;
    println!("\nwrote {} timings to {TIMINGS_FILE}", timings.len());
    if failed.is_empty() {
        Ok(())
    } else {
        // The times are still written: a failing unit's time is a fair guide, and the file must name every unit.
        bail!(
            "{} units failed, so their times may be too short: {}",
            failed.len(),
            failed.join("; ")
        )
    }
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
    let timings = load_timings(root)?;
    let chosen = select(&workspace_units(root)?, &timings, k, n);
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
            .args(["test", "--locked"])
            .args(unit.cargo_args())
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

    fn no_timings() -> Timings {
        Timings::new()
    }

    #[test]
    fn select_is_disjoint_and_complete_for_any_n() {
        let units = plan_units(&fixture()).unwrap();
        for n in [1u32, 2, 3, 5, 6, 9] {
            let mut seen: Vec<Unit> = Vec::new();
            for k in 0..n {
                seen.extend(select(&units, &no_timings(), k, n));
            }
            assert_eq!(seen.len(), units.len(), "n={n}: no unit twice");
            seen.sort();
            assert_eq!(seen, units, "n={n}: no unit missing");
        }
    }

    #[test]
    fn without_timings_the_split_is_the_sorted_position_rule() {
        let units = plan_units(&fixture()).unwrap();
        let shard1 = select(&units, &no_timings(), 1, 2);
        assert_eq!(lines(&shard1), ["b-pkg --bin bee", "b-pkg --lib"]);
    }

    #[test]
    fn more_shards_than_units_leaves_some_shards_empty() {
        let units = plan_units(&fixture()).unwrap();
        assert!(select(&units, &no_timings(), 7, 9).is_empty());
    }

    /// 40 units with uneven fixture durations: a few long ones and a tail of short ones.
    fn many_units() -> (Vec<Unit>, Timings) {
        let mut units = Vec::new();
        let mut timings = Timings::new();
        for i in 0..40u32 {
            let unit = Unit::whole(&format!("pkg{:02}", i / 4), &format!("--test t{i:02}"));
            let seconds = match i % 10 {
                0 => 300.0,
                1 => 120.0,
                2 | 3 => 60.0,
                4..=6 => 20.0,
                _ => 5.0,
            } + f64::from(i);
            timings.insert(unit.line(), seconds);
            units.push(unit);
        }
        units.sort();
        (units, timings)
    }

    #[test]
    fn timed_shards_are_balanced_and_partition_the_units() {
        let (units, timings) = many_units();
        let n = 6u32;
        let mut seen = Vec::new();
        let mut totals = Vec::new();
        for k in 0..n {
            let shard = select(&units, &timings, k, n);
            totals.push(shard.iter().map(|u| timings[&u.line()]).sum::<f64>());
            seen.extend(shard);
        }
        assert_eq!(seen.len(), units.len(), "no unit twice");
        seen.sort();
        assert_eq!(seen, units, "no unit missing");
        let mean = totals.iter().sum::<f64>() / f64::from(n);
        let largest = totals.iter().cloned().fold(0.0, f64::max);
        assert!(
            largest <= 1.25 * mean,
            "largest {largest} vs mean {mean}: {totals:?}"
        );
    }

    #[test]
    fn the_longest_unit_goes_first_to_the_lowest_shard() {
        let (units, timings) = many_units();
        let longest = units
            .iter()
            .max_by(|a, b| timings[&a.line()].total_cmp(&timings[&b.line()]))
            .unwrap();
        assert!(select(&units, &timings, 0, 6).contains(longest));
    }

    #[test]
    fn a_unit_with_no_timing_is_still_assigned_exactly_once() {
        let (mut units, timings) = many_units();
        let extra = Unit::whole("new-pkg", "--test brand_new");
        units.push(extra.clone());
        units.sort();
        let shards = assign(&units, &timings, 6);
        let holding = shards.iter().filter(|s| s.contains(&extra)).count();
        assert_eq!(holding, 1);
        let mut all: Vec<Unit> = shards.into_iter().flatten().collect();
        all.sort();
        assert_eq!(all, units, "every unit is covered once");
    }

    #[test]
    fn assignment_is_deterministic() {
        let (units, timings) = many_units();
        assert_eq!(assign(&units, &timings, 6), assign(&units, &timings, 6));
    }

    #[test]
    fn timings_parse_and_round_trip() {
        let text = "# note\n\nxtask --test ci_shards 12.5\nathena-cli --bin athena 0.25\n";
        let timings = parse_timings(text).unwrap();
        assert_eq!(timings["xtask --test ci_shards"], 12.5);
        assert_eq!(timings["athena-cli --bin athena"], 0.25);
        assert_eq!(parse_timings(&render_timings(&timings)).unwrap(), timings);
    }

    /// A workspace with the split package, plus a plain unit.
    fn split_fixture() -> Value {
        json!({"packages": [
            {"name": "athena-core", "targets": [
                {"name": "athena_core", "kind": ["lib"], "test": true, "doctest": false},
                {"name": "refusal", "kind": ["test"], "test": true, "doctest": false}
            ]}
        ]})
    }

    fn core_parts(units: &[Unit]) -> Vec<Unit> {
        units
            .iter()
            .filter(|u| u.filter.is_some())
            .cloned()
            .collect()
    }

    #[test]
    fn the_core_lib_unit_is_expanded_into_parts_and_other_targets_are_not() {
        let units = plan_units(&split_fixture()).unwrap();
        assert_eq!(core_parts(&units).len(), CORE_LIB_PARTS.len() + 1);
        assert!(units.contains(&Unit::whole("athena-core", "--test refusal")));
        assert!(!units.contains(&Unit::whole("athena-core", "--lib")));
        // other packages' libraries stay whole
        let other = plan_units(&fixture()).unwrap();
        assert!(core_parts(&other).is_empty());
    }

    #[test]
    fn a_filtered_unit_prints_and_runs_its_filter() {
        let units = plan_units(&split_fixture()).unwrap();
        let parts = core_parts(&units);
        let catch_all = parts
            .iter()
            .find(|u| u.filter.as_ref().unwrap().only.is_empty())
            .expect("a catch-all part");
        assert!(
            catch_all
                .line()
                .starts_with("athena-core --lib -- --skip admission:: --skip gh::")
        );
        let args = catch_all.cargo_args();
        assert_eq!(&args[..4], ["-p", "athena-core", "--lib", "--"]);
        assert_eq!(args[4], "--skip");
        let kept = parts
            .iter()
            .find(|u| u.line() == "athena-core --lib -- admission:: gh:: land::")
            .expect("the first declared part");
        assert_eq!(
            kept.cargo_args(),
            [
                "-p",
                "athena-core",
                "--lib",
                "--",
                "admission::",
                "gh::",
                "land::"
            ]
        );
    }

    #[test]
    fn part_order_is_deterministic_and_sorted() {
        let a = plan_units(&split_fixture()).unwrap();
        let b = plan_units(&split_fixture()).unwrap();
        assert_eq!(a, b);
        let mut sorted = a.clone();
        sorted.sort();
        assert_eq!(a, sorted);
        let lines = lines(&a);
        let mut by_text = lines.clone();
        by_text.sort();
        assert_eq!(lines, by_text, "unit order matches line order");
    }

    #[test]
    fn assignment_keeps_every_part_exactly_once_and_spreads_them() {
        let units = plan_units(&split_fixture()).unwrap();
        let parts = core_parts(&units);
        let mut timings = Timings::new();
        for p in &parts {
            timings.insert(p.line(), 40.0);
        }
        for timed in [&timings, &no_timings()] {
            let shards = assign(&units, timed, 6);
            let mut all: Vec<Unit> = shards.iter().flatten().cloned().collect();
            all.sort();
            assert_eq!(all, units, "each unit once");
            let holding = shards
                .iter()
                .filter(|s| s.iter().any(|u| u.filter.is_some()))
                .count();
            assert!(holding >= 2, "parts land on {holding} shard(s)");
        }
    }

    #[test]
    fn timings_round_trip_with_a_filtered_selector() {
        let units = plan_units(&split_fixture()).unwrap();
        let mut timings = Timings::new();
        for (i, u) in units.iter().enumerate() {
            timings.insert(u.line(), 10.5 + i as f64);
        }
        let parsed = parse_timings(&render_timings(&timings)).unwrap();
        assert_eq!(parsed, timings);
        for u in &units {
            assert!(parsed.contains_key(&u.line()), "{}", u.line());
        }
    }

    #[test]
    fn a_malformed_timings_line_is_an_error_naming_the_line() {
        for bad in [
            "xtask --lib\n",
            "xtask --lib fast\n",
            "xtask --lib -3\n",
            "xtask --lib NaN\n",
            "lonely 4\n",
            "justone\n",
            "xtask --lib 1\nxtask --lib 2\n",
        ] {
            let err = parse_timings(bad).unwrap_err().to_string();
            assert!(err.contains("shard-timings.txt line"), "{bad:?}: {err}");
        }
        let err = parse_timings("# ok\nxtask --lib x\n").unwrap_err();
        assert!(err.to_string().contains("line 2"), "{err}");
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
