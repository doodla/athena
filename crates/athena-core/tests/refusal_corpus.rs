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
//! Delivery requests (`check_outcome_state`): `delivery_cases` holds the positions a delivery verb takes in a
//! request and the words that are not delivery, and `delivery_verb_replay_over_the_cited_jobs` runs the recorded
//! request text of the three jobs the check cites through the gate: it refuses 1 of the 3, the real publish
//! misread, and the changelog must state that count. The other two are caught by `check_defined_terms`.
//!
//! Structural terms (`check_defined_terms`): `defined_terms_cases` holds requests that use `ladder`, `rung`,
//! `tier` or `layer` in plain prose (refused until the reading defines the term), requests that only quote or
//! cite those words (accepted), and requests built by Athena's own generators, `hestia::root_cause_request` and
//! `work_decisions::approve_request`, whose copied text is cited content. `defined_terms_replay_over_the_cited_jobs`
//! runs the three cited requests through the gate: 2 of the 3, the ladder request and the Hermes request, are
//! caught.
//!
//! Ambiguous ontology names (`check_defined_terms`): the `ambiguous-name-*` cases hold requests that use
//! `Hermes` (a name `docs/ontology.toml` records with two senses) in plain prose, which is refused until the
//! reading restates the name in a `Term Hermes:` assumption, and requests that only quote it or carry it in a
//! path, an identifier or Athena-generated text, which are accepted.
//!
//! Omitted acceptance bullets (`check_reading_coverage`): the `coverage-*` cases hold readings that leave out a
//! bullet under the request's `Acceptance:` heading (refused), readings that cover every bullet in words of their
//! own, including the recorded criteria of jobs 20261107-031902-ttnw and 20261109-161505-bp5e (accepted), and the
//! requests that allow a split, readings that ask or decline and requests with no heading (not checked).
//! `coverage_replay_over_the_corpus_and_recorded_shapes` runs them through the check alone.
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
use athena_core::store::{JobRecords, JobState, Records};
use athena_data::records::{
    Criterion, CriterionKind, Deliverable, DeliverableAction, Forecast, Forecasts,
    IntrospectionRecord, IntrospectionStatus, IntrospectionTrigger, PreventionKind, Quantiles,
    Reading, Reference, Request, WorkItem, WorkItemKind,
};
use athena_data::roles::Ladder;
use athena_data::{JobId, RecordId};
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
const INPUT_FORMS: [&str; 62] = [
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
    "boundary-word",
    "delivery-verb-first",
    "delivery-verb-after-lead-in",
    "delivery-word-as-noun",
    "delivery-verb-technical-use",
    "out-of-scope-names-delivery",
    "delivery-state-in-content-criterion",
    "ci-workflow-deliverable",
    "host-action-criterion",
    "records-branch-read",
    "records-evidence-citation",
    "environment-assumption",
    "trigger-word-inside-other-word",
    "pr-or-api-wording",
    "landing-history-rule",
    "realistic-branch-fixture",
    "landing-word-without-commit-shape",
    "landing-source-path-without-commit-shape",
    "structural-term-plain-prose",
    "structural-term-quoted-or-cited",
    "structural-term-defined",
    "structural-term-generated-approval",
    "ambiguous-name-plain-prose",
    "ambiguous-name-defined",
    "ambiguous-name-quoted-or-generated",
    "ci-cache-criterion",
    "ci-result-reuse-criterion",
    "cache-scope-assumption",
    "cache-word-in-product-or-path",
    "scope-limit-omits-default-file",
    "scope-limit-forbids-default-file",
    "scope-limit-config-without-consumers",
    "scope-limit-names-default-file",
    "scope-limit-consumer-not-a-deliverable",
    "scope-limit-proposal-apply",
    "scope-limit-bare-form",
    "coverage-bullet-omitted",
    "coverage-sibling-framing",
    "coverage-bullet-paraphrased",
    "coverage-prose-repeated-words",
    "coverage-split-allowed",
    "coverage-split-negated",
    "coverage-split-lookalike-prose",
    "coverage-wrapped-bullet",
    "coverage-no-acceptance-heading",
    "coverage-reading-asks-or-declines",
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

/// The stored corpus. Most request texts are rebuilt from the job brief named in `source`, since the records
/// of those jobs are not part of the repository snapshot; the delivery cases say in `source` which texts are
/// the recorded request, quoted verbatim, and which are rebuilt.
fn cases() -> Vec<Case> {
    let mut out = parser_cases();
    out.extend(request_citing_cases());
    out.extend(reading_check_cases());
    out.extend(delivery_cases());
    out.extend(defined_terms_cases());
    out.extend(scope_limit_cases());
    out.extend(coverage_cases());
    out
}

/// A change reading in the production shape for a request whose bullets are `criteria` after the `head` lines,
/// each bullet starting with `marker`. It lists `paths` and `CHANGELOG.md`, which Athena always adds, all of
/// them at the base commit, and carries `assumptions`.
fn scope_case(
    meta: (&'static str, &'static str, &'static [&'static str]),
    head: &[&str],
    marker: &str,
    criteria: &[&str],
    paths: &[&str],
    assumptions: &[&str],
    expected: Expected,
) -> Case {
    let mut text: Vec<String> = head.iter().map(|l| l.to_string()).collect();
    text.extend(criteria.iter().map(|c| format!("{marker}{c}")));
    let request = text.join("\n");
    let mut c = case(meta.0, meta.1, meta.2, &request, criteria.len(), expected);
    c.reading.deliverables = paths
        .iter()
        .chain(&["CHANGELOG.md"])
        .map(|p| deliverable(p, DeliverableAction::Modify))
        .collect();
    c.reading.assumptions = assumptions.iter().map(|a| a.to_string()).collect();
    let mut files = repo_files();
    files.extend(paths.iter().map(|p| p.to_string()));
    files.insert("CHANGELOG.md".into());
    c.repo_files = Some(files);
    c
}

/// Readings whose criterion limits which files may change. The refusing cases leave out a file the job must
/// change anyway: `CHANGELOG.md`, which Athena always writes, or the readers of a changed config.
fn scope_limit_cases() -> Vec<Case> {
    const SOURCE_9BW7: &str =
        "job 20261023-052543-9bw7, a limit that omits CHANGELOG.md; rebuilt from the job brief";
    const SOURCE_7Z81: &str = "job 20261018-045437-7z81, a limit that omits the tests that load athena.toml; rebuilt from the job brief";
    let head = ["Update the guide."];
    let proposal_head = [
        "apply defaults proposal 20261027-054151-ace0",
        "",
        "Deliverable: defaults.toml (modify). Make one change to it and touch no other file.",
    ];
    let guide = ["docs/guide.md"];
    let config = ["athena.toml", "docs/guide.md"];
    let limit = "Only athena.toml, docs/guide.md and CHANGELOG.md may change";
    let consumer = "crates/athena-cli/tests/common/mod.rs";
    let refuse = Expected::Refuse("check_scope_limit");
    vec![
        // Refused: the limit leaves out CHANGELOG.md.
        scope_case(
            (
                "scope-limit-only-may-change-omits-changelog",
                SOURCE_9BW7,
                &["dash-bullet", "scope-limit-omits-default-file"],
            ),
            &head,
            "- ",
            &["Only docs/guide.md may change"],
            &guide,
            &[],
            refuse.clone(),
        ),
        scope_case(
            (
                "scope-limit-no-file-outside-omits-changelog",
                SOURCE_9BW7,
                &["star-bullet", "scope-limit-omits-default-file"],
            ),
            &head,
            "* ",
            &["No file outside docs/guide.md changed"],
            &guide,
            &[],
            refuse.clone(),
        ),
        scope_case(
            (
                "scope-limit-numbered-part-omits-changelog",
                SOURCE_9BW7,
                &["number-dot-parts", "scope-limit-omits-default-file"],
            ),
            &head,
            "1. ",
            &["Only docs/guide.md may be changed"],
            &guide,
            &[],
            refuse.clone(),
        ),
        scope_case(
            (
                "scope-limit-test-only-change-omits-changelog",
                SOURCE_9BW7,
                &["scope-limit-omits-default-file"],
            ),
            &head,
            "- ",
            &["This is a test-only change"],
            &guide,
            &[],
            refuse.clone(),
        ),
        scope_case(
            (
                "scope-limit-only-the-docs-omits-changelog",
                SOURCE_9BW7,
                &["scope-limit-omits-default-file"],
            ),
            &head,
            "- ",
            &["Only the docs may change"],
            &guide,
            &[],
            refuse.clone(),
        ),
        scope_case(
            (
                "scope-limit-singular-deliverable-omits-changelog",
                SOURCE_9BW7,
                &["scope-limit-omits-default-file"],
            ),
            &head,
            "- ",
            &["Only the deliverable docs/guide.md may change"],
            &guide,
            &[],
            refuse.clone(),
        ),
        scope_case(
            (
                "scope-limit-names-changelog-but-forbids-it",
                SOURCE_9BW7,
                &["scope-limit-forbids-default-file"],
            ),
            &head,
            "- ",
            &["Only docs/guide.md may change; CHANGELOG.md is not changed"],
            &guide,
            &[],
            refuse.clone(),
        ),
        // Refused: a config deliverable with a limit and no answer for the files that load it.
        scope_case(
            (
                "scope-limit-config-deliverable-without-consumers",
                SOURCE_7Z81,
                &[
                    "dash-bullet",
                    "governance-path",
                    "scope-limit-config-without-consumers",
                ],
            ),
            &head,
            "- ",
            &[limit],
            &config,
            &[],
            refuse.clone(),
        ),
        scope_case(
            (
                "scope-limit-consumer-not-a-deliverable",
                SOURCE_7Z81,
                &["scope-limit-consumer-not-a-deliverable"],
            ),
            &head,
            "- ",
            &[limit],
            &config,
            &["Consumers: crates/athena-cli/tests/common/mod.rs loads athena.toml"],
            refuse.clone(),
        ),
        scope_case(
            (
                "scope-limit-consumer-not-a-deliverable-says-unchanged",
                SOURCE_7Z81,
                &["scope-limit-consumer-not-a-deliverable"],
            ),
            &head,
            "- ",
            &[limit],
            &config,
            &["Consumers: crates/athena-cli/tests/common/mod.rs must be updated; others unchanged"],
            refuse.clone(),
        ),
        scope_case(
            (
                "scope-limit-xtask-consumer-not-a-deliverable",
                SOURCE_7Z81,
                &["scope-limit-consumer-not-a-deliverable"],
            ),
            &head,
            "- ",
            &[limit],
            &config,
            &["Consumers: xtask/tests/policy.rs reads athena.toml"],
            refuse.clone(),
        ),
        scope_case(
            (
                "scope-limit-proposal-apply-without-consumers",
                "the request of a defaults-proposal job (`accept_request` in proposals.rs) with the reading its reader writes",
                &[
                    "dash-bullet",
                    "governance-path",
                    "scope-limit-proposal-apply",
                    "scope-limit-bare-form",
                ],
            ),
            &proposal_head,
            "- ",
            &[
                "defaults.toml carries the proposed entry",
                "No other file is changed",
            ],
            &["defaults.toml"],
            &[],
            refuse,
        ),
        // Accepted: the limit allows the files, or names no list.
        scope_case(
            (
                "scope-limit-bare-no-other-file",
                SOURCE_9BW7,
                &["scope-limit-bare-form"],
            ),
            &head,
            "- ",
            &["No other file is changed"],
            &guide,
            &[],
            Expected::Accept,
        ),
        scope_case(
            (
                "scope-limit-bare-beyond-formatting",
                SOURCE_9BW7,
                &["scope-limit-bare-form"],
            ),
            &head,
            "- ",
            &["No other file is changed beyond formatting"],
            &guide,
            &[],
            Expected::Accept,
        ),
        scope_case(
            (
                "scope-limit-earlier-only-then-outside-list",
                SOURCE_9BW7,
                &["scope-limit-names-default-file"],
            ),
            &head,
            "- ",
            &["Only maybe tidy things; no file outside docs/guide.md and CHANGELOG.md changed"],
            &guide,
            &[],
            Expected::Accept,
        ),
        scope_case(
            (
                "scope-limit-bare-production-unchanged",
                SOURCE_9BW7,
                &["scope-limit-bare-form"],
            ),
            &head,
            "- ",
            &["Production code is unchanged"],
            &guide,
            &[],
            Expected::Accept,
        ),
        scope_case(
            (
                "scope-limit-names-changelog",
                SOURCE_9BW7,
                &["scope-limit-names-default-file"],
            ),
            &head,
            "- ",
            &["Only docs/guide.md and CHANGELOG.md may change"],
            &guide,
            &[],
            Expected::Accept,
        ),
        scope_case(
            (
                "scope-limit-names-changelog-with-keeping",
                SOURCE_9BW7,
                &["scope-limit-names-default-file"],
            ),
            &head,
            "- ",
            &["Only docs/guide.md and CHANGELOG.md may change, keeping one entry"],
            &guide,
            &[],
            Expected::Accept,
        ),
        scope_case(
            (
                "scope-limit-points-to-deliverables",
                SOURCE_9BW7,
                &["scope-limit-names-default-file"],
            ),
            &head,
            "- ",
            &["Only the deliverables may change"],
            &guide,
            &[],
            Expected::Accept,
        ),
        scope_case(
            (
                "scope-limit-config-consumers-are-deliverables",
                SOURCE_7Z81,
                &["governance-path", "scope-limit-config-without-consumers"],
            ),
            &head,
            "- ",
            &[limit],
            &["athena.toml", "docs/guide.md", consumer],
            &["Consumers: crates/athena-cli/tests/common/mod.rs loads athena.toml"],
            Expected::Accept,
        ),
        scope_case(
            (
                "scope-limit-xtask-consumer-is-a-deliverable",
                SOURCE_7Z81,
                &["governance-path", "scope-limit-config-without-consumers"],
            ),
            &head,
            "- ",
            &[limit],
            &["athena.toml", "docs/guide.md", "xtask/tests/policy.rs"],
            &["Consumers: xtask/tests/policy.rs reads athena.toml"],
            Expected::Accept,
        ),
        scope_case(
            (
                "scope-limit-config-consumers-unchanged",
                SOURCE_7Z81,
                &["governance-path", "scope-limit-config-without-consumers"],
            ),
            &head,
            "- ",
            &[limit],
            &config,
            &["Consumers: tests that load athena.toml unchanged"],
            Expected::Accept,
        ),
        scope_case(
            (
                "scope-limit-config-consumers-none-found",
                SOURCE_7Z81,
                &["governance-path", "scope-limit-config-without-consumers"],
            ),
            &head,
            "- ",
            &[limit],
            &config,
            &["Consumers: none found, searched the repo for athena.toml reads"],
            Expected::Accept,
        ),
        scope_case(
            (
                "config-deliverable-without-a-limit",
                SOURCE_7Z81,
                &["governance-path"],
            ),
            &head,
            "- ",
            &["athena.toml has the new key"],
            &config,
            &[],
            Expected::Accept,
        ),
        scope_case(
            (
                "scope-limit-proposal-apply-with-consumers",
                "the request of a defaults-proposal job (`accept_request` in proposals.rs) with the reading its reader writes",
                &[
                    "dash-bullet",
                    "governance-path",
                    "scope-limit-proposal-apply",
                    "scope-limit-bare-form",
                ],
            ),
            &proposal_head,
            "- ",
            &[
                "defaults.toml carries the proposed entry",
                "No other file is changed",
            ],
            &["defaults.toml"],
            &["Consumers: none found"],
            Expected::Accept,
        ),
    ]
}

