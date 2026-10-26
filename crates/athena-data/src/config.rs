//! `athena.toml`: the repository's configuration for Athena.
//!
//! A job always reads its configuration from its base commit (D38), so this module only parses text; it
//! never reads files. [`Config::validate`] enforces the rules that must hold before any job runs, such as
//! "nothing verifies its own work" (the verifier model lies outside the producer set, M9).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The parsed configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub athena: AthenaSection,
    pub models: Models,
    pub budget: Budget,
    pub agents: Agents,
    pub capabilities: BTreeMap<String, CapabilityConfig>,
    pub classes: Classes,
    pub forecast: Forecast,
    pub secrets: Secrets,
    pub tools: Tools,
    pub history: History,
    pub records: RecordsConfig,
    pub personal_data: PersonalData,
    /// Pull-request landing. Absent from `athena.toml` means every setting is off.
    #[serde(default)]
    pub landing: Landing,
    /// How many jobs one drain may run at once. Absent from `athena.toml` means one at a time.
    #[serde(default)]
    pub parallel: Parallel,
    /// Job planning (`docs/design/job-planning.md`). Absent means off. It is left out of the serialized form
    /// when absent and reset in [`Config::reading_inputs`], so it never changes a reading's config digest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_plan: Option<JobPlanConfig>,
    /// Free-disk floor (`[disk]`). Absent means [`DEFAULT_MIN_FREE_GB`]. Like `job_plan`, it is left out of the
    /// serialized form when absent and reset in [`Config::reading_inputs`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disk: Option<DiskConfig>,
}

/// The free disk space, in gigabytes, below which a job is held at dispatch.
pub const DEFAULT_MIN_FREE_GB: u64 = 10;

/// `[disk]`: the minimum free disk space Athena wants before it starts a job or runs its checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiskConfig {
    /// Gigabytes (10^9 bytes) that must be free. The default is [`DEFAULT_MIN_FREE_GB`].
    #[serde(default = "DiskConfig::default_min_free_gb")]
    pub min_free_gb: u64,
}

impl DiskConfig {
    fn default_min_free_gb() -> u64 {
        DEFAULT_MIN_FREE_GB
    }
}

/// `[job_plan]`: the planner and critic that run before the producer. Nothing reads these settings yet.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobPlanConfig {
    /// Whether jobs are planned. The default is false.
    #[serde(default)]
    pub enabled: bool,
    /// The most plan-and-critique rounds before a job stops in planning, at least 1.
    #[serde(default = "JobPlanConfig::default_max_rounds")]
    pub max_rounds: u32,
    /// The cap on one planner call, in dollars.
    #[serde(default = "JobPlanConfig::default_usd")]
    pub planner_usd: f64,
    /// The cap on one critic call, in dollars.
    #[serde(default = "JobPlanConfig::default_usd")]
    pub critic_usd: f64,
    #[serde(default = "JobPlanConfig::default_timeout_s")]
    pub planner_timeout_s: u64,
    #[serde(default = "JobPlanConfig::default_timeout_s")]
    pub critic_timeout_s: u64,
}

impl JobPlanConfig {
    fn default_max_rounds() -> u32 {
        3
    }
    fn default_usd() -> f64 {
        0.75
    }
    fn default_timeout_s() -> u64 {
        600
    }
}

impl Default for JobPlanConfig {
    fn default() -> Self {
        JobPlanConfig {
            enabled: false,
            max_rounds: Self::default_max_rounds(),
            planner_usd: Self::default_usd(),
            critic_usd: Self::default_usd(),
            planner_timeout_s: Self::default_timeout_s(),
            critic_timeout_s: Self::default_timeout_s(),
        }
    }
}

/// The most jobs `[parallel] max_concurrency` may allow at once. A drain runs a model and a build per job, so
/// the ceiling keeps one setting from starting an unbounded number of them.
pub const MAX_CONCURRENCY_CEILING: u32 = 8;

/// `[parallel]`: how `athena drain` runs the independent items of one plan group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parallel {
    /// The most jobs a drain runs at the same time, from 1 to [`MAX_CONCURRENCY_CEILING`]. The default, 1, is
    /// the serial drain: one job at a time, in plan order. Only items of one plan group (no dependency, disjoint
    /// touched paths) ever run side by side.
    #[serde(default = "Parallel::default_max_concurrency")]
    pub max_concurrency: u32,
}

