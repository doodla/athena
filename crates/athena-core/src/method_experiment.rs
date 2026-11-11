//! Bounded baseline-versus-candidate experiments for changes to engineering methods.
//!
//! An experiment names the method, a baseline arm and a candidate arm of measured
//! runs, a bound on how many runs it may spend, and which direction of the metric is
//! better. An experiment with no method, a missing arm or a zero bound is refused;
//! one that spends more runs than its bound is stopped; otherwise the candidate is
//! judged better, worse or no different than the baseline.

/// Which direction of the metric counts as an improvement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    LowerIsBetter,
    HigherIsBetter,
}

/// A bounded experiment comparing a candidate method with its baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct MethodExperiment {
    pub method: String,
    pub baseline: Vec<f64>,
    pub candidate: Vec<f64>,
    /// The most runs (both arms together) the experiment may spend.
    pub bound: usize,
    pub direction: Direction,
}

/// Why an experiment was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    EmptyMethod,
    MissingBaseline,
    MissingCandidate,
    ZeroBound,
}

/// The candidate's standing against the baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Better,
    Worse,
    NoDifference,
}

/// The outcome of comparing the two arms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Refused(Refusal),
    /// Runs spent exceeded the bound.
    Stopped,
    Verdict(Verdict),
}

/// Refuse an experiment that is unbounded or incomplete.
pub fn validate(e: &MethodExperiment) -> Result<(), Refusal> {
    if e.method.trim().is_empty() {
        return Err(Refusal::EmptyMethod);
    }
    if e.baseline.is_empty() {
        return Err(Refusal::MissingBaseline);
    }
    if e.candidate.is_empty() {
        return Err(Refusal::MissingCandidate);
    }
    if e.bound == 0 {
        return Err(Refusal::ZeroBound);
    }
    Ok(())
}

fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len() as f64
}

/// Validate, enforce the bound, then compare the arm means.
pub fn compare(e: &MethodExperiment) -> Outcome {
    if let Err(r) = validate(e) {
        return Outcome::Refused(r);
    }
    if e.baseline.len() + e.candidate.len() > e.bound {
        return Outcome::Stopped;
    }
    let (b, c) = (mean(&e.baseline), mean(&e.candidate));
    let verdict = if (c - b).abs() < f64::EPSILON {
        Verdict::NoDifference
    } else if (c < b) == (e.direction == Direction::LowerIsBetter) {
        Verdict::Better
    } else {
        Verdict::Worse
    };
    Outcome::Verdict(verdict)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exp(baseline: &[f64], candidate: &[f64], bound: usize, d: Direction) -> MethodExperiment {
        MethodExperiment {
            method: "review order".into(),
            baseline: baseline.to_vec(),
            candidate: candidate.to_vec(),
            bound,
            direction: d,
        }
    }

    #[test]
    fn zero_bound_is_refused() {
        let e = exp(&[1.0], &[2.0], 0, Direction::LowerIsBetter);
        assert_eq!(compare(&e), Outcome::Refused(Refusal::ZeroBound));
    }

    #[test]
    fn empty_method_is_refused() {
        let mut e = exp(&[1.0], &[2.0], 4, Direction::LowerIsBetter);
        e.method = "  ".into();
        assert_eq!(compare(&e), Outcome::Refused(Refusal::EmptyMethod));
    }

    #[test]
    fn missing_arm_is_refused() {
        let e = exp(&[], &[2.0], 4, Direction::LowerIsBetter);
        assert_eq!(compare(&e), Outcome::Refused(Refusal::MissingBaseline));
        let e = exp(&[1.0], &[], 4, Direction::LowerIsBetter);
        assert_eq!(compare(&e), Outcome::Refused(Refusal::MissingCandidate));
    }

    #[test]
    fn runs_beyond_bound_are_stopped() {
        let e = exp(&[1.0, 1.0], &[2.0, 2.0], 3, Direction::LowerIsBetter);
        assert_eq!(compare(&e), Outcome::Stopped);
    }

    #[test]
    fn verdicts_follow_direction() {
        let lower = Direction::LowerIsBetter;
        let higher = Direction::HigherIsBetter;
        let v = |e| compare(&e);
        assert_eq!(
            v(exp(&[5.0], &[3.0], 2, lower)),
            Outcome::Verdict(Verdict::Better)
        );
        assert_eq!(
            v(exp(&[3.0], &[5.0], 2, lower)),
            Outcome::Verdict(Verdict::Worse)
        );
        assert_eq!(
            v(exp(&[3.0], &[5.0], 2, higher)),
            Outcome::Verdict(Verdict::Better)
        );
        assert_eq!(
            v(exp(&[5.0], &[3.0], 2, higher)),
            Outcome::Verdict(Verdict::Worse)
        );
        assert_eq!(
            v(exp(&[4.0], &[4.0], 2, lower)),
            Outcome::Verdict(Verdict::NoDifference)
        );
    }
}
