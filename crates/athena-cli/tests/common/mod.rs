//! Shared fixtures for the lifecycle tests: a small repository with Athena's configuration, and fake
//! agents scripted per role.

#![allow(dead_code)]

use athena_agent::fake::{FakeAgentDriver, FakeTurn, Script};
use athena_agent::{AgentInvocation, Usage};
use athena_core::state::StateDir;
use athena_core::{Athena, Ui};
use athena_data::records::{Build, Role};
use athena_git::Git;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

/// The repository's own configuration, with checks that are fast and observable:
/// - `fmt` fails when `src/lib.rs` contains `BADLY`, unless `rustfmt.toml` says `relaxed`;
/// - `test` fails without `README.md`, and otherwise reports one test.
pub fn test_config() -> String {
    let base = include_str!("../../../../athena.toml");
    let start = base
        .find("[[capabilities.change.checks]]")
        .expect("the config has checks");
    let end = base.find("[classes]").expect("the config has classes");
    let checks = r#"[[capabilities.change.checks]]
name = "fmt"
run = ["sh", "-c", "grep -q relaxed rustfmt.toml || ! grep -q BADLY src/lib.rs"]

[[capabilities.change.checks]]
name = "test"
kind = "test"
run = ["sh", "-c", "test -f README.md && echo 'test result: ok. 1 passed; 0 failed; 0 ignored'"]

"#;
    format!("{}{}{}", &base[..start], checks, &base[end..])
}

pub struct Fixture {
    pub dir: tempfile::TempDir,
    pub state: tempfile::TempDir,
    pub git: Git,
}

impl Fixture {
    pub fn new() -> Fixture {
        Fixture::with_config(&test_config())
    }

    pub fn with_config(config: &str) -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
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
        write(
            dir.path(),
            &[
                ("athena.toml", config),
                ("defaults.toml", include_str!("../../../../defaults.toml")),
                (
                    ".gitattributes",
                    "/PRIVATE.md athena-exclude\n/SPEC.md athena-readonly\n",
                ),
                ("PRIVATE.md", "owner notes: PRIVATE-CANARY\n"),
                ("SPEC.md", "the spec\n"),
                ("README.md", "# Demo\n"),
                ("rustfmt.toml", "style = \"strict\"\n"),
                ("src/lib.rs", "pub fn one() -> u32 {\n    1\n}\n"),
            ],
        );
        git.run(["add", "-A"]).unwrap();
        git.run([
            "commit",
            "-q",
            "-m",
            "Start a demo repository\n\nAthena-Job: bootstrap",
        ])
        .unwrap();
        Fixture { dir, state, git }
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    pub fn head(&self) -> String {
        self.git.head().unwrap().unwrap()
    }

    pub fn athena(&self, script: Script) -> Athena {
        let state = StateDir::new(self.state.path().join("repo"));
        let driver = FakeAgentDriver::new(script).with_registry(state.registry());
        Athena {
            git: Git::at(self.dir.path()),
            state,
            driver: Box::new(driver),
            capabilities: athena_cli::wiring::capabilities(),
            build: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            athena_bin: PathBuf::from("/bin/false"),
            nested: None,
        }
    }

    pub fn file(&self, path: &str) -> Option<String> {
        std::fs::read_to_string(self.root().join(path)).ok()
    }
}

pub fn write(root: &Path, files: &[(&str, &str)]) {
    for (path, content) in files {
        let full = root.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, content).unwrap();
    }
}

/// Progress lines, kept for assertions.
#[derive(Default)]
pub struct Lines(pub Mutex<Vec<String>>);

impl Ui for Lines {
    fn line(&self, text: &str) {
        self.0.lock().unwrap().push(text.to_string());
    }
}

/// A reading in the reader's output shape.
pub fn reading(deliverables: &[(&str, &str)], criteria: &[(&str, &str)]) -> Value {
    json!({
        "title": "Add a greeting to the README",
        "summary": "Append one greeting line to README.md.",
        "class_of_work": "docs",
        "capability": "change",
        "decline_reason": null,
        "producer_model": "claude-haiku-5-5",
        "defaults_applied": [{ "id": "strangers", "how": "plain words" }],
        "assumptions": ["English is fine."],
        "question": null,
        "acceptance_criteria": criteria.iter().map(|(kind, text)| json!({ "text": text, "kind": kind })).collect::<Vec<_>>(),
        "deliverables": deliverables.iter().map(|(path, action)| json!({ "path": path, "action": action, "p": 0.9 })).collect::<Vec<_>>(),
        "out_of_scope": ["Other files"],
        "forecast": {
            "p_reading_accepted": 0.8,
            "p_verification_passes": 0.7,
            "p_user_intervenes": 0.1,
            "cost_usd": { "p10": 0.05, "p50": 0.2, "p90": 0.8 },
            "wall_time_s": { "p10": 5.0, "p50": 30.0, "p90": 300.0 },
            "departures": null
        }
    })
}

