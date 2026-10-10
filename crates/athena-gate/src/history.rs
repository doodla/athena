//! The full-history check that CI runs (`athena check --history`, design §7.4 and §11.2).
//!
//! It runs gitleaks once over every commit reachable from the tip, under the tip's `[secrets]`, then judges
//! every commit, oldest first, by its parent's rules: the content checks, the governed-content invariant
//! (including the audit of landing commits), the prediction trailers, and the records each commit adds to a
//! job, judged against the rest of the job. The two allow-lists
//! for sensitive content, `[secrets]` and `[personal_data]`, come from the tip, where allow-listing an old
//! false positive is legitimate. Finally it validates and reconciles every record at the tip.

use crate::invariant::{HistoryState, JOB, VERIFIED_CONTENT};
use crate::records::RecordSet;
use crate::{
    Change, Judged, Result, Rule, Violation, config_at, content, invariant, jobs, reconcile,
    secrets, trailer_values,
};
use athena_git::Git;

/// What the history check found.
#[derive(Debug, Default)]
pub struct HistoryReport {
    pub commits: usize,
    pub records: usize,
    pub violations: Vec<Violation>,
}

/// Checks every commit reachable from `tip`.
pub fn check_history(git: &Git, tip: &str) -> Result<HistoryReport> {
    let tip = git
        .rev_parse(tip)?
        .ok_or_else(|| crate::GateError::Config(format!("{tip} is not a commit")))?;
    let mut report = HistoryReport::default();
    let tip_config = config_at(git, Some(&tip))?;
    report
        .violations
        .extend(secrets::check_history(git, &tip_config, &tip)?);

    let mut state = HistoryState {
        tip: tip.clone(),
        ..HistoryState::default()
    };
    for commit in git.rev_list_reverse(&tip)? {
        report.commits += 1;
        let change = Change::of_commit(git, &commit)?;
        let judged = Judged::new(git, &change)?;
        if let Some(error) = &judged.config_error {
            report.violations.push(
                Violation::new(Rule::Configuration, error.clone(), "").in_commit(Some(&commit)),
            );
        }
        report.violations.extend(content::check(
            git,
            &change,
            &judged,
            Some(&tip_config.personal_data),
        )?);
        report
            .violations
            .extend(invariant::check(git, &change, &judged, Some(&state))?);
        report
            .violations
            .extend(invariant::check_prediction(git, &change)?);
        report
            .violations
            .extend(jobs::check(git, &change, &judged, Some(&state))?);
        let trailers = change.trailers(git)?;
        if !trailer_values(&trailers, VERIFIED_CONTENT).is_empty() {
            for job in trailer_values(&trailers, JOB) {
                state.landed.insert(job.to_string());
            }
        }
    }

    let records = RecordSet::load(git, &tip)?;
    report.records = records.files.len();
    for error in records.errors() {
        report
            .violations
            .push(Violation::new(Rule::ValidRecords, error.to_string(), ""));
    }
    report.violations.extend(reconcile::check_records(&records));
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn a_clean_bootstrap_history_passes() {
        // M3 (allows): a clean history passes.
        let (dir, git) = repo(&[("README.md", "hello\n")]);
        write(dir.path(), &[("src/lib.rs", "pub fn f() {}\n")]);
        commit(&git, "code\n\nAthena-Job: bootstrap\n");
        let report = check_history(&git, "HEAD").unwrap();
        assert_eq!(report.commits, 2);
        assert_eq!(report.violations, vec![]);
    }

    #[test]
    fn a_hand_change_anywhere_in_history_is_found() {
        let (dir, git) = repo(&[("README.md", "hello\n")]);
        write(dir.path(), &[("src/lib.rs", "pub fn f() {}\n")]);
        commit(&git, "unlabeled hand change\n");
        write(dir.path(), &[("src/lib.rs", "pub fn f() { 1; }\n")]);
        commit(&git, "later work\n\nAthena-Job: bootstrap\n");
        let report = check_history(&git, "HEAD").unwrap();
        assert_eq!(report.violations.len(), 1, "{:?}", report.violations);
        assert_eq!(report.violations[0].rule, Rule::Invariant);
    }
}
