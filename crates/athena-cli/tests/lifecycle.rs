//! Whole jobs, from request to landed commit, against the fake driver (design §8).

// Scenarios are set up field by field, so each test shows only what differs from the default.
#![allow(clippy::field_reassign_with_default)]

mod common;

use athena_core::run::GoResult;
use athena_data::records::{AcceptedBy, AnyRecord, OutcomeState, Role, Verdict};
use athena_gate::invariant::{PREDICTION, VERIFIED_CONTENT};
use common::*;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

fn finished(result: GoResult) -> athena_core::run::Finished {
    match result {
        GoResult::Finished(f) => *f,
        GoResult::Reread(_) => panic!("expected the job to finish"),
    }
}

fn record_at(fx: &Fixture, rev: &str, path: &str) -> Option<AnyRecord> {
    fx.git
        .show_file(rev, path)
        .unwrap()
        .map(|bytes| AnyRecord::parse(path, &bytes).unwrap())
}

/// Runs one job to its end under `config` and returns it with its result.
fn run_with_checks(
    fx: &Fixture,
    scenario: Scenario,
) -> (athena_data::JobId, athena_core::run::Finished) {
    let athena = fx.athena(scenario.script());
    let ui = Lines::default();
    let proposal = athena.ask("add a greeting to the readme", &ui).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &ui)
            .unwrap(),
    );
    (proposal.job, f)
}

fn baseline_of(fx: &Fixture, job: &athena_data::JobId) -> athena_data::records::Baseline {
    match record_at(fx, "HEAD", &format!("{}/baseline.json", job.dir())) {
        Some(AnyRecord::Baseline(b)) => *b,
        other => panic!("expected a baseline record, got {other:?}"),
    }
}

#[test]
fn baseline_records_failing_and_passing_checks() {
    let config = config_with_change_checks(&[
        ("bad", &["sh", "-c", "exit 1"]),
        ("good", &["sh", "-c", "exit 0"]),
    ]);
    let fx = Fixture::with_config(&config);
    let (job, _) = run_with_checks(&fx, Scenario::default());
    let baseline = baseline_of(&fx, &job);
    let by_name = |n: &str| baseline.checks.iter().find(|c| c.name == n).unwrap();
    assert!(!by_name("bad").passed);
    assert_eq!(by_name("bad").exit_code, Some(1));
    assert!(by_name("good").passed);
    assert_eq!(by_name("good").exit_code, Some(0));
    assert_eq!(baseline.checks.len(), 2);
    // The records check accepts the file where it is.
    let path = format!("{}/baseline.json", job.dir());
    let bytes = fx.git.show_file("HEAD", &path).unwrap().unwrap();
    assert!(athena_data::records::check_record_file(&path, &bytes).is_ok());
}

#[test]
fn baseline_is_taken_from_the_base_before_the_producer() {
    // `grep -q Hello README.md` fails on the base and passes once the producer has edited README.md.
    let config = config_with_change_checks(&[("greeting", &["grep", "-q", "Hello", "README.md"])]);
    let fx = Fixture::with_config(&config);
    let (job, f) = run_with_checks(&fx, Scenario::default());
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    let baseline = baseline_of(&fx, &job);
    // The base commit has no greeting; the baseline names it and the check failed there.
    let readme = fx
        .git
        .show_file(&baseline.base_commit, "README.md")
        .unwrap()
        .unwrap();
    assert!(!String::from_utf8(readme).unwrap().contains("Hello"));
    assert!(!baseline.checks[0].passed, "taken from the base");
    // The same check on the candidate passes.
    let dir = job.dir();
    let candidate = match record_at(&fx, "HEAD", &format!("{dir}/candidate.json")) {
        Some(AnyRecord::Candidate2(c)) => *c,
        other => panic!("unexpected {other:?}"),
    };
    assert_eq!(candidate.base_commit, baseline.base_commit);
    let verification = match record_at(&fx, "HEAD", &format!("{dir}/verification.json")) {
        Some(AnyRecord::Verification(v)) => *v,
        other => panic!("unexpected {other:?}"),
    };
    assert!(verification.checks[0].passed);
    // It precedes the producer: the first producer invocation has the number the baseline names.
    let producer = record_at(
        &fx,
        "HEAD",
        &format!(
            "{dir}/invocations/{:02}-producer.json",
            baseline.before_invocation
        ),
    );
    match producer {
        Some(AnyRecord::Invocation(i)) => assert_eq!(i.role, Role::Producer),
        other => panic!("expected the first producer invocation, got {other:?}"),
    }
    assert!(
        record_at(
            &fx,
            "HEAD",
            &format!(
                "{dir}/invocations/{:02}-producer.json",
                baseline.before_invocation - 1
            )
        )
        .is_none()
    );
}

#[test]
fn baseline_failure_does_not_stop_the_job() {
    // A check that fails on the base but passes on the candidate: the job still lands.
    let config = config_with_change_checks(&[
        ("greeting", &["grep", "-q", "Hello", "README.md"]),
        ("missing", &["/nonexistent/athena-baseline-check"]),
    ]);
    let fx = Fixture::with_config(&config);
    let scenario = Scenario::default();
    let calls = scenario.invocations.clone();
    let (job, f) = run_with_checks(&fx, scenario);
    // The producer ran although both baseline checks failed.
    assert!(calls.lock().unwrap().contains(&Role::Producer));
    let baseline = baseline_of(&fx, &job);
    assert!(baseline.checks.iter().all(|c| !c.passed));
    let missing = baseline
        .checks
        .iter()
        .find(|c| c.name == "missing")
        .unwrap();
    assert_eq!(missing.exit_code, None);
    assert!(missing.note.as_ref().unwrap().contains("could not start"));
    // The command that cannot start fails on the candidate too, exactly as it failed on the base, so it is
    // pre-existing and the job lands.
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);

    // With only the check that fails on the base, the same job lands.
    let config = config_with_change_checks(&[("greeting", &["grep", "-q", "Hello", "README.md"])]);
    let fx = Fixture::with_config(&config);
    let (job, f) = run_with_checks(&fx, Scenario::default());
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    assert!(!baseline_of(&fx, &job).checks[0].passed);
}

#[test]
fn baseline_tail_is_capped() {
    let config = config_with_change_checks(&[(
        "noisy",
        &[
            "sh",
            "-c",
            "i=0; while [ $i -lt 600 ]; do echo 0123456789; i=$((i+1)); done; exit 2",
        ],
    )]);
    let fx = Fixture::with_config(&config);
    let (job, _) = run_with_checks(&fx, Scenario::default());
    let check = &baseline_of(&fx, &job).checks[0];
    assert_eq!(check.exit_code, Some(2));
    assert!(!check.output_tail.is_empty());
    // The tail keeps at most 4 KiB, plus the three-byte ellipsis that marks the cut.
    assert!(check.output_tail.len() <= athena_core::checks::TAIL_BYTES + 3);
}

#[test]
fn the_change_capability_never_pins() {
    // A change job reads no studied repository: the pinner is never called and the job record has no pin.
    let fx = Fixture::new();
    let mut athena = fx.athena(Scenario::default().script());
    let pinner = athena_core::fake::CountingPinner::new(&"b".repeat(40));
    athena.pinner = Box::new(pinner.clone());
    let ui = Lines::default();
    let proposal = athena.ask("add a greeting to the readme", &ui).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &ui)
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    assert_eq!(pinner.count(), 0, "the change capability never pins");
    match record_at(&fx, "HEAD", &format!("{}/job.json", proposal.job.dir())) {
        Some(AnyRecord::Job(job)) => assert_eq!(job.pin, None),
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn a_request_lands_a_traceable_commit() {
    // M6/M7 (allows): a landing commit; a verified candidate lands. M11: a whole fake job leaves the
    // user's .git/config byte-identical. M14 (allows): completed invocations record measured usage.
    let fx = Fixture::new();
    let config_before = std::fs::read(fx.root().join(".git/config")).unwrap();
    let scenario = Scenario::default();
    let athena = fx.athena(scenario.script());
    let ui = Lines::default();
    let proposal = athena.ask("add a greeting to the readme", &ui).unwrap();
    let reading = proposal.reading.clone().expect("a reading");
    assert_eq!(
        reading.forecasts.reference.p_reading_accepted, 0.7,
        "the reference starts from the priors"
    );
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &ui)
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    assert!(
        fx.file("README.md").unwrap().contains("Hello, reader."),
        "the change is in the checkout"
    );
    // The fixture has no GitHub origin, so the early pull-request step is skipped without a word.
    let lines = ui.0.lock().unwrap().clone();
    assert!(
        !lines.iter().any(|l| l.to_lowercase().contains("warning")),
        "no warning line: {lines:?}"
    );

    let head = fx.head();
    let trailers = fx.git.trailers(&fx.git.message(&head).unwrap()).unwrap();
    let get = |key: &str| {
        trailers
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    };
    assert_eq!(get("Athena-Job").as_deref(), Some(proposal.job.as_str()));
    let prediction = get(PREDICTION).expect("a prediction trailer");
    assert!(
        fx.git
            .is_ancestor(&prediction, &format!("{head}~1"))
            .unwrap(),
        "the prediction came first"
    );
    assert_eq!(
        prediction, proposal.commit,
        "the prediction commit is the one that added the accepted reading"
    );
    let verified = get(VERIFIED_CONTENT).expect("a verified-content trailer");
    let base = match record_at(
        &fx,
        &head,
        &format!("{}/candidate.json", proposal.job.dir()),
    ) {
        Some(AnyRecord::Candidate2(c)) => {
            // The core links the job to its base: the base's fingerprint, and no predecessor because
            // no job had landed before this one.
            assert_eq!(
                c.base_fingerprint.as_deref(),
                Some(
                    athena_git::attrs::fingerprint(&fx.git, &c.base_commit, &c.base_commit)
                        .unwrap()
                        .as_str()
                ),
                "base_fingerprint is the base commit's fingerprint"
            );
            assert_eq!(c.previous_job, None, "no job had landed before");
            c.base_commit.clone()
        }
        other => panic!("unexpected {other:?}"),
    };
    assert_eq!(
        verified,
        athena_git::attrs::fingerprint(&fx.git, &head, &base).unwrap(),
        "anyone can recompute it"
    );
    assert!(get("Co-Authored-By").unwrap().contains("Claude Haiku 5.5"));
    assert!(get("Athena-Cost-USD").unwrap().ends_with("reported"));

    let dir = proposal.job.dir();
    for path in [
        "request.json",
        "reading-1.json",
        "job.json",
        "brief-producer.md",
        "invocations/01-reader.json",
        "invocations/02-producer.json",
        "invocations/03-verifier.json",
        "candidate.json",
        "brief-verifier.md",
        "verification.json",
        "outcome.json",
        "score.json",
    ] {
        assert!(
            fx.git.path_exists(&head, &format!("{dir}/{path}")).unwrap(),
            "{path} is committed"
        );
    }
    match record_at(&fx, &head, &format!("{dir}/invocations/02-producer.json")) {
        Some(AnyRecord::Invocation(inv)) => {
            assert_eq!(
                inv.tokens.output.provenance(),
                athena_data::Provenance::Measured
            );
            assert_eq!(inv.cost.value(), Some(0.01));
        }
        other => panic!("unexpected {other:?}"),
    }
    match record_at(&fx, &head, &format!("{dir}/score.json")) {
        Some(AnyRecord::Score(score)) => {
            let passes = score
                .events
                .iter()
                .find(|e| e.event == "verification_passes")
                .unwrap();
            assert_eq!(passes.outcome, Some(true));
            assert!(
                score
                    .deliverables
                    .as_ref()
                    .unwrap()
                    .terms
                    .iter()
                    .any(|t| t.path == "README.md" && t.changed)
            );
            assert_eq!(score.quantities.len(), 2);
        }
        other => panic!("unexpected {other:?}"),
    }
    assert_eq!(
        std::fs::read(fx.root().join(".git/config")).unwrap(),
        config_before
    );
    let report = athena.check_history().unwrap();
    assert!(report.violations.is_empty(), "{:?}", report.violations);
    let status = athena.status().unwrap();
    assert_eq!(status.jobs_by_state.get("landed"), Some(&1));
}

