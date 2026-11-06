//! The link from a decision to the code it governs, and the rule that a change to that code must reconcile the
//! decision before verification.
//!
//! `docs/decisions.md` is prose, so nothing tied a decision to the paths it describes: a change that reversed
//! one was found only at the final verdict. `docs/decision-links.toml` now holds a `[[governs]]` table beside
//! its `[[link]]` tables: a decision id and the paths it governs. This module is the one parser of that table
//! and the one rule built on it, shared by the producer-side check (athena-core), the retry reader and the
//! backtest's validation (athena-cli), so they cannot drift.
//!
//! The rule ([`unreconciled`]) is pure: it takes the texts of the list and the log at the base and at the
//! candidate, and the changed paths. A governed decision is reconciled when the candidate adds a `[[link]]`
//! whose `older` is that decision and whose `newer` is an entry the candidate adds to the log. The link's
//! status is not read: `amended` and `superseded` both reconcile, and a one-line entry that checks the
//! decision and leaves it as it was, with an `amended` link, is the cheap route for a change that reverses
//! nothing. The base's list judges, so a change cannot remove its own governs entry.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The start of the closing reason of a job the check closed. The retry reader recognises the reason by it.
pub const GATE_REASON_PREFIX: &str = "decisions not reconciled:";

/// One `[[governs]]` entry: a decision and the paths whose change it governs. A path is an exact file, or a
/// directory prefix when it ends with `/`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Governs {
    pub decision: String,
    pub paths: Vec<String>,
}

impl Governs {
    /// Whether this entry governs `path`.
    pub fn covers(&self, path: &str) -> bool {
        self.paths.iter().any(|p| {
            if p.ends_with('/') {
                path.starts_with(p.as_str())
            } else {
                path == p
            }
        })
    }

    fn validate(&self) -> Result<(), String> {
        let digits = self.decision.strip_prefix('D').unwrap_or("");
        if digits.len() < 2 || !digits.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!(
                "[[governs]] decision {:?} is not a decision id such as D08",
                self.decision
            ));
        }
        if self.paths.is_empty() {
            return Err(format!("[[governs]] {} lists no paths", self.decision));
        }
        for path in &self.paths {
            if path.is_empty() || path.starts_with('/') || path.split('/').any(|part| part == "..")
            {
                return Err(format!(
                    "[[governs]] {}: path {path:?} must be relative to the repository root, without `..`",
                    self.decision
                ));
            }
        }
        Ok(())
    }
}

/// An older and newer entry id of one `[[link]]` table. Other fields are read by the log's own parse.
#[derive(Debug, Deserialize)]
struct Pair {
    older: String,
    newer: String,
}

#[derive(Debug, Default, Deserialize)]
struct File {
    #[serde(default)]
    governs: Vec<Governs>,
    #[serde(default)]
    link: Vec<Pair>,
}

/// The `[[governs]]` entries of the list text. An unknown field, a malformed id, an empty path list and a path
/// that is absolute or climbs out of the repository are errors.
pub fn parse_governs(text: &str) -> Result<Vec<Governs>, String> {
    let file: File = toml::from_str(text).map_err(|e| format!("reading [[governs]]: {e}"))?;
    for entry in &file.governs {
        entry.validate()?;
    }
    Ok(file.governs)
}

/// The `(older, newer)` pairs of the `[[link]]` tables of the list text.
pub fn link_pairs(text: &str) -> Result<BTreeSet<(String, String)>, String> {
    let file: File = toml::from_str(text).map_err(|e| format!("reading [[link]]: {e}"))?;
    Ok(file.link.into_iter().map(|p| (p.older, p.newer)).collect())
}

/// The ids of the `## D<n> · ` headings of the log text.
pub fn headings(log: &str) -> BTreeSet<String> {
    log.lines()
        .filter_map(|line| {
            let id = line.strip_prefix("## ")?.split_whitespace().next()?;
            let digits = id.strip_prefix('D')?;
            (!digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
                .then(|| id.to_string())
        })
        .collect()
}

/// A governed decision the change touches without reconciling it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unreconciled {
    pub decision: String,
    /// The changed paths the decision governs, in the order given.
    pub touched: Vec<String>,
}

