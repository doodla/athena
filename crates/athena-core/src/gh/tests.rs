use super::client::TEST_NAME_LIMIT;
use super::*;
use crate::reconcile::MergedPr;
use athena_data::RecordId;
use athena_data::history::{
    self, HistoryRecord, Landing, LandingCommit, LandingIntent, LandingKind,
};
use athena_data::records::HostOperation;
use athena_data::records::{HostError, MessageClass};
use athena_git::records_branch::{Change, RecordsWriter};
use serde_json::Value;

#[test]
fn parse_merged_pr_keeps_only_the_pull_request_that_merged_the_commit() {
    let commit = "a".repeat(40);
    let pr = |number: u64, merged: bool, sha: &str| {
        serde_json::json!({
            "number": number,
            "merged_at": if merged { Value::from("2026-10-24T00:00:00Z") } else { Value::Null },
            "merge_commit_sha": sha,
            "head": {"sha": "b".repeat(40)},
        })
    };
    // None: an empty reply, an unmerged pull request, or one merged as another commit.
    assert_eq!(
        parse_merged_pr(&serde_json::json!([]), &commit).unwrap(),
        None
    );
    let reply = serde_json::json!([pr(1, false, &commit), pr(2, true, &"c".repeat(40))]);
    assert_eq!(parse_merged_pr(&reply, &commit).unwrap(), None);
    // One.
    let reply = serde_json::json!([pr(1, false, &commit), pr(3, true, &commit)]);
    assert_eq!(
        parse_merged_pr(&reply, &commit).unwrap(),
        Some(MergedPr {
            number: 3,
            head_sha: "b".repeat(40)
        })
    );
    // Two: a guess would be needed, so it is an error.
    let reply = serde_json::json!([pr(3, true, &commit), pr(4, true, &commit)]);
    let error = parse_merged_pr(&reply, &commit).unwrap_err();
    assert!(format!("{error:#}").contains("ambiguous"), "{error:#}");
}
use serde_json::json;
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

/// The repository reply of a host that allows every merge method.
const ALL_METHODS: &str = r#"{"default_branch": "trunk", "allow_merge_commit": true,
        "allow_squash_merge": true, "allow_rebase_merge": true}"#;
/// The check-runs reply of a commit whose one required check passed.
const REQUIRED_GREEN: &str = r#"{"check_runs": [{"id": 1, "name": "fmt, clippy, tests",
        "status": "completed", "conclusion": "success"}]}"#;
/// The repository reply of a host that allows only squash merges.
const SQUASH_ONLY: &str = r#"{"default_branch": "trunk", "allow_merge_commit": false,
        "allow_squash_merge": true, "allow_rebase_merge": false}"#;

struct FakeRunner {
    calls: RefCell<Vec<Vec<String>>>,
    /// What `GET repos/example/project` answers.
    repo: &'static str,
}
impl FakeRunner {
    fn new(repo: &'static str) -> Self {
        FakeRunner {
            calls: RefCell::new(vec![]),
            repo,
        }
    }
}
fn text() -> CommitText {
    CommitText {
        title: "Change a".into(),
        body: "Body.\n\nAthena-Job: bootstrap\n".into(),
    }
}
impl Runner for FakeRunner {
    fn gh(&self, args: &[String]) -> Result<String> {
        self.calls.borrow_mut().push(args.to_vec());
        let path = args.iter().find(|a| a.starts_with("repos/")).unwrap();
        Ok(if path == "repos/example/project" {
            self.repo.to_string()
        } else if path.ends_with("/pulls") {
            r#"{"number": 7}"#.to_string()
        } else if path.contains("/pulls/") && !path.ends_with("/merge") {
            r#"{"merged": true, "merge_commit_sha": "m1"}"#.to_string()
        } else if path.contains("check-runs") {
            REQUIRED_GREEN.to_string()
        } else if path.ends_with("/status") {
            r#"{"state": "success", "statuses": []}"#.to_string()
        } else {
            "{}".to_string()
        })
    }
}

#[test]
fn the_client_runs_only_gh_api_with_rest_paths() {
    let runner = FakeRunner::new(ALL_METHODS);
    let client = GhClient::new(&runner, "example/project".into());
    let pr = NewPr {
        title: "t",
        body: "b",
        head: "athena/j",
        base: "main",
    };
    assert_eq!(client.open_pr(&pr).unwrap(), 7);
    assert_eq!(client.checks("abc").unwrap(), Checks::Green);
    client.merge(7, "abc", &text()).unwrap();
    let link = PrLink {
        pr_number: 7,
        host: HOST.into(),
        repository: "example/project".into(),
        head: "abc".into(),
        job: "j".into(),
        state: LinkState::Open,
        merged_commit: None,
    };
    assert_eq!(client.merged_commit(&link).unwrap(), Some("m1".into()));

    let calls = runner.calls.borrow();
    // open, checks (two reads), the repository's methods, the merge, the merged lookup.
    assert_eq!(calls.len(), 6);
    for args in calls.iter() {
        assert_eq!(args[0], "api", "{args:?}");
        assert_eq!(args[1], "--method");
        assert!(args[3].starts_with("repos/example/project"), "{args:?}");
        assert!(!args.iter().any(|a| a == "pr" || a == "--auto"), "{args:?}");
    }
    assert_eq!(calls[0][3], "repos/example/project/pulls");
    assert_eq!(calls[0][2], "POST");
    assert_eq!(calls[3][3], "repos/example/project");
    assert_eq!(calls[4][2], "PUT");
    assert_eq!(calls[4][3], "repos/example/project/pulls/7/merge");
    // A merge commit is allowed, so it is requested, with no message of ours.
    assert!(calls[4].contains(&"merge_method=merge".to_string()));
    assert!(!calls[4].iter().any(|a| a.starts_with("commit_")));
}

/// The arguments of the merge request a client makes against a repository reply.
fn merge_request(repo: &'static str) -> Result<Vec<String>> {
    let runner = FakeRunner::new(repo);
    let client = GhClient::new(&runner, "example/project".into());
    client.merge(7, "abc", &text())?;
    let calls = runner.calls.borrow();
    Ok(calls.last().unwrap().clone())
}

#[test]
fn with_only_squash_allowed_the_merge_asks_for_a_squash_carrying_the_jobs_trailers() {
    let args = merge_request(SQUASH_ONLY).unwrap();
    assert_eq!(args[2], "PUT");
    for wanted in [
        "sha=abc",
        "merge_method=squash",
        "commit_title=Change a",
        "commit_message=Body.\n\nAthena-Job: bootstrap\n",
    ] {
        assert!(args.contains(&wanted.to_string()), "{wanted:?} in {args:?}");
    }
    assert!(!args.contains(&"merge_method=merge".to_string()));
}

#[test]
fn the_merge_method_is_chosen_from_what_the_host_allows() {
    let flags = |merge, squash, rebase| MergeMethods {
        merge,
        squash,
        rebase,
    };
    assert_eq!(
        flags(true, true, true).choose().unwrap(),
        MergeMethod::Merge
    );
    assert_eq!(
        flags(false, true, true).choose().unwrap(),
        MergeMethod::Squash
    );
    assert_eq!(
        flags(false, false, true).choose().unwrap(),
        MergeMethod::Rebase
    );
    assert_eq!(
        MergeMethods::from_repository(&json!({
            "allow_merge_commit": false, "allow_squash_merge": true, "allow_rebase_merge": false
        })),
        flags(false, true, false)
    );
    // A flag the host leaves out is not allowed.
    assert_eq!(
        MergeMethods::from_repository(&json!({})),
        flags(false, false, false)
    );
}

#[test]
fn with_no_merge_method_allowed_the_merge_fails_clearly_and_sends_nothing() {
    let none = r#"{"default_branch": "trunk", "allow_merge_commit": false,
            "allow_squash_merge": false, "allow_rebase_merge": false}"#;
    let runner = FakeRunner::new(none);
    let client = GhClient::new(&runner, "example/project".into());
    let err = client.merge(7, "abc", &text()).unwrap_err().to_string();
    assert!(err.contains("allows no merge method"), "{err}");
    assert!(err.contains("allow_squash_merge false"), "{err}");
    // Only the read of the repository happened; no PUT.
    assert_eq!(runner.calls.borrow().len(), 1);
}

#[test]
fn the_ci_run_is_read_with_its_sha_address_conclusions_and_duration() {
    let sha = "0123456789abcdef0123456789abcdef01234567";
    let runs = json!({"workflow_runs": [
            {"name": "pages", "head_sha": sha, "status": "completed", "conclusion": "failure",
             "event": "push", "head_branch": CI_BRANCH,
             "html_url": "https://example.test/pages", "created_at": "2026-01-01T00:00:00Z",
             "updated_at": "2026-01-01T00:09:00Z", "id": 1},
            {"name": "ci", "head_sha": sha, "status": "completed", "conclusion": "success",
             "event": "push", "head_branch": CI_BRANCH,
             "html_url": "https://example.test/ci/2", "created_at": "2026-01-01T00:00:00Z",
             "updated_at": "2026-01-01T00:02:30Z", "id": 2}]});
    let jobs = ci_jobs(&[
        ("fmt, clippy, tests", "completed", "success"),
        ("the full history", "completed", "failure"),
    ]);
    let run = parse_ci_run(&runs, Some(&jobs), CI_BRANCH).unwrap();
    assert_eq!(run.sha, sha);
    assert_eq!(run.address, "https://example.test/ci/2");
    assert!(run.finished);
    assert_eq!(run.duration_s, 150.0);
    assert_eq!(run.steps.len(), 3);
    assert!(run.steps.iter().all(|s| s.passed()));
    // Without the jobs reply, or for a run of another workflow only, there are no step results.
    assert!(
        parse_ci_run(&runs, None, CI_BRANCH)
            .unwrap()
            .steps
            .is_empty()
    );
    let only_pages = json!({"workflow_runs": [runs["workflow_runs"][0].clone()]});
    assert!(parse_ci_run(&only_pages, None, CI_BRANCH).is_none());
    // A run for another commit keeps its own SHA, so the SHA check can reject it.
    let other = json!({"workflow_runs": [{"name": "ci", "head_sha": "fedcba",
            "event": "push", "head_branch": CI_BRANCH,
            "status": "completed", "conclusion": "success", "html_url": "u"}]});
    assert_eq!(parse_ci_run(&other, None, CI_BRANCH).unwrap().sha, "fedcba");
}

/// The CI branch of a job, as the push run is named.
const CI_BRANCH: &str = "athena/ci/20261031-142738-g437";
const CI_SHA: &str = "0123456789abcdef0123456789abcdef01234567";

/// One run of the `ci` workflow for [`CI_SHA`], as the runs reply lists it.
fn ci_run(
    id: u64,
    event: &str,
    branch: &str,
    status: &str,
    conclusion: &str,
    created: &str,
) -> Value {
    json!({"name": "ci", "id": id, "event": event, "head_branch": branch, "head_sha": CI_SHA,
        "status": status, "conclusion": conclusion,
        "html_url": format!("https://example.test/ci/{id}"),
        "created_at": created, "updated_at": "2026-01-01T01:00:00Z"})
}

/// A jobs reply from `(name, status, conclusion)` triples.
fn ci_jobs(jobs: &[(&str, &str, &str)]) -> Value {
    let list: Vec<Value> = jobs
        .iter()
        .map(|(name, status, conclusion)| {
            json!({"name": name, "status": status,
                "conclusion": if conclusion.is_empty() { Value::Null } else { Value::from(*conclusion) }})
        })
        .collect();
    json!({"jobs": list})
}

/// The jobs of a green commit, named as `ci.yml` names them.
fn green_jobs() -> Value {
    ci_jobs(&[
        ("gate (reuse of an earlier pass)", "completed", "success"),
        ("fmt", "completed", "success"),
        ("clippy", "completed", "success"),
        ("test (shard 0)", "completed", "success"),
        ("fmt, clippy, tests", "completed", "success"),
        ("the full history", "completed", "success"),
    ])
}

/// A `Runner` that answers the runs query and the jobs query of the CI source from canned text and records
/// every call. The runs answer ignores the query on purpose, so the parser's own filter is what is tested.
struct CiRunner {
    calls: RefCell<Vec<Vec<String>>>,
    runs: Value,
    /// The jobs reply per run id.
    jobs: BTreeMap<u64, Value>,
}
impl CiRunner {
    fn new(runs: Vec<Value>, jobs: Vec<(u64, Value)>) -> Self {
        CiRunner {
            calls: RefCell::new(vec![]),
            runs: json!({"workflow_runs": runs}),
            jobs: jobs.into_iter().collect(),
        }
    }
    fn paths(&self) -> Vec<String> {
        self.calls
            .borrow()
            .iter()
            .map(|a| a.iter().find(|p| p.starts_with("repos/")).unwrap().clone())
            .collect()
    }
}
impl Runner for CiRunner {
    fn gh(&self, args: &[String]) -> Result<String> {
        self.calls.borrow_mut().push(args.to_vec());
        let path = args.iter().find(|a| a.starts_with("repos/")).unwrap();
        if path.starts_with("repos/example/project/actions/runs?") {
            return Ok(self.runs.to_string());
        }
        let id = path
            .strip_prefix("repos/example/project/actions/runs/")
            .and_then(|rest| rest.split('/').next())
            .and_then(|id| id.parse::<u64>().ok());
        match id.and_then(|id| self.jobs.get(&id)) {
            Some(jobs) if path.contains("/jobs") => Ok(jobs.to_string()),
            _ => anyhow::bail!("unexpected call {path}"),
        }
    }
}

/// A local fallback that fails every check and counts its calls.
struct LocalChecks {
    calls: Cell<u32>,
}
impl crate::ci_result::LocalChecks for LocalChecks {
    fn run(&self, _: &str) -> Result<Vec<crate::ci_result::Step>> {
        self.calls.set(self.calls.get() + 1);
        Ok(crate::ci_result::STEPS
            .iter()
            .map(|name| crate::ci_result::Step {
                name: name.to_string(),
                conclusion: "failure".into(),
                duration_s: 0.0,
            })
            .collect())
    }
}

fn one_poll() -> crate::ci_result::Wait {
    crate::ci_result::Wait {
        polls: 1,
        pause: std::time::Duration::ZERO,
    }
}

#[test]
fn ci_push_run_wins_over_newer_pull_request_run() {
    let pull = ci_run(
        20,
        "pull_request",
        "athena/j",
        "completed",
        "failure",
        "2026-01-01T00:30:00Z",
    );
    let push = ci_run(
        10,
        "push",
        CI_BRANCH,
        "completed",
        "success",
        "2026-01-01T00:10:00Z",
    );
    let runner = CiRunner::new(
        vec![pull.clone(), push.clone()],
        vec![(10, green_jobs()), (20, green_jobs())],
    );
    // The parser alone.
    let runs = runner.runs.clone();
    let parsed = parse_ci_run(&runs, Some(&green_jobs()), CI_BRANCH).unwrap();
    assert_eq!(parsed.address, "https://example.test/ci/10");
    assert_eq!(parsed.conclusion, "success");
    assert_eq!(parsed.steps.len(), 3);
    assert!(parsed.steps.iter().all(|s| s.passed()));
    // The source, end to end: the jobs are read for the push run's id.
    let client = GhClient::new(&runner, "example/project".into());
    let run = crate::ci_result::CiSource::fetch(&client.ci_source(CI_BRANCH), CI_SHA)
        .unwrap()
        .unwrap();
    assert_eq!(run.address, "https://example.test/ci/10");
    assert_eq!(run.conclusion, "success");
    assert!(run.steps.iter().all(|s| s.passed()));
    let paths = runner.paths();
    assert!(paths[1].starts_with("repos/example/project/actions/runs/10/jobs"));

    // The reverse: a newer green pull request run does not hide a red push run.
    let red_jobs = ci_jobs(&[
        ("fmt", "completed", "success"),
        ("clippy", "completed", "failure"),
        ("test (shard 0)", "completed", "success"),
        ("fmt, clippy, tests", "completed", "failure"),
    ]);
    let green_pull = ci_run(
        20,
        "pull_request",
        "athena/j",
        "completed",
        "success",
        "2026-01-01T00:30:00Z",
    );
    let red_push = ci_run(
        10,
        "push",
        CI_BRANCH,
        "completed",
        "failure",
        "2026-01-01T00:10:00Z",
    );
    let runner = CiRunner::new(
        vec![green_pull, red_push],
        vec![(10, red_jobs), (20, green_jobs())],
    );
    let client = GhClient::new(&runner, "example/project".into());
    let run = crate::ci_result::CiSource::fetch(&client.ci_source(CI_BRANCH), CI_SHA)
        .unwrap()
        .unwrap();
    assert_eq!(run.address, "https://example.test/ci/10");
    assert_eq!(run.conclusion, "failure");
}

