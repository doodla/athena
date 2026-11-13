//! The critique ledger (OLY-B-S6-01): `docs/critique-ledger.toml` holds critiques, improvements and open
//! questions as `[[entry]]` tables, and this parser validates it strictly.
//!
//! Each entry has `id`, `kind`, `subject`, `status`, `text` and `source`. The parser rejects unknown keys, unknown
//! kinds or statuses, blank fields and duplicate ids. [`check_standard`] adds the ledger-wide rules: every kind
//! is present, and Athena itself is a subject. That check is weak by design: it shows the ledger covers Athena,
//! not that Athena is critiqued as hard as the others. Not wired into any gate.

use std::collections::BTreeSet;
use thiserror::Error;
use toml_edit::DocumentMut;

/// The kinds an entry may have.
pub const KINDS: [&str; 3] = ["critique", "improvement", "open_question"];
/// The statuses an entry may have.
pub const STATUSES: [&str; 4] = ["open", "accepted", "done", "dropped"];
/// The subject that names Athena.
pub const ATHENA: &str = "athena";

const KEYS: [&str; 6] = ["id", "kind", "subject", "status", "text", "source"];

/// Why a ledger was refused.
#[derive(Debug, Error, PartialEq)]
pub enum LedgerError {
    #[error("the ledger is not valid TOML: {0}")]
    Syntax(String),
    #[error("the ledger has the unknown top-level key `{0}`")]
    UnknownTopKey(String),
    #[error("entry {index}: `entry` must be an array of tables")]
    NotATable { index: usize },
    #[error("entry {index}: unknown key `{key}`")]
    UnknownKey { index: usize, key: String },
    #[error("entry {index}: `{key}` is missing, not a string, or blank")]
    BlankField { index: usize, key: &'static str },
    #[error("entry {id}: unknown kind `{kind}`")]
    UnknownKind { id: String, kind: String },
    #[error("entry {id}: unknown status `{status}`")]
    UnknownStatus { id: String, status: String },
    #[error("duplicate id `{0}`")]
    DuplicateId(String),
    #[error("the ledger has no entries")]
    Empty,
    #[error("no entry has the kind `{0}`")]
    MissingKind(&'static str),
    #[error("no entry has the subject `athena`")]
    NoAthenaEntry,
}

/// One ledger entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: String,
    pub kind: String,
    pub subject: String,
    pub status: String,
    pub text: String,
    pub source: String,
}

/// Parses and validates a ledger, including the ledger-wide rules of [`check_standard`].
pub fn parse(text: &str) -> Result<Vec<Entry>, LedgerError> {
    let entries = parse_entries(text)?;
    check_standard(&entries)?;
    Ok(entries)
}

/// Parses and validates each entry and the ids, without the ledger-wide coverage rules.
pub fn parse_entries(text: &str) -> Result<Vec<Entry>, LedgerError> {
    let doc: DocumentMut = text
        .parse()
        .map_err(|e: toml_edit::TomlError| LedgerError::Syntax(e.to_string()))?;
    if let Some((key, _)) = doc.iter().find(|(k, _)| *k != "entry") {
        return Err(LedgerError::UnknownTopKey(key.to_string()));
    }
    let tables = match doc.get("entry") {
        None => return Err(LedgerError::Empty),
        Some(item) => item
            .as_array_of_tables()
            .ok_or(LedgerError::NotATable { index: 0 })?,
    };
    let mut entries = Vec::new();
    let mut ids = BTreeSet::new();
    for (index, table) in tables.iter().enumerate() {
        if let Some((key, _)) = table.iter().find(|(k, _)| !KEYS.contains(k)) {
            return Err(LedgerError::UnknownKey {
                index,
                key: key.to_string(),
            });
        }
        let field = |key: &'static str| -> Result<String, LedgerError> {
            table
                .get(key)
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .ok_or(LedgerError::BlankField { index, key })
        };
        let entry = Entry {
            id: field("id")?,
            kind: field("kind")?,
            subject: field("subject")?,
            status: field("status")?,
            text: field("text")?,
            source: field("source")?,
        };
        if !KINDS.contains(&entry.kind.as_str()) {
            return Err(LedgerError::UnknownKind {
                id: entry.id,
                kind: entry.kind,
            });
        }
        if !STATUSES.contains(&entry.status.as_str()) {
            return Err(LedgerError::UnknownStatus {
                id: entry.id,
                status: entry.status,
            });
        }
        if !ids.insert(entry.id.clone()) {
            return Err(LedgerError::DuplicateId(entry.id));
        }
        entries.push(entry);
    }
    if entries.is_empty() {
        return Err(LedgerError::Empty);
    }
    Ok(entries)
}

/// The ledger-wide rules: every kind has an entry, and at least one entry has the subject `athena`.
pub fn check_standard(entries: &[Entry]) -> Result<(), LedgerError> {
    for kind in KINDS {
        if !entries.iter().any(|e| e.kind == kind) {
            return Err(LedgerError::MissingKind(kind));
        }
    }
    if !entries.iter().any(|e| e.subject == ATHENA) {
        return Err(LedgerError::NoAthenaEntry);
    }
    Ok(())
}
