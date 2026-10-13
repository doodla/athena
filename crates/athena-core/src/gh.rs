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
    Ancestry, Content, GitContent, Host, LinkState, PrLink, PushRequest, check_push, load_link,
    observe_merge, store_link,
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

/// What the host can be asked, besides where a merge landed ([`Host`]).
pub trait PrHost: Host {
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
    /// The verified candidate commit, pushed as the pull request's head.
    pub candidate: &'a str,
    pub verified_fingerprint: &'a str,
    pub pr: NewPr<'a>,
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
            .field("candidate", &self.candidate)
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
    if link.state == LinkState::Open {
        let seen = wait_for_checks(job)?;
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

fn publish(job: &LandPr) -> Result<PrLink> {
    let allowed = work_branch(job.job);
    let remote_head = job.remote.remote_head(job.branch)?;
    check_push(
        job.ancestry,
        &PushRequest {
            allowed_branch: &allowed,
            branch: job.branch,
            local_head: job.candidate,
            remote_head: remote_head.as_deref(),
        },
    )?;
    job.remote.push(job.branch, job.candidate)?;
    let pr_number = job.host.open_pr(&job.pr)?;
    let link = PrLink {
        pr_number,
        host: job.host_name.to_string(),
        repository: job.repository.to_string(),
        head: job.candidate.to_string(),
        job: job.job.as_str().to_string(),
        state: LinkState::Open,
        merged_commit: None,
    };
    store_link(job.state, job.job, &link)?;
    Ok(link)
}

fn wait_for_checks(job: &LandPr) -> Result<Checks> {
    let attempts = job.poll.attempts.max(1);
    let mut seen = Checks::Pending;
    for n in 0..attempts {
        seen = job.host.checks(job.candidate)?;
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
        let base = self
            .git
            .current_branch()?
            .context("a pull request needs a branch to merge into; HEAD is detached")?;
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
        };
        let client = GhClient {
            runner,
            repository: repository.clone(),
        };
        let remote = GitRemote {
            git: &self.git,
            name: remote_name,
        };
        let report = land_pr(&LandPr {
            state: &self.state,
            job: &id,
            branch: &branch,
            candidate: &verification.candidate_commit,
            verified_fingerprint: &verification.verified_fingerprint,
            pr: NewPr {
                title: &title,
                body: &body,
                head: &branch,
                base: &base,
            },
            host_name: &host_name,
            repository: &repository,
            merge_via_rest: config.landing.merge_via_rest,
            poll,
            ancestry: &self.git,
            remote: &remote,
            host: &client,
            content: &content,
        })?;
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
            Ok(if path.ends_with("/pulls") {
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
    }
    impl Remote for FakeRemote {
        fn remote_head(&self, _: &str) -> Result<Option<String>> {
            Ok(self.head.clone())
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
        fn open_pr(&self, _: &NewPr) -> Result<u64> {
            self.opened.set(self.opened.get() + 1);
            Ok(42)
        }
        fn checks(&self, _: &str) -> Result<Checks> {
            self.check_reads.set(self.check_reads.get() + 1);
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
            let state = StateDir::new(self.dir.path().to_path_buf());
            let nap = |_: Duration| {};
            land_pr(&LandPr {
                state: &state,
                job: &self.job,
                branch,
                candidate: "cand1",
                verified_fingerprint: "verified",
                pr: NewPr {
                    title: "t",
                    body: "b",
                    head: "athena/bootstrap",
                    base: "main",
                },
                host_name: HOST,
                repository: "example/project",
                merge_via_rest,
                poll: Poll {
                    attempts: 3,
                    delay: Duration::from_secs(1),
                    sleep: &nap,
                },
                ancestry: &Anc(true),
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
            candidate: "cand1",
            verified_fingerprint: "verified",
            pr: NewPr {
                title: "t",
                body: "b",
                head: "athena/bootstrap",
                base: "main",
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
