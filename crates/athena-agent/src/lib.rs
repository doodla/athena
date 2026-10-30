//! Athena's model-invocation driver (design §10).
//!
//! - [`AgentDriver`] is the one interface the core uses to run an agent. [`claude::ClaudeCodeDriver`] runs
//!   Claude Code headless and hermetic; [`fake::FakeAgentDriver`] stands in for it in tests and in nested
//!   runs, and makes no model call.
//! - [`env`] builds every agent's environment from an allow-list, plus the variables Athena adds.
//! - [`stream`] reads Claude Code's stream-json output as it arrives and keeps only usage, tool names,
//!   Bash command lines, denials, and the structured output. No transcript is kept.
//! - [`procs`] runs agents in their own process groups, tied to their parent's life, recorded in a
//!   registry that the kill switch reaches.
//! - [`policy`] is the tool policy behind Athena's `PreToolUse` hook.

pub mod claude;
pub mod env;
pub mod fake;
pub mod policy;
pub mod procs;
pub mod stream;

use athena_data::records::{Denial, DriverInfo, InvocationOutcome, ModelUsage, Role};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

/// The environment variable that marks a nested run: Athena running inside one of its own jobs (D47).
pub const NESTED_ENV: &str = "ATHENA_NESTED";

/// Whether this process runs inside an Athena job (an agent's command or a check).
pub fn nested_marker() -> Option<String> {
    std::env::var(NESTED_ENV).ok().filter(|v| !v.is_empty())
}

/// The real driver's refusal to run inside a job: no model call may happen outside a job's records and
/// budget (D47).
pub fn refuse_nested(marker: Option<String>) -> Result<(), DriverError> {
    match marker {
        Some(job) => Err(DriverError::Nested(job)),
        None => Ok(()),
    }
}

/// How the hook reaches Athena's tool policy for one invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookSpec {
    /// The `athena` binary the hook runs.
    pub athena_bin: PathBuf,
    /// The policy file the hook evaluates against ([`policy::Policy`] as JSON).
    pub policy_file: PathBuf,
    /// Directories outside the working directory the agent may read (a job's evidence snapshot). The driver
    /// tells the agent runtime about them, so its own directory scope agrees with the policy's `read_roots`.
    pub read_dirs: Vec<PathBuf>,
}

/// Everything needed to run one agent once.
#[derive(Debug, Clone)]
pub struct AgentInvocation {
    pub job: String,
    pub seq: u32,
    pub role: Role,
    pub model: String,
    /// The effort level (`--effort`), one of `athena_data::config::EFFORT_LEVELS`.
    pub effort: String,
    pub system_prompt: String,
    pub user_prompt: String,
    /// The tools the agent has (`--tools`).
    pub tools: Vec<String>,
    /// Permission allow rules (`--allowedTools`), such as `Bash(cargo test:*)`.
    pub allowed_tools: Vec<String>,
    pub permission_mode: String,
    /// The snapshot the agent works in.
    pub workdir: PathBuf,
    /// The complete environment of the agent's process.
    pub env: BTreeMap<String, String>,
    pub hook: Option<HookSpec>,
    pub max_budget_usd: f64,
    pub timeout: Duration,
    pub json_schema: serde_json::Value,
    /// A private directory for this invocation's prompt and settings files.
    pub scratch: PathBuf,
}

/// Token counts, de-duplicated by API message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Usage {
    pub input: u64,
    pub output: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

/// What an invocation produced and cost.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentOutput {
    pub outcome: InvocationOutcome,
    pub detail: Option<String>,
    pub structured: Option<serde_json::Value>,
    pub started_at: jiff::Timestamp,
    pub ended_at: jiff::Timestamp,
    /// Athena's own wall-clock time for the invocation.
    pub wall_s: f64,
    /// Final usage when `usage_measured`, else the running sums: a lower bound.
    pub usage: Usage,
    pub usage_measured: bool,
    pub models_used: BTreeMap<String, ModelUsage>,
    /// The cost the driver reported, if the invocation completed.
    pub cost_usd: Option<f64>,
    pub turns: Option<u32>,
    pub duration_api_ms: Option<u64>,
    /// Bash commands that ran (denied ones excluded), unredacted.
    pub commands_run: Vec<String>,
    pub denials: Vec<Denial>,
}

/// Receives usage as it accrues, so a crash or kill still leaves a lower bound behind.
pub trait UsageSink {
    fn update(&mut self, usage: &Usage, commands_run: &[String]);
}

/// A sink that keeps nothing.
#[derive(Debug, Default)]
pub struct NoSink;

impl UsageSink for NoSink {
    fn update(&mut self, _: &Usage, _: &[String]) {}
}

/// Why an invocation could not run at all. An invocation that ran and failed returns an [`AgentOutput`]
/// with an error outcome instead, so its usage is still recorded.
#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("the kill switch is on (`athena start` turns it off)")]
    Stopped,
    #[error(
        "this is a nested run (ATHENA_NESTED={0}): the real driver makes no model calls inside a job; Athena uses its fake driver here"
    )]
    Nested(String),
    #[error("could not start the agent: {0}")]
    Spawn(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// The one interface for running agents (design §10.1). Drivers are shared between threads, because the
/// steward runs in the background beside the command that started it.
pub trait AgentDriver: Send + Sync {
    fn info(&self) -> DriverInfo;
    fn invoke(
        &self,
        spec: &AgentInvocation,
        sink: &mut dyn UsageSink,
    ) -> Result<AgentOutput, DriverError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_real_driver_refuses_under_the_nested_marker() {
        // M25 (blocks): under the marker, the real driver refuses to start.
        assert!(matches!(
            refuse_nested(Some("20261010-153012-k7q2".into())),
            Err(DriverError::Nested(_))
        ));
        assert!(refuse_nested(None).is_ok());
    }
}
