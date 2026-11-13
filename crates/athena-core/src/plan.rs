//! Planning, the first stage of every request: each open request is fleshed out (goal, requirements-ladder
//! rung, acceptance, open questions, dependencies, what it touches) and all open requests are planned
//! together into one coordinated plan, instead of being worked one at a time, oldest first.
//!
//! The plan is pure data and deterministic code; it calls no model:
//!
//! - **Order.** Dependencies first, then priority, then rung, then age. Items with every dependency done form
//!   the ready set; only the ready items with the best effective priority make the next wave, and within a
//!   wave the order is the ladder (the top rung first), then age (oldest first) as the tiebreak. An item's
//!   effective priority is its own, raised to the best priority of anything that depends on it, so the
//!   prerequisites of a high-priority item run before unrelated older normal items. A dependent item always
//!   comes after what it needs. A request's priority is [`Priority`]: urgent, high, normal (the default) or
//!   low, read from a `Priority: <level>` line in its words, or from a later priority record
//!   (`athena priority`), which wins. A request that serves a BRIEF.md addendum stating a priority is ranked
//!   at least that high (see `reading::plan_entry`). With no priority set every item is normal and the order
//!   is the ladder, then age. [`Plan::priority_violations`] checks a built order against the priorities.
//!   The job of an approved steward proposal is an item like any other: its priority is normal unless a
//!   priority record or its raising role says otherwise ([`proposal_priority`]), and it is ordered by the same
//!   rules, not worked ahead of the plan.
//! - **Dependencies.** An item depends only on the jobs named on a "Depends on" line of its request and on the
//!   job a "try again on <job>" request retries; a job id cited elsewhere in the text is a reference and never
//!   holds the item (`reading::dependency_ids_in`). A dependency on another open item is met once that item is
//!   ordered. A dependency on a
//!   job that is not an open item is met when that job landed, or when a landed job is linked to it as its
//!   retry ("try again on <job>", also through a chain of retries). A dependency on a job that was rejected,
//!   failed, timed out or went stale, with no landed retry, is unmet: the item is not ordered and is listed
//!   in [`Plan::refused`] with a reason naming the dependency, and so is every item that depends on a held
//!   item ([`Plan::build_with`], given the jobs' end states as [`JobEnd`]s). The retry itself ("try again on
//!   <job>") names the job it retries and is not held by it. Only an id the history does not show failing
//!   (unknown, dropped, declined or abandoned) still counts as done.
//! - **Parallel groups.** The groups are built from the order, taking items in plan order: an item joins the
//!   last group when its touched paths are disjoint from every member's and it depends on none of them,
//!   otherwise it opens a new group. A group is therefore never before the group of anything its items depend
//!   on, the groups flattened equal [`Plan::order`], and a later group never holds an item the order ranks
//!   ahead of an earlier group's (groups allow parallel work; they never reorder it). The items of one group
//!   could run side by side. An item that names no touched path is
//!   never grouped with another, because nothing is known about what it changes. The shared append-only files
//!   (`SHARED_APPEND_ONLY`: `CHANGELOG.md`, `docs/decisions.md`, `README.md`; `docs/design/parallel-jobs.md`
//!   section 9) are left out of the overlap test, since nearly every item touches them; an item whose only
//!   paths are shared files therefore names nothing else and is not grouped. The landing check stays the
//!   authority for conflicting edits to a shared file; this changes only the grouping. The drain works the groups in
//!   sequence and runs the items of one group at once, up to `[parallel] max_concurrency` (default 1: one
//!   job at a time, which dispatches exactly in plan order).
//! - **Gate.** An item whose plan has no goal, no ladder rung or no acceptance is refused for design or code
//!   ([`PlanEntry::missing`], [`Plan::refused`]), so planning cannot be skipped. A read request whose design
//!   names no existing code it changes or reuses, and does not say it is wholly new, is refused the same way
//!   ([`code_reference_problem`]), so a design that does not match the repository is stopped before
//!   production and not at the final verdict. A read request whose design edits `athena.toml` or
//!   `defaults.toml` without listing a file under `crates/athena-data/` (the schema that parses them) is refused
//!   too ([`schema_reference_problem`], on the rule admission and the landing gate share). Nothing is filled in by
//!   default: acceptance comes only from explicit bullets in the request or from a reading's criteria, and
//!   the rung only from the caller or from what the request serves in BRIEF.md and its addenda
//!   (`reading::plan_entry`, `reading::infer_rung`); a word such as "fix" or "code" is not a rung. A request
//!   that serves the owner's latest addendum therefore ranks above plain implementation work, whatever its age. A vague request such as "make it better" therefore has neither, is flagged
//!   with the missing fields named, and is not in [`Plan::order`].
//!
//! The second half of the module is the first slice of the plan-stage audit: every logged gap (an
//! intervention, an introspection, a rejection, a rating) is traced to the planning stage that let it
//! through ([`trace`]), and [`GapLog::fell`] checks whether later gaps of that stage became rarer after a
//! change to it.

use athena_data::roles::Ladder;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// How soon a request should be worked, ahead of its rung and age. Urgent sorts first, then high.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Priority {
    Urgent,
    High,
    #[default]
    Normal,
    Low,
}

impl Priority {
    pub fn as_str(self) -> &'static str {
        match self {
            Priority::Urgent => "urgent",
            Priority::High => "high",
            Priority::Normal => "normal",
            Priority::Low => "low",
        }
    }

    /// The priority named by a bare level word (`urgent`, `high`, `normal` or `low`, any case, trimmed).
    pub fn parse(word: &str) -> Option<Priority> {
        match word.trim().to_ascii_lowercase().as_str() {
            "urgent" => Some(Priority::Urgent),
            "high" => Some(Priority::High),
            "normal" => Some(Priority::Normal),
            "low" => Some(Priority::Low),
            _ => None,
        }
    }

    /// The priority a request's words give: the first line that, trimmed, starts with `Priority:` (any case)
    /// followed by exactly `urgent`, `high`, `normal` or `low` (any case). Anything else, or no such line, is
    /// normal.
    pub fn from_text(text: &str) -> Priority {
        for line in text.lines() {
            let line = line.trim();
            let Some(head) = line.get(..9) else { continue };
            if !head.eq_ignore_ascii_case("priority:") {
                continue;
            }
            if let Some(p) = Priority::parse(&line[9..]) {
                return p;
            }
        }
        Priority::Normal
    }
}

/// The line of a request that names it: the first non-empty line that is not a `Priority:` line. When
/// every line is blank or a Priority line, the first line, as before.
pub fn title_line(text: &str) -> &str {
    let is_priority = |l: &str| {
        l.trim()
            .get(..9)
            .is_some_and(|h| h.eq_ignore_ascii_case("priority:"))
    };
    text.lines()
        .find(|l| !l.trim().is_empty() && !is_priority(l))
        .or_else(|| text.lines().next())
        .unwrap_or("")
}

/// The priority of the job that carries out an approved steward proposal.
///
/// Root cause it addresses: an approved proposal had no rule of its own for how soon it is worked, and drain
/// worked it before the plan, so it ranked above urgent requests. It now ranks like any other item, and this is
/// its priority: normal unless something says otherwise. In order of precedence:
///
/// 1. A priority record (`athena priority`) is the user's word and decides alone, even when it says normal or
///    low. `record` is that record's level, when there is one.
/// 2. Otherwise `own`, what the job's words or reading give, when that is not normal.
/// 3. Otherwise `role`, the default priority the raising role's standing instruction states, which is normal
///    when it states none.
pub fn proposal_priority(record: Option<Priority>, own: Priority, role: Priority) -> Priority {
    record.unwrap_or(if own == Priority::Normal { role } else { own })
}

/// One request, fleshed out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanEntry {
    /// The job id of the request.
    pub id: String,
    pub title: String,
    /// What the request is for, in a sentence. Empty when planning has not found it.
    pub goal: String,
    /// The requirements-ladder rung the request serves. None when planning has not found it.
    pub rung: Option<Ladder>,
    /// What shows the request is done.
    pub acceptance: Vec<String>,
    /// What is still unknown.
    pub open_questions: Vec<String>,
    /// Ids of the requests that must finish first: those on a "Depends on" line of the request, and the job a
    /// "try again on <job>" request retries. Job ids cited elsewhere in the text are not listed.
    pub depends_on: Vec<String>,
    /// Repository paths (a directory ends with `/`) the request will change.
    pub touches: Vec<String>,
    /// How soon the request should be worked; normal unless its words say otherwise.
    pub priority: Priority,
    /// Why the design of a read request does not name the existing code it changes or reuses, in a sentence
    /// a stranger can follow; None when it does, when it states it is wholly new code, or when the request
    /// has not been read yet. Set by `reading::plan_entry`; see [`code_reference_problem`].
    pub code_problem: Option<String>,
    /// Why the design of a read request edits `athena.toml` or `defaults.toml` without the schema that must
    /// accept the edit, in a sentence; None otherwise. Set by `reading::plan_entry`; see
    /// [`schema_reference_problem`].
    pub schema_problem: Option<String>,
    /// True when the user put the request on hold (`athena hold`) and has not released it. [`Plan::build_with`]
    /// never orders such an item, whatever its priority, and holds back whatever depends on it. Set from
    /// `JobRecords::on_hold` where the entry is made.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub on_hold: bool,
}

/// What `PlanEntry::missing` reports when a configuration edit has no schema change beside it.
pub const MISSING_SCHEMA: &str = "the schema change for its configuration edit";

/// The plan-time check that an edit to `athena.toml` or `defaults.toml` comes with the schema that parses it.
/// `changes` is each path the design lists with whether the design deletes it.
///
/// Root cause it addresses: the configuration files and the code that parses them (`crates/athena-data/`,
/// strict on unknown fields) are changed separately, and nothing connected a new section to the schema that
/// must accept it before admission or landing. Jobs 20261016-071202-q0xy and 20261016-185517-qcrz planned a
/// `[parallel]` table in `athena.toml` with the schema in the wrong crate, or not at all, and were refused
/// there after production; 20261016-200605-xh54 was refused by the landing gate for the same cause. Admission
/// and the landing gate already skip their own parse when the change modifies the schema
/// ([`athena_gate::defers_to_schema_test`]); this applies that same rule one stage earlier, through
/// [`athena_gate::config_edits_without_schema`], so there is one definition of the schema directory.
///
/// The check cannot see at plan time whether an edit adds a section, so it takes the narrowest sound form: any
/// non-deleted configuration file in the design needs a non-deleted path under
/// [`athena_gate::SCHEMA_DIR`] beside it. The returned sentence names the schema file to change.
pub fn schema_reference_problem(changes: &[(String, bool)]) -> Option<String> {
    let lacking = athena_gate::config_edits_without_schema(
        changes
            .iter()
            .map(|(path, deleted)| (path.as_str(), *deleted)),
    );
    if lacking.is_empty() {
        return None;
    }
    let schemas: Vec<&str> = lacking
        .iter()
        .map(|p| {
            if *p == athena_gate::DEFAULTS_PATH {
                "crates/athena-data/src/defaults.rs"
            } else {
                "crates/athena-data/src/config.rs"
            }
        })
        .collect();
    Some(format!(
        "the design changes {} but lists no file under {}: the parser is strict on unknown fields, so a new section or key is refused at admission and landing unless the schema ({}) changes in the same job; list the schema file as a deliverable (modify), or drop the configuration edit",
        lacking.join(", "),
        athena_gate::SCHEMA_DIR,
        schemas.join(", "),
    ))
}

/// The changelog Athena adds to every change reading (`reading::ensure_changelog`).
const AUTO_ADDED_CHANGELOG: &str = "CHANGELOG.md";

/// What `PlanEntry::missing` reports when the design names no existing code.
pub const MISSING_CODE: &str = "existing code it changes or reuses";

/// The early check that a design names the code it changes: `paths` are the files the design lists, `created`
/// those it says it creates, and `exists` tells whether a path exists at the base commit.
///
/// Root cause it addresses: the design's references to existing code were first compared with the repository
/// at the final verdict, so a design that named nothing, or named files that are not there, was built in full
/// and rejected at the end. Doing it when the plan is made refuses such a design before production starts. The
/// queued item reading-or-provenance-mismatch-caught-only-at-final-verdict has the same cause; this one check
/// covers its reading half (a reading that names code that is not there) and leaves its provenance half and
/// the item itself open.
///
/// A design passes when at least one listed path exists, or when every listed path is one it creates (it
/// says the code is wholly new). Otherwise the returned sentence says what is missing. `CHANGELOG.md` is left
/// out of the test: Athena adds it to every change reading itself, so it says nothing about the design.
pub fn code_reference_problem(
    paths: &[String],
    created: &[String],
    exists: &dyn Fn(&str) -> bool,
) -> Option<String> {
    let paths: Vec<String> = paths
        .iter()
        .filter(|p| p.as_str() != AUTO_ADDED_CHANGELOG)
        .cloned()
        .collect();
    let paths = paths.as_slice();
    if paths.is_empty() {
        return Some(
            "the design lists no file at all; name the existing files it changes or reuses, or list only files it creates to say the code is wholly new"
                .into(),
        );
    }
    if paths.iter().all(|p| created.contains(p)) || paths.iter().any(|p| exists(p)) {
        return None;
    }
    Some(format!(
        "the design names no existing code: {} {} not exist at the base commit and {} not marked as created; name the existing files it changes or reuses, or mark the new ones as created",
        paths.join(", "),
        if paths.len() == 1 { "does" } else { "do" },
        if paths.len() == 1 { "is" } else { "are" },
    ))
}

