//! The Iris budget guard (`OLY-B-SET-04`): generated images and assets come from Iris within a $15 lifetime
//! budget.
//!
//! Spend is recorded in the committed ledger [`LEDGER_FILE`], one `[[entry]]` per generation, in whole cents.
//! [`check_generation`] takes the ledger entries and the cost of a proposed generation and refuses it when the
//! lifetime total would pass [`LIFETIME_CAP_CENTS`]; a total of exactly the cap is allowed. A generation path
//! must call it before every generation and append an entry only after it returned `Ok`.
//!
//! The cap is a constant here and not a field of the ledger, so editing the data file cannot raise it.
//! [`parse_ledger`] reads only `date`, `cents` and `note`; any other key is an error, so the ledger cannot
//! carry a credential. Calling Iris and handling its credentials are not part of this module, and nothing calls
//! the guard yet because no Iris client exists.
//!
//! The second half of the requirement, that Athena uses the harness-engineering bundle as a reference, is
//! prose and not machine-checked: it is credited in the Attribution section of `README.md` and in section 15 of
//! `docs/design/athena-v0.md`.

use anyhow::{Result, anyhow, bail};
use thiserror::Error;

/// The committed ledger of Iris spend, relative to the repository root.
pub const LEDGER_FILE: &str = "docs/iris-ledger.toml";

/// The lifetime budget for generated images and assets: $15, in cents.
pub const LIFETIME_CAP_CENTS: u64 = 1500;

/// One recorded Iris spend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerEntry {
    /// When the generation was paid for (free text, `YYYY-MM-DD` by convention).
    pub date: String,
    /// What it cost, in whole cents.
    pub cents: u64,
    /// What was generated.
    pub note: String,
}

/// A generation the lifetime budget does not allow.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error(
    "the Iris budget refuses this generation: {spent_cents} cents spent and {proposed_cents} proposed pass the \
     lifetime cap of {cap_cents} cents"
)]
pub struct BudgetRefusal {
    pub spent_cents: u64,
    pub proposed_cents: u64,
    pub cap_cents: u64,
}

/// Reads the ledger. No `[[entry]]` gives an empty list. Only the top-level key `entry` is accepted, and in
/// each entry only `date` (a string), `cents` (a non-negative integer) and `note` (a string), all required.
pub fn parse_ledger(text: &str) -> Result<Vec<LedgerEntry>> {
    let doc: toml_edit::DocumentMut = text
        .parse()
        .map_err(|e| anyhow!("{LEDGER_FILE} is not valid TOML: {e}"))?;
    if let Some((key, _)) = doc.iter().find(|(key, _)| *key != "entry") {
        bail!("{LEDGER_FILE} has the key `{key}`; only `[[entry]]` tables are allowed");
    }
    let Some(item) = doc.get("entry") else {
        return Ok(Vec::new());
    };
    let Some(tables) = item.as_array_of_tables() else {
        bail!("{LEDGER_FILE}: `entry` must be an array of tables, `[[entry]]`");
    };
    let mut entries = Vec::new();
    for (n, table) in tables.iter().enumerate() {
        let at = n + 1;
        if let Some((key, _)) = table
            .iter()
            .find(|(key, _)| !matches!(*key, "date" | "cents" | "note"))
        {
            bail!(
                "{LEDGER_FILE} entry {at} has the key `{key}`; only date, cents and note are allowed"
            );
        }
        let text_field = |key: &str| {
            table
                .get(key)
                .and_then(toml_edit::Item::as_str)
                .map(str::to_string)
                .ok_or_else(|| anyhow!("{LEDGER_FILE} entry {at}: `{key}` must be a string"))
        };
        let cents = table
            .get("cents")
            .and_then(toml_edit::Item::as_integer)
            .and_then(|c| u64::try_from(c).ok())
            .ok_or_else(|| {
                anyhow!("{LEDGER_FILE} entry {at}: `cents` must be a non-negative integer")
            })?;
        entries.push(LedgerEntry {
            date: text_field("date")?,
            cents,
            note: text_field("note")?,
        });
    }
    Ok(entries)
}

/// The lifetime spend in cents. The sum saturates, so it cannot wrap.
pub fn spent_cents(entries: &[LedgerEntry]) -> u64 {
    entries
        .iter()
        .fold(0u64, |total, e| total.saturating_add(e.cents))
}

/// Allows a generation costing `proposed_cents` when the lifetime spend with it stays within
/// [`LIFETIME_CAP_CENTS`] (exactly the cap is allowed), and refuses it otherwise.
pub fn check_generation(entries: &[LedgerEntry], proposed_cents: u64) -> Result<(), BudgetRefusal> {
    check_against(entries, proposed_cents, LIFETIME_CAP_CENTS)
}

