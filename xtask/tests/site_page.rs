//! Assertions on the rendered firstmate-work-handling page. The page is built by the real command,
//! `xtask site`, and checked against the study data, so a job whose criteria are about what the page
//! contains is checked on the HTML itself. The same helpers run on a committed wrong page, which must fail
//! each check. The convention is in `docs/conventions/rendered-page-assertions.md`.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

const STUDY: &str = "studies/firstmate-work-handling";
const CARD: &str = "<article class=\"card\">";
const LIMIT_TAG: &str = "what this page cannot show";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn json(name: &str) -> Value {
    let text = std::fs::read_to_string(root().join(STUDY).join(name)).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// Builds the page with `xtask site --out <dir>` and returns the HTML.
fn rendered_page() -> String {
    let out = tempfile::tempdir().unwrap();
    let run = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["site", "--out"])
        .arg(out.path())
        .output()
        .unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    std::fs::read_to_string(out.path().join("index.html")).unwrap()
}

fn bad_page() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/bad_firstmate_page.html");
    std::fs::read_to_string(path).unwrap()
}

/// The word-defining entries and the next-step link must come before the first transition card; the
/// full glossary section must exist.
fn check_order(html: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let Some(card) = html.find(CARD) else {
        return vec!["no transition card".into()];
    };
    for (what, marker) in [
        ("the glossary entry block", "<h3>Words used first</h3>"),
        ("the next-step link", "<a href=\"#next-steps\">"),
    ] {
        match html.find(marker) {
            Some(at) if at < card => {}
            Some(_) => problems.push(format!("{what} comes after the first transition card")),
            None => problems.push(format!("{what} is missing")),
        }
    }
    if !html.contains("<section id=\"next-steps\">") {
        problems.push("the next-steps section is missing".into());
    }
    if !html.contains("<h2>Glossary</h2>") {
        problems.push("the glossary section is missing".into());
    }
    problems
}

/// The ids of findings shown with the model-limit label must be exactly the model-limit findings.
fn check_findings(html: &str, findings: &Value) -> Vec<String> {
    let expected: Vec<&str> = findings
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["kind"] == "model-limit")
        .map(|f| f["id"].as_str().unwrap())
        .collect();
    let section = html
        .split("<h2>Limits of this study</h2>")
        .nth(1)
        .and_then(|s| s.split("</section>").next())
        .unwrap_or("");
    let shown: Vec<&str> = section
        .split("<li><strong>")
        .skip(1)
        .filter_map(|item| {
            let (id, rest) = item.split_once("</strong>")?;
            rest.starts_with(&format!(" <span class=\"tag\">{LIMIT_TAG}</span>"))
                .then_some(id)
        })
        .collect();
    let mut problems = Vec::new();
    for id in &expected {
        if !shown.contains(id) {
            problems.push(format!("model-limit finding {id} is not rendered"));
        }
    }
    for id in &shown {
        if !expected.contains(id) {
            problems.push(format!("finding {id} is rendered as a model limit"));
        }
    }
    problems
}

/// Every transition card must say who decides the trigger, as the model's `trigger_evaluated_by`
/// says; at least one must be agent-decided so the check cannot pass on nothing.
fn check_evaluators(html: &str, model: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    let transitions = model["transitions"].as_array().unwrap();
    if !transitions
        .iter()
        .any(|t| t["trigger_evaluated_by"] == "agent")
    {
        problems.push("the model has no agent-decided transition".into());
    }
    for t in transitions {
        let id = t["id"].as_str().unwrap();
        let by = t["trigger_evaluated_by"].as_str().unwrap();
        let head = format!("<h3>{id} <span class=\"arrow\">");
        let Some(at) = html.find(&head) else {
            problems.push(format!("transition {id} has no card"));
            continue;
        };
        let card = html[at..].split("</article>").next().unwrap();
        if !card.contains(&format!("(decided by {by})")) {
            problems.push(format!("transition {id} is not shown as decided by {by}"));
        }
    }
    problems
}

#[test]
fn the_rendered_page_puts_glossary_entries_and_the_next_step_link_before_the_first_card() {
    assert_eq!(check_order(&rendered_page()), Vec::<String>::new());
}

#[test]
fn the_rendered_page_shows_exactly_the_model_limit_findings() {
    let findings = json("findings.json");
    assert!(
        findings
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["kind"] == "model-limit"),
        "the study has no model-limit finding to check"
    );
    assert_eq!(
        check_findings(&rendered_page(), &findings),
        Vec::<String>::new()
    );
}

#[test]
fn the_rendered_page_shows_agent_decided_transitions_as_the_model_says() {
    assert_eq!(
        check_evaluators(&rendered_page(), &json("model.json")),
        Vec::<String>::new()
    );
}

#[test]
fn the_wrong_fixture_page_fails_every_check() {
    let html = bad_page();
    let order = check_order(&html);
    assert!(
        order
            .iter()
            .any(|p| p == "the glossary entry block comes after the first transition card"),
        "{order:?}"
    );
    assert!(
        order.iter().any(|p| p == "the next-step link is missing"),
        "{order:?}"
    );
    let findings = check_findings(&html, &json("findings.json"));
    assert!(
        findings.iter().any(|p| p.contains("is not rendered")),
        "{findings:?}"
    );
    assert!(
        findings
            .iter()
            .any(|p| p.contains("rendered as a model limit")),
        "{findings:?}"
    );
    let evaluators = check_evaluators(&html, &json("model.json"));
    assert!(
        evaluators
            .iter()
            .any(|p| p == "transition t1 is not shown as decided by agent"),
        "{evaluators:?}"
    );
}
