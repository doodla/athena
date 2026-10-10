//! Proper scoring rules for Athena's forecasts (design §9, D14).
//!
//! - Events are scored with the Brier score, `(p − o)²`, which is strictly proper: a forecaster minimizes
//!   its expected score only by reporting its true belief, so hedging at 0.5 always costs 0.25.
//! - Deliverables are scored with the *sum* of Brier terms over the union of the paths a forecaster listed
//!   and the paths that changed, with unlisted paths at p = 0. Padding a list adds about nothing, and every
//!   missed path costs about 1.
//! - Cost and time are scored with the weighted interval score (WIS) of the 10th, 50th, and 90th
//!   percentiles on natural-log values, floored first. Each unit of interval width costs 0.1 and each unit
//!   by which the actual falls outside the interval costs 1, so a wide interval buys no safety for free.
//!
//! This crate also computes the reference forecast (the base rate a vague forecaster cannot beat) and
//! aggregate skill, which is reported only for at least [`MIN_JOBS_FOR_SKILL`] resolved jobs.

use athena_data::config::{QuantilePrior, ReferencePriors};
use athena_data::records::{
    DeliverableScore, DeliverableTerm, EventScore, Forecast, ModelsCheck, QuantileScore, Quantiles,
    QuantityScore, Score,
};
use athena_data::{JobId, RecordId};
use std::collections::{BTreeMap, BTreeSet};

/// The rules and their version, recorded in every score.
pub const RULES: &str =
    "brier events; brier-sum deliverables; wis-log 80% (alpha 0.2) quantities; v1";
/// Floors applied before taking logs, so a job that cost nothing still scores (M4).
pub const COST_FLOOR_USD: f64 = 0.001;
pub const TIME_FLOOR_S: f64 = 1.0;
/// Skill against the reference is computed only for aggregates of at least this many jobs.
pub const MIN_JOBS_FOR_SKILL: usize = 10;
/// The reference uses a class's own history once it has this many resolved jobs, else the global history.
pub const MIN_CLASS_HISTORY: usize = 5;

const ALPHA: f64 = 0.2;

/// The Brier score of probability `p` for an event with outcome `outcome`.
pub fn brier(p: f64, outcome: bool) -> f64 {
    let o = if outcome { 1.0 } else { 0.0 };
    (p - o) * (p - o)
}

/// The weighted interval score of quantiles `q` against `actual`, on natural logs after flooring.
/// Returns the score and whether the actual fell inside the central 80 percent interval.
pub fn wis_log(q: &Quantiles, actual: f64, floor: f64) -> (f64, bool) {
    let ln = |x: f64| x.max(floor).ln();
    let (y, l, m, u) = (ln(actual), ln(q.p10()), ln(q.p50()), ln(q.p90()));
    let mut interval = u - l;
    if y < l {
        interval += (2.0 / ALPHA) * (l - y);
    }
    if y > u {
        interval += (2.0 / ALPHA) * (y - u);
    }
    let wis = (1.0 / 1.5) * (0.5 * (y - m).abs() + (ALPHA / 2.0) * interval);
    (wis, l <= y && y <= u)
}

/// The summed Brier score over the union of listed and changed paths; unlisted paths count as p = 0.
pub fn deliverable_score(listed: &[(String, f64)], changed: &BTreeSet<String>) -> DeliverableScore {
    let mut forecast: BTreeMap<&str, f64> = BTreeMap::new();
    for (path, p) in listed {
        // A path listed twice keeps its larger probability; listing twice never helps.
        let entry = forecast.entry(path.as_str()).or_insert(0.0);
        *entry = entry.max(*p);
    }
    let mut paths: BTreeSet<&str> = forecast.keys().copied().collect();
    paths.extend(changed.iter().map(String::as_str));
    let terms: Vec<DeliverableTerm> = paths
        .into_iter()
        .map(|path| {
            let listed = forecast.contains_key(path);
            let p = forecast.get(path).copied().unwrap_or(0.0);
            let was_changed = changed.contains(path);
            DeliverableTerm {
                path: path.to_string(),
                p,
                listed,
                changed: was_changed,
                brier: brier(p, was_changed),
            }
        })
        .collect();
    let sum = terms.iter().map(|t| t.brier).sum();
    DeliverableScore { terms, sum }
}

