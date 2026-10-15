//! The audience against the fake agent over a stubbed page: what it is given, what it records, and what the
//! steward and `athena status` then show. No test reaches the network.

// Scenarios are set up field by field, so each test shows only what differs from the default.
#![allow(clippy::field_reassign_with_default)]

mod common;

use athena_core::Athena;
use athena_core::audience::{self, Page, Rated, Target};
use athena_core::steward::{self, GitView, RequestQueue};
use athena_core::store::Records;
use athena_data::records::{Role, WorkItemKind};
use athena_data::roles;
use common::*;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

const STUB_PAGE: &str = "<h1>Start here</h1><p>PAGE-CANARY firstmate hands work to agents.</p>";

/// A page that is a string, and counts its fetches.
struct Stub(Mutex<Vec<String>>);

impl Page for Stub {
    fn fetch(&self, url: &str) -> anyhow::Result<String> {
        self.0.lock().unwrap().push(url.to_string());
        Ok(STUB_PAGE.into())
    }
}

fn stub() -> Stub {
    Stub(Mutex::new(Vec::new()))
}

fn records(fx: &Fixture) -> Records {
    Records::load(&fx.git, &fx.head()).unwrap()
}

fn report(rating: u32) -> Value {
    json!({
        "rating": rating,
        "findings": [{
            "text": "The page does not say what to do next.",
            "source": "heading: Start here"
        }]
    })
}

fn rate(fx: &Fixture, audience: Value) -> (Athena, anyhow::Result<Option<Rated>>) {
    let mut scenario = Scenario::default();
    scenario.audience = audience;
    let athena = fx.athena(scenario.script());
    let result = athena.audience(&stub(), &Target::explainer());
    (athena, result)
}

#[test]
fn a_run_over_a_stubbed_page_records_a_rating_and_the_steward_sees_it() {
    let fx = Fixture::new();
    let page = stub();
    let mut scenario = Scenario::default();
    scenario.audience = report(4);
    let seen = scenario.seen_by_audience.clone();
    let invocations = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let rated = athena
        .audience(&page, &Target::explainer())
        .unwrap()
        .expect("a rating");

    // The default target is fetched once, and the model is given the page and nothing private.
    assert_eq!(*page.0.lock().unwrap(), [audience::EXPLAINER_URL]);
    assert_eq!(*invocations.lock().unwrap(), [Role::Audience]);
    let seen = seen.lock().unwrap().join("\n");
    assert!(seen.contains("PAGE-CANARY"), "the page is in the workspace");
    assert!(seen.contains(roles::AUDIENCE_PROMPT));
    assert!(seen.contains("newcomer"));
    assert!(!seen.contains("PRIVATE-CANARY"));

    // One records commit holds the role, the rating and the invocation; a rating of 4 raises nothing.
    assert_eq!(rated.rating.rating, 4);
    assert!(rated.work_item.is_none());
    let subject = fx.git.run(["log", "--format=%s", "-n", "1"]).unwrap();
    assert!(
        subject.starts_with("Audience: newcomer rated 4/5"),
        "{subject}"
    );
    let after = records(&fx);
    assert_eq!(after.roles.len(), 1);
    assert_eq!(after.roles[0].path(), "records/roles/audience-1.json");
    assert_eq!(after.audience_ratings, vec![rated.rating.clone()]);
    assert_eq!(after.audience_runs.len(), 1);
    assert_eq!(after.audience_runs[0].role, Role::Audience);
    assert!(after.work_items.is_empty());
    assert_eq!(rated.rating.target, audience::EXPLAINER_URL);
    assert_eq!(rated.rating.findings[0].source, "heading: Start here");
    assert!(fx.git.run(["status", "--porcelain"]).unwrap().is_empty());

    // The steward's brief carries the latest rating and its findings.
    let prompts: Arc<Mutex<Vec<String>>> = Arc::default();
    let mut scenario = Scenario::default();
    let sink = prompts.clone();
    scenario.steward_reads = Some(Arc::new(move |spec| {
        sink.lock().unwrap().push(spec.user_prompt.clone());
        Vec::new()
    }));
    let steward_athena = fx.athena(scenario.script());
    let role = roles::steward(athena_core::now());
    let view = GitView {
        git: &steward_athena.git,
    };
    steward::run(
        &steward_athena,
        &view,
        &RequestQueue,
        &role,
        athena_core::now(),
    )
    .unwrap();
    let prompts = prompts.lock().unwrap();
    assert_eq!(prompts.len(), 1);
    assert!(prompts[0].contains("## Audience rating"));
    assert!(prompts[0].contains("4/5 from a newcomer"));
    assert!(prompts[0].contains("The page does not say what to do next."));
    assert!(prompts[0].contains("evidence: heading: Start here"));

    // So does `athena status`.
    let status = athena.status().unwrap();
    let shown = status.audience.expect("the latest rating");
    assert_eq!(shown.rating, 4);
    assert_eq!(shown.target, audience::EXPLAINER_URL);
    assert_eq!(
        shown.findings,
        ["The page does not say what to do next. (evidence: heading: Start here)"]
    );
    assert!(status.work_proposals.is_empty());
}

