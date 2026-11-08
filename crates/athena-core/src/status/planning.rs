//! Planning cost, the keep rule, later harm and producer settings: the figures `athena status` derives
//! from the planning, critique and producer records of the request jobs.

use super::*;

/// How many planned jobs the first planning round accepted and stopped.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct FirstRoundCounts {
    pub accepted: usize,
    pub stopped: usize,
}

/// Planning spent per landed job over a group of landed jobs: the whole retry chain that led to each landing
/// (its own planning plus that of every job it retried), and the landed job's own planning beside it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct PlanningGroup {
    /// Landed, planned jobs in the group; each retry chain is counted once, at its landed job.
    pub jobs: usize,
    /// Mean planning rounds (job-plan records) over the chain.
    pub mean_rounds: f64,
    /// Mean planner and critic cost in USD over the chain.
    pub mean_cost_usd: f64,
    /// Mean planning rounds of the landed job alone.
    pub own_mean_rounds: f64,
    /// Mean planner and critic cost in USD of the landed job alone.
    pub own_mean_cost_usd: f64,
}

/// Planning rounds and cost per landed job, measured from the records and split at the first resumed plan.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanningCost {
    /// Landed jobs dispatched before the first resumed planning, or all of them when none resumed.
    pub before: PlanningGroup,
    /// Landed jobs dispatched at or after the first resumed planning; empty when none resumed.
    pub after: PlanningGroup,
    /// The first job whose plan resumed from a kept plan, with its dispatch time; the cutover.
    pub first_resumed: Option<(String, String)>,
}

/// Whether the first planning round of `job` accepted without a reading mismatch; `None` for a job with no critique.
pub(super) fn first_round_accepted(job: &JobRecords) -> Option<bool> {
    let first = job.critiques.first()?;
    Some(first.final_verdict == CritiqueVerdict::Accept && first.request_reading_mismatch.is_none())
}

/// Planned jobs up to which the early keep rule applies (job-planning.md section 9, D110): the landed share is
/// held at the baseline through the first 30 planned jobs.
pub const KEEP_RULE_EARLY_JOBS: usize = 30;
/// Planned jobs at which the keep / turn off / inconclusive decision is taken (job-planning.md section 9, D110).
/// Between the two thresholds the early rule continues: the rule states nothing else, and this keeps a breach
/// visible until the decision.
pub const KEEP_RULE_DECISION_JOBS: usize = 100;
/// The landed share planning is held to, and at or below which it is turned off at the decision
/// (job-planning.md section 9, D110).
pub const KEEP_RULE_BASELINE_SHARE: f64 = 0.64;
/// The landed share needed at the decision to keep planning, with the cost ceiling met
/// (job-planning.md section 9, D110).
pub const KEEP_RULE_TARGET_SHARE: f64 = 0.72;
/// The cost per landed job, in USD, that must not be exceeded to keep planning at the decision
/// (job-planning.md section 9, D110).
pub const KEEP_RULE_COST_CEILING_USD: f64 = 2.50;

/// What the job-planning keep rule says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum KeepVerdict {
    Keep,
    Breach,
    TurnOff,
    Inconclusive,
}

/// The job-planning keep rule (job-planning.md section 9, D110) evaluated over the records.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct KeepRule {
    /// Planned jobs that were dispatched; a job stopped in planning counts here and not as landed.
    pub planned: usize,
    pub landed: usize,
    /// `landed / planned`.
    pub share: f64,
    pub baseline_share: f64,
    /// All invocation cost of the planned jobs, in USD.
    pub total_cost_usd: f64,
    /// `total_cost_usd / landed`; none when nothing landed.
    pub cost_per_landed_usd: Option<f64>,
    pub first_round_accepted: usize,
    /// Planned jobs with a critique.
    pub first_round_planned: usize,
    pub first_round_accept_rate: Option<f64>,
    pub verdict: KeepVerdict,
    /// Landings later cited as the cause of a failure, a revert or a fix job, for planned and unplanned jobs,
    /// reported beside the landed share. Reported only; the verdict above still rests on share and cost.
    pub later_harm: LaterHarm,
}

/// How one later record cites a landed job as a cause of trouble (job-planning.md section 9, D143).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LaterCause {
    /// An introspection of a later failure or rejection names the job in its cause or account of what happened.
    Failure,
    /// A later request that asks for a revert names the job.
    Revert,
    /// A later request that asks for a fix names the job.
    Fix,
}