#[test]
fn ci_push_run_ignores_other_events_and_branches() {
    let cases = [
        // Only a pull request run.
        ci_run(
            20,
            "pull_request",
            CI_BRANCH,
            "completed",
            "success",
            "2026-01-01T00:30:00Z",
        ),
        // A push run on another branch.
        ci_run(
            21,
            "push",
            "athena/ci/another-job",
            "completed",
            "success",
            "2026-01-01T00:30:00Z",
        ),
        // A push run on the default branch.
        ci_run(
            22,
            "push",
            "main",
            "completed",
            "success",
            "2026-01-01T00:30:00Z",
        ),
    ];
    for case in cases {
        let id = case["id"].as_u64().unwrap();
        let runner = CiRunner::new(vec![case.clone()], vec![(id, green_jobs())]);
        let runs = runner.runs.clone();
        assert!(parse_ci_run(&runs, Some(&green_jobs()), CI_BRANCH).is_none());
        assert!(parse_ci_run(&runs, None, CI_BRANCH).is_none());
        let client = GhClient::new(&runner, "example/project".into());
        let source = client.ci_source(CI_BRANCH);
        assert!(
            crate::ci_result::CiSource::fetch(&source, CI_SHA)
                .unwrap()
                .is_none()
        );
        // No jobs were read for the run that was not chosen.
        assert_eq!(runner.paths().len(), 1);
        // So `resolve` finds no result and runs the checks locally; the green run is never used.
        let local = LocalChecks {
            calls: Cell::new(0),
        };
        let outcome = crate::ci_result::resolve(CI_SHA, &[], &source, &local, one_poll());
        match outcome {
            crate::ci_result::Outcome::Failed(record) => {
                assert_eq!(record.source, crate::ci_result::Source::Local);
            }
            other => panic!("expected a local failure, got {other:?}"),
        }
        assert_eq!(local.calls.get(), 1);
    }
}

#[test]
fn ci_push_run_query_names_event_and_branch() {
    let push = ci_run(
        10,
        "push",
        CI_BRANCH,
        "completed",
        "success",
        "2026-01-01T00:10:00Z",
    );
    let runner = CiRunner::new(vec![push], vec![(10, green_jobs())]);
    let client = GhClient::new(&runner, "example/project".into());
    crate::ci_result::CiSource::fetch(&client.ci_source(CI_BRANCH), CI_SHA).unwrap();
    let paths = runner.paths();
    assert_eq!(paths.len(), 2, "{paths:?}");
    assert!(
        paths[0].starts_with("repos/example/project/actions/runs?"),
        "{paths:?}"
    );
    for wanted in [
        format!("head_sha={CI_SHA}"),
        "event=push".to_string(),
        format!("branch={CI_BRANCH}"),
    ] {
        assert!(paths[0].contains(&wanted), "{wanted} in {}", paths[0]);
    }
    assert_eq!(
        paths[1],
        "repos/example/project/actions/runs/10/jobs?filter=latest&per_page=100"
    );
}

#[test]
fn ci_push_run_verdict_comes_from_the_aggregate_job() {
    let finished = ci_run(
        10,
        "push",
        CI_BRANCH,
        "completed",
        "failure",
        "2026-01-01T00:10:00Z",
    );
    let runs = json!({"workflow_runs": [finished.clone()]});
    let parse = |jobs: Value| parse_ci_run(&runs, Some(&jobs), CI_BRANCH).unwrap();
    let step = |run: &crate::ci_result::CiRun, name: &str| {
        run.steps
            .iter()
            .find(|s| s.name == name)
            .unwrap()
            .conclusion
            .clone()
    };

    // (a) A failed clippy job: red, and each step has its own job's result.
    let run = parse(ci_jobs(&[
        ("gate (reuse of an earlier pass)", "completed", "success"),
        ("fmt", "completed", "success"),
        ("clippy", "completed", "failure"),
        ("test (shard 0)", "completed", "success"),
        ("fmt, clippy, tests", "completed", "failure"),
    ]));
    assert!(run.finished);
    assert_eq!(run.conclusion, "failure");
    assert_eq!(step(&run, "clippy"), "failure");
    assert_eq!(step(&run, "fmt"), "success");
    assert_eq!(step(&run, "test"), "success");

    // (b) A failed fmt job.
    let run = parse(ci_jobs(&[
        ("fmt", "completed", "failure"),
        ("clippy", "completed", "success"),
        ("test (shard 0)", "completed", "success"),
        ("test (shard 1)", "completed", "success"),
        ("fmt, clippy, tests", "completed", "failure"),
    ]));
    assert_eq!(run.conclusion, "failure");
    assert_eq!(step(&run, "fmt"), "failure");
    assert_eq!(step(&run, "test"), "success");

    // A failed shard makes the test step red, and a cancelled check job does not hide it.
    let run = parse(ci_jobs(&[
        ("fmt", "completed", "cancelled"),
        ("clippy", "completed", "success"),
        ("test (shard 0)", "completed", "success"),
        ("test (shard 1)", "completed", "failure"),
        ("fmt, clippy, tests", "completed", "failure"),
    ]));
    assert_eq!(run.conclusion, "failure");
    assert_eq!(step(&run, "test"), "failure");
    assert_eq!(step(&run, "fmt"), "cancelled");

    // (c) A cancelled check job with a failed aggregate job says nothing about the tree.
    let cancelled_jobs = ci_jobs(&[
        ("fmt", "completed", "cancelled"),
        ("clippy", "completed", "success"),
        ("test (shard 0)", "completed", "success"),
        ("fmt, clippy, tests", "completed", "failure"),
    ]);
    let run = parse(cancelled_jobs.clone());
    assert_eq!(run.conclusion, "unusable");
    assert!(run.steps.is_empty());
    let runner = CiRunner::new(vec![finished.clone()], vec![(10, cancelled_jobs)]);
    let client = GhClient::new(&runner, "example/project".into());
    let source = client.ci_source(CI_BRANCH);
    let local = LocalChecks {
        calls: Cell::new(0),
    };
    let outcome = crate::ci_result::resolve(CI_SHA, &[], &source, &local, one_poll());
    match outcome {
        crate::ci_result::Outcome::Failed(record) => {
            assert_eq!(record.source, crate::ci_result::Source::Local);
        }
        other => panic!("expected the local fallback, got {other:?}"),
    }
    assert_eq!(local.calls.get(), 1);

    // (d) A failed gate: the check jobs were skipped.
    let run = parse(ci_jobs(&[
        ("gate (reuse of an earlier pass)", "completed", "failure"),
        ("fmt", "completed", "skipped"),
        ("clippy", "completed", "skipped"),
        ("test (shard 0)", "completed", "skipped"),
        ("fmt, clippy, tests", "completed", "failure"),
    ]));
    assert_eq!(run.conclusion, "unusable");
    assert!(run.steps.is_empty());

    // (e) A successful aggregate job: three green steps, also when the check jobs were skipped.
    let run = parse(green_jobs());
    assert_eq!(run.conclusion, "success");
    assert_eq!(
        run.steps
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>(),
        ["fmt", "clippy", "test"]
    );
    assert!(run.steps.iter().all(|s| s.passed()));
    let run = parse(ci_jobs(&[
        ("gate (reuse of an earlier pass)", "completed", "success"),
        ("fmt", "completed", "skipped"),
        ("clippy", "completed", "skipped"),
        ("test (shard 0)", "completed", "skipped"),
        ("fmt, clippy, tests", "completed", "success"),
    ]));
    assert_eq!(run.conclusion, "success");
    assert!(run.steps.iter().all(|s| s.passed()));

    // (f) A run failed only by the history job, with a successful aggregate job, is green.
    let run = parse(ci_jobs(&[
        ("fmt, clippy, tests", "completed", "success"),
        ("the full history", "completed", "failure"),
    ]));
    assert_eq!(run.conclusion, "success");
    assert!(run.steps.iter().all(|s| s.passed()));

    // (g) The aggregate job still runs.
    let run = parse(ci_jobs(&[
        ("fmt", "completed", "failure"),
        ("fmt, clippy, tests", "in_progress", ""),
    ]));
    assert!(!run.finished);
    assert!(run.steps.is_empty());

    // (h) A cancelled aggregate job keeps its conclusion and has no steps.
    let run = parse(ci_jobs(&[("fmt, clippy, tests", "completed", "cancelled")]));
    assert!(run.finished);
    assert_eq!(run.conclusion, "cancelled");
    assert!(run.steps.is_empty());

    // (i) No aggregate job in a finished run.
    let run = parse(ci_jobs(&[("the full history", "completed", "success")]));
    assert_eq!(run.conclusion, "unusable");
    assert!(run.steps.is_empty());

    // (j) A job named `fmt` is not the aggregate job.
    let run = parse(ci_jobs(&[("fmt", "completed", "success")]));
    assert_eq!(run.conclusion, "unusable");
    assert!(run.steps.is_empty());
}

#[test]
fn checks_are_classified_by_the_required_check_alone() {
    let required = super::client::CI_AGGREGATE_JOB;
    let done = |c: &str| json!({"check_runs": [{"id": 1, "name": required, "status": "completed", "conclusion": c}]});
    let none = json!({"state": "pending", "statuses": []});
    assert_eq!(classify_checks(&done("success"), &none), Checks::Green);
    assert_eq!(classify_checks(&done("skipped"), &none), Checks::Green);
    assert_eq!(classify_checks(&done("failure"), &none), Checks::Red);
    assert_eq!(classify_checks(&done("cancelled"), &none), Checks::Red);
    let running = json!({"check_runs": [
        {"id": 1, "name": required, "status": "in_progress", "conclusion": null}]});
    assert_eq!(classify_checks(&running, &none), Checks::Pending);
    // No report at all, or none from the required check, is not green.
    assert_eq!(
        classify_checks(&json!({"check_runs": []}), &none),
        Checks::Pending
    );
    let other_only = json!({"check_runs": [
        {"id": 2, "name": "pull request head", "status": "completed", "conclusion": "success"}]});
    assert_eq!(classify_checks(&other_only, &none), Checks::Pending);
    // A check main does not require, failed or still running, never decides.
    let optional_red = json!({"check_runs": [
        {"id": 1, "name": required, "status": "completed", "conclusion": "success"},
        {"id": 2, "name": "pull request head", "status": "completed", "conclusion": "failure"}]});
    assert_eq!(classify_checks(&optional_red, &none), Checks::Green);
    let optional_running = json!({"check_runs": [
        {"id": 1, "name": required, "status": "completed", "conclusion": "success"},
        {"id": 2, "name": "pull request head", "status": "in_progress", "conclusion": null}]});
    assert_eq!(classify_checks(&optional_running, &none), Checks::Green);
    // A status context is not a required check either.
    let failed = json!({"state": "failure", "statuses": [{"state": "failure", "context": "x"}]});
    assert_eq!(classify_checks(&done("success"), &failed), Checks::Green);
    // After a re-run the newest run of the required check decides.
    let rerun = json!({"check_runs": [
        {"id": 5, "name": required, "status": "completed", "conclusion": "failure"},
        {"id": 8, "name": required, "status": "completed", "conclusion": "success"}]});
    assert_eq!(classify_checks(&rerun, &none), Checks::Green);
}

#[test]
fn the_failed_checks_main_does_not_require_are_named_once_each() {
    let required = super::client::CI_AGGREGATE_JOB;
    let runs = json!({"check_runs": [
        {"id": 1, "name": required, "status": "completed", "conclusion": "failure"},
        {"id": 2, "name": "pull request head", "status": "completed", "conclusion": "failure"},
        {"id": 3, "name": "pull request head", "status": "completed", "conclusion": "failure"},
        {"id": 4, "name": "attribution", "status": "completed", "conclusion": "success"}]});
    let status = json!({"state": "failure", "statuses": [
        {"state": "error", "context": "legacy"}, {"state": "success", "context": "fine"}]});
    assert_eq!(
        optional_failures(&runs, &status),
        vec!["pull request head".to_string(), "legacy".to_string()]
    );
}

#[test]
fn remotes_are_split_into_host_and_repository() {
    let ok = ("github.com".to_string(), "example/project".to_string());
    for url in [
        "https://github.com/example/project.git\n",
        "https://github.com/example/project",
        "ssh://github.com/example/project.git",
        "github.com:example/project.git",
    ] {
        assert_eq!(parse_remote(url).unwrap(), ok, "{url}");
    }
    assert!(parse_remote("https://gitlab.example.com/example/project").is_err());
    assert!(parse_remote("https://github.com/example").is_err());
}

// ---- the flow, with a fake host ----

struct Anc(bool);
impl Ancestry for Anc {
    fn is_ancestor(&self, _: &str, _: &str) -> Result<bool> {
        Ok(self.0)
    }
}

#[derive(Default)]
struct FakeRemote {
    /// What the remote branch holds; a push moves it.
    head: RefCell<Option<String>>,
    pushes: RefCell<Vec<(String, String)>>,
    /// Each replace as (branch, new commit, expected old head).
    replaces: RefCell<Vec<(String, String, String)>>,
    /// The commits `commits_after` reports.
    extras: RefCell<Vec<(String, String)>>,
    /// A log shared with the host and the sleep, in which each replace is entered as `replace`.
    log: RefCell<Option<EventLog>>,
}
type EventLog = std::rc::Rc<RefCell<Vec<String>>>;
impl Remote for FakeRemote {
    fn remote_head(&self, _: &str) -> Result<Option<String>> {
        Ok(self.head.borrow().clone())
    }
    fn replace(&self, branch: &str, commit: &str, expected_old: &str) -> Result<()> {
        if self.head.borrow().as_deref() != Some(expected_old) {
            bail!("the remote branch moved");
        }
        if let Some(log) = self.log.borrow().as_ref() {
            log.borrow_mut().push("replace".to_string());
        }
        self.replaces.borrow_mut().push((
            branch.to_string(),
            commit.to_string(),
            expected_old.to_string(),
        ));
        *self.head.borrow_mut() = Some(commit.to_string());
        Ok(())
    }
    fn push(&self, branch: &str, commit: &str) -> Result<()> {
        self.pushes
            .borrow_mut()
            .push((branch.to_string(), commit.to_string()));
        *self.head.borrow_mut() = Some(commit.to_string());
        Ok(())
    }
    fn commits_after(&self, _: &str, _: &str) -> Result<Vec<(String, String)>> {
        Ok(self.extras.borrow().clone())
    }
}

