//! Evidence, survey, history-fixture, environment and design checks on a reading.

use super::*;

/// Phrases that make the producer's own account the evidence for a criterion. Each is a whole-word
/// phrase, lowercase, with a straight apostrophe.
pub(super) const REPORT_NOUNS: [&str; 5] = ["report", "summary", "claims", "statement", "account"];
pub(super) const PRODUCER_VERBS: [&str; 12] = [
    "states",
    "confirms",
    "says",
    "lists",
    "reports",
    "claims",
    "attests",
    "declares",
    "notes",
    "describes",
    "asserts",
    "writes",
];
pub(super) const REPORT_VERBS: [&str; 13] = [
    "lists",
    "states",
    "confirms",
    "says",
    "shows",
    "names",
    "records",
    "claims",
    "mentions",
    "notes",
    "describes",
    "cites",
    "includes",
];

/// Whether `text` contains `phrase` as whole words.
pub(super) fn has_phrase(text: &str, phrase: &str) -> bool {
    text.match_indices(phrase).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + phrase.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

/// The phrase that makes a criterion rest on the producer's own account, if it has one. Matching is on
/// lowercase text with curly apostrophes made straight and runs of white space made single spaces.
pub(super) fn producer_evidence_phrase(criterion: &str) -> Option<String> {
    let text = criterion
        .to_lowercase()
        .replace(['\u{2019}', '\u{2018}'], "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let mut phrases: Vec<String> = Vec::new();
    for noun in REPORT_NOUNS {
        phrases.push(format!("producer's {noun}"));
        phrases.push(format!("producers' {noun}"));
        phrases.push(format!("producer {noun}"));
        phrases.push(format!("{noun} from the producer"));
        phrases.push(format!("{noun} of the producer"));
        phrases.push(format!("{noun} by the producer"));
    }
    for verb in PRODUCER_VERBS {
        phrases.push(format!("the producer {verb}"));
        phrases.push(format!("producer {verb}"));
    }
    for verb in REPORT_VERBS {
        phrases.push(format!("the report {verb}"));
    }
    phrases.push("according to the producer".into());
    phrases.push("reported by the producer".into());
    phrases.into_iter().find(|p| has_phrase(&text, p))
}

/// Refuses a reading whose acceptance criterion rests on the producer's own report, summary, claims or
/// statement as its evidence.
///
/// Root cause this check removes: criteria were written without a defined list of what the verifier can
/// observe (the candidate diff, the repository snapshot, the recorded check output, and the commands it is
/// allowed to run), and nothing compared the evidence a criterion requires with that list. A criterion such
/// as "the producer's report lists the caller-check Grep" can only be satisfied by something the verifier
/// never sees, so it was accepted and then could not be judged. Jobs 20261027-004517-gk9e,
/// 20261027-011733-vrsk, 20261027-012938-yckz, 20261026-232402-6qme and 20261026-232423-hvyb each failed that
/// way. The list is now the reader's brief ("What the verifier can run") and this check enforces it where
/// criteria are first fixed, for every reading with a capability, first attempt or retry.
///
/// The error names each refused criterion by number and text and says to restate it as a command or Grep the
/// verifier can run, or a file in the snapshot.
pub fn check_criteria_evidence(reading: &Reading) -> Result<()> {
    if reading.capability.is_none() {
        return Ok(());
    }
    let refused: Vec<String> = reading
        .acceptance_criteria
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            producer_evidence_phrase(&c.text)
                .map(|phrase| format!("criterion {} ({:?}) cites {phrase:?}", i + 1, c.text))
        })
        .collect();
    if refused.is_empty() {
        return Ok(());
    }
    bail!(
        "the reading was refused because {}; the verifier sees the candidate diff, the repository snapshot and \
         the recorded check output, never the producer's report, summary, claims or statement, so a criterion \
         that rests on them cannot be judged; restate each as a command or Grep the verifier can run, or a \
         file in the snapshot",
        refused.join("; and ")
    )
}

/// The `Code survey:` assumptions of a reading: those whose trimmed text starts with that label, in any case.
pub(super) fn code_surveys(reading: &Reading) -> Vec<&str> {
    reading
        .assumptions
        .iter()
        .map(|a| a.trim())
        .filter(|a| a.to_lowercase().starts_with("code survey:"))
        .collect()
}

/// Whether `text` names a path under `crates/`: `crates/` followed by a letter or digit.
pub(super) fn names_crates_path(text: &str) -> bool {
    text.match_indices("crates/").any(|(at, marker)| {
        text[at + marker.len()..]
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric)
    })
}

/// Whether the deliverable adds or changes a record or data file under `crates/`: it is created there with a
/// data extension (not a `Cargo.toml`) or under a `records` or `data` directory, or it is a `records.rs`.
/// Only the deliverable list is read, never a criterion, so a criterion that says `records/` is not changed
/// triggers nothing.
pub(super) fn is_record_or_data_deliverable(d: &Deliverable) -> bool {
    if !d.path.starts_with("crates/") {
        return false;
    }
    let mut parts = d.path.split('/');
    let file = parts.next_back().unwrap_or("");
    if file == "records.rs" {
        return true;
    }
    if d.action != DeliverableAction::Create || file == "Cargo.toml" {
        return false;
    }
    let data_extension = file.rsplit_once('.').is_some_and(|(_, ext)| {
        ["json", "jsonl", "ndjson", "toml", "yaml", "yml", "csv"].contains(&ext)
    });
    data_extension
        || d.path
            .split('/')
            .any(|part| part == "records" || part == "data")
}

/// Whether a survey names evidence that a new record or data file is accepted: `check_record_file`,
/// `AnyRecord`, or a round-trip test (an identifier with `round_trip` or `roundtrip`, or a path with
/// `round-trip`). A bare word such as "allow-list" or "round-trip" in prose is not evidence.
pub(super) fn names_record_evidence(survey: &str) -> bool {
    survey
        .split(|c: char| !(c.is_alphanumeric() || matches!(c, '_' | '-' | '/')))
        .any(|token| {
            token == "check_record_file" || token == "AnyRecord" || is_round_trip_token(token)
        })
}

/// Whether one token (split as in [`names_record_evidence`]) names a round-trip test: an identifier with
/// `round_trip` or `roundtrip`, or a path with `round-trip`.
pub(super) fn is_round_trip_token(token: &str) -> bool {
    let lower = token.to_lowercase();
    lower.contains("round_trip")
        || lower.contains("roundtrip")
        || (lower.contains("round-trip") && token.contains('/'))
}

/// Refuses a reading that changes code or tests with a behavior criterion and carries no code survey: an
/// assumption that starts `Code survey:` and names a `crates/` path. A reading that also adds a record or data
/// file under `crates/` (or changes a `records.rs`) must have a survey that names `check_record_file`,
/// `AnyRecord`, or a round-trip test.
///
/// Root cause this check removes: the reading stage designs storage, override and test seams from a general
/// idea of the feature, without surveying what the records validation accepts or where resolution and probing
/// happen in the code. Jobs 20261028-055601-hg0n and 20261028-082151-67fa each began from a reading that
/// fixed its seams that way, and introspections 20261028-055651-dhh0 and 20261028-082240-77as traced both to
/// the missing survey. Reader briefs now ask for the survey (`CODE_SURVEY_GUIDE` in athena-context), and this
/// check enforces it where readings are accepted. It costs no model call, and it reads the assumptions and
/// the deliverable list, never the repository.
///
/// It applies when the reading has a capability and no question, its class of work is `code` or `tests`, and
/// at least one criterion is a behavior criterion. A docs reading, a decline, a question and a reading with
/// only content criteria need no survey.
///
/// This check refuses more readings; the higher refusal rate is intended. A replay of accepted readings
/// (`replay`) holds a job that newly refuses more than 10% unless the job's criteria declare a share, as
/// "expected N% of newly refused readings", and give a reason when it is above 25%.
///
/// The error names the missing survey or the missing evidence and tells the reader what to write.
pub fn check_code_survey(reading: &Reading) -> Result<()> {
    if !changes_code_with_behavior_criterion(reading) {
        return Ok(());
    }
    let surveys: Vec<&str> = code_surveys(reading)
        .into_iter()
        .filter(|s| names_crates_path(s))
        .collect();
    if surveys.is_empty() {
        bail!(
            "the reading was refused because it changes {} with a behavior criterion but has no `Code survey:` \
             assumption that names a crates/ path; the criteria were designed from a general idea of the feature \
             and not from the code. Read the code, then add an assumption that starts `Code survey:` and names \
             the crates/ paths you read, with the labels `Entry point:`, `Gate order:`, `Stop gate:`, `Reuses:`, \
             `Output:` and one `Direct check: criterion N` per behavior criterion (`check_behavior_test_design` \
             judges them). Example: {}",
            reading.class_of_work,
            athena_context::briefs::CODE_SURVEY_EXAMPLE
        );
    }
    let new_records: Vec<&str> = reading
        .deliverables
        .iter()
        .filter(|d| is_record_or_data_deliverable(d))
        .map(|d| d.path.as_str())
        .collect();
    if !new_records.is_empty() && !surveys.iter().any(|s| names_record_evidence(s)) {
        bail!(
            "the reading was refused because it adds or changes a record or data file ({}) but its `Code survey:` \
             assumption does not name what accepts it; name `check_record_file`, `AnyRecord` or a round-trip test \
             by its function name or path, as read in the records validation, not the word allow-list alone",
            new_records.join(", ")
        );
    }
    Ok(())
}

/// Whether the reading is a change reading the code checks apply to: it has a capability and no question, its
/// class of work is `code` or `tests`, and at least one criterion is a behavior criterion. One definition for
/// [`check_code_survey`] and [`check_history_fixture`], so the two gates cannot drift.
pub(super) fn changes_code_with_behavior_criterion(reading: &Reading) -> bool {
    reading.capability.is_some()
        && reading.question.is_none()
        && matches!(reading.class_of_work.as_str(), "code" | "tests")
        && reading
            .acceptance_criteria
            .iter()
            .any(|c| c.kind == CriterionKind::Behavior)
}

/// Phrases and function names that mark a criterion as a rule on how commits are classified, or on what the
/// history or attribution check accepts. Matched with [`has_phrase`] on lowercase text, so a hyphen or an
/// underscore next to a phrase counts as a word boundary (`trailer-x` and `trailer_x` match, `trailing` does
/// not). No file path is in the list and neither is the bare word `landing`: a change to the landing queue, to
/// the timing of a landing or to the CI wait names neither a commit shape nor a classification.
pub(super) const HISTORY_RULE_PHRASES: [&str; 15] = [
    "commit shape",
    "commit shapes",
    "commit-shape",
    "pr_commits",
    "classify_commit",
    "classify_pre_merge",
    "history check",
    "attribution check",
    "records-only commit",
    "records only commit",
    "trailer",
    "trailers",
    "--history",
    "--attribution",
    "--pr-head",
];

/// The shapes a criterion must name together to count as a realistic-branch fixture, each with the spellings
/// accepted. A criterion names the shape when it holds any one spelling as whole words.
pub(super) const HISTORY_FIXTURE_SHAPES: [&[&str]; 5] = [
    &["realistic"],
    &["fixture", "fixtures"],
    &["intervention commit", "intervention commits"],
    &[
        "records-only commit",
        "records-only commits",
        "records only commit",
        "records only commits",
    ],
    &["job landing", "job landings", "job's landing"],
];

