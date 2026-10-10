//! Reading every record in a tree, for validation, reconciliation, and status.

use crate::Result;
use athena_data::records::{AnyRecord, RecordError, check_record_file};
use athena_git::Git;

/// One file under `records/` in a tree.
#[derive(Debug, Clone)]
pub struct RecordFile {
    pub path: String,
    /// The parsed record, for JSON files that are valid and in place.
    pub record: Option<AnyRecord>,
    /// Why the file is not acceptable, if it is not.
    pub error: Option<RecordError>,
    /// The raw bytes, kept for Markdown briefs.
    pub bytes: Vec<u8>,
}

/// Every file under `records/` in a tree.
#[derive(Debug, Clone, Default)]
pub struct RecordSet {
    pub files: Vec<RecordFile>,
}

impl RecordSet {
    /// Loads and checks every file under `records/` in `treeish`.
    pub fn load(git: &Git, treeish: &str) -> Result<RecordSet> {
        let entries: Vec<_> = git
            .ls_tree_under(treeish, "records")?
            .into_iter()
            .filter(|e| e.kind == "blob")
            .collect();
        let oids: Vec<String> = entries.iter().map(|e| e.oid.clone()).collect();
        let blobs = git.cat_blobs(&oids)?;
        let files = entries
            .into_iter()
            .zip(blobs)
            .map(
                |(entry, bytes)| match check_record_file(&entry.path, &bytes) {
                    Ok(record) => RecordFile {
                        path: entry.path,
                        record,
                        error: None,
                        bytes,
                    },
                    Err(error) => RecordFile {
                        path: entry.path,
                        record: None,
                        error: Some(error),
                        bytes,
                    },
                },
            )
            .collect();
        Ok(RecordSet { files })
    }

    /// Every parsed record with its path.
    pub fn records(&self) -> impl Iterator<Item = (&str, &AnyRecord)> {
        self.files
            .iter()
            .filter_map(|f| f.record.as_ref().map(|r| (f.path.as_str(), r)))
    }

    /// Files that are not acceptable records.
    pub fn errors(&self) -> impl Iterator<Item = &RecordError> {
        self.files.iter().filter_map(|f| f.error.as_ref())
    }

    /// The raw bytes of a file, if present.
    pub fn bytes(&self, path: &str) -> Option<&[u8]> {
        self.files
            .iter()
            .find(|f| f.path == path)
            .map(|f| f.bytes.as_slice())
    }
}
