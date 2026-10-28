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
    Baseline, CheckRun, Criterion, CriterionKind, CriterionResult, Finding, FindingSource,
    PRE_EXISTING_NOTE, Replay, Severity, Verdict, Verification,
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
    /// Set when the verifier did not complete.
    pub verifier_failure: Option<String>,
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

/// True when `check` failed now and the baseline shows the same check failing on the base commit.
pub fn is_pre_existing(check: &CheckRun, baseline: Option<&Baseline>) -> bool {
    !check.passed && baseline.is_some_and(|b| b.failed_before(&check.name))
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
    // Athena's own passing record for this very tree counts as a command run (D115). Skipped checks and
    // nested-run placeholders carry no exit code and never count.
    if inputs.candidate_tree.is_some() && inputs.candidate_tree == inputs.checks_tree {
        for check in inputs
            .checks
            .iter()
            .filter(|c| c.passed && c.exit_code == Some(0))
        {
            ran.extend(simple_commands(&check.command.join(" ")));
        }
    }

    let blocking_checks = |c: &&CheckRun| !c.passed && !is_pre_existing(c, inputs.baseline);
    for check in inputs.checks.iter().filter(blocking_checks) {
        reasons.push(format!("check {} failed", check.name));
    }
    if let Some(failure) = &inputs.verifier_failure {
        reasons.push(format!("the verifier did not complete: {failure}"));
    }
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
                evidence: "the verifier did not assess this criterion".into(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use athena_context::outputs::{CriterionVerdictOutput, FindingOutput};
    use athena_data::{Quantity, Unit};

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
