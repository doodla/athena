//! Context composition for Athena's agents (design §4, §12).
//!
//! Every agent's context is a composed, budgeted artifact:
//!
//! - prompt assets are compiled into the binary, each with an id, a version, and a SHA-256;
//! - a system prompt is the concatenation of named components in a fixed order: stable text first (the
//!   contract, the role frame, the capability's section), so prompt caching can reuse it;
//! - each role's whole always-loaded set has a fixed token budget, checked by a byte estimator as a
//!   tripwire, with live measurement as the authority;
//! - briefs carry the job-specific part, built deterministically from records ([`briefs`]);
//! - a study's producer and verifier briefs also carry the pin Athena resolved (url, ref, full SHA) in a
//!   "Pinned source" section, shown only when the job pinned a repository;
//! - the three roles' structured outputs are typed, and their JSON Schemas are generated from the types
//!   ([`outputs`]);
//! - the verifier's inputs are a type with no channel for anything the producer said ([`briefs::VerifierInputs`]).

pub mod briefs;
pub mod outputs;

use athena_data::records::ContextComponent;
use athena_data::{Quantity, Unit, sha256_hex};

/// A prompt text compiled into the binary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromptAsset {
    pub id: &'static str,
    pub version: u32,
    pub text: &'static str,
}

/// The contract every agent loads first, whatever its role (design §12).
pub const CONTRACT: PromptAsset = PromptAsset {
    id: "contract",
    version: 1,
    text: include_str!("../prompts/contract.md"),
};
/// The reader's role frame.
pub const READER_FRAME: PromptAsset = PromptAsset {
    id: "frame.reader",
    version: 1,
    text: include_str!("../prompts/reader.md"),
};
/// The producer's role frame.
pub const PRODUCER_FRAME: PromptAsset = PromptAsset {
    id: "frame.producer",
    version: 1,
    text: include_str!("../prompts/producer.md"),
};
/// The verifier's role frame.
pub const VERIFIER_FRAME: PromptAsset = PromptAsset {
    id: "frame.verifier",
    version: 1,
    text: include_str!("../prompts/verifier.md"),
};
/// The job planner's role frame.
pub const PLANNER_FRAME: PromptAsset = PromptAsset {
    id: "frame.planner",
    version: 1,
    text: include_str!("../prompts/planner.md"),
};
/// The plan critic's role frame.
pub const CRITIC_FRAME: PromptAsset = PromptAsset {
    id: "frame.critic",
    version: 1,
    text: include_str!("../prompts/critic.md"),
};

/// The basis recorded for estimated token counts.
pub const ESTIMATE_BASIS: &str = "ceil(bytes/3.0), prose tripwire";

/// Estimates tokens as ⌈bytes / 3.0⌉. For prose like Athena's prompts it errs high (the contract measured
/// 3.3 bytes per token); it is a tripwire, not a measurement.
pub fn estimate_tokens(text: &str) -> u64 {
    (text.len() as f64 / 3.0).ceil() as u64
}

/// One component of a composed prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub id: String,
    pub version: u32,
    pub text: String,
}

impl Component {
    pub fn new(id: impl Into<String>, version: u32, text: impl Into<String>) -> Self {
        Component {
            id: id.into(),
            version,
            text: text.into(),
        }
    }

    /// The record of this component: id, version, hash, and estimated tokens.
    pub fn record(&self) -> ContextComponent {
        ContextComponent {
            id: self.id.clone(),
            version: self.version,
            sha256: sha256_hex(self.text.as_bytes()),
            tokens: estimated_tokens(&self.text),
        }
    }
}

impl From<PromptAsset> for Component {
    fn from(asset: PromptAsset) -> Self {
        Component::new(asset.id, asset.version, asset.text)
    }
}

fn estimated_tokens(text: &str) -> Quantity {
    Quantity::estimated(
        Some(estimate_tokens(text) as f64),
        Unit::Tokens,
        ESTIMATE_BASIS,
        None,
        None,
    )
    .expect("a token estimate is a finite, non-negative value")
}