/// The recorded request of job 20261014-082328-g6vy, verbatim.
const G6VY_REQUEST: &str = "Publish a first explainer page on how firstmate handles work.";

/// A reading that says only that files exist, plus these behavior criteria.
fn with_behavior(mut c: Case, states: &[&str]) -> Case {
    for text in states {
        c.reading.acceptance_criteria.push(Criterion {
            text: (*text).into(),
            kind: CriterionKind::Behavior,
        });
    }
    c
}

/// Delivery requests (publish, deploy, share, release): the reading must name a state the user observes, or
/// put the delivery out of scope. The shapes are the positions a verb can take in a request, and the uses of
/// the four words that are not delivery.
fn delivery_cases() -> Vec<Case> {
    const G6VY: &str = "job 20261014-082328-g6vy, its recorded request, verbatim; the reading is rebuilt as the file-only reading it was given";
    const BRIEF: &str =
        "the request in the brief of job 20261107-094333-4z98; rebuilt, not a recorded request";
    let refuse = |id: &'static str,
                  source: &'static str,
                  forms: &'static [&'static str],
                  request: &str,
                  bullets: usize| {
        case(
            id,
            source,
            forms,
            request,
            bullets,
            Expected::Refuse("check_outcome_state"),
        )
    };
    let mut out: Vec<Case> = vec![
        refuse(
            "delivery-verb-first-reading-names-only-a-file",
            G6VY,
            &["delivery-verb-first"],
            G6VY_REQUEST,
            0,
        ),
        refuse(
            "delivery-verb-in-a-dash-bullet",
            BRIEF,
            &["dash-bullet", "delivery-verb-first"],
            &lines(&["Update the guide.", "- Publish the page"]),
            1,
        ),
        refuse(
            "delivery-verb-in-a-star-bullet",
            BRIEF,
            &["star-bullet", "delivery-verb-first"],
            &lines(&["Update the guide.", "* Share the report"]),
            1,
        ),
        refuse(
            "delivery-verb-in-a-numbered-dot-part",
            BRIEF,
            &["number-dot-parts", "delivery-verb-first"],
            &lines(&["Do this:", "1. Deploy the site"]),
            1,
        ),
        refuse(
            "delivery-verb-in-a-numbered-paren-part",
            BRIEF,
            &["number-paren-parts", "delivery-verb-first"],
            &lines(&["Please do this:", "2) Deploy the site"]),
            0,
        ),
        refuse(
            "delivery-verb-in-a-lettered-paren-part",
            BRIEF,
            &["letter-paren-parts", "delivery-verb-first"],
            &lines(&["Please do this:", "(a) Release v1"]),
            0,
        ),
        refuse(
            "delivery-verb-after-and",
            BRIEF,
            &["delivery-verb-after-lead-in"],
            "Build the docs and publish them.",
            0,
        ),
        refuse(
            "delivery-verb-after-can-you",
            BRIEF,
            &["delivery-verb-after-lead-in"],
            "Can you deploy the site?",
            0,
        ),
        refuse(
            "delivery-verb-after-want-to",
            BRIEF,
            &["delivery-verb-after-lead-in"],
            "I want to release v1.",
            0,
        ),
    ];

    // A criterion of kind content that mentions the state can be met by a Grep on a file, so it is not enough.
    let mut c = refuse(
        "delivery-state-in-a-content-criterion-refused",
        G6VY,
        &["delivery-state-in-content-criterion"],
        G6VY_REQUEST,
        0,
    );
    c.reading.acceptance_criteria.push(Criterion {
        text: "docs/explainer.md says the page is reachable".into(),
        kind: CriterionKind::Content,
    });
    out.push(c);

    // out_of_scope names the delivery only as a whole word: "shared" and "Release notes" do not.
    let mut c = refuse(
        "delivery-verb-out-of-scope-shared-helpers-refused",
        BRIEF,
        &["out-of-scope-names-delivery"],
        "Share the report with the team.",
        0,
    );
    c.reading.out_of_scope = vec!["Changes to shared helpers".into()];
    out.push(c);
    let mut c = refuse(
        "delivery-verb-out-of-scope-release-notes-refused",
        BRIEF,
        &["out-of-scope-names-delivery"],
        "Release v1.",
        0,
    );
    c.reading.out_of_scope = vec!["Release notes are unchanged".into()];
    out.push(c);

    // The governance refusal names the missing workflow and stays first.
    out.push(case(
        "delivery-request-naming-a-workflow-fails-governance-first",
        "job 20261014-082328-g6vy, a publish request naming the Pages workflow; rebuilt from the job brief",
        &["governance-path", "delivery-verb-first"],
        "Publish to GitHub Pages from .github/workflows/pages.yml",
        0,
        Expected::Refuse("check_governance_coverage"),
    ));

    // Accepted: a behavior criterion names the state, or the delivery is out of scope, or the word is not a
    // delivery verb.
    out.push(with_behavior(
        case(
            "delivery-verb-first-reading-names-a-served-state",
            G6VY,
            &["delivery-verb-first"],
            G6VY_REQUEST,
            0,
            Expected::Accept,
        ),
        &["The page is reachable at its public URL"],
    ));
    let mut c = case(
        "delivery-verb-out-of-scope-names-it-accepted",
        BRIEF,
        &["out-of-scope-names-delivery"],
        "Share the report with the team.",
        0,
        Expected::Accept,
    );
    c.reading.out_of_scope = vec!["Sharing the report is out of scope".into()];
    out.push(c);
    out.push(case(
        "delivery-word-as-a-noun-is-accepted",
        BRIEF,
        &["delivery-word-as-noun"],
        "Add release notes for 0.4 and a share button.",
        0,
        Expected::Accept,
    ));
    out.push(case(
        "delivery-verb-technical-use-is-accepted",
        BRIEF,
        &["delivery-verb-technical-use"],
        "Release the lock before awaiting. Workers should share the HTTP client. Publish the event to the \
         bus. Share the parsing code between crates. Release the image buffer before returning.",
        0,
        Expected::Accept,
    ));
    out
}

/// The recorded request of job 20261014-174815-7bf8, verbatim: it quotes "a ladder of requirements" from an
/// addendum and then uses the ladder in plain prose.
const REQUEST_7BF8: &str = "Build what the BRIEF.md addendum '2026-10-14: A loop that steers Athena, roles, and a ladder of \
     requirements' asks for, starting small: roles as recorded definitions (A9, A10), and a first role \
     that runs every 5 tasks in parallel with other work, looks at everything Athena has done, and queues \
     work items, including one for anything it could not see. Record only what would otherwise be lost; \
     pull the rest from where it lives. Use the ladder to decide what it may change itself.";

/// The recorded request of job 20261017-173508-0wtc, verbatim: it names Hermes Agent in plain prose and holds
/// none of the structural words.
const REQUEST_0WTC: &str = "From Sri's decision sheet of 2026-10-17 (relay note in BRIEF.md, addendum \"2026-10-17: Olympus\"), \
     decision 5, worded as on the sheet: Should Athena keep an ontology? A small shared vocabulary, grown \
     from misreads. Defined terms and how they relate, each with its source, in one file that agents read \
     and checks can use. A term is added when it causes a misread. The study keeps its own concept model, \
     so firstmate, Hermes Agent and Athena are compared in the same terms.";

/// The recorded requests of the three jobs the change cites, verbatim from their request records.
const CITED_REQUESTS: [(&str, &str); 3] = [
    ("20261014-082328-g6vy", G6VY_REQUEST),
    ("20261014-174815-7bf8", REQUEST_7BF8),
    ("20261017-173508-0wtc", REQUEST_0WTC),
];

/// A meaning of the Hermes term that is three words or more.
const HERMES_DEFINED: &str = "Term Hermes: the studied self-improving agent, not part of Athena";

#[test]
fn delivery_verb_replay_over_the_cited_jobs() {
    // Each reading is the file-only reading `base_reading` builds, so a request is refused by
    // `check_outcome_state` only when the request itself holds a delivery verb.
    let caught: Vec<&str> = CITED_REQUESTS
        .iter()
        .filter(|(_, request)| {
            matches!(
                refusing_check(
                    &base_reading(request),
                    request,
                    None,
                    &[],
                    Some(&repo_files())
                ),
                Some(("check_outcome_state", _))
            )
        })
        .map(|(job, _)| *job)
        .collect();
    let total = CITED_REQUESTS.len();
    // The check refuses exactly the real delivery-verb misread. The ladder-of-requirements request and the
    // ontology request hold no delivery verb: they are another class, which `check_defined_terms` covers for
    // both (see `defined_terms_replay_over_the_cited_jobs`).
    assert_eq!(
        caught,
        ["20261014-082328-g6vy"],
        "caught {caught:?} of {total}"
    );
    for (job, request) in CITED_REQUESTS {
        let outcome = refusing_check(
            &base_reading(request),
            request,
            None,
            &[],
            Some(&repo_files()),
        );
        if job == "20261014-082328-g6vy" {
            let (name, error) = outcome.expect("the publish request is refused");
            assert_eq!(name, "check_outcome_state");
            assert!(error.to_string().contains("reachable URL"), "{error}");
        } else if job == "20261014-174815-7bf8" {
            let (name, error) = outcome.expect("the ladder request is refused");
            assert_eq!(name, "check_defined_terms");
            assert!(error.to_string().contains("ladder"), "{error}");
        } else {
            let (name, error) = outcome.expect("the ontology request is refused");
            assert_eq!(name, "check_defined_terms");
            assert!(error.to_string().contains("Hermes"), "{error}");
        }
    }
    // The changelog states the same count and does not claim the other two for this check.
    let changelog = include_str!("../../../CHANGELOG.md")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let claim = format!("covers {} of the {} cited jobs", caught.len(), total);
    assert!(changelog.contains(&claim), "the changelog lacks {claim:?}");
    assert!(changelog.contains("are not caught by this check"));
    for job in CITED_REQUESTS.iter().map(|(job, _)| job) {
        assert!(changelog.contains(job), "the changelog does not name {job}");
    }
    assert!(!changelog.contains("3 of the 3 cited jobs"));
}

#[test]
fn defined_terms_replay_over_the_cited_jobs() {
    // Each reading is the file-only reading `base_reading` builds, with no `Term` line, so a request is
    // refused by `check_defined_terms` only when its plain prose uses a structural word or an ambiguous name.
    let caught: Vec<&str> = CITED_REQUESTS
        .iter()
        .filter(|(_, request)| {
            matches!(
                refusing_check(
                    &base_reading(request),
                    request,
                    None,
                    &[],
                    Some(&repo_files())
                ),
                Some(("check_defined_terms", _))
            )
        })
        .map(|(job, _)| *job)
        .collect();
    assert_eq!(
        caught,
        ["20261014-174815-7bf8", "20261017-173508-0wtc"],
        "caught {caught:?} of 3"
    );
    // The ontology request holds none of the four structural words, so it is caught for the ambiguous name
    // alone: Hermes (Hermes Agent, the studied system, or the Orchestration god) used in plain prose with no
    // `Term Hermes:` line.
    let ontology = REQUEST_0WTC;
    for word in ontology
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
    {
        assert!(
            !["ladder", "rung", "tier", "layer"].contains(&word),
            "the ontology request holds {word:?}"
        );
    }
    let refused = refusing_check(
        &base_reading(ontology),
        ontology,
        None,
        &[],
        Some(&repo_files()),
    );
    let (name, error) = refused.expect("the ontology request is refused with no Term Hermes line");
    assert_eq!(name, "check_defined_terms");
    let error = error.to_string();
    assert!(
        error.contains("Hermes") && error.contains("`Term Hermes:"),
        "{error}"
    );
    // The same reading with a meaning of three words or more is accepted.
    let mut defined = base_reading(ontology);
    defined.assumptions = vec![HERMES_DEFINED.into()];
    assert!(
        refusing_check(&defined, ontology, None, &[], Some(&repo_files())).is_none(),
        "a Term Hermes line of three or more words is accepted"
    );
    // A one-word meaning does not define it.
    let mut short = base_reading(ontology);
    short.assumptions = vec!["Term Hermes: agent".into()];
    let (name, _) = refusing_check(&short, ontology, None, &[], Some(&repo_files()))
        .expect("a one-word meaning is refused");
    assert_eq!(name, "check_defined_terms");
    // The changelog states the same count and does not claim the other two.
    let changelog = include_str!("../../../CHANGELOG.md")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let claim = format!(
        "check_defined_terms covers {} of the 3 cited jobs",
        caught.len()
    );
    assert!(changelog.contains(&claim), "the changelog lacks {claim:?}");
    for job in CITED_REQUESTS.iter().map(|(job, _)| job) {
        assert!(changelog.contains(job), "the changelog does not name {job}");
    }
    assert!(!changelog.contains("3 of the 3 cited jobs"));
}

