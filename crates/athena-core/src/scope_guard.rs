//! Scope guard: line-of-work grouping and thrash signals T1 to T5 (build job 1), and the running-line watcher
//! (build job 5).
//!
//! This is the design in `docs/design/scope-guard.md` (section 5 for the signals, section 13 for the jobs,
//! decision D127). The signal functions are pure functions over plain input values: elapsed agent time, spend
//! and the forecast p90 are passed in by the caller.
//!
//! The watcher (job 5) is the caller. After each attempt of a line finishes, [`Athena::watch_line`] rebuilds
//! the [`RequestRecord`]s from the committed records, groups them with [`group_lines`], evaluates T1 to T5 with
//! [`evaluate_before_attempt`] and writes a mark under `<state root>/scope/`: `<first id>.flag.json` for a
//! flag, `<first id>.pause.json` for a pause, named by the first request id of the line. The p90 passed to
//! [`t3_cost`] is `forecasts.reader.cost_usd.p90()` of the latest reading of the newest request of the line
//! that has a finished attempt.
//! `Athena::go` (dispatch) and `Athena::retry` refuse a paused line ([`Athena::line_pause`]). Nothing here
//! stops or signals a running process, and nothing lifts a pause: the owner deletes the pause file.
//!
//! Why an attempt stopped decides whether it counts. The reason comes from one classifier,
//! [`crate::disk::StopCause`], which also decides what `athena retry` may resume; this module keeps no list of
//! its own. Before that, the guard read the reason from a free-text prefix in its own list while the retry
//! path held another, and a producer that died on an expired login counted as spend on the line. The stored
//! pause is also not the decision: [`Athena::line_pause`] recomputes the signals from the line's attempts under
//! the current rule and returns the stored pause only while they still reach a pause.
//!
//! Every threshold is an **estimate, to be tuned** (section 5); the basis for each is in that section's table.

use crate::introspect::normalise;
use crate::reading::retried_job;
use crate::state::StateDir;
use crate::store::{JobRecords, JobState, Records};
use crate::{Athena, Ui};
use anyhow::{Context, Result};
use athena_data::JobId;
use athena_data::records::{OutcomeState, Role};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// T1 (section 5): flag when this attempt number on one line is about to be dispatched. Estimate; basis: the
/// trailer job (3 rejections) and cut A (4 tries).
pub const T1_ATTEMPT: u32 = 3;
/// T2 (section 5): flag when the latest plan lists more than this multiple of the first plan's size. Estimate.
pub const T2_RATIO: f64 = 1.5;
/// T2 (section 5): flag at this planning round. Estimate.
pub const T2_PLANNING_ROUND: u32 = 3;
/// T3 (section 5): flag when spend reaches this multiple of the forecast p90 for the job kind. Estimate.
pub const T3_FLAG_RATIO: f64 = 1.0;
/// T3 (section 5): pause when spend reaches this multiple of the forecast p90. Estimate.
pub const T3_PAUSE_RATIO: f64 = 1.5;
/// T3 (section 5): flag when spend on one line is over this many dollars, whatever the forecast. Estimate;
/// related to the $2.50 per landed job ceiling in `job-planning.md` section 9.
pub const T3_LINE_DOLLARS: f64 = 5.0;
/// T4 (section 5): number of rejections for the same criterion that pauses the line. Basis: the speedup case.
pub const T4_REJECTIONS: usize = 2;
/// T5 (section 5): flag at this many minutes of agent time with no accepted candidate. Basis: prompt-budget trap.
pub const T5_FLAG_MINUTES: u32 = 60;
/// T5 (section 5): pause at this many minutes of agent time with no accepted candidate. Estimate.
pub const T5_PAUSE_MINUTES: u32 = 120;
/// Share of the reading's `cost_usd` forecast that the producer is expected to use. Estimate, not measured: the
/// forecast covers the producer and the verifier only (checks cost no agent dollars, and planning is outside it).
pub const PRODUCER_SHARE: f64 = 0.7;
/// Share of the reading's `cost_usd` forecast that the verifier is expected to use. Estimate, not measured; it is
/// what is left to spend when a retry resumes a kept candidate, so T3 compares such a retry with this share of
/// the forecast p90, not the whole of it. Together with [`PRODUCER_SHARE`] it makes 1.
pub const VERIFIER_SHARE: f64 = 0.3;

/// How a finished attempt ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The candidate was accepted: the line is done.
    Accepted,
    /// The verifier rejected the candidate.
    Rejected,
    /// The attempt ended without a verdict (for example a failed run).
    Failed,
    /// The attempt stopped for an environment cause ([`crate::disk::StopCause`]: low disk, a check timeout, an
    /// expired producer login or a usage limit): the work was not judged and not found wrong. T1, T2 and T5 do not count it, T4 sees only
    /// rejections, and T3 drops its spend when the work was kept (see [`t3_basis`]).
    Environment,
}

/// One finished attempt, as the watcher reads it from job records.
#[derive(Debug, Clone, PartialEq)]
pub struct AttemptRecord {
    /// How the attempt ended.
    pub verdict: Verdict,
    /// The verifier's recorded unmet acceptance criterion text, when rejected and recorded.
    pub unmet_criterion: Option<String>,
    /// Spend on this attempt in dollars.
    pub cost_dollars: f64,
    /// Agent time spent on this attempt, in minutes.
    pub agent_minutes: u32,
    /// Files plus steps in the accepted plan of this attempt, when known.
    pub plan_size: Option<u32>,
    /// Which planning round produced the plan (1 for the first), when known.
    pub planning_round: u32,
    /// True when this attempt resumed a kept candidate: its retry target stopped for an environment cause or was
    /// interrupted ([`crate::disk::resumable_stop`]) and this job ran no producer. Planning, build and checks were
    /// then already done, so only the verifier was left to pay for.
    pub resumed: bool,
}

/// One request and its finished attempts.
#[derive(Debug, Clone, PartialEq)]
pub struct RequestRecord {
    /// Request id.
    pub id: String,
    /// The earlier job this one retries ("try again on <job>"), if any: the previous job of the line.
    pub retry_of: Option<String>,
    /// The nearest earlier request with the same request text that had not been accepted: this one is a
    /// re-ask of it.
    pub same_text_as: Option<String>,
    /// Finished attempts of this request, oldest first.
    pub attempts: Vec<AttemptRecord>,
}

/// Group requests into lines of work (section 5): a request, its retries ("try again on <job>") and any re-ask
/// of the same request text. A job id that only appears in a request's text does not join lines. The count of
/// attempts is therefore per line, not per job id. Links to ids not in `requests` are ignored. Returns indices into `requests`; each line is in input order and lines are
/// ordered by their first member.
pub fn group_lines(requests: &[RequestRecord]) -> Vec<Vec<usize>> {
    let mut parent: Vec<usize> = (0..requests.len()).collect();
    fn find(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let index: HashMap<&str, usize> = requests
        .iter()
        .enumerate()
        .map(|(i, r)| (r.id.as_str(), i))
        .collect();
    for (i, r) in requests.iter().enumerate() {
        let links = r.retry_of.iter().chain(r.same_text_as.iter());
        for target in links {
            if let Some(&j) = index.get(target.as_str()) {
                let (a, b) = (find(&mut parent, i), find(&mut parent, j));
                if a != b {
                    parent[a.max(b)] = a.min(b);
                }
            }
        }
    }
    let mut lines: Vec<Vec<usize>> = Vec::new();
    let mut slot: HashMap<usize, usize> = HashMap::new();
    for i in 0..requests.len() {
        let root = find(&mut parent, i);
        let s = *slot.entry(root).or_insert_with(|| {
            lines.push(Vec::new());
            lines.len() - 1
        });
        lines[s].push(i);
    }
    lines
}

/// All finished attempts of one line, in request order then attempt order.
pub fn line_attempts(requests: &[RequestRecord], line: &[usize]) -> Vec<AttemptRecord> {
    line.iter()
        .flat_map(|&i| requests[i].attempts.iter().cloned())
        .collect()
}

/// The effect of a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// The signal does not hold.
    None,
    /// Show it in the steward report; keep dispatching.
    Flag,
    /// Stop new attempts; finished work and records are left alone.
    Pause,
}

/// Result of the T4 criterion comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum T4Match {
    /// The criteria are clearly different.
    No,
    /// The match is unclear: flag, never pause (section 5).
    Maybe,
    /// The criteria are the same after normalising.
    Yes,
}

