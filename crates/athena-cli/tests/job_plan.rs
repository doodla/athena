//! Job planning, plumbing only (docs/design/job-planning.md, build job 3a): with `[job_plan]` enabled, one
//! accepted planning round runs between the prediction commit and the producer. The fake planner and critic
//! answer deterministically.

// Scenarios are set up field by field, so each test shows only what differs from the default.
#![allow(clippy::field_reassign_with_default)]

mod common;

use athena_context::briefs::render_plan_section;
use athena_core::run::GoResult;
use athena_data::records::{AcceptedBy, AnyRecord, JobPlan, OutcomeState, Role};
use athena_data::sha256_hex;
use common::*;

fn finished(result: GoResult) -> athena_core::run::Finished {
    match result {
        GoResult::Finished(f) => *f,
        GoResult::Reread(_) => panic!("expected the job to finish"),
    }
}

fn planned_config() -> String {
    format!("{}\n[job_plan]\nenabled = true\n", test_config())
}

/// The names of the job's records under `invocations/` in the commit `rev`.
fn invocation_names(fx: &Fixture, rev: &str, dir: &str) -> Vec<String> {
    let prefix = format!("{dir}/invocations/");
    let mut names: Vec<String> = fx
        .git
        .ls_tree_under(rev, &prefix)
        .unwrap()
        .into_iter()
        .map(|e| e.path[prefix.len()..].to_string())
        .collect();
    names.sort();
    names
}

#[test]
fn an_enabled_job_runs_the_planner_and_critic_before_the_producer_and_commits_their_records() {
    let fx = Fixture::with_config(&planned_config());
    let athena = fx.athena(Scenario::default().script());
    let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    let head = fx.head();
    let dir = proposal.job.dir();
    assert_eq!(
        invocation_names(&fx, &head, &dir),
        [
            "01-reader.json",
            "02-planner.json",
            "03-critic.json",
            "04-producer.json",
            "05-verifier.json"
        ]
    );
    for path in ["job-plan-1.json", "critique-1.json"] {
        assert!(
            fx.git.path_exists(&head, &format!("{dir}/{path}")).unwrap(),
            "{path} is in the landing commit"
        );
    }
    let report = athena.check_history().unwrap();
    assert!(report.violations.is_empty(), "{:?}", report.violations);
}

#[test]
fn a_cap_between_the_producer_alone_and_planning_plus_the_producer_stops_the_job_before_the_producer()
 {
    // Each invocation costs $0.01 (the reader's included). The producer and verifier caps are $3 and $2.
    // Left before planning: 5.025 - 0.01 = 5.015, enough. Left after planning: 5.025 - 0.03 = 4.995, not.
    let config = format!(
        "{}\n[job_plan]\nenabled = true\nplanner_usd = 0.005\ncritic_usd = 0.005\n",
        with_job_usd(&test_config(), "5.025")
    );
    let fx = Fixture::with_config(&config);
    let scenario = Scenario::default();
    let calls = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Failed);
    assert!(f.reason.contains("budget"), "{}", f.reason);
    // (A failed job is introspected afterwards, which runs one more reader-role call.)
    let calls = calls.lock().unwrap().clone();
    assert_eq!(calls[..3], [Role::Reader, Role::Planner, Role::Critic]);
    assert!(
        !calls.contains(&Role::Producer) && !calls.contains(&Role::Verifier),
        "the producer never ran: {calls:?}"
    );
    let names = invocation_names(&fx, &fx.head(), &proposal.job.dir());
    assert_eq!(
        names,
        ["01-reader.json", "02-planner.json", "03-critic.json"]
    );
    assert!(athena.check_history().unwrap().violations.is_empty());
}

#[test]
fn a_send_back_keeps_the_plan_section_and_its_hash_matches_the_plan_record() {
    let fx = Fixture::with_config(&planned_config());
    let mut scenario = Scenario::default();
    // The first report shows no full test run, so the producer is sent back once.
    scenario.report = producer_report("First try.", &[]);
    scenario.report_when_sent_back = Some(producer_report("Second try.", &["cargo test --locked"]));
    let prompts = scenario.seen_by_producer.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    let prompts = prompts.lock().unwrap().clone();
    assert_eq!(prompts.len(), 2, "the producer ran twice");
    for prompt in &prompts {
        assert!(prompt.contains("Job plan (accepted, round 1)"), "{prompt}");
    }
    assert!(prompts[1].contains("## Sent back"));

    let path = format!("{}/job-plan-1.json", proposal.job.dir());
    let bytes = fx.git.show_file("HEAD", &path).unwrap().unwrap();
    let plan: JobPlan = match AnyRecord::parse(&path, &bytes).unwrap() {
        AnyRecord::JobPlan(plan) => *plan,
        other => panic!("unexpected {other:?}"),
    };
    let section = render_plan_section(&plan);
    assert_eq!(plan.producer_section_sha256, sha256_hex(section.as_bytes()));
    assert!(
        prompts[0].ends_with(&section),
        "the first prompt is the brief plus the section"
    );
    assert!(prompts[1].contains(&section));
    let brief = fx
        .git
        .show_file("HEAD", &format!("{}/brief-producer.md", proposal.job.dir()))
        .unwrap()
        .unwrap();
    assert_eq!(
        prompts[0],
        format!("{}\n\n{section}", String::from_utf8(brief).unwrap()),
        "the committed brief and the plan record rebuild the prompt"
    );
}

