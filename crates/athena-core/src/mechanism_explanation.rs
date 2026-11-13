//! OLY-B-S4-01: for each instance the study explains how each mechanism works through the prompts, code and
//! observed runs, using Athena's own records as primary evidence for Athena.
//!
//! [`check`] is a pure function over explanation records. It reads and runs nothing. For each explanation it
//! reports a [`Gap::MissingEvidence`] for every absent prompt, code or run source, and, for the Athena instance,
//! a [`Gap::NoOwnRecords`] when no source is one of Athena's own records. A source counts only when it is not
//! blank.
//!
//! Limits:
//! - The presence of one record source stands in for "primary"; the check does not weigh how much of the
//!   evidence is records.
//! - A mechanism with no explanation at all is not detected: `check(&[])` is empty.
//! - Nothing yet feeds `check` a real study of Athena (none exists), so the spec measure ends `status: not shown`.

/// The instance whose explanations must also cite Athena's own records.
pub const ATHENA_INSTANCE: &str = "athena";

/// One kind of evidence an explanation can rest on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceKind {
    /// A prompt the mechanism runs under.
    Prompt,
    /// Code that implements the mechanism.
    Code,
    /// An observed run of the mechanism.
    Run,
    /// One of Athena's own records.
    Record,
}

/// One source cited by an explanation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    /// What kind of source this is.
    pub kind: EvidenceKind,
    /// A path, id or quote naming the source; blank means no source.
    pub source: String,
}

/// An explanation of how one mechanism of one instance works.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Explanation {
    /// The studied system, such as `athena`.
    pub instance: String,
    /// The mechanism explained.
    pub mechanism: String,
    /// The sources the explanation cites.
    pub evidence: Vec<Evidence>,
}

/// A defect in an explanation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Gap {
    /// The explanation cites no prompt, code or run source (`kind` says which).
    MissingEvidence {
        /// The instance of the explanation.
        instance: String,
        /// The mechanism of the explanation.
        mechanism: String,
        /// The absent kind: `Prompt`, `Code` or `Run`.
        kind: EvidenceKind,
    },
    /// An Athena explanation cites none of Athena's own records.
    NoOwnRecords {
        /// The instance of the explanation.
        instance: String,
        /// The mechanism of the explanation.
        mechanism: String,
    },
}

fn has(e: &Explanation, kind: EvidenceKind) -> bool {
    e.evidence
        .iter()
        .any(|v| v.kind == kind && !v.source.trim().is_empty())
}

/// Returns the gaps of `explanations`, in explanation order: prompt, code and run gaps first, then the
/// records gap.
pub fn check(explanations: &[Explanation]) -> Vec<Gap> {
    let mut gaps = Vec::new();
    for e in explanations {
        for kind in [EvidenceKind::Prompt, EvidenceKind::Code, EvidenceKind::Run] {
            if !has(e, kind) {
                gaps.push(Gap::MissingEvidence {
                    instance: e.instance.clone(),
                    mechanism: e.mechanism.clone(),
                    kind,
                });
            }
        }
        if e.instance.eq_ignore_ascii_case(ATHENA_INSTANCE) && !has(e, EvidenceKind::Record) {
            gaps.push(Gap::NoOwnRecords {
                instance: e.instance.clone(),
                mechanism: e.mechanism.clone(),
            });
        }
    }
    gaps
}
