//! The steward's triage pass at the start of a drain, through the real drain loop over the real records.
//!
//! Only the model's answer is faked (a script that returns the scripted triage answer, or a failed call).
//! Everything under test runs for real: the brief, the validation of the answer, the person-set guard, the
//! caps, the records written and committed, and the lines the drain prints. Reading, running and landing a
//! job are stubbed, as in the replay of `tests/drain.rs`, so a drain is quick and the records change only
//! through triage.

mod common;

use athena_cli::drain::{
    self, Approval, Caps, Check, Lander, Landing, Limits, QueueView, Rate, Read, ReadOf, RealSteps,
    Report, Run, Steps, Stop,
};
use athena_core::Athena;
use athena_core::audience::Page;
use athena_core::plan::{JobEnd, PlanEntry, TierCaps};
use athena_core::store::{JobState, Records};
use athena_data::records::{AnyRecord, OutcomeState, WorkItemKind, check_record_file};
use common::*;
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};

struct StubPage;

impl Page for StubPage {
    fn fetch(&self, _: &str) -> anyhow::Result<String> {
        Ok("<h1>Start here</h1>".into())
    }
}

struct NoLander;

impl Lander for NoLander {
    fn land(&self, _: &Athena, _: &str, _: &dyn athena_core::Ui) -> Landing {
        unreachable!("the stubbed land step is used")
    }
}

/// The real queue, plan, priority caps and triage over real records, with the review passes replaced by one
/// fixed line and every read, run and landing faked. `started` is the jobs run, in order;
/// `triage_lines_at_first_run` is how many triage lines had been printed when the first job started.
struct World<'a> {
    real: RealSteps<'a>,
    lines: &'a Lines,
    started: RefCell<Vec<String>>,
    triage_lines_at_first_run: Cell<Option<usize>>,
}

fn triage_lines(lines: &Lines) -> Vec<String> {
    lines
        .0
        .lock()
        .unwrap()
        .iter()
        .filter(|l| l.starts_with("triage"))
        .cloned()
        .collect()
}

impl Steps for World<'_> {
    fn review_passes(&self) -> Vec<String> {
        vec!["backfill: nothing to do".into()]
    }
    fn caps(&self) -> anyhow::Result<Caps> {
        self.real.caps()
    }
    fn queue(&self) -> anyhow::Result<QueueView> {
        self.real.queue()
    }
    fn previous_rating(&self) -> Option<u8> {
        None
    }
    fn read(&self, what: ReadOf<'_>) -> Read {
        let (ReadOf::Queued(job) | ReadOf::Retry(job)) = what;
        Read {
            job: Some(job.to_string()),
            cost: 0.0,
            runnable: true,
            error: None,
            held: false,
        }
    }
    fn approve(&self, _: &str) -> Approval {
        unreachable!("these queues have no proposal to approve")
    }
    fn run(&self, job: &str) -> Run {
        if self.triage_lines_at_first_run.get().is_none() {
            self.triage_lines_at_first_run
                .set(Some(triage_lines(self.lines).len()));
        }
        self.started.borrow_mut().push(job.to_string());
        Run {
            outcome: Some(OutcomeState::Landed),
            cost: 0.0,
            ..Run::default()
        }
    }
    fn land(&self, _: &str) -> Landing {
        Landing::Landed {
            changed: Vec::new(),
        }
    }
    fn rate(&self) -> Rate {
        Rate {
            rating: Some(1),
            cost: 0.0,
            error: None,
        }
    }
    fn check_page(&self) -> Check {
        Check::pass("stub")
    }
    fn check_cap(&self) -> f64 {
        0.0
    }
    fn plan_entry(&self, job: &str) -> Option<PlanEntry> {
        self.real.plan_entry(job)
    }
    fn job_history(&self) -> Vec<JobEnd> {
        self.real.job_history()
    }
    fn priority_caps(&self) -> TierCaps {
        self.real.priority_caps()
    }
    fn person_set(&self) -> Vec<String> {
        self.real.person_set()
    }
    fn triage(&self) -> Vec<String> {
        self.real.triage()
    }
}

/// What one drain did.
struct Drained {
    report: Report,
    started: Vec<String>,
    triage_lines_at_first_run: Option<usize>,
    person_set: Vec<String>,
}

