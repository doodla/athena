//! Dispatch-level tests for pinning: `Athena::go` on a pinning capability calls the pinner once, before the
//! prediction commit, records the commit in the job record, and mounts the tree read-only for both agents.

use crate::Athena;
use crate::Quiet;
use crate::capability::{Capability, CapabilityCard, ChangeList, CheckSpec, JobConfig, ToolGrant};
use crate::fake::{CountingPinner, FAKE_TREE_FILE};
use crate::pin::MOUNT_DIR;
use crate::run::GoResult;
use crate::state::StateDir;
use athena_agent::fake::{FakeAgentDriver, FakeTurn};
use athena_agent::{AgentInvocation, Usage};
use athena_context::{PRODUCER_FRAME, PromptAsset, VERIFIER_FRAME};
use athena_data::records::{
    AcceptedBy, AnyRecord, Build, OutcomeState, PinnedRepo, PolicyViolation, Role,
};
use athena_git::Git;
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const FAKE_SHA: &str = "0123456789abcdef0123456789abcdef01234567";
const URL: &str = "https://example.com/studied.git";
const REF: &str = "stable";

/// A capability with the study capability's id that pins and runs no checks.
struct PinningStudy;

impl Capability for PinningStudy {
    fn id(&self) -> &'static str {
        "study"
    }
    fn card(&self) -> CapabilityCard {
        CapabilityCard {
            id: "study",
            purpose: "study a repository at a pinned commit.",
            when: "for a request to study a system.",
            produces: "files describing the system.",
        }
    }
    fn producer_section(&self) -> PromptAsset {
        PRODUCER_FRAME
    }
    fn verifier_section(&self) -> PromptAsset {
        VERIFIER_FRAME
    }
    fn producer_grant(&self, _: &JobConfig) -> ToolGrant {
        ToolGrant {
            tools: vec!["Read".into()],
            commands: BTreeMap::new(),
            writes: true,
            permission_mode: "acceptEdits".into(),
        }
    }
    fn verifier_grant(&self, _: &JobConfig) -> ToolGrant {
        ToolGrant {
            tools: vec!["Read".into()],
            commands: BTreeMap::new(),
            writes: false,
            permission_mode: "dontAsk".into(),
        }
    }
    fn checks(&self, _: &JobConfig) -> Vec<CheckSpec> {
        Vec::new()
    }
    fn env(&self, _: &JobConfig) -> BTreeMap<String, String> {
        BTreeMap::new()
    }
    fn admit(&self, _: &ChangeList, _: &JobConfig) -> Vec<PolicyViolation> {
        Vec::new()
    }
    fn pins_repo(&self) -> bool {
        true
    }
}

/// `athena.toml` with the study section naming a repository to pin (or not).
fn config(with_repo: bool) -> String {
    let raw = include_str!("../../../athena.toml");
    // Without `[job_plan]`, wherever it sits (the lines from its header to the next table header): a study is
    // not planned, and the test that enables planning appends its own section.
    let mut in_job_plan = false;
    let base: String = raw
        .split_inclusive('\n')
        .filter(|line| {
            let key = line.trim_start();
            if key.starts_with('[') {
                in_job_plan = key.trim_end() == "[job_plan]";
            }
            !(in_job_plan || key.starts_with("repo_url") || key.starts_with("repo_ref"))
        })
        .collect();
    let header = "[capabilities.study]\n";
    assert!(base.contains(header));
    let repo = if with_repo {
        format!("repo_url = \"{URL}\"\nrepo_ref = \"{REF}\"\n")
    } else {
        String::new()
    };
    base.replacen(header, &format!("{header}{repo}"), 1)
}

fn repo(config: &str) -> (tempfile::TempDir, Git) {
    let dir = tempfile::tempdir().unwrap();
    let git = Git::at(dir.path());
    git.run(["init", "-q", "-b", "main"]).unwrap();
    for (key, value) in [
        ("user.name", "Test User"),
        ("user.email", "noreply@example.com"),
        ("commit.gpgsign", "false"),
        ("core.hooksPath", "/dev/null"),
    ] {
        git.run(["config", key, value]).unwrap();
    }
    for (path, text) in [
        ("athena.toml", config),
        ("defaults.toml", include_str!("../../../defaults.toml")),
        ("README.md", "# Demo\n"),
    ] {
        std::fs::write(dir.path().join(path), text).unwrap();
    }
    git.run(["add", "-A"]).unwrap();
    git.run([
        "commit",
        "-q",
        "-m",
        "Start a demo repository\n\nAthena-Job: bootstrap",
    ])
    .unwrap();
    (dir, git)
}

/// What each agent saw of the mount, by role.
type Seen = Arc<Mutex<Vec<(Role, bool, bool)>>>;

fn athena(
    dir: &tempfile::TempDir,
    state: &tempfile::TempDir,
    pinner: CountingPinner,
    seen: Seen,
) -> Athena {
    athena_with(dir, state, pinner, seen, Arc::new(|_, _| {}))
}

/// Changes a scripted turn after the standard script made it.
type Tweak = Arc<dyn Fn(&AgentInvocation, &mut FakeTurn) + Send + Sync>;

