//! Running the capability's checks on the candidate (design §7.2; M12, M13).
//!
//! Checks run in the verifier's snapshot, with the agent environment, the capability's environment, and
//! the nested-run marker, each in its own process group under the kill switch and a deadline. A check of
//! kind `test` fails if it ran zero tests, counted from the test harness's own summary lines, even when
//! its exit status is 0. Output tails are redacted: a passing check keeps 4 KiB, a failed one its last 200
//! lines. A check that names `skip_when_only` patterns is skipped, with a note in its record, when every
//! changed path matches one.

use crate::capability::CheckSpec;
use crate::ci_result::{CiRecord, Source};
use anyhow::Result;
use athena_agent::procs::{GRACE, Registry, isolate, kill_group};
use athena_data::config::CheckKind;
use athena_data::records::{Baseline, CheckRun};
use athena_data::{Quantity, Unit};
use athena_gate::redact::Redactor;
use regex::Regex;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

/// The largest output tail a check record keeps.
pub const TAIL_BYTES: usize = 4096;

/// The number of tests a run of the Rust test harness reports (passed plus failed), or `None` when no
/// summary line appears at all.
pub fn count_tests(output: &str) -> Option<u64> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"test result: \w+\. (\d+) passed; (\d+) failed").expect("valid pattern")
    });
    let mut found = false;
    let mut total = 0;
    for caps in re.captures_iter(output) {
        found = true;
        total += caps[1].parse::<u64>().unwrap_or(0) + caps[2].parse::<u64>().unwrap_or(0);
    }
    found.then_some(total)
}

/// The flag Athena adds to a plain `cargo test`, so every test binary runs and each one's failures are listed.
/// Without it cargo stops after the first failing binary and the later binaries' failures never appear.
const NO_FAIL_FAST: &str = "--no-fail-fast";

/// The note appended to a failed run that got [`NO_FAIL_FAST`], because `command` keeps the configured text.
const NO_FAIL_FAST_NOTE: &str = "ran with --no-fail-fast added to the configured command";

/// The arguments to run a test check with, and whether its failing test names can be recorded.
///
/// Names are recorded only for a plain `cargo test` (program named `cargo`, first argument `test`, no
/// `--fail-fast`); a wrapper script, `cargo +toolchain test` or another runner records none and is compared
/// by check name. For such a command `--no-fail-fast` is inserted right after `test` (before any `--`)
/// unless it is already there. Anything else is returned unchanged.
pub fn cargo_test_args(program: &str, args: &[String]) -> (Vec<String>, bool) {
    let is_cargo = Path::new(program).file_name().is_some_and(|n| n == "cargo");
    let own_flags = args.iter().take_while(|a| a.as_str() != "--");
    let plain = is_cargo
        && args.first().map(String::as_str) == Some("test")
        && own_flags.clone().all(|a| a != "--fail-fast");
    if !plain {
        return (args.to_vec(), false);
    }
    let mut adjusted = args.to_vec();
    if own_flags.clone().all(|a| a != NO_FAIL_FAST) {
        adjusted.insert(1, NO_FAIL_FAST.to_string());
    }
    (adjusted, true)
}

/// The name of the test binary a `Running` line opens: the file name of the path in parentheses, without
/// its `-<hash>` suffix (`target/debug/deps/athena_core-0f3a` gives `athena_core`).
fn binary_of_running_line(rest: &str) -> String {
    let path = match (rest.rfind('('), rest.ends_with(')')) {
        (Some(open), true) => &rest[open + 1..rest.len() - 1],
        _ => rest,
    };
    let file = path.rsplit('/').next().unwrap_or(path);
    match file.rsplit_once('-') {
        Some((stem, hash)) if !hash.is_empty() && hash.chars().all(|c| c.is_ascii_hexdigit()) => {
            stem.to_string()
        }
        _ => file.to_string(),
    }
}

/// One test binary's part of a cargo test run, while it is read.
struct Section {
    binary: String,
    listed: Vec<String>,
    /// The `N failed` count of its `test result:` line, if it printed one.
    failed: Option<usize>,
}

impl Section {
    /// Adds this section's names to `out`: its listed tests, qualified by binary, and the marker
    /// `<binary> (did not finish)` when it printed no `test result:` line (a crash, abort or early exit) or
    /// when its failed count differs from the names listed (a cut-short list).
    fn close_into(self, out: &mut std::collections::BTreeSet<String>) {
        let complete = self.failed == Some(self.listed.len());
        for name in &self.listed {
            out.insert(format!("{} {name}", self.binary));
        }
        if !complete {
            out.insert(format!("{} (did not finish)", self.binary));
        }
    }
}

/// The failing tests of a cargo test run, read from its output section by section (one section per
/// `Running` or `Doc-tests` line): each name from a section's indented `failures:` list, qualified by its
/// binary, as `<binary> <test>`. A section whose process never printed a `test result:` line, or whose
/// failed count differs from the names listed, adds `<binary> (did not finish)`, so a crash or a cut-short
/// list counts as a failure and can never hide behind the names that were listed. The result is sorted
/// and without duplicates. Output with no `Running` or `Doc-tests` line yields an empty list, which the
/// comparison reads as "no names known".
pub fn failed_test_names(output: &str) -> Vec<String> {
    let lines: Vec<&str> = output.lines().collect();
    let mut out = std::collections::BTreeSet::new();
    let mut section: Option<Section> = None;
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();
        let opened = if let Some(rest) = trimmed.strip_prefix("Running ") {
            Some(binary_of_running_line(rest.trim_end()))
        } else {
            trimmed
                .strip_prefix("Doc-tests ")
                .map(|krate| format!("doc-tests {}", krate.trim()))
        };
        if let Some(binary) = opened {
            if let Some(done) = section.take() {
                done.close_into(&mut out);
            }
            section = Some(Section {
                binary,
                listed: Vec::new(),
                failed: None,
            });
        } else if let Some(current) = section.as_mut() {
            if line == "failures:" && lines.get(i + 1).is_some_and(|n| !n.trim().is_empty()) {
                // The list: indented names up to the blank line that ends it.
                while let Some(name) = lines.get(i + 1).and_then(|n| n.strip_prefix("    ")) {
                    if name.trim().is_empty() {
                        break;
                    }
                    current.listed.push(name.trim_end().to_string());
                    i += 1;
                }
            } else if let Some(rest) = line.strip_prefix("test result:") {
                current.failed = failed_count(rest);
            }
        }
        i += 1;
    }
    if let Some(done) = section {
        done.close_into(&mut out);
    }
    out.into_iter().collect()
}

