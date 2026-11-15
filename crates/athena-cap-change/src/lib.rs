//! `change`: v0's one capability (design §6.2).
//!
//! A focused change to this repository's code, tests, docs, or configuration, made by one producer in a
//! detached snapshot and verified independently. Its checks, Bash allow-lists, and environment come from
//! `[capabilities.change]` in `athena.toml`, read from the job's base commit.

use athena_context::PromptAsset;
use athena_core::capability::{
    Capability, CapabilityCard, ChangeList, CheckSpec, JobConfig, ToolGrant,
};
use athena_data::config::CapabilityConfig;
use athena_data::records::PolicyViolation;
use std::collections::BTreeMap;

/// The capability's id in records and in the reader's catalog.
pub const ID: &str = "change";

const PRODUCER_SECTION: PromptAsset = PromptAsset {
    id: "capability.change.producer",
    version: 1,
    text: include_str!("../prompts/producer.md"),
};
const VERIFIER_SECTION: PromptAsset = PromptAsset {
    id: "capability.change.verifier",
    version: 1,
    text: include_str!("../prompts/verifier.md"),
};

/// The `change` capability.
#[derive(Debug, Default, Clone, Copy)]
pub struct Change;

fn section(config: &JobConfig) -> Option<&CapabilityConfig> {
    config.config.capability(ID)
}

impl Capability for Change {
    fn id(&self) -> &'static str {
        ID
    }

    fn card(&self) -> CapabilityCard {
        CapabilityCard {
            id: ID,
            purpose: "a focused change to this repository's code, tests, docs, or configuration, made by one producer and verified independently.",
            when: "for any request that is satisfied by editing files in this repository.",
            produces: "one landed commit, or a rejection with the verifier's findings.",
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
                        skip_when_only: k.skip_when_only.clone(),
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

    #[test]
    fn build_output_is_refused() {
        let change = ChangeList {
            entries: vec![
                ChangeEntry {
                    path: "target/debug/x".into(),
                    status: ChangeStatus::Added,
                    mode: Some("100644".into()),
                    bytes: Some(1),
                },
                ChangeEntry {
                    path: "src/lib.rs".into(),
                    status: ChangeStatus::Modified,
                    mode: Some("100644".into()),
                    bytes: Some(1),
                },
            ],
        };
        let config = athena_data::Config::parse(include_str!("../../../athena.toml")).unwrap();
        let defaults =
            athena_data::DefaultsModel::parse(include_str!("../../../defaults.toml")).unwrap();
        let jc = JobConfig {
            config,
            defaults,
            commit: "x".into(),
            config_sha256: String::new(),
            defaults_sha256: String::new(),
        };
        let violations = Change.admit(&change, &jc);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].path.as_deref(), Some("target/debug/x"));
    }

    #[test]
    fn grants_come_from_the_configuration() {
        let config = athena_data::Config::parse(include_str!("../../../athena.toml")).unwrap();
        let defaults =
            athena_data::DefaultsModel::parse(include_str!("../../../defaults.toml")).unwrap();
        let jc = JobConfig {
            config,
            defaults,
            commit: "x".into(),
            config_sha256: String::new(),
            defaults_sha256: String::new(),
        };
        let section = jc.config.capability(ID).unwrap().clone();
        let producer = Change.producer_grant(&jc);
        assert!(producer.writes);
        assert_eq!(producer.commands, section.producer_commands);
        let verifier = Change.verifier_grant(&jc);
        assert!(!verifier.writes, "the verifier never writes");
        assert_eq!(verifier.commands, section.verifier_commands);
        for program in ["rm", "mv", "chmod", "cp", "touch", "mkdir"] {
            assert!(
                !verifier.commands.contains_key(program),
                "the verifier's list names no writing program: {program}"
            );
        }
        assert_eq!(Change.checks(&jc).len(), section.checks.len());
    }

    fn live_job_config() -> JobConfig {
        let config = athena_data::Config::parse(include_str!("../../../athena.toml")).unwrap();
        let defaults =
            athena_data::DefaultsModel::parse(include_str!("../../../defaults.toml")).unwrap();
        JobConfig {
            config,
            defaults,
            commit: "x".into(),
            config_sha256: String::new(),
            defaults_sha256: String::new(),
        }
    }

    #[test]
    fn the_producer_brief_carries_the_flaky_test_guidance() {
        const EXAMPLE: &str = "crates/athena-testkit/tests/stand_in_stress.rs";
        let text = Change.producer_section().text;
        let paragraphs: Vec<&str> = text.trim_end().split("\n\n").collect();
        let at = paragraphs
            .iter()
            .position(|p| p.starts_with("For an intermittent or racy failure"))
            .expect("the flaky-test guidance is a paragraph of its own");
        let guidance = paragraphs[at];
        for phrase in [
            "first reproduce it under load",
            "loops the action on threads and counts failures",
            EXAMPLE,
            "the Bash policy denies shell loops",
            "in code terms",
            "fix it where it starts",
            "in one shared helper",
            "finds every site",
            "fails with the cause put back",
            "Report the command and counts before and after",
        ] {
            assert!(
                guidance.contains(phrase),
                "the flaky-test guidance lost its phrase: {phrase}"
            );
        }
        assert_eq!(
            paragraphs.len(),
            at + 2,
            "exactly one paragraph follows the guidance"
        );
        assert!(
            paragraphs[at + 1]
                .starts_with("Before you report, reread the brief's criteria_check section"),
            "the report step stays the last paragraph"
        );
        let example = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(EXAMPLE);
        assert!(
            example.is_file(),
            "the prompt names {EXAMPLE}, which no longer exists; update the prompt"
        );
        let grant = Change.producer_grant(&live_job_config());
        for program in ["for", "while", "until", "seq", "sh", "bash"] {
            assert!(
                !grant.commands.contains_key(program),
                "the producer may now run {program}; the prompt says the Bash policy denies shell loops, so update the prompt"
            );
        }
    }
}
