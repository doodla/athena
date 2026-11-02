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

/// The phase of work a host call or a decision belongs to (`docs/design/decision-sets.md` section 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkPhase {
    Reading,
    Producing,
    Verifying,
    Landing,
}

/// The host operation that was refused: a closed list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HostOperation {
    /// Asking the host to merge a pull request.
    Merge,
    /// Opening a pull request.
    CreatePr,
    /// Reading the repository (its default branch and allowed merge methods).
    ReadRepo,
    /// Reading a pull request.
    ReadPr,
    /// Reading the checks of a commit.
    ReadChecks,
}

impl HostOperation {
    pub const ALL: [HostOperation; 5] = [
        HostOperation::Merge,
        HostOperation::CreatePr,
        HostOperation::ReadRepo,
        HostOperation::ReadPr,
        HostOperation::ReadChecks,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            HostOperation::Merge => "merge",
            HostOperation::CreatePr => "create-pr",
            HostOperation::ReadRepo => "read-repo",
            HostOperation::ReadPr => "read-pr",
            HostOperation::ReadChecks => "read-checks",
        }
    }
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|o| o.as_str() == text)
    }
}

/// A short class of a refusal, chosen by code from the status code alone: a closed list. The host's own text
/// is never stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MessageClass {
    /// A 5xx answer.
    ServerError,
    /// 405: the host does not allow this method here.
    MethodNotAllowed,
    /// 422: the host understood the request and refused its content.
    Unprocessable,
    /// 404.
    NotFound,
    /// 401 or 403.
    NotAuthorized,
    /// 429.
    RateLimited,
    /// Any other HTTP answer.
    OtherStatus,
    /// The call failed before an HTTP answer.
    NoAnswer,
}

impl MessageClass {
    pub const ALL: [MessageClass; 8] = [
        MessageClass::ServerError,
        MessageClass::MethodNotAllowed,
        MessageClass::Unprocessable,
        MessageClass::NotFound,
        MessageClass::NotAuthorized,
        MessageClass::RateLimited,
        MessageClass::OtherStatus,
        MessageClass::NoAnswer,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            MessageClass::ServerError => "server-error",
            MessageClass::MethodNotAllowed => "method-not-allowed",
            MessageClass::Unprocessable => "unprocessable",
            MessageClass::NotFound => "not-found",
            MessageClass::NotAuthorized => "not-authorized",
            MessageClass::RateLimited => "rate-limited",
            MessageClass::OtherStatus => "other-status",
            MessageClass::NoAnswer => "no-answer",
        }
    }
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.as_str() == text)
    }
    /// The class of an answer with this status code, or of no answer.
    pub fn of_status(status: Option<u16>) -> Self {
        match status {
            None => MessageClass::NoAnswer,
            Some(500..=599) => MessageClass::ServerError,
            Some(405) => MessageClass::MethodNotAllowed,
            Some(422) => MessageClass::Unprocessable,
            Some(404) => MessageClass::NotFound,
            Some(401 | 403) => MessageClass::NotAuthorized,
            Some(429) => MessageClass::RateLimited,
            Some(_) => MessageClass::OtherStatus,
        }
    }
}

/// `athena.host-error/1`: a call to the code host was refused. The host's text is not stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostError {
    pub id: RecordId,
    pub at: Timestamp,
    /// The HTTP status, or absent when the call failed before an HTTP answer.
    pub status_code: Option<u16>,
    pub operation: HostOperation,
    /// The job the call was made for.
    pub job: JobId,
    pub phase: WorkPhase,
    pub message_class: MessageClass,
    /// True when the client did not observe this refusal: the record was derived by code from the text of a
    /// past intervention (`source_intervention`). Absent (false) for a refusal the client observed. A derived
    /// record is never presented as observed.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub derived: bool,
    /// The intervention a derived record was derived from. Present exactly when `derived` is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_intervention: Option<RecordId>,
    /// True when the host refused a merge while the required checks were still pending: a wait, not a refusal
    /// of the merge method. Absent (false) otherwise, so a record written before this field existed reads and
    /// writes back unchanged.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub checks_pending: bool,
}