struct FakeHost {
    checks: Checks,
    /// What the host reports as merged now.
    merged: RefCell<Option<String>>,
    /// The commit a merge request produces.
    merge_makes: &'static str,
    opened: Cell<u32>,
    merges: Cell<u32>,
    check_reads: Cell<u32>,
    asked: RefCell<Vec<u64>>,
    /// The bases pull requests were opened against.
    bases: RefCell<Vec<String>>,
    /// The commits whose checks were read.
    checked: RefCell<Vec<String>>,
    /// Reads taken from the front before `checks` is returned for every later read.
    script: RefCell<Vec<Checks>>,
    /// How many of the first run looks (before any re-run) show the run still in progress.
    in_progress_looks: Cell<u32>,
    /// How many run looks after the re-run still show the old try.
    stale_looks: Cell<u32>,
    /// Whether the host reports `run_attempt`.
    reports_attempt: Cell<bool>,
    run_looks: Cell<u32>,
    looks_after_rerun: Cell<u32>,
    reruns: Cell<u32>,
    /// How many run looks had happened at each re-run request.
    run_looks_at_rerun: RefCell<Vec<u32>>,
    /// The job log the failing test is read from.
    log: RefCell<String>,
    /// Checks that depend on the commit asked about; other commits use `script` and `checks`.
    per_commit: RefCell<BTreeMap<String, Checks>>,
    /// The head each merge request carried as its guard.
    merge_heads: RefCell<Vec<String>>,
    /// A merge request that the host refuses.
    merge_refused: Cell<bool>,
    /// Whether a merge makes the host report the merged commit (false: merged but not confirmed).
    merge_confirms: Cell<bool>,
    /// Run when a merge request arrives, before the host answers.
    on_merge: RefCell<Option<Box<dyn Fn()>>>,
}
impl FakeHost {
    fn new(checks: Checks) -> Self {
        FakeHost {
            merge_refused: Cell::new(false),
            merge_confirms: Cell::new(true),
            on_merge: RefCell::new(None),
            per_commit: RefCell::new(BTreeMap::new()),
            merge_heads: RefCell::new(vec![]),
            script: RefCell::new(vec![]),
            in_progress_looks: Cell::new(0),
            stale_looks: Cell::new(0),
            reports_attempt: Cell::new(true),
            run_looks: Cell::new(0),
            looks_after_rerun: Cell::new(0),
            reruns: Cell::new(0),
            run_looks_at_rerun: RefCell::new(vec![]),
            log: RefCell::new(String::new()),
            checks,
            merged: RefCell::new(None),
            merge_makes: "merged1",
            opened: Cell::new(0),
            merges: Cell::new(0),
            check_reads: Cell::new(0),
            asked: RefCell::new(vec![]),
            bases: RefCell::new(vec![]),
            checked: RefCell::new(vec![]),
        }
    }
}
impl Host for FakeHost {
    fn merged_commit(&self, link: &PrLink) -> Result<Option<String>> {
        self.asked.borrow_mut().push(link.pr_number);
        Ok(self.merged.borrow().clone())
    }
}
impl PrHost for FakeHost {
    fn default_branch(&self) -> Result<String> {
        Ok("trunk".to_string())
    }
    fn open_pr(&self, pr: &NewPr) -> Result<u64> {
        self.opened.set(self.opened.get() + 1);
        self.bases.borrow_mut().push(pr.base.to_string());
        Ok(42)
    }
    fn checks(&self, commit: &str) -> Result<Checks> {
        self.check_reads.set(self.check_reads.get() + 1);
        self.checked.borrow_mut().push(commit.to_string());
        if let Some(known) = self.per_commit.borrow().get(commit) {
            return Ok(*known);
        }
        let mut script = self.script.borrow_mut();
        Ok(if script.is_empty() {
            self.checks
        } else {
            script.remove(0)
        })
    }
    fn check_run(&self, _: &str, _: &str) -> Result<Option<CiRunState>> {
        Ok(None)
    }
    fn ci_runs(&self, _: &str) -> Result<Vec<CiRunState>> {
        self.run_looks.set(self.run_looks.get() + 1);
        let attempt = if self.reruns.get() == 0 {
            1
        } else {
            self.looks_after_rerun.set(self.looks_after_rerun.get() + 1);
            if self.looks_after_rerun.get() <= self.stale_looks.get() {
                1
            } else {
                2
            }
        };
        let going = self.reruns.get() == 0 && self.run_looks.get() <= self.in_progress_looks.get();
        Ok(vec![CiRunState {
            id: 9,
            status: if going { "in_progress" } else { "completed" }.into(),
            conclusion: (!going).then(|| "failure".to_string()),
            attempt: self.reports_attempt.get().then_some(attempt),
        }])
    }
    fn rerun_failed(&self, run_id: u64) -> Result<()> {
        assert_eq!(run_id, 9);
        self.run_looks_at_rerun
            .borrow_mut()
            .push(self.run_looks.get());
        self.reruns.set(self.reruns.get() + 1);
        Ok(())
    }
    fn failing_test(&self, _: u64) -> Result<Option<String>> {
        Ok(failing_test_name(&self.log.borrow()))
    }
    fn merge(&self, _: u64, head: &str, _: &CommitText) -> Result<MergeMethod> {
        self.merges.set(self.merges.get() + 1);
        self.merge_heads.borrow_mut().push(head.to_string());
        if let Some(hook) = self.on_merge.borrow().as_ref() {
            hook();
        }
        if self.merge_refused.get() {
            bail!("the host refused the merge");
        }
        if self.merge_confirms.get() {
            *self.merged.borrow_mut() = Some(self.merge_makes.to_string());
        }
        Ok(MergeMethod::Merge)
    }
}

struct FakeContent(BTreeMap<&'static str, &'static str>);
impl Content for FakeContent {
    fn fingerprint_of(&self, commit: &str) -> Result<String> {
        Ok(self.0[commit].to_string())
    }
}

struct Rig {
    dir: tempfile::TempDir,
    job: JobId,
    remote: FakeRemote,
    content: FakeContent,
    /// The poll's look budget.
    attempts: u32,
    /// Every sleep (as `sleep <seconds>`), early-run look and replace, in order.
    log: EventLog,
}

impl Rig {
    fn new() -> Self {
        let log = EventLog::default();
        Rig {
            attempts: 3,
            remote: FakeRemote {
                log: RefCell::new(Some(log.clone())),
                ..FakeRemote::default()
            },
            log,
            dir: tempfile::tempdir().unwrap(),
            job: JobId::parse("bootstrap").unwrap(),
            content: FakeContent(BTreeMap::from([
                ("merged1", "verified"),
                ("other", "different"),
            ])),
        }
    }

    fn run(&self, host: &FakeHost, merge_via_rest: bool, branch: &str) -> Result<PrReport> {
        self.run_with(host, merge_via_rest, branch, "cand1", &Anc(true))
    }

    fn run_with(
        &self,
        host: &dyn PrHost,
        merge_via_rest: bool,
        branch: &str,
        commit: &str,
        ancestry: &dyn Ancestry,
    ) -> Result<PrReport> {
        let state = StateDir::new(self.dir.path().to_path_buf());
        let nap = |d: Duration| self.log.borrow_mut().push(format!("sleep {}", d.as_secs()));
        land_pr(&LandPr {
            state: &state,
            job: &self.job,
            branch,
            commit,
            verified_fingerprint: "verified",
            pr: PrDraft {
                title: "t",
                body: "b",
                head: "athena/bootstrap",
            },
            squash: &text(),
            host_name: HOST,
            repository: "example/project",
            merge_via_rest,
            poll: Poll {
                attempts: self.attempts,
                delay: Duration::from_secs(1),
                sleep: &nap,
            },
            ancestry,
            remote: &self.remote,
            host,
            content: &self.content,
            records: None,
        })
    }

    fn stored(&self) -> PrLink {
        let state = StateDir::new(self.dir.path().to_path_buf());
        load_link(&state, &self.job).unwrap().unwrap()
    }
}

#[test]
fn with_the_setting_off_no_merge_is_requested_and_the_pull_request_stays_open_and_green() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    let report = rig.run(&host, false, "athena/bootstrap").unwrap();
    assert_eq!(host.merges.get(), 0);
    assert!(!report.merge_requested);
    assert_eq!(report.checks, Some(Checks::Green));
    assert_eq!(report.link.state, LinkState::Open);
    assert_eq!(report.link.pr_number, 42);
    assert_eq!(report.link.head, "cand1");
    assert_eq!(rig.stored(), report.link);
    // Only the job's own branch was pushed, once.
    assert_eq!(
        *rig.remote.pushes.borrow(),
        vec![("athena/bootstrap".to_string(), "cand1".to_string())]
    );
    assert_eq!(host.opened.get(), 1);
}

#[test]
fn with_the_setting_on_and_green_checks_the_rest_merge_is_requested_and_the_job_lands() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    let report = rig.run(&host, true, "athena/bootstrap").unwrap();
    assert_eq!(host.merges.get(), 1);
    assert!(report.merge_requested);
    assert_eq!(report.link.state, LinkState::Landed);
    assert_eq!(report.link.merged_commit.as_deref(), Some("merged1"));
    assert_eq!(rig.stored().state, LinkState::Landed);
}

#[test]
fn red_or_pending_checks_never_trigger_a_merge_even_with_the_setting_on() {
    for checks in [Checks::Red, Checks::Pending] {
        let rig = Rig::new();
        let host = FakeHost::new(checks);
        let report = rig.run(&host, true, "athena/bootstrap").unwrap();
        assert_eq!(host.merges.get(), 0, "{checks:?}");
        assert!(!report.merge_requested);
        assert_eq!(report.checks, Some(checks));
        assert_eq!(report.link.state, LinkState::Open);
    }
    // Pending checks are asked about up to the poll limit, no more.
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Pending);
    rig.run(&host, true, "athena/bootstrap").unwrap();
    assert_eq!(host.check_reads.get(), 3);
}

#[test]
fn no_rerun_is_asked_until_the_failed_run_completes_and_then_exactly_one() {
    let mut rig = Rig::new();
    rig.attempts = 4;
    let host = FakeHost::new(Checks::Red);
    // The first read is red while the run is still going; it is in progress for two looks.
    host.in_progress_looks.set(2);
    *host.script.borrow_mut() = vec![Checks::Red, Checks::Red, Checks::Red];
    let report = rig.run(&host, true, "athena/bootstrap").unwrap();
    assert_eq!(host.reruns.get(), 1);
    // The request came at the third look, the first that showed the run completed.
    assert_eq!(*host.run_looks_at_rerun.borrow(), vec![3]);
    assert!(host.check_reads.get() <= 2 * rig.attempts);
    assert_eq!(report.reruns, 1);
    assert_eq!(report.checks, Some(Checks::Red));
    assert_eq!(host.merges.get(), 0);
}

#[test]
fn a_stale_run_attempt_is_not_taken_as_the_result_of_the_rerun() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Red);
    host.stale_looks.set(1);
    *host.script.borrow_mut() = vec![Checks::Red, Checks::Red, Checks::Pending, Checks::Red];
    let report = rig.run(&host, true, "athena/bootstrap").unwrap();
    // One read in the first wait and three in the second: the first of them is stale and ignored.
    assert_eq!(host.check_reads.get(), 4);
    assert_eq!(host.reruns.get(), 1);
    assert_eq!(report.reruns, 1);
    assert_eq!(report.checks, Some(Checks::Red));
    assert_eq!(host.merges.get(), 0);
}

#[test]
fn a_host_that_reports_no_run_attempt_is_trusted_from_the_second_look() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Red);
    host.reports_attempt.set(false);
    *host.script.borrow_mut() = vec![Checks::Red, Checks::Red, Checks::Green];
    let report = rig.run(&host, false, "athena/bootstrap").unwrap();
    assert_eq!(host.check_reads.get(), 3);
    assert_eq!(report.checks, Some(Checks::Green));
}

#[test]
fn a_rerun_that_turns_green_is_reported_green_and_merged_when_the_setting_is_on() {
    for merge_via_rest in [true, false] {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Red);
        *host.script.borrow_mut() = vec![Checks::Red, Checks::Pending, Checks::Green];
        let report = rig.run(&host, merge_via_rest, "athena/bootstrap").unwrap();
        assert_eq!(report.checks, Some(Checks::Green), "{merge_via_rest}");
        assert_eq!(report.reruns, 1);
        assert_eq!(report.failing_test, None);
        assert_eq!(host.reruns.get(), 1);
        assert_eq!(host.merges.get(), u32::from(merge_via_rest));
        assert_eq!(report.merge_requested, merge_via_rest);
    }
}

#[test]
fn a_second_red_merges_nothing_reruns_nothing_more_and_names_the_failing_test() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Red);
    *host.log.borrow_mut() = "2026-01-01T00:00:00.1Z test gh::tests::x ... FAILED\n".to_string();
    let report = rig.run(&host, true, "athena/bootstrap").unwrap();
    assert_eq!(host.merges.get(), 0);
    assert!(!report.merge_requested);
    assert_eq!(host.reruns.get(), 1);
    assert_eq!(report.reruns, 1);
    assert_eq!(report.checks, Some(Checks::Red));
    assert_eq!(report.failing_test.as_deref(), Some("gh::tests::x"));
    assert_eq!(report.link.state, LinkState::Open);
}

#[test]
fn checks_still_pending_after_the_bound_give_pending_with_no_rerun_and_no_merge() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Pending);
    let report = rig.run(&host, true, "athena/bootstrap").unwrap();
    assert_eq!(report.checks, Some(Checks::Pending));
    assert_eq!(host.check_reads.get(), rig.attempts);
    assert_eq!(host.reruns.get(), 0);
    assert_eq!(host.run_looks.get(), 0);
    assert_eq!(report.reruns, 0);
    assert_eq!(host.merges.get(), 0);
}

#[test]
fn a_rerun_that_never_restarts_ends_pending_and_is_not_repeated() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Red);
    host.stale_looks.set(100);
    let report = rig.run(&host, true, "athena/bootstrap").unwrap();
    assert_eq!(report.checks, Some(Checks::Pending));
    assert_eq!(host.reruns.get(), 1);
    assert_eq!(host.merges.get(), 0);
    assert!(host.check_reads.get() <= 2 * rig.attempts);
}

#[test]
fn the_failing_test_is_read_from_a_log() {
    let log = "2026-10-18T10:00:00.1234567Z running 2 tests\n\
                   2026-10-18T10:00:00.2Z test gh::tests::ok ... ok\n\
                   2026-10-18T10:00:00.3Z test odd name here ... FAILED\n\
                   2026-10-18T10:00:00.4Z test gh::tests::bad_one ... FAILED\n\
                   test result: FAILED. 1 passed; 1 failed\n";
    assert_eq!(
        failing_test_name(log).as_deref(),
        Some("gh::tests::bad_one")
    );
    let stdout_only = "---- core::thing stdout ----\nthread panicked\n";
    assert_eq!(
        failing_test_name(stdout_only).as_deref(),
        Some("core::thing")
    );
    assert_eq!(failing_test_name("test a b ... FAILED\n"), None);
    assert_eq!(failing_test_name("test result: FAILED. 0 passed\n"), None);
    let long = format!("test {} ... FAILED\n", "a".repeat(TEST_NAME_LIMIT + 1));
    assert_eq!(failing_test_name(&long), None);
}

