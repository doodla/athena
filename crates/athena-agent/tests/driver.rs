//! The Claude Code driver against a stand-in `claude` program (design §10.1: "a configurable agent
//! command also lets tests substitute a fake child process, to exercise timeouts, kills, and parent
//! death").

use athena_agent::claude::ClaudeCodeDriver;
use athena_agent::procs::Registry;
use athena_agent::{AgentDriver, AgentInvocation, DriverError, Usage, UsageSink};
use athena_data::records::{InvocationOutcome, Role};
use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

const STAND_IN: &str = r#"#!/bin/sh
if [ "$1" = "--version" ]; then echo "2.1.296 (Claude Code)"; exit 0; fi
cat > /dev/null
echo '{"type":"system","subtype":"init"}'
echo '{"type":"assistant","message":{"id":"msg_1","model":"claude-haiku-5-5","content":[{"type":"tool_use","id":"tu_1","name":"Bash","input":{"command":"cargo test"}}],"usage":{"input_tokens":3,"cache_creation_input_tokens":100,"cache_read_input_tokens":0,"output_tokens":7}}}'
if [ "$MODE" = "hang" ]; then sleep 60; exit 0; fi
echo '{"type":"result","subtype":"success","is_error":false,"num_turns":2,"duration_api_ms":900,"total_cost_usd":0.002,"usage":{"input_tokens":5,"cache_creation_input_tokens":120,"cache_read_input_tokens":40,"output_tokens":60},"modelUsage":{"claude-haiku-5-5":{"inputTokens":5,"outputTokens":60,"cacheReadInputTokens":40,"cacheCreationInputTokens":120,"costUSD":0.002}},"permission_denials":[],"structured_output":{"answer":42}}'
"#;

fn stand_in(dir: &Path) -> PathBuf {
    let path = dir.join("claude");
    std::fs::write(&path, STAND_IN).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn spec(dir: &Path, mode: &str, timeout: Duration) -> AgentInvocation {
    let env = BTreeMap::from([
        ("PATH".to_string(), std::env::var("PATH").unwrap()),
        ("MODE".to_string(), mode.to_string()),
    ]);
    AgentInvocation {
        job: "20261010-153012-k7q2".into(),
        seq: 1,
        role: Role::Reader,
        model: "claude-haiku-5-5".into(),
        system_prompt: "contract".into(),
        user_prompt: "brief".into(),
        tools: vec!["Read".into()],
        allowed_tools: vec![],
        permission_mode: "dontAsk".into(),
        workdir: dir.to_path_buf(),
        env,
        hook: None,
        max_budget_usd: 0.5,
        timeout,
        json_schema: serde_json::json!({ "type": "object" }),
        scratch: dir.join("agent"),
    }
}

#[derive(Default)]
struct Recorder(Vec<Usage>);

impl UsageSink for Recorder {
    fn update(&mut self, usage: &Usage, _: &[String]) {
        self.0.push(*usage);
    }
}

#[test]
fn a_completed_invocation_records_measured_usage_and_the_reported_cost() {
    // M14 (allows): completed invocations record measured usage.
    let dir = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let driver =
        ClaudeCodeDriver::new(stand_in(dir.path()), "2.1.296", Registry::new(state.path()))
            .unwrap();
    let mut sink = Recorder::default();
    let out = driver
        .invoke(
            &spec(dir.path(), "complete", Duration::from_secs(30)),
            &mut sink,
        )
        .unwrap();
    assert_eq!(out.outcome, InvocationOutcome::Success);
    assert!(out.usage_measured);
    assert_eq!(
        out.usage,
        Usage {
            input: 5,
            output: 60,
            cache_read: 40,
            cache_write: 120
        }
    );
    assert_eq!(out.cost_usd, Some(0.002));
    assert_eq!(out.structured.unwrap()["answer"], 42);
    assert_eq!(out.commands_run, vec!["cargo test".to_string()]);
    assert!(!sink.0.is_empty(), "usage was persisted as it accrued");
}

#[test]
fn a_timed_out_invocation_is_killed_with_a_lower_bound() {
    // M13 (blocks): a timeout kills the group. M14: the record keeps the de-duplicated partial usage.
    let dir = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let registry = Registry::new(state.path());
    let driver = ClaudeCodeDriver::new(stand_in(dir.path()), "2.1.296", registry.clone()).unwrap();
    let started = std::time::Instant::now();
    let out = driver
        .invoke(
            &spec(dir.path(), "hang", Duration::from_secs(2)),
            &mut Recorder::default(),
        )
        .unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(20),
        "the kill did not wait for the agent"
    );
    assert_eq!(out.outcome, InvocationOutcome::Killed);
    assert!(!out.usage_measured);
    assert_eq!(
        out.usage,
        Usage {
            input: 3,
            output: 7,
            cache_read: 0,
            cache_write: 100
        }
    );
    assert_eq!(out.cost_usd, None);
    assert!(registry.entries().is_empty(), "the group left the registry");
}

#[test]
fn the_kill_switch_stops_invocations_before_they_start() {
    // M13 (blocks): a stop file blocks new invocations.
    let dir = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let registry = Registry::new(state.path());
    let driver = ClaudeCodeDriver::new(stand_in(dir.path()), "2.1.296", registry.clone()).unwrap();
    std::fs::write(registry.stop_file(), "").unwrap();
    let result = driver.invoke(
        &spec(dir.path(), "complete", Duration::from_secs(30)),
        &mut Recorder::default(),
    );
    assert!(matches!(result, Err(DriverError::Stopped)));
}

#[test]
fn a_host_that_is_not_the_pinned_version_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let state = tempfile::tempdir().unwrap();
    let result = ClaudeCodeDriver::new(stand_in(dir.path()), "9.9.9", Registry::new(state.path()));
    assert!(matches!(result, Err(DriverError::Spawn(_))));
}