/// One resolved past job, as the reference forecaster sees it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PastJob {
    pub class: String,
    pub reading_accepted: Vec<bool>,
    pub verification_passed: Option<bool>,
    pub user_intervened: Option<bool>,
    pub cost_execution_usd: Option<f64>,
    pub wall_time_execution_s: Option<f64>,
}

/// The reference forecast for a job of `class`, from the configured priors and the resolved history.
pub fn reference_forecast(priors: &ReferencePriors, class: &str, history: &[PastJob]) -> Forecast {
    let class_jobs: Vec<&PastJob> = history.iter().filter(|j| j.class == class).collect();
    let pool: Vec<&PastJob> = if class_jobs.len() >= MIN_CLASS_HISTORY {
        class_jobs
    } else {
        history.iter().collect()
    };
    let k = priors.pseudo_count;
    let posterior = |p0: f64, outcomes: Vec<bool>| {
        let n = outcomes.len() as f64;
        let successes = outcomes.iter().filter(|o| **o).count() as f64;
        (k * p0 + successes) / (k + n)
    };
    let accepted: Vec<bool> = pool
        .iter()
        .flat_map(|j| j.reading_accepted.iter().copied())
        .collect();
    let passed: Vec<bool> = pool.iter().filter_map(|j| j.verification_passed).collect();
    let intervened: Vec<bool> = pool.iter().filter_map(|j| j.user_intervened).collect();
    let costs: Vec<f64> = pool.iter().filter_map(|j| j.cost_execution_usd).collect();
    let times: Vec<f64> = pool
        .iter()
        .filter_map(|j| j.wall_time_execution_s)
        .collect();
    Forecast {
        p_reading_accepted: posterior(priors.p_reading_accepted, accepted),
        p_verification_passes: posterior(priors.p_verification_passes, passed),
        p_user_intervenes: posterior(priors.p_user_intervenes, intervened),
        cost_usd: empirical_quantiles(&costs, priors.cost_usd, COST_FLOOR_USD),
        wall_time_s: empirical_quantiles(&times, priors.wall_time_s, TIME_FLOOR_S),
    }
}

/// Empirical 10th, 50th, and 90th percentiles in log space once there are enough values; the prior before.
fn empirical_quantiles(values: &[f64], prior: QuantilePrior, floor: f64) -> Quantiles {
    let from_prior =
        || Quantiles::new(prior.p10, prior.p50, prior.p90).expect("validated priors are ordered");
    if values.len() < MIN_CLASS_HISTORY {
        return from_prior();
    }
    let mut logs: Vec<f64> = values.iter().map(|v| v.max(floor).ln()).collect();
    logs.sort_by(|a, b| a.partial_cmp(b).expect("finite values"));
    let at = |q: f64| {
        let pos = q * (logs.len() - 1) as f64;
        let (lo, hi) = (pos.floor() as usize, pos.ceil() as usize);
        let frac = pos - lo as f64;
        (logs[lo] * (1.0 - frac) + logs[hi] * frac).exp()
    };
    Quantiles::new(at(0.1), at(0.5), at(0.9)).unwrap_or_else(|_| from_prior())
}

/// What happened, as needed to score a job's forecasts.
#[derive(Debug, Clone, PartialEq)]
pub struct Resolution {
    /// Per reading n (1-based): did the user accept it? `None` when unresolved (`--yes`).
    pub reading_accepted: BTreeMap<u32, Option<bool>>,
    pub verification_passed: Option<bool>,
    pub user_intervened: bool,
    /// True until the job can no longer receive interventions; v0 marks it provisional at closing.
    pub intervention_provisional: bool,
    pub changed_paths: BTreeSet<String>,
    pub cost_execution_usd: Option<f64>,
    pub wall_time_execution_s: Option<f64>,
    pub models_expected: Vec<String>,
    pub models_observed: Vec<String>,
}

/// One reading's forecasts, as the scorer needs them.
#[derive(Debug, Clone, PartialEq)]
pub struct ReadingForecasts {
    pub n: u32,
    pub reader: Forecast,
    pub reference: Forecast,
    pub deliverables: Vec<(String, f64)>,
}

