//! `athena drain`: the loop against a stubbed queue and a stubbed land step (every stop reason, and that a
//! rating is only taken after a landing), and once against the fake agent end to end. No test reaches the
//! network.

// Scenarios are set up field by field, so each test shows only what differs from the default.
#![allow(clippy::field_reassign_with_default)]

mod common;

use athena_cli::display;
use athena_cli::drain::{
    self, Approval, Caps, Item, Lander, Landing, Limits, QueueView, Rate, Read, ReadOf, RealSteps,
    Run, Steps, Stop, Waiting,
};
use athena_core::audience::Page;
use athena_core::commits::{Committer, message};
use athena_core::steward::{ProposedChange, RequestQueue};
use athena_core::store::{JobState, Records};
use athena_core::{Athena, Quiet};
use athena_data::RecordId;
use athena_data::records::{
    AnyRecord, OutcomeState, Reference, Role, WorkDecisionState, WorkItem, WorkItemKind,
};
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
    /// Approved jobs already read, in state proposed.
    read_items: Vec<&'static str>,
    waiting: Vec<Waiting>,
    /// Open proposals within their role's limit.
    approvable: Vec<Waiting>,
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
        let item = |j: &&str, read: bool| Item {
            read,
            ..Item::queued(j, &format!("title of {j}"))
        };
        Ok(QueueView {
            items: self
                .read_items
                .iter()
                .map(|j| item(j, true))
                .chain(self.items.iter().map(|j| item(j, false)))
                .collect(),
            waiting: self.waiting.clone(),
            approvable: self.approvable.clone(),
        })
    }
    fn approve(&self, proposal: &str) -> Approval {
        self.log.borrow_mut().push(format!("approve {proposal}"));
        Approval {
            job: Some(format!("job-of-{proposal}")),
            cost: 0.1,
            error: None,
        }
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
    // Nothing the loop did names the proposal: a proposal that needs a person is never approved or worked.
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

// ---------------------------------------------------------------------------------------------------------
// Proposals and approved jobs

fn in_limit_proposal() -> Waiting {
    Waiting {
        id: "20261015-130000-lowr".into(),
        title: "Fix the typo in the README".into(),
        rung: "implementation".into(),
    }
}

#[test]
fn an_in_limit_proposal_is_approved_then_its_job_is_run_and_landed_without_a_second_reading() {
    let stub = Stub {
        approvable: vec![in_limit_proposal()],
        outcomes: passing(),
        ratings: RefCell::new(vec![3]),
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();
    assert_eq!(
        stub.log(),
        [
            "approve 20261015-130000-lowr",
            "run job-of-20261015-130000-lowr",
            "land job-of-20261015-130000-lowr",
            "rate"
        ],
        "approved, run and landed; the approval was the reading"
    );
    assert_eq!(report.landed, 1);
    assert!(report.waiting.is_empty(), "{:?}", report.waiting);
    assert_eq!(report.jobs[0].landing, "landed");
    // 0.1 approval + 0.5 run + 0.2 rating
    assert!(
        (report.spent_usd - 0.8).abs() < 1e-9,
        "{}",
        report.spent_usd
    );
}

#[test]
fn an_approved_job_still_in_state_proposed_is_run_once_and_not_read_again() {
    let stub = Stub {
        read_items: vec!["approved-job"],
        outcomes: passing(),
        ratings: RefCell::new(vec![3]),
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();
    assert_eq!(
        stub.log(),
        ["run approved-job", "land approved-job", "rate"],
        "no reading, one run"
    );
    assert_eq!(report.landed, 1);
    assert!(report.queued.is_empty());

    // If that run fails, the second attempt reads it again as a retry does.
    let stub = Stub {
        read_items: vec!["approved-job"],
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();
    assert_eq!(
        stub.log(),
        [
            "run approved-job",
            "reread approved-job",
            "run approved-job-retry"
        ]
    );
    assert_eq!(report.stop, Stop::RepeatedFailure);
}

#[test]
fn an_approval_that_does_not_fit_the_budget_is_not_made_and_the_proposal_stays_open() {
    let stub = Stub {
        approvable: vec![in_limit_proposal()],
        ..Stub::default()
    };
    // The reading and the run need 1.2 together.
    let report = drain::drain(&stub, limits(1.1), &Quiet).unwrap();
    assert!(stub.log().is_empty(), "{:?}", stub.log());
    assert_eq!(report.waiting, vec![in_limit_proposal()]);
    assert!(report.notes.iter().any(|n| n.contains("not approved")));
}

const LOW: &str = "20261015-130001-a1b2";
const HIGH: &str = "20261015-130002-c3d4";
const GHOST: &str = "20261015-130003-e5f6";

fn proposal(id: &str, role: &str, rung: Ladder) -> AnyRecord {
    AnyRecord::WorkItem(Box::new(WorkItem {
        id: RecordId::parse(id).unwrap(),
        at: "2026-10-15T01:00:00Z".parse().unwrap(),
        role: role.into(),
        kind: WorkItemKind::Proposal,
        key: format!("proposal:{id}"),
        title: format!("Proposal {id}"),
        rung,
        source: Reference::path("src/lib.rs"),
        evidence: vec![Reference::path("README.md")],
    }))
}

/// A repository with the steward's role recorded and three open proposals: one on the implementation rung
/// from the steward, one on a design rung from the steward, and one on the implementation rung from a role
/// that has no recorded definition.
fn seeded_proposals(fx: &Fixture) {
    let role = AnyRecord::Role(Box::new(athena_data::roles::steward(
        "2026-10-15T00:00:00Z".parse().unwrap(),
    )));
    for record in [
        role,
        proposal(LOW, "steward", Ladder::Implementation),
        proposal(HIGH, "steward", Ladder::DesignDecisions),
        proposal(GHOST, "ghost", Ladder::Implementation),
    ] {
        write(fx.root(), &[(&record.canonical_path(), &record.to_json())]);
    }
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run(["commit", "-q", "-m", "Seed proposals"])
        .unwrap();
}

fn rated_scenario() -> Scenario {
    let mut scenario = Scenario::default();
    scenario.audience = json!({
        "rating": 4,
        "findings": [{ "text": "Clear.", "source": "heading: Start here" }]
    });
    scenario
}

fn ids(list: &[Waiting]) -> Vec<&str> {
    list.iter().map(|w| w.id.as_str()).collect()
}

#[test]
fn the_real_steps_approve_only_the_in_limit_proposal_and_leave_the_others_waiting() {
    let fx = Fixture::new();
    seeded_proposals(&fx);
    let athena = fx.athena(rated_scenario().script());
    let lander = RecordingLander::default();
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &lander,
    };
    let queue = steps.queue().unwrap();
    assert_eq!(ids(&queue.approvable), [LOW]);
    assert_eq!(ids(&queue.waiting), [HIGH, GHOST]);
    // The steps refuse to approve what is not in limit, whoever asks.
    for id in [HIGH, GHOST] {
        let refused = steps.approve(id);
        assert!(
            refused.job.is_none() && refused.error.is_some(),
            "{refused:?}"
        );
    }
    let before = Records::load(&fx.git, &fx.head()).unwrap();
    assert!(before.work_decisions.is_empty());

    let report = drain::drain(&steps, limits(50.0), &Quiet).unwrap();
    assert_eq!(report.stop, Stop::TargetMet, "{}", display::drain(&report));
    assert_eq!(report.landed, 1);
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    let [decision] = records.work_decisions.as_slice() else {
        panic!("one decision: {:?}", records.work_decisions)
    };
    assert_eq!(decision.work_item.as_str(), LOW);
    let WorkDecisionState::Approved { job } = &decision.decision else {
        panic!("approved, not rejected")
    };
    // The approved job was run and landed; the land step was asked once.
    assert_eq!(*lander.0.lock().unwrap(), [(job.as_str().to_string(), 0)]);
    assert_eq!(report.jobs[0].job, job.as_str());
    assert_eq!(report.jobs[0].outcome, "passed");
    assert_eq!(
        records.jobs[job.as_str()].state(),
        JobState::Closed(OutcomeState::Landed)
    );
    assert_eq!(records.jobs[job.as_str()].readings.len(), 1);
    // The two others are untouched and still waiting, with the text that says a person decides.
    assert_eq!(ids(&report.waiting), [HIGH, GHOST]);
    let text = display::drain(&report);
    assert!(text.contains("needs a person"), "{text}");
    assert!(!text.contains(LOW), "{text}");
}

#[test]
fn an_approved_job_is_picked_up_once_and_a_closed_one_is_not_picked_up_again() {
    let fx = Fixture::new();
    seeded_proposals(&fx);
    let scenario = rated_scenario();
    let invocations = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    // A person approved the proposal earlier; its job was read and waits in state proposed.
    let reading = athena.approve_work(LOW, &Quiet).unwrap();
    let job = reading.job.as_str().to_string();
    let lander = RecordingLander::default();
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &lander,
    };
    let queue = steps.queue().unwrap();
    assert!(queue.approvable.is_empty(), "it is decided already");
    assert_eq!(queue.items.len(), 1);
    assert_eq!(
        (queue.items[0].job.as_str(), queue.items[0].read),
        (job.as_str(), true)
    );

    let readers = || {
        invocations
            .lock()
            .unwrap()
            .iter()
            .filter(|r| **r == Role::Reader)
            .count()
    };
    assert_eq!(readers(), 1, "the approval's reading");
    let report = drain::drain(&steps, limits(50.0), &Quiet).unwrap();
    assert_eq!(report.landed, 1, "{}", display::drain(&report));
    assert_eq!(readers(), 1, "drain did not read it again");
    assert_eq!(lander.0.lock().unwrap().len(), 1);

    // Now the job is closed: a second drain finds nothing to work.
    let again = steps.queue().unwrap();
    assert!(again.items.is_empty(), "{:?}", again.items);
    let report = drain::drain(&steps, limits(50.0), &Quiet).unwrap();
    assert_eq!(report.stop, Stop::QueueEmpty);
    assert_eq!(lander.0.lock().unwrap().len(), 1, "landed once, not twice");
}
