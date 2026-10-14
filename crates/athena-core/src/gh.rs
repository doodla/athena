//! `athena land --pr`: push the job's branch, open a pull request, wait for green checks, and, only when the
//! opt-in setting `[landing] merge_via_rest` is on, ask the host to merge (`docs/design/native-pr-landing.md`).
//!
//! The host client runs the `gh` program and only its `api` subcommand, with REST paths such as
//! `repos/{owner}/{repo}/pulls`. It uses no other `gh` subcommand and no query-language endpoint. The program
//! sits behind [`Runner`], and the host behind [`PrHost`], so the logic is tested without a network.
//!
//! With the setting off (the default) the command stops at an open pull request and reports whether its
//! checks are green. A later run of the same command looks the pull request up by its stored number and, if
//! it was merged, runs the merged-content check of [`crate::pr`]: the job is landed on a matching
//! fingerprint, and `merged-unverified` otherwise. The merged commit is never found by reading trailers.

use crate::pr::{
    Ancestry, Content, GitContent, Host, LinkState, PrLink, PushRequest, ReplaceRequest,
    check_push, check_replace, load_link, observe_merge, store_link,
};
use crate::state::StateDir;
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_data::records::{HostError, HostOperation, MessageClass, Verdict, WorkPhase};
use athena_data::{JobId, RecordId};
use athena_git::{CommitText, Git};
use serde::Serialize;
use serde_json::Value;
use std::cell::RefCell;
use std::process::{Command, Stdio};
use std::time::Duration;

/// The only host this slice supports.
const HOST: &str = "github.com";

/// Runs the host client with the given arguments and returns its standard output.
pub trait Runner {
    fn gh(&self, args: &[String]) -> Result<String>;
}

/// [`Runner`] over the installed `gh` program. It never reads or stores a credential: `gh` holds it.
#[derive(Debug, Default)]
pub struct GhCli;

impl Runner for GhCli {
    fn gh(&self, args: &[String]) -> Result<String> {
        let out = Command::new("gh")
            .args(args)
            .stdin(Stdio::null())
            .output()
            .context("running gh; is the GitHub client installed?")?;
        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            return Err(GhFailure {
                call: args.get(3).cloned().unwrap_or_else(|| "api".to_string()),
                status_code: parse_status(&stderr),
                text: stderr.trim().to_string(),
            }
            .into());
        }
        String::from_utf8(out.stdout).context("gh printed non-text output")
    }
}

/// A call that `gh` ran and the host refused. It is the error a [`Runner`] returns for that case, so the client
/// can read the status without matching text again. `text` is for the person reading the error; it is never
/// stored in a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GhFailure {
    /// What was called (the REST path).
    pub call: String,
    /// The HTTP status, when `gh` reported one.
    pub status_code: Option<u16>,
    /// The client's own words, as printed.
    pub text: String,
}

impl std::fmt::Display for GhFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "gh {} failed: {}", self.call, self.text)
    }
}

impl std::error::Error for GhFailure {}

/// The HTTP status in the text `gh` prints for a refused call, which has the form `... (HTTP 405)`. The first
/// `HTTP ` followed by three digits that make a status from 100 to 599 counts; no such text gives `None`.
pub fn parse_status(stderr: &str) -> Option<u16> {
    stderr.match_indices("HTTP ").find_map(|(at, marker)| {
        let digits = &stderr[at + marker.len()..];
        let code = digits.get(..3)?;
        if !code.bytes().all(|b| b.is_ascii_digit())
            || digits[3..].starts_with(|c: char| c.is_ascii_digit())
        {
            return None;
        }
        code.parse().ok().filter(|n| (100..=599).contains(n))
    })
}

/// Whether a status is a failure worth asking again: a server that did not answer well, not one that refused
/// the request.
pub fn is_transient(status: Option<u16>) -> bool {
    matches!(status, Some(500 | 502 | 503 | 504))
}

impl GhFailure {
    /// Why this failure is worth asking again, or `None` when it is not. An HTTP status decides on its own
    /// (5xx of [`is_transient`] yes; 4xx and auth errors no). Without a status, `gh`'s text is read for a timeout
    /// or a connection reset; "not logged in" and the like never match.
    pub fn transient_reason(&self) -> Option<String> {
        if let Some(status) = self.status_code {
            return is_transient(Some(status)).then(|| format!("HTTP {status}"));
        }
        let text = self.text.to_ascii_lowercase();
        if text.contains("timeout") || text.contains("timed out") {
            Some("timeout".to_string())
        } else if text.contains("connection reset") || text.contains("econnreset") {
            Some("connection reset".to_string())
        } else {
            None
        }
    }
}

/// How a call is repeated after a transient failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Retry {
    /// How many times the call is made in all, at least once.
    pub attempts: u32,
    /// The wait before the second call; the wait before the third is twice that, and so on. Jitter is added
    /// on top (see [`Retry::wait`]).
    pub delay: Duration,
}

impl Retry {
    /// The wait after the failed attempt number `attempt` (counting from 1): `delay` doubled for each earlier
    /// failure, plus up to half of that again. `entropy` picks the jitter (any number; it is taken modulo 1000),
    /// so a test can fix it.
    pub fn wait(&self, attempt: u32, entropy: u32) -> Duration {
        let base = self.delay * 2u32.saturating_pow(attempt.saturating_sub(1));
        base + base * (entropy % 1000) / 2000
    }
}

/// A jitter source without a new dependency: the sub-second part of the clock.
fn clock_entropy() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos())
}

impl Retry {
    /// Three calls in all, with a short wait between them.
    pub const STANDARD: Retry = Retry {
        attempts: 3,
        delay: Duration::from_secs(2),
    };
}

/// The refusals of one landing run, kept in memory until the caller commits them as records. The client writes
/// one for every refused call, retried or not.
#[derive(Debug)]
pub struct HostErrorLog {
    job: JobId,
    phase: WorkPhase,
    entries: RefCell<Vec<HostError>>,
}

impl HostErrorLog {
    pub fn new(job: JobId, phase: WorkPhase) -> Self {
        HostErrorLog {
            job,
            phase,
            entries: RefCell::new(Vec::new()),
        }
    }

    /// Notes one refused call. The client does this for every refusal; it is public so a test of the landing
    /// trigger can stand in for the client.
    pub fn record(&self, operation: HostOperation, status_code: Option<u16>) {
        let mut entries = self.entries.borrow_mut();
        let now = crate::now();
        let mut id = RecordId::generate(now);
        while entries.iter().any(|e| e.id == id) {
            id = RecordId::generate(now);
        }
        entries.push(HostError {
            id,
            at: now,
            status_code,
            operation,
            job: self.job.clone(),
            phase: self.phase,
            message_class: MessageClass::of_status(status_code),
            derived: false,
            source_intervention: None,
        });
    }

    /// The refusals so far, oldest first.
    pub fn entries(&self) -> Vec<HostError> {
        self.entries.borrow().clone()
    }
}

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
    fn merge(&self, number: u64, head: &str, text: &CommitText) -> Result<()>;
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

/// The remote git repository the branch is pushed to.
pub trait Remote {
    /// The commit the remote branch holds, or `None` when it does not exist.
    fn remote_head(&self, branch: &str) -> Result<Option<String>>;
    /// A regular push of `commit` to `branch`. There is no force option.
    fn push(&self, branch: &str, commit: &str) -> Result<()>;
    /// Replaces `branch` with `commit` in one atomic push that succeeds only if the remote branch still holds
    /// `expected`. If the remote moved since `expected` was read, the push fails and the remote is unchanged.
    fn replace(&self, branch: &str, commit: &str, expected: &str) -> Result<()>;
}

/// [`Remote`] over a named git remote.
#[derive(Debug)]
pub struct GitRemote<'a> {
    pub git: &'a Git,
    pub name: &'a str,
}