fn drain_once(athena: &Athena, lines: &Lines) -> Drained {
    let steps = World {
        real: RealSteps {
            athena,
            ui: lines,
            page: &StubPage,
            lander: &NoLander,
        },
        lines,
        started: RefCell::new(Vec::new()),
        triage_lines_at_first_run: Cell::new(None),
    };
    let limits = Limits {
        budget_usd: 100.0,
        target_rating: 4,
        max_jobs: 20,
    };
    let report = drain::drain(&steps, limits, lines).unwrap();
    Drained {
        report,
        started: steps.started.borrow().clone(),
        triage_lines_at_first_run: steps.triage_lines_at_first_run.get(),
        person_set: steps.person_set(),
    }
}

/// Queues `n` requests that are complete as written (a path and an acceptance bullet), so nothing is read.
fn queue_requests(athena: &Athena, n: usize) -> Vec<String> {
    (0..n)
        .map(|i| {
            let path = format!("crates/r{i}/lib.rs");
            let text = format!("Edit {path} and README.md\n- {path} is edited");
            athena.queue(&text).unwrap().0.as_str().to_string()
        })
        .collect()
}

fn records(fx: &Fixture) -> Records {
    Records::load(&fx.git, &fx.head()).unwrap()
}

fn change(item: &str, priority: &str, reason: &str) -> Value {
    json!({ "item": item, "priority": priority, "reason": reason })
}

fn new_calls() -> Arc<Mutex<Vec<String>>> {
    Arc::new(Mutex::new(Vec::new()))
}

/// Every priority record of `job`, read back from its committed file and checked as `athena check` would.
fn priority_files(fx: &Fixture, job: &str) -> Vec<athena_data::records::PriorityRecord> {
    let head = fx.head();
    let loaded = records(fx);
    let mut found = Vec::new();
    for p in &loaded.jobs[job].priorities {
        let path = AnyRecord::Priority(p.clone()).canonical_path();
        let bytes = fx
            .git
            .show_file(&head, &path)
            .unwrap()
            .unwrap_or_else(|| panic!("{path} is committed"));
        match check_record_file(&path, &bytes).unwrap() {
            Some(AnyRecord::Priority(read)) => found.push(read),
            other => panic!("{path} is not a priority record: {other:?}"),
        }
    }
    found
}

#[test]
fn triage_writes_steward_priority_records_within_caps_and_leaves_person_set_items() {
    let fx = Fixture::new();
    let calls = new_calls();
    // The ids exist only after the requests are queued, so the answer is built from them.
    let ids = {
        let athena = fx.athena(triage_script(None, new_calls()));
        queue_requests(&athena, 6)
    };
    let athena = fx.athena(triage_script(
        Some(json!({
            "changes": [
                change(&ids[0], "urgent", "serves the focus: speed first"),
                change(&ids[1], "urgent", "serves the focus: faster drains"),
                change(&ids[2], "urgent", "repeated docs failures"),
                // The fourth urgent change is over the cap of three.
                change(&ids[3], "urgent", "also on the focus"),
                // A person set this one low; triage may not touch it.
                change(&ids[5], "urgent", "tries to override a person"),
                change(&ids[4], "high", "close to the focus"),
            ],
            "close": []
        })),
        calls.clone(),
    ));
    athena.set_focus("speed first").unwrap();
    athena.set_priority(&ids[5], "low").unwrap();

    let lines = Lines::default();
    let drained = drain_once(&athena, &lines);
    assert_eq!(drained.report.stop, Stop::QueueEmpty);

    let after = records(&fx);
    let priority_of = |i: usize| after.jobs[&ids[i]].priority().as_str();
    // The first three urgent changes and the high change are applied; the fourth urgent change is not.
    assert_eq!(priority_of(0), "urgent");
    assert_eq!(priority_of(1), "urgent");
    assert_eq!(priority_of(2), "urgent");
    assert_eq!(priority_of(3), "normal", "over the cap of three urgent");
    assert_eq!(priority_of(4), "high");
    let urgent = (0..6).filter(|i| priority_of(*i) == "urgent").count();
    assert!(urgent <= 3, "{urgent} urgent items");
    assert!(
        priority_files(&fx, &ids[3]).is_empty(),
        "a change over a cap writes no record"
    );

    // The records are the steward's, with the model's reason, and each passes the record check.
    for (i, level, reason) in [
        (0, "urgent", "serves the focus: speed first"),
        (1, "urgent", "serves the focus: faster drains"),
        (2, "urgent", "repeated docs failures"),
        (4, "high", "close to the focus"),
    ] {
        let files = priority_files(&fx, &ids[i]);
        assert_eq!(files.len(), 1, "{files:?}");
        assert_eq!(files[0].set_by.as_deref(), Some("steward"));
        assert_eq!(files[0].reason.as_deref(), Some(reason));
        assert_eq!(files[0].priority, level);
    }

    // The person's item keeps exactly its one record, and is still the person's.
    let person = priority_files(&fx, &ids[5]);
    assert_eq!(person.len(), 1, "{person:?}");
    assert!(person[0].is_by_person());
    assert_eq!(person[0].priority, "low");
    assert_eq!(priority_of(5), "low");
    assert_eq!(
        drained.person_set,
        [ids[5].clone()],
        "only a person's word is protected from the plan"
    );

    // One line per applied change, printed before any job started.
    let printed = triage_lines(&lines);
    assert_eq!(printed.len(), 4, "{printed:?}");
    assert_eq!(drained.triage_lines_at_first_run, Some(4));
    assert!(
        printed[0].contains(&ids[0])
            && printed[0].contains("normal -> urgent")
            && printed[0].contains("serves the focus: speed first"),
        "{printed:?}"
    );
    assert!(printed[3].contains("normal -> high"), "{printed:?}");

    // One model call, and it was given the focus and the items.
    let prompts = calls.lock().unwrap();
    assert_eq!(prompts.len(), 1, "one model call for the whole pass");
    assert!(prompts[0].contains("speed first"));
    assert!(prompts[0].contains(&ids[0]) && prompts[0].contains("(set by a person)"));
    // The drain went on and worked all six, urgent ones first.
    assert_eq!(drained.started.len(), 6);
    assert!(drained.started[..3].iter().all(|j| ids[..3].contains(j)));
}

