//! The D87/D88 backtest. Every records/ fixture is written into a temporary directory by the test that uses
//! it and removed afterwards; no fixture file is committed.

use athena_cli::backtest::{self, HitKind, Status};
use athena_cli::decisions::{self, ConfirmedLinks, LinkStatus};
use serde_json::json;
use std::path::Path;

/// A decisions log with exactly the four supersession hits the review expects, and some "Amends" entries.
const LOG: &str = "# Decisions\n\
An entry is never rewritten: a new entry supersedes it and names the entry.\n\
\n\
## D46 · Cache\n\
- Tradeoff: cold builds. (Superseded by D79: the exclusion no longer holds.)\n\
\n\
## D79 · Shared cache\n\
- **Decision:** This supersedes D46's exclusion of the cache.\n\
\n\
## D80 · Brief words\n\
- **Decision:** Amends D62 and D43. Text only.\n\
\n\
## D87 · Merge commits\n\
- **Superseded by D88** for the landing shape. The rerun fixes stand.\n\
\n\
## D88 · Follow the repository\n\
- **Decision:** Supersedes D87's \"pull requests land as merge commits\".\n";

fn write(root: &Path, relative: &str, value: serde_json::Value) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, serde_json::to_vec_pretty(&value).unwrap()).unwrap();
}

fn outcome(root: &Path, job: &str, state: &str) {
    write(
        root,
        &format!("records/jobs/2026-10/{job}/outcome.json"),
        json!({ "schema": "athena.outcome/1", "job": job, "state": state, "reason": "test" }),
    );
}

fn intervention(root: &Path, id: &str, job: Option<&str>, category: &str, class: &str) {
    write(
        root,
        &format!("records/interventions/2026-10/{id}.json"),
        json!({
            "schema": "athena.intervention/1",
            "id": id,
            "at": format!("2026-10-{}T00:00:00Z", &id[6..8]),
            "what": "merged by hand",
            "why": "the host refused",
            "class_of_work": class,
            "category": category,
            "job": job,
            "source": "user",
            "with_staged_changes": false,
        }),
    );
}

/// The confirmed list for `LOG`: every hit, with D87 amended and not superseded.
const LINKS: &str = "\
[[link]]\nline = 5\nolder = \"D46\"\nnewer = \"D79\"\nstatus = \"superseded\"\n\
[[link]]\nline = 8\nolder = \"D46\"\nnewer = \"D79\"\nstatus = \"superseded\"\n\
[[link]]\nline = 11\nolder = \"D62\"\nnewer = \"D80\"\nstatus = \"amended\"\n\
[[link]]\nline = 11\nolder = \"D43\"\nnewer = \"D80\"\nstatus = \"amended\"\n\
[[link]]\nline = 14\nolder = \"D87\"\nnewer = \"D88\"\nstatus = \"amended\"\n\
[[link]]\nline = 17\nolder = \"D87\"\nnewer = \"D88\"\nstatus = \"amended\"\n";

fn links() -> ConfirmedLinks {
    ConfirmedLinks::parse(LINKS).unwrap()
}

fn run_in(root: &Path) -> backtest::Report {
    backtest::run_on(LOG, &links(), &backtest::read_records(root).unwrap())
}

fn real_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn the_parse_returns_the_four_known_hits_and_the_amends() {
    let parsed = backtest::parse_supersessions(LOG);
    let supersessions: Vec<(&str, &str, usize)> = parsed
        .hits
        .iter()
        .filter(|h| h.kind != HitKind::Amends)
        .map(|h| (h.older.as_str(), h.newer.as_str(), h.line))
        .collect();
    assert_eq!(
        supersessions,
        [
            ("D46", "D79", 5),
            ("D46", "D79", 8),
            ("D87", "D88", 14),
            ("D87", "D88", 17)
        ]
    );
    let amended: Vec<&str> = parsed
        .hits
        .iter()
        .filter(|h| h.kind == HitKind::Amends)
        .map(|h| h.older.as_str())
        .collect();
    assert_eq!(amended, ["D62", "D43"]);
    assert!(parsed.lacking_target.is_empty());
    assert_eq!(parsed.entries.len(), 5);
    let standing = backtest::standing(&parsed.hits);
    assert!(standing["D87"].superseded_by.contains("D88"));
    assert!(standing["D62"].amended_by.contains("D80"));
}

#[test]
fn the_real_log_has_no_condition_a_finding_against_the_confirmed_list() {
    let root = real_root();
    let text = std::fs::read_to_string(root.join("docs/decisions.md")).unwrap();
    let confirmed = ConfirmedLinks::load(&root).unwrap();
    let report = backtest::run_on(&text, &confirmed, &backtest::Records::default());
    let a: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.condition == "a")
        .collect();
    assert!(a.is_empty(), "{a:?}");
    assert!(
        report.lacking_target.is_empty(),
        "{:?}",
        report.lacking_target
    );
    assert_eq!(report.hits.len(), confirmed.links.len());
    assert_eq!(report.decision_view.len(), report.entries);
}

