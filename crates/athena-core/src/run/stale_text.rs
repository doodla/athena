//! Stale text: text that still says what a change made false.
//!
//! Root cause of the failure class: a change that altered a value or a stated rule left the old text in a
//! decision record or a help string, and only the final verdict read the repository for it, so each case needed
//! a follow-up job. [`check`] reads the change before any verifier runs and names each place that still says the
//! old thing; the producer loop sends the list back (see `producer_loop`) and the candidate step ends a job
//! that still has it.
//!
//! The check is textual. Old text comes from five sources, each narrow:
//!
//! 1. a `const` or `static` of the diff whose single number or string literal changed;
//! 2. a sentence of the accepted reading with `from A to B` (numbers or quoted strings) and a named identifier;
//! 3. a quoted span of six or more words after a marker such as `no longer` in the reading;
//! 4. a decision id that the reading names as the object of a reversal verb;
//! 5. a comment or prose line of the diff that a replaced hunk removed.
//!
//! A place is reported unless the diff added that line, the entry or paragraph holding it is marked `amended by`
//! or `superseded by`, or the file is `CHANGELOG.md` (which records history), `records/`, `target/` or `.git/`.
//! The module is pure over [`Input`]; [`check`] reads the input from git.

use anyhow::Result;
use athena_git::Git;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// The decision log, and the list that links its decisions to each other.
pub(crate) const DECISION_LOG: &str = "docs/decisions.md";
const DECISION_LINKS: &str = "docs/decision-links.toml";

/// The start of the closing reason of a job that ends with stale text.
pub(crate) const STALE_REASON_PREFIX: &str = "stale text left by the change:";

/// A removed line is a claim only with at least this many words.
pub(crate) const MIN_CLAIM_WORDS: usize = 6;
/// The old value and a word of the name of its constant stand at most this many words apart.
pub(crate) const WINDOW_WORDS: usize = 4;
/// A value that matches more places than this is too common to close a job: its hits are advisory.
pub(crate) const FIRM_VALUE_HITS: usize = 6;
/// How many findings a send-back or a closing reason lists; the rest is counted.
pub(crate) const MAX_LISTED: usize = 30;
/// A file larger than this is not searched.
const MAX_FILE_BYTES: u64 = 2 * 1024 * 1024;
/// Blobs are read from git this many at a time.
const BLOB_CHUNK: usize = 200;
/// The extensions of the files that are searched.
const SEARCHED_EXTENSIONS: [&str; 6] = ["rs", "md", "toml", "yml", "yaml", "sh"];
/// Words of a constant's name that say nothing about what it limits.
const GENERIC: [&str; 10] = [
    "max", "min", "limit", "default", "value", "count", "number", "size", "total", "num",
];
/// Words after which the reading may quote what a rule used to say.
const CLAIM_MARKERS: [&str; 6] = [
    "no longer",
    "instead of",
    "rather than",
    "replaces",
    "reverses",
    "used to say",
];
/// Verbs whose object, a decision id, the reading says is reversed.
const REVERSAL_VERBS: [&str; 7] = [
    "reverses",
    "reversing",
    "supersedes",
    "superseding",
    "overturns",
    "overturning",
    "reverts",
];
/// Words after a decision id that say it is reversed.
const REVERSED_PHRASES: [&str; 6] = [
    "no longer holds",
    "no longer stands",
    "no longer applies",
    "is reversed",
    "is superseded",
    "is overturned",
];

/// One hunk of the diff: the lines it removes and the lines it adds, in one file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Hunk {
    pub path: String,
    pub removed: Vec<String>,
    pub added: Vec<String>,
}

/// Everything the check reads, as plain data.
#[derive(Debug, Clone, Default)]
pub(crate) struct Input {
    /// The searched files as the change leaves them: path to text.
    pub files: BTreeMap<String, String>,
    /// The diff of the change, as hunks.
    pub hunks: Vec<Hunk>,
    /// The sentences of the accepted reading.
    pub reading: Vec<String>,
    /// The ids (`D45`) of the headings of the decision log at the base.
    pub base_headings: BTreeSet<String>,
    /// The text of `docs/decision-links.toml` as the change leaves it.
    pub links: String,
}

/// One place that still says the old thing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Stale {
    pub path: String,
    /// The 1-based line.
    pub line: usize,
    /// The decision whose entry holds the line, for a place in the decision log.
    pub decision: Option<String>,
    /// The old value or claim the place still states.
    pub what: String,
    /// What made it old.
    pub why: String,
    /// The line, cut to a readable length.
    pub text: String,
}

impl fmt::Display for Stale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.path, self.line)?;
        if let Some(decision) = &self.decision {
            write!(f, " ({decision})")?;
        }
        write!(
            f,
            ": still says {} ({}): {}",
            self.what, self.why, self.text
        )
    }
}

/// What the check found: firm findings close a job; advisory ones are printed only.
#[derive(Debug, Clone, Default)]
pub(crate) struct Report {
    pub firm: Vec<Stale>,
    pub advisory: Vec<Stale>,
}

/// An old value and the words of the name that limits it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ValueNeedle {
    old: String,
    new: String,
    keywords: Vec<String>,
    why: String,
}

/// An old sentence, as the words it is made of.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ClaimNeedle {
    words: Vec<String>,
    why: String,
}

/// A decision the reading says is reversed.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DecisionNeedle {
    id: String,
    why: String,
}

#[derive(Debug, Default)]
struct Needles {
    values: Vec<ValueNeedle>,
    claims: Vec<ClaimNeedle>,
    decisions: Vec<DecisionNeedle>,
}

impl Needles {
    fn is_empty(&self) -> bool {
        self.values.is_empty() && self.claims.is_empty() && self.decisions.is_empty()
    }
}

// ---------------------------------------------------------------------------------------------------------
// Words
// ---------------------------------------------------------------------------------------------------------

/// A word as the name matching compares it: lowercase, no `'s`, no plural `s`.
fn norm_word(word: &str) -> String {
    let lower = word.trim_matches('\'').to_lowercase();
    let base = lower.strip_suffix("'s").unwrap_or(&lower);
    match base.strip_suffix('s') {
        Some(stem) if base.len() > 3 => stem.to_string(),
        _ => base.to_string(),
    }
}

/// The words of a text: runs of letters and digits, lowercase. Case, punctuation and line breaks do not count.
fn plain_words(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// The words of a constant's name that tell what it limits.
fn keywords_of(name: &str) -> Vec<String> {
    let mut keywords: Vec<String> = Vec::new();
    for part in name.split('_') {
        let lower = part.to_lowercase();
        if lower.chars().count() <= 2 || GENERIC.contains(&lower.as_str()) {
            continue;
        }
        let word = norm_word(&lower);
        if !keywords.contains(&word) {
            keywords.push(word);
        }
    }
    keywords
}

/// A character of plain prose: a run of these is a stretch of words, and any other character ends the run.
fn is_plain(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, ' ' | ',' | '-' | '\'')
}

