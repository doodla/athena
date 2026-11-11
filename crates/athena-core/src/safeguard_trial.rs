//! Safeguard trial: judges a proposed safeguard on similar defects caught, regressions and overhead.
//!
//! Serves requirement OLY-9-06: Phase 3 is accepted when Olympus shows a proposed safeguard catches similar
//! defects without unacceptable regressions or overhead. The trial is a pure function over its inputs; it is
//! not wired into runs or `athena status`.

/// What the trial measured about one proposed safeguard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeguardTrial {
    /// Name of the proposed safeguard.
    pub safeguard: String,
    /// One entry per similar defect: true when the safeguard caught it.
    pub similar_defects: Vec<bool>,
    /// Known-good work the safeguard wrongly blocked.
    pub regressions: u32,
    /// Cost of the work without the safeguard (any unit, same unit as `with_safeguard_cost`).
    pub baseline_cost: u64,
    /// Cost of the same work with the safeguard.
    pub with_safeguard_cost: u64,
}

/// The limits a safeguard must stay within.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrialLimits {
    /// Smallest share of similar defects caught, in percent (1..=100).
    pub min_catch_percent: u32,
    /// Most regressions allowed.
    pub max_regressions: u32,
    /// Largest with-safeguard cost as a percent of the baseline cost (at least 100).
    pub max_cost_percent: u32,
}

/// Why a trial could not be judged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    EmptySafeguardName,
    NoSimilarDefects,
    ZeroBaselineCost,
    MissingCatchLimit,
    MissingCostLimit,
}

/// Why a judged safeguard was not accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rejection {
    MissesSimilarDefects,
    TooManyRegressions,
    OverheadTooHigh,
}

/// The verdict on a trial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Accepted,
    Rejected(Rejection),
    Refused(Refusal),
}

/// Judge a safeguard against the limits. Checks run in order: catch rate, regressions, overhead.
pub fn judge(trial: &SafeguardTrial, limits: &TrialLimits) -> Verdict {
    if trial.safeguard.trim().is_empty() {
        return Verdict::Refused(Refusal::EmptySafeguardName);
    }
    if trial.similar_defects.is_empty() {
        return Verdict::Refused(Refusal::NoSimilarDefects);
    }
    if trial.baseline_cost == 0 {
        return Verdict::Refused(Refusal::ZeroBaselineCost);
    }
    if limits.min_catch_percent == 0 {
        return Verdict::Refused(Refusal::MissingCatchLimit);
    }
    if limits.max_cost_percent == 0 {
        return Verdict::Refused(Refusal::MissingCostLimit);
    }
    let total = trial.similar_defects.len() as u64;
    let caught = trial.similar_defects.iter().filter(|c| **c).count() as u64;
    if caught * 100 < u64::from(limits.min_catch_percent) * total {
        return Verdict::Rejected(Rejection::MissesSimilarDefects);
    }
    if trial.regressions > limits.max_regressions {
        return Verdict::Rejected(Rejection::TooManyRegressions);
    }
    if trial.with_safeguard_cost * 100 > trial.baseline_cost * u64::from(limits.max_cost_percent) {
        return Verdict::Rejected(Rejection::OverheadTooHigh);
    }
    Verdict::Accepted
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trial() -> SafeguardTrial {
        SafeguardTrial {
            safeguard: "check-x".to_string(),
            similar_defects: vec![true, true, true, false],
            regressions: 0,
            baseline_cost: 100,
            with_safeguard_cost: 110,
        }
    }

    fn limits() -> TrialLimits {
        TrialLimits {
            min_catch_percent: 75,
            max_regressions: 1,
            max_cost_percent: 120,
        }
    }

    #[test]
    fn safeguard_trial_accepts_within_limits() {
        assert_eq!(judge(&trial(), &limits()), Verdict::Accepted);
    }

    #[test]
    fn safeguard_trial_rejects_missed_defects() {
        let mut t = trial();
        t.similar_defects = vec![true, false, false, false];
        assert_eq!(
            judge(&t, &limits()),
            Verdict::Rejected(Rejection::MissesSimilarDefects)
        );
    }

    #[test]
    fn safeguard_trial_rejects_regressions_over_limit() {
        let mut t = trial();
        t.regressions = 2;
        assert_eq!(
            judge(&t, &limits()),
            Verdict::Rejected(Rejection::TooManyRegressions)
        );
    }

    #[test]
    fn safeguard_trial_rejects_overhead_over_limit() {
        let mut t = trial();
        t.with_safeguard_cost = 121;
        assert_eq!(
            judge(&t, &limits()),
            Verdict::Rejected(Rejection::OverheadTooHigh)
        );
    }

    #[test]
    fn safeguard_trial_accepts_at_exact_limits() {
        let mut t = trial();
        t.regressions = 1;
        t.with_safeguard_cost = 120;
        assert_eq!(judge(&t, &limits()), Verdict::Accepted);
    }

    #[test]
    fn safeguard_trial_refuses_empty_defect_set() {
        let mut t = trial();
        t.similar_defects.clear();
        assert_eq!(
            judge(&t, &limits()),
            Verdict::Refused(Refusal::NoSimilarDefects)
        );
    }

    #[test]
    fn safeguard_trial_refuses_missing_inputs_and_limits() {
        let mut t = trial();
        t.safeguard = " ".to_string();
        assert_eq!(
            judge(&t, &limits()),
            Verdict::Refused(Refusal::EmptySafeguardName)
        );
        let mut t = trial();
        t.baseline_cost = 0;
        assert_eq!(
            judge(&t, &limits()),
            Verdict::Refused(Refusal::ZeroBaselineCost)
        );
        let mut l = limits();
        l.min_catch_percent = 0;
        assert_eq!(
            judge(&trial(), &l),
            Verdict::Refused(Refusal::MissingCatchLimit)
        );
        let mut l = limits();
        l.max_cost_percent = 0;
        assert_eq!(
            judge(&trial(), &l),
            Verdict::Refused(Refusal::MissingCostLimit)
        );
    }
}