/// Lowercase text with curly apostrophes made straight and runs of white space made single spaces.
pub(super) fn straight_lowercase(text: &str) -> String {
    text.to_lowercase()
        .replace(['\u{2019}', '\u{2018}'], "'")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Refuses a code or tests reading with a behavior criterion that states a rule on commit classification, the
/// history check or the attribution check, unless a behavior criterion also names a realistic-branch fixture
/// that holds an intervention commit, a records-only commit and the job landing.
///
/// Root cause this check removes: landing, history and attribution rules are derived from the happy-path
/// commit shape, the job's own commits, and not from every shape the real branch holds. The only guard was
/// prose in the planner and critic prompts, so a reading could fix criteria that never required a branch with
/// more than the job's commits. Jobs 20261024-153908-1rve, 20261024-154552-e65v and 20261025-012657-hac3 each
/// began that way: a classification or trailer rule was demonstrated on job commits only, and the missing
/// shapes (an intervention commit, a records-only commit, the retry-after-block landing) surfaced as
/// findings in verification or after landing. Introspections 20261024-154443-51wt, 20261024-161503-q0pp and
/// 20261025-012736-2q6w traced all three to it. The fix is not another change to `pr_commits` or to the trailer
/// cutover, which are the instances: those would leave the next landing rule free to be tested on job commits
/// only. It sits where readings are accepted, the one seam every reading passes, so the property is enforced
/// by mechanism and not by prose.
///
/// It reads the [`Reading`] only: no model call, no repository, no records and no network. It applies under
/// the gate of [`check_code_survey`] (a capability, no question, class `code` or `tests`, a behavior
/// criterion). It then looks for a trigger in the text of any criterion: one of the [`HISTORY_RULE_PHRASES`]
/// (`commit shape`, `pr_commits`, `classify_commit`, `history check`, `trailer`, `--history` and the like),
/// matched as whole words on lowercase text with `has_phrase`. File paths do not trigger it, so a job on
/// `land.rs` about the queue or the CI baseline is not refused, and neither is a survey assumption that only
/// names a function the change reads. The reading is accepted when one behavior criterion names `realistic`,
/// `fixture`, an `intervention commit`, a `records-only commit` and the `job landing`, in any of the
/// spellings of [`HISTORY_FIXTURE_SHAPES`].
///
/// Limits: the match is lexical, with no reading of negation, so a criterion that says a trailer is not
/// written still triggers it. The check enforces those three shapes. The retry-after-block landing and any
/// other shape the branch produces is not enforced; the planner's commit-shape table and the critic remain
/// the guard for them, and the refusal text asks the reader to name them. There is no reader rule line for
/// this check: this reading decided not to grow reader-facing text, and the rule list lives in `briefs.rs`,
/// which it does not change. The reader gets the refusal text, with an example criterion, on its immediate
/// second read.
///
/// This check refuses more readings; the higher refusal rate is intended. A replay of accepted readings
/// (`replay`) holds a job that newly refuses more than 10% unless the job's criteria declare a share. Declared
/// here: expected 4% of newly refused readings. That is an estimate, not measured: about 8 of the last 200
/// commits changed classification, history or attribution behavior, and the replay at verification measures
/// the real share.
///
/// The error names the matched phrase and tells the reader what to add.
pub fn check_history_fixture(reading: &Reading) -> Result<()> {
    if !changes_code_with_behavior_criterion(reading) {
        return Ok(());
    }
    let texts: Vec<(CriterionKind, String)> = reading
        .acceptance_criteria
        .iter()
        .map(|c| (c.kind, straight_lowercase(&c.text)))
        .collect();
    let Some(matched) = texts.iter().find_map(|(_, text)| {
        HISTORY_RULE_PHRASES
            .iter()
            .find(|phrase| has_phrase(text, phrase))
    }) else {
        return Ok(());
    };
    let names_fixture = texts.iter().any(|(kind, text)| {
        *kind == CriterionKind::Behavior
            && HISTORY_FIXTURE_SHAPES
                .iter()
                .all(|spellings| spellings.iter().any(|s| has_phrase(text, s)))
    });
    if names_fixture {
        return Ok(());
    }
    bail!(
        "the reading was refused because a criterion states a rule on commit classification, the history \
         check or the attribution check (it names \"{matched}\") but no behavior criterion names a \
         realistic-branch fixture; a rule tested only on the job's own commits misses the other commit shapes \
         the real branch holds. Add a behavior criterion that runs a test on a realistic-branch fixture with \
         at least one intervention commit, one records-only commit and the job landing, for example: \"Run \
         `cargo test -p <crate> <filter>`: on a realistic-branch fixture with an intervention commit, a \
         records-only commit and the job landing, the rule classifies each as intended\". Also name every \
         other shape the branch produces that the rule touches, such as a retry-after-block landing"
    )
}

/// Phrases that name a measure Athena computes from its own records. Matched with [`has_phrase`] on lowercase
/// text in which a hyphen or an underscore reads as a space, so `landing-time` and `check_durations` match.
/// Bare `time spent` and bare `throughput` are not in the list: they are ordinary words (`time spent waiting
/// for CI`, drain throughput), so the measure is named by `unlanded time spent`, `time spent per job` or the
/// records phrases of `throughput` below.
pub(super) const RECORDS_MEASURE_PHRASES: &[&str] = &[
    "keep rule",
    "planning cost",
    "planning costs",
    "refusal rate",
    "refusal rates",
    "refusal count",
    "refusal counts",
    "refusal share",
    "check duration",
    "check durations",
    "landing time",
    "landing times",
    "landed time",
    "wasted work",
    "unlanded time spent",
    "time spent per job",
    "throughput section",
    "throughput review",
    "throughput figure",
    "throughput figures",
    "throughput measure",
    "throughput measures",
    "throughput status",
];

/// The identifier of a measure in `status/throughput.rs`. Matched on text in which only a hyphen reads as a
/// space, so the underscore keeps it apart from the English phrase `time spent`.
pub(super) const RECORDS_MEASURE_IDENTIFIERS: &[&str] = &["time_spent"];

/// Modules that compute the figures `status` prints from the records: throughput and wasted work in the
/// first, planning cost and the keep rule in the second. `records/throughput.rs` in `athena-data` is not
/// here: it holds record types and computes nothing.
pub(super) const RECORDS_MEASURE_PATHS: &[&str] = &[
    "crates/athena-core/src/status/throughput.rs",
    "crates/athena-core/src/status/planning.rs",
];

/// What a criterion must name to run a records measure against the real records.
pub(super) const REAL_RECORDS_PHRASES: &[&str] = &["real records"];

/// Sources of an expected range that do not depend on the code under change. `status --snapshot` is not one:
/// it prints the figure the change computes. `git log` is also a located source for [`check_environment_reach`],
/// which then asks for an `Environment:` assumption; `history.json` and the others are not.
pub(super) const INDEPENDENT_SOURCE_PHRASES: &[&str] = &[
    "git log",
    "history.json",
    "commit times",
    "recorded check durations",
    "recorded durations",
    "landing record timestamps",
];

/// What a criterion must name so the real output is compared with something other than itself.
pub(super) const EXPECTED_RANGE_PHRASES: &[&str] = &["expected range"];

/// The sentence a refusal-rule criterion carries by design (`CODE_SURVEY_GUIDE` asks for it). It is removed
/// before the measure phrases are searched, so declaring a higher refusal rate triggers nothing.
const RECORDS_MEASURE_DECLARATION: &str = "higher refusal rate is intended";

/// The two lowercase forms of `text` that [`check_records_measure`] searches: hyphens and underscores read as
/// spaces (for the phrase list, with the declared-intended sentence removed), and only hyphens read as spaces
/// (for the identifiers).
fn records_measure_forms(text: &str) -> (String, String) {
    fn squash(text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }
    let base = straight_lowercase(text);
    let words = squash(&base.replace(['-', '_'], " ")).replace(RECORDS_MEASURE_DECLARATION, " ");
    (words, squash(&base.replace('-', " ")))
}

/// What names a records-computed measure in the reading, if anything: a phrase or identifier in the summary or
/// a criterion (a criterion that declares `newly refused` readings is not searched), or a created or modified
/// deliverable that computes the figures.
fn records_measure_trigger(reading: &Reading) -> Option<String> {
    let mut texts: Vec<&str> = vec![reading.summary.as_str()];
    texts.extend(
        reading
            .acceptance_criteria
            .iter()
            .filter(|c| !straight_lowercase(&c.text).contains("newly refused"))
            .map(|c| c.text.as_str()),
    );
    for text in texts {
        let (words, identifiers) = records_measure_forms(text);
        if let Some(phrase) = RECORDS_MEASURE_PHRASES
            .iter()
            .find(|phrase| has_phrase(&words, phrase))
        {
            return Some(format!("it names \"{phrase}\""));
        }
        if let Some(identifier) = RECORDS_MEASURE_IDENTIFIERS
            .iter()
            .find(|identifier| has_phrase(&identifiers, identifier))
        {
            return Some(format!("it names \"{identifier}\""));
        }
    }
    reading
        .deliverables
        .iter()
        .find(|d| {
            d.action != DeliverableAction::Delete
                && RECORDS_MEASURE_PATHS.contains(&d.path.as_str())
        })
        .map(|d| {
            format!(
                "it changes {}, which computes the figures status prints",
                d.path
            )
        })
}

/// Refuses a code or tests reading that changes a measure computed from Athena's records unless a behavior
/// criterion runs it against the real records and takes the expected range from an independent source.
///
/// Root cause this check removes: measures and reports computed from the records (throughput, the keep rule,
/// planning cost, refusal rates, check durations, landing time) were verified on fixture records alone, and
/// the fixtures held the shapes the author expected, so a wrong figure showed only on the real records. On
/// 2026-10-29 the landing time shown by `status` was 0.1 minutes, then, after the first fix, 273.8 minutes,
/// both against real landings of a few minutes. Nothing required a criterion that runs the measure on the
/// repository's real records and compares the output with a range from a source the change does not compute,
/// so both fixes passed their fixtures. The rule sits where readings are accepted, so a later job cannot
/// skip it.
///
/// It reads the [`Reading`] only: no model call, no repository, no records and no network. It applies under
/// the gate of [`check_code_survey`] (a capability, no question, class `code` or `tests`, a behavior
/// criterion). It fires on either of two triggers. (a) Words: the summary or a criterion names a measure
/// (see [`RECORDS_MEASURE_PHRASES`]: `keep rule`, `planning cost`, `refusal rate`, `refusal count`, `check
/// duration`, `landing time`, `wasted work`, `throughput` in a records phrase such as `throughput section` or
/// `throughput figures`, and the identifier `time_spent`), as whole words on lowercase text in which a hyphen
/// or an underscore reads as a space. A criterion that declares `newly refused` readings is not searched, and
/// the sentence `higher refusal rate is intended` is removed first, so a refusal rule's own declaration
/// triggers nothing. (b) Paths: a created or modified deliverable is `status/throughput.rs` or
/// `status/planning.rs` in `athena-core`, the two modules that compute the figures. A code-health split or
/// move of those modules needs the criterion too, because a moved figure can change.
///
/// The reading is accepted when one behavior criterion names `real records`, an independent source (`git log`,
/// `history.json`, `commit times`, `recorded check durations`, `recorded durations` or `landing record
/// timestamps`) and an `expected range`. `status --snapshot` prints the code under change, so it is the real
/// output to compare, never the source of the range. `git log` is a located source for
/// [`check_environment_reach`] and so needs an `Environment:` assumption; `history.json` and `commit times`
/// need none.
///
/// Limits: the check is lexical, with no reading of negation, so a reading that names a measure only to say
/// it is unchanged is still refused, and so is a split or move of the two figure modules with no behavior
/// change. It checks that the criterion exists; the verifier runs it. A criterion can hold the three parts
/// without running anything on real records. Refusal counts and other figures in shared code such as
/// `status.rs` are caught by their words only, not by path. `records/throughput.rs` in `athena-data` holds
/// record types and is not a trigger.
///
/// This check refuses more readings; the higher refusal rate is intended. A replay of accepted readings
/// (`replay`) holds a job that newly refuses more than 10% unless the job's criteria declare a share. Declared
/// here: expected 3% of newly refused readings. That is an estimate, not measured: about 7 of the last 200
/// commit subjects name these measures, plus this job's own reading, and the replay at verification measures
/// the real share. Stored readings of queued requests that lack the criterion leave the drain queue until
/// `athena go` or a new read, as for any failing check.
///
/// The error names the trigger and the missing parts and gives an example criterion that takes its range
/// from `history.json` commit times.
pub fn check_records_measure(reading: &Reading) -> Result<()> {
    if !changes_code_with_behavior_criterion(reading) {
        return Ok(());
    }
    let Some(trigger) = records_measure_trigger(reading) else {
        return Ok(());
    };
    // For each behavior criterion, which of the three parts it holds: real records, an independent source
    // and an expected range.
    let parts: Vec<[bool; 3]> = reading
        .acceptance_criteria
        .iter()
        .filter(|c| c.kind == CriterionKind::Behavior)
        .map(|c| {
            let (words, _) = records_measure_forms(&c.text);
            let plain = straight_lowercase(&c.text);
            let holds = |list: &[&str]| {
                list.iter()
                    .any(|p| has_phrase(&words, p) || has_phrase(&plain, p))
            };
            [
                holds(REAL_RECORDS_PHRASES),
                holds(INDEPENDENT_SOURCE_PHRASES),
                holds(EXPECTED_RANGE_PHRASES),
            ]
        })
        .collect();
    if parts.iter().any(|p| p.iter().all(|held| *held)) {
        return Ok(());
    }
    // Judge the best behavior criterion, so the error names what is still missing from the nearest one.
    let best = parts
        .iter()
        .max_by_key(|p| p.iter().filter(|held| **held).count())
        .copied()
        .unwrap_or([false; 3]);
    let mut missing: Vec<&str> = Vec::new();
    if !best[0] {
        missing.push("`real records`");
    }
    if !best[1] {
        missing.push(
            "an independent source of the range (`git log`, `history.json`, `commit times`, `recorded check \
             durations`, `recorded durations` or `landing record timestamps`)",
        );
    }
    if !best[2] {
        missing.push("an `expected range`");
    }
    bail!(
        "the reading was refused because it changes a measure computed from Athena's records ({trigger}) but no \
         behavior criterion runs it against real records with an expected range from an independent source; \
         missing from the nearest behavior criterion: {}. A measure verified on fixture records alone has \
         shown wrong figures on the real records (2026-10-29: a landing time of 0.1 minutes, then 273.8 \
         minutes, against real landings of a few minutes). Add one behavior criterion that names `real \
         records`, an independent source and the `expected range`, for example: \"Run `cargo run -p \
         athena-cli -- status --snapshot <evidence directory>`: on the real records the landing time it prints \
         is within the expected range taken from the commit times in history.json\". The range must come from \
         a source the change does not compute; the status output is the real output compared with it. `git \
         log` also needs an `Environment:` assumption naming the site, read permission and the records or \
         history it reaches. A split or move of the figure modules with no behavior change needs the same \
         criterion, because a moved figure can change",
        missing.join(", ")
    )
}

/// The `Environment:` assumptions of a reading: those whose trimmed text starts with that label, in any case.
pub(super) fn environment_assumptions(reading: &Reading) -> Vec<&str> {
    reading
        .assumptions
        .iter()
        .map(|a| a.trim())
        .filter(|a| a.to_lowercase().starts_with("environment:"))
        .collect()
}

/// One word of a scanned text: lowercase, with whether it was written exactly `PR`/`PRs` or `CI`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EnvWord {
    lower: String,
    pr: bool,
    ci: bool,
}

