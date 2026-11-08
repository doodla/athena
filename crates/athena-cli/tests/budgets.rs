//! Every role's always-loaded set stays within its fixed budget (design §12; M18).
//!
//! The estimator, ⌈bytes / 3.0⌉, is a tripwire for prose; the opt-in live test measures the contract.

use athena_context::briefs::render_catalog;
use athena_context::{
    BudgetSet, CONTRACT, CRITIC_FRAME, Component, PLANNER_FRAME, PRODUCER_FRAME, READER_FRAME,
    VERIFIER_FRAME, check_budget, compose,
};
use athena_data::DefaultsModel;

fn defaults() -> DefaultsModel {
    DefaultsModel::parse(include_str!("../../../defaults.toml")).unwrap()
}

/// One always-loaded set: its name, its estimated size in tokens and its budget.
type Row = (String, u64, u64);

/// The sets `docs/system-facts.md` records, computed from the code. Producer and verifier are the largest
/// across the capabilities; the planner runs on the producer's budget and the critic on the verifier's.
fn computed_rows() -> Vec<Row> {
    let defaults = defaults();
    let cards: Vec<_> = athena_cli::wiring::capabilities()
        .iter()
        .map(|c| c.card())
        .collect();
    let size = |set: BudgetSet, composed| {
        (
            set.name().to_string(),
            check_budget(set, &composed).unwrap(),
            set.budget(),
        )
    };
    let reader = compose(&[
        CONTRACT.into(),
        READER_FRAME.into(),
        Component::new("defaults", defaults.version, defaults.render()),
        Component::new("catalog", 1, render_catalog(&cards)),
    ]);
    let mut producer: Option<Row> = None;
    let mut verifier: Option<Row> = None;
    for capability in athena_cli::wiring::capabilities() {
        let p = size(
            BudgetSet::Producer,
            compose(&[
                CONTRACT.into(),
                PRODUCER_FRAME.into(),
                Component::from(capability.producer_section()),
            ]),
        );
        let v = size(
            BudgetSet::Verifier,
            compose(&[
                CONTRACT.into(),
                VERIFIER_FRAME.into(),
                Component::from(capability.verifier_section()),
            ]),
        );
        producer = Some(producer.map_or(p.clone(), |old| if p.1 > old.1 { p } else { old }));
        verifier = Some(verifier.map_or(v.clone(), |old| if v.1 > old.1 { v } else { old }));
    }
    let mut planner = size(
        BudgetSet::Producer,
        compose(&[CONTRACT.into(), PLANNER_FRAME.into()]),
    );
    planner.0 = "planner".into();
    let mut critic = size(
        BudgetSet::Verifier,
        compose(&[CONTRACT.into(), CRITIC_FRAME.into()]),
    );
    critic.0 = "critic".into();
    vec![
        size(BudgetSet::Contract, compose(&[CONTRACT.into()])),
        size(BudgetSet::Reader, reader),
        producer.unwrap(),
        verifier.unwrap(),
        planner,
        critic,
    ]
}

/// Compares the `- budget <set>: <size> of <budget>` lines of the facts file with the computed rows. Every
/// computed set needs exactly one line, and a line for any other set is an error.
fn check_system_facts(text: &str, rows: &[Row]) -> Result<(), String> {
    let mut seen = Vec::new();
    for line in text
        .lines()
        .filter_map(|l| l.trim_start().strip_prefix("- budget "))
    {
        let (name, rest) = line
            .split_once(": ")
            .ok_or(format!("malformed budget line: {line}"))?;
        let (size, budget) = rest
            .split_once(" of ")
            .ok_or(format!("malformed budget line: {line}"))?;
        let parse = |s: &str| {
            s.trim()
                .parse::<u64>()
                .map_err(|_| format!("malformed budget line: {line}"))
        };
        let (size, budget) = (parse(size)?, parse(budget)?);
        let Some(row) = rows.iter().find(|r| r.0 == name) else {
            return Err(format!("the facts file records an unknown set: {name}"));
        };
        if seen.contains(&name) {
            return Err(format!("the facts file records {name} twice"));
        }
        seen.push(name);
        if (size, budget) != (row.1, row.2) {
            return Err(format!(
                "{name}: the facts file says {size} of {budget}, the computed value is {} of {}; update docs/system-facts.md",
                row.1, row.2
            ));
        }
    }
    for row in rows {
        if !seen.contains(&row.0.as_str()) {
            return Err(format!("the facts file has no budget line for {}", row.0));
        }
    }
    Ok(())
}

