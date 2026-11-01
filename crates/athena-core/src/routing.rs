//! Routing the checks by the plan's job kind (`docs/design/ci-routing.md` section 3.2 and section 10 item 5).
//!
//! [`route`] maps a job kind and the capability's check list to what runs and where each result comes from:
//!
//! - **Which checks are skipped** stays the path rule, `skip_when_only` (`checks::skip_note`), unchanged. The
//!   kind never skips `fmt` or `clippy` by itself: a docs, design or study job that changes a `.rs` file,
//!   `Cargo.toml` or `Cargo.lock` runs both, because the path rule sees a changed path that is not prose. The
//!   one thing the kind does to skipping is to switch the path rule off: a `config` job, and a job whose kind
//!   is missing or not recognised (section 7), never has a check skipped. That only makes a job stricter.
//! - **Where a result comes from.** Every kind but `build` asks for a reused CI result for the candidate SHA
//!   through [`crate::ci_result::resolve`] when the capability's checks cover `fmt`, `clippy` and `test`; a
//!   result for another SHA is ignored and the checks run locally; a red result is a failed check and the
//!   local run does not hide it. A `build` job never reuses a prior result and adds the workflow's own run
//!   ([`Route::workflow_run`]), because a build job changes how CI works.
//! - `test` always runs for every kind: no kind carries a skip list for it.
//!
//! The job run (`run.rs`) calls [`run_routed_with_record`]. It gives the source of the CI result and the wait
//! together: for a GitHub origin the push run on the job's CI branch, read every [`CI_POLL`] up to the
//! configured bound ([`Paced`], [`ci_polls`]); with no GitHub origin, or in a nested run, [`NoCi`] with one
//! read and no wait. The workflow's own run for a `build` job is flagged, not performed.

use crate::capability::CheckSpec;
use crate::checks::{runs_from_record, skip_note, skipped_run};
use crate::ci_result::{
    BlockReason, CiRecord, CiRun, CiSource, LocalChecks, Outcome, STEPS, Source, Step, Wait,
    resolve,
};
use anyhow::{Result, bail};
use athena_data::records::CheckRun;
use std::cell::{Cell, RefCell};
use std::time::Duration;

/// A kind of work, as section 3.2 names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Code,
    Tests,
    Docs,
    Design,
    Config,
    Build,
    Study,
}

impl Kind {
    /// The kind for a plan's `class_of_work` (or a capability id such as `study`). A missing or unrecognised
    /// kind is `config`, the all-checks row (section 7).
    pub fn parse(kind: Option<&str>) -> Kind {
        match kind {
            Some("code") => Kind::Code,
            Some("tests") => Kind::Tests,
            Some("docs") => Kind::Docs,
            Some("design") => Kind::Design,
            Some("build") => Kind::Build,
            Some("study") => Kind::Study,
            _ => Kind::Config,
        }
    }

    /// Whether the path rule (`skip_when_only`) applies. It does not for `config`, which never skips a check.
    fn path_rule_applies(self) -> bool {
        self != Kind::Config
    }
}

/// One check and the decision made for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Routed {
    pub spec: CheckSpec,
    /// The note to record when the path rule skips it; `None` when it runs.
    pub skip: Option<String>,
}

/// What the plan's kind and the changed paths decide for the checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub kind: Kind,
    pub items: Vec<Routed>,
    /// Whether a CI result for the candidate SHA is asked for before running locally.
    pub reuse: bool,
    /// A `build` job adds the workflow's own run. It is flagged here; the run path does not perform it yet.
    pub workflow_run: bool,
}

/// Routes `specs` for a job of `kind` that changed `changed`.
pub fn route(kind: Option<&str>, specs: Vec<CheckSpec>, changed: &[String]) -> Route {
    let kind = Kind::parse(kind);
    let covers_all = STEPS.iter().all(|n| specs.iter().any(|s| s.name == *n));
    let items = specs
        .into_iter()
        .map(|spec| {
            let skip = if kind.path_rule_applies() {
                skip_note(&spec, changed)
            } else {
                None
            };
            Routed { spec, skip }
        })
        .collect();
    Route {
        kind,
        items,
        reuse: kind != Kind::Build && covers_all,
        workflow_run: kind == Kind::Build,
    }
}

/// A CI source with no result for any SHA. The job run uses it when there is no GitHub origin to read, and in
/// a nested run.
#[derive(Debug, Clone, Copy)]
pub struct NoCi;

