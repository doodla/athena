//! The final verdict, and the type that alone can land (design §6.3, §7.2; M7, M16).
//!
//! The core computes the final verdict. It passes only if the agent says pass, every criterion is met, no
//! finding is a blocker or major, and every check passed or was already failing on the base commit. Each `behavior` criterion must cite at least one
//! command, and every cited command must be one the core saw run, either as a whole command or
//! as one whole simple command inside a chained or piped one; otherwise the criterion counts as unmet, with a
//! finding that says so (D43, D63). A cited command is seen if the verifier ran it, or Athena ran and
//! recorded it on the same candidate tree and it passed; a failed record, or one made on another tree, does
//! not count (D115).
//!
//! [`Verified`] has private fields and is constructed only here, only for a pass, so `land()` cannot be
//! called with anything unverified.

use crate::reading::REPLAY_HOLD_PERCENT;
use athena_context::outputs::{SeverityOutput, VerdictOutput, VerdictValue};
use athena_data::records::{
    Baseline, CheckRun, Criterion, CriterionKind, CriterionResult, Demonstration, Finding,
    FindingSource, PRE_EXISTING_NOTE, Replay, Severity, Verdict, Verification,
};

/// A candidate that passed verification: the only input landing accepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verified {
    base: String,
    candidate: String,
    fingerprint: String,
}

impl Verified {
    pub fn base(&self) -> &str {
        &self.base
    }
    pub fn candidate(&self) -> &str {
        &self.candidate
    }
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// Rebuilds the value from a committed verification that passed, for retrying a blocked landing.
    /// `fingerprint_now` is the candidate's fingerprint recomputed from git; it must equal the recorded one.
    pub fn from_record(
        verification: &Verification,
        base: &str,
        fingerprint_now: &str,
    ) -> Option<Verified> {
        (verification.final_verdict == Verdict::Pass
            && verification.verified_fingerprint == fingerprint_now)
            .then(|| Verified {
                base: base.to_string(),
                candidate: verification.candidate_commit.clone(),
                fingerprint: fingerprint_now.to_string(),
            })
    }
}

/// One recorded check as the verifier sees it: the command, the conclusion, and the note (or the test count).
///
/// A failed check whose result was reused from a CI run also carries the failing tests, the failing units
/// and the end of the log, read from the CI log when the check was recorded, so the verifier runs only those
/// tests. A check Athena ran itself shows none of that: its brief line is as it always was.
pub fn check_result_view(r: &CheckRun) -> athena_context::briefs::CheckResultView {
    use crate::checks::FAILING_UNITS_PREFIX;
    let mut view = athena_context::briefs::CheckResultView {
        name: r.name.clone(),
        command: r.command.join(" "),
        passed: r.passed,
        detail: match (r.tests_ran, &r.note) {
            (_, Some(note)) => note.clone(),
            (Some(n), None) => format!("{n} tests ran"),
            (None, None) => String::new(),
        },
        ..Default::default()
    };
    if r.reused && !r.passed {
        view.failed_tests = r.failed_tests.clone();
        let (header, rest) = match r.output_tail.split_once('\n') {
            Some((first, rest)) if first.starts_with(FAILING_UNITS_PREFIX) => (first, rest),
            _ => ("", r.output_tail.as_str()),
        };
        view.failing_units = header
            .strip_prefix(FAILING_UNITS_PREFIX)
            .unwrap_or_default()
            .to_string();
        view.output_tail = rest.to_string();
    }
    view
}

/// The line Athena writes into the verifier's input about a reused check result: where it came from, the
/// SHA it is for, and each check's conclusion. The verifier never fetches the result itself; it gets this
/// text only when the record's SHA equals the candidate's, and nothing otherwise.
pub fn reuse_statement(record: &crate::ci_result::CiRecord, candidate_sha: &str) -> Option<String> {
    if record.sha != candidate_sha {
        return None;
    }
    let origin = match (&record.source, &record.run_address) {
        (crate::ci_result::Source::Ci, Some(address)) => format!("the CI run {address}"),
        (crate::ci_result::Source::Ci, None) => "a CI run".to_string(),
        (crate::ci_result::Source::Local, _) => "Athena's local run".to_string(),
    };
    let steps: Vec<String> = record
        .steps
        .iter()
        .map(|s| format!("{} {}", s.name, s.conclusion))
        .collect();
    Some(format!(
        "Check results for {candidate_sha} come from {origin} ({:.0}s): {}.",
        record.duration_s,
        steps.join(", ")
    ))
}

fn severity(s: SeverityOutput) -> Severity {
    match s {
        SeverityOutput::Blocker => Severity::Blocker,
        SeverityOutput::Major => Severity::Major,
        SeverityOutput::Minor => Severity::Minor,
        SeverityOutput::Nit => Severity::Nit,
    }
}

/// The parts of a verification the verdict decides.
#[derive(Debug, Clone, PartialEq)]
pub struct Judgment {
    pub criteria: Vec<CriterionResult>,
    pub findings: Vec<Finding>,
    pub agent_verdict: Option<Verdict>,
    pub final_verdict: Verdict,
    pub reasons: Vec<String>,
}

/// Why the verifier gave no verdict.
///
/// The root cause this type guards against: a verifier killed at its deadline was recorded as the generic
/// `verifier_failure` string in `run/verification.rs`, and `judge` failed the verdict on it exactly as it does
/// for a judged failure, so criteria were rejected that nobody had judged. A timeout is its own variant: the
/// core reruns it once, and a second timeout leaves the criteria unjudged. Neither variant ever yields a pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifierFailure {
    /// The verifier could not run, errored, returned nothing usable, or the system could not keep its
    /// snapshot still or its budget ran out. The detail is the cause.
    Incomplete(String),
    /// The verifier was stopped at its deadline. The criteria were not judged. The detail is the cause.
    TimedOut(String),
}

/// The start of the reason `judge` records for a verifier that timed out; the CLI keys off it.
pub const UNJUDGED_REASON_PREFIX: &str = "the verifier timed out, so the criteria are unjudged";

/// What the verdict is computed from.
#[derive(Debug)]
pub struct VerdictInputs<'a> {
    pub criteria: &'a [Criterion],
    pub checks: &'a [CheckRun],
    /// The verifier's structured output, if it produced one.
    pub output: Option<&'a VerdictOutput>,
    /// The commands the core saw the verifier run. A cited command is seen if it is here, or if Athena ran
    /// and recorded it in `checks` on the same candidate tree and it passed; failed or other-tree records
    /// do not count (D43, D115).
    pub commands_run: &'a [String],
    /// The tree being judged. With `checks_tree`, decides whether `checks` count as seen commands.
    pub candidate_tree: Option<&'a str>,
    /// The tree Athena ran `checks` on, or `None` when it ran none. A recorded check counts as a seen
    /// command only if this equals `candidate_tree` and the check passed with exit code 0 (D115).
    pub checks_tree: Option<&'a str>,
    /// Set when the verifier changed its snapshot. The core discarded that verdict and reran verification;
    /// the fault is the system's, so it is recorded and never counted against the work.
    pub system_fault: Option<String>,
    /// Set when the verifier did not complete, or timed out and left the criteria unjudged.
    pub verifier_failure: Option<VerifierFailure>,
    /// `docs/design/requirements-map.md` at the base, if the file exists there.
    pub map_base: Option<&'a str>,
    /// The same file at the candidate, if it exists there.
    pub map_candidate: Option<&'a str>,
    /// The checks on the base commit, if a usable baseline for this job's base exists. A check that failed
    /// there and fails again is pre-existing and does not fail the verdict.
    pub baseline: Option<&'a Baseline>,
    /// The replay of the newest accepted readings through the candidate's reading rules. A held replay is a
    /// blocker: the candidate newly refuses too many readings the base accepted.
    pub replay: Option<&'a Replay>,
}

/// True when `check` failed now and the baseline shows the same check failing on the base commit, with no
/// failing test the base did not also fail.
///
/// The root cause this guards against: the baseline once kept one pass/fail bit per check name, so a check
/// that failed on the base excused every failure in it, including a new, different failing test. When both
/// runs name their failing tests ([`CheckRun::failed_tests`]), every failing test now must also have failed on
/// the base, or the check is the job's own failure. When either side names none (a timeout, a build error, a
/// check that is not a plain `cargo test`, a reused result whose log named no test, an older record) the
/// check-name rule stands. A reused red CI check whose log named failing tests is compared test by test like a
/// local one; its names carry an entry in parentheses (`test (shard 2) (log not read)`) when a shard's log was
/// unread, listed nothing, or did not finish, and no base has such a name, so a partly read set of shards is
/// never excused by the tests that were listed. A failure that comes only from a concurrent change merged after the base is not in the baseline and is
/// not separated here.
pub fn is_pre_existing(check: &CheckRun, baseline: Option<&Baseline>) -> bool {
    let Some(base) = baseline.filter(|b| !check.passed && b.failed_before(&check.name)) else {
        return false;
    };
    let before = base.failed_tests_before(&check.name);
    before.is_empty()
        || check.failed_tests.is_empty()
        || check.failed_tests.iter().all(|t| before.contains(t))
}

/// The failing tests of `check` split into those the base did not fail (newly failing) and those it also
/// failed (already failing on the base). Each list keeps the order of `check.failed_tests`.
pub fn failure_split(check: &CheckRun, baseline: Option<&Baseline>) -> (Vec<String>, Vec<String>) {
    let before = baseline.map_or(&[][..], |b| b.failed_tests_before(&check.name));
    check
        .failed_tests
        .iter()
        .cloned()
        .partition(|t| !before.contains(t))
}

/// Prefixes the note of each pre-existing failed check with [`PRE_EXISTING_NOTE`], so the record shows
/// which failures were already on the base. Safe to call twice.
pub fn mark_pre_existing(checks: &mut [CheckRun], baseline: Option<&Baseline>) {
    for check in checks.iter_mut() {
        if !is_pre_existing(check, baseline) {
            continue;
        }
        match &check.note {
            Some(note) if note.starts_with(PRE_EXISTING_NOTE) => {}
            Some(note) => check.note = Some(format!("{PRE_EXISTING_NOTE}; {note}")),
            None => check.note = Some(PRE_EXISTING_NOTE.to_string()),
        }
    }
}

/// What a system fault's finding starts with.
pub const SYSTEM_FAULT_PREFIX: &str = "System fault";

/// The requirements map, relative to the repository root.
pub const MAP_PATH: &str = "docs/design/requirements-map.md";

/// The map rows as `(ID, covered)`, in order. A row is a table line whose first cell looks like an ID
/// (`A9-1`); it is covered when the first words of its last cell are `Covered`. Same rule as the strategy
/// view's parser in the CLI, which core cannot call.
fn map_rows(text: &str) -> Vec<(&str, bool)> {
    text.lines()
        .filter_map(|line| {
            let inner = line.trim().strip_prefix('|')?;
            let first = inner.split('|').next().unwrap_or("").trim();
            let is_id = first.starts_with(|c: char| c.is_ascii_uppercase())
                && first.contains('-')
                && first.ends_with(|c: char| c.is_ascii_digit())
                && first.chars().all(|c| c.is_ascii_alphanumeric() || c == '-');
            if !is_id {
                return None;
            }
            let last = inner
                .trim_end()
                .trim_end_matches('|')
                .rsplit('|')
                .next()
                .unwrap_or("")
                .trim();
            Some((first, last.starts_with("Covered")))
        })
        .collect()
}

