//! Resolving a job's forecasts and scoring them (design §9).

use crate::store::{JobRecords, Records};
use athena_data::config::ReferencePriors;
use athena_data::records::{
    Candidate, Forecast, Intervention, Invocation, OutcomeState, Role, Score, Totals, Verdict,
    Verification,
};
use athena_data::{JobId, Provenance, Quantity, RecordId, Unit};
use athena_score::{JobFacts, PastJob, reference_forecast, score_records};
use std::collections::BTreeSet;

/// The resolved history the reference forecaster learns from: every closed, scored job.
pub fn history(records: &Records) -> Vec<PastJob> {
    let mut past = Vec::new();
    for job in records.request_jobs() {
        let Some(score) = job.latest_score() else {
            continue;
        };
        let outcome = |event: &str| {
            score
                .events
                .iter()
                .find(|e| e.event == event)
                .and_then(|e| e.outcome)
        };
        past.push(PastJob {
            class: job.class().unwrap_or("other").to_string(),
            reading_accepted: score
                .events
                .iter()
                .filter(|e| e.event == "reading_accepted")
                .filter_map(|e| e.outcome)
                .collect(),
            verification_passed: outcome("verification_passes"),
            user_intervened: outcome("user_intervenes"),
            cost_execution_usd: score
                .quantities
                .iter()
                .find(|q| q.quantity == "cost_usd" && !q.provisional)
                .map(|q| q.actual),
            wall_time_execution_s: score
                .quantities
                .iter()
                .find(|q| q.quantity == "wall_time_s")
                .map(|q| q.actual),
        });
    }
    past
}

/// The reference forecast for a class, and a one-line basis for it.
pub fn reference(priors: &ReferencePriors, class: &str, records: &Records) -> (Forecast, String) {
    let past = history(records);
    let in_class = past.iter().filter(|j| j.class == class).count();
    let basis = if past.is_empty() {
        "Computed by Athena from the priors in athena.toml; no job has been scored yet.".to_string()
    } else if in_class >= athena_score::MIN_CLASS_HISTORY {
        format!(
            "Computed by Athena from {in_class} scored {class} jobs and the priors in athena.toml."
        )
    } else {
        format!(
            "Computed by Athena from {} scored jobs and the priors in athena.toml.",
            past.len()
        )
    };
    (reference_forecast(priors, class, &past), basis)
}

/// Sums a set of invocation costs into one quantity with honest provenance.
pub fn sum_costs<'a>(
    invocations: impl IntoIterator<Item = &'a Invocation>,
    what: &str,
) -> Quantity {
    let invocations: Vec<&Invocation> = invocations.into_iter().collect();
    let value = crate::store::positive_zero(invocations.iter().map(|i| i.cost.best_lower()).sum());
    if invocations
        .iter()
        .all(|i| i.cost.provenance() == Provenance::Reported)
    {
        return Quantity::reported(
            value,
            Unit::Usd,
            format!("sum of the reported costs of {what}"),
        );
    }
    let upper: Option<f64> = invocations
        .iter()
        .map(|i| i.cost.value().or(i.cost.upper()))
        .sum();
    Quantity::estimated(
        Some(value),
        Unit::Usd,
        format!("sum of the costs of {what}; some are bounds"),
        Some(value),
        upper,
    )
    .unwrap_or_else(|_| {
        Quantity::estimated(
            Some(value),
            Unit::Usd,
            format!("sum of the costs of {what}"),
            None,
            None,
        )
        .expect("non-negative")
    })
}

/// The totals an outcome records.
pub fn totals(invocations: &[Invocation], wall_time_execution: Option<f64>) -> Totals {
    let sums = athena_gate::reconcile::sums(invocations.iter());
    Totals {
        cost_all: sum_costs(invocations.iter(), "every invocation of the job"),
        cost_execution: sum_costs(
            invocations.iter().filter(|i| i.role != Role::Reader),
            "the producer and verifier invocations",
        ),
        wall_time_execution: wall_time_execution.map(|s| {
            Quantity::measured(
                s,
                Unit::Seconds,
                "athena clock, from dispatch to the final verdict",
            )
        }),
        tokens: sums.tokens,
        invocations: sums.invocations,
    }
}

/// What happened, as the closing path tracks it.
#[derive(Debug, Clone, Default)]
pub struct Happened {
    /// The final verdict, if verification ran; `Some(false)` for a failure before verification.
    pub verification_passed: Option<bool>,
    pub changed_paths: BTreeSet<String>,
    pub wall_time_execution_s: Option<f64>,
    pub state: Option<OutcomeState>,
    /// Why the job ended where it did, when it did not land.
    pub reason: Option<String>,
}

/// What a closing commit adds that the score reads.
#[derive(Debug, Clone, Copy, Default)]
pub struct Closing<'a> {
    pub candidate: Option<&'a Candidate>,
    pub verification: Option<&'a Verification>,
    pub wall_time_execution_s: Option<f64>,
}

/// Scores a job from its records: the committed ones in `job`, every invocation (committed or new) in
/// `invocations`, what the closing commit adds, and the interventions in `records`, which must be the
/// records the closing commit is built on. The gate recomputes the score from the commit with the same
/// function, so it must see what the commit will hold (M27).
pub fn score(
    job: &JobRecords,
    records: &Records,
    invocations: &[Invocation],
    closing: Closing<'_>,
    cause: Option<RecordId>,
    at: jiff::Timestamp,
) -> Score {
    let id: &JobId = job.id();
    let interventions: Vec<&Intervention> = records.interventions_for(id);
    score_records(
        &JobFacts {
            job: id,
            readings: &job.readings,
            corrections: &job.corrections,
            dispatched: job.job.as_ref(),
            invocations,
            candidate: closing.candidate,
            verification: closing.verification,
            wall_time_execution_s: closing.wall_time_execution_s,
            interventions: &interventions,
        },
        cause,
        at,
    )
}

/// Whether a final verdict passed.
pub fn passed(verdict: Verdict) -> bool {
    verdict == Verdict::Pass
}

pub use athena_score::rescore;