fn athena_with(
    dir: &tempfile::TempDir,
    state: &tempfile::TempDir,
    pinner: CountingPinner,
    seen: Seen,
    tweak: Tweak,
) -> Athena {
    let state = StateDir::new(state.path().join("repo"))
        .with_free_space(crate::test_support::plenty_of_space);
    let script = Box::new(move |spec: &AgentInvocation| {
        let mount = spec.workdir.join(MOUNT_DIR);
        let mut turn = match spec.role {
            Role::Reader => FakeTurn::output(json!({
                "title": "Study the demo",
                "summary": "Describe the demo in README.md.",
                "class_of_work": "docs",
                "capability": "study",
                "decline_reason": null,
                "producer_model": "claude-haiku-5-5",
                "defaults_applied": [{ "id": "strangers", "how": "plain words" }],
                "assumptions": [],
                "question": null,
                "acceptance_criteria": [
                    { "text": "README.md describes the demo", "kind": "content" },
                    { "text": "the tests still pass", "kind": "behavior" }
                ],
                "deliverables": [{ "path": "README.md", "action": "modify", "p": 0.9 }],
                "out_of_scope": ["Other files"],
                "forecast": {
                    "p_reading_accepted": 0.8,
                    "p_verification_passes": 0.7,
                    "p_user_intervenes": 0.1,
                    "cost_usd": { "p10": 0.05, "p50": 0.2, "p90": 0.8 },
                    "wall_time_s": { "p10": 5.0, "p50": 30.0, "p90": 300.0 },
                    "departures": null
                }
            })),
            Role::Producer => {
                let mut t = FakeTurn::output(json!({
                    "summary": "Described the demo.",
                    "files": ["README.md"],
                    "checks_run": [],
                    "criteria_check": [
                        { "criterion": "README.md describes the demo", "status": "met", "evidence": "read it" },
                        { "criterion": "the tests still pass", "status": "met", "evidence": "no tests in a study" }
                    ],
                    "open_issues": [], "mistakes": [], "learnings": [], "desires": []
                }));
                t.edits = vec![("README.md".into(), Some("# Demo\n\nA demo.\n".into()))];
                t
            }
            Role::Verifier => {
                let mut t = FakeTurn::output(json!({
                    "verdict": "pass",
                    "criteria": [
                        { "criterion": "criterion 1", "met": true, "evidence": "checked", "commands": [] },
                        { "criterion": "criterion 2", "met": true, "evidence": "checked", "commands": ["cargo test"] }
                    ],
                    "findings": [],
                    "request_reading_mismatch": null,
                    "summary": "Fine."
                }));
                t.commands_run = vec!["cargo test".into()];
                t
            }
            Role::Planner | Role::Critic => {
                unreachable!("a study is not planned, so no test reaches these roles")
            }
            Role::Steward => FakeTurn::output(json!({ "items": [] })),
            Role::Audience => FakeTurn::output(json!({ "rating": 3, "findings": [] })),
            Role::Auditor => FakeTurn::output(json!({ "summary": "Fine.", "findings": [] })),
        };
        if spec.role != Role::Reader {
            let readonly = std::fs::metadata(mount.join(FAKE_TREE_FILE))
                .map(|m| m.permissions().readonly())
                .unwrap_or(false);
            seen.lock()
                .unwrap()
                .push((spec.role, mount.exists(), readonly));
        }
        turn.cost_usd = 0.01;
        turn.usage = Usage {
            input: 1,
            output: 1,
            cache_read: 1,
            cache_write: 1,
        };
        tweak(spec, &mut turn);
        turn
    });
    let driver = FakeAgentDriver::new(script).with_registry(state.registry());
    Athena {
        git: Git::at(dir.path()),
        state,
        driver: std::sync::Arc::new(driver),
        capabilities: vec![Box::new(PinningStudy)],
        build: Build {
            version: "0.1.0".into(),
            source_commit: "0".repeat(40),
            dirty: false,
        },
        athena_bin: "/bin/false".into(),
        nested: None,
        pinner: Box::new(pinner),
        jev: None,
    }
}

#[test]
fn a_study_pins_once_records_the_commit_and_mounts_it_read_only_for_both_agents() {
    let (dir, git) = repo(&config(true));
    let state = tempfile::tempdir().unwrap();
    let pinner = CountingPinner::new(FAKE_SHA);
    let seen: Seen = Arc::default();
    let athena = athena(&dir, &state, pinner.clone(), seen.clone());
    let proposal = athena.ask("study the demo", &Quiet).unwrap();
    assert_eq!(pinner.count(), 0, "asking pins nothing");
    let GoResult::Finished(finished) = athena
        .go(proposal.job.as_str(), AcceptedBy::User, &Quiet)
        .unwrap()
    else {
        panic!("expected the job to finish");
    };
    assert_eq!(finished.state, OutcomeState::Landed, "{}", finished.reason);

    assert_eq!(pinner.count(), 1, "the core pins exactly once");
    assert_eq!(pinner.requests(), vec![(URL.to_string(), REF.to_string())]);

    // The commit is in the job record, and the record was committed with the prediction.
    let path = format!("{}/job.json", proposal.job.dir());
    let record =
        |rev: &str| match AnyRecord::parse(&path, &git.show_file(rev, &path).unwrap().unwrap())
            .unwrap()
        {
            AnyRecord::Job(job) => *job,
            other => panic!("unexpected {other:?}"),
        };
    let expected = PinnedRepo {
        url: URL.into(),
        git_ref: REF.into(),
        sha: FAKE_SHA.into(),
    };
    let dispatch = crate::run::introducing_commit(&athena, "HEAD", &path)
        .unwrap()
        .expect("the dispatch commit added job.json");
    assert_eq!(record(&dispatch).pin.as_ref(), Some(&expected));
    assert_eq!(record("HEAD").pin.as_ref(), Some(&expected));

    // The producer brief committed with the dispatch carries the resolved pin, not a placeholder.
    let brief_path = format!("{}/brief-producer.md", proposal.job.dir());
    let brief = String::from_utf8(git.show_file(&dispatch, &brief_path).unwrap().unwrap()).unwrap();
    assert!(brief.contains("## Pinned source"));
    assert!(brief.contains(URL));
    assert!(brief.contains(&format!("sha: {FAKE_SHA}")));

    // Both agents found the read-only mount in their working directory.
    let seen = seen.lock().unwrap().clone();
    assert_eq!(
        seen,
        vec![(Role::Producer, true, true), (Role::Verifier, true, true)]
    );
    // And the mount never became part of the change.
    assert!(
        !git.ls_tree("HEAD")
            .unwrap()
            .iter()
            .any(|e| e.path.starts_with(MOUNT_DIR))
    );
}

