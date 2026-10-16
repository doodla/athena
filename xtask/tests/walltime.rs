//! `xtask walltime` on sample job records: it computes the median and p90 of dispatch-to-outcome time, keeps
//! failed and rejected jobs apart, says so when fewer than 20 jobs exist, and says the timestamps are missing
//! instead of printing a figure.

use std::path::Path;
use std::process::Command;

fn write(
    root: &Path,
    job: &str,
    kind: &str,
    dispatched: Option<&str>,
    outcome: Option<(&str, &str)>,
) {
    let dir = root.join("records/jobs/2026-10").join(job);
    std::fs::create_dir_all(&dir).unwrap();
    let mut record =
        serde_json::json!({ "id": job, "class_of_work": kind, "capability": "change" });
    if let Some(at) = dispatched {
        record["dispatched_at"] = at.into();
    }
    std::fs::write(dir.join("job.json"), record.to_string()).unwrap();
    if let Some((state, at)) = outcome {
        let body = serde_json::json!({ "job": job, "state": state, "at": at });
        std::fs::write(dir.join("outcome.json"), body.to_string()).unwrap();
    }
}

fn run(root: &Path, extra: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["walltime", "--root"])
        .arg(root)
        .args(extra)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// Minutes after 2026-10-01T00:00:00Z as a timestamp.
fn at(minutes: u32) -> String {
    format!("2026-10-01T{:02}:{:02}:00Z", minutes / 60, minutes % 60)
}

#[test]
fn median_and_p90_of_dispatch_to_verified_time() {
    let root = tempfile::tempdir().unwrap();
    // Durations 10, 20, 30, 40 minutes for code jobs: median 25, nearest-rank p90 40. A docs job takes 6.
    for (i, minutes) in [10, 20, 30, 40].into_iter().enumerate() {
        let start = i as u32 * 50;
        write(
            root.path(),
            &format!("code-{i}"),
            "code",
            Some(&at(start)),
            Some(("landed", &at(start + minutes))),
        );
    }
    write(
        root.path(),
        "docs-0",
        "docs",
        Some(&at(300)),
        Some(("landed", &at(306))),
    );
    let text = run(root.path(), &[]);
    assert!(
        text.contains("code: n=4 median 25m00s p90 40m00s"),
        "{text}"
    );
    assert!(text.contains("docs: n=1 median 6m00s p90 6m00s"), "{text}");
    assert!(
        text.contains("all kinds: n=5 median 20m00s p90 40m00s"),
        "{text}"
    );
}

#[test]
fn failed_and_rejected_jobs_are_reported_apart() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "a",
        "code",
        Some(&at(0)),
        Some(("landed", &at(10))),
    );
    write(
        root.path(),
        "b",
        "code",
        Some(&at(20)),
        Some(("failed", &at(120))),
    );
    write(
        root.path(),
        "c",
        "code",
        Some(&at(200)),
        Some(("rejected", &at(260))),
    );
    let text = run(root.path(), &[]);
    let (verified, unverified) = text.split_once("failed or rejected jobs").unwrap();
    assert!(verified.contains("code: n=1 median 10m00s"), "{text}");
    assert!(
        unverified.contains("code: n=2 median 80m00s p90 100m00s"),
        "{text}"
    );
}

#[test]
fn fewer_than_twenty_jobs_is_stated_and_twenty_is_not_flagged() {
    let root = tempfile::tempdir().unwrap();
    for i in 0..19u32 {
        write(
            root.path(),
            &format!("j{i:02}"),
            "code",
            Some(&at(i * 30)),
            Some(("landed", &at(i * 30 + 5))),
        );
    }
    assert!(run(root.path(), &[]).contains("CAVEAT: fewer than 20 jobs exist (19)"));
    write(
        root.path(),
        "j19",
        "code",
        Some(&at(19 * 30)),
        Some(("landed", &at(19 * 30 + 5))),
    );
    assert!(!run(root.path(), &[]).contains("CAVEAT"));
}

#[test]
fn the_window_is_the_last_n_jobs_by_dispatch() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "old",
        "code",
        Some(&at(0)),
        Some(("landed", &at(100))),
    );
    write(
        root.path(),
        "new",
        "code",
        Some(&at(200)),
        Some(("landed", &at(210))),
    );
    let text = run(root.path(), &["--last", "1"]);
    assert!(text.contains("code: n=1 median 10m00s"), "{text}");
}

#[test]
fn missing_timestamps_are_stated_not_printed_as_zero() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "a", "code", None, Some(("landed", &at(10))));
    let text = run(root.path(), &[]);
    assert!(
        text.contains("UNAVAILABLE: the records lack the timestamps"),
        "{text}"
    );
    assert!(!text.contains("median"), "{text}");
}

#[test]
fn no_records_is_stated() {
    let root = tempfile::tempdir().unwrap();
    let text = run(root.path(), &[]);
    assert!(text.contains("UNAVAILABLE: no finished jobs"), "{text}");
    assert!(!text.contains("median"), "{text}");
}
