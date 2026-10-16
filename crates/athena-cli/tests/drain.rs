//! `athena drain`: the loop against a stubbed queue and a stubbed land step (every stop reason, and that a
//! rating is only taken after a landing), and once against the fake agent end to end. No test reaches the
//! network.

// Scenarios are set up field by field, so each test shows only what differs from the default.
#![allow(clippy::field_reassign_with_default)]

mod common;

use athena_agent::fake::Script;
use athena_cli::display;
use athena_cli::drain::{
    self, Approval, Caps, Check, Item, Lander, Landing, Limits, QueueView, Rate, Read, ReadOf,
    RealSteps, Run, Steps, Stop, Waiting,
};
use athena_core::audience::Page;
use athena_core::commits::{Committer, message};
use athena_core::plan::PlanEntry;
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
use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

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
    /// When set, the check of the page against the pinned source fails with this finding.
    failing_check: Option<&'static str>,
    /// What the source check costs, and the cap the loop must reserve for it; zero when not set.
    check_cost: f64,
    check_cap: f64,
    /// Jobs whose landing changes only paths outside the explainer's sources; any other landing also changes a
    /// study file.
    unrelated: Vec<&'static str>,
    /// Plan entries of the open requests, oldest first; empty means no planning data.
    plan: Vec<PlanEntry>,
    /// Plan entries as they stand once the item has been read (what a reading adds to the request's words).
    after_read: Vec<PlanEntry>,
    /// When set, a stop is asked for as soon as this job is run (the user types `athena drain-stop` mid-run).
    stop_when_running: Option<&'static str>,
    stop: Cell<bool>,
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
            plan: self.plan.clone(),
        })
    }
    fn stop_requested(&self) -> bool {
        self.stop.get()
    }
    fn plan_entry(&self, job: &str) -> Option<PlanEntry> {
        self.after_read.iter().find(|e| e.id == job).cloned()
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
        if self.stop_when_running == Some(job) {
            self.stop.set(true);
        }
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
            let paths = if self.unrelated.contains(&job) {
                vec!["README.md", "crates/athena-cli/src/drain.rs"]
            } else {
                vec!["README.md", "studies/demo/claims.json"]
            };
            Landing::Landed {
                changed: paths.into_iter().map(String::from).collect(),
            }
        }
    }
    fn check_page(&self) -> Check {
        self.log.borrow_mut().push("check".into());
        match &self.failing_check {
            Some(why) => Check::fail(*why),
            None => Check::pass("stub"),
        }
        .costing(self.check_cost)
    }
    fn check_cap(&self) -> f64 {
        self.check_cap
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

fn planned(id: &str, rung: Ladder, touches: &[&str], deps: &[&str]) -> PlanEntry {
    PlanEntry {
        goal: format!("goal of {id}"),
        rung: Some(rung),
        acceptance: vec![format!("{id} works")],
        depends_on: deps.iter().map(|d| d.to_string()).collect(),
        touches: touches.iter().map(|t| t.to_string()).collect(),
        ..PlanEntry::bare(id, &format!("title of {id}"))
    }
}

#[test]
fn items_are_worked_in_plan_order_not_oldest_first() {
    // a is oldest, c is newest; c serves a higher rung, and b needs a.
    let stub = Stub {
        items: vec!["a", "b", "c"],
        outcomes: passing(),
        ratings: RefCell::new(vec![1, 1, 1]),
        plan: vec![
            planned("a", Ladder::Implementation, &["src/a.rs"], &[]),
            planned("b", Ladder::Implementation, &["src/b.rs"], &["a"]),
            planned("c", Ladder::DesignDecisions, &["docs/c.md"], &[]),
        ],
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(9.0), &Quiet).unwrap();
    let reads: Vec<String> = stub
        .log()
        .into_iter()
        .filter(|e| e.starts_with("read "))
        .collect();
    assert_eq!(reads, ["read c", "read a", "read b"]);
    assert_eq!(report.stop, Stop::QueueEmpty);
    // c and a share no dependency and touch different paths, so they form one group; b waits for a.
    assert_eq!(
        report.plan_groups,
        [
            vec!["c".to_string(), "a".to_string()],
            vec!["b".to_string()]
        ]
    );
    let text = display::drain(&report);
    assert!(text.contains("could run in parallel"), "{text}");
}

#[test]
fn an_item_whose_plan_lacks_goal_rung_or_acceptance_is_not_worked_and_is_reported() {
    let mut no_goal = planned("a", Ladder::Implementation, &["a"], &[]);
    no_goal.goal.clear();
    let mut no_rung = planned("b", Ladder::Implementation, &["b"], &[]);
    no_rung.rung = None;
    let mut no_acceptance = planned("c", Ladder::Implementation, &["c"], &[]);
    no_acceptance.acceptance.clear();
    let stub = Stub {
        items: vec!["a", "b", "c", "d"],
        outcomes: passing(),
        ratings: RefCell::new(vec![1]),
        plan: vec![
            no_goal,
            no_rung,
            no_acceptance,
            planned("d", Ladder::Implementation, &["d"], &[]),
        ],
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(9.0), &Quiet).unwrap();
    // The pass reads the three underspecified items (the stub's readings add nothing), then d is worked.
    assert_eq!(
        stub.log(),
        [
            "read a", "read b", "read c", "read d", "run d", "land d", "rate"
        ]
    );
    let refused: Vec<(String, Vec<String>)> = report
        .refused
        .iter()
        .map(|r| (r.id.clone(), r.missing.clone()))
        .collect();
    assert_eq!(
        refused,
        [
            ("a".to_string(), vec!["goal".to_string()]),
            ("b".to_string(), vec!["ladder rung".to_string()]),
            ("c".to_string(), vec!["acceptance".to_string()]),
        ]
    );
    let queued: Vec<&str> = report.queued.iter().map(|i| i.job.as_str()).collect();
    assert_eq!(queued, ["a", "b", "c"], "they stay queued");
    let text = display::drain(&report);
    assert!(text.contains("the plan lacks acceptance"), "{text}");
}

/// What planning finds in an unread "make it better": a goal and nothing else.
fn vague(id: &str) -> PlanEntry {
    PlanEntry {
        goal: "make it better".into(),
        ..PlanEntry::bare(id, "make it better")
    }
}

#[test]
fn an_unread_vague_request_is_read_by_the_pass_then_refused_with_a_reason_and_the_specified_one_runs()
 {
    let stub = Stub {
        items: vec!["vague", "spec"],
        outcomes: passing(),
        ratings: RefCell::new(vec![1]),
        plan: vec![
            vague("vague"),
            planned("spec", Ladder::Implementation, &["src/a.rs"], &[]),
        ],
        // The reading adds nothing to the vague request.
        after_read: vec![vague("vague")],
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(9.0), &Quiet).unwrap();
    // The vague item was read (not refused merely for being unread) and never run; the specified one ran.
    assert_eq!(
        stub.log(),
        ["read vague", "read spec", "run spec", "land spec", "rate"]
    );
    assert_eq!(report.refused.len(), 1);
    assert_eq!(report.refused[0].id, "vague");
    assert_eq!(report.refused[0].missing, ["ladder rung", "acceptance"]);
    assert!(report.refused[0].reason.contains("acceptance"));
    assert_eq!(report.jobs.len(), 1);
    assert_eq!(report.jobs[0].item, "spec");
    assert_eq!(report.landed, 1);
    let text = display::drain(&report);
    assert!(text.contains("refused     vague"), "{text}");
    assert!(text.contains("is not worked"), "{text}");
}

#[test]
fn an_unread_but_specifiable_request_is_read_once_and_then_runs() {
    let stub = Stub {
        items: vec!["x"],
        outcomes: passing(),
        ratings: RefCell::new(vec![1]),
        plan: vec![vague("x")],
        // Its reading supplies the goal, the rung and the acceptance.
        after_read: vec![planned("x", Ladder::Implementation, &["src/x.rs"], &[])],
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(9.0), &Quiet).unwrap();
    assert_eq!(
        stub.log(),
        ["read x", "run x", "land x", "rate"],
        "read by the pass, run without a second reading"
    );
    assert!(report.refused.is_empty(), "{:?}", report.refused);
    assert_eq!(report.landed, 1);
    // The pass's reading is charged to the job: 0.1 read + 0.5 run.
    assert!((report.jobs[0].cost_usd - 0.6).abs() < 1e-9);
}

#[test]
fn the_pass_reads_only_within_the_budget() {
    let stub = Stub {
        items: vec!["vague"],
        plan: vec![vague("vague")],
        ..Stub::default()
    };
    // A read (cap 0.2) plus a run (cap 1.0) does not fit 1.0.
    let report = drain::drain(&stub, limits(1.0), &Quiet).unwrap();
    assert!(stub.log().is_empty(), "{:?}", stub.log());
    assert_eq!(report.refused.len(), 1);
    assert!(report.notes.iter().any(|n| n.contains("no room")));
}

#[test]
fn plan_prints_order_groups_details_and_flagged_items_and_says_so_when_nothing_is_open() {
    let mut a = planned("a", Ladder::Implementation, &["src/a.rs"], &[]);
    a.open_questions = vec!["the touched path src/a.rs does not exist at the base commit".into()];
    let stub = Stub {
        items: vec!["a", "b", "vague"],
        plan: vec![
            a,
            planned("b", Ladder::DesignDecisions, &["docs/b.md"], &["a"]),
            vague("vague"),
        ],
        ..Stub::default()
    };
    let report = drain::plan(&stub, 5.0, &Quiet).unwrap();
    assert_eq!(report.plan.order, ["a", "b"]);
    assert_eq!(
        stub.log(),
        ["read vague"],
        "only the underspecified one is read"
    );
    let text = display::plan(&report);
    for needle in [
        "order       a → b",
        "group 1     a",
        "group 2     b",
        "goal        goal of a",
        "rung        implementation",
        "acceptance  a works",
        "questions   the touched path src/a.rs does not exist",
        "depends on  a",
        "touches     docs/b.md",
        "flagged     vague · make it better lacks ladder rung, acceptance",
    ] {
        assert!(text.contains(needle), "{needle}\n{text}");
    }
    let none = drain::plan(&Stub::default(), 5.0, &Quiet).unwrap();
    assert!(display::plan(&none).contains("nothing is open"));
    // With no budget nothing is read; the unread item is flagged all the same.
    let unread = Stub {
        items: vec!["vague"],
        plan: vec![vague("vague")],
        ..Stub::default()
    };
    let report = drain::plan(&unread, 0.0, &Quiet).unwrap();
    assert!(unread.log().is_empty());
    assert_eq!(report.plan.refused.len(), 1);
}

#[test]
fn a_stop_asked_for_mid_run_finishes_the_current_job_and_starts_no_other() {
    let stub = Stub {
        items: vec!["a", "b", "c"],
        outcomes: passing(),
        ratings: RefCell::new(vec![1, 1, 1]),
        stop_when_running: Some("a"),
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(9.0), &Quiet).unwrap();
    // a was already running when the stop came: it is landed and rated. Neither b nor c is read.
    assert_eq!(stub.log(), ["read a", "run a", "land a", "rate"]);
    assert_eq!(report.stop, Stop::Requested);
    assert_eq!(report.jobs.len(), 1);
    assert_eq!(report.jobs[0].landing, "landed", "the job was not killed");
    assert_eq!(report.landed, 1);
    let queued: Vec<&str> = report.queued.iter().map(|i| i.job.as_str()).collect();
    assert_eq!(queued, ["b", "c"]);
    assert!(!report.target_met());
    let text = display::drain(&report);
    assert!(text.contains("stop requested"), "{text}");
}

#[test]
fn a_stop_asked_for_before_any_job_starts_runs_nothing() {
    let stub = Stub {
        items: vec!["a"],
        stop: Cell::new(true),
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(9.0), &Quiet).unwrap();
    assert!(stub.log().is_empty(), "{:?}", stub.log());
    assert_eq!(report.stop, Stop::Requested);
}

#[test]
fn the_stop_file_is_in_the_state_directory() {
    let dir = Path::new("/state/repo");
    assert_eq!(drain::stop_file(dir), dir.join("drain.stop"));
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
            "read a", "run a", "land a", "rate", "read b", "run b", "land b", "rate", "check"
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
fn a_rating_at_the_target_stops_drain_only_with_a_passing_source_check() {
    let run = |failing_check| {
        let stub = Stub {
            items: vec!["a", "b"],
            outcomes: passing(),
            ratings: RefCell::new(vec![4, 4]),
            failing_check,
            ..Stub::default()
        };
        let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();
        (stub, report)
    };

    // A 4 with a passing check meets the target after the first landing.
    let (stub, report) = run(None);
    assert_eq!(report.stop, Stop::TargetMet);
    assert!(report.target_met());
    assert_eq!(report.held_back, None);
    assert_eq!(stub.log(), ["read a", "run a", "land a", "rate", "check"]);

    // A 4 with a failing check does not: drain goes on to the next item and ends for another reason.
    let (stub, report) = run(Some("counter places-held omits t4"));
    assert_eq!(report.stop, Stop::QueueEmpty);
    assert!(!report.target_met());
    assert_eq!(report.landed, 2, "drain kept working after the 4");
    assert_eq!(
        report.held_back.as_deref(),
        Some("counter places-held omits t4")
    );
    assert!(!report.rating.unwrap().met_target);
    assert_eq!(stub.log().iter().filter(|e| *e == "check").count(), 2);
    let text = display::drain(&report);
    assert!(text.contains("held back"), "{text}");
    assert!(text.contains("places-held omits t4"), "{text}");
    assert!(!text.contains("target met"), "{text}");
}

#[test]
fn a_rating_below_the_target_does_not_ask_for_the_source_check() {
    let stub = Stub {
        items: vec!["a"],
        outcomes: passing(),
        ratings: RefCell::new(vec![3]),
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();
    assert_eq!(report.held_back, None);
    assert!(!stub.log().contains(&"check".to_string()));
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
    // Both rules for "target met" are in the help text.
    for needle in [
        "did not change the outcome",
        "studies/",
        "pinned source",
        "contradicted or omitted",
    ] {
        assert!(help.contains(needle), "{needle}: {help}");
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
        // `RealSteps` replaces this with the paths the job's candidate changed.
        Landing::Landed {
            changed: Vec::new(),
        }
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
    let scenario = rated_scenario();
    let invocations = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let job = queue_steward_request(&athena, "Add a greeting to the README.");

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
    assert!(
        invocations
            .lock()
            .unwrap()
            .contains(&athena_data::records::Role::Audience)
    );
}

#[test]
fn a_request_deferred_with_ask_later_is_listed_and_worked_like_a_steward_item() {
    let fx = Fixture::new();
    let scenario = rated_scenario();
    let athena = fx.athena(scenario.script());
    // `ask --later` records the request on the "cli" channel.
    let (deferred, _) = athena.queue("Add a greeting to the README.").unwrap();

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
        [deferred.as_str()]
    );

    let report = drain::drain(&steps, limits(50.0), &Quiet).unwrap();
    assert_eq!(report.stop, Stop::TargetMet, "{}", display::drain(&report));
    assert_eq!(report.landed, 1);
    assert_eq!(
        *lander.0.lock().unwrap(),
        [(deferred.as_str().to_string(), 0)]
    );
    assert_eq!(report.jobs[0].outcome, "passed");
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    assert_ne!(
        records.jobs[deferred.as_str()].state().name(),
        "queued",
        "the deferred request was read and run"
    );
}

/// The rated scenario, except that the reader declines any request that says "make it better".
fn declining_vague(scenario: Scenario) -> Script {
    let inner = scenario.script();
    Box::new(move |spec| {
        if spec.role == Role::Reader && spec.user_prompt.contains("make it better") {
            let mut declined = reading(&[], &[]);
            declined["capability"] = json!(null);
            declined["decline_reason"] = json!("it does not say what to improve");
            declined["title"] = json!("Make it better");
            declined["summary"] = json!("Make something better.");
            declined["defaults_applied"] = json!([]);
            return athena_agent::fake::FakeTurn::output(declined);
        }
        inner(spec)
    })
}

#[test]
fn real_steps_read_an_unread_vague_request_and_refuse_it_while_the_specified_one_runs() {
    let fx = Fixture::new();
    let scenario = rated_scenario();
    let invocations = scenario.invocations.clone();
    let athena = fx.athena(declining_vague(scenario));
    let (vague_job, _) = athena.queue("make it better").unwrap();
    let (spec_job, _) = athena
        .queue(
            "Add the demo study to studies/demo/\n- the demo study has a pin, claims and an intro",
        )
        .unwrap();

    let lander = RecordingLander::default();
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &lander,
    };
    // Before the pass: the vague request has a goal and nothing else.
    let queue = steps.queue().unwrap();
    let vague_entry = queue
        .plan
        .iter()
        .find(|e| e.id == vague_job.as_str())
        .unwrap();
    assert_eq!(vague_entry.missing(), ["ladder rung", "acceptance"]);

    let report = drain::drain(&steps, limits(50.0), &Quiet).unwrap();
    let text = display::drain(&report);
    // The vague request was read by the pass, not refused for being unread...
    let readers = invocations
        .lock()
        .unwrap()
        .iter()
        .filter(|r| **r == Role::Reader)
        .count();
    // (The declining reader of the vague request answers outside the scenario, so only the other is counted.)
    assert_eq!(readers, 1, "the specified request was read once: {text}");
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    assert_eq!(records.jobs[vague_job.as_str()].readings.len(), 1);
    // ...then refused with a reason, and never run as a job.
    assert_eq!(report.refused.len(), 1, "{text}");
    assert_eq!(report.refused[0].id, vague_job.as_str());
    assert!(report.refused[0].reason.contains("acceptance"), "{text}");
    assert!(report.jobs.iter().all(|j| j.item != vague_job.as_str()));
    assert!(
        records.jobs[vague_job.as_str()].job.is_none(),
        "not dispatched"
    );
    // The specified request ran and landed.
    assert_eq!(report.landed, 1, "{text}");
    assert_eq!(report.jobs.len(), 1);
    assert_eq!(report.jobs[0].item, spec_job.as_str());
}

#[test]
fn real_steps_read_an_unread_but_specifiable_request_once_and_then_run_it() {
    let fx = Fixture::new();
    let scenario = rated_scenario();
    let invocations = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    // No bullets: its own words give no acceptance, so the pass reads it; the reading gives some.
    let (job, _) = athena
        .queue("Add the demo study to the studies directory.")
        .unwrap();
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &RecordingLander::default(),
    };
    let report = drain::drain(&steps, limits(50.0), &Quiet).unwrap();
    let text = display::drain(&report);
    assert!(report.refused.is_empty(), "{text}");
    assert_eq!(report.landed, 1, "{text}");
    assert_eq!(report.jobs[0].item, job.as_str());
    let readers = invocations
        .lock()
        .unwrap()
        .iter()
        .filter(|r| **r == Role::Reader)
        .count();
    assert_eq!(readers, 1, "read by the pass, not again to run");
}

#[test]
fn the_plan_of_a_read_job_names_a_missing_path_and_an_unresolved_referenced_job() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let (job, _) = athena
        .queue("Edit crates/gone/lib.rs and README.md after 20261012-075824-1x28\n- it is edited")
        .unwrap();
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &RecordingLander::default(),
    };
    let queue = steps.queue().unwrap();
    let entry = queue.plan.iter().find(|e| e.id == job.as_str()).unwrap();
    assert_eq!(entry.rung, Some(Ladder::Implementation));
    assert_eq!(
        entry.open_questions,
        [
            "the touched path crates/gone/lib.rs does not exist at the base commit",
            "the referenced job 20261012-075824-1x28 is not among the records"
        ]
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

/// A scenario whose job adds a study (a pin, claims and an intro), so its landing changes what the explainer
/// is built from, and whose audience rates the page 4. Its source checker finds nothing wrong.
fn rated_scenario() -> Scenario {
    let mut scenario = Scenario::default();
    scenario.audience = json!({
        "rating": 4,
        "findings": [{ "text": "Clear.", "source": "heading: Start here" }]
    });
    scenario.reading = reading(
        &[
            ("studies/demo/pin.json", "create"),
            ("studies/demo/claims.json", "create"),
            ("studies/demo/intro.json", "create"),
        ],
        &[
            ("content", "the demo study has a pin, claims and an intro"),
            ("behavior", "the tests still pass"),
        ],
    );
    scenario.edits = Texts::correct()
        .files()
        .into_iter()
        .filter(|(path, _)| ["pin.json", "claims.json", "intro.json"].contains(path))
        .map(|(path, body)| (format!("studies/demo/{path}"), Some(body)))
        .collect();
    scenario.report["files"] = json!([
        "studies/demo/pin.json",
        "studies/demo/claims.json",
        "studies/demo/intro.json"
    ]);
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

// ---------------------------------------------------------------------------------------------------------
// "Target met" needs an outcome-changing landing

#[test]
fn a_landing_outside_the_explainers_sources_cannot_meet_the_target_even_at_the_rating() {
    let stub = Stub {
        items: vec!["a"],
        outcomes: passing(),
        ratings: RefCell::new(vec![5]),
        unrelated: vec!["a"],
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();
    assert_eq!(report.landed, 1);
    assert_ne!(report.stop, Stop::TargetMet);
    assert!(!report.target_met());
    assert!(!report.outcome_changed);
    // The rating was taken, but the source check, a model call, was not asked: nothing could use it.
    assert_eq!(stub.log(), ["read a", "run a", "land a", "rate"]);
    let text = display::drain(&report);
    assert!(text.contains("no landing changed the outcome"), "{text}");
    assert!(!text.contains("target met"), "{text}");
    assert_eq!(
        serde_json::to_value(&report).unwrap()["outcome_changed"],
        false
    );
}

#[test]
fn a_later_landing_that_changes_a_study_meets_the_target_after_an_unrelated_one() {
    let stub = Stub {
        items: vec!["a", "b"],
        outcomes: passing(),
        ratings: RefCell::new(vec![4, 4]),
        unrelated: vec!["a"],
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();
    assert_eq!(report.stop, Stop::TargetMet);
    assert!(report.target_met() && report.outcome_changed);
    assert_eq!(
        stub.log(),
        [
            "read a", "run a", "land a", "rate", "read b", "run b", "land b", "rate", "check"
        ]
    );
}

#[test]
fn the_site_generator_is_an_outcome_changing_path() {
    use drain::{OUTCOME_PATHS, changes_outcome};
    assert!(changes_outcome(&["xtask/src/site.rs".to_string()]));
    assert!(changes_outcome(&["studies/demo/intro.json".to_string()]));
    assert!(!changes_outcome(&["docs/decisions.md".to_string()]));
    assert!(OUTCOME_PATHS.contains(&"studies/") && OUTCOME_PATHS.contains(&"xtask/"));
}

// ---------------------------------------------------------------------------------------------------------
// The source check is guarded by the budget and counted in the spend

#[test]
fn the_source_check_runs_only_when_its_whole_cap_fits_and_otherwise_drain_stops_as_budget_spent() {
    // One item costs 0.8 (read 0.1, run 0.5, rating 0.2). The check's cap of 1.0 does not fit a budget of 1.5.
    let stub = Stub {
        items: vec!["a"],
        outcomes: passing(),
        ratings: RefCell::new(vec![5]),
        check_cap: 1.0,
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(1.5), &Quiet).unwrap();
    assert_eq!(report.stop, Stop::BudgetSpent);
    assert!(!report.target_met());
    assert!(
        !stub.log().contains(&"check".to_string()),
        "{:?}",
        stub.log()
    );
    assert!(report.notes.iter().any(|n| n.contains("source check")));
    // The same run with room for the cap meets the target.
    let stub = Stub {
        items: vec!["a"],
        outcomes: passing(),
        ratings: RefCell::new(vec![5]),
        check_cap: 1.0,
        ..Stub::default()
    };
    let report = drain::drain(&stub, limits(1.9), &Quiet).unwrap();
    assert_eq!(report.stop, Stop::TargetMet);
}

#[test]
fn what_the_source_check_costs_is_added_to_the_spend_even_when_it_fails() {
    for failing_check in [None, Some("a statement is contradicted")] {
        let stub = Stub {
            items: vec!["a"],
            outcomes: passing(),
            ratings: RefCell::new(vec![5]),
            check_cap: 1.0,
            check_cost: 0.7,
            failing_check,
            ..Stub::default()
        };
        let report = drain::drain(&stub, limits(5.0), &Quiet).unwrap();
        // 0.1 read + 0.5 run + 0.2 rating + 0.7 check
        assert!(
            (report.spent_usd - 1.5).abs() < 1e-9,
            "{failing_check:?}: {}",
            report.spent_usd
        );
    }
}

// ---------------------------------------------------------------------------------------------------------
// The real source check, against a fake agent and a stub pinner

const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// The text of a demo study, one string per input: a claim, the intro, the transition, a glossary entry, and
/// the fact the intro must state.
#[derive(Clone)]
struct Texts {
    claim: &'static str,
    intro: &'static str,
    transition: &'static str,
    glossary: &'static str,
    fact: &'static str,
}

/// What the source says, as the fake checker knows it: a worker holds a place until its pull request is ready
/// or it is cleaned up; with yolo off a person merges; teardown can be reopened; a full spawn exits at once.
const FACT: &str = "holds a place until its pull request is ready or it is cleaned up";
const WRONG_CLAIM: &str = "Teardown closes a task for good.";
const WRONG_INTRO: &str = "With yolo on, a worker may merge its own pull request.";
const WRONG_TRANSITION: &str = "A spawn at capacity waits and retries by itself.";
const WRONG_GLOSSARY: &str = "A slot of capacity that is freed when the worker passes its task on.";

impl Texts {
    fn correct() -> Texts {
        Texts {
            claim: "Teardown closes a task, and a closed task can reopen.",
            intro: "With yolo off, a person merges every pull request.",
            transition: "A spawn at capacity exits at once and the task stays queued.",
            glossary: "A slot of capacity that a worker holds until its pull request is ready or it is cleaned up.",
            fact: FACT,
        }
    }

    /// Every file of the study directory.
    fn files(&self) -> Vec<(&'static str, String)> {
        vec![
            (
                "pin.json",
                json!({"schema": "study.pin/1", "study_id": "demo", "repository_url": "https://example.com/demo.git", "sha": SHA}).to_string(),
            ),
            (
                "claims.json",
                json!([{ "id": "c1", "text": self.claim }]).to_string(),
            ),
            (
                "intro.json",
                json!({
                    "intro": format!("A worker {}. {}", self.fact, self.intro),
                    "transition": self.transition,
                    "glossary": [{ "term": "place", "definition": self.glossary }]
                })
                .to_string(),
            ),
            (
                "model.json",
                json!({ "counters": [], "transitions": [] }).to_string(),
            ),
            ("validation.json", json!({ "results": [] }).to_string()),
        ]
    }
}

/// The fake checker: reports every wrong statement it finds in the page text, names the file, and reports a
/// fact the source requires when no file states it. It reads exactly what the real checker is given.
fn judge(workspace: &Path) -> serde_json::Value {
    let files = page_text_files(workspace);
    let mut problems = Vec::new();
    let mut all = String::new();
    for file in &files {
        let text = std::fs::read_to_string(workspace.join("page-text").join(file)).unwrap();
        for wrong in [WRONG_CLAIM, WRONG_INTRO, WRONG_TRANSITION, WRONG_GLOSSARY] {
            if text.contains(wrong) {
                problems.push(json!({
                    "file": file, "kind": "contradicted", "statement": wrong,
                    "evidence": "pinned-source/bin/fm-spawn.sh lines 10-20"
                }));
            }
        }
        all.push_str(&text);
    }
    if !all.contains(FACT) {
        problems.push(json!({
            "file": "intro.json", "kind": "omitted", "statement": FACT,
            "evidence": "pinned-source/bin/fm-backlog-transition-lib.sh lines 11-13"
        }));
    }
    json!({ "files_checked": files, "problems": problems })
}

/// Commits `files` under `studies/<name>/`.
fn commit_study(fx: &Fixture, name: &str, files: &[(&str, String)]) {
    for (file, body) in files {
        write(fx.root(), &[(&format!("studies/{name}/{file}"), body)]);
    }
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run(["commit", "-q", "-m", &format!("Add study {name}")])
        .unwrap();
}

fn judged_scenario(judge: SourceJudge) -> Scenario {
    let mut scenario = Scenario::default();
    scenario.source_check = Some(judge);
    scenario
}

/// Runs the real page check over a repository whose `demo` study has `texts`, with the fake checker.
fn real_check(texts: &Texts) -> (Check, Vec<Role>) {
    let fx = Fixture::new();
    commit_study(&fx, "demo", &texts.files());
    let scenario = judged_scenario(Arc::new(judge));
    let invocations = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &RecordingLander::default(),
    };
    let check = steps.check_page();
    let roles = invocations.lock().unwrap().clone();
    (check, roles)
}

#[test]
fn a_page_with_correct_text_in_every_file_passes_the_source_check_with_one_verifier_call() {
    let (check, roles) = real_check(&Texts::correct());
    assert!(check.passed, "{}", check.detail);
    // claims.json, intro.json and model.json; the pin and the rule results are not page text.
    assert!(
        check.detail.contains("3 page text file"),
        "{}",
        check.detail
    );
    // One call, as the verifier (its model and cap), not as the audience.
    assert_eq!(roles, [Role::Verifier]);
    assert!((check.cost - 0.01).abs() < 1e-9, "{}", check.cost);
}

#[test]
fn a_wrong_statement_in_any_page_text_input_fails_the_check_and_is_named() {
    let ok = Texts::correct();
    let cases = [
        (
            "claims.json",
            WRONG_CLAIM,
            Texts {
                claim: WRONG_CLAIM,
                ..ok.clone()
            },
        ),
        (
            "intro.json",
            WRONG_INTRO,
            Texts {
                intro: WRONG_INTRO,
                ..ok.clone()
            },
        ),
        (
            "intro.json",
            WRONG_TRANSITION,
            Texts {
                transition: WRONG_TRANSITION,
                ..ok.clone()
            },
        ),
        (
            "intro.json",
            WRONG_GLOSSARY,
            Texts {
                glossary: WRONG_GLOSSARY,
                ..ok.clone()
            },
        ),
        (
            "intro.json",
            FACT,
            Texts {
                fact: "runs for a while",
                ..ok.clone()
            },
        ),
    ];
    for (file, statement, texts) in cases {
        let (check, _) = real_check(&texts);
        assert!(!check.passed, "{statement}: {}", check.detail);
        assert!(check.detail.contains(statement), "{}", check.detail);
        assert!(
            check.detail.contains(&format!("demo/{file}")),
            "{}",
            check.detail
        );
        assert!(check.cost > 0.0, "a failed check still cost money");

        // Drain, with the real check under a stubbed queue and a rating at the target, does not stop as met.
        let fx = Fixture::new();
        commit_study(&fx, "demo", &texts.files());
        let athena = fx.athena(judged_scenario(Arc::new(judge)).script());
        let real = RealSteps {
            athena: &athena,
            ui: &Quiet,
            page: &StubPage,
            lander: &RecordingLander::default(),
        };
        let steps = WithRealCheck {
            stub: Stub {
                items: vec!["a"],
                outcomes: passing(),
                ratings: RefCell::new(vec![5]),
                ..Stub::default()
            },
            real,
        };
        let report = drain::drain(&steps, limits(50.0), &Quiet).unwrap();
        assert_ne!(report.stop, Stop::TargetMet, "{statement}");
        assert!(!report.target_met());
        let held = report.held_back.as_deref().unwrap();
        assert!(held.contains(statement), "{held}");
        let text = display::drain(&report);
        assert!(
            text.contains(statement) && !text.contains("target met"),
            "{text}"
        );
        assert!(report.spent_usd > 0.8, "the check's cost is in the spend");
    }
}

const WRONG_MODEL: &str = "reopened -> Done";

/// A study whose model.json shows a reopened item as finished; the source returns it to Queued.
fn behavior_files() -> Vec<(&'static str, String)> {
    let mut files = Texts::correct().files();
    for (name, body) in files.iter_mut() {
        if *name == "model.json" {
            *body = json!({ "counters": [], "transitions": [WRONG_MODEL] }).to_string();
        }
    }
    files
}

/// A checker that reads every file, finds every quote true, and reports only a behavior error in the model.
fn behavior_judge(workspace: &Path) -> serde_json::Value {
    let files = page_text_files(workspace);
    let model = std::fs::read_to_string(workspace.join("page-text/model.json")).unwrap();
    let problems = if model.contains(WRONG_MODEL) {
        vec![json!({
            "file": "model.json", "kind": "behavior", "statement": WRONG_MODEL,
            "evidence": "pinned-source/bin/fm-backlog-transition-lib.sh returns a reopened item to Queued"
        })]
    } else {
        vec![]
    };
    json!({ "files_checked": files, "problems": problems })
}

#[test]
fn a_model_whose_behavior_differs_from_the_code_fails_the_check_and_holds_drain_back() {
    // The same checker passes the correct model, so the behavior kind is what fails the wrong one.
    let fx = Fixture::new();
    commit_study(&fx, "demo", &Texts::correct().files());
    let athena = fx.athena(judged_scenario(Arc::new(behavior_judge)).script());
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &RecordingLander::default(),
    };
    let check = steps.check_page();
    assert!(check.passed, "{}", check.detail);

    let fx = Fixture::new();
    commit_study(&fx, "demo", &behavior_files());
    let athena = fx.athena(judged_scenario(Arc::new(behavior_judge)).script());
    let real = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &RecordingLander::default(),
    };
    let check = real.check_page();
    assert!(!check.passed, "{}", check.detail);
    assert!(check.detail.contains("behavior"), "{}", check.detail);
    assert!(check.detail.contains(WRONG_MODEL), "{}", check.detail);
    assert!(check.detail.contains("demo/model.json"), "{}", check.detail);

    let steps = WithRealCheck {
        stub: Stub {
            items: vec!["a"],
            outcomes: passing(),
            ratings: RefCell::new(vec![5]),
            ..Stub::default()
        },
        real,
    };
    let report = drain::drain(&steps, limits(50.0), &Quiet).unwrap();
    assert_ne!(report.stop, Stop::TargetMet);
    assert!(!report.target_met());
    let held = report.held_back.as_deref().unwrap();
    assert!(held.contains(WRONG_MODEL), "{held}");
}

#[test]
fn a_checker_that_skips_model_json_is_rejected_even_with_no_problems() {
    let fx = Fixture::new();
    commit_study(&fx, "demo", &Texts::correct().files());
    let skipping: SourceJudge =
        Arc::new(|_| json!({ "files_checked": ["claims.json", "intro.json"], "problems": [] }));
    let athena = fx.athena(judged_scenario(skipping).script());
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &RecordingLander::default(),
    };
    let check = steps.check_page();
    assert!(!check.passed);
    assert!(check.detail.contains("demo/model.json"), "{}", check.detail);
    assert!(
        check.detail.contains("did not report reading"),
        "{}",
        check.detail
    );
}

