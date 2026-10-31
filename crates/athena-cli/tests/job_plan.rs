//! Job planning, plumbing only (docs/design/job-planning.md, build job 3a): with `[job_plan]` enabled, one
//! accepted planning round runs between the prediction commit and the producer. The fake planner and critic
//! answer deterministically.

// Scenarios are set up field by field, so each test shows only what differs from the default.
#![allow(clippy::field_reassign_with_default)]

mod common;

use athena_context::briefs::render_plan_section;
use athena_core::run::GoResult;
use athena_data::records::{
    AcceptedBy, AnyRecord, Critique, CritiqueVerdict, FindingSource, JobPlan, OutcomeState, Role,
    Severity,
};
use athena_data::sha256_hex;
use common::*;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

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
        "{}\n[job_plan]\nenabled = true\nplanner_usd = 0.005\ncritic_usd = 0.005\nmax_rounds = 1\n",
        with_job_usd(&test_config(), "5.025").replace("reader_usd = 0.50", "reader_usd = 0.01")
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

/// The page the drain rates after a landing.
struct StubPage;

impl athena_core::audience::Page for StubPage {
    fn fetch(&self, _: &str) -> anyhow::Result<String> {
        Ok("<h1>Start here</h1><p>firstmate hands work to agents.</p>".into())
    }
}

/// A land step that records the jobs it was asked to land.
#[derive(Default)]
struct RecordingLander(Mutex<Vec<String>>);

impl athena_cli::drain::Lander for RecordingLander {
    fn land(
        &self,
        _: &athena_core::Athena,
        job: &str,
        _: &dyn athena_core::Ui,
    ) -> athena_cli::drain::Landing {
        self.0.lock().unwrap().push(job.to_string());
        athena_cli::drain::Landing::Landed {
            changed: Vec::new(),
        }
    }
}

#[test]
fn a_short_job_is_skipped_with_its_reason_and_the_next_item_runs() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    // No `[job_plan]`: a job needs the reader 0.50, the producer 3.00 and the verifier 2.00, so 5.50 in all.
    // The budget is 6.00. The first item's reader fails once and costs 0.60 (the pattern of a reader that
    // returns no reading), so its second read, made by the drain, finds 5.40 left: $0.10 short.
    let fx = Fixture::with_config(&with_job_usd(&test_config(), "6.00"));
    let inner = Scenario::default().script();
    let first_reads = Arc::new(AtomicUsize::new(0));
    let counted = first_reads.clone();
    let script: athena_agent::fake::Script = Box::new(move |spec| {
        let mut turn = inner(spec);
        if spec.role == Role::Reader
            && spec.user_prompt.contains("first demo study")
            && counted.fetch_add(1, Ordering::SeqCst) == 0
        {
            turn.structured = None;
            turn.cost_usd = 0.60;
        }
        turn
    });
    let athena = fx.athena(script);
    // Each request carries an acceptance bullet, so the drain's plan is complete without reading it first.
    let (first, _) = athena
        .queue("Add the first demo study to studies/demo/\n- the demo study has a pin")
        .unwrap();
    let (second, _) = athena
        .queue("Add the second demo study to studies/demo/\n- the demo study has a pin")
        .unwrap();
    let lander = RecordingLander::default();
    let lines = Lines::default();
    let steps = athena_cli::drain::RealSteps {
        athena: &athena,
        ui: &lines,
        page: &StubPage,
        lander: &lander,
    };
    let limits = athena_cli::drain::Limits {
        budget_usd: 50.0,
        target_rating: 5,
        max_jobs: 10,
    };

    let report = athena_cli::drain::drain(&steps, limits, &lines).unwrap();
    let text = athena_cli::display::drain(&report);

    // The first item is held, once, with its shortfall and the config line, on one note and one log line.
    let prefix = format!("item {} is left queued:", first.as_str());
    let notes: Vec<&String> = report
        .notes
        .iter()
        .filter(|n| n.starts_with(&prefix))
        .collect();
    assert_eq!(notes.len(), 1, "{:?}", report.notes);
    assert!(notes[0].contains("$0.10 short"), "{}", notes[0]);
    assert!(notes[0].contains("[budget] job_usd"), "{}", notes[0]);
    assert!(!notes[0].contains('\n'));
    let log = lines.0.lock().unwrap().clone();
    let held: Vec<&String> = log
        .iter()
        .filter(|l| l.starts_with("drain") && l.contains("(budget short)"))
        .collect();
    assert_eq!(held.len(), 1, "{log:?}");
    assert!(held[0].contains(first.as_str()), "{}", held[0]);
    // The reader ran for it once (the drain's first read); the second read was refused before the reader.
    assert_eq!(first_reads.load(Ordering::SeqCst), 1, "{text}");
    // It is held, not a failed run, so the drain does not stop on it...
    assert_ne!(
        report.stop,
        athena_cli::drain::Stop::RepeatedFailure,
        "{text}"
    );
    assert_eq!(report.failed_item, None, "{text}");
    // ...and the next item, which has the budget, is read, run and landed.
    assert_eq!(
        *lander.0.lock().unwrap(),
        [second.as_str().to_string()],
        "{text}"
    );
    assert_eq!(report.landed, 1, "{text}");
}