#[test]
fn the_runs_reply_is_read_into_states() {
    let reply = json!({"workflow_runs": [
        {"id": 3, "status": "completed", "conclusion": "failure", "run_attempt": 2},
        {"id": 4, "status": "in_progress", "conclusion": null},
        {"status": "queued"}
    ]});
    let runs = parse_ci_runs(&reply);
    assert_eq!(runs.len(), 2);
    assert!(runs[0].failed());
    assert_eq!(runs[0].attempt, Some(2));
    assert!(!runs[1].failed());
    assert_eq!(runs[1].attempt, None);
}

#[test]
fn a_merge_seen_later_is_found_by_number_and_checked_by_content() {
    // Matching fingerprint: landed.
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    rig.run(&host, false, "athena/bootstrap").unwrap();
    *host.merged.borrow_mut() = Some("merged1".into());
    let report = rig.run(&host, false, "athena/bootstrap").unwrap();
    assert_eq!(report.link.state, LinkState::Landed);
    assert_eq!(report.checks, None);
    assert_eq!(host.opened.get(), 1, "no second pull request");
    assert_eq!(rig.remote.pushes.borrow().len(), 1, "no second push");
    assert_eq!(host.asked.borrow().last(), Some(&42));
    assert_eq!(rig.stored().state, LinkState::Landed);
    assert_eq!(report.link.merged_commit.as_deref(), Some("merged1"));

    // Different content: merged-unverified, never landed.
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    rig.run(&host, false, "athena/bootstrap").unwrap();
    *host.merged.borrow_mut() = Some("other".into());
    let report = rig.run(&host, false, "athena/bootstrap").unwrap();
    assert_eq!(report.link.state, LinkState::MergedUnverified);
    assert_eq!(report.link.merged_commit.as_deref(), Some("other"));
    assert_eq!(host.merges.get(), 0);
}

#[test]
fn the_client_reads_the_default_branch_from_the_repository() {
    let runner = FakeRunner::new(ALL_METHODS);
    let client = GhClient::new(&runner, "example/project".into());
    assert_eq!(client.default_branch().unwrap(), "trunk");
    let calls = runner.calls.borrow();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0][..4],
        ["api", "--method", "GET", "repos/example/project"]
    );
}

#[test]
fn the_pull_request_targets_the_host_default_branch_and_the_work_commit() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    let state = StateDir::new(rig.dir.path().to_path_buf());
    let nap = |_: Duration| {};
    let report = land_pr(&LandPr {
        state: &state,
        job: &rig.job,
        branch: "athena/bootstrap",
        commit: "work1",
        verified_fingerprint: "verified",
        pr: PrDraft {
            title: "t",
            body: "b",
            head: "athena/bootstrap",
        },
        squash: &text(),
        host_name: HOST,
        repository: "example/project",
        merge_via_rest: false,
        poll: Poll {
            attempts: 1,
            delay: Duration::ZERO,
            sleep: &nap,
        },
        ancestry: &Anc(true),
        remote: &rig.remote,
        host: &host,
        content: &rig.content,
        records: None,
    })
    .unwrap();
    // The base is what the host reports, whatever the local branch is called.
    assert_eq!(*host.bases.borrow(), vec!["trunk".to_string()]);
    // The pushed commit, the stored head and the checked commit are all the work commit.
    assert_eq!(
        *rig.remote.pushes.borrow(),
        vec![("athena/bootstrap".to_string(), "work1".to_string())]
    );
    assert_eq!(report.link.head, "work1");
    assert_eq!(*host.checked.borrow(), vec!["work1".to_string()]);
}

#[test]
fn the_landed_commit_is_found_by_the_job_and_verified_content_trailers() {
    let dir = tempfile::tempdir().unwrap();
    let git = Git::isolated(dir.path());
    let write = |text: &str, message: &str| {
        std::fs::write(dir.path().join("a.txt"), text).unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", message]).unwrap();
        git.head().unwrap().unwrap()
    };
    git.run(["init", "-q", "-b", "main"]).unwrap();
    write("a", "base");
    let job = JobId::parse("bootstrap").unwrap();
    // Not landed yet: an error, not a fallback.
    assert!(landed_commit(&git, "HEAD", &job, "fp1").is_err());
    write("b", "Read\n\nAthena-Job: bootstrap\n");
    write(
        "c",
        "Other job\n\nAthena-Job: other\nAthena-Verified-Content: fp1\n",
    );
    assert!(landed_commit(&git, "HEAD", &job, "fp1").is_err());
    let landed = write(
        "d",
        "Change\n\nAthena-Job: bootstrap\nAthena-Verified-Content: fp1\n",
    );
    write("e", "records\n\nAthena-Job: bootstrap\n");
    assert_eq!(landed_commit(&git, "HEAD", &job, "fp1").unwrap(), landed);
    assert!(landed_commit(&git, "HEAD", &job, "fp2").is_err());
}

/// A repository whose branch has no landed commit for the job.
fn unlanded_repo() -> (tempfile::TempDir, Git) {
    let dir = tempfile::tempdir().unwrap();
    let git = Git::isolated(dir.path());
    git.run(["init", "-q", "-b", "main"]).unwrap();
    std::fs::write(dir.path().join("a.txt"), "a").unwrap();
    git.run(["add", "-A"]).unwrap();
    git.run(["commit", "-q", "-m", "base"]).unwrap();
    (dir, git)
}

#[test]
fn a_merged_pull_request_confirms_the_landing_though_the_landed_commit_is_off_this_branch() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    // The first run opens the pull request and stores an open link.
    rig.run(&host, false, "athena/bootstrap").unwrap();
    assert_eq!(rig.stored().state, LinkState::Open);
    // The landed commit is not reachable from this branch.
    let (_dir, git) = unlanded_repo();
    assert!(
        landed_commit(&git, "HEAD", &rig.job, "verified")
            .unwrap_err()
            .to_string()
            .contains("not landed on this branch")
    );
    // The host reports the merge.
    *host.merged.borrow_mut() = Some("merged1".into());
    let state = StateDir::new(rig.dir.path().to_path_buf());
    let report = confirm_existing_merge(&state, &rig.job, &host, &rig.content, "verified").unwrap();
    let report = report.expect("the merge confirms the landing");
    assert_eq!(report.link.state, LinkState::Landed);
    assert_eq!(report.link.merged_commit.as_deref(), Some("merged1"));
    assert_eq!(rig.stored().state, LinkState::Landed);
    assert_eq!(host.asked.borrow().last(), Some(&42));
    // Nothing was pushed or opened, and no merge was requested.
    assert_eq!(rig.remote.pushes.borrow().len(), 1);
    assert_eq!(host.opened.get(), 1);
    assert_eq!(host.merges.get(), 0);
}

#[test]
fn a_merge_of_other_content_is_reported_as_merged_unverified_without_the_landed_commit() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    rig.run(&host, false, "athena/bootstrap").unwrap();
    *host.merged.borrow_mut() = Some("other".into());
    let state = StateDir::new(rig.dir.path().to_path_buf());
    let report = confirm_existing_merge(&state, &rig.job, &host, &rig.content, "verified")
        .unwrap()
        .unwrap();
    assert_eq!(report.link.state, LinkState::MergedUnverified);
    assert_eq!(rig.stored().state, LinkState::MergedUnverified);
}

#[test]
fn an_open_pull_request_or_no_link_keeps_the_not_landed_error() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    let state = StateDir::new(rig.dir.path().to_path_buf());
    // No stored link: the host is not asked, and the caller's error stands.
    assert!(
        confirm_existing_merge(&state, &rig.job, &host, &rig.content, "verified")
            .unwrap()
            .is_none()
    );
    assert!(host.asked.borrow().is_empty());
    let (_dir, git) = unlanded_repo();
    let err = landed_commit(&git, "HEAD", &rig.job, "verified").unwrap_err();
    assert!(err.to_string().contains("not landed on this branch"));

    // A stored link whose pull request is not merged: still no landing.
    rig.run(&host, false, "athena/bootstrap").unwrap();
    assert!(
        confirm_existing_merge(&state, &rig.job, &host, &rig.content, "verified")
            .unwrap()
            .is_none()
    );
    assert_eq!(rig.stored().state, LinkState::Open);
}

#[test]
fn a_new_branch_is_published_at_the_landed_commit() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    let report = rig
        .run_with(&host, false, "athena/bootstrap", "landed1", &Anc(true))
        .unwrap();
    assert_eq!(
        *rig.remote.pushes.borrow(),
        vec![("athena/bootstrap".to_string(), "landed1".to_string())]
    );
    assert_eq!(report.link.head, "landed1");
}

#[test]
fn a_rerun_at_the_same_commit_pushes_nothing() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    rig.run(&host, false, "athena/bootstrap").unwrap();
    rig.run(&host, false, "athena/bootstrap").unwrap();
    assert_eq!(rig.remote.pushes.borrow().len(), 1);
}

/// A pull request opened at `cand1`; then a fix `fix1` is pushed to its branch (the job's commit stays
/// `cand1`, red, and `fix1` is green). Returns the rig and host.
fn pushed_fix() -> (Rig, FakeHost) {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    host.per_commit
        .borrow_mut()
        .insert("cand1".into(), Checks::Red);
    host.per_commit
        .borrow_mut()
        .insert("fix1".into(), Checks::Green);
    rig.run(&host, false, "athena/bootstrap").unwrap();
    *rig.remote.head.borrow_mut() = Some("fix1".into());
    *rig.remote.extras.borrow_mut() = vec![("fix1".repeat(10), "Fix the failing test".into())];
    host.checked.borrow_mut().clear();
    (rig, host)
}

#[test]
fn land_pr_judges_the_current_head_not_the_old_commit() {
    let (rig, host) = pushed_fix();
    let reruns_before = host.reruns.get();
    let err = rig
        .run(&host, true, "athena/bootstrap")
        .unwrap_err()
        .to_string();
    // Only the head's checks were read, and they are green; the old red is not reported or re-run.
    assert_eq!(*host.checked.borrow(), vec!["fix1".to_string()]);
    assert!(err.contains("checks of that head are green"), "{err}");
    assert!(!err.contains("are red"), "{err}");
    assert_eq!(host.reruns.get(), reruns_before);
    assert_eq!(host.merges.get(), 0);
    assert_eq!(rig.stored().head, "fix1");
}

#[test]
fn land_pr_merges_when_the_head_is_the_landing_commit() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    let report = rig.run(&host, true, "athena/bootstrap").unwrap();
    assert!(report.merge_requested);
    assert_eq!(host.merges.get(), 1);
    assert_eq!(report.link.state, LinkState::Landed);
}

/// An ancestry that answers yes and records each question.
#[derive(Default)]
struct Spy(RefCell<Vec<(String, String)>>);
impl Ancestry for Spy {
    fn is_ancestor(&self, a: &str, d: &str) -> Result<bool> {
        self.0.borrow_mut().push((a.to_string(), d.to_string()));
        Ok(true)
    }
}

/// Runs the early step for the rig's job at `commit`.
fn early(rig: &Rig, host: &FakeHost, commit: &str) -> Result<()> {
    let state = StateDir::new(rig.dir.path().to_path_buf());
    open_early_pr(&EarlyPr {
        state: &state,
        job: &rig.job,
        commit,
        title: "t",
        repository: "owner/repo",
        ancestry: &Anc(false),
        remote: &rig.remote,
        host,
    })
}

#[test]
fn the_early_step_pushes_and_opens_once_and_never_merges() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    early(&rig, &host, "early1").unwrap();
    assert_eq!(
        *rig.remote.pushes.borrow(),
        vec![("athena/bootstrap".to_string(), "early1".to_string())]
    );
    assert_eq!(host.opened.get(), 1);
    assert_eq!(host.merges.get(), 0);
    let state = StateDir::new(rig.dir.path().to_path_buf());
    let link = load_early_link(&state, &rig.job).unwrap().unwrap();
    assert_eq!((link.pr_number, link.head.as_str()), (42, "early1"));
    assert_eq!(link.repository, "owner/repo");
    assert_eq!(
        load_early(&state, &rig.job).unwrap().as_deref(),
        Some("early1")
    );
    // A second call at the same commit changes nothing and opens no second pull request.
    early(&rig, &host, "early1").unwrap();
    assert_eq!(rig.remote.pushes.borrow().len(), 1);
    assert_eq!(host.opened.get(), 1);
}

#[test]
fn land_pr_replaces_the_early_head_pinned_and_reuses_the_early_pull_request() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    early(&rig, &host, "early1").unwrap();
    let spy = Spy::default();
    let report = rig
        .run_with(&host, true, "athena/bootstrap", "landing1", &spy)
        .unwrap();
    // The same branch, one pinned replace of exactly the early head, and no second push.
    assert_eq!(
        *rig.remote.replaces.borrow(),
        vec![(
            "athena/bootstrap".to_string(),
            "landing1".to_string(),
            "early1".to_string()
        )]
    );
    assert_eq!(rig.remote.pushes.borrow().len(), 1);
    // The replace went through check_replace, which asks how the early head relates to the new one.
    assert_eq!(
        *spy.0.borrow(),
        vec![("early1".to_string(), "landing1".to_string())]
    );
    // The early pull request is reused.
    assert_eq!(host.opened.get(), 1);
    assert_eq!(report.link.pr_number, 42);
    // The head was set to the landing commit before the head check, so the check passed and the merge
    // was guarded with it.
    assert_eq!(report.link.head, "landing1");
    assert_eq!(rig.stored().head, "landing1");
    assert_eq!(*host.merge_heads.borrow(), vec!["landing1".to_string()]);
    assert_eq!(*host.checked.borrow(), vec!["landing1".to_string()]);
    assert_eq!(report.link.state, LinkState::Landed);
    let state = StateDir::new(rig.dir.path().to_path_buf());
    assert_eq!(load_early(&state, &rig.job).unwrap(), None);
}

#[test]
fn land_pr_replaces_an_early_head_that_has_no_pull_request_and_opens_one() {
    // The early push worked and opening failed: the branch holds the early head, no link is stored.
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    let state = StateDir::new(rig.dir.path().to_path_buf());
    *rig.remote.head.borrow_mut() = Some("early1".into());
    store_early(&state, &rig.job, "early1").unwrap();
    let report = rig
        .run_with(&host, true, "athena/bootstrap", "landing1", &Spy::default())
        .unwrap();
    assert_eq!(rig.remote.replaces.borrow().len(), 1);
    assert!(rig.remote.pushes.borrow().is_empty());
    assert_eq!(host.opened.get(), 1);
    assert_eq!(*host.merge_heads.borrow(), vec!["landing1".to_string()]);
    assert_eq!(report.link.state, LinkState::Landed);
}

#[test]
fn land_pr_still_refuses_a_foreign_head_over_a_stored_early_head() {
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    early(&rig, &host, "early1").unwrap();
    // Someone else moved the branch.
    *rig.remote.head.borrow_mut() = Some("foreign1".into());
    let err = rig
        .run_with(&host, true, "athena/bootstrap", "landing1", &Anc(true))
        .unwrap_err()
        .to_string();
    assert!(err.contains("is not the job's landing commit"), "{err}");
    assert!(rig.remote.replaces.borrow().is_empty());
    assert_eq!(host.merges.get(), 0);
    assert_eq!(host.opened.get(), 1);
}