impl Remote for GitRemote<'_> {
    fn remote_head(&self, branch: &str) -> Result<Option<String>> {
        let listing = self.git.run([
            "ls-remote",
            "--heads",
            self.name,
            &format!("refs/heads/{branch}"),
        ])?;
        Ok(listing
            .lines()
            .next()
            .and_then(|l| l.split_whitespace().next())
            .map(str::to_string))
    }

    fn push(&self, branch: &str, commit: &str) -> Result<()> {
        self.git
            .run(["push", self.name, &format!("{commit}:refs/heads/{branch}")])?;
        Ok(())
    }

    fn replace(&self, branch: &str, commit: &str, expected: &str) -> Result<()> {
        Ok(self
            .git
            .push_if_remote_at(self.name, branch, commit, expected)?)
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
    fn api(
        &self,
        operation: HostOperation,
        method: &str,
        path: &str,
        fields: &[(&str, &str)],
    ) -> Result<Value> {
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
        if text.trim().is_empty() {
            return Ok(Value::Null);
        }
        serde_json::from_str(&text).with_context(|| format!("parsing the reply to {path}"))
    }

    fn repo_path(&self, rest: &str) -> String {
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

    fn merge(&self, number: u64, head: &str, text: &CommitText) -> Result<()> {
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
        Ok(())
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

/// The branch Athena pushes for a job: the only one the push guard allows.
pub fn work_branch(job: &JobId) -> String {
    format!("athena/{}", job.as_str())
}

/// The commit pushed as the work branch: the job's locally landed commit. It is found from the job's landing
/// record: the newest commit reachable from `head` that carries the job's `Athena-Job` trailer and the
/// verified content's `Athena-Verified-Content` trailer. It holds the verified change, the trailers and the
/// records. A job that is not landed locally has no such commit, and that is an error: nothing is pushed.
pub fn landed_commit(git: &Git, head: &str, job: &JobId, fingerprint: &str) -> Result<String> {
    let candidates = git.log_grep(head, &format!("^Athena-Verified-Content: {fingerprint}$"))?;
    for commit in candidates {
        let trailers = git.trailers(&git.message(&commit)?)?;
        let for_job = trailers
            .iter()
            .any(|(k, v)| k == "Athena-Job" && v == job.as_str());
        let verified = trailers
            .iter()
            .any(|(k, v)| k == "Athena-Verified-Content" && v == fingerprint);
        if for_job && verified {
            return Ok(commit);
        }
    }
    bail!(
        "job {job} is not landed on this branch, so there is no landed commit to push; run `athena go {job}` first"
    )
}

/// How long to wait for the checks.
pub struct Poll<'a> {
    /// How many times to ask, at least once.
    pub attempts: u32,
    pub delay: Duration,
    pub sleep: &'a dyn Fn(Duration),
}

impl std::fmt::Debug for Poll<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Poll")
            .field("attempts", &self.attempts)
            .field("delay", &self.delay)
            .finish_non_exhaustive()
    }
}

/// Everything `land_pr` needs for one job.
pub struct LandPr<'a> {
    pub state: &'a StateDir,
    pub job: &'a JobId,
    /// The branch the caller asks to push; the guard allows only [`work_branch`].
    pub branch: &'a str,
    /// The commit to push as the pull request's head: the job's locally landed commit (see [`landed_commit`]).
    pub commit: &'a str,
    pub verified_fingerprint: &'a str,
    /// The pull request's text. Its base is the host's default branch, read when the request is opened.
    pub pr: PrDraft<'a>,
    /// The message a squash merge carries: the job's title, and its trailers in the last paragraph.
    pub squash: &'a CommitText,
    pub host_name: &'a str,
    pub repository: &'a str,
    /// The `[landing] merge_via_rest` setting.
    pub merge_via_rest: bool,
    pub poll: Poll<'a>,
    pub ancestry: &'a dyn Ancestry,
    pub remote: &'a dyn Remote,
    pub host: &'a dyn PrHost,
    pub content: &'a dyn Content,
}

impl std::fmt::Debug for LandPr<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LandPr")
            .field("job", &self.job)
            .field("branch", &self.branch)
            .field("repository", &self.repository)
            .field("merge_via_rest", &self.merge_via_rest)
            .finish_non_exhaustive()
    }
}

/// What `land_pr` found or did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrReport {
    pub link: PrLink,
    /// The checks, when the pull request was still open and they were read.
    pub checks: Option<Checks>,
    /// Whether this run asked the host to merge.
    pub merge_requested: bool,
}

/// Runs the flow. With no stored link: guard and push the branch, open the pull request, store the link.
/// Then, while the pull request is open: look for a merge, else read the checks, and merge over REST only if
/// the setting is on and the checks are green.
pub fn land_pr(job: &LandPr) -> Result<PrReport> {
    let mut link = match load_link(job.state, job.job)? {
        Some(link) => link,
        None => publish(job)?,
    };
    let mut checks = None;
    let mut merge_requested = false;
    observe_merge(job.host, job.content, &mut link, job.verified_fingerprint)?;
    if link.state == LinkState::Open && link.head != job.commit {
        replace_head(job, &mut link)?;
    }
    if link.state == LinkState::Open {
        let seen = wait_for_checks(job, &link.head)?;
        checks = Some(seen);
        // The merge gate: the setting is on and every check is green. Nothing else reaches the host's
        // merge endpoint.
        if job.merge_via_rest && seen == Checks::Green {
            job.host.merge(link.pr_number, &link.head, job.squash)?;
            merge_requested = true;
            observe_merge(job.host, job.content, &mut link, job.verified_fingerprint)?;
        }
    }
    store_link(job.state, job.job, &link)?;
    Ok(PrReport {
        link,
        checks,
        merge_requested,
    })
}

/// For a job whose landed commit is not on the current branch: when the job's pull request link already
/// exists, asks the host whether it merged and runs the merged-content check. Returns the report when the
/// link is no longer open (landed, or merged but not verified), and stores it. Returns `None` when there is
/// no link or the pull request is still open, so the caller keeps its "not landed" error. Nothing is pushed.
pub fn confirm_existing_merge(
    state: &StateDir,
    job: &JobId,
    host: &dyn Host,
    content: &dyn Content,
    verified_fingerprint: &str,
) -> Result<Option<PrReport>> {
    let Some(mut link) = load_link(state, job)? else {
        return Ok(None);
    };
    observe_merge(host, content, &mut link, verified_fingerprint)?;
    if link.state == LinkState::Open {
        return Ok(None);
    }
    store_link(state, job, &link)?;
    Ok(Some(PrReport {
        link,
        checks: None,
        merge_requested: false,
    }))
}

/// An open pull request whose stored head is not the commit to push: moves the job's own branch to that
/// commit with one atomic push that is conditional on the remote head just observed, keeps the pull request,
/// and updates the stored head. A remote head that is missing, unknown locally, or whose ancestry check fails
/// is an error and nothing is pushed.
fn replace_head(job: &LandPr, link: &mut PrLink) -> Result<()> {
    let allowed = work_branch(job.job);
    let remote_head = job.remote.remote_head(job.branch)?;
    check_replace(
        job.ancestry,
        &ReplaceRequest {
            allowed_branch: &allowed,
            branch: job.branch,
            local_head: job.commit,
            remote_head: remote_head.as_deref(),
        },
    )?;
    let expected = remote_head.expect("check_replace accepts only an observed remote head");
    if expected != job.commit {
        job.remote.replace(job.branch, job.commit, &expected)?;
    }
    link.head = job.commit.to_string();
    store_link(job.state, job.job, link)
}

fn publish(job: &LandPr) -> Result<PrLink> {
    let allowed = work_branch(job.job);
    let commit = job.commit.to_string();
    let remote_head = job.remote.remote_head(job.branch)?;
    check_push(
        job.ancestry,
        &PushRequest {
            allowed_branch: &allowed,
            branch: job.branch,
            local_head: &commit,
            remote_head: remote_head.as_deref(),
        },
    )?;
    let base = job.host.default_branch()?;
    job.remote.push(job.branch, &commit)?;
    let pr_number = job.host.open_pr(&NewPr {
        title: job.pr.title,
        body: job.pr.body,
        head: job.pr.head,
        base: &base,
    })?;
    let link = PrLink {
        pr_number,
        host: job.host_name.to_string(),
        repository: job.repository.to_string(),
        head: commit,
        job: job.job.as_str().to_string(),
        state: LinkState::Open,
        merged_commit: None,
    };
    store_link(job.state, job.job, &link)?;
    Ok(link)
}