#[test]
fn a_send_back_keeps_the_plan_section_and_its_hash_matches_the_plan_record() {
    let fx = Fixture::with_config(&planned_config());
    let mut scenario = Scenario::default();
    // The first report marks one acceptance criterion unmet, so the producer is sent back once.
    let texts = criteria_texts(&default_reading());
    let mut first = producer_report("First try.", &[]);
    first["criteria_check"] = json!([
        criterion_check(&texts[0], "met"),
        criterion_check(&texts[1], "unmet"),
    ]);
    scenario.report = first;
    scenario.report_when_sent_back = Some(producer_report("Second try.", &[]));
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

// ---------------------------------------------------------------------------------------------------------
// Job 4: revise and stop paths, and Athena's own checks on a plan.

/// One demonstration of the default reading's criterion `n`.
fn demo(n: u32, command: &str, fails_without: Option<&str>) -> Value {
    json!({
        "criterion": n,
        "method": "run the tests",
        "command": command,
        "observes": format!("criterion {n} holds"),
        "fails_without": fails_without,
        "seam": "the planned job script and its fake agents; exists"
    })
}

/// A plan for the default reading (two criteria, README.md to modify) with these files and demonstrations.
fn plan_of(files: Value, demonstrations: Vec<Value>) -> Value {
    json!({
        "approach": "Add a greeting line to README.md.",
        "decisions": [],
        "files": files,
        "steps": ["Edit README.md."],
        "risks": [],
        "demonstrations": demonstrations,
        "open_questions": []
    })
}

fn readme_file() -> Value {
    json!([{ "path": "README.md", "action": "modify", "purpose": "Hold the greeting." }])
}

fn good_demos() -> Vec<Value> {
    vec![
        demo(1, "cargo test --locked", Some("the greeting test fails")),
        demo(2, "cargo test --locked", Some("the suite fails")),
    ]
}

fn good_plan() -> Value {
    plan_of(readme_file(), good_demos())
}

/// A critique with these `(severity, summary)` findings.
fn critique_of(verdict: &str, findings: &[(&str, &str)], mismatch: Option<&str>) -> Value {
    json!({
        "verdict": verdict,
        "findings": findings.iter().map(|(severity, summary)| json!({
            "severity": severity,
            "summary": summary,
            "location": "demonstrations",
            "evidence": "the second demonstration is the same as the first",
            "fix": "name a test that fails without the greeting"
        })).collect::<Vec<_>>(),
        "request_reading_mismatch": mismatch,
        "summary": format!("The critic says {verdict}.")
    })
}

fn read_record(fx: &Fixture, path: &str) -> AnyRecord {
    let bytes = fx.git.show_file("HEAD", path).unwrap().unwrap();
    AnyRecord::parse(path, &bytes).unwrap()
}

fn critique_at(fx: &Fixture, dir: &str, round: u32) -> Critique {
    match read_record(fx, &format!("{dir}/critique-{round}.json")) {
        AnyRecord::Critique(c) => *c,
        other => panic!("unexpected {other:?}"),
    }
}

fn count(calls: &Arc<Mutex<Vec<Role>>>, role: Role) -> usize {
    calls.lock().unwrap().iter().filter(|r| **r == role).count()
}

#[test]
fn after_a_revise_the_second_planner_prompt_quotes_the_first_finding_and_the_job_lands() {
    const FINDING: &str = "Demonstration 2 repeats demonstration 1 and shows nothing new";
    let fx = Fixture::with_config(&planned_config());
    let mut scenario = Scenario::default();
    scenario.plans = vec![good_plan(), good_plan()];
    scenario.critiques = vec![
        critique_of("revise", &[("major", FINDING)], None),
        critique_of("accept", &[], None),
    ];
    let calls = scenario.invocations.clone();
    let briefs = scenario.seen_by_planner.clone();
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
            Role::Planner,
            Role::Critic,
            Role::Producer,
            Role::Verifier
        ]
    );
    let briefs = briefs.lock().unwrap().clone();
    assert!(!briefs[0].contains(FINDING) && !briefs[0].contains("Revision"));
    let second = &briefs[1];
    assert!(second.contains(FINDING), "{second}");
    assert!(
        second.contains("Revision: your plan for round 1"),
        "{second}"
    );
    assert!(
        second.contains("Add a greeting line to README.md."),
        "the previous plan is quoted: {second}"
    );
    let dir = proposal.job.dir();
    assert_eq!(
        critique_at(&fx, &dir, 1).final_verdict,
        CritiqueVerdict::Revise
    );
    assert_eq!(
        critique_at(&fx, &dir, 2).final_verdict,
        CritiqueVerdict::Accept
    );
    // The producer is given the accepted plan, which is the second round's.
    assert!(
        fx.git
            .path_exists("HEAD", &format!("{dir}/job-plan-2.json"))
            .unwrap()
    );
    assert!(athena.check_history().unwrap().violations.is_empty());
}

