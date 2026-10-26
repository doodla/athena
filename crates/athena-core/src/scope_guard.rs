//! Scope guard, build job 1: line-of-work grouping and thrash signals T1 to T5.
//!
//! This is the pure part of the design in `docs/design/scope-guard.md` (section 5 for the signals, section 13
//! job 1 for the scope, decision D127). Everything here is a function over plain input values. There is no file,
//! process, network or clock access: elapsed agent time, spend and the per-job-kind forecast p90 are all passed
//! in by the caller (the later running-line watcher, job 5).
//!
//! Every threshold is an **estimate, to be tuned** (section 5); the basis for each is in that section's table.

use std::collections::HashMap;

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

/// How a finished attempt ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// The candidate was accepted: the line is done.
    Accepted,
    /// The verifier rejected the candidate.
    Rejected,
    /// The attempt ended without a verdict (for example a failed run).
    Failed,
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
}

/// One request and its finished attempts.
#[derive(Debug, Clone, PartialEq)]
pub struct RequestRecord {
    /// Request id.
    pub id: String,
    /// The request this one retries, if any.
    pub retry_of: Option<String>,
    /// Requests whose text this one quotes.
    pub quotes: Vec<String>,
    /// Finished attempts of this request, oldest first.
    pub attempts: Vec<AttemptRecord>,
}

/// Group requests into lines of work (section 5): a request, its retries and any request that quotes it.
/// Links to ids not in `requests` are ignored. Returns indices into `requests`; each line is in input order and
/// lines are ordered by their first member.
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
        let links = r.retry_of.iter().chain(r.quotes.iter());
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

/// T1, repeated retries: flag when the next attempt would be the `T1_ATTEMPT`th on the line.
pub fn t1_retries(attempts: &[AttemptRecord]) -> Level {
    if !done(attempts) && attempts.len() as u32 + 1 >= T1_ATTEMPT {
        Level::Flag
    } else {
        Level::None
    }
}

/// T2, growing plan: flag when the latest known plan is over `T2_RATIO` times the first, or a planning round
/// reached `T2_PLANNING_ROUND`. The "pause if T1 or T4 also holds" rule is applied in [`evaluate_before_attempt`].
pub fn t2_plan_growth(attempts: &[AttemptRecord]) -> Level {
    if done(attempts) {
        return Level::None;
    }
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

/// T3, cost against forecast: pause at `T3_PAUSE_RATIO` times `forecast_p90_dollars`, flag at `T3_FLAG_RATIO`
/// times it, and flag over `T3_LINE_DOLLARS` regardless of forecast. A forecast of 0 or less is not used.
pub fn t3_cost(attempts: &[AttemptRecord], forecast_p90_dollars: f64) -> Level {
    if done(attempts) {
        return Level::None;
    }
    let spend: f64 = attempts.iter().map(|a| a.cost_dollars).sum();
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
/// `T5_PAUSE_MINUTES`.
pub fn t5_wall_time(attempts: &[AttemptRecord]) -> Level {
    if done(attempts) {
        return Level::None;
    }
    let minutes: u32 = attempts.iter().map(|a| a.agent_minutes).sum();
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
        }
    }

    fn rej(criterion: &str, cost: f64, minutes: u32) -> AttemptRecord {
        att(Verdict::Rejected, Some(criterion), cost, minutes)
    }

    fn req(id: &str, retry_of: Option<&str>, quotes: &[&str]) -> RequestRecord {
        RequestRecord {
            id: id.to_string(),
            retry_of: retry_of.map(str::to_string),
            quotes: quotes.iter().map(|s| s.to_string()).collect(),
            attempts: Vec::new(),
        }
    }

    #[test]
    fn grouping_joins_retries_and_quoting_requests() {
        let reqs = vec![
            req("a", None, &[]),
            req("x", None, &[]),
            req("b", Some("a"), &[]),
            req("c", None, &["b"]),
            req("d", Some("missing"), &[]),
        ];
        let lines = group_lines(&reqs);
        assert_eq!(lines, vec![vec![0, 2, 3], vec![1], vec![4]]);
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
    fn accepted_line_has_no_signals() {
        let a = vec![rej("x", 9.0, 200), att(Verdict::Accepted, None, 9.0, 200)];
        assert_eq!(evaluate_before_attempt(&a, 1.0).overall(), Level::None);
    }
}
