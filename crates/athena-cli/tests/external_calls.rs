//! Every call to GitHub goes through the one retrying wrapper in `athena_core::gh`. These tests inject a 500
//! into each GitHub-calling operation and into the whole of `athena land --pr` (its `land_pr` flow): a one-off
//! 500 is retried and succeeds; persistent 500s fail after three attempts with an error naming the call and
//! the status. The source guard is in `crates/athena-core/tests/external_calls_guard.rs`.

use anyhow::Result;
use athena_core::Ui;
use athena_core::gh::{
    Checks, GhClient, GhFailure, LandPr, NewPr, PagesHost, Poll, PrDraft, PrHost, Remote, Retry,
    Runner, land_pr, watch_pages,
};
use athena_core::pr::{Ancestry, Content, Host, LinkState, PrLink};
use athena_core::state::StateDir;
use athena_data::JobId;
use athena_git::CommitText;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Duration;

/// A `gh` that answers like the host, but fails with a 500 the first `failures` times each distinct call is
/// made, or on every call when `persistent`.
struct Injecting {
    failures: u32,
    persistent: bool,
    seen: RefCell<BTreeMap<String, u32>>,
    calls: RefCell<Vec<String>>,
}

impl Injecting {
    fn one_off() -> Self {
        Self::new(1, false)
    }
    fn persistent() -> Self {
        Self::new(0, true)
    }
    fn new(failures: u32, persistent: bool) -> Self {
        Injecting {
            failures,
            persistent,
            seen: RefCell::new(BTreeMap::new()),
            calls: RefCell::new(Vec::new()),
        }
    }
    fn calls_to(&self, path: &str) -> usize {
        self.calls.borrow().iter().filter(|c| *c == path).count()
    }
    fn distinct_calls(&self) -> usize {
        self.seen.borrow().len()
    }
}

impl Runner for Injecting {
    fn gh(&self, args: &[String]) -> Result<String> {
        let path = args[3].clone();
        self.calls.borrow_mut().push(path.clone());
        let n = {
            let mut seen = self.seen.borrow_mut();
            let n = seen.entry(path.clone()).or_insert(0);
            *n += 1;
            *n
        };
        if self.persistent || n <= self.failures {
            return Err(GhFailure {
                call: path,
                status_code: Some(500),
                text: "gh: Server Error (HTTP 500)".into(),
            }
            .into());
        }
        Ok(if path == "repos/example/project" {
            r#"{"default_branch": "trunk", "allow_merge_commit": true}"#.to_string()
        } else if path.ends_with("/pulls") {
            r#"{"number": 7}"#.to_string()
        } else if path.ends_with("/merge")
            || path.ends_with("/rerun")
            || path.ends_with("/rerun-failed-jobs")
        {
            "{}".to_string()
        } else if path.contains("actions/workflows/") {
            r#"{"workflow_runs": [{"id": 5, "status": "completed", "conclusion": "success", "run_attempt": 1}]}"#
                .to_string()
        } else if path.contains("/pulls/") {
            // Merged only once the merge request has been made.
            if self.seen.borrow().keys().any(|p| p.ends_with("/merge")) {
                r#"{"merged": true, "merge_commit_sha": "m1"}"#.to_string()
            } else {
                r#"{"merged": false}"#.to_string()
            }
        } else if path.contains("check-runs") {
            r#"{"check_runs": [{"status": "completed", "conclusion": "success"}]}"#.to_string()
        } else {
            r#"{"state": "success", "statuses": []}"#.to_string()
        })
    }
}

struct Lines(RefCell<Vec<String>>);
impl Ui for Lines {
    fn line(&self, text: &str) {
        self.0.borrow_mut().push(text.to_string());
    }
}

fn client<'a>(runner: &'a Injecting, ui: &'a Lines) -> GhClient<'a> {
    GhClient {
        retry: Retry {
            attempts: 3,
            delay: Duration::ZERO,
        },
        ui: Some(ui),
        ..GhClient::new(runner, "example/project".into())
    }
}

fn link() -> PrLink {
    PrLink {
        pr_number: 7,
        host: "github.com".into(),
        repository: "example/project".into(),
        head: "abc".into(),
        job: "j".into(),
        state: LinkState::Open,
        merged_commit: None,
    }
}

fn text() -> CommitText {
    CommitText {
        title: "Change".into(),
        body: "Body.\n".into(),
    }
}

/// An operation's name, the first path it calls, and the call.
type Operation = (&'static str, &'static str, fn(&GhClient) -> Result<()>);