/// Whether `text` holds a word that is one of `words`, ignoring case; a word is a run of letters.
fn has_word(text: &str, words: &[&str]) -> bool {
    text.split(|c: char| !c.is_ascii_alphabetic())
        .any(|w| words.iter().any(|x| w.eq_ignore_ascii_case(x)))
}

/// The ways later records cite the landed job `landed` as a cause, read from the records and never from the
/// job's own: a later introspection (a record id after the job's, about another job) that names it in its
/// cause or account of what happened; a later request job whose request text names it and asks for a revert or
/// a fix. A request that names the job without asking for either (a retry, a follow-up, a citation) is not a
/// cause. Whole-id matching only, so no record is ever rewritten and no new record kind is needed.
fn later_causes(records: &Records, landed: &JobRecords) -> Vec<LaterCause> {
    let id = landed.id();
    let mut found = Vec::new();
    let failed = records.introspections.iter().any(|i| {
        i.id.as_str() > id.as_str()
            && i.job.as_ref() != Some(id)
            && [i.cause.as_deref(), i.what_happened.as_deref()]
                .into_iter()
                .flatten()
                .any(|text| text_cites_job(text, id))
    });
    if failed {
        found.push(LaterCause::Failure);
    }
    let mut revert = false;
    let mut fix = false;
    for later in records.request_jobs() {
        if later.id().as_str() <= id.as_str() {
            continue;
        }
        let Some(text) = later.request.as_ref().map(|r| r.text.as_str()) else {
            continue;
        };
        if !text_cites_job(text, id) {
            continue;
        }
        revert |= has_word(text, &["revert", "reverts", "reverted", "reverting"]);
        fix |= has_word(text, &["fix", "fixes", "fixed", "fixing"]);
    }
    if revert {
        found.push(LaterCause::Revert);
    }
    if fix {
        found.push(LaterCause::Fix);
    }
    found
}

/// The landed share of one group of dispatched jobs, and how many of its landings were later cited as the
/// cause of a failure, a revert or a fix job.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct LaterHarmGroup {
    /// Dispatched jobs in the group (a planned job stopped in planning counts here, not as landed).
    pub jobs: usize,
    pub landed: usize,
    /// `landed / jobs`; none for an empty group.
    pub share: Option<f64>,
    /// Landings cited at least once, in any of the three ways below.
    pub cited: usize,
    /// Landings a later introspection names as the cause of a failure or rejection.
    pub failures: usize,
    /// Landings a later request asks to revert.
    pub reverts: usize,
    /// Landings a later request asks to fix.
    pub fixes: usize,
}

/// The measure the keep rule adds to the landed share: landings that later caused harm, for planned and for
/// unplanned jobs. Landed share counts a landing as a success even when it later stops drains or has to be
/// reverted, as job 20261027-054211-a2n5 did; this counts those landings against the group they came from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct LaterHarm {
    pub planned: LaterHarmGroup,
    pub unplanned: LaterHarmGroup,
}

/// Whether a job counts as planned for the keep rule: it has plans and was dispatched or stopped in planning.
fn is_planned(job: &JobRecords) -> bool {
    !job.plans.is_empty() && (job.job.is_some() || plan_state(job).is_some_and(|s| !s.accepted))
}

/// Landings that later caused harm, per planned and unplanned dispatched job. Unplanned means dispatched with
/// no plan record, as every job is when planning is off or the job is not hard.
pub fn later_harm(records: &Records) -> LaterHarm {
    let mut harm = LaterHarm::default();
    for job in records.request_jobs() {
        let planned = is_planned(job);
        // Not planned and not dispatched (queued, proposed, declined): in neither group.
        if !planned && job.job.is_none() {
            continue;
        }
        let group = if planned {
            &mut harm.planned
        } else {
            &mut harm.unplanned
        };
        group.jobs += 1;
        if !matches!(job.state(), JobState::Closed(OutcomeState::Landed)) {
            continue;
        }
        group.landed += 1;
        let causes = later_causes(records, job);
        group.cited += usize::from(!causes.is_empty());
        group.failures += usize::from(causes.contains(&LaterCause::Failure));
        group.reverts += usize::from(causes.contains(&LaterCause::Revert));
        group.fixes += usize::from(causes.contains(&LaterCause::Fix));
    }
    for group in [&mut harm.planned, &mut harm.unplanned] {
        group.share = (group.jobs > 0).then(|| group.landed as f64 / group.jobs as f64);
    }
    harm
}

