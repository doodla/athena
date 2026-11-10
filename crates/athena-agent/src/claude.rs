//! The Claude Code driver (design §10.2).
//!
//! For each invocation it writes the composed system prompt and the hook settings to private files,
//! spawns `claude -p` in a new process group tied to its parent's life, with the hermetic flags and an
//! explicit environment, writes the brief to the child's standard input (so prompts never appear in
//! argument lists), and reads stream-json events as they arrive. It refuses to start inside a nested run.

use crate::procs::{
    GRACE, MIN_STALL_WINDOW, Registry, deadline_reason, isolate, kill_group, retry_busy,
    stall_reason, stall_window,
};
use crate::stream::{ResultEvent, StreamState};
use crate::{
    AgentDriver, AgentInvocation, AgentOutput, DriverError, Usage, UsageSink, nested_marker,
    refuse_nested,
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
    /// The shortest stall window; tests lower it to exercise the watchdog quickly.
    min_stall_window: Duration,
    /// The wait before the one retry after an expired login; tests lower it.
    auth_retry_wait: Duration,
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
            "Claude Code {version} at {} is not the pinned {pinned}; to adopt the installed version run `cargo run -p xtask -- repin`, then ./setup.sh",
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
            min_stall_window: MIN_STALL_WINDOW,
            auth_retry_wait: AUTH_RETRY_WAIT,
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
        for dir in spec.hook.iter().flat_map(|hook| &hook.read_dirs) {
            args.push("--add-dir".into());
            args.push(dir.into());
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
        args.push("--effort".into());
        args.push(spec.effort.clone().into());
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

/// The HTTP status of an API error in the detail of a stopped agent, or `None` when the detail is not one.
///
/// The detail is built by [`output_from_result`] as `<subtype>: <first 300 characters of the result text>`.
/// An API error reads `success: Failed to authenticate. API Error: 401 {...}` or
/// `success: API Error: 429 ...`: the CLI reports it as an error result whose subtype is `success`. Only that
/// form counts, with a subtype of `success` or `error`; a model's own closing words under another subtype
/// (`error_max_turns`, `error_during_execution`) are free prose and are never read. No word list is used.
///
/// The driver retries a 401 with this, and the core classifies a stopped producer with it, so the two cannot
/// disagree about what an expired login looks like.
pub fn api_error_status(detail: &str) -> Option<u16> {
    let rest = ["success: ", "error: "]
        .iter()
        .find_map(|subtype| detail.strip_prefix(subtype))
        .unwrap_or(detail);
    let rest = rest
        .strip_prefix("Failed to authenticate. ")
        .unwrap_or(rest);
    let digits = rest.strip_prefix("API Error: ")?;
    let end = digits
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(digits.len());
    digits[..end].parse().ok()
}

/// How long the driver waits before it calls again after an expired login. The login normally refreshes by
/// itself within minutes; this is the short wait of one retry, not a wait for that refresh.
const AUTH_RETRY_WAIT: Duration = Duration::from_secs(30);

/// The least time a retry after an expired login must have left; with less, the first call's error is returned.
const MIN_RETRY_RUN: Duration = Duration::from_secs(10);

/// The least budget a retry after an expired login must have left, in US dollars.
const MIN_RETRY_BUDGET_USD: f64 = 0.01;

/// The line put before the brief of a retried call. The CLI keeps no session (`--no-session-persistence`),
/// so the retried agent does not remember what it did before the login expired; its edits are still in the
/// working directory.
const RETRY_NOTE: &str = "An earlier attempt at this task stopped when the login expired. Anything it changed is already in your working directory: check it before redoing work.";

/// The invocation for the one retry after an expired login, or `None` when too little is left to run it.
/// The retry is part of the same invocation, so it gets what the first call left of the timeout and the
/// budget: `used` is the time since the first call began, including the wait, and the first call's reported
/// cost is taken off the budget. Its brief starts with [`RETRY_NOTE`].
fn retry_spec(
    spec: &AgentInvocation,
    first: &AgentOutput,
    used: Duration,
) -> Option<AgentInvocation> {
    let timeout = spec.timeout.checked_sub(used)?;
    let budget = spec.max_budget_usd - first.cost_usd.unwrap_or(0.0);
    if timeout < MIN_RETRY_RUN || budget < MIN_RETRY_BUDGET_USD {
        return None;
    }
    let mut retry = spec.clone();
    retry.timeout = timeout;
    retry.max_budget_usd = budget;
    retry.user_prompt = format!("{RETRY_NOTE}\n\n{}", spec.user_prompt);
    Some(retry)
}

/// A sink for the second call of one invocation: it passes on the first call's usage and commands plus the
/// second call's, so the stored lower bound never goes backwards when the second call starts.
struct OffsetSink<'a> {
    inner: &'a mut dyn UsageSink,
    usage: Usage,
    commands: Vec<String>,
}

impl UsageSink for OffsetSink<'_> {
    fn update(&mut self, usage: &Usage, commands_run: &[String]) {
        let mut commands = self.commands.clone();
        commands.extend_from_slice(commands_run);
        self.inner.update(&sum_usage(&self.usage, usage), &commands);
    }
}

