//! The Host API client: the check, merge-method and workflow-run types, the [`PrHost`] trait, and
//! [`GhClient`], which makes the REST calls through `gh api` for one repository.

use super::transport::clock_entropy;
use super::{GhFailure, HOST, HostErrorLog, Retry, Runner};
use crate::Ui;
use crate::pr::{Host, PrLink};
use crate::reconcile::{CommitHost, MergedPr};
use anyhow::{Context, Result, bail};
use athena_data::records::HostOperation;
use athena_git::CommitText;
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

/// How long land waits, by default, for the early commit's runs (a bit under the run's own length). It stays
/// here, beside [`PrHost::early_run_wait`], so the client does not name the early row.
pub const EARLY_RUN_WAIT: Duration = Duration::from_secs(600);

/// How a pull request's checks stand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Checks {
    /// Every check finished well.
    Green,
    /// A check failed.
    Red,
    /// A check has not finished, or none has reported yet.
    Pending,
}

/// A pull request to open.
#[derive(Debug, Clone, Copy)]
pub struct NewPr<'a> {
    pub title: &'a str,
    pub body: &'a str,
    /// The branch that holds the change.
    pub head: &'a str,
    /// The branch it is to be merged into.
    pub base: &'a str,
}

/// The parts of a pull request that are known before the host names the base branch.
#[derive(Debug, Clone, Copy)]
pub struct PrDraft<'a> {
    pub title: &'a str,
    pub body: &'a str,
    /// The branch that holds the change.
    pub head: &'a str,
}

/// What the host can be asked, besides where a merge landed ([`Host`]).
pub trait PrHost: Host {
    /// The repository's default branch on the host: the base every pull request is opened against.
    fn default_branch(&self) -> Result<String>;
    /// Opens a pull request and returns its number.
    fn open_pr(&self, pr: &NewPr) -> Result<u64>;
    /// The state of the checks on a commit.
    fn checks(&self, commit: &str) -> Result<Checks>;
    /// Asks the host to merge the pull request, guarded by the head commit it was opened with, by a merge
    /// method the repository allows (see [`MergeMethods::choose`]). `text` is the message a squash carries.
    /// Returns the method the host was asked to use, which the landing record names (§5.3).
    fn merge(&self, number: u64, head: &str, text: &CommitText) -> Result<MergeMethod>;
    /// The workflow runs of a commit: whether each is still going, how it ended, and which try it is on.
    fn ci_runs(&self, commit: &str) -> Result<Vec<CiRunState>>;
    /// The latest check run of a commit with the given job name (workflow runs cannot name a job); none when
    /// the commit has no such check run.
    fn check_run(&self, commit: &str, name: &str) -> Result<Option<CiRunState>>;
    /// Asks the host to run the failed jobs of one completed workflow run again. The host refuses while the
    /// run is still in progress.
    fn rerun_failed(&self, run_id: u64) -> Result<()>;
    /// The name of the first test that failed in the latest try of a completed run, read from the failed
    /// job's log; none when the log names none.
    fn failing_test(&self, run_id: u64) -> Result<Option<String>>;
    /// The longest land waits for the early commit's runs to finish before it pushes the landing commit.
    /// Tests override it so they need no real sleeping.
    fn early_run_wait(&self) -> Duration {
        EARLY_RUN_WAIT
    }
}

/// One workflow run of a commit, as the host reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CiRunState {
    pub id: u64,
    /// `queued`, `in_progress` or `completed`.
    pub status: String,
    /// Set once the run is completed: `success`, `failure`, `timed_out`, and so on.
    pub conclusion: Option<String>,
    /// The try the run is on (1 for the first; a re-run raises it); none when the host does not say.
    pub attempt: Option<u64>,
}

impl CiRunState {
    /// A completed run that failed or timed out: the runs whose failed jobs are re-run.
    pub fn failed(&self) -> bool {
        self.status == "completed"
            && matches!(self.conclusion.as_deref(), Some("failure" | "timed_out"))
    }
}