/// Commands whose backticked form names a host action (`git push`, `gh pr create`).
pub(super) const ENV_GIT_COMMANDS: [&str; 5] = ["push", "fetch", "clone", "pull", "remote"];
pub(super) const ENV_GH_COMMANDS: [&str; 7] =
    ["pr", "api", "run", "auth", "workflow", "release", "issue"];
/// Word stems of the verbs that act on a pull request.
pub(super) const ENV_PR_VERB_STEMS: [&str; 7] =
    ["open", "creat", "rais", "merg", "reus", "push", "submit"];
/// Verbs of using the GitHub API.
pub(super) const ENV_API_VERBS: [&str; 10] = [
    "call", "calls", "called", "calling", "query", "queries", "queried", "use", "uses", "using",
];
/// Verbs that read a source: the phrase that names the source must follow one within 6 words.
pub(super) const ENV_SOURCE_VERBS: [&str; 12] = [
    "from", "use", "uses", "using", "read", "reads", "counts", "measures", "replays", "queries",
    "parses", "scans",
];

/// Splits `text` into runs of words. A run ends at a break: a backticked span that is not a command, and a
/// bare token with `/`, `_`, `::` or an interior `.` (a path, an identifier or a file name). Apostrophes are
/// removed. `git push|fetch|clone|pull|remote`, `gh pr|api|run|auth|workflow|release|issue` and
/// `athena/records` in backticks, and bare `athena/records` and `api.github.com`, are kept as words.
pub(super) fn scan_segments(text: &str) -> Vec<Vec<EnvWord>> {
    fn word(segments: &mut [Vec<EnvWord>], piece: &str) {
        if let Some(last) = segments.last_mut() {
            last.push(EnvWord {
                lower: piece.to_lowercase(),
                pr: piece == "PR" || piece == "PRs",
                ci: piece == "CI",
            });
        }
    }
    fn split(segments: &mut Vec<Vec<EnvWord>>) {
        if segments.last().is_some_and(|s| !s.is_empty()) {
            segments.push(Vec::new());
        }
    }
    let text = text.replace(['\'', '\u{2019}'], "");
    let mut segments: Vec<Vec<EnvWord>> = vec![Vec::new()];
    for (i, part) in text.split('`').enumerate() {
        if i % 2 == 1 {
            let span = part.trim();
            let mut words = span.split_whitespace();
            let first = words.next().unwrap_or("");
            let second = words.next().unwrap_or("");
            if span.starts_with("athena/records") {
                word(&mut segments, "athena/records");
            } else if (first == "git" && ENV_GIT_COMMANDS.contains(&second))
                || (first == "gh" && ENV_GH_COMMANDS.contains(&second))
            {
                word(&mut segments, first);
                word(&mut segments, second);
            } else {
                split(&mut segments);
            }
            continue;
        }
        for token in part.split_whitespace() {
            let token = token.trim_matches(|c: char| !c.is_alphanumeric());
            if token.is_empty() {
                continue;
            }
            let lower = token.to_lowercase();
            if lower == "athena/records" || lower == "api.github.com" {
                word(&mut segments, &lower);
            } else if token.contains(['/', '_', '.']) || token.contains("::") {
                split(&mut segments);
            } else {
                for piece in token
                    .split(|c: char| !c.is_alphanumeric())
                    .filter(|p| !p.is_empty())
                {
                    word(&mut segments, piece);
                }
            }
        }
    }
    segments
}

/// Whether any word in `seg` from `before` words ahead of `at` to `after` words behind it (not `at` itself)
/// satisfies `test`.
pub(super) fn env_near(
    seg: &[EnvWord],
    at: usize,
    before: usize,
    after: usize,
    test: impl Fn(&str) -> bool,
) -> bool {
    let from = at.saturating_sub(before);
    let to = seg.len().min(at + after + 1);
    (from..to).any(|j| j != at && test(&seg[j].lower))
}

/// Whether one run of words holds a host action: a git or gh command, a push with its target, a fetch, clone or
/// pull from a remote, a pull request or `PR` with an acting verb, a GitHub API call, or a CI write.
pub(super) fn segment_has_host_action(seg: &[EnvWord]) -> bool {
    let at = |i: usize| seg.get(i).map_or("", |w| w.lower.as_str());
    let target = |w: &str| {
        matches!(
            w,
            "origin" | "remote" | "upstream" | "branch" | "branches" | "ref" | "refs"
        )
    };
    let remote = |w: &str| matches!(w, "origin" | "remote" | "upstream");
    let pr_verb = |w: &str| ENV_PR_VERB_STEMS.iter().any(|stem| w.starts_with(stem));
    let api_verb = |w: &str| ENV_API_VERBS.contains(&w) || matches!(w, "fetch" | "fetches");
    let ci_write = |w: &str| matches!(w, "write" | "writes" | "written" | "push" | "pushes");
    let commit = |w: &str| matches!(w, "commit" | "commits");
    (0..seg.len()).any(|i| {
        let w = at(i);
        (w == "git" && ENV_GIT_COMMANDS.contains(&at(i + 1)))
            || (w == "gh" && ENV_GH_COMMANDS.contains(&at(i + 1)))
            || (matches!(w, "push" | "pushes" | "pushed" | "pushing")
                && env_near(seg, i, 4, 4, target))
            || (matches!(
                w,
                "fetch"
                    | "fetches"
                    | "fetched"
                    | "clone"
                    | "clones"
                    | "cloned"
                    | "pull"
                    | "pulls"
                    | "pulled"
            ) && !matches!(at(i + 1), "request" | "requests")
                && env_near(seg, i, 0, 3, remote))
            || (w == "pull"
                && matches!(at(i + 1), "request" | "requests")
                && env_near(seg, i, 3, 4, pr_verb))
            || (seg[i].pr && env_near(seg, i, 4, 4, pr_verb))
            || (matches!((w, at(i + 1)), ("github" | "host", "api"))
                && env_near(seg, i, 3, 4, api_verb))
            || (w == "api.github.com" && env_near(seg, i, 3, 3, api_verb))
            || (seg[i].ci
                && (env_near(seg, i, 0, 3, ci_write)
                    || commit(at(i + 1))
                    || (matches!(at(i + 1), "job" | "workflow") && commit(at(i + 2)))))
    })
}

/// Whether any run of words in `segments` holds a host action ([`segment_has_host_action`]).
pub(super) fn has_host_action(segments: &[Vec<EnvWord>]) -> bool {
    segments.iter().any(|seg| segment_has_host_action(seg))
}

/// Whether `segments` name one of `phrases` (or, with `bare_history`, the single word `history` not followed by
/// `of`) with a source verb ([`ENV_SOURCE_VERBS`]) within 6 words before it, in the same run of words.
pub(super) fn names_source(
    segments: &[Vec<EnvWord>],
    phrases: &[&[&str]],
    bare_history: bool,
) -> bool {
    segments.iter().any(|seg| {
        (0..seg.len()).any(|i| {
            let phrase = phrases.iter().any(|p| {
                seg.get(i..i + p.len())
                    .is_some_and(|s| s.iter().zip(p.iter()).all(|(w, x)| w.lower == *x))
            });
            let history = bare_history
                && seg[i].lower == "history"
                && !seg.get(i + 1).is_some_and(|n| n.lower == "of");
            (phrase || history)
                && seg[i.saturating_sub(6)..i]
                    .iter()
                    .any(|w| ENV_SOURCE_VERBS.contains(&w.lower.as_str()))
        })
    })
}

/// Place-naming sources a criterion reads at runtime: the records branch, `athena/records`, git history or log.
pub(super) const ENV_LOCATED_SOURCES: [&[&str]; 5] = [
    &["records", "branch"],
    &["athena/records"],
    &["git", "history"],
    &["git", "log"],
    &["commit", "history"],
];

/// Sources a reading's own evidence can rest on: the located ones, job records and job history.
pub(super) const ENV_EVIDENCE_SOURCES: [&[&str]; 7] = [
    &["records", "branch"],
    &["athena/records"],
    &["git", "history"],
    &["git", "log"],
    &["commit", "history"],
    &["job", "records"],
    &["job", "history"],
];

/// Whether `text` quotes a counted figure: 1 to 5 digits directly before a counted noun (job, reading, commit,
/// record, run, plan, introspection, attempt, case, percent) or before `%`. A year or a bare number is not one.
pub(super) fn quotes_counted_figure(text: &str) -> bool {
    const NOUNS: [&str; 20] = [
        "job",
        "jobs",
        "reading",
        "readings",
        "commit",
        "commits",
        "record",
        "records",
        "run",
        "runs",
        "plan",
        "plans",
        "introspection",
        "introspections",
        "attempt",
        "attempts",
        "case",
        "cases",
        "percent",
        "percentage",
    ];
    let tokens: Vec<&str> = text.split_whitespace().collect();
    tokens.iter().enumerate().any(|(i, raw)| {
        let token = raw.trim_matches(|c: char| !c.is_alphanumeric() && c != '%');
        let (digits, percent) = match token.strip_suffix('%') {
            Some(d) => (d, true),
            None => (token, false),
        };
        if digits.is_empty() || digits.len() > 5 || !digits.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        percent
            || tokens.get(i + 1).is_some_and(|next| {
                let next = next
                    .trim_matches(|c: char| !c.is_alphanumeric())
                    .to_lowercase();
                NOUNS.contains(&next.as_str())
            })
    })
}

