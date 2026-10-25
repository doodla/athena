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
//!   comes after what it needs. A request's priority is [`Priority`]: high, normal (the default) or low,
//!   read from a `Priority: <level>` line in its words. With no priority set every item is normal and the
//!   order is the ladder, then age.
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
//!   ([`PlanEntry::missing`], [`Plan::refused`]), so planning cannot be skipped. Nothing is filled in by
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

/// How soon a request should be worked, ahead of its rung and age. High sorts first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Priority {
    High,
    #[default]
    Normal,
    Low,
}

impl Priority {
    pub fn as_str(self) -> &'static str {
        match self {
            Priority::High => "high",
            Priority::Normal => "normal",
            Priority::Low => "low",
        }
    }

    /// The priority a request's words give: the first line that, trimmed, starts with `Priority:` (any case)
    /// followed by exactly `high`, `normal` or `low` (any case). Anything else, or no such line, is normal.
    pub fn from_text(text: &str) -> Priority {
        for line in text.lines() {
            let line = line.trim();
            let Some(head) = line.get(..9) else { continue };
            if !head.eq_ignore_ascii_case("priority:") {
                continue;
            }
            match line[9..].trim().to_ascii_lowercase().as_str() {
                "high" => return Priority::High,
                "normal" => return Priority::Normal,
                "low" => return Priority::Low,
                _ => {}
            }
        }
        Priority::Normal
    }
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
        }
    }

    /// What a plan must have and this one lacks: `goal`, `ladder rung`, `acceptance`.
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
        let reason = format!(
            "the plan lacks {}; say it in the request (acceptance as bullets, the files it changes) and plan again",
            missing.join(", ")
        );
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

impl Plan {
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
                plan.refused.push(Refusal::lacking(
                    &entry.id,
                    &entry.title,
                    missing.into_iter().map(String::from).collect(),
                ));
            }
        }
        let known: Vec<&str> = pending.iter().map(|(_, e)| e.id.as_str()).collect();
        // Effective priority: an item's own, raised to the best of whatever depends on it (to a fixpoint).
        let mut effective: Vec<Priority> = pending.iter().map(|(_, e)| e.priority).collect();
        loop {
            let mut changed = false;
            for i in 0..pending.len() {
                for dep in &pending[i].1.depends_on {
                    if let Some(j) = known.iter().position(|k| k == dep)
                        && effective[i] < effective[j]
                    {
                        effective[j] = effective[i];
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }
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
    fn priority_is_read_from_a_line_in_the_words() {
        assert_eq!(
            Priority::from_text("Fix it\nPriority: high\n"),
            Priority::High
        );
        assert_eq!(Priority::from_text("x\n  priority: HIGH"), Priority::High);
        assert_eq!(Priority::from_text("Priority: low"), Priority::Low);
        assert_eq!(Priority::from_text("Fix it"), Priority::Normal);
        assert_eq!(Priority::from_text("Priority: urgent"), Priority::Normal);
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
