//! Structural check of `docs/ontology.toml`: every term has a definition, the misread that caused it, sources
//! that exist, and relations that point at defined terms. See the header of that file for how it is read.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Deserialize)]
struct File {
    about: About,
    #[serde(rename = "term")]
    terms: Vec<Term>,
}

#[derive(Deserialize)]
struct About {
    rule: String,
    read_by: String,
}

#[derive(Deserialize)]
struct Term {
    name: String,
    definition: String,
    misread: String,
    sources: Vec<String>,
    relations: Vec<Relation>,
    #[serde(default)]
    not_part_of: Vec<NotPartOf>,
}

#[derive(Deserialize)]
struct Relation {
    kind: String,
    target: String,
}

#[derive(Deserialize)]
struct NotPartOf {
    what: String,
    source: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn source_exists(root: &Path, source: &str) -> bool {
    let is_decision_id = source.len() > 1
        && source.starts_with('D')
        && source[1..].chars().all(|c| c.is_ascii_digit());
    if is_decision_id {
        let heading = format!("## {source} ");
        return std::fs::read_to_string(root.join("docs/decisions.md"))
            .map(|text| text.lines().any(|line| line.starts_with(&heading)))
            .unwrap_or(false);
    }
    !source.is_empty() && root.join(source).exists()
}

/// Every problem found in `text`, read against the repository at `root`. Empty means the file is sound.
fn problems(text: &str, root: &Path) -> Vec<String> {
    let file: File = match toml::from_str(text) {
        Ok(file) => file,
        Err(e) => return vec![format!("does not parse: {e}")],
    };
    let mut found = Vec::new();
    if file.about.rule.trim().is_empty() || file.about.read_by.trim().is_empty() {
        found.push("[about] needs a rule and read_by".to_string());
    }
    let mut names = BTreeSet::new();
    for term in &file.terms {
        if !names.insert(term.name.as_str()) {
            found.push(format!("duplicate term {:?}", term.name));
        }
    }
    for term in &file.terms {
        let name = &term.name;
        if term.definition.trim().is_empty() {
            found.push(format!("{name}: empty definition"));
        }
        if term.misread.trim().is_empty() {
            found.push(format!("{name}: empty misread"));
        }
        if term.sources.is_empty() {
            found.push(format!("{name}: no sources"));
        }
        for source in &term.sources {
            if !source_exists(root, source) {
                found.push(format!("{name}: source {source:?} does not exist"));
            }
        }
        for relation in &term.relations {
            if relation.kind.trim().is_empty() {
                found.push(format!("{name}: relation without a kind"));
            }
            if !names.contains(relation.target.as_str()) {
                found.push(format!(
                    "{name}: relation target {:?} is not a defined term",
                    relation.target
                ));
            }
        }
        for claim in &term.not_part_of {
            if claim.what.trim().is_empty() || !source_exists(root, &claim.source) {
                found.push(format!(
                    "{name}: not-part-of claim {:?} has no existing source",
                    claim.what
                ));
            }
        }
    }
    found
}

fn real_text() -> String {
    std::fs::read_to_string(repo_root().join("docs/ontology.toml")).expect("docs/ontology.toml")
}

fn term<'a>(file: &'a File, name: &str) -> &'a Term {
    file.terms
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("term {name} is defined"))
}

#[test]
fn the_ontology_file_is_sound() {
    let found = problems(&real_text(), &repo_root());
    assert!(found.is_empty(), "ontology problems: {found:#?}");
}

