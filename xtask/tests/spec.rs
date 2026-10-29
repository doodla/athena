//! The cohesive spec (decision D114, enforcement row M33): `docs/spec.toml` turns every part of BRIEF.md
//! (the opening goals and operating model, Part 1, Parts 2 and 3, "Project Olympus" sections 1 to 10, the
//! governing philosophy and every dated addendum) into sourced requirements, and this test checks it.
//!
//! A source quote passes only if, after whitespace is collapsed, it is a substring of the text under
//! the heading it names (from that heading line to the next heading line of any level). The checker is
//! a pure function of the parsed spec and the brief, so each rule has an in-memory negative case.
//!
//! Coverage: every heading of the brief that has text under it must be the heading of the first source of
//! some requirement (`coverage_gaps`). A heading with nothing under it (the title, the part and group
//! headings, `Addenda`) has nothing to cite and is exempt; `the_exempt_headings_are_only_structural` pins
//! that list so a heading cannot drop out of coverage by losing its text unnoticed.
//!
//! Every active requirement also records how it is met: a `mechanism` (the code path, role or check, or the
//! literal `none`), an `owner` (a role) and a `measure` (how it is shown, ending with `status: <label>`). A
//! missing one fails the test. The label is checked against the labels `athena status` prints by the Hestia
//! spec review in `crates/athena-core/src/hestia.rs`, which also queues a request for a mechanism of `none`.

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
    /// The code path, role or check that meets the requirement, or the literal `none`.
    #[serde(default)]
    mechanism: String,
    /// The role responsible for the requirement.
    #[serde(default)]
    owner: String,
    /// How the requirement is shown, ending with `status: <label>` (where it appears in `athena status`) or
    /// `status: not shown` and the test or check that shows it.
    #[serde(default)]
    measure: String,
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

/// One heading of the brief with the collapsed text under it.
struct Section {
    heading: String,
    body: String,
}

/// The `YYYY-MM-DD` a heading starts with, if it starts with one.
fn heading_date(heading: &str) -> Option<&str> {
    let date = heading.get(..10)?;
    let shaped = date.char_indices().all(|(i, c)| match i {
        4 | 7 => c == '-',
        _ => c.is_ascii_digit(),
    });
    (shaped && heading[10..].starts_with(':')).then_some(date)
}