impl Parallel {
    fn default_max_concurrency() -> u32 {
        1
    }
}

impl Default for Parallel {
    fn default() -> Self {
        Parallel {
            max_concurrency: Self::default_max_concurrency(),
        }
    }
}

/// `[landing]`: how `athena land --pr` treats the merge.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Landing {
    /// When true, Athena asks the host to merge a pull request over the REST API once its checks are green.
    /// When false (the default), it stops at an open, green pull request and the merge is made elsewhere.
    #[serde(default)]
    pub merge_via_rest: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AthenaSection {
    pub schema: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Models {
    pub reader: String,
    pub producer_default: String,
    pub producer_allowed: Vec<String>,
    pub verifier: String,
    #[serde(default)]
    pub names: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub reader_usd: f64,
    pub producer_usd: f64,
    pub verifier_usd: f64,
    pub job_usd: f64,
    pub max_readings: u32,
    pub reader_timeout_s: u64,
    pub producer_timeout_s: u64,
    pub verifier_timeout_s: u64,
    pub check_timeout_s: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Agents {
    pub claude_version: String,
    pub env_pass: Vec<String>,
    #[serde(default)]
    pub env_pass_if_set: Vec<String>,
    pub bash_default_timeout_ms: u64,
    pub bash_max_timeout_ms: u64,
    pub host_baseline_tokens: u64,
}

/// One capability's configuration, such as `[capabilities.change]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityConfig {
    /// Program → allowed first arguments for the producer's Bash ("*" allows any).
    pub producer_commands: BTreeMap<String, Vec<String>>,
    /// Program → allowed first arguments for the verifier's Bash.
    pub verifier_commands: BTreeMap<String, Vec<String>>,
    /// Environment for agents and checks; values may use `{job_dir}`, `{side}` and `{athena_home}`.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub checks: Vec<CheckConfig>,
    /// For a capability that studies a repository: the URL the core pins before dispatch.
    #[serde(default)]
    pub repo_url: Option<String>,
    /// For a capability that studies a repository: the branch, tag, or commit of `repo_url` to pin.
    #[serde(default)]
    pub repo_ref: Option<String>,
}

/// A deterministic check the core runs on the candidate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckConfig {
    pub name: String,
    pub run: Vec<String>,
    #[serde(default)]
    pub kind: CheckKind,
    /// Path patterns that let the check be skipped. When set and every changed path matches at least one
    /// pattern, the check does not run and the run records a note. Absent means the check always runs.
    /// A pattern ending in `/` matches every path under that directory; a pattern starting with `*.`
    /// matches any path ending in the rest (`*.json` matches `a.json` and `x/y/a.json`); any other
    /// pattern matches exactly one path.
    #[serde(default)]
    pub skip_when_only: Option<Vec<String>>,
}