/// The governed decisions that `changed` touches and the candidate does not reconcile.
///
/// `Err` only when the base list does not parse, as the base list judges. A candidate list that does not parse
/// counts as adding no link. `changed` holds every path of the change, including a deleted path and both
/// sides of a rename.
pub fn unreconciled(
    base_links: &str,
    cand_links: &str,
    base_log: &str,
    cand_log: &str,
    changed: &[String],
) -> Result<Vec<Unreconciled>, String> {
    let governed = parse_governs(base_links)?;
    let base_pairs = link_pairs(base_links)?;
    let cand_pairs = link_pairs(cand_links).unwrap_or_default();
    let base_headings = headings(base_log);
    let new_headings: BTreeSet<String> = headings(cand_log)
        .into_iter()
        .filter(|h| !base_headings.contains(h))
        .collect();
    let reconciled = |decision: &str| {
        cand_pairs.iter().any(|(older, newer)| {
            older == decision
                && new_headings.contains(newer)
                && !base_pairs.contains(&(older.clone(), newer.clone()))
        })
    };
    let mut out: Vec<Unreconciled> = Vec::new();
    for entry in &governed {
        let touched: Vec<String> = changed
            .iter()
            .filter(|p| entry.covers(p))
            .cloned()
            .collect();
        if touched.is_empty() || reconciled(&entry.decision) {
            continue;
        }
        match out.iter_mut().find(|u| u.decision == entry.decision) {
            Some(existing) => {
                for p in touched {
                    if !existing.touched.contains(&p) {
                        existing.touched.push(p);
                    }
                }
            }
            None => out.push(Unreconciled {
                decision: entry.decision.clone(),
                touched,
            }),
        }
    }
    Ok(out)
}

/// A governed decision and the planned paths it governs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Governing {
    pub decision: String,
    /// The planned paths the decision governs, in the order given.
    pub paths: Vec<String>,
}