/// Normalise criterion text for T4 (section 5): lower case, ASCII digits removed, whitespace collapsed.
pub fn normalise_criterion(text: &str) -> String {
    let lowered: String = text
        .to_lowercase()
        .chars()
        .filter(|c| !c.is_ascii_digit())
        .collect();
    lowered.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Compare two unmet criteria. Equal after normalising is `Yes`; one containing the other, or more than half of
/// the shorter one's words shared, is `Maybe`; an empty criterion is `Maybe`; anything else is `No`.
pub fn compare_criteria(a: &str, b: &str) -> T4Match {
    let (a, b) = (normalise_criterion(a), normalise_criterion(b));
    if a.is_empty() || b.is_empty() {
        return T4Match::Maybe;
    }
    if a == b {
        return T4Match::Yes;
    }
    if a.contains(&b) || b.contains(&a) {
        return T4Match::Maybe;
    }
    let wa: Vec<&str> = a.split(' ').collect();
    let wb: Vec<&str> = b.split(' ').collect();
    let shared = wa.iter().filter(|w| wb.contains(w)).count();
    if shared * 2 > wa.len().min(wb.len()) {
        T4Match::Maybe
    } else {
        T4Match::No
    }
}

fn done(attempts: &[AttemptRecord]) -> bool {
    attempts.iter().any(|a| a.verdict == Verdict::Accepted)
}

/// The attempts that count toward the attempt, round and time signals: every attempt except one that ended on
/// an environment cause. A failed verification (`Rejected`) and a failure of the work (`Failed`) count.
fn counted(attempts: &[AttemptRecord]) -> Vec<&AttemptRecord> {
    attempts
        .iter()
        .filter(|a| a.verdict != Verdict::Environment)
        .collect()
}

/// T1, repeated retries: flag when the next attempt would be the `T1_ATTEMPT`th on the line. An attempt that
/// ended on an environment cause is not an attempt of the work and is not counted.
pub fn t1_retries(attempts: &[AttemptRecord]) -> Level {
    if !done(attempts) && counted(attempts).len() as u32 + 1 >= T1_ATTEMPT {
        Level::Flag
    } else {
        Level::None
    }
}

/// T2, growing plan: flag when the latest known plan is over `T2_RATIO` times the first, or a planning round
/// reached `T2_PLANNING_ROUND`. The "pause if T1 or T4 also holds" rule is applied in [`evaluate_before_attempt`].
/// The plan size and planning round of an attempt that ended on an environment cause are not used.
pub fn t2_plan_growth(attempts: &[AttemptRecord]) -> Level {
    if done(attempts) {
        return Level::None;
    }
    let attempts = counted(attempts);
    let sizes: Vec<u32> = attempts.iter().filter_map(|a| a.plan_size).collect();
    let grew = match (sizes.first(), sizes.last()) {
        (Some(&first), Some(&last)) if sizes.len() > 1 && first > 0 => {
            f64::from(last) > T2_RATIO * f64::from(first)
        }
        _ => false,
    };
    let rounds = attempts
        .iter()
        .any(|a| a.planning_round >= T2_PLANNING_ROUND);
    if grew || rounds {
        Level::Flag
    } else {
        Level::None
    }
}

/// The spend and the forecast T3 compares, from the line's attempts and the reading's forecast p90.
///
/// Spend drops an attempt that ended on an environment cause when its work was kept: it is the last attempt
/// of the line, or the next attempt resumed it. Kept work is not waste. An environment-stopped attempt that a
/// later attempt redid from scratch stays in the spend, since that spend was lost.
///
/// When the last attempt resumed a kept candidate, only the verifier was left to run, so the forecast is
/// `forecast_p90_dollars` times [`VERIFIER_SHARE`] (an estimate); otherwise it is `forecast_p90_dollars`.
/// Known limit: an earlier rejection on the same line still counts in the spend of a resumed attempt.
pub fn t3_basis(attempts: &[AttemptRecord], forecast_p90_dollars: f64) -> (f64, f64) {
    let spend: f64 = attempts
        .iter()
        .enumerate()
        .filter(|(i, a)| {
            let kept = *i + 1 == attempts.len() || attempts[*i + 1].resumed;
            !(a.verdict == Verdict::Environment && kept)
        })
        .map(|(_, a)| a.cost_dollars)
        .sum();
    let resumed_last = attempts
        .last()
        .is_some_and(|a| a.resumed && a.verdict != Verdict::Environment);
    let forecast = if resumed_last {
        forecast_p90_dollars * VERIFIER_SHARE
    } else {
        forecast_p90_dollars
    };
    (spend, forecast)
}

/// T3, cost against forecast: pause at `T3_PAUSE_RATIO` times the forecast p90, flag at `T3_FLAG_RATIO`
/// times it, and flag over `T3_LINE_DOLLARS` regardless of forecast. A forecast of 0 or less is not used.
/// `forecast_p90_dollars` is the p90 for a whole job; spend and forecast are adjusted by [`t3_basis`] for
/// environment stops and for a retry that resumed a kept candidate.
pub fn t3_cost(attempts: &[AttemptRecord], forecast_p90_dollars: f64) -> Level {
    if done(attempts) {
        return Level::None;
    }
    let (spend, forecast_p90_dollars) = t3_basis(attempts, forecast_p90_dollars);
    if forecast_p90_dollars > 0.0 && spend >= T3_PAUSE_RATIO * forecast_p90_dollars {
        Level::Pause
    } else if (forecast_p90_dollars > 0.0 && spend >= T3_FLAG_RATIO * forecast_p90_dollars)
        || spend > T3_LINE_DOLLARS
    {
        Level::Flag
    } else {
        Level::None
    }
}

/// T4, the same criterion rejected twice: compares the unmet criteria of the last `T4_REJECTIONS` rejections.
pub fn t4_match(attempts: &[AttemptRecord]) -> T4Match {
    let rejected: Vec<&AttemptRecord> = attempts
        .iter()
        .filter(|a| a.verdict == Verdict::Rejected)
        .collect();
    if done(attempts) || rejected.len() < T4_REJECTIONS {
        return T4Match::No;
    }
    let last = &rejected[rejected.len() - T4_REJECTIONS..];
    let mut result = T4Match::Yes;
    for pair in last.windows(2) {
        let m = match (&pair[0].unmet_criterion, &pair[1].unmet_criterion) {
            (Some(x), Some(y)) => compare_criteria(x, y),
            _ => T4Match::Maybe,
        };
        result = match (result, m) {
            (T4Match::No, _) | (_, T4Match::No) => T4Match::No,
            (T4Match::Maybe, _) | (_, T4Match::Maybe) => T4Match::Maybe,
            _ => T4Match::Yes,
        };
    }
    result
}

/// T4 as a level: `Yes` pauses, `Maybe` flags, `No` is none.
pub fn t4_level(attempts: &[AttemptRecord]) -> Level {
    match t4_match(attempts) {
        T4Match::Yes => Level::Pause,
        T4Match::Maybe => Level::Flag,
        T4Match::No => Level::None,
    }
}

/// T5, wall time: flag at `T5_FLAG_MINUTES` of agent time on the line with no accepted candidate, pause at
/// `T5_PAUSE_MINUTES`. The minutes of an attempt that ended on an environment cause are not counted.
pub fn t5_wall_time(attempts: &[AttemptRecord]) -> Level {
    if done(attempts) {
        return Level::None;
    }
    let minutes: u32 = counted(attempts).iter().map(|a| a.agent_minutes).sum();
    if minutes >= T5_PAUSE_MINUTES {
        Level::Pause
    } else if minutes >= T5_FLAG_MINUTES {
        Level::Flag
    } else {
        Level::None
    }
}

/// The five signals for one line at one point in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signals {
    /// Repeated retries.
    pub t1: Level,
    /// Growing plan.
    pub t2: Level,
    /// Cost against forecast.
    pub t3: Level,
    /// Same criterion rejected twice.
    pub t4: Level,
    /// Wall time.
    pub t5: Level,
}

impl Signals {
    /// The strongest effect of any signal.
    pub fn overall(&self) -> Level {
        [self.t1, self.t2, self.t3, self.t4, self.t5]
            .into_iter()
            .max()
            .unwrap_or(Level::None)
    }
}

/// Evaluate T1 to T5 before the next attempt of a line, from its finished attempts only. T2 is raised to a
/// pause when T1 holds or T4 is a clear match (section 5). `forecast_p90_dollars` is the forecast p90 for the
/// line's job kind, supplied by the caller.
pub fn evaluate_before_attempt(attempts: &[AttemptRecord], forecast_p90_dollars: f64) -> Signals {
    let t1 = t1_retries(attempts);
    let t4 = t4_level(attempts);
    let mut t2 = t2_plan_growth(attempts);
    if t2 == Level::Flag && (t1 != Level::None || t4 == Level::Pause) {
        t2 = Level::Pause;
    }
    Signals {
        t1,
        t2,
        t3: t3_cost(attempts, forecast_p90_dollars),
        t4,
        t5: t5_wall_time(attempts),
    }
}

/// One signal that holds, with the numbers that tripped it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalFact {
    /// `T1` to `T5`.
    pub signal: String,
    /// `flag` or `pause`.
    pub level: String,
    /// The numbers, in words.
    pub detail: String,
}

/// The numbers a mark was computed from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MarkNumbers {
    /// Finished attempts on the line.
    pub attempts: usize,
    /// Rejected attempts on the line.
    pub rejections: usize,
    /// Spend on the line in dollars.
    pub spend_usd: f64,
    /// The forecast p90 in dollars given to T3; 0 when the reading has none.
    pub forecast_p90_usd: f64,
    /// Agent minutes on the line.
    pub agent_minutes: u32,
    /// The unmet criterion of the latest rejection, when recorded.
    pub matched_criterion: Option<String>,
}

/// A flag or a pause on one line, as stored under `<state root>/scope/`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineMark {
    /// `flag` or `pause`.
    pub level: String,
    /// The first request id of the line: the key of the file.
    pub line: String,
    /// The job whose finish triggered the evaluation.
    pub job: String,
    /// When the mark was written.
    pub at: String,
    /// The signals that hold.
    pub signals: Vec<SignalFact>,
    /// The numbers behind them.
    pub numbers: MarkNumbers,
    /// Why, in one sentence.
    pub reason: String,
    /// What would lift it, and how.
    pub lift: String,
}

impl LineMark {
    /// The signal names, such as `T1, T4`.
    pub fn signal_names(&self) -> String {
        self.signals
            .iter()
            .map(|s| s.signal.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Whether this is a pause.
    pub fn is_pause(&self) -> bool {
        self.level == "pause"
    }
}

/// The directory of the marks of one repository.
pub fn scope_dir(state: &StateDir) -> PathBuf {
    state.root().join("scope")
}

/// The pause file of the line whose first request id is `first`.
pub fn pause_path(state: &StateDir, first: &str) -> PathBuf {
    scope_dir(state).join(format!("{first}.pause.json"))
}

/// The flag file of the line whose first request id is `first`.
pub fn flag_path(state: &StateDir, first: &str) -> PathBuf {
    scope_dir(state).join(format!("{first}.flag.json"))
}

fn read_mark(path: &std::path::Path) -> Option<LineMark> {
    serde_json::from_slice(&std::fs::read(path).ok()?).ok()
}

fn write_mark(path: PathBuf, mark: &LineMark) -> Result<()> {
    std::fs::create_dir_all(path.parent().expect("a mark file has a directory"))?;
    let text = serde_json::to_string_pretty(mark)?;
    std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))
}

/// Every mark in the state directory, pauses first. A flag is not listed for a line that is paused.
pub fn list_lines(state: &StateDir) -> Vec<LineMark> {
    let (pauses, flags) = stored_marks(state);
    merge_marks(pauses, flags)
}

/// Pauses first, then the flags of lines that are not paused, each sorted by line.
fn merge_marks(mut pauses: Vec<LineMark>, mut flags: Vec<LineMark>) -> Vec<LineMark> {
    flags.retain(|f| !pauses.iter().any(|p| p.line == f.line));
    pauses.sort_by(|a, b| a.line.cmp(&b.line));
    flags.sort_by(|a, b| a.line.cmp(&b.line));
    pauses.extend(flags);
    pauses
}

/// The pause marks and the flag marks stored in the state directory, as written.
fn stored_marks(state: &StateDir) -> (Vec<LineMark>, Vec<LineMark>) {
    let Ok(dir) = std::fs::read_dir(scope_dir(state)) else {
        return (Vec::new(), Vec::new());
    };
    let mut pauses = Vec::new();
    let mut flags = Vec::new();
    for entry in dir.filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(mark) = read_mark(&entry.path()) {
            if name.ends_with(".pause.json") {
                pauses.push(mark);
            } else if name.ends_with(".flag.json") {
                flags.push(mark);
            }
        }
    }
    (pauses, flags)
}

