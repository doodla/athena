//! Temporary indexes: building trees and commits without touching the user's index or work tree.

use crate::{Git, Result, null_oid};
use std::ffi::OsStr;
use std::path::Path;

/// An index file of Athena's own, used through `GIT_INDEX_FILE`.
#[derive(Debug)]
pub struct TempIndex<'a> {
    git: &'a Git,
    file: tempfile::TempPath,
}

/// One change to apply to an index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexEdit {
    /// Add or replace `path` with blob `oid` and `mode` (such as `100644`).
    Put {
        mode: String,
        oid: String,
        path: String,
    },
    /// Remove `path`.
    Remove { path: String },
}

impl<'a> TempIndex<'a> {
    /// A temporary index holding `treeish`.
    pub fn from_tree(git: &'a Git, treeish: &str) -> Result<Self> {
        let file = tempfile::NamedTempFile::new()?.into_temp_path();
        // git refuses to read an empty file as an index; start from no file at all.
        std::fs::remove_file(&file)?;
        let index = TempIndex { git, file };
        index.run(["read-tree", treeish], None)?;
        Ok(index)
    }

    fn env(&self) -> [(&str, &OsStr); 1] {
        [("GIT_INDEX_FILE", self.file.as_os_str())]
    }

    fn run<I, S>(&self, args: I, stdin: Option<&[u8]>) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.git.run_bytes(args, stdin, &self.env())
    }

    /// Applies puts and removals in one `update-index --index-info` call.
    pub fn apply(&self, edits: &[IndexEdit]) -> Result<()> {
        if edits.is_empty() {
            return Ok(());
        }
        let mut input = Vec::new();
        for edit in edits {
            let line = match edit {
                IndexEdit::Put { mode, oid, path } => format!("{mode} {oid}\t{path}"),
                IndexEdit::Remove { path } => format!("0 {}\t{path}", null_oid()),
            };
            input.extend_from_slice(line.as_bytes());
            input.push(0);
        }
        self.run(["update-index", "-z", "--index-info"], Some(&input))?;
        Ok(())
    }

    /// Writes the index as a tree and returns the tree id.
    pub fn write_tree(&self) -> Result<String> {
        let out = self.run(["write-tree"], None)?;
        Ok(String::from_utf8_lossy(&out).trim().to_string())
    }

    /// Writes every indexed file under `dir`, keeping modes and symlinks (`checkout-index -a`).
    pub fn checkout_all(&self, dir: &Path) -> Result<()> {
        std::fs::create_dir_all(dir)?;
        let mut prefix = dir.to_string_lossy().into_owned();
        if !prefix.ends_with('/') {
            prefix.push('/');
        }
        self.run(
            ["checkout-index", "-a", "-f", &format!("--prefix={prefix}")],
            None,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn edits_build_a_tree_without_touching_the_real_index() {
        let (dir, git) = repo_with(&[("a.txt", "a"), ("b.txt", "b")]);
        let head = git.head().unwrap().unwrap();
        let before = std::fs::read(dir.path().join(".git/index")).unwrap();
        let index = TempIndex::from_tree(&git, &head).unwrap();
        let oid = git.hash_object_w(b"new").unwrap();
        index
            .apply(&[
                IndexEdit::Put {
                    mode: "100644".into(),
                    oid,
                    path: "c/d.txt".into(),
                },
                IndexEdit::Remove {
                    path: "b.txt".into(),
                },
            ])
            .unwrap();
        let tree = index.write_tree().unwrap();
        let paths: Vec<String> = git
            .ls_tree(&tree)
            .unwrap()
            .into_iter()
            .map(|e| e.path)
            .collect();
        assert_eq!(paths, vec!["a.txt", "c/d.txt"]);
        assert_eq!(
            std::fs::read(dir.path().join(".git/index")).unwrap(),
            before
        );
    }
}
