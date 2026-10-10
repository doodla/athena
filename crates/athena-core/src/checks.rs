//! Running the capability's checks on the candidate (design §7.2; M12, M13).
//!
//! Checks run in the verifier's snapshot, with the agent environment, the capability's environment, and
//! the nested-run marker, each in its own process group under the kill switch and a deadline. A check of
//! kind `test` fails if it ran zero tests, counted from the test harness's own summary lines, even when
//! its exit status is 0. Output tails are capped at 4 KiB and redacted.

use crate::capability::CheckSpec;
use anyhow::Result;
use athena_agent::procs::{GRACE, Registry, isolate, kill_group};
use athena_data::config::CheckKind;
use athena_data::records::CheckRun;
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
    let mut cmd = Command::new(program);
    cmd.args(args)
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
            });
        }
    };
    let pgid = child.id() as i32;
    ctx.registry
        .register(pgid, ctx.job, &format!("check {}", spec.name))?;
    let deadline = started + ctx.timeout;
    let mut note = None;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break Some(status);
        }
        if Instant::now() >= deadline {
            note = Some(format!(
                "killed at the deadline of {}s",
                ctx.timeout.as_secs()
            ));
        } else if ctx.registry.stopped() {
            note = Some("killed by the kill switch".to_string());
        }
        if note.is_some() {
            kill_group(pgid, GRACE);
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(200));
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
        output_tail: ctx.redactor.redact(&tail(&output, TAIL_BYTES))?,
        note,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

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
        };
        let run = run_check(&slow, &ctx).unwrap();
        assert!(!run.passed);
        assert!(run.note.unwrap().contains("deadline"));
    }
}
