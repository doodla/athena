//! The steward against the fake agent: when it starts, what it is given to read, what it queues, and what the
//! ladder lets it change.

// Scenarios are set up field by field, so each test shows only what differs from the default.
#![allow(clippy::field_reassign_with_default)]

mod common;

use athena_agent::AgentInvocation;
use athena_core::Athena;
use athena_core::run::GoResult;
use athena_core::steward::{self, GitView, RECORDS_SOURCE, RequestQueue, View};
use athena_core::store::Records;
use athena_data::records::{AcceptedBy, OutcomeState, RefKind, Role, WorkItemKind};
use athena_data::roles::{self, Ladder};
use common::*;
use serde_json::{Value, json};
use std::path::Path;
use std::sync::Arc;

/// Runs one job to its end. Returns its id and how it ended.
fn go_any(athena: &Athena, ui: &Lines) -> (String, OutcomeState) {
    let proposal = athena.ask("change the readme", ui).unwrap();
    match athena
        .go(proposal.job.as_str(), AcceptedBy::User, ui)
        .unwrap()
    {
        GoResult::Finished(f) => (proposal.job.as_str().to_string(), f.state),
        GoResult::Reread(_) => panic!("expected the job to finish"),
    }
}

/// Runs one job whose change deletes the README, so a check fails and the job is rejected. Returns its id.
fn go_one(athena: &Athena, ui: &Lines) -> String {
    let (job, state) = go_any(athena, ui);
    assert_eq!(state, OutcomeState::Rejected, "{job}");
    job
}

fn records(fx: &Fixture) -> Records {
    Records::load(&fx.git, &fx.head()).unwrap()
}

fn steward_lines(ui: &Lines) -> Vec<String> {
    ui.0.lock()
        .unwrap()
        .iter()
        .filter(|l| l.starts_with("steward"))
        .cloned()
        .collect()
}

/// A work item in the steward's output shape.
fn item(
    kind: &str,
    key: &str,
    title: &str,
    rung: &str,
    source: (&str, &str),
    evidence: &[(&str, &str)],
    request: Option<&str>,
) -> Value {
    json!({
        "kind": kind,
        "key": key,
        "title": title,
        "rung": rung,
        "source": { "kind": source.0, "value": source.1 },
        "evidence": evidence.iter().map(|(k, v)| json!({ "kind": k, "value": v })).collect::<Vec<_>>(),
        "request": request,
    })
}

/// What the fake steward does with its brief and workspace: for each source the brief lists, it "reads" the
/// path when the workspace has it, and says so in the commands it reports.
fn reads(spec: &AgentInvocation) -> Vec<String> {
    let mut done = Vec::new();
    let mut in_sources = false;
    for line in spec.user_prompt.lines() {
        if line.starts_with("## ") {
            in_sources = line == "## Sources";
            continue;
        }
        let Some(rest) = line.strip_prefix("- ").filter(|_| in_sources) else {
            continue;
        };
        let Some((name, paths)) = rest.split_once(": ") else {
            continue;
        };
        for path in paths.split(", ") {
            if spec.workdir.join(path).exists() {
                done.push(format!("read {name} {path}"));
            }
        }
    }
    if !spec.workdir.join("PRIVATE.md").exists() {
        done.push("check private-files-hidden".into());
    }
    if spec.role == Role::Steward && spec.system_prompt.contains(roles::STEWARD_PROMPT) {
        done.push("check system-prompt-is-the-stored-prompt".into());
    }
    done
}

/// A scenario for the steward: it reports `items`, and reads what its brief offers.
fn steward_scenario(items: Vec<Value>) -> Scenario {
    let mut scenario = Scenario::default();
    scenario.steward = json!({ "items": items });
    scenario.steward_reads = Some(Arc::new(reads));
    scenario
}

fn steward_calls(invocations: &Arc<std::sync::Mutex<Vec<Role>>>) -> usize {
    invocations
        .lock()
        .unwrap()
        .iter()
        .filter(|r| **r == Role::Steward)
        .count()
}

