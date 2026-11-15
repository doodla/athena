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
//!
//! A `[[link]]` is keyed on the ids of its `older` and `newer` entries, never on a line of the log: it resolves
//! when a supersession phrase of the log names that pair ([`parse_supersessions`], the one parser, which the
//! backtest's condition (a) uses too). [`link_problems`] is the producer-side check of that, run for a change
//! that edits the log; [`citation_notes`] is the advisory list of docs lines that still cite a decision the
//! change amends or supersedes.

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

/// An older and newer entry id of one `[[link]]` table: the key of the link. Other fields (the `status`, and
/// a `line` that older lists still carry and nothing reads) are ignored here.
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

// ---------------------------------------------------------------------------------------------------------
// Supersession phrases of docs/decisions.md (decision-sets.md section 6, step 3). This is the one parser: the
// backtest's condition (a) and the producer-side link check read the log through it.
// ---------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum HitKind {
    /// "Superseded by Dnn", written inside the older entry.
    SupersededBy,
    /// "Supersedes Dnn", written inside the newer entry.
    Supersedes,
    /// "Amends Dnn", written inside the newer entry.
    Amends,
}

/// One phrase with a target id. `older` is the entry that is superseded or amended; `newer` is the one doing it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hit {
    pub line: usize,
    pub entry: String,
    pub kind: HitKind,
    pub older: String,
    pub newer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Parsed {
    pub entries: Vec<String>,
    pub hits: Vec<Hit>,
    /// Lines where a supersession phrase stands as a statement but names no entry.
    pub lacking_target: Vec<usize>,
}

const PHRASES: [(&str, HitKind); 3] = [
    ("superseded by", HitKind::SupersededBy),
    ("supersedes", HitKind::Supersedes),
    ("amends", HitKind::Amends),
];

/// An entry id (`D` and at least two digits) at the start of `text`, and the rest after it.
fn take_id(text: &str) -> Option<(String, &str)> {
    let rest = text.strip_prefix('D')?;
    let digits = rest.chars().take_while(char::is_ascii_digit).count();
    if digits < 2 {
        return None;
    }
    Some((format!("D{}", &rest[..digits]), &rest[digits..]))
}

/// The ids named after a phrase. A phrase names one id, except "Amends", which may list several joined by
/// commas or "and" ("Amends D06 and D14"). The list stops at the first id that is followed by anything else.
fn target_ids(after: &str, list: bool) -> Vec<String> {
    let mut ids = Vec::new();
    let mut rest = after.trim_start();
    while let Some((id, tail)) = take_id(rest) {
        ids.push(id);
        if !list {
            break;
        }
        let next = [", and ", " and ", ", "]
            .iter()
            .find_map(|sep| tail.strip_prefix(sep));
        match next {
            Some(next) => rest = next,
            None => break,
        }
    }
    ids
}

/// A phrase that opens a statement: after `**`, `(`, "this", or at the start of the text. Descriptive prose
/// ("each supersedes or amends an accepted entry") is not a statement and is not counted as lacking a target.
fn opens_statement(before: &str) -> bool {
    let before = before.trim_end();
    before.is_empty() || before.ends_with("**") || before.ends_with('(') || before.ends_with("this")
}

fn entry_heading(line: &str) -> Option<String> {
    let rest = line.strip_prefix("## ")?;
    take_id(rest).map(|(id, _)| id)
}

/// Parse the phrases of the log. The text before the first entry heading is the log's own header and is not
/// read: it describes the rule and names no entry. Matching is case-insensitive (D79 writes "This supersedes").
pub fn parse_supersessions(text: &str) -> Parsed {
    let mut parsed = Parsed {
        entries: Vec::new(),
        hits: Vec::new(),
        lacking_target: Vec::new(),
    };
    let mut current: Option<String> = None;
    for (index, line) in text.lines().enumerate() {
        if let Some(id) = entry_heading(line) {
            parsed.entries.push(id.clone());
            current = Some(id);
            continue;
        }
        let Some(entry) = &current else { continue };
        let lower = line.to_ascii_lowercase();
        for (phrase, kind) in PHRASES {
            for (at, _) in lower.match_indices(phrase) {
                if lower[..at]
                    .chars()
                    .next_back()
                    .is_some_and(char::is_alphabetic)
                {
                    continue;
                }
                let ids = target_ids(&line[at + phrase.len()..], kind == HitKind::Amends);
                if ids.is_empty() {
                    if opens_statement(&lower[..at]) {
                        parsed.lacking_target.push(index + 1);
                    }
                    continue;
                }
                for id in ids {
                    let (older, newer) = match kind {
                        HitKind::SupersededBy => (entry.clone(), id),
                        HitKind::Supersedes | HitKind::Amends => (id, entry.clone()),
                    };
                    parsed.hits.push(Hit {
                        line: index + 1,
                        entry: entry.clone(),
                        kind,
                        older,
                        newer,
                    });
                }
            }
        }
    }
    parsed
}

