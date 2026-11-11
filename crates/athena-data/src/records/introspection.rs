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
}

impl ProposalRecord {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
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

/// The four stages of the final acceptance chain (spec OLY-10-02) for one job, each present or missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct ImprovementCycle {
    /// The job's repair landed.
    pub repair: bool,
    /// A recorded introspection of the job names a failure class, a cause and a prevention, and code flagged
    /// nothing in it.
    pub finding: bool,
    /// A defaults proposal cites the job and at least one other job, so it rests on more than one case.
    pub candidate: bool,
    /// That proposal's newest record is accepted, or rejected with a reason.
    pub decision: bool,
}

impl ImprovementCycle {
    /// True only when all four stages are present. The default (nothing recorded) is not shown.
    pub fn shown(&self) -> bool {
        self.repair && self.finding && self.candidate && self.decision
    }

    fn present(&self) -> usize {
        [self.repair, self.finding, self.candidate, self.decision]
            .into_iter()
            .filter(|s| *s)
            .count()
    }

    /// The names of the stages that are missing.
    pub fn missing(&self) -> Vec<&'static str> {
        [
            (self.repair, "repair"),
            (self.finding, "supported finding"),
            (self.candidate, "evaluated candidate"),
            (self.decision, "adoption decision"),
        ]
        .into_iter()
        .filter(|(present, _)| !present)
        .map(|(_, name)| name)
        .collect()
    }

    /// The line `athena status` prints: `improvement cycle: shown`, or `improvement cycle: not shown` and
    /// the stages that are missing.
    pub fn line(&self) -> String {
        if self.shown() {
            "improvement cycle: shown".into()
        } else {
            format!(
                "improvement cycle: not shown (missing: {})",
                self.missing().join(", ")
            )
        }
    }
}

/// Whether the records show the final acceptance chain for any one of the `repaired` jobs (the jobs whose
/// repair landed). The result is the job with the most stages present; with no repaired job nothing is
/// present, so an empty record set is never shown.
pub fn improvement_cycle(
    repaired: &[JobId],
    introspections: &[IntrospectionRecord],
    proposals: &[ProposalRecord],
) -> ImprovementCycle {
    // The newest record of each proposal id is its current state.
    let mut newest: std::collections::BTreeMap<&str, &ProposalRecord> = Default::default();
    for p in proposals {
        if newest.get(p.id.as_str()).is_none_or(|have| have.at <= p.at) {
            newest.insert(p.id.as_str(), p);
        }
    }
    let mut best = ImprovementCycle::default();
    for job in repaired {
        let finding = introspections.iter().any(|i| {
            i.job.as_ref() == Some(job)
                && i.status == IntrospectionStatus::Recorded
                && i.failure_class.is_some()
                && i.cause.is_some()
                && i.prevention.is_some()
                && i.flags.is_empty()
        });
        let candidates: Vec<&ProposalRecord> = newest
            .values()
            .copied()
            .filter(|p| p.sources.iter().any(|s| &s.job == job))
            .filter(|p| {
                let mut jobs: Vec<&JobId> = p.sources.iter().map(|s| &s.job).collect();
                jobs.sort();
                jobs.dedup();
                jobs.len() >= 2
            })
            .collect();
        let decision = candidates.iter().any(|p| match &p.status {
            ProposalStatus::Accepted { .. } => true,
            ProposalStatus::Rejected { reason } => !reason.trim().is_empty(),
            ProposalStatus::Pending | ProposalStatus::Withdrawn => false,
        });
        let cycle = ImprovementCycle {
            repair: true,
            finding,
            candidate: !candidates.is_empty(),
            decision,
        };
        if cycle.present() > best.present() {
            best = cycle;
        }
    }
    best
}

#[cfg(test)]
mod cycle_tests {
    use super::*;

    fn job(n: u32) -> JobId {
        JobId::parse(&format!("20261029-00000{n}-abcd")).unwrap()
    }