fn run_steward(athena: &Athena) -> athena_core::steward::Report {
    let role = roles::steward(athena_core::now());
    let view = GitView { git: &athena.git };
    steward::run(athena, &view, &RequestQueue, &role, athena_core::now()).unwrap()
}

/// Two rejected jobs of one class, then an Athena whose steward reports `items(jobs)`.
fn after_two_failures(
    fx: &Fixture,
    items: impl Fn(&[String]) -> Vec<Value>,
) -> (Athena, Vec<String>, Arc<std::sync::Mutex<Vec<Role>>>) {
    let mut failing = Scenario::default();
    failing.edits = vec![("README.md".into(), None)];
    let athena = fx.athena(failing.script());
    let ui = Lines::default();
    let jobs = vec![go_one(&athena, &ui), go_one(&athena, &ui)];
    let scenario = steward_scenario(items(&jobs));
    let invocations = scenario.invocations.clone();
    (fx.athena(scenario.script()), jobs, invocations)
}

fn repeated_failure_items(jobs: &[String]) -> Vec<Value> {
    vec![
        item(
            "repeated-failure",
            "repeated-failure:docs",
            "Two docs jobs were rejected for deleting the README",
            "defaults-conventions",
            ("path", "records/jobs"),
            &[("job", &jobs[0]), ("job", &jobs[1])],
            Some("Jobs of the class docs keep deleting the README. Add the check that stops it."),
        ),
        item(
            "proposal",
            "proposal:brief-readme",
            "BRIEF.md should say the README is never deleted",
            "brief-requirements",
            ("path", "records/jobs"),
            &[("job", &jobs[0])],
            None,
        ),
    ]
}

#[test]
fn the_fifth_finished_task_starts_the_steward_and_the_fourth_does_not() {
    // Every job here is rejected (a rejection counts as finished, as a landing does).
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.edits = vec![("README.md".into(), None)];
    let invocations = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let ui = Lines::default();
    // Three tasks have already finished; this one is the fourth.
    for _ in 0..3 {
        assert!(!steward::tick(&athena.state, 5).unwrap());
    }
    go_one(&athena, &ui);
    assert_eq!(steward::finished_since_run(&athena.state), 4);
    assert!(steward_lines(&ui).is_empty(), "the fourth starts nothing");
    steward::wait_for_background();
    assert_eq!(steward_calls(&invocations), 0, "and no model was called");

    // The fifth starts it, in the background: go has returned a finished job by the time it is counted.
    let head = fx.head();
    go_one(&athena, &ui);
    assert_eq!(steward_lines(&ui).len(), 1, "{:?}", steward_lines(&ui));
    assert_eq!(steward::finished_since_run(&athena.state), 0);
    steward::wait_for_background();
    assert_eq!(steward_calls(&invocations), 1, "the steward called a model");
    assert!(
        records(&fx).roles.is_empty() && records(&fx).work_items.is_empty(),
        "and found nothing to queue"
    );
    assert!(
        !athena.state.root().join("steward-error.txt").exists(),
        "without an error"
    );
    // The job's own commits moved the head; the steward added none.
    assert_ne!(head, fx.head());
    let log = fx.git.run(["log", "--format=%s", "-n", "3"]).unwrap();
    assert!(!log.contains("Steward"), "{log}");
}

