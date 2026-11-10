//! The request-coverage check: every acceptance bullet of the request is covered by the reading's criteria.

use super::*;

/// Words that carry no meaning of their own in a bullet; they never count toward coverage.
const STOP_WORDS: [&str; 28] = [
    "that", "the", "and", "with", "for", "from", "into", "them", "all", "one", "any", "are", "has",
    "have", "not", "its", "this", "but", "only", "each", "every", "also", "when", "then", "than",
    "was", "were", "will",
];

/// Words that give permission in a sentence of the request.
const PERMISSION_WORDS: [&str; 10] = [
    "may",
    "can",
    "could",
    "allowed",
    "permitted",
    "free",
    "ok",
    "okay",
    "fine",
    "acceptable",
];

/// Words that turn a permission into a prohibition or a condition.
const NEGATING_WORDS: [&str; 11] = [
    "not",
    "never",
    "no",
    "only",
    "unless",
    "if",
    "without",
    "must",
    "cannot",
    "forbid",
    "forbidden",
];

/// Words that name the thing that would be split.
const JOB_WORDS: [&str; 6] = ["job", "jobs", "work", "request", "change", "task"];

/// How far, in characters, a job word may lie from the word `split` to belong to it.
const JOB_WORD_REACH: usize = 40;

/// The acceptance bullets of a request and the request's other text.
pub(super) struct ParsedRequest {
    /// The body of each bullet, marker removed, a wrapped continuation line joined with one space.
    pub(super) bullets: Vec<String>,
    /// The trimmed lines that are not part of a bullet and not a list line.
    pub(super) outside: Vec<String>,
}

enum Section {
    Before,
    In,
    After,
}

/// Whether a trimmed line is an `Acceptance` or `Acceptance criteria` heading, with optional `#`, `*` and `_`
/// marks and a colon.
fn is_acceptance_heading(line: &str) -> bool {
    let name = line
        .trim_start_matches(['#', '*', '_', ' '])
        .trim_end_matches([':', '*', '_', ' '])
        .to_lowercase();
    name == "acceptance" || name == "acceptance criteria"
}

/// The text of a bullet line without its marker (`- `, `* ` or `N. `).
fn bullet_body(line: &str) -> String {
    if let Some(rest) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
        return rest.trim().to_string();
    }
    line.split_once(". ")
        .map_or(line, |(_, rest)| rest)
        .trim()
        .to_string()
}

/// Reads the bullets under the first `Acceptance` heading, up to the first blank line after a bullet or a
/// `Files:` line. A non-bullet line right after a bullet is its wrapped continuation. No heading, no bullets.
pub(super) fn parse_request(request: &str) -> ParsedRequest {
    let mut bullets: Vec<String> = Vec::new();
    let mut outside: Vec<String> = Vec::new();
    let mut section = Section::Before;
    for line in request.lines().map(str::trim) {
        match section {
            Section::Before => {
                if is_acceptance_heading(line) {
                    section = Section::In;
                }
                outside.push(line.to_string());
            }
            Section::In => {
                if line.is_empty() {
                    if !bullets.is_empty() {
                        section = Section::After;
                    }
                } else if line.to_lowercase().starts_with("files:") {
                    section = Section::After;
                    outside.push(line.to_string());
                } else if is_bullet(line) {
                    bullets.push(bullet_body(line));
                } else if let Some(last) = bullets.last_mut() {
                    last.push(' ');
                    last.push_str(line);
                } else {
                    outside.push(line.to_string());
                }
            }
            Section::After => {
                if !is_bullet(line) {
                    outside.push(line.to_string());
                }
            }
        }
    }
    outside.retain(|l| !is_bullet(l));
    ParsedRequest { bullets, outside }
}

/// Whether one sentence gives permission to split the job: a permission word, the word `split`, a job word
/// near it, and no negating or conditional word.
fn sentence_allows_split(sentence: &str) -> bool {
    let lower = sentence
        .to_lowercase()
        .replace(['\u{2019}', '\u{2018}'], "'");
    let cleaned: String = lower
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '\'' {
                c
            } else {
                ' '
            }
        })
        .collect();
    let words: Vec<(usize, &str)> = cleaned
        .split(' ')
        .scan(0usize, |at, w| {
            let start = *at;
            *at += w.len() + 1;
            Some((start, w))
        })
        .filter(|(_, w)| !w.is_empty())
        .collect();
    if words
        .iter()
        .any(|(_, w)| NEGATING_WORDS.contains(w) || w.ends_with("n't"))
    {
        return false;
    }
    let Some(&(split_at, _)) = words
        .iter()
        .find(|(_, w)| matches!(*w, "split" | "splits" | "splitting"))
    else {
        return false;
    };
    words.iter().any(|(_, w)| PERMISSION_WORDS.contains(w))
        && words
            .iter()
            .any(|(at, w)| JOB_WORDS.contains(w) && at.abs_diff(split_at) <= JOB_WORD_REACH)
}