#[test]
fn defined_terms_decision_and_changelog() {
    let decisions = include_str!("../../../docs/decisions.md");
    let (_, d164) = decisions
        .split_once("## D164 · ")
        .expect("docs/decisions.md holds D164");
    let d164: String = d164.split_whitespace().collect::<Vec<_>>().join(" ");
    for needle in [
        "**Alternatives:**",
        "**Provenance:**",
        "check_defined_terms",
        "readings are not checked against the outcome the user observes, so structural terms stay undefined",
        "replay record at verification",
        "20261014-174815-7bf8",
        "20261017-173508-0wtc",
    ] {
        assert!(d164.contains(needle), "D164 lacks {needle:?}");
    }
    let changelog = include_str!("../../../CHANGELOG.md")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for needle in [
        "reading::check_defined_terms",
        "readings are not checked against the outcome the user observes, so structural terms stay undefined",
        "replay record at verification",
    ] {
        assert!(changelog.contains(needle), "the changelog lacks {needle:?}");
    }
}

// ---------------------------------------------------------------------------------------------------------
// Structural terms (`check_defined_terms`). The accepted generated requests are built by the real generators,
// so a rewording in `hestia.rs` or `work_decisions.rs` shows here.

/// A recorded introspection `n` about job `of`, whose prevention text is `prevention`.
fn introspection(n: u32, of: char, prevention: &str) -> IntrospectionRecord {
    IntrospectionRecord {
        id: RecordId::parse(&format!("20261027-09{:02}{:02}-1abc", n / 60, n % 60)).unwrap(),
        at: format!("2026-10-27T09:{:02}:{:02}Z", n / 60, n % 60)
            .parse()
            .unwrap(),
        trigger: IntrospectionTrigger::Rejection,
        job: Some(cause_job(of)),
        intervention: None,
        class_of_work: "code".into(),
        category: "rejected".into(),
        what_happened: Some("the job was rejected".into()),
        failure_class: None,
        new_class: false,
        earlier_cases: Vec::new(),
        decisions_matched: Vec::new(),
        cause: Some("the ladder has no rung in this layer".into()),
        prevention_kind: Some(PreventionKind::CodeCheck),
        prevention: Some(prevention.into()),
        proposed_request: Some("Add the check".into()),
        evidence: None,
        status: IntrospectionStatus::Recorded,
        failure: None,
        flags: Vec::new(),
        dropped_ids: Vec::new(),
        planning_stopped: false,
        from_jev: false,
        jev: None,
    }
}

fn cause_job(letter: char) -> JobId {
    JobId::parse(&format!(
        "20261027-0800{:02}-{letter}bcd",
        letter as u32 - 'a' as u32
    ))
    .unwrap()
}

/// The two earlier jobs the generated root-cause request cites as evidence.
fn cause_jobs_referenced() -> Vec<EarlierAttempt> {
    vec![
        earlier(
            cause_job('a').as_str(),
            "Add parts coverage to reading acceptance",
        ),
        earlier(
            cause_job('b').as_str(),
            "Check that a cited prior job exists",
        ),
    ]
}

/// A root-cause request built by `hestia::root_cause_request` for a cause recorded by two introspections
/// whose cause, class and prevention text use `ladder`, `layer` and `tier`. `sentence` is added to the
/// request's own first paragraph.
fn generated_root_cause_request(sentence: Option<&str>) -> String {
    let first = introspection(
        10,
        'a',
        "Add a ladder check for every layer; a tier list is wrong.",
    );
    let second = introspection(20, 'b', "Name each rung of the ladder in the reading.");
    let cause = athena_core::hestia::RepeatedCause {
        key: "cause:ladder".into(),
        class: Some("Ladder of tiers (20261027-090500-1abc)".into()),
        cause: "the ladder has no rung in this layer".into(),
        introspections: vec![first.id.to_string(), second.id.to_string()],
        jobs: vec![
            cause_job('a').as_str().to_string(),
            cause_job('b').as_str().to_string(),
        ],
        request: None,
    };
    let records = Records {
        introspections: vec![first, second],
        ..Records::default()
    };
    let text = athena_core::hestia::root_cause_request(&cause, &records).unwrap();
    match sentence {
        None => text,
        Some(sentence) => text.replacen(
            "do not work around one instance.",
            &format!("do not work around one instance. {sentence}"),
            1,
        ),
    }
}

/// An approved-work request built by `work_decisions::approve_request` for a work item titled `title`, whose
/// source is `docs/guide.md` holding `source_text`, in a temporary repository.
fn generated_approval_request(title: &str, source_text: &str) -> String {
    let dir = tempfile::tempdir().unwrap();
    let git = athena_git::Git::at(dir.path());
    git.run(["init", "-q", "-b", "main"]).unwrap();
    for (key, value) in [
        ("user.name", "Test User"),
        ("user.email", "noreply@example.com"),
        ("commit.gpgsign", "false"),
        ("core.hooksPath", "/dev/null"),
    ] {
        git.run(["config", key, value]).unwrap();
    }
    std::fs::create_dir_all(dir.path().join("docs")).unwrap();
    std::fs::write(dir.path().join("docs/guide.md"), source_text).unwrap();
    git.run(["add", "-A"]).unwrap();
    git.run(["commit", "-q", "-m", "Start"]).unwrap();
    let head = git.run(["rev-parse", "HEAD"]).unwrap().trim().to_string();
    let item = WorkItem {
        id: RecordId::parse("20261015-003903-khz6").unwrap(),
        at: "2026-10-15T00:00:00Z".parse().unwrap(),
        role: "steward".into(),
        kind: WorkItemKind::Proposal,
        key: "k".into(),
        title: title.into(),
        rung: Ladder::DesignDecisions,
        source: Reference::path("docs/guide.md"),
        evidence: vec![Reference::path("docs/")],
        brief: None,
    };
    athena_core::work_decisions::approve_request(&item, &Records::default(), &git, &head).unwrap()
}

#[test]
fn generated_request_has_the_generators_shapes() {
    // `check_defined_terms` skips these lines by shape; this fails if `hestia.rs` rewords one.
    let text = generated_root_cause_request(None);
    for needle in [
        "The same cause has produced failures in more than one job.",
        "\nFailure class: ",
        "\nCause: ",
        "Fix the root cause of a repeated failure: ",
    ] {
        assert!(text.contains(needle), "{needle:?} is not in:\n{text}");
    }
    let introspections: Vec<&str> = text
        .lines()
        .filter(|l| l.starts_with("Introspection "))
        .collect();
    assert_eq!(introspections.len(), 2, "{text}");
    for line in introspections {
        assert!(line.contains(" (job "), "{line}");
        assert!(line.contains("); prevention it proposed: "), "{line}");
    }
    // The words the check must not count are in the generated text.
    let lower = text.to_lowercase();
    for word in ["ladder", "layer", "tier", "rung"] {
        assert!(lower.contains(word), "{word} is not in:\n{text}");
    }
}

#[test]
fn approve_request_has_the_generators_shapes() {
    let text = generated_approval_request(
        "Change the thing",
        "The ladder has five rungs in this layer.\n",
    );
    assert!(text.starts_with("Change the thing\n\n"), "{text}");
    for needle in [
        "A person approved this proposal from the steward role.",
        "\nRung of the requirements ladder: design-decisions.\n",
        "\n## Source\n",
        "The ladder has five rungs in this layer.",
    ] {
        assert!(text.contains(needle), "{needle:?} is not in:\n{text}");
    }
    let proposal = text
        .lines()
        .find(|l| l.starts_with("Proposal: "))
        .unwrap_or_else(|| panic!("no Proposal line in:\n{text}"));
    assert!(proposal.contains(" (key k)"), "{proposal}");
    assert!(proposal.contains(" queued at "), "{proposal}");
    // The copied source comes after the heading, and the rung line before it.
    let source = text.find("\n## Source\n").unwrap();
    assert!(text.find("Rung of the requirements ladder: ").unwrap() < source);
    assert!(text.find("The ladder has five rungs").unwrap() > source);
}

/// Cases for `check_defined_terms`: the ladder request of job 20261014-174815-7bf8 and its companions.
fn defined_terms_cases() -> Vec<Case> {
    const SRC_7BF8: &str = "job 20261014-174815-7bf8, its recorded request, verbatim; the reading is the file-only reading `base_reading` builds";
    const OWN: &str = "this retry, job 20261107-182419-pg2d; text built by the real generator";
    const BRIEF: &str =
        "the request in the brief of job 20261107-182419-pg2d; rebuilt, not a recorded request";
    let refuse = "check_defined_terms";
    let mut out: Vec<Case> = Vec::new();

    // The recorded request uses the ladder in plain prose and its reading defines nothing.
    out.push(case(
        "structural-term-ladder-undefined-in-7bf8-request",
        SRC_7BF8,
        &["structural-term-plain-prose"],
        REQUEST_7BF8,
        0,
        Expected::Refuse(refuse),
    ));
    // The same reading with a definition that cites the file, and a behavior criterion that names the ladder.
    let mut c = with_behavior(
        case(
            "structural-term-ladder-defined-and-named-by-a-behavior-criterion-accepted",
            SRC_7BF8,
            &["structural-term-defined"],
            REQUEST_7BF8,
            0,
            Expected::Accept,
        ),
        &["Run `cargo test -p athena-core`: a requirement on the ladder is ranked by its rung"],
    );
    c.reading.assumptions = vec![
        "Term ladder: the ordered levels of requirements, from the owner's goals down to the code, as docs/guide.md lists them"
            .into(),
    ];
    out.push(c);

    // A root-cause request built by the real generator: the words are in earlier introspections' lines, in a
    // parenthesised list and in quotes only, so the reading needs no Term line.
    let quoted_only = generated_root_cause_request(Some(
        "Keep the names (ladder, layer, tier) and the words \"ladder\" and 'tier' as they are.",
    ));
    let mut c = case(
        "structural-term-quoted-or-cited-mentions-are-accepted",
        OWN,
        &[
            "dash-bullet",
            "cites-job-without-relying",
            "structural-term-quoted-or-cited",
        ],
        &quoted_only,
        3,
        Expected::Accept,
    );
    c.referenced = cause_jobs_referenced();
    out.push(c);
    // The same request with one plain use beside the quoted ones.
    let mut c = case(
        "structural-term-plain-use-beside-quoted-mentions-is-refused",
        OWN,
        &[
            "dash-bullet",
            "cites-job-without-relying",
            "structural-term-plain-prose",
        ],
        &generated_root_cause_request(Some(
            "Keep the names (ladder, layer, tier). Also fix the ladder.",
        )),
        3,
        Expected::Refuse(refuse),
    );
    c.referenced = cause_jobs_referenced();
    out.push(c);
    // The corpus copy of a root-cause request: an Introspection line that uses the words is cited content.
    let mut c = case(
        "structural-term-introspection-line-in-the-corpus-copy-is-accepted",
        "job 20261030-170757-gev8's request plus an introspection line that uses the words; rebuilt from the job brief",
        &["dash-bullet", "structural-term-quoted-or-cited"],
        &root_cause_request(&[
            "Introspection 20261031-000000-abcd (job 20261029-111202-2bpc); prevention it proposed: a ladder of tiers for each layer.",
        ]),
        3,
        Expected::Accept,
    );
    c.referenced = vec![
        earlier(OTHER_JOB, "Add parts coverage to reading acceptance"),
        earlier(RETRIED_JOB, "Check that a cited prior job exists"),
    ];
    out.push(c);

    // Uses in plain prose are refused in every form: possessive, a priority tier, a caching layer. The last
    // two pin a known, intended false refusal: a technical use of the word is refused until a Term line.
    for (id, source, request) in [
        (
            "structural-term-possessive-use-is-refused",
            BRIEF,
            "Move the ladder's order into the guide.",
        ),
        (
            "structural-term-priority-tier-is-refused",
            "a known, intended false refusal: Athena's priority tiers are a technical use of the word; rebuilt, not a recorded request",
            "Order the work items by priority tier.",
        ),
        (
            "structural-term-caching-layer-is-refused",
            "a known, intended false refusal: a caching layer is a technical use of the word; rebuilt, not a recorded request",
            "Add a caching layer to the guide builder.",
        ),
    ] {
        out.push(case(
            id,
            source,
            &["structural-term-plain-prose"],
            request,
            0,
            Expected::Refuse(refuse),
        ));
    }
    // The words inside other words are not uses.
    out.push(case(
        "structural-term-inside-other-words-is-accepted",
        BRIEF,
        &["trigger-word-inside-other-word"],
        "Rename the stepladder helper and remove the delayer module.",
        0,
        Expected::Accept,
    ));
    out.push(case(
        "structural-term-no-structural-word-is-accepted",
        BRIEF,
        &["structural-term-plain-prose"],
        "Update the guide so that it lists the status flag.",
        0,
        Expected::Accept,
    ));

    // The request text of an approved work item, built by the real generator: the rung line and the copied
    // source hold the words and are cited content.
    let approval = generated_approval_request(
        "Change the thing",
        "The ladder has five rungs in this layer.\n",
    );
    out.push(case(
        "structural-term-approve-request-generated-text-is-accepted",
        OWN,
        &["structural-term-generated-approval"],
        &approval,
        0,
        Expected::Accept,
    ));
    // The title is the steward's own wording, and a use in it counts.
    out.push(case(
        "structural-term-approve-request-title-use-is-refused",
        OWN,
        &[
            "structural-term-generated-approval",
            "structural-term-plain-prose",
        ],
        &generated_approval_request("Fix the ladder ordering", "A note.\n"),
        0,
        Expected::Refuse(refuse),
    ));
    // The blind-spot request the steward queues is built by Athena and holds none of the words.
    out.push(case(
        "structural-term-blind-spot-request-is-accepted",
        "athena_core::steward::blind_spot_change, the public builder of the steward's blind-spot request",
        &["dash-bullet", "structural-term-generated-approval"],
        &athena_core::steward::blind_spot_change("jobs", "the job index is cut").request,
        3,
        Expected::Accept,
    ));
    out.extend(ambiguous_name_cases());
    out
}