impl PlanEntry {
    /// A request with nothing found out yet.
    pub fn bare(id: &str, title: &str) -> PlanEntry {
        PlanEntry {
            id: id.into(),
            title: title.into(),
            goal: String::new(),
            rung: None,
            acceptance: Vec::new(),
            open_questions: Vec::new(),
            depends_on: Vec::new(),
            touches: Vec::new(),
            priority: Priority::Normal,
            code_problem: None,
            schema_problem: None,
            on_hold: false,
        }
    }

    /// What a plan must have and this one lacks: `goal`, `ladder rung`, `acceptance`, and
    /// [`MISSING_CODE`] when a read request's design names no existing code (see [`code_reference_problem`]).
    pub fn missing(&self) -> Vec<&'static str> {
        let mut lacks = Vec::new();
        if self.goal.trim().is_empty() {
            lacks.push("goal");
        }
        if self.rung.is_none() {
            lacks.push("ladder rung");
        }
        if self.acceptance.iter().all(|a| a.trim().is_empty()) {
            lacks.push("acceptance");
        }
        if self.code_problem.is_some() {
            lacks.push(MISSING_CODE);
        }
        if self.schema_problem.is_some() {
            lacks.push(MISSING_SCHEMA);
        }
        lacks
    }
}

/// An item refused for design or code because its plan is incomplete.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Refusal {
    pub id: String,
    pub title: String,
    /// The fields the plan lacks: `goal`, `ladder rung`, `acceptance` (or `reading`, when the request could not
    /// be read).
    pub missing: Vec<String>,
    /// Why the item is held back, in a sentence.
    pub reason: String,
}

impl Refusal {
    /// A refusal for an incomplete plan.
    pub fn lacking(id: &str, title: &str, missing: Vec<String>) -> Refusal {
        Refusal::lacking_because(id, title, missing, None)
    }

    /// A refusal for an incomplete plan, with `detail` (what exactly is missing) added to the reason.
    pub fn lacking_because(
        id: &str,
        title: &str,
        missing: Vec<String>,
        detail: Option<&str>,
    ) -> Refusal {
        let mut reason = format!(
            "the plan lacks {}; say it in the request (acceptance as bullets, the files it changes) and plan again",
            missing.join(", ")
        );
        if let Some(detail) = detail {
            reason.push_str(&format!(" ({detail})"));
        }
        Refusal {
            id: id.into(),
            title: title.into(),
            missing,
            reason,
        }
    }
}

/// Why an item the user put on hold is not planned: it names the command that lifts the hold.
pub fn on_hold_reason(id: &str) -> String {
    format!("on hold; `athena release {id}` lifts it")
}

impl Refusal {
    /// A refusal for an item held back because a job it depends on has not landed: `missing` is empty.
    pub fn held(id: &str, title: &str, reason: &str) -> Refusal {
        Refusal {
            id: id.into(),
            title: title.into(),
            missing: Vec::new(),
            reason: reason.into(),
        }
    }
}

/// How a job that is not an open request ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ending {
    Landed,
    /// Ended without landing and needing another attempt: the end word, such as `rejected`, `failed`,
    /// `timed out` or `stale`.
    Failed(String),
    /// Any other state: not closed, or closed in a way that does not make a dependent wait (dropped, declined,
    /// abandoned). Such a job is listed so that its retry link is known.
    Other,
}

/// One finished job, for the dependency rule of [`Plan::build_with`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobEnd {
    pub id: String,
    pub ending: Ending,
    /// The job this one retries ("try again on <job>"), if it names one.
    pub retry_of: Option<String>,
}

/// Whether `item` is itself the retry of `dep` ("try again on <dep>"): a retry names the job it retries, and
/// that job's failure is why it exists, so it is never held by it.
fn retries(item: &str, dep: &str, history: &[JobEnd]) -> bool {
    history
        .iter()
        .any(|j| j.id == item && j.retry_of.as_deref() == Some(dep))
}

/// Why the dependency `dep` is unmet, or None when it is met: it is not shown ending in failure, or it
/// failed and a landed job reaches it through retry links.
fn unmet_dependency(dep: &str, history: &[JobEnd]) -> Option<String> {
    let word = match &history.iter().find(|j| j.id == dep)?.ending {
        Ending::Landed | Ending::Other => return None,
        Ending::Failed(word) => word,
    };
    let retried_by_landed = history
        .iter()
        .filter(|j| j.ending == Ending::Landed)
        .any(|landed| {
            // Walk the retry links up from the landed job; the visited list ends a loop of links.
            let mut seen: Vec<&str> = vec![landed.id.as_str()];
            let mut at = landed.retry_of.as_deref();
            while let Some(parent) = at {
                if parent == dep {
                    return true;
                }
                if seen.contains(&parent) {
                    return false;
                }
                seen.push(parent);
                at = history
                    .iter()
                    .find(|j| j.id == parent)
                    .and_then(|j| j.retry_of.as_deref());
            }
            false
        });
    (!retried_by_landed).then(|| {
        format!(
            "depends on {dep}, which was {word} and has no landed retry; it is held until a retry of it lands"
        )
    })
}

/// The coordinated plan of all open requests.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Plan {
    /// Every planned item, in the order to work them. The drain dispatches in this order when it runs one job
    /// at a time; `groups` only mark which neighbours could run side by side.
    pub order: Vec<String>,
    /// The same items in groups, built from `order`: each group's items could run in parallel, each group comes
    /// after the groups holding what its items depend on, and the groups flattened equal `order`.
    pub groups: Vec<Vec<String>>,
    /// Items whose plan is incomplete: not in `order`.
    pub refused: Vec<Refusal>,
    /// Items that wait on each other in a loop: not in `order`.
    pub cyclic: Vec<String>,
    /// Items [`Plan::build_capped`] demoted one tier because their tier was full. Empty from [`Plan::build`].
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub demoted: Vec<Demotion>,
    /// The dispatches [`Plan::build_improving`] gave to an improvement request, in order. Empty from the other
    /// builders and when no improvement request is queued and ready.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub turns: Vec<ImprovementTurn>,
    /// The improvement requests [`Plan::build_improving`] raised to urgent because their cause is recorded in
    /// [`REPEAT_URGENT_JOBS`] or more distinct jobs. Empty from the other builders.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub promoted: Vec<RepeatPromotion>,
}

/// The most dispatches in a row that may go to other work while a ready improvement request (a request of
/// Athena's own, from the Steward or from Hestia's reviews) is queued: one dispatch in every three is its.
pub const IMPROVEMENT_GAP: usize = 2;

/// The fewest distinct jobs a cause must be recorded in before the request that addresses it is raised to
/// urgent. Hestia files such a request from [`crate::hestia::REPEAT_JOBS`] (two) jobs; this is the higher bar for
/// treating it as urgent.
pub const REPEAT_URGENT_JOBS: usize = 3;

/// What [`Plan::build_improving`] needs to know about Athena's own improvement requests and about the run so
/// far. Plain data: the drain fills it from the records ([`crate::hestia::improvement_of`]) and from its own
/// count of dispatches.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Improvement {
    /// The ids of the improvement requests: the Steward's and Hestia's.
    pub items: Vec<String>,
    /// For an improvement request that addresses a repeated cause, in how many distinct jobs the cause is
    /// recorded. An id with no entry has no cause count and is never raised to urgent.
    pub cause_jobs: BTreeMap<String, usize>,
    /// How many dispatches in a row have gone to other work, as the run stands. Zero for a plan made before
    /// any dispatch; values above [`IMPROVEMENT_GAP`] count as [`IMPROVEMENT_GAP`].
    pub since_turn: usize,
    /// Items the run has already dispatched. They stay where they are in the order and are not counted or moved.
    pub tried: BTreeSet<String>,
}

/// A dispatch that [`Plan::build_improving`] gave to an improvement request: it stood in the slot after two
/// dispatches to other work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImprovementTurn {
    pub id: String,
    /// The place in the order, counted from 1.
    pub position: usize,
    /// True when the request ranked later and was moved up to this place; false when it already stood here.
    pub moved: bool,
}

/// An improvement request raised to urgent in memory because its cause is recorded in many distinct jobs. The
/// request's own record is not changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RepeatPromotion {
    pub id: String,
    /// In how many distinct jobs the cause is recorded.
    pub jobs: usize,
    /// The priority it had before.
    pub from: Priority,
}

/// How many items a plan keeps at urgent and at high; `usize::MAX` means no cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TierCaps {
    pub urgent: usize,
    pub high: usize,
}

impl TierCaps {
    /// No cap: a plan built with it is the plan [`Plan::build_with`] gives.
    pub const UNLIMITED: TierCaps = TierCaps {
        urgent: usize::MAX,
        high: usize::MAX,
    };
}

/// One item moved down a tier because its tier was over its cap. The item's own record is not changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Demotion {
    pub id: String,
    pub from: Priority,
    pub to: Priority,
    /// The cap of the tier the item came from.
    pub cap: usize,
}

/// Whether two touched paths overlap: equal, or one is a directory holding the other.
fn overlap(a: &str, b: &str) -> bool {
    let inside = |dir: &str, path: &str| {
        let dir = dir.trim_end_matches('/');
        path == dir || path.strip_prefix(dir).is_some_and(|r| r.starts_with('/'))
    };
    inside(a, b) || inside(b, a)
}

/// The shared append-only files that many jobs edit (`docs/design/parallel-jobs.md` section 9). Touching one
/// is not a clash when grouping: a conflicting edit is caught by the landing check, not by the plan.
const SHARED_APPEND_ONLY: [&str; 3] = ["CHANGELOG.md", "docs/decisions.md", "README.md"];

/// The paths of `entry` that count for grouping: its touched paths without the shared append-only files.
fn grouping_paths(entry: &PlanEntry) -> Vec<&str> {
    entry
        .touches
        .iter()
        .map(String::as_str)
        .filter(|p| !SHARED_APPEND_ONLY.contains(p))
        .collect()
}

/// Whether two entries may run side by side: both name some path besides the shared append-only files, and
/// nothing of that they touch overlaps.
fn disjoint(a: &PlanEntry, b: &PlanEntry) -> bool {
    let (pa, pb) = (grouping_paths(a), grouping_paths(b));
    !pa.is_empty() && !pb.is_empty() && shared_path(a, b).is_none()
}

/// The first path of `a` that overlaps a path of `b`, ignoring the shared append-only files: the clash that
/// keeps two entries out of one group. None when they share no path. [`disjoint`] is built on it, so the
/// reason the drain gives for a clash is the rule the plan uses.
pub fn shared_path(a: &PlanEntry, b: &PlanEntry) -> Option<String> {
    let pb = grouping_paths(b);
    grouping_paths(a)
        .into_iter()
        .find(|x| pb.iter().any(|y| overlap(x, y)))
        .map(String::from)
}

/// Builds the parallel groups from the plan `order`, taking items in that order. An item joins the last group
/// when it is [`disjoint`] with every member and depends on none of them; otherwise it opens a new group.
/// Dependencies always sit earlier in `order`, so the last group is the earliest one after everything the item
/// depends on that it may join without being placed ahead of an earlier item it could not join. The groups
/// therefore flatten to exactly `order`: working them in sequence is working the plan in order.
fn groups_in_order(order: &[String], planned: &[&PlanEntry]) -> Vec<Vec<String>> {
    let mut groups: Vec<Vec<&PlanEntry>> = Vec::new();
    for id in order {
        let Some(entry) = planned.iter().find(|e| e.id == *id) else {
            continue;
        };
        match groups.last_mut() {
            Some(last)
                if last
                    .iter()
                    .all(|m| disjoint(m, entry) && !entry.depends_on.contains(&m.id)) =>
            {
                last.push(entry);
            }
            _ => groups.push(vec![entry]),
        }
    }
    groups
        .into_iter()
        .map(|g| g.into_iter().map(|e| e.id.clone()).collect())
        .collect()
}

/// Effective priority of each entry, in the entries' order: its own, raised to the best of whatever depends on
/// it (to a fixpoint).
fn effective_priorities(entries: &[&PlanEntry]) -> Vec<Priority> {
    let mut effective: Vec<Priority> = entries.iter().map(|e| e.priority).collect();
    loop {
        let mut changed = false;
        for i in 0..entries.len() {
            for dep in &entries[i].depends_on {
                if let Some(j) = entries.iter().position(|e| e.id == *dep)
                    && effective[i] < effective[j]
                {
                    effective[j] = effective[i];
                    changed = true;
                }
            }
        }
        if !changed {
            return effective;
        }
    }
}