#[test]
fn when_every_round_says_revise_the_job_fails_in_planning_with_nothing_built_and_an_unknown_verification()
 {
    const FINDING: &str = "No demonstration would fail without the greeting";
    let fx = Fixture::with_config(&planned_config());
    let mut scenario = Scenario::default();
    scenario.plans = vec![good_plan(), good_plan(), good_plan()];
    scenario.critiques = (0..3)
        .map(|_| critique_of("revise", &[("major", FINDING)], None))
        .collect();
    let calls = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Failed);
    assert!(
        f.reason
            .starts_with("planning stopped before any code was written:"),
        "{}",
        f.reason
    );
    assert!(f.reason.contains(FINDING), "{}", f.reason);
    assert!(f.reason.contains("round 3 of 3"), "{}", f.reason);
    // No producer or verifier ran. (A failed job is introspected afterwards by one reader-role call.)
    let planning: Vec<Role> = calls.lock().unwrap()[..7].to_vec();
    assert_eq!(
        planning,
        [
            Role::Reader,
            Role::Planner,
            Role::Critic,
            Role::Planner,
            Role::Critic,
            Role::Planner,
            Role::Critic
        ]
    );
    assert_eq!(
        count(&calls, Role::Producer) + count(&calls, Role::Verifier),
        0
    );

    let dir = proposal.job.dir();
    for round in 1..=3 {
        for kind in ["job-plan", "critique"] {
            assert!(
                fx.git
                    .path_exists("HEAD", &format!("{dir}/{kind}-{round}.json"))
                    .unwrap(),
                "{kind}-{round}"
            );
        }
    }
    assert!(
        !fx.git
            .path_exists("HEAD", &format!("{dir}/candidate.json"))
            .unwrap()
    );
    assert!(
        !fx.git
            .path_exists("HEAD", &format!("{dir}/verification.json"))
            .unwrap()
    );
    // The totals count the reader and all six planning invocations.
    assert_eq!(f.totals.as_ref().unwrap().invocations, 7);
    // Verification never ran, so its result is unknown, not false (review F11).
    let score = f.score.as_ref().expect("a closed job is scored");
    let event = score
        .events
        .iter()
        .find(|e| e.event == "verification_passes")
        .expect("the verification event is scored");
    assert_eq!(event.outcome, None, "unknown, not false");
    assert!(athena.check_history().unwrap().violations.is_empty());
}

#[test]
fn a_reject_a_misread_reading_or_a_revise_in_the_last_round_stops_the_job() {
    // (the critic's turns, the round the job stops in, a phrase of the reason)
    let cases: [(Vec<Value>, usize, &str); 3] = [
        (
            vec![critique_of(
                "reject",
                &[("blocker", "The plan is unsound")],
                None,
            )],
            1,
            "the critic rejected the plan",
        ),
        (
            vec![critique_of(
                "accept",
                &[],
                Some("It reads the request as a refactor"),
            )],
            1,
            "misread the request: It reads the request as a refactor",
        ),
        (
            vec![
                critique_of("revise", &[("major", "Not yet")], None),
                critique_of("revise", &[("major", "Still not")], None),
            ],
            2,
            "still needed revising in the last round",
        ),
    ];
    for (critiques, rounds, phrase) in cases {
        let config = format!(
            "{}\n[job_plan]\nenabled = true\nmax_rounds = {}\n",
            test_config(),
            if rounds == 2 { 2 } else { 3 }
        );
        let fx = Fixture::with_config(&config);
        let mut scenario = Scenario::default();
        scenario.plans = vec![good_plan(); 3];
        scenario.critiques = critiques;
        let calls = scenario.invocations.clone();
        let athena = fx.athena(scenario.script());
        let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
        let f = finished(
            athena
                .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
                .unwrap(),
        );
        assert_eq!(f.state, OutcomeState::Failed, "{phrase}");
        assert!(f.reason.contains(phrase), "{}", f.reason);
        assert_eq!(count(&calls, Role::Planner), rounds, "{phrase}");
        assert_eq!(count(&calls, Role::Critic), rounds, "{phrase}");
        assert_eq!(count(&calls, Role::Producer), 0, "{phrase}");
    }
}

#[test]
fn a_plan_that_edits_an_unlisted_governance_file_gets_an_athena_critique_and_no_critic_call() {
    let fx = Fixture::with_config(&planned_config());
    let mut scenario = Scenario::default();
    scenario.plans = vec![
        plan_of(
            json!([
                { "path": "README.md", "action": "modify", "purpose": "Hold the greeting." },
                { "path": "athena.toml", "action": "modify", "purpose": "Turn something on." }
            ]),
            good_demos(),
        ),
        good_plan(),
    ];
    scenario.critiques = vec![critique_of("accept", &[], None)];
    let calls = scenario.invocations.clone();
    let briefs = scenario.seen_by_planner.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    // The first plan went back to the planner without the critic and without using a round; only the
    // revised plan reached the critic, as round 1.
    assert_eq!(
        *calls.lock().unwrap(),
        [
            Role::Reader,
            Role::Planner,
            Role::Planner,
            Role::Critic,
            Role::Producer,
            Role::Verifier
        ]
    );
    // Critique 1 is the critic's own, of the revised plan.
    let first = critique_at(&fx, &proposal.job.dir(), 1);
    assert!(
        first.invocation_seq.is_some(),
        "the critic reviewed round 1"
    );
    assert_eq!(first.final_verdict, CritiqueVerdict::Accept);
    // The planner is shown Athena's finding in its second brief.
    let briefs = briefs.lock().unwrap();
    assert_eq!(briefs.len(), 2);
    assert!(!briefs[0].contains("athena.toml is a governance file"));
    assert!(briefs[1].contains("athena.toml is a governance file"));
}

