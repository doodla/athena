//! The final verdict, and the type that alone can land (design §6.3, §7.2; M7, M16).
//!
//! The core computes the final verdict. It passes only if the agent says pass, every criterion is met, no
//! finding is a blocker or major, and every check passed. Each `behavior` criterion must cite at least one
//! command, and every cited command must be one the core saw the verifier run, either as a whole command or
//! as one whole simple command inside a chained or piped one; otherwise the criterion counts as unmet, with a
//! finding that says so (D43, D63).
//!
//! [`Verified`] has private fields and is constructed only here, only for a pass, so `land()` cannot be
//! called with anything unverified.

use athena_context::outputs::{SeverityOutput, VerdictOutput, VerdictValue};
use athena_data::records::{
    CheckRun, Criterion, CriterionKind, CriterionResult, Finding, FindingSource, Severity, Verdict,
    Verification,
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
    /// The commands the core saw the verifier run.
    pub commands_run: &'a [String],
    /// Set when the verifier changed its snapshot, which invalidates its verdict.
    pub snapshot_changed: Option<String>,
    /// Set when the verifier did not complete.
    pub verifier_failure: Option<String>,
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
    let ran: Vec<String> = inputs
        .commands_run
        .iter()
        .flat_map(|c| simple_commands(c))
        .collect();

    for check in inputs.checks.iter().filter(|c| !c.passed) {
        reasons.push(format!("check {} failed", check.name));
    }
    if let Some(failure) = &inputs.verifier_failure {
        reasons.push(format!("the verifier did not complete: {failure}"));
    }
    if let Some(change) = &inputs.snapshot_changed {
        reasons.push("the verifier changed its snapshot".into());
        findings.push(athena_finding(
            Severity::Blocker,
            "The verifier changed its snapshot, which invalidates its verdict".into(),
            change.clone(),
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
    } else if inputs.verifier_failure.is_none() && inputs.checks.iter().all(|c| c.passed) {
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
        && inputs.checks.iter().all(|c| c.passed)
        && inputs.snapshot_changed.is_none()
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

#[cfg(test)]
mod tests {
    use super::*;
    use athena_context::outputs::{CriterionVerdictOutput, FindingOutput};
    use athena_data::{Quantity, Unit};

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
            snapshot_changed: None,
            verifier_failure: None,
        })
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
