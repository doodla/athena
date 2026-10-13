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
    /// Environment for agents and checks; values may use `{job_dir}` and `{side}`.
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
}

impl Config {
    /// Parses and validates `athena.toml` text.
    pub fn parse(text: &str) -> Result<Self, ConfigError> {
        let config: Config = toml::from_str(text).map_err(|e| ConfigError::Parse(e.to_string()))?;
        config.validate()?;
        Ok(config)
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
    fn skip_when_only_is_optional_and_parsed_when_present() {
        // The repository's own file has no such key yet; every check then always runs.
        let config = Config::parse(REPO_CONFIG).unwrap();
        let change = config.capability("change").unwrap();
        assert!(change.checks.iter().all(|c| c.skip_when_only.is_none()));
        let text = REPO_CONFIG.replacen(
            "[[capabilities.change.checks]]\n",
            "[[capabilities.change.checks]]\nskip_when_only = [\"docs/\", \"*.json\"]\n",
            1,
        );
        let config = Config::parse(&text).unwrap();
        let first = &config.capability("change").unwrap().checks[0];
        assert_eq!(
            first.skip_when_only,
            Some(vec!["docs/".to_string(), "*.json".to_string()])
        );
    }

    #[test]
    fn unknown_keys_are_rejected() {
        let text = format!("{REPO_CONFIG}\n[surprise]\nkey = 1\n");
        assert!(matches!(Config::parse(&text), Err(ConfigError::Parse(_))));
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