impl Plan {
    /// The order check: finds every place where this plan's order contradicts the owner's priorities. `entries`
    /// are the ones the plan was built from. The order must never put an item ahead of one with a better
    /// effective priority, so the top-priority work (with what it needs) is in the first wave. Returns one
    /// sentence per contradiction; empty when the order follows priority.
    pub fn priority_violations(&self, entries: &[PlanEntry]) -> Vec<String> {
        let planned: Vec<&PlanEntry> = entries
            .iter()
            .filter(|e| self.position(&e.id).is_some())
            .collect();
        let effective = effective_priorities(&planned);
        let mut ranked: Vec<(usize, &str, Priority)> = planned
            .iter()
            .zip(&effective)
            .filter_map(|(e, p)| Some((self.position(&e.id)?, e.id.as_str(), *p)))
            .collect();
        ranked.sort();
        let mut out = Vec::new();
        for (i, (_, later, later_p)) in ranked.iter().enumerate() {
            // An improvement request moved up to its guaranteed turn is meant to stand ahead of better-ranked
            // work, so it is not an `earlier` item that contradicts the priorities.
            let moved_up = |id: &str| self.turns.iter().any(|t| t.moved && t.id == id);
            if let Some((_, earlier, earlier_p)) = ranked[..i]
                .iter()
                .find(|(_, id, p)| p > later_p && !moved_up(id))
            {
                out.push(format!(
                    "{later} ({}) is planned after {earlier} ({}), which has a lower priority",
                    later_p.as_str(),
                    earlier_p.as_str()
                ));
            }
        }
        out
    }

    /// Plans `entries`, given oldest first (the position is the age used as the tiebreak), with no job history:
    /// a dependency on an id that is not among the entries counts as done. Callers that know how earlier jobs
    /// ended use [`Plan::build_with`], where a dependency on a rejected or failed job with no landed retry holds
    /// the item.
    pub fn build(entries: &[PlanEntry]) -> Plan {
        Plan::build_with(entries, &[])
    }

    /// Plans `entries`, given oldest first (the position is the age used as the tiebreak), with `history`: how
    /// earlier jobs ended. A dependency on an id that is not among the entries is met when the job landed, when
    /// a landed job is linked to it as its retry (directly, or through other retries), or when `history` does
    /// not show it rejected, failed, timed out or stale (an unknown, dropped, declined or abandoned id counts
    /// as done). A dependency on a rejected or failed job with no landed retry is unmet: the item is not
    /// ordered but listed in [`Plan::refused`] with a reason naming the dependency (and an empty `missing`),
    /// and so is everything that depends on a held item. An item that is itself the retry of that dependency
    /// (the history links it to it) is not held by it.
    ///
    /// An item with [`PlanEntry::on_hold`] set (the user ran `athena hold`) is held back the same way at any
    /// priority, with the reason [`on_hold_reason`] and an empty `missing`, even when its plan is incomplete;
    /// whatever depends on it is held back with it. Priority plays no part in it: only the hold keeps such an
    /// item out of the order, and a low-priority item without one is ordered like any other.
    ///
    /// Waves fix the order only (best effective priority among the ready items, then rung, then age). The
    /// groups are then built from that order alone (see `groups_in_order`): an item joins the last group when
    /// it is path-disjoint with its members and independent of them, else it opens a new group. A lower-priority
    /// item therefore never lands in an earlier group than a higher-priority one, and the drain, which works
    /// the groups in sequence, dispatches in plan order. The cost is breadth: an item that clashes with an
    /// earlier one opens a new group, and later disjoint items join that group, not an earlier one.
    pub fn build_with(entries: &[PlanEntry], history: &[JobEnd]) -> Plan {
        let mut plan = Plan::default();
        let mut pending: Vec<(usize, &PlanEntry)> = Vec::new();
        for (age, entry) in entries.iter().enumerate() {
            let missing = entry.missing();
            // An item on hold is kept even when its plan is incomplete, so it is reported as on hold (the
            // reason the user can act on) and not as lacking a plan.
            if missing.is_empty() || entry.on_hold {
                pending.push((age, entry));
            } else {
                let detail = [
                    entry.code_problem.as_deref(),
                    entry.schema_problem.as_deref(),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("; ");
                plan.refused.push(Refusal::lacking_because(
                    &entry.id,
                    &entry.title,
                    missing.into_iter().map(String::from).collect(),
                    Some(detail.as_str()).filter(|d| !d.is_empty()),
                ));
            }
        }
        // Hold items with an unmet dependency, and then whatever depends on a held item, until nothing changes.
        let open: Vec<&str> = pending.iter().map(|(_, e)| e.id.as_str()).collect();
        let mut held: Vec<(&str, String)> = pending
            .iter()
            .filter(|(_, e)| e.on_hold)
            .map(|(_, e)| (e.id.as_str(), on_hold_reason(&e.id)))
            .collect();
        loop {
            let mut changed = false;
            for (_, entry) in &pending {
                if held.iter().any(|(id, _)| *id == entry.id) {
                    continue;
                }
                let reason = entry.depends_on.iter().find_map(|d| {
                    if held.iter().any(|(id, _)| id == d) {
                        Some(format!("depends on {d}, which is held back"))
                    } else if open.contains(&d.as_str()) || retries(&entry.id, d, history) {
                        None
                    } else {
                        unmet_dependency(d, history)
                    }
                });
                if let Some(reason) = reason {
                    held.push((entry.id.as_str(), reason));
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        for (_, entry) in &pending {
            if let Some((_, reason)) = held.iter().find(|(id, _)| *id == entry.id) {
                plan.refused
                    .push(Refusal::held(&entry.id, &entry.title, reason));
            }
        }
        pending.retain(|(_, e)| !held.iter().any(|(id, _)| *id == e.id));
        let known: Vec<&str> = pending.iter().map(|(_, e)| e.id.as_str()).collect();
        let planned: Vec<&PlanEntry> = pending.iter().map(|(_, e)| *e).collect();
        let effective = effective_priorities(&planned);
        let effective_of = |id: &str| {
            known
                .iter()
                .position(|k| *k == id)
                .map_or(Priority::Normal, |i| effective[i])
        };
        let mut done: Vec<String> = Vec::new();
        while !pending.is_empty() {
            let mut ready: Vec<(usize, &PlanEntry)> = pending
                .iter()
                .filter(|(_, e)| {
                    e.depends_on
                        .iter()
                        .all(|d| done.contains(d) || !known.contains(&d.as_str()))
                })
                .copied()
                .collect();
            if ready.is_empty() {
                plan.cyclic = pending.iter().map(|(_, e)| e.id.clone()).collect();
                break;
            }
            if let Some(best) = ready.iter().map(|(_, e)| effective_of(&e.id)).min() {
                ready.retain(|(_, e)| effective_of(&e.id) == best);
            }
            ready.sort_by_key(|(age, e)| (e.rung, *age));
            for (_, e) in &ready {
                plan.order.push(e.id.clone());
                done.push(e.id.clone());
            }
            pending.retain(|(_, e)| !done.contains(&e.id));
        }
        plan.groups = groups_in_order(&plan.order, &planned);
        plan
    }

    /// Plans `entries` like [`Plan::build_with`], then keeps at most `caps.urgent` items urgent and `caps.high`
    /// high. An item over its tier's cap is demoted one tier (urgent to high, high to normal), in memory only:
    /// `entries` and the records they came from are not changed. Returns the plan, built from the demoted
    /// copies with [`Plan::demoted`] listing each move, and those copies, which the order check must judge the
    /// plan against.
    ///
    /// Within a tier the items in `person_set` (jobs whose priority a person set with a priority record) are
    /// counted first and are never demoted, even when they alone exceed the cap. The other items keep the
    /// remaining slots in the plan's own order and the rest drop. An item demoted from urgent counts in the high
    /// tier (after the person-set items, before the items that were high all along) and is not demoted again.
    pub fn build_capped(
        entries: &[PlanEntry],
        history: &[JobEnd],
        caps: TierCaps,
        person_set: &[String],
    ) -> (Plan, Vec<PlanEntry>) {
        let uncapped = Plan::build_with(entries, history);
        let tier_in_order = |tier: Priority| -> Vec<&PlanEntry> {
            uncapped
                .order
                .iter()
                .filter_map(|id| entries.iter().find(|e| e.id == *id))
                .filter(|e| e.priority == tier)
                .collect()
        };
        let by_person = |e: &PlanEntry| person_set.contains(&e.id);
        let mut capped: Vec<PlanEntry> = entries.to_vec();
        let mut demoted: Vec<Demotion> = Vec::new();
        let set_tier = |capped: &mut Vec<PlanEntry>, id: &str, tier: Priority| {
            if let Some(e) = capped.iter_mut().find(|e| e.id == id) {
                e.priority = tier;
            }
        };

        let urgent = tier_in_order(Priority::Urgent);
        let urgent_people = urgent.iter().filter(|e| by_person(e)).count();
        let mut urgent_slots = caps.urgent.saturating_sub(urgent_people);
        let mut from_urgent = 0;
        for e in urgent.iter().filter(|e| !by_person(e)) {
            if urgent_slots > 0 {
                urgent_slots -= 1;
                continue;
            }
            set_tier(&mut capped, &e.id, Priority::High);
            from_urgent += 1;
            demoted.push(Demotion {
                id: e.id.clone(),
                from: Priority::Urgent,
                to: Priority::High,
                cap: caps.urgent,
            });
        }

        let high = tier_in_order(Priority::High);
        let high_people = high.iter().filter(|e| by_person(e)).count();
        let mut high_slots = caps
            .high
            .saturating_sub(high_people)
            .saturating_sub(from_urgent);
        for e in high.iter().filter(|e| !by_person(e)) {
            if high_slots > 0 {
                high_slots -= 1;
                continue;
            }
            set_tier(&mut capped, &e.id, Priority::Normal);
            demoted.push(Demotion {
                id: e.id.clone(),
                from: Priority::High,
                to: Priority::Normal,
                cap: caps.high,
            });
        }

        let mut plan = Plan::build_with(&capped, history);
        plan.demoted = demoted;
        (plan, capped)
    }

    /// Plans `entries` like [`Plan::build_capped`], then gives Athena's own improvement requests
    /// (`improvement.items`: the Steward's and Hestia's) a guaranteed turn.
    ///
    /// Root cause it addresses: the order was set by priority alone, and a person marks requests urgent or high
    /// freely, so a request of Athena's own, which carries the cause of a failure, always sorted behind them
    /// and rarely ran (5 of 32 Steward requests filed 4 to 6 Nov ran).
    ///
    /// 1. **Urgent promotion.** An improvement request whose cause is recorded in [`REPEAT_URGENT_JOBS`] or more
    ///    distinct jobs (`improvement.cause_jobs`) is raised to urgent, in memory, before the tier caps are
    ///    applied, so a full urgent tier can demote it again (see [`Plan::demoted`]). A priority a person set
    ///    (`person_set`) is never changed. Each one raised goes in [`Plan::promoted`].
    /// 2. **Guaranteed turn.** The order is walked once. After [`IMPROVEMENT_GAP`] dispatches in a row to other
    ///    work (counting `improvement.since_turn` for dispatches already made), the next place goes to the best
    ///    ranked improvement request that is ready: it is in the order, so its reading passed and it is not
    ///    held, and every dependency of it in the order is already dispatched (`improvement.tried`) or stands
    ///    before that place. If the request already stands there nothing moves. A request that is not ready is
    ///    skipped and the next place is tried. Every place so filled is a [`Plan::turns`] entry. A moved request
    ///    is a group of its own, so a wave of parallel items cannot push it back.
    ///
    /// With no improvement request among `entries` the result is exactly that of [`Plan::build_capped`]. Items
    /// in `improvement.tried` stay where they are and are neither counted nor moved, so the plan made after each
    /// dispatch keeps the order the first plan showed.
    pub fn build_improving(
        entries: &[PlanEntry],
        history: &[JobEnd],
        caps: TierCaps,
        person_set: &[String],
        improvement: &Improvement,
    ) -> (Plan, Vec<PlanEntry>) {
        let improving = |id: &str| improvement.items.iter().any(|i| i == id);
        if !entries.iter().any(|e| improving(&e.id)) {
            return Plan::build_capped(entries, history, caps, person_set);
        }
        let mut raised: Vec<PlanEntry> = entries.to_vec();
        let mut promoted: Vec<RepeatPromotion> = Vec::new();
        for e in raised.iter_mut().filter(|e| improving(&e.id)) {
            let jobs = improvement.cause_jobs.get(&e.id).copied().unwrap_or(0);
            if jobs >= REPEAT_URGENT_JOBS
                && e.priority != Priority::Urgent
                && !person_set.contains(&e.id)
            {
                promoted.push(RepeatPromotion {
                    id: e.id.clone(),
                    jobs,
                    from: e.priority,
                });
                e.priority = Priority::Urgent;
            }
        }
        let (mut plan, capped) = Plan::build_capped(&raised, history, caps, person_set);
        // Only a request that is in the order was raised in effect: a refused or held one is not planned.
        promoted.retain(|p| plan.position(&p.id).is_some());
        plan.promoted = promoted;

        let depends = |id: &str| -> &[String] {
            capped
                .iter()
                .find(|e| e.id == id)
                .map_or(&[][..], |e| e.depends_on.as_slice())
        };
        let mut order = std::mem::take(&mut plan.order);
        let mut gap = improvement.since_turn.min(IMPROVEMENT_GAP);
        let mut turns: Vec<ImprovementTurn> = Vec::new();
        let mut i = 0;
        while i < order.len() {
            if improvement.tried.contains(&order[i]) {
                i += 1;
                continue;
            }
            if improving(&order[i]) {
                if gap >= IMPROVEMENT_GAP {
                    turns.push(ImprovementTurn {
                        id: order[i].clone(),
                        position: i + 1,
                        moved: false,
                    });
                }
                gap = 0;
            } else if gap < IMPROVEMENT_GAP {
                gap += 1;
            } else {
                // The place is due to an improvement request: the best ranked one that is ready moves here.
                let ready = |j: usize| {
                    let id = order[j].as_str();
                    !improvement.tried.contains(id)
                        && improving(id)
                        && depends(id).iter().all(|d| {
                            improvement.tried.contains(d)
                                || order.iter().position(|o| o == d).is_none_or(|p| p < i)
                        })
                };
                if let Some(j) = (i + 1..order.len()).find(|&j| ready(j)) {
                    let id = order.remove(j);
                    turns.push(ImprovementTurn {
                        id: id.clone(),
                        position: i + 1,
                        moved: true,
                    });
                    order.insert(i, id);
                    gap = 0;
                }
            }
            i += 1;
        }
        if turns.iter().any(|t| t.moved) {
            // The built groups are runs of the unmoved order, so splitting them around each moved request
            // keeps the groups flattening to the moved order.
            let group_of = |id: &str| plan.groups.iter().position(|g| g.iter().any(|m| m == id));
            let moved = |id: &str| turns.iter().any(|t| t.moved && t.id == id);
            let mut groups: Vec<Vec<String>> = Vec::new();
            let mut previous: Option<&String> = None;
            for id in &order {
                let joins = previous
                    .is_some_and(|p| !moved(id) && !moved(p) && group_of(p) == group_of(id));
                match groups.last_mut() {
                    Some(last) if joins => last.push(id.clone()),
                    _ => groups.push(vec![id.clone()]),
                }
                previous = Some(id);
            }
            plan.groups = groups;
        }
        plan.order = order;
        plan.turns = turns;
        (plan, capped)
    }

    /// The position of `id` in the order, if it is planned.
    pub fn position(&self, id: &str) -> Option<usize> {
        self.order.iter().position(|i| i == id)
    }
}

// ---------------------------------------------------------------------------------------------------------
// The plan-stage audit

/// What kind of logged gap this is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GapKind {
    Intervention,
    Introspection,
    Rejection,
    Rating,
}

/// The stage of planning a gap is traced to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    /// The request was misread.
    Reading,
    /// The work was cut into the wrong pieces, or the scope was wrong.
    Decomposition,
    /// An assumption the plan made was wrong.
    Assumptions,
    /// The acceptance criteria missed or misstated what was wanted.
    Criteria,
    /// The check of the outcome did not catch it.
    OutcomeCheck,
}

impl Stage {
    pub const ALL: [Stage; 5] = [
        Stage::Reading,
        Stage::Decomposition,
        Stage::Assumptions,
        Stage::Criteria,
        Stage::OutcomeCheck,
    ];
}

/// Traces a gap's account (an intervention's `what`/`why`, an introspection's cause, a rejection reason, a
/// rating finding) to a stage by the words it uses. Text that names none of the later stages is traced to the
/// reading, the first stage. The tracing is a heuristic: a person reviewing the audit can overrule it.
pub fn trace(text: &str) -> Stage {
    let t = text.to_lowercase();
    let has = |words: &[&str]| words.iter().any(|w| t.contains(w));
    if has(&["criteria", "criterion", "acceptance"]) {
        Stage::Criteria
    } else if has(&["assum"]) {
        Stage::Assumptions
    } else if has(&[
        "split",
        "decompos",
        "scope",
        "too large",
        "too big",
        "deliverable",
    ]) {
        Stage::Decomposition
    } else if has(&[
        "verif",
        "outcome",
        "missed by",
        "not caught",
        "rating",
        "page",
    ]) {
        Stage::OutcomeCheck
    } else {
        Stage::Reading
    }
}

/// One logged gap, traced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Gap {
    pub kind: GapKind,
    pub stage: Stage,
    /// When it was logged, as a count of jobs so far (any steadily rising number works).
    pub at: u64,
}

/// Logged gaps, to be traced to stages and compared before and after a change to a stage.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GapLog {
    pub gaps: Vec<Gap>,
    /// The total number of jobs the log covers; the same unit as [`Gap::at`].
    pub jobs: u64,
}