/// The state a decision is in beside its `status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DecisionStateKind {
    /// A signal of the decision fired; only the user clears it.
    UnderReview,
    /// The user cleared the review. Nothing writes this yet: clearing is a later build.
    Cleared,
}

impl DecisionStateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DecisionStateKind::UnderReview => "under-review",
            DecisionStateKind::Cleared => "cleared",
        }
    }
}

/// `athena.decision-state/1`: one change of a decision's state. Append-only: the current state is the fold of
/// all of them ([`fold_decision_states`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionState {
    pub id: RecordId,
    pub at: Timestamp,
    /// The decision's id, such as `D87`.
    pub decision: String,
    pub state: DecisionStateKind,
    /// The name of the signal that fired, for an `under-review` change.
    pub signal: Option<String>,
    /// The records that fired it (for a host-error signal, `athena.host-error/1` ids).
    pub firing_records: Vec<RecordId>,
    /// The job that decided a clearing, for a `cleared` change.
    pub decision_job: Option<JobId>,
}

/// The current state of each decision that has any state record: the latest by `(at, id)` wins.
pub fn fold_decision_states(states: &[DecisionState]) -> BTreeMap<String, DecisionStateKind> {
    let mut ordered: Vec<&DecisionState> = states.iter().collect();
    ordered.sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
    ordered
        .into_iter()
        .map(|s| (s.decision.clone(), s.state))
        .collect()
}

/// A signal's source: one of the five of `docs/design/decision-sets.md` section 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignalSource {
    Rejection,
    Intervention,
    Correction,
    CheckFailure,
    HostError,
}

/// How strongly one event of a signal counts: a hard signal needs one job, a soft one repeats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignalStrength {
    Hard,
    Soft,
}

/// One watched condition of a default, in `athena.default-expectations/1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectationSignal {
    pub source: SignalSource,
    pub strength: SignalStrength,
    /// A one-line description.
    pub text: String,
    /// Distinct jobs needed; at least 1, and at least 2 for a soft signal.
    pub threshold_jobs: u32,
}

/// `athena.default-expectations/1`: what a default in `defaults.toml` assumes and how to tell it failed.
/// `defaults.toml` itself is not changed (D26).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DefaultExpectations {
    pub id: RecordId,
    pub at: Timestamp,
    /// The default's id in `defaults.toml`.
    pub default_id: String,
    pub assumptions: Vec<String>,
    pub signals: Vec<ExpectationSignal>,
    /// The `version` of `defaults.toml` this was written against.
    pub defaults_version: u32,
    /// The defaults proposal it came from, if any.
    pub proposal: Option<String>,
}

impl DefaultExpectations {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if self.default_id.trim().is_empty() {
            return Err("default-expectations names no default_id".into());
        }
        if self.defaults_version == 0 {
            return Err(
                "default-expectations records the defaults version it was written against".into(),
            );
        }
        for signal in &self.signals {
            let least = match signal.strength {
                SignalStrength::Hard => 1,
                SignalStrength::Soft => 2,
            };
            if signal.threshold_jobs < least {
                return Err(format!(
                    "a {:?} signal needs a threshold of at least {least} job(s)",
                    signal.strength
                ));
            }
        }
        Ok(())
    }
}

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

/// What kind of prevention would have stopped the class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreventionKind {
    Default,
    CodeCheck,
    DesignChange,
}

impl PreventionKind {
    pub const ALL: [PreventionKind; 3] = [
        PreventionKind::Default,
        PreventionKind::CodeCheck,
        PreventionKind::DesignChange,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            PreventionKind::Default => "default",
            PreventionKind::CodeCheck => "code-check",
            PreventionKind::DesignChange => "design-change",
        }
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
}