/// Scores a job. `readings` holds every reading; `accepted` names the one that was dispatched, if any.
pub fn score_job(
    job: &JobId,
    at: jiff::Timestamp,
    cause: Option<RecordId>,
    readings: &[ReadingForecasts],
    accepted: Option<u32>,
    resolution: &Resolution,
) -> Score {
    let mut events = Vec::new();
    let event = |name: &str,
                 reading: u32,
                 reader: f64,
                 reference: f64,
                 outcome: Option<bool>,
                 provisional: bool| {
        let forecasts = BTreeMap::from([
            ("reader".to_string(), reader),
            ("reference".to_string(), reference),
        ]);
        let brier = match outcome {
            Some(o) => forecasts
                .iter()
                .map(|(k, p)| (k.clone(), brier(*p, o)))
                .collect(),
            None => BTreeMap::new(),
        };
        EventScore {
            event: name.to_string(),
            reading,
            forecasts,
            outcome,
            brier,
            provisional,
        }
    };
    for r in readings {
        let outcome = resolution.reading_accepted.get(&r.n).copied().flatten();
        events.push(event(
            "reading_accepted",
            r.n,
            r.reader.p_reading_accepted,
            r.reference.p_reading_accepted,
            outcome,
            false,
        ));
    }
    let accepted_reading = accepted.and_then(|n| readings.iter().find(|r| r.n == n));
    let mut quantities = Vec::new();
    let mut deliverables = None;
    if let Some(r) = accepted_reading {
        events.push(event(
            "verification_passes",
            r.n,
            r.reader.p_verification_passes,
            r.reference.p_verification_passes,
            resolution.verification_passed,
            false,
        ));
        events.push(event(
            "user_intervenes",
            r.n,
            r.reader.p_user_intervenes,
            r.reference.p_user_intervenes,
            Some(resolution.user_intervened),
            resolution.intervention_provisional,
        ));
        deliverables = Some(deliverable_score(
            &r.deliverables,
            &resolution.changed_paths,
        ));
        let mut quantity =
            |name: &str, actual: Option<f64>, floor: f64, pick: fn(&Forecast) -> Quantiles| {
                if let Some(actual) = actual {
                    let forecasts = [("reader", &r.reader), ("reference", &r.reference)]
                        .into_iter()
                        .map(|(name, f)| {
                            let q = pick(f);
                            let (wis, inside) = wis_log(&q, actual, floor);
                            (
                                name.to_string(),
                                QuantileScore {
                                    p10: q.p10(),
                                    p50: q.p50(),
                                    p90: q.p90(),
                                    wis,
                                    inside_80: inside,
                                },
                            )
                        })
                        .collect();
                    quantities.push(QuantityScore {
                        quantity: name.to_string(),
                        actual,
                        floor,
                        forecasts,
                    });
                }
            };
        quantity(
            "cost_usd",
            resolution.cost_execution_usd,
            COST_FLOOR_USD,
            |f| f.cost_usd,
        );
        quantity(
            "wall_time_s",
            resolution.wall_time_execution_s,
            TIME_FLOOR_S,
            |f| f.wall_time_s,
        );
    }
    let models = (!resolution.models_expected.is_empty()).then(|| {
        let expected: BTreeSet<&String> = resolution.models_expected.iter().collect();
        let unexpected: Vec<String> = resolution
            .models_observed
            .iter()
            .filter(|m| !expected.contains(m))
            .cloned()
            .collect();
        ModelsCheck {
            expected: resolution.models_expected.clone(),
            observed: resolution.models_observed.clone(),
            as_expected: unexpected.is_empty(),
            unexpected,
        }
    });
    Score {
        job: job.clone(),
        at,
        rules: RULES.to_string(),
        cause,
        events,
        deliverables,
        quantities,
        models,
    }
}

/// Summed scores for one forecaster over a set of jobs.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Totals {
    pub jobs: usize,
    pub event_brier_sum: f64,
    pub events: usize,
    pub wis_sum: f64,
    pub quantities: usize,
}

/// Aggregates per forecaster, and skill against the reference once there are enough jobs.
#[derive(Debug, Clone, PartialEq)]
pub struct Aggregate {
    pub jobs: usize,
    pub per_forecaster: BTreeMap<String, Totals>,
    /// 1 − ΣS_reader / ΣS_reference for events, only when `jobs >= MIN_JOBS_FOR_SKILL`.
    pub event_skill: Option<f64>,
    /// The same for quantities.
    pub quantity_skill: Option<f64>,
}