/// Real source check and everything else stubbed.
struct WithRealCheck<'a> {
    stub: Stub,
    real: RealSteps<'a>,
}

impl Steps for WithRealCheck<'_> {
    fn caps(&self) -> anyhow::Result<Caps> {
        self.stub.caps()
    }
    fn queue(&self) -> anyhow::Result<QueueView> {
        self.stub.queue()
    }
    fn previous_rating(&self) -> Option<u8> {
        None
    }
    fn read(&self, what: ReadOf<'_>) -> Read {
        self.stub.read(what)
    }
    fn approve(&self, proposal: &str) -> Approval {
        self.stub.approve(proposal)
    }
    fn run(&self, job: &str) -> Run {
        self.stub.run(job)
    }
    fn land(&self, job: &str) -> Landing {
        self.stub.land(job)
    }
    fn rate(&self) -> Rate {
        self.stub.rate()
    }
    fn check_page(&self) -> Check {
        self.real.check_page()
    }
    fn check_cap(&self) -> f64 {
        self.real.check_cap()
    }
}

#[test]
fn a_file_the_checker_did_not_read_fails_the_check() {
    let fx = Fixture::new();
    commit_study(&fx, "demo", &Texts::correct().files());
    // A checker that finds nothing wrong but reports reading only claims.json.
    let skipping: SourceJudge =
        Arc::new(|_| json!({ "files_checked": ["claims.json"], "problems": [] }));
    let athena = fx.athena(judged_scenario(skipping).script());
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &RecordingLander::default(),
    };
    let check = steps.check_page();
    assert!(!check.passed);
    assert!(check.detail.contains("demo/intro.json"), "{}", check.detail);
    assert!(
        check.detail.contains("did not report reading"),
        "{}",
        check.detail
    );
}