impl WorkItem {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
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

/// The schema tag of a throughput review.
pub const THROUGHPUT_REVIEW_SCHEMA: &str = "athena.throughput-review/1";

/// Why a throughput review ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThroughputTrigger {
    /// Ten more landings since the previous review (or the first ten ever).
    Cadence,
    /// The median minutes per landed job grew by more than 20% against the previous review.
    Worsened,
}

impl ThroughputTrigger {
    pub fn as_str(self) -> &'static str {
        match self {
            ThroughputTrigger::Cadence => "cadence",
            ThroughputTrigger::Worsened => "worsened",
        }
    }
}

/// The top proposal of a throughput review: the biggest wait and the change that removes it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThroughputProposal {
    /// The biggest bottleneck, in one or two sentences.
    pub bottleneck: String,
    /// The stage it sits in (reading, planning, producer, checks, verifier, landing or idle).
    pub stage: String,
    /// The architectural change that removes it.
    pub change: String,
    /// The prediction: minutes saved per landed job once the change lands.
    pub expected_minutes_saved: f64,
    /// The request priority the prediction set (`urgent`, `high`, `normal` or `low`).
    pub priority: String,
    /// The design request this review queued; None when an earlier one was still open.
    pub request_job: Option<JobId>,
}

/// What a later review measured for a change an earlier review cited: the prediction set against the minutes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThroughputFollowup {
    /// The review whose prediction this closes.
    pub review: RecordId,
    /// The landed job that cited the review.
    pub change_job: JobId,
    /// Minutes saved per landed job that review predicted.
    pub predicted_minutes_saved: f64,
    /// The median minutes per landed job when that review ran.
    pub baseline_median_minutes: f64,
    /// The median minutes per landed job over the jobs that landed after the change.
    pub measured_median_minutes: f64,
    /// Baseline minus measured: positive is faster.
    pub measured_minutes_saved: f64,
    /// How many landed jobs the measured median covers.
    pub jobs_measured: u32,
}

/// `athena.throughput-review/1`: one throughput review. `landings` and `window_median_minutes` are the
/// cadence point the next review counts from; the proposal is the prediction; `followups` close earlier
/// predictions. Reviews are never edited; the latest is the newest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThroughputReview {
    pub id: RecordId,
    pub at: Timestamp,
    /// Landed request jobs counted when the review ran.
    pub landings: u32,
    /// The median minutes per landed job over the newest landed jobs when the review ran.
    pub window_median_minutes: f64,
    pub trigger: ThroughputTrigger,
    /// None when the model gave no usable proposal or an earlier request was still open.
    pub proposal: Option<ThroughputProposal>,
    /// Predictions this review closed.
    pub followups: Vec<ThroughputFollowup>,
    /// Why there is no proposal or no request, or any other remark; empty when there is nothing to say.
    pub note: String,
}

impl ThroughputReview {
    /// The most recent of `reviews`, by time and then id; None when there are none.
    pub fn latest(reviews: &[ThroughputReview]) -> Option<&ThroughputReview> {
        reviews
            .iter()
            .max_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)))
    }

    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        let good = |v: f64| v.is_finite() && v >= 0.0;
        if !good(self.window_median_minutes) {
            return Err("a throughput review's median is a finite, non-negative number".into());
        }
        if let Some(p) = &self.proposal {
            if p.bottleneck.trim().is_empty() || p.change.trim().is_empty() {
                return Err("a throughput proposal names its bottleneck and its change".into());
            }
            if !good(p.expected_minutes_saved) {
                return Err("expected minutes saved is a finite, non-negative number".into());
            }
            if !PRIORITY_LEVELS.contains(&p.priority.as_str()) {
                return Err(format!(
                    "priority {:?} is not one of {}",
                    p.priority,
                    PRIORITY_LEVELS.join(", ")
                ));
            }
        }
        for f in &self.followups {
            if !good(f.predicted_minutes_saved)
                || !good(f.baseline_median_minutes)
                || !good(f.measured_median_minutes)
                || !f.measured_minutes_saved.is_finite()
            {
                return Err("a throughput follow-up holds finite minutes".into());
            }
            if f.jobs_measured == 0 {
                return Err("a throughput follow-up measures at least one landed job".into());
            }
        }
        Ok(())
    }
}

