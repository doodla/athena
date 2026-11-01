//! A stored corpus of request and reading shapes, run through the reading gate (`refusing_check`, which
//! `check_reading_acceptance` wraps), and the mechanism that keeps every gate check covered by it.
//!
//! Root cause this file removes: refusal and matching rules (the parts parser, stems, stop lists and the
//! reading checks) were specified by reasoning and hand-checked on a few cases, and nothing executed them
//! against real request and reading shapes before they were planned or landed. Gaps (numbered `N.` and `N)`
//! parts, stop-list and stem order, a job cited but not relied on, a brand word read as a symbol) showed up
//! only in critic rounds; jobs 20261029-111202-2bpc and 20261030-060751-5389 each failed in planning that way.
//!
//! How the corpus relates to the replay in `athena_core::reading` (`replay_cases`): the replay runs the
//! newest accepted readings from `records/` through the base and the candidate checkers at verification and
//! holds a candidate that newly refuses more than its declared share. It sees accepted readings only, reads
//! `records/` (which a planner cannot see) and acts after the code is written. This corpus is its planning-time,
//! refuse-side complement: shapes live in this file, each with the outcome the gate must give, and a planner
//! maps the input forms of a new rule to these cases before any code exists.
//!
//! What the mechanism covers, and what it does not:
//! - Every `check_*` step in `refusing_check` must have a case in which that check refuses
//!   (`every_acceptance_check_has_a_refusing_corpus_case`), so a new reading check cannot land without a case.
//! - A new helper inside an existing check, or a change to the request parser (`is_bullet`,
//!   `request_text_acceptance`), a stem list or a stop list adds no step, so the mechanism cannot see it. Two
//!   things narrow that gap: every case pins its bullet count (`bullets_expected`), so a parser change moves a
//!   visible expectation; and `corpus_covers_the_listed_input_forms` fixes the list of input forms the corpus
//!   holds. The rest is the planner rule (prose only): map each input form of a rule to a case here.
//!
//! To add a case: write it in `cases()` with its source job, whether the text is `verbatim` or rebuilt from the
//! job brief, its form tags, its bullet count and the outcome the gate must give (accept, or refuse by the
//! named check). Build a refusing reading from `base_reading` and change one field, so an earlier check does
//! not refuse first. Fixture text states its source and holds no secrets or personal data.
//!
//! Run: `cargo test -p athena-core --test refusal_corpus`.

use anyhow::Error;
use athena_context::briefs::EarlierAttempt;
use athena_core::reading::{refusing_check, request_text_acceptance};
use athena_data::JobId;
use athena_data::records::{
    Criterion, CriterionKind, Deliverable, DeliverableAction, Forecast, Forecasts, Quantiles,
    Reading,
};
use std::collections::BTreeSet;

/// What the gate must do with a case.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Expected {
    Accept,
    /// Refused by the named check, the first to refuse.
    Refuse(&'static str),
}

/// One request and reading shape, with where it came from and the outcome the gate must give.
struct Case {
    id: &'static str,
    /// The job it came from, and `verbatim` or `rebuilt from the job brief`.
    source: &'static str,
    /// The input forms it holds: a subset of `INPUT_FORMS`.
    forms: &'static [&'static str],
    request: String,
    /// What the request parser (`request_text_acceptance`) must find in `request`.
    bullets_expected: usize,
    reading: Reading,
    attempt: Option<EarlierAttempt>,
    referenced: Vec<EarlierAttempt>,
    repo_files: Option<BTreeSet<String>>,
    expected: Expected,
}

/// The input forms the corpus must hold. A form no case carries fails `corpus_covers_the_listed_input_forms`;
/// a case tag outside this list fails it too, so a typo cannot hide a gap.
const INPUT_FORMS: [&str; 16] = [
    "dash-bullet",
    "star-bullet",
    "number-dot-parts",
    "number-paren-parts",
    "letter-paren-parts",
    "cites-job-without-relying",
    "follow-up-phrase",
    "retry-complaint",
    "governance-path",
    "full-repo-path",
    "crate-relative-path",
    "absent-repo-path",
    "brand-word-or-env-var",
    "producer-report-evidence",
    "code-survey",
    "design-existing-code",
];

