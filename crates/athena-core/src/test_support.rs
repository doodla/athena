//! A shared test fixture for the records-history cutover (decision D111): a scratch repository with a bare
//! remote and a realistic history, made only through Athena's own writers.
//!
//! This module is compiled for this crate's own tests and for any crate that turns on the `test-support`
//! feature of `athena-core` (the shipped binary does not). Nothing here is used in production.
//!
//! # The shape of the history
//!
//! [`CutoverFixture::build`] makes a work repository, a bare remote named `origin`, and this history, in order:
//!
//! 1. **Bootstrap.** One commit on `main` with the trailer `Athena-Job: bootstrap` and an `athena.toml` that
//!    has *no* `[history] cutover_job`. It is pushed to the bare remote.
//! 2. **The old-way job.** A job read (`ask`), dispatched and verified (`go`) and landed on local `main` by
//!    `go`'s own landing, so its commit carries the Athena trailers and its `records/` files. There is no
//!    pull request and no landing record: this is how every job landed before the cutover. `main` is then
//!    pushed to the remote.
//! 3. **The cutover job.** A second job, read and verified the same way, whose change sets
//!    `[history] cutover_job` in `athena.toml` to its own id. It is landed through the public
//!    [`crate::gh::land_pr`] with the real [`crate::gh::GitLandingRecords`] sink, [`crate::gh::GitRemote`],
//!    [`crate::pr::GitContent`] and git ancestry. Only the host is faked ([`FakeHost`]): its `merge` makes a
//!    real squash commit on the bare remote's `main`, as the host does. Landing then writes, on
//!    `athena/records` of the bare remote and of the work repository: the landing-intent record, the landing
//!    record, and, from the landing's cutover hook, one commit holding the import of `records/` at `C0`
//!    with its manifest and the legacy-exempt record. The work repository's `main` is then moved onto the
//!    squash commit with [`crate::Athena::resync_onto_merge`], as `athena land --pr` does.
//!
//! `C0` is the squash commit: the host commit of the cutover job's landing record.
//!
//! # Reusing it
//!
//! Core cannot supply the one thing `ask` and `go` need that it does not own: a capability (the change
//! capability lives in `athena-cap-change`, wired in `athena-cli`). So the caller passes
//! [`CutoverSpec`]: the files of the bootstrap tree, a function that makes the capabilities, and a function
//! from [`Step`] to the fake-agent script for that step.
//!
//! - **From `athena-cli` tests** (the trailer job, job 7 part 2, and any later history job): call
//!   `common::cutover_fixture()` from `tests/common/mod.rs`. It already supplies
//!   `wiring::capabilities()` and the `Scenario` scripts. Run the real binary against the result with
//!   `common::athena_check(&fixture, "--history")` or `"--attribution"`. See `tests/cutover_fixture.rs` for
//!   the assertions that pin the shape.
//! - **From `athena-core` tests:** build a [`CutoverSpec`] with your own capability and scripts and call
//!   [`CutoverFixture::build`]. The `athena-cli` wrapper is the model to copy.
//! - **To add history after the cutover** (a job landed after `C0`): use [`CutoverFixture::athena`] to get an
//!   [`crate::Athena`] over the work repository and the shared state directory, run `ask` and `go`, and land
//!   it with [`land_through_pr`] (then [`Athena::resync_onto_merge`] to move `main` onto the squash).
//! - **If a job changes how commits are made** (the trailer job), change the writers, not this fixture: the
//!   history is rebuilt through them on each run. Update the assertions that read the trailers
//!   ([`CutoverFixture::trailers_of`] returns what the commit carries).
//!
//! Do not hand-build commits or records in a fixture for history tests: an earlier attempt did, and its
//! history did not match what the writers make.

