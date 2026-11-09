//! The boundary scan: the words and fragments Athena keeps out of tracked files, the producer
//! brief's sections that show them, the scan itself and the plain rewrite of text that trips it.

/// Words the boundary scan (M19) rejects, built from fragments so no tracked file contains them. This
/// list is the one the producer brief shows; `xtask/tests/policy.rs` keeps its own scanner list and a test
/// there fails when the two differ (D73). Matched as whole words, case-insensitively.
pub fn boundary_words() -> Vec<String> {
    let fragments: [[&str; 2]; 17] = [
        ["oper", "ator"],
        ["oper", "ators"],
        ["op", "ctl"],
        ["c", "cr"],
        ["sand", "box"],
        ["sand", "boxes"],
        ["sand", "boxed"],
        ["lea", "se"],
        ["lea", "ses"],
        ["hand", "off"],
        ["hand", "offs"],
        ["hand-", "off"],
        ["hand-", "offs"],
        ["cloud", " session"],
        ["cloud", " sessions"],
        ["remote", " session"],
        ["remote", " sessions"],
    ];
    fragments.iter().map(|p| p.concat()).collect()
}

/// Fragments of machine-specific paths and identifiers the boundary scan rejects anywhere in a file,
/// case-insensitively. Built from fragments for the same reason as [`boundary_words`].
pub fn boundary_fragments() -> Vec<String> {
    [
        ["/home", "/user"],
        ["claude", "-0/"],
        ["/root", "/."],
        [".c", "cr/"],
        ["claude.ai", "/code"],
        ["session", "_01"],
    ]
    .iter()
    .map(|p| p.concat())
    .collect()
}

/// The producer brief's banned-words section (D73): placed before the checks so the producer reads it
/// before writing anything.
pub(super) fn banned_words_section() -> String {
    let words: Vec<String> = boundary_words().iter().map(|w| format!("`{w}`")).collect();
    let fragments: Vec<String> = boundary_fragments()
        .iter()
        .map(|w| format!("`{w}`"))
        .collect();
    format!(
        "## Words you must not write\n\n\
         Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. \
         Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or \
         design text. The scan matches whole words, case-insensitively, so capital letters do not help; \
         singular, plural and hyphenated forms are all listed. Reword instead.\n\n\
         Words: {}\n\n\
         Fragments (rejected anywhere in a line, even inside a longer word or path): {}\n\n\
         {BOUNDARY_SCAN_INSTRUCTION}\n",
        words.join(", "),
        fragments.join(", ")
    )
}

/// What the producer brief tells the producer to run before it finishes. A word list alone was not enough
/// (a brief that warned of a word still produced a file that failed on it), so the producer runs the scan
/// itself and rewords what it finds. The scan reads tracked files from the working directory, so it covers
/// edits to existing files; a file the producer created is not tracked until Athena commits it.
pub const BOUNDARY_SCAN_INSTRUCTION: &str = "Before you finish, run `cargo test -p xtask --test policy` on your changed files. \
It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it \
finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: \
reread each new file against the words above.";

/// The producer brief's note on files kept by hand. A helper only: the mechanism is the registry
/// (`athena_core::mirrors`) that Athena runs on the producer's work before verification and the `athena-cli`
/// test target `mirrors`. The note lets the producer find a drift in its own loop and spend no send-back on it.
pub const MIRROR_CHECK_INSTRUCTION: &str = "If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run \
`cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test \
timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by \
content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and \
docs/decisions.md only gains text after its last line.";

/// The "Files kept by hand" section of the producer brief, placed after the banned words and before the checks.
pub(super) fn mirror_check_section() -> String {
    format!("## Files kept by hand\n\n{MIRROR_CHECK_INSTRUCTION}\n")
}

/// One boundary hit in a text: the 1-based line and the banned word or fragment found there. The word is
/// lowercase for fragments, as the policy scan reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundaryHit {
    pub line: usize,
    pub term: String,
}

/// Whether `c` is part of a word for whole-word matching: a letter, a digit or an underscore.
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Whether `word` occurs in `lower_line` as a whole word. Both are lowercase. A match counts when the
/// characters on either side are not word characters, the same rule as a regex `\b` at each end.
fn contains_whole_word(lower_line: &str, word: &str) -> bool {
    let mut from = 0;
    while let Some(found) = lower_line[from..].find(word) {
        let start = from + found;
        let end = start + word.len();
        let before_ok = lower_line[..start]
            .chars()
            .next_back()
            .is_none_or(|c| !is_word_char(c));
        let after_ok = lower_line[end..]
            .chars()
            .next()
            .is_none_or(|c| !is_word_char(c));
        if before_ok && after_ok {
            return true;
        }
        // Step past this match's first character and look again.
        from = start + lower_line[start..].chars().next().map_or(1, char::len_utf8);
    }
    false
}