/// How an attempt ended, from the state of its job and its closing reason; `None` while the job has not
/// reached a final state. A job closed `failed` whose reason is an environment stop by
/// [`crate::disk::StopCause`] (the one classifier; this module keeps no list of its own) is
/// [`Verdict::Environment`]. Only a job closed `failed` can be one: a timed-out producer shares the text
/// `the producer did not complete` but kept work that ran out of time, which is spend of the work.
pub(crate) fn verdict_of(state: JobState, reason: Option<&str>) -> Option<Verdict> {
    match state {
        JobState::Closed(OutcomeState::Landed) => Some(Verdict::Accepted),
        JobState::Closed(OutcomeState::Rejected) => Some(Verdict::Rejected),
        JobState::Closed(OutcomeState::Failed)
            if reason.is_some_and(crate::disk::is_environment_reason) =>
        {
            Some(Verdict::Environment)
        }
        JobState::Closed(_) => Some(Verdict::Failed),
        _ => None,
    }
}

/// The finished attempt a request's job made, when it has one: it was dispatched and reached a final state
/// ([`verdict_of`]); `resumed` says whether it resumed a kept candidate (see [`AttemptRecord::resumed`]).
fn attempt_of(job: &JobRecords, resumed: bool) -> Option<AttemptRecord> {
    job.job.as_ref()?;
    let verdict = verdict_of(job.state(), job.outcome.as_ref().map(|o| o.reason.as_str()))?;
    let unmet_criterion = if verdict == Verdict::Rejected {
        job.verification
            .as_ref()
            .and_then(|v| v.criteria.iter().find(|c| !c.met))
            .map(|c| c.criterion.clone())
    } else {
        None
    };
    let totals = job.outcome.as_ref().and_then(|o| o.totals.as_ref());
    // `cost_execution` is what the reading's `cost_usd` forecast covers (producer and verifier).
    let cost_dollars = totals
        .map(|t| t.cost_execution.best_lower())
        .unwrap_or_else(|| job.spent_usd());
    let agent_minutes = totals
        .and_then(|t| t.wall_time_execution.as_ref())
        .and_then(|q| q.value())
        .map_or(0, |s| (s / 60.0).round().max(0.0) as u32);
    let plan = job.plans.last();
    Some(AttemptRecord {
        verdict,
        unmet_criterion,
        cost_dollars,
        agent_minutes,
        plan_size: plan.map(|p| (p.files.len() + p.steps.len()) as u32),
        planning_round: plan.map_or(1, |p| p.round),
        resumed,
    })
}

/// Whether `job`, a retry of `target`, resumed the target's kept candidate. Basis: the target stopped in a way
/// `athena retry` resumes ([`crate::disk::resumable_stop`]) and this job ran no producer. The resume mark in the
/// state directory is spent when the job closes, so the records are the only evidence left. A retry that planned
/// and built again has a producer invocation and is judged as a fresh job.
fn resumed_candidate(job: &JobRecords, target: Option<&JobRecords>) -> bool {
    job.job.is_some()
        && !job.invocations.iter().any(|i| i.role == Role::Producer)
        && target.is_some_and(|t| {
            crate::disk::resumable_stop(t.state(), t.outcome.as_ref().map(|o| o.reason.as_str()))
        })
}

/// Rebuilds the [`RequestRecord`]s of every request job, oldest first: the retry link from "try again on
/// <job>" (the previous job of the line), the re-ask link to an earlier request with the same text, and the
/// finished attempt read from the job's outcome and verification.
///
/// A job id that merely appears in a request's text (for example as evidence) links nothing: only the retry
/// link and the re-ask link join a request to a line.
///
/// A re-ask links to the nearest earlier request with the same normalised text ([`normalise`]) whose line has
/// not been accepted. A request whose earlier twin landed starts a line of its own: asking the same words again
/// after they were done is new work, and an accepted line shows no signals. The chain link
/// `athena.candidate/2` `previous_job` is not used: it names the latest job landed in the base's history,
/// which every job started on that base shares, so grouping on it would join unrelated work.
pub fn request_records(records: &Records) -> Vec<RequestRecord> {
    let mut jobs = records.request_jobs();
    // Oldest request first. Ids carry the time to the second only, so the time of the request decides.
    jobs.sort_by(|a, b| {
        let at = |j: &JobRecords| j.request.as_ref().map(|r| r.received_at);
        (at(a), a.id().as_str()).cmp(&(at(b), b.id().as_str()))
    });
    // Normalised request text -> the latest request with it whose line is not accepted.
    let mut open_by_text: HashMap<String, String> = HashMap::new();
    // Each request's retry link, so an accepted retry closes the requests it retried.
    let mut retry_parent: HashMap<String, String> = HashMap::new();
    jobs.iter()
        .map(|job| {
            let id = job.id().as_str().to_string();
            let text = job.request.as_ref().map_or("", |r| r.text.as_str());
            let retry_of = retried_job(records, text)
                .map(|j| j.as_str().to_string())
                .filter(|t| *t != id);
            let resumed = resumed_candidate(
                job,
                retry_of.as_ref().and_then(|t| records.jobs.get(t.as_str())),
            );
            let attempts: Vec<AttemptRecord> = attempt_of(job, resumed).into_iter().collect();
            if let Some(parent) = &retry_of {
                retry_parent.insert(id.clone(), parent.clone());
            }
            let key = normalise(text);
            let mut same_text_as = None;
            if !key.is_empty() {
                same_text_as = open_by_text.get(&key).cloned();
                open_by_text.insert(key, id.clone());
            }
            if done(&attempts) {
                // The line is accepted: a later request with the words of any request in it starts anew.
                let mut line = vec![id.clone()];
                let mut at = id.clone();
                while let Some(parent) = retry_parent.get(&at).filter(|p| !line.contains(p)) {
                    line.push(parent.clone());
                    at = parent.clone();
                }
                open_by_text.retain(|_, job| !line.contains(job));
            }
            RequestRecord {
                retry_of,
                same_text_as,
                attempts,
                id,
            }
        })
        .collect()
}

/// The forecast p90 in dollars for a job: the latest reading's `forecasts.reader.cost_usd.p90()`, which
/// forecasts the job's producer and verifier cost. 0 when there is no reading or the value is not positive;
/// [`t3_cost`] ignores a forecast of 0.
fn forecast_p90_dollars(job: &JobRecords) -> f64 {
    let p90 = job
        .latest_reading()
        .map_or(0.0, |r| r.forecasts.reader.cost_usd.p90());
    if p90.is_finite() && p90 > 0.0 {
        p90
    } else {
        0.0
    }
}

/// The forecast p90 T3 compares a line with: that of the newest request of the line that has a finished
/// attempt, or 0 when none has. It is the request whose reading was in force when the last attempt ended, so a
/// request that has only just joined the line (no reading, no attempt) does not change it. The watcher that
/// writes a mark and the check that reads one back both use this, so they agree.
fn line_forecast_p90(records: &Records, requests: &[RequestRecord], line: &[usize]) -> f64 {
    line.iter()
        .rev()
        .map(|&i| &requests[i])
        .find(|r| !r.attempts.is_empty())
        .and_then(|r| records.jobs.get(&r.id))
        .map_or(0.0, forecast_p90_dollars)
}

/// A line's finished attempts, the forecast p90 given to T3 and the signals, under the current rule for which
/// stops count ([`verdict_of`]). The watcher and [`Athena::line_pause`] both call it.
fn line_state(
    records: &Records,
    requests: &[RequestRecord],
    line: &[usize],
) -> (Vec<AttemptRecord>, f64, Signals) {
    let attempts = line_attempts(requests, line);
    let p90 = line_forecast_p90(records, requests, line);
    let signals = evaluate_before_attempt(&attempts, p90);
    (attempts, p90, signals)
}

fn facts(signals: &Signals, attempts: &[AttemptRecord], numbers: &MarkNumbers) -> Vec<SignalFact> {
    let mut out = Vec::new();
    let mut add = |signal: &str, level: Level, detail: String| {
        if level != Level::None {
            out.push(SignalFact {
                signal: signal.to_string(),
                level: if level == Level::Pause {
                    "pause"
                } else {
                    "flag"
                }
                .to_string(),
                detail,
            });
        }
    };
    add(
        "T1",
        signals.t1,
        format!(
            "{} attempts finished without an accepted candidate; the next would be attempt {} (flag from attempt {T1_ATTEMPT})",
            numbers.attempts,
            numbers.attempts + 1
        ),
    );
    let attempts = counted(attempts);
    let sizes: Vec<u32> = attempts.iter().filter_map(|a| a.plan_size).collect();
    add(
        "T2",
        signals.t2,
        format!(
            "plan size {} then {}, planning round {} (flag over {T2_RATIO} times the first plan or at round {T2_PLANNING_ROUND})",
            sizes.first().map_or("?".to_string(), |s| s.to_string()),
            sizes.last().map_or("?".to_string(), |s| s.to_string()),
            attempts.iter().map(|a| a.planning_round).max().unwrap_or(1)
        ),
    );
    add(
        "T3",
        signals.t3,
        format!(
            "spend ${:.2} against forecast p90 ${:.2} (flag at {T3_FLAG_RATIO} times, pause at {T3_PAUSE_RATIO} times; flag over ${T3_LINE_DOLLARS:.2} whatever the forecast)",
            numbers.spend_usd, numbers.forecast_p90_usd
        ),
    );
    add(
        "T4",
        signals.t4,
        format!(
            "{} rejections, the latest for criterion: {}",
            numbers.rejections,
            numbers
                .matched_criterion
                .as_deref()
                .unwrap_or("(not recorded)")
        ),
    );
    add(
        "T5",
        signals.t5,
        format!(
            "{} minutes of agent time with no accepted candidate (flag at {T5_FLAG_MINUTES}, pause at {T5_PAUSE_MINUTES})",
            numbers.agent_minutes
        ),
    );
    out
}