/// The words of one plain run, with their byte spans: letters, digits and apostrophes.
fn word_spans(run: &str) -> Vec<(usize, usize, String)> {
    let mut spans = Vec::new();
    let mut start: Option<usize> = None;
    for (at, c) in run.char_indices() {
        let inside = c.is_alphanumeric() || c == '\'';
        match (inside, start) {
            (true, None) => start = Some(at),
            (false, Some(s)) => {
                spans.push((s, at, run[s..at].to_string()));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(s) = start {
        spans.push((s, run.len(), run[s..].to_string()));
    }
    spans
}

// ---------------------------------------------------------------------------------------------------------
// Needles
// ---------------------------------------------------------------------------------------------------------

/// Splits the texts of a reading into sentences: at `. `, `; ` and line ends.
pub(crate) fn reading_sentences(
    summary: &str,
    assumptions: &[String],
    criteria: &[String],
) -> Vec<String> {
    std::iter::once(summary)
        .chain(assumptions.iter().map(String::as_str))
        .chain(criteria.iter().map(String::as_str))
        .flat_map(|text| {
            text.replace("; ", "\n")
                .replace(". ", "\n")
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// The single number or string a `const` or `static` is set to. A number may carry an unsigned suffix; a
/// string needs at least three characters and no escape.
fn literal(value: &str) -> Option<String> {
    let value = value.trim();
    if let Some(inner) = value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return (inner.chars().count() >= 3 && !inner.contains(['"', '\\']))
            .then(|| inner.to_string());
    }
    let digits: String = value
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '_')
        .collect();
    if !digits.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    let suffix = &value[digits.len()..];
    if !(suffix.is_empty() || ["u8", "u16", "u32", "u64", "u128", "usize"].contains(&suffix)) {
        return None;
    }
    Some(digits.replace('_', ""))
}

/// The name and the literal of a `const NAME: T = LITERAL;` or `static` line.
fn parse_const(line: &str) -> Option<(String, String)> {
    let mut rest = line.trim();
    if let Some(after) = rest.strip_prefix("pub") {
        rest = after.trim_start();
        if rest.starts_with('(') {
            rest = rest[rest.find(')')? + 1..].trim_start();
        }
    }
    let rest = rest
        .strip_prefix("const ")
        .or_else(|| rest.strip_prefix("static "))?
        .trim_start();
    let (name, tail) = rest.split_once(':')?;
    let name = name.trim();
    let screaming = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
    if !screaming {
        return None;
    }
    let (_, value) = tail.split_once('=')?;
    let value = value.trim().strip_suffix(';')?;
    Some((name.to_string(), literal(value)?))
}

/// Each `const` of one file that the diff removes and adds again with a different single literal.
fn const_values(hunks: &[Hunk]) -> Vec<ValueNeedle> {
    let mut paths: Vec<&str> = hunks.iter().map(|h| h.path.as_str()).collect();
    paths.sort_unstable();
    paths.dedup();
    let mut needles = Vec::new();
    for path in paths {
        let side = |removed: bool| -> Vec<(String, String)> {
            hunks
                .iter()
                .filter(|h| h.path == path)
                .flat_map(|h| if removed { &h.removed } else { &h.added })
                .filter_map(|l| parse_const(l))
                .collect()
        };
        let (old, new) = (side(true), side(false));
        for (name, old_value) in &old {
            let once = |list: &[(String, String)]| list.iter().filter(|(n, _)| n == name).count();
            if once(&old) != 1 || once(&new) != 1 {
                continue;
            }
            let Some((_, new_value)) = new.iter().find(|(n, _)| n == name) else {
                continue;
            };
            let keywords = keywords_of(name);
            if new_value == old_value || keywords.is_empty() {
                continue;
            }
            needles.push(ValueNeedle {
                old: old_value.clone(),
                new: new_value.clone(),
                keywords,
                why: format!("`{name}` changed from {old_value} to {new_value} in {path}"),
            });
        }
    }
    needles
}

/// The identifiers a sentence names: `SCREAMING_SNAKE` words and backticked words that are no path or phrase.
fn identifiers(sentence: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for word in sentence.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
        let screaming = word.contains('_')
            && word.starts_with(|c: char| c.is_ascii_uppercase())
            && word
                .chars()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
        if screaming {
            found.push(word.to_string());
        }
    }
    for (n, span) in sentence.split('`').enumerate() {
        let quoted = n % 2 == 1;
        if quoted
            && span.chars().count() >= 3
            && !span.contains(['/', '.', ' ', ':', '(', ')', '"'])
            && span.chars().any(|c| c.is_alphabetic())
        {
            found.push(span.to_string());
        }
    }
    found
}

/// A number or a double-quoted string at the start of `text`, with the rest of the text after it.
fn operand(text: &str) -> Option<(String, &str)> {
    if let Some(rest) = text.strip_prefix('"') {
        let end = rest.find('"')?;
        let inner = &rest[..end];
        return (inner.chars().count() >= 3).then(|| (inner.to_string(), &rest[end + 1..]));
    }
    let digits: String = text.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return None;
    }
    let rest = &text[digits.len()..];
    let mut after = rest.chars();
    let (next, following) = (after.next(), after.next());
    let joined = next.is_some_and(|c| c.is_alphanumeric() || c == '_')
        || (next == Some('.') && following.is_some_and(|c| c.is_ascii_digit()));
    (!joined).then_some((digits, rest))
}

/// Each `from A to B` of a sentence, where A and B are numbers or quoted strings, when the sentence names an
/// identifier: the old value A is limited by the words of that identifier.
fn reading_values(sentence: &str) -> Vec<ValueNeedle> {
    let mut keywords: Vec<String> = Vec::new();
    for ident in identifiers(sentence) {
        for word in keywords_of(&ident) {
            if !keywords.contains(&word) {
                keywords.push(word);
            }
        }
    }
    if keywords.is_empty() {
        return Vec::new();
    }
    let lower = sentence.to_ascii_lowercase();
    let mut needles = Vec::new();
    for (at, _) in lower.match_indices("from ") {
        if sentence[..at]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric())
        {
            continue;
        }
        let Some((old, rest)) = operand(&sentence[at + 5..]) else {
            continue;
        };
        let Some(rest) = rest.strip_prefix(" to ") else {
            continue;
        };
        let Some((new, _)) = operand(rest) else {
            continue;
        };
        if old != new {
            needles.push(ValueNeedle {
                why: format!("the reading changes it from {old} to {new}"),
                old,
                new,
                keywords: keywords.clone(),
            });
        }
    }
    needles
}

/// The quoted or backticked spans of six or more words that start within 40 characters after a marker.
fn reading_claims(sentence: &str) -> Vec<ClaimNeedle> {
    let lower = sentence.to_ascii_lowercase();
    let mut needles = Vec::new();
    for marker in CLAIM_MARKERS {
        for (at, _) in lower.match_indices(marker) {
            let from = at + marker.len();
            let mut to = (from + 40).min(sentence.len());
            while !sentence.is_char_boundary(to) {
                to -= 1;
            }
            let Some(open) = sentence[from..to].find(['"', '`']) else {
                continue;
            };
            let start = from + open;
            let quote = &sentence[start..start + 1];
            let Some(len) = sentence[start + 1..].find(quote) else {
                continue;
            };
            let words = plain_words(&sentence[start + 1..start + 1 + len]);
            if words.len() >= MIN_CLAIM_WORDS {
                needles.push(ClaimNeedle {
                    words,
                    why: format!("the reading says it is no longer so ({marker})"),
                });
            }
        }
    }
    needles
}

/// The id (`D45`) at the start of `text`, written `D` and two or more digits; the rest follows it.
fn leading_id(text: &str) -> Option<(String, &str)> {
    let rest = text.strip_prefix(['D', 'd'])?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    let after = &rest[digits.len()..];
    let ok = digits.len() >= 2 && !after.starts_with(|c: char| c.is_alphanumeric() || c == '_');
    ok.then(|| (format!("D{digits}"), after))
}

/// The ids of the decisions a sentence says are reversed: the object of a reversal verb, or an id followed by
/// words that say it no longer holds.
fn reading_decisions(sentence: &str) -> Vec<DecisionNeedle> {
    let lower = sentence.to_ascii_lowercase();
    let mut found: Vec<DecisionNeedle> = Vec::new();
    let mut push = |id: String, why: String| {
        if !found.iter().any(|n| n.id == id) {
            found.push(DecisionNeedle { id, why });
        }
    };
    for verb in REVERSAL_VERBS {
        for (at, _) in lower.match_indices(verb) {
            let before = lower[..at].chars().next_back();
            if before.is_some_and(|c| c.is_alphanumeric()) {
                continue;
            }
            let after = lower[at + verb.len()..].trim_start();
            let after = after.strip_prefix("decision ").unwrap_or(after);
            if let Some((id, _)) = leading_id(after) {
                push(id, format!("the reading says the change {verb} it"));
            }
        }
    }
    for (at, _) in lower.match_indices(['d']) {
        if lower[..at]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric())
        {
            continue;
        }
        let Some((id, after)) = leading_id(&lower[at..]) else {
            continue;
        };
        let after = after.trim_start_matches(['\'', 's']).trim_start();
        if let Some(phrase) = REVERSED_PHRASES.iter().find(|p| after.starts_with(**p)) {
            push(id, format!("the reading says it {phrase}"));
        }
    }
    found
}