/// Aggregates the latest score of each job. Skill is `None` below [`MIN_JOBS_FOR_SKILL`] jobs.
pub fn aggregate(scores: &[&Score]) -> Aggregate {
    let mut per_forecaster: BTreeMap<String, Totals> = BTreeMap::new();
    for score in scores {
        for e in &score.events {
            for (forecaster, b) in &e.brier {
                let t = per_forecaster.entry(forecaster.clone()).or_default();
                t.event_brier_sum += b;
                t.events += 1;
            }
        }
        for q in &score.quantities {
            for (forecaster, s) in &q.forecasts {
                let t = per_forecaster.entry(forecaster.clone()).or_default();
                t.wis_sum += s.wis;
                t.quantities += 1;
            }
        }
        for t in per_forecaster.values_mut() {
            t.jobs += 1;
        }
    }
    let jobs = scores.len();
    let skill = |reader: f64, reference: f64| {
        (jobs >= MIN_JOBS_FOR_SKILL && reference > 0.0).then(|| 1.0 - reader / reference)
    };
    let get = |name: &str| per_forecaster.get(name).cloned().unwrap_or_default();
    let (reader, reference) = (get("reader"), get("reference"));
    Aggregate {
        jobs,
        event_skill: skill(reader.event_brier_sum, reference.event_brier_sum),
        quantity_skill: skill(reader.wis_sum, reference.wis_sum),
        per_forecaster,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn q(a: f64, b: f64, c: f64) -> Quantiles {
        Quantiles::new(a, b, c).unwrap()
    }

    #[test]
    fn brier_values() {
        assert_eq!(brier(1.0, true), 0.0);
        assert_eq!(brier(0.5, true), 0.25);
        assert_eq!(brier(0.5, false), 0.25);
        assert!((brier(0.9, false) - 0.81).abs() < 1e-12);
    }

    #[test]
    fn brier_is_strictly_proper_on_a_grid() {
        // For each true rate, the expected score is minimized by reporting that rate and nothing else.
        for true_p in [0.05, 0.2, 0.5, 0.73, 0.95] {
            let expected = |q: f64| true_p * brier(q, true) + (1.0 - true_p) * brier(q, false);
            let best = (0..=1000)
                .map(|i| i as f64 / 1000.0)
                .min_by(|a, b| expected(*a).partial_cmp(&expected(*b)).unwrap())
                .unwrap();
            assert!(
                (best - true_p).abs() <= 0.001,
                "true {true_p}, best report {best}"
            );
        }
    }

    #[test]
    fn vague_forecasts_score_worse_than_sharp_calibrated_ones() {
        // Events that happen 9 times in 10: the calibrated forecaster says 0.9, the vague one hedges at 0.5.
        let outcomes: Vec<bool> = (0..100).map(|i| i % 10 != 0).collect();
        let total = |p: f64| outcomes.iter().map(|o| brier(p, *o)).sum::<f64>();
        assert!(total(0.9) < total(0.5));
        // Overconfidence is punished too.
        assert!(total(0.9) < total(1.0));
    }

    #[test]
    fn wis_rewards_tight_correct_intervals_and_punishes_misses() {
        let tight = wis_log(&q(0.4, 0.5, 0.6), 0.5, COST_FLOOR_USD);
        let wide = wis_log(&q(0.01, 0.5, 20.0), 0.5, COST_FLOOR_USD);
        let miss = wis_log(&q(0.4, 0.5, 0.6), 5.0, COST_FLOOR_USD);
        assert!(tight.1 && wide.1 && !miss.1);
        assert!(tight.0 < wide.0, "a wide interval costs its width");
        assert!(tight.0 < miss.0, "a miss costs its distance");
    }

    #[test]
    fn wis_median_term_is_proper_for_the_median() {
        // The pinball loss at the median: among candidate medians, the sample median scores best.
        let actuals = [1.0, 2.0, 2.5, 3.0, 10.0];
        let total = |m: f64| {
            actuals
                .iter()
                .map(|a| wis_log(&q(m, m, m), *a, TIME_FLOOR_S).0)
                .sum::<f64>()
        };
        let best = [1.0, 2.0, 2.5, 3.0, 10.0]
            .into_iter()
            .min_by(|a, b| total(*a).partial_cmp(&total(*b)).unwrap())
            .unwrap();
        assert_eq!(best, 2.5);
    }

    #[test]
    fn zero_cost_is_floored_not_undefined() {
        // M4: a job that failed before any model call costs $0, and still scores.
        let (score, _) = wis_log(&q(0.1, 0.5, 2.0), 0.0, COST_FLOOR_USD);
        assert!(score.is_finite());
    }

    #[test]
    fn padding_deliverables_never_lowers_the_score() {
        // M4: listing paths that will not change adds a little, never subtracts.
        let changed: BTreeSet<String> = ["README.md".to_string()].into();
        let honest = deliverable_score(&[("README.md".into(), 0.95)], &changed);
        let mut padded_list = vec![("README.md".to_string(), 0.95)];
        for i in 0..9 {
            padded_list.push((format!("pad-{i}.md"), 0.01));
        }
        let padded = deliverable_score(&padded_list, &changed);
        assert!(padded.sum >= honest.sum);
        // And a complete miss stays a miss however much is padded.
        let miss = deliverable_score(&[("OTHER.md".into(), 0.95)], &changed);
        let mut padded_miss = vec![("OTHER.md".to_string(), 0.95)];
        padded_miss.extend((0..9).map(|i| (format!("pad-{i}.md"), 0.01)));
        assert!(deliverable_score(&padded_miss, &changed).sum >= miss.sum);
        assert!(miss.sum > 1.0);
    }

    #[test]
    fn omitting_a_changed_path_never_helps() {
        let changed: BTreeSet<String> = ["a.rs".to_string(), "b.rs".to_string()].into();
        let listed_low = deliverable_score(&[("a.rs".into(), 0.9), ("b.rs".into(), 0.1)], &changed);
        let omitted = deliverable_score(&[("a.rs".into(), 0.9)], &changed);
        assert!(listed_low.sum <= omitted.sum);
    }

    #[test]
    fn reference_uses_priors_without_history_and_moves_with_it() {
        let priors = athena_data::Config::parse(include_str!("../../../athena.toml"))
            .unwrap()
            .forecast
            .reference;
        let none = reference_forecast(&priors, "docs", &[]);
        assert!((none.p_verification_passes - priors.p_verification_passes).abs() < 1e-12);
        assert_eq!(none.cost_usd.p50(), priors.cost_usd.p50);
        let failing: Vec<PastJob> = (0..6)
            .map(|_| PastJob {
                class: "docs".into(),
                reading_accepted: vec![true],
                verification_passed: Some(false),
                user_intervened: Some(false),
                cost_execution_usd: Some(1.0),
                wall_time_execution_s: Some(300.0),
            })
            .collect();
        let after = reference_forecast(&priors, "docs", &failing);
        assert!(after.p_verification_passes < none.p_verification_passes);
        assert!((after.cost_usd.p50() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn skill_is_withheld_below_the_minimum_job_count() {
        let reading = ReadingForecasts {
            n: 1,
            reader: Forecast {
                p_reading_accepted: 0.9,
                p_verification_passes: 0.9,
                p_user_intervenes: 0.1,
                cost_usd: q(0.1, 0.5, 1.0),
                wall_time_s: q(60.0, 300.0, 900.0),
            },
            reference: Forecast {
                p_reading_accepted: 0.7,
                p_verification_passes: 0.75,
                p_user_intervenes: 0.2,
                cost_usd: q(0.15, 0.6, 2.5),
                wall_time_s: q(120.0, 420.0, 1500.0),
            },
            deliverables: vec![("README.md".into(), 0.9)],
        };
        let resolution = Resolution {
            reading_accepted: BTreeMap::from([(1, Some(true))]),
            verification_passed: Some(true),
            user_intervened: false,
            intervention_provisional: true,
            changed_paths: ["README.md".to_string()].into(),
            cost_execution_usd: Some(0.4),
            wall_time_execution_s: Some(250.0),
            models_expected: vec!["claude-sonnet-5-5".into(), "claude-opus-5-5".into()],
            models_observed: vec!["claude-sonnet-5-5".into(), "claude-opus-5-5".into()],
        };
        let job = JobId::parse("20261010-153012-k7q2").unwrap();
        let at: jiff::Timestamp = "2026-10-10T16:00:00Z".parse().unwrap();
        let score = score_job(&job, at, None, &[reading], Some(1), &resolution);
        assert_eq!(score.events.len(), 3);
        assert!(
            score
                .events
                .iter()
                .any(|e| e.event == "user_intervenes" && e.provisional)
        );
        assert!(score.models.as_ref().unwrap().as_expected);
        let few: Vec<&Score> = std::iter::repeat_n(&score, 9).collect();
        assert_eq!(aggregate(&few).event_skill, None);
        let enough: Vec<&Score> = std::iter::repeat_n(&score, 10).collect();
        let skill = aggregate(&enough).event_skill.unwrap();
        assert!(
            skill > 0.0,
            "a sharp, correct reader beats the base rate: {skill}"
        );
    }
}
