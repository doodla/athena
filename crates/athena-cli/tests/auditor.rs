//! The auditor against the fake agent over a real (fake-driven) job: what it is given, what it records, and
//! what the owner sees. It is never given the job's acceptance criteria.

// Scenarios are set up field by field, so each test shows only what differs from the default.
#![allow(clippy::field_reassign_with_default)]

mod common;

use athena_core::Athena;
use athena_core::auditor::Audited;
use athena_core::run::GoResult;
use athena_core::store::Records;
use athena_data::records::{AcceptedBy, OutcomeState, Role};
use athena_data::roles;
use common::*;
use serde_json::{Value, json};

const BRIEF: &str = "# The brief\n\nOWNER-GOAL-CANARY: keep it simple.\n";

fn records(fx: &Fixture) -> Records {
    Records::load(&fx.git, &fx.head()).unwrap()
}

/// A repository with a BRIEF.md, in which one job has landed.
fn landed(scenario: Scenario) -> (Fixture, Athena) {
    let fx = Fixture::new();
    write(fx.root(), &[("BRIEF.md", BRIEF)]);
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run([
            "commit",
            "-q",
            "-m",
            "Add the brief\n\nAthena-Job: bootstrap",
        ])
        .unwrap();
    let athena = fx.athena(scenario.script());
    let ui = Lines::default();
    let proposal = athena.ask("add a greeting to the readme", &ui).unwrap();
    let GoResult::Finished(done) = athena
        .go(proposal.job.as_str(), AcceptedBy::User, &ui)
        .unwrap()
    else {
        panic!("expected the job to finish");
    };
    assert_eq!(done.state, OutcomeState::Landed, "{}", done.reason);
    (fx, athena)
}

fn finding(evidence: &str) -> Value {
    json!({
        "summary": "The outcome serves the goal.",
        "findings": [{ "text": "The greeting is there.", "evidence": evidence }]
    })
}

fn audit_with(report: Value) -> (Fixture, Athena, anyhow::Result<Option<Audited>>) {
    let mut scenario = Scenario::default();
    scenario.auditor = report;
    let (fx, athena) = landed(scenario);
    let result = athena.audit("latest");
    (fx, athena, result)
}

#[test]
fn an_audit_records_a_typed_finding_judged_against_the_brief_not_the_criteria() {
    let mut scenario = Scenario::default();
    scenario.auditor = finding("README.md line 3: Hello, reader.");
    let seen = scenario.seen_by_auditor.clone();
    let criteria = criteria_texts(&scenario.reading);
    assert_eq!(criteria.len(), 2);
    let (fx, athena) = landed(scenario);
    let invocations_before = records(&fx).auditor_runs.len();
    assert_eq!(invocations_before, 0);

    let audited = athena.audit("latest").unwrap().expect("an audit");

    // The auditor is given BRIEF.md, its fixed prompt and the real outcome, and no criterion of the job.
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 3, "one auditor run: prompt, brief, workspace");
    assert!(seen[0].contains(roles::AUDITOR_PROMPT));
    assert!(
        seen[1].contains("OWNER-GOAL-CANARY"),
        "the brief has BRIEF.md"
    );
    assert!(
        seen[2].contains("Hello, reader."),
        "the workspace holds the file the job really changed"
    );
    for criterion in &criteria {
        for (what, text) in [
            ("prompt", &seen[0]),
            ("brief", &seen[1]),
            ("workspace", &seen[2]),
        ] {
            assert!(
                !text.contains(criterion),
                "{what} holds the criterion {criterion:?}"
            );
        }
    }
    assert!(!seen[1].contains("Acceptance criteria"));
    assert!(!seen[2].contains("PRIVATE-CANARY"));

    // One typed finding is recorded, with its evidence, beside the auditor's invocation and role.
    let after = records(&fx);
    assert_eq!(after.audits, vec![audited.audit.clone()]);
    assert_eq!(after.audits[0].findings.len(), 1);
    assert_eq!(
        after.audits[0].findings[0].evidence,
        "README.md line 3: Hello, reader."
    );
    assert_eq!(after.auditor_runs.len(), 1);
    assert_eq!(after.auditor_runs[0].role, Role::Auditor);
    assert!(
        after
            .roles
            .iter()
            .any(|r| r.path() == "records/roles/auditor-1.json")
    );
    let subject = fx.git.run(["log", "--format=%s", "-n", "1"]).unwrap();
    assert!(subject.starts_with("Audit: job "), "{subject}");
    assert!(fx.git.run(["status", "--porcelain"]).unwrap().is_empty());

    // The owner sees the finding and the recorded invocation in the CLI output and in `athena status`.
    let shown = athena_cli::display::audit(&audited);
    assert!(shown.contains("The greeting is there."), "{shown}");
    assert!(shown.contains("evidence: README.md line 3"), "{shown}");
    assert!(
        shown.contains("auditor invocation recorded: records/jobs/"),
        "{shown}"
    );
    assert!(shown.contains("01-auditor.json"), "{shown}");
    let status = athena.status().unwrap();
    let view = status.audit.expect("the latest audit");
    assert_eq!(view.id, audited.audit.id.to_string());
    assert!(athena_cli::display::status(&athena.status().unwrap()).contains("audit "));
}

#[test]
fn a_finding_with_no_evidence_is_rejected_and_the_paid_invocation_is_kept() {
    for evidence in ["", "   "] {
        let (fx, _, result) = audit_with(finding(evidence));
        assert!(result.is_err(), "{evidence:?}");
        let after = records(&fx);
        assert!(after.audits.is_empty(), "{evidence:?}");
        assert_eq!(after.auditor_runs.len(), 1, "the invocation is kept");
    }
}