/// The GitHub-calling operations of the client, by name, each returning only success or failure.
fn operations() -> Vec<Operation> {
    vec![
        ("default branch", "repos/example/project", |c| {
            c.default_branch().map(|_| ())
        }),
        ("open pull request", "repos/example/project/pulls", |c| {
            c.open_pr(&NewPr {
                title: "t",
                body: "b",
                head: "athena/j",
                base: "trunk",
            })
            .map(|_| ())
        }),
        (
            "read checks",
            "repos/example/project/commits/abc/check-runs?per_page=100",
            |c| c.checks("abc").map(|_| ()),
        ),
        ("merge", "repos/example/project", |c| {
            c.merge(7, "abc", &text()).map(|_| ())
        }),
        ("merged commit", "repos/example/project/pulls/7", |c| {
            c.merged_commit(&link()).map(|_| ())
        }),
        (
            "list pages runs",
            "repos/example/project/actions/workflows/pages.yml/runs?head_sha=abc&per_page=20",
            |c| c.pages_runs("abc").map(|_| ()),
        ),
        (
            "re-run pages",
            "repos/example/project/actions/runs/5/rerun",
            |c| c.rerun_pages(5),
        ),
        (
            "re-run failed jobs",
            "repos/example/project/actions/runs/5/rerun-failed-jobs",
            |c| c.rerun_failed(5),
        ),
    ]
}

/// A `gh` that refuses every call with the given status, as the host does for a run still in progress.
struct Refusing(u16);

impl Runner for Refusing {
    fn gh(&self, args: &[String]) -> Result<String> {
        Err(GhFailure {
            call: args[3].clone(),
            status_code: Some(self.0),
            text: format!("gh: refused (HTTP {})", self.0),
        }
        .into())
    }
}

#[test]
fn a_refused_rerun_of_failed_jobs_is_posted_once_and_recorded_in_the_host_error_log() {
    use athena_core::gh::HostErrorLog;
    use athena_data::records::{HostOperation, WorkPhase};
    let runner = Refusing(403);
    let log = HostErrorLog::new(JobId::parse("bootstrap").unwrap(), WorkPhase::Landing);
    let client = GhClient {
        log: Some(&log),
        retry: Retry {
            attempts: 3,
            delay: Duration::ZERO,
        },
        ..GhClient::new(&runner, "example/project".into())
    };
    assert!(client.rerun_failed(5).is_err());
    let entries = log.entries();
    assert_eq!(entries.len(), 1, "a 403 is not transient, so not retried");
    assert_eq!(entries[0].operation, HostOperation::ReadChecks);
    assert_eq!(entries[0].status_code, Some(403));
}

#[test]
fn the_rerun_of_failed_jobs_is_a_post() {
    struct Method(RefCell<Vec<String>>);
    impl Runner for Method {
        fn gh(&self, args: &[String]) -> Result<String> {
            self.0.borrow_mut().push(format!("{} {}", args[2], args[3]));
            Ok(String::new())
        }
    }
    let runner = Method(RefCell::new(vec![]));
    GhClient::new(&runner, "example/project".into())
        .rerun_failed(5)
        .unwrap();
    assert_eq!(
        *runner.0.borrow(),
        vec!["POST repos/example/project/actions/runs/5/rerun-failed-jobs".to_string()]
    );
}

#[test]
fn a_one_off_500_is_retried_and_succeeds_for_every_github_operation() {
    for (name, first_path, run) in operations() {
        let runner = Injecting::one_off();
        let lines = Lines(RefCell::new(vec![]));
        run(&client(&runner, &lines)).unwrap_or_else(|e| panic!("{name}: {e:#}"));
        // Every distinct call failed once and was made again.
        assert_eq!(
            runner.calls.borrow().len(),
            2 * runner.distinct_calls(),
            "{name}"
        );
        assert_eq!(runner.calls_to(first_path), 2, "{name}");
        let lines = lines.0.into_inner();
        assert_eq!(lines.len(), runner.distinct_calls(), "{name}: {lines:?}");
        assert!(
            lines[0].contains(first_path) && lines[0].contains("HTTP 500"),
            "{name}: {lines:?}"
        );
    }
}

