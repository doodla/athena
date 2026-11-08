//! Work-item, work-decision, audience-rating, audit and code-health-baseline records.

use super::*;

/// What a reference points at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RefKind {
    /// A job id.
    Job,
    /// A path in the repository or in a state directory.
    Path,
    /// A commit id.
    Commit,
}

/// A pointer to where data lives. Work items carry these instead of the data itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pub kind: RefKind,
    pub value: String,
}

impl Reference {
    pub fn job(id: &JobId) -> Reference {
        Reference {
            kind: RefKind::Job,
            value: id.as_str().to_string(),
        }
    }
    pub fn path(path: impl Into<String>) -> Reference {
        Reference {
            kind: RefKind::Path,
            value: path.into(),
        }
    }
    pub fn commit(sha: impl Into<String>) -> Reference {
        Reference {
            kind: RefKind::Commit,
            value: sha.into(),
        }
    }
}

/// What a work item asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkItemKind {
    /// Something the role could not see; the source names what it could not read.
    BlindSpot,
    /// A failure that repeats, with the jobs as evidence.
    RepeatedFailure,
    /// A change on a rung above the role's limit, proposed for a person to decide.
    Proposal,
}

/// `athena.work-item/1`: work a role queued. It names its source and cites its evidence by reference, and
/// copies no data from either. Work items are never edited.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkItem {
    pub id: RecordId,
    pub at: Timestamp,
    /// The role that queued it.
    pub role: String,
    pub kind: WorkItemKind,
    /// Stable across runs for the same finding, so a role does not queue it twice.
    pub key: String,
    /// One line.
    pub title: String,
    /// The requirement rung it concerns.
    pub rung: crate::roles::Ladder,
    /// Where the finding came from. For a blind spot, the data that could not be read.
    pub source: Reference,
    pub evidence: Vec<Reference>,
    /// The steward's plain-language brief for the owner. Written only when the steward raises a proposal
    /// above its limit; records from before it existed have none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brief: Option<OwnerBrief>,
}

/// The five short fields the steward writes so the owner can decide a proposal without reading its evidence.
/// Each is one line in plain words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerBrief {
    /// One yes/no line.
    pub question: String,
    /// What happens if the owner says yes.
    pub if_yes: String,
    /// What happens if the owner says no.
    pub if_no: String,
    /// What saying yes costs.
    pub cost: String,
    /// The steward's advice, with a one-line reason.
    pub recommendation: String,
}

impl OwnerBrief {
    /// Why this brief is not acceptable, if it is not: every field holds text.
    pub fn validate(&self) -> Result<(), String> {
        for (name, text) in [
            ("question", &self.question),
            ("if_yes", &self.if_yes),
            ("if_no", &self.if_no),
            ("cost", &self.cost),
            ("recommendation", &self.recommendation),
        ] {
            if text.trim().is_empty() {
                return Err(format!("a brief's {name} is not empty"));
            }
        }
        Ok(())
    }
}

impl WorkItem {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(brief) = &self.brief {
            brief.validate()?;
        }
        if self.role.trim().is_empty() || self.key.trim().is_empty() || self.title.trim().is_empty()
        {
            return Err("a work item names its role, key and title".into());
        }
        if self.source.value.trim().is_empty() {
            return Err("a work item names its source".into());
        }
        if self.kind != WorkItemKind::BlindSpot && self.evidence.is_empty() {
            return Err("a work item that is not a blind spot cites its evidence".into());
        }
        if self.evidence.iter().any(|e| e.value.trim().is_empty()) {
            return Err("an evidence reference is not empty".into());
        }
        Ok(())
    }
}

/// How a person decided a work item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum WorkDecisionState {
    /// Approved: the job that carries the work out.
    Approved { job: JobId },
    /// Rejected, with the person's reason.
    Rejected { reason: String },
}

/// `athena.work-decision/1`: a person's decision on one work item (`athena approve-work`,
/// `athena reject-work`). Work items are never edited, so the decision is a record of its own that names the
/// work item. A work item with a decision is no longer open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkDecision {
    /// This record's own id; it names the file.
    pub id: RecordId,
    pub at: Timestamp,
    /// The work item decided.
    pub work_item: RecordId,
    pub decision: WorkDecisionState,
}

