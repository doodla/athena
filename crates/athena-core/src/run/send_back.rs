//! The producer's send-back: which accepted criteria a report leaves unmet, which it leaves to Athena's
//! recorded checks, and the brief for the one send-back (structure pass, R2).

use athena_context::briefs::{AcceptedReading, LEFT_TO_ATHENA};
use athena_context::outputs::{CriterionCheckOutput, CriterionStatusOutput, ProducerOutput};

/// The accepted criteria the report does not show as met: marked unmet, or missing from `criteria_check`.
/// A criterion matches an entry by its text with numbering, kind tag, case, spacing and trailing
/// punctuation ignored (see [`criterion_key`]); any other difference in words is a different criterion.
///
/// A criterion the report marks as left to Athena's checks (see [`LEFT_TO_ATHENA`]) is not unmet when
/// its text names Athena's own recorded checks (see [`is_athena_check_criterion`]); the verifier judges
/// it from the recorded checks instead.
pub(crate) fn unmet_criteria(report: &ProducerOutput, accepted: &AcceptedReading) -> Vec<String> {
    accepted
        .criteria
        .iter()
        .map(|(text, _)| text)
        .filter(|text| {
            let key = criterion_key(text);
            let left = is_athena_check_criterion(text);
            !report.criteria_check.iter().any(|c| {
                criterion_key(&c.criterion) == key
                    && (c.status == CriterionStatusOutput::Met || (left && marked_left(c)))
            })
        })
        .cloned()
        .collect()
}

/// The accepted criteria the report leaves to Athena's checks: marked with [`LEFT_TO_ATHENA`] and
/// naming Athena's own recorded checks. They go to the verifier to judge from the recorded checks.
pub(crate) fn left_to_athena(report: &ProducerOutput, accepted: &AcceptedReading) -> Vec<String> {
    accepted
        .criteria
        .iter()
        .map(|(text, _)| text)
        .filter(|text| {
            let key = criterion_key(text);
            is_athena_check_criterion(text)
                && report
                    .criteria_check
                    .iter()
                    .any(|c| criterion_key(&c.criterion) == key && marked_left(c))
        })
        .cloned()
        .collect()
}

/// Whether the entry's evidence starts with the [`LEFT_TO_ATHENA`] marker (case and outer spacing ignored).
fn marked_left(check: &CriterionCheckOutput) -> bool {
    check
        .evidence
        .trim()
        .to_lowercase()
        .starts_with(&LEFT_TO_ATHENA.to_lowercase())
}