#[test]
fn the_verifier_never_sees_the_producers_report_or_hidden_content() {
    // M8 (blocks): a canary planted in the producer's report and commands is absent from the composed
    // verifier context and snapshot. M11: owner-private files and records are absent from the snapshot.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.report = producer_report(
        "PRODUCER-CANARY: I reasoned about it this way.",
        &["cargo test --locked"],
    );
    scenario.producer_commands = vec!["cargo test PRODUCER-CANARY".into()];
    scenario.reading["forecast"]["p_user_intervenes"] = serde_json::json!(0.1234);
    scenario.reading["forecast"]["cost_usd"]["p50"] = serde_json::json!(0.4321);
    let seen = scenario.seen_by_verifier.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed);
    let seen = seen.lock().unwrap().join("\n");
    assert!(
        seen.contains("add a greeting"),
        "the verifier gets the request"
    );
    assert!(
        seen.contains("README.md ends with a greeting line"),
        "and the criteria"
    );
    assert!(seen.contains("Hello, reader."), "and the diff");
    assert!(
        !seen.contains("PRODUCER-CANARY"),
        "but never the producer's report or commands"
    );
    assert!(!seen.contains("PRIVATE-CANARY"), "nor owner-private files");
    assert!(
        !seen.contains("0.1234") && !seen.contains("0.4321"),
        "nor any forecast"
    );
}

#[test]
fn a_failing_check_still_runs_the_verifier_and_every_criterion_is_judged() {
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.edits = vec![("README.md".into(), None)];
    let calls = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("remove the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Rejected);
    assert!(calls.lock().unwrap().contains(&Role::Verifier));
    assert!(fx.file("README.md").is_some(), "nothing landed");
    assert!(athena.check_history().unwrap().violations.is_empty());
    let verification = match record_at(
        &fx,
        "HEAD",
        &format!("{}/verification.json", proposal.job.dir()),
    ) {
        Some(AnyRecord::Verification(v)) => *v,
        other => panic!("unexpected {other:?}"),
    };
    assert_eq!(verification.criteria.len(), 2);
    for c in &verification.criteria {
        assert_ne!(c.evidence, "the verifier did not assess this criterion");
        assert!(c.met, "the verifier judged it on its own: {c:?}");
    }
    assert_eq!(verification.final_verdict, Verdict::Fail);
    let failed: Vec<_> = verification.checks.iter().filter(|c| !c.passed).collect();
    assert!(!failed.is_empty());
    for c in failed {
        assert!(
            !c.note
                .as_deref()
                .unwrap_or("")
                .starts_with(athena_data::records::PRE_EXISTING_NOTE),
            "it passed at the base, so it is not pre-existing"
        );
        assert!(
            verification
                .reasons
                .contains(&format!("check {} failed", c.name))
        );
    }
}

#[test]
fn a_check_already_failing_on_the_base_does_not_reject_a_job_that_leaves_it_failing() {
    let config = config_with_change_checks(&[("old", &["sh", "-c", "exit 1"])]);
    let fx = Fixture::with_config(&config);
    let (job, f) = run_with_checks(&fx, Scenario::default());
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    assert!(!baseline_of(&fx, &job).checks[0].passed);
    let verification = match record_at(&fx, "HEAD", &format!("{}/verification.json", job.dir())) {
        Some(AnyRecord::Verification(v)) => *v,
        other => panic!("unexpected {other:?}"),
    };
    let old = verification
        .checks
        .iter()
        .find(|c| c.name == "old")
        .unwrap();
    assert!(!old.passed, "the record still says it failed");
    assert!(
        old.note
            .as_deref()
            .unwrap()
            .starts_with(athena_data::records::PRE_EXISTING_NOTE)
    );
    assert_eq!(verification.final_verdict, Verdict::Pass);
    assert!(
        verification.reasons.is_empty(),
        "{:?}",
        verification.reasons
    );

    // Counter case: the same check passes on the base and fails after the change.
    let config = config_with_change_checks(&[("new", &["sh", "-c", "! grep -q Hello README.md"])]);
    let fx = Fixture::with_config(&config);
    let (job, f) = run_with_checks(&fx, Scenario::default());
    assert_eq!(f.state, OutcomeState::Rejected, "{}", f.reason);
    assert!(baseline_of(&fx, &job).checks[0].passed);
}

#[test]
fn a_fake_driver_go_runs_only_the_reader_producer_and_verifier() {
    // Job planning is off unless `[job_plan]` enables it: no planner or critic is called.
    let fx = Fixture::new();
    let scenario = Scenario::default();
    let calls = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed);
    let roles: std::collections::BTreeSet<Role> = calls.lock().unwrap().iter().copied().collect();
    assert_eq!(
        roles,
        [Role::Reader, Role::Producer, Role::Verifier]
            .into_iter()
            .collect()
    );
}

/// A script that also keeps every brief the reader was given.
fn recording_readers(scenario: Scenario) -> (athena_agent::fake::Script, Arc<Mutex<Vec<String>>>) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let inner = scenario.script();
    let kept = seen.clone();
    let script: athena_agent::fake::Script = Box::new(move |spec| {
        if spec.role == Role::Reader {
            kept.lock().unwrap().push(spec.user_prompt.clone());
        }
        inner(spec)
    });
    (script, seen)
}

