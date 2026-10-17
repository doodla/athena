//! The CI result record, keyed by commit SHA, and the rules for using it (`docs/design/ci-routing.md` 5.2, 5.3).
//!
//! Athena reads one workflow run for the exact candidate SHA, through an injected [`CiSource`], and decides
//! what the three checks (`fmt`, `clippy`, `test`) are for that SHA:
//!
//! - a finished green run for the same SHA is reused ([`Source::Ci`]);
//! - a finished red run for the same SHA is a failed check, and the local fallback does not run to hide it;
//! - no run, an unfinished run past the wait bound, a cancelled, skipped or timed-out run, an unreachable
//!   service, a run for another SHA (stale), or a green run when the candidate also changes a workflow file
//!   all select the local fallback, which runs the three checks once and records them as local, keyed by SHA;
//! - when the fallback cannot produce a complete result either, the outcome is [`Outcome::Blocked`] with a
//!   named reason. Nothing here ever yields [`Outcome::Verified`] without three passed steps for the SHA.
//!
//! The record is read by Athena, never by the verifier. Text from a run is data; only fixed fields are kept.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

/// The three checks a record must cover, in order.
pub const STEPS: [&str; 3] = ["fmt", "clippy", "test"];

/// Workflow files live under this directory; a candidate that changes one controls the checker.
pub const WORKFLOW_DIR: &str = ".github/workflows/";

/// Where a record's results came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Ci,
    Local,
}

/// One check's conclusion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    /// One of [`STEPS`].
    pub name: String,
    /// `success` or `failure`.
    pub conclusion: String,
    pub duration_s: f64,
}

impl Step {
    pub fn passed(&self) -> bool {
        self.conclusion == "success"
    }
}

/// The record of the three checks for one commit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CiRecord {
    pub source: Source,
    /// The full 40-character commit SHA the results are for.
    pub sha: String,
    /// The run's address on the platform; none for a local record.
    pub run_address: Option<String>,
    pub steps: Vec<Step>,
    pub duration_s: f64,
}

impl CiRecord {
    /// Whether the record holds all of [`STEPS`] and every one passed.
    pub fn all_passed(&self) -> bool {
        STEPS
            .iter()
            .all(|n| self.steps.iter().any(|s| s.name == *n && s.passed()))
    }
}

/// One workflow run as the platform reports it, before any rule is applied.
#[derive(Debug, Clone, PartialEq)]
pub struct CiRun {
    /// The commit the run was made for.
    pub sha: String,
    pub address: String,
    /// Whether the run has finished.
    pub finished: bool,
    /// `success`, `failure`, `cancelled`, `skipped`, `timed_out`, ...; meaningful once finished.
    pub conclusion: String,
    pub steps: Vec<Step>,
    pub duration_s: f64,
}

/// Reads the run for a SHA from the platform.
pub trait CiSource {
    /// `Ok(None)`: no run yet. `Err`: the service could not be asked.
    fn fetch(&self, sha: &str) -> Result<Option<CiRun>>;
}

/// Runs the three checks locally for a SHA, once, and returns their steps.
pub trait LocalChecks {
    fn run(&self, sha: &str) -> Result<Vec<Step>>;
}

/// How long to wait for an unfinished or absent run: `polls` reads, `pause` between them. The values come
/// from measurement data (section 9), not from this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wait {
    pub polls: u32,
    pub pause: Duration,
}

/// Why no verdict could be reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockReason {
    /// The candidate's SHA is not a full 40-character hex SHA, so nothing can be keyed by it.
    BadSha(String),
    /// The local fallback could not run.
    LocalUnavailable(String),
    /// The local fallback ran but did not report all three checks.
    LocalIncomplete(Vec<String>),
}

impl fmt::Display for BlockReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockReason::BadSha(sha) => write!(f, "the candidate SHA {sha:?} is not a full SHA"),
            BlockReason::LocalUnavailable(why) => write!(
                f,
                "no usable CI result and the local fallback could not run: {why}"
            ),
            BlockReason::LocalIncomplete(missing) => write!(
                f,
                "no usable CI result and the local fallback did not report: {}",
                missing.join(", ")
            ),
        }
    }
}

/// What the three checks came to for the candidate.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// All three checks passed for the SHA, in CI or locally.
    Verified(CiRecord),
    /// At least one check failed for the SHA.
    Failed(CiRecord),
    /// No result could be had.
    Blocked(BlockReason),
}

