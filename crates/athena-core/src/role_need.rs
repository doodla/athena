//! The role-need measure (requirement OLY-9-11): which roles are in use, which look unused, and which classes
//! of work fail often enough that a new or changed role may be worth considering.
//!
//! A role's runs are read from where they are recorded: the invocations kept with each job, and the
//! separate lists of the steward, audience and auditor, whose runs are no job. Reading only the job
//! invocations would show those three as unused, so every source is counted for every role.
//!
//! * A role with a run in the last [`WINDOW_DAYS`] days is in use.
//! * A role with runs on record but none in the window is a retire candidate, when the window itself holds
//!   at least [`MIN_WINDOW_RUNS`] runs, so a quiet period does not flag every role.
//! * A role with no run on record, or judged in a thin window, is not measured; it is never a candidate.
//! * A class of work whose newest [`RECENT_JOBS_PER_CLASS`] closed jobs hold at least [`MIN_FAILURES`]
//!   failures, and at least half of them, is listed as needing attention. A class with fewer than
//!   [`MIN_CLASS_JOBS`] closed jobs yields nothing.
//!
//! The result is advice only: nothing adds or retires a role by itself. `athena status` prints it as the
//! `role need` lines.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::store::{JobState, Records};
use athena_data::records::{OutcomeState, Role};

/// Every role, in the order `athena status` lists them.
pub const ALL_ROLES: [Role; 8] = [
    Role::Reader,
    Role::Planner,
    Role::Critic,
    Role::Producer,
    Role::Verifier,
    Role::Steward,
    Role::Audience,
    Role::Auditor,
];

/// Days back from now that count as recent for a role's runs.
pub const WINDOW_DAYS: i64 = 14;
/// Runs of all roles the window must hold before a role with none in it is called a retire candidate.
pub const MIN_WINDOW_RUNS: usize = 5;
/// The newest closed jobs of a class that are looked at; older ones are ignored.
pub const RECENT_JOBS_PER_CLASS: usize = 10;
/// Closed jobs a class needs before its failures are counted.
pub const MIN_CLASS_JOBS: usize = 4;
/// Failures among a class's recent jobs that make it need attention (and at least half of them).
pub const MIN_FAILURES: usize = 3;

/// One recorded run of a role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoleRun {
    pub role: Role,
    pub at: jiff::Timestamp,
}

/// One closed job: its class and whether it failed. A list of these is newest first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedJob {
    pub class: String,
    pub failed: bool,
}

/// How a role stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleStatus {
    /// `runs` runs in the window.
    InUse { runs: usize },
    /// Runs on record, none in the window.
    RetireCandidate,
    /// Not judged, with the reason.
    NotMeasured { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoleStanding {
    pub role: Role,
    pub status: RoleStatus,
}

/// A class whose recent closed jobs failed repeatedly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClassFailures {
    pub class: String,
    pub failed: usize,
    pub jobs: usize,
}

/// The role-need measure: one standing per role in [`ALL_ROLES`], and the classes that need attention.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoleNeed {
    pub roles: Vec<RoleStanding>,
    pub needed: Vec<ClassFailures>,
}

/// Whether a job that ended in `state` failed; None for a state that says nothing about the work
/// (declined requests and the bootstrap's own close).
pub fn failure_of(state: OutcomeState) -> Option<bool> {
    match state {
        OutcomeState::Failed | OutcomeState::Rejected | OutcomeState::TimedOut => Some(true),
        OutcomeState::Declined | OutcomeState::Closed => None,
        _ => Some(false),
    }
}

