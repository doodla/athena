//! `study.model/1`: the work-handling model as data (design §5.1).
//!
//! Every element lists the ids of the claims that support it. Fields that a rule must be able to find
//! absent are `Option`s: absence is a fault, while an empty value (`none`, an empty list with a claim) is a
//! statement.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MODEL_SCHEMA: &str = "study.model/1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StateKind {
    Initial,
    Active,
    Waiting,
    TerminalSuccess,
    TerminalFailure,
}

impl StateKind {
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            StateKind::TerminalSuccess | StateKind::TerminalFailure
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub id: String,
    pub kind: StateKind,
    pub claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Counter {
    pub id: String,
    pub start: i64,
    /// Ids of the transitions that change it.
    pub changed_by: Vec<String>,
    pub claims: Vec<String>,
}

/// Who performs a transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Actor {
    Person,
    Process,
    Agent,
    Clock,
}

/// Who decides a trigger or a guard. Anything but `code` is model-decided and left to the verifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvaluatedBy {
    Code,
    Agent,
    Person,
}

/// Retry data on a failure edge that is a retry edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Retry {
    /// Maximum attempts, counted in attempts and not in requeues. `None` says the bound is absent.
    pub bound: Option<u32>,
    /// Claims saying the bound is absent; required when `bound` is `None`.
    pub bound_absent_claims: Vec<String>,
    pub backoff: Option<String>,
    /// What changes on the retry (`nothing` is a value). `None` means the list is missing.
    pub changes: Option<Vec<String>>,
    pub claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub id: String,
    pub from: String,
    pub to: String,
    pub trigger: String,
    pub trigger_evaluated_by: EvaluatedBy,
    /// The guard text, or `none`. `None` here means the field is missing, which is a fault.
    pub guard: Option<String>,
    pub guard_evaluated_by: EvaluatedBy,
    /// Counters the guard uses.
    pub guard_counters: Vec<String>,
    pub actor: Actor,
    /// The failure this edge handles; present exactly on failure edges.
    pub failure: Option<String>,
    pub retry: Option<Retry>,
    pub claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnhandledFailure {
    pub name: String,
    pub claims: Vec<String>,
}

/// Failures the system has no edge for. An empty list is allowed when a claim says so.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unhandled {
    pub failures: Vec<UnhandledFailure>,
    pub claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concurrency {
    pub shared_resources: Vec<String>,
    pub serialized_by: Vec<String>,
    /// The limit on simultaneous units, if there is one.
    pub limit: Option<u32>,
    pub on_conflict: String,
    pub claims: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Touchpoint {
    pub id: String,
    /// The state or transition id where a person is addressed.
    pub at: String,
    /// The rule for a person who never answers, or `none`. `None` here means the field is missing.
    pub no_answer: Option<String>,
    pub claims: Vec<String>,
}

/// `study.model/1`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub schema: String,
    pub study_id: String,
    pub states: Vec<State>,
    pub counters: Vec<Counter>,
    pub transitions: Vec<Transition>,
    /// `None` means the list is missing, which is a fault.
    pub unhandled: Option<Unhandled>,
    pub concurrency: Option<Concurrency>,
    pub touchpoints: Vec<Touchpoint>,
}

impl Model {
    /// Ids of transitions that are failure edges.
    pub fn failure_edges(&self) -> impl Iterator<Item = &Transition> {
        self.transitions.iter().filter(|t| t.failure.is_some())
    }

    /// State ids reachable from `starts` by following transitions (the starts included).
    pub fn reachable_from<'a>(
        &'a self,
        starts: impl IntoIterator<Item = &'a str>,
    ) -> BTreeSet<&'a str> {
        let mut next: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for t in &self.transitions {
            next.entry(t.from.as_str()).or_default().push(t.to.as_str());
        }
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut stack: Vec<&str> = starts.into_iter().collect();
        while let Some(s) = stack.pop() {
            if seen.insert(s) {
                stack.extend(next.get(s).into_iter().flatten().copied());
            }
        }
        seen
    }

    /// Retry edges: failure edges that enter a state from which their own source state is reachable.
    pub fn retry_edges(&self) -> Vec<&Transition> {
        self.failure_edges()
            .filter(|t| {
                self.reachable_from([t.to.as_str()])
                    .contains(t.from.as_str())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::tests::example_model;

    #[test]
    fn the_example_timeout_edge_is_the_only_retry_edge() {
        let model = example_model();
        let retry: Vec<&str> = model.retry_edges().iter().map(|t| t.id.as_str()).collect();
        assert_eq!(retry, ["t3"]);
        assert_eq!(model.failure_edges().count(), 2);
    }

    #[test]
    fn a_model_round_trips_through_json() {
        let model = example_model();
        let json = serde_json::to_string(&model).unwrap();
        let back: Model = serde_json::from_str(&json).unwrap();
        assert_eq!(back, model);
        assert!(serde_json::from_str::<Model>(&json.replace("\"states\"", "\"stat\"")).is_err());
    }
}
