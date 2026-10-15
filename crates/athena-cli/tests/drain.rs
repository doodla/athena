//! `athena drain`: the loop against a stubbed queue and a stubbed land step (every stop reason, and that a
//! rating is only taken after a landing), and once against the fake agent end to end. No test reaches the
//! network.

// Scenarios are set up field by field, so each test shows only what differs from the default.
#![allow(clippy::field_reassign_with_default)]

mod common;

use athena_cli::display;
use athena_cli::drain::{
    self, Caps, Item, Lander, Landing, Limits, QueueView, Rate, Read, ReadOf, RealSteps, Run,
    Steps, Stop, Waiting,
};
use athena_core::audience::Page;
use athena_core::commits::{Committer, message};
use athena_core::steward::{ProposedChange, RequestQueue};
use athena_core::store::Records;
use athena_core::{Athena, Quiet};
use athena_data::records::{OutcomeState, Reference, WorkItemKind};
use athena_data::roles::Ladder;
use common::*;
use serde_json::json;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::process::Command;

/// A stub world: a scripted queue, outcomes, landings and ratings, and a log of every call in order.
/// Each read costs 0.1, each run 0.5 and each rating 0.2; their caps are 0.2, 1.0 and 0.3.
#[derive(Default)]
struct Stub {
    items: Vec<&'static str>,
    waiting: Vec<Waiting>,
    previous: Option<u8>,
    /// Outcomes of runs, in order; a run after these are used up is rejected.
    outcomes: RefCell<Vec<OutcomeState>>,
    /// Jobs whose landing fails.
    unlandable: Vec<&'static str>,
    /// Ratings, in order.
    ratings: RefCell<Vec<u8>>,
    /// What a run costs; 0.5 when not set.
    run_cost: Option<f64>,
    log: RefCell<Vec<String>>,
}

impl Stub {
    fn log(&self) -> Vec<String> {
        self.log.borrow().clone()
    }
}

impl Steps for Stub {
    fn caps(&self) -> anyhow::Result<Caps> {
        Ok(Caps {
            read: 0.2,
            run: 1.0,
            rate: 0.3,
        })
    }
    fn queue(&self) -> anyhow::Result<QueueView> {
        Ok(QueueView {
            items: self
                .items
                .iter()
                .map(|j| Item {
                    job: j.to_string(),
                    title: format!("title of {j}"),
                })
                .collect(),
            waiting: self.waiting.clone(),
        })
    }
    fn previous_rating(&self) -> Option<u8> {
        self.previous
    }
    fn read(&self, what: ReadOf<'_>) -> Read {
        let (entry, job) = match what {
            ReadOf::Queued(j) => (format!("read {j}"), j.to_string()),
            ReadOf::Retry(j) => (format!("reread {j}"), format!("{j}-retry")),
        };
        self.log.borrow_mut().push(entry);
        Read {
            job: Some(job),
            cost: 0.1,
            runnable: true,
            error: None,
        }
    }
    fn run(&self, job: &str) -> Run {
        self.log.borrow_mut().push(format!("run {job}"));
        let mut outcomes = self.outcomes.borrow_mut();
        let outcome = if outcomes.is_empty() {
            OutcomeState::Rejected
        } else {
            outcomes.remove(0)
        };
        Run {
            outcome: Some(outcome),
            reason: format!("ended {}", outcome.as_str()),
            cost: self.run_cost.unwrap_or(0.5),
        }
    }
    fn land(&self, job: &str) -> Landing {
        self.log.borrow_mut().push(format!("land {job}"));
        if self.unlandable.contains(&job) {
            Landing::NotLanded("checks failed".into())
        } else {
            Landing::Landed
        }
    }
    fn rate(&self) -> Rate {
        self.log.borrow_mut().push("rate".into());
        let mut ratings = self.ratings.borrow_mut();
        Rate {
            rating: (!ratings.is_empty()).then(|| ratings.remove(0)),
            cost: 0.2,
            error: None,
        }
    }
}

