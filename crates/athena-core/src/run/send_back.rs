//! The producer's send-back: which accepted criteria a report leaves unmet, which it leaves to Athena's
//! recorded checks, and the brief for the one send-back (structure pass, R2).

use athena_context::briefs::{
    AcceptedReading, LEFT_TO_ATHENA, ReplayView, replay_send_back_section,
};
use athena_context::outputs::{CriterionCheckOutput, CriterionStatusOutput, ProducerOutput};

use crate::mirrors::Drift;

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
    send_back_text_with_replay(brief, unmet, unchanged, None)
}

/// [`send_back_text`], with the replay of accepted readings when the producer's change newly refuses more of
/// them than the limit. `replay` is the measured view and the limit in whole percent that applied. With a
/// replay the intro gains one leading sentence about it and the section follows the lists; the intro's own
/// words are unchanged. With `None` the text is exactly [`send_back_text`]'s.
pub(crate) fn send_back_text_with_replay(
    brief: &str,
    unmet: &[String],
    unchanged: &[String],
    replay: Option<(&ReplayView, usize)>,
) -> String {
    send_back_text_full(brief, unmet, unchanged, replay, None)
}

/// [`send_back_text_with_replay`], with the decision gate's message when the producer's change touches code a
/// decision governs and does not reconcile it. With a message the intro gains one leading sentence about it
/// and the message follows the lists and the replay section, word for word. With `None` the text is exactly
/// [`send_back_text_with_replay`]'s.
pub(crate) fn send_back_text_full(
    brief: &str,
    unmet: &[String],
    unchanged: &[String],
    replay: Option<(&ReplayView, usize)>,
    gate: Option<&str>,
) -> String {
    send_back_text_scoped(brief, unmet, unchanged, replay, gate, None)
}

/// [`send_back_text_full`], with the files the producer changed outside the job's allowed paths. `scope` is
/// those paths and the accepted criterion that limits the files. With a scope the intro gains one leading
/// sentence about it and a section follows the lists: it names each path, quotes the criterion and says
/// to restore each file to its base content, or delete it if the producer added it. With `None` the text
/// is exactly [`send_back_text_full`]'s.
pub(crate) fn send_back_text_scoped(
    brief: &str,
    unmet: &[String],
    unchanged: &[String],
    replay: Option<(&ReplayView, usize)>,
    gate: Option<&str>,
    scope: Option<(&[String], &str)>,
) -> String {
    let mut text = format!("{brief}\n\n## Sent back\n\n");
    if scope.is_some() {
        text.push_str(
            "Your change touches files outside the job's allowed paths; the section below names \
             them. When no criteria, planned files, replay or decision gate message are listed, \
             those files are the only reason for this send-back, and the rest of this paragraph \
             applies to it as to any other item. ",
        );
    }
    if gate.is_some() {
        text.push_str(
            "Your change touches code that a logged decision governs and does not reconcile the \
             decision; the decision gate's message below says how. When no criteria, planned files or \
             replay are listed, that message is the only reason for this send-back, and the rest of \
             this paragraph applies to it as to any other item. ",
        );
    }
    if replay.is_some() {
        text.push_str(
            "Your change makes the reading rules refuse accepted readings of earlier jobs that the base \
             accepted, more than the limit allows; the replay section below lists them. When no \
             criteria or planned files are listed below, the replay is the only reason for this \
             send-back, and the rest of this paragraph applies to it as to any other item. ",
        );
    }
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
    if let Some((view, limit)) = replay {
        if !unchanged.is_empty() {
            text.push('\n');
        }
        text.push_str(&replay_send_back_section(view, limit));
    }
    if let Some(message) = gate {
        if !unmet.is_empty() || !unchanged.is_empty() || replay.is_some() {
            text.push_str("\n\n");
        }
        text.push_str("Decision gate: ");
        text.push_str(message);
        text.push('\n');
    }
    if let Some((outside, criterion)) = scope {
        if !unmet.is_empty() || !unchanged.is_empty() || replay.is_some() || gate.is_some() {
            text.push_str("\n\n");
        }
        text.push_str("Files changed outside the job's allowed paths (revert each):\n\n");
        for path in outside {
            text.push_str(&format!("- {path}\n"));
        }
        text.push_str(&format!(
            "\nThe accepted criterion \"{criterion}\" limits which files may change. The allowed paths \
             are the deliverables the reading lists. Revert each file named above: restore it to its \
             base content, or delete it if you added it. If a listed file is still outside the allowed \
             paths afterwards, the job ends before the verifier runs.\n"
        ));
    }
    text
}

