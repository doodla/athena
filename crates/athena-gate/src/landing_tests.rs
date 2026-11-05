//! Landings by squash and rebase merge (design §7.3): the landing is proved by the records the commit
//! adds, with no `Athena-*` trailers and no ancestry.

use crate::testutil::*;
use crate::{Change, Judged, Rule, Scope, Violation, history, invariant, short};
use athena_data::records::{
    AnyRecord, Build, Candidate, Caps, Criterion, CriterionKind, DefaultApplied, Deliverable,
    DeliverableAction, DriverInfo, Forecast, Forecasts, Intervention, InterventionCategory,
    InterventionSource, Invocation, InvocationCaps, InvocationContext, InvocationOutcome, Job,
    ModelUsage, Outcome, OutcomeState, Quantiles, Reading, Request, Role, Tokens, Totals, Verdict,
    Verification,
};
use athena_data::{JobId, Quantity, RecordId, Unit};
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
        urgent_reason: None,
        hard_reason: None,
        producer_effort: None,
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
            version: "2.1.299".into(),
        },
        athena: Build {
            version: "0.1.0".into(),
            source_commit: "0".repeat(40),
            dirty: false,
        },
        model: model.into(),
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
        base_fingerprint: None,
        previous_job: None,
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

pub(crate) fn judge(git: &Git, rev: &str) -> Vec<Violation> {
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

/// The pre-merge classification of the head commit.
fn pre_merge(git: &Git) -> Result<invariant::PreMergeCommit, invariant::Uncovered> {
    let sha = git.head().unwrap().unwrap();
    invariant::classify_pre_merge(git, &sha).unwrap()
}

#[test]
fn the_records_of_a_held_job_are_records_only_and_need_no_verification() {
    // What a job held for lack of disk commits: the opening records and the candidate (`candidate2.json`),
    // with no verification and no outcome.
    let (dir, git) = owner_repo();
    write_records(dir.path(), &opening(&HONEST));
    // A candidate/2 record states its base fingerprint; a record without one does not load.
    let held = Candidate {
        base_fingerprint: Some("e".repeat(40)),
        ..candidate(&"f".repeat(40))
    };
    write_records(dir.path(), &[AnyRecord::Candidate2(Box::new(held))]);
    commit(&git, "Hold job for lack of disk\n");
    let covered = pre_merge(&git).expect("a held job's records are covered");
    assert_eq!(covered.kind, athena_data::history::LandingKind::RecordsOnly);
    assert_eq!(covered.job, None);
}

#[test]
fn the_closing_records_of_a_finished_job_without_a_verification_are_still_refused() {
    let (dir, git) = owner_repo();
    let fp = "f".repeat(40);
    write_records(dir.path(), &opening(&HONEST));
    let without_verification: Vec<AnyRecord> = closing(&fp)
        .into_iter()
        .filter(|r| !matches!(r, AnyRecord::Verification(_)))
        .collect();
    write_records(dir.path(), &without_verification);
    commit(&git, "Close job\n");
    assert_eq!(
        pre_merge(&git),
        Err(invariant::Uncovered::NoVerification(job_id()))
    );
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
fn a_squash_whose_message_is_built_from_the_landed_commit_passes_the_history_check() {
    let (dir, git) = owner_repo();
    let base = git.head().unwrap().unwrap();
    let fp = stage_landing(dir.path(), &git, &base, &HONEST);
    let landed = commit(
        &git,
        &format!(
            "Change a\n\nWhy a changed.\n\nAthena-Job: {JOB}\nAthena-Verified-Content: {fp}\n"
        ),
    );
    // The text the host is asked to use for the squash, and the squash it makes: one commit on the base.
    let text = git.squash_text(&landed).unwrap();
    let message = format!("{}\n\n{}", text.title, text.body);
    let squash = git
        .commit_tree(&git.tree_of(&landed).unwrap(), &[&base], &message, false)
        .unwrap();
    git.run(["reset", "-q", "--hard", &squash]).unwrap();

    // The trailers are the last paragraph and name exactly one job.
    let trailers = git.trailers(&git.message(&squash).unwrap()).unwrap();
    assert_eq!(
        trailers
            .iter()
            .filter(|(key, _)| key == "Athena-Job")
            .collect::<Vec<_>>(),
        [&("Athena-Job".to_string(), JOB.to_string())]
    );
    assert!(trailers.contains(&("Athena-Verified-Content".to_string(), fp)));
    assert!(git.message(&squash).unwrap().starts_with("Change a\n\n"));
    assert_eq!(judge(&git, "HEAD"), vec![]);
    assert_eq!(history_of(&git), vec![]);
    assert_eq!(git.parents(&squash).unwrap(), [base]);
}

/// A squash that carries a landed job, an intervention on it, and the re-score `athena intervene --job`
/// wrote. The closing score was made before the intervention, so it must not be recomputed with it.
fn squash_with_intervention(forge: bool) -> Vec<Violation> {
    let (dir, git) = owner_repo();
    let base = git.head().unwrap().unwrap();
    stage_landing(dir.path(), &git, &base, &HONEST);
    git.run(["add", "-A"]).unwrap();
    let landed_tree = git.write_index_tree().unwrap();
    let closing = crate::jobs::expected_score(
        &git,
        &landed_tree,
        &job_id(),
        None,
        ts("2026-10-10T15:08:30Z"),
        athena_score::RulesVersion::CURRENT,
    )
    .unwrap();
    let cause = RecordId::parse("20261011-090000-abcd").unwrap();
    let intervention = Intervention {
        id: cause.clone(),
        at: ts("2026-10-11T09:00:00Z"),
        what: "Fixed the landed code by hand".into(),
        why: "It was wrong".into(),
        class_of_work: "code".into(),
        category: InterventionCategory::CorrectedOutput,
        job: Some(job_id()),
        source: InterventionSource::User,
        with_staged_changes: false,
    };
    let mut rescore = athena_score::rescore(&closing, cause, ts("2026-10-11T09:00:01Z"));
    if forge {
        for event in &mut rescore.events {
            for b in event.brier.values_mut() {
                *b = 0.0;
            }
        }
    }
    write_records(
        dir.path(),
        &[
            AnyRecord::Score(Box::new(closing)),
            AnyRecord::Intervention(intervention),
            AnyRecord::Score(Box::new(rescore)),
        ],
    );
    commit(&git, "Change a (#1)\n");
    let mut found = judge(&git, "HEAD");
    found.extend(job_records_of(&git, "HEAD"));
    found
}

#[test]
fn a_squash_with_an_intervention_and_its_rescore_lands() {
    assert_eq!(squash_with_intervention(false), vec![]);
}

#[test]
fn a_squash_with_a_forged_rescore_is_still_refused() {
    let found = squash_with_intervention(true);
    assert!(
        found
            .iter()
            .any(|v| v.rule == Rule::JobRecords && v.detail.contains("does not follow")),
        "{found:?}"
    );
}

/// A landing whose producer was killed, so its cost is only a bound, with a closing score computed under
/// `version` and then changed by `edit`. Returns the job-record violations and the score written.
fn landing_with_bound_cost(
    version: athena_score::RulesVersion,
    edit: impl Fn(&mut athena_data::records::Score),
) -> (Vec<Violation>, athena_data::records::Score) {
    let (dir, git) = owner_repo();
    let base = git.head().unwrap().unwrap();
    let fp = stage_landing(dir.path(), &git, &base, &HONEST);
    let mut killed = invocation(
        2,
        Role::Producer,
        PRODUCER,
        "2026-10-10T15:03:00Z",
        "2026-10-10T15:05:00Z",
    );
    killed.cost =
        Quantity::estimated(None, Unit::Usd, "bounded by the cap", Some(0.1), Some(3.0)).unwrap();
    write_records(dir.path(), &[AnyRecord::Invocation(Box::new(killed))]);
    git.run(["add", "-A"]).unwrap();
    let tree = git.write_index_tree().unwrap();
    assert_eq!(fingerprint(&git, &tree, &base).unwrap(), fp);
    let mut score = crate::jobs::expected_score(
        &git,
        &tree,
        &job_id(),
        None,
        ts("2026-10-10T15:08:30Z"),
        version,
    )
    .unwrap();
    edit(&mut score);
    write_records(dir.path(), &[AnyRecord::Score(Box::new(score.clone()))]);
    commit(&git, "Change a (#1)\n");
    assert_eq!(judge(&git, "HEAD"), vec![]);
    (job_records_of(&git, "HEAD"), score)
}

fn bound_cost_is_provisional(score: &athena_data::records::Score) -> bool {
    score
        .quantities
        .iter()
        .find(|q| q.quantity == "cost_usd")
        .expect("a cost quantity")
        .provisional
}

#[test]
fn a_score_written_under_v1_is_accepted_with_its_bound_cost_not_provisional() {
    let (found, score) = landing_with_bound_cost(athena_score::RulesVersion::V1, |_| {});
    assert_eq!(score.rules, athena_score::RULES_V1);
    assert!(!bound_cost_is_provisional(&score));
    assert_eq!(found, vec![]);
}

#[test]
fn a_score_written_under_the_current_rules_marks_a_bound_cost_provisional() {
    let (found, score) = landing_with_bound_cost(athena_score::RulesVersion::CURRENT, |_| {});
    assert_eq!(score.rules, athena_score::RULES);
    assert!(bound_cost_is_provisional(&score));
    assert_eq!(found, vec![]);
}

#[test]
fn a_score_labelled_v1_but_computed_under_v2_is_refused() {
    let (found, _) = landing_with_bound_cost(athena_score::RulesVersion::V2, |s| {
        s.rules = athena_score::RULES_V1.to_string();
    });
    assert!(
        found
            .iter()
            .any(|v| v.rule == Rule::JobRecords && v.detail.contains("does not follow")),
        "{found:?}"
    );
}

#[test]
fn a_score_under_an_unknown_rules_version_is_refused() {
    let (found, _) = landing_with_bound_cost(athena_score::RulesVersion::CURRENT, |s| {
        s.rules = "brier events; v99".to_string();
    });
    assert!(
        found
            .iter()
            .any(|v| v.rule == Rule::JobRecords && v.detail.contains("cannot recompute")),
        "{found:?}"
    );
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

// ---- A squash that lands a chain of jobs (design D69, docs/design/squash-chain-landing.md) ----

const CHAIN_IDS: [&str; 3] = [JOB, "20261010-160000-bbbb", "20261010-170000-cccc"];
const ORIGINAL: &str = "pub fn a() {}\n";
const STEP_1: &str = "pub fn a() { /* step 1 */ }\n";
const STEP_2: &str = "pub fn a() { /* step 2 */ }\n";
const STEP_3: &str = "pub fn a() { /* step 3 */ }\n";

/// One job of a hand-built chain: the fingerprints its `athena.candidate/2` records.
#[derive(Clone)]
pub(crate) struct Link {
    pub id: &'static str,
    pub base_fp: String,
    pub prev: Option<&'static str>,
    pub fp: String,
}

/// A repository whose working tree holds the last of `codes`, and one [`Link`] per code: job `i` takes the
/// content from `codes[i - 1]` (the repository's original code for the first) to `codes[i]`. Nothing is
/// committed yet.
pub(crate) fn chain_repo(codes: &[&str]) -> (tempfile::TempDir, Git, Vec<Link>) {
    let (dir, git) = owner_repo();
    let base = git.head().unwrap().unwrap();
    let mut fps = vec![fingerprint(&git, &base, &base).unwrap()];
    for code in codes {
        write(dir.path(), &[("src/lib.rs", code)]);
        fps.push(landed_fingerprint(&git, &base));
    }
    let links = (0..codes.len())
        .map(|i| Link {
            id: CHAIN_IDS[i],
            base_fp: fps[i].clone(),
            prev: (i > 0).then(|| CHAIN_IDS[i - 1]),
            fp: fps[i + 1].clone(),
        })
        .collect();
    (dir, git, links)
}

/// The records of one job of a chain.
fn chain_records(link: &Link) -> Vec<AnyRecord> {
    let mut records = opening(&HONEST);
    records.extend(closing(&link.fp));
    records
        .into_iter()
        .map(|record| {
            let text = record.to_json().replace(JOB, link.id);
            let record = AnyRecord::parse("record", text.as_bytes()).unwrap();
            match record {
                AnyRecord::Candidate(mut c) => {
                    c.base_fingerprint = Some(link.base_fp.clone());
                    c.previous_job = link.prev.map(|p| JobId::parse(p).unwrap());
                    AnyRecord::Candidate2(c)
                }
                other => other,
            }
        })
        .collect()
}

/// Writes `code` and the records of `links` (each changed by `tweak`, given its index), and commits them
/// as one squash. Returns the commit.
pub(crate) fn land_chain(
    root: &std::path::Path,
    git: &Git,
    links: &[Link],
    code: &str,
    tweak: &dyn Fn(usize, &mut Vec<AnyRecord>),
) -> String {
    write(root, &[("src/lib.rs", code)]);
    for (i, link) in links.iter().enumerate() {
        let mut records = chain_records(link);
        tweak(i, &mut records);
        write_records(root, &records);
    }
    commit(git, "Land the chain (#1)\n")
}

fn untouched(_: usize, _: &mut Vec<AnyRecord>) {}

/// The only violation in `found`, which must be an invariant violation whose detail holds `wanted`.
fn only_refusal(found: Vec<Violation>, wanted: &str) -> Violation {
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].rule, Rule::Invariant);
    assert!(
        found[0].detail.contains(wanted),
        "wanted {wanted:?} in {:?}",
        found[0].detail
    );
    found.into_iter().next().unwrap()
}

fn dir_of(id: &str) -> String {
    JobId::parse(id).unwrap().dir()
}

fn job_ids(ids: &[&str]) -> std::collections::BTreeSet<JobId> {
    ids.iter().map(|i| JobId::parse(i).unwrap()).collect()
}

fn landed_by(git: &Git, rev: &str) -> std::collections::BTreeSet<JobId> {
    let id = git.rev_parse(rev).unwrap().unwrap();
    let change = Change::of_commit(git, &id).unwrap();
    let judged = Judged::new(git, &change).unwrap();
    invariant::landing_job(git, &change, &judged, None).unwrap()
}

fn job_records_of(git: &Git, rev: &str) -> Vec<Violation> {
    let id = git.rev_parse(rev).unwrap().unwrap();
    let change = Change::of_commit(git, &id).unwrap();
    let judged = Judged::new(git, &change).unwrap();
    crate::jobs::check_records(git, &change, &judged, None).unwrap()
}

#[test]
fn a_hand_built_two_job_chain_squash_lands() {
    let (dir, git, links) = chain_repo(&[STEP_1, STEP_2]);
    land_chain(dir.path(), &git, &links, STEP_2, &untouched);
    assert_eq!(judge(&git, "HEAD"), vec![]);
    assert_eq!(job_records_of(&git, "HEAD"), vec![]);
    assert_eq!(history_of(&git), vec![]);
    assert_eq!(landed_by(&git, "HEAD"), job_ids(&CHAIN_IDS[..2]));
}

#[test]
fn a_three_job_chain_squash_lands() {
    let (dir, git, links) = chain_repo(&[STEP_1, STEP_2, STEP_3]);
    land_chain(dir.path(), &git, &links, STEP_3, &untouched);
    assert_eq!(judge(&git, "HEAD"), vec![]);
    assert_eq!(job_records_of(&git, "HEAD"), vec![]);
    assert_eq!(landed_by(&git, "HEAD"), job_ids(&CHAIN_IDS));
}

#[test]
fn chains_with_repeated_fingerprints_land() {
    // A to B, B to A, A to C: the second job reverts the first.
    let (dir, git, links) = chain_repo(&[STEP_1, ORIGINAL, STEP_3]);
    assert_eq!(links[0].base_fp, links[1].fp, "the fingerprint repeats");
    land_chain(dir.path(), &git, &links, STEP_3, &untouched);
    assert_eq!(judge(&git, "HEAD"), vec![]);
    assert_eq!(job_records_of(&git, "HEAD"), vec![]);
    assert_eq!(history_of(&git), vec![]);

    // A job that changes no governed content: its base and its result are the same.
    let (dir, git, links) = chain_repo(&[STEP_1, STEP_1, STEP_3]);
    assert_eq!(links[1].base_fp, links[1].fp);
    land_chain(dir.path(), &git, &links, STEP_3, &untouched);
    assert_eq!(judge(&git, "HEAD"), vec![]);
    assert_eq!(job_records_of(&git, "HEAD"), vec![]);
    assert_eq!(history_of(&git), vec![]);
}

/// Breaks a three-job chain with `edit`, which may change the links and the records, lands it, and
/// returns the one refusal.
fn refused_chain(
    edit: impl Fn(&mut Vec<Link>),
    tweak: &dyn Fn(usize, &mut Vec<AnyRecord>),
    wanted: &str,
) -> Violation {
    let (dir, git, mut links) = chain_repo(&[STEP_1, STEP_2, STEP_3]);
    edit(&mut links);
    land_chain(dir.path(), &git, &links, STEP_3, tweak);
    assert_eq!(landed_by(&git, "HEAD"), job_ids(&[]));
    only_refusal(judge(&git, "HEAD"), wanted)
}

#[test]
fn a_candidate_1_in_a_chain_is_refused() {
    let to_v1 = |i: usize, records: &mut Vec<AnyRecord>| {
        if i == 1 {
            for r in records.iter_mut() {
                if let AnyRecord::Candidate2(c) = r {
                    let mut c = c.clone();
                    c.base_fingerprint = None;
                    c.previous_job = None;
                    *r = AnyRecord::Candidate(c);
                }
            }
        }
    };
    let v = refused_chain(
        |_| {},
        &to_v1,
        &format!(
            "a chain landing needs athena.candidate/2 in {}/candidate.json",
            dir_of(CHAIN_IDS[1])
        ),
    );
    assert!(v.detail.starts_with("a chain landing needs"));
}

#[test]
fn a_chain_without_a_head_or_with_a_wrong_head_is_refused() {
    let no_job_starts = "no job in the commit starts from the parent's content";
    // The head's base is not the parent's content.
    refused_chain(|l| l[0].base_fp = "e".repeat(64), &untouched, no_job_starts);
    // Nothing is first: the jobs follow each other in a circle.
    refused_chain(
        |l| l[0].prev = Some(CHAIN_IDS[2]),
        &untouched,
        no_job_starts,
    );
}

#[test]
fn a_forked_chain_is_refused() {
    refused_chain(
        |l| l[2].prev = Some(CHAIN_IDS[0]),
        &untouched,
        &format!(
            "jobs {} and {} both follow {}, so they do not form one chain",
            CHAIN_IDS[1], CHAIN_IDS[2], CHAIN_IDS[0]
        ),
    );
    // Two jobs that both start a chain.
    refused_chain(
        |l| l[1].prev = None,
        &untouched,
        "both start a chain, so they do not form one chain",
    );
}

/// Marks the outcome of the job at index 0 as rejected.
fn reject_first(i: usize, records: &mut [AnyRecord]) {
    if i == 0 {
        for r in records.iter_mut() {
            if let AnyRecord::Outcome(o) = r {
                o.state = OutcomeState::Rejected;
            }
        }
    }
}

#[test]
fn a_rejected_job_its_landed_retry_and_a_follow_up_land_as_a_chain() {
    // A (rejected), B retries A from the parent's content, C follows B.
    let (dir, git, mut links) = chain_repo(&[STEP_1, STEP_2, STEP_3]);
    links[1].base_fp = links[0].base_fp.clone();
    land_chain(dir.path(), &git, &links, STEP_3, &|i, r| reject_first(i, r));
    assert_eq!(judge(&git, "HEAD"), vec![]);
    assert_eq!(landed_by(&git, "HEAD"), job_ids(&CHAIN_IDS[1..]));
}

#[test]
fn a_rejected_job_does_not_hide_a_fork_of_landed_jobs() {
    // Two landed retries of the same rejected job are still two followers of it.
    let (dir, git, mut links) = chain_repo(&[STEP_1, STEP_2, STEP_3]);
    links[1].base_fp = links[0].base_fp.clone();
    links[2].prev = Some(CHAIN_IDS[0]);
    links[2].base_fp = links[0].base_fp.clone();
    land_chain(dir.path(), &git, &links, STEP_3, &|i, r| reject_first(i, r));
    only_refusal(judge(&git, "HEAD"), "both follow");

    // Landed jobs that really fork are still refused.
    refused_chain(
        |l| l[2].prev = Some(CHAIN_IDS[0]),
        &untouched,
        "both follow",
    );
}

#[test]
fn an_unlinked_or_missing_middle_job_is_refused() {
    // A leftover job whose predecessor is not in the commit.
    refused_chain(
        |l| l[1].prev = Some("20261010-120000-dead"),
        &untouched,
        &format!(
            "job {} is not linked to the others: its previous job 20261010-120000-dead is not in this commit",
            CHAIN_IDS[1]
        ),
    );
    // A cycle that the head does not reach.
    refused_chain(
        |l| {
            l[1].prev = Some(CHAIN_IDS[2]);
            l[2].prev = Some(CHAIN_IDS[1]);
        },
        &untouched,
        "is not linked to the others",
    );
    // The middle job is missing, though its neighbours' fingerprints still link.
    let (dir, git, mut links) = chain_repo(&[STEP_1, STEP_2, STEP_3]);
    assert_eq!(links[1].fp, links[2].base_fp);
    links.remove(1);
    land_chain(dir.path(), &git, &links, STEP_3, &untouched);
    only_refusal(
        judge(&git, "HEAD"),
        &format!(
            "job {} is not linked to the others: its previous job {} is not in this commit",
            CHAIN_IDS[2], CHAIN_IDS[1]
        ),
    );
}

#[test]
fn a_broken_fingerprint_link_is_refused() {
    let stale = "e".repeat(64);
    let v = refused_chain(
        |l| l[1].base_fp = stale.clone(),
        &untouched,
        &format!(
            "job {} starts from {stale} but its previous job {} ended at",
            CHAIN_IDS[1], CHAIN_IDS[0]
        ),
    );
    assert!(v.detail.contains("ended at"));
}

#[test]
fn a_last_job_that_does_not_match_the_tree_is_refused() {
    let elsewhere = "d".repeat(64);
    refused_chain(
        |l| l[2].fp = elsewhere.clone(),
        &untouched,
        &format!(
            "the last job {} holds {elsewhere}, not the landed content",
            CHAIN_IDS[2]
        ),
    );
    // The candidate holds the tree but the verification verified something else.
    let other = "9".repeat(64);
    let verified_elsewhere = |i: usize, records: &mut Vec<AnyRecord>| {
        if i == 2 {
            for r in records.iter_mut() {
                if let AnyRecord::Verification(v) = r {
                    v.verified_fingerprint = other.clone();
                }
            }
        }
    };
    refused_chain(
        |_| {},
        &verified_elsewhere,
        &format!("verified {other}, not the landed content"),
    );
}

#[test]
fn a_chain_that_changes_a_classification_rules_file_is_refused() {
    // A job records a .gitattributes among its changes.
    let records_it = |i: usize, records: &mut Vec<AnyRecord>| {
        if i == 1 {
            for r in records.iter_mut() {
                if let AnyRecord::Candidate2(c) = r {
                    c.changes.push(athena_data::records::ChangeEntry {
                        path: "docs/.gitattributes".into(),
                        status: athena_data::records::ChangeStatus::Added,
                        mode: None,
                        bytes: None,
                    });
                }
            }
        }
    };
    refused_chain(
        |_| {},
        &records_it,
        &format!(
            "job {} changes a path classification rules file, so its fingerprints are not comparable; land it alone",
            CHAIN_IDS[1]
        ),
    );
    // The squash itself touches one, whatever the records say.
    let (dir, git, links) = chain_repo(&[STEP_1, STEP_2]);
    write(dir.path(), &[("docs/.gitattributes", "# a note\n")]);
    land_chain(dir.path(), &git, &links, STEP_2, &untouched);
    only_refusal(
        judge(&git, "HEAD"),
        "docs/.gitattributes, a path classification rules file",
    );
}

#[test]
fn a_chain_on_a_commit_with_two_parents_is_refused() {
    let (dir, git, links) = chain_repo(&[STEP_1, STEP_2]);
    git.run(["checkout", "-q", "-b", "pr"]).unwrap();
    land_chain(dir.path(), &git, &links, STEP_2, &untouched);
    git.run(["checkout", "-q", "main"]).unwrap();
    write(dir.path(), &[("src/other.rs", "pub fn b() {}\n")]);
    commit(&git, "other work\n\nAthena-Job: bootstrap\n");
    git.run(["merge", "-q", "--no-ff", "-m", "merge pr", "pr"])
        .unwrap();
    // The refusal's own message is not shown for a merge: `check` reports the merge message (design §4.6).
    assert_eq!(landed_by(&git, "HEAD"), job_ids(&[]));
    only_refusal(
        judge(&git, "HEAD"),
        "a merge introduces governed content that neither parent has",
    );
    // The same chain on a single-parent commit lands.
    let (dir, git, links) = chain_repo(&[STEP_1, STEP_2]);
    land_chain(dir.path(), &git, &links, STEP_2, &untouched);
    assert_eq!(judge(&git, "HEAD"), vec![]);
}

#[test]
fn a_partly_landed_chain_is_judged_by_the_jobs_left() {
    // One job left: the single-job rule, so even a stale base_fingerprint does not matter.
    let (dir, git, mut links) = chain_repo(&[STEP_1, STEP_2]);
    land_chain(dir.path(), &git, &links[..1], STEP_1, &untouched);
    assert_eq!(judge(&git, "HEAD"), vec![]);
    links[1].base_fp = "e".repeat(64);
    land_chain(dir.path(), &git, &links[1..], STEP_2, &untouched);
    assert_eq!(judge(&git, "HEAD"), vec![]);
    assert_eq!(landed_by(&git, "HEAD"), job_ids(&CHAIN_IDS[1..2]));
    assert_eq!(history_of(&git), vec![]);

    // Two left, with a landed predecessor.
    let (dir, git, links) = chain_repo(&[STEP_1, STEP_2, STEP_3]);
    land_chain(dir.path(), &git, &links[..1], STEP_1, &untouched);
    land_chain(dir.path(), &git, &links[1..], STEP_3, &untouched);
    assert_eq!(judge(&git, "HEAD"), vec![]);
    assert_eq!(job_records_of(&git, "HEAD"), vec![]);
    assert_eq!(landed_by(&git, "HEAD"), job_ids(&CHAIN_IDS[1..]));
    assert_eq!(history_of(&git), vec![]);

    // The same, though the predecessor is an athena.candidate/1 record.
    let to_v1 = |_: usize, records: &mut Vec<AnyRecord>| {
        for r in records.iter_mut() {
            if let AnyRecord::Candidate2(c) = r {
                let mut c = c.clone();
                c.base_fingerprint = None;
                c.previous_job = None;
                *r = AnyRecord::Candidate(c);
            }
        }
    };
    let (dir, git, links) = chain_repo(&[STEP_1, STEP_2, STEP_3]);
    land_chain(dir.path(), &git, &links[..1], STEP_1, &to_v1);
    land_chain(dir.path(), &git, &links[1..], STEP_3, &untouched);
    assert_eq!(judge(&git, "HEAD"), vec![]);
    assert_eq!(history_of(&git), vec![]);

    // Two left, but the predecessor's records are not on the parent: not linked.
    let (dir, git, links) = chain_repo(&[STEP_1, STEP_2, STEP_3]);
    land_chain(dir.path(), &git, &links[1..], STEP_3, &untouched);
    only_refusal(judge(&git, "HEAD"), "is not linked to the others");
}

/// Edits the records of the middle job of a three-job chain, lands the chain, and returns the refusal.
fn refused_middle(edit: impl Fn(&mut AnyRecord), wanted: &str) {
    let tweak = |i: usize, records: &mut Vec<AnyRecord>| {
        if i == 1 {
            records.iter_mut().for_each(&edit);
        }
    };
    let v = refused_chain(|_| {}, &tweak, wanted);
    assert!(v.detail.contains(CHAIN_IDS[1]), "{}", v.detail);
}

#[test]
fn every_per_job_check_refuses_a_chain_when_a_middle_job_fails_it() {
    // No accepted reading's job.json.
    let (dir, git, links) = chain_repo(&[STEP_1, STEP_2, STEP_3]);
    let no_job = |i: usize, records: &mut Vec<AnyRecord>| {
        if i == 1 {
            records.retain(|r| !matches!(r, AnyRecord::Job(_)));
        }
    };
    land_chain(dir.path(), &git, &links, STEP_3, &no_job);
    only_refusal(judge(&git, "HEAD"), "job.json is not in the landed tree");

    // A reading written after the job's own work.
    refused_middle(
        |r| {
            if let AnyRecord::Reading(x) = r {
                x.at = ts("2026-10-10T15:04:00Z");
            }
        },
        "after the job was dispatched",
    );
    // A failing final verdict.
    refused_middle(
        |r| {
            if let AnyRecord::Verification(v) = r {
                v.final_verdict = Verdict::Fail;
            }
        },
        "a final verdict of fail",
    );
    // Different candidate commits.
    refused_middle(
        |r| {
            if let AnyRecord::Verification(v) = r {
                v.candidate_commit = "d".repeat(40);
            }
        },
        "different candidate commits",
    );
    // The candidate and the verification disagree, though each job's link would hold.
    refused_middle(
        |r| {
            if let AnyRecord::Verification(v) = r {
                v.verified_fingerprint = "9".repeat(64);
            }
        },
        "not the content",
    );
    // A verifier invocation that failed, and one that used the wrong model.
    refused_middle(
        |r| {
            if let AnyRecord::Invocation(i) = r
                && i.role == Role::Verifier
            {
                i.outcome = InvocationOutcome::Error;
            }
        },
        "is not a successful verifier invocation",
    );
    refused_middle(
        |r| {
            if let AnyRecord::Invocation(i) = r
                && i.role == Role::Verifier
            {
                i.model = PRODUCER.into();
            }
        },
        "not the job's verifier model",
    );
    // Usage that does not reconcile.
    refused_middle(
        |r| {
            if let AnyRecord::Invocation(i) = r
                && i.role == Role::Verifier
            {
                i.cost = Quantity::reported(9.0, Unit::Usd, "Claude Code total_cost_usd");
            }
        },
        "per-model costs sum to",
    );
    // No outcome at all: the job claims a landing it has no landed outcome for. (A job whose recorded
    // outcome is not landed is not part of the chain; see the rejected-job tests.)
    let (dir, git, links) = chain_repo(&[STEP_1, STEP_2, STEP_3]);
    let no_outcome = |i: usize, records: &mut Vec<AnyRecord>| {
        if i == 1 {
            records.retain(|r| !matches!(r, AnyRecord::Outcome(_)));
        }
    };
    land_chain(dir.path(), &git, &links, STEP_3, &no_outcome);
    only_refusal(judge(&git, "HEAD"), "a landing commit adds");
}

#[test]
fn a_middle_job_that_already_landed_is_refused_in_a_history_walk() {
    let (dir, git, links) = chain_repo(&[STEP_1, STEP_2, STEP_3]);
    land_chain(dir.path(), &git, &links, STEP_3, &untouched);
    let id = git.rev_parse("HEAD").unwrap().unwrap();
    let change = Change::of_commit(&git, &id).unwrap();
    let judged = Judged::new(&git, &change).unwrap();
    let mut state = invariant::HistoryState::default();
    state.landed.insert(CHAIN_IDS[1].to_string());
    let found = invariant::check(&git, &change, &judged, Some(&state)).unwrap();
    only_refusal(found, &format!("job {} has already landed", CHAIN_IDS[1]));
}

// ---- Records written by hand (design §16.2): provenance ----
//
// Records are not signed, so the gate cannot tell a careful forgery from the real thing. It can tell a
// job whose records lack, copy or contradict the links Athena's own writers leave: the request and the
// reader invocation before a reading, the dispatch fields and the producer brief, and the order of times.

/// The `defaults.toml` the genuine repository holds; a reading records the hash of these bytes.
const DEFAULTS_TEXT: &str = "version = 1\n";
/// The producer brief `athena go` commits with `job.json`.
const BRIEF: &str = "# Brief\n\nChange a.\n";

/// The repository `athena ask` reads: the owner repository plus a committed `defaults.toml`.
fn genuine_repo() -> (tempfile::TempDir, Git) {
    repo(&[
        (
            ".gitattributes",
            "/OWNER.md athena-exclude\n/SPEC.md athena-readonly\n",
        ),
        ("src/lib.rs", "pub fn a() {}\n"),
        ("defaults.toml", DEFAULTS_TEXT),
    ])
}

/// Every record of one job, in the shape Athena writes it, ready to be changed by a test.
struct Parts {
    request: Option<Request>,
    reading: Reading,
    reader: Option<Invocation>,
    job: Job,
    /// The text of `brief-producer.md`, or None when the file is not committed.
    brief: Option<String>,
    producer: Invocation,
    verifier: Invocation,
    outcome: Outcome,
}

impl Parts {
    /// The records of a genuine job read at `base`.
    fn genuine(base: &str) -> Parts {
        let mut reading = reading(&HONEST);
        reading.base_commit = base.to_string();
        reading.defaults_sha256 = athena_data::sha256_hex(DEFAULTS_TEXT.as_bytes());
        let AnyRecord::Job(mut job) = job_record(&HONEST) else {
            unreachable!("job_record makes a job")
        };
        job.caps = Some(Caps {
            job_usd: 3.0,
            reader_usd: 0.5,
            producer_usd: 1.5,
            verifier_usd: 1.0,
            producer_timeout_s: 1800,
            verifier_timeout_s: 900,
            check_timeout_s: 600,
        });
        job.base_commit = Some(base.to_string());
        job.config_sha256 = Some(reading.config_sha256.clone());
        job.defaults_sha256 = Some(reading.defaults_sha256.clone());
        job.athena = Some(Build {
            version: "0.1.0".into(),
            source_commit: "0".repeat(40),
            dirty: false,
        });
        job.driver = Some(DriverInfo {
            name: "claude-code".into(),
            version: "2.1.299".into(),
        });
        job.brief_sha256 = Some(athena_data::sha256_hex(BRIEF.as_bytes()));
        Parts {
            request: Some(Request {
                job: job_id(),
                text: "Change a.".into(),
                received_at: ts("2026-10-10T14:59:00Z"),
                channel: "cli".into(),
            }),
            reading,
            reader: Some(invocation(
                1,
                Role::Reader,
                PRODUCER,
                "2026-10-10T15:00:00Z",
                "2026-10-10T15:01:00Z",
            )),
            job: *job,
            brief: Some(BRIEF.to_string()),
            producer: invocation(
                2,
                Role::Producer,
                PRODUCER,
                "2026-10-10T15:03:00Z",
                "2026-10-10T15:05:00Z",
            ),
            verifier: invocation(
                3,
                Role::Verifier,
                VERIFIER,
                "2026-10-10T15:06:00Z",
                "2026-10-10T15:07:00Z",
            ),
            outcome: outcome(),
        }
    }

    /// The records `athena ask` commits: the request, the reader's invocation and the reading.
    fn asked(&self) -> Vec<AnyRecord> {
        let mut records = Vec::new();
        records.extend(self.request.iter().cloned().map(AnyRecord::Request));
        records.extend(
            self.reader
                .iter()
                .cloned()
                .map(|i| AnyRecord::Invocation(Box::new(i))),
        );
        records.push(AnyRecord::Reading(Box::new(self.reading.clone())));
        records
    }

    /// The records `athena go` commits when it dispatches: `job.json`.
    fn dispatched(&self) -> Vec<AnyRecord> {
        vec![AnyRecord::Job(Box::new(self.job.clone()))]
    }

    fn write_brief(&self, root: &std::path::Path) {
        if let Some(brief) = &self.brief {
            let path = format!("{}/brief-producer.md", job_id().dir());
            write(root, &[(&path, brief)]);
        }
    }
}

/// A job landed by commits in the order Athena makes them, each record taken from `Parts::genuine` and
/// changed by `tweak`. With `two_commits`, `job.json` arrives with the reading (as a person would write a
/// whole job in two goes); otherwise the commits are `athena ask`, `athena go` and the landing. Returns
/// the repository; `HEAD` is the landing.
fn landed_job(tweak: impl FnOnce(&mut Parts), two_commits: bool) -> (tempfile::TempDir, Git) {
    let (dir, git) = genuine_repo();
    let root = dir.path();
    let base = git.head().unwrap().unwrap();
    let mut parts = Parts::genuine(&base);
    tweak(&mut parts);
    let trailer = format!("Athena-Job: {JOB}\n");
    write_records(root, &parts.asked());
    let prediction = if two_commits {
        write_records(root, &parts.dispatched());
        parts.write_brief(root);
        commit(&git, &format!("Read a request: t\n\n{trailer}"))
    } else {
        commit(&git, &format!("Read a request: t\n\n{trailer}"));
        write_records(root, &parts.dispatched());
        parts.write_brief(root);
        commit(&git, &format!("Dispatch: t\n\n{trailer}"))
    };
    write(root, &[("src/lib.rs", NEW_CODE)]);
    let fp = landed_fingerprint(&git, &base);
    write_records(
        root,
        &[
            AnyRecord::Invocation(Box::new(parts.producer.clone())),
            AnyRecord::Invocation(Box::new(parts.verifier.clone())),
            AnyRecord::Candidate(Box::new(candidate(&fp))),
            AnyRecord::Verification(Box::new(verification(&fp))),
            AnyRecord::Outcome(Box::new(parts.outcome.clone())),
        ],
    );
    commit(
        &git,
        &format!(
            "Land: t\n\n{trailer}Athena-Prediction: {prediction}\nAthena-Verified-Content: {fp}\n"
        ),
    );
    (dir, git)
}

/// The provenance violations of the commit at `rev`.
fn provenance_of(git: &Git, rev: &str) -> Vec<Violation> {
    let id = git.rev_parse(rev).unwrap().unwrap();
    let change = Change::of_commit(git, &id).unwrap();
    let judged = Judged::new(git, &change).unwrap();
    crate::jobs::provenance(git, &change, &judged, None).unwrap()
}

/// The violations the commit-msg hook gives the commit at `rev`.
fn gate_of(git: &Git, rev: &str) -> Vec<Violation> {
    let id = git.rev_parse(rev).unwrap().unwrap();
    let change = Change::of_commit(git, &id).unwrap();
    crate::gate(git, &change, Scope::CommitMsg).unwrap()
}

/// Asserts that `found` holds a job-records violation that starts with the provenance prefix, names the
/// missing or contradicting evidence (`detail`), carries its commit, and names the command that repairs it.
fn assert_refused(found: &[Violation], detail: &str, repair: &str) {
    let v = found
        .iter()
        .find(|v| v.detail.contains(detail))
        .unwrap_or_else(|| panic!("no violation names {detail:?}: {found:#?}"));
    assert_eq!(v.rule, Rule::JobRecords);
    assert!(
        v.detail.starts_with(crate::jobs::PROVENANCE),
        "{}",
        v.detail
    );
    assert!(v.repair.contains(repair), "{}", v.repair);
    assert!(v.commit.is_some());
}

/// Makes the reading, its request and the reader invocation older than `copied_verifier`, the way a
/// forger who copies records from an earlier job has them.
fn from_an_earlier_job(p: &mut Parts) {
    p.request.as_mut().unwrap().received_at = ts("2026-10-09T09:00:00Z");
    let reader = p.reader.as_mut().unwrap();
    reader.started_at = ts("2026-10-09T09:00:10Z");
    reader.ended_at = ts("2026-10-09T09:01:00Z");
    p.reading.at = ts("2026-10-09T09:01:30Z");
    p.verifier = copied_verifier();
}

/// A verifier invocation copied from a job run before this one was dispatched.
fn copied_verifier() -> Invocation {
    invocation(
        3,
        Role::Verifier,
        VERIFIER,
        "2026-10-09T10:06:00Z",
        "2026-10-09T10:07:00Z",
    )
}

#[test]
fn a_genuine_landing_is_not_refused() {
    // The shape Athena writes: `athena ask`, `athena go`, then the landing, each a commit of its own.
    let (_dir, git) = landed_job(|_| {}, false);
    for rev in ["HEAD~2", "HEAD~1", "HEAD"] {
        assert_eq!(provenance_of(&git, rev), vec![], "at {rev}");
        assert_eq!(gate_of(&git, rev), vec![], "at {rev}");
    }
    assert_eq!(judge(&git, "HEAD"), vec![]);
    let walked = history_of(&git);
    assert!(
        !walked
            .iter()
            .any(|v| v.detail.starts_with(crate::jobs::PROVENANCE)),
        "{walked:?}"
    );
}

#[test]
fn a_job_written_by_hand_across_two_commits_is_refused() {
    // Acceptance review 2's forgery: the request, a reading, a reader invocation and a `job.json` in one
    // commit; the code, candidate, verification, a verifier invocation copied from an earlier job and a
    // landed outcome in the next. The records are consistent with each other apart from what follows.
    let forged = |p: &mut Parts| {
        from_an_earlier_job(p);
        p.brief = None;
    };
    let (_dir, git) = landed_job(forged, true);
    // The first commit closes nothing, so it is not judged for provenance.
    assert_eq!(provenance_of(&git, "HEAD~1"), vec![]);
    // The gate without provenance accepts the second commit.
    assert_eq!(judge(&git, "HEAD"), vec![]);
    assert_eq!(job_records_of(&git, "HEAD"), vec![]);
    let found = provenance_of(&git, "HEAD");
    assert_refused(&found, "brief-producer.md", "athena go");
    assert_refused(&found, "before the job was dispatched", "athena go");
    assert_refused(&found, "before the producer invocation ended", "athena go");
    // The commit-msg hook refuses it.
    assert!(
        gate_of(&git, "HEAD")
            .iter()
            .any(|v| v.detail.starts_with(crate::jobs::PROVENANCE))
    );

    // A brief_sha256 copied from another job, and no brief committed with it.
    let (_dir, git) = landed_job(
        |p| {
            from_an_earlier_job(p);
            p.brief = None;
            p.job.brief_sha256 = Some("d".repeat(64));
        },
        true,
    );
    assert_refused(
        &provenance_of(&git, "HEAD"),
        "brief-producer.md is not in the tree",
        "athena go",
    );

    // A matching brief leaves only the copied times.
    let (_dir, git) = landed_job(from_an_earlier_job, true);
    let found = provenance_of(&git, "HEAD");
    assert_refused(&found, "before the job was dispatched", "athena go");
    assert!(
        !found.iter().any(|v| v.detail.contains("brief-producer.md")),
        "{found:?}"
    );

    // No request record: nothing shows that a request was received.
    let (_dir, git) = landed_job(
        |p| {
            from_an_earlier_job(p);
            p.request = None;
        },
        true,
    );
    assert_refused(
        &provenance_of(&git, "HEAD"),
        "has no request record",
        "athena ask",
    );
}

#[test]
fn a_job_written_by_hand_with_every_link_consistent_still_passes() {
    // The recorded limit: records are not signed, so a forger who writes every link in order, with a
    // brief whose hash matches, is not refused.
    let (_dir, git) = landed_job(|_| {}, true);
    assert_eq!(provenance_of(&git, "HEAD~1"), vec![]);
    assert_eq!(provenance_of(&git, "HEAD"), vec![]);
}

#[test]
fn a_forged_landing_of_a_dispatched_job_that_never_closed_is_refused() {
    // A job that was read and dispatched genuinely (two commits) and then closed by one forged commit.
    // Each variant breaks one link the forger has to mimic.
    type Break = Box<dyn Fn(&mut Parts)>;
    let cases: Vec<(&str, Break, &str)> = vec![
        (
            "a producer started before the dispatch",
            Box::new(|p| p.producer.started_at = ts("2026-10-10T15:01:50Z")),
            "before the job was dispatched",
        ),
        (
            "a verifier started before the producer ended",
            Box::new(|p| p.verifier.started_at = ts("2026-10-10T15:04:00Z")),
            "before the producer invocation ended",
        ),
        (
            "an outcome recorded before the verifier ended",
            Box::new(|p| p.outcome.at = ts("2026-10-10T15:06:30Z")),
            "before the verifier invocation ended",
        ),
        (
            "a job.json with no dispatch time",
            Box::new(|p| p.job.dispatched_at = None),
            "lacks dispatched_at",
        ),
        (
            "a job.json with another defaults hash than the reading's",
            Box::new(|p| p.job.defaults_sha256 = Some("c".repeat(64))),
            "differs from the accepted reading's",
        ),
        (
            "a job.json with another config hash than the reading's",
            Box::new(|p| p.job.config_sha256 = Some("c".repeat(64))),
            "differs from the accepted reading's",
        ),
        (
            "a brief that is not the one job.json hashes",
            Box::new(|p| p.brief = Some("Another brief.\n".into())),
            "brief-producer.md hashes to",
        ),
        (
            "no brief at all",
            Box::new(|p| p.brief = None),
            "brief-producer.md is not in the tree",
        ),
        (
            "no job.json",
            Box::new(|p| p.job.id = JobId::parse("20261010-153012-zzzz").unwrap()),
            "has no job.json",
        ),
    ];
    for (name, tweak, wanted) in cases {
        let (_dir, git) = landed_job(tweak, false);
        // The first two commits are the genuine `ask` and `go`.
        assert_eq!(provenance_of(&git, "HEAD~2"), vec![], "{name}");
        assert_eq!(provenance_of(&git, "HEAD~1"), vec![], "{name}");
        let found = provenance_of(&git, "HEAD");
        assert_refused(&found, wanted, "athena go");
    }
}

#[test]
fn a_reading_recorded_after_the_dispatch_is_refused_by_the_gate() {
    // The existing prediction rule owns this fault, and the commit-msg hook runs it beside provenance;
    // provenance does not repeat it.
    let (_dir, git) = landed_job(|p| p.reading.at = ts("2026-10-10T15:02:30Z"), false);
    assert_eq!(provenance_of(&git, "HEAD"), vec![]);
    let found = gate_of(&git, "HEAD");
    assert!(
        found
            .iter()
            .any(|v| v.rule == Rule::Invariant && v.detail.contains("after the job was dispatched")),
        "{found:?}"
    );
}

#[test]
fn a_forged_landing_of_a_dispatched_job_written_with_current_times_still_passes() {
    // The recorded limit: a forger who stamps the closing records after the dispatch, in order, and copies
    // nothing from another job, leaves no contradiction for the check to find.
    let (_dir, git) = landed_job(
        |p| {
            p.producer.started_at = ts("2026-11-01T10:00:00Z");
            p.producer.ended_at = ts("2026-11-01T10:05:00Z");
            p.verifier.started_at = ts("2026-11-01T10:06:00Z");
            p.verifier.ended_at = ts("2026-11-01T10:07:00Z");
            p.outcome.at = ts("2026-11-01T10:08:00Z");
        },
        false,
    );
    assert_eq!(provenance_of(&git, "HEAD"), vec![]);
}

/// A proposal as `athena ask` commits it and `athena close` closes it: a score and a dropped outcome.
/// `tweak` changes the records first. Returns the repository; `HEAD` is the close.
fn closed_proposal(tweak: impl FnOnce(&mut Parts)) -> (tempfile::TempDir, Git) {
    let (dir, git) = genuine_repo();
    let root = dir.path();
    let base = git.head().unwrap().unwrap();
    let mut parts = Parts::genuine(&base);
    tweak(&mut parts);
    write_records(root, &parts.asked());
    commit(&git, &format!("Read a request: t\n\nAthena-Job: {JOB}\n"));
    let dropped = Outcome {
        state: OutcomeState::Dropped,
        reason: "closed: not wanted".into(),
        at: ts("2026-10-10T15:09:00Z"),
        totals: None,
        ..outcome()
    };
    write_records(root, &[AnyRecord::Outcome(Box::new(dropped))]);
    git.run(["add", "-A"]).unwrap();
    let tree = git.write_index_tree().unwrap();
    let score = crate::jobs::expected_score(
        &git,
        &tree,
        &job_id(),
        None,
        ts("2026-10-10T15:09:00Z"),
        athena_score::RulesVersion::CURRENT,
    )
    .unwrap();
    write_records(root, &[AnyRecord::Score(Box::new(score))]);
    commit(&git, &format!("Dropped: t\n\nAthena-Job: {JOB}\n"));
    (dir, git)
}

#[test]
fn a_hand_made_proposal_closed_with_athena_close_is_refused() {
    // A reading with invented forecasts, with no request and no reader invocation behind it.
    let (_dir, git) = closed_proposal(|p| {
        p.request = None;
        p.reader = None;
    });
    // The scoring and outcome rules alone accept it; that is the forgery.
    assert_eq!(job_records_of(&git, "HEAD"), vec![]);
    let found = provenance_of(&git, "HEAD");
    assert_refused(&found, "has no request record", "athena ask");
    assert_refused(&found, "no reader invocation", "athena ask");
    // The commit-msg hook, which `athena close` also runs, refuses it.
    let gated = gate_of(&git, "HEAD");
    assert!(
        gated
            .iter()
            .any(|v| v.rule == Rule::JobRecords && v.detail.starts_with(crate::jobs::PROVENANCE)),
        "{gated:?}"
    );

    // One missing link is enough.
    let (_dir, git) = closed_proposal(|p| p.request = None);
    assert_refused(
        &provenance_of(&git, "HEAD"),
        "has no request record",
        "athena ask",
    );
    let (_dir, git) = closed_proposal(|p| p.reader = None);
    assert_refused(
        &provenance_of(&git, "HEAD"),
        "no reader invocation",
        "athena ask",
    );
    // A request received after its reading, and a reader that ended after it.
    let (_dir, git) = closed_proposal(|p| {
        p.request.as_mut().unwrap().received_at = ts("2026-10-10T15:02:00Z");
    });
    assert_refused(
        &provenance_of(&git, "HEAD"),
        "before its request was received",
        "athena ask",
    );
    let (_dir, git) = closed_proposal(|p| {
        p.reader.as_mut().unwrap().ended_at = ts("2026-10-10T15:02:00Z");
    });
    assert_refused(
        &provenance_of(&git, "HEAD"),
        "before the reader invocation it names ended",
        "athena ask",
    );
    // A defaults hash that does not belong to the defaults.toml at the reading's base commit.
    let (_dir, git) = closed_proposal(|p| p.reading.defaults_sha256 = "b".repeat(64));
    assert_refused(
        &provenance_of(&git, "HEAD"),
        "does not match defaults.toml at its base commit",
        "athena ask",
    );
    // An invocation number that names a producer, not a reader.
    let (_dir, git) = closed_proposal(|p| {
        p.reader.as_mut().unwrap().role = Role::Producer;
    });
    assert_refused(
        &provenance_of(&git, "HEAD"),
        "no reader invocation",
        "athena ask",
    );
}

#[test]
fn a_proposal_made_by_athena_ask_and_closed_with_athena_close_is_accepted() {
    let (_dir, git) = closed_proposal(|_| {});
    assert_eq!(provenance_of(&git, "HEAD"), vec![]);
    let gated = gate_of(&git, "HEAD");
    assert!(
        !gated
            .iter()
            .any(|v| v.detail.starts_with(crate::jobs::PROVENANCE)),
        "{gated:?}"
    );
    // A reading whose base commit is not held cannot have its defaults hash recomputed; it is not refused.
    let (_dir, git) = closed_proposal(|p| {
        p.reading.base_commit = "1".repeat(40);
        p.reading.defaults_sha256 = "b".repeat(64);
    });
    assert_eq!(provenance_of(&git, "HEAD"), vec![]);
}

#[test]
fn the_older_build_job_in_a_non_cutover_history_is_not_refused() {
    // The fixtures of the tests above are the shape an older build wrote: a job.json with no dispatch
    // fields, no request record, and a base commit that is gone. The history walk judges commits already
    // made and does not call provenance; the live gate would refuse the same commit.
    let (dir, git) = owner_repo();
    let base = git.head().unwrap().unwrap();
    stage_landing(dir.path(), &git, &base, &HONEST);
    commit(&git, "Change a (#1)\n");
    let walked = history_of(&git);
    assert!(
        !walked
            .iter()
            .any(|v| v.detail.starts_with(crate::jobs::PROVENANCE)),
        "{walked:?}"
    );
    let live = provenance_of(&git, "HEAD");
    assert_refused(&live, "has no request record", "athena ask");
    assert_refused(&live, "lacks ", "athena go");
}