/// A host whose reads of the early commit's runs are scripted and logged. Everything else is `inner`.
struct RunHost {
    inner: FakeHost,
    /// Replies to the early commit's run looks, taken from the front; the last repeats. `Err` is an
    /// unreadable reply.
    script: RefCell<Vec<Look>>,
    log: EventLog,
    bound: Duration,
}
/// One look at the early commit's check runs: each job's name and state, or an unreadable reply.
type Look = std::result::Result<Vec<(String, CiRunState)>, ()>;
impl RunHost {
    fn new(rig: &Rig, script: Vec<Look>) -> Self {
        RunHost {
            inner: FakeHost::new(Checks::Green),
            script: RefCell::new(script),
            log: rig.log.clone(),
            bound: Duration::from_secs(60),
        }
    }
}
impl Host for RunHost {
    fn merged_commit(&self, link: &PrLink) -> Result<Option<String>> {
        self.inner.merged_commit(link)
    }
}
impl PrHost for RunHost {
    fn default_branch(&self) -> Result<String> {
        self.inner.default_branch()
    }
    fn open_pr(&self, pr: &NewPr) -> Result<u64> {
        self.inner.open_pr(pr)
    }
    fn checks(&self, commit: &str) -> Result<Checks> {
        self.inner.checks(commit)
    }
    fn merge(&self, number: u64, head: &str, text: &CommitText) -> Result<MergeMethod> {
        self.inner.merge(number, head, text)
    }
    fn ci_runs(&self, commit: &str) -> Result<Vec<CiRunState>> {
        self.inner.ci_runs(commit)
    }
    fn check_run(&self, commit: &str, name: &str) -> Result<Option<CiRunState>> {
        if commit != "early1" {
            return self.inner.check_run(commit, name);
        }
        self.log.borrow_mut().push(format!("check_run {commit}"));
        let mut script = self.script.borrow_mut();
        let reply = if script.len() > 1 {
            script.remove(0)
        } else {
            script[0].clone()
        };
        let jobs = reply.map_err(|()| anyhow::anyhow!("the host did not answer"))?;
        Ok(jobs.into_iter().find(|(n, _)| n == name).map(|(_, r)| r))
    }
    fn rerun_failed(&self, run_id: u64) -> Result<()> {
        self.inner.rerun_failed(run_id)
    }
    fn failing_test(&self, run_id: u64) -> Result<Option<String>> {
        self.inner.failing_test(run_id)
    }
    fn early_run_wait(&self) -> Duration {
        self.bound
    }
}

fn job(name: &str, id: u64, status: &str, conclusion: Option<&str>) -> (String, CiRunState) {
    (
        name.to_string(),
        CiRunState {
            id,
            status: status.into(),
            conclusion: conclusion.map(String::from),
            attempt: None,
        },
    )
}

/// The early commit's check runs where only the job land waits for is present.
fn run_state(status: &str, conclusion: Option<&str>) -> Vec<(String, CiRunState)> {
    vec![job(EARLY_RUN_JOB, 5, status, conclusion)]
}

/// Like [`run_state`], with the candidate's history job failed, as it always is on a candidate.
fn run_state_with_failed_history(
    status: &str,
    conclusion: Option<&str>,
) -> Vec<(String, CiRunState)> {
    let mut jobs = run_state(status, conclusion);
    jobs.push(job("the full history", 6, "completed", Some("failure")));
    jobs
}

#[test]
fn parse_check_run_picks_the_latest_run_of_that_name() {
    let reply = serde_json::json!({"check_runs": [
        {"id": 3, "name": "fmt, clippy, tests", "status": "completed", "conclusion": "failure"},
        {"id": 9, "name": "the full history", "status": "completed", "conclusion": "failure"},
        {"id": 7, "name": "fmt, clippy, tests", "status": "in_progress", "conclusion": null},
    ]});
    let run = parse_check_run(&reply, EARLY_RUN_JOB).unwrap();
    assert_eq!(run.id, 7);
    assert_eq!(run.status, "in_progress");
    assert_eq!(run.conclusion, None);
    assert!(parse_check_run(&reply, "absent").is_none());
    assert!(parse_check_run(&serde_json::json!({}), EARLY_RUN_JOB).is_none());
}

#[test]
fn land_pr_waits_for_the_named_check_run_when_the_history_job_failed() {
    let rig = Rig::new();
    let host = RunHost::new(
        &rig,
        vec![
            Ok(run_state_with_failed_history("in_progress", None)),
            Ok(run_state_with_failed_history("in_progress", None)),
            Ok(run_state_with_failed_history("completed", Some("success"))),
        ],
    );
    let report = land_over_early(&rig, &FakeHost::new(Checks::Green), &host);
    assert_eq!(
        log_of(&rig),
        [
            "check_run early1",
            "sleep 20",
            "check_run early1",
            "sleep 20",
            "check_run early1",
            "replace"
        ]
    );
    assert!(
        report.warnings[0].contains("waited 40s") && report.warnings[0].contains("completed"),
        "{:?}",
        report.warnings
    );
}

#[test]
fn land_pr_pushes_at_once_when_the_named_check_run_failed() {
    for others in ["in_progress", "completed"] {
        let rig = Rig::new();
        let mut jobs = run_state("completed", Some("failure"));
        jobs.push(job("the full history", 6, others, None));
        jobs.push(job("the pull request head", 8, others, None));
        let host = RunHost::new(&rig, vec![Ok(jobs)]);
        let report = land_over_early(&rig, &FakeHost::new(Checks::Green), &host);
        assert_eq!(log_of(&rig), ["check_run early1", "replace"], "{others}");
        assert!(
            report.warnings[0].contains("fmt, clippy, tests failed"),
            "{:?}",
            report.warnings
        );
    }
}

/// Opens the early pull request at `early1`, then lands `landing1` against `host`.
fn land_over_early(rig: &Rig, early_host: &FakeHost, host: &RunHost) -> PrReport {
    early(rig, early_host, "early1").unwrap();
    rig.log.borrow_mut().clear();
    rig.run_with(host, true, "athena/bootstrap", "landing1", &Anc(true))
        .unwrap()
}

fn log_of(rig: &Rig) -> Vec<String> {
    rig.log.borrow().clone()
}

#[test]
fn land_pr_waits_for_the_early_run_before_pushing() {
    let rig = Rig::new();
    let host = RunHost::new(
        &rig,
        vec![
            Ok(run_state("in_progress", None)),
            Ok(run_state("in_progress", None)),
            Ok(run_state("completed", Some("success"))),
        ],
    );
    let report = land_over_early(&rig, &FakeHost::new(Checks::Green), &host);
    assert_eq!(
        log_of(&rig),
        [
            "check_run early1",
            "sleep 20",
            "check_run early1",
            "sleep 20",
            "check_run early1",
            "replace"
        ]
    );
    assert_eq!(report.link.state, LinkState::Landed);
    assert_eq!(report.warnings.len(), 1);
    assert!(
        report.warnings[0].contains("waited 40s") && report.warnings[0].contains("completed"),
        "{:?}",
        report.warnings
    );
}

#[test]
fn land_pr_pushes_at_once_when_the_early_run_is_completed() {
    let rig = Rig::new();
    let host = RunHost::new(&rig, vec![Ok(run_state("completed", Some("success")))]);
    let report = land_over_early(&rig, &FakeHost::new(Checks::Green), &host);
    assert_eq!(log_of(&rig), ["check_run early1", "replace"]);
    assert!(
        report.warnings[0].contains("waited 0s")
            && report.warnings[0].contains("already completed"),
        "{:?}",
        report.warnings
    );
}

#[test]
fn land_pr_pushes_anyway_when_the_early_run_never_completes() {
    let rig = Rig::new();
    let host = RunHost::new(&rig, vec![Ok(run_state("in_progress", None))]);
    let report = land_over_early(&rig, &FakeHost::new(Checks::Green), &host);
    // A 60 second bound: looks at 0, 20, 40 and 60 seconds, three sleeps, then the push.
    let log = log_of(&rig);
    assert_eq!(
        log.iter().filter(|e| *e == "sleep 20").count(),
        3,
        "{log:?}"
    );
    assert_eq!(log.last().map(String::as_str), Some("replace"));
    assert!(report.warnings[0].contains("waited 60s") && report.warnings[0].contains("timed out"));
    assert_eq!(report.link.state, LinkState::Landed);
    assert_eq!(
        *host.inner.merge_heads.borrow(),
        vec!["landing1".to_string()]
    );
    // The default bound a real host uses.
    assert_eq!(
        FakeHost::new(Checks::Green).early_run_wait(),
        Duration::from_secs(600)
    );
}

#[test]
fn land_pr_early_run_failure_changes_nothing() {
    let cases: Vec<(&str, Look)> = vec![
        ("failed", Ok(run_state("completed", Some("failure")))),
        ("failed", Ok(run_state("completed", Some("timed_out")))),
        ("no \"fmt, clippy, tests\" check run", Ok(vec![])),
        ("could not be read", Err(())),
    ];
    // The baseline: the same landing where the early commit shows a plain success.
    let baseline = {
        let rig = Rig::new();
        let host = RunHost::new(&rig, vec![Ok(run_state("completed", Some("success")))]);
        land_over_early(&rig, &FakeHost::new(Checks::Green), &host)
    };
    for (why, reply) in cases {
        let rig = Rig::new();
        let host = RunHost::new(&rig, vec![reply]);
        let report = land_over_early(&rig, &FakeHost::new(Checks::Green), &host);
        assert_eq!(log_of(&rig), ["check_run early1", "replace"], "{why}");
        assert!(report.warnings[0].contains(why), "{:?}", report.warnings);
        assert_eq!(
            *host.inner.merge_heads.borrow(),
            vec!["landing1".to_string()]
        );
        assert_eq!(report.merge_requested, baseline.merge_requested);
        assert_eq!(report.link, baseline.link);
        assert_eq!(report.checks, baseline.checks);
        assert_eq!(report.reruns, baseline.reruns);
    }
}

#[test]
fn land_pr_reports_why_the_early_wait_stopped_only_when_there_is_an_early_head() {
    // No early head: the landing commit is pushed as before, with no look at any run and no note.
    let rig = Rig::new();
    let host = RunHost::new(&rig, vec![Ok(run_state("in_progress", None))]);
    let report = rig
        .run_with(&host, true, "athena/bootstrap", "landing1", &Anc(true))
        .unwrap();
    assert!(log_of(&rig).is_empty(), "{:?}", log_of(&rig));
    assert!(report.warnings.is_empty());
    // The early head is the landing commit: nothing is replaced, so nothing is waited for.
    let rig = Rig::new();
    let host = RunHost::new(&rig, vec![Ok(run_state("in_progress", None))]);
    early(&rig, &FakeHost::new(Checks::Green), "early1").unwrap();
    let report = rig
        .run_with(&host, true, "athena/bootstrap", "early1", &Anc(true))
        .unwrap();
    assert!(log_of(&rig).is_empty(), "{:?}", log_of(&rig));
    assert!(report.warnings.is_empty());
}

#[test]
fn land_pr_guards_the_merge_with_the_current_head_and_pushes_nothing_over_a_newer_one() {
    // Head equals the landing commit: the guard is that head.
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    rig.run(&host, true, "athena/bootstrap").unwrap();
    assert_eq!(*host.merge_heads.borrow(), vec!["cand1".to_string()]);

    // A newer head: no merge, and no push of any kind after the first publish.
    let (rig, host) = pushed_fix();
    assert!(rig.run(&host, true, "athena/bootstrap").is_err());
    assert!(host.merge_heads.borrow().is_empty());
    assert_eq!(
        *rig.remote.pushes.borrow(),
        vec![("athena/bootstrap".to_string(), "cand1".to_string())]
    );
    assert_eq!(rig.remote.head.borrow().as_deref(), Some("fix1"));
    assert_eq!(rig.stored().head, "fix1");
}

#[test]
fn land_pr_refuses_any_other_head_naming_each_extra_commit_and_why() {
    let (rig, host) = pushed_fix();
    *rig.remote.extras.borrow_mut() = vec![
        ("a".repeat(40), "Fix the failing test".into()),
        ("b".repeat(40), "Declared intervention: hand edit".into()),
    ];
    let err = rig
        .run(&host, true, "athena/bootstrap")
        .unwrap_err()
        .to_string();
    assert!(err.contains(&format!("{} Fix the failing test", "a".repeat(12))));
    assert!(err.contains(&format!("{} Declared intervention", "b".repeat(12))));
    assert!(err.contains("records-history build (D111)"), "{err}");
    assert!(err.contains("declared intervention too"), "{err}");
    assert!(err.contains("squash"), "{err}");
    assert_eq!(host.merges.get(), 0);

    // A head the remote cannot list commits for is still refused, and named.
    *rig.remote.extras.borrow_mut() = vec![];
    let err = rig
        .run(&host, true, "athena/bootstrap")
        .unwrap_err()
        .to_string();
    assert!(err.contains("Refused head: fix1"), "{err}");
    assert_eq!(host.merges.get(), 0);
}

#[test]
fn a_missing_remote_branch_is_an_error_and_nothing_is_merged() {
    let (rig, host) = pushed_fix();
    *rig.remote.head.borrow_mut() = None;
    let err = rig
        .run(&host, true, "athena/bootstrap")
        .unwrap_err()
        .to_string();
    assert!(err.contains("not on the remote"), "{err}");
    assert_eq!(host.merges.get(), 0);
}

#[test]
fn git_remote_lists_the_commits_after_the_landing_commit() {
    let bare_dir = tempfile::tempdir().unwrap();
    let bare = Git::isolated(bare_dir.path());
    bare.run(["init", "-q", "--bare", "-b", "main"]).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let git = Git::isolated(dir.path());
    git.run(["init", "-q", "-b", "main"]).unwrap();
    let step = |text: &str| {
        std::fs::write(dir.path().join("a.txt"), text).unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", text]).unwrap();
        git.head().unwrap().unwrap()
    };
    let landed = step("landed");
    let x = step("extra one");
    let y = step("extra two");
    git.run(["remote", "add", "origin", bare_dir.path().to_str().unwrap()])
        .unwrap();
    git.run(["push", "-q", "origin", &format!("{y}:refs/heads/work")])
        .unwrap();
    let remote = GitRemote {
        git: &git,
        name: "origin",
    };
    assert_eq!(
        remote.commits_after(&landed, &y).unwrap(),
        vec![
            (x.clone(), "extra one".to_string()),
            (y.clone(), "extra two".to_string())
        ]
    );
    assert!(remote.commits_after(&y, &landed).unwrap().is_empty());
    assert!(remote.commits_after(&y, &y).unwrap().is_empty());
    // Nothing here can replace the remote branch.
    assert_eq!(remote.remote_head("work").unwrap(), Some(y));
}

#[test]
fn git_remote_deletes_a_pushed_branch_and_a_remote_without_delete_refuses() {
    let bare_dir = tempfile::tempdir().unwrap();
    let bare = Git::isolated(bare_dir.path());
    bare.run(["init", "-q", "--bare", "-b", "main"]).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let git = Git::isolated(dir.path());
    git.run(["init", "-q", "-b", "main"]).unwrap();
    std::fs::write(dir.path().join("a.txt"), "x").unwrap();
    git.run(["add", "-A"]).unwrap();
    git.run(["commit", "-q", "-m", "x"]).unwrap();
    let commit = git.head().unwrap().unwrap();
    git.run(["remote", "add", "origin", bare_dir.path().to_str().unwrap()])
        .unwrap();
    let remote = GitRemote {
        git: &git,
        name: "origin",
    };
    let branch = ci_branch(&JobId::parse("20261031-230906-3rze").unwrap());
    assert_eq!(branch, "athena/ci/20261031-230906-3rze");
    remote.push(&branch, &commit).unwrap();
    // A neighbouring branch must survive the delete.
    remote.push("athena/ci/other", &commit).unwrap();
    assert_eq!(remote.remote_head(&branch).unwrap(), Some(commit.clone()));

    remote.delete_branch(&branch).unwrap();
    assert_eq!(remote.remote_head(&branch).unwrap(), None);
    assert_eq!(
        remote.remote_head("athena/ci/other").unwrap(),
        Some(commit.clone())
    );
    // A remote that does not implement the delete refuses rather than doing nothing.
    let refusing = FakeRemote::default();
    let err = refusing.delete_branch(&branch).unwrap_err().to_string();
    assert!(err.contains("cannot delete"), "{err}");
}

