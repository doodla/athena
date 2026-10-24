//! The legacy rule: main's commits up to and including a cutover commit `C0` are judged by the existing
//! history check, with its exempt list read from the records branch.
//!
//! Design: `docs/design/records-history.md` §4 (the `legacy-exempt` record), §10 (build job 5) and decision
//! D111.
//!
//! **`C0` is a caller-supplied parameter.** Nothing in this module, in the CLI or in CI chooses it; a later
//! job decides which commit is `C0` and passes it in.
//!
//! - [`check_legacy`] runs [`check_history_with_exempt`] on `C0`, the one history check with only its exempt
//!   source replaced: the list comes from `legacy-exempt.json` at the tip of the records branch instead of
//!   from `athena.toml`'s `[history] exempt`. A commit up to `C0` is excused exactly when that record names
//!   it; otherwise it fails exactly as the history check fails it today.
//! - [`check_attribution_with_legacy`] adds that verdict to the part 1 records-only walk, which then judges
//!   only the commits after `C0`. A commit exactly at `C0` is judged by the legacy rule, never by attribution.
//! - `legacy-exempt.json` is append-only (§4, §10): [`RecordsWriter`](athena_git::records_branch::RecordsWriter)
//!   refuses other bytes at an existing path, and the legacy rule reports a records history in which the file
//!   changed ([`FindingKind::LegacyExemptChanged`]).
//! - [`legacy_exempt_record`] builds the record from `athena.toml`'s current `[history] exempt` list and the
//!   reasons given in that file's comments. It writes nothing; [`legacy_exempt_change`] turns the record into
//!   the change a later job will append to the records branch.

use crate::attribution::{self, AttributionReport, Finding, FindingKind};
use crate::history::{HistoryReport, check_history_with_exempt};
use crate::{GateError, Result};
use athena_data::RecordId;
use athena_data::history::{HistoryRecord, LegacyExempt, LegacyExemptEntry};
use athena_git::Git;
use athena_git::records_branch::{self, Change, RecordsTip};
use jiff::Timestamp;

/// Path of the legacy-exempt record on the records branch (§4).
pub const LEGACY_EXEMPT_PATH: &str = "legacy-exempt.json";

/// Why each commit on `athena.toml`'s `[history] exempt` list is there, keyed by the first seven characters
/// of its id. Copied from the comments above that list in `athena.toml`.
const REASONS: [(&str, &str); 7] = [
    (
        "49c6ac1",
        "Predates Athena and is exempt from the governed-content invariant (design section 7.3); \
         athena.toml gives it no comment of its own.",
    ),
    (
        "64ecdc9",
        "A parentless second root that Athena's own commit path wrote during job 20261010-132713-ezm8; it \
         joined main through the merge of the bootstrap (361a786) and fails the records-based landing check.",
    ),
    (
        "ee4b0dd",
        "The squash of PR #8, which carried several jobs' landings in one commit; the landing check refuses \
         that shape. Excused by the user's choice of option (a) in D66: this one commit's content skips the \
         invariant, and its jobs' own records still get the per-record checks.",
    ),
    (
        "f2a8453",
        "The squash of PR #9. Most of that PR's jobs predate athena.candidate/2, so the chain landing rule \
         (D69) cannot link them (D70).",
    ),
    (
        "5a73478",
        "The squash of PR #14. It changes records/ but names no job, so the per-job records check cannot \
         tie its content to a job (D76).",
    ),
    (
        "39962ed",
        "The squash of PR #130. It carried two jobs' landings (20261010-154004-60r4 and 20261017-121143-sg0j) \
         whose chain is broken, so neither landing is valid in that commit. Excused as ee4b0dd (D66) and \
         f2a8453 (D70) were: this one commit's content skips the invariant, and each job's own records still \
         get the per-record checks.",
    ),
    (
        "02837e0",
        "The squash of PR #149 (job 20261018-145008-k7fq), merged by hand around athena land. Its content \
         carries two hand changes beside the job's verified content, so it is not a valid landing. Excused \
         with the user's approval of 2026-10-23 (BRIEF.md addendum of that date) as 39962ed was.",
    ),
];

/// The reason `athena.toml` gives for exempting the commit `sha`, when it is one of the known seven.
pub fn legacy_reason(sha: &str) -> Option<&'static str> {
    REASONS
        .iter()
        .find(|(prefix, _)| sha.starts_with(prefix))
        .map(|(_, reason)| *reason)
}