/// The boundary scan (M19) over one text: every line that holds a banned word or fragment, in line order.
/// It uses [`boundary_words`] and [`boundary_fragments`], the lists the producer brief shows, and applies the
/// policy scan's rules: whole words and fragments, case-insensitively. Athena runs it on a request and on
/// the files the request names, before the job is admitted.
pub fn boundary_scan(text: &str) -> Vec<BoundaryHit> {
    let words: Vec<String> = boundary_words().iter().map(|w| w.to_lowercase()).collect();
    let fragments: Vec<String> = boundary_fragments()
        .iter()
        .map(|f| f.to_lowercase())
        .collect();
    let mut found = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let lower = line.to_lowercase();
        for word in &words {
            if contains_whole_word(&lower, word) {
                found.push(BoundaryHit {
                    line: i + 1,
                    term: word.clone(),
                });
            }
        }
        for fragment in &fragments {
            if lower.contains(fragment.as_str()) {
                found.push(BoundaryHit {
                    line: i + 1,
                    term: fragment.clone(),
                });
            }
        }
    }
    found
}

/// The plain wording that stands in for each word of [`boundary_words`], as (word, replacement) pairs.
/// The words are built from fragments, like the list itself, so no tracked file holds them. Every
/// replacement reads as ordinary prose and holds no boundary word.
fn plain_alternatives() -> Vec<(String, &'static str)> {
    let table: [([&str; 2], &str); 17] = [
        (["oper", "ator"], "the person running Athena"),
        (["oper", "ators"], "the people running Athena"),
        (["op", "ctl"], "the Athena command-line tool"),
        (["c", "cr"], "the hosted runner"),
        (["sand", "box"], "isolated environment"),
        (["sand", "boxes"], "isolated environments"),
        (["sand", "boxed"], "isolated"),
        (["lea", "se"], "claim"),
        (["lea", "ses"], "claims"),
        (["hand", "off"], "transfer"),
        (["hand", "offs"], "transfers"),
        (["hand-", "off"], "transfer"),
        (["hand-", "offs"], "transfers"),
        (["cloud", " session"], "hosted run"),
        (["cloud", " sessions"], "hosted runs"),
        (["remote", " session"], "hosted run"),
        (["remote", " sessions"], "hosted runs"),
    ];
    let mut pairs: Vec<(String, &'static str)> =
        table.iter().map(|(w, r)| (w.concat(), *r)).collect();
    // Longest first, so a plural is never cut short by its singular.
    pairs.sort_by_key(|(w, _)| std::cmp::Reverse(w.len()));
    pairs
}

/// Replace every case-insensitive occurrence of the ASCII `needle` in `text` with `replacement`.
fn replace_ascii_ignore_case(text: &str, needle: &str, replacement: &str) -> String {
    let lower = text.to_ascii_lowercase();
    let needle = needle.to_ascii_lowercase();
    let mut out = String::with_capacity(text.len());
    let mut from = 0;
    while let Some(found) = lower[from..].find(&needle) {
        let start = from + found;
        out.push_str(&text[from..start]);
        out.push_str(replacement);
        from = start + needle.len();
    }
    out.push_str(&text[from..]);
    out
}

/// Replace each whole-word occurrence of a boundary word in `text` with its plain wording, keeping a
/// leading capital. The same whole-word rule as [`boundary_scan`] decides what counts as a match.
fn replace_boundary_words(text: &str) -> String {
    let pairs = plain_alternatives();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        let Some(ch) = text[i..].chars().next() else {
            break;
        };
        let at_word_start = text[..i]
            .chars()
            .next_back()
            .is_none_or(|c| !is_word_char(c));
        let hit = if at_word_start {
            pairs.iter().find(|(word, _)| {
                text.get(i..i + word.len())
                    .is_some_and(|piece| piece.eq_ignore_ascii_case(word))
                    && text[i + word.len()..]
                        .chars()
                        .next()
                        .is_none_or(|c| !is_word_char(c))
            })
        } else {
            None
        };
        match hit {
            Some((word, replacement)) => {
                if ch.is_uppercase() {
                    let mut chars = replacement.chars();
                    if let Some(first) = chars.next() {
                        out.extend(first.to_uppercase());
                        out.push_str(chars.as_str());
                    }
                } else {
                    out.push_str(replacement);
                }
                i += word.len();
            }
            None => {
                out.push(ch);
                i += ch.len_utf8();
            }
        }
    }
    out
}

/// Rewrite `text` so the boundary scan ([`boundary_scan`]) finds nothing in it, for text that Athena itself
/// writes and that later becomes a request or a record. Each word of [`boundary_words`] becomes plain
/// wording (the first word, for the one who runs Athena, becomes "the person running Athena"); each
/// machine-path fragment of [`boundary_fragments`] becomes `[local path]`. Matching ignores case, and a
/// replacement keeps a leading capital. Text with no hit comes back unchanged.
///
/// The result is scanned again. If a term survives, for example because a replacement landed against
/// other characters and made a new match, this returns an error that names the term and line, so a writer
/// refuses the text instead of writing something admission would reject.
pub fn plain_boundary_text(text: &str) -> Result<String, String> {
    if boundary_scan(text).is_empty() {
        return Ok(text.to_string());
    }
    let mut cleaned = text.to_string();
    for fragment in boundary_fragments() {
        cleaned = replace_ascii_ignore_case(&cleaned, &fragment, "[local path]");
    }
    let cleaned = replace_boundary_words(&cleaned);
    match boundary_scan(&cleaned).first() {
        None => Ok(cleaned),
        Some(hit) => Err(format!(
            "the text holds `{}` on line {}, which could not be reworded",
            hit.term, hit.line
        )),
    }
}
