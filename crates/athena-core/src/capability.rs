//! The core/capability seam (design §6.2).
//!
//! A capability owns what is specific to one kind of work: how to brief its producer, what its verifier
//! must look for, which checks prove it, and which extra changes it refuses. The core owns everything
//! common to all work. `JobConfig` is always read from the job's base commit, never from the candidate or
//! the working tree, so a candidate cannot choose its own checks, caps, allow-lists, or verifier (D38).

use anyhow::{Context, Result, anyhow};
use athena_context::PromptAsset;
pub use athena_context::briefs::CapabilityCard;
use athena_data::config::CheckKind;
use athena_data::records::{ChangeEntry, PolicyViolation};
use athena_data::{Config, DefaultsModel, sha256_hex};
use athena_gate::{CONFIG_PATH, DEFAULTS_PATH};
use athena_git::Git;
use std::collections::BTreeMap;

/// The configuration a job runs under, read from one commit.
#[derive(Debug, Clone)]
pub struct JobConfig {
    pub config: Config,
    pub defaults: DefaultsModel,
    /// The commit both files were read from.
    pub commit: String,
    pub config_sha256: String,
    pub defaults_sha256: String,
}

impl JobConfig {
    /// Reads `athena.toml` and `defaults.toml` from `commit`. Both must exist and parse.
    pub fn at(git: &Git, commit: &str) -> Result<JobConfig> {
        let short = &commit[..commit.len().min(12)];
        let config_bytes = git
            .show_file(commit, CONFIG_PATH)?
            .ok_or_else(|| anyhow!("{CONFIG_PATH} is missing at {short}; Athena needs it committed at the repository root"))?;
        let defaults_bytes = git
            .show_file(commit, DEFAULTS_PATH)?
            .ok_or_else(|| anyhow!("{DEFAULTS_PATH} is missing at {short}; Athena needs it committed at the repository root"))?;
        let config = Config::parse(&String::from_utf8_lossy(&config_bytes))
            .with_context(|| format!("{CONFIG_PATH} at {short}"))?;
        let defaults = DefaultsModel::parse(&String::from_utf8_lossy(&defaults_bytes))
            .with_context(|| format!("{DEFAULTS_PATH} at {short}"))?;
        Ok(JobConfig {
            config,
            defaults,
            commit: commit.to_string(),
            config_sha256: sha256_hex(&config_bytes),
            defaults_sha256: sha256_hex(&defaults_bytes),
        })
    }
}

/// Tools and permissions for one executing role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolGrant {
    pub tools: Vec<String>,
    /// Program → allowed first arguments, for Bash.
    pub commands: BTreeMap<String, Vec<String>>,
    pub writes: bool,
    pub permission_mode: String,
}

/// A deterministic check the core runs on the candidate before the verifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckSpec {
    pub name: String,
    pub run: Vec<String>,
    pub kind: CheckKind,
    /// Path patterns that let the check be skipped; `None` means it always runs (see `checks::skip_note`).
    pub skip_when_only: Option<Vec<String>>,
}

/// The producer's change as the core captured it: one entry per path, with modes and blob ids.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChangeList {
    pub entries: Vec<ChangeEntry>,
}

impl ChangeList {
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|e| e.path.as_str())
    }
}

/// What a kind of work needs from the core, and what it gives the core.
pub trait Capability: Send + Sync {
    /// Stable id used in records and in the reader's catalog, such as "change".
    fn id(&self) -> &'static str;
    /// The card the reader sees in the capability catalog.
    fn card(&self) -> CapabilityCard;
    /// Capability-specific prompt sections appended to the producer and verifier frames.
    fn producer_section(&self) -> PromptAsset;
    fn verifier_section(&self) -> PromptAsset;
    /// Tools and permission rules for the executing roles, from the job's configuration.
    fn producer_grant(&self, config: &JobConfig) -> ToolGrant;
    fn verifier_grant(&self, config: &JobConfig) -> ToolGrant;
    /// Deterministic checks the core runs on the candidate before the verifier agent.
    fn checks(&self, config: &JobConfig) -> Vec<CheckSpec>;
    /// The environment templates for agents and checks (`{job_dir}` and `{side}` are filled in).
    fn env(&self, config: &JobConfig) -> BTreeMap<String, String>;
    /// Capability-specific admission policy on the change list, beyond the core's own.
    fn admit(&self, change: &ChangeList, config: &JobConfig) -> Vec<PolicyViolation>;
    /// Whether the core must pin a repository, from this capability's `repo_url` and `repo_ref`, before
    /// dispatch and mount it read-only for both agents. Only a study does.
    fn pins_repo(&self) -> bool {
        false
    }
}