/// Builds the legacy-exempt record from `exempt`, `athena.toml`'s current `[history] exempt` list, giving
/// each entry its reason from `athena.toml`'s comments. `id` and `created_at` are the caller's. Nothing is
/// written anywhere. An entry with no known reason is an error, never an empty reason.
pub fn legacy_exempt_record(
    exempt: &[String],
    id: RecordId,
    created_at: Timestamp,
) -> Result<LegacyExempt> {
    let mut entries = Vec::new();
    for sha in exempt {
        let reason = legacy_reason(sha).ok_or_else(|| {
            GateError::Config(format!(
                "exempt commit {sha} has no reason in the legacy reason table"
            ))
        })?;
        entries.push(LegacyExemptEntry {
            sha: sha.clone(),
            reason: reason.to_string(),
        });
    }
    let record = LegacyExempt {
        id,
        created_at,
        entries,
    };
    record
        .validate()
        .map_err(|e| GateError::Config(format!("legacy-exempt record is invalid: {e}")))?;
    Ok(record)
}

/// The change that writes `record` to [`LEGACY_EXEMPT_PATH`] on the records branch. It is only built here;
/// applying it to the real branch is a later job's work.
pub fn legacy_exempt_change(record: &LegacyExempt) -> Result<Change> {
    let json =
        serde_json::to_string(&HistoryRecord::LegacyExempt(record.clone())).map_err(|e| {
            GateError::Config(format!("cannot serialise the legacy-exempt record: {e}"))
        })?;
    Ok(Change::put(LEGACY_EXEMPT_PATH, json))
}

/// Reads the legacy-exempt record at `records_tip`. The inner `Err` says why there is none.
fn read_legacy_exempt(
    git: &Git,
    records_tip: &str,
) -> Result<std::result::Result<LegacyExempt, String>> {
    let Some(bytes) = git.show_file(records_tip, LEGACY_EXEMPT_PATH)? else {
        return Ok(Err(format!(
            "the records branch holds no {LEGACY_EXEMPT_PATH}"
        )));
    };
    Ok(
        match HistoryRecord::parse(&String::from_utf8_lossy(&bytes)) {
            Ok(HistoryRecord::LegacyExempt(record)) => Ok(record),
            Ok(other) => Err(format!(
                "{LEGACY_EXEMPT_PATH} holds a {} record, not a legacy-exempt record",
                other.schema()
            )),
            Err(error) => Err(format!(
                "{LEGACY_EXEMPT_PATH} is not a valid record: {error}"
            )),
        },
    )
}

/// The result of the legacy rule.
#[derive(Debug, Default)]
pub struct LegacyReport {
    /// The history check's own report for the commits up to `C0`; `None` when it could not run.
    pub history: Option<HistoryReport>,
    /// The findings: one per history violation, and one for each problem with the record.
    pub findings: Vec<Finding>,
}

/// Applies the legacy rule to the commits up to and including `c0` (§4; D111). `c0` is the caller's.
///
/// The exempt list is read from `legacy-exempt.json` at the records tip. A records branch that is missing
/// or diverged is left for the attribution walk to report, so this returns no findings for it.
pub fn check_legacy(git: &Git, c0: &str) -> Result<LegacyReport> {
    let mut report = LegacyReport::default();
    let records_tip = match records_branch::resolve_tip(git)? {
        RecordsTip::At { tip, .. } => tip,
        RecordsTip::Missing | RecordsTip::Diverged { .. } => return Ok(report),
    };

    for problem in records_branch::audit_history(git, &records_tip)? {
        if problem.detail.contains(LEGACY_EXEMPT_PATH) {
            report.findings.push(Finding {
                kind: FindingKind::LegacyExemptChanged,
                commit: Some(problem.commit.clone()),
                detail: format!(
                    "records commit {} {}: the legacy-exempt record is append-only",
                    problem.commit, problem.detail
                ),
            });
        }
    }

    match read_legacy_exempt(git, &records_tip)? {
        Err(why) => report.findings.push(Finding {
            kind: FindingKind::LegacyExemptMissing,
            commit: None,
            detail: why,
        }),
        Ok(record) => {
            let history = check_history_with_exempt(git, c0, Some(record.shas()))?;
            for violation in &history.violations {
                report.findings.push(Finding {
                    kind: FindingKind::LegacyHistory,
                    commit: violation.commit.clone(),
                    detail: violation.to_string(),
                });
            }
            report.history = Some(history);
        }
    }
    Ok(report)
}

