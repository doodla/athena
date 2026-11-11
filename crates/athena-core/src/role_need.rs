//! The role-need measure (requirement OLY-9-11): where the records show a need to add a role or retire one.
//!
//! A class of work with at least [`MIN_CLASS_JOBS`] closed jobs, of which at least [`MIN_FAILURES`] failed and
//! at least half failed, is a candidate for a new, more specialized role. A role with no invocation in the
//! newest [`RETIRE_WINDOW`] closed jobs is a candidate for retirement. Both are advisory: nothing here creates
//! or retires a role. `athena status` prints the result as the `role need` line and prints nothing when the
//! history is too thin, so a thin history is not shown as a good result.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::store::{JobState, Records};
use athena_data::records::{OutcomeState, Role};

/// Closed jobs a class needs before its failures are counted.
pub const MIN_CLASS_JOBS: usize = 4;
/// Failed jobs a class needs, and at least half of its jobs, to be a role-needed candidate.
pub const MIN_FAILURES: usize = 2;
/// Newest closed jobs in which a role must have no invocation to be a retire candidate.
pub const RETIRE_WINDOW: usize = 8;

/// One closed job: its class, whether it failed, and the roles that were invoked for it.
#[derive(Debug, Clone, PartialEq)]
pub struct NeedJob {
    pub class: String,
    pub failed: bool,
    pub roles: BTreeSet<String>,
}

/// A class of work whose repeated failures suggest a role specialized for it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RoleNeeded {
    pub class: String,
    pub jobs: usize,
    pub failed: usize,
}

/// The candidates found in the records.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RoleNeed {
    pub needed: Vec<RoleNeeded>,
    /// Roles with no invocation in the newest [`RETIRE_WINDOW`] closed jobs.
    pub retire: Vec<String>,
}

/// Finds candidates in `jobs` (closed jobs, oldest first) given the `known` roles. None when there are none.
pub fn analyze(jobs: &[NeedJob], known: &[String]) -> Option<RoleNeed> {
    let mut by_class: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for job in jobs {
        let entry = by_class.entry(job.class.as_str()).or_default();
        entry.0 += 1;
        entry.1 += usize::from(job.failed);
    }
    let needed: Vec<RoleNeeded> = by_class
        .into_iter()
        .filter(|&(_, (n, failed))| {
            n >= MIN_CLASS_JOBS && failed >= MIN_FAILURES && failed * 2 >= n
        })
        .map(|(class, (jobs, failed))| RoleNeeded {
            class: class.to_string(),
            jobs,
            failed,
        })
        .collect();
    let retire: Vec<String> = if jobs.len() >= RETIRE_WINDOW {
        let recent = &jobs[jobs.len() - RETIRE_WINDOW..];
        known
            .iter()
            .filter(|role| !recent.iter().any(|j| j.roles.contains(*role)))
            .cloned()
            .collect()
    } else {
        Vec::new()
    };
    (!needed.is_empty() || !retire.is_empty()).then_some(RoleNeed { needed, retire })
}

const ALL_ROLES: [Role; 8] = [
    Role::Reader,
    Role::Producer,
    Role::Verifier,
    Role::Steward,
    Role::Audience,
    Role::Auditor,
    Role::Planner,
    Role::Critic,
];

impl RoleNeed {
    /// Measures `records`. None when no class or role qualifies.
    pub fn compute(records: &Records) -> Option<RoleNeed> {
        let mut closed: Vec<(String, NeedJob)> = records
            .request_jobs()
            .into_iter()
            .filter_map(|j| {
                let JobState::Closed(state) = j.state() else {
                    return None;
                };
                if matches!(state, OutcomeState::Declined | OutcomeState::Closed) {
                    return None;
                }
                let failed = matches!(
                    state,
                    OutcomeState::Failed | OutcomeState::Rejected | OutcomeState::TimedOut
                );
                Some((
                    j.id().as_str().to_string(),
                    NeedJob {
                        class: j.class()?.to_string(),
                        failed,
                        roles: j
                            .invocations
                            .iter()
                            .map(|i| i.role.as_str().to_string())
                            .collect(),
                    },
                ))
            })
            .collect();
        // Job ids begin with their creation time, so id order is time order.
        closed.sort_by(|a, b| a.0.cmp(&b.0));
        let jobs: Vec<NeedJob> = closed.into_iter().map(|(_, j)| j).collect();
        let known: Vec<String> = ALL_ROLES.iter().map(|r| r.as_str().to_string()).collect();
        analyze(&jobs, &known)
    }

    /// The text `athena status` prints after the `role need` label: one line per candidate.
    pub fn lines(&self) -> Vec<String> {
        let mut out = Vec::new();
        for n in &self.needed {
            out.push(format!(
                "role needed: class {} failed {} of {} jobs; a role specialized for it may help",
                n.class, n.failed, n.jobs
            ));
        }
        for role in &self.retire {
            out.push(format!(
                "retire candidate: {role} had no invocation in the newest {RETIRE_WINDOW} closed jobs"
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(class: &str, failed: bool, roles: &[&str]) -> NeedJob {
        NeedJob {
            class: class.into(),
            failed,
            roles: roles.iter().map(|r| r.to_string()).collect(),
        }
    }

    fn known() -> Vec<String> {
        vec!["producer".into(), "auditor".into()]
    }

    #[test]
    fn a_class_with_repeated_failures_is_a_role_needed_candidate() {
        let jobs = vec![
            job("code", true, &["producer", "auditor"]),
            job("code", true, &["producer", "auditor"]),
            job("code", false, &["producer", "auditor"]),
            job("code", true, &["producer", "auditor"]),
        ];
        let need = analyze(&jobs, &known()).expect("a candidate");
        assert_eq!(
            need.needed,
            vec![RoleNeeded {
                class: "code".into(),
                jobs: 4,
                failed: 3
            }]
        );
        assert!(need.retire.is_empty());
        assert_eq!(
            need.lines(),
            vec!["role needed: class code failed 3 of 4 jobs; a role specialized for it may help"]
        );
    }

    #[test]
    fn a_role_with_no_recent_invocation_is_a_retire_candidate() {
        let jobs: Vec<NeedJob> = (0..RETIRE_WINDOW)
            .map(|_| job("docs", false, &["producer"]))
            .collect();
        let need = analyze(&jobs, &known()).expect("a candidate");
        assert!(need.needed.is_empty());
        assert_eq!(need.retire, vec!["auditor".to_string()]);
        assert!(need.lines()[0].starts_with("retire candidate: auditor"));
    }

    #[test]
    fn a_thin_history_yields_nothing() {
        // Three failures in one class is below the minimum job count; seven jobs is below the retire window.
        let jobs = vec![
            job("code", true, &[]),
            job("code", true, &[]),
            job("code", true, &[]),
            job("docs", false, &["producer"]),
            job("docs", false, &["producer"]),
            job("docs", false, &["producer"]),
            job("docs", false, &["producer"]),
        ];
        assert_eq!(analyze(&jobs, &known()), None);
        assert_eq!(analyze(&[], &known()), None);
    }

    #[test]
    fn a_class_that_mostly_succeeds_is_not_a_candidate() {
        let mut jobs = vec![job("code", true, &["producer", "auditor"]); 2];
        jobs.extend(vec![job("code", false, &["producer", "auditor"]); 4]);
        assert_eq!(analyze(&jobs, &known()), None);
    }
}