/// The `N` of `N failed` in the rest of a `test result:` line.
fn failed_count(rest: &str) -> Option<usize> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"(\d+) failed").expect("valid pattern"));
    re.captures(rest)?[1].parse().ok()
}

/// The last `max` bytes of `text`, on a character boundary.
pub fn tail(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_string();
    }
    let mut start = text.len() - max;
    while !text.is_char_boundary(start) {
        start += 1;
    }
    format!("…{}", &text[start..])
}

/// How many final output lines a failed check's record keeps, so a failing test's name and message
/// survive for a retry or a person to read.
pub const FAILED_TAIL_LINES: usize = 200;

/// The byte ceiling on a failed check's kept lines, so one enormous line cannot bloat the committed
/// record. Ordinary output stays far below it, and the cut keeps the end.
pub const FAILED_TAIL_BYTES: usize = 64 * 1024;

/// The last `n` lines of `text`, with its final newline kept when it had one.
pub fn tail_lines(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= n {
        return text.to_string();
    }
    let mut kept = lines[lines.len() - n..].join("\n");
    if text.ends_with('\n') {
        kept.push('\n');
    }
    kept
}

/// The most failing names one log entry lists before it counts the rest.
const LOGGED_NAMES: usize = 5;

/// The names of the tests that failed in `run`: its recorded `failed_tests` when it has any, else each
/// `test <name> ... FAILED` line of its kept output (the Rust harness's own line, which also covers a
/// command that is not a plain `cargo test`). Without duplicates, in the order found; empty when none show.
pub fn failing_names(run: &CheckRun) -> Vec<String> {
    if !run.failed_tests.is_empty() {
        return run.failed_tests.clone();
    }
    let mut names: Vec<String> = Vec::new();
    for line in run.output_tail.lines() {
        let name = line
            .trim_end()
            .strip_prefix("test ")
            .and_then(|rest| rest.strip_suffix(" ... FAILED"));
        if let Some(name) = name
            && !names.iter().any(|n| n == name)
        {
            names.push(name.to_string());
        }
    }
    names
}

/// One check's part of the drain log's `checks` line: `name ok (n tests)`, `name ok`, or `name FAILED`
/// followed by the failing test names (up to five, then `+N more`) when any are known.
pub fn log_entry(run: &CheckRun) -> String {
    if run.passed {
        return match run.tests_ran {
            Some(n) => format!("{} ok ({n} tests)", run.name),
            None => format!("{} ok", run.name),
        };
    }
    let names = failing_names(run);
    if names.is_empty() {
        return format!("{} FAILED", run.name);
    }
    let mut listed = names
        .iter()
        .take(LOGGED_NAMES)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    if names.len() > LOGGED_NAMES {
        listed.push_str(&format!(", +{} more", names.len() - LOGGED_NAMES));
    }
    format!("{} FAILED: {listed}", run.name)
}

/// Whether `path` matches one skip pattern. A pattern ending in `/` matches every path under that
/// directory; a pattern starting with `*.` matches any path ending in the rest, at any depth (`*.json`
/// matches `a.json` and `studies/x/a.json`); any other pattern matches exactly that path.
pub fn path_matches(pattern: &str, path: &str) -> bool {
    if pattern.ends_with('/') {
        path.starts_with(pattern)
    } else if let Some(suffix) = pattern.strip_prefix('*') {
        suffix.starts_with('.') && suffix.len() > 1 && path.ends_with(suffix)
    } else {
        pattern == path
    }
}

/// The note to record when a check is skipped, or `None` when it must run. A check is skipped only when
/// it names `skip_when_only` patterns and the change list is not empty and every changed path matches at
/// least one pattern. An absent or empty pattern list, an empty change list, or any other path runs it.
pub fn skip_note(spec: &CheckSpec, changed: &[String]) -> Option<String> {
    let patterns = spec.skip_when_only.as_ref()?;
    if patterns.is_empty() || changed.is_empty() {
        return None;
    }
    changed
        .iter()
        .all(|p| patterns.iter().any(|pat| path_matches(pat, p)))
        .then(|| {
            format!(
                "not run: every changed path matches skip_when_only ({})",
                patterns.join(", ")
            )
        })
}

/// The record of a check that was skipped. It counts as passed and carries the note.
pub fn skipped_run(spec: &CheckSpec, note: String) -> CheckRun {
    CheckRun {
        name: spec.name.clone(),
        command: spec.run.clone(),
        exit_code: None,
        passed: true,
        tests_ran: None,
        duration: Quantity::measured(0.0, Unit::Seconds, "not run"),
        output_tail: String::new(),
        note: Some(note),
        failed_tests: Vec::new(),
        reused: false,
    }
}

