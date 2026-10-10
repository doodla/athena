//! Check-tool configuration is judged from the base commit (design §7.2, D38; M12).
//!
//! A file the candidate controls could switch a check off: a `.cargo/config.toml` runner that skips tests,
//! a `[lints]` table that allows everything, or a `rustfmt.toml` that disables formatting. So before the
//! checks run, the verifier's working tree gets the base's version of every check-tool configuration
//! file, and every manifest's `[lints]` and `[workspace.lints]` tables come from the base too. A manifest
//! new in the candidate gets `[lints] workspace = true` when the base workspace defines lints. The
//! candidate's own versions stay in the snapshot's candidate commit, where `git diff HEAD~1` shows them.

use crate::admission::{is_manifest, is_tool_config};
use anyhow::Result;
use athena_git::snapshot::Snapshot;
use std::collections::BTreeSet;

fn set_executable(path: &std::path::Path, mode: &str) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perm = if mode == "100755" { 0o755 } else { 0o644 };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(perm))?;
    Ok(())
}

/// Replaces the lints tables of `candidate` with those of `base` (or removes them when `base` has none).
pub fn lints_from_base(
    candidate: &str,
    base: Option<&str>,
    workspace_has_lints: bool,
) -> Result<String> {
    let mut doc: toml_edit::DocumentMut = candidate.parse()?;
    match base {
        Some(base) => {
            let base: toml_edit::DocumentMut = base.parse()?;
            match base.get("lints") {
                Some(item) => {
                    doc.insert("lints", item.clone());
                }
                None => {
                    doc.remove("lints");
                }
            }
            let base_ws = base.get("workspace").and_then(|w| w.get("lints")).cloned();
            if let Some(workspace) = doc.get_mut("workspace").and_then(|w| w.as_table_like_mut()) {
                match base_ws {
                    Some(item) => {
                        workspace.insert("lints", item);
                    }
                    None => {
                        workspace.remove("lints");
                    }
                }
            } else if let Some(item) = base_ws {
                let mut table = toml_edit::Table::new();
                table.insert("lints", item);
                doc.insert("workspace", toml_edit::Item::Table(table));
            }
        }
        None => {
            doc.remove("lints");
            if let Some(workspace) = doc.get_mut("workspace").and_then(|w| w.as_table_like_mut()) {
                workspace.remove("lints");
            }
            if workspace_has_lints && doc.get("package").is_some() {
                let mut table = toml_edit::Table::new();
                table.insert("workspace", toml_edit::value(true));
                doc.insert("lints", toml_edit::Item::Table(table));
            }
        }
    }
    Ok(doc.to_string())
}

/// Puts the base's tool configuration into the verifier's working tree. Returns the paths whose working
/// copy now differs from the candidate's.
pub fn apply_base_tool_config(snapshot: &Snapshot) -> Result<Vec<String>> {
    let git = snapshot.git();
    let base = snapshot
        .base()
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("the verifier's snapshot has no base commit"))?;
    let candidate = snapshot
        .candidate()
        .map(str::to_string)
        .ok_or_else(|| anyhow::anyhow!("the verifier's snapshot has no candidate"))?;
    let base_entries = git.ls_tree(&base)?;
    let candidate_entries = git.ls_tree(&candidate)?;
    let mut changed = Vec::new();

    let mut tool_paths: BTreeSet<&str> = BTreeSet::new();
    tool_paths.extend(
        base_entries
            .iter()
            .filter(|e| is_tool_config(&e.path))
            .map(|e| e.path.as_str()),
    );
    tool_paths.extend(
        candidate_entries
            .iter()
            .filter(|e| is_tool_config(&e.path))
            .map(|e| e.path.as_str()),
    );
    for path in tool_paths {
        let in_base = base_entries.iter().find(|e| e.path == path);
        let in_candidate = candidate_entries.iter().find(|e| e.path == path);
        if in_base.map(|e| (&e.oid, &e.mode)) == in_candidate.map(|e| (&e.oid, &e.mode)) {
            continue;
        }
        let full = snapshot.dir().join(path);
        match in_base {
            Some(entry) => {
                if let Some(parent) = full.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(&full, git.cat_blob(&entry.oid)?)?;
                set_executable(&full, &entry.mode)?;
            }
            None => {
                let _ = std::fs::remove_file(&full);
            }
        }
        changed.push(path.to_string());
    }

    let workspace_has_lints = git
        .show_file(&base, "Cargo.toml")?
        .and_then(|b| {
            String::from_utf8_lossy(&b)
                .parse::<toml_edit::DocumentMut>()
                .ok()
        })
        .is_some_and(|doc| doc.get("workspace").and_then(|w| w.get("lints")).is_some());
    for entry in candidate_entries.iter().filter(|e| is_manifest(&e.path)) {
        let candidate_text = String::from_utf8_lossy(&git.cat_blob(&entry.oid)?).into_owned();
        let base_text = git
            .show_file(&base, &entry.path)?
            .map(|b| String::from_utf8_lossy(&b).into_owned());
        let Ok(merged) =
            lints_from_base(&candidate_text, base_text.as_deref(), workspace_has_lints)
        else {
            continue;
        };
        if crate::admission::lints_tables(&merged)
            != crate::admission::lints_tables(&candidate_text)
        {
            std::fs::write(snapshot.dir().join(&entry.path), merged)?;
            changed.push(entry.path.clone());
        }
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_relaxed_lints_table_is_replaced_by_the_base() {
        let base = "[package]\nname = \"a\"\n\n[lints]\nworkspace = true\n";
        let candidate =
            "[package]\nname = \"a\"\nversion = \"0.2.0\"\n\n[lints.rust]\nwarnings = \"allow\"\n";
        let merged = lints_from_base(candidate, Some(base), true).unwrap();
        assert!(
            merged.contains("version = \"0.2.0\""),
            "the rest of the manifest is the candidate's"
        );
        assert!(merged.contains("workspace = true"));
        assert!(!merged.contains("warnings"));
    }

    #[test]
    fn a_new_manifest_inherits_the_workspace_lints() {
        let candidate = "[package]\nname = \"new\"\n\n[lints.clippy]\nall = \"allow\"\n";
        let merged = lints_from_base(candidate, None, true).unwrap();
        assert!(!merged.contains("all = \"allow\""));
        assert!(merged.contains("[lints]\nworkspace = true"));
    }

    #[test]
    fn workspace_lints_come_from_the_base() {
        let base = "[workspace]\nmembers = []\n\n[workspace.lints.rust]\nunsafe_code = \"deny\"\n";
        let candidate =
            "[workspace]\nmembers = [\"x\"]\n\n[workspace.lints.rust]\nunsafe_code = \"allow\"\n";
        let merged = lints_from_base(candidate, Some(base), true).unwrap();
        assert!(merged.contains("unsafe_code = \"deny\""));
        assert!(merged.contains("members = [\"x\"]"));
    }
}