#[test]
fn a_plan_citing_a_command_outside_the_verifiers_grant_gets_an_athena_critique_and_no_critic_call()
{
    let fx = Fixture::with_config(&planned_config());
    let mut scenario = Scenario::default();
    scenario.plans = vec![
        plan_of(
            readme_file(),
            vec![
                demo(1, "cargo test --locked", Some("the greeting test fails")),
                demo(2, "python3 check_greeting.py", Some("the check fails")),
            ],
        ),
        good_plan(),
    ];
    let calls = scenario.invocations.clone();
    let briefs = scenario.seen_by_planner.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    assert_eq!(
        count(&calls, Role::Critic),
        1,
        "only the second plan reached the critic"
    );
    // Critique 1 is the critic's own; Athena's finding went to the planner's second brief.
    let first = critique_at(&fx, &proposal.job.dir(), 1);
    assert!(
        first.invocation_seq.is_some(),
        "the critic reviewed round 1"
    );
    let briefs = briefs.lock().unwrap();
    assert!(!briefs[0].contains("cites a command the verifier may not run"));
    assert!(
        briefs[1].contains("cites a command the verifier may not run")
            && briefs[1].contains("python3 check_greeting.py"),
        "{}",
        briefs[1]
    );
}

#[test]
fn athena_checks_a_demonstration_for_every_criterion_and_a_fails_without_for_behavior_ones() {
    let fx = Fixture::with_config(&planned_config());
    let mut scenario = Scenario::default();
    scenario.plans = vec![
        // Criterion 2 (behavior) has no fails_without, and criterion 1 is shown twice.
        plan_of(
            readme_file(),
            vec![
                demo(1, "cargo test --locked", Some("x")),
                demo(1, "cargo test --locked", Some("y")),
                demo(2, "cargo test --locked", None),
            ],
        ),
        // Criterion 2 is not shown at all.
        plan_of(
            readme_file(),
            vec![demo(1, "cargo test --locked", Some("x"))],
        ),
        good_plan(),
    ];
    let calls = scenario.invocations.clone();
    let briefs = scenario.seen_by_planner.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    // Two mechanical revisions, then the third plan reaches the critic, as round 1.
    assert_eq!(count(&calls, Role::Planner), 3);
    assert_eq!(count(&calls, Role::Critic), 1);
    assert!(
        critique_at(&fx, &proposal.job.dir(), 1)
            .invocation_seq
            .is_some(),
        "critique 1 is the critic's"
    );
    // Each revision brief carries the findings on the plan before it.
    let briefs = briefs.lock().unwrap();
    assert_eq!(briefs.len(), 3);
    assert!(
        briefs[1].contains("Criterion 1 has 2 demonstrations"),
        "{}",
        briefs[1]
    );
    assert!(
        briefs[1].contains("does not say what fails without"),
        "{}",
        briefs[1]
    );
    assert!(
        briefs[2].contains("Criterion 2 has 0 demonstrations"),
        "{}",
        briefs[2]
    );
}

#[test]
fn athena_checks_that_files_to_modify_exist_and_files_to_create_do_not() {
    let fx = Fixture::with_config(&planned_config());
    let mut scenario = Scenario::default();
    scenario.plans = vec![
        plan_of(
            json!([
                { "path": "NOPE.md", "action": "modify", "purpose": "Not there." },
                { "path": "README.md", "action": "create", "purpose": "Already there." }
            ]),
            good_demos(),
        ),
        good_plan(),
    ];
    let briefs = scenario.seen_by_planner.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    // Critique 1 is the critic's own; both of Athena's findings went to the planner's second brief.
    assert!(
        critique_at(&fx, &proposal.job.dir(), 1)
            .invocation_seq
            .is_some(),
        "critique 1 is the critic's"
    );
    let second = briefs.lock().unwrap()[1].clone();
    assert!(
        second.contains("The plan changes NOPE.md, which does not exist"),
        "{second}"
    );
    assert!(
        second.contains("The plan creates README.md, which already exists"),
        "{second}"
    );
}

#[test]
fn a_plan_that_lists_a_directory_gets_a_major_athena_finding_and_the_job_goes_to_the_next_round() {
    // `src` is a directory in the fixture's base commit; `docs/` is a path ending in a slash.
    for (dir_path, action) in [("docs/", "create"), ("src", "create"), ("src", "modify")] {
        let fx = Fixture::with_config(&planned_config());
        let mut scenario = Scenario::default();
        scenario.plans = vec![
            plan_of(
                json!([
                    { "path": "README.md", "action": "modify", "purpose": "Hold the greeting." },
                    { "path": dir_path, "action": action, "purpose": "A directory." }
                ]),
                good_demos(),
            ),
            good_plan(),
        ];
        let calls = scenario.invocations.clone();
        let briefs = scenario.seen_by_planner.clone();
        let athena = fx.athena(scenario.script());
        let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
        let f = finished(
            athena
                .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
                .unwrap(),
        );
        assert_eq!(f.state, OutcomeState::Landed, "{dir_path}: {}", f.reason);
        assert_eq!(count(&calls, Role::Planner), 2, "{dir_path}");
        assert_eq!(count(&calls, Role::Critic), 1, "{dir_path}");
        // Critique 1 is the critic's own; Athena's finding went to the planner's second brief.
        let first = critique_at(&fx, &proposal.job.dir(), 1);
        assert!(
            first.invocation_seq.is_some(),
            "the critic reviewed round 1"
        );
        assert_eq!(first.final_verdict, CritiqueVerdict::Accept);
        let second = briefs.lock().unwrap()[1].clone();
        assert!(
            second.contains(&format!("The plan lists {dir_path:?}")),
            "{dir_path}: {second}"
        );
    }
}