/// End to end over real git: the branch holds the landing commit plus a declared-intervention commit.
/// Land refuses naming that commit, requests no merge, and leaves the remote branch untouched.
#[test]
fn land_pr_refuses_a_real_branch_with_a_declared_intervention_after_the_landing() {
    let bare_dir = tempfile::tempdir().unwrap();
    let bare = Git::isolated(bare_dir.path());
    bare.run(["init", "-q", "--bare", "-b", "main"]).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let git = Git::isolated(dir.path());
    git.run(["init", "-q", "-b", "main"]).unwrap();
    let step = |text: &str| {
        std::fs::write(dir.path().join("a.txt"), text).unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", text]).unwrap();
        git.head().unwrap().unwrap()
    };
    let landed = step("landed");
    let branch = "athena/bootstrap";
    git.run(["remote", "add", "origin", bare_dir.path().to_str().unwrap()])
        .unwrap();
    let remote = GitRemote {
        git: &git,
        name: "origin",
    };
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    host.per_commit
        .borrow_mut()
        .insert(landed.clone(), Checks::Red);
    let run = |commit: &str| {
        let state = StateDir::new(rig.dir.path().to_path_buf());
        let nap = |_: Duration| {};
        land_pr(&LandPr {
            state: &state,
            job: &rig.job,
            branch,
            commit,
            verified_fingerprint: "verified",
            pr: PrDraft {
                title: "t",
                body: "b",
                head: branch,
            },
            squash: &text(),
            host_name: HOST,
            repository: "example/project",
            merge_via_rest: true,
            poll: Poll {
                attempts: 1,
                delay: Duration::ZERO,
                sleep: &nap,
            },
            ancestry: &git,
            remote: &remote,
            host: &host,
            content: &rig.content,
            records: None,
        })
    };
    // Publish the landing commit; it is red, so it is not merged.
    let first = run(&landed).unwrap();
    assert!(!first.merge_requested);
    let fix = step("fix the failure");
    let intervention = step("Declared intervention: edit by hand");
    git.run([
        "push",
        "-q",
        "origin",
        &format!("{intervention}:refs/heads/{branch}"),
    ])
    .unwrap();
    let err = run(&landed).unwrap_err().to_string();
    assert!(
        err.contains(&format!("{} fix the failure", &fix[..12])),
        "{err}"
    );
    assert!(
        err.contains(&format!(
            "{} Declared intervention: edit by hand",
            &intervention[..12]
        )),
        "{err}"
    );
    assert!(err.contains("records-history build (D111)"), "{err}");
    assert!(err.contains("are green"), "{err}");
    assert_eq!(host.merges.get(), 0);
    let held = bare.rev_parse(&format!("refs/heads/{branch}")).unwrap();
    assert_eq!(held.as_deref(), Some(intervention.as_str()));
}

#[test]
fn a_push_the_guard_refuses_opens_nothing() {
    // Another branch.
    let rig = Rig::new();
    let host = FakeHost::new(Checks::Green);
    assert!(rig.run(&host, true, "main").is_err());
    assert!(rig.remote.pushes.borrow().is_empty());
    assert_eq!(host.opened.get(), 0);

    // A remote branch that is not an ancestor of the candidate.
    let rig = Rig::new();
    *rig.remote.head.borrow_mut() = Some("rewritten".into());
    let state = StateDir::new(rig.dir.path().to_path_buf());
    let nap = |_: Duration| {};
    let host = FakeHost::new(Checks::Green);
    let refused = land_pr(&LandPr {
        state: &state,
        job: &rig.job,
        branch: "athena/bootstrap",
        commit: "cand1",
        verified_fingerprint: "verified",
        pr: PrDraft {
            title: "t",
            body: "b",
            head: "athena/bootstrap",
        },
        squash: &text(),
        host_name: HOST,
        repository: "example/project",
        merge_via_rest: true,
        poll: Poll {
            attempts: 1,
            delay: Duration::ZERO,
            sleep: &nap,
        },
        ancestry: &Anc(false),
        remote: &rig.remote,
        host: &host,
        content: &rig.content,
        records: None,
    });
    assert!(refused.is_err());
    assert!(rig.remote.pushes.borrow().is_empty());
    assert_eq!(host.opened.get(), 0);
    assert_eq!(load_link(&state, &rig.job).unwrap(), None);
}

// ---- refused host calls: the status, the record and the bounded retry ----

#[test]
fn the_status_is_read_from_the_text_gh_prints() {
    // The form `gh api` prints for a refused call.
    assert_eq!(parse_status("gh: Method Not Allowed (HTTP 405)"), Some(405));
    assert_eq!(parse_status("gh: Server Error (HTTP 500)\n"), Some(500));
    assert_eq!(
        parse_status("gh: Merge failed (HTTP 405) after HTTP 500"),
        Some(405)
    );
    assert_eq!(parse_status("gh: connection refused"), None);
    assert_eq!(parse_status("HTTP 4050"), None);
    assert_eq!(parse_status("HTTP 999"), None);
    assert_eq!(parse_status("HTTP 40"), None);
    assert_eq!(parse_status(""), None);
    for status in [500, 502, 503, 504] {
        assert!(is_transient(Some(status)));
    }
    for status in [400, 404, 405, 422, 501] {
        assert!(!is_transient(Some(status)));
    }
    assert!(!is_transient(None));
}

/// A host that fails the merge request with the given statuses, one per call, then (unless `always` is
/// set) lets it through.
struct Refusing {
    failures: RefCell<std::collections::VecDeque<Option<u16>>>,
    always: Option<Option<u16>>,
    merged: Cell<bool>,
    merge_calls: Cell<u32>,
    other_calls: Cell<u32>,
}
impl Refusing {
    fn new(failures: &[Option<u16>], always: Option<Option<u16>>) -> Self {
        Refusing {
            failures: RefCell::new(failures.iter().copied().collect()),
            always,
            merged: Cell::new(false),
            merge_calls: Cell::new(0),
            other_calls: Cell::new(0),
        }
    }
}
impl Runner for Refusing {
    fn gh(&self, args: &[String]) -> Result<String> {
        let path = args.iter().find(|a| a.starts_with("repos/")).unwrap();
        if !path.ends_with("/merge") {
            self.other_calls.set(self.other_calls.get() + 1);
            return Ok(if path == "repos/example/project" {
                ALL_METHODS.to_string()
            } else if path.ends_with("/pulls") {
                r#"{"number": 7}"#.to_string()
            } else if path.contains("/pulls/") {
                if self.merged.get() {
                    r#"{"merged": true, "merge_commit_sha": "merged1"}"#.to_string()
                } else {
                    r#"{"merged": false}"#.to_string()
                }
            } else if path.contains("check-runs") {
                REQUIRED_GREEN.to_string()
            } else {
                r#"{"state": "success", "statuses": []}"#.to_string()
            });
        }
        self.merge_calls.set(self.merge_calls.get() + 1);
        let refusal = match self.always {
            Some(status) => Some(status),
            None => self.failures.borrow_mut().pop_front(),
        };
        match refusal {
            Some(status_code) => Err(GhFailure {
                call: "api".into(),
                status_code,
                text: "refused".into(),
            }
            .into()),
            None => {
                self.merged.set(true);
                Ok("{}".to_string())
            }
        }
    }
}

/// Lands over REST through a client on `runner`, with a log of its refusals.
fn land_through(runner: &Refusing, rig: &Rig) -> (Result<PrReport>, Vec<HostError>) {
    let log = HostErrorLog::new(rig.job.clone(), WorkPhase::Landing);
    let client = GhClient {
        log: Some(&log),
        retry: Retry {
            attempts: 3,
            delay: Duration::ZERO,
        },
        ..GhClient::new(runner, "example/project".into())
    };
    let result = rig.run_with(&client, true, "athena/bootstrap", "cand1", &Anc(true));
    (result, log.entries())
}

/// A host whose check-runs reply is given, so the production classifier judges it. The pull request is
/// open until the merge request arrives, then merged as `merged1`.
struct Reporting {
    check_runs: Value,
    merged: Cell<bool>,
    merge_calls: Cell<u32>,
}
impl Reporting {
    fn new(check_runs: Value) -> Self {
        Reporting {
            check_runs,
            merged: Cell::new(false),
            merge_calls: Cell::new(0),
        }
    }
}
impl Runner for Reporting {
    fn gh(&self, args: &[String]) -> Result<String> {
        let path = args.iter().find(|a| a.starts_with("repos/")).unwrap();
        if path.ends_with("/merge") {
            self.merge_calls.set(self.merge_calls.get() + 1);
            self.merged.set(true);
            return Ok("{}".to_string());
        }
        Ok(if path == "repos/example/project" {
            ALL_METHODS.to_string()
        } else if path.ends_with("/pulls") {
            r#"{"number": 7}"#.to_string()
        } else if path.contains("/pulls/") {
            if self.merged.get() {
                r#"{"merged": true, "merge_commit_sha": "merged1"}"#.to_string()
            } else {
                r#"{"merged": false}"#.to_string()
            }
        } else if path.contains("check-runs") {
            self.check_runs.to_string()
        } else if path.ends_with("/status") {
            r#"{"state": "failure", "statuses": []}"#.to_string()
        } else {
            "{}".to_string()
        })
    }
}

/// The check-runs reply with the required check in the given state and the pull request head check failed.
fn runs_with(status: &str, conclusion: Value, optional: &str) -> Value {
    json!({"check_runs": [
        {"id": 1, "name": super::client::CI_AGGREGATE_JOB, "status": status, "conclusion": conclusion},
        {"id": 2, "name": "pull request head", "status": "completed", "conclusion": optional},
    ]})
}

/// Lands through a real client on `runner`.
fn land_on(runner: &Reporting, merge_via_rest: bool) -> PrReport {
    let rig = Rig::new();
    let client = GhClient::new(runner, "example/project".into());
    rig.run_with(
        &client,
        merge_via_rest,
        "athena/bootstrap",
        "cand1",
        &Anc(true),
    )
    .expect("a red optional check is not an error")
}

#[test]
fn a_red_optional_check_does_not_fail_the_landing() {
    let runner = Reporting::new(runs_with("completed", json!("success"), "failure"));
    let report = land_on(&runner, true);
    // The required check is green, so the pull request merges and lands; nothing is red.
    assert_eq!(report.checks, Some(Checks::Green));
    assert_eq!(report.reruns, 0);
    assert!(report.merge_requested);
    assert_eq!(runner.merge_calls.get(), 1);
    assert_eq!(report.link.state, LinkState::Landed);
    assert_eq!(report.link.merged_commit.as_deref(), Some("merged1"));
    // The failed optional check is a note, not the "checks Red" failure.
    let notes: Vec<_> = report
        .warnings
        .iter()
        .filter(|w| w.contains("pull request head"))
        .collect();
    assert_eq!(notes.len(), 1, "{:?}", report.warnings);
    assert!(notes[0].contains("main does not require it"), "{notes:?}");
    assert!(notes[0].contains("not a failure"), "{notes:?}");
    for text in &report.warnings {
        assert!(
            !text.contains("checks Red") && !text.contains("not landed"),
            "{text}"
        );
    }
}

#[test]
fn with_the_setting_off_a_red_optional_check_leaves_the_pull_request_open_and_green() {
    let runner = Reporting::new(runs_with("completed", json!("success"), "failure"));
    let report = land_on(&runner, false);
    assert_eq!(report.checks, Some(Checks::Green));
    assert_eq!(report.link.state, LinkState::Open);
    assert_eq!(runner.merge_calls.get(), 0);
    assert_eq!(report.reruns, 0);
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.contains("pull request head"))
    );
}

#[test]
fn a_required_check_that_failed_is_still_red_and_nothing_merges() {
    let runner = Reporting::new(runs_with("completed", json!("failure"), "success"));
    let report = land_on(&runner, true);
    assert_eq!(report.checks, Some(Checks::Red));
    assert!(!report.merge_requested);
    assert_eq!(runner.merge_calls.get(), 0);
    assert_eq!(report.link.state, LinkState::Open);
    // The optional check passed, so there is nothing to note.
    assert!(
        !report.warnings.iter().any(|w| w.contains("not required")),
        "{:?}",
        report.warnings
    );
}

#[test]
fn a_required_check_that_is_pending_or_absent_stays_waiting_and_nothing_merges() {
    let pending = runs_with("in_progress", Value::Null, "success");
    let absent = json!({"check_runs": [
        {"id": 2, "name": "pull request head", "status": "completed", "conclusion": "success"}]});
    for runs in [pending, absent] {
        let runner = Reporting::new(runs);
        let report = land_on(&runner, true);
        assert_eq!(report.checks, Some(Checks::Pending));
        assert_eq!(report.reruns, 0);
        assert!(!report.merge_requested);
        assert_eq!(runner.merge_calls.get(), 0);
        assert_eq!(report.link.state, LinkState::Open);
    }
}

#[test]
fn a_500_twice_then_success_lands_with_two_host_errors_and_no_error_for_the_user() {
    let runner = Refusing::new(&[Some(500), Some(500)], None);
    let rig = Rig::new();
    let (result, errors) = land_through(&runner, &rig);
    let report = result.expect("a transient failure is retried, not shown");
    assert_eq!(report.link.state, LinkState::Landed);
    assert!(report.merge_requested);
    assert_eq!(runner.merge_calls.get(), 3);
    assert_eq!(errors.len(), 2);
    for error in &errors {
        assert_eq!(error.status_code, Some(500));
        assert_eq!(error.operation, HostOperation::Merge);
        assert_eq!(error.phase, WorkPhase::Landing);
        assert_eq!(error.job, rig.job);
        assert_eq!(error.message_class, MessageClass::ServerError);
    }
    assert_ne!(errors[0].id, errors[1].id);
}

#[test]
fn a_persistent_500_stops_at_the_bound_and_the_error_reaches_the_user() {
    let runner = Refusing::new(&[], Some(Some(500)));
    let rig = Rig::new();
    let (result, errors) = land_through(&runner, &rig);
    let error = result.unwrap_err();
    let failure = error.downcast_ref::<GhFailure>().expect("the host's error");
    assert_eq!(failure.status_code, Some(500));
    assert_eq!(runner.merge_calls.get(), 3, "three calls in all, then stop");
    assert_eq!(errors.len(), 3);
    // The pull request was opened and stays open: a rerun finds it by its number.
    assert_eq!(rig.stored().state, LinkState::Open);
}

#[test]
fn a_refusal_that_is_not_transient_is_recorded_once_and_not_retried() {
    for status in [Some(405), Some(404), Some(422), Some(403), Some(501), None] {
        let runner = Refusing::new(&[], Some(status));
        let rig = Rig::new();
        let (result, errors) = land_through(&runner, &rig);
        assert!(result.is_err(), "{status:?}");
        assert_eq!(runner.merge_calls.get(), 1, "{status:?}");
        assert_eq!(errors.len(), 1, "{status:?}");
        assert_eq!(errors[0].status_code, status);
        assert_eq!(errors[0].operation, HostOperation::Merge);
        assert_eq!(errors[0].message_class, MessageClass::of_status(status));
    }
}

