//! After C0 (decision D111) Athena writes no `Athena-` trailers. These tests run real jobs on the shared
//! cutover fixture, through the real writers (`intervene`, `ask`, `go`), the real `land_pr` (with a fake host
//! that makes a real squash commit), `retry_landing`, `athena status`'s trace and the auditor.
//!
//! They live in `athena-cli`'s tests because the `change` capability and the fake-agent scenarios are wired
//! here; `athena-core` cannot depend on them.

#![allow(clippy::field_reassign_with_default)]

mod common;

use athena_core::Athena;
use athena_core::manage::InterveneArgs;
use athena_core::run::{Finished, GoResult};
use athena_core::test_support::{CutoverFixture, land_through_pr, make_athena};
use athena_data::JobId;
use athena_data::history::{HistoryRecord, LandingKind};
use athena_data::records::{AcceptedBy, OutcomeState};
use athena_git::Git;
use common::*;
use std::sync::Arc;

fn finished(result: GoResult) -> Finished {
    match result {
        GoResult::Finished(f) => *f,
        GoResult::Reread(_) => panic!("read again instead of run"),
    }
}

fn athena_for(fx: &CutoverFixture, scenario: Scenario) -> Athena {
    fx.athena(athena_cli::wiring::capabilities(), scenario.script())
}

/// A scenario whose job appends a line to README.md (the fixture's README already has two).
fn readme_scenario() -> Scenario {
    let mut scenario = Scenario::default();
    scenario.edits = vec![(
        "README.md".into(),
        Some("# Demo\n\nHello, reader.\n\nThis is the cutover.\n\nA post-C0 line.\n".into()),
    )];
    scenario
}

fn has_athena_line(git: &Git, commit: &str) -> bool {
    git.message(commit)
        .unwrap()
        .lines()
        .any(|l| l.starts_with("Athena-"))
}

fn verified_fingerprint(git: &Git, commit: &str, job: &JobId) -> String {
    let records = athena_core::store::Records::load(git, commit).unwrap();
    records.jobs[job.as_str()]
        .verification
        .as_ref()
        .unwrap()
        .verified_fingerprint
        .clone()
}

/// The landing record on the bare remote's records branch that has `host` as its host commit.
fn landing_for(fx: &CutoverFixture, host: &str) -> athena_data::history::Landing {
    let tip = fx.records_tip().unwrap();
    for commit in fx.bare.rev_list_reverse(&tip).unwrap().iter().rev() {
        for path in fx.bare.added_paths(commit).unwrap() {
            if path.starts_with("landings/")
                && let Some(bytes) = fx.bare.show_file(commit, &path).unwrap()
                && let Ok(HistoryRecord::Landing(landing)) =
                    HistoryRecord::parse(&String::from_utf8_lossy(&bytes))
                && landing.host_commit == host
            {
                return landing;
            }
        }
    }
    panic!("no landing record for {host}");
}

struct PostC0 {
    fx: CutoverFixture,
    athena: Athena,
    job: JobId,
    landed: String,
    merged: String,
}

/// A hand intervention with staged changes, a job (its reading is a records-only commit) and the job's
/// landing, landed through `land_pr` with a squash merge, then `main` moved onto the squash.
fn land_post_c0_job() -> PostC0 {
    let fx = cutover_fixture();
    let athena = athena_for(&fx, readme_scenario());
    write(
        fx.root(),
        &[("src/lib.rs", "pub fn one() -> u32 {\n    11\n}\n")],
    );
    fx.git.run(["add", "src/lib.rs"]).unwrap();
    athena
        .intervene(
            &InterveneArgs {
                what: "Changed the constant by hand".into(),
                why: "It was wrong".into(),
                class: "code".into(),
                category: "did-work-themselves".into(),
                job: None,
                with_staged: true,
            },
            &Lines::default(),
        )
        .unwrap();
    let proposal = athena.ask("add a post-C0 line", &Lines::default()).unwrap();
    let done = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(done.state, OutcomeState::Landed, "{}", done.reason);
    let landed = done.landed_commit.unwrap();
    let fingerprint = verified_fingerprint(&fx.git, &landed, &proposal.job);
    let report = land_through_pr(
        &fx.git,
        &fx.state_root(),
        &proposal.job,
        &landed,
        &fx.c0,
        &fingerprint,
    )
    .unwrap();
    assert_eq!(report.link.state, athena_core::pr::LinkState::Landed);
    let merged = report.link.merged_commit.clone().unwrap();
    athena
        .resync_onto_merge(&landed, &merged, &Lines::default())
        .unwrap();
    PostC0 {
        fx,
        athena,
        job: proposal.job,
        landed,
        merged,
    }
}

