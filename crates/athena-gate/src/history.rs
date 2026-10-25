//! The full-history check that CI runs (`athena check --history`, design §7.4 and §11.2).
//!
//! It runs gitleaks once over every commit reachable from the tip, under the tip's `[secrets]`, then judges
//! every commit, oldest first, by its parent's rules: the content checks, the governed-content invariant
//! (including the audit of landing commits), the prediction trailers, and the records each commit adds to a
//! job, judged against the rest of the job. The two allow-lists
//! for sensitive content, `[secrets]` and `[personal_data]`, and the `[history] exempt` list come from the
//! tip, where allow-listing an old false positive or excusing old history is legitimate. Finally it validates and reconciles every record at the tip.

use crate::attribution::{self, CutoverState};
use crate::invariant::HistoryState;
use crate::legacy;
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
    /// Things a person should see that do not fail the check, such as a cutover still waiting for its import.
    pub notes: Vec<String>,
}

/// Checks every commit reachable from `tip`.
///
/// This is the records-history cutover's one switch (`docs/design/records-history.md` §4 and §10, D111). With
/// no `[history] cutover_job` at the tip, or before the records branch shows the cutover (the job's landing
/// record **and** an import manifest naming its `C0`), it is [`check_history_with_exempt`] with the tip's own
/// exempt list, exactly as before the cutover. Once the records branch shows the cutover, the commits up to and
/// including `C0` are judged by the old rules (with the exempt list from the `legacy-exempt` record) and the
/// commits after `C0` only by the attribution check against records; commit trailers are no longer read for them.
/// The gitleaks scan still covers every commit reachable from the tip.
pub fn check_history(git: &Git, tip: &str) -> Result<HistoryReport> {
    let Some(resolved) = git.rev_parse(tip)? else {
        return check_history_with_exempt(git, tip, None);
    };
    // A tip whose configuration does not parse is reported by the old check, which judges it.
    let Ok(tip_config) = config_at(git, Some(&resolved)) else {
        return check_history_with_exempt(git, tip, None);
    };
    let Some(job) = tip_config.history.cutover_job.clone() else {
        return check_history_with_exempt(git, tip, None);
    };
    match attribution::cutover_state(git, &job)? {
        CutoverState::Shown { c0 } if git.is_ancestor(&c0, &resolved)? => {
            check_after_cutover(git, &resolved, &c0, &tip_config)
        }
        CutoverState::Pending { c0 } => {
            let mut report = check_history_with_exempt(git, tip, None)?;
            report.notes.push(format!(
                "the cutover is pending: job {job} is landed at {c0}, but the records branch has no import \
                 manifest for it yet, so every commit is still judged by the old rules; `athena status` \
                 imports records at {c0}"
            ));
            Ok(report)
        }
        _ => check_history_with_exempt(git, tip, None),
    }
}

/// How a tip after the cutover relates to main.
enum MainSplit {
    /// The tip is main or part of it: every commit after `C0` is on main and goes to attribution.
    OnMain,
    /// The tip is a head that main does not hold; its commits after `merge_base` are not on main.
    Head { merge_base: String },
    /// Main cannot be told apart from the checkout; why.
    Unknown(String),
}

/// The refs that may name main in a checkout: the remote's, as CI fetches it, then the local branch.
const MAIN_REFS: [&str; 2] = ["refs/remotes/origin/main", "refs/heads/main"];

/// Splits `tip` from main. The tip is on main when it is an ancestor of (or equal to) either main ref; the
/// merge base is taken with the first main ref that exists. A merge base that does not descend from `c0`
/// cannot divide the commits after `c0`, so it is `Unknown`.
fn split_from_main(git: &Git, tip: &str, c0: &str) -> Result<MainSplit> {
    let mut mains = Vec::new();
    for name in MAIN_REFS {
        if let Some(main) = git.rev_parse(name)? {
            mains.push(main);
        }
    }
    let Some(first) = mains.first() else {
        return Ok(MainSplit::Unknown(
            "main is not in this checkout (neither origin/main nor main)".into(),
        ));
    };
    for main in &mains {
        if git.is_ancestor(tip, main)? {
            return Ok(MainSplit::OnMain);
        }
    }
    // `git merge-base` exits non-zero when the two share no history.
    let Some(merge_base) = git
        .run(["merge-base", first, tip])
        .ok()
        .map(|out| out.trim().to_string())
        .filter(|base| !base.is_empty())
    else {
        return Ok(MainSplit::Unknown(
            "the head shares no history with main".into(),
        ));
    };
    if !git.is_ancestor(c0, &merge_base)? {
        return Ok(MainSplit::Unknown(format!(
            "the head's merge base with main ({}) does not descend from {c0}",
            &merge_base[..merge_base.len().min(12)]
        )));
    }
    Ok(MainSplit::Head { merge_base })
}

