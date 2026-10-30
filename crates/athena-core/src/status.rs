//! `athena status`: the state of the work, derived from the records at `HEAD` and the state directory
//! (design §3.6). Nothing about the state of the work lives in a hand-maintained file (D35).

use crate::Athena;
use crate::manage::pending_quantities;
use crate::state::format_bytes;
use crate::store::{JobRecords, JobState, Records};
use anyhow::{Result, bail};
use athena_data::records::{
    Build, BuildMeasurement, Critique, CritiqueVerdict, InterventionSource, Invocation, JobPlan,
    OutcomeState, Reading, Role, SccacheFigures, text_cites_job,
};
use athena_gate::invariant::{BOOTSTRAP_OUTCOME, BootstrapRange, bootstrap_range};
use athena_git::Git;
use serde::Serialize;
use std::collections::BTreeMap;

/// The paths the installed binary is built from: Athena's own code and the files that build and install it. A
/// change to one of them after the build commit leaves the installed binary stale.
pub const BUILD_PATHS: [&str; 5] = ["crates/", "xtask/", "Cargo.toml", "Cargo.lock", "setup.sh"];

/// Whether any of `paths` is one the installed binary is built from.
pub fn touches_build(paths: &[String]) -> bool {
    paths.iter().any(|p| {
        BUILD_PATHS
            .iter()
            .any(|root| p == root || (root.ends_with('/') && p.starts_with(root)))
    })
}

/// The one line that says the binary built from `build_commit` is older than `head` for [`BUILD_PATHS`], or
/// None when it is not: the build commit is not in this repository (a build that names no commit, or another
/// repository's) or is not an ancestor of `head`, or none of those paths changed since it. Changes to
/// anything else, such as `docs/` or `records/`, never make a build stale.
pub fn stale_build_line(git: &Git, build_commit: &str, head: &str) -> Option<String> {
    if !git.commit_exists(build_commit).ok()? {
        return None;
    }
    git.run(["merge-base", "--is-ancestor", build_commit, head])
        .ok()?;
    let mut args = vec!["diff", "--name-only", build_commit, head, "--"];
    args.extend(BUILD_PATHS);
    let changed = git
        .run(args)
        .ok()?
        .lines()
        .filter(|l| !l.is_empty())
        .count();
    if changed == 0 {
        return None;
    }
    let short = |c: &str| c[..c.len().min(12)].to_string();
    Some(format!(
        "stale build: this athena was built from {}, and {changed} file(s) under crates/, xtask/, Cargo.toml, Cargo.lock or setup.sh changed by {}; run ./setup.sh",
        short(build_commit),
        short(head)
    ))
}

/// One job, in one line.
#[derive(Debug, Clone, Serialize)]
pub struct JobLine {
    pub id: String,
    pub state: String,
    pub title: String,
    pub class: Option<String>,
    pub cost_usd: f64,
    /// "plan accepted r<n>" or "plan stopped r<n>", for a job that went through job planning; absent otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    /// The job's priority: the latest priority record, else the request's words, else urgent if the reading
    /// gave an urgent reason, else normal (`JobRecords::priority`).
    pub priority: crate::plan::Priority,
    /// "environment: low disk: 3.0 GB free, 10.0 GB required" for a proposed job whose dispatch was held for
    /// lack of disk space; absent otherwise. The job is still queued to run, not rejected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<String>,
}

/// How job planning ended for one job: the round of the last critique and whether it let the job go on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanState {
    pub accepted: bool,
    pub round: u32,
}

impl PlanState {
    /// "plan accepted r2" or "plan stopped r3".
    pub fn text(self) -> String {
        format!(
            "plan {} r{}",
            if self.accepted { "accepted" } else { "stopped" },
            self.round
        )
    }
}

/// The plan state of a job: that of its last critique. A critique that accepts stands unless it also says the
/// reading misread the request, which stops the job. A job with no critique has no plan state.
pub fn plan_state(job: &JobRecords) -> Option<PlanState> {
    let last = job.critiques.last()?;
    Some(PlanState {
        accepted: last.final_verdict == CritiqueVerdict::Accept
            && last.request_reading_mismatch.is_none(),
        round: last.round,
    })
}

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
fn first_round_accepted(job: &JobRecords) -> Option<bool> {
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

/// How many of the newest jobs with a measured test check the durations report reads.
pub const CHECK_DURATION_JOBS: usize = 20;

/// The share of the check time limit above which the slowest test check is flagged.
pub const CHECK_DURATION_FLAG: f64 = 0.8;

/// How long the `test` check took over the newest jobs, against the limit a check is killed at. A suite that
/// creeps toward the limit shows here before a run is killed.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CheckDurations {
    /// The configured `check_timeout_s`.
    pub limit_s: u64,
    /// Jobs counted: the newest ones with a measured `test` run, at most [`CHECK_DURATION_JOBS`].
    pub jobs: usize,
    pub median_s: f64,
    pub slowest_s: f64,
    /// The job of the slowest run.
    pub slowest_job: String,
    /// True when the slowest run took more than [`CHECK_DURATION_FLAG`] of the limit.
    pub flagged: bool,
}

impl CheckDurations {
    /// The line `athena status` prints.
    pub fn text(&self) -> String {
        let mut line = format!(
            "checks     test: median {:.0}s · slowest {:.0}s ({}) of {}s limit over last {} jobs",
            self.median_s, self.slowest_s, self.slowest_job, self.limit_s, self.jobs
        );
        if self.flagged {
            line.push_str(&format!(
                " · over {:.0}% of the limit",
                CHECK_DURATION_FLAG * 100.0
            ));
        }
        line
    }
}

/// The seconds the `test` check took in a job, or none when the job ran no such check. A check that did not
/// run (a skipped check or the placeholder of a nested run, both measured as `not run`) has no duration.
fn test_check_seconds(job: &JobRecords) -> Option<f64> {
    let check = job
        .verification
        .as_ref()?
        .checks
        .iter()
        .find(|c| c.name == "test")?;
    if check.duration.basis() == "not run" {
        return None;
    }
    check.duration.value()
}

/// Median and slowest `test` check duration over the newest [`CHECK_DURATION_JOBS`] request jobs that ran
/// one, against `limit_s`. Jobs with no measured `test` run are skipped and do not use up the window. None
/// when no job qualifies. An even count takes the mean of the two middle values.
pub fn check_durations(records: &Records, limit_s: u64) -> Option<CheckDurations> {
    let runs: Vec<(&JobRecords, f64)> = records
        .request_jobs()
        .into_iter()
        .filter_map(|j| test_check_seconds(j).map(|s| (j, s)))
        .take(CHECK_DURATION_JOBS)
        .collect();
    let (slowest_job, slowest_s) = runs
        .iter()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(j, s)| (j.id().as_str().to_string(), *s))?;
    let mut seconds: Vec<f64> = runs.iter().map(|(_, s)| *s).collect();
    seconds.sort_by(f64::total_cmp);
    let mid = seconds.len() / 2;
    let median_s = if seconds.len() % 2 == 1 {
        seconds[mid]
    } else {
        (seconds[mid - 1] + seconds[mid]) / 2.0
    };
    Some(CheckDurations {
        limit_s,
        jobs: runs.len(),
        median_s,
        slowest_s,
        slowest_job,
        flagged: slowest_s > CHECK_DURATION_FLAG * limit_s as f64,
    })
}

/// How many of the newest landed jobs the throughput medians read. The throughput review compares the same
/// median from one review to the next, so the trigger and the status line share this window.
pub const THROUGHPUT_WINDOW: usize = 20;

/// Minutes a job spent in each stage. Reading is the reader's invocations; planning the planner's and the
/// critic's; producer and verifier their own. Checks is the sum of the durations Athena measured for the
/// job's checks. Landing and rebuild is one stage: the time from the end of the job's last invocation (the
/// verifier, which runs after the checks) to the later of the landing outcome and the landing record
/// (`landings/`, written after the merge), and never below zero. A job with no landing record ends at its outcome.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct StageMinutes {
    pub reading: f64,
    pub planning: f64,
    pub producer: f64,
    pub checks: f64,
    pub verifier: f64,
    pub landing_and_rebuild: f64,
}

impl StageMinutes {
    /// The six stages, named and in pipeline order.
    pub fn named(&self) -> [(&'static str, f64); 6] {
        [
            ("reading", self.reading),
            ("planning", self.planning),
            ("producer", self.producer),
            ("checks", self.checks),
            ("verifier", self.verifier),
            ("landing and rebuild", self.landing_and_rebuild),
        ]
    }

    /// The minutes of the whole job: the sum of the six stages.
    pub fn total(&self) -> f64 {
        self.named().iter().map(|(_, m)| m).sum()
    }
}

/// One landed job with the minutes it spent in each stage.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct LandedJob {
    pub id: String,
    /// The end of landing and rebuild: the later of the landing outcome and the landing record.
    pub landed_at: jiff::Timestamp,
    /// Whether a landing record (`landings/`) was found for the job; when not, `landed_at` is the outcome time.
    pub landing_found: bool,
    pub stages: StageMinutes,
}

/// Landed jobs per UTC day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DayCount {
    /// `YYYY-MM-DD`, in UTC.
    pub day: String,
    pub landed: usize,
}

/// How fast jobs go through the pipeline, computed from the job records alone.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Throughput {
    /// Request jobs that landed.
    pub landed_total: usize,
    /// Landed jobs per UTC day, oldest day first.
    pub landed_per_day: Vec<DayCount>,
    /// Landed jobs the medians cover: the newest ones with a measured stage, at most [`THROUGHPUT_WINDOW`].
    pub window_jobs: usize,
    /// The median of those jobs' total minutes; 0 when there are none.
    pub median_minutes: f64,
    /// The median of each stage over the same jobs.
    pub median_stages: StageMinutes,
    /// How many of the window's jobs have a landing record; the others end at their landing outcome.
    pub window_with_landing_record: usize,
    /// Minutes between jobs: the gaps between the merged busy intervals (invocations, checks, landing wait) of
    /// all request jobs.
    pub idle_minutes: f64,
    /// Request jobs that did not land but did work: closed rejected, failed or abandoned, or stopped in planning.
    pub unlanded_jobs: usize,
    /// Their time spent: invocation wall times plus measured check durations, not the time the job stayed open.
    pub unlanded_minutes: f64,
}

/// The median of `values`; an even count takes the mean of the two middle values; 0 for none.
pub fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        sorted[mid]
    } else {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    }
}

fn minutes_between(from: jiff::Timestamp, to: jiff::Timestamp) -> f64 {
    to.duration_since(from).as_secs_f64() / 60.0
}

/// The outcome that closed a job, the later one first.
fn closing_outcome(job: &JobRecords) -> Option<&athena_data::records::Outcome> {
    job.later_outcome.as_ref().or(job.outcome.as_ref())
}

/// The minutes of a job's invocations in the roles `roles`.
fn role_minutes(job: &JobRecords, roles: &[Role]) -> f64 {
    crate::store::positive_zero(
        job.invocations
            .iter()
            .filter(|i| roles.contains(&i.role))
            .map(|i| minutes_between(i.started_at, i.ended_at).max(0.0))
            .sum(),
    )
}

/// The minutes the job's checks took, as measured; a check that did not run counts nothing.
fn check_minutes(job: &JobRecords) -> f64 {
    job.verification.as_ref().map_or(0.0, |v| {
        crate::store::positive_zero(
            v.checks
                .iter()
                .filter(|c| c.duration.basis() != "not run")
                .filter_map(|c| c.duration.value())
                .sum::<f64>()
                / 60.0,
        )
    })
}

/// When the landing record for `job` was written: the newest `landings/` record with a job-landing commit
/// of the job. None when there is none.
fn landing_record_time(records: &Records, job: &JobRecords) -> Option<jiff::Timestamp> {
    records
        .landings
        .iter()
        .filter(|l| {
            l.commits.iter().any(|c| {
                c.kind == athena_data::history::LandingKind::JobLanding
                    && c.job.as_ref() == Some(job.id())
            })
        })
        .map(|l| l.created_at)
        .max()
}

/// The end of a landed job's landing and rebuild: the later of the landing outcome and the landing record.
/// The second value is whether a landing record was found.
fn landing_end(records: &Records, job: &JobRecords) -> Option<(jiff::Timestamp, bool)> {
    let outcome = closing_outcome(job)?.at;
    Some(match landing_record_time(records, job) {
        Some(t) => (outcome.max(t), true),
        None => (outcome, false),
    })
}

/// The intervals in which the job was worked: each invocation, its checks (placed by estimate, ending when the
/// verifier started, or when the last invocation ended if there is no verifier), and, for a landed job, the
/// wait from the last invocation's end to the landing end.
fn busy_intervals(records: &Records, job: &JobRecords) -> Vec<(jiff::Timestamp, jiff::Timestamp)> {
    let mut out: Vec<_> = job
        .invocations
        .iter()
        .filter(|i| i.ended_at >= i.started_at)
        .map(|i| (i.started_at, i.ended_at))
        .collect();
    let Some(last) = job.invocations.iter().map(|i| i.ended_at).max() else {
        return out;
    };
    let checks = check_minutes(job);
    if checks > 0.0 {
        let end = job
            .invocations
            .iter()
            .filter(|i| i.role == Role::Verifier)
            .map(|i| i.started_at)
            .min()
            .unwrap_or(last);
        if let Ok(start) = end.checked_sub(jiff::SignedDuration::from_secs_f64(checks * 60.0)) {
            out.push((start, end));
        }
    }
    if matches!(job.state(), JobState::Closed(OutcomeState::Landed))
        && let Some((end, _)) = landing_end(records, job)
        && end > last
    {
        out.push((last, end));
    }
    out
}

/// The minutes a job was worked: invocation wall times, plus measured check durations, plus (landed jobs) the
/// wait for the landing record. Time the job sat open between those counts nothing.
fn time_spent(records: &Records, job: &JobRecords) -> f64 {
    let invocations: f64 = job
        .invocations
        .iter()
        .map(|i| minutes_between(i.started_at, i.ended_at).max(0.0))
        .sum();
    let landing_wait = if matches!(job.state(), JobState::Closed(OutcomeState::Landed)) {
        let last = job.invocations.iter().map(|i| i.ended_at).max();
        match (last, landing_end(records, job)) {
            (Some(last), Some((end, _))) => minutes_between(last, end).max(0.0),
            _ => 0.0,
        }
    } else {
        0.0
    };
    crate::store::positive_zero(invocations + check_minutes(job) + landing_wait)
}