/// Whether `text` names a reachable data or doc file: a token with `/` that ends `.md`, `.json`, `.jsonl`,
/// `.csv` or `.txt` and is not under `records/` or `athena/records`. Whether the file exists is left to
/// [`check_reading_references`].
pub(super) fn names_reachable_file(text: &str) -> bool {
    text.split(|c: char| {
        c.is_whitespace() || matches!(c, ',' | ';' | '\'' | '"' | '(' | ')' | '[' | ']' | '`')
    })
    .map(|token| token.trim_end_matches(['.', ':']))
    .any(|token| {
        token.contains('/')
            && !token.starts_with("records/")
            && !token.starts_with("athena/records")
            && token
                .rsplit_once('.')
                .is_some_and(|(_, ext)| ["md", "json", "jsonl", "csv", "txt"].contains(&ext))
    })
}

/// The words of `text`, lowercase, apostrophes removed, split on anything that is not a letter or digit.
pub(super) fn plain_words(text: &str) -> Vec<String> {
    text.to_lowercase()
        .replace(['\'', '\u{2019}'], "")
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_string)
        .collect()
}

/// Words that name a place an action runs.
pub(super) const ENV_SITE_WORDS: [&str; 12] = [
    "ci", "runner", "workflow", "producer", "snapshot", "local", "host", "github", "reader",
    "planner", "verifier", "critic",
];
/// Words that name a permission.
pub(super) const ENV_PERMISSION_WORDS: [&str; 8] = [
    "read",
    "reads",
    "write",
    "writes",
    "writable",
    "readonly",
    "permission",
    "permissions",
];
/// Words that name the data or refs a place reaches.
pub(super) const ENV_DATA_WORDS: [&str; 18] = [
    "refs",
    "ref",
    "branch",
    "branches",
    "records",
    "history",
    "files",
    "repo",
    "repository",
    "checkout",
    "data",
    "commits",
    "logs",
    "figures",
    "origin",
    "remote",
    "api",
    "prs",
];
/// Sites that run on one machine with a working copy: for these the case with no origin must be stated.
pub(super) const ENV_LOCAL_SITE_WORDS: [&str; 5] =
    ["local", "producer", "snapshot", "checkout", "copy"];
/// Words that say a thing is missing.
pub(super) const ENV_ABSENT_WORDS: [&str; 8] = [
    "no", "without", "missing", "absent", "none", "non", "always", "lacks",
];
/// Words that name a stored copy a later run can restore: a cache or a stored artifact.
pub(super) const ENV_CACHE_WORDS: [&str; 6] = [
    "cache",
    "caches",
    "cached",
    "caching",
    "artifact",
    "artifacts",
];
/// Words that name a CI place only beside `github`, or when the reading has a workflow deliverable.
pub(super) const ENV_WEAK_CI_PLACE_WORDS: [&str; 4] =
    ["workflow", "workflows", "runner", "runners"];
/// Verbs of using a stored CI result again.
pub(super) const ENV_REUSE_WORDS: [&str; 4] = ["reuse", "reuses", "reused", "reusing"];
/// Nouns of a CI result, on the trigger side.
pub(super) const ENV_RESULT_WORDS: [&str; 7] = [
    "run", "runs", "pass", "passes", "result", "results", "status",
];
/// Words that tie a reused CI result to another ref or an earlier run, not to one commit.
pub(super) const ENV_CROSS_REF_WORDS: [&str; 16] = [
    "early", "earlier", "push", "pushes", "pushed", "pushing", "branch", "branches", "land",
    "lands", "landing", "another", "previous", "base", "ref", "refs",
];
/// Nouns a cache statement in an `Environment:` assumption names: the cache words and a stored CI result.
pub(super) const ENV_CACHE_NOUN_WORDS: [&str; 11] = [
    "cache",
    "caches",
    "cached",
    "caching",
    "artifact",
    "artifacts",
    "result",
    "results",
    "status",
    "pass",
    "passes",
];
/// Words that say a ref or event saves an entry.
pub(super) const ENV_SAVE_WORDS: [&str; 10] = [
    "save", "saves", "saved", "saving", "store", "stores", "stored", "upload", "uploads",
    "uploaded",
];
/// Words that say which runs restore an entry.
pub(super) const ENV_RESTORE_WORDS: [&str; 11] = [
    "restore",
    "restores",
    "restored",
    "restoring",
    "download",
    "downloads",
    "downloaded",
    "reuse",
    "reuses",
    "reused",
    "reusing",
];
/// Words that say who may write shared entries (plain `read` is not one).
pub(super) const ENV_WRITE_RIGHTS_WORDS: [&str; 7] = [
    "write",
    "writes",
    "written",
    "writable",
    "readonly",
    "permission",
    "permissions",
];
/// Words that name the ref scope of a cache; `default` and `pull` count only in a pair, see [`names_ref_scope`].
pub(super) const ENV_SCOPE_WORDS: [&str; 9] = [
    "branch", "branches", "ref", "refs", "main", "pr", "prs", "scope", "scoped",
];

/// Whether `words` name a ref scope: a scope word, `default` before branch, branches, ref or refs, or `pull`
/// before request or requests.
pub(super) fn names_ref_scope(words: &[String]) -> bool {
    words.iter().enumerate().any(|(i, w)| {
        let next = words.get(i + 1).map_or("", String::as_str);
        ENV_SCOPE_WORDS.contains(&w.as_str())
            || (w == "default" && matches!(next, "branch" | "branches" | "ref" | "refs"))
            || (w == "pull" && matches!(next, "request" | "requests"))
    })
}

/// Whether `text` names one of the cache and artifact actions of GitHub Actions (`actions/cache`,
/// `actions/upload-artifact`, `actions/download-artifact`), with or without a version after `@`.
pub(super) fn names_actions_cache(text: &str) -> bool {
    text.split(|c: char| c.is_whitespace() || matches!(c, '`' | ',' | '(' | ')'))
        .map(|token| token.trim_matches(|c: char| !c.is_alphanumeric()))
        .map(|token| token.split('@').next().unwrap_or("").to_lowercase())
        .any(|token| {
            matches!(
                token.as_str(),
                "actions/cache" | "actions/upload-artifact" | "actions/download-artifact"
            )
        })
}

/// Whether one run of words ties a CI cache, a stored artifact or a reused CI result to a ref: a CI place (`CI`
/// written exactly, `github actions`, or a workflow or runner beside `github` or in a reading with a workflow
/// deliverable) with a cache word, or with a reuse word, a result noun and a cross-ref word and no `sha` or
/// `commit` word (a result matched by commit is not scoped to a ref).
pub(super) fn segment_has_cache_scope_trigger(
    seg: &[EnvWord],
    has_workflow_deliverable: bool,
) -> bool {
    let has = |list: &[&str]| seg.iter().any(|w| list.contains(&w.lower.as_str()));
    let github = seg.iter().any(|w| w.lower == "github");
    let ci_place = seg.iter().enumerate().any(|(i, w)| {
        w.ci || (w.lower == "github" && seg.get(i + 1).is_some_and(|n| n.lower == "actions"))
            || (ENV_WEAK_CI_PLACE_WORDS.contains(&w.lower.as_str())
                && (github || has_workflow_deliverable))
    });
    if !ci_place {
        return false;
    }
    if has(&ENV_CACHE_WORDS) {
        return true;
    }
    has(&ENV_REUSE_WORDS)
        && has(&ENV_RESULT_WORDS)
        && has(&ENV_CROSS_REF_WORDS)
        && !has(&["sha", "commit", "commits"])
}