fn wait_for_checks(job: &LandPr, head: &str) -> Result<Checks> {
    let attempts = job.poll.attempts.max(1);
    let mut seen = Checks::Pending;
    for n in 0..attempts {
        seen = job.host.checks(head)?;
        if seen != Checks::Pending {
            break;
        }
        if n + 1 < attempts {
            (job.poll.sleep)(job.poll.delay);
        }
    }
    Ok(seen)
}

impl Athena {
    /// `athena land --pr <job>`: takes a verified job to an open pull request, or, when one exists, looks at
    /// it again (see the module text).
    pub fn land_pr(
        &self,
        job_ref: &str,
        runner: &dyn Runner,
        poll: Poll<'_>,
        ui: &dyn Ui,
    ) -> Result<PrReport> {
        self.refuse_if_stopped()?;
        let head = self.head()?;
        let records = crate::store::Records::load(&self.git, &head)?;
        let id = records.resolve(job_ref)?;
        let Some(_lock) = self.state.try_lock_job(&id)? else {
            bail!("job {id} is busy in another athena process");
        };
        let job = records.jobs.get(id.as_str()).expect("resolved");
        let verification = job
            .verification
            .as_ref()
            .with_context(|| format!("job {id} has no verification; it cannot be landed"))?;
        if verification.final_verdict != Verdict::Pass {
            bail!("job {id} did not pass verification; it cannot be landed");
        }
        let candidate = job
            .candidate
            .as_ref()
            .with_context(|| format!("job {id} has no candidate record"))?;
        let remote_name = "origin";
        let url = self.git.run(["remote", "get-url", remote_name])?;
        let (host_name, repository) = parse_remote(&url)?;
        let config = athena_gate::config_at(&self.git, Some(&head))?;
        let branch = work_branch(&id);
        let title = job.title();
        let body = format!(
            "{}\n\nVerified job {}.",
            verification.summary.as_deref().unwrap_or(&title),
            id.as_str()
        );
        let content = GitContent {
            git: &self.git,
            rules_commit: &candidate.base_commit,
            remote: remote_name,
        };
        let log = HostErrorLog::new(id.clone(), WorkPhase::Landing);
        let client = GhClient {
            log: Some(&log),
            ui: Some(ui),
            ..GhClient::new(runner, repository.clone())
        };
        let remote = GitRemote {
            git: &self.git,
            name: remote_name,
        };
        let landed = landed_commit(&self.git, &head, &id, &verification.verified_fingerprint);
        let landed_id = landed.as_ref().ok().cloned();
        let attempt = || -> Result<PrReport> {
            match landed {
                Ok(landed) => land_pr(&LandPr {
                    state: &self.state,
                    job: &id,
                    branch: &branch,
                    commit: &landed,
                    verified_fingerprint: &verification.verified_fingerprint,
                    pr: PrDraft {
                        title: &title,
                        body: &body,
                        head: &branch,
                    },
                    squash: &self.git.squash_text(&landed)?,
                    host_name: &host_name,
                    repository: &repository,
                    merge_via_rest: config.landing.merge_via_rest,
                    poll,
                    ancestry: &self.git,
                    remote: &remote,
                    host: &client,
                    content: &content,
                }),
                // The landed commit is not on this branch. If the job's pull request already exists, the
                // host may have merged it, and that confirms the landing without the local commit.
                Err(not_landed) => match confirm_existing_merge(
                    &self.state,
                    &id,
                    &client,
                    &content,
                    &verification.verified_fingerprint,
                )? {
                    Some(report) => Ok(report),
                    None => Err(not_landed),
                },
            }
        };
        let outcome = attempt();
        // The watcher runs at landing, after a host error or not: it records every refused call of this run
        // and puts a decision under review when a signal fires. It runs before the outcome is returned, so a
        // refusal that failed the landing is seen too. A failure of the watcher is shown, not allowed to hide
        // the landing's own result.
        if let Err(problem) = self.watch_landing(&log, ui) {
            ui.line(&format!(
                "warning: could not record host errors or watch the decisions: {problem:#}"
            ));
        }
        let report = outcome?;
        ui.line(&format!(
            "pull request #{} on {}/{}: {:?}",
            report.link.pr_number, report.link.host, report.link.repository, report.link.state
        ));
        // A merge the host made without the branch's commits (a squash or a rebase) leaves the local branch
        // beside main; bring it onto the merged commit so the next pull request opens clean.
        if report.link.state == LinkState::Landed
            && let (Some(landed), Some(merged)) = (&landed_id, &report.link.merged_commit)
        {
            self.resync_onto_merge(landed, merged, ui)?;
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeMap;

    /// The repository reply of a host that allows every merge method.
    const ALL_METHODS: &str = r#"{"default_branch": "trunk", "allow_merge_commit": true,
        "allow_squash_merge": true, "allow_rebase_merge": true}"#;
    /// The repository reply of a host that allows only squash merges.
    const SQUASH_ONLY: &str = r#"{"default_branch": "trunk", "allow_merge_commit": false,
        "allow_squash_merge": true, "allow_rebase_merge": false}"#;

    struct FakeRunner {
        calls: RefCell<Vec<Vec<String>>>,
        /// What `GET repos/example/project` answers.
        repo: &'static str,
    }
    impl FakeRunner {
        fn new(repo: &'static str) -> Self {
            FakeRunner {
                calls: RefCell::new(vec![]),
                repo,
            }
        }
    }
    fn text() -> CommitText {
        CommitText {
            title: "Change a".into(),
            body: "Body.\n\nAthena-Job: bootstrap\n".into(),
        }
    }
    impl Runner for FakeRunner {
        fn gh(&self, args: &[String]) -> Result<String> {
            self.calls.borrow_mut().push(args.to_vec());
            let path = args.iter().find(|a| a.starts_with("repos/")).unwrap();
            Ok(if path == "repos/example/project" {
                self.repo.to_string()
            } else if path.ends_with("/pulls") {
                r#"{"number": 7}"#.to_string()
            } else if path.contains("/pulls/") && !path.ends_with("/merge") {
                r#"{"merged": true, "merge_commit_sha": "m1"}"#.to_string()
            } else if path.contains("check-runs") {
                r#"{"check_runs": [{"status": "completed", "conclusion": "success"}]}"#.to_string()
            } else if path.ends_with("/status") {
                r#"{"state": "success", "statuses": []}"#.to_string()
            } else {
                "{}".to_string()
            })
        }
    }

    #[test]
    fn the_client_runs_only_gh_api_with_rest_paths() {
        let runner = FakeRunner::new(ALL_METHODS);
        let client = GhClient::new(&runner, "example/project".into());
        let pr = NewPr {
            title: "t",
            body: "b",
            head: "athena/j",
            base: "main",
        };
        assert_eq!(client.open_pr(&pr).unwrap(), 7);
        assert_eq!(client.checks("abc").unwrap(), Checks::Green);
        client.merge(7, "abc", &text()).unwrap();
        let link = PrLink {
            pr_number: 7,
            host: HOST.into(),
            repository: "example/project".into(),
            head: "abc".into(),
            job: "j".into(),
            state: LinkState::Open,
            merged_commit: None,
        };
        assert_eq!(client.merged_commit(&link).unwrap(), Some("m1".into()));

        let calls = runner.calls.borrow();
        // open, checks (two reads), the repository's methods, the merge, the merged lookup.
        assert_eq!(calls.len(), 6);
        for args in calls.iter() {
            assert_eq!(args[0], "api", "{args:?}");
            assert_eq!(args[1], "--method");
            assert!(args[3].starts_with("repos/example/project"), "{args:?}");
            assert!(!args.iter().any(|a| a == "pr" || a == "--auto"), "{args:?}");
        }
        assert_eq!(calls[0][3], "repos/example/project/pulls");
        assert_eq!(calls[0][2], "POST");
        assert_eq!(calls[3][3], "repos/example/project");
        assert_eq!(calls[4][2], "PUT");
        assert_eq!(calls[4][3], "repos/example/project/pulls/7/merge");
        // A merge commit is allowed, so it is requested, with no message of ours.
        assert!(calls[4].contains(&"merge_method=merge".to_string()));
        assert!(!calls[4].iter().any(|a| a.starts_with("commit_")));
    }