/// Reads the workflow-runs reply into [`CiRunState`]s. Only fixed fields are read.
pub fn parse_ci_runs(reply: &Value) -> Vec<CiRunState> {
    reply["workflow_runs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| {
            Some(CiRunState {
                id: r["id"].as_u64()?,
                status: r["status"].as_str().unwrap_or("").to_string(),
                conclusion: r["conclusion"].as_str().map(str::to_string),
                attempt: r["run_attempt"].as_u64(),
            })
        })
        .collect()
}

/// The latest check run (highest id) with the given name in a check-runs reply, as a [`CiRunState`] without
/// an attempt. Only fixed fields are read.
pub fn parse_check_run(reply: &Value, name: &str) -> Option<CiRunState> {
    reply["check_runs"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|r| r["name"].as_str() == Some(name))
        .filter_map(|r| {
            Some(CiRunState {
                id: r["id"].as_u64()?,
                status: r["status"].as_str().unwrap_or("").to_string(),
                conclusion: r["conclusion"].as_str().map(str::to_string),
                attempt: None,
            })
        })
        .max_by_key(|r| r.id)
}

/// The longest test name kept from a log.
pub(super) const TEST_NAME_LIMIT: usize = 200;

/// The name of the first failed test in a job log: the first `test <name> ... FAILED` line, else the first
/// `---- <name> stdout ----` line. A line may start with a timestamp. A name with a character outside
/// `[A-Za-z0-9_:]`, or longer than [`TEST_NAME_LIMIT`], is dropped, so log text never reaches the report.
pub fn failing_test_name(log: &str) -> Option<String> {
    let valid = |name: &str| {
        !name.is_empty()
            && name.len() <= TEST_NAME_LIMIT
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':')
    };
    let lines: Vec<&str> = log
        .lines()
        .map(|line| {
            let line = line.trim();
            match line.split_once(' ') {
                Some((stamp, rest))
                    if stamp.ends_with('Z')
                        && stamp.contains('T')
                        && stamp.starts_with(|c: char| c.is_ascii_digit()) =>
                {
                    rest.trim_start()
                }
                _ => line,
            }
        })
        .collect();
    lines
        .iter()
        .filter_map(|l| l.strip_prefix("test ")?.strip_suffix(" ... FAILED"))
        .find(|n| valid(n))
        .or_else(|| {
            lines
                .iter()
                .filter_map(|l| l.strip_prefix("---- ")?.strip_suffix(" stdout ----"))
                .find(|n| valid(n))
        })
        .map(str::to_string)
}

/// A way the host can merge a pull request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeMethod {
    /// A merge commit.
    Merge,
    /// One new commit on the base branch holding the whole change.
    Squash,
    /// The branch's commits replayed on the base branch.
    Rebase,
}

impl MergeMethod {
    /// The name the host's merge endpoint takes.
    pub fn as_str(self) -> &'static str {
        match self {
            MergeMethod::Merge => "merge",
            MergeMethod::Squash => "squash",
            MergeMethod::Rebase => "rebase",
        }
    }
}

/// The merge methods a repository allows, as the host reports them
/// (`allow_merge_commit`, `allow_squash_merge`, `allow_rebase_merge`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MergeMethods {
    pub merge: bool,
    pub squash: bool,
    pub rebase: bool,
}

impl MergeMethods {
    /// Reads the three flags from the repository reply. A flag the host does not report counts as not allowed.
    pub fn from_repository(repo: &Value) -> MergeMethods {
        let flag = |name: &str| repo[name].as_bool() == Some(true);
        MergeMethods {
            merge: flag("allow_merge_commit"),
            squash: flag("allow_squash_merge"),
            rebase: flag("allow_rebase_merge"),
        }
    }

