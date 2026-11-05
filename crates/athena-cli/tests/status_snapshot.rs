//! `athena status --snapshot <dir>`: the records and history of an evidence snapshot, read with no
//! repository and no `records/` in the current directory.

mod common;

use athena_core::run::GoResult;
use athena_core::snapshot;
use athena_core::status::Throughput;
use athena_core::store::Records;
use athena_data::records::{AcceptedBy, OutcomeState};
use common::*;
use std::path::{Path, PathBuf};
use std::process::Output;

/// Unlocks a snapshot when the test ends, so its temporary directory can be removed.
struct Unlock(PathBuf);
impl Drop for Unlock {
    fn drop(&mut self) {
        let _ = snapshot::unlock(&self.0);
    }
}

/// A repository where one job went from request to landed commit.
fn landed() -> (Fixture, athena_data::JobId) {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let ui = Lines::default();
    let proposal = athena.ask("add a greeting to the readme", &ui).unwrap();
    let GoResult::Finished(finished) = athena
        .go(proposal.job.as_str(), AcceptedBy::User, &ui)
        .unwrap()
    else {
        panic!("expected the job to finish");
    };
    assert_eq!(finished.state, OutcomeState::Landed, "{}", finished.reason);
    (fx, proposal.job)
}

/// Runs the binary in `cwd`, with a state directory and home of its own.
fn athena_in(cwd: &Path, args: &[&str]) -> Output {
    let scratch = tempfile::tempdir().unwrap();
    std::process::Command::new(env!("CARGO_BIN_EXE_athena"))
        .current_dir(cwd)
        .args(args)
        .env("HOME", scratch.path())
        .env("ATHENA_STATE_DIR", scratch.path().join("state"))
        .env_remove("CI")
        .env_remove("ATHENA_NESTED")
        .env_remove("ATHENA_SCRATCH_DIR")
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn status_snapshot_reads_records_and_history_from_the_snapshot_with_no_records_in_the_current_directory()
 {
    let (fx, job) = landed();
    let head = fx.head();
    let out = tempfile::tempdir().unwrap();
    let dir = out.path().join("evidence");
    snapshot::write(&fx.git, &head, &dir).unwrap();
    let _guard = Unlock(dir.clone());

    // The directory the command runs in is empty: no repository and no records/.
    let cwd = tempfile::tempdir().unwrap();
    assert!(!cwd.path().join("records").exists());
    assert!(!cwd.path().join(".git").exists());
    let dir_arg = dir.to_str().unwrap();

    // Without the option the command needs a repository, so the success below is the option's.
    let plain = athena_in(cwd.path(), &["status"]);
    assert!(
        !plain.status.success(),
        "status without --snapshot ran with no repository: {}",
        text(&plain.stdout)
    );

    // JSON: the job counts and the throughput are those of the repository's own status.
    let run = athena_in(cwd.path(), &["status", "--snapshot", dir_arg, "--json"]);
    assert!(run.status.success(), "{}", text(&run.stderr));
    let json: serde_json::Value = serde_json::from_slice(&run.stdout).unwrap();
    let view = fx.athena(Scenario::default().script()).status().unwrap();
    assert_eq!(
        json["jobs_by_state"],
        serde_json::to_value(&view.jobs_by_state).unwrap()
    );
    assert_eq!(json["jobs_by_state"]["landed"], 1);
    // Compared as parsed text on both sides: the parser without exact float round trips reads the last digit
    // of a float differently from the value it was printed from.
    let expected: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&view.throughput).unwrap()).unwrap();
    assert_eq!(json["throughput"], expected);
    assert!(
        json["throughput"].is_object(),
        "a landed job has throughput"
    );
    assert_eq!(json["base"], head.as_str());
    let log = fx
        .git
        .run(["log", "--first-parent", "--format=%H", &head])
        .unwrap();
    let hashes: Vec<&str> = log.lines().collect();
    assert_eq!(json["history"]["commits"], hashes.len());
    assert_eq!(json["history"]["newest"]["hash"], hashes[0]);
    assert_eq!(json["history"]["oldest"]["hash"], *hashes.last().unwrap());
    assert!(json["invalid_records"].as_array().unwrap().is_empty());

    // Plain text: the history summary and the throughput section `athena status` prints.
    let run = athena_in(cwd.path(), &["status", "--snapshot", dir_arg]);
    assert!(run.status.success(), "{}", text(&run.stderr));
    let shown = text(&run.stdout);
    let throughput = Throughput::compute(&Records::load(&fx.git, &head).unwrap())
        .expect("a landed job")
        .text();
    assert!(shown.contains(&throughput), "{shown}");
    assert!(shown.contains("1 landed"), "{shown}");
    assert!(
        shown.contains(&format!("history     {} first-parent", hashes.len())),
        "{shown}"
    );
    assert!(shown.contains(&head[..12]), "{shown}");
    assert!(
        !shown.contains(job.as_str()),
        "the summary lists counts, not jobs"
    );
}

#[test]
fn status_snapshot_names_a_missing_directory_and_refuses_a_job_argument() {
    let cwd = tempfile::tempdir().unwrap();
    let missing = cwd.path().join("no-such-snapshot");
    let run = athena_in(
        cwd.path(),
        &["status", "--snapshot", missing.to_str().unwrap()],
    );
    assert!(!run.status.success());
    let message = text(&run.stderr);
    assert!(message.contains(missing.to_str().unwrap()), "{message}");

    // A directory that is not a snapshot says what is missing.
    let empty = tempfile::tempdir().unwrap();
    let run = athena_in(
        cwd.path(),
        &["status", "--snapshot", empty.path().to_str().unwrap()],
    );
    assert!(!run.status.success());
    assert!(
        text(&run.stderr).contains("history.json"),
        "{}",
        text(&run.stderr)
    );

    // One job's trace and a snapshot cannot be asked for together.
    let run = athena_in(
        cwd.path(),
        &[
            "status",
            "latest",
            "--snapshot",
            empty.path().to_str().unwrap(),
        ],
    );
    assert!(!run.status.success());
    assert!(
        text(&run.stderr).contains("cannot be used with"),
        "{}",
        text(&run.stderr)
    );
}