#[test]
fn try_again_on_a_rejected_job_carries_its_request_and_findings_to_the_reader() {
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.verdict = serde_json::json!({
        "verdict": "fail",
        "criteria": [
            { "criterion": "criterion 1", "met": false, "evidence": "missing", "commands": [] },
            { "criterion": "criterion 2", "met": true, "evidence": "ran", "commands": ["cargo test"] }
        ],
        "findings": [{
            "severity": "major",
            "summary": "FINDING-CANARY the greeting is missing",
            "evidence": "README.md has no greeting",
            "location": "README.md"
        }],
        "request_reading_mismatch": null,
        "summary": "The greeting is missing."
    });
    let (script, readers) = recording_readers(scenario);
    let athena = fx.athena(script);
    let first = athena
        .ask("add a greeting ORIGINAL-CANARY", &Lines::default())
        .unwrap();
    let rejected = finished(
        athena
            .go(first.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(rejected.state, OutcomeState::Rejected);

    let retry = athena
        .ask(
            &format!("Try again on {}, please", first.job.as_str()),
            &Lines::default(),
        )
        .unwrap();
    assert_ne!(retry.job, first.job);
    let briefs = readers.lock().unwrap();
    assert!(!briefs[0].contains("Earlier attempt"));
    let brief = briefs.last().unwrap();
    assert!(brief.contains("# Earlier attempt"));
    assert!(brief.contains("> add a greeting ORIGINAL-CANARY"));
    assert!(brief.contains("FINDING-CANARY the greeting is missing"));
    assert!(
        brief.contains(&format!("> Try again on {}, please", first.job.as_str())),
        "the new request stays as typed"
    );
    assert!(athena.check_history().unwrap().violations.is_empty());
}

/// A failing verdict with one major finding whose summary is `canary`.
fn failing_verdict(canary: &str) -> serde_json::Value {
    serde_json::json!({
        "verdict": "fail",
        "criteria": [
            { "criterion": "criterion 1", "met": false, "evidence": "missing", "commands": [] },
            { "criterion": "criterion 2", "met": true, "evidence": "ran", "commands": ["cargo test"] }
        ],
        "findings": [{
            "severity": "major",
            "summary": canary,
            "evidence": "README.md has no greeting",
            "location": "README.md"
        }],
        "request_reading_mismatch": null,
        "summary": "The greeting is missing."
    })
}

#[test]
fn try_again_on_a_job_rejected_by_a_check_carries_the_failed_checks_output() {
    let config = test_config()
        .replace(
            "run = [\"sh\", \"-c\", \"grep -q relaxed",
            "run = [\"sh\", \"-c\", \"echo PASSING-CANARY; grep -q relaxed",
        )
        .replace(
            "\"test -f README.md &&",
            "\"test -f README.md || { echo FAILED-CHECK-CANARY; exit 1; }; test -f README.md &&",
        );
    let fx = Fixture::with_config(&config);
    let mut scenario = Scenario::default();
    scenario.edits = vec![("README.md".into(), None)];
    let (script, readers) = recording_readers(scenario);
    let athena = fx.athena(script);
    let first = athena.ask("remove the readme", &Lines::default()).unwrap();
    let rejected = finished(
        athena
            .go(first.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(rejected.state, OutcomeState::Rejected);

    athena
        .ask(
            &format!("try again on {}", first.job.as_str()),
            &Lines::default(),
        )
        .unwrap();
    let briefs = readers.lock().unwrap();
    let brief = briefs.last().unwrap();
    assert!(brief.contains("failed check test"), "{brief}");
    assert!(brief.contains("FAILED-CHECK-CANARY"), "{brief}");
    assert!(!brief.contains("PASSING-CANARY"), "{brief}");
}

#[test]
fn retrying_a_retry_carries_the_original_request_redacted() {
    let fx = Fixture::new();
    let mut first_scenario = Scenario::default();
    first_scenario.verdict = failing_verdict("FINDING-A-CANARY the greeting is missing");
    let mut second_scenario = Scenario::default();
    second_scenario.verdict = failing_verdict("FINDING-B-CANARY the greeting is still missing");
    // The retry of a content-only rejection starts from the first attempt's files, so an identical edit
    // would change nothing and fail as an empty candidate: this attempt writes something else.
    second_scenario.edits = vec![(
        "README.md".into(),
        Some("# Demo\n\nHello again, reader.\n".into()),
    )];
    let first_script = first_scenario.script();
    let second_script = second_scenario.script();
    let verifier_calls = Arc::new(Mutex::new(0usize));
    let producer_calls = Arc::new(Mutex::new(0usize));
    let readers = Arc::new(Mutex::new(Vec::new()));
    let kept = readers.clone();
    let script: athena_agent::fake::Script = Box::new(move |spec| {
        if spec.role == Role::Reader {
            kept.lock().unwrap().push(spec.user_prompt.clone());
        }
        if spec.role == Role::Producer {
            let mut calls = producer_calls.lock().unwrap();
            *calls += 1;
            if *calls > 1 {
                return second_script(spec);
            }
        }
        if spec.role == Role::Verifier {
            let mut calls = verifier_calls.lock().unwrap();
            *calls += 1;
            if *calls > 1 {
                return second_script(spec);
            }
        }
        first_script(spec)
    });
    let athena = fx.athena(script);

    let a = athena
        .ask("add a greeting ORIGINAL-CANARY", &Lines::default())
        .unwrap();
    let a_end = finished(
        athena
            .go(a.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(a_end.state, OutcomeState::Rejected);

    let b = athena
        .ask(
            &format!("try again on {}", a.job.as_str()),
            &Lines::default(),
        )
        .unwrap();
    let b_end = finished(
        athena
            .go(b.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(b_end.state, OutcomeState::Rejected);

    let c = athena
        .ask(
            &format!("try again on {}", b.job.as_str()),
            &Lines::default(),
        )
        .unwrap();
    assert_ne!(c.job, b.job);
    let briefs = readers.lock().unwrap();
    let brief = briefs.last().unwrap();
    assert!(brief.contains("# Earlier attempt"));
    assert!(
        brief.contains("> add a greeting ORIGINAL-CANARY"),
        "the original request is carried"
    );
    assert!(
        !brief.contains(&format!("> try again on {}", a.job.as_str())),
        "the retry phrase is not carried as the earlier request"
    );
    assert!(
        brief.contains("FINDING-B-CANARY"),
        "B's findings are carried"
    );
    assert!(
        !brief.contains("FINDING-A-CANARY"),
        "A's findings are not carried"
    );
    assert!(athena.check_history().unwrap().violations.is_empty());
}

#[test]
fn a_request_naming_a_job_carries_its_request_and_findings_to_the_reader() {
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.verdict = serde_json::json!({
        "verdict": "fail",
        "criteria": [
            { "criterion": "criterion 1", "met": false, "evidence": "missing", "commands": [] },
            { "criterion": "criterion 2", "met": true, "evidence": "ran", "commands": ["cargo test"] }
        ],
        "findings": [{
            "severity": "major",
            "summary": "FINDING-CANARY the greeting is missing",
            "evidence": "README.md has no greeting",
            "location": "README.md"
        }],
        "request_reading_mismatch": null,
        "summary": "The greeting is missing."
    });
    let (script, readers) = recording_readers(scenario);
    let athena = fx.athena(script);
    let first = athena
        .ask("add a greeting ORIGINAL-CANARY", &Lines::default())
        .unwrap();
    finished(
        athena
            .go(first.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );

    let text = format!("make it friendlier than in {} please", first.job.as_str());
    let second = athena.ask(&text, &Lines::default()).unwrap();
    assert_ne!(second.job, first.job);
    let briefs = readers.lock().unwrap();
    let brief = briefs.last().unwrap();
    assert!(brief.contains("# Referenced jobs"));
    assert!(brief.contains("> add a greeting ORIGINAL-CANARY"));
    assert!(brief.contains("FINDING-CANARY the greeting is missing"));
    assert!(!brief.contains("# Earlier attempt"));
    assert!(
        brief.contains(&format!("> {text}")),
        "the new request stays as typed"
    );
    drop(briefs);

    // An id that matches no job is ignored.
    athena
        .ask("see 19990101-000000-zzzz for context", &Lines::default())
        .unwrap();
    assert!(
        !readers
            .lock()
            .unwrap()
            .last()
            .unwrap()
            .contains("# Referenced jobs")
    );
}

#[test]
fn try_again_on_a_job_that_failed_at_admission_carries_the_refusal_reason() {
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario
        .edits
        .push(("rustfmt.toml".into(), Some("style = \"relaxed\"\n".into())));
    let (script, readers) = recording_readers(scenario);
    let athena = fx.athena(script);
    let first = athena
        .ask("add a greeting ORIGINAL-CANARY", &Lines::default())
        .unwrap();
    let failed = finished(
        athena
            .go(first.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(failed.state, OutcomeState::Failed);
    assert!(failed.verification.is_none(), "admission refused it early");

    athena
        .ask(
            &format!("try again on {}", first.job.as_str()),
            &Lines::default(),
        )
        .unwrap();
    let briefs = readers.lock().unwrap();
    let brief = briefs.last().unwrap();
    assert!(brief.contains("# Earlier attempt"), "{brief}");
    assert!(brief.contains("which failed"), "{brief}");
    assert!(
        brief.contains("> add a greeting ORIGINAL-CANARY"),
        "{brief}"
    );
    assert!(brief.contains("admission refused the change"), "{brief}");
    assert!(brief.contains("governance"), "{brief}");
    assert!(athena.check_history().unwrap().violations.is_empty());
}

#[test]
fn try_again_on_a_landed_or_unknown_job_is_refused() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let first = athena.ask("add a greeting", &Lines::default()).unwrap();
    let landed = finished(
        athena
            .go(first.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(landed.state, OutcomeState::Landed);
    let head = fx.head();

    let error = athena
        .ask(
            &format!("try again on {}", first.job.as_str()),
            &Lines::default(),
        )
        .unwrap_err()
        .to_string();
    assert!(error.contains(first.job.as_str()), "{error}");
    assert!(error.contains("landed"), "{error}");

    let error = athena
        .ask("try again on 19990101-000000-zzzz", &Lines::default())
        .unwrap_err()
        .to_string();
    assert!(error.contains("19990101-000000-zzzz"), "{error}");
    assert_eq!(fx.head(), head, "a refused retry commits nothing");
}

#[test]
fn a_candidate_that_deletes_a_check_still_has_it_run() {
    // M12 (blocks): a candidate that deletes a check still has that check run.
    let fx = Fixture::new();
    let config = test_config();
    let without_test_check = config.replace(
        "[[capabilities.change.checks]]\nname = \"test\"\nkind = \"test\"\nrun = [\"sh\", \"-c\", \"test -f README.md && ! test -f .cargo/config.toml && echo 'test result: ok. 1 passed; 0 failed; 0 ignored'\"]\n",
        "",
    );
    assert_ne!(config, without_test_check);
    let mut scenario = Scenario::default();
    scenario.reading = reading(
        &[("athena.toml", "modify"), ("README.md", "delete")],
        &[
            ("content", "athena.toml no longer has the test check"),
            ("content", "README.md is gone"),
        ],
    );
    scenario.edits = vec![
        ("athena.toml".into(), Some(without_test_check)),
        ("README.md".into(), None),
    ];
    let athena = fx.athena(scenario.script());
    let proposal = athena
        .ask("drop the test check and the readme", &Lines::default())
        .unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(
        f.state,
        OutcomeState::Rejected,
        "the base's test check ran and failed: {}",
        f.reason
    );
    let checks = &f.verification.unwrap().checks;
    assert!(checks.iter().any(|c| c.name == "test" && !c.passed));
}

#[test]
fn relaxed_tool_configuration_does_not_relax_its_check() {
    // M12 (blocks): a candidate rustfmt.toml that disables formatting does not relax its check.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.reading = reading(
        &[("rustfmt.toml", "modify"), ("src/lib.rs", "modify")],
        &[
            ("content", "rustfmt.toml is relaxed"),
            ("content", "src/lib.rs has a new function"),
        ],
    );
    scenario.edits = vec![
        ("rustfmt.toml".into(), Some("style = \"relaxed\"\n".into())),
        (
            "src/lib.rs".into(),
            Some(
                "pub fn one() -> u32 {\n    1\n}\npub fn two()->u32{2} // BADLY formatted\n".into(),
            ),
        ),
    ];
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a function", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Rejected, "{}", f.reason);
    let verification = f.verification.unwrap();
    assert!(
        verification
            .checks
            .iter()
            .any(|c| c.name == "fmt" && !c.passed)
    );
    assert_eq!(
        verification.tool_config_from_base,
        vec!["rustfmt.toml".to_string()]
    );
}

#[test]
fn a_cargo_config_runner_in_the_candidate_is_set_aside_for_the_checks() {
    // M12 (blocks): a `.cargo/config.toml` runner that skips tests does not relax the test check: the
    // checks run with the base's tool configuration, where the file does not exist.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.reading = reading(
        &[(".cargo/config.toml", "create"), ("README.md", "modify")],
        &[
            ("content", ".cargo/config.toml sets a test runner"),
            ("behavior", "the tests still pass"),
        ],
    );
    scenario.edits.push((
        ".cargo/config.toml".into(),
        Some("[target.x86_64-unknown-linux-gnu]\nrunner = \"true\"\n".into()),
    ));
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a test runner", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    let verification = f.verification.clone().unwrap();
    assert_eq!(
        verification.tool_config_from_base,
        vec![".cargo/config.toml".to_string()]
    );
    assert!(
        verification.checks.iter().all(|c| c.passed),
        "the checks ran without the candidate's runner"
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    assert!(
        fx.file(".cargo/config.toml").is_some(),
        "the change itself lands, and governs later jobs"
    );
}

#[test]
fn a_governance_change_must_be_a_listed_deliverable_named_in_a_criterion() {
    // M12 (blocks): a governance file changed without being a listed deliverable is refused.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario
        .edits
        .push(("rustfmt.toml".into(), Some("style = \"relaxed\"\n".into())));
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Failed);
    assert!(f.reason.contains("governance"), "{}", f.reason);
}

#[test]
fn a_listed_criterion_covered_change_to_athena_toml_lands() {
    // M12 (allows): a listed, criterion-covered change to athena.toml.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    let raised = with_job_usd(&test_config(), "9.25");
    scenario.reading = reading(
        &[("athena.toml", "modify")],
        &[
            ("content", "athena.toml raises job_usd to 9.25"),
            ("behavior", "the tests still pass"),
        ],
    );
    scenario.edits = vec![("athena.toml".into(), Some(raised))];
    let athena = fx.athena(scenario.script());
    let proposal = athena
        .ask("raise the job budget to nine dollars", &Lines::default())
        .unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    assert!(fx.file("athena.toml").unwrap().contains("job_usd = 9.25"));
}

#[test]
fn a_candidate_that_marks_a_path_excluded_is_refused() {
    // M12 (blocks): a candidate that marks a path athena-exclude is refused.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.reading = reading(
        &[(".gitattributes", "modify")],
        &[("content", ".gitattributes hides src")],
    );
    scenario.edits = vec![(
        ".gitattributes".into(),
        Some(
            "/PRIVATE.md athena-exclude\n/SPEC.md athena-readonly\n/src/** athena-exclude\n".into(),
        ),
    )];
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("hide src", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Failed);
    assert!(f.reason.contains("classified"), "{}", f.reason);
}

#[test]
fn a_candidate_with_a_generated_token_does_not_land() {
    // M2 (blocks): a candidate containing a generated token does not land.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    let token = format!(
        "{}{}",
        ["gh", "p_"].concat(),
        "Zq8rT2vK9mW4xN7bP3cH6jL1sD5fG0aYuE2i"
    );
    scenario.edits = vec![(
        "README.md".into(),
        Some(format!("# Demo\n\nauth: {token}\n")),
    )];
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a token", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Failed, "{}", f.reason);
    assert!(f.reason.contains("gitleaks"), "{}", f.reason);
    let head = fx.head();
    let mut found = false;
    for entry in fx.git.ls_tree(&head).unwrap() {
        let text = String::from_utf8_lossy(&fx.git.cat_blob(&entry.oid).unwrap()).into_owned();
        found |= text.contains(&token);
    }
    assert!(!found, "the token is nowhere in the committed tree");
    assert!(athena.check_history().unwrap().violations.is_empty());
}

#[test]
fn a_behavior_criterion_citing_a_command_that_never_ran_is_unmet() {
    // M16 (blocks): a criterion citing a command that never ran counts as unmet.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.verifier_commands = vec!["cargo build".into()];
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Rejected);
    let v = f.verification.unwrap();
    assert_eq!(v.agent_verdict, Some(Verdict::Pass));
    assert_eq!(v.final_verdict, Verdict::Fail);
    assert!(!v.criteria[1].commands_verified);
}

#[test]
fn ask_without_a_terminal_leaves_a_proposal_commit_and_close_drops_it() {
    // M14 (blocks): `ask` without a terminal leaves a proposal commit.
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let before = fx.head();
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    assert_eq!(fx.head(), proposal.commit);
    assert!(fx.git.is_ancestor(&before, &proposal.commit).unwrap());
    let closed = athena
        .close(proposal.job.as_str(), "changed my mind", &Lines::default())
        .unwrap();
    assert_eq!(closed.state, OutcomeState::Dropped);
    match record_at(
        &fx,
        &fx.head(),
        &format!("{}/score.json", proposal.job.dir()),
    ) {
        Some(AnyRecord::Score(score)) => {
            assert_eq!(score.events.len(), 1);
            assert_eq!(
                score.events[0].outcome,
                Some(false),
                "a dropped reading resolves as not accepted"
            );
        }
        other => panic!("unexpected {other:?}"),
    }
    assert!(athena.check_history().unwrap().violations.is_empty());
}

#[test]
fn a_correction_rereads_and_records_an_intervention() {
    // Q3 / D27: a one-line correction of the reading is an intervention, recorded automatically.
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let again = athena
        .correct(
            proposal.job.as_str(),
            "put it in a new section",
            &Lines::default(),
        )
        .unwrap();
    assert_eq!(again.reading.as_ref().unwrap().n, 2);
    let status = athena.status().unwrap();
    assert_eq!(
        status.interventions_by_category.get("corrected-output"),
        Some(&1)
    );
    let f = finished(
        athena
            .go(again.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed);
    let score = f.score.unwrap();
    let accepted: Vec<Option<bool>> = score
        .events
        .iter()
        .filter(|e| e.event == "reading_accepted")
        .map(|e| e.outcome)
        .collect();
    assert_eq!(accepted, vec![Some(false), Some(true)]);
    let intervenes = score
        .events
        .iter()
        .find(|e| e.event == "user_intervenes")
        .unwrap();
    assert_eq!(
        intervenes.outcome,
        Some(false),
        "a correction before dispatch is not an intervention after it"
    );
    assert!(athena.check_history().unwrap().violations.is_empty());
}

#[test]
fn a_killed_invocation_is_recorded_with_bounds() {
    // M14 (blocks): a killed invocation yields a record with lower-bound tokens and cost bounded by its cap.
    let fx = Fixture::new();
    let base = Scenario::default().script();
    let script: athena_agent::fake::Script = Box::new(move |spec| {
        let mut turn = base(spec);
        if spec.role == Role::Producer {
            turn.outcome = athena_data::records::InvocationOutcome::Killed;
            turn.structured = None;
            turn.detail = Some("deadline of 1800s reached".into());
        }
        turn
    });
    let athena = fx.athena(script);
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Failed);
    match record_at(
        &fx,
        &fx.head(),
        &format!("{}/invocations/02-producer.json", proposal.job.dir()),
    ) {
        Some(AnyRecord::Invocation(inv)) => {
            assert_eq!(
                inv.tokens.output.provenance(),
                athena_data::Provenance::Estimated
            );
            assert_eq!(inv.tokens.output.lower(), Some(20.0));
            assert_eq!(inv.cost.value(), None);
            assert_eq!((inv.cost.lower(), inv.cost.upper()), (Some(0.0), Some(3.0)));
        }
        other => panic!("unexpected {other:?}"),
    }
    assert!(
        athena.check_history().unwrap().violations.is_empty(),
        "the bounds reconcile"
    );
}

#[test]
fn a_producer_stopped_by_the_kill_switch_closes_killed_with_a_provisional_cost_score() {
    // m8 of the v0 acceptance review: the stop is a kill, not an error, and a cost known only as a bound
    // is not scored as a firm figure.
    let fx = Fixture::new();
    let base = Scenario::default().script();
    let script: athena_agent::fake::Script = Box::new(move |spec| {
        let mut turn = base(spec);
        if spec.role == Role::Producer {
            turn.outcome = athena_data::records::InvocationOutcome::Killed;
            turn.structured = None;
            turn.detail = Some("the kill switch was turned on".into());
        }
        turn
    });
    let athena = fx.athena(script);
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Failed);
    let head = fx.head();
    let dir = proposal.job.dir();
    match record_at(&fx, &head, &format!("{dir}/invocations/02-producer.json")) {
        Some(AnyRecord::Invocation(inv)) => {
            assert_eq!(inv.outcome, athena_data::records::InvocationOutcome::Killed);
            assert!(inv.detail.as_deref().unwrap().contains("kill switch"));
            assert_eq!(inv.cost.value(), None);
        }
        other => panic!("unexpected {other:?}"),
    }
    match record_at(&fx, &head, &format!("{dir}/score.json")) {
        Some(AnyRecord::Score(score)) => {
            let cost = score
                .quantities
                .iter()
                .find(|q| q.quantity == "cost_usd")
                .expect("a cost score");
            assert!(cost.provisional, "a bound-only cost is not a firm figure");
            let time = score
                .quantities
                .iter()
                .find(|q| q.quantity == "wall_time_s")
                .expect("a time score");
            assert!(!time.provisional);
        }
        other => panic!("unexpected {other:?}"),
    }
    let report = athena.check_history().unwrap();
    assert!(
        report.violations.is_empty(),
        "the gate recomputes the same score: {:?}",
        report.violations
    );
}

/// An interrupted job: the history of a killed run cut back to just before its closing commit, with the
/// producer's invocation record rewritten into the shape written before the usage fix (per-model costs
/// that do not sum to the reported total) and committed by hand. Returns the job and the invocation path.
fn interrupted_job_with_legacy_record(
    fx: &Fixture,
    athena: &athena_core::Athena,
) -> (athena_data::JobId, String) {
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Failed);
    let dir = proposal.job.dir();
    // The closing commit follows the dispatch commit, and an introspection of the failure follows it.
    let closing = fx
        .git
        .run(["rev-parse", "HEAD~1"])
        .unwrap()
        .trim()
        .to_string();
    fx.git.run(["reset", "-q", "--hard", "HEAD~2"]).unwrap();
    assert!(record_at(fx, &closing, &format!("{dir}/outcome.json")).is_some());
    assert!(record_at(fx, &fx.head(), &format!("{dir}/outcome.json")).is_none());
    let path = format!("{dir}/invocations/02-producer.json");
    let Some(AnyRecord::Invocation(mut inv)) = record_at(fx, &closing, &path) else {
        panic!("the producer's invocation is in the closing commit")
    };
    inv.cost = athena_data::Quantity::reported(1.0, athena_data::Unit::Usd, "legacy total");
    inv.models_used.insert(
        inv.model.clone(),
        athena_data::records::ModelUsage {
            input: 1,
            output: 1,
            cache_read: 0,
            cache_write: 0,
            cost_usd: 0.25,
        },
    );
    write(fx.root(), &[(&path, &AnyRecord::Invocation(inv).to_json())]);
    commit_by_hand(
        fx,
        &format!("Legacy usage\n\nAthena-Job: {}\n", proposal.job),
    );
    (proposal.job, path)
}

fn killed_producer() -> athena_agent::fake::Script {
    let base = Scenario::default().script();
    Box::new(move |spec| {
        let mut turn = base(spec);
        if spec.role == Role::Producer {
            turn.outcome = athena_data::records::InvocationOutcome::Killed;
            turn.structured = None;
        }
        turn
    })
}

#[test]
fn closing_a_job_with_a_legacy_usage_record_needs_the_flag_and_names_the_record() {
    // M15 (allows, legacy): an interrupted job whose committed invocation fails reconciliation is closed
    // with --allow-legacy-usage; without the flag the gate still refuses.
    let fx = Fixture::new();
    let athena = fx.athena(killed_producer());
    let (job, path) = interrupted_job_with_legacy_record(&fx, &athena);
    let before = fx.head();
    let invocation_blob = |rev: &str| fx.git.show_file(rev, &path).unwrap();

    let err = athena
        .close(job.as_str(), "stuck", &Lines::default())
        .unwrap_err();
    assert!(
        format!("{err:#}").contains("usage records reconcile"),
        "{err:#}"
    );
    assert!(
        format!("{err:#}").contains("--allow-legacy-usage"),
        "{err:#}"
    );
    assert_eq!(fx.head(), before, "a refused close commits nothing");

    let ui = Lines::default();
    let closed = athena.close_with(job.as_str(), "stuck", true, &ui).unwrap();
    assert!(matches!(
        closed.state,
        OutcomeState::Abandoned | OutcomeState::Failed
    ));
    assert_ne!(fx.head(), before);
    let dir = job.dir();
    let Some(AnyRecord::Outcome(outcome)) =
        record_at(&fx, &fx.head(), &format!("{dir}/outcome.json"))
    else {
        panic!("an outcome")
    };
    assert_eq!(outcome.totals.unwrap().invocations, 2);
    assert!(
        fx.git
            .ls_tree_under(&fx.head(), &dir)
            .unwrap()
            .iter()
            .any(|e| e.path.contains("score")),
        "a score is written"
    );
    assert_eq!(
        invocation_blob(&fx.head()),
        invocation_blob(&before),
        "the legacy record is left unmodified"
    );
    let said = ui.0.lock().unwrap().join("\n");
    assert!(
        said.contains("legacy usage record") && said.contains("per-model costs sum"),
        "the close names what it let through: {said}"
    );
}

/// A producer report whose only check is the full test run, with the given result.
fn full_test_report(result: &str) -> serde_json::Value {
    let mut report = producer_report("Added a greeting line.", &[]);
    report["checks_run"] = serde_json::json!([
        { "command": "cargo test --locked", "result": result, "note": null }
    ]);
    report
}

#[test]
fn a_not_run_full_test_is_not_sent_back_and_reaches_the_verifier() {
    // Athena runs the full suite on the candidate, so a producer report with it `not-run` goes on.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.report = full_test_report("not-run");
    let calls = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let ui = Lines::default();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &ui)
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    let roles = calls.lock().unwrap().clone();
    assert_eq!(
        roles.iter().filter(|r| **r == Role::Producer).count(),
        1,
        "{roles:?}"
    );
    assert!(roles.contains(&Role::Verifier), "{roles:?}");
    assert!(!ui.0.lock().unwrap().iter().any(|l| l.contains("sent back")));
}

#[test]
fn a_producer_that_makes_no_change_fails_whatever_its_test_result() {
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.report = full_test_report("not-run");
    scenario.edits = Vec::new();
    let calls = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Failed, "{}", f.reason);
    assert!(f.reason.contains("made no change"), "{}", f.reason);
    assert!(!calls.lock().unwrap().contains(&Role::Verifier));
}

#[test]
fn a_report_with_no_full_test_check_or_a_failing_one_goes_to_the_verifier_without_a_send_back() {
    // The full suite is Athena's check on the candidate, not a report requirement.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.report = producer_report("Added a greeting line.", &["cargo clippy"]);
    let calls = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    let roles = calls.lock().unwrap().clone();
    assert_eq!(
        roles.iter().filter(|r| **r == Role::Producer).count(),
        1,
        "{roles:?}"
    );
    assert!(roles.contains(&Role::Verifier), "{roles:?}");

    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.report = full_test_report("fail");
    let calls = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    athena
        .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
        .unwrap();
    let roles = calls.lock().unwrap().clone();
    assert_eq!(
        roles.iter().filter(|r| **r == Role::Producer).count(),
        1,
        "{roles:?}"
    );
    assert!(roles.contains(&Role::Verifier), "{roles:?}");
}

/// A producer report that marks the second default criterion unmet and the first met.
fn unmet_criterion_report() -> serde_json::Value {
    let texts = criteria_texts(&default_reading());
    let mut report = full_test_report("pass");
    report["criteria_check"] = serde_json::json!([
        criterion_check(&texts[0], "met"),
        criterion_check(&texts[1], "unmet"),
    ]);
    report
}

#[test]
fn a_criterion_unmet_before_verification_goes_back_to_the_producer_not_the_verifier() {
    let texts = criteria_texts(&default_reading());
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.report = unmet_criterion_report();
    scenario.report_when_sent_back = Some(full_test_report("pass")); // empty criteria_check: all met
    let calls = scenario.invocations.clone();
    let briefs = scenario.seen_by_producer.clone();
    let seen_by_verifier = scenario.seen_by_verifier.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let ui = Lines::default();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &ui)
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    let roles = calls.lock().unwrap().clone();
    let first_verifier = roles.iter().position(|r| *r == Role::Verifier).unwrap();
    assert_eq!(
        roles[..first_verifier]
            .iter()
            .filter(|r| **r == Role::Producer)
            .count(),
        2,
        "both producer runs come before verification: {roles:?}"
    );
    let briefs = briefs.lock().unwrap().clone();
    assert_eq!(briefs.len(), 2);
    assert!(!briefs[0].contains("## Sent back"));
    assert!(briefs[1].contains("## Sent back"), "{}", briefs[1]);
    assert!(
        briefs[1].contains(&format!("- {}", texts[1])),
        "{}",
        briefs[1]
    );
    assert!(
        !briefs[1].contains(&format!("- {}", texts[0])),
        "a met criterion is not listed: {}",
        briefs[1]
    );
    assert!(
        ui.0.lock()
            .unwrap()
            .iter()
            .any(|l| l.contains("sent back") && l.contains(&texts[1]))
    );
    // The verifier never saw the producer's report, so it was not the first to find the unmet criterion.
    assert!(
        seen_by_verifier
            .lock()
            .unwrap()
            .iter()
            .all(|s| !s.contains("## Sent back") && !s.contains("criteria_check"))
    );
}

#[test]
fn a_criterion_still_unmet_after_the_send_back_reaches_the_verifier_flagged() {
    let texts = criteria_texts(&default_reading());
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.report = unmet_criterion_report();
    let calls = scenario.invocations.clone();
    let seen_by_verifier = scenario.seen_by_verifier.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert!(
        !f.reason.contains("still had problems when sent back"),
        "{}",
        f.reason
    );
    let roles = calls.lock().unwrap().clone();
    assert_eq!(
        roles.iter().filter(|r| **r == Role::Producer).count(),
        2,
        "{roles:?}"
    );
    assert!(roles.contains(&Role::Verifier), "{roles:?}");
    let seen = seen_by_verifier.lock().unwrap().join("\n");
    assert!(seen.contains("producer-report-gaps"), "{seen}");
    assert!(seen.contains(&texts[1]), "{seen}");
}

#[test]
fn a_criterion_still_unmet_after_the_send_back_with_no_change_fails_the_job() {
    let texts = criteria_texts(&default_reading());
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.report = unmet_criterion_report();
    scenario.edits = Vec::new();
    let calls = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Failed, "{}", f.reason);
    assert!(f.reason.contains("made no change"), "{}", f.reason);
    assert!(f.reason.contains(&texts[1]), "{}", f.reason);
    assert!(!calls.lock().unwrap().contains(&Role::Verifier));
}

#[test]
fn a_criterion_missing_from_the_report_counts_as_unmet() {
    // It is sent back, and when still missing it is flagged to the verifier.
    let texts = criteria_texts(&default_reading());
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    let mut report = full_test_report("pass");
    report["criteria_check"] = serde_json::json!([criterion_check(&texts[0], "met")]);
    scenario.report = report;
    let briefs = scenario.seen_by_producer.clone();
    let seen_by_verifier = scenario.seen_by_verifier.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert!(briefs.lock().unwrap()[1].contains(&format!("- {}", texts[1])));
    let seen = seen_by_verifier.lock().unwrap().join("\n");
    assert!(seen.contains("producer-report-gaps"), "{seen}");
    assert!(seen.contains(&texts[1]), "{seen}");
}

#[test]
fn a_job_over_budget_is_refused_before_its_next_invocation() {
    // M13 (blocks): a job over budget fails before its next invocation.
    let fx = Fixture::with_config(&with_job_usd(&test_config(), "4.00"));
    let scenario = Scenario {
        cost: 0.4,
        ..Scenario::default()
    };
    let calls = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let before = fx.head();
    let error = athena
        .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
        .unwrap_err();
    assert!(error.to_string().contains("budget"), "{error}");
    assert_eq!(fx.head(), before, "nothing was dispatched");
    assert_eq!(*calls.lock().unwrap(), vec![Role::Reader]);
}

#[test]
fn the_kill_switch_blocks_new_work() {
    // M13 (blocks): a stop file blocks new invocations. (allows): normal runs proceed after start.
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    athena.stop().unwrap();
    assert!(athena.ask("add a greeting", &Lines::default()).is_err());
    assert!(athena.start().unwrap());
    assert!(athena.ask("add a greeting", &Lines::default()).is_ok());
}

#[test]
fn local_edits_block_the_landing_and_go_retries_it() {
    // M14 (blocks): a blocked job's records are committed. Then `go` lands it.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    let root = fx.root().to_path_buf();
    scenario.during_verification = Some(Arc::new(move || {
        std::fs::write(root.join("README.md"), "# Demo\n\nmy own local edit\n").unwrap();
    }));
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Blocked, "{}", f.reason);
    assert!(
        fx.git
            .path_exists(
                &fx.head(),
                &format!("{}/verification.json", proposal.job.dir())
            )
            .unwrap()
    );
    assert!(
        fx.file("README.md").unwrap().contains("my own local edit"),
        "the local edit is untouched"
    );
    fx.git.run(["checkout", "--", "README.md"]).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed);
    assert!(
        fx.git
            .path_exists(
                &fx.head(),
                &format!("{}/outcome-landed.json", proposal.job.dir())
            )
            .unwrap()
    );
    assert!(athena.check_history().unwrap().violations.is_empty());
}

