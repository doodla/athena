//! The attribution check: every commit on main is named by a landing record, from records alone.
//!
//! Design: `docs/design/records-history.md` §5.2 (the gap and the pending state), §6 (attribution of every
//! commit on main, from records alone) and §10 (build job 5), decision D111.
//!
//! The check reads the records branch `athena/records` and main's first-parent graph, nothing else. It never
//! reads a commit message or a trailer (§5.3: "the check never reads a commit message for attribution"), and
//! it makes no call to the host.
//!
//! It walks main's first-parent commits, oldest first, starting from the commit the **earliest
//! landing-intent** expects as its base, up to the tip that is checked. That base is the anchor, as `C0` is
//! in §6 ("after `C0`"): it is where the walk starts and is not itself judged, because it was made before
//! any intent announced it. Commits up to and including it are out of scope here: the old history check
//! judges them, and the legacy rule will later. For each commit `M` after the base, with first parent `P`:
//!
//! - **Attributed** when a landing record names `M` as its `host_commit` and the intent that landing cites
//!   matches `M`: the intent's head tree equals `tree(M)` and its expected base equals `P`
//!   ([`LandingIntent::matches`], the one rule shared with landing; §6 squash rule).
//! - **Pending** (§5.2) when no landing names `M` but some landing-intent matches it and the check time is
//!   at most [`PENDING_WINDOW`] after that intent's creation. Pending commits pass, with a notice.
//! - Otherwise a **finding**: a landing that names `M` but cites no matching intent, a matching intent whose
//!   window has run out, or a commit nothing names (unattributed).
//!
//! The check time is an argument, so a test controls the 15-minute window.
//!
//! It also audits the records branch itself ([`athena_git::records_branch::audit_history`]): a records commit
//! that is not a fast-forward of its predecessor is a finding, and so is a local `athena/records` and a
//! remote-tracking ref that have diverged. A missing branch or a branch with no landing-intent is an
//! explicit finding, never a silent pass.

use crate::Result;
use athena_data::history::{HistoryRecord, Landing, LandingIntent};
use athena_git::Git;
use athena_git::records_branch::{self, RecordsTip};
use jiff::{SignedDuration, Timestamp};

/// How long a commit with a matching landing-intent and no landing record may stay pending (§5.2, decided
/// in the 2026-10-23 addenda of `docs/design/records-history.md`; D111).
pub const PENDING_WINDOW: SignedDuration = SignedDuration::from_mins(15);

/// Directory of landing-intent files on the records branch.
const INTENT_DIR: &str = "landing-intents";
/// Directory of landing files on the records branch.
const LANDING_DIR: &str = "landings";

/// What kind of problem a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingKind {
    /// Neither `athena/records` nor its remote-tracking ref exists.
    RecordsMissing,
    /// The local and remote-tracking records refs have diverged.
    RecordsDiverged,
    /// A records commit is not a fast-forward of its predecessor (§10).
    RecordsHistory,
    /// A file in the records directories is not a valid record of the expected kind.
    BadRecord,
    /// The records branch has no landing-intent, so there is no base to start from.
    NoIntent,
    /// The earliest intent's expected base is not on main's first-parent line.
    BaseNotOnMain,
    /// A main commit no landing record names, with no matching intent (§5.2, §6).
    Unattributed,
    /// A landing names the commit, but the intent it cites is missing or does not match (§6).
    LandingMismatch,
    /// A matching intent exists, no landing came, and the 15-minute window ran out (§5.2).
    PendingExpired,
}

impl FindingKind {
    /// A short label for output.
    pub fn label(self) -> &'static str {
        match self {
            FindingKind::RecordsMissing => "records branch missing",
            FindingKind::RecordsDiverged => "records branch diverged",
            FindingKind::RecordsHistory => "records history not a fast-forward",
            FindingKind::BadRecord => "bad record",
            FindingKind::NoIntent => "no landing-intent",
            FindingKind::BaseNotOnMain => "base not on main",
            FindingKind::Unattributed => "unattributed",
            FindingKind::LandingMismatch => "landing does not match",
            FindingKind::PendingExpired => "pending window expired",
        }
    }
}

