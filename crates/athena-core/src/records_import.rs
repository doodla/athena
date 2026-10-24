//! The records importer (§4 and §10 item 2 of `docs/design/records-history.md`, D111).
//!
//! Given a commit `C0`, [`import_records`] copies every file under `records/` at `C0` into **one commit** on
//! the records branch `athena/records`, at the same paths. The same commit holds `import-manifest.json` at
//! the root of the branch: an [`ImportManifest`] that lists each source path, its git blob id at `C0` and the
//! SHA-256 of its contents. The commit is written with [`RecordsWriter`], so it is an orphan commit published
//! by compare-and-swap.
//!
//! # The self-check
//!
//! After writing, [`verify_import`] re-reads the new commit (not the data it just wrote) and compares it with
//! `C0`. It fails, each with its own [`ImportError`] variant, when:
//!
//! - the manifest is missing, does not parse as an import manifest, does not validate, or names another `C0`;
//! - a `records/` file at `C0` is absent from the commit ([`ImportError::Missing`]);
//! - the commit holds a file that `C0` does not ([`ImportError::Extra`]);
//! - a file's blob id differs from `C0`'s ([`ImportError::BlobMismatch`]);
//! - a file's SHA-256 differs from the manifest's, or a manifest entry's blob id or SHA-256 differs from
//!   `C0`'s ([`ImportError::HashMismatch`], [`ImportError::BlobMismatch`]);
//! - the file counts of the commit or of the manifest differ from `C0`'s ([`ImportError::CountMismatch`]).
//!
//! # Refusals
//!
//! - Records are append-only, so a second import is refused ([`ImportError::AlreadyImported`]) when the
//!   branch already has a tip. The branch is not touched.
//! - An empty `records/` at `C0` is an error ([`ImportError::EmptyRecords`]): an import of nothing is not a
//!   success.
//! - Only regular files (mode `100644`) are imported. The writer stores nothing else, so a symlink or an
//!   executable is refused ([`ImportError::UnsupportedEntry`]) instead of being silently changed.
//!
//! If the self-check fails, the error is returned and the commit stays on the branch, because the branch is
//! append-only and nothing may remove a commit from it. Files are read into memory; `records/` holds small
//! JSON and text files.
//!
//! # Status
//!
//! The importer is **not wired in**: no command, job pipeline step, CI workflow or other caller runs it, and
//! it has not been run against this repository's `main`. It is tested on scratch repositories only. Running
//! it for real is the cutover (job 7 of the plan).

use athena_data::history::{HistoryRecord, ImportManifest, ImportedPath};
use athena_data::{RecordId, sha256_hex};
use athena_git::records_branch::{Change, RecordsError, RecordsWriter};
use athena_git::{Git, GitError};
use std::collections::BTreeMap;

/// Where the manifest is written, at the root of the records branch.
pub const MANIFEST_PATH: &str = "import-manifest.json";

/// The directory that is imported.
const SOURCE_DIR: &str = "records/";

/// Why an import was refused or its self-check failed.
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error(transparent)]
    Records(#[from] RecordsError),
    /// `c0` does not name a commit in this repository.
    #[error("{c0:?} is not a commit in this repository")]
    NotACommit { c0: String },
    /// The records branch already has a tip. Records are append-only, so nothing was written.
    #[error(
        "the records branch already exists at {tip}; records are append-only, so a second import is refused"
    )]
    AlreadyImported { tip: String },
    /// There is no file under `records/` at `c0`.
    #[error("there is no file under records/ at {c0}; an empty import is not a success")]
    EmptyRecords { c0: String },
    /// A `records/` entry that is not a regular file; the writer stores only mode 100644.
    #[error(
        "{path:?} at C0 is not a regular file (mode {mode}, type {kind}) and cannot be imported"
    )]
    UnsupportedEntry {
        path: String,
        mode: String,
        kind: String,
    },
    /// The commit has no `import-manifest.json`.
    #[error("the imported commit has no {MANIFEST_PATH}")]
    ManifestMissing,
    /// The manifest does not parse, does not validate, or names another `C0`.
    #[error("the import manifest is invalid: {0}")]
    ManifestInvalid(String),
    /// A file at `C0` is absent from the imported commit.
    #[error("{path:?} is at C0 but missing from the imported commit")]
    Missing { path: String },
    /// A file is in the imported commit (or the manifest) but not at `C0`.
    #[error("{path:?} is in the imported commit but not at C0")]
    Extra { path: String },
    /// A blob id differs from `C0`'s.
    #[error("{path:?} has blob {found} but C0 has {expected}")]
    BlobMismatch {
        path: String,
        expected: String,
        found: String,
    },
    /// A SHA-256 differs from the manifest's, or the manifest's from `C0`'s.
    #[error("the SHA-256 of {path:?} does not match")]
    HashMismatch { path: String },
    /// The number of files differs from `C0`'s.
    #[error("{what} lists {found} files but C0 has {expected}")]
    CountMismatch {
        what: &'static str,
        expected: usize,
        found: usize,
    },
}

