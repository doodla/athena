//! The delivery-verb and outcome-state check.

use super::*;

/// The delivery verbs the outcome-state check reads, each with its `-ing` form.
pub(super) const DELIVERY_VERBS: [(&str, &str); 4] = [
    ("publish", "publishing"),
    ("deploy", "deploying"),
    ("share", "sharing"),
    ("release", "releasing"),
];

/// Words before a verb that leave it a verb of the request: "Build the docs and publish them", "Please deploy
/// the site", "Workers should share the report".
pub(super) const LEAD_INS: [&str; 5] = ["and", "then", "please", "also", "should"];

/// Two-word lead-ins: "Can you deploy the site", "I want to release v1".
pub(super) const LEAD_IN_PAIRS: [(&str, &str); 7] = [
    ("can", "you"),
    ("could", "you"),
    ("would", "you"),
    ("want", "to"),
    ("need", "to"),
    ("like", "to"),
    ("going", "to"),
];

/// A word right after the verb that makes it part of a noun phrase: "release notes", "share button".
pub(super) const NOUN_FOLLOWERS: [&str; 11] = [
    "notes",
    "note",
    "process",
    "pipeline",
    "workflow",
    "checklist",
    "candidate",
    "date",
    "plan",
    "button",
    "of",
];

/// Words that end the stretch after a verb in which its object is looked for.
pub(super) const OBJECT_STOP_WORDS: [&str; 8] = [
    "before", "after", "when", "while", "if", "because", "until", "unless",
];

/// Things an audience sees: a verb with one of these as its object delivers something to a reader.
pub(super) const VISIBLE_NOUNS: [&str; 13] = [
    "page",
    "pages",
    "site",
    "website",
    "docs",
    "documentation",
    "explainer",
    "guide",
    "book",
    "report",
    "tutorial",
    "article",
    "dashboard",
];

/// Places a delivery goes to. `crate`, `package`, `binary`, `app`, `image` and `artifact` are not listed: in a
/// Rust workspace they are ordinary technical nouns ("share the parsing code between crates"), so they count
/// only through a destination or a version token beside them.
pub(super) const DESTINATION_WORDS: [&str; 8] = [
    "npm",
    "pypi",
    "docker",
    "crates.io",
    "registry",
    "production",
    "online",
    "pages",
];

/// People a delivery reaches.
pub(super) const AUDIENCE_WORDS: [&str; 7] = [
    "team",
    "everyone",
    "colleagues",
    "customers",
    "stakeholders",
    "readers",
    "public",
];

pub(super) const OBJECT_PRONOUNS: [&str; 4] = ["it", "them", "these", "those"];

/// Whole-word lowercase phrases that name a state an audience can observe. `url`, `tag`, `link`, `page`,
/// `can open`, `serves` and a bare `available at` are not listed: they occur in criteria that only check a
/// file ("the <title> tag names the page", "docs/explainer.md is available at docs/").
pub(super) const OUTCOME_STATE_PHRASES: [&str; 22] = [
    "reachable",
    "is served",
    "are served",
    "served at",
    "served from",
    "served page",
    "http 200",
    "200 ok",
    "is live",
    "goes live",
    "live at",
    "available at https",
    "available at http",
    "accessible at https",
    "opens in a browser",
    "loads in a browser",
    "can be downloaded",
    "can download",
    "available for download",
    "downloadable",
    "installable",
    "can be installed from",
];

/// One word of a request, lowercase, and whether it opens a segment: the first word, a word after a line
/// break or after `.`, `!`, `?`, `:` or `;` followed by white space, or the word after a list marker.
pub(super) struct RequestWord {
    pub(super) text: String,
    pub(super) segment_start: bool,
}

/// Whether the text between two words ends a segment.
pub(super) fn gap_ends_segment(gap: &str) -> bool {
    if gap.contains('\n') {
        return true;
    }
    let chars: Vec<char> = gap.chars().collect();
    chars.iter().enumerate().any(|(i, c)| {
        ".!?:;".contains(*c) && chars.get(i + 1).is_none_or(|next| next.is_whitespace())
    })
}

