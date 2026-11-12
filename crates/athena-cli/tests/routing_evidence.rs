//! Requirement OLY-B-NOTE-01: the starting model routing is revised from evidence. `athena status` prints a
//! `specialization` block that compares producer model and effort per class of work, and flags a setting that
//! lands more of its jobs than the most-used one once it has enough jobs. Records are seeded in a temporary
//! repository; no model is called.

mod common;

use athena_core::status::SPECIALIZATION_MIN_JOBS;
use athena_data::records::{
    AnyRecord, Build, DriverInfo, Invocation, InvocationCaps, InvocationContext, InvocationOutcome,
    Outcome, OutcomeState, Role, Tokens, Totals,
};
use athena_data::{JobId, Quantity, Unit};
use common::*;
use std::collections::BTreeMap;

fn job_id(n: u32) -> JobId {
    JobId::parse(&format!("20261110-1200{n:02}-r{n:03}")).unwrap()
}

fn job_record(job: &JobId, class: &str) -> AnyRecord {
    let job: athena_data::records::Job = serde_json::from_value(serde_json::json!({
        "id": job.as_str(), "kind": "request", "title": "t", "class_of_work": class,
        "capability": null, "accepted_reading": null, "accepted_by": null,
        "owner": { "role": "producer", "capability": null, "model": null, "note": null },
        "verifier_model": null, "caps": null, "base_commit": null, "config_sha256": null,
        "defaults_sha256": null, "athena": null, "driver": null, "brief_sha256": null,
        "dispatched_at": "2026-11-10T12:00:00Z", "request_path": null, "request_sha256": null,
        "cost": null, "tokens": null, "note": null, "pin": null
    }))
    .unwrap();
    AnyRecord::Job(Box::new(job))
}

fn producer_run(job: &JobId, effort: &str) -> AnyRecord {
    let tokens = |v: f64| Quantity::measured(v, Unit::Tokens, "result.usage");
    AnyRecord::Invocation(Box::new(Invocation {
        job: job.clone(),
        seq: 1,
        role: Role::Producer,
        driver: DriverInfo {
            name: "claude-code".into(),
            version: "2.1.299".into(),
        },
        athena: Build {
            version: "0.1.0".into(),
            source_commit: "0".repeat(40),
            dirty: false,
        },
        model: "m".into(),
        effort: Some(effort.into()),
        context: InvocationContext {
            components: vec![],
            prompt_sha256: "0".repeat(64),
            prompt_tokens: tokens(1.0),
            host_baseline: tokens(600.0),
            tools: vec![],
        },
        caps: InvocationCaps {
            max_budget_usd: 1.0,
            timeout_s: 600,
        },
        started_at: "2026-11-10T12:00:00Z".parse().unwrap(),
        ended_at: "2026-11-10T12:05:00Z".parse().unwrap(),
        wall_time: Quantity::measured(60.0, Unit::Seconds, "athena clock"),
        tokens: Tokens {
            input: tokens(1.0),
            output: tokens(1.0),
            cache_read: tokens(1.0),
            cache_write: tokens(1.0),
        },
        models_used: BTreeMap::new(),
        cost: Quantity::reported(0.5, Unit::Usd, "Claude Code total_cost_usd"),
        outcome: InvocationOutcome::Success,
        detail: None,
        turns: None,
        commands_run: vec![],
        policy_denials: vec![],
        reading_attempt: None,
    }))
}

fn outcome(job: &JobId, state: OutcomeState) -> AnyRecord {
    AnyRecord::Outcome(Box::new(Outcome {
        job: job.clone(),
        state,
        reason: "r".into(),
        at: "2026-11-10T13:00:00Z".parse().unwrap(),
        totals: Some(Totals {
            cost_all: Quantity::reported(1.5, Unit::Usd, "Claude Code total_cost_usd"),
            cost_execution: Quantity::reported(1.0, Unit::Usd, "Claude Code total_cost_usd"),
            wall_time_execution: None,
            tokens: BTreeMap::new(),
            invocations: 1,
        }),
        branch: None,
        base_commit: None,
        candidate_commit: None,
        after_blocked: false,
        through_commit: None,
    }))
}

/// Seeds one class "code": four `medium` jobs that all fail, and `high_jobs` `high` jobs that all land.
fn seed(fx: &Fixture, high_jobs: u32) {
    let mut records = Vec::new();
    let mut n = 0;
    let mut add = |effort: &str, state: OutcomeState| {
        n += 1;
        let job = job_id(n);
        records.push(job_record(&job, "code"));
        records.push(producer_run(&job, effort));
        records.push(outcome(&job, state));
    };
    for _ in 0..4 {
        add("medium", OutcomeState::Failed);
    }
    for _ in 0..high_jobs {
        add("high", OutcomeState::Landed);
    }
    let files: Vec<(String, String)> = records
        .iter()
        .map(|r| (r.canonical_path(), r.to_json()))
        .collect();
    let refs: Vec<(&str, &str)> = files
        .iter()
        .map(|(p, c)| (p.as_str(), c.as_str()))
        .collect();
    write(fx.root(), &refs);
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run(["commit", "-q", "-m", "Add fixture records"])
        .unwrap();
}

#[test]
fn status_flags_a_producer_setting_that_lands_more_once_it_has_enough_jobs() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let none = athena_cli::display::status(&athena.status().unwrap());
    assert!(!none.contains("specialization"), "{none}");

    seed(&fx, SPECIALIZATION_MIN_JOBS as u32);
    let view = athena.status().unwrap();
    assert_eq!(view.specialization.len(), 1);
    let text = athena_cli::display::status(&view);
    assert!(
        text.lines().any(|l| l.starts_with("specialization ")),
        "no specialization line in:\n{text}"
    );
    assert!(
        text.contains("code: specialization helps: m · high beat the default"),
        "{text}"
    );
}

#[test]
fn status_does_not_flag_a_setting_with_too_few_jobs() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    seed(&fx, SPECIALIZATION_MIN_JOBS as u32 - 1);
    let text = athena_cli::display::status(&athena.status().unwrap());
    assert!(
        text.lines().any(|l| l.starts_with("specialization ")),
        "{text}"
    );
    assert!(!text.contains("specialization helps"), "{text}");
    assert!(text.contains("code: not enough jobs"), "{text}");
}