#[test]
fn system_facts_budgets_match_computed() {
    // The facts file the planner and critic read states the budgets and sizes the code computes.
    let result = check_system_facts(athena_context::briefs::SYSTEM_FACTS, &computed_rows());
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn system_facts_drift_is_rejected() {
    // Negative case: a doctored size, a doctored budget, a missing set and an unknown set are each refused.
    let rows = computed_rows();
    let real = athena_context::briefs::SYSTEM_FACTS;
    for (name, size, budget) in &rows {
        let line = format!("- budget {name}: {size} of {budget}");
        assert!(real.contains(&line), "the file has no line `{line}`");
        let wrong_size = real.replace(&line, &format!("- budget {name}: {} of {budget}", size + 1));
        let err = check_system_facts(&wrong_size, &rows).unwrap_err();
        assert!(err.contains(name.as_str()), "{err}");
        let wrong_budget =
            real.replace(&line, &format!("- budget {name}: {size} of {}", budget + 1));
        assert!(check_system_facts(&wrong_budget, &rows).is_err());
        let missing = real.replace(&line, "");
        assert!(check_system_facts(&missing, &rows).is_err());
    }
    let unknown = format!("{real}\n- budget steward: 1 of 600\n");
    assert!(check_system_facts(&unknown, &rows).is_err());
}

/// One workspace crate as its Cargo.toml files and source layout show it.
#[derive(Clone)]
struct CrateRow {
    /// The package name without the `athena-` prefix.
    name: String,
    /// `lib`, `bin` or `lib+bin`.
    target: &'static str,
    /// The workspace crates in `[dependencies]`, without the prefix.
    deps: Vec<String>,
    /// The workspace crates in `[dev-dependencies]` that `deps` does not hold, without the prefix.
    dev: Vec<String>,
}

/// The names of the `athena-*` crates a manifest section lists, without the prefix, sorted.
fn workspace_crates(manifest: &toml::Table, section: &str) -> Vec<String> {
    let mut names: Vec<String> = manifest
        .get(section)
        .and_then(|v| v.as_table())
        .map(|table| {
            table
                .keys()
                .filter_map(|key| key.strip_prefix("athena-"))
                .map(String::from)
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// The workspace crates, computed from the root `[workspace] members`, each member's Cargo.toml and its
/// `src` layout. Cargo.toml is the oracle for `docs/system-facts.md`.
fn crate_rows() -> Vec<CrateRow> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |dir: &std::path::Path| -> toml::Table {
        let path = dir.join("Cargo.toml");
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path:?}: {e}"));
        toml::from_str(&text).unwrap_or_else(|e| panic!("{path:?}: {e}"))
    };
    let workspace = read(&root);
    let members = workspace["workspace"]["members"].as_array().unwrap();
    members
        .iter()
        .map(|member| {
            let dir = root.join(member.as_str().unwrap());
            let manifest = read(&dir);
            let package = manifest["package"]["name"].as_str().unwrap();
            let lib = dir.join("src/lib.rs").is_file() || manifest.contains_key("lib");
            let bin = manifest.contains_key("bin")
                || dir.join("src/main.rs").is_file()
                || dir.join("src/bin").is_dir();
            let target = match (lib, bin) {
                (true, true) => "lib+bin",
                (true, false) => "lib",
                (false, true) => "bin",
                (false, false) => panic!("{package} has no library or binary target"),
            };
            let deps = workspace_crates(&manifest, "dependencies");
            let dev = workspace_crates(&manifest, "dev-dependencies")
                .into_iter()
                .filter(|name| !deps.contains(name))
                .collect();
            CrateRow {
                name: package
                    .strip_prefix("athena-")
                    .unwrap_or(package)
                    .to_string(),
                target,
                deps,
                dev,
            }
        })
        .collect()
}

/// The facts line for a crate: `- crate <name>: <target>[; deps <names>][; dev <names>]`.
fn crate_line(row: &CrateRow) -> String {
    let mut line = format!("- crate {}: {}", row.name, row.target);
    if !row.deps.is_empty() {
        line.push_str(&format!("; deps {}", row.deps.join(" ")));
    }
    if !row.dev.is_empty() {
        line.push_str(&format!("; dev {}", row.dev.join(" ")));
    }
    line
}

/// Compares the `- crate ...` lines of the facts file with the lines computed from the manifests. Every crate
/// needs exactly one line equal to its computed line; any other `- crate ` line is an error.
fn check_crate_facts(text: &str, rows: &[CrateRow]) -> Result<(), String> {
    let mut seen: Vec<&str> = Vec::new();
    for line in text
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("- crate "))
    {
        let name = line["- crate ".len()..].split(':').next().unwrap_or("");
        let Some(row) = rows.iter().find(|r| r.name == name) else {
            return Err(format!(
                "the facts file has a line for an unknown crate: {line}"
            ));
        };
        let expected = crate_line(row);
        if line != expected {
            return Err(format!(
                "{name}: the facts file says `{line}`, the Cargo.toml files give `{expected}`; update docs/system-facts.md"
            ));
        }
        if seen.contains(&line) {
            return Err(format!("the facts file records crate {name} twice"));
        }
        seen.push(line);
    }
    for row in rows {
        if !seen.contains(&crate_line(row).as_str()) {
            return Err(format!("the facts file has no crate line for {}", row.name));
        }
    }
    Ok(())
}

#[test]
fn system_facts_crate_lines_match_manifests() {
    // The crate facts the planner and critic read state what the Cargo.toml files show.
    let rows = crate_rows();
    let result = check_crate_facts(athena_context::briefs::SYSTEM_FACTS, &rows);
    assert!(result.is_ok(), "{}", result.unwrap_err());
    // xtask has no library target and cli is the only crate with a binary next to its library.
    let target = |name: &str| rows.iter().find(|r| r.name == name).unwrap().target;
    assert_eq!(target("xtask"), "bin");
    assert_eq!(target("cli"), "lib+bin");
    assert!(
        rows.iter()
            .filter(|r| r.name != "xtask" && r.name != "cli")
            .all(|r| r.target == "lib")
    );
}

#[test]
fn system_facts_crate_drift_is_rejected() {
    // Negative case: a doctored line, a missing line, an unknown crate, a repeated line and a malformed line
    // are each refused.
    let rows = crate_rows();
    let real = athena_context::briefs::SYSTEM_FACTS;
    let swapped = |row: &CrateRow, edit: &dyn Fn(&mut CrateRow)| {
        let mut doctored = row.clone();
        edit(&mut doctored);
        real.replace(&crate_line(row), &crate_line(&doctored))
    };
    for row in &rows {
        let line = crate_line(row);
        assert!(real.contains(&line), "the file has no line `{line}`");
        let flipped = swapped(row, &|r| {
            r.target = if r.target == "lib" { "bin" } else { "lib" };
        });
        let err = check_crate_facts(&flipped, &rows).unwrap_err();
        assert!(err.contains(row.name.as_str()), "{err}");
        let more_deps = swapped(row, &|r| r.deps.push("steward".into()));
        assert!(check_crate_facts(&more_deps, &rows).is_err());
        let fewer_deps = swapped(row, &|r| {
            r.deps.pop();
        });
        assert_eq!(
            check_crate_facts(&fewer_deps, &rows).is_err(),
            !row.deps.is_empty()
        );
        let more_dev = swapped(row, &|r| r.dev.push("steward".into()));
        assert!(check_crate_facts(&more_dev, &rows).is_err());
        let fewer_dev = swapped(row, &|r| {
            r.dev.pop();
        });
        assert_eq!(
            check_crate_facts(&fewer_dev, &rows).is_err(),
            !row.dev.is_empty()
        );
        let missing = real.replace(&line, "");
        let err = check_crate_facts(&missing, &rows).unwrap_err();
        assert!(err.contains(row.name.as_str()), "{err}");
        let twice = format!("{real}\n  {line}\n");
        assert!(check_crate_facts(&twice, &rows).is_err());
    }
    let unknown = format!("{real}\n  - crate steward: lib\n");
    assert!(check_crate_facts(&unknown, &rows).is_err());
    let malformed = format!("{real}\n  - crate steward\n");
    assert!(check_crate_facts(&malformed, &rows).is_err());
}

#[test]
fn every_role_set_is_within_its_budget() {
    // M18 (allows): the current sets pass.
    let defaults = defaults();
    let cards: Vec<_> = athena_cli::wiring::capabilities()
        .iter()
        .map(|c| c.card())
        .collect();
    let reader = compose(&[
        CONTRACT.into(),
        READER_FRAME.into(),
        Component::new("defaults", defaults.version, defaults.render()),
        Component::new("catalog", 1, render_catalog(&cards)),
    ]);
    let mut report = vec![format!(
        "reader {}",
        check_budget(BudgetSet::Reader, &reader).unwrap()
    )];
    for capability in athena_cli::wiring::capabilities() {
        let producer = compose(&[
            CONTRACT.into(),
            PRODUCER_FRAME.into(),
            Component::from(capability.producer_section()),
        ]);
        let verifier = compose(&[
            CONTRACT.into(),
            VERIFIER_FRAME.into(),
            Component::from(capability.verifier_section()),
        ]);
        report.push(format!(
            "{} producer {}",
            capability.id(),
            check_budget(BudgetSet::Producer, &producer).unwrap()
        ));
        report.push(format!(
            "{} verifier {}",
            capability.id(),
            check_budget(BudgetSet::Verifier, &verifier).unwrap()
        ));
    }
    // Planning roles reuse the producer's and verifier's budgets: the planner runs on the producer's model
    // and the critic on the verifier's.
    let planner = compose(&[CONTRACT.into(), PLANNER_FRAME.into()]);
    let critic = compose(&[CONTRACT.into(), CRITIC_FRAME.into()]);
    report.push(format!(
        "planner {}, critic {}",
        check_budget(BudgetSet::Producer, &planner).unwrap(),
        check_budget(BudgetSet::Verifier, &critic).unwrap()
    ));
    report.push(format!(
        "contract {}",
        check_budget(BudgetSet::Contract, &compose(&[CONTRACT.into()])).unwrap()
    ));
    println!("estimated tokens: {}", report.join(", "));
}

#[test]
fn the_reader_headroom_rows_equal_the_budget_figures_and_name_shared_files() {
    // The rows the reader's brief shows come from the same compose and budgets as `computed_rows`.
    let defaults = defaults();
    let capabilities = athena_cli::wiring::capabilities();
    let cards: Vec<_> = capabilities.iter().map(|c| c.card()).collect();
    let sections: Vec<_> = capabilities
        .iter()
        .map(|c| {
            (
                c.id().to_string(),
                c.producer_section(),
                c.verifier_section(),
            )
        })
        .collect();
    let headroom = athena_context::headroom_rows(
        Component::new("defaults", defaults.version, defaults.render()),
        Component::new("catalog", 1, render_catalog(&cards)),
        &sections,
    );
    for (name, size, budget) in computed_rows() {
        // The producer and verifier rows are the largest across capabilities.
        let mine: Vec<_> = headroom
            .iter()
            .filter(|r| r.set == name || r.set.starts_with(&format!("{name} (")))
            .collect();
        assert!(!mine.is_empty(), "no headroom row for {name}");
        let used = mine.iter().map(|r| r.used).max().unwrap();
        assert_eq!((used, mine[0].budget), (size, budget), "{name}");
        for r in mine {
            assert_eq!(r.left(), r.budget - r.used, "{}", r.set);
        }
    }
    // The shared verifier frame is listed in the verifier set of every capability.
    for capability in &capabilities {
        let row = headroom
            .iter()
            .find(|r| r.set == format!("verifier ({})", capability.id()))
            .unwrap();
        assert!(row.components.contains(&VERIFIER_FRAME.id.to_string()));
        assert!(
            row.components
                .contains(&capability.verifier_section().id.to_string())
        );
    }
    let frame_rows = headroom
        .iter()
        .filter(|r| r.components.contains(&VERIFIER_FRAME.id.to_string()))
        .count();
    assert_eq!(frame_rows, capabilities.len());
}

#[test]
fn an_over_budget_role_set_fails() {
    // M18 (blocks): an over-budget role set fails. Moving rules between components escapes nothing.
    let padding =
        "A rule that was moved here from the contract instead of being paid for. ".repeat(40);
    let producer = compose(&[
        CONTRACT.into(),
        PRODUCER_FRAME.into(),
        Component::new("capability.x.producer", 1, padding),
    ]);
    assert!(check_budget(BudgetSet::Producer, &producer).is_err());
}
