//! The record types and where each one lives (design §5; the host-error, decision-state and
//! default-expectations types come from `docs/design/decision-sets.md`).
//!
//! A record is a pretty-printed JSON file under `records/`, written once and never modified (D06). Its
//! `schema` field names its type and version, and [`AnyRecord`] parses any record with unknown fields
//! denied. Every record has exactly one canonical location, derived from its own fields, so the gate can
//! check that a file sits where its content says it belongs (M5).
//!
//! The records-branch types of `docs/design/records-history.md` (D111) are in [`crate::history`], not here.

use crate::id::{JobId, RecordId};
use crate::quantity::Quantity;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The agent roles: the three that work on a job, and the steward, the audience and the auditor, which run
/// beside them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Reader,
    Producer,
    Verifier,
    Steward,
    Audience,
    Auditor,
    /// Plans a job's work before the producer starts (`docs/design/job-planning.md`).
    Planner,
    /// Reviews the planner's plan.
    Critic,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Planner => "planner",
            Role::Critic => "critic",
            Role::Reader => "reader",
            Role::Producer => "producer",
            Role::Verifier => "verifier",
            Role::Steward => "steward",
            Role::Audience => "audience",
            Role::Auditor => "auditor",
        }
    }
    pub fn parse(text: &str) -> Option<Role> {
        match text {
            "reader" => Some(Role::Reader),
            "producer" => Some(Role::Producer),
            "verifier" => Some(Role::Verifier),
            "steward" => Some(Role::Steward),
            "audience" => Some(Role::Audience),
            "auditor" => Some(Role::Auditor),
            "planner" => Some(Role::Planner),
            "critic" => Some(Role::Critic),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriterionKind {
    /// Checkable by reading the change.
    Content,
    /// Checkable only by running something; its evidence must cite commands that ran (D43).
    Behavior,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliverableAction {
    Create,
    Modify,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Blocker,
    Major,
    Minor,
    Nit,
}

/// A job's terminal (or waiting) state, as recorded in its outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeState {
    Landed,
    /// Verified, but the fast-forward was refused by local edits; `go` retries the landing.
    Blocked,
    Rejected,
    Failed,
    /// The producer reached its deadline. Its edits are kept and a progress note lists the changed files;
    /// `athena retry` resumes from them. Records written before this state existed never carry it.
    TimedOut,
    Stale,
    Dropped,
    Declined,
    Abandoned,
    /// The bootstrap's own outcome, which closes its range of unverified commits.
    Closed,
}

impl OutcomeState {
    pub fn as_str(self) -> &'static str {
        match self {
            OutcomeState::Landed => "landed",
            OutcomeState::Blocked => "blocked",
            OutcomeState::Rejected => "rejected",
            OutcomeState::Failed => "failed",
            OutcomeState::TimedOut => "timed_out",
            OutcomeState::Stale => "stale",
            OutcomeState::Dropped => "dropped",
            OutcomeState::Declined => "declined",
            OutcomeState::Abandoned => "abandoned",
            OutcomeState::Closed => "closed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvocationOutcome {
    Success,
    Error,
    /// Terminated by a budget, the kill switch, or the death of its parent.
    Killed,
    /// Terminated because its own deadline passed; the detail names the deadline.
    TimedOut,
    /// The process and its state were lost; recorded later with bounds by `athena close`.
    Missing,
}

/// The brief's own intervention categories (O3), plus `other`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InterventionCategory {
    DidWorkThemselves,
    CorrectedOutput,
    GaveDetailedInstructions,
    OverrodeDecision,
    AnsweredQuestion,
    CarriedState,
    FixedEnvironment,
    /// A mistake Athena or a model made, logged so status and the audit can count it per class and per cause.
    Mistake,
    Other,
}

impl InterventionCategory {
    pub const ALL: [InterventionCategory; 9] = [
        InterventionCategory::DidWorkThemselves,
        InterventionCategory::CorrectedOutput,
        InterventionCategory::GaveDetailedInstructions,
        InterventionCategory::OverrodeDecision,
        InterventionCategory::AnsweredQuestion,
        InterventionCategory::CarriedState,
        InterventionCategory::FixedEnvironment,
        InterventionCategory::Mistake,
        InterventionCategory::Other,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            InterventionCategory::DidWorkThemselves => "did-work-themselves",
            InterventionCategory::CorrectedOutput => "corrected-output",
            InterventionCategory::GaveDetailedInstructions => "gave-detailed-instructions",
            InterventionCategory::OverrodeDecision => "overrode-decision",
            InterventionCategory::AnsweredQuestion => "answered-question",
            InterventionCategory::CarriedState => "carried-state",
            InterventionCategory::FixedEnvironment => "fixed-environment",
            InterventionCategory::Mistake => "mistake",
            InterventionCategory::Other => "other",
        }
    }
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.as_str() == text)
    }
}

/// The text prefix that marks a legacy mistake: an intervention logged with category `other` before the
/// `mistake` category existed.
pub const LEGACY_MISTAKE_PREFIX: &str = "Mistake:";

/// Whether an intervention is a mistake. Two rules, and no record is ever rewritten:
/// the category is `mistake`; or the category is `other` and `what` starts with "Mistake:" (case sensitive;
/// the legacy rule for records written before the category existed). Any other category is not a mistake,
/// whatever its text.
pub fn is_mistake(category: InterventionCategory, what: &str) -> bool {
    match category {
        InterventionCategory::Mistake => true,
        InterventionCategory::Other => what.starts_with(LEGACY_MISTAKE_PREFIX),
        _ => false,
    }
}

impl Intervention {
    /// Whether this intervention is a mistake; see [`is_mistake`].
    pub fn is_mistake(&self) -> bool {
        is_mistake(self.category, &self.what)
    }
}

