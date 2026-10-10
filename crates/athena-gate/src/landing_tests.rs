//! Landings by squash and rebase merge (design §7.3): the landing is proved by the records the commit
//! adds, with no `Athena-*` trailers and no ancestry.

use crate::testutil::*;
use crate::{Change, Judged, Rule, Violation, history, invariant, short};
use athena_data::records::{
    AnyRecord, Build, Candidate, Criterion, CriterionKind, DefaultApplied, Deliverable,
    DeliverableAction, DriverInfo, Forecast, Forecasts, Invocation, InvocationCaps,
    InvocationContext, InvocationOutcome, ModelUsage, Outcome, OutcomeState, Quantiles, Reading,
    Role, Tokens, Totals, Verdict, Verification,
};
use athena_data::{JobId, Quantity, Unit};
use athena_git::Git;
use athena_git::attrs::fingerprint;
use std::collections::BTreeMap;

const JOB: &str = "20261010-153012-k7q2";
const PRODUCER: &str = "claude-sonnet-5-5";
const VERIFIER: &str = "claude-opus-5-5";
const NEW_CODE: &str = "pub fn a() { /* landed */ }\n";

/// When each step of the job happened.
struct Times {
    reading: &'static str,
    dispatched: Option<&'static str>,
}

const HONEST: Times = Times {
    reading: "2026-10-10T15:01:30Z",
    dispatched: Some("2026-10-10T15:02:00Z"),
};

fn job_id() -> JobId {
    JobId::parse(JOB).unwrap()
}

fn ts(text: &str) -> jiff::Timestamp {
    text.parse().unwrap()
}

fn owner_repo() -> (tempfile::TempDir, Git) {
    repo(&[
        (
            ".gitattributes",
            "/OWNER.md athena-exclude\n/SPEC.md athena-readonly\n",
        ),
        ("src/lib.rs", "pub fn a() {}\n"),
    ])
}

fn reading(times: &Times) -> Reading {
    let forecast = Forecast {
        p_reading_accepted: 0.8,
        p_verification_passes: 0.8,
        p_user_intervenes: 0.15,
        cost_usd: Quantiles::new(0.2, 0.45, 1.1).unwrap(),
        wall_time_s: Quantiles::new(120.0, 240.0, 540.0).unwrap(),
    };
    Reading {
        job: job_id(),
        n: 1,
        at: ts(times.reading),
        invocation_seq: 1,
        base_commit: "0".repeat(40),
        config_sha256: "a".repeat(64),
        defaults_sha256: "b".repeat(64),
        defaults_version: 1,
        title: "Change a".into(),
        summary: "Change a.".into(),
        class_of_work: "code".into(),
        capability: Some("change".into()),
        decline_reason: None,
        producer_model: PRODUCER.into(),
        verifier_model: VERIFIER.into(),
        defaults_applied: vec![DefaultApplied {
            id: "tested".into(),
            how: "a test".into(),
        }],
        assumptions: vec![],
        question: None,
        acceptance_criteria: vec![Criterion {
            text: "a changes".into(),
            kind: CriterionKind::Content,
        }],
        deliverables: vec![Deliverable {
            path: "src/lib.rs".into(),
            action: DeliverableAction::Modify,
            p: 0.95,
        }],
        out_of_scope: vec![],
        forecasts: Forecasts {
            reader: forecast.clone(),
            reference: forecast,
        },
        departures: None,
    }
}

fn job_record(times: &Times) -> AnyRecord {
    let dispatched = times
        .dispatched
        .map_or("null".to_string(), |t| format!("\"{t}\""));
    let text = format!(
        "{{\"schema\":\"athena.job/1\",\"id\":\"{JOB}\",\"kind\":\"request\",\"title\":\"t\",\"class_of_work\":\"code\",\"capability\":\"change\",\"accepted_reading\":1,\"accepted_by\":\"user\",\"owner\":{{\"role\":\"producer\",\"capability\":null,\"model\":null,\"note\":null}},\"verifier_model\":\"{VERIFIER}\",\"caps\":null,\"base_commit\":null,\"config_sha256\":null,\"defaults_sha256\":null,\"athena\":null,\"driver\":null,\"brief_sha256\":null,\"dispatched_at\":{dispatched},\"request_path\":null,\"request_sha256\":null,\"cost\":null,\"tokens\":null,\"note\":null}}"
    );
    serde_json::from_str(&text).unwrap()
}