/// The schema tag of a build measurement.
pub const BUILD_MEASUREMENT_SCHEMA: &str = "athena.build-measurement/1";

/// What `sccache` counted during one build: compile requests it answered from its cache and those it had
/// to compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SccacheCounts {
    pub hits: u64,
    pub misses: u64,
}

/// The compile-cache figures of a build measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SccacheFigures {
    /// `sccache` was not on the PATH, so both builds compiled everything themselves. This is not a count of
    /// zero hits.
    NotInstalled,
    /// The counts of each build; the second build running after the first shows whether the cache answers
    /// across worktrees.
    Measured {
        first: SccacheCounts,
        second: SccacheCounts,
    },
}

/// `athena.build-measurement/1`: the size and time of a cold workspace build, measured by
/// `athena measure-build` in two separate worktrees with different target directories. Measurements are
/// never edited; the latest is the newest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildMeasurement {
    pub id: RecordId,
    pub at: Timestamp,
    /// The commit whose tree was built.
    pub commit: String,
    /// The git blob id of `Cargo.lock` at that commit; a new measurement is due when it changes.
    pub cargo_lock_hash: String,
    /// T: bytes in one cold `target/` directory after the first build.
    pub target_bytes: u64,
    /// S: bytes of the tracked files of one worktree.
    pub source_bytes: u64,
    /// Wall time of the first cold build, in seconds.
    pub first_build_seconds: f64,
    /// Wall time of the second cold build, in seconds.
    pub second_build_seconds: f64,
    pub sccache: SccacheFigures,
}

impl BuildMeasurement {
    /// The most recent of `measurements`, by time and then id; None when there are none.
    pub fn latest(measurements: &[BuildMeasurement]) -> Option<&BuildMeasurement> {
        measurements
            .iter()
            .max_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)))
    }

    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        let good = |v: f64| v.is_finite() && v >= 0.0;
        if !good(self.first_build_seconds) || !good(self.second_build_seconds) {
            return Err("a build measurement's times are finite, non-negative seconds".into());
        }
        if self.commit.trim().is_empty() || self.cargo_lock_hash.trim().is_empty() {
            return Err("a build measurement names its commit and its Cargo.lock hash".into());
        }
        Ok(())
    }
}

/// The schema tag of a Hestia sweep.
pub const HESTIA_SWEEP_SCHEMA: &str = "athena.hestia-sweep/1";

/// Where a Hestia finding stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HestiaFindingState {
    /// Reported and not yet seen to be fixed.
    Open,
    /// Seen to be fixed by a later sweep.
    Resolved,
}

impl HestiaFindingState {
    pub fn as_str(self) -> &'static str {
        match self {
            HestiaFindingState::Open => "open",
            HestiaFindingState::Resolved => "resolved",
        }
    }
}

/// One thing a Hestia check reports (`docs/design/hestia.md` section 9, job 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HestiaFinding {
    /// The id of the check that reported it.
    pub check: String,
    /// What the finding is about: a branch, a path, a request, a binary.
    pub subject: String,
    /// What the check saw, or the error it hit. Never empty.
    pub evidence: String,
    /// How long the problem has stood, in whole seconds; 0 for one first seen in this sweep.
    pub age_seconds: u64,
    pub state: HestiaFindingState,
}

/// What one check did in one sweep. A check that ran and found nothing is listed with a count of 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HestiaCheckResult {
    pub check: String,
    /// How many things the check looked at.
    pub examined: u32,
    /// How many findings the check reported; equals the sweep's findings for this check.
    pub count: u32,
    /// True when the check failed to run. Its error is then one of its findings, so it never looks clean.
    pub errored: bool,
}