/// Cases for the ambiguous names of `docs/ontology.toml` that `check_defined_terms` also covers: Hermes, the
/// name of the studied agent and of the Orchestration god, as in the request of job 20261017-173508-0wtc.
fn ambiguous_name_cases() -> Vec<Case> {
    const SRC_0WTC: &str = "job 20261017-173508-0wtc, its recorded request, verbatim; the reading is the file-only reading `base_reading` builds";
    const OWN: &str = "this job, 20261107-200727-xwss; text built by the real generator";
    const BRIEF: &str =
        "the request in the brief of job 20261107-200727-xwss; rebuilt, not a recorded request";
    let refuse = "check_defined_terms";
    let mut out: Vec<Case> = Vec::new();

    // The recorded request names Hermes Agent in plain prose and its reading defines nothing.
    out.push(case(
        "ambiguous-name-hermes-undefined-in-0wtc-request",
        SRC_0WTC,
        &["ambiguous-name-plain-prose"],
        REQUEST_0WTC,
        0,
        Expected::Refuse(refuse),
    ));
    // The same reading with a `Term Hermes:` line of three or more words is accepted.
    let mut c = case(
        "ambiguous-name-hermes-defined-is-accepted",
        SRC_0WTC,
        &["ambiguous-name-defined"],
        REQUEST_0WTC,
        0,
        Expected::Accept,
    );
    c.reading.assumptions = vec![HERMES_DEFINED.into()];
    out.push(c);

    // Other uses in plain prose are refused: a possessive, a bullet, a lowercase spelling.
    out.push(case(
        "ambiguous-name-possessive-and-lowercase-use-is-refused",
        BRIEF,
        &["ambiguous-name-plain-prose"],
        "Compare Hermes's skills with Athena's.",
        0,
        Expected::Refuse(refuse),
    ));
    out.push(case(
        "ambiguous-name-lowercase-use-is-refused",
        BRIEF,
        &["ambiguous-name-plain-prose"],
        "Fix the hermes note in the guide.",
        0,
        Expected::Refuse(refuse),
    ));
    out.push(case(
        "ambiguous-name-in-acceptance-bullet-is-refused",
        BRIEF,
        &["dash-bullet", "ambiguous-name-plain-prose"],
        &lines(&[
            "Update the guide.",
            "- Hermes Agent is described in the guide",
        ]),
        1,
        Expected::Refuse(refuse),
    ));

    // Quotes, paths, identifiers and other words are not uses.
    out.push(case(
        "ambiguous-name-quoted-path-and-identifier-are-accepted",
        BRIEF,
        &["ambiguous-name-quoted-or-generated"],
        "Rename \"Hermes\" in the guide, then fix docs/hermes.md and Hermes::new.",
        0,
        Expected::Accept,
    ));
    out.push(case(
        "ambiguous-name-inside-other-words-is-accepted",
        BRIEF,
        &[
            "trigger-word-inside-other-word",
            "ambiguous-name-quoted-or-generated",
        ],
        "Rename the hermeshub helper.",
        0,
        Expected::Accept,
    ));

    // The request text of an approved work item, built by the real generator: the copied source is cited
    // content, and the title is the steward's own wording.
    out.push(case(
        "ambiguous-name-approve-request-source-is-accepted",
        OWN,
        &[
            "structural-term-generated-approval",
            "ambiguous-name-quoted-or-generated",
        ],
        &generated_approval_request("Change the thing", "Hermes Agent writes its own skills.\n"),
        0,
        Expected::Accept,
    ));
    out.push(case(
        "ambiguous-name-approve-request-title-use-is-refused",
        OWN,
        &[
            "structural-term-generated-approval",
            "ambiguous-name-plain-prose",
        ],
        &generated_approval_request("Compare Hermes with Athena", "A note.\n"),
        0,
        Expected::Refuse(refuse),
    ));
    // An Introspection line of a root-cause request is cited content.
    let mut c = case(
        "ambiguous-name-introspection-line-is-accepted",
        "job 20261030-170757-gev8's request plus an introspection line that names Hermes; rebuilt from the job brief",
        &["dash-bullet", "ambiguous-name-quoted-or-generated"],
        &root_cause_request(&[
            "Introspection 20261031-000000-abcd (job 20261029-111202-2bpc); prevention it proposed: compare Hermes with Athena.",
        ]),
        3,
        Expected::Accept,
    );
    c.referenced = vec![
        earlier(OTHER_JOB, "Add parts coverage to reading acceptance"),
        earlier(RETRIED_JOB, "Check that a cited prior job exists"),
    ];
    out.push(c);
    out
}

#[test]
fn ambiguous_name_changelog_and_decision() {
    let changelog = include_str!("../../../CHANGELOG.md")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for needle in [
        "readings are not checked against the outcome the user observes, so structural terms stay undefined, and so do ambiguous ontology names such as Hermes",
        "20261014-082328-g6vy",
        "20261014-174815-7bf8",
        "20261017-173508-0wtc",
        "check_defined_terms covers 2 of the 3 cited jobs",
        "covers 1 of the 3 cited jobs",
        "`check_outcome_state` and `check_governance_coverage` cover 20261014-082328-g6vy",
        "are not caught by this check",
        "AMBIGUOUS_NAMES",
        "UNGATED_TERMS",
        "`Term Hermes: <meaning of at least three words>`",
        "expected 2% of newly refused readings",
    ] {
        assert!(changelog.contains(needle), "the changelog lacks {needle:?}");
    }
    assert!(!changelog.contains("3 of the 3 cited jobs"));
    assert!(!changelog.contains("20261017-173508-0wtc by no check"));
    let decisions = include_str!("../../../docs/decisions.md");
    let (_, d165) = decisions
        .split_once("## D165 · ")
        .expect("docs/decisions.md holds D165");
    let d165: String = d165.split_whitespace().collect::<Vec<_>>().join(" ");
    for needle in [
        "**Alternatives:**",
        "**Provenance:**",
        "revises D164's scope sentence",
        "readings are not checked against the outcome the user observes",
        "expected 2% of newly refused readings",
        "cannot verify",
        "UNGATED_TERMS",
        "20261014-082328-g6vy",
        "20261014-174815-7bf8",
        "20261017-173508-0wtc",
    ] {
        assert!(d165.contains(needle), "D165 lacks {needle:?}");
    }
}

/// A proposed request of the user's queue: its stored reading is `base_reading`, with these assumptions.
fn proposed_records(entries: &[(&str, &str, &[&str])]) -> Records {
    let mut records = Records::default();
    for (job_id, text, assumptions) in entries {
        let job = JobId::parse(job_id).unwrap();
        let mut reading = base_reading(text);
        reading.job = job.clone();
        reading.assumptions = assumptions.iter().map(|a| a.to_string()).collect();
        records.jobs.insert(
            job_id.to_string(),
            JobRecords {
                id: Some(job.clone()),
                request: Some(Request {
                    job,
                    text: text.to_string(),
                    received_at: "2026-10-16T12:00:00Z".parse().unwrap(),
                    channel: athena_core::steward::USER_QUEUE_CHANNEL.into(),
                }),
                readings: vec![reading],
                ..Default::default()
            },
        );
    }
    records
}

#[test]
fn queued_work_holds_a_proposed_request_whose_stored_reading_leaves_a_term_undefined() {
    const BASE: &str = "Add a greeting.\n\nAcceptance:\n- the greeting prints\n\nFiles: src/a.rs";
    let with_tier = format!("{BASE}\n\nShow it in the top tier.");
    let records = proposed_records(&[
        ("20261016-120000-aaaa", &with_tier, &[]),
        (
            "20261016-120001-bbbb",
            &with_tier,
            &["Term tier: the priority tier a work item sits in."],
        ),
        ("20261016-120002-cccc", BASE, &[]),
    ]);
    for job in records.jobs.values() {
        assert_eq!(job.state(), JobState::Proposed, "{}", job.id());
    }
    let queued = athena_core::steward::queued_work(&records);
    let listed: Vec<(&str, bool)> = queued.iter().map(|w| (w.job.as_str(), w.read)).collect();
    // The request that uses `tier` with no definition is held out until a person runs `athena go`; the
    // one that defines it, and the control with no structural word, are listed read.
    assert_eq!(
        listed,
        [
            ("20261016-120001-bbbb", true),
            ("20261016-120002-cccc", true)
        ],
        "{listed:?}"
    );
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

    // A code reading with a rule on commit classification, the history check or the attribution check needs a
    // behavior criterion that names a realistic-branch fixture. Each source job below is rebuilt from its brief:
    // the deliverable lists and the nature of the rule, not the original reading text.
    let survey =
        "Code survey: read crates/athena-core/src/reading.rs, the gate every reading passes";
    let fixture_text = "Run `cargo test -p athena-cli --test reconcile`: on a realistic-branch fixture with an \
                        intervention commit, a records-only commit and the job landing, the rule classifies \
                        each as intended";
    let history_case = |id: &'static str,
                        source: &'static str,
                        forms: &'static [&'static str],
                        request: &str,
                        paths: &[&str],
                        expected: Expected| {
        let mut c = case(id, source, forms, request, 1, expected);
        c.reading.class_of_work = "code".into();
        c.reading.acceptance_criteria[0].kind = CriterionKind::Behavior;
        c.reading.assumptions = vec![survey.into()];
        c.reading.deliverables = paths
            .iter()
            .map(|p| deliverable(p, DeliverableAction::Modify))
            .collect();
        // The land, gh and commits sources are not in the small file set of the corpus.
        c.repo_files = None;
        c
    };
    out.push(history_case(
        "landing-rule-on-job-commits-only-job-1rve",
        "job 20261024-153908-1rve, a pr_commits classification tested on job commits; rebuilt from the job brief",
        &["dash-bullet", "landing-history-rule"],
        &lines(&[
            "Classify pull request commits by content.",
            "- Run `cargo test -p athena-core land::`: pr_commits classifies the job commits by content",
        ]),
        &[
            "crates/athena-core/src/land.rs",
            "crates/athena-core/src/gh.rs",
            "crates/athena-gate/src/invariant.rs",
        ],
        Expected::Refuse("check_history_fixture"),
    ));
    out.push(history_case(
        "commit-shape-rule-without-a-fixture-job-e65v",
        "job 20261024-154552-e65v, a merge method that follows the commit shape; rebuilt from the job brief",
        &["dash-bullet", "landing-history-rule"],
        &lines(&[
            "Make the reconcile merge method follow the commit shape.",
            "- Run `cargo test -p athena-core land::`: the merge method follows the commit shape of the branch",
        ]),
        &[
            "crates/athena-core/src/land.rs",
            "crates/athena-gate/src/invariant.rs",
        ],
        Expected::Refuse("check_history_fixture"),
    ));
    out.push(history_case(
        "trailer-cutover-reading-without-a-fixture-job-hac3",
        "job 20261025-012657-hac3, a trailer cutover; rebuilt from the job brief",
        &["dash-bullet", "landing-history-rule"],
        &lines(&[
            "Stop writing Athena trailers after the cutover commit.",
            "- Run `cargo test -p athena-cli --test lifecycle`: no Athena trailers are written after C0",
        ]),
        &[
            "crates/athena-core/src/commits.rs",
            "crates/athena-git/src/attrs.rs",
        ],
        Expected::Refuse("check_history_fixture"),
    ));
    let mut c = history_case(
        "landing-rule-with-a-realistic-branch-fixture",
        "job 20261024-153908-1rve, the same rule with a realistic-branch fixture criterion; rebuilt from the job brief",
        &[
            "dash-bullet",
            "landing-history-rule",
            "realistic-branch-fixture",
        ],
        &lines(&[
            "Classify pull request commits by content.",
            "- Run `cargo test -p athena-core land::`: pr_commits classifies the job commits by content",
        ]),
        &["crates/athena-core/src/land.rs"],
        Expected::Accept,
    );
    c.reading.acceptance_criteria.push(Criterion {
        text: fixture_text.into(),
        kind: CriterionKind::Behavior,
    });
    out.push(c);
    out.push(history_case(
        "landing-queue-reading-is-not-refused",
        "the landing queue and CI wait jobs, which name landing but no commit shape; rebuilt from the job brief",
        &["dash-bullet", "landing-word-without-commit-shape"],
        &lines(&[
            "Land jobs first in, first out.",
            "- Run `cargo test -p athena-core gh::`: a job waits for the early CI run and lands in queue order",
        ]),
        &["crates/athena-core/src/gh/landing.rs"],
        Expected::Accept,
    ));
    out.push(history_case(
        "land-path-deliverable-without-a-commit-shape-rule",
        "the dispatch-branch job, which changed land.rs with no commit-shape rule; rebuilt from the job brief",
        &["dash-bullet", "landing-source-path-without-commit-shape"],
        &lines(&[
            "Land a job only on the branch it was dispatched from.",
            "- Run `cargo test -p athena-core land::`: a job lands only on the branch it was dispatched from",
        ]),
        &["crates/athena-core/src/land.rs"],
        Expected::Accept,
    ));
    let mut c = case(
        "docs-reading-naming-the-history-check",
        "job 20261024-220048-engm, a docs reading that names the history check; rebuilt from the job brief",
        &["dash-bullet", "landing-history-rule"],
        &lines(&[
            "Describe the history check.",
            "- The history check reads the job records on the records branch",
        ]),
        1,
        Expected::Accept,
    );
    c.reading.acceptance_criteria[0].kind = CriterionKind::Content;
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
    // A publish request: its reading also names the served state, so only the brand-word rule is under test.
    let c = with_behavior(c, &["The status page is reachable at its GitHub Pages URL"]);
    out.push(c);

    out.extend(environment_cases());
    out.extend(boundary_word_cases());
    out
}