/// The landed jobs whose stages were measured, oldest landing first (ties by id). A landed job that ran no
/// model has nothing to measure and is left out.
pub fn landed_jobs(records: &Records) -> Vec<LandedJob> {
    let mut landed: Vec<LandedJob> = records
        .request_jobs()
        .into_iter()
        .filter(|j| matches!(j.state(), JobState::Closed(OutcomeState::Landed)))
        .filter_map(|j| {
            let (landed_at, landing_found) = landing_end(records, j)?;
            let last_end = j.invocations.iter().map(|i| i.ended_at).max()?;
            let checks = check_minutes(j);
            Some(LandedJob {
                id: j.id().as_str().to_string(),
                landed_at,
                landing_found,
                stages: StageMinutes {
                    reading: role_minutes(j, &[Role::Reader]),
                    planning: role_minutes(j, &[Role::Planner, Role::Critic]),
                    producer: role_minutes(j, &[Role::Producer]),
                    checks,
                    verifier: role_minutes(j, &[Role::Verifier]),
                    landing_and_rebuild: minutes_between(last_end, landed_at).max(0.0),
                },
            })
        })
        .collect();
    landed.sort_by(|a, b| (a.landed_at, &a.id).cmp(&(b.landed_at, &b.id)));
    landed
}

impl Throughput {
    /// Measures `records`. None when no request job landed, so a new repository shows no section.
    pub fn compute(records: &Records) -> Option<Throughput> {
        let landed_total = records
            .request_jobs()
            .into_iter()
            .filter(|j| matches!(j.state(), JobState::Closed(OutcomeState::Landed)))
            .count();
        if landed_total == 0 {
            return None;
        }
        let mut per_day: BTreeMap<String, usize> = BTreeMap::new();
        for job in records.request_jobs() {
            if matches!(job.state(), JobState::Closed(OutcomeState::Landed))
                && let Some(outcome) = closing_outcome(job)
            {
                let day: String = outcome.at.to_string().chars().take(10).collect();
                *per_day.entry(day).or_default() += 1;
            }
        }
        let measured = landed_jobs(records);
        let window = &measured[measured.len().saturating_sub(THROUGHPUT_WINDOW)..];
        let column = |pick: fn(&StageMinutes) -> f64| -> f64 {
            median(&window.iter().map(|j| pick(&j.stages)).collect::<Vec<_>>())
        };
        let median_stages = StageMinutes {
            reading: column(|s| s.reading),
            planning: column(|s| s.planning),
            producer: column(|s| s.producer),
            checks: column(|s| s.checks),
            verifier: column(|s| s.verifier),
            landing_and_rebuild: column(|s| s.landing_and_rebuild),
        };
        let median_minutes = median(&window.iter().map(|j| j.stages.total()).collect::<Vec<_>>());

        let mut spans: Vec<(jiff::Timestamp, jiff::Timestamp)> = Vec::new();
        let mut unlanded_jobs = 0;
        let mut unlanded_minutes = 0.0;
        for job in records.request_jobs() {
            if job.invocations.is_empty() {
                continue;
            }
            spans.extend(busy_intervals(records, job));
            let state = job.state();
            let stopped = plan_state(job).is_some_and(|s| !s.accepted);
            let failed = matches!(state, JobState::Closed(s) if s != OutcomeState::Landed);
            if failed || (stopped && state != JobState::Closed(OutcomeState::Landed)) {
                unlanded_jobs += 1;
                unlanded_minutes += time_spent(records, job);
            }
        }
        spans.sort();
        let mut idle_minutes = 0.0;
        let mut reach: Option<jiff::Timestamp> = None;
        for (start, end) in spans {
            if let Some(reach_end) = reach {
                if start > reach_end {
                    idle_minutes += minutes_between(reach_end, start);
                }
                reach = Some(reach_end.max(end));
            } else {
                reach = Some(end);
            }
        }
        Some(Throughput {
            landed_total,
            landed_per_day: per_day
                .into_iter()
                .map(|(day, landed)| DayCount { day, landed })
                .collect(),
            window_jobs: window.len(),
            median_minutes,
            median_stages,
            window_with_landing_record: window.iter().filter(|j| j.landing_found).count(),
            idle_minutes,
            unlanded_jobs,
            unlanded_minutes,
        })
    }

    /// The lines `athena status` prints, and the section the throughput review is given.
    pub fn text(&self) -> String {
        let stages = self
            .median_stages
            .named()
            .iter()
            .map(|(name, m)| format!("{name} {m:.1}"))
            .collect::<Vec<_>>()
            .join(" · ");
        let days = self
            .landed_per_day
            .iter()
            .map(|d| format!("{} {}", d.day, d.landed))
            .collect::<Vec<_>>()
            .join(" · ");
        let landing_basis = if self.window_with_landing_record == self.window_jobs {
            "landing record (landings/) found for each".to_string()
        } else if self.window_with_landing_record == 0 {
            "no landing record (landings/) found, so landing ends at the landing outcome"
                .to_string()
        } else {
            format!(
                "landing record (landings/) found for {} of {}, the rest end at the landing outcome",
                self.window_with_landing_record, self.window_jobs
            )
        };
        format!(
            "throughput {} landed ({days})\n           basis: landed jobs counted by their landing outcome's UTC day\n           median per landed job over the last {} measured: {:.1} min ({stages})\n           basis: invocation wall times, measured check durations, and landing and rebuild from the verifier's end to the later of the outcome and the landing record; {landing_basis}\n           idle between jobs {:.1} min\n           basis: gaps between merged busy intervals of all jobs (invocations, checks placed by estimate before the verifier, landing wait), not the time a job stayed open\n           did not land: {} job(s), {:.1} min\n           basis: time spent (invocation wall times plus measured check durations), not the time from dispatch to close",
            self.landed_total,
            self.window_jobs,
            self.median_minutes,
            self.idle_minutes,
            self.unlanded_jobs,
            self.unlanded_minutes
        )
    }
}

/// One class's forecast headline.
#[derive(Debug, Clone, Serialize)]
pub struct ClassHeadline {
    pub class: String,
    pub scored_jobs: usize,
    pub reader_mean_brier: Option<f64>,
    pub reference_mean_brier: Option<f64>,
}

/// A quantity that is not known yet.
#[derive(Debug, Clone, Serialize)]
pub struct Pending {
    pub record: String,
    pub field: String,
    pub basis: String,
    pub resolved_by: Option<String>,
}

/// The bootstrap's range of unverified commits.
#[derive(Debug, Clone, Serialize)]
pub struct Bootstrap {
    pub range_open: bool,
    pub through_commit: Option<String>,
    pub note: String,
}

/// Where a requirement's measure can appear in `athena status`: the label `athena status` prints at the start
/// of a line, and the [`StatusView`] field that holds what it prints. A measure in `docs/spec.toml` ends with
/// `status: <label>` using one of these, or with `status: not shown` and names the test or check that shows
/// it. The Hestia spec pass flags a measure whose label is not here, and the `xtask` spec test requires the
/// marker.
pub const STATUS_MEASURE_LABELS: &[(&str, &str)] = &[
    ("throughput", "throughput"),
    ("trend", "class_trend"),
    ("checks", "check_durations"),
    ("planning", "planning_cost"),
    ("keep rule", "keep_rule"),
    ("producers", "producer_settings"),
    ("mistakes", "mistakes"),
    ("repeated", "repeated_causes"),
    ("code health", "code_health"),
    ("audit", "audit"),
    ("audience", "audience"),
    ("requirement", "spec_gaps"),
];

/// The line `athena status` prints for the latest build measurement (`athena measure-build`): T (one cold
/// target directory), S (one worktree's sources), the wall time of each cold build and the compile-cache
/// counts, with the commit and time it was taken. With no measurement it says so and how to make one.
pub fn build_measurement_line(latest: Option<&BuildMeasurement>) -> String {
    let Some(m) = latest else {
        return "build      measurement: none yet (run `athena measure-build`, or the Hestia sweep makes one)"
            .into();
    };
    let cache = match m.sccache {
        SccacheFigures::NotInstalled => "sccache not installed".to_string(),
        SccacheFigures::Measured { first, second } => format!(
            "sccache build 1 {} hits / {} misses, build 2 {} hits / {} misses",
            first.hits, first.misses, second.hits, second.misses
        ),
    };
    format!(
        "build      measurement: T {} target (one cold target directory) · S {} source (one worktree) · build 1 {:.2}s, build 2 {:.2}s · {cache} · commit {} at {}",
        format_bytes(m.target_bytes),
        format_bytes(m.source_bytes),
        m.first_build_seconds,
        m.second_build_seconds,
        &m.commit[..m.commit.len().min(12)],
        m.at
    )
}

/// The state of the work.
#[derive(Debug, Clone, Serialize)]
pub struct StatusView {
    pub repository: String,
    pub branch: Option<String>,
    pub head: String,
    pub athena: Build,
    /// Set when the installed binary's build commit is older than `head` for the paths it is built from
    /// ([`stale_build_line`]): the line `athena status` and the agent-running commands print.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_build: Option<String>,
    pub driver: String,
    pub kill_switch_on: bool,
    pub nested: Option<String>,
    pub bootstrap: Bootstrap,
    pub jobs_by_state: BTreeMap<String, usize>,
    pub recent: Vec<JobLine>,
    pub open: Vec<JobLine>,
    /// First-round accept and stop counts over planned jobs; absent when no job was planned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planning: Option<FirstRoundCounts>,
    /// Planning rounds and cost per landed job, before and after the first resumed planning; absent when no
    /// planned job landed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planning_cost: Option<PlanningCost>,
    /// The job-planning keep rule (job-planning.md section 9, D110) over planned dispatched jobs; absent when
    /// no job was planned and dispatched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_rule: Option<KeepRule>,
    /// First-pass verification rate, landed share and cost per landed job by producer model and effort; empty
    /// when no dispatched job ran a producer.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub producer_settings: Vec<ProducerSettingStats>,
    /// Median and slowest `test` check duration over the newest jobs, against the check time limit; absent
    /// when no job ran a measured test check or the configuration cannot be read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_durations: Option<CheckDurations>,
    /// Landed jobs per day, median minutes per landed job by stage, idle time between jobs and time spent on
    /// jobs that did not land, computed from the job records; absent until a request job has landed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throughput: Option<Throughput>,
    /// The latest build measurement as one printed line ([`build_measurement_line`]), or the line that says
    /// there is none yet.
    pub build_measurement: String,
    /// Classes of work whose minutes per landed job fell or rose between their older and newer landed jobs,
    /// with the job after which a rising class got worse; absent until a class has enough landed jobs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_trend: Option<crate::class_trend::ClassTrend>,
    pub interventions_by_category: BTreeMap<String, usize>,
    pub interventions_by_class: BTreeMap<String, usize>,
    /// Mistakes per class and per cause, with the prevention proposed. Apart from `interventions_by_category`,
    /// which counts each mistake under its own category as before.
    pub mistakes: MistakeReport,
    /// Interventions that call for follow-up and that no later request job cites yet (see
    /// [`open_interventions`]). They are not in `open`, which lists jobs.
    pub open_interventions: Vec<OpenIntervention>,
    pub restarts: usize,
    pub forecasts: Vec<ClassHeadline>,
    pub pending: Vec<Pending>,
    /// Introspections no class-label record names yet (`crate::labeled::unreviewed`); `athena review` clears
    /// them one at a time.
    pub unreviewed_introspections: usize,
    /// Open defaults proposals (`athena.defaults-proposal/1`): pending ones only.
    pub proposals: Vec<ProposalView>,
    /// Open steward proposals (work items of kind proposal with no decision), decided by id with
    /// `athena approve-work` or `athena reject-work`.
    pub work_proposals: Vec<WorkProposalView>,
    /// The latest audience rating (`crate::audience`), if one is recorded.
    pub audience: Option<AudienceView>,
    /// The latest audit of a job's outcome (`crate::auditor`), if one is recorded.
    pub audit: Option<AuditView>,
    /// The code-health table (`crate::code_health`); absent when the sources could not be measured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_health: Option<crate::code_health::CodeHealth>,
    /// Lines of work the scope guard flagged or paused, with their signals T1 to T5 (`crate::scope_guard`).
    /// Pauses first. Empty when no mark file exists.
    pub lines: Vec<crate::scope_guard::LineMark>,
    /// The open repeated causes from Hestia's error review (`crate::hestia::repeated_causes`): causes that
    /// occurred in two or more distinct jobs and whose root-cause request has not landed, each with its count
    /// and the request that addresses it. Empty when there are none. Causes in a stage that `athena.toml` has
    /// switched off are not here; see `dormant_causes`.
    pub repeated_causes: Vec<crate::hestia::RepeatedCause>,
    /// The open repeated causes whose stage (planner or critic) `athena.toml` has switched off: listed apart,
    /// never queued by the error review backfill that Athena runs at drain start and in each sweep.
    pub dormant_causes: Vec<crate::hestia::DormantCause>,
    /// The active requirements of `docs/spec.toml` with no mechanism, or with a measure that is blank or names
    /// a label `athena status` does not print (`crate::hestia::spec_gaps`). Empty when every active requirement
    /// has both, or when the repository has no spec. Athena queues one request for each at the next drain or
    /// sweep.
    pub spec_gaps: Vec<crate::hestia::SpecGap>,
    pub invalid_records: Vec<String>,
    pub next: Vec<String>,
    /// The reconciliation that ran before this view was built (`Athena::status_with_reconcile`); absent from
    /// [`Athena::status`], which asks nothing of the remote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reconcile: Option<crate::reconcile::ReconcileReport>,
    /// Set when the view was built from an evidence snapshot (`Athena::status_from_snapshot`) and not from
    /// this checkout: where the records and history came from, and what the view leaves out.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<SnapshotNote>,
}

/// Where a snapshot-built [`StatusView`] came from, and what it does not show.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SnapshotNote {
    /// The snapshot directory given to `athena status --snapshot`.
    pub path: String,
    /// The commits in the snapshot's history file; `head` is the newest.
    pub commits: usize,
    /// The sections left out because they need the live repository or process state, not records or history.
    pub omitted: Vec<String>,
}

/// The sections a snapshot-built view leaves out.
const SNAPSHOT_OMITS: [&str; 6] = [
    "branch",
    "stale build",
    "kill switch",
    "scope-guard lines",
    "code health",
    "spec gaps",
];

/// An intervention that still calls for follow-up, as `athena status` lists it under open work.
#[derive(Debug, Clone, Serialize)]
pub struct OpenIntervention {
    pub id: String,
    pub what: String,
    pub category: String,
    /// The job the intervention was recorded against, if any.
    pub job: Option<String>,
    /// The command that starts the follow-up.
    pub next: String,
}