/// Whether the request's own text, outside its bullets, affirmatively allows splitting the job.
pub(super) fn allows_split(outside: &[String]) -> bool {
    outside
        .join("\n")
        .split(['.', '!', '?', ';', ':', '\n'])
        .any(sentence_allows_split)
}

/// The pieces of a distinctive token: its path segments and the parts of each segment around dots.
fn token_pieces(token: &str) -> Vec<&str> {
    let mut pieces: Vec<&str> = Vec::new();
    for segment in token.split(['/', '-', '_']) {
        pieces.push(segment);
        pieces.extend(segment.split('.'));
    }
    pieces
}

/// Whether `token` is a path, a `--flag` or a file name.
fn is_distinctive(token: &str) -> bool {
    if token.starts_with("--") {
        return token.len() > 2;
    }
    if token.contains('/') {
        return token.contains(['.', '-', '_'])
            || token.ends_with('/')
            || token.matches('/').count() >= 2;
    }
    token.rsplit_once('.').is_some_and(|(first, last)| {
        (2..=5).contains(&last.len())
            && last.chars().all(|c| c.is_ascii_alphabetic())
            && first.chars().count() >= 2
            && first.chars().any(char::is_alphabetic)
    })
}

/// The paths, `--flags` and file names of a bullet, lowercase.
pub(super) fn distinctive_tokens(body: &str) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    for piece in body.split_whitespace() {
        let token = piece
            .trim_start_matches(['"', '\'', '`', '(', '[', '{', '<'])
            .trim_end_matches([
                '"', '\'', '`', ')', ']', '}', '>', '.', ',', ';', ':', '!', '?',
            ])
            .to_lowercase();
        if is_distinctive(&token) && !tokens.contains(&token) {
            tokens.push(token);
        }
    }
    tokens
}

/// Whether a token occurs in the criteria text; a path also counts when its last segment occurs.
fn token_found(token: &str, criteria_lower: &str) -> bool {
    if criteria_lower.contains(token) {
        return true;
    }
    token
        .trim_end_matches('/')
        .rsplit_once('/')
        .is_some_and(|(_, last)| !last.is_empty() && criteria_lower.contains(last))
}

/// Strips `ing`, `ed`, `es` or `s`, the first that fits, while three or more letters remain.
fn stem_of(word: &str) -> String {
    for suffix in ["ing", "ed", "es", "s"] {
        if let Some(rest) = word.strip_suffix(suffix)
            && rest.chars().count() >= 3
        {
            return rest.to_string();
        }
    }
    word.to_string()
}

/// The stems of the meaningful words of `text`, without duplicates. Words of three or more letters; not a
/// stop word; not a part of one of `tokens` (a file name is judged as a token, not as words).
pub(super) fn stems_of(text: &str, tokens: &[String]) -> Vec<String> {
    let mut stems: Vec<String> = Vec::new();
    for word in request_words(text) {
        let word = word
            .text
            .strip_suffix("'s")
            .unwrap_or(&word.text)
            .to_string();
        if word.chars().count() <= 2 || STOP_WORDS.contains(&word.as_str()) {
            continue;
        }
        if tokens
            .iter()
            .any(|t| *t == word || token_pieces(t).contains(&word.as_str()))
        {
            continue;
        }
        let stem = stem_of(&word);
        if !stems.contains(&stem) {
            stems.push(stem);
        }
    }
    stems
}

/// Whether two stems are the same word: equal, or both three or more characters long with a common prefix of
/// at least the shorter length less one (and at least three).
pub(super) fn related(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let (la, lb) = (a.chars().count(), b.chars().count());
    if la < 3 || lb < 3 {
        return false;
    }
    let common = a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count();
    common >= 3.max(la.min(lb) - 1)
}

/// One acceptance bullet with what is needed to judge it.
struct Bullet {
    body: String,
    tokens: Vec<String>,
    stems: Vec<String>,
}