#[test]
fn persistent_500s_fail_after_three_attempts_naming_the_call_and_status() {
    for (name, first_path, run) in operations() {
        let runner = Injecting::persistent();
        let lines = Lines(RefCell::new(vec![]));
        let error = run(&client(&runner, &lines)).unwrap_err();
        let message = error.to_string();
        assert_eq!(runner.calls_to(first_path), 3, "{name}");
        assert_eq!(
            runner.calls.borrow().len(),
            3,
            "{name}: stops at the first failed call"
        );
        assert!(message.contains(first_path), "{name}: {message}");
        assert!(message.contains("HTTP 500"), "{name}: {message}");
        assert!(message.contains("3 attempts"), "{name}: {message}");
        assert_eq!(lines.0.borrow().len(), 2, "{name}: one line per retry");
    }
}

// ---- the Pages watch after a merge ----

/// A `gh` for the Pages workflow: the first try of run 5 concludes `first` (no run at all when none), and
/// once it has been re-run the second try concludes `second`.
struct PagesFake {
    first: Option<&'static str>,
    second: &'static str,
    reruns: RefCell<u32>,
    calls: RefCell<Vec<String>>,
}

impl PagesFake {
    fn new(first: Option<&'static str>, second: &'static str) -> Self {
        PagesFake {
            first,
            second,
            reruns: RefCell::new(0),
            calls: RefCell::new(vec![]),
        }
    }
    fn rerun_calls(&self) -> usize {
        self.calls
            .borrow()
            .iter()
            .filter(|c| c.ends_with("/rerun"))
            .count()
    }
}