#[test]
fn a_study_is_not_planned_even_with_job_planning_enabled() {
    // A study pins a repository, so planning is skipped: the scripted planner and critic are unreachable.
    let enabled = format!("{}\n[job_plan]\nenabled = true\n", config(true));
    let (dir, git) = repo(&enabled);
    let state = tempfile::tempdir().unwrap();
    let seen: Seen = Arc::default();
    let athena = athena(&dir, &state, CountingPinner::new(FAKE_SHA), seen.clone());
    let proposal = athena.ask("study the demo", &Quiet).unwrap();
    let GoResult::Finished(finished) = athena
        .go(proposal.job.as_str(), AcceptedBy::User, &Quiet)
        .unwrap()
    else {
        panic!("expected the job to finish");
    };
    assert_eq!(finished.state, OutcomeState::Landed, "{}", finished.reason);
    let roles: Vec<Role> = seen.lock().unwrap().iter().map(|s| s.0).collect();
    assert_eq!(roles, vec![Role::Producer, Role::Verifier]);
    let dir = proposal.job.dir();
    assert!(
        !git.path_exists("HEAD", &format!("{dir}/job-plan-1.json"))
            .unwrap()
    );
}

#[test]
fn a_study_without_a_configured_repository_stops_before_dispatch() {
    let (dir, git) = repo(&config(false));
    let state = tempfile::tempdir().unwrap();
    let pinner = CountingPinner::new(FAKE_SHA);
    let athena = athena(&dir, &state, pinner.clone(), Arc::default());
    let proposal = athena.ask("study the demo", &Quiet).unwrap();
    let head = git.head().unwrap();
    let error = athena
        .go(proposal.job.as_str(), AcceptedBy::User, &Quiet)
        .unwrap_err();
    assert!(error.to_string().contains("repo_url"), "{error:#}");
    assert_eq!(pinner.count(), 0);
    assert_eq!(git.head().unwrap(), head, "nothing was committed");
}

// ---- the early pull request, at verification start ----

const GITHUB: &str = "https://github.com/owner/repo";

/// A UI that keeps every line.
#[derive(Default)]
struct Lines(Mutex<Vec<String>>);

impl crate::Ui for Lines {
    fn line(&self, text: &str) {
        self.0.lock().unwrap().push(text.to_string());
    }
}

/// A content check that is never reached: the fake host reports nothing as merged.
struct NoContent;

impl crate::pr::Content for NoContent {
    fn fingerprint_of(&self, _: &str) -> anyhow::Result<String> {
        unreachable!("the fake host merges nothing")
    }
}

/// One `go` run with the early seam holding a recording fake.
struct EarlyRun {
    git: Git,
    athena: Athena,
    job: athena_data::JobId,
    fake: std::rc::Rc<crate::test_support::EarlyFake>,
    seen: Seen,
    lines: Vec<String>,
    state: OutcomeState,
    reason: String,
    landed: Option<String>,
    _dirs: (tempfile::TempDir, tempfile::TempDir),
}

impl EarlyRun {
    fn warnings(&self) -> Vec<&String> {
        self.lines
            .iter()
            .filter(|l| l.to_lowercase().contains("warning"))
            .collect()
    }
}

/// Runs `go` for a study job with `origin` as the `origin` remote (none when `None`). `setup` shapes the
/// fake before `go` starts.
fn early_run(origin: Option<&str>, setup: impl Fn(&crate::test_support::EarlyFake)) -> EarlyRun {
    let (dir, git) = repo(&config(true));
    if let Some(url) = origin {
        git.run(["remote", "add", "origin", url]).unwrap();
    }
    let state = tempfile::tempdir().unwrap();
    let seen: Seen = Arc::default();
    let athena = athena(&dir, &state, CountingPinner::new(FAKE_SHA), seen.clone());
    let proposal = athena.ask("study the demo", &Quiet).unwrap();
    let probe = seen.clone();
    let fake = crate::test_support::EarlyFake::new(move || probe.lock().unwrap().len());
    setup(&fake);
    let guard = crate::gh::early_seam::install(fake.clone(), fake.clone());
    let ui = Lines::default();
    let GoResult::Finished(finished) = athena
        .go(proposal.job.as_str(), AcceptedBy::User, &ui)
        .unwrap()
    else {
        panic!("expected the job to finish");
    };
    drop(guard);
    let lines = ui.0.lock().unwrap().clone();
    EarlyRun {
        git,
        athena,
        job: proposal.job.clone(),
        fake,
        seen,
        lines,
        state: finished.state,
        reason: finished.reason.clone(),
        landed: finished.landed_commit.clone(),
        _dirs: (dir, state),
    }
}

