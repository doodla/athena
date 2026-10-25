//! A driver that makes no model calls (design §10.1, §7.6).
//!
//! Tests script it per invocation: its structured output, the edits it makes in the snapshot, the commands
//! it claims to have run, and its usage. Nested runs use [`FakeAgentDriver::deterministic`], which answers
//! every role with a small, valid, request-derived output, so a nested `athena ask` or `go` exercises the
//! real command-line behavior at no model cost.

use crate::procs::Registry;
use crate::{AgentDriver, AgentInvocation, AgentOutput, DriverError, Usage, UsageSink};
use athena_data::records::{DriverInfo, InvocationOutcome, ModelUsage, Role};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// What a scripted invocation does.
#[derive(Debug, Clone, PartialEq)]
pub struct FakeTurn {
    pub outcome: InvocationOutcome,
    pub structured: Option<Value>,
    /// Files to write (`Some`) or delete (`None`) in the snapshot, by relative path.
    pub edits: Vec<(String, Option<String>)>,
    pub commands_run: Vec<String>,
    pub cost_usd: f64,
    pub usage: Usage,
    pub detail: Option<String>,
}

impl FakeTurn {
    /// A successful turn with this structured output and nothing else.
    pub fn output(structured: Value) -> FakeTurn {
        FakeTurn {
            outcome: InvocationOutcome::Success,
            structured: Some(structured),
            edits: Vec::new(),
            commands_run: Vec::new(),
            cost_usd: 0.0,
            usage: Usage::default(),
            detail: None,
        }
    }
}

/// A script: the turn to play for each invocation.
pub type Script = Box<dyn Fn(&AgentInvocation) -> FakeTurn + Send + Sync>;

/// The fake driver.
pub struct FakeAgentDriver {
    script: Script,
    registry: Option<Registry>,
}

impl std::fmt::Debug for FakeAgentDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FakeAgentDriver")
            .field("registry", &self.registry)
            .finish_non_exhaustive()
    }
}

impl FakeAgentDriver {
    pub fn new(script: Script) -> Self {
        FakeAgentDriver {
            script,
            registry: None,
        }
    }

    /// Honors the kill switch of this registry, as the real driver does.
    pub fn with_registry(mut self, registry: Registry) -> Self {
        self.registry = Some(registry);
        self
    }

    /// The deterministic fake used by nested runs.
    pub fn deterministic() -> Self {
        FakeAgentDriver::new(Box::new(deterministic_turn))
    }
}

impl AgentDriver for FakeAgentDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            name: "fake".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }

    fn invoke(
        &self,
        spec: &AgentInvocation,
        sink: &mut dyn UsageSink,
    ) -> Result<AgentOutput, DriverError> {
        if self.registry.as_ref().is_some_and(Registry::stopped) {
            return Err(DriverError::Stopped);
        }
        let started_at = jiff::Timestamp::now();
        let clock = std::time::Instant::now();
        let turn = (self.script)(spec);
        for (path, content) in &turn.edits {
            let full = spec.workdir.join(path);
            match content {
                Some(text) => {
                    if let Some(parent) = full.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&full, text)?;
                }
                None => {
                    let _ = std::fs::remove_file(&full);
                }
            }
        }
        sink.update(&turn.usage, &turn.commands_run);
        let completed = turn.outcome != InvocationOutcome::Killed;
        let u = turn.usage;
        Ok(AgentOutput {
            outcome: turn.outcome,
            detail: turn.detail.clone(),
            structured: turn.structured.clone(),
            started_at,
            ended_at: jiff::Timestamp::now(),
            wall_s: clock.elapsed().as_secs_f64(),
            usage: u,
            usage_measured: completed,
            models_used: if completed {
                BTreeMap::from([(
                    spec.model.clone(),
                    ModelUsage {
                        input: u.input,
                        output: u.output,
                        cache_read: u.cache_read,
                        cache_write: u.cache_write,
                        cost_usd: turn.cost_usd,
                    },
                )])
            } else {
                BTreeMap::new()
            },
            cost_usd: completed.then_some(turn.cost_usd),
            turns: Some(1),
            duration_api_ms: Some(0),
            commands_run: turn.commands_run.clone(),
            denials: Vec::new(),
        })
    }
}

/// The text under a Markdown heading, with block-quote markers removed.
fn section(text: &str, heading: &str) -> String {
    let mut lines = text.lines().skip_while(|l| !l.starts_with(heading)).skip(1);
    let mut out = Vec::new();
    for line in lines.by_ref() {
        if line.starts_with('#') {
            break;
        }
        out.push(
            line.strip_prefix("> ")
                .or_else(|| line.strip_prefix('>'))
                .unwrap_or(line),
        );
    }
    out.join("\n").trim().to_string()
}