/// Measures the runs and closed jobs (newest first) as of `now`.
pub fn analyze(runs: &[RoleRun], jobs: &[ClosedJob], now: jiff::Timestamp) -> RoleNeed {
    let since = now - jiff::SignedDuration::from_hours(24 * WINDOW_DAYS);
    let in_window = |r: &&RoleRun| r.at >= since && r.at <= now;
    let window_total = runs.iter().filter(in_window).count();
    let roles = ALL_ROLES
        .iter()
        .map(|&role| {
            let recorded = runs.iter().filter(|r| r.role == role).count();
            let recent = runs
                .iter()
                .filter(|r| r.role == role)
                .filter(in_window)
                .count();
            let status = if recent > 0 {
                RoleStatus::InUse { runs: recent }
            } else if recorded == 0 {
                RoleStatus::NotMeasured {
                    reason: "no run of this role is recorded".into(),
                }
            } else if window_total < MIN_WINDOW_RUNS {
                RoleStatus::NotMeasured {
                    reason: format!(
                        "fewer than {MIN_WINDOW_RUNS} runs of any role in the last {WINDOW_DAYS} days"
                    ),
                }
            } else {
                RoleStatus::RetireCandidate
            };
            RoleStanding { role, status }
        })
        .collect();

    let mut by_class: BTreeMap<&str, Vec<bool>> = BTreeMap::new();
    for job in jobs {
        let recent = by_class.entry(job.class.as_str()).or_default();
        if recent.len() < RECENT_JOBS_PER_CLASS {
            recent.push(job.failed);
        }
    }
    let needed = by_class
        .into_iter()
        .filter(|(_, recent)| recent.len() >= MIN_CLASS_JOBS)
        .filter_map(|(class, recent)| {
            let failed = recent.iter().filter(|f| **f).count();
            (failed >= MIN_FAILURES && failed * 2 >= recent.len()).then(|| ClassFailures {
                class: class.to_string(),
                failed,
                jobs: recent.len(),
            })
        })
        .collect();
    RoleNeed { roles, needed }
}

impl RoleNeed {
    /// Measures `records` as of now.
    pub fn compute(records: &Records) -> RoleNeed {
        Self::compute_at(records, crate::now())
    }

    /// Measures `records` as of `now`: runs from every job's invocations and from the steward, audience and
    /// auditor lists; closed request jobs newest first.
    pub fn compute_at(records: &Records, now: jiff::Timestamp) -> RoleNeed {
        let runs: Vec<RoleRun> = records
            .jobs
            .values()
            .flat_map(|j| j.invocations.iter())
            .chain(&records.steward_runs)
            .chain(&records.audience_runs)
            .chain(&records.auditor_runs)
            .map(|i| RoleRun {
                role: i.role,
                at: i.started_at,
            })
            .collect();
        let jobs: Vec<ClosedJob> = records
            .request_jobs()
            .into_iter()
            .filter_map(|j| {
                let JobState::Closed(state) = j.state() else {
                    return None;
                };
                Some(ClosedJob {
                    class: j.class()?.to_string(),
                    failed: failure_of(state)?,
                })
            })
            .collect();
        analyze(&runs, &jobs, now)
    }

    /// The roles flagged as retire candidates.
    pub fn candidates(&self) -> Vec<Role> {
        self.roles
            .iter()
            .filter(|r| r.status == RoleStatus::RetireCandidate)
            .map(|r| r.role)
            .collect()
    }