/// What a check's success means.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckKind {
    /// Success is exit status 0.
    #[default]
    Command,
    /// Success is exit status 0 and at least one test ran (M12).
    Test,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Classes {
    pub list: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Forecast {
    pub reference: ReferencePriors,
}

/// The reference forecaster's priors, used until history accumulates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferencePriors {
    pub p_reading_accepted: f64,
    pub p_verification_passes: f64,
    pub p_user_intervenes: f64,
    pub pseudo_count: f64,
    pub cost_usd: QuantilePrior,
    pub wall_time_s: QuantilePrior,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuantilePrior {
    pub p10: f64,
    pub p50: f64,
    pub p90: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Secrets {
    #[serde(default)]
    pub allowlist: Vec<SecretAllow>,
}

/// One reviewed gitleaks allow-list entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretAllow {
    pub description: String,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub regexes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tools {
    pub gitleaks: ToolPin,
    /// The compiler cache, when the repository pins one. Absent means setup installs none.
    #[serde(default)]
    pub sccache: Option<CratePin>,
}

/// A tool built by `cargo install --locked` at one pinned version; there are no archive digests to keep.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CratePin {
    pub version: String,
}

/// A pinned tool: its version and the SHA-256 of its release archive per platform.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolPin {
    pub version: String,
    pub sha256: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct History {
    #[serde(default)]
    pub exempt: Vec<String>,
    /// The id of the job whose landing is the records-history cutover (`docs/design/records-history.md`
    /// §4 and §10, D111). `C0` is the host commit of that job's landing record on `athena/records`. Unset
    /// means no cutover: the history check judges every commit by the old rules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cutover_job: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordsConfig {
    pub max_file_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersonalData {
    pub email_allow: Vec<String>,
}

impl PersonalData {
    /// Whether an email address may appear in committed content.
    ///
    /// An entry ending in `@` allows that local part, one starting with `@` allows that exact domain, and
    /// one starting with `.` allows every domain ending in that suffix.
    pub fn allows(&self, email: &str) -> bool {
        let email = email.to_ascii_lowercase();
        self.email_allow.iter().any(|entry| {
            let entry = entry.to_ascii_lowercase();
            if entry.ends_with('@') {
                email.starts_with(&entry)
            } else if entry.starts_with('@') || entry.starts_with('.') {
                email.ends_with(&entry)
            } else {
                email == entry
            }
        })
    }
}

/// Why a configuration cannot be used.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ConfigError {
    #[error("athena.toml does not parse: {0}")]
    Parse(String),
    #[error("athena.toml schema {0} is not supported; this Athena reads schema 1")]
    Schema(u32),
    #[error(
        "the verifier model {0} is also allowed to produce; nothing may verify its own work (remove it from producer_allowed)"
    )]
    VerifierCanProduce(String),
    #[error("the default producer {0} is not in producer_allowed")]
    DefaultProducerNotAllowed(String),
    #[error("the class list must include \"other\"")]
    NoOtherClass,
    #[error("budget {0} must be positive")]
    Budget(&'static str),
    #[error("reference forecast: {0}")]
    Reference(&'static str),
    #[error(
        "[history] exempt entry {0:?} is not a full commit id; an exemption names exactly one commit, by all 40 (or, in a SHA-256 repository, 64) hexadecimal characters"
    )]
    Exempt(String),
    #[error("[parallel] max_concurrency {0} must be from 1 to {MAX_CONCURRENCY_CEILING}")]
    Parallel(u32),
    #[error("[job_plan] {0} must be positive")]
    JobPlan(&'static str),
}

impl Config {
    /// Parses and validates `athena.toml` text.
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        let config: Config = toml::from_str(text).map_err(|e| ConfigError::Parse(e.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    /// The configuration that bears on a reading, as canonical text: every setting except `[parallel]`
    /// (how many jobs a drain runs at once), `[landing]` (how a pull request is merged) and `[job_plan]`
    /// (not yet read by any stage, and absent from the text so old digests hold). None changes
    /// what the reader sees or what a job may do, so a change to them leaves a reading valid (D38). The text
    /// does not depend on how the file is laid out, only on the parsed values, so comments, spacing and key
    /// order do not count either.
    pub fn reading_inputs(&self) -> String {
        let mut relevant = self.clone();
        relevant.parallel = Parallel::default();
        relevant.landing = Landing::default();
        relevant.job_plan = None;
        relevant.disk = None;
        serde_json::to_string(&relevant).expect("a parsed configuration serializes")
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.athena.schema != 1 {
            return Err(ConfigError::Schema(self.athena.schema));
        }
        if self.models.producer_allowed.contains(&self.models.verifier) {
            return Err(ConfigError::VerifierCanProduce(
                self.models.verifier.clone(),
            ));
        }
        if !self
            .models
            .producer_allowed
            .contains(&self.models.producer_default)
        {
            return Err(ConfigError::DefaultProducerNotAllowed(
                self.models.producer_default.clone(),
            ));
        }
        if !self.classes.list.iter().any(|c| c == "other") {
            return Err(ConfigError::NoOtherClass);
        }
        let b = &self.budget;
        for (name, value) in [
            ("reader_usd", b.reader_usd),
            ("producer_usd", b.producer_usd),
            ("verifier_usd", b.verifier_usd),
            ("job_usd", b.job_usd),
        ] {
            if !(value.is_finite() && value > 0.0) {
                return Err(ConfigError::Budget(name));
            }
        }
        if b.max_readings == 0 {
            return Err(ConfigError::Budget("max_readings"));
        }
        let r = &self.forecast.reference;
        for p in [
            r.p_reading_accepted,
            r.p_verification_passes,
            r.p_user_intervenes,
        ] {
            if !(0.0..=1.0).contains(&p) {
                return Err(ConfigError::Reference("probabilities must lie in [0, 1]"));
            }
        }
        for q in [r.cost_usd, r.wall_time_s] {
            if !(q.p10 > 0.0 && q.p10 <= q.p50 && q.p50 <= q.p90) {
                return Err(ConfigError::Reference(
                    "quantiles must satisfy 0 < p10 <= p50 <= p90",
                ));
            }
        }
        if r.pseudo_count <= 0.0 {
            return Err(ConfigError::Reference("pseudo_count must be positive"));
        }
        // A short entry would match any commit whose id begins with it, later ones included
        // (acceptance verification 2, N1).
        for entry in &self.history.exempt {
            let hex = entry
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c));
            if !(hex && (entry.len() == 40 || entry.len() == 64)) {
                return Err(ConfigError::Exempt(entry.clone()));
            }
        }
        if !(1..=MAX_CONCURRENCY_CEILING).contains(&self.parallel.max_concurrency) {
            return Err(ConfigError::Parallel(self.parallel.max_concurrency));
        }
        if let Some(plan) = &self.job_plan {
            if plan.max_rounds == 0 {
                return Err(ConfigError::JobPlan("max_rounds"));
            }
            for (name, value) in [
                ("planner_usd", plan.planner_usd),
                ("critic_usd", plan.critic_usd),
            ] {
                if !(value.is_finite() && value > 0.0) {
                    return Err(ConfigError::JobPlan(name));
                }
            }
            if plan.planner_timeout_s == 0 {
                return Err(ConfigError::JobPlan("planner_timeout_s"));
            }
            if plan.critic_timeout_s == 0 {
                return Err(ConfigError::JobPlan("critic_timeout_s"));
            }
        }
        Ok(())
    }

