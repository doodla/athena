//! The verification report (requirement OLY-B-A7-04): what verification cost, what it caught, the rejections
//! that were overruled (false positives) and the landings a later revert or fix request cites (escapes).
//! Read from the verification, invocation, outcome and request records; nothing new is recorded.

use super::planning::{LaterCause, later_causes};
use crate::store::{JobRecords, JobState, Records};
use athena_data::records::{OutcomeState, Role, Verdict};
use serde::Serialize;

/// Verification figures over every request job that has a verification record.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct VerificationReport {
    /// Jobs with a verification record.
    pub verified: usize,
    /// Verifier invocation cost over those jobs, in USD (the best known lower bound of each invocation).
    pub cost_usd: f64,
    /// Verifications whose final verdict rejected the candidate.
    pub caught: usize,
    /// Rejections overruled: the verifier's own verdict was a fail but the final verdict passed, or the job
    /// rejected by the final verdict landed all the same. The best the records show; not a judgment.
    pub false_positives: usize,
    /// Landed jobs a later request cites with a revert or a fix.
    pub escaped: usize,
}

/// Whether `job` landed, at its outcome or at a later one.
fn landed(job: &JobRecords) -> bool {
    matches!(job.state(), JobState::Closed(OutcomeState::Landed))
        || job
            .later_outcome
            .as_ref()
            .is_some_and(|o| o.state == OutcomeState::Landed)
}

/// Counts verifier cost, rejections caught, false positives and escapes in `records`.
pub fn verification_report(records: &Records) -> VerificationReport {
    let mut report = VerificationReport::default();
    for job in records.request_jobs() {
        let Some(v) = job.verification.as_ref() else {
            continue;
        };
        report.verified += 1;
        report.cost_usd += job
            .invocations
            .iter()
            .filter(|i| i.role == Role::Verifier)
            .map(|i| i.cost.best_lower())
            .sum::<f64>();
        let rejected = v.final_verdict == Verdict::Fail;
        report.caught += usize::from(rejected);
        let overruled =
            (v.agent_verdict == Some(Verdict::Fail) && !rejected) || (rejected && landed(job));
        report.false_positives += usize::from(overruled);
        if landed(job) {
            let causes = later_causes(records, job);
            report.escaped += usize::from(
                causes
                    .iter()
                    .any(|c| matches!(c, LaterCause::Revert | LaterCause::Fix)),
            );
        }
    }
    report
}