#[test]
fn scripted_output_with_a_real_gap_commits_the_work_items_and_a_queued_request() {
    // Criterion (a).
    let fx = Fixture::new();
    let (athena, jobs, invocations) = after_two_failures(&fx, repeated_failure_items);
    let before = fx.head();
    let report = run_steward(&athena);
    assert_eq!(steward_calls(&invocations), 1);
    assert_eq!(
        report.queued,
        vec!["repeated-failure:docs", "proposal:brief-readme"]
    );
    assert_eq!(report.applied.len(), 1, "only the lower rung is applied");
    assert!(report.rejected.is_empty(), "{:?}", report.rejected);

    assert_ne!(fx.head(), before, "one records commit");
    let subject = fx.git.run(["log", "--format=%s", "-n", "1"]).unwrap();
    assert!(subject.starts_with("Steward: 2 work item(s), 1 request(s)"));
    let after = records(&fx);
    assert_eq!(after.roles.len(), 1, "the role definition is recorded");
    assert_eq!(after.roles[0].name, "steward");
    assert_eq!(after.roles[0].prompt, roles::STEWARD_PROMPT);
    assert_eq!(after.work_items.len(), 2);

    let failure = after
        .work_items
        .iter()
        .find(|w| w.kind == WorkItemKind::RepeatedFailure)
        .unwrap();
    assert_eq!(failure.rung, Ladder::DefaultsConventions);
    assert_eq!(failure.role, "steward");
    let cited: Vec<&str> = failure
        .evidence
        .iter()
        .filter(|e| e.kind == RefKind::Job)
        .map(|e| e.value.as_str())
        .collect();
    assert_eq!(cited.len(), 3, "the two failed jobs and the queued request");
    assert!(cited.contains(&jobs[0].as_str()) && cited.contains(&jobs[1].as_str()));
    let queued = &report.applied[0];
    assert!(cited.contains(&queued.as_str()));
    let job = after.jobs.get(queued.as_str()).expect("the request exists");
    let request = job.request.as_ref().unwrap();
    assert!(request.text.contains("keep deleting the README"));
    assert_eq!(request.channel, "steward");

    let proposal = after
        .work_items
        .iter()
        .find(|w| w.kind == WorkItemKind::Proposal)
        .unwrap();
    assert_eq!(proposal.rung, Ladder::BriefRequirements);
    assert!(
        proposal.evidence.iter().all(|e| e.value != queued.as_str()),
        "a rung above the limit queues no request"
    );
    assert_eq!(
        after
            .jobs
            .values()
            .filter(|j| j.request.as_ref().is_some_and(|r| r.channel == "steward"))
            .count(),
        1
    );

    // The invocation is recorded like any other role's: role, usage, and cost; and is no job.
    assert_eq!(after.steward_runs.len(), 1);
    let run = &after.steward_runs[0];
    assert_eq!(run.role, Role::Steward);
    assert_eq!(run.tokens.input.value(), Some(10.0));
    assert_eq!(run.tokens.output.value(), Some(20.0));
    assert_eq!(run.cost.best_lower(), 0.01);
    assert!(
        !after.jobs.contains_key(run.job.as_str()),
        "a steward run is not listed as a job"
    );
    assert_eq!(run.context.tools, ["Read", "Grep", "Glob", "Bash"]);
    assert!(
        athena
            .state
            .root()
            .join("steward-pending")
            .read_dir()
            .map(|mut d| d.next().is_none())
            .unwrap_or(true),
        "the committed invocation is no longer waiting"
    );
}

#[test]
fn empty_output_makes_no_commit_and_its_invocation_waits_for_the_next_one() {
    // Criterion (b).
    let fx = Fixture::new();
    let (quiet, _, quiet_calls) = after_two_failures(&fx, |_| Vec::new());
    let before = fx.head();
    let report = run_steward(&quiet);
    assert_eq!(report, steward::Report::default());
    assert_eq!(steward_calls(&quiet_calls), 1);
    assert_eq!(fx.head(), before, "no commit");
    let waiting = quiet.state.root().join("steward-pending");
    assert_eq!(std::fs::read_dir(&waiting).unwrap().count(), 1);

    // The next run that queues something carries both invocations.
    let scenario = steward_scenario(vec![item(
        "blind-spot",
        "x",
        "gh could not be run",
        "implementation",
        ("path", "github"),
        &[],
        None,
    )]);
    let athena = fx.athena(scenario.script());
    let report = run_steward(&athena);
    assert_eq!(report.queued, vec!["blind-spot:github"]);
    assert_eq!(records(&fx).steward_runs.len(), 2);
    assert_eq!(std::fs::read_dir(&waiting).unwrap().count(), 0);
}

