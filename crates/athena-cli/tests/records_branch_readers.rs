//! D111: after the cutover, main's HEAD has no `records/`, so live readers read the worktree of the
//! `athena/records` branch (`athena_data::paths::records_worktree`) and fall back to the in-tree `records/`
//! only when that worktree is absent.
//!
//! Each test builds a real git repository. The worktree is created at the path the production code derives
//! from the repository's git directory, which is unique per temporary repository, and removed afterwards.

mod common;

use athena_core::introspect::read_records;
use athena_core::store::Records;
use athena_data::JobId;
use athena_data::paths::{records_root, records_worktree};
use athena_data::records::{AnyRecord, Job, JobKind, Outcome, OutcomeState, Owner};
use athena_gate::records::RecordSet;
use athena_git::Git;
use common::*;
use std::path::{Path, PathBuf};
use std::process::Command;

const JOB_A: &str = "20261010-153012-aaaa";
const JOB_B: &str = "20261011-153012-bbbb";

fn job(id: &str) -> AnyRecord {
    AnyRecord::Job(Box::new(Job {
        id: JobId::parse(id).unwrap(),
        kind: JobKind::Request,
        title: format!("Job {id}"),
        class_of_work: None,
        capability: None,
        accepted_reading: None,
        accepted_by: None,
        owner: Owner {
            role: "athena".into(),
            capability: None,
            model: None,
            note: None,
        },
        verifier_model: None,
        caps: None,
        base_commit: None,
        config_sha256: None,
        defaults_sha256: None,
        athena: None,
        driver: None,
        brief_sha256: None,
        dispatched_at: None,
        request_path: None,
        request_sha256: None,
        cost: None,
        tokens: None,
        note: None,
        pin: None,
    }))
}

fn outcome(id: &str) -> AnyRecord {
    AnyRecord::Outcome(Box::new(Outcome {
        job: JobId::parse(id).unwrap(),
        state: OutcomeState::Landed,
        reason: "landed".into(),
        at: "2026-10-11T16:00:00Z".parse().unwrap(),
        totals: None,
        branch: None,
        base_commit: None,
        candidate_commit: None,
        after_blocked: false,
        through_commit: None,
    }))
}

fn put(root: &Path, records: &[AnyRecord]) {
    for record in records {
        let path = root.join(record.canonical_path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, record.to_json()).unwrap();
    }
}

fn commit_all(git: &Git, message: &str) {
    git.run(["add", "-A"]).unwrap();
    git.run(["commit", "-q", "--allow-empty", "-m", message])
        .unwrap();
}

/// A repository whose HEAD holds `in_tree` records under `records/` (none when empty).
fn repo(in_tree: &[AnyRecord]) -> (tempfile::TempDir, Git) {
    let Fixture { dir, git, state } = Fixture::new();
    drop(state);
    put(dir.path(), in_tree);
    commit_all(&git, "Add records");
    (dir, git)
}

/// The records worktree of a repository, removed when dropped.
struct Worktree(PathBuf);

impl Worktree {
    fn create(git: &Git, records: &[AnyRecord]) -> Worktree {
        let path = records_worktree(&git.common_dir().unwrap());
        put(&path, records);
        Worktree(path)
    }
}

impl Drop for Worktree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn job_ids(records: &Records) -> Vec<String> {
    records.jobs.keys().cloned().collect()
}

#[test]
fn load_current_reads_the_worktree_where_head_has_no_records() {
    let (_dir, git) = repo(&[]);
    let _wt = Worktree::create(&git, &[job(JOB_B)]);
    let head = git.head().unwrap().unwrap();
    assert_eq!(
        job_ids(&Records::load_current(&git, &head).unwrap()),
        [JOB_B]
    );
    let set = RecordSet::load_current(&git, &head).unwrap();
    assert_eq!(set.records().count(), 1);
    // Negative control: the old HEAD-based load finds nothing.
    assert!(Records::load(&git, &head).unwrap().jobs.is_empty());
    assert_eq!(RecordSet::load(&git, &head).unwrap().files.len(), 0);
}