fn usage() -> ModelUsage {
    ModelUsage {
        input: 10,
        output: 20,
        cache_read: 30,
        cache_write: 40,
        cost_usd: 0.1,
    }
}

fn invocation(seq: u32, role: Role, model: &str, start: &str, end: &str) -> Invocation {
    let tokens = |v: f64| Quantity::measured(v, Unit::Tokens, "result.usage");
    Invocation {
        job: job_id(),
        seq,
        role,
        driver: DriverInfo {
            name: "claude-code".into(),
            version: "2.1.296".into(),
        },
        athena: Build {
            version: "0.1.0".into(),
            source_commit: "0".repeat(40),
            dirty: false,
        },
        model: model.into(),
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
        started_at: ts(start),
        ended_at: ts(end),
        wall_time: Quantity::measured(60.0, Unit::Seconds, "athena clock"),
        tokens: Tokens {
            input: tokens(10.0),
            output: tokens(20.0),
            cache_read: tokens(30.0),
            cache_write: tokens(40.0),
        },
        models_used: BTreeMap::from([(model.to_string(), usage())]),
        cost: Quantity::reported(0.1, Unit::Usd, "Claude Code total_cost_usd"),
        outcome: InvocationOutcome::Success,
        detail: None,
        turns: Some(3),
        commands_run: vec![],
        policy_denials: vec![],
    }
}

fn candidate(fp: &str) -> Candidate {
    Candidate {
        job: job_id(),
        base_commit: "0".repeat(40),
        candidate_commit: Some("c".repeat(40)),
        candidate_tree: None,
        changes: vec![],
        fingerprint: Some(fp.to_string()),
        violations: vec![],
        producer_report: None,
    }
}

fn verification(fp: &str) -> Verification {
    Verification {
        job: job_id(),
        candidate_commit: "c".repeat(40),
        verified_fingerprint: fp.to_string(),
        tool_config_from_base: vec![],
        checks: vec![],
        check_cleanup: vec![],
        verifier_invocation: Some(3),
        criteria: vec![],
        findings: vec![],
        request_reading_mismatch: None,
        agent_verdict: Some(Verdict::Pass),
        final_verdict: Verdict::Pass,
        reasons: vec![],
        inputs: vec![],
        summary: None,
    }
}

fn outcome() -> Outcome {
    // Reader and producer used PRODUCER (two invocations); the verifier used VERIFIER.
    let sonnet = ModelUsage {
        input: 20,
        output: 40,
        cache_read: 60,
        cache_write: 80,
        cost_usd: 0.2,
    };
    Outcome {
        job: job_id(),
        state: OutcomeState::Landed,
        reason: "verified".into(),
        at: ts("2026-10-10T15:08:00Z"),
        totals: Some(Totals {
            cost_all: Quantity::reported(0.3, Unit::Usd, "sum"),
            cost_execution: Quantity::reported(0.2, Unit::Usd, "sum"),
            wall_time_execution: None,
            tokens: BTreeMap::from([
                (PRODUCER.to_string(), sonnet),
                (VERIFIER.to_string(), usage()),
            ]),
            invocations: 3,
        }),
        branch: None,
        base_commit: None,
        candidate_commit: None,
        after_blocked: false,
        through_commit: None,
    }
}

/// The records that open the job: the reading, the dispatch, and the reader's invocation.
fn opening(times: &Times) -> Vec<AnyRecord> {
    vec![
        AnyRecord::Reading(Box::new(reading(times))),
        job_record(times),
        AnyRecord::Invocation(Box::new(invocation(
            1,
            Role::Reader,
            PRODUCER,
            "2026-10-10T15:00:00Z",
            "2026-10-10T15:01:00Z",
        ))),
    ]
}

/// The records that close the job, verified against `fp`.
fn closing(fp: &str) -> Vec<AnyRecord> {
    vec![
        AnyRecord::Invocation(Box::new(invocation(
            2,
            Role::Producer,
            PRODUCER,
            "2026-10-10T15:03:00Z",
            "2026-10-10T15:05:00Z",
        ))),
        AnyRecord::Invocation(Box::new(invocation(
            3,
            Role::Verifier,
            VERIFIER,
            "2026-10-10T15:06:00Z",
            "2026-10-10T15:07:00Z",
        ))),
        AnyRecord::Candidate(Box::new(candidate(fp))),
        AnyRecord::Verification(Box::new(verification(fp))),
        AnyRecord::Outcome(Box::new(outcome())),
    ]
}

