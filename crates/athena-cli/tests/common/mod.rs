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

/// `config` without its `[job_plan]` table, wherever it sits: the lines from its header to the next table
/// header. Tests of the unplanned path use it, and tests that append their own `[job_plan]` need it to avoid
/// a duplicate table.
pub fn without_job_plan(config: &str) -> String {
    let mut skipping = false;
    config
        .split_inclusive('\n')
        .filter(|line| {
            let head = line.trim_start();
            if head.starts_with('[') {
                skipping = head.trim_end() == "[job_plan]";
            }
            !skipping
        })
        .collect()
}

/// The repository's own configuration, without job planning, and with checks that are fast and observable:
/// - `fmt` fails when `src/lib.rs` contains `BADLY`, unless `rustfmt.toml` says `relaxed`;
/// - `test` fails without `README.md` or with a `.cargo/config.toml` in the tree, and otherwise reports
///   one test.
pub fn test_config() -> String {
    // The repository's own configuration names its cutover job, after which Athena writes no trailers
    // (D111). These tests exercise the trailers, so the demo repository starts before the cutover; the
    // cutover fixture and `tests/trailer_stop.rs` cover the other side.
    let base = &athena_core::test_support::with_cutover_job(
        &without_job_plan(include_str!("../../../../athena.toml")),
        None,
    );
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
    // The disk guard compares against the machine's real free space. Tests must not depend on that, so they
    // run with no minimum; tests of the guard itself set one with `with_min_free_gb`.
    // It is the last table, so replacing the checks block (config_with_change_checks) keeps it.
    let disk = "\n[disk]\nmin_free_gb = 0\n";
    format!("{}{}{}{}", &base[..start], checks, &base[end..], disk)
}

/// `config` with `[disk] min_free_gb` set to `gb` (whatever it was).
pub fn with_min_free_gb(config: &str, gb: u64) -> String {
    let from = "min_free_gb = 0\n";
    assert!(
        config.contains(from),
        "the configuration has a disk minimum"
    );
    config.replacen(from, &format!("min_free_gb = {gb}\n"), 1)
}

/// [`test_config`] with the change capability's checks replaced by `checks`: each is a name and the
/// command line to run, as a program and its arguments.
pub fn config_with_change_checks(checks: &[(&str, &[&str])]) -> String {
    let base = test_config();
    let start = base
        .find("[[capabilities.change.checks]]")
        .expect("the config has checks");
    let end = base.find("[classes]").expect("the config has classes");
    let mut block = String::new();
    for (name, run) in checks {
        let argv: Vec<String> = run.iter().map(|a| format!("{a:?}")).collect();
        block.push_str(&format!(
            "[[capabilities.change.checks]]\nname = {name:?}\nrun = [{}]\n\n",
            argv.join(", ")
        ));
    }
    format!("{}{}{}", &base[..start], block, &base[end..])
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
        let state = StateDir::new(self.state.path().join("repo"))
            .with_free_space(athena_core::test_support::plenty_of_space);
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

/// The files of a demo repository's bootstrap commit under `config`: the same tree [`Fixture::with_config`]
/// commits.
pub fn demo_files(config: &str) -> Vec<(String, String)> {
    [
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
    ]
    .into_iter()
    .map(|(path, content)| (path.to_string(), content.to_string()))
    .collect()
}

/// The records-history cutover fixture (decision D111), shared with `athena-core`'s tests: a scratch
/// repository with a bare remote, a bootstrap commit, one job landed the old way, and the cutover job landed
/// through the pull-request path with its landing record, import and legacy-exempt record on
/// `athena/records`. Built only through Athena's own writers; see `athena_core::test_support` for the shape
/// and for how a later job (the trailer job, job 7 part 2) reuses it.
///
/// Building runs two whole jobs against the fake agents, so a test that only needs the history should build
/// it once and share it.
pub fn cutover_fixture() -> athena_core::test_support::CutoverFixture {
    use athena_core::test_support::{CutoverFixture, CutoverSpec, Step};
    let capabilities = athena_cli::wiring::capabilities;
    let scripts = |step: Step<'_>| -> Script {
        match step {
            Step::OldWay => Scenario::default().script(),
            Step::CutoverRead | Step::CutoverGo { .. } => {
                let mut scenario = Scenario {
                    reading: reading(
                        &[("README.md", "modify"), ("athena.toml", "modify")],
                        &[
                            (
                                "content",
                                "athena.toml names this job as the cutover job, and README.md says so",
                            ),
                            ("behavior", "the tests still pass"),
                        ],
                    ),
                    ..Scenario::default()
                };
                if let Step::CutoverGo { athena_toml } = step {
                    scenario.edits = vec![
                        (
                            "README.md".into(),
                            Some("# Demo\n\nHello, reader.\n\nThis is the cutover.\n".into()),
                        ),
                        ("athena.toml".into(), Some(athena_toml.to_string())),
                    ];
                }
                scenario.script()
            }
        }
    };
    CutoverFixture::build(CutoverSpec {
        files: demo_files(&test_config()),
        capabilities: &capabilities,
        scripts: &scripts,
    })
    .expect("the cutover fixture builds")
}

