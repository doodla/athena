//! The typed decision view of `docs/decisions.md` and the user-confirmed list of its supersession and amend
//! hits (`docs/design/decision-sets.md` section 8, step 2; section 6, steps 2-3).
//!
//! The log's prose is never edited. Each entry heading gets one derived [`DecisionView`]: its id, whether it is
//! still `active` or `superseded`, the entry that supersedes it and the entries that amend it. The links come
//! from two sources that must agree:
//!
//! * the parse of the log's phrases ([`crate::backtest::parse_supersessions`]), which lists every hit with its
//!   line; and
//! * `docs/decision-links.toml`, the list the user confirmed hit by hit. A phrase such as "superseded by" does
//!   not say whether the old decision stands in part, so the list, not the wording, decides the status. A
//!   partial revision is `amended`, and the older decision stays `active` (D11 by D39 at line 322; D87 by D88).
//!
//! A parsed hit that is not in the list, or a listed hit the parse does not find, is a finding of the backtest's
//! condition (a); see [`crate::backtest::run_on`]. The view only applies hits that are in both.

use crate::backtest::{Hit, Parsed, Status};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::Path;

/// Where the confirmed list lives, relative to the repository root.
pub const LINKS_PATH: &str = "docs/decision-links.toml";

/// What the user confirmed a hit to mean.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LinkStatus {
    /// Nothing of the older decision still stands.
    Superseded,
    /// The older decision is revised in part (or in prose only) and stays active.
    Amended,
}

/// One confirmed hit: the line of the phrase in `docs/decisions.md`, the older and newer entry ids, and the
/// confirmed meaning.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Link {
    pub line: usize,
    pub older: String,
    pub newer: String,
    pub status: LinkStatus,
}

impl Link {
    fn matches(&self, hit: &Hit) -> bool {
        self.line == hit.line && self.older == hit.older && self.newer == hit.newer
    }
}

/// The confirmed list, as stored in `docs/decision-links.toml` (`[[link]]` tables).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ConfirmedLinks {
    #[serde(default, rename = "link")]
    pub links: Vec<Link>,
}

impl ConfirmedLinks {
    /// Parse the text of the list. A decision superseded by two different entries is an error: `superseded_by`
    /// of the view holds one id.
    pub fn parse(text: &str) -> Result<Self> {
        let list: ConfirmedLinks =
            toml::from_str(text).context("parsing the confirmed link list")?;
        for link in &list.links {
            let rivals: BTreeSet<&str> = list
                .links
                .iter()
                .filter(|l| l.older == link.older && l.status == LinkStatus::Superseded)
                .map(|l| l.newer.as_str())
                .collect();
            if rivals.len() > 1 {
                bail!(
                    "{} is confirmed as superseded by more than one entry: {rivals:?}",
                    link.older
                );
            }
        }
        Ok(list)
    }

    /// Read `docs/decision-links.toml` under `root`.
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join(LINKS_PATH);
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        Self::parse(&text).with_context(|| format!("in {}", path.display()))
    }

    /// The hits the parse found that the list does not contain.
    pub fn unlisted<'a>(&self, hits: &'a [Hit]) -> Vec<&'a Hit> {
        hits.iter()
            .filter(|h| !self.links.iter().any(|l| l.matches(h)))
            .collect()
    }

    /// The listed links the parse did not find.
    pub fn not_found(&self, hits: &[Hit]) -> Vec<&Link> {
        self.links
            .iter()
            .filter(|l| !hits.iter().any(|h| l.matches(h)))
            .collect()
    }
}

/// The derived, read-only view of one entry of the log.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecisionView {
    pub id: String,
    /// `Active` or `Superseded`; never set from a phrase alone.
    pub status: Status,
    pub superseded_by: Option<String>,
    pub amended_by: Vec<String>,
    /// Set when a signal fired: the ids of the records that fired it. This is the state a decision is in beside
    /// its `status` (`athena.decision-state/1`); the log's prose is untouched.
    pub under_review: Option<Vec<String>>,
}

/// Marks the view of `decision` as under review, citing `records`. A second firing adds its records; an id the
/// log has no entry for is ignored.
pub fn mark_under_review(views: &mut [DecisionView], decision: &str, records: &[String]) {
    if let Some(view) = views.iter_mut().find(|v| v.id == decision) {
        let cited = view.under_review.get_or_insert_with(Vec::new);
        for id in records {
            if !cited.contains(id) {
                cited.push(id.clone());
            }
        }
    }
}