fn write_records(root: &std::path::Path, records: &[AnyRecord]) {
    for record in records {
        write(root, &[(&record.canonical_path(), &record.to_json())]);
    }
}

/// The fingerprint of the repository's content with the landed change applied to the working tree.
fn landed_fingerprint(git: &Git, base: &str) -> String {
    git.run(["add", "-A"]).unwrap();
    let tree = git.write_index_tree().unwrap();
    fingerprint(git, &tree, base).unwrap()
}

/// A job's records and the landed code in the working tree, uncommitted; returns the fingerprint.
fn stage_landing(root: &std::path::Path, git: &Git, base: &str, times: &Times) -> String {
    write(root, &[("src/lib.rs", NEW_CODE)]);
    let fp = landed_fingerprint(git, base);
    write_records(root, &opening(times));
    write_records(root, &closing(&fp));
    fp
}

fn judge(git: &Git, rev: &str) -> Vec<Violation> {
    let id = git.rev_parse(rev).unwrap().unwrap();
    let change = Change::of_commit(git, &id).unwrap();
    let judged = Judged::new(git, &change).unwrap();
    invariant::check(git, &change, &judged, None).unwrap()
}

fn history_of(git: &Git) -> Vec<Violation> {
    history::check_history(git, "HEAD").unwrap().violations
}

fn amend(git: &Git, message: &str) {
    git.run(["commit", "-q", "--amend", "-m", message]).unwrap();
}

#[test]
fn a_squash_merge_with_no_trailers_lands() {
    let (dir, git) = owner_repo();
    let base = git.head().unwrap().unwrap();
    stage_landing(dir.path(), &git, &base, &HONEST);
    commit(&git, "Change a (#1)\n");
    assert_eq!(judge(&git, "HEAD"), vec![]);
    assert_eq!(history_of(&git), vec![]);
}

#[test]
fn a_rebase_merge_lands_though_the_prediction_commit_is_no_ancestor() {
    let (dir, git) = owner_repo();
    let base = git.head().unwrap().unwrap();
    git.run(["checkout", "-q", "-b", "pr"]).unwrap();
    write_records(dir.path(), &opening(&HONEST));
    let prediction = commit(&git, &format!("Read\n\nAthena-Job: {JOB}\n"));
    write(dir.path(), &[("src/lib.rs", NEW_CODE)]);
    let fp = landed_fingerprint(&git, &prediction);
    write_records(dir.path(), &closing(&fp));
    commit(
        &git,
        &format!(
            "Land\n\nAthena-Job: {JOB}\nAthena-Prediction: {prediction}\nAthena-Verified-Content: {fp}\n"
        ),
    );
    // Main moved on while the PR was open; the rebase replays both commits, with new ids.
    git.run(["checkout", "-q", "main"]).unwrap();
    write(dir.path(), &[("OWNER.md", "a note\n")]);
    commit(&git, "owner note\n");
    git.run(["checkout", "-q", "-b", "rebased", "main"])
        .unwrap();
    git.run(["cherry-pick", &format!("{base}..pr")]).unwrap();
    git.run(["checkout", "-q", "main"]).unwrap();
    git.run(["merge", "-q", "--ff-only", "rebased"]).unwrap();
    assert!(!git.is_ancestor(&prediction, "main").unwrap());
    assert_eq!(history_of(&git), vec![]);
}

#[test]
fn a_squash_that_changes_governed_content_without_the_jobs_records_is_refused() {
    let (dir, git) = owner_repo();
    write(dir.path(), &[("src/lib.rs", NEW_CODE)]);
    commit(&git, "Change a (#1)\n");
    let found = judge(&git, "HEAD");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].rule, Rule::Invariant);

    // Records that verify some other content do not help.
    let (dir, git) = owner_repo();
    let base = git.head().unwrap().unwrap();
    let elsewhere = fingerprint(&git, &base, &base).unwrap();
    write(dir.path(), &[("src/lib.rs", NEW_CODE)]);
    write_records(dir.path(), &opening(&HONEST));
    write_records(dir.path(), &closing(&elsewhere));
    commit(&git, "Change a (#1)\n");
    let found = judge(&git, "HEAD");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].rule, Rule::Invariant);
}

