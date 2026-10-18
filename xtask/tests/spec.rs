//! Checks `docs/spec.toml` against `BRIEF.md` (D111): every quote is a verbatim passage of the brief, ids are
//! unique, `supersedes` targets exist, every requirement has a source, and the spec covers the Olympus
//! sections 1 to 6 it claims to.

use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Spec {
    #[serde(default)]
    no_open_questions_reason: Option<String>,
    #[serde(default)]
    requirement: Vec<Requirement>,
    #[serde(default)]
    open_question: Vec<OpenQuestion>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Requirement {
    id: String,
    statement: String,
    kind: String,
    status: String,
    #[serde(default)]
    supersedes: Vec<String>,
    #[serde(default)]
    source: Vec<Source>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OpenQuestion {
    id: String,
    question: String,
    #[serde(default)]
    source: Vec<Source>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    quote: String,
    #[serde(default)]
    heading: String,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn spec_text() -> String {
    fs::read_to_string(root().join("docs/spec.toml")).unwrap()
}

fn brief_text() -> String {
    fs::read_to_string(root().join("BRIEF.md")).unwrap()
}

fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Every problem found in `spec_text` when checked against `brief_text`; empty when the spec is sound.
fn check(spec_text: &str, brief_text: &str) -> Vec<String> {
    let spec: Spec = match toml::from_str(spec_text) {
        Ok(spec) => spec,
        Err(e) => return vec![format!("the spec does not parse: {e}")],
    };
    let brief = collapse(brief_text);
    let headings: HashSet<String> = brief_text
        .lines()
        .filter(|l| l.starts_with('#'))
        .map(|l| collapse(l.trim_start_matches('#')))
        .collect();
    let mut problems = Vec::new();

    let check_source = |owner: &str, s: &Source, needs_heading: bool| {
        let mut found = Vec::new();
        let quote = collapse(&s.quote);
        if quote.is_empty() || !brief.contains(&quote) {
            found.push(format!("{owner}: quote is not in BRIEF.md: {}", s.quote));
        }
        if needs_heading {
            let last = s.heading.rsplit(" / ").next().unwrap_or("");
            if !headings.contains(&collapse(last)) {
                found.push(format!(
                    "{owner}: heading is not a heading of BRIEF.md: {}",
                    s.heading
                ));
            }
        }
        found
    };

    let mut ids = HashSet::new();
    for r in &spec.requirement {
        if !ids.insert(r.id.as_str()) {
            problems.push(format!("duplicate id {}", r.id));
        }
        if r.statement.trim().is_empty() {
            problems.push(format!("{}: empty statement", r.id));
        }
        if !["goal", "requirement", "constraint", "acceptance"].contains(&r.kind.as_str()) {
            problems.push(format!("{}: unknown kind {}", r.id, r.kind));
        }
        if !["active", "superseded"].contains(&r.status.as_str()) {
            problems.push(format!("{}: unknown status {}", r.id, r.status));
        }
        if r.source.is_empty() {
            problems.push(format!("{}: no source", r.id));
        }
        for s in &r.source {
            problems.extend(check_source(&r.id, s, true));
        }
    }
    for r in &spec.requirement {
        for target in &r.supersedes {
            if !ids.contains(target.as_str()) {
                problems.push(format!("{}: supersedes missing id {target}", r.id));
            }
        }
    }
    for q in &spec.open_question {
        if !ids.insert(q.id.as_str()) {
            problems.push(format!("duplicate id {}", q.id));
        }
        if q.question.trim().is_empty() {
            problems.push(format!("{}: empty question", q.id));
        }
        if q.source.len() < 2 {
            problems.push(format!("{}: an open question needs two quotes", q.id));
        }
        for s in &q.source {
            problems.extend(check_source(&q.id, s, false));
        }
    }
    if spec.open_question.is_empty()
        && spec
            .no_open_questions_reason
            .as_deref()
            .is_none_or(|r| r.trim().is_empty())
    {
        problems.push("no open question and no no_open_questions_reason".to_string());
    }
    problems
}

fn has(problems: &[String], needle: &str) -> bool {
    problems.iter().any(|p| p.contains(needle))
}

#[test]
fn the_spec_matches_the_brief() {
    let problems = check(&spec_text(), &brief_text());
    assert!(problems.is_empty(), "{problems:#?}");
}

#[test]
fn a_quote_not_in_the_brief_is_flagged() {
    let spec = spec_text().replacen(
        "quote = \"Neither loop owns the other.\"",
        "quote = \"Each loop owns the other.\"",
        1,
    );
    let problems = check(&spec, &brief_text());
    assert!(has(&problems, "Each loop owns the other."), "{problems:?}");
}

#[test]
fn a_duplicate_id_is_flagged() {
    let spec = spec_text().replacen("id = \"V-02\"", "id = \"V-01\"", 1);
    let problems = check(&spec, &brief_text());
    assert!(has(&problems, "duplicate id V-01"), "{problems:?}");
}

#[test]
fn a_missing_supersedes_target_is_flagged() {
    let spec = spec_text().replacen(
        "id = \"V-02\"\n",
        "id = \"V-02\"\nsupersedes = [\"X-99\"]\n",
        1,
    );
    let problems = check(&spec, &brief_text());
    assert!(has(&problems, "supersedes missing id X-99"), "{problems:?}");
}

#[test]
fn a_supersedes_target_that_exists_is_accepted() {
    let spec = spec_text().replacen(
        "id = \"V-02\"\n",
        "id = \"V-02\"\nsupersedes = [\"V-01\"]\n",
        1,
    );
    assert!(check(&spec, &brief_text()).is_empty());
}

#[test]
fn a_requirement_without_a_source_is_flagged() {
    let spec = "[[requirement]]\nid = \"A-1\"\nstatement = \"x\"\nkind = \"goal\"\nstatus = \"active\"\n\n\
                [[open_question]]\nid = \"Q\"\nquestion = \"q\"\n";
    let problems = check(spec, &brief_text());
    assert!(has(&problems, "A-1: no source"), "{problems:?}");
}

#[test]
fn an_unknown_kind_or_status_is_flagged() {
    let spec = spec_text()
        .replacen("kind = \"goal\"", "kind = \"wish\"", 1)
        .replacen("status = \"active\"", "status = \"gone\"", 1);
    let problems = check(&spec, &brief_text());
    assert!(has(&problems, "unknown kind wish"), "{problems:?}");
    assert!(has(&problems, "unknown status gone"), "{problems:?}");
}

#[test]
fn an_invented_heading_is_flagged() {
    let spec = spec_text().replacen("heading = \"1. Vision\"", "heading = \"1. Invented\"", 1);
    let problems = check(&spec, &brief_text());
    assert!(has(&problems, "1. Invented"), "{problems:?}");
}

#[test]
fn a_spec_with_neither_open_question_nor_reason_is_flagged() {
    let problems = check("", &brief_text());
    assert!(has(&problems, "no_open_questions_reason"), "{problems:?}");
}

#[test]
fn an_open_question_quote_not_in_the_brief_is_flagged() {
    let spec = "[[open_question]]\nid = \"Q-1\"\nquestion = \"which?\"\n\
                [[open_question.source]]\nquote = \"Neither loop owns the other.\"\n\
                [[open_question.source]]\nquote = \"A quote Sri never wrote.\"\n";
    let problems = check(spec, &brief_text());
    assert!(has(&problems, "A quote Sri never wrote."), "{problems:?}");
    assert!(!has(&problems, "Neither loop"), "{problems:?}");
}

#[test]
fn an_open_question_with_verbatim_quotes_is_accepted() {
    let spec = "[[open_question]]\nid = \"Q-1\"\nquestion = \"which?\"\n\
                [[open_question.source]]\nquote = \"Neither loop owns the other.\"\n\
                [[open_question.source]]\nquote = \"Roles must cooperate through explicit contracts.\"\n";
    assert!(check(spec, &brief_text()).is_empty());
}

#[test]
fn every_requirement_area_has_a_requirement() {
    let spec: Spec = toml::from_str(&spec_text()).unwrap();
    let count = |prefix: &str| {
        spec.requirement
            .iter()
            .filter(|r| r.source.iter().any(|s| s.heading.contains(prefix)))
            .count()
    };
    let areas = [
        "1. Vision",
        "2. The Pantheon",
        "3. Feedback is a first-class engineering input",
        "4. Example:",
        "5. A unified engineering and learning lifecycle",
        "R1 — Causal investigation",
        "R2 — Feedback-to-work decomposition",
        "R3 — Generalizable prevention",
        "R4 — Independent evaluation",
        "R5 — Governed self-modification",
        "R6 — Measured experiments",
        "R7 — Persistent, reconstructable knowledge",
        "R8 — Adaptive specialization",
    ];
    for area in areas {
        assert!(
            count(area) >= 1,
            "no requirement cites a source under {area}"
        );
    }
}