#[test]
fn a_file_outside_the_deliverables_is_a_minor_finding_that_does_not_send_the_plan_back() {
    let fx = Fixture::with_config(&planned_config());
    let mut scenario = Scenario::default();
    scenario.plans = vec![plan_of(
        json!([
            { "path": "README.md", "action": "modify", "purpose": "Hold the greeting." },
            { "path": "NOTES.md", "action": "create", "purpose": "Also write notes." }
        ]),
        good_demos(),
    )];
    let calls = scenario.invocations.clone();
    let briefs = scenario.seen_by_planner.clone();
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    assert_eq!(count(&calls, Role::Planner), 1);
    assert_eq!(count(&calls, Role::Critic), 1);
    assert_eq!(briefs.lock().unwrap().len(), 1);
    let critique = critique_at(&fx, &proposal.job.dir(), 1);
    assert_eq!(critique.final_verdict, CritiqueVerdict::Accept);
    assert_eq!(critique.findings.len(), 1, "{:?}", critique.findings);
    assert_eq!(critique.findings[0].severity, Severity::Minor);
    assert_eq!(critique.findings[0].source, FindingSource::Athena);
    assert!(critique.findings[0].summary.contains("NOTES.md"));
}

#[test]
fn a_retry_of_a_job_stopped_in_planning_carries_the_last_critiques_findings() {
    const FIRST: &str = "The first round finding about demonstrations";
    const LAST: &str = "The last round finding about the greeting test";
    let fx = Fixture::with_config(&planned_config());
    let mut scenario = Scenario::default();
    scenario.plans = vec![good_plan(); 3];
    scenario.critiques = vec![
        critique_of("revise", &[("major", FIRST)], None),
        critique_of("revise", &[("major", "Middle")], None),
        critique_of("revise", &[("major", LAST)], None),
    ];
    let readers: Arc<Mutex<Vec<String>>> = Arc::default();
    let inner = scenario.script();
    let seen = readers.clone();
    let athena = fx.athena(Box::new(move |spec| {
        if spec.role == Role::Reader {
            seen.lock().unwrap().push(spec.user_prompt.clone());
        }
        inner(spec)
    }));
    let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(f.state, OutcomeState::Failed);

    athena
        .retry(proposal.job.as_str(), &Lines::default())
        .unwrap();
    let brief = readers.lock().unwrap().last().cloned().unwrap();
    assert!(brief.contains("# Earlier attempt"), "{brief}");
    assert!(
        brief.contains(LAST),
        "the last critique's finding is carried: {brief}"
    );
    assert!(!brief.contains(FIRST), "earlier critiques are not: {brief}");
    assert!(
        brief.contains("planning stopped before any code was written"),
        "{brief}"
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

// ---------------------------------------------------------------------------------------------------------
// Hard-jobs-only planning (D143): the repository's own `[job_plan]` plans a job whose reading is marked hard.

/// [`test_config`] with the repository's own `[job_plan]` table from `athena.toml`, as the file sets it.
fn repository_job_plan_config() -> String {
    let raw = include_str!("../../../athena.toml");
    let start = raw.find("[job_plan]").expect("athena.toml has [job_plan]");
    let table: String = raw[start..]
        .split_inclusive('\n')
        .enumerate()
        .take_while(|(i, line)| *i == 0 || !line.trim_start().starts_with('['))
        .map(|(_, line)| line)
        .collect();
    format!("{}\n{table}", test_config())
}

/// A scripted reader that marks its reading hard, or not.
fn reader_scenario(hard_reason: Value) -> Scenario {
    let mut scenario = Scenario::default();
    scenario.reading["hard_reason"] = hard_reason;
    scenario
}

#[test]
fn with_athena_toml_planning_hard_jobs_only_a_hard_reading_runs_the_planner_and_critic_and_an_ordinary_one_does_not()
 {
    let config = repository_job_plan_config();
    let plan = athena_data::Config::parse(&config)
        .unwrap()
        .job_plan
        .unwrap();
    assert!(plan.enabled && plan.mode == athena_data::config::PlanMode::Hard);

    for (hard_reason, roles, names) in [
        (
            json!("changes a rule that can hold a drain"),
            vec![
                Role::Reader,
                Role::Planner,
                Role::Critic,
                Role::Producer,
                Role::Verifier,
            ],
            vec![
                "01-reader.json",
                "02-planner.json",
                "03-critic.json",
                "04-producer.json",
                "05-verifier.json",
            ],
        ),
        (
            json!(null),
            vec![Role::Reader, Role::Producer, Role::Verifier],
            vec!["01-reader.json", "02-producer.json", "03-verifier.json"],
        ),
    ] {
        let fx = Fixture::with_config(&config);
        let scenario = reader_scenario(hard_reason.clone());
        let calls = scenario.invocations.clone();
        let athena = fx.athena(scenario.script());
        let (job, f) = run_job(&athena);
        assert_eq!(f.state, OutcomeState::Landed, "{hard_reason}: {}", f.reason);
        assert_eq!(*calls.lock().unwrap(), roles, "{hard_reason}");
        let dir = job.dir();
        assert_eq!(invocation_names(&fx, &fx.head(), &dir), names);
        let has_plan = hard_reason != json!(null);
        for path in ["job-plan-1.json", "critique-1.json"] {
            assert_eq!(
                fx.git
                    .path_exists(&fx.head(), &format!("{dir}/{path}"))
                    .unwrap(),
                has_plan,
                "{path} for {hard_reason}"
            );
        }
        assert!(athena.check_history().unwrap().violations.is_empty());
    }
}

#[test]
fn a_hard_reading_with_planning_enabled_for_every_job_and_one_with_planning_off_follow_the_mode() {
    // `mode = "all"` plans an ordinary reading too; `enabled = false` plans nothing, not even a hard one.
    for (table, ordinary_planned, hard_planned) in [
        ("[job_plan]\nenabled = true\nmode = \"all\"\n", true, true),
        (
            "[job_plan]\nenabled = false\nmode = \"hard\"\n",
            false,
            false,
        ),
    ] {
        for (hard_reason, expected) in [
            (json!(null), ordinary_planned),
            (json!("changes landing"), hard_planned),
        ] {
            let fx = Fixture::with_config(&format!("{}\n{table}", test_config()));
            let scenario = reader_scenario(hard_reason.clone());
            let calls = scenario.invocations.clone();
            let athena = fx.athena(scenario.script());
            let (_, f) = run_job(&athena);
            assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
            assert_eq!(
                calls.lock().unwrap().contains(&Role::Planner),
                expected,
                "{table} {hard_reason}"
            );
        }
    }
}

#[test]
fn status_reports_planned_and_unplanned_landings_later_cited_as_a_revert_or_fix_beside_the_landed_share()
 {
    let fx = Fixture::with_config(&repository_job_plan_config());
    // The hard job is planned and lands; the ordinary job is not planned and lands.
    let (planned, f) = run_job(&fx.athena(reader_scenario(json!("changes landing")).script()));
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    // Its edit differs from the first job's, so the producer makes a change of its own.
    let mut ordinary = reader_scenario(json!(null));
    ordinary.edits = vec![(
        "README.md".into(),
        Some("# Demo\n\nHello again, reader.\n".into()),
    )];
    let athena = fx.athena(ordinary.script());
    let (unplanned, f) = run_job(&athena);
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);

    let line_of = |text: &str, group: &str| -> String {
        text.lines()
            .find(|l| l.trim_start().starts_with(group))
            .unwrap_or_else(|| panic!("no {group} line: {text}"))
            .trim()
            .to_string()
    };
    let before = athena_cli::display::status(&athena.status().unwrap());
    assert!(
        before.contains("landings later cited as the cause of a failure, revert or fix job:"),
        "{before}"
    );
    assert_eq!(
        line_of(&before, "planned "),
        "planned   0 of 1 landings cited (0 failure · 0 revert · 0 fix) · 1 of 1 jobs landed (100%)"
    );
    assert_eq!(
        line_of(&before, "unplanned"),
        "unplanned 0 of 1 landings cited (0 failure · 0 revert · 0 fix) · 1 of 1 jobs landed (100%)"
    );

    // Later requests cite each landing: a fix for the planned one and a revert for the unplanned one. A request
    // that merely names a landing is not a cause.
    athena
        .ask(
            &format!("Fix the refusal rule that job {planned} landed."),
            &Lines::default(),
        )
        .unwrap();
    athena
        .ask(
            &format!("Revert job {unplanned}, it stopped a drain."),
            &Lines::default(),
        )
        .unwrap();
    athena
        .ask(
            &format!("Document how job {planned} works."),
            &Lines::default(),
        )
        .unwrap();
    let view = athena.status().unwrap();
    let text = athena_cli::display::status(&view);
    assert_eq!(
        line_of(&text, "planned "),
        "planned   1 of 1 landings cited (0 failure · 0 revert · 1 fix) · 1 of 1 jobs landed (100%)"
    );
    assert_eq!(
        line_of(&text, "unplanned"),
        "unplanned 1 of 1 landings cited (0 failure · 1 revert · 0 fix) · 1 of 1 jobs landed (100%)"
    );
    let json = serde_json::to_value(&view).unwrap();
    assert_eq!(json["keep_rule"]["later_harm"]["planned"]["fixes"], 1);
    assert_eq!(json["keep_rule"]["later_harm"]["unplanned"]["reverts"], 1);
    assert_eq!(json["keep_rule"]["later_harm"]["unplanned"]["failures"], 0);
}

// ---------------------------------------------------------------------------------------------------------
// Job 5a: job planning in `athena status`, and recovery of a job killed during planning.

/// Asks for and runs a job to the end.
fn run_job(athena: &athena_core::Athena) -> (athena_data::JobId, athena_core::run::Finished) {
    let proposal = athena.ask("tidy the readme", &Lines::default()).unwrap();
    let f = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    (proposal.job, f)
}

#[test]
fn status_shows_each_planned_jobs_plan_state_and_round_and_the_first_round_counts() {
    let fx = Fixture::with_config(&planned_config());
    let mut scenario = Scenario::default();
    // Job one: the first critique accepts. Job two: the first critique rejects.
    scenario.critiques = vec![
        critique_of("accept", &[], None),
        critique_of("reject", &[("blocker", "The plan is unsound")], None),
    ];
    let athena = fx.athena(scenario.script());
    let (accepted, f) = run_job(&athena);
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    let (stopped, f) = run_job(&athena);
    assert_eq!(f.state, OutcomeState::Failed);

    let text = athena_cli::display::status(&athena.status().unwrap());
    let line = |job: &athena_data::JobId| -> String {
        text.lines()
            .find(|l| l.contains(job.as_str()))
            .unwrap_or_else(|| panic!("no line for {job}: {text}"))
            .to_string()
    };
    assert!(line(&accepted).ends_with(" · plan accepted r1"), "{text}");
    assert!(line(&stopped).ends_with(" · plan stopped r1"), "{text}");
    assert!(
        text.contains("planning    first round: 1 accepted · 1 stopped\n"),
        "{text}"
    );
}

#[test]
fn a_second_round_accept_shows_as_plan_accepted_r2_and_is_not_a_first_round_accept() {
    let fx = Fixture::with_config(&planned_config());
    let mut scenario = Scenario::default();
    scenario.plans = vec![good_plan(), good_plan()];
    scenario.critiques = vec![
        critique_of("revise", &[("major", "Demonstration 2 repeats 1")], None),
        critique_of("accept", &[], None),
    ];
    let athena = fx.athena(scenario.script());
    let (job, f) = run_job(&athena);
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
    let text = athena_cli::display::status(&athena.status().unwrap());
    assert!(
        text.lines()
            .any(|l| l.contains(job.as_str()) && l.ends_with(" · plan accepted r2")),
        "{text}"
    );
    assert!(
        text.contains("planning    first round: 0 accepted · 0 stopped\n"),
        "{text}"
    );
}

#[test]
fn status_of_a_planned_job_lists_each_round_with_its_verdict_and_findings() {
    const FINDING: &str = "Demonstration 2 repeats demonstration 1 and shows nothing new";
    let fx = Fixture::with_config(&planned_config());
    let mut scenario = Scenario::default();
    scenario.plans = vec![good_plan(), good_plan()];
    scenario.critiques = vec![
        critique_of("revise", &[("major", FINDING)], None),
        critique_of("accept", &[("minor", "Name the file in step 1")], None),
    ];
    let athena = fx.athena(scenario.script());
    let (job, f) = run_job(&athena);
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);

    let text = athena_cli::display::trace(&athena.trace(job.as_str()).unwrap());
    assert!(text.contains("planning    plan accepted r2\n"), "{text}");
    let round = |n: u32| -> usize {
        text.find(&format!("plan round {n}  "))
            .unwrap_or_else(|| panic!("round {n} is listed: {text}"))
    };
    let (first, second) = (round(1), round(2));
    assert!(first < second);
    assert!(text[first..second].contains("revise"), "{text}");
    assert!(
        text[first..second].contains(&format!("Major: {FINDING}")),
        "{text}"
    );
    assert!(text[second..].contains("accept"), "{text}");
    assert!(
        text[second..].contains("Minor: Name the file in step 1"),
        "{text}"
    );
}