#[test]
fn a_failed_triage_call_prints_a_note_and_the_drain_goes_on() {
    let fx = Fixture::new();
    let calls = new_calls();
    let athena = fx.athena(triage_script(None, calls.clone()));
    let ids = queue_requests(&athena, 2);

    let lines = Lines::default();
    let drained = drain_once(&athena, &lines);

    let printed = lines.0.lock().unwrap().clone();
    let failed: Vec<&String> = printed
        .iter()
        .filter(|l| l.starts_with("triage failed"))
        .collect();
    assert_eq!(failed.len(), 1, "{printed:?}");
    assert!(failed[0].contains("the drain goes on"), "{failed:?}");
    // The other step's line is still printed, before the note.
    let review = printed
        .iter()
        .position(|l| l.starts_with("backfill"))
        .unwrap();
    let note = printed
        .iter()
        .position(|l| l.starts_with("triage failed"))
        .unwrap();
    assert!(review < note, "{printed:?}");
    // The drain worked both items and ended on an empty queue.
    assert_eq!(calls.lock().unwrap().len(), 1);
    assert_eq!(drained.report.stop, Stop::QueueEmpty);
    let mut started = drained.started.clone();
    started.sort();
    let mut expected = ids.clone();
    expected.sort();
    assert_eq!(started, expected);
    // Nothing was written for the items.
    let after = records(&fx);
    assert!(ids.iter().all(|id| after.jobs[id].priorities.is_empty()));
}

#[test]
fn an_unusable_triage_answer_is_a_note_too() {
    let fx = Fixture::new();
    let athena = fx.athena(triage_script(Some(json!("not an object")), new_calls()));
    let ids = queue_requests(&athena, 1);
    let lines = Lines::default();
    let drained = drain_once(&athena, &lines);
    assert_eq!(
        triage_lines(&lines).len(),
        1,
        "{:?}",
        lines.0.lock().unwrap()
    );
    assert!(triage_lines(&lines)[0].starts_with("triage failed"));
    assert_eq!(drained.started, ids);
}

#[test]
fn the_drain_prints_one_line_per_applied_change() {
    let fx = Fixture::new();
    let ids = {
        let athena = fx.athena(triage_script(None, new_calls()));
        queue_requests(&athena, 3)
    };
    let athena = fx.athena(triage_script(
        Some(json!({
            "changes": [
                change(&ids[0], "high", "on the focus"),
                change(&ids[1], "low", "off the focus"),
                // No change: ignored, and so no line.
                change(&ids[2], "normal", "already normal"),
                change("20250101-000000-nope", "urgent", "not an open item"),
            ],
            "close": []
        })),
        new_calls(),
    ));
    let lines = Lines::default();
    let drained = drain_once(&athena, &lines);

    let printed = triage_lines(&lines);
    assert_eq!(printed.len(), 2, "{printed:?}");
    assert!(printed[0].contains(&ids[0]) && printed[0].ends_with("normal -> high: on the focus"));
    assert!(printed[1].contains(&ids[1]) && printed[1].ends_with("normal -> low: off the focus"));
    assert_eq!(drained.triage_lines_at_first_run, Some(2));
    // Low runs last.
    assert_eq!(drained.started.last(), Some(&ids[1]));
}