/// The check records for a result reused from a [`CiRecord`], one per spec the record covers, so Athena's
/// check step reads the record instead of running the check again. A spec the record does not name is left
/// out (the caller runs it); a record for another SHA than `sha` yields nothing.
pub fn runs_from_record(specs: &[CheckSpec], record: &CiRecord, sha: &str) -> Vec<CheckRun> {
    if record.sha != sha {
        return Vec::new();
    }
    let origin = match (&record.source, &record.run_address) {
        (Source::Ci, Some(address)) => format!("CI run {address}"),
        (Source::Ci, None) => "CI run".to_string(),
        (Source::Local, _) => "local run".to_string(),
    };
    specs
        .iter()
        .filter_map(|spec| {
            let step = record.steps.iter().find(|s| s.name == spec.name)?;
            Some(CheckRun {
                name: spec.name.clone(),
                command: spec.run.clone(),
                exit_code: None,
                passed: step.passed(),
                tests_ran: None,
                duration: Quantity::measured(step.duration_s, Unit::Seconds, "recorded result"),
                output_tail: String::new(),
                note: Some(format!("result reused from the {origin} for {sha}")),
                failed_tests: Vec::new(),
                reused: true,
            })
        })
        .collect()
}

/// Where a check runs, and under what limits.
#[derive(Debug)]
pub struct CheckContext<'a> {
    pub dir: &'a Path,
    pub env: &'a BTreeMap<String, String>,
    pub timeout: Duration,
    pub registry: &'a Registry,
    pub job: &'a str,
    pub redactor: &'a Redactor,
}

/// The first wait between polls of a running check.
const POLL_START: Duration = Duration::from_millis(5);
/// The longest wait between polls.
const POLL_CAP: Duration = Duration::from_millis(200);

/// The note of a check killed at its deadline. [`run_check`] writes it and [`timed_out`] reads it, so the two
/// cannot drift apart.
pub fn deadline_note(limit_s: u64) -> String {
    format!("{DEADLINE_NOTE_START}{limit_s}s")
}

/// What every deadline note starts with.
const DEADLINE_NOTE_START: &str = "killed at the deadline of ";

/// True when `run` was killed at its deadline: not passed, no exit code, and the deadline note (which a
/// `pre-existing` prefix does not hide). A check killed by the kill switch or one that ran zero tests is not.
pub fn timed_out(run: &CheckRun) -> bool {
    !run.passed
        && run.exit_code.is_none()
        && run
            .note
            .as_deref()
            .is_some_and(|n| n.contains(DEADLINE_NOTE_START))
}

/// The check that holds the job for a timeout, if one does. Only new failures count (failed and not
/// pre-existing on the base): the hold applies when there is at least one and every one of them timed out.
/// A new failure that did not time out means the work failed for real, and the job goes on to be judged and
/// rejected. A pre-existing failure never counts either way, as in the verdict. Returns the longest-running
/// timed-out check.
pub fn held_for_timeout<'a>(
    checks: &'a [CheckRun],
    baseline: Option<&Baseline>,
) -> Option<&'a CheckRun> {
    let new_failures: Vec<&CheckRun> = checks
        .iter()
        .filter(|c| !c.passed && !crate::verify::is_pre_existing(c, baseline))
        .collect();
    if new_failures.is_empty() || !new_failures.iter().all(|c| timed_out(c)) {
        return None;
    }
    new_failures.into_iter().max_by(|a, b| {
        let secs = |c: &CheckRun| c.duration.value().unwrap_or(0.0);
        secs(a).total_cmp(&secs(b))
    })
}

/// Runs one check.
pub fn run_check(spec: &CheckSpec, ctx: &CheckContext<'_>) -> Result<CheckRun> {
    let started = Instant::now();
    let output_file = tempfile::NamedTempFile::new()?;
    let (program, args) = spec
        .run
        .split_first()
        .ok_or_else(|| anyhow::anyhow!("check {} has no command", spec.name))?;
    // One open file shared by both streams: two separate opens would each write from offset 0 and
    // overwrite each other, losing the harness's summary lines (acceptance verification 1, m2).
    let stdout = output_file.reopen()?;
    let stderr = stdout.try_clone()?;
    // A plain `cargo test` runs every binary (`--no-fail-fast`) so the failing tests of all of them are
    // listed; `command` in the record stays as configured, and a failed run says what was added.
    let (run_args, records_names) = if spec.kind == CheckKind::Test {
        cargo_test_args(program, args)
    } else {
        (args.to_vec(), false)
    };
    let flag_added = run_args.len() != args.len();
    let mut cmd = Command::new(program);
    cmd.args(&run_args)
        .env_clear()
        .envs(ctx.env)
        .current_dir(ctx.dir)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr));
    isolate(&mut cmd);
    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            return Ok(CheckRun {
                name: spec.name.clone(),
                command: spec.run.clone(),
                exit_code: None,
                passed: false,
                tests_ran: None,
                duration: Quantity::measured(
                    started.elapsed().as_secs_f64(),
                    Unit::Seconds,
                    "athena monotonic clock",
                ),
                output_tail: String::new(),
                note: Some(format!("could not start {program}: {e}")),
                failed_tests: Vec::new(),
                reused: false,
            });
        }
    };
    let pgid = child.id() as i32;
    ctx.registry
        .register(pgid, ctx.job, &format!("check {}", spec.name))?;
    let deadline = started + ctx.timeout;
    let mut note = None;
    // Poll after 5 ms at first, doubling up to the old 200 ms, so a short check is noticed sooner.
    let mut delay = POLL_START;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break Some(status);
        }
        if Instant::now() >= deadline {
            note = Some(deadline_note(ctx.timeout.as_secs()));
        } else if ctx.registry.stopped() {
            note = Some("killed by the kill switch".to_string());
        }
        if note.is_some() {
            kill_group(pgid, GRACE);
            let _ = child.wait();
            break None;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        std::thread::sleep(delay.min(remaining));
        delay = (delay * 2).min(POLL_CAP);
    };
    ctx.registry.unregister(pgid);
    let output = String::from_utf8_lossy(&std::fs::read(output_file.path())?).into_owned();
    let exit_code = status.and_then(|s| s.code());
    let tests_ran = (spec.kind == CheckKind::Test).then(|| count_tests(&output).unwrap_or(0));
    let mut passed = exit_code == Some(0);
    if spec.kind == CheckKind::Test && tests_ran == Some(0) {
        passed = false;
        note.get_or_insert_with(|| {
            "ran zero tests; a test check must run at least one".to_string()
        });
    }
    // Names only from a process that exited with a code: a killed run has a partial output.
    let mut failed_tests = Vec::new();
    if records_names && exit_code.is_some() {
        for name in failed_test_names(&output) {
            failed_tests.push(ctx.redactor.redact(&name)?);
        }
    }
    if flag_added && !passed {
        note = Some(match note {
            Some(n) => format!("{n}; {NO_FAIL_FAST_NOTE}"),
            None => NO_FAIL_FAST_NOTE.to_string(),
        });
    }
    // A passing run keeps a short byte tail. A failed run keeps its last lines, whole, because the
    // failing test's name and message are what a retry or a person needs.
    let kept = if passed {
        tail(&output, TAIL_BYTES)
    } else {
        tail(&tail_lines(&output, FAILED_TAIL_LINES), FAILED_TAIL_BYTES)
    };
    Ok(CheckRun {
        name: spec.name.clone(),
        command: spec.run.clone(),
        exit_code,
        passed,
        tests_ran,
        duration: Quantity::measured(
            started.elapsed().as_secs_f64(),
            Unit::Seconds,
            "athena monotonic clock",
        ),
        output_tail: ctx.redactor.redact(&kept)?,
        note,
        failed_tests,
        reused: false,
    })
}