/// The verdict of the keep rule for `planned` jobs with the given share and cost per landed job.
fn keep_verdict(planned: usize, share: f64, cost_per_landed: Option<f64>) -> KeepVerdict {
    if planned < KEEP_RULE_DECISION_JOBS {
        return if share < KEEP_RULE_BASELINE_SHARE {
            KeepVerdict::Breach
        } else {
            KeepVerdict::Keep
        };
    }
    if share >= KEEP_RULE_TARGET_SHARE
        && cost_per_landed.is_some_and(|c| c <= KEEP_RULE_COST_CEILING_USD)
    {
        KeepVerdict::Keep
    } else if share <= KEEP_RULE_BASELINE_SHARE {
        KeepVerdict::TurnOff
    } else {
        KeepVerdict::Inconclusive
    }
}

/// Evaluates the keep rule over the planned, dispatched jobs. `None` when no job was planned and dispatched.
pub fn keep_rule(records: &Records) -> Option<KeepRule> {
    let planned: Vec<&JobRecords> = records
        .request_jobs()
        .into_iter()
        .filter(|j| is_planned(j))
        .collect();
    if planned.is_empty() {
        return None;
    }
    let landed = planned
        .iter()
        .filter(|j| matches!(j.state(), JobState::Closed(OutcomeState::Landed)))
        .count();
    let total_cost_usd: f64 = planned.iter().map(|j| j.spent_usd()).sum();
    let share = landed as f64 / planned.len() as f64;
    let cost_per_landed_usd = (landed > 0).then(|| total_cost_usd / landed as f64);
    let rounds: Vec<bool> = planned
        .iter()
        .filter_map(|j| first_round_accepted(j))
        .collect();
    let first_round_accepted = rounds.iter().filter(|a| **a).count();
    Some(KeepRule {
        planned: planned.len(),
        landed,
        share,
        baseline_share: KEEP_RULE_BASELINE_SHARE,
        total_cost_usd,
        cost_per_landed_usd,
        first_round_accepted,
        first_round_planned: rounds.len(),
        first_round_accept_rate: (!rounds.is_empty())
            .then(|| first_round_accepted as f64 / rounds.len() as f64),
        verdict: keep_verdict(planned.len(), share, cost_per_landed_usd),
        later_harm: later_harm(records),
    })
}

/// The most predecessors followed back from a landed job.
const MAX_CHAIN: usize = 16;

/// Planner and critic cost of the invocations of `job`, in USD.
fn planning_usd(job: &JobRecords) -> f64 {
    job.invocations
        .iter()
        .filter(|i| matches!(i.role, Role::Planner | Role::Critic))
        .map(|i| i.cost.best_lower())
        .sum()
}

/// When `job` was dispatched, from its job record; the time of its first resumed plan if that is missing.
fn dispatched_at(job: &JobRecords) -> Option<jiff::Timestamp> {
    job.job.as_ref().and_then(|j| j.dispatched_at).or_else(|| {
        job.plans
            .iter()
            .find(|p| p.resumed_from.is_some())
            .map(|p| p.at)
    })
}

