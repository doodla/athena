//! The code-health check runs after each landing: on the finished-job path (`Athena::go` ends a job) and on the
//! pull-request path (`Athena::land_pr` sees the pull request merged). Each test puts a file over the limit in the
//! checkout, runs the real path on the shared cutover fixture, and looks for the `code health:` line and the
//! queued structural-pass request. Removing the hook call from either path fails its test.

mod common;

use anyhow::Result;
use athena_core::gh::{Poll, Runner};
use athena_core::run::GoResult;
use athena_core::store::{JobState, Records};
use athena_data::records::{AcceptedBy, OutcomeState};
use common::*;
use std::time::Duration;

const BIG: &str = "crates/big/src/lib.rs";

/// A Rust file with `lines` production lines, each a constant.
fn big_file(root: &std::path::Path, lines: usize) {
    let text: String = (0..lines)
        .map(|i| format!("pub const C{i}: u32 = {i};\n"))
        .collect();
    write(root, &[(BIG, text.as_str())]);
}

/// The queued structural-pass requests for `path` at the checkout's HEAD.
fn structural_requests(fx: &athena_core::test_support::CutoverFixture, path: &str) -> Vec<String> {
    let head = fx.git.head().unwrap().unwrap();
    let records = Records::load(&fx.git, &head).unwrap();
    let marker = format!("Structural pass: {path}");
    records
        .jobs
        .values()
        .filter(|j| j.state() == JobState::Queued)
        .filter_map(|j| j.request.as_ref())
        .filter(|r| r.text.lines().any(|l| l.trim() == marker))
        .map(|r| r.text.clone())
        .collect()
}

#[test]
fn a_finished_job_runs_the_code_health_check_and_queues_a_structural_pass() {
    let fx = cutover_fixture();
    big_file(fx.root(), 2001);
    assert!(structural_requests(&fx, BIG).is_empty());
    let scenario = Scenario {
        edits: vec![(
            "README.md".into(),
            Some("# Demo\n\nHello, reader.\n\nThis is the cutover.\n\nA later job.\n".into()),
        )],
        ..Scenario::default()
    };
    let athena = fx.athena(athena_cli::wiring::capabilities(), scenario.script());
    let ui = Lines::default();
    let proposal = athena.ask("add a line to the readme", &ui).unwrap();
    match athena
        .go(proposal.job.as_str(), AcceptedBy::User, &ui)
        .unwrap()
    {
        GoResult::Finished(f) => assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason),
        GoResult::Reread(_) => panic!("the job was read again"),
    }
    let lines = ui.0.lock().unwrap().clone();
    let said = lines
        .iter()
        .find(|l| l.starts_with("code health:"))
        .unwrap_or_else(|| panic!("no code health line in {lines:#?}"));
    assert!(
        said.contains("1 over a limit") && said.contains("queued 1 structural pass"),
        "{said}"
    );
    let queued = structural_requests(&fx, BIG);
    assert_eq!(queued.len(), 1, "{queued:#?}");
    assert!(queued[0].contains("2001 production lines"), "{}", queued[0]);
}

/// A `gh` for a pull request the host has merged as `merged`; every other call gets an empty answer.
struct Merged(String);

impl Runner for Merged {
    fn gh(&self, args: &[String]) -> Result<String> {
        let path = args.iter().find(|a| a.starts_with("repos/")).unwrap();
        Ok(if path.contains("/pulls/") && !path.ends_with("/merge") {
            format!(r#"{{"merged": true, "merge_commit_sha": "{}"}}"#, self.0)
        } else {
            "{}".to_string()
        })
    }
}

#[test]
fn landing_a_pull_request_runs_the_code_health_check_and_queues_a_structural_pass() {
    let fx = cutover_fixture();
    // The remote must look like a GitHub repository to `land_pr`: a copy of the bare remote whose path names
    // github.com. Everything stays on disk.
    let host = tempfile::tempdir().unwrap();
    let copy = host.path().join("github.com/example/project.git");
    std::fs::create_dir_all(copy.parent().unwrap()).unwrap();
    fx.git
        .run([
            "clone",
            "-q",
            "--bare",
            fx.bare_root().to_str().unwrap(),
            copy.to_str().unwrap(),
        ])
        .unwrap();
    fx.git
        .run(["remote", "set-url", "origin", copy.to_str().unwrap()])
        .unwrap();
    big_file(fx.root(), 2001);
    let athena = fx.athena(
        athena_cli::wiring::capabilities(),
        Scenario::default().script(),
    );
    let ui = Lines::default();
    let nap = |_: Duration| {};
    let report = athena
        .land_pr(
            fx.cutover_job.as_str(),
            &Merged(fx.c0.clone()),
            Poll {
                attempts: 1,
                delay: Duration::ZERO,
                sleep: &nap,
            },
            &ui,
        )
        .unwrap();
    assert_eq!(report.link.state, athena_core::pr::LinkState::Landed);
    let lines = ui.0.lock().unwrap().clone();
    assert!(
        lines.iter().any(|l| l.starts_with("code health:")),
        "no code health line in {lines:#?}"
    );
    let queued = structural_requests(&fx, BIG);
    assert_eq!(queued.len(), 1, "{queued:#?}");
}
