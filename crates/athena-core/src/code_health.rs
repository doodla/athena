//! The code-health check: how big and how troubled each Rust source file is, and the work it queues when a
//! file crosses a limit.
//!
//! What is measured, per file under `crates/` (the checkout the instance runs in):
//! - **production lines**: lines that are not blank, not whole-line comments (`//` or `/* */`), and not inside an
//!   item gated by `#[cfg(test)]`. One definition, used by every number shown or compared;
//! - **longest function**: the most lines from a function's signature to its closing brace (nested functions
//!   count on their own and inside their parent), over non-test code. The source is parsed with `syn`, so
//!   braces in strings and comments cannot miscount (the call is recorded in `docs/decisions.md`);
//! - **weekly landings**: landed jobs in the last seven days whose candidate changed the file;
//! - **citations**: rejected or failed jobs whose findings, reasons or outcome text name the file's path.
//!
//! Limits (constants, see [`Thresholds`]): over 2000 production lines; a function over 200 lines; over 1500
//! lines with 5 or more landings in the week. A breach queues one request for a structural pass on that file,
//! unless an open request for it exists. Every tenth landing since the last design review queues a
//! self-contained, high-priority design-review request. Both are derived from the records and the open
//! requests, never from a counter, so running the check from two places for one job queues nothing twice. The
//! one stored number is the baseline: with no earlier review, the first check writes the current landing count
//! to the state directory, so existing history does not fire a review at once.
//!
//! A check that measured no file is reported as such, never as healthy.

use crate::store::{JobRecords, JobState, Records};
use crate::{Athena, Ui};
use anyhow::{Context, Result};
use athena_data::records::{Outcome, OutcomeState, Request};
use jiff::Timestamp;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

/// The limits. Constants, not configuration: they are enforced here and shown in the status table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Thresholds {
    /// A file over this many production lines is a breach.
    pub max_file_lines: usize,
    /// A function over this many lines is a breach.
    pub max_fn_lines: usize,
    /// A file over this many production lines with `busy_weekly_landings` landings in the week is a breach.
    pub busy_file_lines: usize,
    pub busy_weekly_landings: usize,
}

impl Thresholds {
    pub const CURRENT: Thresholds = Thresholds {
        max_file_lines: 2000,
        max_fn_lines: 200,
        busy_file_lines: 1500,
        busy_weekly_landings: 5,
    };
}

/// The window for "landings in a week", in days.
pub const WINDOW_DAYS: i64 = 7;

/// A design review is queued when this many jobs have landed since the last review.
pub const REVIEW_EVERY_LANDINGS: usize = 10;

/// The first line of a design-review request; earlier reviews are found by it.
pub const REVIEW_TITLE: &str = "Design review: code health and structure";

/// The words a structural-pass request starts its marker line with: `Structural pass: <path>`.
pub const STRUCTURAL_MARKER: &str = "Structural pass: ";

/// How many rows of the table are shown in status and in the review request (every file over a limit is
/// always shown).
const TABLE_ROWS: usize = 12;

/// One file's numbers.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FileHealth {
    pub path: String,
    /// Production lines (see the module text).
    pub lines: usize,
    pub longest_fn: usize,
    /// The name of the longest function; empty when the file has none.
    pub longest_fn_name: String,
    pub weekly_landings: usize,
    pub citations: usize,
    /// Why the file is over a limit, one sentence each; empty when it is not.
    pub breaches: Vec<String>,
}

impl FileHealth {
    pub fn over(&self) -> bool {
        !self.breaches.is_empty()
    }
}

/// What the check found.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CodeHealth {
    /// Measured files, largest first.
    pub files: Vec<FileHealth>,
    /// Files that could not be read or parsed, by path.
    pub unmeasured: Vec<String>,
    /// How long the test suite took in the latest landed verification, in seconds; absent when no record has it.
    pub test_run_secs: Option<f64>,
    pub thresholds: Thresholds,
}

impl CodeHealth {
    /// The files over a limit.
    pub fn breaching(&self) -> Vec<&FileHealth> {
        self.files.iter().filter(|f| f.over()).collect()
    }

    /// The table as plain lines: the limits, one row per file (files over a limit first, then the largest, at
    /// most [`TABLE_ROWS`] unless more are over), and the test suite's run time.
    pub fn table_text(&self) -> String {
        let t = &self.thresholds;
        let mut out = String::new();
        let _ = writeln!(
            out,
            "limits: a file over {} lines · a function over {} lines · a file over {} lines with {}+ landings in {} days",
            t.max_file_lines,
            t.max_fn_lines,
            t.busy_file_lines,
            t.busy_weekly_landings,
            WINDOW_DAYS
        );
        if self.files.is_empty() {
            let _ = writeln!(out, "no files measured; this is not a healthy result");
        } else {
            let _ = writeln!(
                out,
                "{:>6} {:>9} {:>9} {:>9}  {:<5} file",
                "lines", "longest fn", "week", "citations", "over"
            );
            let mut rows: Vec<&FileHealth> = self.files.iter().collect();
            rows.sort_by(|a, b| {
                (!a.over(), std::cmp::Reverse(a.lines), &a.path).cmp(&(
                    !b.over(),
                    std::cmp::Reverse(b.lines),
                    &b.path,
                ))
            });
            let over = rows.iter().filter(|f| f.over()).count();
            let shown = over.max(TABLE_ROWS).min(rows.len());
            for f in &rows[..shown] {
                let _ = writeln!(
                    out,
                    "{:>6} {:>9} {:>9} {:>9}  {:<5} {}",
                    f.lines,
                    f.longest_fn,
                    f.weekly_landings,
                    f.citations,
                    if f.over() { "OVER" } else { "no" },
                    f.path
                );
                for b in &f.breaches {
                    let _ = writeln!(out, "         breach: {b}");
                }
            }
            if shown < rows.len() {
                let _ = writeln!(
                    out,
                    "... and {} more files under the limits",
                    rows.len() - shown
                );
            }
            let _ = writeln!(
                out,
                "{} files measured, {} over a limit",
                self.files.len(),
                over
            );
        }
        if !self.unmeasured.is_empty() {
            let _ = writeln!(out, "not measured: {}", self.unmeasured.join(", "));
        }
        let _ = match self.test_run_secs {
            Some(s) => writeln!(
                out,
                "test suite run time: {s:.0} s (latest landed verification)"
            ),
            None => writeln!(
                out,
                "test suite run time: unknown (no landed verification has one)"
            ),
        };
        out
    }
}

