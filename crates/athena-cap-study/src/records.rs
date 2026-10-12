//! The typed records of the study capability (design §4).
//!
//! Each record carries a `schema` field and is written once. Numbers that are measurements or estimates
//! (a cap, a cost) are [`Quantity`] values, which cannot be built without a provenance and a basis. Counts
//! that describe the shape of a model (a retry bound, a counter's start, a concurrency limit) are plain
//! integers, because they are structure and not measurements.

use athena_data::Quantity;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

pub const PIN_SCHEMA: &str = "study.pin/1";
pub const SOURCE_SCHEMA: &str = "study.source/1";
pub const CLAIM_SCHEMA: &str = "study.claim/1";
pub const RUN_SCHEMA: &str = "study.run/1";
pub const RATIONALE_SCHEMA: &str = "study.rationale/1";
pub const FINDING_SCHEMA: &str = "study.finding/1";
pub const GAP_SCHEMA: &str = "study.gap/1";
pub const VALIDATION_SCHEMA: &str = "study.validation/1";

/// `study.pin/1`: the studied system fixed at one commit before anything is read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub schema: String,
    pub study_id: String,
    pub system: String,
    pub repository_url: String,
    /// The full commit SHA.
    pub sha: String,
    /// The tag name when the reference was a tagged release.
    pub tag: Option<String>,
    pub resolved_at: Timestamp,
    pub how_resolved: String,
}

/// What a citation points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    File,
    Commit,
    PullRequest,
    Run,
}

/// `study.source/1`: one citation. Which fields apply depends on `kind`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub schema: String,
    pub kind: SourceKind,
    /// `file`: the path.
    pub path: Option<String>,
    /// `file`: first and last line, one-based and inclusive.
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
    /// `file`: the SHA read; `commit` and `pull-request`: the commit or the head or merge commit.
    pub sha: Option<String>,
    /// `pull-request`: the identifier.
    pub id: Option<String>,
    /// `file`: a verbatim quote from the cited lines.
    pub quote: Option<String>,
    /// `run`: the `study.run` id.
    pub run_id: Option<String>,
    /// `run`: when, within the run.
    pub timestamp: Option<Timestamp>,
}

impl Source {
    fn blank(kind: SourceKind) -> Source {
        Source {
            schema: SOURCE_SCHEMA.into(),
            kind,
            path: None,
            start_line: None,
            end_line: None,
            sha: None,
            id: None,
            quote: None,
            run_id: None,
            timestamp: None,
        }
    }

    /// A file citation.
    pub fn file(path: &str, start: u32, end: u32, sha: &str, quote: &str) -> Source {
        Source {
            path: Some(path.into()),
            start_line: Some(start),
            end_line: Some(end),
            sha: Some(sha.into()),
            quote: Some(quote.into()),
            ..Source::blank(SourceKind::File)
        }
    }

    /// A commit citation.
    pub fn commit(sha: &str) -> Source {
        Source {
            sha: Some(sha.into()),
            ..Source::blank(SourceKind::Commit)
        }
    }

    /// A pull-request citation with the SHA of its head or merge commit.
    pub fn pull_request(id: &str, sha: &str) -> Source {
        Source {
            id: Some(id.into()),
            sha: Some(sha.into()),
            ..Source::blank(SourceKind::PullRequest)
        }
    }

    /// A run citation.
    pub fn run(run_id: &str, at: Timestamp) -> Source {
        Source {
            run_id: Some(run_id.into()),
            timestamp: Some(at),
            ..Source::blank(SourceKind::Run)
        }
    }
}

/// How a claim was learned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mark {
    Read,
    Observed,
    Both,
}

/// `study.claim/1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub schema: String,
    pub id: String,
    pub text: String,
    pub mark: Mark,
    pub sources: Vec<Source>,
    /// Ids of claims this one contradicts.
    pub contradicts: Vec<String>,
    /// The mechanism it belongs to; `work-handling` for M1.
    pub mechanism: String,
}

/// The status of one capture stream in a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StreamStatus {
    Native,
    Added,
    Absent,
}

/// One capture stream of a run and its status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stream {
    pub name: String,
    pub status: StreamStatus,
}

/// What governed a cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Meter {
    WallClockProcess,
    ModelSpend,
}

/// `study.run/1`: one run of the studied system (or the record of why it did not start).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub schema: String,
    pub id: String,
    /// The SHA of the tree that ran.
    pub sha: String,
    pub scenario_id: String,
    pub configuration: String,
    pub command: String,
    pub start: Timestamp,
    pub end: Timestamp,
    /// The boundary it ran in; `None` only when the run did not start.
    pub boundary: Option<String>,
    pub cap: Quantity,
    pub kill_switch: String,
    pub meters: Vec<Meter>,
    pub cost: Vec<Quantity>,
    pub exit_state: String,
    pub blocker: Option<String>,
    /// Ids of the model transitions the run observed.
    pub observed_transitions: Vec<String>,
    pub streams: Vec<Stream>,
}

/// Where a rationale comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Provenance {
    Stated,
    Evidenced,
    Inferred,
}

/// `study.rationale/1`: why a significant design decision was made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rationale {
    pub schema: String,
    pub id: String,
    pub decision: String,
    pub alternatives: Vec<String>,
    pub tradeoff: String,
    pub reversal_cost: String,
    /// `None` is an incomplete card.
    pub provenance: Option<Provenance>,
    /// The authors' voice. Allowed only when the provenance is `stated`.
    pub authors_say: Option<String>,
    /// Ids of the claims the card cites.
    pub claims: Vec<String>,
    /// Sources the card cites directly.
    pub sources: Vec<Source>,
}

/// The kinds of finding (design §4, with the review-2 addition of `replay-difference`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingKind {
    /// Two claims disagree; the subject is both claim ids.
    Divergence,
    Waiver,
    NotNative,
    /// A stream the system did not expose; the subject names the edges it would have shown.
    NoObservation,
    /// A blocker; the subject names the edges that could not be exercised.
    CouldNotRun,
    ModelLimit,
    /// A replay differs from its original capture; the subject is the two run ids.
    ReplayDifference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    Info,
    Minor,
    Major,
}

/// The waiver part of a finding of kind `waiver`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Waiver {
    /// The rule waived, by its id.
    pub rule: String,
    /// The user's reason, in the user's words.
    pub reason: String,
    /// The id of the user intervention record that wrote the waiver.
    pub intervention: Option<String>,
}

/// `study.finding/1`. Every kind names what it covers in `subject` (review 2, R2-m2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub schema: String,
    pub id: String,
    pub kind: FindingKind,
    pub severity: Severity,
    /// Element ids, claim ids, or run ids the finding covers.
    pub subject: Vec<String>,
    pub detail: String,
    pub waiver: Option<Waiver>,
}

/// `study.gap/1`: a gap in the capability found during a study.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gap {
    pub schema: String,
    pub id: String,
    pub study_id: String,
    pub description: String,
    pub fix: String,
    pub rechecked_studies: Vec<String>,
}

/// One rule's result in a `study.validation/1` record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuleResult {
    pub rule: String,
    pub pass: bool,
    /// Offending record ids still standing after waivers.
    pub offending: Vec<String>,
    /// Offending record ids cleared by a valid waiver (model rules only).
    pub waived: Vec<String>,
}

/// `study.validation/1`: the result of running every rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Validation {
    pub schema: String,
    pub study_id: String,
    pub results: Vec<RuleResult>,
}

impl Validation {
    /// True when every rule passed (after waivers).
    pub fn passed(&self) -> bool {
        self.results.iter().all(|r| r.pass)
    }
}