/// [`check_generation`] with an explicit cap.
fn check_against(
    entries: &[LedgerEntry],
    proposed_cents: u64,
    cap_cents: u64,
) -> Result<(), BudgetRefusal> {
    let spent_cents = spent_cents(entries);
    if spent_cents.saturating_add(proposed_cents) > cap_cents {
        return Err(BudgetRefusal {
            spent_cents,
            proposed_cents,
            cap_cents,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ledger(amounts: &[u64]) -> Vec<LedgerEntry> {
        let text: String = amounts
            .iter()
            .map(|c| {
                format!("[[entry]]\ndate = \"2026-01-01\"\ncents = {c}\nnote = \"test image\"\n")
            })
            .collect();
        parse_ledger(&text).unwrap()
    }

    #[test]
    fn a_generation_over_the_lifetime_budget_is_refused() {
        let entries = ledger(&[1400]);
        // Without the cap the same input is allowed, so the refusal comes from the guard.
        assert_eq!(check_against(&entries, 101, u64::MAX), Ok(()));
        assert_eq!(
            check_generation(&entries, 101),
            Err(BudgetRefusal {
                spent_cents: 1400,
                proposed_cents: 101,
                cap_cents: 1500,
            })
        );
        assert!(
            check_generation(&entries, 101)
                .unwrap_err()
                .to_string()
                .contains("1400 cents spent and 101 proposed")
        );
        assert!(check_generation(&ledger(&[]), 1501).is_err());
    }

    #[test]
    fn a_generation_within_or_exactly_at_the_budget_is_allowed() {
        assert_eq!(check_generation(&ledger(&[1400]), 100), Ok(()));
        assert_eq!(check_generation(&ledger(&[]), 1500), Ok(()));
        assert_eq!(check_generation(&ledger(&[]), 250), Ok(()));
        assert_eq!(check_generation(&ledger(&[1500]), 0), Ok(()));
    }

    #[test]
    fn spend_is_summed_over_every_entry() {
        let entries = ledger(&[500, 300]);
        assert_eq!(spent_cents(&entries), 800);
        assert_eq!(check_generation(&entries, 700), Ok(()));
        assert!(check_generation(&entries, 701).is_err());
    }

    #[test]
    fn a_huge_cost_is_refused_without_overflow() {
        assert!(check_generation(&ledger(&[1]), u64::MAX).is_err());
        assert!(check_generation(&ledger(&[]), u64::MAX).is_err());
        let full = vec![
            LedgerEntry {
                date: "d".into(),
                cents: u64::MAX,
                note: "n".into(),
            };
            2
        ];
        assert_eq!(spent_cents(&full), u64::MAX);
        assert!(check_generation(&full, 1).is_err());
    }

    #[test]
    fn the_committed_ledger_parses_and_is_within_budget() {
        let entries = parse_ledger(include_str!("../../../docs/iris-ledger.toml")).unwrap();
        assert!(spent_cents(&entries) <= LIFETIME_CAP_CENTS, "{entries:?}");
        // Every field of an entry is one of the three the parser reads; a credential-like key is an error.
        assert!(check_generation(&entries, 0).is_ok());
    }

    #[test]
    fn a_credential_or_malformed_field_is_refused() {
        let entry = "[[entry]]\ndate = \"2026-01-01\"\ncents = 5\nnote = \"n\"\n";
        assert!(parse_ledger(entry).is_ok());
        let cases = [
            format!("{entry}api_key = \"x\"\n"),
            format!("token = \"x\"\n{entry}"),
            format!("{entry}\n[[entry]]\ndate = \"d\"\ncents = 1\nnote = \"n\"\nsecret = \"x\"\n"),
            "[[entry]]\ndate = \"d\"\ncents = -5\nnote = \"n\"\n".to_string(),
            "[[entry]]\ndate = \"d\"\ncents = \"5\"\nnote = \"n\"\n".to_string(),
            "[[entry]]\ndate = \"d\"\ncents = 1.5\nnote = \"n\"\n".to_string(),
            "[[entry]]\ndate = \"d\"\ncents = 5\n".to_string(),
            "entry = 3\n".to_string(),
            "[[entry]\n".to_string(),
        ];
        for case in &cases {
            assert!(parse_ledger(case).is_err(), "{case}");
        }
        assert_eq!(parse_ledger("").unwrap(), Vec::new());
    }

    #[test]
    fn iris_spec_entry_names_files_that_exist() {
        let requirements =
            crate::hestia::parse_spec(include_str!("../../../docs/spec.toml")).unwrap();
        let r = requirements
            .iter()
            .find(|r| r.id == "OLY-B-SET-04")
            .expect("docs/spec.toml has OLY-B-SET-04");
        assert!(r.active);
        assert_eq!(r.owner, "Athena");
        assert!(r.mechanism.contains("crates/athena-core/src/iris.rs"));
        assert!(r.mechanism.contains(LEDGER_FILE));
        assert!(r.measure.ends_with("status: not shown"), "{}", r.measure);
        assert!(r.measure.contains("cargo test -p athena-core iris"));
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        assert!(root.join("crates/athena-core/src/iris.rs").is_file());
        assert!(root.join(LEDGER_FILE).is_file());
    }
}