/// Refuses a reading that assigns an action or a measurement to a place without saying who performs it, where,
/// with what permissions and what data it can reach.
///
/// Root cause this check removes: the reading stage had no step asking who performs each action, where, with
/// what permissions, and what data it reaches. It worked from the request text and the local repository view,
/// so a plan assumed that CI can write, that a GitHub origin exists, or that the job records are reachable from
/// the place that needs them. Job 20261024-220048-engm stopped in planning on records the plan could not reach
/// from a read-only CI job; job 20261025-070538-8ra0 was refused because its early push and pull request met a
/// working copy with no origin or a non-GitHub origin; job 20261025-034247-nm0k cited job records and history
/// as its evidence without quoting the figures or listing a file that holds them. Introspections
/// 20261024-220314-yjtg, 20261025-070613-4vfa and 20261026-011842-889s traced all three to the same missing
/// step.
///
/// It applies when the reading has a capability and no question. It reads four things for the place statement,
/// and a fifth for the cache statement described below, and nothing else (not the request, a content criterion,
/// an assumption or `out_of_scope`, so the repair text cannot trigger it):
/// (A) a created or modified deliverable under `.github/workflows/`; (B) a behavior criterion with a host
/// action: a `git` or `gh` command, a push with a ref, branch or origin, a fetch, clone or pull from a remote,
/// a pull request or `PR` (written exactly so) with an opening, creating, raising, merging, reusing, pushing or
/// submitting verb, a GitHub API call, or a `CI` (written exactly so) write or commit; (C1) a behavior
/// criterion that reads the records branch, `athena/records`, git history or the git log, with a read verb
/// (from, use, read, counts, measures, replays, queries, parses, scans) within 6 words before it; (C2) a summary
/// that cites job records, job history, commit history, git history, the git log, the records branch or the
/// single word `history` (not `history of`) as a source, with such a verb within 6 words before it.
///
/// A, B and C1 are met by one assumption that starts `Environment:` and names a site (CI, runner, workflow,
/// producer, snapshot, local, host, github, reader, planner, verifier, critic), a permission (read, write,
/// readonly, permission) and the data or refs it reaches (refs, branch, records, history, files, repo, origin,
/// and the like). For B, when the site is a working copy (local, producer, snapshot, checkout, copy), the
/// assumption must also say the case with no origin or a non-GitHub origin (`origin` with no, without,
/// missing, absent, none, non, always or lacks). C2 is met by a counted figure quoted in any assumption (1 to 5
/// digits before jobs, readings, commits, records, runs, plans, introspections, attempts, cases, percent or
/// `%`; a year is not one) or by an `Environment:` assumption that lists a reachable data or doc file (a path
/// ending `.md`, `.json`, `.jsonl`, `.csv` or `.txt`, not under `records/`).
///
/// C1 and C2 differ from a plain reading of "figures or a reachable file" on purpose. Code that reads the
/// records branch at runtime needs to know where it runs and what that place can see; quoted figures say
/// nothing about that, so C1 asks for the `Environment:` statement. Figures matter when the reading's own
/// claims rest on the records, which is C2. Ordinary product wording such as "computed from the job records" in
/// a behavior criterion names no place and is not a trigger.
///
/// Limits: the check is lexical. It reads behavior criteria, the deliverables and the summary only. Paths,
/// identifiers and backticked text that is not a command are skipped, so `pr.rs`, `gh/early.rs` and
/// `parse_remote` trigger nothing. A source more than 6 words from its verb is missed, as are "counts readings
/// in records" and a lowercase `pr`. A sentence can meet the word lists without being true; the check makes the
/// reader state the three facts, as `Code survey:` makes it name a path. Whether a listed file exists is left
/// to [`check_reading_references`], which does not verify paths under an unknown top directory.
///
/// This check refuses more readings; the higher refusal rate is intended. A replay of accepted readings
/// (`replay`) holds a job that newly refuses more than 10% unless the job's criteria declare a share, as
/// "expected N% of newly refused readings". The declared share for this change, expected 8% of newly refused
/// readings, is an estimate that was not measured: the records are not in the producer's working copy, and the
/// basis is that few stored readings carry a workflow deliverable, a push or pull request action, or records
/// cited in the summary. The replay at verification measures it.
///
/// Root cause of the cache rule: the reading had no step asking, for each CI cache or stored artifact, which ref
/// saves it, which runs restore it, and which runs may write shared entries. It worked from the request text and
/// the local repository view, so two plans (jobs 20261025-034731-2am0 and 20261025-055626-er96) designed a
/// result cache keyed by a fingerprint and an early push whose CI run is reused at landing without asking
/// which ref saves the entry or which runs can restore it. Introspections 20261025-050603-wrv7 and
/// 20261025-065849-n99a traced both to that missing step. The fifth input is a cache trigger in a behavior
/// criterion or the summary: (D1) a behavior criterion with a CI place (`CI` written exactly, `github actions`,
/// or a workflow or runner beside `github` or in a reading with a workflow deliverable) and a cache or artifact
/// word, or naming `actions/cache`, `actions/upload-artifact` or `actions/download-artifact`; (D2) a behavior
/// criterion with a CI place, a reuse word, a result noun and a cross-ref word (early, push, branch, landing and
/// the like) and no `sha` or `commit` word; (D3) a created or modified `.github/workflows/` deliverable with a
/// cache or artifact word in a behavior criterion or the summary. The reading then needs one `Environment:`
/// assumption that names the cache, artifact or result and says which ref and event saves it, which runs
/// restore it, which runs may write shared entries, and the ref scope. The best such assumption (fewest parts
/// missing) is judged; the union of all `Environment:` assumptions is not, so a CI block cannot hide a missing
/// part. Limits: the check is lexical; a lowercase `ci` is missed; a CI result matched by commit SHA is
/// deliberately not a trigger; a bare `runner` or `workflow` is not a CI place; a pasted CI block that holds a
/// write word and a scope word beside a cache noun can meet the write and scope parts without saying who
/// writes shared entries.
///
/// This check refuses more readings; the higher refusal rate is intended, expected 3% of newly refused readings
/// (an estimate, not measured).
///
/// The error says which trigger fired and which part is missing, and tells the reader what to write.
pub fn check_environment_reach(reading: &Reading) -> Result<()> {
    if reading.capability.is_none() || reading.question.is_some() {
        return Ok(());
    }
    let workflows: Vec<&str> = reading
        .deliverables
        .iter()
        .filter(|d| {
            d.action != DeliverableAction::Delete && d.path.starts_with(".github/workflows/")
        })
        .map(|d| d.path.as_str())
        .collect();
    let behavior: Vec<Vec<Vec<EnvWord>>> = reading
        .acceptance_criteria
        .iter()
        .filter(|c| c.kind == CriterionKind::Behavior)
        .map(|c| scan_segments(&c.text))
        .collect();
    let host_action = behavior.iter().any(|s| has_host_action(s));
    let located_read = behavior
        .iter()
        .any(|s| names_source(s, &ENV_LOCATED_SOURCES, false));
    let summary = scan_segments(&reading.summary);
    let evidence = names_source(&summary, &ENV_EVIDENCE_SOURCES, true);
    // A cache or stored artifact is tied to a ref: D1 a CI place with a cache word, D2 a reused CI result tied
    // to another ref, D3 a workflow deliverable with a cache word in a behavior criterion or the summary.
    let names_action = reading
        .acceptance_criteria
        .iter()
        .filter(|c| c.kind == CriterionKind::Behavior)
        .any(|c| names_actions_cache(&c.text));
    let mentions_cache = |segments: &[Vec<EnvWord>]| {
        segments.iter().any(|seg| {
            seg.iter()
                .any(|w| ENV_CACHE_WORDS.contains(&w.lower.as_str()))
        })
    };
    let has_workflow = !workflows.is_empty();
    let cache_trigger = names_action
        || behavior.iter().any(|segments| {
            segments
                .iter()
                .any(|seg| segment_has_cache_scope_trigger(seg, has_workflow))
        })
        || (has_workflow
            && (behavior.iter().any(|s| mentions_cache(s))
                || mentions_cache(&summary)
                || names_actions_cache(&reading.summary)));
    if workflows.is_empty() && !host_action && !located_read && !evidence && !cache_trigger {
        return Ok(());
    }
    let environments = environment_assumptions(reading);
    let mut triggers: Vec<String> = Vec::new();
    let mut problems: Vec<String> = Vec::new();
    if !workflows.is_empty() || host_action || located_read {
        if !workflows.is_empty() {
            triggers.push(format!(
                "it adds a CI workflow deliverable ({})",
                workflows.join(", ")
            ));
        }
        if host_action {
            triggers.push(
                "a behavior criterion assigns a push, pull request, GitHub API call or CI write"
                    .into(),
            );
        }
        if located_read {
            triggers.push(
                "a behavior criterion reads the records branch or git history at runtime".into(),
            );
        }
        let has =
            |words: &[String], list: &[&str]| words.iter().any(|w| list.contains(&w.as_str()));
        let complete = environments.iter().any(|text| {
            let words = plain_words(text);
            let local = has(&words, &ENV_LOCAL_SITE_WORDS);
            has(&words, &ENV_SITE_WORDS)
                && has(&words, &ENV_PERMISSION_WORDS)
                && has(&words, &ENV_DATA_WORDS)
                && (!(host_action && local)
                    || (words.iter().any(|w| w == "origin") && has(&words, &ENV_ABSENT_WORDS)))
        });
        if !complete {
            let words: Vec<String> = environments.iter().flat_map(|t| plain_words(t)).collect();
            let mut missing: Vec<&str> = Vec::new();
            if environments.is_empty() {
                missing.push("an assumption that starts `Environment:`");
            }
            if !has(&words, &ENV_SITE_WORDS) {
                missing.push(
                    "the site that performs the action (CI job, producer working copy, local run)",
                );
            }
            if !has(&words, &ENV_PERMISSION_WORDS) {
                missing.push("its read or write permissions");
            }
            if !has(&words, &ENV_DATA_WORDS) {
                missing.push("the refs or data it can reach");
            }
            if missing.is_empty() {
                missing.push(
                    "the case with no origin or a non-GitHub origin, which a local site must state",
                );
            }
            problems.push(format!(
                "it has no complete `Environment:` assumption; missing {}. Repair: add an assumption that \
                 starts `Environment:` and says who performs the action and where, its read or write \
                 permissions, and the refs or data it can reach; for a local working copy also say what \
                 happens with no origin, a non-GitHub origin and a GitHub origin",
                missing.join("; ")
            ));
        }
    }
    if evidence {
        triggers.push("its summary cites job records or history as a source".into());
        let figure = reading.assumptions.iter().any(|a| quotes_counted_figure(a));
        let file = environments.iter().any(|a| names_reachable_file(a));
        if !figure && !file {
            problems.push(
                "no assumption quotes the counted figures (for example `3 jobs` or `40%`) and no \
                 `Environment:` assumption lists a reachable data or doc file; the records are not in the \
                 producer's working copy. Repair: quote the figures inline in an assumption, or list the \
                 file that holds them in an `Environment:` assumption"
                    .into(),
            );
        }
    }
    let place_problems = problems.len();
    if cache_trigger {
        triggers.push(
            "it describes a CI cache, a stored artifact or a CI result reused from another ref"
                .into(),
        );
        let has =
            |words: &[String], list: &[&str]| words.iter().any(|w| list.contains(&w.as_str()));
        // The best `Environment:` assumption that names the cache, artifact or result: the one lacking the
        // fewest parts. The union of all of them would let a CI block hide a missing part.
        let best = environments
            .iter()
            .map(|text| plain_words(text))
            .filter(|words| has(words, &ENV_CACHE_NOUN_WORDS))
            .map(|words| {
                let mut lacks: Vec<&str> = Vec::new();
                if !has(&words, &ENV_SAVE_WORDS) {
                    lacks.push("which ref and event saves it");
                }
                if !has(&words, &ENV_RESTORE_WORDS) {
                    lacks.push("which runs restore it");
                }
                if !has(&words, &ENV_WRITE_RIGHTS_WORDS) {
                    lacks.push("which runs may write shared entries");
                }
                if !names_ref_scope(&words) {
                    lacks.push("the ref scope");
                }
                lacks
            })
            .min_by_key(Vec::len);
        let repair = "Repair: add one assumption that starts `Environment:`, names the cache, artifact or \
                      result and states its save scope, restore scope and write rights, for example \
                      \"Environment: the CI cache is saved by the main branch push run, restored by pull \
                      request runs of the same branch and by main, written only by main\"";
        match best {
            Some(lacks) if lacks.is_empty() => {}
            Some(lacks) => problems.push(format!(
                "the `Environment:` assumption that names the cache, artifact or result lacks {}. {repair}",
                lacks.join("; ")
            )),
            None => problems.push(format!(
                "no `Environment:` assumption names the cache, artifact or result; it must state which ref \
                 and event saves it; which runs restore it; which runs may write shared entries; the ref \
                 scope. {repair}"
            )),
        }
    }
    if problems.is_empty() {
        return Ok(());
    }
    let mut tails: Vec<&str> = Vec::new();
    if place_problems > 0 {
        tails.push("who performs it, where, with what permissions and what data it reaches");
    }
    if problems.len() > place_problems {
        tails.push(
            "which ref saves the cache or artifact, which runs restore it and which runs may write \
             shared entries",
        );
    }
    bail!(
        "the reading was refused because {}, and nothing in the reading says {}: {}",
        triggers.join("; and "),
        tails.join(", or "),
        problems.join("; and ")
    )
}

/// The `Existing code:` assumptions of a reading: those whose trimmed text starts with that label, in any case.
pub(super) fn existing_code_assumptions(reading: &Reading) -> Vec<&str> {
    reading
        .assumptions
        .iter()
        .map(|a| a.trim())
        .filter(|a| a.to_lowercase().starts_with("existing code:"))
        .collect()
}

/// Whether `text` names a path under `crates/` or `docs/` (the root followed by a letter or digit) that is not
/// in `creates`, the paths the job adds. Tokens split on white space, quotes, brackets, commas, semicolons and
/// backticks, and a trailing `.` or `:` is dropped.
pub(super) fn names_existing_path(text: &str, creates: &[&str]) -> bool {
    text.split(|c: char| {
        c.is_whitespace() || matches!(c, ',' | ';' | '\'' | '"' | '(' | ')' | '[' | ']' | '`')
    })
    .map(|token| token.trim_end_matches(['.', ':']))
    .any(|token| {
        ["crates/", "docs/"].iter().any(|root| {
            token
                .strip_prefix(root)
                .is_some_and(|rest| rest.chars().next().is_some_and(char::is_alphanumeric))
        }) && !creates.contains(&token)
    })
}

/// Whether a criterion requires an existing code section that reuses or extends what it lists: the words
/// "existing code section", then a reuse or extend word within 12 words, with no negator (no, not, never,
/// without, nothing, nor, cannot) from 3 words before the phrase through that word. A criterion such as "the
/// existing code in section 3 is unchanged" or "no existing code section is needed, nothing to reuse" fails.
pub(super) fn requires_existing_code_section(text: &str) -> bool {
    const REUSE: [&str; 8] = [
        "reuse",
        "reuses",
        "reused",
        "reusing",
        "extend",
        "extends",
        "extended",
        "extending",
    ];
    const NEGATORS: [&str; 7] = ["no", "not", "never", "without", "nothing", "nor", "cannot"];
    let lower = text
        .to_lowercase()
        .replace("n't", " not")
        .replace("n\u{2019}t", " not");
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .collect();
    (0..words.len()).any(|i| {
        if words.get(i..i + 3) != Some(&["existing", "code", "section"][..]) {
            return false;
        }
        let from = i + 3;
        let Some(offset) = words[from..words.len().min(from + 12)]
            .iter()
            .position(|w| REUSE.contains(w))
        else {
            return false;
        };
        !words[i.saturating_sub(3)..=from + offset]
            .iter()
            .any(|w| NEGATORS.contains(w))
    })
}

