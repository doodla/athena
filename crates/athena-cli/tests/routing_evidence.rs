//! Requirement OLY-B-NOTE-01: the starting model routing is revised from evidence. `athena status` and
//! `athena status --snapshot` print the `specialization` block: per class of work, each producer model and
//! effort against the most-used one, flagged as helping only from SPECIALIZATION_MIN_JOBS jobs.

mod common;

use athena_core::snapshot;
use athena_core::status::SPECIALIZATION_MIN_JOBS;
use athena_data::records::{
    AnyRecord, Build, DriverInfo, Invocation, InvocationCaps, InvocationContext, InvocationOutcome,
    Job, Outcome, OutcomeState, Role, Tokens, Totals,
};
use athena_data::{JobId, Quantity, Unit};
use common::*;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Unlocks a snapshot when the test ends, so its temporary directory can be removed.
struct Unlock(PathBuf);
impl Drop for Unlock {
    fn drop(&mut self) {
        let _ = snapshot::unlock(&self.0);
    }
}

fn job_id(n: u32) -> JobId {
    JobId::parse(&format!("202611{n:02}-120000-k{n:03}")).unwrap()
}

fn job_record(job: &JobId, class: &str) -> AnyRecord {
    let job: Job = serde_json::from_value(serde_json::json!({
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

fn producer(job: &JobId, model: &str, effort: &str) -> AnyRecord {
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
        model: model.into(),
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
        at: "2026-11-10T12:00:00Z".parse().unwrap(),
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

/// Records for class "code": four jobs on `medium` (the most used) of which one landed, and `high_jobs` jobs
/// on `high`, all landed.
fn fixture_records(high_jobs: u32) -> Vec<AnyRecord> {
    let mut records = Vec::new();
    let mut add = |n: u32, effort: &str, state: OutcomeState| {
        let job = job_id(n);
        records.push(job_record(&job, "code"));
        records.push(producer(&job, "m", effort));
        records.push(outcome(&job, state));
    };
    for n in 1..=3 {
        add(n, "medium", OutcomeState::Failed);
    }
    add(4, "medium", OutcomeState::Landed);
    for k in 0..high_jobs {
        add(5 + k, "high", OutcomeState::Landed);
    }
    records
}

fn seed(fx: &Fixture, records: &[AnyRecord]) {
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

const HELPS: &str = "code: specialization helps: m · high beat the default";

#[test]
fn status_flags_a_setting_with_enough_jobs_and_a_higher_landed_share_as_helping() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    seed(&fx, &fixture_records(SPECIALIZATION_MIN_JOBS as u32));
    let text = athena_cli::display::status(&athena.status().unwrap());
    assert!(text.contains("\nspecialization   per class"), "{text}");
    assert!(text.contains(HELPS), "{text}");
    assert!(text.contains("m · high: 3 jobs"), "{text}");
    assert!(text.contains("m · medium: 4 jobs"), "{text}");
}

#[test]
fn status_does_not_flag_a_setting_with_fewer_jobs_than_the_minimum() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    seed(&fx, &fixture_records(SPECIALIZATION_MIN_JOBS as u32 - 1));
    let text = athena_cli::display::status(&athena.status().unwrap());
    assert!(text.contains("\nspecialization   per class"), "{text}");
    assert!(!text.contains("specialization helps"), "{text}");
    assert!(text.contains("code: not enough jobs"), "{text}");
}

#[test]
fn status_snapshot_prints_the_specialization_block_from_the_snapshot_records() {
    let fx = Fixture::new();
    seed(&fx, &fixture_records(SPECIALIZATION_MIN_JOBS as u32));
    let out = tempfile::tempdir().unwrap();
    let dir = out.path().join("evidence");
    snapshot::write(&fx.git, &fx.head(), &dir).unwrap();
    let _guard = Unlock(dir.clone());

    let view = snapshot::status(&dir).unwrap();
    let text = athena_cli::display::snapshot_status(&view);
    assert!(text.contains("\nspecialization   per class"), "{text}");
    assert!(text.contains(HELPS), "{text}");
    assert!(text.contains("m · medium: 4 jobs"), "{text}");
}