#[test]
fn a_candidate_lands_over_a_records_only_interleave() {
    // M7 (allows): a candidate lands over a records-only interleave.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    let root = fx.root().to_path_buf();
    scenario.during_verification = Some(Arc::new(move || {
        let git = athena_git::Git::at(&root);
        let record = r#"{
  "schema": "athena.restart/1",
  "id": "20261010-170000-abcd",
  "at": "2026-10-10T17:00:00Z",
  "cost": "re-read the status",
  "minutes": null,
  "tokens": null,
  "lost": [],
  "via": ["athena status"]
}
"#;
        write(
            &root,
            &[("records/restarts/2026-10/20261010-170000-abcd.json", record)],
        );
        git.run(["add", "-A"]).unwrap();
        git.run([
            "commit",
            "-q",
            "-m",
            "Record a restart\n\nAthena-Record: records/restarts/2026-10/20261010-170000-abcd.json",
        ])
        .unwrap();
    }));
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    assert!(
        fx.git
            .path_exists(
                &fx.head(),
                "records/restarts/2026-10/20261010-170000-abcd.json"
            )
            .unwrap()
    );
    assert!(athena.check_history().unwrap().violations.is_empty());
}

#[test]
fn a_governed_change_on_the_branch_makes_the_job_stale() {
    // M7 (blocks): landing after the base's governed content changed.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    let root = fx.root().to_path_buf();
    let once = Arc::new(std::sync::atomic::AtomicBool::new(false));
    scenario.during_verification = Some(Arc::new(move || {
        if once.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        let git = athena_git::Git::at(&root);
        let record = r#"{
  "schema": "athena.intervention/1",
  "id": "20261010-170000-abcd",
  "at": "2026-10-10T17:00:00Z",
  "what": "Edited the library by hand",
  "why": "Faster than asking",
  "class_of_work": "code",
  "category": "did-work-themselves",
  "job": null,
  "source": "user",
  "with_staged_changes": true
}
"#;
        write(
            &root,
            &[
                ("src/lib.rs", "pub fn one() -> u32 {\n    1 // by hand\n}\n"),
                (
                    "records/interventions/2026-10/20261010-170000-abcd.json",
                    record,
                ),
            ],
        );
        git.run(["add", "-A"]).unwrap();
        git.run([
            "commit",
            "-q",
            "-m",
            "Edit by hand\n\nAthena-Intervention: 20261010-170000-abcd",
        ])
        .unwrap();
    }));
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Stale, "{}", f.reason);
    assert!(
        f.reason.contains("\"Edit by hand\"") && f.reason.contains("nothing landed"),
        "the reason names the commit that moved the base: {}",
        f.reason
    );
    assert!(!fx.file("README.md").unwrap().contains("Hello, reader."));
    assert!(
        fx.file("src/lib.rs").unwrap().contains("// by hand"),
        "the user's own change is untouched"
    );
    assert!(athena.check_history().unwrap().violations.is_empty());
    let shown = athena_cli::display::finished(&f);
    assert!(shown.contains("ask again"), "{shown}");
    let status = athena.status().unwrap();
    assert!(
        status.open.is_empty(),
        "nothing is left open or locked: {:?}",
        status.open
    );
    // Recovering costs nothing beyond asking again: the next job runs on top of the user's change.
    let again = athena.ask("add a greeting", &Lines::default()).unwrap();
    let redo = finished(
        athena
            .go(again.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(redo.state, OutcomeState::Landed, "{}", redo.reason);
    assert!(fx.file("README.md").unwrap().contains("Hello, reader."));
    assert!(fx.file("src/lib.rs").unwrap().contains("// by hand"));
    assert!(athena.check_history().unwrap().violations.is_empty());
}

fn intervene_on_latest(athena: &athena_core::Athena) -> athena_data::RecordId {
    athena
        .intervene(
            &athena_core::manage::InterveneArgs {
                what: "Reworded the greeting by hand".into(),
                why: "The landed wording was off".into(),
                class: "docs".into(),
                category: "corrected-output".into(),
                job: Some("latest".into()),
                with_staged: false,
            },
            &Lines::default(),
        )
        .unwrap()
        .intervention
        .id
}

#[test]
fn status_lists_an_uncited_intervention_as_open_work_with_a_next_command() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    assert!(athena.status().unwrap().open_interventions.is_empty());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed);
    let id = intervene_on_latest(&athena);

    let status = athena.status().unwrap();
    let [open] = status.open_interventions.as_slice() else {
        panic!("one open intervention, got {:?}", status.open_interventions)
    };
    assert_eq!(open.id, id.to_string());
    assert_eq!(open.category, "corrected-output");
    assert_eq!(open.job.as_deref(), Some(proposal.job.as_str()));
    assert_eq!(
        open.next,
        format!("athena ask \"<request that addresses intervention {id}>\"")
    );
    assert!(
        status.next.iter().any(|n| n.contains(&open.next)),
        "{:?}",
        status.next
    );
    // The counts are unchanged by the new list.
    assert_eq!(status.interventions_by_category["corrected-output"], 1);
}