#[test]
fn status_json_of_a_planned_job_includes_every_rounds_plan_and_critique() {
    const FINDING: &str = "Demonstration 2 repeats demonstration 1 and shows nothing new";
    let fx = Fixture::with_config(&planned_config());
    let mut scenario = Scenario::default();
    scenario.plans = vec![good_plan(), good_plan()];
    scenario.critiques = vec![
        critique_of("revise", &[("major", FINDING)], None),
        critique_of("accept", &[], None),
    ];
    let athena = fx.athena(scenario.script());
    let (job, f) = run_job(&athena);
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);

    let json = serde_json::to_value(athena.trace(job.as_str()).unwrap()).unwrap();
    assert_eq!(json["plan"], "plan accepted r2");
    let plans = json["plans"].as_array().unwrap();
    assert_eq!(plans.len(), 2);
    assert_eq!(plans[0]["round"], 1);
    assert_eq!(plans[1]["round"], 2);
    assert_eq!(plans[1]["approach"], "Add a greeting line to README.md.");
    let critiques = json["critiques"].as_array().unwrap();
    assert_eq!(critiques.len(), 2);
    assert_eq!(critiques[0]["final_verdict"], "revise");
    assert_eq!(critiques[0]["findings"][0]["summary"], FINDING);
    assert_eq!(critiques[0]["findings"][0]["severity"], "major");
    assert_eq!(critiques[1]["final_verdict"], "accept");
    assert_eq!(critiques[1]["findings"].as_array().unwrap().len(), 0);
}

