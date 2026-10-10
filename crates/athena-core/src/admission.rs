//! Admission: what a candidate may change (design §7.3, D38, D44).
//!
//! The core refuses a change if any of the following hold:
//!
//! - a path is under `records/`, or is marked `athena-exclude` or `athena-readonly` under the base's rules;
//! - a submodule or gitlink appears;
//! - a file exceeds the size limit;
//! - an added line contains an email address outside the allow-list;
//! - the candidate would change how any path is classified;
//! - a governance file changed without the accepted reading listing it as a deliverable and naming it in
//!   an acceptance criterion. For a `Cargo.toml`, only its `[lints]` and `[workspace.lints]` tables are
//!   governance content.

use anyhow::Result;
use athena_data::config::PersonalData;
use athena_data::records::{ChangeStatus, PolicyViolation, Reading};
use athena_gate::content::disallowed_emails;
use athena_git::Git;
use athena_git::attrs::{PathClass, classification_changes, classify};
use std::collections::BTreeSet;

/// Root files that govern how Athena works in this repository.
const ROOT_GOVERNANCE: [&str; 4] = [
    "athena.toml",
    "defaults.toml",
    "setup.sh",
    "rust-toolchain.toml",
];

/// File names that configure the check tools, wherever they appear.
const TOOL_CONFIG_NAMES: [&str; 4] = [
    "rustfmt.toml",
    ".rustfmt.toml",
    "clippy.toml",
    ".clippy.toml",
];

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Whether `path` configures a check tool (cargo, rustfmt, clippy).
pub fn is_tool_config(path: &str) -> bool {
    TOOL_CONFIG_NAMES.contains(&file_name(path))
        || path.ends_with(".cargo/config.toml")
        || path.ends_with(".cargo/config")
}

/// Whether `path` is a governance file (whole-file); `Cargo.toml` lints tables are handled separately.
pub fn is_governance(path: &str) -> bool {
    ROOT_GOVERNANCE.contains(&path)
        || file_name(path) == ".gitattributes"
        || (path.starts_with(".github/workflows/") && !path.ends_with('/'))
        || is_tool_config(path)
}

/// Whether `path` is a Cargo manifest.
pub fn is_manifest(path: &str) -> bool {
    file_name(path) == "Cargo.toml"
}

/// The `[lints]` and `[workspace.lints]` tables of a manifest, normalized for comparison.
pub fn lints_tables(manifest: &str) -> Option<String> {
    let doc: toml_edit::DocumentMut = manifest.parse().ok()?;
    let lints = doc.get("lints").map(|i| i.to_string()).unwrap_or_default();
    let workspace = doc
        .get("workspace")
        .and_then(|w| w.get("lints"))
        .map(|i| i.to_string())
        .unwrap_or_default();
    Some(format!("{lints}\n--\n{workspace}"))
}

/// One changed path of the candidate, with what admission needs to know.
#[derive(Debug, Clone)]
pub struct Changed {
    pub path: String,
    pub status: ChangeStatus,
    pub mode: String,
    pub size: u64,
}

/// Everything admission reads.
#[derive(Debug)]
pub struct AdmissionInput<'a> {
    pub git: &'a Git,
    /// The base commit P (its rules judge).
    pub base: &'a str,
    /// The candidate commit C, already built.
    pub candidate: &'a str,
    pub changes: &'a [Changed],
    pub reading: &'a Reading,
    pub max_file_bytes: u64,
    pub personal_data: &'a PersonalData,
}

fn violation(rule: &str, path: Option<&str>, detail: String) -> PolicyViolation {
    PolicyViolation {
        rule: rule.into(),
        path: path.map(str::to_string),
        detail,
    }
}

/// Whether the reading lists `path` as a deliverable and names it in an acceptance criterion.
pub fn governance_change_is_listed(reading: &Reading, path: &str) -> bool {
    let listed = reading.deliverables.iter().any(|d| d.path == path);
    let named = reading
        .acceptance_criteria
        .iter()
        .any(|c| c.text.contains(path) || c.text.contains(file_name(path)));
    listed && named
}

