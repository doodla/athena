//! Path classification from committed rules, and the content fingerprint (design §4.2, §7.3).
//!
//! A repository owner declares paths in committed `.gitattributes` files:
//!
//! - `athena-exclude`: private to the owner; absent from every agent snapshot.
//! - `athena-readonly`: the owner's own document; agents read it, no job may change it.
//!
//! Rules are evaluated with `check-attr --source=<commit>`, with the per-user attributes file pointed at
//! `/dev/null` and system attributes off, so only committed rules count. `$GIT_DIR/info/attributes` cannot
//! be disabled by any option, so Athena refuses to run when it mentions an `athena-*` attribute (E14).

use crate::{Git, GitError, Result};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::ffi::OsStr;

pub const EXCLUDE: &str = "athena-exclude";
pub const READONLY: &str = "athena-readonly";
pub const RECORDS_PREFIX: &str = "records/";

/// How Athena treats a path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PathClass {
    /// Athena's governed content: what jobs change and fingerprints cover.
    Governed,
    /// Under `records/`: append-only evidence.
    Records,
    /// `athena-exclude`: private to the owner.
    Excluded,
    /// `athena-readonly`: the owner's document, readable by agents.
    Readonly,
}

impl PathClass {
    /// Owner paths: changes to them alone need no trailer and are never a job's to make.
    pub fn is_owner(self) -> bool {
        matches!(self, PathClass::Excluded | PathClass::Readonly)
    }
    /// Whether agents may see the path in a snapshot.
    pub fn visible_to_agents(self) -> bool {
        matches!(self, PathClass::Governed | PathClass::Readonly)
    }
}

/// Classifies `paths` by the rules committed in `rules_commit`. Paths need not exist in that commit.
pub fn classify(
    git: &Git,
    rules_commit: &str,
    paths: &[String],
) -> Result<BTreeMap<String, PathClass>> {
    refuse_info_attributes(git)?;
    let mut classes = BTreeMap::new();
    let mut to_check = Vec::new();
    for path in paths {
        if path.starts_with(RECORDS_PREFIX) {
            classes.insert(path.clone(), PathClass::Records);
        } else {
            to_check.push(path.clone());
        }
    }
    if to_check.is_empty() {
        return Ok(classes);
    }
    let mut input = Vec::new();
    for path in &to_check {
        input.extend_from_slice(path.as_bytes());
        input.push(0);
    }
    let rules = git
        .clone()
        .with_config("core.attributesFile", "/dev/null")
        .with_env("GIT_ATTR_NOSYSTEM", "1");
    let out = rules.run_bytes(
        [
            "check-attr",
            &format!("--source={rules_commit}"),
            "-z",
            "--stdin",
            EXCLUDE,
            READONLY,
        ],
        Some(&input),
        &[] as &[(&str, &OsStr)],
    )?;
    let fields: Vec<String> = out
        .split(|b| *b == 0)
        .map(|c| String::from_utf8_lossy(c).into_owned())
        .collect();
    let mut excluded = std::collections::BTreeSet::new();
    let mut readonly = std::collections::BTreeSet::new();
    for triple in fields.chunks(3) {
        if let [path, attr, value] = triple
            && is_set(value)
        {
            if attr == EXCLUDE {
                excluded.insert(path.clone());
            } else if attr == READONLY {
                readonly.insert(path.clone());
            }
        }
    }
    for path in to_check {
        let class = if excluded.contains(&path) {
            PathClass::Excluded
        } else if readonly.contains(&path) {
            PathClass::Readonly
        } else {
            PathClass::Governed
        };
        classes.insert(path, class);
    }
    Ok(classes)
}

fn is_set(value: &str) -> bool {
    !matches!(value, "unspecified" | "unset" | "false" | "")
}

/// Refuses to classify while `info/attributes` mentions an Athena attribute, since no option disables it.
pub fn refuse_info_attributes(git: &Git) -> Result<()> {
    let path = git.common_dir()?.join("info").join("attributes");
    if let Ok(text) = std::fs::read_to_string(&path)
        && text.contains("athena-")
    {
        return Err(GitError::Refused(format!(
            "{} declares athena-* attributes; Athena reads these only from committed .gitattributes files. \
             Move the declarations into a committed .gitattributes and remove them from {}",
            path.display(),
            path.display()
        )));
    }
    Ok(())
}

/// The governed-content fingerprint of `treeish` under the rules committed in `rules_commit`.
///
/// It is the SHA-256 of the `ls-tree -r -z` entries of governed paths: everything except `records/` and
/// owner paths. Equal fingerprints mean the governed content is byte-identical.
pub fn fingerprint(git: &Git, treeish: &str, rules_commit: &str) -> Result<String> {
    let entries = git.ls_tree(treeish)?;
    let paths: Vec<String> = entries.iter().map(|e| e.path.clone()).collect();
    let classes = classify(git, rules_commit, &paths)?;
    let mut hasher = Sha256::new();
    for entry in &entries {
        if classes.get(&entry.path) == Some(&PathClass::Governed) {
            hasher.update(
                format!(
                    "{} {} {}\t{}",
                    entry.mode, entry.kind, entry.oid, entry.path
                )
                .as_bytes(),
            );
            hasher.update([0u8]);
        }
    }
    let digest = hasher.finalize();
    Ok(format!(
        "sha256:{}",
        digest
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    ))
}