/// Refuses a design reading that does not list the existing code it changes or reuses: it needs an assumption
/// that starts `Existing code:` and names a `crates/` or `docs/` path the job does not create, and a criterion
/// that requires an existing code section that reuses or extends it. Root cause this check removes: no
/// inventory of existing code was required before a design reading was accepted, so a design that rebuilds or
/// contradicts what the repository already holds was accepted; jobs 20261026-043137-zc0v and
/// 20261026-065046-0c1d are the design jobs that were accepted that way. It applies when the reading has a
/// capability and no question and its class of work is `design`; every other class, a question and a decline
/// are not affected. It reads only the reading (deliverable paths, assumptions, criteria), never the
/// repository; `check_reading_references` compares the named paths with the snapshot when the files are known.
/// This check refuses more readings; the higher refusal rate is intended. A replay of accepted readings
/// (`replay`) holds a job that newly refuses more than 10% unless the job's criteria declare a share, as
/// "expected N% of newly refused readings". The error names the `Existing code:` label and tells the reader
/// what to list.
pub fn check_design_existing_code(reading: &Reading) -> Result<()> {
    if reading.capability.is_none()
        || reading.question.is_some()
        || reading.class_of_work != "design"
    {
        return Ok(());
    }
    let creates: Vec<&str> = reading
        .deliverables
        .iter()
        .filter(|d| d.action == DeliverableAction::Create)
        .map(|d| d.path.as_str())
        .collect();
    if !existing_code_assumptions(reading)
        .into_iter()
        .any(|a| names_existing_path(a, &creates))
    {
        bail!(
            "the reading was refused because it is a design reading with no `Existing code:` assumption that \
             names a crates/ or docs/ path the job does not create ({}); the design was written without an \
             inventory of what is already there. Read the code, then add an assumption that starts \
             `Existing code:` and lists the crates/ or docs/ files, functions and docs the design changes or \
             reuses, not the files the job creates",
            if creates.is_empty() {
                "it creates none".to_string()
            } else {
                format!("it creates {}", creates.join(", "))
            }
        );
    }
    if !reading
        .acceptance_criteria
        .iter()
        .any(|c| requires_existing_code_section(&c.text))
    {
        bail!(
            "the reading was refused because it is a design reading whose criteria do not require the design to \
             reuse or extend the code its `Existing code:` assumption lists; add a criterion that requires an \
             existing code section that reuses or extends those files, for example 'The design has an Existing \
             code section that reuses or extends <path>' (the words existing code section, then reuses or \
             extends, with no \"no\" or \"not\" before them)"
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------------------------------------
// The test design of a code survey

/// The labels `check_behavior_test_design` reads inside the `Code survey:` assumptions, lowercase. A clause
/// runs from its label to the next label. `existing code:` is an alias of `reuses:`.
pub(super) const SURVEY_LABELS: [&str; 7] = [
    "entry point:",
    "gate order:",
    "stop gate:",
    "reuses:",
    "existing code:",
    "output:",
    "direct check:",
];

/// Every record kind the records validation accepts: (enum, variant, schema tag) for `AnyRecord` in
/// `athena-data` `records.rs` and `HistoryRecord` in `history.rs`. Written by hand and pinned to the enum
/// source by the test `record_kinds_match_the_enums`, so a new variant fails a test, not a reading. A pub
/// `SCHEMA_TAGS` constant in `athena-data` would replace this list; that is a follow-up, not part of this change.
pub(super) const RECORD_KINDS: [(&str, &str, &str); 43] = [
    ("AnyRecord", "Request", "athena.request/1"),
    ("AnyRecord", "Reading", "athena.reading/1"),
    ("AnyRecord", "Correction", "athena.correction/1"),
    ("AnyRecord", "Priority", "athena.priority/1"),
    ("AnyRecord", "Hold", "athena.hold/1"),
    ("AnyRecord", "Focus", "athena.focus/1"),
    ("AnyRecord", "Job", "athena.job/1"),
    ("AnyRecord", "Invocation", "athena.invocation/1"),
    ("AnyRecord", "Candidate", "athena.candidate/1"),
    ("AnyRecord", "Candidate2", "athena.candidate/2"),
    ("AnyRecord", "Verification", "athena.verification/1"),
    ("AnyRecord", "Outcome", "athena.outcome/1"),
    ("AnyRecord", "Score", "athena.score/1"),
    ("AnyRecord", "Intervention", "athena.intervention/1"),
    ("AnyRecord", "Restart", "athena.restart/1"),
    ("AnyRecord", "Resolution", "athena.resolution/1"),
    ("AnyRecord", "HostError", "athena.host-error/1"),
    ("AnyRecord", "DecisionState", "athena.decision-state/1"),
    (
        "AnyRecord",
        "DefaultExpectations",
        "athena.default-expectations/1",
    ),
    ("AnyRecord", "Introspection", "athena.introspection/1"),
    ("AnyRecord", "FailureClass", "athena.failure-class/1"),
    (
        "AnyRecord",
        "DefaultsProposal",
        "athena.defaults-proposal/1",
    ),
    ("AnyRecord", "ClassLabel", "athena.class-label/1"),
    ("AnyRecord", "Role", "athena.role/1"),
    ("AnyRecord", "WorkItem", "athena.work-item/1"),
    ("AnyRecord", "WorkDecision", "athena.work-decision/1"),
    ("AnyRecord", "AudienceRating", "athena.audience-rating/1"),
    ("AnyRecord", "Audit", "athena.audit/1"),
    ("AnyRecord", "HestiaSweep", "athena.hestia-sweep/1"),
    ("AnyRecord", "JobPlan", "athena.job-plan/1"),
    ("AnyRecord", "Critique", "athena.critique/1"),
    ("AnyRecord", "Baseline", "athena.baseline/1"),
    ("AnyRecord", "Replay", "athena.replay/1"),
    (
        "AnyRecord",
        "CodeHealthBaseline",
        "athena.code-health-baseline/1",
    ),
    (
        "AnyRecord",
        "ThroughputReview",
        "athena.throughput-review/1",
    ),
    (
        "AnyRecord",
        "BuildMeasurement",
        "athena.build-measurement/1",
    ),
    ("AnyRecord", "DesignSurface", "athena.design-surface/1"),
    ("HistoryRecord", "LandingIntent", "athena.landing-intent/1"),
    ("HistoryRecord", "Landing", "athena.landing/1"),
    (
        "HistoryRecord",
        "DeclaredIntervention",
        "athena.declared-intervention/1",
    ),
    ("HistoryRecord", "Checkpoint", "athena.checkpoint/1"),
    ("HistoryRecord", "LegacyExempt", "athena.legacy-exempt/1"),
    (
        "HistoryRecord",
        "ImportManifest",
        "athena.import-manifest/1",
    ),
];

/// Field names that start with a skip word but are not switches: the producer report's `no_change` and
/// `no_change_reason`. Only these are exempt from the skip-identifier rule; a name in the entry point, gate
/// order, stop gate or reuse clause is not.
pub(super) const KNOWN_FIELD_NAMES: [&str; 2] = ["no_change", "no_change_reason"];

/// Words between a negator and the item it governs: articles, prepositions and linking words. A negator
/// governs an item only through these (and other matched items), so `does not run when X is set` does not
/// negate `X`.
pub(super) const BRIDGE_WORDS: &[&str] = &[
    "a", "an", "the", "any", "by", "via", "using", "through", "with", "reading", "read", "reads",
    "adding", "add", "of", "as", "or", "and", "nor", "to", "just", "only", "for", "",
];

/// Past participles that pass a negator only when it comes right after it (`not measured through ...`).
const PARTICIPLES: [&str; 4] = ["measured", "checked", "verified", "tested"];

/// The stems of the words that state a measurement in a direct check.
pub(super) const MEASURE_STEMS: [&str; 8] = [
    "assert", "count", "compar", "observ", "measur", "check", "verif", "expect",
];

/// Words a routine criterion (a regression run of fmt, clippy and the tests) may hold.
pub(super) const ROUTINE_WORDS: [&str; 38] = [
    "cargo",
    "fmt",
    "clippy",
    "test",
    "tests",
    "check",
    "pass",
    "passes",
    "and",
    "the",
    "on",
    "final",
    "tree",
    "existing",
    "still",
    "all",
    "a",
    "recorded",
    "athena",
    "same",
    "locked",
    "targets",
    "warnings",
    "workspace",
    "no",
    "unrelated",
    "behavior",
    "change",
    "changes",
    "build",
    "clean",
    "green",
    "as",
    "is",
    "are",
    "with",
    "run",
    "numbers",
];

/// The parts of a code survey, read from its labelled clauses.
#[derive(Debug, Default)]
pub(super) struct SurveyParts {
    entry: Option<String>,
    gates: Option<String>,
    stop: Option<String>,
    reuses: Option<String>,
    output: Option<String>,
    directs: Vec<String>,
}

fn append(slot: &mut Option<String>, text: &str) {
    match slot {
        Some(existing) => {
            existing.push(' ');
            existing.push_str(text);
        }
        None => *slot = Some(text.to_string()),
    }
}

/// Reads the labelled clauses of every `Code survey:` assumption. A clause runs from its label (any case, at
/// the start of the text or after white space or a full stop) to the next label. A repeated label appends
/// (except `Direct check:`, one entry per clause); `Existing code:` folds into `Reuses:`.
pub(super) fn survey_parts(reading: &Reading) -> SurveyParts {
    let text = code_surveys(reading)
        .into_iter()
        .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join(" ");
    let lower = text.to_ascii_lowercase();
    let mut marks: Vec<(usize, usize, usize)> = Vec::new();
    for (k, label) in SURVEY_LABELS.iter().enumerate() {
        for (at, _) in lower.match_indices(label) {
            let before = lower[..at].chars().next_back();
            if before.is_none_or(|c| c.is_whitespace() || c == '.') {
                marks.push((at, label.len(), k));
            }
        }
    }
    marks.sort();
    let mut parts = SurveyParts::default();
    for (i, (at, len, k)) in marks.iter().enumerate() {
        let end = marks.get(i + 1).map_or(text.len(), |m| m.0);
        let clause = text[at + len..end].trim().trim_end_matches('.').trim();
        match k {
            0 => append(&mut parts.entry, clause),
            1 => append(&mut parts.gates, clause),
            2 => append(&mut parts.stop, clause),
            3 | 4 => append(&mut parts.reuses, clause),
            5 => append(&mut parts.output, clause),
            _ => parts.directs.push(clause.to_string()),
        }
    }
    parts
}

/// The text that states what the change reuses: the `Reuses:` clause (and `Existing code:` inside the survey)
/// and the text after the label of each `Existing code:` assumption.
fn reuse_text(reading: &Reading, parts: &SurveyParts) -> String {
    let mut text = parts.reuses.clone().unwrap_or_default();
    for assumption in existing_code_assumptions(reading) {
        text.push(' ');
        text.push_str(assumption.get("existing code:".len()..).unwrap_or(""));
    }
    text
}

/// One whitespace token with the punctuation around a name trimmed. Returns the name and whether it was in
/// backticks or ended in `()`.
fn clean_token(token: &str) -> (String, bool) {
    let mut t = token;
    loop {
        let before = t.len();
        t = t.trim_start_matches(['(', '[', '"', '\'']);
        t = t.trim_end_matches([',', ';', ':', '.', '"', '\'', ']']);
        if t.ends_with(')') && !t.ends_with("()") {
            t = &t[..t.len() - 1];
        }
        if t.len() == before {
            break;
        }
    }
    let ticked = t.starts_with('`') && t.ends_with('`') && t.len() > 1;
    let name = t.trim_matches('`');
    let call = name.ends_with("()");
    (name.trim_end_matches("()").to_string(), ticked || call)
}

/// Whether a trimmed name is a code identifier: letters, digits, `_` and `::`, starting with a letter or `_`.
/// A path (`/`, a file extension) fails because `/` and `.` are not allowed.
fn is_identifier_name(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|c| c.is_alphabetic() || c == '_')
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == ':')
}

/// The identifiers in `text`: a backtick-quoted token with no white space, or a bare white-space token
/// containing `_` or `::` or ending in `()`. Paths and file names are not identifiers.
pub(super) fn identifiers(text: &str) -> Vec<String> {
    text.split_whitespace()
        .filter_map(|token| {
            let (name, marked) = clean_token(token);
            let shaped = marked || name.contains('_') || name.contains("::");
            (shaped && is_identifier_name(&name)).then_some(name)
        })
        .collect()
}

/// Whether `text` holds `name` as a whole name (the neighbours are not letters, digits or `_`).
fn contains_name(text: &str, name: &str) -> bool {
    text.match_indices(name).any(|(at, _)| {
        let word = |c: char| c.is_alphanumeric() || c == '_';
        !text[..at].chars().next_back().is_some_and(word)
            && !text[at + name.len()..].chars().next().is_some_and(word)
    })
}

/// The criterion numbers a direct check clause cites: the numbers after `criterion` or `criteria`, with
/// lists (`1, 2`, `1 and 2`) and ranges (`1-4`, `1 to 4`) expanded.
pub(super) fn cited_criteria(clause: &str) -> Vec<usize> {
    let lower = clause.to_lowercase();
    let mut cited = Vec::new();
    for key in ["criterion", "criteria"] {
        for (at, _) in lower.match_indices(key) {
            if lower[..at]
                .chars()
                .next_back()
                .is_some_and(char::is_alphanumeric)
            {
                continue;
            }
            let rest: Vec<char> = lower[at + key.len()..].chars().collect();
            let (mut i, mut previous, mut dash) = (0, None::<usize>, false);
            loop {
                while rest.get(i).is_some_and(|c| c.is_whitespace()) {
                    i += 1;
                }
                let Some(c) = rest.get(i) else { break };
                if c.is_ascii_digit() {
                    let mut n = 0usize;
                    while let Some(d) = rest.get(i).and_then(|c| c.to_digit(10)) {
                        n = n.saturating_mul(10).saturating_add(d as usize);
                        i += 1;
                    }
                    match previous {
                        Some(p) if dash && p < n && n - p < 200 => cited.extend(p + 1..=n),
                        _ => cited.push(n),
                    }
                    previous = Some(n);
                    dash = false;
                } else if previous.is_some() && matches!(c, '-' | '\u{2013}') {
                    dash = true;
                    i += 1;
                } else if previous.is_some() && matches!(c, ',' | '&') {
                    i += 1;
                } else if previous.is_some()
                    && ["and", "to"].iter().any(|w| {
                        let word: Vec<char> = w.chars().collect();
                        rest.get(i..i + word.len()) == Some(&word[..])
                            && !rest
                                .get(i + word.len())
                                .is_some_and(|c| c.is_alphanumeric())
                    })
                {
                    dash = rest.get(i..i + 2) == Some(&['t', 'o'][..]);
                    i += if dash { 2 } else { 3 };
                } else {
                    break;
                }
            }
        }
    }
    cited.sort_unstable();
    cited.dedup();
    cited
}

/// Whether a criterion only asks for a regression run: fmt, clippy or the tests pass. Backticks are removed,
/// flag tokens (and the value after `-p`, `--package`, `--test`, `--bin`, `--features`) are dropped, and every
/// remaining word must be in [`ROUTINE_WORDS`]; the text must hold a check word and a pass word.
pub(super) fn is_routine_criterion(text: &str) -> bool {
    let plain = text.replace('`', "");
    let mut kept: Vec<&str> = Vec::new();
    let mut skip_next = false;
    for token in plain.split_whitespace() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if token.starts_with('-') {
            skip_next = matches!(
                token,
                "-p" | "--package" | "--test" | "--bin" | "--features"
            );
            continue;
        }
        kept.push(token);
    }
    let words = crate::verify::clause_words(&kept.join(" "));
    !words.is_empty()
        && words.iter().all(|w| ROUTINE_WORDS.contains(&w.as_str()))
        && words
            .iter()
            .any(|w| matches!(w.as_str(), "fmt" | "clippy" | "test" | "tests"))
        && words
            .iter()
            .any(|w| matches!(w.as_str(), "pass" | "passes" | "clean" | "green"))
}