/// The text of a line that states something in prose: a comment of source, a `#` comment of a config or
/// script, or a non-table line of a document. `None` for code.
fn prose_of(path: &str, line: &str) -> Option<String> {
    let t = line.trim();
    if path.ends_with(".md") {
        if t.is_empty() || t.starts_with(['|', '`']) {
            return None;
        }
        return Some(t.trim_start_matches(['#', '-', '*', '>', ' ']).to_string());
    }
    let body = if path.ends_with(".rs") {
        t.strip_prefix("//")?
            .trim_start_matches(['/', '!'])
            .to_string()
    } else {
        t.strip_prefix('#')?.trim_start_matches('#').to_string()
    };
    Some(body)
}

/// The sentences of the lines a replaced hunk removed and the diff does not add back anywhere.
fn diff_claims(hunks: &[Hunk]) -> Vec<ClaimNeedle> {
    let added: BTreeSet<Vec<String>> = hunks
        .iter()
        .flat_map(|h| &h.added)
        .map(|l| plain_words(l))
        .collect();
    let mut claims: Vec<ClaimNeedle> = Vec::new();
    for hunk in hunks
        .iter()
        .filter(|h| !h.removed.is_empty() && !h.added.is_empty())
    {
        for line in &hunk.removed {
            let Some(prose) = prose_of(&hunk.path, line) else {
                continue;
            };
            let words = plain_words(&prose);
            let moved = added.iter().any(|a| a.join(" ").contains(&words.join(" ")));
            if words.len() >= MIN_CLAIM_WORDS && !moved && !claims.iter().any(|c| c.words == words)
            {
                claims.push(ClaimNeedle {
                    words,
                    why: format!("the change replaced this sentence in {}", hunk.path),
                });
            }
        }
    }
    claims
}

fn needles(hunks: &[Hunk], reading: &[String]) -> Needles {
    let mut found = Needles {
        values: const_values(hunks),
        claims: diff_claims(hunks),
        decisions: Vec::new(),
    };
    for sentence in reading {
        found.values.extend(reading_values(sentence));
        found.claims.extend(reading_claims(sentence));
        found.decisions.extend(reading_decisions(sentence));
    }
    found
}

// ---------------------------------------------------------------------------------------------------------
// Marks
// ---------------------------------------------------------------------------------------------------------

/// Whether `token` is a decision id (`D45`, `(D45)`, `D45.`).
fn is_id_token(token: &str) -> bool {
    leading_id(token.trim_start_matches(['(', '`', '*', '_'])).is_some()
}

/// Whether `token` is a path or file name with an extension (`docs/x.md`, `main.rs`).
fn is_path_token(token: &str) -> bool {
    let chars: Vec<char> = token
        .trim_matches(|c: char| ".,;:()'\"`".contains(c))
        .chars()
        .collect();
    chars
        .windows(3)
        .any(|w| w[1] == '.' && w[0].is_alphanumeric() && w[2].is_alphanumeric())
}

/// Whether a text says `amended by` or `superseded by` something: a decision id, and outside the log also a
/// backticked name or a path with an extension. `amended by hand` marks nothing.
fn says_marked(text: &str, ids_only: bool) -> bool {
    let lower = text.to_ascii_lowercase();
    ["amended by", "superseded by"].iter().any(|phrase| {
        lower.match_indices(phrase).any(|(at, _)| {
            let before = lower[..at].chars().next_back();
            if before.is_some_and(|c| c.is_alphanumeric()) {
                return false;
            }
            let token = lower[at + phrase.len()..]
                .split_whitespace()
                .next()
                .unwrap_or("");
            is_id_token(token) || (!ids_only && (token.starts_with('`') || is_path_token(token)))
        })
    })
}

/// The id of a heading line of the decision log, `## D45 · Title`.
fn heading_id(line: &str) -> Option<String> {
    let rest = line.strip_prefix("## D")?;
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    let after = &rest[digits.len()..];
    let ok = !digits.is_empty() && !after.starts_with(|c: char| c.is_alphanumeric());
    ok.then(|| format!("D{digits}"))
}

/// The ids of the headings of a decision log.
pub(crate) fn headings(log: &str) -> BTreeSet<String> {
    log.lines().filter_map(heading_id).collect()
}

/// The decisions the links list marks: one with a `superseded` link, and one that a link names as older whose
/// newer entry is not in the base log, because this change reconciles it itself. A link whose newer entry stood
/// at the base and whose status is `amended` marks nothing: the older decision stays active. A list that does
/// not parse marks nothing.
fn link_marked(links: &str, base_headings: &BTreeSet<String>) -> BTreeSet<String> {
    let mut marked = BTreeSet::new();
    let Ok(doc) = links.parse::<toml_edit::DocumentMut>() else {
        return marked;
    };
    let Some(tables) = doc.get("link").and_then(|l| l.as_array_of_tables()) else {
        return marked;
    };
    for table in tables {
        let text = |key: &str| table.get(key).and_then(|v| v.as_str());
        let (Some(older), Some(newer)) = (text("older"), text("newer")) else {
            continue;
        };
        if text("status") == Some("superseded") || !base_headings.contains(newer) {
            marked.insert(older.to_string());
        }
    }
    marked
}

/// One entry of the decision log: its id, its first and last line (1-based, inclusive) and whether it is marked.
struct Entry {
    id: String,
    first: usize,
    last: usize,
    marked: bool,
}

fn log_entries(log: &str, link_marked: &BTreeSet<String>) -> Vec<Entry> {
    let lines: Vec<&str> = log.lines().collect();
    let mut entries: Vec<Entry> = Vec::new();
    let mut open: Option<(String, usize)> = None;
    let close = |entries: &mut Vec<Entry>, open: Option<(String, usize)>, last: usize| {
        if let Some((id, first)) = open {
            let text = lines[first - 1..last].join("\n");
            let marked = says_marked(&text, true) || link_marked.contains(&id);
            entries.push(Entry {
                id,
                first,
                last,
                marked,
            });
        }
    };
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("## ") {
            close(&mut entries, open.take(), i);
            open = heading_id(line).map(|id| (id, i + 1));
        }
    }
    close(&mut entries, open.take(), lines.len());
    entries
}

/// Whether the blank-line paragraph around line `i` is marked.
fn paragraph_marked(lines: &[&str], i: usize) -> bool {
    let first = (0..=i)
        .rev()
        .find(|k| lines[*k].trim().is_empty())
        .map_or(0, |k| k + 1);
    let last = (i..lines.len())
        .find(|k| lines[*k].trim().is_empty())
        .unwrap_or(lines.len());
    says_marked(&lines[first..last].join("\n"), false)
}

