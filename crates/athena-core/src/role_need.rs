//! Roles added or retired by observed need (requirement OLY-9-11): which roles have not run lately, and how
//! each class of work fared over its newest closed jobs.
//!
//! Root cause this module is built against: an earlier usage measure counted only the per-job invocation
//! lists, so the steward, the audience and the auditor, whose runs are kept apart from the jobs in
//! [`Records`], always looked unused and would have been advised for retirement while they were running.
//! The measure here reads [`Records::all_invocations`], the one accessor over every run source, and never
//! `JobRecords::invocations` alone. A role is a retire candidate only when no run from any source began since
//! the window opened, and only once the history holds a full window of closed jobs.
//!
//! `athena status` prints the `role need` line only when a candidate exists. The advice is a line to read; no
//! role is created or retired by it.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::store::{JobState, Records};
use athena_data::records::{OutcomeState, Role};

/// The newest closed jobs (overall, and within each class of work) that the measure looks at.
pub const WINDOW: usize = 8;

/// Every role a run can record.
const ROLES: [Role; 8] = [
    Role::Reader,
    Role::Producer,
    Role::Verifier,
    Role::Planner,
    Role::Critic,
    Role::Steward,
    Role::Audience,
    Role::Auditor,
];

/// Whether a closed job ended badly. A declined job and the bootstrap's own `closed` outcome are not counted
/// as work done at all ([`counts_as_work`]).
fn is_failed(state: OutcomeState) -> bool {
    matches!(
        state,
        OutcomeState::Failed | OutcomeState::Rejected | OutcomeState::TimedOut
    )
}

/// Whether a terminal outcome is a job that was worked: not declined, and not the bootstrap's `closed`.
fn counts_as_work(state: OutcomeState) -> bool {
    !matches!(state, OutcomeState::Declined | OutcomeState::Closed)
}

/// One role's runs inside the window.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoleUse {
    pub role: String,
    /// Runs that began inside the window, from every run source.
    pub runs: usize,
    pub retire_candidate: bool,
}

/// How a class of work fared over its newest closed jobs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClassOutcomes {
    pub class: String,
    pub closed: usize,
    pub failed: usize,
}

/// The measure: every role's use and every class's outcomes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoleNeed {
    /// Closed jobs the window holds.
    pub window_jobs: usize,
    pub roles: Vec<RoleUse>,
    pub classes: Vec<ClassOutcomes>,
}

impl RoleNeed {
    /// The roles with no run inside the window.
    pub fn candidates(&self) -> Vec<&str> {
        self.roles
            .iter()
            .filter(|r| r.retire_candidate)
            .map(|r| r.role.as_str())
            .collect()
    }

    /// What `athena status` prints after the `role need` label.
    pub fn detail(&self) -> String {
        let classes: Vec<String> = self
            .classes
            .iter()
            .map(|c| format!("{} {}/{} failed", c.class, c.failed, c.closed))
            .collect();
        let mut text = format!(
            "retire candidates: {} (no run began in the newest {} closed jobs)",
            self.candidates().join(", "),
            self.window_jobs
        );
        if !classes.is_empty() {
            text.push_str(" · ");
            text.push_str(&classes.join(", "));
        }
        text
    }