/// The leading sentence of a send-back that lists hand-kept files disagreeing with their source.
const MIRROR_DRIFT_SENTENCE: &str = "Files that are kept by hand disagree with the source they summarize; \
     this is a reason for this send-back whether or not criteria are listed. ";

/// `text`, a send-back brief, with the hand-kept files that disagree with their source (`athena_core::mirrors`).
/// With no drift the text is returned unchanged. Otherwise one leading sentence goes in front of the intro,
/// because the intro speaks of criteria, and a section after the lists names each drift: the file and the fix.
pub(crate) fn append_mirror_drift(text: String, drift: &[Drift]) -> String {
    if drift.is_empty() {
        return text;
    }
    const HEADING: &str = "## Sent back\n\n";
    let mut text = match text.rfind(HEADING) {
        Some(at) => {
            let end = at + HEADING.len();
            format!("{}{MIRROR_DRIFT_SENTENCE}{}", &text[..end], &text[end..])
        }
        None => text,
    };
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str("\nHand-kept files that disagree with their source (fix each):\n\n");
    for d in drift {
        text.push_str(&format!("- {d}\n"));
    }
    text
}

/// `text`, a send-back brief, with a closing section for a `CHANGELOG.md` the change reading asks for and the
/// work tree does not change. With `missing` false the text is returned unchanged.
///
/// Root cause of the failure class: the changelog default was a rule the producer must remember, and no code
/// compared the diff with the deliverables. A reason in `unchanged_planned_files` excused the file, so the
/// section says that a reason does not.
pub(crate) fn append_missing_changelog(mut text: String, missing: bool) -> String {
    if !missing {
        return text;
    }
    if !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(
        "\nCHANGELOG.md is a deliverable (the changelog default applies) and your change does not \
         touch it; add an entry. A reason in unchanged_planned_files does not excuse it. If it is \
         still unchanged afterwards, the job ends at admission, before the verifier runs.\n",
    );
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
mod mirror_drift_tests {
    use super::*;

    fn drift(message: &str) -> Drift {
        Drift {
            mirror: "xtask/shard-timings.txt".into(),
            message: message.into(),
        }
    }

    #[test]
    fn no_drift_leaves_the_text_unchanged() {
        let text = send_back_text("brief", &["c".to_string()], &[]);
        assert_eq!(append_mirror_drift(text.clone(), &[]), text);
    }

    #[test]
    fn drift_adds_a_leading_sentence_and_a_section_naming_each_item() {
        let text = send_back_text("brief", &[], &[]);
        let out = append_mirror_drift(
            text,
            &[
                drift("xtask/shard-timings.txt: add the line `a --test b 5.00`"),
                drift("docs/decisions.md is append-only"),
            ],
        );
        let sent = out.split("## Sent back\n\n").nth(1).unwrap();
        assert!(sent.starts_with(MIRROR_DRIFT_SENTENCE), "{out}");
        assert!(sent.contains("Your previous report did not show every acceptance criterion"));
        assert!(out.contains(
            "(fix each):\n\n- xtask/shard-timings.txt: add the line `a --test b 5.00`\n"
        ));
        assert!(
            out.ends_with("- docs/decisions.md is append-only\n"),
            "{out}"
        );
    }
}

#[cfg(test)]
mod missing_changelog_tests {
    use super::*;

    #[test]
    fn a_changelog_not_missing_leaves_the_text_unchanged() {
        let text = send_back_text("brief", &["c".to_string()], &[]);
        assert_eq!(append_missing_changelog(text.clone(), false), text);
    }

    #[test]
    fn a_missing_changelog_adds_a_section_naming_the_file_and_the_reason_rule() {
        let text = send_back_text("brief", &[], &[]);
        let out = append_missing_changelog(text.clone(), true);
        assert!(out.starts_with(&text), "{out}");
        assert!(out.contains("CHANGELOG.md is a deliverable"), "{out}");
        assert!(out.contains("does not excuse it"), "{out}");
        assert!(out.ends_with('\n'));
    }
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
            failed_tests: Vec::new(),
            reused: false,
            revert_for: Vec::new(),
            queued_s: None,
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
            request_misread: false,
            request_misread_reason: None,
            summary: "ok".into(),
        };
        let checks = [failed];
        let judgment = crate::verify::judge(&crate::verify::VerdictInputs {
            criteria: &criteria,
            checks: &checks,
            output: Some(&output),
            commands_run: &[],
            changed_paths: &[],
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

    fn view(newly: usize, total: usize) -> ReplayView {
        ReplayView {
            total,
            newly_refused: (0..newly)
                .map(|n| (format!("20201001-{n:06}-{n:04}"), "too broad".into()))
                .collect(),
            held: true,
            declared: None,
            reason: None,
            unsupported: None,
            no_replay: None,
        }
    }

    #[test]
    fn a_send_back_without_a_replay_is_the_plain_text_and_with_one_keeps_the_pinned_phrases() {
        let unmet = vec!["a criterion".to_string()];
        let plain = send_back_text("BRIEF", &unmet, &[]);
        assert_eq!(
            plain,
            send_back_text_with_replay("BRIEF", &unmet, &[], None)
        );
        assert!(!plain.contains("Replay of accepted readings"));

        let v = view(3, 10);
        let with =
            send_back_text_with_replay("BRIEF", &unmet, &["src/a.rs".into()], Some((&v, 10)));
        assert!(with.starts_with("BRIEF\n\n## Sent back"), "{with}");
        for phrase in [
            "copying each line from the brief's criteria_check section",
            "unchanged_planned_files",
            "- a criterion",
            "- src/a.rs",
            "Replay of accepted readings: 3 of 10 (30%) newly refused, limit 10%",
            "- 20201001-000002-0002: too broad",
        ] {
            assert!(with.contains(phrase), "omits {phrase}: {with}");
        }
        // The replay section comes after the lists.
        assert!(with.find("- src/a.rs").unwrap() < with.find("Replay of accepted").unwrap());
    }

    #[test]
    fn the_gate_section_appears_only_when_given_and_without_it_the_text_is_unchanged() {
        let unmet = vec!["a criterion".to_string()];
        let unchanged = vec!["src/a.rs".to_string()];
        assert_eq!(
            send_back_text_full("BRIEF", &unmet, &unchanged, None, None),
            send_back_text_with_replay("BRIEF", &unmet, &unchanged, None)
        );
        let message = "decisions not reconciled: D02 (README.md). Add an entry.";
        let with = send_back_text_full("BRIEF", &unmet, &unchanged, None, Some(message));
        assert!(with.starts_with("BRIEF\n\n## Sent back"), "{with}");
        assert!(
            with.contains(&format!("Decision gate: {message}")),
            "{with}"
        );
        // The message follows the lists.
        assert!(with.find("- src/a.rs").unwrap() < with.find("Decision gate:").unwrap());
        // A gate-only send-back lists no criteria or files.
        let only = send_back_text_full("BRIEF", &[], &[], None, Some(message));
        assert!(!only.contains("Criteria not met"), "{only}");
        assert!(
            only.ends_with(&format!("Decision gate: {message}\n")),
            "{only}"
        );
        assert!(!send_back_text("BRIEF", &unmet, &[]).contains("Decision gate"));
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
