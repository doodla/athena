//! Wall time per verified result (docs/design/ci-routing.md section 9), computed from job records.
//!
//! Reads `records/jobs/<month>/<job>/job.json` (for `dispatched_at`, `class_of_work`, `capability`) and the
//! job's `outcome*.json` files (for `state` and `at`) under a repository root. The measure is the elapsed time
//! from dispatch to the outcome record. Verified jobs (landed, or blocked after verification) are reported
//! apart from failed and rejected jobs, so dropping hard jobs cannot improve the number. Per-phase times are
//! not in the records, so none are computed.

use anyhow::{Context, Result, bail};
use jiff::Timestamp;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

/// The default sample size per side (design 9.2).
pub const DEFAULT_N: usize = 20;

pub const USAGE: &str = "cargo run --locked -p xtask -- walltime [--root <dir>] [--last <N>] [--capability <name>]\n       \
     wall time from dispatch to outcome per job kind, over the last N (default 20) finished jobs under <dir>/records/jobs";

/// How a finished job ended, for this measure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Verified,
    /// Failed or rejected: reported separately.
    Unverified,
}

/// One job with the facts this measure needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobTime {
    pub id: String,
    pub kind: String,
    pub dispatched: Timestamp,
    pub finished: Timestamp,
    pub class: Class,
}

impl JobTime {
    pub fn seconds(&self) -> f64 {
        self.finished
            .as_second()
            .saturating_sub(self.dispatched.as_second()) as f64
    }
}

/// What reading the records found, before the window is applied.
#[derive(Debug, Default)]
pub struct Found {
    /// Jobs that have a dispatch time and a counted outcome.
    pub jobs: Vec<JobTime>,
    /// Job directories read.
    pub job_dirs: usize,
    /// Jobs with a counted outcome but no usable `dispatched_at` or `at`.
    pub missing_timestamps: usize,
    /// Jobs with no outcome, or an outcome that is neither verified nor failed/rejected.
    pub other: usize,
}

fn children(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut out: Vec<_> = std::fs::read_dir(dir)
        .map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.path()).collect())
        .unwrap_or_default();
    out.sort();
    out
}

fn read_json(path: &Path) -> Result<Value> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

fn time(value: &Value, key: &str) -> Option<Timestamp> {
    value.get(key)?.as_str()?.parse().ok()
}

/// Reads the job records under `root`. A job counts under `capability` when it is `Some` and the job's
/// capability equals it.
pub fn read_jobs(root: &Path, capability: Option<&str>) -> Result<Found> {
    let mut found = Found::default();
    for month in children(&athena_data::paths::records_dir(root).join("jobs")) {
        for dir in children(&month).into_iter().filter(|d| d.is_dir()) {
            let job_file = dir.join("job.json");
            if !job_file.is_file() {
                continue;
            }
            let job = read_json(&job_file)?;
            if let Some(want) = capability
                && job.get("capability").and_then(Value::as_str) != Some(want)
            {
                continue;
            }
            found.job_dirs += 1;
            // The last outcome wins: a blocked job later lands, and `outcome-landed.json` is the later one.
            let mut last: Option<(Timestamp, Class)> = None;
            let mut counted = false;
            for file in children(&dir) {
                let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with("outcome") || !name.ends_with(".json") {
                    continue;
                }
                let outcome = read_json(&file)?;
                let class = match outcome.get("state").and_then(Value::as_str) {
                    Some("landed" | "blocked") => Class::Verified,
                    Some("failed" | "rejected") => Class::Unverified,
                    _ => continue,
                };
                counted = true;
                if let Some(at) = time(&outcome, "at")
                    && last.is_none_or(|(prev, _)| at >= prev)
                {
                    last = Some((at, class));
                }
            }
            let id = dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("?")
                .to_string();
            let kind = job
                .get("class_of_work")
                .and_then(Value::as_str)
                .filter(|k| !k.is_empty())
                .unwrap_or("unclassified")
                .to_string();
            match (counted, time(&job, "dispatched_at"), last) {
                (false, _, _) => found.other += 1,
                (true, Some(dispatched), Some((finished, class))) => found.jobs.push(JobTime {
                    id,
                    kind,
                    dispatched,
                    finished,
                    class,
                }),
                (true, _, _) => found.missing_timestamps += 1,
            }
        }
    }
    found
        .jobs
        .sort_by(|a, b| (a.dispatched, &a.id).cmp(&(b.dispatched, &b.id)));
    Ok(found)
}