/// A composed system prompt and the record of its parts.
#[derive(Debug, Clone, PartialEq)]
pub struct Composed {
    pub text: String,
    pub components: Vec<ContextComponent>,
}

impl Composed {
    pub fn sha256(&self) -> String {
        sha256_hex(self.text.as_bytes())
    }
    pub fn tokens(&self) -> Quantity {
        estimated_tokens(&self.text)
    }
    pub fn estimated_tokens(&self) -> u64 {
        estimate_tokens(&self.text)
    }
}

/// Concatenates components in the given order, separated by blank lines.
pub fn compose(components: &[Component]) -> Composed {
    let text = components
        .iter()
        .map(|c| c.text.trim_end())
        .collect::<Vec<_>>()
        .join("\n\n")
        + "\n";
    Composed {
        text,
        components: components.iter().map(Component::record).collect(),
    }
}

/// The always-loaded sets that carry fixed budgets (design §12, D25).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetSet {
    Contract,
    Reader,
    Producer,
    Verifier,
    /// The contract and the steward's stored prompt (`athena_data::roles::STEWARD_PROMPT`).
    Steward,
    /// The contract and the audience's stored prompt (`athena_data::roles::AUDIENCE_PROMPT`).
    Audience,
    /// The contract and the auditor's stored prompt (`athena_data::roles::AUDITOR_PROMPT`).
    Auditor,
}

impl BudgetSet {
    /// The budget, in tokens.
    pub fn budget(self) -> u64 {
        match self {
            BudgetSet::Contract => 350,
            BudgetSet::Reader => 2_000,
            BudgetSet::Producer => 1_300,
            BudgetSet::Verifier => 1_300,
            BudgetSet::Steward => 600,
            BudgetSet::Audience => 600,
            BudgetSet::Auditor => 600,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            BudgetSet::Contract => "contract",
            BudgetSet::Reader => "reader",
            BudgetSet::Producer => "producer",
            BudgetSet::Verifier => "verifier",
            BudgetSet::Steward => "steward",
            BudgetSet::Audience => "audience",
            BudgetSet::Auditor => "auditor",
        }
    }
}

/// The auditor's system prompt: the contract, then the role's stored prompt.
pub fn auditor_system_prompt(prompt: &str, version: u32) -> Composed {
    compose(&[
        CONTRACT.into(),
        Component::new("auditor.prompt", version, prompt),
    ])
}

/// The audience's system prompt: the contract, then the role's stored prompt.
pub fn audience_system_prompt(prompt: &str, version: u32) -> Composed {
    compose(&[
        CONTRACT.into(),
        Component::new("audience.prompt", version, prompt),
    ])
}

/// The steward's system prompt: the contract, then the role's stored prompt. The stored prompt is the
/// role definition's, so a change to it is a new recorded version of the role.
pub fn steward_system_prompt(prompt: &str, version: u32) -> Composed {
    compose(&[
        CONTRACT.into(),
        Component::new("steward.prompt", version, prompt),
    ])
}

/// Checks a composed set against its budget with the estimator.
pub fn check_budget(set: BudgetSet, composed: &Composed) -> Result<u64, String> {
    let estimate = composed.estimated_tokens();
    if estimate > set.budget() {
        return Err(format!(
            "the {} set is about {estimate} tokens ({ESTIMATE_BASIS}), over its budget of {}; pay for additions with removals",
            set.name(),
            set.budget()
        ));
    }
    Ok(estimate)
}

/// How much of one budgeted prompt set is used: which components it composes and what is left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadroomRow {
    /// The set's name, such as `reader` or `verifier (change)`.
    pub set: String,
    /// The ids of the components composed into the set, in order.
    pub components: Vec<String>,
    /// The estimated tokens of the composed set.
    pub used: u64,
    /// The set's budget, in tokens.
    pub budget: u64,
}

impl HeadroomRow {
    /// The tokens left under the budget; zero when the set is over.
    pub fn left(&self) -> u64 {
        self.budget.saturating_sub(self.used)
    }
}

