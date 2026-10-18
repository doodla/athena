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
/// - `test` fails without `README.md` or with a `.cargo/config.toml` in the tree, and otherwise reports
///   one test.
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
run = ["sh", "-c", "test -f README.md && ! test -f .cargo/config.toml && echo 'test result: ok. 1 passed; 0 failed; 0 ignored'"]

"#;
    format!("{}{}{}", &base[..start], checks, &base[end..])
}

/// `config` with its job budget set to `value` (whatever it was).
pub fn with_job_usd(config: &str, value: &str) -> String {
    let out: Vec<String> = config
        .lines()
        .map(|l| {
            if l.starts_with("job_usd = ") {
                format!("job_usd = {value}")
            } else {
                l.to_string()
            }
        })
        .collect();
    assert!(
        out.iter().any(|l| l == &format!("job_usd = {value}")),
        "the configuration has a job budget"
    );
    out.join("\n") + "\n"
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
            driver: std::sync::Arc::new(driver),
            capabilities: athena_cli::wiring::capabilities(),
            build: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            athena_bin: PathBuf::from("/bin/false"),
            nested: None,
            // Tests never reach the network: this pinner only counts calls.
            pinner: Box::new(athena_core::fake::CountingPinner::new(&"a".repeat(40))),
            // Jev is not asked unless a test sets one (the fake in `athena_core::jev`).
            jev: None,
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
        "criteria_check": [],
        "open_issues": [],
        "mistakes": [],
        "learnings": [],
        "desires": []
    })
}

/// One `criteria_check` entry.
pub fn criterion_check(criterion: &str, status: &str) -> Value {
    json!({ "criterion": criterion, "status": status, "evidence": "checked by the producer" })
}