#[test]
fn a_job_with_no_outcome_is_not_audited_and_calls_no_model() {
    let fx = Fixture::new();
    let scenario = Scenario::default();
    let invocations = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    athena.ask("add a greeting", &Lines::default()).unwrap();
    let before = invocations.lock().unwrap().len();
    let head = fx.head();
    assert!(athena.audit("latest").is_err());
    assert_eq!(invocations.lock().unwrap().len(), before);
    assert_eq!(fx.head(), head);
}

/// Makes the landed job's candidate commit absent, as in a fresh clone, and returns the job and that commit.
fn drop_candidate(fx: &Fixture) -> (String, String) {
    let after = records(fx);
    let (job, record) = after.jobs.iter().next().expect("one job");
    let candidate = record
        .candidate
        .as_ref()
        .and_then(|c| c.candidate_commit.clone())
        .expect("a candidate commit");
    assert!(fx.git.commit_exists(&candidate).unwrap());
    let object = fx
        .root()
        .join(".git/objects")
        .join(&candidate[..2])
        .join(&candidate[2..]);
    std::fs::remove_file(&object).expect("the candidate is a loose object");
    assert!(!fx.git.commit_exists(&candidate).unwrap());
    (job.to_string(), candidate)
}

#[test]
fn a_missing_candidate_commit_is_read_from_the_squash_commit_with_the_job_trailer() {
    let mut scenario = Scenario::default();
    scenario.auditor = finding("README.md line 3: Hello, reader.");
    let seen = scenario.seen_by_auditor.clone();
    let (fx, athena) = landed(scenario);
    drop_candidate(&fx);

    let audited = athena.audit("latest").unwrap().expect("an audit");

    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen.len(), 3, "one auditor run");
    assert!(
        seen[2].contains("Hello, reader."),
        "the workspace holds the file as the squash commit has it: {}",
        seen[2]
    );
    let after = records(&fx);
    assert_eq!(after.audits, vec![audited.audit]);
    assert_eq!(after.auditor_runs.len(), 1);
}

#[test]
fn with_no_commit_to_read_the_audit_refuses_before_paying() {
    let mut scenario = Scenario::default();
    scenario.auditor = finding("README.md line 3: Hello, reader.");
    let invocations = scenario.invocations.clone();
    let (fx, athena) = landed(scenario);
    let (job, candidate) = drop_candidate(&fx);
    // Rewrite history as one commit with the same tree, so no commit carries the job's trailer.
    let tree = fx.git.run(["rev-parse", "HEAD^{tree}"]).unwrap();
    let flat = fx
        .git
        .run(["commit-tree", tree.trim(), "-m", "Snapshot"])
        .unwrap();
    fx.git.run(["reset", "-q", "--hard", flat.trim()]).unwrap();
    let before = invocations.lock().unwrap().len();
    let head = fx.head();

    let error = athena.audit(&job).unwrap_err().to_string();

    assert!(error.contains(&job), "{error}");
    assert!(error.contains(&candidate), "{error}");
    assert_eq!(invocations.lock().unwrap().len(), before, "no model call");
    assert_eq!(fx.head(), head, "nothing was committed");
    let after = records(&fx);
    assert!(after.audits.is_empty());
    assert!(after.auditor_runs.is_empty());
}

/// Logs a mistake of `class` by the command a user would run.
fn log_mistake(athena: &Athena, class: &str, what: &str) {
    athena
        .intervene(
            &athena_core::manage::InterveneArgs {
                what: what.into(),
                why: "It went wrong.".into(),
                class: class.into(),
                category: "mistake".into(),
                job: None,
                with_staged: false,
            },
            &common::Lines::default(),
        )
        .unwrap();
}

#[test]
fn the_audit_output_reports_mistakes_per_class_and_per_cause() {
    let mut scenario = Scenario::default();
    scenario.auditor = finding("README.md line 3: Hello, reader.");
    let (_fx, athena) = landed(scenario);
    log_mistake(&athena, "docs", "Skipped the docs test");
    log_mistake(&athena, "docs", "Skipped it again");
    log_mistake(&athena, "code", "Wrong file");
    let audited = athena.audit("latest").unwrap().expect("an audit");

    let shown = athena_cli::display::audit_report(&athena.mistakes().unwrap(), Some(&audited));
    assert!(
        shown.contains("mistakes 3: by class: code 1, docs 2"),
        "{shown}"
    );
    assert!(
        shown.contains("mistakes by cause: no cause recorded 3"),
        "{shown}"
    );
    assert!(
        shown.contains("The greeting is there."),
        "the audit still prints: {shown}"
    );
}

#[test]
fn the_audit_output_reports_mistakes_when_no_audit_is_produced() {
    let (_fx, mut athena) = landed(Scenario::default());
    log_mistake(&athena, "docs", "Skipped the docs test");
    log_mistake(&athena, "code", "Wrong file");
    athena.nested = Some("20261014-120000-outr".into());
    let audited = athena.audit("latest").unwrap();
    assert!(audited.is_none(), "a nested run produces no audit");

    let shown = athena_cli::display::audit_report(&athena.mistakes().unwrap(), audited.as_ref());
    assert!(
        shown.contains("mistakes 2: by class: code 1, docs 1"),
        "{shown}"
    );
    assert!(shown.contains("mistakes by cause:"), "{shown}");
    assert!(shown.contains("no job audit produced"), "{shown}");
}

#[test]
fn a_nested_run_audits_nothing() {
    let (fx, mut athena) = landed(Scenario::default());
    athena.nested = Some("20261014-120000-outr".into());
    let head = fx.head();
    assert_eq!(athena.audit("latest").unwrap(), None);
    assert_eq!(fx.head(), head);
}