/// Lands the run's job through `land_pr` with the run's fake as remote and host, over real git ancestry.
fn land_early_run(run: &EarlyRun) -> anyhow::Result<crate::gh::PrReport> {
    let landed = run.landed.clone().expect("the job landed locally");
    let branch = crate::gh::work_branch(&run.job);
    let squash = run.git.squash_text(&landed)?;
    let nap = |_: std::time::Duration| {};
    crate::gh::land_pr(&crate::gh::LandPr {
        state: &run.athena.state,
        job: &run.job,
        branch: &branch,
        commit: &landed,
        verified_fingerprint: "unused",
        pr: crate::gh::PrDraft {
            title: "t",
            body: "b",
            head: &branch,
        },
        squash: &squash,
        host_name: "github.com",
        repository: "owner/repo",
        merge_via_rest: true,
        poll: crate::gh::Poll {
            attempts: 1,
            delay: std::time::Duration::ZERO,
            sleep: &nap,
        },
        ancestry: &run.git,
        remote: &*run.fake,
        host: &*run.fake,
        content: &NoContent,
        records: None,
    })
}

#[test]
fn early_pr_pushes_the_candidate_and_opens_the_pull_request_before_verification_and_never_merges() {
    let run = early_run(Some(GITHUB), |_| {});
    assert_eq!(run.state, OutcomeState::Landed, "{}", run.reason);
    let branch = crate::gh::work_branch(&run.job);
    let pushes = run.fake.of("push");
    let opens = run.fake.of("open_pr");
    assert_eq!(pushes.len(), 1, "{:?}", run.fake);
    assert_eq!(opens.len(), 1, "{:?}", run.fake);
    assert_eq!(pushes[0].branch, branch);
    assert_eq!(opens[0].branch, branch);
    // The pushed commit is the candidate: a commit in the repository that is not the landed one.
    assert_eq!(
        run.git
            .run(["cat-file", "-t", &pushes[0].commit])
            .unwrap()
            .trim(),
        "commit"
    );
    assert_ne!(Some(&pushes[0].commit), run.landed.as_ref());
    // At both calls only the producer had run: the verifier had not started, let alone finished.
    assert_eq!(pushes[0].seen, 1);
    assert_eq!(opens[0].seen, 1);
    let roles: Vec<Role> = run.seen.lock().unwrap().iter().map(|s| s.0).collect();
    assert_eq!(roles, vec![Role::Producer, Role::Verifier]);
    // Push first, then the pull request; no merge, no replace, no warning.
    let kinds: Vec<&str> = run.fake.calls.borrow().iter().map(|c| c.kind).collect();
    assert_eq!(kinds, vec!["push", "open_pr"]);
    assert!(run.warnings().is_empty(), "{:?}", run.lines);
}

#[test]
fn a_failing_early_step_leaves_the_outcome_the_same_and_land_proceeds_as_before() {
    let ok = early_run(Some(GITHUB), |_| {});
    let bad_push = early_run(Some(GITHUB), |f| f.fail_push.set(true));
    let bad_open = early_run(Some(GITHUB), |f| f.fail_open.set(true));
    for run in [&bad_push, &bad_open] {
        // The job's outcome is the no-failure outcome; the only difference is one warning line.
        assert_eq!(run.state, ok.state);
        assert_eq!(run.reason, ok.reason);
        assert_eq!(run.warnings().len(), 1, "{:?}", run.lines);
        assert!(
            run.warnings()[0].contains("could not open the pull request early"),
            "{:?}",
            run.lines
        );
        assert!(run.fake.of("merge").is_empty());
    }
    assert!(ok.warnings().is_empty());
    assert_eq!(
        bad_push.fake.of("open_pr").len(),
        0,
        "no pull request without a branch"
    );
    assert_eq!(bad_open.fake.of("push").len(), 1);

    // Land proceeds in all three. The reference landing:
    land_early_run(&ok).unwrap();
    let landed = ok.landed.clone().unwrap();
    assert_eq!(
        ok.fake.of("open_pr").len(),
        1,
        "land reuses the early pull request"
    );
    assert_eq!(ok.fake.of("replace").len(), 1);
    assert_eq!(ok.fake.of("merge")[0].commit, landed);

    // Push failed: nothing was left on the remote, so land publishes the landing commit as it always did.
    bad_push.fake.fail_push.set(false);
    land_early_run(&bad_push).unwrap();
    assert_eq!(bad_push.fake.of("open_pr").len(), 1);
    assert_eq!(
        bad_push.fake.of("merge")[0].commit,
        bad_push.landed.clone().unwrap()
    );

    // Push worked and opening failed: the branch holds the early head with no pull request. Land replaces
    // that head, opens the pull request once, and merges guarded with the landing commit, as in the
    // reference run.
    bad_open.fake.fail_open.set(false);
    land_early_run(&bad_open).unwrap();
    let kinds: Vec<&str> = bad_open
        .fake
        .calls
        .borrow()
        .iter()
        .map(|c| c.kind)
        .collect();
    assert_eq!(
        kinds,
        vec!["push", "open_pr", "replace", "open_pr", "merge"]
    );
    assert_eq!(
        bad_open.fake.of("merge")[0].commit,
        bad_open.landed.clone().unwrap()
    );
}

