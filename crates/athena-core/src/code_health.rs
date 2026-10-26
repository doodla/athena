//! Code health: how big the files and functions of Athena's own sources are, how often they change, and how
//! often work on them failed. The check runs after every landing (`Athena::land_pr` and `note_finished`).
//!
//! - **Measuring.** Each tracked Rust source file under a `src/` directory is parsed with `syn`. Its production
//!   lines are its lines minus the `#[cfg(test)]` modules, and its longest function is the longest `fn` outside
//!   test code. A file that does not parse is left out of the table.
//! - **Landings.** A landing is a first-parent commit that carries both an `Athena-Job` and an
//!   `Athena-Verified-Content` trailer. Records-only commits carry the first and never the second, so they do
//!   not count. Both the 7-day churn of a file and the design-review cadence count landings this way.
//! - **Thresholds.** A file is over when it has more than [`MAX_FILE_LINES`] production lines, or a function of
//!   more than [`MAX_FN_LINES`] lines, or more than [`BUSY_FILE_LINES`] lines with [`BUSY_WEEKLY_LANDINGS`] or
//!   more landings in the last week.
//! - **Structural passes.** For each file over a threshold, one request is queued (`Structural pass: <path>`),
//!   unless one for the file is still open.
//! - **Design review.** The first check writes a [`CodeHealthBaseline`] record and queues nothing. Once
//!   [`REVIEW_EVERY`] landings stand beyond the latest baseline, a self-contained high-priority design review is
//!   queued and a new baseline is written. The baseline is a record in git, so it survives a fresh state
//!   directory.
//!
//! The check only queues requests and writes the baseline; a failure is shown as a warning and never changes the
//! result of the landing.

use crate::commits::{NewFile, message};
use crate::store::{JobRecords, JobState, Records};
use crate::{Athena, Ui};
use anyhow::{Context, Result};
use athena_data::records::{AnyRecord, CodeHealthBaseline, OutcomeState};
use athena_data::{JobId, RecordId};
use athena_git::Git;
use jiff::Timestamp;
use serde::Serialize;
use syn::visit::Visit;

/// A file with more production lines than this is over.
pub const MAX_FILE_LINES: u32 = 2000;
/// A function longer than this is over.
pub const MAX_FN_LINES: u32 = 200;
/// A file with more production lines than this is over when it also changes this often.
pub const BUSY_FILE_LINES: u32 = 1500;
/// Landings in the last [`WINDOW_DAYS`] days that make a large file busy.
pub const BUSY_WEEKLY_LANDINGS: u32 = 5;
/// The window of the churn count, in days.
pub const WINDOW_DAYS: i64 = 7;
/// Landings after a baseline that make a design review due.
pub const REVIEW_EVERY: u32 = 10;
/// How many files the status table shows besides those that are over.
pub const SHOWN_FILES: usize = 12;
/// The first line of a structural-pass request, before the path.
pub const STRUCTURAL_MARKER: &str = "Structural pass: ";

// ---------------------------------------------------------------------------------------------------------
// Measuring

/// What one source file measures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Measure {
    /// Lines of the file outside `#[cfg(test)]` modules.
    pub lines: u32,
    /// Lines of the longest function outside test code, from its `fn` to its closing brace.
    pub longest_fn: u32,
    pub longest_fn_name: String,
}

fn line_of(span: proc_macro2::Span) -> u32 {
    span.start().line as u32
}

fn end_line_of(span: proc_macro2::Span) -> u32 {
    span.end().line as u32
}

fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path().is_ident("cfg")
            && matches!(a.parse_args::<syn::Meta>(), Ok(syn::Meta::Path(p)) if p.is_ident("test"))
    })
}

fn is_test_fn(attrs: &[syn::Attribute]) -> bool {
    is_cfg_test(attrs) || attrs.iter().any(|a| a.path().is_ident("test"))
}

#[derive(Default)]
struct Scan {
    /// First and last line of each `#[cfg(test)]` module with a body.
    test_ranges: Vec<(u32, u32)>,
    longest: u32,
    longest_name: String,
}

impl Scan {
    fn note_fn(&mut self, attrs: &[syn::Attribute], sig: &syn::Signature, block: &syn::Block) {
        if is_test_fn(attrs) {
            return;
        }
        let start = line_of(sig.fn_token.span);
        let end = end_line_of(block.brace_token.span.close());
        let length = end.saturating_sub(start) + 1;
        if length > self.longest {
            self.longest = length;
            self.longest_name = sig.ident.to_string();
        }
    }
}

impl<'ast> Visit<'ast> for Scan {
    fn visit_item_mod(&mut self, m: &'ast syn::ItemMod) {
        if is_cfg_test(&m.attrs) {
            if let Some((brace, _)) = &m.content {
                let start = m.attrs.first().map_or_else(
                    || line_of(m.mod_token.span),
                    |a| line_of(a.pound_token.span),
                );
                self.test_ranges
                    .push((start, end_line_of(brace.span.close())));
            }
            return;
        }
        syn::visit::visit_item_mod(self, m);
    }

    fn visit_item_fn(&mut self, f: &'ast syn::ItemFn) {
        self.note_fn(&f.attrs, &f.sig, &f.block);
        syn::visit::visit_item_fn(self, f);
    }

    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        self.note_fn(&f.attrs, &f.sig, &f.block);
        syn::visit::visit_impl_item_fn(self, f);
    }

    fn visit_trait_item_fn(&mut self, f: &'ast syn::TraitItemFn) {
        if let Some(block) = &f.default {
            self.note_fn(&f.attrs, &f.sig, block);
        }
        syn::visit::visit_trait_item_fn(self, f);
    }
}

