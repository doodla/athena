//! `athena check --history` after the cutover, on a pull request head (the required "the full history" check).
//!
//! The commits of a head that main does not hold yet cannot be named by a landing record, so they are judged
//! by the pre-merge rule (records only, a declared intervention, or a verified job landing); the commits on
//! main are still judged by attribution. These tests run the real binary on the shared cutover fixture.

mod common;

use athena_core::run::GoResult;
use athena_core::test_support::CutoverFixture;
use athena_data::records::OutcomeState;
use common::*;

const INTERVENTION: &str = r#"{
  "schema": "athena.intervention/1",
  "id": "20261026-100000-abcd",
  "at": "2026-10-26T10:00:00Z",
  "what": "Edited a file by hand",
  "why": "A small fix",
  "class_of_work": "code",
  "category": "did-work-themselves",
  "job": null,
  "source": "user",
  "with_staged_changes": true
}
"#;

fn commit_all(fx: &CutoverFixture, message: &str) -> String {
    fx.git.run(["add", "-A"]).unwrap();
    fx.git.run(["commit", "-q", "-m", message]).unwrap();
    fx.git.head().unwrap().unwrap()
}

/// Runs `athena check --history` and returns whether it passed, with its output.
fn history(fx: &CutoverFixture) -> (bool, String) {
    let out = athena_check(fx, "--history");
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    (out.status.success(), text)
}

/// A job read, dispatched, verified and landed by `go` on the current branch, after `C0`. Returns the commit.
fn land_a_job(fx: &CutoverFixture) -> String {
    let scenario = Scenario {
        edits: vec![(
            "README.md".into(),
            Some("# Demo\n\nHello, reader.\n\nThis is the cutover.\n\nA later job.\n".into()),
        )],
        ..Scenario::default()
    };
    let athena = fx.athena(athena_cli::wiring::capabilities(), scenario.script());
    let proposal = athena
        .ask("add a line to the readme", &Lines::default())
        .unwrap();
    match athena
        .go(
            proposal.job.as_str(),
            athena_data::records::AcceptedBy::User,
            &Lines::default(),
        )
        .unwrap()
    {
        GoResult::Finished(f) => {
            assert_eq!(f.state, OutcomeState::Landed, "{}", f.reason);
            f.landed_commit.expect("a landed commit")
        }
        GoResult::Reread(_) => panic!("the job was read again"),
    }
}

fn start_branch(fx: &CutoverFixture) {
    fx.git.run(["checkout", "-q", "-b", "pr"]).unwrap();
}

#[test]
fn a_pr_head_of_records_only_intervention_and_job_landing_commits_passes() {
    let fx = cutover_fixture();
    start_branch(&fx);

    // A job landed on the branch (`go` commits on the current branch); main stays at C0.
    let landing = land_a_job(&fx);
    assert!(!fx.git.is_ancestor(&landing, "main").unwrap());

    // A records-only commit (an excluded path) and a declared intervention.
    write(fx.root(), &[("PRIVATE.md", "owner notes, edited\n")]);
    let records_only = commit_all(&fx, "Edit the owner notes");
    write(
        fx.root(),
        &[(
            "records/interventions/2026-10/20261026-100000-abcd.json",
            INTERVENTION,
        )],
    );
    let intervention = commit_all(&fx, "Declare an intervention");

    // No landing record names any of these.
    let tip = fx.records_tip().unwrap();
    for sha in [&landing, &records_only, &intervention] {
        assert_eq!(
            athena_gate::attribution::cutover_commit_at(&fx.git, &tip, sha).unwrap(),
            None
        );
    }

    let (passed, text) = history(&fx);
    assert!(passed, "{text}");
    assert!(text.contains("0 violation(s)"), "{text}");
}

#[test]
fn a_pr_head_with_an_unverified_governed_change_fails_naming_it() {
    let fx = cutover_fixture();
    start_branch(&fx);
    write(fx.root(), &[("PRIVATE.md", "owner notes, edited\n")]);
    let fine = commit_all(&fx, "Edit the owner notes");
    write(
        fx.root(),
        &[("src/lib.rs", "pub fn one() -> u32 {\n    2\n}\n")],
    );
    let bad = commit_all(&fx, "Change governed content by hand");

    let (passed, text) = history(&fx);
    assert!(!passed, "{text}");
    assert!(text.contains(&bad[..12]), "{text}");
    assert!(text.contains("changes governed content"), "{text}");
    assert!(!text.contains(&fine[..12]), "{text}");
    assert!(!text.contains("unattributed"), "{text}");
}

#[test]
fn history_on_main_after_c0_still_judges_its_commits_by_attribution() {
    let fx = cutover_fixture();
    // On main, local main is ahead of the remote's: the head is main, so attribution applies. A commit the
    // pre-merge rule would accept (records only) is still unattributed there.
    write(fx.root(), &[("PRIVATE.md", "owner notes, edited\n")]);
    let records_only = commit_all(&fx, "Edit the owner notes");
    let (passed, text) = history(&fx);
    assert!(!passed, "{text}");
    assert!(text.contains(&records_only[..12]), "{text}");
    assert!(text.contains("unattributed"), "{text}");

    write(
        fx.root(),
        &[("src/lib.rs", "pub fn one() -> u32 {\n    2\n}\n")],
    );
    let hand = commit_all(&fx, "Change governed content by hand");
    let (passed, text) = history(&fx);
    assert!(!passed, "{text}");
    assert!(text.contains(&hand[..12]), "{text}");
    assert!(text.contains("unattributed"), "{text}");
    assert!(!text.contains("changes governed content"), "{text}");
}