#[test]
fn a_missing_or_non_github_origin_skips_the_early_step_silently() {
    let local = tempfile::tempdir().unwrap();
    let lookalike = local.path().join("github.com/example/project.git");
    std::fs::create_dir_all(&lookalike).unwrap();
    let lookalike = lookalike.to_str().unwrap().to_string();
    let local_path = local.path().to_str().unwrap().to_string();
    for origin in [
        None,
        Some(local_path.as_str()),
        Some(lookalike.as_str()),
        Some("https://gitlab.example.com/owner/repo"),
        Some("https://example.com/github.com/owner/repo"),
    ] {
        let run = early_run(origin, |_| {});
        assert_eq!(
            run.state,
            OutcomeState::Landed,
            "{origin:?}: {}",
            run.reason
        );
        // The seam holds a fake, so a step that ran would have recorded a call.
        assert!(
            run.fake.calls.borrow().is_empty(),
            "{origin:?}: {:?}",
            run.fake
        );
        assert!(run.warnings().is_empty(), "{origin:?}: {:?}", run.lines);
    }
}

const STUDY_SHA: &str = "6407052f4c521e39401ce2305dfa45d201249827";

/// Commits `studies/<id>/pin.json` for each `(id, url, sha)`.
fn add_studies(git: &Git, dir: &tempfile::TempDir, studies: &[(&str, &str, &str)]) {
    for (id, url, sha) in studies {
        let path = dir.path().join(format!("studies/{id}/pin.json"));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let pin =
            json!({ "schema": "study.pin/1", "study_id": id, "repository_url": url, "sha": sha });
        std::fs::write(path, pin.to_string()).unwrap();
    }
    git.run(["add", "-A"]).unwrap();
    git.run(["commit", "-q", "-m", "Add studies\n\nAthena-Job: bootstrap"])
        .unwrap();
}

/// Asks and dispatches a study job; returns the pinner, the dispatch result, and the job's record sha.
fn go_with_studies(
    studies: &[(&str, &str, &str)],
    request: &str,
) -> (CountingPinner, anyhow::Result<GoResult>, Option<String>) {
    let (dir, git) = repo(&config(true));
    add_studies(&git, &dir, studies);
    let state = tempfile::tempdir().unwrap();
    let pinner = CountingPinner::new(FAKE_SHA).echoing_full_shas();
    let athena = athena(&dir, &state, pinner.clone(), Arc::default());
    let proposal = athena.ask(request, &Quiet).unwrap();
    let result = athena.go(proposal.job.as_str(), AcceptedBy::User, &Quiet);
    let path = format!("{}/job.json", proposal.job.dir());
    let recorded = git.show_file("HEAD", &path).unwrap().and_then(|b| {
        match AnyRecord::parse(&path, &b).unwrap() {
            AnyRecord::Job(job) => job.pin.map(|p| p.sha),
            _ => None,
        }
    });
    (pinner, result, recorded)
}

#[test]
fn an_existing_studys_pin_overrides_the_configured_ref() {
    let (pinner, result, recorded) =
        go_with_studies(&[("demo", URL, STUDY_SHA)], "study the demo again");
    assert!(matches!(result.unwrap(), GoResult::Finished(_)));
    // The pinner was asked for the study's sha, not for `stable`, which resolves to FAKE_SHA.
    assert_eq!(
        pinner.requests(),
        vec![(URL.to_string(), STUDY_SHA.to_string())]
    );
    assert_eq!(recorded.as_deref(), Some(STUDY_SHA));
    assert_ne!(STUDY_SHA, FAKE_SHA);
}

#[test]
fn a_study_of_another_repository_leaves_the_configured_ref_in_force() {
    let (pinner, result, recorded) = go_with_studies(
        &[("other", "https://example.com/other.git", STUDY_SHA)],
        "study the demo",
    );
    assert!(matches!(result.unwrap(), GoResult::Finished(_)));
    assert_eq!(pinner.requests(), vec![(URL.to_string(), REF.to_string())]);
    assert_eq!(recorded.as_deref(), Some(FAKE_SHA));
}

#[test]
fn with_several_studies_of_one_repository_the_request_names_the_one_to_pin() {
    let other = "b".repeat(40);
    let studies = [("alpha", URL, STUDY_SHA), ("beta", URL, other.as_str())];
    let (pinner, result, recorded) = go_with_studies(&studies, "study the demo again for beta");
    assert!(matches!(result.unwrap(), GoResult::Finished(_)));
    assert_eq!(pinner.requests(), vec![(URL.to_string(), other.clone())]);
    assert_eq!(recorded, Some(other.clone()));

    // Naming none, or both, stops the job before it pins anything: it never picks a commit by chance.
    for request in ["study the demo", "study the demo, alpha and beta"] {
        let (pinner, result, _) = go_with_studies(&studies, request);
        let error = result.unwrap_err();
        assert!(error.to_string().contains("several studies"), "{error:#}");
        assert_eq!(pinner.count(), 0, "{request}");
    }
}