#[test]
fn an_intervention_leaves_open_work_once_a_later_request_cites_its_id() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    let id = intervene_on_latest(&athena);
    assert_eq!(athena.status().unwrap().open_interventions.len(), 1);

    // A request that does not name the id does not clear it.
    athena.ask("tidy the docs", &Lines::default()).unwrap();
    assert_eq!(athena.status().unwrap().open_interventions.len(), 1);

    athena
        .ask(
            &format!("address intervention {id}: reword the greeting"),
            &Lines::default(),
        )
        .unwrap();
    let status = athena.status().unwrap();
    assert!(status.open_interventions.is_empty());
    assert!(!status.next.iter().any(|n| n.contains(id.as_str())));
    assert_eq!(status.interventions_by_category["corrected-output"], 1);
}

fn log_as(
    athena: &athena_core::Athena,
    class: &str,
    category: &str,
    what: &str,
) -> Result<String, String> {
    athena
        .intervene(
            &athena_core::manage::InterveneArgs {
                what: what.into(),
                why: "It went wrong.".into(),
                class: class.into(),
                category: category.into(),
                job: None,
                with_staged: false,
            },
            &Lines::default(),
        )
        .map(|done| done.intervention.id.to_string())
        .map_err(|e| format!("{e:#}"))
}

fn status_bin(fx: &Fixture, args: &[&str]) -> String {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_athena"))
        .env_remove("ATHENA_NESTED")
        .env_remove("ATHENA_SCRATCH_DIR")
        .env("ATHENA_STATE_DIR", fx.state.path())
        .current_dir(fx.root())
        .args(args)
        .output()
        .unwrap();
    // Not every command succeeds here (`audit` of a repository with no job fails); callers read the output.
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn intervene_accepts_mistake_and_rejects_unknown_category_listing_mistake() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    log_as(&athena, "docs", "mistake", "Shipped without the docs test").expect("accepted");
    let status = athena.status().unwrap();
    assert_eq!(status.interventions_by_category["mistake"], 1);
    let error = log_as(&athena, "docs", "blunder", "x").unwrap_err();
    assert!(
        error.contains("mistake"),
        "the error lists the category: {error}"
    );
    assert!(error.contains("blunder"), "{error}");
}