#[test]
fn the_worktree_wins_over_different_in_tree_records() {
    let (dir, git) = repo(&[job(JOB_A), outcome(JOB_A)]);
    let _wt = Worktree::create(&git, &[job(JOB_B), outcome(JOB_B)]);
    let head = git.head().unwrap().unwrap();
    assert_eq!(
        job_ids(&Records::load_current(&git, &head).unwrap()),
        [JOB_B]
    );
    let paths: Vec<_> = RecordSet::load_current(&git, &head)
        .unwrap()
        .files
        .iter()
        .map(|f| f.path.clone())
        .collect();
    assert!(
        paths.iter().all(|p| p.contains(JOB_B)),
        "unexpected {paths:?}"
    );
    // The file reader (introspection, backtest, proposals) takes the same root.
    let root = records_root(&git.common_dir().unwrap(), dir.path());
    let past = read_records(&root).unwrap();
    assert!(past.jobs.contains(JOB_B) && !past.jobs.contains(JOB_A));
    // The old HEAD-based load still returns the in-tree records.
    assert_eq!(job_ids(&Records::load(&git, &head).unwrap()), [JOB_A]);
}

#[test]
fn without_a_worktree_every_reader_falls_back_to_the_in_tree_records() {
    let (dir, git) = repo(&[job(JOB_A), outcome(JOB_A)]);
    let head = git.head().unwrap().unwrap();
    assert!(!records_worktree(&git.common_dir().unwrap()).exists());
    let current = Records::load_current(&git, &head).unwrap();
    assert_eq!(
        job_ids(&current),
        job_ids(&Records::load(&git, &head).unwrap())
    );
    assert_eq!(job_ids(&current), [JOB_A]);
    assert_eq!(
        RecordSet::load_current(&git, &head).unwrap().files.len(),
        RecordSet::load(&git, &head).unwrap().files.len()
    );
    let root = records_root(&git.common_dir().unwrap(), dir.path());
    assert_eq!(root, dir.path());
    assert!(read_records(&root).unwrap().jobs.contains(JOB_A));
    // A worktree that holds no records/ is not a worktree of records.
    let empty = records_worktree(&git.common_dir().unwrap());
    std::fs::create_dir_all(&empty).unwrap();
    let _wt = Worktree(empty);
    assert_eq!(
        job_ids(&Records::load_current(&git, &head).unwrap()),
        [JOB_A]
    );
}

/// Runs `athena status <args>` in the repository with the worktree under a scratch state directory.
fn status(
    dir: &Path,
    git: &Git,
    worktree: Option<&[AnyRecord]>,
    args: &[&str],
) -> std::process::Output {
    let state = tempfile::tempdir().unwrap();
    if let Some(records) = worktree {
        let common = git.common_dir().unwrap();
        let real = athena_data::paths::repo_state_dir(&common);
        let relative = real.strip_prefix(athena_data::paths::state_root()).unwrap();
        put(
            &state.path().join(relative).join("records-worktree"),
            records,
        );
    }
    Command::new(env!("CARGO_BIN_EXE_athena"))
        .arg("status")
        .args(args)
        .env_remove("ATHENA_NESTED")
        .env_remove("ATHENA_SCRATCH_DIR")
        .env("ATHENA_STATE_DIR", state.path())
        .current_dir(dir)
        .output()
        .unwrap()
}

#[test]
fn status_reads_the_worktree_and_matches_the_in_tree_run() {
    let records = [job(JOB_B), outcome(JOB_B)];
    // The records at HEAD, no worktree: the fallback.
    let (in_tree_dir, in_tree_git) = repo(&records);
    let before = status(in_tree_dir.path(), &in_tree_git, None, &[JOB_B, "--json"]);
    assert!(
        before.status.success(),
        "{}",
        String::from_utf8_lossy(&before.stderr)
    );
    // HEAD without records, the same records in the worktree.
    let (dir, git) = repo(&[]);
    let after = status(dir.path(), &git, Some(&records), &[JOB_B, "--json"]);
    assert!(
        after.status.success(),
        "{}",
        String::from_utf8_lossy(&after.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&before.stdout).replace(in_tree_dir.path().to_str().unwrap(), "R"),
        String::from_utf8_lossy(&after.stdout).replace(dir.path().to_str().unwrap(), "R")
    );
    // Negative control: without the worktree the same HEAD does not know the job.
    let missing = status(dir.path(), &git, None, &[JOB_B, "--json"]);
    assert!(!missing.status.success());
    // The overview and a worktree that differs from HEAD also succeed.
    assert!(
        status(dir.path(), &git, Some(&records), &[])
            .status
            .success()
    );
    assert!(
        status(in_tree_dir.path(), &in_tree_git, None, &[])
            .status
            .success()
    );
}