/// Measures the text of a Rust source file.
pub fn measure_source(text: &str) -> Result<Measure> {
    let file = syn::parse_file(text).context("the file does not parse as Rust")?;
    let mut scan = Scan::default();
    scan.visit_file(&file);
    let total = text.lines().count() as u32;
    let in_tests: u32 = scan.test_ranges.iter().map(|(s, e)| e - s + 1).sum();
    Ok(Measure {
        lines: total.saturating_sub(in_tests),
        longest_fn: scan.longest,
        longest_fn_name: scan.longest_name,
    })
}

/// Whether a tracked path is production source the check measures: Rust under a `src/` directory, and not a
/// file that only holds tests.
pub fn is_measured(path: &str) -> bool {
    let Some(stem) = path.strip_suffix(".rs") else {
        return false;
    };
    let name = stem.rsplit('/').next().unwrap_or(stem);
    (path.starts_with("src/") || path.contains("/src/"))
        && !path.contains("/tests/")
        && name != "tests"
        && !name.ends_with("_tests")
}

// ---------------------------------------------------------------------------------------------------------
// Thresholds

/// Why a file is over.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Breach {
    /// More than [`MAX_FILE_LINES`] production lines.
    FileLength { lines: u32 },
    /// A function of more than [`MAX_FN_LINES`] lines.
    FunctionLength { name: String, lines: u32 },
    /// More than [`BUSY_FILE_LINES`] lines, and [`BUSY_WEEKLY_LANDINGS`] or more landings in a week.
    BusyFile { lines: u32, landings: u32 },
}

impl Breach {
    /// The breach in words, with the measured numbers.
    pub fn text(&self) -> String {
        match self {
            Breach::FileLength { lines } => {
                format!("{lines} production lines, over the limit of {MAX_FILE_LINES}")
            }
            Breach::FunctionLength { name, lines } => {
                format!("function {name} is {lines} lines, over the limit of {MAX_FN_LINES}")
            }
            Breach::BusyFile { lines, landings } => format!(
                "{lines} production lines with {landings} landings in the last {WINDOW_DAYS} days, \
                 over the limit of {BUSY_FILE_LINES} lines with {BUSY_WEEKLY_LANDINGS} landings in a week"
            ),
        }
    }

    fn short(&self) -> &'static str {
        match self {
            Breach::FileLength { .. } => "file length",
            Breach::FunctionLength { .. } => "function length",
            Breach::BusyFile { .. } => "busy and large",
        }
    }
}

/// The breaches of a file with these measurements.
pub fn breaches(measure: &Measure, weekly_landings: u32) -> Vec<Breach> {
    let mut out = Vec::new();
    if measure.lines > MAX_FILE_LINES {
        out.push(Breach::FileLength {
            lines: measure.lines,
        });
    }
    if measure.longest_fn > MAX_FN_LINES {
        out.push(Breach::FunctionLength {
            name: measure.longest_fn_name.clone(),
            lines: measure.longest_fn,
        });
    }
    if measure.lines > BUSY_FILE_LINES && weekly_landings >= BUSY_WEEKLY_LANDINGS {
        out.push(Breach::BusyFile {
            lines: measure.lines,
            landings: weekly_landings,
        });
    }
    out
}

// ---------------------------------------------------------------------------------------------------------
// The table

/// One file's row of the code-health table.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FileHealth {
    pub path: String,
    pub lines: u32,
    pub longest_fn: u32,
    pub longest_fn_name: String,
    /// Landings in the last [`WINDOW_DAYS`] days that changed the file.
    pub weekly_landings: u32,
    /// Rejected or failed jobs and stopped plans whose request or deliverables name the file.
    pub citations: u32,
    /// The production-line limit that applies to the file now: [`BUSY_FILE_LINES`] when it is busy.
    pub limit_lines: u32,
    pub limit_fn: u32,
    pub breaches: Vec<Breach>,
}

impl FileHealth {
    pub fn is_over(&self) -> bool {
        !self.breaches.is_empty()
    }
}

/// The code-health table: every measured file, and the time the test suite took.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CodeHealth {
    /// Largest file first.
    pub files: Vec<FileHealth>,
    /// How long the latest recorded run of the test check took, when one is recorded.
    pub suite_seconds: Option<f64>,
}

impl CodeHealth {
    /// The files that are over, then the largest others, up to [`SHOWN_FILES`] rows in all besides the over ones.
    pub fn shown(&self) -> CodeHealth {
        let files = self
            .files
            .iter()
            .enumerate()
            .filter(|(i, f)| f.is_over() || *i < SHOWN_FILES)
            .map(|(_, f)| f.clone())
            .collect();
        CodeHealth {
            files,
            suite_seconds: self.suite_seconds,
        }
    }

    pub fn over(&self) -> Vec<&FileHealth> {
        self.files.iter().filter(|f| f.is_over()).collect()
    }
}

/// The thresholds in one line.
pub fn thresholds_line() -> String {
    format!(
        "limits: {MAX_FILE_LINES} production lines per file; {MAX_FN_LINES} lines per function; \
         {BUSY_FILE_LINES} lines with {BUSY_WEEKLY_LANDINGS} or more landings in {WINDOW_DAYS} days"
    )
}

/// The suite time in words.
pub fn suite_line(seconds: Option<f64>) -> String {
    match seconds {
        Some(s) => format!("test suite run time: {s:.1}s (latest recorded run)"),
        None => "test suite run time: unknown (no run recorded)".to_string(),
    }
}