#[test]
fn status_reports_mistakes_per_class_and_per_cause_apart_from_categories() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    log_as(&athena, "docs", "mistake", "Skipped the docs test").unwrap();
    log_as(&athena, "docs", "mistake", "Skipped it again").unwrap();
    log_as(&athena, "code", "mistake", "Edited the wrong file").unwrap();
    log_as(&athena, "code", "corrected-output", "Reworded a line").unwrap();

    let view = athena.status().unwrap();
    assert_eq!(view.mistakes.total, 3);
    assert_eq!(view.mistakes.by_class["docs"], 2);
    assert_eq!(view.mistakes.by_class["code"], 1);
    assert_eq!(view.mistakes.by_cause["no cause recorded"], 3);
    // The category counts are their own field and still count every intervention by its category.
    assert_eq!(view.interventions_by_category["mistake"], 3);
    assert_eq!(view.interventions_by_category["corrected-output"], 1);

    let text = athena_cli::display::status(&view);
    assert!(
        text.contains("mistakes 3: by class: code 1, docs 2"),
        "{text}"
    );
    assert!(
        text.contains("mistakes by cause: no cause recorded 3"),
        "{text}"
    );

    let json: serde_json::Value =
        serde_json::from_str(&status_bin(&fx, &["status", "--json"])).unwrap();
    assert_eq!(json["mistakes"]["total"], 3);
    assert_eq!(json["mistakes"]["by_class"]["docs"], 2);
    assert_eq!(json["mistakes"]["by_cause"]["no cause recorded"], 3);
    assert_eq!(json["interventions_by_category"]["mistake"], 3);

    // With no mistakes the status text has no mistake line.
    let clean = Fixture::new();
    let text = status_bin(&clean, &["status"]);
    assert!(!text.contains("mistakes"), "{text}");
}

#[test]
fn a_legacy_other_record_starting_mistake_colon_is_counted_in_status_and_audit_and_not_rewritten() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let id = log_as(&athena, "docs", "other", "Mistake: forgot the changelog").unwrap();
    log_as(&athena, "docs", "other", "A mistake: only mentioned").unwrap();
    let path = format!(
        "records/interventions/{}/{id}.json",
        athena_data::RecordId::parse(&id).unwrap().month()
    );
    let before = fx.file(&path).expect("the record file");

    let view = athena.status().unwrap();
    assert_eq!(view.mistakes.total, 1);
    assert_eq!(view.mistakes.by_class["docs"], 1);
    // It is still an `other` intervention in the category counts; nothing was rewritten.
    assert_eq!(view.interventions_by_category["other"], 2);
    assert!(!view.interventions_by_category.contains_key("mistake"));
    assert!(athena_cli::display::status(&view).contains("mistakes 1:"));
    let audit_text = status_bin(&fx, &["audit"]);
    assert!(
        audit_text.contains("mistakes 1: by class: docs 1"),
        "{audit_text}"
    );
    assert_eq!(fx.file(&path).unwrap(), before, "the record is unchanged");
    assert!(before.contains("\"category\": \"other\""), "{before}");
}

#[test]
fn the_readme_changelog_and_design_name_the_mistake_category() {
    for (name, text) in [
        ("README.md", include_str!("../../../README.md")),
        ("CHANGELOG.md", include_str!("../../../CHANGELOG.md")),
        (
            "docs/design/athena-v0.md",
            include_str!("../../../docs/design/athena-v0.md"),
        ),
    ] {
        assert!(
            text.contains("`mistake`"),
            "{name} does not name the mistake category"
        );
    }
}

#[test]
fn hand_work_is_declared_with_staged_changes_and_rescores_the_job() {
    // M6 (allows): an intervention commit with its record. §9.5: a late intervention re-scores a job.
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed);
    write(fx.root(), &[("README.md", "# Demo\n\nHello, reader!\n")]);
    fx.git.run(["add", "README.md"]).unwrap();
    let done = athena
        .intervene(
            &athena_core::manage::InterveneArgs {
                what: "Fixed the punctuation of the greeting".into(),
                why: "The landed line ended with a period".into(),
                class: "docs".into(),
                category: "corrected-output".into(),
                job: Some("latest".into()),
                with_staged: true,
            },
            &Lines::default(),
        )
        .unwrap();
    let rescore = done.rescore.expect("a re-score");
    let event = rescore
        .events
        .iter()
        .find(|e| e.event == "user_intervenes")
        .unwrap();
    assert_eq!((event.outcome, event.provisional), (Some(true), false));
    assert!(
        fx.git
            .path_exists(
                &fx.head(),
                &format!("{}/score-{}.json", proposal.job.dir(), done.intervention.id)
            )
            .unwrap()
    );
    let (restart, _) = athena
        .restart(
            &athena_core::manage::RestartArgs {
                cost: "re-read the status".into(),
                minutes: Some(5.0),
                lost: vec!["my notes".into()],
                via: vec!["athena status".into()],
                ..Default::default()
            },
            &Lines::default(),
        )
        .unwrap();
    assert_eq!(
        restart.minutes.unwrap().provenance(),
        athena_data::Provenance::Estimated
    );
    let report = athena.check_history().unwrap();
    assert!(report.violations.is_empty(), "{:?}", report.violations);
    let status = athena.status().unwrap();
    assert_eq!(status.restarts, 1);
    assert_eq!(
        status.interventions_by_category.get("corrected-output"),
        Some(&1)
    );
}

/// What a producer that was resumed from kept work saw: whether the earlier edit was in its working directory
/// before it did anything, and its brief.
type Seen = Arc<Mutex<Vec<(bool, String)>>>;

/// A script whose first producer run edits two files and then hits its deadline; later producer runs record
/// what they see and finish normally. `kill_with` makes the first run end with that outcome instead.
fn deadline_script(
    seen: Seen,
    kill_with: athena_data::records::InvocationOutcome,
) -> athena_agent::fake::Script {
    let base = Scenario::default().script();
    let runs = Arc::new(Mutex::new(0u32));
    Box::new(move |spec| {
        let mut turn = base(spec);
        if spec.role == Role::Producer {
            let mut runs = runs.lock().unwrap();
            *runs += 1;
            if *runs == 1 {
                turn.outcome = kill_with;
                turn.structured = None;
                turn.detail = Some("deadline of 1800s reached".into());
                turn.edits = vec![
                    ("src/partial.rs".into(), Some("// half done\n".into())),
                    ("README.md".into(), Some("# Demo\n\nHello\n".into())),
                ];
            } else {
                seen.lock().unwrap().push((
                    spec.workdir.join("src/partial.rs").exists(),
                    spec.user_prompt.clone(),
                ));
            }
        }
        turn
    })
}

#[test]
fn a_producer_that_hits_its_deadline_keeps_its_work_and_closes_timed_out() {
    let fx = Fixture::new();
    let athena = fx.athena(deadline_script(
        Seen::default(),
        athena_data::records::InvocationOutcome::TimedOut,
    ));
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::TimedOut, "{}", f.reason);
    let dir = proposal.job.dir();
    let head = fx.head();
    match record_at(&fx, &head, &format!("{dir}/outcome.json")) {
        Some(AnyRecord::Outcome(outcome)) => {
            assert_eq!(outcome.state, OutcomeState::TimedOut);
            assert!(outcome.reason.contains("deadline of 1800s reached"));
        }
        other => panic!("unexpected {other:?}"),
    }
    match record_at(&fx, &head, &format!("{dir}/invocations/02-producer.json")) {
        Some(AnyRecord::Invocation(inv)) => {
            assert_eq!(
                inv.outcome,
                athena_data::records::InvocationOutcome::TimedOut
            );
            assert!(inv.detail.unwrap().contains("deadline of 1800s reached"));
        }
        other => panic!("unexpected {other:?}"),
    }
    let note = String::from_utf8(
        fx.git
            .show_file(&head, &format!("{dir}/progress-note.md"))
            .unwrap()
            .expect("a progress note is committed with the closing"),
    )
    .unwrap();
    assert!(
        note.contains("src/partial.rs") && note.contains("README.md"),
        "{note}"
    );
    let kept = fx
        .state
        .path()
        .join("repo/kept")
        .join(proposal.job.as_str())
        .join("work");
    assert_eq!(
        std::fs::read_to_string(kept.join("src/partial.rs")).unwrap(),
        "// half done\n"
    );
    assert!(kept.join("README.md").exists());
    assert!(
        athena.check_history().unwrap().violations.is_empty(),
        "the records of a timed out job pass the history check"
    );
}