/// The attribution check with the legacy rule: the commits up to and including `c0` go to [`check_legacy`],
/// and the commits after `c0` to the records-only walk of part 1 (§6, §10; D111). `c0` is a parameter the
/// caller supplies; nothing here picks it. `now` bounds the pending window as in
/// [`attribution::check_attribution`].
pub fn check_attribution_with_legacy(
    git: &Git,
    main_tip: &str,
    now: Timestamp,
    c0: &str,
) -> Result<AttributionReport> {
    let legacy = check_legacy(git, c0)?;
    let report = AttributionReport {
        findings: legacy.findings,
        ..AttributionReport::default()
    };
    attribution::walk(git, main_tip, now, Some(c0), report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{commit, repo, write};
    use crate::{Rule, builtin_config};
    use athena_git::records_branch::{RECORDS_REF, RecordsError, RecordsWriter};

    const T0: &str = "2026-10-24T10:00:00Z";

    /// The seven names the request lists, as short ids.
    const NAMES: [&str; 7] = [
        "49c6ac1", "64ecdc9", "ee4b0dd", "f2a8453", "5a73478", "39962ed", "02837e0",
    ];

    fn ts(text: &str) -> Timestamp {
        text.parse().unwrap()
    }

    fn rid(bits: u32) -> RecordId {
        RecordId::from_parts(ts(T0), bits)
    }

    fn entry_record(entries: &[(&str, &str)]) -> LegacyExempt {
        LegacyExempt {
            id: rid(1),
            created_at: ts(T0),
            entries: entries
                .iter()
                .map(|(sha, reason)| LegacyExemptEntry {
                    sha: sha.to_string(),
                    reason: reason.to_string(),
                })
                .collect(),
        }
    }

    fn write_record(git: &Git, record: &LegacyExempt) -> String {
        RecordsWriter::new(git)
            .append("legacy-exempt", &[legacy_exempt_change(record).unwrap()])
            .unwrap()
    }

    /// A root commit and a hand change on top of it: returns `(dir, git, root, hand)`. The hand change
    /// stands for one of the named commits.
    fn world() -> (tempfile::TempDir, Git, String, String) {
        let (dir, git) = repo(&[("README.md", "hello\n")]);
        let root = git.head().unwrap().unwrap();
        write(dir.path(), &[("README.md", "edited by hand\n")]);
        let hand = commit(&git, "hand edit\n");
        (dir, git, root, hand)
    }

    #[test]
    fn each_named_commit_is_excused_when_its_entry_is_present() {
        for name in NAMES {
            let (_d, git, _root, hand) = world();
            let reason = legacy_reason(name).unwrap();
            write_record(&git, &entry_record(&[(&hand, reason)]));
            let report = check_legacy(&git, &hand).unwrap();
            assert!(report.findings.is_empty(), "{name}: {:?}", report.findings);
            let history = report.history.expect("the history check ran");
            assert_eq!(history.violations, vec![], "{name}");
            assert_eq!(history.commits, 2, "{name}");
        }
    }

    #[test]
    fn each_named_commit_fails_as_today_when_its_entry_is_absent() {
        for name in NAMES {
            let (_d, git, root, hand) = world();
            // The record names another commit, not the hand change.
            write_record(
                &git,
                &entry_record(&[(&root, legacy_reason(name).unwrap())]),
            );
            let report = check_legacy(&git, &hand).unwrap();
            let history = report.history.expect("the history check ran");

            // The violations equal those of today's history check, and of one with an empty list.
            let today = crate::history::check_history(&git, &hand).unwrap();
            let empty = check_history_with_exempt(&git, &hand, Some(vec![])).unwrap();
            assert_eq!(history.violations, today.violations, "{name}");
            assert_eq!(history.violations, empty.violations, "{name}");
            assert!(
                history
                    .violations
                    .iter()
                    .any(|v| v.rule == Rule::Invariant && v.commit.as_deref() == Some(&hand[..12])),
                "{name}: {:?}",
                history.violations
            );
            assert_eq!(report.findings.len(), history.violations.len(), "{name}");
            assert!(
                report
                    .findings
                    .iter()
                    .all(|f| f.kind == FindingKind::LegacyHistory)
            );
        }
    }

    #[test]
    fn the_exempt_list_comes_from_the_record_not_from_athena_toml() {
        // The built-in configuration's list names the real commits; a constructed commit is not on it.
        let (_d, git, _root, hand) = world();
        assert!(!builtin_config().history.exempt.contains(&hand));
        assert!(
            !crate::history::check_history(&git, &hand)
                .unwrap()
                .violations
                .is_empty()
        );
    }

    #[test]
    fn a_missing_legacy_exempt_record_is_a_finding() {
        let (_d, git, _root, hand) = world();
        RecordsWriter::new(&git)
            .append("other", &[Change::put("note.json", "{}")])
            .unwrap();
        let report = check_legacy(&git, &hand).unwrap();
        assert!(report.history.is_none());
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].kind, FindingKind::LegacyExemptMissing);
    }

    #[test]
    fn a_legacy_exempt_record_that_gains_an_entry_is_refused() {
        let (_d, git, root, hand) = world();
        let original = entry_record(&[(&hand, "first reason")]);
        let tip = write_record(&git, &original);
        let before = git.show_file(&tip, LEGACY_EXEMPT_PATH).unwrap().unwrap();

        let mut grown = original.clone();
        grown.entries.push(LegacyExemptEntry {
            sha: root,
            reason: "added later".into(),
        });
        let err = RecordsWriter::new(&git)
            .append("grow", &[legacy_exempt_change(&grown).unwrap()])
            .unwrap_err();
        assert!(matches!(err, RecordsError::Refused { .. }), "{err}");

        // Nothing changed: the same tip, the same bytes, the original entries.
        assert_eq!(git.rev_parse(RECORDS_REF).unwrap().as_deref(), Some(&*tip));
        let after = git.show_file(&tip, LEGACY_EXEMPT_PATH).unwrap().unwrap();
        assert_eq!(before, after);
        assert_eq!(read_legacy_exempt(&git, &tip).unwrap().unwrap(), original);
    }

    #[test]
    fn a_records_history_in_which_the_record_changed_is_a_finding() {
        let (_d, git, root, hand) = world();
        let original = entry_record(&[(&hand, "first reason")]);
        let tip = write_record(&git, &original);
        let mut grown = original;
        grown.entries.push(LegacyExemptEntry {
            sha: root,
            reason: "added later".into(),
        });
        // Force the rewrite past the writer: a commit on top of the tip with other bytes at the path.
        let json = serde_json::to_string(&HistoryRecord::LegacyExempt(grown)).unwrap();
        let blob = git.hash_object_w(json.as_bytes()).unwrap();
        let listing = format!("100644 blob {blob}\t{LEGACY_EXEMPT_PATH}\n");
        let mktree = git
            .run_bytes(["mktree"], Some(listing.as_bytes()), &[])
            .unwrap();
        let tree = String::from_utf8(mktree).unwrap().trim().to_string();
        let forced = git.commit_tree(&tree, &[&tip], "forced", false).unwrap();
        git.set_ref(RECORDS_REF, &forced).unwrap();

        let report = check_legacy(&git, &hand).unwrap();
        let finding = report
            .findings
            .iter()
            .find(|f| f.kind == FindingKind::LegacyExemptChanged)
            .unwrap_or_else(|| panic!("{:?}", report.findings));
        assert_eq!(finding.commit.as_deref(), Some(forced.as_str()));
    }

    /// A root, a hand change `c0`, and an unrecorded commit after it: `(dir, git, root, c0, after)`.
    fn world_with_later_commit() -> (tempfile::TempDir, Git, String, String, String) {
        let (dir, git, root, c0) = world();
        write(dir.path(), &[("later.txt", "later\n")]);
        let after = commit(&git, "later commit\n\nAthena-Job: bootstrap\n");
        (dir, git, root, c0, after)
    }

    #[test]
    fn commits_after_c0_are_judged_only_by_records_and_pre_c0_never_by_attribution() {
        let (_d, git, root, c0, after) = world_with_later_commit();
        write_record(&git, &entry_record(&[(&c0, "excused hand change")]));
        let report = check_attribution_with_legacy(&git, &after, ts(T0), &c0).unwrap();
        assert_eq!(report.base.as_deref(), Some(c0.as_str()));
        // Only the commit after C0 is judged, and it is unattributed.
        assert_eq!(report.checked, 1);
        let kinds: Vec<_> = report.findings.iter().map(|f| f.kind).collect();
        assert_eq!(
            kinds,
            vec![FindingKind::Unattributed],
            "{:?}",
            report.findings
        );
        assert_eq!(report.findings[0].commit.as_deref(), Some(after.as_str()));
        // The root and C0 themselves get no attribution finding.
        assert!(
            report
                .findings
                .iter()
                .all(|f| f.commit.as_deref() != Some(root.as_str())
                    && f.commit.as_deref() != Some(c0.as_str()))
        );
    }

    #[test]
    fn a_commit_exactly_at_c0_is_judged_by_the_legacy_rule() {
        // Entry absent: the C0 hand change is a legacy finding and not an attribution finding.
        let (_d, git, root, c0) = world();
        write_record(&git, &entry_record(&[(&root, "some other commit")]));
        let report = check_attribution_with_legacy(&git, &c0, ts(T0), &c0).unwrap();
        assert_eq!(report.checked, 0);
        assert!(!report.passed());
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.kind == FindingKind::LegacyHistory
                    && f.commit.as_deref() == Some(&c0[..12])),
            "{:?}",
            report.findings
        );
        assert!(
            report
                .findings
                .iter()
                .all(|f| f.kind != FindingKind::Unattributed)
        );

        // Entry present: the same commit passes.
        let (_d2, git2, _root2, c02) = world();
        write_record(&git2, &entry_record(&[(&c02, "excused hand change")]));
        let report = check_attribution_with_legacy(&git2, &c02, ts(T0), &c02).unwrap();
        assert!(report.passed(), "{:?}", report.findings);
        assert_eq!(report.checked, 0);
    }

    #[test]
    fn a_cutover_that_is_not_a_commit_is_an_error() {
        let (_d, git, _root, c0) = world();
        write_record(&git, &entry_record(&[(&c0, "excused hand change")]));
        let off = "9".repeat(40);
        let report = check_attribution_with_legacy(&git, &c0, ts(T0), &off).unwrap_err();
        // An id that is not a commit stops the history check with an error rather than passing.
        assert!(report.to_string().contains(&off), "{report}");
    }

    fn builtin_list() -> Vec<String> {
        builtin_config().history.exempt
    }

    #[test]
    fn legacy_exempt_record_holds_the_seven_entries_with_reasons_and_writes_nothing() {
        let (_d, git) = repo(&[("README.md", "hello\n")]);
        let exempt = builtin_list();
        let record = legacy_exempt_record(&exempt, rid(7), ts(T0)).unwrap();
        assert_eq!(record.entries.len(), 7);
        assert_eq!(record.shas(), exempt);
        for (entry, name) in record.entries.iter().zip(NAMES) {
            assert!(entry.sha.starts_with(name), "{} vs {name}", entry.sha);
            assert!(!entry.reason.trim().is_empty(), "{name}");
        }
        record.validate().unwrap();
        // The record round-trips as a record and the change targets the right path.
        match legacy_exempt_change(&record).unwrap() {
            Change::Put { path, bytes } => {
                assert_eq!(path, LEGACY_EXEMPT_PATH);
                let parsed = HistoryRecord::parse(&String::from_utf8(bytes).unwrap()).unwrap();
                assert_eq!(parsed, HistoryRecord::LegacyExempt(record.clone()));
            }
            other => panic!("{other:?}"),
        }
        // Nothing was written to any ref.
        assert_eq!(git.rev_parse(RECORDS_REF).unwrap(), None);
        assert!(matches!(
            records_branch::resolve_tip(&git).unwrap(),
            RecordsTip::Missing
        ));
    }

    #[test]
    fn an_entry_with_no_known_reason_is_an_error() {
        let unknown = vec!["1".repeat(40)];
        assert!(legacy_exempt_record(&unknown, rid(7), ts(T0)).is_err());
        assert!(legacy_exempt_record(&[], rid(7), ts(T0)).is_err());
    }
}
