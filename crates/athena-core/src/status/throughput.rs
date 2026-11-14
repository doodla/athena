//! Throughput and wasted work: how many jobs landed, how long each stage took, how much work did not
//! land, and how long the checks ran, all derived from the invocation, check and landing records.

use super::*;

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

    /// The median of each stage over `window`, stage by stage. The throughput section and the throughput
    /// review's comparison with the previous review share this one definition, so the two compare like with
    /// like. All zero for an empty window.
    pub fn median_of(window: &[LandedJob]) -> StageMinutes {
        let column = |pick: fn(&StageMinutes) -> f64| -> f64 {
            median(&window.iter().map(|j| pick(&j.stages)).collect::<Vec<_>>())
        };
        StageMinutes {
            reading: column(|s| s.reading),
            planning: column(|s| s.planning),
            producer: column(|s| s.producer),
            checks: column(|s| s.checks),
            verifier: column(|s| s.verifier),
            landing_and_rebuild: column(|s| s.landing_and_rebuild),
        }
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
    /// Time checks waited for a build slot, over all request jobs; none when no check recorded a wait.
    pub build_wait: Option<BuildWait>,
    /// Wasted work over the newest [`THROUGHPUT_WINDOW`] landings (fewer when fewer have landed).
    pub waste: WasteWindow,
    /// The same figures over the [`THROUGHPUT_WINDOW`] landings before those; none until 20 more have landed.
    pub waste_previous: Option<WasteWindow>,
    /// The median minutes over the measured landed jobs before the newest window; none until 20 more exist.
    pub median_previous: Option<f64>,
    /// The noise between recent windows, and the smallest change one job can make, per measure.
    pub bars: Bars,
}

/// Wasted work over one window of landings: attempts per landed change, money spent on attempts that did not
/// land, and the share of landed changes that landed on their first attempt.
///
/// A landing is a request job that landed. An attempt that did not land is a request job that ran a model
/// and failed, was closed unlanded, or was stopped in planning ([`did_not_land`]); one that never ran is not
/// an attempt. A window holds the landings and the unlanded attempts that closed between the cut times of
/// its landings (by closing outcome, or by the end of the last invocation when there is none). A landing is a
/// first-attempt landing unless its request is "try again on <job>" and that job ran a model.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WasteWindow {
    /// Landings in the window; fewer than the window size only for the newest, partial window.
    pub landings: usize,
    /// Landings plus the attempts that did not land.
    pub attempts: usize,
    /// Attempts that ran a model and did not land.
    pub unlanded_attempts: usize,
    /// Landings that were not a retry of a job that ran a model.
    pub first_attempt_landings: usize,
    /// What the unlanded attempts cost, in US dollars (the lower bound of each invocation's best cost).
    pub wasted_usd: f64,
    /// What each unlanded attempt cost, for the one-job floor of the spend measure.
    #[serde(skip)]
    unlanded_costs: Vec<f64>,
}

impl WasteWindow {
    /// Attempts per landing; 0 for a window with no landing.
    pub fn attempts_per_landing(&self) -> f64 {
        if self.landings == 0 {
            return 0.0;
        }
        self.attempts as f64 / self.landings as f64
    }

    /// The share of landings that landed on the first attempt, from 0 to 1; 0 for a window with no landing.
    pub fn first_attempt_share(&self) -> f64 {
        if self.landings == 0 {
            return 0.0;
        }
        self.first_attempt_landings as f64 / self.landings as f64
    }

    /// The three figures, as a throughput review stores them.
    pub fn figures(&self) -> athena_data::records::WasteFigures {
        athena_data::records::WasteFigures {
            attempts_per_landing: self.attempts_per_landing().max(1.0),
            wasted_usd: self.wasted_usd,
            first_attempt_share: self.first_attempt_share(),
        }
    }
}

/// How large a change in one measure must be before it counts as more than the noise between recent windows.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Bar {
    /// The largest difference between two adjacent windows older than the newest one: `|w1 - w2|` and, when
    /// there is a fourth window, `|w2 - w3|`. None unless the newest three windows are full (60 landings).
    pub noise: Option<f64>,
    /// How many differences the noise is the largest of (1 or 2); 0 when there is no noise figure.
    pub differences: usize,
    /// The smallest change one job can make to the measure; a change at or below it is never counted.
    pub floor: f64,
}

impl Bar {
    /// The size a change must exceed, or None when the noise cannot be estimated.
    pub fn bar(&self) -> Option<f64> {
        self.noise.map(|n| n.max(self.floor))
    }
}

/// The [`Bar`] of each measure the throughput review watches.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Bars {
    pub median_minutes: Bar,
    pub attempts_per_landing: Bar,
    pub wasted_usd: Bar,
    pub first_attempt_share: Bar,
}