    /// The method to request: a merge commit when allowed (it keeps the branch's commits as ancestors), else a
    /// squash, else a rebase. When none is allowed it is an error that names what the host reported.
    pub fn choose(&self) -> Result<MergeMethod> {
        if self.merge {
            Ok(MergeMethod::Merge)
        } else if self.squash {
            Ok(MergeMethod::Squash)
        } else if self.rebase {
            Ok(MergeMethod::Rebase)
        } else {
            bail!(
                "the host allows no merge method for this repository (allow_merge_commit {}, \
                 allow_squash_merge {}, allow_rebase_merge {}); allow one in the repository settings",
                self.merge,
                self.squash,
                self.rebase
            )
        }
    }
}

/// The host client: REST calls through `gh api`, for one repository.
pub struct GhClient<'a> {
    pub runner: &'a dyn Runner,
    /// `owner/name`.
    pub repository: String,
    /// Where every refused call is recorded; none records nothing.
    pub log: Option<&'a HostErrorLog>,
    /// How a transient failure is repeated.
    pub retry: Retry,
    /// Where each retry is reported (one line naming the command, the status and the attempt); none is silent.
    pub ui: Option<&'a dyn Ui>,
}

impl std::fmt::Debug for GhClient<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GhClient")
            .field("repository", &self.repository)
            .finish_non_exhaustive()
    }
}

impl<'a> GhClient<'a> {
    /// A client that records nothing and repeats a transient failure as [`Retry::STANDARD`] says.
    pub fn new(runner: &'a dyn Runner, repository: String) -> Self {
        GhClient {
            runner,
            repository,
            log: None,
            retry: Retry::STANDARD,
            ui: None,
        }
    }

    /// Runs one call. A refusal by the host is recorded in the log, whatever its status. A transient failure
    /// (see [`is_transient`]) is called again, up to the bound of [`Retry`]; any other failure, and the last
    /// transient one, is returned to the caller.
    pub(super) fn api(
        &self,
        operation: HostOperation,
        method: &str,
        path: &str,
        fields: &[(&str, &str)],
    ) -> Result<Value> {
        let text = self.api_text(operation, method, path, fields)?;
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(&text).with_context(|| format!("parsing the reply to {path}"))
    }

    /// The retry and error-log loop of [`GhClient::api`], returning the reply as text (a job log is not JSON).
    fn api_text(
        &self,
        operation: HostOperation,
        method: &str,
        path: &str,
        fields: &[(&str, &str)],
    ) -> Result<String> {
        let mut args: Vec<String> = ["api", "--method", method, path]
            .iter()
            .map(|s| s.to_string())
            .collect();
        for (key, value) in fields {
            args.push("-f".into());
            args.push(format!("{key}={value}"));
        }
        let attempts = self.retry.attempts.max(1);
        let mut attempt = 1;
        let text = loop {
            match self.runner.gh(&args) {
                Ok(text) => break text,
                Err(error) => {
                    let Some(failure) = error.downcast_ref::<GhFailure>() else {
                        return Err(error);
                    };
                    if let Some(log) = self.log {
                        log.record(operation, failure.status_code);
                    }
                    let Some(reason) = failure.transient_reason() else {
                        return Err(error);
                    };
                    if attempt >= attempts {
                        let message = format!(
                            "gh {} failed after {attempt} attempts ({reason}): {}",
                            failure.call, failure.text
                        );
                        return Err(error.context(message));
                    }
                    let wait = self.retry.wait(attempt, clock_entropy());
                    if let Some(ui) = self.ui {
                        ui.line(&format!(
                            "retrying gh {} after {reason} (attempt {attempt} of {attempts} failed; waiting {} ms)",
                            failure.call,
                            wait.as_millis()
                        ));
                    }
                    std::thread::sleep(wait);
                    attempt += 1;
                }
            }
        };
        Ok(text)
    }

    pub(super) fn repo_path(&self, rest: &str) -> String {
        format!("repos/{}/{rest}", self.repository)
    }
}

