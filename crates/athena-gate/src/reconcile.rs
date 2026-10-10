//! Reconciliation of usage records (design §10.3, M15).
//!
//! - Per invocation: the per-model costs sum to the reported total within $0.000001, and the requested
//!   model's per-model tokens equal the invocation's measured tokens.
//! - Per job: the totals in each outcome record equal the sums over the job's invocation records.

use crate::records::RecordSet;
use crate::{Result, Rule, Violation};
use athena_data::records::{AnyRecord, Invocation, ModelUsage, Outcome};
use athena_data::{Provenance, Quantity};
use athena_git::Git;
use std::collections::BTreeMap;

/// The tolerance for sums of dollar amounts.
pub const COST_TOLERANCE_USD: f64 = 0.000_001;

/// Sums over a set of invocations, as an outcome records them.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sums {
    pub invocations: u32,
    pub cost_usd: f64,
    pub tokens: BTreeMap<String, ModelUsage>,
}

/// Sums the cost (best value or lower bound) and per-model usage of `invocations`.
pub fn sums<'a>(invocations: impl IntoIterator<Item = &'a Invocation>) -> Sums {
    let mut sums = Sums::default();
    for inv in invocations {
        sums.invocations += 1;
        sums.cost_usd += inv.cost.best_lower();
        for (model, usage) in &inv.models_used {
            let total = sums.tokens.entry(model.clone()).or_insert(ModelUsage {
                input: 0,
                output: 0,
                cache_read: 0,
                cache_write: 0,
                cost_usd: 0.0,
            });
            total.input += usage.input;
            total.output += usage.output;
            total.cache_read += usage.cache_read;
            total.cache_write += usage.cache_write;
            total.cost_usd += usage.cost_usd;
        }
    }
    sums
}

fn measured(q: &Quantity) -> Option<u64> {
    (q.provenance() == Provenance::Measured)
        .then(|| q.value().map(|v| v as u64))
        .flatten()
}

/// Checks one invocation record against itself.
pub fn check_invocation(path: &str, inv: &Invocation) -> Vec<Violation> {
    let mut violations = Vec::new();
    let fail = |detail: String| {
        Violation::new(
            Rule::Reconciliation,
            detail,
            "invocation records are written by athena from the driver's own figures",
        )
        .at_path(path)
    };
    if inv.cost.provenance() == Provenance::Reported
        && let Some(total) = inv.cost.value()
        && !inv.models_used.is_empty()
    {
        let sum: f64 = inv.models_used.values().map(|u| u.cost_usd).sum();
        if (sum - total).abs() > COST_TOLERANCE_USD {
            violations.push(fail(format!(
                "{path}: per-model costs sum to {sum}, but the reported total is {total}"
            )));
        }
    }
    let t = &inv.tokens;
    if let (Some(input), Some(output), Some(cache_read), Some(cache_write)) = (
        measured(&t.input),
        measured(&t.output),
        measured(&t.cache_read),
        measured(&t.cache_write),
    ) {
        match inv.models_used.get(&inv.model) {
            Some(u) => {
                if (u.input, u.output, u.cache_read, u.cache_write)
                    != (input, output, cache_read, cache_write)
                {
                    violations.push(fail(format!(
                        "{path}: the requested model {} used {}/{}/{}/{} tokens, but the invocation records {input}/{output}/{cache_read}/{cache_write}",
                        inv.model, u.input, u.output, u.cache_read, u.cache_write
                    )));
                }
            }
            None => violations.push(fail(format!(
                "{path}: measured tokens, but no usage for the requested model {}",
                inv.model
            ))),
        }
    }
    violations
}

/// Checks an outcome's totals against the job's invocation records.
pub fn check_outcome(path: &str, outcome: &Outcome, invocations: &[&Invocation]) -> Vec<Violation> {
    let Some(totals) = &outcome.totals else {
        return Vec::new();
    };
    let mut violations = Vec::new();
    let fail = |detail: String| {
        Violation::new(
            Rule::Reconciliation,
            detail,
            "outcomes are written by athena from the job's invocation records",
        )
        .at_path(path)
    };
    let expected = sums(invocations.iter().copied());
    if totals.invocations != expected.invocations {
        violations.push(fail(format!(
            "{path}: totals count {} invocations, but the job has {} invocation records",
            totals.invocations, expected.invocations
        )));
    }
    let recorded = totals.cost_all.best_lower();
    if (recorded - expected.cost_usd).abs() > COST_TOLERANCE_USD {
        violations.push(fail(format!(
            "{path}: total cost {recorded} differs from the invocations' sum {}",
            expected.cost_usd
        )));
    }
    for (model, usage) in &expected.tokens {
        match totals.tokens.get(model) {
            Some(t)
                if (t.input, t.output, t.cache_read, t.cache_write)
                    == (
                        usage.input,
                        usage.output,
                        usage.cache_read,
                        usage.cache_write,
                    ) => {}
            _ => violations.push(fail(format!(
                "{path}: token totals for {model} differ from the invocations' sums"
            ))),
        }
    }
    for model in totals.tokens.keys() {
        if !expected.tokens.contains_key(model) {
            violations.push(fail(format!(
                "{path}: token totals name {model}, which no invocation used"
            )));
        }
    }
    violations
}