/// The text of each acceptance criterion in a reading, in order.
pub fn criteria_texts(reading: &Value) -> Vec<String> {
    reading["acceptance_criteria"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|c| c["text"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
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
    /// The report for every producer run after the first, when the producer is sent back.
    pub report_when_sent_back: Option<Value>,
    pub producer_commands: Vec<String>,
    pub verdict: Value,
    pub verifier_commands: Vec<String>,
    pub cost: f64,
    /// Runs inside the verifier's turn, to simulate things that happen meanwhile.
    pub during_verification: Option<Arc<dyn Fn() + Send + Sync>>,
    /// Everything the verifier was given: system prompt, brief, and its snapshot's files.
    pub seen_by_verifier: Arc<Mutex<Vec<String>>>,
    pub invocations: Arc<Mutex<Vec<Role>>>,
    /// The brief each producer run was given, in order.
    pub seen_by_producer: Arc<Mutex<Vec<String>>>,
    /// What the steward reports (`{ "items": [...] }`); nothing, unless a test says otherwise.
    pub steward: Value,
    /// Reads the fake steward claims to have made, from what it was given. Tests use it to record what the
    /// steward's brief and workspace offered.
    pub steward_reads: Option<StewardReads>,
    /// What the audience reports (`{ "rating": n, "findings": [...] }`).
    pub audience: Value,
    /// Everything the audience was given: system prompt, brief, and its workspace's files.
    pub seen_by_audience: Arc<Mutex<Vec<String>>>,
    /// What the auditor reports (`{ "summary": "...", "findings": [...] }`).
    pub auditor: Value,
    /// Everything the auditor was given: system prompt, brief, and its workspace's files.
    pub seen_by_auditor: Arc<Mutex<Vec<String>>>,
    /// What the source check reports, given the checker's workspace. None: it reads every file under
    /// `page-text/` and finds nothing wrong.
    pub source_check: Option<SourceJudge>,
    /// The planner's output for each call, in order; a call past the end gets the deterministic fake plan.
    pub plans: Vec<Value>,
    /// The critic's output for each call, in order; a call past the end gets the deterministic accepting fake.
    pub critiques: Vec<Value>,
    /// The brief each planner call was given, in order.
    pub seen_by_planner: Arc<Mutex<Vec<String>>>,
}

/// A fake source checker: from the checker's workspace to its report.
pub type SourceJudge = Arc<dyn Fn(&Path) -> Value + Send + Sync>;

/// The files under `workspace/page-text/`, as paths relative to it.
pub fn page_text_files(workspace: &Path) -> Vec<String> {
    let root = workspace.join("page-text");
    let mut out = Vec::new();
    let mut stack = vec![root.clone()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(rel) = path.strip_prefix(&root) {
                out.push(rel.display().to_string());
            }
        }
    }
    out.sort();
    out
}

/// The commands a fake steward reports having run, given its invocation.
pub type StewardReads = Arc<dyn Fn(&AgentInvocation) -> Vec<String> + Send + Sync>;

impl Default for Scenario {
    fn default() -> Self {
        Scenario {
            reading: default_reading(),
            edits: vec![(
                "README.md".into(),
                Some("# Demo\n\nHello, reader.\n".into()),
            )],
            report: producer_report(
                "Added a greeting line to README.md.",
                &["cargo test --locked"],
            ),
            report_when_sent_back: None,
            producer_commands: vec!["cargo test".into()],
            verdict: passing_verdict(2),
            verifier_commands: vec!["cargo test".into()],
            cost: 0.01,
            during_verification: None,
            seen_by_verifier: Arc::new(Mutex::new(Vec::new())),
            invocations: Arc::new(Mutex::new(Vec::new())),
            seen_by_producer: Arc::new(Mutex::new(Vec::new())),
            steward: json!({ "items": [] }),
            steward_reads: None,
            audience: json!({ "rating": 3, "findings": [] }),
            seen_by_audience: Arc::new(Mutex::new(Vec::new())),
            auditor: json!({ "summary": "The outcome serves the goals.", "findings": [] }),
            seen_by_auditor: Arc::new(Mutex::new(Vec::new())),
            source_check: None,
            plans: Vec::new(),
            critiques: Vec::new(),
            seen_by_planner: Arc::new(Mutex::new(Vec::new())),
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
                    self.seen_by_producer
                        .lock()
                        .unwrap()
                        .push(spec.user_prompt.clone());
                    let producer_runs = self
                        .invocations
                        .lock()
                        .unwrap()
                        .iter()
                        .filter(|r| **r == Role::Producer)
                        .count();
                    let report = match (&self.report_when_sent_back, producer_runs) {
                        (Some(second), n) if n > 1 => second.clone(),
                        _ => self.report.clone(),
                    };
                    // A report with an empty `criteria_check` reports every criterion of the scenario's
                    // reading as met; a test that wants otherwise lists the entries itself.
                    let mut report = report;
                    if report["criteria_check"]
                        .as_array()
                        .is_some_and(|a| a.is_empty())
                    {
                        report["criteria_check"] = Value::Array(
                            criteria_texts(&self.reading)
                                .iter()
                                .map(|c| criterion_check(c, "met"))
                                .collect(),
                        );
                    }
                    let mut t = FakeTurn::output(report);
                    t.edits = self.edits.clone();
                    t.commands_run = self.producer_commands.clone();
                    t
                }
                Role::Verifier if spec.user_prompt.starts_with("# Source check") => {
                    let report = match &self.source_check {
                        Some(judge) => judge(&spec.workdir),
                        None => json!({
                            "files_checked": page_text_files(&spec.workdir),
                            "problems": []
                        }),
                    };
                    FakeTurn::output(report)
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
                Role::Steward => {
                    let mut t = FakeTurn::output(self.steward.clone());
                    if let Some(reads) = &self.steward_reads {
                        t.commands_run = reads(spec);
                    }
                    t
                }
                Role::Audience => {
                    let mut seen = self.seen_by_audience.lock().unwrap();
                    seen.push(spec.system_prompt.clone());
                    seen.push(spec.user_prompt.clone());
                    seen.push(snapshot_text(&spec.workdir));
                    drop(seen);
                    FakeTurn::output(self.audience.clone())
                }
                // A scripted plan or critique for this call, or else the deterministic fake's.
                Role::Planner => {
                    self.seen_by_planner
                        .lock()
                        .unwrap()
                        .push(spec.user_prompt.clone());
                    let call = self
                        .invocations
                        .lock()
                        .unwrap()
                        .iter()
                        .filter(|r| **r == Role::Planner)
                        .count()
                        - 1;
                    match self.plans.get(call) {
                        Some(plan) => FakeTurn::output(plan.clone()),
                        None => athena_agent::fake::deterministic_turn(spec),
                    }
                }
                Role::Critic => {
                    let call = self
                        .invocations
                        .lock()
                        .unwrap()
                        .iter()
                        .filter(|r| **r == Role::Critic)
                        .count()
                        - 1;
                    match self.critiques.get(call) {
                        Some(critique) => FakeTurn::output(critique.clone()),
                        None => athena_agent::fake::deterministic_turn(spec),
                    }
                }
                Role::Auditor => {
                    let mut seen = self.seen_by_auditor.lock().unwrap();
                    seen.push(spec.system_prompt.clone());
                    seen.push(spec.user_prompt.clone());
                    seen.push(snapshot_text(&spec.workdir));
                    drop(seen);
                    FakeTurn::output(self.auditor.clone())
                }
            };
            turn.cost_usd = self.cost;
            turn.usage = usage;
            turn
        })
    }
}
