//! The registry of hand-kept mirrors: files that summarize or copy a governing source and are kept in step with
//! it by hand.
//!
//! **Root cause this module fixes.** A file kept by hand as a mirror of a governing source (the shard timings of
//! the test files, the decision log against its base text, the CI cache key against `BUILD_PATHS`, the ontology
//! against the documents it scans) was tested for its structure only, or checked late. Content that contradicted
//! the source passed every earlier check and was found only at the final verdict, in job after job.
//!
//! **The fix.** [`REGISTRY`] is the one place a mirror is declared: its file, its source and the comparison
//! rule. A rule reads both sides and compares their content, never only that a file exists, and its message
//! names the file and the fix. [`check_all`] runs the registry. It is called from the producer loop before
//! verification (`run::producer`), where a drift is sent back once, and from the `athena-cli` test target
//! `mirrors`, which runs every entry over the real files under `cargo test`. A new entry with an existing
//! [`Rule`] is covered by that test with no other edit; a new [`Rule`] must get a fixture there, or it does not
//! compile.
//!
//! Registering a new mirror is prose-only: nothing finds a hand-kept file that has no entry. The registry
//! makes the comparison a declared unit, so a mirror that is registered cannot drift unseen.
//!
//! The ontology walk (which governing lines name a term, and whether a sense or an excuse covers each) lives
//! here as plain functions over plain data, so the registry and `crates/athena-cli/tests/ontology.rs` share
//! one copy instead of keeping two.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};

use toml_edit::{DocumentMut, Item, TableLike, Value};

/// The file of shard timings.
const TIMINGS_FILE: &str = "xtask/shard-timings.txt";

/// What the mirror is compared with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// One file.
    Path(&'static str),
    /// Every file matching one of these patterns (`*` matches within one path segment).
    Globs(&'static [&'static str]),
    /// The mirror's own text at the base commit.
    GitHead,
    /// The documents the ontology scans: `BRIEF.md`, `README.md`, every `docs/*.md` and each term's file sources.
    Docs,
}

/// How the mirror is compared with its source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    /// Every `--test` and `--lib` unit on disk has a line in the timings file, and every such line has a unit.
    TimingUnits,
    /// The base text is an exact byte prefix of the new text.
    AppendOnly,
    /// The CI `athena-bin` cache key and `BUILD_PATHS` cover each other, both ways.
    HashFilesCover,
    /// Every line in a governing document that names a term as a definition or role is covered by a sense or an
    /// excuse of that term.
    OntologyTerms,
}

/// One declared mirror.
#[derive(Debug, Clone, Copy)]
pub struct Mirror {
    pub name: &'static str,
    /// The hand-kept file, relative to the repository root.
    pub mirror: &'static str,
    pub source: Source,
    pub rule: Rule,
}

/// A mirror that disagrees with its source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Drift {
    /// The mirror file.
    pub mirror: String,
    /// What disagrees, naming the file and the fix.
    pub message: String,
}

impl fmt::Display for Drift {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

/// The patterns of the test files and library roots the timings file lists.
const TEST_UNIT_GLOBS: &[&str] = &[
    "crates/*/tests/*.rs",
    "crates/*/tests/*/main.rs",
    "crates/*/src/lib.rs",
    "xtask/tests/*.rs",
    "xtask/tests/*/main.rs",
    "xtask/src/lib.rs",
];

/// Every hand-kept mirror. Add an entry here when a new file is kept in step with a source by hand.
pub const REGISTRY: &[Mirror] = &[
    Mirror {
        name: "shard timings",
        mirror: TIMINGS_FILE,
        source: Source::Globs(TEST_UNIT_GLOBS),
        rule: Rule::TimingUnits,
    },
    Mirror {
        name: "decision log",
        mirror: "docs/decisions.md",
        source: Source::GitHead,
        rule: Rule::AppendOnly,
    },
    Mirror {
        name: "build paths",
        mirror: "crates/athena-core/src/status.rs",
        source: Source::Path(".github/workflows/ci.yml"),
        rule: Rule::HashFilesCover,
    },
    Mirror {
        name: "ontology",
        mirror: "docs/ontology.toml",
        source: Source::Docs,
        rule: Rule::OntologyTerms,
    },
];

/// Reads a file at the base commit; None when there is no base or the file is not there.
pub type BaseFn<'a> = &'a dyn Fn(&str) -> Option<Vec<u8>>;

// ---------------------------------------------------------------------------------------------------------
// Rule: timing units

/// The `--lib` and `--test` units on disk, as `<package> --lib` and `<package> --test <name>`: each
/// `crates/*/src/lib.rs`, and each `crates/*/tests/*.rs` and `crates/*/tests/*/main.rs`, with the same under
/// `xtask/`. `--doc` and `--bin` units are not derived: a binary can be renamed and a doc test depends on a
/// flag, so the heavier test in `xtask/tests/ci_shards.rs` keeps them.
pub fn test_units(root: &Path) -> BTreeSet<String> {
    let mut packages: Vec<(String, PathBuf)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root.join("crates")) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                packages.push((
                    entry.file_name().to_string_lossy().into_owned(),
                    entry.path(),
                ));
            }
        }
    }
    if root.join("xtask").is_dir() {
        packages.push(("xtask".to_string(), root.join("xtask")));
    }
    let mut units = BTreeSet::new();
    for (package, dir) in packages {
        if dir.join("src/lib.rs").is_file() {
            units.insert(format!("{package} --lib"));
        }
        let Ok(entries) = std::fs::read_dir(dir.join("tests")) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let stem = if path.is_file() {
                name.strip_suffix(".rs").map(str::to_string)
            } else if path.join("main.rs").is_file() {
                Some(name)
            } else {
                None
            };
            if let Some(stem) = stem {
                units.insert(format!("{package} --test {stem}"));
            }
        }
    }
    units
}