// ---------------------------------------------------------------------------------------------------------
// Search
// ---------------------------------------------------------------------------------------------------------

/// Whether a path is searched: not the changelog (which records history), the records, the build folder or git.
fn searchable(path: &str) -> bool {
    path != "CHANGELOG.md"
        && !path.starts_with("records/")
        && !path.starts_with("target/")
        && !path.starts_with(".git/")
        && path
            .rsplit_once('.')
            .is_some_and(|(_, ext)| SEARCHED_EXTENSIONS.contains(&ext))
}

/// The byte offsets where `token` stands in `line` as a whole token: no letter, digit or `_` touches it and no
/// `.` joins it to a digit.
fn token_positions(line: &str, token: &str) -> Vec<usize> {
    if token.is_empty() {
        return Vec::new();
    }
    line.match_indices(token)
        .map(|(at, _)| at)
        .filter(|at| {
            let end = at + token.len();
            let mut before = line[..*at].chars().rev();
            let (prev, prev2) = (before.next(), before.next());
            let mut after = line[end..].chars();
            let (next, next2) = (after.next(), after.next());
            let touches = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '_');
            let dotted = |dot: Option<char>, other: Option<char>| {
                dot == Some('.') && other.is_some_and(|c| c.is_ascii_digit())
            };
            !touches(prev) && !touches(next) && !dotted(prev, prev2) && !dotted(next, next2)
        })
        .collect()
}

/// Whether the old value at `start..end` of `line` stands in one plain run with a word of the name, at most
/// [`WINDOW_WORDS`] words away.
fn near_keyword(line: &str, start: usize, end: usize, keywords: &[String]) -> bool {
    let run_start = line[..start]
        .char_indices()
        .rev()
        .find(|(_, c)| !is_plain(*c))
        .map_or(0, |(at, c)| at + c.len_utf8());
    let run_end = line[end..]
        .char_indices()
        .find(|(_, c)| !is_plain(*c))
        .map_or(line.len(), |(at, _)| end + at);
    let words = word_spans(&line[run_start..run_end]);
    let (from, to) = (start - run_start, end - run_start);
    let old: Vec<usize> = (0..words.len())
        .filter(|k| words[*k].0 >= from && words[*k].1 <= to)
        .collect();
    let (Some(first), Some(last)) = (old.first().copied(), old.last().copied()) else {
        return false;
    };
    (0..words.len())
        .filter(|k| !old.contains(k))
        .filter(|k| keywords.contains(&norm_word(&words[*k].2)))
        .any(|k| {
            let distance = if k > last { k - last } else { first - k };
            distance <= WINDOW_WORDS
        })
}

/// Whether line `i` still states the old value of `needle` in prose.
fn value_hit(needle: &ValueNeedle, lines: &[&str], i: usize) -> bool {
    let line = lines[i];
    if !needle.old.chars().all(is_plain) || !token_positions(line, &needle.new).is_empty() {
        return false;
    }
    let near = token_positions(line, &needle.old)
        .into_iter()
        .any(|at| near_keyword(line, at, at + needle.old.len(), &needle.keywords));
    if !near {
        return false;
    }
    // A single digit is everywhere: with two or more name words, two of them must stand within two lines.
    if needle.old.len() == 1 && needle.keywords.len() >= 2 {
        let from = i.saturating_sub(2);
        let to = (i + 2).min(lines.len() - 1);
        let mut seen: BTreeSet<&String> = BTreeSet::new();
        for text in &lines[from..=to] {
            for word in text.split(|c: char| !(c.is_alphanumeric() || c == '\'')) {
                if let Some(keyword) = needle.keywords.iter().find(|k| **k == norm_word(word)) {
                    seen.insert(keyword);
                }
            }
        }
        return seen.len() >= 2;
    }
    true
}

/// The lines of the text and a lookup from word runs to lines, for the claim search.
fn word_lines(lines: &[&str]) -> Vec<(String, usize)> {
    lines
        .iter()
        .enumerate()
        .flat_map(|(i, l)| plain_words(l).into_iter().map(move |w| (w, i)))
        .collect()
}

/// The first line of each place where `words` stand in a row in `text`.
fn claim_lines(words: &[String], text: &[(String, usize)]) -> Vec<usize> {
    let mut lines: Vec<usize> = Vec::new();
    let mut k = 0;
    while k + words.len() <= text.len() {
        if text[k].0 == words[0] && (0..words.len()).all(|n| text[k + n].0 == words[n]) {
            lines.push(text[k].1);
            k += words.len();
        } else {
            k += 1;
        }
    }
    lines
}

/// The marks that exempt a place and the lines the diff added.
struct Exempt<'a> {
    added: BTreeMap<&'a str, BTreeSet<String>>,
    entries: Vec<Entry>,
}

impl<'a> Exempt<'a> {
    fn new(input: &'a Input) -> Self {
        let mut added: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
        for hunk in &input.hunks {
            added
                .entry(hunk.path.as_str())
                .or_default()
                .extend(hunk.added.iter().map(|l| l.trim().to_string()));
        }
        let marked = link_marked(&input.links, &input.base_headings);
        let entries = input
            .files
            .get(DECISION_LOG)
            .map(|log| log_entries(log, &marked))
            .unwrap_or_default();
        Exempt { added, entries }
    }

    fn added(&self, path: &str, line: &str) -> bool {
        self.added
            .get(path)
            .is_some_and(|set| set.contains(line.trim()))
    }

    /// `None` when the place is exempt; otherwise the decision whose entry holds it, if any.
    fn place(&self, path: &str, lines: &[&str], i: usize) -> Option<Option<String>> {
        if self.added(path, lines[i]) {
            return None;
        }
        if path == DECISION_LOG
            && let Some(entry) = self.entries.iter().find(|e| e.first <= i + 1 && i < e.last)
        {
            return (!entry.marked).then(|| Some(entry.id.clone()));
        }
        (!paragraph_marked(lines, i)).then_some(None)
    }
}

fn shorten(line: &str) -> String {
    let line = line.trim();
    if line.chars().count() <= 140 {
        return line.to_string();
    }
    let cut: String = line.chars().take(137).collect();
    format!("{cut}...")
}

/// Each place of the searched files that still says what the change made false, split into firm findings and
/// advisory ones. A value that matches more than [`FIRM_VALUE_HITS`] places is too common to be a finding: its
/// hits are advisory. Claims and decisions are always firm.
pub(crate) fn find(input: &Input) -> Report {
    search(&needles(&input.hunks, &input.reading), input)
}