/// Median; the mean of the two middle values when the count is even. `None` for no values.
pub fn median(sorted: &[f64]) -> Option<f64> {
    let n = sorted.len();
    match n {
        0 => None,
        _ if n % 2 == 1 => Some(sorted[n / 2]),
        _ => Some((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0),
    }
}

/// The nearest-rank 90th percentile: the value at rank ceil(0.9 n). `None` for no values.
pub fn p90(sorted: &[f64]) -> Option<f64> {
    let n = sorted.len();
    (n > 0).then(|| sorted[(n * 9).div_ceil(10) - 1])
}

fn duration(seconds: f64) -> String {
    let total = seconds.round() as i64;
    format!("{}m{:02}s", total / 60, total % 60)
}

fn stats(values: &mut [f64]) -> String {
    values.sort_by(f64::total_cmp);
    format!(
        "n={} median {} p90 {}",
        values.len(),
        duration(median(values).unwrap_or(0.0)),
        duration(p90(values).unwrap_or(0.0)),
    )
}

/// The report for the last `n` finished jobs of `found`. Never prints a figure for an empty set: it says why
/// there is none.
pub fn report(found: &Found, n: usize) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "wall time per verified result: dispatch to outcome record (seconds are measured from job records)"
    );
    let _ = writeln!(
        out,
        "job directories read: {}; without a usable dispatched_at or outcome time: {}; other outcomes (not verified, failed or rejected): {}",
        found.job_dirs, found.missing_timestamps, found.other
    );
    if found.jobs.is_empty() {
        let _ = if found.missing_timestamps > 0 {
            writeln!(
                out,
                "UNAVAILABLE: the records lack the timestamps (dispatched_at, outcome at) for {} finished job(s), and no job has both; no wall time is computed.",
                found.missing_timestamps
            )
        } else {
            writeln!(
                out,
                "UNAVAILABLE: no finished jobs with a dispatch time were found under records/jobs; no wall time is computed."
            )
        };
        return out;
    }
    let window = &found.jobs[found.jobs.len().saturating_sub(n)..];
    let _ = writeln!(
        out,
        "window: the last {} finished job(s) by dispatch time (N={n})",
        window.len()
    );
    if window.len() < n {
        let _ = writeln!(
            out,
            "CAVEAT: fewer than {n} jobs exist ({}); read these figures with that in mind, they are not a stable baseline.",
            window.len()
        );
    }
    let mut kinds: BTreeMap<&str, (Vec<f64>, Vec<f64>)> = BTreeMap::new();
    for job in window {
        let entry = kinds.entry(job.kind.as_str()).or_default();
        match job.class {
            Class::Verified => entry.0.push(job.seconds()),
            Class::Unverified => entry.1.push(job.seconds()),
        }
    }
    let (mut all_ok, mut all_bad) = (Vec::new(), Vec::new());
    let _ = writeln!(
        out,
        "\nverified results (landed or verified-then-blocked), by job kind:"
    );
    for (kind, (ok, _)) in &mut kinds {
        all_ok.extend(ok.iter().copied());
        let line = if ok.is_empty() {
            "none in the window".to_string()
        } else {
            stats(ok)
        };
        let _ = writeln!(out, "  {kind}: {line}");
    }
    let _ = writeln!(
        out,
        "  all kinds: {}",
        if all_ok.is_empty() {
            "none in the window".to_string()
        } else {
            stats(&mut all_ok)
        }
    );
    let _ = writeln!(
        out,
        "\nfailed or rejected jobs (time per job, reported separately), by job kind:"
    );
    for (kind, (_, bad)) in &mut kinds {
        all_bad.extend(bad.iter().copied());
        let line = if bad.is_empty() {
            "none in the window".to_string()
        } else {
            stats(bad)
        };
        let _ = writeln!(out, "  {kind}: {line}");
    }
    let _ = writeln!(
        out,
        "  all kinds: {}",
        if all_bad.is_empty() {
            "none in the window".to_string()
        } else {
            stats(&mut all_bad)
        }
    );
    out
}

/// `walltime [--root <dir>] [--last <N>] [--capability <name>]`.
pub fn command(args: &[String], default_root: &Path) -> Result<()> {
    // The records worktree stands in for the checkout once it exists (D111).
    let mut root = athena_data::paths::records_root(&default_root.join(".git"), default_root);
    let mut n = DEFAULT_N;
    let mut capability = None;
    let mut it = args.iter();
    while let Some(flag) = it.next() {
        let value = it.next();
        match (flag.as_str(), value) {
            ("--root", Some(v)) => root = v.into(),
            ("--last", Some(v)) => {
                n = v
                    .parse()
                    .ok()
                    .filter(|n| *n > 0)
                    .with_context(|| format!("--last needs a positive number, got {v}"))?
            }
            ("--capability", Some(v)) => capability = Some(v.clone()),
            _ => bail!("{USAGE}"),
        }
    }
    let found = read_jobs(&root, capability.as_deref())?;
    print!("{}", report(&found, n));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_and_p90_use_middle_and_nearest_rank() {
        assert_eq!(median(&[]), None);
        assert_eq!(median(&[1.0, 2.0, 9.0]), Some(2.0));
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), Some(2.5));
        let ten: Vec<f64> = (1..=10).map(f64::from).collect();
        assert_eq!(p90(&ten), Some(9.0));
        assert_eq!(p90(&[7.0]), Some(7.0));
        assert_eq!(p90(&[]), None);
    }

    fn write_job(root: &Path, id: &str) {
        let dir = root.join(format!("records/jobs/2026-10/{id}"));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("job.json"),
            r#"{"dispatched_at":"2026-10-11T10:00:00Z","class_of_work":"change"}"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("outcome-landed.json"),
            r#"{"state":"landed","at":"2026-10-11T10:05:00Z"}"#,
        )
        .unwrap();
    }

    #[test]
    fn the_reader_root_is_the_records_worktree_when_it_has_records() {
        let checkout = tempfile::tempdir().unwrap();
        std::fs::create_dir(checkout.path().join(".git")).unwrap();
        let common = checkout.path().join(".git");
        write_job(checkout.path(), "20261010-100000-aaaa");
        // No worktree: the checkout's own records/ is read.
        let root = athena_data::paths::records_root(&common, checkout.path());
        assert_eq!(root, checkout.path());
        let found = read_jobs(&root, None).unwrap();
        assert_eq!(found.jobs[0].id, "20261010-100000-aaaa");
        // A worktree with different records wins.
        let worktree = athena_data::paths::records_worktree(&common);
        write_job(&worktree, "20261011-100000-bbbb");
        let root = athena_data::paths::records_root(&common, checkout.path());
        let found = read_jobs(&root, None);
        let _ = std::fs::remove_dir_all(&worktree);
        let found = found.unwrap();
        assert_eq!(root, worktree);
        assert_eq!(found.jobs.len(), 1);
        assert_eq!(found.jobs[0].id, "20261011-100000-bbbb");
    }
}
