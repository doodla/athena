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
//! any intent announced it. Commits up to and including it are out of scope for this walk. For each
//! commit `M` after the base, with first parent `P`:
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
//!
//! # The legacy rule
//!
//! [`crate::legacy::check_attribution_with_legacy`] adds the legacy rule (§4, §10 build job 5; D111). Given
//! a cutover commit `C0`, supplied by the caller (nothing here chooses it), the commits up to and
//! including `C0` are judged by the existing history check with its exempt list read from the
//! `legacy-exempt.json` record on the records branch, and never by attribution. The walk above then
//! starts after `C0` instead of after the earliest intent's base, and judges only the commits after it,
//! by records alone. [`check_attribution`] without a `C0` is unchanged.

use crate::Result;
use athena_data::RecordId;
use athena_data::history::{Checkpoint, HistoryRecord, Landing, LandingIntent};
use athena_git::Git;
use athena_git::records_branch::{self, Change, RECORDS_REF, RecordsTip, RecordsWriter};
use jiff::{SignedDuration, Timestamp};
use std::time::Instant;

/// How long a commit with a matching landing-intent and no landing record may stay pending (§5.2, decided
/// in the 2026-10-23 addenda of `docs/design/records-history.md`; D111).
pub const PENDING_WINDOW: SignedDuration = SignedDuration::from_mins(15);

/// Directory of landing-intent files on the records branch.
const INTENT_DIR: &str = "landing-intents";
/// Directory of landing files on the records branch.
const LANDING_DIR: &str = "landings";
/// Directory of checkpoint files on the records branch (§9).
pub const CHECKPOINT_DIR: &str = "checkpoints";

/// The version of this check (§9; D111). A checkpoint records the version that produced it, and a check whose
/// version differs ignores that checkpoint and runs a full replay. Change it whenever the judging rules do.
pub const CHECK_VERSION: &str = "attribution-check/1";

/// Where the importer writes its manifest, at the root of the records branch (`records_import` in
/// `athena-core`). The gate cannot depend on that crate, so the name is repeated here.
pub const IMPORT_MANIFEST_PATH: &str = "import-manifest.json";

/// How far the records-history cutover has got, as the records branch shows it (D111).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CutoverState {
    /// No records branch (or a diverged one), or no landing record for the cutover job: the cutover has not
    /// happened and every commit is judged by the old rules.
    NotLanded,
    /// The cutover job's landing record names `c0`, but the records branch holds no import manifest naming
    /// `c0` yet. The import is still due, so the old rules still apply.
    Pending { c0: String },
    /// The landing record names `c0` and the import manifest naming `c0` is on the records branch.
    Shown { c0: String },
}

/// The host commit of the landing record that lands `job` at the records tip, which is `C0` when `job` is the
/// cutover job. A landing lands `job` when one of its commits names the job, or the intent it cites does. With
/// several such landings the earliest decides. `None` when there is no records branch or no such landing.
pub fn cutover_commit(git: &Git, job: &str) -> Result<Option<String>> {
    let RecordsTip::At { tip, .. } = records_branch::resolve_tip(git)? else {
        return Ok(None);
    };
    cutover_commit_at(git, &tip, job)
}

/// As [`cutover_commit`], reading the records commit `tip` instead of resolving the branch.
pub fn cutover_commit_at(git: &Git, tip: &str, job: &str) -> Result<Option<String>> {
    let mut scratch = AttributionReport::default();
    let intents: Vec<LandingIntent> = read_records(git, tip, INTENT_DIR, &mut scratch)?
        .into_iter()
        .filter_map(|r| match r {
            HistoryRecord::LandingIntent(i) => Some(i),
            _ => None,
        })
        .collect();
    let mut found: Vec<Landing> = read_records(git, tip, LANDING_DIR, &mut scratch)?
        .into_iter()
        .filter_map(|r| match r {
            HistoryRecord::Landing(l) => Some(l),
            _ => None,
        })
        .filter(|landing| {
            landing
                .commits
                .iter()
                .any(|c| c.job.as_ref().is_some_and(|j| j.as_str() == job))
                || intents
                    .iter()
                    .any(|i| i.id == landing.intent && i.job_ids.iter().any(|j| j.as_str() == job))
        })
        .collect();
    found.sort_by(|a, b| (a.created_at, a.id.as_str()).cmp(&(b.created_at, b.id.as_str())));
    Ok(found.into_iter().next().map(|l| l.host_commit))
}