    /// The arguments of the merge request a client makes against a repository reply.
    fn merge_request(repo: &'static str) -> Result<Vec<String>> {
        let runner = FakeRunner::new(repo);
        let client = GhClient::new(&runner, "example/project".into());
        client.merge(7, "abc", &text())?;
        let calls = runner.calls.borrow();
        Ok(calls.last().unwrap().clone())
    }

    #[test]
    fn with_only_squash_allowed_the_merge_asks_for_a_squash_carrying_the_jobs_trailers() {
        let args = merge_request(SQUASH_ONLY).unwrap();
        assert_eq!(args[2], "PUT");
        for wanted in [
            "sha=abc",
            "merge_method=squash",
            "commit_title=Change a",
            "commit_message=Body.\n\nAthena-Job: bootstrap\n",
        ] {
            assert!(args.contains(&wanted.to_string()), "{wanted:?} in {args:?}");
        }
        assert!(!args.contains(&"merge_method=merge".to_string()));
    }

    #[test]
    fn the_merge_method_is_chosen_from_what_the_host_allows() {
        let flags = |merge, squash, rebase| MergeMethods {
            merge,
            squash,
            rebase,
        };
        assert_eq!(
            flags(true, true, true).choose().unwrap(),
            MergeMethod::Merge
        );
        assert_eq!(
            flags(false, true, true).choose().unwrap(),
            MergeMethod::Squash
        );
        assert_eq!(
            flags(false, false, true).choose().unwrap(),
            MergeMethod::Rebase
        );
        assert_eq!(
            MergeMethods::from_repository(&json!({
                "allow_merge_commit": false, "allow_squash_merge": true, "allow_rebase_merge": false
            })),
            flags(false, true, false)
        );
        // A flag the host leaves out is not allowed.
        assert_eq!(
            MergeMethods::from_repository(&json!({})),
            flags(false, false, false)
        );
    }