/// The IDs of rows that are Covered in the candidate's map but were not Covered at the base: moved to
/// Covered, or added already Covered. A job may not mark its own requirement rows covered.
fn rows_newly_covered(base: Option<&str>, candidate: Option<&str>) -> Vec<String> {
    let Some(candidate) = candidate else {
        return Vec::new();
    };
    let before = map_rows(base.unwrap_or(""));
    map_rows(candidate)
        .into_iter()
        .filter(|(id, covered)| {
            *covered
                && !before
                    .iter()
                    .any(|(old, was_covered)| old == id && *was_covered)
        })
        .map(|(id, _)| id.to_string())
        .collect()
}

fn athena_finding(severity: Severity, summary: String, evidence: String) -> Finding {
    Finding {
        severity,
        summary,
        evidence,
        location: None,
        source: FindingSource::Athena,
    }
}

/// Splits a command line into its whole simple commands on `&&`, `||`, `;`, `|`, and newlines, never inside
/// quotes. A lone `&` (as in `2>&1`) is not a separator, and a trailing `2>&1` is dropped. Whitespace outside
/// quotes is trimmed and collapsed.
fn simple_commands(line: &str) -> Vec<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut i = 0;
    let mut flush = |cur: &mut String| {
        let piece = cur.trim();
        // Merging stderr into stdout changes no argument, so `cmd 2>&1` and `cmd` are the same command.
        let piece = piece
            .strip_suffix(" 2>&1")
            .map_or(piece, |rest| rest.trim_end());
        if !piece.is_empty() {
            out.push(piece.to_string());
        }
        cur.clear();
    };
    while i < chars.len() {
        let c = chars[i];
        match quote {
            Some(q) => {
                cur.push(c);
                if c == '\\' && q == '"' && i + 1 < chars.len() {
                    i += 1;
                    cur.push(chars[i]);
                } else if c == q {
                    quote = None;
                }
            }
            None => match c {
                '\'' | '"' => {
                    quote = Some(c);
                    cur.push(c);
                }
                '\\' if i + 1 < chars.len() => {
                    cur.push(c);
                    i += 1;
                    cur.push(chars[i]);
                }
                ';' | '\n' | '|' => {
                    if c == '|' && chars.get(i + 1) == Some(&'|') {
                        i += 1;
                    }
                    flush(&mut cur);
                }
                '&' if chars.get(i + 1) == Some(&'&') => {
                    i += 1;
                    flush(&mut cur);
                }
                c if c.is_whitespace() => {
                    if !cur.is_empty() && !cur.ends_with(' ') {
                        cur.push(' ');
                    }
                }
                _ => cur.push(c),
            },
        }
        i += 1;
    }
    flush(&mut cur);
    out
}