/// Runs the real `athena check <flag>` in the fixture's work repository, with a scratch state directory and
/// no `CI` variable. `--attribution` also gets `--no-checkpoint`, so the records branch is left as built.
pub fn athena_check(
    fixture: &athena_core::test_support::CutoverFixture,
    flag: &str,
) -> std::process::Output {
    let scratch = tempfile::tempdir().unwrap();
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_athena"));
    cmd.current_dir(fixture.root())
        .arg("check")
        .arg(flag)
        .env("ATHENA_STATE_DIR", scratch.path())
        .env_remove("CI")
        .env_remove("ATHENA_NESTED")
        .env_remove("ATHENA_SCRATCH_DIR");
    if flag == "--attribution" {
        cmd.arg("--no-checkpoint");
    }
    cmd.output().unwrap()
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
        "urgent_reason": null,
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

/// A scripted reader's reading, made to name each earlier job its brief carries a result for. A reading
/// for a retry or follow-up that does not is refused, so a reader that returns one fixed reading uses this.
pub fn naming_earlier_jobs(mut reading: Value, brief: &str) -> Value {
    let named: String = brief
        .lines()
        .filter(|l| l.contains(": end state "))
        .filter_map(|l| l.strip_prefix("Job ")?.split(':').next())
        .map(|id| format!(" It addresses the earlier result of {id}."))
        .collect();
    if !named.is_empty() {
        let summary = reading["summary"].as_str().unwrap_or_default();
        reading["summary"] = Value::String(format!("{summary}{named}"));
    }
    reading
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

/// The paths a producer prompt lists with an action, each once: the expected deliverables
/// (`- path (modify)`) and the accepted plan's files (`- path (modify, exists): purpose`).
fn listed_paths(prompt: &str) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    for line in prompt.lines() {
        let Some((path, rest)) = line.strip_prefix("- ").and_then(|l| l.split_once(" (")) else {
            continue;
        };
        let action = rest.split([',', ')']).next().unwrap_or("");
        if ["create", "modify", "delete"].contains(&action)
            && !path.contains(' ')
            && !paths.iter().any(|p| p == path)
        {
            paths.push(path.to_string());
        }
    }
    paths
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
                Role::Reader => {
                    FakeTurn::output(naming_earlier_jobs(self.reading.clone(), &spec.user_prompt))
                }
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
                    // A report that lists no `unchanged_planned_files` explains every file the brief or the
                    // plan lists and the scenario's edits do not touch (a change reading also lists
                    // CHANGELOG.md); a test that wants the gap itself sets the field.
                    if report.get("unchanged_planned_files").is_none() {
                        let untouched: Vec<Value> = listed_paths(&spec.user_prompt)
                            .into_iter()
                            .filter(|path| !self.edits.iter().any(|(edited, _)| edited == path))
                            .map(|path| json!({ "path": path, "reason": "the scenario's producer does not edit it" }))
                            .collect();
                        report["unchanged_planned_files"] = Value::Array(untouched);
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
