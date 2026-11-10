//! Dionysus exploration: a bounded experiment.
//!
//! An experiment states a hypothesis, an unconventional approach and a hard
//! bound on the effort it may spend. An experiment without a hypothesis,
//! approach or bound is refused, and one that spends past its bound is stopped.

/// How an experiment ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Not run yet.
    Pending,
    /// Finished within its bound.
    Completed,
    /// Stopped because it spent past its bound.
    Stopped,
}

/// One Dionysus experiment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Experiment {
    /// What the experiment tests.
    pub hypothesis: String,
    /// The unconventional approach tried.
    pub approach: String,
    /// Hard bound on the effort, in steps. Zero means no bound.
    pub bound: u32,
    /// How it ended.
    pub outcome: Outcome,
}

/// Why an experiment is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The hypothesis is empty.
    EmptyHypothesis,
    /// The approach is empty.
    EmptyApproach,
    /// There is no bound.
    Unbounded,
}

/// Accept an experiment only if it has a hypothesis, an approach and a bound.
pub fn validate(e: &Experiment) -> Result<(), Refusal> {
    if e.hypothesis.trim().is_empty() {
        return Err(Refusal::EmptyHypothesis);
    }
    if e.approach.trim().is_empty() {
        return Err(Refusal::EmptyApproach);
    }
    if e.bound == 0 {
        return Err(Refusal::Unbounded);
    }
    Ok(())
}

/// Run an accepted experiment that spent `spent` steps and report its outcome:
/// `Stopped` when it exceeded the bound, otherwise `Completed`.
pub fn conclude(e: &Experiment, spent: u32) -> Result<Outcome, Refusal> {
    validate(e)?;
    Ok(if spent > e.bound {
        Outcome::Stopped
    } else {
        Outcome::Completed
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exp(h: &str, a: &str, bound: u32) -> Experiment {
        Experiment {
            hypothesis: h.into(),
            approach: a.into(),
            bound,
            outcome: Outcome::Pending,
        }
    }

    #[test]
    fn exploration_refuses_unbounded() {
        assert_eq!(validate(&exp("h", "a", 0)), Err(Refusal::Unbounded));
    }

    #[test]
    fn exploration_refuses_empty_hypothesis() {
        assert_eq!(validate(&exp("  ", "a", 3)), Err(Refusal::EmptyHypothesis));
    }

    #[test]
    fn exploration_refuses_empty_approach() {
        assert_eq!(validate(&exp("h", "", 3)), Err(Refusal::EmptyApproach));
    }

    #[test]
    fn exploration_accepts_bounded() {
        let e = exp("h", "a", 3);
        assert_eq!(validate(&e), Ok(()));
        assert_eq!(conclude(&e, 3), Ok(Outcome::Completed));
    }

    #[test]
    fn exploration_reports_exceeded_bound_as_stopped() {
        assert_eq!(conclude(&exp("h", "a", 3), 4), Ok(Outcome::Stopped));
        assert_eq!(conclude(&exp("h", "a", 0), 1), Err(Refusal::Unbounded));
    }
}