/// Cut the whole brief at every heading line, whatever its level. Text before the first heading belongs to
/// no section.
fn sections(brief: &str) -> Vec<Section> {
    let mut out: Vec<Section> = Vec::new();
    for line in brief.lines() {
        let marks = line.chars().take_while(|c| *c == '#').count();
        let is_heading = marks > 0 && line[marks..].starts_with(' ');
        if is_heading {
            out.push(Section {
                heading: line[marks..].trim().to_string(),
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
            "{who}: no heading {:?} in the brief",
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

/// Whether a field holds nothing, or only the word `none`.
fn blank_or_none(text: &str) -> bool {
    let t = text.trim();
    t.is_empty() || t.eq_ignore_ascii_case("none")
}

/// What an active requirement must record about how it is met: a mechanism (the literal `none` when nothing
/// meets it), an owner that is a role and not `none`, and a measure that ends with `status: <label>`. The
/// Hestia sweep (`Athena::spec_review`) queues a request for a mechanism of `none` and for a measure whose
/// label `athena status` does not print; this test only requires that the fields are there.
fn check_how_it_is_met(r: &Requirement, errors: &mut Vec<String>) {
    if r.mechanism.trim().is_empty() {
        errors.push(format!(
            "{}: no mechanism (name the code path, role or check, or write none)",
            r.id
        ));
    }
    if blank_or_none(&r.owner) {
        errors.push(format!("{}: no owner (name the responsible role)", r.id));
    }
    let label = r.measure.rsplit_once("status:").map(|(_, l)| l.trim());
    if blank_or_none(&r.measure) {
        errors.push(format!("{}: no measure", r.id));
    } else if label.is_none_or(str::is_empty) {
        errors.push(format!(
            "{}: no measure label (end the measure with `status: <label>` or `status: not shown`)",
            r.id
        ));
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
        if matches!(r.status, Status::Active) {
            check_how_it_is_met(r, &mut errors);
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

/// The headings of the brief that have text under them, each paired with whether it is a dated addendum.
fn citable_headings(brief: &str) -> Vec<(String, bool)> {
    sections(brief)
        .into_iter()
        .filter(|s| !s.body.is_empty())
        .map(|s| {
            let dated = heading_date(&s.heading).is_some();
            (s.heading, dated)
        })
        .collect()
}

/// The headings of the brief with nothing under them, which have nothing to cite.
fn exempt_headings(brief: &str) -> Vec<String> {
    sections(brief)
        .into_iter()
        .filter(|s| s.body.is_empty())
        .map(|s| s.heading)
        .collect()
}

/// Every heading of the brief that has text under it (a numbered section, a part item such as A6 or S3, a
/// requirement R1 to R8, the governing philosophy, a dated addendum, and so on) and is not the heading of the
/// first source of any requirement. A dated addendum is reported as `addendum <heading>`, any other heading
/// as `heading <heading>`.
fn coverage_gaps(spec: &Spec, brief: &str) -> Vec<String> {
    let cited: Vec<&str> = spec
        .requirement
        .iter()
        .filter_map(|r| r.source.first())
        .map(|s| s.heading.as_str())
        .collect();
    citable_headings(brief)
        .into_iter()
        .filter(|(heading, _)| !cited.iter().any(|c| names_heading(c, heading)))
        .map(|(heading, dated)| {
            let kind = if dated { "addendum" } else { "heading" };
            format!("{kind} {heading}")
        })
        .collect()
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
fn the_real_spec_covers_every_heading_and_addendum_of_the_brief() {
    // The name is cited by docs/enforcement.md.
    let gaps = coverage_gaps(&real_spec(), &real_brief());
    assert!(gaps.is_empty(), "no requirement for: {gaps:?}");
}

#[test]
fn the_exempt_headings_are_only_structural() {
    // These have nothing under them to cite. A heading that loses its text drops out of coverage, so the
    // list is pinned: a change to it is a decision, not an accident.
    let exempt = exempt_headings(&real_brief());
    assert_eq!(
        exempt,
        [
            "Athena",
            "The operating model (your requirements)",
            "Part 1: Athena (the main deliverable)",
            "Part 2: The first big task: the study",
            "Part 3: Deliverables",
            "Addenda",
            "6. Requirements for systemic learning",
            "9. Implementation plan",
            "Addenda",
        ]
    );
}

#[test]
fn the_coverage_reads_the_whole_brief_including_early_and_late_addenda() {
    let headings = citable_headings(&real_brief());
    let has = |h: &str| headings.iter().any(|(x, _)| x == h);
    for h in [
        "Goal",
        "A6. Efficiency",
        "2026-10-12: Chunking and parallel work",
        "2026-10-13: Athena owns acceleration",
        "2026-10-26: Written for agents",
        "Project Olympus",
    ] {
        assert!(has(h), "{h}");
    }
    assert!(
        headings
            .iter()
            .any(|(h, dated)| *dated && h.starts_with("2026-10-10:"))
    );
    assert!(!has("Addenda"));
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
    assert_eq!(
        coverage_gaps(&spec, &brief),
        vec!["heading R3 — Generalizable prevention".to_string()]
    );

    let mut spec = real_spec();
    spec.requirement.retain(|r| !r.id.starts_with("OLY-2-"));
    assert!(coverage_gaps(&spec, &brief).contains(&"heading 2. The Pantheon".to_string()));
}

#[test]
fn removing_the_requirements_of_one_brief_heading_reports_exactly_that_heading() {
    let brief = real_brief();
    // The negative case for the new coverage: a part item (A6), a pre-Olympus addendum (the chunking note of
    // 2026-10-12), a late addendum, and the opening goal about getting measurably faster.
    for (heading, expected) in [
        ("A6. Efficiency", "heading A6. Efficiency"),
        (
            "2026-10-12: Chunking and parallel work",
            "addendum 2026-10-12: Chunking and parallel work",
        ),
        (
            "2026-10-26: Written for agents",
            "addendum 2026-10-26: Written for agents",
        ),
        ("Goal", "heading Goal"),
        ("How I'll judge it", "heading How I'll judge it"),
    ] {
        let mut spec = real_spec();
        let before = spec.requirement.len();
        spec.requirement
            .retain(|r| r.source.first().is_none_or(|s| s.heading != heading));
        assert!(
            spec.requirement.len() < before,
            "{heading} had requirements"
        );
        assert_eq!(coverage_gaps(&spec, &brief), vec![expected.to_string()]);
    }
}

#[test]
fn a_heading_cited_only_by_a_later_source_is_still_a_coverage_gap() {
    // Coverage looks at the first source, as the earlier layers do: a requirement that cites a heading only
    // as a second source does not cover it.
    let brief = real_brief();
    let mut spec = real_spec();
    for r in &mut spec.requirement {
        if r.source[0].heading == "A6. Efficiency" {
            r.source[0].heading = "A7. Adversarial verification".into();
            r.source[0].quote = "Everything substantive is verified before it's accepted".into();
        }
    }
    assert_eq!(
        coverage_gaps(&spec, &brief),
        vec!["heading A6. Efficiency".to_string()]
    );
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
        vec!["heading 8. Coordination and authority".to_string()]
    );
}

#[test]
fn a_missing_philosophy_or_addendum_requirement_is_a_coverage_gap() {
    let brief = real_brief();
    let mut spec = real_spec();
    spec.requirement.retain(|r| !r.id.starts_with("OLY-GP-"));
    assert_eq!(
        coverage_gaps(&spec, &brief),
        vec!["heading Governing philosophy".to_string()]
    );

    let mut spec = real_spec();
    spec.requirement.retain(|r| r.id != "OLY-A-21");
    assert_eq!(
        coverage_gaps(&spec, &brief),
        vec!["addendum 2026-10-24: Athena keeps track".to_string()]
    );
}

#[test]
fn a_heading_that_is_not_in_the_brief_is_rejected_even_when_it_is_dated() {
    let brief = real_brief();
    let mut spec = real_spec();
    spec.requirement[0].source[0].heading = "2026-10-27: Not in the brief".into();
    assert!(has_error(&check(&spec, &brief), "no heading"));
}

#[test]
fn early_and_late_headings_and_the_goal_are_citable() {
    let brief = real_brief();
    let secs = sections(&brief);
    for heading in [
        "2026-10-13: Athena owns acceleration",
        "2026-10-25: Speed first",
        "2026-10-25: Evidence labels Jev",
        "2026-10-12: Chunking and parallel work",
        "Goal",
    ] {
        assert_eq!(secs.iter().filter(|s| s.heading == heading).count(), 1);
        let kept = secs.iter().find(|s| s.heading == heading).unwrap();
        assert!(!kept.body.is_empty(), "{heading} keeps its text");
    }
    let mut spec = real_spec();
    for (heading, quote) in [
        (
            "2026-10-13: Athena owns acceleration",
            "give Athena the mandate to accelerate",
        ),
        (
            "2026-10-25: Speed first",
            "Athena should prioritize anything that'll help it move faster",
        ),
        (
            "2026-10-25: Evidence labels Jev",
            "Evidence should label jev",
        ),
        (
            "2026-10-12: Chunking and parallel work",
            "Do we chunk and parallelize today?",
        ),
        (
            "Goal",
            "gets measurably faster, cheaper, and more correct at work it has done before",
        ),
    ] {
        spec.requirement[0].source[0].heading = heading.into();
        spec.requirement[0].source[0].quote = quote.into();
        assert!(check(&spec, &brief).is_empty(), "{heading}");
        spec.requirement[0].source[0].quote = "words the brief never says".into();
        assert!(
            has_error(&check(&spec, &brief), "not verbatim"),
            "{heading}"
        );
    }
}

fn find<'a>(spec: &'a Spec, id: &str) -> &'a Requirement {
    spec.requirement
        .iter()
        .find(|r| r.id == id)
        .unwrap_or_else(|| panic!("{id} is not in the spec"))
}

#[test]
fn every_active_requirement_has_a_mechanism_an_owner_and_a_measure() {
    let spec = real_spec();
    let active: Vec<&Requirement> = spec
        .requirement
        .iter()
        .filter(|r| matches!(r.status, Status::Active))
        .collect();
    assert!(active.len() >= 264, "{} active requirements", active.len());
    let complete = active
        .iter()
        .filter(|r| {
            !r.mechanism.trim().is_empty()
                && !blank_or_none(&r.owner)
                && !blank_or_none(&r.measure)
                && r.measure.contains("status:")
        })
        .count();
    assert_eq!(complete, active.len());
    // The literal `none` is a recorded answer, not a gap in the file: the sweep queues a request for each.
    let none: Vec<&str> = active
        .iter()
        .filter(|r| r.mechanism.trim().eq_ignore_ascii_case("none"))
        .map(|r| r.id.as_str())
        .collect();
    println!(
        "{} requirements record mechanism none: {none:?}",
        none.len()
    );
}

#[test]
fn an_active_requirement_missing_mechanism_owner_or_measure_fails() {
    let brief = real_brief();
    type Damage = fn(&mut Requirement);
    let cases: [(&str, Damage, &str); 7] = [
        ("mechanism", |r| r.mechanism.clear(), "no mechanism"),
        ("owner", |r| r.owner.clear(), "no owner"),
        ("owner none", |r| r.owner = "none".into(), "no owner"),
        ("measure", |r| r.measure.clear(), "no measure"),
        ("measure none", |r| r.measure = "None".into(), "no measure"),
        (
            "measure without a label",
            |r| r.measure = "shown in a test".into(),
            "no measure label",
        ),
        (
            "measure with an empty label",
            |r| r.measure = "shown in a test; status: ".into(),
            "no measure label",
        ),
    ];
    for (name, damage, expected) in cases {
        let mut spec = real_spec();
        assert!(check(&spec, &brief).is_empty());
        damage(&mut spec.requirement[0]);
        let errors = check(&spec, &brief);
        let id = &spec.requirement[0].id;
        assert!(
            errors
                .iter()
                .any(|e| e.starts_with(id) && e.contains(expected)),
            "{name}: {errors:?}"
        );

        // A superseded requirement is exempt.
        spec.requirement[0].status = Status::Superseded;
        assert!(check(&spec, &brief).is_empty(), "{name} when superseded");
    }
    // The literal `none` as a mechanism is a valid answer for the file.
    let mut spec = real_spec();
    spec.requirement[0].mechanism = "none".into();
    assert!(check(&spec, &brief).is_empty());
}

#[test]
fn the_acceleration_requirements_record_a_mechanism_and_an_owner() {
    let spec = real_spec();
    let a = find(&spec, "OLY-A-22");
    assert!(matches!(a.status, Status::Active));
    assert_eq!(a.source[0].heading, "2026-10-13: Athena owns acceleration");
    assert!(
        a.statement.contains("measurably faster") && a.statement.contains("owns its acceleration")
    );
    assert!(a.mechanism.contains("throughput_review"), "{}", a.mechanism);
    assert_eq!(a.owner, "Hestia");
    assert!(a.measure.contains("status: throughput"), "{}", a.measure);

    let s = find(&spec, "OLY-A-23");
    assert_eq!(s.source[0].heading, "2026-10-25: Speed first");
    assert!(!s.mechanism.trim().is_empty() && !blank_or_none(&s.owner));
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