#[test]
fn status_trace_and_mistakes_read_the_worktree_not_the_in_tree_records() {
    let fx = Fixture::new();
    put(fx.dir.path(), &[job(JOB_A), outcome(JOB_A)]);
    commit_all(&fx.git, "Add records");
    let athena = fx.athena(Box::new(|_| unreachable!("no model in this test")));
    let _wt = Worktree::create(&fx.git, &[job(JOB_B), outcome(JOB_B)]);
    let view = athena.status().unwrap();
    let ids: Vec<_> = view
        .recent
        .iter()
        .chain(&view.open)
        .map(|j| j.id.as_str())
        .collect();
    assert!(ids.contains(&JOB_B), "{ids:?}");
    assert!(!ids.contains(&JOB_A), "{ids:?}");
    assert_eq!(view.jobs_by_state.values().sum::<usize>(), 1);
    // trace resolves a job reference against the worktree only.
    assert_eq!(athena.trace(JOB_B).unwrap().id, JOB_B);
    assert!(athena.trace(JOB_A).is_err());
    assert!(athena.mistakes().is_ok());
}

#[test]
fn land_pr_resolves_the_job_from_the_worktree() {
    use athena_core::gh::{GhFailure, Poll, Runner};
    use athena_data::records::{Candidate, Verdict, Verification};

    struct Refusing;
    impl Runner for Refusing {
        fn gh(&self, args: &[String]) -> anyhow::Result<String> {
            Err(GhFailure {
                call: args.get(3).cloned().unwrap_or_default(),
                status_code: Some(500),
                text: "no host in this test".into(),
            }
            .into())
        }
    }
    use athena_core::Quiet;
    fn poll() -> Poll<'static> {
        Poll {
            attempts: 1,
            delay: std::time::Duration::ZERO,
            sleep: &|_| {},
        }
    }

    let id = JobId::parse(JOB_B).unwrap();
    let verification = AnyRecord::Verification(Box::new(Verification {
        job: id.clone(),
        candidate_commit: "c".repeat(40),
        verified_fingerprint: "f".repeat(64),
        tool_config_from_base: vec![],
        checks: vec![],
        check_cleanup: vec![],
        verifier_invocation: None,
        criteria: vec![],
        findings: vec![],
        request_reading_mismatch: None,
        agent_verdict: Some(Verdict::Pass),
        final_verdict: Verdict::Pass,
        reasons: vec![],
        inputs: vec![],
        summary: None,
    }));
    let candidate = AnyRecord::Candidate(Box::new(Candidate {
        job: id.clone(),
        base_commit: "b".repeat(40),
        base_fingerprint: None,
        previous_job: None,
        candidate_commit: Some("c".repeat(40)),
        candidate_tree: None,
        changes: vec![],
        fingerprint: None,
        violations: vec![],
        producer_report: None,
    }));
    let fx = Fixture::new();
    let athena = fx.athena(Box::new(|_| unreachable!("no model in this test")));
    // HEAD has no records and there is no worktree: the job is unknown.
    let err = athena
        .land_pr(JOB_B, &Refusing, poll(), &Quiet)
        .unwrap_err();
    let text = format!("{err:#}");
    assert!(!text.contains("origin"), "{text}");
    // With the worktree the job resolves, passes the verification and candidate checks, and stops at the
    // next step: this repository has no `origin` remote.
    let _wt = Worktree::create(&fx.git, &[job(JOB_B), verification, candidate]);
    let err = athena
        .land_pr(JOB_B, &Refusing, poll(), &Quiet)
        .unwrap_err();
    let text = format!("{err:#}");
    assert!(
        !text.contains("no verification") && !text.contains("no candidate"),
        "{text}"
    );
    assert!(text.contains("origin"), "expected the remote step: {text}");
}
