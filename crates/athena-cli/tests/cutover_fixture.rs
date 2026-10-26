//! The shared records-history cutover fixture (`athena_core::test_support`, decision D111): its shape, and the
//! two history checks run on it with the real `athena` binary.
//!
//! The assertions here pin what the fixture is. A later job that reuses it (the trailer job, job 7 part 2)
//! changes the assertion that reads what it changed, and no other.

mod common;

use athena_core::gh::cutover_job_for;
use athena_core::records_import::{MANIFEST_PATH, verify_cutover_import};
use athena_core::store::{JobState, Records};
use athena_data::history::HistoryRecord;
use athena_data::records::{OutcomeState, Verdict};
use athena_gate::attribution::cutover_commit_at;
use athena_gate::invariant::{JOB, VERIFIED_CONTENT};
use athena_gate::legacy::LEGACY_EXEMPT_PATH;
use common::*;

fn trailer(trailers: &[(String, String)], key: &str) -> Option<String> {
    trailers
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
}

#[test]
fn the_history_and_attribution_checks_pass_on_the_fixtures_main() {
    let fx = cutover_fixture();

    let history = athena_check(&fx, "--history");
    let text = String::from_utf8_lossy(&history.stdout).to_string();
    assert!(
        history.status.success(),
        "{text}{}",
        String::from_utf8_lossy(&history.stderr)
    );
    assert!(text.contains("0 violation(s)"), "{text}");

    let attribution = athena_check(&fx, "--attribution");
    let text = String::from_utf8_lossy(&attribution.stdout).to_string();
    assert!(
        attribution.status.success(),
        "{text}{}",
        String::from_utf8_lossy(&attribution.stderr)
    );
    assert!(text.contains("attribution pass"), "{text}");
    assert!(text.contains("0 finding(s)"), "{text}");
}

#[test]
fn the_old_way_job_landed_with_its_trailers_and_was_read_dispatched_and_verified() {
    let fx = cutover_fixture();
    let landing = &fx.old_way_landing;

    // The landing commit descends from the bootstrap commit (the job's own record commits lie between
    // them) and carries the trailers `go` wrote.
    assert!(fx.git.is_ancestor(&fx.bootstrap, landing).unwrap());
    let trailers = fx.trailers_of(landing).unwrap();
    assert_eq!(
        trailer(&trailers, JOB).as_deref(),
        Some(fx.old_way_job.as_str())
    );
    assert!(
        trailer(&trailers, VERIFIED_CONTENT).is_some_and(|v| !v.is_empty()),
        "{trailers:?}"
    );
    // It is on main, and the bootstrap commit is the root.
    let main = fx.git.rev_list_reverse("main").unwrap();
    assert_eq!(main.first(), Some(&fx.bootstrap));
    assert!(main.contains(landing));
    assert!(fx.git.is_ancestor(landing, &fx.c0).unwrap());
    assert_eq!(
        trailer(&fx.trailers_of(&fx.bootstrap).unwrap(), JOB).as_deref(),
        Some("bootstrap")
    );

    // The records at that commit show the job read, dispatched, verified and landed.
    let records = Records::load(&fx.git, landing).unwrap();
    let job = records.jobs.get(fx.old_way_job.as_str()).expect("the job");
    assert!(job.latest_reading().is_some(), "read");
    let candidate = job.candidate.as_ref().expect("dispatched: a candidate");
    assert!(
        fx.git
            .is_ancestor(&fx.bootstrap, &candidate.base_commit)
            .unwrap()
    );
    let verification = job.verification.as_ref().expect("verified");
    assert_eq!(verification.final_verdict, Verdict::Pass);
    assert_eq!(job.state(), JobState::Closed(OutcomeState::Landed));

    // The old way leaves no landing record: the records branch knows nothing of this job.
    let tip = fx.records_tip().unwrap();
    assert_eq!(
        cutover_commit_at(&fx.git, &tip, fx.old_way_job.as_str()).unwrap(),
        None
    );
}