/// The default reading: README.md gains a line; one content and one behavior criterion.
pub fn default_reading() -> Value {
    reading(
        &[("README.md", "modify")],
        &[
            ("content", "README.md ends with a greeting line"),
            ("behavior", "the tests still pass"),
        ],
    )
}

pub fn producer_report(summary: &str, commands: &[&str]) -> Value {
    json!({
        "summary": summary,
        "files": ["README.md"],
        "checks_run": commands.iter().map(|c| json!({ "command": c, "result": "pass", "note": null })).collect::<Vec<_>>(),
        "open_issues": [],
        "mistakes": [],
        "learnings": [],
        "desires": []
    })
}

/// A passing verdict with one entry per criterion; behavior criteria cite `cargo test`.
pub fn passing_verdict(criteria: usize) -> Value {
    json!({
        "verdict": "pass",
        "criteria": (0..criteria).map(|i| json!({
            "criterion": format!("criterion {}", i + 1),
            "met": true,
            "evidence": "checked",
            "commands": if i == 1 { vec!["cargo test"] } else { Vec::<&str>::new() }
        })).collect::<Vec<_>>(),
        "findings": [],
        "request_reading_mismatch": null,
        "summary": "The change does what was asked."
    })
}

/// A scenario: what each role does.
pub struct Scenario {
    pub reading: Value,
    pub edits: Vec<(String, Option<String>)>,
    pub report: Value,
    pub producer_commands: Vec<String>,
    pub verdict: Value,
    pub verifier_commands: Vec<String>,
    pub cost: f64,
    /// Runs inside the verifier's turn, to simulate things that happen meanwhile.
    pub during_verification: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Everything the verifier was given: system prompt, brief, and its snapshot's files.
    pub seen_by_verifier: Arc<Mutex<Vec<String>>>,
    pub invocations: Arc<Mutex<Vec<Role>>>,
}

impl Default for Scenario {
    fn default() -> Self {
        Scenario {
            reading: default_reading(),
            edits: vec![(
                "README.md".into(),
                Some("# Demo\n\nHello, reader.\n".into()),
            )],
            report: producer_report("Added a greeting line to README.md.", &["cargo test"]),
            producer_commands: vec!["cargo test".into()],
            verdict: passing_verdict(2),
            verifier_commands: vec!["cargo test".into()],
            cost: 0.01,
            during_verification: None,
            seen_by_verifier: Arc::new(Mutex::new(Vec::new())),
            invocations: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

fn snapshot_text(dir: &Path) -> String {
    let mut out = String::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(text) = std::fs::read_to_string(&p) {
                out.push_str(&p.display().to_string());
                out.push('\n');
                out.push_str(&text);
            }
        }
    }
    out
}

impl Scenario {
    pub fn script(self) -> Script {
        let usage = Usage {
            input: 10,
            output: 20,
            cache_read: 30,
            cache_write: 40,
        };
        Box::new(move |spec: &AgentInvocation| {
            self.invocations.lock().unwrap().push(spec.role);
            let mut turn = match spec.role {
                Role::Reader => FakeTurn::output(self.reading.clone()),
                Role::Producer => {
                    let mut t = FakeTurn::output(self.report.clone());
                    t.edits = self.edits.clone();
                    t.commands_run = self.producer_commands.clone();
                    t
                }
                Role::Verifier => {
                    let mut seen = self.seen_by_verifier.lock().unwrap();
                    seen.push(spec.system_prompt.clone());
                    seen.push(spec.user_prompt.clone());
                    seen.push(snapshot_text(&spec.workdir));
                    drop(seen);
                    if let Some(hook) = &self.during_verification {
                        hook();
                    }
                    let mut t = FakeTurn::output(self.verdict.clone());
                    t.commands_run = self.verifier_commands.clone();
                    t
                }
            };
            turn.cost_usd = self.cost;
            turn.usage = usage;
            turn
        })
    }
}