/// Copies `records/` at `c0` to the records branch in one commit with [`MANIFEST_PATH`], checks the new commit
/// with [`verify_import`], and returns its id. See the module docs for the refusals.
pub fn import_records(git: &Git, c0: &str) -> Result<String, ImportError> {
    import_with(git, c0, |_| {})
}

/// As [`import_records`], letting `tamper` change the batch before it is written. Tests use it to commit a bad
/// import and show the self-check catches it. The self-check always runs on the committed tree.
fn import_with(
    git: &Git,
    c0: &str,
    tamper: impl FnOnce(&mut Vec<Change>),
) -> Result<String, ImportError> {
    let c0 = git
        .rev_parse(c0)?
        .ok_or_else(|| ImportError::NotACommit { c0: c0.to_string() })?;
    let writer = RecordsWriter::new(git);
    if let Some(tip) = writer.tip()? {
        return Err(ImportError::AlreadyImported { tip });
    }

    let files = source_files(git, &c0)?;
    if files.is_empty() {
        return Err(ImportError::EmptyRecords { c0 });
    }
    let mut changes = Vec::with_capacity(files.len() + 1);
    let mut paths = Vec::with_capacity(files.len());
    for (path, blob_id) in &files {
        let bytes = git.cat_blob(blob_id)?;
        paths.push(ImportedPath {
            path: path.clone(),
            blob_id: blob_id.clone(),
            sha256: sha256_hex(&bytes),
        });
        changes.push(Change::put(path.clone(), bytes));
    }
    let manifest = ImportManifest {
        id: RecordId::generate(crate::now()),
        created_at: crate::now(),
        c0: c0.clone(),
        paths,
    };
    manifest
        .validate()
        .map_err(|e| ImportError::ManifestInvalid(e.to_string()))?;
    let json = serde_json::to_vec_pretty(&HistoryRecord::ImportManifest(manifest))
        .map_err(|e| ImportError::ManifestInvalid(e.to_string()))?;
    changes.push(Change::put(MANIFEST_PATH, json));
    tamper(&mut changes);

    let message = format!(
        "Import records/ from {c0}: {} files and {MANIFEST_PATH}",
        files.len()
    );
    let commit = writer.append(&message, &changes)?;
    verify_import(git, &c0, &commit)?;
    Ok(commit)
}

/// The regular files under `records/` at `c0`, as path to blob id. Refuses any other kind of entry.
fn source_files(git: &Git, c0: &str) -> Result<BTreeMap<String, String>, ImportError> {
    let mut files = BTreeMap::new();
    for entry in git.ls_tree(c0)? {
        if !entry.path.starts_with(SOURCE_DIR) {
            continue;
        }
        if entry.kind != "blob" || entry.mode != "100644" {
            return Err(ImportError::UnsupportedEntry {
                path: entry.path,
                mode: entry.mode,
                kind: entry.kind,
            });
        }
        files.insert(entry.path, entry.oid);
    }
    Ok(files)
}