#[test]
fn retrying_a_timed_out_job_starts_from_the_kept_work_with_the_note() {
    let fx = Fixture::new();
    let seen = Seen::default();
    let athena = fx.athena(deadline_script(
        seen.clone(),
        athena_data::records::InvocationOutcome::TimedOut,
    ));
    let first = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(first.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::TimedOut);

    let (second, _) = athena.retry(first.job.as_str(), &Lines::default()).unwrap();
    assert_ne!(second.job, first.job);
    finished(
        athena
            .go(second.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    {
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        let (had_file, brief) = &seen[0];
        assert!(
            *had_file,
            "the earlier edit is in the new working directory"
        );
        assert!(brief.contains("## Resume"), "{brief}");
        assert!(
            brief.contains("src/partial.rs") && brief.contains("README.md"),
            "{brief}"
        );
    }
    assert!(
        !fx.state
            .path()
            .join("repo/kept")
            .join(first.job.as_str())
            .exists(),
        "the kept work is removed once the retry closes"
    );
}

#[test]
fn retrying_a_timed_out_job_whose_base_moved_starts_clean() {
    let fx = Fixture::new();
    let seen = Seen::default();
    let athena = fx.athena(deadline_script(
        seen.clone(),
        athena_data::records::InvocationOutcome::TimedOut,
    ));
    let first = athena.ask("add a greeting", &Lines::default()).unwrap();
    finished(
        athena
            .go(first.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    write(
        fx.root(),
        &[("src/lib.rs", "pub fn one() -> u32 {\n    2\n}\n")],
    );
    commit_by_hand(&fx, "Change the base");
    let (second, _) = athena.retry(first.job.as_str(), &Lines::default()).unwrap();
    finished(
        athena
            .go(second.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 1);
    assert!(!seen[0].0, "the kept work is not applied on a moved base");
    assert!(!seen[0].1.contains("## Resume"));
}

#[test]
fn status_shows_timed_out_apart_from_failed() {
    let fx = Fixture::new();
    let mode = Arc::new(Mutex::new(
        athena_data::records::InvocationOutcome::TimedOut,
    ));
    let base = deadline_script(
        Seen::default(),
        athena_data::records::InvocationOutcome::TimedOut,
    );
    let script: athena_agent::fake::Script = {
        let mode = mode.clone();
        Box::new(move |spec| {
            let mut turn = base(spec);
            if spec.role == Role::Producer {
                turn.outcome = *mode.lock().unwrap();
                turn.structured = None;
            }
            turn
        })
    };
    let athena = fx.athena(script);
    let timed = athena.ask("add a greeting", &Lines::default()).unwrap();
    finished(
        athena
            .go(timed.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    *mode.lock().unwrap() = athena_data::records::InvocationOutcome::Killed;
    let failed = athena.ask("add a farewell", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(failed.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Failed);
    let status = athena.status().unwrap();
    let state_of = |id: &athena_data::JobId| {
        status
            .recent
            .iter()
            .find(|l| l.id == id.as_str())
            .map(|l| l.state.clone())
    };
    assert_eq!(state_of(&timed.job).as_deref(), Some("timed_out"));
    assert_eq!(state_of(&failed.job).as_deref(), Some("failed"));
    assert_eq!(status.jobs_by_state.get("timed_out"), Some(&1));
}

#[test]
fn a_killed_producer_keeps_nothing_and_a_job_in_time_leaves_no_note() {
    let fx = Fixture::new();
    let athena = fx.athena(killed_producer());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Failed);
    let dir = proposal.job.dir();
    assert!(
        fx.git
            .show_file(&fx.head(), &format!("{dir}/progress-note.md"))
            .unwrap()
            .is_none()
    );
    assert!(!fx.state.path().join("repo/kept").exists());

    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    assert!(!fx.state.path().join("repo/kept").exists());
}

/// Commits the work tree as it is, with hooks off, the way a user who skips them would.
fn commit_by_hand(fx: &Fixture, message: &str) -> String {
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run(["commit", "-q", "--no-verify", "-m", message])
        .unwrap();
    fx.head()
}

/// What the commit-msg hook and the history check say about the commit at `HEAD`.
fn refusals(fx: &Fixture, athena: &athena_core::Athena) -> (Vec<String>, Vec<String>) {
    let head = fx.head();
    let change = athena_gate::Change::of_commit(&fx.git, &head).unwrap();
    let hook = athena_gate::gate(&fx.git, &change, athena_gate::Scope::CommitMsg)
        .unwrap()
        .iter()
        .map(|v| v.to_string())
        .collect();
    let history = athena
        .check_history()
        .unwrap()
        .violations
        .iter()
        .map(|v| v.to_string())
        .collect();
    (hook, history)
}

#[test]
fn a_hand_written_verification_does_not_make_a_landing() {
    // M6/M7 (blocks), acceptance verification 1, M1: a hand change dressed up with a copied
    // verification record and the landing trailers is refused by the hook and found by the history check.
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let real = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(real.state, OutcomeState::Landed, "{}", real.reason);
    let prediction = real.prediction_commit.clone().unwrap();
    let parent = fx.head();

    write(fx.root(), &[("README.md", "# Demo\n\nedited by hand\n")]);
    fx.git.run(["add", "README.md"]).unwrap();
    let tree = fx.git.write_index_tree().unwrap();
    let fp = athena_git::attrs::fingerprint(&fx.git, &tree, &parent).unwrap();
    let forged_job = "20261010-142000-f0rg";
    let forged_dir = athena_data::JobId::parse(forged_job).unwrap().dir();
    let mut verification = real.verification.clone().unwrap();
    verification.job = athena_data::JobId::parse(forged_job).unwrap();
    verification.verified_fingerprint = fp.clone();
    write(
        fx.root(),
        &[(
            &format!("{forged_dir}/verification.json"),
            &AnyRecord::Verification(Box::new(verification)).to_json(),
        )],
    );
    commit_by_hand(
        &fx,
        &format!(
            "Hand edit\n\nAthena-Job: {forged_job}\nAthena-Prediction: {prediction}\n{VERIFIED_CONTENT}: {fp}\n"
        ),
    );
    let (hook, history) = refusals(&fx, &athena);
    assert!(
        hook.iter().any(|v| v.contains("governed content")),
        "the commit-msg hook refuses it: {hook:?}"
    );
    assert!(
        history.iter().any(|v| v.contains("governed content")),
        "the history check finds it: {history:?}"
    );

    // Naming the real job instead, which already landed, fails too.
    fx.git.run(["reset", "-q", "--hard", &parent]).unwrap();
    write(fx.root(), &[("README.md", "# Demo\n\nedited by hand\n")]);
    commit_by_hand(
        &fx,
        &format!(
            "Hand edit\n\nAthena-Job: {}\nAthena-Prediction: {prediction}\n{VERIFIED_CONTENT}: {fp}\n",
            proposal.job.as_str()
        ),
    );
    let (hook, _) = refusals(&fx, &athena);
    assert!(
        hook.iter().any(|v| v.contains("governed content")),
        "{hook:?}"
    );
}

#[test]
fn hand_written_scores_and_outcomes_are_refused() {
    // M27 (blocks), acceptance verification 1, M2: a re-score that no intervention caused, a re-score
    // that does not follow from the records, an edited closing score, and a landed outcome for a rejected
    // job are refused by the hook and found by the history check.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.verdict = serde_json::json!({
        "verdict": "fail",
        "criteria": [
            { "criterion": "criterion 1", "met": false, "evidence": "missing", "commands": [] },
            { "criterion": "criterion 2", "met": true, "evidence": "ran", "commands": ["cargo test"] }
        ],
        "findings": [],
        "request_reading_mismatch": null,
        "summary": "The greeting is missing."
    });
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let rejected = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(
        rejected.state,
        OutcomeState::Rejected,
        "{}",
        rejected.reason
    );
    assert!(athena.check_history().unwrap().violations.is_empty());
    let clean = fx.head();
    let dir = proposal.job.dir();
    let score = rejected.score.clone().unwrap();
    let perfect = |mut s: athena_data::records::Score| {
        for e in s.events.iter_mut() {
            for b in e.brier.values_mut() {
                *b = 0.0;
            }
        }
        s
    };

    // A re-score naming an intervention that does not exist.
    let cause = athena_data::RecordId::parse("20261010-150000-abcd").unwrap();
    let mut forged = perfect(score.clone());
    forged.cause = Some(cause.clone());
    write(
        fx.root(),
        &[(
            &format!("{dir}/score-{cause}.json"),
            &AnyRecord::Score(Box::new(forged.clone())).to_json(),
        )],
    );
    commit_by_hand(
        &fx,
        &format!("Re-score\n\nAthena-Job: {}\n", proposal.job.as_str()),
    );
    let (hook, history) = refusals(&fx, &athena);
    assert!(hook.iter().any(|v| v.contains("re-score")), "{hook:?}");
    assert!(
        history.iter().any(|v| v.contains("re-score")),
        "{history:?}"
    );

    // The same perfect re-score, with a real intervention in the same commit: it does not follow from
    // the records.
    fx.git.run(["reset", "-q", "--hard", &clean]).unwrap();
    let intervention = athena_data::records::Intervention {
        id: cause.clone(),
        at: "2026-10-10T15:00:00Z".parse().unwrap(),
        what: "Fixed the greeting by hand".into(),
        why: "The job was rejected".into(),
        class_of_work: "docs".into(),
        category: athena_data::records::InterventionCategory::DidWorkThemselves,
        job: Some(proposal.job.clone()),
        source: athena_data::records::InterventionSource::User,
        with_staged_changes: false,
    };
    let record = AnyRecord::Intervention(intervention);
    write(
        fx.root(),
        &[
            (&record.canonical_path(), &record.to_json()),
            (
                &format!("{dir}/score-{cause}.json"),
                &AnyRecord::Score(Box::new(forged)).to_json(),
            ),
        ],
    );
    commit_by_hand(
        &fx,
        &format!(
            "Re-score\n\nAthena-Intervention: {cause}\nAthena-Job: {}\n",
            proposal.job.as_str()
        ),
    );
    let (hook, history) = refusals(&fx, &athena);
    assert!(
        hook.iter().any(|v| v.contains("does not follow")),
        "{hook:?}"
    );
    assert!(
        history.iter().any(|v| v.contains("does not follow")),
        "{history:?}"
    );

    // A landed outcome for the rejected job.
    fx.git.run(["reset", "-q", "--hard", &clean]).unwrap();
    let AnyRecord::Outcome(mut outcome) =
        record_at(&fx, &clean, &format!("{dir}/outcome.json")).unwrap()
    else {
        panic!("an outcome")
    };
    outcome.state = OutcomeState::Landed;
    outcome.after_blocked = true;
    write(
        fx.root(),
        &[(
            &format!("{dir}/outcome-landed.json"),
            &AnyRecord::Outcome(outcome).to_json(),
        )],
    );
    commit_by_hand(
        &fx,
        &format!(
            "Landed after all\n\nAthena-Job: {}\n",
            proposal.job.as_str()
        ),
    );
    let (hook, history) = refusals(&fx, &athena);
    assert!(hook.iter().any(|v| v.contains("blocked")), "{hook:?}");
    assert!(history.iter().any(|v| v.contains("blocked")), "{history:?}");
}

#[test]
fn an_intervention_during_the_job_is_scored_when_it_closes() {
    // M27 (allows): the closing score reads the interventions on the commit it is built on, so an
    // intervention recorded while the job ran resolves `user_intervenes`, and the gate agrees.
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    let root = fx.root().to_path_buf();
    let job_slot: Arc<std::sync::Mutex<Option<String>>> = Arc::new(std::sync::Mutex::new(None));
    let slot = job_slot.clone();
    scenario.during_verification = Some(Arc::new(move || {
        let job = slot.lock().unwrap().clone().unwrap();
        let git = athena_git::Git::at(&root);
        let now = jiff::Timestamp::now();
        let record = format!(
            r#"{{
  "schema": "athena.intervention/1",
  "id": "20261010-170000-abcd",
  "at": "{now}",
  "what": "Told the producer which file to change",
  "why": "It was taking long",
  "class_of_work": "docs",
  "category": "gave-detailed-instructions",
  "job": "{job}",
  "source": "user",
  "with_staged_changes": false
}}
"#
        );
        write(
            &root,
            &[(
                "records/interventions/2026-10/20261010-170000-abcd.json",
                &record,
            )],
        );
        git.run(["add", "-A"]).unwrap();
        git.run([
            "commit",
            "-q",
            "-m",
            "Record an intervention\n\nAthena-Intervention: 20261010-170000-abcd",
        ])
        .unwrap();
    }));
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    *job_slot.lock().unwrap() = Some(proposal.job.as_str().to_string());
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    let event = f
        .score
        .as_ref()
        .unwrap()
        .events
        .iter()
        .find(|e| e.event == "user_intervenes")
        .cloned()
        .unwrap();
    assert_eq!(event.outcome, Some(true));
    let report = athena.check_history().unwrap();
    assert!(report.violations.is_empty(), "{:?}", report.violations);
}

#[test]
fn a_queued_request_is_open_work_until_it_is_read_and_run() {
    // Capability 10, point 3 of the fix round after acceptance verification 1: `ask --later` records the
    // request with no model call; `status` lists it as open work with its next command; `go` reads it,
    // and a second `go` runs it. A queued request that is not wanted is dropped by `close`.
    let fx = Fixture::new();
    let scenario = Scenario::default();
    let calls = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let (job, _) = athena.queue("add a greeting to the readme").unwrap();
    let (unwanted, _) = athena.queue("translate the readme into Latin").unwrap();
    assert!(calls.lock().unwrap().is_empty(), "no model was called");
    let status = athena.status().unwrap();
    assert!(status.recent.is_empty(), "queued requests have not run");
    for id in [&job, &unwanted] {
        assert!(
            status
                .open
                .iter()
                .any(|l| l.id == id.as_str() && l.state == "queued"),
            "{:?}",
            status.open
        );
        assert!(
            status
                .next
                .iter()
                .any(|n| n.starts_with(&format!("athena go {}", id.as_str()))),
            "{:?}",
            status.next
        );
    }

    let GoResult::Reread(proposal) = athena
        .go(job.as_str(), AcceptedBy::User, &Lines::default())
        .unwrap()
    else {
        panic!("go reads a queued request first");
    };
    assert!(proposal.reading.is_some());
    assert_eq!(*calls.lock().unwrap(), vec![Role::Reader]);
    let f = finished(
        athena
            .go(job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);

    let closed = athena
        .close(unwanted.as_str(), "not wanted after all", &Lines::default())
        .unwrap();
    assert_eq!(closed.state, OutcomeState::Dropped);
    let status = athena.status().unwrap();
    assert!(status.open.is_empty());
    let dropped = status
        .recent
        .iter()
        .find(|l| l.id == unwanted.as_str())
        .unwrap();
    assert!(
        dropped.cost_usd.is_sign_positive(),
        "a job that never ran costs $0, not $-0"
    );
    let report = athena.check_history().unwrap();
    assert!(report.violations.is_empty(), "{:?}", report.violations);
}

#[test]
fn merging_a_branch_where_a_job_landed_passes() {
    // M27 (allows): a merge brings in a job's records from the branch where they were judged.
    let fx = Fixture::new();
    fx.git.run(["checkout", "-q", "-b", "side"]).unwrap();
    let athena = fx.athena(Scenario::default().script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    fx.git.run(["checkout", "-q", "main"]).unwrap();
    fx.git
        .run([
            "merge",
            "-q",
            "--no-ff",
            "-m",
            "Merge the side branch",
            "side",
        ])
        .unwrap();
    let report = athena.check_history().unwrap();
    assert!(report.violations.is_empty(), "{:?}", report.violations);
}

// ---------------------------------------------------------------------------------------------------------
// Closing a job deletes its directory, build cache included, and reports the space reclaimed.

/// Size of the planted build file: several blocks, so its allocation is clearly visible.
const PLANTED: usize = 300_000;

fn jobs_dir(fx: &Fixture) -> PathBuf {
    fx.state.path().join("repo").join("jobs")
}

fn job_dir_of(fx: &Fixture, job: &athena_data::JobId) -> PathBuf {
    jobs_dir(fx).join(job.as_str())
}

/// Gives the job directory a lock file and a build cache holding one large file.
fn plant_cache(dir: &Path) {
    std::fs::create_dir_all(dir.join("target/debug")).unwrap();
    std::fs::write(dir.join("job.lock"), b"").unwrap();
    std::fs::write(dir.join("target/debug/big"), vec![7u8; PLANTED]).unwrap();
}

/// What the directory occupies on disk, measured here and not by the code under test: allocated blocks of
/// every entry, each inode once, symlinks not followed.
fn occupied(path: &Path) -> u64 {
    fn walk(path: &Path, seen: &mut std::collections::HashSet<(u64, u64)>) -> u64 {
        let Ok(meta) = std::fs::symlink_metadata(path) else {
            return 0;
        };
        let mut total = if seen.insert((meta.dev(), meta.ino())) {
            meta.blocks() * 512
        } else {
            0
        };
        if meta.is_dir() {
            for entry in std::fs::read_dir(path).unwrap().flatten() {
                total += walk(&entry.path(), seen);
            }
        }
        total
    }
    walk(path, &mut Default::default())
}

/// A script that, whenever the producer runs, leaves a build cache in every job directory.
fn planting(base: athena_agent::fake::Script, jobs: PathBuf) -> athena_agent::fake::Script {
    Box::new(move |spec| {
        if spec.role == Role::Producer {
            for entry in std::fs::read_dir(&jobs).unwrap().flatten() {
                plant_cache(&entry.path());
            }
        }
        base(spec)
    })
}

fn said(ui: &Lines) -> Vec<String> {
    ui.0.lock().unwrap().clone()
}

/// The text after `reclaimed ` on the one line that reports it.
fn reclaimed(lines: &[String]) -> String {
    let found: Vec<&String> = lines
        .iter()
        .filter(|l| l.starts_with("reclaimed "))
        .collect();
    assert_eq!(found.len(), 1, "one reclaimed line: {lines:?}");
    found[0]["reclaimed ".len()..].to_string()
}

/// The reported figure in bytes, read back from its unit (to one decimal, so approximate).
fn bytes_of(figure: &str) -> f64 {
    let (value, unit) = figure.split_once(' ').expect("a number and a unit");
    let scale = match unit {
        "B" => 1.0,
        "KB" => 1e3,
        "MB" => 1e6,
        "GB" => 1e9,
        "TB" => 1e12,
        other => panic!("unknown unit {other}"),
    };
    value.parse::<f64>().unwrap() * scale
}

/// Runs a job to its end with a build cache planted by the producer; returns its job, result and output.
fn go_with_cache(
    fx: &Fixture,
    scenario: Scenario,
) -> (athena_data::JobId, athena_core::run::Finished, Lines) {
    go_with_script(fx, scenario.script())
}

fn go_with_script(
    fx: &Fixture,
    script: athena_agent::fake::Script,
) -> (athena_data::JobId, athena_core::run::Finished, Lines) {
    let athena = fx.athena(planting(script, jobs_dir(fx)));
    let ui = Lines::default();
    let proposal = athena.ask("add a greeting", &ui).unwrap();
    std::fs::create_dir_all(job_dir_of(fx, &proposal.job)).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &ui)
            .unwrap(),
    );
    (proposal.job, f, ui)
}

fn assert_cache_gone_and_reported(fx: &Fixture, job: &athena_data::JobId, ui: &Lines) {
    assert!(
        !job_dir_of(fx, job).exists(),
        "the closed job's directory is gone"
    );
    let figure = reclaimed(&said(ui));
    assert!(
        bytes_of(&figure) >= PLANTED as f64 * 0.99,
        "reclaimed {figure} covers the planted {PLANTED} bytes"
    );
    assert!(
        !said(ui).iter().any(|l| l.contains("warning")),
        "{:?}",
        said(ui)
    );
}

#[test]
fn a_landed_job_deletes_its_build_cache_and_reports_the_space() {
    let fx = Fixture::new();
    let (job, f, ui) = go_with_cache(&fx, Scenario::default());
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    assert_cache_gone_and_reported(&fx, &job, &ui);
}

#[test]
fn a_rejected_job_deletes_its_build_cache_and_reports_the_space() {
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.verdict = failing_verdict("the greeting is missing");
    let (job, f, ui) = go_with_cache(&fx, scenario);
    assert_eq!(f.state, OutcomeState::Rejected);
    assert_cache_gone_and_reported(&fx, &job, &ui);
}

#[test]
fn a_failed_job_deletes_its_build_cache_and_reports_the_space() {
    let fx = Fixture::new();
    let (job, f, ui) = go_with_script(&fx, killed_producer());
    assert_eq!(f.state, OutcomeState::Failed);
    assert_cache_gone_and_reported(&fx, &job, &ui);
}

/// A scenario whose local edit during verification blocks the landing.
fn blocking_scenario(root: PathBuf) -> Scenario {
    let mut scenario = Scenario::default();
    scenario.during_verification = Some(Arc::new(move || {
        std::fs::write(root.join("README.md"), "# Demo\n\nmy own local edit\n").unwrap();
    }));
    scenario
}

#[test]
fn a_blocked_job_keeps_its_directory_until_it_is_closed_and_closing_reports_the_exact_space() {
    let fx = Fixture::new();
    let (job, f, ui) = go_with_cache(&fx, blocking_scenario(fx.root().to_path_buf()));
    assert_eq!(f.state, OutcomeState::Blocked, "{}", f.reason);
    let dir = job_dir_of(&fx, &job);
    assert!(
        dir.join("target/debug/big").exists(),
        "a blocked job keeps its build cache"
    );
    assert!(
        !said(&ui).iter().any(|l| l.contains("reclaimed")),
        "{:?}",
        said(&ui)
    );

    // Closing it abandons it and deletes the directory; the figure is what the directory occupied.
    let athena = fx.athena(Scenario::default().script());
    let expected = occupied(&dir);
    assert!(expected >= PLANTED as u64);
    let ui = Lines::default();
    let closed = athena
        .close(job.as_str(), "not going to resolve the edit", &ui)
        .unwrap();
    assert_eq!(closed.state, OutcomeState::Abandoned);
    assert!(!dir.exists());
    assert_eq!(
        reclaimed(&said(&ui)),
        athena_core::state::format_bytes(expected)
    );
}

#[test]
fn closing_an_interrupted_job_deletes_its_build_cache_and_reports_the_exact_space() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let job = proposal.job.to_string();
    athena
        .go(&job, AcceptedBy::User, &Lines::default())
        .unwrap();
    // Cut the history back to just before the closing commit, as after a restart mid-run.
    for _ in 0..6 {
        let records = athena_core::store::Records::load(&fx.git, &fx.head()).unwrap();
        if records.jobs[&job].state() == athena_core::store::JobState::Dispatched {
            break;
        }
        fx.git.run(["reset", "-q", "--hard", "HEAD~1"]).unwrap();
    }
    let dir = job_dir_of(&fx, &proposal.job);
    plant_cache(&dir);
    let expected = occupied(&dir);
    assert!(expected >= PLANTED as u64);
    let ui = Lines::default();
    let closed = athena.close(&job, "interrupted", &ui).unwrap();
    assert_eq!(closed.state, OutcomeState::Abandoned);
    assert!(!dir.exists());
    assert_eq!(
        reclaimed(&said(&ui)),
        athena_core::state::format_bytes(expected)
    );
}

#[test]
fn closing_a_dropped_proposal_with_nothing_but_a_lock_reports_its_small_size_and_no_warning() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let dir = job_dir_of(&fx, &proposal.job);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("job.lock"), b"").unwrap();
    let expected = occupied(&dir);
    let ui = Lines::default();
    athena.close(proposal.job.as_str(), "no", &ui).unwrap();
    assert!(!dir.exists());
    assert_eq!(
        reclaimed(&said(&ui)),
        athena_core::state::format_bytes(expected)
    );
    assert!(!said(&ui).iter().any(|l| l.contains("warning")));
}

