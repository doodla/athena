//! The defined-terms check and its quoted-span helpers.

use super::*;

/// The structural words a request can use that a reader must define before building on them
/// ([`check_defined_terms`]). Each is matched as a whole word, in the singular, the plural and the possessive.
pub(super) const STRUCTURAL_TERMS: [&str; 4] = ["ladder", "rung", "tier", "layer"];

/// The names in `docs/ontology.toml` that a request can use and that a reader must restate before building on
/// them ([`check_defined_terms`]): a name that carries more than one recorded sense and has caused a misread.
/// Each is the `[[term]]` name exactly as the ontology writes it, and is matched as a whole word in the exact
/// form and the possessive (`Hermes`, `hermes`, `Hermes's`), with no plural. Policy: a name is added only after
/// it caused a misread, as the ontology itself requires. Every ontology term is either in this list or in
/// `UNGATED_TERMS`, which only the test build holds; a test fails when a term is in neither, when it is in both,
/// or when a listed name is not an ontology term, so this list cannot drift from the shared vocabulary.
pub(super) const AMBIGUOUS_NAMES: [&str; 1] = ["Hermes"];

/// The ontology terms that [`check_defined_terms`] does not require a `Term <name>:` line for, each with the
/// reason. The test `ambiguous_and_ungated_terms_cover_the_ontology` requires every `[[term]]` of
/// `docs/ontology.toml` to be in exactly one of this list and [`AMBIGUOUS_NAMES`], so adding a term to the
/// ontology fails the build until someone decides whether readings must define it.
#[cfg(test)]
pub(super) const UNGATED_TERMS: [(&str, &str); 6] = [
    (
        "planning",
        "a common word that most requests use in its plain sense",
    ),
    (
        "job",
        "a common word that most requests use in its plain sense",
    ),
    (
        "landed",
        "a common word that most requests use in its plain sense",
    ),
    (
        "Athena",
        "the name of the system itself, which nearly every request uses",
    ),
    (
        "Sri",
        "a naming convention for the owner, not a word a reading must define",
    ),
    (
        "product versus system",
        "a phrase of several words that the single-word matcher cannot match",
    ),
];

/// The sentence with which Athena's root-cause request opens its body (`hestia::root_cause_request`). A line of
/// that request that carries the words of earlier introspections is skipped only when this sentence is present.
pub(super) const ROOT_CAUSE_ANCHOR: &str =
    "The same cause has produced failures in more than one job.";

/// The sentence with which Athena's approved-work request describes itself (`work_decisions::approve_request`).
/// Its rung line, proposal line and copied sections are skipped only when this sentence is present.
pub(super) const APPROVAL_ANCHOR: &str = "A person approved this proposal from the steward role.";

/// Whether a line of a request is text Athena's own generators wrote or copied, not the request's own use of a
/// word. Each shape is exact, and the lines that carry free text from earlier records need the generator's
/// anchor sentence as well, so a hand-written line that starts the same way still counts.
pub(super) fn is_generated_line(line: &str, root_cause: bool, approval: bool) -> bool {
    let introspection = line.starts_with("Introspection ")
        && line.contains(" (job ")
        && line.contains("); prevention it proposed:");
    let root_cause_line = root_cause
        && [
            "Failure class: ",
            "Cause: ",
            "Fix the root cause of a repeated failure: ",
        ]
        .iter()
        .any(|prefix| line.starts_with(prefix));
    let approval_line = approval
        && (line.starts_with("Rung of the requirements ladder: ")
            || (line.starts_with("Proposal: ")
                && line.contains(" (key ")
                && line.contains(" queued at ")));
    introspection || root_cause_line || approval_line
}

/// The text of `line` without its quoted spans: double-quoted (straight or curly), backtick pairs, and
/// single-quoted spans. A single quote opens a span only after the start, white space or a bracket and before a
/// letter or digit, and closes it after a non-space and before a non-alphanumeric, so the apostrophes of
/// "Sri's" and "don't" open nothing. An opener with no closer on the line stays as text.
pub(super) fn without_quoted_spans(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let find_from = |from: usize, close: char| {
        chars[from..]
            .iter()
            .position(|c| *c == close)
            .map(|at| from + at)
    };
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    while i < chars.len() {
        let end = match chars[i] {
            '"' => find_from(i + 1, '"'),
            '\u{201C}' => find_from(i + 1, '\u{201D}'),
            '`' => find_from(i + 1, '`'),
            '\'' | '\u{2018}' => single_quote_end(&chars, i),
            _ => None,
        };
        match end {
            Some(end) => {
                out.push(' ');
                i = end + 1;
            }
            None => {
                out.push(chars[i]);
                i += 1;
            }
        }
    }
    out
}