/// Whether the records branch holds an import manifest at its root naming `c0`.
pub fn manifest_names(git: &Git, tip: &str, c0: &str) -> Result<bool> {
    let Some(bytes) = git.show_file(tip, IMPORT_MANIFEST_PATH)? else {
        return Ok(false);
    };
    Ok(matches!(
        HistoryRecord::parse(&String::from_utf8_lossy(&bytes)),
        Ok(HistoryRecord::ImportManifest(m)) if m.c0 == c0
    ))
}

/// The cutover state for `job` (the configured `[history] cutover_job`) on the records branch.
pub fn cutover_state(git: &Git, job: &str) -> Result<CutoverState> {
    let RecordsTip::At { tip, .. } = records_branch::resolve_tip(git)? else {
        return Ok(CutoverState::NotLanded);
    };
    let Some(c0) = cutover_commit_at(git, &tip, job)? else {
        return Ok(CutoverState::NotLanded);
    };
    Ok(if manifest_names(git, &tip, &c0)? {
        CutoverState::Shown { c0 }
    } else {
        CutoverState::Pending { c0 }
    })
}

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
    /// The history check, run on the commits up to and including `C0`, found a violation (§4; D111).
    LegacyHistory,
    /// The records branch holds no `legacy-exempt.json`, or it is not a valid legacy-exempt record (§4).
    LegacyExemptMissing,
    /// `legacy-exempt.json` was changed after it was first written: it is append-only (§4, §10).
    LegacyExemptChanged,
    /// A full replay disagrees with the newest checkpoint: the checkpoint names a commit the replay fails, or
    /// one that is not on main (§9; D111).
    CheckpointDisagrees,
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
            FindingKind::LegacyHistory => "legacy history check",
            FindingKind::LegacyExemptMissing => "legacy-exempt record missing",
            FindingKind::LegacyExemptChanged => "legacy-exempt record changed",
            FindingKind::CheckpointDisagrees => "checkpoint disagrees with the full replay",
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
    /// How many records were judged: every record on the branch for a full walk, only those added since the
    /// checkpoint for an incremental one. Checkpoint records themselves are not counted.
    pub records_judged: usize,
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
    walk(git, main_tip, now, None, AttributionReport::default())
}

/// The records-only walk. With `cutover` `None` it starts after the earliest landing-intent's expected base
/// (the part 1 rule). With `Some(c0)` it starts after `c0` on main's first-parent line, and a records branch
/// with no landing-intent is not a finding: every commit after `c0` is then simply unattributed. `report`
/// carries findings already made (the legacy rule's), and is extended.
pub(crate) fn walk(
    git: &Git,
    main_tip: &str,
    now: Timestamp,
    cutover: Option<&str>,
    report: AttributionReport,
) -> Result<AttributionReport> {
    walk_since(git, main_tip, now, cutover, report, None)
}