#[test]
fn an_unplanned_jobs_status_text_and_json_gain_nothing() {
    let fx = Fixture::with_config(&test_config());
    let athena = fx.athena(Scenario::default().script());
    let (job, f) = run_job(&athena);
    assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);

    let view = athena.status().unwrap();
    let text = athena_cli::display::status(&view);
    // The recent line is exactly what it was before planning existed: id, state padded to 11, cost, title.
    let expected = format!(
        "recent      {job} {:<11} ${:.2} · {}\n",
        "landed", view.recent[0].cost_usd, view.recent[0].title
    );
    assert!(text.contains(&expected), "{text}");
    // The throughput section names a "planning" stage for every job; it is not plan state, so it is set aside.
    let throughput = view.throughput.as_ref().expect("a landed job").text();
    assert!(text.contains(&throughput), "{text}");
    assert!(!text.replace(&throughput, "").contains("plan"), "{text}");
    let json = serde_json::to_value(&view).unwrap();
    assert!(json.get("planning").is_none());
    assert!(json["recent"][0].get("plan").is_none());

    let trace = athena.trace(job.as_str()).unwrap();
    let trace_text = athena_cli::display::trace(&trace);
    assert!(
        !trace_text.contains("plan round") && !trace_text.contains("planning"),
        "{trace_text}"
    );
    let json = serde_json::to_value(&trace).unwrap();
    for key in ["plan", "plans", "critiques"] {
        assert!(json.get(key).is_none(), "{key} appears in {json}");
    }
}