#[test]
fn a_close_proposal_becomes_a_steward_work_item_and_never_a_close() {
    let fx = Fixture::new();
    let ids = {
        let athena = fx.athena(triage_script(None, new_calls()));
        queue_requests(&athena, 2)
    };
    let answer = Some(json!({
        "changes": [change(&ids[1], "high", "on the focus")],
        "close": [{ "item": ids[0], "reason": "off the focus and never started" }]
    }));
    let athena = fx.athena(triage_script(answer.clone(), new_calls()));

    let lines = Lines::default();
    let drained = drain_once(&athena, &lines);

    let after = records(&fx);
    let key = format!("triage-close:{}", ids[0]);
    let proposals: Vec<_> = after.work_items.iter().filter(|w| w.key == key).collect();
    assert_eq!(proposals.len(), 1, "{:?}", after.work_items);
    let work = proposals[0];
    assert_eq!(work.role, "steward");
    assert_eq!(work.kind, WorkItemKind::Proposal);
    assert!(work.title.contains("off the focus and never started"));
    assert_eq!(work.evidence.len(), 1);
    assert_eq!(work.evidence[0].value, ids[0]);
    // Nothing was closed: no outcome record, still queued, and the drain still worked it.
    let job = &after.jobs[&ids[0]];
    assert!(job.outcome.is_none() && job.later_outcome.is_none());
    assert_eq!(job.state(), JobState::Queued);
    assert!(drained.started.contains(&ids[0]), "{:?}", drained.started);
    // The priority change in the same answer was applied.
    assert_eq!(after.jobs[&ids[1]].priority().as_str(), "high");
    // One line for the change and one for the proposal.
    assert_eq!(triage_lines(&lines).len(), 2, "{:?}", triage_lines(&lines));

    // A second drain with the same answer proposes nothing again.
    let athena = fx.athena(triage_script(answer, new_calls()));
    let again = Lines::default();
    drain_once(&athena, &again);
    let after = records(&fx);
    assert_eq!(after.work_items.iter().filter(|w| w.key == key).count(), 1);
    assert!(
        triage_lines(&again).is_empty(),
        "{:?}",
        triage_lines(&again)
    );
}

#[test]
fn an_empty_queue_makes_no_triage_call() {
    let fx = Fixture::new();
    let calls = new_calls();
    let athena = fx.athena(triage_script(
        Some(json!({ "changes": [], "close": [] })),
        calls.clone(),
    ));
    let lines = Lines::default();
    let drained = drain_once(&athena, &lines);
    assert!(calls.lock().unwrap().is_empty());
    assert!(drained.started.is_empty());
    assert!(triage_lines(&lines).is_empty());
}

#[test]
fn a_steward_set_priority_is_not_protected_from_the_plan_as_a_persons_is() {
    let fx = Fixture::new();
    let athena = fx.athena(triage_script(None, new_calls()));
    let ids = queue_requests(&athena, 5);
    // Four urgent records by the steward, and one by a person.
    let steward = |job: &str| {
        let id = athena_data::JobId::parse(job).unwrap();
        AnyRecord::Priority(athena_data::records::PriorityRecord {
            job: id,
            n: 1,
            at: "2026-10-30T00:00:00Z".parse().unwrap(),
            priority: "urgent".into(),
            set_by: Some("steward".into()),
            reason: Some("serves the focus".into()),
        })
    };
    for id in &ids[..4] {
        let record = steward(id);
        write(fx.root(), &[(&record.canonical_path(), &record.to_json())]);
    }
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run(["commit", "-q", "-m", "Steward priorities"])
        .unwrap();
    athena.set_priority(&ids[4], "urgent").unwrap();

    let steps = RealSteps {
        athena: &athena,
        ui: &Lines::default(),
        page: &StubPage,
        lander: &NoLander,
    };
    // Only the person's record keeps an item out of the plan's demotion.
    assert_eq!(steps.person_set(), [ids[4].clone()]);
}