/// The table as aligned text, one row per line, header first. Columns are padded to the widest cell.
pub fn table_text(health: &CodeHealth) -> String {
    let header = [
        "FILE",
        "LINES",
        "LONGEST FN",
        "LANDINGS 7D",
        "CITED",
        "LIMITS",
        "OVER",
    ];
    let mut rows: Vec<[String; 7]> = vec![header.map(str::to_string)];
    for f in &health.files {
        let over = if f.is_over() {
            let why: Vec<&str> = f.breaches.iter().map(Breach::short).collect();
            format!("yes: {}", why.join(", "))
        } else {
            "no".to_string()
        };
        rows.push([
            f.path.clone(),
            f.lines.to_string(),
            f.longest_fn.to_string(),
            f.weekly_landings.to_string(),
            f.citations.to_string(),
            format!("{} / {}", f.limit_lines, f.limit_fn),
            over,
        ]);
    }
    let widths: Vec<usize> = (0..7)
        .map(|c| rows.iter().map(|r| r[c].chars().count()).max().unwrap_or(0))
        .collect();
    let mut out = String::new();
    for row in &rows {
        let cells: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(c, cell)| format!("{cell:<width$}", width = widths[c]))
            .collect();
        out.push_str(cells.join("  ").trim_end());
        out.push('\n');
    }
    out
}

// ---------------------------------------------------------------------------------------------------------
// Landings and citations

/// One landing on the current branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Landing {
    /// Commit time, in seconds since the epoch.
    pub at: i64,
    pub files: Vec<String>,
}

/// The landings on the first-parent history of `head`, newest first.
pub fn landings(git: &Git, head: &str) -> Result<Vec<Landing>> {
    let out = git
        .run([
            "log",
            "--first-parent",
            "-E",
            "--all-match",
            "--grep=^Athena-Job: ",
            "--grep=^Athena-Verified-Content: ",
            "--name-only",
            "--format=%x1e%ct",
            head,
        ])
        .context("listing the landings")?;
    Ok(out
        .split('\u{1e}')
        .filter(|chunk| !chunk.trim().is_empty())
        .filter_map(|chunk| {
            let mut lines = chunk.lines();
            let at = lines.next()?.trim().parse().ok()?;
            let files = lines
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(str::to_string)
                .collect();
            Some(Landing { at, files })
        })
        .collect())
}

fn is_stopped_plan(job: &JobRecords) -> bool {
    crate::status::plan_state(job).is_some_and(|p| !p.accepted)
}

fn is_failure(job: &JobRecords) -> bool {
    matches!(
        job.state(),
        JobState::Closed(OutcomeState::Rejected | OutcomeState::Failed)
    ) || is_stopped_plan(job)
}

/// The text a job's citations are found in: its request and the paths of its readings' deliverables.
fn cited_text(job: &JobRecords) -> String {
    let mut text = job
        .request
        .as_ref()
        .map_or(String::new(), |r| r.text.clone());
    for reading in &job.readings {
        for d in &reading.deliverables {
            text.push('\n');
            text.push_str(&d.path);
        }
    }
    text
}

/// How many failed, rejected or plan-stopped jobs name `path`.
pub fn citations(records: &Records, path: &str) -> u32 {
    records
        .request_jobs()
        .iter()
        .filter(|job| is_failure(job) && cited_text(job).contains(path))
        .count() as u32
}

/// How long the latest recorded run of the test check took.
pub fn suite_seconds(records: &Records) -> Option<f64> {
    records.request_jobs().iter().find_map(|job| {
        let verification = job.verification.as_ref()?;
        verification
            .checks
            .iter()
            .find(|c| c.name == "test")
            .and_then(|c| c.duration.value())
    })
}

/// Measures the repository at `head` as of `now`.
pub fn compute(git: &Git, head: &str, records: &Records, now: Timestamp) -> Result<CodeHealth> {
    let landed = landings(git, head)?;
    let since = now.as_second() - WINDOW_DAYS * 24 * 3600;
    let mut files = Vec::new();
    for path in git.ls_files()?.into_iter().filter(|p| is_measured(p)) {
        let Ok(text) = std::fs::read_to_string(git.dir().join(&path)) else {
            continue;
        };
        let Ok(measure) = measure_source(&text) else {
            continue;
        };
        let weekly_landings = landed
            .iter()
            .filter(|l| l.at >= since && l.files.contains(&path))
            .count() as u32;
        let breaches = breaches(&measure, weekly_landings);
        files.push(FileHealth {
            citations: citations(records, &path),
            limit_lines: if weekly_landings >= BUSY_WEEKLY_LANDINGS {
                BUSY_FILE_LINES
            } else {
                MAX_FILE_LINES
            },
            limit_fn: MAX_FN_LINES,
            lines: measure.lines,
            longest_fn: measure.longest_fn,
            longest_fn_name: measure.longest_fn_name,
            weekly_landings,
            breaches,
            path,
        });
    }
    files.sort_by(|a, b| b.lines.cmp(&a.lines).then_with(|| a.path.cmp(&b.path)));
    Ok(CodeHealth {
        files,
        suite_seconds: suite_seconds(records),
    })
}

// ---------------------------------------------------------------------------------------------------------
// The requests

/// The request that asks for a structural pass on one file, with the measured numbers.
pub fn structural_text(f: &FileHealth) -> String {
    let reasons: Vec<String> = f.breaches.iter().map(Breach::text).collect();
    format!(
        "{STRUCTURAL_MARKER}{path}\n\n\
         Code health found {path} over a limit: {reasons}.\n\n\
         Measured: {lines} production lines (test modules excluded); longest function {name} at {longest} \
         lines; {weekly} landings in the last {WINDOW_DAYS} days; {cited} failure or rejection citations.\n\
         {limits}.\n\n\
         Restructure the file so it is under its limits: split it by responsibility or break up the long \
         function, without changing behavior. Existing tests keep passing.",
        path = f.path,
        reasons = reasons.join("; "),
        lines = f.lines,
        name = if f.longest_fn_name.is_empty() {
            "(none)"
        } else {
            &f.longest_fn_name
        },
        longest = f.longest_fn,
        weekly = f.weekly_landings,
        cited = f.citations,
        limits = thresholds_line(),
    )
}

