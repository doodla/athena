//! Per-stage durations and the main reason for each, for the newest landed jobs.
//!
//! The throughput section gives medians; this one answers "why was this stage slow" for single jobs. Every
//! figure is read from records that already exist: invocation start and end times, token counts, turns and
//! commands run, check durations and build-slot waits, the dispatch record and the landing record. A stage
//! whose time or reason has no record shows as `not recorded`, never as zero.

use super::throughput::{landed_jobs, minutes_between};
use super::*;

/// How many of the newest landed jobs the per-stage report covers.
pub const STAGE_REPORT_JOBS: usize = 5;

/// The text printed where a time or a reason has no record.
pub const NOT_RECORDED: &str = "not recorded";

/// One stage of one job: how long it took and the main reason it took that long.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StageTiming {
    /// `reading`, `planning round N`, `producer`, `checks`, `verifier`, `landing`, `queue wait` or `disk wait`.
    pub stage: String,
    /// Minutes the stage took; none when no record gives it.
    pub minutes: Option<f64>,
    /// Why it took that long (model and effort, tokens, tool calls, build and test time, retries, waits);
    /// none when no record gives a reason.
    pub reason: Option<String>,
}

/// The stages of one landed job, in pipeline order.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JobStages {
    pub job: String,
    pub stages: Vec<StageTiming>,
}

impl JobStages {
    /// The stage that took the longest, when any stage has a time.
    pub fn slowest(&self) -> Option<&StageTiming> {
        self.stages
            .iter()
            .filter(|s| s.minutes.is_some())
            .max_by(|a, b| {
                a.minutes
                    .unwrap_or(0.0)
                    .total_cmp(&b.minutes.unwrap_or(0.0))
            })
    }
}

fn group_minutes(invs: &[&Invocation]) -> f64 {
    crate::store::positive_zero(
        invs.iter()
            .map(|i| minutes_between(i.started_at, i.ended_at).max(0.0))
            .sum(),
    )
}

/// The reason line of a group of invocations: model and effort, tokens, turns and commands, retries.
fn invocation_reason(invs: &[&Invocation]) -> String {
    let mut models: Vec<String> = Vec::new();
    for i in invs {
        let m = match &i.effort {
            Some(e) => format!("{} at {e} effort", i.model),
            None => format!("{} (effort {NOT_RECORDED})", i.model),
        };
        if !models.contains(&m) {
            models.push(m);
        }
    }
    let sum = |pick: fn(&Invocation) -> f64| invs.iter().map(|i| pick(i)).sum::<f64>();
    let input = sum(|i| i.tokens.input.best_lower());
    let output = sum(|i| i.tokens.output.best_lower());
    let turns: Option<u32> = invs.iter().map(|i| i.turns).sum();
    let turns = turns.map_or_else(|| format!("turns {NOT_RECORDED}"), |t| format!("{t} turns"));
    let commands: usize = invs.iter().map(|i| i.commands_run.len()).sum();
    let mut parts = vec![
        models.join(", "),
        format!("{input:.0} tokens in, {output:.0} out"),
        turns,
        format!("{commands} commands run"),
    ];
    if invs.len() > 1 {
        let roles: BTreeSet<&str> = invs.iter().map(|i| i.role.as_str()).collect();
        if roles.len() == 1 {
            parts.push(format!("{} invocations (retries)", invs.len()));
        } else {
            parts.push(format!(
                "{} invocations ({})",
                invs.len(),
                roles.into_iter().collect::<Vec<_>>().join(" and ")
            ));
        }
    }
    let failed = invs
        .iter()
        .filter(|i| i.outcome != athena_data::records::InvocationOutcome::Success)
        .count();
    if failed > 0 {
        parts.push(format!("{failed} did not finish cleanly"));
    }
    parts.join(" · ")
}

/// The checks stage: total time, with the compile and lint checks apart from the test check, the slowest
/// check named, and the wait for a build slot.
fn checks_stage(job: &JobRecords) -> StageTiming {
    let ran: Vec<(&str, f64)> = job.verification.as_ref().map_or_else(Vec::new, |v| {
        v.checks
            .iter()
            .filter(|c| c.duration.basis() != "not run")
            .filter_map(|c| Some((c.name.as_str(), c.duration.value()?)))
            .collect()
    });
    if ran.is_empty() {
        return StageTiming {
            stage: "checks".into(),
            minutes: None,
            reason: None,
        };
    }
    let total: f64 = ran.iter().map(|(_, s)| s).sum();
    let test: f64 = ran
        .iter()
        .filter(|(n, _)| *n == "test")
        .map(|(_, s)| s)
        .sum();
    let (slowest, slowest_s) = ran
        .iter()
        .copied()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap_or(("", 0.0));
    let waits: Vec<f64> = job
        .verification
        .iter()
        .flat_map(|v| v.checks.iter())
        .filter_map(|c| c.queued_s)
        .collect();
    let wait = if waits.is_empty() {
        format!("build-slot wait {NOT_RECORDED}")
    } else {
        format!("waited {:.0}s for a build slot", waits.iter().sum::<f64>())
    };
    StageTiming {
        stage: "checks".into(),
        minutes: Some(total / 60.0),
        reason: Some(format!(
            "build and lint checks {:.1} min · test {:.1} min · slowest {slowest} {:.1} min · {} check(s) run · {wait}",
            (total - test) / 60.0,
            test / 60.0,
            slowest_s / 60.0,
            ran.len()
        )),
    }
}