/// The core's admission rules. Returns every violation found.
pub fn admit(input: &AdmissionInput<'_>) -> Result<Vec<PolicyViolation>> {
    let git = input.git;
    let mut violations = Vec::new();
    let paths: Vec<String> = input.changes.iter().map(|c| c.path.clone()).collect();
    let classes = classify(git, input.base, &paths)?;
    for change in input.changes {
        let path = change.path.as_str();
        match classes.get(path).copied().unwrap_or(PathClass::Governed) {
            PathClass::Records => violations.push(violation(
                "records",
                Some(path),
                format!("{path} is under records/, which only Athena writes"),
            )),
            PathClass::Excluded => violations.push(violation(
                "owner-private",
                Some(path),
                format!("{path} is private to the repository owner"),
            )),
            PathClass::Readonly => violations.push(violation(
                "owner-maintained",
                Some(path),
                format!("{path} is the owner's own document; no job may change it"),
            )),
            PathClass::Governed => {}
        }
        if change.mode == "160000" {
            violations.push(violation(
                "gitlink",
                Some(path),
                format!("{path} is a submodule or gitlink"),
            ));
        }
        if change.size > input.max_file_bytes {
            violations.push(violation(
                "size",
                Some(path),
                format!(
                    "{path} is {} bytes, over the limit of {}",
                    change.size, input.max_file_bytes
                ),
            ));
        }
    }

    // Added lines with private email addresses.
    let mut flagged = BTreeSet::new();
    for (path, line) in git.added_lines(input.base, input.candidate)? {
        if !disallowed_emails(&line, input.personal_data).is_empty() && flagged.insert(path.clone())
        {
            violations.push(violation(
                "personal-data",
                Some(&path),
                format!("{path} adds an email address outside the allow-list"),
            ));
        }
    }

    // Classification stability: no path may change class between the base's rules and the candidate's.
    let mut all: BTreeSet<String> = git
        .ls_tree(input.base)?
        .into_iter()
        .map(|e| e.path)
        .collect();
    all.extend(git.ls_tree(input.candidate)?.into_iter().map(|e| e.path));
    let all: Vec<String> = all.into_iter().collect();
    let moved = classification_changes(git, input.base, input.candidate, &all)?;
    if !moved.is_empty() {
        violations.push(violation(
            "classification",
            moved.first().map(String::as_str),
            format!("the candidate changes how {} path(s) are classified (athena-* declarations), starting with {}", moved.len(), moved[0]),
        ));
    }

    // Governance files.
    for change in input.changes {
        let path = change.path.as_str();
        let governance = if is_manifest(path) {
            let before = git
                .show_file(input.base, path)?
                .map(|b| String::from_utf8_lossy(&b).into_owned());
            let after = git
                .show_file(input.candidate, path)?
                .map(|b| String::from_utf8_lossy(&b).into_owned());
            let lints = |text: &Option<String>| {
                text.as_deref()
                    .and_then(lints_tables)
                    .unwrap_or_else(|| "\n--\n".into())
            };
            lints(&before) != lints(&after)
        } else {
            is_governance(path)
        };
        if governance && !governance_change_is_listed(input.reading, path) {
            violations.push(violation(
                "governance",
                Some(path),
                format!(
                    "{path} is a governance file{}; a job may change it only when the accepted reading lists it as a deliverable and names it in an acceptance criterion",
                    if is_manifest(path) { " (its [lints] tables)" } else { "" }
                ),
            ));
        }
    }
    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn governance_files_are_recognized() {
        for path in [
            "athena.toml",
            "defaults.toml",
            ".gitattributes",
            "crates/x/.gitattributes",
            ".github/workflows/ci.yml",
            "setup.sh",
            "rust-toolchain.toml",
            ".cargo/config.toml",
            "crates/x/.cargo/config",
            "rustfmt.toml",
            ".rustfmt.toml",
            "clippy.toml",
            "sub/.clippy.toml",
        ] {
            assert!(is_governance(path), "{path}");
        }
        for path in [
            "README.md",
            "crates/x/Cargo.toml",
            "docs/athena.toml",
            "src/setup.sh",
        ] {
            assert!(!is_governance(path), "{path}");
        }
    }

    #[test]
    fn lints_tables_are_compared_alone() {
        let a = "[package]\nname = \"a\"\n\n[lints]\nworkspace = true\n";
        let b = "[package]\nname = \"a\"\nversion = \"2\"\n\n[lints]\nworkspace = true\n";
        let c = "[package]\nname = \"a\"\n\n[lints.rust]\nunsafe_code = \"allow\"\n";
        assert_eq!(lints_tables(a), lints_tables(b));
        assert_ne!(lints_tables(a), lints_tables(c));
    }
}
