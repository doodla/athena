//! The Claude Code driver (design §10.2).
//!
//! For each invocation it writes the composed system prompt and the hook settings to private files,
//! spawns `claude -p` in a new process group tied to its parent's life, with the hermetic flags and an
//! explicit environment, writes the brief to the child's standard input (so prompts never appear in
//! argument lists), and reads stream-json events as they arrive. It refuses to start inside a nested run.

use crate::procs::{
    GRACE, MIN_STALL_WINDOW, Registry, deadline_reason, isolate, kill_group, stall_reason,
    stall_window,
};
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
    /// The shortest stall window; tests lower it to exercise the watchdog quickly.
    min_stall_window: Duration,
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
            min_stall_window: MIN_STALL_WINDOW,
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

/// The marker the detail of a limited invocation starts with; it is followed by the reset time.
const LIMIT_DETAIL: &str = "usage limit";

/// The longest part of the reply text kept in the detail of a limited invocation.
const LIMIT_TEXT_MAX: usize = 200;

/// True when a result is the service's refusal for a reached usage limit, such as
/// "You've hit your weekly limit · resets Oct 29, 6pm (UTC)": an error that carries no model usage
/// (no model ran) and whose text speaks of a limit and when it resets. Weekly, daily and session limits
/// share that wording. An ordinary error, or a success that quotes the words, is not a limit.
pub fn is_usage_limit(result: &ResultEvent) -> bool {
    if !result.is_error || !result.models_used.is_empty() {
        return false;
    }
    let text = result.text.as_deref().unwrap_or("").to_lowercase();
    text.contains("limit") && text.contains("resets")
}

/// Reads the reset time out of a limit reply such as "resets Oct 29, 6pm (UTC)" or "resets 3pm (UTC)".
/// The reply names no year, so the first such time that is not before `reference` is taken. Only UTC
/// is understood; another zone, or text without a time, gives `None`.
pub fn parse_reset(text: &str, reference: jiff::Timestamp) -> Option<jiff::Timestamp> {
    const MONTHS: [&str; 12] = [
        "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
    ];
    let lower = text.to_lowercase();
    let after = &lower[lower.find("resets")? + "resets".len()..];
    let mut month: Option<i8> = None;
    let mut day: Option<i8> = None;
    let mut time: Option<(i8, i8)> = None;
    let mut utc = false;
    for word in after.split(|c: char| c.is_whitespace() || c == ',' || c == '(' || c == ')') {
        if word.is_empty() {
            continue;
        }
        if word == "utc" {
            utc = true;
            break;
        }
        if time.is_some() {
            continue;
        }
        let clock = word
            .starts_with(|c: char| c.is_ascii_digit())
            .then(|| {
                word.strip_suffix("am")
                    .map(|h| (h, false))
                    .or_else(|| word.strip_suffix("pm").map(|h| (h, true)))
            })
            .flatten();
        if let Some((digits, pm)) = clock {
            let (hour, minute) = match digits.split_once(':') {
                Some((h, m)) => (h.parse::<i8>().ok()?, m.parse::<i8>().ok()?),
                None => (digits.parse::<i8>().ok()?, 0),
            };
            if !(1..=12).contains(&hour) || !(0..60).contains(&minute) {
                return None;
            }
            time = Some((hour % 12 + if pm { 12 } else { 0 }, minute));
        } else if month.is_none()
            && word.len() >= 3
            && word.chars().all(|c| c.is_ascii_alphabetic())
            && let Some(at) = MONTHS.iter().position(|m| word.starts_with(m))
        {
            month = Some(at as i8 + 1);
        } else if month.is_some()
            && day.is_none()
            && let Ok(d) = word.parse::<i8>()
        {
            day = Some(d);
        }
    }
    let (hour, minute) = time?;
    if !utc {
        return None;
    }
    let zone = jiff::tz::TimeZone::UTC;
    let at = |date: jiff::civil::Date| -> Option<jiff::Timestamp> {
        Some(
            date.at(hour, minute, 0, 0)
                .to_zoned(zone.clone())
                .ok()?
                .timestamp(),
        )
    };
    let today = reference.to_zoned(zone.clone()).date();
    match (month, day) {
        (Some(month), Some(day)) => (0..=1).find_map(|years| {
            let date = jiff::civil::Date::new(today.year() + years, month, day).ok()?;
            at(date).filter(|t| *t >= reference)
        }),
        (None, None) => {
            let first = at(today)?;
            if first >= reference {
                Some(first)
            } else {
                at(today.tomorrow().ok()?)
            }
        }
        _ => None,
    }
}