impl CiSource for NoCi {
    fn fetch(&self, _sha: &str) -> Result<Option<CiRun>> {
        Ok(None)
    }
}

/// How long to wait between two reads of the CI run.
pub const CI_POLL: Duration = Duration::from_secs(20);

/// The number of reads for a wait bound of `wait_s` seconds: the first read, then one every [`CI_POLL`]. A
/// bound of 0 is one read and no wait.
pub fn ci_polls(wait_s: u64) -> u32 {
    u32::try_from(wait_s / CI_POLL.as_secs() + 1).unwrap_or(u32::MAX)
}

/// A [`CiSource`] that sleeps `pause`, through the given sleeper, before every read after the first. The
/// pause is made here and not in [`resolve`], so a test can see each step and spend no time.
pub struct Paced<'a> {
    inner: &'a dyn CiSource,
    sleep: &'a dyn Fn(Duration),
    pause: Duration,
    reads: Cell<u32>,
}

impl std::fmt::Debug for Paced<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Paced")
            .field("pause", &self.pause)
            .field("reads", &self.reads.get())
            .finish_non_exhaustive()
    }
}

impl<'a> Paced<'a> {
    pub fn new(inner: &'a dyn CiSource, sleep: &'a dyn Fn(Duration), pause: Duration) -> Self {
        Paced {
            inner,
            sleep,
            pause,
            reads: Cell::new(0),
        }
    }
}

impl CiSource for Paced<'_> {
    fn fetch(&self, sha: &str) -> Result<Option<CiRun>> {
        if self.reads.get() > 0 {
            (self.sleep)(self.pause);
        }
        self.reads.set(self.reads.get() + 1);
        self.inner.fetch(sha)
    }
}

/// Runs one check in the verifier's snapshot (supplied by the job run).
pub trait Exec {
    fn exec(&self, spec: &CheckSpec) -> Result<CheckRun>;
}

/// The checks in order, each skipped or run, stopping after the first failure.
fn run_local(items: &[Routed], exec: &dyn Exec) -> Result<Vec<CheckRun>> {
    let mut runs = Vec::new();
    for item in items {
        if let Some(note) = &item.skip {
            runs.push(skipped_run(&item.spec, note.clone()));
            continue;
        }
        let run = exec.exec(&item.spec)?;
        let failed = !run.passed;
        runs.push(run);
        if failed {
            break;
        }
    }
    Ok(runs)
}

/// Runs the routed checks locally for [`resolve`], keeping the check records for the caller.
struct Fallback<'a> {
    route: &'a Route,
    exec: &'a dyn Exec,
    runs: RefCell<Option<Vec<CheckRun>>>,
    error: RefCell<Option<anyhow::Error>>,
}

impl LocalChecks for Fallback<'_> {
    fn run(&self, _sha: &str) -> Result<Vec<Step>> {
        let runs = match run_local(&self.route.items, self.exec) {
            Ok(runs) => runs,
            Err(e) => {
                let why = format!("{e:#}");
                *self.error.borrow_mut() = Some(e);
                bail!("{why}");
            }
        };
        let steps = STEPS
            .iter()
            .map(|name| match runs.iter().find(|r| r.name == *name) {
                Some(r) => Step {
                    name: name.to_string(),
                    conclusion: if r.passed { "success" } else { "failure" }.into(),
                    duration_s: r.duration.value().unwrap_or(0.0),
                },
                // Not reached because an earlier check failed: not a pass.
                None => Step {
                    name: name.to_string(),
                    conclusion: "not_run".into(),
                    duration_s: 0.0,
                },
            })
            .collect();
        *self.runs.borrow_mut() = Some(runs);
        Ok(steps)
    }
}

/// Runs the routed checks for the candidate `sha` and returns their records.
///
/// When the route reuses CI results, [`resolve`] is called with `sha`: a green run for that SHA stands in for
/// the checks; a run for another SHA is ignored and the checks run through `exec`; a red run for the SHA is
/// returned as failed check records and nothing runs locally. Otherwise the checks run through `exec`.
pub fn run_routed(
    route: &Route,
    sha: &str,
    changed: &[String],
    ci: &dyn CiSource,
    exec: &dyn Exec,
    wait: Wait,
) -> Result<Vec<CheckRun>> {
    run_routed_with_record(route, sha, changed, ci, exec, wait).map(|(runs, _)| runs)
}

