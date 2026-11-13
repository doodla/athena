//! Prompt-surface inventory (OLY-B-S2-01): for a studied instance, list every file it cites that the
//! instance feeds to an agent as instructions.
//!
//! The inventory is a pure function over the sources a study cites, so it is reproducible: the same study
//! gives the same list, sorted by path, and rerunning it changes nothing. It names no instance; the instance
//! is whatever study is passed in.
//!
//! What counts as a prompt surface: a cited file that is an instruction document (extension `md`, `txt`,
//! `prompt`, `tmpl`) or whose file name contains `prompt`, `instruction`, `agents`, `skill` or `claude`.
//! Limit: a prompt generated at runtime by a script is listed only when a study cites a file that holds its
//! text; capturing runtime output is a run record, not this inventory.

use crate::records::{Claim, Rationale, Source, SourceKind};
use std::collections::{BTreeMap, BTreeSet};

/// One prompt surface of an instance and the claims or cards that cite it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptSurface {
    /// Repository-relative path of the file in the studied system.
    pub path: String,
    /// Ids of the claims and rationale cards citing the file, sorted and unique.
    pub cited_by: Vec<String>,
}

const TEXT_EXTENSIONS: &[&str] = &["md", "txt", "prompt", "tmpl"];
const NAME_HINTS: &[&str] = &["prompt", "instruction", "agents", "skill", "claude"];

/// Whether a cited path is a prompt surface.
pub fn is_prompt_surface(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path).to_lowercase();
    let ext = name.rsplit_once('.').map(|(_, e)| e);
    ext.is_some_and(|e| TEXT_EXTENSIONS.contains(&e)) || NAME_HINTS.iter().any(|h| name.contains(h))
}

fn add(found: &mut BTreeMap<String, BTreeSet<String>>, id: &str, src: &Source) {
    if src.kind != SourceKind::File {
        return;
    }
    if let Some(path) = src.path.as_deref().filter(|p| is_prompt_surface(p)) {
        found
            .entry(path.to_string())
            .or_default()
            .insert(id.to_string());
    }
}

/// Every prompt surface the study cites, sorted by path.
pub fn inventory(claims: &[Claim], rationales: &[Rationale]) -> Vec<PromptSurface> {
    let mut found: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for c in claims {
        for s in &c.sources {
            add(&mut found, &c.id, s);
        }
    }
    for r in rationales {
        for s in &r.sources {
            add(&mut found, &r.id, s);
        }
    }
    found
        .into_iter()
        .map(|(path, ids)| PromptSurface {
            path,
            cited_by: ids.into_iter().collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::records::Mark;

    fn claim(id: &str, sources: Vec<Source>) -> Claim {
        Claim {
            schema: "study.claim/1".into(),
            id: id.into(),
            text: "t".into(),
            mark: Mark::Read,
            sources,
            contradicts: vec![],
            mechanism: String::new(),
        }
    }

    #[test]
    fn classifies_paths() {
        assert!(is_prompt_surface("AGENTS.md"));
        assert!(is_prompt_surface("docs/guide.md"));
        assert!(is_prompt_surface("lib/system_prompt.rs"));
        assert!(!is_prompt_surface("bin/run.sh"));
        assert!(!is_prompt_surface("src/main.rs"));
    }

    #[test]
    fn lists_sorted_unique_surfaces_with_citers() {
        let sha = "a".repeat(40);
        let claims = vec![
            claim(
                "c2",
                vec![
                    Source::file("docs/b.md", 1, 2, &sha, "q"),
                    Source::file("bin/x.sh", 1, 2, &sha, "q"),
                ],
            ),
            claim("c1", vec![Source::file("docs/b.md", 3, 4, &sha, "q")]),
            claim("c3", vec![Source::file("AGENTS.md", 1, 1, &sha, "q")]),
        ];
        let inv = inventory(&claims, &[]);
        let paths: Vec<&str> = inv.iter().map(|s| s.path.as_str()).collect();
        assert_eq!(paths, ["AGENTS.md", "docs/b.md"]);
        assert_eq!(inv[1].cited_by, ["c1", "c2"]);
        assert_eq!(inv, inventory(&claims, &[]));
    }

    #[test]
    fn a_study_citing_no_instruction_file_gives_an_empty_inventory() {
        let sha = "a".repeat(40);
        let claims = vec![claim("c", vec![Source::file("bin/x.sh", 1, 2, &sha, "q")])];
        assert!(inventory(&claims, &[]).is_empty());
    }
}