    /// The display name for a model, for Co-Authored-By trailers.
    pub fn model_name<'a>(&'a self, model: &'a str) -> &'a str {
        self.models
            .names
            .get(model)
            .map(String::as_str)
            .unwrap_or(model)
    }

    /// The minimum free disk space, in gigabytes: `[disk] min_free_gb`, or [`DEFAULT_MIN_FREE_GB`] when absent.
    pub fn min_free_gb(&self) -> u64 {
        self.disk.map_or(DEFAULT_MIN_FREE_GB, |d| d.min_free_gb)
    }

    pub fn capability(&self, id: &str) -> Option<&CapabilityConfig> {
        self.capabilities.get(id)
    }

    pub fn is_class(&self, class: &str) -> bool {
        self.classes.list.iter().any(|c| c == class)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// The repository's own `athena.toml`, which every Athena build must be able to read.
    pub(crate) const REPO_CONFIG: &str = include_str!("../../../athena.toml");

    /// `athena.toml` without its `[job_plan]` table, wherever it sits: the lines from the header to the next
    /// table header. For tests that append their own `[job_plan]` or need job planning absent.
    fn unplanned(text: &str) -> String {
        let mut skipping = false;
        text.split_inclusive('\n')
            .filter(|line| {
                let head = line.trim_start();
                if head.starts_with('[') {
                    skipping = head.trim_end() == "[job_plan]";
                }
                !skipping
            })
            .collect()
    }

    #[test]
    fn the_repository_config_turns_job_planning_on_with_the_designs_caps() {
        let plan = Config::parse(REPO_CONFIG).unwrap().job_plan.unwrap();
        assert!(plan.enabled);
        assert_eq!(plan.planner_usd, 0.75);
        assert_eq!(plan.critic_usd, 0.75);
        assert_eq!(plan.planner_timeout_s, 600);
        assert_eq!(plan.critic_timeout_s, 600);
        assert_eq!(plan.max_rounds, 3);
        let stripped = unplanned(REPO_CONFIG);
        assert!(!stripped.contains("[job_plan]"));
        assert!(Config::parse(&stripped).unwrap().job_plan.is_none());
    }

    #[test]
    fn repository_config_parses_and_validates() {
        // The rules, not the values: a job may change the values through a listed governance change.
        let config = Config::parse(REPO_CONFIG).expect("athena.toml is valid");
        assert!(
            !config
                .models
                .producer_allowed
                .contains(&config.models.verifier)
        );
        assert!(config.capability("change").is_some());
        assert!(config.is_class("other"));
    }

    #[test]
    fn reading_inputs_ignore_parallel_and_landing_but_not_other_settings() {
        let base = Config::parse(REPO_CONFIG).unwrap();
        let mut other = base.clone();
        other.parallel.max_concurrency = if base.parallel.max_concurrency == 1 {
            2
        } else {
            1
        };
        other.landing.merge_via_rest = !base.landing.merge_via_rest;
        assert_eq!(base.reading_inputs(), other.reading_inputs());
        let mut changed = base.clone();
        changed.budget.job_usd += 1.0;
        assert_ne!(base.reading_inputs(), changed.reading_inputs());
    }

    fn digest(config: &Config) -> String {
        crate::sha256_hex(config.reading_inputs().as_bytes())
    }

    #[test]
    fn reading_inputs_digest_is_unchanged_by_the_job_plan_section() {
        let unplanned = unplanned(REPO_CONFIG);
        let base = Config::parse(&unplanned).unwrap();
        assert!(base.job_plan.is_none());
        let text = format!("{unplanned}\n[job_plan]\nenabled = true\n");
        let on = Config::parse(&text).unwrap();
        assert!(on.job_plan.as_ref().is_some_and(|j| j.enabled));
        // A reading records this digest, so adding the section must not move it. Compared with the same
        // config without the section, not a pinned value, so a real config change does not break the test.
        assert_eq!(digest(&on), digest(&base));
        assert_eq!(on.reading_inputs(), base.reading_inputs());
    }

    #[test]
    fn min_free_disk_defaults_to_ten_gigabytes_and_parses_when_set() {
        let base = Config::parse(REPO_CONFIG).unwrap();
        assert!(base.disk.is_none());
        assert_eq!(base.min_free_gb(), 10);
        let set = Config::parse(&format!("{REPO_CONFIG}\n[disk]\nmin_free_gb = 3\n")).unwrap();
        assert_eq!(set.min_free_gb(), 3);
        let empty = Config::parse(&format!("{REPO_CONFIG}\n[disk]\n")).unwrap();
        assert_eq!(empty.min_free_gb(), 10);
        // A reading records this digest; the section must not move it.
        assert_eq!(set.reading_inputs(), base.reading_inputs());
        assert_eq!(digest(&set), digest(&base));
    }

    #[test]
    fn job_plan_defaults_are_off_and_bounded() {
        let text = format!("{}\n[job_plan]\n", unplanned(REPO_CONFIG));
        let config = Config::parse(&text).unwrap();
        let plan = config.job_plan.unwrap();
        assert!(!plan.enabled);
        assert_eq!(plan.max_rounds, 3);
        assert_eq!(plan.planner_usd, 0.75);
        assert_eq!(plan.critic_usd, 0.75);
        assert_eq!(plan.planner_timeout_s, 600);
        assert_eq!(plan.critic_timeout_s, 600);
    }

    #[test]
    fn bad_job_plan_settings_are_refused_naming_the_field() {
        for (line, field) in [
            ("max_rounds = 0", "max_rounds"),
            ("planner_usd = 0.0", "planner_usd"),
            ("critic_usd = -1.0", "critic_usd"),
            ("planner_timeout_s = 0", "planner_timeout_s"),
            ("critic_timeout_s = 0", "critic_timeout_s"),
            ("surprise = true", "surprise"),
        ] {
            let text = format!(
                "{}\n[job_plan]\nenabled = true\n{line}\n",
                unplanned(REPO_CONFIG)
            );
            let error = Config::parse(&text).expect_err(line).to_string();
            assert!(error.contains(field), "{line}: {error}");
        }
    }

    #[test]
    fn verifier_that_can_produce_is_rejected() {
        // M9 (blocks): a configuration letting one model produce and verify is rejected.
        let mut config = Config::parse(REPO_CONFIG).unwrap();
        let verifier = config.models.verifier.clone();
        config.models.producer_allowed.push(verifier);
        assert!(matches!(
            config.validate(),
            Err(ConfigError::VerifierCanProduce(_))
        ));
    }

    #[test]
    fn default_configuration_loads() {
        // M9 (allows): the default configuration, whose verifier is outside the producer set, loads.
        assert!(Config::parse(REPO_CONFIG).is_ok());
    }

    #[test]
    fn skip_when_only_is_set_on_fmt_and_clippy_only() {
        // M31 (allows): the repository skips fmt and clippy for prose; the test check never skips.
        let config = Config::parse(REPO_CONFIG).unwrap();
        let change = config.capability("change").unwrap();
        let skip = |name: &str| {
            change
                .checks
                .iter()
                .find(|c| c.name == name)
                .unwrap()
                .skip_when_only
                .clone()
        };
        let prose = Some(vec!["docs/".to_string(), "*.md".to_string()]);
        assert_eq!(skip("fmt"), prose);
        assert_eq!(skip("clippy"), prose);
        assert_eq!(skip("test"), None);
    }

    #[test]
    fn skip_when_only_is_optional_and_parsed_when_present() {
        // M31 (blocks nothing): a key on a check that has none is parsed into its patterns.
        let text = REPO_CONFIG.replacen(
            "[[capabilities.change.checks]]\nname = \"test\"\n",
            "[[capabilities.change.checks]]\nname = \"test\"\nskip_when_only = [\"docs/\", \"*.json\"]\n",
            1,
        );
        let config = Config::parse(&text).unwrap();
        let test = config
            .capability("change")
            .unwrap()
            .checks
            .iter()
            .find(|c| c.name == "test")
            .unwrap();
        assert_eq!(
            test.skip_when_only,
            Some(vec!["docs/".to_string(), "*.json".to_string()])
        );
    }

    #[test]
    fn sccache_pin_is_optional_and_holds_only_a_version() {
        const PIN: &str = "[tools.sccache]\nversion = \"0.10.0\"\n";
        // athena.toml pins none (D84), so no sccache is configured.
        assert!(!REPO_CONFIG.contains("[tools.sccache]"));
        assert_eq!(Config::parse(REPO_CONFIG).unwrap().tools.sccache, None);
        // The table is still accepted, and the version is read.
        let text = format!("{REPO_CONFIG}\n{PIN}");
        assert_eq!(
            Config::parse(&text).unwrap().tools.sccache,
            Some(CratePin {
                version: "0.10.0".into()
            })
        );
        // A digest table is not part of the pin.
        let text = format!("{REPO_CONFIG}\n{PIN}sha256 = {{}}\n");
        assert!(matches!(Config::parse(&text), Err(ConfigError::Parse(_))));
    }

    #[test]
    fn neither_env_wires_a_compiler_cache() {
        let config = Config::parse(REPO_CONFIG).unwrap();
        for id in ["change", "study"] {
            let env = &config.capabilities[id].env;
            for key in ["RUSTC_WRAPPER", "SCCACHE_DIR", "CARGO_INCREMENTAL"] {
                assert!(!env.contains_key(key), "{id} sets {key}");
            }
            assert_eq!(env["CARGO_TARGET_DIR"], "{job_dir}/target/{side}");
            assert_eq!(env["CARGO_TERM_COLOR"], "never");
        }
    }

    #[test]
    fn rest_merge_is_off_unless_the_config_turns_it_on() {
        // The default is tested on a config with no landing table: everything before it in athena.toml.
        let (stripped, _) = REPO_CONFIG
            .split_once("[landing]")
            .expect("athena.toml has a landing table");
        assert!(!stripped.contains("merge_via_rest"));
        assert!(!Config::parse(stripped).unwrap().landing.merge_via_rest);
        let on = format!("{stripped}\n[landing]\nmerge_via_rest = true\n");
        assert!(Config::parse(&on).unwrap().landing.merge_via_rest);
        let empty = format!("{stripped}\n[landing]\n");
        assert!(!Config::parse(&empty).unwrap().landing.merge_via_rest);
        // This repository turns the merge on.
        assert!(Config::parse(REPO_CONFIG).unwrap().landing.merge_via_rest);
    }

    #[test]
    fn unknown_keys_are_rejected() {
        let text = format!("{REPO_CONFIG}\n[surprise]\nkey = 1\n");
        assert!(matches!(Config::parse(&text), Err(ConfigError::Parse(_))));
        let text = REPO_CONFIG.replacen("[parallel]\n", "[parallel]\nsurprise = 1\n", 1);
        assert!(matches!(Config::parse(&text), Err(ConfigError::Parse(_))));
    }

    #[test]
    fn the_real_athena_toml_parses_with_its_parallel_section() {
        // The schema field and the athena.toml section must agree: this reads the file the repository ships.
        assert!(
            REPO_CONFIG.contains("\n[parallel]\n"),
            "athena.toml has a [parallel] section"
        );
        let config = Config::parse(REPO_CONFIG).expect("athena.toml parses under Config");
        assert!((1..=MAX_CONCURRENCY_CEILING).contains(&config.parallel.max_concurrency));
    }

    #[test]
    fn without_a_parallel_section_the_drain_is_serial() {
        let (stripped, _) = REPO_CONFIG
            .split_once("[parallel]")
            .expect("athena.toml has a parallel table");
        let config = Config::parse(stripped).expect("a config without [parallel] parses");
        assert_eq!(config.parallel.max_concurrency, 1);
        let empty = format!("{stripped}\n[parallel]\n");
        assert_eq!(Config::parse(&empty).unwrap().parallel.max_concurrency, 1);
    }

    #[test]
    fn max_concurrency_must_lie_between_one_and_the_ceiling() {
        let (stripped, _) = REPO_CONFIG.split_once("[parallel]").unwrap();
        let with =
            |n: u32| Config::parse(&format!("{stripped}\n[parallel]\nmax_concurrency = {n}\n"));
        assert_eq!(
            with(MAX_CONCURRENCY_CEILING)
                .unwrap()
                .parallel
                .max_concurrency,
            MAX_CONCURRENCY_CEILING
        );
        assert!(matches!(with(0), Err(ConfigError::Parallel(0))));
        assert!(matches!(
            with(MAX_CONCURRENCY_CEILING + 1),
            Err(ConfigError::Parallel(_))
        ));
    }

    #[test]
    fn email_allow_list_matches_by_kind() {
        let config = Config::parse(REPO_CONFIG).unwrap();
        let allow =
            |local: &str, domain: &str| config.personal_data.allows(&format!("{local}@{domain}"));
        assert!(allow("noreply", "anthropic.com"));
        assert!(allow("someone", "example.org"));
        assert!(allow("snapshot", "noreply.invalid"));
        assert!(!allow("someone", "sub.example.org"));
        assert!(!allow("jane", "corp.com"));
        assert!(!allow("noreply.jane", "corp.com"));
    }

    #[test]
    fn test_checks_are_marked() {
        let config = Config::parse(REPO_CONFIG).unwrap();
        let checks = &config.capability("change").unwrap().checks;
        assert!(
            checks
                .iter()
                .any(|c| c.name == "test" && c.kind == CheckKind::Test)
        );
        assert!(
            checks
                .iter()
                .any(|c| c.name == "fmt" && c.kind == CheckKind::Command)
        );
    }

    #[test]
    fn cutover_job_is_optional_and_the_repo_config_names_the_cutover_job() {
        let config = Config::parse(REPO_CONFIG).unwrap();
        assert_eq!(
            config.history.cutover_job.as_deref(),
            Some("20261025-001232-vesh")
        );
        let without = REPO_CONFIG
            .lines()
            .filter(|l| !l.starts_with("cutover_job = "))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(without.len() < REPO_CONFIG.len(), "the line was removed");
        assert_eq!(Config::parse(&without).unwrap().history.cutover_job, None);
    }

    #[test]
    fn history_exemptions_must_be_full_commit_ids() {
        // N1 of acceptance verification 2: an exemption names one commit, in full, never a prefix.
        let with = |entries: &str| {
            let text = REPO_CONFIG
                .lines()
                .map(|l| {
                    if l.starts_with("exempt = ") {
                        format!("exempt = [{entries}]")
                    } else {
                        l.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            Config::parse(&text)
        };
        for bad in [
            "\"0\"",
            "\"49c6ac1\"",
            "\"\"",
            "\"49C6AC1524677B8091081115FB5D410ED6D2B2EA\"",
            "\"49c6ac1524677b8091081115fb5d410ed6d2b2eaz\"",
        ] {
            assert!(
                matches!(with(bad), Err(ConfigError::Exempt(_))),
                "{bad} must be refused"
            );
        }
        assert!(with("\"49c6ac1524677b8091081115fb5d410ed6d2b2ea\"").is_ok());
        assert!(
            with(&format!("\"{}\"", "ab".repeat(32))).is_ok(),
            "SHA-256 ids"
        );
        assert!(with("").is_ok(), "no exemptions at all");
    }
}
