//! Roles added or retired by observed need (requirement OLY-9-11): which roles ran in the newest jobs that
//! did model work, and which defined roles did not run at all.
//!
//! Usage is read ONLY through [`Records::all_invocations`]. A job's own `invocations` list is a partial
//! source: the steward, audience and auditor run outside any job, so their runs are kept apart. Those three
//! also run on demand (`athena audit`, the steward's cadence, the audience's rating), so a window of recent
//! jobs cannot show whether they are needed. They are reported as "not measured" with the runs seen since the
//! window began, and are never retire candidates ([`ON_DEMAND_ROLES`]).
//!
//! The window is the newest [`WINDOW_JOBS`] jobs that did model work: closed as landed, rejected, failed or
//! timed out, and with at least one invocation of a job role. Jobs that never ran a model (dropped, stale,
//! abandoned, declined, the bootstrap's closed outcome) are left out of the window and of every count.
//! `athena status` prints the `role need` line only when a role is a retire candidate; the measure advises and
//! refuses nothing.

use crate::store::{JobRecords, JobState, Records};
use athena_data::records::{OutcomeState, Role};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// Jobs that did model work in the window; fewer than this and nothing is measured.
pub const WINDOW_JOBS: usize = 8;

/// Roles that run on demand or beside jobs. Their runs are not in any job's invocation list and a recent-jobs
/// window cannot show their need, so they are reported as not measured and never as retire candidates.
pub const ON_DEMAND_ROLES: &[Role] = &[Role::Steward, Role::Audience, Role::Auditor];

fn on_demand(role: Role) -> bool {
    ON_DEMAND_ROLES.contains(&role)
}

/// The roles seen and not seen in the newest jobs that did model work.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RoleNeed {
    /// Jobs in the window (always [`WINDOW_JOBS`]).
    pub window_jobs: usize,
    /// Of those, the jobs that closed failed, rejected or timed out.
    pub failed_jobs: usize,
    /// Roles measured, with their runs in the window's jobs.
    pub used: Vec<(String, usize)>,
    /// Roles defined or built in that did not run in the window's jobs, in role order.
    pub retire_candidates: Vec<String>,
    /// On-demand roles, with their runs (from every source) since the window's oldest invocation.
    pub not_measured: Vec<(String, usize)>,
}

/// Whether a job counts as work for the window: it closed after a model ran. `ran` holds the ids of jobs with
/// an invocation of a job role.
fn counts_as_work(job: &JobRecords, ran: &BTreeSet<&str>) -> bool {
    matches!(
        job.state(),
        JobState::Closed(
            OutcomeState::Landed
                | OutcomeState::Rejected
                | OutcomeState::Failed
                | OutcomeState::TimedOut
        )
    ) && ran.contains(job.id().as_str())
}

impl RoleNeed {
    /// Measures `records`. None while fewer than [`WINDOW_JOBS`] jobs did model work.
    pub fn compute(records: &Records) -> Option<RoleNeed> {
        let ran: BTreeSet<&str> = records
            .all_invocations()
            .filter(|i| !on_demand(i.role))
            .map(|i| i.job.as_str())
            .collect();
        let window: Vec<&JobRecords> = records
            .request_jobs()
            .into_iter()
            .filter(|j| counts_as_work(j, &ran))
            .take(WINDOW_JOBS)
            .collect();
        if window.len() < WINDOW_JOBS {
            return None;
        }
        let in_window: BTreeSet<&str> = window.iter().map(|j| j.id().as_str()).collect();
        let failed_jobs = window
            .iter()
            .filter(|j| {
                matches!(
                    j.state(),
                    JobState::Closed(
                        OutcomeState::Failed | OutcomeState::Rejected | OutcomeState::TimedOut
                    )
                )
            })
            .count();
        let window_start = records
            .all_invocations()
            .filter(|i| in_window.contains(i.job.as_str()))
            .map(|i| i.started_at)
            .min()?;

        let mut runs: BTreeMap<Role, usize> = BTreeMap::new();
        let mut on_demand_runs: BTreeMap<Role, usize> = BTreeMap::new();
        for i in records.all_invocations() {
            if on_demand(i.role) {
                if i.started_at >= window_start {
                    *on_demand_runs.entry(i.role).or_default() += 1;
                }
            } else if in_window.contains(i.job.as_str()) {
                *runs.entry(i.role).or_default() += 1;
            }
        }

        let all_roles = [
            Role::Reader,
            Role::Planner,
            Role::Critic,
            Role::Producer,
            Role::Verifier,
        ];
        let mut used = Vec::new();
        let mut retire_candidates = Vec::new();
        for role in all_roles {
            let n = runs.get(&role).copied().unwrap_or(0);
            if n > 0 {
                used.push((role.as_str().to_string(), n));
            } else {
                retire_candidates.push(role.as_str().to_string());
            }
        }
        // A role added by name has no invocation role; its use is the work it queued since the window began.
        let custom: BTreeSet<&str> = records
            .roles
            .iter()
            .map(|r| r.name.as_str())
            .filter(|name| Role::parse(name).is_none())
            .collect();
        for name in custom {
            let n = records
                .work_items
                .iter()
                .filter(|w| w.role == name && w.at >= window_start)
                .count();
            if n > 0 {
                used.push((name.to_string(), n));
            } else {
                retire_candidates.push(name.to_string());
            }
        }
        let not_measured = ON_DEMAND_ROLES
            .iter()
            .map(|r| {
                (
                    r.as_str().to_string(),
                    on_demand_runs.get(r).copied().unwrap_or(0),
                )
            })
            .collect();
        Some(RoleNeed {
            window_jobs: window.len(),
            failed_jobs,
            used,
            retire_candidates,
            not_measured,
        })
    }