/// One measure in the newest window and in the window before it, with its bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Measure {
    pub name: &'static str,
    pub unit: &'static str,
    /// The figure over the newest window, and over the window before it (None until that window is full).
    pub now: f64,
    pub before: Option<f64>,
    /// Whether a rise is the bad direction (true for all but the first-attempt share).
    pub higher_is_worse: bool,
    pub bar: Bar,
}

impl Measure {
    /// How much worse `now` is than `before`; negative when it is better. None without a window before.
    pub fn worsening(&self) -> Option<f64> {
        let before = self.before?;
        Some(if self.higher_is_worse {
            self.now - before
        } else {
            before - self.now
        })
    }
}

/// The time checks waited for a build slot: the sum of `queued_s` over the checks of every request job's
/// `verification.json` that took a slot (a reused or skipped check has none). The basis is the athena
/// monotonic clock, which measures each wait.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BuildWait {
    /// Checks that took a slot and recorded how long they waited.
    pub checks: usize,
    /// The sum of their waits, in seconds.
    pub total_s: f64,
    /// The longest single wait, in seconds.
    pub longest_s: f64,
}

impl BuildWait {
    /// Sums `queued_s` over the checks of every request job's verification. None when no check has one.
    fn of(records: &Records) -> Option<BuildWait> {
        let waits: Vec<f64> = records
            .request_jobs()
            .into_iter()
            .filter_map(|j| j.verification.as_ref())
            .flat_map(|v| v.checks.iter())
            .filter_map(|c| c.queued_s)
            .filter(|s| s.is_finite() && *s >= 0.0)
            .collect();
        if waits.is_empty() {
            return None;
        }
        Some(BuildWait {
            checks: waits.len(),
            total_s: waits.iter().sum(),
            longest_s: waits.iter().copied().fold(0.0, f64::max),
        })
    }
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

/// When the landing record for `job` was written: the earliest `landings/` record with a job-landing commit
/// of the job. None when there is none. The earliest, because on a stacked branch later landings carry
/// earlier jobs' commits, and the newest would move the job's landing end later with every new landing.
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
        .min()
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

/// The measured landed jobs among the first `landings` request jobs to land, oldest first.
///
/// A throughput review stores `landings`, the count of every landed request job at that time
/// (`Throughput::landed_total`), but [`landed_jobs`] leaves out landed jobs that ran no model. Cutting
/// `landed_jobs` by that count would mix the two sets and drift forward by the number of unmeasured jobs. So
/// the cut is made over all landed request jobs, ordered as [`landed_jobs`] orders them (landing end, then id),
/// and only then are the unmeasured ones dropped. The result is a reconstruction of what that review saw: a
/// landing record written after the review can move a job's landing end and so its rank.
pub fn landed_jobs_among_first(records: &Records, landings: usize) -> Vec<LandedJob> {
    let first: BTreeSet<String> = ranked_landings(records)
        .into_iter()
        .take(landings)
        .map(|(_, id)| id)
        .collect();
    landed_jobs(records)
        .into_iter()
        .filter(|j| first.contains(&j.id))
        .collect()
}

/// Every landed request job as (landing end, id), oldest first: the order [`landed_jobs`] uses, over all
/// landed jobs and not only the measured ones.
fn ranked_landings(records: &Records) -> Vec<(jiff::Timestamp, String)> {
    let mut ranked: Vec<(jiff::Timestamp, String)> = records
        .request_jobs()
        .into_iter()
        .filter(|j| matches!(j.state(), JobState::Closed(OutcomeState::Landed)))
        .filter_map(|j| Some((landing_end(records, j)?.0, j.id().as_str().to_string())))
        .collect();
    ranked.sort();
    ranked
}

/// Whether `job` ran a model and did not land: it failed, was closed unlanded, or was stopped in planning. A
/// job that never ran a model is not an attempt, and a job still open is neither landed nor wasted yet.
fn did_not_land(job: &JobRecords) -> bool {
    if job.invocations.is_empty() {
        return false;
    }
    let state = job.state();
    let stopped = plan_state(job).is_some_and(|s| !s.accepted);
    let failed = matches!(state, JobState::Closed(s) if s != OutcomeState::Landed);
    failed || (stopped && state != JobState::Closed(OutcomeState::Landed))
}

/// Windows of wasted work, the newest first: the newest `window` landings (all of them when fewer have
/// landed) and up to three full windows of `window` landings before it. Each holds its landings and the
/// attempts that did not land ([`did_not_land`]) that closed from the end of the window's oldest landing up
/// to the end of the next newer window's oldest landing; an attempt that closed before the oldest reported
/// landing is in no window. An attempt is placed by its closing outcome, or by the end of its last invocation
/// when it has none. Empty when nothing has landed.
///
/// The figures are a reconstruction from job and invocation records: a landing record written later can move
/// a landing end and so a cut.
pub fn waste_windows(records: &Records, window: usize) -> Vec<WasteWindow> {
    let ranked = ranked_landings(records);
    let n = ranked.len();
    if n == 0 || window == 0 {
        return Vec::new();
    }
    // The (start, end) of each window in `ranked`.
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for k in 0..4 {
        let end = n.saturating_sub(k * window);
        if end == 0 || (k > 0 && end < window) {
            break;
        }
        spans.push((end.saturating_sub(window), end));
    }
    let attempts: Vec<((jiff::Timestamp, String), f64)> = records
        .request_jobs()
        .into_iter()
        .filter(|j| did_not_land(j))
        .filter_map(|j| {
            let at = closing_outcome(j)
                .map(|o| o.at)
                .or_else(|| j.invocations.iter().map(|i| i.ended_at).max())?;
            Some(((at, j.id().as_str().to_string()), j.spent_usd()))
        })
        .collect();
    let did_work = |id: &str| {
        records
            .jobs
            .get(id)
            .is_some_and(|j| !j.invocations.is_empty())
    };
    spans
        .iter()
        .enumerate()
        .map(|(k, &(start, end))| {
            let lower = &ranked[start];
            let upper = k.checked_sub(1).map(|newer| &ranked[spans[newer].0]);
            let costs: Vec<f64> = attempts
                .iter()
                .filter(|(key, _)| key >= lower && upper.is_none_or(|u| key < u))
                .map(|(_, cost)| *cost)
                .collect();
            let first_attempt_landings = ranked[start..end]
                .iter()
                .filter(|(_, id)| {
                    let retried = records
                        .jobs
                        .get(id.as_str())
                        .and_then(|j| j.request.as_ref())
                        .and_then(|r| crate::reading::retried_job(records, &r.text));
                    !retried.is_some_and(|r| r.as_str() != id && did_work(r.as_str()))
                })
                .count();
            WasteWindow {
                landings: end - start,
                attempts: end - start + costs.len(),
                unlanded_attempts: costs.len(),
                first_attempt_landings,
                wasted_usd: crate::store::positive_zero(costs.iter().sum()),
                unlanded_costs: costs,
            }
        })
        .collect()
}

/// The median of the total minutes in the newest `window` measured landed jobs and in up to three full
/// windows before it, newest first. `measured` is [`landed_jobs`], oldest first.
fn median_windows(measured: &[LandedJob], window: usize) -> Vec<f64> {
    let mut out = Vec::new();
    for k in 0..4 {
        let end = measured.len().saturating_sub(k * window);
        if end == 0 || (k > 0 && end < window) {
            break;
        }
        let jobs = &measured[end.saturating_sub(window)..end];
        out.push(median(
            &jobs.iter().map(|j| j.stages.total()).collect::<Vec<_>>(),
        ));
    }
    out
}

/// The bar of a measure from its windows, newest first: the noise is the largest difference between
/// adjacent windows older than the newest, which needs the newest three (so 60 landings).
fn bar_of(values: &[f64], floor: f64) -> Bar {
    if values.len() < 3 {
        return Bar {
            noise: None,
            differences: 0,
            floor,
        };
    }
    let differences: Vec<f64> = values[1..]
        .windows(2)
        .map(|p| (p[0] - p[1]).abs())
        .collect();
    Bar {
        noise: differences.iter().copied().reduce(f64::max),
        differences: differences.len(),
        floor,
    }
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
        let median_stages = StageMinutes::median_of(window);
        let median_minutes = median(&window.iter().map(|j| j.stages.total()).collect::<Vec<_>>());

        let mut spans: Vec<(jiff::Timestamp, jiff::Timestamp)> = Vec::new();
        let mut unlanded_jobs = 0;
        let mut unlanded_minutes = 0.0;
        for job in records.request_jobs() {
            if job.invocations.is_empty() {
                continue;
            }
            spans.extend(busy_intervals(records, job));
            if did_not_land(job) {
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
        let windows = waste_windows(records, THROUGHPUT_WINDOW);
        let waste = windows.first().cloned()?;
        let medians = median_windows(&measured, THROUGHPUT_WINDOW);
        let one_job = 1.0 / THROUGHPUT_WINDOW as f64;
        let costs: Vec<f64> = windows
            .iter()
            .filter(|w| w.landings == THROUGHPUT_WINDOW)
            .flat_map(|w| w.unlanded_costs.iter().copied())
            .collect();
        let series =
            |pick: fn(&WasteWindow) -> f64| -> Vec<f64> { windows.iter().map(pick).collect() };
        let bars = Bars {
            median_minutes: bar_of(&medians, 1.0),
            attempts_per_landing: bar_of(&series(WasteWindow::attempts_per_landing), one_job),
            wasted_usd: bar_of(&series(|w| w.wasted_usd), median(&costs)),
            first_attempt_share: bar_of(&series(WasteWindow::first_attempt_share), one_job),
        };
        Some(Throughput {
            waste,
            waste_previous: windows.get(1).cloned(),
            median_previous: medians.get(1).copied(),
            bars,
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
            build_wait: BuildWait::of(records),
        })
    }

    /// The four measures the throughput review watches, each in the newest window and in the one before it,
    /// with its bar: median minutes per landed job, attempts per landed change, money spent on attempts that
    /// did not land, and the share of changes that landed on the first attempt.
    pub fn measures(&self) -> Vec<Measure> {
        let before = self.waste_previous.as_ref();
        vec![
            Measure {
                name: "median minutes per landed job",
                unit: "min",
                now: self.median_minutes,
                before: self.median_previous,
                higher_is_worse: true,
                bar: self.bars.median_minutes,
            },
            Measure {
                name: "attempts per landed change",
                unit: "",
                now: self.waste.attempts_per_landing(),
                before: before.map(WasteWindow::attempts_per_landing),
                higher_is_worse: true,
                bar: self.bars.attempts_per_landing,
            },
            Measure {
                name: "spent on attempts that did not land",
                unit: "USD",
                now: self.waste.wasted_usd,
                before: before.map(|w| w.wasted_usd),
                higher_is_worse: true,
                bar: self.bars.wasted_usd,
            },
            Measure {
                name: "share landed on the first attempt",
                unit: "",
                now: self.waste.first_attempt_share(),
                before: before.map(WasteWindow::first_attempt_share),
                higher_is_worse: false,
                bar: self.bars.first_attempt_share,
            },
        ]
    }

    /// The lines `athena status` prints, and the section the throughput review is given.
    pub fn text(&self) -> String {
        let waste = &self.waste;
        let landings = waste.landings;
        let earlier = |show: &dyn Fn(&WasteWindow) -> String| {
            self.waste_previous.as_ref().map_or_else(String::new, |p| {
                format!("; the {} landings before: {}", p.landings, show(p))
            })
        };
        let waste_lines = format!(
            "\n           attempts per landed change: {:.2} over the last {landings} landing(s){}\n           basis: landings plus attempts that ran a model and did not land (failed, closed unlanded or stopped), placed by when they closed; jobs still open and requests that never ran are not counted\n           spent on attempts that did not land: {:.2} USD over the last {landings} landing(s){}\n           basis: the cost recorded for the invocations of those attempts, lower bound\n           landed on the first attempt: {:.0}% over the last {landings} landing(s){}\n           basis: landed changes whose request is not 'try again on' a job that ran a model, out of all landed changes",
            waste.attempts_per_landing(),
            earlier(&|w| format!("{:.2}", w.attempts_per_landing())),
            waste.wasted_usd,
            earlier(&|w| format!("{:.2} USD", w.wasted_usd)),
            waste.first_attempt_share() * 100.0,
            earlier(&|w| format!("{:.0}%", w.first_attempt_share() * 100.0)),
        );
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
        let build_wait = self.build_wait.map_or_else(String::new, |w| {
            format!(
                "\n           build wait {:.1} s over {} check(s), longest {:.1} s\n           basis: queued_s of each check in verification.json, athena monotonic clock; a check that took no build slot is not counted",
                w.total_s, w.checks, w.longest_s
            )
        });
        format!(
            "throughput {} landed ({days})\n           basis: landed jobs counted by their landing outcome's UTC day\n           median per landed job over the last {} measured: {:.1} min ({stages})\n           basis: invocation wall times, measured check durations, and landing and rebuild from the verifier's end to the later of the outcome and the landing record; {landing_basis}\n           idle between jobs {:.1} min\n           basis: gaps between merged busy intervals of all jobs (invocations, checks placed by estimate before the verifier, landing wait), not the time a job stayed open\n           did not land: {} job(s), {:.1} min\n           basis: time spent (invocation wall times plus measured check durations), not the time from dispatch to close{waste_lines}{build_wait}",
            self.landed_total,
            self.window_jobs,
            self.median_minutes,
            self.idle_minutes,
            self.unlanded_jobs,
            self.unlanded_minutes
        )
    }
}
