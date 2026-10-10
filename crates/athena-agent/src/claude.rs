//! The Claude Code driver (design §10.2).
//!
//! For each invocation it writes the composed system prompt and the hook settings to private files,
//! spawns `claude -p` in a new process group tied to its parent's life, with the hermetic flags and an
//! explicit environment, writes the brief to the child's standard input (so prompts never appear in
//! argument lists), and reads stream-json events as they arrive. It refuses to start inside a nested run.

use crate::procs::{GRACE, Registry, isolate, kill_group};
use crate::stream::StreamState;
use crate::{AgentDriver, AgentInvocation, AgentOutput, DriverError, UsageSink, nested_marker};
use athena_data::records::{DriverInfo, InvocationOutcome};
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
    let out = cmd.output().ok()?;
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
        if let Some(job) = nested_marker() {
            return Err(DriverError::Nested(job));
        }
        let version = claude_version(&bin).ok_or_else(|| {
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
        Ok(ClaudeCodeDriver {
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
        if let Some(job) = nested_marker() {
            return Err(DriverError::Nested(job));
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
        let mut child = cmd
            .spawn()
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
            (Some(result), _) => {
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
                    usage: result.usage,
                    usage_measured: true,
                    models_used: result.models_used.clone(),
                    cost_usd: result.cost_usd,
                    turns: result.turns,
                    duration_api_ms: result.duration_api_ms,
                    commands_run,
                    denials: result.denials.clone(),
                }
            }
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
