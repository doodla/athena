//! Introspection, failure-class, class-label and defaults-proposal records.

use super::*;

/// The schema tag of an introspection record. The one definition: the backtest and the introspection step
/// both name it through this constant.
pub const INTROSPECTION_SCHEMA: &str = "athena.introspection/1";

/// The fields of an introspection that a language model writes (`docs/design/defaults-loop.md` section 3.2).
/// Until they are written they are pending.
pub const INTROSPECTION_PROSE_FIELDS: [&str; 6] = [
    "what_happened",
    "failure_class",
    "cause",
    "prevention_kind",
    "prevention",
    "proposed_request",
];

/// What set an introspection off (`docs/design/defaults-loop.md` section 3.1, rows 1 to 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntrospectionTrigger {
    /// A job ended `rejected`.
    Rejection,
    /// A job ended `failed`.
    Failure,
    /// A correction of a reading, or an answer to its question.
    Correction,
    /// An intervention of any other kind.
    Intervention,
}

impl IntrospectionTrigger {
    pub fn as_str(self) -> &'static str {
        match self {
            IntrospectionTrigger::Rejection => "rejection",
            IntrospectionTrigger::Failure => "failure",
            IntrospectionTrigger::Correction => "correction",
            IntrospectionTrigger::Intervention => "intervention",
        }
    }
}

/// What kind of prevention would have stopped the class. `Relax` and `Remove` are the two kinds for a rule or
/// check that was wrong: they propose loosening a gate, not adding a protection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreventionKind {
    Default,
    CodeCheck,
    DesignChange,
    /// A gate was too strict: loosen it.
    Relax,
    /// A gate should not exist: remove it.
    Remove,
}

impl PreventionKind {
    pub const ALL: [PreventionKind; 5] = [
        PreventionKind::Default,
        PreventionKind::CodeCheck,
        PreventionKind::DesignChange,
        PreventionKind::Relax,
        PreventionKind::Remove,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            PreventionKind::Default => "default",
            PreventionKind::CodeCheck => "code-check",
            PreventionKind::DesignChange => "design-change",
            PreventionKind::Relax => "relax",
            PreventionKind::Remove => "remove",
        }
    }
    /// True for the kinds that propose loosening a gate. Such a record is evidence about a check, not a case
    /// for a new default or a warning to a producer.
    pub fn loosens(self) -> bool {
        matches!(self, PreventionKind::Relax | PreventionKind::Remove)
    }
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == text)
    }
}

/// Whether the introspection was written. `acted-on` is a later build.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntrospectionStatus {
    Recorded,
    Failed,
}

impl IntrospectionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            IntrospectionStatus::Recorded => "recorded",
            IntrospectionStatus::Failed => "failed",
        }
    }
}

/// What the model that wrote the prose cost. The probabilities and confidence of Jev are in
/// [`IntrospectionJev`], not here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntrospectionEvidence {
    pub model: String,
    pub cost_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

/// Jev's answer to the failure-class question, as it came back: the option it chose, the probability of each
/// option, and its confidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JevClassAnswer {
    /// A registry class id, or `new`.
    pub choice: String,
    pub probabilities: std::collections::BTreeMap<String, f64>,
    pub confidence: f64,
}

/// The fixed outcomes of the "context that would have prevented it" question, in the order Jev is shown them.
pub const CONTEXT_PREVENTION_OUTCOMES: [&str; 7] = [
    "system-facts-line",
    "failure-class",
    "brief-assembly",
    "vocabulary-term",
    "spec-fix",
    "structure",
    "none",
];

/// The start of the outcome reason of a job that failed in planning, before any code was written.
pub const PLANNING_STOPPED_PREFIX: &str = "planning stopped before any code was written:";

/// The words of the outcome reason of a job whose plan still needed revising in the last planning round. A
/// retry resumes planning from the kept plan only for a planning stop whose reason holds this text.
pub const PLANNING_CAP_REASON: &str = "the plan still needed revising in the last round";