    /// Reads the records. None while fewer than [`WINDOW`] jobs are closed, so a thin history never retires a
    /// role, and while the window's jobs recorded no run to date it by.
    pub fn compute(records: &Records) -> Option<RoleNeed> {
        // Job ids start with the time the job was made, so the map's order is oldest first.
        let closed: Vec<(&str, &crate::store::JobRecords, OutcomeState)> = records
            .jobs
            .iter()
            .filter_map(|(id, job)| match job.state() {
                JobState::Closed(state) if counts_as_work(state) => Some((id.as_str(), job, state)),
                _ => None,
            })
            .collect();
        if closed.len() < WINDOW {
            return None;
        }
        let window = &closed[closed.len() - WINDOW..];
        let opened = window
            .iter()
            .flat_map(|(_, job, _)| job.invocations.iter())
            .map(|i| i.started_at)
            .min()?;

        let roles = ROLES
            .iter()
            .map(|role| {
                let runs = records
                    .all_invocations()
                    .filter(|i| i.role == *role && i.started_at >= opened)
                    .count();
                RoleUse {
                    role: role.as_str().to_string(),
                    runs,
                    retire_candidate: runs == 0,
                }
            })
            .collect();

        let mut by_class: BTreeMap<&str, Vec<OutcomeState>> = BTreeMap::new();
        for (_, job, state) in &closed {
            if let Some(class) = job.class() {
                by_class.entry(class).or_default().push(*state);
            }
        }
        let classes = by_class
            .into_iter()
            .map(|(class, states)| {
                let newest = &states[states.len().saturating_sub(WINDOW)..];
                ClassOutcomes {
                    class: class.to_string(),
                    closed: newest.len(),
                    failed: newest.iter().filter(|s| is_failed(**s)).count(),
                }
            })
            .collect();

        Some(RoleNeed {
            window_jobs: WINDOW,
            roles,
            classes,
        })
    }