fn sum_usage(a: &Usage, b: &Usage) -> Usage {
    Usage {
        input: a.input + b.input,
        output: a.output + b.output,
        cache_read: a.cache_read + b.cache_read,
        cache_write: a.cache_write + b.cache_write,
    }
}

/// One output for two calls that make one invocation. Usage, models, turns, API time, commands and denials
/// are summed or joined; the cost is the sum only when both calls reported one (else it is unknown, and the
/// record bounds it by the cap); the usage is measured only when both were. The outcome, detail and structured
/// report are the second call's, which is what the invocation ended with. `wall_s` is the time of the pair.
fn merge_outputs(first: AgentOutput, second: AgentOutput, wall_s: f64) -> AgentOutput {
    let mut models_used = first.models_used;
    for (model, used) in second.models_used {
        match models_used.get_mut(&model) {
            Some(total) => {
                total.input += used.input;
                total.output += used.output;
                total.cache_read += used.cache_read;
                total.cache_write += used.cache_write;
                total.cost_usd += used.cost_usd;
            }
            None => {
                models_used.insert(model, used);
            }
        }
    }
    let mut commands_run = first.commands_run;
    commands_run.extend(second.commands_run);
    let mut denials = first.denials;
    denials.extend(second.denials);
    AgentOutput {
        outcome: second.outcome,
        detail: second.detail,
        structured: second.structured,
        started_at: first.started_at,
        ended_at: second.ended_at,
        wall_s,
        usage: sum_usage(&first.usage, &second.usage),
        usage_measured: first.usage_measured && second.usage_measured,
        models_used,
        cost_usd: first.cost_usd.zip(second.cost_usd).map(|(a, b)| a + b),
        turns: sum_options(first.turns, second.turns),
        duration_api_ms: sum_options(first.duration_api_ms, second.duration_api_ms),
        commands_run,
        denials,
    }
}

/// The sum of two optional counts; absent only when both are.
fn sum_options<T: std::ops::Add<Output = T>>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(a), Some(b)) => Some(a + b),
        (a, b) => a.or(b),
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
        let began = Instant::now();
        let first = self.invoke_once(spec, sink)?;
        if first.outcome != InvocationOutcome::Error
            || first.detail.as_deref().and_then(api_error_status) != Some(401)
        {
            return Ok(first);
        }
        self.retry_after_login_expiry(spec, sink, first, began)
    }
}

impl ClaudeCodeDriver {
    /// Runs the call again, once, after `first` ended on an expired login (an API error 401). The login
    /// usually refreshes by itself within minutes, so a producer that has worked for half an hour is not
    /// thrown away for it. The two calls are one invocation to the caller: the second gets what is left of the
    /// time and the budget, its usage reaches `sink` on top of the first call's, and the output merges both
    /// ([`merge_outputs`]). The retry is skipped, and `first` returned for the caller to close as an
    /// environment stop, when too little time or budget is left. If the retry fails too, its output is
    /// returned, whatever it is: there is no third call.
    fn retry_after_login_expiry(
        &self,
        spec: &AgentInvocation,
        sink: &mut dyn UsageSink,
        first: AgentOutput,
        began: Instant,
    ) -> Result<AgentOutput, DriverError> {
        let Some(retry) = retry_spec(spec, &first, began.elapsed() + self.auth_retry_wait) else {
            return Ok(first);
        };
        let until = Instant::now() + self.auth_retry_wait;
        loop {
            if self.registry.stopped() {
                return Err(DriverError::Stopped);
            }
            let left = until.saturating_duration_since(Instant::now());
            if left.is_zero() {
                break;
            }
            std::thread::sleep(left.min(Duration::from_secs(1)));
        }
        let mut offset = OffsetSink {
            inner: sink,
            usage: first.usage,
            commands: first.commands_run.clone(),
        };
        let second = self.invoke_once(&retry, &mut offset)?;
        Ok(merge_outputs(first, second, began.elapsed().as_secs_f64()))
    }