fn first_line(job: &JobRecords) -> &str {
    job.request
        .as_ref()
        .and_then(|r| r.text.lines().next())
        .unwrap_or("")
        .trim()
}

/// Whether a structural-pass request for `path` is still open (not closed with an outcome).
pub fn structural_pass_open(records: &Records, path: &str) -> bool {
    let marker = format!("{STRUCTURAL_MARKER}{path}");
    records
        .request_jobs()
        .iter()
        .any(|job| first_line(job) == marker && !matches!(job.state(), JobState::Closed(_)))
}

/// Every open structural-pass request: the job and its first line.
pub fn open_structural_passes(records: &Records) -> Vec<(String, String)> {
    records
        .request_jobs()
        .iter()
        .filter(|job| {
            first_line(job).starts_with(STRUCTURAL_MARKER)
                && !matches!(job.state(), JobState::Closed(_))
        })
        .map(|job| (job.id().as_str().to_string(), first_line(job).to_string()))
        .collect()
}

/// The self-contained design-review request. `since_review` holds the landings after the previous baseline.
pub fn design_review_text(
    health: &CodeHealth,
    records: &Records,
    since_review: &[Landing],
    total_landings: u32,
) -> String {
    let mut changed: std::collections::BTreeMap<&str, u32> = std::collections::BTreeMap::new();
    for landing in since_review {
        for file in landing.files.iter().filter(|f| !f.starts_with("records/")) {
            *changed.entry(file.as_str()).or_default() += 1;
        }
    }
    let mut changed: Vec<(&str, u32)> = changed.into_iter().collect();
    changed.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    let mut broken: Vec<&FileHealth> = health.files.iter().filter(|f| f.citations > 0).collect();
    broken.sort_by(|a, b| {
        b.citations
            .cmp(&a.citations)
            .then_with(|| a.path.cmp(&b.path))
    });

    let mut out = String::new();
    out.push_str("Priority: high\n\n");
    out.push_str(&format!(
        "Design review after {} landings ({total_landings} in all)\n\n",
        since_review.len()
    ));
    out.push_str(
        "This request is self-contained. Athena queues it every ten landings so the shape of its own code \
         is looked at on a schedule, not only when a limit is crossed.\n\n",
    );
    out.push_str("What to review:\n\n");
    out.push_str("1. The code-health table as it stands now:\n\n");
    out.push_str(&table_text(&health.shown()));
    out.push_str(&format!(
        "\n{}\n{}\n\n",
        thresholds_line(),
        suite_line(health.suite_seconds)
    ));
    out.push_str("2. The files most changed since the last review (landings that touched each):\n");
    if changed.is_empty() {
        out.push_str("   none\n");
    }
    for (file, n) in changed.iter().take(8) {
        out.push_str(&format!("   - {file}: {n}\n"));
    }
    out.push_str("\n3. The files most broken since the last review (failure, rejection and plan-stop citations):\n");
    if broken.is_empty() {
        out.push_str("   none\n");
    }
    for f in broken.iter().take(8) {
        out.push_str(&format!("   - {}: {}\n", f.path, f.citations));
    }
    out.push_str("\n4. The open structural-pass requests:\n");
    let open = open_structural_passes(records);
    if open.is_empty() {
        out.push_str("   none\n");
    }
    for (job, line) in &open {
        out.push_str(&format!("   - {job}: {line}\n"));
    }
    out.push_str("\n5. The recent rejections and planning stops:\n");
    let mut any = false;
    for job in records
        .request_jobs()
        .iter()
        .filter(|j| is_failure(j))
        .take(8)
    {
        any = true;
        let what = match job.state() {
            JobState::Closed(state) => state.as_str().to_string(),
            _ => crate::status::plan_state(job).map_or("stopped".to_string(), |p| p.text()),
        };
        out.push_str(&format!("   - {} ({what}): {}\n", job.id(), job.title()));
    }
    if !any {
        out.push_str("   none\n");
    }
    out.push_str(
        "\nWhat to produce: a short design note with proposed changes, naming each file or module to split, \
         merge or reshape and why, ordered by value. This is a design-only job: change no code and no tests, \
         and add only the note (under docs/design/). Each proposed change is something a later request can \
         carry out.\n",
    );
    out
}

// ---------------------------------------------------------------------------------------------------------
// The check

/// What one run of the check did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckReport {
    pub files_measured: usize,
    /// The files a structural-pass request was queued for, with the job.
    pub structural: Vec<(String, JobId)>,
    /// The design review queued by this run.
    pub review: Option<JobId>,
    /// Whether this run wrote the first baseline.
    pub first_baseline: bool,
    pub landings: u32,
}

impl Athena {
    /// Runs the check after a landing and tells the user what it did. A failure is shown as a warning.
    pub fn code_health_after_landing(&self, ui: &dyn Ui) {
        match self.code_health_check(crate::now()) {
            Ok(report) => {
                let mut said = format!(
                    "code health  {} file(s) measured, {} structural pass request(s) queued",
                    report.files_measured,
                    report.structural.len()
                );
                for (path, job) in &report.structural {
                    said.push_str(&format!("; {path} as {job}"));
                }
                ui.line(&said);
                if let Some(job) = &report.review {
                    ui.line(&format!(
                        "code health  design review queued as {job} ({REVIEW_EVERY} landings since the last)"
                    ));
                }
                if report.first_baseline {
                    ui.line(&format!(
                        "code health  baseline set at {} landings; the first design review is due {REVIEW_EVERY} landings later",
                        report.landings
                    ));
                }
            }
            Err(problem) => ui.line(&format!(
                "warning: could not check code health: {problem:#}"
            )),
        }
    }