/// Cases for `check_environment_reach`: an action or a source the reading assigns to a place must come with an
/// `Environment:` assumption (the place, its permissions, the data it reaches), and a summary that cites job
/// records or history needs quoted figures or a reachable file. Shapes are those of jobs 20261024-220048-engm
/// (a records read from a read-only place), 20261025-070538-8ra0 (a push and pull request with no origin) and
/// 20261025-034247-nm0k (records cited without figures); rebuilt from the job briefs.
fn environment_cases() -> Vec<Case> {
    const ENGM: &str = "job 20261024-220048-engm, a records read from a place that cannot reach them; rebuilt from the job brief";
    const EIGHT_RA0: &str = "job 20261025-070538-8ra0, a push and pull request met by a working copy with no origin; rebuilt from the job brief";
    const NM0K: &str = "job 20261025-034247-nm0k, records and history cited without figures; rebuilt from the job brief";
    const CI: &str = "Environment: the CI job has write permission to refs on the GitHub origin";
    const LOCAL: &str = "Environment: the local Athena working copy has write access to the repo; with no origin \
                         or a non-GitHub origin it skips the push; with a GitHub origin it can push refs";
    let request = "Review the guide.";
    let refuse = "check_environment_reach";
    let mut out: Vec<Case> = Vec::new();
    let mut add = |id: &'static str,
                   source: &'static str,
                   forms: &'static [&'static str],
                   expected: Expected,
                   change: &dyn Fn(&mut Case)| {
        let mut c = case(id, source, forms, request, 0, expected);
        change(&mut c);
        out.push(c);
    };
    let workflow = |c: &mut Case| {
        c.reading.deliverables.push(deliverable(
            ".github/workflows/lint.yml",
            DeliverableAction::Create,
        ));
        c.reading.acceptance_criteria.push(Criterion {
            text: ".github/workflows/lint.yml runs the lint job".into(),
            kind: CriterionKind::Content,
        });
    };
    let behavior = |c: &mut Case, text: &str| {
        c.reading.acceptance_criteria.push(Criterion {
            text: text.into(),
            kind: CriterionKind::Behavior,
        });
    };
    let assume = |c: &mut Case, text: &str| c.reading.assumptions.push(text.into());

    add(
        "environment-ci-workflow-without-an-environment",
        ENGM,
        &["ci-workflow-deliverable"],
        Expected::Refuse(refuse),
        &|c| workflow(c),
    );
    add(
        "environment-ci-workflow-with-an-environment",
        ENGM,
        &["ci-workflow-deliverable", "environment-assumption"],
        Expected::Accept,
        &|c| {
            workflow(c);
            assume(c, CI);
        },
    );
    add(
        "environment-host-action-without-an-environment",
        EIGHT_RA0,
        &["host-action-criterion"],
        Expected::Refuse(refuse),
        &|c| behavior(c, "Athena pushes the job branch to origin"),
    );
    add(
        "environment-host-action-with-origin-cases",
        EIGHT_RA0,
        &["host-action-criterion", "environment-assumption"],
        Expected::Accept,
        &|c| {
            behavior(c, "Athena pushes the job branch to origin");
            assume(c, LOCAL);
        },
    );
    add(
        "environment-pr-without-an-environment",
        EIGHT_RA0,
        &["host-action-criterion", "pr-or-api-wording"],
        Expected::Refuse(refuse),
        &|c| behavior(c, "Athena opens the job's PR at verification start"),
    );
    add(
        "environment-github-api-without-an-environment",
        EIGHT_RA0,
        &["pr-or-api-wording"],
        Expected::Refuse(refuse),
        &|c| behavior(c, "The check calls the GitHub API for the run status"),
    );
    add(
        "environment-ci-write-without-an-environment",
        ENGM,
        &["pr-or-api-wording"],
        Expected::Refuse(refuse),
        &|c| behavior(c, "The CI job writes the records"),
    );
    add(
        "environment-records-branch-read-without-an-environment",
        ENGM,
        &["records-branch-read"],
        Expected::Refuse(refuse),
        &|c| {
            behavior(
                c,
                "The history check reads the job records on the records branch",
            )
        },
    );
    add(
        "environment-records-branch-read-with-an-environment",
        ENGM,
        &["records-branch-read", "environment-assumption"],
        Expected::Accept,
        &|c| {
            behavior(
                c,
                "The history check reads the job records on the records branch",
            );
            assume(
                c,
                "Environment: the CI job has read-only access to contents and cannot write the records \
                 branch; the producer snapshot has no records directory",
            );
        },
    );
    add(
        "environment-records-summary-without-figures",
        NM0K,
        &["records-evidence-citation"],
        Expected::Refuse(refuse),
        &|c| c.reading.summary = "Use the figures from job records.".into(),
    );
    add(
        "environment-bare-history-summary-without-figures",
        NM0K,
        &["records-evidence-citation"],
        Expected::Refuse(refuse),
        &|c| c.reading.summary = "Use the figures from history.".into(),
    );
    add(
        "environment-records-summary-with-a-bare-year",
        NM0K,
        &["records-evidence-citation", "environment-assumption"],
        Expected::Refuse(refuse),
        &|c| {
            c.reading.summary = "Use the figures from job records.".into();
            assume(c, "Environment: the figures date from 2026");
        },
    );
    add(
        "environment-records-summary-with-quoted-figures",
        NM0K,
        &["records-evidence-citation"],
        Expected::Accept,
        &|c| {
            c.reading.summary = "Use the figures from job records.".into();
            assume(c, "The figures cover 3 jobs, 40% of them landed");
        },
    );
    add(
        "environment-records-summary-with-a-reachable-file",
        NM0K,
        &["records-evidence-citation", "environment-assumption"],
        Expected::Accept,
        &|c| {
            c.reading.summary = "Use the figures from job records.".into();
            assume(
                c,
                "Environment: the figures are listed in docs/design/roadmap.md",
            );
        },
    );
    add(
        "environment-product-reads-records-accepted",
        ENGM,
        &["records-branch-read"],
        Expected::Accept,
        &|c| {
            behavior(
                c,
                "Run `athena status`: rounds per landed job are computed from the job records",
            );
            behavior(c, "Scores a job from its records");
        },
    );
    add(
        "environment-trigger-words-only-inside-other-words",
        EIGHT_RA0,
        &["dash-bullet", "trigger-word-inside-other-word"],
        Expected::Accept,
        &|c| {
            behavior(c, "Fixed where it originates");
            behavior(c, "`parse_remote` returns None for a local path");
            behavior(c, "Entries are pushed onto the list");
            behavior(c, "A Vec push appends the entry");
        },
    );
    add(
        "environment-gh-module-path-only",
        EIGHT_RA0,
        &["trigger-word-inside-other-word"],
        Expected::Accept,
        &|c| behavior(c, "gh/early.rs is not changed"),
    );
    add(
        "environment-pr-inside-paths-only",
        EIGHT_RA0,
        &["trigger-word-inside-other-word"],
        Expected::Accept,
        &|c| {
            behavior(c, "pr.rs is not changed");
            behavior(c, "The page lists the history of the page");
            behavior(c, "The record is written in the records branch");
        },
    );
    // CI caches, stored artifacts and CI results reused across refs: the reading must say which ref saves the
    // entry, which runs restore it and which runs may write shared entries.
    const TWO_AM0: &str = "job 20261025-034731-2am0, a result cache keyed by a fingerprint with no ref scope; rebuilt from the job brief";
    const ER96: &str = "job 20261025-055626-er96, an early push whose CI run is reused at landing; rebuilt from the job brief";
    const CACHE_SCOPE: &str = "Environment: the CI cache is saved by the main branch push run, restored by pull \
                               request runs of the same branch and by main, written only by main";
    const RESULT_SCOPE: &str = "Environment: the CI result is saved by the early push run on the job branch, \
                                restored by the landing run of the same branch and by main, written only by main";
    let ci_workflow = |c: &mut Case| {
        c.reading.deliverables.push(deliverable(
            ".github/workflows/ci.yml",
            DeliverableAction::Create,
        ));
        c.reading.acceptance_criteria.push(Criterion {
            text: ".github/workflows/ci.yml runs the build".into(),
            kind: CriterionKind::Content,
        });
        c.reading.acceptance_criteria.push(Criterion {
            text: "ci.yml saves a result cache keyed by the build fingerprint".into(),
            kind: CriterionKind::Behavior,
        });
    };
    let fingerprint = "CI restores a result cache keyed by the fingerprint of the build paths";
    add(
        "environment-cache-workflow-without-cache-scope",
        TWO_AM0,
        &["ci-workflow-deliverable", "environment-assumption"],
        Expected::Refuse(refuse),
        &|c| {
            ci_workflow(c);
            assume(c, CI);
        },
    );
    add(
        "environment-cache-workflow-with-cache-scope",
        TWO_AM0,
        &["ci-workflow-deliverable", "cache-scope-assumption"],
        Expected::Accept,
        &|c| {
            ci_workflow(c);
            assume(c, CI);
            assume(c, CACHE_SCOPE);
        },
    );
    add(
        "environment-cache-criterion-without-cache-scope",
        TWO_AM0,
        &["ci-cache-criterion"],
        Expected::Refuse(refuse),
        &|c| behavior(c, fingerprint),
    );
    add(
        "environment-cache-criterion-with-cache-scope",
        TWO_AM0,
        &["ci-cache-criterion", "cache-scope-assumption"],
        Expected::Accept,
        &|c| {
            behavior(c, fingerprint);
            assume(c, CACHE_SCOPE);
        },
    );
    add(
        "environment-actions-cache-criterion-without-cache-scope",
        TWO_AM0,
        &["ci-cache-criterion"],
        Expected::Refuse(refuse),
        &|c| {
            behavior(
                c,
                "The build uses `actions/cache@v4` keyed by the lock file",
            )
        },
    );
    add(
        "environment-ci-result-reuse-without-cache-scope",
        ER96,
        &["ci-result-reuse-criterion"],
        Expected::Refuse(refuse),
        &|c| behavior(c, "An early push whose CI run is reused at landing"),
    );
    add(
        "environment-ci-result-reuse-with-cache-scope",
        ER96,
        &["ci-result-reuse-criterion", "cache-scope-assumption"],
        Expected::Accept,
        &|c| {
            behavior(c, "An early push whose CI run is reused at landing");
            assume(c, RESULT_SCOPE);
        },
    );
    add(
        "environment-ci-result-reuse-by-sha-accepted",
        ER96,
        &["ci-result-reuse-criterion"],
        Expected::Accept,
        &|c| behavior(c, "A green CI run for the candidate SHA is reused"),
    );
    add(
        "environment-cache-assumption-without-write-rights",
        TWO_AM0,
        &["cache-scope-assumption", "environment-assumption"],
        Expected::Refuse(refuse),
        &|c| {
            behavior(c, fingerprint);
            assume(c, CI);
            assume(
                c,
                "Environment: the CI cache is saved by the main branch push run and restored by pull request \
                 runs of the same branch",
            );
        },
    );
    add(
        "environment-cache-word-in-product-code-accepted",
        TWO_AM0,
        &["cache-word-in-product-or-path"],
        Expected::Accept,
        &|c| {
            behavior(c, "The reader caches the survey in memory");
            behavior(c, "CI runs the tests");
        },
    );
    add(
        "environment-cache-word-inside-a-path-accepted",
        TWO_AM0,
        &["trigger-word-inside-other-word"],
        Expected::Accept,
        &|c| {
            behavior(c, "cache.rs is not changed");
            behavior(c, "`cache_dir` is kept");
        },
    );
    add(
        "environment-github-api-response-cached-accepted",
        TWO_AM0,
        &["pr-or-api-wording"],
        Expected::Accept,
        &|c| behavior(c, "The GitHub API response is cached in memory"),
    );
    add(
        "environment-corpus-runner-cache-accepted",
        TWO_AM0,
        &["cache-word-in-product-or-path"],
        Expected::Accept,
        &|c| behavior(c, "The corpus runner caches the readings"),
    );
    // The dash-bullet request names a push and a PR in a content criterion, which the check does not read.
    let bullet_request = lines(&["Update the guide.", "- Push to origin when the PR is ready"]);
    let mut c = case(
        "environment-trigger-words-only-inside-other-words-request",
        EIGHT_RA0,
        &["dash-bullet", "trigger-word-inside-other-word"],
        &bullet_request,
        1,
        Expected::Accept,
    );
    behavior(&mut c, "Fixed where it originates");
    out.push(c);
    out
}

/// A boundary word, built from pieces so this file passes the boundary scan it exercises.
fn banned_word() -> String {
    ["oper", "ator"].concat()
}

/// Where a boundary word is refused: a deliverable path, or an acceptance criterion of kind `content`.
/// Where it is not: prose in the record, such as the summary or an assumption.
fn boundary_word_cases() -> Vec<Case> {
    const SOURCE: &str = "request of job 20261031-215030-b0nv citing refusals on 2026-10-31 and 2026-10-30; rebuilt from the request";
    let word = banned_word();
    let request = "Stop refusing a reading over a boundary word in its prose.";
    let mut out = Vec::new();
    let mut c = case(
        "boundary-word-in-the-summary",
        SOURCE,
        &["boundary-word"],
        request,
        0,
        Expected::Accept,
    );
    c.reading.summary = format!("Ask the {word} to read the guide.");
    out.push(c);
    let mut c = case(
        "boundary-word-in-an-assumption",
        SOURCE,
        &["boundary-word"],
        request,
        0,
        Expected::Accept,
    );
    c.reading.assumptions = vec![format!("The {word} reads the guide.")];
    out.push(c);
    let mut c = case(
        "boundary-word-in-a-deliverable-path",
        SOURCE,
        &["boundary-word"],
        request,
        0,
        Expected::Refuse("check_boundary_and_repo_files"),
    );
    c.reading.deliverables = vec![deliverable(
        &format!("docs/{word}.md"),
        DeliverableAction::Create,
    )];
    out.push(c);
    let mut c = case(
        "boundary-word-in-a-content-criterion",
        SOURCE,
        &["boundary-word"],
        request,
        0,
        Expected::Refuse("check_boundary_and_repo_files"),
    );
    c.reading.acceptance_criteria.push(Criterion {
        text: format!("docs/guide.md says: ask the {word} to read it"),
        kind: CriterionKind::Content,
    });
    out.push(c);
    out
}

/// The Acceptance bullets of job 20261107-031902-ttnw, verbatim from its recorded request.
const TTNW_BULLETS: [&str; 5] = [
    "a test: with six person-filed urgent requests and two Steward requests queued, the first three dispatches include at least one Steward request",
    "a test: a Hestia request whose failure class appears in three distinct jobs is ordered as urgent",
    "a test: with no improvement requests queued, ordering is unchanged",
    "athena plan prints which items take the improvement turn",
    "all tests pass; CHANGELOG states only what the diff does",
];