#[test]
fn the_ontology_defines_the_seven_seeded_terms_with_their_meaning() {
    let file: File = toml::from_str(&real_text()).unwrap();
    let names: BTreeSet<&str> = file.terms.iter().map(|t| t.name.as_str()).collect();
    let want = [
        "planning",
        "job",
        "Athena",
        "Hermes",
        "product versus system",
        "landed",
        "Sri",
    ];
    assert_eq!(names, BTreeSet::from(want));

    let hermes = term(&file, "Hermes");
    assert!(hermes.definition.contains("Hermes Agent"));
    assert!(hermes.definition.contains("Orchestration god"));
    assert!(
        hermes
            .definition
            .contains("Only this second sense is part of Olympus")
    );
    assert!(hermes.not_part_of.len() >= 2);

    assert!(
        term(&file, "landed")
            .definition
            .contains("Merged on main through a verified pull request")
    );
    let sri = term(&file, "Sri");
    assert!(sri.definition.contains("never he or she"));
    assert!(sri.misread.contains("20261017-173504-91ys"));

    let athena = term(&file, "Athena");
    assert!(athena.definition.contains("strategy and learning role"));
    assert!(athena.definition.contains("not the whole system"));
    assert!(
        athena
            .relations
            .iter()
            .any(|r| r.target == "product versus system")
    );
    assert!(
        term(&file, "Sri")
            .relations
            .iter()
            .any(|r| r.kind == "owns")
    );
}

fn scratch_root() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("docs")).unwrap();
    std::fs::write(dir.path().join("docs/decisions.md"), "## D01 · One\n").unwrap();
    std::fs::write(dir.path().join("BRIEF.md"), "brief\n").unwrap();
    dir
}

const GOOD: &str = r#"
[about]
rule = "r"
read_by = "t"

[[term]]
name = "a"
definition = "d"
misread = "m"
sources = ["BRIEF.md", "D01"]
relations = [{ kind = "k", target = "b" }]

[[term]]
name = "b"
definition = "d"
misread = "m"
sources = ["BRIEF.md"]
relations = []
"#;

#[test]
fn a_sound_file_has_no_problems() {
    let dir = scratch_root();
    assert_eq!(problems(GOOD, dir.path()), Vec::<String>::new());
}

#[test]
fn a_term_without_a_sources_field_is_flagged() {
    let dir = scratch_root();
    let text = GOOD.replacen("sources = [\"BRIEF.md\", \"D01\"]\n", "", 1);
    let found = problems(&text, dir.path());
    assert!(
        found.iter().any(|p| p.contains("does not parse")),
        "{found:?}"
    );
}

#[test]
fn a_term_with_empty_sources_is_flagged() {
    let dir = scratch_root();
    let text = GOOD.replacen("sources = [\"BRIEF.md\", \"D01\"]", "sources = []", 1);
    let found = problems(&text, dir.path());
    assert!(
        found.iter().any(|p| p.contains("a: no sources")),
        "{found:?}"
    );
}

#[test]
fn a_source_that_does_not_exist_is_flagged() {
    let dir = scratch_root();
    for bad in ["docs/missing.md", "D99"] {
        let text = GOOD.replacen("\"D01\"", &format!("\"{bad}\""), 1);
        let found = problems(&text, dir.path());
        assert!(found.iter().any(|p| p.contains(bad)), "{bad}: {found:?}");
    }
}

#[test]
fn a_dangling_relation_target_is_flagged() {
    let dir = scratch_root();
    let text = GOOD.replacen("target = \"b\"", "target = \"nowhere\"", 1);
    let found = problems(&text, dir.path());
    assert!(found.iter().any(|p| p.contains("nowhere")), "{found:?}");
}

#[test]
fn a_duplicate_term_is_flagged() {
    let dir = scratch_root();
    let text = GOOD.replacen("name = \"b\"", "name = \"a\"", 1);
    let found = problems(&text, dir.path());
    assert!(
        found.iter().any(|p| p.contains("duplicate term")),
        "{found:?}"
    );
}

#[test]
fn a_not_part_of_claim_needs_an_existing_source() {
    let dir = scratch_root();
    let text = GOOD.replacen(
        "relations = []",
        "relations = []\nnot_part_of = [{ what = \"x\", source = \"nope.md\" }]",
        1,
    );
    let found = problems(&text, dir.path());
    assert!(found.iter().any(|p| p.contains("not-part-of")), "{found:?}");
}
