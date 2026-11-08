//! Host-error, decision-state and default-expectation records.

use super::*;

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