use crate::capability::Capability;
use crate::gh::{
    Checks, CiRunState, GitLandingRecords, GitRemote, LandPr, MergeMethod, NewPr, Poll, PrDraft,
    PrHost, PrReport, land_pr,
};
use crate::pr::{GitContent, Host, LinkState, PrLink};
use crate::state::StateDir;
use crate::{Athena, Quiet};
use anyhow::{Context, Result, bail};
use athena_agent::fake::{FakeAgentDriver, Script};
use athena_data::JobId;
use athena_data::records::{AcceptedBy, Build, OutcomeState};
use athena_git::{CommitText, Git};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Which step of the history a script is wanted for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step<'a> {
    /// The old-way job, read, dispatched, verified and landed with one script.
    OldWay,
    /// The cutover job's reading. Its id is not known yet, so the script cannot name it.
    CutoverRead,
    /// The cutover job's dispatch. The script's producer must make `athena_toml` the content of
    /// `athena.toml` (it sets `[history] cutover_job` to the job's id), and the reading it was given must list
    /// `athena.toml` as a deliverable to modify and name it in an acceptance criterion (admission refuses a
    /// change to a governance file otherwise).
    CutoverGo { athena_toml: &'a str },
}

/// What [`CutoverFixture::build`] needs from its caller.
pub struct CutoverSpec<'a> {
    /// The files of the bootstrap commit, as (path, content). It must include `athena.toml`. Any
    /// `cutover_job` line in it is removed, so the bootstrap names no cutover job.
    pub files: Vec<(String, String)>,
    /// Makes the capabilities for one [`Athena`]; called once for each of the three runs.
    pub capabilities: &'a dyn Fn() -> Vec<Box<dyn Capability>>,
    /// The fake-agent script for a step.
    pub scripts: &'a dyn Fn(Step<'_>) -> Script,
}

impl std::fmt::Debug for CutoverSpec<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CutoverSpec")
            .field(
                "files",
                &self.files.iter().map(|(p, _)| p).collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

/// The built history and the handles to look at it. The temporary directories live as long as this value.
pub struct CutoverFixture {
    work_dir: tempfile::TempDir,
    bare_dir: tempfile::TempDir,
    state_dir: tempfile::TempDir,
    /// The work repository.
    pub git: Git,
    /// The bare remote, named `origin` in the work repository.
    pub bare: Git,
    /// The bootstrap commit.
    pub bootstrap: String,
    /// The old-way job and the commit its landing made (on `main`, with trailers).
    pub old_way_job: JobId,
    pub old_way_landing: String,
    /// The cutover job.
    pub cutover_job: JobId,
    /// The commit `go` landed the cutover job as on the work repository's `main`. It is what the pull
    /// request pushed as its head; `main` no longer holds it once the squash is resynced.
    pub cutover_local_landing: String,
    /// `C0`: the squash commit the host made, and the tip of `main` in both repositories.
    pub c0: String,
    /// What `land_pr` reported for the cutover job.
    pub report: PrReport,
}

impl std::fmt::Debug for CutoverFixture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CutoverFixture")
            .field("work", &self.work_dir.path())
            .field("bare", &self.bare_dir.path())
            .field("old_way_job", &self.old_way_job)
            .field("cutover_job", &self.cutover_job)
            .field("c0", &self.c0)
            .finish_non_exhaustive()
    }
}

/// `config` with its `[history] cutover_job` removed, or, for `Some(job)`, set to `job`.
pub fn with_cutover_job(config: &str, job: Option<&str>) -> String {
    let mut out = String::new();
    for line in config.lines() {
        if line.starts_with("cutover_job = ") {
            continue;
        }
        out.push_str(line);
        out.push('\n');
        if let (Some(job), "[history]") = (job, line.trim_end()) {
            out.push_str(&format!("cutover_job = \"{job}\"\n"));
        }
    }
    if let Some(job) = job {
        assert!(
            out.contains(&format!("cutover_job = \"{job}\"")),
            "the configuration has a [history] table"
        );
    }
    out
}