    fn introspection(of: &JobId, flags: Vec<String>) -> IntrospectionRecord {
        IntrospectionRecord {
            id: RecordId::parse("20261029-000001-1abc").unwrap(),
            at: jiff::Timestamp::UNIX_EPOCH,
            trigger: IntrospectionTrigger::Failure,
            job: Some(of.clone()),
            intervention: None,
            class_of_work: "change".into(),
            category: "failed".into(),
            what_happened: Some("a gate ended it".into()),
            failure_class: Some(RecordId::parse("20261029-000002-2abc").unwrap()),
            new_class: false,
            earlier_cases: Vec::new(),
            decisions_matched: Vec::new(),
            cause: Some("c".into()),
            other_causes: Vec::new(),
            open_hypotheses: Vec::new(),
            contradictions: Vec::new(),
            prevention_kind: Some(PreventionKind::CodeCheck),
            prevention: Some("p".into()),
            proposed_request: Some("r".into()),
            evidence: None,
            status: IntrospectionStatus::Recorded,
            failure: None,
            flags,
            dropped_ids: Vec::new(),
            planning_stopped: false,
            from_jev: false,
            jev: None,
        }
    }

    fn proposal(jobs: &[&JobId], status: ProposalStatus, secs: i64) -> ProposalRecord {
        let at = jiff::Timestamp::from_second(secs).unwrap();
        ProposalRecord {
            record: RecordId::parse("20261029-000003-3abc").unwrap(),
            at,
            id: "D1".into(),
            kind: ProposalKind::Add,
            statement: "s".into(),
            applies_to: vec!["change".into()],
            version: ProposalVersion { from: 1, to: 2 },
            sources: jobs
                .iter()
                .map(|j| ProposalSource {
                    introspection: RecordId::parse("20261029-000001-1abc").unwrap(),
                    intervention: None,
                    job: (*j).clone(),
                    at,
                })
                .collect(),
            failure_class: RecordId::parse("20261029-000002-2abc").unwrap(),
            status,
            candidate_sha256: "0".repeat(64),
            notes: String::new(),
        }
    }

    fn accepted() -> ProposalStatus {
        ProposalStatus::Accepted { job: job(9) }
    }

    #[test]
    fn a_full_chain_is_shown() {
        let (a, b) = (job(1), job(2));
        let c = improvement_cycle(
            std::slice::from_ref(&a),
            &[introspection(&a, Vec::new())],
            &[proposal(&[&a, &b], accepted(), 1)],
        );
        assert!(c.shown(), "{}", c.line());
        assert_eq!(c.line(), "improvement cycle: shown");
    }

    #[test]
    fn a_rejection_with_a_reason_is_a_decision() {
        let (a, b) = (job(1), job(2));
        let rejected = ProposalStatus::Rejected {
            reason: "no gain on the cases".into(),
        };
        let c = improvement_cycle(
            std::slice::from_ref(&a),
            &[introspection(&a, Vec::new())],
            &[proposal(&[&a, &b], rejected, 1)],
        );
        assert!(c.shown());
    }

    #[test]
    fn each_missing_stage_is_named() {
        let (a, b) = (job(1), job(2));
        let one = std::slice::from_ref(&a);
        let good_i = introspection(&a, Vec::new());
        let good_p = proposal(&[&a, &b], accepted(), 1);

        let gi = std::slice::from_ref(&good_i);
        let gp = std::slice::from_ref(&good_p);

        let c = improvement_cycle(&[], gi, gp);
        assert!(c.missing().contains(&"repair"), "{}", c.line());

        let flagged = introspection(&a, vec!["restates the event".into()]);
        let c = improvement_cycle(one, &[flagged], gp);
        assert_eq!(c.missing(), vec!["supported finding"]);

        let single = proposal(&[&a], accepted(), 1);
        let c = improvement_cycle(one, gi, &[single]);
        assert!(c.missing().contains(&"evaluated candidate"));

        let pending = proposal(&[&a, &b], ProposalStatus::Pending, 1);
        let c = improvement_cycle(one, gi, &[pending]);
        assert_eq!(c.missing(), vec!["adoption decision"]);

        // A later withdrawal replaces the accepted record.
        let withdrawn = proposal(&[&a, &b], ProposalStatus::Withdrawn, 2);
        let c = improvement_cycle(one, gi, &[good_p.clone(), withdrawn]);
        assert_eq!(c.missing(), vec!["adoption decision"]);
        assert!(c.line().contains("not shown"));
    }

    #[test]
    fn an_empty_record_set_is_not_shown() {
        let c = improvement_cycle(&[], &[], &[]);
        assert!(!c.shown());
        assert_eq!(c.missing().len(), 4);
        assert!(c.line().starts_with("improvement cycle: not shown"));
    }
}
