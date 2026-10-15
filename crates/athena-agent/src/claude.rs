//! The Claude Code driver (design §10.2).
//!
//! For each invocation it writes the composed system prompt and the hook settings to private files,
//! spawns `claude -p` in a new process group tied to its parent's life, with the hermetic flags and an
//! explicit environment, writes the brief to the child's standard input (so prompts never appear in
//! argument lists), and reads stream-json events as they arrive. It refuses to start inside a nested run.

use crate::procs::{GRACE, Registry, isolate, kill_group};
use crate::stream::{ResultEvent, StreamState};
use crate::{
    AgentDriver, AgentInvocation, AgentOutput, DriverError, UsageSink, nested_marker, refuse_nested,
};
use athena_data::records::{DriverInfo, InvocationOutcome};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// The flags that make an invocation hermetic. A command line missing any of them is refused (M10).
pub const HERMETIC_FLAGS: [&str; 4] = [
    "--restricted",
    "--strict-mcp-config",
    "--disable-slash-commands",
    "--no-session-persistence",
];

/// The Claude Code driver.
#[derive(Debug, Clone)]
pub struct ClaudeCodeDriver {
    bin: PathBuf,
    version: String,
    registry: Registry,
    /// Always true outside this crate's own tests, which run a stand-in program instead of Claude Code.
    refuse_nested_runs: bool,
}

/// Retries an operation that executes a program while the program file is busy (ETXTBSY): a file that
/// was just written can stay open for writing in a child that another thread forked, until it execs.
fn retry_busy<T>(mut op: impl FnMut() -> std::io::Result<T>) -> std::io::Result<T> {
    let mut attempt = 0;
    loop {
        match op() {
            Err(e) if e.raw_os_error() == Some(libc::ETXTBSY) && attempt < 20 => {
                attempt += 1;
                std::thread::sleep(Duration::from_millis(50));
            }
            other => return other,
        }
    }
}

/// Checks that the binary at `bin` reports the pinned version, and returns it.
pub fn check_pinned(bin: &Path, pinned: &str) -> Result<String, DriverError> {
    let version = claude_version(bin).ok_or_else(|| {
        DriverError::Spawn(format!(
            "{} --version failed; run ./setup.sh",
            bin.display()
        ))
    })?;
    if version != pinned {
        return Err(DriverError::Spawn(format!(
            "Claude Code {version} at {} is not the pinned {pinned}; run ./setup.sh",
            bin.display()
        )));
    }
    Ok(version)
}

/// Reads the version a Claude Code binary reports, such as `2.1.296`.
pub fn claude_version(bin: &Path) -> Option<String> {
    let mut cmd = Command::new(bin);
    cmd.arg("--version")
        .env_clear()
        .stdin(Stdio::null())
        .stderr(Stdio::null());
    for key in ["HOME", "PATH"] {
        if let Some(value) = std::env::var_os(key) {
            cmd.env(key, value);
        }
    }
    let out = retry_busy(|| cmd.output()).ok()?;
    out.status.success().then(|| {
        String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .next()
            .unwrap_or("")
            .to_string()
    })
}

impl ClaudeCodeDriver {
    /// A driver for the binary at `bin`, which must report the pinned `version`.
    pub fn new(bin: PathBuf, pinned: &str, registry: Registry) -> Result<Self, DriverError> {
        refuse_nested(nested_marker())?;
        let version = check_pinned(&bin, pinned)?;
        Ok(ClaudeCodeDriver {
            refuse_nested_runs: true,
            bin,
            version,
            registry,
        })
    }