/// The criteria of the recorded reading of job 20261107-031902-ttnw, verbatim. Each bullet is covered in
/// the reading's own words, for example `ordering is unchanged` as `the plan order is identical`.
const TTNW_CRITERIA: [&str; 8] = [
    "Run `cargo test -p athena-core`: a new test queues six person-filed urgent requests and two Steward requests, builds the plan, and asserts the first three dispatched items include at least one Steward request.",
    "Run `cargo test -p athena-core`: a new test gives a Hestia request whose failure class appears in introspections of three distinct jobs and asserts it is ordered as urgent; a companion case with two distinct jobs asserts it is not promoted.",
    "Run `cargo test`: a new test with no Steward or Hestia requests queued asserts the plan order is identical to the order before the change, and the existing plan tests still pass.",
    "Run `cargo test -p athena-cli`: a test of the `athena plan` output (display::plan or the CLI drain test) asserts it names each item that takes the improvement turn, and that the drain report lists them.",
    "An improvement request whose reading fails (missing goal, rung or acceptance) or whose dependency is unmet is not given a turn; a test or the diff in plan.rs shows the turn is taken only from ready, passing entries.",
    "Run `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test`: all pass.",
    "CHANGELOG.md has an entry for the guaranteed improvement turn, the three-job urgent promotion and the plan and report display, and each claim matches the diff, with nothing claimed beyond it.",
    "The changed files are limited to the ordering and planning code, the plan display, tests and CHANGELOG.md, with no change to athena.toml or defaults.toml.",
];

/// The Acceptance bullets of job 20261109-161505-bp5e, verbatim from its recorded request.
const BP5E_BULLETS: [&str; 3] = [
    "The root cause is named in the change description and fixed where it originates, not at one instance",
    "A test fails without the change and passes with it, for the cause and not only for one job",
    "fmt, clippy and test pass",
];

/// The criteria of the recorded reading of job 20261109-161505-bp5e, verbatim.
const BP5E_CRITERIA: [&str; 6] = [
    "crates/athena-core/src/admission.rs contains a rule that emits a PolicyViolation naming CHANGELOG.md when the reading's deliverables list it, defaults_applied has id `changelog`, and no change in input.changes has path CHANGELOG.md; its doc comment names the root cause (the default was a rule the producer must remember, with no code comparing the diff to the deliverables).",
    "Run `cargo test -p athena-core admission`: new tests show a candidate that changes only docs/ files with a change reading carrying the changelog default yields a violation naming CHANGELOG.md; the same candidate plus a CHANGELOG.md change yields none; a non-change reading or a reading without the default yields none. The first test fails when the new rule is removed.",
    "Run `cargo test -p athena-core` : a test with a docs-only design reading built through ensure_changelog (the path both failed jobs took) is refused by admit before verification, showing the check is for the class and not one job.",
    "Existing tests that run admit or the run pipeline with a change reading still pass (cargo test) after fixtures are updated where they omitted CHANGELOG.md.",
    "cargo fmt --check, cargo clippy --all-targets and cargo test pass.",
    "CHANGELOG.md has an entry for this change naming the cause and the new admission rule.",
];

/// A request whose acceptance bullets are `bullets` under an `Acceptance:` heading, after the `head` lines.
fn acceptance_request(head: &[&str], marker: &str, bullets: &[&str]) -> String {
    let mut text: Vec<String> = head.iter().map(|l| l.to_string()).collect();
    text.push(String::new());
    text.push("Acceptance:".into());
    text.extend(bullets.iter().map(|b| format!("{marker}{b}")));
    text.join("\n")
}

/// Sets the criteria of `c`'s reading to `texts`, all `content` criteria.
fn set_criteria(c: &mut Case, texts: &[&str]) {
    c.reading.acceptance_criteria = texts
        .iter()
        .map(|text| Criterion {
            text: text.to_string(),
            kind: CriterionKind::Content,
        })
        .collect();
}

/// Removes the criterion at `index`, so the reading leaves out the request's bullet that it copied.
fn drop_criterion(c: &mut Case, index: usize) {
    c.reading.acceptance_criteria.remove(index);
}

/// Readings whose criteria leave out a bullet under the request's `Acceptance:` heading
/// (`check_reading_coverage`), and the readings that cover every bullet or are not checked. The bullet and
/// criterion texts of the accepted recorded jobs are verbatim; the requests around them are rebuilt, and
/// each case says which.
fn coverage_cases() -> Vec<Case> {
    const SZA7: &str = "job 20261029-084645-sza7, the status bullet dropped as a split; bullets rebuilt from the recorded request";
    const SIBLING: &str =
        "job 20261029-084645-sza7 shape, bullets that share framing; rebuilt from the job brief";
    const SPLIT: &str = "job 20261029-084645-sza7 shape, a request that says something about a split; rebuilt from the job brief";
    let sza7_bullets = [
        "A test with a fake check runner that exceeds its time limit shows the job ends as an environment stop, not a rejection, and a real test failure is still a failed check.",
        "A test shows status reports the median and slowest test-check durations and flags one over 80% of the limit.",
        "The guide says the check limit is 1800 seconds and the config test passes.",
    ];
    let sza7_head = [
        "The test check is cut off at random.",
        "Fix: (1) raise the limit; (2) record a killed check as an environment stop; (3) report the median and slowest durations in athena status.",
    ];
    let sibling_bullets = [
        "The status list shows each job's state",
        "The status list shows each job's owner",
        "The status list shows each job's duration in seconds",
    ];
    let mut out = Vec::new();

    let mut c = case(
        "coverage-omitted-bullet-sza7-refused",
        SZA7,
        &["coverage-bullet-omitted", "dash-bullet"],
        &acceptance_request(&sza7_head, "- ", &sza7_bullets),
        3,
        Expected::Refuse("check_reading_coverage"),
    );
    drop_criterion(&mut c, 1);
    out.push(c);

    let mut c = case(
        "coverage-sibling-framing-refused",
        SIBLING,
        &["coverage-sibling-framing"],
        &acceptance_request(&["Make the status list richer."], "- ", &sibling_bullets),
        3,
        Expected::Refuse("check_reading_coverage"),
    );
    set_criteria(&mut c, &[sibling_bullets[0], sibling_bullets[1]]);
    out.push(c);

    let mut c = case(
        "coverage-star-bullet-omitted-refused",
        SIBLING,
        &["star-bullet", "coverage-bullet-omitted"],
        &acceptance_request(&["Make the status list richer."], "* ", &sibling_bullets),
        3,
        Expected::Refuse("check_reading_coverage"),
    );
    drop_criterion(&mut c, 2);
    out.push(c);

    let mut c = case(
        "coverage-number-dot-omitted-refused",
        SIBLING,
        &["number-dot-parts", "coverage-bullet-omitted"],
        &lines(&[
            "Make the status list richer.",
            "",
            "Acceptance:",
            "1. The status list shows each job's state",
            "2. The status list shows each job's owner",
            "3. The status list shows each job's duration in seconds",
        ]),
        3,
        Expected::Refuse("check_reading_coverage"),
    );
    drop_criterion(&mut c, 2);
    out.push(c);

    let mut c = case(
        "coverage-wrapped-bullet-omitted-refused",
        SIBLING,
        &["coverage-wrapped-bullet", "coverage-bullet-omitted"],
        &lines(&[
            "Make the status list richer.",
            "",
            "Acceptance:",
            "- The status list shows each job's state, and a job that has no",
            "  state yet shows a dash",
            "- The status list shows each job's owner",
            "- The status list shows each job's duration in seconds",
        ]),
        3,
        Expected::Refuse("check_reading_coverage"),
    );
    set_criteria(
        &mut c,
        &[
            "The status list shows each job's state, and a job that has no state yet shows a dash",
            "The status list shows each job's owner",
        ],
    );
    out.push(c);

    // The prose says 'measured duration' and 'message' as the first bullet does.
    let repeated = lines(&[
        "Print the measured duration. The message must name the measured duration with the limit, and the limit is the one in the message.",
        "",
        "Acceptance:",
        "- The message names the measured duration and the limit",
        "- the guide says 1800 seconds",
    ]);
    let mut c = case(
        "coverage-prose-repeated-words-refused",
        SPLIT,
        &["coverage-prose-repeated-words"],
        &repeated,
        2,
        Expected::Refuse("check_reading_coverage"),
    );
    set_criteria(&mut c, &["the guide says the value is 1800 seconds"]);
    out.push(c);
    let mut c = case(
        "coverage-prose-repeated-words-covered-accepted",
        SPLIT,
        &["coverage-prose-repeated-words"],
        &repeated,
        2,
        Expected::Accept,
    );
    set_criteria(
        &mut c,
        &[
            "the guide says the value is 1800 seconds",
            "The stop message names the measured duration against the limit",
        ],
    );
    out.push(c);

    let splits: [(&'static str, &'static [&'static str], &str, Expected); 3] = [
        (
            "coverage-split-negated-refused",
            &["coverage-split-negated"],
            "Do not split this job.",
            Expected::Refuse("check_reading_coverage"),
        ),
        (
            "coverage-split-lookalike-prose-refused",
            &["coverage-split-lookalike-prose"],
            "Split the output into two columns.",
            Expected::Refuse("check_reading_coverage"),
        ),
        (
            "coverage-split-allowed-accepted",
            &["coverage-split-allowed"],
            "You may split this job.",
            Expected::Accept,
        ),
    ];
    for (id, forms, line, expected) in splits {
        let mut c = case(
            id,
            SPLIT,
            forms,
            &acceptance_request(
                &[line, "Make the status list richer."],
                "- ",
                &sibling_bullets,
            ),
            3,
            expected,
        );
        drop_criterion(&mut c, 2);
        out.push(c);
    }

    // The recorded readings of jobs ttnw and bp5e cover each bullet in words of their own.
    let mut c = case(
        "coverage-paraphrased-ttnw-accepted",
        "job 20261107-031902-ttnw; Acceptance bullets and the reading's criteria verbatim, the rest of the request left out",
        &["coverage-bullet-paraphrased", "dash-bullet"],
        &acceptance_request(
            &["Give Athena's own improvement requests a guaranteed turn in the drain."],
            "- ",
            &TTNW_BULLETS,
        ),
        5,
        Expected::Accept,
    );
    set_criteria(&mut c, &TTNW_CRITERIA);
    out.push(c);
    let mut c = case(
        "coverage-paraphrased-bp5e-accepted",
        "job 20261109-161505-bp5e; Acceptance bullets and the reading's criteria verbatim, the rest of the request left out",
        &["coverage-bullet-paraphrased"],
        &acceptance_request(
            &["Fix the root cause of a repeated failure."],
            "- ",
            &BP5E_BULLETS,
        ),
        3,
        Expected::Accept,
    );
    set_criteria(&mut c, &BP5E_CRITERIA);
    // The recorded first criterion names the file the job changed, which is in the repository at its base.
    c.repo_files
        .as_mut()
        .unwrap()
        .extend(["crates/athena-core/src/admission.rs", "CHANGELOG.md"].map(String::from));
    out.push(c);

    let mut c = case(
        "coverage-reading-asks-a-question-accepted",
        SPLIT,
        &["coverage-reading-asks-or-declines"],
        &acceptance_request(&["Make the status list richer."], "- ", &sibling_bullets),
        3,
        Expected::Accept,
    );
    set_criteria(&mut c, &[sibling_bullets[0]]);
    c.reading.question = Some("May I split this job into the state part and the rest?".into());
    out.push(c);
    let mut c = case(
        "coverage-declined-reading-accepted",
        SPLIT,
        &["coverage-reading-asks-or-declines"],
        &acceptance_request(&["Make the status list richer."], "- ", &sibling_bullets),
        3,
        Expected::Accept,
    );
    c.reading.capability = None;
    c.reading.decline_reason = Some("no capability fits this request".into());
    c.reading.acceptance_criteria.clear();
    c.reading.deliverables.clear();
    out.push(c);

    let mut c = case(
        "coverage-bullets-without-heading-accepted",
        SIBLING,
        &["coverage-no-acceptance-heading"],
        &lines(&[
            "Make the status list richer:",
            "- The status list shows each job's state",
            "- The status list shows each job's owner",
            "- The status list shows each job's duration in seconds",
        ]),
        3,
        Expected::Accept,
    );
    drop_criterion(&mut c, 2);
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

/// The text of `reading.rs` and its child modules, which hold the gate. The child modules come first, so a
/// search for a function finds it before any test text that names it.
fn reading_source() -> &'static str {
    static SOURCE: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SOURCE.get_or_init(|| {
        [
            include_str!("../src/reading/retry.rs"),
            include_str!("../src/reading/topics.rs"),
            include_str!("../src/reading/checks.rs"),
            include_str!("../src/reading/outcome.rs"),
            include_str!("../src/reading/terms.rs"),
            include_str!("../src/reading/surveys.rs"),
            include_str!("../src/reading/replay.rs"),
            include_str!("../src/reading/coverage.rs"),
            include_str!("../src/reading.rs"),
        ]
        .join("\n")
    })
}

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
    let names = acceptance_check_names(reading_source());
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
    assert!(reading_source().contains(anchor), "the anchor line moved");
    let doctored = reading_source().replacen(
        anchor,
        &format!("    step!(check_new_rule(reading));\n{anchor}"),
        1,
    );
    let names = acceptance_check_names(&doctored);
    assert_eq!(
        names.len(),
        acceptance_check_names(reading_source()).len() + 1
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
        acceptance_check_names(reading_source())
            .contains(&"check_design_existing_code".to_string())
    );
    let doc = reading_source()
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
fn environment_reach_refusal_is_declared_intended() {
    // The declared share is read from the criterion text by the replay.
    let declared = athena_core::reading::replay_declaration(
        &[Criterion {
            text: "The change refuses more readings: expected 8% of newly refused readings".into(),
            kind: CriterionKind::Behavior,
        }],
        &[],
    );
    assert_eq!(declared.share, Some(8));
    // The step line is in refusing_check, and the doc comment names the cause and the estimate.
    assert!(
        acceptance_check_names(reading_source()).contains(&"check_environment_reach".to_string())
    );
    let doc = reading_source()
        .split("pub fn check_environment_reach(")
        .next()
        .unwrap()
        .lines()
        .rev()
        .take_while(|l| l.starts_with("///"))
        .map(|l| l.trim_start_matches('/').trim())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" ");
    for needle in [
        "This check refuses more readings; the higher refusal rate is intended.",
        "no step asking who performs each action, where, with what permissions, and what data it reaches",
        "20261024-220048-engm",
        "20261025-070538-8ra0",
        "20261025-034247-nm0k",
        "estimate",
    ] {
        assert!(doc.contains(needle), "the doc comment lacks {needle:?}");
    }
    let changelog = include_str!("../../../CHANGELOG.md")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(changelog.contains("check_environment_reach"));
    assert!(changelog.contains("higher refusal rate is intended"));
    assert!(changelog.contains("expected 8% of newly refused readings"));
}

#[test]
fn environment_cache_scope_refusal_is_declared_intended() {
    // The declared share is read from the criterion text by the replay.
    let declared = athena_core::reading::replay_declaration(
        &[Criterion {
            text: "The change refuses more readings: expected 3% of newly refused readings".into(),
            kind: CriterionKind::Behavior,
        }],
        &[],
    );
    assert_eq!(declared.share, Some(3));
    // The check is one of the acceptance checks, and its doc comment names the cause, the jobs and the estimate.
    assert!(
        acceptance_check_names(reading_source()).contains(&"check_environment_reach".to_string())
    );
    let collapse = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    let doc = collapse(
        &reading_source()
            .split("pub fn check_environment_reach(")
            .next()
            .unwrap()
            .lines()
            .rev()
            .take_while(|l| l.starts_with("///"))
            .map(|l| l.trim_start_matches('/').trim())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" "),
    );
    let changelog = collapse(include_str!("../../../CHANGELOG.md"));
    for (label, text) in [("the doc comment", &doc), ("the changelog", &changelog)] {
        for needle in [
            "no step asking, for each CI cache or stored artifact, which ref saves it, which runs restore it, \
             and which runs may write shared entries",
            "20261025-034731-2am0",
            "20261025-055626-er96",
            "20261025-050603-wrv7",
            "20261025-065849-n99a",
            "higher refusal rate is intended, expected 3% of newly refused readings",
            "estimate, not measured",
        ] {
            assert!(text.contains(&collapse(needle)), "{label} lacks {needle:?}");
        }
    }
}

