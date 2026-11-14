//! OLY-B-S7-01: the study maps all three systems onto shared concepts, compares them mechanism by mechanism,
//! and closes with a short essay on building orchestration from prompts.
//!
//! [`check`] is a pure function over one [`Comparison`] record. It reads and runs nothing. It reports a gap for
//! each concept or mechanism comparison that lacks an entry for one of the three systems in
//! [`crate::lab::SYSTEMS`], for an empty list of concepts or of comparisons, and for an essay that is absent,
//! blank, longer than [`ESSAY_WORD_CAP`] words, or not about prompts and orchestration. Gaps come out in a fixed
//! order: concepts, comparisons, essay.
//!
//! Limits:
//! - An entry counts when it names one of the three systems (ignoring case) and has non-blank text; whether the
//!   text is right is not judged.
//! - A concept or mechanism the study leaves out is not detected; only the ones listed are checked.
//! - "Short" means at most `ESSAY_WORD_CAP` (800) whitespace-separated words.
//! - The topic is a case-insensitive match for both `prompt` and `orchestrat` in the essay text; a good essay
//!   that avoids those words fails and a poor one that uses them passes.
//! - Nothing yet feeds `check` a real three-system study, so the spec measure ends `status: not shown`.

use crate::lab::SYSTEMS;

/// The most whitespace-separated words a "short" essay may have.
pub const ESSAY_WORD_CAP: usize = 800;

/// Terms an essay on building orchestration from prompts must contain (case-insensitive).
pub const ESSAY_TERMS: [&str; 2] = ["prompt", "orchestrat"];

/// What one system does or means for a concept or mechanism.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The system's name, one of the three in [`SYSTEMS`].
    pub system: String,
    /// The mapping or comparison text; blank means none.
    pub text: String,
}

/// A shared concept with each system's mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Concept {
    /// The concept's name.
    pub name: String,
    /// One entry per system.
    pub entries: Vec<Entry>,
}

/// A mechanism compared across the systems.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MechanismComparison {
    /// The mechanism's name.
    pub mechanism: String,
    /// One entry per system.
    pub entries: Vec<Entry>,
}

/// The closing essay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Essay {
    /// The essay text.
    pub text: String,
}

/// A study's comparison of the three systems.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comparison {
    /// The shared concepts.
    pub concepts: Vec<Concept>,
    /// The mechanism-by-mechanism comparisons.
    pub comparisons: Vec<MechanismComparison>,
    /// The closing essay, if any.
    pub essay: Option<Essay>,
}

/// One way a comparison falls short of the requirement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gap {
    /// The study lists no shared concepts.
    NoConcepts,
    /// A concept has no usable entry for a system.
    ConceptMissingSystem {
        /// The concept's name.
        concept: String,
        /// The system with no entry.
        system: String,
    },
    /// The study lists no mechanism comparisons.
    NoComparisons,
    /// A mechanism comparison has no usable entry for a system.
    ComparisonMissingSystem {
        /// The mechanism's name.
        mechanism: String,
        /// The system with no entry.
        system: String,
    },
    /// The essay is absent or blank.
    NoEssay,
    /// The essay has more than [`ESSAY_WORD_CAP`] words.
    EssayTooLong {
        /// The essay's word count.
        words: usize,
        /// The cap it exceeds.
        cap: usize,
    },
    /// The essay does not mention both prompts and orchestration.
    EssayOffTopic,
}

fn covers(entries: &[Entry], system: &str) -> bool {
    entries
        .iter()
        .any(|e| e.system.eq_ignore_ascii_case(system) && !e.text.trim().is_empty())
}

/// Check a comparison against OLY-B-S7-01; an empty result means no gap.
pub fn check(comparison: &Comparison) -> Vec<Gap> {
    let mut gaps = Vec::new();
    if comparison.concepts.is_empty() {
        gaps.push(Gap::NoConcepts);
    }
    for c in &comparison.concepts {
        for system in SYSTEMS {
            if !covers(&c.entries, system) {
                gaps.push(Gap::ConceptMissingSystem {
                    concept: c.name.clone(),
                    system: system.to_string(),
                });
            }
        }
    }
    if comparison.comparisons.is_empty() {
        gaps.push(Gap::NoComparisons);
    }
    for m in &comparison.comparisons {
        for system in SYSTEMS {
            if !covers(&m.entries, system) {
                gaps.push(Gap::ComparisonMissingSystem {
                    mechanism: m.mechanism.clone(),
                    system: system.to_string(),
                });
            }
        }
    }
    match &comparison.essay {
        Some(essay) if !essay.text.trim().is_empty() => {
            let words = essay.text.split_whitespace().count();
            if words > ESSAY_WORD_CAP {
                gaps.push(Gap::EssayTooLong {
                    words,
                    cap: ESSAY_WORD_CAP,
                });
            }
            let lower = essay.text.to_lowercase();
            if !ESSAY_TERMS.iter().all(|t| lower.contains(t)) {
                gaps.push(Gap::EssayOffTopic);
            }
        }
        _ => gaps.push(Gap::NoEssay),
    }
    gaps
}