impl Bullet {
    fn new(body: &str) -> Bullet {
        let tokens = distinctive_tokens(body);
        let stems = stems_of(body, &tokens);
        Bullet {
            body: body.to_string(),
            tokens,
            stems,
        }
    }
}

/// The stems that tell bullet `at` from the other bullets and from the request's other text. When none is
/// left, the stems the other bullets do not share; when none is left either, all of its stems.
fn specific_stems(at: usize, bullets: &[Bullet], context: &[String]) -> Vec<String> {
    let others: Vec<&String> = bullets
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != at)
        .flat_map(|(_, b)| b.stems.iter())
        .collect();
    let own = &bullets[at].stems;
    let unique: Vec<String> = own
        .iter()
        .filter(|s| !others.contains(s))
        .cloned()
        .collect();
    let specific: Vec<String> = unique
        .iter()
        .filter(|s| !context.contains(s))
        .cloned()
        .collect();
    if !specific.is_empty() {
        specific
    } else if !unique.is_empty() {
        unique
    } else {
        own.clone()
    }
}

/// The bullets of the request that the criteria leave out.
fn uncovered_bullets(parsed: &ParsedRequest, criteria: &str) -> Vec<String> {
    let criteria_lower = criteria.to_lowercase();
    let criteria_stems = stems_of(criteria, &[]);
    let bullets: Vec<Bullet> = parsed.bullets.iter().map(|b| Bullet::new(b)).collect();
    let context = stems_of(&parsed.outside.join("\n"), &[]);
    let mut missing = Vec::new();
    for (at, bullet) in bullets.iter().enumerate() {
        let covered = if bullet.tokens.is_empty() {
            let specific = specific_stems(at, &bullets, &context);
            specific.is_empty()
                || specific
                    .iter()
                    .any(|s| criteria_stems.iter().any(|c| related(s, c)))
        } else {
            bullet
                .tokens
                .iter()
                .all(|t| token_found(t, &criteria_lower))
        };
        if !covered {
            missing.push(bullet.body.clone());
        }
    }
    missing
}

/// Refuses a reading whose acceptance criteria leave out an acceptance bullet of the request, naming each
/// omitted bullet. Root cause: on 2026-10-29 a job passed its checks and the verifier and was then rejected
/// for a split the reading chose, because the small-job default let a reading drop one of the request's
/// acceptance bullets into a follow-up and nothing compared the criteria with the bullets until the final
/// verdict, after the full job had run (the request of job 20261029-111121-fzx0 describes it). A bullet is a
/// `-`, `*` or `N.` line under the request's `Acceptance` heading, up to a blank line or a `Files:` line, with
/// a wrapped line joined to it. It is covered when every path, `--flag` and file name in it occurs in the
/// criteria taken together (a path also by its last segment), or, when it has none, when at least one word
/// that tells it from the other bullets and the request's other text is related to a word of the criteria
/// (same stem: name, names, naming). Not checked: a reading that declines or asks a question, since a split
/// may be proposed only as a question, never by dropping a bullet; a request with no `Acceptance` heading;
/// and a request whose own text outside the bullets affirmatively allows splitting the job (a permission
/// word, the word split, a job word near it, no negation or condition). This check refuses more readings; the
/// higher refusal rate is intended. The share is expected 10% of newly refused readings, an estimate, not a
/// measurement. Limits: it reads words, not meaning, so a paraphrase that shares no specific word with its
/// bullet is refused, and a dropped bullet whose specific words occur in other criteria is accepted.
pub fn check_request_coverage(reading: &Reading, request: &str) -> Result<()> {
    if reading.capability.is_none() || reading.question.is_some() {
        return Ok(());
    }
    let parsed = parse_request(request);
    if parsed.bullets.is_empty() || allows_split(&parsed.outside) {
        return Ok(());
    }
    let criteria = reading
        .acceptance_criteria
        .iter()
        .map(|c| c.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let missing = uncovered_bullets(&parsed, &criteria);
    if missing.is_empty() {
        return Ok(());
    }
    let named: Vec<String> = missing.iter().map(|b| format!("\"{b}\"")).collect();
    bail!(
        "the reading was refused because its acceptance criteria leave out {} of the request's {} \
         acceptance bullets: {}. Cover each one in a criterion, or, if the job is too large for one run, \
         ask the split as a question; a reading may not drop a bullet into a follow-up",
        missing.len(),
        parsed.bullets.len(),
        named.join("; ")
    )
}