    /// The text `athena status` prints after the `role need` label; None when no role is a retire candidate,
    /// so a clean result prints nothing.
    pub fn line(&self) -> Option<String> {
        if self.retire_candidates.is_empty() {
            return None;
        }
        let not_measured = self
            .not_measured
            .iter()
            .map(|(role, n)| format!("{role} ({n} runs since the window began)"))
            .collect::<Vec<_>>()
            .join(", ");
        Some(format!(
            "retire candidates: {} (no runs in the newest {} jobs that ran a model; {} of them failed); not measured, run on demand: {not_measured}",
            self.retire_candidates.join(", "),
            self.window_jobs,
            self.failed_jobs,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::planning_cost_tests::invocation;
    use athena_data::JobId;
    use athena_data::records::{Outcome, Reference, WorkItem, WorkItemKind};
    use athena_data::roles::{Ladder, RoleDefinition};

    fn id(n: u32) -> JobId {
        JobId::parse(&format!("202611{n:02}-120000-k{n:03}")).unwrap()
    }

    fn at(secs: i64) -> jiff::Timestamp {
        jiff::Timestamp::from_second(1_800_000_000 + secs).unwrap()
    }

    /// A job number `n` closed in `state`, with one run of each role in `roles`, all started at `n` seconds.
    fn job(n: u32, state: OutcomeState, roles: &[Role]) -> JobRecords {
        let job = id(n);
        let invocations = roles
            .iter()
            .enumerate()
            .map(|(seq, role)| {
                let mut i = invocation(&job, seq as u32 + 1, *role, 1.0);
                i.started_at = at(i64::from(n));
                i
            })
            .collect();
        let mut j = JobRecords {
            id: Some(job.clone()),
            invocations,
            ..JobRecords::default()
        };
        j.outcome = Some(Outcome {
            job,
            state,
            reason: "r".into(),
            at: at(i64::from(n)),
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
        let mut r = Records::default();
        for j in jobs {
            r.jobs.insert(j.id().as_str().to_string(), j);
        }
        r
    }

    fn out_of_job_run(n: u32, role: Role, secs: i64) -> athena_data::records::Invocation {
        let mut i = invocation(&id(n), 1, role, 1.0);
        i.started_at = at(secs);
        i
    }

    const JOB_ROLES: [Role; 3] = [Role::Reader, Role::Producer, Role::Verifier];

    /// Ten closed jobs holding only reader, producer and verifier runs; recent steward, audience and auditor
    /// runs kept outside them, as the real records keep them.
    fn known_active() -> Records {
        known_active_with(&JOB_ROLES)
    }

    /// As [`known_active`], with the newest job holding `newest_roles`.
    fn known_active_with(newest_roles: &[Role]) -> Records {
        let states = [
            OutcomeState::Landed,
            OutcomeState::Rejected,
            OutcomeState::Landed,
            OutcomeState::Failed,
            OutcomeState::Landed,
            OutcomeState::TimedOut,
            OutcomeState::Landed,
            OutcomeState::Landed,
            OutcomeState::Landed,
            OutcomeState::Landed,
        ];
        let jobs = states
            .iter()
            .enumerate()
            .map(|(k, s)| {
                let roles = if k == 9 { newest_roles } else { &JOB_ROLES };
                job(k as u32 + 1, *s, roles)
            })
            .collect();
        let mut r = records(jobs);
        r.steward_runs.push(out_of_job_run(20, Role::Steward, 50));
        r.audience_runs.push(out_of_job_run(21, Role::Audience, 51));
        r.auditor_runs.push(out_of_job_run(22, Role::Auditor, 52));
        r
    }

    #[test]
    fn known_active_roles_are_never_retire_candidates_and_an_unused_role_is() {
        let mut r = known_active();
        r.roles.push(RoleDefinition {
            name: "scribe".into(),
            version: 1,
            at: at(0),
            goal: "Keep notes.".into(),
            prompt: "Keep notes.".into(),
            cadence_tasks: 5,
            highest_rung: Ladder::Implementation,
        });
        let need = RoleNeed::compute(&r).expect("ten closed jobs ran a model");
        for known in ["steward", "audience", "auditor"] {
            assert!(
                !need.retire_candidates.iter().any(|c| c == known),
                "{known} flagged: {need:?}"
            );
            assert!(
                need.not_measured.iter().any(|(role, _)| role == known),
                "{known} not reported as not measured: {need:?}"
            );
        }
        // Their runs are in no job's list, so counting them proves the measure read every source.
        assert_eq!(
            need.not_measured,
            vec![
                ("steward".to_string(), 1),
                ("audience".to_string(), 1),
                ("auditor".to_string(), 1)
            ]
        );
        // Roles with no runs in any source are flagged, whether built in or added by name.
        assert_eq!(
            need.retire_candidates,
            vec!["planner", "critic", "scribe"],
            "{need:?}"
        );
        let line = need.line().expect("candidates exist");
        assert!(!line.contains("retire candidates: steward"), "{line}");
        assert!(
            line.contains("not measured, run on demand: steward"),
            "{line}"
        );
    }

    #[test]
    fn a_role_that_queued_work_in_the_window_is_in_use() {
        let mut r = known_active();
        r.roles.push(RoleDefinition {
            name: "scribe".into(),
            version: 1,
            at: at(0),
            goal: "Keep notes.".into(),
            prompt: "Keep notes.".into(),
            cadence_tasks: 5,
            highest_rung: Ladder::Implementation,
        });
        r.work_items.push(WorkItem {
            id: athena_data::RecordId::parse("20261111-120000-ab12").unwrap(),
            at: at(30),
            role: "scribe".into(),
            kind: WorkItemKind::BlindSpot,
            key: "k".into(),
            title: "t".into(),
            rung: Ladder::Implementation,
            source: Reference::path("docs/x.md"),
            evidence: vec![],
            brief: None,
        });
        let need = RoleNeed::compute(&r).unwrap();
        assert!(!need.retire_candidates.contains(&"scribe".to_string()));
        assert!(need.used.iter().any(|(role, _)| role == "scribe"));
    }

    #[test]
    fn the_measure_yields_both_outcomes_on_a_mixed_fixture() {
        // Planner and critic ran in the window, so every built-in role is in use and nothing is flagged.
        let r = known_active_with(&[
            Role::Reader,
            Role::Planner,
            Role::Critic,
            Role::Producer,
            Role::Verifier,
        ]);
        let some = RoleNeed::compute(&r).unwrap();
        assert!(some.retire_candidates.is_empty(), "{some:?}");
        assert_eq!(some.line(), None);
        assert_eq!(some.used.len(), 5);

        // Remove them again: the same records now yield flagged roles, so the measure is neither all-true
        // nor all-false.
        let flagged = RoleNeed::compute(&known_active()).unwrap();
        assert_eq!(flagged.retire_candidates, vec!["planner", "critic"]);
        assert!(flagged.used.iter().any(|(role, _)| role == "producer"));
        assert!(flagged.line().is_some());
    }

    #[test]
    fn failed_rejected_and_timed_out_jobs_count_as_failed() {
        let need = RoleNeed::compute(&known_active()).unwrap();
        assert_eq!(need.window_jobs, WINDOW_JOBS);
        // Of the newest eight (jobs 3 to 10) the failed one is job 4 and the timed-out one is job 6; job 2,
        // rejected, is older than the window.
        assert_eq!(need.failed_jobs, 2, "{need:?}");
        assert!(need.line().unwrap().contains("2 of them failed"));
    }

    #[test]
    fn jobs_that_never_ran_a_model_are_left_out_of_the_window() {
        let mut r = known_active();
        // Newer jobs: dropped, stale, abandoned, declined and closed, each holding a planner run that must
        // not count; and a landed job with no invocation at all.
        for (k, state) in [
            OutcomeState::Dropped,
            OutcomeState::Stale,
            OutcomeState::Abandoned,
            OutcomeState::Declined,
            OutcomeState::Closed,
        ]
        .into_iter()
        .enumerate()
        {
            let j = job(
                11 + k as u32,
                state,
                &[Role::Reader, Role::Planner, Role::Critic],
            );
            r.jobs.insert(j.id().as_str().to_string(), j);
        }
        let empty = job(17, OutcomeState::Landed, &[]);
        r.jobs.insert(empty.id().as_str().to_string(), empty);
        let need = RoleNeed::compute(&r).unwrap();
        assert_eq!(need.window_jobs, WINDOW_JOBS);
        assert_eq!(
            need.retire_candidates,
            vec!["planner", "critic"],
            "{need:?}"
        );
        assert_eq!(need.failed_jobs, 2, "{need:?}");
    }

    #[test]
    fn too_few_jobs_measure_nothing() {
        let jobs = (1..WINDOW_JOBS as u32)
            .map(|n| job(n, OutcomeState::Landed, &JOB_ROLES))
            .collect();
        assert_eq!(RoleNeed::compute(&records(jobs)), None);
    }

    #[test]
    fn this_module_reads_usage_only_through_all_invocations() {
        let source = include_str!("role_need.rs");
        let code = source.split("#[cfg(test)]").next().unwrap();
        let forbidden = [
            concat!(".", "invocations"),
            "steward_runs",
            "audience_runs",
            "auditor_runs",
        ];
        for word in forbidden {
            assert!(
                !code
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("//"))
                    .any(|l| l.contains(word)),
                "role_need.rs reads {word} directly; read Records::all_invocations"
            );
        }
    }
}
