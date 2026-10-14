//! The strategic view, computed: coverage per goal from `docs/design/requirements-map.md`.
//!
//! The map has no goal column. Each row's status comes from the map's tables (a row is a table line whose
//! first cell is an ID; the status is the first words of its last cell: Covered, Partial or Not covered).
//! Which goal a row belongs to comes from the goal sections of `docs/design/strategy-view.md`, whose
//! Covered, Partial and Not covered bullet lists name the row IDs. Only membership is read from those lists;
//! the counts come from the map. An ID listed in a goal but absent from the map, an ID listed twice, and a
//! map row in no goal are errors. Nothing is written.
//!
//! [`drift`] compares the computed counts with the "Coverage at a glance" table of the view; a test runs it
//! over the real files, so the table cannot go stale.

use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;

/// The map file, relative to the repository root.
pub const MAP_PATH: &str = "docs/design/requirements-map.md";
/// The strategy view, relative to the repository root.
pub const VIEW_PATH: &str = "docs/design/strategy-view.md";

/// A row's status in the map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Status {
    Covered,
    Partial,
    NotCovered,
}

/// One goal's counts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GoalCount {
    /// The goal's heading, such as `G1 Run the work through Athena, restartably`.
    pub goal: String,
    pub rows: usize,
    pub covered: usize,
    pub partial: usize,
    pub not_covered: usize,
    pub not_covered_ids: Vec<String>,
}

/// The computed coverage.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    pub goals: Vec<GoalCount>,
    pub total: GoalCount,
    /// Mismatches between the map and the goal lists; empty when they agree.
    pub errors: Vec<String>,
}

fn is_id(cell: &str) -> bool {
    let mut chars = cell.chars();
    chars.next().is_some_and(|c| c.is_ascii_uppercase())
        && cell.contains('-')
        && cell.ends_with(|c: char| c.is_ascii_digit())
        && cell.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// The status rows of the map, in order. A row with an unreadable status or a repeated ID is an error.
pub fn parse_map(text: &str) -> Result<Vec<(String, Status)>> {
    let mut rows: Vec<(String, Status)> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        let Some(inner) = line.strip_prefix('|') else {
            continue;
        };
        let first = inner.split('|').next().unwrap_or("").trim();
        if !is_id(first) {
            continue;
        }
        let last = inner
            .trim_end()
            .trim_end_matches('|')
            .rsplit('|')
            .next()
            .unwrap_or("")
            .trim();
        let status = if last.starts_with("Not covered") {
            Status::NotCovered
        } else if last.starts_with("Partial") {
            Status::Partial
        } else if last.starts_with("Covered") {
            Status::Covered
        } else {
            bail!(
                "map row {first}: the Status cell {last:?} does not start with Covered, Partial or Not covered"
            );
        };
        if rows.iter().any(|(id, _)| id == first) {
            bail!("map row {first} appears more than once");
        }
        rows.push((first.to_string(), status));
    }
    Ok(rows)
}

/// The goals of the view in order, each with the row IDs of its three lists.
pub fn parse_goals(view: &str) -> Vec<(String, Vec<String>)> {
    let mut goals: Vec<(String, Vec<String>)> = Vec::new();
    let mut in_list = false;
    for line in view.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            in_list = false;
            let is_goal = heading
                .strip_prefix('G')
                .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_digit()));
            if is_goal {
                goals.push((heading.trim().to_string(), Vec::new()));
            }
            continue;
        }
        let Some((_, ids)) = goals.last_mut() else {
            continue;
        };
        let body = if let Some(item) = line.strip_prefix("- **") {
            in_list = ["Covered:**", "Partial:**", "Not covered:**"]
                .iter()
                .any(|label| item.starts_with(label));
            item
        } else if in_list && line.starts_with(' ') {
            line
        } else {
            in_list = false;
            continue;
        };
        if in_list {
            ids.extend(
                body.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
                    .filter(|w| is_id(w))
                    .map(str::to_string),
            );
        }
    }
    goals
}

fn tally(goal: String, rows: &[(&str, Status)]) -> GoalCount {
    let count = |s: Status| rows.iter().filter(|(_, st)| *st == s).count();
    GoalCount {
        goal,
        rows: rows.len(),
        covered: count(Status::Covered),
        partial: count(Status::Partial),
        not_covered: count(Status::NotCovered),
        not_covered_ids: rows
            .iter()
            .filter(|(_, st)| *st == Status::NotCovered)
            .map(|(id, _)| id.to_string())
            .collect(),
    }
}

