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
    Change, GateError, Judged, Result, Rule, Violation, builtin_config, config_at, content,
    invariant, jobs, reconcile, secrets, trailer_values,
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
    // A tip whose configuration does not parse is a violation to report, not a reason to stop: the
    // built-in configuration judges instead, as it does for any commit whose parent's does not parse.
    let tip_config = match config_at(git, Some(&tip)) {
        Ok(config) => config,
        Err(GateError::Config(message)) => {
            report.violations.push(Violation::new(
                Rule::Configuration,
                message,
                "fix athena.toml in a commit of its own",
            ));
            builtin_config()
        }
        Err(other) => return Err(other),
    };
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

#[cfg(test)]
mod exemption_tests {
    use super::*;
    use crate::testutil::*;

    /// The built-in configuration with `entries` as its `[history] exempt` list.
    fn config_exempting(entries: &[&str]) -> String {
        let list = entries
            .iter()
            .map(|e| format!("\"{e}\""))
            .collect::<Vec<_>>()
            .join(", ");
        crate::BUILTIN_CONFIG
            .lines()
            .map(|l| {
                if l.starts_with("exempt = ") {
                    format!("exempt = [{list}]")
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    }

    #[test]
    fn a_short_history_exemption_does_not_hide_a_hand_change() {
        // M6 (blocks), N1 of acceptance verification 2: an exemption "0", added by a declared
        // intervention, and a later hand edit whose commit id begins with 0, committed without hooks. The
        // history check flags the exemption and the hand edit.
        let builtin = crate::builtin_config();
        let (dir, git) = repo(&[
            (
                "athena.toml",
                &config_exempting(&[&builtin.history.exempt[0]]),
            ),
            ("README.md", "hello\n"),
        ]);
        let record = r#"{
  "schema": "athena.intervention/1",
  "id": "20261010-160000-abcd",
  "at": "2026-10-10T16:00:00Z",
  "what": "Exempted old commits from the history check",
  "why": "They predate Athena",
  "class_of_work": "code",
  "category": "did-work-themselves",
  "job": null,
  "source": "user",
  "with_staged_changes": true
}
"#;
        write(
            dir.path(),
            &[
                ("athena.toml", &config_exempting(&["0"])),
                (
                    "records/interventions/2026-10/20261010-160000-abcd.json",
                    record,
                ),
            ],
        );
        let exemption = commit(
            &git,
            "Exempt old commits\n\nAthena-Intervention: 20261010-160000-abcd\n",
        );
        write(dir.path(), &[("README.md", "edited by hand\n")]);
        let mut hand = commit(&git, "hand edit 0\n");
        let mut tries = 0;
        while !hand.starts_with('0') {
            tries += 1;
            git.run([
                "commit",
                "-q",
                "--amend",
                "-m",
                &format!("hand edit {tries}\n"),
            ])
            .unwrap();
            hand = git.head().unwrap().unwrap();
        }
        let report = check_history(&git, "HEAD").unwrap();
        let in_commit = |id: &str, rule: Rule| {
            report
                .violations
                .iter()
                .any(|v| v.rule == rule && v.commit.as_deref() == Some(&id[..12]))
        };
        assert!(
            in_commit(&exemption, Rule::Configuration),
            "{:?}",
            report.violations
        );
        assert!(
            in_commit(&hand, Rule::Invariant),
            "the hand edit is not hidden: {:?}",
            report.violations
        );
    }

    #[test]
    fn an_exemption_can_only_name_a_commit_already_in_history() {
        // M6 (blocks): an exemption naming a commit that does not exist, or one outside this history.
        // (allows): an exemption naming an earlier commit of this history.
        let builtin = crate::builtin_config();
        let config = |entries: &[&str]| {
            let mut all = vec![builtin.history.exempt[0].as_str()];
            all.extend_from_slice(entries);
            config_exempting(&all)
        };
        let (dir, git) = repo(&[("athena.toml", &config(&[])), ("README.md", "hello\n")]);
        let earlier = git.head().unwrap().unwrap();
        git.run(["checkout", "-q", "-b", "side"]).unwrap();
        write(dir.path(), &[("SIDE.md", "elsewhere\n")]);
        let elsewhere = commit(&git, "side work\n\nAthena-Job: bootstrap\n");
        git.run(["checkout", "-q", "main"]).unwrap();
        for (entry, allowed) in [
            (earlier.as_str(), true),
            ("1234567890abcdef1234567890abcdef12345678", false),
            (elsewhere.as_str(), false),
        ] {
            write(dir.path(), &[("athena.toml", &config(&[entry]))]);
            commit(&git, "exempt a commit\n\nAthena-Job: bootstrap\n");
            let head = git.head().unwrap().unwrap();
            let change = Change::of_commit(&git, &head).unwrap();
            let judged = Judged::new(&git, &change).unwrap();
            let found = content::check(&git, &change, &judged, None).unwrap();
            assert_eq!(found.is_empty(), allowed, "{entry}: {found:?}");
            git.run(["reset", "-q", "--hard", "HEAD~1"]).unwrap();
        }
    }
}