/// Jev's answer to the post-mortem question. Exactly one of the two fields is set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JevContextAnswer {
    /// One of [`CONTEXT_PREVENTION_OUTCOMES`].
    pub outcome: Option<String>,
    /// Why there is no outcome: the answer was malformed or not on the list.
    pub error: Option<String>,
}

/// What Jev said about an event, recorded in shadow so that it can be scored later (`defaults-loop.md`
/// sections 3.3 and 12). Nothing here decides anything: the recorded class, status and outcome come from the
/// language model and from code. A call that failed is recorded with `error` and no answers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntrospectionJev {
    /// The model asked, such as `typesafe-ai/jev`.
    pub model: String,
    /// The failure-class question: a choice over the registry ids plus `new`.
    pub class: Option<JevClassAnswer>,
    /// The is-repeat question: the probability that the event is a repeat of an earlier one.
    pub repeat_probability: Option<f64>,
    /// The post-mortem question: what one piece of context would have prevented the event. Absent when Jev
    /// gave no answer to it or the whole call failed; an answer that is not on the fixed list is kept as an
    /// error and blocks nothing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_prevention: Option<JevContextAnswer>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    /// The gateway's cost for the call, when it reported one.
    pub cost_usd: Option<f64>,
    /// Why the call produced no answers (no secret, no response text beyond a short excerpt).
    pub error: Option<String>,
}

/// The most entries one of the cause, hypothesis or contradiction lists of an introspection may hold.
pub const INTROSPECTION_LIST_CAP: usize = 5;

/// `athena.introspection/1`: why one event happened and what would have stopped its class, not only this
/// instance. One per triggering event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntrospectionRecord {
    pub id: RecordId,
    pub at: Timestamp,
    pub trigger: IntrospectionTrigger,
    /// The job the event concerns, if any.
    pub job: Option<JobId>,
    /// The intervention of the event, for a correction or an intervention. A rejection or a failure has none.
    pub intervention: Option<RecordId>,
    pub class_of_work: String,
    /// The intervention category, or `rejected` or `failed` for a job.
    pub category: String,
    pub what_happened: Option<String>,
    /// A class of the registry; written with `new_class` when this record introduced it.
    pub failure_class: Option<RecordId>,
    pub new_class: bool,
    /// Introspections of earlier events in the same class, by code.
    pub earlier_cases: Vec<RecordId>,
    /// Typed decisions whose signals the event matched, by code.
    pub decisions_matched: Vec<String>,
    pub cause: Option<String>,
    /// Other causes that contributed besides `cause`. Absent on older records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub other_causes: Vec<String>,
    /// Hypotheses the given facts cannot confirm or rule out. Absent on older records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open_hypotheses: Vec<String>,
    /// Input facts that point against `cause`. Absent on older records.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contradictions: Vec<String>,
    pub prevention_kind: Option<PreventionKind>,
    pub prevention: Option<String>,
    pub proposed_request: Option<String>,
    pub evidence: Option<IntrospectionEvidence>,
    pub status: IntrospectionStatus,
    /// Why the record is `failed`.
    pub failure: Option<String>,
    /// Problems code found in the model's answer, such as a cause that restates the event.
    pub flags: Vec<String>,
    /// Ids the model returned that were not on the list code passed in; dropped, not repaired.
    pub dropped_ids: Vec<String>,
    /// True when the job failed in planning, before any code was written. Absent (false) otherwise and on
    /// records written before the marker existed.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub planning_stopped: bool,
    /// Always false: the class and prevention kind came from a language model, not from Jev. Jev runs in shadow.
    pub from_jev: bool,
    /// Jev's shadow answers to the two introspection questions. Absent on records written before step 5 and
    /// when Jev is not asked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jev: Option<IntrospectionJev>,
}