/// `athena.hestia-sweep/1`: one run of Hestia's checks. Sweeps are never edited; the latest is the newest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HestiaSweep {
    pub id: RecordId,
    pub at: Timestamp,
    /// Every check that was registered, in order, with its count.
    pub checks: Vec<HestiaCheckResult>,
    pub findings: Vec<HestiaFinding>,
}

impl HestiaSweep {
    /// The most recent of `sweeps`, by time and then id; None when there are none.
    pub fn latest(sweeps: &[HestiaSweep]) -> Option<&HestiaSweep> {
        sweeps
            .iter()
            .max_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)))
    }

    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        for (i, c) in self.checks.iter().enumerate() {
            if c.check.trim().is_empty() {
                return Err("every check has an id".into());
            }
            if self.checks[..i].iter().any(|p| p.check == c.check) {
                return Err(format!("check {} is listed twice", c.check));
            }
            let reported = self.findings.iter().filter(|f| f.check == c.check).count();
            if reported != c.count as usize {
                return Err(format!(
                    "check {} counts {} finding(s) but the sweep holds {reported}",
                    c.check, c.count
                ));
            }
        }
        for f in &self.findings {
            if !self.checks.iter().any(|c| c.check == f.check) {
                return Err(format!(
                    "a finding names check {}, which is not listed",
                    f.check
                ));
            }
            if f.subject.trim().is_empty() || f.evidence.trim().is_empty() {
                return Err("every finding has a subject and evidence".into());
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------------------
// Job planning (`docs/design/job-planning.md` sections 3 to 5)

pub const JOB_PLAN_SCHEMA: &str = "athena.job-plan/1";
pub const CRITIQUE_SCHEMA: &str = "athena.critique/1";

/// An option the planner weighed and turned down.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RejectedOption {
    pub option: String,
    pub why: String,
}

/// One design choice in a plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanDecision {
    pub choice: String,
    pub reason: String,
    pub rejected: Vec<RejectedOption>,
}

/// A file the plan touches. Athena, not the planner, sets `governance` and `exists`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanFile {
    pub path: String,
    pub action: DeliverableAction,
    pub purpose: String,
    pub governance: bool,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanRisk {
    pub risk: String,
    pub mitigation: String,
}

/// How one acceptance criterion will be shown to hold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Demonstration {
    /// The criterion's number in the reading, from 1.
    pub criterion: u32,
    pub method: String,
    /// The command or test that shows it.
    pub command: String,
    /// What running it observes.
    pub observes: String,
    /// What would fail if the behavior were missing; a behavior demonstration needs it.
    pub fails_without: Option<String>,
    /// The test seam the demonstration drives; a behavior demonstration needs it. It names the production
    /// entry point called, the fake or fixture used and how it is injected, and either "exists" or the plan
    /// step that adds it. Records written before the field existed lack it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seam: Option<String>,
}

/// `athena.job-plan/1`: one round of the planner's plan, at `<job>/job-plan-<round>.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobPlan {
    pub job: JobId,
    /// From 1.
    pub round: u32,
    pub at: Timestamp,
    /// The planner's invocation, as numbered in the job's `invocations/` directory.
    pub invocation_seq: u32,
    pub approach: String,
    pub decisions: Vec<PlanDecision>,
    pub files: Vec<PlanFile>,
    pub steps: Vec<String>,
    pub risks: Vec<PlanRisk>,
    pub demonstrations: Vec<Demonstration>,
    pub open_questions: Vec<String>,
    /// The SHA-256 of the section of the producer's prompt that code renders from this plan.
    pub producer_section_sha256: String,
    /// Set on the first plan of a job that resumed planning from the kept plan of this earlier job, which
    /// stopped at the planning round cap. Absent on every other plan.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from: Option<JobId>,
}