/// The index of the character that closes the single-quoted span opened at `open`, if one opens there.
pub(super) fn single_quote_end(chars: &[char], open: usize) -> Option<usize> {
    let opens =
        (open == 0 || chars[open - 1].is_whitespace() || matches!(chars[open - 1], '(' | '['))
            && chars.get(open + 1).is_some_and(|c| c.is_alphanumeric());
    if !opens {
        return None;
    }
    (open + 2..chars.len()).find(|&j| {
        matches!(chars[j], '\'' | '\u{2019}')
            && !chars[j - 1].is_whitespace()
            && chars.get(j + 1).is_none_or(|next| !next.is_alphanumeric())
    })
}

/// Whether the text inside parentheses is a list of examples, not a statement: two or more commas, or an
/// opening of `e.g.`, `i.e.`, `such as`, `for example` or `like`.
pub(super) fn is_example_list(inner: &str) -> bool {
    let lower = inner.trim().to_lowercase();
    inner.matches(',').count() >= 2
        || ["e.g.", "i.e.", "such as ", "for example", "like "]
            .iter()
            .any(|opening| lower.starts_with(opening))
}

/// The text of `line` without its parenthesised example lists ([`is_example_list`]). A parenthetical that is a
/// statement, such as "(the ladder, which ranks requirements)", stays.
pub(super) fn without_example_lists(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '(' {
            let mut depth = 0usize;
            let close = (i..chars.len()).find(|&j| {
                match chars[j] {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ => {}
                }
                depth == 0
            });
            if let Some(close) = close {
                let inner: String = chars[i + 1..close].iter().collect();
                if is_example_list(&inner) {
                    out.push(' ');
                    i = close + 1;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// One line of a request as plain prose: no quoted spans, no example lists, and no token that is a path or an
/// identifier (it holds `/`, `_` or `::`).
pub(super) fn plain_line(line: &str) -> String {
    let unquoted = without_quoted_spans(line);
    without_example_lists(&unquoted)
        .split_whitespace()
        .filter(|token| !(token.contains('/') || token.contains('_') || token.contains("::")))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The request as plain prose, where a structural word is a use and not a mention: it leaves out the text
/// Athena's generators wrote or copied ([`is_generated_line`], and, in an approved-work request, everything from
/// the `## Source` heading on), fenced code, quoted spans, example lists and paths or identifiers.
pub(super) fn plain_request_prose(request: &str) -> String {
    let root_cause = request.contains(ROOT_CAUSE_ANCHOR);
    let approval = request.contains(APPROVAL_ANCHOR);
    let mut kept: Vec<String> = Vec::new();
    let mut fenced = false;
    for line in request.lines() {
        if approval && line.trim_end() == "## Source" {
            break;
        }
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced || is_generated_line(line, root_cause, approval) {
            continue;
        }
        kept.push(plain_line(line));
    }
    kept.join("\n")
}

/// The term a lowercase word is that the reading must define: a structural term in its singular, plural or
/// possessive form, or an ambiguous name ([`AMBIGUOUS_NAMES`]) in its exact or possessive form. A name comes
/// back as the list writes it (`Hermes`), so the error and the `Term Hermes:` line use the ontology's spelling.
pub(super) fn named_term(word: &str) -> Option<&'static str> {
    let word = word.strip_suffix("'s").unwrap_or(word);
    STRUCTURAL_TERMS
        .into_iter()
        .find(|term| word == *term || word.strip_suffix('s') == Some(*term))
        .or_else(|| {
            AMBIGUOUS_NAMES
                .into_iter()
                .find(|name| name.to_lowercase() == word)
        })
}

/// The terms the reading defines: an assumption that starts `Term <word>:` and gives at least three words of
/// meaning after the colon, as the reader writes a `Code survey:` line.
pub(super) fn defined_terms(reading: &Reading) -> BTreeSet<&'static str> {
    reading
        .assumptions
        .iter()
        .filter_map(|assumption| {
            let lower = assumption.trim().to_lowercase().replace('\u{2019}', "'");
            let (word, meaning) = lower.strip_prefix("term ")?.split_once(':')?;
            (meaning.split_whitespace().count() >= 3)
                .then(|| named_term(word.trim()))
                .flatten()
        })
        .collect()
}

/// Refuses a reading that does not define a structural term (`ladder`, `rung`, `tier` or `layer`) or an
/// ambiguous ontology name ([`AMBIGUOUS_NAMES`], today `Hermes`) the request itself uses. Root cause this check
/// removes: readings are not checked against the outcome the user observes, so structural terms stay undefined,
/// and so do ambiguous ontology names. A request for "a ladder of requirements" was read as if the ladder were
/// understood, no assumption said what it was, and the work was built on a guess; job 20261014-174815-7bf8
/// failed that way. Job 20261017-173508-0wtc (an ontology decision) named Hermes Agent, a name that
/// `docs/ontology.toml` records with two senses (the studied agent, and the Orchestration god), and was built on
/// one guess at it. Job 20261014-082328-g6vy (a publish request) is the same cause for a delivery verb and is
/// covered by [`check_outcome_state`] and [`check_governance_coverage`]. This check covers 2 of the 3 cited
/// jobs, 20261014-174815-7bf8 and 20261017-173508-0wtc. It reads only the request and the reading, costs no
/// model call and, like [`check_outcome_state`], skips a reading with no capability, a question, or no
/// deliverable. A use is a whole word (`ladder`, `rungs`, `tier's`, `Hermes`, `hermes's`) in the plain prose of
/// the request: `stepladder`, `delayer`, `hermeshub`, `layer_count` and `ladder.md` are not uses, and neither
/// are quoted spans, backtick spans, fenced code, a parenthesised list of examples (two or more commas, or an
/// opening of `e.g.`, `i.e.`, `such as`, `for example` or `like`), paths and identifiers, or text Athena's own
/// generators wrote or copied (the earlier introspections a root-cause request carries, and the rung line,
/// proposal line and `## Source` onward of an approved-work request, each skipped only when the generator's own
/// sentence is present). A term is defined by an assumption that starts `Term ladder:` or `Term Hermes:` (the
/// plural and `'s` forms of a structural word are accepted) with at least three words of meaning; the line may
/// cite the file that defines it. For an ambiguous name the error asks the line to say which recorded sense
/// applies and to cite `docs/ontology.toml`. The error names the terms and the repair. Limits: quoted
/// mentions, terms outside the list (milestone, glossary, and every ontology term not in the ambiguous-name
/// list) and the sense of an ambiguous name are not covered: the check forces a restatement of the name but
/// cannot verify that the restatement matches the recorded senses, so a line that names one of two senses
/// passes. This check refuses more readings; the higher refusal rate is intended. The declared share, expected
/// 2% of newly refused readings, is an estimate and not a measurement, over hand-written requests; the share
/// the loop replay measures is recorded in the job's replay record at verification. Terms the repository
/// defines (ladder, rung) are refused until the reading restates them with a `Term ladder:` line, and so are
/// plain technical uses of tier (priority tiers) and layer (a caching layer). A proposed request read before
/// this check whose plain prose uses a term is held until a person runs `athena go`:
/// [`crate::steward::queued_work`] re-checks its stored reading and leaves it out of the queue.
pub fn check_defined_terms(reading: &Reading, request: &str) -> Result<()> {
    if reading.capability.is_none() || reading.question.is_some() || reading.deliverables.is_empty()
    {
        return Ok(());
    }
    let defined = defined_terms(reading);
    let mut undefined: Vec<&'static str> = Vec::new();
    for word in request_words(&plain_request_prose(request)) {
        if let Some(term) = named_term(&word.text)
            && !defined.contains(term)
            && !undefined.contains(&term)
        {
            undefined.push(term);
        }
    }
    let Some(first) = undefined.first() else {
        return Ok(());
    };
    bail!(
        "the reading was refused because the request uses {} as a structural term or an ambiguous name but the \
         reading does not define {}, so the work would be built on a guess at what it means. Repair: add an \
         assumption that starts `Term {first}:` (one per term) saying what the term means in this job, in at \
         least three words, and cite the file that defines it if there is one; for an ambiguous name say which \
         of its recorded senses applies and cite docs/ontology.toml; a quoted or cited mention needs no \
         definition",
        undefined.join(" and "),
        if undefined.len() == 1 { "it" } else { "them" }
    )
}