#[test]
fn a_text_file_added_to_a_study_is_checked_without_anyone_listing_it() {
    // A new file with a wrong statement fails the check; with correct text it passes.
    let mut files = Texts::correct().files();
    files.push(("notes.md", format!("# Notes\n\n{WRONG_CLAIM}\n")));
    let fx = Fixture::new();
    commit_study(&fx, "demo", &files);
    let athena = fx.athena(judged_scenario(Arc::new(judge)).script());
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &RecordingLander::default(),
    };
    let check = steps.check_page();
    assert!(!check.passed);
    assert!(check.detail.contains("demo/notes.md"), "{}", check.detail);

    let mut files = Texts::correct().files();
    files.push(("notes.md", "# Notes\n\nNothing wrong here.\n".to_string()));
    let fx = Fixture::new();
    commit_study(&fx, "demo", &files);
    let athena = fx.athena(judged_scenario(Arc::new(judge)).script());
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &RecordingLander::default(),
    };
    let check = steps.check_page();
    assert!(check.passed, "{}", check.detail);
    assert!(
        check.detail.contains("4 page text file"),
        "{}",
        check.detail
    );
}

#[test]
fn every_file_of_the_shipped_study_is_page_text_except_the_two_structural_ones() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../studies/firstmate-work-handling");
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        files.push((
            path.file_name().unwrap().to_string_lossy().into_owned(),
            std::fs::read(&path).unwrap(),
        ));
    }
    let study = drain::study_text("firstmate-work-handling", &files).unwrap();
    let mut covered: Vec<&str> = study.files.iter().map(|f| f.name.as_str()).collect();
    covered.sort_unstable();
    let mut expected: Vec<&str> = files
        .iter()
        .map(|(n, _)| n.as_str())
        .filter(|n| !athena_core::pin::STRUCTURAL_FILES.contains(n))
        .collect();
    expected.sort_unstable();
    assert_eq!(covered, expected, "every other file is read by the check");
    for text in ["claims.json", "intro.json"] {
        assert!(covered.contains(&text), "{text} is page text");
    }
    assert_eq!(study.sha.len(), 40);
}

