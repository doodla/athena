//! The full-history check that CI runs (`athena check --history`, design §7.4 and §11.2).
//!
//! It runs gitleaks once over every commit reachable from the tip, under the tip's `[secrets]`, then judges
//! every commit, oldest first, by its parent's rules: the content checks, the governed-content invariant
//! (including the audit of landing commits), the prediction trailers, and the records each commit adds to a
//! job, judged against the rest of the job. The two allow-lists
//! for sensitive content, `[secrets]` and `[personal_data]`, and the `[history] exempt` list come from the
//! tip, where allow-listing an old false positive or excusing old history is legitimate. Finally it validates and reconciles every record at the tip.

use crate::invariant::HistoryState;
use crate::records::RecordSet;
use crate::{
    Change, GateError, Judged, Result, Rule, Violation, builtin_config, config_at, content,
    invariant, jobs, reconcile, secrets,
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
        exempt: tip_config.history.exempt.clone(),
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
            .extend(jobs::check(git, &change, &judged, Some(&state))?);
        // A job has landed once a commit validly lands it, by its records and not by its trailers.
        // A commit that lands a chain of jobs lands every one of them.
        for job in invariant::landing_job(git, &change, &judged, Some(&state))? {
            state.landed.insert(job.as_str().to_string());
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
mod chain_tests {
    use super::*;
    use crate::landing_tests::{chain_repo, land_chain};
    use crate::testutil::*;

    const STEP_1: &str = "pub fn a() { /* one */ }\n";
    const STEP_2: &str = "pub fn a() { /* two */ }\n";
    const STEP_3: &str = "pub fn a() { /* three */ }\n";

    #[test]
    fn repeating_a_job_of_a_landed_chain_is_refused() {
        let (dir, git, links) = chain_repo(&[STEP_1, STEP_2, STEP_3]);
        land_chain(dir.path(), &git, &links, STEP_3, &|_, _| {});
        assert_eq!(check_history(&git, "HEAD").unwrap().violations, vec![]);

        // Undo the landing by hand, then land only the middle job again, by itself.
        std::fs::remove_dir_all(dir.path().join("records")).unwrap();
        write(dir.path(), &[("src/lib.rs", "pub fn a() {}\n")]);
        commit(&git, "Undo\n\nAthena-Job: bootstrap\n");
        let again = land_chain(dir.path(), &git, &links[1..2], STEP_2, &|_, _| {});
        let report = check_history(&git, "HEAD").unwrap();
        assert!(
            report.violations.iter().any(|v| v.rule == Rule::Invariant
                && v.commit.as_deref() == Some(&again[..12])
                && v.detail.contains("already landed")),
            "{:?}",
            report.violations
        );
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

    const EXEMPTION_RECORD: &str = r#"{
  "schema": "athena.intervention/1",
  "id": "20261010-160000-abcd",
  "at": "2026-10-10T16:00:00Z",
  "what": "Exempted an old commit from the history check",
  "why": "It predates Athena",
  "class_of_work": "code",
  "category": "did-work-themselves",
  "job": null,
  "source": "user",
  "with_staged_changes": true
}
"#;

    /// A history of a hand change, then a declared intervention exempting `exempt(hand)`.
    /// Returns the repository, the hand change's id, and the exemption commit's id.
    fn hand_change_then_exemption(
        exempt: impl Fn(&str) -> String,
    ) -> (tempfile::TempDir, Git, String, String) {
        let builtin = crate::builtin_config();
        let (dir, git) = repo(&[
            (
                "athena.toml",
                &config_exempting(&[&builtin.history.exempt[0]]),
            ),
            ("README.md", "hello\n"),
        ]);
        write(dir.path(), &[("README.md", "edited by hand\n")]);
        let hand = commit(&git, "hand edit\n");
        let entry = exempt(&hand);
        write(
            dir.path(),
            &[
                (
                    "athena.toml",
                    &config_exempting(&[&builtin.history.exempt[0], &entry]),
                ),
                (
                    "records/interventions/2026-10/20261010-160000-abcd.json",
                    EXEMPTION_RECORD,
                ),
            ],
        );
        let exemption = commit(
            &git,
            "Exempt a commit\n\nAthena-Intervention: 20261010-160000-abcd\n",
        );
        (dir, git, hand, exemption)
    }

    fn flagged(report: &HistoryReport, id: &str, rule: Rule) -> bool {
        report
            .violations
            .iter()
            .any(|v| v.rule == rule && v.commit.as_deref() == Some(&id[..12]))
    }

    #[test]
    fn an_exemption_added_later_excuses_the_commit_it_names() {
        // The history check reads the exempt list from the tip, so an entry added after the commit
        // takes effect, though the commit was judged by a parent configuration without it.
        let (_dir, git, hand, _) = hand_change_then_exemption(|hand| hand.to_string());
        let report = check_history(&git, "HEAD").unwrap();
        assert!(
            !flagged(&report, &hand, Rule::Invariant),
            "{:?}",
            report.violations
        );
        assert_eq!(report.violations, vec![]);
    }

    #[test]
    fn an_exemption_does_not_cover_a_later_commit_or_one_not_in_history() {
        // A commit made after the entry is not named by it, so it is still judged.
        let (dir, git, hand, _) = hand_change_then_exemption(|hand| hand.to_string());
        write(dir.path(), &[("README.md", "edited again\n")]);
        let later = commit(&git, "later hand edit\n");
        let report = check_history(&git, "HEAD").unwrap();
        assert!(!flagged(&report, &hand, Rule::Invariant));
        assert!(
            flagged(&report, &later, Rule::Invariant),
            "{:?}",
            report.violations
        );

        // An entry naming a commit not in this history is refused where it is added.
        let absent = "1234567890abcdef1234567890abcdef12345678";
        let (_dir, git, _, exemption) = hand_change_then_exemption(|_| absent.to_string());
        let report = check_history(&git, "HEAD").unwrap();
        assert!(
            flagged(&report, &exemption, Rule::Configuration),
            "{:?}",
            report.violations
        );
    }

    /// A landed outcome and a hand-written score for the job `id`, as `(path, content)` pairs. Neither
    /// follows from any landing, so the job-records check flags both unless the commit is exempt.
    fn hand_written_job_records(id: &str) -> Vec<(String, String)> {
        let dir = format!("records/jobs/2026-10/{id}");
        use athena_data::quantity::{Quantity, Unit};
        let cost = Quantity::measured(0.0, Unit::Usd, "hand written");
        let outcome = serde_json::to_string_pretty(&serde_json::json!({
            "schema": "athena.outcome/1",
            "job": id,
            "state": "landed",
            "reason": "hand written",
            "at": "2026-10-10T18:00:00Z",
            "totals": {
                "cost_all": cost,
                "cost_execution": cost,
                "wall_time_execution": null,
                "tokens": {},
                "invocations": 0
            },
            "branch": null,
            "base_commit": null,
            "candidate_commit": null,
            "after_blocked": false,
            "through_commit": null
        }))
        .unwrap()
            + "\n";
        let score = format!(
            "{{\n  \"schema\": \"athena.score/1\",\n  \"job\": \"{id}\",\n  \"at\": \"2026-10-10T18:00:00Z\",\n  \"rules\": \"{}\",\n  \"cause\": null,\n  \"events\": [],\n  \"deliverables\": {{\"terms\": [], \"sum\": 1.5}},\n  \"quantities\": [],\n  \"models\": null\n}}\n",
            athena_score::RULES
        );
        vec![
            (format!("{dir}/outcome.json"), outcome),
            (format!("{dir}/score.json"), score),
        ]
    }

    fn add_records(dir: &std::path::Path, git: &Git, id: &str) -> String {
        let records = hand_written_job_records(id);
        let files: Vec<(&str, &str)> = records
            .iter()
            .map(|(p, c)| (p.as_str(), c.as_str()))
            .collect();
        write(dir, &files);
        commit(git, &format!("add records\n\nAthena-Record: {id}\n"))
    }

    const JOB_A: &str = "20261010-170000-aaaa";
    const JOB_B: &str = "20261010-171000-bbbb";

    #[test]
    fn an_exemption_also_excuses_the_job_records_the_commit_adds() {
        // Records that arrive in an exempt commit are not flagged, though they are not a valid landing.
        let builtin = crate::builtin_config();
        let (dir, git) = repo(&[
            (
                "athena.toml",
                &config_exempting(&[&builtin.history.exempt[0]]),
            ),
            ("README.md", "hello\n"),
        ]);
        let records = add_records(dir.path(), &git, JOB_A);
        write(
            dir.path(),
            &[
                (
                    "athena.toml",
                    &config_exempting(&[&builtin.history.exempt[0], &records]),
                ),
                (
                    "records/interventions/2026-10/20261010-160000-abcd.json",
                    EXEMPTION_RECORD,
                ),
            ],
        );
        commit(
            &git,
            "Exempt a commit\n\nAthena-Intervention: 20261010-160000-abcd\n",
        );
        let report = check_history(&git, "HEAD").unwrap();
        assert!(
            !report
                .violations
                .iter()
                .any(|v| v.rule == Rule::JobRecords && v.commit.as_deref() == Some(&records[..12])),
            "{:?}",
            report.violations
        );

        // A commit after the exempt one is not excused, and neither are the same records without an
        // exemption.
        let later = add_records(dir.path(), &git, JOB_B);
        let report = check_history(&git, "HEAD").unwrap();
        assert!(
            flagged(&report, &later, Rule::JobRecords),
            "{:?}",
            report.violations
        );
        assert!(!flagged(&report, &records, Rule::JobRecords));
    }

    #[test]
    fn the_same_records_in_a_non_exempt_commit_are_flagged() {
        let (dir, git) = repo(&[("README.md", "hello\n")]);
        let records = add_records(dir.path(), &git, JOB_A);
        let report = check_history(&git, "HEAD").unwrap();
        let details: Vec<&str> = report
            .violations
            .iter()
            .filter(|v| v.rule == Rule::JobRecords && v.commit.as_deref() == Some(&records[..12]))
            .map(|v| v.detail.as_str())
            .collect();
        assert!(
            details
                .iter()
                .any(|d| d.contains("this commit is not a valid landing")),
            "{:?}",
            report.violations
        );
        assert!(
            details.iter().any(|d| d.contains("does not follow")),
            "{:?}",
            report.violations
        );
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
