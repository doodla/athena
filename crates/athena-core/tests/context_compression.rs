//! OLY-B-A13-01: context compression is measured. Athena reports tokens loaded per landed job for each class of
//! work, beside the share of the class's closed jobs that landed and its recorded mistakes.
//!
//! The mechanism is `crates/athena-core/src/context_load.rs`, shown by `athena status` as the `context` line.
//! These tests write record files and load them with `Records::load_dir`, the loader `athena status` uses, so a
//! record shape the loader rejects would show in `records.invalid`. Limits: the figures are checked on fixtures;
//! no run on the real records is part of this test, and whether a smaller context holds outcomes is not judged.

use std::collections::BTreeMap;
use std::path::Path;

use athena_core::context_load::ContextLoad;
use athena_core::hestia::{SpecRequirement, measure_problem, parse_spec, spec_gaps};
use athena_core::status::STATUS_MEASURE_LABELS;
use athena_core::store::Records;
use athena_data::records::{
    AnyRecord, Build, DriverInfo, Intervention, InterventionCategory, InterventionSource,
    Invocation, InvocationCaps, InvocationContext, InvocationOutcome, Job, JobKind, ModelUsage,
    Outcome, OutcomeState, Owner, Role, Tokens, Totals,
};
use athena_data::{JobId, Quantity, RecordId, Unit};

const REQUIREMENT: &str = "OLY-B-A13-01";

fn repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn real_requirement() -> SpecRequirement {
    parse_spec(&repo_file("docs/spec.toml"))
        .expect("docs/spec.toml parses")
        .into_iter()
        .find(|r| r.id == REQUIREMENT)
        .expect("docs/spec.toml has OLY-B-A13-01")
}

fn at() -> jiff::Timestamp {
    "2026-10-20T10:00:00Z".parse().unwrap()
}

fn job_record(id: &str, class: &str) -> AnyRecord {
    AnyRecord::Job(Box::new(Job {
        id: JobId::parse(id).unwrap(),
        kind: JobKind::Request,
        title: "A job".into(),
        class_of_work: Some(class.into()),
        capability: None,
        accepted_reading: None,
        accepted_by: None,
        owner: Owner {
            role: "athena".into(),
            capability: None,
            model: None,
            note: None,
        },
        verifier_model: None,
        caps: None,
        base_commit: None,
        config_sha256: None,
        defaults_sha256: None,
        athena: None,
        driver: None,
        brief_sha256: None,
        dispatched_at: None,
        request_path: None,
        request_sha256: None,
        cost: None,
        tokens: None,
        note: None,
        pin: None,
        dispatch_branch: None,
        allowed_paths: None,
    }))
}

/// One invocation whose output is 5000 tokens, which must never be counted, and whose input, cache read and
/// cache write are the given quantities.
fn invocation(
    job: &str,
    seq: u32,
    role: Role,
    input: Quantity,
    cache_read: f64,
    cache_write: f64,
) -> AnyRecord {
    let tokens = |v: f64| Quantity::measured(v, Unit::Tokens, "result.usage");
    AnyRecord::Invocation(Box::new(Invocation {
        job: JobId::parse(job).unwrap(),
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
            prompt_tokens: Quantity::estimated(Some(1.0), Unit::Tokens, "estimate", None, None)
                .unwrap(),
            host_baseline: Quantity::estimated(Some(600.0), Unit::Tokens, "E3", None, None)
                .unwrap(),
            tools: vec![],
        },
        caps: InvocationCaps {
            max_budget_usd: 3.0,
            timeout_s: 1800,
        },
        started_at: at(),
        ended_at: at(),
        wall_time: Quantity::measured(60.0, Unit::Seconds, "athena clock"),
        tokens: Tokens {
            input,
            output: tokens(5000.0),
            cache_read: tokens(cache_read),
            cache_write: tokens(cache_write),
        },
        models_used: BTreeMap::from([(
            "m".to_string(),
            ModelUsage {
                input: 1,
                output: 1,
                cache_read: 1,
                cache_write: 1,
                cost_usd: 0.0,
            },
        )]),
        cost: Quantity::reported(0.0, Unit::Usd, "Claude Code total_cost_usd"),
        outcome: InvocationOutcome::Success,
        detail: None,
        turns: None,
        commands_run: vec![],
        policy_denials: vec![],
        reading_attempt: None,
    }))
}