fn search(needles: &Needles, input: &Input) -> Report {
    let mut report = Report::default();
    if needles.is_empty() {
        return report;
    }
    let exempt = Exempt::new(input);
    let files: Vec<(&String, Vec<&str>)> = input
        .files
        .iter()
        .filter(|(path, _)| searchable(path))
        .map(|(path, text)| (path, text.lines().collect()))
        .collect();
    let stale = |path: &str, lines: &[&str], i: usize, decision, what: &str, why: &str| Stale {
        path: path.to_string(),
        line: i + 1,
        decision,
        what: what.to_string(),
        why: why.to_string(),
        text: shorten(lines[i]),
    };
    let mut firm: Vec<Stale> = Vec::new();
    for needle in &needles.values {
        let mut hits: Vec<Stale> = Vec::new();
        for (path, lines) in &files {
            for i in 0..lines.len() {
                if !value_hit(needle, lines, i) {
                    continue;
                }
                if let Some(decision) = exempt.place(path, lines, i) {
                    hits.push(stale(path, lines, i, decision, &needle.old, &needle.why));
                }
            }
        }
        if hits.len() > FIRM_VALUE_HITS {
            report.advisory.extend(hits);
        } else {
            firm.extend(hits);
        }
    }
    if !needles.claims.is_empty() {
        for (path, lines) in &files {
            let words = word_lines(lines);
            for claim in &needles.claims {
                for i in claim_lines(&claim.words, &words) {
                    if let Some(decision) = exempt.place(path, lines, i) {
                        let what = format!("\"{}\"", claim.words.join(" "));
                        firm.push(stale(path, lines, i, decision, &what, &claim.why));
                    }
                }
            }
        }
    }
    if let Some((_, lines)) = files.iter().find(|(path, _)| path.as_str() == DECISION_LOG) {
        for needle in &needles.decisions {
            if !input.base_headings.contains(&needle.id) {
                continue;
            }
            let Some(i) = lines
                .iter()
                .position(|l| heading_id(l).as_deref() == Some(needle.id.as_str()))
            else {
                continue;
            };
            let marked = exempt.entries.iter().any(|e| e.id == needle.id && e.marked);
            if !marked && !exempt.added(DECISION_LOG, lines[i]) {
                let id = Some(needle.id.clone());
                firm.push(stale(DECISION_LOG, lines, i, id, &needle.id, &needle.why));
            }
        }
    }
    for list in [&mut firm, &mut report.advisory] {
        list.sort_by(|a, b| (&a.path, a.line, &a.what).cmp(&(&b.path, b.line, &b.what)));
        list.dedup_by(|a, b| a.path == b.path && a.line == b.line && a.what == b.what);
    }
    report.firm = firm;
    report
}

// ---------------------------------------------------------------------------------------------------------
// Reading git
// ---------------------------------------------------------------------------------------------------------

/// The hunks of the diff between two trees or commits, with no context lines.
pub(crate) fn read_diff(git: &Git, from: &str, to: &str) -> Result<Vec<Hunk>> {
    let out = git.run_bytes(
        [
            "diff",
            "-U0",
            "--no-color",
            "--no-renames",
            "--no-ext-diff",
            from,
            to,
        ],
        None,
        &[],
    )?;
    Ok(parse_diff(&String::from_utf8_lossy(&out)))
}

/// Hunks from the text of `git diff -U0`. A `+++` or `---` line is a header only before a file's first hunk.
fn parse_diff(diff: &str) -> Vec<Hunk> {
    let mut hunks: Vec<Hunk> = Vec::new();
    let (mut old_path, mut new_path) = (String::new(), String::new());
    let mut in_hunk = false;
    for line in diff.lines() {
        if line.starts_with("diff --git ") {
            in_hunk = false;
            (old_path, new_path) = (String::new(), String::new());
        } else if line.starts_with("@@") {
            in_hunk = true;
            let path = if new_path.is_empty() {
                &old_path
            } else {
                &new_path
            };
            hunks.push(Hunk {
                path: path.clone(),
                ..Hunk::default()
            });
        } else if in_hunk {
            let Some(hunk) = hunks.last_mut() else {
                continue;
            };
            if let Some(text) = line.strip_prefix('-') {
                hunk.removed.push(text.trim_end_matches('\r').to_string());
            } else if let Some(text) = line.strip_prefix('+') {
                hunk.added.push(text.trim_end_matches('\r').to_string());
            }
        } else if let Some(path) = line.strip_prefix("--- ") {
            old_path = path.strip_prefix("a/").unwrap_or("").to_string();
        } else if let Some(path) = line.strip_prefix("+++ ") {
            new_path = path.strip_prefix("b/").unwrap_or("").to_string();
        }
    }
    hunks
}

/// The searched files of a tree, as text: the extensions of [`SEARCHED_EXTENSIONS`], at most 2 MiB each, and
/// never `CHANGELOG.md`, `records/`, `target/` or `.git/`.
pub(crate) fn read_files(git: &Git, tree: &str) -> Result<BTreeMap<String, String>> {
    let entries: Vec<_> = git
        .ls_tree(tree)?
        .into_iter()
        .filter(|e| e.kind == "blob" && e.mode != "120000" && searchable(&e.path))
        .collect();
    let mut files = BTreeMap::new();
    for chunk in entries.chunks(BLOB_CHUNK) {
        let oids: Vec<String> = chunk.iter().map(|e| e.oid.clone()).collect();
        let sizes = git.blob_sizes(&oids)?;
        let small: Vec<_> = chunk
            .iter()
            .filter(|e| sizes.get(&e.oid).is_some_and(|s| *s <= MAX_FILE_BYTES))
            .collect();
        let small_oids: Vec<String> = small.iter().map(|e| e.oid.clone()).collect();
        for (entry, bytes) in small.iter().zip(git.cat_blobs(&small_oids)?) {
            files.insert(
                entry.path.clone(),
                String::from_utf8_lossy(&bytes).into_owned(),
            );
        }
    }
    Ok(files)
}