#[test]
fn c0_resolves_from_the_cutover_landing_and_the_import_and_exempt_record_are_on_the_records_branch()
{
    let fx = cutover_fixture();
    let tip = fx.records_tip().unwrap();

    // C0 is the host commit of the cutover job's landing record, and it is main's tip.
    let c0 = cutover_commit_at(&fx.git, &tip, fx.cutover_job.as_str())
        .unwrap()
        .expect("a landing of the cutover job");
    assert_eq!(c0, fx.c0);
    assert_eq!(
        fx.git.rev_parse("main").unwrap().as_deref(),
        Some(c0.as_str())
    );
    // The code-health check wrote its baseline record after the old-way landing, so C0 sits on that record
    // commit, which sits on the old-way landing.
    let parents = fx.git.parents(&c0).unwrap();
    assert_eq!(parents.len(), 1);
    assert!(
        fx.git
            .is_ancestor(&fx.old_way_landing, &parents[0])
            .unwrap()
    );

    // The import is one commit on top of the landing record, whose second parent is the pull request's head.
    let import = tip.clone();
    let landing_record = fx.bare.parents(&import).unwrap();
    assert_eq!(
        landing_record.len(),
        1,
        "the import sits on the landing record"
    );
    let landing_commit = &landing_record[0];
    assert!(
        fx.bare
            .added_paths(landing_commit)
            .unwrap()
            .iter()
            .any(|p| p.starts_with("landings/")),
        "the commit before the import adds the landing record"
    );
    assert_eq!(
        fx.bare.parents(landing_commit).unwrap().last(),
        Some(&fx.cutover_local_landing)
    );

    // The import's self-check passes against C0, and the manifest names C0.
    verify_cutover_import(&fx.git, &c0, &tip).unwrap();
    let manifest = fx.bare.show_file(&import, MANIFEST_PATH).unwrap().unwrap();
    match HistoryRecord::parse(&String::from_utf8(manifest).unwrap()).unwrap() {
        HistoryRecord::ImportManifest(m) => {
            assert_eq!(m.c0, c0);
            assert!(!m.paths.is_empty());
        }
        other => panic!("{other:?}"),
    }
    assert!(athena_gate::attribution::manifest_names(&fx.git, &tip, &c0).unwrap());

    // The legacy-exempt record is there, in the same commit, with a reason for every entry.
    assert!(
        fx.bare
            .added_paths(&import)
            .unwrap()
            .contains(&LEGACY_EXEMPT_PATH.to_string())
    );
    let exempt = fx
        .bare
        .show_file(&tip, LEGACY_EXEMPT_PATH)
        .unwrap()
        .unwrap();
    match HistoryRecord::parse(&String::from_utf8(exempt).unwrap()).unwrap() {
        HistoryRecord::LegacyExempt(record) => {
            assert!(!record.entries.is_empty());
            assert!(record.entries.iter().all(|e| !e.reason.trim().is_empty()));
        }
        other => panic!("{other:?}"),
    }

    // The landing reported the cutover.
    assert!(
        fx.report
            .warnings
            .iter()
            .any(|w| w.starts_with("cutover: imported records/ at C0")),
        "{:?}",
        fx.report.warnings
    );
}

#[test]
fn the_fixture_has_a_bare_remote_a_bootstrap_commit_and_names_the_cutover_job() {
    let fx = cutover_fixture();
    assert_eq!(
        fx.bare
            .run(["rev-parse", "--is-bare-repository"])
            .unwrap()
            .trim(),
        "true"
    );
    // The remote holds the same main as the work repository, and the same records branch.
    assert_eq!(
        fx.bare.rev_parse("refs/heads/main").unwrap(),
        fx.git.rev_parse("main").unwrap()
    );
    assert_eq!(
        fx.git
            .rev_parse(athena_git::records_branch::RECORDS_REF)
            .unwrap()
            .as_deref(),
        Some(fx.records_tip().unwrap().as_str())
    );
    assert_eq!(
        fx.git.run(["remote", "get-url", "origin"]).unwrap().trim(),
        fx.bare_root().to_str().unwrap()
    );
    // The bootstrap names no cutover job; the cutover job's landed commit does, and names itself.
    assert_eq!(cutover_job_for(&fx.git, &fx.bootstrap), None);
    assert_eq!(cutover_job_for(&fx.git, &fx.old_way_landing), None);
    assert_eq!(
        cutover_job_for(&fx.git, "main").as_deref(),
        Some(fx.cutover_job.as_str())
    );
    assert_ne!(fx.cutover_job, fx.old_way_job);
}
