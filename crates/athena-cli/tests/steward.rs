//! The steward against the fake agent: when it starts, what it queues, and what the ladder lets it change.

// Scenarios are set up field by field, so each test shows only what differs from the default.
#![allow(clippy::field_reassign_with_default)]

mod common;

use athena_core::commits::Committer;
use athena_core::run::GoResult;
use athena_core::steward::{self, GitView, RECORDS_SOURCE, RequestQueue, View};
use athena_core::store::Records;
use athena_data::records::{AcceptedBy, OutcomeState, RefKind, WorkItemKind};
use athena_data::roles::{self, Ladder};
use common::*;

/// Runs one job whose change deletes the README, so a check fails and the job is rejected.
fn go_one(athena: &athena_core::Athena, ui: &Lines) {
    let proposal = athena.ask("change the readme", ui).unwrap();
    match athena
        .go(proposal.job.as_str(), AcceptedBy::User, ui)
        .unwrap()
    {
        GoResult::Finished(f) => assert_eq!(f.state, OutcomeState::Rejected, "{}", f.reason),
        GoResult::Reread(_) => panic!("expected the job to finish"),
    }
}

fn records(fx: &Fixture) -> Records {
    Records::load(&fx.git, &fx.head()).unwrap()
}

fn steward_lines(ui: &Lines) -> Vec<String> {
    ui.0.lock()
        .unwrap()
        .iter()
        .filter(|l| l.starts_with("steward"))
        .cloned()
        .collect()
}

#[test]
fn the_fifth_finished_task_starts_the_steward_and_the_fourth_does_not() {
    // Every job here is rejected (a rejection counts as finished, as a landing does).
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.edits = vec![("README.md".into(), None)];
    let athena = fx.athena(scenario.script());
    let ui = Lines::default();
    // Three tasks have already finished; this one is the fourth.
    for _ in 0..3 {
        assert!(!steward::tick(&athena.state, 5).unwrap());
    }
    go_one(&athena, &ui);
    assert_eq!(steward::finished_since_run(&athena.state), 4);
    assert!(steward_lines(&ui).is_empty(), "the fourth starts nothing");
    steward::wait_for_background();
    assert!(
        records(&fx).roles.is_empty(),
        "and the steward has recorded nothing"
    );

    // The fifth starts it, in the background: go has returned a finished job by the time it is counted.
    go_one(&athena, &ui);
    assert_eq!(steward_lines(&ui).len(), 1, "{:?}", steward_lines(&ui));
    assert_eq!(steward::finished_since_run(&athena.state), 0);
    steward::wait_for_background();
    let after = records(&fx);
    assert_eq!(after.roles.len(), 1, "the steward recorded its definition");
    assert_eq!(after.roles[0].name, "steward");
    assert_eq!(after.roles[0].cadence_tasks, 5);
}

/// A view whose records cannot be read.
struct Unreadable;

impl View for Unreadable {
    fn records(&self) -> anyhow::Result<Records> {
        anyhow::bail!("not readable")
    }
}

#[test]
fn an_unreadable_source_becomes_a_blind_spot_work_item() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let role = roles::steward(athena_core::now());
    let committer = Committer {
        git: &athena.git,
        state: &athena.state,
    };
    let report = steward::run(
        &committer,
        &Unreadable,
        &RequestQueue,
        &role,
        athena_core::now(),
    )
    .unwrap();
    assert_eq!(report.queued, vec!["blind-spot:records/".to_string()]);
    let after = records(&fx);
    assert_eq!(after.work_items.len(), 1);
    let item = &after.work_items[0];
    assert_eq!(item.kind, WorkItemKind::BlindSpot);
    assert_eq!(item.source.kind, RefKind::Path);
    assert_eq!(item.source.value, RECORDS_SOURCE);
    assert_eq!(item.role, "steward");
}

#[test]
fn repeated_failures_are_proposed_on_a_rung_the_steward_may_change_and_queued_once() {
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.edits = vec![("README.md".into(), None)];
    let athena = fx.athena(scenario.script());
    let ui = Lines::default();
    go_one(&athena, &ui);
    go_one(&athena, &ui);

    let role = roles::steward(athena_core::now());
    let committer = Committer {
        git: &athena.git,
        state: &athena.state,
    };
    let view = GitView { git: &athena.git };
    let report = steward::run(&committer, &view, &RequestQueue, &role, athena_core::now()).unwrap();
    assert_eq!(report.queued.len(), 1, "{report:?}");
    assert_eq!(report.applied.len(), 1, "a lower rung is applied");

    let after = records(&fx);
    let item = &after.work_items[0];
    assert_eq!(item.kind, WorkItemKind::RepeatedFailure);
    assert_eq!(item.rung, Ladder::DefaultsConventions);
    assert_eq!(
        item.evidence
            .iter()
            .filter(|e| e.kind == RefKind::Job)
            .count(),
        3,
        "the two failed jobs and the request queued for them"
    );
    let queued = &report.applied[0];
    let job = after.jobs.get(queued.as_str()).expect("the request exists");
    assert!(job.request.as_ref().unwrap().text.contains("keep ending"));
    assert_eq!(job.request.as_ref().unwrap().channel, "steward");

    // Nothing new has happened, so a second run adds nothing.
    let again = steward::run(&committer, &view, &RequestQueue, &role, athena_core::now()).unwrap();
    assert_eq!(again, steward::Report::default());
    assert_eq!(records(&fx).work_items.len(), 1);
}