#[test]
fn a_second_run_with_the_same_keys_queues_nothing_again() {
    // Criterion (c).
    let fx = Fixture::new();
    let (athena, _, invocations) = after_two_failures(&fx, repeated_failure_items);
    let first = run_steward(&athena);
    assert_eq!(first.queued.len(), 2);
    let head = fx.head();
    let second = run_steward(&athena);
    assert_eq!(steward_calls(&invocations), 2, "the model is asked again");
    assert_eq!(second, steward::Report::default(), "but nothing is queued");
    assert_eq!(fx.head(), head, "and nothing is committed");
    let after = records(&fx);
    assert_eq!(after.work_items.len(), 2);
    assert_eq!(
        after
            .jobs
            .values()
            .filter(|j| j.request.as_ref().is_some_and(|r| r.channel == "steward"))
            .count(),
        1,
        "the request is queued once"
    );
}

#[test]
fn a_nested_run_makes_no_model_call() {
    // Criterion (d): nothing counts toward the cadence, nothing starts the steward, and running it does
    // nothing. The real driver also refuses under the nested marker (`athena_agent::refuse_nested`).
    let fx = Fixture::new();
    let scenario = steward_scenario(vec![item(
        "blind-spot",
        "x",
        "gh could not be run",
        "implementation",
        ("path", "github"),
        &[],
        None,
    )]);
    let invocations = scenario.invocations.clone();
    let mut athena = fx.athena(scenario.script());
    athena.nested = Some("20261014-120000-outr".into());
    let ui = Lines::default();
    for _ in 0..5 {
        // A nested run lands its job (its checks are the fakes'); what matters is that none is counted.
        go_any(&athena, &ui);
    }
    assert_eq!(steward::finished_since_run(&athena.state), 0, "not counted");
    assert!(steward_lines(&ui).is_empty());
    steward::wait_for_background();
    let head = fx.head();
    assert_eq!(run_steward(&athena), steward::Report::default());
    assert_eq!(steward_calls(&invocations), 0, "no model call");
    assert_eq!(fx.head(), head);
    assert!(
        athena_agent::refuse_nested(Some("20261014-120000-outr".into())).is_err(),
        "and the real driver refuses"
    );
}

#[test]
fn the_fake_steward_reads_every_named_source_in_a_workspace_without_private_files() {
    // Criterion (e).
    let fx = Fixture::new();
    let (athena, _, _) = after_two_failures(&fx, repeated_failure_items);
    run_steward(&athena);
    let run = &records(&fx).steward_runs[0];
    for read in [
        "read jobs records/jobs",
        "read events records/interventions",
        "read events records/restarts",
        "read introspections records/introspections",
        "read class-registry records/failure-classes",
        "read decisions records/decision-states",
        "read git-log GIT-LOG.txt",
        "read code .",
        "check private-files-hidden",
        "check system-prompt-is-the-stored-prompt",
    ] {
        assert!(
            run.commands_run.iter().any(|c| c == read),
            "{read} not in {:?}",
            run.commands_run
        );
    }
}

/// A view that reads like the real one but reports a source it could not write.
struct LosesTheGitLog<'a>(GitView<'a>);

impl View for LosesTheGitLog<'_> {
    fn records(&self) -> anyhow::Result<Records> {
        self.0.records()
    }

    fn workspace(&self, dir: &Path) -> anyhow::Result<Vec<&'static str>> {
        let mut lost = self.0.workspace(dir)?;
        std::fs::remove_file(dir.join(steward::GIT_LOG_FILE))?;
        lost.push("git-log");
        Ok(lost)
    }
}

