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