/// The findings of comparing `units` with the `--test` and `--lib` lines of the timings file text: a unit with
/// no line, and a line with no unit. `--doc` and `--bin` lines are never reported.
pub fn missing_timing_lines(units: &BTreeSet<String>, timings_text: &str) -> Vec<String> {
    let mut listed: BTreeMap<String, usize> = BTreeMap::new();
    for (i, raw) in timings_text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((unit, _seconds)) = line.rsplit_once(char::is_whitespace) else {
            continue;
        };
        let key = unit.split_whitespace().collect::<Vec<_>>().join(" ");
        if key.contains(" --test ") || key.ends_with(" --lib") {
            listed.insert(key, i + 1);
        }
    }
    let mut found = Vec::new();
    for unit in units.iter().filter(|u| !listed.contains_key(*u)) {
        found.push(format!(
            "{TIMINGS_FILE} has no line for the test unit `{unit}`; add the line `{unit} 5.00` to {TIMINGS_FILE} \
             (the seconds are an estimate; `cargo run -p xtask -- shard-timings` measures them)"
        ));
    }
    for (key, line) in &listed {
        if !units.contains(key) {
            found.push(format!(
                "{TIMINGS_FILE} line {line} (`{key}`) names a test unit that is not on disk; remove the line from {TIMINGS_FILE}"
            ));
        }
    }
    found
}

// ---------------------------------------------------------------------------------------------------------
// Rule: append only

/// None when `base` is an exact byte prefix of `new`; else the finding, naming `docs/decisions.md`, the first
/// line of the base text that changed and the fix.
pub fn append_only_problem(base: &[u8], new: &[u8]) -> Option<String> {
    if new.starts_with(base) {
        return None;
    }
    let at = base.iter().zip(new).take_while(|(a, b)| a == b).count();
    let line = base[..at].iter().filter(|&&b| b == b'\n').count() + 1;
    let start = base[..at]
        .iter()
        .rposition(|&b| b == b'\n')
        .map_or(0, |i| i + 1);
    let end = base[at..]
        .iter()
        .position(|&b| b == b'\n')
        .map_or(base.len(), |i| at + i);
    let text: String = String::from_utf8_lossy(&base[start..end])
        .chars()
        .take(80)
        .collect();
    Some(format!(
        "docs/decisions.md is append-only, but line {line} of its base version is changed or removed (`{text}`); \
         put the old text back and add the new decision after the last line"
    ))
}

// ---------------------------------------------------------------------------------------------------------
// Rule: the CI cache key and BUILD_PATHS

/// The quoted strings of the `pub const BUILD_PATHS` array in the text of `status.rs`: after the `=` (so the
/// array type is skipped) up to the next `;`.
pub fn status_build_paths(status_text: &str) -> Result<Vec<String>, String> {
    let at = status_text
        .find("pub const BUILD_PATHS")
        .ok_or("no `pub const BUILD_PATHS` found in crates/athena-core/src/status.rs")?;
    let rest = &status_text[at..];
    let eq = rest
        .find('=')
        .ok_or("`BUILD_PATHS` in crates/athena-core/src/status.rs has no `=`")?;
    let body = &rest[eq + 1..];
    let end = body
        .find(';')
        .ok_or("`BUILD_PATHS` in crates/athena-core/src/status.rs has no closing `;`")?;
    let paths = quoted(&body[..end], '"');
    if paths.is_empty() {
        return Err("`BUILD_PATHS` in crates/athena-core/src/status.rs lists no paths".to_string());
    }
    Ok(paths)
}

/// The strings between pairs of `quote` characters in `text`.
fn quoted(text: &str, quote: char) -> Vec<String> {
    text.split(quote)
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect()
}

/// The globs inside the `hashFiles(...)` of every `athena-bin-` cache key line, with the 1-based line number.
fn key_lines(ci_text: &str) -> Vec<(usize, Vec<String>)> {
    let mut keys = Vec::new();
    for (i, line) in ci_text.lines().enumerate() {
        if !line.contains("athena-bin-") {
            continue;
        }
        let Some(at) = line.find("hashFiles(") else {
            continue;
        };
        let args = &line[at + "hashFiles(".len()..];
        let args = args.split(')').next().unwrap_or("");
        keys.push((i + 1, quoted(args, '\'')));
    }
    keys
}

/// Whether a hashFiles `glob` is covered by a BUILD_PATHS `root`: a directory root covers a glob under it, a
/// file root its exact name.
fn root_covers(root: &str, glob: &str) -> bool {
    if root.ends_with('/') {
        glob.starts_with(root)
    } else {
        glob == root
    }
}

/// Whether some `glob` hashes everything of a BUILD_PATHS `root`: a directory root needs a glob under it that
/// ends in `**`, a file root its exact name.
fn glob_hashes(root: &str, glob: &str) -> bool {
    if root.ends_with('/') {
        glob.starts_with(root) && glob.ends_with("**")
    } else {
        glob == root
    }
}

