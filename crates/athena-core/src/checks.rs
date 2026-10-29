//! Running the capability's checks on the candidate (design §7.2; M12, M13).
//!
//! Checks run in the verifier's snapshot, with the agent environment, the capability's environment, and
//! the nested-run marker, each in its own process group under the kill switch and a deadline. A check of
//! kind `test` fails if it ran zero tests, counted from the test harness's own summary lines, even when
//! its exit status is 0. Output tails are capped at 4 KiB and redacted. A check that names
//! `skip_when_only` patterns is skipped, with a note in its record, when every changed path matches one.

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