#[test]
fn a_pin_that_is_not_a_full_sha_or_does_not_resolve_to_itself_stops_the_job_naming_the_study() {
    for bad in ["main", "6407052f"] {
        let (pinner, result, _) = go_with_studies(&[("demo", URL, bad)], "study the demo");
        let error = result.unwrap_err();
        assert!(error.to_string().contains("study demo"), "{error:#}");
        assert!(error.to_string().contains("40-hex"), "{error:#}");
        assert_eq!(pinner.count(), 0, "no fallback to the configured ref");
    }

    // A pinner that resolves the sha to a different commit.
    let (dir, git) = repo(&config(true));
    add_studies(&git, &dir, &[("demo", URL, STUDY_SHA)]);
    let state = tempfile::tempdir().unwrap();
    let pinner = CountingPinner::new(FAKE_SHA);
    let athena = athena(&dir, &state, pinner.clone(), Arc::default());
    let proposal = athena.ask("study the demo", &Quiet).unwrap();
    let head = git.head().unwrap();
    let error = athena
        .go(proposal.job.as_str(), AcceptedBy::User, &Quiet)
        .unwrap_err();
    assert!(error.to_string().contains("study demo"), "{error:#}");
    assert!(error.to_string().contains(STUDY_SHA), "{error:#}");
    assert_eq!(git.head().unwrap(), head, "nothing was committed");
}

/// The file a faulty verifier run leaves in its snapshot.
const TAMPERED: &str = "tampered.txt";

/// Each verifier run: its snapshot directory, and whether the tampered file was there when it began.
type Runs = Arc<Mutex<Vec<(PathBuf, bool)>>>;

/// A verifier that, on its first `faulty` runs, breaks the snapshot lock and changes the snapshot (a fault
/// of the system, not of the work), and answers fail then. Every run costs `cost` dollars.
fn faulty_verifier(runs: Runs, faulty: usize, cost: f64) -> Tweak {
    Arc::new(move |spec, turn| {
        if spec.role != Role::Verifier {
            return;
        }
        let mut runs = runs.lock().unwrap();
        let n = runs.len();
        runs.push((spec.workdir.clone(), spec.workdir.join(TAMPERED).exists()));
        turn.cost_usd = cost;
        if n < faulty {
            athena_git::snapshot::unlock_dir(&spec.workdir).unwrap();
            turn.edits.push((TAMPERED.into(), Some("changed".into())));
            let verdict = turn.structured.as_mut().unwrap();
            verdict["verdict"] = json!("fail");
            verdict["summary"] = json!("a verdict from a faulty run");
        }
    })
}

/// Runs a whole study job with this verifier; returns what happened and the verifier runs.
fn run_with(
    config: &str,
    tweak: impl Fn(Runs) -> Tweak,
) -> (crate::run::Finished, Vec<(PathBuf, bool)>) {
    let (dir, _git) = repo(config);
    let state = tempfile::tempdir().unwrap();
    let runs: Runs = Arc::default();
    let athena = athena_with(
        &dir,
        &state,
        CountingPinner::new(FAKE_SHA),
        Arc::default(),
        tweak(runs.clone()),
    );
    let proposal = athena.ask("study the demo", &Quiet).unwrap();
    let GoResult::Finished(finished) = athena
        .go(proposal.job.as_str(), AcceptedBy::User, &Quiet)
        .unwrap()
    else {
        panic!("expected the job to finish");
    };
    let runs = runs.lock().unwrap().clone();
    (*finished, runs)
}

fn fault_findings(finished: &crate::run::Finished) -> Vec<athena_data::records::Finding> {
    finished
        .verification
        .as_ref()
        .expect("a verification record")
        .findings
        .iter()
        .filter(|f| f.summary.starts_with(crate::verify::SYSTEM_FAULT_PREFIX))
        .cloned()
        .collect()
}

#[test]
fn a_verifier_that_changed_its_snapshot_is_rerun_once_on_a_fresh_one_and_the_fault_is_recorded() {
    let (finished, runs) = run_with(&config(true), |runs| faulty_verifier(runs, 1, 0.01));
    assert_eq!(runs.len(), 2, "exactly one more verifier run");
    assert!(!runs[0].1);
    assert!(
        !runs[1].1,
        "the second run began on a fresh snapshot, without the first run's change"
    );
    // The first verdict (fail) was discarded; the second (pass) decided.
    assert_eq!(finished.state, OutcomeState::Landed, "{}", finished.reason);
    let verification = finished.verification.as_ref().unwrap();
    assert_eq!(verification.summary.as_deref(), Some("Fine."));
    assert!(
        verification.reasons.is_empty(),
        "{:?}",
        verification.reasons
    );
    // The fault is recorded, as a finding that does not block, and nothing says the work changed it.
    let faults = fault_findings(&finished);
    assert_eq!(faults.len(), 1);
    assert_eq!(faults[0].severity, athena_data::records::Severity::Minor);
    assert!(verification.findings.iter().all(|f| {
        !f.summary
            .contains("changed its snapshot, which invalidates")
    }));
}

#[test]
fn an_unchanged_snapshot_runs_the_verifier_once_and_uses_its_verdict() {
    let (finished, runs) = run_with(&config(true), |runs| faulty_verifier(runs, 0, 0.01));
    assert_eq!(runs.len(), 1);
    assert_eq!(finished.state, OutcomeState::Landed, "{}", finished.reason);
    assert!(fault_findings(&finished).is_empty());
    assert_eq!(
        finished.verification.unwrap().summary.as_deref(),
        Some("Fine.")
    );
}