// ---------------------------------------------------------------------------------------------------------
// Measuring one source file

/// What parsing one file gives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStats {
    pub lines: usize,
    pub longest_fn: usize,
    pub longest_fn_name: String,
}

fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg") && a.parse_args::<syn::Ident>().is_ok_and(|i| i == "test")
    })
}

fn item_attrs(item: &syn::Item) -> &[syn::Attribute] {
    use syn::Item::*;
    match item {
        Mod(i) => &i.attrs,
        Fn(i) => &i.attrs,
        Impl(i) => &i.attrs,
        Const(i) => &i.attrs,
        Static(i) => &i.attrs,
        Struct(i) => &i.attrs,
        Enum(i) => &i.attrs,
        Trait(i) => &i.attrs,
        Type(i) => &i.attrs,
        Use(i) => &i.attrs,
        Union(i) => &i.attrs,
        Macro(i) => &i.attrs,
        _ => &[],
    }
}

#[derive(Default)]
struct Scan {
    test_ranges: Vec<(usize, usize)>,
    longest: usize,
    longest_name: String,
}

impl Scan {
    fn function(&mut self, name: &syn::Ident, signature: &syn::Signature, block: &syn::Block) {
        let start = signature.span().start().line;
        let end = block.brace_token.span.close().end().line;
        let len = end.saturating_sub(start) + 1;
        if len > self.longest {
            self.longest = len;
            self.longest_name = name.to_string();
        }
    }
}

impl<'ast> Visit<'ast> for Scan {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        if is_cfg_test(item_attrs(item)) {
            let span = item.span();
            self.test_ranges.push((span.start().line, span.end().line));
            return;
        }
        syn::visit::visit_item(self, item);
    }

    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.function(&f.sig.ident, &f.sig, &f.block);
        syn::visit::visit_item_fn(self, f);
    }

    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        if is_cfg_test(&f.attrs) {
            let span = f.span();
            self.test_ranges.push((span.start().line, span.end().line));
            return;
        }
        self.function(&f.sig.ident, &f.sig, &f.block);
        syn::visit::visit_impl_item_fn(self, f);
    }

    fn visit_trait_item_fn(&mut self, f: &'ast syn::TraitItemFn) {
        if let Some(block) = &f.default {
            self.function(&f.sig.ident, &f.sig, block);
        }
        syn::visit::visit_trait_item_fn(self, f);
    }
}

/// Whether a line counts as code, given whether a block comment is open at its start (updated).
fn is_code_line(line: &str, in_block: &mut bool) -> bool {
    let mut text = line.trim();
    if *in_block {
        match text.find("*/") {
            Some(i) => {
                *in_block = false;
                text = text[i + 2..].trim();
            }
            None => return false,
        }
    }
    if text.is_empty() || text.starts_with("//") {
        return false;
    }
    if text.starts_with("/*") {
        return match text.find("*/") {
            Some(i) => {
                let rest = text[i + 2..].trim();
                !rest.is_empty() && !rest.starts_with("//")
            }
            None => {
                *in_block = true;
                false
            }
        };
    }
    true
}

/// Production lines and the longest function of one source text. An error means it does not parse.
pub fn source_stats(source: &str) -> Result<SourceStats> {
    let file = syn::parse_file(source).context("the source does not parse")?;
    let mut scan = Scan::default();
    scan.visit_file(&file);
    let mut in_block = false;
    let mut lines = 0;
    for (index, line) in source.lines().enumerate() {
        let number = index + 1;
        let code = is_code_line(line, &mut in_block);
        let in_test = scan
            .test_ranges
            .iter()
            .any(|&(s, e)| number >= s && number <= e);
        if code && !in_test {
            lines += 1;
        }
    }
    Ok(SourceStats {
        lines,
        longest_fn: scan.longest,
        longest_fn_name: scan.longest_name,
    })
}

// ---------------------------------------------------------------------------------------------------------
// Joining the records

fn outcome_of(job: &JobRecords) -> Option<&Outcome> {
    job.later_outcome.as_ref().or(job.outcome.as_ref())
}

fn is_request_job(job: &JobRecords) -> bool {
    job.id.as_ref().is_some_and(|id| id.is_request())
}

/// The landed request jobs: when each landed and the paths its candidate changed. A design review is not
/// counted: it changes only a note, and counting it would move the next review's clock.
fn landings(records: &Records) -> Vec<(Timestamp, Vec<&str>)> {
    records
        .jobs
        .values()
        .filter(|j| is_request_job(j) && j.state() == JobState::Closed(OutcomeState::Landed))
        .filter(|j| !j.request.as_ref().is_some_and(is_review))
        .filter_map(|j| {
            let at = outcome_of(j)?.at;
            let paths = j
                .candidate
                .as_ref()
                .map(|c| c.changes.iter().map(|e| e.path.as_str()).collect())
                .unwrap_or_default();
            Some((at, paths))
        })
        .collect()
}

/// Rejected or failed request jobs, with when they ended.
fn failures(records: &Records) -> Vec<(Timestamp, &JobRecords)> {
    records
        .jobs
        .values()
        .filter(|j| is_request_job(j))
        .filter_map(|j| {
            let outcome = outcome_of(j)?;
            matches!(outcome.state, OutcomeState::Rejected | OutcomeState::Failed)
                .then_some((outcome.at, j))
        })
        .collect()
}

