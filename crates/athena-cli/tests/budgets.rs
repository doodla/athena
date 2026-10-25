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
