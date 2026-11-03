//! `athena focus`, the focus line of `athena status`, and the priority caps of `athena plan`.

mod common;

use athena_agent::fake::Script;
use athena_cli::display;
use athena_cli::drain::{self, Landing, RealSteps};
use athena_core::audience::Page;
use athena_core::plan::Priority;
use athena_core::store::Records;
use athena_core::{Athena, Quiet};
use athena_data::records::{AnyRecord, check_record_file};
use common::*;
use std::process::Command;

fn athena_bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_athena"));
    cmd.env_remove("ATHENA_NESTED")
        .env_remove("ATHENA_SCRATCH_DIR");
    cmd
}

/// Runs the real binary in the fixture's repository; returns (success, stdout, stderr).
fn run(fx: &Fixture, args: &[&str]) -> (bool, String, String) {
    let out = athena_bin()
        .current_dir(fx.root())
        .env("ATHENA_STATE_DIR", fx.state.path())
        .args(args)
        .output()
        .unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn run_ok(fx: &Fixture, args: &[&str]) -> String {
    let (ok, stdout, stderr) = run(fx, args);
    assert!(ok, "athena {args:?} failed: {stderr}");
    stdout
}

fn status_focus_lines(status: &str) -> Vec<&str> {
    status.lines().filter(|l| l.starts_with("focus ")).collect()
}

#[test]
fn focus_is_recorded_printed_and_shown_in_status() {
    let fx = Fixture::new();
    assert!(
        run_ok(&fx, &["focus"]).contains("no focus is set"),
        "before any focus is set, `athena focus` says so"
    );
    assert!(status_focus_lines(&run_ok(&fx, &["status"])).is_empty());

    let printed = run_ok(&fx, &["focus", "speed first"]);
    assert_eq!(printed.trim(), "speed first");

    // A focus record file is committed in the new focus records directory.
    let bytes = fx
        .git
        .show_file(&fx.head(), "records/focus/focus-1.json")
        .unwrap()
        .expect("records/focus/focus-1.json is committed");
    let Some(AnyRecord::Focus(record)) =
        check_record_file("records/focus/focus-1.json", &bytes).unwrap()
    else {
        panic!("the file is a focus record");
    };
    assert_eq!((record.n, record.text.as_str()), (1, "speed first"));
    assert_eq!(record.set_by, "Test User");

    assert_eq!(run_ok(&fx, &["focus"]).trim(), "speed first");
    let status = run_ok(&fx, &["status"]);
    let lines = status_focus_lines(&status);
    assert_eq!(lines.len(), 1, "{status}");
    assert!(lines[0].contains("speed first"), "{status}");
    let json: serde_json::Value =
        serde_json::from_str(&run_ok(&fx, &["status", "--json"])).unwrap();
    assert_eq!(json["focus"], "speed first");
}

#[test]
fn only_the_latest_focus_is_printed_and_shown() {
    let fx = Fixture::new();
    run_ok(&fx, &["focus", "first thing"]);
    run_ok(&fx, &["focus", "speed first"]);
    // Both records are kept; the first is never rewritten.
    for path in ["records/focus/focus-1.json", "records/focus/focus-2.json"] {
        assert!(
            fx.git.show_file(&fx.head(), path).unwrap().is_some(),
            "{path}"
        );
    }
    let printed = run_ok(&fx, &["focus"]);
    assert_eq!(printed.trim(), "speed first");
    assert!(!printed.contains("first thing"));
    let status = run_ok(&fx, &["status"]);
    assert!(status.contains("speed first"), "{status}");
    assert!(!status.contains("first thing"), "{status}");
}

#[test]
fn a_blank_focus_or_a_blank_git_user_name_is_refused_and_writes_nothing() {
    let fx = Fixture::new();
    let head = fx.head();
    let (ok, _, stderr) = run(&fx, &["focus", "   "]);
    assert!(!ok);
    assert!(stderr.contains("focus is empty"), "{stderr}");
    assert_eq!(fx.head(), head);

    fx.git.run(["config", "user.name", ""]).unwrap();
    let (ok, _, stderr) = run(&fx, &["focus", "speed first"]);
    assert!(!ok);
    assert!(stderr.contains("git user.name is not set"), "{stderr}");
    assert_eq!(fx.head(), head, "a refused focus commits nothing");
}

// ---------------------------------------------------------------------------------------------------------
// The priority caps

struct StubPage;

impl Page for StubPage {
    fn fetch(&self, _: &str) -> anyhow::Result<String> {
        Ok("<h1>Start here</h1>".into())
    }
}

/// Planning lands nothing.
struct NoLander;

impl drain::Lander for NoLander {
    fn land(&self, _: &Athena, _: &str, _: &dyn athena_core::Ui) -> Landing {
        panic!("planning lands nothing");
    }
}

fn deterministic() -> Script {
    Box::new(athena_agent::fake::deterministic_turn)
}

/// `count` queued requests, oldest first, each saying `Priority: <level>` in its own words.
fn queue_requests(athena: &Athena, count: usize, level: &str) -> Vec<String> {
    (1..=count)
        .map(|i| {
            athena
                .queue(&format!("Tidy note {i} in docs.\nPriority: {level}"))
                .unwrap()
                .0
                .as_str()
                .to_string()
        })
        .collect()
}

fn plan_text(athena: &Athena) -> (drain::PlanReport, String) {
    let steps = RealSteps {
        athena,
        ui: &Quiet,
        page: &StubPage,
        lander: &NoLander,
    };
    let report = drain::plan(&steps, 50.0, &Quiet).unwrap();
    let text = display::plan(&report);
    (report, text)
}

fn demotion_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|l| l.starts_with("note        demoted"))
        .collect()
}