    /// The full argument list for an invocation (after the binary).
    pub fn command_line(
        spec: &AgentInvocation,
        system_prompt_file: &Path,
        settings_file: &Path,
    ) -> Vec<OsString> {
        let mut args: Vec<OsString> = vec![
            "-p".into(),
            "--system-prompt-file".into(),
            system_prompt_file.into(),
        ];
        args.extend(["--restricted", "--strict-mcp-config"].map(OsString::from));
        args.push("--tools".into());
        args.push(spec.tools.join(",").into());
        args.extend(["--disable-slash-commands", "--no-session-persistence"].map(OsString::from));
        args.push("--permission-mode".into());
        args.push(spec.permission_mode.clone().into());
        args.push("--permission-prompts".into());
        args.push("none".into());
        if !spec.allowed_tools.is_empty() {
            args.push("--allowedTools".into());
            args.extend(spec.allowed_tools.iter().map(OsString::from));
        }
        args.push("--settings".into());
        args.push(settings_file.into());
        args.push("--max-budget-usd".into());
        args.push(format!("{:.2}", spec.max_budget_usd).into());
        args.extend(["--output-format", "stream-json", "--verbose"].map(OsString::from));
        args.push("--json-schema".into());
        args.push(spec.json_schema.to_string().into());
        args.push("--model".into());
        args.push(spec.model.clone().into());
        args
    }

    /// The settings that register Athena's hook for every tool call.
    pub fn settings(spec: &AgentInvocation) -> serde_json::Value {
        match &spec.hook {
            Some(hook) => {
                let command = format!(
                    "{} hook pre-tool-use --policy {}",
                    shell_quote(&hook.athena_bin.display().to_string()),
                    shell_quote(&hook.policy_file.display().to_string())
                );
                serde_json::json!({
                    "hooks": { "PreToolUse": [ { "matcher": "*", "hooks": [ { "type": "command", "command": command } ] } ] }
                })
            }
            None => serde_json::json!({}),
        }
    }
}

/// Checks that an argument list carries every hermetic flag (M10).
pub fn check_hermetic(args: &[OsString]) -> Result<(), String> {
    for flag in HERMETIC_FLAGS.iter().chain(
        [
            "--system-prompt-file",
            "--tools",
            "--permission-prompts",
            "--settings",
            "--max-budget-usd",
        ]
        .iter(),
    ) {
        if !args.iter().any(|a| a == *flag) {
            return Err(format!("the command line lacks {flag}"));
        }
    }
    Ok(())
}

/// The longest project folder name Claude Code uses before it shortens the name and adds a hash.
const PROJECT_NAME_MAX: usize = 200;

/// Claude Code's name for a working directory's project folder: every character that is not an ASCII
/// letter or digit becomes `-`.
pub fn project_dir_name(workdir: &Path) -> String {
    workdir
        .to_string_lossy()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// Removes what Claude Code leaves under its configuration directory after a run, even with
/// `--no-session-persistence`: the project folder named after the working directory, where it spills large
/// tool results, and the run's empty session folder. Agents work only in per-job snapshots, so both belong
/// to this invocation alone (acceptance verification 1, m4).
pub fn remove_host_traces(env: &BTreeMap<String, String>, workdir: &Path, session: Option<&str>) {
    let config = match env.get("CLAUDE_CONFIG_DIR").filter(|v| !v.is_empty()) {
        Some(dir) => PathBuf::from(dir),
        None => match env.get("HOME").filter(|v| !v.is_empty()) {
            Some(home) => Path::new(home).join(".claude"),
            None => return,
        },
    };
    let name = project_dir_name(workdir);
    let projects = config.join("projects");
    if name.len() <= PROJECT_NAME_MAX {
        let _ = std::fs::remove_dir_all(projects.join(&name));
    } else if let Ok(entries) = std::fs::read_dir(&projects) {
        let prefix = &name[..PROJECT_NAME_MAX];
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().starts_with(prefix) {
                let _ = std::fs::remove_dir_all(entry.path());
            }
        }
    }
    if let Some(id) =
        session.filter(|id| !id.is_empty() && id.chars().all(|c| c.is_ascii_hexdigit() || c == '-'))
    {
        // `remove_dir` removes only an empty folder.
        let _ = std::fs::remove_dir(config.join("session-env").join(id));
    }
}