impl IntrospectionRecord {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        for list in [
            &self.other_causes,
            &self.open_hypotheses,
            &self.contradictions,
        ] {
            if list.len() > INTROSPECTION_LIST_CAP {
                return Err(format!(
                    "an introspection list holds at most {INTROSPECTION_LIST_CAP} entries"
                ));
            }
            if list.iter().any(|entry| entry.trim().is_empty()) {
                return Err("an introspection list holds no blank entry".into());
            }
        }
        match self.status {
            IntrospectionStatus::Failed if self.failure.is_none() => {
                Err("a failed introspection says why".into())
            }
            IntrospectionStatus::Recorded
                if self.what_happened.is_none()
                    || self.cause.is_none()
                    || self.failure_class.is_none()
                    || self.prevention_kind.is_none()
                    || self.prevention.is_none()
                    || self.proposed_request.is_none() =>
            {
                Err("a recorded introspection has every field written".into())
            }
            _ if self.trigger != IntrospectionTrigger::Rejection
                && self.trigger != IntrospectionTrigger::Failure
                && self.intervention.is_none() =>
            {
                Err("a correction or an intervention names its intervention".into())
            }
            _ if self.from_jev => Err("Jev only runs in shadow; from_jev stays false".into()),
            _ => match &self.jev {
                Some(jev)
                    if jev.error.is_none()
                        && (jev.class.is_none() || jev.repeat_probability.is_none()) =>
                {
                    Err("a Jev record without an error holds both answers".into())
                }
                Some(jev)
                    if jev.error.is_some()
                        && (jev.class.is_some() || jev.repeat_probability.is_some()) =>
                {
                    Err("a Jev record with an error holds no answers".into())
                }
                Some(jev) if jev.error.is_some() && jev.context_prevention.is_some() => {
                    Err("a Jev record with an error holds no answers".into())
                }
                Some(IntrospectionJev {
                    context_prevention: Some(answer),
                    ..
                }) if answer.outcome.is_some() == answer.error.is_some()
                    || answer
                        .outcome
                        .as_deref()
                        .is_some_and(|o| !CONTEXT_PREVENTION_OUTCOMES.contains(&o)) =>
                {
                    Err(
                        "a context-prevention answer holds one outcome from the list, or an error"
                            .into(),
                    )
                }
                _ => Ok(()),
            },
        }
    }
}

/// Where a class stands in the registry. Only `proposed` is written by code; the user renames or merges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClassStatus {
    Proposed,
}

/// `athena.failure-class/1`: one entry of the class registry (`docs/design/defaults-loop.md` section 3.4).
/// The registry is the set of these records; no file in the repository holds it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureClass {
    /// Assigned by code.
    pub id: RecordId,
    pub at: Timestamp,
    pub name: String,
    /// One line.
    pub description: String,
    pub status: ClassStatus,
    /// The introspection that proposed it.
    pub proposed_by: RecordId,
}

/// Who wrote a class label (`docs/design/defaults-loop.md` section 10.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Labeler {
    /// A person named the class.
    Person,
    /// A stronger model named the class and a person spot-checked it.
    StrongerModel,
    /// Code derived the class from what happened afterwards, citing the records in the label's `basis`. No
    /// person or model judged it, and any label from a person or a stronger model outranks it.
    Evidence,
}

impl Labeler {
    pub fn as_str(self) -> &'static str {
        match self {
            Labeler::Person => "person",
            Labeler::StrongerModel => "stronger-model",
            Labeler::Evidence => "evidence",
        }
    }

    /// Whether a person or a stronger model gave the label. Such a label always beats an evidence-derived one.
    pub fn is_judgement(self) -> bool {
        !matches!(self, Labeler::Evidence)
    }
}

/// The kind of later event that an evidence-derived class label rests on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceKind {
    /// The retry that followed the introspection's proposed request landed, or failed for the same cause.
    RetryOutcome,
    /// After a prevention for the class landed, the class stopped recurring or kept recurring.
    RecurrenceAfterPrevention,
    /// The critic's or the verifier's findings on the same job name the same cause.
    SameCauseFindings,
}

impl EvidenceKind {
    pub const ALL: [EvidenceKind; 3] = [
        EvidenceKind::RetryOutcome,
        EvidenceKind::RecurrenceAfterPrevention,
        EvidenceKind::SameCauseFindings,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            EvidenceKind::RetryOutcome => "retry-outcome",
            EvidenceKind::RecurrenceAfterPrevention => "recurrence-after-prevention",
            EvidenceKind::SameCauseFindings => "same-cause-findings",
        }
    }
}