/// The findings of comparing the `athena-bin` cache keys of the CI workflow text with the `roots` of
/// `BUILD_PATHS`: a hashed glob no root covers, a root no glob hashes, and a workflow with no such key.
pub fn hash_glob_problems(ci_text: &str, roots: &[String]) -> Vec<String> {
    let keys = key_lines(ci_text);
    if keys.is_empty() {
        return vec![
            "no `athena-bin-` cache key with hashFiles(...) found in .github/workflows/ci.yml; the key must hash \
             every path in BUILD_PATHS in crates/athena-core/src/status.rs"
                .to_string(),
        ];
    }
    let mut found = Vec::new();
    for (line, globs) in &keys {
        for glob in globs {
            if !roots.iter().any(|r| root_covers(r, glob)) {
                found.push(format!(
                    ".github/workflows/ci.yml line {line}: the athena-bin key hashes `{glob}`, which is not in BUILD_PATHS; \
                     add the path to BUILD_PATHS in crates/athena-core/src/status.rs"
                ));
            }
        }
        for root in roots {
            if !globs.iter().any(|g| glob_hashes(root, g)) {
                found.push(format!(
                    "BUILD_PATHS lists `{root}`, but the athena-bin key at .github/workflows/ci.yml line {line} does not hash it; \
                     add the glob to every athena-bin key in .github/workflows/ci.yml, or remove the path from BUILD_PATHS in \
                     crates/athena-core/src/status.rs"
                ));
            }
        }
    }
    found
}

// ---------------------------------------------------------------------------------------------------------
// Rule: ontology terms. The walk below is shared with crates/athena-cli/tests/ontology.rs.

/// A sense of a term: a marker found in one scanned file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologySense {
    pub in_file: String,
    pub marker: String,
}

/// An excuse for an identifying line that is not a sense.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyExcuse {
    pub files: Vec<String>,
    pub marker: String,
    pub reason: String,
}

/// The parts of an ontology term the content walk reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OntologyTerm {
    pub name: String,
    pub aliases: Vec<String>,
    pub sources: Vec<String>,
    pub senses: Vec<OntologySense>,
    pub excused: Vec<OntologyExcuse>,
}

pub fn is_word_char(c: char) -> bool {
    c.is_alphanumeric()
}

