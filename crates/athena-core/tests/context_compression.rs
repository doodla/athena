//! Requirement OLY-B-A13-01: the `context` measure. Records are written as files, loaded through the real
//! loader (`Records::load_dir`), and the per-class figures are checked against token counts summed by hand.
//! The spec entry and the gap rule are checked against the real `docs/spec.toml`.

use std::collections::BTreeMap;
use std::path::Path;

use athena_core::context_load::{ContextLoad, MIN_CLASS_JOBS};
use athena_core::hestia::{measure_problem, parse_spec, spec_gaps};
use athena_core::status::{STATUS_MEASURE_LABELS, mistake_report};
use athena_core::store::Records;
use athena_data::records::{
    AnyRecord, Build, DriverInfo, Intervention, InterventionCategory, InterventionSource,
    Invocation, InvocationCaps, InvocationContext, InvocationOutcome, Job, JobKind, Outcome,
    OutcomeState, Owner, Role, Tokens, Totals,
};
use athena_data::{JobId, Quantity, RecordId, Unit};

fn job_id(n: u32) -> JobId {
    JobId::parse(&format!("202611{n:02}-120000-k{n:03}")).unwrap()
}

fn tokens(v: f64) -> Quantity {
    Quantity::measured(v, Unit::Tokens, "result.usage")
}