impl Host for GhClient<'_> {
    fn merged_commit(&self, link: &PrLink) -> Result<Option<String>> {
        let pull = self.api(
            HostOperation::ReadPr,
            "GET",
            &self.repo_path(&format!("pulls/{}", link.pr_number)),
            &[],
        )?;
        if pull["merged"].as_bool() != Some(true) {
            return Ok(None);
        }
        match pull["merge_commit_sha"].as_str() {
            Some(sha) => Ok(Some(sha.to_string())),
            None => bail!(
                "the host reports pull request {} merged but names no merged commit",
                link.pr_number
            ),
        }
    }
}

impl PrHost for GhClient<'_> {
    fn default_branch(&self) -> Result<String> {
        let repo = self.api(
            HostOperation::ReadRepo,
            "GET",
            &format!("repos/{}", self.repository),
            &[],
        )?;
        match repo["default_branch"].as_str() {
            Some(name) if !name.is_empty() => Ok(name.to_string()),
            _ => bail!("the host's reply for the repository names no default branch"),
        }
    }

    fn open_pr(&self, pr: &NewPr) -> Result<u64> {
        let created = self.api(
            HostOperation::CreatePr,
            "POST",
            &self.repo_path("pulls"),
            &[
                ("title", pr.title),
                ("body", pr.body),
                ("head", pr.head),
                ("base", pr.base),
            ],
        )?;
        created["number"]
            .as_u64()
            .context("the host's reply to opening a pull request has no number")
    }

    fn checks(&self, commit: &str) -> Result<Checks> {
        let runs = self.api(
            HostOperation::ReadChecks,
            "GET",
            &self.repo_path(&format!("commits/{commit}/check-runs?per_page=100")),
            &[],
        )?;
        let status = self.api(
            HostOperation::ReadChecks,
            "GET",
            &self.repo_path(&format!("commits/{commit}/status")),
            &[],
        )?;
        Ok(classify_checks(&runs, &status))
    }

    fn merge(&self, number: u64, head: &str, text: &CommitText) -> Result<MergeMethod> {
        let repo = self.api(
            HostOperation::ReadRepo,
            "GET",
            &format!("repos/{}", self.repository),
            &[],
        )?;
        let method = MergeMethods::from_repository(&repo).choose()?;
        let mut fields = vec![("sha", head), ("merge_method", method.as_str())];
        if method == MergeMethod::Squash {
            // The squash message is built by Athena, so the job's trailers stay in its last paragraph.
            fields.push(("commit_title", text.title.as_str()));
            fields.push(("commit_message", text.body.as_str()));
        }
        self.api(
            HostOperation::Merge,
            "PUT",
            &self.repo_path(&format!("pulls/{number}/merge")),
            &fields,
        )?;
        Ok(method)
    }

    fn ci_runs(&self, commit: &str) -> Result<Vec<CiRunState>> {
        let reply = self.api(
            HostOperation::ReadChecks,
            "GET",
            &self.repo_path(&format!("actions/runs?head_sha={commit}&per_page=100")),
            &[],
        )?;
        Ok(parse_ci_runs(&reply))
    }

    fn check_run(&self, commit: &str, name: &str) -> Result<Option<CiRunState>> {
        let reply = self.api(
            HostOperation::ReadChecks,
            "GET",
            &self.repo_path(&format!("commits/{commit}/check-runs?per_page=100")),
            &[],
        )?;
        Ok(parse_check_run(&reply, name))
    }

    fn rerun_failed(&self, run_id: u64) -> Result<()> {
        // Recorded under the checks operation: the host operation list is closed (as for `rerun_pages`).
        self.api(
            HostOperation::ReadChecks,
            "POST",
            &self.repo_path(&format!("actions/runs/{run_id}/rerun-failed-jobs")),
            &[],
        )?;
        Ok(())
    }

    fn failing_test(&self, run_id: u64) -> Result<Option<String>> {
        let jobs = self.api(
            HostOperation::ReadChecks,
            "GET",
            &self.repo_path(&format!(
                "actions/runs/{run_id}/jobs?filter=latest&per_page=100"
            )),
            &[],
        )?;
        let failed = jobs["jobs"].as_array().into_iter().flatten().filter(|j| {
            matches!(
                j["conclusion"].as_str(),
                Some("failure" | "timed_out" | "cancelled")
            )
        });
        for job in failed {
            let Some(id) = job["id"].as_u64() else {
                continue;
            };
            let log = self.api_text(
                HostOperation::ReadChecks,
                "GET",
                &self.repo_path(&format!("actions/jobs/{id}/logs")),
                &[],
            )?;
            if let Some(name) = failing_test_name(&log) {
                return Ok(Some(name));
            }
        }
        Ok(None)
    }
}

