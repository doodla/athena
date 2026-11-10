//! The request's `Acceptance:` bullets that no criterion of the reading covers.

use super::*;

/// The least share, in whole percent, of a bullet's own content stems that its best-matching criterion must
/// hold for the bullet to count as covered. Set from the scores `coverage_threshold_is_tuned_on_the_recorded_shapes`
/// prints for the recorded jobs: every covered bullet there scores at least this, the omitted one scores below.
pub const COVERAGE_MIN_PERCENT: u64 = 25;

/// Function words and request boilerplate that say nothing about what a bullet asks for. Lowercase.
const STOP_WORDS: [&str; 64] = [
    "a", "an", "the", "and", "or", "of", "to", "in", "on", "at", "by", "for", "with", "from", "as",
    "is", "are", "was", "were", "be", "been", "it", "its", "that", "this", "these", "those",
    "then", "than", "not", "no", "but", "if", "so", "all", "any", "each", "one", "only", "also",
    "when", "which", "who", "has", "have", "had", "does", "do", "did", "can", "may", "will",
    "would", "should", "must", "there", "their", "they", "into", "over", "new", "where", "how",
    "what",
];

/// Stems of the verbs every acceptance bullet and criterion uses to say that something is checked (`a test
/// shows`, `all tests pass`). They say nothing of what is asked, so a criterion that shares only them with a
/// bullet does not cover it. Counted, they pad the score of a bullet with words that any criterion about a
/// test holds, so a criterion about another part would come closer to covering it.
const GENERIC_STEMS: [&str; 3] = ["test", "show", "pass"];

/// One acceptance bullet of a request that its reading leaves uncovered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CoverageGap {
    /// The bullet text, marker and `[tag]` removed.
    pub bullet: String,
    /// The best share, in whole percent, of the bullet's stems that one criterion holds.
    pub best_percent: u64,
}

/// The content stems of `text`: lowercase alphanumeric runs, without stop words and one-character tokens, each
/// reduced by [`stem`]. A leading `[tag]` is ignored.
pub(super) fn coverage_stems(text: &str) -> BTreeSet<String> {
    let text = text.trim_start();
    // A steward request's bullet `The change is made as described: <title>` points at the request's own
    // description; the title after it is not something a criterion repeats, so the bullet has no stems.
    if text.contains("as described:") {
        return BTreeSet::new();
    }
    let text = match text.strip_prefix('[').and_then(|t| t.split_once(']')) {
        Some((_, rest)) => rest,
        None => text,
    };
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() > 1 && !STOP_WORDS.contains(w))
        .map(stem)
        .filter(|s| s.chars().count() > 1 && !GENERIC_STEMS.contains(&s.as_str()))
        .collect()
}

/// A word reduced so that `flag`, `flags`, `flagged` and `flagging` agree: one suffix of `ing`, `ed`, `es` or
/// `s` is removed when at least three characters remain (never from a word ending in `ss`), a doubled final
/// consonant left by `ed` or `ing` is collapsed (not `l`, `s`, `z`), and a final `e` is dropped.
fn stem(word: &str) -> String {
    let mut w = word.to_string();
    let mut doubled = false;
    if !w.ends_with("ss") {
        for suffix in ["ing", "ed", "es", "s"] {
            if w.len() >= suffix.len() + 3 && w.ends_with(suffix) {
                w.truncate(w.len() - suffix.len());
                doubled = suffix == "ing" || suffix == "ed";
                break;
            }
        }
    }
    if doubled {
        let b = w.as_bytes();
        let n = b.len();
        if n >= 2
            && b[n - 1] == b[n - 2]
            && !b"aeiouylsz".contains(&b[n - 1])
            && b[n - 1].is_ascii_alphabetic()
        {
            w.truncate(n - 1);
        }
    }
    if w.len() > 3 && w.ends_with('e') {
        w.pop();
    }
    w
}