/// What a read of the platform came to.
enum Reading {
    Green(CiRun),
    Red(CiRun),
    Missing,
}

fn is_full_sha(sha: &str) -> bool {
    sha.len() == 40 && sha.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn record_of(run: CiRun) -> CiRecord {
    CiRecord {
        source: Source::Ci,
        sha: run.sha,
        run_address: Some(run.address),
        steps: run.steps,
        duration_s: run.duration_s,
    }
}

fn read_ci(source: &dyn CiSource, sha: &str, wait: Wait) -> Reading {
    for poll in 0..wait.polls.max(1) {
        if poll > 0 {
            std::thread::sleep(wait.pause);
        }
        match source.fetch(sha) {
            // The service cannot be asked: waiting longer will not help.
            Err(_) => return Reading::Missing,
            Ok(None) => {}
            Ok(Some(run)) => {
                // A result for another commit is stale: ignored for every claim.
                if run.sha != sha || !run.finished {
                    continue;
                }
                return match run.conclusion.as_str() {
                    "success" => Reading::Green(run),
                    "failure" => Reading::Red(run),
                    // cancelled, skipped, timed_out, startup_failure, anything else: no result.
                    _ => Reading::Missing,
                };
            }
        }
    }
    Reading::Missing
}

/// Decides the three checks for `sha`. `changed` is the candidate's changed paths.
pub fn resolve(
    sha: &str,
    changed: &[String],
    source: &dyn CiSource,
    local: &dyn LocalChecks,
    wait: Wait,
) -> Outcome {
    if !is_full_sha(sha) {
        return Outcome::Blocked(BlockReason::BadSha(sha.to_string()));
    }
    let workflow_changed = changed.iter().any(|p| p.starts_with(WORKFLOW_DIR));
    match read_ci(source, sha, wait) {
        Reading::Red(run) => return Outcome::Failed(record_of(run)),
        Reading::Green(run) if !workflow_changed => {
            let record = record_of(run);
            // A green run that does not cover all three checks is not a result for them.
            if record.all_passed() {
                return Outcome::Verified(record);
            }
        }
        // Green, but the candidate controls the checker: not trusted.
        Reading::Green(_) | Reading::Missing => {}
    }
    fall_back(sha, local)
}

fn fall_back(sha: &str, local: &dyn LocalChecks) -> Outcome {
    let steps = match local.run(sha) {
        Ok(steps) => steps,
        Err(e) => return Outcome::Blocked(BlockReason::LocalUnavailable(format!("{e:#}"))),
    };
    let missing: Vec<String> = STEPS
        .iter()
        .filter(|n| !steps.iter().any(|s| s.name == **n))
        .map(|n| n.to_string())
        .collect();
    if !missing.is_empty() {
        return Outcome::Blocked(BlockReason::LocalIncomplete(missing));
    }
    let record = CiRecord {
        source: Source::Local,
        sha: sha.to_string(),
        run_address: None,
        duration_s: steps.iter().map(|s| s.duration_s).sum(),
        steps,
    };
    if record.all_passed() {
        Outcome::Verified(record)
    } else {
        Outcome::Failed(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    const SHA: &str = "0123456789abcdef0123456789abcdef01234567";
    const OTHER: &str = "fedcba9876543210fedcba9876543210fedcba98";

    fn steps(conclusion: &str) -> Vec<Step> {
        STEPS
            .iter()
            .map(|n| Step {
                name: n.to_string(),
                conclusion: conclusion.into(),
                duration_s: 2.0,
            })
            .collect()
    }

    fn run(sha: &str, finished: bool, conclusion: &str) -> CiRun {
        CiRun {
            sha: sha.into(),
            address: "https://example.test/runs/1".into(),
            finished,
            conclusion: conclusion.into(),
            steps: steps(if conclusion == "success" {
                "success"
            } else {
                "failure"
            }),
            duration_s: 6.0,
        }
    }

    struct Ci(RefCell<Vec<Result<Option<CiRun>, String>>>, Cell<u32>);

    impl Ci {
        fn new(replies: Vec<Result<Option<CiRun>, String>>) -> Self {
            Ci(RefCell::new(replies), Cell::new(0))
        }
        fn one(run: Option<CiRun>) -> Self {
            Ci::new(vec![Ok(run)])
        }
    }

    impl CiSource for Ci {
        fn fetch(&self, _: &str) -> Result<Option<CiRun>> {
            self.1.set(self.1.get() + 1);
            let mut replies = self.0.borrow_mut();
            let reply = if replies.len() > 1 {
                replies.remove(0)
            } else {
                replies[0].clone()
            };
            reply.map_err(|e| anyhow::anyhow!(e))
        }
    }

    struct Local {
        reply: Result<Vec<Step>, String>,
        calls: Cell<u32>,
    }

    impl Local {
        fn ok(conclusion: &str) -> Self {
            Local {
                reply: Ok(steps(conclusion)),
                calls: Cell::new(0),
            }
        }
        fn broken() -> Self {
            Local {
                reply: Err("no toolchain".into()),
                calls: Cell::new(0),
            }
        }
    }

    impl LocalChecks for Local {
        fn run(&self, _: &str) -> Result<Vec<Step>> {
            self.calls.set(self.calls.get() + 1);
            self.reply.clone().map_err(|e| anyhow::anyhow!(e))
        }
    }

    const WAIT: Wait = Wait {
        polls: 3,
        pause: Duration::ZERO,
    };

    fn go(ci: &Ci, local: &Local, changed: &[&str]) -> Outcome {
        let changed: Vec<String> = changed.iter().map(|s| s.to_string()).collect();
        resolve(SHA, &changed, ci, local, WAIT)
    }

    fn verified(o: &Outcome) -> &CiRecord {
        match o {
            Outcome::Verified(r) => r,
            other => panic!("expected Verified, got {other:?}"),
        }
    }

    #[test]
    fn a_green_run_for_the_same_sha_is_reused_and_nothing_runs_locally() {
        let ci = Ci::one(Some(run(SHA, true, "success")));
        let local = Local::ok("success");
        let out = go(&ci, &local, &["src/a.rs"]);
        let record = verified(&out);
        assert_eq!(record.source, Source::Ci);
        assert_eq!(record.sha, SHA);
        assert_eq!(
            record.run_address.as_deref(),
            Some("https://example.test/runs/1")
        );
        assert_eq!(record.steps.len(), 3);
        assert_eq!(record.duration_s, 6.0);
        assert_eq!(local.calls.get(), 0);
    }

    #[test]
    fn a_green_run_for_another_sha_is_stale_and_never_verifies() {
        // Without the SHA comparison this would be Verified from CI.
        let ci = Ci::one(Some(run(OTHER, true, "success")));
        let local = Local::broken();
        let out = go(&ci, &local, &["src/a.rs"]);
        assert!(matches!(
            out,
            Outcome::Blocked(BlockReason::LocalUnavailable(_))
        ));
        assert_eq!(local.calls.get(), 1);
    }

    #[test]
    fn a_stale_run_is_ignored_and_the_local_record_is_used() {
        let ci = Ci::one(Some(run(OTHER, true, "success")));
        let local = Local::ok("success");
        let out = go(&ci, &local, &[]);
        let record = verified(&out);
        assert_eq!(record.source, Source::Local);
        assert_eq!(record.sha, SHA);
        assert_eq!(record.run_address, None);
    }

    #[test]
    fn a_red_run_for_another_sha_is_not_this_candidates_failure() {
        let ci = Ci::one(Some(run(OTHER, true, "failure")));
        let local = Local::ok("success");
        let out = go(&ci, &local, &[]);
        assert_eq!(verified(&out).source, Source::Local);
    }

    #[test]
    fn no_unfinished_cancelled_skipped_or_timed_out_run_selects_the_local_fallback() {
        let cases: Vec<Ci> = vec![
            Ci::one(None),
            Ci::one(Some(run(SHA, false, ""))),
            Ci::one(Some(run(SHA, true, "cancelled"))),
            Ci::one(Some(run(SHA, true, "skipped"))),
            Ci::one(Some(run(SHA, true, "timed_out"))),
            Ci::new(vec![Err("service down".into())]),
        ];
        for ci in cases {
            let local = Local::ok("success");
            let out = go(&ci, &local, &["src/a.rs"]);
            let record = verified(&out);
            assert_eq!(record.source, Source::Local);
            assert_eq!(record.sha, SHA);
            assert_eq!(local.calls.get(), 1, "the fallback runs the checks once");
        }
    }

    #[test]
    fn an_unfinished_run_is_waited_for_up_to_the_bound_and_then_read() {
        let ci = Ci::new(vec![
            Ok(None),
            Ok(Some(run(SHA, false, ""))),
            Ok(Some(run(SHA, true, "success"))),
        ]);
        let local = Local::broken();
        let out = go(&ci, &local, &[]);
        assert_eq!(verified(&out).source, Source::Ci);
        assert_eq!(ci.1.get(), 3);
        assert_eq!(local.calls.get(), 0);

        let never = Ci::one(Some(run(SHA, false, "")));
        let out = go(&never, &Local::ok("success"), &[]);
        assert_eq!(verified(&out).source, Source::Local);
        assert_eq!(never.1.get(), 3, "the wait is bounded");
    }

    #[test]
    fn a_red_run_for_the_same_sha_fails_and_the_fallback_does_not_hide_it() {
        let ci = Ci::one(Some(run(SHA, true, "failure")));
        let local = Local::ok("success");
        let out = go(&ci, &local, &[]);
        match out {
            Outcome::Failed(record) => {
                assert_eq!(record.source, Source::Ci);
                assert!(!record.all_passed());
            }
            other => panic!("expected Failed, got {other:?}"),
        }
        assert_eq!(local.calls.get(), 0, "the fallback must not run");
    }

    #[test]
    fn a_green_run_with_a_workflow_file_changed_is_not_trusted() {
        let ci = Ci::one(Some(run(SHA, true, "success")));
        let local = Local::ok("success");
        let out = go(&ci, &local, &["src/a.rs", ".github/workflows/ci.yml"]);
        assert_eq!(verified(&out).source, Source::Local);
        assert_eq!(local.calls.get(), 1);
    }

    #[test]
    fn a_red_run_stays_red_even_when_a_workflow_file_changed() {
        let ci = Ci::one(Some(run(SHA, true, "failure")));
        let local = Local::ok("success");
        let out = go(&ci, &local, &[".github/workflows/ci.yml"]);
        assert!(matches!(out, Outcome::Failed(_)));
        assert_eq!(local.calls.get(), 0);
    }

    #[test]
    fn a_green_run_missing_a_check_is_not_a_result_for_all_three() {
        let mut partial = run(SHA, true, "success");
        partial.steps.pop();
        let local = Local::ok("success");
        let out = go(&Ci::one(Some(partial)), &local, &[]);
        assert_eq!(verified(&out).source, Source::Local);
    }

    #[test]
    fn a_failing_local_check_is_a_failure_keyed_by_sha() {
        let out = go(&Ci::one(None), &Local::ok("failure"), &[]);
        match out {
            Outcome::Failed(record) => {
                assert_eq!(record.source, Source::Local);
                assert_eq!(record.sha, SHA);
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn with_no_ci_result_and_no_local_run_the_job_is_blocked_with_a_reason() {
        let out = go(&Ci::one(None), &Local::broken(), &[]);
        match out {
            Outcome::Blocked(reason) => {
                assert!(reason.to_string().contains("no toolchain"));
            }
            other => panic!("expected Blocked, got {other:?}"),
        }
    }

    #[test]
    fn a_local_run_that_reports_too_little_is_blocked_not_verified() {
        let mut short = steps("success");
        short.retain(|s| s.name != "test");
        let local = Local {
            reply: Ok(short),
            calls: Cell::new(0),
        };
        let out = go(&Ci::one(None), &local, &[]);
        assert_eq!(
            out,
            Outcome::Blocked(BlockReason::LocalIncomplete(vec!["test".into()]))
        );
        let empty = Local {
            reply: Ok(vec![]),
            calls: Cell::new(0),
        };
        assert!(matches!(
            go(&Ci::one(None), &empty, &[]),
            Outcome::Blocked(BlockReason::LocalIncomplete(_))
        ));
    }

    #[test]
    fn a_short_sha_is_blocked_before_anything_is_read() {
        let ci = Ci::one(Some(run("0123", true, "success")));
        let local = Local::ok("success");
        let out = resolve("0123", &[], &ci, &local, WAIT);
        assert!(matches!(out, Outcome::Blocked(BlockReason::BadSha(_))));
        assert_eq!(ci.1.get(), 0);
        assert_eq!(local.calls.get(), 0);
    }

    #[test]
    fn the_record_round_trips_with_its_address_sha_steps_and_duration() {
        let ci = Ci::one(Some(run(SHA, true, "success")));
        let out = go(&ci, &Local::broken(), &[]);
        let record = verified(&out).clone();
        let text = serde_json::to_string(&record).unwrap();
        let back: CiRecord = serde_json::from_str(&text).unwrap();
        assert_eq!(back, record);
        assert!(text.contains(SHA));
        assert!(text.contains("run_address"));
        assert!(text.contains("duration_s"));
    }
}