#[test]
fn post_c0_landing_through_land_pr() {
    let p = land_post_c0_job();
    let fx = &p.fx;

    // The squash commit on the remote's main and every pull-request commit carry no Athena- line.
    assert!(!has_athena_line(&fx.git, &p.merged));
    assert_eq!(
        fx.bare.rev_parse("refs/heads/main").unwrap().as_deref(),
        Some(p.merged.as_str())
    );
    let landing = landing_for(fx, &p.merged);
    let kinds: Vec<LandingKind> = landing.commits.iter().map(|c| c.kind).collect();
    // The intervention first, the job's landing last, and between them the job's own records-only commits
    // (its reading, and the records `go` commits before it lands).
    assert_eq!(kinds.first(), Some(&LandingKind::DeclaredIntervention));
    assert_eq!(kinds.last(), Some(&LandingKind::JobLanding));
    assert!(kinds.len() >= 3, "{landing:#?}");
    assert!(
        kinds[1..kinds.len() - 1]
            .iter()
            .all(|k| *k == LandingKind::RecordsOnly),
        "{landing:#?}"
    );
    for commit in &landing.commits {
        assert!(
            !has_athena_line(&fx.git, &commit.sha),
            "{} carries an Athena- line:\n{}",
            commit.sha,
            fx.git.message(&commit.sha).unwrap()
        );
    }
    assert!(!has_athena_line(&fx.git, &p.landed));
    // The pull request's commits are reached from the records branch: its landing record merges the head.
    let tip = fx.records_tip().unwrap();
    assert!(fx.bare.is_ancestor(&p.landed, &tip).unwrap());

    // The history and attribution checks pass on the result.
    let history = athena_check(fx, "--history");
    let text = String::from_utf8_lossy(&history.stdout).to_string();
    assert!(
        history.status.success(),
        "{text}{}",
        String::from_utf8_lossy(&history.stderr)
    );
    assert!(text.contains("0 violation(s)"), "{text}");
    let attribution = athena_check(fx, "--attribution");
    let text = String::from_utf8_lossy(&attribution.stdout).to_string();
    assert!(
        attribution.status.success(),
        "{text}{}",
        String::from_utf8_lossy(&attribution.stderr)
    );
    assert!(text.contains("0 finding(s)"), "{text}");
}

#[test]
fn post_c0_job_is_read_by_the_trace_and_the_auditor() {
    let p = land_post_c0_job();
    let fx = &p.fx;

    // `athena status`'s trace lists the job's commits: the squash commit, with the job's state and outcome.
    let trace = p.athena.trace(p.job.as_str()).unwrap();
    assert_eq!(trace.state, "landed");
    assert!(trace.outcome.is_some());
    assert!(
        trace.commits.iter().any(|l| l.starts_with(&p.merged[..7])),
        "{:?}",
        trace.commits
    );

    // The auditor in a fresh clone: main and athena/records are fetched, the candidate commit is absent.
    let scratch = tempfile::tempdir().unwrap();
    let clone_dir = scratch.path().join("clone");
    Git::at(scratch.path())
        .run([
            "clone",
            "-q",
            fx.bare_root().to_str().unwrap(),
            clone_dir.to_str().unwrap(),
        ])
        .unwrap();
    let clone = Git::at(&clone_dir);
    for (key, value) in [
        ("user.name", "Test User"),
        ("user.email", "noreply@example.com"),
        ("commit.gpgsign", "false"),
        ("core.hooksPath", "/dev/null"),
    ] {
        clone.run(["config", key, value]).unwrap();
    }
    let records_ref = athena_git::records_branch::RECORDS_REF;
    clone
        .run([
            "fetch",
            "-q",
            "origin",
            &format!("{records_ref}:{records_ref}"),
        ])
        .unwrap();
    assert!(clone.rev_parse(records_ref).unwrap().is_some());
    write(
        &clone_dir,
        &[("BRIEF.md", "# The brief\n\nKeep it simple.\n")],
    );
    clone.run(["add", "BRIEF.md"]).unwrap();
    clone.run(["commit", "-q", "-m", "Add the brief"]).unwrap();
    let state = tempfile::tempdir().unwrap();
    let auditor = make_athena(
        &clone,
        state.path(),
        athena_cli::wiring::capabilities(),
        Scenario::default().script(),
    );
    // Without the landing-record path, the trailer-only lookup cannot find the squash commit.
    let err = athena_core::auditor::read_changed_files_by_trailers(
        &auditor,
        "HEAD",
        &p.job,
        &["README.md".to_string()],
    )
    .unwrap_err();
    assert!(err.to_string().contains("none could be read"), "{err}");
    // The auditor, through its real entry point, takes the files from the squash commit.
    let audited = auditor.audit(p.job.as_str()).unwrap().expect("an audit");
    assert_eq!(audited.audit.target.as_str(), p.job.as_str());
}