/// [`walk`] with an optional records delta. With `records_since` `Some(r)` only the records added after the
/// records commit `r` are audited and counted; every intent and landing at the records tip is still read, so
/// a landing in the delta can cite an older intent (§9; D111).
fn walk_since(
    git: &Git,
    main_tip: &str,
    now: Timestamp,
    cutover: Option<&str>,
    mut report: AttributionReport,
    records_since: Option<&str>,
) -> Result<AttributionReport> {
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

    let problems = match records_since {
        Some(since) => records_branch::audit_history_since(git, &records_tip, since)?,
        None => records_branch::audit_history(git, &records_tip)?,
    };
    report.records_judged = records_branch::files_added_between(git, records_since, &records_tip)?
        .iter()
        .filter(|path| !path.starts_with("checkpoints/"))
        .count();
    for problem in problems {
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

    let base = match cutover {
        Some(c0) => c0.to_string(),
        None => {
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
            first.expected_base.clone()
        }
    };
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
            if cutover.is_some() {
                format!("the cutover commit {base} is not on main's first-parent line")
            } else {
                format!("the earliest landing-intent expects base {base}, which is not on main's first-parent line")
            },
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

/// How a run judges (§9; D111).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Judge only main's commits and the records added since the newest checkpoint. Falls back to a full
    /// replay, with a stated reason, when there is no usable checkpoint.
    Incremental,
    /// Ignore checkpoints, judge everything from the start, and fail if the newest checkpoint disagrees.
    FullReplay,
}

impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Mode::Incremental => "incremental",
            Mode::FullReplay => "full replay",
        }
    }
}

/// What a run is asked to do.
#[derive(Debug, Clone)]
pub struct Options {
    pub mode: Mode,
    /// Whether a passing run may write a checkpoint. False in CI and for a read-only run: the check then
    /// never touches the records branch.
    pub write_checkpoint: bool,
    /// The check version a checkpoint must carry to be trusted, and the one a new checkpoint records.
    pub version: String,
}

impl Options {
    /// `mode`, with the current [`CHECK_VERSION`].
    pub fn new(mode: Mode, write_checkpoint: bool) -> Self {
        Options {
            mode,
            write_checkpoint,
            version: CHECK_VERSION.to_string(),
        }
    }
}

/// The outcome of [`run_check`]: the verdict and what the run did.
#[derive(Debug, Clone)]
pub struct RunReport {
    pub report: AttributionReport,
    /// The mode asked for.
    pub requested: Mode,
    /// The mode that ran. Differs from `requested` when an incremental run fell back.
    pub mode: Mode,
    /// Why an incremental run became a full replay.
    pub fallback_reason: Option<String>,
    /// The checkpoint an incremental run started from.
    pub started_from: Option<Checkpoint>,
    /// The checkpoint this run wrote, if it wrote one.
    pub checkpoint_written: Option<Checkpoint>,
    /// Why a checkpoint that should have been written was not. The verdict is unaffected.
    pub checkpoint_error: Option<String>,
    /// Wall-clock seconds the whole run took.
    pub seconds: f64,
}

impl RunReport {
    /// Main commits judged.
    pub fn commits_judged(&self) -> usize {
        self.report.checked
    }

    /// Records judged.
    pub fn records_judged(&self) -> usize {
        self.report.records_judged
    }
}

/// Reads the newest checkpoint on `tip`: the one added by the newest first-parent records commit that added
/// any (`None` when there is none). The inner `Err` says why the file cannot be used.
fn read_newest_checkpoint(
    git: &Git,
    tip: &str,
) -> Result<Option<std::result::Result<Checkpoint, String>>> {
    let Some(path) = records_branch::newest_added_under(git, tip, CHECKPOINT_DIR)? else {
        return Ok(None);
    };
    let Some(bytes) = git.show_file(tip, &path)? else {
        return Ok(Some(Err(format!("{path} is missing"))));
    };
    Ok(Some(
        match HistoryRecord::parse(&String::from_utf8_lossy(&bytes)) {
            Ok(HistoryRecord::Checkpoint(cp)) => Ok(cp),
            Ok(other) => Err(format!("{path} holds a {} record", other.schema())),
            Err(error) => Err(format!("{path} is not a valid record: {error}")),
        },
    ))
}