// ---------------------------------------------------------------------------------------------------------
// The producer-side check of the links, for a change that edits the log.
// ---------------------------------------------------------------------------------------------------------

/// One way a change to `docs/decisions.md` and `docs/decision-links.toml` leaves a link unresolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinkProblem {
    /// A `[[link]]` whose pair no phrase of the candidate log names.
    NotNamed { older: String, newer: String },
    /// A phrase of the candidate log, new in this change, whose pair has no `[[link]]`.
    NoLink {
        older: String,
        newer: String,
        kind: HitKind,
    },
    /// A statement-opening phrase with no entry id, on a line the base log does not have.
    NoTarget { line: usize, text: String },
    /// The candidate list does not parse.
    ListUnreadable(String),
}

type PairSet = BTreeSet<(String, String)>;

fn named_pairs(parsed: &Parsed) -> PairSet {
    parsed
        .hits
        .iter()
        .map(|h| (h.older.clone(), h.newer.clone()))
        .collect()
}

/// The trimmed texts of the lines of `log` that [`parse_supersessions`] reports as phrases with no target.
fn no_target_lines(log: &str, parsed: &Parsed) -> Vec<(usize, String)> {
    let lines: Vec<&str> = log.lines().collect();
    parsed
        .lacking_target
        .iter()
        .map(|n| {
            (
                *n,
                lines.get(n - 1).copied().unwrap_or("").trim().to_string(),
            )
        })
        .collect()
}

/// What a change to the log and the list leaves unresolved, counting only what the change introduces.
///
/// A link resolves when a phrase of the log names its `(older, newer)` pair, wherever in the log the phrase
/// stands, so lines inserted above a phrase break nothing. The findings are the three halves of the backtest's
/// condition (a), run before the verifier:
///
/// * [`LinkProblem::NotNamed`]: a candidate link no phrase of the candidate log names, when the link is new or
///   the base log named it (the change removed the phrase);
/// * [`LinkProblem::NoLink`]: a pair the candidate log names that the candidate list lacks, when the base log
///   did not name the pair or the base list had it (the change removed the link);
/// * [`LinkProblem::NoTarget`]: a statement-opening phrase with no id on a line the base log does not have;
/// * [`LinkProblem::ListUnreadable`]: the candidate list does not parse.
///
/// A fault already present at the base is not blamed, so a producer is never held for what it did not cause. A
/// base list that does not parse counts as empty: the caller checked it first.
pub fn link_problems(
    base_links: &str,
    cand_links: &str,
    base_log: &str,
    cand_log: &str,
) -> Vec<LinkProblem> {
    let cand_list = match link_pairs(cand_links) {
        Ok(pairs) => pairs,
        Err(why) => return vec![LinkProblem::ListUnreadable(why)],
    };
    let base_list = link_pairs(base_links).unwrap_or_default();
    let base = parse_supersessions(base_log);
    let cand = parse_supersessions(cand_log);
    let base_named = named_pairs(&base);
    let cand_named = named_pairs(&cand);
    let mut out = Vec::new();
    for pair in &cand_list {
        if !cand_named.contains(pair) && (!base_list.contains(pair) || base_named.contains(pair)) {
            out.push(LinkProblem::NotNamed {
                older: pair.0.clone(),
                newer: pair.1.clone(),
            });
        }
    }
    let mut seen = PairSet::new();
    for hit in &cand.hits {
        let pair = (hit.older.clone(), hit.newer.clone());
        if cand_list.contains(&pair)
            || (base_named.contains(&pair) && !base_list.contains(&pair))
            || !seen.insert(pair)
        {
            continue;
        }
        out.push(LinkProblem::NoLink {
            older: hit.older.clone(),
            newer: hit.newer.clone(),
            kind: hit.kind,
        });
    }
    let base_lacking: BTreeSet<String> = no_target_lines(base_log, &base)
        .into_iter()
        .map(|(_, text)| text)
        .collect();
    for (line, text) in no_target_lines(cand_log, &cand) {
        if !base_lacking.contains(&text) {
            out.push(LinkProblem::NoTarget { line, text });
        }
    }
    out
}