fn write_files(root: &Path, files: &[(String, String)]) -> Result<()> {
    for (path, content) in files {
        let full = root.join(path);
        std::fs::create_dir_all(full.parent().expect("a file path has a parent"))?;
        std::fs::write(&full, content).with_context(|| format!("writing {}", full.display()))?;
    }
    Ok(())
}

/// A host that merges by making a real squash commit on the bare remote's `main`. Everything else about a
/// pull request is as a host reports it for a green, mergeable request.
#[derive(Debug)]
pub struct FakeHost<'a> {
    /// The work repository, whose object store makes the squash and pushes it.
    pub git: &'a Git,
    merged: RefCell<Option<String>>,
}

impl<'a> FakeHost<'a> {
    pub fn new(git: &'a Git) -> Self {
        FakeHost {
            git,
            merged: RefCell::new(None),
        }
    }
}

impl Host for FakeHost<'_> {
    fn merged_commit(&self, _: &PrLink) -> Result<Option<String>> {
        Ok(self.merged.borrow().clone())
    }
}

impl PrHost for FakeHost<'_> {
    fn default_branch(&self) -> Result<String> {
        Ok("main".to_string())
    }

    fn open_pr(&self, _: &NewPr) -> Result<u64> {
        Ok(1)
    }

    fn checks(&self, _: &str) -> Result<Checks> {
        Ok(Checks::Green)
    }

    /// The squash: the head's tree on top of the remote `main`, with the title and body given, pushed to
    /// `main` of the remote.
    fn merge(&self, _: u64, head: &str, text: &CommitText) -> Result<MergeMethod> {
        let base = self
            .git
            .fetch_remote_branch("origin", "main")?
            .context("the remote has no main to merge into")?;
        let tree = self.git.tree_of(head)?;
        let message = format!("{}\n\n{}", text.title, text.body.trim_end());
        let squash = self.git.commit_tree(&tree, &[&base], &message, false)?;
        self.git
            .run(["push", "-q", "origin", &format!("{squash}:refs/heads/main")])?;
        *self.merged.borrow_mut() = Some(squash);
        Ok(MergeMethod::Squash)
    }

    fn ci_runs(&self, _: &str) -> Result<Vec<CiRunState>> {
        Ok(Vec::new())
    }

    fn check_run(&self, _: &str, _: &str) -> Result<Option<CiRunState>> {
        Ok(None)
    }

    fn rerun_failed(&self, _: u64) -> Result<()> {
        Ok(())
    }

    fn failing_test(&self, _: u64) -> Result<Option<String>> {
        Ok(None)
    }
}

/// One call the early step made on [`EarlyFake`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EarlyCall {
    /// `push`, `replace`, `open_pr` or `merge`.
    pub kind: &'static str,
    /// The branch pushed or replaced, or the pull request's head branch.
    pub branch: String,
    /// The commit pushed or replaced to; empty for the other calls.
    pub commit: String,
    /// What the probe reported when the call arrived (for `go` tests: how many agents had run).
    pub seen: usize,
}

/// A remote and a host in one recording fake, for tests that drive `go` through the early-pull-request seam.
/// It records every call with a probe reading taken at call time, and can be told to refuse a push or an
/// open.
pub struct EarlyFake {
    /// What the remote branch holds; a push or replace moves it.
    pub head: RefCell<Option<String>>,
    pub calls: RefCell<Vec<EarlyCall>>,
    pub fail_push: std::cell::Cell<bool>,
    pub fail_open: std::cell::Cell<bool>,
    probe: Box<dyn Fn() -> usize>,
}

impl std::fmt::Debug for EarlyFake {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EarlyFake")
            .field("calls", &self.calls)
            .finish_non_exhaustive()
    }
}

impl EarlyFake {
    /// A fake whose calls record `probe()` as they arrive.
    pub fn new(probe: impl Fn() -> usize + 'static) -> std::rc::Rc<Self> {
        std::rc::Rc::new(EarlyFake {
            head: RefCell::new(None),
            calls: RefCell::new(Vec::new()),
            fail_push: std::cell::Cell::new(false),
            fail_open: std::cell::Cell::new(false),
            probe: Box::new(probe),
        })
    }

