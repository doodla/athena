//! `athena status` prints the `role need` line (requirement OLY-9-11) only when a role is a retire candidate,
//! on records seeded in a temporary repository. The seeded records hold the steward, audience and auditor runs
//! outside any job, as the real records do; those roles must never be advised for retirement.

mod common;

use athena_data::records::{
    AnyRecord, Build, DriverInfo, Invocation, InvocationCaps, InvocationContext, InvocationOutcome,
    Outcome, OutcomeState, Role, Tokens, Totals,
};
use athena_data::{JobId, Quantity, Unit};
use common::*;
use std::collections::BTreeMap;

fn job_id(n: u32) -> JobId {
    JobId::parse(&format!("202611{n:02}-120000-k{n:03}")).unwrap()
}

fn invocation(job: &JobId, seq: u32, role: Role) -> AnyRecord {
    let tokens = |v: f64| Quantity::measured(v, Unit::Tokens, "result.usage");
    AnyRecord::Invocation(Box::new(Invocation {
        job: job.clone(),
        seq,
        role,
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
        effort: None,
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
        started_at: format!("2026-11-{:02}T12:00:00Z", job_number(job))
            .parse()
            .unwrap(),
        ended_at: format!("2026-11-{:02}T12:05:00Z", job_number(job))
            .parse()
            .unwrap(),
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

fn job_number(job: &JobId) -> u32 {
    job.as_str()[6..8].parse().unwrap()
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
            invocations: 3,
        }),
        branch: None,
        base_commit: None,
        candidate_commit: None,
        after_blocked: false,
        through_commit: None,
    }))
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

fn role_need_line(text: &str) -> Option<String> {
    text.lines()
        .find(|l| l.starts_with("role need"))
        .map(str::to_string)
}

#[test]
fn status_prints_the_role_need_line_only_when_a_role_is_a_retire_candidate() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());

    // No records: nothing to measure, nothing printed.
    let view = athena.status().unwrap();
    assert_eq!(role_need_line(&athena_cli::display::status(&view)), None);

    // Nine closed jobs that ran only reader, producer and verifier, with the failed, rejected and timed-out
    // outcomes among them; the steward, audience and auditor ran outside any job.
    let states = [
        OutcomeState::Landed,
        OutcomeState::Landed,
        OutcomeState::Rejected,
        OutcomeState::Failed,
        OutcomeState::TimedOut,
        OutcomeState::Landed,
        OutcomeState::Landed,
        OutcomeState::Landed,
        OutcomeState::Landed,
    ];
    let mut records = Vec::new();
    for (k, state) in states.iter().enumerate() {
        let job = job_id(k as u32 + 1);
        records.push(invocation(&job, 1, Role::Reader));
        records.push(invocation(&job, 2, Role::Producer));
        records.push(invocation(&job, 3, Role::Verifier));
        records.push(outcome(&job, *state));
    }
    records.push(invocation(&job_id(20), 1, Role::Steward));
    records.push(invocation(&job_id(21), 1, Role::Audience));
    records.push(invocation(&job_id(22), 1, Role::Auditor));
    seed(&fx, &records);

    let view = athena.status().unwrap();
    let text = athena_cli::display::status(&view);
    let line = role_need_line(&text).unwrap_or_else(|| panic!("no role need line in:\n{text}"));
    assert!(
        line.contains("retire candidates: planner, critic "),
        "{line}"
    );
    for known in ["steward", "audience", "auditor"] {
        let candidates = line.split("(no runs").next().unwrap();
        assert!(!candidates.contains(known), "{known} flagged: {line}");
    }
    assert!(
        line.contains("not measured, run on demand: steward (1 runs"),
        "{line}"
    );
    assert!(line.contains("auditor (1 runs"), "{line}");
    // The replay lists each role with its count and data source.
    assert!(
        text.contains("steward: 1 (runs outside job records"),
        "{text}"
    );
    assert!(
        text.contains("producer: 8 (runs in the window's jobs)"),
        "{text}"
    );
    assert!(
        text.contains("planner: 0 (runs in the window's jobs) [retire candidate]"),
        "{text}"
    );
    // The newest eight (jobs 2 to 9) hold the rejected, failed and timed-out jobs.
    assert!(line.contains("3 of them failed"), "{line}");

    // The planner and critic run in the newest job: no candidate, no line.
    let newest = job_id(9);
    seed(
        &fx,
        &[
            invocation(&newest, 4, Role::Planner),
            invocation(&newest, 5, Role::Critic),
        ],
    );
    let view = athena.status().unwrap();
    assert_eq!(role_need_line(&athena_cli::display::status(&view)), None);
}