/// The detail of a limited invocation: it leads with the reset time, so that cutting the reply text
/// never loses it. [`reset_time_in`] reads it back.
fn limit_detail(text: &str, reset: Option<jiff::Timestamp>) -> String {
    let text: String = text.chars().take(LIMIT_TEXT_MAX).collect();
    match reset {
        Some(at) => format!("{LIMIT_DETAIL} (resets at {at}): {text}"),
        None => format!("{LIMIT_DETAIL} (reset time unknown): {text}"),
    }
}

/// The reset time a limited invocation's detail carries, if it names one.
pub fn reset_time_in(detail: &str) -> Option<jiff::Timestamp> {
    let rest = detail
        .strip_prefix(LIMIT_DETAIL)?
        .strip_prefix(" (resets at ")?;
    rest.split_once(')')?.0.parse().ok()
}

/// The output of an invocation that ended with a result event. Its tokens are the requested model's
/// `modelUsage` tokens (see [`ResultEvent::requested_usage`]), the quantity the gate reconciles.
///
/// A reply that says a usage limit was reached is [`InvocationOutcome::Limited`]: no model ran, so the
/// usage is zero and unmeasured, the cost is a reported zero, and no model is listed as used. The gate
/// then has no measured tokens to reconcile against a model that never ran.
pub fn output_from_result(
    result: &ResultEvent,
    model: &str,
    started_at: jiff::Timestamp,
    ended_at: jiff::Timestamp,
    wall_s: f64,
    commands_run: Vec<String>,
) -> AgentOutput {
    if is_usage_limit(result) {
        let text = result.text.as_deref().unwrap_or("");
        return AgentOutput {
            outcome: InvocationOutcome::Limited,
            detail: Some(limit_detail(text, parse_reset(text, ended_at))),
            structured: None,
            started_at,
            ended_at,
            wall_s,
            usage: Default::default(),
            usage_measured: false,
            models_used: Default::default(),
            cost_usd: Some(0.0),
            turns: result.turns,
            duration_api_ms: result.duration_api_ms,
            commands_run,
            denials: result.denials.clone(),
        };
    }
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

    const WEEKLY: &str = "You've hit your weekly limit · resets Oct 29, 6pm (UTC)";

    fn at(text: &str) -> jiff::Timestamp {
        text.parse().unwrap()
    }

    /// The output the driver builds from a `result` event line made of `fields`, ended at `ended`.
    fn output_of(fields: serde_json::Value, ended: &str) -> AgentOutput {
        let mut line = serde_json::json!({ "type": "result" });
        line.as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        let mut state = StreamState::default();
        state.on_line(&line.to_string());
        let result = state.result.expect("the line is a result event");
        output_from_result(
            &result,
            "claude-sonnet-5-5",
            at("2026-10-27T11:59:58Z"),
            at(ended),
            2.0,
            Vec::new(),
        )
    }

    /// A refusal as the service sends it: an error with the text, no usage and no model usage.
    fn refusal(text: &str) -> serde_json::Value {
        serde_json::json!({
            "subtype": "success", "is_error": true, "num_turns": 0, "duration_api_ms": 0,
            "total_cost_usd": 0, "usage": {}, "modelUsage": {}, "permission_denials": [],
            "result": text,
        })
    }

    #[test]
    fn a_weekly_limit_reply_is_a_usage_limit_with_its_reset_time() {
        let out = output_of(refusal(WEEKLY), "2026-10-27T12:00:00Z");
        assert_eq!(out.outcome, InvocationOutcome::Limited);
        let detail = out.detail.clone().unwrap();
        assert!(
            detail.starts_with("usage limit (resets at 2026-10-29T18:00:00Z): "),
            "{detail}"
        );
        assert!(detail.contains(WEEKLY), "{detail}");
        assert_eq!(reset_time_in(&detail), Some(at("2026-10-29T18:00:00Z")));
        // Nothing ran: zero cost, zero unmeasured usage, and no model listed as used.
        assert_eq!(out.cost_usd, Some(0.0));
        assert_eq!(out.usage, crate::Usage::default());
        assert!(!out.usage_measured);
        assert!(out.models_used.is_empty());
        assert!(out.structured.is_none());
    }

    #[test]
    fn other_limit_kinds_and_reset_forms_are_understood() {
        let now = at("2026-10-27T10:00:00Z");
        for (text, expected) in [
            (
                "You've hit your session limit · resets 3pm (UTC)",
                Some("2026-10-27T15:00:00Z"),
            ),
            (
                "5-hour limit reached ∙ resets 9:30am (UTC)",
                Some("2026-10-28T09:30:00Z"),
            ),
            (
                "daily limit · resets 12am (UTC)",
                Some("2026-10-28T00:00:00Z"),
            ),
            (
                "weekly limit · resets Jan 2, 6:30am (UTC)",
                Some("2027-01-02T06:30:00Z"),
            ),
            (
                "weekly limit · resets Oct 20, 6pm (UTC)",
                Some("2027-10-20T18:00:00Z"),
            ),
            // Another zone, or no time at all, is a limit with an unknown reset.
            ("weekly limit · resets Oct 29, 6pm (PST)", None),
            ("weekly limit · resets soon", None),
        ] {
            assert_eq!(parse_reset(text, now), expected.map(at), "{text}");
            let out = output_of(refusal(text), "2026-10-27T10:00:00Z");
            assert_eq!(out.outcome, InvocationOutcome::Limited, "{text}");
            assert_eq!(
                reset_time_in(&out.detail.unwrap()),
                expected.map(at),
                "{text}"
            );
        }
    }

    #[test]
    fn an_ordinary_error_stays_an_error_with_its_usage() {
        // An API error is not a limit.
        let out = output_of(
            serde_json::json!({
                "subtype": "error_during_execution", "is_error": true,
                "usage": { "input_tokens": 1, "output_tokens": 2 },
                "modelUsage": { "claude-sonnet-5-5": { "inputTokens": 1, "outputTokens": 2, "costUSD": 0.01 } },
                "total_cost_usd": 0.01, "result": "API Error: 500",
            }),
            "2026-10-27T12:00:00Z",
        );
        assert_eq!(out.outcome, InvocationOutcome::Error);
        assert_eq!(
            out.detail.as_deref(),
            Some("error_during_execution: API Error: 500")
        );
        assert!(out.usage_measured);
        assert_eq!(out.usage.output, 2);
        // A rate-limit error without a reset time is not a usage limit either.
        let out = output_of(refusal("rate limit exceeded"), "2026-10-27T12:00:00Z");
        assert_eq!(out.outcome, InvocationOutcome::Error);
        assert_eq!(out.detail.as_deref(), Some("success: rate limit exceeded"));
        assert!(out.usage_measured);
        // Nor is an error from a call that ran a model, whatever words its text quotes.
        let mut quoting = refusal(WEEKLY);
        quoting["modelUsage"] = serde_json::json!({ "claude-sonnet-5-5": { "inputTokens": 3 } });
        let out = output_of(quoting, "2026-10-27T12:00:00Z");
        assert_eq!(out.outcome, InvocationOutcome::Error);
        // A success that quotes the words is a success.
        let out = output_of(
            serde_json::json!({
                "subtype": "success", "is_error": false, "result": WEEKLY,
                "modelUsage": { "claude-sonnet-5-5": { "inputTokens": 3 } },
            }),
            "2026-10-27T12:00:00Z",
        );
        assert_eq!(out.outcome, InvocationOutcome::Success);
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
if [ "$MODE" = "limit" ]; then
  echo '{"type":"result","subtype":"success","is_error":true,"num_turns":0,"duration_api_ms":0,"total_cost_usd":0,"usage":{},"modelUsage":{},"permission_denials":[],"result":"You'\''ve hit your weekly limit · resets Oct 29, 6pm (UTC)"}'
  exit 1
fi
echo '{"type":"assistant","message":{"id":"msg_1","model":"claude-haiku-5-5","content":[{"type":"tool_use","id":"tu_1","name":"Bash","input":{"command":"cargo test"}}],"usage":{"input_tokens":3,"cache_creation_input_tokens":100,"cache_read_input_tokens":0,"output_tokens":7}}}'
if [ "$MODE" = "hang" ]; then sleep 60; exit 0; fi
if [ "$MODE" = "slow" ]; then
  for i in 1 2 3 4; do sleep 2; echo '{"type":"system","subtype":"progress"}'; done
fi
if [ "$MODE" = "stopped" ]; then echo on > "$STOP_FILE"; exit 143; fi
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
            min_stall_window: crate::procs::MIN_STALL_WINDOW,
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

    #[test]
    fn a_usage_limit_reply_from_the_child_is_a_limited_invocation_at_zero_cost() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let driver = driver(stand_in(dir.path()), Registry::new(state.path()));
        let out = driver
            .invoke(
                &spec(dir.path(), "limit", Duration::from_secs(30)),
                &mut Recorder::default(),
            )
            .unwrap();
        assert_eq!(out.outcome, InvocationOutcome::Limited, "{:?}", out.detail);
        let detail = out.detail.unwrap();
        assert!(detail.starts_with("usage limit (resets at "), "{detail}");
        assert!(
            detail.contains("You've hit your weekly limit · resets Oct 29, 6pm (UTC)"),
            "{detail}"
        );
        assert!(reset_time_in(&detail).is_some(), "{detail}");
        assert_eq!(out.cost_usd, Some(0.0));
        assert!(!out.usage_measured);
        assert_eq!(out.usage, Usage::default());
        assert!(out.models_used.is_empty());
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
        assert!(matches!(
            check_pinned(&stand_in(dir.path()), "9.9.9"),
            Err(DriverError::Spawn(_))
        ));
    }
}