#[test]
fn a_squash_whose_reading_was_written_after_the_work_is_refused() {
    // After the dispatch, and so after the producer began.
    let late = Times {
        reading: "2026-10-10T15:04:00Z",
        dispatched: Some("2026-10-10T15:02:00Z"),
    };
    // No dispatch time to lean on: the producer invocation alone shows the reading came late.
    let undated = Times {
        reading: "2026-10-10T15:04:00Z",
        dispatched: None,
    };
    for (times, wanted) in [
        (late, "after the job was dispatched"),
        (undated, "does not precede"),
    ] {
        let (dir, git) = owner_repo();
        let base = git.head().unwrap().unwrap();
        stage_landing(dir.path(), &git, &base, &times);
        commit(&git, "Change a (#1)\n");
        let found = judge(&git, "HEAD");
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].rule, Rule::Invariant);
        assert!(found[0].detail.contains(wanted), "{}", found[0].detail);
        assert!(!history_of(&git).is_empty());
    }
}

#[test]
fn a_second_landing_of_the_same_job_after_a_squash_is_refused() {
    let (dir, git) = owner_repo();
    let base = git.head().unwrap().unwrap();
    stage_landing(dir.path(), &git, &base, &HONEST);
    commit(&git, "Change a (#1)\n");
    assert_eq!(history_of(&git), vec![]);

    // The landing is undone by hand and the same squash is made again: the parent holds no record of
    // the first landing, so only the walk can see it.
    std::fs::remove_dir_all(dir.path().join("records")).unwrap();
    write(dir.path(), &[("src/lib.rs", "pub fn a() {}\n")]);
    commit(&git, "Undo\n\nAthena-Job: bootstrap\n");
    let fp = stage_landing(dir.path(), &git, &base, &HONEST);
    let again = commit(&git, "Change a again (#2)\n");
    assert_eq!(fp, landed_fingerprint(&git, &base));
    let found = history_of(&git);
    assert!(
        found.iter().any(|v| v.rule == Rule::Invariant
            && v.commit.as_deref() == Some(short(&again).as_str())
            && v.detail.contains("already landed")),
        "{found:?}"
    );
}

#[test]
fn trailers_on_a_squash_neither_admit_nor_refuse_by_themselves() {
    let (dir, git) = owner_repo();
    let base = git.head().unwrap().unwrap();
    let fp = stage_landing(dir.path(), &git, &base, &HONEST);
    commit(&git, "Change a (#1)\n");

    // Stale or edited hints that do not contradict the records change nothing.
    amend(
        &git,
        &format!(
            "Change a (#1)\n\nAthena-Job: {JOB}\nAthena-Prediction: {}\nAthena-Verified-Content: {fp}\n",
            "1".repeat(40)
        ),
    );
    assert_eq!(judge(&git, "HEAD"), vec![]);
    amend(
        &git,
        &format!("Change a (#1)\n\nAthena-Prediction: {}\n", "2".repeat(40)),
    );
    assert_eq!(judge(&git, "HEAD"), vec![]);

    // Hints that contradict them are refused, whatever the records say.
    amend(&git, "Change a (#1)\n\nAthena-Job: 20261010-153012-abcd\n");
    let found = judge(&git, "HEAD");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].detail.contains("Athena-Job"),
        "{}",
        found[0].detail
    );
    amend(
        &git,
        &format!(
            "Change a (#1)\n\nAthena-Verified-Content: {}\n",
            "3".repeat(64)
        ),
    );
    let found = judge(&git, "HEAD");
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].detail.contains("Athena-Verified-Content"));

    // And trailers cannot admit a change whose records are missing.
    let (dir, git) = owner_repo();
    write(dir.path(), &[("src/lib.rs", NEW_CODE)]);
    commit(
        &git,
        &format!(
            "Change a\n\nAthena-Job: {JOB}\nAthena-Prediction: {}\nAthena-Verified-Content: {}\n",
            "1".repeat(40),
            "3".repeat(64)
        ),
    );
    assert_eq!(judge(&git, "HEAD").len(), 1);
}