    /// The measure for `athena status`: present only when it names a candidate.
    pub fn shown(records: &Records) -> Option<RoleNeed> {
        Self::compute(records).filter(|n| !n.candidates().is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::planning_cost_tests::invocation;
    use crate::status::records_worktree_tests::job_record;
    use crate::store::JobRecords;
    use athena_data::JobId;
    use athena_data::records::{AnyRecord, Invocation, Outcome};

    fn id(n: u32) -> String {
        format!("20261001-10{:02}{:02}-k7q2", n / 60, n % 60)
    }

    fn run(job: &str, seq: u32, role: Role, at: i64) -> Invocation {
        let mut i = invocation(&JobId::parse(job).unwrap(), seq, role, 0.1);
        i.started_at = jiff::Timestamp::from_second(at).unwrap();
        i
    }

    /// A closed job `n` of `class` that ended `state`, with reader, producer and verifier runs at `1000 + n`.
    fn closed_job(n: u32, class: &str, state: OutcomeState) -> JobRecords {
        let job = id(n);
        let AnyRecord::Job(record) = job_record(&job, class) else {
            unreachable!()
        };
        JobRecords {
            id: Some(JobId::parse(&job).unwrap()),
            job: Some(*record),
            invocations: [Role::Reader, Role::Producer, Role::Verifier]
                .into_iter()
                .enumerate()
                .map(|(s, r)| run(&job, s as u32, r, 1000 + i64::from(n)))
                .collect(),
            outcome: Some(Outcome {
                job: JobId::parse(&job).unwrap(),
                state,
                reason: "ended".into(),
                at: jiff::Timestamp::from_second(2000).unwrap(),
                totals: None,
                branch: None,
                base_commit: None,
                candidate_commit: None,
                after_blocked: false,
                through_commit: None,
            }),
            ..Default::default()
        }
    }

    fn with_jobs(jobs: Vec<JobRecords>) -> Records {
        let mut records = Records::default();
        for j in jobs {
            records.jobs.insert(j.id().as_str().to_string(), j);
        }
        records
    }

    fn named(need: &RoleNeed) -> Vec<&str> {
        need.candidates()
    }

    /// Ten closed jobs that only reader, producer and verifier ran in; the steward, audience and auditor ran
    /// recently, outside every job.
    fn fixture() -> Records {
        let mut records = with_jobs(
            (0..10)
                .map(|n| closed_job(n, "code", OutcomeState::Landed))
                .collect(),
        );
        let outside = id(99);
        records
            .steward_runs
            .push(run(&outside, 0, Role::Steward, 1009));
        records
            .audience_runs
            .push(run(&outside, 0, Role::Audience, 1009));
        records
            .auditor_runs
            .push(run(&outside, 0, Role::Auditor, 1009));
        records
    }

    #[test]
    fn runs_kept_outside_the_jobs_are_not_retire_candidates() {
        let records = fixture();
        // The fixture is a real-shaped hazard: job invocations alone hold none of the three roles.
        let job_only: Vec<Role> = records
            .jobs
            .values()
            .flat_map(|j| j.invocations.iter().map(|i| i.role))
            .collect();
        for role in [Role::Steward, Role::Audience, Role::Auditor] {
            assert!(!job_only.contains(&role), "{role:?} must be outside jobs");
        }
        let need = RoleNeed::compute(&records).expect("ten closed jobs fill the window");
        let candidates = named(&need);
        for active in [
            "steward", "audience", "auditor", "reader", "producer", "verifier",
        ] {
            assert!(
                !candidates.contains(&active),
                "{active} is running and must not be a candidate: {candidates:?}"
            );
        }
        // Roles that never ran are the candidates.
        assert_eq!(candidates, ["planner", "critic"]);
    }

    #[test]
    fn the_measure_gives_both_outcomes_on_a_mixed_fixture() {
        let mut records = fixture();
        // An audience run from before the window opened does not keep the role active.
        records.audience_runs[0].started_at = jiff::Timestamp::from_second(5).unwrap();
        let need = RoleNeed::compute(&records).unwrap();
        let flags: Vec<bool> = need.roles.iter().map(|r| r.retire_candidate).collect();
        assert!(flags.contains(&true) && flags.contains(&false), "{flags:?}");
        assert_eq!(named(&need), ["planner", "critic", "audience"]);
        assert!(
            need.roles
                .iter()
                .any(|r| r.role == "steward" && r.runs == 1)
        );
    }

    #[test]
    fn a_thin_history_names_no_candidate() {
        let records = with_jobs(
            (0..WINDOW as u32 - 1)
                .map(|n| closed_job(n, "code", OutcomeState::Landed))
                .collect(),
        );
        assert_eq!(RoleNeed::compute(&records), None);
        assert_eq!(RoleNeed::shown(&Records::default()), None);
    }

    #[test]
    fn outcomes_are_mapped_and_the_class_history_is_windowed() {
        let mut jobs = Vec::new();
        // Two old failures of class "code" fall outside its newest eight.
        jobs.push(closed_job(0, "code", OutcomeState::Failed));
        jobs.push(closed_job(1, "code", OutcomeState::Rejected));
        // The newest eight: failed, rejected and timed out count as failed; landed does not.
        for (n, state) in [
            (2, OutcomeState::Failed),
            (3, OutcomeState::Rejected),
            (4, OutcomeState::TimedOut),
            (5, OutcomeState::Landed),
            (6, OutcomeState::Landed),
            (7, OutcomeState::Landed),
            (8, OutcomeState::Landed),
            (9, OutcomeState::Landed),
        ] {
            jobs.push(closed_job(n, "code", state));
        }
        // Declined and bootstrap-closed jobs are not work and are left out of every count.
        jobs.push(closed_job(10, "code", OutcomeState::Declined));
        jobs.push(closed_job(11, "docs", OutcomeState::Closed));
        jobs.push(closed_job(12, "docs", OutcomeState::Abandoned));
        let need = RoleNeed::compute(&with_jobs(jobs)).unwrap();
        let code = need.classes.iter().find(|c| c.class == "code").unwrap();
        assert_eq!((code.closed, code.failed), (8, 3));
        let docs = need.classes.iter().find(|c| c.class == "docs").unwrap();
        assert_eq!((docs.closed, docs.failed), (1, 0));
    }

    #[test]
    fn the_detail_names_candidates_and_classes() {
        let need = RoleNeed::compute(&fixture()).unwrap();
        let text = need.detail();
        assert!(
            text.contains("retire candidates: planner, critic"),
            "{text}"
        );
        assert!(text.contains("code 0/8 failed"), "{text}");
    }
}