/// One problem the check found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub kind: FindingKind,
    /// The full sha of the commit the finding is about (a main commit, or a records commit for
    /// [`FindingKind::RecordsHistory`] and [`FindingKind::RecordsDiverged`]).
    pub commit: Option<String>,
    pub detail: String,
}

/// A main commit that passes for now because its landing is still due (§5.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pending {
    pub commit: String,
    /// When the matching intent was written.
    pub intent_at: Timestamp,
}

/// The result of the attribution check.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttributionReport {
    /// The commit the walk started from (the earliest intent's expected base).
    pub base: Option<String>,
    /// How many main commits were judged.
    pub checked: usize,
    /// The commits named by a matching landing record.
    pub attributed: Vec<String>,
    /// The commits still within the pending window.
    pub pending: Vec<Pending>,
    pub findings: Vec<Finding>,
}

impl AttributionReport {
    /// Whether the check passed: no findings. Pending commits do not fail it.
    pub fn passed(&self) -> bool {
        self.findings.is_empty()
    }

    fn find(&mut self, kind: FindingKind, commit: Option<&str>, detail: impl Into<String>) {
        self.findings.push(Finding {
            kind,
            commit: commit.map(str::to_string),
            detail: detail.into(),
        });
    }
}

/// Reads the records of kind `dir` on `tip`: every `.json` file under it, parsed.
fn read_records(
    git: &Git,
    tip: &str,
    dir: &str,
    report: &mut AttributionReport,
) -> Result<Vec<HistoryRecord>> {
    let entries: Vec<_> = git
        .ls_tree_under(tip, dir)?
        .into_iter()
        .filter(|e| e.path.ends_with(".json"))
        .collect();
    let oids: Vec<String> = entries.iter().map(|e| e.oid.clone()).collect();
    let blobs = git.cat_blobs(&oids)?;
    let mut records = Vec::new();
    for (entry, bytes) in entries.iter().zip(blobs) {
        match HistoryRecord::parse(&String::from_utf8_lossy(&bytes)) {
            Ok(record) => records.push(record),
            Err(error) => report.find(
                FindingKind::BadRecord,
                None,
                format!("{} is not a valid record: {error}", entry.path),
            ),
        }
    }
    Ok(records)
}