impl WorkDecision {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        match &self.decision {
            WorkDecisionState::Rejected { reason } if reason.trim().is_empty() => {
                Err("a rejection gives a reason".into())
            }
            _ => Ok(()),
        }
    }
}

/// The schema tag of an audience rating.
pub const AUDIENCE_RATING_SCHEMA: &str = "athena.audience-rating/1";

/// The lowest and highest rating.
pub const RATING_RANGE: std::ops::RangeInclusive<u8> = 1..=5;

/// One finding of an audience rating: what the reader noticed, and where on the target they noticed it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudienceFinding {
    /// One or two sentences.
    pub text: String,
    /// The evidence: a quoted passage, or the heading or place on the target where it was found.
    pub source: String,
}

/// `athena.audience-rating/1`: how a reader of one kind rated a shipped outcome. Ratings are never edited; the
/// latest one by `at` is the current one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudienceRating {
    pub id: RecordId,
    pub at: Timestamp,
    /// What was read: the URL of the page.
    pub target: String,
    /// Who read it, such as `newcomer`.
    pub audience: String,
    /// 1 (the reader could not use it) to 5 (the reader got what they came for).
    pub rating: u8,
    pub findings: Vec<AudienceFinding>,
}

impl AudienceRating {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if self.target.trim().is_empty() || self.audience.trim().is_empty() {
            return Err("an audience rating names its target and its audience".into());
        }
        if !RATING_RANGE.contains(&self.rating) {
            return Err(format!(
                "a rating is from {} to {}, not {}",
                RATING_RANGE.start(),
                RATING_RANGE.end(),
                self.rating
            ));
        }
        if self
            .findings
            .iter()
            .any(|f| f.text.trim().is_empty() || f.source.trim().is_empty())
        {
            return Err("every finding has text and cites its source".into());
        }
        Ok(())
    }
}

/// The schema tag of an audit.
pub const AUDIT_SCHEMA: &str = "athena.audit/1";

/// One finding of an audit: how an outcome fell short of, or served, the owner's goals, and the evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditFinding {
    /// One or two sentences.
    pub text: String,
    /// The evidence: a file and line, a command's output, or a quoted passage of the real outcome.
    pub evidence: String,
}

/// `athena.audit/1`: what the auditor found when it judged one job's real outcome against `BRIEF.md` and the
/// owner's goals. Audits are never edited; a later audit of the same job is a new record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Audit {
    pub id: RecordId,
    pub at: Timestamp,
    /// The job whose outcome was judged.
    pub target: JobId,
    /// One or two sentences: whether the outcome serves the goals.
    pub summary: String,
    pub findings: Vec<AuditFinding>,
}

impl Audit {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if self.summary.trim().is_empty() {
            return Err("an audit has a summary".into());
        }
        if self
            .findings
            .iter()
            .any(|f| f.text.trim().is_empty() || f.evidence.trim().is_empty())
        {
            return Err("every audit finding has text and cites its evidence".into());
        }
        Ok(())
    }
}

/// The schema tag of a code-health baseline.
pub const CODE_HEALTH_BASELINE_SCHEMA: &str = "athena.code-health-baseline/1";

/// `athena.code-health-baseline/1`: where the design-review cadence counts from. `landings` is how many
/// landings the history held when the baseline was written; a review is due when the history holds ten
/// more. Baselines are never edited; the latest is the newest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CodeHealthBaseline {
    pub id: RecordId,
    pub at: Timestamp,
    /// Landings (first-parent commits with an `Athena-Job` trailer) counted at this moment.
    pub landings: u32,
    /// True when this baseline closed a design review that was queued; false for the first one.
    pub review_queued: bool,
}

impl CodeHealthBaseline {
    /// The most recent of `baselines`, by time and then id; None when there are none.
    pub fn latest(baselines: &[CodeHealthBaseline]) -> Option<&CodeHealthBaseline> {
        baselines
            .iter()
            .max_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)))
    }
}