fn first_enum(schema: &Value, field: &str, prefer: &str) -> Value {
    let options = schema
        .pointer(&format!("/properties/{field}/enum"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    options
        .iter()
        .find(|v| v.as_str() == Some(prefer))
        .or_else(|| options.iter().find(|v| !v.is_null()))
        .cloned()
        .unwrap_or(Value::Null)
}

/// Words that make the fake reader mark a request urgent: it speeds Athena up or cuts wasted time.
pub const SPEED_WORDS: [&str; 6] = [
    "speed up",
    "faster",
    "wasted time",
    "timeouts",
    "thrash",
    "duplicated checks",
];

/// The file the deterministic producer writes.
pub const FAKE_DELIVERABLE: &str = "FAKE-CHANGE.md";

/// The deterministic turn for each role.
pub fn deterministic_turn(spec: &AgentInvocation) -> FakeTurn {
    match spec.role {
        Role::Reader => {
            let request = section(&spec.user_prompt, "# Request");
            let short: String = request
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(50)
                .collect();
            // A request that asks to make Athena faster or to cut wasted time is marked urgent, with a reason.
            let lowered = request.to_lowercase();
            let urgent_reason = SPEED_WORDS
                .iter()
                .find(|w| lowered.contains(*w))
                .map(|w| format!("A fake reader: the request mentions \"{w}\", so it makes Athena faster or cuts wasted time."));
            FakeTurn::output(json!({
                "urgent_reason": urgent_reason,
                "title": format!("Record the request in {FAKE_DELIVERABLE}"),
                "summary": format!("A fake reading made without a model call: write the request \"{short}\" to {FAKE_DELIVERABLE}."),
                "class_of_work": first_enum(&spec.json_schema, "class_of_work", "other"),
                "capability": first_enum(&spec.json_schema, "capability", "change"),
                "decline_reason": null,
                "producer_model": first_enum(&spec.json_schema, "producer_model", ""),
                "defaults_applied": [],
                "assumptions": ["This is a nested run with fake agents."],
                "question": null,
                "acceptance_criteria": [{ "text": format!("{FAKE_DELIVERABLE} contains the request"), "kind": "content" }],
                "deliverables": [{ "path": FAKE_DELIVERABLE, "action": "create", "p": 0.9 }],
                "out_of_scope": [],
                "forecast": {
                    "p_reading_accepted": 0.9,
                    "p_verification_passes": 0.9,
                    "p_user_intervenes": 0.1,
                    "cost_usd": { "p10": 0.001, "p50": 0.001, "p90": 0.002 },
                    "wall_time_s": { "p10": 1.0, "p50": 5.0, "p90": 60.0 },
                    "departures": "a fake reader: no model call"
                }
            }))
        }
        Role::Producer => {
            let request = section(&spec.user_prompt, "## Request");
            let criteria_check: Vec<Value> = section(&spec.user_prompt, "## Acceptance criteria")
                .lines()
                .filter_map(|l| {
                    l.split_once(". ")
                        .filter(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
                        .map(|(_, rest)| rest)
                })
                // The brief gives "[kind] text"; the report names the text alone.
                .map(|rest| rest.split_once("] ").map_or(rest, |(_, text)| text))
                .map(|text| json!({ "criterion": text, "status": "met", "evidence": "fake producer: nothing was checked" }))
                .collect();
            let mut turn = FakeTurn::output(json!({
                "summary": format!("Wrote the request to {FAKE_DELIVERABLE} (fake producer, no model call)."),
                "files": [FAKE_DELIVERABLE],
                "checks_run": [],
                "criteria_check": criteria_check,
                "open_issues": [],
                "mistakes": [],
                "learnings": [],
                "desires": []
            }));
            turn.edits.push((
                FAKE_DELIVERABLE.to_string(),
                Some(format!("# Fake change\n\n{request}\n")),
            ));
            turn
        }
        Role::Verifier => {
            let criteria = section(&spec.user_prompt, "## Acceptance criteria");
            let entries: Vec<Value> = criteria
                .lines()
                .filter_map(|l| l.split_once(". ").filter(|(n, _)| n.chars().all(|c| c.is_ascii_digit())).map(|(_, rest)| rest))
                .map(|text| json!({ "criterion": text, "met": true, "evidence": "fake verifier: no model call", "commands": [] }))
                .collect();
            FakeTurn::output(json!({
                "verdict": "pass",
                "criteria": entries,
                "findings": [],
                "request_reading_mismatch": null,
                "summary": "A fake verdict made without a model call."
            }))
        }
        // A plan with one demonstration for each numbered criterion in the brief, and one file to create.
        Role::Planner => {
            let demonstrations: Vec<Value> = section(&spec.user_prompt, "## Acceptance criteria")
                .lines()
                .filter_map(|l| {
                    l.split_once(". ")
                        .filter(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
                        .map(|(n, rest)| (n.parse::<u32>().unwrap_or(0), rest))
                })
                .map(|(n, rest)| (n, rest.split_once("] ").map_or(rest, |(_, text)| text)))
                .map(|(n, text)| {
                    json!({
                        "criterion": n,
                        "method": "read the change",
                        "command": "cargo test --locked",
                        "observes": format!("fake planner: {text}"),
                        "fails_without": "fake planner: nothing was checked"
                    })
                })
                .collect();
            FakeTurn::output(json!({
                "approach": format!("Write the request to {FAKE_DELIVERABLE} (fake planner, no model call)."),
                "decisions": [],
                "files": [{ "path": FAKE_DELIVERABLE, "action": "create", "purpose": "Hold the request." }],
                "steps": [format!("Create {FAKE_DELIVERABLE}.")],
                "risks": [],
                "demonstrations": demonstrations,
                "open_questions": []
            }))
        }
        // A critic that finds nothing wrong.
        Role::Critic => FakeTurn::output(json!({
            "verdict": "accept",
            "findings": [],
            "request_reading_mismatch": null,
            "summary": "A fake critique made without a model call."
        })),
        // A nested run has nothing for the steward to report.
        Role::Steward => FakeTurn::output(json!({ "items": [] })),
        // A nested run rates nothing: a middling rating with no findings raises no work item.
        Role::Audience => FakeTurn::output(json!({ "rating": 3, "findings": [] })),
        // A nested run audits nothing: a summary with no findings.
        Role::Auditor => FakeTurn::output(
            json!({ "summary": "A fake audit made without a model call.", "findings": [] }),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NoSink;
    use std::path::PathBuf;
    use std::time::Duration;

    fn spec(role: Role, prompt: &str, workdir: PathBuf) -> AgentInvocation {
        AgentInvocation {
            job: "j".into(),
            seq: 1,
            role,
            model: "claude-haiku-5-5".into(),
            system_prompt: String::new(),
            user_prompt: prompt.into(),
            tools: vec![],
            allowed_tools: vec![],
            permission_mode: "dontAsk".into(),
            workdir,
            env: BTreeMap::new(),
            hook: None,
            max_budget_usd: 1.0,
            timeout: Duration::from_secs(10),
            json_schema: json!({ "properties": { "class_of_work": { "enum": ["docs", "other"] }, "capability": { "enum": ["change", null] }, "producer_model": { "enum": ["claude-haiku-5-5"] } } }),
            scratch: PathBuf::from("/nonexistent"),
        }
    }

    #[test]
    fn the_deterministic_fake_answers_every_role() {
        let dir = tempfile::tempdir().unwrap();
        let driver = FakeAgentDriver::deterministic();
        let reading = driver
            .invoke(
                &spec(
                    Role::Reader,
                    "# Request (verbatim)\n\n> add a note\n",
                    dir.path().into(),
                ),
                &mut NoSink,
            )
            .unwrap();
        let structured = reading.structured.unwrap();
        assert_eq!(structured["class_of_work"], "other");
        assert_eq!(structured["capability"], "change");
        let produced = driver
            .invoke(
                &spec(
                    Role::Producer,
                    "## Request (verbatim)\n\n> add a note\n\n## Reading",
                    dir.path().into(),
                ),
                &mut NoSink,
            )
            .unwrap();
        assert_eq!(produced.cost_usd, Some(0.0));
        assert!(
            std::fs::read_to_string(dir.path().join(FAKE_DELIVERABLE))
                .unwrap()
                .contains("add a note")
        );
        let verdict = driver
            .invoke(&spec(Role::Verifier, "## Acceptance criteria\n\n1. [content] it exists\n2. [behavior] it runs\n\n## Next", dir.path().into()), &mut NoSink)
            .unwrap();
        assert_eq!(
            verdict.structured.unwrap()["criteria"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        let steward = driver
            .invoke(&spec(Role::Steward, "", dir.path().into()), &mut NoSink)
            .unwrap();
        assert_eq!(steward.structured.unwrap()["items"], json!([]));
        assert_eq!(steward.cost_usd, Some(0.0), "no model call, no cost");
    }

    #[test]
    fn the_fake_planner_covers_every_criterion_and_the_critic_accepts() {
        let dir = tempfile::tempdir().unwrap();
        let driver = FakeAgentDriver::deterministic();
        let brief = "## Acceptance criteria\n\n1. [content] it exists\n2. [behavior] it runs\n3. [content] it is documented\n\n## Next";
        let plan = driver
            .invoke(&spec(Role::Planner, brief, dir.path().into()), &mut NoSink)
            .unwrap()
            .structured
            .unwrap();
        let numbers: Vec<u64> = plan["demonstrations"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d["criterion"].as_u64().unwrap())
            .collect();
        assert_eq!(numbers, [1, 2, 3]);
        let critique = driver
            .invoke(&spec(Role::Critic, "", dir.path().into()), &mut NoSink)
            .unwrap()
            .structured
            .unwrap();
        assert_eq!(critique["verdict"], "accept");
        assert_eq!(critique["findings"], json!([]));
    }

    #[test]
    fn the_fake_honors_the_kill_switch() {
        // M13 (blocks): a stop file blocks new invocations.
        let state = tempfile::tempdir().unwrap();
        let registry = Registry::new(state.path());
        std::fs::write(registry.stop_file(), "").unwrap();
        let driver = FakeAgentDriver::deterministic().with_registry(registry);
        let result = driver.invoke(&spec(Role::Reader, "", state.path().into()), &mut NoSink);
        assert!(matches!(result, Err(DriverError::Stopped)));
    }
}