/// Whether a failed job's findings, reasons or outcome text name `path`.
fn cites(job: &JobRecords, path: &str) -> bool {
    let outcome = outcome_of(job).is_some_and(|o| o.reason.contains(path));
    let verification = job.verification.as_ref().is_some_and(|v| {
        v.reasons.iter().any(|r| r.contains(path))
            || v.findings.iter().any(|f| {
                f.summary.contains(path)
                    || f.evidence.contains(path)
                    || f.location.as_deref().is_some_and(|l| l.contains(path))
            })
    });
    outcome || verification
}

/// The test suite's run time in the latest landed verification that has a `test` check with a known time.
fn test_run_secs(records: &Records) -> Option<f64> {
    use athena_data::Unit;
    records
        .jobs
        .values()
        .filter(|j| is_request_job(j) && j.state() == JobState::Closed(OutcomeState::Landed))
        .filter_map(|j| {
            let at = outcome_of(j)?.at;
            let check = j
                .verification
                .as_ref()?
                .checks
                .iter()
                .find(|c| c.name == "test")?;
            let value = check.duration.value()?;
            let secs = match check.duration.unit() {
                Unit::Seconds => value,
                Unit::Minutes => value * 60.0,
                _ => return None,
            };
            Some((at, secs))
        })
        .max_by_key(|(at, _)| *at)
        .map(|(_, secs)| secs)
}

/// The sentences for each limit a file is over.
pub fn breaches(stats: &SourceStats, weekly_landings: usize, t: &Thresholds) -> Vec<String> {
    let mut out = Vec::new();
    if stats.lines > t.max_file_lines {
        out.push(format!(
            "{} production lines is over {}",
            stats.lines, t.max_file_lines
        ));
    }
    if stats.longest_fn > t.max_fn_lines {
        out.push(format!(
            "function {} is {} lines, over {}",
            stats.longest_fn_name, stats.longest_fn, t.max_fn_lines
        ));
    }
    if stats.lines > t.busy_file_lines && weekly_landings >= t.busy_weekly_landings {
        out.push(format!(
            "{} production lines is over {} with {} landings in {} days (limit {})",
            stats.lines, t.busy_file_lines, weekly_landings, WINDOW_DAYS, t.busy_weekly_landings
        ));
    }
    out
}

/// Measures source texts (path, text) against the records at `now`. Pure: no file is read here.
pub fn measure_sources(
    sources: &[(String, String)],
    unreadable: Vec<String>,
    records: &Records,
    now: Timestamp,
) -> CodeHealth {
    let cutoff = now.as_second() - WINDOW_DAYS * 86_400;
    let landed = landings(records);
    let failed = failures(records);
    let mut files = Vec::new();
    let mut unmeasured = unreadable;
    for (path, text) in sources {
        let Ok(stats) = source_stats(text) else {
            unmeasured.push(path.clone());
            continue;
        };
        let weekly_landings = landed
            .iter()
            .filter(|(at, paths)| at.as_second() >= cutoff && paths.contains(&path.as_str()))
            .count();
        let citations = failed.iter().filter(|(_, j)| cites(j, path)).count();
        let breaches = breaches(&stats, weekly_landings, &Thresholds::CURRENT);
        files.push(FileHealth {
            path: path.clone(),
            lines: stats.lines,
            longest_fn: stats.longest_fn,
            longest_fn_name: stats.longest_fn_name,
            weekly_landings,
            citations,
            breaches,
        });
    }
    files.sort_by(|a, b| {
        (std::cmp::Reverse(a.lines), &a.path).cmp(&(std::cmp::Reverse(b.lines), &b.path))
    });
    unmeasured.sort();
    CodeHealth {
        files,
        unmeasured,
        test_run_secs: test_run_secs(records),
        thresholds: Thresholds::CURRENT,
    }
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with('.') || name == "target" {
            continue;
        }
        match entry.file_type() {
            Ok(t) if t.is_dir() => walk(&path, out),
            Ok(t) if t.is_file() && name.ends_with(".rs") => out.push(path),
            _ => {}
        }
    }
}

/// Measures the Rust files under `root/crates` against the records at `now`.
pub fn measure(root: &Path, records: &Records, now: Timestamp) -> CodeHealth {
    let mut paths = Vec::new();
    walk(&root.join("crates"), &mut paths);
    paths.sort();
    let mut sources = Vec::new();
    let mut unreadable = Vec::new();
    for path in paths {
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        match std::fs::read_to_string(&path) {
            Ok(text) => sources.push((rel, text)),
            Err(_) => unreadable.push(rel),
        }
    }
    measure_sources(&sources, unreadable, records, now)
}

// ---------------------------------------------------------------------------------------------------------
// What to queue

/// Jobs that have not ended: a request still waiting, being worked, or blocked.
fn open_jobs(records: &Records) -> impl Iterator<Item = &JobRecords> {
    records
        .jobs
        .values()
        .filter(|j| is_request_job(j) && !matches!(j.state(), JobState::Closed(_)))
}

fn request_text(job: &JobRecords) -> &str {
    job.request.as_ref().map_or("", |r| r.text.as_str())
}

fn structural_marker(path: &str) -> String {
    format!("{STRUCTURAL_MARKER}{path}")
}

fn has_open_structural_pass(records: &Records, path: &str) -> bool {
    let marker = structural_marker(path);
    open_jobs(records).any(|j| request_text(j).lines().any(|l| l.trim() == marker))
}

fn is_review(request: &Request) -> bool {
    request
        .text
        .lines()
        .next()
        .is_some_and(|l| l.trim() == REVIEW_TITLE)
}