/// The interventions that call for follow-up: those recorded by the user (`source` `user`) that no later
/// request job cites. A request job cites an intervention when its request text names the intervention's
/// record id and it was received at or after the intervention was recorded. Interventions Athena observed
/// itself (a correction of a reading) are already applied to the job that took them, so they are not listed.
/// Oldest first.
pub fn open_interventions(records: &Records) -> Vec<OpenIntervention> {
    records
        .interventions
        .iter()
        .filter(|i| i.source == InterventionSource::User)
        .filter(|i| {
            !records.request_jobs().iter().any(|job| {
                job.request
                    .as_ref()
                    .is_some_and(|r| r.received_at >= i.at && r.text.contains(i.id.as_str()))
            })
        })
        .map(|i| OpenIntervention {
            id: i.id.to_string(),
            what: i.what.clone(),
            category: i.category.as_str().into(),
            job: i.job.as_ref().map(|j| j.as_str().to_string()),
            next: format!(
                "athena ask \"<request that addresses intervention {}>\"",
                i.id
            ),
        })
        .collect()
}

/// The cause text under which a mistake is counted when no recorded introspection gives one.
pub const NO_CAUSE: &str = "no cause recorded";

/// Prevention work proposed for the mistakes that share one cause.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MistakeProposal {
    pub cause: String,
    /// `check` (a code check), `knowledge` (a default the readers are given) or `design change`.
    pub kind: String,
    /// What would have prevented the mistakes of this cause.
    pub prevention: String,
    /// The introspections that propose it.
    pub introspections: Vec<String>,
    /// How many mistakes it would have prevented.
    pub mistakes: usize,
}

/// The mistakes in the records: counted per class of work and per cause, apart from the intervention
/// categories. A mistake is an intervention for which [`athena_data::records::is_mistake`] holds: category
/// `mistake`, or a legacy record with category `other` whose text starts "Mistake:". Records are not rewritten.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct MistakeReport {
    pub total: usize,
    pub by_class: BTreeMap<String, usize>,
    /// The cause the recorded introspection gives, grouped by [`crate::introspect::normalise`]; a mistake with
    /// no recorded introspection counts under [`NO_CAUSE`].
    pub by_cause: BTreeMap<String, usize>,
    /// One proposal per cause.
    pub proposals: Vec<MistakeProposal>,
}

/// Counts the mistakes in `records` and gathers the prevention proposed for them, one proposal per cause.
pub fn mistake_report(records: &Records) -> MistakeReport {
    use athena_data::records::{IntrospectionStatus, PreventionKind};
    let mut report = MistakeReport::default();
    // Normalised cause -> (display text, proposal).
    let mut groups: BTreeMap<String, MistakeProposal> = BTreeMap::new();
    for i in records.interventions.iter().filter(|i| i.is_mistake()) {
        report.total += 1;
        *report.by_class.entry(i.class_of_work.clone()).or_default() += 1;
        let recorded = records
            .introspections
            .iter()
            .rfind(|n| n.intervention.as_ref() == Some(&i.id))
            .filter(|n| n.status == IntrospectionStatus::Recorded);
        let Some(n) = recorded else {
            *report.by_cause.entry(NO_CAUSE.into()).or_default() += 1;
            continue;
        };
        let cause = n.cause.as_deref().unwrap_or("").trim();
        let key = crate::introspect::normalise(cause);
        if key.is_empty() {
            *report.by_cause.entry(NO_CAUSE.into()).or_default() += 1;
            continue;
        }
        let proposal = groups.entry(key).or_insert_with(|| MistakeProposal {
            cause: cause.to_string(),
            kind: match n.prevention_kind {
                Some(PreventionKind::CodeCheck) => "check",
                Some(PreventionKind::Default) => "knowledge",
                Some(PreventionKind::DesignChange) => "design change",
                None => "unknown",
            }
            .into(),
            prevention: n.prevention.clone().unwrap_or_default(),
            introspections: Vec::new(),
            mistakes: 0,
        });
        proposal.mistakes += 1;
        let id = n.id.to_string();
        if !proposal.introspections.contains(&id) {
            proposal.introspections.push(id);
        }
    }
    for p in groups.into_values() {
        *report.by_cause.entry(p.cause.clone()).or_default() += p.mistakes;
        report.proposals.push(p);
    }
    report
}

/// The latest audience rating as `athena status` lists it.
#[derive(Debug, Clone, Serialize)]
pub struct AudienceView {
    pub id: String,
    pub target: String,
    pub audience: String,
    pub rating: u8,
    pub at: String,
    /// Each finding with the evidence it cites.
    pub findings: Vec<String>,
}

fn audience_view(records: &Records) -> Option<AudienceView> {
    crate::audience::latest(records).map(|r| AudienceView {
        id: r.id.to_string(),
        target: r.target.clone(),
        audience: r.audience.clone(),
        rating: r.rating,
        at: r.at.to_string(),
        findings: r
            .findings
            .iter()
            .map(|f| format!("{} (evidence: {})", f.text, f.source))
            .collect(),
    })
}

/// The latest audit as `athena status` lists it.
#[derive(Debug, Clone, Serialize)]
pub struct AuditView {
    pub id: String,
    /// The job whose outcome was judged.
    pub job: String,
    pub summary: String,
    pub at: String,
    /// Each finding with the evidence it cites.
    pub findings: Vec<String>,
}

fn audit_view(records: &Records) -> Option<AuditView> {
    records.audits.last().map(|a| AuditView {
        id: a.id.to_string(),
        job: a.target.as_str().to_string(),
        summary: a.summary.clone(),
        at: a.at.to_string(),
        findings: a
            .findings
            .iter()
            .map(|f| format!("{} (evidence: {})", f.text, f.evidence))
            .collect(),
    })
}

/// An open defaults proposal as `athena status` lists it (`defaults-loop.md` section 9).
#[derive(Debug, Clone, Serialize)]
pub struct ProposalView {
    pub id: String,
    pub kind: String,
    pub statement: String,
    pub applies_to: Vec<String>,
    pub version_from: u32,
    pub version_to: u32,
    /// The failure class's name and id.
    pub failure_class: String,
    /// The proposal record that holds it.
    pub record: String,
    /// The introspections it cites, each with its job.
    pub sources: Vec<String>,
}

/// An open steward proposal as `athena status` lists it.
#[derive(Debug, Clone, Serialize)]
pub struct WorkProposalView {
    pub id: String,
    pub title: String,
    pub rung: String,
    /// The source and the evidence, each as `kind value`.
    pub source: String,
    pub evidence: Vec<String>,
}

fn work_proposal_views(records: &Records) -> Vec<WorkProposalView> {
    let show =
        |r: &athena_data::records::Reference| format!("{:?} {}", r.kind, r.value).to_lowercase();
    crate::work_decisions::open_proposals(records)
        .into_iter()
        .map(|w| WorkProposalView {
            id: w.id.to_string(),
            title: w.title.clone(),
            rung: w.rung.as_str().into(),
            source: show(&w.source),
            evidence: w.evidence.iter().map(show).collect(),
        })
        .collect()
}

fn proposal_views(records: &Records) -> Vec<ProposalView> {
    crate::proposals::open(&records.proposals)
        .into_iter()
        .map(|p| ProposalView {
            id: p.id.clone(),
            kind: p.kind.as_str().into(),
            statement: p.statement.clone(),
            applies_to: p.applies_to.clone(),
            version_from: p.version.from,
            version_to: p.version.to,
            failure_class: format!(
                "{} ({})",
                records
                    .classes
                    .iter()
                    .find(|c| c.id == p.failure_class)
                    .map_or("unknown class", |c| c.name.as_str()),
                p.failure_class
            ),
            record: p.record.to_string(),
            sources: p
                .sources
                .iter()
                .map(|s| format!("introspection {} (job {})", s.introspection, s.job))
                .collect(),
        })
        .collect()
}

/// One invocation in a job's trace.
#[derive(Debug, Clone, Serialize)]
pub struct InvocationLine {
    pub seq: u32,
    pub role: String,
    pub model: String,
    pub outcome: String,
    pub cost: String,
    pub tokens: String,
    pub wall_time_s: Option<f64>,
    pub commands_run: usize,
    pub denials: usize,
}

/// An introspection as `athena status <job>` shows it (`defaults-loop.md` section 3.2).
#[derive(Debug, Clone, Serialize)]
pub struct IntrospectionView {
    pub id: String,
    pub trigger: String,
    pub intervention: Option<String>,
    pub what_happened: Option<String>,
    /// The class's name and id; `proposed` marks a class this introspection introduced.
    pub failure_class: Option<String>,
    pub new_class: bool,
    /// Earlier cases, each with its job where it has one.
    pub earlier_cases: Vec<String>,
    pub decisions_matched: Vec<String>,
    pub cause: Option<String>,
    pub prevention_kind: Option<String>,
    pub prevention: Option<String>,
    pub proposed_request: Option<String>,
    pub status: String,
    pub failure: Option<String>,
    pub flags: Vec<String>,
    pub dropped_ids: Vec<String>,
    /// False: the class and prevention kind came from a language model; Jev runs in shadow only (see `jev`).
    pub from_jev: bool,
    pub cost_usd: Option<f64>,
    /// Jev's shadow answers, when it was asked. They decide nothing.
    pub jev: Option<athena_data::records::IntrospectionJev>,
}

/// The introspections of one job, oldest first, with class names and the jobs of earlier cases.
fn introspection_views(records: &Records, job: &athena_data::JobId) -> Vec<IntrospectionView> {
    let class_name = |id: &athena_data::RecordId| {
        let name = records
            .classes
            .iter()
            .find(|c| &c.id == id)
            .map_or("unknown class", |c| c.name.as_str());
        format!("{name} ({id})")
    };
    records
        .introspections
        .iter()
        .filter(|i| i.job.as_ref() == Some(job))
        .map(|i| IntrospectionView {
            id: i.id.to_string(),
            trigger: i.trigger.as_str().into(),
            intervention: i.intervention.as_ref().map(ToString::to_string),
            what_happened: i.what_happened.clone(),
            failure_class: i.failure_class.as_ref().map(class_name),
            new_class: i.new_class,
            earlier_cases: i
                .earlier_cases
                .iter()
                .map(|case| {
                    let job = records
                        .introspections
                        .iter()
                        .find(|c| &c.id == case)
                        .and_then(|c| c.job.as_ref());
                    match job {
                        Some(job) => format!("{case} (job {job})"),
                        None => case.to_string(),
                    }
                })
                .collect(),
            decisions_matched: i.decisions_matched.clone(),
            cause: i.cause.clone(),
            prevention_kind: i.prevention_kind.map(|k| k.as_str().into()),
            prevention: i.prevention.clone(),
            proposed_request: i.proposed_request.clone(),
            status: i.status.as_str().into(),
            failure: i.failure.clone(),
            flags: i.flags.clone(),
            dropped_ids: i.dropped_ids.clone(),
            from_jev: i.from_jev,
            cost_usd: i.evidence.as_ref().map(|e| e.cost_usd),
            jev: i.jev.clone(),
        })
        .collect()
}

/// A job's full trace: request → prediction → owner → model → cost → verification → outcome → score.
#[derive(Debug, Clone, Serialize)]
pub struct JobTrace {
    pub id: String,
    pub state: String,
    pub request: Option<String>,
    pub readings: Vec<Reading>,
    pub corrections: Vec<String>,
    pub accepted_reading: Option<u32>,
    pub owner: Option<String>,
    pub verifier: Option<String>,
    pub invocations: Vec<InvocationLine>,
    /// The job's plan state, plans and critiques by round; all absent for a job that was not planned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub plans: Vec<JobPlan>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub critiques: Vec<Critique>,
    pub candidate: Option<serde_json::Value>,
    pub verification: Option<serde_json::Value>,
    pub outcome: Option<serde_json::Value>,
    pub later_outcome: Option<serde_json::Value>,
    pub score: Option<serde_json::Value>,
    pub interventions: Vec<String>,
    /// What went wrong or was changed, and why the system allowed it (`athena.introspection/1`).
    pub introspections: Vec<IntrospectionView>,
    pub commits: Vec<String>,
    pub directory: String,
}

fn job_line(job: &JobRecords, state: &str) -> JobLine {
    JobLine {
        id: job.id().as_str().to_string(),
        state: state.to_string(),
        title: job.title(),
        class: job.class().map(str::to_string),
        cost_usd: job.spent_usd(),
        plan: plan_state(job).map(PlanState::text),
        priority: job.priority(),
        hold: None,
    }
}

/// A number without float noise: at most six decimals, trailing zeros removed.
fn number(v: f64) -> String {
    let text = format!("{v:.6}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text.is_empty() {
        "0".into()
    } else {
        text.to_string()
    }
}

fn quantity_text(q: &athena_data::Quantity) -> String {
    match (q.value(), q.lower(), q.upper()) {
        (Some(v), _, _) => {
            format!("{} {} ({:?})", number(v), q.unit(), q.provenance()).to_lowercase()
        }
        (None, l, u) => format!(
            "[{}, {}] {} ({:?})",
            l.map_or("?".into(), number),
            u.map_or("?".into(), number),
            q.unit(),
            q.provenance()
        )
        .to_lowercase(),
    }
}

fn invocation_line(i: &Invocation) -> InvocationLine {
    let t = &i.tokens;
    InvocationLine {
        seq: i.seq,
        role: i.role.as_str().into(),
        model: i.model.clone(),
        outcome: format!("{:?}", i.outcome).to_lowercase(),
        cost: quantity_text(&i.cost),
        tokens: format!(
            "in {} · out {} · cache read {} · cache write {} ({:?})",
            t.input.best_lower(),
            t.output.best_lower(),
            t.cache_read.best_lower(),
            t.cache_write.best_lower(),
            t.output.provenance()
        )
        .to_lowercase(),
        wall_time_s: i.wall_time.value(),
        commands_run: i.commands_run.len(),
        denials: i.policy_denials.len(),
    }
}

impl Athena {
    /// Whether a process of the registry still works on `job`.
    pub fn running(&self, job: &str) -> bool {
        self.state.registry().entries().iter().any(|e| e.job == job)
    }

    fn state_name(&self, job: &JobRecords) -> String {
        match job.state() {
            JobState::Dispatched if self.running(job.id().as_str()) => "running".into(),
            JobState::Dispatched => "interrupted".into(),
            other => other.name().into(),
        }
    }

    /// The stale-build line when this binary's build commit is older than `HEAD` for the paths it is built
    /// from, else None ([`stale_build_line`]). Never an error: a repository that cannot be read has no line.
    pub fn stale_build(&self) -> Option<String> {
        stale_build_line(&self.git, &self.build.source_commit, &self.head().ok()?)
    }

