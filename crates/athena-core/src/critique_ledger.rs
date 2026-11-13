//! The critique ledger (`OLY-B-S6-01`): a living, machine-readable list of critiques, improvements and open
//! questions, in the committed file [`LEDGER_FILE`], one `[[entry]]` per item.
//!
//! [`parse_ledger`] reads the file strictly: only the top-level key `entry`, and in each entry only `id`,
//! `kind`, `subject`, `status`, `text` and `source`, all required strings. It refuses an unknown key, a kind
//! other than `critique`, `improvement` or `open_question`, a status other than `open` or `resolved`, blank
//! text, and a repeated id. [`check_standard`] holds Athena to the same standard as the others: the ledger must
//! hold at least one entry whose subject is `athena`, and every kind must appear. Athena's entries go through
//! the same parser as every other entry; there is no separate path for them.
//!
//! The ledger is living because entries are appended and statuses change; nothing requires it to be empty or
//! closed. Nothing in a command reads it yet; the test `critique_ledger` in `crates/athena-core/tests` parses the
//! real file.

use anyhow::{Result, anyhow, bail};

/// The committed ledger, relative to the repository root.
pub const LEDGER_FILE: &str = "docs/critique-ledger.toml";

/// The subject that names Athena itself.
pub const ATHENA_SUBJECT: &str = "athena";

/// What an entry is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Critique,
    Improvement,
    OpenQuestion,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::Critique, Kind::Improvement, Kind::OpenQuestion];

    fn parse(s: &str) -> Option<Kind> {
        match s {
            "critique" => Some(Kind::Critique),
            "improvement" => Some(Kind::Improvement),
            "open_question" => Some(Kind::OpenQuestion),
            _ => None,
        }
    }
}

/// Whether an entry still needs work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Open,
    Resolved,
}

/// One ledger entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: String,
    pub kind: Kind,
    /// What is critiqued or questioned: `athena`, or another named component.
    pub subject: String,
    pub status: Status,
    pub text: String,
    /// Where the item comes from: a file, a record or a review.
    pub source: String,
}

const KEYS: [&str; 6] = ["id", "kind", "subject", "status", "text", "source"];

/// Reads the ledger. No `[[entry]]` gives an empty list.
pub fn parse_ledger(text: &str) -> Result<Vec<Entry>> {
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
    let mut entries: Vec<Entry> = Vec::new();
    for (n, table) in tables.iter().enumerate() {
        let at = n + 1;
        if let Some((key, _)) = table.iter().find(|(key, _)| !KEYS.contains(key)) {
            bail!(
                "{LEDGER_FILE} entry {at} has the key `{key}`; only {} are allowed",
                KEYS.join(", ")
            );
        }
        let field = |key: &str| -> Result<String> {
            let value = table
                .get(key)
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("{LEDGER_FILE} entry {at} needs `{key}` as a string"))?;
            if value.trim().is_empty() {
                bail!("{LEDGER_FILE} entry {at} has a blank `{key}`");
            }
            Ok(value.trim().to_string())
        };
        let id = field("id")?;
        let kind_text = field("kind")?;
        let kind = Kind::parse(&kind_text).ok_or_else(|| {
            anyhow!(
                "{LEDGER_FILE} entry {id} has the kind `{kind_text}`; use critique, improvement or open_question"
            )
        })?;
        let status = match field("status")?.as_str() {
            "open" => Status::Open,
            "resolved" => Status::Resolved,
            other => {
                bail!("{LEDGER_FILE} entry {id} has the status `{other}`; use open or resolved")
            }
        };
        if entries.iter().any(|e| e.id == id) {
            bail!("{LEDGER_FILE} has the id `{id}` twice");
        }
        entries.push(Entry {
            id,
            kind,
            subject: field("subject")?,
            status,
            text: field("text")?,
            source: field("source")?,
        });
    }
    Ok(entries)
}

/// What is missing for Athena to be critiqued to the same standard as the others; empty when nothing is.
pub fn check_standard(entries: &[Entry]) -> Vec<String> {
    let mut gaps = Vec::new();
    if !entries.iter().any(|e| e.subject == ATHENA_SUBJECT) {
        gaps.push(format!("no entry has the subject `{ATHENA_SUBJECT}`"));
    }
    for kind in Kind::ALL {
        if !entries.iter().any(|e| e.kind == kind) {
            gaps.push(format!("no entry has the kind {kind:?}"));
        }
    }
    gaps
}