/// One piece of evidence behind a derived label: its kind, the ids of the records it rests on (job ids,
/// introspection ids and proposal record ids), and one line saying what they show.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabelBasis {
    pub kind: EvidenceKind,
    pub records: Vec<String>,
    pub detail: String,
}

/// `athena.class-label/1`: the failure class a reviewer assigned to one recorded introspection, written before
/// Jev's answers are read (`docs/design/defaults-loop.md` section 10.2). Introspections without such a record
/// are unreviewed and are not part of the labeled set. A later label for the same introspection replaces an
/// earlier one (the newest by time, then id, counts), except that a label from a person or a stronger model
/// always beats one derived from evidence, whichever is newer. `athena review` writes the person's records and
/// `athena score-jev --derive` writes the evidence-derived ones; the introspection itself is never edited.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassLabel {
    pub id: RecordId,
    pub at: Timestamp,
    /// The introspection that was reviewed.
    pub introspection: RecordId,
    /// The class the reviewer assigned: a registry id, or `None` when no registry class fits.
    pub class: Option<RecordId>,
    pub labeled_by: Labeler,
    /// Whether the reviewer judged the event a repeat of an earlier case with the same cause. Absent when the
    /// reviewer did not say; such a label is used for the class score only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_repeat: Option<bool>,
    /// A one-line note from the reviewer. Absent when none was given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// What an evidence-derived label rests on: one entry per kind of evidence, each citing its records.
    /// Required (and non-empty) when `labeled_by` is `evidence`, and absent otherwise. A label written before
    /// evidence labels existed has none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub basis: Vec<LabelBasis>,
}

impl ClassLabel {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        match (self.labeled_by, self.basis.is_empty()) {
            (Labeler::Evidence, true) => {
                Err("an evidence-derived label cites the records it rests on".into())
            }
            (Labeler::Evidence, false) if self.class.is_none() => {
                Err("an evidence-derived label names a class".into())
            }
            (Labeler::Evidence, false)
                if self.basis.iter().any(|b| {
                    b.records.is_empty()
                        || b.records.iter().any(|r| r.trim().is_empty())
                        || b.detail.trim().is_empty()
                }) =>
            {
                Err("each piece of evidence cites records and says what they show".into())
            }
            (Labeler::Person | Labeler::StrongerModel, false) => {
                Err("only an evidence-derived label has a basis".into())
            }
            _ => Ok(()),
        }
    }
}

/// The schema tag of a defaults proposal.
pub const PROPOSAL_SCHEMA: &str = "athena.defaults-proposal/1";

/// Whether a proposal adds a default or amends one that exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProposalKind {
    Add,
    Amend,
}

impl ProposalKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ProposalKind::Add => "add",
            ProposalKind::Amend => "amend",
        }
    }
}

/// The model version a proposal is built on, and the one it would produce (`to` is `from + 1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalVersion {
    pub from: u32,
    pub to: u32,
}

/// One record a proposal cites: an introspection, the intervention it extends (if any), and its job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalSource {
    pub introspection: RecordId,
    pub intervention: Option<RecordId>,
    pub job: JobId,
    /// When the cited introspection was recorded.
    pub at: Timestamp,
}

/// Where a proposal stands. A change of status is a new record, never an edit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case")]
pub enum ProposalStatus {
    Pending,
    /// The job that applies it to `defaults.toml`.
    Accepted {
        job: JobId,
    },
    Rejected {
        reason: String,
    },
    Withdrawn,
}

impl ProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProposalStatus::Pending => "pending",
            ProposalStatus::Accepted { .. } => "accepted",
            ProposalStatus::Rejected { .. } => "rejected",
            ProposalStatus::Withdrawn => "withdrawn",
        }
    }
}