/// The words of `text`: runs of letters and digits, where a `.` or `'` between two of them stays in the word
/// (`crates.io`, `v1.2`, `don't`). A word that is digits or one ASCII letter, followed by `)` or `.`, is a list
/// marker (`2)`, `(a)`, `1.`) and the word after it opens a segment.
pub(super) fn request_words(text: &str) -> Vec<RequestWord> {
    let text = text.to_lowercase().replace(['\u{2019}', '\u{2018}'], "'");
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].1.is_alphanumeric() {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < chars.len() {
            let c = chars[j].1;
            let joins = (c == '.' || c == '\'')
                && chars
                    .get(j + 1)
                    .is_some_and(|(_, next)| next.is_alphanumeric());
            if c.is_alphanumeric() || joins {
                j += 1;
            } else {
                break;
            }
        }
        spans.push((chars[i].0, chars.get(j).map_or(text.len(), |(at, _)| *at)));
        i = j;
    }
    let mut words = Vec::new();
    let mut after_marker = false;
    for (k, &(start, end)) in spans.iter().enumerate() {
        let gap = if k == 0 {
            ""
        } else {
            &text[spans[k - 1].1..start]
        };
        let opens = k == 0 || after_marker || gap_ends_segment(gap);
        let word = &text[start..end];
        after_marker = opens
            && (word.chars().all(|c| c.is_ascii_digit())
                || (word.len() == 1 && word.chars().all(|c| c.is_ascii_alphabetic())))
            && spans
                .get(k + 1)
                .is_some_and(|&(next, _)| matches!(text[end..next].trim(), ")" | "."));
        words.push(RequestWord {
            text: word.to_string(),
            segment_start: opens,
        });
    }
    words
}

/// Whether `word` is `v` and digits, with dots between them: a version token (`v1`, `v1.2`).
pub(super) fn is_version_token(word: &str) -> bool {
    word.strip_prefix('v').is_some_and(|rest| {
        rest.chars().any(|c| c.is_ascii_digit())
            && rest.chars().all(|c| c.is_ascii_digit() || c == '.')
    })
}

/// Whether `word` names something an audience sees, a place a delivery goes to, or the people it reaches.
pub(super) fn is_delivery_object(word: &str) -> bool {
    VISIBLE_NOUNS.contains(&word)
        || DESTINATION_WORDS.contains(&word)
        || AUDIENCE_WORDS.contains(&word)
        || is_version_token(word)
}

/// The delivery verbs the request uses as verbs, each once, as `(verb, -ing form)`. A verb counts when it
/// opens a segment (a sentence, a line, a list part) or follows a lead-in ([`LEAD_INS`], [`LEAD_IN_PAIRS`]),
/// is not followed by a word of [`NOUN_FOLLOWERS`], and has a delivery object ([`is_delivery_object`]) within
/// five words in the same segment, before a word of [`OBJECT_STOP_WORDS`]; or is followed within two words by
/// `it`, `them`, `these` or `those` when the segment has such an object before the verb.
pub(super) fn delivery_verbs(request: &str) -> Vec<(&'static str, &'static str)> {
    let words = request_words(request);
    let mut found: Vec<(&'static str, &'static str)> = Vec::new();
    for (i, word) in words.iter().enumerate() {
        let Some(&entry) = DELIVERY_VERBS.iter().find(|(verb, _)| *verb == word.text) else {
            continue;
        };
        if found.contains(&entry) {
            continue;
        }
        let at_verb_position = word.segment_start
            || (i >= 1 && LEAD_INS.contains(&words[i - 1].text.as_str()))
            || (i >= 2
                && LEAD_IN_PAIRS
                    .contains(&(words[i - 2].text.as_str(), words[i - 1].text.as_str())));
        if !at_verb_position {
            continue;
        }
        let rest = &words[i + 1..];
        if rest
            .first()
            .is_some_and(|next| !next.segment_start && NOUN_FOLLOWERS.contains(&next.text.as_str()))
        {
            continue;
        }
        let window: Vec<&str> = rest
            .iter()
            .take(5)
            .take_while(|next| {
                !next.segment_start && !OBJECT_STOP_WORDS.contains(&next.text.as_str())
            })
            .map(|next| next.text.as_str())
            .collect();
        let object_after = window.iter().any(|w| is_delivery_object(w));
        let pronoun = window.iter().take(2).any(|w| OBJECT_PRONOUNS.contains(w));
        let segment_from = if word.segment_start {
            i
        } else {
            words[..i]
                .iter()
                .rposition(|w| w.segment_start)
                .unwrap_or(0)
        };
        let object_before = words[segment_from..i]
            .iter()
            .any(|w| is_delivery_object(&w.text));
        if object_after || (pronoun && object_before) {
            found.push(entry);
        }
    }
    found
}

/// Whether an out_of_scope item names the delivery: the verb or its `-ing` form as a whole word, not followed
/// by a word of [`NOUN_FOLLOWERS`]. "Changes to shared helpers" and "Release notes are unchanged" do not.
pub(super) fn out_of_scope_names(item: &str, verb: &str, ing: &str) -> bool {
    let words = request_words(item);
    words.iter().enumerate().any(|(i, word)| {
        (word.text == verb || word.text == ing)
            && !words.get(i + 1).is_some_and(|next| {
                !next.segment_start && NOUN_FOLLOWERS.contains(&next.text.as_str())
            })
    })
}

