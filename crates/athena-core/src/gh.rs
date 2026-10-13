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
use athena_data::JobId;
use athena_data::records::Verdict;
use athena_git::Git;
use serde::Serialize;
use serde_json::Value;
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
            bail!(
                "gh {} failed: {}",
                args.get(3).map(String::as_str).unwrap_or("api"),
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        String::from_utf8(out.stdout).context("gh printed non-text output")
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
    /// Asks the host to merge the pull request, guarded by the head commit it was opened with.
    fn merge(&self, number: u64, head: &str) -> Result<()>;
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
}

impl std::fmt::Debug for GhClient<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GhClient")
            .field("repository", &self.repository)
            .finish_non_exhaustive()
    }
}

impl GhClient<'_> {
    fn api(&self, method: &str, path: &str, fields: &[(&str, &str)]) -> Result<Value> {
        let mut args: Vec<String> = ["api", "--method", method, path]
            .iter()
            .map(|s| s.to_string())
            .collect();
        for (key, value) in fields {
            args.push("-f".into());
            args.push(format!("{key}={value}"));
        }
        let text = self.runner.gh(&args)?;
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
        let repo = self.api("GET", &format!("repos/{}", self.repository), &[])?;
        match repo["default_branch"].as_str() {
            Some(name) if !name.is_empty() => Ok(name.to_string()),
            _ => bail!("the host's reply for the repository names no default branch"),
        }
    }

    fn open_pr(&self, pr: &NewPr) -> Result<u64> {
        let created = self.api(
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
            "GET",
            &self.repo_path(&format!("commits/{commit}/check-runs?per_page=100")),
            &[],
        )?;
        let status = self.api(
            "GET",
            &self.repo_path(&format!("commits/{commit}/status")),
            &[],
        )?;
        Ok(classify_checks(&runs, &status))
    }

    fn merge(&self, number: u64, head: &str) -> Result<()> {
        self.api(
            "PUT",
            &self.repo_path(&format!("pulls/{number}/merge")),
            &[("sha", head), ("merge_method", "merge")],
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
            job.host.merge(link.pr_number, &link.head)?;
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
        let client = GhClient {
            runner,
            repository: repository.clone(),
        };
        let remote = GitRemote {
            git: &self.git,
            name: remote_name,
        };
        let report = match landed_commit(&self.git, &head, &id, &verification.verified_fingerprint)
        {
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
                host_name: &host_name,
                repository: &repository,
                merge_via_rest: config.landing.merge_via_rest,
                poll,
                ancestry: &self.git,
                remote: &remote,
                host: &client,
                content: &content,
            }),
            // The landed commit is not on this branch. If the job's pull request already exists, the host
            // may have merged it, and that confirms the landing without the local commit.
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
        }?;
        ui.line(&format!(
            "pull request #{} on {}/{}: {:?}",
            report.link.pr_number, report.link.host, report.link.repository, report.link.state
        ));
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::cell::{Cell, RefCell};
    use std::collections::BTreeMap;

    struct FakeRunner {
        calls: RefCell<Vec<Vec<String>>>,
    }
    impl Runner for FakeRunner {
        fn gh(&self, args: &[String]) -> Result<String> {
            self.calls.borrow_mut().push(args.to_vec());
            let path = args.iter().find(|a| a.starts_with("repos/")).unwrap();
            Ok(if path == "repos/example/project" {
                r#"{"default_branch": "trunk"}"#.to_string()
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
        let runner = FakeRunner {
            calls: RefCell::new(vec![]),
        };
        let client = GhClient {
            runner: &runner,
            repository: "example/project".into(),
        };
        let pr = NewPr {
            title: "t",
            body: "b",
            head: "athena/j",
            base: "main",
        };
        assert_eq!(client.open_pr(&pr).unwrap(), 7);
        assert_eq!(client.checks("abc").unwrap(), Checks::Green);
        client.merge(7, "abc").unwrap();
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
        assert_eq!(calls.len(), 5);
        for args in calls.iter() {
            assert_eq!(args[0], "api", "{args:?}");
            assert_eq!(args[1], "--method");
            assert!(args[3].starts_with("repos/example/project/"), "{args:?}");
            assert!(!args.iter().any(|a| a == "pr" || a == "--auto"), "{args:?}");
        }
        assert_eq!(calls[0][3], "repos/example/project/pulls");
        assert_eq!(calls[0][2], "POST");
        assert_eq!(calls[3][2], "PUT");
        assert_eq!(calls[3][3], "repos/example/project/pulls/7/merge");
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
        fn merge(&self, _: u64, _: &str) -> Result<()> {
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
            host: &FakeHost,
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
        let runner = FakeRunner {
            calls: RefCell::new(vec![]),
        };
        let client = GhClient {
            runner: &runner,
            repository: "example/project".into(),
        };
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
}
