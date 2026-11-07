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
//!
//! The same file holds a `[[governs]]` table: each decision and the code paths it governs. The log is prose, so
//! nothing else tied a decision to the code it describes, and a change that reversed one was found only at the
//! final verdict. A change that touches a governed path must now add an entry that amends or supersedes the
//! decision, with a `[[link]]`, before the verifier starts (`athena_data::decision_links`, run by
//! `athena-core`). The list is read and validated here through the same parser; a governed id that is not an
//! entry of the log is a finding of condition (a).

use crate::backtest::{Hit, Parsed, Status};
use anyhow::{Context, Result, bail};
use athena_data::decision_links::Governs;
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

/// The confirmed list, as stored in `docs/decision-links.toml` (`[[link]]` and `[[governs]]` tables).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ConfirmedLinks {
    #[serde(default, rename = "link")]
    pub links: Vec<Link>,
    /// The decisions and the code paths they govern. A change to a governed path must amend or supersede the
    /// decision before verification (`athena_data::decision_links`).
    #[serde(default, rename = "governs")]
    pub governs: Vec<Governs>,
}

impl ConfirmedLinks {
    /// Parse the text of the list. A decision superseded by two different entries is an error: `superseded_by`
    /// of the view holds one id. A `[[governs]]` entry with a malformed id, no paths or a path outside the
    /// repository is an error too; whether its id is an entry of the log is checked by [`Self::unknown_governed`].
    pub fn parse(text: &str) -> Result<Self> {
        let list: ConfirmedLinks =
            toml::from_str(text).context("parsing the confirmed link list")?;
        athena_data::decision_links::parse_governs(text).map_err(anyhow::Error::msg)?;
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

    /// The governs entries whose decision is not among `entries`, the ids of the log's headings. Such an entry
    /// governs a decision that does not exist, so no entry could ever amend it.
    pub fn unknown_governed(&self, entries: &[String]) -> Vec<&Governs> {
        self.governs
            .iter()
            .filter(|g| !entries.contains(&g.decision))
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

    /// Length in bytes of `docs/decisions.md` when the log was last known good (it ended with D158). Entries
    /// are never rewritten, so these bytes must stay exactly as they were; new entries go after them.
    const PINNED_LEN: usize = 359622;
    /// SHA-256 of the first `PINNED_LEN` bytes, computed from the file at that commit (`git show
    /// <commit>:docs/decisions.md`) before D159 was appended. Raise both constants only when a later
    /// entry is appended, to the length and hash of the log up to and including that entry.
    const PINNED_SHA256: &str = "4bec69d2468c86b0c361a6daa9eacb1ac63f0558e48d62ce4227e4087b19d648";

    /// The tail of every error about the pinned base or the place a new entry goes.
    const APPEND_AT_END: &str = "append at end of file";

    /// Check the log's structure: the first `pinned_len` bytes hash to `pinned_sha256` (existing entries
    /// untouched, with additions only after them); every `## D<n> · ` entry has a Provenance bullet; ids are
    /// unique and ascending; and an entry that starts at or after the pinned base has the id one greater than
    /// the highest before it. The pin is checked first, so an insertion before the end of the pinned base is
    /// reported as that and not as whatever else the insertion upsets. Every error about the pin or about a
    /// new entry's place ends with `append at end of file`.
    ///
    /// The id rule applies only past the pin: the pinned part of the real log has a gap that the parse cannot
    /// see (the heading of D145 is not in the `## D<n> · ` shape), and it may not be rewritten.
    fn check_log(text: &str, pinned_len: usize, pinned_sha256: &str) -> Result<(), String> {
        let prefix = text.as_bytes().get(..pinned_len).ok_or_else(|| {
            format!(
                "the log is shorter than the pinned base: existing entries were removed; {APPEND_AT_END}"
            )
        })?;
        if athena_data::sha256_hex(prefix) != pinned_sha256 {
            return Err(format!(
                "the first {pinned_len} bytes changed: entries are append-only; {APPEND_AT_END}"
            ));
        }
        if !athena_core::mirrors::parse_decision_entries(text)
            .iter()
            .any(|e| e.shaped)
        {
            return Err("no decision entries found".into());
        }
        // Shape, ids and Provenance are defined once, in the mirror rule `decision entries` that the producer
        // step runs, so this test and that rule cannot disagree.
        match athena_core::mirrors::decision_log_problems(text, pinned_len, true)
            .into_iter()
            .next()
        {
            Some(problem) => Err(problem),
            None => Ok(()),
        }
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
        let error = check_log(&mid, good.len(), &hash).unwrap_err();
        assert!(error.contains("changed"), "{error}");
        assert!(error.ends_with("append at end of file"), "{error}");
        // An edit to an existing entry, and a removal.
        let edited = good.replace("x", "q");
        assert!(
            check_log(&edited, good.len(), &hash)
                .unwrap_err()
                .contains("changed")
        );
        let error = check_log(&good[..good.len() - 1], good.len(), &hash).unwrap_err();
        assert!(error.contains("shorter"), "{error}");
        assert!(error.ends_with("append at end of file"), "{error}");
    }

    #[test]
    fn the_log_guard_and_the_producer_step_rule_give_the_same_finding_text() {
        // The same parser: a new entry with no Provenance and a repeated id read the same in both.
        let base = "# t\n\n## D01 · a\n\n- **Provenance:** x\n";
        let hash = athena_data::sha256_hex(base.as_bytes());
        for (tail, expect) in [
            (
                "\n## D02 · b\n\n- **Decision:** z\n",
                "D2 has no Provenance bullet",
            ),
            ("\n## D01 · b\n\n- **Provenance:** z\n", "D1 appears twice"),
        ] {
            let new = format!("{base}{tail}");
            let guard = check_log(&new, base.len(), &hash).unwrap_err();
            let rule = athena_core::mirrors::decision_log_problems(&new, base.len(), false);
            assert!(guard.contains(expect), "{guard}");
            assert_eq!(
                rule,
                vec![guard],
                "the producer step rule and the guard differ"
            );
        }
    }

    #[test]
    fn the_log_guard_tells_where_a_new_entry_goes() {
        let base =
            "# t\n\n## D01 · a\n\n- **Provenance:** x\n\n## D02 · b\n\n- **Provenance:** y\n";
        let hash = athena_data::sha256_hex(base.as_bytes());
        let entry = |id: &str| format!("\n## D{id} · c\n\n- **Provenance:** z\n");
        // (a) An entry inserted before the end of the pinned base, ids ascending, whole entry well formed.
        let inserted = base.replacen("## D02", "## D015 · m\n\n- **Provenance:** m\n\n## D02", 1);
        // The inserted id does not matter: the bytes before the end of the pinned base changed.
        let error = check_log(&inserted, base.len(), &hash).unwrap_err();
        assert!(error.contains("changed"), "{error}");
        assert!(error.ends_with("append at end of file"), "{error}");
        // The same insertion as a duplicate id is still reported as a change of the pinned base.
        let duplicate = base.replacen("## D02", "## D01 · m\n\n- **Provenance:** m\n\n## D02", 1);
        let error = check_log(&duplicate, base.len(), &hash).unwrap_err();
        assert!(error.ends_with("append at end of file"), "{error}");
        // (b) A new entry whose id skips one, or repeats the highest, or goes back.
        for id in ["04", "10"] {
            let skipped = format!("{base}{}", entry(id));
            let error = check_log(&skipped, base.len(), &hash).unwrap_err();
            assert!(
                error.contains(&format!("D{} follows D2", id.trim_start_matches('0'))),
                "{error}"
            );
            assert!(
                error.contains("one greater than the previous highest (D3)"),
                "{error}"
            );
            assert!(error.ends_with("append at end of file"), "{error}");
        }
        assert!(
            check_log(&format!("{base}{}", entry("02")), base.len(), &hash)
                .unwrap_err()
                .contains("twice")
        );
        // (c) The entry with the next id is accepted, and so is another one after it.
        let appended = format!("{base}{}", entry("03"));
        assert_eq!(check_log(&appended, base.len(), &hash), Ok(()));
        let two = format!("{appended}{}", entry("04"));
        assert_eq!(check_log(&two, base.len(), &hash), Ok(()));
        // A gap inside the pinned base is not judged: the rule applies from the end of the pin.
        let gapped =
            "# t\n\n## D01 · a\n\n- **Provenance:** x\n\n## D04 · b\n\n- **Provenance:** y\n";
        let gapped_hash = athena_data::sha256_hex(gapped.as_bytes());
        assert_eq!(check_log(gapped, gapped.len(), &gapped_hash), Ok(()));
        assert!(
            check_log(
                &format!("{gapped}{}", entry("06")),
                gapped.len(),
                &gapped_hash
            )
            .is_err()
        );
        assert_eq!(
            check_log(
                &format!("{gapped}{}", entry("05")),
                gapped.len(),
                &gapped_hash
            ),
            Ok(())
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
    fn governs_entries_are_read_and_a_list_without_any_still_parses() {
        let links = list(
            "[[link]]\nline = 2\nolder = \"D01\"\nnewer = \"D02\"\nstatus = \"amended\"\n\n\
             [[governs]]\ndecision = \"D01\"\npaths = [\"src/a.rs\", \"docs/\"]\n",
        );
        assert_eq!(links.links.len(), 1);
        assert_eq!(links.governs.len(), 1);
        assert_eq!(links.governs[0].decision, "D01");
        assert_eq!(links.governs[0].paths, ["src/a.rs", "docs/"]);
        assert!(list("").governs.is_empty());
    }

    #[test]
    fn a_malformed_governs_entry_is_an_error() {
        for bad in [
            "[[governs]]\ndecision = \"D1\"\npaths = [\"a\"]\n",
            "[[governs]]\ndecision = \"D01\"\npaths = []\n",
            "[[governs]]\ndecision = \"D01\"\npaths = [\"/abs\"]\n",
            "[[governs]]\ndecision = \"D01\"\npaths = [\"../up\"]\n",
            "[[governs]]\ndecision = \"D01\"\n",
            "[[governs]]\ndecision = \"D01\"\npaths = [\"a\"]\ntypo = 1\n",
        ] {
            assert!(ConfirmedLinks::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn an_unknown_governed_decision_is_reported() {
        let links = list(
            "[[governs]]\ndecision = \"D01\"\npaths = [\"a\"]\n\n\
             [[governs]]\ndecision = \"D9999\"\npaths = [\"b\"]\n",
        );
        let entries = vec!["D01".to_string(), "D02".to_string()];
        let unknown = links.unknown_governed(&entries);
        assert_eq!(unknown.len(), 1);
        assert_eq!(unknown[0].decision, "D9999");
    }

    fn repo_file(path: &str) -> String {
        std::fs::read_to_string(format!("{}/../../{path}", env!("CARGO_MANIFEST_DIR"))).unwrap()
    }

    #[test]
    fn the_real_list_maps_d08_to_the_snapshot_and_candidate_code_and_names_only_logged_decisions() {
        let links = ConfirmedLinks::parse(&repo_file(LINKS_PATH)).unwrap();
        let d08 = links
            .governs
            .iter()
            .find(|g| g.decision == "D08")
            .expect("D08 is governed");
        for path in [
            "crates/athena-git/src/snapshot.rs",
            "crates/athena-core/src/run/candidate.rs",
        ] {
            assert!(d08.covers(path), "{path}");
            assert!(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .join(path)
                    .exists(),
                "{path} no longer exists: update the list"
            );
        }
        let entries = parse_supersessions(&repo_file("docs/decisions.md")).entries;
        assert!(links.unknown_governed(&entries).is_empty());
    }

    /// The sentence that names the root cause, in the changelog and in the log entry that records the fix.
    const ROOT_CAUSE: &str = "The decision log is prose with no link from a decision to the code or limit it governs, so reversals were found only at the final verdict.";

    #[test]
    fn the_root_cause_is_named_in_the_changelog_and_the_log() {
        let words = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
        let log = repo_file("docs/decisions.md");
        let entry = log
            .split("\n## ")
            .find(|e| e.starts_with("D156 · "))
            .expect("D156 records the decision-to-code link");
        assert!(entry.contains("- **Provenance:**"));
        assert!(
            words(entry).contains(ROOT_CAUSE),
            "D156 lacks the root cause"
        );
        let changelog = repo_file("CHANGELOG.md");
        let unreleased = changelog
            .split("\n## [")
            .find(|s| s.starts_with("Unreleased]"))
            .expect("an Unreleased section");
        assert!(
            words(unreleased).contains(ROOT_CAUSE),
            "the Unreleased changelog lacks the root cause"
        );
    }

    #[test]
    fn two_superseders_of_one_entry_are_an_error() {
        let text = "[[link]]\nline = 1\nolder = \"D01\"\nnewer = \"D02\"\nstatus = \"superseded\"\n\
                    [[link]]\nline = 2\nolder = \"D01\"\nnewer = \"D03\"\nstatus = \"superseded\"\n";
        assert!(ConfirmedLinks::parse(text).is_err());
    }
}
