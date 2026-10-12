//! The link from a job to the one before it in a chain of jobs (`previous_job` in `athena.candidate/2`).
//!
//! The core derives it; the user supplies nothing. It is the latest job landed in the base commit's own
//! history: the job named by the most recent commit that added a landed outcome. Hand changes and
//! records-only commits add no landed outcome, so they are skipped and do not reset the chain. A job that
//! has not landed is never named.

use anyhow::Result;
use athena_data::JobId;
use athena_data::records::{AnyRecord, OutcomeState};
use athena_git::Git;

/// The latest job that landed in the history of `base` (the commit and its ancestors), if any.
///
/// When one commit lands several jobs (a squash of a chain), the job whose id sorts last is named.
pub fn latest_landed_job(git: &Git, base: &str) -> Result<Option<JobId>> {
    let mut commits = git.rev_list_reverse(base)?;
    commits.reverse();
    for commit in commits {
        let mut landed: Vec<JobId> = Vec::new();
        for path in git.added_paths(&commit)? {
            let Some(job) = outcome_job(&path) else {
                continue;
            };
            let Some(bytes) = git.show_file(&commit, &path)? else {
                continue;
            };
            // Only the state and job are read, so the record's other validity is the gate's to judge.
            if let Ok(AnyRecord::Outcome(o)) = serde_json::from_slice::<AnyRecord>(&bytes)
                && o.state == OutcomeState::Landed
                && o.job == job
            {
                landed.push(job);
            }
        }
        if let Some(job) = landed.into_iter().max() {
            return Ok(Some(job));
        }
    }
    Ok(None)
}

/// The request job a path names when it is `<job dir>/outcome.json` or `<job dir>/outcome-landed.json`.
fn outcome_job(path: &str) -> Option<JobId> {
    let (dir, name) = path.rsplit_once('/')?;
    if name != "outcome.json" && name != "outcome-landed.json" {
        return None;
    }
    let job = JobId::parse(dir.rsplit('/').next()?).ok()?;
    (job.is_request() && dir == job.dir()).then_some(job)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const OLD: &str = "20261009-101010-aaaa";
    const NEW: &str = "20261010-101010-bbbb";

    fn outcome(job: &str, state: &str) -> String {
        format!(
            r#"{{
  "schema": "athena.outcome/1",
  "job": "{job}",
  "state": "{state}",
  "reason": "test",
  "at": "2026-10-10T10:10:10Z",
  "totals": null,
  "branch": null,
  "base_commit": null,
  "candidate_commit": null,
  "after_blocked": false,
  "through_commit": null
}}
"#
        )
    }

    fn put(dir: &std::path::Path, job: &str, name: &str, text: &str) {
        let id = JobId::parse(job).unwrap();
        let path = dir.join(id.dir()).join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn commit(git: &Git, message: &str) -> String {
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "--allow-empty", "-m", message])
            .unwrap();
        git.head().unwrap().unwrap()
    }

    fn repo() -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        fs::write(dir.path().join("README.md"), "hello\n").unwrap();
        commit(&git, "init");
        (dir, git)
    }

    #[test]
    fn no_landed_job_means_no_previous_job() {
        let (dir, git) = repo();
        let head = git.head().unwrap().unwrap();
        assert_eq!(latest_landed_job(&git, &head).unwrap(), None);
        // A job that was rejected, not landed, is not named.
        put(dir.path(), OLD, "outcome.json", &outcome(OLD, "rejected"));
        let head = commit(&git, "rejected job");
        assert_eq!(latest_landed_job(&git, &head).unwrap(), None);
    }

    #[test]
    fn the_latest_landed_job_is_named_and_hand_changes_are_skipped() {
        let (dir, git) = repo();
        put(dir.path(), OLD, "outcome.json", &outcome(OLD, "landed"));
        commit(&git, "lands the old job");
        let first = git.head().unwrap().unwrap();
        assert_eq!(
            latest_landed_job(&git, &first).unwrap(),
            Some(JobId::parse(OLD).unwrap())
        );

        // A hand change adds no landed outcome, so the old job is still the latest.
        fs::write(dir.path().join("README.md"), "changed\n").unwrap();
        let hand = commit(&git, "hand change");
        assert_eq!(
            latest_landed_job(&git, &hand).unwrap(),
            Some(JobId::parse(OLD).unwrap())
        );

        // A job landed after a block is `outcome-landed.json`, and the newer job is named.
        put(
            dir.path(),
            NEW,
            "outcome-landed.json",
            &outcome(NEW, "landed"),
        );
        let newest = commit(&git, "lands the new job");
        assert_eq!(
            latest_landed_job(&git, &newest).unwrap(),
            Some(JobId::parse(NEW).unwrap())
        );
        // Asking at an earlier base still gives the earlier answer.
        assert_eq!(
            latest_landed_job(&git, &hand).unwrap(),
            Some(JobId::parse(OLD).unwrap())
        );
    }
}