impl JobPlan {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if self.round == 0 {
            return Err("a job plan's round starts at 1".into());
        }
        if self.approach.trim().is_empty() {
            return Err("a job plan states its approach".into());
        }
        if self.demonstrations.iter().any(|d| d.criterion == 0) {
            return Err("a demonstration names its criterion by number, from 1".into());
        }
        if self.producer_section_sha256.len() != 64
            || !self
                .producer_section_sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
        {
            return Err("producer_section_sha256 is a SHA-256 in hexadecimal".into());
        }
        Ok(())
    }
}

/// A critique's verdict on a plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CritiqueVerdict {
    Accept,
    Revise,
    Reject,
}

/// `athena.critique/1`: the critic's review of one plan round, at `<job>/critique-<round>.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Critique {
    pub job: JobId,
    /// The round of the plan reviewed, from 1.
    pub round: u32,
    pub at: Timestamp,
    /// The critic's invocation; none when Athena's own checks stopped the round before any critic call.
    pub invocation_seq: Option<u32>,
    pub agent_verdict: Option<CritiqueVerdict>,
    pub final_verdict: CritiqueVerdict,
    /// Findings from the critic and from Athena's own checks.
    pub findings: Vec<Finding>,
    /// Set when the critic judged that the accepted reading misread the request.
    pub request_reading_mismatch: Option<String>,
    pub summary: Option<String>,
}

impl Critique {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if self.round == 0 {
            return Err("a critique's round starts at 1".into());
        }
        if self.final_verdict == CritiqueVerdict::Accept
            && self
                .findings
                .iter()
                .any(|f| matches!(f.severity, Severity::Blocker | Severity::Major))
        {
            return Err("a critique that accepts cannot hold a blocker or major finding".into());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------------------
// The baseline of checks

pub const BASELINE_SCHEMA: &str = "athena.baseline/1";

/// `athena.baseline/1`: the job's configured checks run on the base commit, before the first producer
/// invocation, at `<job>/baseline.json`. It records which checks already fail before the producer touches
/// anything. At verification, a check that failed here and fails again is marked pre-existing and does not
/// fail the verdict, unless both runs name their failing tests and the candidate fails a test the base did
/// not (see [`CheckRun::failed_tests`]).
///
/// A check that fails, times out or cannot start is recorded as not passed, with a note; none of these stops
/// the job. When the baseline could not run at all, `checks` is empty and `note` says why.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    pub job: JobId,
    /// The commit the checks ran on: the job's base commit.
    pub base_commit: String,
    /// The sequence number the first producer invocation gets. The baseline was taken before it.
    pub before_invocation: u32,
    /// One entry per configured check. No path-skipping applies, since no change list exists yet.
    pub checks: Vec<CheckRun>,
    /// Why the baseline is empty or partial, if it is.
    pub note: Option<String>,
}

/// What a verification check's note starts with when the same check already failed on the base commit.
pub const PRE_EXISTING_NOTE: &str = "pre-existing: this check already failed on the base commit";

impl Baseline {
    /// True only when this baseline ran without a note and holds a failed entry named `name`. A baseline
    /// with a note (it could not run, or is partial), an empty one, or one without that check says nothing
    /// about the check, so it is never read as a reason to excuse a failure.
    pub fn failed_before(&self, name: &str) -> bool {
        self.note.is_none() && self.checks.iter().any(|c| c.name == name && !c.passed)
    }