/// Runs every spec on the base commit for the job's baseline record. Unlike [`run_check`] it never fails:
/// a check that cannot be run at all (an error from `run_check`) is recorded as not passed, with a note.
/// A check that fails, times out or cannot start is already recorded that way by `run_check`. Every
/// spec runs, whatever the earlier ones did, and `skip_when_only` is not applied, since no change list
/// exists yet. The result is recorded only; nothing here judges a candidate.
pub fn baseline_runs(specs: &[CheckSpec], ctx: &CheckContext<'_>) -> Vec<CheckRun> {
    specs
        .iter()
        .map(|spec| match run_check(spec, ctx) {
            Ok(run) => run,
            Err(e) => CheckRun {
                name: spec.name.clone(),
                command: spec.run.clone(),
                exit_code: None,
                passed: false,
                tests_ran: None,
                duration: Quantity::measured(0.0, Unit::Seconds, "not run"),
                output_tail: String::new(),
                note: Some(format!("baseline check could not run: {e}")),
                failed_tests: Vec::new(),
                reused: false,
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sh_spec(name: &str, script: &str) -> CheckSpec {
        CheckSpec {
            name: name.into(),
            run: vec!["sh".into(), "-c".into(), script.into()],
            kind: CheckKind::Command,
            skip_when_only: None,
        }
    }

    #[test]
    fn baseline_records_each_check_whatever_the_others_did() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let registry = Registry::new(state.path());
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let env = BTreeMap::from([("PATH".to_string(), std::env::var("PATH").unwrap())]);
        let ctx = CheckContext {
            dir: dir.path(),
            env: &env,
            timeout: Duration::from_secs(30),
            registry: &registry,
            job: "j",
            redactor: &redactor,
        };
        let specs = [
            sh_spec("bad", "exit 3"),
            sh_spec("good", "exit 0"),
            CheckSpec {
                name: "missing".into(),
                run: vec!["/nonexistent/athena-baseline-check".into()],
                kind: CheckKind::Command,
                skip_when_only: Some(vec!["*.md".into()]),
            },
            CheckSpec {
                name: "empty".into(),
                run: vec![],
                kind: CheckKind::Command,
                skip_when_only: None,
            },
        ];
        let runs = baseline_runs(&specs, &ctx);
        assert_eq!(runs.len(), 4);
        assert!(!runs[0].passed && runs[0].exit_code == Some(3));
        assert!(runs[1].passed && runs[1].exit_code == Some(0));
        assert!(!runs[2].passed);
        assert!(runs[2].note.as_ref().unwrap().contains("could not start"));
        assert!(!runs[3].passed);
        assert!(runs[3].note.as_ref().unwrap().contains("could not run"));
    }

    #[test]
    fn baseline_records_a_timeout_as_failed_with_a_note() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let registry = Registry::new(state.path());
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let env = BTreeMap::from([("PATH".to_string(), std::env::var("PATH").unwrap())]);
        let ctx = CheckContext {
            dir: dir.path(),
            env: &env,
            timeout: Duration::from_millis(300),
            registry: &registry,
            job: "j",
            redactor: &redactor,
        };
        let runs = baseline_runs(&[sh_spec("slow", "sleep 30")], &ctx);
        assert!(!runs[0].passed);
        assert!(runs[0].note.as_ref().unwrap().contains("deadline"));
    }

    fn run_of(passed: bool, tail: &str, failed_tests: &[&str]) -> CheckRun {
        CheckRun {
            name: "probe".into(),
            command: vec!["x".into()],
            exit_code: Some(i32::from(!passed)),
            passed,
            tests_ran: None,
            duration: Quantity::measured(0.0, Unit::Seconds, "test"),
            output_tail: tail.into(),
            note: None,
            failed_tests: failed_tests.iter().map(|s| s.to_string()).collect(),
            reused: false,
        }
    }

    fn run_script(spec: &CheckSpec) -> CheckRun {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let registry = Registry::new(state.path());
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let env = BTreeMap::from([("PATH".to_string(), std::env::var("PATH").unwrap())]);
        let ctx = CheckContext {
            dir: dir.path(),
            env: &env,
            timeout: Duration::from_secs(30),
            registry: &registry,
            job: "j",
            redactor: &redactor,
        };
        run_check(spec, &ctx).unwrap()
    }

    #[test]
    fn tail_lines_keeps_the_last_n_lines_and_the_final_newline() {
        assert_eq!(tail_lines("a\nb\nc\nd\n", 2), "c\nd\n");
        assert_eq!(tail_lines("a\nb\nc\nd", 2), "c\nd");
        assert_eq!(tail_lines("a\nb\n", 5), "a\nb\n");
        assert_eq!(tail_lines("", 3), "");
    }

    #[test]
    fn a_failed_run_keeps_its_last_200_lines_and_a_passing_run_its_byte_tail() {
        let script = "i=1; while [ $i -le 600 ]; do echo \"line $i\"; i=$((i+1)); done";
        let failed = run_script(&sh_spec("noisy", &format!("{script}; exit 1")));
        let lines: Vec<&str> = failed.output_tail.lines().collect();
        assert_eq!(lines.len(), 200);
        assert_eq!(lines[0], "line 401");
        assert_eq!(lines[199], "line 600");
        // The same output from a run that passes keeps only the byte tail.
        let passed = run_script(&sh_spec("noisy", script));
        assert!(passed.output_tail.len() <= TAIL_BYTES + 3);
        assert!(
            passed.output_tail.starts_with('…'),
            "cut by bytes, not lines"
        );
    }

    #[test]
    fn a_failed_runs_kept_lines_stop_at_the_byte_ceiling() {
        let script = "i=0; while [ $i -lt 150 ]; do printf '%0900d\\n' 0; i=$((i+1)); done; \
                      echo the-end; exit 1";
        let failed = run_script(&sh_spec("wide", script));
        assert!(failed.output_tail.len() <= FAILED_TAIL_BYTES + 3);
        assert!(failed.output_tail.starts_with('…'));
        assert!(failed.output_tail.trim_end().ends_with("the-end"));
    }

    #[test]
    fn the_log_entry_names_the_failing_tests() {
        assert_eq!(log_entry(&run_of(true, "", &[])), "probe ok");
        let mut ran = run_of(true, "", &[]);
        ran.tests_ran = Some(7);
        assert_eq!(log_entry(&ran), "probe ok (7 tests)");
        // Recorded names come first.
        assert_eq!(
            log_entry(&run_of(
                false,
                "test other ... FAILED\n",
                &["bin a::b", "bin c"]
            )),
            "probe FAILED: bin a::b, bin c"
        );
        // Without recorded names, the harness's FAILED lines give them; a repeated name shows once.
        let out = "test a::ok ... ok\ntest a::bad ... FAILED\ntest a::bad ... FAILED\n\
                   test result: FAILED. 1 passed; 1 failed\n";
        assert_eq!(log_entry(&run_of(false, out, &[])), "probe FAILED: a::bad");
        // No name anywhere: the check name alone.
        assert_eq!(log_entry(&run_of(false, "boom\n", &[])), "probe FAILED");
    }

    #[test]
    fn the_log_entry_counts_names_beyond_five() {
        let names = ["t1", "t2", "t3", "t4", "t5", "t6", "t7"];
        assert_eq!(
            log_entry(&run_of(false, "", &names)),
            "probe FAILED: t1, t2, t3, t4, t5, +2 more"
        );
    }

    /// Two binaries, each with one failing test, as cargo prints them under `--no-fail-fast`.
    const TWO_BINARIES: &str = "\
   Compiling athena-core v0.1.0 (/w/crates/athena-core)
     Running unittests src/lib.rs (target/debug/deps/athena_core-0f3a9c1d2b4e5f60)

running 2 tests
test checks::tests::ok ... ok
test checks::tests::x ... FAILED

failures:

---- checks::tests::x stdout ----
thread 'checks::tests::x' panicked at src/checks.rs:9:5:
boom


failures:
    checks::tests::x

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-core --lib`
     Running tests/policy.rs (target/debug/deps/policy-1122334455667788)

running 1 test
test scan ... FAILED

failures:

---- scan stdout ----
panicked

failures:
    scan

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

";

    #[test]
    fn failing_tests_are_read_from_each_failures_list_and_qualified_by_binary() {
        assert_eq!(
            failed_test_names(TWO_BINARIES),
            ["athena_core checks::tests::x", "policy scan"]
        );
    }

    #[test]
    fn a_doc_test_failure_is_named_under_its_crate() {
        let output = "\
   Doc-tests athena_core

running 2 tests
test src/lib.rs - f (line 3) ... ok
test src/lib.rs - g (line 9) ... FAILED

failures:

---- src/lib.rs - g (line 9) stdout ----
Test executable failed (exit status: 101).

failures:
    src/lib.rs - g (line 9)

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
";
        assert_eq!(
            failed_test_names(output),
            ["doc-tests athena_core src/lib.rs - g (line 9)"]
        );
    }

    #[test]
    fn output_with_no_failures_or_no_sections_names_nothing() {
        let passing = "     Running unittests src/lib.rs (target/debug/deps/a-00ff)\n\n\
                       running 1 test\ntest t ... ok\n\n\
                       test result: ok. 1 passed; 0 failed; 0 ignored\n";
        assert!(failed_test_names(passing).is_empty());
        assert!(failed_test_names("").is_empty());
        // A build error prints no section at all, and a bare failures block before any section is not read.
        let build_error = "error[E0432]: unresolved import\nerror: could not compile `x`\n\
                           failures:\n    stray\n";
        assert!(failed_test_names(build_error).is_empty());
    }

    #[test]
    fn a_crashed_binary_is_marked_as_not_finished() {
        // The second binary printed its header and `running` line, then died: no list, no result line.
        let output = "\
     Running unittests src/lib.rs (target/debug/deps/a-00ff)

running 1 test
test x ... FAILED

failures:

---- x stdout ----
boom

failures:
    x

test result: FAILED. 0 passed; 1 failed; 0 ignored
     Running tests/crashy.rs (target/debug/deps/crashy-aa11)

running 1 test
test boom ... ";
        assert_eq!(
            failed_test_names(output),
            ["a x", "crashy (did not finish)"]
        );
    }

    #[test]
    fn a_list_that_disagrees_with_the_failed_count_is_marked_as_not_finished() {
        // Only the stdout block, no `failures:` list: 1 failed, 0 listed.
        let output = "\
     Running tests/t.rs (target/debug/deps/t-aa11)

running 1 test
test x ... FAILED

failures:

---- x stdout ----
boom

test result: FAILED. 0 passed; 1 failed; 0 ignored
";
        assert_eq!(failed_test_names(output), ["t (did not finish)"]);
    }

    #[test]
    fn the_flag_goes_right_after_test_and_only_for_a_plain_cargo_test() {
        let v = |s: &[&str]| s.iter().map(|a| a.to_string()).collect::<Vec<_>>();
        let (args, records) = cargo_test_args("cargo", &v(&["test", "--locked"]));
        assert_eq!(args, v(&["test", "--no-fail-fast", "--locked"]));
        assert!(records);
        let (args, _) = cargo_test_args("/usr/bin/cargo", &v(&["test", "--", "--nocapture"]));
        assert_eq!(args, v(&["test", "--no-fail-fast", "--", "--nocapture"]));
        let given = v(&["test", "--no-fail-fast"]);
        assert_eq!(cargo_test_args("cargo", &given), (given.clone(), true));
        for (program, args) in [
            ("cargo", v(&["test", "--fail-fast"])),
            ("cargo", v(&["+nightly", "test"])),
            ("cargo", v(&["nextest", "run"])),
            ("make", v(&["test"])),
            ("sh", v(&["-c", "cargo test"])),
        ] {
            assert_eq!(
                cargo_test_args(program, &args),
                (args.clone(), false),
                "{program} {args:?}"
            );
        }
        // A `--fail-fast` after `--` belongs to the test binary, not to cargo.
        let (_, records) = cargo_test_args("cargo", &v(&["test", "--", "--fail-fast"]));
        assert!(records);
    }

    fn check_ctx<'a>(
        dir: &'a Path,
        env: &'a BTreeMap<String, String>,
        registry: &'a Registry,
        redactor: &'a Redactor,
    ) -> CheckContext<'a> {
        CheckContext {
            dir,
            env,
            timeout: Duration::from_secs(30),
            registry,
            job: "j",
            redactor,
        }
    }

    #[test]
    fn a_cargo_test_check_records_its_failing_tests_and_the_flag_it_ran_with() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let bin = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        // A script named `cargo` that echoes its arguments, prints canned failing output and exits 101.
        let script = bin.path().join("cargo");
        std::fs::write(
            &script,
            format!("#!/bin/sh\necho \"args: $*\"\ncat <<'EOF'\n{TWO_BINARIES}EOF\nexit 101\n"),
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let registry = Registry::new(state.path());
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let env = BTreeMap::from([("PATH".to_string(), std::env::var("PATH").unwrap())]);
        let ctx = check_ctx(dir.path(), &env, &registry, &redactor);
        let spec = CheckSpec {
            name: "test".into(),
            run: vec![
                script.to_string_lossy().into_owned(),
                "test".into(),
                "--locked".into(),
            ],
            kind: CheckKind::Test,
            skip_when_only: None,
        };
        let run = run_check(&spec, &ctx).unwrap();
        assert!(!run.passed && run.exit_code == Some(101));
        assert_eq!(
            run.failed_tests,
            ["athena_core checks::tests::x", "policy scan"]
        );
        assert!(
            run.output_tail
                .contains("args: test --no-fail-fast --locked")
        );
        assert_eq!(
            run.command, spec.run,
            "the record keeps the configured command"
        );
        assert!(run.note.as_deref().unwrap().contains("--no-fail-fast"));
        assert!(!timed_out(&run));

        // The same output from a command that is not cargo records no names and no flag note.
        let sh = CheckSpec {
            name: "test".into(),
            run: vec![
                "sh".into(),
                "-c".into(),
                format!("cat <<'EOF'\n{TWO_BINARIES}EOF\nexit 101"),
            ],
            kind: CheckKind::Test,
            skip_when_only: None,
        };
        let run = run_check(&sh, &ctx).unwrap();
        assert!(!run.passed);
        assert!(run.failed_tests.is_empty());
        assert_eq!(run.note, None);
    }

    #[test]
    fn tests_are_counted_from_the_harness_summary() {
        let output = "test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n\
                      test result: FAILED. 2 passed; 1 failed; 0 ignored\n\
                      test result: ok. 0 passed; 0 failed; 0 ignored\n";
        assert_eq!(count_tests(output), Some(34));
        assert_eq!(count_tests("Finished. no tests here"), None);
    }

    #[test]
    fn a_test_check_that_ran_zero_tests_fails() {
        // M12 (blocks): a test check that ran zero tests fails, even with exit status 0.
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let registry = Registry::new(state.path());
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let env = BTreeMap::from([("PATH".to_string(), std::env::var("PATH").unwrap())]);
        let ctx = CheckContext {
            dir: dir.path(),
            env: &env,
            timeout: Duration::from_secs(30),
            registry: &registry,
            job: "j",
            redactor: &redactor,
        };
        let empty = CheckSpec {
            name: "test".into(),
            run: vec![
                "sh".into(),
                "-c".into(),
                "echo 'test result: ok. 0 passed; 0 failed; 0 ignored'".into(),
            ],
            kind: CheckKind::Test,
            skip_when_only: None,
        };
        let run = run_check(&empty, &ctx).unwrap();
        assert_eq!(run.exit_code, Some(0));
        assert!(!run.passed);
        assert_eq!(run.tests_ran, Some(0));

        let real = CheckSpec {
            name: "test".into(),
            run: vec![
                "sh".into(),
                "-c".into(),
                "echo 'test result: ok. 3 passed; 0 failed; 0 ignored'".into(),
            ],
            kind: CheckKind::Test,
            skip_when_only: None,
        };
        assert!(run_check(&real, &ctx).unwrap().passed);
    }

    #[test]
    fn test_counts_survive_output_on_both_streams() {
        // m2 of acceptance verification 1: with stdout and stderr interleaved, every summary line counts.
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let registry = Registry::new(state.path());
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let env = BTreeMap::from([("PATH".to_string(), std::env::var("PATH").unwrap())]);
        let ctx = CheckContext {
            dir: dir.path(),
            env: &env,
            timeout: Duration::from_secs(30),
            registry: &registry,
            job: "j",
            redactor: &redactor,
        };
        let noisy = CheckSpec {
            name: "test".into(),
            run: vec![
                "sh".into(),
                "-c".into(),
                "i=0; while [ $i -lt 200 ]; do echo '   Compiling something on stderr' >&2; \
                 echo 'test result: ok. 1 passed; 0 failed; 0 ignored'; i=$((i+1)); done"
                    .into(),
            ],
            kind: CheckKind::Test,
            skip_when_only: None,
        };
        let run = run_check(&noisy, &ctx).unwrap();
        assert_eq!(run.tests_ran, Some(200));
        assert!(run.passed);
    }

    #[test]
    fn a_check_past_its_deadline_is_killed() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let registry = Registry::new(state.path());
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let env = BTreeMap::from([("PATH".to_string(), std::env::var("PATH").unwrap())]);
        let ctx = CheckContext {
            dir: dir.path(),
            env: &env,
            timeout: Duration::from_millis(300),
            registry: &registry,
            job: "j",
            redactor: &redactor,
        };
        let slow = CheckSpec {
            name: "slow".into(),
            run: vec!["sleep".into(), "30".into()],
            kind: CheckKind::Command,
            skip_when_only: None,
        };
        let run = run_check(&slow, &ctx).unwrap();
        assert!(!run.passed);
        assert!(run.note.unwrap().contains("deadline"));
    }

    #[test]
    fn a_real_deadline_kill_is_classified_as_a_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let registry = Registry::new(state.path());
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let env = BTreeMap::from([("PATH".to_string(), std::env::var("PATH").unwrap())]);
        let ctx = |timeout| CheckContext {
            dir: dir.path(),
            env: &env,
            timeout,
            registry: &registry,
            job: "j",
            redactor: &redactor,
        };
        let killed = run_check(
            &sh_spec("slow", "sleep 30"),
            &ctx(Duration::from_millis(300)),
        )
        .unwrap();
        assert!(timed_out(&killed));
        let failed = run_check(&sh_spec("bad", "exit 1"), &ctx(Duration::from_secs(30))).unwrap();
        assert!(!timed_out(&failed));
    }

    fn fake_run(
        name: &str,
        passed: bool,
        exit_code: Option<i32>,
        note: Option<String>,
    ) -> CheckRun {
        CheckRun {
            name: name.into(),
            command: vec!["x".into()],
            exit_code,
            passed,
            tests_ran: None,
            duration: Quantity::measured(10.0, Unit::Seconds, "athena monotonic clock"),
            output_tail: String::new(),
            note,
            failed_tests: Vec::new(),
            reused: false,
        }
    }

    fn timeout_run(name: &str) -> CheckRun {
        fake_run(name, false, None, Some(deadline_note(900)))
    }

    fn real_failure(name: &str) -> CheckRun {
        fake_run(name, false, Some(1), None)
    }

    fn passing(name: &str) -> CheckRun {
        fake_run(name, true, Some(0), None)
    }

    fn baseline_failing(names: &[&str]) -> Baseline {
        Baseline {
            job: athena_data::JobId::parse("20260101-000000-aaaa").unwrap(),
            base_commit: "0".repeat(40),
            before_invocation: 1,
            checks: names.iter().map(|n| real_failure(n)).collect(),
            note: None,
        }
    }

    #[test]
    fn only_a_deadline_kill_reads_as_a_timeout() {
        assert!(timed_out(&timeout_run("test")));
        let marked = fake_run(
            "test",
            false,
            None,
            Some(format!(
                "{}; {}",
                athena_data::records::PRE_EXISTING_NOTE,
                deadline_note(900)
            )),
        );
        assert!(timed_out(&marked), "a pre-existing prefix does not hide it");
        let switched = fake_run(
            "test",
            false,
            None,
            Some("killed by the kill switch".into()),
        );
        assert!(!timed_out(&switched));
        let zero = fake_run(
            "test",
            false,
            Some(0),
            Some("ran zero tests; a test check must run at least one".into()),
        );
        assert!(!timed_out(&zero));
        assert!(!timed_out(&real_failure("test")));
        assert!(!timed_out(&passing("test")));
    }

    #[test]
    fn held_for_timeout_needs_a_new_failure_and_every_new_failure_timed_out() {
        // A timeout alone is held.
        let only = [passing("fmt"), timeout_run("test")];
        assert_eq!(held_for_timeout(&only, None).unwrap().name, "test");
        // Nothing failed: nothing to hold.
        assert!(held_for_timeout(&[passing("fmt"), passing("test")], None).is_none());
        // A real failure alone, or beside a timeout in either order, is rejected as today.
        assert!(held_for_timeout(&[real_failure("test")], None).is_none());
        assert!(held_for_timeout(&[timeout_run("test"), real_failure("clippy")], None).is_none());
        assert!(held_for_timeout(&[real_failure("clippy"), timeout_run("test")], None).is_none());
        // A failure that was not a deadline kill is not a timeout.
        let switched = fake_run(
            "test",
            false,
            None,
            Some("killed by the kill switch".into()),
        );
        assert!(held_for_timeout(&[switched], None).is_none());
        // Pre-existing failures are judged by the baseline, as the verdict does.
        let base = baseline_failing(&["test"]);
        assert!(
            held_for_timeout(&[timeout_run("test")], Some(&base)).is_none(),
            "a timeout that also failed on the base passes today, so it is not held"
        );
        let base = baseline_failing(&["clippy"]);
        assert!(
            held_for_timeout(&[real_failure("clippy"), timeout_run("test")], Some(&base)).is_some(),
            "a pre-existing real failure beside a new timeout does not count as a real failure"
        );
        let base = baseline_failing(&["test"]);
        assert!(
            held_for_timeout(&[timeout_run("test"), real_failure("clippy")], Some(&base)).is_none(),
            "a pre-existing timeout beside a new real failure is rejected"
        );
    }

    #[test]
    fn the_longest_timed_out_check_is_the_one_held() {
        let mut long = timeout_run("test");
        long.duration = Quantity::measured(905.0, Unit::Seconds, "athena monotonic clock");
        let runs = [timeout_run("clippy"), long];
        assert_eq!(held_for_timeout(&runs, None).unwrap().name, "test");
    }

    #[test]
    fn the_poll_schedule_detects_fast_and_slow_completion_and_a_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let registry = Registry::new(state.path());
        let redactor = Redactor::new(&athena_gate::builtin_config()).unwrap();
        let env = BTreeMap::from([("PATH".to_string(), std::env::var("PATH").unwrap())]);
        let ctx = |timeout| CheckContext {
            dir: dir.path(),
            env: &env,
            timeout,
            registry: &registry,
            job: "j",
            redactor: &redactor,
        };
        let long = Duration::from_secs(30);

        let fast = run_check(&sh_spec("fast", "exit 0"), &ctx(long)).unwrap();
        assert!(fast.passed && fast.exit_code == Some(0) && fast.note.is_none());

        let slow = run_check(&sh_spec("slow", "sleep 1"), &ctx(long)).unwrap();
        assert!(slow.passed && slow.exit_code == Some(0) && slow.note.is_none());

        let started = Instant::now();
        let hung = run_check(
            &sh_spec("hung", "sleep 30"),
            &ctx(Duration::from_millis(300)),
        )
        .unwrap();
        assert!(!hung.passed && hung.exit_code.is_none());
        assert!(hung.note.unwrap().contains("deadline"));
        assert!(started.elapsed() < Duration::from_secs(20));
    }

    #[test]
    fn a_record_for_the_same_sha_stands_in_for_the_checks_and_another_sha_does_not() {
        use crate::ci_result::Step;
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let record = CiRecord {
            source: Source::Ci,
            sha: sha.into(),
            run_address: Some("https://example.test/runs/9".into()),
            steps: vec![
                Step {
                    name: "fmt".into(),
                    conclusion: "success".into(),
                    duration_s: 1.0,
                },
                Step {
                    name: "test".into(),
                    conclusion: "failure".into(),
                    duration_s: 3.0,
                },
            ],
            duration_s: 4.0,
        };
        let specs = vec![
            CheckSpec {
                name: "fmt".into(),
                ..spec_skipping(None)
            },
            CheckSpec {
                name: "test".into(),
                ..spec_skipping(None)
            },
            CheckSpec {
                name: "clippy".into(),
                ..spec_skipping(None)
            },
        ];
        let runs = runs_from_record(&specs, &record, sha);
        assert_eq!(
            runs.len(),
            2,
            "clippy is not in the record, so it still runs"
        );
        assert!(runs[0].passed && !runs[1].passed);
        assert!(runs[0].note.as_ref().unwrap().contains("runs/9"));
        assert!(
            runs.iter().all(|r| r.reused && r.exit_code.is_none()),
            "every record built from a CI result is marked reused"
        );
        assert!(!skipped_run(&specs[0], "not run: x".into()).reused);
        let other = "fedcba9876543210fedcba9876543210fedcba98";
        assert!(runs_from_record(&specs, &record, other).is_empty());
    }

    fn spec_skipping(patterns: Option<&[&str]>) -> CheckSpec {
        CheckSpec {
            name: "test".into(),
            run: vec!["true".into()],
            kind: CheckKind::Command,
            skip_when_only: patterns.map(|p| p.iter().map(|s| s.to_string()).collect()),
        }
    }

    fn paths(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn path_patterns_match_by_directory_suffix_or_exact_path() {
        assert!(path_matches("docs/", "docs/a.md"));
        assert!(path_matches("docs/", "docs/x/b.md"));
        assert!(!path_matches("docs/", "docsx/a.md"));
        assert!(path_matches("*.json", "a.json"));
        assert!(path_matches("*.json", "studies/x/y/a.json"));
        assert!(!path_matches("*.json", "a.jsonl"));
        assert!(!path_matches("*", "anything"));
        assert!(path_matches("CHANGELOG.md", "CHANGELOG.md"));
        assert!(!path_matches("CHANGELOG.md", "docs/CHANGELOG.md"));
    }

    #[test]
    fn a_check_without_a_skip_list_always_runs() {
        assert_eq!(
            skip_note(&spec_skipping(None), &paths(&["docs/a.md"])),
            None
        );
        assert_eq!(
            skip_note(&spec_skipping(Some(&[])), &paths(&["docs/a.md"])),
            None
        );
    }

    #[test]
    fn a_docs_only_change_skips_and_says_so() {
        let spec = spec_skipping(Some(&["docs/", "*.md"]));
        let note = skip_note(&spec, &paths(&["docs/a.md", "README.md"])).unwrap();
        assert!(note.contains("not run"));
        let run = skipped_run(&spec, note);
        assert!(run.passed);
        assert!(run.note.is_some());
    }

    #[test]
    fn a_studies_json_only_change_runs_every_check() {
        // With no skip list configured, nothing is skipped, whatever the change.
        let change = paths(&["studies/x/claims.json"]);
        assert_eq!(skip_note(&spec_skipping(None), &change), None);
        // With a list that names only docs, the same change still runs.
        assert_eq!(skip_note(&spec_skipping(Some(&["docs/"])), &change), None);
    }

    #[test]
    fn an_empty_or_mixed_change_runs_the_check() {
        let spec = spec_skipping(Some(&["docs/", "*.json"]));
        assert_eq!(skip_note(&spec, &[]), None);
        assert_eq!(skip_note(&spec, &paths(&["docs/a.md", "src/lib.rs"])), None);
        assert!(skip_note(&spec, &paths(&["docs/a.md", "x/b.json"])).is_some());
    }
}