    /// One call of the CLI, with no retry.
    fn invoke_once(
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
        let mut deadline_hit = false;
        // Root cause of long dead runs: the deadline alone cannot tell a stalled agent from a slow one.
        // Any line on the stream is activity; silence for the stall window ends the run early.
        let window = stall_window(spec.timeout, self.min_stall_window);
        let mut last_activity = Instant::now();
        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(line) => {
                    last_activity = Instant::now();
                    state.on_line(&line);
                    sink.update(&state.partial_usage(), &state.commands_run());
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
            if killed.is_none() {
                if Instant::now() >= deadline {
                    killed = Some(deadline_reason(spec.timeout.as_secs()));
                    deadline_hit = true;
                } else if last_activity.elapsed() >= window {
                    killed = Some(stall_reason(window.as_secs()));
                    deadline_hit = true;
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
        // `athena stop` ends the registered groups itself, so the child can die between two of the loop's
        // checks and the driver never saw the switch. A child with no result while the switch is on was
        // stopped by it, not broken (m8 of the v0 acceptance review).
        if killed.is_none() && state.result.is_none() && self.registry.stopped() {
            killed = Some("the kill switch was turned on".into());
        }
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
                outcome: if reason.is_some() && deadline_hit {
                    InvocationOutcome::TimedOut
                } else if reason.is_some() {
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

    #[test]
    fn the_model_and_effort_of_an_invocation_reach_the_command_line() {
        let mut invocation = spec();
        let args = |s: &AgentInvocation| -> Vec<String> {
            ClaudeCodeDriver::command_line(s, Path::new("p.md"), Path::new("s.json"))
                .iter()
                .map(|a| a.to_string_lossy().into_owned())
                .collect()
        };
        let value_after = |args: &[String], flag: &str| -> String {
            let at = args
                .iter()
                .position(|a| a == flag)
                .unwrap_or_else(|| panic!("no {flag}: {args:?}"));
            args[at + 1].clone()
        };
        for (model, effort) in [
            ("claude-sonnet-5-5", "high"),
            ("claude-haiku-5-5", "low"),
            ("claude-opus-5-5", "xhigh"),
        ] {
            invocation.model = model.into();
            invocation.effort = effort.into();
            let args = args(&invocation);
            assert_eq!(value_after(&args, "--model"), model);
            assert_eq!(value_after(&args, "--effort"), effort);
        }
    }

    fn spec() -> AgentInvocation {
        AgentInvocation {
            job: "20261010-153012-k7q2".into(),
            seq: 3,
            role: Role::Producer,
            model: "claude-sonnet-5-5".into(),
            effort: "high".into(),
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
                read_dirs: Vec::new(),
            }),
            max_budget_usd: 3.0,
            timeout: Duration::from_secs(1800),
            json_schema: serde_json::json!({"type": "object"}),
            scratch: PathBuf::from("/state/jobs/x/agent"),
        }
    }

    #[test]
    fn a_read_directory_reaches_the_command_line_as_add_dir_and_only_then() {
        let text = |s: &AgentInvocation| -> Vec<String> {
            ClaudeCodeDriver::command_line(s, Path::new("p.md"), Path::new("s.json"))
                .iter()
                .map(|a| a.to_string_lossy().into_owned())
                .collect()
        };
        let mut invocation = spec();
        assert!(!text(&invocation).iter().any(|a| a == "--add-dir"));
        invocation.hook.as_mut().unwrap().read_dirs = vec![PathBuf::from("/state/jobs/x/evidence")];
        let args = text(&invocation);
        assert!(
            args.windows(2)
                .any(|w| w[0] == "--add-dir" && w[1] == "/state/jobs/x/evidence"),
            "{args:?}"
        );
        assert!(
            check_hermetic(&ClaudeCodeDriver::command_line(
                &invocation,
                Path::new("p.md"),
                Path::new("s.json")
            ))
            .is_ok()
        );
        // A run without a hook has no directory to add.
        invocation.hook = None;
        assert!(!text(&invocation).iter().any(|a| a == "--add-dir"));
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

    const STAND_IN: &str = r#"#!/bin/sh
if [ "$1" = "--version" ]; then echo "2.1.299 (Claude Code)"; exit 0; fi
cat > "${PROMPT_FILE:-/dev/null}"
n=1
if [ -n "$COUNT_FILE" ]; then
  if [ -f "$COUNT_FILE" ]; then n=$(( $(cat "$COUNT_FILE") + 1 )); fi
  echo "$n" > "$COUNT_FILE"
  printf '%s\n' "$*" >> "$COUNT_FILE.args"
fi
AUTH='{"type":"result","subtype":"success","is_error":true,"num_turns":1,"duration_api_ms":100,"total_cost_usd":0.25,"result":"Failed to authenticate. API Error: 401 {\"type\":\"error\",\"error\":{\"type\":\"authentication_error\",\"message\":\"OAuth access token has expired. Please obtain a new token or refresh your existing token.\"}}","usage":{"input_tokens":4,"cache_creation_input_tokens":10,"cache_read_input_tokens":2,"output_tokens":9},"modelUsage":{"claude-haiku-5-5":{"inputTokens":4,"outputTokens":9,"cacheReadInputTokens":2,"cacheCreationInputTokens":10,"costUSD":0.25}},"permission_denials":[]}'
echo '{"type":"system","subtype":"init","session_id":"0c32611b-5c0a-43eb-bacb-20a90569b268"}'
if [ "$MODE" = "spill" ]; then
  d="$HOME/.claude/projects/$(pwd | sed 's/[^A-Za-z0-9]/-/g')/0c32611b-5c0a-43eb-bacb-20a90569b268/tool-results"
  mkdir -p "$d" && echo 'a large tool result' > "$d/toolu_1.txt"
  mkdir -p "$HOME/.claude/session-env/0c32611b-5c0a-43eb-bacb-20a90569b268"
fi
echo '{"type":"assistant","message":{"id":"msg_1","model":"claude-haiku-5-5","content":[{"type":"tool_use","id":"tu_1","name":"Bash","input":{"command":"cargo test"}}],"usage":{"input_tokens":3,"cache_creation_input_tokens":100,"cache_read_input_tokens":0,"output_tokens":7}}}'
if [ "$MODE" = "hang" ]; then sleep 60; exit 0; fi
if [ "$MODE" = "slow" ]; then
  for i in 1 2 3 4; do sleep 2; echo '{"type":"system","subtype":"progress"}'; done
fi
if [ "$MODE" = "stopped" ]; then echo on > "$STOP_FILE"; exit 143; fi
if [ "$MODE" = "auth_always" ] || { [ "$MODE" = "auth_once" ] && [ "$n" = "1" ]; }; then printf '%s\n' "$AUTH"; exit 0; fi
if [ "$MODE" = "auth_stop" ]; then printf '%s\n' "$AUTH"; echo on > "$STOP_FILE"; exit 0; fi
if [ "$MODE" = "limit" ]; then
  echo '{"type":"result","subtype":"success","is_error":true,"num_turns":1,"total_cost_usd":0.01,"result":"API Error: 429 usage limit reached","permission_denials":[]}'
  exit 0
fi
echo '{"type":"result","subtype":"success","is_error":false,"num_turns":2,"duration_api_ms":900,"total_cost_usd":0.002,"usage":{"input_tokens":5,"cache_creation_input_tokens":120,"cache_read_input_tokens":40,"output_tokens":60},"modelUsage":{"claude-haiku-5-5":{"inputTokens":5,"outputTokens":60,"cacheReadInputTokens":40,"cacheCreationInputTokens":120,"costUSD":0.002}},"permission_denials":[],"structured_output":{"answer":42}}'
"#;

    fn stand_in(dir: &Path) -> PathBuf {
        let path = dir.join("claude");
        athena_testkit::write_executable(&path, STAND_IN);
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
            min_stall_window: crate::procs::MIN_STALL_WINDOW,
            auth_retry_wait: AUTH_RETRY_WAIT,
        }
    }

    /// A spec whose child counts its calls in `count` and keeps its last prompt in `prompt`.
    fn counted(dir: &Path, mode: &str, timeout: Duration) -> (AgentInvocation, PathBuf, PathBuf) {
        let count = dir.join("count");
        let prompt = dir.join("prompt");
        let mut invocation = spec(dir, mode, timeout);
        invocation
            .env
            .insert("COUNT_FILE".into(), count.display().to_string());
        invocation
            .env
            .insert("PROMPT_FILE".into(), prompt.display().to_string());
        (invocation, count, prompt)
    }

    fn calls(count: &Path) -> u32 {
        std::fs::read_to_string(count)
            .unwrap()
            .trim()
            .parse()
            .unwrap()
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
            effort: "medium".into(),
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

    /// A sink that keeps every update, with the commands it was given.
    #[derive(Default)]
    struct Trace(Vec<(Usage, Vec<String>)>);

    impl UsageSink for Trace {
        fn update(&mut self, usage: &Usage, commands_run: &[String]) {
            self.0.push((*usage, commands_run.to_vec()));
        }
    }

    const LOGIN_EXPIRED_DETAIL: &str = "success: Failed to authenticate. API Error: 401";

    #[test]
    fn a_login_expiry_is_retried_once_and_a_second_call_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let mut driver = driver(stand_in(dir.path()), Registry::new(state.path()));
        driver.auth_retry_wait = Duration::from_millis(300);
        let (invocation, count, prompt) = counted(dir.path(), "auth_once", Duration::from_secs(30));
        let mut sink = Trace::default();
        let started = Instant::now();
        let out = driver.invoke(&invocation, &mut sink).unwrap();
        assert!(started.elapsed() >= Duration::from_millis(300), "it waited");
        // Exactly two calls, and the job's caller sees one successful invocation.
        assert_eq!(calls(&count), 2);
        assert_eq!(out.outcome, InvocationOutcome::Success);
        assert_eq!(out.detail, None);
        assert_eq!(out.structured.clone().unwrap()["answer"], 42);
        // The second call ran with what the first left of the budget (0.50 less its 0.25), and was told
        // that an earlier attempt had stopped.
        let args = std::fs::read_to_string(format!("{}.args", count.display())).unwrap();
        let args: Vec<&str> = args.lines().collect();
        assert_eq!(args.len(), 2);
        assert!(args[0].contains("--max-budget-usd 0.50"), "{}", args[0]);
        assert!(args[1].contains("--max-budget-usd 0.25"), "{}", args[1]);
        let second_prompt = std::fs::read_to_string(&prompt).unwrap();
        assert!(second_prompt.starts_with(RETRY_NOTE), "{second_prompt}");
        assert!(second_prompt.ends_with("brief"), "{second_prompt}");
        // Both calls' spend is in the one output: the cost, the tokens, the models and the commands.
        assert!(
            (out.cost_usd.unwrap() - 0.252).abs() < 1e-9,
            "{:?}",
            out.cost_usd
        );
        assert_eq!(
            out.usage,
            Usage {
                input: 9,
                output: 69,
                cache_read: 42,
                cache_write: 130
            }
        );
        assert!(out.usage_measured);
        assert_eq!(out.turns, Some(3));
        assert_eq!(out.duration_api_ms, Some(1000));
        let haiku = &out.models_used["claude-haiku-5-5"];
        assert_eq!((haiku.input, haiku.output), (9, 69));
        assert!((haiku.cost_usd - 0.252).abs() < 1e-9);
        assert_eq!(out.commands_run, vec!["cargo test", "cargo test"]);
        assert!(out.wall_s >= 0.3, "the pair's time includes the wait");
        assert!(out.started_at <= out.ended_at);
        // The sink never goes backwards: the second call's updates carry the first call's totals.
        let (usage, commands) = sink.0.last().unwrap();
        assert_eq!(
            *usage,
            Usage {
                input: 7,
                output: 16,
                cache_read: 2,
                cache_write: 110
            }
        );
        assert_eq!(commands, &vec!["cargo test".to_string(); 2]);
        assert!(
            sink.0.windows(2).all(|w| w[0].0.input <= w[1].0.input),
            "{:?}",
            sink.0
        );
    }

    #[test]
    fn two_login_expiries_make_two_calls_and_return_the_error() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let mut driver = driver(stand_in(dir.path()), Registry::new(state.path()));
        driver.auth_retry_wait = Duration::from_millis(300);
        let (invocation, count, _) = counted(dir.path(), "auth_always", Duration::from_secs(30));
        let started = Instant::now();
        let out = driver
            .invoke(&invocation, &mut Recorder::default())
            .unwrap();
        // One retry and no more: the second 401 is returned for the caller to close as an environment stop.
        assert_eq!(calls(&count), 2);
        assert!(started.elapsed() < Duration::from_secs(20));
        assert_eq!(out.outcome, InvocationOutcome::Error);
        let detail = out.detail.unwrap();
        assert!(detail.starts_with(LOGIN_EXPIRED_DETAIL), "{detail}");
        assert_eq!(api_error_status(&detail), Some(401));
        assert!((out.cost_usd.unwrap() - 0.5).abs() < 1e-9, "both spends");
    }

    #[test]
    fn a_login_expiry_with_no_time_left_is_not_retried() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let mut driver = driver(stand_in(dir.path()), Registry::new(state.path()));
        driver.auth_retry_wait = Duration::from_secs(3);
        let (invocation, count, _) = counted(dir.path(), "auth_once", Duration::from_secs(2));
        let started = Instant::now();
        let out = driver
            .invoke(&invocation, &mut Recorder::default())
            .unwrap();
        assert_eq!(calls(&count), 1);
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "it did not wait"
        );
        assert_eq!(out.outcome, InvocationOutcome::Error);
        assert!(out.detail.unwrap().starts_with(LOGIN_EXPIRED_DETAIL));
    }