/// One view per entry heading, in log order. Only hits that are both parsed and confirmed are applied; the
/// confirmed status decides between `superseded_by` and `amended_by`.
pub fn view(parsed: &Parsed, confirmed: &ConfirmedLinks) -> Vec<DecisionView> {
    let applied: Vec<&Link> = confirmed
        .links
        .iter()
        .filter(|l| parsed.hits.iter().any(|h| l.matches(h)))
        .collect();
    parsed
        .entries
        .iter()
        .map(|id| {
            let superseded_by = applied
                .iter()
                .find(|l| &l.older == id && l.status == LinkStatus::Superseded)
                .map(|l| l.newer.clone());
            let mut amended_by: Vec<String> = applied
                .iter()
                .filter(|l| &l.older == id && l.status == LinkStatus::Amended)
                .map(|l| l.newer.clone())
                .collect();
            amended_by.sort();
            amended_by.dedup();
            DecisionView {
                id: id.clone(),
                status: if superseded_by.is_some() {
                    Status::Superseded
                } else {
                    Status::Active
                },
                superseded_by,
                amended_by,
                under_review: None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtest::parse_supersessions;

    /// Length in bytes of `docs/decisions.md` when the log was last known good (it ended with D121). Entries
    /// are never rewritten, so these bytes must stay exactly as they were; new entries go after them.
    const PINNED_LEN: usize = 230046;
    /// SHA-256 of the first `PINNED_LEN` bytes, computed from the file at that commit (`git show
    /// <commit>:docs/decisions.md`) before D122 was appended. Raise both constants only when a later
    /// entry is appended, to the length and hash of the log up to and including that entry.
    const PINNED_SHA256: &str = "bbb3929b73240f6c45520338447ec5b17083aa2a96a171674ac2999b0b5bdca1";

    /// Check the log's structure: every `## D<n> · ` entry has a Provenance bullet, ids are unique and
    /// ascending, and the first `pinned_len` bytes hash to `pinned_sha256` (existing entries untouched, with
    /// additions only after them).
    fn check_log(text: &str, pinned_len: usize, pinned_sha256: &str) -> Result<(), String> {
        let mut starts: Vec<(u32, usize)> = Vec::new();
        let mut offset = 0;
        for line in text.split_inclusive('\n') {
            if let Some(rest) = line.strip_prefix("## D") {
                let (digits, tail) =
                    rest.split_at(rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(0));
                if !digits.is_empty() && tail.starts_with(" · ") {
                    let id: u32 = digits.parse().map_err(|_| format!("bad id in {line:?}"))?;
                    if let Some(&(prev, _)) = starts.last() {
                        if id == prev {
                            return Err(format!("D{id} appears twice"));
                        }
                        if id < prev {
                            return Err(format!("D{id} comes after D{prev}: ids must ascend"));
                        }
                    }
                    starts.push((id, offset));
                }
            }
            offset += line.len();
        }
        if starts.is_empty() {
            return Err("no decision entries found".into());
        }
        for (i, &(id, from)) in starts.iter().enumerate() {
            let to = starts.get(i + 1).map_or(text.len(), |&(_, s)| s);
            if !text[from..to].contains("- **Provenance:**") {
                return Err(format!("D{id} has no Provenance bullet"));
            }
        }
        let prefix = text
            .as_bytes()
            .get(..pinned_len)
            .ok_or("the log is shorter than the pinned base: existing entries were removed")?;
        if athena_data::sha256_hex(prefix) != pinned_sha256 {
            return Err(format!(
                "the first {pinned_len} bytes changed: entries are append-only, add new ones at the end"
            ));
        }
        Ok(())
    }

    #[test]
    fn the_decision_log_is_well_formed_and_only_appended_to() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/decisions.md"
        ))
        .unwrap();
        if let Err(e) = check_log(&text, PINNED_LEN, PINNED_SHA256) {
            panic!(
                "docs/decisions.md: {e} (actual prefix hash {})",
                athena_data::sha256_hex(&text.as_bytes()[..PINNED_LEN])
            );
        }
    }

    #[test]
    fn the_log_guard_rejects_a_missing_provenance_duplicate_and_descending_ids_and_edits() {
        let good =
            "# t\n\n## D01 · a\n\n- **Provenance:** x\n\n## D02 · b\n\n- **Provenance:** y\n";
        let hash = athena_data::sha256_hex(good.as_bytes());
        assert_eq!(check_log(good, good.len(), &hash), Ok(()));
        // Appending after the pinned bytes is allowed.
        let more = format!("{good}\n## D03 · c\n\n- **Provenance:** z\n");
        assert_eq!(check_log(&more, good.len(), &hash), Ok(()));
        // A new entry without Provenance.
        let bare = format!("{good}\n## D03 · c\n\n- **Decision:** z\n");
        assert!(
            check_log(&bare, good.len(), &hash)
                .unwrap_err()
                .contains("D3 has no Provenance")
        );
        // A duplicate id and a descending id.
        let dup = format!("{good}\n## D02 · c\n\n- **Provenance:** z\n");
        assert!(
            check_log(&dup, good.len(), &hash)
                .unwrap_err()
                .contains("twice")
        );
        let down = format!("{good}\n## D01 · c\n\n- **Provenance:** z\n");
        assert!(
            check_log(&down, good.len(), &hash)
                .unwrap_err()
                .contains("ascend")
        );
        // An entry inserted in the middle (ids stay ascending) changes the pinned bytes.
        let spaced =
            "# t\n\n## D01 · a\n\n- **Provenance:** x\n\n## D03 · b\n\n- **Provenance:** y\n";
        let spaced_hash = athena_data::sha256_hex(spaced.as_bytes());
        let mid = spaced.replace("## D03", "## D02 · m\n\n- **Provenance:** m\n\n## D03");
        let hash = spaced_hash;
        let good = spaced;
        assert!(
            check_log(&mid, good.len(), &hash)
                .unwrap_err()
                .contains("changed")
        );
        // An edit to an existing entry, and a removal.
        let edited = good.replace("x", "q");
        assert!(
            check_log(&edited, good.len(), &hash)
                .unwrap_err()
                .contains("changed")
        );
        assert!(
            check_log(&good[..good.len() - 1], good.len(), &hash)
                .unwrap_err()
                .contains("shorter")
        );
    }

    const LOG: &str = "## D01 · a\n- Revised, superseded by D02's rule.\n\n## D02 · b\n- **Decision:** Amends D01.\n\n## D03 · c\n- **Decision:** Supersedes D01.\n";

    fn list(text: &str) -> ConfirmedLinks {
        ConfirmedLinks::parse(text).unwrap()
    }

    #[test]
    fn the_confirmed_status_decides_not_the_phrase() {
        let parsed = parse_supersessions(LOG);
        let links = list(
            "[[link]]\nline = 2\nolder = \"D01\"\nnewer = \"D02\"\nstatus = \"amended\"\n\
             [[link]]\nline = 5\nolder = \"D01\"\nnewer = \"D02\"\nstatus = \"amended\"\n\
             [[link]]\nline = 8\nolder = \"D01\"\nnewer = \"D03\"\nstatus = \"superseded\"\n",
        );
        assert!(links.unlisted(&parsed.hits).is_empty());
        assert!(links.not_found(&parsed.hits).is_empty());
        let views = view(&parsed, &links);
        assert_eq!(views.len(), 3);
        assert_eq!(views[0].status, Status::Superseded);
        assert_eq!(views[0].superseded_by.as_deref(), Some("D03"));
        assert_eq!(views[0].amended_by, ["D02"]);
        assert_eq!(views[1].status, Status::Active);
        assert!(views[1].amended_by.is_empty());
    }

    #[test]
    fn unlisted_and_unfound_hits_are_reported_and_not_applied() {
        let parsed = parse_supersessions(LOG);
        let links = list(
            "[[link]]\nline = 99\nolder = \"D01\"\nnewer = \"D03\"\nstatus = \"superseded\"\n",
        );
        assert_eq!(links.unlisted(&parsed.hits).len(), 3);
        assert_eq!(links.not_found(&parsed.hits).len(), 1);
        assert!(
            view(&parsed, &links)
                .iter()
                .all(|v| v.status == Status::Active)
        );
    }

    #[test]
    fn two_superseders_of_one_entry_are_an_error() {
        let text = "[[link]]\nline = 1\nolder = \"D01\"\nnewer = \"D02\"\nstatus = \"superseded\"\n\
                    [[link]]\nline = 2\nolder = \"D01\"\nnewer = \"D03\"\nstatus = \"superseded\"\n";
        assert!(ConfirmedLinks::parse(text).is_err());
    }
}