impl Athena {
    /// The pause on the line that `job_id` belongs to, if there is one. Rebuilds the [`RequestRecord`]s from
    /// `records`, groups them with [`group_lines`], and looks the pause file up by the line's first request id.
    ///
    /// The stored pause is a record of what the watcher decided, not the decision: it is returned only while
    /// the line's attempts, read under the current rule for which stops count, still reach a pause. A pause
    /// written when an environment stop was counted as spend stops blocking the line once the rule no longer
    /// counts it. The file is left as it is; this never creates a pause.
    pub fn line_pause(&self, records: &Records, job_id: &str) -> Option<LineMark> {
        let requests = request_records(records);
        let at = requests.iter().position(|r| r.id == job_id)?;
        let lines = group_lines(&requests);
        let line = lines.iter().find(|l| l.contains(&at))?;
        let first = &requests[line[0]].id;
        let mark = read_mark(&pause_path(&self.state, first)).filter(LineMark::is_pause)?;
        (line_state(records, &requests, line).2.overall() == Level::Pause).then_some(mark)
    }

    /// The marks `athena status` shows: the stored marks ([`list_lines`]) that still hold. A pause is shown
    /// only while the line's recomputed level is a pause ([`Athena::line_pause`]), a flag while it is a flag or
    /// a pause. A mark whose line has no finished attempt in `records` cannot be recomputed and is shown as
    /// stored.
    pub fn live_marks(&self, records: &Records) -> Vec<LineMark> {
        let requests = request_records(records);
        let lines = group_lines(&requests);
        let holds = |mark: &LineMark| {
            let Some(line) = lines.iter().find(|l| requests[l[0]].id == mark.line) else {
                return true;
            };
            if line_attempts(&requests, line).is_empty() {
                return true;
            }
            let level = line_state(records, &requests, line).2.overall();
            if mark.is_pause() {
                level == Level::Pause
            } else {
                level >= Level::Flag
            }
        };
        // Stale pauses go before the merge, so the flag of a line whose pause is stale is still shown.
        let (pauses, flags) = stored_marks(&self.state);
        merge_marks(
            pauses.into_iter().filter(&holds).collect(),
            flags.into_iter().filter(&holds).collect(),
        )
    }

    /// The refusal text for a paused line: the signal, the reason, what lifts it and which file to delete.
    pub fn pause_refusal(&self, mark: &LineMark) -> String {
        format!(
            "line {} is paused ({}): {}. To lift it: {}. The pause is the file {}",
            mark.line,
            mark.signal_names(),
            mark.reason,
            mark.lift,
            pause_path(&self.state, &mark.line).display()
        )
    }

    /// After an attempt of a line finished: evaluates T1 to T5 for the line `job` belongs to and records a
    /// flag or a pause. Never changes a job and never touches the process registry. An error is shown and
    /// ignored: the watcher cannot change the result of `go`.
    pub fn watch_line(&self, job: &JobId, ui: &dyn Ui) {
        if let Err(e) = self.try_watch_line(job, ui) {
            ui.line(&format!("scope      the line watcher could not run: {e:#}"));
        }
    }

    fn try_watch_line(&self, job: &JobId, ui: &dyn Ui) -> Result<()> {
        let head = self.head()?;
        let records = Records::load(&self.git, &head)?;
        let requests = request_records(&records);
        let Some(at) = requests.iter().position(|r| r.id == job.as_str()) else {
            return Ok(());
        };
        let lines = group_lines(&requests);
        let Some(line) = lines.iter().find(|l| l.contains(&at)) else {
            return Ok(());
        };
        let first = requests[line[0]].id.clone();
        let (attempts, p90, signals) = line_state(&records, &requests, line);
        // The mark shows the numbers the signals used: environment stops are not counted, and a resumed retry
        // is shown against the forecast of its remaining stages.
        let counted_attempts = counted(&attempts);
        let rejected: Vec<&AttemptRecord> = counted_attempts
            .iter()
            .copied()
            .filter(|a| a.verdict == Verdict::Rejected)
            .collect();
        let (spend_usd, forecast_p90_usd) = t3_basis(&attempts, p90);
        let numbers = MarkNumbers {
            attempts: counted_attempts.len(),
            rejections: rejected.len(),
            spend_usd,
            forecast_p90_usd,
            agent_minutes: counted_attempts.iter().map(|a| a.agent_minutes).sum(),
            matched_criterion: rejected.last().and_then(|a| a.unmet_criterion.clone()),
        };
        let (flag_file, pause_file) = (
            flag_path(&self.state, &first),
            pause_path(&self.state, &first),
        );
        let level = signals.overall();
        if level == Level::None {
            let _ = std::fs::remove_file(&flag_file);
            return Ok(());
        }
        let facts = facts(&signals, &attempts, &numbers);
        let paused = level == Level::Pause;
        let reason = facts
            .iter()
            .filter(|f| f.level == if paused { "pause" } else { "flag" })
            .map(|f| format!("{}: {}", f.signal, f.detail))
            .collect::<Vec<_>>()
            .join("; ");
        let lift = if paused && signals.t4 == Level::Pause {
            "narrow the request so it asks for less than the criterion that was rejected twice, or add a measurement of what fails".to_string()
        } else if paused {
            "a measurement of cost or progress that shows the next attempt will not repeat the spend".to_string()
        } else {
            "no action needed; the line is still dispatched".to_string()
        };
        let mark = LineMark {
            level: if paused { "pause" } else { "flag" }.to_string(),
            line: first.clone(),
            job: job.as_str().to_string(),
            at: crate::now().to_string(),
            signals: facts,
            numbers,
            reason,
            lift: if paused {
                format!("{lift}. Then delete {}", pause_file.display())
            } else {
                lift
            },
        };
        let names = mark.signal_names();
        if paused {
            write_mark(pause_file, &mark)?;
            let _ = std::fs::remove_file(&flag_file);
            ui.line(&format!(
                "scope      line {first} paused: {names}; {}",
                mark.reason
            ));
        } else {
            // A pause file stays until the owner deletes it; a flag does not replace it.
            write_mark(flag_file, &mark)?;
            ui.line(&format!("scope      line {first} flagged: {names}"));
        }
        Ok(())
    }
}

/// Watcher tests: they drive `Athena::go` with the fake agent driver, as `run_pin_tests` does.
#[cfg(test)]
mod watcher_tests {
    use super::*;
    use crate::Quiet;
    use crate::capability::{
        Capability, CapabilityCard, ChangeList, CheckSpec, JobConfig, ToolGrant,
    };
    use crate::run::GoResult;
    use athena_agent::fake::{FakeAgentDriver, FakeTurn};
    use athena_agent::{AgentInvocation, Usage};
    use athena_context::{PRODUCER_FRAME, PromptAsset, VERIFIER_FRAME};
    use athena_data::records::{AcceptedBy, Build, PolicyViolation, Role};
    use athena_git::Git;
    use serde_json::json;
    use std::collections::{BTreeMap, VecDeque};
    use std::sync::{Arc, Mutex};

    /// The criterion texts the fake reader gives; the verifier fails the one whose index the script names.
    const CRITERIA: [&str; 2] = ["README.md describes the demo", "the file is plain text"];

    /// A capability that edits files and runs no checks.
    struct Docs;