    fn record(&self, kind: &'static str, branch: &str, commit: &str) {
        self.calls.borrow_mut().push(EarlyCall {
            kind,
            branch: branch.to_string(),
            commit: commit.to_string(),
            seen: (self.probe)(),
        });
    }

    /// The calls of one kind, in order.
    pub fn of(&self, kind: &str) -> Vec<EarlyCall> {
        self.calls
            .borrow()
            .iter()
            .filter(|c| c.kind == kind)
            .cloned()
            .collect()
    }
}

impl crate::gh::Remote for EarlyFake {
    fn remote_head(&self, _: &str) -> Result<Option<String>> {
        Ok(self.head.borrow().clone())
    }

    fn push(&self, branch: &str, commit: &str) -> Result<()> {
        self.record("push", branch, commit);
        if self.fail_push.get() {
            bail!("the remote refused the push");
        }
        *self.head.borrow_mut() = Some(commit.to_string());
        Ok(())
    }

    fn replace(&self, branch: &str, commit: &str, expected_old: &str) -> Result<()> {
        self.record("replace", branch, commit);
        if self.head.borrow().as_deref() != Some(expected_old) {
            bail!("the remote branch moved");
        }
        *self.head.borrow_mut() = Some(commit.to_string());
        Ok(())
    }

    fn commits_after(&self, _: &str, _: &str) -> Result<Vec<(String, String)>> {
        Ok(Vec::new())
    }
}

impl Host for EarlyFake {
    fn merged_commit(&self, _: &PrLink) -> Result<Option<String>> {
        Ok(None)
    }
}

impl PrHost for EarlyFake {
    fn default_branch(&self) -> Result<String> {
        Ok("main".to_string())
    }

    fn open_pr(&self, pr: &NewPr) -> Result<u64> {
        self.record("open_pr", pr.head, "");
        if self.fail_open.get() {
            bail!("the host refused the pull request");
        }
        Ok(7)
    }

    fn checks(&self, _: &str) -> Result<Checks> {
        Ok(Checks::Green)
    }

    fn merge(&self, _: u64, head: &str, _: &CommitText) -> Result<MergeMethod> {
        self.record("merge", "", head);
        Ok(MergeMethod::Merge)
    }

    fn ci_runs(&self, _: &str) -> Result<Vec<CiRunState>> {
        Ok(Vec::new())
    }

    fn check_run(&self, _: &str, _: &str) -> Result<Option<CiRunState>> {
        Ok(None)
    }

    fn rerun_failed(&self, _: u64) -> Result<()> {
        Ok(())
    }

    fn failing_test(&self, _: u64) -> Result<Option<String>> {
        Ok(None)
    }
}