#[test]
fn five_urgent_requests_keep_three_urgent_with_the_one_a_person_set_among_them() {
    let fx = Fixture::new();
    let athena = fx.athena(deterministic());
    let jobs = queue_requests(&athena, 5, "urgent");
    // The newest request is the one a person gave a priority record.
    athena.set_priority(&jobs[4], "urgent").unwrap();

    let (report, text) = plan_text(&athena);
    let kept: std::collections::BTreeSet<&str> =
        report.plan.order[..3].iter().map(String::as_str).collect();
    assert_eq!(
        kept,
        [jobs[0].as_str(), jobs[1].as_str(), jobs[4].as_str()].into(),
        "{:?}",
        report.plan.order
    );
    let demoted: Vec<&str> = report.plan.demoted.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(demoted, [jobs[2].as_str(), jobs[3].as_str()]);
    assert!(
        report
            .plan
            .demoted
            .iter()
            .all(|d| d.from == Priority::Urgent && d.to == Priority::High)
    );
    assert_eq!(report.plan.order[3..].len(), 2);

    // One line for each demoted request, and none for a kept one.
    let lines = demotion_lines(&text);
    assert_eq!(lines.len(), 2, "{text}");
    for (line, job) in lines.iter().zip([&jobs[2], &jobs[3]]) {
        assert!(line.contains(job.as_str()), "{line}");
        assert!(line.contains("from urgent to high"), "{line}");
    }
    assert!(!text.contains("contradicts priority"), "{text}");
}

#[test]
fn twelve_high_requests_keep_ten_a_priority_record_still_wins_and_no_record_is_rewritten() {
    let fx = Fixture::new();
    let athena = fx.athena(deterministic());
    let mut jobs = queue_requests(&athena, 11, "high");
    // The newest request says low in its words; the person's priority record says high and wins over them.
    let last = athena
        .queue("Tidy note 12 in docs.\nPriority: low")
        .unwrap()
        .0
        .as_str()
        .to_string();
    athena.set_priority(&last, "high").unwrap();
    jobs.push(last.clone());

    let before = Records::load(&fx.git, &fx.head()).unwrap();
    let head_before = fx.head();
    let (report, text) = plan_text(&athena);

    let demoted: Vec<&str> = report.plan.demoted.iter().map(|d| d.id.as_str()).collect();
    assert_eq!(demoted, [jobs[9].as_str(), jobs[10].as_str()]);
    assert!(
        report
            .plan
            .demoted
            .iter()
            .all(|d| d.from == Priority::High && d.to == Priority::Normal && d.cap == 10)
    );
    assert!(
        !demoted.contains(&last.as_str()),
        "the person's record wins"
    );
    assert_eq!(demotion_lines(&text).len(), 2, "{text}");
    // The person-set item is still ahead of the two demoted ones.
    let at = |id: &str| report.plan.position(id).unwrap();
    assert!(at(&last) < at(&jobs[9]) && at(&last) < at(&jobs[10]));

    // Planning may read a request, which adds records; it rewrites no request or priority record, and
    // each job keeps the tier its own records give it.
    let after = Records::load(&fx.git, &fx.head()).unwrap();
    for (id, job) in &before.jobs {
        assert_eq!(after.jobs[id].request, job.request, "{id}");
        assert_eq!(after.jobs[id].priorities, job.priorities, "{id}");
        assert_eq!(after.jobs[id].priority(), job.priority(), "{id}");
    }
    assert_eq!(after.jobs[&last].priority(), Priority::High);
    assert_eq!(after.jobs[&jobs[10]].priority(), Priority::High);
    let changes = fx
        .git
        .run(["diff", "--name-status", &head_before, &fx.head()])
        .unwrap();
    assert!(
        changes.lines().all(|l| l.starts_with("A\t")),
        "planning only adds files:\n{changes}"
    );
}