#[test]
fn a_directory_that_cannot_be_removed_gives_a_warning_naming_it_and_the_close_still_succeeds() {
    let fx = Fixture::new();
    let mut athena = fx.athena(Scenario::default().script());
    athena.state = athena.state.clone().with_remover(|_| {
        Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "injected failure",
        ))
    });
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let dir = job_dir_of(&fx, &proposal.job);
    plant_cache(&dir);
    let ui = Lines::default();
    let closed = athena.close(proposal.job.as_str(), "no", &ui).unwrap();
    assert_eq!(closed.state, OutcomeState::Dropped);
    let lines = said(&ui);
    let warning = lines
        .iter()
        .find(|l| l.starts_with("warning"))
        .unwrap_or_else(|| panic!("a warning: {lines:?}"));
    assert!(warning.contains(&dir.display().to_string()), "{warning}");
    assert!(warning.contains("injected failure"), "{warning}");
    assert!(
        !lines.iter().any(|l| l.contains("reclaimed")),
        "no space is claimed: {lines:?}"
    );
    assert!(dir.join("target/debug/big").exists());

    // Control: the default remover removes the same directory without complaint.
    let default = athena_core::state::StateDir::new(fx.state.path().join("repo"));
    let report = default.clean_job(&proposal.job);
    assert_eq!(report.error, None);
    assert!(report.bytes >= PLANTED as u64);
    assert!(!dir.exists());
}

#[test]
fn a_close_that_fails_to_commit_keeps_the_job_directory_and_claims_nothing() {
    let fx = Fixture::new();
    let athena = fx.athena(killed_producer());
    let (job, _path) = interrupted_job_with_legacy_record(&fx, &athena);
    let dir = job_dir_of(&fx, &job);
    plant_cache(&dir);
    let ui = Lines::default();
    let error = athena.close(job.as_str(), "stuck", &ui).unwrap_err();
    assert!(
        format!("{error:#}").contains("usage records reconcile"),
        "{error:#}"
    );
    assert!(dir.join("target/debug/big").exists(), "nothing was removed");
    assert!(
        !said(&ui).iter().any(|l| l.contains("reclaimed")),
        "{:?}",
        said(&ui)
    );
}
