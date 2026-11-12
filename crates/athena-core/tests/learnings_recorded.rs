//! OLY-B-A3-02: what Athena learns from firstmate and Hermes is recorded with rationale, and where the
//! harness-engineering bundle shaped or did not shape a decision that is recorded and attributed under its license.
//!
//! The mechanism is this test. It checks three things on the real files:
//! - every card in `studies/firstmate-work-handling/rationales.json` has a decision, a tradeoff, a reversal
//!   cost, an alternative, a valid provenance and at least one quoted source, on the card or on a claim it cites;
//! - `README.md` names the bundle, its repository URL and the CC BY 4.0 license;
//! - section 15 of `docs/design/athena-v0.md` carries the attribution and a "Departure, and why" table with rows.
//!
//! Limits: Hermes has no study records yet, so what Athena learns from Hermes is not covered. Whether each row of
//! the section 15 table is true is a verifier judgement and is not checked. The rationale checker here is a
//! snapshot of the rules in `crates/athena-cap-study/src/rules.rs` (stated cards without a source, incomplete
//! cards) with no link to them; `crates/athena-cli/tests/drain.rs` (`drain::study_text` on
//! `firstmate-work-handling`) is the path that runs the real rules on this study.

use athena_core::hestia::{SpecRequirement, measure_problem, parse_spec, spec_gaps};
use serde_json::Value;
use std::path::Path;

const TEST_COMMAND: &str = "cargo test -p athena-core --test learnings_recorded";
const REQUIREMENT: &str = "OLY-B-A3-02";
const BUNDLE_URL: &str = "https://github.com/lopopolo/harness-engineering";

fn repo_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn blank(v: Option<&Value>) -> bool {
    v.and_then(Value::as_str)
        .is_none_or(|s| s.trim().is_empty())
}

fn quotes(sources: Option<&Value>) -> usize {
    sources
        .and_then(Value::as_array)
        .map(|a| a.iter().filter(|s| !blank(s.get("quote"))).count())
        .unwrap_or(0)
}

/// Problems with the rationale cards; empty when every card is complete and reaches a quoted source.
fn rationale_problems(rationales: &str, claims: &str) -> Vec<String> {
    let mut out = Vec::new();
    let cards: Value = match serde_json::from_str(rationales) {
        Ok(v) => v,
        Err(e) => return vec![format!("rationales do not parse: {e}")],
    };
    let claim_list: Value = match serde_json::from_str(claims) {
        Ok(v) => v,
        Err(e) => return vec![format!("claims do not parse: {e}")],
    };
    let Some(cards) = cards.as_array().filter(|a| !a.is_empty()) else {
        return vec!["no rationale cards".into()];
    };
    let claim_of = |id: &str| {
        claim_list.as_array().and_then(|a| {
            a.iter()
                .find(|c| c.get("id").and_then(Value::as_str) == Some(id))
        })
    };
    for card in cards {
        let id = card.get("id").and_then(Value::as_str).unwrap_or("?");
        for field in ["decision", "tradeoff", "reversal_cost"] {
            if blank(card.get(field)) {
                out.push(format!("{id}: blank {field}"));
            }
        }
        let has_alternative = card
            .get("alternatives")
            .and_then(Value::as_array)
            .is_some_and(|a| a.iter().any(|x| !blank(Some(x))));
        if !has_alternative {
            out.push(format!("{id}: no alternative"));
        }
        let provenance = card.get("provenance").and_then(Value::as_str).unwrap_or("");
        if !["stated", "evidenced", "inferred"].contains(&provenance) {
            out.push(format!(
                "{id}: provenance `{provenance}` is not stated, evidenced or inferred"
            ));
        }
        let mut reached = quotes(card.get("sources"));
        for cited in card
            .get("claims")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            match claim_of(cited) {
                Some(c) => reached += quotes(c.get("sources")),
                None => out.push(format!(
                    "{id}: cites claim {cited}, which is not in claims.json"
                )),
            }
        }
        if reached == 0 {
            if provenance == "stated" {
                out.push(format!("{id}: stated card with no quoted source"));
            } else {
                out.push(format!("{id}: reaches no quoted source"));
            }
        }
    }
    out
}