/// Planning rounds: a round starts at a planner invocation and holds the critic invocations that follow it.
fn planning_rounds(job: &JobRecords) -> Vec<StageTiming> {
    let mut rounds: Vec<Vec<&Invocation>> = Vec::new();
    let mut sorted: Vec<&Invocation> = job.invocations.iter().collect();
    sorted.sort_by_key(|i| (i.started_at, i.seq));
    for i in sorted {
        match i.role {
            Role::Planner => rounds.push(vec![i]),
            Role::Critic => match rounds.last_mut() {
                Some(round) => round.push(i),
                None => rounds.push(vec![i]),
            },
            _ => {}
        }
    }
    rounds
        .iter()
        .enumerate()
        .map(|(n, round)| StageTiming {
            stage: format!("planning round {}", n + 1),
            minutes: Some(group_minutes(round)),
            reason: Some(invocation_reason(round)),
        })
        .collect()
}

fn role_stage(job: &JobRecords, stage: &str, role: Role) -> StageTiming {
    let invs: Vec<&Invocation> = job.invocations.iter().filter(|i| i.role == role).collect();
    if invs.is_empty() {
        return StageTiming {
            stage: stage.into(),
            minutes: None,
            reason: None,
        };
    }
    let mut reason = invocation_reason(&invs);
    if role == Role::Reader {
        let refused = invs
            .iter()
            .filter(|i| {
                i.reading_attempt
                    .as_ref()
                    .is_some_and(|a| a.refusal.is_some())
            })
            .count();
        if refused > 0 {
            reason.push_str(&format!(" · {refused} reading(s) refused and read again"));
        }
    }
    StageTiming {
        stage: stage.into(),
        minutes: Some(group_minutes(&invs)),
        reason: Some(reason),
    }
}

/// Queue wait: from the accepted reading to the dispatch of the job.
fn queue_wait(job: &JobRecords) -> StageTiming {
    let dispatched = job.job.as_ref().and_then(|j| j.dispatched_at);
    let accepted = job
        .job
        .as_ref()
        .and_then(|j| j.accepted_reading)
        .and_then(|n| job.reading(n))
        .map(|r| r.at);
    queue_wait_between(accepted, dispatched)
}

/// The queue-wait stage from the time the reading was accepted and the time the job was dispatched.
pub(super) fn queue_wait_between(
    accepted: Option<jiff::Timestamp>,
    dispatched: Option<jiff::Timestamp>,
) -> StageTiming {
    let minutes = match (accepted, dispatched) {
        (Some(from), Some(to)) => Some(minutes_between(from, to).max(0.0)),
        _ => None,
    };
    StageTiming {
        stage: "queue wait".into(),
        minutes,
        reason: minutes.map(|_| "from the accepted reading to dispatch".to_string()),
    }
}

/// The stages of `job`, with the landing minutes the throughput section already measures.
pub fn job_stage_timings(
    job: &JobRecords,
    landing_minutes: Option<f64>,
    landing_found: bool,
) -> Vec<StageTiming> {
    let mut out = vec![queue_wait(job), role_stage(job, "reading", Role::Reader)];
    out.extend(planning_rounds(job));
    out.push(role_stage(job, "producer", Role::Producer));
    out.push(checks_stage(job));
    out.push(role_stage(job, "verifier", Role::Verifier));
    out.push(StageTiming {
        stage: "landing".into(),
        minutes: landing_minutes,
        reason: landing_minutes.map(|_| {
            if landing_found {
                "verifier end to the landing record (merge, push, rebuild)".to_string()
            } else {
                "verifier end to the landing outcome; no landing record found".to_string()
            }
        }),
    });
    // No record keeps how long a job waited for disk space, so this stage is always unrecorded.
    out.push(StageTiming {
        stage: "disk wait".into(),
        minutes: None,
        reason: None,
    });
    out
}

/// The per-stage report of the newest `count` landed jobs, newest first.
pub fn job_stages(records: &Records, count: usize) -> Vec<JobStages> {
    landed_jobs(records)
        .iter()
        .rev()
        .take(count)
        .filter_map(|l| {
            let job = records.jobs.get(&l.id)?;
            Some(JobStages {
                job: l.id.clone(),
                stages: job_stage_timings(job, Some(l.stages.landing_and_rebuild), l.landing_found),
            })
        })
        .collect()
}

/// The lines `athena status` prints: for each job, each stage with its minutes and its reason.
pub fn stages_text(jobs: &[JobStages]) -> String {
    let mut out = String::new();
    for job in jobs {
        let slowest = job.slowest().map(|s| s.stage.clone());
        out.push_str(&format!("stages     {}\n", job.job));
        for s in &job.stages {
            let minutes = s
                .minutes
                .map_or_else(|| NOT_RECORDED.to_string(), |m| format!("{m:.1} min"));
            let reason = s.reason.as_deref().unwrap_or(NOT_RECORDED);
            let mark = if slowest.as_deref() == Some(s.stage.as_str()) {
                " (slowest)"
            } else {
                ""
            };
            out.push_str(&format!(
                "           {:<18} {minutes}{mark} · reason: {reason}\n",
                s.stage
            ));
        }
    }
    out.push_str("           basis: times from record timestamps and measured check durations; reasons from invocation, check and landing records; 'not recorded' where no record holds the figure");
    out
}