impl Runner for PagesFake {
    fn gh(&self, args: &[String]) -> Result<String> {
        let path = args[3].clone();
        self.calls.borrow_mut().push(path.clone());
        if path.ends_with("/rerun") {
            assert_eq!(args[2], "POST");
            *self.reruns.borrow_mut() += 1;
            return Ok("{}".into());
        }
        assert!(path.contains("actions/workflows/pages.yml/runs?head_sha=m1"));
        let Some(first) = self.first else {
            return Ok(r#"{"workflow_runs": []}"#.into());
        };
        let (conclusion, attempt) = if *self.reruns.borrow() == 0 {
            (first, 1)
        } else {
            (self.second, 2)
        };
        Ok(format!(
            r#"{{"workflow_runs": [{{"id": 5, "status": "completed", "conclusion": "{conclusion}", "run_attempt": {attempt}}}]}}"#
        ))
    }
}

fn watch(runner: &PagesFake) -> (athena_core::gh::PagesReport, Vec<String>) {
    let lines = Lines(RefCell::new(vec![]));
    let nap = |_: Duration| {};
    let poll = Poll {
        attempts: 3,
        delay: Duration::ZERO,
        sleep: &nap,
    };
    let host = GhClient::new(runner, "example/project".into());
    let report = watch_pages(&host, "m1", &poll, &lines).unwrap();
    (report, lines.0.into_inner())
}

#[test]
fn a_timed_out_pages_run_is_re_run_once_and_then_succeeds() {
    let runner = PagesFake::new(Some("timed_out"), "success");
    let (report, lines) = watch(&runner);
    assert_eq!(runner.rerun_calls(), 1);
    assert_eq!(
        runner
            .calls
            .borrow()
            .iter()
            .find(|c| c.ends_with("/rerun"))
            .unwrap(),
        "repos/example/project/actions/runs/5/rerun"
    );
    assert_eq!(report.run, Some(5));
    assert_eq!(report.reruns, 1);
    assert_eq!(report.conclusion.as_deref(), Some("success"));
    let naming: Vec<_> = lines
        .iter()
        .filter(|l| l.contains("run 5") && l.contains("timed_out"))
        .collect();
    assert_eq!(naming.len(), 1, "{lines:?}");
}

#[test]
fn a_cancelled_pages_run_is_re_run_once_and_a_second_failure_is_not_re_run() {
    let runner = PagesFake::new(Some("cancelled"), "timed_out");
    let (report, lines) = watch(&runner);
    assert_eq!(runner.rerun_calls(), 1);
    assert_eq!(report.reruns, 1);
    assert_eq!(report.conclusion.as_deref(), Some("timed_out"));
    assert!(
        lines
            .iter()
            .any(|l| l.contains("run 5") && l.contains("cancelled")),
        "{lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains("timed_out") && l.contains("not re-running")),
        "{lines:?}"
    );
}

#[test]
fn a_successful_pages_run_is_not_re_run() {
    let runner = PagesFake::new(Some("success"), "success");
    let (report, _) = watch(&runner);
    assert_eq!(runner.rerun_calls(), 0);
    assert_eq!(report.reruns, 0);
    assert_eq!(report.conclusion.as_deref(), Some("success"));
}

#[test]
fn a_commit_with_no_pages_run_is_not_an_error_and_nothing_is_re_run() {
    let runner = PagesFake::new(None, "success");
    let (report, lines) = watch(&runner);
    assert_eq!(runner.rerun_calls(), 0);
    assert_eq!(report.run, None);
    assert_eq!(lines.len(), 1, "{lines:?}");
}

#[test]
fn a_5xx_on_the_runs_listing_is_retried_and_reported_during_the_watch() {
    let runner = Injecting::one_off();
    let lines = Lines(RefCell::new(vec![]));
    let nap = |_: Duration| {};
    let poll = Poll {
        attempts: 3,
        delay: Duration::ZERO,
        sleep: &nap,
    };
    let report = watch_pages(&client(&runner, &lines), "m1", &poll, &lines).unwrap();
    assert_eq!(report.conclusion.as_deref(), Some("success"));
    assert_eq!(runner.calls.borrow().len(), 2);
    let lines = lines.0.into_inner();
    assert!(
        lines[0].contains("HTTP 500") && lines[0].contains("pages.yml"),
        "{lines:?}"
    );
}

// ---- `athena land --pr`: the whole flow over the client ----

struct Anc;
impl Ancestry for Anc {
    fn is_ancestor(&self, _: &str, _: &str) -> Result<bool> {
        Ok(true)
    }
}

/// A remote that holds nothing until the first push, then the pushed commit.
struct NoRemote(RefCell<Option<String>>);
impl Remote for NoRemote {
    fn remote_head(&self, _: &str) -> Result<Option<String>> {
        Ok(self.0.borrow().clone())
    }
    fn push(&self, _: &str, commit: &str) -> Result<()> {
        *self.0.borrow_mut() = Some(commit.to_string());
        Ok(())
    }
    fn commits_after(&self, _: &str, _: &str) -> Result<Vec<(String, String)>> {
        Ok(vec![])
    }
}

struct Fingerprints;
impl Content for Fingerprints {
    fn fingerprint_of(&self, _: &str) -> Result<String> {
        Ok("verified".into())
    }
}

fn land(runner: &Injecting) -> (Result<athena_core::gh::PrReport>, Vec<String>) {
    let dir = tempfile::tempdir().unwrap();
    let state = StateDir::new(dir.path().to_path_buf());
    let job = JobId::parse("bootstrap").unwrap();
    let lines = Lines(RefCell::new(vec![]));
    let client = client(runner, &lines);
    let nap = |_: Duration| {};
    let squash = text();
    let result = land_pr(&LandPr {
        state: &state,
        job: &job,
        branch: "athena/bootstrap",
        commit: "cand1",
        verified_fingerprint: "verified",
        pr: PrDraft {
            title: "t",
            body: "b",
            head: "athena/bootstrap",
        },
        squash: &squash,
        host_name: "github.com",
        repository: "example/project",
        merge_via_rest: true,
        poll: Poll {
            attempts: 1,
            delay: Duration::ZERO,
            sleep: &nap,
        },
        ancestry: &Anc,
        remote: &NoRemote(RefCell::new(None)),
        host: &client,
        content: &Fingerprints,
        records: None,
    });
    (result, lines.0.into_inner())
}

#[test]
fn land_pr_with_a_one_off_500_on_each_call_is_retried_and_lands() {
    let runner = Injecting::one_off();
    let (result, lines) = land(&runner);
    let report = result.unwrap_or_else(|e| panic!("{e:#}"));
    assert_eq!(report.link.state, LinkState::Landed);
    assert_eq!(report.checks, Some(Checks::Green));
    assert!(report.merge_requested);
    // Each distinct call failed once (one retry line each); a call repeated later by the flow succeeds at once.
    assert_eq!(lines.len(), runner.distinct_calls(), "{lines:?}");
    assert!(runner.calls_to("repos/example/project/pulls") >= 2);
    assert!(runner.calls_to("repos/example/project/pulls/7/merge") >= 2);
}

#[test]
fn land_pr_with_persistent_500s_fails_after_three_attempts_with_a_clear_error() {
    let runner = Injecting::persistent();
    let (result, lines) = land(&runner);
    let message = result.unwrap_err().to_string();
    // The first host call of the flow is the default branch.
    assert_eq!(runner.calls_to("repos/example/project"), 3);
    assert_eq!(runner.calls.borrow().len(), 3);
    assert!(message.contains("repos/example/project"), "{message}");
    assert!(message.contains("HTTP 500"), "{message}");
    assert!(message.contains("3 attempts"), "{message}");
    assert_eq!(lines.len(), 2);
}
