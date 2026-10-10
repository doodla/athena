//! Resolving a job's forecasts and scoring them (design §9).

use crate::store::{JobRecords, Records};
use athena_data::config::ReferencePriors;
use athena_data::records::{
    AcceptedBy, Forecast, InterventionSource, Invocation, OutcomeState, Role, Score, Totals,
    Verdict,
};
use athena_data::{JobId, Provenance, Quantity, RecordId, Unit};
use athena_score::{PastJob, ReadingForecasts, Resolution, reference_forecast, score_job};
use std::collections::{BTreeMap, BTreeSet};

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
                .find(|q| q.quantity == "cost_usd")
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
    let value: f64 = invocations.iter().map(|i| i.cost.best_lower()).sum();
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

/// What happened, as the scorer needs it.
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

/// Scores a job from its records plus what just happened. `invocations` holds every invocation of the
/// job, committed or new. `cause` is set for a re-score.
pub fn score(
    job: &JobRecords,
    records: &Records,
    invocations: &[Invocation],
    happened: &Happened,
    cause: Option<RecordId>,
    at: jiff::Timestamp,
) -> Score {
    let id: &JobId = job.id();
    let readings: Vec<ReadingForecasts> = job
        .readings
        .iter()
        .map(|r| ReadingForecasts {
            n: r.n,
            reader: r.forecasts.reader.clone(),
            reference: r.forecasts.reference.clone(),
            deliverables: r
                .deliverables
                .iter()
                .map(|d| (d.path.clone(), d.p))
                .collect(),
        })
        .collect();
    let accepted = job.job.as_ref().and_then(|j| j.accepted_reading);
    let accepted_by = job.job.as_ref().and_then(|j| j.accepted_by);
    let corrected: BTreeSet<u32> = job.corrections.iter().map(|c| c.n).collect();
    let mut reading_accepted = BTreeMap::new();
    for r in &job.readings {
        let outcome = if corrected.contains(&r.n) {
            Some(false)
        } else if Some(r.n) == accepted {
            match accepted_by {
                Some(AcceptedBy::User) => Some(true),
                _ => None,
            }
        } else {
            // Closed without being accepted or corrected.
            Some(false)
        };
        reading_accepted.insert(r.n, outcome);
    }
    let dispatched_at = job.job.as_ref().and_then(|j| j.dispatched_at);
    let intervened = dispatched_at.is_some_and(|t| {
        records
            .interventions_for(id)
            .iter()
            .any(|i| i.source == InterventionSource::User && i.at >= t)
    }) || cause.is_some();
    let execution: Vec<&Invocation> = invocations
        .iter()
        .filter(|i| i.role != Role::Reader)
        .collect();
    let mut observed: BTreeSet<String> = BTreeSet::new();
    for inv in invocations {
        observed.extend(inv.models_used.keys().cloned());
    }
    let mut expected: Vec<String> = Vec::new();
    if let Some(r) = job.latest_reading() {
        expected.push(r.producer_model.clone());
        expected.push(r.verifier_model.clone());
    }
    for inv in invocations.iter().filter(|i| i.role == Role::Reader) {
        if !expected.contains(&inv.model) {
            expected.push(inv.model.clone());
        }
    }
    let resolution = Resolution {
        reading_accepted,
        verification_passed: if accepted.is_some() {
            happened.verification_passed
        } else {
            None
        },
        user_intervened: intervened,
        intervention_provisional: cause.is_none(),
        changed_paths: happened.changed_paths.clone(),
        cost_execution_usd: (accepted.is_some() && !execution.is_empty())
            .then(|| execution.iter().map(|i| i.cost.best_lower()).sum())
            .or(accepted.map(|_| 0.0)),
        wall_time_execution_s: happened.wall_time_execution_s,
        models_expected: expected,
        models_observed: observed.into_iter().collect(),
    };
    score_job(id, at, cause, &readings, accepted, &resolution)
}

/// Whether a final verdict passed.
pub fn passed(verdict: Verdict) -> bool {
    verdict == Verdict::Pass
}

/// A re-score after an intervention against a closed job: the closing score with `user_intervenes`
/// resolved true for good (design §9.5).
pub fn rescore(previous: &Score, cause: RecordId, at: jiff::Timestamp) -> Score {
    let mut score = previous.clone();
    score.cause = Some(cause);
    score.at = at;
    for event in score
        .events
        .iter_mut()
        .filter(|e| e.event == "user_intervenes")
    {
        event.outcome = Some(true);
        event.provisional = false;
        event.brier = event
            .forecasts
            .iter()
            .map(|(k, p)| (k.clone(), athena_score::brier(*p, true)))
            .collect();
    }
    score
}