/// The check on a change: the diff from `from` to `to` (a commit or a tree of `git`) and the reading's
/// sentences give the old text; the tree `to` is searched for it. With no old text nothing else is read.
pub(crate) fn check(git: &Git, from: &str, to: &str, reading: &[String]) -> Result<Report> {
    let hunks = read_diff(git, from, to)?;
    let found = needles(&hunks, reading);
    if found.is_empty() {
        return Ok(Report::default());
    }
    let files = read_files(git, to)?;
    let base_headings = git
        .show_file(from, DECISION_LOG)?
        .map(|bytes| headings(&String::from_utf8_lossy(&bytes)))
        .unwrap_or_default();
    let links = files.get(DECISION_LINKS).cloned().unwrap_or_default();
    Ok(find(&Input {
        files,
        hunks,
        reading: reading.to_vec(),
        base_headings,
        links,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hunk(path: &str, removed: &[&str], added: &[&str]) -> Hunk {
        Hunk {
            path: path.to_string(),
            removed: removed.iter().map(|s| s.to_string()).collect(),
            added: added.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn files(list: &[(&str, &str)]) -> BTreeMap<String, String> {
        list.iter()
            .map(|(p, t)| (p.to_string(), t.to_string()))
            .collect()
    }

    fn sentences(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn headings_of(ids: &[&str]) -> BTreeSet<String> {
        ids.iter().map(|s| s.to_string()).collect()
    }

    const LIMIT_OLD: &str = "pub const MAX_WIDGETS: u32 = 8;";
    const LIMIT_NEW: &str = "pub const MAX_WIDGETS: u32 = 16;";

    /// The constant of the diff changes from 8 to 16 while the help text still says 8.
    fn widgets(help: &str) -> Input {
        Input {
            files: files(&[
                (
                    "crates/core/src/limits.rs",
                    "//! Limits.\n\npub const MAX_WIDGETS: u32 = 16;\n",
                ),
                ("crates/cli/src/main.rs", help),
            ]),
            hunks: vec![hunk(
                "crates/core/src/limits.rs",
                &[LIMIT_OLD],
                &[LIMIT_NEW],
            )],
            ..Input::default()
        }
    }

    const HELP_STALE: &str =
        "/// Lists the jobs.\n/// Runs at most 8 widgets per job.\n/// Takes 8 seconds.\n";

    #[test]
    fn a_constant_raised_from_8_to_16_names_the_help_line_still_saying_8() {
        let report = find(&widgets(HELP_STALE));
        assert_eq!(report.firm.len(), 1, "{:?}", report.firm);
        let stale = &report.firm[0];
        assert_eq!(stale.path, "crates/cli/src/main.rs");
        assert_eq!(stale.line, 2);
        assert_eq!(stale.what, "8");
        assert!(report.advisory.is_empty());
        let text = stale.to_string();
        assert!(text.contains("crates/cli/src/main.rs:2"), "{text}");
        assert!(text.contains("Runs at most 8 widgets per job"), "{text}");
        assert!(
            text.contains("`MAX_WIDGETS` changed from 8 to 16"),
            "{text}"
        );
    }

    #[test]
    fn a_help_line_that_follows_the_new_value_is_quiet() {
        let help = HELP_STALE.replace("at most 8 widgets", "at most 16 widgets");
        let report = find(&widgets(&help));
        assert!(report.firm.is_empty(), "{:?}", report.firm);
    }

    #[test]
    fn a_repeat_urgent_jobs_change_leaves_unrelated_3_jobs_lines_quiet() {
        let input = Input {
            files: files(&[
                ("docs/ops.md", "The batch ran 3 jobs overnight.\n"),
                ("crates/a/src/tests.rs", "    assert_eq!(jobs.len(), 3);\n"),
                ("crates/a/src/other.rs", "// Retries 3 times.\n"),
                (
                    "crates/a/src/plan.rs",
                    "/// A cause recorded in 3 or more distinct jobs is raised to urgent for the plan.\n",
                ),
            ]),
            hunks: vec![hunk(
                "crates/a/src/plan.rs",
                &["pub const REPEAT_URGENT_JOBS: usize = 3;"],
                &["pub const REPEAT_URGENT_JOBS: usize = 4;"],
            )],
            ..Input::default()
        };
        let report = find(&input);
        let paths: Vec<&str> = report.firm.iter().map(|s| s.path.as_str()).collect();
        assert_eq!(paths, ["crates/a/src/plan.rs"], "{:?}", report.firm);
    }

    #[test]
    fn a_value_that_matches_many_places_is_advisory_only() {
        let mut input = widgets("");
        for n in 0..7 {
            input.files.insert(
                format!("docs/page{n}.md"),
                "Each job runs at most 8 widgets.\n".to_string(),
            );
        }
        let report = find(&input);
        assert!(report.firm.is_empty(), "{:?}", report.firm);
        assert_eq!(report.advisory.len(), 7);
        // Six places are still firm.
        input.files.remove("docs/page0.md");
        let report = find(&input);
        assert_eq!(report.firm.len(), 6);
        assert!(report.advisory.is_empty());
    }

    #[test]
    fn a_changed_constant_with_no_shared_keyword_is_not_reported() {
        let report = find(&widgets("/// Waits 8 minutes for the build.\n"));
        assert!(report.firm.is_empty(), "{:?}", report.firm);
    }

    #[test]
    fn code_and_a_digit_inside_a_longer_number_are_not_prose() {
        let help = "let widgets = 8;\nassert_eq!(widgets.len(), 8);\n/// Up to 18 widgets or 8.5 widgets.\n";
        let report = find(&widgets(help));
        assert!(report.firm.is_empty(), "{:?}", report.firm);
    }

    #[test]
    fn a_constant_with_two_literals_is_skipped() {
        let mut input = widgets(HELP_STALE);
        input.hunks = vec![hunk(
            "crates/core/src/limits.rs",
            &["pub const MAX_WIDGETS: (u32, u32) = (8, 9);"],
            &["pub const MAX_WIDGETS: (u32, u32) = (16, 9);"],
        )];
        assert!(find(&input).firm.is_empty());
    }

    #[test]
    fn a_constant_with_only_generic_name_words_is_skipped() {
        let mut input = widgets(HELP_STALE);
        input.hunks = vec![hunk(
            "crates/core/src/limits.rs",
            &["pub const MAX_VALUE: u32 = 8;"],
            &["pub const MAX_VALUE: u32 = 16;"],
        )];
        assert!(find(&input).firm.is_empty());
        assert!(keywords_of("MAX_VALUE").is_empty());
        assert_eq!(
            keywords_of("REPEAT_URGENT_JOBS"),
            ["repeat", "urgent", "job"]
        );
    }

    #[test]
    fn a_small_value_beside_one_of_two_name_words_is_not_reported() {
        // The stated miss: the name has two words and a single digit needs both near the line.
        let input = Input {
            files: files(&[("docs/a.md", "Raised after 3 jobs.\n")]),
            hunks: vec![hunk(
                "crates/a/src/plan.rs",
                &["const REPEAT_URGENT_JOBS: usize = 3;"],
                &["const REPEAT_URGENT_JOBS: usize = 4;"],
            )],
            ..Input::default()
        };
        assert!(find(&input).firm.is_empty());
    }

    #[test]
    fn a_string_constant_is_found_in_prose() {
        let input = Input {
            files: files(&[(
                "docs/a.md",
                "The branch is named athena-main for records.\n",
            )]),
            hunks: vec![hunk(
                "crates/a/src/lib.rs",
                &["pub const RECORDS_BRANCH: &str = \"athena-main\";"],
                &["pub const RECORDS_BRANCH: &str = \"athena-records\";"],
            )],
            ..Input::default()
        };
        let report = find(&input);
        assert_eq!(report.firm.len(), 1, "{:?}", report.firm);
        assert_eq!(report.firm[0].what, "athena-main");
    }

    // The decision log.

    const OLD_RULE: &str = "A retry keeps the work of the failed attempt for the next producer.";

    fn retry_input(log: &str, links: &str, base: &[&str]) -> Input {
        Input {
            files: files(&[
                (
                    "crates/core/src/retry.rs",
                    "/// A retry discards the work of the failed attempt and starts from the base.\n",
                ),
                (DECISION_LOG, log),
            ]),
            hunks: vec![hunk(
                "crates/core/src/retry.rs",
                &[format!("/// {OLD_RULE}").as_str()],
                &["/// A retry discards the work of the failed attempt and starts from the base."],
            )],
            reading: Vec::new(),
            base_headings: headings_of(base),
            links: links.to_string(),
        }
    }

    const LOG: &str = "# Decisions\n\n## D45 · Retries\n\n- **Decision:** A retry keeps the work of the failed\n  attempt for the next producer.\n\n## D46 · Other\n\n- **Decision:** Unrelated.\n";

    #[test]
    fn a_decision_that_says_the_removed_rule_and_is_not_marked_is_named_with_its_id_and_line() {
        let report = find(&retry_input(LOG, "", &["D45", "D46"]));
        assert_eq!(report.firm.len(), 1, "{:?}", report.firm);
        let stale = &report.firm[0];
        assert_eq!(stale.path, DECISION_LOG);
        assert_eq!(stale.decision.as_deref(), Some("D45"));
        assert_eq!(stale.line, 5);
        let text = stale.to_string();
        assert!(text.contains("docs/decisions.md:5 (D45)"), "{text}");
    }

    #[test]
    fn a_decision_marked_superseded_by_or_amended_by_an_id_is_not_reported() {
        for phrase in ["(superseded by D99)", "Amended by D99."] {
            let log = LOG.replace(
                "## D45 · Retries\n",
                &format!("## D45 · Retries\n\n{phrase}\n"),
            );
            let report = find(&retry_input(&log, "", &["D45", "D46", "D99"]));
            assert!(report.firm.is_empty(), "{phrase}: {:?}", report.firm);
        }
    }

    #[test]
    fn amended_by_hand_without_an_id_does_not_mark_an_entry() {
        let log = LOG.replace(
            "## D45 · Retries\n",
            "## D45 · Retries\n\nThis list is amended by hand.\n",
        );
        let report = find(&retry_input(&log, "", &["D45", "D46"]));
        assert_eq!(report.firm.len(), 1, "{:?}", report.firm);
    }

    #[test]
    fn a_superseded_link_or_a_link_to_a_new_entry_marks_the_older_decision() {
        let superseded = "[[link]]\nolder = \"D45\"\nnewer = \"D60\"\nstatus = \"superseded\"\n";
        let report = find(&retry_input(LOG, superseded, &["D45", "D46", "D60"]));
        assert!(report.firm.is_empty(), "{:?}", report.firm);
        // An amended link whose newer entry is added by this change: the change reconciles it itself.
        let amended = "[[link]]\nolder = \"D45\"\nnewer = \"D60\"\nstatus = \"amended\"\n";
        let report = find(&retry_input(LOG, amended, &["D45", "D46"]));
        assert!(report.firm.is_empty(), "{:?}", report.firm);
    }

    #[test]
    fn a_decision_an_older_entry_amended_still_stands_and_is_reported() {
        let amended = "[[link]]\nolder = \"D45\"\nnewer = \"D60\"\nstatus = \"amended\"\n";
        let report = find(&retry_input(LOG, amended, &["D45", "D46", "D60"]));
        assert_eq!(report.firm.len(), 1, "{:?}", report.firm);
        assert_eq!(report.firm[0].decision.as_deref(), Some("D45"));
        // A list that does not parse marks nothing.
        let report = find(&retry_input(LOG, "[[link", &["D45", "D46"]));
        assert_eq!(report.firm.len(), 1);
    }

    #[test]
    fn a_rule_changed_only_in_the_reading_is_found_without_any_removed_line() {
        let reading = sentences(&[
            "Raise `MAX_WIDGETS` from 8 to 16 in HELP.md",
            "A retry no longer \"keeps the work of the failed attempt for the next producer\"",
            "This reverses D45",
        ]);
        let input = Input {
            files: files(&[
                ("README.md", "Intro.\n"),
                ("HELP.md", "At most 8 widgets run.\n"),
                (DECISION_LOG, LOG),
            ]),
            hunks: vec![hunk("README.md", &[], &["Intro."])],
            reading,
            base_headings: headings_of(&["D45", "D46"]),
            links: String::new(),
        };
        let report = find(&input);
        let places: Vec<(String, usize)> = report
            .firm
            .iter()
            .map(|s| (s.path.clone(), s.line))
            .collect();
        assert!(places.contains(&("HELP.md".to_string(), 1)), "{places:?}");
        assert!(
            places.contains(&(DECISION_LOG.to_string(), 3)),
            "{places:?}"
        );
        assert!(
            places.contains(&(DECISION_LOG.to_string(), 5)),
            "{places:?}"
        );
        // The same reading is quiet once the change adds an entry that amends the decision, with a link.
        let mut fixed = input.clone();
        fixed.files.insert(
            DECISION_LOG.to_string(),
            format!("{LOG}\n## D60 · New rule\n\n- **Decision:** Amends D45.\n"),
        );
        fixed.files.insert(
            "HELP.md".to_string(),
            "At most 16 widgets run.\n".to_string(),
        );
        fixed.links =
            "[[link]]\nolder = \"D45\"\nnewer = \"D60\"\nstatus = \"amended\"\n".to_string();
        let report = find(&fixed);
        assert!(report.firm.is_empty(), "{:?}", report.firm);
    }

    #[test]
    fn reading_forms_that_state_no_reversal_make_no_needle() {
        for text in [
            "D156's placeholder replaces the literal",
            "This follows D156",
            "It moves `records` from main to athena/records",
            "Diff `Snapshot` from the base to the candidate",
            "A constant changes from 8 to 16",
            "A cause moves \"a\" from 2 to 3 here",
            "See D156 for the rule, which no longer is a problem",
            "Raise `MAX_VALUE` from 8 to 16",
        ] {
            let found = needles(&[], &sentences(&[text]));
            assert!(found.is_empty(), "{text}: {:?}", found.values);
        }
        let found = needles(&[], &sentences(&["Raise MAX_WIDGETS from 8 to 16"]));
        assert_eq!(found.values.len(), 1);
        assert_eq!(found.values[0].old, "8");
        let found = needles(
            &[],
            &sentences(&["Set `max_widgets` from \"low\" to \"high\""]),
        );
        assert_eq!(found.values.len(), 1);
        assert_eq!(found.values[0].old, "low");
    }

    #[test]
    fn reversal_verbs_and_phrases_name_the_decision_they_act_on() {
        for text in [
            "This reverses D45",
            "It supersedes decision D45",
            "Reverting is not it, but it overturns D45",
            "D45 no longer holds",
            "D45's rule is superseded here, and D45 is reversed",
        ] {
            let found = needles(&[], &sentences(&[text]));
            let ids: Vec<&str> = found.decisions.iter().map(|d| d.id.as_str()).collect();
            assert_eq!(ids, ["D45"], "{text}");
        }
        let found = needles(
            &[],
            &sentences(&["The marker reverses nothing", "Reverts to D1"]),
        );
        assert!(found.decisions.is_empty());
    }

    #[test]
    fn a_quoted_span_of_fewer_than_six_words_is_no_claim() {
        let found = needles(&[], &sentences(&["It no longer \"keeps the work\""]));
        assert!(found.claims.is_empty());
        for marker in CLAIM_MARKERS {
            let text = format!("It {marker} `a retry keeps the work of the failed attempt`");
            let found = needles(&[], &sentences(&[&text]));
            assert_eq!(found.claims.len(), 1, "{marker}");
        }
    }

    #[test]
    fn a_decision_id_missing_from_the_base_log_is_ignored() {
        let mut input = retry_input(LOG, "", &["D46"]);
        input.reading = sentences(&["This reverses D45"]);
        let report = find(&input);
        // Only the claim from the replaced hunk is found; the id is not a base heading.
        assert!(
            report.firm.iter().all(|s| s.what != "D45"),
            "{:?}",
            report.firm
        );
    }

    // After the text is updated, and what is exempt.

    #[test]
    fn both_fixtures_pass_once_the_text_is_updated_and_exempt_places_are_not_reported() {
        // Criterion 1 fixture, updated.
        let updated = HELP_STALE.replace("at most 8 widgets", "at most 16 widgets");
        let mut input = widgets(&updated);
        assert!(find(&input).firm.is_empty());
        // Criterion 2 fixture, updated: the decision entry holds the new rule (lines the diff added).
        let log_updated = LOG.replace(
            "A retry keeps the work of the failed\n  attempt for the next producer.",
            "A retry discards the work of the failed attempt and starts from the base.",
        );
        let mut retry = retry_input(&log_updated, "", &["D45", "D46"]);
        assert!(find(&retry).firm.is_empty(), "{:?}", find(&retry).firm);
        // A stale line the diff itself added is not reported.
        input.files.insert(
            "docs/new.md".to_string(),
            "Runs at most 8 widgets.\n".to_string(),
        );
        assert_eq!(find(&input).firm.len(), 1);
        input
            .hunks
            .push(hunk("docs/new.md", &[], &["Runs at most 8 widgets."]));
        assert!(find(&input).firm.is_empty());
        // A stale line in a marked entry is not reported, and neither is one in CHANGELOG.md.
        retry.files.insert(
            DECISION_LOG.to_string(),
            LOG.replace("## D46 · Other", "(superseded by D46)\n\n## D46 · Other"),
        );
        assert!(find(&retry).firm.is_empty());
        retry.files.insert(
            "CHANGELOG.md".to_string(),
            format!("- {OLD_RULE}\n- Runs at most 8 widgets.\n"),
        );
        assert!(find(&retry).firm.is_empty());
        input.files.insert(
            "CHANGELOG.md".to_string(),
            "- Runs at most 8 widgets.\n".to_string(),
        );
        assert!(find(&input).firm.is_empty());
    }

    #[test]
    fn a_paragraph_marked_outside_the_log_is_not_reported() {
        let mut input = widgets("");
        for (mark, quiet) in [
            ("Superseded by D77.", true),
            ("Amended by `limits`.", true),
            ("Amended by docs/limits.md.", true),
            ("Amended by hand.", false),
        ] {
            input.files.insert(
                "docs/page.md".to_string(),
                format!("Runs at most 8 widgets.\n{mark}\n\nRuns at most 8 widgets again.\n"),
            );
            let report = find(&input);
            let lines: Vec<usize> = report.firm.iter().map(|s| s.line).collect();
            assert_eq!(lines, if quiet { vec![4] } else { vec![1, 4] }, "{mark}");
        }
    }

    // Claims of replaced hunks.

    #[test]
    fn a_deleted_function_does_not_make_a_claim() {
        let hunks = vec![hunk(
            "crates/a/src/lib.rs",
            &[
                "/// A retry keeps the work of the failed attempt for the next producer.",
                "pub fn keep() {}",
            ],
            &[],
        )];
        assert!(needles(&hunks, &[]).is_empty());
    }

    #[test]
    fn a_deleted_file_does_not_make_a_claim() {
        let diff = "diff --git a/docs/old.md b/docs/old.md\ndeleted file mode 100644\nindex 1..0\n--- a/docs/old.md\n+++ /dev/null\n@@ -1,2 +0,0 @@\n-A retry keeps the work of the failed attempt for the next producer.\n-More text here.\n";
        let hunks = parse_diff(diff);
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].path, "docs/old.md");
        assert!(needles(&hunks, &[]).is_empty());
    }

    #[test]
    fn a_moved_comment_is_not_a_claim() {
        let comment = "/// A retry keeps the work of the failed attempt for the next producer.";
        let hunks = vec![
            hunk("crates/a/src/a.rs", &[comment, "fn a() {}"], &["fn b() {}"]),
            hunk("crates/a/src/b.rs", &[], &[comment]),
        ];
        assert!(needles(&hunks, &[]).claims.is_empty());
    }

    #[test]
    fn a_short_replaced_line_and_a_table_row_are_not_claims() {
        let hunks = vec![
            hunk(
                "docs/a.md",
                &["Takes eight seconds."],
                &["Takes nine seconds."],
            ),
            hunk(
                "docs/b.md",
                &["| one | two | three | four | five | six |"],
                &["| 1 | 2 | 3 | 4 | 5 | 6 |"],
            ),
            hunk(
                "crates/a/src/a.rs",
                &["let a_long_line_of_code = call(one, two, three, four, five, six);"],
                &["x"],
            ),
        ];
        assert!(needles(&hunks, &[]).claims.is_empty());
    }

    #[test]
    fn a_claim_wrapped_over_two_lines_is_found_at_its_first_line() {
        let report = find(&retry_input(LOG, "", &["D45", "D46"]));
        assert_eq!(report.firm[0].line, 5);
        assert!(report.firm[0].what.starts_with("\"a retry keeps the work"));
    }

    #[test]
    fn the_diff_text_is_read_into_hunks_per_file() {
        let diff = "diff --git a/a.rs b/a.rs\nindex 1..2 100644\n--- a/a.rs\n+++ b/a.rs\n@@ -3 +3 @@\n--- a removed line that starts with dashes\n+pub const MAX_WIDGETS: u32 = 16;\n@@ -9,0 +10 @@\n+// new\ndiff --git a/b.md b/b.md\n--- a/b.md\n+++ b/b.md\n@@ -1 +1 @@\n-old\n+new\n";
        let hunks = parse_diff(diff);
        assert_eq!(hunks.len(), 3);
        assert_eq!(hunks[0].path, "a.rs");
        assert_eq!(
            hunks[0].removed,
            ["-- a removed line that starts with dashes"]
        );
        assert_eq!(hunks[0].added, ["pub const MAX_WIDGETS: u32 = 16;"]);
        assert_eq!(hunks[1].added, ["// new"]);
        assert_eq!(hunks[2].path, "b.md");
    }

    // Reading git.

    fn repo(files: &[(&str, &str)]) -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        write(dir.path(), files);
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "base"]).unwrap();
        (dir, git)
    }

    fn write(root: &std::path::Path, files: &[(&str, &str)]) {
        for (path, text) in files {
            let full = root.join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, text).unwrap();
        }
    }

    #[test]
    fn read_files_leaves_out_changelog_records_target_and_large_files() {
        let big = "Runs at most 8 widgets.\n".repeat(100_000);
        assert!(big.len() as u64 > MAX_FILE_BYTES);
        let (_dir, git) = repo(&[
            ("CHANGELOG.md", "Runs at most 8 widgets.\n"),
            ("records/x.md", "Runs at most 8 widgets.\n"),
            ("target/y.md", "Runs at most 8 widgets.\n"),
            ("docs/big.md", &big),
            ("docs/image.png", "Runs at most 8 widgets.\n"),
            ("docs/kept.md", "Runs at most 8 widgets.\n"),
            ("src/lib.rs", "// text\n"),
        ]);
        let files = read_files(&git, "HEAD").unwrap();
        let paths: Vec<&str> = files.keys().map(String::as_str).collect();
        assert_eq!(paths, ["docs/kept.md", "src/lib.rs"]);
    }

    #[test]
    fn check_reads_the_diff_and_the_tree_and_names_the_help_line() {
        let (dir, git) = repo(&[
            ("src/limits.rs", &format!("{LIMIT_OLD}\n")),
            ("HELP.md", "Runs at most 8 widgets.\n"),
            ("CHANGELOG.md", "Runs at most 8 widgets.\n"),
        ]);
        let base = git.head().unwrap().unwrap();
        write(dir.path(), &[("src/limits.rs", &format!("{LIMIT_NEW}\n"))]);
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "raise"]).unwrap();
        let head = git.head().unwrap().unwrap();
        let report = check(&git, &base, &head, &[]).unwrap();
        assert_eq!(report.firm.len(), 1, "{:?}", report.firm);
        assert_eq!(report.firm[0].path, "HELP.md");
        assert_eq!(report.firm[0].line, 1);
        // Once the help text is updated, nothing is left.
        write(dir.path(), &[("HELP.md", "Runs at most 16 widgets.\n")]);
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "help"]).unwrap();
        let head = git.head().unwrap().unwrap();
        assert!(check(&git, &base, &head, &[]).unwrap().firm.is_empty());
    }

    #[test]
    fn check_reads_nothing_more_when_the_change_states_no_old_value() {
        let (dir, git) = repo(&[("README.md", "Intro.\n")]);
        let base = git.head().unwrap().unwrap();
        write(dir.path(), &[("README.md", "Intro, longer.\n")]);
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "edit"]).unwrap();
        let head = git.head().unwrap().unwrap();
        let report = check(&git, &base, &head, &[]).unwrap();
        assert!(report.firm.is_empty() && report.advisory.is_empty());
    }

    #[test]
    fn sentences_split_at_full_stops_semicolons_and_line_ends() {
        let found = reading_sentences(
            "One. Two; three\nfour",
            &["Five".to_string()],
            &["Six. Seven".to_string()],
        );
        assert_eq!(
            found,
            ["One", "Two", "three", "four", "Five", "Six", "Seven"]
        );
    }
}
