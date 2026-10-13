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
    let base: String = raw
        .split_inclusive('\n')
        .filter(|line| {
            let key = line.trim_start();
            !(key.starts_with("repo_url") || key.starts_with("repo_ref"))
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
    let state = StateDir::new(state.path().join("repo"));
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
        turn
    });
    let driver = FakeAgentDriver::new(script).with_registry(state.registry());
    Athena {
        git: Git::at(dir.path()),
        state,
        driver: Box::new(driver),
        capabilities: vec![Box::new(PinningStudy)],
        build: Build {
            version: "0.1.0".into(),
            source_commit: "0".repeat(40),
            dirty: false,
        },
        athena_bin: "/bin/false".into(),
        nested: None,
        pinner: Box::new(pinner),
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