    /// The mistakes in the records at `HEAD`, per class and per cause, with the prevention proposed.
    pub fn mistakes(&self) -> Result<MistakeReport> {
        let head = self.head()?;
        Ok(mistake_report(&Records::load(&self.git, &head)?))
    }

    /// The state of the work.
    pub fn status(&self) -> Result<StatusView> {
        self.state.registry().reap();
        self.status_from(None)
    }

    /// The state of the work as `status` reports it, but with the records and history read from the evidence
    /// snapshot at `dir` (`crate::snapshot`) and not from this checkout. `head` is the snapshot's newest commit.
    /// Job states come from the records alone, since no process of the snapshot's repository can be asked; the
    /// branch, stale build, kill switch, scope-guard lines, code health and spec gaps are left out and named in
    /// the view's `snapshot` note. Everything computed from the records, including the throughput section, is
    /// the same code as `status`.
    pub fn status_from_snapshot(&self, dir: &std::path::Path) -> Result<StatusView> {
        self.status_from(Some(dir))
    }

    fn status_from(&self, snapshot: Option<&std::path::Path>) -> Result<StatusView> {
        let (head, records, set, snapshot_note) = match snapshot {
            None => {
                let head = self.head()?;
                let records = Records::load(&self.git, &head)?;
                let set = athena_gate::records::RecordSet::load(&self.git, &head)?;
                (head, records, set, None)
            }
            Some(dir) => {
                let history = crate::snapshot::read_history(dir)?;
                let Some(newest) = history.first() else {
                    bail!("{} holds no history: history.json is empty", dir.display());
                };
                let note = SnapshotNote {
                    path: dir.display().to_string(),
                    commits: history.len(),
                    omitted: SNAPSHOT_OMITS.iter().map(|s| s.to_string()).collect(),
                };
                (
                    newest.hash.clone(),
                    crate::snapshot::load_records(dir)?,
                    crate::snapshot::record_set(dir)?,
                    Some(note),
                )
            }
        };
        let from_snapshot = snapshot_note.is_some();
        let mut jobs_by_state: BTreeMap<String, usize> = BTreeMap::new();
        let mut recent = Vec::new();
        let mut open = Vec::new();
        let mut first_round: Option<FirstRoundCounts> = None;
        for job in records.request_jobs() {
            if let Some(accepted) = first_round_accepted(job)
                && let Some(state) = plan_state(job)
            {
                let counts = first_round.get_or_insert(FirstRoundCounts {
                    accepted: 0,
                    stopped: 0,
                });
                // A round-1 revise sends the plan back; only a job whose planning ended in round 1 stopped there.
                if accepted {
                    counts.accepted += 1;
                } else if !state.accepted && state.round == 1 {
                    counts.stopped += 1;
                }
            }
            // No process of the snapshot's repository can be asked, so a snapshot names the state its records give.
            let state = if from_snapshot {
                job.state().name().to_string()
            } else {
                self.state_name(job)
            };
            *jobs_by_state.entry(state.clone()).or_default() += 1;
            let mut line = job_line(job, &state);
            if state == "proposed" && !from_snapshot {
                line.hold = crate::disk::read_hold(&self.state, job.id()).map(|h| h.text());
            }
            // Queued requests have not run; they show under open work.
            if recent.len() < 10 && state != "queued" {
                recent.push(line.clone());
            }
            if matches!(
                state.as_str(),
                "queued" | "proposed" | "declined" | "interrupted" | "running" | "blocked"
            ) {
                open.push(line);
            }
        }
        let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
        let mut by_class: BTreeMap<String, usize> = BTreeMap::new();
        for i in &records.interventions {
            *by_category.entry(i.category.as_str().into()).or_default() += 1;
            *by_class.entry(i.class_of_work.clone()).or_default() += 1;
        }
        let mut per_class: BTreeMap<String, Vec<&athena_data::records::Score>> = BTreeMap::new();
        for job in records.request_jobs() {
            if let (Some(score), Some(class)) = (job.latest_score(), job.class()) {
                per_class.entry(class.to_string()).or_default().push(score);
            }
        }
        let forecasts = per_class
            .into_iter()
            .map(|(class, scores)| {
                let agg = athena_score::aggregate(&scores);
                let mean = |name: &str| {
                    agg.per_forecaster
                        .get(name)
                        .filter(|t| t.events > 0)
                        .map(|t| t.event_brier_sum / t.events as f64)
                };
                ClassHeadline {
                    class,
                    scored_jobs: scores.len(),
                    reader_mean_brier: mean("reader"),
                    reference_mean_brier: mean("reference"),
                }
            })
            .collect();
        let resolutions: BTreeMap<(String, String), String> = records
            .resolutions
            .iter()
            .map(|r| {
                (
                    (r.target_path.clone(), r.target_field.clone()),
                    format!("records/resolutions/{}/{}.json", r.id.month(), r.id),
                )
            })
            .collect();
        let pending: Vec<Pending> = pending_quantities(&set)
            .into_iter()
            .map(|(record, field, basis)| {
                let resolved_by = resolutions.get(&(record.clone(), field.clone())).cloned();
                Pending {
                    record,
                    field,
                    basis,
                    resolved_by,
                }
            })
            .collect();
        let range = if from_snapshot {
            // The same reading as `bootstrap_range`, from the snapshot's copy of the outcome record.
            match set.bytes(BOOTSTRAP_OUTCOME) {
                None => BootstrapRange::Open,
                Some(bytes) => {
                    match athena_data::records::AnyRecord::parse(BOOTSTRAP_OUTCOME, bytes) {
                        Ok(athena_data::records::AnyRecord::Outcome(outcome)) => {
                            BootstrapRange::ClosedThrough(
                                outcome.through_commit.clone().unwrap_or_default(),
                            )
                        }
                        _ => BootstrapRange::ClosedThrough(String::new()),
                    }
                }
            }
        } else {
            bootstrap_range(&self.git, &head)?
        };
        let bootstrap = match &range {
            BootstrapRange::Open => Bootstrap {
                range_open: true,
                through_commit: None,
                note: format!(
                    "OPEN: commits labeled Athena-Job: bootstrap may still change governed content unverified. The bootstrap \
                     closes its range by adding {BOOTSTRAP_OUTCOME}, naming its last commit (docs/records.md)."
                ),
            },
            BootstrapRange::ClosedThrough(through) => Bootstrap {
                range_open: false,
                through_commit: Some(through.clone()),
                note: "closed: governed content changes only through verified landings and declared interventions".into(),
            },
        };
        let kill_switch_on = !from_snapshot && self.state.registry().stopped();
        let mut next = Vec::new();
        if kill_switch_on {
            next.push("athena start   # the kill switch is on; new work is blocked".into());
        }
        for line in &open {
            let id = &line.id;
            match line.state.as_str() {
                "queued" => next.push(format!("athena go {id}   # reads it first, then asks to run it · or: athena close {id} --why \"...\"")),
                "proposed" => next.push(format!("athena go {id}   # or: athena correct {id} \"<line>\" · athena close {id} --why \"...\"")),
                "declined" => next.push(format!("athena correct {id} \"<line>\"   # or: athena close {id} --why \"...\"")),
                "interrupted" => next.push(format!("athena retry {id}   # reads it again to resume · or: athena close {id} --why \"...\" (commits what the interrupted job left)")),
                "blocked" => next.push(format!("athena go {id}   # after resolving local edits; or: athena close {id} --why \"...\"")),
                _ => {}
            }
        }
        let lines = if from_snapshot {
            Vec::new()
        } else {
            crate::scope_guard::list_lines(&self.state)
        };
        for l in lines.iter().filter(|l| l.is_pause()) {
            next.push(format!(
                "line {} is paused ({})   # {}",
                l.line,
                l.signal_names(),
                l.lift
            ));
        }
        let open_causes: Vec<crate::hestia::RepeatedCause> =
            crate::hestia::repeated_causes(&records)
                .into_iter()
                .filter(crate::hestia::RepeatedCause::is_open)
                .collect();
        // The stages athena.toml switches off are read from the configuration at HEAD; an unreadable
        // configuration leaves every cause live.
        // A snapshot holds no configuration; this checkout's `athena.toml` (the base's, in a job's working copy)
        // answers.
        let job_config = if from_snapshot {
            self.head()
                .ok()
                .and_then(|local| crate::capability::JobConfig::at(&self.git, &local).ok())
        } else {
            crate::capability::JobConfig::at(&self.git, &head).ok()
        };
        let disabled = job_config
            .as_ref()
            .map(|jc| crate::hestia::Stage::disabled(&jc.config))
            .unwrap_or_default();
        let check_durations = job_config
            .as_ref()
            .and_then(|jc| check_durations(&records, jc.config.budget.check_timeout_s));
        let (repeated_causes, dormant_causes) =
            crate::hestia::split_dormant(open_causes, &disabled);
        for c in repeated_causes.iter().filter(|c| c.request.is_none()) {
            next.push(format!(
                "repeated cause in {} jobs has no root-cause request   # {} · Athena queues it at the next drain or sweep (athena hestia review --backfill does it now)",
                c.jobs.len(),
                c.cause
            ));
        }
        // A spec that cannot be read or parsed lists nothing here; the sweep's spec pass reports it.
        let spec_gaps = if from_snapshot {
            Vec::new()
        } else {
            crate::hestia::spec_gaps_at(&self.git, &head).unwrap_or_default()
        };
        for g in &spec_gaps {
            next.push(g.next_line());
        }
        let open_interventions = open_interventions(&records);
        for i in &open_interventions {
            next.push(format!(
                "{}   # follow up intervention {} ({})",
                i.next, i.id, i.category
            ));
        }
        if bootstrap.range_open {
            next.push(format!("close the bootstrap range: commit {BOOTSTRAP_OUTCOME} (docs/records.md, \"Closing the bootstrap\")"));
        }
        for p in pending.iter().filter(|p| p.resolved_by.is_none()) {
            next.push(format!("resolve {}#{}: add an athena.resolution/1 record (docs/records.md, \"Resolving a pending quantity\")", p.record, p.field));
        }
        let proposals = proposal_views(&records);
        for p in &proposals {
            let id = &p.id;
            next.push(format!("athena accept-proposal {id}   # runs a job that edits defaults.toml · or: athena reject-proposal {id} --why \"...\""));
        }
        let work_proposals = work_proposal_views(&records);
        for w in &work_proposals {
            let id = &w.id;
            next.push(format!("athena approve-work {id}   # submits a job that carries the proposal's cited content · or: athena reject-work {id} --why \"...\""));
        }
        let unreviewed = crate::labeled::unreviewed(&records.introspections, &records.class_labels);
        if let Some(oldest) = unreviewed.first() {
            next.push(format!(
                "athena review {}   # {} introspection(s) have no class label; one of: --right · --class <id> · --new-class <name> --description \"...\"",
                oldest.id,
                unreviewed.len()
            ));
        }
        if records.request_jobs().is_empty() {
            next.push("athena ask \"<a one- or two-line request>\"".into());
        }
        let code_health = if from_snapshot {
            None
        } else {
            crate::code_health::compute(&self.git, &head, &records, crate::now())
                .ok()
                .map(|health| health.shown())
        };
        let stale_build = if from_snapshot {
            None
        } else {
            stale_build_line(&self.git, &self.build.source_commit, &head)
        };
        Ok(StatusView {
            repository: self.git.dir().display().to_string(),
            branch: if from_snapshot {
                None
            } else {
                self.git.current_branch()?
            },
            head,
            athena: self.build.clone(),
            stale_build,
            driver: format!("{} {}", self.driver.info().name, self.driver.info().version),
            kill_switch_on,
            nested: self.nested.clone(),
            bootstrap,
            jobs_by_state,
            recent,
            open,
            planning: first_round,
            planning_cost: planning_cost(&records),
            keep_rule: keep_rule(&records),
            producer_settings: producer_settings(&records),
            check_durations,
            throughput: Throughput::compute(&records),
            build_measurement: build_measurement_line(BuildMeasurement::latest(
                &records.build_measurements,
            )),
            class_trend: crate::class_trend::ClassTrend::compute(&records),
            interventions_by_category: by_category,
            interventions_by_class: by_class,
            mistakes: mistake_report(&records),
            open_interventions,
            restarts: records.restarts.len(),
            forecasts,
            pending,
            unreviewed_introspections: unreviewed.len(),
            proposals,
            work_proposals,
            audience: audience_view(&records),
            audit: audit_view(&records),
            code_health,
            lines,
            repeated_causes,
            dormant_causes,
            spec_gaps,
            invalid_records: records.invalid.clone(),
            next,
            reconcile: None,
            snapshot: snapshot_note,
        })
    }

    /// A job's full trace.
    pub fn trace(&self, job_ref: &str) -> Result<JobTrace> {
        let head = self.head()?;
        let records = Records::load(&self.git, &head)?;
        let id = records.resolve(job_ref)?;
        let job = records.jobs.get(id.as_str()).expect("resolved");
        let to_value = |v: &dyn erased::ToValue| v.to_value();
        let mut commits: Vec<String> = self
            .git
            .run([
                "log",
                "--format=%h %s",
                "-E",
                &format!("--grep=^Athena-Job: {}$", id.as_str()),
                &head,
            ])?
            .lines()
            .map(str::to_string)
            .collect();
        if crate::gh::cutover_job_for(&self.git, &head).is_some() {
            // After C0 (D111) commits carry no trailers: list those that touch the job's records, then the
            // host commit of the job's landing record, which is the squash commit on the default branch.
            let mut more = self
                .git
                .run(["log", "--format=%h %s", &head, "--", &id.dir()])?
                .lines()
                .map(str::to_string)
                .collect::<Vec<_>>();
            if let Ok(Some(host)) = athena_gate::attribution::cutover_commit(&self.git, id.as_str())
                && self.git.commit_exists(&host)?
            {
                let line = self
                    .git
                    .run(["log", "-1", "--format=%h %s", &host])?
                    .trim()
                    .to_string();
                more.push(line);
            }
            for line in more {
                if !commits.contains(&line) {
                    commits.push(line);
                }
            }
        }
        Ok(JobTrace {
            id: id.as_str().to_string(),
            state: self.state_name(job),
            request: job.request.as_ref().map(|r| r.text.clone()),
            readings: job.readings.clone(),
            corrections: job
                .corrections
                .iter()
                .map(|c| format!("reading {}: {}", c.n, c.text))
                .collect(),
            accepted_reading: job.job.as_ref().and_then(|j| j.accepted_reading),
            owner: job.job.as_ref().map(|j| {
                format!(
                    "{} · capability {} · model {}",
                    j.owner.role,
                    j.owner.capability.clone().unwrap_or_default(),
                    j.owner.model.clone().unwrap_or_default()
                )
            }),
            verifier: job.job.as_ref().and_then(|j| j.verifier_model.clone()),
            invocations: job.invocations.iter().map(invocation_line).collect(),
            plan: plan_state(job).map(PlanState::text),
            plans: job.plans.clone(),
            critiques: job.critiques.clone(),
            candidate: job.candidate.as_ref().map(|c| to_value(c)),
            verification: job.verification.as_ref().map(|v| to_value(v)),
            outcome: job.outcome.as_ref().map(|o| to_value(o)),
            later_outcome: job.later_outcome.as_ref().map(|o| to_value(o)),
            score: job.latest_score().map(|s| to_value(s)),
            interventions: records
                .interventions_for(&id)
                .iter()
                .map(|i| format!("{} [{}] {}", i.id, i.category.as_str(), i.what))
                .collect(),
            introspections: introspection_views(&records, &id),
            commits,
            directory: id.dir(),
        })
    }
}