/// The text of the structural-pass request for one breaching file.
pub fn structural_pass_text(f: &FileHealth, t: &Thresholds) -> String {
    format!(
        "{marker}\n\n\
         The code-health check measured {path}: {lines} production lines, longest function {fnlen} lines ({fname}), \
         {weekly} landings in the last {days} days, {cites} failure or rejection citations. \
         Over a limit: {why}.\n\n\
         Do a structural pass on this file: split or restructure it so it is within the limits, with no change in \
         behavior. Move code into modules that each have one job; do not rewrite logic.\n\n\
         Acceptance criteria:\n\
         - {path} has at most {max_lines} production lines (blank lines, whole-line comments and #[cfg(test)] items \
         are not counted), and no function in it is longer than {max_fn} lines. Run cargo test and see the \
         code-health table in athena status.\n\
         - cargo test passes with the same tests as before; no behavior changes.\n\
         - cargo clippy with warnings denied and cargo fmt --check are clean.\n",
        marker = structural_marker(&f.path),
        path = f.path,
        lines = f.lines,
        fnlen = f.longest_fn,
        fname = if f.longest_fn_name.is_empty() {
            "none"
        } else {
            &f.longest_fn_name
        },
        weekly = f.weekly_landings,
        days = WINDOW_DAYS,
        cites = f.citations,
        why = f.breaches.join("; "),
        max_lines = if f.weekly_landings >= t.busy_weekly_landings && f.lines > t.busy_file_lines {
            t.busy_file_lines
        } else {
            t.max_file_lines
        },
        max_fn = t.max_fn_lines,
    )
}

/// How many jobs landed since the last design review, and whether a first baseline must be written.
struct ReviewClock {
    since_last: usize,
    baseline_to_write: Option<usize>,
    last_review_at: Option<Timestamp>,
}

fn review_clock(records: &Records, baseline: Option<usize>) -> ReviewClock {
    let landed = landings(records);
    let last_review = records
        .jobs
        .values()
        .filter_map(|j| j.request.as_ref())
        .filter(|r| is_review(r))
        .map(|r| r.received_at)
        .max();
    match (last_review, baseline) {
        (Some(at), _) => ReviewClock {
            since_last: landed.iter().filter(|(t, _)| *t > at).count(),
            baseline_to_write: None,
            last_review_at: Some(at),
        },
        (None, Some(base)) => ReviewClock {
            since_last: landed.len().saturating_sub(base),
            baseline_to_write: None,
            last_review_at: None,
        },
        (None, None) => ReviewClock {
            since_last: 0,
            baseline_to_write: Some(landed.len()),
            last_review_at: None,
        },
    }
}

/// The self-contained design-review request. It names what to review, with the current data, and what to
/// produce.
pub fn design_review_text(
    health: &CodeHealth,
    records: &Records,
    since: Option<Timestamp>,
    landed_since: usize,
) -> String {
    let after = |at: Timestamp| since.is_none_or(|s| at > s);
    let mut changed: BTreeMap<&str, usize> = BTreeMap::new();
    for (at, paths) in landings(records) {
        if after(at) {
            for p in paths {
                *changed.entry(p).or_default() += 1;
            }
        }
    }
    let mut broken: BTreeMap<&str, usize> = BTreeMap::new();
    for (at, job) in failures(records) {
        if after(at) {
            for f in &health.files {
                if cites(job, &f.path) {
                    *broken.entry(f.path.as_str()).or_default() += 1;
                }
            }
        }
    }
    let top = |counts: &BTreeMap<&str, usize>, unit: &str| -> String {
        let mut rows: Vec<_> = counts.iter().collect();
        rows.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        if rows.is_empty() {
            return "  - none\n".into();
        }
        rows.iter()
            .take(5)
            .map(|(p, n)| format!("  - {p}: {n} {unit}\n"))
            .collect()
    };
    let mut structural = String::new();
    for j in open_jobs(records) {
        if let Some(line) = request_text(j)
            .lines()
            .find(|l| l.trim().starts_with(STRUCTURAL_MARKER))
        {
            let _ = writeln!(structural, "  - {} ({})", line.trim(), j.id().as_str());
        }
    }
    if structural.is_empty() {
        structural.push_str("  - none\n");
    }
    let mut events = String::new();
    let mut rejected: Vec<_> = failures(records)
        .into_iter()
        .filter(|(at, _)| after(*at))
        .collect();
    rejected.sort_by_key(|(at, _)| std::cmp::Reverse(*at));
    for (_, j) in rejected.iter().take(10) {
        let reason = outcome_of(j).map_or("", |o| o.reason.as_str());
        let _ = writeln!(
            events,
            "  - job {} ended {}: {}",
            j.id().as_str(),
            outcome_of(j).map_or("", |o| o.state.as_str()),
            reason.lines().next().unwrap_or("")
        );
    }
    let mut stops: Vec<&JobRecords> = records
        .jobs
        .values()
        .filter(|j| is_request_job(j))
        .filter(|j| crate::status::plan_state(j).is_some_and(|s| !s.accepted))
        .filter(|j| j.request.as_ref().is_some_and(|r| after(r.received_at)))
        .collect();
    stops.sort_by(|a, b| b.id().as_str().cmp(a.id().as_str()));
    for j in stops.iter().take(10) {
        let round = crate::status::plan_state(j).map_or(0, |s| s.round);
        let _ = writeln!(
            events,
            "  - job {} planning stopped at round {round}",
            j.id().as_str()
        );
    }
    if events.is_empty() {
        events.push_str("  - none\n");
    }
    let table: String = health
        .table_text()
        .lines()
        .map(|l| format!("  {l}\n"))
        .collect();
    format!(
        "{REVIEW_TITLE}\n\
         Priority: high\n\n\
         This is a design-only job: change no source code and no tests. Its one deliverable is a short design \
         note. It was queued because {landed_since} jobs have landed since the last design review (the check \
         queues one every {every} landings).\n\n\
         What to review:\n\
         - The code-health table (`athena status` shows it), as measured when this was queued:\n{table}\
         - The files changed most since the last review (landed jobs that changed them):\n{changed}\
         - The files most broken since the last review (rejections and failures that cite them):\n{broken}\
         - The open structural-pass requests:\n{structural}\
         - Recent rejections and planning stops:\n{events}\n\
         What to produce:\n\
         - A short design note, one file under docs/design/, with proposed changes. For each change give the \
         problem it fixes, the files it touches, and a scope small enough to run as one job. Say which proposed \
         changes make an open structural-pass request unnecessary.\n\n\
         Acceptance criteria:\n\
         - The note covers each of: the code-health table, the files changed most, the files most broken, the \
         open structural-pass requests, and the recent rejections and planning stops.\n\
         - The note lists proposed changes, each with its problem, its files and its scope.\n\
         - The job changes only the design note: no source, test or configuration file.\n",
        every = REVIEW_EVERY_LANDINGS,
        changed = top(&changed, "landings"),
        broken = top(&broken, "citations"),
    )
}

