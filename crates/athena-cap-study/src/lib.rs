//! `study`: Athena's studying capability (design: `docs/design/study-capability-m1.md`).
//!
//! This crate is the reusable validation core: the typed study records ([`records`], [`model`]) and the
//! validation rules of design §5.4 ([`rules`]), plus the [`Capability`] impl with its prompts. It does not
//! run any studied system. It is registered in the CLI wiring, so the reader can pick `study`. Containment, the run step, replay,
//! and publishing are later jobs (see D65 in `docs/decisions.md`).

pub mod model;
pub mod records;
pub mod rules;

use athena_context::PromptAsset;
use athena_core::capability::{
    Capability, CapabilityCard, ChangeList, CheckSpec, JobConfig, ToolGrant,
};
use athena_data::config::CapabilityConfig;
use athena_data::records::PolicyViolation;
use std::collections::BTreeMap;

/// The capability's id in records and in the reader's catalog.
pub const ID: &str = "study";

const PRODUCER_SECTION: PromptAsset = PromptAsset {
    id: "capability.study.producer",
    version: 1,
    text: include_str!("../prompts/producer.md"),
};
const VERIFIER_SECTION: PromptAsset = PromptAsset {
    id: "capability.study.verifier",
    version: 1,
    text: include_str!("../prompts/verifier.md"),
};

/// The `study` capability.
#[derive(Debug, Default, Clone, Copy)]
pub struct Study;

fn section(config: &JobConfig) -> Option<&CapabilityConfig> {
    config.config.capability(ID)
}

impl Capability for Study {
    fn id(&self) -> &'static str {
        ID
    }

    fn card(&self) -> CapabilityCard {
        CapabilityCard {
            id: ID,
            purpose: "study how a system works at one pinned commit, as cited claims and a model that code validates.",
            when: "for a request to study, model, or explain a system's design from its repository.",
            produces: "a validated study of typed records, or a rejection with findings.",
        }
    }

    fn producer_section(&self) -> PromptAsset {
        PRODUCER_SECTION
    }

    fn verifier_section(&self) -> PromptAsset {
        VERIFIER_SECTION
    }

    fn producer_grant(&self, config: &JobConfig) -> ToolGrant {
        ToolGrant {
            tools: ["Read", "Grep", "Glob", "Edit", "Write", "Bash"]
                .map(String::from)
                .to_vec(),
            commands: section(config)
                .map(|c| c.producer_commands.clone())
                .unwrap_or_default(),
            writes: true,
            permission_mode: "acceptEdits".into(),
        }
    }

    fn verifier_grant(&self, config: &JobConfig) -> ToolGrant {
        ToolGrant {
            tools: ["Read", "Grep", "Glob", "Bash"].map(String::from).to_vec(),
            commands: section(config)
                .map(|c| c.verifier_commands.clone())
                .unwrap_or_default(),
            writes: false,
            permission_mode: "dontAsk".into(),
        }
    }

    fn checks(&self, config: &JobConfig) -> Vec<CheckSpec> {
        section(config)
            .map(|c| {
                c.checks
                    .iter()
                    .map(|k| CheckSpec {
                        name: k.name.clone(),
                        run: k.run.clone(),
                        kind: k.kind,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    fn env(&self, config: &JobConfig) -> BTreeMap<String, String> {
        section(config).map(|c| c.env.clone()).unwrap_or_default()
    }

    fn admit(&self, change: &ChangeList, _config: &JobConfig) -> Vec<PolicyViolation> {
        change
            .paths()
            .filter(|p| p.starts_with("target/") || p.contains("/target/"))
            .map(|p| PolicyViolation {
                rule: "build-output".into(),
                path: Some(p.to_string()),
                detail: format!(
                    "{p} looks like build output, which never belongs in the repository"
                ),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::records::{ChangeEntry, ChangeStatus};

    fn job_config() -> JobConfig {
        JobConfig {
            config: athena_data::Config::parse(include_str!("../../../athena.toml")).unwrap(),
            defaults: athena_data::DefaultsModel::parse(include_str!("../../../defaults.toml"))
                .unwrap(),
            commit: "x".into(),
            config_sha256: String::new(),
            defaults_sha256: String::new(),
        }
    }

    #[test]
    fn the_capability_identifies_itself_with_prompts() {
        let study = Study;
        assert_eq!(study.id(), "study");
        assert_eq!(study.card().id, "study");
        assert_eq!(study.producer_section().id, "capability.study.producer");
        assert_eq!(study.verifier_section().id, "capability.study.verifier");
        assert!(
            study
                .producer_section()
                .text
                .starts_with("## Capability: study")
        );
        assert!(
            study
                .verifier_section()
                .text
                .starts_with("## Capability: study")
        );
    }

    #[test]
    fn the_verifier_never_writes_and_grants_and_checks_come_from_the_config() {
        let jc = job_config();
        assert!(!Study.verifier_grant(&jc).writes);
        assert!(Study.producer_grant(&jc).writes);
        // The capability is configured in athena.toml, and its grants and checks come from there.
        let section = jc.config.capability(ID).expect("[capabilities.study]");
        assert_eq!(
            Study.producer_grant(&jc).commands,
            section.producer_commands
        );
        assert_eq!(
            Study.verifier_grant(&jc).commands,
            section.verifier_commands
        );
        assert!(!section.producer_commands.is_empty());
        assert!(!section.verifier_commands.is_empty());
        let names: Vec<String> = Study.checks(&jc).into_iter().map(|c| c.name).collect();
        assert_eq!(names, ["fmt", "clippy", "test"]);
        assert_eq!(Study.env(&jc), section.env);
    }

    #[test]
    fn build_output_is_refused() {
        let entry = |path: &str| ChangeEntry {
            path: path.into(),
            status: ChangeStatus::Added,
            mode: Some("100644".into()),
            bytes: Some(1),
        };
        let change = ChangeList {
            entries: vec![entry("target/debug/x"), entry("studies/s1/claims.json")],
        };
        let violations = Study.admit(&change, &job_config());
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].path.as_deref(), Some("target/debug/x"));
    }
}