const RETRIED_JOB: &str = "20261030-060751-5389";
const OTHER_JOB: &str = "20261029-111202-2bpc";

fn forecast() -> Forecast {
    Forecast {
        p_reading_accepted: 0.8,
        p_verification_passes: 0.7,
        p_user_intervenes: 0.1,
        cost_usd: Quantiles::new(0.2, 0.5, 1.5).unwrap(),
        wall_time_s: Quantiles::new(100.0, 300.0, 900.0).unwrap(),
    }
}

/// A reading that passes every check against `repo_files()`: a docs change to one existing file, with the
/// request's bullets as content criteria (one stock criterion when the request has none). A refusing case
/// changes one field of it.
fn base_reading(request: &str) -> Reading {
    let mut criteria: Vec<Criterion> = request_text_acceptance(request)
        .into_iter()
        .map(|text| Criterion {
            text,
            kind: CriterionKind::Content,
        })
        .collect();
    if criteria.is_empty() {
        criteria.push(Criterion {
            text: "The guide describes the change".into(),
            kind: CriterionKind::Content,
        });
    }
    Reading {
        job: JobId::parse("20261031-000000-aaaa").unwrap(),
        n: 1,
        at: jiff::Timestamp::UNIX_EPOCH,
        invocation_seq: 1,
        base_commit: "a".repeat(40),
        config_sha256: "b".repeat(64),
        defaults_sha256: "c".repeat(64),
        defaults_version: 1,
        title: "Corpus case".into(),
        summary: "Update the guide.".into(),
        class_of_work: "docs".into(),
        capability: Some("change".into()),
        decline_reason: None,
        producer_model: "producer".into(),
        verifier_model: "verifier".into(),
        defaults_applied: vec![],
        assumptions: vec![],
        question: None,
        acceptance_criteria: criteria,
        deliverables: vec![deliverable("docs/guide.md", DeliverableAction::Modify)],
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

fn deliverable(path: &str, action: DeliverableAction) -> Deliverable {
    Deliverable {
        path: path.into(),
        action,
        p: 0.9,
    }
}

/// The files at the base commit that the cases are checked against.
fn repo_files() -> BTreeSet<String> {
    [
        "docs/guide.md",
        "docs/design/roadmap.md",
        "crates/athena-core/src/reading.rs",
        "crates/athena-context/prompts/planner.md",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// A case that starts as an accepted docs change for `request`; the caller changes what the case is about.
fn case(
    id: &'static str,
    source: &'static str,
    forms: &'static [&'static str],
    request: &str,
    bullets_expected: usize,
    expected: Expected,
) -> Case {
    Case {
        id,
        source,
        forms,
        request: request.to_string(),
        bullets_expected,
        reading: base_reading(request),
        attempt: None,
        referenced: vec![],
        repo_files: Some(repo_files()),
        expected,
    }
}

fn lines(lines: &[&str]) -> String {
    lines.join("\n")
}

/// An earlier attempt of `job` whose candidate changed `crates/athena-core/src/reading.rs`.
fn earlier(job: &str, request: &str) -> EarlierAttempt {
    EarlierAttempt {
        job: job.into(),
        request: request.into(),
        end_state: "rejected".into(),
        changed_files: vec!["modified crates/athena-core/src/reading.rs".into()],
        ..EarlierAttempt::default()
    }
}

/// The request of a root-cause job that cites two rejected jobs as evidence. Rebuilt from the shape of
/// job 20261030-170757-gev8's request; the quoted parts are shortened.
fn root_cause_request(extra: &[&str]) -> String {
    let mut text = vec![
        "Priority: urgent",
        "",
        "Fix the root cause of a repeated failure: a rule is planned by reasoning and never run on real inputs.",
        "Seen 2 time(s) in 2 jobs. The introspections that record it:",
        "Introspection 20261029-181811-bs86 (job 20261029-111202-2bpc); prevention it proposed: list every input form.",
        "Introspection 20261030-170739-hq09 (job 20261030-060751-5389); prevention it proposed: add a planning gate.",
    ];
    text.extend_from_slice(extra);
    text.extend_from_slice(&[
        "",
        "Acceptance:",
        "- The root cause is named in the change description and fixed where it originates",
        "- A test fails without the change and passes with it",
        "- fmt, clippy and test pass",
    ]);
    lines(&text)
}

/// The stored corpus. Every request text is rebuilt from the job brief named in `source`, since the records
/// of those jobs are not part of the repository snapshot.
fn cases() -> Vec<Case> {
    let mut out = parser_cases();
    out.extend(request_citing_cases());
    out.extend(reading_check_cases());
    out
}

/// Request parser shapes, each with a reading the gate accepts.
fn parser_cases() -> Vec<Case> {
    vec![
        case(
            "steward-three-dash-bullets",
            "job 20261029-111202-2bpc, the Steward acceptance bullets; rebuilt from the job brief",
            &["dash-bullet"],
            &lines(&[
                "Make the steward digest list failed jobs first.",
                "",
                "Acceptance:",
                "- The weekly digest lists failed jobs before passed jobs",
                "- A job with no outcome is listed last",
                "- cargo test passes",
            ]),
            3,
            Expected::Accept,
        ),
        case(
            "star-bullets",
            "job 20261029-111202-2bpc, star-marked acceptance; rebuilt from the job brief",
            &["star-bullet"],
            &lines(&[
                "Document the status flag.",
                "* The guide names the flag",
                "* The guide gives one example",
            ]),
            2,
            Expected::Accept,
        ),
        case(
            "numbered-dot-parts",
            "job 20261029-111202-2bpc, numbered `1.` `2.` `3.` parts; rebuilt from the job brief",
            &["number-dot-parts"],
            &lines(&[
                "Do three things:",
                "1. Add a --json flag to athena status",
                "2. Document the flag in the guide",
                "3. Cover the flag with a test",
            ]),
            3,
            Expected::Accept,
        ),
        // The parser does not read `N)` or `(a)` parts today. The next two cases pin that behavior; they are a
        // known gap, and the parts-coverage follow-up changes these counts on purpose.
        case(
            "numbered-paren-parts-not-read-today",
            "job 20261029-111202-2bpc, `1)` parts; rebuilt from the job brief; current parser behavior, a known gap",
            &["number-paren-parts"],
            &lines(&[
                "Please do all of these:",
                "1) Add a --json flag to athena status",
                "2) Document the flag in the guide",
                "3) Cover the flag with a test",
            ]),
            0,
            Expected::Accept,
        ),
        case(
            "lettered-paren-parts-not-read-today",
            "job 20261029-111202-2bpc, `(a)` parts; rebuilt from the job brief; current parser behavior, a known gap",
            &["letter-paren-parts"],
            &lines(&[
                "Please do all of these:",
                "(a) Add a --json flag to athena status",
                "(b) Document the flag in the guide",
                "(c) Cover the flag with a test",
            ]),
            0,
            Expected::Accept,
        ),
    ]
}

/// Requests that cite earlier jobs, and retries. Citing a job as evidence is not following it up.
fn request_citing_cases() -> Vec<Case> {
    let mut out: Vec<Case> = Vec::new();
    let mut c = case(
        "root-cause-cites-two-jobs-without-relying",
        "job 20261030-170757-gev8, a root-cause request citing two rejected jobs; rebuilt from the job brief",
        &["dash-bullet", "cites-job-without-relying"],
        &root_cause_request(&[]),
        3,
        Expected::Accept,
    );
    c.referenced = vec![
        earlier(OTHER_JOB, "Add parts coverage to reading acceptance"),
        earlier(RETRIED_JOB, "Check that a cited prior job exists"),
    ];
    out.push(c);
    let mut c = case(
        "follow-up-on-a-job-the-reading-ignores",
        "job 20261030-170757-gev8's request plus a follow-up line; rebuilt from the job brief",
        &["dash-bullet", "follow-up-phrase"],
        &root_cause_request(&[&format!("Follow up on job {OTHER_JOB}.")]),
        3,
        Expected::Refuse("check_prior_result_and_scope"),
    );
    c.referenced = vec![
        earlier(OTHER_JOB, "Add parts coverage to reading acceptance"),
        earlier(RETRIED_JOB, "Check that a cited prior job exists"),
    ];
    out.push(c);

    // A retry complaint: the retried job is `attempt`, and the reading must account for it.
    let retry_request =
        "The last attempt was rejected: its rule was never run on a real request. Try again.";
    let mut c = case(
        "retry-complaint-reading-names-the-job",
        "job 20261030-060751-5389 retried; rebuilt from the job brief",
        &["retry-complaint"],
        retry_request,
        0,
        Expected::Accept,
    );
    c.attempt = Some(earlier(RETRIED_JOB, "Check that a cited prior job exists"));
    c.reading.summary = format!(
        "Retry of job {RETRIED_JOB}: the rule is re-planned with a corpus of real request shapes."
    );
    c.reading.deliverables = vec![deliverable(
        "crates/athena-core/src/reading.rs",
        DeliverableAction::Modify,
    )];
    c.reading.out_of_scope = vec!["The reader and verifier prompts are not changed".into()];
    out.push(c);
    let mut c = case(
        "retry-complaint-reading-omits-the-job",
        "job 20261030-060751-5389 retried, reading without the job id; rebuilt from the job brief",
        &["retry-complaint"],
        retry_request,
        0,
        Expected::Refuse("check_prior_result_and_scope"),
    );
    c.attempt = Some(earlier(RETRIED_JOB, "Check that a cited prior job exists"));
    c.reading.summary = "Re-plan the rule with a corpus of real request shapes.".into();
    c.reading.deliverables = vec![deliverable(
        "crates/athena-core/src/reading.rs",
        DeliverableAction::Modify,
    )];
    c.reading.out_of_scope = vec!["The reader and verifier prompts are not changed".into()];
    out.push(c);
    out
}

/// Cases for the checks on the reading's own text, criteria, survey and paths.
fn reading_check_cases() -> Vec<Case> {
    // The producer's own account as the evidence of a criterion.
    let evidence = case(
        "criterion-rests-on-the-producer-report",
        "job 20261027-004517-gk9e, a criterion resting on the producer's report; rebuilt from the job brief",
        &["dash-bullet", "producer-report-evidence"],
        &lines(&[
            "Add a caller check.",
            "- The producer's report lists the caller-check Grep",
        ]),
        1,
        Expected::Refuse("check_criteria_evidence"),
    );
    let mut out: Vec<Case> = vec![evidence];

    // A code reading with a behavior criterion needs a code survey that names a crates/ path.
    let code_request = lines(&[
        "Make the gate refuse a retry that omits its prior job.",
        "- A retry reading that omits the prior job id is refused by cargo test",
    ]);
    let mut c = case(
        "code-reading-without-a-survey",
        "job 20261028-055601-hg0n, a code reading with no survey; rebuilt from the job brief",
        &["dash-bullet", "code-survey"],
        &code_request,
        1,
        Expected::Refuse("check_code_survey"),
    );
    c.reading.class_of_work = "code".into();
    c.reading.acceptance_criteria[0].kind = CriterionKind::Behavior;
    out.push(c);
    let mut c = case(
        "code-reading-with-a-survey",
        "job 20261028-055601-hg0n, the same reading with a survey; rebuilt from the job brief",
        &["dash-bullet", "code-survey"],
        &code_request,
        1,
        Expected::Accept,
    );
    c.reading.class_of_work = "code".into();
    c.reading.acceptance_criteria[0].kind = CriterionKind::Behavior;
    c.reading.assumptions = vec![
        "Code survey: read crates/athena-core/src/reading.rs, the gate every reading passes".into(),
    ];
    out.push(c);

    // A design reading is written from request text alone, with no inventory of the existing code.
    let design_request = lines(&[
        "Design a code-health score for the repository.",
        "- The design says how the score is computed",
        "- The design says where the score is shown",
    ]);
    let design_source = "job 20261026-043137-zc0v, a design request; rebuilt from the job brief";
    let design = |id: &'static str, expected: Expected| {
        let mut c = case(
            id,
            design_source,
            &["dash-bullet", "design-existing-code"],
            &design_request,
            2,
            expected,
        );
        c.reading.class_of_work = "design".into();
        c.reading.deliverables = vec![deliverable(
            "docs/design/code-health-new.md",
            DeliverableAction::Create,
        )];
        c
    };
    out.push(design(
        "design-reading-without-existing-code",
        Expected::Refuse("check_design_existing_code"),
    ));
    let mut c = design(
        "design-reading-names-only-the-created-file",
        Expected::Refuse("check_design_existing_code"),
    );
    c.reading.assumptions = vec!["Existing code: docs/design/code-health-new.md".into()];
    out.push(c);
    let mut c = design(
        "design-reading-with-opposite-criterion",
        Expected::Refuse("check_design_existing_code"),
    );
    c.reading.assumptions =
        vec!["Existing code: crates/athena-core/src/reading.rs and docs/design/roadmap.md".into()];
    c.reading.acceptance_criteria.push(Criterion {
        text: "No survey needed: the existing code in section 3 is unchanged".into(),
        kind: CriterionKind::Content,
    });
    out.push(c);
    let mut c = design("design-reading-with-existing-code", Expected::Accept);
    c.reading.assumptions =
        vec!["Existing code: crates/athena-core/src/reading.rs and docs/design/roadmap.md".into()];
    c.reading.acceptance_criteria.push(Criterion {
        text: "The design has an Existing code section that reuses or extends \
               crates/athena-core/src/reading.rs"
            .into(),
        kind: CriterionKind::Content,
    });
    out.push(c);

    // A request that names a governance path.
    let governance_request = "Add a timeout setting that is read from athena.toml.";
    out.push(case(
        "request-names-a-governance-file-the-reading-ignores",
        "jobs 20261014-082328-g6vy and 20261015-014012-akwq, a request naming a configuration file; rebuilt from the job brief",
        &["governance-path"],
        governance_request,
        0,
        Expected::Refuse("check_governance_coverage"),
    ));
    let mut c = case(
        "request-names-a-governance-file-the-reading-excludes",
        "jobs 20261014-082328-g6vy and 20261015-014012-akwq, the same request with the file out of scope; rebuilt from the job brief",
        &["governance-path"],
        governance_request,
        0,
        Expected::Accept,
    );
    c.reading.out_of_scope = vec!["athena.toml is not changed".into()];
    out.push(c);

    // Deliverable paths are compared with the files at the base commit.
    let mut c = case(
        "crate-relative-deliverable-path",
        "job 20261030-060751-5389, a deliverable named relative to its crate; rebuilt from the job brief",
        &["crate-relative-path"],
        "Change the reading gate.",
        0,
        Expected::Refuse("check_boundary_and_repo_files"),
    );
    c.reading.deliverables = vec![deliverable("src/reading.rs", DeliverableAction::Modify)];
    out.push(c);
    let mut c = case(
        "full-deliverable-path",
        "job 20261030-060751-5389, the same deliverable with its full path; rebuilt from the job brief",
        &["full-repo-path"],
        "Change the reading gate.",
        0,
        Expected::Accept,
    );
    c.reading.deliverables = vec![deliverable(
        "crates/athena-core/src/reading.rs",
        DeliverableAction::Modify,
    )];
    out.push(c);

    // Repo paths the reading's text relies on.
    let mut c = case(
        "text-relies-on-an-absent-file",
        "job 20261026-042558-5p65, a summary relying on a file an earlier job was to add; rebuilt from the job brief",
        &["absent-repo-path"],
        "Check that a cited prior job exists.",
        0,
        Expected::Refuse("check_reading_references"),
    );
    c.reading.summary =
        "Extend crates/athena-core/src/prior_job_check.rs to refuse a retry.".into();
    out.push(c);
    let mut c = case(
        "text-says-the-file-does-not-exist-yet",
        "job 20261026-042558-5p65, the same summary saying the file is absent; rebuilt from the job brief",
        &["absent-repo-path"],
        "Check that a cited prior job exists.",
        0,
        Expected::Accept,
    );
    c.reading.summary =
        "crates/athena-core/src/prior_job_check.rs does not exist yet; this change creates it."
            .into();
    out.push(c);
    let mut c = case(
        "brand-word-and-placeholder-are-not-repo-paths",
        "job 20261030-060751-5389, GitHub/env-var and owner/repo tokens read as symbols; rebuilt from the job brief",
        &["brand-word-or-env-var", "full-repo-path"],
        "Publish the status page to GitHub.",
        0,
        Expected::Accept,
    );
    c.reading.summary =
        "Read the GitHub/env-var settings of owner/repo and update docs/design/roadmap.md.".into();
    out.push(c);

    out
}

/// The cases that fail their expectation, each as a line naming the case and its source.
fn mismatches(cases: &[Case]) -> Vec<String> {
    let mut found = Vec::new();
    for c in cases {
        let got = run(c);
        let ok = match (&c.expected, &got) {
            (Expected::Accept, None) => true,
            (Expected::Refuse(want), Some((name, _))) => want == name,
            _ => false,
        };
        if !ok {
            let seen = match &got {
                None => "accepted".to_string(),
                Some((name, error)) => format!("refused by {name}: {error}"),
            };
            found.push(format!(
                "{} ({}): expected {:?}, got {seen}",
                c.id, c.source, c.expected
            ));
        }
    }
    found
}

fn run(c: &Case) -> Option<(&'static str, Error)> {
    refusing_check(
        &c.reading,
        &c.request,
        c.attempt.as_ref(),
        &c.referenced,
        c.repo_files.as_ref(),
    )
}

#[test]
fn corpus_cases_get_the_expected_outcome() {
    let cases = cases();
    assert!(cases.len() >= 12, "the corpus holds {} cases", cases.len());
    let ids: BTreeSet<&str> = cases.iter().map(|c| c.id).collect();
    assert_eq!(ids.len(), cases.len(), "case ids are unique");
    for c in &cases {
        assert!(!c.source.is_empty(), "{} names its source", c.id);
        assert_eq!(
            request_text_acceptance(&c.request).len(),
            c.bullets_expected,
            "{} ({}): the request parser found another number of bullets",
            c.id,
            c.source
        );
    }
    let mismatches = mismatches(&cases);
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn the_wrapper_agrees_with_refusing_check_on_every_case() {
    for c in cases() {
        let wrapper = athena_core::reading::check_reading_acceptance(
            &c.reading,
            &c.request,
            c.attempt.as_ref(),
            &c.referenced,
            c.repo_files.as_ref(),
        );
        match (wrapper, run(&c)) {
            (Ok(()), None) => {}
            (Err(a), Some((_, b))) => {
                assert_eq!(a.to_string(), b.to_string(), "{}: error text", c.id)
            }
            (a, b) => panic!(
                "{} ({}): the gate and refusing_check disagree: {a:?} against {:?}",
                c.id,
                c.source,
                b.map(|(name, _)| name)
            ),
        }
    }
}

#[test]
fn corpus_covers_the_listed_input_forms() {
    let cases = cases();
    let known: BTreeSet<&str> = INPUT_FORMS.into_iter().collect();
    let mut held: BTreeSet<&str> = BTreeSet::new();
    for c in &cases {
        for form in c.forms {
            assert!(known.contains(form), "{}: unknown form tag {form}", c.id);
            held.insert(form);
        }
    }
    let missing: Vec<&&str> = known.difference(&held).collect();
    assert!(missing.is_empty(), "no corpus case holds: {missing:?}");
}

// ---------------------------------------------------------------------------------------------------------
// The mechanism: every check of the gate has a case in which it refuses.

/// The text of `reading.rs`, which holds the gate.
const READING_SOURCE: &str = include_str!("../src/reading.rs");

/// The body of the function that starts at `heading`: up to the first line that is a lone `}`.
fn function_body<'a>(source: &'a str, heading: &str) -> &'a str {
    let start = source
        .find(heading)
        .unwrap_or_else(|| panic!("{heading} not found in reading.rs"));
    let rest = &source[start..];
    let end = rest
        .find("\n}\n")
        .unwrap_or_else(|| panic!("the end of {heading} not found"));
    &rest[..end]
}

/// The names of the checks `refusing_check` runs: each `step!(check_name(` line of its body.
fn acceptance_check_names(source: &str) -> Vec<String> {
    let body = function_body(source, "pub fn refusing_check(");
    body.match_indices("step!(check_")
        .map(|(at, _)| {
            body[at + "step!(".len()..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect::<String>()
        })
        .collect()
}

/// The checks in `names` that no case refused by.
fn uncovered(names: &[String], covered: &BTreeSet<&str>) -> Vec<String> {
    names
        .iter()
        .filter(|name| !covered.contains(name.as_str()))
        .cloned()
        .collect()
}

/// The checks that refuse in at least one corpus case, found by running every case.
fn checks_that_refuse() -> BTreeSet<&'static str> {
    cases()
        .iter()
        .filter_map(|c| run(c).map(|(name, _)| name))
        .collect()
}

#[test]
fn every_acceptance_check_has_a_refusing_corpus_case() {
    let names = acceptance_check_names(READING_SOURCE);
    assert!(
        names.len() >= 6,
        "found only {names:?} in refusing_check; the extractor or the gate changed shape"
    );
    let unique: BTreeSet<&String> = names.iter().collect();
    assert_eq!(
        unique.len(),
        names.len(),
        "a check is listed twice: {names:?}"
    );
    let missing = uncovered(&names, &checks_that_refuse());
    assert!(
        missing.is_empty(),
        "no corpus case in which {missing:?} refuses; add a case to cases() in \
         crates/athena-core/tests/refusal_corpus.rs"
    );
}

#[test]
fn an_extra_check_with_no_case_is_reported() {
    let anchor = "    step!(check_reading_references(";
    assert!(READING_SOURCE.contains(anchor), "the anchor line moved");
    let doctored = READING_SOURCE.replacen(
        anchor,
        &format!("    step!(check_new_rule(reading));\n{anchor}"),
        1,
    );
    let names = acceptance_check_names(&doctored);
    assert_eq!(
        names.len(),
        acceptance_check_names(READING_SOURCE).len() + 1
    );
    assert_eq!(
        uncovered(&names, &checks_that_refuse()),
        vec!["check_new_rule".to_string()]
    );
}

#[test]
fn design_existing_code_refusal_is_declared_intended() {
    // The declared share is read from the criterion text by the replay.
    let declared = athena_core::reading::replay_declaration(
        &[Criterion {
            text: "The change refuses more readings: expected 5% of newly refused readings".into(),
            kind: CriterionKind::Behavior,
        }],
        &[],
    );
    assert_eq!(declared.share, Some(5));
    // The step line is in refusing_check, and the check's doc comment is one paragraph with the sentence.
    assert!(
        acceptance_check_names(READING_SOURCE).contains(&"check_design_existing_code".to_string())
    );
    let doc = READING_SOURCE
        .split("pub fn check_design_existing_code(")
        .next()
        .unwrap()
        .rsplit("\n\n")
        .next()
        .unwrap();
    assert!(
        doc.lines().all(|l| l.starts_with("///") && l != "///"),
        "{doc}"
    );
    let doc = doc
        .lines()
        .map(|l| l.trim_start_matches('/').trim())
        .collect::<Vec<_>>()
        .join(" ");
    for needle in [
        "This check refuses more readings; the higher refusal rate is intended.",
        "no inventory of existing code",
        "20261026-043137-zc0v",
        "20261026-065046-0c1d",
    ] {
        assert!(doc.contains(needle), "the doc comment lacks {needle:?}");
    }
    // The changelog wraps lines, so compare with runs of white space made single spaces.
    let changelog = include_str!("../../../CHANGELOG.md")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(changelog.contains("check_design_existing_code"));
    assert!(changelog.contains("higher refusal rate is intended"));
    assert!(changelog.contains("expected 5% of newly refused readings"));
}

#[test]
fn no_check_bypasses_refusing_check() {
    let body = function_body(READING_SOURCE, "pub fn check_reading_acceptance(");
    let (_, inside) = body
        .split_once(") -> Result<()> {\n")
        .expect("the signature of check_reading_acceptance");
    assert!(
        !inside.contains("check_"),
        "check_reading_acceptance calls a check directly; add it as a step! line in refusing_check:\n{inside}"
    );
    assert!(inside.contains("refusing_check("));
}