/// Planning rounds and cost per landed job, with chain totals over the jobs each landing retried. `None` when
/// no landed job was planned. The split is at the dispatch of the first job with a resumed plan, so that
/// 'after' reads 'since the first resumed planning'. A landed job with no recorded dispatch time counts as before.
pub fn planning_cost(records: &Records) -> Option<PlanningCost> {
    let cutover = records
        .request_jobs()
        .into_iter()
        .filter(|j| j.plans.iter().any(|p| p.resumed_from.is_some()))
        .filter_map(|j| dispatched_at(j).map(|at| (at, j.id().as_str().to_string())))
        .min();
    let mut sums = [PlanningGroup::default(); 2];
    for job in records.request_jobs() {
        if job.plans.is_empty() || !matches!(job.state(), JobState::Closed(OutcomeState::Landed)) {
            continue;
        }
        let mut rounds = job.plans.len();
        let mut cost = planning_usd(job);
        let own = (rounds as f64, cost);
        let mut seen = vec![job.id().as_str().to_string()];
        let mut current = job;
        for _ in 0..MAX_CHAIN {
            let Some(text) = current.request.as_ref().map(|r| r.text.as_str()) else {
                break;
            };
            let Some(earlier) = crate::reading::retried_job(records, text)
                .filter(|id| !seen.iter().any(|s| s == id.as_str()))
                .and_then(|id| records.jobs.get(id.as_str()))
            else {
                break;
            };
            seen.push(earlier.id().as_str().to_string());
            rounds += earlier.plans.len();
            cost += planning_usd(earlier);
            current = earlier;
        }
        let after = match (&cutover, dispatched_at(job)) {
            (Some((cut, _)), Some(at)) => at >= *cut,
            _ => false,
        };
        let group = &mut sums[usize::from(after)];
        group.jobs += 1;
        group.mean_rounds += rounds as f64;
        group.mean_cost_usd += cost;
        group.own_mean_rounds += own.0;
        group.own_mean_cost_usd += own.1;
    }
    if sums[0].jobs + sums[1].jobs == 0 {
        return None;
    }
    for group in &mut sums {
        if group.jobs > 0 {
            let n = group.jobs as f64;
            group.mean_rounds /= n;
            group.mean_cost_usd /= n;
            group.own_mean_rounds /= n;
            group.own_mean_cost_usd /= n;
        }
    }
    let [before, after] = sums;
    Some(PlanningCost {
        before,
        after,
        first_resumed: cutover.map(|(at, id)| (id, at.to_string())),
    })
}

/// The effort shown for a producer invocation recorded before effort was kept.
pub const EFFORT_UNRECORDED: &str = "unrecorded";

/// How dispatched jobs fared under one producer model and effort.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProducerSettingStats {
    pub model: String,
    pub effort: String,
    /// Dispatched jobs whose first producer invocation ran with this model and effort.
    pub jobs: usize,
    /// Jobs with a verification record, and how many of them passed at their first verifier run.
    pub verified: usize,
    pub first_pass: usize,
    /// `first_pass / verified`; none when no job was verified.
    pub first_pass_rate: Option<f64>,
    pub landed: usize,
    /// `landed / jobs`.
    pub landed_share: f64,
    /// All invocation cost of the jobs, in USD.
    pub total_cost_usd: f64,
    /// `total_cost_usd / landed`; none when nothing landed, never a division by zero.
    pub cost_per_landed_usd: Option<f64>,
}

/// First-pass verification, landed share and cost per landed job for each producer model and effort, measured
/// from the records of dispatched request jobs. A job counts under the model and effort of its first producer
/// invocation; a job that never ran a producer is left out. Sorted by model, then effort.
pub fn producer_settings(records: &Records) -> Vec<ProducerSettingStats> {
    use athena_data::records::Verdict;
    let mut buckets: BTreeMap<(String, String), ProducerSettingStats> = BTreeMap::new();
    for job in records.request_jobs() {
        if job.job.is_none() {
            continue;
        }
        let Some(producer) = job.invocations.iter().find(|i| i.role == Role::Producer) else {
            continue;
        };
        let effort = producer
            .effort
            .clone()
            .unwrap_or_else(|| EFFORT_UNRECORDED.to_string());
        let key = (producer.model.clone(), effort);
        let b = buckets
            .entry(key.clone())
            .or_insert_with(|| ProducerSettingStats {
                model: key.0.clone(),
                effort: key.1.clone(),
                jobs: 0,
                verified: 0,
                first_pass: 0,
                first_pass_rate: None,
                landed: 0,
                landed_share: 0.0,
                total_cost_usd: 0.0,
                cost_per_landed_usd: None,
            });
        b.jobs += 1;
        b.total_cost_usd += job.spent_usd();
        if let Some(v) = &job.verification {
            b.verified += 1;
            let verifier_runs = job
                .invocations
                .iter()
                .filter(|i| i.role == Role::Verifier)
                .count();
            if v.final_verdict == Verdict::Pass && verifier_runs <= 1 {
                b.first_pass += 1;
            }
        }
        if matches!(job.state(), JobState::Closed(OutcomeState::Landed)) {
            b.landed += 1;
        }
    }
    buckets
        .into_values()
        .map(|mut b| {
            b.first_pass_rate = (b.verified > 0).then(|| b.first_pass as f64 / b.verified as f64);
            b.landed_share = b.landed as f64 / b.jobs as f64;
            b.cost_per_landed_usd = (b.landed > 0).then(|| b.total_cost_usd / b.landed as f64);
            b
        })
        .collect()
}