    impl Capability for Docs {
        fn id(&self) -> &'static str {
            "study"
        }
        fn card(&self) -> CapabilityCard {
            CapabilityCard {
                id: "study",
                purpose: "describe a demo.",
                when: "for a request to describe a demo.",
                produces: "files describing the demo.",
            }
        }
        fn producer_section(&self) -> PromptAsset {
            PRODUCER_FRAME
        }
        fn verifier_section(&self) -> PromptAsset {
            VERIFIER_FRAME
        }
        fn producer_grant(&self, _: &JobConfig) -> ToolGrant {
            ToolGrant {
                tools: vec!["Read".into()],
                commands: BTreeMap::new(),
                writes: true,
                permission_mode: "acceptEdits".into(),
            }
        }
        fn verifier_grant(&self, _: &JobConfig) -> ToolGrant {
            ToolGrant {
                tools: vec!["Read".into()],
                commands: BTreeMap::new(),
                writes: false,
                permission_mode: "dontAsk".into(),
            }
        }
        fn checks(&self, _: &JobConfig) -> Vec<CheckSpec> {
            Vec::new()
        }
        fn env(&self, _: &JobConfig) -> BTreeMap<String, String> {
            BTreeMap::new()
        }
        fn admit(&self, _: &ChangeList, _: &JobConfig) -> Vec<PolicyViolation> {
            Vec::new()
        }
    }

    /// What the fake agents do.
    struct Plan {
        /// The reading's forecast p90 for the job's cost.
        p90: f64,
        /// What each producer and each verifier call costs.
        cost: f64,
        /// One entry per verifier call: the index of the criterion to fail, or `None` to pass.
        verdicts: Mutex<VecDeque<Option<usize>>>,
    }

    struct Rig {
        _dir: tempfile::TempDir,
        _state: tempfile::TempDir,
        athena: Athena,
    }

    fn rig(p90: f64, cost: f64, verdicts: &[Option<usize>]) -> Rig {
        rig_with_space(p90, cost, verdicts, crate::test_support::plenty_of_space)
    }

    /// As [`rig`], with the given free-space reader (for a test that makes the disk run low).
    fn rig_with_space(
        p90: f64,
        cost: f64,
        verdicts: &[Option<usize>],
        free_space: crate::state::FreeSpace,
    ) -> Rig {
        let raw = include_str!("../../../athena.toml");
        let mut in_job_plan = false;
        let config: String = raw
            .split_inclusive('\n')
            .filter(|line| {
                let key = line.trim_start();
                if key.starts_with('[') {
                    in_job_plan = key.trim_end() == "[job_plan]";
                }
                !(in_job_plan || key.starts_with("repo_url") || key.starts_with("repo_ref"))
            })
            .collect();
        let dir = tempfile::tempdir().unwrap();
        let git = Git::at(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value]).unwrap();
        }
        for (path, text) in [
            ("athena.toml", config.as_str()),
            ("defaults.toml", include_str!("../../../defaults.toml")),
            ("README.md", "# Demo\n"),
        ] {
            std::fs::write(dir.path().join(path), text).unwrap();
        }
        git.run(["add", "-A"]).unwrap();
        git.run([
            "commit",
            "-q",
            "-m",
            "Start a demo repository\n\nAthena-Job: bootstrap",
        ])
        .unwrap();
        let state_dir = tempfile::tempdir().unwrap();
        let state = StateDir::new(state_dir.path().join("repo")).with_free_space(free_space);
        let plan = Arc::new(Plan {
            p90,
            cost,
            verdicts: Mutex::new(verdicts.iter().copied().collect()),
        });
        let script = Box::new(move |spec: &AgentInvocation| {
            // A reading of a retry must name the earlier job whose result the brief carries.
            let earlier: String = spec
                .user_prompt
                .lines()
                .filter(|l| l.contains(": end state "))
                .filter_map(|l| l.strip_prefix("Job ")?.split(':').next())
                .map(|id| format!(" Retries {id}."))
                .collect();
            let mut turn = match spec.role {
                Role::Reader => FakeTurn::output(json!({
                    "title": "Describe the demo",
                    "summary": format!("Describe the demo in README.md.{earlier}"),
                    "class_of_work": "docs",
                    "capability": "study",
                    "decline_reason": null,
                    "producer_model": "claude-haiku-5-5",
                    "defaults_applied": [{ "id": "strangers", "how": "plain words" }],
                    "assumptions": [],
                    "question": null,
                    "acceptance_criteria": [
                        { "text": CRITERIA[0], "kind": "content" },
                        { "text": CRITERIA[1], "kind": "content" }
                    ],
                    "deliverables": [{ "path": "README.md", "action": "modify", "p": 0.9 }],
                    "out_of_scope": ["Other files"],
                    "forecast": {
                        "p_reading_accepted": 0.8,
                        "p_verification_passes": 0.7,
                        "p_user_intervenes": 0.1,
                        "cost_usd": { "p10": plan.p90 / 4.0, "p50": plan.p90 / 2.0, "p90": plan.p90 },
                        "wall_time_s": { "p10": 5.0, "p50": 30.0, "p90": 300.0 },
                        "departures": null
                    }
                })),
                Role::Producer => {
                    let mut t = FakeTurn::output(json!({
                        "summary": "Described the demo.",
                        "files": ["README.md"],
                        "checks_run": [],
                        "criteria_check": [
                            { "criterion": CRITERIA[0], "status": "met", "evidence": "read it" },
                            { "criterion": CRITERIA[1], "status": "met", "evidence": "read it" }
                        ],
                        "open_issues": [], "mistakes": [], "learnings": [], "desires": []
                    }));
                    t.edits = vec![("README.md".into(), Some("# Demo\n\nA demo.\n".into()))];
                    t
                }
                Role::Verifier => {
                    let fail = plan.verdicts.lock().unwrap().pop_front().flatten();
                    FakeTurn::output(json!({
                        "verdict": if fail.is_some() { "fail" } else { "pass" },
                        "criteria": [
                            { "criterion": CRITERIA[0], "met": fail != Some(0), "evidence": "checked", "commands": [] },
                            { "criterion": CRITERIA[1], "met": fail != Some(1), "evidence": "checked", "commands": [] }
                        ],
                        "findings": [],
                        "request_reading_mismatch": null,
                        "summary": "Checked."
                    }))
                }
                Role::Planner | Role::Critic => unreachable!("planning is off in this repository"),
                Role::Steward => FakeTurn::output(json!({ "items": [] })),
                Role::Audience => FakeTurn::output(json!({ "rating": 3, "findings": [] })),
                Role::Auditor => FakeTurn::output(json!({ "summary": "Fine.", "findings": [] })),
            };
            turn.cost_usd = if spec.role == Role::Reader {
                0.01
            } else {
                plan.cost
            };
            turn.usage = Usage {
                input: 1,
                output: 1,
                cache_read: 1,
                cache_write: 1,
            };
            turn
        });
        let driver = FakeAgentDriver::new(script).with_registry(state.registry());
        let athena = Athena {
            git,
            state,
            driver: Arc::new(driver),
            capabilities: vec![Box::new(Docs)],
            build: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            athena_bin: "/bin/false".into(),
            nested: None,
            pinner: Box::new(crate::fake::CountingPinner::new(&"a".repeat(40))),
            jev: None,
        };
        Rig {
            _dir: dir,
            _state: state_dir,
            athena,
        }
    }

    impl Rig {
        fn ask(&self, text: &str) -> JobId {
            self.athena.ask(text, &Quiet).unwrap().job
        }

        fn go(&self, job: &JobId) -> Result<OutcomeState> {
            match self.athena.go(job.as_str(), AcceptedBy::User, &Quiet)? {
                GoResult::Finished(f) => Ok(f.state),
                GoResult::Reread(_) => panic!("the job was read again"),
            }
        }

        fn records(&self) -> Records {
            Records::load(&self.athena.git, &self.athena.head().unwrap()).unwrap()
        }

        fn pause(&self, first: &JobId) -> PathBuf {
            pause_path(&self.athena.state, first.as_str())
        }

        fn flag(&self, first: &JobId) -> PathBuf {
            flag_path(&self.athena.state, first.as_str())
        }

        /// Two attempts rejected for the same criterion, through `ask` and `go`.
        fn two_rejections(&self) -> (JobId, JobId) {
            let a = self.ask("describe the demo in the readme");
            assert_eq!(self.go(&a).unwrap(), OutcomeState::Rejected);
            let b = self.ask(&format!("try again on {a}"));
            assert_eq!(self.go(&b).unwrap(), OutcomeState::Rejected);
            (a, b)
        }
    }

    fn mark(level: &str, line: &str, signal: &str) -> LineMark {
        LineMark {
            level: level.into(),
            line: line.into(),
            job: line.into(),
            at: "2026-10-26T00:00:00Z".into(),
            signals: vec![SignalFact {
                signal: signal.into(),
                level: level.into(),
                detail: "two rejections".into(),
            }],
            numbers: MarkNumbers {
                attempts: 2,
                rejections: 2,
                spend_usd: 0.5,
                forecast_p90_usd: 0.0,
                agent_minutes: 1,
                matched_criterion: None,
            },
            reason: format!("{signal}: two rejections"),
            lift: "narrow the request".into(),
        }
    }

    #[test]
    fn t4_pause_via_go_and_ask() {
        let r = rig(0.8, 0.01, &[Some(0), Some(0)]);
        let a = r.ask("describe the demo in the readme");
        assert_eq!(r.go(&a).unwrap(), OutcomeState::Rejected);
        assert!(!r.pause(&a).exists(), "one rejection pauses nothing");
        let b = r.ask(&format!("try again on {a}"));
        assert_eq!(r.go(&b).unwrap(), OutcomeState::Rejected);
        assert!(
            r.pause(&a).exists(),
            "two rejections of one criterion pause the line"
        );

        // The third request is made with `ask`, not `retry`; `go` refuses to dispatch it.
        let c = r.ask(&format!("try again on {b}"));
        let err = r.go(&c).unwrap_err().to_string();
        assert!(err.contains("paused") && err.contains("T4"), "{err}");
        assert!(
            r.pause(&a).exists(),
            "the refusal leaves the pause in place"
        );
        let records = r.records();
        assert!(
            records.jobs[c.as_str()].job.is_none(),
            "the third attempt was not dispatched"
        );

        // Control: with no pause file the same request is dispatched.
        std::fs::remove_file(r.pause(&a)).unwrap();
        assert_eq!(r.go(&c).unwrap(), OutcomeState::Landed);
        assert!(r.records().jobs[c.as_str()].job.is_some());
    }

    #[test]
    fn citing_a_paused_lines_jobs_as_evidence_starts_its_own_line() {
        let r = rig(0.8, 0.01, &[Some(0), Some(0)]);
        let (a, b) = r.two_rejections();
        assert!(r.pause(&a).exists(), "the line is paused naturally");

        // A separate fix that only cites the paused line's jobs as evidence.
        let c = r.ask(&format!(
            "fix disk and resume handling; evidence: jobs {a} and {b} were paused"
        ));
        let records = r.records();
        let requests = request_records(&records);
        assert_eq!(requests[2].retry_of, None);
        assert_eq!(requests[2].same_text_as, None);
        assert_eq!(
            group_lines(&requests),
            vec![vec![0, 1], vec![2]],
            "citing jobs as evidence is not a link"
        );
        assert_eq!(r.athena.line_pause(&records, c.as_str()), None);
        match r.go(&c) {
            Ok(_) => {}
            Err(e) => assert!(!e.to_string().contains("paused"), "{e}"),
        }
        assert!(
            r.records().jobs[c.as_str()].job.is_some(),
            "the citing request was dispatched, not refused"
        );

        // A retry of the paused line is still part of it and is refused.
        let d = r.ask(&format!("try again on {b}"));
        let records = r.records();
        let requests = request_records(&records);
        assert_eq!(requests[3].retry_of.as_deref(), Some(b.as_str()));
        assert_eq!(
            group_lines(&requests),
            vec![vec![0, 1, 3], vec![2]],
            "the retry joins the paused line"
        );
        let err = r.go(&d).unwrap_err().to_string();
        assert!(err.contains("paused") && err.contains("T4"), "{err}");
    }

    #[test]
    fn a_retry_and_a_re_ask_of_the_same_request_are_one_line_that_pauses_at_its_threshold() {
        // Criterion 0 is rejected, then criterion 1 twice. Counted per job id no request has two rejections of
        // one criterion; counted per line, the retry and the re-ask are attempts 2 and 3 of one line.
        let r = rig(0.8, 0.01, &[Some(0), Some(1), Some(1)]);
        let a = r.ask("describe the demo in the readme");
        assert_eq!(r.go(&a).unwrap(), OutcomeState::Rejected);
        let b = r.ask(&format!("try again on {a}"));
        assert_eq!(r.go(&b).unwrap(), OutcomeState::Rejected);
        assert!(
            !r.pause(&a).exists(),
            "different criteria, so the line is only flagged"
        );
        assert!(r.flag(&a).exists());

        // The same words asked again, with no mention of the job: a new job id on the same line.
        let c = r.ask("describe the demo in the readme");
        assert_ne!(c, a);
        assert_eq!(r.go(&c).unwrap(), OutcomeState::Rejected);
        let paused = read_mark(&r.pause(&a)).expect("the third attempt of the line pauses it");
        assert_eq!(paused.signal_names(), "T1, T4");
        assert_eq!(paused.numbers.attempts, 3);
        assert_eq!(paused.numbers.rejections, 3);
        assert!(
            !r.pause(&c).exists() && !r.flag(&c).exists(),
            "the re-ask has no line of its own"
        );

        let records = r.records();
        let requests = request_records(&records);
        assert_eq!(
            group_lines(&requests),
            vec![vec![0, 1, 2]],
            "retry and re-ask are one line"
        );
        assert_eq!(
            r.athena.line_pause(&records, c.as_str()),
            Some(paused),
            "the re-ask is refused through the line's pause"
        );
        let d = r.ask("describe the demo in the readme");
        let err = r.go(&d).unwrap_err().to_string();
        assert!(err.contains("paused") && err.contains("T4"), "{err}");
    }

    #[test]
    fn asking_the_words_of_a_landed_request_again_starts_a_new_line() {
        let r = rig(0.8, 0.01, &[None, Some(0)]);
        let a = r.ask("describe the demo in the readme");
        assert_eq!(r.go(&a).unwrap(), OutcomeState::Landed);
        let b = r.ask("describe the demo in the readme");
        // The fake producer writes the same file again, so this job does not land (it ends without a verdict).
        assert_ne!(r.go(&b).unwrap(), OutcomeState::Landed);
        let requests = request_records(&r.records());
        assert_eq!(group_lines(&requests), vec![vec![0], vec![1]]);
        assert_eq!(requests[1].same_text_as, None);
    }

    #[test]
    fn differing_criteria_flag_only() {
        let r = rig(0.8, 0.01, &[Some(0), Some(1)]);
        let (a, b) = r.two_rejections();
        assert!(!r.pause(&a).exists(), "differing criteria do not pause");
        let flag = read_mark(&r.flag(&a)).expect("a flag is recorded");
        assert_eq!(flag.level, "flag");
        assert_eq!(flag.signal_names(), "T1");
        let c = r.ask(&format!("try again on {b}"));
        assert_eq!(
            r.go(&c).unwrap(),
            OutcomeState::Landed,
            "the next attempt is dispatched"
        );
        assert!(
            !r.flag(&a).exists(),
            "an accepted line has no signal left, so its flag is cleared"
        );
    }

    #[test]
    fn retry_refuses_paused_line() {
        // Two rejections of one criterion: the line really is at a pause, so the stored pause holds.
        let r = rig(0.8, 0.01, &[Some(0), Some(0)]);
        let (a, b) = r.two_rejections();
        // Control: with the pause file removed, retry reads the request again.
        std::fs::remove_file(r.pause(&a)).unwrap();
        r.athena
            .retry(b.as_str(), &Quiet)
            .expect("retry without a pause");
        let tip = r.athena.head().unwrap();

        let paused = mark("pause", a.as_str(), "T4");
        write_mark(r.pause(&a), &paused).unwrap();
        let err = r.athena.retry(b.as_str(), &Quiet).unwrap_err().to_string();
        assert!(
            err.contains("T4") && err.contains("two rejections"),
            "{err}"
        );
        assert_eq!(
            r.athena.head().unwrap(),
            tip,
            "a refused retry commits nothing"
        );

        // The pause is found through RequestRecords and group_lines, keyed by the line's first id, for
        // the retried job and for a later job on the line.
        let records = r.records();
        assert_eq!(
            r.athena.line_pause(&records, b.as_str()),
            Some(paused.clone())
        );
        let later = records
            .request_jobs()
            .into_iter()
            .map(|j| j.id().as_str().to_string())
            .find(|id| id != a.as_str() && id != b.as_str())
            .expect("the control retry made a later job on the line");
        assert_eq!(r.athena.line_pause(&records, &later), Some(paused));
        let other = r.ask("something unrelated about the licence");
        assert_eq!(r.athena.line_pause(&r.records(), other.as_str()), None);
    }

    #[test]
    fn a_stored_pause_that_no_longer_holds_is_not_returned_and_does_not_block_a_retry() {
        // Two rejections of different criteria: the line is flagged, not at a pause.
        let r = rig(0.8, 0.01, &[Some(0), Some(1)]);
        let (a, b) = r.two_rejections();
        assert!(!r.pause(&a).exists());
        // A pause written under an earlier rule, such as one that counted an environment stop as spend.
        write_mark(r.pause(&a), &mark("pause", a.as_str(), "T4")).unwrap();
        let records = r.records();
        assert_eq!(r.athena.line_pause(&records, b.as_str()), None);
        assert_eq!(r.athena.line_pause(&records, a.as_str()), None);
        assert!(r.pause(&a).exists(), "the stored file is left as it is");
        r.athena
            .retry(b.as_str(), &Quiet)
            .expect("a pause that no longer holds does not refuse the retry");

        // Control: two rejections of one criterion are a real pause, and it is returned and refuses.
        let r = rig(0.8, 0.01, &[Some(0), Some(0)]);
        let (a, b) = r.two_rejections();
        let records = r.records();
        let held = r.athena.line_pause(&records, b.as_str()).expect("T4 holds");
        assert!(held.signal_names().contains("T4"), "{held:?}");
        assert!(r.athena.retry(b.as_str(), &Quiet).is_err());
        assert!(r.pause(&a).exists());
    }

    #[test]
    fn status_shows_only_the_marks_that_still_hold() {
        let r = rig(0.8, 0.01, &[Some(0), Some(1)]);
        let (a, _) = r.two_rejections();
        assert!(r.flag(&a).exists(), "T1 flags the line naturally");
        // A stale pause on the same line, and a mark for a line the records do not know.
        write_mark(r.pause(&a), &mark("pause", a.as_str(), "T4")).unwrap();
        write_mark(
            pause_path(&r.athena.state, "20261026-000001-aaaa"),
            &mark("pause", "20261026-000001-aaaa", "T4"),
        )
        .unwrap();
        let shown = |r: &Rig| -> Vec<(String, String)> {
            r.athena
                .status()
                .unwrap()
                .lines
                .iter()
                .map(|l| (l.level.clone(), l.line.clone()))
                .collect()
        };
        assert_eq!(
            shown(&r),
            vec![
                ("pause".into(), "20261026-000001-aaaa".into()),
                ("flag".into(), a.as_str().to_string()),
            ],
            "the stale pause is hidden and the flag that holds is shown"
        );

        // A flag for a line that has since landed no longer holds.
        let r = rig(0.8, 0.01, &[None]);
        let a = r.ask("describe the demo in the readme");
        assert_eq!(r.go(&a).unwrap(), OutcomeState::Landed);
        write_mark(r.flag(&a), &mark("flag", a.as_str(), "T1")).unwrap();
        assert!(shown(&r).is_empty());
    }

    #[test]
    fn a_t3_pause_still_holds_after_a_new_request_joins_the_line() {
        // One attempt costs 1.6 against a p90 of 1.0: a real T3 pause written by the watcher.
        let r = rig(1.0, 0.8, &[Some(0)]);
        let a = r.ask("describe the demo in the readme");
        assert_eq!(r.go(&a).unwrap(), OutcomeState::Rejected);
        assert_eq!(read_mark(&r.pause(&a)).unwrap().signal_names(), "T3");
        // A request that joins the line has no reading and no attempt yet. The forecast is that of the last
        // finished attempt, so the pause is not dropped for lack of one.
        let b = r.ask(&format!("try again on {a}"));
        // `retry` checks the line before the new request is read, so its records carry no reading for it.
        let mut records = r.records();
        records.jobs.get_mut(b.as_str()).unwrap().readings.clear();
        assert_eq!(
            line_forecast_p90(
                &records,
                &request_records(&records),
                &group_lines(&request_records(&records))[0]
            ),
            1.0,
            "the forecast is that of the request with the finished attempt"
        );
        let held = r.athena.line_pause(&records, b.as_str()).expect("T3 holds");
        assert_eq!(held.signal_names(), "T3");
        let err = r.go(&b).unwrap_err().to_string();
        assert!(
            err.contains("T3") && err.contains("1.60") && err.contains("1.00"),
            "{err}"
        );
        let tip = r.athena.head().unwrap();
        let err = r.athena.retry(a.as_str(), &Quiet).unwrap_err().to_string();
        assert!(err.contains("T3"), "{err}");
        assert_eq!(
            r.athena.head().unwrap(),
            tip,
            "a refused retry commits nothing"
        );
    }

    #[test]
    fn t3_pause_from_reading_forecast() {
        // Spend is 1.6 dollars on one attempt: 1.6 is at least 1.5 times the reading's p90 of 1.0.
        let r = rig(1.0, 0.8, &[Some(0)]);
        let a = r.ask("describe the demo in the readme");
        assert_eq!(r.go(&a).unwrap(), OutcomeState::Rejected);
        let paused = read_mark(&r.pause(&a)).expect("T3 pauses the line");
        assert_eq!(paused.signal_names(), "T3");
        assert!((paused.numbers.spend_usd - 1.6).abs() < 1e-9, "{paused:?}");
        assert!((paused.numbers.forecast_p90_usd - 1.0).abs() < 1e-12);
        let b = r.ask(&format!("try again on {a}"));
        let err = r.go(&b).unwrap_err().to_string();
        assert!(
            err.contains("T3") && err.contains("1.60") && err.contains("1.00"),
            "{err}"
        );
        std::fs::remove_file(r.pause(&a)).unwrap();
        assert_eq!(
            r.go(&b).unwrap(),
            OutcomeState::Landed,
            "without the pause it dispatches"
        );

        // The same spend against a reading whose p90 is 10.0: the limit is 15.0, so no pause. Only the
        // reading's forecast differs from the run above.
        let r = rig(10.0, 0.8, &[Some(0)]);
        let a = r.ask("describe the demo in the readme");
        assert_eq!(r.go(&a).unwrap(), OutcomeState::Rejected);
        assert!(!r.pause(&a).exists() && !r.flag(&a).exists());
    }

    #[test]
    fn a_low_disk_job_is_read_as_an_environment_attempt() {
        let r = rig_with_space(0.8, 0.01, &[None], crate::test_support::low_once_verifying);
        let a = r.ask("describe the demo in the readme");
        // Low space at the verifier gate holds the job: `go` returns the hold, and the job closes with no
        // outcome, so it is no attempt at all (neither a failure nor an environment attempt).
        let hold = r.go(&a).expect_err("low space holds the job");
        assert!(
            hold.downcast_ref::<crate::disk::LowDisk>().is_some(),
            "{hold:#}"
        );
        let records = r.records();
        assert!(records.jobs[a.as_str()].outcome.is_none());
        let requests = request_records(&records);
        assert!(requests[0].attempts.is_empty(), "{requests:?}");
        assert!(
            !r.pause(&a).exists() && !r.flag(&a).exists(),
            "a hold marks nothing"
        );

        // Space is back: the retry resumes the kept candidate (no producer), lands, and is read as resumed.
        std::fs::write(r.athena.state.root().join("freed"), b"").unwrap();
        let (retry, _) = r.athena.retry(a.as_str(), &Quiet).unwrap();
        assert_eq!(r.go(&retry.job).unwrap(), OutcomeState::Landed);
        let requests = request_records(&r.records());
        assert_eq!(requests[1].retry_of.as_deref(), Some(a.as_str()));
        assert!(requests[1].attempts[0].resumed, "{requests:?}");
        assert_eq!(requests[1].attempts[0].verdict, Verdict::Accepted);
        assert!(!r.pause(&a).exists() && !r.flag(&a).exists());
    }

    /// Kills and reaps a child at the end of a test, even when an assertion fails.
    struct Reaper(std::process::Child);

    impl Drop for Reaper {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    #[test]
    fn pause_leaves_running_child_alone() {
        use std::os::unix::process::CommandExt;
        let r = rig(0.8, 0.01, &[Some(0), Some(0)]);
        let a = r.ask("describe the demo in the readme");
        assert_eq!(r.go(&a).unwrap(), OutcomeState::Rejected);
        let mut child = Reaper(
            std::process::Command::new("sleep")
                .arg("60")
                .process_group(0)
                .spawn()
                .unwrap(),
        );
        let pgid = child.0.id() as i32;
        let registry = r.athena.state.registry();
        registry
            .register(pgid, a.as_str(), "test child of the line")
            .unwrap();
        let before = registry.entries();
        assert_eq!(before.len(), 1);

        let b = r.ask(&format!("try again on {a}"));
        assert_eq!(r.go(&b).unwrap(), OutcomeState::Rejected);
        assert!(r.pause(&a).exists(), "the watcher paused the line");
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "the child is still alive"
        );
        assert_eq!(
            registry.entries(),
            before,
            "the registry entry is unchanged"
        );
        registry.unregister(pgid);
    }

    #[test]
    fn pause_record_content() {
        let r = rig(0.8, 0.01, &[Some(1), Some(1)]);
        let (a, _) = r.two_rejections();
        let text = std::fs::read_to_string(r.pause(&a)).unwrap();
        let v: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["level"], "pause");
        assert_eq!(v["line"], a.as_str());
        assert_eq!(v["signals"][1]["signal"], "T4");
        assert_eq!(v["signals"][1]["level"], "pause");
        assert_eq!(v["numbers"]["rejections"], 2);
        assert_eq!(v["numbers"]["matched_criterion"], CRITERIA[1]);
        let lift = v["lift"].as_str().unwrap();
        assert!(lift.contains("narrow the request"), "{lift}");
        assert!(lift.contains(&format!("{a}.pause.json")), "{lift}");
        assert!(v["reason"].as_str().unwrap().contains(CRITERIA[1]));

        let r = rig(1.0, 0.8, &[Some(0)]);
        let a = r.ask("describe the demo in the readme");
        assert_eq!(r.go(&a).unwrap(), OutcomeState::Rejected);
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(r.pause(&a)).unwrap()).unwrap();
        assert_eq!(v["signals"][0]["signal"], "T3");
        assert!((v["numbers"]["spend_usd"].as_f64().unwrap() - 1.6).abs() < 1e-9);
        assert_eq!(v["numbers"]["forecast_p90_usd"], 1.0);
        assert!(v["lift"].as_str().unwrap().contains("measurement"));
    }

    #[test]
    fn status_lists_flagged_and_paused_lines() {
        let r = rig(0.8, 0.01, &[]);
        assert!(
            r.athena.status().unwrap().lines.is_empty(),
            "no marks, no lines"
        );
        write_mark(
            pause_path(&r.athena.state, "20261026-000001-aaaa"),
            &mark("pause", "20261026-000001-aaaa", "T4"),
        )
        .unwrap();
        write_mark(
            flag_path(&r.athena.state, "20261026-000002-bbbb"),
            &mark("flag", "20261026-000002-bbbb", "T1"),
        )
        .unwrap();
        let view = r.athena.status().unwrap();
        let shown: Vec<(String, String, String)> = view
            .lines
            .iter()
            .map(|l| (l.level.clone(), l.line.clone(), l.signal_names()))
            .collect();
        assert_eq!(
            shown,
            vec![
                ("pause".into(), "20261026-000001-aaaa".into(), "T4".into()),
                ("flag".into(), "20261026-000002-bbbb".into(), "T1".into()),
            ]
        );
        assert!(
            view.next
                .iter()
                .any(|n| n.contains("20261026-000001-aaaa") && n.contains("T4")),
            "{:?}",
            view.next
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn att(verdict: Verdict, criterion: Option<&str>, cost: f64, minutes: u32) -> AttemptRecord {
        AttemptRecord {
            verdict,
            unmet_criterion: criterion.map(str::to_string),
            cost_dollars: cost,
            agent_minutes: minutes,
            plan_size: None,
            planning_round: 1,
            resumed: false,
        }
    }

    fn rej(criterion: &str, cost: f64, minutes: u32) -> AttemptRecord {
        att(Verdict::Rejected, Some(criterion), cost, minutes)
    }

    fn req(id: &str, retry_of: Option<&str>) -> RequestRecord {
        RequestRecord {
            id: id.to_string(),
            retry_of: retry_of.map(str::to_string),
            same_text_as: None,
            attempts: Vec::new(),
        }
    }

    #[test]
    fn grouping_joins_a_re_ask_to_the_request_it_repeats() {
        let mut again = req("c", None);
        again.same_text_as = Some("a".into());
        let reqs = vec![req("a", None), req("b", Some("a")), req("x", None), again];
        assert_eq!(group_lines(&reqs), vec![vec![0, 1, 3], vec![2]]);
    }

    #[test]
    fn grouping_joins_retries_only() {
        let reqs = vec![
            req("a", None),
            req("x", None),
            req("b", Some("a")),
            req("c", None),
            req("d", Some("missing")),
        ];
        let lines = group_lines(&reqs);
        assert_eq!(lines, vec![vec![0, 2], vec![1], vec![3], vec![4]]);
    }

    #[test]
    fn normalise_lowercases_and_drops_numbers() {
        assert_eq!(
            normalise_criterion("Under 7 minutes"),
            normalise_criterion("under  12 MINUTES")
        );
        assert_eq!(normalise_criterion("Under 7 minutes"), "under minutes");
    }

    #[test]
    fn trailer_fires_t1_and_t3_before_attempt_3() {
        // Even split of about $11 over 3 attempts; criteria invented (not known from the record).
        let a = vec![
            rej("split keeps old names", 11.0 / 3.0, 30),
            rej("tests cover the new file", 11.0 / 3.0, 30),
        ];
        let before2 = evaluate_before_attempt(&a[..1], 6.0);
        assert_eq!(before2.overall(), Level::None);
        let before3 = evaluate_before_attempt(&a, 6.0);
        assert_eq!(before3.t1, Level::Flag);
        assert_eq!(before3.t3, Level::Flag); // about $7.33 is over the $5 line limit, under 1.5 x p90
        assert_eq!(before3.t4, Level::None);
        assert_eq!(before3.overall(), Level::Flag);
    }

    #[test]
    fn speedup_t4_pauses_after_second_rejection() {
        let a = vec![
            rej("Suite runs in under 7 minutes", 3.0, 80),
            rej("suite runs in under 12 minutes", 3.0, 80),
        ];
        let after1 = evaluate_before_attempt(&a[..1], 8.0);
        assert_eq!(after1.t1, Level::None);
        assert_eq!(after1.t3, Level::None);
        assert_eq!(after1.t4, Level::None);
        assert_eq!(after1.t5, Level::Flag); // 80 minutes
        let after2 = evaluate_before_attempt(&a, 8.0);
        assert_eq!(after2.t1, Level::Flag);
        assert_eq!(after2.t3, Level::Flag); // $6 over the $5 limit, under p90 x 1.0
        assert_eq!(after2.t4, Level::Pause);
        assert_eq!(after2.t5, Level::Pause); // 160 minutes
        assert_eq!(after2.overall(), Level::Pause);
    }

    #[test]
    fn t4_different_criteria_do_not_pause() {
        let a = vec![
            rej("timing race in the watcher test", 3.0, 10),
            rej("waits on the wrong check", 3.0, 10),
        ];
        assert_eq!(t4_match(&a), T4Match::No);
        assert_eq!(t4_level(&a), Level::None);
    }

    #[test]
    fn t4_unclear_match_flags_without_pausing() {
        let contained = vec![
            rej("time target", 1.0, 10),
            rej("the time target is not met", 1.0, 10),
        ];
        assert_eq!(t4_match(&contained), T4Match::Maybe);
        assert_eq!(t4_level(&contained), Level::Flag);
        let missing = vec![att(Verdict::Rejected, None, 1.0, 10), rej("x y", 1.0, 10)];
        assert_eq!(evaluate_before_attempt(&missing, 8.0).t4, Level::Flag);
        assert_ne!(
            evaluate_before_attempt(&missing, 8.0).overall(),
            Level::Pause
        );
    }

    #[test]
    fn prompt_budget_t5_flags_at_60_minutes() {
        let at = |m| vec![att(Verdict::Failed, None, 1.0, m)];
        assert_eq!(evaluate_before_attempt(&at(59), 8.0).t5, Level::None);
        assert_eq!(evaluate_before_attempt(&at(60), 8.0).t5, Level::Flag);
        let eighty = evaluate_before_attempt(&at(80), 8.0);
        assert_eq!(eighty.t5, Level::Flag);
        assert_eq!(
            (eighty.t1, eighty.t2, eighty.t4),
            (Level::None, Level::None, Level::None)
        );
        assert_eq!(evaluate_before_attempt(&at(120), 8.0).t5, Level::Pause);
    }

    #[test]
    fn cut_a_flags_t1_without_pausing() {
        let a = vec![
            rej("timing race in the watcher test", 2.5, 10),
            rej("waits on the wrong check", 2.5, 10),
            rej("duplicate push triggered runs", 2.5, 10),
        ];
        let before3 = evaluate_before_attempt(&a[..2], 20.0);
        assert_eq!(before3.t1, Level::Flag);
        let before4 = evaluate_before_attempt(&a, 20.0);
        assert_eq!(before4.t1, Level::Flag);
        assert_eq!(before3.t4, Level::None);
        assert_eq!(before4.t4, Level::None);
        assert_eq!(before3.overall(), Level::Flag);
        assert_eq!(before4.overall(), Level::Flag);
    }

    #[test]
    fn t3_forecast_ratios() {
        let spend = |d| vec![att(Verdict::Failed, None, d, 1)];
        assert_eq!(t3_cost(&spend(1.9), 2.0), Level::None);
        assert_eq!(t3_cost(&spend(2.0), 2.0), Level::Flag);
        assert_eq!(t3_cost(&spend(3.0), 2.0), Level::Pause);
        assert_eq!(t3_cost(&spend(5.5), 100.0), Level::Flag);
    }

    #[test]
    fn t2_growth_and_pause_with_t1() {
        let mut a = vec![rej("a", 1.0, 5), rej("b", 1.0, 5)];
        a[0].plan_size = Some(4);
        a[1].plan_size = Some(6);
        assert_eq!(t2_plan_growth(&a), Level::None); // exactly 1.5 times is not over
        a[1].plan_size = Some(7);
        assert_eq!(t2_plan_growth(&a), Level::Flag);
        assert_eq!(evaluate_before_attempt(&a, 20.0).t2, Level::Pause); // T1 also holds
        assert_eq!(evaluate_before_attempt(&a[..1], 20.0).t2, Level::None);
        let mut rounds = vec![rej("a", 1.0, 5)];
        rounds[0].planning_round = 3;
        assert_eq!(t2_plan_growth(&rounds), Level::Flag);
    }

    #[test]
    fn the_shares_of_the_forecast_make_one() {
        assert!((PRODUCER_SHARE + VERIFIER_SHARE - 1.0).abs() < 1e-12);
    }

    /// The verdict the guard gives a job that closed failed with `reason`.
    fn failed_with(reason: &str) -> Verdict {
        verdict_of(JobState::Closed(OutcomeState::Failed), Some(reason)).unwrap()
    }

    /// The closing reasons of a producer that stopped on an expired login, built three ways: as the producer
    /// stage writes it now, as an earlier record stored it (no environment prefix), and as a literal copy of
    /// that stored text. The detail goes through the real stream parser and formatter.
    fn expired_login_reasons() -> Vec<String> {
        use crate::disk::{EXPIRED_LOGIN_TEXT, driver_detail, producer_stop_reason, result_line};
        let detail = driver_detail(&result_line("success", EXPIRED_LOGIN_TEXT));
        vec![
            producer_stop_reason(&detail),
            format!("the producer did not complete: {detail}"),
            "the producer did not complete: success: Failed to authenticate. API Error: 401 {\"type\":\"error\",\"error\":{\"type\":\"authentication_error\",\"message\":\"OAuth access token has expired.\"}}".to_string(),
        ]
    }

    /// Four attempts of 40 minutes and 60 cents each, every one closed with `reason`.
    fn four_attempts_closed_with(reason: &str) -> Vec<AttemptRecord> {
        (0..4)
            .map(|_| att(failed_with(reason), None, 0.6, 40))
            .collect()
    }

    #[test]
    fn a_line_of_expired_login_stops_raises_no_signal() {
        for reason in expired_login_reasons() {
            let attempts = four_attempts_closed_with(&reason);
            assert!(
                attempts.iter().all(|a| a.verdict == Verdict::Environment),
                "{reason}"
            );
            let signals = evaluate_before_attempt(&attempts, 2.0);
            assert_eq!(signals.t1, Level::None, "{reason}");
            assert_eq!(signals.t3, Level::None, "{reason}");
            assert_eq!(signals.t5, Level::None, "{reason}");
            assert_eq!(signals.overall(), Level::None, "{reason}");
        }
    }

    #[test]
    fn the_same_line_closed_for_a_failure_of_the_work_does_raise_signals() {
        // Controls: with a reason that is not an environment stop, the same attempts trip T1, T3 and T5.
        let max_turns = format!(
            "the producer did not complete: {}",
            crate::disk::driver_detail(&crate::disk::result_line(
                "error_max_turns",
                crate::disk::EXPIRED_LOGIN_TEXT
            ))
        );
        for reason in ["the producer returned no valid report", max_turns.as_str()] {
            let attempts = four_attempts_closed_with(reason);
            assert!(
                attempts.iter().all(|a| a.verdict == Verdict::Failed),
                "{reason}"
            );
            let signals = evaluate_before_attempt(&attempts, 2.0);
            assert_eq!(signals.t1, Level::Flag, "{reason}");
            assert_eq!(signals.t3, Level::Flag, "{reason}");
            assert_eq!(signals.t5, Level::Pause, "{reason}");
        }
    }

    #[test]
    fn every_kind_of_environment_stop_is_an_environment_attempt() {
        use crate::disk::{LowDisk, check_timeout_reason, producer_stop_reason};
        let job = JobId::parse("20261026-001000-aaa1").unwrap();
        let mut reasons = expired_login_reasons();
        reasons.extend([
            LowDisk {
                free: 3_000_000_000,
                required: 10_000_000_000,
            }
            .text(),
            check_timeout_reason("test", 1805, 1800, &job),
            producer_stop_reason("success: API Error: 429 {\"type\":\"error\"}"),
            "environment: something recorded later".to_string(),
        ]);
        for reason in &reasons {
            assert_eq!(failed_with(reason), Verdict::Environment, "{reason}");
            let attempts = four_attempts_closed_with(reason);
            assert_eq!(
                evaluate_before_attempt(&attempts, 2.0).overall(),
                Level::None,
                "{reason}"
            );
        }
        for reason in [
            "the producer returned no valid report",
            "admission refused the change: x",
            "the verifier returned no valid verdict",
        ] {
            assert_eq!(failed_with(reason), Verdict::Failed, "{reason}");
        }
    }

    #[test]
    fn only_a_job_closed_failed_is_an_environment_attempt() {
        // A producer that timed out keeps its work and shares the producer's reason text; it is spend of the
        // work, even when its detail has the form of an API error.
        let reason = &expired_login_reasons()[1];
        assert_eq!(
            verdict_of(JobState::Closed(OutcomeState::TimedOut), Some(reason)),
            Some(Verdict::Failed)
        );
        assert_eq!(
            verdict_of(JobState::Closed(OutcomeState::Rejected), Some(reason)),
            Some(Verdict::Rejected)
        );
        assert_eq!(
            verdict_of(JobState::Closed(OutcomeState::Landed), Some(reason)),
            Some(Verdict::Accepted)
        );
        assert_eq!(
            verdict_of(JobState::Closed(OutcomeState::Failed), None),
            Some(Verdict::Failed)
        );
        assert_eq!(verdict_of(JobState::Dispatched, Some(reason)), None);
    }

    #[test]
    fn environment_stop_is_not_counted_but_a_rejection_is() {
        // Two attempts, each long and costly, with growing plans and a late planning round: counted as work, they
        // would trip T1, T2 (paused with T1) and T5 (240 minutes).
        let line = |verdict: Verdict| {
            let mut a = vec![
                att(verdict, Some("the file is plain text"), 0.1, 120),
                att(verdict, Some("the file is plain text"), 0.1, 120),
            ];
            a[0].plan_size = Some(4);
            a[1].plan_size = Some(9);
            a[1].planning_round = T2_PLANNING_ROUND;
            a
        };
        let env = evaluate_before_attempt(&line(Verdict::Environment), 50.0);
        assert_eq!(
            (env.t1, env.t2, env.t3, env.t4, env.t5),
            (
                Level::None,
                Level::None,
                Level::None,
                Level::None,
                Level::None
            )
        );
        assert_eq!(env.overall(), Level::None);

        let rejected = evaluate_before_attempt(&line(Verdict::Rejected), 50.0);
        assert_eq!(rejected.t1, Level::Flag);
        assert_eq!(rejected.t2, Level::Pause);
        assert_eq!(rejected.t4, Level::Pause);
        assert_eq!(rejected.t5, Level::Pause);
        assert_eq!(rejected.overall(), Level::Pause);

        // One environment stop between two real attempts is not an attempt: two counted attempts, so no T1.
        let mixed = vec![
            rej("a b", 0.1, 5),
            att(Verdict::Environment, None, 0.1, 5),
            rej("c d", 0.1, 5),
        ];
        assert_eq!(t1_retries(&mixed[..2]), Level::None);
        assert_eq!(t1_retries(&mixed), Level::Flag);
    }

    #[test]
    fn resumed_retry_is_judged_on_remaining_stages() {
        let p90 = 1.0;
        let resumed = |cost| AttemptRecord {
            resumed: true,
            ..att(Verdict::Failed, None, cost, 1)
        };
        let stopped = att(Verdict::Environment, None, 1.4, 30);

        // The stopped attempt's work was kept: its spend is not counted, and the forecast is the verifier's share.
        let line = vec![stopped.clone(), resumed(0.1)];
        let (spend, forecast) = t3_basis(&line, p90);
        assert!((spend - 0.1).abs() < 1e-12 && (forecast - p90 * VERIFIER_SHARE).abs() < 1e-12);
        assert_eq!(t3_cost(&line, p90), Level::None);

        // Control: the same costs where the retry did not resume the candidate redid the work, so the stopped
        // attempt's spend was lost; 1.5 against 1.0 pauses as a fresh job does.
        let redone = vec![stopped.clone(), att(Verdict::Failed, None, 0.1, 1)];
        let (spend, forecast) = t3_basis(&redone, p90);
        assert!((spend - 1.5).abs() < 1e-12 && (forecast - p90).abs() < 1e-12);
        assert_eq!(t3_cost(&redone, p90), Level::Pause);

        // T3 stays active for a resumed retry: spend over 1.5 times the remaining-stage forecast pauses, and
        // at the remaining-stage forecast it flags.
        let over = vec![stopped.clone(), resumed(0.5)];
        assert_eq!(t3_cost(&over, p90), Level::Pause);
        let at = vec![stopped.clone(), resumed(0.35)];
        assert_eq!(t3_cost(&at, p90), Level::Flag);
        // The same 0.5 spent by a non-resuming retry is below the whole-job limit and does not pause.
        let fresh = vec![att(Verdict::Failed, None, 0.5, 1)];
        assert_eq!(t3_cost(&fresh, p90), Level::None);

        // A stopped attempt that is last keeps its work: its spend is dropped and the forecast is the whole p90.
        let last = vec![att(Verdict::Environment, None, 1.6, 30)];
        assert_eq!(t3_basis(&last, p90), (0.0, p90));
        assert_eq!(evaluate_before_attempt(&last, p90).overall(), Level::None);
        // The same spend as a failure of the work pauses.
        let failed = vec![att(Verdict::Failed, None, 1.6, 30)];
        assert_eq!(t3_cost(&failed, p90), Level::Pause);
    }

    #[test]
    fn accepted_line_has_no_signals() {
        let a = vec![rej("x", 9.0, 200), att(Verdict::Accepted, None, 9.0, 200)];
        assert_eq!(evaluate_before_attempt(&a, 1.0).overall(), Level::None);
    }
}