/// `athena.defaults-proposal/1`: a proposed default, written by code when introspections of one failure class
/// reach the repeat threshold across distinct jobs (`docs/design/defaults-loop.md` sections 8 and 9). It is a
/// record, never an edit to `defaults.toml`. Each status change is a further record with the same `id`; the
/// newest by time is current. Without Jev there are no assumptions, signals or probabilities, and `notes` says so.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalRecord {
    /// This record's own id; it names the file.
    pub record: RecordId,
    pub at: Timestamp,
    /// The default's id: new for `add`, existing for `amend`.
    pub id: String,
    pub kind: ProposalKind,
    pub statement: String,
    pub applies_to: Vec<String>,
    pub version: ProposalVersion,
    pub sources: Vec<ProposalSource>,
    /// The failure class whose repeats produced it.
    pub failure_class: RecordId,
    pub status: ProposalStatus,
    /// Hash of the full candidate `defaults.toml` that passed `DefaultsModel::parse`.
    pub candidate_sha256: String,
    /// What this build leaves out (no Jev: no assumptions, signals or evidence probabilities).
    pub notes: String,
    /// The recorded result of testing the candidate against the baseline. Absent until someone records one
    /// (and on every record written before the field existed); only its presence makes the candidate
    /// "evaluated" for [`improvement_cycle`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evaluation: Option<ProposalEvaluation>,
}

/// A baseline-versus-candidate result for one proposal, in the shape of `method_experiment`: the same
/// representative cases run once with the current defaults (the baseline arm) and once with the candidate
/// (the candidate arm), counting the cases each arm got wrong. Fewer failures is better.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProposalEvaluation {
    /// The representative cases both arms ran, such as job ids. Never the cases that triggered the proposal
    /// alone; a case list that is empty is no evaluation.
    pub cases: Vec<String>,
    /// How many of `cases` the baseline arm got wrong.
    pub baseline_failures: u32,
    /// How many of `cases` the candidate arm got wrong.
    pub candidate_failures: u32,
}

impl ProposalEvaluation {
    /// Why this evaluation is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if self.cases.is_empty() || self.cases.iter().any(|c| c.trim().is_empty()) {
            return Err("an evaluation lists the cases both arms ran".into());
        }
        let most = self.cases.len() as u64;
        if u64::from(self.baseline_failures) > most || u64::from(self.candidate_failures) > most {
            return Err("an arm cannot fail more cases than were run".into());
        }
        Ok(())
    }
}

impl ProposalRecord {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(evaluation) = &self.evaluation {
            evaluation.validate()?;
        }
        if self.id.trim().is_empty() || self.statement.trim().is_empty() {
            return Err("a proposal has an id and a statement".into());
        }
        if self.applies_to.is_empty() || self.applies_to.iter().any(|a| a.trim().is_empty()) {
            return Err("a proposal lists the classes it applies to".into());
        }
        if self.version.from.checked_add(1) != Some(self.version.to) {
            return Err("a proposal's version is the base plus one".into());
        }
        if self.sources.is_empty() {
            return Err("a proposal cites the records that support it".into());
        }
        if self.candidate_sha256.len() != 64
            || !self.candidate_sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("candidate_sha256 is a SHA-256 in hexadecimal".into());
        }
        match &self.status {
            ProposalStatus::Rejected { reason } if reason.trim().is_empty() => {
                Err("a rejection gives a reason".into())
            }
            _ => Ok(()),
        }
    }
}

/// One link of the final acceptance chain (`docs/spec.toml` OLY-10-02), in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleStage {
    /// A job landed that an introspection or a proposal names.
    Repair,
    /// A recorded introspection that assigned a failure class.
    Finding,
    /// A defaults proposal linked to such a finding.
    Candidate,
    /// A linked proposal that has a recorded evaluation result.
    Evaluated,
    /// A linked, evaluated proposal that was accepted or rejected.
    Adoption,
}

impl CycleStage {
    pub const ALL: [CycleStage; 5] = [
        CycleStage::Repair,
        CycleStage::Finding,
        CycleStage::Candidate,
        CycleStage::Evaluated,
        CycleStage::Adoption,
    ];