/// [`run_routed`], also returning the record of the CI run when the check records came from it (green or
/// red). It is `None` when the checks ran locally, so a caller states a reused result only when there is one.
pub fn run_routed_with_record(
    route: &Route,
    sha: &str,
    changed: &[String],
    ci: &dyn CiSource,
    exec: &dyn Exec,
    wait: Wait,
) -> Result<(Vec<CheckRun>, Option<CiRecord>)> {
    if !route.reuse {
        return Ok((run_local(&route.items, exec)?, None));
    }
    let fallback = Fallback {
        route,
        exec,
        runs: RefCell::new(None),
        error: RefCell::new(None),
    };
    match resolve(sha, changed, ci, &fallback, wait) {
        Outcome::Verified(record) | Outcome::Failed(record) if record.source == Source::Ci => {
            let specs: Vec<CheckSpec> = route.items.iter().map(|i| i.spec.clone()).collect();
            let reused = runs_from_record(&specs, &record, sha);
            let runs = route
                .items
                .iter()
                .map(|item| {
                    let from_ci = reused.iter().find(|r| r.name == item.spec.name);
                    match (&item.skip, from_ci) {
                        // A skipped check stays skipped unless the CI result for it is red.
                        (Some(note), Some(r)) if r.passed => skipped_run(&item.spec, note.clone()),
                        (_, Some(r)) => r.clone(),
                        (Some(note), None) => skipped_run(&item.spec, note.clone()),
                        // Not named by the record: cannot be, since resolve needs all three; checked anyway.
                        (None, None) => CheckRun {
                            passed: false,
                            note: Some("no CI result for this check".into()),
                            ..skipped_run(&item.spec, String::new())
                        },
                    }
                })
                .collect();
            Ok((runs, Some(record)))
        }
        Outcome::Blocked(BlockReason::BadSha(_)) => Ok((run_local(&route.items, exec)?, None)),
        Outcome::Blocked(reason) => match fallback.error.take() {
            Some(e) => Err(e),
            None => bail!("checks blocked: {reason}"),
        },
        Outcome::Verified(_) | Outcome::Failed(_) => match fallback.runs.take() {
            Some(runs) => Ok((runs, None)),
            None => bail!("the local checks produced no records"),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ci_result::{CiRun, Source};
    use athena_data::config::{CheckKind, Config};
    use athena_data::{Quantity, Unit};
    use std::cell::Cell;
    use std::time::Duration;

    const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
    const OTHER: &str = "fedcba9876543210fedcba9876543210fedcba98";
    const WAIT: Wait = Wait {
        polls: 1,
        pause: Duration::ZERO,
    };

    fn prose() -> Option<Vec<String>> {
        Some(vec!["docs/".into(), "*.md".into()])
    }

    /// The three checks as athena.toml's change capability gives them.
    fn specs() -> Vec<CheckSpec> {
        let spec = |name: &str, kind, skip| CheckSpec {
            name: name.into(),
            run: vec!["true".into()],
            kind,
            skip_when_only: skip,
        };
        vec![
            spec("fmt", CheckKind::Command, prose()),
            spec("clippy", CheckKind::Command, prose()),
            spec("test", CheckKind::Test, None),
        ]
    }

    fn paths(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    /// Names of the checks that run, and of those skipped.
    fn outcome(r: &Route) -> (Vec<&str>, Vec<&str>) {
        let names = |skipped: bool| -> Vec<&str> {
            r.items
                .iter()
                .filter(|i| i.skip.is_some() == skipped)
                .map(|i| i.spec.name.as_str())
                .collect()
        };
        (names(false), names(true))
    }

    fn all_run(kind: Option<&str>, changed: &[&str]) {
        let r = route(kind, specs(), &paths(changed));
        assert_eq!(
            outcome(&r),
            (vec!["fmt", "clippy", "test"], vec![]),
            "{kind:?} {changed:?}"
        );
    }

    #[test]
    fn routing_docs_job_with_only_prose_skips_fmt_and_clippy_and_runs_test() {
        let r = route(
            Some("docs"),
            specs(),
            &paths(&["docs/design/a.md", "README.md"]),
        );
        assert_eq!(outcome(&r), (vec!["test"], vec!["fmt", "clippy"]));
        assert!(r.items[0].skip.as_ref().unwrap().contains("not run"));
    }

    #[test]
    fn routing_docs_job_that_changes_rust_runs_fmt_and_clippy() {
        all_run(Some("docs"), &["docs/a.md", "crates/x/src/lib.rs"]);
        all_run(Some("docs"), &["Cargo.toml"]);
        all_run(Some("docs"), &["Cargo.lock"]);
    }

    #[test]
    fn routing_design_job_that_changes_rust_runs_fmt_and_clippy() {
        all_run(Some("design"), &["docs/design/a.md", "xtask/src/main.rs"]);
        all_run(Some("design"), &["Cargo.toml"]);
        all_run(Some("design"), &["Cargo.lock"]);
        let prose_only = route(Some("design"), specs(), &paths(&["docs/design/a.md"]));
        assert_eq!(outcome(&prose_only), (vec!["test"], vec!["fmt", "clippy"]));
    }

    #[test]
    fn routing_study_job_that_changes_rust_runs_fmt_and_clippy() {
        all_run(Some("study"), &["studies/x/notes.md", "src/lib.rs"]);
        all_run(Some("study"), &["Cargo.toml"]);
        all_run(Some("study"), &["Cargo.lock"]);
    }

    #[test]
    fn routing_code_and_tests_jobs_run_every_check() {
        for kind in ["code", "tests"] {
            all_run(Some(kind), &["crates/x/src/lib.rs"]);
            all_run(Some(kind), &["crates/x/src/lib.rs", "docs/a.md"]);
            all_run(Some(kind), &["Cargo.lock"]);
        }
    }

    #[test]
    fn routing_config_job_never_skips_a_check() {
        all_run(Some("config"), &["athena.toml"]);
        // Even a change of only prose: config never skips.
        all_run(Some("config"), &["docs/a.md"]);
        all_run(Some("config"), &["README.md"]);
    }

    #[test]
    fn routing_build_job_adds_the_workflow_run_and_never_reuses() {
        let r = route(
            Some("build"),
            specs(),
            &paths(&[".github/workflows/ci.yml"]),
        );
        assert!(r.workflow_run);
        assert!(!r.reuse);
        assert_eq!(outcome(&r), (vec!["fmt", "clippy", "test"], vec![]));
        for kind in ["code", "tests", "docs", "design", "config", "study"] {
            let other = route(Some(kind), specs(), &paths(&["src/a.rs"]));
            assert!(!other.workflow_run && other.reuse, "{kind}");
        }
    }

    #[test]
    fn routing_missing_or_unrecognised_kind_routes_as_config() {
        for kind in [None, Some(""), Some("other"), Some("prompts"), Some("Docs")] {
            let r = route(kind, specs(), &paths(&["docs/a.md"]));
            assert_eq!(r.kind, Kind::Config, "{kind:?}");
            assert_eq!(outcome(&r), (vec!["fmt", "clippy", "test"], vec![]));
            assert_eq!(r, route(Some("config"), specs(), &paths(&["docs/a.md"])));
        }
    }

    #[test]
    fn routing_no_kind_skips_by_itself() {
        // With no skip lists configured, no kind skips anything, whatever the paths.
        let bare: Vec<CheckSpec> = specs()
            .into_iter()
            .map(|s| CheckSpec {
                skip_when_only: None,
                ..s
            })
            .collect();
        for kind in [
            "code", "tests", "docs", "design", "config", "build", "study", "x",
        ] {
            let r = route(Some(kind), bare.clone(), &paths(&["docs/a.md"]));
            assert_eq!(outcome(&r).1, Vec::<&str>::new(), "{kind}");
        }
    }

    // --- the routed path and `resolve` ---

    struct Ci {
        run: Option<CiRun>,
        reads: Cell<u32>,
        asked: RefCell<Vec<String>>,
    }

    impl Ci {
        fn new(run: Option<CiRun>) -> Ci {
            Ci {
                run,
                reads: Cell::new(0),
                asked: RefCell::new(Vec::new()),
            }
        }
    }

    impl CiSource for Ci {
        fn fetch(&self, sha: &str) -> Result<Option<CiRun>> {
            self.reads.set(self.reads.get() + 1);
            self.asked.borrow_mut().push(sha.to_string());
            Ok(self.run.clone())
        }
    }

    fn ci_run(sha: &str, conclusion: &str) -> CiRun {
        CiRun {
            sha: sha.into(),
            address: "https://example.test/runs/5".into(),
            finished: true,
            conclusion: conclusion.into(),
            steps: STEPS
                .iter()
                .map(|n| Step {
                    name: n.to_string(),
                    conclusion: conclusion.into(),
                    duration_s: 2.0,
                })
                .collect(),
            duration_s: 6.0,
        }
    }

    struct Local {
        ran: RefCell<Vec<String>>,
        fail: Option<&'static str>,
    }

    impl Local {
        fn new(fail: Option<&'static str>) -> Local {
            Local {
                ran: RefCell::new(Vec::new()),
                fail,
            }
        }
    }

    impl Exec for Local {
        fn exec(&self, spec: &CheckSpec) -> Result<CheckRun> {
            self.ran.borrow_mut().push(spec.name.clone());
            Ok(CheckRun {
                name: spec.name.clone(),
                command: spec.run.clone(),
                exit_code: Some(0),
                passed: self.fail != Some(spec.name.as_str()),
                tests_ran: None,
                duration: Quantity::measured(1.0, Unit::Seconds, "test"),
                output_tail: String::new(),
                note: None,
                failed_tests: Vec::new(),
                reused: false,
            })
        }
    }

    fn go(kind: &str, changed: &[&str], ci: &Ci, local: &Local) -> Vec<CheckRun> {
        let changed = paths(changed);
        let r = route(Some(kind), specs(), &changed);
        run_routed(&r, SHA, &changed, ci, local, WAIT).unwrap()
    }

    #[test]
    fn routing_calls_resolve_with_the_candidate_sha_and_reuses_a_green_run() {
        let ci = Ci::new(Some(ci_run(SHA, "success")));
        let local = Local::new(None);
        let runs = go("code", &["src/a.rs"], &ci, &local);
        assert_eq!(*ci.asked.borrow(), vec![SHA.to_string()]);
        assert!(local.ran.borrow().is_empty(), "nothing runs locally");
        assert_eq!(runs.len(), 3);
        assert!(runs.iter().all(|r| r.passed));
        assert!(runs[0].note.as_ref().unwrap().contains("runs/5"));
        assert!(
            runs.iter().all(|r| r.reused),
            "records taken from CI are marked reused"
        );
    }

    #[test]
    fn routing_ignores_a_ci_result_for_another_sha_and_runs_locally() {
        let ci = Ci::new(Some(ci_run(OTHER, "success")));
        let local = Local::new(None);
        let runs = go("code", &["src/a.rs"], &ci, &local);
        assert_eq!(*local.ran.borrow(), vec!["fmt", "clippy", "test"]);
        assert!(runs.iter().all(|r| r.passed && r.exit_code == Some(0)));
        assert!(runs.iter().all(|r| !r.reused), "locally run, not reused");
        // A red run for another SHA is not this candidate's failure either.
        let ci = Ci::new(Some(ci_run(OTHER, "failure")));
        let local = Local::new(None);
        assert!(
            go("code", &["src/a.rs"], &ci, &local)
                .iter()
                .all(|r| r.passed)
        );
        assert_eq!(local.ran.borrow().len(), 3);
    }

    #[test]
    fn routing_a_red_ci_result_fails_and_does_not_rerun_locally() {
        let ci = Ci::new(Some(ci_run(SHA, "failure")));
        let local = Local::new(None);
        let runs = go("docs", &["src/a.rs"], &ci, &local);
        assert!(local.ran.borrow().is_empty(), "the fallback must not run");
        assert!(runs.iter().any(|r| !r.passed));
    }

    #[test]
    fn routing_no_ci_result_runs_locally_and_stops_at_the_first_failure() {
        let ci = Ci::new(None);
        let local = Local::new(Some("clippy"));
        let runs = go("code", &["src/a.rs"], &ci, &local);
        assert_eq!(*local.ran.borrow(), vec!["fmt", "clippy"]);
        assert_eq!(runs.len(), 2);
        assert!(runs[0].passed && !runs[1].passed);
    }

    #[test]
    fn routing_a_docs_job_with_no_ci_result_skips_by_path_and_runs_test() {
        let ci = Ci::new(None);
        let local = Local::new(None);
        let runs = go("docs", &["docs/a.md"], &ci, &local);
        assert_eq!(*local.ran.borrow(), vec!["test"]);
        assert_eq!(runs.len(), 3);
        assert!(runs[0].note.as_ref().unwrap().contains("not run"));
        assert!(runs.iter().all(|r| r.passed));
        assert!(runs.iter().all(|r| !r.reused), "skipped is not reused");
    }

    // --- the wait: one read every CI_POLL, up to the bound ---

    /// A sleeper that records each pause and spends no time.
    fn recording_sleeper() -> (std::rc::Rc<RefCell<Vec<Duration>>>, impl Fn(Duration)) {
        let slept = std::rc::Rc::new(RefCell::new(Vec::new()));
        let log = slept.clone();
        (slept, move |d: Duration| log.borrow_mut().push(d))
    }

    fn waiting(wait_s: u64) -> Wait {
        Wait {
            polls: ci_polls(wait_s),
            pause: Duration::ZERO,
        }
    }

    #[test]
    fn routing_ci_polls_is_one_read_then_one_every_twenty_seconds() {
        assert_eq!(CI_POLL, Duration::from_secs(20));
        assert_eq!(ci_polls(0), 1);
        assert_eq!(ci_polls(19), 1);
        assert_eq!(ci_polls(20), 2);
        assert_eq!(ci_polls(1500), 76);
        assert_eq!(ci_polls(u64::MAX), u32::MAX);
    }

    #[test]
    fn routing_a_run_that_never_finishes_is_read_76_times_with_75_pauses_then_runs_locally() {
        let mut unfinished = ci_run(SHA, "");
        unfinished.finished = false;
        let ci = Ci::new(Some(unfinished));
        let (slept, sleep) = recording_sleeper();
        let paced = Paced::new(&ci, &sleep, CI_POLL);
        let local = Local::new(None);
        let changed = paths(&["src/a.rs"]);
        let r = route(Some("code"), specs(), &changed);
        let (runs, record) =
            run_routed_with_record(&r, SHA, &changed, &paced, &local, waiting(1500)).unwrap();
        assert_eq!(ci.reads.get(), 76);
        assert_eq!(*slept.borrow(), vec![CI_POLL; 75]);
        assert_eq!(*local.ran.borrow(), vec!["fmt", "clippy", "test"]);
        assert!(runs.iter().all(|r| r.passed));
        assert_eq!(record, None, "the checks ran locally: no CI record");
    }

    #[test]
    fn routing_no_wait_is_one_read_and_no_pause() {
        let (slept, sleep) = recording_sleeper();
        let changed = paths(&["src/a.rs"]);
        let r = route(Some("code"), specs(), &changed);
        // A bound of 0 against a source with a run that never finishes: one read.
        let mut unfinished = ci_run(SHA, "");
        unfinished.finished = false;
        let ci = Ci::new(Some(unfinished));
        let paced = Paced::new(&ci, &sleep, CI_POLL);
        let local = Local::new(None);
        run_routed_with_record(&r, SHA, &changed, &paced, &local, waiting(0)).unwrap();
        assert_eq!(ci.reads.get(), 1);
        // NoCi with one read: nothing to wait for, nothing slept.
        let local = Local::new(None);
        let (runs, record) =
            run_routed_with_record(&r, SHA, &changed, &NoCi, &local, waiting(0)).unwrap();
        assert_eq!(*local.ran.borrow(), vec!["fmt", "clippy", "test"]);
        assert_eq!((runs.len(), record), (3, None));
        assert!(slept.borrow().is_empty());
    }

    #[test]
    fn routing_returns_the_ci_record_for_a_green_or_red_run_and_none_for_a_local_one() {
        let changed = paths(&["src/a.rs"]);
        let r = route(Some("code"), specs(), &changed);
        for (conclusion, all_passed) in [("success", true), ("failure", false)] {
            let ci = Ci::new(Some(ci_run(SHA, conclusion)));
            let local = Local::new(None);
            let (runs, record) =
                run_routed_with_record(&r, SHA, &changed, &ci, &local, WAIT).unwrap();
            let record = record.expect("the record of the CI run");
            assert_eq!(record.source, Source::Ci);
            assert_eq!(record.sha, SHA);
            assert_eq!(record.all_passed(), all_passed);
            assert!(local.ran.borrow().is_empty());
            assert_eq!(runs.iter().all(|r| r.passed), all_passed);
        }
        let ci = Ci::new(None);
        let local = Local::new(None);
        let (_, record) = run_routed_with_record(&r, SHA, &changed, &ci, &local, WAIT).unwrap();
        assert_eq!(record, None);
    }

    #[test]
    fn routing_a_build_job_does_not_read_a_prior_ci_result() {
        let ci = Ci::new(Some(ci_run(SHA, "success")));
        let local = Local::new(None);
        let runs = go("build", &[".github/workflows/ci.yml"], &ci, &local);
        assert_eq!(ci.reads.get(), 0, "no CI read");
        assert_eq!(*local.ran.borrow(), vec!["fmt", "clippy", "test"]);
        assert_eq!(runs.len(), 3);
    }

    #[test]
    fn routing_a_capability_without_the_three_checks_runs_all_locally() {
        let only_test = vec![specs().pop().unwrap()];
        let changed = paths(&["src/a.rs"]);
        let r = route(Some("code"), only_test, &changed);
        assert!(!r.reuse);
        let ci = Ci::new(Some(ci_run(SHA, "success")));
        let local = Local::new(None);
        run_routed(&r, SHA, &changed, &ci, &local, WAIT).unwrap();
        assert_eq!(ci.reads.get(), 0);
        assert_eq!(*local.ran.borrow(), vec!["test"]);
    }

    #[test]
    fn routing_a_local_record_is_keyed_by_the_sha() {
        // The record resolve builds from the local run carries the candidate SHA.
        let ci = Ci::new(None);
        let local = Local::new(None);
        let r = route(Some("code"), specs(), &[]);
        let fallback = Fallback {
            route: &r,
            exec: &local,
            runs: RefCell::new(None),
            error: RefCell::new(None),
        };
        match resolve(SHA, &[], &ci, &fallback, WAIT) {
            Outcome::Verified(record) => {
                assert_eq!(record.source, Source::Local);
                assert_eq!(record.sha, SHA);
            }
            other => panic!("expected Verified, got {other:?}"),
        }
    }

    // --- the committed configuration ---

    fn committed(capability: &str) -> Vec<CheckSpec> {
        let config = Config::parse(include_str!("../../../athena.toml"))
            .expect("the committed athena.toml parses under the schema in the tree");
        config
            .capability(capability)
            .unwrap()
            .checks
            .iter()
            .map(|k| CheckSpec {
                name: k.name.clone(),
                run: k.run.clone(),
                kind: k.kind,
                skip_when_only: k.skip_when_only.clone(),
            })
            .collect()
    }

    #[test]
    fn routing_with_the_committed_config_follows_the_table() {
        let run_names = |kind: &str, caps: &str, changed: &[&str]| {
            let r = route(Some(kind), committed(caps), &paths(changed));
            let (ran, skipped) = outcome(&r);
            (
                ran.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                skipped.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
                r.reuse,
                r.workflow_run,
            )
        };
        let all = vec!["fmt".to_string(), "clippy".into(), "test".into()];
        let test_only = vec!["test".to_string()];
        let both = vec!["fmt".to_string(), "clippy".into()];
        // docs and design: prose-only skips fmt and clippy by the path rule; a Rust change does not.
        for kind in ["docs", "design"] {
            assert_eq!(
                run_names(kind, "change", &["docs/design/a.md", "CHANGELOG.md"]),
                (test_only.clone(), both.clone(), true, false)
            );
            assert_eq!(
                run_names(kind, "change", &["docs/a.md", "crates/x/src/lib.rs"]),
                (all.clone(), vec![], true, false)
            );
            assert_eq!(
                run_names(kind, "change", &["Cargo.lock"]),
                (all.clone(), vec![], true, false)
            );
        }
        // code, tests, config: all three run.
        for kind in ["code", "tests", "config"] {
            assert_eq!(
                run_names(kind, "change", &["crates/x/src/lib.rs"]),
                (all.clone(), vec![], true, false)
            );
        }
        assert_eq!(
            run_names("config", "change", &["docs/a.md"]),
            (all.clone(), vec![], true, false)
        );
        // build: all three, plus the workflow's own run, no reuse.
        assert_eq!(
            run_names("build", "change", &[".github/workflows/ci.yml"]),
            (all.clone(), vec![], false, true)
        );
        // unrecognised kind: as config.
        assert_eq!(
            run_names("other", "change", &["docs/a.md"]),
            (all.clone(), vec![], true, false)
        );
        // study: committed study checks have no skip list, so every check runs for any change.
        assert_eq!(
            run_names("study", "study", &["src/lib.rs"]),
            (all.clone(), vec![], true, false)
        );
        assert_eq!(
            run_names("study", "study", &["notes.md"]),
            (all, vec![], true, false)
        );
    }
}