/// Whether a state is terminal.
pub fn is_terminal(state: OutcomeState) -> bool {
    !matches!(state, OutcomeState::Blocked)
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod mistake_tests {
    use super::*;
    use athena_data::RecordId;
    use athena_data::records::{
        Intervention, InterventionCategory, IntrospectionRecord, IntrospectionStatus,
        IntrospectionTrigger, PreventionKind,
    };
    use jiff::Timestamp;

    fn at() -> Timestamp {
        "2026-10-17T12:00:00Z".parse().unwrap()
    }

    fn rid(n: u32) -> RecordId {
        RecordId::parse(&format!("20261017-1200{n:02}-1abc")).unwrap()
    }

    fn intervention(
        n: u32,
        class: &str,
        category: InterventionCategory,
        what: &str,
    ) -> Intervention {
        Intervention {
            id: rid(n),
            at: at(),
            what: what.into(),
            why: "why".into(),
            class_of_work: class.into(),
            category,
            job: None,
            source: InterventionSource::User,
            with_staged_changes: false,
        }
    }

    fn introspection(
        n: u32,
        of: u32,
        cause: &str,
        kind: PreventionKind,
        prevention: &str,
    ) -> IntrospectionRecord {
        IntrospectionRecord {
            id: rid(n),
            at: at(),
            trigger: IntrospectionTrigger::Intervention,
            job: None,
            intervention: Some(rid(of)),
            class_of_work: "docs".into(),
            category: "mistake".into(),
            what_happened: Some("it happened".into()),
            failure_class: None,
            new_class: false,
            earlier_cases: Vec::new(),
            decisions_matched: Vec::new(),
            cause: Some(cause.into()),
            prevention_kind: Some(kind),
            prevention: Some(prevention.into()),
            proposed_request: Some("do it".into()),
            evidence: None,
            status: IntrospectionStatus::Recorded,
            failure: None,
            flags: Vec::new(),
            dropped_ids: Vec::new(),
            planning_stopped: false,
            from_jev: false,
            jev: None,
        }
    }

    #[test]
    fn mistakes_count_per_class_and_per_cause_and_same_cause_gives_one_proposal() {
        use InterventionCategory::Mistake;
        let mut records = Records::default();
        records.interventions = vec![
            intervention(1, "docs", Mistake, "skipped the check"),
            intervention(2, "docs", Mistake, "skipped it again"),
            intervention(3, "code", Mistake, "wrong file"),
            intervention(4, "code", Mistake, "no introspection"),
            intervention(5, "code", InterventionCategory::CorrectedOutput, "not one"),
        ];
        records.introspections = vec![
            introspection(
                11,
                1,
                "Nothing runs the docs test.",
                PreventionKind::CodeCheck,
                "Run it.",
            ),
            introspection(
                12,
                2,
                "nothing runs the docs test",
                PreventionKind::CodeCheck,
                "Run it.",
            ),
            introspection(
                13,
                3,
                "The brief omits file names.",
                PreventionKind::Default,
                "Name files.",
            ),
        ];
        let report = mistake_report(&records);
        assert_eq!(report.total, 4);
        assert_eq!(report.by_class["docs"], 2);
        assert_eq!(report.by_class["code"], 2);
        assert_eq!(report.by_cause["Nothing runs the docs test."], 2);
        assert_eq!(report.by_cause["The brief omits file names."], 1);
        assert_eq!(report.by_cause[NO_CAUSE], 1);
        assert_eq!(report.proposals.len(), 2, "same cause, one proposal");
        let shared = report
            .proposals
            .iter()
            .find(|p| p.mistakes == 2)
            .expect("the shared cause");
        assert_eq!(shared.kind, "check");
        assert_eq!(shared.introspections.len(), 2);
        let other = report.proposals.iter().find(|p| p.mistakes == 1).unwrap();
        assert_eq!(other.kind, "knowledge");
        assert_eq!(other.prevention, "Name files.");
    }

    #[test]
    fn a_legacy_other_record_starting_mistake_colon_is_counted_and_not_rewritten() {
        use InterventionCategory::Other;
        let mut records = Records::default();
        records.interventions = vec![
            intervention(1, "docs", Other, "Mistake: forgot the changelog"),
            intervention(2, "docs", Other, "A mistake: not counted"),
        ];
        let before = records.interventions.clone();
        let report = mistake_report(&records);
        assert_eq!(report.total, 1);
        assert_eq!(report.by_class["docs"], 1);
        assert_eq!(records.interventions, before, "the records are unchanged");
        assert_eq!(records.interventions[0].category, Other);
    }

    #[test]
    fn a_failed_introspection_gives_no_cause() {
        let mut records = Records::default();
        records.interventions = vec![intervention(1, "docs", InterventionCategory::Mistake, "x")];
        let mut n = introspection(11, 1, "c", PreventionKind::Default, "p");
        n.status = IntrospectionStatus::Failed;
        n.failure = Some("no".into());
        records.introspections = vec![n];
        let report = mistake_report(&records);
        assert_eq!(report.by_cause[NO_CAUSE], 1);
        assert!(report.proposals.is_empty());
    }

    #[test]
    fn the_report_is_empty_with_no_mistakes() {
        assert_eq!(mistake_report(&Records::default()).total, 0);
    }

    #[test]
    fn every_measure_label_names_a_status_view_field_and_a_printed_line() {
        let here = include_str!("status.rs");
        let display = include_str!("../../athena-cli/src/display.rs");
        // The `requirement` line is a next step built where the spec gaps are found.
        let hestia = include_str!("hestia.rs");
        for (label, field) in STATUS_MEASURE_LABELS {
            assert!(
                here.contains(&format!("    pub {field}:")),
                "StatusView has no field {field} for the label {label:?}"
            );
            let printed = format!("\"{label} ");
            let is_printed = [here, display, hestia]
                .iter()
                .any(|source| source.contains(&printed));
            assert!(
                is_printed,
                "`athena status` prints no line starting {label:?}"
            );
        }
    }
}

#[cfg(test)]
mod build_measurement_tests {
    use super::*;
    use crate::commits::{NewFile, message};
    use athena_data::RecordId;
    use athena_data::records::{AnyRecord, SccacheCounts};

    fn measurement(
        id: &str,
        hours_ago: i64,
        target: u64,
        sccache: SccacheFigures,
    ) -> BuildMeasurement {
        BuildMeasurement {
            id: RecordId::parse(id).unwrap(),
            at: crate::now()
                .checked_sub(jiff::SignedDuration::from_hours(hours_ago))
                .unwrap(),
            commit: "0123456789abcdef0123456789abcdef01234567".into(),
            cargo_lock_hash: "89abcdef0123456789abcdef0123456789abcdef".into(),
            target_bytes: target,
            source_bytes: 2_500_000,
            first_build_seconds: 61.5,
            second_build_seconds: 12.25,
            sccache,
        }
    }

    fn commit(athena: &Athena, m: &BuildMeasurement) {
        let record = AnyRecord::BuildMeasurement(Box::new(m.clone()));
        athena
            .commit_records(
                &[NewFile::record(&record)],
                &message(
                    "Build measurement",
                    "",
                    &[("Athena-Record", record.canonical_path())],
                ),
            )
            .unwrap();
    }

    #[test]
    fn the_line_shows_t_s_both_times_and_the_cache_counts_or_that_there_is_no_cache() {
        let measured = measurement(
            "20261029-090000-5abc",
            0,
            4_100_000_000,
            SccacheFigures::Measured {
                first: SccacheCounts { hits: 0, misses: 9 },
                second: SccacheCounts { hits: 4, misses: 5 },
            },
        );
        let line = build_measurement_line(Some(&measured));
        for want in [
            "T 4.1 GB",
            "S 2.5 MB",
            "build 1 61.50s",
            "build 2 12.25s",
            "sccache build 1 0 hits / 9 misses",
            "build 2 4 hits / 5 misses",
            "commit 0123456789ab",
        ] {
            assert!(line.contains(want), "{want:?} not in {line}");
        }
        let plain = measurement("20261029-090000-5abc", 0, 5, SccacheFigures::NotInstalled);
        let line = build_measurement_line(Some(&plain));
        assert!(line.contains("sccache not installed"), "{line}");
        assert!(!line.contains("hits"), "{line}");
        let none = build_measurement_line(None);
        assert!(none.contains("measurement: none yet"), "{none}");
        assert!(none.contains("athena measure-build"), "{none}");
    }

    #[test]
    fn build_measurement_status_shows_the_latest_record_and_a_clear_line_when_there_is_none() {
        let (_dir, _state, athena) = crate::code_health::tests::set_up();
        let before = athena.status().unwrap();
        assert!(
            before.build_measurement.contains("none yet"),
            "{}",
            before.build_measurement
        );

        let older = measurement(
            "20261029-080000-1abc",
            5,
            1_000_000,
            SccacheFigures::NotInstalled,
        );
        let newer = measurement(
            "20261029-090000-2abc",
            1,
            7_300_000_000,
            SccacheFigures::Measured {
                first: SccacheCounts { hits: 0, misses: 3 },
                second: SccacheCounts { hits: 1, misses: 2 },
            },
        );
        // Committed newest first: the latest is picked by time, not by file order.
        commit(&athena, &newer);
        commit(&athena, &older);

        let view = athena.status().unwrap();
        let line = &view.build_measurement;
        assert!(line.contains("T 7.3 GB"), "{line}");
        assert!(line.contains("S 2.5 MB"), "{line}");
        assert!(line.contains("build 1 61.50s, build 2 12.25s"), "{line}");
        assert!(line.contains("build 2 1 hits / 2 misses"), "{line}");
        assert!(!line.contains("not installed"), "{line}");
        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["build_measurement"], line.as_str());
    }
}

#[cfg(test)]
pub(crate) mod planning_cost_tests {
    use super::*;
    use athena_data::records::{
        AnyRecord, DriverInfo, InvocationCaps, InvocationContext, InvocationOutcome, ModelUsage,
        Request, Tokens,
    };
    use athena_data::{JobId, Quantity, Unit};

    fn id(text: &str) -> JobId {
        JobId::parse(text).unwrap()
    }

    pub(crate) fn invocation(job: &JobId, seq: u32, role: Role, usd: f64) -> Invocation {
        let tokens = |v: f64| Quantity::measured(v, Unit::Tokens, "result.usage");
        Invocation {
            job: job.clone(),
            seq,
            role,
            driver: DriverInfo {
                name: "claude-code".into(),
                version: "2.1.299".into(),
            },
            athena: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            model: "m".into(),
            effort: None,
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
            started_at: jiff::Timestamp::UNIX_EPOCH,
            ended_at: jiff::Timestamp::UNIX_EPOCH,
            wall_time: Quantity::measured(60.0, Unit::Seconds, "athena clock"),
            tokens: Tokens {
                input: tokens(1.0),
                output: tokens(1.0),
                cache_read: tokens(1.0),
                cache_write: tokens(1.0),
            },
            models_used: BTreeMap::from([(
                "m".to_string(),
                ModelUsage {
                    input: 1,
                    output: 1,
                    cache_read: 1,
                    cache_write: 1,
                    cost_usd: usd,
                },
            )]),
            cost: Quantity::reported(usd, Unit::Usd, "Claude Code total_cost_usd"),
            outcome: InvocationOutcome::Success,
            detail: None,
            turns: None,
            commands_run: vec![],
            policy_denials: vec![],
        }
    }

    fn plan(job: &JobId, round: u32, resumed_from: Option<JobId>) -> JobPlan {
        JobPlan {
            job: job.clone(),
            round,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: 1,
            approach: "Approach.".into(),
            decisions: vec![],
            files: vec![],
            steps: vec![],
            risks: vec![],
            demonstrations: vec![],
            open_questions: vec![],
            producer_section_sha256: "0".repeat(64),
            resumed_from,
        }
    }

    /// A job with `rounds` plans, each costing `usd` for the planner and the same for the critic, a producer
    /// invocation that must not count, a request text, and the outcome.
    fn job(
        text: &str,
        request: &str,
        dispatched: &str,
        rounds: u32,
        resumed_from: Option<&str>,
        state: OutcomeState,
        usd: f64,
    ) -> JobRecords {
        let job_id = id(text);
        let mut j = JobRecords {
            id: Some(job_id.clone()),
            request: Some(Request {
                job: job_id.clone(),
                text: request.into(),
                received_at: jiff::Timestamp::UNIX_EPOCH,
                channel: "cli".into(),
            }),
            ..JobRecords::default()
        };
        let mut stub: athena_data::records::Job = serde_json::from_value(serde_json::json!({
            "id": text, "kind": "request", "title": "t", "class_of_work": null,
            "capability": null, "accepted_reading": null, "accepted_by": null,
            "owner": { "role": "producer", "capability": null, "model": null, "note": null },
            "verifier_model": null, "caps": null, "base_commit": null, "config_sha256": null,
            "defaults_sha256": null, "athena": null, "driver": null, "brief_sha256": null,
            "dispatched_at": null, "request_path": null, "request_sha256": null, "cost": null,
            "tokens": null, "note": null, "pin": null
        }))
        .unwrap();
        stub.dispatched_at = Some(dispatched.parse().unwrap());
        j.job = Some(stub);
        for round in 1..=rounds {
            let from = (round == 1).then(|| resumed_from.map(id)).flatten();
            j.plans.push(plan(&job_id, round, from));
            j.invocations
                .push(invocation(&job_id, 2 * round, Role::Planner, usd));
            j.invocations
                .push(invocation(&job_id, 2 * round + 1, Role::Critic, usd));
        }
        j.invocations
            .push(invocation(&job_id, 99, Role::Producer, 50.0));
        j.outcome = Some(athena_data::records::Outcome {
            job: job_id,
            state,
            reason: "r".into(),
            at: jiff::Timestamp::UNIX_EPOCH,
            totals: None,
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        });
        j
    }