    /// The words `athena status` prints for the stage.
    pub fn label(self) -> &'static str {
        match self {
            CycleStage::Repair => "repair",
            CycleStage::Finding => "process finding",
            CycleStage::Candidate => "candidate improvement",
            CycleStage::Evaluated => "evaluated candidate",
            CycleStage::Adoption => "adoption decision",
        }
    }
}

/// Which stages of the chain the records show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImprovementCycle {
    /// The stages no record supports, in chain order. Empty only when the whole chain is shown.
    pub missing: Vec<CycleStage>,
}

impl ImprovementCycle {
    pub fn shown(&self) -> bool {
        self.missing.is_empty()
    }
}

/// Whether the records show the chain of OLY-10-02: a landed repair, a process finding, a candidate improvement
/// linked to that finding, an evaluation result for that candidate, and an adoption decision on it.
///
/// `landed` is the jobs that landed; `introspections` and `proposals` are all introspections and the current
/// record of each proposal. A proposal is linked to a finding when it cites the finding's introspection id or
/// is about the finding's failure class; a finding about another class and another introspection does not
/// count for it. The evaluated stage is not inferred from how many jobs the proposal cites (those are the
/// cases that triggered it, not cases it was tested on): it needs the proposal's `evaluation` field, which
/// holds a recorded baseline-versus-candidate result. Nothing at all gives every stage missing, never success.
pub fn improvement_cycle(
    landed: &[JobId],
    introspections: &[IntrospectionRecord],
    proposals: &[&ProposalRecord],
) -> ImprovementCycle {
    let findings: Vec<&IntrospectionRecord> = introspections
        .iter()
        .filter(|i| i.status == IntrospectionStatus::Recorded && i.failure_class.is_some())
        .collect();
    let repair = landed.iter().any(|job| {
        introspections.iter().any(|i| i.job.as_ref() == Some(job))
            || proposals
                .iter()
                .any(|p| p.sources.iter().any(|s| &s.job == job))
    });
    let linked: Vec<&ProposalRecord> = proposals
        .iter()
        .copied()
        .filter(|p| {
            findings.iter().any(|f| {
                p.sources.iter().any(|s| s.introspection == f.id)
                    || f.failure_class.as_ref() == Some(&p.failure_class)
            })
        })
        .collect();
    // The evaluation signal is `ProposalRecord::evaluation`, and only a valid one counts.
    let evaluated: Vec<&ProposalRecord> = linked
        .iter()
        .copied()
        .filter(|p| p.evaluation.as_ref().is_some_and(|e| e.validate().is_ok()))
        .collect();
    let decided = evaluated.iter().any(|p| {
        matches!(
            p.status,
            ProposalStatus::Accepted { .. } | ProposalStatus::Rejected { .. }
        )
    });
    let reached = [
        repair,
        !findings.is_empty(),
        !linked.is_empty(),
        !evaluated.is_empty(),
        decided,
    ];
    ImprovementCycle {
        missing: CycleStage::ALL
            .into_iter()
            .zip(reached)
            .filter(|(_, reached)| !reached)
            .map(|(stage, _)| stage)
            .collect(),
    }
}

#[cfg(test)]
mod improvement_cycle_tests {
    use super::*;

    fn rid(n: u32) -> RecordId {
        RecordId::parse(&format!("20261101-1000{:02}-{:04x}", n % 60, n)).unwrap()
    }

    fn job(n: u32) -> JobId {
        JobId::parse(&format!("20261101-2000{:02}-{:04x}", n % 60, n)).unwrap()
    }

    fn ts() -> Timestamp {
        "2026-11-01T10:00:00Z".parse().unwrap()
    }