/// True when `text` holds `phrase` with a word boundary at each end that is a letter or digit. A phrase that
/// starts or ends with punctuation (`(jobs)`, `Athena:`) needs no boundary at that end.
pub fn phrase_in(text: &str, phrase: &str) -> bool {
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
pub fn names_term(line: &str, name: &str) -> bool {
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
pub fn is_identifying(line: &str, name: &str) -> bool {
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
#[derive(Debug, Clone)]
pub struct Hit {
    pub number: usize,
    pub line: String,
    /// The line, plus the next one when the line ends with `:`.
    pub context: String,
    pub identifying: bool,
}

/// The lines of `text` that name any of `names`.
pub fn scan(text: &str, names: &[&str]) -> Vec<Hit> {
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

/// A source like `D105` is a decision id, not a file.
pub fn is_decision_id(source: &str) -> bool {
    source.len() > 1 && source.starts_with('D') && source[1..].chars().all(|c| c.is_ascii_digit())
}

/// The term's name followed by its aliases.
pub fn names_of<'a>(name: &'a str, aliases: &'a [String]) -> Vec<&'a str> {
    std::iter::once(name)
        .chain(aliases.iter().map(String::as_str))
        .collect()
}

/// Every governing document: `BRIEF.md`, `README.md` and each `docs/*.md`, for every term, plus the term's own
/// sources that are files. Decision ids are not scanned. Scanning only the sources an author listed is the gap
/// that let a sense named in another governing document go unseen. Subfolders of `docs/` are scanned only when
/// a term lists them.
pub fn scanned_files(term: &OntologyTerm, root: &Path) -> Vec<String> {
    let mut files = vec!["BRIEF.md".to_string()];
    if root.join("README.md").is_file() {
        files.push("README.md".to_string());
    }
    if let Ok(entries) = std::fs::read_dir(root.join("docs")) {
        let mut docs: Vec<String> = entries
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".md"))
            .map(|n| format!("docs/{n}"))
            .filter(|p| root.join(p).is_file())
            .collect();
        docs.sort();
        files.extend(docs);
    }
    for source in &term.sources {
        if !is_decision_id(source) && root.join(source).is_file() && !files.contains(source) {
            files.push(source.clone());
        }
    }
    files
}

/// The hits of `term` in each scanned file, and the problems met reading them.
pub fn term_hits(term: &OntologyTerm, root: &Path) -> (BTreeMap<String, Vec<Hit>>, Vec<String>) {
    let names = names_of(&term.name, &term.aliases);
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

/// True when an excuse names exactly one file, a marker that is more than a name of the term, and a reason.
pub fn excuse_ok(excuse: &OntologyExcuse, names: &[&str]) -> bool {
    let marker = excuse.marker.trim();
    excuse.files.len() == 1
        && !marker.is_empty()
        && !names.contains(&marker)
        && !excuse.reason.trim().is_empty()
}

/// One finding for each identifying line in `hits` that no sense and no valid excuse of `term` covers.
pub fn uncovered_lines(term: &OntologyTerm, hits: &BTreeMap<String, Vec<Hit>>) -> Vec<String> {
    let name = &term.name;
    let names = names_of(&term.name, &term.aliases);
    let mut found = Vec::new();
    for (path, file_hits) in hits {
        for hit in file_hits.iter().filter(|h| h.identifying) {
            let by_sense = term.senses.iter().any(|s| {
                s.in_file == *path
                    && !s.marker.trim().is_empty()
                    && phrase_in(&hit.context, s.marker.trim())
            });
            let by_excuse = term.excused.iter().any(|e| {
                excuse_ok(e, &names)
                    && e.files[0] == *path
                    && phrase_in(&hit.context, e.marker.trim())
            });
            if !by_sense && !by_excuse {
                found.push(format!(
                    "{name}: {path}:{} names the term as a definition or role and no sense or excuse covers it: {}",
                    hit.number, hit.line
                ));
            }
        }
    }
    found
}

/// Every uncovered identifying line of every term, and every scanned file that cannot be read.
pub fn ontology_problems(terms: &[OntologyTerm], root: &Path) -> Vec<String> {
    let mut found = Vec::new();
    for term in terms {
        let (hits, read_problems) = term_hits(term, root);
        found.extend(read_problems);
        found.extend(uncovered_lines(term, &hits));
    }
    found
}

/// The tables of an `[[array of tables]]` or an inline array of inline tables.
fn tables(item: Option<&Item>) -> Vec<&dyn TableLike> {
    match item {
        Some(Item::ArrayOfTables(array)) => array.iter().map(|t| t as &dyn TableLike).collect(),
        Some(Item::Value(Value::Array(array))) => array
            .iter()
            .filter_map(|v| v.as_inline_table().map(|t| t as &dyn TableLike))
            .collect(),
        _ => Vec::new(),
    }
}

fn table_text(table: &dyn TableLike, key: &str) -> String {
    table
        .get(key)
        .and_then(Item::as_str)
        .unwrap_or_default()
        .to_string()
}

fn table_strings(table: &dyn TableLike, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(Item::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// The terms of the text of `docs/ontology.toml`, or why it does not parse.
pub fn parse_ontology(text: &str) -> Result<Vec<OntologyTerm>, String> {
    let doc: DocumentMut = text.parse().map_err(|e| format!("{e}"))?;
    let terms = tables(doc.get("term"));
    if terms.is_empty() {
        return Err("it has no [[term]] tables".to_string());
    }
    Ok(terms
        .into_iter()
        .map(|t| OntologyTerm {
            name: table_text(t, "name"),
            aliases: table_strings(t, "aliases"),
            sources: table_strings(t, "sources"),
            senses: tables(t.get("senses"))
                .into_iter()
                .map(|s| OntologySense {
                    in_file: table_text(s, "in"),
                    marker: table_text(s, "marker"),
                })
                .collect(),
            excused: tables(t.get("excused"))
                .into_iter()
                .map(|e| OntologyExcuse {
                    files: table_strings(e, "files"),
                    marker: table_text(e, "marker"),
                    reason: table_text(e, "reason"),
                })
                .collect(),
        })
        .collect())
}

// ---------------------------------------------------------------------------------------------------------
// The registry

/// Whether `path` matches the glob `pattern`, segment by segment; `*` matches within one segment.
fn glob_matches(pattern: &str, path: &str) -> bool {
    let patterns: Vec<&str> = pattern.split('/').collect();
    let segments: Vec<&str> = path.split('/').collect();
    patterns.len() == segments.len()
        && patterns
            .iter()
            .zip(&segments)
            .all(|(p, s)| segment_matches(p, s))
}

fn segment_matches(pattern: &str, text: &str) -> bool {
    match pattern.find('*') {
        None => pattern == text,
        Some(star) => {
            let (head, rest) = (&pattern[..star], &pattern[star + 1..]);
            text.strip_prefix(head).is_some_and(|t| {
                (0..=t.len())
                    .filter(|&k| t.is_char_boundary(k))
                    .any(|k| segment_matches(rest, &t[k..]))
            })
        }
    }
}

/// Whether `root` is Athena's own source tree: the registry holds Athena's conventions (an append-only decision
/// log, a timings file for its test units), so it is not run on any other repository.
pub fn is_athena_source(root: &Path) -> bool {
    ["setup.sh", TIMINGS_FILE, "crates/athena-core/Cargo.toml"]
        .iter()
        .all(|p| root.join(p).is_file())
}

/// The text of `path` at `HEAD` of the repository at `root`; None when it is not there or not a repository.
pub fn git_base(root: &Path) -> impl Fn(&str) -> Option<Vec<u8>> {
    let git = athena_git::Git::at(root);
    move |path| git.show_file("HEAD", path).ok().flatten()
}

impl Mirror {
    /// Whether a change to any of `paths` can make this entry drift: the mirror, its source, or for the
    /// ontology the documents it scans. Used to charge a producer only with entries it touched.
    pub fn watches(&self, root: &Path, paths: &[String]) -> bool {
        let touches = |p: &str| paths.iter().any(|c| c == p);
        if touches(self.mirror) {
            return true;
        }
        match self.source {
            Source::Path(p) => touches(p),
            Source::Globs(globs) => paths
                .iter()
                .any(|c| globs.iter().any(|g| glob_matches(g, c))),
            Source::GitHead => false,
            Source::Docs => {
                let governing = |p: &String| {
                    p == "BRIEF.md"
                        || p == "README.md"
                        || p.strip_prefix("docs/")
                            .is_some_and(|n| n.ends_with(".md") && !n.contains('/'))
                };
                if paths.iter().any(governing) {
                    return true;
                }
                let Some(terms) = std::fs::read_to_string(root.join(self.mirror))
                    .ok()
                    .and_then(|t| parse_ontology(&t).ok())
                else {
                    return false;
                };
                terms
                    .iter()
                    .flat_map(|t| &t.sources)
                    .any(|s| !is_decision_id(s) && touches(s))
            }
        }
    }
}

/// The findings of one entry over the tree at `root`; `base` reads a file at the base commit.
pub fn check_entry(entry: &Mirror, root: &Path, base: BaseFn<'_>) -> Vec<Drift> {
    let read = |path: &str| std::fs::read_to_string(root.join(path));
    let inner: Vec<String> = match (entry.rule, entry.source) {
        (Rule::TimingUnits, _) => match read(entry.mirror) {
            Ok(text) => missing_timing_lines(&test_units(root), &text),
            Err(e) => vec![format!("cannot read {}: {e}", entry.mirror)],
        },
        (Rule::AppendOnly, _) => match (base(entry.mirror), std::fs::read(root.join(entry.mirror)))
        {
            (Some(old), Ok(new)) => append_only_problem(&old, &new).into_iter().collect(),
            (None, _) => Vec::new(),
            (_, Err(e)) => vec![format!("cannot read {}: {e}", entry.mirror)],
        },
        (Rule::HashFilesCover, Source::Path(ci)) => match (read(entry.mirror), read(ci)) {
            (Ok(status), Ok(ci_text)) => match status_build_paths(&status) {
                Ok(roots) => hash_glob_problems(&ci_text, &roots),
                Err(e) => vec![e],
            },
            (Err(e), _) => vec![format!("cannot read {}: {e}", entry.mirror)],
            (_, Err(e)) => vec![format!("cannot read {ci}: {e}")],
        },
        (Rule::HashFilesCover, _) => {
            vec!["the build paths rule needs a Source::Path to the workflow file".to_string()]
        }
        (Rule::OntologyTerms, _) => match read(entry.mirror) {
            Ok(text) => match parse_ontology(&text) {
                Ok(terms) => ontology_problems(&terms, root)
                    .into_iter()
                    .map(|p| {
                        format!(
                            "{p}; to fix, add a senses entry or an excuse for that term in {}",
                            entry.mirror
                        )
                    })
                    .collect(),
                Err(e) => vec![format!("does not parse: {e}")],
            },
            Err(e) => vec![format!("cannot read {}: {e}", entry.mirror)],
        },
    };
    inner
        .into_iter()
        .map(|message| Drift {
            mirror: entry.mirror.to_string(),
            message: if message.starts_with(entry.mirror) {
                message
            } else {
                format!("{}: {message}", entry.mirror)
            },
        })
        .collect()
}

/// The findings of every entry of [`REGISTRY`] whose mirror file exists in `root`. With `touched`, only the
/// entries that watch one of those paths run (the filter is per entry, not per finding: an entry a producer
/// touched reports all of its findings, drift already in the base included).
pub fn check_all(root: &Path, base: BaseFn<'_>, touched: Option<&[String]>) -> Vec<Drift> {
    let mut found = Vec::new();
    for entry in REGISTRY {
        if !root.join(entry.mirror).is_file() {
            continue;
        }
        if touched.is_some_and(|paths| !entry.watches(root, paths)) {
            continue;
        }
        found.extend(check_entry(entry, root, base));
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn units(list: &[&str]) -> BTreeSet<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn strings(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn a_test_file_missing_from_the_timings_fails_and_names_the_file_and_the_line() {
        let found = missing_timing_lines(
            &units(&["demo --test a", "demo --test b"]),
            "# header\ndemo --test a 1.00\n",
        );
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("xtask/shard-timings.txt"), "{found:?}");
        assert!(
            found[0].contains("add the line `demo --test b 5.00`"),
            "{found:?}"
        );
        let clean = missing_timing_lines(
            &units(&["demo --test a", "demo --lib"]),
            "demo --test a 1.00\ndemo --lib 2.00\n",
        );
        assert!(clean.is_empty(), "{clean:?}");
    }

    #[test]
    fn a_timing_line_with_no_unit_fails_but_doc_and_bin_lines_are_not_reported() {
        let found = missing_timing_lines(
            &units(&["demo --test a"]),
            "demo --test a 1.00\ndemo --test gone 1.00\ndemo --lib 2.00\n",
        );
        assert_eq!(found.len(), 2, "{found:?}");
        assert!(
            found
                .iter()
                .any(|f| f.contains("line 2") && f.contains("remove the line"))
        );
        assert!(found.iter().any(|f| f.contains("`demo --lib`")));
        let quiet = missing_timing_lines(
            &units(&["demo --test a"]),
            "demo --test a 1.00\ndemo --doc 3.00\ndemo --bin tool 5.00\n",
        );
        assert!(quiet.is_empty(), "{quiet:?}");
    }

    #[test]
    fn units_are_derived_from_the_files_on_disk() {
        let dir = tempfile::tempdir().unwrap();
        let put = |p: &str| {
            let path = dir.path().join(p);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "").unwrap();
        };
        put("crates/demo/src/lib.rs");
        put("crates/demo/tests/a.rs");
        put("crates/demo/tests/folder/main.rs");
        put("crates/demo/tests/common/mod.rs");
        put("crates/demo/tests/notes.txt");
        put("xtask/tests/spec.rs");
        put("xtask/src/main.rs");
        assert_eq!(
            test_units(dir.path()),
            units(&[
                "demo --lib",
                "demo --test a",
                "demo --test folder",
                "xtask --test spec"
            ])
        );
    }

    #[test]
    fn a_rewritten_decision_fails_and_added_text_passes() {
        let base = b"## D01 one\nbody\n## D02 two\n";
        assert_eq!(
            append_only_problem(base, b"## D01 one\nbody\n## D02 two\n## D03 three\n"),
            None
        );
        assert_eq!(append_only_problem(base, base), None);
        let found = append_only_problem(base, b"## D01 one\nbody changed\n## D02 two\n").unwrap();
        assert!(found.contains("docs/decisions.md"), "{found}");
        assert!(
            found.contains("line 2") && found.contains("`body`"),
            "{found}"
        );
        assert!(
            found.contains("add the new decision after the last line"),
            "{found}"
        );
        let cut = append_only_problem(base, b"## D01 one\n").unwrap();
        assert!(cut.contains("line 2"), "{cut}");
    }

    const STATUS: &str = "/// doc\npub const BUILD_PATHS: [&str; 3] = [\n    \"crates/\",\n    \"athena.toml\",\n    \"setup.sh\",\n];\nfn x() {}\n";

    fn ci(globs: &str) -> String {
        format!(
            "      - uses: cache\n        with:\n          key: athena-bin-${{{{ runner.os }}}}-${{{{ hashFiles({globs}) }}}}\n"
        )
    }

    #[test]
    fn build_paths_are_read_from_the_status_text() {
        assert_eq!(
            status_build_paths(STATUS).unwrap(),
            strings(&["crates/", "athena.toml", "setup.sh"])
        );
        assert!(status_build_paths("fn main() {}").is_err());
        assert!(status_build_paths("pub const BUILD_PATHS: [&str; 0] = [];").is_err());
    }

    #[test]
    fn a_ci_key_and_build_paths_that_match_pass() {
        let roots = strings(&["crates/", "athena.toml", "setup.sh"]);
        let text = ci("'crates/**', 'athena.toml', 'setup.sh'");
        assert_eq!(hash_glob_problems(&text, &roots), Vec::<String>::new());
    }

    #[test]
    fn a_glob_hashed_but_missing_from_build_paths_fails_with_the_fix() {
        let roots = strings(&["crates/", "setup.sh"]);
        let text = ci("'crates/**', 'athena.toml', 'setup.sh'");
        let found = hash_glob_problems(&text, &roots);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("`athena.toml`"), "{found:?}");
        assert!(
            found[0].contains("add the path to BUILD_PATHS in crates/athena-core/src/status.rs"),
            "{found:?}"
        );
    }

    #[test]
    fn a_build_path_the_key_does_not_hash_fails_with_the_fix() {
        let roots = strings(&["crates/", "athena.toml", "setup.sh"]);
        let found = hash_glob_problems(&ci("'crates/**', 'setup.sh'"), &roots);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("`athena.toml`"), "{found:?}");
        assert!(
            found[0].contains("add the glob to every athena-bin key"),
            "{found:?}"
        );
        // A directory root hashed without `**` is not covered.
        let narrow = hash_glob_problems(&ci("'crates/', 'athena.toml', 'setup.sh'"), &roots);
        assert!(narrow.iter().any(|f| f.contains("`crates/`")), "{narrow:?}");
    }

    #[test]
    fn a_workflow_with_no_athena_bin_key_fails() {
        let found = hash_glob_problems("key: other-${{ hashFiles('a') }}\n", &strings(&["a"]));
        assert_eq!(found.len(), 1);
        assert!(found[0].contains(".github/workflows/ci.yml"), "{found:?}");
    }

    #[test]
    fn globs_match_by_segment() {
        assert!(glob_matches(
            "crates/*/tests/*.rs",
            "crates/athena-cli/tests/drain.rs"
        ));
        assert!(!glob_matches(
            "crates/*/tests/*.rs",
            "crates/athena-cli/tests/a/main.rs"
        ));
        assert!(glob_matches(
            "crates/*/tests/*/main.rs",
            "crates/x/tests/a/main.rs"
        ));
        assert!(!glob_matches("crates/*/src/lib.rs", "xtask/src/lib.rs"));
        assert!(glob_matches("xtask/tests/*.rs", "xtask/tests/policy.rs"));
        assert!(!glob_matches(
            "xtask/tests/*.rs",
            "xtask/tests/fixtures/x.html"
        ));
    }

    // -- the ontology walk -------------------------------------------------------------------------------

    const ONTOLOGY: &str = r#"
[about]
rule = "r"
read_by = "t"

[[term]]
name = "Hermes"
aliases = ["Hermes Agent"]
sources = ["D01", "BRIEF.md", "docs/roadmap.md"]
senses = [{ in = "BRIEF.md", marker = "a tool", says = "tool" }]
excused = [{ files = ["docs/roadmap.md"], marker = "shipping order", reason = "a schedule line" }]

[[term]]
name = "job"
sources = ["BRIEF.md"]

[[term.senses]]
in = "BRIEF.md"
marker = "one unit"
says = "unit"
"#;

    fn tree_with_brief(brief: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("docs")).unwrap();
        std::fs::write(dir.path().join("BRIEF.md"), brief).unwrap();
        std::fs::write(
            dir.path().join("docs/roadmap.md"),
            "* Hermes: shipping order matters\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn the_ontology_text_parses_into_plain_terms() {
        let terms = parse_ontology(ONTOLOGY).unwrap();
        assert_eq!(terms.len(), 2);
        assert_eq!(terms[0].name, "Hermes");
        assert_eq!(terms[0].aliases, strings(&["Hermes Agent"]));
        assert_eq!(terms[0].senses[0].marker, "a tool");
        assert_eq!(terms[0].excused[0].files, strings(&["docs/roadmap.md"]));
        assert_eq!(terms[1].senses[0].in_file, "BRIEF.md");
        assert!(parse_ontology("not toml [").is_err());
        assert!(parse_ontology("[about]\nrule = \"r\"\n").is_err());
    }

    #[test]
    fn a_governing_line_naming_a_second_sense_is_found_until_a_sense_or_excuse_covers_it() {
        let dir = tree_with_brief("* Hermes: a tool\n* Hermes: a named role\n* job: one unit\n");
        let terms = parse_ontology(ONTOLOGY).unwrap();
        let found = ontology_problems(&terms, dir.path());
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(
            found[0].starts_with("Hermes: BRIEF.md:2 names the term as a definition or role"),
            "{found:?}"
        );

        let mut both = terms.clone();
        both[0].senses.push(OntologySense {
            in_file: "BRIEF.md".into(),
            marker: "a named role".into(),
        });
        assert!(ontology_problems(&both, dir.path()).is_empty());

        let mut excused = terms;
        excused[0].excused.push(OntologyExcuse {
            files: vec!["BRIEF.md".into()],
            marker: "a named role".into(),
            reason: "a different sentence".into(),
        });
        assert!(ontology_problems(&excused, dir.path()).is_empty());
        // An excuse with no reason covers nothing.
        excused[0].excused[1].reason = " ".into();
        assert_eq!(ontology_problems(&excused, dir.path()).len(), 1);
    }

    // -- a whole tree ---------------------------------------------------------------------------------------

    fn put(dir: &Path, path: &str, text: &str) {
        let full = dir.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, text).unwrap();
    }

    const TREE_STATUS: &str =
        "pub const BUILD_PATHS: [&str; 2] = [\"crates/\", \"athena.toml\"];\n";

    /// A small tree that follows every rule.
    fn clean_tree() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        put(root, "setup.sh", "echo\n");
        put(root, "crates/athena-core/Cargo.toml", "[package]\n");
        put(root, "crates/demo/tests/a.rs", "");
        put(root, "crates/demo/tests/b.rs", "");
        put(
            root,
            TIMINGS_FILE,
            "demo --test a 1.00\ndemo --test b 1.00\n",
        );
        put(root, "crates/athena-core/src/status.rs", TREE_STATUS);
        put(
            root,
            ".github/workflows/ci.yml",
            &ci("'crates/**', 'athena.toml'"),
        );
        put(root, "docs/decisions.md", "## D01\n");
        put(root, "BRIEF.md", "* Hermes: a tool\n");
        put(
            root,
            "docs/ontology.toml",
            "[[term]]\nname = \"Hermes\"\nsources = [\"BRIEF.md\"]\nsenses = [{ in = \"BRIEF.md\", marker = \"a tool\", says = \"x\" }]\n",
        );
        dir
    }

    fn base_of(text: &'static str) -> impl Fn(&str) -> Option<Vec<u8>> {
        move |p| (p == "docs/decisions.md").then(|| text.as_bytes().to_vec())
    }

    #[test]
    fn a_clean_tree_has_no_drift() {
        let dir = clean_tree();
        let found = check_all(dir.path(), &base_of("## D01\n"), None);
        assert!(found.is_empty(), "{found:?}");
    }

    #[test]
    fn every_rule_reports_a_drift_with_the_file_and_the_fix() {
        let dir = clean_tree();
        let root = dir.path();
        put(root, "crates/demo/tests/c.rs", "");
        put(root, "docs/decisions.md", "## D01 rewritten\n");
        put(
            root,
            "crates/athena-core/src/status.rs",
            "pub const BUILD_PATHS: [&str; 1] = [\"crates/\"];\n",
        );
        put(
            root,
            "BRIEF.md",
            "* Hermes: a tool\n* Hermes: a named role\n",
        );
        let found = check_all(root, &base_of("## D01\n"), None);
        let by =
            |file: &str| -> Vec<&Drift> { found.iter().filter(|d| d.mirror == file).collect() };
        assert!(
            by(TIMINGS_FILE)[0].message.contains("demo --test c"),
            "{found:?}"
        );
        assert!(
            by("docs/decisions.md")[0].message.contains("append"),
            "{found:?}"
        );
        assert!(
            by("crates/athena-core/src/status.rs")[0]
                .message
                .contains("add the path to BUILD_PATHS in crates/athena-core/src/status.rs"),
            "{found:?}"
        );
        let ontology = by("docs/ontology.toml");
        assert!(
            ontology[0].message.starts_with("docs/ontology.toml:"),
            "{found:?}"
        );
        assert!(
            ontology[0]
                .message
                .ends_with("add a senses entry or an excuse for that term in docs/ontology.toml"),
            "{found:?}"
        );
        assert!(ontology[0].message.contains("BRIEF.md:2"), "{found:?}");
    }

    #[test]
    fn a_missing_base_skips_the_append_only_rule() {
        let dir = clean_tree();
        put(dir.path(), "docs/decisions.md", "something else\n");
        assert!(check_all(dir.path(), &|_: &str| None, None).is_empty());
    }

    #[test]
    fn a_tree_that_is_not_athenas_source_is_recognised() {
        let dir = clean_tree();
        assert!(is_athena_source(dir.path()));
        let other = tempfile::tempdir().unwrap();
        put(other.path(), "docs/decisions.md", "x\n");
        assert!(!is_athena_source(other.path()));
        // The registry also runs nothing on a tree that holds none of the mirror files.
        assert!(check_all(other.path(), &|_: &str| None, None).is_empty());
    }

    #[test]
    fn the_touch_filter_runs_only_the_entries_a_change_can_affect() {
        let dir = clean_tree();
        let root = dir.path();
        put(root, "crates/demo/tests/c.rs", "");
        let base = base_of("## D01\n");
        let touched = |paths: &[&str]| check_all(root, &base, Some(&strings(paths)));
        assert!(touched(&["README.md"]).is_empty());
        assert!(
            touched(&["crates/demo/tests/b.rs"])[0]
                .message
                .contains("demo --test c")
        );
        assert!(!touched(&[TIMINGS_FILE]).is_empty());
        // A change to BRIEF.md reaches the ontology entry.
        put(
            root,
            "BRIEF.md",
            "* Hermes: a tool\n* Hermes: a named role\n",
        );
        let found = touched(&["BRIEF.md"]);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].mirror, "docs/ontology.toml");
    }

    #[test]
    fn a_status_and_ci_edited_together_pass_but_one_alone_drifts() {
        let dir = clean_tree();
        let root = dir.path();
        let base = base_of("## D01\n");
        put(
            root,
            "crates/athena-core/src/status.rs",
            "pub const BUILD_PATHS: [&str; 3] = [\"crates/\", \"athena.toml\", \"setup.sh\"];\n",
        );
        let found = check_all(root, &base, None);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].message.contains("`setup.sh`"));
        put(
            root,
            ".github/workflows/ci.yml",
            &ci("'crates/**', 'athena.toml', 'setup.sh'"),
        );
        assert!(check_all(root, &base, None).is_empty());
    }

    #[test]
    fn watches_follow_each_source_form() {
        let dir = clean_tree();
        let root = dir.path();
        let watch = |name: &str, paths: &[&str]| {
            let entry = REGISTRY.iter().find(|m| m.name == name).unwrap();
            entry.watches(root, &strings(paths))
        };
        // Path: the mirror or the named source.
        assert!(watch("build paths", &["crates/athena-core/src/status.rs"]));
        assert!(watch("build paths", &[".github/workflows/ci.yml"]));
        assert!(!watch("build paths", &["README.md"]));
        // Globs: the mirror or any matching path.
        assert!(watch("shard timings", &["crates/new/tests/x.rs"]));
        assert!(watch("shard timings", &["crates/new/src/lib.rs"]));
        assert!(!watch("shard timings", &["crates/new/src/main.rs"]));
        // GitHead: the mirror only.
        assert!(watch("decision log", &["docs/decisions.md"]));
        assert!(!watch("decision log", &["README.md"]));
        // Docs: the ontology, BRIEF.md, README.md, any docs/*.md and a term's file source; a docs subfolder
        // file no term lists, and a decision id, are not.
        put(
            root,
            "docs/ontology.toml",
            "[[term]]\nname = \"Hermes\"\nsources = [\"D01\", \"BRIEF.md\", \"docs/roadmap.md\"]\n",
        );
        assert!(watch("ontology", &["docs/ontology.toml"]));
        assert!(watch("ontology", &["BRIEF.md"]));
        assert!(watch("ontology", &["docs/roadmap.md"]));
        assert!(watch("ontology", &["docs/other.md"]));
        assert!(watch("ontology", &["README.md"]));
        assert!(!watch("ontology", &["docs/sub/other.md"]));
        assert!(!watch("ontology", &["D01"]));
        // A file that does not parse watches only itself and the governing documents.
        put(root, "docs/ontology.toml", "not toml [");
        assert!(watch("ontology", &["BRIEF.md"]));
        assert!(watch("ontology", &["docs/roadmap.md"]));
        assert!(!watch("ontology", &["docs/sub/roadmap.md"]));
    }

    #[test]
    fn a_registered_entry_has_a_name_and_a_mirror_that_is_a_relative_path() {
        let mut names = BTreeSet::new();
        for entry in REGISTRY {
            assert!(names.insert(entry.name), "duplicate name {}", entry.name);
            assert!(!entry.mirror.starts_with('/') && !entry.mirror.is_empty());
        }
    }
}