/// The name of the CI workflow whose run carries the three checks.
pub const CI_WORKFLOW: &str = "ci";

/// The CI step that runs `fmt`, `clippy` and `test` together (`xtask ci`); its name starts with this.
const CI_CHECK_STEP: &str = "cargo fmt --check";

/// Builds the CI result for one commit from the workflow-runs reply and, for the chosen run, the jobs reply.
/// Only fixed fields are read; log text is never used. The newest run of [`CI_WORKFLOW`] is chosen. The
/// run's own `head_sha` is kept as the SHA, so a run for another commit stays visible to the SHA check in
/// [`crate::ci_result::resolve`]. The platform reports one step for all three checks, so its conclusion is
/// given to each; per-check time is not reported, so each step carries 0 and the run's time is the total.
pub fn parse_ci_run(runs: &Value, jobs: Option<&Value>) -> Option<crate::ci_result::CiRun> {
    use crate::ci_result::{CiRun, STEPS, Step};
    let run = runs["workflow_runs"]
        .as_array()?
        .iter()
        .filter(|r| r["name"].as_str() == Some(CI_WORKFLOW))
        .max_by_key(|r| r["created_at"].as_str().unwrap_or("").to_string())?;
    let finished = run["status"].as_str() == Some("completed");
    let conclusion = run["conclusion"].as_str().unwrap_or("").to_string();
    let seconds = |key: &str| {
        run[key]
            .as_str()
            .and_then(|t| t.parse::<jiff::Timestamp>().ok())
    };
    let duration_s = match (seconds("created_at"), seconds("updated_at")) {
        (Some(start), Some(end)) => (end.as_second() - start.as_second()).max(0) as f64,
        _ => 0.0,
    };
    let step_conclusion = jobs
        .and_then(|j| j["jobs"].as_array())
        .into_iter()
        .flatten()
        .flat_map(|job| job["steps"].as_array().into_iter().flatten())
        .find(|s| {
            s["name"]
                .as_str()
                .is_some_and(|n| n.starts_with(CI_CHECK_STEP))
        })
        .and_then(|s| s["conclusion"].as_str());
    let steps = match step_conclusion {
        Some(c) => STEPS
            .iter()
            .map(|name| Step {
                name: name.to_string(),
                conclusion: c.to_string(),
                duration_s: 0.0,
            })
            .collect(),
        None => Vec::new(),
    };
    Some(CiRun {
        sha: run["head_sha"].as_str()?.to_string(),
        address: run["html_url"].as_str()?.to_string(),
        finished,
        conclusion,
        steps,
        duration_s,
    })
}