fn job(n: u32, class: &str) -> AnyRecord {
    AnyRecord::Job(Box::new(Job {
        id: job_id(n),
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

/// One invocation of job `n` that loaded `input + cache_read + cache_write` tokens and generated `output`.
fn invocation(
    n: u32,
    seq: u32,
    input: f64,
    cache_read: f64,
    cache_write: f64,
    output: f64,
) -> AnyRecord {
    AnyRecord::Invocation(Box::new(Invocation {
        job: job_id(n),
        seq,
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
        started_at: "2026-11-01T12:00:00Z".parse().unwrap(),
        ended_at: "2026-11-01T12:05:00Z".parse().unwrap(),
        wall_time: Quantity::measured(60.0, Unit::Seconds, "athena clock"),
        tokens: Tokens {
            input: tokens(input),
            output: tokens(output),
            cache_read: tokens(cache_read),
            cache_write: tokens(cache_write),
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

fn outcome(n: u32, state: OutcomeState) -> AnyRecord {
    AnyRecord::Outcome(Box::new(Outcome {
        job: job_id(n),
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

fn intervention(
    n: u32,
    class: &str,
    category: InterventionCategory,
    on_job: Option<u32>,
) -> AnyRecord {
    AnyRecord::Intervention(Intervention {
        id: RecordId::parse(&format!("20261105-1200{n:02}-1abc")).unwrap(),
        at: "2026-11-05T12:00:00Z".parse().unwrap(),
        what: "what".into(),
        why: "why".into(),
        class_of_work: class.into(),
        category,
        job: on_job.map(job_id),
        source: InterventionSource::User,
        with_staged_changes: false,
    })
}

/// A landed job of `class` whose single invocation loaded `input + cache_read + cache_write` tokens; the
/// large output must not count.
fn landed(n: u32, class: &str, input: f64, cache_read: f64, cache_write: f64) -> Vec<AnyRecord> {
    vec![
        job(n, class),
        invocation(n, 1, input, cache_read, cache_write, 99_999.0),
        outcome(n, OutcomeState::Landed),
    ]
}

fn write_and_load(records: &[AnyRecord]) -> Records {
    let dir = tempfile::tempdir().unwrap();
    for r in records {
        let path = dir.path().join(r.canonical_path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, r.to_json()).unwrap();
    }
    assert!(Path::new(dir.path()).join("records").is_dir());
    let loaded = Records::load_dir(dir.path()).unwrap();
    assert!(loaded.invalid.is_empty(), "{:?}", loaded.invalid);
    loaded
}

/// Two classes with a figure, one thin class, and four mistakes of which two belong to counted jobs.
fn seeded() -> Vec<AnyRecord> {
    let mut r = Vec::new();
    // code: 1000, 2000 and 3000 tokens loaded, one failed job; 3 landed of 4 closed.
    r.extend(landed(1, "code", 400.0, 500.0, 100.0));
    r.extend(landed(2, "code", 1000.0, 600.0, 400.0));
    r.extend(landed(3, "code", 1500.0, 1000.0, 500.0));
    r.extend([
        job(4, "code"),
        invocation(4, 1, 50_000.0, 0.0, 0.0, 1.0),
        outcome(4, OutcomeState::Failed),
    ]);
    // docs: 500, 1500 and 1000 tokens; 3 of 3.
    r.extend(landed(5, "docs", 200.0, 200.0, 100.0));
    r.extend(landed(6, "docs", 500.0, 500.0, 500.0));
    r.extend(landed(7, "docs", 1000.0, 0.0, 0.0));
    // study: two landed jobs, below the floor.
    r.extend(landed(8, "study", 100.0, 0.0, 0.0));
    r.extend(landed(9, "study", 100.0, 0.0, 0.0));
    // A job still running: not closed.
    r.push(job(10, "code"));
    // Mistakes: one on a landed code job, one on the failed code job (both counted for code); one with no
    // job and one on the running job (both left out). A non-mistake intervention on a code job is ignored.
    r.push(intervention(
        1,
        "code",
        InterventionCategory::Mistake,
        Some(1),
    ));
    r.push(intervention(
        2,
        "code",
        InterventionCategory::Mistake,
        Some(4),
    ));
    r.push(intervention(3, "code", InterventionCategory::Mistake, None));
    r.push(intervention(
        4,
        "code",
        InterventionCategory::Mistake,
        Some(10),
    ));
    r.push(intervention(
        5,
        "code",
        InterventionCategory::CorrectedOutput,
        Some(2),
    ));
    r
}

#[test]
fn per_class_figures_are_exact_for_known_token_counts() {
    let records = write_and_load(&seeded());
    let context = ContextLoad::compute(&records).expect("two classes have enough landed jobs");
    let names: Vec<&str> = context.classes.iter().map(|c| c.class.as_str()).collect();
    assert_eq!(names, ["code", "docs"]);

    let code = &context.classes[0];
    assert_eq!(code.landed_jobs, 3);
    assert_eq!(code.closed_jobs, 4);
    assert_eq!(code.tokens_per_landed_job, 2000.0);
    assert_eq!(code.landed_share, 0.75);
    assert_eq!(code.mistakes, 2);

    let docs = &context.classes[1];
    assert_eq!(docs.landed_jobs, 3);
    assert_eq!(docs.closed_jobs, 3);
    assert_eq!(docs.tokens_per_landed_job, 1000.0);
    assert_eq!(docs.landed_share, 1.0);
    assert_eq!(docs.mistakes, 0);

    assert_eq!(context.thin_classes, 1, "study has two landed jobs");
    assert_eq!(context.landed_without_tokens, 0);

    let lines = context.lines();
    assert!(
        lines.contains(
            &"code: 2000 tokens per landed job over 3 landed jobs · landed 3 of 4 closed (75%) · 2 mistakes in those 4 jobs"
                .to_string()
        ),
        "{lines:#?}"
    );
    assert!(
        lines.contains(
            &"docs: 1000 tokens per landed job over 3 landed jobs · landed 3 of 3 closed (100%) · 0 mistakes in those 3 jobs"
                .to_string()
        ),
        "{lines:#?}"
    );
    assert!(
        lines.iter().all(|l| !l.starts_with("study")),
        "a thin class yields no figure: {lines:#?}"
    );
}

#[test]
fn only_thin_classes_yield_no_context_at_all() {
    let mut r = Vec::new();
    for n in 1..=(MIN_CLASS_JOBS as u32 - 1) {
        r.extend(landed(n, "code", 100.0, 100.0, 100.0));
    }
    let records = write_and_load(&r);
    assert_eq!(ContextLoad::compute(&records), None);
    assert_eq!(ContextLoad::compute(&Records::default()), None);
}

#[test]
fn class_figures_use_only_the_counted_jobs() {
    let records = write_and_load(&seeded());
    let context = ContextLoad::compute(&records).unwrap();
    // The unattached mistake and the mistake on the running job are left out, never folded into code.
    assert_eq!(context.mistakes_left_out, 2);
    let shown: usize = context.classes.iter().map(|c| c.mistakes).sum();
    assert_eq!(shown, 2);
    // Shown plus left out is every mistake the existing report counts.
    let report = mistake_report(&records);
    assert_eq!(shown + context.mistakes_left_out, report.total);
    assert_eq!(report.total, 4);
    assert!(
        context
            .lines()
            .iter()
            .any(|l| l.starts_with("2 mistake(s) are not tied to a closed job")),
        "{:#?}",
        context.lines()
    );
}

#[test]
fn a_landed_job_with_no_token_record_is_named_and_does_not_lower_the_mean() {
    let mut r = seeded();
    // A fourth landed docs job that has no invocation, so no token value.
    r.push(job(11, "docs"));
    r.push(outcome(11, OutcomeState::Landed));
    let records = write_and_load(&r);
    let context = ContextLoad::compute(&records).unwrap();
    let docs = context.classes.iter().find(|c| c.class == "docs").unwrap();
    assert_eq!(docs.tokens_per_landed_job, 1000.0);
    assert_eq!(docs.landed_jobs, 3);
    assert_eq!(docs.closed_jobs, 4);
    assert_eq!(context.landed_without_tokens, 1);
    assert!(
        context
            .lines()
            .iter()
            .any(|l| l.starts_with("1 landed job(s) have no token record")),
        "{:#?}",
        context.lines()
    );
}

#[test]
fn oly_b_a13_01_is_met_by_context_load_and_shown_by_the_context_line() {
    let requirements = parse_spec(include_str!("../../../docs/spec.toml")).unwrap();
    let req = requirements
        .iter()
        .find(|r| r.id == "OLY-B-A13-01")
        .expect("OLY-B-A13-01 is in the spec");
    assert!(req.mechanism != "none", "{}", req.mechanism);
    assert!(
        req.mechanism
            .contains("crates/athena-core/src/context_load.rs"),
        "{}",
        req.mechanism
    );
    assert!(
        req.mechanism.contains("context_compression"),
        "{}",
        req.mechanism
    );
    assert_eq!(req.owner, "Athena");
    assert!(req.measure.ends_with("status: context"), "{}", req.measure);
    assert_eq!(measure_problem(&req.measure), None);
    assert!(STATUS_MEASURE_LABELS.contains(&("context", "context_load")));
    assert!(
        include_str!("../src/lib.rs").contains("pub mod context_load;"),
        "context_load is registered in lib.rs"
    );

    let gaps = spec_gaps(&requirements);
    assert!(gaps.iter().all(|g| g.id != "OLY-B-A13-01"));
    // Without the mechanism the requirement is a gap again.
    let mut without = req.clone();
    without.mechanism = "none".into();
    let gaps = spec_gaps(&[without]);
    assert!(
        gaps.len() == 1 && gaps[0].no_mechanism && gaps[0].id == "OLY-B-A13-01",
        "{gaps:?}"
    );
}