    #[test]
    fn an_athena_stop_during_a_login_expiry_wait_ends_the_retry() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let registry = Registry::new(state.path());
        let mut driver = driver(stand_in(dir.path()), registry.clone());
        driver.auth_retry_wait = Duration::from_millis(300);
        let (mut invocation, count, _) = counted(dir.path(), "auth_stop", Duration::from_secs(30));
        invocation.env.insert(
            "STOP_FILE".to_string(),
            registry.stop_file().display().to_string(),
        );
        let result = driver.invoke(&invocation, &mut Recorder::default());
        assert!(matches!(result, Err(DriverError::Stopped)), "{result:?}");
        assert_eq!(calls(&count), 1, "no second call after the stop");
    }

    #[test]
    fn a_usage_limit_is_not_retried() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let mut driver = driver(stand_in(dir.path()), Registry::new(state.path()));
        driver.auth_retry_wait = Duration::from_millis(300);
        let (invocation, count, _) = counted(dir.path(), "limit", Duration::from_secs(30));
        let out = driver
            .invoke(&invocation, &mut Recorder::default())
            .unwrap();
        assert_eq!(calls(&count), 1);
        assert_eq!(out.outcome, InvocationOutcome::Error);
        assert_eq!(api_error_status(&out.detail.unwrap()), Some(429));
    }

    #[test]
    fn retry_spec_leaves_the_remaining_time_and_budget() {
        let mut original = spec(Path::new("/w"), "complete", Duration::from_secs(600));
        original.user_prompt = "the brief".into();
        original.max_budget_usd = 3.0;
        let at = jiff::Timestamp::UNIX_EPOCH;
        let mut first = output_from_result(
            &ResultEvent {
                is_error: true,
                subtype: "success".into(),
                text: Some("Failed to authenticate. API Error: 401".into()),
                cost_usd: Some(1.25),
                ..ResultEvent::default()
            },
            "claude-haiku-5-5",
            at,
            at,
            100.0,
            Vec::new(),
        );
        // 100 s of the first call and a 30 s wait leave 470 s; $3.00 less $1.25 leaves $1.75.
        let retry = retry_spec(&original, &first, Duration::from_secs(130)).unwrap();
        assert_eq!(retry.timeout, Duration::from_secs(470));
        assert!((retry.max_budget_usd - 1.75).abs() < 1e-9);
        assert_eq!(retry.user_prompt, format!("{RETRY_NOTE}\n\nthe brief"));
        assert_eq!(retry.job, original.job);
        // Less than the floor of time, or of budget, is not worth a call.
        assert!(retry_spec(&original, &first, Duration::from_secs(591)).is_none());
        assert!(retry_spec(&original, &first, Duration::from_secs(600)).is_none());
        assert!(retry_spec(&original, &first, Duration::from_secs(900)).is_none());
        first.cost_usd = Some(2.995);
        assert!(retry_spec(&original, &first, Duration::from_secs(130)).is_none());
        // A first call with no reported cost leaves the whole budget to the retry.
        first.cost_usd = None;
        let retry = retry_spec(&original, &first, Duration::from_secs(130)).unwrap();
        assert!((retry.max_budget_usd - 3.0).abs() < 1e-9);
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
    fn a_timed_out_invocation_is_timed_out_with_a_lower_bound() {
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
        assert_eq!(out.outcome, InvocationOutcome::TimedOut);
        assert_eq!(out.detail.as_deref(), Some("deadline of 2s reached"));
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
    fn a_silent_agent_is_stopped_early_as_stalled_not_at_the_deadline() {
        // The stand-in emits one line and goes silent; the 15s timeout gives a 5s stall window.
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let mut driver = driver(stand_in(dir.path()), Registry::new(state.path()));
        driver.min_stall_window = Duration::from_secs(1);
        let started = Instant::now();
        let out = driver
            .invoke(
                &spec(dir.path(), "hang", Duration::from_secs(15)),
                &mut Recorder::default(),
            )
            .unwrap();
        assert!(
            started.elapsed() < Duration::from_secs(12),
            "the stall ended the run well before the timeout"
        );
        assert_eq!(out.outcome, InvocationOutcome::TimedOut);
        let detail = out.detail.unwrap();
        assert!(detail.contains("no progress for 5s"), "{detail}");
    }

    #[test]
    fn a_slow_agent_that_keeps_emitting_is_not_stopped_as_stalled() {
        // Lines arrive every 2s, inside the 5s window, and the run lasts longer than the window.
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let mut driver = driver(stand_in(dir.path()), Registry::new(state.path()));
        driver.min_stall_window = Duration::from_secs(1);
        let out = driver
            .invoke(
                &spec(dir.path(), "slow", Duration::from_secs(15)),
                &mut Recorder::default(),
            )
            .unwrap();
        assert_eq!(out.outcome, InvocationOutcome::Success);
        assert!(out.wall_s > 5.0, "the run outlasted the stall window");
    }

    #[test]
    fn a_stall_window_is_never_shorter_than_its_floor_or_a_third_of_the_timeout() {
        use crate::procs::stall_window;
        let floor = Duration::from_secs(60);
        assert_eq!(
            stall_window(Duration::from_secs(1800), floor).as_secs(),
            600
        );
        assert_eq!(stall_window(Duration::from_secs(2), floor), floor);
    }

    #[test]
    fn a_child_ending_without_a_result_while_the_kill_switch_is_on_is_killed_not_an_error() {
        // m8 of the v0 acceptance review: the switch ended the child before the driver's own check.
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let registry = Registry::new(state.path());
        let driver = driver(stand_in(dir.path()), registry.clone());
        let mut invocation = spec(dir.path(), "stopped", Duration::from_secs(30));
        invocation.env.insert(
            "STOP_FILE".to_string(),
            registry.stop_file().display().to_string(),
        );
        assert!(!registry.stopped());
        let out = driver
            .invoke(&invocation, &mut Recorder::default())
            .unwrap();
        assert_eq!(out.outcome, InvocationOutcome::Killed);
        assert!(out.detail.unwrap().contains("kill switch"));
        // Without the switch the same early exit is an error.
        std::fs::remove_file(registry.stop_file()).unwrap();
        let mut plain = spec(dir.path(), "stopped", Duration::from_secs(30));
        plain.env.insert(
            "STOP_FILE".to_string(),
            dir.path().join("elsewhere").display().to_string(),
        );
        let out = driver.invoke(&plain, &mut Recorder::default()).unwrap();
        assert!(!registry.stopped());
        assert_eq!(out.outcome, InvocationOutcome::Error);
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
        let Err(DriverError::Spawn(message)) = check_pinned(&stand_in(dir.path()), "9.9.9") else {
            panic!("a different version must be refused");
        };
        assert!(message.contains("2.1.299"), "{message}");
        assert!(message.contains("9.9.9"), "{message}");
        assert!(message.contains("cargo run -p xtask -- repin"), "{message}");
    }

    #[test]
    fn a_host_at_exactly_the_pinned_version_is_accepted() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            check_pinned(&stand_in(dir.path()), "2.1.299").unwrap(),
            "2.1.299"
        );
    }
}