/// Whether `text` names the job `id` as a whole token: the full id, not preceded or followed by a letter, a
/// digit or a hyphen, so an id that only starts with `id` (or ends with it) does not match. Used to read, from
/// the free text of later records, which landed job they cite as a cause.
pub fn text_cites_job(text: &str, id: &JobId) -> bool {
    let needle = id.as_str();
    let token = |c: char| c.is_ascii_alphanumeric() || c == '-';
    text.match_indices(needle).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + needle.len()..].chars().next();
        !before.is_some_and(token) && !after.is_some_and(token)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionSource {
    /// Recorded by the user with `athena intervene`.
    User,
    /// Captured by Athena itself, such as a one-line correction of a reading.
    Observed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrectionKind {
    Correction,
    /// An answer to the reader's question.
    Answer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    Request,
    Bootstrap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcceptedBy {
    /// The user pressed Enter or ran `go`.
    User,
    /// `--yes` skipped the prompt; nobody looked, so the reading's acceptance is unresolved.
    Flag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeStatus {
    Added,
    Modified,
    Deleted,
    TypeChanged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingSource {
    Verifier,
    Athena,
    /// The job-planning critic; its critique reuses [`Finding`].
    Critic,
}

// ---------------------------------------------------------------------------------------------------------
// Shared parts

/// Forecast percentiles of a quantity. Parsing refuses anything but 0 < p10 <= p50 <= p90 (M4).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawQuantiles", into = "RawQuantiles")]
pub struct Quantiles {
    p10: f64,
    p50: f64,
    p90: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawQuantiles {
    p10: f64,
    p50: f64,
    p90: f64,
}

impl Quantiles {
    pub fn new(p10: f64, p50: f64, p90: f64) -> Result<Self, String> {
        if !(p10.is_finite() && p50.is_finite() && p90.is_finite()) {
            return Err("quantiles must be finite".into());
        }
        if !(p10 > 0.0 && p10 <= p50 && p50 <= p90) {
            return Err(format!(
                "quantiles must satisfy 0 < p10 <= p50 <= p90, got {p10}, {p50}, {p90}"
            ));
        }
        Ok(Quantiles { p10, p50, p90 })
    }
    pub fn p10(&self) -> f64 {
        self.p10
    }
    pub fn p50(&self) -> f64 {
        self.p50
    }
    pub fn p90(&self) -> f64 {
        self.p90
    }
}

impl TryFrom<RawQuantiles> for Quantiles {
    type Error = String;
    fn try_from(raw: RawQuantiles) -> Result<Self, Self::Error> {
        Quantiles::new(raw.p10, raw.p50, raw.p90)
    }
}

impl From<Quantiles> for RawQuantiles {
    fn from(q: Quantiles) -> Self {
        RawQuantiles {
            p10: q.p10,
            p50: q.p50,
            p90: q.p90,
        }
    }
}

/// One forecaster's forecast for a reading.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Forecast {
    pub p_reading_accepted: f64,
    pub p_verification_passes: f64,
    pub p_user_intervenes: f64,
    pub cost_usd: Quantiles,
    pub wall_time_s: Quantiles,
}

impl Forecast {
    pub fn validate(&self) -> Result<(), String> {
        for (name, p) in [
            ("p_reading_accepted", self.p_reading_accepted),
            ("p_verification_passes", self.p_verification_passes),
            ("p_user_intervenes", self.p_user_intervenes),
        ] {
            if !(0.0..=1.0).contains(&p) {
                return Err(format!("{name} must lie in [0, 1], got {p}"));
            }
        }
        Ok(())
    }
}

/// The two forecasters' forecasts (D15).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Forecasts {
    pub reader: Forecast,
    pub reference: Forecast,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultApplied {
    pub id: String,
    pub how: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Criterion {
    pub text: String,
    pub kind: CriterionKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Deliverable {
    pub path: String,
    pub action: DeliverableAction,
    pub p: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Build {
    pub version: String,
    pub source_commit: String,
    pub dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriverInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelUsage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tokens {
    pub input: Quantity,
    pub output: Quantity,
    pub cache_read: Quantity,
    pub cache_write: Quantity,
}

// ---------------------------------------------------------------------------------------------------------
// The records

/// `athena.request/1`: the user's request, verbatim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub job: JobId,
    pub text: String,
    pub received_at: Timestamp,
    pub channel: String,
}

/// `athena.reading/1`: how the reader read the request, and the forecasts made before dispatch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reading {
    pub job: JobId,
    pub n: u32,
    pub at: Timestamp,
    pub invocation_seq: u32,
    pub base_commit: String,
    pub config_sha256: String,
    pub defaults_sha256: String,
    pub defaults_version: u32,
    pub title: String,
    pub summary: String,
    pub class_of_work: String,
    pub capability: Option<String>,
    pub decline_reason: Option<String>,
    pub producer_model: String,
    pub verifier_model: String,
    pub defaults_applied: Vec<DefaultApplied>,
    pub assumptions: Vec<String>,
    pub question: Option<String>,
    pub acceptance_criteria: Vec<Criterion>,
    pub deliverables: Vec<Deliverable>,
    pub out_of_scope: Vec<String>,
    pub forecasts: Forecasts,
    pub departures: Option<String>,
    /// Why the reader marked the request urgent (it makes Athena faster or cuts wasted time); None when it
    /// did not. An older reading has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub urgent_reason: Option<String>,
    /// Why the reader judged the job hard (it adds or changes a rule that can refuse or hold work, or changes
    /// landing, the history check, records, the run pipeline or the drain); None when it did not. A hard job
    /// gets the producer setting `[models] hard_producer_*` names, and is planned when `[job_plan] mode` is
    /// `hard`. An older reading has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hard_reason: Option<String>,
    /// The producer effort this reading dispatches with, set when a hard job's escalation changed the
    /// producer setting. None means `[models] producer_effort`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producer_effort: Option<String>,
}

impl Reading {
    pub fn validate(&self) -> Result<(), String> {
        self.forecasts.reader.validate()?;
        self.forecasts.reference.validate()?;
        for d in &self.deliverables {
            if !(0.0..=1.0).contains(&d.p) {
                return Err(format!(
                    "deliverable {} has p = {} outside [0, 1]",
                    d.path, d.p
                ));
            }
        }
        if self.capability.is_none() && self.decline_reason.is_none() {
            return Err("a reading without a capability must give a decline reason".into());
        }
        Ok(())
    }
}

/// `athena.correction/1`: a one-line correction of a reading, or an answer to its question.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Correction {
    pub job: JobId,
    /// The reading this line corrects.
    pub n: u32,
    pub at: Timestamp,
    pub kind: CorrectionKind,
    pub text: String,
    /// The intervention recorded for it automatically.
    pub intervention: RecordId,
}

/// The levels a priority record may name, best first.
pub const PRIORITY_LEVELS: [&str; 4] = ["urgent", "high", "normal", "low"];

/// `athena.priority/1`: the user's choice of how soon a queued request is worked, at `<job>/priority-<n>.json`.
/// The request record is never rewritten; the latest priority record (highest `n`) wins over any
/// `Priority:` line in the request's words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriorityRecord {
    pub job: JobId,
    /// 1 for the first priority record of the job, then counting up.
    pub n: u32,
    pub at: Timestamp,
    /// One of [`PRIORITY_LEVELS`].
    pub priority: String,
}

impl PriorityRecord {
    pub fn validate(&self) -> Result<(), String> {
        if self.n == 0 {
            return Err("a priority record is numbered from 1".into());
        }
        if !PRIORITY_LEVELS.contains(&self.priority.as_str()) {
            return Err(format!(
                "priority {:?} is not one of {}",
                self.priority,
                PRIORITY_LEVELS.join(", ")
            ));
        }
        Ok(())
    }
}

/// Whether a job is on hold, as one [`HoldRecord`] states it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HoldState {
    /// Drain does not dispatch the job, whatever its priority.
    OnHold,
    /// The hold is lifted; the job is worked in its priority order again.
    Released,
}

/// `athena.hold/1`: the user's choice to keep a queued request out of `athena drain`, at
/// `<job>/hold-<n>.json`. The latest record (highest `n`) states whether the job is on hold now; a
/// release is a later record, never an edit. The request record is never rewritten.
///
/// This is unrelated to the disk-space hold shown on a status line and to the dependency hold shown as
/// a `held` line in `athena plan`, and it is independent of the job's priority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HoldRecord {
    pub job: JobId,
    /// 1 for the first hold record of the job, then counting up.
    pub n: u32,
    pub at: Timestamp,
    pub state: HoldState,
}

impl HoldRecord {
    pub fn validate(&self) -> Result<(), String> {
        if self.n == 0 {
            return Err("a hold record is numbered from 1".into());
        }
        Ok(())
    }
}

/// `athena.focus/1`: what the user wants worked on now, at `records/focus/focus-<n>.json`.
/// Records are never rewritten; the latest one (highest `n`) is the current focus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FocusRecord {
    /// 1 for the first focus record, then counting up.
    pub n: u32,
    pub at: Timestamp,
    /// The focus, in the user's words.
    pub text: String,
    /// Who set it (the git user name).
    pub set_by: String,
}

impl FocusRecord {
    pub fn validate(&self) -> Result<(), String> {
        if self.n == 0 {
            return Err("a focus record is numbered from 1".into());
        }
        if self.text.trim().is_empty() {
            return Err("a focus record has text".into());
        }
        if self.set_by.trim().is_empty() {
            return Err("a focus record says who set it".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Owner {
    pub role: String,
    pub capability: Option<String>,
    pub model: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Caps {
    pub job_usd: f64,
    pub reader_usd: f64,
    pub producer_usd: f64,
    pub verifier_usd: f64,
    pub producer_timeout_s: u64,
    pub verifier_timeout_s: u64,
    pub check_timeout_s: u64,
}

/// `athena.job/1`: what was dispatched, by whom, under which configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Job {
    pub id: JobId,
    pub kind: JobKind,
    pub title: String,
    pub class_of_work: Option<String>,
    pub capability: Option<String>,
    pub accepted_reading: Option<u32>,
    pub accepted_by: Option<AcceptedBy>,
    pub owner: Owner,
    pub verifier_model: Option<String>,
    pub caps: Option<Caps>,
    pub base_commit: Option<String>,
    pub config_sha256: Option<String>,
    pub defaults_sha256: Option<String>,
    pub athena: Option<Build>,
    pub driver: Option<DriverInfo>,
    pub brief_sha256: Option<String>,
    pub dispatched_at: Option<Timestamp>,
    pub request_path: Option<String>,
    pub request_sha256: Option<String>,
    pub cost: Option<Quantity>,
    pub tokens: Option<Quantity>,
    pub note: Option<String>,
    /// The repository a study job reads, pinned by the core before dispatch. Absent for other work.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin: Option<PinnedRepo>,
    /// The branch that was checked out when the job was dispatched; the job lands only there. The empty
    /// string means HEAD was detached at dispatch. Absent in a record made before this field existed: such
    /// a job is not held to a branch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dispatch_branch: Option<String>,
    /// The files this job may change, when an accepted criterion limits which files may change (for
    /// example "no file outside X changed"). The list is the accepted reading's deliverable paths, so a
    /// reading that limits scope needs a complete deliverable list. An entry ending in `/` covers every
    /// path under that directory. Athena compares the candidate's changed files with it before the
    /// producer's report is accepted and again before verification. Absent when no criterion limits the
    /// files, and in a record made before this field existed: such a job is not checked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allowed_paths: Option<Vec<String>>,
}

/// A repository ref resolved by the core to one commit: the URL and ref the configuration named, and the
/// full 40-hex commit id they resolved to when the job was dispatched.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinnedRepo {
    pub url: String,
    #[serde(rename = "ref")]
    pub git_ref: String,
    pub sha: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextComponent {
    pub id: String,
    pub version: u32,
    pub sha256: String,
    pub tokens: Quantity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationContext {
    pub components: Vec<ContextComponent>,
    pub prompt_sha256: String,
    pub prompt_tokens: Quantity,
    pub host_baseline: Quantity,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationCaps {
    pub max_budget_usd: f64,
    pub timeout_s: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Denial {
    pub tool: String,
    pub detail: String,
}

/// The rule name recorded for a refusal that no reading rule is named for: output that did not parse or
/// validate as a reading.
pub const OTHER_READING_RULE: &str = "other";

/// One refusal of a reading: the reading rule (the name of its check function) and the text it gave.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadingRefusal {
    pub rule: String,
    pub text: String,
}

/// What became of the output of one reader invocation once it reached the reading rules.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadingAttempt {
    /// 1 for the first reading of a run, 2 for the reading made again with a refusal as a correction.
    pub attempt: u32,
    /// The refusal; absent when the reading was accepted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<ReadingRefusal>,
}

/// `athena.invocation/1`: one model invocation, with its usage, cost, time, and outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invocation {
    pub job: JobId,
    pub seq: u32,
    pub role: Role,
    pub driver: DriverInfo,
    pub athena: Build,
    pub model: String,
    /// The effort level the agent ran at (`--effort`). An older record has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    pub context: InvocationContext,
    pub caps: InvocationCaps,
    pub started_at: Timestamp,
    pub ended_at: Timestamp,
    pub wall_time: Quantity,
    pub tokens: Tokens,
    pub models_used: BTreeMap<String, ModelUsage>,
    pub cost: Quantity,
    pub outcome: InvocationOutcome,
    pub detail: Option<String>,
    pub turns: Option<u32>,
    pub commands_run: Vec<String>,
    pub policy_denials: Vec<Denial>,
    /// For a reader invocation whose output reached the reading rules: which reading it was and the rule that
    /// refused it, if one did. Absent for other roles, for a reader that did not finish, and in older records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reading_attempt: Option<ReadingAttempt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeEntry {
    pub path: String,
    pub status: ChangeStatus,
    pub mode: Option<String>,
    pub bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyViolation {
    pub rule: String,
    pub path: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportedCheck {
    pub command: String,
    pub result: String,
    pub note: Option<String>,
}

/// The producer's own check of its work against one accepted criterion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportedCriterion {
    pub criterion: String,
    pub status: String,
    pub evidence: String,
}

/// The producer's structured report: kept with its run, redacted, never shown to the verifier (D32).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerReport {
    pub summary: String,
    pub files: Vec<String>,
    pub checks_run: Vec<ReportedCheck>,
    /// Absent in records made before the pre-verification criteria check.
    #[serde(default)]
    pub criteria_check: Vec<ReportedCriterion>,
    pub open_issues: Vec<String>,
    pub mistakes: Vec<String>,
    pub learnings: Vec<String>,
    pub desires: Vec<String>,
    /// Why the producer left the tree unchanged. Absent in records made before the outcome was asked for,
    /// and when the producer changed files; it is not written when unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_change: Option<NoChange>,
}

/// The producer's own account of an empty change: `kind` is `already_done`, `blocked` or `gave_up`, and
/// `detail` is the evidence, the blocker or the reason, redacted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoChange {
    pub kind: String,
    pub detail: String,
}

/// `athena.candidate/1` and `athena.candidate/2`: the producer's change, as the core captured and admitted
/// it. Both schemas share this type; the envelope's tag says which one a record is.
///
/// `/2` adds `base_fingerprint` and `previous_job`, which link a job to the one before it in a chain of
/// jobs. A `/1` record has neither field ([`AnyRecord::parse`] refuses a `/1` that has one, and a `/2` that
/// lacks `base_fingerprint`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub job: JobId,
    pub base_commit: String,
    /// `/2` only: the fingerprint of the base commit's tree, under the base commit's own rules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_fingerprint: Option<String>,
    /// `/2` only: the latest job landed in the base commit's history; absent if there is none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_job: Option<JobId>,
    pub candidate_commit: Option<String>,
    pub candidate_tree: Option<String>,
    pub changes: Vec<ChangeEntry>,
    pub fingerprint: Option<String>,
    pub violations: Vec<PolicyViolation>,
    pub producer_report: Option<ProducerReport>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckRun {
    pub name: String,
    pub command: Vec<String>,
    pub exit_code: Option<i32>,
    pub passed: bool,
    pub tests_ran: Option<u64>,
    pub duration: Quantity,
    pub output_tail: String,
    pub note: Option<String>,
    /// The tests that failed in this run, so the baseline and the candidate can be compared test by test.
    /// Before this field a baseline kept one pass/fail bit per check name, so a check that failed on the
    /// base excused every failure in it, including a new, different failing test.
    ///
    /// Each name is `<binary> <test>` (`doc-tests <crate> <test>` for doc tests), or `<binary> (did not
    /// finish)` for a test binary that crashed or whose failure list was cut short. Empty means no names are
    /// known (the check is not a plain `cargo test`, it timed out, or the record is older than this field),
    /// and the comparison falls back to the check name. For a plain `cargo test`, Athena adds
    /// `--no-fail-fast` so every binary runs; `command` keeps the configured text without it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failed_tests: Vec<String>,
    /// True when this record was not run locally but taken from a green CI run for the candidate commit
    /// (it then has no exit code). The verdict counts such a passing record as a command seen running
    /// (D43, D115). Absent in records written before this field, which load as `false`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reused: bool,
    /// The criterion numbers (1-based) whose planned "fails without the fix" demonstration this record is.
    /// Empty for every ordinary check; absent in records written before this field.
    ///
    /// For such a record Athena ran the criterion's test on a copy of the tree with the fix reverted, so a
    /// nonzero exit code is the expected result and `passed` is always `true`, meaning "not a pass/fail
    /// check" (it never blocks a verdict). The verdict counts the record's command, and its name, as seen
    /// running only for the criteria listed here (D43).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revert_for: Vec<u32>,
    /// Seconds this check waited for a build slot before it started (basis: athena monotonic clock), from the
    /// build-slot queue. Absent for a check that took no slot (a reused CI result, a skipped or nested
    /// check) and in records written before this field; `athena status` sums it into its "build wait" line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queued_s: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CriterionResult {
    pub criterion: String,
    pub kind: CriterionKind,
    pub met: bool,
    pub evidence: String,
    pub commands: Vec<String>,
    /// For behavior criteria: whether every cited command was seen running (D43).
    pub commands_verified: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub severity: Severity,
    pub summary: String,
    pub evidence: String,
    pub location: Option<String>,
    pub source: FindingSource,
}

/// `athena.verification/1`: the checks, the independent verdict, and the final verdict.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Verification {
    pub job: JobId,
    pub candidate_commit: String,
    pub verified_fingerprint: String,
    pub tool_config_from_base: Vec<String>,
    pub checks: Vec<CheckRun>,
    pub check_cleanup: Vec<String>,
    pub verifier_invocation: Option<u32>,
    pub criteria: Vec<CriterionResult>,
    pub findings: Vec<Finding>,
    pub request_reading_mismatch: Option<String>,
    pub agent_verdict: Option<Verdict>,
    pub final_verdict: Verdict,
    pub reasons: Vec<String>,
    pub inputs: Vec<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Totals {
    /// Every invocation of the job, readings included.
    pub cost_all: Quantity,
    /// The producer's and verifier's invocations: what `cost_usd` forecasts.
    pub cost_execution: Quantity,
    /// From dispatch to the final verdict: what `wall_time_s` forecasts.
    pub wall_time_execution: Option<Quantity>,
    pub tokens: BTreeMap<String, ModelUsage>,
    pub invocations: u32,
}

/// `athena.outcome/1`: where the job ended, and its totals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub job: JobId,
    pub state: OutcomeState,
    pub reason: String,
    pub at: Timestamp,
    pub totals: Option<Totals>,
    pub branch: Option<String>,
    pub base_commit: Option<String>,
    pub candidate_commit: Option<String>,
    /// True for the outcome that follows `blocked`: `outcome-landed.json` when the retried landing
    /// succeeds, `outcome-abandoned.json` when the job is closed instead.
    pub after_blocked: bool,
    /// For the bootstrap: the last commit of its range (D39).
    pub through_commit: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventScore {
    pub event: String,
    pub reading: u32,
    pub forecasts: BTreeMap<String, f64>,
    pub outcome: Option<bool>,
    pub brier: BTreeMap<String, f64>,
    pub provisional: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeliverableTerm {
    pub path: String,
    pub p: f64,
    pub listed: bool,
    pub changed: bool,
    pub brier: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeliverableScore {
    pub terms: Vec<DeliverableTerm>,
    pub sum: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantileScore {
    pub p10: f64,
    pub p50: f64,
    pub p90: f64,
    pub wis: f64,
    pub inside_80: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantityScore {
    pub quantity: String,
    pub actual: f64,
    pub floor: f64,
    /// True when `actual` is only a bound (a cost known as a lower bound, not a firm figure). Such a score
    /// is not final and does not count in aggregates or in the reference's history. Records written before
    /// this field existed read as not provisional.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub provisional: bool,
    pub forecasts: BTreeMap<String, QuantileScore>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelsCheck {
    pub expected: Vec<String>,
    pub observed: Vec<String>,
    pub unexpected: Vec<String>,
    pub as_expected: bool,
}

/// `athena.score/1`: every forecast scored against what happened, components kept separate (D14).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Score {
    pub job: JobId,
    pub at: Timestamp,
    pub rules: String,
    /// The intervention that caused this re-score; `None` for the closing score.
    pub cause: Option<RecordId>,
    pub events: Vec<EventScore>,
    pub deliverables: Option<DeliverableScore>,
    pub quantities: Vec<QuantityScore>,
    pub models: Option<ModelsCheck>,
}

/// `athena.intervention/1`: something the user did beyond what a lazy user would (O3).
///
/// A record has no field that says it was followed up. `athena status` lists a `user` intervention as open
/// work until a later request job's request text names its id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Intervention {
    pub id: RecordId,
    pub at: Timestamp,
    pub what: String,
    pub why: String,
    pub class_of_work: String,
    pub category: InterventionCategory,
    pub job: Option<JobId>,
    pub source: InterventionSource,
    /// True when the user's staged hand changes were committed together with this record (D39).
    pub with_staged_changes: bool,
}

/// `athena.restart/1`: a reset of the user's own context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Restart {
    pub id: RecordId,
    pub at: Timestamp,
    pub cost: String,
    pub minutes: Option<Quantity>,
    pub tokens: Option<Quantity>,
    pub lost: Vec<String>,
    pub via: Vec<String>,
}

/// `athena.resolution/1`: supplies a pending quantity in another record, which stays immutable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resolution {
    pub id: RecordId,
    pub at: Timestamp,
    pub target_path: String,
    pub target_field: String,
    pub value: Quantity,
    pub note: Option<String>,
}

mod core;
mod design_surface;
mod introspection;
mod plan;
mod throughput;
mod work;

pub use self::core::*;
pub use self::design_surface::*;
pub use self::introspection::*;
pub use self::plan::*;
pub use self::throughput::*;
pub use self::work::*;

// ---------------------------------------------------------------------------------------------------------
// The envelope

/// Any record, tagged by its `schema` field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "schema")]
pub enum AnyRecord {
    #[serde(rename = "athena.request/1")]
    Request(Request),
    #[serde(rename = "athena.reading/1")]
    Reading(Box<Reading>),
    #[serde(rename = "athena.correction/1")]
    Correction(Correction),
    #[serde(rename = "athena.priority/1")]
    Priority(PriorityRecord),
    #[serde(rename = "athena.hold/1")]
    Hold(HoldRecord),
    #[serde(rename = "athena.focus/1")]
    Focus(FocusRecord),
    #[serde(rename = "athena.job/1")]
    Job(Box<Job>),
    #[serde(rename = "athena.invocation/1")]
    Invocation(Box<Invocation>),
    #[serde(rename = "athena.candidate/1")]
    Candidate(Box<Candidate>),
    #[serde(rename = "athena.candidate/2")]
    Candidate2(Box<Candidate>),
    #[serde(rename = "athena.verification/1")]
    Verification(Box<Verification>),
    #[serde(rename = "athena.outcome/1")]
    Outcome(Box<Outcome>),
    #[serde(rename = "athena.score/1")]
    Score(Box<Score>),
    #[serde(rename = "athena.intervention/1")]
    Intervention(Intervention),
    #[serde(rename = "athena.restart/1")]
    Restart(Restart),
    #[serde(rename = "athena.resolution/1")]
    Resolution(Resolution),
    #[serde(rename = "athena.host-error/1")]
    HostError(HostError),
    #[serde(rename = "athena.decision-state/1")]
    DecisionState(DecisionState),
    #[serde(rename = "athena.default-expectations/1")]
    DefaultExpectations(DefaultExpectations),
    #[serde(rename = "athena.introspection/1")]
    Introspection(Box<IntrospectionRecord>),
    #[serde(rename = "athena.failure-class/1")]
    FailureClass(FailureClass),
    #[serde(rename = "athena.defaults-proposal/1")]
    DefaultsProposal(Box<ProposalRecord>),
    #[serde(rename = "athena.class-label/1")]
    ClassLabel(ClassLabel),
    #[serde(rename = "athena.role/1")]
    Role(Box<crate::roles::RoleDefinition>),
    #[serde(rename = "athena.work-item/1")]
    WorkItem(Box<WorkItem>),
    #[serde(rename = "athena.work-decision/1")]
    WorkDecision(WorkDecision),
    #[serde(rename = "athena.audience-rating/1")]
    AudienceRating(Box<AudienceRating>),
    #[serde(rename = "athena.audit/1")]
    Audit(Box<Audit>),
    #[serde(rename = "athena.hestia-sweep/1")]
    HestiaSweep(Box<HestiaSweep>),
    #[serde(rename = "athena.job-plan/1")]
    JobPlan(Box<JobPlan>),
    #[serde(rename = "athena.critique/1")]
    Critique(Box<Critique>),
    #[serde(rename = "athena.baseline/1")]
    Baseline(Box<Baseline>),
    #[serde(rename = "athena.replay/1")]
    Replay(Box<Replay>),
    #[serde(rename = "athena.code-health-baseline/1")]
    CodeHealthBaseline(CodeHealthBaseline),
    #[serde(rename = "athena.throughput-review/1")]
    ThroughputReview(Box<ThroughputReview>),
    #[serde(rename = "athena.build-measurement/1")]
    BuildMeasurement(Box<BuildMeasurement>),
    #[serde(rename = "athena.design-surface/1")]
    DesignSurface(Box<DesignSurface>),
}

/// Why a file under `records/` is not acceptable.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RecordError {
    #[error("{path}: does not parse as a record: {reason}")]
    Parse { path: String, reason: String },
    #[error("{path}: invalid record: {reason}")]
    Invalid { path: String, reason: String },
    #[error("{path}: misplaced; a record with this content belongs at {expected}")]
    Misplaced { path: String, expected: String },
    #[error("{path}: only JSON records and the documented Markdown briefs may live under records/")]
    UnknownFile { path: String },
}

impl AnyRecord {
    /// Parses a record and checks its internal consistency.
    pub fn parse(path: &str, bytes: &[u8]) -> Result<AnyRecord, RecordError> {
        let record: AnyRecord = serde_json::from_slice(bytes).map_err(|e| RecordError::Parse {
            path: path.to_string(),
            reason: e.to_string(),
        })?;
        let invalid = |reason: String| RecordError::Invalid {
            path: path.to_string(),
            reason,
        };
        match &record {
            AnyRecord::Reading(r) => r.validate().map_err(invalid)?,
            AnyRecord::Priority(p) => p.validate().map_err(invalid)?,
            AnyRecord::Hold(h) => h.validate().map_err(invalid)?,
            AnyRecord::Focus(f) => f.validate().map_err(invalid)?,
            AnyRecord::Candidate(c) if c.base_fingerprint.is_some() || c.previous_job.is_some() => {
                return Err(invalid(
                    "athena.candidate/1 has no base_fingerprint or previous_job; use athena.candidate/2"
                        .into(),
                ));
            }
            AnyRecord::Candidate2(c) if c.base_fingerprint.is_none() => {
                return Err(invalid(
                    "athena.candidate/2 records the base_fingerprint".into(),
                ));
            }
            AnyRecord::Intervention(i) if i.what.trim().is_empty() || i.why.trim().is_empty() => {
                return Err(invalid("an intervention needs what and why".into()));
            }
            AnyRecord::HostError(h) if h.status_code.is_some_and(|s| !(100..=599).contains(&s)) => {
                return Err(invalid(
                    "a host error's status_code is an HTTP status".into(),
                ));
            }
            AnyRecord::HostError(h)
                if h.message_class != MessageClass::of_status(h.status_code) =>
            {
                return Err(invalid(
                    "a host error's message_class follows from its status_code".into(),
                ));
            }
            AnyRecord::HostError(h) if h.derived != h.source_intervention.is_some() => {
                return Err(invalid(
                    "a derived host error cites its source intervention, and an observed one has none"
                        .into(),
                ));
            }
            AnyRecord::DecisionState(d) if d.decision.trim().is_empty() => {
                return Err(invalid("a decision-state names its decision".into()));
            }
            AnyRecord::DecisionState(d)
                if d.state == DecisionStateKind::UnderReview && d.firing_records.is_empty() =>
            {
                return Err(invalid(
                    "an under-review decision-state cites the records that fired it".into(),
                ));
            }
            AnyRecord::DefaultExpectations(e) => e.validate().map_err(invalid)?,
            AnyRecord::Introspection(i) => i.validate().map_err(invalid)?,
            AnyRecord::FailureClass(c) if c.name.trim().is_empty() => {
                return Err(invalid("a failure class has a name".into()));
            }
            AnyRecord::ClassLabel(l) => l.validate().map_err(invalid)?,
            AnyRecord::Role(r) => r.validate().map_err(invalid)?,
            AnyRecord::WorkItem(w) => w.validate().map_err(invalid)?,
            AnyRecord::WorkDecision(d) => d.validate().map_err(invalid)?,
            AnyRecord::AudienceRating(a) => a.validate().map_err(invalid)?,
            AnyRecord::Audit(a) => a.validate().map_err(invalid)?,
            AnyRecord::HestiaSweep(s) => s.validate().map_err(invalid)?,
            AnyRecord::JobPlan(p) => p.validate().map_err(invalid)?,
            AnyRecord::Critique(c) => c.validate().map_err(invalid)?,
            AnyRecord::Baseline(b) => b.validate().map_err(invalid)?,
            AnyRecord::Replay(r) => r.validate().map_err(invalid)?,
            AnyRecord::ThroughputReview(r) => r.validate().map_err(invalid)?,
            AnyRecord::BuildMeasurement(r) => r.validate().map_err(invalid)?,
            AnyRecord::DesignSurface(r) => r.validate().map_err(invalid)?,
            AnyRecord::DefaultsProposal(p) => p.validate().map_err(invalid)?, // Totals are what reconciliation checks an outcome against; only the bootstrap has none
            // (acceptance verification 2).
            AnyRecord::Outcome(o) if o.job.is_request() && o.totals.is_none() => {
                return Err(invalid(
                    "an outcome of a request job records its totals".into(),
                ));
            }
            _ => {}
        }
        Ok(record)
    }

    /// Serializes the record the way it is committed.
    pub fn to_json(&self) -> String {
        crate::to_pretty_json(self).expect("records always serialize")
    }

    /// The one path, relative to the repository root, where this record belongs.
    pub fn canonical_path(&self) -> String {
        match self {
            AnyRecord::Request(r) => format!("{}/request.json", r.job.dir()),
            AnyRecord::Reading(r) => format!("{}/reading-{}.json", r.job.dir(), r.n),
            AnyRecord::Correction(r) => format!("{}/correction-{}.json", r.job.dir(), r.n),
            AnyRecord::Priority(r) => format!("{}/priority-{}.json", r.job.dir(), r.n),
            AnyRecord::Hold(r) => format!("{}/hold-{}.json", r.job.dir(), r.n),
            AnyRecord::Focus(r) => format!("records/focus/focus-{}.json", r.n),
            AnyRecord::Job(r) => format!("{}/job.json", r.id.dir()),
            AnyRecord::Invocation(r) => {
                format!(
                    "{}/invocations/{:02}-{}.json",
                    r.job.dir(),
                    r.seq,
                    r.role.as_str()
                )
            }
            AnyRecord::Candidate(r) | AnyRecord::Candidate2(r) => {
                format!("{}/candidate.json", r.job.dir())
            }
            AnyRecord::Verification(r) => format!("{}/verification.json", r.job.dir()),
            AnyRecord::Outcome(r) if r.after_blocked => {
                format!("{}/outcome-{}.json", r.job.dir(), r.state.as_str())
            }
            AnyRecord::Outcome(r) => format!("{}/outcome.json", r.job.dir()),
            AnyRecord::Score(r) => match &r.cause {
                None => format!("{}/score.json", r.job.dir()),
                Some(cause) => format!("{}/score-{}.json", r.job.dir(), cause),
            },
            AnyRecord::Intervention(r) => {
                format!("records/interventions/{}/{}.json", r.id.month(), r.id)
            }
            AnyRecord::Restart(r) => format!("records/restarts/{}/{}.json", r.id.month(), r.id),
            AnyRecord::Resolution(r) => {
                format!("records/resolutions/{}/{}.json", r.id.month(), r.id)
            }
            AnyRecord::HostError(r) => {
                format!("records/host-errors/{}/{}.json", r.id.month(), r.id)
            }
            AnyRecord::DecisionState(r) => {
                format!("records/decision-states/{}/{}.json", r.id.month(), r.id)
            }
            AnyRecord::DefaultExpectations(r) => {
                format!(
                    "records/default-expectations/{}/{}.json",
                    r.id.month(),
                    r.id
                )
            }
            AnyRecord::Introspection(r) => {
                format!("records/introspections/{}/{}.json", r.id.month(), r.id)
            }
            AnyRecord::FailureClass(r) => {
                format!("records/failure-classes/{}/{}.json", r.id.month(), r.id)
            }
            AnyRecord::DefaultsProposal(r) => {
                format!(
                    "records/defaults-proposals/{}/{}.json",
                    r.record.month(),
                    r.record
                )
            }
            AnyRecord::ClassLabel(r) => {
                format!("records/class-labels/{}/{}.json", r.id.month(), r.id)
            }
            AnyRecord::Role(r) => r.path(),
            AnyRecord::WorkItem(r) => {
                format!("records/work-items/{}/{}.json", r.id.month(), r.id)
            }
            AnyRecord::WorkDecision(r) => {
                format!("records/work-decisions/{}/{}.json", r.id.month(), r.id)
            }
            AnyRecord::AudienceRating(r) => {
                format!("records/audience-ratings/{}/{}.json", r.id.month(), r.id)
            }
            AnyRecord::Audit(r) => format!("records/audits/{}/{}.json", r.id.month(), r.id),
            AnyRecord::HestiaSweep(r) => {
                format!("records/hestia-sweeps/{}/{}.json", r.id.month(), r.id)
            }
            AnyRecord::JobPlan(r) => format!("{}/job-plan-{}.json", r.job.dir(), r.round),
            AnyRecord::Critique(r) => format!("{}/critique-{}.json", r.job.dir(), r.round),
            AnyRecord::Baseline(r) => format!("{}/baseline.json", r.job.dir()),
            AnyRecord::Replay(r) => format!("{}/replay.json", r.job.dir()),
            AnyRecord::CodeHealthBaseline(r) => {
                format!("records/code-health/{}/{}.json", r.id.month(), r.id)
            }
            AnyRecord::ThroughputReview(r) => {
                format!("records/throughput-reviews/{}/{}.json", r.id.month(), r.id)
            }
            AnyRecord::BuildMeasurement(r) => {
                format!("records/build-measurements/{}/{}.json", r.id.month(), r.id)
            }
            AnyRecord::DesignSurface(r) => {
                format!("records/design-surface/{}/{}.json", r.id.month(), r.id)
            }
        }
    }

    pub fn schema(&self) -> &'static str {
        match self {
            AnyRecord::Request(_) => "athena.request/1",
            AnyRecord::Reading(_) => "athena.reading/1",
            AnyRecord::Correction(_) => "athena.correction/1",
            AnyRecord::Priority(_) => "athena.priority/1",
            AnyRecord::Hold(_) => "athena.hold/1",
            AnyRecord::Focus(_) => "athena.focus/1",
            AnyRecord::Job(_) => "athena.job/1",
            AnyRecord::Invocation(_) => "athena.invocation/1",
            AnyRecord::Candidate(_) => "athena.candidate/1",
            AnyRecord::Candidate2(_) => "athena.candidate/2",
            AnyRecord::Verification(_) => "athena.verification/1",
            AnyRecord::Outcome(_) => "athena.outcome/1",
            AnyRecord::Score(_) => "athena.score/1",
            AnyRecord::Intervention(_) => "athena.intervention/1",
            AnyRecord::Restart(_) => "athena.restart/1",
            AnyRecord::Resolution(_) => "athena.resolution/1",
            AnyRecord::HostError(_) => "athena.host-error/1",
            AnyRecord::DecisionState(_) => "athena.decision-state/1",
            AnyRecord::DefaultExpectations(_) => "athena.default-expectations/1",
            AnyRecord::Introspection(_) => INTROSPECTION_SCHEMA,
            AnyRecord::FailureClass(_) => "athena.failure-class/1",
            AnyRecord::DefaultsProposal(_) => PROPOSAL_SCHEMA,
            AnyRecord::ClassLabel(_) => "athena.class-label/1",
            AnyRecord::Role(_) => crate::roles::ROLE_SCHEMA,
            AnyRecord::WorkItem(_) => "athena.work-item/1",
            AnyRecord::WorkDecision(_) => "athena.work-decision/1",
            AnyRecord::AudienceRating(_) => AUDIENCE_RATING_SCHEMA,
            AnyRecord::Audit(_) => AUDIT_SCHEMA,
            AnyRecord::HestiaSweep(_) => HESTIA_SWEEP_SCHEMA,
            AnyRecord::JobPlan(_) => JOB_PLAN_SCHEMA,
            AnyRecord::Critique(_) => CRITIQUE_SCHEMA,
            AnyRecord::Baseline(_) => BASELINE_SCHEMA,
            AnyRecord::Replay(_) => REPLAY_SCHEMA,
            AnyRecord::CodeHealthBaseline(_) => CODE_HEALTH_BASELINE_SCHEMA,
            AnyRecord::ThroughputReview(_) => THROUGHPUT_REVIEW_SCHEMA,
            AnyRecord::BuildMeasurement(_) => BUILD_MEASUREMENT_SCHEMA,
            AnyRecord::DesignSurface(_) => DESIGN_SURFACE_SCHEMA,
        }
    }
}

/// Markdown files allowed under a job directory, beside the JSON records.
const JOB_MARKDOWN: [&str; 3] = ["brief-producer.md", "brief-verifier.md", "progress-note.md"];

/// Checks a file added under `records/`: a record at its canonical location, or an allowed Markdown file.
/// Returns the parsed record for JSON files.
pub fn check_record_file(path: &str, bytes: &[u8]) -> Result<Option<AnyRecord>, RecordError> {
    if let Some(name) = path
        .strip_suffix(".md")
        .map(|_| path.rsplit('/').next().unwrap_or(path))
    {
        let dir = path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
        let in_job_dir = is_job_dir(dir);
        let bootstrap_brief = dir == "records/jobs/bootstrap"
            && (name == "brief.md"
                || name
                    .strip_prefix("brief-update-")
                    .and_then(|rest| rest.strip_suffix(".md"))
                    .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())));
        if (in_job_dir && JOB_MARKDOWN.contains(&name)) || bootstrap_brief {
            return Ok(None);
        }
        return Err(RecordError::UnknownFile {
            path: path.to_string(),
        });
    }
    if !path.ends_with(".json") {
        return Err(RecordError::UnknownFile {
            path: path.to_string(),
        });
    }
    let record = AnyRecord::parse(path, bytes)?;
    let expected = record.canonical_path();
    if expected != path {
        return Err(RecordError::Misplaced {
            path: path.to_string(),
            expected,
        });
    }
    Ok(Some(record))
}

/// True for `records/jobs/bootstrap` and `records/jobs/<YYYY-MM>/<id>`.
fn is_job_dir(dir: &str) -> bool {
    if dir == "records/jobs/bootstrap" {
        return true;
    }
    let Some(rest) = dir.strip_prefix("records/jobs/") else {
        return false;
    };
    let mut parts = rest.split('/');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(month), Some(id), None) => RecordId::parse(id).is_ok_and(|id| id.month() == month),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quantity::Unit;

    fn ts() -> Timestamp {
        "2026-10-10T15:30:12Z".parse().unwrap()
    }

    #[test]
    fn old_killed_and_failed_outcomes_still_deserialize() {
        let killed: InvocationOutcome = serde_json::from_str("\"killed\"").unwrap();
        assert_eq!(killed, InvocationOutcome::Killed);
        let failed: OutcomeState = serde_json::from_str("\"failed\"").unwrap();
        assert_eq!(failed, OutcomeState::Failed);
        let timed_out: OutcomeState = serde_json::from_str("\"timed_out\"").unwrap();
        assert_eq!(timed_out, OutcomeState::TimedOut);
        assert_eq!(timed_out.as_str(), "timed_out");
        assert_eq!(
            serde_json::to_string(&InvocationOutcome::TimedOut).unwrap(),
            "\"timed_out\""
        );
    }

    fn job() -> JobId {
        JobId::parse("20261010-153012-k7q2").unwrap()
    }

    pub(crate) fn sample_reading() -> Reading {
        let q = Quantiles::new(0.2, 0.45, 1.1).unwrap();
        let t = Quantiles::new(120.0, 240.0, 540.0).unwrap();
        let forecast = Forecast {
            p_reading_accepted: 0.8,
            p_verification_passes: 0.8,
            p_user_intervenes: 0.15,
            cost_usd: q,
            wall_time_s: t,
        };
        Reading {
            job: job(),
            n: 1,
            at: ts(),
            invocation_seq: 1,
            base_commit: "0".repeat(40),
            config_sha256: "a".repeat(64),
            defaults_sha256: "b".repeat(64),
            defaults_version: 1,
            title: "Add a changelog".into(),
            summary: "Create CHANGELOG.md.".into(),
            class_of_work: "docs".into(),
            capability: Some("change".into()),
            decline_reason: None,
            producer_model: "claude-sonnet-5-5".into(),
            verifier_model: "claude-opus-5-5".into(),
            defaults_applied: vec![DefaultApplied {
                id: "strangers".into(),
                how: "plain prose".into(),
            }],
            assumptions: vec![],
            question: None,
            acceptance_criteria: vec![Criterion {
                text: "CHANGELOG.md exists".into(),
                kind: CriterionKind::Content,
            }],
            deliverables: vec![Deliverable {
                path: "CHANGELOG.md".into(),
                action: DeliverableAction::Create,
                p: 0.95,
            }],
            out_of_scope: vec![],
            forecasts: Forecasts {
                reader: forecast.clone(),
                reference: forecast,
            },
            departures: None,
            urgent_reason: None,
            hard_reason: None,
            producer_effort: None,
        }
    }

    #[test]
    fn records_without_effort_or_hard_fields_still_load() {
        let record = AnyRecord::Reading(Box::new(sample_reading()));
        let text = record.to_json();
        assert!(!text.contains("hard_reason") && !text.contains("producer_effort"));
        let back = AnyRecord::parse("x", text.as_bytes()).unwrap();
        assert_eq!(back, record);
        let mut hard = sample_reading();
        hard.hard_reason = Some("changes landing".into());
        hard.producer_effort = Some("high".into());
        let text = AnyRecord::Reading(Box::new(hard.clone())).to_json();
        assert!(text.contains("\"hard_reason\": \"changes landing\""));
        assert_eq!(
            AnyRecord::parse("x", text.as_bytes()).unwrap(),
            AnyRecord::Reading(Box::new(hard))
        );
    }

    #[test]
    fn records_round_trip_through_the_envelope() {
        let record = AnyRecord::Reading(Box::new(sample_reading()));
        let text = record.to_json();
        assert!(text.contains("\"schema\": \"athena.reading/1\""));
        let back = AnyRecord::parse("x", text.as_bytes()).unwrap();
        assert_eq!(back, record);
    }

    fn sample_invocation(reading_attempt: Option<ReadingAttempt>) -> Invocation {
        let tokens = |v: f64| Quantity::measured(v, Unit::Tokens, "result.usage");
        Invocation {
            job: job(),
            seq: 1,
            role: Role::Reader,
            driver: DriverInfo {
                name: "claude-code".into(),
                version: "2.1.299".into(),
            },
            athena: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            model: "m".into(),
            effort: None,
            context: InvocationContext {
                components: vec![],
                prompt_sha256: "0".repeat(64),
                prompt_tokens: tokens(1.0),
                host_baseline: tokens(600.0),
                tools: vec![],
            },
            caps: InvocationCaps {
                max_budget_usd: 1.0,
                timeout_s: 600,
            },
            started_at: ts(),
            ended_at: ts(),
            wall_time: Quantity::measured(60.0, Unit::Seconds, "athena clock"),
            tokens: Tokens {
                input: tokens(1.0),
                output: tokens(1.0),
                cache_read: tokens(1.0),
                cache_write: tokens(1.0),
            },
            models_used: BTreeMap::new(),
            cost: Quantity::reported(0.5, Unit::Usd, "Claude Code total_cost_usd"),
            outcome: InvocationOutcome::Success,
            detail: None,
            turns: None,
            commands_run: vec![],
            policy_denials: vec![],
            reading_attempt,
        }
    }

    #[test]
    fn a_reading_attempt_on_an_invocation_round_trips_and_is_checked_at_its_path() {
        let refused = ReadingAttempt {
            attempt: 1,
            refusal: Some(ReadingRefusal {
                rule: "check_code_survey".into(),
                text: "the reading has no code survey".into(),
            }),
        };
        let accepted = ReadingAttempt {
            attempt: 2,
            refusal: None,
        };
        for attempt in [refused, accepted] {
            let record = AnyRecord::Invocation(Box::new(sample_invocation(Some(attempt))));
            let text = record.to_json();
            assert!(text.contains("\"reading_attempt\""));
            assert_eq!(AnyRecord::parse("x", text.as_bytes()).unwrap(), record);
            let path = record.canonical_path();
            assert_eq!(
                check_record_file(&path, text.as_bytes()).unwrap(),
                Some(record)
            );
            assert!(matches!(
                check_record_file("records/jobs/x/invocations/09-reader.json", text.as_bytes()),
                Err(RecordError::Misplaced { .. })
            ));
        }
    }

    #[test]
    fn an_invocation_without_a_reading_attempt_has_no_key_and_an_old_one_parses() {
        let record = AnyRecord::Invocation(Box::new(sample_invocation(None)));
        let text = record.to_json();
        assert!(!text.contains("reading_attempt"));
        assert_eq!(AnyRecord::parse("x", text.as_bytes()).unwrap(), record);
        let mut value: serde_json::Value = serde_json::from_str(&text).unwrap();
        value["reading_attempt"] = serde_json::json!({"attempt": 1, "extra": true});
        let bad = serde_json::to_string(&value).unwrap();
        assert!(AnyRecord::parse("x", bad.as_bytes()).is_err());
        value["reading_attempt"] = serde_json::json!({"attempt": 1});
        let bare = serde_json::to_string(&value).unwrap();
        let AnyRecord::Invocation(parsed) = AnyRecord::parse("x", bare.as_bytes()).unwrap() else {
            panic!("not an invocation");
        };
        assert_eq!(
            parsed.reading_attempt,
            Some(ReadingAttempt {
                attempt: 1,
                refusal: None
            })
        );
    }

    fn hold(n: u32, state: HoldState) -> HoldRecord {
        HoldRecord {
            job: job(),
            n,
            at: ts(),
            state,
        }
    }

    #[test]
    fn a_hold_record_has_a_path_a_schema_and_round_trips() {
        for (state, word) in [
            (HoldState::OnHold, "on_hold"),
            (HoldState::Released, "released"),
        ] {
            let record = AnyRecord::Hold(hold(1, state));
            assert_eq!(record.schema(), "athena.hold/1");
            let path = format!("{}/hold-1.json", job().dir());
            assert_eq!(record.canonical_path(), path);
            let text = record.to_json();
            assert!(text.contains("\"schema\": \"athena.hold/1\""));
            assert!(text.contains(&format!("\"state\": \"{word}\"")));
            assert_eq!(AnyRecord::parse("x", text.as_bytes()).unwrap(), record);
            assert_eq!(
                check_record_file(&path, text.as_bytes()).unwrap(),
                Some(record)
            );
            let wrong = format!("{}/hold-2.json", job().dir());
            assert!(matches!(
                check_record_file(&wrong, text.as_bytes()),
                Err(RecordError::Misplaced { .. })
            ));
        }
    }

    #[test]
    fn a_hold_record_numbered_zero_or_with_an_unknown_state_is_refused() {
        let zero = AnyRecord::Hold(hold(0, HoldState::OnHold));
        assert!(matches!(
            check_record_file(&zero.canonical_path(), zero.to_json().as_bytes()),
            Err(RecordError::Invalid { .. })
        ));
        let text = AnyRecord::Hold(hold(1, HoldState::OnHold))
            .to_json()
            .replace("on_hold", "held");
        assert!(matches!(
            AnyRecord::parse("x", text.as_bytes()),
            Err(RecordError::Parse { .. })
        ));
    }

    fn focus(n: u32, text: &str, set_by: &str) -> FocusRecord {
        FocusRecord {
            n,
            at: ts(),
            text: text.into(),
            set_by: set_by.into(),
        }
    }

    #[test]
    fn a_focus_record_has_a_path_a_schema_and_round_trips() {
        let record = AnyRecord::Focus(focus(1, "speed first", "Ada"));
        assert_eq!(record.schema(), "athena.focus/1");
        assert_eq!(record.canonical_path(), "records/focus/focus-1.json");
        let text = record.to_json();
        assert!(text.contains("\"schema\": \"athena.focus/1\""));
        assert_eq!(AnyRecord::parse("x", text.as_bytes()).unwrap(), record);
        let checked = check_record_file("records/focus/focus-1.json", text.as_bytes()).unwrap();
        assert_eq!(checked, Some(record));
        assert!(matches!(
            check_record_file("records/focus/focus-2.json", text.as_bytes()),
            Err(RecordError::Misplaced { .. })
        ));
    }

    #[test]
    fn a_focus_record_with_no_text_no_author_or_no_number_is_refused() {
        for bad in [
            focus(1, "  ", "Ada"),
            focus(1, "speed first", ""),
            focus(0, "speed first", "Ada"),
        ] {
            let record = AnyRecord::Focus(bad);
            assert!(matches!(
                check_record_file(&record.canonical_path(), record.to_json().as_bytes()),
                Err(RecordError::Invalid { .. })
            ));
        }
    }

    fn sample_introspection(kind: &str) -> String {
        format!(
            r#"{{
  "schema": "athena.introspection/1",
  "id": "20261010-153012-k7q2",
  "at": "2026-10-10T15:30:12Z",
  "trigger": "rejection",
  "job": "20261010-153012-k7q2",
  "intervention": null,
  "class_of_work": "code",
  "category": "rejected",
  "what_happened": "the job was rejected",
  "failure_class": "20261010-153012-k7q2",
  "new_class": false,
  "earlier_cases": [],
  "decisions_matched": [],
  "cause": "a cause",
  "prevention_kind": "{kind}",
  "prevention": "a prevention",
  "proposed_request": "a request",
  "evidence": null,
  "status": "recorded",
  "failure": null,
  "flags": [],
  "dropped_ids": [],
  "from_jev": false
}}
"#
        )
    }

    #[test]
    fn introspections_of_the_three_old_kinds_still_parse_and_keep_their_bytes() {
        for (text, kind) in [
            ("default", PreventionKind::Default),
            ("code-check", PreventionKind::CodeCheck),
            ("design-change", PreventionKind::DesignChange),
        ] {
            let bytes = sample_introspection(text);
            let path = "records/introspections/2026-10/20261010-153012-k7q2.json";
            let checked = check_record_file(path, bytes.as_bytes())
                .unwrap()
                .expect("a record");
            let AnyRecord::Introspection(i) = &checked else {
                panic!("not an introspection");
            };
            assert_eq!(i.prevention_kind, Some(kind));
            assert_eq!(kind.as_str(), text);
            assert!(!kind.loosens());
            let back = AnyRecord::parse(path, checked.to_json().as_bytes()).unwrap();
            assert_eq!(back, checked);
            assert_eq!(checked.to_json().trim_end(), bytes.trim_end(), "{text}");
        }
    }

    /// The sample with the three lists written after `cause`, where the record serialises them.
    fn sample_introspection_with_lists(others: &str, open: &str, against: &str) -> String {
        sample_introspection("code-check").replace(
            "  \"cause\": \"a cause\",\n",
            &format!(
                "  \"cause\": \"a cause\",\n  \"other_causes\": {others},\n  \"open_hypotheses\": {open},\n  \"contradictions\": {against},\n"
            ),
        )
    }

    #[test]
    fn an_introspection_with_the_three_lists_round_trips_and_an_old_one_has_them_empty() {
        let path = "records/introspections/2026-10/20261010-153012-k7q2.json";
        let old = check_record_file(path, sample_introspection("default").as_bytes())
            .unwrap()
            .expect("a record");
        let AnyRecord::Introspection(o) = &old else {
            panic!("not an introspection");
        };
        assert!(
            o.other_causes.is_empty()
                && o.open_hypotheses.is_empty()
                && o.contradictions.is_empty()
        );

        let bytes = sample_introspection_with_lists(r#"["b", "c"]"#, r#"["h"]"#, r#"["x"]"#);
        assert!(bytes.contains("other_causes"));
        let checked = check_record_file(path, bytes.as_bytes())
            .unwrap()
            .expect("a record");
        let AnyRecord::Introspection(i) = &checked else {
            panic!("not an introspection");
        };
        assert_eq!(i.other_causes, ["b", "c"]);
        assert_eq!(i.open_hypotheses, ["h"]);
        assert_eq!(i.contradictions, ["x"]);
        let back = AnyRecord::parse(path, checked.to_json().as_bytes()).unwrap();
        assert_eq!(back, checked);
    }

    #[test]
    fn an_introspection_list_with_a_blank_entry_or_too_many_entries_is_invalid() {
        let path = "records/introspections/2026-10/20261010-153012-k7q2.json";
        let six = r#"["1", "2", "3", "4", "5", "6"]"#;
        let five = r#"["1", "2", "3", "4", "5"]"#;
        assert!(
            check_record_file(
                path,
                sample_introspection_with_lists(five, "[]", "[]").as_bytes()
            )
            .is_ok()
        );
        for bytes in [
            sample_introspection_with_lists("[\" \"]", "[]", "[]"),
            sample_introspection_with_lists("[]", six, "[]"),
            sample_introspection_with_lists("[]", "[]", six),
        ] {
            assert!(matches!(
                check_record_file(path, bytes.as_bytes()),
                Err(RecordError::Invalid { .. })
            ));
        }
    }

    #[test]
    fn relax_and_remove_introspections_round_trip_and_only_they_loosen() {
        for (text, kind) in [
            ("relax", PreventionKind::Relax),
            ("remove", PreventionKind::Remove),
        ] {
            let bytes = sample_introspection(text);
            let path = "records/introspections/2026-10/20261010-153012-k7q2.json";
            let checked = check_record_file(path, bytes.as_bytes())
                .unwrap()
                .expect("a record");
            let AnyRecord::Introspection(i) = &checked else {
                panic!("not an introspection");
            };
            assert_eq!(i.prevention_kind, Some(kind));
            assert_eq!(PreventionKind::parse(text), Some(kind));
            assert!(kind.loosens());
            assert_eq!(checked.to_json().trim_end(), bytes.trim_end(), "{text}");
        }
        assert_eq!(PreventionKind::parse("loosen"), None);
        assert!(
            check_record_file(
                "records/introspections/2026-10/20261010-153012-k7q2.json",
                sample_introspection("loosen").as_bytes()
            )
            .is_err()
        );
        assert_eq!(PreventionKind::ALL.len(), 5);
    }

    fn sample_candidate() -> Candidate {
        Candidate {
            job: job(),
            base_commit: "0".repeat(40),
            base_fingerprint: None,
            previous_job: None,
            candidate_commit: None,
            candidate_tree: None,
            changes: vec![],
            fingerprint: None,
            violations: vec![],
            producer_report: None,
        }
    }

    #[test]
    fn candidate_2_round_trips_with_and_without_a_previous_job() {
        let mut c = sample_candidate();
        c.base_fingerprint = Some("f".repeat(64));
        let none = AnyRecord::Candidate2(Box::new(c.clone()));
        let text = none.to_json();
        assert!(text.contains("\"schema\": \"athena.candidate/2\""));
        assert!(text.contains("base_fingerprint"));
        assert!(!text.contains("previous_job"), "absent, not null: {text}");
        assert_eq!(AnyRecord::parse("x", text.as_bytes()).unwrap(), none);
        assert_eq!(none.schema(), "athena.candidate/2");

        c.previous_job = Some(JobId::parse("20261009-101010-abcd").unwrap());
        let some = AnyRecord::Candidate2(Box::new(c));
        let text = some.to_json();
        assert!(text.contains("\"previous_job\": \"20261009-101010-abcd\""));
        assert_eq!(AnyRecord::parse("x", text.as_bytes()).unwrap(), some);
    }

    fn sample_report(no_change: Option<NoChange>) -> ProducerReport {
        ProducerReport {
            summary: "nothing to do".into(),
            files: vec![],
            checks_run: vec![],
            criteria_check: vec![],
            open_issues: vec![],
            mistakes: vec![],
            learnings: vec![],
            desires: vec![],
            no_change,
        }
    }

    #[test]
    fn a_report_made_before_the_no_change_outcome_still_parses_and_is_written_back_unchanged() {
        let old = r#"{"summary":"s","files":[],"checks_run":[],"open_issues":[],"mistakes":[],"learnings":[],"desires":[]}"#;
        let report: ProducerReport = serde_json::from_str(old).unwrap();
        assert_eq!(report.no_change, None);
        let written = serde_json::to_string(&report).unwrap();
        assert!(
            !written.contains("no_change"),
            "unset is not written: {written}"
        );
    }

    #[test]
    fn candidate_2_round_trips_with_and_without_a_no_change_outcome() {
        for no_change in [
            None,
            Some(NoChange {
                kind: "already_done".into(),
                detail: "Transport sits in gh/transport.rs".into(),
            }),
        ] {
            let mut c = sample_candidate();
            c.base_fingerprint = Some("f".repeat(64));
            c.producer_report = Some(sample_report(no_change.clone()));
            let record = AnyRecord::Candidate2(Box::new(c));
            let text = record.to_json();
            assert_eq!(
                text.contains("\"no_change\""),
                no_change.is_some(),
                "{text}"
            );
            assert_eq!(AnyRecord::parse("x", text.as_bytes()).unwrap(), record);
        }
    }

    #[test]
    fn candidate_1_still_parses_and_keeps_its_shape() {
        let record = AnyRecord::Candidate(Box::new(sample_candidate()));
        let text = record.to_json();
        assert!(text.contains("\"schema\": \"athena.candidate/1\""));
        assert!(!text.contains("base_fingerprint") && !text.contains("previous_job"));
        assert_eq!(AnyRecord::parse("x", text.as_bytes()).unwrap(), record);
    }

    #[test]
    fn candidate_versions_keep_to_their_fields() {
        // A /2 without the base fingerprint, and a /1 with a /2 field, are both invalid.
        let no_fp = AnyRecord::Candidate2(Box::new(sample_candidate())).to_json();
        assert!(matches!(
            AnyRecord::parse("x", no_fp.as_bytes()),
            Err(RecordError::Invalid { .. })
        ));
        let mut c = sample_candidate();
        c.previous_job = Some(JobId::parse("20261009-101010-abcd").unwrap());
        let v1 = AnyRecord::Candidate(Box::new(c)).to_json();
        assert!(matches!(
            AnyRecord::parse("x", v1.as_bytes()),
            Err(RecordError::Invalid { .. })
        ));
    }

    #[test]
    fn a_quantity_score_written_before_the_provisional_field_still_reads() {
        // m8 of the v0 acceptance review: the new field has a serde default, and a firm score omits it.
        let old = r#"{"quantity":"cost_usd","actual":0.4,"floor":0.001,"forecasts":{}}"#;
        let score: QuantityScore = serde_json::from_str(old).unwrap();
        assert!(!score.provisional);
        assert!(
            !serde_json::to_string(&score)
                .unwrap()
                .contains("provisional")
        );
        let mut bound = score;
        bound.provisional = true;
        let text = serde_json::to_string(&bound).unwrap();
        assert!(
            serde_json::from_str::<QuantityScore>(&text)
                .unwrap()
                .provisional
        );
    }

    #[test]
    fn unknown_fields_in_records_are_refused() {
        // M5 (blocks): an unknown field fails parsing.
        let mut value: serde_json::Value =
            serde_json::from_str(&AnyRecord::Reading(Box::new(sample_reading())).to_json())
                .unwrap();
        value["surprise"] = serde_json::json!(1);
        let text = serde_json::to_vec(&value).unwrap();
        assert!(matches!(
            AnyRecord::parse("x", &text),
            Err(RecordError::Parse { .. })
        ));
    }

    #[test]
    fn records_must_sit_at_their_canonical_path() {
        let record = AnyRecord::Reading(Box::new(sample_reading()));
        let text = record.to_json();
        let good = "records/jobs/2026-10/20261010-153012-k7q2/reading-1.json";
        assert!(check_record_file(good, text.as_bytes()).is_ok());
        let bad = "records/jobs/2026-10/20261010-153012-k7q2/reading-2.json";
        assert!(matches!(
            check_record_file(bad, text.as_bytes()),
            Err(RecordError::Misplaced { .. })
        ));
    }

    #[test]
    fn out_of_order_quantiles_are_refused() {
        // M4: a reading with p10 > p50 does not parse.
        let text = AnyRecord::Reading(Box::new(sample_reading()))
            .to_json()
            .replacen("\"p10\": 0.2", "\"p10\": 0.9", 1);
        assert!(AnyRecord::parse("x", text.as_bytes()).is_err());
    }

    #[test]
    fn probabilities_must_lie_in_the_unit_interval() {
        let mut reading = sample_reading();
        reading.deliverables[0].p = 1.5;
        let text = AnyRecord::Reading(Box::new(reading)).to_json();
        assert!(matches!(
            AnyRecord::parse("x", text.as_bytes()),
            Err(RecordError::Invalid { .. })
        ));
    }

    #[test]
    fn markdown_briefs_are_allowed_only_where_documented() {
        let dir = "records/jobs/2026-10/20261010-153012-k7q2";
        assert!(check_record_file(&format!("{dir}/brief-producer.md"), b"x").is_ok());
        assert!(check_record_file(&format!("{dir}/notes.md"), b"x").is_err());
        assert!(check_record_file("records/jobs/bootstrap/brief.md", b"x").is_ok());
        assert!(check_record_file("records/jobs/bootstrap/brief-update-1.md", b"x").is_ok());
        assert!(check_record_file("records/jobs/bootstrap/brief-update-x.md", b"x").is_err());
        assert!(check_record_file("records/readme.txt", b"x").is_err());
    }

    #[test]
    fn interventions_live_in_their_month() {
        let id = RecordId::parse("20261010-153012-k7q2").unwrap();
        let record = AnyRecord::Intervention(Intervention {
            id,
            at: ts(),
            what: "Fixed a typo by hand".into(),
            why: "The verifier missed it".into(),
            class_of_work: "docs".into(),
            category: InterventionCategory::DidWorkThemselves,
            job: None,
            source: InterventionSource::User,
            with_staged_changes: true,
        });
        assert_eq!(
            record.canonical_path(),
            "records/interventions/2026-10/20261010-153012-k7q2.json"
        );
    }

    #[test]
    fn mistake_is_a_category_and_in_all() {
        assert_eq!(InterventionCategory::ALL.len(), 9);
        assert!(InterventionCategory::ALL.contains(&InterventionCategory::Mistake));
        assert_eq!(InterventionCategory::Mistake.as_str(), "mistake");
    }

    #[test]
    fn parse_accepts_mistake() {
        assert_eq!(
            InterventionCategory::parse("mistake"),
            Some(InterventionCategory::Mistake)
        );
        assert_eq!(InterventionCategory::parse("mistakes"), None);
    }

    #[test]
    fn is_mistake_rule() {
        use InterventionCategory::*;
        assert!(is_mistake(Mistake, "anything"));
        assert!(is_mistake(Other, "Mistake: the verifier was skipped"));
        assert!(!is_mistake(Other, "A mistake: not a prefix"));
        assert!(!is_mistake(Other, "mistake: case differs"));
        assert!(!is_mistake(CorrectedOutput, "Mistake: wrong category"));
    }

    fn host_error(status: Option<u16>) -> HostError {
        HostError {
            id: RecordId::parse("20261013-192156-k7q2").unwrap(),
            at: ts(),
            status_code: status,
            operation: HostOperation::Merge,
            job: JobId::parse("20261013-192156-ar1x").unwrap(),
            phase: WorkPhase::Landing,
            message_class: MessageClass::of_status(status),
            derived: false,
            source_intervention: None,
            checks_pending: false,
        }
    }

    #[test]
    fn an_old_format_host_error_still_validates_and_writes_back_unchanged() {
        // A record written before `checks_pending` existed: the field is absent.
        let path = "records/host-errors/2026-11/20261101-174051-3t6g.json";
        let old = "{\n  \"schema\": \"athena.host-error/1\",\n  \"id\": \"20261101-174051-3t6g\",\n  \"at\": \"2026-11-01T17:40:51Z\",\n  \"status_code\": 405,\n  \"operation\": \"merge\",\n  \"job\": \"20261101-171216-w4rd\",\n  \"phase\": \"landing\",\n  \"message_class\": \"method-not-allowed\"\n}\n";
        let Ok(Some(AnyRecord::HostError(error))) = check_record_file(path, old.as_bytes()) else {
            panic!("the old record must validate at its canonical path");
        };
        assert!(!error.checks_pending);
        let record = AnyRecord::HostError(error.clone());
        assert_eq!(record.canonical_path(), path);
        assert_eq!(record.to_json(), old);
        // The field is written only when true, and then reads back.
        let mut marked = error;
        marked.checks_pending = true;
        let marked_text = AnyRecord::HostError(marked.clone()).to_json();
        assert!(marked_text.contains("\"checks_pending\": true"));
        assert_eq!(
            check_record_file(path, marked_text.as_bytes()).unwrap(),
            Some(AnyRecord::HostError(marked))
        );
        // A field outside the record is still refused.
        let stray = old.replace("\"phase\"", "\"stderr\": \"x\", \"phase\"");
        assert!(check_record_file(path, stray.as_bytes()).is_err());
    }

    #[test]
    fn a_derived_host_error_needs_a_source_and_an_observed_one_has_none() {
        let mut derived = host_error(Some(405));
        derived.derived = true;
        // A derived record without a source is refused.
        let json = AnyRecord::HostError(derived.clone()).to_json();
        assert!(AnyRecord::parse("x", json.as_bytes()).is_err());
        derived.source_intervention = Some(RecordId::parse("20261013-192156-ar1x").unwrap());
        let record = AnyRecord::HostError(derived);
        let text = record.to_json();
        assert!(text.contains("\"derived\": true"));
        assert!(text.contains("\"source_intervention\": \"20261013-192156-ar1x\""));
        assert_eq!(AnyRecord::parse("x", text.as_bytes()).unwrap(), record);
        // A source on an observed record is refused, and an observed record writes neither field.
        let mut sourced = host_error(Some(405));
        sourced.source_intervention = Some(RecordId::parse("20261013-192156-ar1x").unwrap());
        let json = AnyRecord::HostError(sourced).to_json();
        assert!(AnyRecord::parse("x", json.as_bytes()).is_err());
        let observed = AnyRecord::HostError(host_error(Some(405))).to_json();
        assert!(!observed.contains("derived") && !observed.contains("source_intervention"));
    }

    #[test]
    fn a_host_error_has_a_status_a_closed_class_and_no_text() {
        let record = AnyRecord::HostError(host_error(Some(405)));
        assert_eq!(
            record.canonical_path(),
            "records/host-errors/2026-10/20261013-192156-k7q2.json"
        );
        let text = record.to_json();
        assert!(text.contains("\"schema\": \"athena.host-error/1\""));
        assert!(text.contains("\"status_code\": 405"));
        assert!(text.contains("\"operation\": \"merge\""));
        assert!(text.contains("\"message_class\": \"method-not-allowed\""));
        assert_eq!(AnyRecord::parse("x", text.as_bytes()).unwrap(), record);
        // A field outside the record, such as the host's text, is refused.
        let with_text = text.replace("\"phase\"", "\"stderr\": \"x\", \"phase\"");
        assert!(AnyRecord::parse("x", with_text.as_bytes()).is_err());
        // No HTTP answer: the status is absent.
        let none = AnyRecord::HostError(host_error(None));
        assert!(AnyRecord::parse("x", none.to_json().as_bytes()).is_ok());
        // The class follows from the status.
        let mut wrong = host_error(Some(405));
        wrong.message_class = MessageClass::NotFound;
        assert!(AnyRecord::parse("x", AnyRecord::HostError(wrong).to_json().as_bytes()).is_err());
        let bad_status = host_error(Some(999));
        assert!(
            AnyRecord::parse("x", AnyRecord::HostError(bad_status).to_json().as_bytes()).is_err()
        );
    }

    #[test]
    fn the_message_classes_and_operations_are_closed_lists() {
        for class in MessageClass::ALL {
            assert_eq!(MessageClass::parse(class.as_str()), Some(class));
        }
        for operation in HostOperation::ALL {
            assert_eq!(HostOperation::parse(operation.as_str()), Some(operation));
        }
        assert_eq!(HostOperation::parse("delete-repo"), None);
        assert_eq!(
            MessageClass::of_status(Some(503)),
            MessageClass::ServerError
        );
        assert_eq!(
            MessageClass::of_status(Some(418)),
            MessageClass::OtherStatus
        );
    }

    fn state(id: &str, at: &str, decision: &str, kind: DecisionStateKind) -> DecisionState {
        DecisionState {
            id: RecordId::parse(id).unwrap(),
            at: at.parse().unwrap(),
            decision: decision.into(),
            state: kind,
            signal: None,
            firing_records: vec![RecordId::parse("20261013-192156-k7q2").unwrap()],
            decision_job: None,
        }
    }

    #[test]
    fn decision_states_fold_to_the_latest_state_per_decision() {
        let states = [
            state(
                "20261014-100000-aaaa",
                "2026-10-14T10:00:00Z",
                "D87",
                DecisionStateKind::Cleared,
            ),
            state(
                "20261013-100000-bbbb",
                "2026-10-13T10:00:00Z",
                "D87",
                DecisionStateKind::UnderReview,
            ),
            state(
                "20261013-110000-cccc",
                "2026-10-13T11:00:00Z",
                "D12",
                DecisionStateKind::UnderReview,
            ),
        ];
        let folded = fold_decision_states(&states);
        assert_eq!(folded["D87"], DecisionStateKind::Cleared);
        assert_eq!(folded["D12"], DecisionStateKind::UnderReview);
        assert_eq!(folded.len(), 2);
        assert!(fold_decision_states(&[]).is_empty());
        let record = AnyRecord::DecisionState(states[1].clone());
        assert_eq!(
            record.canonical_path(),
            "records/decision-states/2026-10/20261013-100000-bbbb.json"
        );
        assert_eq!(
            AnyRecord::parse("x", record.to_json().as_bytes()).unwrap(),
            record
        );
        // An under-review change cites the records that fired it.
        let mut uncited = states[1].clone();
        uncited.firing_records.clear();
        assert!(
            AnyRecord::parse("x", AnyRecord::DecisionState(uncited).to_json().as_bytes()).is_err()
        );
    }

    #[test]
    fn default_expectations_are_keyed_by_the_default_and_validated() {
        let expectations = DefaultExpectations {
            id: RecordId::parse("20261014-100000-dddd").unwrap(),
            at: ts(),
            default_id: "small".into(),
            assumptions: vec!["A focused change is easier to review.".into()],
            signals: vec![ExpectationSignal {
                source: SignalSource::Rejection,
                strength: SignalStrength::Soft,
                text: "a change rejected as too large".into(),
                threshold_jobs: 2,
            }],
            defaults_version: 1,
            proposal: None,
        };
        let record = AnyRecord::DefaultExpectations(expectations.clone());
        assert_eq!(
            record.canonical_path(),
            "records/default-expectations/2026-10/20261014-100000-dddd.json"
        );
        assert_eq!(
            AnyRecord::parse("x", record.to_json().as_bytes()).unwrap(),
            record
        );
        let mut weak = expectations.clone();
        weak.signals[0].threshold_jobs = 1;
        assert!(weak.validate().is_err(), "a soft signal needs repeats");
        let mut unkeyed = expectations;
        unkeyed.default_id = " ".into();
        assert!(unkeyed.validate().is_err());
    }

    fn sample_proposal() -> ProposalRecord {
        ProposalRecord {
            record: RecordId::parse("20261014-110000-eeee").unwrap(),
            at: ts(),
            id: "field-with-use".into(),
            kind: ProposalKind::Add,
            statement: "A new field lands with its first use.".into(),
            applies_to: vec!["code".into()],
            version: ProposalVersion { from: 1, to: 2 },
            sources: vec![ProposalSource {
                introspection: RecordId::parse("20261014-100000-aaaa").unwrap(),
                intervention: None,
                job: job(),
                at: ts(),
            }],
            failure_class: RecordId::parse("20261014-090000-cccc").unwrap(),
            status: ProposalStatus::Pending,
            candidate_sha256: "a".repeat(64),
            notes: "No Jev.".into(),
            evaluation: None,
        }
    }

    #[test]
    fn a_proposal_with_an_evaluation_is_an_accepted_record_and_round_trips() {
        let mut proposal = sample_proposal();
        proposal.evaluation = Some(ProposalEvaluation {
            cases: vec!["20261014-120000-ffff".into()],
            baseline_failures: 1,
            candidate_failures: 0,
        });
        let record = AnyRecord::DefaultsProposal(Box::new(proposal.clone()));
        let path = record.canonical_path();
        let checked = check_record_file(&path, record.to_json().as_bytes()).unwrap();
        assert_eq!(checked, Some(record.clone()));
        assert_eq!(
            AnyRecord::parse("x", record.to_json().as_bytes()).unwrap(),
            record
        );
        proposal.evaluation = Some(ProposalEvaluation {
            cases: Vec::new(),
            baseline_failures: 0,
            candidate_failures: 0,
        });
        let bad = AnyRecord::DefaultsProposal(Box::new(proposal));
        assert!(AnyRecord::parse("x", bad.to_json().as_bytes()).is_err());
    }

    #[test]
    fn defaults_proposals_have_a_path_a_schema_and_are_validated() {
        let proposal = sample_proposal();
        let record = AnyRecord::DefaultsProposal(Box::new(proposal.clone()));
        assert_eq!(record.schema(), "athena.defaults-proposal/1");
        assert_eq!(
            record.canonical_path(),
            "records/defaults-proposals/2026-10/20261014-110000-eeee.json"
        );
        assert_eq!(
            AnyRecord::parse("x", record.to_json().as_bytes()).unwrap(),
            record
        );
        assert!(
            record
                .to_json()
                .contains("\"schema\": \"athena.defaults-proposal/1\"")
        );
        let invalid = |edit: &dyn Fn(&mut ProposalRecord)| {
            let mut p = proposal.clone();
            edit(&mut p);
            AnyRecord::parse(
                "x",
                AnyRecord::DefaultsProposal(Box::new(p))
                    .to_json()
                    .as_bytes(),
            )
            .is_err()
        };
        assert!(invalid(&|p| p.version.to = 5));
        assert!(invalid(&|p| p.sources.clear()));
        assert!(invalid(&|p| p.statement = " ".into()));
        assert!(invalid(&|p| p.candidate_sha256 = "abc".into()));
        assert!(invalid(
            &|p| p.status = ProposalStatus::Rejected { reason: " ".into() }
        ));
        let mut accepted = proposal;
        accepted.status = ProposalStatus::Accepted { job: job() };
        assert!(accepted.validate().is_ok());
        assert_eq!(accepted.status.as_str(), "accepted");
    }

    #[test]
    fn class_labels_have_a_path_a_schema_and_round_trip() {
        let label = ClassLabel {
            id: RecordId::parse("20261014-120000-1abc").unwrap(),
            at: ts(),
            introspection: RecordId::parse("20261014-100000-aaaa").unwrap(),
            class: None,
            labeled_by: Labeler::StrongerModel,
            is_repeat: None,
            note: None,
            basis: vec![],
        };
        let record = AnyRecord::ClassLabel(label.clone());
        assert_eq!(record.schema(), "athena.class-label/1");
        assert_eq!(
            record.canonical_path(),
            "records/class-labels/2026-10/20261014-120000-1abc.json"
        );
        let json = record.to_json();
        assert!(json.contains("\"labeled_by\": \"stronger-model\""));
        assert!(!json.contains("is_repeat"));
        assert!(!json.contains("note"));
        assert!(!json.contains("basis"));
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), record);

        // With a note the field is written and read back.
        let noted = AnyRecord::ClassLabel(ClassLabel {
            note: Some("The cause is the missing default.".into()),
            ..label.clone()
        });
        let json = noted.to_json();
        assert!(json.contains("\"note\": \"The cause is the missing default.\""));
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), noted);
    }

    #[test]
    fn a_label_written_before_evidence_labels_existed_still_parses_and_a_derived_one_round_trips() {
        // The exact shape older records have: no `basis` key.
        let old = r#"{
          "schema": "athena.class-label/1",
          "id": "20261014-120000-1abc",
          "at": "2026-10-14T12:00:00Z",
          "introspection": "20261014-100000-aaaa",
          "class": null,
          "labeled_by": "person"
        }"#;
        let AnyRecord::ClassLabel(old) = AnyRecord::parse("x", old.as_bytes()).unwrap() else {
            panic!("a class label");
        };
        assert!(old.basis.is_empty());

        let derived = ClassLabel {
            id: RecordId::parse("20261014-120000-1abc").unwrap(),
            at: ts(),
            introspection: RecordId::parse("20261014-100000-aaaa").unwrap(),
            class: Some(RecordId::parse("20261001-000000-c001").unwrap()),
            labeled_by: Labeler::Evidence,
            is_repeat: None,
            note: None,
            basis: vec![LabelBasis {
                kind: EvidenceKind::RetryOutcome,
                records: vec!["20261014-100000-aaaa".into(), "20261014-110000-bbbb".into()],
                detail: "The retry landed.".into(),
            }],
        };
        let record = AnyRecord::ClassLabel(derived.clone());
        let json = record.to_json();
        assert!(json.contains("\"labeled_by\": \"evidence\""));
        assert!(json.contains("\"kind\": \"retry-outcome\""));
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), record);

        // The rules: evidence needs a basis and a class; a person's label has no basis.
        let refused = |edit: &dyn Fn(&mut ClassLabel)| {
            let mut l = derived.clone();
            edit(&mut l);
            AnyRecord::parse("x", AnyRecord::ClassLabel(l).to_json().as_bytes()).is_err()
        };
        assert!(refused(&|l| l.basis.clear()));
        assert!(refused(&|l| l.class = None));
        assert!(refused(&|l| l.basis[0].records.clear()));
        assert!(refused(&|l| l.basis[0].detail = " ".into()));
        assert!(refused(&|l| l.labeled_by = Labeler::Person));
        assert!(!Labeler::Evidence.is_judgement());
        assert!(Labeler::StrongerModel.is_judgement());
    }

    fn sample_work_item() -> WorkItem {
        WorkItem {
            id: RecordId::parse("20261014-120000-1abc").unwrap(),
            at: ts(),
            role: "steward".into(),
            kind: WorkItemKind::BlindSpot,
            key: "blind-spot:records".into(),
            title: "The records could not be read".into(),
            rung: crate::roles::Ladder::Implementation,
            source: Reference::path("records/"),
            evidence: vec![],
            brief: None,
        }
    }

    fn sample_brief() -> OwnerBrief {
        OwnerBrief {
            question: "Should the steward retry a failed check once?".into(),
            if_yes: "A failed check runs a second time.".into(),
            if_no: "A failed check stops the job at once.".into(),
            cost: "About one more minute per failure.".into(),
            recommendation: "Yes: most failures here pass on the second run.".into(),
        }
    }

    #[test]
    fn a_work_item_brief_is_optional_round_trips_and_has_no_empty_field() {
        // An old record has no brief key and still parses.
        let old = AnyRecord::WorkItem(Box::new(sample_work_item()));
        let old_json = old.to_json();
        assert!(!old_json.contains("brief"), "{old_json}");
        assert_eq!(
            check_record_file(&old.canonical_path(), old_json.as_bytes()).unwrap(),
            Some(old)
        );
        // A work item with a brief round-trips through the file check.
        let mut with = sample_work_item();
        with.kind = WorkItemKind::Proposal;
        with.evidence.push(Reference::job(&job()));
        with.brief = Some(sample_brief());
        let record = AnyRecord::WorkItem(Box::new(with.clone()));
        let json = record.to_json();
        assert!(json.contains("\"if_yes\""), "{json}");
        assert_eq!(
            check_record_file(&record.canonical_path(), json.as_bytes()).unwrap(),
            Some(record)
        );
        // A brief with an empty field is refused, whichever field it is.
        for field in 0..5 {
            let mut bad = with.clone();
            let b = bad.brief.as_mut().unwrap();
            let slot = match field {
                0 => &mut b.question,
                1 => &mut b.if_yes,
                2 => &mut b.if_no,
                3 => &mut b.cost,
                _ => &mut b.recommendation,
            };
            *slot = "  ".into();
            let bad = AnyRecord::WorkItem(Box::new(bad));
            assert!(
                check_record_file(&bad.canonical_path(), bad.to_json().as_bytes()).is_err(),
                "field {field}"
            );
        }
    }

    #[test]
    fn roles_and_work_items_have_paths_schemas_and_round_trip() {
        let role = AnyRecord::Role(Box::new(crate::roles::steward(ts())));
        assert_eq!(role.schema(), "athena.role/1");
        assert_eq!(role.canonical_path(), "records/roles/steward-3.json");
        let json = role.to_json();
        assert!(json.contains("\"highest_rung\": \"defaults-conventions\""));
        assert_eq!(
            AnyRecord::parse(&role.canonical_path(), json.as_bytes()).unwrap(),
            role
        );

        let item = AnyRecord::WorkItem(Box::new(sample_work_item()));
        assert_eq!(item.schema(), "athena.work-item/1");
        assert_eq!(
            item.canonical_path(),
            "records/work-items/2026-10/20261014-120000-1abc.json"
        );
        let json = item.to_json();
        assert!(json.contains("\"kind\": \"blind-spot\""));
        assert!(json.contains("\"rung\": \"implementation\""));
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), item);
        assert!(check_record_file(&item.canonical_path(), json.as_bytes()).is_ok());
    }

    #[test]
    fn a_work_decision_has_a_path_a_schema_and_round_trips() {
        let id = RecordId::parse("20261015-090000-2def").unwrap();
        let approved = AnyRecord::WorkDecision(WorkDecision {
            id: id.clone(),
            at: ts(),
            work_item: RecordId::parse("20261014-120000-1abc").unwrap(),
            decision: WorkDecisionState::Approved { job: job() },
        });
        assert_eq!(approved.schema(), "athena.work-decision/1");
        assert_eq!(
            approved.canonical_path(),
            "records/work-decisions/2026-10/20261015-090000-2def.json"
        );
        let json = approved.to_json();
        assert!(json.contains("\"state\": \"approved\""));
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), approved);
        assert!(check_record_file(&approved.canonical_path(), json.as_bytes()).is_ok());

        let rejected = |reason: &str| {
            AnyRecord::WorkDecision(WorkDecision {
                id: id.clone(),
                at: ts(),
                work_item: RecordId::parse("20261014-120000-1abc").unwrap(),
                decision: WorkDecisionState::Rejected {
                    reason: reason.into(),
                },
            })
        };
        let good = rejected("Not now.");
        assert_eq!(
            AnyRecord::parse("x", good.to_json().as_bytes()).unwrap(),
            good
        );
        assert!(AnyRecord::parse("x", rejected(" ").to_json().as_bytes()).is_err());
    }

    fn sample_rating() -> AudienceRating {
        AudienceRating {
            id: RecordId::parse("20261015-090000-2def").unwrap(),
            at: ts(),
            target: "https://example.invalid/page/".into(),
            audience: "newcomer".into(),
            rating: 4,
            findings: vec![AudienceFinding {
                text: "The first screen says what the page is for.".into(),
                source: "heading: Start here".into(),
            }],
        }
    }

    #[test]
    fn an_audience_rating_has_a_path_a_schema_and_round_trips() {
        let record = AnyRecord::AudienceRating(Box::new(sample_rating()));
        assert_eq!(record.schema(), "athena.audience-rating/1");
        assert_eq!(
            record.canonical_path(),
            "records/audience-ratings/2026-10/20261015-090000-2def.json"
        );
        let json = record.to_json();
        assert!(json.contains("\"schema\": \"athena.audience-rating/1\""));
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), record);
        assert!(check_record_file(&record.canonical_path(), json.as_bytes()).is_ok());
    }

    #[test]
    fn a_rating_outside_one_to_five_or_a_finding_without_evidence_is_refused() {
        for rating in RATING_RANGE {
            let ok = AudienceRating {
                rating,
                ..sample_rating()
            };
            assert_eq!(ok.validate(), Ok(()), "{rating}");
        }
        for rating in [0, 6, 255] {
            let bad = AudienceRating {
                rating,
                ..sample_rating()
            };
            assert!(bad.validate().is_err(), "{rating}");
            let record = AnyRecord::AudienceRating(Box::new(bad));
            assert!(AnyRecord::parse("x", record.to_json().as_bytes()).is_err());
        }
        let mut no_source = sample_rating();
        no_source.findings[0].source = "  ".into();
        assert!(no_source.validate().is_err());
        let mut no_text = sample_rating();
        no_text.findings[0].text.clear();
        assert!(no_text.validate().is_err());
        let mut no_target = sample_rating();
        no_target.target.clear();
        assert!(no_target.validate().is_err());
    }

    fn sample_audit() -> Audit {
        Audit {
            id: RecordId::parse("20261017-090000-3abc").unwrap(),
            at: ts(),
            target: JobId::parse("20261016-120000-4def").unwrap(),
            summary: "The outcome serves the goal.".into(),
            findings: vec![AuditFinding {
                text: "The command exists.".into(),
                evidence: "crates/athena-cli/src/main.rs: Command::Audit".into(),
            }],
        }
    }

    #[test]
    fn an_audit_has_a_path_a_schema_and_round_trips() {
        let record = AnyRecord::Audit(Box::new(sample_audit()));
        assert_eq!(record.schema(), "athena.audit/1");
        assert_eq!(
            record.canonical_path(),
            "records/audits/2026-10/20261017-090000-3abc.json"
        );
        let json = record.to_json();
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), record);
        assert!(check_record_file(&record.canonical_path(), json.as_bytes()).is_ok());
    }

    #[test]
    fn an_audit_finding_without_evidence_is_refused() {
        let mut bad = sample_audit();
        bad.findings[0].evidence = "  ".into();
        assert!(bad.validate().is_err());
        let record = AnyRecord::Audit(Box::new(bad));
        assert!(AnyRecord::parse("x", record.to_json().as_bytes()).is_err());
        let mut no_summary = sample_audit();
        no_summary.summary.clear();
        assert!(no_summary.validate().is_err());
    }

    fn sweep_with(id: &str) -> HestiaSweep {
        HestiaSweep {
            id: RecordId::parse(id).unwrap(),
            at: ts(),
            checks: vec![
                HestiaCheckResult {
                    check: "branches".into(),
                    examined: 4,
                    count: 0,
                    errored: false,
                },
                HestiaCheckResult {
                    check: "binary-age".into(),
                    examined: 0,
                    count: 1,
                    errored: true,
                },
            ],
            findings: vec![HestiaFinding {
                check: "binary-age".into(),
                subject: "binary-age".into(),
                evidence: "the check failed: no installed binary".into(),
                age_seconds: 0,
                state: HestiaFindingState::Open,
            }],
            duty_pass: None,
        }
    }

    #[test]
    fn a_hestia_sweep_with_a_duty_pass_round_trips_and_an_old_sweep_still_parses() {
        let mut sweep = sweep_with("20261026-090000-5abc");
        sweep.checks[0].check = "code-health".into();
        sweep.checks[1].check = "keep-rule".into();
        sweep.findings[0].check = "keep-rule".into();
        sweep.duty_pass = Some(DutyPass {
            occasion: DutyOccasion::Landing,
            landings: 7,
        });
        let record = AnyRecord::HestiaSweep(Box::new(sweep.clone()));
        let json = record.to_json();
        assert!(
            json.contains("\"occasion\": \"landing\"") || json.contains("\"occasion\":\"landing\"")
        );
        let parsed = AnyRecord::parse(&record.canonical_path(), json.as_bytes()).unwrap();
        assert_eq!(parsed, record);
        assert!(check_record_file(&record.canonical_path(), json.as_bytes()).is_ok());
        assert!(sweep.is_duty_pass());
        // The three occasions are written by name; only drain-start and landing passes are duty passes.
        for (occasion, name, duty_pass) in [
            (DutyOccasion::DrainStart, "drain_start", true),
            (DutyOccasion::Landing, "landing", true),
            (DutyOccasion::Sweep, "sweep", false),
        ] {
            assert_eq!(
                serde_json::to_string(&occasion).unwrap(),
                format!("\"{name}\"")
            );
            let mut s = sweep.clone();
            s.duty_pass = Some(DutyPass {
                occasion,
                landings: 0,
            });
            assert_eq!(s.is_duty_pass(), duty_pass);
        }
        // A sweep as written before the field existed parses, with no duty pass.
        let old = r#"{"schema":"athena.hestia-sweep/1","id":"20261026-090000-5abc","at":"2026-10-26T09:00:00Z","checks":[{"check":"branches","examined":4,"count":0,"errored":false}],"findings":[]}"#;
        let path = "records/hestia-sweeps/2026-10/20261026-090000-5abc.json";
        let AnyRecord::HestiaSweep(parsed_old) = AnyRecord::parse(path, old.as_bytes()).unwrap()
        else {
            panic!("an old sweep parses as a sweep");
        };
        assert!(parsed_old.duty_pass.is_none());
        assert!(!parsed_old.is_duty_pass());
        assert!(check_record_file(path, old.as_bytes()).is_ok());
        // A repeated check id is still refused, and so is an unknown field in the pass.
        let mut twice = sweep.clone();
        twice.checks[1].check = "code-health".into();
        assert!(twice.validate().is_err());
        let unknown = json.replace("\"landings\"", "\"landing_total\"");
        assert!(AnyRecord::parse(&record.canonical_path(), unknown.as_bytes()).is_err());
    }

    #[test]
    fn a_hestia_sweep_has_a_path_a_schema_and_round_trips() {
        let sweep = sweep_with("20261026-090000-5abc");
        let record = AnyRecord::HestiaSweep(Box::new(sweep.clone()));
        assert_eq!(record.schema(), "athena.hestia-sweep/1");
        assert_eq!(
            record.canonical_path(),
            "records/hestia-sweeps/2026-10/20261026-090000-5abc.json"
        );
        let json = record.to_json();
        let parsed = AnyRecord::parse(&record.canonical_path(), json.as_bytes()).unwrap();
        assert_eq!(parsed, record);
        let AnyRecord::HestiaSweep(parsed) = parsed else {
            panic!("a sweep parses as a sweep");
        };
        assert_eq!(*parsed, sweep);
        assert!(check_record_file(&record.canonical_path(), json.as_bytes()).is_ok());
    }

    #[test]
    fn a_code_health_baseline_has_a_path_a_schema_and_round_trips() {
        let baseline = CodeHealthBaseline {
            id: RecordId::parse("20261026-090000-5abc").unwrap(),
            at: ts(),
            landings: 42,
            review_queued: true,
        };
        let record = AnyRecord::CodeHealthBaseline(baseline.clone());
        assert_eq!(record.schema(), "athena.code-health-baseline/1");
        assert_eq!(
            record.canonical_path(),
            "records/code-health/2026-10/20261026-090000-5abc.json"
        );
        let json = record.to_json();
        let parsed = AnyRecord::parse(&record.canonical_path(), json.as_bytes()).unwrap();
        assert_eq!(parsed, record);
        assert!(check_record_file(&record.canonical_path(), json.as_bytes()).is_ok());
        assert_eq!(
            CodeHealthBaseline::latest(std::slice::from_ref(&baseline)),
            Some(&baseline)
        );
        assert!(CodeHealthBaseline::latest(&[]).is_none());
    }

    #[test]
    fn a_design_surface_has_a_path_a_schema_and_round_trips() {
        let mut inventory = SurfaceInventory::default();
        inventory.commands.insert("ask".into(), "visible".into());
        let surface = DesignSurface {
            id: RecordId::parse("20261026-090000-5abc").unwrap(),
            at: ts(),
            commit: "0123456789abcdef0123456789abcdef01234567".into(),
            inventory,
        };
        let record = AnyRecord::DesignSurface(Box::new(surface.clone()));
        assert_eq!(record.schema(), "athena.design-surface/1");
        assert_eq!(
            record.canonical_path(),
            "records/design-surface/2026-10/20261026-090000-5abc.json"
        );
        let json = record.to_json();
        let parsed = AnyRecord::parse(&record.canonical_path(), json.as_bytes()).unwrap();
        assert_eq!(parsed, record);
        assert!(check_record_file(&record.canonical_path(), json.as_bytes()).is_ok());
        assert_eq!(
            DesignSurface::latest(std::slice::from_ref(&surface)),
            Some(&surface)
        );
        assert!(DesignSurface::latest(&[]).is_none());
        let mut short = surface.clone();
        short.commit = "abc".into();
        assert!(short.validate().is_err());
        let mut empty = surface;
        empty.inventory = SurfaceInventory::default();
        assert!(empty.validate().is_err());
    }

    fn throughput_review_with(id: &str) -> ThroughputReview {
        ThroughputReview {
            id: RecordId::parse(id).unwrap(),
            at: ts(),
            landings: 20,
            window_median_minutes: 31.5,
            trigger: ThroughputTrigger::Cadence,
            proposal: Some(ThroughputProposal {
                bottleneck: "The verifier waits for checks that the producer already ran.".into(),
                stage: "verifier".into(),
                change: "Run the verifier while the checks run.".into(),
                expected_minutes_saved: 6.5,
                priority: "high".into(),
                request_job: Some(JobId::parse("20261026-090000-5abc").unwrap()),
            }),
            followups: vec![ThroughputFollowup {
                review: RecordId::parse("20261020-090000-4abc").unwrap(),
                change_job: JobId::parse("20261021-090000-6abc").unwrap(),
                predicted_minutes_saved: 5.0,
                baseline_median_minutes: 40.0,
                measured_median_minutes: 34.0,
                measured_minutes_saved: 6.0,
                jobs_measured: 4,
            }],
            waste: None,
            note: String::new(),
        }
    }

    #[test]
    fn a_throughput_review_without_waste_figures_still_parses_and_one_with_them_round_trips() {
        // An old record has no `waste` key.
        let old = throughput_review_with("20261026-090000-5abc");
        let record = AnyRecord::ThroughputReview(Box::new(old.clone()));
        let json = record.to_json();
        assert!(!json.contains("waste"), "an absent figure is not written");
        let parsed = AnyRecord::parse(&record.canonical_path(), json.as_bytes()).unwrap();
        let AnyRecord::ThroughputReview(parsed) = parsed else {
            panic!("a throughput review parses as one");
        };
        assert_eq!(parsed.waste, None);
        assert!(check_record_file(&record.canonical_path(), json.as_bytes()).is_ok());

        let mut new = old;
        new.trigger = ThroughputTrigger::Wasted;
        new.waste = Some(WasteFigures {
            attempts_per_landing: 2.0,
            wasted_usd: 10.5,
            first_attempt_share: 0.5,
        });
        let record = AnyRecord::ThroughputReview(Box::new(new.clone()));
        let json = record.to_json();
        assert!(json.contains("\"wasted\""));
        let parsed = AnyRecord::parse(&record.canonical_path(), json.as_bytes()).unwrap();
        assert_eq!(parsed, record);
        assert_eq!(ThroughputTrigger::Wasted.as_str(), "wasted");
    }

    #[test]
    fn waste_figures_that_are_not_numbers_a_review_can_hold_are_refused() {
        let good = WasteFigures {
            attempts_per_landing: 1.5,
            wasted_usd: 0.0,
            first_attempt_share: 1.0,
        };
        let with = |f: WasteFigures| {
            let mut r = throughput_review_with("20261026-090000-5abc");
            r.waste = Some(f);
            r.validate()
        };
        assert!(with(good).is_ok());
        for bad in [
            WasteFigures {
                attempts_per_landing: f64::NAN,
                ..good
            },
            WasteFigures {
                attempts_per_landing: 0.5,
                ..good
            },
            WasteFigures {
                wasted_usd: -0.01,
                ..good
            },
            WasteFigures {
                wasted_usd: f64::INFINITY,
                ..good
            },
            WasteFigures {
                first_attempt_share: 1.01,
                ..good
            },
            WasteFigures {
                first_attempt_share: -0.1,
                ..good
            },
        ] {
            assert!(with(bad).is_err(), "{bad:?} should be refused");
        }
    }

    #[test]
    fn a_throughput_review_has_a_path_a_schema_and_round_trips() {
        let review = throughput_review_with("20261026-090000-5abc");
        let record = AnyRecord::ThroughputReview(Box::new(review.clone()));
        assert_eq!(record.schema(), "athena.throughput-review/1");
        assert_eq!(
            record.canonical_path(),
            "records/throughput-reviews/2026-10/20261026-090000-5abc.json"
        );
        let json = record.to_json();
        let parsed = AnyRecord::parse(&record.canonical_path(), json.as_bytes()).unwrap();
        assert_eq!(parsed, record);
        let AnyRecord::ThroughputReview(parsed) = parsed else {
            panic!("a throughput review parses as one");
        };
        assert_eq!(*parsed, review);
        assert!(check_record_file(&record.canonical_path(), json.as_bytes()).is_ok());
        assert!(matches!(
            check_record_file(
                "records/code-health/2026-10/20261026-090000-5abc.json",
                json.as_bytes()
            ),
            Err(RecordError::Misplaced { .. })
        ));
        assert_eq!(
            ThroughputReview::latest(std::slice::from_ref(&review)),
            Some(&review)
        );
        assert!(ThroughputReview::latest(&[]).is_none());
    }

    #[test]
    fn a_throughput_review_with_bad_numbers_or_an_empty_proposal_is_refused() {
        let mut nan = throughput_review_with("20261026-090000-5abc");
        nan.window_median_minutes = f64::NAN;
        assert!(nan.validate().is_err());
        let mut negative = throughput_review_with("20261026-090000-5abc");
        negative.proposal.as_mut().unwrap().expected_minutes_saved = -1.0;
        assert!(negative.validate().is_err());
        let mut no_bottleneck = throughput_review_with("20261026-090000-5abc");
        no_bottleneck.proposal.as_mut().unwrap().bottleneck = "  ".into();
        assert!(no_bottleneck.validate().is_err());
        let mut bad_priority = throughput_review_with("20261026-090000-5abc");
        bad_priority.proposal.as_mut().unwrap().priority = "asap".into();
        assert!(bad_priority.validate().is_err());
        let mut no_jobs = throughput_review_with("20261026-090000-5abc");
        no_jobs.followups[0].jobs_measured = 0;
        assert!(no_jobs.validate().is_err());
        let record = AnyRecord::ThroughputReview(Box::new(nan));
        assert!(AnyRecord::parse("x", record.to_json().as_bytes()).is_err());
        let mut none = throughput_review_with("20261026-090000-5abc");
        none.proposal = None;
        none.followups.clear();
        assert!(none.validate().is_ok());
    }

    fn build_measurement_with(id: &str, sccache: SccacheFigures) -> BuildMeasurement {
        BuildMeasurement {
            id: RecordId::parse(id).unwrap(),
            at: ts(),
            commit: "0123456789abcdef0123456789abcdef01234567".into(),
            cargo_lock_hash: "89abcdef0123456789abcdef0123456789abcdef".into(),
            target_bytes: 4096,
            source_bytes: 1024,
            first_build_seconds: 61.5,
            second_build_seconds: 12.25,
            sccache,
            first_build_peak: None,
            second_build_peak: None,
        }
    }

    #[test]
    fn an_old_build_measurement_without_peaks_still_parses_with_none() {
        let old = r#"{
            "schema": "athena.build-measurement/1",
            "id": "20261029-090000-5abc",
            "at": "2026-10-29T09:00:00Z",
            "commit": "0123456789abcdef0123456789abcdef01234567",
            "cargo_lock_hash": "89abcdef0123456789abcdef0123456789abcdef",
            "target_bytes": 4096,
            "source_bytes": 1024,
            "first_build_seconds": 61.5,
            "second_build_seconds": 12.25,
            "sccache": "not_installed"
        }"#;
        let parsed = check_record_file(
            "records/build-measurements/2026-10/20261029-090000-5abc.json",
            old.as_bytes(),
        )
        .unwrap()
        .unwrap();
        let AnyRecord::BuildMeasurement(m) = parsed else {
            panic!("a build measurement parses as one");
        };
        assert_eq!(m.first_build_peak, None);
        assert_eq!(m.second_build_peak, None);
        assert_eq!(m.sccache, SccacheFigures::NotInstalled);
        // Records without peaks are written without the keys, so old files stay byte-for-byte as they were.
        assert!(!AnyRecord::BuildMeasurement(m).to_json().contains("peak"));
    }

    #[test]
    fn build_peaks_round_trip_and_bad_ones_are_refused() {
        let mut m = build_measurement_with("20261029-090000-5abc", SccacheFigures::NotInstalled);
        m.first_build_peak = Some(BuildPeak {
            memory_gb: 3.25,
            processes: 6,
        });
        m.second_build_peak = Some(BuildPeak {
            memory_gb: 2.5,
            processes: 4,
        });
        assert!(m.validate().is_ok());
        let record = AnyRecord::BuildMeasurement(Box::new(m.clone()));
        let parsed = check_record_file(&record.canonical_path(), record.to_json().as_bytes())
            .unwrap()
            .unwrap();
        assert_eq!(parsed, record);

        for bad_memory in [-1.0, f64::NAN, f64::INFINITY] {
            let mut bad = m.clone();
            bad.second_build_peak = Some(BuildPeak {
                memory_gb: bad_memory,
                processes: 4,
            });
            assert!(bad.validate().is_err(), "{bad_memory}");
        }
        let mut none = m.clone();
        none.first_build_peak = Some(BuildPeak {
            memory_gb: 1.0,
            processes: 0,
        });
        assert!(none.validate().is_err());
        // A negative value written into a file is refused when the file is read.
        let negative = record
            .to_json()
            .replace("\"memory_gb\": 3.25", "\"memory_gb\": -3.25");
        assert_ne!(negative, record.to_json());
        assert!(check_record_file(&record.canonical_path(), negative.as_bytes()).is_err());
    }

    #[test]
    fn a_build_measurement_has_a_path_a_schema_and_round_trips() {
        let measured = SccacheFigures::Measured {
            first: SccacheCounts { hits: 0, misses: 7 },
            second: SccacheCounts { hits: 5, misses: 2 },
        };
        for sccache in [measured, SccacheFigures::NotInstalled] {
            let measurement = build_measurement_with("20261029-090000-5abc", sccache);
            let record = AnyRecord::BuildMeasurement(Box::new(measurement.clone()));
            assert_eq!(record.schema(), "athena.build-measurement/1");
            assert_eq!(
                record.canonical_path(),
                "records/build-measurements/2026-10/20261029-090000-5abc.json"
            );
            let json = record.to_json();
            let parsed = check_record_file(&record.canonical_path(), json.as_bytes())
                .unwrap()
                .unwrap();
            assert_eq!(parsed, record);
            let AnyRecord::BuildMeasurement(parsed) = parsed else {
                panic!("a build measurement parses as one");
            };
            assert_eq!(*parsed, measurement);
            assert!(matches!(
                check_record_file(
                    "records/code-health/2026-10/20261029-090000-5abc.json",
                    json.as_bytes()
                ),
                Err(RecordError::Misplaced { .. })
            ));
        }
        let one = build_measurement_with("20261029-090000-5abc", SccacheFigures::NotInstalled);
        assert_eq!(
            BuildMeasurement::latest(std::slice::from_ref(&one)),
            Some(&one)
        );
        let mut newer =
            build_measurement_with("20261029-100000-6abc", SccacheFigures::NotInstalled);
        newer.at = one
            .at
            .checked_add(jiff::SignedDuration::from_secs(60))
            .unwrap();
        let both = vec![newer.clone(), one];
        assert_eq!(BuildMeasurement::latest(&both), Some(&newer));
        assert!(BuildMeasurement::latest(&[]).is_none());
    }

    #[test]
    fn a_build_measurement_with_bad_times_or_no_lock_hash_is_refused() {
        let ok = build_measurement_with("20261029-090000-5abc", SccacheFigures::NotInstalled);
        assert!(ok.validate().is_ok());
        let mut nan = ok.clone();
        nan.first_build_seconds = f64::NAN;
        assert!(nan.validate().is_err());
        let mut negative = ok.clone();
        negative.second_build_seconds = -1.0;
        assert!(negative.validate().is_err());
        let mut no_lock = ok.clone();
        no_lock.cargo_lock_hash = " ".into();
        assert!(no_lock.validate().is_err());
        let mut no_commit = ok;
        no_commit.commit.clear();
        assert!(no_commit.validate().is_err());
        // A NaN is written as null, so the file does not even parse; the rest parse and are refused as invalid.
        let record = AnyRecord::BuildMeasurement(Box::new(nan));
        assert!(check_record_file(&record.canonical_path(), record.to_json().as_bytes()).is_err());
        for bad in [negative, no_lock, no_commit] {
            let record = AnyRecord::BuildMeasurement(Box::new(bad));
            assert!(matches!(
                check_record_file(&record.canonical_path(), record.to_json().as_bytes()),
                Err(RecordError::Invalid { .. })
            ));
        }
    }

    #[test]
    fn the_latest_sweep_is_the_most_recent_by_time_whatever_the_order() {
        assert!(HestiaSweep::latest(&[]).is_none());
        let older = sweep_with("20261026-090000-5abc");
        let mut newer = sweep_with("20261026-100000-6abc");
        newer.at = older
            .at
            .checked_add(jiff::SignedDuration::from_secs(3600))
            .unwrap();
        let sweeps = vec![newer.clone(), older];
        assert_eq!(HestiaSweep::latest(&sweeps), Some(&newer));
    }

    #[test]
    fn a_hestia_sweep_with_inconsistent_counts_or_ids_is_refused() {
        let mut wrong_count = sweep_with("20261026-090000-5abc");
        wrong_count.checks[0].count = 2;
        assert!(wrong_count.validate().is_err());
        let mut duplicate = sweep_with("20261026-090000-5abc");
        duplicate.checks[1].check = "branches".into();
        assert!(duplicate.validate().is_err());
        let mut stray = sweep_with("20261026-090000-5abc");
        stray.findings[0].check = "unlisted".into();
        assert!(stray.validate().is_err());
        let mut no_evidence = sweep_with("20261026-090000-5abc");
        no_evidence.findings[0].evidence = " ".into();
        assert!(no_evidence.validate().is_err());
        let record = AnyRecord::HestiaSweep(Box::new(wrong_count));
        assert!(AnyRecord::parse("x", record.to_json().as_bytes()).is_err());
    }

    #[test]
    fn every_role_round_trips_through_its_name() {
        for role in [
            Role::Reader,
            Role::Producer,
            Role::Verifier,
            Role::Steward,
            Role::Audience,
            Role::Auditor,
            Role::Planner,
            Role::Critic,
        ] {
            assert_eq!(Role::parse(role.as_str()), Some(role));
        }
        assert_eq!(Role::Auditor.as_str(), "auditor");
        assert_eq!(Role::Planner.as_str(), "planner");
        assert_eq!(Role::Critic.as_str(), "critic");
    }

    fn sample_plan(round: u32) -> JobPlan {
        JobPlan {
            job: job(),
            round,
            at: ts(),
            invocation_seq: 1,
            approach: "Add the records.".into(),
            decisions: vec![PlanDecision {
                choice: "Reuse Finding".into(),
                reason: "One shape for every finding.".into(),
                rejected: vec![RejectedOption {
                    option: "A new finding type".into(),
                    why: "Two shapes to keep in step.".into(),
                }],
            }],
            files: vec![PlanFile {
                path: "src/lib.rs".into(),
                action: DeliverableAction::Modify,
                purpose: "Add the types.".into(),
                governance: false,
                exists: true,
            }],
            steps: vec!["Write the types.".into()],
            risks: vec![PlanRisk {
                risk: "Digest drift".into(),
                mitigation: "A recorded digest test.".into(),
            }],
            dependents: vec![],
            demonstrations: vec![Demonstration {
                criterion: 1,
                method: "test".into(),
                command: "cargo test".into(),
                observes: "all pass".into(),
                fails_without: Some("the new test fails".into()),
                seam: None,
            }],
            open_questions: vec![],
            producer_section_sha256: "c".repeat(64),
            resumed_from: None,
        }
    }

    #[test]
    fn plan_dependents_are_optional_in_old_records_and_round_trip_when_set() {
        // A record written before the field existed has no `dependents` key; it parses with none and
        // re-serializes to the same text.
        let old = AnyRecord::JobPlan(Box::new(sample_plan(1)));
        let old_json = old.to_json();
        assert!(
            !old_json.contains("dependents"),
            "an empty list is not written"
        );
        let AnyRecord::JobPlan(parsed) = AnyRecord::parse("x", old_json.as_bytes()).unwrap() else {
            panic!("a job plan");
        };
        assert!(parsed.dependents.is_empty());
        assert_eq!(
            AnyRecord::JobPlan(parsed).to_json(),
            old_json,
            "an old record re-serializes byte for byte"
        );
        let path = format!("{}/job-plan-1.json", job().dir());
        assert!(check_record_file(&path, old_json.as_bytes()).is_ok());

        // A record with dependents keeps every field.
        let mut plan = sample_plan(1);
        plan.dependents = vec![
            PlanDependent {
                name: "exit_code of a check".into(),
                dependent: "verify.rs judge: the seen-commands filter".into(),
                kind: DependentKind::Rule,
                test: Some("cargo test -p athena-core verify".into()),
                reason: None,
            },
            PlanDependent {
                name: "budget planner".into(),
                dependent: "docs/system-facts.md".into(),
                kind: DependentKind::HostSetting,
                test: None,
                reason: Some("the line is regenerated by a test".into()),
            },
        ];
        let with = AnyRecord::JobPlan(Box::new(plan));
        let json = with.to_json();
        assert!(json.contains("\"dependents\"") && json.contains("host_setting"));
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), with);
        assert!(check_record_file(&path, json.as_bytes()).is_ok());
    }

    #[test]
    fn a_demonstration_seam_is_optional_in_old_records_and_round_trips_when_set() {
        // A record written before the field existed has no `seam` key; it parses with none and re-serializes
        // to the same text.
        let old = AnyRecord::JobPlan(Box::new(sample_plan(1)));
        let old_json = old.to_json();
        assert!(!old_json.contains("seam"), "none is not written");
        let AnyRecord::JobPlan(parsed) = AnyRecord::parse("x", old_json.as_bytes()).unwrap() else {
            panic!("a job plan");
        };
        assert_eq!(parsed.demonstrations[0].seam, None);
        assert_eq!(
            AnyRecord::JobPlan(parsed).to_json(),
            old_json,
            "an old record re-serializes byte for byte"
        );
        let path = format!("{}/job-plan-1.json", job().dir());
        assert!(check_record_file(&path, old_json.as_bytes()).is_ok());

        // A record with a seam keeps it.
        let mut plan = sample_plan(1);
        plan.demonstrations[0].seam = Some("run_planned with a scripted planner; exists".into());
        let with = AnyRecord::JobPlan(Box::new(plan));
        let json = with.to_json();
        assert!(json.contains("\"seam\""));
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), with);
        assert!(check_record_file(&path, json.as_bytes()).is_ok());
    }

    #[test]
    fn a_plan_records_where_it_resumed_from_and_an_old_plan_without_it_still_parses() {
        let old = AnyRecord::JobPlan(Box::new(sample_plan(1)));
        let old_json = old.to_json();
        assert!(!old_json.contains("resumed_from"), "none is not written");
        // A record written before the field existed is the same text.
        assert_eq!(AnyRecord::parse("x", old_json.as_bytes()).unwrap(), old);

        let mut plan = sample_plan(1);
        plan.resumed_from = Some(job());
        let resumed = AnyRecord::JobPlan(Box::new(plan));
        let json = resumed.to_json();
        assert!(json.contains("resumed_from"));
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), resumed);
    }

    fn sample_critique(round: u32) -> Critique {
        Critique {
            job: job(),
            round,
            at: ts(),
            invocation_seq: Some(2),
            agent_verdict: Some(CritiqueVerdict::Accept),
            final_verdict: CritiqueVerdict::Accept,
            findings: vec![Finding {
                severity: Severity::Minor,
                summary: "Name is vague.".into(),
                evidence: "step 1".into(),
                location: None,
                source: FindingSource::Critic,
            }],
            request_reading_mismatch: None,
            summary: None,
        }
    }

    #[test]
    fn plans_and_critiques_belong_at_their_round_and_nowhere_else() {
        let plan = AnyRecord::JobPlan(Box::new(sample_plan(2)));
        let critique = AnyRecord::Critique(Box::new(sample_critique(2)));
        assert_eq!(plan.schema(), "athena.job-plan/1");
        assert_eq!(critique.schema(), "athena.critique/1");
        let dir = job().dir();
        for (record, name) in [(&plan, "job-plan"), (&critique, "critique")] {
            let json = record.to_json();
            assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), *record);
            let right = format!("{dir}/{name}-2.json");
            assert_eq!(record.canonical_path(), right);
            assert!(check_record_file(&right, json.as_bytes()).is_ok());
            let wrong = format!("{dir}/{name}-1.json");
            assert!(matches!(
                check_record_file(&wrong, json.as_bytes()),
                Err(RecordError::Misplaced { .. })
            ));
        }
    }

    fn sample_baseline() -> Baseline {
        let check = |name: &str| CheckRun {
            name: name.into(),
            command: vec!["true".into()],
            exit_code: Some(1),
            passed: false,
            tests_ran: None,
            duration: Quantity::measured(0.5, crate::Unit::Seconds, "test"),
            output_tail: "boom".into(),
            note: None,
            failed_tests: Vec::new(),
            reused: false,
            revert_for: Vec::new(),
            queued_s: None,
        };
        Baseline {
            job: job(),
            base_commit: "a".repeat(40),
            before_invocation: 1,
            checks: vec![check("fmt"), check("test")],
            note: None,
            reused_from: None,
        }
    }

    #[test]
    fn a_check_written_before_queued_s_existed_reads_with_none_and_writes_without_the_key() {
        let check = sample_baseline().checks.remove(0);
        let json = serde_json::to_value(&check).unwrap();
        assert!(json.get("queued_s").is_none(), "{json}");
        let old: CheckRun = serde_json::from_value(json).unwrap();
        assert_eq!(old.queued_s, None);
        assert_eq!(old, check);
    }

    #[test]
    fn a_checks_wait_for_a_build_slot_round_trips() {
        let mut check = sample_baseline().checks.remove(0);
        check.queued_s = Some(12.5);
        let json = serde_json::to_string(&check).unwrap();
        assert!(json.contains("\"queued_s\":12.5"), "{json}");
        let back: CheckRun = serde_json::from_str(&json).unwrap();
        assert_eq!(back.queued_s, Some(12.5));
        let mut baseline = sample_baseline();
        baseline.checks = vec![check];
        let record = AnyRecord::Baseline(Box::new(baseline));
        let text = record.to_json();
        assert_eq!(AnyRecord::parse("x", text.as_bytes()).unwrap(), record);
    }

    #[test]
    fn a_baseline_written_before_reuse_existed_still_parses_with_no_source() {
        let record = AnyRecord::Baseline(Box::new(sample_baseline()));
        let json = record.to_json();
        assert!(
            !json.contains("reused_from"),
            "a baseline that ran here serialises as before: {json}"
        );
        let path = format!("{}/baseline.json", job().dir());
        assert!(check_record_file(&path, json.as_bytes()).is_ok());
        let AnyRecord::Baseline(b) = AnyRecord::parse("x", json.as_bytes()).unwrap() else {
            panic!("a baseline")
        };
        assert_eq!(b.reused_from, None);
        assert_eq!(b.validate(), Ok(()));
    }

    #[test]
    fn a_reused_baseline_round_trips_and_validates() {
        let mut b = sample_baseline();
        for c in &mut b.checks {
            c.passed = true;
            c.exit_code = None;
            c.reused = true;
        }
        b.reused_from = Some("https://github.com/owner/repo/actions/runs/4242".into());
        let record = AnyRecord::Baseline(Box::new(b));
        let json = record.to_json();
        assert!(json.contains("reused_from"), "{json}");
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), record);
        let path = format!("{}/baseline.json", job().dir());
        assert!(check_record_file(&path, json.as_bytes()).is_ok());
        let AnyRecord::Baseline(back) = AnyRecord::parse("x", json.as_bytes()).unwrap() else {
            panic!("a baseline")
        };
        assert_eq!(
            back.reused_from.as_deref(),
            Some("https://github.com/owner/repo/actions/runs/4242")
        );
        assert!(
            !back.failed_before("fmt"),
            "a reused baseline excuses nothing"
        );
    }

    #[test]
    fn a_baseline_with_an_unknown_key_is_still_refused() {
        let json = AnyRecord::Baseline(Box::new(sample_baseline())).to_json();
        let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
        value["reused_form"] = serde_json::json!("typo");
        let path = format!("{}/baseline.json", job().dir());
        assert!(check_record_file(&path, value.to_string().as_bytes()).is_err());
    }

    #[test]
    fn a_check_run_written_before_reuse_was_marked_loads_as_not_reused() {
        let old = sample_baseline().checks[0].clone();
        let json = serde_json::to_string(&old).unwrap();
        assert!(
            !json.contains("reused"),
            "an unreused record serialises as before: {json}"
        );
        let parsed: CheckRun = serde_json::from_str(&json).expect("the old shape loads");
        assert!(!parsed.reused);
        assert_eq!(parsed, old);
    }

    #[test]
    fn a_reused_check_record_round_trips_and_validates() {
        let mut b = sample_baseline();
        b.checks[1].reused = true;
        b.checks[1].passed = true;
        b.checks[1].exit_code = None;
        b.checks[1].note = Some("result reused from the CI run".into());
        let record = AnyRecord::Baseline(Box::new(b));
        let json = record.to_json();
        assert!(json.contains("\"reused\": true") || json.contains("\"reused\":true"));
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), record);
        let path = format!("{}/baseline.json", job().dir());
        assert!(check_record_file(&path, json.as_bytes()).is_ok());
        let AnyRecord::Baseline(back) = AnyRecord::parse("x", json.as_bytes()).unwrap() else {
            panic!("a baseline")
        };
        assert!(!back.checks[0].reused && back.checks[1].reused);
    }

    #[test]
    fn a_check_run_written_before_revert_runs_were_marked_loads_with_no_marker() {
        let old = sample_baseline().checks[0].clone();
        let json = serde_json::to_string(&old).unwrap();
        assert!(
            !json.contains("revert_for"),
            "an ordinary record serialises as before: {json}"
        );
        let parsed: CheckRun = serde_json::from_str(&json).expect("the old shape loads");
        assert!(parsed.revert_for.is_empty());
        assert_eq!(parsed, old);
    }

    #[test]
    fn a_revert_check_record_round_trips_and_validates() {
        let mut b = sample_baseline();
        b.checks[1].name = "fails-without-1".into();
        b.checks[1].passed = true;
        b.checks[1].exit_code = Some(101);
        b.checks[1].revert_for = vec![1, 2];
        let record = AnyRecord::Baseline(Box::new(b));
        let json = record.to_json();
        assert!(json.contains("revert_for"), "{json}");
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), record);
        let path = format!("{}/baseline.json", job().dir());
        assert!(check_record_file(&path, json.as_bytes()).is_ok());
        let AnyRecord::Baseline(back) = AnyRecord::parse("x", json.as_bytes()).unwrap() else {
            panic!("a baseline")
        };
        assert!(back.checks[0].revert_for.is_empty());
        assert_eq!(back.checks[1].revert_for, [1, 2]);
    }

    #[test]
    fn a_baseline_written_before_failed_tests_existed_still_parses_and_validates() {
        let record = AnyRecord::Baseline(Box::new(sample_baseline()));
        let json = record.to_json();
        assert!(
            !json.contains("failed_tests"),
            "a record without names serialises as before: {json}"
        );
        let path = format!("{}/baseline.json", job().dir());
        assert!(check_record_file(&path, json.as_bytes()).is_ok());
        let parsed = AnyRecord::parse("x", json.as_bytes()).expect("the old shape parses");
        let AnyRecord::Baseline(b) = parsed else {
            panic!("a baseline")
        };
        assert!(b.checks.iter().all(|c| c.failed_tests.is_empty()));
        assert_eq!(b.validate(), Ok(()));
    }

    #[test]
    fn failing_test_names_round_trip_and_are_read_per_check() {
        let mut b = sample_baseline();
        b.checks[1].failed_tests = vec!["lib a::x".into(), "lib b::y".into()];
        let record = AnyRecord::Baseline(Box::new(b.clone()));
        let json = record.to_json();
        assert!(json.contains("failed_tests"));
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), record);
        let path = format!("{}/baseline.json", job().dir());
        assert!(check_record_file(&path, json.as_bytes()).is_ok());
        assert_eq!(b.failed_tests_before("test"), ["lib a::x", "lib b::y"]);
        assert!(b.failed_tests_before("fmt").is_empty(), "no names recorded");
        assert!(b.failed_tests_before("clippy").is_empty(), "no entry");
        b.checks[1].passed = true;
        assert!(b.failed_tests_before("test").is_empty(), "it passed");
        b.checks[1].passed = false;
        b.note = Some("the baseline could not run".into());
        assert!(b.failed_tests_before("test").is_empty(), "a noted baseline");
    }

    #[test]
    fn only_a_noteless_baseline_with_a_failed_entry_excuses_a_check() {
        let b = sample_baseline();
        assert!(b.failed_before("fmt") && b.failed_before("test"));
        assert!(!b.failed_before("clippy"), "no entry");
        let mut passed = sample_baseline();
        passed.checks[0].passed = true;
        assert!(!passed.failed_before("fmt"), "it passed at the base");
        let mut empty = sample_baseline();
        empty.checks.clear();
        assert!(
            !empty.failed_before("fmt"),
            "an empty baseline is not success or failure"
        );
        let mut noted = sample_baseline();
        noted.note = Some("the baseline could not run: no snapshot".into());
        assert!(
            !noted.failed_before("fmt"),
            "a noted baseline excuses nothing"
        );
    }

    #[test]
    fn a_baseline_belongs_at_baseline_json_and_nowhere_else() {
        let record = AnyRecord::Baseline(Box::new(sample_baseline()));
        assert_eq!(record.schema(), "athena.baseline/1");
        let json = record.to_json();
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), record);
        let right = format!("{}/baseline.json", job().dir());
        assert_eq!(record.canonical_path(), right);
        assert!(check_record_file(&right, json.as_bytes()).is_ok());
        let wrong = format!("{}/baseline-1.json", job().dir());
        assert!(matches!(
            check_record_file(&wrong, json.as_bytes()),
            Err(RecordError::Misplaced { .. })
        ));
    }

    #[test]
    fn a_baseline_without_a_base_or_with_a_repeated_check_is_refused() {
        let mut bad = sample_baseline();
        bad.checks[1].name = "fmt".into();
        let mut empty = sample_baseline();
        empty.base_commit = " ".into();
        for b in [bad, empty] {
            let record = AnyRecord::Baseline(Box::new(b));
            assert!(matches!(
                AnyRecord::parse("x", record.to_json().as_bytes()),
                Err(RecordError::Invalid { .. })
            ));
        }
    }

    fn sample_replay() -> Replay {
        Replay {
            job: job(),
            base_commit: "a".repeat(40),
            candidate_commit: "b".repeat(40),
            window: 50,
            total: 10,
            newly_refused: vec![
                ReplayRefusal {
                    job: JobId::parse("20261001-000000-aaaa").unwrap(),
                    reason: "refused: criterion 1".into(),
                },
                ReplayRefusal {
                    job: JobId::parse("20261001-000001-bbbb").unwrap(),
                    reason: "refused: criterion 2".into(),
                },
            ],
            held: true,
            intended: false,
            no_replay: None,
        }
    }

    #[test]
    fn a_replay_round_trips_and_belongs_at_replay_json_and_nowhere_else() {
        let record = AnyRecord::Replay(Box::new(sample_replay()));
        assert_eq!(record.schema(), "athena.replay/1");
        let json = record.to_json();
        assert_eq!(AnyRecord::parse("x", json.as_bytes()).unwrap(), record);
        let right = format!("{}/replay.json", job().dir());
        assert_eq!(record.canonical_path(), right);
        assert_eq!(
            check_record_file(&right, json.as_bytes()).unwrap(),
            Some(record.clone())
        );
        let wrong = format!("{}/replay-1.json", job().dir());
        assert!(matches!(
            check_record_file(&wrong, json.as_bytes()),
            Err(RecordError::Misplaced { .. })
        ));
        // A replay that did not run is a valid record too.
        let mut none = sample_replay();
        none.total = 0;
        none.newly_refused.clear();
        none.held = false;
        none.no_replay = Some("no replay: no accepted readings in the history".into());
        let record = AnyRecord::Replay(Box::new(none));
        assert_eq!(
            AnyRecord::parse("x", record.to_json().as_bytes()).unwrap(),
            record
        );
    }

    #[test]
    fn an_inconsistent_replay_is_refused() {
        let mut too_many = sample_replay();
        too_many.total = 1;
        let mut held_empty = sample_replay();
        held_empty.newly_refused.clear();
        let mut absent_with_list = sample_replay();
        absent_with_list.no_replay = Some("no replay: x".into());
        let mut held_but_intended = sample_replay();
        held_but_intended.intended = true;
        let mut no_window = sample_replay();
        no_window.window = 0;
        let mut blank_reason = sample_replay();
        blank_reason.held = false;
        blank_reason.newly_refused.clear();
        blank_reason.no_replay = Some(" ".into());
        for r in [
            too_many,
            held_empty,
            absent_with_list,
            held_but_intended,
            no_window,
            blank_reason,
        ] {
            let record = AnyRecord::Replay(Box::new(r));
            assert!(matches!(
                AnyRecord::parse("x", record.to_json().as_bytes()),
                Err(RecordError::Invalid { .. })
            ));
        }
    }

    #[test]
    fn a_round_zero_plan_or_critique_is_refused() {
        let plan = AnyRecord::JobPlan(Box::new(sample_plan(0)));
        let critique = AnyRecord::Critique(Box::new(sample_critique(0)));
        for record in [plan, critique] {
            assert!(matches!(
                AnyRecord::parse("x", record.to_json().as_bytes()),
                Err(RecordError::Invalid { .. })
            ));
        }
    }

    #[test]
    fn an_accepting_critique_with_a_blocker_or_major_finding_is_refused() {
        for severity in [Severity::Blocker, Severity::Major] {
            let mut bad = sample_critique(1);
            bad.findings[0].severity = severity;
            let record = AnyRecord::Critique(Box::new(bad.clone()));
            assert!(matches!(
                AnyRecord::parse("x", record.to_json().as_bytes()),
                Err(RecordError::Invalid { .. })
            ));
            bad.final_verdict = CritiqueVerdict::Revise;
            assert!(bad.validate().is_ok());
        }
    }

    #[test]
    fn a_plan_with_a_bad_section_hash_or_criterion_number_is_refused() {
        let mut plan = sample_plan(1);
        assert_eq!(plan.validate(), Ok(()));
        plan.producer_section_sha256 = "abc".into();
        assert!(plan.validate().is_err());
        let mut plan = sample_plan(1);
        plan.demonstrations[0].criterion = 0;
        assert!(plan.validate().is_err());
    }

    #[test]
    fn a_work_item_that_is_not_a_blind_spot_cites_evidence() {
        let mut item = sample_work_item();
        assert_eq!(item.validate(), Ok(()));
        item.kind = WorkItemKind::Proposal;
        assert!(item.validate().is_err());
        item.evidence.push(Reference::job(&job()));
        assert_eq!(item.validate(), Ok(()));
        item.source.value = " ".into();
        assert!(item.validate().is_err());
    }

    #[test]
    fn text_cites_a_job_only_by_its_whole_id() {
        let id = JobId::parse("20261027-054211-a2n5").unwrap();
        for cited in [
            "20261027-054211-a2n5",
            "the refusal rule from job 20261027-054211-a2n5 stopped three drains",
            "(20261027-054211-a2n5).",
            "first 20261027-054211-a2n5\nsecond line",
        ] {
            assert!(text_cites_job(cited, &id), "{cited}");
        }
        for not_cited in [
            "",
            "20261027-054211-a2n",
            "120261027-054211-a2n5",
            "20261027-054211-a2n5-extra",
            "20261027-054211-a2n5x",
            "a2n5",
        ] {
            assert!(!text_cites_job(not_cited, &id), "{not_cited}");
        }
        // A later whole-token match counts even when an earlier match is part of a longer token.
        assert!(text_cites_job(
            "20261027-054211-a2n5x then 20261027-054211-a2n5",
            &id
        ));
    }

    #[test]
    fn rescore_names_its_cause() {
        let cause = RecordId::parse("20261011-090000-abcd").unwrap();
        let score = AnyRecord::Score(Box::new(Score {
            job: job(),
            at: ts(),
            rules: "v1".into(),
            cause: Some(cause),
            events: vec![],
            deliverables: None,
            quantities: vec![],
            models: None,
        }));
        assert!(
            score
                .canonical_path()
                .ends_with("/score-20261011-090000-abcd.json")
        );
    }

    #[test]
    fn an_outcome_of_a_request_job_records_its_totals() {
        // M5 (blocks): an outcome without totals would skip reconciliation. (allows): the bootstrap's.
        let outcome = |job: JobId| Outcome {
            job,
            state: OutcomeState::Rejected,
            reason: "the final verdict is fail".into(),
            at: ts(),
            totals: None,
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        };
        let text = AnyRecord::Outcome(Box::new(outcome(job()))).to_json();
        assert!(matches!(
            AnyRecord::parse("x", text.as_bytes()),
            Err(RecordError::Invalid { .. })
        ));
        let mut closed = outcome(JobId::Bootstrap);
        closed.state = OutcomeState::Closed;
        let text = AnyRecord::Outcome(Box::new(closed)).to_json();
        assert!(AnyRecord::parse("x", text.as_bytes()).is_ok());
    }

    #[test]
    fn pending_bootstrap_cost_is_representable() {
        let job = Job {
            id: JobId::Bootstrap,
            kind: JobKind::Bootstrap,
            title: "Bootstrap".into(),
            class_of_work: None,
            capability: None,
            accepted_reading: None,
            accepted_by: None,
            owner: Owner {
                role: "automation engineer".into(),
                capability: None,
                model: None,
                note: None,
            },
            verifier_model: None,
            caps: None,
            base_commit: None,
            config_sha256: None,
            defaults_sha256: None,
            athena: None,
            driver: None,
            brief_sha256: None,
            dispatched_at: None,
            request_path: Some("records/jobs/bootstrap/brief.md".into()),
            request_sha256: None,
            cost: Some(Quantity::pending(
                Unit::Usd,
                "resolved later from measured usage",
            )),
            tokens: Some(Quantity::pending(
                Unit::Tokens,
                "resolved later from measured usage",
            )),
            note: None,
            pin: None,
            dispatch_branch: None,
            allowed_paths: None,
        };
        let record = AnyRecord::Job(Box::new(job));
        assert_eq!(record.canonical_path(), "records/jobs/bootstrap/job.json");
        let back = AnyRecord::parse("x", record.to_json().as_bytes()).unwrap();
        assert_eq!(back, record);
    }

    /// A bootstrap job record with the given dispatch branch.
    fn job_on(branch: Option<&str>) -> AnyRecord {
        let AnyRecord::Job(mut job) = AnyRecord::parse(
            "x",
            br#"{"schema": "athena.job/1", "id": "bootstrap", "kind": "bootstrap", "title": "Bootstrap",
                "class_of_work": null, "capability": null, "accepted_reading": null, "accepted_by": null,
                "owner": {"role": "automation engineer", "capability": null, "model": null, "note": null},
                "verifier_model": null, "caps": null, "base_commit": null, "config_sha256": null,
                "defaults_sha256": null, "athena": null, "driver": null, "brief_sha256": null,
                "dispatched_at": null, "request_path": null, "request_sha256": null, "cost": null,
                "tokens": null, "note": null}"#,
        )
        .unwrap() else {
            panic!("a job record")
        };
        job.dispatch_branch = branch.map(str::to_string);
        AnyRecord::Job(job)
    }

    #[test]
    fn a_job_keeps_its_dispatch_branch_through_a_round_trip() {
        // A named branch and the empty string (a detached HEAD at dispatch) both survive, and the
        // records check accepts the file.
        for branch in ["main", ""] {
            let record = job_on(Some(branch));
            let text = record.to_json();
            assert!(text.contains("\"dispatch_branch\""), "{text}");
            let back = AnyRecord::parse("x", text.as_bytes()).unwrap();
            assert_eq!(back, record);
            let AnyRecord::Job(job) = &back else {
                panic!("a job record")
            };
            assert_eq!(job.dispatch_branch.as_deref(), Some(branch));
            let checked = check_record_file(&record.canonical_path(), text.as_bytes())
                .unwrap()
                .expect("a record");
            assert_eq!(checked, record);
        }
    }

    #[test]
    fn a_job_record_made_before_the_dispatch_branch_existed_still_parses() {
        let record = job_on(None);
        let text = record.to_json();
        assert!(!text.contains("dispatch_branch"), "{text}");
        let AnyRecord::Job(job) = AnyRecord::parse("x", text.as_bytes()).unwrap() else {
            panic!("a job record")
        };
        assert_eq!(job.dispatch_branch, None);
        let checked = check_record_file(&record.canonical_path(), text.as_bytes())
            .unwrap()
            .expect("a record");
        assert_eq!(checked, record);
    }

    /// A bootstrap job record holding the given allowed paths.
    fn job_limited_to(paths: Option<&[&str]>) -> AnyRecord {
        let AnyRecord::Job(mut job) = job_on(None) else {
            panic!("a job record")
        };
        job.allowed_paths = paths.map(|p| p.iter().map(|s| s.to_string()).collect());
        AnyRecord::Job(job)
    }

    #[test]
    fn a_job_keeps_its_allowed_paths_through_a_round_trip() {
        let record = job_limited_to(Some(&["crates/a.rs", "README.md", "docs/"]));
        let text = record.to_json();
        assert!(text.contains("\"allowed_paths\""), "{text}");
        let back = AnyRecord::parse("x", text.as_bytes()).unwrap();
        assert_eq!(back, record);
        let AnyRecord::Job(job) = &back else {
            panic!("a job record")
        };
        assert_eq!(
            job.allowed_paths.as_deref(),
            Some(
                &[
                    "crates/a.rs".to_string(),
                    "README.md".to_string(),
                    "docs/".to_string()
                ][..]
            )
        );
        let checked = check_record_file(&record.canonical_path(), text.as_bytes())
            .unwrap()
            .expect("a record");
        assert_eq!(checked, record);
    }

    #[test]
    fn a_job_record_made_before_allowed_paths_existed_still_parses() {
        let record = job_limited_to(None);
        let text = record.to_json();
        assert!(!text.contains("allowed_paths"), "{text}");
        let checked = check_record_file(&record.canonical_path(), text.as_bytes())
            .unwrap()
            .expect("a record");
        let AnyRecord::Job(job) = &checked else {
            panic!("a job record")
        };
        assert_eq!(job.allowed_paths, None);
        assert_eq!(checked, record);
        assert!(!checked.to_json().contains("allowed_paths"));
    }
}