fn readme_attribution_problems(readme: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (what, text) in [
        ("the bundle name Harness Engineering", "Harness Engineering"),
        ("the bundle URL", BUNDLE_URL),
        ("the license CC BY 4.0", "CC BY 4.0"),
    ] {
        if !readme.contains(text) {
            out.push(format!("README.md does not name {what}"));
        }
    }
    out
}

fn design_attribution_problems(design: &str) -> Vec<String> {
    let mut out = Vec::new();
    let Some(start) = design.find("## 15. The harness-engineering bundle") else {
        return vec!["design has no section 15 heading".into()];
    };
    let rest = &design[start..];
    let section = &rest[3..];
    let section = &rest[..section.find("\n## ").map_or(rest.len(), |i| i + 3)];
    if !section.contains("CC BY 4.0") {
        out.push("section 15 does not name the license CC BY 4.0".into());
    }
    if !section.contains(BUNDLE_URL) {
        out.push("section 15 does not name the bundle URL".into());
    }
    match section.find("Departure, and why") {
        None => out.push("section 15 has no `Departure, and why` table".into()),
        Some(i) => {
            let rows = section[i..]
                .lines()
                .skip(2)
                .take_while(|l| l.starts_with('|'))
                .count();
            if rows == 0 {
                out.push("the `Departure, and why` table has no rows".into());
            }
        }
    }
    out
}

fn owner_problem(r: &SpecRequirement) -> Option<String> {
    (r.owner != "Athena").then(|| format!("the owner is `{}`, not Athena", r.owner))
}

fn measure_label_problem(r: &SpecRequirement) -> Option<String> {
    if !r.measure.trim_end().ends_with("status: not shown") {
        return Some("the measure does not end with `status: not shown`".into());
    }
    (!r.measure.contains(TEST_COMMAND))
        .then(|| format!("the measure does not name `{TEST_COMMAND}`"))
}

fn real_requirement() -> SpecRequirement {
    parse_spec(&repo_file("docs/spec.toml"))
        .expect("docs/spec.toml parses")
        .into_iter()
        .find(|r| r.id == REQUIREMENT)
        .expect("docs/spec.toml has OLY-B-A3-02")
}

fn rationales() -> String {
    repo_file("studies/firstmate-work-handling/rationales.json")
}

fn claims() -> String {
    repo_file("studies/firstmate-work-handling/claims.json")
}

#[test]
fn the_firstmate_rationale_records_carry_their_rationale() {
    let text = rationales();
    let cards: Value = serde_json::from_str(&text).expect("rationales parse");
    assert_eq!(cards.as_array().map(Vec::len), Some(6));
    // ra4 has no sources of its own; it reaches quotes through the claims c7 and c8.
    assert_eq!(rationale_problems(&text, &claims()), Vec::<String>::new());
}

#[test]
fn the_readme_and_design_attribute_the_harness_engineering_bundle() {
    assert_eq!(
        readme_attribution_problems(&repo_file("README.md")),
        Vec::<String>::new()
    );
    assert_eq!(
        design_attribution_problems(&repo_file("docs/design/athena-v0.md")),
        Vec::<String>::new()
    );
}

fn mutate(real: &str, from: &str, to: &str) -> String {
    assert!(real.contains(from), "target text missing: {from}");
    real.replacen(from, to, 1)
}

#[test]
fn an_emptied_tradeoff_is_reported() {
    let (r, c) = (rationales(), claims());
    assert!(rationale_problems(&r, &c).is_empty());
    let from = "\"tradeoff\": \"Spawn and teardown refuse rather than report success when the backlog transition cannot land, so a backlog fault can stop a dispatch or a cleanup.\"";
    let bad = mutate(&r, from, "\"tradeoff\": \"\"");
    let problems = rationale_problems(&bad, &c);
    assert_eq!(problems, vec!["ra1: blank tradeoff"]);
}

