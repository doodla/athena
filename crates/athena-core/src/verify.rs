//! The final verdict, and the type that alone can land (design §6.3, §7.2; M7, M16).
//!
//! The core computes the final verdict. It passes only if the agent says pass, every criterion is met, no
//! finding is a blocker or major, and every check passed. Each `behavior` criterion must cite at least one
//! command, and every cited command must be one the core saw the verifier run; otherwise the criterion
//! counts as unmet, with a finding that says so (D43).
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

/// Computes the final verdict.
pub fn judge(inputs: &VerdictInputs<'_>) -> Judgment {
    let mut findings = Vec::new();
    let mut reasons = Vec::new();
    let ran: Vec<String> = inputs
        .commands_run
        .iter()
        .map(|c| c.trim().to_string())
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
        let unseen: Vec<&String> = commands.iter().filter(|c| !ran.contains(c)).collect();
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