/// The job's accepted plan and verification record, read from the landing commit.
fn plan_and_verification(fx: &Fixture, dir: &str) -> (JobPlan, athena_data::records::Verification) {
    let plan_path = format!("{dir}/job-plan-1.json");
    let bytes = fx.git.show_file("HEAD", &plan_path).unwrap().unwrap();
    let plan = match AnyRecord::parse(&plan_path, &bytes).unwrap() {
        AnyRecord::JobPlan(plan) => *plan,
        other => panic!("unexpected {other:?}"),
    };
    let ver_path = format!("{dir}/verification.json");
    let bytes = fx.git.show_file("HEAD", &ver_path).unwrap().unwrap();
    let verification = match AnyRecord::parse(&ver_path, &bytes).unwrap() {
        AnyRecord::Verification(v) => *v,
        other => panic!("unexpected {other:?}"),
    };
    (plan, verification)
}

#[test]
fn the_accept_path_runs_every_role_in_order_and_hands_each_the_plan_it_may_see() {
    let fx = Fixture::with_config(&planned_config());
    let scenario = Scenario::default();
    let calls = scenario.invocations.clone();
    let prompts = scenario.seen_by_producer.clone();
    let seen_by_verifier = scenario.seen_by_verifier.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    assert_eq!(
        *calls.lock().unwrap(),
        [
            Role::Reader,
            Role::Planner,
            Role::Critic,
            Role::Producer,
            Role::Verifier
        ]
    );
    let dir = proposal.job.dir();
    let (plan, verification) = plan_and_verification(&fx, &dir);
    assert!(!plan.demonstrations.is_empty() && !plan.files.is_empty());
    assert!(!plan.approach.is_empty());

    // The producer's prompt: the planned files, each demonstration's command, and the hashed section.
    let prompts = prompts.lock().unwrap().clone();
    assert_eq!(prompts.len(), 1);
    for file in &plan.files {
        assert!(prompts[0].contains(&file.path), "{}", file.path);
    }
    for d in &plan.demonstrations {
        assert!(prompts[0].contains(&d.command), "{}", d.command);
    }
    let section = render_plan_section(&plan);
    assert!(prompts[0].ends_with(&section));
    assert_eq!(plan.producer_section_sha256, sha256_hex(section.as_bytes()));

    // The verifier's brief: demonstrations and planned files, nothing of the producer-side reasoning (F5).
    let seen = seen_by_verifier.lock().unwrap().clone();
    let brief = &seen[1];
    assert!(brief.contains("## Planned demonstrations"), "{brief}");
    for file in &plan.files {
        assert!(brief.contains(&file.path), "{}", file.path);
    }
    for d in &plan.demonstrations {
        assert!(brief.contains(&d.command), "{}", d.command);
        assert!(brief.contains(&d.observes), "{}", d.observes);
        if let Some(fails) = &d.fails_without {
            assert!(brief.contains(fails), "{fails}");
        }
    }
    assert!(
        !brief.contains(&plan.approach),
        "the approach reached the verifier"
    );
    for step in &plan.steps {
        assert!(!brief.contains(step.as_str()), "{step}");
    }
    for file in &plan.files {
        assert!(!brief.contains(&file.purpose), "{}", file.purpose);
    }
    assert!(
        verification
            .inputs
            .contains(&"planned demonstrations and files".to_string()),
        "{:?}",
        verification.inputs
    );
    // The stored copy of the brief carries the same section.
    let stored = fx
        .git
        .show_file("HEAD", &format!("{dir}/brief-verifier.md"))
        .unwrap()
        .unwrap();
    let stored = String::from_utf8(stored).unwrap();
    assert!(stored.contains("## Planned demonstrations"));
    assert!(!stored.contains(&plan.approach));
}

#[test]
fn with_planning_off_the_verifier_brief_and_inputs_have_no_plan() {
    let fx = Fixture::with_config(&test_config());
    let scenario = Scenario::default();
    let seen_by_verifier = scenario.seen_by_verifier.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    assert!(!seen_by_verifier.lock().unwrap()[1].contains("Planned demonstrations"));
    let path = format!("{}/verification.json", proposal.job.dir());
    let bytes = fx.git.show_file("HEAD", &path).unwrap().unwrap();
    let AnyRecord::Verification(v) = AnyRecord::parse(&path, &bytes).unwrap() else {
        panic!("not a verification");
    };
    assert!(
        !v.inputs.iter().any(|i| i.contains("planned")),
        "{:?}",
        v.inputs
    );
}

#[test]
fn with_planning_absent_or_off_the_invocations_are_the_producer_verifier_pair_only() {
    for config in [
        test_config(),
        format!("{}\n[job_plan]\nenabled = false\n", test_config()),
    ] {
        let fx = Fixture::with_config(&config);
        let scenario = Scenario::default();
        let calls = scenario.invocations.clone();
        let athena = fx.athena(scenario.script());
        let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
        let f = finished(
            athena
                .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
                .unwrap(),
        );
        assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
        assert_eq!(
            *calls.lock().unwrap(),
            [Role::Reader, Role::Producer, Role::Verifier]
        );
        let dir = proposal.job.dir();
        assert_eq!(
            invocation_names(&fx, &fx.head(), &dir),
            ["01-reader.json", "02-producer.json", "03-verifier.json"]
        );
        for path in ["job-plan-1.json", "critique-1.json"] {
            assert!(
                !fx.git
                    .path_exists(&fx.head(), &format!("{dir}/{path}"))
                    .unwrap()
            );
        }
    }
}