    /// A recorded introspection of `class` about `job`, shaped like the ones the introspection step writes.
    fn introspection(n: u32, class: u32, about: u32) -> IntrospectionRecord {
        IntrospectionRecord {
            id: rid(n),
            at: ts(),
            trigger: IntrospectionTrigger::Rejection,
            job: Some(job(about)),
            intervention: None,
            class_of_work: "code".into(),
            category: "rejected".into(),
            what_happened: Some("The check failed.".into()),
            failure_class: Some(rid(class)),
            new_class: false,
            earlier_cases: Vec::new(),
            decisions_matched: Vec::new(),
            cause: Some("A cause.".into()),
            other_causes: Vec::new(),
            open_hypotheses: Vec::new(),
            contradictions: Vec::new(),
            prevention_kind: Some(PreventionKind::CodeCheck),
            prevention: Some("A check.".into()),
            proposed_request: Some("Add it.".into()),
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

    /// A proposal like a real one: three sources in three distinct jobs, all introspections of `class`.
    fn proposal(id: &str, class: u32, intros: [u32; 3]) -> ProposalRecord {
        ProposalRecord {
            record: rid(500 + class),
            at: ts(),
            id: id.into(),
            kind: ProposalKind::Add,
            statement: "A new default.".into(),
            applies_to: vec!["code".into()],
            version: ProposalVersion { from: 1, to: 2 },
            sources: intros
                .iter()
                .map(|n| ProposalSource {
                    introspection: rid(*n),
                    intervention: None,
                    job: job(*n),
                    at: ts(),
                })
                .collect(),
            failure_class: rid(class),
            status: ProposalStatus::Pending,
            candidate_sha256: "a".repeat(64),
            notes: String::new(),
            evaluation: None,
        }
    }

    fn evaluation() -> ProposalEvaluation {
        ProposalEvaluation {
            cases: vec!["20261020-100000-aaaa".into(), "20261020-110000-bbbb".into()],
            baseline_failures: 2,
            candidate_failures: 0,
        }
    }

    fn findings(class: u32) -> Vec<IntrospectionRecord> {
        (1..=3).map(|n| introspection(n, class, n)).collect()
    }

    fn missing(
        landed: &[JobId],
        intros: &[IntrospectionRecord],
        proposals: &[&ProposalRecord],
    ) -> Vec<CycleStage> {
        improvement_cycle(landed, intros, proposals).missing
    }

    #[test]
    fn introspection_cycle_of_an_empty_record_set_is_not_shown() {
        let cycle = improvement_cycle(&[], &[], &[]);
        assert!(!cycle.shown());
        assert_eq!(cycle.missing, CycleStage::ALL.to_vec());
    }

    #[test]
    fn introspection_cycle_counts_a_proposal_without_an_evaluation_as_not_evaluated() {
        // Three distinct source jobs, as every real proposal has: that alone is no evaluation.
        let mut p = proposal("d", 9, [1, 2, 3]);
        assert_eq!(p.sources.len(), 3);
        p.status = ProposalStatus::Accepted { job: job(40) };
        assert_eq!(
            missing(&[job(1)], &findings(9), &[&p]),
            vec![CycleStage::Evaluated, CycleStage::Adoption]
        );
    }

    #[test]
    fn introspection_cycle_is_shown_when_the_linked_proposal_is_evaluated_and_decided() {
        let mut p = proposal("d", 9, [1, 2, 3]);
        p.evaluation = Some(evaluation());
        p.status = ProposalStatus::Rejected {
            reason: "No gain.".into(),
        };
        let cycle = improvement_cycle(&[job(1)], &findings(9), &[&p]);
        assert!(cycle.shown(), "{:?}", cycle.missing);
        // Evaluated but still pending: the decision is the missing stage.
        p.status = ProposalStatus::Pending;
        assert_eq!(
            missing(&[job(1)], &findings(9), &[&p]),
            vec![CycleStage::Adoption]
        );
    }

    #[test]
    fn introspection_cycle_ignores_a_finding_about_another_class_and_introspection() {
        let mut p = proposal("d", 9, [1, 2, 3]);
        p.evaluation = Some(evaluation());
        p.status = ProposalStatus::Accepted { job: job(40) };
        // Findings 11..13 are about class 8; the proposal cites introspections 1..3 and class 9.
        let other: Vec<_> = (11..=13).map(|n| introspection(n, 8, n)).collect();
        assert_eq!(
            missing(&[job(11)], &other, &[&p]),
            vec![
                CycleStage::Candidate,
                CycleStage::Evaluated,
                CycleStage::Adoption
            ]
        );
    }

    #[test]
    fn introspection_cycle_links_by_the_cited_introspection_when_the_class_differs() {
        let mut p = proposal("d", 9, [1, 2, 3]);
        p.evaluation = Some(evaluation());
        p.status = ProposalStatus::Accepted { job: job(40) };
        let mut intros = findings(9);
        intros[0].failure_class = Some(rid(8));
        let cycle = improvement_cycle(&[job(2)], &intros[..1], &[&p]);
        assert!(cycle.shown(), "{:?}", cycle.missing);
    }

    #[test]
    fn introspection_cycle_reports_each_single_missing_stage() {
        let mut p = proposal("d", 9, [1, 2, 3]);
        p.evaluation = Some(evaluation());
        p.status = ProposalStatus::Accepted { job: job(40) };
        let intros = findings(9);
        let landed = [job(1)];
        assert!(improvement_cycle(&landed, &intros, &[&p]).shown());
        // No landed job.
        assert_eq!(missing(&[], &intros, &[&p]), vec![CycleStage::Repair]);
        // A landed job that nothing names.
        assert_eq!(
            missing(&[job(30)], &intros, &[&p]),
            vec![CycleStage::Repair]
        );
        // A failed introspection is no finding; with it the proposal is linked to nothing either.
        let mut failed = intros.clone();
        for f in &mut failed {
            f.status = IntrospectionStatus::Failed;
        }
        assert_eq!(
            missing(&landed, &failed, &[&p]),
            vec![
                CycleStage::Finding,
                CycleStage::Candidate,
                CycleStage::Evaluated,
                CycleStage::Adoption
            ]
        );
        // No proposal.
        assert_eq!(
            missing(&landed, &intros, &[]),
            vec![
                CycleStage::Candidate,
                CycleStage::Evaluated,
                CycleStage::Adoption
            ]
        );
        // An evaluation with no cases is no evaluation.
        let mut empty = p.clone();
        empty.evaluation.as_mut().unwrap().cases.clear();
        assert_eq!(
            missing(&landed, &intros, &[&empty]),
            vec![CycleStage::Evaluated, CycleStage::Adoption]
        );
    }

    #[test]
    fn introspection_cycle_replay_over_mixed_proposals_finds_both_outcomes() {
        let intros = findings(9);
        let mut evaluated = proposal("with-eval", 9, [1, 2, 3]);
        evaluated.evaluation = Some(evaluation());
        let plain = proposal("no-eval", 9, [1, 2, 3]);
        let other = proposal("other-class", 7, [21, 22, 23]);
        let all = [&evaluated, &plain, &other];
        let results: Vec<bool> = all
            .iter()
            .map(|p| !missing(&[job(1)], &intros, &[*p]).contains(&CycleStage::Evaluated))
            .collect();
        assert_eq!(results, vec![true, false, false]);
        assert!(results.iter().any(|r| *r) && results.iter().any(|r| !*r));
        for p in all {
            let jobs: std::collections::BTreeSet<_> =
                p.sources.iter().map(|s| s.job.as_str()).collect();
            assert_eq!(jobs.len(), 3);
        }
    }

    #[test]
    fn introspection_proposal_evaluation_round_trips_and_is_validated() {
        let mut p = proposal("d", 9, [1, 2, 3]);
        p.evaluation = Some(evaluation());
        let record = AnyRecord::DefaultsProposal(Box::new(p.clone()));
        assert!(record.to_json().contains("\"candidate_failures\": 0"));
        assert_eq!(
            AnyRecord::parse("x", record.to_json().as_bytes()).unwrap(),
            record
        );
        p.evaluation.as_mut().unwrap().baseline_failures = 9;
        assert!(p.validate().is_err());
        p.evaluation = None;
        let json = AnyRecord::DefaultsProposal(Box::new(p)).to_json();
        assert!(!json.contains("evaluation"));
    }
}