/// Computes the final verdict.
pub fn judge(inputs: &VerdictInputs<'_>) -> Judgment {
    let mut findings = Vec::new();
    let mut reasons = Vec::new();
    let mut ran: Vec<String> = inputs
        .commands_run
        .iter()
        .flat_map(|c| simple_commands(c))
        .collect();
    // Athena's own passing record for this very tree counts as a command run (D115). So does a passing
    // record reused from a green CI run for the candidate commit: it has no exit code but is marked
    // `reused`. Skipped checks and nested-run placeholders carry no exit code and are not reused, so they
    // never count.
    if inputs.candidate_tree.is_some() && inputs.candidate_tree == inputs.checks_tree {
        for check in inputs
            .checks
            .iter()
            .filter(|c| c.passed && (c.exit_code == Some(0) || (c.reused && c.exit_code.is_none())))
        {
            ran.extend(simple_commands(&check.command.join(" ")));
        }
    }

    let blocking_checks = |c: &&CheckRun| !c.passed && !is_pre_existing(c, inputs.baseline);
    for check in inputs.checks.iter().filter(blocking_checks) {
        reasons.push(format!("check {} failed", check.name));
        // The check already fails the verdict; this finding only shows which failures are the job's own, so
        // it is Minor and a failure is not counted twice.
        if !check.failed_tests.is_empty() {
            let (new, already) = failure_split(check, inputs.baseline);
            let mut parts = vec![format!("newly failing: {}", new.join(", "))];
            if !already.is_empty() {
                parts.push(format!(
                    "already failing on the base: {}",
                    already.join(", ")
                ));
            }
            findings.push(athena_finding(
                Severity::Minor,
                format!(
                    "Check {}: {} test(s) newly failing, {} already failing on the base",
                    check.name,
                    new.len(),
                    already.len()
                ),
                parts.join("; "),
            ));
        }
    }
    match &inputs.verifier_failure {
        Some(VerifierFailure::Incomplete(failure)) => {
            reasons.push(format!("the verifier did not complete: {failure}"));
        }
        Some(VerifierFailure::TimedOut(failure)) => {
            reasons.push(format!("{UNJUDGED_REASON_PREFIX}: {failure}"));
        }
        None => {}
    }
    let unjudged = matches!(inputs.verifier_failure, Some(VerifierFailure::TimedOut(_)));
    if let Some(change) = &inputs.system_fault {
        findings.push(athena_finding(
            Severity::Minor,
            format!("{SYSTEM_FAULT_PREFIX}: the verifier changed its snapshot; its verdict was discarded and verification reran"),
            change.clone(),
        ));
    }

    for id in rows_newly_covered(inputs.map_base, inputs.map_candidate) {
        reasons.push(format!("the change marked requirement row {id} Covered"));
        findings.push(athena_finding(
            Severity::Major,
            format!("The change marked requirement row {id} Covered; a job may not mark its own rows covered"),
            format!("{id} is Covered in docs/design/requirements-map.md at the candidate and was not Covered at the base"),
        ));
    }

    if let Some(replay) = inputs.replay.filter(|r| r.held) {
        let list: Vec<String> = replay
            .newly_refused
            .iter()
            .map(|r| format!("{}: {}", r.job, r.reason))
            .collect();
        reasons.push(format!(
            "the change newly refuses {} of {} replayed accepted readings",
            replay.newly_refused.len(),
            replay.total
        ));
        findings.push(athena_finding(
            Severity::Blocker,
            format!(
                "The change newly refuses {} of {} accepted readings the base accepted ({})",
                replay.newly_refused.len(),
                replay.total,
                replay
                    .newly_refused
                    .iter()
                    .map(|r| r.job.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            format!(
                "more than {REPLAY_HOLD_PERCENT}% of the newest {} accepted readings are refused by the candidate and accepted by the base, more than the share the criteria declare (plus 5 points) or, when they declare none or declare one above 25% without a reason, more than that limit: {}",
                replay.total,
                list.join("; ")
            ),
        ));
    }

    let mut criteria = Vec::new();
    for (i, criterion) in inputs.criteria.iter().enumerate() {
        let given = inputs.output.and_then(|o| o.criteria.get(i));
        let Some(given) = given else {
            criteria.push(CriterionResult {
                criterion: criterion.text.clone(),
                kind: criterion.kind,
                met: false,
                evidence: if unjudged {
                    "unjudged: the verifier timed out before it assessed this criterion".into()
                } else {
                    "the verifier did not assess this criterion".into()
                },
                commands: Vec::new(),
                commands_verified: false,
            });
            if inputs.output.is_some() {
                findings.push(athena_finding(
                    Severity::Major,
                    format!("Criterion {} was not assessed", i + 1),
                    criterion.text.clone(),
                ));
            }
            continue;
        };
        let commands: Vec<String> = given
            .commands
            .iter()
            .map(|c| c.trim().to_string())
            .collect();
        let unseen: Vec<&String> = commands
            .iter()
            .filter(|c| {
                let pieces = simple_commands(c);
                pieces.is_empty() || !pieces.iter().all(|p| ran.contains(p))
            })
            .collect();
        let commands_verified = match criterion.kind {
            CriterionKind::Behavior => !commands.is_empty() && unseen.is_empty(),
            CriterionKind::Content => unseen.is_empty(),
        };
        let mut met = given.met;
        if criterion.kind == CriterionKind::Behavior && !commands_verified {
            met = false;
            let evidence = if commands.is_empty() {
                "it cites no command".to_string()
            } else {
                format!(
                    "cited but not seen running: {}",
                    unseen
                        .iter()
                        .map(|c| format!("`{c}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            findings.push(athena_finding(
                Severity::Major,
                format!("Behavior criterion {} is unmet: its evidence must cite commands that ran (D43)", i + 1),
                evidence,
            ));
        }
        criteria.push(CriterionResult {
            criterion: criterion.text.clone(),
            kind: criterion.kind,
            met,
            evidence: given.evidence.clone(),
            commands,
            commands_verified,
        });
    }

    if let Some(output) = inputs.output {
        for f in &output.findings {
            findings.push(Finding {
                severity: severity(f.severity),
                summary: f.summary.clone(),
                evidence: f.evidence.clone(),
                location: f.location.clone(),
                source: FindingSource::Verifier,
            });
        }
        if let Some(mismatch) = output
            .request_reading_mismatch
            .as_deref()
            .map(str::trim)
            .filter(|m| !m.is_empty())
        {
            reasons.push("the reading misread the request".into());
            findings.push(athena_finding(
                Severity::Major,
                "The reading does not match the request, so passing criteria do not show the request was met".into(),
                mismatch.to_string(),
            ));
        }
    } else if inputs.verifier_failure.is_none() {
        reasons.push("the verifier returned no verdict".into());
    }

    let agent_verdict = inputs.output.map(|o| {
        if o.verdict == VerdictValue::Pass {
            Verdict::Pass
        } else {
            Verdict::Fail
        }
    });
    if agent_verdict == Some(Verdict::Fail) {
        reasons.push("the verifier's verdict is fail".into());
    }
    let unmet = criteria.iter().filter(|c| !c.met).count();
    if unmet > 0 && inputs.output.is_some() {
        reasons.push(format!("{unmet} of {} criteria unmet", criteria.len()));
    }
    let serious = findings
        .iter()
        .filter(|f| matches!(f.severity, Severity::Blocker | Severity::Major))
        .count();
    if serious > 0 {
        reasons.push(format!("{serious} blocker or major finding(s)"));
    }
    let pass = agent_verdict == Some(Verdict::Pass)
        && unmet == 0
        && serious == 0
        && inputs.checks.iter().filter(blocking_checks).count() == 0
        && inputs.verifier_failure.is_none();
    Judgment {
        criteria,
        findings,
        agent_verdict,
        final_verdict: if pass { Verdict::Pass } else { Verdict::Fail },
        reasons,
    }
}

/// Constructs [`Verified`] from a judgment: `Some` only on a final verdict of pass.
pub fn verified(
    judgment: &Judgment,
    base: &str,
    candidate: &str,
    fingerprint: &str,
) -> Option<Verified> {
    (judgment.final_verdict == Verdict::Pass).then(|| Verified {
        base: base.to_string(),
        candidate: candidate.to_string(),
        fingerprint: fingerprint.to_string(),
    })
}

/// Whether a finding blocks a pass: severity blocker or major. Minor and nit findings do not.
pub fn finding_blocks(finding: &Finding) -> bool {
    matches!(finding.severity, Severity::Blocker | Severity::Major)
}

/// Whether every path a finding cites is a non-code file (Markdown or text, a CHANGELOG, or anything under
/// `docs/`), so that fixing it cannot need a code change.
///
/// The paths come from `location` (tokens split on whitespace, commas and semicolons, each with a trailing
/// `:line[-line]` or `#anchor` removed). Only when `location` is absent are they read from `summary` and
/// `evidence`. A finding that cites no recognizable path is not content-only, and neither is one that cites
/// any code or configuration file (`.rs`, `Cargo.toml`, `athena.toml`, workflows, prompts).
pub fn finding_is_content_only(finding: &Finding) -> bool {
    let paths: Vec<String> = match finding.location.as_deref() {
        Some(location) => location_tokens(location)
            .into_iter()
            .map(path_of_token)
            .collect(),
        None => {
            let prose = format!("{} {}", finding.summary, finding.evidence);
            location_tokens(&prose)
                .into_iter()
                .map(path_of_token)
                .filter(|p| looks_like_path(p))
                .collect()
        }
    };
    !paths.is_empty() && paths.iter().all(|p| is_non_code_path(p))
}

fn location_tokens(text: &str) -> Vec<&str> {
    text.split(|c: char| c.is_whitespace() || c == ',' || c == ';')
        .map(|t| t.trim_matches(|c: char| matches!(c, '(' | ')' | '`' | '\'' | '"' | '.')))
        .filter(|t| !t.is_empty())
        .collect()
}

/// The token without a trailing `#anchor` or `:line[-line]`.
fn path_of_token(token: &str) -> String {
    let token = token.split('#').next().unwrap_or(token);
    let mut path = token;
    if let Some((head, tail)) = token.rsplit_once(':')
        && !tail.is_empty()
        && tail.chars().all(|c| c.is_ascii_digit() || c == '-')
    {
        path = head;
    }
    path.trim_start_matches("./").to_string()
}

/// A prose token counts as a path when it has a directory separator or an alphabetic extension.
fn looks_like_path(token: &str) -> bool {
    if token.contains("://") {
        return false;
    }
    if token.contains('/') {
        return true;
    }
    let name = token.rsplit('/').next().unwrap_or(token);
    name.rsplit_once('.').is_some_and(|(stem, ext)| {
        !stem.is_empty() && !ext.is_empty() && ext.chars().all(|c| c.is_ascii_alphabetic())
    }) || name.starts_with("CHANGELOG")
}

fn is_non_code_path(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    if path.starts_with("docs/") || name.starts_with("CHANGELOG") {
        return true;
    }
    name.rsplit_once('.').is_some_and(|(_, ext)| {
        ["md", "markdown", "txt"]
            .iter()
            .any(|e| ext.eq_ignore_ascii_case(e))
    })
}

// ---------------------------------------------------------------------------------------------------------
// The "fails without the fix" demonstration run by Athena
//
// The verifier gets no producer report and cannot change files, so it can never see a planned "fails without
// the fix" run. Athena runs it: in a scratch copy of the candidate the fix is put back to its base version,
// the tests are kept, and the named test command runs. These are the pure parts; `run/checks.rs` does the
// copying and the run.

/// The most distinct commands whose demonstration Athena runs for one job.
pub const MAX_DEMONSTRATION_RUNS: usize = 3;

/// The planned demonstrations of one command: every criterion that names it, and the command to run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemonstrationGroup {
    /// The criterion numbers, in plan order.
    pub criteria: Vec<u32>,
    /// The command as written in the plan, backticks removed and spaces single.
    pub command: String,
    /// The program and arguments when the command is a plain `cargo test ...`; `None` otherwise.
    pub argv: Option<Vec<String>>,
}

/// `command` as program and arguments, when it is a plain `cargo test ...`: backticks removed, split on
/// white space, and no shell metacharacter or quote. Anything else is not run.
pub fn demonstration_command(command: &str) -> Option<Vec<String>> {
    let words: Vec<String> = command
        .replace('`', "")
        .split_whitespace()
        .map(String::from)
        .collect();
    let plain = words.len() >= 2
        && words[0] == "cargo"
        && words[1] == "test"
        && !words
            .iter()
            .any(|w| w.chars().any(|c| "|&;<>()$\\\"'*?{}[]~#!".contains(c)));
    plain.then_some(words)
}

/// The planned demonstrations that name a failure without the fix, grouped by distinct command so that each
/// command runs once. A demonstration without a non-blank `fails_without` is not one.
pub fn demonstration_groups(demonstrations: &[Demonstration]) -> Vec<DemonstrationGroup> {
    let mut groups: Vec<DemonstrationGroup> = Vec::new();
    for d in demonstrations {
        if d.fails_without
            .as_deref()
            .is_none_or(|f| f.trim().is_empty())
        {
            continue;
        }
        let command = d.command.replace('`', "");
        let command = command.split_whitespace().collect::<Vec<_>>().join(" ");
        match groups.iter_mut().find(|g| g.command == command) {
            Some(group) => {
                if !group.criteria.contains(&d.criterion) {
                    group.criteria.push(d.criterion);
                }
            }
            None => groups.push(DemonstrationGroup {
                criteria: vec![d.criterion],
                argv: demonstration_command(&command),
                command,
            }),
        }
    }
    groups
}

/// Whether a path is test code that the revert keeps: a `tests` or `test` directory, `tests.rs`,
/// `*_tests.rs`, `*_test.rs` or `test_*`.
pub fn is_test_path(path: &str) -> bool {
    let mut parts: Vec<&str> = path.split('/').collect();
    let name = parts.pop().unwrap_or_default();
    let stem = name.rsplit_once('.').map_or(name, |(s, _)| s);
    parts.iter().any(|p| *p == "tests" || *p == "test")
        || stem == "tests"
        || stem.ends_with("_tests")
        || stem.ends_with("_test")
        || stem.starts_with("test_")
}

/// The files `mod <module>;` can mean when declared in the Rust file `parent`.
pub fn declared_module_paths(parent: &str, module: &str) -> Vec<String> {
    let (dir, file) = parent.rsplit_once('/').unwrap_or(("", parent));
    let stem = file.strip_suffix(".rs").unwrap_or(file);
    let base = if matches!(stem, "lib" | "main" | "mod") {
        dir.to_string()
    } else if dir.is_empty() {
        stem.to_string()
    } else {
        format!("{dir}/{stem}")
    };
    let join = |tail: &str| {
        if base.is_empty() {
            tail.to_string()
        } else {
            format!("{base}/{tail}")
        }
    };
    vec![
        join(&format!("{module}.rs")),
        join(&format!("{module}/mod.rs")),
    ]
}

/// A Rust file with its fix reverted and its tests kept.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevertedFile {
    pub text: String,
    /// The names of the bodiless `#[cfg(test)] mod x;` declarations carried over from the candidate.
    pub declared: Vec<String>,
}

/// The base text of a Rust file with the candidate's test modules in place of the base's own: the base's
/// `#[cfg(test)] mod` blocks and bodiless declarations are dropped, the candidate's are appended. `None`
/// when either text does not parse; the caller then reverts the whole file.
pub fn revert_keeping_tests(base: &str, candidate: &str) -> Option<RevertedFile> {
    use crate::code_health::test_module_ranges;
    let (b, c) = (
        test_module_ranges(base).ok()?,
        test_module_ranges(candidate).ok()?,
    );
    let ranges = |m: &crate::code_health::TestModules| -> Vec<(usize, usize)> {
        let mut all: Vec<(usize, usize)> = m
            .bodied
            .iter()
            .map(|(s, e)| (*s as usize, *e as usize))
            .chain(
                m.declared
                    .iter()
                    .map(|(s, e, _)| (*s as usize, *e as usize)),
            )
            .collect();
        all.sort_unstable();
        all
    };
    let (base_lines, cand_lines): (Vec<&str>, Vec<&str>) =
        (base.lines().collect(), candidate.lines().collect());
    let inside = |ranges: &[(usize, usize)], line: usize| {
        ranges.iter().any(|(s, e)| (*s..=*e).contains(&line))
    };
    let (base_ranges, cand_ranges) = (ranges(&b), ranges(&c));
    let mut text = String::new();
    for (i, line) in base_lines.iter().enumerate() {
        if !inside(&base_ranges, i + 1) {
            text.push_str(line);
            text.push('\n');
        }
    }
    for (s, e) in &cand_ranges {
        text.push('\n');
        for line in cand_lines.iter().take(*e).skip(s.saturating_sub(1)) {
            text.push_str(line);
            text.push('\n');
        }
    }
    Some(RevertedFile {
        text,
        declared: c.declared.into_iter().map(|(_, _, name)| name).collect(),
    })
}

/// What to do to the scratch copy of the candidate to take the fix out and keep the tests.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RevertPlan {
    /// Files to write, with their new bytes.
    pub write: Vec<(String, Vec<u8>)>,
    /// Files the candidate added; they are removed.
    pub remove: Vec<String>,
    /// Every path that was put back or removed.
    pub reverted: Vec<String>,
    /// Paths whose tests went with the revert.
    pub tests_removed: Vec<String>,
}

/// Plans the revert. `changed` is the candidate's change from the base, each as its status letter (`A`, `M`,
/// `D` or `T`) and path; `base_of` and `candidate_of` read a path's bytes at the base and the candidate.
/// Test files, files holding a declared test module, and `Cargo.toml` and `Cargo.lock` stay as the candidate
/// has them; the checks run `--locked`, so a manifest cannot go back without failing for another reason.
pub fn revert_plan(
    changed: &[(char, String)],
    base_of: &dyn Fn(&str) -> Option<Vec<u8>>,
    candidate_of: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> RevertPlan {
    let text_of = |bytes: Option<Vec<u8>>| bytes.and_then(|b| String::from_utf8(b).ok());
    let has_tests = |text: &str| {
        crate::code_health::test_module_ranges(text).map_or_else(
            |_| text.contains("#[cfg(test)]"),
            |m| !m.bodied.is_empty() || !m.declared.is_empty(),
        )
    };
    // Files that hold a test module declared by a changed file are test code.
    let mut declared_files: Vec<String> = Vec::new();
    for (status, path) in changed {
        if *status == 'D' || !path.ends_with(".rs") {
            continue;
        }
        if let Some(text) = text_of(candidate_of(path))
            && let Ok(modules) = crate::code_health::test_module_ranges(&text)
        {
            for (_, _, name) in &modules.declared {
                for candidate_path in declared_module_paths(path, name) {
                    if changed.iter().any(|(_, p)| *p == candidate_path) {
                        declared_files.push(candidate_path);
                    }
                }
            }
        }
    }
    let mut plan = RevertPlan::default();
    for (status, path) in changed {
        let name = path.rsplit('/').next().unwrap_or(path);
        if matches!(name, "Cargo.toml" | "Cargo.lock")
            || is_test_path(path)
            || declared_files.contains(path)
        {
            continue;
        }
        let candidate_text = text_of(candidate_of(path));
        match status {
            'A' => {
                plan.remove.push(path.clone());
                plan.reverted.push(path.clone());
                if candidate_text.as_deref().is_some_and(has_tests) {
                    plan.tests_removed.push(path.clone());
                }
            }
            'D' => {
                if let Some(bytes) = base_of(path) {
                    plan.write.push((path.clone(), bytes));
                    plan.reverted.push(path.clone());
                }
            }
            _ => {
                let Some(base_bytes) = base_of(path) else {
                    continue;
                };
                let kept = match (&candidate_text, text_of(Some(base_bytes.clone()))) {
                    (Some(candidate), Some(base)) if path.ends_with(".rs") => {
                        revert_keeping_tests(&base, candidate)
                    }
                    _ => None,
                };
                match kept {
                    Some(file) => {
                        let squeeze = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
                        if candidate_text.as_deref().map(squeeze) == Some(squeeze(&file.text)) {
                            continue;
                        }
                        plan.write.push((path.clone(), file.text.into_bytes()));
                    }
                    None => {
                        if path.ends_with(".rs") && candidate_text.as_deref().is_some_and(has_tests)
                        {
                            plan.tests_removed.push(path.clone());
                        }
                        plan.write.push((path.clone(), base_bytes));
                    }
                }
                plan.reverted.push(path.clone());
            }
        }
    }
    plan
}

/// Athena's record of one demonstration run, as an entry under the checks that is not a check: it states no
/// pass or fail of its own, so the recorded fmt, clippy and test checks still decide the verdict. The result
/// comes from the exit code and the count of tests that ran, never from the clipped tail.
pub fn demonstration_view(
    group: &DemonstrationGroup,
    run: &CheckRun,
    plan: &RevertPlan,
) -> athena_context::briefs::CheckResultView {
    let output_has = |text: &str| run.output_tail.contains(text);
    let result = match (run.exit_code, run.tests_ran) {
        (None, _) => format!(
            "the run did not finish{}",
            run.note
                .as_ref()
                .map_or(String::new(), |n| format!(" ({n})"))
        ),
        (Some(0), Some(n)) if n > 0 => {
            "passed without the fix: the revert did not make the test fail".to_string()
        }
        (Some(0), _) => "the command ran no tests without the fix, so it shows nothing".to_string(),
        (Some(code), _) => {
            let cause = if output_has("could not compile") || output_has("error[E") {
                ": the test did not compile"
            } else {
                ""
            };
            format!("the named test command failed without the fix (exit {code}){cause}")
        }
    };
    demonstration_entry(
        group,
        result,
        run.exit_code,
        run.output_tail.clone(),
        plan,
        true,
    )
}

/// The record of a demonstration that did not run, with the reason.
pub fn demonstration_not_run(
    group: &DemonstrationGroup,
    reason: &str,
) -> athena_context::briefs::CheckResultView {
    demonstration_entry(
        group,
        format!("not run: {reason}"),
        None,
        String::new(),
        &RevertPlan::default(),
        false,
    )
}

fn demonstration_entry(
    group: &DemonstrationGroup,
    result: String,
    exit_code: Option<i32>,
    output_tail: String,
    plan: &RevertPlan,
    ran: bool,
) -> athena_context::briefs::CheckResultView {
    athena_context::briefs::CheckResultView {
        name: format!(
            "fails-without-{}",
            group.criteria.first().copied().unwrap_or(0)
        ),
        command: "not a check".into(),
        passed: true,
        demonstration: Some(athena_context::briefs::DemonstrationRunView {
            ran,
            criteria: group.criteria.clone(),
            command: group.command.clone(),
            reverted: plan.reverted.clone(),
            tests_removed: plan.tests_removed.clone(),
            result,
            exit_code,
            output_tail,
        }),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_context::outputs::{CriterionVerdictOutput, FindingOutput};
    use athena_data::{Quantity, Unit};

    fn demo(criterion: u32, command: &str, fails_without: Option<&str>) -> Demonstration {
        Demonstration {
            criterion,
            method: "run".into(),
            command: command.into(),
            observes: "it".into(),
            fails_without: fails_without.map(String::from),
            seam: None,
        }
    }

    #[test]
    fn demonstration_command_accepts_only_a_plain_cargo_test() {
        assert_eq!(
            demonstration_command("`cargo test -p athena-core  demo`").unwrap(),
            ["cargo", "test", "-p", "athena-core", "demo"]
        );
        assert!(demonstration_command("cargo test --locked").is_some());
        for refused in [
            "cargo build",
            "cargo test && rm -rf x",
            "cargo test | cat",
            "cargo test 'a b'",
            "cat README.md",
            "cargo test $(id)",
            "",
        ] {
            assert!(demonstration_command(refused).is_none(), "{refused}");
        }
    }

    #[test]
    fn demonstrations_are_grouped_by_command_and_need_a_fails_without() {
        let groups = demonstration_groups(&[
            demo(1, "cargo test --locked", Some("fails")),
            demo(2, "`cargo test  --locked`", Some("fails")),
            demo(3, "cargo test --locked", None),
            demo(4, "cat README.md", Some("  ")),
            demo(5, "cat README.md", Some("empty")),
        ]);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].criteria, [1, 2]);
        assert_eq!(groups[0].command, "cargo test --locked");
        assert!(groups[0].argv.is_some());
        assert_eq!(groups[1].criteria, [5]);
        assert!(groups[1].argv.is_none());
    }

    #[test]
    fn is_test_path_knows_the_test_file_forms() {
        for yes in [
            "crates/a/tests/x.rs",
            "tests/x.rs",
            "src/run/tests.rs",
            "src/notes_tests.rs",
            "src/notes_test.rs",
            "src/test_notes.rs",
        ] {
            assert!(is_test_path(yes), "{yes}");
        }
        for no in ["src/notes.rs", "src/latest.rs", "src/contest/x.rs"] {
            assert!(!is_test_path(no), "{no}");
        }
    }

    #[test]
    fn declared_module_paths_follow_the_parent_file() {
        assert_eq!(
            declared_module_paths("crates/a/src/lib.rs", "x_tests"),
            ["crates/a/src/x_tests.rs", "crates/a/src/x_tests/mod.rs"]
        );
        assert_eq!(
            declared_module_paths("crates/a/src/gh/mod.rs", "t"),
            ["crates/a/src/gh/t.rs", "crates/a/src/gh/t/mod.rs"]
        );
        assert_eq!(
            declared_module_paths("src/notes.rs", "t"),
            ["src/notes/t.rs", "src/notes/t/mod.rs"]
        );
    }

    const BASE_SRC: &str = "pub fn n() -> u32 {\n    1\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn old() {}\n}\n";
    const FIXED_SRC: &str = "pub fn n() -> u32 {\n    2\n}\n\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn new() {\n        assert_eq!(n(), 2);\n    }\n}\n";

    #[test]
    fn revert_keeping_tests_takes_the_fix_out_and_keeps_the_candidates_tests() {
        let file = revert_keeping_tests(BASE_SRC, FIXED_SRC).unwrap();
        assert!(file.text.contains("    1\n"), "{}", file.text);
        assert!(!file.text.contains("    2\n"));
        assert!(file.text.contains("fn new()"));
        assert!(!file.text.contains("fn old()"), "the base's own tests go");
        assert_eq!(file.text.matches("mod tests").count(), 1);
        assert!(file.declared.is_empty());
        // A base with no test module still gets the candidate's.
        let bare = revert_keeping_tests("pub fn n() -> u32 {\n    1\n}\n", FIXED_SRC).unwrap();
        assert!(bare.text.contains("fn new()") && bare.text.contains("    1\n"));
        // A candidate with no test module leaves none.
        let none = revert_keeping_tests(BASE_SRC, "pub fn n() -> u32 {\n    2\n}\n").unwrap();
        assert!(!none.text.contains("mod tests"));
        // A file that does not parse is not handled here.
        assert!(revert_keeping_tests(BASE_SRC, "fn (").is_none());
        assert!(revert_keeping_tests("fn (", FIXED_SRC).is_none());
    }

    #[test]
    fn revert_keeping_tests_carries_a_bodiless_test_declaration_over() {
        let base = "pub fn n() -> u32 {\n    1\n}\n";
        let candidate = "pub fn n() -> u32 {\n    2\n}\n#[cfg(test)]\nmod n_tests;\n";
        let file = revert_keeping_tests(base, candidate).unwrap();
        assert!(
            file.text.contains("#[cfg(test)]\nmod n_tests;"),
            "{}",
            file.text
        );
        assert!(file.text.contains("    1\n"));
        assert_eq!(file.declared, ["n_tests"]);
    }

    fn bytes_of<'a>(files: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<Vec<u8>> + 'a {
        move |path| {
            files
                .iter()
                .find(|(p, _)| *p == path)
                .map(|(_, t)| t.as_bytes().to_vec())
        }
    }

    #[test]
    fn revert_plan_reverts_the_fix_and_keeps_tests_manifests_and_declared_modules() {
        let changed: Vec<(char, String)> = [
            ('M', "Cargo.toml"),
            ('M', "README.md"),
            ('M', "src/lib.rs"),
            ('M', "src/notes.rs"),
            ('A', "src/helper.rs"),
            ('A', "src/inline.rs"),
            ('A', "src/n_tests.rs"),
            ('A', "src/odd_name.rs"),
            ('A', "tests/it.rs"),
            ('D', "src/gone.rs"),
        ]
        .into_iter()
        .map(|(s, p)| (s, p.to_string()))
        .collect();
        let base = [
            ("README.md", "old\n"),
            ("src/lib.rs", "mod notes;\n"),
            ("src/notes.rs", BASE_SRC),
            ("src/gone.rs", "gone\n"),
        ];
        let candidate = [
            ("README.md", "new\n"),
            (
                "src/lib.rs",
                "mod notes;\nmod helper;\n#[cfg(test)]\nmod odd_name;\n#[cfg(test)]\nmod n_tests;\n",
            ),
            ("src/notes.rs", FIXED_SRC),
            ("src/helper.rs", "pub fn h() {}\n"),
            ("src/inline.rs", FIXED_SRC),
            ("src/n_tests.rs", "#[test]\nfn t() {}\n"),
            ("src/odd_name.rs", "#[test]\nfn o() {}\n"),
            ("tests/it.rs", "#[test]\nfn i() {}\n"),
        ];
        let plan = revert_plan(&changed, &bytes_of(&base), &bytes_of(&candidate));
        assert_eq!(
            plan.reverted,
            [
                "README.md",
                "src/lib.rs",
                "src/notes.rs",
                "src/helper.rs",
                "src/inline.rs",
                "src/gone.rs"
            ]
        );
        assert_eq!(plan.remove, ["src/helper.rs", "src/inline.rs"]);
        assert_eq!(plan.tests_removed, ["src/inline.rs"]);
        let written = |p: &str| {
            plan.write
                .iter()
                .find(|(path, _)| path == p)
                .map(|(_, b)| String::from_utf8(b.clone()).unwrap())
        };
        assert_eq!(written("README.md").as_deref(), Some("old\n"));
        assert_eq!(written("src/gone.rs").as_deref(), Some("gone\n"));
        let lib = written("src/lib.rs").unwrap();
        assert!(lib.contains("mod odd_name;") && lib.contains("mod n_tests;"));
        assert!(
            !lib.contains("mod helper;"),
            "the fix's declaration is reverted"
        );
        let notes = written("src/notes.rs").unwrap();
        assert!(notes.contains("    1\n") && notes.contains("fn new()"));
        for kept in [
            "Cargo.toml",
            "src/n_tests.rs",
            "src/odd_name.rs",
            "tests/it.rs",
        ] {
            assert!(written(kept).is_none() && !plan.remove.iter().any(|p| p == kept));
        }
    }

    #[test]
    fn revert_plan_reverts_a_file_that_does_not_parse_whole_and_names_it() {
        let changed = vec![('M', "src/broken.rs".to_string())];
        let plan = revert_plan(
            &changed,
            &bytes_of(&[("src/broken.rs", "old\n")]),
            &bytes_of(&[("src/broken.rs", "fn (\n#[cfg(test)]\n")]),
        );
        assert_eq!(plan.tests_removed, ["src/broken.rs"]);
        assert_eq!(
            plan.write,
            [("src/broken.rs".to_string(), b"old\n".to_vec())]
        );
    }

    fn demo_group() -> DemonstrationGroup {
        DemonstrationGroup {
            criteria: vec![2, 3],
            command: "cargo test --locked demo".into(),
            argv: demonstration_command("cargo test --locked demo"),
        }
    }

    fn demo_result(run: &CheckRun) -> String {
        demonstration_view(&demo_group(), run, &RevertPlan::default())
            .demonstration
            .unwrap()
            .result
    }

    #[test]
    fn demonstration_view_classifies_by_exit_code_and_tests_ran() {
        let run = |exit_code, tests_ran, tail: &str| CheckRun {
            exit_code,
            tests_ran,
            output_tail: tail.into(),
            ..check(false)
        };
        let failed = demo_result(&run(
            Some(101),
            Some(1),
            "test result: FAILED. 0 passed; 1 failed",
        ));
        assert_eq!(
            failed,
            "the named test command failed without the fix (exit 101)"
        );
        assert!(
            demo_result(&run(Some(101), Some(0), "error[E0425]: cannot find"))
                .ends_with(": the test did not compile")
        );
        assert!(
            demo_result(&run(Some(101), Some(0), "error: could not compile `x`"))
                .contains("did not compile")
        );
        // Passing: the tail holds only trailing `0 passed` summaries, but the full output counted a test.
        let passed = demo_result(&run(
            Some(0),
            Some(1),
            "test result: ok. 0 passed; 0 failed",
        ));
        assert_eq!(
            passed,
            "passed without the fix: the revert did not make the test fail"
        );
        assert!(demo_result(&run(Some(0), Some(0), "")).contains("ran no tests"));
        assert!(demo_result(&run(None, None, "")).starts_with("the run did not finish"));
    }

    #[test]
    fn demonstration_view_is_not_a_check_and_names_the_run() {
        let plan = RevertPlan {
            reverted: vec!["src/a.rs".into()],
            tests_removed: vec!["src/b.rs".into()],
            ..Default::default()
        };
        let run = CheckRun {
            output_tail: "tail".into(),
            ..check(false)
        };
        let view = demonstration_view(&demo_group(), &run, &plan);
        assert_eq!(view.name, "fails-without-2");
        assert_eq!(view.command, "not a check");
        assert!(view.passed);
        let record = view.demonstration.unwrap();
        assert_eq!(record.criteria, [2, 3]);
        assert_eq!(record.exit_code, Some(101));
        assert_eq!(record.reverted, ["src/a.rs"]);
        assert_eq!(record.tests_removed, ["src/b.rs"]);
        assert_eq!(record.output_tail, "tail");
        let not_run = demonstration_not_run(&demo_group(), "no scratch copy")
            .demonstration
            .unwrap();
        assert_eq!(not_run.result, "not run: no scratch copy");
        assert!(!not_run.ran && record.ran);
        assert_eq!(not_run.exit_code, None);
    }

    fn finding_at(severity: Severity, location: Option<&str>, summary: &str) -> Finding {
        Finding {
            severity,
            summary: summary.into(),
            evidence: String::new(),
            location: location.map(String::from),
            source: FindingSource::Verifier,
        }
    }

    #[test]
    fn finding_is_content_only_when_every_cited_path_is_non_code() {
        let content =
            |loc: &str| finding_is_content_only(&finding_at(Severity::Major, Some(loc), "s"));
        for loc in [
            "docs/decisions.md:3005-3053",
            "docs/x.md:3-9",
            "docs/design/diagram.png",
            "notes.md",
            "README.markdown:2",
            "CHANGELOG.md",
            "docs/a.md, CHANGELOG.md:4",
            "docs/a.md#heading",
        ] {
            assert!(content(loc), "{loc} is content-only");
        }
        for loc in [
            "src/lib.rs",
            "crates/athena-core/src/run.rs:10",
            "Cargo.toml",
            "athena.toml",
            ".github/workflows/ci.yml",
            "docs/a.md src/b.rs",
            "docs/a.md;Cargo.toml:3",
            "",
        ] {
            assert!(!content(loc), "{loc:?} is not content-only");
        }
        // No location: paths are read from the prose, and no path means not content-only.
        let prose = |s: &str| finding_is_content_only(&finding_at(Severity::Major, None, s));
        assert!(prose("D122 sits inside D121 in docs/decisions.md"));
        assert!(!prose("the entry is out of order"));
        assert!(!prose("see docs/a.md and fix src/lib.rs"));
        assert!(!prose("version 1.2 is wrong"));
        // A location wins over prose that names code.
        let located = finding_at(Severity::Major, Some("docs/a.md"), "mentions src/lib.rs");
        assert!(finding_is_content_only(&located));
    }

    #[test]
    fn only_blocker_and_major_findings_block() {
        let blocks = |s| finding_blocks(&finding_at(s, Some("src/lib.rs"), "s"));
        assert!(blocks(Severity::Blocker));
        assert!(blocks(Severity::Major));
        assert!(!blocks(Severity::Minor));
        assert!(!blocks(Severity::Nit));
    }

    #[test]
    fn the_reuse_statement_names_the_run_and_exists_only_for_the_same_sha() {
        use crate::ci_result::{CiRecord, Source, Step};
        let sha = "0123456789abcdef0123456789abcdef01234567";
        let record = CiRecord {
            source: Source::Ci,
            sha: sha.into(),
            run_address: Some("https://example.test/runs/5".into()),
            steps: vec![Step {
                name: "fmt".into(),
                conclusion: "success".into(),
                duration_s: 1.0,
            }],
            duration_s: 12.0,
        };
        let text = reuse_statement(&record, sha).unwrap();
        assert!(text.contains("runs/5") && text.contains(sha) && text.contains("fmt success"));
        assert_eq!(
            reuse_statement(&record, "fedcba9876543210fedcba9876543210fedcba98"),
            None
        );
    }

    fn criteria() -> Vec<Criterion> {
        vec![
            Criterion {
                text: "CHANGELOG.md exists".into(),
                kind: CriterionKind::Content,
            },
            Criterion {
                text: "cargo test passes".into(),
                kind: CriterionKind::Behavior,
            },
        ]
    }

    fn check(passed: bool) -> CheckRun {
        CheckRun {
            name: "test".into(),
            command: vec!["cargo".into(), "test".into()],
            exit_code: Some(if passed { 0 } else { 101 }),
            passed,
            tests_ran: Some(3),
            duration: Quantity::measured(1.0, Unit::Seconds, "clock"),
            output_tail: String::new(),
            note: None,
            failed_tests: Vec::new(),
            reused: false,
        }
    }

    /// A `test` check whose result was reused from a green CI run: no exit code, marked `reused`.
    fn reused_from_ci(passed: bool) -> CheckRun {
        CheckRun {
            exit_code: None,
            passed,
            reused: true,
            note: Some("result reused from the CI run for abc".into()),
            ..check(true)
        }
    }

    /// A failed `test` check that names its failing tests.
    fn failing(tests: &[&str]) -> CheckRun {
        CheckRun {
            failed_tests: tests.iter().map(|t| t.to_string()).collect(),
            ..check(false)
        }
    }

    /// A failed `test` check reused from a red CI run, with the names and tail its log gave.
    fn reused_red(tests: &[&str]) -> CheckRun {
        CheckRun {
            failed_tests: tests.iter().map(|t| t.to_string()).collect(),
            output_tail: "failing units: athena-core --lib\nlast line\n".into(),
            ..reused_from_ci(false)
        }
    }

    #[test]
    fn check_result_view_shows_names_units_and_tail_only_for_a_reused_failed_check() {
        let red = check_result_view(&reused_red(&["athena_core tests::a"]));
        assert_eq!(red.failed_tests, vec!["athena_core tests::a"]);
        assert_eq!(red.failing_units, "athena-core --lib");
        assert_eq!(red.output_tail, "last line\n");
        assert!(!red.passed);
        // No header: the whole tail is the tail.
        let plain = check_result_view(&CheckRun {
            output_tail: "just text".into(),
            ..reused_red(&[])
        });
        assert_eq!(
            (plain.failing_units.as_str(), plain.output_tail.as_str()),
            ("", "just text")
        );
        // A check Athena ran itself, failed, with names and a tail: the view is as it always was.
        let local = CheckRun {
            output_tail: "failing units: x\nboom".into(),
            ..failing(&["athena_core tests::a"])
        };
        let view = check_result_view(&local);
        assert!(view.failed_tests.is_empty() && view.output_tail.is_empty());
        assert!(view.failing_units.is_empty());
        assert_eq!(view.detail, "3 tests ran");
        // A reused check that passed shows none either.
        let green = CheckRun {
            failed_tests: vec!["x".into()],
            ..reused_from_ci(true)
        };
        assert!(check_result_view(&green).failed_tests.is_empty());
    }

    #[test]
    fn reused_red_with_names_is_compared_test_by_test() {
        let base = baseline_with(
            vec![failing(&["athena_core tests::a", "athena_core tests::b"])],
            None,
        );
        // Every name also failed on the base: pre-existing.
        assert!(is_pre_existing(
            &reused_red(&["athena_core tests::a"]),
            Some(&base)
        ));
        // One name the base did not fail: the job's own failure.
        assert!(!is_pre_existing(
            &reused_red(&["athena_core tests::a", "athena_core tests::new"]),
            Some(&base)
        ));
        // No names: the check-name rule stands, as before.
        assert!(is_pre_existing(&reused_red(&[]), Some(&base)));
    }

    #[test]
    fn reused_red_incomplete_list_is_not_pre_existing() {
        let base = baseline_with(vec![failing(&["athena_core tests::a"])], None);
        let complete = reused_red(&["athena_core tests::a"]);
        assert!(is_pre_existing(&complete, Some(&base)));
        for marker in [
            "test (shard 2) (log not read)",
            "test (shard 2) (no failing tests listed)",
            "test (shard 2) (did not finish)",
            "(list cut at 50)",
        ] {
            let incomplete = reused_red(&["athena_core tests::a", marker]);
            assert!(
                !is_pre_existing(&incomplete, Some(&base)),
                "{marker} must not be excused by the base"
            );
            let (new, old) = failure_split(&incomplete, Some(&base));
            assert_eq!(
                (new, old),
                (
                    vec![marker.to_string()],
                    vec!["athena_core tests::a".to_string()]
                )
            );
        }
    }

    fn output(commands: Vec<&str>, findings: Vec<FindingOutput>) -> VerdictOutput {
        VerdictOutput {
            verdict: VerdictValue::Pass,
            criteria: vec![
                CriterionVerdictOutput {
                    criterion: "CHANGELOG.md exists".into(),
                    met: true,
                    evidence: "read it".into(),
                    commands: vec![],
                },
                CriterionVerdictOutput {
                    criterion: "cargo test passes".into(),
                    met: true,
                    evidence: "ran it".into(),
                    commands: commands.into_iter().map(String::from).collect(),
                },
            ],
            findings,
            request_reading_mismatch: None,
            summary: "ok".into(),
        }
    }

    fn judge_with(out: &VerdictOutput, ran: &[String], checks: &[CheckRun]) -> Judgment {
        judge(&VerdictInputs {
            criteria: &criteria(),
            checks,
            output: Some(out),
            commands_run: ran,
            candidate_tree: None,
            checks_tree: None,
            system_fault: None,
            verifier_failure: None,
            map_base: None,
            map_candidate: None,
            baseline: None,
            replay: None,
        })
    }

    fn baseline_with(checks: Vec<CheckRun>, note: Option<&str>) -> Baseline {
        Baseline {
            job: athena_data::JobId::parse("20261018-073510-m3yh").unwrap(),
            base_commit: "a".repeat(40),
            before_invocation: 1,
            checks,
            note: note.map(String::from),
        }
    }

    fn judge_against(baseline: Option<&Baseline>, checks: &[CheckRun]) -> Judgment {
        let ran = ["cargo test".to_string()];
        judge(&VerdictInputs {
            criteria: &criteria(),
            checks,
            output: Some(&output(vec!["cargo test"], vec![])),
            commands_run: &ran,
            candidate_tree: None,
            checks_tree: None,
            system_fault: None,
            verifier_failure: None,
            map_base: None,
            map_candidate: None,
            baseline,
            replay: None,
        })
    }

    /// The verifier ran nothing; `recorded` is Athena's check list, run on `checks_tree`, judged for `candidate_tree`.
    fn judge_recorded(
        recorded: &[CheckRun],
        candidate_tree: Option<&str>,
        checks_tree: Option<&str>,
        cited: Vec<&str>,
    ) -> Judgment {
        judge(&VerdictInputs {
            criteria: &criteria(),
            checks: recorded,
            output: Some(&output(cited, vec![])),
            commands_run: &[],
            candidate_tree,
            checks_tree,
            system_fault: None,
            verifier_failure: None,
            map_base: None,
            map_candidate: None,
            baseline: None,
            replay: None,
        })
    }

    fn judge_replay(replay: Option<&Replay>) -> Judgment {
        let ran = ["cargo test".to_string()];
        judge(&VerdictInputs {
            criteria: &criteria(),
            checks: &[check(true)],
            output: Some(&output(vec!["cargo test"], vec![])),
            commands_run: &ran,
            candidate_tree: None,
            checks_tree: None,
            system_fault: None,
            verifier_failure: None,
            map_base: None,
            map_candidate: None,
            baseline: None,
            replay,
        })
    }

    fn judge_failure(
        failure: Option<VerifierFailure>,
        out: Option<&VerdictOutput>,
        checks: &[CheckRun],
    ) -> Judgment {
        let ran = ["cargo test".to_string()];
        judge(&VerdictInputs {
            criteria: &criteria(),
            checks,
            output: out,
            commands_run: &ran,
            candidate_tree: None,
            checks_tree: None,
            system_fault: None,
            verifier_failure: failure,
            map_base: None,
            map_candidate: None,
            baseline: None,
            replay: None,
        })
    }

    #[test]
    fn a_verifier_that_times_out_is_unjudged_never_a_pass() {
        let detail = "deadline of 1s reached";
        // No verdict: every criterion is unjudged, and the reason is the unjudged one, not a judged failure.
        let none = judge_failure(
            Some(VerifierFailure::TimedOut(detail.into())),
            None,
            &[check(true)],
        );
        assert_eq!(none.final_verdict, Verdict::Fail);
        assert_eq!(none.agent_verdict, None);
        assert_eq!(
            none.reasons,
            vec![format!("{UNJUDGED_REASON_PREFIX}: {detail}")]
        );
        assert!(none.reasons[0].contains("unjudged"));
        assert!(!none.reasons.iter().any(|r| r.contains("did not complete")));
        assert!(
            !none
                .reasons
                .iter()
                .any(|r| r.contains("returned no verdict"))
        );
        assert_eq!(none.criteria.len(), 2);
        for c in &none.criteria {
            assert!(!c.met);
            assert!(c.evidence.contains("unjudged"), "{}", c.evidence);
            assert!(!c.evidence.contains("did not assess"), "{}", c.evidence);
        }
        // Even with a passing verdict in hand, a timeout never yields a pass.
        let out = output(vec!["cargo test"], vec![]);
        let with_pass = judge_failure(
            Some(VerifierFailure::TimedOut(detail.into())),
            Some(&out),
            &[check(true)],
        );
        assert_eq!(with_pass.final_verdict, Verdict::Fail);
        assert!(with_pass.reasons[0].contains("unjudged"));
        // Without the timeout the same inputs pass.
        let ok = judge_failure(None, Some(&out), &[check(true)]);
        assert_eq!(ok.final_verdict, Verdict::Pass);
    }

    #[test]
    fn non_timeout_incomplete_failure_keeps_the_did_not_complete_reason() {
        let j = judge_failure(
            Some(VerifierFailure::Incomplete("x".into())),
            None,
            &[check(true)],
        );
        assert_eq!(j.final_verdict, Verdict::Fail);
        assert_eq!(j.reasons, vec!["the verifier did not complete: x"]);
        for c in &j.criteria {
            assert_eq!(c.evidence, "the verifier did not assess this criterion");
            assert!(!c.evidence.contains("unjudged"));
        }
    }

    #[test]
    fn non_timeout_failing_check_and_judged_fail_keep_their_reasons() {
        // A failing check fails an otherwise passing verdict, with its existing reason.
        let out = output(vec!["cargo test"], vec![]);
        let failing_check = judge_failure(None, Some(&out), &[check(false)]);
        assert_eq!(failing_check.final_verdict, Verdict::Fail);
        assert!(
            failing_check.reasons.iter().any(|r| r.contains("failed")),
            "{:?}",
            failing_check.reasons
        );
        assert!(!failing_check.reasons.iter().any(|r| r.contains("unjudged")));
        // A judged fail verdict keeps its reason.
        let mut failed = output(vec!["cargo test"], vec![]);
        failed.verdict = VerdictValue::Fail;
        let judged = judge_failure(None, Some(&failed), &[check(true)]);
        assert_eq!(judged.final_verdict, Verdict::Fail);
        assert_eq!(judged.agent_verdict, Some(Verdict::Fail));
        assert_eq!(judged.reasons, vec!["the verifier's verdict is fail"]);
    }

    #[test]
    fn replay_hold_is_a_blocker_that_names_the_list_and_fails_an_otherwise_passing_verdict() {
        use athena_data::records::ReplayRefusal;
        let id = |s: &str| athena_data::JobId::parse(s).unwrap();
        let mut replay = Replay {
            job: id("20261018-073510-m3yh"),
            base_commit: "a".repeat(40),
            candidate_commit: "b".repeat(40),
            window: 50,
            total: 10,
            newly_refused: vec![
                ReplayRefusal {
                    job: id("20261001-000000-aaaa"),
                    reason: "criterion 1 cites the producer's report".into(),
                },
                ReplayRefusal {
                    job: id("20261001-000001-bbbb"),
                    reason: "no governance file".into(),
                },
            ],
            held: true,
            intended: false,
            no_replay: None,
        };
        assert_eq!(judge_replay(None).final_verdict, Verdict::Pass);
        let held = judge_replay(Some(&replay));
        assert_eq!(held.final_verdict, Verdict::Fail);
        let blocker = held
            .findings
            .iter()
            .find(|f| f.severity == Severity::Blocker)
            .expect("a blocker");
        assert_eq!(blocker.source, FindingSource::Athena);
        let text = format!("{} {}", blocker.summary, blocker.evidence);
        assert!(text.contains("2 of 10"), "{text}");
        for needle in [
            "20261001-000000-aaaa",
            "20261001-000001-bbbb",
            "criterion 1 cites the producer's report",
        ] {
            assert!(text.contains(needle), "{needle}: {text}");
        }
        assert!(
            held.reasons
                .iter()
                .any(|r| r.contains("newly refuses 2 of 10")),
            "{:?}",
            held.reasons
        );
        // The same list, not held, is only a record.
        replay.held = false;
        assert_eq!(judge_replay(Some(&replay)).final_verdict, Verdict::Pass);
    }

    fn d43_finding(j: &Judgment) -> Option<&Finding> {
        j.findings.iter().find(|f| f.summary.contains("D43"))
    }

    #[test]
    fn a_passing_check_recorded_on_the_same_tree_counts_as_run() {
        let j = judge_recorded(&[check(true)], Some("abc"), Some("abc"), vec!["cargo test"]);
        assert!(j.criteria[1].met);
        assert!(j.criteria[1].commands_verified);
        assert!(d43_finding(&j).is_none());
    }

    #[test]
    fn a_failed_recorded_check_does_not_count_as_run() {
        let j = judge_recorded(
            &[check(false)],
            Some("abc"),
            Some("abc"),
            vec!["cargo test"],
        );
        assert!(!j.criteria[1].met);
        assert!(!j.criteria[1].commands_verified);
        let finding = d43_finding(&j).expect("D43 finding");
        assert!(
            finding
                .evidence
                .contains("cited but not seen running: `cargo test`")
        );
    }

    #[test]
    fn a_passing_check_recorded_on_another_tree_does_not_count_as_run() {
        for checks_tree in [Some("def"), None] {
            let j = judge_recorded(&[check(true)], Some("abc"), checks_tree, vec!["cargo test"]);
            assert!(!j.criteria[1].met, "{checks_tree:?}");
            assert!(!j.criteria[1].commands_verified);
            assert!(d43_finding(&j).is_some());
        }
        // No candidate tree known: nothing can match.
        let j = judge_recorded(&[check(true)], None, None, vec!["cargo test"]);
        assert!(!j.criteria[1].met);
    }

    #[test]
    fn a_placeholder_record_without_an_exit_code_does_not_count_as_run() {
        let mut placeholder = check(true);
        placeholder.exit_code = None;
        let j = judge_recorded(&[placeholder], Some("abc"), Some("abc"), vec!["cargo test"]);
        assert!(!j.criteria[1].met);
        assert!(d43_finding(&j).is_some());
    }

    #[test]
    fn a_passing_check_reused_from_ci_on_the_same_commit_counts_as_run() {
        let j = judge_recorded(
            &[reused_from_ci(true)],
            Some("abc"),
            Some("abc"),
            vec!["cargo test"],
        );
        assert!(j.criteria[1].met);
        assert!(j.criteria[1].commands_verified);
        assert!(d43_finding(&j).is_none());
        // fmt and clippy reused from CI count the same way, by their command text.
        let named = |name: &str, command: &[&str]| CheckRun {
            name: name.into(),
            command: command.iter().map(|s| s.to_string()).collect(),
            ..reused_from_ci(true)
        };
        let recorded = [
            named("fmt", &["cargo", "fmt", "--all", "--check"]),
            named("clippy", &["cargo", "clippy", "--all-targets"]),
            named("test", &["cargo", "test", "--locked"]),
        ];
        for cited in ["cargo fmt --all --check", "cargo clippy --all-targets"] {
            let j = judge_recorded(&recorded, Some("abc"), Some("abc"), vec![cited]);
            assert!(j.criteria[1].commands_verified, "{cited}");
            assert!(d43_finding(&j).is_none(), "{cited}");
        }
    }

    #[test]
    fn a_check_reused_from_ci_for_another_commit_does_not_count_as_run() {
        for checks_tree in [Some("def"), None] {
            let j = judge_recorded(
                &[reused_from_ci(true)],
                Some("abc"),
                checks_tree,
                vec!["cargo test"],
            );
            assert!(!j.criteria[1].met, "{checks_tree:?}");
            assert!(!j.criteria[1].commands_verified);
            assert!(d43_finding(&j).is_some());
        }
    }

    #[test]
    fn a_failed_check_reused_from_ci_does_not_count_as_run() {
        let j = judge_recorded(
            &[reused_from_ci(false)],
            Some("abc"),
            Some("abc"),
            vec!["cargo test"],
        );
        assert!(!j.criteria[1].met);
        assert!(!j.criteria[1].commands_verified);
        assert!(d43_finding(&j).is_some());
    }

    #[test]
    fn a_skipped_check_that_was_not_reused_does_not_count_as_run() {
        let skipped = CheckRun {
            exit_code: None,
            note: Some("not run: CI was not green for abc".into()),
            ..check(true)
        };
        assert!(!skipped.reused);
        let j = judge_recorded(&[skipped], Some("abc"), Some("abc"), vec!["cargo test"]);
        assert!(!j.criteria[1].met);
        assert!(!j.criteria[1].commands_verified);
        assert!(d43_finding(&j).is_some());
    }

    #[test]
    fn a_command_neither_run_nor_recorded_is_unseen_and_a_recorded_one_needs_the_same_text() {
        let j = judge_recorded(&[], Some("abc"), Some("abc"), vec!["cargo test"]);
        assert!(!j.criteria[1].met);
        let j = judge_recorded(
            &[check(true)],
            Some("abc"),
            Some("abc"),
            vec!["cargo clippy"],
        );
        assert!(!j.criteria[1].met);
        let j = judge_recorded(&[check(true)], Some("abc"), Some("abc"), vec![]);
        assert!(!j.criteria[1].met);
        assert!(
            d43_finding(&j)
                .unwrap()
                .evidence
                .contains("it cites no command")
        );
    }

    #[test]
    fn a_check_failing_on_the_base_and_again_is_pre_existing_and_does_not_fail_the_verdict() {
        let base = baseline_with(vec![check(false)], None);
        let mut now = vec![check(false)];
        mark_pre_existing(&mut now, Some(&base));
        assert!(
            now[0]
                .note
                .as_deref()
                .unwrap()
                .starts_with(PRE_EXISTING_NOTE)
        );
        let j = judge_against(Some(&base), &now);
        assert_eq!(j.final_verdict, Verdict::Pass, "{:?}", j.reasons);
        assert!(j.reasons.is_empty(), "{:?}", j.reasons);
        // Marking twice changes nothing.
        let once = now.clone();
        mark_pre_existing(&mut now, Some(&base));
        assert_eq!(now, once);
        // judge alone, with no mark applied, reaches the same verdict.
        let j = judge_against(Some(&base), &[check(false)]);
        assert_eq!(j.final_verdict, Verdict::Pass, "{:?}", j.reasons);
    }

    #[test]
    fn a_new_failing_test_beside_one_failing_on_the_base_is_not_pre_existing() {
        let base = baseline_with(vec![failing(&["lib X"])], None);
        // Base {X}, now {X, Y}: the check is the job's, and the verdict names Y.
        let mut now = vec![failing(&["lib X", "lib Y"])];
        assert!(!is_pre_existing(&now[0], Some(&base)));
        mark_pre_existing(&mut now, Some(&base));
        assert_eq!(now[0].note, None, "not marked pre-existing");
        let j = judge_against(Some(&base), &now);
        assert_eq!(j.final_verdict, Verdict::Fail);
        assert!(j.reasons.iter().any(|r| r == "check test failed"));
        let finding = j.findings.iter().find(|f| f.summary.contains("Check test"));
        let text = finding.map(|f| f.evidence.clone()).unwrap_or_default();
        assert!(text.contains("newly failing: lib Y"), "{text}");
        // Base {X}, now {X}: still pre-existing, and the verdict passes.
        let mut same = vec![failing(&["lib X"])];
        assert!(is_pre_existing(&same[0], Some(&base)));
        mark_pre_existing(&mut same, Some(&base));
        assert!(
            same[0]
                .note
                .as_deref()
                .unwrap()
                .starts_with(PRE_EXISTING_NOTE)
        );
        assert_eq!(
            judge_against(Some(&base), &same).final_verdict,
            Verdict::Pass
        );
        // Fewer failures than the base is still pre-existing.
        let fewer = baseline_with(vec![failing(&["lib X", "lib Z"])], None);
        assert!(is_pre_existing(&failing(&["lib X"]), Some(&fewer)));
        // A different single failure is new.
        assert!(!is_pre_existing(&failing(&["lib Y"]), Some(&base)));
    }

    #[test]
    fn the_same_test_name_in_another_binary_is_a_new_failure() {
        let base = baseline_with(vec![failing(&["a X"])], None);
        assert!(is_pre_existing(&failing(&["a X"]), Some(&base)));
        assert!(!is_pre_existing(&failing(&["a X", "b X"]), Some(&base)));
    }

    #[test]
    fn a_crashed_binary_is_new_unless_the_base_crashed_the_same_way() {
        let base = baseline_with(vec![failing(&["a X"])], None);
        let crashed = failing(&["a X", "b (did not finish)"]);
        assert!(!is_pre_existing(&crashed, Some(&base)));
        assert_eq!(
            judge_against(Some(&base), std::slice::from_ref(&crashed)).final_verdict,
            Verdict::Fail
        );
        let base_crashed = baseline_with(vec![failing(&["a X", "b (did not finish)"])], None);
        assert!(is_pre_existing(&crashed, Some(&base_crashed)));
    }

    #[test]
    fn with_no_names_on_either_side_the_check_name_rule_stands() {
        let named = baseline_with(vec![failing(&["lib X"])], None);
        let unnamed = baseline_with(vec![check(false)], None);
        // The base named tests, the candidate none (a build error, a timeout): excused as before.
        assert!(is_pre_existing(&check(false), Some(&named)));
        // The candidate named tests, the base none: excused as before.
        assert!(is_pre_existing(&failing(&["lib Y"]), Some(&unnamed)));
        assert!(is_pre_existing(&check(false), Some(&unnamed)));
        // A check that passed is never pre-existing, with or without names.
        assert!(!is_pre_existing(&check(true), Some(&named)));
    }

    #[test]
    fn the_finding_for_a_failed_check_separates_new_from_base_failures() {
        let base = baseline_with(vec![failing(&["lib X"])], None);
        let j = judge_against(Some(&base), &[failing(&["lib X", "lib Y"])]);
        assert_eq!(
            j.reasons
                .iter()
                .filter(|r| *r == "check test failed")
                .count(),
            1
        );
        let found: Vec<_> = j
            .findings
            .iter()
            .filter(|f| f.summary.starts_with("Check test:"))
            .collect();
        assert_eq!(found.len(), 1, "one finding, not a second blocker");
        assert_eq!(found[0].severity, Severity::Minor);
        assert_eq!(found[0].source, FindingSource::Athena);
        assert!(
            found[0]
                .summary
                .contains("1 test(s) newly failing, 1 already failing")
        );
        assert_eq!(
            found[0].evidence,
            "newly failing: lib Y; already failing on the base: lib X"
        );
        assert_eq!(j.reasons.len(), 1, "{:?}", j.reasons);
        // Only new names: no already part.
        let j = judge_against(None, &[failing(&["lib Y"])]);
        let f = j
            .findings
            .iter()
            .find(|f| f.summary.starts_with("Check test:"));
        assert_eq!(f.unwrap().evidence, "newly failing: lib Y");
        // No names: no finding. Fully pre-existing: none either.
        assert!(judge_against(None, &[check(false)]).findings.is_empty());
        assert!(
            judge_against(Some(&base), &[failing(&["lib X"])])
                .findings
                .is_empty()
        );
    }

    #[test]
    fn a_check_that_passed_or_was_absent_at_the_base_still_fails_the_verdict() {
        let mut other = check(false);
        other.name = "fmt".into();
        let passed_before = baseline_with(vec![check(true)], None);
        let other_only = baseline_with(vec![other], None);
        for base in [Some(&passed_before), Some(&other_only), None] {
            let mut now = vec![check(false)];
            mark_pre_existing(&mut now, base);
            assert_eq!(now[0].note, None, "not marked");
            let j = judge_against(base, &now);
            assert_eq!(j.final_verdict, Verdict::Fail);
            assert!(
                j.reasons.iter().any(|r| r == "check test failed"),
                "{:?}",
                j.reasons
            );
        }
    }

    #[test]
    fn an_empty_or_noted_baseline_excuses_nothing() {
        let empty = baseline_with(vec![], None);
        let noted = baseline_with(
            vec![check(false)],
            Some("the baseline could not run: no snapshot"),
        );
        for base in [&empty, &noted] {
            let mut now = vec![check(false)];
            mark_pre_existing(&mut now, Some(base));
            assert_eq!(now[0].note, None);
            let j = judge_against(Some(base), &now);
            assert_eq!(j.final_verdict, Verdict::Fail);
            assert!(j.reasons.iter().any(|r| r == "check test failed"));
        }
    }

    #[test]
    fn a_failed_check_does_not_hide_the_criteria_the_verifier_judged() {
        let j = judge_against(None, &[check(false)]);
        assert_eq!(j.final_verdict, Verdict::Fail);
        assert!(j.criteria.iter().all(|c| c.met));
        assert!(!j.reasons.iter().any(|r| r.contains("returned no verdict")));
    }

    fn map(a: &str, b: &str) -> String {
        format!(
            "| ID | Requirement | Status |\n|---|---|---|\n| A1-1 | x | `f` | {a} |\n| A2-1 | y | `g` | {b} |\n"
        )
    }

    fn judged_with_maps(base: &str, candidate: &str) -> Judgment {
        let ran = ["cargo test".to_string()];
        judge(&VerdictInputs {
            criteria: &criteria(),
            checks: &[check(true)],
            output: Some(&output(vec!["cargo test"], vec![])),
            commands_run: &ran,
            candidate_tree: None,
            checks_tree: None,
            system_fault: None,
            verifier_failure: None,
            map_base: Some(base),
            map_candidate: Some(candidate),
            baseline: None,
            replay: None,
        })
    }

    #[test]
    fn a_row_moved_to_covered_is_a_major_finding_and_the_verdict_fails() {
        for from in ["Partial: later", "Not covered"] {
            let j = judged_with_maps(&map(from, "Covered"), &map("Covered", "Covered"));
            assert_eq!(j.final_verdict, Verdict::Fail, "{from}");
            let named: Vec<_> = j
                .findings
                .iter()
                .filter(|f| f.source == FindingSource::Athena && f.severity == Severity::Major)
                .collect();
            assert_eq!(named.len(), 1, "{from}");
            assert!(named[0].summary.contains("A1-1"), "{from}");
        }
    }

    #[test]
    fn a_row_added_already_covered_is_a_major_finding() {
        let base = "| A1-1 | x | `f` | Covered |\n";
        let candidate = "| A1-1 | x | `f` | Covered |\n| A3-1 | z | `h` | Covered |\n";
        let j = judged_with_maps(base, candidate);
        assert_eq!(j.final_verdict, Verdict::Fail);
        assert_eq!(j.findings.len(), 1);
        assert!(j.findings[0].summary.contains("A3-1"));
    }

    #[test]
    fn a_row_left_covered_or_moved_away_from_covered_adds_no_finding() {
        let same = judged_with_maps(&map("Covered", "Partial"), &map("Covered", "Partial"));
        assert_eq!(same.final_verdict, Verdict::Pass);
        assert!(same.findings.is_empty());
        let away = judged_with_maps(
            &map("Covered", "Covered"),
            &map("Partial: reason", "Not covered"),
        );
        assert_eq!(away.final_verdict, Verdict::Pass);
        assert!(away.findings.is_empty());
        assert!(away.reasons.is_empty());
    }

    #[test]
    fn a_missing_map_adds_no_finding() {
        assert!(rows_newly_covered(None, None).is_empty());
        assert!(rows_newly_covered(Some("| A1-1 | x | Covered |"), None).is_empty());
    }

    #[test]
    fn a_behavior_criterion_citing_a_command_that_ran_passes() {
        // M16 (allows): a criterion citing a command that ran.
        let j = judge_with(
            &output(vec!["cargo test"], vec![]),
            &["cargo test".to_string()],
            &[check(true)],
        );
        assert_eq!(j.final_verdict, Verdict::Pass);
        assert!(verified(&j, "b", "c", "sha256:x").is_some());
    }

    #[test]
    fn a_criterion_citing_a_command_that_never_ran_is_unmet() {
        // M16 (blocks): a criterion citing a command that never ran counts as unmet.
        let j = judge_with(
            &output(vec!["cargo test --all"], vec![]),
            &["cargo build".to_string()],
            &[check(true)],
        );
        assert_eq!(j.final_verdict, Verdict::Fail);
        assert!(!j.criteria[1].met);
        assert!(
            verified(&j, "b", "c", "sha256:x").is_none(),
            "only a pass constructs Verified"
        );
    }

    fn judged_with_mismatch(mismatch: Option<&str>) -> Judgment {
        let mut out = output(vec!["cargo test"], vec![]);
        out.request_reading_mismatch = mismatch.map(String::from);
        judge_with(&out, &["cargo test".to_string()], &[check(true)])
    }

    #[test]
    fn a_request_reading_mismatch_is_a_major_finding_and_the_verdict_is_not_pass() {
        let j = judged_with_mismatch(Some("The request asked for the goal, not the checklist"));
        assert_eq!(j.final_verdict, Verdict::Fail);
        assert!(j.findings.iter().any(|f| f.severity == Severity::Major
            && f.evidence == "The request asked for the goal, not the checklist"));
        assert!(j.reasons.iter().any(|r| r.contains("misread the request")));
    }

    #[test]
    fn an_absent_or_blank_request_reading_mismatch_adds_nothing() {
        for mismatch in [None, Some(""), Some("  \n")] {
            let j = judged_with_mismatch(mismatch);
            assert_eq!(j.final_verdict, Verdict::Pass, "{mismatch:?}");
            assert!(j.findings.is_empty(), "{mismatch:?}");
            assert!(j.reasons.is_empty(), "{mismatch:?}");
        }
    }

    fn passes_when_ran(cited: &str, ran: &str) -> bool {
        let j = judge_with(
            &output(vec![cited], vec![]),
            &[ran.to_string()],
            &[check(true)],
        );
        j.criteria[1].commands_verified && j.criteria[1].met
    }

    #[test]
    fn cited_commands_match_whole_simple_commands_inside_what_ran() {
        let t = "cargo test --locked -p athena-core verify";
        assert!(passes_when_ran(t, t), "run alone");
        assert!(
            passes_when_ran(t, &format!("cargo build && {t}")),
            "after &&"
        );
        assert!(
            passes_when_ran(t, &format!("{t} 2>&1 | tail -n 30")),
            "piped to tail"
        );
        assert!(
            passes_when_ran(&format!("{t} 2>&1"), &format!("{t} 2>&1 | tail -n 30")),
            "cited with its redirect"
        );
        assert!(
            passes_when_ran(t, &format!("a; b || {t}")),
            "after ; and ||"
        );
        assert!(
            passes_when_ran("cargo  test   --locked", "  cargo test --locked  "),
            "extra whitespace"
        );
    }

    #[test]
    fn cited_commands_with_other_arguments_or_never_run_stay_unseen() {
        let ran = "cargo test --locked -p athena-core verify && cargo fmt --check";
        assert!(!passes_when_ran(
            "cargo test --locked -p athena-core other",
            ran
        ));
        assert!(!passes_when_ran(
            "cargo test --locked -p athena-agent verify",
            ran
        ));
        assert!(!passes_when_ran("cargo clippy", ran), "never run");
        assert!(
            !passes_when_ran("cargo test --locked", ran),
            "a prefix is not a command"
        );
        assert!(
            !passes_when_ran("echo b", "echo 'a && echo b'"),
            "never matches inside quotes"
        );
        assert!(!passes_when_ran("", ran), "an empty citation");
    }

    fn judged_with_fault(fault: Option<&str>) -> Judgment {
        let ran = ["cargo test".to_string()];
        judge(&VerdictInputs {
            criteria: &criteria(),
            checks: &[check(true)],
            output: Some(&output(vec!["cargo test"], vec![])),
            commands_run: &ran,
            candidate_tree: None,
            checks_tree: None,
            system_fault: fault.map(String::from),
            verifier_failure: None,
            map_base: None,
            map_candidate: None,
            baseline: None,
            replay: None,
        })
    }

    #[test]
    fn a_system_fault_is_recorded_and_never_rejects_the_work() {
        let clean = judged_with_fault(None);
        assert_eq!(clean.final_verdict, Verdict::Pass);
        assert!(clean.findings.is_empty());
        let faulted = judged_with_fault(Some("snapshot HEAD None"));
        assert_eq!(
            faulted.final_verdict,
            Verdict::Pass,
            "{:?}",
            faulted.reasons
        );
        assert!(faulted.reasons.is_empty(), "{:?}", faulted.reasons);
        assert!(faulted.findings.iter().all(|f| {
            !matches!(f.severity, Severity::Blocker | Severity::Major)
                && !f
                    .summary
                    .contains("changed its snapshot, which invalidates")
        }));
        let fault: Vec<_> = faulted
            .findings
            .iter()
            .filter(|f| f.summary.starts_with(SYSTEM_FAULT_PREFIX))
            .collect();
        assert_eq!(fault.len(), 1, "the fault is recorded once");
        assert_eq!(fault[0].evidence, "snapshot HEAD None");
        assert_eq!(fault[0].source, FindingSource::Athena);
    }

    fn soundness_finding(severity: SeverityOutput) -> FindingOutput {
        FindingOutput {
            severity,
            summary: "Soundness: the refusal rule matches common words, so it has false positives".into(),
            evidence: "The rule refuses any request holding a common word. Read the request's structure instead."
                .into(),
            location: Some("src/reading.rs".into()),
        }
    }

    #[test]
    fn soundness_finding_blocker_fails_the_verdict_with_all_criteria_met() {
        let ran = ["cargo test".to_string()];
        let j = judge_with(
            &output(
                vec!["cargo test"],
                vec![soundness_finding(SeverityOutput::Blocker)],
            ),
            &ran,
            &[check(true)],
        );
        // Every criterion is met and the check passed; the finding alone fails the verdict.
        assert!(j.criteria.iter().all(|c| c.met), "{:?}", j.criteria);
        assert_eq!(j.final_verdict, Verdict::Fail);
        assert!(
            j.reasons
                .iter()
                .any(|r| r == "1 blocker or major finding(s)"),
            "{:?}",
            j.reasons
        );
        assert!(verified(&j, "b", "c", "sha256:x").is_none());
    }

    #[test]
    fn soundness_finding_minor_or_nit_leaves_a_passing_verdict_passing() {
        let ran = ["cargo test".to_string()];
        for severity in [SeverityOutput::Minor, SeverityOutput::Nit] {
            let j = judge_with(
                &output(vec!["cargo test"], vec![soundness_finding(severity)]),
                &ran,
                &[check(true)],
            );
            assert!(j.criteria.iter().all(|c| c.met));
            assert_eq!(j.final_verdict, Verdict::Pass, "{:?}", j.reasons);
            assert!(j.reasons.is_empty(), "{:?}", j.reasons);
            assert_eq!(j.findings.len(), 1);
        }
    }

    #[test]
    fn major_findings_and_failed_checks_fail_the_verdict() {
        let major = FindingOutput {
            severity: SeverityOutput::Major,
            summary: "x".into(),
            evidence: "y".into(),
            location: None,
        };
        let ran = ["cargo test".to_string()];
        assert_eq!(
            judge_with(
                &output(vec!["cargo test"], vec![major]),
                &ran,
                &[check(true)]
            )
            .final_verdict,
            Verdict::Fail
        );
        assert_eq!(
            judge_with(&output(vec!["cargo test"], vec![]), &ran, &[check(false)]).final_verdict,
            Verdict::Fail
        );
        let minor = FindingOutput {
            severity: SeverityOutput::Minor,
            summary: "x".into(),
            evidence: "y".into(),
            location: None,
        };
        assert_eq!(
            judge_with(
                &output(vec!["cargo test"], vec![minor]),
                &ran,
                &[check(true)]
            )
            .final_verdict,
            Verdict::Pass
        );
    }
}