#[test]
fn the_confirmed_list_has_the_known_hits_and_the_view_follows_it() {
    let root = real_root();
    let text = std::fs::read_to_string(root.join("docs/decisions.md")).unwrap();
    let confirmed = ConfirmedLinks::load(&root).unwrap();
    let has = |line, older, newer, status| {
        confirmed
            .links
            .iter()
            .any(|l| l.line == line && l.older == older && l.newer == newer && l.status == status)
    };
    assert!(has(322, "D11", "D39", LinkStatus::Amended));
    assert!(has(1028, "D46", "D79", LinkStatus::Superseded));
    assert!(has(1760, "D46", "D79", LinkStatus::Superseded));
    assert!(has(1909, "D87", "D88", LinkStatus::Amended));
    assert!(has(1930, "D87", "D88", LinkStatus::Amended));
    assert!(confirmed.links.iter().any(|l| l.newer == "D48"));

    let parsed = backtest::parse_supersessions(&text);
    let views = decisions::view(&parsed, &confirmed);
    let get = |id: &str| views.iter().find(|v| v.id == id).unwrap();
    assert_eq!(get("D11").status, Status::Active);
    assert_eq!(get("D11").superseded_by, None);
    assert_eq!(get("D11").amended_by, ["D39"]);
    assert_eq!(get("D46").status, Status::Superseded);
    assert_eq!(get("D46").superseded_by.as_deref(), Some("D79"));
    assert_eq!(get("D87").status, Status::Active);
    assert_eq!(get("D87").amended_by, ["D88"]);
    assert_eq!(views.first().unwrap().id, "D01");
    assert!(views.iter().any(|v| v.id == "D90"));
}

#[test]
fn two_hand_changes_on_two_landed_jobs_pass_and_name_d87_in_one_introspection() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    outcome(root, "20261001-000000-aaaa", "landed");
    outcome(root, "20261002-000000-bbbb", "landed");
    outcome(root, "20261003-000000-cccc", "rejected");
    intervention(
        root,
        "20261002-000000-i002",
        Some("20261002-000000-bbbb"),
        "did-work-themselves",
        "docs",
    );
    intervention(
        root,
        "20261001-000000-i001",
        Some("20261001-000000-aaaa"),
        "did-work-themselves",
        "docs",
    );
    // Not matches: wrong category, wrong class, a job that did not land, and no job.
    intervention(
        root,
        "20261004-000000-i004",
        Some("20261001-000000-aaaa"),
        "corrected-output",
        "docs",
    );
    intervention(
        root,
        "20261005-000000-i005",
        Some("20261001-000000-aaaa"),
        "did-work-themselves",
        "code",
    );
    intervention(
        root,
        "20261006-000000-i006",
        Some("20261003-000000-cccc"),
        "did-work-themselves",
        "docs",
    );
    intervention(
        root,
        "20261007-000000-i007",
        None,
        "did-work-themselves",
        "docs",
    );

    let report = run_in(root);
    assert!(report.passed(), "{:?}", report.findings);
    assert_eq!(report.under_review, ["D87"]);
    let record = report.introspection.expect("one introspection record");
    assert_eq!(record.schema, "athena.introspection/1");
    assert_eq!(record.intervention, "20261001-000000-i001");
    assert_eq!(record.job, "20261001-000000-aaaa");
    assert_eq!(record.decisions_matched, ["D87"]);
    assert!(record.earlier_cases.is_empty());

    let path = root.to_path_buf();
    dir.close().unwrap();
    assert!(!path.exists(), "the temporary records are removed");
}

#[test]
fn one_job_is_a_finding_with_its_reason_and_no_introspection() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    outcome(root, "20261001-000000-aaaa", "landed");
    intervention(
        root,
        "20261001-000000-i001",
        Some("20261001-000000-aaaa"),
        "did-work-themselves",
        "docs",
    );
    // A second matching intervention on the same job is still one job.
    intervention(
        root,
        "20261002-000000-i002",
        Some("20261001-000000-aaaa"),
        "did-work-themselves",
        "docs",
    );

    let report = run_in(root);
    assert!(!report.passed());
    assert!(report.introspection.is_none());
    assert!(report.under_review.is_empty());
    let finding = report.findings.iter().find(|f| f.condition == "b").unwrap();
    assert!(
        finding.reason.contains("1 distinct job"),
        "{}",
        finding.reason
    );
    assert!(finding.reason.contains("F3"), "{}", finding.reason);
    assert!(backtest::render(&report).contains("backtest: fail"));
}

#[test]
fn missing_records_is_a_finding_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let report = run_in(dir.path());
    assert_eq!(report.interventions_read, 0);
    assert!(!report.passed());
}