/// Computes the per-goal counts from the map text and the view text.
pub fn compute(map: &str, view: &str) -> Result<Report> {
    let rows = parse_map(map)?;
    let status: BTreeMap<&str, Status> = rows.iter().map(|(id, s)| (id.as_str(), *s)).collect();
    let mut errors = Vec::new();
    let mut seen: BTreeMap<&str, String> = BTreeMap::new();
    let goal_lists = parse_goals(view);
    if goal_lists.is_empty() {
        bail!("{VIEW_PATH} has no goal sections (headings that start with G and a number)");
    }
    let mut goals = Vec::new();
    for (name, ids) in &goal_lists {
        let mut members: Vec<(&str, Status)> = Vec::new();
        for id in ids {
            let Some((key, st)) = status.get_key_value(id.as_str()) else {
                errors.push(format!(
                    "{id} is listed under goal {name} but is not a row of the map"
                ));
                continue;
            };
            if let Some(other) = seen.insert(key, name.clone()) {
                errors.push(format!("{id} is listed under both {other} and {name}"));
                continue;
            }
            members.push((key, *st));
        }
        goals.push(tally(name.clone(), &members));
    }
    for (id, _) in &rows {
        if !seen.contains_key(id.as_str()) {
            errors.push(format!("map row {id} is in no goal of the view"));
        }
    }
    let all: Vec<(&str, Status)> = rows.iter().map(|(id, s)| (id.as_str(), *s)).collect();
    Ok(Report {
        goals,
        total: tally("Total".to_string(), &all),
        errors,
    })
}

/// Reads the map and the view under `root` and computes the report.
pub fn run(root: &Path) -> Result<Report> {
    let read = |rel: &str| {
        std::fs::read_to_string(root.join(rel)).with_context(|| format!("reading {rel}"))
    };
    compute(&read(MAP_PATH)?, &read(VIEW_PATH)?)
}

/// The text of the command's output.
pub fn render(report: &Report) -> String {
    let mut out = String::new();
    for g in report.goals.iter().chain(std::iter::once(&report.total)) {
        out.push_str(&format!(
            "{}\n  rows {}, covered {}, partial {}, not covered {}\n",
            g.goal, g.rows, g.covered, g.partial, g.not_covered
        ));
        if g.goal != "Total" {
            let list = if g.not_covered_ids.is_empty() {
                "none".to_string()
            } else {
                g.not_covered_ids.join(", ")
            };
            out.push_str(&format!("  not covered: {list}\n"));
        }
    }
    for e in &report.errors {
        out.push_str(&format!("error: {e}\n"));
    }
    out
}

/// The "Coverage at a glance" table of the view: for each line, the first cell (without emphasis marks) and
/// the four numbers.
fn parse_glance(view: &str) -> Result<Vec<(String, [usize; 4])>> {
    let mut rows = Vec::new();
    let mut inside = false;
    for line in view.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            inside = h.trim() == "Coverage at a glance";
            continue;
        }
        if !inside || !line.starts_with('|') {
            continue;
        }
        let cells: Vec<String> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim().replace('*', ""))
            .collect();
        let nums: Vec<usize> = cells
            .iter()
            .skip(1)
            .filter_map(|c| c.parse().ok())
            .collect();
        if cells.len() == 5 && nums.len() == 4 {
            rows.push((cells[0].clone(), [nums[0], nums[1], nums[2], nums[3]]));
        }
    }
    if rows.is_empty() {
        bail!("{VIEW_PATH} has no 'Coverage at a glance' table with numbers");
    }
    Ok(rows)
}