/// The acceptance bullets of `request`, each with its marker removed, or an empty list when the request has no
/// `Acceptance:` or `Acceptance criteria:` heading.
///
/// The section starts after the heading line. A line that passes [`is_bullet`] starts a bullet. A non-empty
/// line directly under a bullet (no blank line between) continues it when it is indented, or when the line
/// before does not end a sentence (`.`, `!`, `?`, `:`), the line does not start with a capital letter and it is
/// not a heading (ends in `:` or starts with `Files:` or `Priority:`). The section ends at a heading line or at
/// a non-bullet line after a blank line.
pub(super) fn request_acceptance_bullets(request: &str) -> Vec<String> {
    let mut lines = request.lines();
    let found = lines.any(|l| {
        let t = l
            .trim()
            .trim_start_matches(['#', '*'])
            .trim()
            .trim_end_matches('*');
        let t = t.to_lowercase();
        t == "acceptance:" || t == "acceptance criteria:"
    });
    if !found {
        return Vec::new();
    }
    let mut bullets: Vec<String> = Vec::new();
    let mut open = false;
    let mut after_blank = false;
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            open = false;
            after_blank = true;
            continue;
        }
        if is_bullet(trimmed) {
            bullets.push(strip_marker(trimmed));
            open = true;
            after_blank = false;
            continue;
        }
        let heading = trimmed.ends_with(':')
            || trimmed.starts_with("Files:")
            || trimmed.starts_with("Priority:");
        if open && !after_blank && !heading {
            let indented = line.starts_with([' ', '\t']);
            let last = bullets.last().map(String::as_str).unwrap_or("");
            let ended = last.ends_with(['.', '!', '?', ':']);
            let mid_sentence = trimmed.starts_with(|c: char| !c.is_uppercase());
            if indented || (!ended && mid_sentence) {
                if let Some(last) = bullets.last_mut() {
                    last.push(' ');
                    last.push_str(trimmed);
                }
                continue;
            }
        }
        break;
    }
    bullets
}

/// `bullet` without its `-`, `*` or `N.` marker.
fn strip_marker(bullet: &str) -> String {
    let rest = if let Some(r) = bullet
        .strip_prefix("- ")
        .or_else(|| bullet.strip_prefix("* "))
    {
        r
    } else {
        bullet.split_once(". ").map_or(bullet, |(_, r)| r)
    };
    rest.trim().to_string()
}

/// Words, lowercase, that let the reader do what follows.
const SPLIT_MODALS: [&str; 7] = [
    "may",
    "can",
    "could",
    "free to",
    "allowed to",
    "ok to",
    "okay to",
];

/// What a split is of, lowercase: the job, the request or the work.
const SPLIT_WHOLES: [&str; 6] = [
    "this job",
    "the job",
    "this request",
    "the request",
    "this work",
    "the work",
];

/// What a split makes, lowercase, so that `split it into jobs` names the pieces.
const SPLIT_PIECES: [&str; 7] = [
    "it into jobs",
    "it into parts",
    "it into follow-ups",
    "it into separate jobs",
    "into separate jobs",
    "into follow-up jobs",
    "into follow-ups",
];

/// Whether a sentence denies or questions a split: it holds a question mark or a negation token.
fn has_negation(sentence: &str) -> bool {
    sentence.contains('?')
        || sentence.contains("instead of")
        || sentence
            .split(|c: char| !(c.is_alphanumeric() || c == '\''))
            .any(|w| {
                matches!(
                    w,
                    "not"
                        | "never"
                        | "no"
                        | "cannot"
                        | "without"
                        | "avoid"
                        | "forbid"
                        | "forbidden"
                        | "dont"
                ) || w.ends_with("n't")
            })
}

/// Whether the request's own prose (its lines that are not bullets) affirmatively lets the reader split the
/// job: a sentence with a modal and `split` and the job, request or work (or the pieces), or an imperative
/// `split this job`, and no negation and no question mark. `Do not split this job` and `Should we split this
/// job?` allow nothing, and neither does code prose such as `Split the log line into fields`.
pub(super) fn request_allows_split(request: &str) -> bool {
    let prose = request
        .lines()
        .filter(|l| !is_bullet(l.trim()))
        .collect::<Vec<_>>()
        .join("\n")
        .to_lowercase()
        .replace(['\u{2019}', '\u{2018}'], "'");
    sentences(&prose).into_iter().any(|sentence| {
        if has_negation(sentence) {
            return false;
        }
        let modal = SPLIT_MODALS.iter().any(|m| {
            SPLIT_WHOLES
                .iter()
                .chain(&SPLIT_PIECES)
                .any(|o| has_phrase(sentence, &format!("{m} split {o}")))
        });
        let imperative = SPLIT_WHOLES.iter().any(|o| {
            let sentence = sentence.trim_start();
            let sentence = sentence.strip_prefix("please ").unwrap_or(sentence);
            sentence.starts_with(&format!("split {o}"))
        });
        modal || imperative
    })
}

/// `|a ∩ b|`.
fn overlap(a: &BTreeSet<String>, b: &BTreeSet<String>) -> usize {
    a.intersection(b).count()
}

/// Whether the share of `a` held by `c` is above the share of `b` held by `c`, in integers.
fn better_share(a: &BTreeSet<String>, b: &BTreeSet<String>, c: &BTreeSet<String>) -> bool {
    (overlap(a, c) * b.len()) > (overlap(b, c) * a.len())
}