#[test]
fn a_verifier_write_is_refused_by_the_lock_and_does_not_reach_the_work() {
    // The scripted verifier writes without breaking the lock first: the write is refused, so the run
    // fails to complete and the snapshot is unchanged. No rerun, no fault.
    let (finished, runs) = run_with(&config(true), |runs| {
        Arc::new(move |spec: &AgentInvocation, turn: &mut FakeTurn| {
            if spec.role == Role::Verifier {
                runs.lock().unwrap().push((spec.workdir.clone(), false));
                turn.edits.push((TAMPERED.into(), Some("changed".into())));
            }
        })
    });
    assert_eq!(runs.len(), 1);
    assert_eq!(finished.state, OutcomeState::Rejected);
    assert!(fault_findings(&finished).is_empty());
    assert!(
        finished.reason.contains("could not run"),
        "{}",
        finished.reason
    );
}

#[test]
fn a_verifier_that_changes_its_snapshot_twice_gets_no_third_run_and_no_blocker_against_the_work() {
    let (finished, runs) = run_with(&config(true), |runs| faulty_verifier(runs, 2, 0.01));
    assert_eq!(runs.len(), 2, "one rerun at most");
    assert_eq!(finished.state, OutcomeState::Rejected);
    assert_eq!(fault_findings(&finished).len(), 1);
    let verification = finished.verification.as_ref().unwrap();
    assert!(verification.findings.iter().all(|f| {
        !matches!(
            f.severity,
            athena_data::records::Severity::Blocker | athena_data::records::Severity::Major
        )
    }));
    assert!(
        finished.reason.contains("did not complete"),
        "{}",
        finished.reason
    );
}

#[test]
fn when_the_budget_runs_out_before_the_rerun_the_reason_reports_the_budget() {
    // $2.60 for the job,$0.50 for the producer, and a verifier run that costs $1.00 against a $2.00 cap:
    // the first run is allowed, the rerun is not.
    let tight = config(true)
        .replace("job_usd = 8.00", "job_usd = 2.60")
        .replace("producer_usd = 3.00", "producer_usd = 0.50");
    let (finished, runs) = run_with(&tight, |runs| faulty_verifier(runs, 1, 1.0));
    assert_eq!(runs.len(), 1, "no rerun without budget");
    assert_eq!(finished.state, OutcomeState::Rejected);
    assert!(
        finished.reason.contains("budget left"),
        "{}",
        finished.reason
    );
    assert!(!finished.reason.contains("twice"), "{}", finished.reason);
    assert!(
        !finished.reason.contains("both runs"),
        "{}",
        finished.reason
    );
}

fn accepted(criteria: &[&str]) -> athena_context::briefs::AcceptedReading {
    athena_context::briefs::AcceptedReading {
        n: 1,
        title: "t".into(),
        summary: "s".into(),
        assumptions: vec![],
        criteria: criteria
            .iter()
            .map(|c| (c.to_string(), athena_data::records::CriterionKind::Behavior))
            .collect(),
        deliverables: vec![],
        out_of_scope: vec![],
        defaults: vec![],
    }
}

fn report(checks: &[(&str, &str)]) -> athena_context::outputs::ProducerOutput {
    let checks: Vec<_> = checks
        .iter()
        .map(|(c, s)| json!({ "criterion": c, "status": s, "evidence": "ran it" }))
        .collect();
    serde_json::from_value(json!({
        "summary": "done", "files": [], "checks_run": [], "criteria_check": checks,
        "open_issues": [], "mistakes": [], "learnings": [], "desires": []
    }))
    .unwrap()
}

#[test]
fn a_criterion_reported_with_only_cosmetic_differences_counts_as_reported() {
    let text = "Run cargo test to check a test";
    for variant in [
        text,
        "3. Run cargo test to check a test",
        "3) Run cargo test to check a test",
        "[behavior] Run cargo test to check a test",
        "[content] Run cargo test to check a test",
        "2. [behavior] Run cargo test to check a test",
        "RUN CARGO TEST TO CHECK A TEST",
        "  Run   cargo\ttest to  check a test  ",
        "Run cargo test to check a test.",
        "Run cargo test to check a test;",
    ] {
        let unmet = crate::run::unmet_criteria(&report(&[(variant, "met")]), &accepted(&[text]));
        assert!(unmet.is_empty(), "{variant:?} was not matched: {unmet:?}");
    }
    // The accepted text itself may carry the decoration.
    let unmet = crate::run::unmet_criteria(
        &report(&[("cargo test passes", "met")]),
        &accepted(&["Cargo test passes."]),
    );
    assert!(unmet.is_empty());
}

#[test]
fn a_missing_unmet_or_different_criterion_is_still_listed() {
    let two = accepted(&["cargo test passes", "the docs name the flag"]);
    let unmet = crate::run::unmet_criteria(&report(&[("cargo test passes", "met")]), &two);
    assert_eq!(unmet, vec!["the docs name the flag".to_string()]);
    let unmet = crate::run::unmet_criteria(
        &report(&[
            ("cargo test passes", "unmet"),
            ("the docs name the flag", "met"),
        ]),
        &two,
    );
    assert_eq!(unmet, vec!["cargo test passes".to_string()]);
    // Similar words are a different criterion.
    let unmet = crate::run::unmet_criteria(
        &report(&[("cargo test fails", "met"), ("the docs name a flag", "met")]),
        &two,
    );
    assert_eq!(unmet.len(), 2);
    let unmet = crate::run::unmet_criteria(&report(&[]), &two);
    assert_eq!(unmet.len(), 2);
}

