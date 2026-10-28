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
//! - **Parallel groups.** A wave is split into groups of items that no dependency links and whose touched
//!   paths are disjoint; the items of one group could run side by side. An item that names no touched path is
//!   never grouped with another, because nothing is known about what it changes. The shared append-only files
//!   (`SHARED_APPEND_ONLY`: `CHANGELOG.md`, `docs/decisions.md`, `README.md`; `docs/design/parallel-jobs.md`
//!   section 9) are left out of the overlap test, since nearly every item touches them; an item whose only
//!   paths are shared files therefore names nothing else and is not grouped. The landing check stays the
//!   authority for conflicting edits to a shared file; this changes only the grouping. The drain works the groups in
//!   plan order and runs the items of one group at once, up to `[parallel] max_concurrency` (default 1: one
//!   job at a time).
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
    /// Ids of the requests that must finish first.
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

/// The coordinated plan of all open requests.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct Plan {
    /// Every planned item, in the order to work them.
    pub order: Vec<String>,
    /// The same items in groups: each group's items could run in parallel, and each group comes after the
    /// groups holding what its items depend on.
    pub groups: Vec<Vec<String>>,
    /// Items whose plan is incomplete: not in `order`.
    pub refused: Vec<Refusal>,
    /// Items that wait on each other in a loop: not in `order`.
    pub cyclic: Vec<String>,
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
    !pa.is_empty() && !pb.is_empty() && !pa.iter().any(|x| pb.iter().any(|y| overlap(x, y)))
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
            if let Some((_, earlier, earlier_p)) = ranked[..i].iter().find(|(_, _, p)| p > later_p)
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

    /// Plans `entries`, given oldest first (the position is the age used as the tiebreak). A dependency on an
    /// id that is not among the entries counts as done.
    pub fn build(entries: &[PlanEntry]) -> Plan {
        let mut plan = Plan::default();
        let mut pending: Vec<(usize, &PlanEntry)> = Vec::new();
        for (age, entry) in entries.iter().enumerate() {
            let missing = entry.missing();
            if missing.is_empty() {
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
            // Greedy groups: an item joins the first group all of whose members it can run beside.
            let mut groups: Vec<Vec<&PlanEntry>> = Vec::new();
            for (_, entry) in &ready {
                match groups
                    .iter_mut()
                    .find(|g| g.iter().all(|m| disjoint(m, entry)))
                {
                    Some(group) => group.push(entry),
                    None => groups.push(vec![entry]),
                }
            }
            for group in groups {
                let ids: Vec<String> = group.iter().map(|e| e.id.clone()).collect();
                plan.order.extend(ids.iter().cloned());
                plan.groups.push(ids);
            }
            for (_, e) in &ready {
                done.push(e.id.clone());
            }
            pending.retain(|(_, e)| !done.contains(&e.id));
        }
        plan
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
    fn disjoint_independent_items_share_a_group_and_dependents_come_after() {
        let (top, low) = rungs();
        let plan = Plan::build(&[
            entry("base", low, &["crates/core/"], &[]),
            entry("docs", low, &["docs/a.md"], &[]),
            entry("clash", low, &["crates/core/src/x.rs"], &[]),
            entry("after", top, &["crates/cli/"], &["base"]),
        ]);
        let group = |ids: &[&str]| ids.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        // `clash` overlaps `base`, so it is not beside it; `after` needs `base`, so it waits for the next wave.
        assert_eq!(
            plan.groups,
            [
                group(&["base", "docs"]),
                group(&["clash"]),
                group(&["after"])
            ]
        );
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
        assert_ne!(group_of("one"), group_of("two"));
        assert_eq!(group_of("one"), group_of("three"));
        assert_eq!(group_of("three"), group_of("four"));
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
}