/// What a check decided to queue.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Plan {
    /// One request text per breaching file without an open structural-pass request.
    pub structural: Vec<String>,
    /// The design-review request, when due.
    pub review: Option<String>,
    /// The first baseline to store, when there is no earlier review and none stored.
    pub baseline_to_write: Option<usize>,
}

/// Decides what to queue from the measurement and the records. Pure.
pub fn plan_requests(health: &CodeHealth, records: &Records, baseline: Option<usize>) -> Plan {
    let structural = health
        .breaching()
        .into_iter()
        .filter(|f| !has_open_structural_pass(records, &f.path))
        .map(|f| structural_pass_text(f, &health.thresholds))
        .collect();
    let clock = review_clock(records, baseline);
    let review_open = open_jobs(records).any(|j| j.request.as_ref().is_some_and(is_review));
    let review = (clock.since_last >= REVIEW_EVERY_LANDINGS && !review_open)
        .then(|| design_review_text(health, records, clock.last_review_at, clock.since_last));
    Plan {
        structural,
        review,
        baseline_to_write: clock.baseline_to_write,
    }
}

// ---------------------------------------------------------------------------------------------------------
// Running it

impl Athena {
    fn baseline_path(&self) -> PathBuf {
        self.state.root().join("code-health-baseline")
    }

    /// Runs the check after a landing or a finished job: measures, queues what a breach or the cadence calls
    /// for, and says what it did in one `code health:` line. It never fails its caller: a problem becomes a
    /// line.
    pub fn run_code_health(&self, ui: &dyn Ui) {
        match self.check_code_health() {
            Ok(line) => ui.line(&line),
            Err(e) => ui.line(&format!("code health: could not run: {e:#}")),
        }
    }

