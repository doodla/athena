//! Whole jobs, from request to landed commit, against the fake driver (design §8).

// Scenarios are set up field by field, so each test shows only what differs from the default.
#![allow(clippy::field_reassign_with_default)]

mod common;

use athena_core::run::GoResult;
use athena_data::records::{AcceptedBy, AnyRecord, OutcomeState, Role, Verdict};
use athena_gate::invariant::{PREDICTION, VERIFIED_CONTENT};
use common::*;
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
        Some(AnyRecord::Candidate(c)) => c.base_commit.clone(),
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
        &["cargo test PRODUCER-CANARY"],
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
fn a_failing_check_rejects_the_job_without_spending_a_verifier() {
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
    assert!(!calls.lock().unwrap().contains(&Role::Verifier));
    assert!(fx.file("README.md").is_some(), "nothing landed");
    assert!(athena.check_history().unwrap().violations.is_empty());
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
    let first_script = first_scenario.script();
    let second_script = second_scenario.script();
    let verifier_calls = Arc::new(Mutex::new(0usize));
    let readers = Arc::new(Mutex::new(Vec::new()));
    let kept = readers.clone();
    let script: athena_agent::fake::Script = Box::new(move |spec| {
        if spec.role == Role::Reader {
            kept.lock().unwrap().push(spec.user_prompt.clone());
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
