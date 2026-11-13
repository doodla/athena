//! OLY-B-S3-02: observed behavior is reconciled against what the prompts and docs say, and every divergence
//! is a first-class finding.
//!
//! [`reconcile`] is a pure function. The caller supplies the claims (what a prompt or doc says should
//! happen, with the file it was read from) and the observations (what happened). It compares them by key and
//! returns one [`Divergence`] for each mismatch; nothing is dropped, and nothing is read or run here.

use std::collections::BTreeMap;

/// A statement in a prompt or doc about what should happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    /// The key that ties the claim to an observation.
    pub key: String,
    /// The file the claim was read from.
    pub source: String,
    /// The claim as written.
    pub text: String,
    /// The value the claim says to expect.
    pub expected: String,
}

/// A fact seen in a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observation {
    /// The key of the claim it bears on.
    pub key: String,
    /// The value seen.
    pub observed: String,
}

/// A finding: observed behavior and the documented claim do not agree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Divergence {
    /// Both exist and the values differ.
    Mismatch { claim: Claim, observed: String },
    /// A claim has no observation, so it was not checked.
    Unobserved { claim: Claim },
    /// An observation has no claim: behavior no prompt or doc describes.
    Undocumented { key: String, observed: String },
}

impl Divergence {
    /// The key the finding is about.
    pub fn key(&self) -> &str {
        match self {
            Divergence::Mismatch { claim, .. } | Divergence::Unobserved { claim } => &claim.key,
            Divergence::Undocumented { key, .. } => key,
        }
    }
}

/// Compares claims with observations and returns every divergence: claims in order, then undocumented
/// observations in key order. When a key repeats among observations, each observation is compared.
pub fn reconcile(claims: &[Claim], observations: &[Observation]) -> Vec<Divergence> {
    let mut by_key: BTreeMap<&str, Vec<&Observation>> = BTreeMap::new();
    for observation in observations {
        by_key
            .entry(&observation.key)
            .or_default()
            .push(observation);
    }
    let mut findings = Vec::new();
    for claim in claims {
        match by_key.get(claim.key.as_str()) {
            None => findings.push(Divergence::Unobserved {
                claim: claim.clone(),
            }),
            Some(seen) => {
                for observation in seen {
                    if observation.observed != claim.expected {
                        findings.push(Divergence::Mismatch {
                            claim: claim.clone(),
                            observed: observation.observed.clone(),
                        });
                    }
                }
            }
        }
    }
    for (key, seen) in by_key {
        if !claims.iter().any(|claim| claim.key == key) {
            for observation in seen {
                findings.push(Divergence::Undocumented {
                    key: key.to_string(),
                    observed: observation.observed.clone(),
                });
            }
        }
    }
    findings
}
