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
}

impl BudgetSet {
    /// The budget, in tokens.
    pub fn budget(self) -> u64 {
        match self {
            BudgetSet::Contract => 350,
            BudgetSet::Reader => 1_800,
            BudgetSet::Producer => 1_100,
            BudgetSet::Verifier => 1_300,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            BudgetSet::Contract => "contract",
            BudgetSet::Reader => "reader",
            BudgetSet::Producer => "producer",
            BudgetSet::Verifier => "verifier",
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

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
