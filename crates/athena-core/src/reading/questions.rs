//! The reader-question check.

use super::*;
use crate::admission::{is_user_addressed, says_code_answers};

/// The reader-question rule. Root cause it addresses: the reader's `question` was ungated free text, so a
/// question the repository or the defaults could settle reached the customer with no alternatives and no
/// default, and the customer's time went to a point Athena should have settled or framed before asking.
///
/// A question is accepted only when it is a precise decision for the customer: it begins `User:` (the same
/// prefix normalisation as the planner's open questions; `Host:` does not count, because a host fact is not the
/// customer's decision), it does not say the code answers it, it offers at least two distinct alternatives
/// labelled `(a)`, `(b)` or `(1)`, `(2)`, and it ends with `Default: (a)`, naming the alternative the job runs
/// on if the customer does not answer. A reading with no question is not checked. This check refuses more
/// readings; the higher refusal rate is intended, because questions the repository or defaults could settle
/// cost the customer time. The declared share, expected 3% of newly refused readings, is an estimate, not a
/// measurement. Limits: only labelled alternatives are read, so prose such as "either X or Y" is refused, and
/// the `User:` prefix is a declaration, not proof that only the customer can decide the point.
pub fn check_reader_question(reading: &Reading) -> Result<()> {
    let Some(question) = reading.question.as_deref() else {
        return Ok(());
    };
    let mut gaps = Vec::new();
    if !is_user_addressed(question) {
        gaps.push("it does not begin with `User:`");
    }
    if says_code_answers(question) {
        gaps.push("it says the code answers it");
    }
    let body = question_body(question);
    let (head, tail) = split_default(body);
    let alternatives = alternatives_of(head);
    if alternatives.len() < 2 {
        gaps.push("it offers fewer than two distinct alternatives labelled (a), (b)");
    }
    let names_default = tail.is_some_and(|tail| {
        labels_in(tail)
            .iter()
            .any(|(label, _, _)| alternatives.iter().any(|(l, _)| l == label))
    });
    if !names_default {
        gaps.push("it names no `Default:` that is one of the alternative labels");
    }
    if gaps.is_empty() {
        return Ok(());
    }
    bail!(
        "the reading was refused because its question is not a precise decision for the customer: {}. \
         Question: \"{}\". Repair: if the repository or the defaults can settle it, settle it, record the \
         answer as an assumption and leave `question` null. Otherwise write `User:` + the decision + the \
         alternatives labelled (a), (b) + `Default: (a)`, the alternative the job runs on if unanswered, for \
         example `User: Which greeting: (a) hello or (b) hi? Default: (a).` A question must not say the code \
         answers it",
        gaps.join("; "),
        question.trim()
    )
}

/// The question's text after its `User:` prefix; the whole text when it has none.
fn question_body(question: &str) -> &str {
    let trimmed =
        question.trim_start_matches(|c: char| c.is_whitespace() || matches!(c, '*' | '_' | '`'));
    if is_user_addressed(question) {
        trimmed.get("user:".len()..).unwrap_or("")
    } else {
        trimmed
    }
}

/// Splits at the last `default:` (letter case ignored): the text before it and the text after it.
fn split_default(body: &str) -> (&str, Option<&str>) {
    match body.to_ascii_lowercase().rfind("default:") {
        Some(at) => (&body[..at], Some(&body[at + "default:".len()..])),
        None => (body, None),
    }
}

/// The label tokens in `text` as (lowercased label, start, end) byte offsets. A label is one ASCII letter or
/// one or two digits in parentheses, and counts only at the start of the text or after whitespace or `:`, so
/// `file(s)` holds no label.
fn labels_in(text: &str) -> Vec<(String, usize, usize)> {
    let mut found = Vec::new();
    let mut i = 0;
    while i < text.len() {
        match label_at(text, i) {
            Some((label, end)) => {
                found.push((label, i, end));
                i = end;
            }
            None => i += 1,
        }
    }
    found
}

/// The label that starts at byte `i` of `text`: its lowercased name and the offset just after it.
fn label_at(text: &str, i: usize) -> Option<(String, usize)> {
    let bytes = text.as_bytes();
    let after_break = i == 0 || bytes[i - 1].is_ascii_whitespace() || bytes[i - 1] == b':';
    if bytes[i] != b'(' || !after_break {
        return None;
    }
    let close = text[i + 1..].find(')')?;
    let inner = &text[i + 1..i + 1 + close];
    let letter = inner.len() == 1 && inner.as_bytes()[0].is_ascii_alphabetic();
    let digits = (1..=2).contains(&inner.len()) && inner.bytes().all(|b| b.is_ascii_digit());
    (letter || digits).then(|| (inner.to_ascii_lowercase(), i + close + 2))
}

/// The (label, text) of each alternative in `head` with non-empty text that no earlier alternative repeats.
fn alternatives_of(head: &str) -> Vec<(String, String)> {
    let labels = labels_in(head);
    let mut alternatives: Vec<(String, String)> = Vec::new();
    for (n, (label, _, end)) in labels.iter().enumerate() {
        let stop = labels.get(n + 1).map_or(head.len(), |next| next.1);
        let text = trim_alternative(&head[*end..stop]);
        let repeated = alternatives
            .iter()
            .any(|(l, t)| l == label || t.eq_ignore_ascii_case(&text));
        if !text.is_empty() && !repeated {
            alternatives.push((label.clone(), text));
        }
    }
    alternatives
}

/// An alternative's text without surrounding whitespace, `,;?.` or a trailing joining word `or`.
fn trim_alternative(text: &str) -> String {
    let mut text =
        text.trim_matches(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | '?' | '.'));
    loop {
        let lowered = text.to_ascii_lowercase();
        if lowered == "or" {
            return String::new();
        }
        if lowered.ends_with(" or") {
            text = text[..text.len() - 3]
                .trim_end_matches(|c: char| c.is_whitespace() || matches!(c, ',' | ';'));
        } else {
            return text.to_string();
        }
    }
}