/// Reconciles every invocation and outcome record in `records`.
pub fn check_records(records: &RecordSet) -> Vec<Violation> {
    let mut violations = Vec::new();
    let mut by_job: BTreeMap<String, Vec<&Invocation>> = BTreeMap::new();
    let mut outcomes: Vec<(&str, &Outcome)> = Vec::new();
    for (path, record) in records.records() {
        match record {
            AnyRecord::Invocation(inv) => {
                violations.extend(check_invocation(path, inv));
                by_job
                    .entry(inv.job.as_str().to_string())
                    .or_default()
                    .push(inv);
            }
            AnyRecord::Outcome(outcome) => outcomes.push((path, outcome)),
            _ => {}
        }
    }
    for (path, outcome) in outcomes {
        let invocations = by_job
            .get(outcome.job.as_str())
            .cloned()
            .unwrap_or_default();
        violations.extend(check_outcome(path, outcome, &invocations));
    }
    violations
}

/// Reconciles every record in `treeish`.
pub fn check_tree(git: &Git, treeish: &str) -> Result<Vec<Violation>> {
    Ok(check_records(&RecordSet::load(git, treeish)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::records::{
        Build, DriverInfo, InvocationCaps, InvocationContext, InvocationOutcome, OutcomeState,
        Role, Tokens, Totals,
    };
    use athena_data::{JobId, Unit};

    fn invocation(cost: f64, per_model: f64) -> Invocation {
        let tokens = |v: f64| Quantity::measured(v, Unit::Tokens, "result.usage");
        Invocation {
            job: JobId::parse("20261010-153012-k7q2").unwrap(),
            seq: 1,
            role: Role::Producer,
            driver: DriverInfo {
                name: "claude-code".into(),
                version: "2.1.296".into(),
            },
            athena: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            model: "claude-sonnet-5-5".into(),
            context: InvocationContext {
                components: vec![],
                prompt_sha256: "0".repeat(64),
                prompt_tokens: Quantity::estimated(Some(1.0), Unit::Tokens, "estimate", None, None)
                    .unwrap(),
                host_baseline: Quantity::estimated(Some(600.0), Unit::Tokens, "E3", None, None)
                    .unwrap(),
                tools: vec![],
            },
            caps: InvocationCaps {
                max_budget_usd: 3.0,
                timeout_s: 1800,
            },
            started_at: "2026-10-10T15:30:12Z".parse().unwrap(),
            ended_at: "2026-10-10T15:31:12Z".parse().unwrap(),
            wall_time: Quantity::measured(60.0, Unit::Seconds, "athena clock"),
            tokens: Tokens {
                input: tokens(10.0),
                output: tokens(20.0),
                cache_read: tokens(30.0),
                cache_write: tokens(40.0),
            },
            models_used: BTreeMap::from([(
                "claude-sonnet-5-5".into(),
                ModelUsage {
                    input: 10,
                    output: 20,
                    cache_read: 30,
                    cache_write: 40,
                    cost_usd: per_model,
                },
            )]),
            cost: Quantity::reported(cost, Unit::Usd, "Claude Code total_cost_usd"),
            outcome: InvocationOutcome::Success,
            detail: None,
            turns: Some(3),
            commands_run: vec![],
            policy_denials: vec![],
        }
    }

    fn outcome(cost: f64, invocations: u32) -> Outcome {
        Outcome {
            job: JobId::parse("20261010-153012-k7q2").unwrap(),
            state: OutcomeState::Landed,
            reason: "verified".into(),
            at: "2026-10-10T15:40:00Z".parse().unwrap(),
            totals: Some(Totals {
                cost_all: Quantity::reported(cost, Unit::Usd, "sum"),
                cost_execution: Quantity::reported(cost, Unit::Usd, "sum"),
                wall_time_execution: None,
                tokens: BTreeMap::from([(
                    "claude-sonnet-5-5".into(),
                    ModelUsage {
                        input: 10,
                        output: 20,
                        cache_read: 30,
                        cache_write: 40,
                        cost_usd: cost,
                    },
                )]),
                invocations,
            }),
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        }
    }

    #[test]
    fn consistent_records_reconcile() {
        // M15 (allows): consistent records.
        let inv = invocation(0.41, 0.41);
        assert_eq!(check_invocation("x", &inv), vec![]);
        assert_eq!(check_outcome("o", &outcome(0.41, 1), &[&inv]), vec![]);
    }

    #[test]
    fn per_model_costs_that_do_not_sum_are_flagged() {
        // M15 (blocks): per-model costs that do not sum to the total.
        assert_eq!(check_invocation("x", &invocation(0.41, 0.40)).len(), 1);
    }

    #[test]
    fn outcome_totals_that_differ_from_the_invocations_are_flagged() {
        // M15 (blocks): outcome totals that differ from the invocations.
        let inv = invocation(0.41, 0.41);
        assert!(!check_outcome("o", &outcome(0.50, 1), &[&inv]).is_empty());
        assert!(!check_outcome("o", &outcome(0.41, 2), &[&inv]).is_empty());
    }

    #[test]
    fn requested_model_tokens_must_match() {
        let mut inv = invocation(0.41, 0.41);
        inv.models_used.get_mut("claude-sonnet-5-5").unwrap().output = 21;
        assert_eq!(check_invocation("x", &inv).len(), 1);
    }
}