    /// The failing test names recorded for the check `name` on the base: empty when the baseline has a
    /// note, holds no failed entry of that name, or recorded no names for it (see
    /// [`CheckRun::failed_tests`]). The check-name rule of [`Baseline::failed_before`] is the fallback.
    pub fn failed_tests_before(&self, name: &str) -> &[String] {
        if self.note.is_some() {
            return &[];
        }
        self.checks
            .iter()
            .find(|c| c.name == name && !c.passed)
            .map_or(&[], |c| c.failed_tests.as_slice())
    }

    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if self.base_commit.trim().is_empty() {
            return Err("a baseline names the base commit it ran on".into());
        }
        let mut seen = std::collections::BTreeSet::new();
        for c in &self.checks {
            if !seen.insert(c.name.as_str()) {
                return Err(format!("a baseline lists check {} twice", c.name));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------------------
// The replay of accepted readings

pub const REPLAY_SCHEMA: &str = "athena.replay/1";

/// One request that the candidate's reading rules refuse and the base's rules accepted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayRefusal {
    /// The job whose accepted reading is now refused.
    pub job: JobId,
    /// The candidate's refusal text.
    pub reason: String,
}

/// `athena.replay/1`: the newest accepted readings run through the base build and the candidate build of
/// `athena check-reading`, at `<job>/replay.json`. It lists the requests the candidate refuses and the base
/// accepted. A candidate that newly refuses more than a tenth of the replayed readings is held, unless its
/// acceptance criteria state that a higher refusal rate is intended; the record keeps the list either way.
///
/// When the replay could not run (no history, a build or probe failure, a failed check), `no_replay` says why,
/// `total` is 0 and the list is empty. That is a recorded absence, never a pass.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Replay {
    pub job: JobId,
    /// The base commit the base build was made from.
    pub base_commit: String,
    /// The candidate commit the candidate build was made from.
    pub candidate_commit: String,
    /// The most readings one replay uses.
    pub window: u32,
    /// How many accepted readings were replayed.
    pub total: u32,
    /// The requests the candidate refuses and the base accepted.
    pub newly_refused: Vec<ReplayRefusal>,
    /// Whether the job is held for the refusal rate.
    pub held: bool,
    /// Whether the criteria state that a higher refusal rate is intended.
    pub intended: bool,
    /// Why no replay ran, if none did.
    pub no_replay: Option<String>,
}

impl Replay {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if self.window == 0 {
            return Err("a replay's window is above 0".into());
        }
        if self.newly_refused.len() > self.total as usize {
            return Err("a replay cannot refuse more requests than it replayed".into());
        }
        if self
            .no_replay
            .as_deref()
            .is_some_and(|r| r.trim().is_empty())
        {
            return Err("a replay that did not run says why".into());
        }
        if self.no_replay.is_some() && (self.total != 0 || !self.newly_refused.is_empty()) {
            return Err("a replay that did not run holds no readings and no refusals".into());
        }
        if self.held && self.newly_refused.is_empty() {
            return Err("a held replay lists the requests it refuses".into());
        }
        if self.held && self.intended {
            return Err("a replay whose criteria intend the refusals is not held".into());
        }
        Ok(())
    }
}

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
        }
    }

    pub fn schema(&self) -> &'static str {
        match self {
            AnyRecord::Request(_) => "athena.request/1",
            AnyRecord::Reading(_) => "athena.reading/1",
            AnyRecord::Correction(_) => "athena.correction/1",
            AnyRecord::Priority(_) => "athena.priority/1",
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
        }
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
        }
    }

    #[test]
    fn roles_and_work_items_have_paths_schemas_and_round_trip() {
        let role = AnyRecord::Role(Box::new(crate::roles::steward(ts())));
        assert_eq!(role.schema(), "athena.role/1");
        assert_eq!(role.canonical_path(), "records/roles/steward-2.json");
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
        }
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
            note: String::new(),
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
        }
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
        };
        Baseline {
            job: job(),
            base_commit: "a".repeat(40),
            before_invocation: 1,
            checks: vec![check("fmt"), check("test")],
            note: None,
        }
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
        };
        let record = AnyRecord::Job(Box::new(job));
        assert_eq!(record.canonical_path(), "records/jobs/bootstrap/job.json");
        let back = AnyRecord::parse("x", record.to_json().as_bytes()).unwrap();
        assert_eq!(back, record);
    }
}