#[test]
fn an_emptied_reversal_cost_is_reported() {
    let (r, c) = (rationales(), claims());
    let from = "\"reversal_cost\": \"High in effect: spawn, teardown and bootstrap all rely on the pairing (inferred from the cited header, not stated by the authors).\"";
    let bad = mutate(&r, from, "\"reversal_cost\": \" \"");
    assert_eq!(
        rationale_problems(&bad, &c),
        vec!["ra1: blank reversal_cost"]
    );
}

#[test]
fn a_stated_card_without_a_quoted_source_is_reported() {
    let (r, c) = (rationales(), claims());
    let bad = mutate(
        &r,
        "\"quote\": \"Nothing else - not a later agent turn, not a printed reminder - is\"",
        "\"quote\": \"\"",
    );
    // The cited claims still carry quotes, so the card is still reached through them.
    assert!(rationale_problems(&bad, &c).is_empty());
    let bad = mutate(&bad, "\"claims\": [\"c1\", \"c13\"]", "\"claims\": []");
    assert_eq!(
        rationale_problems(&bad, &c),
        vec!["ra1: stated card with no quoted source"]
    );
}

#[test]
fn a_card_that_reaches_no_quote_is_reported() {
    let (r, c) = (rationales(), claims());
    let bad = mutate(&r, "\"claims\": [\"c7\", \"c8\"]", "\"claims\": []");
    assert_eq!(
        rationale_problems(&bad, &c),
        vec!["ra4: reaches no quoted source"]
    );
}

#[test]
fn a_cited_claim_that_does_not_exist_is_reported() {
    let (r, c) = (rationales(), claims());
    let bad = mutate(
        &r,
        "\"claims\": [\"c7\", \"c8\"]",
        "\"claims\": [\"c7\", \"c99\"]",
    );
    assert_eq!(
        rationale_problems(&bad, &c),
        vec!["ra4: cites claim c99, which is not in claims.json"]
    );
}

#[test]
fn a_readme_without_the_license_is_reported() {
    let readme = repo_file("README.md");
    assert!(readme_attribution_problems(&readme).is_empty());
    let bad = mutate(&readme, "CC BY 4.0", "the license");
    let problems = readme_attribution_problems(&bad);
    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("CC BY 4.0"), "{problems:?}");
}

#[test]
fn a_design_without_the_departure_table_is_reported() {
    let design = repo_file("docs/design/athena-v0.md");
    assert!(design_attribution_problems(&design).is_empty());
    let bad = mutate(&design, "Departure, and why", "Notes");
    let problems = design_attribution_problems(&bad);
    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("Departure, and why"), "{problems:?}");
}

#[test]
fn spec_entry_has_a_real_mechanism() {
    let text = repo_file("docs/spec.toml");
    let requirements = parse_spec(&text).expect("docs/spec.toml parses");
    let r = real_requirement();
    assert_ne!(r.mechanism, "none");
    assert!(
        r.mechanism
            .contains("crates/athena-core/tests/learnings_recorded.rs"),
        "{}",
        r.mechanism
    );
    assert!(r.mechanism.contains("Hermes"), "{}", r.mechanism);
    assert_eq!(owner_problem(&r), None);
    assert_eq!(measure_problem(&r.measure), None, "{}", r.measure);
    assert_eq!(measure_label_problem(&r), None, "{}", r.measure);
    assert!(
        !spec_gaps(&requirements).iter().any(|g| g.id == REQUIREMENT),
        "OLY-B-A3-02 is still a spec gap"
    );
}

#[test]
fn spec_check_refuses_a_none_mechanism() {
    let mut r = real_requirement();
    r.mechanism = "none".into();
    let gaps = spec_gaps(&[r]);
    assert_eq!(gaps.len(), 1);
    assert!(gaps[0].no_mechanism);
}

#[test]
fn spec_check_refuses_a_wrong_status_label() {
    let mut r = real_requirement();
    r.measure = format!("The spec gap list; {TEST_COMMAND}; status: requirement");
    assert!(measure_label_problem(&r).is_some());
}

#[test]
fn spec_check_refuses_a_wrong_owner() {
    let mut r = real_requirement();
    r.owner = "Hestia".into();
    assert!(owner_problem(&r).is_some());
}