/// The decisions of the base list that govern at least one of the `planned` paths, in the list's order, each
/// once with the planned paths it governs. It reads the list only: nothing here says whether a change
/// reconciles a decision (see [`unreconciled`]). An empty list text governs nothing; `Err` only when the base
/// list does not parse.
pub fn governing(base_links: &str, planned: &[String]) -> Result<Vec<Governing>, String> {
    let mut out: Vec<Governing> = Vec::new();
    for entry in &parse_governs(base_links)? {
        let paths: Vec<String> = planned
            .iter()
            .filter(|p| entry.covers(p))
            .cloned()
            .collect();
        if paths.is_empty() {
            continue;
        }
        match out.iter_mut().find(|g| g.decision == entry.decision) {
            Some(existing) => {
                for p in paths {
                    if !existing.paths.contains(&p) {
                        existing.paths.push(p);
                    }
                }
            }
            None => out.push(Governing {
                decision: entry.decision.clone(),
                paths,
            }),
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_LOG: &str =
        "# log\n\n## D02 · a\n\n- text\n\n## D08 · b\n\n- text\n\n## D12 · c\n\n- text\n";

    fn log_with(extra: &[&str]) -> String {
        let mut log = BASE_LOG.to_string();
        for id in extra {
            log.push_str(&format!("\n## {id} · new\n\n- Checked; unchanged.\n"));
        }
        log
    }

    const LIST: &str = "\
[[governs]]
decision = \"D08\"
paths = [\"crates/git/snapshot.rs\", \"crates/core/run/\"]

[[governs]]
decision = \"D12\"
paths = [\"src/\"]

[[governs]]
decision = \"D02\"
paths = [\"README.md\"]

[[link]]
line = 3
older = \"D02\"
newer = \"D08\"
status = \"amended\"
";

    fn list_with_link(older: &str, newer: &str, status: &str) -> String {
        format!(
            "{LIST}\n[[link]]\nline = 9\nolder = \"{older}\"\nnewer = \"{newer}\"\nstatus = \"{status}\"\n"
        )
    }

    fn run(cand_links: &str, cand_log: &str, changed: &[&str]) -> Vec<Unreconciled> {
        let changed: Vec<String> = changed.iter().map(|s| s.to_string()).collect();
        unreconciled(LIST, cand_links, BASE_LOG, cand_log, &changed).unwrap()
    }

    fn ids(found: &[Unreconciled]) -> Vec<&str> {
        found.iter().map(|u| u.decision.as_str()).collect()
    }

    #[test]
    fn an_untouched_governed_path_needs_nothing() {
        assert!(run(LIST, BASE_LOG, &["docs/other.md", "Cargo.toml"]).is_empty());
    }

    #[test]
    fn a_touched_governed_path_with_no_link_is_named_for_d08_d12_and_d02() {
        for (path, id) in [
            ("crates/git/snapshot.rs", "D08"),
            ("src/main.rs", "D12"),
            ("README.md", "D02"),
        ] {
            let found = run(LIST, BASE_LOG, &[path]);
            assert_eq!(ids(&found), [id], "{path}");
            assert_eq!(found[0].touched, [path]);
        }
    }

    #[test]
    fn a_new_link_to_a_new_entry_reconciles_for_d08_d12_and_d02() {
        for (path, id) in [
            ("crates/git/snapshot.rs", "D08"),
            ("src/main.rs", "D12"),
            ("README.md", "D02"),
        ] {
            for status in ["amended", "superseded"] {
                let found = run(
                    &list_with_link(id, "D13", status),
                    &log_with(&["D13"]),
                    &[path],
                );
                assert!(found.is_empty(), "{id} {status}: {found:?}");
            }
        }
    }

    #[test]
    fn a_link_for_another_id_does_not_reconcile() {
        let found = run(
            &list_with_link("D12", "D13", "amended"),
            &log_with(&["D13"]),
            &["README.md"],
        );
        assert_eq!(ids(&found), ["D02"]);
    }

    #[test]
    fn a_link_to_an_entry_the_log_already_had_does_not_reconcile() {
        // D08 is not new in the candidate's log, so a link to it records nothing about this change.
        let found = run(
            &list_with_link("D02", "D12", "amended"),
            BASE_LOG,
            &["README.md"],
        );
        assert_eq!(ids(&found), ["D02"]);
        // A new link whose entry is not in the log at all does not reconcile either.
        let found = run(
            &list_with_link("D02", "D13", "amended"),
            BASE_LOG,
            &["README.md"],
        );
        assert_eq!(ids(&found), ["D02"]);
    }

    #[test]
    fn a_new_entry_with_no_link_does_not_reconcile() {
        let found = run(LIST, &log_with(&["D13"]), &["README.md"]);
        assert_eq!(ids(&found), ["D02"]);
    }

    #[test]
    fn a_link_the_base_already_had_does_not_reconcile() {
        // The base list already links D02 to D13. The candidate keeps that pair and its log gains D13, but the
        // pair is not new, so it records nothing about this change.
        let base = list_with_link("D02", "D13", "amended");
        let changed = vec!["README.md".to_string()];
        let found = unreconciled(&base, &base, BASE_LOG, &log_with(&["D13"]), &changed).unwrap();
        assert_eq!(ids(&found), ["D02"]);
    }

    #[test]
    fn a_directory_prefix_covers_its_files_and_not_a_sibling() {
        assert_eq!(
            ids(&run(LIST, BASE_LOG, &["crates/core/run/mod.rs"])),
            ["D08"]
        );
        assert_eq!(
            ids(&run(LIST, BASE_LOG, &["crates/core/run/a/b.rs"])),
            ["D08"]
        );
        assert!(run(LIST, BASE_LOG, &["crates/core/runtime.rs"]).is_empty());
        assert!(run(LIST, BASE_LOG, &["crates/core/run"]).is_empty());
        assert!(run(LIST, BASE_LOG, &["crates/git/snapshot.rs.bak"]).is_empty());
        assert!(run(LIST, BASE_LOG, &["srcs/main.rs"]).is_empty());
    }

    #[test]
    fn a_deleted_path_and_both_sides_of_a_rename_count_as_touched() {
        // The change lists a deleted file as a path, and a rename as the old path and the new path.
        let renamed_away = run(LIST, BASE_LOG, &["src/old.rs", "lib/new.rs"]);
        assert_eq!(ids(&renamed_away), ["D12"]);
        assert_eq!(renamed_away[0].touched, ["src/old.rs"]);
        let renamed_in = run(LIST, BASE_LOG, &["lib/old.rs", "src/new.rs"]);
        assert_eq!(renamed_in[0].touched, ["src/new.rs"]);
    }

    #[test]
    fn one_decision_with_several_touched_paths_is_listed_once() {
        let found = run(
            LIST,
            BASE_LOG,
            &[
                "crates/git/snapshot.rs",
                "crates/core/run/mod.rs",
                "src/a.rs",
            ],
        );
        assert_eq!(ids(&found), ["D08", "D12"]);
        assert_eq!(
            found[0].touched,
            ["crates/git/snapshot.rs", "crates/core/run/mod.rs"]
        );
    }

    #[test]
    fn the_base_list_judges_so_a_change_cannot_drop_its_own_entry() {
        let without =
            "[[link]]\nline = 3\nolder = \"D02\"\nnewer = \"D08\"\nstatus = \"amended\"\n";
        let changed = vec!["README.md".to_string()];
        let found = unreconciled(LIST, without, BASE_LOG, BASE_LOG, &changed).unwrap();
        assert_eq!(ids(&found), ["D02"]);
    }

    #[test]
    fn a_candidate_list_that_does_not_parse_adds_no_link() {
        let found = run("this is = not [ toml", &log_with(&["D13"]), &["README.md"]);
        assert_eq!(ids(&found), ["D02"]);
    }

    #[test]
    fn a_base_list_that_does_not_parse_is_an_error_and_an_empty_one_governs_nothing() {
        let changed = vec!["README.md".to_string()];
        assert!(unreconciled("not = [toml", "", "", "", &changed).is_err());
        assert!(unreconciled("", "", "", "", &changed).unwrap().is_empty());
    }

    #[test]
    fn governs_entries_are_validated() {
        assert_eq!(parse_governs(LIST).unwrap().len(), 3);
        for bad in [
            "[[governs]]\ndecision = \"8\"\npaths = [\"a\"]\n",
            "[[governs]]\ndecision = \"D8\"\npaths = [\"a\"]\n",
            "[[governs]]\ndecision = \"D08\"\npaths = []\n",
            "[[governs]]\ndecision = \"D08\"\npaths = [\"\"]\n",
            "[[governs]]\ndecision = \"D08\"\npaths = [\"/etc/passwd\"]\n",
            "[[governs]]\ndecision = \"D08\"\npaths = [\"a/../b\"]\n",
            "[[governs]]\ndecision = \"D08\"\npaths = [\"a\"]\nextra = 1\n",
            "[[governs]]\ndecision = \"D08\"\n",
        ] {
            assert!(parse_governs(bad).is_err(), "{bad}");
        }
    }

    fn planned(paths: &[&str]) -> Vec<String> {
        paths.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn governing_names_the_decision_of_a_governed_planned_file() {
        let found = governing(LIST, &planned(&["README.md", "docs/other.md"])).unwrap();
        assert_eq!(
            found,
            [Governing {
                decision: "D02".into(),
                paths: planned(&["README.md"])
            }]
        );
    }

    #[test]
    fn governing_matches_a_directory_prefix_and_not_a_sibling() {
        let found = governing(LIST, &planned(&["crates/core/run/mod.rs"])).unwrap();
        assert_eq!(found[0].decision, "D08");
        assert_eq!(found[0].paths, ["crates/core/run/mod.rs"]);
        assert!(
            governing(LIST, &planned(&["crates/core/runtime.rs"]))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn governing_lists_a_decision_once_with_its_planned_paths_in_order() {
        let found = governing(
            LIST,
            &planned(&[
                "src/b.rs",
                "README.md",
                "crates/git/snapshot.rs",
                "src/a.rs",
            ]),
        )
        .unwrap();
        let ids: Vec<&str> = found.iter().map(|g| g.decision.as_str()).collect();
        // The list's order: D08, D12, D02.
        assert_eq!(ids, ["D08", "D12", "D02"]);
        assert_eq!(found[1].paths, ["src/b.rs", "src/a.rs"]);
    }

    #[test]
    fn governing_is_empty_for_no_match_or_an_empty_list_and_errs_on_a_bad_one() {
        assert!(
            governing(LIST, &planned(&["docs/other.md"]))
                .unwrap()
                .is_empty()
        );
        assert!(governing(LIST, &[]).unwrap().is_empty());
        assert!(governing("", &planned(&["README.md"])).unwrap().is_empty());
        assert!(governing("not = [toml", &planned(&["README.md"])).is_err());
    }

    #[test]
    fn headings_read_only_decision_entries() {
        let log = "# t\n## D01 · a\n### D02 · not a heading\n## Notes\n## D100 · c\n  ## D03 · indented\n";
        let found: Vec<String> = headings(log).into_iter().collect();
        assert_eq!(found, ["D01", "D100"]);
    }
}