    /// The check, as of `now`: queues one structural-pass request for each file over a threshold that has none
    /// open, and keeps the design-review cadence.
    pub fn code_health_check(&self, now: Timestamp) -> Result<CheckReport> {
        let head = self.head()?;
        let records = Records::load(&self.git, &head)?;
        let health = compute(&self.git, &head, &records, now)?;
        let landed = landings(&self.git, &head)?;
        let total = landed.len() as u32;
        let mut report = CheckReport {
            files_measured: health.files.len(),
            landings: total,
            ..CheckReport::default()
        };
        for file in health.over() {
            if structural_pass_open(&records, &file.path) {
                continue;
            }
            let (job, _) = self.queue(&structural_text(file))?;
            report.structural.push((file.path.clone(), job));
        }
        match CodeHealthBaseline::latest(&records.code_health_baselines) {
            None => {
                self.write_code_health_baseline(&records, total, false, now)?;
                report.first_baseline = true;
            }
            Some(baseline) if total >= baseline.landings + REVIEW_EVERY => {
                // The queued structural passes are open work the review should list.
                let records = Records::load(&self.git, &self.head()?)?;
                let since = &landed[..(total - baseline.landings) as usize];
                let (job, _) = self.queue(&design_review_text(&health, &records, since, total))?;
                report.review = Some(job);
                self.write_code_health_baseline(&records, total, true, now)?;
            }
            Some(_) => {}
        }
        Ok(report)
    }