/// Runs a planned job to its end, then cuts the history back to just before the closing commit: the job is
/// dispatched with no closing record and no process, as after a machine restart in the middle of its run.
fn killed_after_dispatch(fx: &Fixture, athena: &athena_core::Athena) -> athena_data::JobId {
    use athena_core::store::{JobState, Records};
    let (job, _) = run_job(athena);
    for _ in 0..8 {
        let records = Records::load(&fx.git, &fx.head()).unwrap();
        if records.jobs[job.as_str()].state() == JobState::Dispatched {
            return job;
        }
        fx.git.run(["reset", "-q", "--hard", "HEAD~1"]).unwrap();
    }
    panic!("the job never was dispatched without an outcome");
}

/// A partial usage file such as a running invocation leaves in the state directory.
fn leave_partial_usage(fx: &Fixture, job: &athena_data::JobId, seq: u32, role: &str) {
    let dir = fx
        .state
        .path()
        .join("repo/jobs")
        .join(job.as_str())
        .join("usage");
    std::fs::create_dir_all(&dir).unwrap();
    let partial = json!({
        "seq": seq, "role": role, "model": "fake-model", "cap_usd": 1.0, "timeout_s": 600,
        "started_at": null, "input": 5, "output": 7, "cache_read": 0, "cache_write": 0, "commands": []
    });
    std::fs::write(dir.join(format!("{seq:02}.json")), partial.to_string()).unwrap();
}

/// The job's committed invocation records, as file name and outcome.
fn closed_invocations(fx: &Fixture, job: &athena_data::JobId) -> Vec<(String, String)> {
    let dir = job.dir();
    invocation_names(fx, &fx.head(), &dir)
        .into_iter()
        .map(|name| {
            let path = format!("{dir}/invocations/{name}");
            match read_record(fx, &path) {
                AnyRecord::Invocation(i) => (name, format!("{:?}", i.outcome)),
                other => panic!("unexpected {other:?}"),
            }
        })
        .collect()
}

#[test]
fn closing_a_job_killed_during_planning_records_its_planning_and_adds_no_producer() {
    let fx = Fixture::with_config(&planned_config());
    let athena = fx.athena(Scenario::default().script());
    let job = killed_after_dispatch(&fx, &athena);
    // The planner and critic were running when the process died.
    leave_partial_usage(&fx, &job, 2, "planner");
    leave_partial_usage(&fx, &job, 3, "critic");

    athena
        .close(job.as_str(), "died while planning", &Lines::default())
        .unwrap();
    let invocations = closed_invocations(&fx, &job);
    let names: Vec<&str> = invocations.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        ["01-reader.json", "02-planner.json", "03-critic.json"],
        "{invocations:?}"
    );
    assert!(
        invocations.iter().all(|(_, outcome)| outcome != "Missing"),
        "{invocations:?}"
    );
    assert!(athena.check_history().unwrap().violations.is_empty());
}

#[test]
fn closing_a_job_with_no_trace_of_any_invocation_still_records_a_missing_producer() {
    let fx = Fixture::with_config(&planned_config());
    let athena = fx.athena(Scenario::default().script());
    let job = killed_after_dispatch(&fx, &athena);

    athena
        .close(job.as_str(), "died at once", &Lines::default())
        .unwrap();
    let invocations = closed_invocations(&fx, &job);
    let names: Vec<&str> = invocations.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(
        names,
        ["01-reader.json", "02-producer.json"],
        "{invocations:?}"
    );
    assert_eq!(invocations[1].1, "Missing", "{invocations:?}");
}
