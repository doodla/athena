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
    /// What the experiment showed. `None` until a result is recorded.
    pub result: Option<ExperimentResult>,
}

/// What an experiment showed about its hypothesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExperimentResult {
    /// The hypothesis held.
    Supported,
    /// The hypothesis did not hold.
    Unsupported,
    /// The experiment could not tell.
    Inconclusive,
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
    /// The experiment has not concluded, so it has no result to record.
    NotConcluded,
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

/// Record the result of a validated experiment that has concluded (its outcome
/// is not `Pending`). An invalid or unconcluded experiment is refused and keeps
/// no result.
pub fn record_result(e: &mut Experiment, result: ExperimentResult) -> Result<(), Refusal> {
    validate(e)?;
    if e.outcome == Outcome::Pending {
        return Err(Refusal::NotConcluded);
    }
    e.result = Some(result);
    Ok(())
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
            result: None,
        }
    }

    #[test]
    fn exploration_records_each_result_on_concluded_experiment() {
        for r in [
            ExperimentResult::Supported,
            ExperimentResult::Unsupported,
            ExperimentResult::Inconclusive,
        ] {
            let mut e = exp("h", "a", 3);
            e.outcome = conclude(&e, 2).unwrap();
            assert_eq!(e.result, None);
            assert_eq!(record_result(&mut e, r), Ok(()));
            assert_eq!(e.result, Some(r));
        }
    }

    #[test]
    fn exploration_refuses_result_when_unconcluded_or_invalid() {
        let mut e = exp("h", "a", 3);
        assert_eq!(
            record_result(&mut e, ExperimentResult::Supported),
            Err(Refusal::NotConcluded)
        );
        assert_eq!(e.result, None);
        let mut u = exp("h", "a", 0);
        u.outcome = Outcome::Completed;
        assert_eq!(
            record_result(&mut u, ExperimentResult::Supported),
            Err(Refusal::Unbounded)
        );
        assert_eq!(u.result, None);
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