/// The history check once the cutover is shown: the old rules up to and including `c0`, attribution after
/// for the commits on main, and the pre-merge rule for a head's own commits that main does not hold yet.
fn check_after_cutover(
    git: &Git,
    tip: &str,
    c0: &str,
    tip_config: &athena_data::config::Config,
) -> Result<HistoryReport> {
    let mut report = HistoryReport::default();
    report
        .violations
        .extend(secrets::check_history(git, tip_config, tip)?);
    // A head that is not on main holds commits no landing record can name yet. Attribution judges only the
    // part that is on main (up to the merge base); the head's own commits get the pre-merge rule.
    let (attribution_tip, own_commits) = match split_from_main(git, tip, c0)? {
        MainSplit::OnMain => (tip.to_string(), Vec::new()),
        MainSplit::Unknown(why) => {
            report.notes.push(format!(
                "{why}, so every commit after {c0} is judged by the attribution rule"
            ));
            (tip.to_string(), Vec::new())
        }
        MainSplit::Head { merge_base } => {
            let range = format!("{merge_base}..{tip}");
            let own = git
                .run(["rev-list", "--reverse", &range])?
                .lines()
                .map(str::to_string)
                .collect();
            (merge_base, own)
        }
    };
    for sha in &own_commits {
        if let Err(uncovered) = invariant::classify_pre_merge(git, sha)? {
            report.violations.push(
                Violation::new(
                    Rule::JobRecords,
                    format!(
                        "this commit is on the head but not on main, and {}",
                        uncovered.clause()
                    ),
                    "land verified work through athena land --pr, declare the intervention, or \
                     keep the commit to records only",
                )
                .in_commit(Some(sha)),
            );
        }
    }
    let judged =
        legacy::check_attribution_with_legacy(git, &attribution_tip, jiff::Timestamp::now(), c0)?;
    for finding in judged.findings {
        report.violations.push(
            Violation::new(
                Rule::JobRecords,
                format!("{}: {}", finding.kind.label(), finding.detail),
                "land the change through athena land --pr so a landing record names it, or run \
                 `athena status` to finish the cutover import",
            )
            .in_commit(finding.commit.as_deref()),
        );
    }
    for pending in judged.pending {
        report.notes.push(format!(
            "commit {} is pending: its landing-intent is on the records branch and its landing record is due",
            pending.commit
        ));
    }
    report.commits = git.rev_list_reverse(tip)?.len();
    let records = RecordSet::load(git, tip)?;
    report.records = records.files.len();
    for error in records.errors() {
        report
            .violations
            .push(Violation::new(Rule::ValidRecords, error.to_string(), ""));
    }
    report.violations.extend(reconcile::check_records(&records));
    Ok(report)
}