fn shell_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)
}

/// The output of an invocation that ended with a result event. Its tokens are the requested model's
/// `modelUsage` tokens (see [`ResultEvent::requested_usage`]), the quantity the gate reconciles.
pub fn output_from_result(
    result: &ResultEvent,
    model: &str,
    started_at: jiff::Timestamp,
    ended_at: jiff::Timestamp,
    wall_s: f64,
    commands_run: Vec<String>,
) -> AgentOutput {
    let ok = !result.is_error && result.subtype == "success";
    AgentOutput {
        outcome: if ok {
            InvocationOutcome::Success
        } else {
            InvocationOutcome::Error
        },
        detail: (!ok).then(|| {
            let text: String = result
                .text
                .clone()
                .unwrap_or_default()
                .chars()
                .take(300)
                .collect();
            format!(
                "{}: {}",
                if result.subtype.is_empty() {
                    "error"
                } else {
                    &result.subtype
                },
                text
            )
        }),
        structured: result.structured.clone(),
        started_at,
        ended_at,
        wall_s,
        usage: result.requested_usage(model),
        usage_measured: true,
        models_used: result.models_used.clone(),
        cost_usd: result.cost_usd,
        turns: result.turns,
        duration_api_ms: result.duration_api_ms,
        commands_run,
        denials: result.denials.clone(),
    }
}