/// Runs the attribution check on main's first-parent line ending at `main_tip`, judging the pending window
/// against `now` (§5.2, §6, §10; D111). Only records and the commit graph are read.
pub fn check_attribution(git: &Git, main_tip: &str, now: Timestamp) -> Result<AttributionReport> {
    let mut report = AttributionReport::default();

    let records_tip = match records_branch::resolve_tip(git)? {
        RecordsTip::Missing => {
            report.find(
                FindingKind::RecordsMissing,
                None,
                "neither the branch athena/records nor origin/athena/records exists; fetch it with \
                 `git fetch origin athena/records:athena/records`",
            );
            return Ok(report);
        }
        RecordsTip::Diverged { local, tracking } => {
            report.find(
                FindingKind::RecordsDiverged,
                Some(&local),
                format!(
                    "athena/records at {local} and origin/athena/records at {tracking} have diverged: \
                     neither descends from the other"
                ),
            );
            return Ok(report);
        }
        RecordsTip::At { tip, .. } => tip,
    };

    for problem in records_branch::audit_history(git, &records_tip)? {
        report.find(
            FindingKind::RecordsHistory,
            Some(&problem.commit),
            format!(
                "records commit {} is not a fast-forward of its predecessor: it {}",
                problem.commit, problem.detail
            ),
        );
    }

    let mut intents: Vec<LandingIntent> = Vec::new();
    for record in read_records(git, &records_tip, INTENT_DIR, &mut report)? {
        if let HistoryRecord::LandingIntent(intent) = record {
            intents.push(intent);
        }
    }
    let mut landings: Vec<Landing> = Vec::new();
    for record in read_records(git, &records_tip, LANDING_DIR, &mut report)? {
        if let HistoryRecord::Landing(landing) = record {
            landings.push(landing);
        }
    }

    let Some(first) = intents
        .iter()
        .min_by(|a, b| (a.created_at, a.id.as_str()).cmp(&(b.created_at, b.id.as_str())))
    else {
        report.find(
            FindingKind::NoIntent,
            None,
            "the records branch holds no landing-intent, so there is no base to walk from",
        );
        return Ok(report);
    };
    let base = first.expected_base.clone();
    report.base = Some(base.clone());

    // Main's first-parent line, newest first, each with its first parent.
    let listing = git.run(["rev-list", "--first-parent", "--parents", main_tip])?;
    let mut line: Vec<(String, Option<String>)> = listing
        .lines()
        .filter_map(|l| {
            let mut ids = l.split_whitespace();
            let sha = ids.next()?.to_string();
            Some((sha, ids.next().map(str::to_string)))
        })
        .collect();
    line.reverse();
    let Some(start) = line.iter().position(|(sha, _)| *sha == base) else {
        report.find(
            FindingKind::BaseNotOnMain,
            Some(&base),
            format!("the earliest landing-intent expects base {base}, which is not on main's first-parent line"),
        );
        return Ok(report);
    };

    for (sha, parent) in &line[start + 1..] {
        report.checked += 1;
        let tree = git.tree_of(sha)?;
        let parent = parent.as_deref().unwrap_or("");
        let matching = |intent: &&LandingIntent| intent.matches(&tree, parent);

        let naming: Vec<&Landing> = landings.iter().filter(|l| l.host_commit == *sha).collect();
        if !naming.is_empty() {
            let fulfilled = naming.iter().any(|landing| {
                intents
                    .iter()
                    .filter(|i| i.id == landing.intent)
                    .any(|i| i.matches(&tree, parent))
            });
            if fulfilled {
                report.attributed.push(sha.clone());
            } else {
                report.find(
                    FindingKind::LandingMismatch,
                    Some(sha),
                    format!(
                        "commit {sha} is named by landing {}, but the intent it cites ({}) is missing or does not \
                         match: the intent's head tree must equal tree {tree} and its expected base must \
                         equal the commit's parent {parent}",
                        naming[0].id, naming[0].intent
                    ),
                );
            }
            continue;
        }

        let candidates: Vec<&LandingIntent> = intents.iter().filter(matching).collect();
        // The newest matching intent decides; an intent dated after `now` is still pending.
        match candidates.iter().max_by_key(|i| i.created_at) {
            None => report.find(
                FindingKind::Unattributed,
                Some(sha),
                format!(
                    "commit {sha} is unattributed: no landing record names it and no landing-intent \
                     matches its tree {tree} and parent {parent}"
                ),
            ),
            Some(intent) => {
                if now.duration_since(intent.created_at) <= PENDING_WINDOW {
                    report.pending.push(Pending {
                        commit: sha.clone(),
                        intent_at: intent.created_at,
                    });
                } else {
                    report.find(
                        FindingKind::PendingExpired,
                        Some(sha),
                        format!(
                            "commit {sha} is unattributed: landing-intent {} (written {}) matches it, but no \
                             landing record came within {} minutes",
                            intent.id,
                            intent.created_at,
                            PENDING_WINDOW.as_secs() / 60
                        ),
                    );
                }
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{commit, repo, write};
    use athena_data::history::{LandingCommit, LandingKind, MergeMethod};
    use athena_data::{JobId, RecordId};
    use athena_git::records_branch::{Change, RECORDS_REF, RECORDS_TRACKING_REF, RecordsWriter};

    const T0: &str = "2026-10-24T10:00:00Z";

    fn ts(text: &str) -> Timestamp {
        text.parse().unwrap()
    }

    fn rid(bits: u32) -> RecordId {
        RecordId::from_parts(ts(T0), bits)
    }

    fn put(git: &Git, path: &str, record: &HistoryRecord) -> String {
        RecordsWriter::new(git)
            .append(
                "record",
                &[Change::put(path, serde_json::to_string(record).unwrap())],
            )
            .unwrap()
    }

    fn intent(id: RecordId, at: &str, tree: &str, base: &str) -> LandingIntent {
        LandingIntent {
            id,
            created_at: ts(at),
            pr_number: 1,
            head_sha: "1".repeat(40),
            head_tree_sha: tree.to_string(),
            expected_base: base.to_string(),
            job_ids: vec![JobId::parse("20261024-100000-aaaa").unwrap()],
        }
    }

    fn put_intent(git: &Git, i: &LandingIntent) {
        let path = format!("{INTENT_DIR}/{}.json", i.id);
        put(git, &path, &HistoryRecord::LandingIntent(i.clone()));
    }

    fn put_landing(git: &Git, id: RecordId, host: &str, intent: &RecordId) {
        let landing = Landing {
            id: id.clone(),
            created_at: ts(T0),
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
            intent: intent.clone(),
        };
        put(
            git,
            &format!("{LANDING_DIR}/{id}.json"),
            &HistoryRecord::Landing(landing),
        );
    }

    /// A repository with a base commit and a second commit, the "squash". Returns
    /// `(dir, git, base, squash, squash_tree)`.
    fn world() -> (tempfile::TempDir, Git, String, String, String) {
        let (dir, git) = repo(&[("a.txt", "a\n")]);
        let base = git.head().unwrap().unwrap();
        write(dir.path(), &[("b.txt", "b\n")]);
        let squash = commit(&git, "Squash of a pull request");
        let tree = git.tree_of(&squash).unwrap();
        (dir, git, base, squash, tree)
    }

    #[test]
    fn a_properly_landed_squash_passes() {
        let (_d, git, base, squash, tree) = world();
        let i = intent(rid(1), T0, &tree, &base);
        put_intent(&git, &i);
        put_landing(&git, rid(2), &squash, &i.id);
        let report = check_attribution(&git, &squash, ts("2026-10-24T10:01:00Z")).unwrap();
        assert!(report.findings.is_empty(), "{:?}", report.findings);
        assert!(report.passed());
        assert_eq!(report.attributed, vec![squash]);
        assert_eq!(report.base.as_deref(), Some(base.as_str()));
        assert!(report.pending.is_empty());
    }

    #[test]
    fn an_unrecorded_commit_fails_and_is_named() {
        let (dir, git, base, squash, tree) = world();
        let i = intent(rid(1), T0, &tree, &base);
        put_intent(&git, &i);
        put_landing(&git, rid(2), &squash, &i.id);
        write(dir.path(), &[("c.txt", "c\n")]);
        let rogue = commit(&git, "Hand-made commit\n\nAthena-Job: bootstrap\n");
        let report = check_attribution(&git, &rogue, ts("2026-10-24T10:01:00Z")).unwrap();
        assert!(!report.passed());
        let finding = report
            .findings
            .iter()
            .find(|f| f.kind == FindingKind::Unattributed)
            .unwrap_or_else(|| panic!("{:?}", report.findings));
        assert_eq!(finding.commit.as_deref(), Some(rogue.as_str()));
        assert!(finding.detail.contains(&rogue), "{}", finding.detail);
        assert_eq!(report.attributed, vec![squash]);
    }

    #[test]
    fn a_trailer_does_not_attribute_a_commit() {
        // The check reads no commit message: a commit claiming a job is still unattributed.
        let (dir, git, base, squash, tree) = world();
        put_intent(&git, &intent(rid(1), T0, &tree, &base));
        put_landing(&git, rid(2), &squash, &rid(1));
        write(dir.path(), &[("c.txt", "c\n")]);
        let claimed = commit(
            &git,
            "Claims\n\nAthena-Job: 20261024-100000-aaaa\nAthena-Verified-Content: x\n",
        );
        let report = check_attribution(&git, &claimed, ts(T0)).unwrap();
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.kind == FindingKind::Unattributed
                    && f.commit.as_deref() == Some(claimed.as_str()))
        );
    }

    #[test]
    fn a_pending_commit_passes_within_fifteen_minutes_and_fails_after() {
        let (_d, git, base, squash, tree) = world();
        put_intent(&git, &intent(rid(1), T0, &tree, &base));
        let at = |t: &str| check_attribution(&git, &squash, ts(t)).unwrap();

        let early = at("2026-10-24T10:14:00Z");
        assert_eq!(
            early.pending,
            vec![Pending {
                commit: squash.clone(),
                intent_at: ts(T0)
            }]
        );
        assert!(
            !early
                .findings
                .iter()
                .any(|f| f.commit.as_deref() == Some(squash.as_str())),
            "{:?}",
            early.findings
        );
        let edge = at("2026-10-24T10:15:00Z");
        assert_eq!(edge.pending.len(), 1);
        let late = at("2026-10-24T10:16:00Z");
        assert!(late.pending.is_empty());
        let finding = late
            .findings
            .iter()
            .find(|f| f.kind == FindingKind::PendingExpired)
            .unwrap_or_else(|| panic!("{:?}", late.findings));
        assert_eq!(finding.commit.as_deref(), Some(squash.as_str()));
        assert!(!late.passed());
    }

    #[test]
    fn a_landing_whose_intent_does_not_match_is_a_finding() {
        let (_d, git, base, squash, tree) = world();
        // The intent announced a different tree.
        let i = intent(rid(1), T0, &"3".repeat(40), &base);
        put_intent(&git, &i);
        put_landing(&git, rid(2), &squash, &i.id);
        let report = check_attribution(&git, &squash, ts(T0)).unwrap();
        let f = report
            .findings
            .iter()
            .find(|f| f.kind == FindingKind::LandingMismatch)
            .unwrap_or_else(|| panic!("{:?}", report.findings));
        assert_eq!(f.commit.as_deref(), Some(squash.as_str()));
        assert!(f.detail.contains(&tree));

        // A landing that cites an intent that does not exist.
        let (_d2, git2, base2, squash2, tree2) = world();
        put_intent(&git2, &intent(rid(1), T0, &tree2, &base2));
        put_landing(&git2, rid(2), &squash2, &rid(9));
        let report = check_attribution(&git2, &squash2, ts(T0)).unwrap();
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.kind == FindingKind::LandingMismatch)
        );
    }

    #[test]
    fn commits_before_the_first_intents_base_are_not_judged() {
        let (dir, git) = repo(&[("a.txt", "a\n")]);
        let old = git.head().unwrap().unwrap();
        write(dir.path(), &[("old.txt", "old\n")]);
        let unrecorded_old = commit(&git, "Old, unrecorded");
        write(dir.path(), &[("b.txt", "b\n")]);
        let squash = commit(&git, "Squash");
        let tree = git.tree_of(&squash).unwrap();
        let i = intent(rid(1), T0, &tree, &unrecorded_old);
        put_intent(&git, &i);
        put_landing(&git, rid(2), &squash, &i.id);
        // Neither the unrecorded base nor the commit before it is judged.
        let report = check_attribution(&git, &squash, ts(T0)).unwrap();
        assert_eq!(report.checked, 1);
        assert!(report.passed(), "{:?}", report.findings);
        assert_eq!(report.attributed, vec![squash]);
        let _ = old;
    }

    #[test]
    fn a_missing_branch_no_intent_and_an_off_main_base_are_explicit_findings() {
        let (_d, git, base, squash, tree) = world();
        let report = check_attribution(&git, &squash, ts(T0)).unwrap();
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].kind, FindingKind::RecordsMissing);

        // A records branch with only a landing: no intent.
        put_landing(&git, rid(2), &squash, &rid(1));
        let report = check_attribution(&git, &squash, ts(T0)).unwrap();
        assert_eq!(report.findings[0].kind, FindingKind::NoIntent);

        // An intent whose base is not on main.
        put_intent(&git, &intent(rid(1), T0, &tree, &"9".repeat(40)));
        let report = check_attribution(&git, &squash, ts(T0)).unwrap();
        assert_eq!(report.findings[0].kind, FindingKind::BaseNotOnMain);
        let _ = base;
    }

    #[test]
    fn an_invalid_record_file_is_a_finding() {
        let (_d, git, base, squash, tree) = world();
        put_intent(&git, &intent(rid(1), T0, &tree, &base));
        RecordsWriter::new(&git)
            .append("junk", &[Change::put("landings/junk.json", "{}")])
            .unwrap();
        let report = check_attribution(&git, &squash, ts(T0)).unwrap();
        assert!(
            report
                .findings
                .iter()
                .any(|f| f.kind == FindingKind::BadRecord && f.detail.contains("junk.json")),
            "{:?}",
            report.findings
        );
    }

    #[test]
    fn a_records_commit_that_is_not_a_fast_forward_is_reported() {
        let (_d, git, base, squash, tree) = world();
        let i = intent(rid(1), T0, &tree, &base);
        put_intent(&git, &i);
        let intent_tip = git.rev_parse(RECORDS_REF).unwrap().unwrap();
        put_landing(&git, rid(2), &squash, &i.id);
        let landing_tip = git.rev_parse(RECORDS_REF).unwrap().unwrap();
        // A rewrite: a commit on top of the landing whose tree is the older tree, dropping the landing.
        let older = git.tree_of(&intent_tip).unwrap();
        let rewrite = git
            .commit_tree(&older, &[&landing_tip], "rewrite", false)
            .unwrap();
        git.set_ref(RECORDS_REF, &rewrite).unwrap();
        let report = check_attribution(&git, &squash, ts(T0)).unwrap();
        let f = report
            .findings
            .iter()
            .find(|f| f.kind == FindingKind::RecordsHistory)
            .unwrap_or_else(|| panic!("{:?}", report.findings));
        assert_eq!(f.commit.as_deref(), Some(rewrite.as_str()));

        // A normal history reports nothing of the kind.
        git.set_ref(RECORDS_REF, &landing_tip).unwrap();
        let report = check_attribution(&git, &squash, ts(T0)).unwrap();
        assert!(report.passed(), "{:?}", report.findings);
    }

    #[test]
    fn diverged_local_and_tracking_refs_are_reported() {
        let (_d, git, base, squash, tree) = world();
        put_intent(&git, &intent(rid(1), T0, &tree, &base));
        let empty = git.empty_tree().unwrap();
        let rival = git.commit_tree(&empty, &[], "rival", false).unwrap();
        git.set_ref(RECORDS_TRACKING_REF, &rival).unwrap();
        let report = check_attribution(&git, &squash, ts(T0)).unwrap();
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].kind, FindingKind::RecordsDiverged);
    }

    #[test]
    fn the_tracking_ref_alone_is_read() {
        let (_d, git, base, squash, tree) = world();
        let i = intent(rid(1), T0, &tree, &base);
        put_intent(&git, &i);
        put_landing(&git, rid(2), &squash, &i.id);
        let tip = git.rev_parse(RECORDS_REF).unwrap().unwrap();
        git.set_ref(RECORDS_TRACKING_REF, &tip).unwrap();
        git.run(["update-ref", "-d", RECORDS_REF]).unwrap();
        let report = check_attribution(&git, &squash, ts(T0)).unwrap();
        assert!(report.passed(), "{:?}", report.findings);
        assert_eq!(report.attributed, vec![squash]);
    }
}
