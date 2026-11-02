//! Check of `docs/ontology.toml`. Structure: every term has a definition, the misread that caused it, sources
//! that exist, and relations that point at defined terms. Content: every line of BRIEF.md and of the term's own
//! source files that names the term (whole word or phrase, ignoring case) must be accounted for, either by a
//! `[[term.senses]]` entry (a `label` stated in the definition and `markers` found in the line) or by a
//! `[[term.excused]]` entry (a `reason`, anchored by a `marker` and/or a `source` file). Otherwise a term with
//! several meanings could be defined from one source while another names a second meaning (job
//! 20261017-173508-0wtc, Hermes as a studied agent and as the Orchestration god). Decision-id sources are not
//! scanned. See the header of that file for how it is read.

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
    #[serde(default)]
    senses: Vec<Sense>,
    #[serde(default)]
    excused: Vec<Excuse>,
}

/// One meaning of a term. `label` must appear in the definition; a source line that names the term and contains
/// any of `markers` (case-insensitive) is accounted for by this sense.
#[derive(Deserialize)]
struct Sense {
    label: String,
    markers: Vec<String>,
}

/// A recorded reason why source lines naming the term are not a further sense. It applies to a line that
/// contains `marker` (if given) in the file `source` (if given); at least one of the two is required.
#[derive(Deserialize)]
struct Excuse {
    marker: Option<String>,
    source: Option<String>,
    reason: String,
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

/// True when `name` occurs in `line` as a whole word or phrase, ignoring case.
fn names_term(line: &str, name: &str) -> bool {
    let (line, name) = (line.to_lowercase(), name.to_lowercase());
    line.match_indices(&name).any(|(at, _)| {
        let before = line[..at].chars().next_back();
        let after = line[at + name.len()..].chars().next();
        !before.is_some_and(char::is_alphanumeric) && !after.is_some_and(char::is_alphanumeric)
    })
}

/// The governing files a term is checked against: BRIEF.md plus each of the term's own sources that is a file.
fn governing_files(term: &Term) -> Vec<&str> {
    let mut files = vec!["BRIEF.md"];
    for source in &term.sources {
        let file = source.as_str();
        if !files.contains(&file) && file.contains('.') {
            files.push(file);
        }
    }
    files
}

/// Content check: every line of a governing file that names the term must belong to a listed sense or carry a
/// recorded excuse. This is what catches a term defined from one source while another names a second meaning.
fn sense_problems(term: &Term, root: &Path) -> Vec<String> {
    let name = &term.name;
    let mut found = Vec::new();
    let definition = term.definition.to_lowercase();
    for sense in &term.senses {
        if sense.markers.iter().all(|m| m.trim().is_empty()) {
            found.push(format!("{name}: sense {:?} has no markers", sense.label));
        }
        if sense.label.trim().is_empty() || !definition.contains(&sense.label.to_lowercase()) {
            found.push(format!(
                "{name}: sense {:?} is not stated in the definition",
                sense.label
            ));
        }
    }
    for excuse in &term.excused {
        let anchored = excuse.marker.as_ref().is_some_and(|m| !m.trim().is_empty())
            || excuse.source.as_ref().is_some_and(|s| !s.trim().is_empty());
        if !anchored || excuse.reason.trim().is_empty() {
            found.push(format!(
                "{name}: an excused hit needs a marker or source, and a reason"
            ));
        }
    }
    for file in governing_files(term) {
        let Ok(text) = std::fs::read_to_string(root.join(file)) else {
            continue;
        };
        for (index, line) in text.lines().enumerate() {
            if !names_term(line, name) {
                continue;
            }
            let lower = line.to_lowercase();
            let sense = term.senses.iter().any(|s| {
                s.markers
                    .iter()
                    .any(|m| !m.trim().is_empty() && lower.contains(&m.to_lowercase()))
            });
            let excused = term.excused.iter().any(|e| {
                e.source.as_deref().is_none_or(|s| s == file)
                    && e.marker
                        .as_deref()
                        .is_none_or(|m| lower.contains(&m.to_lowercase()))
                    && (e.marker.is_some() || e.source.is_some())
                    && !e.reason.trim().is_empty()
            });
            if !sense && !excused {
                found.push(format!(
                    "{name}: {file}:{} names the term outside its listed senses; add a sense or an excused \
                     hit with a reason: {}",
                    index + 1,
                    line.trim()
                ));
            }
        }
    }
    found
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
        found.extend(sense_problems(term, root));
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

/// The Hermes case in miniature: the entry defines the tool and the role, the scratch source names both.
fn hermes_case(senses: &str) -> (tempfile::TempDir, String) {
    let dir = scratch_root();
    std::fs::write(
        dir.path().join("BRIEF.md"),
        "Hermes Agent is a studied agent.\nHermes is the Orchestration god.\nNo mention here.\n",
    )
    .unwrap();
    let text = format!(
        "[about]\nrule = \"r\"\nread_by = \"t\"\n\n[[term]]\nname = \"Hermes\"\n\
         definition = \"A Hermes Agent, or the Orchestration god.\"\nmisread = \"m\"\n\
         sources = [\"BRIEF.md\"]\nrelations = []\n{senses}\n"
    );
    (dir, text)
}

const TOOL: &str = "{ label = \"Hermes Agent\", markers = [\"Hermes Agent\"] }";
const ROLE: &str = "{ label = \"Orchestration god\", markers = [\"Orchestration\"] }";

#[test]
fn a_sense_found_in_a_source_but_not_on_the_entry_is_flagged() {
    let (dir, text) = hermes_case(&format!("senses = [{TOOL}]"));
    let found = problems(&text, dir.path());
    assert!(
        found
            .iter()
            .any(|p| p.contains("BRIEF.md:2") && p.contains("outside its listed senses")),
        "{found:?}"
    );
    assert_eq!(
        found.len(),
        1,
        "only the role line is unaccounted: {found:?}"
    );
}

#[test]
fn an_entry_listing_both_senses_passes() {
    let (dir, text) = hermes_case(&format!("senses = [{TOOL}, {ROLE}]"));
    assert_eq!(problems(&text, dir.path()), Vec::<String>::new());
}

#[test]
fn an_excused_hit_with_a_reason_passes_and_without_one_fails() {
    let excuse = |reason: &str| {
        format!(
            "senses = [{TOOL}]\nexcused = [{{ marker = \"Orchestration\", reason = \"{reason}\" }}]"
        )
    };
    let (dir, text) = hermes_case(&excuse("a quoted example, not a sense"));
    assert_eq!(problems(&text, dir.path()), Vec::<String>::new());
    let (dir, text) = hermes_case(&excuse(""));
    let found = problems(&text, dir.path());
    assert!(found.iter().any(|p| p.contains("needs a marker or source")));
    assert!(found.iter().any(|p| p.contains("BRIEF.md:2")), "{found:?}");
}

#[test]
fn a_sense_not_stated_in_the_definition_is_flagged() {
    let senses =
        format!("senses = [{TOOL}, {ROLE}, {{ label = \"a lost cause\", markers = [\"x\"] }}]");
    let (dir, text) = hermes_case(&senses);
    let found = problems(&text, dir.path());
    assert!(
        found
            .iter()
            .any(|p| p.contains("not stated in the definition")),
        "{found:?}"
    );
}

#[test]
fn term_names_match_whole_words_only() {
    assert!(names_term("Hermes's provider", "Hermes"));
    assert!(names_term("a HERMES agent", "Hermes"));
    assert!(!names_term("Hermesian", "Hermes"));
    assert!(names_term(
        "the product versus system split",
        "product versus system"
    ));
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