    fn records(jobs: Vec<JobRecords>) -> Records {
        let mut records = Records::default();
        for j in jobs {
            records.jobs.insert(j.id().as_str().to_string(), j);
        }
        records
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    const OLD_STOP: &str = "20261020-100000-aaaa";
    const OLD_LANDED: &str = "20261021-100000-bbbb";
    const NEW_STOP: &str = "20261026-100000-cccc";
    const NEW_LANDED: &str = "20261026-110000-dddd";

    fn fixture(resumed: bool) -> Records {
        let landed = OutcomeState::Landed;
        let failed = OutcomeState::Failed;
        records(vec![
            // Before: a cap stop of three rounds at 0.2 USD a round, then a fresh plan that landed.
            job(
                OLD_STOP,
                "add x",
                "2026-10-20T10:00:00Z",
                3,
                None,
                failed,
                0.1,
            ),
            job(
                OLD_LANDED,
                &format!("try again on {OLD_STOP}"),
                "2026-10-21T10:00:00Z",
                1,
                None,
                landed,
                0.2,
            ),
            // After: a cap stop, then a retry that resumed from it with one round of revision.
            job(
                NEW_STOP,
                "add y",
                "2026-10-26T10:00:00Z",
                3,
                None,
                failed,
                0.1,
            ),
            job(
                NEW_LANDED,
                &format!("try again on {NEW_STOP}"),
                "2026-10-26T11:00:00Z",
                if resumed { 2 } else { 3 },
                resumed.then_some(NEW_STOP),
                landed,
                if resumed { 0.05 } else { 0.1 },
            ),
            // Excluded: landed without planning, and planned without landing.
            job(
                "20261022-100000-eeee",
                "add z",
                "2026-10-22T10:00:00Z",
                0,
                None,
                landed,
                0.0,
            ),
            job(
                "20261022-110000-ffff",
                "add w",
                "2026-10-22T11:00:00Z",
                2,
                None,
                failed,
                9.0,
            ),
        ])
    }

    #[test]
    fn planning_cost_sums_each_chain_at_its_landed_job_and_splits_at_the_first_resumed_plan() {
        let cost = planning_cost(&fixture(true)).expect("landed planned jobs");
        assert_eq!(
            cost.first_resumed,
            Some((NEW_LANDED.to_string(), "2026-10-26T11:00:00Z".to_string()))
        );
        // Before: one landed planned job (the unplanned landed job and the unlanded one are left out). Its
        // chain is 3 + 1 rounds and 0.6 + 0.4 USD of planner and critic; its own are 1 round and 0.4 USD.
        let before = cost.before;
        assert_eq!(before.jobs, 1);
        assert!(close(before.mean_rounds, 4.0), "{before:?}");
        assert!(close(before.mean_cost_usd, 1.0), "{before:?}");
        assert!(close(before.own_mean_rounds, 1.0), "{before:?}");
        assert!(close(before.own_mean_cost_usd, 0.4), "{before:?}");
        // After: the resumed retry. Its chain is 3 + 2 rounds and 0.6 + 0.2 USD; its own are 2 and 0.2.
        let after = cost.after;
        assert_eq!(after.jobs, 1);
        assert!(close(after.mean_rounds, 5.0), "{after:?}");
        assert!(close(after.mean_cost_usd, 0.8), "{after:?}");
        assert!(close(after.own_mean_rounds, 2.0), "{after:?}");
        assert!(close(after.own_mean_cost_usd, 0.2), "{after:?}");
    }

    #[test]
    fn planning_cost_follows_a_three_job_chain_once_and_stops_at_a_cycle() {
        let landed = OutcomeState::Landed;
        let failed = OutcomeState::Failed;
        let a = "20261020-100000-aaaa";
        let b = "20261021-100000-bbbb";
        let c = "20261022-100000-cccc";
        let chain = records(vec![
            job(
                a,
                &format!("try again on {c}"),
                "2026-10-20T10:00:00Z",
                1,
                None,
                failed,
                0.1,
            ),
            job(
                b,
                &format!("try again on {a}"),
                "2026-10-21T10:00:00Z",
                2,
                None,
                failed,
                0.1,
            ),
            job(
                c,
                &format!("try again on {b}"),
                "2026-10-22T10:00:00Z",
                3,
                None,
                landed,
                0.1,
            ),
        ]);
        let cost = planning_cost(&chain).expect("a landed planned job");
        assert_eq!(
            cost.before.jobs, 1,
            "the chain counts once, at its landed job"
        );
        assert!(close(cost.before.mean_rounds, 6.0), "{cost:?}");
        assert!(close(cost.before.mean_cost_usd, 1.2), "{cost:?}");
    }

    #[test]
    fn planning_cost_is_absent_when_no_planned_job_landed() {
        let none = records(vec![job(
            OLD_STOP,
            "add x",
            "2026-10-20T10:00:00Z",
            3,
            None,
            OutcomeState::Failed,
            0.1,
        )]);
        assert_eq!(planning_cost(&none), None);
    }

    #[test]
    fn planning_cost_has_no_after_group_when_no_plan_was_resumed() {
        let cost = planning_cost(&fixture(false)).expect("landed planned jobs");
        assert_eq!(cost.first_resumed, None);
        assert_eq!(cost.after, PlanningGroup::default());
        assert_eq!(cost.before.jobs, 2);
        // Chains of 4 and 6 rounds, costing 1.0 and 1.2 USD.
        assert!(close(cost.before.mean_rounds, 5.0), "{cost:?}");
        assert!(close(cost.before.mean_cost_usd, 1.1), "{cost:?}");
    }

    fn critique(job: &str, round: u32, verdict: CritiqueVerdict) -> Critique {
        Critique {
            job: id(job),
            round,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: None,
            agent_verdict: Some(verdict),
            final_verdict: verdict,
            findings: vec![],
            request_reading_mismatch: None,
            summary: None,
        }
    }

    /// A planned, dispatched job costing `usd` in all, closed as `state`, whose critiques give `verdicts` in turn.
    fn planned(
        i: usize,
        state: OutcomeState,
        usd: f64,
        verdicts: &[CritiqueVerdict],
    ) -> JobRecords {
        let text = format!("20261001-00{:02}{:02}-abcd", i / 60, i % 60);
        let mut j = job(
            &text,
            &format!("request {i}"),
            "2026-10-01T10:00:00Z",
            1,
            None,
            state,
            0.0,
        );
        j.invocations = vec![invocation(&id(&text), 99, Role::Producer, usd)];
        for (n, v) in verdicts.iter().enumerate() {
            j.critiques.push(critique(&text, n as u32 + 1, *v));
        }
        j
    }

    /// `n` planned jobs of which `landed` landed (accepted in round 1); the rest were stopped in planning.
    fn fleet(n: usize, landed: usize, usd: f64) -> Records {
        records(
            (0..n)
                .map(|i| {
                    if i < landed {
                        planned(i, OutcomeState::Landed, usd, &[CritiqueVerdict::Accept])
                    } else {
                        planned(i, OutcomeState::Failed, usd, &[CritiqueVerdict::Reject])
                    }
                })
                .collect(),
        )
    }

    #[test]
    fn keep_rule_flags_28_landed_of_51_planned_as_a_breach_with_the_measures() {
        use CritiqueVerdict::{Accept, Reject, Revise};
        let jobs = (0..51)
            .map(|i| match i {
                // 20 landed after an accept in round 1, 8 after a revise first.
                0..20 => planned(i, OutcomeState::Landed, 1.0, &[Accept]),
                20..28 => planned(i, OutcomeState::Landed, 1.0, &[Revise, Accept]),
                // 23 not landed; stopped in planning at round 1.
                _ => planned(i, OutcomeState::Failed, 1.0, &[Reject]),
            })
            .collect();
        let rule = keep_rule(&records(jobs)).expect("planned jobs");
        assert_eq!((rule.planned, rule.landed), (51, 28));
        assert!(close(rule.share, 28.0 / 51.0), "{rule:?}");
        assert!(close(rule.total_cost_usd, 51.0), "{rule:?}");
        assert!(
            close(rule.cost_per_landed_usd.unwrap(), 51.0 / 28.0),
            "{rule:?}"
        );
        assert_eq!(
            (rule.first_round_accepted, rule.first_round_planned),
            (20, 51)
        );
        assert!(
            close(rule.first_round_accept_rate.unwrap(), 20.0 / 51.0),
            "{rule:?}"
        );
        assert_eq!(rule.verdict, KeepVerdict::Breach);

        let json = serde_json::to_value(&rule).unwrap();
        assert_eq!(json["planned"], 51);
        assert_eq!(json["landed"], 28);
        assert_eq!(json["baseline_share"], 0.64);
        assert_eq!(json["verdict"], "breach");
        assert!((json["share"].as_f64().unwrap() - 0.549).abs() < 1e-3);
        assert!(json["cost_per_landed_usd"].is_f64());
        assert!(json["first_round_accept_rate"].is_f64());
    }

    #[test]
    fn keep_rule_counts_a_job_stopped_in_planning_without_a_dispatch_as_planned_and_not_landed() {
        let mut stopped = planned(1, OutcomeState::Failed, 0.5, &[CritiqueVerdict::Reject]);
        stopped.job = None;
        stopped.outcome = None;
        let rule = keep_rule(&records(vec![
            planned(0, OutcomeState::Landed, 1.5, &[CritiqueVerdict::Accept]),
            stopped,
        ]))
        .unwrap();
        assert_eq!((rule.planned, rule.landed), (2, 1));
        // All cost of the planned jobs over the landed ones.
        assert!(close(rule.cost_per_landed_usd.unwrap(), 2.0), "{rule:?}");
    }

    #[test]
    fn keep_rule_early_branches_keep_at_the_baseline_and_breach_below_it() {
        // 30 planned: 20 of 30 is 0.667 (keep); 19 of 30 is 0.633 (breach).
        assert_eq!(
            keep_rule(&fleet(30, 20, 1.0)).unwrap().verdict,
            KeepVerdict::Keep
        );
        assert_eq!(
            keep_rule(&fleet(30, 19, 1.0)).unwrap().verdict,
            KeepVerdict::Breach
        );
        // Exactly the baseline: 16 of 25 is 0.64, kept.
        assert_eq!(
            keep_rule(&fleet(25, 16, 1.0)).unwrap().verdict,
            KeepVerdict::Keep
        );
        // Between the thresholds the early rule continues.
        assert_eq!(
            keep_rule(&fleet(60, 30, 1.0)).unwrap().verdict,
            KeepVerdict::Breach
        );
    }

    #[test]
    fn keep_rule_decision_branches_at_100_planned_jobs() {
        let verdict = |landed, usd| keep_rule(&fleet(100, landed, usd)).unwrap().verdict;
        // 72% landed at $1.39 a landed job ($100 in all): keep.
        assert_eq!(verdict(72, 1.0), KeepVerdict::Keep);
        // 72% landed but $2.78 a landed job ($200 in all): inconclusive.
        assert_eq!(verdict(72, 2.0), KeepVerdict::Inconclusive);
        // 64% landed: turn off. 65% landed: inconclusive.
        assert_eq!(verdict(64, 1.0), KeepVerdict::TurnOff);
        assert_eq!(verdict(65, 1.0), KeepVerdict::Inconclusive);
        assert_eq!(verdict(10, 1.0), KeepVerdict::TurnOff);
    }

    /// A dispatched job with no planning whose producer ran with `model` and `effort`, costing `usd`, with one
    /// verifier run that gave `verdict`.
    fn producer_job(
        text: &str,
        model: &str,
        effort: Option<&str>,
        state: OutcomeState,
        verdict: athena_data::records::Verdict,
        usd: f64,
    ) -> JobRecords {
        let mut j = job(text, "r", "2026-10-26T10:00:00Z", 0, None, state, usd);
        let job_id = id(text);
        j.invocations.clear();
        let mut producer = invocation(&job_id, 1, Role::Producer, usd);
        producer.model = model.into();
        producer.effort = effort.map(str::to_string);
        j.invocations.push(producer);
        j.invocations
            .push(invocation(&job_id, 2, Role::Verifier, 0.0));
        j.verification = Some(athena_data::records::Verification {
            job: job_id,
            candidate_commit: "0".repeat(40),
            verified_fingerprint: String::new(),
            tool_config_from_base: vec![],
            checks: vec![],
            check_cleanup: vec![],
            verifier_invocation: Some(2),
            criteria: vec![],
            findings: vec![],
            request_reading_mismatch: None,
            agent_verdict: Some(verdict),
            final_verdict: verdict,
            reasons: vec![],
            inputs: vec![],
            summary: None,
        });
        j
    }

    #[test]
    fn producer_settings_split_first_pass_landed_share_and_cost_by_model_and_effort() {
        use athena_data::records::Verdict::{Fail, Pass};
        let landed = OutcomeState::Landed;
        let failed = OutcomeState::Failed;
        let stats = producer_settings(&records(vec![
            producer_job(
                "20261026-100000-aaaa",
                "s",
                Some("medium"),
                landed,
                Pass,
                1.0,
            ),
            producer_job(
                "20261026-100001-aaab",
                "s",
                Some("medium"),
                failed,
                Fail,
                3.0,
            ),
            producer_job("20261026-100002-aaac", "s", Some("high"), failed, Fail, 2.0),
            producer_job("20261026-100003-aaad", "s", None, landed, Pass, 0.5),
        ]));
        let get = |effort: &str| {
            stats
                .iter()
                .find(|s| s.model == "s" && s.effort == effort)
                .unwrap_or_else(|| panic!("no bucket for {effort}: {stats:?}"))
        };
        assert_eq!(stats.len(), 3);
        let medium = get("medium");
        assert_eq!((medium.jobs, medium.verified, medium.first_pass), (2, 2, 1));
        assert_eq!(medium.first_pass_rate, Some(0.5));
        assert_eq!(medium.landed, 1);
        assert!(close(medium.landed_share, 0.5));
        assert!(close(medium.cost_per_landed_usd.unwrap(), 4.0));
        // A bucket with no landed job has a zero share and no cost per landed job, not a division by zero.
        let high = get("high");
        assert_eq!((high.jobs, high.landed), (1, 0));
        assert_eq!(high.landed_share, 0.0);
        assert_eq!(high.cost_per_landed_usd, None);
        assert_eq!(high.first_pass_rate, Some(0.0));
        // A producer invocation recorded before effort was kept gets its own bucket.
        assert_eq!(get(EFFORT_UNRECORDED).landed, 1);
    }

    fn test_run(seconds: f64, basis: &str) -> athena_data::records::CheckRun {
        athena_data::records::CheckRun {
            name: "test".into(),
            command: vec!["cargo".into(), "test".into()],
            exit_code: Some(0),
            passed: true,
            tests_ran: Some(10),
            duration: Quantity::measured(seconds, Unit::Seconds, basis),
            output_tail: String::new(),
            note: None,
        }
    }

    /// 24 request jobs, oldest first: the 2 oldest took 5000 s (outside the window of 20); job 12 has no test
    /// check; the newest has the 0 s placeholder of a check that did not run; job 15 took `slowest` s; the
    /// others took 100 s plus their number.
    fn seeded_durations(slowest: f64) -> Records {
        let jobs = (0..24)
            .map(|n: u32| {
                let text = format!("20261026-1000{n:02}-aa{n:02}");
                let mut j = producer_job(
                    &text,
                    "s",
                    None,
                    OutcomeState::Landed,
                    athena_data::records::Verdict::Pass,
                    0.0,
                );
                let checks = match n {
                    0 | 1 => vec![test_run(5000.0, "athena monotonic clock")],
                    12 => vec![],
                    15 => vec![test_run(slowest, "athena monotonic clock")],
                    23 => vec![test_run(0.0, "not run")],
                    _ => vec![test_run(100.0 + n as f64, "athena monotonic clock")],
                };
                j.verification.as_mut().unwrap().checks = checks;
                j
            })
            .collect();
        records(jobs)
    }

    #[test]
    fn check_durations_report_median_slowest_and_the_eighty_percent_flag() {
        let limit = 1800;
        // The 20 counted jobs are numbers 2 to 22 but 12: durations 102 to 111, 113 to 122 with job 15 slow.
        // The median is the mean of 111 and 113. The placeholder, the missing check and the two old jobs are out.
        let over = check_durations(&seeded_durations(1500.0), limit).unwrap();
        assert_eq!(over.jobs, 20);
        assert!(close(over.median_s, 112.0), "{over:?}");
        assert!(close(over.slowest_s, 1500.0));
        assert_eq!(over.slowest_job, "20261026-100015-aa15");
        assert_eq!(over.limit_s, limit);
        assert!(over.flagged, "1500 s is over 80% of 1800 s");
        let line = over.text();
        assert!(line.contains("median 112s"), "{line}");
        assert!(
            line.contains("slowest 1500s (20261026-100015-aa15)"),
            "{line}"
        );
        assert!(line.contains("of 1800s limit over last 20 jobs"), "{line}");
        assert!(line.contains("over 80% of the limit"), "{line}");

        // Under the threshold: reported, not flagged.
        let under = check_durations(&seeded_durations(1000.0), limit).unwrap();
        assert!(!under.flagged);
        assert!(close(under.slowest_s, 1000.0));
        assert!(!under.text().contains("80%"), "{}", under.text());

        // Exactly 80% of the limit is not over it.
        let edge = check_durations(&seeded_durations(1440.0), limit).unwrap();
        assert!(!edge.flagged);
    }

    #[test]
    fn check_durations_need_a_measured_test_run() {
        assert_eq!(check_durations(&Records::default(), 1800), None);
        let mut only_placeholder = producer_job(
            "20261026-100000-aaaa",
            "s",
            None,
            OutcomeState::Landed,
            athena_data::records::Verdict::Pass,
            0.0,
        );
        only_placeholder.verification.as_mut().unwrap().checks = vec![test_run(0.0, "not run")];
        assert_eq!(
            check_durations(&records(vec![only_placeholder]), 1800),
            None
        );
    }

    #[test]
    fn producer_settings_are_empty_without_a_dispatched_producer() {
        assert!(producer_settings(&Records::default()).is_empty());
        let mut queued = job(
            "20261026-100000-aaaa",
            "r",
            "2026-10-26T10:00:00Z",
            0,
            None,
            OutcomeState::Landed,
            1.0,
        );
        queued.invocations.clear();
        assert!(producer_settings(&records(vec![queued])).is_empty());
    }

    #[test]
    fn keep_rule_with_nothing_landed_divides_by_nothing() {
        let rule = keep_rule(&fleet(5, 0, 1.0)).unwrap();
        assert_eq!(rule.landed, 0);
        assert_eq!(rule.share, 0.0);
        assert_eq!(rule.cost_per_landed_usd, None);
        assert_eq!(rule.verdict, KeepVerdict::Breach);
        assert!(serde_json::to_value(&rule).unwrap()["cost_per_landed_usd"].is_null());
        assert_eq!(
            keep_rule(&fleet(100, 0, 1.0)).unwrap().verdict,
            KeepVerdict::TurnOff
        );
        assert_eq!(keep_rule(&Records::default()), None);
    }

    /// A later introspection about `job`, whose cause is `cause`.
    fn failure_cause(
        record: &str,
        job: &str,
        cause: &str,
    ) -> athena_data::records::IntrospectionRecord {
        use athena_data::records::{
            IntrospectionRecord, IntrospectionStatus, IntrospectionTrigger, PreventionKind,
        };
        IntrospectionRecord {
            id: athena_data::RecordId::parse(record).unwrap(),
            at: jiff::Timestamp::UNIX_EPOCH,
            trigger: IntrospectionTrigger::Failure,
            job: Some(id(job)),
            intervention: None,
            class_of_work: "change".into(),
            category: "failed".into(),
            what_happened: Some("a drain stopped".into()),
            failure_class: None,
            new_class: false,
            earlier_cases: Vec::new(),
            decisions_matched: Vec::new(),
            cause: Some(cause.into()),
            prevention_kind: Some(PreventionKind::CodeCheck),
            prevention: Some("p".into()),
            proposed_request: Some("r".into()),
            evidence: None,
            status: IntrospectionStatus::Recorded,
            failure: None,
            flags: Vec::new(),
            dropped_ids: Vec::new(),
            planning_stopped: false,
            from_jev: false,
            jev: None,
        }
    }

    #[test]
    fn later_harm_counts_landings_cited_as_a_failure_revert_or_fix_for_planned_and_unplanned_jobs()
    {
        const P_FAILED: &str = "20261020-100000-a1a1";
        const P_CLEAN: &str = "20261020-110000-b2b2";
        const U_FAILED: &str = "20261021-100000-c3c3";
        const U_REVERTED: &str = "20261021-110000-d4d4";
        const U_FIXED: &str = "20261021-120000-e5e5";
        const U_CLEAN: &str = "20261021-130000-f6f6";
        const U_NOT_LANDED: &str = "20261021-140000-g7g7";
        let landed = OutcomeState::Landed;
        // A dispatched job with no plan record, whose request is `request`.
        let unplanned = |text: &str, request: &str, state| {
            job(text, request, "2026-10-21T10:00:00Z", 0, None, state, 0.0)
        };
        let planned_job = |text: &str| {
            let mut j = job(text, "tidy", "2026-10-20T10:00:00Z", 1, None, landed, 0.0);
            j.critiques.push(critique(text, 1, CritiqueVerdict::Accept));
            j
        };
        let mut records = records(vec![
            planned_job(P_FAILED),
            planned_job(P_CLEAN),
            unplanned(U_FAILED, "tidy", landed),
            unplanned(U_REVERTED, "tidy", landed),
            unplanned(U_FIXED, "tidy", landed),
            unplanned(U_CLEAN, "tidy", landed),
            unplanned(U_NOT_LANDED, "tidy", OutcomeState::Failed),
            // Later jobs: one asks to revert a landing, one to fix another. A retry of a job that did not land,
            // and a request that cites a landing without asking for either, are no cause.
            unplanned(
                "20261022-100000-h8h8",
                &format!("Revert job {U_REVERTED}: it blocks the drain."),
                OutcomeState::Failed,
            ),
            unplanned(
                "20261022-110000-j9j9",
                &format!("Fix the rule that job {U_FIXED} landed."),
                OutcomeState::Failed,
            ),
            unplanned(
                "20261022-120000-k1k1",
                &format!("try again on {U_NOT_LANDED}; background: {U_CLEAN}"),
                OutcomeState::Failed,
            ),
        ]);
        records.introspections = vec![
            // A later failure names a planned landing and an unplanned one as its cause.
            failure_cause(
                "20261022-130000-m2m2",
                "20261022-100000-h8h8",
                &format!("The refusal rule landed by {P_FAILED} matched words and held the drain."),
            ),
            failure_cause(
                "20261022-140000-n3n3",
                "20261022-110000-j9j9",
                &format!("{U_FAILED} changed the gate."),
            ),
            // An introspection of the landing's own job, and one older than the landing, are no later cause.
            failure_cause(
                "20261022-150000-p4p4",
                P_CLEAN,
                &format!("{P_CLEAN} itself."),
            ),
            failure_cause(
                "20261019-100000-q5q5",
                "20261022-100000-h8h8",
                &format!("{U_CLEAN} earlier."),
            ),
        ];

        let harm = later_harm(&records);
        let p = harm.planned;
        assert_eq!((p.jobs, p.landed), (2, 2), "{harm:?}");
        assert_eq!(
            (p.cited, p.failures, p.reverts, p.fixes),
            (1, 1, 0, 0),
            "{harm:?}"
        );
        assert_eq!(p.share, Some(1.0));
        let u = harm.unplanned;
        // Eight unplanned dispatched jobs: four landings, the job that did not land, and the three later jobs.
        assert_eq!((u.jobs, u.landed), (8, 4), "{harm:?}");
        assert_eq!(
            (u.cited, u.failures, u.reverts, u.fixes),
            (3, 1, 1, 1),
            "{harm:?}"
        );
        assert!(close(u.share.unwrap(), 4.0 / 8.0), "{harm:?}");

        // It rides on the keep rule, beside the landed share, and in its JSON.
        let rule = keep_rule(&records).expect("planned jobs");
        assert_eq!(rule.later_harm, harm);
        let json = serde_json::to_value(&rule).unwrap();
        assert_eq!(json["later_harm"]["planned"]["cited"], 1);
        assert_eq!(json["later_harm"]["unplanned"]["reverts"], 1);
        assert_eq!(json["later_harm"]["unplanned"]["fixes"], 1);
        assert_eq!(json["later_harm"]["unplanned"]["failures"], 1);
    }

    /// The records of one request job whose stages took the minutes in `m`, starting at `start`: the reader,
    /// the planner (when `m.planning` is above zero), the producer, the checks (a verification with one
    /// `test` check, when `m.checks` is above zero), the verifier, then `m.landing_and_rebuild` minutes before
    /// the outcome `state`. A stage of zero minutes is left out, so a job can fail early.
    pub(crate) fn timed_job(
        text: &str,
        start: jiff::Timestamp,
        m: StageMinutes,
        state: OutcomeState,
    ) -> Vec<AnyRecord> {
        use athena_data::records::{Outcome, Request, Verdict, Verification};
        let job_id = id(text);
        let after = |t: jiff::Timestamp, minutes: f64| {
            t.checked_add(jiff::SignedDuration::from_secs_f64(minutes * 60.0))
                .unwrap()
        };
        let mut records = vec![AnyRecord::Request(Request {
            job: job_id.clone(),
            text: "A request.".into(),
            received_at: start,
            channel: "cli".into(),
        })];
        let mut cursor = start;
        let mut seq = 0;
        let mut invocations: Vec<Invocation> = Vec::new();
        let mut run = |role: Role, minutes: f64, cursor: &mut jiff::Timestamp| {
            if minutes > 0.0 {
                seq += 1;
                let mut i = invocation(&job_id, seq, role, 0.0);
                i.started_at = *cursor;
                i.ended_at = after(*cursor, minutes);
                *cursor = i.ended_at;
                Some(i)
            } else {
                None
            }
        };
        invocations.extend(run(Role::Reader, m.reading, &mut cursor));
        invocations.extend(run(Role::Planner, m.planning, &mut cursor));
        invocations.extend(run(Role::Producer, m.producer, &mut cursor));
        if m.checks > 0.0 {
            // The checks run between the producer and the verifier, outside any invocation.
            let mut check = test_run(m.checks * 60.0, "athena monotonic clock");
            check.name = "test".into();
            cursor = after(cursor, m.checks);
            records.push(AnyRecord::Verification(Box::new(Verification {
                job: job_id.clone(),
                candidate_commit: "0".repeat(40),
                verified_fingerprint: String::new(),
                tool_config_from_base: vec![],
                checks: vec![check],
                check_cleanup: vec![],
                verifier_invocation: None,
                criteria: vec![],
                findings: vec![],
                request_reading_mismatch: None,
                agent_verdict: Some(Verdict::Pass),
                final_verdict: Verdict::Pass,
                reasons: vec![],
                inputs: vec![],
                summary: None,
            })));
        }
        invocations.extend(run(Role::Verifier, m.verifier, &mut cursor));
        let totals = crate::scoring::totals(&invocations, None);
        records.extend(
            invocations
                .into_iter()
                .map(|i| AnyRecord::Invocation(Box::new(i))),
        );
        records.push(AnyRecord::Outcome(Box::new(Outcome {
            job: job_id,
            state,
            reason: "r".into(),
            at: after(cursor, m.landing_and_rebuild),
            totals: Some(totals),
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        })));
        records
    }

    /// The in-memory `Records` of records made by [`timed_job`].
    fn loaded(all: Vec<AnyRecord>) -> Records {
        let mut records = Records::default();
        for record in all {
            let job = match &record {
                AnyRecord::Request(r) => &r.job,
                AnyRecord::Invocation(r) => &r.job,
                AnyRecord::Verification(r) => &r.job,
                AnyRecord::Outcome(r) => &r.job,
                other => panic!("not a timed job record: {other:?}"),
            };
            let entry = records.jobs.entry(job.as_str().to_string()).or_default();
            entry.id.get_or_insert_with(|| job.clone());
            match record {
                AnyRecord::Request(r) => entry.request = Some(r),
                AnyRecord::Invocation(r) => entry.invocations.push(*r),
                AnyRecord::Verification(r) => entry.verification = Some(*r),
                AnyRecord::Outcome(r) => entry.outcome = Some(*r),
                _ => unreachable!(),
            }
        }
        records
    }

    fn minutes(
        reading: f64,
        planning: f64,
        producer: f64,
        checks: f64,
        verifier: f64,
        landing: f64,
    ) -> StageMinutes {
        StageMinutes {
            reading,
            planning,
            producer,
            checks,
            verifier,
            landing_and_rebuild: landing,
        }
    }

    #[test]
    fn throughput_reports_landed_per_day_stage_medians_idle_time_and_time_on_jobs_that_did_not_land()
     {
        let t = |s: &str| -> jiff::Timestamp { s.parse().unwrap() };
        let landed = OutcomeState::Landed;
        let mut all = Vec::new();
        // A lands on the 28th. B and C land on the 29th, and C ran while B ran.
        all.extend(timed_job(
            "20261028-230000-aaaa",
            t("2026-10-28T23:00:00Z"),
            minutes(2.0, 0.0, 10.0, 4.0, 6.0, 3.0),
            landed,
        ));
        all.extend(timed_job(
            "20261029-000000-aaab",
            t("2026-10-29T00:00:00Z"),
            minutes(4.0, 6.0, 20.0, 6.0, 8.0, 5.0),
            landed,
        ));
        all.extend(timed_job(
            "20261029-003000-aaac",
            t("2026-10-29T00:30:00Z"),
            minutes(3.0, 0.0, 15.0, 5.0, 5.0, 4.0),
            landed,
        ));
        // Two jobs that did not land: 30 minutes and 10 minutes of work.
        all.extend(timed_job(
            "20261029-030000-aaad",
            t("2026-10-29T03:00:00Z"),
            minutes(2.0, 0.0, 20.0, 0.0, 0.0, 8.0),
            OutcomeState::Failed,
        ));
        all.extend(timed_job(
            "20261029-040000-aaae",
            t("2026-10-29T04:00:00Z"),
            minutes(1.0, 0.0, 0.0, 0.0, 0.0, 9.0),
            OutcomeState::Rejected,
        ));
        let throughput = Throughput::compute(&loaded(all)).expect("jobs landed");

        assert_eq!(throughput.landed_total, 3);
        assert_eq!(
            throughput.landed_per_day,
            vec![
                DayCount {
                    day: "2026-10-28".into(),
                    landed: 1
                },
                DayCount {
                    day: "2026-10-29".into(),
                    landed: 2
                },
            ]
        );
        assert_eq!(throughput.window_jobs, 3);
        // Totals 25, 49 and 32 minutes: the median is 32, not the sum of the stage medians.
        assert_eq!(throughput.median_minutes, 32.0);
        assert_eq!(
            throughput.median_stages,
            minutes(3.0, 0.0, 15.0, 5.0, 6.0, 4.0)
        );
        // Gaps: A ends 23:25, B starts 00:00 (35); B and C overlap (0); C ends 01:02, D starts 03:00 (118);
        // D's invocations end 03:22 (the closing record after them is not time spent), the rejected job
        // starts 04:00 (38).
        assert_eq!(throughput.idle_minutes, 191.0);
        assert_eq!(throughput.unlanded_jobs, 2);
        assert_eq!(throughput.unlanded_minutes, 22.0 + 1.0);

        let text = throughput.text();
        for expected in [
            "3 landed (2026-10-28 1 · 2026-10-29 2)",
            "median per landed job over the last 3 measured: 32.0 min",
            "reading 3.0 · planning 0.0 · producer 15.0 · checks 5.0 · verifier 6.0 · landing and rebuild 4.0",
            "idle between jobs 191.0 min",
            "did not land: 2 job(s), 23.0 min",
        ] {
            assert!(text.contains(expected), "{expected:?} not in:\n{text}");
        }
    }

    fn t(s: &str) -> jiff::Timestamp {
        s.parse().unwrap()
    }

    /// A landing record, written at `created_at`, whose one commit is the job landing of `job`.
    fn landing_of(job: &str, created_at: jiff::Timestamp) -> athena_data::history::Landing {
        use athena_data::history::{Landing, LandingCommit, LandingKind, MergeMethod};
        Landing {
            id: athena_data::RecordId::generate(created_at),
            created_at,
            pr_number: 7,
            head_sha: "a".repeat(40),
            host_commit: "b".repeat(40),
            merge_method: MergeMethod::Squash,
            commits: vec![LandingCommit {
                sha: "a".repeat(40),
                tree: "c".repeat(40),
                kind: LandingKind::JobLanding,
                job: Some(id(job)),
                verification: None,
            }],
            intent: athena_data::RecordId::generate(created_at),
        }
    }

    const LANDED_JOB: &str = "20261029-100000-mnd1";

    /// One landed job: verifier ends 10:17, the outcome is set 0.1 minute later.
    fn landed_job_records() -> Records {
        loaded(timed_job(
            LANDED_JOB,
            t("2026-10-29T10:00:00Z"),
            minutes(0.0, 0.0, 10.0, 2.0, 5.0, 0.1),
            OutcomeState::Landed,
        ))
    }

    #[test]
    fn the_landing_stage_runs_to_the_landing_record_written_after_the_outcome() {
        let mut records = landed_job_records();
        // The landing record is written 6 minutes after the verifier ended: the CI wait, push and merge.
        records.landings = vec![landing_of(LANDED_JOB, t("2026-10-29T10:23:00Z"))];
        let landed = landed_jobs(&records);
        assert_eq!(landed.len(), 1);
        assert!(landed[0].landing_found);
        assert!((landed[0].stages.landing_and_rebuild - 6.0).abs() < 1e-6);
        let throughput = Throughput::compute(&records).unwrap();
        assert!((throughput.median_stages.landing_and_rebuild - 6.0).abs() < 1e-6);
        assert_eq!(throughput.window_with_landing_record, 1);
        // The wait for the landing record is time spent, so it is not idle time.
        assert_eq!(throughput.idle_minutes, 0.0);
    }

    /// Commits `records` and one landing record in `athena`'s repository, as a landed job leaves them.
    fn commit_records_and_landing(athena: &Athena, records: &[AnyRecord]) {
        for record in records {
            let file = crate::commits::NewFile::record(record);
            let path = athena.git.dir().join(&file.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, &file.bytes).unwrap();
        }
        let mut landing = landing_of(LANDED_JOB, t("2026-10-29T10:23:00Z"));
        // A job landing names the verification that covers it, or the record is not acceptable.
        landing.commits[0].verification =
            Some(athena_data::RecordId::generate(t("2026-10-29T10:17:00Z")));
        let dir = athena.git.dir().join("landings");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{}.json", landing.id)),
            serde_json::to_string_pretty(&athena_data::history::HistoryRecord::Landing(landing))
                .unwrap(),
        )
        .unwrap();
        athena.git.run(["add", "-A"]).unwrap();
        athena
            .git
            .run(["commit", "-q", "-m", "Records\n\nAthena-Job: bootstrap"])
            .unwrap();
    }