/// The token headroom of every budgeted set a prompt edit can touch, computed with [`compose`] and the
/// budgets of [`BudgetSet`], the same figures the budgets test checks. Rows: contract, reader, planner,
/// critic, then a producer and a verifier set for each capability as `(id, producer section, verifier
/// section)`. The planner runs on the producer's budget and the critic on the verifier's. A file listed in
/// several rows (the contract, a role frame) costs every one of those sets when it grows.
pub fn headroom_rows(
    defaults: Component,
    catalog: Component,
    capabilities: &[(String, PromptAsset, PromptAsset)],
) -> Vec<HeadroomRow> {
    let row = |name: String, set: BudgetSet, components: Vec<Component>| {
        let composed = compose(&components);
        HeadroomRow {
            set: name,
            components: components.into_iter().map(|c| c.id).collect(),
            used: composed.estimated_tokens(),
            budget: set.budget(),
        }
    };
    let mut rows = vec![
        row(
            "contract".into(),
            BudgetSet::Contract,
            vec![CONTRACT.into()],
        ),
        row(
            "reader".into(),
            BudgetSet::Reader,
            vec![CONTRACT.into(), READER_FRAME.into(), defaults, catalog],
        ),
        row(
            "planner".into(),
            BudgetSet::Producer,
            vec![CONTRACT.into(), PLANNER_FRAME.into()],
        ),
        row(
            "critic".into(),
            BudgetSet::Verifier,
            vec![CONTRACT.into(), CRITIC_FRAME.into()],
        ),
    ];
    for (id, producer, verifier) in capabilities {
        rows.push(row(
            format!("producer ({id})"),
            BudgetSet::Producer,
            vec![CONTRACT.into(), PRODUCER_FRAME.into(), (*producer).into()],
        ));
        rows.push(row(
            format!("verifier ({id})"),
            BudgetSet::Verifier,
            vec![CONTRACT.into(), VERIFIER_FRAME.into(), (*verifier).into()],
        ));
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headroom_rows_name_shared_components_in_every_set_that_composes_them() {
        let section = PromptAsset {
            id: "capability.x.verifier",
            version: 1,
            text: "short",
        };
        let rows = headroom_rows(
            Component::new("defaults", 1, "d"),
            Component::new("catalog", 1, "c"),
            &[("x".into(), section, section)],
        );
        let names: Vec<_> = rows.iter().map(|r| r.set.as_str()).collect();
        assert_eq!(
            names,
            [
                "contract",
                "reader",
                "planner",
                "critic",
                "producer (x)",
                "verifier (x)"
            ]
        );
        let verifier = rows.iter().find(|r| r.set == "verifier (x)").unwrap();
        assert_eq!(
            verifier.components,
            ["contract", "frame.verifier", "capability.x.verifier"]
        );
        assert_eq!(verifier.budget, BudgetSet::Verifier.budget());
        assert_eq!(verifier.left(), verifier.budget - verifier.used);
    }

    #[test]
    fn the_contract_is_within_its_budget() {
        // M18 (allows): the current contract passes.
        let composed = compose(&[CONTRACT.into()]);
        assert!(check_budget(BudgetSet::Contract, &composed).is_ok());
    }

    #[test]
    fn an_over_budget_contract_fails() {
        // M18 (blocks): an over-budget contract fails.
        let padded = format!(
            "{}\n9. {}\n",
            CONTRACT.text,
            "Another rule that was not paid for. ".repeat(10)
        );
        let composed = compose(&[Component::new("contract", 2, padded)]);
        assert!(check_budget(BudgetSet::Contract, &composed).is_err());
    }

    #[test]
    fn the_contract_keeps_one_rule_per_line_as_measured() {
        // The measured 294 tokens were for 977 bytes with one rule per line (design §12, E8).
        assert_eq!(CONTRACT.text.len(), 977);
        assert_eq!(
            CONTRACT
                .text
                .lines()
                .filter(|l| l.starts_with(char::is_numeric))
                .count(),
            8
        );
    }

    #[test]
    fn producer_and_verifier_frames_leave_room_for_capability_sections() {
        // The capability's section is checked with the whole set where capabilities are registered.
        for (set, frame) in [
            (BudgetSet::Producer, PRODUCER_FRAME),
            (BudgetSet::Verifier, VERIFIER_FRAME),
            (BudgetSet::Reader, READER_FRAME),
        ] {
            let composed = compose(&[CONTRACT.into(), frame.into()]);
            assert!(check_budget(set, &composed).is_ok(), "{} frame", set.name());
        }
    }

    #[test]
    fn planner_and_critic_frames_are_short_and_fit_the_budgets() {
        // The planner works on the producer's model and the critic on the verifier's.
        for (set, frame) in [
            (BudgetSet::Producer, PLANNER_FRAME),
            (BudgetSet::Verifier, CRITIC_FRAME),
        ] {
            let composed = compose(&[CONTRACT.into(), frame.into()]);
            assert!(check_budget(set, &composed).is_ok(), "{}", frame.id);
            assert!(estimate_tokens(frame.text) < 500, "{} is short", frame.id);
        }
    }

    #[test]
    fn the_auditor_prompt_follows_the_contract_and_fits_its_budget() {
        let composed = auditor_system_prompt("You are the auditor.", 1);
        assert_eq!(composed.components[1].id, "auditor.prompt");
        assert!(check_budget(BudgetSet::Auditor, &composed).is_ok());
        let long = auditor_system_prompt(&"word ".repeat(2_000), 1);
        assert!(check_budget(BudgetSet::Auditor, &long).is_err());
    }

    #[test]
    fn the_audience_prompt_follows_the_contract_and_fits_its_budget() {
        let composed = audience_system_prompt("You are the audience.", 1);
        assert_eq!(composed.components[1].id, "audience.prompt");
        assert!(check_budget(BudgetSet::Audience, &composed).is_ok());
        let long = audience_system_prompt(&"word ".repeat(2_000), 1);
        assert!(check_budget(BudgetSet::Audience, &long).is_err());
    }

    #[test]
    fn the_steward_prompt_follows_the_contract_and_fits_its_budget() {
        let prompt = "You are the steward.";
        let composed = steward_system_prompt(prompt, 2);
        assert!(composed.text.starts_with("# Athena agent contract"));
        assert!(composed.text.contains(prompt));
        assert_eq!(composed.components[1].id, "steward.prompt");
        assert!(check_budget(BudgetSet::Steward, &composed).is_ok());
        let long = steward_system_prompt(&"word ".repeat(2_000), 2);
        assert!(check_budget(BudgetSet::Steward, &long).is_err());
    }

    #[test]
    fn reader_checks_for_governance_files_and_producer_stops_on_unlisted_ones() {
        // Guidance only: admission enforces the rule; these frames help the agents avoid a refusal.
        let reader = READER_FRAME.text;
        for wording in [
            "Check whether the outcome needs a change to athena.toml, defaults.toml",
            ".github/workflows/",
            "list that file as a deliverable and name it in an acceptance criterion",
        ] {
            assert!(reader.contains(wording), "reader frame lacks: {wording}");
        }
        let producer = PRODUCER_FRAME.text;
        for wording in [
            "governance file",
            "does not list as a deliverable",
            "stop and report it in open_issues; do not edit it",
        ] {
            assert!(
                producer.contains(wording),
                "producer frame lacks: {wording}"
            );
        }
    }

    #[test]
    fn the_verifier_frame_limits_snapshot_runs_to_the_snapshot() {
        let text = VERIFIER_FRAME.text;
        for wording in [
            "a snapshot, not the real repository",
            "no records/, no owner files, no history",
            "a result about the snapshot only",
            "say so explicitly in evidence and findings",
            "never report it as a result on the real repository",
        ] {
            assert!(text.contains(wording), "verifier frame lacks: {wording}");
        }
    }

    #[test]
    fn components_are_recorded_with_hashes_and_estimates() {
        let composed = compose(&[CONTRACT.into(), READER_FRAME.into()]);
        assert_eq!(composed.components.len(), 2);
        assert_eq!(composed.components[0].id, "contract");
        assert_eq!(
            composed.components[0].sha256,
            sha256_hex(CONTRACT.text.as_bytes())
        );
        assert!(composed.text.starts_with("# Athena agent contract"));
    }
}
