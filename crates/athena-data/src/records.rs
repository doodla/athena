//! The record types and where each one lives (design §5; the host-error, decision-state and
//! default-expectations types come from `docs/design/decision-sets.md`).
//!
//! A record is a pretty-printed JSON file under `records/`, written once and never modified (D06). Its
//! `schema` field names its type and version, and [`AnyRecord`] parses any record with unknown fields
//! denied. Every record has exactly one canonical location, derived from its own fields, so the gate can
//! check that a file sits where its content says it belongs (M5).

use crate::id::{JobId, RecordId};
use crate::quantity::Quantity;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The three agent roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Reader,
    Producer,
    Verifier,
}

impl Role {
    pub fn as_str(self) -> &'static str {
        match self {
            Role::Reader => "reader",
            Role::Producer => "producer",
            Role::Verifier => "verifier",
        }
    }
    pub fn parse(text: &str) -> Option<Role> {
        match text {
            "reader" => Some(Role::Reader),
            "producer" => Some(Role::Producer),
            "verifier" => Some(Role::Verifier),
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
    /// Terminated by a deadline, a budget, the kill switch, or the death of its parent.
    Killed,
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
    Other,
}

impl InterventionCategory {
    pub const ALL: [InterventionCategory; 8] = [
        InterventionCategory::DidWorkThemselves,
        InterventionCategory::CorrectedOutput,
        InterventionCategory::GaveDetailedInstructions,
        InterventionCategory::OverrodeDecision,
        InterventionCategory::AnsweredQuestion,
        InterventionCategory::CarriedState,
        InterventionCategory::FixedEnvironment,
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
            InterventionCategory::Other => "other",
        }
    }
    pub fn parse(text: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.as_str() == text)
    }
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

/// The producer's structured report: kept with its run, redacted, never shown to the verifier (D32).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerReport {
    pub summary: String,
    pub files: Vec<String>,
    pub checks_run: Vec<ReportedCheck>,
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

/// What the model that wrote the prose cost. There are no probabilities or confidence: no Jev is called.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntrospectionEvidence {
    pub model: String,
    pub cost_usd: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
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
    /// Always false in this build: the class and prevention kind came from a language model, not from Jev.
    pub from_jev: bool,
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
            _ if self.from_jev => Err("no Jev is called in this build".into()),
            _ => Ok(()),
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
            // Totals are what reconciliation checks an outcome against; only the bootstrap has none
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
        }
    }

    pub fn schema(&self) -> &'static str {
        match self {
            AnyRecord::Request(_) => "athena.request/1",
            AnyRecord::Reading(_) => "athena.reading/1",
            AnyRecord::Correction(_) => "athena.correction/1",
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
        }
    }
}

/// Markdown files allowed under a job directory, beside the JSON records.
const JOB_MARKDOWN: [&str; 2] = ["brief-producer.md", "brief-verifier.md"];

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
        }
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
        }
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
