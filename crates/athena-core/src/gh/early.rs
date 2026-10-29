//! The early pull request and the CI wait: the early step of `go` opens or updates the job's pull request at
//! verification start, and landing waits, boundedly, for the early commit's run and for the pull request's
//! checks.

#[cfg(test)]
use super::early_seam;
use super::{
    Checks, CiRunState, GhCli, GhClient, GitRemote, HOST, LandPr, NewPr, PrHost, Remote,
    parse_remote, work_branch,
};
use crate::pr::{
    Ancestry, LinkState, PrLink, PushRequest, ReplaceRequest, check_push, check_replace,
    clear_early, load_early, load_early_link, load_link, store_early, store_early_link,
};
use crate::state::StateDir;
use crate::{Athena, Ui};
use anyhow::Result;
use athena_data::JobId;
use std::time::Duration;

/// How often land looks at the early commit's runs while it waits.
const EARLY_RUN_STEP: Duration = Duration::from_secs(20);
/// The check run land waits for on the early commit: the job of the CI workflow whose pass the host caches.
/// It must match the job name in the CI workflow.
pub(super) const EARLY_RUN_JOB: &str = "fmt, clippy, tests";

/// Replaces the job's early-pushed head `early_head` with the landing commit: allowed by [`check_replace`]
/// for the job's own branch only, and pinned to `early_head` on the remote. Forgets the early head after.
///
/// Before the push it waits, boundedly, for the early commit's CI run to finish (see
/// [`wait_for_early_run`]), so the landing commit's run finds the cached pass. Returns the note saying how
/// long it waited and why it stopped.
pub(super) fn replace_early(job: &LandPr, early_head: &str) -> Result<String> {
    let note = wait_for_early_run(job, early_head);
    let allowed = work_branch(job.job);
    check_replace(
        job.ancestry,
        &ReplaceRequest {
            allowed_branch: &allowed,
            branch: job.branch,
            local_head: job.commit,
            remote_head: Some(early_head),
        },
    )?;
    job.remote.replace(job.branch, job.commit, early_head)?;
    clear_early(job.state, job.job)?;
    Ok(note)
}

/// Waits for the check run named [`EARLY_RUN_JOB`] of the early commit to complete, at most
/// [`PrHost::early_run_wait`], looking every [`EARLY_RUN_STEP`] through the poll's sleep. Only that job
/// decides: a candidate is not a landing, so its other jobs (the full history, the pull request head) always
/// fail there and say nothing about the job the host caches. That job, once passed, is cached by the host, so
/// the landing commit's run (the same tree) is short. The job failed, missing, or an unreadable reply end the
/// wait at once: nothing is gained by waiting, and the push and the merge go on as without a wait. Returns a
/// note with the seconds waited and why the wait stopped.
fn wait_for_early_run(job: &LandPr, early_head: &str) -> String {
    let bound = job.host.early_run_wait();
    let short: String = early_head.chars().take(12).collect();
    let note = |waited: Duration, why: &str| {
        format!(
            "waited {}s for the early run of {short} before pushing the landing commit: {why}",
            waited.as_secs()
        )
    };
    let mut waited = Duration::ZERO;
    loop {
        match job.host.check_run(early_head, EARLY_RUN_JOB) {
            Err(problem) => {
                return note(waited, &format!("its runs could not be read ({problem:#})"));
            }
            Ok(None) => {
                return note(waited, &format!("it has no {EARLY_RUN_JOB:?} check run"));
            }
            Ok(Some(run)) if run.failed() => {
                return note(
                    waited,
                    &format!("{EARLY_RUN_JOB} failed, so the landing run decides"),
                );
            }
            Ok(Some(run)) if run.status == "completed" => {
                return note(
                    waited,
                    if waited.is_zero() {
                        "already completed"
                    } else {
                        "completed"
                    },
                );
            }
            Ok(_) => {}
        }
        if waited >= bound {
            return note(waited, "timed out, pushed anyway");
        }
        (job.poll.sleep)(EARLY_RUN_STEP);
        waited += EARLY_RUN_STEP;
    }
}

/// What the wait for checks found.
pub(super) struct Waited {
    pub(super) checks: Checks,
    pub(super) reruns: u32,
    pub(super) failing_test: Option<String>,
}