impl CutoverFixture {
    /// Builds the history described in the module text. Fails, and builds nothing partial that matters, if a
    /// writer refuses a step: the error names the step.
    pub fn build(spec: CutoverSpec<'_>) -> Result<CutoverFixture> {
        let work_dir = tempfile::tempdir()?;
        let bare_dir = tempfile::tempdir()?;
        let state_dir = tempfile::tempdir()?;
        let bare = Git::at(bare_dir.path());
        bare.run(["init", "-q", "--bare", "-b", "main"])?;
        let git = Git::at(work_dir.path());
        git.run(["init", "-q", "-b", "main"])?;
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value])?;
        }
        let url = bare_dir.path().to_str().context("a UTF-8 path")?;
        git.run(["remote", "add", "origin", url])?;

        // 1. Bootstrap.
        let mut files = spec.files.clone();
        let Some(config) = files.iter_mut().find(|(path, _)| path == "athena.toml") else {
            bail!("the bootstrap files must include athena.toml");
        };
        config.1 = with_cutover_job(&config.1, None);
        write_files(work_dir.path(), &files)?;
        git.run(["add", "-A"])?;
        git.run([
            "commit",
            "-q",
            "-m",
            "Start a demo repository\n\nAthena-Job: bootstrap",
        ])?;
        let bootstrap = git.head()?.context("the bootstrap commit")?;
        git.run(["push", "-q", "origin", "main"])?;

        let state_root = state_dir.path().join("repo");
        let athena_with =
            |script: Script| make_athena(&git, &state_root, (spec.capabilities)(), script);

        // 2. The old-way job: read, dispatched, verified, landed locally; then pushed.
        let old_way = athena_with((spec.scripts)(Step::OldWay));
        let proposal = old_way
            .ask("add a greeting to the readme", &Quiet)
            .context("reading the old-way job")?;
        let old_way_job = proposal.job.clone();
        let old_way_landing = landed(
            old_way.go(old_way_job.as_str(), AcceptedBy::User, &Quiet),
            "the old-way job",
        )?;
        git.run(["push", "-q", "origin", "main"])?;

        // 3. The cutover job: its change names it as the cutover job.
        let read = athena_with((spec.scripts)(Step::CutoverRead));
        let proposal = read
            .ask("name this job as the records-history cutover", &Quiet)
            .context("reading the cutover job")?;
        let cutover_job = proposal.job.clone();
        let current = git
            .show_file("HEAD", "athena.toml")?
            .context("athena.toml at HEAD")?;
        let athena_toml = with_cutover_job(
            &String::from_utf8_lossy(&current),
            Some(cutover_job.as_str()),
        );
        let base = git.head()?.context("main before the cutover job")?;
        let go = athena_with((spec.scripts)(Step::CutoverGo {
            athena_toml: &athena_toml,
        }));
        let cutover_local_landing = landed(
            go.go(cutover_job.as_str(), AcceptedBy::User, &Quiet),
            "the cutover job",
        )?;

        // The pull request: the real landing flow with the real records sink; only the host is faked.
        let records = crate::store::Records::load(&git, &cutover_local_landing)?;
        let fingerprint = records
            .jobs
            .get(cutover_job.as_str())
            .and_then(|job| job.verification.as_ref())
            .map(|v| v.verified_fingerprint.clone())
            .context("the cutover job's verification")?;
        let report = land_through_pr(
            &git,
            &state_root,
            &cutover_job,
            &cutover_local_landing,
            &base,
            &fingerprint,
        )
        .context("landing the cutover job through the pull request path")?;
        if report.link.state != LinkState::Landed {
            bail!("the cutover job's pull request did not land: {:?}", report);
        }
        let c0 = report
            .link
            .merged_commit
            .clone()
            .context("the merged commit")?;
        go.resync_onto_merge(&cutover_local_landing, &c0, &Quiet)
            .context("moving main onto the squash commit")?;

        Ok(CutoverFixture {
            work_dir,
            bare_dir,
            state_dir,
            git,
            bare,
            bootstrap,
            old_way_job,
            old_way_landing,
            cutover_job,
            cutover_local_landing,
            c0,
            report,
        })
    }

    /// The work repository's directory.
    pub fn root(&self) -> &Path {
        self.work_dir.path()
    }

    /// The bare remote's directory.
    pub fn bare_root(&self) -> &Path {
        self.bare_dir.path()
    }

    /// The state directory root every [`Athena`] of this fixture shares (the value `ATHENA_STATE_DIR` takes
    /// for a binary that should see the same jobs).
    pub fn state_root(&self) -> PathBuf {
        self.state_dir.path().join("repo")
    }

    /// An [`Athena`] over the work repository and the shared state directory, with `capabilities` and
    /// `script`, for adding history after the cutover.
    pub fn athena(&self, capabilities: Vec<Box<dyn Capability>>, script: Script) -> Athena {
        make_athena(&self.git, &self.state_root(), capabilities, script)
    }

    /// The tip of `athena/records` on the bare remote.
    pub fn records_tip(&self) -> Result<String> {
        self.bare
            .rev_parse(athena_git::records_branch::RECORDS_REF)?
            .context("the bare remote has no athena/records")
    }

    /// The trailers of `commit` as (key, value), in order, read with git's own parser.
    pub fn trailers_of(&self, commit: &str) -> Result<Vec<(String, String)>> {
        Ok(self.git.trailers(&self.git.message(commit)?)?)
    }
}