#[test]
fn a_rating_of_two_or_lower_raises_a_work_item_and_three_does_not() {
    for (rating, raised) in [(1, true), (2, true), (3, false), (5, false)] {
        let fx = Fixture::new();
        let (athena, result) = rate(&fx, report(rating));
        let rated = result.unwrap().unwrap();
        assert_eq!(rated.work_item.is_some(), raised, "{rating}");
        let after = records(&fx);
        assert_eq!(after.work_items.len(), usize::from(raised), "{rating}");
        let status = athena.status().unwrap();
        assert_eq!(status.work_proposals.len(), usize::from(raised), "{rating}");
        if raised {
            let item = &after.work_items[0];
            assert_eq!(item.kind, WorkItemKind::Proposal);
            assert_eq!(item.role, "audience");
            assert_eq!(item.key, audience::work_item_key(&rated.rating));
            let path = format!(
                "records/audience-ratings/{}/{}.json",
                rated.rating.id.month(),
                rated.rating.id
            );
            assert_eq!(item.evidence[0].value, path);
            assert!(
                status.work_proposals[0]
                    .title
                    .contains(&format!("{rating}/5"))
            );
            assert!(
                status
                    .next
                    .iter()
                    .any(|n| n.starts_with(&format!("athena approve-work {}", item.id))),
                "{:?}",
                status.next
            );
        }
    }
}

#[test]
fn the_latest_rating_is_the_one_shown() {
    let fx = Fixture::new();
    let (_, first) = rate(&fx, report(2));
    first.unwrap().unwrap();
    let (athena, second) = rate(&fx, report(5));
    let second = second.unwrap().unwrap();
    assert_eq!(records(&fx).audience_ratings.len(), 2);
    assert_eq!(
        athena.status().unwrap().audience.unwrap().id,
        second.rating.id.to_string()
    );
}

#[test]
fn an_out_of_range_rating_or_a_finding_without_evidence_records_no_rating() {
    let bad = [
        report(0),
        report(6),
        json!({ "rating": 3, "findings": [{ "text": "Unclear.", "source": " " }] }),
    ];
    for audience in bad {
        let fx = Fixture::new();
        let (_, result) = rate(&fx, audience.clone());
        assert!(result.is_err(), "{audience}");
        let after = records(&fx);
        assert!(after.audience_ratings.is_empty(), "{audience}");
        assert!(after.work_items.is_empty(), "{audience}");
        assert_eq!(after.audience_runs.len(), 1, "the paid invocation is kept");
    }
}

#[test]
fn a_nested_run_fetches_nothing_and_records_nothing() {
    let fx = Fixture::new();
    let scenario = Scenario::default();
    let invocations = scenario.invocations.clone();
    let mut athena = fx.athena(scenario.script());
    athena.nested = Some("20261014-120000-outr".into());
    let page = stub();
    let head = fx.head();
    assert_eq!(athena.audience(&page, &Target::explainer()).unwrap(), None);
    assert!(page.0.lock().unwrap().is_empty());
    assert!(invocations.lock().unwrap().is_empty());
    assert_eq!(fx.head(), head);
}

#[test]
fn a_page_that_cannot_be_fetched_calls_no_model() {
    struct Down;
    impl Page for Down {
        fn fetch(&self, _: &str) -> anyhow::Result<String> {
            anyhow::bail!("connection refused")
        }
    }
    let fx = Fixture::new();
    let scenario = Scenario::default();
    let invocations = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let head = fx.head();
    assert!(athena.audience(&Down, &Target::explainer()).is_err());
    assert!(invocations.lock().unwrap().is_empty());
    assert_eq!(fx.head(), head);
}

#[test]
fn another_target_is_rated_by_url_and_reader() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let target = Target {
        url: "https://example.invalid/guide/".into(),
        audience: "maintainer".into(),
    };
    let rated = athena.audience(&stub(), &target).unwrap().unwrap();
    assert_eq!(rated.rating.target, "https://example.invalid/guide/");
    assert_eq!(rated.rating.audience, "maintainer");
}