    fn write_code_health_baseline(
        &self,
        known: &Records,
        landings: u32,
        review_queued: bool,
        now: Timestamp,
    ) -> Result<()> {
        let mut id = RecordId::generate(now);
        while known.has_record_id(&id) {
            id = RecordId::generate(now);
        }
        let record = AnyRecord::CodeHealthBaseline(CodeHealthBaseline {
            id,
            at: now,
            landings,
            review_queued,
        });
        self.commit_records(
            &[NewFile::record(&record)],
            &message(
                &format!("Code health baseline at {landings} landings"),
                "Where the design-review cadence counts from: a review is queued when ten landings stand beyond the latest baseline.",
                &[("Athena-Record", record.canonical_path())],
            ),
        )?;
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use athena_agent::fake::FakeAgentDriver;
    use athena_data::records::Build;

    /// A function of `body` statement lines, `body + 2` lines in all.
    fn function(name: &str, body: usize) -> String {
        let mut text = format!("fn {name}() {{\n");
        for i in 0..body {
            text.push_str(&format!("    let _v{i} = {i};\n"));
        }
        text.push_str("}\n");
        text
    }

    fn filler(lines: usize) -> String {
        (0..lines)
            .map(|i| format!("const C{i}: u32 = {i};\n"))
            .collect()
    }

    #[test]
    fn measuring_excludes_cfg_test_modules_and_finds_the_longest_function() {
        let source = format!(
            "//! A fixture.\n{}{}\n#[cfg(test)]\nmod tests {{\n    {}\n    fn huge() {{\n{}    }}\n}}\n",
            function("short", 3),
            function("long", 10),
            "use super::*;",
            "        let _x = 1;\n".repeat(50),
        );
        let total = source.lines().count() as u32;
        let measure = measure_source(&source).unwrap();
        // 1 doc line, 5 + 12 function lines, one blank line.
        assert_eq!(measure.lines, 1 + 5 + 12 + 1, "of {total} lines");
        assert_eq!(measure.longest_fn, 12);
        assert_eq!(measure.longest_fn_name, "long");
    }

    #[test]
    fn measuring_finds_methods_and_ignores_braces_in_strings() {
        let source =
            "struct S;\nimpl S {\n    fn m(&self) -> &'static str {\n        \"}}}}\"\n    }\n}\n";
        let measure = measure_source(source).unwrap();
        assert_eq!(measure.longest_fn, 3);
        assert_eq!(measure.longest_fn_name, "m");
        assert_eq!(measure.lines, 6);
        assert!(measure_source("fn broken( {").is_err());
    }

    #[test]
    fn only_production_sources_are_measured() {
        assert!(is_measured("crates/athena-core/src/run.rs"));
        assert!(is_measured("src/lib.rs"));
        assert!(!is_measured("crates/athena-cli/tests/lifecycle.rs"));
        assert!(!is_measured("crates/athena-gate/src/landing_tests.rs"));
        assert!(!is_measured("crates/athena-core/src/README.md"));
    }

    #[test]
    fn breach_reports_each_threshold_and_spares_a_file_under_them() {
        let measure = |lines, longest_fn| Measure {
            lines,
            longest_fn,
            longest_fn_name: "f".into(),
        };
        // Over: 2001 lines, a 201-line function, 1501 lines with 5 landings.
        assert_eq!(
            breaches(&measure(2001, 10), 0),
            vec![Breach::FileLength { lines: 2001 }]
        );
        assert_eq!(
            breaches(&measure(100, 201), 0),
            vec![Breach::FunctionLength {
                name: "f".into(),
                lines: 201
            }]
        );
        assert_eq!(
            breaches(&measure(1501, 10), 5),
            vec![Breach::BusyFile {
                lines: 1501,
                landings: 5
            }]
        );
        // Not over: at the limits, or large but quiet, or busy but small.
        assert!(breaches(&measure(2000, 200), 0).is_empty());
        assert!(breaches(&measure(1501, 10), 4).is_empty());
        assert!(breaches(&measure(1500, 10), 9).is_empty());
        assert!(breaches(&measure(300, 30), 20).is_empty());
        assert!(
            Breach::BusyFile {
                lines: 1501,
                landings: 5
            }
            .text()
            .contains("1501")
        );
    }

    #[test]
    fn the_table_aligns_its_columns_under_the_headers() {
        let file = |path: &str, lines, over: bool| FileHealth {
            path: path.into(),
            lines,
            longest_fn: 12,
            longest_fn_name: "f".into(),
            weekly_landings: 3,
            citations: 2,
            limit_lines: MAX_FILE_LINES,
            limit_fn: MAX_FN_LINES,
            breaches: if over {
                vec![Breach::FileLength { lines }]
            } else {
                vec![]
            },
        };
        let health = CodeHealth {
            files: vec![
                file("a/very/long/path/to/file.rs", 2100, true),
                file("b.rs", 30, false),
            ],
            suite_seconds: Some(540.0),
        };
        let table = table_text(&health);
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(lines.len(), 3);
        let column = |line: &str, text: &str| line.find(text).unwrap();
        assert_eq!(column(lines[0], "LINES"), column(lines[1], "2100"));
        assert_eq!(column(lines[0], "LINES"), column(lines[2], "30"));
        assert_eq!(
            column(lines[0], "OVER"),
            column(lines[1], "yes: file length")
        );
        assert_eq!(column(lines[0], "OVER"), column(lines[2], "no"));
        assert!(suite_line(health.suite_seconds).contains("540.0s"));
        assert!(suite_line(None).contains("unknown"));
    }

    // -------------------------------------------------------------------------------------------------
    // A scratch repository

    /// An Athena over the repository at `dir` with the state directory `state`.
    pub(crate) fn athena_over(dir: &std::path::Path, state: &std::path::Path) -> Athena {
        Athena {
            git: Git::at(dir),
            state: crate::state::StateDir::new(state.join("repo"))
                .with_free_space(crate::test_support::plenty_of_space),
            driver: std::sync::Arc::new(FakeAgentDriver::deterministic()),
            capabilities: vec![],
            build: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            athena_bin: "/bin/false".into(),
            nested: None,
            pinner: Box::new(crate::fake::CountingPinner::new(&"0".repeat(40))),
            jev: None,
        }
    }

    pub(crate) fn athena_in(dir: &std::path::Path, state: &std::path::Path) -> Athena {
        let git = Git::at(dir);
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value]).unwrap();
        }
        // The disk guard compares against the machine's free space; tests run with no minimum.
        std::fs::write(
            dir.join("athena.toml"),
            format!(
                "{}\n[disk]\nmin_free_gb = 0\n",
                include_str!("../../../athena.toml")
            ),
        )
        .unwrap();
        std::fs::write(
            dir.join("defaults.toml"),
            include_str!("../../../defaults.toml"),
        )
        .unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "Start"]).unwrap();
        athena_over(dir, state)
    }

    /// Writes `files` and commits them as a landing made `days_ago` days before `now`.
    pub(crate) fn land(
        athena: &Athena,
        files: &[(&str, String)],
        job: &str,
        now: Timestamp,
        days_ago: i64,
    ) {
        let when = now.as_second() - days_ago * 24 * 3600;
        let dated = Git::at(athena.git.dir())
            .with_env("GIT_COMMITTER_DATE", format!("{when} +0000"))
            .with_env("GIT_AUTHOR_DATE", format!("{when} +0000"));
        for (path, text) in files {
            let full = athena.git.dir().join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, text).unwrap();
        }
        dated.run(["add", "-A"]).unwrap();
        dated
            .run([
                "commit",
                "-q",
                "-m",
                &format!("Land {job}\n\nAthena-Job: {job}\nAthena-Verified-Content: fp-{job}\n"),
            ])
            .unwrap();
    }

    pub(crate) fn big_file() -> String {
        // 2001 production lines and a test module that does not count.
        format!(
            "{}#[cfg(test)]\nmod tests {{\n    #[test]\n    fn t() {{}}\n}}\n",
            filler(2001)
        )
    }

    pub(crate) fn open_requests(athena: &Athena) -> Vec<String> {
        let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        let mut out: Vec<String> = records
            .request_jobs()
            .iter()
            .map(|j| j.request.as_ref().unwrap().text.clone())
            .collect();
        out.sort();
        out
    }

    pub(crate) fn set_up() -> (tempfile::TempDir, tempfile::TempDir, Athena) {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        (dir, state, athena)
    }

    #[test]
    fn code_health_counts_lines_functions_weekly_landings_and_citations() {
        let (_dir, _state, athena) = set_up();
        let now = crate::now();
        let source = format!(
            "{}{}#[cfg(test)]\nmod tests {{\n{}}}\n",
            function("small", 2),
            function("larger", 20),
            filler(40)
        );
        land(
            &athena,
            &[
                ("crates/x/src/a.rs", source),
                ("crates/x/src/b.rs", function("b", 1)),
            ],
            "j1",
            now,
            2,
        );
        land(
            &athena,
            &[("crates/x/src/a.rs", function("small", 3))],
            "j2",
            now,
            1,
        );
        // Too old to count in the week.
        land(
            &athena,
            &[("crates/x/src/b.rs", function("b", 2))],
            "j3",
            now,
            30,
        );
        // A landing without the verified-content trailer is no landing.
        std::fs::write(athena.git.dir().join("crates/x/src/b.rs"), function("b", 3)).unwrap();
        athena.git.run(["add", "-A"]).unwrap();
        athena
            .git
            .run(["commit", "-q", "-m", "Records only\n\nAthena-Job: j9"])
            .unwrap();
        // The last landing rewrote a.rs: restore the measured source for the numbers below.
        let source = format!(
            "{}{}#[cfg(test)]\nmod tests {{\n{}}}\n",
            function("small", 2),
            function("larger", 20),
            filler(40)
        );
        land(&athena, &[("crates/x/src/a.rs", source)], "j4", now, 0);

        // Two rejected jobs and a failed one cite a.rs; a closed-as-landed one and an unrelated one do not.
        athena.queue("Fix crates/x/src/a.rs").unwrap();

        let head = athena.head().unwrap();
        let records = Records::load(&athena.git, &head).unwrap();
        let health = compute(&athena.git, &head, &records, now).unwrap();
        let a = health
            .files
            .iter()
            .find(|f| f.path == "crates/x/src/a.rs")
            .unwrap();
        // 4 + 22 function lines; the test module is not counted.
        assert_eq!(a.lines, 26);
        assert_eq!(a.longest_fn, 22);
        assert_eq!(a.longest_fn_name, "larger");
        // j1, j2 and j4 touched it in the week.
        assert_eq!(a.weekly_landings, 3);
        assert_eq!(a.citations, 0, "a queued request is no failure");
        let b = health
            .files
            .iter()
            .find(|f| f.path == "crates/x/src/b.rs")
            .unwrap();
        assert_eq!(
            b.weekly_landings, 1,
            "j1 only: j3 is old and the records-only commit is no landing"
        );
        assert!(health.over().is_empty());
        assert_eq!(health.suite_seconds, None);
    }

    /// Closes a queued job with an outcome record.
    pub(crate) fn close_job(athena: &Athena, job: &JobId, state: OutcomeState, now: Timestamp) {
        let outcome = AnyRecord::Outcome(Box::new(athena_data::records::Outcome {
            job: job.clone(),
            state,
            reason: "test".into(),
            at: now,
            totals: Some(crate::scoring::totals(&[], None)),
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        }));
        athena
            .commit_records(
                &[NewFile::record(&outcome)],
                &message(
                    "Close a test job",
                    "A test outcome.",
                    &[("Athena-Job", job.as_str().to_string())],
                ),
            )
            .unwrap();
    }

    /// `now` and `seconds` later: baselines are ordered by time.
    fn later(now: Timestamp, seconds: i64) -> Timestamp {
        now.checked_add(jiff::SignedDuration::from_secs(seconds))
            .unwrap()
    }

    #[test]
    fn citations_count_rejected_failed_and_plan_stopped_jobs_that_name_the_file() {
        let (_dir, _state, athena) = set_up();
        let now = crate::now();
        land(
            &athena,
            &[("crates/x/src/a.rs", function("a", 1))],
            "j1",
            now,
            0,
        );
        let mut ids = Vec::new();
        for text in [
            "Rework crates/x/src/a.rs",
            "Also crates/x/src/a.rs please",
            "Other work on crates/x/src/other.rs",
            "Fine work on crates/x/src/a.rs",
        ] {
            ids.push(athena.queue(text).unwrap().0);
        }
        for (job, state) in [
            (&ids[0], OutcomeState::Rejected),
            (&ids[1], OutcomeState::Failed),
            (&ids[2], OutcomeState::Rejected),
            (&ids[3], OutcomeState::Abandoned),
        ] {
            close_job(&athena, job, state, now);
        }
        let head = athena.head().unwrap();
        let records = Records::load(&athena.git, &head).unwrap();
        let health = compute(&athena.git, &head, &records, now).unwrap();
        assert_eq!(health.files[0].citations, 2);
    }

    fn structural_requests(athena: &Athena) -> Vec<String> {
        open_requests(athena)
            .into_iter()
            .filter(|t| t.starts_with(STRUCTURAL_MARKER))
            .collect()
    }

    #[test]
    fn structural_pass_queues_one_request_citing_the_numbers_and_none_on_a_rerun() {
        let (_dir, _state, athena) = set_up();
        let now = crate::now();
        land(
            &athena,
            &[
                ("crates/x/src/big.rs", big_file()),
                ("crates/x/src/ok.rs", function("ok", 3)),
            ],
            "j1",
            now,
            0,
        );
        let first = athena.code_health_check(now).unwrap();
        assert_eq!(first.structural.len(), 1);
        assert_eq!(first.structural[0].0, "crates/x/src/big.rs");
        let requests = structural_requests(&athena);
        assert_eq!(requests.len(), 1, "{requests:?}");
        let text = &requests[0];
        assert!(
            text.starts_with("Structural pass: crates/x/src/big.rs\n"),
            "{text}"
        );
        assert!(text.contains("2001 production lines"), "{text}");
        assert!(text.contains("1 landings in the last 7 days"), "{text}");
        assert!(text.contains("0 failure or rejection citations"), "{text}");
        // The file under the limits has no request.
        assert!(!text.contains("ok.rs"));

        // A second run queues none: the first request is still open.
        let second = athena.code_health_check(now).unwrap();
        assert!(second.structural.is_empty());
        assert_eq!(structural_requests(&athena).len(), 1);
    }

    #[test]
    fn structural_pass_queues_nothing_for_a_file_with_an_open_request() {
        let (_dir, _state, athena) = set_up();
        let now = crate::now();
        land(
            &athena,
            &[("crates/x/src/big.rs", big_file())],
            "j1",
            now,
            0,
        );
        athena
            .queue("Structural pass: crates/x/src/big.rs\n\nQueued by hand earlier.")
            .unwrap();
        let report = athena.code_health_check(now).unwrap();
        assert!(report.structural.is_empty());
        assert_eq!(structural_requests(&athena).len(), 1);
    }

    #[test]
    fn structural_pass_for_a_long_function_and_a_busy_file() {
        let (_dir, _state, athena) = set_up();
        let now = crate::now();
        land(
            &athena,
            &[("crates/x/src/long.rs", function("sprawl", 200))],
            "j1",
            now,
            0,
        );
        let busy = filler(1501);
        for n in 0..5 {
            land(
                &athena,
                &[(
                    "crates/x/src/busy.rs",
                    format!("{busy}const N: u32 = {n};\n"),
                )],
                &format!("b{n}"),
                now,
                1,
            );
        }
        let report = athena.code_health_check(now).unwrap();
        let paths: Vec<&str> = report.structural.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(paths, ["crates/x/src/busy.rs", "crates/x/src/long.rs"]);
        let requests = structural_requests(&athena);
        assert!(
            requests
                .iter()
                .any(|t| t.contains("function sprawl is 202 lines"))
        );
        assert!(
            requests
                .iter()
                .any(|t| t.contains("5 landings in the last 7 days"))
        );
    }

    #[test]
    fn the_first_landing_sets_the_baseline_and_queues_no_review() {
        let (dir, _state, athena) = set_up();
        let now = crate::now();
        land(
            &athena,
            &[("crates/x/src/ok.rs", function("ok", 1))],
            "j1",
            now,
            0,
        );
        let report = athena.code_health_check(now).unwrap();
        assert!(report.first_baseline);
        assert!(report.review.is_none());
        let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        assert_eq!(records.code_health_baselines.len(), 1);
        assert_eq!(records.code_health_baselines[0].landings, 1);
        assert!(!records.code_health_baselines[0].review_queued);
        assert!(open_requests(&athena).is_empty());

        // The baseline is a record in git: a fresh state directory over the same repository finds it, so
        // the next check neither sets another baseline nor queues a review.
        let fresh_state = tempfile::tempdir().unwrap();
        let fresh = athena_over(dir.path(), fresh_state.path());
        let reloaded = Records::load(&fresh.git, &fresh.head().unwrap()).unwrap();
        assert_eq!(
            reloaded.code_health_baselines,
            records.code_health_baselines
        );
        let again = fresh.code_health_check(now).unwrap();
        assert!(!again.first_baseline);
        assert!(again.review.is_none());
        let after = Records::load(&fresh.git, &fresh.head().unwrap()).unwrap();
        assert_eq!(after.code_health_baselines.len(), 1);
    }

    /// Lands `count` small changes, named from `first`.
    fn land_many(athena: &Athena, first: usize, count: usize, now: Timestamp) {
        for n in first..first + count {
            land(
                athena,
                &[("crates/x/src/ok.rs", function("ok", n))],
                &format!("j{n}"),
                now,
                0,
            );
        }
    }

    #[test]
    fn design_review_cadence_queues_on_the_tenth_landing_after_the_baseline_not_the_ninth() {
        let (_dir, _state, athena) = set_up();
        let now = crate::now();
        land_many(&athena, 1, 1, now);
        assert!(athena.code_health_check(now).unwrap().first_baseline);
        land_many(&athena, 2, 9, now);
        let ninth = athena.code_health_check(now).unwrap();
        assert!(ninth.review.is_none(), "nine landings after the baseline");
        assert!(open_requests(&athena).is_empty());

        land_many(&athena, 11, 1, now);
        let tenth = athena.code_health_check(later(now, 60)).unwrap();
        let job = tenth.review.expect("the tenth landing queues the review");
        let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        let queued = records.jobs.get(job.as_str()).unwrap();
        assert_eq!(queued.priority(), crate::plan::Priority::High);
        assert_eq!(queued.state(), JobState::Queued);
        // A new baseline counts from here: the next check queues nothing.
        let latest = CodeHealthBaseline::latest(&records.code_health_baselines).unwrap();
        assert_eq!(latest.landings, 11);
        assert!(latest.review_queued);
        let rerun = athena.code_health_check(later(now, 120)).unwrap();
        assert!(rerun.review.is_none());
        assert_eq!(open_requests(&athena).len(), 1);
    }

    #[test]
    fn design_review_text_names_what_to_review_and_what_to_produce() {
        let (_dir, _state, athena) = set_up();
        let now = crate::now();
        land(
            &athena,
            &[("crates/x/src/big.rs", big_file())],
            "j1",
            now,
            0,
        );
        let (rejected, _) = athena
            .queue("Rework crates/x/src/big.rs for speed")
            .unwrap();
        close_job(&athena, &rejected, OutcomeState::Rejected, now);
        assert!(athena.code_health_check(now).unwrap().first_baseline);
        land_many(&athena, 2, 10, now);
        let report = athena.code_health_check(later(now, 60)).unwrap();
        let review = report.review.expect("a review is queued");
        let records = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        let text = records.jobs[review.as_str()]
            .request
            .as_ref()
            .unwrap()
            .text
            .clone();
        for phrase in [
            "Priority: high",
            "self-contained",
            "code-health table",
            "LONGEST FN",
            "crates/x/src/big.rs",
            "files most changed since the last review",
            "crates/x/src/ok.rs: 10",
            "files most broken since the last review",
            "open structural-pass requests",
            "Structural pass: crates/x/src/big.rs",
            "recent rejections and planning stops",
            rejected.as_str(),
            "short design note with proposed changes",
            "design-only job",
        ] {
            assert!(text.contains(phrase), "missing {phrase:?} in:\n{text}");
        }
    }
}