/// Whether a criterion's text is one only Athena's recorded checks (fmt, clippy, the full test suite)
/// can show: it names `cargo fmt`, `cargo clippy` or `cargo test`, and no targeted-test scope such as
/// `-p`, `--package`, `--test`, `--lib`, `--bin`, `--doc`, `--bench` or `--example`.
fn is_athena_check_criterion(text: &str) -> bool {
    let lower = text.to_lowercase();
    if !["cargo fmt", "cargo clippy", "cargo test"]
        .iter()
        .any(|k| lower.contains(k))
    {
        return false;
    }
    const SCOPES: [&str; 8] = [
        "-p",
        "--package",
        "--test",
        "--lib",
        "--bin",
        "--doc",
        "--bench",
        "--example",
    ];
    !lower
        .split_whitespace()
        .map(|t| t.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-'))
        .any(|t| SCOPES.contains(&t))
}

/// The brief for the producer's one send-back: the original brief, then the criteria the report did not
/// show as met, each with its exact text, and the planned files left unchanged without a reason.
pub(crate) fn send_back_text(brief: &str, unmet: &[String], unchanged: &[String]) -> String {
    let mut text = format!("{brief}\n\n## Sent back\n\n");
    text.push_str(
        "Your previous report did not show every acceptance criterion as met in criteria_check, \
         or left planned files unchanged without a reason. \
         Your edits are still in your working directory. Fix the work for each item below, \
         then report every acceptance criterion in criteria_check, copying each line from the brief's \
         criteria_check section as its text, with its status and your evidence. A planned file \
         listed below must be changed, or explained with its path and a reason in \
         unchanged_planned_files in your report. This is the only send-back: \
         if your changes are still in place afterwards, they go to the verifier with the criteria \
         your report still leaves unmet or unreported, and the planned files still unchanged and \
         unexplained, flagged, and the verifier judges them. \
         A run that leaves no change fails.\n\n",
    );
    if !unmet.is_empty() {
        text.push_str("Criteria not met or not reported:\n\n");
        for criterion in unmet {
            text.push_str(&format!("- {criterion}\n"));
        }
        text.push('\n');
    }
    if !unchanged.is_empty() {
        text.push_str("Planned files left unchanged and not explained:\n\n");
        for path in unchanged {
            text.push_str(&format!("- {path}\n"));
        }
    }
    text
}

/// A criterion's text reduced for comparison: no leading number (`1.`, `2)`, `3:`), no leading `[kind]`
/// tag, lowercase, single spaces, no trailing punctuation.
fn criterion_key(text: &str) -> String {
    let mut rest = text.trim();
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    if digits > 0
        && let Some(after) = rest[digits..].strip_prefix(['.', ')', ':'])
    {
        rest = after.trim_start();
    }
    if let Some(inner) = rest.strip_prefix('[')
        && let Some(end) = inner.find(']')
        && inner[..end].chars().all(|c| c.is_ascii_alphabetic())
    {
        rest = inner[end + 1..].trim_start();
    }
    let words = rest.split_whitespace().collect::<Vec<_>>().join(" ");
    words
        .trim_end_matches(|c: char| c.is_ascii_punctuation() && c != '`' && c != ')')
        .trim_end()
        .to_lowercase()
}

#[cfg(test)]
mod left_to_athena_tests {
    use super::*;
    use athena_context::outputs::{CriterionVerdictOutput, VerdictOutput, VerdictValue};
    use athena_data::records::CheckRun;
    use athena_data::records::{Criterion, CriterionKind, Verdict};
    use serde_json::json;

    const FULL_SUITE: &str = "cargo clippy --all-targets and cargo fmt --check are clean, and cargo test for the workspace passes";
    const TARGETED: &str = "cargo test -p athena-core passes the new test";
    const CONTENT: &str = "the docs name the flag";

    fn accepted(criteria: &[&str]) -> AcceptedReading {
        AcceptedReading {
            n: 1,
            title: "t".into(),
            summary: "s".into(),
            assumptions: vec![],
            criteria: criteria
                .iter()
                .map(|c| (c.to_string(), CriterionKind::Behavior))
                .collect(),
            deliverables: vec![],
            out_of_scope: vec![],
            defaults: vec![],
        }
    }

    /// A report with one entry per `(criterion, status, evidence)`.
    fn report(entries: &[(&str, &str, &str)]) -> ProducerOutput {
        let checks: Vec<_> = entries
            .iter()
            .map(|(c, s, e)| json!({ "criterion": c, "status": s, "evidence": e }))
            .collect();
        serde_json::from_value(json!({
            "summary": "done", "files": [], "checks_run": [], "criteria_check": checks,
            "open_issues": [], "mistakes": [], "learnings": [], "desires": []
        }))
        .unwrap()
    }

    fn left(why: &str) -> String {
        format!("{LEFT_TO_ATHENA}: {why}")
    }

    #[test]
    fn a_full_suite_criterion_left_to_athenas_checks_is_not_sent_back_but_an_omitted_targeted_one_is()
     {
        let accepted = accepted(&[FULL_SUITE, TARGETED]);
        let r = report(&[(
            FULL_SUITE,
            "unmet",
            &left("Athena records fmt, clippy and the suite"),
        )]);
        assert_eq!(unmet_criteria(&r, &accepted), vec![TARGETED.to_string()]);
        assert_eq!(left_to_athena(&r, &accepted), vec![FULL_SUITE.to_string()]);
        // Marked met works too, and the marker ignores case.
        let r = report(&[(FULL_SUITE, "met", "LEFT TO ATHENA'S CHECKS")]);
        assert_eq!(unmet_criteria(&r, &accepted), vec![TARGETED.to_string()]);
    }

    #[test]
    fn the_marker_does_not_excuse_other_criteria_or_an_unmarked_unmet_one() {
        let accepted = accepted(&[CONTENT, TARGETED, FULL_SUITE]);
        let r = report(&[
            (CONTENT, "unmet", &left("trust me")),
            (TARGETED, "unmet", &left("trust me")),
            (FULL_SUITE, "unmet", "did not run it"),
        ]);
        assert_eq!(
            unmet_criteria(&r, &accepted),
            vec![
                CONTENT.to_string(),
                TARGETED.to_string(),
                FULL_SUITE.to_string()
            ]
        );
        assert!(left_to_athena(&r, &accepted).is_empty());
    }

    #[test]
    fn a_failed_recorded_check_still_fails_a_criterion_the_producer_left_to_athena() {
        let accepted = accepted(&[FULL_SUITE]);
        let r = report(&[(FULL_SUITE, "unmet", &left("Athena records the suite"))]);
        assert!(unmet_criteria(&r, &accepted).is_empty(), "not sent back");
        let left_over = left_to_athena(&r, &accepted);
        assert_eq!(left_over, vec![FULL_SUITE.to_string()]);

        let criteria: Vec<Criterion> = left_over
            .into_iter()
            .map(|text| Criterion {
                text,
                kind: CriterionKind::Behavior,
            })
            .collect();
        let failed = CheckRun {
            name: "test".into(),
            command: vec!["cargo".into(), "test".into()],
            exit_code: Some(101),
            passed: false,
            tests_ran: Some(3),
            duration: athena_data::Quantity::measured(1.0, athena_data::Unit::Seconds, "test"),
            output_tail: String::new(),
            note: None,
        };
        let output = VerdictOutput {
            verdict: VerdictValue::Pass,
            criteria: vec![CriterionVerdictOutput {
                criterion: FULL_SUITE.into(),
                met: true,
                evidence: "judged from the recorded checks".into(),
                commands: vec![],
            }],
            findings: vec![],
            request_reading_mismatch: None,
            summary: "ok".into(),
        };
        let checks = [failed];
        let judgment = crate::verify::judge(&crate::verify::VerdictInputs {
            criteria: &criteria,
            checks: &checks,
            output: Some(&output),
            commands_run: &[],
            candidate_tree: None,
            checks_tree: None,
            system_fault: None,
            verifier_failure: None,
            map_base: None,
            map_candidate: None,
            baseline: None,
            replay: None,
        });
        assert_eq!(judgment.final_verdict, Verdict::Fail);
        assert!(
            judgment.reasons.iter().any(|r| r == "check test failed"),
            "{:?}",
            judgment.reasons
        );
    }

    #[test]
    fn only_unscoped_cargo_checks_count_as_athena_check_criteria() {
        assert!(is_athena_check_criterion(FULL_SUITE));
        assert!(is_athena_check_criterion("cargo fmt --check is clean."));
        assert!(!is_athena_check_criterion(TARGETED));
        assert!(!is_athena_check_criterion("cargo test --lib passes"));
        assert!(!is_athena_check_criterion(CONTENT));
    }
}
