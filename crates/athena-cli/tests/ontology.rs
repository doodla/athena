//! Check of `docs/ontology.toml`, in two parts. `problems` is the structural part: every term has a definition,
//! the misread that caused it, sources that exist, and relations that point at defined terms.
//!
//! `claim_problems` is the content part: it compares every term with the sources that govern it, so a definition
//! cannot drift from them unnoticed. It is generic over all terms. For each term it scans `BRIEF.md` and the
//! term's own file sources (decision ids are not files and are not scanned) for lines that name the term or one
//! of its optional `aliases`, as a whole word (a trailing `s` or `'s` is allowed). Matching is case-sensitive.
//!
//! * `senses`: each entry is `{ in, marker, says }`. `in` is one scanned file. `marker` is a whole-word phrase
//!   that must occur on a line naming the term in that file (or on the next line, when that line ends with `:`);
//!   it must say more than the term's name, and it describes the sense rather than copying a line. `says` is a
//!   phrase the definition must contain. A marker that finds no line is stale. A definition without the `says`
//!   phrase is flagged. A term needs at least one sense that passes both ways (a checked claim).
//! * Identifying lines: a line where the term or an alias opens a bullet, heading or table row and is followed by
//!   `:` or `|` defines the term or gives it a role (as the pantheon table does). Each one must be covered by a
//!   sense marker or an excuse, so a source that names a second sense cannot be missed.
//! * `excused`: optional, for an identifying line that is not a sense. Each entry is
//!   `{ files = [one file], marker, reason }`: exactly one file, a marker (more than the term's name) and a reason.
//!   There is no file-wide excuse. An excuse that covers no identifying line is stale, and one that covers more
//!   than `MAX_EXCUSED_LINES` lines is too broad. Both are flagged.

use std::collections::{BTreeMap, BTreeSet};
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
    aliases: Vec<String>,
    #[serde(default)]
    senses: Vec<Sense>,
    #[serde(default)]
    excused: Vec<Excuse>,
}

#[derive(Deserialize)]
struct Sense {
    #[serde(rename = "in", default)]
    in_file: String,
    #[serde(default)]
    marker: String,
    #[serde(default)]
    says: String,
}