/// Runs the attribution check as `options` direct (§9; D111): incremental from the newest checkpoint, or a
/// full replay; and on a pass writes a checkpoint when allowed.
///
/// An incremental run judges main's commits after the checkpoint's `main_sha` and the records added after its
/// `records_sha`. Intents and landings are still looked up across the whole records tip, because an intent is
/// written before its merge and its landing after, so a checkpoint can fall between them. It falls back to a
/// full replay, and says why, when there is no checkpoint, the checkpoint's version differs from
/// `options.version`, or it names a commit that is not on main's first-parent line or a records commit that is
/// not on the records branch. A full replay (asked for or fallen back to) adds a
/// [`FindingKind::CheckpointDisagrees`] finding when the newest checkpoint names a commit that is not on
/// main or one the replay fails.
///
/// A checkpoint is written only when the run passed with no pending commit, writing is allowed, the records
/// tip was read from the local branch, and something new was judged (or the run was a full replay). It is
/// appended to the local records branch only; this function never pushes.
pub fn run_check(
    git: &Git,
    main_tip: &str,
    now: Timestamp,
    options: &Options,
) -> Result<RunReport> {
    let started = Instant::now();
    let (records_tip, from) = match records_branch::resolve_tip(git)? {
        RecordsTip::At { tip, from } => (Some(tip), Some(from)),
        _ => (None, None),
    };
    let newest = match &records_tip {
        Some(tip) => read_newest_checkpoint(git, tip)?,
        None => None,
    };
    let line = if newest.is_some() {
        git.run(["rev-list", "--first-parent", main_tip])?
            .lines()
            .map(str::to_string)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let mut fallback_reason = None;
    let mut started_from = None;
    if options.mode == Mode::Incremental && records_tip.is_some() {
        match &newest {
            None => fallback_reason = Some("no checkpoint on the records branch".to_string()),
            Some(Err(why)) => {
                fallback_reason = Some(format!("the newest checkpoint is unusable: {why}"));
            }
            Some(Ok(cp)) if cp.verifier_version != options.version => {
                fallback_reason = Some(format!(
                    "the check version changed: the newest checkpoint was written by {}, this check is {}",
                    cp.verifier_version, options.version
                ));
            }
            Some(Ok(cp)) if !line.contains(&cp.main_sha) => {
                fallback_reason = Some(format!(
                    "the newest checkpoint names {}, which is not on main's first-parent line",
                    cp.main_sha
                ));
            }
            Some(Ok(cp))
                if !(git.commit_exists(&cp.records_sha)?
                    && records_tip.as_deref().is_some_and(|tip| {
                        git.is_ancestor(&cp.records_sha, tip).unwrap_or(false)
                    })) =>
            {
                fallback_reason = Some(format!(
                    "the newest checkpoint names records commit {}, which is not on the records branch",
                    cp.records_sha
                ));
            }
            Some(Ok(cp)) => started_from = Some(cp.clone()),
        }
    }

    let mut report = walk_since(
        git,
        main_tip,
        now,
        started_from.as_ref().map(|cp| cp.main_sha.as_str()),
        AttributionReport::default(),
        started_from.as_ref().map(|cp| cp.records_sha.as_str()),
    )?;
    let mode = if started_from.is_some() {
        Mode::Incremental
    } else {
        Mode::FullReplay
    };

    if mode == Mode::FullReplay && records_tip.is_some() {
        match &newest {
            Some(Ok(cp)) => {
                if !line.contains(&cp.main_sha) {
                    report.find(
                        FindingKind::CheckpointDisagrees,
                        Some(&cp.main_sha),
                        format!(
                            "the newest checkpoint names {}, which is not on main's first-parent line",
                            cp.main_sha
                        ),
                    );
                } else if report
                    .findings
                    .iter()
                    .any(|f| f.commit.as_deref() == Some(cp.main_sha.as_str()))
                {
                    report.find(
                        FindingKind::CheckpointDisagrees,
                        Some(&cp.main_sha),
                        format!(
                            "the newest checkpoint says {} passed, but the full replay fails it",
                            cp.main_sha
                        ),
                    );
                }
            }
            Some(Err(why)) if options.mode == Mode::FullReplay => report.find(
                FindingKind::CheckpointDisagrees,
                None,
                format!("the newest checkpoint is unusable: {why}"),
            ),
            _ => {}
        }
    }

    let mut checkpoint_written = None;
    let mut checkpoint_error = None;
    let something_new = mode == Mode::FullReplay || report.checked > 0 || report.records_judged > 0;
    if options.write_checkpoint
        && report.passed()
        && report.pending.is_empty()
        && something_new
        && let Some(tip) = &records_tip
    {
        if from != Some(RECORDS_REF) {
            checkpoint_error = Some(
                "the records branch is only in its remote-tracking ref; fetch it into athena/records first"
                    .to_string(),
            );
        } else {
            match write_checkpoint(git, main_tip, tip, now, &options.version) {
                Ok(cp) => checkpoint_written = Some(cp),
                Err(error) => checkpoint_error = Some(error),
            }
        }
    }

    Ok(RunReport {
        report,
        requested: options.mode,
        mode,
        fallback_reason,
        started_from,
        checkpoint_written,
        checkpoint_error,
        seconds: started.elapsed().as_secs_f64(),
    })
}

/// Appends a checkpoint to the local records branch with the records writer.
fn write_checkpoint(
    git: &Git,
    main_sha: &str,
    records_sha: &str,
    now: Timestamp,
    version: &str,
) -> std::result::Result<Checkpoint, String> {
    let checkpoint = Checkpoint {
        id: RecordId::generate(now),
        created_at: now,
        main_sha: main_sha.to_string(),
        records_sha: records_sha.to_string(),
        verifier_version: version.to_string(),
    };
    let json = serde_json::to_string(&HistoryRecord::Checkpoint(checkpoint.clone()))
        .map_err(|e| e.to_string())?;
    RecordsWriter::new(git)
        .append(
            "checkpoint",
            &[Change::put(
                format!("{CHECKPOINT_DIR}/{}.json", checkpoint.id),
                json,
            )],
        )
        .map_err(|e| e.to_string())?;
    Ok(checkpoint)
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

    // ---- Incremental checking with checkpoints (§9; D111) ----

    /// Adds one landed squash on top of HEAD: the commit, its intent and its landing. Returns the commit.
    fn land(dir: &tempfile::TempDir, git: &Git, n: u32) -> String {
        let base = git.head().unwrap().unwrap();
        let name = format!("f{n}.txt");
        write(dir.path(), &[(name.as_str(), "x\n")]);
        let sha = commit(git, "Squash of a pull request");
        let tree = git.tree_of(&sha).unwrap();
        let i = intent(rid(2 * n), T0, &tree, &base);
        put_intent(git, &i);
        put_landing(git, rid(2 * n + 1), &sha, &i.id);
        sha
    }

    fn now() -> Timestamp {
        ts("2026-10-24T10:01:00Z")
    }

    fn head(git: &Git) -> String {
        git.head().unwrap().unwrap()
    }

    fn records_tip(git: &Git) -> String {
        git.rev_parse(RECORDS_REF).unwrap().unwrap()
    }

    fn incremental() -> Options {
        Options::new(Mode::Incremental, true)
    }

    fn put_checkpoint(git: &Git, name: &str, main_sha: &str, version: &str) {
        let cp = Checkpoint {
            id: rid(900),
            created_at: ts(T0),
            main_sha: main_sha.to_string(),
            records_sha: records_tip(git),
            verifier_version: version.to_string(),
        };
        put(
            git,
            &format!("{CHECKPOINT_DIR}/{name}.json"),
            &HistoryRecord::Checkpoint(cp),
        );
    }

    #[test]
    fn incremental_run_judges_only_new_commits() {
        let (dir, git) = repo(&[("a.txt", "a\n")]);
        for n in 1..=3 {
            land(&dir, &git, n);
        }
        let first = run_check(&git, &head(&git), now(), &incremental()).unwrap();
        assert!(first.report.passed(), "{:?}", first.report.findings);
        assert_eq!(first.mode, Mode::FullReplay, "no checkpoint yet");
        assert!(
            first
                .fallback_reason
                .as_deref()
                .unwrap()
                .contains("no checkpoint")
        );
        assert_eq!((first.commits_judged(), first.records_judged()), (3, 6));
        let written = first.checkpoint_written.clone().unwrap();

        land(&dir, &git, 4);
        land(&dir, &git, 5);
        let second = run_check(&git, &head(&git), now(), &incremental()).unwrap();
        assert!(second.report.passed(), "{:?}", second.report.findings);
        assert_eq!(second.mode, Mode::Incremental);
        assert_eq!(second.started_from, Some(written));
        // Two landings: two commits, and an intent and a landing record each. The checkpoint is not counted.
        assert_eq!(second.commits_judged(), 2);
        assert_eq!(second.records_judged(), 4);
        assert_eq!(second.report.attributed.len(), 2);
        assert!(second.seconds >= 0.0);

        let full = run_check(
            &git,
            &head(&git),
            now(),
            &Options::new(Mode::FullReplay, false),
        )
        .unwrap();
        assert!(full.report.passed(), "{:?}", full.report.findings);
        assert_eq!((full.commits_judged(), full.records_judged()), (5, 10));
    }

    #[test]
    fn intent_before_checkpoint_landing_after_passes() {
        let (dir, git) = repo(&[("a.txt", "a\n")]);
        let _ = land(&dir, &git, 1);
        let two = land(&dir, &git, 2);
        // Commit 3 is merged and its intent was written, but its landing is not yet.
        write(dir.path(), &[("f3.txt", "x\n")]);
        let three = commit(&git, "Squash of a pull request");
        let i = intent(rid(6), T0, &git.tree_of(&three).unwrap(), &two);
        put_intent(&git, &i);
        // The checkpoint is written at commit 2, after the intent and before the landing.
        let at_two = run_check(&git, &two, now(), &incremental()).unwrap();
        assert!(at_two.checkpoint_written.is_some());
        put_landing(&git, rid(7), &three, &i.id);

        let run = run_check(&git, &three, now(), &incremental()).unwrap();
        assert_eq!(run.mode, Mode::Incremental);
        assert!(run.report.passed(), "{:?}", run.report.findings);
        assert_eq!(run.commits_judged(), 1);
        assert_eq!(run.records_judged(), 1, "only the landing is new");
        assert_eq!(run.report.attributed, vec![three]);
    }

    #[test]
    fn tampered_checkpoint_is_caught_by_full_replay() {
        let (dir, git) = repo(&[("a.txt", "a\n")]);
        land(&dir, &git, 1);
        write(dir.path(), &[("rogue.txt", "r\n")]);
        let rogue = commit(&git, "Hand-made commit");
        // A hand-written checkpoint says the rogue commit passed.
        put_checkpoint(&git, "forged", &rogue, CHECK_VERSION);

        let trusting =
            run_check(&git, &rogue, now(), &Options::new(Mode::Incremental, false)).unwrap();
        assert_eq!(trusting.mode, Mode::Incremental);
        assert!(
            trusting.report.passed(),
            "an incremental run trusts the checkpoint"
        );

        let replay =
            run_check(&git, &rogue, now(), &Options::new(Mode::FullReplay, false)).unwrap();
        assert!(!replay.report.passed());
        let kinds: Vec<_> = replay.report.findings.iter().map(|f| f.kind).collect();
        assert!(kinds.contains(&FindingKind::Unattributed), "{kinds:?}");
        assert!(
            kinds.contains(&FindingKind::CheckpointDisagrees),
            "{kinds:?}"
        );

        // A checkpoint naming a commit that is not on main disagrees too.
        let (dir2, git2) = repo(&[("a.txt", "a\n")]);
        land(&dir2, &git2, 1);
        put_checkpoint(&git2, "elsewhere", &"7".repeat(40), CHECK_VERSION);
        let replay = run_check(
            &git2,
            &head(&git2),
            now(),
            &Options::new(Mode::FullReplay, false),
        )
        .unwrap();
        assert!(
            replay
                .report
                .findings
                .iter()
                .any(|f| f.kind == FindingKind::CheckpointDisagrees),
            "{:?}",
            replay.report.findings
        );
    }

    #[test]
    fn check_version_change_forces_full_replay() {
        let (dir, git) = repo(&[("a.txt", "a\n")]);
        for n in 1..=3 {
            land(&dir, &git, n);
        }
        let mut old = incremental();
        old.version = "attribution-check/0".to_string();
        assert!(
            run_check(&git, &head(&git), now(), &old)
                .unwrap()
                .checkpoint_written
                .is_some()
        );
        land(&dir, &git, 4);

        let run = run_check(&git, &head(&git), now(), &incremental()).unwrap();
        assert_eq!(run.requested, Mode::Incremental);
        assert_eq!(run.mode, Mode::FullReplay);
        let why = run.fallback_reason.clone().unwrap();
        assert!(
            why.contains("attribution-check/0") && why.contains(CHECK_VERSION),
            "{why}"
        );
        assert_eq!((run.commits_judged(), run.records_judged()), (4, 8));
        // The new checkpoint carries the new version, so the next run is incremental again.
        assert_eq!(
            run.checkpoint_written.unwrap().verifier_version,
            CHECK_VERSION
        );
        let next = run_check(&git, &head(&git), now(), &incremental()).unwrap();
        assert_eq!(next.mode, Mode::Incremental);
    }

    #[test]
    fn failing_check_writes_no_checkpoint() {
        let (dir, git) = repo(&[("a.txt", "a\n")]);
        land(&dir, &git, 1);
        write(dir.path(), &[("rogue.txt", "r\n")]);
        let rogue = commit(&git, "Hand-made commit");
        let before = records_tip(&git);
        let run = run_check(&git, &rogue, now(), &incremental()).unwrap();
        assert!(!run.report.passed());
        assert!(run.checkpoint_written.is_none() && run.checkpoint_error.is_none());
        assert_eq!(records_tip(&git), before, "the records branch did not move");

        // A pending commit passes but is not a point to start from.
        let (dir, git) = repo(&[("a.txt", "a\n")]);
        let one = land(&dir, &git, 1);
        write(dir.path(), &[("p.txt", "p\n")]);
        let pending = commit(&git, "Merged, landing due");
        let i = intent(rid(10), T0, &git.tree_of(&pending).unwrap(), &one);
        put_intent(&git, &i);
        let before = records_tip(&git);
        let run = run_check(&git, &pending, ts("2026-10-24T10:05:00Z"), &incremental()).unwrap();
        assert!(run.report.passed(), "{:?}", run.report.findings);
        assert_eq!(run.report.pending.len(), 1);
        assert!(run.checkpoint_written.is_none());
        assert_eq!(records_tip(&git), before);
    }

    #[test]
    fn passing_check_writes_checkpoint() {
        let (dir, git) = repo(&[("a.txt", "a\n")]);
        land(&dir, &git, 1);
        let main = land(&dir, &git, 2);
        let before = records_tip(&git);
        let run = run_check(&git, &main, now(), &incremental()).unwrap();
        let cp = run.checkpoint_written.clone().unwrap();
        assert_eq!(cp.main_sha, main);
        assert_eq!(cp.records_sha, before);
        assert_eq!(cp.verifier_version, CHECK_VERSION);

        let tip = records_tip(&git);
        assert_eq!(
            git.parents(&tip).unwrap(),
            vec![before],
            "one new records commit"
        );
        let stored = git
            .show_file(&tip, &format!("{CHECKPOINT_DIR}/{}.json", cp.id))
            .unwrap()
            .unwrap();
        match HistoryRecord::parse(&String::from_utf8(stored).unwrap()).unwrap() {
            HistoryRecord::Checkpoint(read) => assert_eq!(read, cp),
            other => panic!("{other:?}"),
        }
        // Nothing new since: a second run judges nothing and writes nothing.
        let again = run_check(&git, &main, now(), &incremental()).unwrap();
        assert_eq!((again.commits_judged(), again.records_judged()), (0, 0));
        assert!(again.checkpoint_written.is_none());
        assert_eq!(records_tip(&git), tip);
    }

    #[test]
    fn full_replay_ignores_checkpoints_and_read_only_never_writes() {
        let (dir, git) = repo(&[("a.txt", "a\n")]);
        for n in 1..=3 {
            land(&dir, &git, n);
        }
        assert!(
            run_check(&git, &head(&git), now(), &incremental())
                .unwrap()
                .checkpoint_written
                .is_some()
        );
        land(&dir, &git, 4);
        let before = records_tip(&git);

        let read_only_full = run_check(
            &git,
            &head(&git),
            now(),
            &Options::new(Mode::FullReplay, false),
        )
        .unwrap();
        assert_eq!(read_only_full.mode, Mode::FullReplay);
        assert!(
            read_only_full.fallback_reason.is_none(),
            "asked for, not fallen back to"
        );
        assert!(read_only_full.started_from.is_none());
        assert_eq!(
            (
                read_only_full.commits_judged(),
                read_only_full.records_judged()
            ),
            (4, 8)
        );
        assert!(read_only_full.report.passed());

        let read_only_inc = run_check(
            &git,
            &head(&git),
            now(),
            &Options::new(Mode::Incremental, false),
        )
        .unwrap();
        assert_eq!(read_only_inc.mode, Mode::Incremental);
        assert_eq!(
            (
                read_only_inc.commits_judged(),
                read_only_inc.records_judged()
            ),
            (1, 2)
        );
        for run in [&read_only_full, &read_only_inc] {
            assert!(run.checkpoint_written.is_none() && run.checkpoint_error.is_none());
        }
        assert_eq!(records_tip(&git), before, "a read-only run never writes");
    }

    #[test]
    fn checkpoint_write_failure_keeps_verdict() {
        let (dir, git) = repo(&[("a.txt", "a\n")]);
        land(&dir, &git, 1);
        let before = records_tip(&git);
        // A lock on the records ref makes every update fail.
        let lock = git
            .common_dir()
            .unwrap()
            .join("refs/heads/athena/records.lock");
        std::fs::write(&lock, "").unwrap();
        let run = run_check(&git, &head(&git), now(), &incremental()).unwrap();
        assert!(run.report.passed(), "the verdict does not change");
        assert!(run.checkpoint_written.is_none());
        assert!(run.checkpoint_error.is_some());
        std::fs::remove_file(&lock).unwrap();
        assert_eq!(records_tip(&git), before);
    }

    #[test]
    fn timing_on_synthetic_200_landings() {
        let (dir, git) = repo(&[("a.txt", "a\n")]);
        for n in 1..=200 {
            land(&dir, &git, n);
        }
        let full = run_check(
            &git,
            &head(&git),
            now(),
            &Options::new(Mode::FullReplay, true),
        )
        .unwrap();
        assert!(full.report.passed(), "{:?}", full.report.findings);
        assert_eq!((full.commits_judged(), full.records_judged()), (200, 400));
        assert!(full.checkpoint_written.is_some());

        land(&dir, &git, 201);
        let inc = run_check(&git, &head(&git), now(), &incremental()).unwrap();
        assert!(inc.report.passed(), "{:?}", inc.report.findings);
        assert_eq!(inc.mode, Mode::Incremental);
        assert_eq!((inc.commits_judged(), inc.records_judged()), (1, 2));
        // Printed, never asserted: seconds vary with the machine.
        println!(
            "measured on the synthetic 200-landing fixture: full replay {:.3} s ({} commits, {} records); \
             incremental {:.3} s ({} commit, {} records)",
            full.seconds,
            full.commits_judged(),
            full.records_judged(),
            inc.seconds,
            inc.commits_judged(),
            inc.records_judged()
        );
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