    fn check_code_health(&self) -> Result<String> {
        let head = self.head()?;
        let records = Records::load(&self.git, &head)?;
        let health = measure(self.git.dir(), &records, crate::now());
        let baseline = std::fs::read_to_string(self.baseline_path())
            .ok()
            .and_then(|t| t.trim().parse().ok());
        let plan = plan_requests(&health, &records, baseline);
        if let Some(count) = plan.baseline_to_write {
            std::fs::create_dir_all(self.state.root()).context("creating the state directory")?;
            std::fs::write(self.baseline_path(), format!("{count}\n"))
                .context("writing the code-health baseline")?;
        }
        for text in &plan.structural {
            self.queue(text)?;
        }
        if let Some(text) = &plan.review {
            self.queue(text)?;
        }
        if health.files.is_empty() {
            return Ok(format!(
                "code health: no files measured ({} could not be parsed); not treated as healthy",
                health.unmeasured.len()
            ));
        }
        Ok(format!(
            "code health: {} files measured, {} over a limit; queued {} structural pass request(s){}",
            health.files.len(),
            health.breaching().len(),
            plan.structural.len(),
            if plan.review.is_some() {
                " and a design review"
            } else {
                ""
            }
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::Priority;
    use athena_data::records::{
        Candidate, ChangeEntry, ChangeStatus, CheckRun, Finding, FindingSource, Severity, Verdict,
        Verification,
    };
    use athena_data::{JobId, Quantity, RecordId, Unit};

    fn ts(text: &str) -> Timestamp {
        text.parse().unwrap()
    }

    const NOW: &str = "2026-10-26T12:00:00Z";

    fn new_job(received: &str, text: &str) -> JobRecords {
        let id = JobId::Id(RecordId::generate(ts(received)));
        JobRecords {
            id: Some(id.clone()),
            request: Some(Request {
                job: id,
                text: text.into(),
                received_at: ts(received),
                channel: "steward".into(),
            }),
            ..JobRecords::default()
        }
    }

    fn outcome(job: &JobRecords, state: OutcomeState, at: &str, reason: &str) -> Outcome {
        Outcome {
            job: job.id().clone(),
            state,
            reason: reason.into(),
            at: ts(at),
            totals: None,
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        }
    }

    fn candidate(job: &JobRecords, paths: &[&str]) -> Candidate {
        Candidate {
            job: job.id().clone(),
            base_commit: "0".repeat(40),
            base_fingerprint: None,
            previous_job: None,
            candidate_commit: None,
            candidate_tree: None,
            changes: paths
                .iter()
                .map(|p| ChangeEntry {
                    path: p.to_string(),
                    status: ChangeStatus::Modified,
                    mode: None,
                    bytes: None,
                })
                .collect(),
            fingerprint: None,
            violations: Vec::new(),
            producer_report: None,
        }
    }

    fn verification(
        job: &JobRecords,
        verdict: Verdict,
        test_secs: f64,
        cited: &[&str],
    ) -> Verification {
        Verification {
            job: job.id().clone(),
            candidate_commit: "0".repeat(40),
            verified_fingerprint: "sha256:x".into(),
            tool_config_from_base: Vec::new(),
            checks: vec![CheckRun {
                name: "test".into(),
                command: Vec::new(),
                exit_code: Some(0),
                passed: true,
                tests_ran: Some(5),
                duration: Quantity::measured(test_secs, Unit::Seconds, "test"),
                output_tail: String::new(),
                note: None,
            }],
            check_cleanup: Vec::new(),
            verifier_invocation: None,
            criteria: Vec::new(),
            findings: cited
                .iter()
                .map(|p| Finding {
                    severity: Severity::Major,
                    summary: "a defect".into(),
                    evidence: "seen".into(),
                    location: Some(p.to_string()),
                    source: FindingSource::Verifier,
                })
                .collect(),
            request_reading_mismatch: None,
            agent_verdict: None,
            final_verdict: verdict,
            reasons: Vec::new(),
            inputs: Vec::new(),
            summary: None,
        }
    }

    /// A job that landed at `at`, having changed `paths`.
    fn landed(at: &str, paths: &[&str]) -> JobRecords {
        let mut job = new_job(at, "Change something");
        job.candidate = Some(candidate(&job, paths));
        job.verification = Some(verification(&job, Verdict::Pass, 540.0, &[]));
        job.outcome = Some(outcome(&job, OutcomeState::Landed, at, "landed"));
        job
    }

    fn records(jobs: Vec<JobRecords>) -> Records {
        let mut records = Records::default();
        for job in jobs {
            records.jobs.insert(job.id().as_str().to_string(), job);
        }
        records
    }

    /// `lines` production lines, one constant each.
    fn consts(lines: usize) -> String {
        (0..lines)
            .map(|i| format!("pub const C{i}: u32 = {i};\n"))
            .collect()
    }

    /// One function of exactly `lines` lines.
    fn function_of(lines: usize) -> String {
        let mut text = String::from("pub fn big() {\n");
        text.push_str(&"    let _ = 0;\n".repeat(lines - 2));
        text.push_str("}\n");
        text
    }

    const FIXTURE: &str = "\
//! Module doc.
use std::fmt;

/// Doc.
pub fn short() -> u32 {
    1
}

pub fn long_one() -> u32 {
    let s = \"}\"; // a brace in a string
    let n = s.len() as u32;
    // a comment
    n
}

struct S;
impl S {
    fn method(&self) {
        let _ = 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t() {
        assert_eq!(short(), 1);
    }
}
";

    #[test]
    fn measures_lines_longest_function_weekly_landings_and_citations_from_files_and_records() {
        let root = tempfile::tempdir().unwrap();
        let a = "crates/x/src/a.rs";
        let other = "crates/x/src/other.rs";
        std::fs::create_dir_all(root.path().join("crates/x/src")).unwrap();
        std::fs::write(root.path().join(a), FIXTURE).unwrap();
        std::fs::write(root.path().join(other), "pub const A: u32 = 1;\n").unwrap();
        // A target directory is skipped.
        std::fs::create_dir_all(root.path().join("crates/x/target")).unwrap();
        std::fs::write(
            root.path().join("crates/x/target/gen.rs"),
            "pub fn g() {}\n",
        )
        .unwrap();

        let mut rejected = new_job("2026-10-24T00:00:00Z", "Change a");
        rejected.verification = Some(verification(&rejected, Verdict::Fail, 10.0, &[a]));
        rejected.outcome = Some(outcome(
            &rejected,
            OutcomeState::Rejected,
            "2026-10-24T01:00:00Z",
            "no",
        ));
        let mut failed = new_job("2026-10-23T00:00:00Z", "Change a again");
        failed.outcome = Some(outcome(
            &failed,
            OutcomeState::Failed,
            "2026-10-23T01:00:00Z",
            &format!("the build broke in {a}"),
        ));
        let records = records(vec![
            landed("2026-10-24T12:00:00Z", &[a]),
            // Ten days ago: outside the week.
            landed("2026-10-16T12:00:00Z", &[a]),
            // Inside the week, another file.
            landed("2026-10-25T12:00:00Z", &[other]),
            rejected,
            failed,
        ]);

        let health = measure(root.path(), &records, ts(NOW));
        assert_eq!(health.files.len(), 2, "{health:#?}");
        let file = health.files.iter().find(|f| f.path == a).unwrap();
        assert_eq!(
            file.lines, 15,
            "the test module, blanks and comments are not counted"
        );
        assert_eq!(file.longest_fn, 6);
        assert_eq!(file.longest_fn_name, "long_one");
        assert_eq!(
            file.weekly_landings, 1,
            "the landing ten days ago is outside the week"
        );
        assert_eq!(
            file.citations, 2,
            "a finding and an outcome reason name the file"
        );
        assert!(!file.over());
        let other = health.files.iter().find(|f| f.path == other).unwrap();
        assert_eq!((other.weekly_landings, other.citations), (1, 0));
        assert_eq!(health.test_run_secs, Some(540.0));
        assert!(health.unmeasured.is_empty());
    }

    #[test]
    fn a_file_that_does_not_parse_is_unmeasured_and_no_files_is_not_healthy() {
        let health = measure_sources(
            &[("crates/x/src/bad.rs".into(), "fn {".into())],
            Vec::new(),
            &Records::default(),
            ts(NOW),
        );
        assert!(health.files.is_empty());
        assert_eq!(health.unmeasured, vec!["crates/x/src/bad.rs".to_string()]);
        let text = health.table_text();
        assert!(text.contains("no files measured"), "{text}");
        assert!(text.contains("not measured: crates/x/src/bad.rs"), "{text}");
        assert!(text.contains("test suite run time: unknown"), "{text}");
        // A checkout without crates/ also measures nothing.
        let empty = tempfile::tempdir().unwrap();
        assert!(
            measure(empty.path(), &Records::default(), ts(NOW))
                .files
                .is_empty()
        );
    }

    fn weekly(path: &str, n: usize) -> Vec<JobRecords> {
        (0..n)
            .map(|i| landed(&format!("2026-10-2{}T10:00:00Z", 1 + i % 5), &[path]))
            .collect()
    }

    fn breaches_of(source: String, landings: usize) -> Vec<String> {
        let path = "crates/x/src/f.rs";
        let records = records(weekly(path, landings));
        let health = measure_sources(&[(path.into(), source)], Vec::new(), &records, ts(NOW));
        health.files[0].breaches.clone()
    }

    #[test]
    fn thresholds_flag_each_breach_and_leave_a_file_under_them_alone() {
        // More than 2000 production lines.
        let b = breaches_of(consts(2001), 0);
        assert_eq!(b.len(), 1, "{b:?}");
        assert!(b[0].contains("2001 production lines is over 2000"), "{b:?}");
        assert!(
            breaches_of(consts(2000), 0).is_empty(),
            "2000 is at the limit, not over"
        );
        // A function over 200 lines.
        let b = breaches_of(function_of(201), 0);
        assert_eq!(b.len(), 1, "{b:?}");
        assert!(
            b[0].contains("function big is 201 lines, over 200"),
            "{b:?}"
        );
        assert!(breaches_of(function_of(200), 0).is_empty());
        // Over 1500 lines with 5 landings in the week.
        let b = breaches_of(consts(1501), 5);
        assert_eq!(b.len(), 1, "{b:?}");
        assert!(
            b[0].contains("1501 production lines is over 1500 with 5 landings in 7 days"),
            "{b:?}"
        );
        assert!(
            breaches_of(consts(1501), 4).is_empty(),
            "four landings do not breach"
        );
        assert!(
            breaches_of(consts(1500), 5).is_empty(),
            "1500 lines is at the limit"
        );
        // A small, quiet file.
        assert!(breaches_of(consts(50), 0).is_empty());
        assert!(
            breaches_of(consts(50), 9).is_empty(),
            "landings alone are not a breach"
        );
    }

    // -----------------------------------------------------------------------------------------------------
    // Queueing

    fn breaching_health(path: &str) -> CodeHealth {
        measure_sources(
            &[(path.into(), consts(2001))],
            Vec::new(),
            &Records::default(),
            ts(NOW),
        )
    }

    #[test]
    fn a_breach_plans_one_structural_pass_naming_the_numbers_unless_one_is_open() {
        let path = "crates/x/src/f.rs";
        let health = breaching_health(path);
        let plan = plan_requests(&health, &Records::default(), Some(0));
        assert_eq!(plan.structural.len(), 1);
        let text = &plan.structural[0];
        assert!(
            text.lines().next().unwrap() == format!("Structural pass: {path}"),
            "{text}"
        );
        assert!(text.contains("2001 production lines"), "{text}");
        assert!(text.contains("longest function"), "{text}");
        assert!(text.contains("Acceptance criteria:\n- "), "{text}");
        // An open request for the file (queued, so not closed) stops another.
        let open = new_job("2026-10-26T00:00:00Z", text);
        let with_open = records(vec![open.clone()]);
        assert!(
            plan_requests(&health, &with_open, Some(0))
                .structural
                .is_empty()
        );
        // A request for another file does not.
        let other = records(vec![new_job(
            "2026-10-26T00:00:00Z",
            "Structural pass: crates/y/src/g.rs\n",
        )]);
        assert_eq!(plan_requests(&health, &other, Some(0)).structural.len(), 1);
        // A closed one does not block a new request.
        let mut closed = open;
        closed.outcome = Some(outcome(&closed, OutcomeState::Rejected, NOW, "no"));
        let with_closed = records(vec![closed]);
        assert_eq!(
            plan_requests(&health, &with_closed, Some(0))
                .structural
                .len(),
            1
        );
        // A healthy tree plans nothing.
        let fine = measure_sources(
            &[(path.into(), consts(10))],
            Vec::new(),
            &Records::default(),
            ts(NOW),
        );
        assert!(
            plan_requests(&fine, &Records::default(), Some(0))
                .structural
                .is_empty()
        );
    }

    /// An `Athena` over `git` with a scratch state directory. No agent is ever run.
    fn athena_over(git: athena_git::Git, state: &tempfile::TempDir) -> Athena {
        use athena_agent::fake::FakeAgentDriver;
        let stateful = crate::state::StateDir::new(state.path().join("repo"))
            .with_free_space(crate::test_support::plenty_of_space);
        let driver = FakeAgentDriver::new(Box::new(|_| unreachable!("no agent runs")));
        Athena {
            git,
            state: stateful,
            driver: std::sync::Arc::new(driver),
            capabilities: Vec::new(),
            build: athena_data::records::Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            athena_bin: PathBuf::from("/bin/false"),
            nested: None,
            pinner: Box::new(crate::fake::CountingPinner::new(&"a".repeat(40))),
            jev: None,
        }
    }

    #[test]
    fn a_check_that_cannot_run_becomes_a_line_and_never_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        // Not a repository: there is no head to read records at.
        let athena = athena_over(athena_git::Git::at(dir.path()), &state);
        let ui = Lines(Default::default());
        athena.run_code_health(&ui);
        let lines = ui.0.lock().unwrap().clone();
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(
            lines[0].starts_with("code health: could not run:"),
            "{lines:?}"
        );
    }

    /// An `Athena` over a scratch repository with a 2001-line file in the checkout.
    fn scratch() -> (tempfile::TempDir, tempfile::TempDir, Athena) {
        use athena_git::Git;
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let git = Git::at(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value]).unwrap();
        }
        for (path, text) in [
            ("athena.toml", include_str!("../../../athena.toml")),
            ("defaults.toml", include_str!("../../../defaults.toml")),
        ] {
            std::fs::write(dir.path().join(path), text).unwrap();
        }
        git.run(["add", "-A"]).unwrap();
        git.run([
            "commit",
            "-q",
            "-m",
            "Start a demo repository\n\nAthena-Job: bootstrap",
        ])
        .unwrap();
        std::fs::create_dir_all(dir.path().join("crates/x/src")).unwrap();
        std::fs::write(dir.path().join("crates/x/src/f.rs"), consts(2001)).unwrap();
        let athena = athena_over(git, &state);
        (dir, state, athena)
    }

    struct Lines(std::sync::Mutex<Vec<String>>);
    impl Ui for Lines {
        fn line(&self, text: &str) {
            self.0.lock().unwrap().push(text.to_string());
        }
    }

    fn queued_marked(athena: &Athena, marker: &str) -> Vec<String> {
        let head = athena.head().unwrap();
        Records::load(&athena.git, &head)
            .unwrap()
            .jobs
            .values()
            .filter(|j| j.state() == JobState::Queued)
            .filter_map(|j| j.request.as_ref())
            .filter(|r| r.text.lines().any(|l| l.trim() == marker))
            .map(|r| r.text.clone())
            .collect()
    }

    #[test]
    fn running_the_check_queues_exactly_one_structural_pass_and_a_second_run_queues_none() {
        let (_dir, _state, athena) = scratch();
        let marker = "Structural pass: crates/x/src/f.rs";
        let ui = Lines(Default::default());
        athena.run_code_health(&ui);
        let queued = queued_marked(&athena, marker);
        assert_eq!(queued.len(), 1, "{queued:?} {:?}", ui.0.lock().unwrap());
        assert!(queued[0].contains("2001 production lines"), "{}", queued[0]);
        athena.run_code_health(&ui);
        assert_eq!(
            queued_marked(&athena, marker).len(),
            1,
            "the second run queued nothing"
        );
        let lines = ui.0.lock().unwrap();
        assert!(
            lines[0].contains("1 files measured, 1 over a limit; queued 1 structural pass"),
            "{lines:?}"
        );
        assert!(lines[1].contains("queued 0 structural pass"), "{lines:?}");
    }

    #[test]
    fn status_carries_the_code_health_table() {
        let (_dir, _state, athena) = scratch();
        let view = athena.status().unwrap();
        assert_eq!(view.code_health.files.len(), 1);
        assert_eq!(view.code_health.files[0].path, "crates/x/src/f.rs");
        assert!(view.code_health.files[0].over());
    }

    // -----------------------------------------------------------------------------------------------------
    // The design review

    /// A closed design review received at `at`, then `n` jobs landed after it.
    fn after_review(n: usize) -> Records {
        let mut review = new_job(
            "2026-10-01T00:00:00Z",
            &format!("{REVIEW_TITLE}\nPriority: high\n"),
        );
        review.outcome = Some(outcome(
            &review,
            OutcomeState::Landed,
            "2026-10-02T00:00:00Z",
            "landed",
        ));
        let mut jobs = vec![review];
        for i in 0..n {
            jobs.push(landed(
                &format!("2026-10-{:02}T10:00:00Z", 3 + i),
                &["crates/x/src/f.rs"],
            ));
        }
        records(jobs)
    }

    #[test]
    fn design_review_cadence_queues_at_high_priority_on_the_tenth_landing_not_the_ninth() {
        let health = breaching_health("crates/x/src/f.rs");
        let nine = plan_requests(&health, &after_review(9), None);
        assert_eq!(nine.review, None, "nine landings since the review");
        let ten = plan_requests(&health, &after_review(10), None);
        let text = ten.review.expect("the tenth landing queues a review");
        assert_eq!(Priority::from_text(&text), Priority::High);
        assert!(text.starts_with(REVIEW_TITLE), "{text}");
        // The reviewing job itself, landed, is not a landing: the clock restarts at the review request.
        let eleven = plan_requests(&health, &after_review(11), None);
        assert!(eleven.review.is_some());
        // An open review request holds the next one back.
        let mut open = after_review(10);
        open.jobs.insert(
            "open".into(),
            JobRecords {
                id: Some(JobId::parse("20261020-000000-aaaa").unwrap()),
                request: Some(Request {
                    job: JobId::parse("20261020-000000-aaaa").unwrap(),
                    text: REVIEW_TITLE.into(),
                    received_at: ts("2026-10-20T00:00:00Z"),
                    channel: "steward".into(),
                }),
                ..JobRecords::default()
            },
        );
        assert_eq!(plan_requests(&health, &open, None).review, None);
    }

    #[test]
    fn design_review_text_names_what_to_review_and_what_to_produce() {
        let path = "crates/x/src/f.rs";
        let health = breaching_health(path);
        let mut records = after_review(10);
        let mut rejected = new_job("2026-10-14T00:00:00Z", "Change f");
        rejected.verification = Some(verification(&rejected, Verdict::Fail, 1.0, &[path]));
        rejected.outcome = Some(outcome(
            &rejected,
            OutcomeState::Rejected,
            "2026-10-14T01:00:00Z",
            "the change broke f",
        ));
        let rejected_id = rejected.id().as_str().to_string();
        records.jobs.insert(rejected_id.clone(), rejected);
        let open = new_job(
            "2026-10-15T00:00:00Z",
            &structural_pass_text(&health.files[0], &health.thresholds),
        );
        let open_id = open.id().as_str().to_string();
        records.jobs.insert(open_id.clone(), open);
        let text = plan_requests(&health, &records, None).review.unwrap();
        for wanted in [
            "design-only job",
            "What to review:",
            "The code-health table",
            "2001",
            "The files changed most since the last review",
            &format!("{path}: 10 landings"),
            "The files most broken since the last review",
            &format!("{path}: 1 citations"),
            "The open structural-pass requests:",
            &format!("Structural pass: {path} ({open_id})"),
            "Recent rejections and planning stops:",
            &format!("job {rejected_id} ended rejected: the change broke f"),
            "What to produce:",
            "A short design note",
            "proposed changes",
            "Acceptance criteria:\n- ",
        ] {
            assert!(text.contains(wanted), "{wanted:?} missing from:\n{text}");
        }
    }

    #[test]
    fn the_first_review_does_not_fire_on_existing_history() {
        let health = breaching_health("crates/x/src/f.rs");
        let many = |n| {
            records(
                (0..n)
                    .map(|i| {
                        let at = format!("2026-{:02}-{:02}T10:00:00Z", 1 + i / 28, 1 + i % 28);
                        landed(&at, &["a"])
                    })
                    .collect(),
            )
        };
        // 25 landings and no review request: nothing is queued; the baseline is the current count.
        let first = plan_requests(&health, &many(25), None);
        assert_eq!(first.review, None);
        assert_eq!(first.baseline_to_write, Some(25));
        // With the baseline stored: 9 more landings do not fire, the 10th does.
        assert_eq!(plan_requests(&health, &many(34), Some(25)).review, None);
        assert_eq!(
            plan_requests(&health, &many(34), Some(25)).baseline_to_write,
            None
        );
        assert!(plan_requests(&health, &many(35), Some(25)).review.is_some());
        // End to end: the first run writes the baseline file and queues no review.
        let (_dir, _state, athena) = scratch();
        athena.run_code_health(&Lines(Default::default()));
        let stored = std::fs::read_to_string(athena.baseline_path()).unwrap();
        assert_eq!(stored.trim(), "0");
    }
}