/// Paths whose class differs between the rules of two commits.
pub fn classification_changes(
    git: &Git,
    rules_a: &str,
    rules_b: &str,
    paths: &[String],
) -> Result<Vec<String>> {
    let a = classify(git, rules_a, paths)?;
    let b = classify(git, rules_b, paths)?;
    Ok(paths
        .iter()
        .filter(|p| a.get(*p) != b.get(*p))
        .cloned()
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    fn owner_repo() -> (tempfile::TempDir, Git) {
        repo_with(&[
            (
                ".gitattributes",
                "/FRONT.md athena-exclude\n/SPEC.md athena-readonly\n",
            ),
            ("private/.gitattributes", "* athena-exclude\n"),
            ("private/notes.md", "owner notes"),
            ("FRONT.md", "front door"),
            ("SPEC.md", "the spec"),
            ("records/jobs/x.json", "{}"),
            ("src/main.rs", "fn main() {}\n"),
        ])
    }

    #[test]
    fn committed_rules_classify_paths() {
        let (_dir, git) = owner_repo();
        let head = git.head().unwrap().unwrap();
        let paths: Vec<String> = [
            "private/notes.md",
            "private/new-file.md",
            "FRONT.md",
            "SPEC.md",
            "records/jobs/x.json",
            "src/main.rs",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        let classes = classify(&git, &head, &paths).unwrap();
        assert_eq!(classes["private/notes.md"], PathClass::Excluded);
        assert_eq!(
            classes["private/new-file.md"],
            PathClass::Excluded,
            "rules apply to paths that do not exist yet"
        );
        assert_eq!(classes["FRONT.md"], PathClass::Excluded);
        assert_eq!(classes["SPEC.md"], PathClass::Readonly);
        assert_eq!(classes["records/jobs/x.json"], PathClass::Records);
        assert_eq!(classes["src/main.rs"], PathClass::Governed);
    }

    #[test]
    fn per_user_attributes_do_not_count() {
        // M12: a file outside the repository cannot mark governed paths as excluded.
        let (dir, git) = owner_repo();
        let home = dir.path().join("fakehome");
        std::fs::create_dir_all(home.join(".config/git")).unwrap();
        std::fs::write(
            home.join(".config/git/attributes"),
            "src/main.rs athena-exclude\n",
        )
        .unwrap();
        let git = git
            .with_env("HOME", home.to_str().unwrap())
            .with_env("XDG_CONFIG_HOME", home.join(".config").to_str().unwrap());
        let head = git.head().unwrap().unwrap();
        let classes = classify(&git, &head, &["src/main.rs".to_string()]).unwrap();
        assert_eq!(classes["src/main.rs"], PathClass::Governed);
    }

    #[test]
    fn info_attributes_naming_athena_attributes_are_refused() {
        // M12 (blocks): info/attributes cannot be disabled, so Athena refuses to run.
        let (dir, git) = owner_repo();
        let head = git.head().unwrap().unwrap();
        let info = dir.path().join(".git/info");
        std::fs::create_dir_all(&info).unwrap();
        std::fs::write(info.join("attributes"), "/src/main.rs athena-exclude\n").unwrap();
        let err = classify(&git, &head, &["src/main.rs".to_string()]).unwrap_err();
        assert!(matches!(err, GitError::Refused(_)));
    }

    #[test]
    fn fingerprint_ignores_records_and_owner_paths() {
        let (dir, git) = owner_repo();
        let first = git.head().unwrap().unwrap();
        let fp1 = fingerprint(&git, &first, &first).unwrap();
        write_files(
            dir.path(),
            &[
                ("records/jobs/y.json", "{}"),
                ("private/more.md", "x"),
                ("SPEC.md", "edited spec"),
            ],
        );
        let second = commit_all(&git, "owner and records changes");
        assert_eq!(fingerprint(&git, &second, &first).unwrap(), fp1);
        write_files(
            dir.path(),
            &[("src/main.rs", "fn main() { println!(); }\n")],
        );
        let third = commit_all(&git, "code change");
        assert_ne!(fingerprint(&git, &third, &first).unwrap(), fp1);
    }

    #[test]
    fn classification_changes_are_detected() {
        let (dir, git) = owner_repo();
        let first = git.head().unwrap().unwrap();
        write_files(
            dir.path(),
            &[(
                ".gitattributes",
                "/FRONT.md athena-exclude\n/SPEC.md athena-readonly\n/src/** athena-exclude\n",
            )],
        );
        let second = commit_all(&git, "exclude src");
        let changed = classification_changes(
            &git,
            &first,
            &second,
            &["src/main.rs".into(), "SPEC.md".into()],
        )
        .unwrap();
        assert_eq!(changed, vec!["src/main.rs".to_string()]);
    }
}
