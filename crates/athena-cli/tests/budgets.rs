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
