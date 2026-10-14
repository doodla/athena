//! The landing trigger of the watcher (`docs/design/decision-sets.md` section 8, step 4), against a real
//! repository: refused host calls become records, and a refused merge puts D87 under review at once.

mod common;

use athena_core::gh::HostErrorLog;
use athena_core::store::Records;
use athena_data::JobId;
use athena_data::records::{
    DecisionStateKind, DefaultExpectations, ExpectationSignal, HostOperation, SignalSource,
    SignalStrength, WorkPhase,
};
use common::*;

fn log(refusals: &[(HostOperation, Option<u16>)]) -> HostErrorLog {
    let log = HostErrorLog::new(
        JobId::parse("20261013-192156-ar1x").unwrap(),
        WorkPhase::Landing,
    );
    for (operation, status) in refusals {
        log.record(*operation, *status);
    }
    log
}

fn records(fx: &Fixture) -> Records {
    Records::load(&fx.git, &fx.head()).unwrap()
}

#[test]
fn a_405_on_merge_writes_one_host_error_and_puts_d87_under_review_on_the_first_occurrence() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let lines = Lines::default();
    let written = athena
        .watch_landing(&log(&[(HostOperation::Merge, Some(405))]), &lines)
        .unwrap();
    assert_eq!(written.len(), 1);
    let records = records(&fx);
    assert_eq!(records.host_errors.len(), 1);
    assert_eq!(records.host_errors[0].status_code, Some(405));
    assert_eq!(records.decision_states.len(), 1);
    let state = &records.decision_states[0];
    assert_eq!(state.decision, "D87");
    assert_eq!(state.state, DecisionStateKind::UnderReview);
    assert_eq!(state.firing_records, [records.host_errors[0].id.clone()]);
    assert_eq!(
        records.decision_state()["D87"],
        DecisionStateKind::UnderReview
    );
    assert!(
        lines
            .0
            .lock()
            .unwrap()
            .iter()
            .any(|l| l.contains("under review") && l.contains("D87"))
    );
    // Both records are accepted by `athena check`.
    let report = athena.check_working().unwrap();
    assert!(report.violations.is_empty(), "{:?}", report.violations);

    // The run at landing again, with nothing new refused, writes nothing more: the record is not evaluated
    // twice.
    let again = athena.watch_landing(&log(&[]), &Lines::default()).unwrap();
    assert!(again.is_empty());
    assert_eq!(self::records(&fx).decision_states.len(), 1);
}

#[test]
fn a_404_or_a_500_is_recorded_but_does_not_put_d87_under_review() {
    for status in [Some(404), Some(500), None] {
        let fx = Fixture::new();
        let athena = fx.athena(Scenario::default().script());
        let written = athena
            .watch_landing(&log(&[(HostOperation::Merge, status)]), &Lines::default())
            .unwrap();
        assert!(written.is_empty(), "{status:?}");
        let records = records(&fx);
        assert_eq!(records.host_errors.len(), 1, "{status:?}");
        assert!(records.decision_states.is_empty(), "{status:?}");
        assert!(records.decision_state().is_empty());
    }
}

#[test]
fn a_405_on_another_operation_does_not_put_d87_under_review() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let written = athena
        .watch_landing(
            &log(&[(HostOperation::CreatePr, Some(405))]),
            &Lines::default(),
        )
        .unwrap();
    assert!(written.is_empty());
    assert!(records(&fx).decision_states.is_empty());
}

#[test]
fn default_expectations_are_written_and_read_back_by_default_id() {
    let fx = Fixture::new();
    let athena = fx.athena(Scenario::default().script());
    let expectations = DefaultExpectations {
        id: athena_data::RecordId::generate(athena_core::now()),
        at: athena_core::now(),
        default_id: "small".into(),
        assumptions: vec!["A focused change is easier to verify.".into()],
        signals: vec![ExpectationSignal {
            source: SignalSource::Rejection,
            strength: SignalStrength::Soft,
            text: "a change rejected as too large".into(),
            threshold_jobs: 2,
        }],
        defaults_version: 1,
        proposal: None,
    };
    let (written, _commit) = athena.record_default_expectations(expectations).unwrap();
    let records = records(&fx);
    assert_eq!(records.expectations_for("small"), Some(&written));
    assert_eq!(records.expectations_for("tested"), None);
    let mut weak = written.clone();
    weak.signals[0].threshold_jobs = 1;
    assert!(athena.record_default_expectations(weak).is_err());
    assert!(athena.check_working().unwrap().violations.is_empty());
}