fn measured(v: f64) -> Quantity {
    Quantity::measured(v, Unit::Tokens, "result.usage")
}

fn outcome(job: &str, state: OutcomeState) -> AnyRecord {
    AnyRecord::Outcome(Box::new(Outcome {
        job: JobId::parse(job).unwrap(),
        state,
        reason: "r".into(),
        at: at(),
        totals: Some(Totals {
            cost_all: Quantity::reported(0.0, Unit::Usd, "Claude Code total_cost_usd"),
            cost_execution: Quantity::reported(0.0, Unit::Usd, "Claude Code total_cost_usd"),
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

fn intervention(n: u32, class: &str, category: InterventionCategory) -> AnyRecord {
    AnyRecord::Intervention(Intervention {
        id: RecordId::parse(&format!("20261020-110000-m{n:03}")).unwrap(),
        at: at(),
        what: "what".into(),
        why: "why".into(),
        class_of_work: class.into(),
        category,
        job: None,
        source: InterventionSource::User,
        with_staged_changes: false,
    })
}

/// A closed job of `class` with one producer run that loaded `total` tokens (half as input, half as cache read).
fn simple_job(id: &str, class: &str, state: OutcomeState, total: f64) -> Vec<AnyRecord> {
    vec![
        job_record(id, class),
        invocation(
            id,
            1,
            Role::Producer,
            measured(total / 2.0),
            total / 2.0,
            0.0,
        ),
        outcome(id, state),
    ]
}

fn load(records: &[AnyRecord]) -> Records {
    let dir = tempfile::tempdir().unwrap();
    for record in records {
        let path = dir.path().join(record.canonical_path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, record.to_json()).unwrap();
    }
    let loaded = Records::load_dir(dir.path()).unwrap();
    assert!(loaded.invalid.is_empty(), "{:?}", loaded.invalid);
    loaded
}

#[test]
fn tokens_per_landed_job_and_outcomes_are_exact_for_two_classes() {
    let landed = OutcomeState::Landed;
    let mut files = Vec::new();
    // docs d1: a reader run of 100 + 200 + 50 = 350 and a killed producer run that recorded only a lower bound
    // of 30 input tokens, plus 20 cache read: 50. Total 400.
    let d1 = "20261021-100000-aaaa";
    files.push(job_record(d1, "docs"));
    files.push(invocation(
        d1,
        1,
        Role::Reader,
        measured(100.0),
        200.0,
        50.0,
    ));
    let bound = Quantity::estimated(None, Unit::Tokens, "killed", Some(30.0), None).unwrap();
    files.push(invocation(d1, 2, Role::Producer, bound, 20.0, 0.0));
    files.push(outcome(d1, landed));
    // d2: 300 + 500 + 100 = 900. d3: 200 + 1400 + 100 = 1700. d4 failed: 100 + 400 + 100 = 600.
    for (id, input, cache_read, cache_write, state) in [
        ("20261022-100000-aaab", 300.0, 500.0, 100.0, landed),
        ("20261023-100000-aaac", 200.0, 1400.0, 100.0, landed),
        (
            "20261024-100000-aaad",
            100.0,
            400.0,
            100.0,
            OutcomeState::Failed,
        ),
    ] {
        files.push(job_record(id, "docs"));
        files.push(invocation(
            id,
            1,
            Role::Producer,
            measured(input),
            cache_read,
            cache_write,
        ));
        files.push(outcome(id, state));
    }
    // code: three landed jobs of 2000, 3000 and 4000 tokens.
    for (id, total) in [
        ("20261025-100000-bbba", 2000.0),
        ("20261026-100000-bbbb", 3000.0),
        ("20261027-100000-bbbc", 4000.0),
    ] {
        files.extend(simple_job(id, "code", landed, total));
    }
    // An auditor run has its own generated job id and belongs to no class.
    files.push(invocation(
        "20261031-090000-zzzz",
        1,
        Role::Auditor,
        measured(100_000.0),
        0.0,
        0.0,
    ));
    // Two mistakes and one correction in docs; only the mistakes count.
    files.push(intervention(1, "docs", InterventionCategory::Mistake));
    files.push(intervention(2, "docs", InterventionCategory::Mistake));
    files.push(intervention(
        3,
        "docs",
        InterventionCategory::CorrectedOutput,
    ));

    let records = load(&files);
    assert_eq!(records.auditor_runs.len(), 1);
    let load = ContextLoad::compute(&records).expect("two classes have 3 landed jobs");
    assert_eq!(load.classes.len(), 2);

    let code = &load.classes[0];
    assert_eq!(code.class, "code");
    assert_eq!((code.closed_jobs, code.landed_jobs), (3, 3));
    assert_eq!(code.tokens, 9000.0);
    assert_eq!(code.tokens_per_landed_job, 3000.0);
    assert_eq!(code.landed_percent, 100.0);
    assert_eq!((code.mistakes, code.mistakes_per_closed_job), (0, 0.0));

    let docs = &load.classes[1];
    assert_eq!(docs.class, "docs");
    assert_eq!((docs.closed_jobs, docs.landed_jobs), (4, 3));
    assert_eq!(docs.tokens, 3600.0);
    assert_eq!(docs.tokens_per_landed_job, 1200.0);
    assert_eq!(docs.landed_percent, 75.0);
    assert_eq!((docs.mistakes, docs.mistakes_per_closed_job), (2, 0.5));

    let text = load.lines().join("\n");
    assert!(
        text.contains(
            "docs: 1200 tokens per landed job · 3 of 4 closed jobs landed (75%) · 2 mistake(s) recorded, 0.50 per closed job"
        ),
        "{text}"
    );
    assert!(
        text.contains(
            "code: 3000 tokens per landed job · 3 of 3 closed jobs landed (100%) · 0 mistake(s) recorded, 0.00 per closed job"
        ),
        "{text}"
    );
}

#[test]
fn a_class_below_the_minimum_yields_no_figure() {
    let landed = OutcomeState::Landed;
    let mut files = Vec::new();
    files.extend(simple_job("20261021-100000-cccc", "config", landed, 100.0));
    files.extend(simple_job("20261022-100000-cccd", "config", landed, 100.0));
    files.extend(simple_job(
        "20261023-100000-ccce",
        "config",
        OutcomeState::Failed,
        100.0,
    ));
    // Only two landed jobs: no figure at all, not a success.
    assert_eq!(ContextLoad::compute(&load(&files)), None);
    // Beside a class that qualifies, the thin class is absent.
    for (id, total) in [
        ("20261025-100000-bbba", 10.0),
        ("20261026-100000-bbbb", 10.0),
        ("20261027-100000-bbbc", 10.0),
    ] {
        files.extend(simple_job(id, "code", landed, total));
    }
    let loaded = ContextLoad::compute(&load(&files)).unwrap();
    assert_eq!(loaded.classes.len(), 1);
    assert_eq!(loaded.classes[0].class, "code");
}

#[test]
fn the_spec_entry_is_not_a_gap_and_none_makes_it_one() {
    let requirements = parse_spec(&repo_file("docs/spec.toml")).expect("docs/spec.toml parses");
    assert!(
        !spec_gaps(&requirements).iter().any(|g| g.id == REQUIREMENT),
        "OLY-B-A13-01 is still a spec gap"
    );
    let mut r = real_requirement();
    r.mechanism = "none".into();
    let gaps = spec_gaps(&[r]);
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0].id, REQUIREMENT);
    assert!(gaps[0].no_mechanism);
}

#[test]
fn spec_entry_names_the_module_its_test_and_a_shown_label() {
    let r = real_requirement();
    assert!(
        r.mechanism
            .contains("crates/athena-core/src/context_load.rs"),
        "{}",
        r.mechanism
    );
    assert!(
        r.mechanism
            .contains("crates/athena-core/tests/context_compression.rs"),
        "{}",
        r.mechanism
    );
    assert_eq!(r.owner, "Athena");
    assert!(
        r.measure.trim_end().ends_with("status: context"),
        "{}",
        r.measure
    );
    assert_eq!(measure_problem(&r.measure), None, "{}", r.measure);
    assert!(STATUS_MEASURE_LABELS.contains(&("context", "context_load")));
}