/// As [`check_history`], with the exempt list taken from `exempt` when it is `Some`, instead of from the
/// tip's `[history] exempt`. This is the one difference the legacy rule needs (`docs/design/records-history.md`
/// §4, D111): it reads the list from a legacy-exempt record on the records branch. Everything else, the
/// tip's other configuration included, is judged exactly as [`check_history`] does.
pub fn check_history_with_exempt(
    git: &Git,
    tip: &str,
    exempt: Option<Vec<String>>,
) -> Result<HistoryReport> {
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
        exempt: exempt.unwrap_or_else(|| tip_config.history.exempt.clone()),
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

#[cfg(test)]
mod cutover_tests {
    use super::*;
    use crate::testutil::*;
    use athena_data::history::{
        HistoryRecord, ImportManifest, ImportedPath, Landing, LandingCommit, LandingIntent,
        LandingKind, LegacyExempt, LegacyExemptEntry, MergeMethod,
    };
    use athena_data::{JobId, RecordId};
    use athena_git::records_branch::{Change as RecordsChange, RecordsWriter};

    const CUTOVER_JOB: &str = "20261025-001232-vesh";
    const T0: &str = "2026-10-25T10:00:00Z";

    fn ts() -> jiff::Timestamp {
        T0.parse().unwrap()
    }

    fn rid(bits: u32) -> RecordId {
        RecordId::from_parts(ts(), bits)
    }

    /// The built-in configuration (which names the cutover job) with `exempt` as its exempt list, and the
    /// `cutover_job` line kept only when `cutover` is true.
    fn config(exempt: &[&str], cutover: bool) -> String {
        let list = exempt
            .iter()
            .map(|e| format!("\"{e}\""))
            .collect::<Vec<_>>()
            .join(", ");
        crate::BUILTIN_CONFIG
            .lines()
            .filter(|l| cutover || !l.starts_with("cutover_job = "))
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

    fn put(git: &Git, path: &str, record: &HistoryRecord) {
        RecordsWriter::new(git)
            .append(
                "record",
                &[RecordsChange::put(
                    path,
                    serde_json::to_string(record).unwrap(),
                )],
            )
            .unwrap();
    }

    /// Writes an intent and a landing for the commit `host`, whose parent is `parent`, naming `job`.
    fn record_landing(git: &Git, n: u32, host: &str, parent: &str, job: &str) {
        let intent = LandingIntent {
            id: rid(2 * n),
            created_at: ts(),
            pr_number: 1,
            head_sha: "1".repeat(40),
            head_tree_sha: git.tree_of(host).unwrap(),
            expected_base: parent.to_string(),
            job_ids: vec![JobId::parse(job).unwrap()],
        };
        put(
            git,
            &format!("landing-intents/{}.json", intent.id),
            &HistoryRecord::LandingIntent(intent.clone()),
        );
        let landing = Landing {
            id: rid(2 * n + 1),
            created_at: ts(),
            pr_number: 1,
            head_sha: "1".repeat(40),
            host_commit: host.to_string(),
            merge_method: MergeMethod::Squash,
            commits: vec![LandingCommit {
                sha: "1".repeat(40),
                tree: "2".repeat(40),
                kind: LandingKind::RecordsOnly,
                job: None,
                verification: None,
            }],
            intent: intent.id,
        };
        put(
            git,
            &format!("landings/{}.json", landing.id),
            &HistoryRecord::Landing(landing),
        );
    }

    fn put_manifest(git: &Git, c0: &str) {
        let manifest = ImportManifest {
            id: rid(90),
            created_at: ts(),
            c0: c0.to_string(),
            paths: vec![ImportedPath {
                path: "records/x.json".into(),
                blob_id: "3".repeat(40),
                sha256: "4".repeat(64),
            }],
        };
        put(
            git,
            "import-manifest.json",
            &HistoryRecord::ImportManifest(manifest),
        );
    }

    fn put_exempt(git: &Git, shas: &[&str]) {
        let record = LegacyExempt {
            id: rid(91),
            created_at: ts(),
            entries: shas
                .iter()
                .map(|sha| LegacyExemptEntry {
                    sha: sha.to_string(),
                    reason: "predates the cutover".into(),
                })
                .collect(),
        };
        put(
            git,
            "legacy-exempt.json",
            &HistoryRecord::LegacyExempt(record),
        );
    }

    struct World {
        _dir: tempfile::TempDir,
        git: Git,
        /// A hand edit before `C0`, with no trailer.
        hand: String,
        c0: String,
        /// A commit after `C0`, with no trailer.
        post: String,
    }

    /// root, a trailerless hand edit, `C0` (a bootstrap commit), a trailerless commit after `C0`. No records
    /// branch yet.
    fn world(cutover_in_config: bool) -> World {
        let builtin = crate::builtin_config();
        let (dir, git) = repo(&[
            (
                "athena.toml",
                &config(&[&builtin.history.exempt[0]], cutover_in_config),
            ),
            ("README.md", "hello\n"),
        ]);
        write(dir.path(), &[("README.md", "edited by hand\n")]);
        let hand = commit(&git, "hand edit\n");
        write(dir.path(), &[("src/lib.rs", "pub fn f() {}\n")]);
        let c0 = commit(&git, "cutover\n\nAthena-Job: bootstrap\n");
        write(dir.path(), &[("src/lib.rs", "pub fn f() { 1; }\n")]);
        let post = commit(&git, "later work without a trailer\n");
        World {
            _dir: dir,
            git,
            hand,
            c0,
            post,
        }
    }

    /// The cutover landing, its import and the legacy-exempt record, excusing `exempt`.
    fn record_cutover(world: &World, exempt: &[&str]) {
        let parent = world.git.parents(&world.c0).unwrap()[0].clone();
        record_landing(&world.git, 1, &world.c0, &parent, CUTOVER_JOB);
        put_manifest(&world.git, &world.c0);
        put_exempt(&world.git, exempt);
    }

    fn flagged(report: &HistoryReport, commit: &str) -> Vec<String> {
        report
            .violations
            .iter()
            .filter(|v| v.commit.as_deref() == Some(&commit[..12]))
            .map(|v| v.detail.clone())
            .collect()
    }

    #[test]
    fn a_commit_before_c0_is_judged_by_the_old_rules_and_one_after_only_by_records() {
        let w = world(true);
        record_cutover(&w, &[&w.hand]);
        record_landing(&w.git, 2, &w.post, &w.c0, "20261025-120000-aaaa");

        // Today's rules would fail the trailerless commit after C0.
        let old = check_history_with_exempt(&w.git, "HEAD", None).unwrap();
        assert!(
            !flagged(&old, &w.post).is_empty(),
            "the old rules flag the later commit: {:?}",
            old.violations
        );

        // With the cutover shown, it passes: a landing record names it and it has no trailer. The hand edit
        // before C0 is excused by the legacy-exempt record.
        let report = check_history(&w.git, "HEAD").unwrap();
        assert_eq!(report.violations, vec![], "{:?}", report.violations);
        assert_eq!(report.commits, 4);
        assert!(report.notes.is_empty(), "{:?}", report.notes);
    }

    #[test]
    fn a_commit_before_c0_that_the_legacy_exempt_record_does_not_name_is_flagged_by_the_old_rules()
    {
        let w = world(true);
        record_cutover(&w, &["1".repeat(40).as_str()]);
        record_landing(&w.git, 2, &w.post, &w.c0, "20261025-120000-aaaa");
        let report = check_history(&w.git, "HEAD").unwrap();
        let found = flagged(&report, &w.hand);
        assert!(
            found.iter().any(|d| d.contains("legacy history check")),
            "{:?}",
            report.violations
        );
        assert!(flagged(&report, &w.post).is_empty());
        assert!(flagged(&report, &w.c0).is_empty());
    }

    #[test]
    fn a_commit_after_c0_that_no_landing_names_is_flagged_by_attribution() {
        let w = world(true);
        record_cutover(&w, &[&w.hand]);
        let report = check_history(&w.git, "HEAD").unwrap();
        let found = flagged(&report, &w.post);
        assert!(
            found.iter().any(|d| d.contains("unattributed")),
            "{:?}",
            report.violations
        );
        assert!(flagged(&report, &w.hand).is_empty());
    }

    /// The old check's verdict, to compare the fallbacks with.
    fn same_as_today(git: &Git) -> HistoryReport {
        let routed = check_history(git, "HEAD").unwrap();
        let today = check_history_with_exempt(git, "HEAD", None).unwrap();
        assert_eq!(routed.violations, today.violations);
        assert_eq!(
            (routed.commits, routed.records),
            (today.commits, today.records)
        );
        assert!(
            !today.violations.is_empty(),
            "the fixture breaks the old rules"
        );
        routed
    }

    #[test]
    fn with_no_cutover_job_the_check_is_unchanged_even_when_records_show_a_landing() {
        let w = world(false);
        record_cutover(&w, &[&w.hand]);
        let report = same_as_today(&w.git);
        assert!(report.notes.is_empty());
    }

    #[test]
    fn before_the_landing_record_exists_the_check_is_unchanged() {
        // No records branch at all.
        let w = world(true);
        let report = same_as_today(&w.git);
        assert!(report.notes.is_empty());

        // A records branch with a landing for some other job only.
        let parent = w.git.parents(&w.c0).unwrap()[0].clone();
        record_landing(&w.git, 3, &w.c0, &parent, "20261025-130000-bbbb");
        let report = same_as_today(&w.git);
        assert!(report.notes.is_empty());
    }

    #[test]
    fn a_landing_without_its_import_keeps_the_old_rules_and_says_the_cutover_is_pending() {
        let w = world(true);
        let parent = w.git.parents(&w.c0).unwrap()[0].clone();
        record_landing(&w.git, 1, &w.c0, &parent, CUTOVER_JOB);
        let report = same_as_today(&w.git);
        assert_eq!(report.notes.len(), 1, "{:?}", report.notes);
        assert!(report.notes[0].contains("pending"), "{:?}", report.notes);
        assert!(report.notes[0].contains(&w.c0), "{:?}", report.notes);

        // A manifest naming another commit does not show the cutover either.
        put_manifest(&w.git, &w.post);
        let report = same_as_today(&w.git);
        assert_eq!(report.notes.len(), 1);
    }

    #[test]
    fn cutover_state_reads_the_landing_and_the_manifest() {
        let w = world(true);
        assert_eq!(
            attribution::cutover_state(&w.git, CUTOVER_JOB).unwrap(),
            CutoverState::NotLanded
        );
        let parent = w.git.parents(&w.c0).unwrap()[0].clone();
        record_landing(&w.git, 1, &w.c0, &parent, CUTOVER_JOB);
        assert_eq!(
            attribution::cutover_commit(&w.git, CUTOVER_JOB).unwrap(),
            Some(w.c0.clone())
        );
        assert_eq!(
            attribution::cutover_commit(&w.git, "20261025-130000-bbbb").unwrap(),
            None,
            "a landing of another job is not the cutover"
        );
        assert_eq!(
            attribution::cutover_state(&w.git, CUTOVER_JOB).unwrap(),
            CutoverState::Pending { c0: w.c0.clone() }
        );
        put_manifest(&w.git, &w.c0);
        assert_eq!(
            attribution::cutover_state(&w.git, CUTOVER_JOB).unwrap(),
            CutoverState::Shown { c0: w.c0.clone() }
        );
    }
}