/// The first test name found in the logs of the failed runs; a failed lookup names nothing.
fn name_failure(host: &dyn PrHost, runs: &[CiRunState]) -> Option<String> {
    runs.iter()
        .filter(|r| r.failed())
        .find_map(|r| host.failing_test(r.id).ok().flatten())
}

/// Waits for the checks, in at most three steps that share one look budget of about twice `poll.attempts`.
///
/// 1. Look at the checks up to `attempts` times until they are not pending.
/// 2. If they are red, the failing run may still have sibling jobs running, and the host refuses to re-run a
///    run in progress. So look at the commit's runs (the rest of the `attempts` looks, at least one) until
///    none is in progress, and name the failing test from the failed job's log.
/// 3. Ask the host to re-run the failed jobs of each failed run, once, and look at the checks up to
///    `attempts` more times. A read is ignored until every re-run run shows a higher `run_attempt`, since
///    right after the request the commit still shows the old red. Green or red is then final; anything else
///    is pending. There is no second re-run.
pub(super) fn wait_for_checks(job: &LandPr, head: &str) -> Result<Waited> {
    let attempts = job.poll.attempts.max(1);
    let nap = |more: bool| {
        if more {
            (job.poll.sleep)(job.poll.delay);
        }
    };
    let mut seen = Checks::Pending;
    let mut looks = 0;
    for n in 0..attempts {
        seen = job.host.checks(head)?;
        looks += 1;
        if seen != Checks::Pending {
            break;
        }
        nap(n + 1 < attempts);
    }
    let done = |checks, reruns, failing_test| {
        Ok(Waited {
            checks,
            reruns,
            failing_test,
        })
    };
    if seen != Checks::Red {
        return done(seen, 0, None);
    }
    // Step 2: wait for the runs to complete.
    let run_looks = (attempts - looks.min(attempts)).max(1);
    let mut runs = Vec::new();
    for n in 0..run_looks {
        runs = job.host.ci_runs(head)?;
        if runs.iter().all(|r| r.status == "completed") {
            break;
        }
        nap(n + 1 < run_looks);
    }
    let failing_test = name_failure(job.host, &runs);
    if runs.iter().any(|r| r.status != "completed") {
        return done(Checks::Red, 0, failing_test);
    }
    // Step 3: re-run each failed run once.
    let mut rerun: Vec<(u64, Option<u64>)> = Vec::new();
    for run in runs.iter().filter(|r| r.failed()) {
        if job.host.rerun_failed(run.id).is_ok() {
            rerun.push((run.id, run.attempt));
        }
    }
    if rerun.is_empty() {
        // Nothing to re-run (a red the runs do not explain), or the host refused: the refusal is in the log.
        return done(Checks::Red, 0, failing_test);
    }
    let mut seen = Checks::Pending;
    let mut last_runs = Vec::new();
    for n in 0..attempts {
        let now = job.host.ci_runs(head)?;
        let read = job.host.checks(head)?;
        let restarted = rerun.iter().all(|(id, before)| {
            match (
                before,
                now.iter().find(|r| r.id == *id).and_then(|r| r.attempt),
            ) {
                (Some(before), Some(after)) => after > *before,
                // The host does not say which try it is on: accept after the second look.
                _ => n >= 1,
            }
        });
        last_runs = now;
        seen = if restarted { read } else { Checks::Pending };
        if seen != Checks::Pending {
            break;
        }
        nap(n + 1 < attempts);
    }
    let failing_test = if seen == Checks::Red {
        name_failure(job.host, &last_runs).or(failing_test)
    } else {
        None
    };
    done(seen, 1, failing_test)
}

/// What the early step needs: the job's candidate, and the seams it pushes and opens through.
pub struct EarlyPr<'a> {
    pub state: &'a StateDir,
    pub job: &'a JobId,
    /// The candidate commit, pushed as the job's branch head.
    pub commit: &'a str,
    pub title: &'a str,
    /// `owner/name` on the host.
    pub repository: &'a str,
    pub ancestry: &'a dyn Ancestry,
    pub remote: &'a dyn Remote,
    pub host: &'a dyn PrHost,
}

impl std::fmt::Debug for EarlyPr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EarlyPr")
            .field("job", &self.job)
            .field("commit", &self.commit)
            .finish_non_exhaustive()
    }
}

