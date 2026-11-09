//! Reading every record in a tree, for validation, reconciliation, and status.

use crate::Result;
use athena_data::records::{AnyRecord, RecordError, check_record_file};
use athena_git::Git;
use std::path::Path;

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

    /// Loads and checks every file under `<dir>/records/` on disk, with paths relative to `dir` (such as
    /// `records/jobs/...`) and sorted by path. An absent `records/` directory gives an empty set.
    pub fn load_dir(dir: &Path) -> Result<RecordSet> {
        fn walk(root: &Path, at: &Path, out: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
            let Ok(entries) = std::fs::read_dir(at) else {
                return Ok(());
            };
            for entry in entries {
                let path = entry?.path();
                if path.is_dir() {
                    walk(root, &path, out)?;
                } else {
                    let relative = path
                        .strip_prefix(root)
                        .expect("walked paths are under the root")
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy().into_owned())
                        .collect::<Vec<_>>()
                        .join("/");
                    out.push((relative, std::fs::read(&path)?));
                }
            }
            Ok(())
        }
        let mut found = Vec::new();
        walk(dir, &dir.join("records"), &mut found)?;
        found.sort_by(|a, b| a.0.cmp(&b.0));
        let files = found
            .into_iter()
            .map(|(path, bytes)| match check_record_file(&path, &bytes) {
                Ok(record) => RecordFile {
                    path,
                    record,
                    error: None,
                    bytes,
                },
                Err(error) => RecordFile {
                    path,
                    record: None,
                    error: Some(error),
                    bytes,
                },
            })
            .collect();
        Ok(RecordSet { files })
    }

    /// Loads the current records: those in the `athena/records` worktree at `worktree` when that directory
    /// exists, and those under `records/` in `treeish` only when it is absent (D111). An existing worktree
    /// is read as it is, even when it is empty, so a broken one is not hidden behind the in-tree records.
    pub fn load_current(git: &Git, treeish: &str, worktree: &Path) -> Result<RecordSet> {
        if worktree.is_dir() {
            RecordSet::load_dir(worktree)
        } else {
            RecordSet::load(git, treeish)
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{repo, write};

    const TREE: &str = "records/jobs/bootstrap/brief.md";
    const WORKTREE: &str = "records/jobs/bootstrap/brief-update-1.md";

    fn paths(set: &RecordSet) -> Vec<&str> {
        set.files.iter().map(|f| f.path.as_str()).collect()
    }

    #[test]
    fn an_absent_worktree_reads_the_in_tree_records() {
        let (dir, git) = repo(&[(TREE, "tree")]);
        let absent = dir.path().join("no-such-worktree");
        let current = RecordSet::load_current(&git, "HEAD", &absent).unwrap();
        let loaded = RecordSet::load(&git, "HEAD").unwrap();
        assert_eq!(paths(&current), paths(&loaded));
        assert_eq!(paths(&current), vec![TREE]);
    }

    #[test]
    fn an_existing_worktree_is_read_instead_of_the_in_tree_records() {
        let (_dir, git) = repo(&[(TREE, "tree")]);
        let worktree = tempfile::tempdir().unwrap();
        write(worktree.path(), &[(WORKTREE, "worktree")]);
        let current = RecordSet::load_current(&git, "HEAD", worktree.path()).unwrap();
        assert_eq!(paths(&current), vec![WORKTREE]);
        assert!(current.errors().next().is_none());
        assert_eq!(current.bytes(WORKTREE), Some(b"worktree".as_slice()));
    }

    #[test]
    fn an_empty_existing_worktree_does_not_fall_back() {
        let (_dir, git) = repo(&[(TREE, "tree")]);
        let worktree = tempfile::tempdir().unwrap();
        let current = RecordSet::load_current(&git, "HEAD", worktree.path()).unwrap();
        assert!(current.files.is_empty());
    }

    #[test]
    fn load_dir_sorts_by_path_and_flags_unknown_files() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            &[(WORKTREE, "b"), (TREE, "a"), ("records/stray.txt", "x")],
        );
        let set = RecordSet::load_dir(dir.path()).unwrap();
        assert_eq!(paths(&set), vec![WORKTREE, TREE, "records/stray.txt"]);
        assert_eq!(set.errors().count(), 1);
    }
}