#[test]
fn history_fixture_refusal_is_declared_intended() {
    // The declared share is read from the criterion text by the replay.
    let declared = athena_core::reading::replay_declaration(
        &[Criterion {
            text: "The change refuses more readings: expected 4% of newly refused readings".into(),
            kind: CriterionKind::Behavior,
        }],
        &[],
    );
    assert_eq!(declared.share, Some(4));
    // The step line is in refusing_check, and the doc comment names the cause and the estimate.
    assert!(
        acceptance_check_names(reading_source()).contains(&"check_history_fixture".to_string())
    );
    let doc = reading_source()
        .split("pub fn check_history_fixture(")
        .next()
        .unwrap()
        .lines()
        .rev()
        .take_while(|l| l.starts_with("///"))
        .map(|l| l.trim_start_matches('/').trim())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" ");
    for needle in [
        "This check refuses more readings; the higher refusal rate is intended.",
        "derived from the happy-path commit shape",
        "20261024-153908-1rve",
        "20261024-154552-e65v",
        "20261025-012657-hac3",
        "estimate",
    ] {
        assert!(doc.contains(needle), "the doc comment lacks {needle:?}");
    }
    let changelog = include_str!("../../../CHANGELOG.md")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(changelog.contains("check_history_fixture"));
    assert!(changelog.contains("higher refusal rate is intended"));
    assert!(changelog.contains("expected 4% of newly refused readings"));
}

#[test]
fn coverage_replay_over_the_corpus_and_recorded_shapes() {
    // Every corpus reading goes through `check_reading_coverage` alone. A refusal must be a case that expects
    // it, so no reading that covers every bullet is refused; and a case that expects it must be refused.
    let cases = cases();
    let mut refused = 0;
    let mut covered_but_refused = Vec::new();
    for c in &cases {
        let got = athena_core::reading::check_reading_coverage(&c.reading, &c.request);
        let expected = c.expected == Expected::Refuse("check_reading_coverage");
        match (got.is_err(), expected) {
            (true, true) => refused += 1,
            (false, false) => {}
            (true, false) => covered_but_refused.push(c.id),
            (false, true) => panic!("{} ({}): expected a coverage refusal", c.id, c.source),
        }
    }
    assert!(
        covered_but_refused.is_empty(),
        "refused by check_reading_coverage though not expected to be: {covered_but_refused:?}"
    );
    // Of the readings the corpus held before this check, which `base_reading` builds with every bullet as a
    // criterion, none is newly refused; that figure is a property of the sample and not the history rate.
    let before: Vec<&Case> = cases
        .iter()
        .filter(|c| !c.id.starts_with("coverage-") && c.expected == Expected::Accept)
        .collect();
    let newly = before
        .iter()
        .filter(|c| athena_core::reading::check_reading_coverage(&c.reading, &c.request).is_err())
        .count();
    println!(
        "coverage replay: {} readings, {refused} refused by check_reading_coverage, 0 readings that cover every bullet refused; \
         of the {} corpus readings accepted before this check, {newly} are newly refused (a chosen sample, not the history rate)",
        cases.len(),
        before.len()
    );
    assert_eq!(newly, 0);
    // The three recorded jobs: sza7 is refused and ttnw and bp5e are accepted.
    let by_id = |id: &str| cases.iter().find(|c| c.id == id).unwrap();
    for id in [
        "coverage-paraphrased-ttnw-accepted",
        "coverage-paraphrased-bp5e-accepted",
    ] {
        assert!(run(by_id(id)).is_none(), "{id}");
    }
    let (name, error) = run(by_id("coverage-omitted-bullet-sza7-refused")).unwrap();
    assert_eq!(name, "check_reading_coverage");
    assert!(
        error.to_string().contains("A test shows status reports"),
        "{error}"
    );
}

#[test]
fn reading_coverage_refusal_is_declared_intended() {
    // The step line is in refusing_check, and the doc comment names the cause, the job and the estimate.
    assert!(
        acceptance_check_names(reading_source()).contains(&"check_reading_coverage".to_string())
    );
    let doc = reading_source()
        .split("pub fn check_reading_coverage(")
        .next()
        .unwrap()
        .lines()
        .rev()
        .take_while(|l| l.starts_with("///"))
        .map(|l| l.trim_start_matches('/').trim())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join(" ");
    for needle in [
        "20261029-084645-sza7",
        "the 2026-10-29 job rejected for a split the reading chose",
        "[`check_reading_acceptance`]",
        "a split may be proposed only as a question, never by dropping a bullet",
        "This check refuses more readings; the higher refusal rate is intended.",
        "estimate",
    ] {
        assert!(doc.contains(needle), "the doc comment lacks {needle:?}");
    }
    let changelog = include_str!("../../../CHANGELOG.md")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for needle in [
        "check_reading_coverage",
        "refuses more readings",
        "higher refusal rate is intended",
        "expected 10% of newly refused readings",
        "an estimate",
        "20261029-084645-sza7",
        "a split may be proposed only as a question, never by dropping a bullet",
        "the verification replay",
    ] {
        assert!(changelog.contains(needle), "the changelog lacks {needle:?}");
    }
}

#[test]
fn no_check_bypasses_refusing_check() {
    let body = function_body(reading_source(), "pub fn check_reading_acceptance(");
    let (_, inside) = body
        .split_once(") -> Result<()> {\n")
        .expect("the signature of check_reading_acceptance");
    assert!(
        !inside.contains("check_"),
        "check_reading_acceptance calls a check directly; add it as a step! line in refusing_check:\n{inside}"
    );
    assert!(inside.contains("refusing_check("));
}

// ---------------------------------------------------------------------------------------------------------
// The plan side: Athena's own checks on a plan (`plan_findings`) refuse a plan whose `dependents` section is
// missing or incomplete. Plans do not exist at reading time, so these cases drive `plan_findings`, not
// `refusing_check`. The rule refuses more plans than before (every plan without dependents); the higher
// refusal rate is intended, and expected 15% of newly refused readings is the share the change declares.
// Sources are the four earlier breakages, where a change left a reader of the changed behavior unexamined,
// and job 20261102-063419-0beg, the retry of which added the section. The same module also drives the
// open-questions rule and the impact rule (a risk or dependent that defers caller and test impact to the
// producer).

mod plan_side {
    use athena_agent::policy::Policy;
    use athena_core::admission::{dependents_unnamed, plan_findings};
    use athena_data::JobId;
    use athena_data::records::{
        Criterion, CriterionKind, Demonstration, DependentKind, Finding, FindingSource, JobPlan,
        PlanDependent, PlanRisk, Role, Severity,
    };

    /// What `plan_findings` must do with a plan case.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum PlanExpected {
        Accept,
        /// A major finding at `dependents` or `dependents, dependent N` whose summary holds the text.
        Refuse(&'static str),
    }

    struct PlanCase {
        id: &'static str,
        source: &'static str,
        dependents: Vec<PlanDependent>,
        expected: PlanExpected,
    }

    fn entry(
        name: &str,
        dependent: &str,
        test: Option<&str>,
        reason: Option<&str>,
    ) -> PlanDependent {
        PlanDependent {
            name: name.into(),
            dependent: dependent.into(),
            kind: DependentKind::Rule,
            test: test.map(str::to_string),
            reason: reason.map(str::to_string),
        }
    }

    fn plan_cases() -> Vec<PlanCase> {
        let ci_reuse_source = "the CI-reuse breakage: a recorded check's exit code changed and the \
            seen-commands rule in verify.rs stopped counting reused checks; rebuilt from job \
            20261102-063419-0beg";
        vec![
            PlanCase {
                id: "dependents-missing",
                source: ci_reuse_source,
                dependents: vec![],
                expected: PlanExpected::Refuse("no dependents section"),
            },
            PlanCase {
                id: "dependents-blank-name",
                source: ci_reuse_source,
                dependents: vec![entry("  ", "verify.rs judge", Some("a test"), None)],
                expected: PlanExpected::Refuse("Dependent 1 has a blank name"),
            },
            PlanCase {
                id: "dependents-no-test-or-reason",
                source: ci_reuse_source,
                dependents: vec![entry("recorded exit code", "verify.rs judge", None, None)],
                expected: PlanExpected::Refuse("neither a test nor a reason"),
            },
            PlanCase {
                id: "dependents-whitespace-only-test-and-reason",
                source: ci_reuse_source,
                dependents: vec![entry(
                    "recorded exit code",
                    "verify.rs judge",
                    Some(" "),
                    Some("\n"),
                )],
                expected: PlanExpected::Refuse("neither a test nor a reason"),
            },
            PlanCase {
                id: "dependents-test-only",
                source: ci_reuse_source,
                dependents: vec![entry(
                    "recorded exit code",
                    "verify.rs judge",
                    Some("cargo test -p athena-core verify"),
                    None,
                )],
                expected: PlanExpected::Accept,
            },
            PlanCase {
                id: "dependents-reason-only",
                source: ci_reuse_source,
                dependents: vec![entry(
                    "recorded exit code",
                    "status.rs",
                    None,
                    Some("it prints the code and never branches on it"),
                )],
                expected: PlanExpected::Accept,
            },
            PlanCase {
                id: "dependents-complete-ci-reuse",
                source: ci_reuse_source,
                dependents: vec![entry(
                    "recorded exit code",
                    "verify.rs judge: the seen-commands filter counts a reused passing check",
                    Some("cargo test -p athena-core verify"),
                    None,
                )],
                expected: PlanExpected::Accept,
            },
        ]
    }

    fn plan_for(dependents: Vec<PlanDependent>) -> JobPlan {
        JobPlan {
            job: JobId::parse("20261031-000000-aaaa").unwrap(),
            round: 1,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: 2,
            approach: "Approach.".into(),
            decisions: vec![],
            files: vec![],
            steps: vec![],
            risks: vec![],
            dependents,
            demonstrations: vec![Demonstration {
                criterion: 1,
                method: "read".into(),
                command: "cat x".into(),
                observes: "it holds".into(),
                fails_without: None,
                seam: None,
            }],
            open_questions: vec![],
            producer_section_sha256: "0".repeat(64),
            resumed_from: None,
        }
    }

    fn findings(dependents: Vec<PlanDependent>) -> Vec<Finding> {
        let mut reading = super::base_reading("");
        reading.acceptance_criteria = vec![Criterion {
            text: "The plan lists its dependents".into(),
            kind: CriterionKind::Content,
        }];
        let verifier = Policy {
            role: Role::Verifier,
            root: std::path::PathBuf::from("."),
            commands: Default::default(),
            writes: false,
            evidence: None,
        };
        plan_findings(&plan_for(dependents), &reading, &verifier, &|_| false)
    }

    /// The plan cases that fail their expectation, each as a line naming the case and its source.
    fn plan_mismatches(cases: &[PlanCase]) -> Vec<String> {
        let mut found = Vec::new();
        for c in cases {
            let got = findings(c.dependents.clone());
            let ok = match c.expected {
                PlanExpected::Accept => got.is_empty(),
                PlanExpected::Refuse(text) => {
                    got.len() == 1
                        && got[0].severity == Severity::Major
                        && got[0].source == FindingSource::Athena
                        && got[0]
                            .location
                            .as_deref()
                            .is_some_and(|l| l.starts_with("dependents"))
                        && got[0].summary.contains(text)
                }
            };
            if !ok {
                found.push(format!(
                    "{} ({}): expected {:?}, got {got:?}",
                    c.id, c.source, c.expected
                ));
            }
        }
        found
    }

