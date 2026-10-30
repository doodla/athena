//! `athena status --snapshot <dir>` reads the records and main's history from a job's evidence snapshot, in a
//! checkout that has no `records/` of its own (a producer's working copy). The binary runs for real; the
//! figures it prints are compared with those of the checkout the snapshot was taken from. No model is called.

mod common;

use athena_core::Quiet;
use athena_core::run::GoResult;
use athena_data::records::{AcceptedBy, OutcomeState};
use common::*;
use std::process::{Command, Output};

fn athena_in(fx: &Fixture, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_athena"))
        .current_dir(fx.root())
        .env("ATHENA_STATE_DIR", fx.state.path())
        .env_remove("ATHENA_NESTED")
        .env_remove("ATHENA_JEV_SHADOW")
        .args(args)
        .output()
        .unwrap()
}

/// Equal JSON, with numbers compared to a part in 10^12: parsing a printed float can differ in its last digit.
fn assert_same(got: &serde_json::Value, expected: &serde_json::Value) {
    use serde_json::Value;
    match (got, expected) {
        (Value::Number(a), Value::Number(b)) => {
            let (a, b) = (a.as_f64().unwrap(), b.as_f64().unwrap());
            assert!(
                (a - b).abs() <= 1e-12 * a.abs().max(b.abs()).max(1e-12),
                "{a} != {b}"
            );
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len());
            a.iter().zip(b).for_each(|(a, b)| assert_same(a, b));
        }
        (Value::Object(a), Value::Object(b)) => {
            assert_eq!(a.keys().collect::<Vec<_>>(), b.keys().collect::<Vec<_>>());
            a.iter().for_each(|(k, v)| assert_same(v, &b[k]));
        }
        (a, b) => assert_eq!(a, b),
    }
}

/// A repository with one landed job (made through the fake agents), and a snapshot of it in `out`.
fn landed_and_snapshotted(out: &std::path::Path) -> (Fixture, std::path::PathBuf) {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let proposal = athena.ask("change the demo", &Quiet).unwrap();
    let GoResult::Finished(finished) = athena
        .go(proposal.job.as_str(), AcceptedBy::User, &Quiet)
        .unwrap()
    else {
        panic!("expected the job to finish");
    };
    assert_eq!(finished.state, OutcomeState::Landed, "{}", finished.reason);
    let snapshot = out.join("evidence");
    athena_core::snapshot::write(&fx.git, &fx.head(), &snapshot).unwrap();
    (fx, snapshot)
}

#[test]
fn status_snapshot_reads_the_snapshot_and_needs_no_records_directory_in_the_checkout() {
    let out = tempfile::tempdir().unwrap();
    let (fx, snapshot) = landed_and_snapshotted(out.path());
    let expected =
        serde_json::to_value(fx.athena(Scenario::default().script()).status().unwrap()).unwrap();
    assert!(
        expected["throughput"].is_object(),
        "the fixture landed a job, so there is a throughput section"
    );

    // Where the command runs: a one-commit checkout of the base with no records/.
    let working_copy = Fixture::new();
    assert!(!working_copy.root().join("records").exists());
    let run = athena_in(
        &working_copy,
        &["status", "--snapshot", snapshot.to_str().unwrap(), "--json"],
    );
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    let got: serde_json::Value = serde_json::from_slice(&run.stdout).unwrap();

    // The named fields come from the snapshot: the working copy has no job, and its head is another commit.
    assert_same(&got["throughput"], &expected["throughput"]);
    assert_eq!(got["jobs_by_state"], expected["jobs_by_state"]);
    assert_eq!(got["head"], expected["head"]);
    assert_ne!(got["head"].as_str().unwrap(), working_copy.head());
    assert_eq!(got["snapshot"]["path"], snapshot.to_str().unwrap());
    assert!(
        got["snapshot"]["omitted"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s == "code health")
    );
    assert!(!working_copy.root().join("records").exists());

    // The printed form says where the figures came from.
    let text = athena_in(
        &working_copy,
        &["status", "--snapshot", snapshot.to_str().unwrap()],
    );
    let text = String::from_utf8(text.stdout).unwrap();
    assert!(
        text.contains("snapshot    records and history of"),
        "{text}"
    );
}

#[test]
fn status_snapshot_with_a_job_or_without_history_is_refused() {
    let out = tempfile::tempdir().unwrap();
    let (_fx, snapshot) = landed_and_snapshotted(out.path());
    let working_copy = Fixture::new();

    let with_job = athena_in(
        &working_copy,
        &["status", "latest", "--snapshot", snapshot.to_str().unwrap()],
    );
    assert!(!with_job.status.success());

    let empty = tempfile::tempdir().unwrap();
    let missing = athena_in(
        &working_copy,
        &["status", "--snapshot", empty.path().to_str().unwrap()],
    );
    assert!(!missing.status.success());
    let message = String::from_utf8_lossy(&missing.stderr);
    assert!(message.contains("history.json"), "{message}");
}