/// Opens or updates the job's pull request at verification start, so the host's checks begin on the
/// candidate's fingerprint while the verifier works. It pushes the candidate to the job's own branch and
/// opens the pull request, or, when one is already linked, moves its branch to the candidate. It never
/// merges, and it records the pushed head first so landing can replace exactly that head later.
pub fn open_early_pr(early: &EarlyPr) -> Result<()> {
    let branch = work_branch(early.job);
    let remote_head = early.remote.remote_head(&branch)?;
    let stored = load_early(early.state, early.job)?;
    if remote_head.as_deref() != Some(early.commit) {
        match (&stored, &remote_head) {
            // The branch still holds this job's earlier early push: replace it, pinned to that head.
            (Some(old), Some(head)) if old == head => {
                check_replace(
                    early.ancestry,
                    &ReplaceRequest {
                        allowed_branch: &branch,
                        branch: &branch,
                        local_head: early.commit,
                        remote_head: Some(head),
                    },
                )?;
                early.remote.replace(&branch, early.commit, head)?;
            }
            _ => {
                check_push(
                    early.ancestry,
                    &PushRequest {
                        allowed_branch: &branch,
                        branch: &branch,
                        local_head: early.commit,
                        remote_head: remote_head.as_deref(),
                    },
                )?;
                early.remote.push(&branch, early.commit)?;
            }
        }
    }
    // Recorded before the pull request is opened: if opening fails, landing still knows this head is its own.
    store_early(early.state, early.job, early.commit)?;
    let existing = match load_early_link(early.state, early.job)? {
        Some(link) => Some(link),
        None => load_link(early.state, early.job)?,
    };
    if let Some(mut link) = existing {
        link.head = early.commit.to_string();
        return store_early_link(early.state, early.job, &link);
    }
    let base = early.host.default_branch()?;
    let body = format!(
        "Verification in progress for job {}. This pull request is updated when the job lands.",
        early.job.as_str()
    );
    let pr_number = early.host.open_pr(&NewPr {
        title: early.title,
        body: &body,
        head: &branch,
        base: &base,
    })?;
    store_early_link(
        early.state,
        early.job,
        &PrLink {
            pr_number,
            host: HOST.to_string(),
            repository: early.repository.to_string(),
            head: early.commit.to_string(),
            job: early.job.as_str().to_string(),
            state: LinkState::Open,
            merged_commit: None,
        },
    )
}

/// Whether `url` is an https URL on github.com (credentials in the authority are ignored), and its
/// `owner/name`. Anything else, such as a local path that merely contains the host name, is not.
fn https_github_repository(url: &str) -> Option<String> {
    let url = url.trim();
    let rest = url.strip_prefix("https://")?;
    let authority = rest.split('/').next()?;
    let host = authority.rsplit_once('@').map_or(authority, |(_, h)| h);
    if host != HOST {
        return None;
    }
    parse_remote(url).ok().map(|(_, repository)| repository)
}

impl Athena {
    /// The early step of `go`: at verification start, opens or updates the job's pull request through the
    /// host (see [`open_early_pr`]) so CI caches by the candidate's fingerprint. Never merges. A missing
    /// origin, or one that is not an https URL on github.com, is skipped without a word. Any failure is one
    /// warning line and does not change the job's outcome.
    pub(crate) fn early_pr(&self, job: &JobId, commit: &str, title: &str, ui: &dyn Ui) {
        if self.nested.is_some() {
            return;
        }
        let Ok(url) = self.git.run(["remote", "get-url", "origin"]) else {
            return;
        };
        let Some(repository) = https_github_repository(&url) else {
            return;
        };
        if let Err(problem) = self.early_pr_to(job, commit, title, &repository) {
            ui.line(&format!(
                "warning: could not open the pull request early: {problem:#}"
            ));
        }
    }

    fn early_pr_to(&self, job: &JobId, commit: &str, title: &str, repository: &str) -> Result<()> {
        let request = |remote: &dyn Remote, host: &dyn PrHost| {
            open_early_pr(&EarlyPr {
                state: &self.state,
                job,
                commit,
                title,
                repository,
                ancestry: &self.git,
                remote,
                host,
            })
        };
        #[cfg(test)]
        if let Some((remote, host)) = early_seam::current() {
            return request(&*remote, &*host);
        }
        let runner = GhCli;
        let client = GhClient::new(&runner, repository.to_string());
        let remote = GitRemote {
            git: &self.git,
            name: "origin",
        };
        request(&remote, &client)
    }
}