/// The phrases the candidate log adds: one hit for each `(older, newer)` pair the base log does not name, the
/// first such hit of the pair, in log order.
pub fn new_hits(base_log: &str, cand_log: &str) -> Vec<Hit> {
    let base_named = named_pairs(&parse_supersessions(base_log));
    let mut seen = PairSet::new();
    parse_supersessions(cand_log)
        .hits
        .into_iter()
        .filter(|h| {
            let pair = (h.older.clone(), h.newer.clone());
            !base_named.contains(&pair) && seen.insert(pair)
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------------------
// The advisory list of docs lines that cite an amended or superseded decision.
// ---------------------------------------------------------------------------------------------------------

/// A line of a docs file that cites a decision by its id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    pub decision: String,
    pub path: String,
    /// The 1-based line of the citation.
    pub line: usize,
    /// The trimmed text of the line.
    pub text: String,
}

/// The most lines listed for a superseded decision, and for an amended one.
const SUPERSEDED_CAP: usize = 40;
const AMENDED_CAP: usize = 5;
/// The longest line text a note quotes, in characters.
const NOTE_TEXT_CHARS: usize = 120;

/// Whether `line` holds `id` as a whole token: the character before is not alphanumeric and the character after
/// is not an ASCII letter or digit, so `D08`, `D08's` and `(D08)` match and `D080`, `AD08` and `D08x` do not.
fn cites(line: &str, id: &str) -> bool {
    line.match_indices(id).any(|(at, _)| {
        let before = line[..at].chars().next_back();
        let after = line[at + id.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric)
            && !after.is_some_and(|c| c.is_ascii_alphanumeric())
    })
}

/// The lines of `docs` (path and text, in the order given) that cite any of `decisions` as a whole token. Only
/// `.md` paths are read, and `docs/decisions.md` is skipped: the log says what each entry decided, and the
/// check is for the prose that restates a decision elsewhere. A line citing two of the decisions is listed once
/// for each.
pub fn citing_lines(decisions: &[String], docs: &[(String, String)]) -> Vec<Citation> {
    let mut out = Vec::new();
    for decision in decisions {
        for (path, text) in docs {
            if !path.ends_with(".md") || path == "docs/decisions.md" {
                continue;
            }
            for (index, line) in text.lines().enumerate() {
                if cites(line, decision) {
                    out.push(Citation {
                        decision: decision.clone(),
                        path: path.clone(),
                        line: index + 1,
                        text: line.trim().to_string(),
                    });
                }
            }
        }
    }
    out
}

/// The advisory notes for the phrases a change adds ([`new_hits`]): for each older decision, the docs lines
/// that cite it. A decision a hit supersedes lists every line (at most 40, then a count); one that is only
/// amended lists the first 5 and a count, since most amendments leave what the other docs say true. Each note
/// reads `<path>:<line> cites <id>, which <newer> amends or supersedes: <text>`. It is a heuristic on ids: it
/// cannot read meaning, and a note is never a finding.
pub fn citation_notes(hits: &[Hit], docs: &[(String, String)]) -> Vec<String> {
    let mut older: Vec<&str> = Vec::new();
    for hit in hits {
        if !older.contains(&hit.older.as_str()) {
            older.push(&hit.older);
        }
    }
    let mut notes = Vec::new();
    for id in older {
        let of_id: Vec<&Hit> = hits.iter().filter(|h| h.older == id).collect();
        let newer = of_id[0].newer.as_str();
        let superseded = of_id.iter().any(|h| h.kind != HitKind::Amends);
        let cap = if superseded {
            SUPERSEDED_CAP
        } else {
            AMENDED_CAP
        };
        let cited = citing_lines(&[id.to_string()], docs);
        for c in cited.iter().take(cap) {
            let text: String = c.text.chars().take(NOTE_TEXT_CHARS).collect();
            notes.push(format!(
                "{}:{} cites {id}, which {newer} amends or supersedes: {text}",
                c.path, c.line
            ));
        }
        if cited.len() > cap {
            notes.push(format!(
                "and {} more docs lines cite {id}, which {newer} amends or supersedes",
                cited.len() - cap
            ));
        }
    }
    notes
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
older = \"D02\"
newer = \"D08\"
status = \"amended\"
";

    fn list_with_link(older: &str, newer: &str, status: &str) -> String {
        format!(
            "{LIST}\n[[link]]\nolder = \"{older}\"\nnewer = \"{newer}\"\nstatus = \"{status}\"\n"
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
        let without = "[[link]]\nolder = \"D02\"\nnewer = \"D08\"\nstatus = \"amended\"\n";
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

    // -- the phrase parse ----------------------------------------------------------------------------------

    #[test]
    fn take_id_needs_two_digits() {
        assert_eq!(take_id("D88**"), Some(("D88".to_string(), "**")));
        assert_eq!(take_id("D8 x"), None);
        assert_eq!(take_id("Dx"), None);
    }

    #[test]
    fn amends_lists_split_on_and_and_commas() {
        assert_eq!(target_ids(" D06 and D14. The gate", true), ["D06", "D14"]);
        assert_eq!(target_ids(" D39's exemption list and D06.", true), ["D39"]);
        assert_eq!(target_ids(" D87's", false), ["D87"]);
    }

    #[test]
    fn descriptive_prose_is_not_a_missing_target() {
        let text = "## D01 · x\n\nEach supersedes or amends an accepted entry\n(active, superseded by a named decision)\n";
        let parsed = parse_supersessions(text);
        assert!(parsed.hits.is_empty());
        assert!(parsed.lacking_target.is_empty());
    }

    #[test]
    fn a_statement_without_an_id_is_reported_with_its_line() {
        let text = "## D01 · x\n- **Superseded by** nothing\n";
        assert_eq!(parse_supersessions(text).lacking_target, [2]);
    }

    // -- link_problems -------------------------------------------------------------------------------------

    /// A log in which D08 amends D02, the pair the list's one link names.
    const PHRASE_LOG: &str = "# log\n\n## D02 · a\n\n- text\n\n## D08 · b\n\n- Amends D02 in prose only.\n\n## D12 · c\n\n- text\n";

    fn with_link(list: &str, older: &str, newer: &str) -> String {
        format!(
            "{list}\n[[link]]\nolder = \"{older}\"\nnewer = \"{newer}\"\nstatus = \"amended\"\n"
        )
    }

    fn append(log: &str, entry: &str) -> String {
        format!("{log}\n{entry}\n")
    }

    fn not_named(older: &str, newer: &str) -> LinkProblem {
        LinkProblem::NotNamed {
            older: older.into(),
            newer: newer.into(),
        }
    }

    #[test]
    fn a_link_that_no_phrase_names_is_not_named() {
        let cand = with_link(LIST, "D02", "D99");
        let found = link_problems(LIST, &cand, PHRASE_LOG, PHRASE_LOG);
        assert_eq!(found, [not_named("D02", "D99")]);
    }

    #[test]
    fn a_log_shifted_by_added_lines_still_resolves_every_link() {
        let shifted =
            PHRASE_LOG.replacen("# log\n", &format!("# log\n{}", "added\n".repeat(10)), 1);
        assert_eq!(shifted.lines().count(), PHRASE_LOG.lines().count() + 10);
        assert!(link_problems(LIST, LIST, PHRASE_LOG, &shifted).is_empty());
        // The phrase moved by 10 lines and nothing about the list says where it is.
        let before = parse_supersessions(PHRASE_LOG).hits[0].line;
        let after = parse_supersessions(&shifted).hits[0].line;
        assert_eq!(after, before + 10);
    }

    #[test]
    fn a_fault_already_at_the_base_is_not_blamed_on_the_change() {
        // D12 to D99 is listed and no phrase names it, at the base and still at the candidate.
        let broken = with_link(LIST, "D12", "D99");
        assert!(link_problems(&broken, &broken, PHRASE_LOG, PHRASE_LOG).is_empty());
        // A log entry the base already had with a phrase and no link is not blamed either.
        let unlinked = "[[governs]]\ndecision = \"D12\"\npaths = [\"src/\"]\n";
        assert!(link_problems(unlinked, unlinked, PHRASE_LOG, PHRASE_LOG).is_empty());
        // A new fault beside it is.
        let found = link_problems(
            &broken,
            &with_link(&broken, "D02", "D98"),
            PHRASE_LOG,
            PHRASE_LOG,
        );
        assert_eq!(found, [not_named("D02", "D98")]);
    }

    #[test]
    fn a_phrase_the_change_removes_leaves_its_link_not_named() {
        let cand_log = PHRASE_LOG.replace("Amends D02 in prose only.", "Checked D02.");
        let found = link_problems(LIST, LIST, PHRASE_LOG, &cand_log);
        assert_eq!(found, [not_named("D02", "D08")]);
    }

    #[test]
    fn a_new_phrase_with_no_link_has_no_link() {
        let cand_log = append(PHRASE_LOG, "## D13 · n\n\n- Amends D02 in prose only.");
        let found = link_problems(LIST, LIST, PHRASE_LOG, &cand_log);
        assert_eq!(
            found,
            [LinkProblem::NoLink {
                older: "D02".into(),
                newer: "D13".into(),
                kind: HitKind::Amends
            }]
        );
        // With its link the same change resolves.
        let cand = with_link(LIST, "D02", "D13");
        assert!(link_problems(LIST, &cand, PHRASE_LOG, &cand_log).is_empty());
        // A superseded-by phrase names the pair the other way round and wants the same link.
        let cand_log = append(
            PHRASE_LOG,
            "## D13 · n\n\n- Revised. **Superseded by D14**.",
        );
        let found = link_problems(LIST, LIST, PHRASE_LOG, &cand_log);
        assert_eq!(
            found,
            [LinkProblem::NoLink {
                older: "D13".into(),
                newer: "D14".into(),
                kind: HitKind::SupersededBy
            }]
        );
    }

    #[test]
    fn a_link_the_change_drops_is_reported_when_its_phrase_stays() {
        let found = link_problems(LIST, "", PHRASE_LOG, PHRASE_LOG);
        assert_eq!(
            found,
            [LinkProblem::NoLink {
                older: "D02".into(),
                newer: "D08".into(),
                kind: HitKind::Amends
            }]
        );
    }

    #[test]
    fn a_new_statement_without_an_id_is_reported_and_an_old_one_is_not() {
        let line = "- **Superseded by** nothing yet";
        let cand_log = append(PHRASE_LOG, &format!("## D13 · n\n\n{line}"));
        let found = link_problems(LIST, LIST, PHRASE_LOG, &cand_log);
        assert_eq!(
            found,
            [LinkProblem::NoTarget {
                line: cand_log.lines().position(|l| l == line).unwrap() + 1,
                text: line.into()
            }]
        );
        // The base already had that line: the change is not blamed, wherever the line now stands.
        let base_log = append(PHRASE_LOG, &format!("## D13 · n\n\n{line}"));
        let shifted = base_log.replacen("# log\n", "# log\n\nmore\n", 1);
        assert!(link_problems(LIST, LIST, &base_log, &shifted).is_empty());
    }

    #[test]
    fn a_candidate_list_that_does_not_parse_is_one_finding() {
        let found = link_problems(LIST, "this is = not [ toml", PHRASE_LOG, PHRASE_LOG);
        assert_eq!(found.len(), 1);
        assert!(matches!(&found[0], LinkProblem::ListUnreadable(why) if why.contains("[[link]]")));
    }

    #[test]
    fn a_stray_line_key_in_an_older_list_is_ignored() {
        let old = "[[link]]\nline = 7\nolder = \"D02\"\nnewer = \"D08\"\nstatus = \"amended\"\n";
        assert!(link_problems(old, old, PHRASE_LOG, PHRASE_LOG).is_empty());
    }

    #[test]
    fn new_hits_lists_one_hit_for_each_pair_the_base_does_not_name() {
        let cand_log = append(
            PHRASE_LOG,
            "## D13 · n\n\n- Amends D02 in prose only.\n- Amends D02 again.\n- Amends D12.",
        );
        let found = new_hits(PHRASE_LOG, &cand_log);
        let pairs: Vec<(&str, &str)> = found
            .iter()
            .map(|h| (h.older.as_str(), h.newer.as_str()))
            .collect();
        assert_eq!(pairs, [("D02", "D13"), ("D12", "D13")]);
        assert!(new_hits(PHRASE_LOG, PHRASE_LOG).is_empty());
    }

    // -- citing_lines and citation_notes -------------------------------------------------------------------

    fn docs(files: &[(&str, &str)]) -> Vec<(String, String)> {
        files
            .iter()
            .map(|(p, t)| (p.to_string(), t.to_string()))
            .collect()
    }

    fn lines_of(found: &[Citation]) -> Vec<(String, usize)> {
        found.iter().map(|c| (c.path.clone(), c.line)).collect()
    }

    #[test]
    fn an_id_is_cited_only_as_a_whole_token() {
        let text = "D08 first\nD08's rule\n(D08)\nends with D08.\nD08-based\nD080\nD8\nAD08\nD08x\nx_D08\n";
        let found = citing_lines(&["D08".to_string()], &docs(&[("docs/a.md", text)]));
        // D08, D08's, (D08), D08., D08- and a leading underscore match; D080, D8, AD08 and D08x do not.
        assert_eq!(
            lines_of(&found),
            [1, 2, 3, 4, 5, 10].map(|n| ("docs/a.md".to_string(), n))
        );
        assert_eq!(found[1].text, "D08's rule");
        assert_eq!(found[0].decision, "D08");
        // One of two occurrences on a line is enough.
        let found = citing_lines(
            &["D08".to_string()],
            &docs(&[("docs/a.md", "D080 and D08\n")]),
        );
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn the_log_and_files_that_are_not_markdown_are_not_read() {
        let found = citing_lines(
            &["D08".to_string()],
            &docs(&[
                ("docs/decisions.md", "D08 here\n"),
                ("docs/decision-links.toml", "D08\n"),
                ("src/a.rs", "// D08\n"),
                ("docs/design/b.md", "see D08\n"),
            ]),
        );
        assert_eq!(lines_of(&found), [("docs/design/b.md".to_string(), 1)]);
    }

    fn hit(kind: HitKind, older: &str, newer: &str) -> Hit {
        Hit {
            line: 1,
            entry: if kind == HitKind::SupersededBy {
                older.into()
            } else {
                newer.into()
            },
            kind,
            older: older.into(),
            newer: newer.into(),
        }
    }

    fn eight_lines() -> Vec<(String, String)> {
        docs(&[("docs/a.md", &"D08 says so\n".repeat(8))])
    }

    #[test]
    fn an_amended_decision_lists_five_lines_and_a_count_and_a_superseded_one_lists_all() {
        let amended = citation_notes(&[hit(HitKind::Amends, "D08", "D13")], &eight_lines());
        assert_eq!(amended.len(), 6, "{amended:#?}");
        assert_eq!(
            amended[0],
            "docs/a.md:1 cites D08, which D13 amends or supersedes: D08 says so"
        );
        assert_eq!(
            amended[5],
            "and 3 more docs lines cite D08, which D13 amends or supersedes"
        );
        for kind in [HitKind::Supersedes, HitKind::SupersededBy] {
            let superseded = citation_notes(&[hit(kind, "D08", "D13")], &eight_lines());
            assert_eq!(superseded.len(), 8, "{superseded:#?}");
            assert!(superseded.iter().all(|n| n.starts_with("docs/a.md:")));
        }
    }

    #[test]
    fn a_superseded_decision_is_capped_at_forty_lines_and_a_long_line_is_cut() {
        let many = docs(&[("docs/a.md", &"D08\n".repeat(45))]);
        let notes = citation_notes(&[hit(HitKind::Supersedes, "D08", "D13")], &many);
        assert_eq!(notes.len(), 41);
        assert_eq!(
            notes[40],
            "and 5 more docs lines cite D08, which D13 amends or supersedes"
        );
        let long = docs(&[("docs/a.md", &format!("D08 {}\n", "x".repeat(300)))]);
        let notes = citation_notes(&[hit(HitKind::Amends, "D08", "D13")], &long);
        assert!(
            notes[0].ends_with(&format!("D08 {}", "x".repeat(116))),
            "{}",
            notes[0]
        );
    }

    #[test]
    fn an_older_decision_with_both_kinds_of_hit_is_listed_once_as_superseded() {
        let hits = [
            hit(HitKind::Amends, "D08", "D13"),
            hit(HitKind::Supersedes, "D08", "D14"),
        ];
        let notes = citation_notes(&hits, &eight_lines());
        assert_eq!(notes.len(), 8, "{notes:#?}");
        assert!(notes.iter().all(|n| n.contains("which D13")));
        assert!(citation_notes(&[], &eight_lines()).is_empty());
    }
}