    /// The text `athena status` prints after the `role need` label, one entry per line.
    pub fn lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        for s in &self.roles {
            let name = s.role.as_str();
            out.push(match &s.status {
                RoleStatus::InUse { runs } => {
                    format!("{name:<9} in use, {runs} run(s) in the last {WINDOW_DAYS} days")
                }
                RoleStatus::RetireCandidate => format!(
                    "{name:<9} retire candidate: no run in the last {WINDOW_DAYS} days (advice only; no role changes by itself)"
                ),
                RoleStatus::NotMeasured { reason } => format!("{name:<9} not measured: {reason}"),
            });
        }
        for c in &self.needed {
            out.push(format!(
                "needed: class {} failed in {} of its last {} closed jobs; a new or changed role for it may help",
                c.class, c.failed, c.jobs
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::planning_cost_tests::invocation;
    use crate::store::JobRecords;
    use athena_data::JobId;
    use athena_data::records::{Invocation, Outcome};

    fn now() -> jiff::Timestamp {
        "2026-11-11T12:00:00Z".parse().unwrap()
    }

    fn days_ago(days: i64) -> jiff::Timestamp {
        now() - jiff::SignedDuration::from_hours(24 * days)
    }

    fn run(role: Role, days: i64) -> RoleRun {
        RoleRun {
            role,
            at: days_ago(days),
        }
    }

    fn recorded(role: Role, job: &str, seq: u32, days: i64) -> Invocation {
        let mut i = invocation(&JobId::parse(job).unwrap(), seq, role, 1.0);
        i.started_at = days_ago(days);
        i
    }

    fn standing(need: &RoleNeed, role: Role) -> &RoleStatus {
        &need.roles.iter().find(|r| r.role == role).unwrap().status
    }

    fn job_id(n: u32) -> String {
        format!("202610{:02}-120000-a{:03}", n % 28 + 1, n)
    }

    /// A request job with a class, an outcome and invocations of the given roles started `days` ago.
    fn job(n: u32, class: &str, state: OutcomeState, roles: &[(Role, i64)]) -> JobRecords {
        let text = job_id(n);
        let id = JobId::parse(&text).unwrap();
        let stub: athena_data::records::Job = serde_json::from_value(serde_json::json!({
            "id": text, "kind": "request", "title": "t", "class_of_work": class,
            "capability": null, "accepted_reading": null, "accepted_by": null,
            "owner": { "role": "producer", "capability": null, "model": null, "note": null },
            "verifier_model": null, "caps": null, "base_commit": null, "config_sha256": null,
            "defaults_sha256": null, "athena": null, "driver": null, "brief_sha256": null,
            "dispatched_at": null, "request_path": null, "request_sha256": null, "cost": null,
            "tokens": null, "note": null, "pin": null
        }))
        .unwrap();
        JobRecords {
            id: Some(id.clone()),
            job: Some(stub),
            invocations: roles
                .iter()
                .enumerate()
                .map(|(k, (role, days))| recorded(*role, &text, k as u32 + 1, *days))
                .collect(),
            outcome: Some(Outcome {
                job: id,
                state,
                reason: "r".into(),
                at: jiff::Timestamp::UNIX_EPOCH,
                totals: None,
                branch: None,
                base_commit: None,
                candidate_commit: None,
                after_blocked: false,
                through_commit: None,
            }),
            ..JobRecords::default()
        }
    }

    fn records(jobs: Vec<JobRecords>) -> Records {
        let mut records = Records::default();
        for j in jobs {
            records.jobs.insert(j.id().as_str().to_string(), j);
        }
        records
    }

    #[test]
    fn steward_audience_and_auditor_runs_kept_apart_from_jobs_are_not_retire_candidates() {
        // The real shape: request jobs hold reader, producer and verifier runs only; the other three roles
        // run alone and are recorded in their own lists.
        let mut recs = records(
            (0..6)
                .map(|n| {
                    job(
                        n,
                        "code",
                        OutcomeState::Landed,
                        &[(Role::Reader, 1), (Role::Producer, 1), (Role::Verifier, 1)],
                    )
                })
                .collect(),
        );
        let other = "20261110-100000-zzzz";
        recs.steward_runs = (0..3)
            .map(|k| recorded(Role::Steward, other, 1, k))
            .collect();
        recs.audience_runs = vec![recorded(Role::Audience, other, 1, 1)];
        recs.auditor_runs = vec![recorded(Role::Auditor, other, 1, 2)];
        let need = RoleNeed::compute_at(&recs, now());
        for role in [Role::Steward, Role::Audience, Role::Auditor] {
            assert!(
                matches!(standing(&need, role), RoleStatus::InUse { .. }),
                "{role:?} must be in use: {:?}",
                standing(&need, role)
            );
            assert!(!need.candidates().contains(&role));
        }
        assert_eq!(
            standing(&need, Role::Steward),
            &RoleStatus::InUse { runs: 3 }
        );
        // Planner and critic never ran: not measured, not a candidate.
        assert!(matches!(
            standing(&need, Role::Planner),
            RoleStatus::NotMeasured { .. }
        ));
        assert!(need.candidates().is_empty());
        assert_eq!(need.roles.len(), ALL_ROLES.len());
    }

    #[test]
    fn a_role_with_runs_only_before_the_window_is_a_retire_candidate() {
        let mut runs: Vec<RoleRun> = (0..6).map(|_| run(Role::Reader, 1)).collect();
        runs.push(run(Role::Planner, WINDOW_DAYS + 5));
        let need = analyze(&runs, &[], now());
        assert_eq!(standing(&need, Role::Planner), &RoleStatus::RetireCandidate);
        assert_eq!(need.candidates(), vec![Role::Planner]);
        assert_eq!(
            standing(&need, Role::Reader),
            &RoleStatus::InUse { runs: 6 }
        );
        // Never recorded: not measured.
        assert!(matches!(
            standing(&need, Role::Critic),
            RoleStatus::NotMeasured { .. }
        ));
        let text = need.lines().join("\n");
        assert!(text.contains("planner   retire candidate"), "{text}");
    }

    #[test]
    fn a_thin_window_flags_nothing_for_retirement() {
        let runs = vec![run(Role::Reader, 1), run(Role::Planner, 40)];
        let need = analyze(&runs, &[], now());
        assert!(need.candidates().is_empty());
        assert!(matches!(
            standing(&need, Role::Planner),
            RoleStatus::NotMeasured { .. }
        ));
        assert!(need.needed.is_empty());
    }

    fn closed(class: &str, failed: bool) -> ClosedJob {
        ClosedJob {
            class: class.into(),
            failed,
        }
    }

    #[test]
    fn repeated_recent_failures_in_a_class_make_it_needed() {
        let jobs = vec![
            closed("code", true),
            closed("code", true),
            closed("code", false),
            closed("code", true),
            closed("docs", false),
        ];
        let need = analyze(&[], &jobs, now());
        assert_eq!(
            need.needed,
            vec![ClassFailures {
                class: "code".into(),
                failed: 3,
                jobs: 4
            }]
        );
        let text = need.lines().join("\n");
        assert!(
            text.contains("needed: class code failed in 3 of its last 4"),
            "{text}"
        );
    }

    #[test]
    fn thin_class_history_yields_nothing() {
        let jobs = vec![
            closed("code", true),
            closed("code", true),
            closed("code", true),
        ];
        assert!(analyze(&[], &jobs, now()).needed.is_empty());
    }

    #[test]
    fn only_the_newest_jobs_of_a_class_are_counted() {
        // Newest first: ten good jobs, then older failures that are out of the window.
        let mut jobs: Vec<ClosedJob> = (0..RECENT_JOBS_PER_CLASS)
            .map(|_| closed("code", false))
            .collect();
        jobs.extend((0..5).map(|_| closed("code", true)));
        assert!(analyze(&[], &jobs, now()).needed.is_empty());
        // The same failures among the newest are counted.
        let mut fresh: Vec<ClosedJob> = (0..5).map(|_| closed("code", true)).collect();
        fresh.extend((0..RECENT_JOBS_PER_CLASS).map(|_| closed("code", false)));
        let need = analyze(&[], &fresh, now());
        assert_eq!(need.needed[0].jobs, RECENT_JOBS_PER_CLASS);
        assert_eq!(need.needed[0].failed, 5);
    }

    #[test]
    fn outcome_states_map_to_failed_or_excluded() {
        for state in [
            OutcomeState::Failed,
            OutcomeState::Rejected,
            OutcomeState::TimedOut,
        ] {
            assert_eq!(failure_of(state), Some(true), "{state:?}");
        }
        for state in [OutcomeState::Declined, OutcomeState::Closed] {
            assert_eq!(failure_of(state), None, "{state:?}");
        }
        assert_eq!(failure_of(OutcomeState::Landed), Some(false));
    }

    #[test]
    fn compute_reads_closed_jobs_from_records_and_skips_excluded_outcomes() {
        let mut jobs: Vec<JobRecords> = Vec::new();
        for (n, state) in [
            (1, OutcomeState::Failed),
            (2, OutcomeState::Rejected),
            (3, OutcomeState::TimedOut),
            (4, OutcomeState::Landed),
            // Excluded: they neither fail nor count as a job of the class.
            (5, OutcomeState::Declined),
            (6, OutcomeState::Closed),
        ] {
            jobs.push(job(n, "code", state, &[(Role::Reader, 1)]));
        }
        let need = RoleNeed::compute_at(&records(jobs), now());
        assert_eq!(
            need.needed,
            vec![ClassFailures {
                class: "code".into(),
                failed: 3,
                jobs: 4
            }]
        );
    }

    #[test]
    fn compute_orders_closed_jobs_newest_first() {
        // Ten newest landed; the five oldest failed. Only the newest ten are looked at.
        let mut jobs: Vec<JobRecords> = (0..5)
            .map(|n| job(n, "code", OutcomeState::Failed, &[]))
            .collect();
        jobs.extend((5..15).map(|n| job(n, "code", OutcomeState::Landed, &[])));
        let need = RoleNeed::compute_at(&records(jobs), now());
        assert!(need.needed.is_empty(), "{:?}", need.needed);
    }
}