/// Whether the clause holds a plain word that states a measurement: a word of letters only (a backticked
/// token or one with `_`, `::` or `()` never counts) that starts with one of [`MEASURE_STEMS`].
pub(super) fn has_measurement_word(clause: &str) -> bool {
    clause.split_whitespace().any(|token| {
        let plain = token.trim_matches(|c: char| !c.is_alphanumeric());
        !token.contains('`')
            && !plain.is_empty()
            && plain.chars().all(char::is_alphabetic)
            && token
                .chars()
                .all(|c| c.is_alphabetic() || ",;:.()".contains(c))
            && {
                let lower = plain.to_lowercase();
                MEASURE_STEMS.iter().any(|stem| lower.starts_with(stem))
            }
    })
}

/// Whether the clause names a planner, producer or model call: one of those words and `call` or `calls`.
fn names_model_call(clause: &str) -> bool {
    let words = crate::verify::clause_words(clause);
    words
        .iter()
        .any(|w| matches!(w.as_str(), "planner" | "producer" | "model"))
        && words.iter().any(|w| matches!(w.as_str(), "call" | "calls"))
}

/// A negator that governs the item after it: the verifier's negators and the words of refusal.
fn is_means_negator(word: &str) -> bool {
    crate::verify::is_negator(word)
        || matches!(
            word,
            "without"
                | "instead"
                | "refuse"
                | "refuses"
                | "refused"
                | "reject"
                | "rejects"
                | "rejected"
                | "forbid"
                | "forbids"
        )
}

/// Splits text into segments at `;`, at a full stop followed by white space or the end, at line breaks, and
/// at the word `but`. Each segment is its white-space tokens. Commas, colons and dots inside a token do not
/// split, so `athena.failure-class/1` and `a::b` stay whole.
pub(super) fn measure_segments(text: &str) -> Vec<Vec<String>> {
    let mut pieces: Vec<String> = vec![String::new()];
    let chars: Vec<char> = text.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        let ends = match c {
            ';' | '\n' => true,
            '.' => chars.get(i + 1).is_none_or(|n| n.is_whitespace()),
            _ => false,
        };
        if ends {
            pieces.push(String::new());
        } else if let Some(last) = pieces.last_mut() {
            last.push(*c);
        }
    }
    let mut segments = Vec::new();
    for piece in pieces {
        let mut current: Vec<String> = Vec::new();
        for token in piece.split_whitespace() {
            if token
                .trim_matches(|c: char| !c.is_alphanumeric())
                .eq_ignore_ascii_case("but")
            {
                segments.push(std::mem::take(&mut current));
            } else {
                current.push(token.to_string());
            }
        }
        segments.push(current);
    }
    segments.retain(|s| !s.is_empty());
    segments
}

/// The lowercase word of a token: backticks, commas, brackets, colons and `()` trimmed.
fn token_word(token: &str) -> String {
    token
        .trim_matches(|c: char| {
            matches!(
                c,
                '`' | ',' | '(' | ')' | '[' | ']' | ':' | '"' | '\'' | ';'
            )
        })
        .to_lowercase()
}

/// Whether a raw token is an identifier that names a skip switch: an all-caps name with a NO, SKIP, SKIPPED,
/// DISABLE, DISABLED or BYPASS part (`ATHENA_NO_PROBE`, `=value` ignored), or a lowercase `snake_case` name or
/// `--flag` whose first part is `skip`, `skipped`, `no`, `disable`, `disabled` or `bypass`. The names in
/// [`KNOWN_FIELD_NAMES`] are exempt.
pub(super) fn skip_identifier(token: &str) -> bool {
    let trimmed = token.trim_matches(|c: char| {
        matches!(
            c,
            '`' | ',' | '(' | ')' | '[' | ']' | ':' | '"' | '\'' | ';' | '.'
        )
    });
    let name = trimmed.split('=').next().unwrap_or("");
    if name.is_empty() || KNOWN_FIELD_NAMES.contains(&name) {
        return false;
    }
    let flag = name.starts_with("--");
    let body = name.trim_start_matches('-');
    if !body
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        || body.is_empty()
    {
        return false;
    }
    if !flag && !body.contains('_') {
        return false;
    }
    let has_letter = body.chars().any(char::is_alphabetic);
    if has_letter
        && body
            .chars()
            .filter(|c| c.is_alphabetic())
            .all(char::is_uppercase)
    {
        return body.split(['_', '-']).any(|p| {
            matches!(
                p,
                "NO" | "SKIP" | "SKIPPED" | "DISABLE" | "DISABLED" | "BYPASS"
            )
        });
    }
    if body.chars().any(char::is_uppercase) {
        return false;
    }
    body.split(['_', '-']).next().is_some_and(|p| {
        matches!(
            p,
            "skip" | "skipped" | "no" | "disable" | "disabled" | "bypass"
        )
    })
}

const SWITCH_PHRASES: [&[&str]; 7] = [
    &["skip", "switch"],
    &["skip", "flag"],
    &["skip", "option"],
    &["internal", "helper"],
    &["private", "helper"],
    &["test", "only", "switch"],
    &["test-only", "switch"],
];

/// A skip-switch or internal-helper item in a segment: token range and whether it is a phrase.
#[derive(Debug, Clone)]
struct SwitchItem {
    start: usize,
    end: usize,
    phrase: bool,
}

fn switch_items(tokens: &[String]) -> Vec<SwitchItem> {
    let words: Vec<String> = tokens.iter().map(|t| token_word(t)).collect();
    let mut items = Vec::new();
    for start in 0..words.len() {
        for phrase in SWITCH_PHRASES {
            let fits = start + phrase.len() <= words.len()
                && phrase.iter().enumerate().all(|(k, p)| {
                    let w = words[start + k].as_str();
                    w == *p || (k + 1 == phrase.len() && w.strip_suffix('s') == Some(p))
                });
            if fits {
                items.push(SwitchItem {
                    start,
                    end: start + phrase.len(),
                    phrase: true,
                });
            }
        }
        if skip_identifier(&tokens[start]) {
            items.push(SwitchItem {
                start,
                end: start + 1,
                phrase: false,
            });
        }
    }
    items
}

/// Whether the item is negated: scanning back from its first token, every token passed is a bridge word or
/// part of another item, and a negator is reached before any other word.
fn item_is_negated(words: &[String], items: &[SwitchItem], item: &SwitchItem) -> bool {
    let mut i = item.start;
    while i > 0 {
        i -= 1;
        if items
            .iter()
            .any(|o| (o.start, o.end) != (item.start, item.end) && o.start <= i && i < o.end)
        {
            continue;
        }
        let word = words[i].as_str();
        if is_means_negator(word) {
            return true;
        }
        if PARTICIPLES.contains(&word) && i > 0 && crate::verify::is_negator(&words[i - 1]) {
            return true;
        }
        if !BRIDGE_WORDS.contains(&word) {
            return false;
        }
    }
    false
}

/// Whether one of the four words before the item states how the property is measured: `through`, `via`,
/// `using`, `by reading`, `by inspecting`, or `measured`, `checked`, `verified`, `tested` followed by `by` or
/// `with`.
fn has_means_phrase(words: &[String], start: usize) -> bool {
    let from = start.saturating_sub(4);
    (from..start).any(|i| {
        let w = words[i].as_str();
        let next = words.get(i + 1).map(String::as_str);
        matches!(w, "through" | "via" | "using")
            || (w == "by" && matches!(next, Some("reading" | "inspecting")))
            || (matches!(w, "measured" | "checked" | "verified" | "tested")
                && matches!(next, Some("by" | "with")))
    })
}

