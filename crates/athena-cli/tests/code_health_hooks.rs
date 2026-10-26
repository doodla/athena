//! The code-health check after a landing, through the real `ask` and `go` flow with scripted agents, and the
//! rendered `athena status` table. The disk guard runs with no minimum (`test_config` sets `min_free_gb = 0`).

mod common;

use athena_cli::display;
use athena_core::run::GoResult;
use athena_core::store::Records;
use athena_data::records::{AcceptedBy, OutcomeState};
use common::*;

/// A source file of `lines` production lines.
fn source(lines: usize) -> String {
    (0..lines)
        .map(|i| format!("pub const C{i}: u32 = {i};\n"))
        .collect()
}

/// A fixture whose repository holds a source file over the file-length limit.
fn fixture_with_a_big_file() -> Fixture {
    let fx = Fixture::new();
    write(fx.root(), &[("crates/x/src/big.rs", &source(2001))]);
    fx.git.run(["add", "-A"]).unwrap();
    fx.git
        .run([
            "commit",
            "-q",
            "-m",
            "Add a large file\n\nAthena-Job: bootstrap",
        ])
        .unwrap();
    fx
}

fn run_job(athena: &athena_core::Athena, ui: &Lines) -> OutcomeState {
    let proposal = athena.ask("change the readme", ui).unwrap();
    match athena
        .go(proposal.job.as_str(), AcceptedBy::User, ui)
        .unwrap()
    {
        GoResult::Finished(f) => {
            if f.state != OutcomeState::Landed && f.state != OutcomeState::Rejected {
                panic!("the job ended {:?}: {}", f.state, f.reason);
            }
            f.state
        }
        GoResult::Reread(_) => panic!("expected the job to finish"),
    }
}

fn structural_requests(fx: &Fixture) -> Vec<String> {
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    records
        .request_jobs()
        .iter()
        .filter_map(|j| j.request.as_ref().map(|r| r.text.clone()))
        .filter(|t| t.starts_with("Structural pass: "))
        .collect()
}

fn code_health_lines(ui: &Lines) -> Vec<String> {
    ui.0.lock()
        .unwrap()
        .iter()
        .filter(|l| l.starts_with("code health"))
        .cloned()
        .collect()
}

#[test]
fn a_landed_job_is_followed_by_one_structural_pass_request_for_the_oversized_file() {
    let fx = fixture_with_a_big_file();
    let athena = fx.athena(Scenario::default().script());
    let ui = Lines::default();
    assert!(structural_requests(&fx).is_empty());

    let state = run_job(&athena, &ui);
    assert_eq!(state, OutcomeState::Landed, "{:?}", ui.0.lock().unwrap());

    let requests = structural_requests(&fx);
    assert_eq!(requests.len(), 1, "{requests:?}");
    assert!(
        requests[0].starts_with("Structural pass: crates/x/src/big.rs\n"),
        "{}",
        requests[0]
    );
    assert!(
        requests[0].contains("2001 production lines"),
        "{}",
        requests[0]
    );
    assert!(
        !code_health_lines(&ui).is_empty(),
        "the check said what it did"
    );

    // The first landing set the baseline and fired no review.
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    assert_eq!(records.code_health_baselines.len(), 1);
    assert!(!records.code_health_baselines[0].review_queued);

    // A second landing queues no second request while the first is open.
    let again = Scenario {
        edits: vec![("README.md".into(), Some("# Demo\n\nHello again.\n".into()))],
        ..Scenario::default()
    };
    let athena = fx.athena(again.script());
    assert_eq!(run_job(&athena, &ui), OutcomeState::Landed);
    assert_eq!(structural_requests(&fx).len(), 1);
}

#[test]
fn a_rejected_job_queues_nothing() {
    let fx = fixture_with_a_big_file();
    // Deleting the README fails the test check, so the job is rejected.
    let scenario = Scenario {
        edits: vec![("README.md".into(), None)],
        ..Scenario::default()
    };
    let athena = fx.athena(scenario.script());
    let ui = Lines::default();

    assert_eq!(run_job(&athena, &ui), OutcomeState::Rejected);

    assert!(structural_requests(&fx).is_empty());
    assert!(
        code_health_lines(&ui).is_empty(),
        "{:?}",
        ui.0.lock().unwrap()
    );
    let records = Records::load(&fx.git, &fx.head()).unwrap();
    assert!(records.code_health_baselines.is_empty());
}

#[test]
fn status_shows_the_code_health_table_with_the_oversized_file_marked() {
    let fx = fixture_with_a_big_file();
    let athena = fx.athena(Scenario::default().script());
    let text = display::status(&athena.status().unwrap());

    let header = text
        .lines()
        .find(|l| l.contains("FILE"))
        .expect("a table header");
    for name in [
        "LINES",
        "LONGEST FN",
        "LANDINGS 7D",
        "CITED",
        "LIMITS",
        "OVER",
    ] {
        assert!(header.contains(name), "{header}");
    }
    let big = text
        .lines()
        .find(|l| l.contains("crates/x/src/big.rs"))
        .unwrap();
    let small = text.lines().find(|l| l.contains("src/lib.rs")).unwrap();
    assert_eq!(header.find("LINES"), big.find("2001"), "{text}");
    assert_eq!(header.find("OVER"), big.find("yes: file length"), "{text}");
    assert_eq!(header.find("OVER"), small.find("no"), "{text}");
    assert!(text.contains("2000 production lines per file"), "{text}");
    assert!(text.contains("test suite run time: unknown"), "{text}");
}
