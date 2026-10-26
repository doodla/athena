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

    /// Loads and checks every file under `records/` in a directory root (a checkout or the records
    /// worktree). Paths are relative to the root, as `load` reports them.
    pub fn load_dir(root: &std::path::Path) -> Result<RecordSet> {
        fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
            for entry in std::fs::read_dir(dir)? {
                let path = entry?.path();
                if path.is_dir() {
                    walk(&path, out)?;
                } else {
                    out.push(path);
                }
            }
            Ok(())
        }
        let mut paths = Vec::new();
        let base = athena_data::paths::records_dir(root);
        if base.is_dir() {
            walk(&base, &mut paths)?;
        }
        paths.sort();
        let mut files = Vec::new();
        for path in paths {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            let bytes = std::fs::read(&path)?;
            files.push(match check_record_file(&rel, &bytes) {
                Ok(record) => RecordFile {
                    path: rel,
                    record,
                    error: None,
                    bytes,
                },
                Err(error) => RecordFile {
                    path: rel,
                    record: None,
                    error: Some(error),
                    bytes,
                },
            });
        }
        Ok(RecordSet { files })
    }

    /// The live records: the records worktree when it holds `records/`, otherwise `treeish` as `load`
    /// reads it. After the D111 cutover the checkout's HEAD has no `records/`, so live readers use this.
    pub fn load_current(git: &Git, treeish: &str) -> Result<RecordSet> {
        if let Ok(common) = git.common_dir() {
            let worktree = athena_data::paths::records_worktree(&common);
            if athena_data::paths::records_dir(&worktree).is_dir() {
                return Self::load_dir(&worktree);
            }
        }
        Self::load(git, treeish)
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