/// Whether a criterion text names a state an audience can observe ([`OUTCOME_STATE_PHRASES`], whole words).
pub(super) fn names_outcome_state(criterion: &str) -> bool {
    let text = criterion
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    OUTCOME_STATE_PHRASES.iter().any(|p| has_phrase(&text, p))
}

/// Refuses a reading for a delivery request (publish, deploy, share, release) that has no behavior criterion
/// naming a state the audience can observe and does not put the delivery out of scope. Root cause this check removes: a delivery verb was read as a narrow deliverable, and reading acceptance had
/// no check against the state the user observes. "Publish a first explainer page on how firstmate handles work"
/// was read as "a page file exists", the criteria passed on the file, and nothing asked whether anyone could
/// reach the page. Job 20261014-082328-g6vy failed that way. This check covers 1 of the 3 cited jobs, that
/// one; the other two, 20261014-174815-7bf8 (a ladder of requirements) and 20261017-173508-0wtc (an ontology
/// decision), contain no delivery verb, are another class (structural terms and ambiguous names that need a
/// definition) and are outside the check: [`check_defined_terms`] catches both of them; all
/// three recorded requests are replayed in `delivery_verb_replay_over_the_cited_jobs` in
/// `crates/athena-core/tests/refusal_corpus.rs`, which asserts the count. The check reads only the request and the reading, costs no model call and, like the governance check,
/// skips a reading with no capability, a question, or no deliverable. A verb is one of `publish`, `deploy`,
/// `share` and `release` at the start of a segment (sentence, line or list part such as `2)`, `(a)`, `1.`, `-`
/// or `*`) or after `and`, `then`, `please`, `also`, `should`, `can you`, `could you`, `would you`, `want to`,
/// `need to`, `like to` or `going to`, with a delivery object near it: a page, site, docs, guide, report or
/// similar, a version such as `v1`, a destination such as `crates.io` or `production`, or an audience such as
/// the team. A crate, package, binary, app, image or artifact is not an object by itself, and a following word
/// such as `notes` or `button` makes the verb a noun, so "release the lock", "share the HTTP client" and "add
/// release notes" are not delivery. The reading passes when a criterion of kind behavior holds a whole-word
/// state phrase (a reachable URL, a served page, an HTTP 200 answer, a downloadable release), or when every
/// detected verb or its `-ing` form appears as a whole word in an out_of_scope item ("Publishing is out of
/// scope"); a content criterion that mentions a state still checks only a file, so it does not pass. The error
/// names the verbs and the two repairs. Limits: other verbs (ship, launch, upload), a verb inside a comma list
/// or as a gerund start, an object not in the lists, `reachable` said of a file, and a false refusal for a
/// technical use with a visible object ("release the page lock"). This check refuses more readings; the higher
/// refusal rate is intended. The expected share of newly refused readings is 5%, an estimate and not a
/// measurement: 29 of the 1045 recorded requests contain one of the four words anywhere, most as the noun
/// "share", which bounds the share from above.
pub fn check_outcome_state(reading: &Reading, request: &str) -> Result<()> {
    if reading.capability.is_none() || reading.question.is_some() || reading.deliverables.is_empty()
    {
        return Ok(());
    }
    let verbs = delivery_verbs(request);
    if verbs.is_empty() {
        return Ok(());
    }
    if reading
        .acceptance_criteria
        .iter()
        .any(|c| matches!(c.kind, CriterionKind::Behavior) && names_outcome_state(&c.text))
    {
        return Ok(());
    }
    let uncovered: Vec<&str> = verbs
        .iter()
        .filter(|(verb, ing)| {
            !reading
                .out_of_scope
                .iter()
                .any(|item| out_of_scope_names(item, verb, ing))
        })
        .map(|(verb, _)| *verb)
        .collect();
    if uncovered.is_empty() {
        return Ok(());
    }
    bail!(
        "the reading was refused because the request asks to {} but no behavior criterion names a state the \
         audience can observe, so the work would be judged on a file that exists, not on what the user sees; a \
         content criterion that mentions a state still checks only a file. Repair: add a criterion of kind \
         behavior that names the state, such as a reachable URL that answers, a served page that loads or a \
         downloadable release; or, if this job must not deliver, name the delivery in out_of_scope (for \
         example \"Publishing is out of scope\")",
        uncovered.join(" and ")
    )
}