#[test]
fn the_reserve_for_the_source_check_grows_with_the_number_of_studies() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &RecordingLander::default(),
    };
    let one_call = steps.caps().unwrap().rate;
    assert!(one_call > 0.0);
    commit_study(&fx, "one", &Texts::correct().files());
    assert!((steps.check_cap() - one_call).abs() < 1e-9);
    commit_study(&fx, "two", &Texts::correct().files());
    commit_study(&fx, "three", &Texts::correct().files());
    assert!((steps.check_cap() - 3.0 * one_call).abs() < 1e-9);
}

#[test]
fn a_repository_with_no_study_cannot_pass_the_source_check() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &RecordingLander::default(),
    };
    let check = steps.check_page();
    assert!(!check.passed);
    assert_eq!(check.cost, 0.0);
}

#[test]
fn a_study_whose_model_contradicts_itself_fails_the_floor_without_a_model_call() {
    let fx = Fixture::new();
    let mut files = Texts::correct().files();
    files.retain(|(n, _)| *n != "model.json");
    files.push((
        "model.json",
        json!({
            "counters": [{ "id": "places-held", "changed_by": ["t1"] }],
            "transitions": [{ "id": "t2", "retry": { "changes": ["places-held falls when a PR is ready or it is cleaned up"] } }]
        })
        .to_string(),
    ));
    commit_study(&fx, "demo", &files);
    let scenario = judged_scenario(Arc::new(judge));
    let invocations = scenario.invocations.clone();
    let athena = fx.athena(scenario.script());
    let steps = RealSteps {
        athena: &athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &RecordingLander::default(),
    };
    let check = steps.check_page();
    assert!(!check.passed);
    assert!(check.detail.contains("places-held"), "{}", check.detail);
    assert!(
        invocations.lock().unwrap().is_empty(),
        "the floor came first"
    );
    assert_eq!(check.cost, 0.0);
}