impl crate::ci_result::CiSource for GhClient<'_> {
    fn fetch(&self, sha: &str) -> Result<Option<crate::ci_result::CiRun>> {
        let runs = self.api(
            HostOperation::ReadChecks,
            "GET",
            &self.repo_path(&format!("actions/runs?head_sha={sha}&per_page=20")),
            &[],
        )?;
        let Some(first) = parse_ci_run(&runs, None) else {
            return Ok(None);
        };
        let id = runs["workflow_runs"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|r| r["name"].as_str() == Some(CI_WORKFLOW))
            .max_by_key(|r| r["created_at"].as_str().unwrap_or("").to_string())
            .and_then(|r| r["id"].as_u64());
        let (Some(id), true) = (id, first.finished) else {
            return Ok(Some(first));
        };
        let jobs = self.api(
            HostOperation::ReadChecks,
            "GET",
            &self.repo_path(&format!("actions/runs/{id}/jobs")),
            &[],
        )?;
        Ok(parse_ci_run(&runs, Some(&jobs)))
    }
}

/// Reads the check-runs reply and the combined-status reply for one commit. A failure anywhere is red;
/// anything unfinished, or no report at all, is pending; green needs at least one report and no other state.
pub fn classify_checks(runs: &Value, status: &Value) -> Checks {
    let runs = runs["check_runs"].as_array().cloned().unwrap_or_default();
    let statuses = status["statuses"].as_array().cloned().unwrap_or_default();
    let run_failed = |r: &Value| {
        matches!(
            r["conclusion"].as_str(),
            Some(
                "failure"
                    | "cancelled"
                    | "timed_out"
                    | "action_required"
                    | "startup_failure"
                    | "stale"
            )
        )
    };
    let status_state = status["state"].as_str();
    if runs.iter().any(run_failed) || matches!(status_state, Some("failure" | "error")) {
        return Checks::Red;
    }
    let runs_done = runs
        .iter()
        .all(|r| r["status"].as_str() == Some("completed"));
    let statuses_done = statuses.is_empty() || status_state == Some("success");
    if (runs.is_empty() && statuses.is_empty()) || !runs_done || !statuses_done {
        return Checks::Pending;
    }
    Checks::Green
}

/// Splits a remote URL into its host and `owner/name`. Only github.com is supported; the URL is not echoed
/// in the error because it may carry a credential.
pub fn parse_remote(url: &str) -> Result<(String, String)> {
    let url = url.trim().trim_end_matches('/');
    let url = url.strip_suffix(".git").unwrap_or(url);
    let Some(at) = url.find(HOST) else {
        bail!("only {HOST} remotes are supported for pull requests");
    };
    let rest = url[at + HOST.len()..].trim_start_matches([':', '/']);
    match rest.split_once('/') {
        Some((owner, name)) if !owner.is_empty() && !name.is_empty() && !name.contains('/') => {
            Ok((HOST.to_string(), format!("{owner}/{name}")))
        }
        _ => bail!("the remote does not name an owner and a repository on {HOST}"),
    }
}

/// Parses the host's reply to "which pull requests contain this commit" into the one that merged `commit`: a
/// pull request with `merged_at` set whose `merge_commit_sha` is `commit`. None when there is none; several
/// is an error, because choosing one would be a guess.
pub fn parse_merged_pr(reply: &Value, commit: &str) -> Result<Option<MergedPr>> {
    let mut found: Vec<MergedPr> = reply
        .as_array()
        .into_iter()
        .flatten()
        .filter(|pr| !pr["merged_at"].is_null() && pr["merge_commit_sha"].as_str() == Some(commit))
        .filter_map(|pr| {
            Some(MergedPr {
                number: pr["number"].as_u64()?,
                head_sha: pr["head"]["sha"].as_str()?.to_string(),
            })
        })
        .collect();
    match found.len() {
        0 => Ok(None),
        1 => Ok(found.pop()),
        n => bail!("the host names {n} merged pull requests for commit {commit}; it is ambiguous"),
    }
}

impl CommitHost for GhClient<'_> {
    fn merged_pr(&self, commit: &str) -> Result<Option<MergedPr>> {
        let reply = self.api(
            HostOperation::ReadPr,
            "GET",
            &self.repo_path(&format!("commits/{commit}/pulls")),
            &[],
        )?;
        parse_merged_pr(&reply, commit)
    }
}