fn passing() -> RefCell<Vec<OutcomeState>> {
    RefCell::new(vec![OutcomeState::Landed; 8])
}

fn limits(budget: f64) -> Limits {
    Limits {
        budget_usd: budget,
        target_rating: 4,
        max_jobs: 10,
    }
}

fn waiting_proposal() -> Waiting {
    Waiting {
        id: "20261015-120000-wait".into(),
        title: "The brief should say the README is never deleted".into(),
        rung: "brief-requirements".into(),
    }
}

#[test]
fn two_items_are_landed_and_then_a_rating_of_four_meets_the_target() {
    let stub = Stub {
        items: vec!["a", "b"],
        outcomes: passing(),
        ratings: RefCell::new(vec![3, 4]),
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();

    // Each item is read, run and landed, and the page is rated after that landing and not before.
    assert_eq!(
        stub.log(),
        [
            "read a", "run a", "land a", "rate", "read b", "run b", "land b", "rate"
        ]
    );
    assert_eq!(report.stop, Stop::TargetMet);
    assert!(report.target_met());
    assert_eq!(report.landed, 2);
    assert!(!report.page_unchanged);
    let rating = report.rating.unwrap();
    assert_eq!((rating.value, rating.after_landing), (4, true));
    assert!(report.queued.is_empty());
    assert_eq!(report.jobs.len(), 2);
    assert!(report.jobs.iter().all(|j| j.landing == "landed"));
    // 2 x (0.1 read + 0.5 run + 0.2 rating)
    assert!(
        (report.spent_usd - 1.6).abs() < 1e-9,
        "{}",
        report.spent_usd
    );
    let text = display::drain(&report);
    assert!(text.contains("target met"), "{text}");
    assert!(text.contains("taken after a landing"), "{text}");
}

#[test]
fn a_rating_is_never_taken_before_a_landing() {
    let stub = Stub {
        items: vec!["a", "b", "c"],
        outcomes: passing(),
        ratings: RefCell::new(vec![1, 2, 3]),
        unlandable: vec!["b"],
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(9.0), &Quiet).unwrap();
    let log = stub.log();
    for (n, entry) in log.iter().enumerate() {
        if entry == "rate" {
            assert!(
                log[..n].iter().any(|e| e.starts_with("land ")),
                "a rating at {n} came before any landing: {log:?}"
            );
        }
    }
    // The first rating follows the landing of a, and b's failed landing is followed by no rating.
    assert_eq!(&log[..4], ["read a", "run a", "land a", "rate"]);
    assert_eq!(report.stop, Stop::RepeatedFailure);
    assert_eq!(report.failed_item.as_deref(), Some("b"));
}

#[test]
fn a_pass_that_landed_nothing_cannot_report_target_met_from_a_stale_rating() {
    // A rating of 5 was recorded before this run. Item a fails twice, so nothing lands.
    let stub = Stub {
        items: vec!["a"],
        previous: Some(5),
        ratings: RefCell::new(vec![5]),
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();
    assert!(
        !stub.log().contains(&"rate".to_string()),
        "{:?}",
        stub.log()
    );
    assert_ne!(report.stop, Stop::TargetMet);
    assert!(!report.target_met());
    assert_eq!(report.landed, 0);
    assert!(report.page_unchanged);
    let rating = report.rating.unwrap();
    assert_eq!(rating.value, 5, "the stale rating is shown");
    assert!(!rating.after_landing && !rating.met_target);
    let text = display::drain(&report);
    assert!(text.contains("unchanged"), "{text}");
    assert!(text.contains("not taken after a landing"), "{text}");
    assert!(!text.contains("target met"), "{text}");
    let value = serde_json::to_value(&report).unwrap();
    assert_eq!(value["page_unchanged"], true);
    assert_eq!(value["rating"]["after_landing"], false);
}

#[test]
fn a_budget_too_small_for_the_next_call_stops_with_the_rest_still_queued() {
    // Item a costs 0.8 with its rating. Item b needs room for a reading and a run (0.2 + 1.0) on top of that.
    let stub = Stub {
        items: vec!["a", "b", "c"],
        outcomes: passing(),
        ratings: RefCell::new(vec![3, 3, 3]),
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(1.9), &Quiet).unwrap();
    assert_eq!(report.stop, Stop::BudgetSpent);
    assert_eq!(stub.log(), ["read a", "run a", "land a", "rate"]);
    let queued: Vec<&str> = report.queued.iter().map(|i| i.job.as_str()).collect();
    assert_eq!(queued, ["b", "c"], "the rest stays queued");
    assert!(!report.target_met());
    assert!(display::drain(&report).contains("budget spent"));
}

#[test]
fn a_rating_that_does_not_fit_the_budget_is_not_taken() {
    // With a budget of 1.3 the job fits to start (0.2 + 1.0 of caps). The run then costs 1.0, not the 0.5 the
    // other tests use, so 0.1 + 1.0 is spent and the rating's cap of 0.3 no longer fits.
    let stub = Stub {
        items: vec!["a"],
        outcomes: passing(),
        ratings: RefCell::new(vec![5]),
        run_cost: Some(1.0),
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(1.3), &Quiet).unwrap();
    assert_eq!(stub.log(), ["read a", "run a", "land a"], "no rating call");
    assert_eq!(report.stop, Stop::BudgetSpent);
    assert_eq!(report.landed, 1);
    assert!(!report.target_met());
    let rating = report.rating;
    assert!(
        rating.is_none(),
        "no rating, so none can be called a result"
    );
    // With room for the rating, the same run meets the target.
    let stub = Stub {
        items: vec!["a"],
        outcomes: passing(),
        ratings: RefCell::new(vec![5]),
        run_cost: Some(1.0),
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(1.4), &Quiet).unwrap();
    assert_eq!(report.stop, Stop::TargetMet);
}

#[test]
fn an_empty_queue_stops_at_once_with_no_spend_and_is_not_success() {
    let stub = Stub {
        waiting: vec![waiting_proposal()],
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();
    assert!(
        stub.log().is_empty(),
        "no call of any kind: {:?}",
        stub.log()
    );
    assert_eq!(report.stop, Stop::QueueEmpty);
    assert_eq!(report.spent_usd, 0.0);
    assert!(!report.target_met());
    assert!(report.jobs.is_empty());
    let text = display::drain(&report);
    assert!(text.contains("queue empty"), "{text}");
    assert!(text.contains("not success"), "{text}");
}

#[test]
fn the_same_item_failing_twice_is_a_repeated_failure_naming_the_item() {
    let stub = Stub {
        items: vec!["a", "b"],
        ratings: RefCell::new(vec![5]),
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();
    // The second attempt re-reads the failed job (a retry) and runs the new job.
    assert_eq!(
        stub.log(),
        ["read a", "run a", "reread a", "run a-retry"],
        "the loop stops there; b is not started"
    );
    assert_eq!(report.stop, Stop::RepeatedFailure);
    assert_eq!(report.failed_item.as_deref(), Some("a"));
    assert_eq!(report.jobs.len(), 2);
    assert_eq!(report.queued.len(), 2);
    let text = display::drain(&report);
    assert!(
        text.contains("repeated failure: item a failed twice"),
        "{text}"
    );
    assert!(text.contains("rejected"), "{text}");
}

#[test]
fn a_landing_failure_is_a_failure_and_no_rating_is_taken_as_success() {
    let stub = Stub {
        items: vec!["a"],
        outcomes: passing(),
        unlandable: vec!["a"],
        ratings: RefCell::new(vec![5]),
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();
    // The job passed, so the second attempt lands again; it does not run the job again.
    assert_eq!(stub.log(), ["read a", "run a", "land a", "land a"]);
    assert_eq!(report.stop, Stop::RepeatedFailure);
    assert_eq!(report.failed_item.as_deref(), Some("a"));
    assert_eq!(report.landed, 0);
    assert!(report.rating.is_none());
    assert!(!report.target_met());
    assert!(report.page_unchanged);
    assert_eq!(report.jobs[0].landing, "not landed: checks failed");
    let text = display::drain(&report);
    assert!(text.contains("not landed: checks failed"), "{text}");
    assert!(text.contains("unchanged"), "{text}");
}

#[test]
fn a_proposal_that_needs_a_person_is_listed_waiting_and_left_alone() {
    let stub = Stub {
        items: vec!["a"],
        waiting: vec![waiting_proposal()],
        outcomes: passing(),
        ratings: RefCell::new(vec![4]),
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();
    // Nothing the loop did names the proposal: the steps it calls have no way to accept, approve or land one.
    assert!(
        stub.log().iter().all(|e| !e.contains("wait")),
        "{:?}",
        stub.log()
    );
    assert_eq!(report.waiting, vec![waiting_proposal()]);
    let text = display::drain(&report);
    assert!(text.contains("waiting     20261015-120000-wait"), "{text}");
    assert!(text.contains("needs a person"), "{text}");
    let value = serde_json::to_value(&report).unwrap();
    assert_eq!(value["waiting"][0]["rung"], "brief-requirements");
}

#[test]
fn the_job_limit_stops_the_loop() {
    let stub = Stub {
        items: vec!["a", "b"],
        outcomes: passing(),
        ratings: RefCell::new(vec![3, 3]),
        ..Stub::default()
    };
    let report = drain::drain(
        &stub,
        Limits {
            max_jobs: 1,
            ..limits(9.0)
        },
        &Quiet,
    )
    .unwrap();
    assert_eq!(report.stop, Stop::JobLimit);
    assert_eq!(report.jobs.len(), 1);
    assert_eq!(report.queued.len(), 1);
}

#[test]
fn the_json_report_names_the_stop_the_money_the_rating_and_what_is_left() {
    let stub = Stub {
        items: vec!["a", "b"],
        waiting: vec![waiting_proposal()],
        outcomes: passing(),
        ratings: RefCell::new(vec![5]),
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();
    let value = serde_json::to_value(&report).unwrap();
    assert_eq!(value["stop"], "target-met");
    assert_eq!(value["failed_item"], json!(null));
    assert_eq!(value["jobs"][0]["outcome"], "passed");
    assert_eq!(value["jobs"][0]["landing"], "landed");
    assert!(value["jobs"][0]["cost_usd"].as_f64().unwrap() > 0.0);
    assert!(value["spent_usd"].as_f64().unwrap() > 0.0);
    assert_eq!(value["budget_usd"], 5.0);
    assert_eq!(value["target_rating"], 4);
    assert_eq!(value["rating"]["value"], 5);
    assert_eq!(value["rating"]["after_landing"], true);
    assert_eq!(value["queued"][0]["job"], "b");
    assert_eq!(value["waiting"][0]["id"], "20261015-120000-wait");
    let text = display::drain(&report);
    for needle in ["stopped", "job ", "spent", "rating", "queued", "waiting"] {
        assert!(text.contains(needle), "{needle}: {text}");
    }
}

#[test]
fn drain_help_documents_the_flags_and_lists_every_stop_reason() {
    let out = Command::new(env!("CARGO_BIN_EXE_athena"))
        .args(["drain", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let help = String::from_utf8_lossy(&out.stdout);
    for flag in ["--budget", "--rating-threshold", "--job-limit", "--json"] {
        assert!(help.contains(flag), "{flag}: {help}");
    }
    for stop in Stop::ALL {
        assert!(help.contains(stop.label()), "{}: {help}", stop.label());
    }
    // The budget is required.
    let out = Command::new(env!("CARGO_BIN_EXE_athena"))
        .arg("drain")
        .output()
        .unwrap();
    assert!(!out.status.success());
}

// ---------------------------------------------------------------------------------------------------------
// Against the fake agent

struct StubPage;

impl Page for StubPage {
    fn fetch(&self, _: &str) -> anyhow::Result<String> {
        Ok("<h1>Start here</h1><p>firstmate hands work to agents.</p>".into())
    }
}

/// A land step that records the job and how many ratings existed when it was asked.
#[derive(Default)]
struct RecordingLander(std::sync::Mutex<Vec<(String, usize)>>);

impl Lander for RecordingLander {
    fn land(&self, athena: &Athena, job: &str, _: &dyn athena_core::Ui) -> Landing {
        let ratings = Records::load(&athena.git, &athena.head().unwrap())
            .unwrap()
            .audience_ratings
            .len();
        self.0.lock().unwrap().push((job.to_string(), ratings));
        Landing::Landed
    }
}

/// Commits a request on the steward's channel, as the steward's queue does, and returns its job id.
fn queue_steward_request(athena: &Athena, text: &str) -> String {
    let change = ProposedChange {
        kind: WorkItemKind::RepeatedFailure,
        key: "repeated-failure:docs".into(),
        title: "A repeated failure".into(),
        rung: Ladder::Implementation,
        source: Reference::path("records/jobs"),
        evidence: Vec::new(),
        request: text.into(),
    };
    let queued =
        athena_core::steward::Applier::apply(&RequestQueue, &change, &BTreeSet::new()).unwrap();
    let committer = Committer {
        git: &athena.git,
        state: &athena.state,
    };
    committer
        .commit_records(
            std::slice::from_ref(&queued.file),
            &message(
                "Queue a request",
                "A request on the steward's channel.",
                &[("Athena-Job", queued.job.as_str().to_string())],
            ),
        )
        .unwrap();
    queued.job.as_str().to_string()
}

#[test]
fn the_real_steps_work_a_queued_request_land_it_and_only_then_rate_the_page() {
    let fx = Fixture::new();
    let mut scenario = Scenario::default();
    scenario.audience = json!({
        "rating": 4,
        "findings": [{ "text": "Clear.", "source": "heading: Start here" }]
    });
    let invocations = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let job = queue_steward_request(&athena, "Add a greeting to the README.");
    // A request the user deferred is not the steward's, and is left alone.
    let (deferred, _) = athena.queue("Something the user deferred.").unwrap();

    let lander = RecordingLander::default();
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &lander,
    };
    let queue = steps.queue().unwrap();
    assert_eq!(
        queue
            .items
            .iter()
            .map(|i| i.job.as_str())
            .collect::<Vec<_>>(),
        [job.as_str()]
    );
    assert!(queue.waiting.is_empty());
    assert!(steps.caps().unwrap().run > 0.0);

    let report = drain::drain(&steps, limits(50.0), &Quiet).unwrap();
    assert_eq!(report.stop, Stop::TargetMet, "{}", display::drain(&report));
    assert_eq!(report.landed, 1);
    // The land step was asked once, for the job, before any rating existed.
    assert_eq!(*lander.0.lock().unwrap(), [(job.clone(), 0)]);
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    assert_eq!(
        records.audience_ratings.len(),
        1,
        "rated once, after landing"
    );
    assert!(report.rating.unwrap().after_landing);
    assert!(report.spent_usd > 0.0);
    assert_eq!(report.jobs[0].outcome, "passed");
    // The deferred request was never read.
    assert_eq!(
        records.jobs[deferred.as_str()].state().name(),
        "queued",
        "the user's own deferred request is untouched"
    );
    assert!(
        invocations
            .lock()
            .unwrap()
            .contains(&athena_data::records::Role::Audience)
    );
}
