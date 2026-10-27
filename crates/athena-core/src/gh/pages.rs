//! The Pages watch after a merge.

use super::{GhClient, Poll};
use crate::Ui;
use anyhow::Result;
use athena_data::records::HostOperation;
use std::time::Duration;

/// The workflow file whose runs are the Pages deploy.
pub const PAGES_WORKFLOW: &str = "pages.yml";

/// One run of the Pages workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PagesRun {
    pub id: u64,
    /// `queued`, `in_progress` or `completed`.
    pub status: String,
    /// Set once the run is completed: `success`, `timed_out`, `cancelled`, and so on.
    pub conclusion: Option<String>,
    /// 1 for the first try; a re-run raises it.
    pub attempt: u64,
}

/// The two host calls the Pages watch makes. Both go through the retrying wrapper of [`GhClient`].
pub trait PagesHost {
    /// The Pages workflow runs for one commit.
    fn pages_runs(&self, commit: &str) -> Result<Vec<PagesRun>>;
    /// Asks the host to run one workflow run again.
    fn rerun_pages(&self, run_id: u64) -> Result<()>;
}

impl PagesHost for GhClient<'_> {
    fn pages_runs(&self, commit: &str) -> Result<Vec<PagesRun>> {
        // Reads of runs are recorded under the checks operation: the host operation list is closed.
        let reply = self.api(
            HostOperation::ReadChecks,
            "GET",
            &self.repo_path(&format!(
                "actions/workflows/{PAGES_WORKFLOW}/runs?head_sha={commit}&per_page=20"
            )),
            &[],
        )?;
        let runs = reply["workflow_runs"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        Ok(runs
            .iter()
            .filter_map(|r| {
                Some(PagesRun {
                    id: r["id"].as_u64()?,
                    status: r["status"].as_str().unwrap_or("").to_string(),
                    conclusion: r["conclusion"].as_str().map(str::to_string),
                    attempt: r["run_attempt"].as_u64().unwrap_or(1),
                })
            })
            .collect())
    }

    fn rerun_pages(&self, run_id: u64) -> Result<()> {
        self.api(
            HostOperation::ReadChecks,
            "POST",
            &self.repo_path(&format!("actions/runs/{run_id}/rerun")),
            &[],
        )?;
        Ok(())
    }
}

/// What the Pages watch saw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PagesReport {
    /// The run watched; none when the merged commit has no Pages run.
    pub run: Option<u64>,
    /// How many times the run was re-run: 0 or 1.
    pub reruns: u32,
    /// The last conclusion seen; none when no run, or the run did not finish within the bound.
    pub conclusion: Option<String>,
}

/// The wait for one Pages try: 60 looks, 10 seconds apart, about ten minutes. A re-run gets the same wait.
pub const PAGES_WAIT_LOOKS: u32 = 60;
pub const PAGES_WAIT_DELAY: Duration = Duration::from_secs(10);

/// How many times to look for a Pages run that has not appeared yet.
const PAGES_APPEAR_LOOKS: u32 = 3;

/// Waits (at most `poll.attempts` looks) for the newest run of the commit to be completed, and for it to be
/// at least `min_attempt` tries in.
fn wait_pages_run(
    host: &dyn PagesHost,
    commit: &str,
    poll: &Poll,
    min_attempt: u64,
    looks: u32,
) -> Result<Option<PagesRun>> {
    let looks = looks.max(1);
    let mut latest = None;
    for n in 0..looks {
        latest = host
            .pages_runs(commit)?
            .into_iter()
            .max_by_key(|r| r.id)
            .filter(|r| r.attempt >= min_attempt);
        if latest.as_ref().is_some_and(|r| r.status == "completed") {
            break;
        }
        if n + 1 < looks {
            (poll.sleep)(poll.delay);
        }
    }
    Ok(latest)
}

/// After a merge: finds the Pages run for the merged commit, waits for it (bounded by `poll`), and when it
/// timed out or was cancelled asks the host to run it again, once, and waits for that try too. Every step is
/// reported through `ui`. A commit with no Pages run is not an error. The caller decides what a failure of
/// the watch itself means; the landing is already done.
pub fn watch_pages(
    host: &dyn PagesHost,
    merged: &str,
    poll: &Poll,
    ui: &dyn Ui,
) -> Result<PagesReport> {
    let none = PagesReport {
        run: None,
        reruns: 0,
        conclusion: None,
    };
    let Some(run) = wait_pages_run(host, merged, poll, 1, PAGES_APPEAR_LOOKS.min(poll.attempts))?
    else {
        ui.line("no Pages run found for the merged commit; nothing to watch");
        return Ok(none);
    };
    let run = if run.status == "completed" {
        run
    } else {
        match wait_pages_run(host, merged, poll, 1, poll.attempts)? {
            Some(run) if run.status == "completed" => run,
            _ => {
                ui.line(&format!(
                    "Pages run {} is still {} after waiting; not waiting longer",
                    run.id, run.status
                ));
                return Ok(PagesReport {
                    run: Some(run.id),
                    ..none
                });
            }
        }
    };
    let conclusion = run.conclusion.clone().unwrap_or_default();
    let report = |reruns, conclusion: &str| PagesReport {
        run: Some(run.id),
        reruns,
        conclusion: Some(conclusion.to_string()),
    };
    match conclusion.as_str() {
        "success" => {
            ui.line(&format!("Pages run {} succeeded", run.id));
            Ok(report(0, &conclusion))
        }
        "timed_out" | "cancelled" => {
            ui.line(&format!(
                "Pages run {} concluded {conclusion}; re-running it once",
                run.id
            ));
            host.rerun_pages(run.id)?;
            let again = wait_pages_run(host, merged, poll, run.attempt + 1, poll.attempts)?
                .filter(|r| r.id == run.id && r.status == "completed");
            let Some(again) = again else {
                ui.line(&format!(
                    "Pages run {} was re-run and has not finished; not re-running it again",
                    run.id
                ));
                return Ok(report(1, &conclusion));
            };
            let second = again.conclusion.unwrap_or_default();
            if second == "success" {
                ui.line(&format!("Pages run {} succeeded after its re-run", run.id));
            } else {
                ui.line(&format!(
                    "Pages run {} concluded {second} after its re-run; not re-running it again",
                    run.id
                ));
            }
            Ok(report(1, &second))
        }
        other => {
            ui.line(&format!(
                "Pages run {} concluded {other}; only timed_out and cancelled runs are re-run",
                run.id
            ));
            Ok(report(0, other))
        }
    }
}