/// The differences between the view's "Coverage at a glance" table and the computed report, one message per
/// goal (or the total) that differs; empty when they agree.
pub fn drift(report: &Report, view: &str) -> Result<Vec<String>> {
    let table = parse_glance(view)?;
    let mut messages = Vec::new();
    for g in report.goals.iter().chain(std::iter::once(&report.total)) {
        let key = g.goal.split_whitespace().next().unwrap_or("");
        let found = table.iter().find(|(label, _)| {
            if g.goal == "Total" {
                label == "Total"
            } else {
                label.split_whitespace().next() == Some(key)
            }
        });
        let computed = [g.rows, g.covered, g.partial, g.not_covered];
        match found {
            None => messages.push(format!("{}: missing from the table", g.goal)),
            Some((_, shown)) if *shown != computed => messages.push(format!(
                "{}: table shows rows/covered/partial/not covered {:?}, the map gives {:?}",
                g.goal, shown, computed
            )),
            Some(_) => {}
        }
    }
    for (label, _) in &table {
        let known = label == "Total"
            || report
                .goals
                .iter()
                .any(|g| g.goal.split_whitespace().next() == label.split_whitespace().next());
        if !known {
            messages.push(format!("{label}: the table has a goal the sections lack"));
        }
    }
    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAP: &str = "\
| ID | Requirement | BRIEF | Evidence | Status |
| --- | --- | --- | --- | --- |
| X-1 | a | b | c | Covered |
| X-2 | a | b | `x | y` | Partial: some |
| Y-1 | a | b | none | Not covered |
| Y-2 | a | b | none | Not covered: needs a lab |
";

    fn view(table: &str) -> String {
        format!(
            "# View\n\n## Coverage at a glance\n\n| Goal | Rows | Covered | Partial | Not covered |\n| --- | --- | --- | --- | --- |\n{table}\n\n## G1 First goal\n\nProse.\n\n- **Covered:** X-1.\n- **Partial:** X-2.\n- **Not covered:** none.\n\n## G2 Second goal\n\n- **Covered:** none.\n- **Partial:** none.\n- **Not covered:** Y-1,\n  Y-2.\n- **Bet:** Z-9 is not a list.\n"
        )
    }

    const GOOD: &str = "| G1 First goal | 2 | 1 | 1 | 0 |\n| G2 Second goal | 2 | 0 | 0 | 2 |\n| **Total** | **4** | **1** | **1** | **2** |";

    #[test]
    fn parses_statuses_including_trailing_text() {
        let rows = parse_map(MAP).unwrap();
        let statuses: Vec<Status> = rows.iter().map(|(_, s)| *s).collect();
        assert_eq!(
            statuses,
            [
                Status::Covered,
                Status::Partial,
                Status::NotCovered,
                Status::NotCovered
            ]
        );
    }

    #[test]
    fn computes_counts_and_not_covered_lists() {
        let r = compute(MAP, &view(GOOD)).unwrap();
        assert!(r.errors.is_empty(), "{:?}", r.errors);
        assert_eq!(r.goals[1].not_covered_ids, ["Y-1", "Y-2"]);
        assert_eq!(r.goals[0].not_covered_ids, Vec::<String>::new());
        assert_eq!(r.total.rows, 4);
        let text = render(&r);
        assert!(text.contains("not covered: none"));
        assert!(text.contains("not covered: Y-1, Y-2"));
    }

    #[test]
    fn agreeing_table_has_no_drift() {
        let v = view(GOOD);
        let r = compute(MAP, &v).unwrap();
        assert_eq!(drift(&r, &v).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn a_changed_count_is_drift() {
        let v = view(&GOOD.replace(
            "| G2 Second goal | 2 | 0 | 0 | 2 |",
            "| G2 Second goal | 2 | 0 | 1 | 1 |",
        ));
        let r = compute(MAP, &v).unwrap();
        let messages = drift(&r, &v).unwrap();
        assert_eq!(messages.len(), 1, "{messages:?}");
        assert!(messages[0].contains("G2 Second goal") && messages[0].contains("[2, 0, 1, 1]"));
    }

    #[test]
    fn a_changed_total_is_drift() {
        let v = view(&GOOD.replace("**4**", "**5**"));
        let r = compute(MAP, &v).unwrap();
        let messages = drift(&r, &v).unwrap();
        assert!(
            messages.iter().any(|m| m.starts_with("Total")),
            "{messages:?}"
        );
    }

    #[test]
    fn a_flipped_status_is_drift() {
        let v = view(GOOD);
        let flipped = MAP.replace("| Covered |", "| Partial |");
        let r = compute(&flipped, &v).unwrap();
        assert!(!drift(&r, &v).unwrap().is_empty());
    }

    #[test]
    fn unknown_unlisted_and_repeated_ids_are_errors() {
        let v = view(GOOD).replace("X-2.", "X-2, Q-1.");
        let r = compute(MAP, &v).unwrap();
        assert!(r.errors.iter().any(|e| e.contains("Q-1")));
        let v = view(GOOD).replace("**Partial:** X-2.", "**Partial:** none.");
        let r = compute(MAP, &v).unwrap();
        assert!(
            r.errors
                .iter()
                .any(|e| e.contains("X-2") && e.contains("no goal"))
        );
        let v = view(GOOD).replace("**Partial:** X-2.", "**Partial:** X-2, Y-1.");
        let r = compute(MAP, &v).unwrap();
        assert!(r.errors.iter().any(|e| e.contains("both")));
    }

    #[test]
    fn a_bad_status_is_an_error() {
        assert!(parse_map("| X-1 | a | b | c | Maybe |").is_err());
    }

    /// The real view's table must equal what the command computes from the real map.
    #[test]
    fn strategy_view_table_matches_the_map() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = run(&root).unwrap();
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        let view = std::fs::read_to_string(root.join(VIEW_PATH)).unwrap();
        let messages = drift(&report, &view).unwrap();
        assert!(
            messages.is_empty(),
            "{VIEW_PATH} is out of date; run `athena strategy`:\n{}",
            messages.join("\n")
        );
    }
}