#[test]
fn retry_landing_after_c0_searches_no_trailer_and_writes_none() {
    let fx = cutover_fixture();
    let mut scenario = readme_scenario();
    let root = fx.root().to_path_buf();
    scenario.during_verification = Some(Arc::new(move || {
        std::fs::write(root.join("README.md"), "# Demo\n\nmy own local edit\n").unwrap();
    }));
    let athena = athena_for(&fx, scenario);
    let proposal = athena.ask("add a post-C0 line", &Lines::default()).unwrap();
    let blocked = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(blocked.state, OutcomeState::Blocked, "{}", blocked.reason);
    assert!(!has_athena_line(&fx.git, &blocked.closing_commit));

    fx.git.run(["checkout", "--", "README.md"]).unwrap();
    let retried = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(retried.state, OutcomeState::Landed, "{}", retried.reason);
    let commit = retried.landed_commit.unwrap();
    let message = fx.git.message(&commit).unwrap();
    assert!(!message.contains("Athena-"), "{message}");
    assert!(
        fx.git
            .path_exists(
                &commit,
                &format!("{}/outcome-landed.json", proposal.job.dir())
            )
            .unwrap()
    );

    // The retried job lands through land_pr: its landed commit is found from the records.
    let fingerprint = verified_fingerprint(&fx.git, &commit, &proposal.job);
    assert_eq!(
        athena_core::gh::landed_commit_for(&fx.git, "HEAD", &proposal.job, &fingerprint).unwrap(),
        commit
    );
    let report = land_through_pr(
        &fx.git,
        &fx.state_root(),
        &proposal.job,
        &commit,
        &fx.c0,
        &fingerprint,
    )
    .unwrap();
    assert_eq!(report.link.state, athena_core::pr::LinkState::Landed);
    assert!(!has_athena_line(
        &fx.git,
        report.link.merged_commit.as_deref().unwrap()
    ));
}

#[test]
fn retry_landing_without_a_cutover_job_still_writes_trailers() {
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    let root = fx.root().to_path_buf();
    scenario.during_verification = Some(Arc::new(move || {
        std::fs::write(root.join("README.md"), "# Demo\n\nmy own local edit\n").unwrap();
    }));
    let athena = fx.athena(scenario.script());
    let proposal = athena.ask("add a greeting", &Lines::default()).unwrap();
    let blocked = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    assert_eq!(blocked.state, OutcomeState::Blocked);
    fx.git.run(["checkout", "--", "README.md"]).unwrap();
    let retried = finished(
        athena
            .go(proposal.job.as_str(), AcceptedBy::User, &Lines::default())
            .unwrap(),
    );
    let commit = retried.landed_commit.unwrap();
    let message = fx.git.message(&commit).unwrap();
    assert!(message.contains("Athena-Verified-Content: "), "{message}");
    assert!(message.contains("Athena-Job: "), "{message}");
}