    #[test]
    fn plan_cases_get_the_expected_outcome() {
        let cases = plan_cases();
        let ids: std::collections::BTreeSet<&str> = cases.iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), cases.len(), "plan case ids are unique");
        assert!(
            cases.iter().any(|c| c.expected == PlanExpected::Accept)
                && cases
                    .iter()
                    .any(|c| matches!(c.expected, PlanExpected::Refuse(_))),
            "the table holds accepted and refused plans"
        );
        let mismatches = plan_mismatches(&cases);
        assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    }

    #[test]
    fn a_plan_case_that_expects_the_wrong_outcome_is_reported() {
        let wrong = vec![
            PlanCase {
                id: "missing-but-expected-accepted",
                source: "test",
                dependents: vec![],
                expected: PlanExpected::Accept,
            },
            PlanCase {
                id: "complete-but-expected-refused",
                source: "test",
                dependents: vec![entry("n", "d", Some("t"), None)],
                expected: PlanExpected::Refuse("no dependents section"),
            },
        ];
        assert_eq!(plan_mismatches(&wrong).len(), 2);
    }

    // The open-questions rule: only a `User:` or `Host:` question stays open, and one that says the code
    // answers it is refused. It refuses more plans; expected 5% of newly refused plans (the reading's
    // estimate, not a measurement); the higher refusal rate is intended, because those plans spent critic
    // rounds on questions code could answer. The records of the failed jobs are not in the snapshot, so the
    // reworded cases are synthetic: a paraphrase of the failure class's wording, not the jobs' own text.

    struct OpenQuestionCase {
        id: &'static str,
        source: &'static str,
        question: &'static str,
        expected: PlanExpected,
    }

    fn open_question_cases() -> Vec<OpenQuestionCase> {
        let landing = "synthetic: paraphrase of the failure class in the request (a plan left \
            questions about the landing path open); jobs 20261030-013555-f5qm and 20261031-030119-tw3v";
        let made_up = "synthetic: written for this corpus, no job behind it";
        vec![
            OpenQuestionCase {
                id: "question-max-concurrency-into-landing",
                source: landing,
                question: "Does the [parallel] max_concurrency path reach landing?",
                expected: PlanExpected::Refuse("has no addressee"),
            },
            OpenQuestionCase {
                id: "question-old-job-records-on-the-landing-branch",
                source: landing,
                question: "What do old job records look like on the landing branch?",
                expected: PlanExpected::Refuse("has no addressee"),
            },
            OpenQuestionCase {
                id: "question-with-a-code-path-and-no-addressee",
                source: made_up,
                question: "Does `crates/athena-cli/src/main.rs` pass max_concurrency into landing?",
                expected: PlanExpected::Refuse("has no addressee"),
            },
            OpenQuestionCase {
                id: "question-lookalike-prefix",
                source: made_up,
                question: "Username: which account lands the change?",
                expected: PlanExpected::Refuse("has no addressee"),
            },
            OpenQuestionCase {
                id: "user-question-that-says-the-code-answers-it",
                source: made_up,
                question: "User: is this found by reading the code?",
                expected: PlanExpected::Refuse("says the code answers it"),
            },
            OpenQuestionCase {
                id: "host-question-that-says-the-code-answers-it",
                source: made_up,
                question: "Host: can we check the code for the answer?",
                expected: PlanExpected::Refuse("says the code answers it"),
            },
            OpenQuestionCase {
                id: "user-decision",
                source: made_up,
                question: "User: Should the CHANGELOG entry go under Added or Changed?",
                expected: PlanExpected::Accept,
            },
            OpenQuestionCase {
                id: "user-scope-question-citing-a-path",
                source: made_up,
                question: "user: may the change edit `crates/athena-core/tests/refusal_corpus.rs`?",
                expected: PlanExpected::Accept,
            },
            OpenQuestionCase {
                id: "host-fact-marked-in-bold",
                source: made_up,
                question: "**Host:** Is the production host disk large enough?",
                expected: PlanExpected::Accept,
            },
        ]
    }

    fn question_findings(question: &str) -> Vec<Finding> {
        let mut plan = plan_for(vec![entry(
            "recorded exit code",
            "verify.rs judge: the seen-commands filter counts a reused passing check",
            Some("cargo test -p athena-core verify"),
            None,
        )]);
        plan.open_questions = vec![question.to_string()];
        let mut reading = super::base_reading("");
        reading.acceptance_criteria = vec![Criterion {
            text: "The plan lists its dependents".into(),
            kind: CriterionKind::Content,
        }];
        let verifier = Policy {
            role: Role::Verifier,
            root: std::path::PathBuf::from("."),
            commands: Default::default(),
            writes: false,
            evidence: None,
        };
        plan_findings(&plan, &reading, &verifier, &|_| false)
    }

    #[test]
    fn open_question_cases_get_the_expected_outcome() {
        let cases = open_question_cases();
        let ids: std::collections::BTreeSet<&str> = cases.iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), cases.len(), "open-question case ids are unique");
        let mut mismatches = Vec::new();
        for c in &cases {
            let got = question_findings(c.question);
            let ok = match c.expected {
                PlanExpected::Accept => got.is_empty(),
                PlanExpected::Refuse(text) => {
                    got.len() == 1
                        && got[0].severity == Severity::Major
                        && got[0].source == FindingSource::Athena
                        && got[0].location.as_deref() == Some("open questions")
                        && got[0].summary.contains(text)
                }
            };
            if !ok {
                mismatches.push(format!(
                    "{} ({}): expected {:?}, got {got:?}",
                    c.id, c.source, c.expected
                ));
            }
        }
        assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
        assert!(
            cases.iter().any(|c| c.expected == PlanExpected::Accept)
                && cases
                    .iter()
                    .any(|c| matches!(c.expected, PlanExpected::Refuse(_))),
            "the table holds accepted and refused questions"
        );
    }

    // The impact rule: a risk, or a dependent's test or reason, that defers the search for existing callers and
    // tests to the producer is refused at `risks` or `dependents, dependent N`, and a risk that states a found
    // answer, names a command the producer runs, or an empty list is accepted. It refuses more plans; expected
    // 5% of newly refused plans (the reading's estimate, not a measurement); the higher refusal rate is
    // intended, because those plans cost critic rounds on impact the planner can find by searching. Only the
    // first case is a real job's wording (job 20261025-055626-er96); the others are synthetic.

    struct ImpactCase {
        id: &'static str,
        source: &'static str,
        /// `(risk, mitigation)` pairs.
        risks: Vec<(&'static str, &'static str)>,
        dependent_test: Option<&'static str>,
        dependent_reason: Option<&'static str>,
        /// The location of the one expected finding, when the case is refused.
        location: &'static str,
        expected: PlanExpected,
    }

    fn risk_case(
        id: &'static str,
        risk: &'static str,
        mitigation: &'static str,
        expected: PlanExpected,
    ) -> ImpactCase {
        ImpactCase {
            id,
            source: "synthetic: written for this corpus",
            risks: vec![(risk, mitigation)],
            dependent_test: Some("cargo test -p athena-core verify"),
            dependent_reason: None,
            location: "risks",
            expected,
        }
    }

    fn impact_cases() -> Vec<ImpactCase> {
        use PlanExpected::{Accept, Refuse};
        let defers = "defers its impact to the producer";
        let cases = vec![
            ImpactCase {
                id: "risk-producer-to-check",
                source: "job 20261025-055626-er96: the risk and mitigation its plan passed the gate with",
                risks: vec![(
                    "Existing go tests would call real gh",
                    "producer to check existing go tests",
                )],
                dependent_test: Some("cargo test -p athena-core verify"),
                dependent_reason: None,
                location: "risks",
                expected: Refuse(defers),
            },
            risk_case(
                "risk-producer-should-check",
                "Callers may break",
                "The producer should check the callers",
                Refuse(defers),
            ),
            risk_case(
                "risk-producer-will-check",
                "Readers of the file",
                "Producer will check other crates for readers",
                Refuse(defers),
            ),
            risk_case(
                "risk-producer-to-verify",
                "Output assertions",
                "producer to verify the existing tests",
                Refuse(defers),
            ),
            risk_case(
                "risk-producer-should-also-check",
                "Tests",
                "producer should also search the tests",
                Refuse(defers),
            ),
            risk_case(
                "risk-producer-to-look-for",
                "A moved file",
                "producer to look for include_str! readers",
                Refuse(defers),
            ),
            risk_case(
                "risk-producer-checks-callers",
                "Callers",
                "producer checks existing callers",
                Refuse(defers),
            ),
            risk_case(
                "risk-possessive-producers-to-verify",
                "Readers",
                "producer's to verify the readers",
                Refuse(defers),
            ),
            risk_case(
                "risk-left-to-the-producer",
                "Warnings",
                "which tests assert no warning is left to the producer",
                Refuse(defers),
            ),
            risk_case(
                "risk-mixed-case-deferral",
                "Existing tests",
                "PRODUCER To Check Existing Tests",
                Refuse(defers),
            ),
            risk_case(
                "risk-bold-and-backtick-markup-deferral",
                "Readers",
                "**Producer** to `check` the readers",
                Refuse(defers),
            ),
            risk_case(
                "risk-deferral-across-a-line-break",
                "Callers",
                "producer to\ncheck existing callers",
                Refuse(defers),
            ),
            risk_case(
                "risk-deferral-in-the-risk-text",
                "A move may break an include_str! reader of display.rs; producer to check existing readers",
                "lifecycle.rs:2627 asserts no warning",
                Refuse(defers),
            ),
            risk_case(
                "risk-settled-answer-with-file-and-line",
                "Existing go tests would call real gh",
                "lifecycle.rs:2627 asserts no warning; GhEarly skips quietly",
                Accept,
            ),
            risk_case(
                "risk-producer-runs-a-command",
                "The budget line drifts",
                "The producer runs cargo test -p athena-cli --test budgets after the edit",
                Accept,
            ),
            risk_case(
                "risk-producer-should-check-that-fmt-passes",
                "Formatting",
                "the producer should check that cargo fmt --check passes",
                Accept,
            ),
            risk_case(
                "risk-producer-will-verify-with-cargo-test-budgets",
                "Budget",
                "the producer will verify with cargo test -p athena-cli --test budgets",
                Accept,
            ),
            risk_case(
                "risk-producer-must-read-the-figure",
                "Budget",
                "the producer must read the computed figure from the failure message",
                Accept,
            ),
            risk_case(
                "risk-producer-must-confirm-fmt",
                "Formatting",
                "the producer must confirm cargo fmt --check passes",
                Accept,
            ),
            risk_case(
                "risk-producer-verifies-existing-tests-pass",
                "Regression",
                "the producer should verify the existing tests pass",
                Accept,
            ),
            risk_case(
                "risk-left-to-the-producer-without-impact",
                "Budget",
                "updating the budget line is left to the producer",
                Accept,
            ),
            ImpactCase {
                id: "risk-empty-list",
                source: "synthetic: written for this corpus",
                risks: vec![],
                dependent_test: Some("cargo test -p athena-core verify"),
                dependent_reason: None,
                location: "risks",
                expected: Accept,
            },
            ImpactCase {
                id: "dependent-reason-defers",
                source: "synthetic: written for this corpus",
                risks: vec![],
                dependent_test: None,
                dependent_reason: Some("producer to check for include_str! readers"),
                location: "dependents, dependent 1",
                expected: Refuse(defers),
            },
            ImpactCase {
                id: "dependent-test-defers",
                source: "synthetic: written for this corpus",
                risks: vec![],
                dependent_test: Some("producer to check existing tests"),
                dependent_reason: None,
                location: "dependents, dependent 1",
                expected: Refuse(defers),
            },
            ImpactCase {
                id: "dependent-reason-states-found-answer",
                source: "synthetic: written for this corpus",
                risks: vec![],
                dependent_test: None,
                dependent_reason: Some(
                    "status.rs:410 reads display.rs via include_str!; test status_reads_display updated in step 4",
                ),
                location: "dependents, dependent 1",
                expected: Accept,
            },
        ];
        cases
    }

    fn impact_findings(c: &ImpactCase) -> Vec<Finding> {
        let mut plan = plan_for(vec![entry(
            "recorded exit code",
            "verify.rs judge: the seen-commands filter counts a reused passing check",
            c.dependent_test,
            c.dependent_reason,
        )]);
        plan.risks = c
            .risks
            .iter()
            .map(|(risk, mitigation)| PlanRisk {
                risk: (*risk).into(),
                mitigation: (*mitigation).into(),
            })
            .collect();
        let mut reading = super::base_reading("");
        reading.acceptance_criteria = vec![Criterion {
            text: "The plan lists its dependents".into(),
            kind: CriterionKind::Content,
        }];
        let verifier = Policy {
            role: Role::Verifier,
            root: std::path::PathBuf::from("."),
            commands: Default::default(),
            writes: false,
            evidence: None,
        };
        plan_findings(&plan, &reading, &verifier, &|_| false)
    }

    /// The impact cases that fail their expectation, each as a line naming the case and its source.
    fn impact_mismatches(cases: &[ImpactCase]) -> Vec<String> {
        let mut found = Vec::new();
        for c in cases {
            let got = impact_findings(c);
            let ok = match c.expected {
                PlanExpected::Accept => got.is_empty(),
                PlanExpected::Refuse(text) => {
                    got.len() == 1
                        && got[0].severity == Severity::Major
                        && got[0].source == FindingSource::Athena
                        && got[0].location.as_deref() == Some(c.location)
                        && got[0].summary.contains(text)
                }
            };
            if !ok {
                found.push(format!(
                    "{} ({}): expected {:?}, got {got:?}",
                    c.id, c.source, c.expected
                ));
            }
        }
        found
    }

    #[test]
    fn impact_cases_get_the_expected_outcome() {
        let cases = impact_cases();
        let ids: std::collections::BTreeSet<&str> = cases.iter().map(|c| c.id).collect();
        assert_eq!(ids.len(), cases.len(), "impact case ids are unique");
        assert!(
            cases.iter().any(|c| c.expected == PlanExpected::Accept)
                && cases
                    .iter()
                    .any(|c| matches!(c.expected, PlanExpected::Refuse(_))),
            "the table holds accepted and refused plans"
        );
        let mismatches = impact_mismatches(&cases);
        assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    }

    #[test]
    fn an_impact_case_that_expects_the_wrong_outcome_is_reported() {
        let wrong = vec![
            risk_case(
                "deferral-but-expected-accepted",
                "Tests",
                "producer to check existing tests",
                PlanExpected::Accept,
            ),
            risk_case(
                "settled-but-expected-refused",
                "Tests",
                "lifecycle.rs:2627 asserts no warning",
                PlanExpected::Refuse("defers its impact to the producer"),
            ),
        ];
        assert_eq!(impact_mismatches(&wrong).len(), 2);
    }

    #[test]
    fn the_complete_ci_reuse_case_names_the_rule_the_search_finds() {
        let cases = plan_cases();
        let complete = cases
            .iter()
            .find(|c| c.id == "dependents-complete-ci-reuse")
            .unwrap();
        let hits = vec!["crates/athena-core/src/verify.rs".to_string()];
        assert!(dependents_unnamed(&complete.dependents, &hits).is_empty());
        let test_only = cases
            .iter()
            .find(|c| c.id == "dependents-reason-only")
            .unwrap();
        assert_eq!(dependents_unnamed(&test_only.dependents, &hits), hits);
    }
}