impl AgentDriver for ClaudeCodeDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            name: "claude-code".into(),
            version: self.version.clone(),
        }
    }

    fn invoke(
        &self,
        spec: &AgentInvocation,
        sink: &mut dyn UsageSink,
    ) -> Result<AgentOutput, DriverError> {
        if self.refuse_nested_runs {
            refuse_nested(nested_marker())?;
        }
        if self.registry.stopped() {
            return Err(DriverError::Stopped);
        }
        std::fs::create_dir_all(&spec.scratch)?;
        let system_file = spec.scratch.join(format!("system-{:02}.md", spec.seq));
        let settings_file = spec.scratch.join(format!("settings-{:02}.json", spec.seq));
        let stderr_file = spec.scratch.join(format!("stderr-{:02}.log", spec.seq));
        write_private(&system_file, spec.system_prompt.as_bytes())?;
        write_private(&settings_file, Self::settings(spec).to_string().as_bytes())?;
        let args = Self::command_line(spec, &system_file, &settings_file);
        check_hermetic(&args).map_err(DriverError::Spawn)?;

        let mut cmd = Command::new(&self.bin);
        cmd.args(&args)
            .env_clear()
            .envs(&spec.env)
            .current_dir(&spec.workdir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::from(std::fs::File::create(&stderr_file)?));
        isolate(&mut cmd);
        let started_at = jiff::Timestamp::now();
        let clock = Instant::now();
        let mut child = retry_busy(|| cmd.spawn())
            .map_err(|e| DriverError::Spawn(format!("{}: {e}", self.bin.display())))?;
        let pgid = child.id() as i32;
        self.registry
            .register(pgid, &spec.job, spec.role.as_str())?;

        if let Some(mut stdin) = child.stdin.take() {
            // A child that exits early closes the pipe; its output says why.
            let _ = stdin.write_all(spec.user_prompt.as_bytes());
        }
        let stdout = child.stdout.take().expect("stdout was piped");
        let (tx, rx) = mpsc::channel::<String>();
        let reader = std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                match line {
                    Ok(line) => {
                        if tx.send(line).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        let deadline = clock + spec.timeout;
        let mut state = StreamState::default();
        let mut killed: Option<String> = None;
        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(line) => {
                    state.on_line(&line);
                    sink.update(&state.partial_usage(), &state.commands_run());
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
            if killed.is_none() {
                if Instant::now() >= deadline {
                    killed = Some(format!("deadline of {}s reached", spec.timeout.as_secs()));
                } else if self.registry.stopped() {
                    killed = Some("the kill switch was turned on".into());
                }
                if killed.is_some() {
                    kill_group(pgid, GRACE);
                }
            }
        }
        let _ = reader.join();
        let _ = child.wait();
        self.registry.unregister(pgid);
        remove_host_traces(&spec.env, &spec.workdir, state.session_id.as_deref());
        let ended_at = jiff::Timestamp::now();
        let wall_s = clock.elapsed().as_secs_f64();
        let stderr_tail = std::fs::read_to_string(&stderr_file)
            .map(|t| {
                t.lines()
                    .rev()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or("")
                    .chars()
                    .take(300)
                    .collect::<String>()
            })
            .unwrap_or_default();

        let commands_run = state.commands_run();
        let partial = state.partial_usage();
        let output = match (state.result.clone(), killed) {
            (Some(result), _) => output_from_result(
                &result,
                &spec.model,
                started_at,
                ended_at,
                wall_s,
                commands_run,
            ),
            (None, reason) => AgentOutput {
                outcome: if reason.is_some() {
                    InvocationOutcome::Killed
                } else {
                    InvocationOutcome::Error
                },
                detail: Some(
                    reason.unwrap_or_else(|| format!("exited without a result: {stderr_tail}")),
                ),
                structured: None,
                started_at,
                ended_at,
                wall_s,
                usage: partial,
                usage_measured: false,
                models_used: Default::default(),
                cost_usd: None,
                turns: None,
                duration_api_ms: None,
                commands_run,
                denials: Vec::new(),
            },
        };
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HookSpec;
    use athena_data::records::Role;
    use std::collections::BTreeMap;

    fn spec() -> AgentInvocation {
        AgentInvocation {
            job: "20261010-153012-k7q2".into(),
            seq: 3,
            role: Role::Producer,
            model: "claude-sonnet-5-5".into(),
            system_prompt: "contract".into(),
            user_prompt: "brief".into(),
            tools: vec![
                "Read".into(),
                "Grep".into(),
                "Glob".into(),
                "Edit".into(),
                "Write".into(),
                "Bash".into(),
            ],
            allowed_tools: vec!["Bash(cargo test:*)".into()],
            permission_mode: "acceptEdits".into(),
            workdir: PathBuf::from("/state/jobs/x/work"),
            env: BTreeMap::new(),
            hook: Some(HookSpec {
                athena_bin: PathBuf::from("/opt/athena bin/athena"),
                policy_file: PathBuf::from("/state/policy.json"),
            }),
            max_budget_usd: 3.0,
            timeout: Duration::from_secs(1800),
            json_schema: serde_json::json!({"type": "object"}),
            scratch: PathBuf::from("/state/jobs/x/agent"),
        }
    }

    #[test]
    fn every_command_line_is_hermetic() {
        // M10 (allows): the flags; (blocks): a command line missing --restricted or another hermetic flag.
        let args = ClaudeCodeDriver::command_line(
            &spec(),
            Path::new("/s/system.md"),
            Path::new("/s/settings.json"),
        );
        assert!(check_hermetic(&args).is_ok());
        let text: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert!(
            text.windows(2)
                .any(|w| w[0] == "--tools" && w[1] == "Read,Grep,Glob,Edit,Write,Bash")
        );
        assert!(
            text.windows(2)
                .any(|w| w[0] == "--permission-prompts" && w[1] == "none")
        );
        assert!(
            !text.iter().any(|a| a.contains("brief")),
            "the prompt goes to stdin, never to the argument list"
        );
        for flag in HERMETIC_FLAGS {
            let without: Vec<OsString> = args.iter().filter(|a| *a != flag).cloned().collect();
            assert!(
                check_hermetic(&without).is_err(),
                "a command line without {flag} is refused"
            );
        }
    }

    #[test]
    fn the_hook_command_is_athena_itself() {
        let settings = ClaudeCodeDriver::settings(&spec());
        let command = settings["hooks"]["PreToolUse"][0]["hooks"][0]["command"]
            .as_str()
            .unwrap();
        assert_eq!(
            command,
            "'/opt/athena bin/athena' hook pre-tool-use --policy '/state/policy.json'"
        );
    }
}

/// The driver against a stand-in `claude` program (design §10.1: "a configurable agent command also lets
/// tests substitute a fake child process, to exercise timeouts, kills, and parent death"). These are unit
/// tests so that only this crate's test build can point the driver at a stand-in while the nested-run
/// marker is set, as it is when Athena's own checks run this suite inside a job.
#[cfg(test)]
mod stand_in_tests {
    use super::*;
    use crate::Usage;
    use athena_data::records::Role;
    use std::collections::BTreeMap;
    use std::os::unix::fs::PermissionsExt;

    const STAND_IN: &str = r#"#!/bin/sh
if [ "$1" = "--version" ]; then echo "2.1.299 (Claude Code)"; exit 0; fi
cat > /dev/null
echo '{"type":"system","subtype":"init","session_id":"0c32611b-5c0a-43eb-bacb-20a90569b268"}'
if [ "$MODE" = "spill" ]; then
  d="$HOME/.claude/projects/$(pwd | sed 's/[^A-Za-z0-9]/-/g')/0c32611b-5c0a-43eb-bacb-20a90569b268/tool-results"
  mkdir -p "$d" && echo 'a large tool result' > "$d/toolu_1.txt"
  mkdir -p "$HOME/.claude/session-env/0c32611b-5c0a-43eb-bacb-20a90569b268"
fi
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

    /// A driver for the stand-in, which makes no model call, so the nested-run marker does not apply.
    fn driver(bin: PathBuf, registry: Registry) -> ClaudeCodeDriver {
        let version = check_pinned(&bin, "2.1.299").unwrap();
        ClaudeCodeDriver {
            bin,
            version,
            registry,
            refuse_nested_runs: false,
        }
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
        let driver = driver(stand_in(dir.path()), Registry::new(state.path()));
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
    fn claude_codes_leftovers_are_removed_after_each_run() {
        // m4 of acceptance verification 1: the project folder Claude Code names after the snapshot, where
        // it spills large tool results, and the run's empty session folder do not outlive the run. Other
        // folders under the configuration directory are left alone.
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let home = tempfile::tempdir().unwrap();
        let other = home.path().join(".claude/projects/-home-someone-project");
        std::fs::create_dir_all(&other).unwrap();
        let driver = driver(stand_in(dir.path()), Registry::new(state.path()));
        let mut invocation = spec(dir.path(), "spill", Duration::from_secs(30));
        invocation
            .env
            .insert("HOME".to_string(), home.path().display().to_string());
        let out = driver
            .invoke(&invocation, &mut Recorder::default())
            .unwrap();
        assert_eq!(out.outcome, InvocationOutcome::Success);
        let projects = home.path().join(".claude/projects");
        assert!(
            !projects.join(project_dir_name(dir.path())).exists(),
            "the snapshot's project folder is gone"
        );
        assert!(
            !home
                .path()
                .join(".claude/session-env/0c32611b-5c0a-43eb-bacb-20a90569b268")
                .exists(),
            "the session folder is gone"
        );
        assert!(other.exists(), "nothing else is touched");
    }

    #[test]
    fn a_timed_out_invocation_is_killed_with_a_lower_bound() {
        // M13 (blocks): a timeout kills the group. M14: the record keeps the de-duplicated partial usage.
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let registry = Registry::new(state.path());
        let driver = driver(stand_in(dir.path()), registry.clone());
        let started = Instant::now();
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
        let driver = driver(stand_in(dir.path()), registry.clone());
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
        assert!(matches!(
            check_pinned(&stand_in(dir.path()), "9.9.9"),
            Err(DriverError::Spawn(_))
        ));
    }
}
