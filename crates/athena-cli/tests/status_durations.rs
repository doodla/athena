//! `athena status` reports how long the `test` check took over the newest jobs, against the limit a check is
//! killed at, on seeded verification records in a temporary repository. No model is called.

mod common;

use athena_data::JobId;
use athena_data::records::{AnyRecord, CheckRun, Verdict, Verification};
use athena_data::{Quantity, Unit};
use common::*;

fn test_run(seconds: f64) -> CheckRun {
    CheckRun {
        name: "test".into(),
        command: vec!["cargo".into(), "test".into()],
        exit_code: Some(0),
        passed: true,
        tests_ran: Some(10),
        duration: Quantity::measured(seconds, Unit::Seconds, "athena monotonic clock"),
        output_tail: String::new(),
        note: None,
        failed_tests: Vec::new(),
    }
}

/// Commits one verification record per entry of `seconds`, oldest job first.
fn commit_runs(fx: &Fixture, seconds: &[f64]) {
    for (n, s) in seconds.iter().enumerate() {
        let job = JobId::parse(&format!("20261026-1000{n:02}-aa{n:02}")).unwrap();
        let record = AnyRecord::Verification(Box::new(Verification {
            job,
            candidate_commit: "0".repeat(40),
            verified_fingerprint: String::new(),
            tool_config_from_base: vec![],
            checks: vec![test_run(*s)],
            check_cleanup: vec![],
            verifier_invocation: None,
            criteria: vec![],
            findings: vec![],
            request_reading_mismatch: None,
            agent_verdict: Some(Verdict::Pass),
            final_verdict: Verdict::Pass,
            reasons: vec![],
            inputs: vec![],
            summary: None,
        }));
        write(fx.root(), &[(&record.canonical_path(), &record.to_json())]);
    }
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run(["commit", "-q", "-m", "Add fixture verifications"])
        .unwrap();
}

#[test]
fn status_prints_the_check_durations_line() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    // No job ran a test check: the line is not printed.
    let none = athena_cli::display::status(&athena.status().unwrap());
    assert!(!none.contains("median"), "{none}");

    // Three jobs: the median is 600 s and the slowest 1700 s, over 80% of the configured limit.
    commit_runs(&fx, &[600.0, 1700.0, 100.0]);
    let view = athena.status().unwrap();
    let durations = view.check_durations.as_ref().expect("durations");
    assert_eq!(durations.jobs, 3);
    assert_eq!(durations.median_s, 600.0);
    assert_eq!(durations.slowest_s, 1700.0);
    assert_eq!(durations.slowest_job, "20261026-100001-aa01");
    let limit = athena_core::capability::JobConfig::at(&fx.git, &fx.head())
        .unwrap()
        .config
        .budget
        .check_timeout_s;
    assert_eq!(durations.limit_s, limit);
    assert_eq!(durations.flagged, 1700.0 > 0.8 * limit as f64);

    let text = athena_cli::display::status(&view);
    let line = text
        .lines()
        .find(|l| l.starts_with("checks     test:"))
        .unwrap_or_else(|| panic!("no durations line in:\n{text}"));
    assert_eq!(line, durations.text());
    assert!(line.contains("median 600s"), "{line}");
    assert!(
        line.contains("slowest 1700s (20261026-100001-aa01)"),
        "{line}"
    );
    assert!(
        line.contains(&format!("of {limit}s limit over last 3 jobs")),
        "{line}"
    );
}