/// Lands `job`, whose verified landing commit `landed` is on the work repository's `main` on top of `base`
/// (the remote's `main`), through [`land_pr`] with the real records sink and a [`FakeHost`]. `fingerprint` is
/// the job's verified content fingerprint. The remote's `main` and `athena/records` are updated; the work
/// repository's `main` is not moved (see [`Athena::resync_onto_merge`]).
pub fn land_through_pr(
    git: &Git,
    state_root: &Path,
    job: &JobId,
    landed: &str,
    base: &str,
    fingerprint: &str,
) -> Result<PrReport> {
    let state = StateDir::new(state_root.to_path_buf());
    let branch = crate::gh::work_branch(job);
    // Strict before C0; after it (the base names a cutover job) the commit carries no trailers.
    let squash = crate::gh::squash_text_for(git, base, landed)?;
    let remote = GitRemote {
        git,
        name: "origin",
    };
    let content = GitContent {
        git,
        rules_commit: base,
        remote: "origin",
    };
    let sink = GitLandingRecords::for_remote(git, "origin")?;
    let host = FakeHost::new(git);
    let nap = |_: Duration| {};
    land_pr(&LandPr {
        state: &state,
        job,
        branch: &branch,
        commit: landed,
        verified_fingerprint: fingerprint,
        pr: PrDraft {
            title: &squash.title,
            body: &squash.body,
            head: &branch,
        },
        squash: &squash,
        host_name: "github.com",
        repository: "example/project",
        merge_via_rest: true,
        poll: Poll {
            attempts: 1,
            delay: Duration::ZERO,
            sleep: &nap,
        },
        ancestry: git,
        remote: &remote,
        host: &host,
        content: &content,
        records: Some(&sink),
    })
}

/// A free-space reader that reports a very large disk, so a test does not depend on the host's free space.
/// Install it with [`StateDir::with_free_space`].
pub fn plenty_of_space(_: &Path) -> std::io::Result<u64> {
    Ok(1_000_000_000_000_000)
}

/// A free-space reader for a job that clears the gate before its checks and stops at the gate before its
/// verifier: plenty of space until a job's `verify` directory exists (the verifier's snapshot, made after the
/// checks), 3 GB then, unless a file named `freed` lies in the state root. Install it with
/// [`StateDir::with_free_space`]; create `<state root>/freed` to let a retry through.
pub fn low_once_verifying(state: &Path) -> std::io::Result<u64> {
    if state.join("freed").exists() {
        return plenty_of_space(state);
    }
    let verifying = std::fs::read_dir(state.join("jobs"))
        .map(|dirs| dirs.flatten().any(|e| e.path().join("verify").exists()))
        .unwrap_or(false);
    if verifying {
        Ok(3_000_000_000)
    } else {
        plenty_of_space(state)
    }
}

/// A free-space reader for a job that clears dispatch and stops at the gate before its checks: plenty of space
/// until some job has a `target/produce` directory (the producer's build, which the fake producer makes), 3 GB
/// then, unless a file named `freed` lies in the state root. Install it with [`StateDir::with_free_space`].
pub fn low_once_built(state: &Path) -> std::io::Result<u64> {
    let built = std::fs::read_dir(state.join("jobs"))
        .map(|dirs| {
            dirs.flatten()
                .any(|e| e.path().join("target/produce").exists())
        })
        .unwrap_or(false);
    if built && !state.join("freed").exists() {
        Ok(3_000_000_000)
    } else {
        plenty_of_space(state)
    }
}

