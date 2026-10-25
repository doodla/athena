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

use crate::land::pr_commits;
use crate::pr::{
    Ancestry, Content, GitContent, Host, LinkState, PrLink, PushRequest, check_push, load_link,
    observe_merge, store_link,
};
use crate::reconcile::{CommitHost, MergedPr, ReconcileRecords, RecordsSnapshot};
use crate::records_import::{MANIFEST_PATH, import_at_cutover};
use crate::state::StateDir;
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_data::history::{self, HistoryRecord, Landing, LandingCommit, LandingIntent};
use athena_data::records::{HostError, HostOperation, MessageClass, Verdict, WorkPhase};
use athena_data::{JobId, RecordId};
use athena_git::records_branch::{Change, RecordsWriter};
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

/// The arguments of a plain read of a REST path: `gh api <path>`. `gh api` defaults to GET, so no method
/// override and no field or body argument is added.
pub fn get_args(path: &str) -> Vec<String> {
    vec!["api".to_string(), path.to_string()]
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
        transient_text(&self.text)
    }
}

/// Why a failure with no HTTP status is worth asking again, read from the client's own words: a timeout or a
/// connection reset. Anything else, such as "not logged in", is not.
pub fn transient_text(text: &str) -> Option<String> {
    let text = text.to_ascii_lowercase();
    if text.contains("timeout") || text.contains("timed out") {
        Some("timeout".to_string())
    } else if text.contains("connection reset") || text.contains("econnreset") {
        Some("connection reset".to_string())
    } else {
        None
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

/// Runs `call` and repeats it while it fails transiently, up to the bound of `retry`. `transient` says whether
/// an error is worth asking again, and why; any other error, and the last transient one, is returned. Each
/// repeat is reported to `ui` as one line naming `what`, the reason and the attempt. The wait between calls is
/// [`Retry::wait`].
pub fn retry_transient<T, E>(
    retry: Retry,
    ui: Option<&dyn Ui>,
    what: &str,
    mut call: impl FnMut() -> Result<T, E>,
    transient: impl Fn(&E) -> Option<String>,
) -> Result<T, E> {
    let attempts = retry.attempts.max(1);
    let mut attempt = 1;
    loop {
        match call() {
            Ok(value) => return Ok(value),
            Err(error) => {
                let Some(reason) = transient(&error) else {
                    return Err(error);
                };
                if attempt >= attempts {
                    return Err(error);
                }
                let wait = retry.wait(attempt, clock_entropy());
                if let Some(ui) = ui {
                    ui.line(&format!(
                        "retrying {what} after {reason} (attempt {attempt} of {attempts} failed; waiting {} ms)",
                        wait.as_millis()
                    ));
                }
                std::thread::sleep(wait);
                attempt += 1;
            }
        }
    }
}

/// What an HTTP server answered: the status and the body, whether or not it is a success.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpReply {
    pub status: u16,
    pub body: String,
}

/// Posts a JSON body and returns the reply, whatever its status. An `Err` means no reply was received (the
/// connection failed, timed out, or the client program could not run). The bearer token, when there is one, is
/// the caller's secret: an implementation never writes it to a file, a log or an error.
///
/// `Send + Sync` so that a client holding one (the Jev client inside [`crate::Athena`]) can be shared by the
/// workers of a parallel drain.
pub trait HttpPost: Send + Sync {
    fn post_json(&self, url: &str, bearer: Option<&str>, body: &str) -> Result<HttpReply>;
}

/// [`HttpPost`] over the installed `curl` program, the same way [`GhCli`] runs `gh`: no HTTP crate is added to
/// the workspace. The request settings, including the token header, go to the program on its standard input,
/// so the token is never in its argument list; the body goes through a temporary file that is removed after.
/// A proxy named by the usual environment variables is used by the program itself.
#[derive(Debug, Clone, Copy)]
pub struct CurlHttp {
    /// The most seconds one call may take.
    pub max_seconds: u64,
}

impl Default for CurlHttp {
    fn default() -> Self {
        CurlHttp { max_seconds: 60 }
    }
}

impl HttpPost for CurlHttp {
    fn post_json(&self, url: &str, bearer: Option<&str>, body: &str) -> Result<HttpReply> {
        use std::io::Write;
        let unsafe_text = |s: &str| s.contains(['"', '\\', '\n', '\r']);
        if unsafe_text(url) || bearer.is_some_and(unsafe_text) {
            bail!("the request has a character that cannot be sent safely");
        }
        let mut body_file = tempfile::NamedTempFile::new().context("preparing the request body")?;
        body_file
            .write_all(body.as_bytes())
            .context("preparing the request body")?;
        let mut config = format!(
            "silent\nshow-error\nrequest = \"POST\"\nheader = \"Content-Type: application/json\"\n\
             max-time = {}\nwrite-out = \"\\n%{{http_code}}\"\nurl = \"{url}\"\ndata-binary = \"@{}\"\n",
            self.max_seconds,
            body_file.path().display()
        );
        if let Some(token) = bearer {
            config.push_str(&format!("header = \"Authorization: Bearer {token}\"\n"));
        }
        let mut child = Command::new("curl")
            .args(["--config", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("running curl; is it installed?")?;
        child
            .stdin
            .take()
            .context("curl has no standard input")?
            .write_all(config.as_bytes())
            .context("sending the request settings to curl")?;
        let out = child.wait_with_output().context("waiting for curl")?;
        let scrub = |text: &str| match bearer {
            Some(token) if !token.is_empty() => text.replace(token, "<key>"),
            _ => text.to_string(),
        };
        if !out.status.success() {
            bail!(
                "the request failed: {}",
                scrub(String::from_utf8_lossy(&out.stderr).trim())
            );
        }
        let text = String::from_utf8_lossy(&out.stdout);
        let (body, code) = text
            .rsplit_once('\n')
            .context("the reply has no status line")?;
        let status = code
            .trim()
            .parse()
            .context("the reply has no HTTP status")?;
        Ok(HttpReply {
            status,
            body: body.to_string(),
        })
    }
}

impl CurlHttp {
    /// The text of the page at an `http` or `https` URL, following redirects. An `Err` means no page: the
    /// URL is not http, the program could not run, the call timed out, or the server answered with an error
    /// status. No credential is sent.
    pub fn get_text(&self, url: &str) -> Result<String> {
        if !(url.starts_with("https://") || url.starts_with("http://")) {
            bail!("{url:?} is not an http or https URL");
        }
        let out = Command::new("curl")
            .args(["--fail", "--silent", "--show-error", "--location"])
            .args(["--max-time", &self.max_seconds.to_string(), "--"])
            .arg(url)
            .output()
            .context("running curl; is it installed?")?;
        if !out.status.success() {
            bail!(
                "could not fetch {url}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
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
    /// Returns the method the host was asked to use, which the landing record names (§5.3).
    fn merge(&self, number: u64, head: &str, text: &CommitText) -> Result<MergeMethod>;
    /// The workflow runs of a commit: whether each is still going, how it ended, and which try it is on.
    fn ci_runs(&self, commit: &str) -> Result<Vec<CiRunState>>;
    /// Asks the host to run the failed jobs of one completed workflow run again. The host refuses while the
    /// run is still in progress.
    fn rerun_failed(&self, run_id: u64) -> Result<()>;
    /// The name of the first test that failed in the latest try of a completed run, read from the failed
    /// job's log; none when the log names none.
    fn failing_test(&self, run_id: u64) -> Result<Option<String>>;
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

/// The longest test name kept from a log.
const TEST_NAME_LIMIT: usize = 200;

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

/// The remote git repository the branch is pushed to.
pub trait Remote {
    /// The commit the remote branch holds, or `None` when it does not exist.
    fn remote_head(&self, branch: &str) -> Result<Option<String>>;
    /// A regular push of `commit` to `branch`. There is no force option.
    fn push(&self, branch: &str, commit: &str) -> Result<()>;
    /// The commits after `landed` up to `head`, oldest first, as (full sha, subject). Empty when `head` is
    /// `landed` or does not descend from it. There is deliberately no way to replace a remote head.
    fn commits_after(&self, landed: &str, head: &str) -> Result<Vec<(String, String)>>;
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

    fn commits_after(&self, landed: &str, head: &str) -> Result<Vec<(String, String)>> {
        self.git.ensure_commit(self.name, head)?;
        let listing = self.git.run([
            "log",
            "--reverse",
            "--format=%H %s",
            &format!("{landed}..{head}"),
        ])?;
        Ok(listing
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| match l.split_once(' ') {
                Some((sha, subject)) => (sha.to_string(), subject.to_string()),
                None => (l.to_string(), String::new()),
            })
            .collect())
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
    /// Where the landing-intent and landing records are written (§5.2, §5.3 of
    /// `docs/design/records-history.md`). `Athena::land_pr` always passes one, built by
    /// [`GitLandingRecords::for_remote`]; `None` skips the records and is for tests of the rest of the flow.
    pub records: Option<&'a dyn LandingRecords>,
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
    /// How many times the failed jobs were re-run: 0 or 1.
    pub reruns: u32,
    /// The first failing test's name, read from the failed job's log, when the checks were red.
    pub failing_test: Option<String>,
    /// Things the caller must be told that are not failures: a merge whose landing record was not written,
    /// naming the landing-intent record that reconciliation will use (§8.1).
    pub warnings: Vec<String>,
}

/// The records `land_pr` writes around the merge (§5.2, §5.3, §8 of `docs/design/records-history.md`).
pub trait LandingRecords {
    /// The commit the remote's default branch `branch` holds now: the intent's expected base, and the lower
    /// bound of the pull request's commits.
    fn base_tip(&self, branch: &str) -> Result<String>;
    /// Writes the landing-intent record for the pull request, **before** the merge, and returns its id. The
    /// commits in `expected_base..head` are checked first; one without a verification is an error and
    /// nothing is written. The intent of an earlier run for the same pull request and head is reused.
    fn intent(&self, pr_number: u64, head: &str, expected_base: &str) -> Result<RecordId>;
    /// Writes the landing record, **after** the merge was confirmed, and returns its id. Its records commit
    /// takes `head` as a second parent (§6.1).
    fn landing(&self, landing: &LandingFacts) -> Result<RecordId>;
    /// Runs after the landing record was written, with the host commit the merge made. The records-history
    /// cutover hooks in here (`docs/design/records-history.md` §4 and §10, D111): when the landed commit's
    /// `athena.toml` names a `[history] cutover_job`, the import at that job's `C0` and the legacy-exempt
    /// record are written if they are not on the records branch yet. Returns a line to show when something was
    /// written. The default does nothing.
    fn cutover(&self, _host_commit: &str) -> Result<Option<String>> {
        Ok(None)
    }
}

/// What is known about a merge once the host reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandingFacts {
    pub intent: RecordId,
    pub expected_base: String,
    pub pr_number: u64,
    pub head: String,
    pub host_commit: String,
    pub merge_method: MergeMethod,
}

/// The verification record that covers a commit of `job`. A verification has no id of its own: there is
/// exactly one per job, stored under the job's directory, so the job's id names it (§5.3). The bootstrap job
/// has no such id, so a commit of it cannot be recorded and the error names the commit. If a verification
/// ever gets an id of its own, this is the one place to change.
pub fn verification_id(job: &JobId, commit: &str) -> Result<RecordId> {
    match job {
        JobId::Id(id) => Ok(id.clone()),
        JobId::Bootstrap => bail!(
            "commit {commit} belongs to the bootstrap job, which has no verification record to name in a \
             landing record"
        ),
    }
}

/// Directory of landing-intent files on the records branch.
const INTENT_DIR: &str = "landing-intents";
/// Directory of landing files on the records branch.
const LANDING_DIR: &str = "landings";

fn history_method(method: MergeMethod) -> history::MergeMethod {
    match method {
        MergeMethod::Merge => history::MergeMethod::Merge,
        MergeMethod::Squash => history::MergeMethod::Squash,
        MergeMethod::Rebase => history::MergeMethod::Rebase,
    }
}

/// [`LandingRecords`] over a git repository and one of its remotes: records go to the remote's
/// `athena/records` branch through [`RecordsWriter::append_to_remote`] (§8.2), which builds on the remote
/// tip and moves the local branch only after the push succeeded.
#[derive(Debug)]
pub struct GitLandingRecords<'a> {
    git: &'a Git,
    remote: String,
}

impl<'a> GitLandingRecords<'a> {
    /// The real sink for `git` and the remote named `remote`. The remote must exist, so a landing cannot go
    /// ahead with nowhere to put its records (§5.2).
    pub fn for_remote(git: &'a Git, remote: &str) -> Result<Self> {
        if remote.trim().is_empty() {
            bail!("no remote is named, so landing records have nowhere to go");
        }
        git.run(["remote", "get-url", remote]).with_context(|| {
            format!("remote {remote:?} is not configured, so landing records have nowhere to go")
        })?;
        Ok(GitLandingRecords {
            git,
            remote: remote.to_string(),
        })
    }

    /// The history records under `dir` on the remote's records branch (the local branch is first reset to
    /// it, see [`RecordsWriter::sync_from`]).
    fn existing(&self, dir: &str) -> Result<Vec<HistoryRecord>> {
        let Some(tip) = RecordsWriter::new(self.git).sync_from(&self.remote)? else {
            return Ok(Vec::new());
        };
        self.records_under(&tip, dir)
    }

    /// The history records under `dir` at the records commit `tip`.
    fn records_under(&self, tip: &str, dir: &str) -> Result<Vec<HistoryRecord>> {
        let oids: Vec<String> = self
            .git
            .ls_tree_under(tip, dir)?
            .into_iter()
            .map(|e| e.oid)
            .collect();
        self.git
            .cat_blobs(&oids)?
            .iter()
            .map(|bytes| {
                HistoryRecord::parse(&String::from_utf8_lossy(bytes))
                    .context("an existing record on the records branch is not valid")
            })
            .collect()
    }

    /// The pull request's commits, each with its kind and, for a job landing, the job and the verification
    /// record that covers it, all read from the commit's content (§5.3).
    fn commits(&self, expected_base: &str, head: &str) -> Result<Vec<LandingCommit>> {
        let commits: Vec<LandingCommit> = pr_commits(self.git, expected_base, head)?
            .into_iter()
            .map(|c| LandingCommit {
                sha: c.sha,
                tree: c.tree,
                kind: c.kind,
                job: c.job,
                verification: c.verification,
            })
            .collect();
        for commit in &commits {
            commit.validate()?;
        }
        Ok(commits)
    }

    /// Validates `record`, then writes it as `<dir>/<id>.json` in one commit with `extra_parents`.
    fn write(
        &self,
        dir: &str,
        id: &RecordId,
        record: &HistoryRecord,
        message: &str,
        extra_parents: &[&str],
    ) -> Result<()> {
        record.validate()?;
        let bytes = serde_json::to_vec_pretty(record)?;
        RecordsWriter::new(self.git).append_to_remote(
            &self.remote,
            message,
            &[Change::put(format!("{dir}/{id}.json"), bytes)],
            extra_parents,
        )?;
        Ok(())
    }
}

impl LandingRecords for GitLandingRecords<'_> {
    fn base_tip(&self, branch: &str) -> Result<String> {
        self.git
            .fetch_remote_branch(&self.remote, branch)?
            .with_context(|| {
                format!(
                    "the remote {} has no branch {branch}, so the expected base cannot be recorded",
                    self.remote
                )
            })
    }

    fn intent(&self, pr_number: u64, head: &str, expected_base: &str) -> Result<RecordId> {
        let commits = self.commits(expected_base, head)?;
        for record in self.existing(INTENT_DIR)? {
            if let HistoryRecord::LandingIntent(found) = record
                && found.pr_number == pr_number
                && found.head_sha == head
            {
                return Ok(found.id);
            }
        }
        let mut job_ids: Vec<JobId> = Vec::new();
        for job in commits.iter().filter_map(|c| c.job.as_ref()) {
            if !job_ids.contains(job) {
                job_ids.push(job.clone());
            }
        }
        if job_ids.is_empty() {
            bail!(
                "no commit between the remote base {expected_base} and the PR head {head} lands a job, so \
                 a landing intent has no job to name; nothing was recorded"
            );
        }
        let now = crate::now();
        let intent = LandingIntent {
            id: RecordId::generate(now),
            created_at: now,
            pr_number,
            head_sha: head.to_string(),
            head_tree_sha: self.git.tree_of(head)?,
            expected_base: expected_base.to_string(),
            job_ids,
        };
        let id = intent.id.clone();
        self.write(
            INTENT_DIR,
            &id,
            &HistoryRecord::LandingIntent(intent),
            &format!("Landing intent for pull request #{pr_number}"),
            &[],
        )?;
        Ok(id)
    }

    fn landing(&self, facts: &LandingFacts) -> Result<RecordId> {
        let commits = self.commits(&facts.expected_base, &facts.head)?;
        for record in self.existing(LANDING_DIR)? {
            if let HistoryRecord::Landing(found) = record
                && found.intent == facts.intent
            {
                return Ok(found.id);
            }
        }
        let now = crate::now();
        let landing = Landing {
            id: RecordId::generate(now),
            created_at: now,
            pr_number: facts.pr_number,
            head_sha: facts.head.clone(),
            host_commit: facts.host_commit.clone(),
            merge_method: history_method(facts.merge_method),
            commits,
            intent: facts.intent.clone(),
        };
        let id = landing.id.clone();
        self.write(
            LANDING_DIR,
            &id,
            &HistoryRecord::Landing(landing),
            &format!("Landing record for pull request #{}", facts.pr_number),
            &[facts.head.as_str()],
        )?;
        Ok(id)
    }

    fn cutover(&self, host_commit: &str) -> Result<Option<String>> {
        // The landed commit's own `athena.toml` names the cutover job: the file that adds the setting is part
        // of the merge that lands it. A commit that cannot be fetched cannot be read; the reconcile catch-up
        // then finishes the work from the default branch's tip.
        if !self
            .git
            .ensure_commit(&self.remote, host_commit)
            .unwrap_or(false)
        {
            return Ok(None);
        }
        let Some(job) = cutover_job_for(self.git, host_commit) else {
            return Ok(None);
        };
        Ok(self
            .run_cutover(&job)?
            .filter(CutoverRun::wrote)
            .map(|run| run.describe()))
    }
}

/// What one run of the cutover did (`docs/design/records-history.md` §4 and §10, D111).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutoverRun {
    /// `C0`: the host commit of the cutover job's landing record.
    pub c0: String,
    /// The records commit that imported `records/` at `c0` with the manifest, when this run wrote it.
    pub import: Option<String>,
    /// Whether this run wrote the legacy-exempt record.
    pub exempt_written: bool,
}

impl CutoverRun {
    /// Whether this run changed the records branch.
    pub fn wrote(&self) -> bool {
        self.import.is_some() || self.exempt_written
    }

    /// A line for a person.
    pub fn describe(&self) -> String {
        let mut did = Vec::new();
        if let Some(commit) = &self.import {
            did.push(format!(
                "imported records/ at C0 ({commit} holds the files and the manifest)"
            ));
        }
        if self.exempt_written {
            did.push("wrote the legacy-exempt record".to_string());
        }
        format!("cutover: {} for C0 {}", did.join(" and "), self.c0)
    }
}

/// Reads `[history] cutover_job` from the `athena.toml` at `commit`. `None` when the commit has no
/// `athena.toml`, it does not parse, or it sets no cutover job. The built-in configuration is not consulted:
/// the setting is only ever what a commit says.
///
/// The cutover job is read from the landed commit (after a merge) or the default branch's tip (in the
/// reconcile catch-up), never from the checkout a landing started in: the job that adds the setting lands
/// with a checkout that does not have it yet.
pub fn cutover_job_for(git: &Git, commit: &str) -> Option<String> {
    let bytes = git.show_file(commit, "athena.toml").ok()??;
    athena_data::config::Config::parse(&String::from_utf8_lossy(&bytes))
        .ok()?
        .history
        .cutover_job
}

impl GitLandingRecords<'_> {
    /// Runs the cutover for `job`, the configured `[history] cutover_job`, against the remote's records
    /// branch (the local branch is first reset to it). `C0` is the host commit of the landing record that
    /// lands `job`; `None` when there is none yet. Each of the two records is written only when it is not on
    /// the branch, each tested by its file: `import-manifest.json` for the import of `records/` at `C0`, and
    /// `legacy-exempt.json` for the legacy-exempt record, built from `[history] exempt` of `athena.toml` at
    /// `C0`. When both are missing they go in one commit, so the branch never shows an import without its
    /// exempt record. A second run writes nothing.
    pub fn run_cutover(&self, job: &str) -> Result<Option<CutoverRun>> {
        let Some(tip) = RecordsWriter::new(self.git).sync_from(&self.remote)? else {
            return Ok(None);
        };
        let Some(c0) = athena_gate::attribution::cutover_commit_at(self.git, &tip, job)? else {
            return Ok(None);
        };
        if !self.git.ensure_commit(&self.remote, &c0)? {
            bail!(
                "C0 {c0}, the host commit of the landing of job {job}, is not available from remote {}",
                self.remote
            );
        }
        let has = |path: &str| -> Result<bool> { Ok(self.git.show_file(&tip, path)?.is_some()) };
        let has_manifest = has(MANIFEST_PATH)?;
        let exempt = if has(athena_gate::legacy::LEGACY_EXEMPT_PATH)? {
            None
        } else {
            let now = crate::now();
            let config = athena_gate::config_at(self.git, Some(&c0))?;
            let record = athena_gate::legacy::legacy_exempt_record(
                &config.history.exempt,
                RecordId::generate(now),
                now,
            )?;
            Some(athena_gate::legacy::legacy_exempt_change(&record)?)
        };
        let mut run = CutoverRun {
            c0: c0.clone(),
            import: None,
            exempt_written: false,
        };
        if !has_manifest {
            let extra: Vec<Change> = exempt.iter().cloned().collect();
            run.import = Some(import_at_cutover(self.git, &self.remote, &c0, &extra)?);
            run.exempt_written = exempt.is_some();
        } else if let Some(change) = exempt {
            RecordsWriter::new(self.git).append_to_remote(
                &self.remote,
                &format!("Legacy-exempt record for the cutover at {c0}"),
                &[change],
                &[],
            )?;
            run.exempt_written = true;
        }
        Ok(Some(run))
    }
}

/// The cutover catch-up of the reconcile step (`athena land` and `athena status` run it first): reads
/// `[history] cutover_job` from `tip`, the fetched tip of the default branch, and runs the cutover for it, so
/// a landing whose own run stopped before the import still gets it. The lines are for a person; nothing is
/// returned when there is nothing to do, and a failure is a line, never an error.
pub fn cutover_catch_up(sink: &GitLandingRecords<'_>, tip: &str) -> Vec<String> {
    let Some(job) = cutover_job_for(sink.git, tip) else {
        return Vec::new();
    };
    match sink.run_cutover(&job) {
        Ok(Some(run)) if run.wrote() => vec![run.describe()],
        Ok(_) => Vec::new(),
        Err(problem) => vec![format!(
            "cutover: the import for job {job} did not finish ({problem:#}); it is tried again next time"
        )],
    }
}

/// Directory of declared-intervention files on the records branch (§5.3). Nothing writes it yet; a missing
/// directory reads as empty.
const DECLARED_DIR: &str = "declared-interventions";

impl ReconcileRecords for GitLandingRecords<'_> {
    fn snapshot(&self) -> Result<Option<RecordsSnapshot>> {
        let Some(tip) = RecordsWriter::new(self.git).sync_from(&self.remote)? else {
            return Ok(None);
        };
        let mut snapshot = RecordsSnapshot::default();
        for record in self.records_under(&tip, INTENT_DIR)? {
            if let HistoryRecord::LandingIntent(intent) = record {
                snapshot.intents.push(intent);
            }
        }
        for record in self.records_under(&tip, LANDING_DIR)? {
            if let HistoryRecord::Landing(landing) = record {
                snapshot.landings.push(landing);
            }
        }
        for record in self.records_under(&tip, DECLARED_DIR)? {
            if let HistoryRecord::DeclaredIntervention(declared) = record {
                snapshot.declared.push(declared);
            }
        }
        Ok(Some(snapshot))
    }

    fn write_landing(&self, facts: &LandingFacts) -> Result<RecordId> {
        // The head and the base must be present to list the pull request's commits.
        for commit in [&facts.head, &facts.expected_base] {
            if !self.git.ensure_commit(&self.remote, commit)? {
                bail!(
                    "commit {commit} is not available from remote {}",
                    self.remote
                );
            }
        }
        self.landing(facts)
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
    let mut reruns = 0;
    let mut failing_test = None;
    let mut warnings = Vec::new();
    observe_merge(job.host, job.content, &mut link, job.verified_fingerprint)?;
    if link.state == LinkState::Open {
        // The pull request's head is whatever its branch holds now, not the job's commit: a fix may have
        // been pushed since. Everything below judges and guards that head, and nothing replaces it.
        let Some(head) = job.remote.remote_head(job.branch)? else {
            bail!(
                "the pull request's branch {} is not on the remote, so its head cannot be judged",
                job.branch
            );
        };
        link.head = head.clone();
        store_link(job.state, job.job, &link)?;
        if head != job.commit {
            let seen = job.host.checks(&head);
            let extras = job
                .remote
                .commits_after(job.commit, &head)
                .unwrap_or_default();
            let text = refusal_text(link.pr_number, job.commit, &head, &seen, &extras);
            bail!("{text}");
        }
    }
    if link.state == LinkState::Open {
        let waited = wait_for_checks(job, &link.head)?;
        let seen = waited.checks;
        reruns = waited.reruns;
        failing_test = waited.failing_test;
        checks = Some(seen);
        // The merge gate: the setting is on and every check is green. Nothing else reaches the host's
        // merge endpoint.
        if job.merge_via_rest && seen == Checks::Green {
            // The landing-intent record is on the records branch of the remote before the host is asked to
            // merge (§5.2). An error here, such as a commit nothing verified, stops the landing before
            // anything is merged.
            let intent = match job.records {
                Some(records) => {
                    let base = records.base_tip(&job.host.default_branch()?)?;
                    let id = records.intent(link.pr_number, &link.head, &base)?;
                    Some((records, base, id))
                }
                None => None,
            };
            let method = job.host.merge(link.pr_number, &link.head, job.squash)?;
            merge_requested = true;
            observe_merge(job.host, job.content, &mut link, job.verified_fingerprint)?;
            if let Some((records, expected_base, intent)) = intent {
                match (&link.state, &link.merged_commit) {
                    (LinkState::Landed, Some(host_commit)) => {
                        let facts = LandingFacts {
                            intent: intent.clone(),
                            expected_base,
                            pr_number: link.pr_number,
                            head: link.head.clone(),
                            host_commit: host_commit.clone(),
                            merge_method: method,
                        };
                        // The merge is done and stays done: a landing record that cannot be written is a
                        // warning, and reconciliation (§8.1) writes it from the intent.
                        match records.landing(&facts) {
                            Err(problem) => warnings.push(format!(
                                "the pull request was merged but its landing record could not be written \
                                 ({problem:#}); landing-intent record {intent} is on the records branch for reconciliation"
                            )),
                            // The cutover runs after the landing record, and a failure of it is a warning
                            // too: the merge and the record stay done, and the reconcile step of the next
                            // `athena land` or `athena status` finishes the import.
                            Ok(_) => match records.cutover(&facts.host_commit) {
                                Ok(Some(line)) => warnings.push(line),
                                Ok(None) => {}
                                Err(problem) => warnings.push(format!(
                                    "the cutover import did not finish ({problem:#}); the next `athena land` \
                                     or `athena status` tries it again"
                                )),
                            },
                        }
                    }
                    _ => warnings.push(format!(
                        "a merge was requested but not confirmed, so no landing record was written; \
                         landing-intent record {intent} is on the records branch for reconciliation"
                    )),
                }
            }
        }
    }
    store_link(job.state, job.job, &link)?;
    Ok(PrReport {
        link,
        checks,
        merge_requested,
        reruns,
        failing_test,
        warnings,
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
        reruns: 0,
        failing_test: None,
        warnings: Vec::new(),
    }))
}

/// The refusal for a pull request whose head is not the job's landing commit. It names the head, how its
/// checks stand (one look, so the old commit's result is never what is reported), each extra commit, and why
/// the head is refused. Nothing is merged or pushed.
fn refusal_text(
    number: u64,
    landed: &str,
    head: &str,
    checks: &Result<Checks>,
    extras: &[(String, String)],
) -> String {
    let short = |sha: &str| sha.chars().take(12).collect::<String>();
    let standing = match checks {
        Ok(Checks::Green) => "green".to_string(),
        Ok(Checks::Red) => "red".to_string(),
        Ok(Checks::Pending) => "pending".to_string(),
        Err(e) => format!("unreadable ({e:#})"),
    };
    let mut text = format!(
        "pull request #{number} is not merged: its current head {} is not the job's landing commit {}. \
         The checks of that head are {standing}.",
        short(head),
        short(landed)
    );
    if extras.is_empty() {
        text.push_str(&format!(
            "\nRefused head: {} (it adds no commit after the landing commit, so it is older than it or \
             unrelated to it).",
            short(head)
        ));
    } else {
        text.push_str("\nExtra commits after the landing commit:");
        for (sha, subject) in extras {
            text.push_str(&format!("\n  {} {subject}", short(sha)));
        }
    }
    text.push_str(
        "\nWhy it is refused: until the records-history build (D111) lands, a squash of a job's landing \
         plus any later commit is not a valid landing (it breaks the history check). This holds for a \
         declared intervention too. Athena did not merge it and did not replace the head with the job's \
         commit.",
    );
    text
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

/// What the wait for checks found.
struct Waited {
    checks: Checks,
    reruns: u32,
    failing_test: Option<String>,
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
fn wait_for_checks(job: &LandPr, head: &str) -> Result<Waited> {
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
        // Reconcile first (§8.1): a landing record missing from an earlier crash is written, or the commit is
        // reported unattributed. It never fails the landing.
        for line in self.reconcile(runner, ui).lines() {
            ui.line(&line);
        }
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
        // The records sink is built here, on the production path, and its error stops the landing: a
        // landing never goes ahead without somewhere to write its records (§5.2).
        let sink = GitLandingRecords::for_remote(&self.git, remote_name)?;
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
                    records: Some(&sink),
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
        // The merge is done and stays done: a failed Pages watch is shown, never returned as an error.
        if report.link.state == LinkState::Landed
            && let Some(merged) = &report.link.merged_commit
        {
            let pages_poll = Poll {
                attempts: PAGES_WAIT_LOOKS,
                delay: PAGES_WAIT_DELAY,
                sleep: &std::thread::sleep,
            };
            if let Err(problem) = watch_pages(&client, merged, &pages_poll, ui) {
                ui.line(&format!(
                    "warning: could not watch the Pages deploy: {problem:#}"
                ));
            }
        }
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::history::LandingKind;

    #[test]
    fn parse_merged_pr_keeps_only_the_pull_request_that_merged_the_commit() {
        let commit = "a".repeat(40);
        let pr = |number: u64, merged: bool, sha: &str| {
            serde_json::json!({
                "number": number,
                "merged_at": if merged { Value::from("2026-10-24T00:00:00Z") } else { Value::Null },
                "merge_commit_sha": sha,
                "head": {"sha": "b".repeat(40)},
            })
        };
        // None: an empty reply, an unmerged pull request, or one merged as another commit.
        assert_eq!(
            parse_merged_pr(&serde_json::json!([]), &commit).unwrap(),
            None
        );
        let reply = serde_json::json!([pr(1, false, &commit), pr(2, true, &"c".repeat(40))]);
        assert_eq!(parse_merged_pr(&reply, &commit).unwrap(), None);
        // One.
        let reply = serde_json::json!([pr(1, false, &commit), pr(3, true, &commit)]);
        assert_eq!(
            parse_merged_pr(&reply, &commit).unwrap(),
            Some(MergedPr {
                number: 3,
                head_sha: "b".repeat(40)
            })
        );
        // Two: a guess would be needed, so it is an error.
        let reply = serde_json::json!([pr(3, true, &commit), pr(4, true, &commit)]);
        let error = parse_merged_pr(&reply, &commit).unwrap_err();
        assert!(format!("{error:#}").contains("ambiguous"), "{error:#}");
    }
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
    fn the_ci_run_is_read_with_its_sha_address_conclusions_and_duration() {
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let runs = json!({"workflow_runs": [
            {"name": "pages", "head_sha": sha, "status": "completed", "conclusion": "failure",
             "html_url": "https://example.test/pages", "created_at": "2026-01-01T00:00:00Z",
             "updated_at": "2026-01-01T00:09:00Z", "id": 1},
            {"name": "ci", "head_sha": sha, "status": "completed", "conclusion": "success",
             "html_url": "https://example.test/ci/2", "created_at": "2026-01-01T00:00:00Z",
             "updated_at": "2026-01-01T00:02:30Z", "id": 2}]});
        let jobs = json!({"jobs": [{"steps": [
            {"name": "Set up", "conclusion": "success"},
            {"name": "cargo fmt --check, cargo clippy -D warnings, cargo test",
             "conclusion": "success"}]}]});
        let run = parse_ci_run(&runs, Some(&jobs)).unwrap();
        assert_eq!(run.sha, sha);
        assert_eq!(run.address, "https://example.test/ci/2");
        assert!(run.finished);
        assert_eq!(run.duration_s, 150.0);
        assert_eq!(run.steps.len(), 3);
        assert!(run.steps.iter().all(|s| s.passed()));
        // Without the jobs reply, or for a run of another workflow only, there are no step results.
        assert!(parse_ci_run(&runs, None).unwrap().steps.is_empty());
        let only_pages = json!({"workflow_runs": [runs["workflow_runs"][0].clone()]});
        assert!(parse_ci_run(&only_pages, None).is_none());
        // A run for another commit keeps its own SHA, so the SHA check can reject it.
        let other = json!({"workflow_runs": [{"name": "ci", "head_sha": "fedcba",
            "status": "completed", "conclusion": "success", "html_url": "u"}]});
        assert_eq!(parse_ci_run(&other, None).unwrap().sha, "fedcba");
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
        /// What the remote branch holds; a push moves it.
        head: RefCell<Option<String>>,
        pushes: RefCell<Vec<(String, String)>>,
        /// The commits `commits_after` reports.
        extras: RefCell<Vec<(String, String)>>,
    }
    impl Remote for FakeRemote {
        fn remote_head(&self, _: &str) -> Result<Option<String>> {
            Ok(self.head.borrow().clone())
        }
        fn push(&self, branch: &str, commit: &str) -> Result<()> {
            self.pushes
                .borrow_mut()
                .push((branch.to_string(), commit.to_string()));
            *self.head.borrow_mut() = Some(commit.to_string());
            Ok(())
        }
        fn commits_after(&self, _: &str, _: &str) -> Result<Vec<(String, String)>> {
            Ok(self.extras.borrow().clone())
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
        /// Reads taken from the front before `checks` is returned for every later read.
        script: RefCell<Vec<Checks>>,
        /// How many of the first run looks (before any re-run) show the run still in progress.
        in_progress_looks: Cell<u32>,
        /// How many run looks after the re-run still show the old try.
        stale_looks: Cell<u32>,
        /// Whether the host reports `run_attempt`.
        reports_attempt: Cell<bool>,
        run_looks: Cell<u32>,
        looks_after_rerun: Cell<u32>,
        reruns: Cell<u32>,
        /// How many run looks had happened at each re-run request.
        run_looks_at_rerun: RefCell<Vec<u32>>,
        /// The job log the failing test is read from.
        log: RefCell<String>,
        /// Checks that depend on the commit asked about; other commits use `script` and `checks`.
        per_commit: RefCell<BTreeMap<String, Checks>>,
        /// The head each merge request carried as its guard.
        merge_heads: RefCell<Vec<String>>,
        /// A merge request that the host refuses.
        merge_refused: Cell<bool>,
        /// Whether a merge makes the host report the merged commit (false: merged but not confirmed).
        merge_confirms: Cell<bool>,
        /// Run when a merge request arrives, before the host answers.
        on_merge: RefCell<Option<Box<dyn Fn()>>>,
    }
    impl FakeHost {
        fn new(checks: Checks) -> Self {
            FakeHost {
                merge_refused: Cell::new(false),
                merge_confirms: Cell::new(true),
                on_merge: RefCell::new(None),
                per_commit: RefCell::new(BTreeMap::new()),
                merge_heads: RefCell::new(vec![]),
                script: RefCell::new(vec![]),
                in_progress_looks: Cell::new(0),
                stale_looks: Cell::new(0),
                reports_attempt: Cell::new(true),
                run_looks: Cell::new(0),
                looks_after_rerun: Cell::new(0),
                reruns: Cell::new(0),
                run_looks_at_rerun: RefCell::new(vec![]),
                log: RefCell::new(String::new()),
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
            if let Some(known) = self.per_commit.borrow().get(commit) {
                return Ok(*known);
            }
            let mut script = self.script.borrow_mut();
            Ok(if script.is_empty() {
                self.checks
            } else {
                script.remove(0)
            })
        }
        fn ci_runs(&self, _: &str) -> Result<Vec<CiRunState>> {
            self.run_looks.set(self.run_looks.get() + 1);
            let attempt = if self.reruns.get() == 0 {
                1
            } else {
                self.looks_after_rerun.set(self.looks_after_rerun.get() + 1);
                if self.looks_after_rerun.get() <= self.stale_looks.get() {
                    1
                } else {
                    2
                }
            };
            let going =
                self.reruns.get() == 0 && self.run_looks.get() <= self.in_progress_looks.get();
            Ok(vec![CiRunState {
                id: 9,
                status: if going { "in_progress" } else { "completed" }.into(),
                conclusion: (!going).then(|| "failure".to_string()),
                attempt: self.reports_attempt.get().then_some(attempt),
            }])
        }
        fn rerun_failed(&self, run_id: u64) -> Result<()> {
            assert_eq!(run_id, 9);
            self.run_looks_at_rerun
                .borrow_mut()
                .push(self.run_looks.get());
            self.reruns.set(self.reruns.get() + 1);
            Ok(())
        }
        fn failing_test(&self, _: u64) -> Result<Option<String>> {
            Ok(failing_test_name(&self.log.borrow()))
        }
        fn merge(&self, _: u64, head: &str, _: &CommitText) -> Result<MergeMethod> {
            self.merges.set(self.merges.get() + 1);
            self.merge_heads.borrow_mut().push(head.to_string());
            if let Some(hook) = self.on_merge.borrow().as_ref() {
                hook();
            }
            if self.merge_refused.get() {
                bail!("the host refused the merge");
            }
            if self.merge_confirms.get() {
                *self.merged.borrow_mut() = Some(self.merge_makes.to_string());
            }
            Ok(MergeMethod::Merge)
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
        /// The poll's look budget.
        attempts: u32,
    }

    impl Rig {
        fn new() -> Self {
            Rig {
                attempts: 3,
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
                    attempts: self.attempts,
                    delay: Duration::from_secs(1),
                    sleep: &nap,
                },
                ancestry,
                remote: &self.remote,
                host,
                content: &self.content,
                records: None,
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
    fn no_rerun_is_asked_until_the_failed_run_completes_and_then_exactly_one() {
        let mut rig = Rig::new();
        rig.attempts = 4;
        let host = FakeHost::new(Checks::Red);
        // The first read is red while the run is still going; it is in progress for two looks.
        host.in_progress_looks.set(2);
        *host.script.borrow_mut() = vec![Checks::Red, Checks::Red, Checks::Red];
        let report = rig.run(&host, true, "athena/bootstrap").unwrap();
        assert_eq!(host.reruns.get(), 1);
        // The request came at the third look, the first that showed the run completed.
        assert_eq!(*host.run_looks_at_rerun.borrow(), vec![3]);
        assert!(host.check_reads.get() <= 2 * rig.attempts);
        assert_eq!(report.reruns, 1);
        assert_eq!(report.checks, Some(Checks::Red));
        assert_eq!(host.merges.get(), 0);
    }

    #[test]
    fn a_stale_run_attempt_is_not_taken_as_the_result_of_the_rerun() {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Red);
        host.stale_looks.set(1);
        *host.script.borrow_mut() = vec![Checks::Red, Checks::Red, Checks::Pending, Checks::Red];
        let report = rig.run(&host, true, "athena/bootstrap").unwrap();
        // One read in the first wait and three in the second: the first of them is stale and ignored.
        assert_eq!(host.check_reads.get(), 4);
        assert_eq!(host.reruns.get(), 1);
        assert_eq!(report.reruns, 1);
        assert_eq!(report.checks, Some(Checks::Red));
        assert_eq!(host.merges.get(), 0);
    }

    #[test]
    fn a_host_that_reports_no_run_attempt_is_trusted_from_the_second_look() {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Red);
        host.reports_attempt.set(false);
        *host.script.borrow_mut() = vec![Checks::Red, Checks::Red, Checks::Green];
        let report = rig.run(&host, false, "athena/bootstrap").unwrap();
        assert_eq!(host.check_reads.get(), 3);
        assert_eq!(report.checks, Some(Checks::Green));
    }

    #[test]
    fn a_rerun_that_turns_green_is_reported_green_and_merged_when_the_setting_is_on() {
        for merge_via_rest in [true, false] {
            let rig = Rig::new();
            let host = FakeHost::new(Checks::Red);
            *host.script.borrow_mut() = vec![Checks::Red, Checks::Pending, Checks::Green];
            let report = rig.run(&host, merge_via_rest, "athena/bootstrap").unwrap();
            assert_eq!(report.checks, Some(Checks::Green), "{merge_via_rest}");
            assert_eq!(report.reruns, 1);
            assert_eq!(report.failing_test, None);
            assert_eq!(host.reruns.get(), 1);
            assert_eq!(host.merges.get(), u32::from(merge_via_rest));
            assert_eq!(report.merge_requested, merge_via_rest);
        }
    }

    #[test]
    fn a_second_red_merges_nothing_reruns_nothing_more_and_names_the_failing_test() {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Red);
        *host.log.borrow_mut() =
            "2026-01-01T00:00:00.1Z test gh::tests::x ... FAILED\n".to_string();
        let report = rig.run(&host, true, "athena/bootstrap").unwrap();
        assert_eq!(host.merges.get(), 0);
        assert!(!report.merge_requested);
        assert_eq!(host.reruns.get(), 1);
        assert_eq!(report.reruns, 1);
        assert_eq!(report.checks, Some(Checks::Red));
        assert_eq!(report.failing_test.as_deref(), Some("gh::tests::x"));
        assert_eq!(report.link.state, LinkState::Open);
    }

    #[test]
    fn checks_still_pending_after_the_bound_give_pending_with_no_rerun_and_no_merge() {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Pending);
        let report = rig.run(&host, true, "athena/bootstrap").unwrap();
        assert_eq!(report.checks, Some(Checks::Pending));
        assert_eq!(host.check_reads.get(), rig.attempts);
        assert_eq!(host.reruns.get(), 0);
        assert_eq!(host.run_looks.get(), 0);
        assert_eq!(report.reruns, 0);
        assert_eq!(host.merges.get(), 0);
    }

    #[test]
    fn a_rerun_that_never_restarts_ends_pending_and_is_not_repeated() {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Red);
        host.stale_looks.set(100);
        let report = rig.run(&host, true, "athena/bootstrap").unwrap();
        assert_eq!(report.checks, Some(Checks::Pending));
        assert_eq!(host.reruns.get(), 1);
        assert_eq!(host.merges.get(), 0);
        assert!(host.check_reads.get() <= 2 * rig.attempts);
    }

    #[test]
    fn the_failing_test_is_read_from_a_log() {
        let log = "2026-10-18T10:00:00.1234567Z running 2 tests\n\
                   2026-10-18T10:00:00.2Z test gh::tests::ok ... ok\n\
                   2026-10-18T10:00:00.3Z test odd name here ... FAILED\n\
                   2026-10-18T10:00:00.4Z test gh::tests::bad_one ... FAILED\n\
                   test result: FAILED. 1 passed; 1 failed\n";
        assert_eq!(
            failing_test_name(log).as_deref(),
            Some("gh::tests::bad_one")
        );
        let stdout_only = "---- core::thing stdout ----\nthread panicked\n";
        assert_eq!(
            failing_test_name(stdout_only).as_deref(),
            Some("core::thing")
        );
        assert_eq!(failing_test_name("test a b ... FAILED\n"), None);
        assert_eq!(failing_test_name("test result: FAILED. 0 passed\n"), None);
        let long = format!("test {} ... FAILED\n", "a".repeat(TEST_NAME_LIMIT + 1));
        assert_eq!(failing_test_name(&long), None);
    }

    #[test]
    fn the_runs_reply_is_read_into_states() {
        let reply = json!({"workflow_runs": [
            {"id": 3, "status": "completed", "conclusion": "failure", "run_attempt": 2},
            {"id": 4, "status": "in_progress", "conclusion": null},
            {"status": "queued"}
        ]});
        let runs = parse_ci_runs(&reply);
        assert_eq!(runs.len(), 2);
        assert!(runs[0].failed());
        assert_eq!(runs[0].attempt, Some(2));
        assert!(!runs[1].failed());
        assert_eq!(runs[1].attempt, None);
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
            records: None,
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
        assert_eq!(report.link.head, "landed1");
    }

    #[test]
    fn a_rerun_at_the_same_commit_pushes_nothing() {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        rig.run(&host, false, "athena/bootstrap").unwrap();
        rig.run(&host, false, "athena/bootstrap").unwrap();
        assert_eq!(rig.remote.pushes.borrow().len(), 1);
    }

    /// A pull request opened at `cand1`; then a fix `fix1` is pushed to its branch (the job's commit stays
    /// `cand1`, red, and `fix1` is green). Returns the rig and host.
    fn pushed_fix() -> (Rig, FakeHost) {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        host.per_commit
            .borrow_mut()
            .insert("cand1".into(), Checks::Red);
        host.per_commit
            .borrow_mut()
            .insert("fix1".into(), Checks::Green);
        rig.run(&host, false, "athena/bootstrap").unwrap();
        *rig.remote.head.borrow_mut() = Some("fix1".into());
        *rig.remote.extras.borrow_mut() = vec![("fix1".repeat(10), "Fix the failing test".into())];
        host.checked.borrow_mut().clear();
        (rig, host)
    }

    #[test]
    fn land_pr_judges_the_current_head_not_the_old_commit() {
        let (rig, host) = pushed_fix();
        let reruns_before = host.reruns.get();
        let err = rig
            .run(&host, true, "athena/bootstrap")
            .unwrap_err()
            .to_string();
        // Only the head's checks were read, and they are green; the old red is not reported or re-run.
        assert_eq!(*host.checked.borrow(), vec!["fix1".to_string()]);
        assert!(err.contains("checks of that head are green"), "{err}");
        assert!(!err.contains("are red"), "{err}");
        assert_eq!(host.reruns.get(), reruns_before);
        assert_eq!(host.merges.get(), 0);
        assert_eq!(rig.stored().head, "fix1");
    }

    #[test]
    fn land_pr_merges_when_the_head_is_the_landing_commit() {
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        let report = rig.run(&host, true, "athena/bootstrap").unwrap();
        assert!(report.merge_requested);
        assert_eq!(host.merges.get(), 1);
        assert_eq!(report.link.state, LinkState::Landed);
    }

    #[test]
    fn land_pr_guards_the_merge_with_the_current_head_and_pushes_nothing_over_a_newer_one() {
        // Head equals the landing commit: the guard is that head.
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        rig.run(&host, true, "athena/bootstrap").unwrap();
        assert_eq!(*host.merge_heads.borrow(), vec!["cand1".to_string()]);

        // A newer head: no merge, and no push of any kind after the first publish.
        let (rig, host) = pushed_fix();
        assert!(rig.run(&host, true, "athena/bootstrap").is_err());
        assert!(host.merge_heads.borrow().is_empty());
        assert_eq!(
            *rig.remote.pushes.borrow(),
            vec![("athena/bootstrap".to_string(), "cand1".to_string())]
        );
        assert_eq!(rig.remote.head.borrow().as_deref(), Some("fix1"));
        assert_eq!(rig.stored().head, "fix1");
    }

    #[test]
    fn land_pr_refuses_any_other_head_naming_each_extra_commit_and_why() {
        let (rig, host) = pushed_fix();
        *rig.remote.extras.borrow_mut() = vec![
            ("a".repeat(40), "Fix the failing test".into()),
            ("b".repeat(40), "Declared intervention: hand edit".into()),
        ];
        let err = rig
            .run(&host, true, "athena/bootstrap")
            .unwrap_err()
            .to_string();
        assert!(err.contains(&format!("{} Fix the failing test", "a".repeat(12))));
        assert!(err.contains(&format!("{} Declared intervention", "b".repeat(12))));
        assert!(err.contains("records-history build (D111)"), "{err}");
        assert!(err.contains("declared intervention too"), "{err}");
        assert!(err.contains("squash"), "{err}");
        assert_eq!(host.merges.get(), 0);

        // A head the remote cannot list commits for is still refused, and named.
        *rig.remote.extras.borrow_mut() = vec![];
        let err = rig
            .run(&host, true, "athena/bootstrap")
            .unwrap_err()
            .to_string();
        assert!(err.contains("Refused head: fix1"), "{err}");
        assert_eq!(host.merges.get(), 0);
    }

    #[test]
    fn a_missing_remote_branch_is_an_error_and_nothing_is_merged() {
        let (rig, host) = pushed_fix();
        *rig.remote.head.borrow_mut() = None;
        let err = rig
            .run(&host, true, "athena/bootstrap")
            .unwrap_err()
            .to_string();
        assert!(err.contains("not on the remote"), "{err}");
        assert_eq!(host.merges.get(), 0);
    }

    #[test]
    fn git_remote_lists_the_commits_after_the_landing_commit() {
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
        let landed = step("landed");
        let x = step("extra one");
        let y = step("extra two");
        git.run(["remote", "add", "origin", bare_dir.path().to_str().unwrap()])
            .unwrap();
        git.run(["push", "-q", "origin", &format!("{y}:refs/heads/work")])
            .unwrap();
        let remote = GitRemote {
            git: &git,
            name: "origin",
        };
        assert_eq!(
            remote.commits_after(&landed, &y).unwrap(),
            vec![
                (x.clone(), "extra one".to_string()),
                (y.clone(), "extra two".to_string())
            ]
        );
        assert!(remote.commits_after(&y, &landed).unwrap().is_empty());
        assert!(remote.commits_after(&y, &y).unwrap().is_empty());
        // Nothing here can replace the remote branch.
        assert_eq!(remote.remote_head("work").unwrap(), Some(y));
    }

    /// End to end over real git: the branch holds the landing commit plus a declared-intervention commit.
    /// Land refuses naming that commit, requests no merge, and leaves the remote branch untouched.
    #[test]
    fn land_pr_refuses_a_real_branch_with_a_declared_intervention_after_the_landing() {
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
        let landed = step("landed");
        let branch = "athena/bootstrap";
        git.run(["remote", "add", "origin", bare_dir.path().to_str().unwrap()])
            .unwrap();
        let remote = GitRemote {
            git: &git,
            name: "origin",
        };
        let rig = Rig::new();
        let host = FakeHost::new(Checks::Green);
        host.per_commit
            .borrow_mut()
            .insert(landed.clone(), Checks::Red);
        let run = |commit: &str| {
            let state = StateDir::new(rig.dir.path().to_path_buf());
            let nap = |_: Duration| {};
            land_pr(&LandPr {
                state: &state,
                job: &rig.job,
                branch,
                commit,
                verified_fingerprint: "verified",
                pr: PrDraft {
                    title: "t",
                    body: "b",
                    head: branch,
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
                ancestry: &git,
                remote: &remote,
                host: &host,
                content: &rig.content,
                records: None,
            })
        };
        // Publish the landing commit; it is red, so it is not merged.
        let first = run(&landed).unwrap();
        assert!(!first.merge_requested);
        let fix = step("fix the failure");
        let intervention = step("Declared intervention: edit by hand");
        git.run([
            "push",
            "-q",
            "origin",
            &format!("{intervention}:refs/heads/{branch}"),
        ])
        .unwrap();
        let err = run(&landed).unwrap_err().to_string();
        assert!(
            err.contains(&format!("{} fix the failure", &fix[..12])),
            "{err}"
        );
        assert!(
            err.contains(&format!(
                "{} Declared intervention: edit by hand",
                &intervention[..12]
            )),
            "{err}"
        );
        assert!(err.contains("records-history build (D111)"), "{err}");
        assert!(err.contains("are green"), "{err}");
        assert_eq!(host.merges.get(), 0);
        let held = bare.rev_parse(&format!("refs/heads/{branch}")).unwrap();
        assert_eq!(held.as_deref(), Some(intervention.as_str()));
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
        let rig = Rig::new();
        *rig.remote.head.borrow_mut() = Some("rewritten".into());
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
            records: None,
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

    /// `land_pr` with the real records sink, a real repository, a bare remote and a fake host
    /// (`docs/design/records-history.md` §5.2, §5.3, §6.1, §8).
    mod records_flow {
        use super::*;
        use athena_git::records_branch::RECORDS_REF;
        use std::rc::Rc;

        const JOB: &str = "20261024-115923-02gf";
        /// The commit the fake host makes: a valid sha, as the landing record requires.
        const HOST_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

        struct World {
            _bare_dir: tempfile::TempDir,
            bare: Git,
            _work_dir: tempfile::TempDir,
            git: Git,
            state_dir: tempfile::TempDir,
            /// The remote main's tip when the pull request was opened.
            base: String,
            head: String,
            remote: FakeRemote,
            content: FakeContent,
            job: JobId,
        }

        impl World {
            fn new() -> Self {
                let bare_dir = tempfile::tempdir().unwrap();
                let bare = Git::isolated(bare_dir.path());
                // The fake host's default branch is `trunk`.
                bare.run(["init", "-q", "--bare", "-b", "trunk"]).unwrap();
                let work_dir = tempfile::tempdir().unwrap();
                let git = Git::isolated(work_dir.path());
                git.run(["init", "-q", "-b", "trunk"]).unwrap();
                let url = bare_dir.path().to_str().unwrap().to_string();
                git.run(["remote", "add", "origin", &url]).unwrap();
                std::fs::write(work_dir.path().join("a.txt"), "a").unwrap();
                git.run(["add", "-A"]).unwrap();
                git.run(["commit", "-q", "-m", "init"]).unwrap();
                git.run(["push", "-q", "origin", "trunk"]).unwrap();
                let base = git.head().unwrap().unwrap();
                git.run(["checkout", "-q", "-b", &format!("athena/{JOB}")])
                    .unwrap();
                let mut world = World {
                    _bare_dir: bare_dir,
                    bare,
                    _work_dir: work_dir,
                    git,
                    state_dir: tempfile::tempdir().unwrap(),
                    base,
                    head: String::new(),
                    remote: FakeRemote::default(),
                    content: FakeContent(BTreeMap::from([(HOST_COMMIT, "verified")])),
                    job: JobId::parse(JOB).unwrap(),
                };
                // The pull request's one commit lands the job: its work, with its candidate, verification
                // and landed outcome.
                let mut files = crate::land::fixtures::landed(&world.job);
                files.push(crate::land::fixtures::plain("b.txt", "b.txt"));
                world.add_files(&files, "Work");
                world
            }

            /// A commit on the work branch, pushed, so the pull request's head moves to it.
            fn add_commit(&mut self, file: &str, message: &str) -> String {
                self.add_files(&[crate::land::fixtures::plain(file, file)], message)
            }

            /// A commit of `files` on the work branch, pushed, so the pull request's head moves to it.
            fn add_files(
                &mut self,
                files: &[crate::land::fixtures::File],
                message: &str,
            ) -> String {
                self.head = crate::land::fixtures::commit(&self.git, files, message);
                self.git
                    .run(["push", "-q", "-f", "origin", &format!("athena/{JOB}")])
                    .unwrap();
                *self.remote.head.borrow_mut() = Some(self.head.clone());
                self.head.clone()
            }

            fn host(&self) -> FakeHost {
                let mut host = FakeHost::new(Checks::Green);
                host.merge_makes = HOST_COMMIT;
                host
            }

            fn sink(&self) -> GitLandingRecords<'_> {
                GitLandingRecords::for_remote(&self.git, "origin").unwrap()
            }

            fn run(
                &self,
                host: &FakeHost,
                records: Option<&dyn LandingRecords>,
            ) -> Result<PrReport> {
                let state = StateDir::new(self.state_dir.path().to_path_buf());
                let nap = |_: Duration| {};
                land_pr(&LandPr {
                    state: &state,
                    job: &self.job,
                    branch: &format!("athena/{JOB}"),
                    commit: &self.head,
                    verified_fingerprint: "verified",
                    pr: PrDraft {
                        title: "t",
                        body: "b",
                        head: "athena/x",
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
                    ancestry: &Anc(true),
                    remote: &self.remote,
                    host,
                    content: &self.content,
                    records,
                })
            }

            fn remote_tip(&self) -> Option<String> {
                self.bare.rev_parse(RECORDS_REF).unwrap()
            }

            /// The records on the remote branch, oldest commit first, with the commit that added each.
            fn records(&self) -> Vec<(String, HistoryRecord)> {
                let Some(tip) = self.remote_tip() else {
                    return Vec::new();
                };
                let mut out = Vec::new();
                for commit in self
                    .bare
                    .run(["rev-list", "--reverse", "--first-parent", &tip])
                    .unwrap()
                    .lines()
                {
                    for path in self.bare.added_paths(commit).unwrap() {
                        // Other writers' files (the rival's, in one test) are not landing records.
                        if !path.starts_with("landing") {
                            continue;
                        }
                        let bytes = self.bare.show_file(commit, &path).unwrap().unwrap();
                        let record = HistoryRecord::parse(&String::from_utf8(bytes).unwrap());
                        out.push((commit.to_string(), record.unwrap()));
                    }
                }
                out
            }
        }

        fn intents(records: &[(String, HistoryRecord)]) -> Vec<(String, LandingIntent)> {
            records
                .iter()
                .filter_map(|(c, r)| match r {
                    HistoryRecord::LandingIntent(i) => Some((c.clone(), i.clone())),
                    _ => None,
                })
                .collect()
        }

        fn landings(records: &[(String, HistoryRecord)]) -> Vec<(String, Landing)> {
            records
                .iter()
                .filter_map(|(c, r)| match r {
                    HistoryRecord::Landing(l) => Some((c.clone(), l.clone())),
                    _ => None,
                })
                .collect()
        }

        #[test]
        fn land_pr_writes_the_intent_then_the_landing_with_the_head_as_second_parent() {
            let world = World::new();
            let host = world.host();
            let sink = world.sink();
            let report = world.run(&host, Some(&sink)).unwrap();
            assert_eq!(report.link.state, LinkState::Landed);
            assert!(report.warnings.is_empty(), "{:?}", report.warnings);

            let records = world.records();
            assert_eq!(records.len(), 2, "one intent, then one landing");
            let (intent_commit, intent) = &intents(&records)[0];
            let (landing_commit, landing) = &landings(&records)[0];
            assert!(matches!(records[0].1, HistoryRecord::LandingIntent(_)));
            assert!(matches!(records[1].1, HistoryRecord::Landing(_)));

            let job = JobId::parse(JOB).unwrap();
            assert_eq!(intent.pr_number, 42);
            assert_eq!(intent.head_sha, world.head);
            assert_eq!(
                intent.head_tree_sha,
                world.git.tree_of(&world.head).unwrap()
            );
            assert_eq!(intent.expected_base, world.base);
            assert_eq!(intent.job_ids, vec![job.clone()]);
            intent.validate().unwrap();

            assert_eq!(landing.pr_number, 42);
            assert_eq!(landing.head_sha, world.head);
            assert_eq!(landing.host_commit, HOST_COMMIT);
            assert_eq!(landing.merge_method, history::MergeMethod::Merge);
            assert_eq!(landing.intent, intent.id);
            assert_eq!(
                landing.commits,
                vec![LandingCommit {
                    sha: world.head.clone(),
                    tree: world.git.tree_of(&world.head).unwrap(),
                    kind: LandingKind::JobLanding,
                    job: Some(job),
                    verification: Some(RecordId::parse(JOB).unwrap()),
                }]
            );
            landing.validate().unwrap();

            // The intent commit is the branch's root; the landing commit's parents are the intent commit and
            // the pull request's head, which keeps the head reachable (§6.1).
            assert!(world.bare.parents(intent_commit).unwrap().is_empty());
            assert_eq!(
                world.bare.parents(landing_commit).unwrap(),
                vec![intent_commit.clone(), world.head.clone()]
            );
            // The local branch holds what the remote holds.
            assert_eq!(
                world.git.rev_parse(RECORDS_REF).unwrap(),
                world.remote_tip()
            );
            assert_eq!(world.remote_tip().as_deref(), Some(landing_commit.as_str()));
        }

        #[test]
        fn the_intent_is_on_the_remote_before_the_merge_and_the_landing_after() {
            let world = World::new();
            let host = world.host();
            let seen: Rc<RefCell<Vec<(usize, usize)>>> = Rc::new(RefCell::new(Vec::new()));
            let bare = world.bare.clone();
            let at_merge = Rc::clone(&seen);
            *host.on_merge.borrow_mut() = Some(Box::new(move || {
                let tip = bare.rev_parse(RECORDS_REF).unwrap().unwrap();
                let count = |dir: &str| bare.ls_tree_under(&tip, dir).unwrap().len();
                at_merge
                    .borrow_mut()
                    .push((count("landing-intents"), count("landings")));
            }));
            let sink = world.sink();
            world.run(&host, Some(&sink)).unwrap();
            assert_eq!(
                *seen.borrow(),
                vec![(1, 0)],
                "at merge time: intent, no landing"
            );
            // After land_pr returns, the bare remote itself holds the intent and the landing.
            let tip = world.bare.rev_parse(RECORDS_REF).unwrap().unwrap();
            assert_eq!(
                world
                    .bare
                    .ls_tree_under(&tip, "landing-intents")
                    .unwrap()
                    .len(),
                1
            );
            assert_eq!(world.bare.ls_tree_under(&tip, "landings").unwrap().len(), 1);
            let records = world.records();
            assert_eq!((intents(&records).len(), landings(&records).len()), (1, 1));
        }

        #[test]
        fn a_refused_merge_leaves_the_intent_on_both_branches_and_no_landing() {
            let world = World::new();
            let host = world.host();
            host.merge_refused.set(true);
            let sink = world.sink();
            let err = world.run(&host, Some(&sink)).unwrap_err();
            assert!(
                err.to_string().contains("the host refused the merge"),
                "{err:#}"
            );
            let records = world.records();
            assert_eq!((intents(&records).len(), landings(&records).len()), (1, 0));
            assert!(world.remote_tip().is_some());
            assert_eq!(
                world.git.rev_parse(RECORDS_REF).unwrap(),
                world.remote_tip()
            );
        }

        #[test]
        fn a_merge_that_is_not_confirmed_writes_no_landing_warns_and_a_rerun_adds_nothing() {
            let world = World::new();
            let host = world.host();
            host.merge_confirms.set(false);
            let sink = world.sink();
            let report = world.run(&host, Some(&sink)).unwrap();
            let records = world.records();
            assert_eq!((intents(&records).len(), landings(&records).len()), (1, 0));
            let intent_id = intents(&records)[0].1.id.to_string();
            assert_eq!(report.warnings.len(), 1, "{:?}", report.warnings);
            assert!(
                report.warnings[0].contains(&intent_id),
                "{:?}",
                report.warnings
            );
            // The result is the one the CLI already maps: an open pull request with a merge requested.
            assert_eq!(report.link.state, LinkState::Open);
            assert!(report.merge_requested);

            let tip = world.remote_tip();
            let again = world.run(&host, Some(&sink)).unwrap();
            assert_eq!(world.remote_tip(), tip, "a rerun makes no commit");
            let records = world.records();
            assert_eq!((intents(&records).len(), landings(&records).len()), (1, 0));
            assert!(again.warnings[0].contains(&intent_id));
        }

        #[test]
        fn a_leftover_local_records_branch_is_reset_to_the_remote_before_writing() {
            let world = World::new();
            let host = world.host();
            host.merge_refused.set(true);
            let sink = world.sink();
            world.run(&host, Some(&sink)).unwrap_err();
            let intent_tip = world.remote_tip().unwrap();

            // Ahead: a record that only the local branch has. Diverged: the remote also moves, through
            // another clone.
            let writer = RecordsWriter::new(&world.git);
            let leftover = writer
                .append("leftover", &[Change::put("leftover.json", "{}")])
                .unwrap();
            let other_dir = tempfile::tempdir().unwrap();
            let other = Git::isolated(other_dir.path());
            other.run(["init", "-q"]).unwrap();
            let url = world.bare.dir().to_str().unwrap().to_string();
            other.run(["remote", "add", "origin", &url]).unwrap();
            let rival = RecordsWriter::new(&other)
                .append_to_remote("origin", "rival", &[Change::put("rival.json", "r")], &[])
                .unwrap();
            assert_eq!(world.git.parents(&leftover).unwrap(), vec![intent_tip]);

            host.merge_refused.set(false);
            let report = world.run(&host, Some(&sink)).unwrap();
            assert_eq!(report.link.state, LinkState::Landed);
            let tip = world.remote_tip().unwrap();
            assert_eq!(
                world.git.rev_parse(RECORDS_REF).unwrap().as_deref(),
                Some(tip.as_str())
            );
            assert_eq!(
                world.bare.rev_parse(&leftover).unwrap(),
                None,
                "the leftover never left"
            );
            assert!(world.bare.is_ancestor(&rival, &tip).unwrap());
            let records = world.records();
            assert_eq!(
                (intents(&records).len(), landings(&records).len()),
                (1, 1),
                "the intent was reused"
            );
        }

        #[test]
        fn for_remote_builds_a_sink_that_writes_to_the_remote_records_branch() {
            let world = World::new();
            let sink = GitLandingRecords::for_remote(&world.git, "origin").unwrap();
            assert_eq!(sink.base_tip("trunk").unwrap(), world.base);
            assert!(sink.base_tip("nope").is_err());
            assert_eq!(world.remote_tip(), None);
            let intent = sink.intent(7, &world.head, &world.base).unwrap();
            let facts = LandingFacts {
                intent: intent.clone(),
                expected_base: world.base.clone(),
                pr_number: 7,
                head: world.head.clone(),
                host_commit: HOST_COMMIT.to_string(),
                merge_method: MergeMethod::Squash,
            };
            let landing = sink.landing(&facts).unwrap();
            assert_eq!(
                sink.landing(&facts).unwrap(),
                landing,
                "a repeat finds the same landing"
            );
            let records = world.records();
            assert_eq!(intents(&records)[0].1.id, intent);
            assert_eq!(landings(&records)[0].1.id, landing);
            assert_eq!(
                landings(&records)[0].1.merge_method,
                history::MergeMethod::Squash
            );
            let err = GitLandingRecords::for_remote(&world.git, "elsewhere").unwrap_err();
            assert!(err.to_string().contains("elsewhere"), "{err:#}");
        }

        #[test]
        fn commits_carry_their_kind_job_and_verification_and_pass_validation() {
            let mut world = World::new();
            let note = crate::land::fixtures::plain(
                "records/introspections/2026-10/20261024-120000-abcd.json",
                "{}\n",
            );
            let records_only = world.add_files(&[note], "Records");
            let intervention = RecordId::parse("20261024-121000-abcd").unwrap();
            let path = format!(
                "records/interventions/{}/{intervention}.json",
                intervention.month()
            );
            let declared = world.add_files(
                &[
                    crate::land::fixtures::plain("c.txt", "by hand\n"),
                    crate::land::fixtures::plain(&path, "{}\n"),
                ],
                "Hand work",
            );
            let commits = world.sink().commits(&world.base, &world.head).unwrap();
            for commit in &commits {
                commit.validate().unwrap();
            }
            assert_eq!(
                commits
                    .iter()
                    .map(|c| (c.kind, c.job.is_some(), c.verification.is_some()))
                    .collect::<Vec<_>>(),
                [
                    (LandingKind::JobLanding, true, true),
                    (LandingKind::RecordsOnly, false, false),
                    (LandingKind::DeclaredIntervention, false, false),
                ]
            );
            assert_eq!(commits[0].job.as_ref(), Some(&world.job));
            assert_eq!(commits[0].verification, Some(RecordId::parse(JOB).unwrap()));
            assert_eq!(commits[1].sha, records_only);
            assert_eq!(commits[2].sha, declared);
            // The intent names only the jobs that landed.
            let sink = world.sink();
            sink.intent(42, &world.head, &world.base).unwrap();
            let records = world.records();
            assert_eq!(intents(&records)[0].1.job_ids, vec![world.job.clone()]);
        }

        #[test]
        fn a_pull_request_that_lands_no_job_has_no_intent() {
            let mut world = World::new();
            // The only commit above the base is an intervention; nothing lands a job.
            world
                .git
                .run(["reset", "-q", "--hard", &world.base])
                .unwrap();
            let intervention = RecordId::parse("20261024-121000-abcd").unwrap();
            let path = format!(
                "records/interventions/{}/{intervention}.json",
                intervention.month()
            );
            world.add_files(
                &[
                    crate::land::fixtures::plain("c.txt", "by hand\n"),
                    crate::land::fixtures::plain(&path, "{}\n"),
                ],
                "Hand work",
            );
            let err = world
                .sink()
                .intent(42, &world.head, &world.base)
                .unwrap_err()
                .to_string();
            assert!(err.contains("lands a job"), "{err}");
            assert_eq!(world.remote_tip(), None);
        }

        #[test]
        fn a_governed_commit_no_record_covers_is_named_and_nothing_is_recorded_or_merged() {
            let mut world = World::new();
            let bad = world.add_commit("c.txt", "Hand edit with no records");
            let host = world.host();
            let sink = world.sink();
            let err = world.run(&host, Some(&sink)).unwrap_err().to_string();
            assert!(err.contains(&bad), "{err}");
            assert!(
                err.contains("lies between the remote base and the PR head"),
                "{err}"
            );
            assert_eq!(world.remote_tip(), None);
            assert_eq!(host.merges.get(), 0);
        }

        #[test]
        fn a_bootstrap_commit_has_no_verification_and_is_named() {
            let err = verification_id(&JobId::Bootstrap, "abc123").unwrap_err();
            assert!(err.to_string().contains("abc123"), "{err}");
            let id = RecordId::parse(JOB).unwrap();
            assert_eq!(verification_id(&JobId::Id(id.clone()), "abc").unwrap(), id);
        }

        /// The repository's own configuration with `[history] cutover_job` set to the test job, or removed.
        fn cutover_config(names_job: bool) -> String {
            athena_gate::BUILTIN_CONFIG
                .lines()
                .filter(|l| names_job || !l.starts_with("cutover_job = "))
                .map(|l| {
                    if l.starts_with("cutover_job = ") {
                        format!("cutover_job = \"{JOB}\"")
                    } else {
                        l.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
                + "\n"
        }

        /// A world whose pull request also adds `athena.toml` (naming the job as the cutover job when
        /// `names_job`), and whose host merge makes a real squash commit on the remote's default branch.
        /// Returns the world and that commit. The checkout the landing starts from has no `athena.toml`.
        fn cutover_world(names_job: bool) -> (World, &'static str) {
            let mut world = World::new();
            world
                .git
                .run(["reset", "-q", "--hard", &world.base])
                .unwrap();
            let mut files = crate::land::fixtures::landed(&world.job);
            files.push(crate::land::fixtures::plain("b.txt", "b.txt"));
            files.push(crate::land::fixtures::plain(
                "athena.toml",
                &cutover_config(names_job),
            ));
            world.add_files(&files, "Work");
            let tree = world.git.tree_of(&world.head).unwrap();
            let squash = world
                .git
                .commit_tree(&tree, &[&world.base], "Squash", false)
                .unwrap();
            world
                .git
                .run([
                    "push",
                    "-q",
                    "origin",
                    &format!("{squash}:refs/heads/trunk"),
                ])
                .unwrap();
            let squash: &'static str = Box::leak(squash.into_boxed_str());
            world.content = FakeContent(BTreeMap::from([(squash, "verified")]));
            (world, squash)
        }

        fn legacy_record_at(world: &World, tip: &str) -> history::LegacyExempt {
            let bytes = world
                .bare
                .show_file(tip, athena_gate::legacy::LEGACY_EXEMPT_PATH)
                .unwrap()
                .expect("the legacy-exempt record is on the records branch");
            match HistoryRecord::parse(&String::from_utf8(bytes).unwrap()).unwrap() {
                HistoryRecord::LegacyExempt(record) => record,
                other => panic!("{other:?}"),
            }
        }

        #[test]
        fn cutover_job_is_read_from_the_commit_given_not_the_checkout() {
            let (world, squash) = cutover_world(true);
            assert_eq!(cutover_job_for(&world.git, squash).as_deref(), Some(JOB));
            // The base commit of the landing has no `athena.toml`, and the built-in configuration (which
            // names this repository's own cutover job) is not consulted for it.
            assert_eq!(cutover_job_for(&world.git, &world.base), None);
            let (without, squash) = cutover_world(false);
            assert_eq!(cutover_job_for(&without.git, squash), None);
        }

        #[test]
        fn cutover_land_imports_at_c0_and_writes_the_legacy_exempt_record() {
            let (world, squash) = cutover_world(true);
            let mut host = world.host();
            host.merge_makes = squash;
            let sink = world.sink();
            let report = world.run(&host, Some(&sink)).unwrap();
            assert_eq!(report.link.state, LinkState::Landed);
            assert_eq!(report.warnings.len(), 1, "{:?}", report.warnings);
            assert!(
                report.warnings[0].contains("cutover: imported records/ at C0"),
                "{:?}",
                report.warnings
            );

            // The landing record, then one commit: the import with its manifest and the legacy-exempt record.
            let records = world.records();
            let (landing_commit, landing) = landings(&records)[0].clone();
            assert_eq!(landing.host_commit, squash);
            let tip = world.remote_tip().unwrap();
            assert_eq!(
                world.bare.parents(&tip).unwrap(),
                vec![landing_commit],
                "the import is one commit on top of the landing"
            );

            // C0 is resolved from the landing record, and the import's self-check passes against it.
            let c0 = athena_gate::attribution::cutover_commit_at(&world.git, &tip, JOB)
                .unwrap()
                .expect("a landing of the cutover job");
            assert_eq!(c0, squash);
            crate::records_import::verify_cutover_import(&world.git, &c0, &tip).unwrap();
            let manifest = world
                .bare
                .show_file(&tip, crate::records_import::MANIFEST_PATH)
                .unwrap()
                .expect("the manifest is on the records branch");
            match HistoryRecord::parse(&String::from_utf8(manifest).unwrap()).unwrap() {
                HistoryRecord::ImportManifest(m) => {
                    assert_eq!(m.c0, squash);
                    assert!(!m.paths.is_empty());
                    for path in &m.paths {
                        assert!(world.bare.show_file(&tip, &path.path).unwrap().is_some());
                    }
                }
                other => panic!("{other:?}"),
            }
            assert!(
                athena_gate::attribution::manifest_names(&world.git, &tip, &c0).unwrap(),
                "the history check sees the cutover as shown"
            );

            // Seven entries, six listed in athena.toml plus 02837e0, each with a reason.
            let exempt = legacy_record_at(&world, &tip);
            assert_eq!(exempt.entries.len(), 7);
            assert!(
                exempt
                    .entries
                    .iter()
                    .any(|e| e.sha.starts_with("02837e0") && !e.reason.is_empty())
            );
            assert!(exempt.entries.iter().all(|e| !e.reason.trim().is_empty()));

            // A landing of another job is not the cutover.
            assert_eq!(
                athena_gate::attribution::cutover_commit_at(
                    &world.git,
                    &tip,
                    "20261024-100000-zzzz"
                )
                .unwrap(),
                None
            );
        }

        #[test]
        fn a_land_whose_config_names_no_cutover_job_writes_no_import() {
            let (world, squash) = cutover_world(false);
            let mut host = world.host();
            host.merge_makes = squash;
            let sink = world.sink();
            let report = world.run(&host, Some(&sink)).unwrap();
            assert_eq!(report.link.state, LinkState::Landed);
            assert!(report.warnings.is_empty(), "{:?}", report.warnings);
            let tip = world.remote_tip().unwrap();
            assert!(
                world
                    .bare
                    .show_file(&tip, crate::records_import::MANIFEST_PATH)
                    .unwrap()
                    .is_none()
            );
            assert_eq!(world.records().len(), 2, "the intent and the landing only");
        }

        #[test]
        fn run_cutover_writes_import_and_exempt_record_once() {
            let (world, squash) = cutover_world(true);
            let mut host = world.host();
            host.merge_makes = squash;
            // Land without the cutover: the sink the land used does nothing on the hook.
            struct NoCutover<'a>(GitLandingRecords<'a>);
            impl LandingRecords for NoCutover<'_> {
                fn base_tip(&self, branch: &str) -> Result<String> {
                    self.0.base_tip(branch)
                }
                fn intent(&self, pr: u64, head: &str, base: &str) -> Result<RecordId> {
                    self.0.intent(pr, head, base)
                }
                fn landing(&self, facts: &LandingFacts) -> Result<RecordId> {
                    self.0.landing(facts)
                }
            }
            let plain = NoCutover(world.sink());
            world.run(&host, Some(&plain)).unwrap();
            let landed_tip = world.remote_tip().unwrap();
            assert!(
                world
                    .bare
                    .show_file(&landed_tip, crate::records_import::MANIFEST_PATH)
                    .unwrap()
                    .is_none()
            );

            let sink = world.sink();
            let first = sink.run_cutover(JOB).unwrap().unwrap();
            assert!(first.import.is_some() && first.exempt_written, "{first:?}");
            let after = world.remote_tip().unwrap();
            assert_ne!(after, landed_tip);
            let second = sink.run_cutover(JOB).unwrap().unwrap();
            assert!(!second.wrote(), "{second:?}");
            assert_eq!(world.remote_tip().unwrap(), after, "nothing was written");
            assert_eq!(
                sink.run_cutover("20261024-100000-zzzz").unwrap(),
                None,
                "a job with no landing has no cutover"
            );
        }
    }
}