/// Re-reads `commit` and fails if it is not a faithful import of `records/` at `c0`. See the module docs for
/// the list of checks, in the order they run.
pub fn verify_import(git: &Git, c0: &str, commit: &str) -> Result<(), ImportError> {
    let source = source_files(git, c0)?;

    let manifest_bytes = git
        .show_file(commit, MANIFEST_PATH)?
        .ok_or(ImportError::ManifestMissing)?;
    let text = String::from_utf8(manifest_bytes)
        .map_err(|e| ImportError::ManifestInvalid(e.to_string()))?;
    let manifest = match HistoryRecord::parse(&text) {
        Ok(HistoryRecord::ImportManifest(m)) => m,
        Ok(other) => {
            return Err(ImportError::ManifestInvalid(format!(
                "the record is {}, not an import manifest",
                other.schema()
            )));
        }
        Err(e) => return Err(ImportError::ManifestInvalid(e.to_string())),
    };
    if manifest.c0 != c0 {
        return Err(ImportError::ManifestInvalid(format!(
            "it names C0 {} but the import was of {c0}",
            manifest.c0
        )));
    }

    let imported: BTreeMap<String, String> = git
        .ls_tree(commit)?
        .into_iter()
        .filter(|e| e.path != MANIFEST_PATH)
        .map(|e| (e.path, e.oid))
        .collect();
    let by_path: BTreeMap<&str, &ImportedPath> = manifest
        .paths
        .iter()
        .map(|p| (p.path.as_str(), p))
        .collect();

    for path in source.keys() {
        if !imported.contains_key(path) {
            return Err(ImportError::Missing { path: path.clone() });
        }
    }
    for path in imported.keys() {
        if !source.contains_key(path) {
            return Err(ImportError::Extra { path: path.clone() });
        }
    }
    for (path, found) in &imported {
        let expected = &source[path];
        if found != expected {
            return Err(ImportError::BlobMismatch {
                path: path.clone(),
                expected: expected.clone(),
                found: found.clone(),
            });
        }
        let hash = sha256_hex(&git.cat_blob(found)?);
        match by_path.get(path.as_str()) {
            Some(entry) if entry.sha256 == hash => {}
            _ => return Err(ImportError::HashMismatch { path: path.clone() }),
        }
    }
    for entry in &manifest.paths {
        let Some(expected) = source.get(&entry.path) else {
            return Err(ImportError::Extra {
                path: entry.path.clone(),
            });
        };
        if &entry.blob_id != expected {
            return Err(ImportError::BlobMismatch {
                path: entry.path.clone(),
                expected: expected.clone(),
                found: entry.blob_id.clone(),
            });
        }
        if entry.sha256 != sha256_hex(&git.cat_blob(expected)?) {
            return Err(ImportError::HashMismatch {
                path: entry.path.clone(),
            });
        }
    }
    for (what, found) in [
        ("the imported commit", imported.len()),
        ("the manifest", manifest.paths.len()),
    ] {
        if found != source.len() {
            return Err(ImportError::CountMismatch {
                what,
                expected: source.len(),
                found,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_git::records_branch::RECORDS_REF;
    use std::fs;
    use std::path::Path;

    const FILES: &[(&str, &[u8])] = &[
        ("records/jobs/a/outcome.json", b"{\"a\":1}\n"),
        ("records/jobs/a/deep/er/note.txt", b"nested\n"),
        ("records/jobs/b/outcome.json", b"{\"b\":2}\n"),
        ("records/binary.bin", &[0xff, 0xfe, 0x00, 0x80, 0x01]),
        ("records/top.md", b"top\n"),
    ];

    fn write(dir: &Path, path: &str, bytes: &[u8]) {
        let full = dir.join(path);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(full, bytes).unwrap();
    }

    /// A scratch repository whose one commit holds `files` and a file outside `records/`.
    fn repo(files: &[(&str, &[u8])]) -> (tempfile::TempDir, Git, String) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        write(dir.path(), "README.md", b"outside\n");
        for (path, bytes) in files {
            write(dir.path(), path, bytes);
        }
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "c0"]).unwrap();
        let c0 = git.head().unwrap().unwrap();
        (dir, git, c0)
    }

    fn manifest_of(git: &Git, commit: &str) -> ImportManifest {
        let bytes = git.show_file(commit, MANIFEST_PATH).unwrap().unwrap();
        match HistoryRecord::parse(std::str::from_utf8(&bytes).unwrap()).unwrap() {
            HistoryRecord::ImportManifest(m) => m,
            other => panic!("not a manifest: {other:?}"),
        }
    }

    /// Rewrites the manifest in the batch.
    fn edit_manifest(changes: &mut [Change], edit: impl Fn(&mut ImportManifest)) {
        for change in changes {
            if let Change::Put { path, bytes } = change
                && path == MANIFEST_PATH
            {
                let text = std::str::from_utf8(bytes).unwrap();
                let HistoryRecord::ImportManifest(mut m) = HistoryRecord::parse(text).unwrap()
                else {
                    panic!("not a manifest");
                };
                edit(&mut m);
                *bytes = serde_json::to_vec_pretty(&HistoryRecord::ImportManifest(m)).unwrap();
            }
        }
    }

    #[test]
    fn full_import_passes_self_check_and_copies_every_file() {
        let (_dir, git, c0) = repo(FILES);
        let commit = import_records(&git, &c0).unwrap();
        assert_eq!(git.rev_parse(RECORDS_REF).unwrap().unwrap(), commit);

        let source: BTreeMap<String, String> = git
            .ls_tree(&c0)
            .unwrap()
            .into_iter()
            .filter(|e| e.path.starts_with("records/"))
            .map(|e| (e.path, e.oid))
            .collect();
        assert_eq!(source.len(), FILES.len());
        let mut imported: BTreeMap<String, String> = git
            .ls_tree(&commit)
            .unwrap()
            .into_iter()
            .map(|e| (e.path, e.oid))
            .collect();
        assert!(
            imported.remove(MANIFEST_PATH).is_some(),
            "the manifest is in the commit"
        );
        assert_eq!(imported, source, "same paths, same blob ids, nothing else");
        assert!(git.parents(&commit).unwrap().is_empty());
        verify_import(&git, &c0, &commit).unwrap();
    }

    #[test]
    fn manifest_lists_each_path_with_blob_and_sha256() {
        let (_dir, git, c0) = repo(FILES);
        let commit = import_records(&git, &c0).unwrap();
        let manifest = manifest_of(&git, &commit);
        HistoryRecord::ImportManifest(manifest.clone())
            .validate()
            .unwrap();
        assert_eq!(manifest.c0, c0);

        let source: Vec<_> = git
            .ls_tree(&c0)
            .unwrap()
            .into_iter()
            .filter(|e| e.path.starts_with("records/"))
            .collect();
        assert_eq!(manifest.paths.len(), source.len());
        for entry in source {
            let bytes = git.cat_blob(&entry.oid).unwrap();
            let listed = manifest
                .paths
                .iter()
                .find(|p| p.path == entry.path)
                .unwrap_or_else(|| panic!("{} is not listed", entry.path));
            assert_eq!(listed.blob_id, entry.oid);
            assert_eq!(listed.sha256, sha256_hex(&bytes));
        }
    }

    #[test]
    fn control_untampered_import_passes() {
        let (_dir, git, c0) = repo(FILES);
        import_with(&git, &c0, |_| {}).unwrap();
    }

    #[test]
    fn self_check_fails_on_missing_file() {
        let (_dir, git, c0) = repo(FILES);
        let err = import_with(&git, &c0, |changes| {
            changes.retain(|c| !matches!(c, Change::Put { path, .. } if path == "records/top.md"));
        })
        .unwrap_err();
        assert!(
            matches!(&err, ImportError::Missing { path } if path == "records/top.md"),
            "{err}"
        );
    }

    #[test]
    fn self_check_fails_on_changed_file() {
        let (_dir, git, c0) = repo(FILES);
        let err = import_with(&git, &c0, |changes| {
            for change in changes {
                if let Change::Put { path, bytes } = change
                    && path == "records/top.md"
                {
                    bytes.push(b'!');
                }
            }
        })
        .unwrap_err();
        assert!(
            matches!(&err, ImportError::BlobMismatch { path, .. } if path == "records/top.md"),
            "{err}"
        );
    }

    #[test]
    fn self_check_fails_on_extra_file() {
        let (_dir, git, c0) = repo(FILES);
        let err = import_with(&git, &c0, |changes| {
            changes.push(Change::put("records/extra.json", "{}"));
        })
        .unwrap_err();
        assert!(
            matches!(&err, ImportError::Extra { path } if path == "records/extra.json"),
            "{err}"
        );
    }

    #[test]
    fn self_check_fails_on_count_mismatch() {
        let (_dir, git, c0) = repo(FILES);
        let err = import_with(&git, &c0, |changes| {
            edit_manifest(changes, |m| {
                let first = m.paths[0].clone();
                m.paths.push(first);
            });
        })
        .unwrap_err();
        assert!(
            matches!(
                err,
                ImportError::CountMismatch {
                    what: "the manifest",
                    expected: 5,
                    found: 6
                }
            ),
            "{err}"
        );
    }

    #[test]
    fn self_check_fails_on_wrong_manifest_hash_or_blob() {
        let (_dir, git, c0) = repo(FILES);
        let err = import_with(&git, &c0, |changes| {
            edit_manifest(changes, |m| m.paths[0].sha256 = "0".repeat(64));
        })
        .unwrap_err();
        assert!(matches!(err, ImportError::HashMismatch { .. }), "{err}");

        let (_dir, git, c0) = repo(FILES);
        let err = import_with(&git, &c0, |changes| {
            edit_manifest(changes, |m| m.paths[0].blob_id = "1".repeat(40));
        })
        .unwrap_err();
        assert!(matches!(err, ImportError::BlobMismatch { .. }), "{err}");
    }

    #[test]
    fn self_check_fails_when_the_manifest_is_missing() {
        let (_dir, git, c0) = repo(FILES);
        let err = import_with(&git, &c0, |changes| {
            changes.retain(|c| !matches!(c, Change::Put { path, .. } if path == MANIFEST_PATH));
        })
        .unwrap_err();
        assert!(matches!(err, ImportError::ManifestMissing), "{err}");
    }

    #[test]
    fn second_import_is_refused_and_tip_unchanged() {
        let (_dir, git, c0) = repo(FILES);
        let first = import_records(&git, &c0).unwrap();
        let err = import_records(&git, &c0).unwrap_err();
        assert!(
            matches!(&err, ImportError::AlreadyImported { tip } if *tip == first),
            "{err}"
        );
        assert_eq!(git.rev_parse(RECORDS_REF).unwrap().unwrap(), first);
        assert_eq!(git.rev_list_reverse(&first).unwrap().len(), 1);
    }

    #[test]
    fn an_existing_unrelated_records_branch_blocks_the_import() {
        let (_dir, git, c0) = repo(FILES);
        let tip = RecordsWriter::new(&git)
            .append("other", &[Change::put("other.json", "{}")])
            .unwrap();
        let err = import_records(&git, &c0).unwrap_err();
        assert!(matches!(err, ImportError::AlreadyImported { .. }), "{err}");
        assert_eq!(git.rev_parse(RECORDS_REF).unwrap().unwrap(), tip);
    }

    #[test]
    fn empty_records_is_an_error_and_writes_nothing() {
        let (_dir, git, c0) = repo(&[]);
        let err = import_records(&git, &c0).unwrap_err();
        assert!(matches!(err, ImportError::EmptyRecords { .. }), "{err}");
        assert_eq!(git.rev_parse(RECORDS_REF).unwrap(), None);
    }

    #[test]
    fn a_non_commit_is_refused() {
        let (_dir, git, _c0) = repo(FILES);
        let err = import_records(&git, "0123456789abcdef0123456789abcdef01234567").unwrap_err();
        assert!(matches!(err, ImportError::NotACommit { .. }), "{err}");
        assert_eq!(git.rev_parse(RECORDS_REF).unwrap(), None);
    }

    #[test]
    fn a_symlink_under_records_is_refused() {
        let (dir, git, _c0) = repo(FILES);
        std::os::unix::fs::symlink("top.md", dir.path().join("records/link")).unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "link"]).unwrap();
        let head = git.head().unwrap().unwrap();
        let err = import_records(&git, &head).unwrap_err();
        assert!(
            matches!(&err, ImportError::UnsupportedEntry { path, .. } if path == "records/link"),
            "{err}"
        );
        assert_eq!(git.rev_parse(RECORDS_REF).unwrap(), None);
    }

    #[test]
    fn import_leaves_main_and_the_work_tree_untouched() {
        let (_dir, git, c0) = repo(FILES);
        import_records(&git, &c0).unwrap();
        assert_eq!(git.head().unwrap().unwrap(), c0);
        assert_eq!(git.current_branch().unwrap().as_deref(), Some("main"));
        assert_eq!(git.run(["status", "--porcelain"]).unwrap(), "");
    }
}