/// Whether sibling bullet `s` claims criterion `c` over bullet `b`: `c` holds a word only `s` has and none
/// that only `b` has, and it matches `s` better than `b`. Such a criterion was written for `s`; it shares
/// the framing words of `b` and says nothing of `b`'s own.
fn claimed_by(s: &BTreeSet<String>, b: &BTreeSet<String>, c: &BTreeSet<String>) -> bool {
    let only_s = s.difference(b).any(|w| c.contains(w));
    let only_b = b.difference(s).any(|w| c.contains(w));
    only_s && !only_b && better_share(s, b, c)
}

/// Each bullet of `request` with the share of its own content stems that its best criterion holds, among the
/// criteria no sibling bullet claims ([`claimed_by`]), as (bullet, stems held, stems of the bullet). A bullet
/// with no content stems is left out: it is covered. No word is removed because the request's prose repeats it.
pub(super) fn coverage_scores(request: &str, criteria: &[&str]) -> Vec<(String, u64, u64)> {
    let bullets = request_acceptance_bullets(request);
    let stemmed: Vec<BTreeSet<String>> = bullets.iter().map(|b| coverage_stems(b)).collect();
    let criteria: Vec<BTreeSet<String>> = criteria.iter().map(|c| coverage_stems(c)).collect();
    let mut scores = Vec::new();
    for (i, bullet) in bullets.iter().enumerate() {
        let own = &stemmed[i];
        if own.is_empty() {
            continue;
        }
        let best = criteria
            .iter()
            .filter(|c| {
                !stemmed
                    .iter()
                    .enumerate()
                    .any(|(j, s)| j != i && s != own && claimed_by(s, own, c))
            })
            .map(|c| overlap(own, c))
            .max()
            .unwrap_or(0) as u64;
        scores.push((bullet.clone(), best, own.len() as u64));
    }
    scores
}

/// The bullets of `request` that none of `criteria` covers: those whose [`coverage_scores`] share is below
/// [`COVERAGE_MIN_PERCENT`].
pub(super) fn coverage_gaps(request: &str, criteria: &[&str]) -> Vec<CoverageGap> {
    coverage_scores(request, criteria)
        .into_iter()
        .filter(|(_, best, total)| best * 100 < COVERAGE_MIN_PERCENT * total)
        .map(|(bullet, best, total)| CoverageGap {
            bullet,
            best_percent: best * 100 / total,
        })
        .collect()
}