/// The skip switches and internal-helper reads in `text` that are not negated. In a direct check
/// (`strict`) every phrase and skip identifier counts; in a behavior criterion only a phrase counts, and only
/// after a means phrase.
pub(super) fn unnegated_switches(text: &str, strict: bool) -> Vec<String> {
    let mut found = Vec::new();
    for tokens in measure_segments(text) {
        let words: Vec<String> = tokens.iter().map(|t| token_word(t)).collect();
        let items = switch_items(&tokens);
        for item in &items {
            if !strict && (!item.phrase || !has_means_phrase(&words, item.start)) {
                continue;
            }
            if item_is_negated(&words, &items, item) {
                continue;
            }
            let shown = tokens[item.start..item.end].join(" ");
            let shown = shown
                .trim_matches(|c: char| matches!(c, '`' | ',' | '.'))
                .to_string();
            if !found.contains(&shown) {
                found.push(shown);
            }
        }
    }
    found
}

/// What is wrong with the `Output:` clause, if anything: an unknown schema tag or variant, or no accepted
/// record kind and no round-trip test deliverable.
fn record_kind_problem(reading: &Reading, output: &str) -> Option<String> {
    let lower = output.to_lowercase();
    let says_none = has_phrase(&lower, "no new output") || has_phrase(&lower, "no new record");
    let record_deliverables: Vec<&str> = reading
        .deliverables
        .iter()
        .filter(|d| is_record_or_data_deliverable(d))
        .map(|d| d.path.as_str())
        .collect();
    if says_none {
        return (!record_deliverables.is_empty()).then(|| {
            format!(
                "(c) the Output clause says no new output but the deliverables add or change a record or data file ({})",
                record_deliverables.join(", ")
            )
        });
    }
    let mut kinds = 0usize;
    let mut unknown: Vec<String> = Vec::new();
    for token in output.split_whitespace() {
        let token = token.trim_matches(|c: char| matches!(c, '`' | ',' | ';' | '.' | '(' | ')'));
        if token.starts_with("athena.") && token.contains('/') {
            if RECORD_KINDS.iter().any(|(_, _, tag)| *tag == token) {
                kinds += 1;
            } else {
                unknown.push(token.to_string());
            }
        } else if let Some((enum_name, variant)) = token.split_once("::")
            && matches!(enum_name, "AnyRecord" | "HistoryRecord")
        {
            if RECORD_KINDS
                .iter()
                .any(|(e, v, _)| *e == enum_name && *v == variant)
            {
                kinds += 1;
            } else {
                unknown.push(token.to_string());
            }
        } else if token == "check_record_file" {
            kinds += 1;
        }
    }
    if !unknown.is_empty() {
        return Some(format!(
            "(c) the Output clause names a record kind the records validation does not accept ({}); name a tag or variant of AnyRecord or HistoryRecord",
            unknown.join(", ")
        ));
    }
    if kinds > 0 {
        return None;
    }
    let tokens: Vec<&str> = output
        .split(|c: char| !(c.is_alphanumeric() || matches!(c, '_' | '-' | '/' | '.')))
        .map(|t| t.trim_end_matches('.'))
        .filter(|t| is_round_trip_token(t))
        .collect();
    if tokens.is_empty() {
        return Some(
            "(c) the Output clause names no accepted record kind (an `athena.<kind>/<n>` tag, `AnyRecord::<Variant>` or `HistoryRecord::<Variant>`), no round-trip test deliverable and does not say no new output"
                .to_string(),
        );
    }
    let named = tokens.iter().any(|t| {
        reading
            .acceptance_criteria
            .iter()
            .any(|c| c.text.contains(t))
    });
    let holder = reading.deliverables.iter().any(|d| {
        d.path.contains("/tests/") || d.path.ends_with("records.rs") || output.contains(&d.path)
    });
    if named && holder {
        return None;
    }
    Some(format!(
        "(c) the round-trip test ({}) is not named by a criterion together with a deliverable that can hold it (a tests/ path, a records.rs or a path in the Output clause)",
        tokens.join(", ")
    ))
}

/// Refuses a code or tests reading with a behavior criterion whose `Code survey:` does not state how the
/// change is built and measured, or states it against the code. It reads labelled clauses of the survey:
/// `Entry point:`, `Gate order:`, `Stop gate:`, `Reuses:` (or `Existing code:`), `Output:` and `Direct check:`.
///
/// The reading is refused when any of these is missing:
/// (a) the entry point (one or more identifiers), the gate order (at least two identifiers) and the stop gate
/// (one identifier of the gate order);
/// (b) a reused existing function (an identifier) or the words "no existing function";
/// (c) the record kind the change writes (an `athena.<kind>/<n>` tag, `AnyRecord::<Variant>` or
/// `HistoryRecord::<Variant>`, `check_record_file`), a round-trip test that a criterion names with a
/// deliverable that can hold it, or "no new output" when no record or data file is added;
/// (d) one `Direct check: criterion N` clause for each behavior criterion that is not routine, with a
/// measurement word, naming an entry point, a gate or a planner, producer or model call, and not measuring
/// through a skip switch or a read of an internal helper.
/// It also refuses what contradicts the code: a stop gate that is not in the gate order, a record kind that
/// is not in `AnyRecord` or `HistoryRecord`, "no new output" beside a record or data deliverable, and a skip
/// switch or internal-helper read as the measurement. A criterion that only says fmt, clippy and the tests
/// pass is routine and needs no clause.
///
/// Root cause this check removes: readings fixed tests that measured a property through a skip switch or an
/// internal helper, or invented a record kind the records validation rejects, because the gate order, the
/// functions to reuse and the way the property is measured were never stated before the criteria were
/// accepted. Jobs 20261028-055601-hg0n and 20261028-082151-67fa each failed that way, and introspections
/// 20261028-055651-dhh0 and 20261028-082240-77as traced both to it. [`check_code_survey`] asked only for a
/// `crates/` path. It sits directly after that check in the gate order and costs no model call.
///
/// Limits: matching is lexical and the check reads the reading, never the repository, so it cannot tell that
/// a named function does not exist or that a stated measurement is true; the critic and the verifier judge
/// that. Record kinds come from [`RECORD_KINDS`], a list pinned to the enums by a test; a pub `SCHEMA_TAGS`
/// constant in `athena-data` would replace it and is a follow-up. Skip-switch matching: a phrase (`skip
/// switch`, `internal helper`) or an identifier whose first part is `skip`, `skipped`, `no`, `disable`,
/// `disabled` or `bypass` (or an all-caps name with a `NO`, `SKIP`, `DISABLE` or `BYPASS` part) counts unless a
/// negator governs it through bridge words only, so a name such as `skip_whitespace` is refused in a direct
/// check unless negated and must be reworded or left out. Only `no_change` and `no_change_reason` are exempt.
/// A skip switch named in other words is not caught. A behavior criterion that describes a refusal fixture
/// with a means phrase ("through a skip switch") is refused and must be reworded.
///
/// This check refuses more readings; the higher refusal rate is intended. A replay of accepted readings
/// (`replay`) holds a job that newly refuses more than 10% unless the job's criteria declare a share. Declared
/// here: expected 30% of newly refused readings, because the check now asks for new required labels that past
/// readings never used. That is an estimate, not measured: past readings carry none of the labels, so the real
/// share may be higher, and the markers are not loosened to meet it. Queued proposed readings may need a
/// re-read.
///
/// The error names this check, each missing part by its letter, each contradiction, and ends with an example
/// survey (`CODE_SURVEY_EXAMPLE` in athena-context).
pub fn check_behavior_test_design(reading: &Reading) -> Result<()> {
    if !changes_code_with_behavior_criterion(reading) {
        return Ok(());
    }
    let parts = survey_parts(reading);
    let mut problems: Vec<String> = Vec::new();

    // (a) entry point, gate order and stop gate.
    let entry = parts.entry.as_deref().map(identifiers).unwrap_or_default();
    if entry.is_empty() {
        problems.push("(a) no `Entry point:` with a function name (`name` in backticks)".into());
    }
    let mut gates = parts.gates.as_deref().map(identifiers).unwrap_or_default();
    gates.dedup();
    let distinct: BTreeSet<&String> = gates.iter().collect();
    if distinct.len() < 2 {
        problems.push(
            "(a) no `Gate order:` naming at least two gates by function name, in run order".into(),
        );
    }
    match parts.stop.as_deref() {
        None => problems.push("(a) no `Stop gate:`".into()),
        Some(stop) => match identifiers(stop).first() {
            None => problems
                .push("(a) the `Stop gate:` is prose; name the gate by function name".into()),
            Some(name) if !gates.contains(name) => problems.push(format!(
                "contradiction: the stop gate `{name}` is not in the stated gate order"
            )),
            Some(_) => {}
        },
    }

    // (b) reuse.
    let reuse = reuse_text(reading, &parts);
    if identifiers(&reuse).is_empty() && !has_phrase(&reuse.to_lowercase(), "no existing function")
    {
        problems.push(
            "(b) no `Reuses:` naming an existing function by name, or the words 'no existing function'"
                .into(),
        );
    }

    // (c) record kind or round-trip deliverable.
    match parts.output.as_deref() {
        None => problems.push(
            "(c) no `Output:` clause: say no new output, name a record kind or name a round-trip test deliverable"
                .into(),
        ),
        Some(output) => {
            if let Some(problem) = record_kind_problem(reading, output) {
                problems.push(problem);
            }
        }
    }

    // (d) one direct check per non-routine behavior criterion.
    let needing: Vec<usize> = reading
        .acceptance_criteria
        .iter()
        .enumerate()
        .filter(|(_, c)| c.kind == CriterionKind::Behavior && !is_routine_criterion(&c.text))
        .map(|(i, _)| i + 1)
        .collect();
    let mut covered: BTreeSet<usize> = BTreeSet::new();
    for clause in &parts.directs {
        let cited = cited_criteria(clause);
        let label = match cited.as_slice() {
            [] if needing.len() == 1 => {
                covered.insert(needing[0]);
                format!("criterion {}", needing[0])
            }
            [] => "an unnumbered clause".to_string(),
            [n] => {
                covered.insert(*n);
                format!("criterion {n}")
            }
            many => {
                problems.push(format!(
                    "(d) one direct check covers {} criteria; write one Direct check clause per criterion",
                    many.len()
                ));
                continue;
            }
        };
        let mut faults: Vec<String> = Vec::new();
        if !has_measurement_word(clause) {
            faults.push("states no measurement (assert, count, compare, observe, measure, check, verify or expect)".into());
        }
        let names_entry = entry.iter().any(|n| contains_name(clause, n));
        let names_gate = gates.iter().any(|n| contains_name(clause, n));
        if !names_entry && !names_gate && !names_model_call(clause) {
            faults.push("names no entry point, gate or model call".into());
        }
        let switches = unnegated_switches(clause, true);
        if !switches.is_empty() {
            faults.push(format!(
                "measures through a skip switch or an internal helper ({}); cite the entry point or another gate, or reword",
                switches.join(", ")
            ));
        }
        if !faults.is_empty() {
            problems.push(format!(
                "(d) the direct check for {label} {}",
                faults.join(" and ")
            ));
        }
    }
    let uncovered: Vec<String> = needing
        .iter()
        .filter(|n| !covered.contains(n))
        .map(|n| n.to_string())
        .collect();
    if !uncovered.is_empty() {
        problems.push(format!(
            "(d) no `Direct check: criterion N` clause for behavior criterion {}",
            uncovered.join(", ")
        ));
    }
    for (i, c) in reading.acceptance_criteria.iter().enumerate() {
        if c.kind == CriterionKind::Behavior {
            let switches = unnegated_switches(&c.text, false);
            if !switches.is_empty() {
                problems.push(format!(
                    "(d) criterion {} measures through a skip switch or an internal helper ({}); measure through the entry point",
                    i + 1,
                    switches.join(", ")
                ));
            }
        }
    }

    if problems.is_empty() {
        return Ok(());
    }
    bail!(
        "the reading was refused because check_behavior_test_design found that the `Code survey:` of this {} reading does not state how the change is built and measured: {}. Under the labels `Entry point:`, `Gate order:`, `Stop gate:`, `Reuses:`, `Output:` and one `Direct check: criterion N` per behavior criterion, state each part. A criterion that only says fmt, clippy and test pass needs no Direct check. Example: {}",
        reading.class_of_work,
        problems.join("; "),
        athena_context::briefs::CODE_SURVEY_EXAMPLE
    )
}