#[test]
fn a_failure_that_is_not_a_host_refusal_is_not_recorded() {
    struct Missing;
    impl Runner for Missing {
        fn gh(&self, _: &[String]) -> Result<String> {
            bail!("running gh; is the GitHub client installed?")
        }
    }
    let log = HostErrorLog::new(JobId::parse("bootstrap").unwrap(), WorkPhase::Landing);
    let client = GhClient {
        log: Some(&log),
        ..GhClient::new(&Missing, "example/project".into())
    };
    assert!(client.default_branch().is_err());
    assert!(log.entries().is_empty());
}

fn failure(status_code: Option<u16>, text: &str) -> GhFailure {
    GhFailure {
        call: "repos/example/project".into(),
        status_code,
        text: text.into(),
    }
}

#[test]
fn timeouts_and_connection_resets_are_transient_and_auth_and_4xx_are_not() {
    let reason = |f: GhFailure| f.transient_reason();
    assert_eq!(reason(failure(Some(503), "")).as_deref(), Some("HTTP 503"));
    assert_eq!(
        reason(failure(None, "Get \"https://api.github.com\": i/o timeout")).as_deref(),
        Some("timeout")
    );
    assert_eq!(
        reason(failure(None, "read tcp: connection reset by peer")).as_deref(),
        Some("connection reset")
    );
    for (status, text) in [
        (Some(404), "Not Found"),
        (Some(401), "Bad credentials"),
        (Some(403), "Forbidden"),
        // A status decides on its own, whatever the words.
        (Some(404), "request timeout"),
        (
            None,
            "You are not logged into any GitHub hosts. Run gh auth login",
        ),
        (None, "gh: unknown failure"),
    ] {
        assert_eq!(reason(failure(status, text)), None, "{status:?} {text}");
    }
}

#[test]
fn the_wait_doubles_with_each_failure_and_jitter_adds_up_to_half() {
    let retry = Retry {
        attempts: 3,
        delay: Duration::from_secs(2),
    };
    assert_eq!(retry.wait(1, 0), Duration::from_secs(2));
    assert_eq!(retry.wait(2, 0), Duration::from_secs(4));
    assert_eq!(retry.wait(2, 999), Duration::from_millis(4000 + 1998));
    assert_eq!(retry.wait(1, 1000), retry.wait(1, 0));
    let none = Retry {
        attempts: 3,
        delay: Duration::ZERO,
    };
    assert_eq!(none.wait(3, 500), Duration::ZERO);
}

/// Fails each call with the given failures in turn, then answers with an empty object.
struct Scripted {
    failures: RefCell<std::collections::VecDeque<GhFailure>>,
    calls: Cell<u32>,
}
impl Runner for Scripted {
    fn gh(&self, _: &[String]) -> Result<String> {
        self.calls.set(self.calls.get() + 1);
        match self.failures.borrow_mut().pop_front() {
            Some(f) => Err(f.into()),
            None => Ok(r#"{"default_branch": "trunk"}"#.to_string()),
        }
    }
}

struct Lines(RefCell<Vec<String>>);
impl Ui for Lines {
    fn line(&self, text: &str) {
        self.0.borrow_mut().push(text.to_string());
    }
}

/// Calls `default_branch` against a runner that fails with `failures`; returns the outcome, the number of
/// calls and the retry lines.
fn scripted(failures: Vec<GhFailure>) -> (Result<String>, u32, Vec<String>) {
    let runner = Scripted {
        failures: RefCell::new(failures.into()),
        calls: Cell::new(0),
    };
    let lines = Lines(RefCell::new(vec![]));
    let client = GhClient {
        retry: Retry {
            attempts: 3,
            delay: Duration::ZERO,
        },
        ui: Some(&lines),
        ..GhClient::new(&runner, "example/project".into())
    };
    let result = client.default_branch();
    (result, runner.calls.get(), lines.0.into_inner())
}

#[test]
fn timeouts_and_resets_are_retried_and_404_and_auth_failures_are_not() {
    for text in ["dial tcp: i/o timeout", "read: connection reset by peer"] {
        let (result, calls, lines) = scripted(vec![failure(None, text)]);
        assert_eq!(result.unwrap(), "trunk", "{text}");
        assert_eq!(calls, 2, "{text}");
        assert_eq!(lines.len(), 1);
    }
    for (status, text) in [
        (Some(404), "Not Found"),
        (Some(401), "Bad credentials"),
        (Some(403), "Forbidden"),
        (None, "You are not logged into any GitHub hosts"),
    ] {
        let (result, calls, lines) = scripted(vec![failure(status, text); 3]);
        assert!(result.is_err());
        assert_eq!(calls, 1, "{status:?} {text}");
        assert!(lines.is_empty());
    }
}

#[test]
fn each_retry_is_logged_with_the_command_status_and_attempt_and_the_last_error_names_them() {
    let (result, calls, lines) = scripted(vec![failure(Some(500), "boom"); 3]);
    assert_eq!(calls, 3);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("repos/example/project"), "{lines:?}");
    assert!(lines[0].contains("HTTP 500"), "{lines:?}");
    assert!(lines[0].contains("attempt 1 of 3"), "{lines:?}");
    assert!(lines[1].contains("attempt 2 of 3"), "{lines:?}");
    let error = result.unwrap_err();
    let message = error.to_string();
    assert!(message.contains("repos/example/project"), "{message}");
    assert!(message.contains("HTTP 500"), "{message}");
    assert!(message.contains("3 attempts"), "{message}");
    assert!(error.downcast_ref::<GhFailure>().is_some());
}

#[test]
fn the_stored_record_has_no_text_of_the_host() {
    let runner = Refusing::new(&[], Some(Some(405)));
    let rig = Rig::new();
    let (_, errors) = land_through(&runner, &rig);
    let json = athena_data::records::AnyRecord::HostError(errors[0].clone()).to_json();
    assert!(!json.contains("refused"), "{json}");
    assert!(json.contains("\"status_code\": 405"), "{json}");
}

/// `land_pr` with the real records sink, a real repository, a bare remote and a fake host
/// (`docs/design/records-history.md` §5.2, §5.3, §6.1, §8).
mod records_flow {
    use super::*;
    use athena_git::records_branch::RECORDS_REF;
    use std::rc::Rc;

    const JOB: &str = "20261024-115923-02gf";
    /// The commit the fake host makes: a valid sha, as the landing record requires.
    const HOST_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

    struct World {
        _bare_dir: tempfile::TempDir,
        bare: Git,
        _work_dir: tempfile::TempDir,
        git: Git,
        state_dir: tempfile::TempDir,
        /// The remote main's tip when the pull request was opened.
        base: String,
        head: String,
        remote: FakeRemote,
        content: FakeContent,
        job: JobId,
    }

    impl World {
        fn new() -> Self {
            let bare_dir = tempfile::tempdir().unwrap();
            let bare = Git::isolated(bare_dir.path());
            // The fake host's default branch is `trunk`.
            bare.run(["init", "-q", "--bare", "-b", "trunk"]).unwrap();
            let work_dir = tempfile::tempdir().unwrap();
            let git = Git::isolated(work_dir.path());
            git.run(["init", "-q", "-b", "trunk"]).unwrap();
            let url = bare_dir.path().to_str().unwrap().to_string();
            git.run(["remote", "add", "origin", &url]).unwrap();
            std::fs::write(work_dir.path().join("a.txt"), "a").unwrap();
            git.run(["add", "-A"]).unwrap();
            git.run(["commit", "-q", "-m", "init"]).unwrap();
            git.run(["push", "-q", "origin", "trunk"]).unwrap();
            let base = git.head().unwrap().unwrap();
            git.run(["checkout", "-q", "-b", &format!("athena/{JOB}")])
                .unwrap();
            let mut world = World {
                _bare_dir: bare_dir,
                bare,
                _work_dir: work_dir,
                git,
                state_dir: tempfile::tempdir().unwrap(),
                base,
                head: String::new(),
                remote: FakeRemote::default(),
                content: FakeContent(BTreeMap::from([(HOST_COMMIT, "verified")])),
                job: JobId::parse(JOB).unwrap(),
            };
            // The pull request's one commit lands the job: its work, with its candidate, verification
            // and landed outcome.
            let mut files = crate::land::fixtures::landed(&world.job);
            files.push(crate::land::fixtures::plain("b.txt", "b.txt"));
            world.add_files(&files, "Work");
            world
        }

        /// A commit on the work branch, pushed, so the pull request's head moves to it.
        fn add_commit(&mut self, file: &str, message: &str) -> String {
            self.add_files(&[crate::land::fixtures::plain(file, file)], message)
        }

        /// A commit of `files` on the work branch, pushed, so the pull request's head moves to it.
        fn add_files(&mut self, files: &[crate::land::fixtures::File], message: &str) -> String {
            self.head = crate::land::fixtures::commit(&self.git, files, message);
            self.git
                .run(["push", "-q", "-f", "origin", &format!("athena/{JOB}")])
                .unwrap();
            *self.remote.head.borrow_mut() = Some(self.head.clone());
            self.head.clone()
        }

        fn host(&self) -> FakeHost {
            let mut host = FakeHost::new(Checks::Green);
            host.merge_makes = HOST_COMMIT;
            host
        }

        fn sink(&self) -> GitLandingRecords<'_> {
            GitLandingRecords::for_remote(&self.git, "origin").unwrap()
        }

        fn run(&self, host: &FakeHost, records: Option<&dyn LandingRecords>) -> Result<PrReport> {
            let state = StateDir::new(self.state_dir.path().to_path_buf());
            let nap = |_: Duration| {};
            land_pr(&LandPr {
                state: &state,
                job: &self.job,
                branch: &format!("athena/{JOB}"),
                commit: &self.head,
                verified_fingerprint: "verified",
                pr: PrDraft {
                    title: "t",
                    body: "b",
                    head: "athena/x",
                },
                squash: &text(),
                host_name: HOST,
                repository: "example/project",
                merge_via_rest: true,
                poll: Poll {
                    attempts: 1,
                    delay: Duration::ZERO,
                    sleep: &nap,
                },
                ancestry: &Anc(true),
                remote: &self.remote,
                host,
                content: &self.content,
                records,
            })
        }

        fn remote_tip(&self) -> Option<String> {
            self.bare.rev_parse(RECORDS_REF).unwrap()
        }