impl GapLog {
    /// Gaps of `stage`, by kind, as `(kind, count)` for the kinds that occur.
    pub fn by_kind(&self, stage: Stage) -> Vec<(GapKind, usize)> {
        let mut out: Vec<(GapKind, usize)> = Vec::new();
        for g in self.gaps.iter().filter(|g| g.stage == stage) {
            match out.iter_mut().find(|(k, _)| *k == g.kind) {
                Some((_, n)) => *n += 1,
                None => out.push((g.kind, 1)),
            }
        }
        out
    }

    /// Whether gaps of `stage` became rarer per job after the stage was changed at `changed_at`: the rate
    /// after must be below the rate before. False when either side has no jobs to measure by.
    pub fn fell(&self, stage: Stage, changed_at: u64) -> bool {
        let before_jobs = changed_at.min(self.jobs);
        let after_jobs = self.jobs.saturating_sub(changed_at);
        if before_jobs == 0 || after_jobs == 0 {
            return false;
        }
        let at_stage = |g: &&Gap| g.stage == stage;
        let before = self
            .gaps
            .iter()
            .filter(at_stage)
            .filter(|g| g.at < changed_at)
            .count() as f64;
        let after = self
            .gaps
            .iter()
            .filter(at_stage)
            .filter(|g| g.at >= changed_at)
            .count() as f64;
        after / (after_jobs as f64) < before / (before_jobs as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_title_line_skips_priority_lines_and_blanks() {
        assert_eq!(
            title_line("Priority: urgent\n\nFix the widget\nmore"),
            "Fix the widget"
        );
        assert_eq!(
            title_line("  priority: high\nFix the widget"),
            "Fix the widget"
        );
        assert_eq!(
            title_line("Fix the widget\nPriority: low"),
            "Fix the widget"
        );
        assert_eq!(title_line("\n\nFix it"), "Fix it");
        // Only Priority lines: the first line, as before.
        assert_eq!(
            title_line("Priority: high\nPriority: low"),
            "Priority: high"
        );
        assert_eq!(title_line(""), "");
    }

    fn entry(id: &str, rung: Ladder, touches: &[&str], deps: &[&str]) -> PlanEntry {
        PlanEntry {
            goal: format!("goal of {id}"),
            rung: Some(rung),
            acceptance: vec![format!("{id} works")],
            open_questions: vec![],
            depends_on: deps.iter().map(|d| d.to_string()).collect(),
            touches: touches.iter().map(|t| t.to_string()).collect(),
            ..PlanEntry::bare(id, id)
        }
    }

    fn rungs() -> (Ladder, Ladder) {
        (Ladder::BriefRequirements, Ladder::Implementation)
    }

    fn ended(id: &str, word: &str, retry_of: Option<&str>) -> JobEnd {
        JobEnd {
            id: id.into(),
            ending: if word == "landed" {
                Ending::Landed
            } else {
                Ending::Failed(word.into())
            },
            retry_of: retry_of.map(String::from),
        }
    }

    const LEVELS: [Priority; 4] = [
        Priority::Urgent,
        Priority::High,
        Priority::Normal,
        Priority::Low,
    ];

    #[test]
    fn an_on_hold_item_is_not_planned_at_any_priority() {
        let (_, low) = rungs();
        for level in LEVELS {
            let mut held = entry("h", low, &["a/"], &[]);
            held.priority = level;
            held.on_hold = true;
            let plan = Plan::build(std::slice::from_ref(&held));
            assert!(
                plan.order.is_empty() && plan.groups.is_empty(),
                "{level:?}: {plan:?}"
            );
            assert_eq!(plan.refused.len(), 1, "{level:?}");
            assert_eq!(plan.refused[0].id, "h");
            assert!(plan.refused[0].missing.is_empty());
            assert!(
                plan.refused[0].reason.contains("on hold"),
                "{}",
                plan.refused[0].reason
            );
            // Released, the same item is planned.
            held.on_hold = false;
            let plan = Plan::build(&[held]);
            assert_eq!(plan.order, ["h"], "{level:?}");
            assert!(plan.refused.is_empty(), "{level:?}");
        }
    }

    #[test]
    fn an_on_hold_item_with_an_incomplete_plan_is_reported_as_on_hold() {
        let mut held = PlanEntry::bare("h", "h");
        held.on_hold = true;
        let plan = Plan::build(&[held]);
        assert_eq!(plan.refused.len(), 1);
        assert!(plan.refused[0].missing.is_empty(), "{:?}", plan.refused[0]);
        assert!(plan.refused[0].reason.contains("on hold"));
    }

    #[test]
    fn a_low_priority_item_alone_is_still_planned() {
        let (_, low) = rungs();
        let mut only = entry("l", low, &["a/"], &[]);
        only.priority = Priority::Low;
        let plan = Plan::build(&[only.clone()]);
        assert_eq!(plan.order, ["l"]);
        assert!(plan.refused.is_empty(), "{plan:?}");
        // Beside an urgent item on hold, the low one is still ordered: only the hold keeps an item out.
        let mut urgent = entry("u", low, &["b/"], &[]);
        urgent.priority = Priority::Urgent;
        urgent.on_hold = true;
        let plan = Plan::build(&[urgent, only]);
        assert_eq!(plan.order, ["l"]);
        assert_eq!(plan.refused.len(), 1);
        assert_eq!(plan.refused[0].id, "u");
    }

    #[test]
    fn an_item_depending_on_an_on_hold_item_is_held_with_it() {
        let (_, low) = rungs();
        let mut h = entry("h", low, &["a/"], &[]);
        h.priority = Priority::Urgent;
        h.on_hold = true;
        let d = entry("d", low, &["b/"], &["h"]);
        let plan = Plan::build(&[h.clone(), d.clone()]);
        assert!(plan.order.is_empty(), "{plan:?}");
        assert_eq!(plan.refused.len(), 2);
        let reason = &plan.refused.iter().find(|r| r.id == "d").unwrap().reason;
        assert!(
            reason.contains("depends on h, which is held back"),
            "{reason}"
        );
        assert!(plan.refused.iter().all(|r| r.missing.is_empty()));
        h.on_hold = false;
        let plan = Plan::build(&[h, d]);
        assert_eq!(plan.order, ["h", "d"]);
        assert!(plan.refused.is_empty());
    }

    #[test]
    fn an_on_hold_flag_is_left_out_of_the_json_unless_set() {
        let mut e = PlanEntry::bare("h", "h");
        assert!(!serde_json::to_string(&e).unwrap().contains("on_hold"));
        e.on_hold = true;
        assert!(
            serde_json::to_string(&e)
                .unwrap()
                .contains("\"on_hold\":true")
        );
    }

    #[test]
    fn a_dependency_on_a_rejected_job_with_no_landed_retry_holds_the_item_and_its_dependents() {
        let (_, low) = rungs();
        let entries = [entry("b", low, &["a/"], &["x"])];
        // Without a history the id outside the entries counts as done, as before.
        assert_eq!(Plan::build(&entries).order, ["b"]);
        let history = [ended("x", "rejected", None)];
        let plan = Plan::build_with(&entries, &history);
        assert!(plan.order.is_empty() && plan.groups.is_empty(), "{plan:?}");
        assert_eq!(plan.refused.len(), 1);
        let held = &plan.refused[0];
        assert_eq!(held.id, "b");
        assert!(held.missing.is_empty());
        assert!(
            held.reason.contains("depends on x") && held.reason.contains("rejected"),
            "{}",
            held.reason
        );
        // The retry of x names x, but is not held by it; an unrelated item that names x is.
        let entries = [
            entry("b", low, &["a/"], &["x"]),
            entry("retry", low, &["c/"], &["x"]),
        ];
        let history = [
            ended("x", "rejected", None),
            JobEnd {
                id: "retry".into(),
                ending: Ending::Other,
                retry_of: Some("x".into()),
            },
        ];
        let plan = Plan::build_with(&entries, &history);
        assert_eq!(plan.order, ["retry"]);
        assert_eq!(plan.refused.len(), 1);
        assert_eq!(plan.refused[0].id, "b");
        let entries = [entry("b", low, &["a/"], &["x"])];
        // A failed job, a timed-out job and a stale job hold the item too.
        for word in ["failed", "timed out", "stale"] {
            let plan = Plan::build_with(&entries, &[ended("x", word, None)]);
            assert_eq!(plan.refused.len(), 1, "{word}");
            assert!(plan.refused[0].reason.contains(word));
        }
        // Landed, dropped (not a failure end) and unknown dependencies are met.
        assert_eq!(
            Plan::build_with(&entries, &[ended("x", "landed", None)]).order,
            ["b"]
        );
        assert_eq!(Plan::build_with(&entries, &[]).order, ["b"]);
    }

    #[test]
    fn a_dependency_is_met_once_a_retry_of_it_landed_and_the_item_is_ordered_after_it() {
        let (_, low) = rungs();
        let entries = [entry("b", low, &["a/"], &["x"])];
        // x was rejected and re-asked as r1, which was rejected too; r2 retried r1 and landed.
        let mut history = vec![
            ended("x", "rejected", None),
            ended("r1", "rejected", Some("x")),
        ];
        assert_eq!(Plan::build_with(&entries, &history).refused.len(), 1);
        history.push(ended("r2", "landed", Some("r1")));
        let plan = Plan::build_with(&entries, &history);
        assert_eq!(plan.order, ["b"]);
        assert!(plan.refused.is_empty(), "{:?}", plan.refused);

        // A landed job that retries some other job leaves b held; a rejected retry is no help.
        let history = [
            ended("x", "rejected", None),
            ended("other", "landed", Some("y")),
            ended("r", "rejected", Some("x")),
        ];
        assert_eq!(Plan::build_with(&entries, &history).refused.len(), 1);

        // Links that loop end, and a loop that holds no landed job leaves b held.
        let history = [
            ended("x", "rejected", Some("r")),
            ended("r", "rejected", Some("x")),
            ended("l", "landed", Some("r")),
        ];
        assert!(Plan::build_with(&entries, &history).refused.is_empty());
        let history = [
            ended("x", "rejected", Some("r")),
            ended("r", "rejected", Some("x")),
            ended("l", "landed", Some("q")),
            ended("q", "rejected", Some("l")),
        ];
        assert_eq!(Plan::build_with(&entries, &history).refused.len(), 1);

        // The landed retry is also an open item that b needs: b comes after it.
        let entries = [
            entry("b", low, &["a/"], &["x", "r"]),
            entry("r", low, &["c/"], &[]),
        ];
        let history = [
            ended("x", "rejected", None),
            ended("r", "landed", Some("x")),
        ];
        let plan = Plan::build_with(&entries, &history);
        assert_eq!(plan.order, ["r", "b"]);
        assert!(plan.refused.is_empty());
    }

    #[test]
    fn a_dependent_of_a_held_item_is_held_too() {
        let (_, low) = rungs();
        // c and d come before b in the entries, so the hold must repeat until nothing changes.
        let entries = [
            entry("d", low, &["d/"], &["c"]),
            entry("c", low, &["c/"], &["b"]),
            entry("b", low, &["b/"], &["x"]),
            entry("e", low, &["e/"], &[]),
        ];
        let plan = Plan::build_with(&entries, &[ended("x", "failed", None)]);
        assert_eq!(plan.order, ["e"]);
        let held: Vec<(&str, &str)> = plan
            .refused
            .iter()
            .map(|r| (r.id.as_str(), r.reason.as_str()))
            .collect();
        assert_eq!(
            held.iter().map(|(i, _)| *i).collect::<Vec<_>>(),
            ["d", "c", "b"]
        );
        assert!(held[0].1.contains("depends on c, which is held back"));
        assert!(held[1].1.contains("depends on b, which is held back"));
        assert!(held[2].1.contains("depends on x") && held[2].1.contains("failed"));
        assert!(plan.cyclic.is_empty());
    }

    #[test]
    fn a_newer_item_on_a_higher_rung_comes_first_and_age_breaks_ties() {
        let (top, low) = rungs();
        assert!(top < low, "the top rung sorts first");
        let plan = Plan::build(&[
            entry("old", low, &["a/"], &[]),
            entry("older-peer", low, &["b/"], &[]),
            entry("new", top, &["c/"], &[]),
        ]);
        assert_eq!(plan.order, ["new", "old", "older-peer"]);
    }

    #[test]
    fn shared_path_names_the_overlap_and_ignores_append_only_files() {
        let (_, low) = rungs();
        let a = entry("a", low, &["crates/core/", "CHANGELOG.md"], &[]);
        let inside = entry("b", low, &["crates/core/src/x.rs"], &[]);
        let apart = entry("c", low, &["docs/a.md", "CHANGELOG.md"], &[]);
        assert_eq!(shared_path(&a, &inside).as_deref(), Some("crates/core/"));
        assert_eq!(shared_path(&a, &apart), None, "CHANGELOG.md is not a clash");
        assert!(disjoint(&a, &apart));
        assert!(!disjoint(&a, &inside));
    }

    #[test]
    fn disjoint_independent_items_share_a_group_and_dependents_come_after() {
        let (top, low) = rungs();
        let plan = Plan::build(&[
            entry("base", low, &["crates/core/"], &[]),
            entry("docs", low, &["docs/a.md"], &[]),
            entry("clash", low, &["crates/core/src/x.rs"], &[]),
            entry("after", top, &["crates/cli/"], &["base"]),
        ]);
        let group = |ids: &[&str]| ids.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        // `clash` overlaps `base`, so it opens the next group; `after` needs `base` (in an earlier group) and
        // is disjoint from `clash`, so it joins that group. The groups flatten to the order.
        assert_eq!(
            plan.groups,
            [group(&["base", "docs"]), group(&["clash", "after"])]
        );
        assert_eq!(plan.groups.concat(), plan.order);
        assert!(plan.position("after") > plan.position("base"));
    }

    #[test]
    fn an_item_that_names_no_path_is_never_grouped_and_a_loop_is_not_scheduled() {
        let (_, low) = rungs();
        let plan = Plan::build(&[
            entry("x", low, &[], &[]),
            entry("y", low, &["y.md"], &[]),
            entry("p", low, &["p.md"], &["q"]),
            entry("q", low, &["q.md"], &["p"]),
        ]);
        assert_eq!(plan.groups, [vec!["x".to_string()], vec!["y".to_string()]]);
        assert_eq!(plan.cyclic, ["p", "q"]);
    }

    #[test]
    fn a_plan_without_goal_rung_or_acceptance_is_refused() {
        let (_, low) = rungs();
        let mut no_goal = entry("g", low, &["g"], &[]);
        no_goal.goal = " ".into();
        let mut no_rung = entry("r", low, &["r"], &[]);
        no_rung.rung = None;
        let mut no_accept = entry("a", low, &["a"], &[]);
        no_accept.acceptance.clear();
        let plan = Plan::build(&[no_goal, no_rung, no_accept, entry("ok", low, &["o"], &[])]);
        assert_eq!(plan.order, ["ok"]);
        let lacks: Vec<_> = plan.refused.iter().map(|r| r.missing.clone()).collect();
        assert_eq!(lacks, [["goal"], ["ladder rung"], ["acceptance"]]);
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_design_that_names_no_existing_code_is_refused_before_production_and_says_what_is_missing()
    {
        let there = |p: &str| p == "crates/a/src/lib.rs";
        // No file at all.
        let none = code_reference_problem(&[], &[], &there).unwrap();
        assert!(none.contains("lists no file at all"), "{none}");
        // Only a file that is not in the repository and is not marked as created.
        let gone = code_reference_problem(&strings(&["crates/gone.rs"]), &[], &there).unwrap();
        assert!(gone.contains("crates/gone.rs does not exist"), "{gone}");
        assert!(gone.contains("name the existing files"), "{gone}");

        // The changelog Athena adds to every change reading does not count as naming existing code.
        let always = |_: &str| true;
        let only_changelog = code_reference_problem(&strings(&["CHANGELOG.md"]), &[], &always);
        assert!(only_changelog.is_some(), "{only_changelog:?}");

        // The plan refuses such an entry, so it never reaches production, and names the cause.
        let (_, low) = rungs();
        let mut bad = entry("bad", low, &["crates/gone.rs"], &[]);
        bad.code_problem = Some(gone.clone());
        assert_eq!(bad.missing(), [MISSING_CODE]);
        let plan = Plan::build(&[bad, entry("ok", low, &["o"], &[])]);
        assert_eq!(plan.order, ["ok"]);
        assert_eq!(plan.refused.len(), 1);
        assert_eq!(plan.refused[0].missing, [MISSING_CODE]);
        assert!(
            plan.refused[0].reason.contains("existing code")
                && plan.refused[0]
                    .reason
                    .contains("crates/gone.rs does not exist"),
            "{}",
            plan.refused[0].reason
        );
    }

    #[test]
    fn a_design_that_names_existing_code_or_says_it_is_wholly_new_is_accepted() {
        let there = |p: &str| p == "crates/a/src/lib.rs";
        // One existing file is enough, beside a file that is not there yet.
        assert_eq!(
            code_reference_problem(
                &strings(&["crates/a/src/lib.rs", "crates/a/src/new.rs"]),
                &strings(&["crates/a/src/new.rs"]),
                &there
            ),
            None
        );
        // Wholly new code: every listed file is one the design creates.
        assert_eq!(
            code_reference_problem(
                &strings(&["new/a.rs", "new/b.rs"]),
                &strings(&["new/a.rs", "new/b.rs"]),
                &there
            ),
            None
        );
        let (_, low) = rungs();
        let ok = entry("ok", low, &["crates/a/src/lib.rs"], &[]);
        assert!(ok.missing().is_empty());
        assert_eq!(Plan::build(&[ok]).order, ["ok"]);
    }

    #[test]
    fn a_config_edit_without_the_schema_is_refused_and_with_it_accepted() {
        let change = |items: &[(&str, bool)]| -> Vec<(String, bool)> {
            items.iter().map(|(p, d)| (p.to_string(), *d)).collect()
        };
        let schema = "crates/athena-data/src/config.rs";
        let alone = schema_reference_problem(&change(&[("athena.toml", false)])).unwrap();
        assert!(alone.contains("athena.toml"), "{alone}");
        assert!(alone.contains(schema), "{alone}");
        let defaults = schema_reference_problem(&change(&[("defaults.toml", false)])).unwrap();
        assert!(
            defaults.contains("crates/athena-data/src/defaults.rs"),
            "{defaults}"
        );
        assert_eq!(
            schema_reference_problem(&change(&[("athena.toml", false), (schema, false)])),
            None
        );
        assert_eq!(schema_reference_problem(&change(&[("a.rs", false)])), None);

        let (_, low) = rungs();
        let mut bad = entry("bad", low, &["athena.toml"], &[]);
        bad.schema_problem = Some(alone);
        assert_eq!(bad.missing(), [MISSING_SCHEMA]);
        // Both problems at once are both reported.
        bad.code_problem = Some("no code".into());
        assert_eq!(bad.missing(), [MISSING_CODE, MISSING_SCHEMA]);
        let plan = Plan::build(&[bad, entry("ok", low, &["o"], &[])]);
        assert_eq!(plan.order, ["ok"]);
        assert!(
            plan.refused[0]
                .reason
                .contains("no code; the design changes athena.toml"),
            "{}",
            plan.refused[0].reason
        );
    }

    #[test]
    fn a_vague_entry_is_flagged_with_its_missing_fields_and_left_out_of_the_order() {
        let (_, low) = rungs();
        // What reading::plan_entry gives "make it better": a goal and nothing else.
        let vague = PlanEntry {
            goal: "make it better".into(),
            ..PlanEntry::bare("vague", "make it better")
        };
        let plan = Plan::build(&[vague, entry("ok", low, &["ok.md"], &[])]);
        assert_eq!(plan.order, ["ok"]);
        assert!(!plan.order.contains(&"vague".to_string()));
        assert_eq!(plan.refused.len(), 1);
        assert_eq!(plan.refused[0].missing, ["ladder rung", "acceptance"]);
        assert!(plan.refused[0].reason.contains("acceptance"));
    }

    #[test]
    fn two_items_naming_the_same_path_are_in_different_groups_and_disjoint_ones_share_one() {
        let (_, low) = rungs();
        let plan = Plan::build(&[
            entry("one", low, &["crates/x/src/lib.rs"], &[]),
            entry("two", low, &["crates/x/src/lib.rs"], &[]),
            entry("three", low, &["docs/a.md"], &[]),
            entry("four", low, &["docs/b.md"], &[]),
        ]);
        let group_of = |id: &str| {
            plan.groups
                .iter()
                .position(|g| g.iter().any(|i| i == id))
                .unwrap()
        };
        // Groups follow the order: `two` clashes with `one`, so it opens a new group, which the later
        // disjoint items join (they are not placed ahead of `two`).
        assert_ne!(group_of("one"), group_of("two"));
        assert_eq!(group_of("two"), group_of("three"));
        assert_eq!(group_of("three"), group_of("four"));
        assert_eq!(plan.groups.concat(), plan.order);
    }

    #[test]
    fn shared_append_only_files_are_not_a_clash_but_other_shared_paths_and_lone_shared_entries_are()
    {
        let (_, low) = rungs();
        fn with(p: &str) -> [&str; 3] {
            [p, "CHANGELOG.md", "docs/decisions.md"]
        }
        let plan = Plan::build(&[
            entry("one", low, &with("crates/a/src/lib.rs"), &[]),
            entry("two", low, &with("crates/b/src/lib.rs"), &[]),
            entry("same", low, &with("crates/a/src/lib.rs"), &[]),
            entry("only", low, &["CHANGELOG.md"], &[]),
            entry("only2", low, &["CHANGELOG.md"], &[]),
        ]);
        let group_of = |id: &str| {
            plan.groups
                .iter()
                .position(|g| g.iter().any(|i| i == id))
                .unwrap()
        };
        assert_eq!(group_of("one"), group_of("two"));
        assert_ne!(group_of("one"), group_of("same"));
        assert_eq!(plan.groups[group_of("only")], ["only"]);
        assert_eq!(plan.groups[group_of("only2")], ["only2"]);
    }

    #[test]
    fn a_dependency_outside_the_plan_counts_as_done() {
        let (_, low) = rungs();
        let plan = Plan::build(&[entry("a", low, &["a"], &["landed-long-ago"])]);
        assert_eq!(plan.order, ["a"]);
    }

    fn with_priority(mut e: PlanEntry, p: Priority) -> PlanEntry {
        e.priority = p;
        e
    }

    const CAPS: TierCaps = TierCaps {
        urgent: 3,
        high: 10,
    };

    fn tiered(prefix: &str, count: usize, tier: Priority) -> Vec<PlanEntry> {
        let (_, low) = rungs();
        (1..=count)
            .map(|i| {
                let id = format!("{prefix}{i}");
                with_priority(entry(&id, low, &[&format!("{id}/")], &[]), tier)
            })
            .collect()
    }

    fn priority_of(entries: &[PlanEntry], id: &str) -> Priority {
        entries.iter().find(|e| e.id == id).unwrap().priority
    }

    #[test]
    fn five_urgent_items_keep_three_urgent_and_a_person_set_one_is_among_them() {
        let entries = tiered("u", 5, Priority::Urgent);
        let people = vec!["u5".to_string()];
        let (plan, capped) = Plan::build_capped(&entries, &[], CAPS, &people);
        let urgent: Vec<&str> = capped
            .iter()
            .filter(|e| e.priority == Priority::Urgent)
            .map(|e| e.id.as_str())
            .collect();
        // The person-set item is the youngest, yet it keeps its tier; the oldest others fill the rest.
        assert_eq!(urgent, ["u1", "u2", "u5"]);
        assert_eq!(plan.order[..3], ["u1", "u2", "u5"]);
        assert_eq!(
            plan.demoted,
            [
                Demotion {
                    id: "u3".into(),
                    from: Priority::Urgent,
                    to: Priority::High,
                    cap: 3
                },
                Demotion {
                    id: "u4".into(),
                    from: Priority::Urgent,
                    to: Priority::High,
                    cap: 3
                },
            ]
        );
        assert_eq!(plan.order[3..], ["u3", "u4"]);
        // The order follows the tiers the plan was built from, so the check finds no contradiction.
        assert!(plan.priority_violations(&capped).is_empty());
        // The input is unchanged.
        assert!(entries.iter().all(|e| e.priority == Priority::Urgent));
    }

    #[test]
    fn a_person_set_item_is_never_demoted_even_when_they_alone_exceed_the_cap() {
        let entries = tiered("u", 4, Priority::Urgent);
        let people: Vec<String> = ["u1", "u2", "u3", "u4"].map(String::from).to_vec();
        let (plan, capped) = Plan::build_capped(&entries, &[], CAPS, &people);
        assert!(plan.demoted.is_empty());
        assert!(capped.iter().all(|e| e.priority == Priority::Urgent));
        // With three of four set by a person, the one other item is the one that drops.
        let (plan, _) = Plan::build_capped(&entries, &[], CAPS, &people[1..]);
        assert_eq!(plan.demoted.len(), 1);
        assert_eq!(plan.demoted[0].id, "u1");
    }

    #[test]
    fn twelve_high_items_keep_ten_and_a_person_set_one_wins() {
        let entries = tiered("h", 12, Priority::High);
        let people = vec!["h12".to_string()];
        let (plan, capped) = Plan::build_capped(&entries, &[], CAPS, &people);
        let ids: Vec<&str> = plan.demoted.iter().map(|d| d.id.as_str()).collect();
        assert_eq!(ids, ["h10", "h11"]);
        assert!(
            plan.demoted
                .iter()
                .all(|d| d.from == Priority::High && d.to == Priority::Normal && d.cap == 10)
        );
        assert_eq!(priority_of(&capped, "h12"), Priority::High);
        assert_eq!(priority_of(&capped, "h10"), Priority::Normal);
        assert_eq!(plan.order.len(), 12);
        assert!(plan.priority_violations(&capped).is_empty());
    }

    #[test]
    fn an_item_demoted_from_urgent_counts_in_the_high_tier_and_is_not_demoted_twice() {
        let mut entries = tiered("u", 5, Priority::Urgent);
        entries.extend(tiered("h", 9, Priority::High));
        let (plan, capped) = Plan::build_capped(&entries, &[], CAPS, &[]);
        // Two drop from urgent to high, which then holds 9 + 2 = 11: the youngest high item drops to normal.
        let moves: Vec<(&str, Priority, Priority)> = plan
            .demoted
            .iter()
            .map(|d| (d.id.as_str(), d.from, d.to))
            .collect();
        assert_eq!(
            moves,
            [
                ("u4", Priority::Urgent, Priority::High),
                ("u5", Priority::Urgent, Priority::High),
                ("h9", Priority::High, Priority::Normal),
            ]
        );
        assert_eq!(priority_of(&capped, "u4"), Priority::High);
        assert_eq!(priority_of(&capped, "h9"), Priority::Normal);
    }

    #[test]
    fn no_cap_and_items_under_the_caps_change_nothing() {
        let mut entries = tiered("u", 5, Priority::Urgent);
        entries.extend(tiered("h", 12, Priority::High));
        let (plan, capped) = Plan::build_capped(&entries, &[], TierCaps::UNLIMITED, &[]);
        assert_eq!(plan, Plan::build(&entries));
        assert_eq!(capped, entries);
        let few = tiered("u", 3, Priority::Urgent);
        let (plan, _) = Plan::build_capped(&few, &[], CAPS, &[]);
        assert!(plan.demoted.is_empty());
    }

    #[test]
    fn a_newer_high_priority_item_comes_before_older_normal_ones() {
        let (_, low) = rungs();
        let plan = Plan::build(&[
            entry("n1", low, &["a/"], &[]),
            entry("n2", low, &["b/"], &[]),
            entry("n3", low, &["c/"], &[]),
            with_priority(entry("h", low, &["d/"], &[]), Priority::High),
        ]);
        assert_eq!(plan.order, ["h", "n1", "n2", "n3"]);
    }

    #[test]
    fn priority_ranks_high_then_normal_then_low() {
        assert!(Priority::High < Priority::Normal && Priority::Normal < Priority::Low);
        let (_, low) = rungs();
        let plan = Plan::build(&[
            with_priority(entry("l", low, &["a/"], &[]), Priority::Low),
            entry("n", low, &["b/"], &[]),
        ]);
        assert_eq!(plan.order, ["n", "l"]);
    }

    #[test]
    fn the_order_check_flags_a_plan_that_puts_a_lower_priority_item_first() {
        let (_, low) = rungs();
        let entries = vec![
            entry("a", low, &["a/"], &[]),
            with_priority(entry("b", low, &["b/"], &[]), Priority::Low),
            with_priority(entry("c", low, &["c/"], &[]), Priority::Urgent),
            with_priority(entry("d", low, &["d/"], &["b"]), Priority::High),
            entry("e", low, &["e/"], &[]),
        ];
        // A plan built by `build` follows priority, so the check passes it.
        let good = Plan::build(&entries);
        assert_eq!(good.order, ["c", "b", "d", "a", "e"]);
        assert!(good.priority_violations(&entries).is_empty());
        // Age order puts the urgent item third: the check names it.
        let bad = Plan {
            order: ["a", "b", "c", "d", "e"].map(String::from).to_vec(),
            ..Plan::default()
        };
        let found = bad.priority_violations(&entries);
        assert!(
            found.iter().any(|m| m.starts_with("c (urgent)")),
            "{found:?}"
        );
        assert!(found.iter().any(|m| m.starts_with("b (high)")), "{found:?}");
        // Items equal in priority never contradict each other.
        let same = vec![entry("x", low, &["x/"], &[]), entry("y", low, &["y/"], &[])];
        let swapped = Plan {
            order: vec!["y".into(), "x".into()],
            ..Plan::default()
        };
        assert!(swapped.priority_violations(&same).is_empty());
    }

    #[test]
    fn dependencies_win_over_priority_and_pull_their_prerequisites_forward() {
        let (_, low) = rungs();
        let plan = Plan::build(&[
            entry("n1", low, &["a/"], &[]),
            entry("n2", low, &["b/"], &[]),
            entry("n3", low, &["c/"], &[]),
            with_priority(entry("h", low, &["d/"], &["n1"]), Priority::High),
        ]);
        assert_eq!(plan.order, ["n1", "h", "n2", "n3"]);
        // A low item with a high dependent runs before the normals.
        let plan = Plan::build(&[
            entry("n", low, &["a/"], &[]),
            with_priority(entry("base", low, &["b/"], &[]), Priority::Low),
            with_priority(entry("h", low, &["c/"], &["base"]), Priority::High),
        ]);
        assert_eq!(plan.order, ["base", "h", "n"]);
    }

    #[test]
    fn urgent_comes_before_high_before_normal_and_a_dependency_still_comes_first() {
        let (_, low) = rungs();
        assert!(Priority::Urgent < Priority::High);
        let plan = Plan::build(&[
            entry("n", low, &["a/"], &[]),
            with_priority(entry("h", low, &["b/"], &[]), Priority::High),
            with_priority(entry("u", low, &["c/"], &[]), Priority::Urgent),
        ]);
        assert_eq!(plan.order, ["u", "h", "n"]);
        // An urgent item that needs a normal one: the normal one runs first, then the urgent one, ahead of
        // an unrelated high item.
        let plan = Plan::build(&[
            with_priority(entry("h", low, &["b/"], &[]), Priority::High),
            entry("base", low, &["a/"], &[]),
            with_priority(entry("u", low, &["c/"], &["base"]), Priority::Urgent),
        ]);
        assert_eq!(plan.order, ["base", "u", "h"]);
    }

    #[test]
    fn without_priorities_the_order_is_rung_then_age() {
        let (top, low) = rungs();
        let plan = Plan::build(&[
            entry("a", low, &["a/"], &[]),
            entry("b", top, &["b/"], &[]),
            entry("c", low, &["c/"], &[]),
            entry("d", top, &["d/"], &["a"]),
        ]);
        assert_eq!(plan.order, ["b", "a", "c", "d"]);
        assert_eq!(plan.groups, [vec!["b", "a", "c"], vec!["d"]]);
    }

    #[test]
    fn an_approved_proposal_is_normal_unless_a_record_or_its_role_says_otherwise() {
        use Priority::{High, Low, Normal, Urgent};
        // Nothing says otherwise: normal.
        assert_eq!(proposal_priority(None, Normal, Normal), Normal);
        // The role's default applies when the job's own words say nothing; what they do say beats the role.
        assert_eq!(proposal_priority(None, Normal, High), High);
        assert_eq!(proposal_priority(None, Normal, Low), Low);
        assert_eq!(proposal_priority(None, Urgent, High), Urgent);
        assert_eq!(proposal_priority(None, Low, High), Low);
        assert_eq!(proposal_priority(None, Low, Normal), Low);
        // A priority record is the user's word and decides alone, even against an urgent role.
        assert_eq!(proposal_priority(Some(Low), Urgent, Urgent), Low);
        assert_eq!(proposal_priority(Some(Normal), High, High), Normal);
        assert_eq!(proposal_priority(Some(Urgent), Normal, Normal), Urgent);
    }

    #[test]
    fn an_approved_proposal_is_ordered_among_other_items_by_dependency_priority_rung_then_age() {
        let (top, low) = rungs();
        let proposal = |id: &str, rung, deps: &[&str], priority| {
            with_priority(entry(id, rung, &[id], deps), priority)
        };
        // The proposal's job is old and on the lowest rung, with the default priority.
        let normal = proposal(
            "proposal-job",
            low,
            &[],
            proposal_priority(None, Priority::Normal, Priority::Normal),
        );
        let urgent_request =
            with_priority(entry("urgent-request", low, &["u"], &[]), Priority::Urgent);
        let plan = Plan::build(&[normal.clone(), urgent_request.clone()]);
        assert_eq!(
            plan.order,
            ["urgent-request", "proposal-job"],
            "priority first"
        );

        // Against normal requests: a higher rung first, then age (the proposal is oldest).
        let plan = Plan::build(&[
            normal.clone(),
            entry("old-request", low, &["o"], &[]),
            entry("ladder-request", top, &["l"], &[]),
        ]);
        assert_eq!(
            plan.order,
            ["ladder-request", "proposal-job", "old-request"]
        );

        // A role that states a higher default puts the proposal ahead of the older normal request.
        let by_role = proposal(
            "proposal-job",
            low,
            &[],
            proposal_priority(None, Priority::Normal, Priority::High),
        );
        let plan = Plan::build(&[
            entry("old-request", low, &["o"], &[]),
            by_role,
            urgent_request.clone(),
        ]);
        assert_eq!(
            plan.order,
            ["urgent-request", "proposal-job", "old-request"]
        );

        // A priority record of low puts it behind even though its role says high.
        let recorded = proposal(
            "proposal-job",
            low,
            &[],
            proposal_priority(Some(Priority::Low), Priority::Normal, Priority::High),
        );
        let plan = Plan::build(&[recorded, entry("old-request", low, &["o"], &[])]);
        assert_eq!(plan.order, ["old-request", "proposal-job"]);

        // Dependency still comes first: an urgent request that needs the proposal's job runs after it.
        let needs = with_priority(
            entry("needs", low, &["n"], &["proposal-job"]),
            Priority::Urgent,
        );
        let plan = Plan::build(&[normal, needs]);
        assert_eq!(plan.order, ["proposal-job", "needs"]);
    }

    #[test]
    fn priority_is_read_from_a_line_in_the_words() {
        assert_eq!(
            Priority::from_text("Fix it\nPriority: high\n"),
            Priority::High
        );
        assert_eq!(Priority::from_text("x\n  priority: HIGH"), Priority::High);
        assert_eq!(Priority::from_text("Priority: low"), Priority::Low);
        assert_eq!(Priority::from_text("Fix it"), Priority::Normal);
        assert_eq!(Priority::from_text("Priority: urgent"), Priority::Urgent);
        assert_eq!(Priority::from_text("Priority: asap"), Priority::Normal);
        assert_eq!(Priority::parse(" Urgent "), Some(Priority::Urgent));
        assert_eq!(Priority::parse("bogus"), None);
        assert_eq!(
            Priority::from_text("this is a high priority: high stakes"),
            Priority::Normal
        );
    }

    #[test]
    fn gaps_are_traced_to_stages() {
        assert_eq!(trace("the acceptance criteria missed it"), Stage::Criteria);
        assert_eq!(
            trace("a wrong assumption about the API"),
            Stage::Assumptions
        );
        assert_eq!(
            trace("the work was too large; split it"),
            Stage::Decomposition
        );
        assert_eq!(trace("verification did not catch it"), Stage::OutcomeCheck);
        assert_eq!(trace("it misunderstood the request"), Stage::Reading);
        assert_eq!(Stage::ALL.len(), 5);
    }

    #[test]
    fn a_stage_fell_when_its_gaps_per_job_dropped_after_the_change() {
        let gap = |at| Gap {
            kind: GapKind::Intervention,
            stage: Stage::Criteria,
            at,
        };
        let log = GapLog {
            gaps: vec![
                gap(1),
                gap(2),
                gap(3),
                gap(8),
                Gap {
                    kind: GapKind::Rating,
                    stage: Stage::Reading,
                    at: 9,
                },
            ],
            jobs: 10,
        };
        assert!(log.fell(Stage::Criteria, 5), "3 in 5 jobs, then 1 in 5");
        assert!(!log.fell(Stage::Reading, 5), "it rose from none to one");
        assert!(!log.fell(Stage::Criteria, 0), "nothing before to compare");
        assert_eq!(log.by_kind(Stage::Criteria), [(GapKind::Intervention, 4)]);
    }

    // -----------------------------------------------------------------------------------------------------
    // The guaranteed turn of Athena's own improvement requests

    fn improvement(items: &[&str]) -> Improvement {
        Improvement {
            items: items.iter().map(|s| s.to_string()).collect(),
            ..Improvement::default()
        }
    }

    /// Six person-filed urgent requests (`u1`..`u6`, in this order) and two normal Steward requests, oldest last.
    fn group_index(plan: &Plan, id: &str) -> usize {
        plan.groups
            .iter()
            .position(|g| g.iter().any(|m| m == id))
            .unwrap_or_else(|| panic!("{id} is in no group: {:?}", plan.groups))
    }

    #[test]
    fn a_dependency_group_comes_before_its_dependent_whatever_their_priorities() {
        let (top, low) = rungs();
        // (dependency priority, dependent priority, a distinct unrelated item's priority)
        let cases = [
            (Priority::Normal, Priority::Urgent, Priority::Normal),
            (Priority::Urgent, Priority::Low, Priority::Normal),
            (Priority::Normal, Priority::High, Priority::Urgent),
            (Priority::Low, Priority::Low, Priority::Urgent),
        ];
        for (base_p, after_p, other_p) in cases {
            // All three touch different paths, so only the dependency keeps `after` from joining `base`.
            let entries = [
                with_priority(entry("base", low, &["base/"], &[]), base_p),
                with_priority(entry("other", low, &["other/"], &[]), other_p),
                with_priority(entry("after", top, &["after/"], &["base"]), after_p),
            ];
            let plan = Plan::build(&entries);
            let label = format!("{base_p:?} base, {after_p:?} dependent, {other_p:?} other");
            assert!(
                plan.position("base") < plan.position("after"),
                "{label}: {:?}",
                plan.order
            );
            assert!(
                group_index(&plan, "base") < group_index(&plan, "after"),
                "{label}: {:?}",
                plan.groups
            );
            assert_eq!(plan.groups.concat(), plan.order, "{label}");
        }
    }

    /// Whether `id` is `of` or something `of` depends on, directly or through other entries.
    fn needed_by(entries: &[PlanEntry], id: &str, of: &str) -> bool {
        let mut stack = vec![of.to_string()];
        let mut seen = BTreeSet::new();
        while let Some(cur) = stack.pop() {
            if !seen.insert(cur.clone()) {
                continue;
            }
            if let Some(e) = entries.iter().find(|e| e.id == cur) {
                for d in &e.depends_on {
                    if d == id {
                        return true;
                    }
                    stack.push(d.clone());
                }
            }
        }
        false
    }

    /// Every pair where a later item has a better own priority than an earlier one, with no item at least as
    /// urgent as the later one depending (transitively) on the earlier one.
    fn unexplained_inversions(plan: &Plan, entries: &[PlanEntry]) -> Vec<String> {
        let prio = |id: &str| entries.iter().find(|e| e.id == id).unwrap().priority;
        let mut out = Vec::new();
        for (i, x) in plan.order.iter().enumerate() {
            for y in &plan.order[i + 1..] {
                if prio(y) < prio(x)
                    && !entries.iter().any(|z| {
                        prio(&z.id) <= prio(y) && (z.id == *x || needed_by(entries, x, &z.id))
                    })
                {
                    out.push(format!("{y} is after {x} with no dependency reason"));
                }
            }
        }
        out
    }

    #[test]
    fn groups_flatten_to_the_order_and_a_later_group_never_outranks_an_earlier_one_without_a_dependency()
     {
        let (top, low) = rungs();
        // The 2026-10-27 shape: u1 and u2 clash; normal `n` is a dependency of urgent `u4`; `u3` is urgent.
        let entries = vec![
            with_priority(entry("u1", low, &["src/p.rs"], &[]), Priority::Urgent),
            with_priority(entry("u2", low, &["src/p.rs"], &[]), Priority::Urgent),
            entry("n", low, &["n/"], &[]),
            with_priority(entry("u3", low, &["u3/"], &[]), Priority::Urgent),
            with_priority(entry("u4", low, &["u4/"], &["n"]), Priority::Urgent),
            with_priority(entry("h", top, &["h/"], &[]), Priority::High),
            entry("m", low, &["m/"], &[]),
            with_priority(entry("l", low, &["l/"], &[]), Priority::Low),
        ];
        let ids = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let plan = Plan::build(&entries);
        assert_eq!(
            plan.order,
            ids(&["u1", "u2", "n", "u3", "u4", "h", "m", "l"])
        );
        assert_eq!(
            plan.groups,
            [
                ids(&["u1"]),
                ids(&["u2", "n", "u3"]),
                ids(&["u4", "h", "m", "l"])
            ]
        );
        assert!(plan.priority_violations(&entries).is_empty());
        assert_eq!(
            unexplained_inversions(&plan, &entries),
            Vec::<String>::new()
        );

        let (capped, capped_entries) = Plan::build_capped(&entries, &[], CAPS, &[]);
        assert_eq!(capped.groups.concat(), capped.order);
        assert!(capped.priority_violations(&capped_entries).is_empty());
        assert_eq!(
            unexplained_inversions(&capped, &capped_entries),
            Vec::<String>::new()
        );

        // With a steward request moved up, the groups still flatten to the moved order.
        let mut with_steward = entries.clone();
        with_steward.push(entry("s1", low, &["s1/"], &[]));
        let (improved, _) = Plan::build_improving(
            &with_steward,
            &[],
            TierCaps::UNLIMITED,
            &[],
            &improvement(&["s1"]),
        );
        assert!(
            improved.turns.iter().any(|t| t.moved),
            "{:?}",
            improved.turns
        );
        assert_eq!(improved.groups.concat(), improved.order);
        let unmoved = Plan::build(&with_steward);
        assert_eq!(unmoved.groups.concat(), unmoved.order);
    }

    fn six_urgent_and_two_steward() -> Vec<PlanEntry> {
        let (_, low) = rungs();
        let mut entries: Vec<PlanEntry> = (1..=6)
            .map(|n| {
                with_priority(
                    entry(&format!("u{n}"), low, &[&format!("u{n}/")], &[]),
                    Priority::Urgent,
                )
            })
            .collect();
        entries.push(entry("s1", low, &["s1/"], &[]));
        entries.push(entry("s2", low, &["s2/"], &[]));
        entries
    }

    fn person_urgent() -> Vec<String> {
        (1..=6).map(|n| format!("u{n}")).collect()
    }

    #[test]
    fn improvement_turn_puts_a_steward_request_among_the_first_three_dispatches() {
        let entries = six_urgent_and_two_steward();
        let steward = improvement(&["s1", "s2"]);
        // Without the rule both Steward requests are last, behind the six urgent ones.
        let (before, _) = Plan::build_capped(&entries, &[], TierCaps::UNLIMITED, &person_urgent());
        assert_eq!(before.order[6..], ["s1", "s2"]);
        // Case A: no caps. Case B: the default caps, with the six urgent ones set by a person, so all stay urgent.
        for (caps, person) in [
            (TierCaps::UNLIMITED, Vec::new()),
            (
                TierCaps {
                    urgent: 3,
                    high: 10,
                },
                person_urgent(),
            ),
        ] {
            let (plan, _) = Plan::build_improving(&entries, &[], caps, &person, &steward);
            assert!(
                plan.order[..3].iter().any(|id| id.starts_with('s')),
                "{:?}",
                plan.order
            );
            assert_eq!(plan.order, ["u1", "u2", "s1", "u3", "u4", "s2", "u5", "u6"]);
            assert_eq!(
                plan.turns,
                [
                    ImprovementTurn {
                        id: "s1".into(),
                        position: 3,
                        moved: true
                    },
                    ImprovementTurn {
                        id: "s2".into(),
                        position: 6,
                        moved: true
                    },
                ]
            );
            // A moved request is a group of its own, so a wave cannot dispatch it behind its neighbours.
            for id in ["s1", "s2"] {
                assert!(
                    plan.groups.iter().any(|g| g == &[id.to_string()]),
                    "{:?}",
                    plan.groups
                );
            }
            let flat: Vec<&String> = plan.groups.iter().flatten().collect();
            assert_eq!(flat, plan.order.iter().collect::<Vec<_>>());
            assert!(
                plan.groups.iter().all(|g| g.len() <= 2),
                "{:?}",
                plan.groups
            );
        }
    }

    #[test]
    fn improvement_turn_counts_the_dispatches_already_made() {
        let entries = six_urgent_and_two_steward();
        let build = |since_turn: usize, tried: &[&str]| {
            let mut improving = improvement(&["s1", "s2"]);
            improving.since_turn = since_turn;
            improving.tried = tried.iter().map(|s| s.to_string()).collect();
            Plan::build_improving(&entries, &[], TierCaps::UNLIMITED, &[], &improving).0
        };
        // Two dispatches have already gone to other work: the very next one is the turn.
        let plan = build(2, &["u1", "u2"]);
        assert_eq!(plan.order[2], "s1");
        assert_eq!(plan.turns[0].position, 3);
        // One has: the second place is the turn. Counts above the gap count as the gap.
        assert_eq!(build(1, &["u1"]).order[1..3], ["u2", "s1"]);
        assert_eq!(build(9, &[]).order[0], "s1");
        // A request already dispatched stays where it is, is not counted and is not moved.
        let mut improving = improvement(&["s1", "s2"]);
        improving.tried = ["s1".to_string()].into();
        let (plan, _) = Plan::build_improving(&entries, &[], TierCaps::UNLIMITED, &[], &improving);
        assert_eq!(plan.order, ["u1", "u2", "s2", "u3", "u4", "u5", "u6", "s1"]);
        assert_eq!(plan.turns.len(), 1);
        assert_eq!(plan.turns[0].id, "s2");
    }

    #[test]
    fn improvement_turn_replans_keep_the_order_the_first_plan_showed() {
        let entries = six_urgent_and_two_steward();
        let first = Plan::build_improving(
            &entries,
            &[],
            TierCaps::UNLIMITED,
            &[],
            &improvement(&["s1", "s2"]),
        )
        .0;
        // The drain plans again after each dispatch with its count and its tried set.
        let mut improving = improvement(&["s1", "s2"]);
        let mut dispatched: Vec<String> = Vec::new();
        for _ in 0..entries.len() {
            let (plan, _) =
                Plan::build_improving(&entries, &[], TierCaps::UNLIMITED, &[], &improving);
            let next = plan
                .order
                .iter()
                .find(|id| !improving.tried.contains(*id))
                .unwrap()
                .clone();
            improving.since_turn = if next.starts_with('s') {
                0
            } else {
                improving.since_turn + 1
            };
            improving.tried.insert(next.clone());
            dispatched.push(next);
        }
        assert_eq!(dispatched, first.order);
        // At most two dispatches to other work in a row while an improvement request waits.
        let mut run = 0;
        for id in &dispatched {
            run = if id.starts_with('s') { 0 } else { run + 1 };
            assert!(run <= IMPROVEMENT_GAP, "{dispatched:?}");
        }
    }

    #[test]
    fn improvement_turn_waits_for_nothing_but_leaves_a_better_ranked_request_in_place() {
        let (_, low) = rungs();
        // The Steward request already ranks first: it is at the front, so nothing moves.
        let entries = [
            with_priority(entry("s1", low, &["s1/"], &[]), Priority::Urgent),
            with_priority(entry("u1", low, &["u1/"], &[]), Priority::High),
            entry("n1", low, &["n1/"], &[]),
        ];
        let (plan, _) = Plan::build_improving(
            &entries,
            &[],
            TierCaps::UNLIMITED,
            &[],
            &improvement(&["s1"]),
        );
        let (plain, _) = Plan::build_capped(&entries, &[], TierCaps::UNLIMITED, &[]);
        assert_eq!(plan.order, plain.order);
        assert_eq!(plan.groups, plain.groups);
        assert!(plan.turns.is_empty(), "it was not due: {:?}", plan.turns);
    }

    #[test]
    fn build_improving_without_improvement_items_equals_build_capped() {
        let (top, low) = rungs();
        let entries = vec![
            entry("a", low, &["a/"], &[]),
            with_priority(entry("b", top, &["b/"], &[]), Priority::High),
            with_priority(entry("c", low, &["c/"], &["a"]), Priority::Urgent),
            with_priority(entry("d", low, &["d/"], &[]), Priority::Low),
            entry("e", low, &["a/x"], &[]),
            entry("f", low, &["f/"], &[]),
            PlanEntry::bare("g", "g has no plan"),
            entry("h", low, &["h/"], &["x"]),
            with_priority(entry("i", low, &["i/"], &[]), Priority::Urgent),
            with_priority(entry("j", low, &["j/"], &[]), Priority::Urgent),
            with_priority(entry("k", low, &["k/"], &[]), Priority::Urgent),
        ];
        let history = [ended("x", "rejected", None)];
        for caps in [
            TierCaps::UNLIMITED,
            TierCaps {
                urgent: 3,
                high: 10,
            },
        ] {
            let person = ["i".to_string()];
            let expected = Plan::build_capped(&entries, &history, caps, &person);
            assert!(expected.0.refused.len() >= 2, "a refused and a held item");
            // No improvement request queued; one that is not among the entries; one that is refused.
            for items in [&[][..], &["not-queued"][..], &["g"][..], &["h"][..]] {
                let got =
                    Plan::build_improving(&entries, &history, caps, &person, &improvement(items));
                assert_eq!(got, expected, "items {items:?}");
            }
        }
    }

    fn repeat_entries() -> Vec<PlanEntry> {
        let (_, low) = rungs();
        vec![
            entry("n1", low, &["n1/"], &[]),
            entry("n2", low, &["n2/"], &[]),
            entry("h", low, &["h/"], &[]),
        ]
    }

    fn with_causes(id: &str, jobs: usize) -> Improvement {
        let mut improving = improvement(&[id]);
        improving.cause_jobs.insert(id.to_string(), jobs);
        improving
    }

    #[test]
    fn repeat_urgent_raises_a_cause_in_three_distinct_jobs_and_not_in_two() {
        let entries = repeat_entries();
        let build = |improving: &Improvement, person: &[String]| {
            Plan::build_improving(&entries, &[], TierCaps::UNLIMITED, person, improving)
        };
        // Three jobs: urgent, so it is ordered first, ahead of the older normal items.
        let (plan, capped) = build(&with_causes("h", 3), &[]);
        assert_eq!(plan.order, ["h", "n1", "n2"]);
        assert_eq!(
            plan.promoted,
            [RepeatPromotion {
                id: "h".into(),
                jobs: 3,
                from: Priority::Normal
            }]
        );
        assert_eq!(
            capped.iter().find(|e| e.id == "h").unwrap().priority,
            Priority::Urgent
        );
        // The request's own entry is not changed.
        assert_eq!(entries[2].priority, Priority::Normal);
        // More jobs behave the same.
        assert_eq!(build(&with_causes("h", 7), &[]).0.order[0], "h");
        // Two jobs (the filing threshold, not the urgent one): ordered by age, not promoted.
        let (plan, _) = build(&with_causes("h", 2), &[]);
        assert_eq!(plan.order, ["n1", "n2", "h"]);
        assert!(plan.promoted.is_empty());
        // A cause count for an item that is not an improvement request promotes nothing.
        let mut other = improvement(&["n2"]);
        other.cause_jobs.insert("h".into(), 5);
        let (plan, _) = build(&other, &[]);
        assert!(plan.promoted.is_empty(), "{:?}", plan.promoted);
        // A priority a person set is never changed.
        let (plan, _) = build(&with_causes("h", 3), &["h".to_string()]);
        assert!(plan.promoted.is_empty());
        assert_eq!(plan.order, ["n1", "n2", "h"]);
        // A high request is raised to urgent and says where it came from.
        let mut high = entries.clone();
        high[2].priority = Priority::High;
        let (plan, _) =
            Plan::build_improving(&high, &[], TierCaps::UNLIMITED, &[], &with_causes("h", 3));
        assert_eq!(plan.promoted[0].from, Priority::High);
        // An urgent one stays as it is and is not listed as raised.
        let mut urgent = entries.clone();
        urgent[2].priority = Priority::Urgent;
        let (plan, _) =
            Plan::build_improving(&urgent, &[], TierCaps::UNLIMITED, &[], &with_causes("h", 3));
        assert!(plan.promoted.is_empty());
    }

    #[test]
    fn repeat_urgent_is_demoted_again_when_person_set_requests_fill_the_urgent_tier() {
        let (_, low) = rungs();
        let mut entries = repeat_entries();
        for n in 1..=3 {
            entries.push(with_priority(
                entry(&format!("p{n}"), low, &[&format!("p{n}/")], &[]),
                Priority::Urgent,
            ));
        }
        let person: Vec<String> = (1..=3).map(|n| format!("p{n}")).collect();
        let caps = TierCaps {
            urgent: 3,
            high: 10,
        };
        let (plan, capped) =
            Plan::build_improving(&entries, &[], caps, &person, &with_causes("h", 3));
        assert_eq!(plan.promoted.len(), 1);
        assert_eq!(plan.demoted.len(), 1);
        assert_eq!(plan.demoted[0].id, "h");
        assert_eq!(plan.demoted[0].from, Priority::Urgent);
        assert_eq!(plan.demoted[0].to, Priority::High);
        assert_eq!(
            capped.iter().find(|e| e.id == "h").unwrap().priority,
            Priority::High
        );
        // High still sorts ahead of the older normal items.
        assert!(plan.position("h") < plan.position("n1"));
    }

    #[test]
    fn improvement_turn_skips_a_request_whose_reading_fails_and_takes_a_ready_one() {
        let (_, low) = rungs();
        let mut entries = six_urgent_and_two_steward();
        // s1 has no acceptance: refused, so it is never planned and never takes a turn.
        entries[6].acceptance.clear();
        let (plan, _) = Plan::build_improving(
            &entries,
            &[],
            TierCaps::UNLIMITED,
            &[],
            &improvement(&["s1", "s2"]),
        );
        assert!(plan.refused.iter().any(|r| r.id == "s1"));
        assert!(plan.position("s1").is_none());
        assert_eq!(plan.turns.len(), 1);
        assert_eq!(plan.turns[0].id, "s2");
        assert_eq!(plan.position("s2"), Some(2));
        // A request that is held for a failed dependency is not given a turn either.
        let mut entries = six_urgent_and_two_steward();
        entries[6].depends_on = vec!["x".into()];
        let history = [ended("x", "rejected", None)];
        let (plan, _) = Plan::build_improving(
            &entries,
            &history,
            TierCaps::UNLIMITED,
            &[],
            &improvement(&["s1", "s2"]),
        );
        assert!(plan.refused.iter().any(|r| r.id == "s1"));
        assert_eq!(
            plan.turns.iter().map(|t| t.id.as_str()).collect::<Vec<_>>(),
            ["s2"]
        );
        // Nothing ready: no turn, and the order is the plain one.
        let mut entries = six_urgent_and_two_steward();
        entries[6].acceptance.clear();
        entries[7].acceptance.clear();
        let (plan, _) = Plan::build_improving(
            &entries,
            &[],
            TierCaps::UNLIMITED,
            &[],
            &improvement(&["s1", "s2"]),
        );
        let (plain, _) = Plan::build_capped(&entries, &[], TierCaps::UNLIMITED, &[]);
        assert!(plan.turns.is_empty());
        assert_eq!(plan.order, plain.order);
        let _ = low;
    }

    #[test]
    fn improvement_turn_skips_a_request_whose_dependency_is_not_yet_dispatched() {
        let mut entries = six_urgent_and_two_steward();
        // s1 needs u5, which stands at the fifth place: s1 cannot take a place before it.
        entries[6].depends_on = vec!["u5".into()];
        let (plan, _) = Plan::build_improving(
            &entries,
            &[],
            TierCaps::UNLIMITED,
            &[],
            &improvement(&["s1"]),
        );
        assert_eq!(plan.order, ["u1", "u2", "u3", "u4", "u5", "s1", "u6", "s2"]);
        assert_eq!(plan.turns.len(), 1);
        assert_eq!(plan.turns[0].position, 6);
        for e in &entries {
            for dep in &e.depends_on {
                assert!(
                    plan.position(dep) < plan.position(&e.id),
                    "{dep} before {}",
                    e.id
                );
            }
        }
        // Once the dependency has been dispatched, the request takes the first due place.
        let mut improving = improvement(&["s1"]);
        improving.since_turn = 2;
        improving.tried = ["u5".to_string()].into();
        let (plan, _) = Plan::build_improving(&entries, &[], TierCaps::UNLIMITED, &[], &improving);
        assert_eq!(plan.order[0], "s1");
    }

    #[test]
    fn improvement_turn_skips_a_moved_request_in_the_order_check() {
        let entries = six_urgent_and_two_steward();
        let (plan, capped) = Plan::build_improving(
            &entries,
            &[],
            TierCaps::UNLIMITED,
            &[],
            &improvement(&["s1", "s2"]),
        );
        assert!(plan.priority_violations(&capped).is_empty());
        // The same order without the record of the turns is a contradiction of priority.
        let unexplained = Plan {
            turns: Vec::new(),
            ..plan.clone()
        };
        let found = unexplained.priority_violations(&capped);
        assert!(
            found.iter().any(|m| m.starts_with("u3 (urgent)")),
            "{found:?}"
        );
        // Other contradictions are still reported when a request was moved.
        let mut with_normal = entries.clone();
        with_normal.push(entry("n1", rungs().1, &["n1/"], &[]));
        let (mut broken, capped) = Plan::build_improving(
            &with_normal,
            &[],
            TierCaps::UNLIMITED,
            &[],
            &improvement(&["s1", "s2"]),
        );
        assert!(broken.priority_violations(&capped).is_empty());
        broken.order.retain(|id| id != "n1");
        broken.order.insert(0, "n1".into());
        assert!(!broken.priority_violations(&capped).is_empty());
    }
}
