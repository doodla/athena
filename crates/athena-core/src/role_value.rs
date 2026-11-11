//! Value of a new role against a simpler alternative, and safe retirement of a role.
//!
//! A role is kept only when a bounded experiment (`method_experiment::compare`) shows
//! it better than the simpler alternative. A role may be retired only when every item
//! of institutional knowledge it holds is recorded elsewhere.

use crate::method_experiment::{self, MethodExperiment, Outcome, Refusal, Verdict};

/// Whether a role earns its place against the simpler alternative.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Value {
    /// The role beat the alternative.
    WorthKeeping,
    /// The role was no better, or worse.
    NotWorthKeeping,
}

/// Why a value judgment was not made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueRefusal {
    /// The experiment was refused (empty role name, missing arm or zero bound).
    Refused(Refusal),
    /// The experiment spent more runs than its bound.
    Stopped,
}

/// Judge a role (`e.method` is the role name; baseline is the simpler alternative).
pub fn judge(e: &MethodExperiment) -> Result<Value, ValueRefusal> {
    match method_experiment::compare(e) {
        Outcome::Refused(r) => Err(ValueRefusal::Refused(r)),
        Outcome::Stopped => Err(ValueRefusal::Stopped),
        Outcome::Verdict(Verdict::Better) => Ok(Value::WorthKeeping),
        Outcome::Verdict(_) => Ok(Value::NotWorthKeeping),
    }
}

/// Retirement is refused because knowledge is still held only by the role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetirementRefused {
    pub unrecorded: Vec<String>,
}

/// Allow retiring a role only if every knowledge item it holds is in `recorded`.
/// A role holding no knowledge items may be retired.
pub fn can_retire(held: &[String], recorded: &[String]) -> Result<(), RetirementRefused> {
    let unrecorded: Vec<String> = held
        .iter()
        .filter(|k| !recorded.contains(k))
        .cloned()
        .collect();
    if unrecorded.is_empty() {
        Ok(())
    } else {
        Err(RetirementRefused { unrecorded })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method_experiment::Direction;

    fn exp(name: &str, base: &[f64], cand: &[f64], bound: usize) -> MethodExperiment {
        MethodExperiment {
            method: name.into(),
            baseline: base.to_vec(),
            candidate: cand.to_vec(),
            bound,
            direction: Direction::LowerIsBetter,
        }
    }

    fn s(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn better_role_is_worth_keeping() {
        assert_eq!(
            judge(&exp("scout", &[5.0], &[3.0], 2)),
            Ok(Value::WorthKeeping)
        );
    }

    #[test]
    fn equal_or_worse_role_is_not_worth_keeping() {
        assert_eq!(
            judge(&exp("scout", &[4.0], &[4.0], 2)),
            Ok(Value::NotWorthKeeping)
        );
        assert_eq!(
            judge(&exp("scout", &[3.0], &[5.0], 2)),
            Ok(Value::NotWorthKeeping)
        );
    }

    #[test]
    fn incomplete_experiments_are_refused() {
        assert_eq!(
            judge(&exp(" ", &[1.0], &[1.0], 2)),
            Err(ValueRefusal::Refused(Refusal::EmptyMethod))
        );
        assert_eq!(
            judge(&exp("r", &[], &[1.0], 2)),
            Err(ValueRefusal::Refused(Refusal::MissingBaseline))
        );
        assert_eq!(
            judge(&exp("r", &[1.0], &[], 2)),
            Err(ValueRefusal::Refused(Refusal::MissingCandidate))
        );
        assert_eq!(
            judge(&exp("r", &[1.0], &[1.0], 0)),
            Err(ValueRefusal::Refused(Refusal::ZeroBound))
        );
        assert_eq!(
            judge(&exp("r", &[1.0, 1.0], &[1.0], 2)),
            Err(ValueRefusal::Stopped)
        );
    }

    #[test]
    fn retirement_allowed_when_all_knowledge_recorded() {
        let held = s(&["a", "b"]);
        assert_eq!(can_retire(&held, &s(&["b", "a", "c"])), Ok(()));
    }

    #[test]
    fn retirement_refused_naming_unrecorded_items() {
        let r = can_retire(&s(&["a", "b", "c"]), &s(&["b"])).unwrap_err();
        assert_eq!(r.unrecorded, s(&["a", "c"]));
    }

    #[test]
    fn role_with_no_knowledge_may_retire() {
        assert_eq!(can_retire(&[], &[]), Ok(()));
    }
}
