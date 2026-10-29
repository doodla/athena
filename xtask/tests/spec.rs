//! The cohesive spec (decision D114, enforcement row M33): `docs/spec.toml` turns BRIEF.md
//! "Project Olympus" sections 1 to 10, the governing philosophy and the addenda dated 2026-10-17 to
//! 2026-10-24 into sourced requirements, and this test checks it.
//!
//! A source quote passes only if, after whitespace is collapsed, it is a substring of the text under
//! the heading it names (from that heading line to the next heading line of any level). The checker is
//! a pure function of the parsed spec and the brief, so each rule has an in-memory negative case.

use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Spec {
    #[serde(default)]
    requirement: Vec<Requirement>,
    #[serde(default)]
    open_question: Vec<OpenQuestion>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Requirement {
    id: String,
    statement: String,
    kind: Kind,
    status: Status,
    #[serde(default)]
    supersedes: Vec<String>,
    #[serde(default)]
    source: Vec<Source>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct OpenQuestion {
    id: String,
    question: String,
    #[serde(default)]
    side: Vec<Source>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    heading: String,
    quote: String,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Goal,
    Requirement,
    Constraint,
    Acceptance,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Status {
    Active,
    Superseded,
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn real_spec() -> Spec {
    let text = std::fs::read_to_string(root().join("docs/spec.toml")).unwrap();
    toml::from_str(&text).unwrap()
}

fn real_brief() -> String {
    std::fs::read_to_string(root().join("BRIEF.md")).unwrap()
}

/// Whitespace runs (including line breaks) become one space; the ends are trimmed.
fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One heading of the Olympus part of the brief with the collapsed text under it.
struct Section {
    heading: String,
    /// The number of the `## N.` section it falls in, or `None` before section 1.
    number: Option<u32>,
    body: String,
}

/// The last addendum date the spec covers; a heading dated later ends the text the spec may cite.
const LAST_DATE: &str = "2026-10-24";

/// The `YYYY-MM-DD` a heading starts with, if it starts with one.
fn heading_date(heading: &str) -> Option<&str> {
    let date = heading.get(..10)?;
    let shaped = date.char_indices().all(|(i, c)| match i {
        4 | 7 => c == '-',
        _ => c.is_ascii_digit(),
    });
    (shaped && heading[10..].starts_with(':')).then_some(date)
}

/// Cut the Olympus part of the brief (from `# Project Olympus` through the addenda dated up to
/// `LAST_DATE`) at every heading line, whatever its level.
fn sections(brief: &str) -> Vec<Section> {
    let mut out: Vec<Section> = Vec::new();
    let mut number = None;
    let mut started = false;
    for line in brief.lines() {
        if line == "# Project Olympus" {
            started = true;
        }
        if !started {
            continue;
        }
        let marks = line.chars().take_while(|c| *c == '#').count();
        if marks > 0
            && line[marks..].starts_with(' ')
            && heading_date(line[marks..].trim()).is_some_and(|d| d > LAST_DATE)
        {
            break;
        }
        if marks > 0 && line[marks..].starts_with(' ') {
            let heading = line[marks..].trim().to_string();
            if marks == 2 {
                // A `## Governing philosophy` or `## Addenda` heading belongs to no numbered section.
                number = heading.split('.').next().and_then(|n| n.parse().ok());
            }
            out.push(Section {
                heading,
                number,
                body: String::new(),
            });
        } else if let Some(last) = out.last_mut() {
            last.body.push(' ');
            last.body.push_str(line);
        }
    }
    for s in &mut out {
        s.body = collapse(&s.body);
    }
    out
}

/// Whether a heading cited in the spec names this brief heading: the same text, or a cited start ending
/// in "…" (kept for a heading whose full text holds a word the boundary scan rejects). A start that
/// matches more than one heading is reported as not unique by `check_source`.
fn names_heading(cited: &str, actual: &str) -> bool {
    match cited.strip_suffix('…') {
        Some(start) => !start.is_empty() && actual.starts_with(start),
        None => cited == actual,
    }
}

fn check_source(source: &Source, who: &str, secs: &[Section], errors: &mut Vec<String>) {
    let quote = collapse(&source.quote);
    if quote.is_empty() {
        errors.push(format!("{who}: an empty quote"));
        return;
    }
    let under: Vec<&Section> = secs
        .iter()
        .filter(|s| names_heading(&source.heading, &s.heading))
        .collect();
    match under.as_slice() {
        [] => errors.push(format!(
            "{who}: no heading {:?} in the Olympus part of the brief",
            source.heading
        )),
        [one] => {
            if !one.body.contains(&quote) {
                errors.push(format!(
                    "{who}: quote is not verbatim under heading {:?}: {quote:?}",
                    source.heading
                ));
            }
        }
        _ => errors.push(format!("{who}: heading {:?} is not unique", source.heading)),
    }
}

/// Every rule of the spec, as a list of errors; empty means the spec passes.
fn check(spec: &Spec, brief: &str) -> Vec<String> {
    let secs = sections(brief);
    let mut errors = Vec::new();
    let mut ids = BTreeSet::new();
    for r in &spec.requirement {
        if !ids.insert(r.id.as_str()) {
            errors.push(format!("{}: duplicate id", r.id));
        }
        if r.statement.trim().is_empty() {
            errors.push(format!("{}: an empty statement", r.id));
        }
        if r.source.is_empty() {
            errors.push(format!("{}: no source", r.id));
        }
        for s in &r.source {
            check_source(s, &r.id, &secs, &mut errors);
        }
    }
    for r in &spec.requirement {
        for target in &r.supersedes {
            if !ids.contains(target.as_str()) {
                errors.push(format!(
                    "{}: supersedes {target}, which does not exist",
                    r.id
                ));
            }
        }
    }
    for q in &spec.open_question {
        if !ids.insert(q.id.as_str()) {
            errors.push(format!("{}: duplicate id", q.id));
        }
        if q.question.trim().is_empty() {
            errors.push(format!("{}: an empty question", q.id));
        }
        if q.side.len() < 2 {
            errors.push(format!(
                "{}: an open question needs two or more sides",
                q.id
            ));
        }
        for s in &q.side {
            check_source(s, &q.id, &secs, &mut errors);
        }
    }
    errors
}

/// Which of R1 to R8, sections 1 to 5 and 7 to 10, the governing philosophy, and each addendum dated
/// up to `LAST_DATE` have no requirement citing text under them.
fn coverage_gaps(spec: &Spec, brief: &str) -> Vec<String> {
    let secs = sections(brief);
    let cites = |pred: &dyn Fn(&Section) -> bool| {
        spec.requirement
            .iter()
            .filter_map(|r| r.source.first())
            .any(|s| {
                secs.iter()
                    .any(|sec| names_heading(&s.heading, &sec.heading) && pred(sec))
            })
    };
    let mut missing = Vec::new();
    for n in (1..=5u32).chain(7..=10) {
        if !cites(&|sec| sec.number == Some(n)) {
            missing.push(format!("section {n}"));
        }
    }
    if !cites(&|sec| sec.heading == "Governing philosophy") {
        missing.push("Governing philosophy".to_string());
    }
    for addendum in secs.iter().filter(|s| heading_date(&s.heading).is_some()) {
        if !cites(&|sec| sec.heading == addendum.heading) {
            missing.push(format!("addendum {}", addendum.heading));
        }
    }
    for n in 1..=8 {
        let prefix = format!("R{n} ");
        let covered = spec
            .requirement
            .iter()
            .filter_map(|r| r.source.first())
            .any(|s| s.heading.starts_with(&prefix));
        if !covered {
            missing.push(format!("R{n}"));
        }
    }
    missing
}

fn has_error(errors: &[String], needle: &str) -> bool {
    errors.iter().any(|e| e.contains(needle))
}

#[test]
fn the_real_spec_passes() {
    let errors = check(&real_spec(), &real_brief());
    assert!(errors.is_empty(), "{errors:#?}");
}

#[test]
fn the_real_spec_covers_r1_to_r8_and_sections_1_to_5() {
    // The name is cited by docs/enforcement.md; the coverage now also spans sections 7 to 10, the
    // governing philosophy and every addendum heading up to LAST_DATE.
    let gaps = coverage_gaps(&real_spec(), &real_brief());
    assert!(gaps.is_empty(), "no requirement for: {gaps:?}");
}

#[test]
fn every_heading_the_spec_uses_appears_once_in_the_brief() {
    let secs = sections(&real_brief());
    for r in real_spec().requirement {
        for s in r.source {
            let n = secs
                .iter()
                .filter(|sec| names_heading(&s.heading, &sec.heading))
                .count();
            assert_eq!(n, 1, "{}: heading {:?}", r.id, s.heading);
        }
    }
}

#[test]
fn a_quote_that_is_not_in_the_brief_fails() {
    let mut spec = real_spec();
    spec.requirement[0].source[0].quote.push_str(" and more");
    assert!(has_error(&check(&spec, &real_brief()), "not verbatim"));
}

#[test]
fn a_quote_that_differs_only_in_whitespace_passes() {
    let mut spec = real_spec();
    let q = &mut spec.requirement[0].source[0].quote;
    *q = q.replace(' ', "\n  ");
    assert!(check(&spec, &real_brief()).is_empty());
}

#[test]
fn a_real_quote_under_the_wrong_heading_fails() {
    let mut spec = real_spec();
    // Real text of section 1, claimed to sit in section 2.
    spec.requirement[0].source[0].heading = "2. The Pantheon".into();
    assert!(has_error(
        &check(&spec, &real_brief()),
        "not verbatim under heading"
    ));
}

#[test]
fn a_subsection_quote_under_its_parent_heading_fails() {
    let mut spec = real_spec();
    let r = spec
        .requirement
        .iter_mut()
        .find(|r| r.id == "OLY-3-04")
        .unwrap();
    assert_eq!(r.source[0].heading, "3.1 Preserve the original signal");
    r.source[0].heading = "3. Feedback is a first-class engineering input".into();
    assert!(has_error(&check(&spec, &real_brief()), "OLY-3-04"));
}

#[test]
fn a_heading_that_is_not_in_the_brief_fails() {
    let mut spec = real_spec();
    spec.requirement[0].source[0].heading = "99. Nothing".into();
    assert!(has_error(&check(&spec, &real_brief()), "no heading"));
}

#[test]
fn a_duplicate_id_fails() {
    let mut spec = real_spec();
    let copy = spec.requirement[0].clone();
    spec.requirement.push(copy);
    assert!(has_error(&check(&spec, &real_brief()), "duplicate id"));
}

#[test]
fn a_dangling_supersedes_fails() {
    let mut spec = real_spec();
    spec.requirement[0].supersedes = vec!["OLY-9-99".into()];
    assert!(has_error(&check(&spec, &real_brief()), "does not exist"));
}

#[test]
fn a_supersedes_target_that_exists_passes() {
    let mut spec = real_spec();
    let id = spec.requirement[1].id.clone();
    spec.requirement[0].supersedes = vec![id];
    spec.requirement[1].status = Status::Superseded;
    assert!(check(&spec, &real_brief()).is_empty());
}

#[test]
fn a_requirement_without_a_source_fails() {
    let mut spec = real_spec();
    spec.requirement[0].source.clear();
    assert!(has_error(&check(&spec, &real_brief()), "no source"));
}

#[test]
fn a_missing_requirement_for_a_section_or_r_fails() {
    let brief = real_brief();
    let mut spec = real_spec();
    spec.requirement
        .retain(|r| !r.source.iter().any(|s| s.heading.starts_with("R3 ")));
    assert_eq!(coverage_gaps(&spec, &brief), vec!["R3".to_string()]);

    let mut spec = real_spec();
    spec.requirement.retain(|r| !r.id.starts_with("OLY-2-"));
    assert!(coverage_gaps(&spec, &brief).contains(&"section 2".to_string()));
}

#[test]
fn a_quote_that_is_not_verbatim_under_a_section_7_or_later_heading_fails() {
    let mut spec = real_spec();
    for id in ["OLY-7-01", "OLY-8-03", "OLY-9-01", "OLY-GP-01", "OLY-A-14"] {
        let r = spec.requirement.iter_mut().find(|r| r.id == id).unwrap();
        r.source[0].quote.push_str(" and more");
        assert!(has_error(&check(&spec, &real_brief()), id), "{id}");
    }
}

#[test]
fn a_missing_section_8_requirement_is_a_coverage_gap() {
    let mut spec = real_spec();
    spec.requirement.retain(|r| !r.id.starts_with("OLY-8-"));
    assert_eq!(
        coverage_gaps(&spec, &real_brief()),
        vec!["section 8".to_string()]
    );
}

#[test]
fn a_missing_philosophy_or_addendum_requirement_is_a_coverage_gap() {
    let brief = real_brief();
    let mut spec = real_spec();
    spec.requirement.retain(|r| !r.id.starts_with("OLY-GP-"));
    assert_eq!(
        coverage_gaps(&spec, &brief),
        vec!["Governing philosophy".to_string()]
    );

    let mut spec = real_spec();
    spec.requirement.retain(|r| r.id != "OLY-A-21");
    assert_eq!(
        coverage_gaps(&spec, &brief),
        vec!["addendum 2026-10-24: Athena keeps track".to_string()]
    );
}

#[test]
fn a_heading_dated_after_the_last_covered_addendum_is_rejected() {
    let brief = real_brief();
    assert!(brief.contains("### 2026-10-25: Speed first"));
    assert!(
        sections(&brief)
            .iter()
            .all(|s| heading_date(&s.heading).is_none_or(|d| d <= LAST_DATE))
    );
    let mut spec = real_spec();
    spec.requirement[0].source[0].heading = "2026-10-25: Speed first".into();
    assert!(has_error(&check(&spec, &brief), "no heading"));
}

#[test]
fn a_cited_heading_start_must_match_exactly_one_heading() {
    let brief = real_brief();
    let mut spec = real_spec();
    spec.requirement[0].source[0].heading = "2026-10-17:…".into();
    assert!(has_error(&check(&spec, &brief), "is not unique"));
    spec.requirement[0].source[0].heading = "2026-10-99:…".into();
    assert!(has_error(&check(&spec, &brief), "no heading"));
    spec.requirement[0].source[0].heading = "…".into();
    assert!(has_error(&check(&spec, &brief), "no heading"));
}

#[test]
fn every_open_question_has_two_sides_quoting_different_statements() {
    let spec = real_spec();
    assert!(!spec.open_question.is_empty());
    for q in spec.open_question {
        assert!(q.side.len() >= 2, "{}", q.id);
        assert_ne!(
            collapse(&q.side[0].quote),
            collapse(&q.side[1].quote),
            "{}",
            q.id
        );
    }
}

#[test]
fn open_question_sides_are_checked_like_sources() {
    let brief = real_brief();
    let good = Source {
        heading: "1. Vision".into(),
        quote: "Its defining capability is turning observations".into(),
    };
    let mut spec = real_spec();
    spec.open_question.push(OpenQuestion {
        id: "OQ-TEST".into(),
        question: "A question quoting two sides.".into(),
        side: vec![good.clone(), good.clone()],
    });
    let last = spec.open_question.len() - 1;
    assert!(check(&spec, &brief).is_empty());

    let mut bad = good.clone();
    bad.quote = "words the brief never says".into();
    spec.open_question[last].side[1] = bad;
    assert!(has_error(&check(&spec, &brief), "OQ-TEST"));

    spec.open_question[last].side.truncate(1);
    assert!(has_error(&check(&spec, &brief), "two or more sides"));
}

#[test]
fn an_unknown_kind_or_status_does_not_parse() {
    let text = |kind: &str, status: &str| {
        format!(
            "[[requirement]]\nid = \"X\"\nstatement = \"s\"\nkind = \"{kind}\"\nstatus = \"{status}\"\n"
        )
    };
    assert!(toml::from_str::<Spec>(&text("goal", "active")).is_ok());
    assert!(toml::from_str::<Spec>(&text("wish", "active")).is_err());
    assert!(toml::from_str::<Spec>(&text("goal", "maybe")).is_err());
}

#[test]
fn every_kind_and_status_is_in_use_or_valid() {
    // The enums are closed; this reads each requirement's fields so a variant cannot be dropped silently.
    for r in real_spec().requirement {
        assert!(matches!(
            r.kind,
            Kind::Goal | Kind::Requirement | Kind::Constraint | Kind::Acceptance
        ));
        assert!(matches!(r.status, Status::Active | Status::Superseded));
        assert!(!r.statement.trim().is_empty());
    }
}