#[test]
fn a_source_athena_cannot_write_becomes_a_blind_spot_the_model_is_told_about() {
    let fx = Fixture::new();
    let (athena, _, _) = after_two_failures(&fx, |_| Vec::new());
    let role = roles::steward(athena_core::now());
    let view = LosesTheGitLog(GitView { git: &athena.git });
    let report = steward::run(&athena, &view, &RequestQueue, &role, athena_core::now()).unwrap();
    assert_eq!(report.queued, vec!["blind-spot:git-log"]);
    let after = records(&fx);
    let spot = &after.work_items[0];
    assert_eq!(spot.kind, WorkItemKind::BlindSpot);
    assert_eq!(spot.source.value, "git-log");
    assert!(
        !after.steward_runs[0]
            .commands_run
            .iter()
            .any(|c| c.contains("GIT-LOG")),
        "the brief did not offer the lost source"
    );
}

/// A view whose records cannot be read.
struct Unreadable;

impl View for Unreadable {
    fn records(&self) -> anyhow::Result<Records> {
        anyhow::bail!("not readable")
    }

    fn workspace(&self, _: &Path) -> anyhow::Result<Vec<&'static str>> {
        panic!("with no records there is nothing to give a model");
    }
}

#[test]
fn an_unreadable_source_becomes_a_blind_spot_work_item_and_calls_no_model() {
    let fx = Fixture::new();
    let scenario = Scenario::default();
    let invocations = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let role = roles::steward(athena_core::now());
    let report = steward::run(
        &athena,
        &Unreadable,
        &RequestQueue,
        &role,
        athena_core::now(),
    )
    .unwrap();
    assert_eq!(report.queued, vec!["blind-spot:records/".to_string()]);
    assert_eq!(steward_calls(&invocations), 0);
    let after = records(&fx);
    assert_eq!(after.work_items.len(), 1);
    let item = &after.work_items[0];
    assert_eq!(item.kind, WorkItemKind::BlindSpot);
    assert_eq!(item.source.kind, RefKind::Path);
    assert_eq!(item.source.value, RECORDS_SOURCE);
    assert_eq!(item.role, "steward");
}

#[test]
fn github_being_unavailable_is_one_blind_spot_whatever_the_error_says() {
    let fx = Fixture::new();
    let report_for = |title: &str| {
        let scenario = steward_scenario(vec![item(
            "blind-spot",
            title,
            title,
            "owner-goals",
            ("path", "github"),
            &[],
            Some("ignored"),
        )]);
        let athena = fx.athena(scenario.script());
        run_steward(&athena)
    };
    let first = report_for("gh: HTTP 401 at 12:00:01");
    assert_eq!(first.queued, vec!["blind-spot:github"]);
    assert_eq!(first.applied.len(), 1, "a request to make it readable");
    let head = fx.head();
    let second = report_for("gh: HTTP 502 Bad Gateway at 12:07:44");
    assert_eq!(second, steward::Report::default(), "{second:?}");
    assert_eq!(fx.head(), head);
    let after = records(&fx);
    assert_eq!(after.work_items.len(), 1);
    assert_eq!(after.work_items[0].key, "blind-spot:github");
    assert!(!after.work_items[0].key.contains("401"));
}

#[test]
fn items_that_fail_validation_are_reported_and_nothing_is_queued_for_them() {
    let fx = Fixture::new();
    let (athena, jobs, _) = after_two_failures(&fx, |jobs| {
        vec![
            item(
                "repeated-failure",
                "bad:job",
                "Cites a job that does not exist",
                "implementation",
                ("path", "records/jobs"),
                &[("job", "20200101-000000-zzzz")],
                Some("Do it."),
            ),
            item(
                "repeated-failure",
                "bad:path",
                "Cites a path outside the workspace",
                "implementation",
                ("path", "/etc/passwd"),
                &[("job", &jobs[0])],
                Some("Do it."),
            ),
            item(
                "repeated-failure",
                "bad:request",
                "Has no request on a rung it may change",
                "implementation",
                ("path", "records/jobs"),
                &[("job", &jobs[0])],
                None,
            ),
        ]
    });
    let _ = jobs;
    let head = fx.head();
    let report = run_steward(&athena);
    assert!(report.queued.is_empty());
    assert_eq!(report.rejected.len(), 3, "{:?}", report.rejected);
    assert_eq!(fx.head(), head);
}