#[test]
fn a_changed_log_is_a_finding_for_the_parse_condition() {
    let dir = tempfile::tempdir().unwrap();
    let log = format!(
        "{LOG}\n## D89 · More\n- **Decision:** Supersedes D80.\n- **Superseded by** nothing\n"
    );
    let report = backtest::run_on(&log, &links(), &backtest::read_records(dir.path()).unwrap());
    let reasons: Vec<&str> = report
        .findings
        .iter()
        .filter(|f| f.condition == "a")
        .map(|f| f.reason.as_str())
        .collect();
    assert!(
        reasons
            .iter()
            .any(|r| r.contains("hit not in the confirmed list: D80 superseded by D89")),
        "{reasons:?}"
    );
    assert!(
        reasons.iter().any(|r| r.contains("without a target id")),
        "{reasons:?}"
    );
}

#[test]
fn only_d87_and_d88_have_headers_and_only_d87_has_an_exact_signal() {
    let headers = backtest::headers();
    let ids: Vec<&str> = headers.iter().map(|h| h.id).collect();
    assert_eq!(ids, ["D87", "D88"]);
    let exact: Vec<&str> = headers
        .iter()
        .filter(|h| h.signals.iter().any(|s| s.exact.is_some()))
        .map(|h| h.id)
        .collect();
    assert_eq!(exact, ["D87"]);
    let d87 = &headers[0];
    assert_eq!(d87.enforcement, backtest::Enforcement::Brief);
    assert_eq!(d87.amended_by, ["D88"]);
}

fn host_error(root: &Path, id: &str, status: Option<u16>, operation: &str, class: &str) {
    write(
        root,
        &format!("records/host-errors/2026-10/{id}.json"),
        json!({
            "schema": "athena.host-error/1",
            "id": id,
            "at": "2026-10-13T19:21:56Z",
            "status_code": status,
            "operation": operation,
            "job": "20261013-192156-ar1x",
            "phase": "landing",
            "message_class": class,
        }),
    );
}

#[test]
fn a_host_error_record_of_the_405_puts_d87_under_review_and_cites_the_record() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    host_error(
        root,
        "20261013-192200-k7q2",
        Some(405),
        "merge",
        "method-not-allowed",
    );

    let report = run_in(root);
    assert_eq!(report.under_review, ["D87"]);
    let run = report
        .runs
        .iter()
        .find(|r| r.decision == "D87" && !r.host_error_records.is_empty())
        .expect("the host-error signal ran");
    assert!(run.fired);
    assert_eq!(run.host_error_records, ["20261013-192200-k7q2"]);
    let d87 = report
        .decision_view
        .iter()
        .find(|v| v.id == "D87")
        .expect("D87 is in the log");
    assert_eq!(
        d87.under_review.as_deref(),
        Some(&["20261013-192200-k7q2".to_string()][..])
    );
    // No other decision is marked.
    assert_eq!(
        report
            .decision_view
            .iter()
            .filter(|v| v.under_review.is_some())
            .count(),
        1
    );
    // The first occurrence is enough, and condition (b) holds without an intervention.
    assert!(report.passed(), "{:?}", report.findings);
    let text = backtest::render(&report);
    assert!(text.contains("D87 is under review"), "{text}");
    assert!(text.contains("20261013-192200-k7q2"), "{text}");
}

#[test]
fn without_the_host_error_record_d87_is_not_under_review() {
    let dir = tempfile::tempdir().unwrap();
    let report = run_in(dir.path());
    assert!(report.under_review.is_empty());
    assert!(
        report
            .decision_view
            .iter()
            .all(|v| v.under_review.is_none())
    );
    assert!(!backtest::render(&report).contains("under review"));
    assert!(report.findings.iter().any(|f| f.condition == "b"));
}

#[test]
fn host_errors_that_do_not_falsify_the_assumption_leave_d87_alone() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    host_error(
        root,
        "20261013-192201-aaaa",
        Some(404),
        "merge",
        "not-found",
    );
    host_error(
        root,
        "20261013-192202-bbbb",
        Some(500),
        "merge",
        "server-error",
    );
    host_error(root, "20261013-192203-cccc", None, "merge", "no-answer");
    host_error(
        root,
        "20261013-192204-dddd",
        Some(405),
        "create-pr",
        "method-not-allowed",
    );
    let report = run_in(root);
    assert!(report.under_review.is_empty());
    assert!(
        report
            .decision_view
            .iter()
            .all(|v| v.under_review.is_none())
    );
}

#[test]
fn a_malformed_host_error_record_is_an_error_naming_the_file() {
    let dir = tempfile::tempdir().unwrap();
    // The class does not follow from the status.
    host_error(
        dir.path(),
        "20261013-192200-k7q2",
        Some(405),
        "merge",
        "not-found",
    );
    let error = backtest::read_records(dir.path()).unwrap_err();
    assert!(format!("{error:#}").contains("20261013-192200-k7q2.json"));
}

#[test]
fn a_malformed_record_is_an_error_naming_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("records/interventions/2026-10/bad.json");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "{").unwrap();
    let error = backtest::read_records(dir.path()).unwrap_err();
    assert!(format!("{error:#}").contains("bad.json"));
}