    #[test]
    fn with_no_merge_method_allowed_the_merge_fails_clearly_and_sends_nothing() {
        let none = r#"{"default_branch": "trunk", "allow_merge_commit": false,
            "allow_squash_merge": false, "allow_rebase_merge": false}"#;
        let runner = FakeRunner::new(none);
        let client = GhClient::new(&runner, "example/project".into());
        let err = client.merge(7, "abc", &text()).unwrap_err().to_string();
        assert!(err.contains("allows no merge method"), "{err}");
        assert!(err.contains("allow_squash_merge false"), "{err}");
        // Only the read of the repository happened; no PUT.
        assert_eq!(runner.calls.borrow().len(), 1);
    }

    #[test]
    fn checks_are_classified_from_the_two_replies() {
        let done = |c: &str| json!({"check_runs": [{"status": "completed", "conclusion": c}]});
        let none = json!({"state": "pending", "statuses": []});
        assert_eq!(classify_checks(&done("success"), &none), Checks::Green);
        assert_eq!(classify_checks(&done("skipped"), &none), Checks::Green);
        assert_eq!(classify_checks(&done("failure"), &none), Checks::Red);
        let running = json!({"check_runs": [{"status": "in_progress", "conclusion": null}]});
        assert_eq!(classify_checks(&running, &none), Checks::Pending);
        // No report at all is not green.
        assert_eq!(
            classify_checks(&json!({"check_runs": []}), &none),
            Checks::Pending
        );
        let failed = json!({"state": "failure", "statuses": [{"state": "failure"}]});
        assert_eq!(classify_checks(&done("success"), &failed), Checks::Red);
        let waiting = json!({"state": "pending", "statuses": [{"state": "pending"}]});
        assert_eq!(classify_checks(&done("success"), &waiting), Checks::Pending);
    }

    #[test]
    fn remotes_are_split_into_host_and_repository() {
        let ok = ("github.com".to_string(), "example/project".to_string());
        for url in [
            "https://github.com/example/project.git\n",
            "https://github.com/example/project",
            "ssh://github.com/example/project.git",
            "github.com:example/project.git",
        ] {
            assert_eq!(parse_remote(url).unwrap(), ok, "{url}");
        }
        assert!(parse_remote("https://gitlab.example.com/example/project").is_err());
        assert!(parse_remote("https://github.com/example").is_err());
    }

    // ---- the flow, with a fake host ----

    struct Anc(bool);
    impl Ancestry for Anc {
        fn is_ancestor(&self, _: &str, _: &str) -> Result<bool> {
            Ok(self.0)
        }
    }

    #[derive(Default)]
    struct FakeRemote {
        head: Option<String>,
        pushes: RefCell<Vec<(String, String)>>,
        /// (branch, commit, expected) of each replacement.
        replaces: RefCell<Vec<(String, String, String)>>,
    }
    impl Remote for FakeRemote {
        fn remote_head(&self, _: &str) -> Result<Option<String>> {
            Ok(self.head.clone())
        }
        fn replace(&self, branch: &str, commit: &str, expected: &str) -> Result<()> {
            self.replaces.borrow_mut().push((
                branch.to_string(),
                commit.to_string(),
                expected.to_string(),
            ));
            Ok(())
        }
        fn push(&self, branch: &str, commit: &str) -> Result<()> {
            self.pushes
                .borrow_mut()
                .push((branch.to_string(), commit.to_string()));
            Ok(())
        }
    }

    struct FakeHost {
        checks: Checks,
        /// What the host reports as merged now.
        merged: RefCell<Option<String>>,
        /// The commit a merge request produces.
        merge_makes: &'static str,
        opened: Cell<u32>,
        merges: Cell<u32>,
        check_reads: Cell<u32>,
        asked: RefCell<Vec<u64>>,
        /// The bases pull requests were opened against.
        bases: RefCell<Vec<String>>,
        /// The commits whose checks were read.
        checked: RefCell<Vec<String>>,
    }
    impl FakeHost {
        fn new(checks: Checks) -> Self {
            FakeHost {
                checks,
                merged: RefCell::new(None),
                merge_makes: "merged1",
                opened: Cell::new(0),
                merges: Cell::new(0),
                check_reads: Cell::new(0),
                asked: RefCell::new(vec![]),
                bases: RefCell::new(vec![]),
                checked: RefCell::new(vec![]),
            }
        }
    }
    impl Host for FakeHost {
        fn merged_commit(&self, link: &PrLink) -> Result<Option<String>> {
            self.asked.borrow_mut().push(link.pr_number);
            Ok(self.merged.borrow().clone())
        }
    }
    impl PrHost for FakeHost {
        fn default_branch(&self) -> Result<String> {
            Ok("trunk".to_string())
        }
        fn open_pr(&self, pr: &NewPr) -> Result<u64> {
            self.opened.set(self.opened.get() + 1);
            self.bases.borrow_mut().push(pr.base.to_string());
            Ok(42)
        }
        fn checks(&self, commit: &str) -> Result<Checks> {
            self.check_reads.set(self.check_reads.get() + 1);
            self.checked.borrow_mut().push(commit.to_string());
            Ok(self.checks)
        }
        fn merge(&self, _: u64, _: &str, _: &CommitText) -> Result<()> {
            self.merges.set(self.merges.get() + 1);
            *self.merged.borrow_mut() = Some(self.merge_makes.to_string());
            Ok(())
        }
    }

    struct FakeContent(BTreeMap<&'static str, &'static str>);
    impl Content for FakeContent {
        fn fingerprint_of(&self, commit: &str) -> Result<String> {
            Ok(self.0[commit].to_string())
        }
    }

    struct Rig {
        dir: tempfile::TempDir,
        job: JobId,
        remote: FakeRemote,
        content: FakeContent,
    }

    impl Rig {
        fn new() -> Self {
            Rig {
                dir: tempfile::tempdir().unwrap(),
                job: JobId::parse("bootstrap").unwrap(),
                remote: FakeRemote::default(),
                content: FakeContent(BTreeMap::from([
                    ("merged1", "verified"),
                    ("other", "different"),
                ])),
            }
        }

        fn run(&self, host: &FakeHost, merge_via_rest: bool, branch: &str) -> Result<PrReport> {
            self.run_with(host, merge_via_rest, branch, "cand1", &Anc(true))
        }

        fn run_with(
            &self,
            host: &dyn PrHost,
            merge_via_rest: bool,
            branch: &str,
            commit: &str,
            ancestry: &dyn Ancestry,
        ) -> Result<PrReport> {
            let state = StateDir::new(self.dir.path().to_path_buf());
            let nap = |_: Duration| {};
            land_pr(&LandPr {
                state: &state,
                job: &self.job,
                branch,
                commit,
                verified_fingerprint: "verified",
                pr: PrDraft {
                    title: "t",
                    body: "b",
                    head: "athena/bootstrap",
                },
                squash: &text(),
                host_name: HOST,
                repository: "example/project",
                merge_via_rest,
                poll: Poll {
                    attempts: 3,
                    delay: Duration::from_secs(1),
                    sleep: &nap,
                },
                ancestry,
                remote: &self.remote,
                host,
                content: &self.content,
            })
        }

        fn stored(&self) -> PrLink {
            let state = StateDir::new(self.dir.path().to_path_buf());
            load_link(&state, &self.job).unwrap().unwrap()
        }
    }

    #[test]
    fn with_the_setting_off_no_merge_is_requested_and_the_pull_request_stays_open_and_green() {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        let report = rig.run(&host, false, "athena/bootstrap").unwrap();
        assert_eq!(host.merges.get(), 0);
        assert!(!report.merge_requested);
        assert_eq!(report.checks, Some(Checks::Green));
        assert_eq!(report.link.state, LinkState::Open);
        assert_eq!(report.link.pr_number, 42);
        assert_eq!(report.link.head, "cand1");
        assert_eq!(rig.stored(), report.link);
        // Only the job's own branch was pushed, once.
        assert_eq!(
            *rig.remote.pushes.borrow(),
            vec![("athena/bootstrap".to_string(), "cand1".to_string())]
        );
        assert_eq!(host.opened.get(), 1);
    }

    #[test]
    fn with_the_setting_on_and_green_checks_the_rest_merge_is_requested_and_the_job_lands() {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        let report = rig.run(&host, true, "athena/bootstrap").unwrap();
        assert_eq!(host.merges.get(), 1);
        assert!(report.merge_requested);
        assert_eq!(report.link.state, LinkState::Landed);
        assert_eq!(report.link.merged_commit.as_deref(), Some("merged1"));
        assert_eq!(rig.stored().state, LinkState::Landed);
    }

    #[test]
    fn red_or_pending_checks_never_trigger_a_merge_even_with_the_setting_on() {
        for checks in [Checks::Red, Checks::Pending] {
            let rig = Rig::new();
            let host = FakeHost::new(checks);
            let report = rig.run(&host, true, "athena/bootstrap").unwrap();
            assert_eq!(host.merges.get(), 0, "{checks:?}");
            assert!(!report.merge_requested);
            assert_eq!(report.checks, Some(checks));
            assert_eq!(report.link.state, LinkState::Open);
        }
        // Pending checks are asked about up to the poll limit, no more.
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Pending);
        rig.run(&host, true, "athena/bootstrap").unwrap();
        assert_eq!(host.check_reads.get(), 3);
    }

    #[test]
    fn a_merge_seen_later_is_found_by_number_and_checked_by_content() {
        // Matching fingerprint: landed.
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        rig.run(&host, false, "athena/bootstrap").unwrap();
        *host.merged.borrow_mut() = Some("merged1".into());
        let report = rig.run(&host, false, "athena/bootstrap").unwrap();
        assert_eq!(report.link.state, LinkState::Landed);
        assert_eq!(report.checks, None);
        assert_eq!(host.opened.get(), 1, "no second pull request");
        assert_eq!(rig.remote.pushes.borrow().len(), 1, "no second push");
        assert_eq!(host.asked.borrow().last(), Some(&42));
        assert_eq!(rig.stored().state, LinkState::Landed);
        assert_eq!(report.link.merged_commit.as_deref(), Some("merged1"));

        // Different content: merged-unverified, never landed.
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        rig.run(&host, false, "athena/bootstrap").unwrap();
        *host.merged.borrow_mut() = Some("other".into());
        let report = rig.run(&host, false, "athena/bootstrap").unwrap();
        assert_eq!(report.link.state, LinkState::MergedUnverified);
        assert_eq!(report.link.merged_commit.as_deref(), Some("other"));
        assert_eq!(host.merges.get(), 0);
    }

    #[test]
    fn the_client_reads_the_default_branch_from_the_repository() {
        let runner = FakeRunner::new(ALL_METHODS);
        let client = GhClient::new(&runner, "example/project".into());
        assert_eq!(client.default_branch().unwrap(), "trunk");
        let calls = runner.calls.borrow();
        assert_eq!(calls.len(), 1);
        assert_eq!(
            calls[0][..4],
            ["api", "--method", "GET", "repos/example/project"]
        );
    }

    #[test]
    fn the_pull_request_targets_the_host_default_branch_and_the_work_commit() {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        let state = StateDir::new(rig.dir.path().to_path_buf());
        let nap = |_: Duration| {};
        let report = land_pr(&LandPr {
            state: &state,
            job: &rig.job,
            branch: "athena/bootstrap",
            commit: "work1",
            verified_fingerprint: "verified",
            pr: PrDraft {
                title: "t",
                body: "b",
                head: "athena/bootstrap",
            },
            squash: &text(),
            host_name: HOST,
            repository: "example/project",
            merge_via_rest: false,
            poll: Poll {
                attempts: 1,
                delay: Duration::ZERO,
                sleep: &nap,
            },
            ancestry: &Anc(true),
            remote: &rig.remote,
            host: &host,
            content: &rig.content,
        })
        .unwrap();
        // The base is what the host reports, whatever the local branch is called.
        assert_eq!(*host.bases.borrow(), vec!["trunk".to_string()]);
        // The pushed commit, the stored head and the checked commit are all the work commit.
        assert_eq!(
            *rig.remote.pushes.borrow(),
            vec![("athena/bootstrap".to_string(), "work1".to_string())]
        );
        assert_eq!(report.link.head, "work1");
        assert_eq!(*host.checked.borrow(), vec!["work1".to_string()]);
    }

    #[test]
    fn the_landed_commit_is_found_by_the_job_and_verified_content_trailers() {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        let write = |text: &str, message: &str| {
            std::fs::write(dir.path().join("a.txt"), text).unwrap();
            git.run(["add", "-A"]).unwrap();
            git.run(["commit", "-q", "-m", message]).unwrap();
            git.head().unwrap().unwrap()
        };
        git.run(["init", "-q", "-b", "main"]).unwrap();
        write("a", "base");
        let job = JobId::parse("bootstrap").unwrap();
        // Not landed yet: an error, not a fallback.
        assert!(landed_commit(&git, "HEAD", &job, "fp1").is_err());
        write("b", "Read\n\nAthena-Job: bootstrap\n");
        write(
            "c",
            "Other job\n\nAthena-Job: other\nAthena-Verified-Content: fp1\n",
        );
        assert!(landed_commit(&git, "HEAD", &job, "fp1").is_err());
        let landed = write(
            "d",
            "Change\n\nAthena-Job: bootstrap\nAthena-Verified-Content: fp1\n",
        );
        write("e", "records\n\nAthena-Job: bootstrap\n");
        assert_eq!(landed_commit(&git, "HEAD", &job, "fp1").unwrap(), landed);
        assert!(landed_commit(&git, "HEAD", &job, "fp2").is_err());
    }

    /// A repository whose branch has no landed commit for the job.
    fn unlanded_repo() -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        std::fs::write(dir.path().join("a.txt"), "a").unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "base"]).unwrap();
        (dir, git)
    }

    #[test]
    fn a_merged_pull_request_confirms_the_landing_though_the_landed_commit_is_off_this_branch() {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        // The first run opens the pull request and stores an open link.
        rig.run(&host, false, "athena/bootstrap").unwrap();
        assert_eq!(rig.stored().state, LinkState::Open);
        // The landed commit is not reachable from this branch.
        let (_dir, git) = unlanded_repo();
        assert!(
            landed_commit(&git, "HEAD", &rig.job, "verified")
                .unwrap_err()
                .to_string()
                .contains("not landed on this branch")
        );
        // The host reports the merge.
        *host.merged.borrow_mut() = Some("merged1".into());
        let state = StateDir::new(rig.dir.path().to_path_buf());
        let report =
            confirm_existing_merge(&state, &rig.job, &host, &rig.content, "verified").unwrap();
        let report = report.expect("the merge confirms the landing");
        assert_eq!(report.link.state, LinkState::Landed);
        assert_eq!(report.link.merged_commit.as_deref(), Some("merged1"));
        assert_eq!(rig.stored().state, LinkState::Landed);
        assert_eq!(host.asked.borrow().last(), Some(&42));
        // Nothing was pushed or opened, and no merge was requested.
        assert_eq!(rig.remote.pushes.borrow().len(), 1);
        assert!(rig.remote.replaces.borrow().is_empty());
        assert_eq!(host.opened.get(), 1);
        assert_eq!(host.merges.get(), 0);
    }

    #[test]
    fn a_merge_of_other_content_is_reported_as_merged_unverified_without_the_landed_commit() {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        rig.run(&host, false, "athena/bootstrap").unwrap();
        *host.merged.borrow_mut() = Some("other".into());
        let state = StateDir::new(rig.dir.path().to_path_buf());
        let report = confirm_existing_merge(&state, &rig.job, &host, &rig.content, "verified")
            .unwrap()
            .unwrap();
        assert_eq!(report.link.state, LinkState::MergedUnverified);
        assert_eq!(rig.stored().state, LinkState::MergedUnverified);
    }

    #[test]
    fn an_open_pull_request_or_no_link_keeps_the_not_landed_error() {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        let state = StateDir::new(rig.dir.path().to_path_buf());
        // No stored link: the host is not asked, and the caller's error stands.
        assert!(
            confirm_existing_merge(&state, &rig.job, &host, &rig.content, "verified")
                .unwrap()
                .is_none()
        );
        assert!(host.asked.borrow().is_empty());
        let (_dir, git) = unlanded_repo();
        let err = landed_commit(&git, "HEAD", &rig.job, "verified").unwrap_err();
        assert!(err.to_string().contains("not landed on this branch"));

        // A stored link whose pull request is not merged: still no landing.
        rig.run(&host, false, "athena/bootstrap").unwrap();
        assert!(
            confirm_existing_merge(&state, &rig.job, &host, &rig.content, "verified")
                .unwrap()
                .is_none()
        );
        assert_eq!(rig.stored().state, LinkState::Open);
    }

    #[test]
    fn a_new_branch_is_published_at_the_landed_commit() {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        let report = rig
            .run_with(&host, false, "athena/bootstrap", "landed1", &Anc(true))
            .unwrap();
        assert_eq!(
            *rig.remote.pushes.borrow(),
            vec![("athena/bootstrap".to_string(), "landed1".to_string())]
        );
        assert!(rig.remote.replaces.borrow().is_empty());
        assert_eq!(report.link.head, "landed1");
    }

    #[test]
    fn a_rerun_replaces_a_bare_candidate_head_with_the_landed_commit_and_reuses_the_pull_request() {
        let mut rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        rig.run(&host, false, "athena/bootstrap").unwrap();
        assert_eq!(rig.stored().head, "cand1");
        // The remote still holds the bare candidate pushed earlier.
        rig.remote.head = Some("cand1".into());
        let report = rig
            .run_with(&host, false, "athena/bootstrap", "landed1", &Anc(false))
            .unwrap();
        assert_eq!(
            *rig.remote.replaces.borrow(),
            vec![(
                "athena/bootstrap".to_string(),
                "landed1".to_string(),
                "cand1".to_string()
            )]
        );
        assert_eq!(rig.remote.pushes.borrow().len(), 1, "no plain push again");
        assert_eq!(host.opened.get(), 1, "no second pull request");
        assert_eq!(report.link.pr_number, 42);
        assert_eq!(report.link.head, "landed1");
        assert_eq!(rig.stored().head, "landed1");
        assert_eq!(*host.checked.borrow().last().unwrap(), "landed1");
    }

    #[test]
    fn a_rerun_at_the_same_commit_pushes_nothing() {
        let mut rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        rig.run(&host, false, "athena/bootstrap").unwrap();
        rig.remote.head = Some("cand1".into());
        rig.run(&host, false, "athena/bootstrap").unwrap();
        assert!(rig.remote.replaces.borrow().is_empty());
        assert_eq!(rig.remote.pushes.borrow().len(), 1);
    }

    /// Opens a pull request at `cand1`, then reruns for `landed1` and returns the error.
    fn rerun_error(
        remote_head: Option<&str>,
        branch: &str,
        ancestry: &dyn Ancestry,
    ) -> (Rig, anyhow::Error) {
        let mut rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        rig.run(&host, false, "athena/bootstrap").unwrap();
        rig.remote.head = remote_head.map(str::to_string);
        let err = rig
            .run_with(&host, false, branch, "landed1", ancestry)
            .unwrap_err();
        assert_eq!(host.opened.get(), 1);
        (rig, err)
    }

    #[test]
    fn an_ancestry_error_on_the_replace_path_pushes_nothing() {
        struct Broken;
        impl Ancestry for Broken {
            fn is_ancestor(&self, _: &str, _: &str) -> Result<bool> {
                bail!("merge-base blew up")
            }
        }
        let (rig, err) = rerun_error(Some("cand1"), "athena/bootstrap", &Broken);
        assert!(format!("{err:#}").contains("merge-base blew up"));
        assert!(rig.remote.replaces.borrow().is_empty());
        assert_eq!(rig.remote.pushes.borrow().len(), 1);
        assert_eq!(rig.stored().head, "cand1");
    }

    #[test]
    fn a_remote_head_unknown_locally_pushes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        git.run(["commit", "-q", "--allow-empty", "-m", "base"])
            .unwrap();
        let unknown = "0123456789abcdef0123456789abcdef01234567";
        let (rig, err) = rerun_error(Some(unknown), "athena/bootstrap", &git);
        assert!(format!("{err:#}").contains(unknown));
        assert!(rig.remote.replaces.borrow().is_empty());
        // A remote branch that cannot be read at all is also an error.
        let (rig, _) = rerun_error(None, "athena/bootstrap", &Anc(true));
        assert!(rig.remote.replaces.borrow().is_empty());
    }

    #[test]
    fn the_replace_path_is_refused_for_any_other_branch() {
        let (rig, err) = rerun_error(Some("cand1"), "main", &Anc(true));
        assert!(err.downcast_ref::<crate::pr::PushRefusal>().is_some());
        assert!(rig.remote.replaces.borrow().is_empty());
        assert_eq!(rig.remote.pushes.borrow().len(), 1);
    }

    /// A real remote whose branch moves right after Athena reads it.
    struct Racy<'a> {
        inner: GitRemote<'a>,
        moved_to: &'a str,
    }
    impl Remote for Racy<'_> {
        fn remote_head(&self, branch: &str) -> Result<Option<String>> {
            let seen = self.inner.remote_head(branch)?;
            self.inner.push(branch, self.moved_to)?;
            Ok(seen)
        }
        fn push(&self, branch: &str, commit: &str) -> Result<()> {
            self.inner.push(branch, commit)
        }
        fn replace(&self, branch: &str, commit: &str, expected: &str) -> Result<()> {
            self.inner.replace(branch, commit, expected)
        }
    }

    #[test]
    fn the_replacement_is_atomic_and_a_remote_that_moved_is_reported_and_left_alone() {
        let bare_dir = tempfile::tempdir().unwrap();
        let bare = Git::isolated(bare_dir.path());
        bare.run(["init", "-q", "--bare", "-b", "main"]).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        let step = |text: &str| {
            std::fs::write(dir.path().join("a.txt"), text).unwrap();
            git.run(["add", "-A"]).unwrap();
            git.run(["commit", "-q", "-m", text]).unwrap();
            git.head().unwrap().unwrap()
        };
        let base = step("base");
        let candidate = step("candidate");
        let moved = step("moved");
        git.run(["checkout", "-q", "-b", "landed", &base]).unwrap();
        let landed = step("landed");
        git.run(["remote", "add", "origin", bare_dir.path().to_str().unwrap()])
            .unwrap();
        let branch = "athena/bootstrap";
        git.run([
            "push",
            "-q",
            "origin",
            &format!("{candidate}:refs/heads/{branch}"),
        ])
        .unwrap();

        let rig = Rig::new();
        let state = StateDir::new(rig.dir.path().to_path_buf());
        crate::pr::store_link(
            &state,
            &rig.job,
            &PrLink {
                pr_number: 42,
                host: HOST.into(),
                repository: "example/project".into(),
                head: candidate.clone(),
                job: "bootstrap".into(),
                state: LinkState::Open,
                merged_commit: None,
            },
        )
        .unwrap();
        let host = FakeHost::new(Checks::Green);
        let nap = |_: Duration| {};
        let run = |remote: &dyn Remote| {
            land_pr(&LandPr {
                state: &state,
                job: &rig.job,
                branch,
                commit: &landed,
                verified_fingerprint: "verified",
                pr: PrDraft {
                    title: "t",
                    body: "b",
                    head: branch,
                },
                squash: &text(),
                host_name: HOST,
                repository: "example/project",
                merge_via_rest: false,
                poll: Poll {
                    attempts: 1,
                    delay: Duration::ZERO,
                    sleep: &nap,
                },
                ancestry: &git,
                remote,
                host: &host,
                content: &rig.content,
            })
        };
        let plain = GitRemote {
            git: &git,
            name: "origin",
        };

        // The remote moves after it is read: the push is rejected, reported, and changes nothing.
        let racy = Racy {
            inner: GitRemote {
                git: &git,
                name: "origin",
            },
            moved_to: &moved,
        };
        assert!(run(&racy).is_err());
        let held = bare.rev_parse(&format!("refs/heads/{branch}")).unwrap();
        assert_eq!(held.as_deref(), Some(moved.as_str()));
        assert_eq!(
            load_link(&state, &rig.job).unwrap().unwrap().head,
            candidate
        );

        // Nothing moves it: the branch becomes the landed commit, on the same pull request.
        let report = run(&plain).unwrap();
        let held = bare.rev_parse(&format!("refs/heads/{branch}")).unwrap();
        assert_eq!(held.as_deref(), Some(landed.as_str()));
        assert_eq!(report.link.pr_number, 42);
        assert_eq!(report.link.head, landed);
        assert_eq!(host.opened.get(), 0);
    }

    #[test]
    fn a_push_the_guard_refuses_opens_nothing() {
        // Another branch.
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        assert!(rig.run(&host, true, "main").is_err());
        assert!(rig.remote.pushes.borrow().is_empty());
        assert_eq!(host.opened.get(), 0);

        // A remote branch that is not an ancestor of the candidate.
        let mut rig = Rig::new();
        rig.remote.head = Some("rewritten".into());
        let state = StateDir::new(rig.dir.path().to_path_buf());
        let nap = |_: Duration| {};
        let host = FakeHost::new(Checks::Green);
        let refused = land_pr(&LandPr {
            state: &state,
            job: &rig.job,
            branch: "athena/bootstrap",
            commit: "cand1",
            verified_fingerprint: "verified",
            pr: PrDraft {
                title: "t",
                body: "b",
                head: "athena/bootstrap",
            },
            squash: &text(),
            host_name: HOST,
            repository: "example/project",
            merge_via_rest: true,
            poll: Poll {
                attempts: 1,
                delay: Duration::ZERO,
                sleep: &nap,
            },
            ancestry: &Anc(false),
            remote: &rig.remote,
            host: &host,
            content: &rig.content,
        });
        assert!(refused.is_err());
        assert!(rig.remote.pushes.borrow().is_empty());
        assert_eq!(host.opened.get(), 0);
        assert_eq!(load_link(&state, &rig.job).unwrap(), None);
    }

    // ---- refused host calls: the status, the record and the bounded retry ----

    #[test]
    fn the_status_is_read_from_the_text_gh_prints() {
        // The form `gh api` prints for a refused call.
        assert_eq!(parse_status("gh: Method Not Allowed (HTTP 405)"), Some(405));
        assert_eq!(parse_status("gh: Server Error (HTTP 500)\n"), Some(500));
        assert_eq!(
            parse_status("gh: Merge failed (HTTP 405) after HTTP 500"),
            Some(405)
        );
        assert_eq!(parse_status("gh: connection refused"), None);
        assert_eq!(parse_status("HTTP 4050"), None);
        assert_eq!(parse_status("HTTP 999"), None);
        assert_eq!(parse_status("HTTP 40"), None);
        assert_eq!(parse_status(""), None);
        for status in [500, 502, 503, 504] {
            assert!(is_transient(Some(status)));
        }
        for status in [400, 404, 405, 422, 501] {
            assert!(!is_transient(Some(status)));
        }
        assert!(!is_transient(None));
    }

    /// A host that fails the merge request with the given statuses, one per call, then (unless `always` is
    /// set) lets it through.
    struct Refusing {
        failures: RefCell<std::collections::VecDeque<Option<u16>>>,
        always: Option<Option<u16>>,
        merged: Cell<bool>,
        merge_calls: Cell<u32>,
        other_calls: Cell<u32>,
    }
    impl Refusing {
        fn new(failures: &[Option<u16>], always: Option<Option<u16>>) -> Self {
            Refusing {
                failures: RefCell::new(failures.iter().copied().collect()),
                always,
                merged: Cell::new(false),
                merge_calls: Cell::new(0),
                other_calls: Cell::new(0),
            }
        }
    }
    impl Runner for Refusing {
        fn gh(&self, args: &[String]) -> Result<String> {
            let path = args.iter().find(|a| a.starts_with("repos/")).unwrap();
            if !path.ends_with("/merge") {
                self.other_calls.set(self.other_calls.get() + 1);
                return Ok(if path == "repos/example/project" {
                    ALL_METHODS.to_string()
                } else if path.ends_with("/pulls") {
                    r#"{"number": 7}"#.to_string()
                } else if path.contains("/pulls/") {
                    if self.merged.get() {
                        r#"{"merged": true, "merge_commit_sha": "merged1"}"#.to_string()
                    } else {
                        r#"{"merged": false}"#.to_string()
                    }
                } else if path.contains("check-runs") {
                    r#"{"check_runs": [{"status": "completed", "conclusion": "success"}]}"#
                        .to_string()
                } else {
                    r#"{"state": "success", "statuses": []}"#.to_string()
                });
            }
            self.merge_calls.set(self.merge_calls.get() + 1);
            let refusal = match self.always {
                Some(status) => Some(status),
                None => self.failures.borrow_mut().pop_front(),
            };
            match refusal {
                Some(status_code) => Err(GhFailure {
                    call: "api".into(),
                    status_code,
                    text: "refused".into(),
                }
                .into()),
                None => {
                    self.merged.set(true);
                    Ok("{}".to_string())
                }
            }
        }
    }

    /// Lands over REST through a client on `runner`, with a log of its refusals.
    fn land_through(runner: &Refusing, rig: &Rig) -> (Result<PrReport>, Vec<HostError>) {
        let log = HostErrorLog::new(rig.job.clone(), WorkPhase::Landing);
        let client = GhClient {
            log: Some(&log),
            retry: Retry {
                attempts: 3,
                delay: Duration::ZERO,
            },
            ..GhClient::new(runner, "example/project".into())
        };
        let result = rig.run_with(&client, true, "athena/bootstrap", "cand1", &Anc(true));
        (result, log.entries())
    }

    #[test]
    fn a_500_twice_then_success_lands_with_two_host_errors_and_no_error_for_the_user() {
        let runner = Refusing::new(&[Some(500), Some(500)], None);
        let rig = Rig::new();
        let (result, errors) = land_through(&runner, &rig);
        let report = result.expect("a transient failure is retried, not shown");
        assert_eq!(report.link.state, LinkState::Landed);
        assert!(report.merge_requested);
        assert_eq!(runner.merge_calls.get(), 3);
        assert_eq!(errors.len(), 2);
        for error in &errors {
            assert_eq!(error.status_code, Some(500));
            assert_eq!(error.operation, HostOperation::Merge);
            assert_eq!(error.phase, WorkPhase::Landing);
            assert_eq!(error.job, rig.job);
            assert_eq!(error.message_class, MessageClass::ServerError);
        }
        assert_ne!(errors[0].id, errors[1].id);
    }

    #[test]
    fn a_persistent_500_stops_at_the_bound_and_the_error_reaches_the_user() {
        let runner = Refusing::new(&[], Some(Some(500)));
        let rig = Rig::new();
        let (result, errors) = land_through(&runner, &rig);
        let error = result.unwrap_err();
        let failure = error.downcast_ref::<GhFailure>().expect("the host's error");
        assert_eq!(failure.status_code, Some(500));
        assert_eq!(runner.merge_calls.get(), 3, "three calls in all, then stop");
        assert_eq!(errors.len(), 3);
        // The pull request was opened and stays open: a rerun finds it by its number.
        assert_eq!(rig.stored().state, LinkState::Open);
    }

    #[test]
    fn a_refusal_that_is_not_transient_is_recorded_once_and_not_retried() {
        for status in [Some(405), Some(404), Some(422), Some(403), Some(501), None] {
            let runner = Refusing::new(&[], Some(status));
            let rig = Rig::new();
            let (result, errors) = land_through(&runner, &rig);
            assert!(result.is_err(), "{status:?}");
            assert_eq!(runner.merge_calls.get(), 1, "{status:?}");
            assert_eq!(errors.len(), 1, "{status:?}");
            assert_eq!(errors[0].status_code, status);
            assert_eq!(errors[0].operation, HostOperation::Merge);
            assert_eq!(errors[0].message_class, MessageClass::of_status(status));
        }
    }

    #[test]
    fn a_failure_that_is_not_a_host_refusal_is_not_recorded() {
        struct Missing;
        impl Runner for Missing {
            fn gh(&self, _: &[String]) -> Result<String> {
                bail!("running gh; is the GitHub client installed?")
            }
        }
        let log = HostErrorLog::new(JobId::parse("bootstrap").unwrap(), WorkPhase::Landing);
        let client = GhClient {
            log: Some(&log),
            ..GhClient::new(&Missing, "example/project".into())
        };
        assert!(client.default_branch().is_err());
        assert!(log.entries().is_empty());
    }

    fn failure(status_code: Option<u16>, text: &str) -> GhFailure {
        GhFailure {
            call: "repos/example/project".into(),
            status_code,
            text: text.into(),
        }
    }

    #[test]
    fn timeouts_and_connection_resets_are_transient_and_auth_and_4xx_are_not() {
        let reason = |f: GhFailure| f.transient_reason();
        assert_eq!(reason(failure(Some(503), "")).as_deref(), Some("HTTP 503"));
        assert_eq!(
            reason(failure(None, "Get \"https://api.github.com\": i/o timeout")).as_deref(),
            Some("timeout")
        );
        assert_eq!(
            reason(failure(None, "read tcp: connection reset by peer")).as_deref(),
            Some("connection reset")
        );
        for (status, text) in [
            (Some(404), "Not Found"),
            (Some(401), "Bad credentials"),
            (Some(403), "Forbidden"),
            // A status decides on its own, whatever the words.
            (Some(404), "request timeout"),
            (
                None,
                "You are not logged into any GitHub hosts. Run gh auth login",
            ),
            (None, "gh: unknown failure"),
        ] {
            assert_eq!(reason(failure(status, text)), None, "{status:?} {text}");
        }
    }

    #[test]
    fn the_wait_doubles_with_each_failure_and_jitter_adds_up_to_half() {
        let retry = Retry {
            attempts: 3,
            delay: Duration::from_secs(2),
        };
        assert_eq!(retry.wait(1, 0), Duration::from_secs(2));
        assert_eq!(retry.wait(2, 0), Duration::from_secs(4));
        assert_eq!(retry.wait(2, 999), Duration::from_millis(4000 + 1998));
        assert_eq!(retry.wait(1, 1000), retry.wait(1, 0));
        let none = Retry {
            attempts: 3,
            delay: Duration::ZERO,
        };
        assert_eq!(none.wait(3, 500), Duration::ZERO);
    }

    /// Fails each call with the given failures in turn, then answers with an empty object.
    struct Scripted {
        failures: RefCell<std::collections::VecDeque<GhFailure>>,
        calls: Cell<u32>,
    }
    impl Runner for Scripted {
        fn gh(&self, _: &[String]) -> Result<String> {
            self.calls.set(self.calls.get() + 1);
            match self.failures.borrow_mut().pop_front() {
                Some(f) => Err(f.into()),
                None => Ok(r#"{"default_branch": "trunk"}"#.to_string()),
            }
        }
    }

    struct Lines(RefCell<Vec<String>>);
    impl Ui for Lines {
        fn line(&self, text: &str) {
            self.0.borrow_mut().push(text.to_string());
        }
    }

    /// Calls `default_branch` against a runner that fails with `failures`; returns the outcome, the number of
    /// calls and the retry lines.
    fn scripted(failures: Vec<GhFailure>) -> (Result<String>, u32, Vec<String>) {
        let runner = Scripted {
            failures: RefCell::new(failures.into()),
            calls: Cell::new(0),
        };
        let lines = Lines(RefCell::new(vec![]));
        let client = GhClient {
            retry: Retry {
                attempts: 3,
                delay: Duration::ZERO,
            },
            ui: Some(&lines),
            ..GhClient::new(&runner, "example/project".into())
        };
        let result = client.default_branch();
        (result, runner.calls.get(), lines.0.into_inner())
    }

    #[test]
    fn timeouts_and_resets_are_retried_and_404_and_auth_failures_are_not() {
        for text in ["dial tcp: i/o timeout", "read: connection reset by peer"] {
            let (result, calls, lines) = scripted(vec![failure(None, text)]);
            assert_eq!(result.unwrap(), "trunk", "{text}");
            assert_eq!(calls, 2, "{text}");
            assert_eq!(lines.len(), 1);
        }
        for (status, text) in [
            (Some(404), "Not Found"),
            (Some(401), "Bad credentials"),
            (Some(403), "Forbidden"),
            (None, "You are not logged into any GitHub hosts"),
        ] {
            let (result, calls, lines) = scripted(vec![failure(status, text); 3]);
            assert!(result.is_err());
            assert_eq!(calls, 1, "{status:?} {text}");
            assert!(lines.is_empty());
        }
    }

    #[test]
    fn each_retry_is_logged_with_the_command_status_and_attempt_and_the_last_error_names_them() {
        let (result, calls, lines) = scripted(vec![failure(Some(500), "boom"); 3]);
        assert_eq!(calls, 3);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("repos/example/project"), "{lines:?}");
        assert!(lines[0].contains("HTTP 500"), "{lines:?}");
        assert!(lines[0].contains("attempt 1 of 3"), "{lines:?}");
        assert!(lines[1].contains("attempt 2 of 3"), "{lines:?}");
        let error = result.unwrap_err();
        let message = error.to_string();
        assert!(message.contains("repos/example/project"), "{message}");
        assert!(message.contains("HTTP 500"), "{message}");
        assert!(message.contains("3 attempts"), "{message}");
        assert!(error.downcast_ref::<GhFailure>().is_some());
    }

    #[test]
    fn the_stored_record_has_no_text_of_the_host() {
        let runner = Refusing::new(&[], Some(Some(405)));
        let rig = Rig::new();
        let (_, errors) = land_through(&runner, &rig);
        let json = athena_data::records::AnyRecord::HostError(errors[0].clone()).to_json();
        assert!(!json.contains("refused"), "{json}");
        assert!(json.contains("\"status_code\": 405"), "{json}");
    }
}