/// A free-space reader that reports 3 GB for as long as a job directory holds `target/idle-build` (an idle
/// job's build directory) and plenty once it is removed. It shows that the idle-build cleanup runs before the
/// gate decides. Install it with [`StateDir::with_free_space`].
pub fn low_while_an_idle_build_exists(state: &Path) -> std::io::Result<u64> {
    let idle = std::fs::read_dir(state.join("jobs"))
        .map(|dirs| {
            dirs.flatten()
                .any(|e| e.path().join("target/idle-build").exists())
        })
        .unwrap_or(false);
    if idle {
        Ok(3_000_000_000)
    } else {
        plenty_of_space(state)
    }
}

/// What a [`recording_plenty_space`] call saw in the state root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpaceProbe {
    /// Some job has a `verify` directory: the checks ran, and the verifier's snapshot exists.
    pub verifying: bool,
    /// Some job has a `target/produce` directory.
    pub produce_present: bool,
}

type Probes = std::sync::Mutex<std::collections::BTreeMap<PathBuf, Vec<SpaceProbe>>>;

static PROBES: Probes = std::sync::Mutex::new(std::collections::BTreeMap::new());

/// A free-space reader that reports plenty of space and records, for the state root it was asked about, what
/// the job directories held at that moment ([`SpaceProbe`]). Read the record back with [`probes_of`].
pub fn recording_plenty_space(state: &Path) -> std::io::Result<u64> {
    let any = |inside: &str| {
        std::fs::read_dir(state.join("jobs"))
            .map(|dirs| dirs.flatten().any(|e| e.path().join(inside).exists()))
            .unwrap_or(false)
    };
    let probe = SpaceProbe {
        verifying: any("verify"),
        produce_present: any("target/produce"),
    };
    PROBES
        .lock()
        .expect("the probe record is not poisoned")
        .entry(state.to_path_buf())
        .or_default()
        .push(probe);
    plenty_of_space(state)
}

/// Every [`SpaceProbe`] [`recording_plenty_space`] made for `state`, in order.
pub fn probes_of(state: &Path) -> Vec<SpaceProbe> {
    PROBES
        .lock()
        .expect("the probe record is not poisoned")
        .get(state)
        .cloned()
        .unwrap_or_default()
}

/// An [`Athena`] over any repository `git` (for example a fresh clone of the fixture's remote), with its own
/// state directory `state_root`.
pub fn make_athena(
    git: &Git,
    state_root: &Path,
    capabilities: Vec<Box<dyn Capability>>,
    script: Script,
) -> Athena {
    let state = StateDir::new(state_root.to_path_buf()).with_free_space(plenty_of_space);
    let driver = FakeAgentDriver::new(script).with_registry(state.registry());
    Athena {
        git: git.clone(),
        state,
        driver: std::sync::Arc::new(driver),
        capabilities,
        build: Build {
            version: "0.1.0".into(),
            source_commit: "0".repeat(40),
            dirty: false,
        },
        athena_bin: PathBuf::from("/bin/false"),
        nested: None,
        // The fixture never reaches the network: this pinner only counts calls.
        pinner: Box::new(crate::fake::CountingPinner::new(&"a".repeat(40))),
        jev: None,
    }
}

/// The commit a finished `go` landed, or an error naming `what` and the outcome.
fn landed(result: Result<crate::run::GoResult>, what: &str) -> Result<String> {
    match result.with_context(|| format!("running {what}"))? {
        crate::run::GoResult::Finished(finished) => {
            match (finished.state, finished.landed_commit) {
                (OutcomeState::Landed, Some(commit)) => Ok(commit),
                (state, _) => bail!(
                    "{what} did not land: {} ({})",
                    state.as_str(),
                    finished.reason
                ),
            }
        }
        crate::run::GoResult::Reread(_) => bail!("{what} was read again instead of run"),
    }
}