    #[test]
    fn status_snapshot_throughput_is_the_checkouts_throughput() {
        let (_dir, _state, athena) = crate::code_health::tests::set_up();
        let mut all = timed_job(
            LANDED_JOB,
            t("2026-10-29T10:00:00Z"),
            minutes(0.0, 0.0, 10.0, 2.0, 5.0, 0.1),
            OutcomeState::Landed,
        );
        all.extend(timed_job(
            "20261029-120000-mnd2",
            t("2026-10-29T12:00:00Z"),
            minutes(1.0, 0.0, 7.0, 0.0, 0.0, 3.0),
            OutcomeState::Failed,
        ));
        commit_records_and_landing(&athena, &all);
        let head = athena.head().unwrap();

        let out = tempfile::tempdir().unwrap();
        let snapshot = out.path().join("evidence");
        crate::snapshot::write(&athena.git, &head, &snapshot).unwrap();
        crate::snapshot::lock(&snapshot).unwrap();
        struct Unlock<'a>(&'a std::path::Path);
        impl Drop for Unlock<'_> {
            fn drop(&mut self) {
                let _ = athena_git::snapshot::unlock_dir(self.0);
            }
        }
        let _unlock = Unlock(&snapshot);

        let checkout = athena.status().unwrap();
        let from_snapshot = athena.status_from_snapshot(&snapshot).unwrap();
        let throughput = checkout.throughput.clone().expect("a job landed");
        assert_eq!(throughput.landed_total, 1);
        assert_eq!(
            throughput.window_with_landing_record, 1,
            "the landing record travels with the snapshot"
        );
        assert!(!throughput.text().is_empty());
        assert_eq!(from_snapshot.throughput.as_ref(), Some(&throughput));
        assert_eq!(
            from_snapshot.throughput.unwrap().text(),
            throughput.text(),
            "the rendered section is identical"
        );
        assert_eq!(from_snapshot.head, checkout.head);
        assert_eq!(from_snapshot.jobs_by_state, checkout.jobs_by_state);
        assert_eq!(
            from_snapshot.bootstrap.range_open,
            checkout.bootstrap.range_open
        );
        assert!(checkout.snapshot.is_none());
        let note = from_snapshot.snapshot.expect("a snapshot view says so");
        assert_eq!(note.commits, 2);
        assert!(note.omitted.iter().any(|s| s == "code health"));
        assert!(from_snapshot.code_health.is_none());
    }

    #[test]
    fn status_snapshot_without_history_is_refused_by_name() {
        let (_dir, _state, athena) = crate::code_health::tests::set_up();
        let out = tempfile::tempdir().unwrap();
        let error = athena
            .status_from_snapshot(out.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains("history.json"), "{error}");
    }

    #[test]
    fn a_landing_record_of_another_job_does_not_extend_the_landing_stage() {
        let mut records = landed_job_records();
        records.landings = vec![landing_of(
            "20261029-110000-pth1",
            t("2026-10-29T10:23:00Z"),
        )];
        let landed = landed_jobs(&records);
        assert!(!landed[0].landing_found);
        assert!((landed[0].stages.landing_and_rebuild - 0.1).abs() < 1e-6);
    }

    #[test]
    fn a_landed_job_with_no_landing_record_reports_the_outcome_time_and_says_so() {
        let records = landed_job_records();
        let landed = landed_jobs(&records);
        assert!(!landed[0].landing_found);
        assert!((landed[0].stages.landing_and_rebuild - 0.1).abs() < 1e-6);
        let text = Throughput::compute(&records).unwrap().text();
        assert!(
            text.contains(
                "no landing record (landings/) found, so landing ends at the landing outcome"
            ),
            "{text}"
        );
    }

    #[test]
    fn did_not_land_counts_the_time_spent_not_the_days_the_job_stayed_open() {
        let mut all = Vec::new();
        all.extend(timed_job(
            "20261029-100000-mnd1",
            t("2026-10-29T10:00:00Z"),
            minutes(0.0, 0.0, 10.0, 0.0, 0.0, 0.0),
            OutcomeState::Landed,
        ));
        // Runs 20 minutes and is closed three days later.
        all.extend(timed_job(
            "20261029-120000-pqn1",
            t("2026-10-29T12:00:00Z"),
            minutes(0.0, 0.0, 20.0, 0.0, 0.0, 3.0 * 24.0 * 60.0),
            OutcomeState::Abandoned,
        ));
        let throughput = Throughput::compute(&loaded(all)).unwrap();
        assert_eq!(throughput.unlanded_jobs, 1);
        assert_eq!(throughput.unlanded_minutes, 20.0);
    }

    #[test]
    fn idle_time_is_the_gap_between_time_spent_whatever_stays_open() {
        let mut all = Vec::new();
        all.extend(timed_job(
            "20261029-000000-jdk1",
            t("2026-10-29T00:00:00Z"),
            minutes(0.0, 0.0, 10.0, 0.0, 0.0, 0.0),
            OutcomeState::Landed,
        ));
        // Works 01:00 to 01:20, is closed three days later: its open days do not cover the next job.
        all.extend(timed_job(
            "20261029-010000-jdk2",
            t("2026-10-29T01:00:00Z"),
            minutes(0.0, 0.0, 20.0, 0.0, 0.0, 3.0 * 24.0 * 60.0),
            OutcomeState::Abandoned,
        ));
        all.extend(timed_job(
            "20261029-020000-jdk3",
            t("2026-10-29T02:00:00Z"),
            minutes(0.0, 0.0, 10.0, 0.0, 0.0, 0.0),
            OutcomeState::Landed,
        ));
        // Gaps: 00:10 to 01:00 (50) and 01:20 to 02:00 (40).
        let throughput = Throughput::compute(&loaded(all)).unwrap();
        assert_eq!(throughput.idle_minutes, 90.0);
    }

    #[test]
    fn every_printed_figure_states_its_basis_and_the_landing_basis_names_the_landing_record() {
        let mut records = landed_job_records();
        records.landings = vec![landing_of(LANDED_JOB, t("2026-10-29T10:23:00Z"))];
        let text = Throughput::compute(&records).unwrap().text();
        let lines: Vec<&str> = text.lines().collect();
        // Each figure line is followed by the line that states how it was measured.
        for (figure, basis) in [
            ("throughput 1 landed", "basis: landed jobs counted"),
            ("median per landed job", "basis: invocation wall times"),
            (
                "idle between jobs",
                "basis: gaps between merged busy intervals",
            ),
            ("did not land", "basis: time spent"),
        ] {
            let at = lines
                .iter()
                .position(|l| l.trim_start().starts_with(figure))
                .unwrap_or_else(|| panic!("{figure:?} not in:\n{text}"));
            assert!(
                lines[at + 1].contains(basis),
                "{figure:?} has no basis in:\n{text}"
            );
        }
        assert!(
            text.contains("landing record (landings/) found for each"),
            "{text}"
        );
        assert!(text.contains("placed by estimate"), "{text}");
        assert!(!text.contains("have no record"), "{text}");
    }

    #[test]
    fn throughput_is_absent_before_a_landing_and_its_medians_cover_only_the_newest_twenty() {
        assert_eq!(Throughput::compute(&Records::default()), None);
        let mut all = Vec::new();
        for n in 0..22u32 {
            // The two oldest jobs took 100 minutes; the newest twenty took 10.
            let producer = if n < 2 { 100.0 } else { 10.0 };
            all.extend(timed_job(
                &format!("20261029-1000{n:02}-ab{n:02}"),
                format!("2026-10-29T{:02}:00:00Z", n).parse().unwrap(),
                minutes(0.0, 0.0, producer, 0.0, 0.0, 0.0),
                OutcomeState::Landed,
            ));
        }
        let throughput = Throughput::compute(&loaded(all)).unwrap();
        assert_eq!(throughput.landed_total, 22);
        assert_eq!(throughput.window_jobs, THROUGHPUT_WINDOW);
        assert_eq!(throughput.median_minutes, 10.0);
    }
}

mod erased {
    /// Serializes any record to a JSON value for the trace.
    pub trait ToValue {
        fn to_value(&self) -> serde_json::Value;
    }
    impl<T: serde::Serialize> ToValue for T {
        fn to_value(&self) -> serde_json::Value {
            serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
        }
    }
}