        /// The records on the remote branch, oldest commit first, with the commit that added each.
        fn records(&self) -> Vec<(String, HistoryRecord)> {
            let Some(tip) = self.remote_tip() else {
                return Vec::new();
            };
            let mut out = Vec::new();
            for commit in self
                .bare
                .run(["rev-list", "--reverse", "--first-parent", &tip])
                .unwrap()
                .lines()
            {
                for path in self.bare.added_paths(commit).unwrap() {
                    // Other writers' files (the rival's, in one test) are not landing records.
                    if !path.starts_with("landing") {
                        continue;
                    }
                    let bytes = self.bare.show_file(commit, &path).unwrap().unwrap();
                    let record = HistoryRecord::parse(&String::from_utf8(bytes).unwrap());
                    out.push((commit.to_string(), record.unwrap()));
                }
            }
            out
        }
    }

    fn intents(records: &[(String, HistoryRecord)]) -> Vec<(String, LandingIntent)> {
        records
            .iter()
            .filter_map(|(c, r)| match r {
                HistoryRecord::LandingIntent(i) => Some((c.clone(), i.clone())),
                _ => None,
            })
            .collect()
    }

    fn landings(records: &[(String, HistoryRecord)]) -> Vec<(String, Landing)> {
        records
            .iter()
            .filter_map(|(c, r)| match r {
                HistoryRecord::Landing(l) => Some((c.clone(), l.clone())),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn land_pr_writes_the_intent_then_the_landing_with_the_head_as_second_parent() {
        let world = World::new();
        let host = world.host();
        let sink = world.sink();
        let report = world.run(&host, Some(&sink)).unwrap();
        assert_eq!(report.link.state, LinkState::Landed);
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);

        let records = world.records();
        assert_eq!(records.len(), 2, "one intent, then one landing");
        let (intent_commit, intent) = &intents(&records)[0];
        let (landing_commit, landing) = &landings(&records)[0];
        assert!(matches!(records[0].1, HistoryRecord::LandingIntent(_)));
        assert!(matches!(records[1].1, HistoryRecord::Landing(_)));

        let job = JobId::parse(JOB).unwrap();
        assert_eq!(intent.pr_number, 42);
        assert_eq!(intent.head_sha, world.head);
        assert_eq!(
            intent.head_tree_sha,
            world.git.tree_of(&world.head).unwrap()
        );
        assert_eq!(intent.expected_base, world.base);
        assert_eq!(intent.job_ids, vec![job.clone()]);
        intent.validate().unwrap();

        assert_eq!(landing.pr_number, 42);
        assert_eq!(landing.head_sha, world.head);
        assert_eq!(landing.host_commit, HOST_COMMIT);
        assert_eq!(landing.merge_method, history::MergeMethod::Merge);
        assert_eq!(landing.intent, intent.id);
        assert_eq!(
            landing.commits,
            vec![LandingCommit {
                sha: world.head.clone(),
                tree: world.git.tree_of(&world.head).unwrap(),
                kind: LandingKind::JobLanding,
                job: Some(job),
                verification: Some(RecordId::parse(JOB).unwrap()),
            }]
        );
        landing.validate().unwrap();

        // The intent commit is the branch's root; the landing commit's parents are the intent commit and
        // the pull request's head, which keeps the head reachable (§6.1).
        assert!(world.bare.parents(intent_commit).unwrap().is_empty());
        assert_eq!(
            world.bare.parents(landing_commit).unwrap(),
            vec![intent_commit.clone(), world.head.clone()]
        );
        // The local branch holds what the remote holds.
        assert_eq!(
            world.git.rev_parse(RECORDS_REF).unwrap(),
            world.remote_tip()
        );
        assert_eq!(world.remote_tip().as_deref(), Some(landing_commit.as_str()));
    }

    #[test]
    fn the_intent_is_on_the_remote_before_the_merge_and_the_landing_after() {
        let world = World::new();
        let host = world.host();
        let seen: Rc<RefCell<Vec<(usize, usize)>>> = Rc::new(RefCell::new(Vec::new()));
        let bare = world.bare.clone();
        let at_merge = Rc::clone(&seen);
        *host.on_merge.borrow_mut() = Some(Box::new(move || {
            let tip = bare.rev_parse(RECORDS_REF).unwrap().unwrap();
            let count = |dir: &str| bare.ls_tree_under(&tip, dir).unwrap().len();
            at_merge
                .borrow_mut()
                .push((count("landing-intents"), count("landings")));
        }));
        let sink = world.sink();
        world.run(&host, Some(&sink)).unwrap();
        assert_eq!(
            *seen.borrow(),
            vec![(1, 0)],
            "at merge time: intent, no landing"
        );
        // After land_pr returns, the bare remote itself holds the intent and the landing.
        let tip = world.bare.rev_parse(RECORDS_REF).unwrap().unwrap();
        assert_eq!(
            world
                .bare
                .ls_tree_under(&tip, "landing-intents")
                .unwrap()
                .len(),
            1
        );
        assert_eq!(world.bare.ls_tree_under(&tip, "landings").unwrap().len(), 1);
        let records = world.records();
        assert_eq!((intents(&records).len(), landings(&records).len()), (1, 1));
    }

    #[test]
    fn a_refused_merge_leaves_the_intent_on_both_branches_and_no_landing() {
        let world = World::new();
        let host = world.host();
        host.merge_refused.set(true);
        let sink = world.sink();
        let err = world.run(&host, Some(&sink)).unwrap_err();
        assert!(
            err.to_string().contains("the host refused the merge"),
            "{err:#}"
        );
        let records = world.records();
        assert_eq!((intents(&records).len(), landings(&records).len()), (1, 0));
        assert!(world.remote_tip().is_some());
        assert_eq!(
            world.git.rev_parse(RECORDS_REF).unwrap(),
            world.remote_tip()
        );
    }

    #[test]
    fn a_merge_that_is_not_confirmed_writes_no_landing_warns_and_a_rerun_adds_nothing() {
        let world = World::new();
        let host = world.host();
        host.merge_confirms.set(false);
        let sink = world.sink();
        let report = world.run(&host, Some(&sink)).unwrap();
        let records = world.records();
        assert_eq!((intents(&records).len(), landings(&records).len()), (1, 0));
        let intent_id = intents(&records)[0].1.id.to_string();
        assert_eq!(report.warnings.len(), 1, "{:?}", report.warnings);
        assert!(
            report.warnings[0].contains(&intent_id),
            "{:?}",
            report.warnings
        );
        // The result is the one the CLI already maps: an open pull request with a merge requested.
        assert_eq!(report.link.state, LinkState::Open);
        assert!(report.merge_requested);

        let tip = world.remote_tip();
        let again = world.run(&host, Some(&sink)).unwrap();
        assert_eq!(world.remote_tip(), tip, "a rerun makes no commit");
        let records = world.records();
        assert_eq!((intents(&records).len(), landings(&records).len()), (1, 0));
        assert!(again.warnings[0].contains(&intent_id));
    }

    #[test]
    fn a_leftover_local_records_branch_is_reset_to_the_remote_before_writing() {
        let world = World::new();
        let host = world.host();
        host.merge_refused.set(true);
        let sink = world.sink();
        world.run(&host, Some(&sink)).unwrap_err();
        let intent_tip = world.remote_tip().unwrap();

        // Ahead: a record that only the local branch has. Diverged: the remote also moves, through
        // another clone.
        let writer = RecordsWriter::new(&world.git);
        let leftover = writer
            .append("leftover", &[Change::put("leftover.json", "{}")])
            .unwrap();
        let other_dir = tempfile::tempdir().unwrap();
        let other = Git::isolated(other_dir.path());
        other.run(["init", "-q"]).unwrap();
        let url = world.bare.dir().to_str().unwrap().to_string();
        other.run(["remote", "add", "origin", &url]).unwrap();
        let rival = RecordsWriter::new(&other)
            .append_to_remote("origin", "rival", &[Change::put("rival.json", "r")], &[])
            .unwrap();
        assert_eq!(world.git.parents(&leftover).unwrap(), vec![intent_tip]);

        host.merge_refused.set(false);
        let report = world.run(&host, Some(&sink)).unwrap();
        assert_eq!(report.link.state, LinkState::Landed);
        let tip = world.remote_tip().unwrap();
        assert_eq!(
            world.git.rev_parse(RECORDS_REF).unwrap().as_deref(),
            Some(tip.as_str())
        );
        assert_eq!(
            world.bare.rev_parse(&leftover).unwrap(),
            None,
            "the leftover never left"
        );
        assert!(world.bare.is_ancestor(&rival, &tip).unwrap());
        let records = world.records();
        assert_eq!(
            (intents(&records).len(), landings(&records).len()),
            (1, 1),
            "the intent was reused"
        );
    }

    #[test]
    fn for_remote_builds_a_sink_that_writes_to_the_remote_records_branch() {
        let world = World::new();
        let sink = GitLandingRecords::for_remote(&world.git, "origin").unwrap();
        assert_eq!(sink.base_tip("trunk").unwrap(), world.base);
        assert!(sink.base_tip("nope").is_err());
        assert_eq!(world.remote_tip(), None);
        let intent = sink.intent(7, &world.head, &world.base).unwrap();
        let facts = LandingFacts {
            intent: intent.clone(),
            expected_base: world.base.clone(),
            pr_number: 7,
            head: world.head.clone(),
            host_commit: HOST_COMMIT.to_string(),
            merge_method: MergeMethod::Squash,
        };
        let landing = sink.landing(&facts).unwrap();
        assert_eq!(
            sink.landing(&facts).unwrap(),
            landing,
            "a repeat finds the same landing"
        );
        let records = world.records();
        assert_eq!(intents(&records)[0].1.id, intent);
        assert_eq!(landings(&records)[0].1.id, landing);
        assert_eq!(
            landings(&records)[0].1.merge_method,
            history::MergeMethod::Squash
        );
        let err = GitLandingRecords::for_remote(&world.git, "elsewhere").unwrap_err();
        assert!(err.to_string().contains("elsewhere"), "{err:#}");
    }

    #[test]
    fn commits_carry_their_kind_job_and_verification_and_pass_validation() {
        let mut world = World::new();
        let note = crate::land::fixtures::plain(
            "records/introspections/2026-10/20261024-120000-abcd.json",
            "{}\n",
        );
        let records_only = world.add_files(&[note], "Records");
        let intervention = RecordId::parse("20261024-121000-abcd").unwrap();
        let path = format!(
            "records/interventions/{}/{intervention}.json",
            intervention.month()
        );
        let declared = world.add_files(
            &[
                crate::land::fixtures::plain("c.txt", "by hand\n"),
                crate::land::fixtures::plain(&path, "{}\n"),
            ],
            "Hand work",
        );
        let commits = world.sink().commits(&world.base, &world.head).unwrap();
        for commit in &commits {
            commit.validate().unwrap();
        }
        assert_eq!(
            commits
                .iter()
                .map(|c| (c.kind, c.job.is_some(), c.verification.is_some()))
                .collect::<Vec<_>>(),
            [
                (LandingKind::JobLanding, true, true),
                (LandingKind::RecordsOnly, false, false),
                (LandingKind::DeclaredIntervention, false, false),
            ]
        );
        assert_eq!(commits[0].job.as_ref(), Some(&world.job));
        assert_eq!(commits[0].verification, Some(RecordId::parse(JOB).unwrap()));
        assert_eq!(commits[1].sha, records_only);
        assert_eq!(commits[2].sha, declared);
        // The intent names only the jobs that landed.
        let sink = world.sink();
        sink.intent(42, &world.head, &world.base).unwrap();
        let records = world.records();
        assert_eq!(intents(&records)[0].1.job_ids, vec![world.job.clone()]);
    }

    #[test]
    fn a_pull_request_that_lands_no_job_has_no_intent() {
        let mut world = World::new();
        // The only commit above the base is an intervention; nothing lands a job.
        world
            .git
            .run(["reset", "-q", "--hard", &world.base])
            .unwrap();
        let intervention = RecordId::parse("20261024-121000-abcd").unwrap();
        let path = format!(
            "records/interventions/{}/{intervention}.json",
            intervention.month()
        );
        world.add_files(
            &[
                crate::land::fixtures::plain("c.txt", "by hand\n"),
                crate::land::fixtures::plain(&path, "{}\n"),
            ],
            "Hand work",
        );
        let err = world
            .sink()
            .intent(42, &world.head, &world.base)
            .unwrap_err()
            .to_string();
        assert!(err.contains("lands a job"), "{err}");
        assert_eq!(world.remote_tip(), None);
    }

    #[test]
    fn a_governed_commit_no_record_covers_is_named_and_nothing_is_recorded_or_merged() {
        let mut world = World::new();
        let bad = world.add_commit("c.txt", "Hand edit with no records");
        let host = world.host();
        let sink = world.sink();
        let err = world.run(&host, Some(&sink)).unwrap_err().to_string();
        assert!(err.contains(&bad), "{err}");
        assert!(
            err.contains("lies between the remote base and the PR head"),
            "{err}"
        );
        assert_eq!(world.remote_tip(), None);
        assert_eq!(host.merges.get(), 0);
    }

    #[test]
    fn a_bootstrap_commit_has_no_verification_and_is_named() {
        let err = verification_id(&JobId::Bootstrap, "abc123").unwrap_err();
        assert!(err.to_string().contains("abc123"), "{err}");
        let id = RecordId::parse(JOB).unwrap();
        assert_eq!(verification_id(&JobId::Id(id.clone()), "abc").unwrap(), id);
    }

    /// The repository's own configuration with `[history] cutover_job` set to the test job, or removed.
    fn cutover_config(names_job: bool) -> String {
        athena_gate::BUILTIN_CONFIG
            .lines()
            .filter(|l| names_job || !l.starts_with("cutover_job = "))
            .map(|l| {
                if l.starts_with("cutover_job = ") {
                    format!("cutover_job = \"{JOB}\"")
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }

    /// A world whose pull request also adds `athena.toml` (naming the job as the cutover job when
    /// `names_job`), and whose host merge makes a real squash commit on the remote's default branch.
    /// Returns the world and that commit. The checkout the landing starts from has no `athena.toml`.
    fn cutover_world(names_job: bool) -> (World, &'static str) {
        let mut world = World::new();
        world
            .git
            .run(["reset", "-q", "--hard", &world.base])
            .unwrap();
        let mut files = crate::land::fixtures::landed(&world.job);
        files.push(crate::land::fixtures::plain("b.txt", "b.txt"));
        files.push(crate::land::fixtures::plain(
            "athena.toml",
            &cutover_config(names_job),
        ));
        world.add_files(&files, "Work");
        let tree = world.git.tree_of(&world.head).unwrap();
        let squash = world
            .git
            .commit_tree(&tree, &[&world.base], "Squash", false)
            .unwrap();
        world
            .git
            .run([
                "push",
                "-q",
                "origin",
                &format!("{squash}:refs/heads/trunk"),
            ])
            .unwrap();
        let squash: &'static str = Box::leak(squash.into_boxed_str());
        world.content = FakeContent(BTreeMap::from([(squash, "verified")]));
        (world, squash)
    }

    fn legacy_record_at(world: &World, tip: &str) -> history::LegacyExempt {
        let bytes = world
            .bare
            .show_file(tip, athena_gate::legacy::LEGACY_EXEMPT_PATH)
            .unwrap()
            .expect("the legacy-exempt record is on the records branch");
        match HistoryRecord::parse(&String::from_utf8(bytes).unwrap()).unwrap() {
            HistoryRecord::LegacyExempt(record) => record,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn cutover_job_is_read_from_the_commit_given_not_the_checkout() {
        let (world, squash) = cutover_world(true);
        assert_eq!(cutover_job_for(&world.git, squash).as_deref(), Some(JOB));
        // The base commit of the landing has no `athena.toml`, and the built-in configuration (which
        // names this repository's own cutover job) is not consulted for it.
        assert_eq!(cutover_job_for(&world.git, &world.base), None);
        let (without, squash) = cutover_world(false);
        assert_eq!(cutover_job_for(&without.git, squash), None);
    }

    #[test]
    fn cutover_land_imports_at_c0_and_writes_the_legacy_exempt_record() {
        let (world, squash) = cutover_world(true);
        let mut host = world.host();
        host.merge_makes = squash;
        let sink = world.sink();
        let report = world.run(&host, Some(&sink)).unwrap();
        assert_eq!(report.link.state, LinkState::Landed);
        assert_eq!(report.warnings.len(), 1, "{:?}", report.warnings);
        assert!(
            report.warnings[0].contains("cutover: imported records/ at C0"),
            "{:?}",
            report.warnings
        );

        // The landing record, then one commit: the import with its manifest and the legacy-exempt record.
        let records = world.records();
        let (landing_commit, landing) = landings(&records)[0].clone();
        assert_eq!(landing.host_commit, squash);
        let tip = world.remote_tip().unwrap();
        assert_eq!(
            world.bare.parents(&tip).unwrap(),
            vec![landing_commit],
            "the import is one commit on top of the landing"
        );

        // C0 is resolved from the landing record, and the import's self-check passes against it.
        let c0 = athena_gate::attribution::cutover_commit_at(&world.git, &tip, JOB)
            .unwrap()
            .expect("a landing of the cutover job");
        assert_eq!(c0, squash);
        crate::records_import::verify_cutover_import(&world.git, &c0, &tip).unwrap();
        let manifest = world
            .bare
            .show_file(&tip, crate::records_import::MANIFEST_PATH)
            .unwrap()
            .expect("the manifest is on the records branch");
        match HistoryRecord::parse(&String::from_utf8(manifest).unwrap()).unwrap() {
            HistoryRecord::ImportManifest(m) => {
                assert_eq!(m.c0, squash);
                assert!(!m.paths.is_empty());
                for path in &m.paths {
                    assert!(world.bare.show_file(&tip, &path.path).unwrap().is_some());
                }
            }
            other => panic!("{other:?}"),
        }
        assert!(
            athena_gate::attribution::manifest_names(&world.git, &tip, &c0).unwrap(),
            "the history check sees the cutover as shown"
        );

        // Seven entries, six listed in athena.toml plus 02837e0, each with a reason.
        let exempt = legacy_record_at(&world, &tip);
        assert_eq!(exempt.entries.len(), 7);
        assert!(
            exempt
                .entries
                .iter()
                .any(|e| e.sha.starts_with("02837e0") && !e.reason.is_empty())
        );
        assert!(exempt.entries.iter().all(|e| !e.reason.trim().is_empty()));

        // A landing of another job is not the cutover.
        assert_eq!(
            athena_gate::attribution::cutover_commit_at(&world.git, &tip, "20261024-100000-zzzz")
                .unwrap(),
            None
        );
    }

    #[test]
    fn a_land_whose_config_names_no_cutover_job_writes_no_import() {
        let (world, squash) = cutover_world(false);
        let mut host = world.host();
        host.merge_makes = squash;
        let sink = world.sink();
        let report = world.run(&host, Some(&sink)).unwrap();
        assert_eq!(report.link.state, LinkState::Landed);
        assert!(report.warnings.is_empty(), "{:?}", report.warnings);
        let tip = world.remote_tip().unwrap();
        assert!(
            world
                .bare
                .show_file(&tip, crate::records_import::MANIFEST_PATH)
                .unwrap()
                .is_none()
        );
        assert_eq!(world.records().len(), 2, "the intent and the landing only");
    }

    #[test]
    fn run_cutover_writes_import_and_exempt_record_once() {
        let (world, squash) = cutover_world(true);
        let mut host = world.host();
        host.merge_makes = squash;
        // Land without the cutover: the sink the land used does nothing on the hook.
        struct NoCutover<'a>(GitLandingRecords<'a>);
        impl LandingRecords for NoCutover<'_> {
            fn base_tip(&self, branch: &str) -> Result<String> {
                self.0.base_tip(branch)
            }
            fn intent(&self, pr: u64, head: &str, base: &str) -> Result<RecordId> {
                self.0.intent(pr, head, base)
            }
            fn landing(&self, facts: &LandingFacts) -> Result<RecordId> {
                self.0.landing(facts)
            }
        }
        let plain = NoCutover(world.sink());
        world.run(&host, Some(&plain)).unwrap();
        let landed_tip = world.remote_tip().unwrap();
        assert!(
            world
                .bare
                .show_file(&landed_tip, crate::records_import::MANIFEST_PATH)
                .unwrap()
                .is_none()
        );

        let sink = world.sink();
        let first = sink.run_cutover(JOB).unwrap().unwrap();
        assert!(first.import.is_some() && first.exempt_written, "{first:?}");
        let after = world.remote_tip().unwrap();
        assert_ne!(after, landed_tip);
        let second = sink.run_cutover(JOB).unwrap().unwrap();
        assert!(!second.wrote(), "{second:?}");
        assert_eq!(world.remote_tip().unwrap(), after, "nothing was written");
        assert_eq!(
            sink.run_cutover("20261024-100000-zzzz").unwrap(),
            None,
            "a job with no landing has no cutover"
        );
    }
}
