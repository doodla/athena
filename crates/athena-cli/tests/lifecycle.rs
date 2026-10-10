//! Whole jobs, from request to landed commit, against the fake driver (design §8).

// Scenarios are set up field by field, so each test shows only what differs from the default.
#![allow(clippy::field_reassign_with_default)]

mod common;

use athena_core::run::GoResult;
use athena_data::records::{AcceptedBy, AnyRecord, OutcomeState, Role, Verdict};
use athena_gate::invariant::{PREDICTION, VERIFIED_CONTENT};
use common::*;
use std::sync::Arc;

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
    let raised = test_config().replace("job_usd = 8.00", "job_usd = 9.00");
    scenario.reading = reading(
        &[("athena.toml", "modify")],
        &[
            ("content", "athena.toml raises job_usd to 9.00"),
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
    assert!(fx.file("athena.toml").unwrap().contains("job_usd = 9.00"));
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
    let fx = Fixture::with_config(&test_config().replace("job_usd = 8.00", "job_usd = 4.00"));
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
    scenario.during_verification = Some(Arc::new(move || {
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
    assert!(!fx.file("README.md").unwrap().contains("Hello, reader."));
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