#[derive(Deserialize)]
struct Excuse {
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    marker: String,
    #[serde(default)]
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

fn is_decision_id(source: &str) -> bool {
    source.len() > 1 && source.starts_with('D') && source[1..].chars().all(|c| c.is_ascii_digit())
}

fn source_exists(root: &Path, source: &str) -> bool {
    if is_decision_id(source) {
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

/// The most identifying lines one excuse may cover.
const MAX_EXCUSED_LINES: usize = 2;

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric()
}

/// True when `text` holds `phrase` with a word boundary at each end that is a letter or digit. A phrase that
/// starts or ends with punctuation (`(jobs)`, `Athena:`) needs no boundary at that end.
fn phrase_in(text: &str, phrase: &str) -> bool {
    let (Some(first), Some(last)) = (phrase.chars().next(), phrase.chars().next_back()) else {
        return false;
    };
    text.match_indices(phrase).any(|(at, m)| {
        let before = text[..at].chars().next_back();
        let after = text[at + m.len()..].chars().next();
        (!is_word_char(first) || before.is_none_or(|c| !is_word_char(c)))
            && (!is_word_char(last) || after.is_none_or(|c| !is_word_char(c)))
    })
}

/// True when `line` names `name` as a whole word, with an optional trailing `s` or `'s`.
fn names_term(line: &str, name: &str) -> bool {
    let (Some(first), Some(last)) = (name.chars().next(), name.chars().next_back()) else {
        return false;
    };
    line.match_indices(name).any(|(at, m)| {
        let before = line[..at].chars().next_back();
        if is_word_char(first) && before.is_some_and(is_word_char) {
            return false;
        }
        if !is_word_char(last) {
            return true;
        }
        let rest = &line[at + m.len()..];
        [Some(rest), rest.strip_prefix("'s"), rest.strip_prefix('s')]
            .into_iter()
            .flatten()
            .any(|r| r.chars().next().is_none_or(|c| !is_word_char(c)))
    })
}

/// True when the term or an alias opens a bullet, heading or table row and is followed by `:` or `|`.
fn is_identifying(line: &str, name: &str) -> bool {
    let start = line.trim_start_matches(|c: char| {
        matches!(c, '*' | '#' | '>' | '-' | '|' | '.' | ')')
            || c.is_ascii_digit()
            || c.is_whitespace()
    });
    let Some(rest) = start.strip_prefix(name) else {
        return false;
    };
    let rest = rest.trim_start_matches('*').trim_start();
    rest.starts_with(':') || rest.starts_with('|')
}

/// One line of a scanned file that names the term.
struct Hit {
    number: usize,
    line: String,
    /// The line, plus the next one when the line ends with `:`.
    context: String,
    identifying: bool,
}

fn scan(text: &str, names: &[&str]) -> Vec<Hit> {
    let lines: Vec<&str> = text.lines().collect();
    let mut hits = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if !names.iter().any(|n| names_term(line, n)) {
            continue;
        }
        let context = match lines.get(i + 1) {
            Some(next) if line.trim_end().ends_with(':') => format!("{line}\n{next}"),
            _ => (*line).to_string(),
        };
        hits.push(Hit {
            number: i + 1,
            line: line.trim().to_string(),
            context,
            identifying: names.iter().any(|n| is_identifying(line, n)),
        });
    }
    hits
}

/// `BRIEF.md` plus the term's sources that are files. Decision ids are not scanned.
fn scanned_files(term: &Term, root: &Path) -> Vec<String> {
    let mut files = vec!["BRIEF.md".to_string()];
    for source in &term.sources {
        if !is_decision_id(source) && root.join(source).is_file() && !files.contains(source) {
            files.push(source.clone());
        }
    }
    files
}

fn names_of(term: &Term) -> Vec<&str> {
    std::iter::once(term.name.as_str())
        .chain(term.aliases.iter().map(String::as_str))
        .collect()
}

/// The hits of `term` in each scanned file, and the problems met reading them.
fn term_hits(term: &Term, root: &Path) -> (BTreeMap<String, Vec<Hit>>, Vec<String>) {
    let names = names_of(term);
    let mut hits = BTreeMap::new();
    let mut found = Vec::new();
    for file in scanned_files(term, root) {
        match std::fs::read_to_string(root.join(&file)) {
            Ok(text) => {
                hits.insert(file, scan(&text, &names));
            }
            Err(e) => found.push(format!("{}: cannot read {file}: {e}", term.name)),
        }
    }
    (hits, found)
}

/// True when the definition holds `says`, ignoring where its lines break.
fn definition_says(term: &Term, says: &str) -> bool {
    flat(&term.definition).contains(&flat(says))
}

/// `text` with every run of whitespace, line breaks included, as one space.
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// How many of the term's senses have a marker found in a source and a `says` phrase in the definition.
fn checked_senses(term: &Term, root: &Path) -> usize {
    let (hits, _) = term_hits(term, root);
    let names = names_of(term);
    term.senses
        .iter()
        .filter(|s| {
            let marker = s.marker.trim();
            !marker.is_empty()
                && !s.says.trim().is_empty()
                && !names.contains(&marker)
                && definition_says(term, s.says.trim())
                && hits
                    .get(&s.in_file)
                    .is_some_and(|hs| hs.iter().any(|h| phrase_in(&h.context, marker)))
        })
        .count()
}

/// Every problem found comparing each term with the sources that govern it. Empty means each definition agrees
/// with what the sources say about the term. See the header of this file for the rule.
fn claim_problems(file: &File, root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for term in &file.terms {
        let name = &term.name;
        let names = names_of(term);
        let (hits, read_problems) = term_hits(term, root);
        found.extend(read_problems);

        for sense in &term.senses {
            let marker = sense.marker.trim();
            let says = sense.says.trim();
            if marker.is_empty() || says.is_empty() {
                found.push(format!(
                    "{name}: a sense in {:?} needs a marker and a says phrase",
                    sense.in_file
                ));
                continue;
            }
            if names.contains(&marker) {
                found.push(format!(
                    "{name}: sense marker {marker:?} is only the term's name"
                ));
                continue;
            }
            let Some(file_hits) = hits.get(&sense.in_file) else {
                found.push(format!(
                    "{name}: sense file {:?} is not BRIEF.md or one of the term's file sources",
                    sense.in_file
                ));
                continue;
            };
            if !definition_says(term, says) {
                found.push(format!(
                    "{name}: the definition does not say {says:?}, which its source says (marker {marker:?} in {})",
                    sense.in_file
                ));
            }
            if !file_hits.iter().any(|h| phrase_in(&h.context, marker)) {
                found.push(format!(
                    "{name}: sense marker {marker:?} matches no line naming the term in {} (stale)",
                    sense.in_file
                ));
            }
        }
        if checked_senses(term, root) == 0 {
            found.push(format!(
                "{name}: no checked claim: no sense has both a marker found in a source and a says phrase in the definition"
            ));
        }

        let excuse_ok = |e: &Excuse| {
            let marker = e.marker.trim();
            e.files.len() == 1
                && !marker.is_empty()
                && !names.contains(&marker)
                && !e.reason.trim().is_empty()
        };
        for excuse in &term.excused {
            let marker = excuse.marker.trim();
            if excuse.files.len() != 1 {
                found.push(format!(
                    "{name}: excuse {marker:?} must name exactly one file, not {}",
                    excuse.files.len()
                ));
            }
            if marker.is_empty() {
                found.push(format!("{name}: an excuse has no marker"));
            } else if names.contains(&marker) {
                found.push(format!(
                    "{name}: excuse marker {marker:?} is only the term's name"
                ));
            }
            if excuse.reason.trim().is_empty() {
                found.push(format!("{name}: excuse {marker:?} has no reason"));
            }
            if !excuse_ok(excuse) {
                continue;
            }
            let covered = hits.get(&excuse.files[0]).map_or(0, |hs| {
                hs.iter()
                    .filter(|h| h.identifying && phrase_in(&h.context, marker))
                    .count()
            });
            if !hits.contains_key(&excuse.files[0]) {
                found.push(format!(
                    "{name}: excuse {marker:?} names {:?}, which is not BRIEF.md or one of the term's file sources",
                    excuse.files[0]
                ));
            } else if covered == 0 {
                found.push(format!(
                    "{name}: excuse {marker:?} matches no identifying line in {} (stale)",
                    excuse.files[0]
                ));
            } else if covered > MAX_EXCUSED_LINES {
                found.push(format!(
                    "{name}: excuse {marker:?} covers {covered} lines in {}, more than {MAX_EXCUSED_LINES}",
                    excuse.files[0]
                ));
            }
        }

        for (path, file_hits) in &hits {
            for hit in file_hits.iter().filter(|h| h.identifying) {
                let by_sense = term.senses.iter().any(|s| {
                    s.in_file == *path
                        && !s.marker.trim().is_empty()
                        && phrase_in(&hit.context, s.marker.trim())
                });
                let by_excuse = term.excused.iter().any(|e| {
                    excuse_ok(e) && e.files[0] == *path && phrase_in(&hit.context, e.marker.trim())
                });
                if !by_sense && !by_excuse {
                    found.push(format!(
                        "{name}: {path}:{} names the term as a definition or role and no sense or excuse covers it: {}",
                        hit.number, hit.line
                    ));
                }
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
    let found = claim_problems(&parse_real(), &repo_root());
    assert!(found.is_empty(), "ontology claim problems: {found:#?}");
}

fn parse_real() -> File {
    toml::from_str(&real_text()).expect("docs/ontology.toml parses")
}

const SEVEN: [&str; 7] = [
    "Hermes",
    "planning",
    "job",
    "Athena",
    "product versus system",
    "landed",
    "Sri",
];

fn term_mut<'a>(file: &'a mut File, name: &str) -> &'a mut Term {
    file.terms
        .iter_mut()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("term {name} is defined"))
}

#[test]
fn the_real_file_claims_are_checked_against_the_sources() {
    let file = parse_real();
    let root = repo_root();
    let found = claim_problems(&file, &root);
    assert!(found.is_empty(), "ontology claim problems: {found:#?}");
    for name in SEVEN {
        let t = term(&file, name);
        assert!(checked_senses(t, &root) >= 1, "{name} has no checked claim");
        let (hits, _) = term_hits(t, &root);
        assert!(
            hits.values().any(|hs| !hs.is_empty()),
            "{name} is named in no scanned file"
        );
        assert!(
            hits.contains_key("BRIEF.md"),
            "{name}: BRIEF.md is always scanned"
        );
    }
}

#[test]
fn altered_definition_is_flagged_for_each_seed_term() {
    let root = repo_root();
    for name in SEVEN {
        let senses = term(&parse_real(), name).senses.len();
        assert!(senses >= 1, "{name} has senses");
        for i in 0..senses {
            let mut file = parse_real();
            let t = term_mut(&mut file, name);
            let says = t.senses[i].says.clone();
            t.definition = flat(&t.definition).replace(&flat(&says), "");
            let found = claim_problems(&file, &root);
            assert!(
                found.iter().any(
                    |p| p.starts_with(&format!("{name}: the definition does not say {says:?}"))
                ),
                "{name} without {says:?} is not flagged: {found:?}"
            );
        }
    }
}

#[test]
fn a_sense_whose_marker_is_gone_from_the_source_is_flagged_for_each_seed_term() {
    let root = repo_root();
    for name in SEVEN {
        let mut file = parse_real();
        let t = term_mut(&mut file, name);
        t.senses[0].marker = "a phrase no source holds".to_string();
        let found = claim_problems(&file, &root);
        assert!(
            found
                .iter()
                .any(|p| p.starts_with(&format!("{name}: sense marker")) && p.contains("(stale)")),
            "{name}: {found:?}"
        );
    }
}

#[test]
fn hermes_with_only_the_tool_sense_is_flagged_until_both_senses_are_listed() {
    let root = repo_root();
    let mut file = parse_real();
    let hermes = term_mut(&mut file, "Hermes");
    let role = hermes
        .senses
        .iter()
        .position(|s| s.marker == "Orchestration")
        .expect("the role sense is listed");
    let kept = hermes.senses.remove(role);
    let found = claim_problems(&file, &root);
    assert!(
        found
            .iter()
            .any(|p| p.starts_with("Hermes: BRIEF.md:") && p.contains("| Hermes | Orchestration |")),
        "a tool-only Hermes entry passes while the source names a role: {found:?}"
    );

    term_mut(&mut file, "Hermes").senses.push(kept);
    let found = claim_problems(&file, &root);
    assert!(found.is_empty(), "both senses listed: {found:?}");
}

#[test]
fn athena_needs_its_role_and_its_program_sense() {
    let root = repo_root();
    for (marker, line) in [
        (
            "Strategy and learning",
            "| Athena | Strategy and learning |",
        ),
        (
            "orchestration system you build",
            "Athena: the orchestration system",
        ),
    ] {
        let mut file = parse_real();
        term_mut(&mut file, "Athena")
            .senses
            .retain(|s| s.marker != marker);
        let found = claim_problems(&file, &root);
        assert!(
            found
                .iter()
                .any(|p| p.starts_with("Athena: BRIEF.md:") && p.contains(line)),
            "dropping {marker:?}: {found:?}"
        );
    }
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
    std::fs::write(
        dir.path().join("BRIEF.md"),
        "* a: first one, marked alpha\n* b: second one, marked beta\nnothing else here\n",
    )
    .unwrap();
    dir
}

/// A scratch repository whose BRIEF.md also holds `extra`, which the terms of `GOOD` do not cover.
fn scratch_root_with(extra: &str) -> tempfile::TempDir {
    let dir = scratch_root();
    let path = dir.path().join("BRIEF.md");
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str(extra);
    std::fs::write(path, text).unwrap();
    dir
}

fn claims(text: &str, root: &Path) -> Vec<String> {
    let file: File = toml::from_str(text).expect("fixture parses");
    claim_problems(&file, root)
}

/// `GOOD` with an `excused` field on term `a`.
fn good_with_excuse(excused: &str) -> String {
    GOOD.replacen(
        "relations = [{ kind = \"k\", target = \"b\" }]",
        &format!("relations = [{{ kind = \"k\", target = \"b\" }}]\nexcused = [{excused}]"),
        1,
    )
}

const GOOD: &str = r#"
[about]
rule = "r"
read_by = "t"

[[term]]
name = "a"
definition = "the alpha thing"
misread = "m"
sources = ["BRIEF.md", "D01"]
relations = [{ kind = "k", target = "b" }]
senses = [{ in = "BRIEF.md", marker = "marked alpha", says = "alpha" }]

[[term]]
name = "b"
definition = "the beta thing"
misread = "m"
sources = ["BRIEF.md"]
relations = []
senses = [{ in = "BRIEF.md", marker = "marked beta", says = "beta" }]
"#;

const ODD_LINE: &str = "* a: an odd line, marked gamma\n";
const GOOD_EXCUSE: &str = "{ files = [\"BRIEF.md\"], marker = \"marked gamma\", reason = \"wraps a different sentence\" }";

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

#[test]
fn a_sound_fixture_has_no_claim_problems() {
    let dir = scratch_root();
    assert_eq!(claims(GOOD, dir.path()), Vec::<String>::new());
}

#[test]
fn a_well_formed_excuse_covers_an_identifying_line_that_is_not_a_sense() {
    let dir = scratch_root_with(ODD_LINE);
    let found = claims(GOOD, dir.path());
    assert!(
        found
            .iter()
            .any(|p| p.contains("BRIEF.md:4") && p.contains("covers it")),
        "an uncovered identifying line is flagged: {found:?}"
    );
    let found = claims(&good_with_excuse(GOOD_EXCUSE), dir.path());
    assert_eq!(found, Vec::<String>::new());
}

#[test]
fn an_excuse_without_a_reason_is_flagged() {
    let dir = scratch_root_with(ODD_LINE);
    let text =
        good_with_excuse("{ files = [\"BRIEF.md\"], marker = \"marked gamma\", reason = \" \" }");
    let found = claims(&text, dir.path());
    assert!(
        found.iter().any(|p| p.contains("has no reason")),
        "{found:?}"
    );
    assert!(
        found.iter().any(|p| p.contains("covers it")),
        "a reasonless excuse does not cover the line: {found:?}"
    );
}

#[test]
fn an_excuse_without_a_marker_is_flagged() {
    let dir = scratch_root_with(ODD_LINE);
    let text = good_with_excuse("{ files = [\"BRIEF.md\"], reason = \"whole file\" }");
    let found = claims(&text, dir.path());
    assert!(
        found.iter().any(|p| p.contains("has no marker")),
        "{found:?}"
    );
    assert!(
        found.iter().any(|p| p.contains("covers it")),
        "a markerless excuse does not cover the line: {found:?}"
    );
}

#[test]
fn an_excuse_covering_more_than_one_file_is_flagged() {
    let dir = scratch_root_with(ODD_LINE);
    let text = good_with_excuse(
        "{ files = [\"BRIEF.md\", \"docs/decisions.md\"], marker = \"marked gamma\", reason = \"r\" }",
    );
    let found = claims(&text, dir.path());
    assert!(
        found.iter().any(|p| p.contains("exactly one file, not 2")),
        "{found:?}"
    );
    let none = good_with_excuse("{ marker = \"marked gamma\", reason = \"r\" }");
    let found = claims(&none, dir.path());
    assert!(
        found.iter().any(|p| p.contains("exactly one file, not 0")),
        "{found:?}"
    );
}

#[test]
fn a_stale_excuse_is_flagged() {
    let dir = scratch_root_with(ODD_LINE);
    let text = good_with_excuse(
        "{ files = [\"BRIEF.md\"], marker = \"marked gamma\", reason = \"r\" }, { files = [\"BRIEF.md\"], marker = \"marked nowhere\", reason = \"r\" }",
    );
    let found = claims(&text, dir.path());
    assert!(
        found
            .iter()
            .any(|p| p.contains("\"marked nowhere\"") && p.contains("(stale)")),
        "{found:?}"
    );
    assert_eq!(
        found.len(),
        1,
        "only the stale excuse is flagged: {found:?}"
    );
}

#[test]
fn an_overbroad_excuse_is_flagged() {
    let dir = scratch_root_with(
        "* a: an odd line, one\n* a: an odd line, two\n* a: an odd line, three\n",
    );
    let text =
        good_with_excuse("{ files = [\"BRIEF.md\"], marker = \"an odd line\", reason = \"r\" }");
    let found = claims(&text, dir.path());
    assert!(
        found.iter().any(|p| p.contains("covers 3 lines")),
        "{found:?}"
    );
}

#[test]
fn an_excuse_marker_that_is_only_the_term_name_is_flagged() {
    let dir = scratch_root_with(ODD_LINE);
    let text = good_with_excuse("{ files = [\"BRIEF.md\"], marker = \"a\", reason = \"r\" }");
    let found = claims(&text, dir.path());
    assert!(
        found.iter().any(|p| p.contains("only the term's name")),
        "{found:?}"
    );
}

#[test]
fn a_term_without_a_checked_claim_is_flagged() {
    let dir = scratch_root();
    let mut file: File = toml::from_str(GOOD).unwrap();
    file.terms[1].senses.clear();
    let found = claim_problems(&file, dir.path());
    assert!(
        found.iter().any(|p| p.starts_with("b: no checked claim")),
        "{found:?}"
    );
    assert!(
        !found.iter().any(|p| p.starts_with("a: ")),
        "term a still has its claim: {found:?}"
    );
}

#[test]
fn a_sense_that_does_not_pass_both_ways_is_not_a_checked_claim() {
    let dir = scratch_root();
    // The marker is in the source, but the definition does not say it.
    let text = GOOD.replacen("says = \"beta\"", "says = \"gamma\"", 1);
    let found = claims(&text, dir.path());
    assert!(
        found
            .iter()
            .any(|p| p.starts_with("b: the definition does not say \"gamma\"")),
        "{found:?}"
    );
    assert!(
        found.iter().any(|p| p.starts_with("b: no checked claim")),
        "{found:?}"
    );
}

#[test]
fn a_sense_in_a_file_the_term_does_not_scan_is_flagged() {
    let dir = scratch_root();
    let text = GOOD.replacen(
        "in = \"BRIEF.md\", marker = \"marked beta\"",
        "in = \"docs/decisions.md\", marker = \"marked beta\"",
        1,
    );
    let found = claims(&text, dir.path());
    assert!(
        found.iter().any(|p| p.starts_with("b: sense file")),
        "{found:?}"
    );
}

#[test]
fn a_sense_marker_equal_to_the_term_name_or_empty_is_flagged() {
    let dir = scratch_root();
    let text = GOOD.replacen("marker = \"marked alpha\"", "marker = \"a\"", 1);
    let found = claims(&text, dir.path());
    assert!(
        found
            .iter()
            .any(|p| p.starts_with("a: sense marker \"a\" is only the term's name")),
        "{found:?}"
    );
    let text = GOOD.replacen("marker = \"marked alpha\"", "marker = \"\"", 1);
    let found = claims(&text, dir.path());
    assert!(
        found
            .iter()
            .any(|p| p.starts_with("a: a sense in") && p.contains("needs a marker")),
        "{found:?}"
    );
}

#[test]
fn a_marker_that_is_only_part_of_a_word_is_stale() {
    let dir = scratch_root();
    let text = GOOD.replacen("marker = \"marked alpha\"", "marker = \"marked alph\"", 1);
    let found = claims(&text, dir.path());
    assert!(
        found
            .iter()
            .any(|p| p.starts_with("a: sense marker") && p.contains("(stale)")),
        "{found:?}"
    );
}

#[test]
fn markers_match_whole_words_only() {
    assert!(!phrase_in("concatenate", "cat"));
    assert!(phrase_in("a cat sat", "cat"));
    assert!(phrase_in("a cat", "cat"));
    assert!(phrase_in("units (jobs) here", "units (jobs)"));
    assert!(phrase_in("`merged-unverified`, never", "merged-unverified"));
    assert!(phrase_in("Athena: the system", "Athena:"));
    assert!(!phrase_in("anything", ""));

    assert!(names_term("units (jobs)", "job"));
    assert!(names_term("Sri's words", "Sri"));
    assert!(names_term("Athena: x", "Athena"));
    assert!(names_term("| Hermes | Orchestration |", "Hermes"));
    assert!(!names_term("jobber", "job"));
    assert!(!names_term("a subjob", "job"));
    assert!(!names_term("athena", "Athena"));
    assert!(!names_term("Athenaeum", "Athena"));
    assert!(!names_term("jobs-queue", "jobx"));
}

#[test]
fn identifying_lines_are_bullets_headings_and_table_rows() {
    for line in [
        "* Athena: the orchestration system",
        "   * Athena:",
        "- Athena: x",
        "### Athena: x",
        "| Athena | Strategy and learning |",
        "1. Athena: x",
        "**Athena**: x",
        "> Athena | x",
    ] {
        assert!(is_identifying(line, "Athena"), "{line}");
    }
    for line in [
        "Athena builds a deep understanding",
        "* The study of Athena: a thing",
        "* Athena builds: a thing",
        "Athenas: x",
        "athena: x",
    ] {
        assert!(!is_identifying(line, "Athena"), "{line}");
    }
}