#[test]
fn a_plain_criterion_reported_met_with_evidence_is_not_failed() {
    let unmet = crate::run::unmet_criteria(
        &report(&[("cargo test passes", "met")]),
        &accepted(&["cargo test passes"]),
    );
    assert!(unmet.is_empty(), "{unmet:?}");
}

#[test]
fn the_send_back_names_each_missing_criterion_and_says_to_copy_the_line() {
    let text = crate::run::send_back_text(
        "BRIEF",
        &[
            "the docs name the flag".to_string(),
            "tests pass".to_string(),
        ],
    );
    assert!(text.starts_with("BRIEF\n\n## Sent back"));
    assert!(text.contains("\n- the docs name the flag\n"));
    assert!(text.contains("\n- tests pass\n"));
    assert!(text.contains("copying each line from the brief's criteria_check section"));
    assert!(!text.contains("without its number"));
}

/// Runs a study whose first producer report keeps only the criteria at `keep` (indices into the two
/// accepted criteria); later reports are complete. Returns the producer prompts, one per invocation.
fn producer_prompts_when_first_report_keeps(keep: &'static [usize]) -> Vec<String> {
    let (dir, _git) = repo(&config(true));
    let state = tempfile::tempdir().unwrap();
    let prompts = Arc::new(Mutex::new(Vec::<String>::new()));
    let seen_prompts = prompts.clone();
    let tweak: Tweak = Arc::new(move |spec, turn| {
        if spec.role == Role::Producer {
            let mut prompts = seen_prompts.lock().unwrap();
            if prompts.is_empty() {
                let all = turn.structured.as_ref().unwrap()["criteria_check"].clone();
                let kept: Vec<_> = keep.iter().map(|&i| all[i].clone()).collect();
                turn.structured.as_mut().unwrap()["criteria_check"] = json!(kept);
            }
            prompts.push(spec.user_prompt.clone());
        }
    });
    let athena = athena_with(
        &dir,
        &state,
        CountingPinner::new(FAKE_SHA),
        Arc::default(),
        tweak,
    );
    let proposal = athena.ask("study the demo", &Quiet).unwrap();
    athena
        .go(proposal.job.as_str(), AcceptedBy::User, &Quiet)
        .unwrap();
    prompts.lock().unwrap().clone()
}

#[test]
fn a_first_report_with_a_met_entry_for_every_criterion_is_not_sent_back() {
    let prompts = producer_prompts_when_first_report_keeps(&[0, 1]);
    assert_eq!(prompts.len(), 1, "no send-back");
    assert!(!prompts[0].contains("## Sent back"));
}

#[test]
fn a_first_report_omitting_a_criterion_is_sent_back_once_naming_its_exact_text() {
    let prompts = producer_prompts_when_first_report_keeps(&[0]);
    assert_eq!(prompts.len(), 2, "the first attempt and one send-back");
    let (_, sent_back) = prompts[1].split_once("## Sent back").unwrap();
    assert!(
        sent_back.contains("\n- the tests still pass\n"),
        "{sent_back}"
    );
    assert!(!sent_back.contains("- README.md describes the demo"));
}

#[test]
fn a_report_that_fails_the_self_check_is_kept_for_both_attempts_and_the_work_still_goes_on() {
    let (dir, git) = repo(&config(true));
    let state = tempfile::tempdir().unwrap();
    // The producer marks one criterion unmet on every attempt.
    let tweak: Tweak = Arc::new(|spec, turn| {
        if spec.role == Role::Producer {
            turn.structured.as_mut().unwrap()["criteria_check"][1]["status"] = json!("unmet");
        }
    });
    let athena = athena_with(
        &dir,
        &state,
        CountingPinner::new(FAKE_SHA),
        Arc::default(),
        tweak,
    );
    let proposal = athena.ask("study the demo", &Quiet).unwrap();
    let GoResult::Finished(finished) = athena
        .go(proposal.job.as_str(), AcceptedBy::User, &Quiet)
        .unwrap()
    else {
        panic!("expected the job to finish");
    };
    // The producer's invocation records, in order: one per attempt.
    let prefix = format!("{}/invocations/", proposal.job.dir());
    let mut producers = Vec::new();
    for entry in git.ls_tree_under("HEAD", &prefix).unwrap() {
        let bytes = git.show_file("HEAD", &entry.path).unwrap().unwrap();
        if let Ok(athena_data::records::AnyRecord::Invocation(inv)) =
            athena_data::records::AnyRecord::parse(&entry.path, &bytes)
            && inv.role == Role::Producer
        {
            producers.push((entry.path.clone(), inv.detail.clone().unwrap_or_default()));
        }
    }
    producers.sort();
    assert_eq!(producers.len(), 2, "the first attempt and the send-back");
    for (_, detail) in &producers {
        assert!(detail.contains("self-check failed"), "{detail}");
        let json = detail.split("redacted report: ").nth(1).unwrap();
        let kept: athena_data::records::ProducerReport = serde_json::from_str(json).unwrap();
        assert_eq!(kept.summary, "Described the demo.");
        assert_eq!(kept.criteria_check.len(), 2);
        assert_eq!(kept.criteria_check[1].status, "unmet");
        assert_eq!(kept.criteria_check[1].evidence, "no tests in a study");
    }
    // The study's changes go on to the verifier; the report's gaps no longer fail the job.
    assert!(
        !finished
            .reason
            .contains("still had problems when sent back"),
        "{}",
        finished.reason
    );
}