/// Refuses a reading that leaves out an `Acceptance:` bullet of its request: no criterion holds
/// [`COVERAGE_MIN_PERCENT`] percent of the bullet's own content words. The error names each omitted bullet,
/// the best share and the threshold, so the reading can cover it in the bullet's own words.
///
/// Root cause this check removes: a reading chose a split the request never allowed. Job 20261029-084645-sza7,
/// the 2026-10-29 job rejected for a split the reading chose, dropped the Acceptance bullet for the status
/// report (part 3 of its request) and moved that part to a follow-up. Nothing compared the request's bullets
/// with the reading's criteria, so the omission was found only at the verdict. The rule is: a split may be
/// proposed only as a question, never by dropping a bullet; the reading's `question` is how the owner is
/// asked. A request allows a split when its own prose affirmatively says so (`You may split this job`);
/// `Do not split this job` does not.
///
/// Scoring: each bullet is scored against its best criterion by the share of the bullet's own stems that the
/// criterion holds, and a word the request's prose repeats still counts. A criterion that holds a word only a
/// sibling bullet has, none that only this bullet has, and matches the sibling better is that sibling's and
/// does not cover this bullet. A reading that asks a question or declines is not checked, nor is a request
/// with no `Acceptance:` heading. Out-of-scope items never count as coverage.
///
/// This check refuses more readings; the higher refusal rate is intended. The expected share of newly refused
/// readings was set at 10% (an estimate). Run over the 793 accepted readings in the records snapshot of the
/// change it refused 8 (1%), none of the newest 50; the changelog states the figure. The check is a step of
/// [`check_reading_acceptance`], and [`COVERAGE_MIN_PERCENT`] is tuned on the recorded readings of jobs
/// 20261029-084645-sza7, 20261107-031902-ttnw and 20261109-161505-bp5e.
pub fn check_reading_coverage(reading: &Reading, request: &str) -> Result<()> {
    if reading.capability.is_none()
        || reading.decline_reason.is_some()
        || reading.question.is_some()
    {
        return Ok(());
    }
    if request_allows_split(request) {
        return Ok(());
    }
    let criteria: Vec<&str> = reading
        .acceptance_criteria
        .iter()
        .map(|c| c.text.as_str())
        .collect();
    let gaps = coverage_gaps(request, &criteria);
    if gaps.is_empty() {
        return Ok(());
    }
    let list = gaps
        .iter()
        .map(|g| {
            format!(
                "\"{}\" (best criterion holds {}% of its words)",
                g.bullet.chars().take(120).collect::<String>(),
                g.best_percent
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    bail!(
        "the reading leaves out {} acceptance bullet(s) of the request: {list}. A criterion must hold at least {COVERAGE_MIN_PERCENT}% of a bullet's own words. \
         Cover each bullet in a criterion in the bullet's own words, or ask the split as a question; a split may \
         be proposed only as a question, never by dropping a bullet",
        gaps.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::records::{Criterion, CriterionKind, Forecast, Forecasts, Quantiles};

    /// The request of job 20261029-084645-sza7, verbatim from its recorded request.
    const SZA7_REQUEST: &str = r#"Priority: urgent
The test check is now cut off at random. [budget] check_timeout_s in athena.toml is 900 seconds, and the full suite (1617 tests) takes 750 to 940 seconds, so a run near the limit is killed partway and recorded as a failed test check: on 2026-10-29 job 20261026-232434-6wfq was rejected with 'check test failed' after only 591 tests ran, exit code none, 905 s, while the verifier's own full run of the same tree passed. Fix: (1) raise check_timeout_s to 1800 in athena.toml, with the schema accepting it unchanged; (2) record a check killed by its time limit as an environment stop (the existing environment-reason path), not a failed check, so the job is held and retried rather than rejected, and print the measured duration against the limit; (3) report in athena status the median and slowest test-check durations over the last 20 jobs against the limit, flagged when the slowest exceeds 80% of it.
Acceptance:
- A test with a fake check runner that exceeds its time limit shows the job ends as an environment stop, not a rejection, and a real test failure is still a failed check.
- A test shows status reports the median and slowest test-check durations and flags one over 80% of the limit.
- athena.toml has check_timeout_s = 1800 and the repository config test passes.
- fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
Files: athena.toml, crates/athena-core/src/run/mod.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/status.rs, CHANGELOG.md"#;

    /// The status bullet of [`SZA7_REQUEST`], the one its reading left out.
    const SZA7_STATUS_BULLET: &str = "A test shows status reports the median and slowest test-check durations and flags one over 80% of the limit.";

    /// The seven acceptance criteria of the recorded reading of job 20261029-084645-sza7, verbatim. None of
    /// them covers the status bullet: the reading moved that part to a follow-up.
    const SZA7_CRITERIA: [&str; 7] = [
        r#"athena.toml has check_timeout_s = 1800 (Grep athena.toml), and crates/athena-data/src/config.rs still declares check_timeout_s: u64 unchanged."#,
        r#"Run cargo test --locked: a new test with a fake check runner whose check is killed at its deadline (exit code None, deadline note) shows the job ends closed as an environment stop (disk::is_environment_reason true on its reason, resumable_stop true), not Rejected; the test asserts this."#,
        r#"Run cargo test --locked: a companion test shows a fake check that fails with a real nonzero exit code is still a failed check that leads to rejection, not an environment stop."#,
        r#"Run cargo test --locked: a test shows the timeout stop message and printed line contain the measured duration and the limit (for example '905s of 900s limit'), and the repository config test (the one loading athena.toml) passes with 1800."#,
        r#"crates/athena-core/src/disk.rs recognises the new check-timeout reason in is_environment_reason (Grep disk.rs), with a unit test in the same file for it."#,
        r#"Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked: all pass."#,
        r#"CHANGELOG.md has an entry describing the 1800 s limit and the timeout-as-environment-stop behavior, citing job 20261026-232434-6wfq."#,
    ];

    /// A criterion that covers the status bullet in the words of its part (3).
    const SZA7_STATUS_CRITERION: &str = "Run cargo test -p athena-core: a test shows athena status reports the median and slowest test-check durations over the last 20 jobs and flags one over 80% of the limit.";

    /// The request of job 20261107-031902-ttnw, verbatim from its recorded request.
    const TTNW_REQUEST: &str = r#"Give Athena's own improvement requests a guaranteed turn in the drain.

Requests from the Steward and from Hestia's error review often name the right cause, but they rarely run: of 32 Steward-sourced requests filed 4 to 6 Nov, 5 ran, because requests a person marks urgent or high always sort first.

Change the drain's ordering so that:
- at least one job in every three dispatched comes from Athena's own improvement requests (Steward or Hestia) when any are queued and their reading passes
- an improvement request whose cause has been recorded in three or more distinct jobs is treated as urgent
- the plan and the drain report show which jobs took the improvement turn

Acceptance:
- a test: with six person-filed urgent requests and two Steward requests queued, the first three dispatches include at least one Steward request
- a test: a Hestia request whose failure class appears in three distinct jobs is ordered as urgent
- a test: with no improvement requests queued, ordering is unchanged
- athena plan prints which items take the improvement turn
- all tests pass; CHANGELOG states only what the diff does

Files changed: the drain's ordering and planning code, the plan display, tests, CHANGELOG.md."#;

    /// The eight acceptance criteria of the recorded reading of job 20261107-031902-ttnw, verbatim.
    const TTNW_CRITERIA: [&str; 8] = [
        r#"Run `cargo test -p athena-core`: a new test queues six person-filed urgent requests and two Steward requests, builds the plan, and asserts the first three dispatched items include at least one Steward request."#,
        r#"Run `cargo test -p athena-core`: a new test gives a Hestia request whose failure class appears in introspections of three distinct jobs and asserts it is ordered as urgent; a companion case with two distinct jobs asserts it is not promoted."#,
        r#"Run `cargo test`: a new test with no Steward or Hestia requests queued asserts the plan order is identical to the order before the change, and the existing plan tests still pass."#,
        r#"Run `cargo test -p athena-cli`: a test of the `athena plan` output (display::plan or the CLI drain test) asserts it names each item that takes the improvement turn, and that the drain report lists them."#,
        r#"An improvement request whose reading fails (missing goal, rung or acceptance) or whose dependency is unmet is not given a turn; a test or the diff in plan.rs shows the turn is taken only from ready, passing entries."#,
        r#"Run `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test`: all pass."#,
        r#"CHANGELOG.md has an entry for the guaranteed improvement turn, the three-job urgent promotion and the plan and report display, and each claim matches the diff, with nothing claimed beyond it."#,
        r#"The changed files are limited to the ordering and planning code, the plan display, tests and CHANGELOG.md, with no change to athena.toml or defaults.toml."#,
    ];

    /// The request of job 20261109-161505-bp5e, verbatim from its recorded request (the recorded text is cut
    /// off at `crates/`).
    const BP5E_REQUEST: &str = r#"Priority: high

Fix the root cause of a repeated failure: The changelog default is a rule the producer must remember, and no code compares the diff's file lis

The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.

Failure class: changelog-default-deliverable-not-produced-caught-only-at-final-verdict (20261108-185500-x0jg)
Cause: The changelog default is a rule the producer must remember, and no code compares the diff's file list with the expected deliverables before the verdict. For doc-only design jobs the changelog feels optional, so it is easy to skip. The readi...
Seen 2 time(s) in 2 jobs. The introspections that record it:
Introspection 20261108-185451-t573 (job 20261108-183842-fsaz); prevention it proposed: Add a code check that runs before the verifier. When the changelog default applies to a job, it fails fast if the diff does not touch CHANGELOG.md. It should report the missing file to the producer, which gets a repair round. The reading sh...
Introspection 20261109-161451-cn7n (job 20261028-000831-rkv5); prevention it proposed: Add a code check before handover and verification. When the changelog default applies, the check fails the producer's run if the diff has no CHANGELOG.md change, and it names the missing file. Readings should list CHANGELOG.md as a delivera...

Acceptance:
- The root cause is named in the change description and fixed where it originates, not at one instance
- A test fails without the change and passes with it, for the cause and not only for one job
- fmt, clippy and test pass

Files: crates/"#;

    /// The six acceptance criteria of the recorded reading of job 20261109-161505-bp5e, verbatim.
    const BP5E_CRITERIA: [&str; 6] = [
        r#"crates/athena-core/src/admission.rs contains a rule that emits a PolicyViolation naming CHANGELOG.md when the reading's deliverables list it, defaults_applied has id `changelog`, and no change in input.changes has path CHANGELOG.md; its doc comment names the root cause (the default was a rule the producer must remember, with no code comparing the diff to the deliverables)."#,
        r#"Run `cargo test -p athena-core admission`: new tests show a candidate that changes only docs/ files with a change reading carrying the changelog default yields a violation naming CHANGELOG.md; the same candidate plus a CHANGELOG.md change yields none; a non-change reading or a reading without the default yields none. The first test fails when the new rule is removed."#,
        r#"Run `cargo test -p athena-core` : a test with a docs-only design reading built through ensure_changelog (the path both failed jobs took) is refused by admit before verification, showing the check is for the class and not one job."#,
        r#"Existing tests that run admit or the run pipeline with a change reading still pass (cargo test) after fixtures are updated where they omitted CHANGELOG.md."#,
        r#"cargo fmt --check, cargo clippy --all-targets and cargo test pass."#,
        r#"CHANGELOG.md has an entry for this change naming the cause and the new admission rule."#,
    ];

    /// A rebuilt request in the shape of the status-report omission of sza7, where each bullet shares the
    /// framing `The status list shows each job's ...`.
    const SIBLING_REQUEST: &str = "Make the status list richer.

Acceptance:
- The status list shows each job's state
- The status list shows each job's owner
- The status list shows each job's duration in seconds
";

    fn reading(criteria: &[&str]) -> Reading {
        let forecast = || Forecast {
            p_reading_accepted: 0.8,
            p_verification_passes: 0.7,
            p_user_intervenes: 0.1,
            cost_usd: Quantiles::new(0.2, 0.5, 1.5).unwrap(),
            wall_time_s: Quantiles::new(100.0, 300.0, 900.0).unwrap(),
        };
        Reading {
            job: JobId::parse("20261031-000000-aaaa").unwrap(),
            n: 1,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: 1,
            base_commit: "a".repeat(40),
            config_sha256: "b".repeat(64),
            defaults_sha256: "c".repeat(64),
            defaults_version: 1,
            title: "Coverage case".into(),
            summary: "Update the guide.".into(),
            class_of_work: "docs".into(),
            capability: Some("change".into()),
            decline_reason: None,
            producer_model: "producer".into(),
            verifier_model: "verifier".into(),
            defaults_applied: vec![],
            assumptions: vec![],
            question: None,
            acceptance_criteria: criteria
                .iter()
                .map(|text| Criterion {
                    text: text.to_string(),
                    kind: CriterionKind::Content,
                })
                .collect(),
            deliverables: vec![],
            out_of_scope: vec![],
            forecasts: Forecasts {
                reader: forecast(),
                reference: forecast(),
            },
            departures: None,
            urgent_reason: None,
            hard_reason: None,
            producer_effort: None,
        }
    }

    fn refusal(request: &str, criteria: &[&str]) -> String {
        check_reading_coverage(&reading(criteria), request)
            .unwrap_err()
            .to_string()
    }

    #[test]
    fn coverage_names_the_omitted_bullet_and_accepts_full_cover() {
        // The recorded reading of sza7 moved part (3), the status report, to a follow-up.
        let error = refusal(SZA7_REQUEST, &SZA7_CRITERIA);
        assert!(error.contains(SZA7_STATUS_BULLET), "{error}");
        assert!(error.contains("1 acceptance bullet"), "{error}");
        assert!(error.contains("never by dropping a bullet"), "{error}");
        // Only that bullet is named: the other three are covered by the recorded criteria.
        assert_eq!(coverage_gaps(SZA7_REQUEST, &SZA7_CRITERIA).len(), 1);

        let mut full = SZA7_CRITERIA.to_vec();
        full.push(SZA7_STATUS_CRITERION);
        check_reading_coverage(&reading(&full), SZA7_REQUEST).unwrap();
    }

    #[test]
    fn coverage_sibling_bullet_is_not_covered_by_shared_framing() {
        let state = "The status list shows each job's state";
        let owner = "The status list shows each job's owner";
        let duration = "The status list shows each job's duration in seconds";
        let error = refusal(SIBLING_REQUEST, &[state, owner]);
        assert!(error.contains(duration), "{error}");
        assert!(!error.contains("job's owner\""), "{error}");
        // Without the claim rule the state criterion would hold 3 of the 5 stems of the duration bullet.
        assert_eq!(coverage_gaps(SIBLING_REQUEST, &[state, owner]).len(), 1);
        check_reading_coverage(&reading(&[state, owner, duration]), SIBLING_REQUEST).unwrap();
    }

    #[test]
    fn coverage_accepts_the_recorded_ttnw_and_bp5e_readings() {
        // Both recorded readings cover every bullet in their own words; neither is refused.
        for (request, criteria) in [
            (TTNW_REQUEST, TTNW_CRITERIA.to_vec()),
            (BP5E_REQUEST, BP5E_CRITERIA.to_vec()),
        ] {
            check_reading_coverage(&reading(&criteria), request).unwrap();
            assert!(coverage_gaps(request, &criteria).is_empty());
        }
        // The bullets the plan names, scored against the recorded criterion that covers each.
        let ttnw = coverage_scores(TTNW_REQUEST, &TTNW_CRITERIA);
        assert_eq!(ttnw.len(), 5);
        assert!(
            ttnw.iter().any(|(b, _, _)| b
                == "a test: with no improvement requests queued, ordering is unchanged")
        );
        let bp5e = coverage_scores(BP5E_REQUEST, &BP5E_CRITERIA);
        assert_eq!(bp5e.len(), 3);
        assert!(bp5e.iter().any(|(b, _, _)| b
            == "The root cause is named in the change description and fixed where it originates, not at one instance"));
    }

    #[test]
    fn coverage_threshold_is_tuned_on_the_recorded_shapes() {
        let percent = |best: u64, total: u64| best * 100 / total;
        let mut covered = Vec::new();
        for (request, criteria) in [
            (SZA7_REQUEST, SZA7_CRITERIA.to_vec()),
            (TTNW_REQUEST, TTNW_CRITERIA.to_vec()),
            (BP5E_REQUEST, BP5E_CRITERIA.to_vec()),
        ] {
            for (bullet, best, total) in coverage_scores(request, &criteria) {
                println!(
                    "coverage score {:>3}% ({best}/{total}) {bullet}",
                    percent(best, total)
                );
                if bullet != SZA7_STATUS_BULLET {
                    covered.push(percent(best, total));
                }
            }
        }
        let omitted = coverage_scores(SZA7_REQUEST, &SZA7_CRITERIA)
            .into_iter()
            .find(|(b, _, _)| b == SZA7_STATUS_BULLET)
            .map(|(_, best, total)| percent(best, total))
            .unwrap();
        let lowest_covered = *covered.iter().min().unwrap();
        println!(
            "coverage threshold {COVERAGE_MIN_PERCENT}%: omitted sza7 bullet {omitted}%, lowest covered bullet {lowest_covered}%"
        );
        assert!(omitted < COVERAGE_MIN_PERCENT, "{omitted}");
        assert!(lowest_covered >= COVERAGE_MIN_PERCENT, "{lowest_covered}");
    }

    #[test]
    fn coverage_paraphrased_rare_word_is_pinned() {
        let state = "The status list shows each job's state";
        let owner = "The status list shows each job's owner";
        // A reworded rare word ('duration' as 'elapsed time') costs one stem and is not zeroed.
        let elapsed = "Each row of the status list shows the elapsed time of the job";
        check_reading_coverage(&reading(&[state, owner, elapsed]), SIBLING_REQUEST).unwrap();
        // Known limit: a criterion of framing words only covers every bullet that shares them.
        let framing = "The status list shows jobs";
        check_reading_coverage(&reading(&[state, owner, framing]), SIBLING_REQUEST).unwrap();
        // Known limit: one merged criterion for the state and the duration is the state bullet's, so the
        // duration bullet is refused and the reading words it on its own.
        let merged = "The status list shows job state and elapsed time";
        let error = refusal(SIBLING_REQUEST, &[merged, owner]);
        assert!(error.contains("duration in seconds"), "{error}");
    }

    #[test]
    fn coverage_judges_words_the_prose_repeats() {
        let request = "Print the measured duration. The message must name the measured duration with the limit, and the limit is the one in the message.

Acceptance:
- The message names the measured duration and the limit
- athena.toml sets 1800 seconds
";
        let toml = "athena.toml sets the value to 1800 seconds";
        // Every distinguishing word of the first bullet (message, name, measured, duration) is repeated in the
        // prose; none is dropped, so the bullet is refused while no criterion holds them.
        let error = refusal(request, &[toml]);
        assert!(
            error.contains("The message names the measured duration"),
            "{error}"
        );
        assert!(!error.contains("seconds\" "), "{error}");
        let message = "The stop message names the measured duration against the limit";
        check_reading_coverage(&reading(&[toml, message]), request).unwrap();
    }

    #[test]
    fn coverage_a_split_is_allowed_only_when_affirmed() {
        let allowed = [
            "You may split this job.",
            "Feel free to split this job into follow-ups.",
            "It is ok to split the request into separate jobs.",
            "Please split this job if it is too big.",
            "You can split the work.",
        ];
        let denied = [
            "Do not split this job.",
            "Never split it into jobs.",
            "Don't split the job.",
            "Don\u{2019}t split the job.",
            "You can't split this job.",
            "Finish it without splitting.",
            "Should we split this job?",
            "Split the log line into fields.",
            "Then split it on commas.",
            "Split the output into two columns.",
            "",
        ];
        let criteria = [SZA7_CRITERIA[0], SZA7_CRITERIA[1], SZA7_CRITERIA[2]];
        for line in allowed {
            let request = format!("{line}\n\n{SIBLING_REQUEST}");
            assert!(request_allows_split(&request), "{line}");
            check_reading_coverage(&reading(&criteria), &request).unwrap();
        }
        for line in denied {
            let request = format!("{line}\n\n{SIBLING_REQUEST}");
            assert!(!request_allows_split(&request), "{line}");
            let error = check_reading_coverage(&reading(&[]), &request)
                .unwrap_err()
                .to_string();
            assert!(error.contains("duration in seconds"), "{line}: {error}");
        }
        // A bullet that mentions a split is not the request's own permission.
        let bullet =
            "Acceptance:\n- You may split this job\n- The status list shows each job's state";
        assert!(!request_allows_split(bullet));
    }

    #[test]
    fn coverage_a_reading_that_asks_declines_or_has_no_heading_is_not_checked() {
        let mut asking = reading(&[]);
        asking.question = Some("May I split this job?".into());
        check_reading_coverage(&asking, SIBLING_REQUEST).unwrap();
        let mut declining = reading(&[]);
        declining.capability = None;
        declining.decline_reason = Some("no capability fits".into());
        check_reading_coverage(&declining, SIBLING_REQUEST).unwrap();
        // Bullets without an Acceptance heading are a plain list, not acceptance.
        let plain = "Do these:\n- The status list shows each job's state\n- The status list shows each job's owner";
        check_reading_coverage(&reading(&[]), plain).unwrap();
        // A request with an Acceptance heading and no bullet under it asks for nothing.
        check_reading_coverage(&reading(&[]), "Do it.\nAcceptance:\nIt works.").unwrap();
    }

    #[test]
    fn coverage_reads_the_bullet_forms_and_the_heading_forms() {
        let bullets = |request: &str| request_acceptance_bullets(request);
        assert_eq!(
            bullets("Acceptance:\n- first thing\n* second thing\n3. third thing\n"),
            ["first thing", "second thing", "third thing"]
        );
        assert_eq!(bullets("## Acceptance criteria:\n- one"), ["one"]);
        assert_eq!(bullets("**Acceptance:**\n- one"), ["one"]);
        assert_eq!(bullets("ACCEPTANCE:\n- one"), ["one"]);
        assert!(bullets("- one\n- two").is_empty());
        // A heading, or a non-bullet line after a blank line, ends the section.
        assert_eq!(
            bullets("Acceptance:\n- one\n\nClosing words.\n- two"),
            ["one"]
        );
        assert_eq!(bullets("Acceptance:\n- one\nFiles: a.rs\n- two"), ["one"]);
        assert_eq!(bullets("Acceptance:\n- one\nOther things:\n- two"), ["one"]);
        // A bullet after a blank line is still in the section.
        assert_eq!(bullets("Acceptance:\n- one\n\n- two"), ["one", "two"]);
    }

    #[test]
    fn coverage_wrapped_bullet_is_one_bullet_and_later_bullets_are_checked() {
        let request = "Acceptance:
- A test with a fake check runner that exceeds
  its time limit shows the job ends as an environment stop
- the status list shows each job's state and
duration in seconds
- athena.toml has check_timeout_s = 1800
Depends on 20261105-180334-rj4j
";
        let bullets = request_acceptance_bullets(request);
        assert_eq!(
            bullets,
            [
                "A test with a fake check runner that exceeds its time limit shows the job ends as an environment stop",
                "the status list shows each job's state and duration in seconds",
                "athena.toml has check_timeout_s = 1800",
            ]
        );
        // The wrapped bullet is scored on its whole text, and the bullet after it is still checked.
        let wrapped = "A test with a fake check runner whose time limit passes ends the job as an environment stop";
        let error = refusal(request, &[wrapped]);
        assert!(
            error.contains("the status list shows each job's state and duration in seconds"),
            "{error}"
        );
        assert!(
            error.contains("athena.toml has check_timeout_s = 1800"),
            "{error}"
        );
        assert!(
            !error.contains("A test with a fake check runner that exceeds"),
            "{error}"
        );
        let all = [
            wrapped,
            "The status list shows each job's state and duration in seconds",
            "athena.toml has check_timeout_s = 1800",
        ];
        check_reading_coverage(&reading(&all), request).unwrap();
    }

    #[test]
    fn coverage_scores_tags_stop_words_and_generated_pointer_bullets() {
        // A leading [tag] is not part of the bullet's words.
        assert_eq!(
            coverage_stems("[behavior] The status list shows jobs"),
            coverage_stems("The status list shows jobs")
        );
        // A bullet of stop words only has nothing to cover.
        let request =
            "Acceptance:\n- It is all of the one\n- The status list shows each job's state";
        let error = refusal(request, &["Something else"]);
        assert!(!error.contains("It is all"), "{error}");
        assert!(
            error.contains("The status list shows each job's state"),
            "{error}"
        );
        // The steward's pointer bullet repeats a title no criterion repeats, so it is not scored.
        let steward = "Acceptance:\n- The change is made as described: Four jobs in a row failed to fix a flaky test\n- fmt, clippy and test pass";
        check_reading_coverage(
            &reading(&["Run cargo fmt and clippy; the tests pass."]),
            steward,
        )
        .unwrap();
    }

    #[test]
    fn coverage_stems_agree_across_word_forms() {
        for group in [
            ["flag", "flags", "flagged", "flagging"],
            ["queue", "queues", "queued", "queue"],
            ["order", "orders", "ordered", "ordering"],
            ["name", "names", "named", "naming"],
            ["fix", "fixes", "fixed", "fixing"],
        ] {
            let stems: BTreeSet<String> = group.iter().map(|w| stem(w)).collect();
            assert_eq!(stems.len(), 1, "{group:?} -> {stems:?}");
        }
        assert_ne!(stem("unchanged"), stem("identical"));
        assert_eq!(stem("pass"), "pass");
        assert_eq!(stem("ss"), "ss");
    }
}
