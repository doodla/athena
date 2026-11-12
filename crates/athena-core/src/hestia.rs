//! Hestia's sweep skeleton (`docs/design/hestia.md` section 9, job 1; decision D129).
//!
//! A sweep runs a list of [`Check`]s and records one report, an `athena.hestia-sweep/1` record: every check that
//! ran, with the number of things it looked at and the number of findings, and each finding. A check that
//! returns an error does not stop the sweep and does not look clean: [`run_sweep`] turns the error into a
//! finding for that check and marks the check as errored. Hestia reports and never gates work (D127).
//!
//! The trigger is an explicit command: `athena hestia sweep` runs and records a sweep, and `athena hestia last`
//! prints the newest full one. Nothing runs a sweep on a timer; D129 says why. The recurring duties are the
//! exception to "nothing runs inside `athena drain`": see the next paragraph.
//!
//! Hestia's sweep owns Athena's recurring duties (`docs/design/hestia.md` section 13): [`DUTIES`] lists each one
//! with its cadence ([`Cadence`]: every drain start, every landing, every N landings) and where it runs. A pass
//! over them starts from a drain start ([`Athena::review_passes`]), a landing
//! ([`Athena::code_health_after_landing`]) or the sweep ([`Athena::hestia_sweep_with`]); all three go through
//! [`Athena::execute_duties`], which runs the duties that are due ([`is_due`]) and the sweep record stores each
//! duty that ran with the landing total (`duty_pass`). `athena status` shows [`duty_standings`] for every duty and
//! flags the overdue ones.
//!
//! This skeleton ships no checks. [`default_checks`] is empty, and the checks of the design's rows 1 to 8 are
//! later jobs that add themselves to it.
//!
//! The sweep also reads Athena's own spec. Every active requirement in `docs/spec.toml` records a mechanism, an
//! owner and a measure. [`Athena::spec_review`] (a review pass, see [`Athena::review_passes`], because a [`Check`]
//! cannot reach the records or the queue) queues one request through [`StewardRequest`] for each active
//! requirement whose mechanism is `none` or whose measure is blank or names a label `athena status` does not
//! print ([`measure_problem`]). The request is high when the requirement is about speed ([`is_speed`]), and an
//! open request that holds the line `Spec requirement: <id>` stops a second one. At most
//! [`SPEC_REQUESTS_PER_PASS`] are queued in one pass, speed first. `athena status` lists the same gaps
//! ([`spec_gaps`]).
//!
//! Hestia also reviews errors across jobs (`docs/design/hestia.md`, "Error review"). Every failure gets an
//! introspection with a cause, but each one is written about one job. After an introspection is recorded,
//! [`Athena::error_review`] groups it with the earlier ones by cause ([`cause_key`]). A cause that has occurred
//! in two or more distinct jobs, and that no open or landed request cites, gets one high-priority root-cause request,
//! built by [`crate::steward::StewardRequest`] and citing the introspection ids. [`repeated_causes`] lists the
//! repeated causes for `athena status`.
//!
//! The review only reacts to a new introspection, so causes that repeated before it existed have no request.
//! The backfill ([`Athena::error_review_backfill`]) sweeps them. Athena runs it at the start of every drain and in
//! each sweep ([`Athena::review_passes`]); `athena hestia review --backfill` runs it on request: a cause whose stage is
//! switched off in `athena.toml` is dormant and skipped; an open request that already addresses a cause is
//! re-queued through the same builder and the old one closed; any other cause gets a new root-cause request.
//! The five highest-ranked causes (most occurrences, then most recent) are high and the rest normal. A second
//! run finds every cause covered and changes nothing.
//!
//! Hestia never queues a request above high: urgent is for the person running Athena (`athena priority`, or a
//! `Priority: urgent` line they write). A cause whose request a person closed with `athena close` is not queued
//! again until a new introspection of that cause is recorded after the close ([`addressing_request`]); a
//! request Hestia closed to queue it again, or one that was rejected or failed, does not hold the cause back.
//!
//! A rule or check can be wrong. An introspection of kind `relax` or `remove` names a gate in its prevention
//! text (`[gate: NAME] ...`, see [`crate::gate_counts`]) and is accepted only for a gate that a later try overruled
//! at least once. It is not a cause: [`cause_key`] leaves it out, so it never joins a root-cause request.
//! [`Athena::gate_review`] queues one normal-priority request for it ([`gate_request`]) that cites the gate's
//! end and disagreement counts as the evidence; an open request that holds the line `Gate under review: NAME.`
//! stops a second one for the same gate ([`open_gate_request`]). Nothing is relaxed or removed by Athena: the
//! request asks for a reviewed change.
//!
//! The throughput review (`docs/design/hestia.md`, "Throughput review") asks a different question: not what
//! failed but what the pipeline waits on. It runs in [`Athena::review_passes`] when [`throughput_due`] says so:
//! every [`crate::code_health::REVIEW_EVERY`] landings, or when the median minutes per landed job grew by more
//! than 20% since the previous review, or when wasted work rose by more than 20% against the figures the
//! previous review stored: attempts per landed change, money spent on attempts that did not land, or the share
//! of changes that landed on the first attempt ([`waste_rises`]). A strong model is given the status
//! throughput section ([`crate::status::Throughput`]) and `docs/design`, and names the biggest bottleneck and
//! the architectural change that removes it, with the minutes it expects that change to save per landed job.
//! An open earlier design request does not stop the review: the review notes it and tells the model about it.
//! The proposal is queued as a design request through [`StewardRequest`], at a priority set by those minutes
//! ([`priority_for_minutes`]), only when a measure got worse by more than the noise between recent windows
//! ([`request_gate`]); otherwise it is recorded without a request and the note says why. The prediction is
//! kept in an `athena.throughput-review/1` record. A later review compares the minutes measured after a
//! change that cites the review with the prediction ([`throughput_followups`]).
//!
//! Every throughput review run also compares the latest median minutes per landed job with the previous
//! review's, and does so even while a design request is open. The previous per-stage medians are rebuilt from
//! the job records ([`previous_stage_medians`]), and the stages whose median minutes grew are named, largest
//! first ([`stage_growth`]), in the review's note and in the line `athena drain` prints. When the median grew
//! by more than 25% ([`SLOWDOWN_RATIO`], [`is_slowdown`]), the review also queues, without asking the model, a
//! high-priority request that names the stage that grew most with its measured minutes
//! ([`slowdown_request`]). Its title starts with [`SLOWDOWN_MARKER`], and an open request with that marker
//! holds back another. A rise between 20% and 25% still runs a review and names the stages, but queues no
//! slowdown request. With 60 or more landings the slowdown request is also queued only when the rise over the
//! previous review's median is more than the noise between recent windows ([`slowdown_beyond_noise`]).
//!
//! The sweep also measures the build. [`Athena::hestia_sweep_with`] runs [`Athena::measure_build_pass`], which
//! calls [`Athena::measure_build`] (two cold workspace builds in separate worktrees, committed as an
//! `athena.build-measurement/1` record) when no measurement record exists, and again whenever the content of
//! `Cargo.lock` differs from the one the latest measurement built ([`measure_build_due`]). It is in the sweep
//! and not in [`Athena::review_passes`], so `athena drain` never waits for two cold builds, and it is not a
//! [`Check`], which cannot reach the records. A repository with no `Cargo.lock` is never measured.

use crate::capability::{JobConfig, ToolGrant};
use crate::commits::{NewFile, message};
use crate::disk::{CargoBuild, CargoBuilder, GB, cargo_lock_blob};
use crate::gate_counts::{GateCount, gate_counts, gate_of_prevention, prevention_without_gate};
use crate::plan::{Improvement, Priority};
use crate::state::format_bytes;
use crate::status::{
    Bar, LandedJob, Measure, ReadingRuleStats, StageMinutes, THROUGHPUT_WINDOW, Throughput,
    landed_jobs, landed_jobs_among_first, median,
};
use crate::steward::StewardRequest;
use crate::store::{JobRecords, JobState, Records};
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_agent::env::Side;
use athena_context::briefs::plain_boundary_text;
use athena_context::{CONTRACT, Component, Composed, compose};
use athena_data::records::{
    AnyRecord, BuildMeasurement, CodeHealthBaseline, DutyPass, HestiaCheckResult,
    IntrospectionRecord, IntrospectionStatus, Invocation, InvocationOutcome, OutcomeState,
    PreventionKind, Role, ThroughputFollowup, ThroughputProposal, ThroughputReview,
    ThroughputTrigger, WasteFigures,
};
use athena_data::{JobId, RecordId};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

pub use athena_data::records::HestiaCheckResult as CheckResult;
pub use athena_data::records::HestiaFinding as Finding;
pub use athena_data::records::HestiaFindingState as FindingState;
pub use athena_data::records::HestiaSweep as Sweep;

/// What a check saw when it ran without error.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckOutput {
    /// How many things it looked at (branches, requests, docs).
    pub examined: u32,
    pub findings: Vec<Finding>,
}

/// One Hestia check: a fact it looks at, and what is wrong with it.
pub trait Check {
    /// A short stable id, such as `leftover-branches`. Unique within a sweep.
    fn id(&self) -> &str;
    /// Looks at the system. `Err` means the check could not run, which is itself reported.
    fn run(&self, now: Timestamp) -> Result<CheckOutput>;
}

/// Whether a build measurement is due: none exists, or the git blob id of `Cargo.lock` now (`lock_hash`) is not
/// the one the latest measurement built. The content of `Cargo.lock` decides, so a landing that leaves it
/// alone does not cause another pair of cold builds.
pub fn measure_build_due(latest: Option<&BuildMeasurement>, lock_hash: &str) -> bool {
    latest.is_none_or(|m| m.cargo_lock_hash != lock_hash)
}

/// The checks a sweep runs when none are given. Empty in the skeleton; later jobs register theirs here.
pub fn default_checks() -> Vec<Box<dyn Check>> {
    Vec::new()
}

/// Runs every check, in order, and reports on all of them. A check that fails, or that returns a finding with
/// no subject or no evidence, becomes one open finding for that check, and the sweep goes on. Whatever check id
/// a check puts on its findings, they are filed under the check that returned them.
pub fn run_sweep(checks: &[Box<dyn Check>], id: RecordId, at: Timestamp) -> Sweep {
    let mut results = Vec::new();
    let mut findings = Vec::new();
    for check in checks {
        let (result, found) = settle(check.id(), check.run(at));
        results.push(result);
        findings.extend(found);
    }
    Sweep {
        id,
        at,
        checks: results,
        findings,
        duty_pass: None,
    }
}

/// The result and findings a check or duty `check_id` leaves in a sweep, from what it returned. A finding with
/// no subject or no evidence, or an error, makes the check errored with one open finding that says it failed to
/// run. Whatever check id a finding carries, it is filed under `check_id`.
fn settle(check_id: &str, outcome: Result<CheckOutput>) -> (CheckResult, Vec<Finding>) {
    let outcome = outcome.and_then(|output| {
        if output
            .findings
            .iter()
            .any(|f| f.subject.trim().is_empty() || f.evidence.trim().is_empty())
        {
            bail!("a finding came back with no subject or no evidence");
        }
        Ok(output)
    });
    match outcome {
        Ok(output) => (
            CheckResult {
                check: check_id.to_string(),
                examined: output.examined,
                count: output.findings.len() as u32,
                errored: false,
            },
            output
                .findings
                .into_iter()
                .map(|f| Finding {
                    check: check_id.to_string(),
                    ..f
                })
                .collect(),
        ),
        Err(error) => (
            HestiaCheckResult {
                check: check_id.to_string(),
                examined: 0,
                count: 1,
                errored: true,
            },
            vec![Finding {
                check: check_id.to_string(),
                subject: check_id.to_string(),
                evidence: format!("the check failed to run: {error:#}"),
                age_seconds: 0,
                state: FindingState::Open,
            }],
        ),
    }
}

// ---------------------------------------------------------------------------------------------------------
// Recurring duties (`docs/design/hestia.md`, "Recurring duties")

/// When a duty is due.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cadence {
    /// At every drain start.
    DrainStart,
    /// After every landing.
    EveryLanding,
    /// After every N landings since the last run.
    EveryLandings(u32),
}

impl Cadence {
    /// The cadence in the words `docs/design/hestia.md` uses in its duty table.
    pub fn describe(self) -> String {
        match self {
            Cadence::DrainStart => "every drain start".into(),
            Cadence::EveryLanding => "every landing".into(),
            Cadence::EveryLandings(n) => format!("every {n} landings"),
        }
    }
}

/// How a duty runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum How {
    /// The duty's runner does the work ([`DutyRunner`]).
    Runner,
    /// The work is part of the code-health check, which queues the review when its own landing counter says so.
    /// The duty's last run and next due point are read from the code-health baselines.
    WithCodeHealth,
}

/// One recurring duty and its owner, Hestia's sweep.
#[derive(Debug, Clone, Copy)]
pub struct Duty {
    /// A short stable id: the check name its runs are recorded under.
    pub id: &'static str,
    pub name: &'static str,
    pub cadence: Cadence,
    pub how: How,
    /// Where it runs, in words.
    pub runs_in: &'static str,
}

/// The id of the keep-rule threshold check.
pub const KEEP_RULE_CHECK_ID: &str = "keep-rule";

/// Every recurring duty, in the order of the table in `docs/design/hestia.md` section 13.
pub const DUTIES: [Duty; 6] = [
    Duty {
        id: "error-review",
        name: "Error-review backlog pass",
        cadence: Cadence::DrainStart,
        how: How::Runner,
        runs_in: "Athena::error_review_backfill, from the drain start and the sweep",
    },
    Duty {
        id: "evidence-labels",
        name: "Evidence-label derivation",
        cadence: Cadence::DrainStart,
        how: How::Runner,
        runs_in: "Athena::derive_labels, from the drain start and the sweep",
    },
    Duty {
        id: "code-health",
        name: "Code-health check",
        cadence: Cadence::EveryLanding,
        how: How::Runner,
        runs_in: "Athena::code_health_check, from the landing hook and the sweep",
    },
    Duty {
        id: "design-review",
        name: "Periodic design review",
        cadence: Cadence::EveryLandings(crate::code_health::REVIEW_EVERY),
        how: How::WithCodeHealth,
        runs_in: "the code-health check, which queues the review",
    },
    Duty {
        id: KEEP_RULE_CHECK_ID,
        name: "Keep-rule threshold check",
        cadence: Cadence::EveryLandings(10),
        how: How::Runner,
        runs_in: "the sweep and the landing hook, over the records",
    },
    Duty {
        id: READING_RULES_CHECK_ID,
        name: "Reading-rule refusal-rate check",
        cadence: Cadence::EveryLandings(5),
        how: How::Runner,
        runs_in: "the sweep and the landing hook, over the records",
    },
];

pub use athena_data::records::DutyOccasion as Occasion;

/// What a pass over the duties knows when it decides: where it was started from, how many landings stand on the
/// branch and the time. Passed in, so a test needs no landed commits and no clock.
#[derive(Debug, Clone, Copy)]
pub struct DutyContext {
    pub occasion: Occasion,
    pub landings: u32,
    pub now: Timestamp,
}

/// What a duty's runner did: the check result and the lines to tell the user.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DutyOutput {
    pub check: CheckOutput,
    pub lines: Vec<String>,
}

/// Does the work of a duty.
pub trait DutyRunner {
    /// Runs `duty`. `Err` means it could not run, which is recorded as an errored check and retried.
    fn run(&self, duty: &Duty, ui: &dyn Ui, now: Timestamp) -> Result<DutyOutput>;
}

/// What a pass over the duties did.
#[derive(Debug, Clone, Default)]
pub struct DutyReport {
    pub results: Vec<CheckResult>,
    pub findings: Vec<Finding>,
    pub lines: Vec<String>,
}

/// What the records say about the past runs of one duty.
#[derive(Debug, Clone, Copy, Default)]
struct PastRuns {
    /// The newest run that did not fail: when, and the landing total then.
    ok: Option<(Timestamp, u32)>,
    /// When the newest attempt was, if it failed.
    failed_at: Option<Timestamp>,
}

/// The past runs of duty `id` in the duty passes among `sweeps`.
fn past_runs(sweeps: &[Sweep], id: &str) -> PastRuns {
    let mut attempts: Vec<(&Sweep, &CheckResult)> = sweeps
        .iter()
        .filter(|s| s.duty_pass.is_some())
        .filter_map(|s| s.checks.iter().find(|c| c.check == id).map(|c| (s, c)))
        .collect();
    attempts.sort_by(|a, b| (a.0.at, &a.0.id).cmp(&(b.0.at, &b.0.id)));
    PastRuns {
        ok: attempts
            .iter()
            .rev()
            .find(|(_, c)| !c.errored)
            .and_then(|(s, _)| s.duty_pass.map(|p| (s.at, p.landings))),
        failed_at: attempts
            .last()
            .filter(|(_, c)| c.errored)
            .map(|(s, _)| s.at),
    }
}

/// Whether a pass from `occasion` may run `duty` at all. A drain start runs the drain-start duties, a landing
/// the landing duties, and a sweep every duty that has a runner.
fn eligible(duty: &Duty, occasion: Occasion) -> bool {
    if duty.how != How::Runner {
        return false;
    }
    match occasion {
        Occasion::Sweep => true,
        Occasion::DrainStart => duty.cadence == Cadence::DrainStart,
        Occasion::Landing => duty.cadence != Cadence::DrainStart,
    }
}

/// How many landings must stand beyond the last run before a duty with `cadence` is due again; None for a
/// drain-start duty.
fn landings_needed(cadence: Cadence) -> Option<u32> {
    match cadence {
        Cadence::DrainStart => None,
        Cadence::EveryLanding => Some(1),
        Cadence::EveryLandings(n) => Some(n.max(1)),
    }
}

/// Whether a pass with `ctx` runs `duty`, given its past runs. A duty that never ran, or whose newest attempt
/// failed, is due at every pass that may run it. Otherwise a drain-start duty is due at a drain start, and a
/// landing duty once its landings since the last run reach its cadence.
fn is_due(duty: &Duty, past: &PastRuns, ctx: &DutyContext) -> bool {
    if !eligible(duty, ctx.occasion) {
        return false;
    }
    let Some((_, recorded)) = past.ok else {
        return true;
    };
    if past.failed_at.is_some() {
        return true;
    }
    match landings_needed(duty.cadence) {
        None => ctx.occasion == Occasion::DrainStart,
        Some(n) => ctx.landings.saturating_sub(recorded) >= n,
    }
}

/// One duty as `athena status` shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DutyStanding {
    pub id: String,
    pub name: String,
    /// The cadence in words ([`Cadence::describe`]).
    pub cadence: String,
    pub runs_in: String,
    /// When it last ran without failing; None when it never has (for the design review: was never queued).
    pub last_run: Option<Timestamp>,
    /// When its newest attempt failed; None when that attempt did not fail.
    pub last_failed_at: Option<Timestamp>,
    /// When the duty is next due, in words.
    pub next_due: String,
    /// True when the next sweep would run it: it never ran, its last attempt failed, or its cadence has passed.
    /// A drain-start duty has no time limit: it is overdue only when never run or failed.
    pub overdue: bool,
}

fn more_landings(n: u32) -> String {
    format!("after {n} more landing{}", if n == 1 { "" } else { "s" })
}

/// The standing of every duty in `table`: last run, next due point and whether it is overdue. `sweeps` are the
/// recorded sweeps, `baselines` the code-health baselines that count the design review, and `ctx` gives the
/// landing total now (its occasion is ignored: overdue means the next sweep would run it).
pub fn duty_standings(
    table: &[Duty],
    sweeps: &[Sweep],
    baselines: &[CodeHealthBaseline],
    ctx: &DutyContext,
) -> Vec<DutyStanding> {
    let ctx = DutyContext {
        occasion: Occasion::Sweep,
        ..*ctx
    };
    table
        .iter()
        .map(|duty| {
            let mut standing = DutyStanding {
                id: duty.id.to_string(),
                name: duty.name.to_string(),
                cadence: duty.cadence.describe(),
                runs_in: duty.runs_in.to_string(),
                last_run: None,
                last_failed_at: None,
                next_due: String::new(),
                overdue: false,
            };
            if duty.how == How::WithCodeHealth {
                let every = landings_needed(duty.cadence).unwrap_or(1);
                standing.last_run = baselines
                    .iter()
                    .filter(|b| b.review_queued)
                    .map(|b| b.at)
                    .max();
                match CodeHealthBaseline::latest(baselines) {
                    None => {
                        standing.overdue = true;
                        standing.next_due =
                            "due now (no code-health baseline yet; the code-health check sets it)"
                                .into();
                    }
                    Some(baseline) => {
                        let since = ctx.landings.saturating_sub(baseline.landings);
                        standing.overdue = since >= every;
                        standing.next_due = if standing.overdue {
                            format!("due now ({since} landings since the last baseline)")
                        } else {
                            more_landings(every - since)
                        };
                    }
                }
                return standing;
            }
            let past = past_runs(sweeps, duty.id);
            standing.last_run = past.ok.map(|(at, _)| at);
            standing.last_failed_at = past.failed_at;
            standing.overdue = is_due(duty, &past, &ctx);
            standing.next_due = match (past.ok, past.failed_at) {
                (None, None) => "due now (never run)".into(),
                (_, Some(_)) => "due now (last attempt failed)".into(),
                (Some((_, recorded)), None) => match landings_needed(duty.cadence) {
                    None => "at the next drain start".into(),
                    Some(n) => {
                        let since = ctx.landings.saturating_sub(recorded);
                        if since >= n {
                            format!("due now ({since} landings since the last run)")
                        } else if n == 1 {
                            "at the next landing".into()
                        } else {
                            more_landings(n - since)
                        }
                    }
                },
            };
            standing
        })
        .collect()
}

/// The line shown when duty `duty` could not run.
fn duty_error_line(duty: &Duty, error: &anyhow::Error) -> String {
    if duty.id == "code-health" {
        return format!("warning: could not check code health: {error:#}");
    }
    let label = match duty.id {
        "error-review" => "backfill",
        "evidence-labels" => "evidence labels",
        other => other,
    };
    format!("hestia     {label}: could not run: {error:#}")
}

/// The summary line of the error review backfill.
fn backfill_line(r: &BackfillReport) -> String {
    format!(
        "hestia     backfill: {} repeated cause(s) · queued {} · re-queued {} · priority set {} · skipped {}",
        r.lines.len(),
        r.count(BackfillAction::Queued),
        r.count(BackfillAction::Requeued),
        r.count(BackfillAction::Reprioritised),
        r.count(BackfillAction::Skipped)
    )
}

/// The runner that does the real work of each duty.
pub struct AthenaDuties<'a> {
    pub athena: &'a Athena,
}

impl std::fmt::Debug for AthenaDuties<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AthenaDuties")
    }
}

impl DutyRunner for AthenaDuties<'_> {
    fn run(&self, duty: &Duty, ui: &dyn Ui, now: Timestamp) -> Result<DutyOutput> {
        let athena = self.athena;
        match duty.id {
            "error-review" => {
                let r = athena.error_review_backfill(ui)?;
                Ok(DutyOutput {
                    check: CheckOutput {
                        examined: r.lines.len() as u32,
                        findings: Vec::new(),
                    },
                    lines: vec![backfill_line(&r)],
                })
            }
            "evidence-labels" => {
                let s = athena.derive_labels()?;
                Ok(DutyOutput {
                    check: CheckOutput {
                        examined: (s.derived + s.already_derived + s.reviewed + s.no_evidence)
                            as u32,
                        findings: Vec::new(),
                    },
                    lines: vec![format!(
                        "hestia     evidence labels: {} derived · {} already derived · {} reviewed by a person · {} without evidence",
                        s.derived, s.already_derived, s.reviewed, s.no_evidence
                    )],
                })
            }
            "code-health" => {
                let report = athena.code_health_check(now)?;
                Ok(DutyOutput {
                    check: CheckOutput {
                        examined: report.files_measured as u32,
                        findings: Vec::new(),
                    },
                    lines: crate::code_health::report_lines(&report),
                })
            }
            KEEP_RULE_CHECK_ID => {
                let records = athena.current_records()?;
                Ok(DutyOutput {
                    check: keep_rule_output(&records),
                    lines: Vec::new(),
                })
            }
            READING_RULES_CHECK_ID => {
                let head = athena.head()?;
                Ok(DutyOutput {
                    check: athena.reading_rules_check(&head).run(now)?,
                    lines: Vec::new(),
                })
            }
            other => bail!("duty {other} is not yet automated"),
        }
    }
}

/// What the keep-rule threshold check sees: one finding when the rule's verdict is a breach or says to turn
/// planning off. The thresholds are the rule's own ([`crate::status::KEEP_RULE_TARGET_SHARE`] and its
/// neighbours); this check reads the verdict and changes nothing.
fn keep_rule_output(records: &Records) -> CheckOutput {
    let Some(rule) = crate::status::keep_rule(records) else {
        return CheckOutput::default();
    };
    let mut findings = Vec::new();
    if matches!(
        rule.verdict,
        crate::status::KeepVerdict::Breach | crate::status::KeepVerdict::TurnOff
    ) {
        findings.push(Finding {
            check: KEEP_RULE_CHECK_ID.into(),
            subject: "job-planning keep rule".into(),
            evidence: format!(
                "the verdict is {:?}: {} of {} planned job(s) landed ({:.0}%, baseline {:.0}%){}",
                rule.verdict,
                rule.landed,
                rule.planned,
                rule.share * 100.0,
                rule.baseline_share * 100.0,
                rule.cost_per_landed_usd
                    .map(|c| format!(", ${c:.2} per landed job"))
                    .unwrap_or_default()
            ),
            age_seconds: 0,
            state: FindingState::Open,
        });
    }
    CheckOutput {
        examined: rule.planned as u32,
        findings,
    }
}

// ---------------------------------------------------------------------------------------------------------
// Reading rule refusals

/// A reading rule is flagged when it refused more than this share, in percent, of the first readings in the
/// window ([`crate::status::READING_WINDOW`]). A rule at exactly this share is not flagged.
pub const REFUSAL_FLAG_PERCENT: usize = 30;

/// The fewest first readings the window must hold before any rule is flagged. With fewer, one refusal moves a
/// rule's share by ten points or more, so the share says nothing; the check then looks at nothing and says
/// nothing, and in particular does not report the rules as healthy.
pub const MIN_FIRST_READINGS: usize = 10;

/// The id of the check that flags reading rules that refuse too many first readings.
pub const READING_RULES_CHECK_ID: &str = "reading-rule-refusals";

/// The findings for the reading rules that refused more than [`REFUSAL_FLAG_PERCENT`] of the first readings in
/// `stats`, one for each, most refusals first. A finding names the rule and quotes the refusal text it gave most
/// often. No finding is made when the window holds fewer than [`MIN_FIRST_READINGS`] first readings.
///
/// A refused first reading costs a reader call and a second reading, so a rule that refuses often is a rule
/// whose text the reader brief does not teach, or a rule that is too broad; the finding says to fix one of them.
pub fn reading_rule_findings(stats: &ReadingRuleStats) -> Vec<Finding> {
    if stats.first_readings < MIN_FIRST_READINGS {
        return Vec::new();
    }
    stats
        .rules
        .iter()
        .filter(|r| r.first_refused * 100 > stats.first_readings * REFUSAL_FLAG_PERCENT)
        .map(|r| Finding {
            check: READING_RULES_CHECK_ID.into(),
            subject: r.rule.clone(),
            evidence: format!(
                "{} refused {} of the last {} first readings ({:.1}%), more than {REFUSAL_FLAG_PERCENT}%; \
                 its most common refusal text ({} time(s)): \"{}\". Fix the reader's brief so the reader meets \
                 this rule the first time, or fix the rule if it refuses what it should not.",
                r.rule,
                r.first_refused,
                stats.first_readings,
                r.first_refused as f64 * 100.0 / stats.first_readings as f64,
                r.top_text_count,
                one_line(&r.top_text)
            ),
            age_seconds: 0,
            state: FindingState::Open,
        })
        .collect()
}

/// The Hestia check over [`ReadingRuleStats`]. A [`Check`] cannot reach the records, so the stats are computed
/// first and handed in. A window below [`MIN_FIRST_READINGS`] first readings, or with no recorded reading
/// attempt, examines nothing and finds nothing.
#[derive(Debug)]
pub struct ReadingRulesCheck {
    stats: std::result::Result<Option<ReadingRuleStats>, String>,
}

impl ReadingRulesCheck {
    pub fn new(stats: std::result::Result<Option<ReadingRuleStats>, String>) -> Self {
        Self { stats }
    }
}

impl Check for ReadingRulesCheck {
    fn id(&self) -> &str {
        READING_RULES_CHECK_ID
    }

    fn run(&self, _: Timestamp) -> Result<CheckOutput> {
        let stats = match &self.stats {
            Err(error) => bail!("the records could not be read: {error}"),
            Ok(Some(stats)) if stats.first_readings >= MIN_FIRST_READINGS => stats,
            Ok(_) => return Ok(CheckOutput::default()),
        };
        Ok(CheckOutput {
            examined: stats.first_readings as u32,
            findings: reading_rule_findings(stats),
        })
    }
}

// ---------------------------------------------------------------------------------------------------------
// Error review

/// The fewest distinct jobs a cause must occur in before it is a repeat.
pub const REPEAT_JOBS: usize = 2;

/// The longest cause text shown in a title or a request line.
const CAUSE_CAP: usize = 240;

/// The request that addresses a repeated cause, and where it stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AddressingRequest {
    pub job: String,
    /// The request's state: `queued`, `landed`, and so on (`JobState::name`).
    pub state: String,
    /// True when the request landed, so the cause is no longer open.
    pub landed: bool,
}

/// A cause that occurred in two or more distinct jobs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RepeatedCause {
    /// The grouping key: see [`cause_key`].
    pub key: String,
    /// The failure class's name and id, when the introspections resolve to a registry class.
    pub class: Option<String>,
    /// The cause as the earliest introspection of the group words it, on one line.
    pub cause: String,
    /// The introspections of the group, oldest first.
    pub introspections: Vec<String>,
    /// The distinct jobs they were about, in order of first occurrence.
    pub jobs: Vec<String>,
    /// An open or landed request that cites one of the introspections; none when no request addresses it yet.
    pub request: Option<AddressingRequest>,
}

impl RepeatedCause {
    /// How many introspections record this cause.
    pub fn count(&self) -> usize {
        self.introspections.len()
    }

    /// Whether the cause still needs work: no request addresses it, or the request has not landed.
    pub fn is_open(&self) -> bool {
        !self.request.as_ref().is_some_and(|r| r.landed)
    }
}

/// The failure class an introspection belongs to: the class its record names when the class registry holds
/// it, else the class Jev chose when that is a registry class (`new` and unknown ids are not).
fn class_of(records: &Records, i: &IntrospectionRecord) -> Option<RecordId> {
    let known = |id: &RecordId| records.classes.iter().any(|c| &c.id == id);
    i.failure_class.clone().filter(known).or_else(|| {
        let choice = i.jev.as_ref()?.class.as_ref()?.choice.as_str();
        RecordId::parse(choice).ok().filter(known)
    })
}

/// The one line of text of a cause, cut to [`CAUSE_CAP`] characters.
fn one_line(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= CAUSE_CAP {
        return flat;
    }
    let cut: String = flat.chars().take(CAUSE_CAP).collect();
    format!("{cut}...")
}

/// The key two introspections share when they have the same cause: the failure class they resolve to
/// ([`class_of`], through the class registry or Jev's class answer), and, for one that resolves to none, its
/// cause text with case and punctuation ignored ([`crate::introspect::normalise`]). `None` for an
/// introspection that is not a case: one that failed, has no job, or has no cause, or that proposes relaxing or
/// removing a gate. The last is a challenge to a check, reviewed apart from the root-cause requests
/// ([`Athena::gate_review`]); in a cause group it would hide a code-check of the same cause from the backfill.
pub fn cause_key(records: &Records, i: &IntrospectionRecord) -> Option<String> {
    let cause = i.cause.as_deref()?;
    if i.status != IntrospectionStatus::Recorded
        || i.job.is_none()
        || i.prevention_kind.is_some_and(PreventionKind::loosens)
    {
        return None;
    }
    if let Some(class) = class_of(records, i) {
        return Some(format!("class:{class}"));
    }
    let text = crate::introspect::normalise(cause);
    (!text.is_empty()).then(|| format!("cause:{text}"))
}

/// The request that addresses a group of introspections: an open or landed request job whose text cites one of
/// their ids. An open one is preferred to a landed one. A request that was rejected, failed or closed does not
/// address the cause, so the review may queue another. The exception is a request a person closed with
/// `athena close` ([`closed_by_person`]): it keeps addressing the cause until an introspection of the cause is
/// recorded after that close, so the review does not queue again what a person turned down. An open or landed
/// request is preferred to such a closed one.
fn addressing_request(records: &Records, introspections: &[String]) -> Option<AddressingRequest> {
    let mut found: Option<AddressingRequest> = None;
    let mut turned_down: Option<AddressingRequest> = None;
    let newest = records
        .introspections
        .iter()
        .filter(|i| introspections.contains(&i.id.to_string()))
        .map(|i| i.at)
        .max();
    for job in records.request_jobs() {
        let state = job.state();
        let landed = state == JobState::Closed(OutcomeState::Landed);
        let cites = job
            .request
            .as_ref()
            .is_some_and(|r| introspections.iter().any(|id| r.text.contains(id.as_str())));
        if !cites {
            continue;
        }
        if matches!(state, JobState::Closed(_)) && !landed {
            let closed_at = job.outcome.as_ref().map(|o| o.at);
            if closed_by_person(job)
                && closed_at.is_some_and(|at| newest.is_none_or(|n| n <= at))
                && turned_down.is_none()
            {
                turned_down = Some(AddressingRequest {
                    job: job.id().as_str().to_string(),
                    state: state.name().to_string(),
                    landed: false,
                });
            }
            continue;
        }
        if found.as_ref().is_none_or(|f| f.landed && !landed) {
            found = Some(AddressingRequest {
                job: job.id().as_str().to_string(),
                state: state.name().to_string(),
                landed,
            });
        }
    }
    found.or(turned_down)
}

/// The words a reason starts with when Hestia itself closed a request to queue it again ([`Athena::close`] is
/// called with them in the backfill). Any other close of a request that was waiting is a person's.
const REQUEUE_CLOSE_REASON: &str = "re-queued as ";

/// Whether a person closed the request with `athena close`: it was dropped, declined or abandoned, and the
/// reason is not the one Hestia writes when it closes a request to queue it in its place.
fn closed_by_person(job: &JobRecords) -> bool {
    matches!(
        job.state(),
        JobState::Closed(OutcomeState::Dropped | OutcomeState::Declined | OutcomeState::Abandoned)
    ) && job.outcome.as_ref().is_some_and(|o| {
        !o.reason
            .strip_prefix("closed: ")
            .unwrap_or(&o.reason)
            .starts_with(REQUEUE_CLOSE_REASON)
    })
}

/// The causes that occurred in [`REPEAT_JOBS`] or more distinct jobs, with their introspections and the
/// request that addresses each. Ordered by the time of the earliest introspection of each cause.
pub fn repeated_causes(records: &Records) -> Vec<RepeatedCause> {
    let mut groups: Vec<(String, Vec<&IntrospectionRecord>)> = Vec::new();
    for i in &records.introspections {
        let Some(key) = cause_key(records, i) else {
            continue;
        };
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, members)) => members.push(i),
            None => groups.push((key, vec![i])),
        }
    }
    groups
        .into_iter()
        .filter_map(|(key, members)| {
            let mut jobs: Vec<String> = Vec::new();
            for job in members.iter().filter_map(|i| i.job.as_ref()) {
                if !jobs.iter().any(|j| j == job.as_str()) {
                    jobs.push(job.as_str().to_string());
                }
            }
            if jobs.len() < REPEAT_JOBS {
                return None;
            }
            let introspections: Vec<String> = members.iter().map(|i| i.id.to_string()).collect();
            let class = class_of(records, members[0]).map(|id| {
                let name = records
                    .classes
                    .iter()
                    .find(|c| c.id == id)
                    .map_or("unknown class", |c| c.name.as_str());
                format!("{name} ({id})")
            });
            Some(RepeatedCause {
                request: addressing_request(records, &introspections),
                cause: one_line(members[0].cause.as_deref().unwrap_or("")),
                key,
                class,
                introspections,
                jobs,
            })
        })
        .collect()
}

/// Whether `id` is a request job still waiting to be worked: queued or proposed.
fn is_waiting_request(records: &Records, id: &str) -> bool {
    records.jobs.get(id).is_some_and(|j| {
        j.request.is_some() && matches!(j.state(), JobState::Queued | JobState::Proposed)
    })
}

/// Athena's own improvement requests among the open ones, for the drain's guaranteed turn
/// ([`Improvement`], [`crate::plan::Plan::build_improving`]). Requests are found by what the records link, never
/// by their title. The kinds:
///
/// - open requests the Steward queued (channel `steward`);
/// - the open request that addresses a repeated cause, filed by the error review ([`repeated_causes`] links it
///   by the introspection ids its text cites). Its entry in `cause_jobs` is the number of distinct jobs the
///   cause is recorded in; with several causes the largest count counts;
/// - the design request of a throughput review: the `request_job` of a recorded proposal;
/// - the open request that holds the line `Spec requirement: <id>`, filed by the spec review.
///
/// A request is linked by what it cites, not by who wrote it: a person's own request that cites a repeated
/// cause's introspection ids, or holds a `Spec requirement:` line, counts as an improvement request too.
///
/// Not covered: the throughput slowdown request ([`slowdown_request`]). Nothing in the records links it to the
/// review (only its title prefix and the free text of the review's note), so it is not listed, and a title match
/// is not used. `since_turn` and `tried` are left empty: they are the drain's own count.
pub fn improvement_of(records: &Records) -> Improvement {
    let mut items: BTreeSet<String> = crate::steward::improvement_requests(records)
        .into_iter()
        .collect();
    let mut cause_jobs: BTreeMap<String, usize> = BTreeMap::new();
    for cause in repeated_causes(records) {
        let Some(request) = cause.request.filter(|r| !r.landed) else {
            continue;
        };
        if !is_waiting_request(records, &request.job) {
            continue;
        }
        let count = cause_jobs.entry(request.job.clone()).or_insert(0);
        *count = (*count).max(cause.jobs.len());
        items.insert(request.job);
    }
    for job in records
        .throughput_reviews
        .iter()
        .filter_map(|r| r.proposal.as_ref()?.request_job.as_ref())
        .filter(|job| is_waiting_request(records, job.as_str()))
    {
        items.insert(job.as_str().to_string());
    }
    for job in open_spec_requests(records, None) {
        if is_waiting_request(records, job.id().as_str()) {
            items.insert(job.id().as_str().to_string());
        }
    }
    Improvement {
        items: items.into_iter().collect(),
        cause_jobs,
        ..Improvement::default()
    }
}

/// The text of the high-priority root-cause request for a repeated cause, built with [`StewardRequest`]. High is
/// the most Hestia queues: urgent is for a person. No body line starts like a bullet: the builder refuses one,
/// since `athena plan` would read it as acceptance.
pub fn root_cause_request(cause: &RepeatedCause, records: &Records) -> Result<String, String> {
    root_cause_request_at(cause, records, Priority::High)
}

/// [`root_cause_request`] at the given priority: the one place the text of a root-cause request is built.
fn root_cause_request_at(
    cause: &RepeatedCause,
    records: &Records,
    priority: Priority,
) -> Result<String, String> {
    // The cause and the prevention text come from earlier introspections, which a person or a model wrote,
    // so a word the admission scan rejects is reworded before it enters the request.
    let cause_text = plain_boundary_text(&cause.cause)?;
    let mut body = String::from(
        "The same cause has produced failures in more than one job. Find the cause where it originates and fix \
         it there, so the whole class is prevented; do not work around one instance.\n\n\
         First ask what information the failing stage lacked, and change the code so that information is \
         given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. \
         Prefer this over any rule that refuses work.\n\n\
         A refusal rule is allowed only when giving the stage that information cannot prevent the failure. \
         A request that adds one must state the expected share of past accepted readings the rule would \
         refuse.\n\n",
    );
    if let Some(class) = &cause.class {
        body.push_str(&format!("Failure class: {}\n", plain_boundary_text(class)?));
    }
    body.push_str(&format!("Cause: {cause_text}\n"));
    body.push_str(&format!(
        "Seen {} time(s) in {} jobs. The introspections that record it:\n",
        cause.count(),
        cause.jobs.len()
    ));
    for id in &cause.introspections {
        let i = records.introspections.iter().find(|i| i.id.as_str() == id);
        let job = i.and_then(|i| i.job.as_ref()).map_or("?", JobId::as_str);
        let prevention = i
            .and_then(|i| i.prevention.as_deref())
            .map(one_line)
            .unwrap_or_default();
        body.push_str(&format!(
            "Introspection {id} (job {job}); prevention it proposed: {prevention}\n"
        ));
    }
    let acceptance = [
        "The root cause is named in the change description and fixed where it originates, not at one instance"
            .to_string(),
        "A test fails without the change and passes with it, for the cause and not only for one job".to_string(),
        "fmt, clippy and test pass".to_string(),
    ];
    let title: String = cause_text.chars().take(100).collect();
    let request = StewardRequest::new(
        Some(priority),
        &format!("Fix the root cause of a repeated failure: {title}"),
        &body,
        &acceptance,
        &["crates/".to_string()],
    )
    .map(|r| r.render())?;
    // A last check of the finished text: what is returned is what admission scans.
    plain_boundary_text(&request)
}

// ---------------------------------------------------------------------------------------------------------
// Gate review

/// The start of the line that ties a request to the gate it reviews. A request job that is not closed and
/// holds a line equal to `Gate under review: NAME.` is the open review of that gate.
const GATE_LINE: &str = "Gate under review: ";

/// The line of a gate request that names its gate.
fn gate_line(gate: &str) -> String {
    format!("{GATE_LINE}{gate}.")
}

/// The text of the normal-priority request that reviews one gate, built with [`StewardRequest`]. It cites
/// the gate's counts (the jobs it ended and the times a later try landed the same change) and the
/// introspection that proposed `kind`, and asks for a reviewed change; it does not say the gate is wrong. No
/// body line starts like a bullet.
pub fn gate_request(
    count: &GateCount,
    kind: PreventionKind,
    introspection: &IntrospectionRecord,
) -> Result<String, String> {
    let proposed = introspection
        .prevention
        .as_deref()
        .map(|p| one_line(prevention_without_gate(p)))
        .unwrap_or_default();
    let body = format!(
        "Athena counts, for each gate (a check or rule that ends a job), the jobs it ended and the times a later \
         try at the same request landed the same change the gate had ended. For the {gate} gate, an \
         introspection proposed to {kind} it. Decide from these counts, and from what the gate checks, whether \
         it is wrong; change it only if the evidence supports that, and keep it if it does not.\n\n\
         {line}\n\
         Counts: {counts}.\n\
         Proposal: {kind}.\n\
         Introspection {id} proposed: {proposed}\n\n\
         A relaxed or removed gate must leave a test that shows what the loosened gate still refuses, or the \
         change must say why none is needed.",
        gate = count.gate,
        kind = kind.as_str(),
        line = gate_line(&count.gate),
        counts = count.describe(),
        id = introspection.id,
    );
    let acceptance = [
        "The change names the gate and the counts it rests on, and says whether the gate was relaxed, removed or kept"
            .to_string(),
        "fmt, clippy and test pass".to_string(),
    ];
    let title = format!(
        "Review the {} gate: a later try landed the same change after it ended a job",
        count.gate
    );
    let request = StewardRequest::new(
        Some(Priority::Normal),
        &title,
        &body,
        &acceptance,
        &["crates/".to_string(), "docs/".to_string()],
    )
    .map(|r| r.render())?;
    plain_boundary_text(&request)
}

/// The open request job, if any, that reviews `gate`: a request that is not closed and has a line equal to
/// `Gate under review: <gate>.` The match is the whole line, so the review of `test` is not the review of
/// `test-all`. A closed one (landed, rejected, failed or dropped) does not count.
fn open_gate_request<'a>(records: &'a Records, gate: &str) -> Option<&'a JobRecords> {
    let wanted = gate_line(gate);
    records.request_jobs().into_iter().find(|j| {
        !matches!(j.state(), JobState::Closed(_))
            && j.request
                .as_ref()
                .is_some_and(|r| r.text.lines().any(|l| l.trim_end() == wanted))
    })
}

// ---------------------------------------------------------------------------------------------------------
// Spec review

/// The spec Athena reads its own requirements from, at HEAD.
pub const SPEC_FILE: &str = "docs/spec.toml";

/// The most requests one spec review queues. The rest wait for a later pass and stay in `athena status`.
pub const SPEC_REQUESTS_PER_PASS: usize = 5;

/// The line that ties a request to its requirement. A request job that is not closed and holds this exact
/// line, followed by the requirement id, is the open request for that requirement.
const SPEC_LINE: &str = "Spec requirement: ";

/// The status labels whose measures are about time, so a requirement measured by them is about speed.
const SPEED_LABELS: &[&str] = &["throughput", "checks"];

/// One `[[requirement]]` of `docs/spec.toml`, with the fields the spec review reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecRequirement {
    pub id: String,
    pub statement: String,
    /// True for `status = "active"`; a superseded requirement is exempt from the review.
    pub active: bool,
    /// The code path, role or check that meets the requirement, or `none`.
    pub mechanism: String,
    /// The role responsible for the requirement.
    pub owner: String,
    /// How the requirement is shown, ending with `status: <label>` (see [`crate::status::STATUS_MEASURE_LABELS`])
    /// or `status: not shown` and the test or check that shows it.
    pub measure: String,
}

/// Parses the requirements of `docs/spec.toml`. A file with none gives an empty list.
pub fn parse_spec(text: &str) -> Result<Vec<SpecRequirement>> {
    let doc: toml_edit::DocumentMut = text.parse().context("docs/spec.toml is not valid TOML")?;
    let Some(tables) = doc
        .get("requirement")
        .and_then(toml_edit::Item::as_array_of_tables)
    else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for table in tables {
        let field = |key: &str| {
            table
                .get(key)
                .and_then(toml_edit::Item::as_str)
                .unwrap_or("")
                .trim()
                .to_string()
        };
        let id = field("id");
        if id.is_empty() {
            bail!("a requirement in docs/spec.toml has no id");
        }
        out.push(SpecRequirement {
            statement: field("statement"),
            active: field("status") == "active",
            mechanism: field("mechanism"),
            owner: field("owner"),
            measure: field("measure"),
            id,
        });
    }
    Ok(out)
}

fn blank_or_none(text: &str) -> bool {
    let t = text.trim();
    t.is_empty() || t.eq_ignore_ascii_case("none")
}

/// The label a measure ends with: the text after its last `status:`.
fn measure_label(measure: &str) -> Option<&str> {
    let (_, label) = measure.rsplit_once("status:")?;
    Some(label.trim().trim_end_matches('.').trim())
}

/// What is wrong with a measure, or `None` when it is valid: it is not blank or `none`, and it ends with
/// `status: not shown` or `status: <label>` with a label `athena status` prints.
pub fn measure_problem(measure: &str) -> Option<String> {
    if blank_or_none(measure) {
        return Some("the measure is blank or none".into());
    }
    let Some(label) = measure_label(measure) else {
        return Some("the measure does not end with `status: <label>`".into());
    };
    if label.is_empty() {
        return Some("the measure has `status:` with no label".into());
    }
    if label.starts_with("not shown")
        || crate::status::STATUS_MEASURE_LABELS
            .iter()
            .any(|(known, _)| *known == label)
    {
        return None;
    }
    Some(format!(
        "the measure names `{label}`, which `athena status` does not print"
    ))
}

/// Whether a requirement is about speed: its measure is shown in a status section about time. Such a
/// requirement's request is high.
pub fn is_speed(requirement: &SpecRequirement) -> bool {
    measure_label(&requirement.measure).is_some_and(|label| SPEED_LABELS.contains(&label))
}

/// An active requirement that has no mechanism or a failing measure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpecGap {
    pub id: String,
    pub statement: String,
    /// The role named as owner; empty when the spec names none.
    pub owner: String,
    /// The mechanism is blank or `none`.
    pub no_mechanism: bool,
    /// Why the measure fails, when it does.
    pub measure_problem: Option<String>,
    /// The requirement is about speed ([`is_speed`]).
    pub speed: bool,
}

impl SpecGap {
    /// What is missing, in words.
    pub fn what(&self) -> String {
        let mut parts = Vec::new();
        if self.no_mechanism {
            parts.push("has no mechanism".to_string());
        }
        if let Some(problem) = &self.measure_problem {
            parts.push(format!("has a failing measure: {problem}"));
        }
        parts.join(" and ")
    }

    /// The `next` line of `athena status` for this gap.
    pub fn next_line(&self) -> String {
        let owner = if self.owner.is_empty() {
            "no owner named"
        } else {
            &self.owner
        };
        format!(
            "requirement {} {}   # owner: {owner} · Athena queues a request at the next drain or sweep",
            self.id,
            self.what()
        )
    }
}

/// The active requirements with no mechanism or a failing measure, in spec order.
pub fn spec_gaps(requirements: &[SpecRequirement]) -> Vec<SpecGap> {
    requirements
        .iter()
        .filter(|r| r.active)
        .filter_map(|r| {
            let no_mechanism = blank_or_none(&r.mechanism);
            let measure_problem = measure_problem(&r.measure);
            (no_mechanism || measure_problem.is_some()).then(|| SpecGap {
                id: r.id.clone(),
                statement: one_line(&r.statement),
                owner: r.owner.clone(),
                no_mechanism,
                measure_problem,
                speed: is_speed(r),
            })
        })
        .collect()
}

/// The gaps of the spec committed at `head`. A repository with no spec has none.
pub fn spec_gaps_at(git: &athena_git::Git, head: &str) -> Result<Vec<SpecGap>> {
    let Some(bytes) = git.show_file(head, SPEC_FILE)? else {
        return Ok(Vec::new());
    };
    let text = String::from_utf8(bytes).context("docs/spec.toml is not UTF-8")?;
    Ok(spec_gaps(&parse_spec(&text)?))
}

/// The request that asks for a requirement's mechanism or measure, built with [`StewardRequest`]. It is
/// high for a requirement about speed and normal otherwise.
pub fn spec_request(gap: &SpecGap) -> Result<String, String> {
    let title = if gap.no_mechanism {
        format!("Give requirement {} a mechanism", gap.id)
    } else {
        format!("Give requirement {} a valid measure", gap.id)
    };
    let owner = if gap.owner.is_empty() {
        "none named"
    } else {
        &gap.owner
    };
    let body = format!(
        "Requirement {id} in docs/spec.toml is active and {what}.\n\
         Requirement: {statement}\n\
         Owner: {owner}\n\n\
         Build the code path, role or check that meets the requirement, and record it as the mechanism in \
         docs/spec.toml. Write a measure that says how the requirement is shown and ends with \
         `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the \
         test that shows it.\n\n\
         {SPEC_LINE}{id}\n\n\
         This request is queued again at each drain while the requirement is active and its mechanism or \
         measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the \
         requirement superseded.",
        id = gap.id,
        what = gap.what(),
        statement = gap.statement,
    );
    let acceptance = [
        format!(
            "Requirement {} names a mechanism that exists in the code, an owner and a measure",
            gap.id
        ),
        "A test fails without the mechanism and passes with it".to_string(),
        "fmt, clippy and test pass".to_string(),
    ];
    let priority = if gap.speed {
        Priority::High
    } else {
        Priority::Normal
    };
    StewardRequest::new(
        Some(priority),
        &title,
        &body,
        &acceptance,
        &["docs/spec.toml".to_string(), "crates/".to_string()],
    )
    .map(|r| r.render())
}

/// Whether a request job that is not closed already asks for this requirement.
fn spec_request_is_open(records: &Records, id: &str) -> bool {
    !open_spec_requests(records, Some(id)).is_empty()
}

/// The request jobs that are not closed and hold a `Spec requirement:` line: the one for requirement `id`, or
/// with `None` the line of any requirement. The line test is exact: a whole line, trimmed.
fn open_spec_requests<'a>(records: &'a Records, id: Option<&str>) -> Vec<&'a JobRecords> {
    let line = id.map(|id| format!("{SPEC_LINE}{id}"));
    let holds = |l: &str| {
        let l = l.trim();
        match &line {
            Some(line) => l == line,
            None => l
                .strip_prefix(SPEC_LINE)
                .is_some_and(|rest| !rest.trim().is_empty()),
        }
    };
    records
        .request_jobs()
        .into_iter()
        .filter(|job| {
            !matches!(job.state(), JobState::Closed(_))
                && job
                    .request
                    .as_ref()
                    .is_some_and(|r| r.text.lines().any(holds))
        })
        .collect()
}

/// What a spec review would do over the records.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct SpecPlan {
    /// The gaps to queue now and the request text for each, speed first, then in spec order.
    pub queue: Vec<(String, Result<String, String>)>,
    /// Gaps an open request already covers.
    pub already_open: usize,
    /// Gaps past the per-pass cap, left for a later pass.
    pub deferred: usize,
}

/// Decides which gaps get a request: none that an open request covers, at most
/// [`SPEC_REQUESTS_PER_PASS`], speed first.
pub fn plan_spec_requests(gaps: &[SpecGap], records: &Records) -> SpecPlan {
    let mut plan = SpecPlan::default();
    let mut ordered: Vec<&SpecGap> = gaps.iter().collect();
    ordered.sort_by_key(|g| !g.speed);
    for gap in ordered {
        if spec_request_is_open(records, &gap.id) {
            plan.already_open += 1;
        } else if plan.queue.len() >= SPEC_REQUESTS_PER_PASS {
            plan.deferred += 1;
        } else {
            plan.queue.push((gap.id.clone(), spec_request(gap)));
        }
    }
    plan
}

/// What one spec review did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct SpecReport {
    /// How many active requirements lack a mechanism or a valid measure.
    pub gaps: usize,
    /// The requests queued: requirement id and job.
    pub queued: Vec<(String, JobId)>,
    pub already_open: usize,
    pub deferred: usize,
    /// Requests that could not be built or queued, with the reason.
    pub failed: Vec<String>,
}

impl Athena {
    /// The spec review: for each active requirement of `docs/spec.toml` at HEAD that has no mechanism or a
    /// failing measure, queues one request through the steward request builder, high when the requirement is
    /// about speed. A requirement that an open request already covers gets none, so a second run changes
    /// nothing. Athena runs it in [`Athena::review_passes`], at the start of every drain and in each sweep.
    pub fn spec_review(&self, ui: &dyn Ui) -> Result<SpecReport> {
        let head = self.head()?;
        let gaps = spec_gaps_at(&self.git, &head)?;
        let records = Records::load(&self.git, &head)?;
        let plan = plan_spec_requests(&gaps, &records);
        let mut report = SpecReport {
            gaps: gaps.len(),
            already_open: plan.already_open,
            deferred: plan.deferred,
            ..SpecReport::default()
        };
        for (id, text) in plan.queue {
            let queued = text
                .map_err(|e| anyhow::anyhow!("the request for {id}: {e}"))
                .and_then(|text| self.queue(&text));
            match queued {
                Ok((job, _)) => {
                    ui.line(&format!(
                        "hestia     requirement {id} has no mechanism or a failing measure; request {job} queued"
                    ));
                    report.queued.push((id, job));
                }
                Err(e) => report.failed.push(format!("{id}: {e:#}")),
            }
        }
        Ok(report)
    }
}

// ---------------------------------------------------------------------------------------------------------
// Backfill

/// How many of the highest-ranked causes a backfill makes high. The rest are normal. Hestia never queues
/// urgent: that level is for a person.
pub const BACKFILL_HIGH: usize = 5;

/// A stage of the job pipeline that `athena.toml` can switch off.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    Planner,
    Critic,
}

impl Stage {
    pub fn name(self) -> &'static str {
        match self {
            Stage::Planner => "planner",
            Stage::Critic => "critic",
        }
    }

    /// The stages the configuration switches off. The planner and the critic run only inside job planning, so
    /// they are off when `[job_plan]` is absent or has `enabled = false`.
    pub fn disabled(config: &athena_data::Config) -> Vec<Stage> {
        if config.job_plan.as_ref().is_some_and(|p| p.enabled) {
            Vec::new()
        } else {
            vec![Stage::Planner, Stage::Critic]
        }
    }

    /// The stage a cause is about, read from the words of its class name and its cause text, because an
    /// introspection records no stage. The first word that names a stage decides; none means a stage that
    /// cannot be switched off.
    pub fn of(cause: &RepeatedCause) -> Option<Stage> {
        let words = format!("{} {}", cause.class.as_deref().unwrap_or(""), cause.cause);
        words
            .split(|c: char| !c.is_alphanumeric())
            .find_map(|w| match w.to_lowercase().as_str() {
                "planner" | "planners" => Some(Stage::Planner),
                "critic" | "critics" => Some(Stage::Critic),
                _ => None,
            })
    }
}

/// A repeated cause in a stage that `athena.toml` has switched off: listed, never queued.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DormantCause {
    pub stage: Stage,
    pub cause: RepeatedCause,
}

/// Splits causes into those that can be worked and those whose stage is in `disabled`.
pub fn split_dormant(
    causes: Vec<RepeatedCause>,
    disabled: &[Stage],
) -> (Vec<RepeatedCause>, Vec<DormantCause>) {
    let mut live = Vec::new();
    let mut dormant = Vec::new();
    for cause in causes {
        match Stage::of(&cause).filter(|s| disabled.contains(s)) {
            Some(stage) => dormant.push(DormantCause { stage, cause }),
            None => live.push(cause),
        }
    }
    (live, dormant)
}

/// What a backfill did about one cause.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackfillAction {
    /// A new root-cause request was queued.
    Queued,
    /// An existing open request was queued again through the builder, and the old one closed.
    Requeued,
    /// An existing queued request already covered the cause; only its priority was set.
    Reprioritised,
    /// Nothing was changed, for the reason given.
    Skipped,
}

impl BackfillAction {
    fn label(self) -> &'static str {
        match self {
            BackfillAction::Queued => "queued",
            BackfillAction::Requeued => "re-queued",
            BackfillAction::Reprioritised => "priority set",
            BackfillAction::Skipped => "skipped",
        }
    }
}

/// One cause's line in the report of a backfill.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BackfillLine {
    pub action: BackfillAction,
    pub cause: String,
    pub introspections: Vec<String>,
    /// The priority the ranking gave it; none for a cause that is not ranked (dormant or landed).
    pub priority: Option<String>,
    /// The request queued, or the existing request the line is about.
    pub job: Option<String>,
    /// The old request a re-queue closed.
    pub replaced: Option<String>,
    pub reason: String,
}

/// What a backfill queued, re-queued and skipped, in rank order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct BackfillReport {
    pub lines: Vec<BackfillLine>,
}

impl BackfillReport {
    pub fn count(&self, action: BackfillAction) -> usize {
        self.lines.iter().filter(|l| l.action == action).count()
    }

    /// True when the backfill changed no record.
    pub fn changed_nothing(&self) -> bool {
        self.count(BackfillAction::Skipped) == self.lines.len()
    }

    /// The report as text: a count line, then one line per cause with the reason for what was done.
    pub fn render(&self) -> String {
        if self.lines.is_empty() {
            return "no repeated causes in the records; nothing to do\n".into();
        }
        let mut out = format!(
            "Hestia error review backfill: {} repeated cause(s) · queued {} · re-queued {} · priority set {} · skipped {}\n",
            self.lines.len(),
            self.count(BackfillAction::Queued),
            self.count(BackfillAction::Requeued),
            self.count(BackfillAction::Reprioritised),
            self.count(BackfillAction::Skipped),
        );
        for l in &self.lines {
            out.push_str(&format!(
                "{:<13} {:<7} {} · {}\n",
                l.action.label(),
                l.priority.as_deref().unwrap_or("-"),
                l.job.as_deref().unwrap_or("-"),
                l.cause
            ));
            out.push_str(&format!("{:<13}   why: {}\n", "", l.reason));
            if let Some(old) = &l.replaced {
                out.push_str(&format!("{:<13}   closed: {old}\n", ""));
            }
        }
        out
    }
}

/// What a backfill will do about one cause.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Step {
    Skip { job: Option<String>, reason: String },
    Queue { text: String },
    Requeue { old: JobId, text: String },
    Reprioritise { job: JobId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Planned {
    cause: RepeatedCause,
    /// The priority the ranking gave the cause; none for a cause that is not ranked.
    priority: Option<Priority>,
    step: Step,
}

/// The newest introspection time of a cause.
fn latest_at(records: &Records, cause: &RepeatedCause) -> Option<Timestamp> {
    records
        .introspections
        .iter()
        .filter(|i| cause.introspections.contains(&i.id.to_string()))
        .map(|i| i.at)
        .max()
}

/// Whether a line is a `Priority:` line with a valid level, as `athena plan` reads one.
fn is_priority_line(line: &str) -> bool {
    let line = line.trim();
    line.get(..9)
        .is_some_and(|head| head.eq_ignore_ascii_case("priority:"))
        && Priority::parse(&line[9..]).is_some()
}

/// A request's text without its priority line, for telling whether two requests differ in more than priority.
fn without_priority(text: &str) -> String {
    text.lines()
        .filter(|l| !is_priority_line(l))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Whether a request's words are about a cause: they name its failure class (id or name), or repeat its
/// cause text with case and punctuation ignored.
fn mentions(text: &str, cause: &RepeatedCause, records: &Records) -> bool {
    if let Some(id) = cause.key.strip_prefix("class:") {
        if text.contains(id) {
            return true;
        }
        let lowered = text.to_lowercase();
        if records
            .classes
            .iter()
            .any(|c| c.id.as_str() == id && lowered.contains(&c.name.to_lowercase()))
        {
            return true;
        }
    }
    let first = cause.introspections.first();
    let wanted = records
        .introspections
        .iter()
        .find(|i| Some(&i.id.to_string()) == first)
        .and_then(|i| i.cause.as_deref())
        // A root-cause request holds the cause as reworded by `root_cause_request`.
        .map(|cause| plain_boundary_text(cause).unwrap_or_else(|_| cause.to_string()))
        .map(|cause| crate::introspect::normalise(&cause))
        .unwrap_or_default();
    !wanted.is_empty() && crate::introspect::normalise(text).contains(&wanted)
}

/// The open request that addresses a cause: the one that cites its introspections, else one whose words are
/// about it ([`mentions`]), such as a steward request the planner refuses for missing acceptance. A request
/// that can be queued again is preferred. A request in `used` already belongs to another cause.
fn existing_request<'a>(
    records: &'a Records,
    cause: &RepeatedCause,
    used: &BTreeSet<String>,
) -> Option<&'a JobRecords> {
    if let Some(job) = cause
        .request
        .as_ref()
        .and_then(|r| records.jobs.get(&r.job))
        .filter(|j| !used.contains(j.id().as_str()))
    {
        return Some(job);
    }
    let mut found: Vec<&JobRecords> = records
        .request_jobs()
        .into_iter()
        .filter(|j| !used.contains(j.id().as_str()) && !matches!(j.state(), JobState::Closed(_)))
        .filter(|j| {
            j.request
                .as_ref()
                .is_some_and(|r| mentions(&r.text, cause, records))
        })
        .collect();
    found.sort_by_key(|j| !can_requeue(j.state()));
    found.into_iter().next()
}

/// The states in which a request is only waiting, so it can be closed and queued again.
fn can_requeue(state: JobState) -> bool {
    matches!(
        state,
        JobState::Queued | JobState::Proposed | JobState::Declined
    )
}

/// An existing request's text queued again at `priority`, through [`StewardRequest`]: its title, its body, its
/// acceptance (a default when it has none, which is why the planner refused it), and the files it names (the
/// `Files:` line, else any path in it, else `crates/`). The introspection ids are added when it does not cite
/// them, so `athena status` finds the request.
fn rebuild_request(
    text: &str,
    priority: Priority,
    cause: &RepeatedCause,
) -> Result<String, String> {
    let mut title: Option<&str> = None;
    let mut body: Vec<&str> = Vec::new();
    let mut acceptance: Vec<String> = Vec::new();
    let mut files_line: Option<&str> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if is_priority_line(trimmed) {
            continue;
        }
        if title.is_none() {
            if !trimmed.is_empty() {
                title = Some(trimmed);
            }
            continue;
        }
        if trimmed.eq_ignore_ascii_case("acceptance:") {
            continue;
        }
        if trimmed
            .get(..6)
            .is_some_and(|head| head.eq_ignore_ascii_case("files:"))
        {
            files_line = Some(&trimmed[6..]);
            continue;
        }
        if crate::reading::request_text_acceptance(trimmed).is_empty() {
            body.push(line);
        } else {
            let words = trimmed.split_once(' ').map_or(trimmed, |(_, rest)| rest);
            acceptance.push(words.trim().to_string());
        }
    }
    let title = title.ok_or("the request has no title")?;
    if acceptance.is_empty() {
        acceptance = vec![
            "The change the request describes is made, with the root cause named".to_string(),
            "A test fails without the change and passes with it".to_string(),
            "fmt, clippy and test pass".to_string(),
        ];
    }
    let mut files = crate::reading::request_text_touches(files_line.unwrap_or(text));
    if files.is_empty() {
        files.push("crates/".to_string());
    }
    let mut body = body.join("\n").trim().to_string();
    if !cause
        .introspections
        .iter()
        .any(|id| text.contains(id.as_str()))
    {
        body.push_str(&format!(
            "\n\nIntrospections that record the repeated cause: {}",
            cause.introspections.join(", ")
        ));
    }
    StewardRequest::new(Some(priority), title, body.trim(), &acceptance, &files).map(|r| r.render())
}

/// What to do about one cause that can be worked, given the priority its rank earned.
fn backfill_step(
    records: &Records,
    cause: &RepeatedCause,
    priority: Priority,
    used: &mut BTreeSet<String>,
) -> Step {
    let Some(old) = existing_request(records, cause, used) else {
        return match root_cause_request_at(cause, records, priority) {
            Ok(text) => Step::Queue { text },
            Err(e) => Step::Skip {
                job: None,
                reason: format!("the root-cause request could not be built: {e}"),
            },
        };
    };
    let id = old.id().clone();
    used.insert(id.as_str().to_string());
    // A request a person already made urgent keeps that level: the backfill never lowers it.
    let priority = priority.min(old.priority());
    let state = old.state();
    let skip = |reason: String| Step::Skip {
        job: Some(id.as_str().to_string()),
        reason,
    };
    if !can_requeue(state) {
        return skip(format!(
            "request {id} is {}; it is left alone",
            state.name()
        ));
    }
    let text = old.request.as_ref().map_or("", |r| r.text.as_str());
    let rebuilt = match rebuild_request(text, priority, cause) {
        Ok(rebuilt) => rebuilt,
        Err(e) => return skip(format!("request {id} cannot be queued again: {e}")),
    };
    if without_priority(&rebuilt) != without_priority(text) {
        return Step::Requeue {
            old: id,
            text: rebuilt,
        };
    }
    if old.priority() == priority {
        return skip(format!(
            "request {id} ({}) already covers the cause at {}",
            state.name(),
            priority.as_str()
        ));
    }
    if state == JobState::Queued {
        Step::Reprioritise { job: id }
    } else {
        Step::Requeue {
            old: id,
            text: rebuilt,
        }
    }
}

/// Plans a backfill over the records. Dormant causes (stage in `disabled`) and causes whose request has landed
/// are skipped. The rest are ranked by occurrences, then by the newest introspection, and the first
/// [`BACKFILL_HIGH`] are high (never urgent) and the rest normal. A cause whose request a person closed is
/// skipped until it recurs ([`addressing_request`]). The plan lists the ranked causes first, in rank order.
fn plan_backfill(records: &Records, disabled: &[Stage]) -> Vec<Planned> {
    let mut skipped: Vec<Planned> = Vec::new();
    let mut ranked: Vec<(Option<Timestamp>, RepeatedCause)> = Vec::new();
    for cause in repeated_causes(records) {
        let reason = if let Some(stage) = Stage::of(&cause).filter(|s| disabled.contains(s)) {
            format!(
                "dormant: the {} stage is disabled in athena.toml ([job_plan] enabled is false)",
                stage.name()
            )
        } else if let Some(r) = cause.request.as_ref().filter(|r| r.landed) {
            format!("request {} has landed", r.job)
        } else {
            ranked.push((latest_at(records, &cause), cause));
            continue;
        };
        skipped.push(Planned {
            step: Step::Skip {
                job: cause.request.as_ref().map(|r| r.job.clone()),
                reason,
            },
            priority: None,
            cause,
        });
    }
    ranked
        .sort_by(|(at_a, a), (at_b, b)| (b.count(), at_b, &a.key).cmp(&(a.count(), at_a, &b.key)));
    let mut used = BTreeSet::new();
    let mut plan: Vec<Planned> = ranked
        .into_iter()
        .enumerate()
        .map(|(rank, (_, cause))| {
            let priority = if rank < BACKFILL_HIGH {
                Priority::High
            } else {
                Priority::Normal
            };
            Planned {
                step: backfill_step(records, &cause, priority, &mut used),
                priority: Some(priority),
                cause,
            }
        })
        .collect();
    plan.extend(skipped);
    plan
}

// ---------------------------------------------------------------------------------------------------------
// Throughput review

/// The median minutes per landed job must be more than this times the previous review's to count as worse:
/// a rise of more than 20%.
pub const WORSEN_RATIO: f64 = 1.2;

/// The title of a throughput design request starts with this. An open request with a line that starts so
/// means the last proposal is still being worked; a review still runs and names it, and queues another only
/// when the gate ([`request_gate`]) finds a change beyond the noise.
pub const THROUGHPUT_MARKER: &str = "Throughput design: ";

/// The median minutes per landed job must be more than this times the previous review's for the review to
/// queue a slowdown request: a rise of more than 25%. A rise between [`WORSEN_RATIO`] and this still runs a
/// review and names the stages that grew, but queues no slowdown request.
pub const SLOWDOWN_RATIO: f64 = 1.25;

/// The title of a slowdown request starts with this. An open request with a line that starts so means the
/// last slowdown is still being worked, and no second slowdown request is queued. It shares no prefix with
/// [`THROUGHPUT_MARKER`], so an open design request never holds back a slowdown request, nor the reverse.
pub const SLOWDOWN_MARKER: &str = "Throughput slowdown: ";

/// A stage whose median grew by less than this many minutes is not reported as grown.
const GROWTH_EPSILON: f64 = 1e-6;

/// Expected minutes saved per landed job at or above which a design request is high, the most Hestia queues.
pub const HIGH_MINUTES: f64 = 5.0;
/// ... normal. Below this a request is low.
pub const NORMAL_MINUTES: f64 = 2.0;

/// The most landed jobs after a cited change that its measured median covers (the newest of them).
pub const FOLLOWUP_WINDOW: usize = 10;
/// The fewest landed jobs after a cited change that a follow-up needs.
pub const FOLLOWUP_MIN_JOBS: usize = 1;

/// The file in the model's workspace that holds the throughput section.
pub const THROUGHPUT_FILE: &str = "THROUGHPUT.md";
/// The most characters of each design document copied into the model's workspace.
const MAX_DOC_CHARS: usize = 60_000;

/// The role prompt of the throughput review.
const THROUGHPUT_PROMPT: &str = "You review how fast Athena's pipeline turns requests into landed changes. \
You are given measurements of the pipeline (THROUGHPUT.md) and its design (design/). Find the one biggest \
bottleneck: the stage or wait that costs the most minutes per landed job. Then name the architectural change \
to the shape of the pipeline that removes it, not a tweak inside one stage, and estimate honestly how many \
minutes per landed job that change would save. Speed comes first. Base every claim on THROUGHPUT.md or on \
the design documents. Do not propose changes to the error review or the code-health review.";

/// Why a throughput review is due, or None. A review is due when [`crate::code_health::REVIEW_EVERY`]
/// landings stand beyond the previous review (the first ten landings when there is none), or when at least
/// one job landed since the previous review and either the median minutes per landed job is more than
/// [`WORSEN_RATIO`] times the one that review recorded ([`ThroughputTrigger::Worsened`]) or wasted work
/// rose by more than that ratio against the figures that review stored ([`waste_rises`],
/// [`ThroughputTrigger::Wasted`]). `landed` and `median` are the status throughput section's
/// `landed_total` and `median_minutes`; `waste` is the figures over its newest window.
pub fn throughput_due(
    landed: usize,
    median: f64,
    waste: Option<WasteFigures>,
    latest: Option<&ThroughputReview>,
) -> Option<ThroughputTrigger> {
    let every = crate::code_health::REVIEW_EVERY as usize;
    let Some(previous) = latest else {
        return (landed >= every).then_some(ThroughputTrigger::Cadence);
    };
    let before = previous.landings as usize;
    if landed >= before + every {
        return Some(ThroughputTrigger::Cadence);
    }
    if landed <= before {
        return None;
    }
    let worse = previous.window_median_minutes > 0.0
        && median / previous.window_median_minutes > WORSEN_RATIO;
    if worse {
        return Some(ThroughputTrigger::Worsened);
    }
    let rises = previous
        .waste
        .zip(waste)
        .is_some_and(|(before, now)| !waste_rises(&before, &now).is_empty());
    rises.then_some(ThroughputTrigger::Wasted)
}

/// The measures of wasted work that rose by more than [`WORSEN_RATIO`] from `previous` to `now`, by name:
/// attempts per landed change and money spent on attempts that did not land rising by more than 20%, and the
/// first-attempt share falling by more than 20% (below `2 - WORSEN_RATIO` times the previous share). When
/// nothing was spent on attempts that did not land before, a spend above zero counts only if attempts per
/// landed change also rose by more than 20%, so that one failed run after a clean window starts no review.
pub fn waste_rises(previous: &WasteFigures, now: &WasteFigures) -> Vec<&'static str> {
    let mut rises = Vec::new();
    let attempts = now.attempts_per_landing > previous.attempts_per_landing * WORSEN_RATIO;
    if attempts {
        rises.push("attempts per landed change");
    }
    let spend = if previous.wasted_usd > 0.0 {
        now.wasted_usd > previous.wasted_usd * WORSEN_RATIO
    } else {
        now.wasted_usd > 0.0 && attempts
    };
    if spend {
        rises.push("spent on attempts that did not land");
    }
    if now.first_attempt_share < previous.first_attempt_share * (2.0 - WORSEN_RATIO) {
        rises.push("share landed on the first attempt (fell)");
    }
    rises
}

/// The landings the design-request gate needs to estimate noise: the newest three windows.
const GATE_LANDINGS: usize = 3 * THROUGHPUT_WINDOW;

/// A change must exceed its bar by more than this to count, so that a change of exactly one job (such as
/// 21/20 - 20/20) is not made larger than the one-job floor by rounding.
const NOISE_EPSILON: f64 = 1e-9;

/// Whether a design request is queued, and the sentence for the review's note. See [`request_gate`].
#[derive(Debug, Clone, PartialEq)]
pub struct Gate {
    pub queue: bool,
    /// Starts with `design request:`.
    pub note: String,
}

/// Decides whether the throughput review queues a design request. It never decides whether the model is
/// asked: a due review always asks, and records the proposal.
///
/// With the newest three windows full (60 landings) a measure is judged: its change is the newest window
/// against the one before it, in the bad direction, and it counts only if the change is more than the larger
/// of the noise between the older windows and the smallest change one job can make ([`Bar`]). A request is
/// queued when at least one measure is beyond that bar. The noise is taken from adjacent differences, so a
/// steady worsening of equal steps (say spend 2, 4, 6, then 8) is within it; the 20% trigger still runs the
/// review. With fewer landings the noise cannot be estimated: nothing is queued, except that the very first
/// review (`first_review`) queues one when no throughput design request is open (`request_open`).
pub fn request_gate(
    measures: &[Measure],
    landed: usize,
    first_review: bool,
    request_open: bool,
) -> Gate {
    let judged: Vec<&Measure> = measures
        .iter()
        .filter(|m| m.bar.bar().is_some() && m.before.is_some())
        .collect();
    if judged.is_empty() {
        if first_review && !request_open {
            return Gate {
                queue: true,
                note: format!(
                    "design request: the first review has no earlier window to judge a change against ({landed} of {GATE_LANDINGS} landings)"
                ),
            };
        }
        return Gate {
            queue: false,
            note: format!(
                "design request: not enough history: {landed} of {GATE_LANDINGS} landings, none queued"
            ),
        };
    }
    let describe = |m: &Measure, change: f64, bar: &Bar| {
        format!(
            "{} {} (noise {} from {} difference(s), one-job floor {})",
            m.name,
            fmt_change(m, change),
            fmt_amount(m, bar.noise.unwrap_or(0.0)),
            bar.differences,
            fmt_amount(m, bar.floor),
        )
    };
    let mut beyond = Vec::new();
    let mut within = Vec::new();
    for m in &judged {
        let (Some(change), Some(bar)) = (m.worsening(), m.bar.bar()) else {
            continue;
        };
        if change > bar + NOISE_EPSILON {
            beyond.push(describe(m, change, &m.bar));
        } else if change > 0.0 {
            within.push(describe(m, change, &m.bar));
        }
    }
    let left_out: Vec<&str> = measures
        .iter()
        .filter(|m| m.bar.bar().is_none() || m.before.is_none())
        .map(|m| m.name)
        .collect();
    let left_out = if left_out.is_empty() {
        String::new()
    } else {
        format!(
            "; not judged, too few full windows: {}",
            left_out.join(", ")
        )
    };
    if !beyond.is_empty() {
        return Gate {
            queue: true,
            note: format!(
                "design request: worse by more than the noise: {}{left_out}",
                beyond.join("; ")
            ),
        };
    }
    let note = if within.is_empty() {
        "design request: no measure worsened, none queued".to_string()
    } else {
        format!(
            "design request: within noise, none queued: worsened by {}",
            within.join("; ")
        )
    };
    Gate {
        queue: false,
        note: format!("{note}{left_out}"),
    }
}

fn fmt_amount(m: &Measure, value: f64) -> String {
    match m.unit {
        "USD" => format!("{value:.2} USD"),
        "min" => format!("{value:.1} min"),
        _ if m.name.starts_with("share") => format!("{:.0} points", value * 100.0),
        _ => format!("{value:.2}"),
    }
}

fn fmt_change(m: &Measure, change: f64) -> String {
    let word = if m.higher_is_worse { "up" } else { "down" };
    format!("{word} {}", fmt_amount(m, change))
}

/// Applies the noise rule to a slowdown request. `rise` is the median minutes per landed job now minus the
/// previous review's. `Err` holds the remark when the rise is no more than the larger of the median's noise
/// and its one-minute floor, so no request is queued. `Ok(None)` means the rise is beyond that bar.
/// `Ok(Some(remark))` means the noise cannot be estimated (under 60 landings), so the 25% rule alone decides
/// and the remark says so.
pub fn slowdown_beyond_noise(
    rise: f64,
    bar: &Bar,
    landed: usize,
) -> Result<Option<String>, String> {
    let Some(limit) = bar.bar() else {
        return Ok(Some(format!(
            "noise could not be estimated ({landed} of {GATE_LANDINGS} landings); the 25% rule applied"
        )));
    };
    if rise > limit + NOISE_EPSILON {
        return Ok(None);
    }
    Err(format!(
        "slowdown request: median rose {rise:.1} min over the previous review, within noise (noise {:.1} min from {} difference(s), floor {:.1} min); none queued",
        bar.noise.unwrap_or(0.0),
        bar.differences,
        bar.floor
    ))
}

/// The priority a design request gets from the minutes per landed job it is expected to save: more minutes,
/// higher priority (speed first), and never above high: urgent is for a person.
pub fn priority_for_minutes(expected_minutes_saved: f64) -> Priority {
    if expected_minutes_saved >= HIGH_MINUTES {
        Priority::High
    } else if expected_minutes_saved >= NORMAL_MINUTES {
        Priority::Normal
    } else {
        Priority::Low
    }
}

/// What the model said about the bottleneck.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThroughputOutput {
    /// The biggest bottleneck, in one or two sentences.
    pub bottleneck: String,
    /// The stage it sits in.
    pub stage: String,
    /// The architectural change that removes it.
    pub change: String,
    /// Minutes saved per landed job once the change has landed.
    pub expected_minutes_saved: f64,
    /// Why the change removes the bottleneck, citing THROUGHPUT.md or a design document.
    pub rationale: String,
}

impl ThroughputOutput {
    fn validate(&self) -> Result<()> {
        if self.bottleneck.trim().is_empty() || self.change.trim().is_empty() {
            bail!("the throughput proposal names no bottleneck or no change");
        }
        if !self.expected_minutes_saved.is_finite() || self.expected_minutes_saved < 0.0 {
            bail!("the throughput proposal's expected minutes saved is not a number of minutes");
        }
        Ok(())
    }
}

/// The schema of the model's answer: every field required, none extra.
fn throughput_schema() -> serde_json::Value {
    serde_json::json!({
        "type": "object",
        "properties": {
            "bottleneck": { "type": "string", "description": "The biggest bottleneck, in one or two sentences." },
            "stage": { "type": "string", "description": "The stage it sits in: reading, planning, producer, checks, verifier, landing and rebuild, or idle." },
            "change": { "type": "string", "description": "The architectural change that removes it." },
            "expected_minutes_saved": { "type": "number", "description": "Minutes saved per landed job once the change has landed." },
            "rationale": { "type": "string", "description": "Why the change removes the bottleneck, citing THROUGHPUT.md or a design document." }
        },
        "required": ["bottleneck", "stage", "change", "expected_minutes_saved", "rationale"],
        "additionalProperties": false
    })
}

/// The text of the design request for a proposal, built with [`StewardRequest`] at the priority its expected
/// minutes set. The title starts with [`THROUGHPUT_MARKER`]. The body cites the review's id and asks that a
/// change which implements the design cites it too, so that the next review can measure the change.
pub fn throughput_request(
    review: &RecordId,
    answer: &ThroughputOutput,
    median_minutes: f64,
) -> Result<String, String> {
    let priority = priority_for_minutes(answer.expected_minutes_saved);
    let body = format!(
        "Athena's throughput review {review} named the biggest wait in the pipeline and an architectural change \
         that would remove it. Design that change.\n\n\
         Bottleneck: {}\n\
         Stage: {}\n\
         Proposed change: {}\n\
         Predicted saving: {:.1} minutes per landed job; the median was {median_minutes:.1} minutes per landed \
         job when the review ran.\n\
         Why: {}\n\n\
         A change that implements this design must cite throughput review {review} in its request text. The next \
         throughput review then sets the minutes it measured after that change landed against this prediction.",
        one_line(&answer.bottleneck),
        one_line(&answer.stage),
        one_line(&answer.change),
        answer.expected_minutes_saved,
        one_line(&answer.rationale),
    );
    let title: String = one_line(&answer.bottleneck).chars().take(100).collect();
    let acceptance = [
        "A design note under docs/design/ describes the change and how it removes the bottleneck".to_string(),
        "The note states the minutes per landed job the change should save and how the next throughput review measures them".to_string(),
    ];
    StewardRequest::new(
        Some(priority),
        &format!("{THROUGHPUT_MARKER}{title}"),
        &body,
        &acceptance,
        &["docs/design/".to_string()],
    )
    .map(|r| r.render())
}

/// The open request job, if any, whose text has a line that starts with `marker`. A closed one (landed,
/// rejected, failed or abandoned) does not count.
fn open_request_with_marker<'a>(records: &'a Records, marker: &str) -> Option<&'a JobRecords> {
    records.request_jobs().into_iter().find(|j| {
        !matches!(j.state(), JobState::Closed(_))
            && j.request
                .as_ref()
                .is_some_and(|r| r.text.lines().any(|l| l.starts_with(marker)))
    })
}

/// The open design request, if any: see [`THROUGHPUT_MARKER`].
fn open_throughput_request(records: &Records) -> Option<&JobRecords> {
    open_request_with_marker(records, THROUGHPUT_MARKER)
}

/// Whether the median minutes per landed job rose by more than 25% ([`SLOWDOWN_RATIO`]) from `previous` to
/// `current`. False when `previous` is not above zero, since a ratio to nothing says nothing. Exactly 25% is
/// not more than 25%.
pub fn is_slowdown(previous: f64, current: f64) -> bool {
    previous > 0.0 && current / previous > SLOWDOWN_RATIO
}

/// One stage whose median minutes per landed job grew from one throughput review to the next.
#[derive(Debug, Clone, PartialEq)]
pub struct StageGrowth {
    /// The stage's name, as [`StageMinutes::named`] gives it.
    pub stage: &'static str,
    /// Its median minutes in the window the previous review saw.
    pub previous_minutes: f64,
    /// Its median minutes in the latest window.
    pub minutes: f64,
}

impl StageGrowth {
    /// The minutes the stage's median gained.
    pub fn added(&self) -> f64 {
        self.minutes - self.previous_minutes
    }
}

/// The stages whose median minutes grew from `previous` to `current`, the stage that gained the most minutes
/// first. A stage that shrank, stayed the same or grew by under a millionth of a minute is left out. Stages
/// that gained the same minutes keep pipeline order. The stage medians are medians of each stage alone, so
/// the list can be empty although the median of the totals rose.
pub fn stage_growth(previous: &StageMinutes, current: &StageMinutes) -> Vec<StageGrowth> {
    let mut grown: Vec<StageGrowth> = previous
        .named()
        .into_iter()
        .zip(current.named())
        .filter_map(|((stage, before), (_, now))| {
            (now - before > GROWTH_EPSILON).then_some(StageGrowth {
                stage,
                previous_minutes: before,
                minutes: now,
            })
        })
        .collect();
    grown.sort_by(|a, b| b.added().total_cmp(&a.added()));
    grown
}

/// The median minutes of each stage in the window the `previous` review saw, or None when no measured job
/// is left in it.
///
/// A review records only the count of landed jobs it counted and the median of their totals, not the stage
/// medians. The window is rebuilt from the job records: the first `previous.landings` request jobs to land
/// ([`landed_jobs_among_first`], which cuts over every landed job so that jobs that ran no model do not move
/// the cut), then the newest [`THROUGHPUT_WINDOW`] measured ones, as the status section does. It is a
/// reconstruction: a landing record written after the review can have moved a job's landing time.
pub fn previous_stage_medians(
    records: &Records,
    previous: &ThroughputReview,
) -> Option<StageMinutes> {
    let jobs = landed_jobs_among_first(records, previous.landings as usize);
    let window = &jobs[jobs.len().saturating_sub(THROUGHPUT_WINDOW)..];
    (!window.is_empty()).then(|| StageMinutes::median_of(window))
}

/// The text of the slowdown request: the median rose from `previous_median` to `median` minutes per landed
/// job, and `growth` (from [`stage_growth`], the stage that gained the most first) lists the stages whose
/// medians grew. It is built with [`StewardRequest`] at high priority. The title starts with
/// [`SLOWDOWN_MARKER`] and names the stage that grew most with its measured median minutes; the body adds its
/// previous minutes and the other stages that grew. It does not cite a review id, because
/// [`throughput_followups`] reads any landed job that cites one as the change that implements that review's
/// proposal. Err when `growth` is empty, because a request must name a stage.
pub fn slowdown_request(
    previous_median: f64,
    median: f64,
    growth: &[StageGrowth],
) -> Result<String, String> {
    let Some(biggest) = growth.first() else {
        return Err("no stage's median grew, so there is no stage to name".to_string());
    };
    let stages = growth
        .iter()
        .map(|g| format!("{} {:.1} to {:.1}", g.stage, g.previous_minutes, g.minutes))
        .collect::<Vec<_>>()
        .join(", ");
    let title = format!(
        "{SLOWDOWN_MARKER}{} grew from {:.1} to {:.1} min per landed job",
        biggest.stage, biggest.previous_minutes, biggest.minutes
    );
    let body = format!(
        "Athena's throughput review found that the median minutes per landed job rose from \
         {previous_median:.1} to {median:.1}, more than 25%, since the previous throughput review. Find why the \
         stage that grew most now takes longer and remove the cause.\n\n\
         Stage that grew most: {}, a median of {:.1} minutes per landed job now, {:.1} before.\n\
         Stages whose median minutes grew, in minutes per landed job before to now: {stages}.\n\n\
         The medians are over the newest landed jobs that Athena measured. The status section of athena status \
         prints the same stage medians.",
        biggest.stage, biggest.minutes, biggest.previous_minutes,
    );
    let acceptance = [
        format!(
            "The cause of the longer {} stage is found and stated in the change or in a note under docs/design/",
            biggest.stage
        ),
        "The change shortens that stage, and the next throughput review shows the median minutes per landed job falling".to_string(),
    ];
    StewardRequest::new(
        Some(Priority::High),
        &title,
        &body,
        &acceptance,
        &["crates/".to_string(), "docs/design/".to_string()],
    )
    .map(|r| r.render())
}

/// The commit subject of a throughput review record: the median, then the design request and the slowdown
/// request the review queued, or that it queued none.
pub fn review_subject(median: f64, design: Option<&JobId>, slowdown: Option<&JobId>) -> String {
    let queued = match (design, slowdown) {
        (Some(d), Some(s)) => format!("design request {d}; slowdown request {s}"),
        (Some(d), None) => format!("design request {d}"),
        (None, Some(s)) => format!("slowdown request {s}"),
        (None, None) => "no request queued".to_string(),
    };
    format!("Throughput review: median {median:.1} min per landed job; {queued}")
}

/// What `athena drain` prints after the median for a throughput review: the design request it queued (and the
/// note, which can carry a slowdown remark), or why there is none.
pub fn throughput_outcome(r: &ThroughputReview) -> String {
    match r.proposal.as_ref() {
        Some(p) => match &p.request_job {
            Some(job) => {
                let queued = format!(
                    "design request {job} queued at {} priority, predicting {:.1} min saved per landed job",
                    p.priority, p.expected_minutes_saved
                );
                if r.note.is_empty() {
                    queued
                } else {
                    format!("{queued} · {}", r.note)
                }
            }
            None => format!("proposal recorded, no request queued: {}", r.note),
        },
        None => format!("no proposal: {}", r.note),
    }
}

/// The predictions of earlier reviews that a change has since closed, each set against the measured minutes.
/// A review's prediction is closed once, by the first landed job (other than the design request itself) whose
/// request text cites the review's id and that landed after the review. The measured median is that of the
/// newest [`FOLLOWUP_WINDOW`] measured landed jobs that landed after the change, and needs at least
/// [`FOLLOWUP_MIN_JOBS`]. `landed` is [`landed_jobs`], oldest first. A review whose prediction an earlier
/// review already closed is skipped.
pub fn throughput_followups(records: &Records, landed: &[LandedJob]) -> Vec<ThroughputFollowup> {
    let closed: BTreeSet<&RecordId> = records
        .throughput_reviews
        .iter()
        .flat_map(|r| r.followups.iter().map(|f| &f.review))
        .collect();
    let mut found = Vec::new();
    for review in &records.throughput_reviews {
        let Some(proposal) = &review.proposal else {
            continue;
        };
        if closed.contains(&review.id) {
            continue;
        }
        let cites = |l: &&LandedJob| {
            l.landed_at > review.at
                && proposal
                    .request_job
                    .as_ref()
                    .is_none_or(|j| j.as_str() != l.id)
                && records
                    .jobs
                    .get(&l.id)
                    .and_then(|j| j.request.as_ref())
                    .is_some_and(|r| r.text.contains(review.id.as_str()))
        };
        let Some(change) = landed.iter().find(cites) else {
            continue;
        };
        let after: Vec<&LandedJob> = landed
            .iter()
            .filter(|l| l.landed_at > change.landed_at)
            .collect();
        let newest = &after[after.len().saturating_sub(FOLLOWUP_WINDOW)..];
        if newest.len() < FOLLOWUP_MIN_JOBS {
            continue;
        }
        let measured = median(&newest.iter().map(|l| l.stages.total()).collect::<Vec<_>>());
        let Ok(change_job) = JobId::parse(&change.id) else {
            continue;
        };
        found.push(ThroughputFollowup {
            review: review.id.clone(),
            change_job,
            predicted_minutes_saved: proposal.expected_minutes_saved,
            baseline_median_minutes: review.window_median_minutes,
            measured_median_minutes: measured,
            measured_minutes_saved: review.window_median_minutes - measured,
            jobs_measured: newest.len() as u32,
        });
    }
    found
}

/// Removes a run's directory in the state directory when the run ends, however it ends.
struct Cleanup(PathBuf);

impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl Athena {
    /// The throughput review: when [`throughput_due`], asks the model for the biggest bottleneck and the
    /// architectural change that removes it, and records the review with the prediction, the wasted-work
    /// figures and the follow-ups it closes. An earlier design request that is still open does not stop it:
    /// the note and THROUGHPUT.md name that request. A design request for the proposal is queued only when
    /// [`request_gate`] finds a change beyond the noise between recent windows; the note says whether it was.
    /// Returns the record, or None when the review is not due, the run is nested, or the kill switch is on. A
    /// model that fails or answers badly still gives a record, without a proposal, so the cadence moves on;
    /// the invocation is recorded either way, since it cost money.
    ///
    /// Whether or not a design request is open, a due review also compares the latest stage medians with the
    /// previous review's, puts the stages that grew in the record's note and in a line to `ui`, and, when the
    /// median grew by more than 25% (and, with 60 or more landings, by more than the noise), queues a
    /// high-priority slowdown request (`compare_with_previous_review`).
    pub fn throughput_review(&self, ui: &dyn Ui) -> Result<Option<ThroughputReview>> {
        if self.nested.is_some() || self.state.registry().stopped() {
            return Ok(None);
        }
        let head = self.head()?;
        let records = Records::load(&self.git, &head)?;
        let Some(measured) = Throughput::compute(&records) else {
            return Ok(None);
        };
        let latest = ThroughputReview::latest(&records.throughput_reviews);
        let waste = measured.waste.figures();
        let Some(trigger) = throughput_due(
            measured.landed_total,
            measured.median_minutes,
            Some(waste),
            latest,
        ) else {
            return Ok(None);
        };
        let now = crate::now();
        let mut id = RecordId::generate(now);
        while records.has_record_id(&id) {
            id = RecordId::generate(now);
        }
        let followups = throughput_followups(&records, &landed_jobs(&records));
        // Compare with the previous review before anything else, so that an open design request cannot
        // hide a slowdown.
        let (remark, slowdown_job) = self.compare_with_previous_review(&records, latest, &measured);

        // An open design request does not stop the review: it is named in the note and in THROUGHPUT.md,
        // and the model is asked. Whether a second request is queued is the gate's decision.
        let open = open_throughput_request(&records);
        let gate = request_gate(
            &measured.measures(),
            measured.landed_total,
            latest.is_none(),
            open.is_some(),
        );
        let mut notes: Vec<String> = Vec::new();
        if let Some(open) = open {
            notes.push(format!(
                "throughput design request {} is still open; this review ran anyway",
                open.id()
            ));
        }
        notes.push(gate.note.clone());
        let mut proposal = None;
        let (made, answer) = self.ask_throughput(&head, &measured, trigger, now, open)?;
        let invocation: Option<Invocation> = made;
        match answer {
            Ok(answer) => {
                let priority = priority_for_minutes(answer.expected_minutes_saved);
                let request_job = if !gate.queue {
                    None
                } else {
                    match throughput_request(&id, &answer, measured.median_minutes) {
                        Ok(text) => match self.queue(&text) {
                            Ok((job, _)) => Some(job),
                            Err(e) => {
                                notes
                                    .push(format!("the design request could not be queued: {e:#}"));
                                None
                            }
                        },
                        Err(e) => {
                            notes.push(format!("the design request could not be built: {e}"));
                            None
                        }
                    }
                };
                proposal = Some(ThroughputProposal {
                    bottleneck: one_line(&answer.bottleneck),
                    stage: one_line(&answer.stage),
                    change: one_line(&answer.change),
                    expected_minutes_saved: answer.expected_minutes_saved,
                    priority: priority.as_str().to_string(),
                    request_job,
                });
            }
            Err(e) => notes.push(format!("no proposal: {e:#}")),
        }
        if !remark.is_empty() {
            notes.push(remark.clone());
        }
        let review = ThroughputReview {
            id,
            at: now,
            landings: u32::try_from(measured.landed_total).unwrap_or(u32::MAX),
            window_median_minutes: measured.median_minutes,
            trigger,
            proposal,
            waste: Some(waste),
            followups,
            note: notes.join("; "),
        };
        review.validate().map_err(|e| anyhow::anyhow!(e))?;
        let record = AnyRecord::ThroughputReview(Box::new(review.clone()));
        let mut files = vec![NewFile::record(&record)];
        let mut trailers = vec![("Athena-Record", record.canonical_path())];
        if let Some(invocation) = invocation {
            let invocation_record = AnyRecord::Invocation(Box::new(invocation));
            trailers.push(("Athena-Record", invocation_record.canonical_path()));
            files.push(NewFile::record(&invocation_record));
        }
        let subject = review_subject(
            review.window_median_minutes,
            review
                .proposal
                .as_ref()
                .and_then(|p| p.request_job.as_ref()),
            slowdown_job.as_ref(),
        );
        self.commit_records(
            &files,
            &message(
                &subject,
                "A review of how long landed jobs take and what the pipeline waits on. It records the prediction the \
                 proposal makes and, for an earlier proposal whose change has landed, the minutes measured since.",
                &trailers,
            ),
        )?;
        for f in &review.followups {
            ui.line(&format!(
                "hestia     throughput review {}: change {} landed; predicted {:.1} min saved per landed job, measured {:.1} (median {:.1} to {:.1} over {} job(s))",
                f.review,
                f.change_job,
                f.predicted_minutes_saved,
                f.measured_minutes_saved,
                f.baseline_median_minutes,
                f.measured_median_minutes,
                f.jobs_measured
            ));
        }
        if !remark.is_empty() {
            ui.line(&format!(
                "hestia     throughput review {}: {remark}",
                review.id
            ));
        }
        Ok(Some(review))
    }

    /// Sets the latest stage medians against the ones the previous review saw ([`previous_stage_medians`]),
    /// whether or not a design request is open. Returns a remark naming the stages whose median minutes grew,
    /// largest first (empty when none did, or when there is no previous review), and the slowdown request
    /// job when one was queued.
    ///
    /// When the median minutes per landed job grew by more than 25% ([`is_slowdown`]) and a stage grew, a
    /// high-priority request naming the stage that grew most is queued ([`slowdown_request`]), unless an
    /// open request with [`SLOWDOWN_MARKER`] already exists, or the rise over the previous review's median is
    /// no more than the noise between recent windows and the one-minute floor ([`slowdown_beyond_noise`];
    /// with under 60 landings the noise cannot be estimated, the remark says so and the 25% rule alone
    /// decides). The comparison is code, not the model, and it never queues a design request. When the
    /// median rose but no stage's median did, or the previous window cannot be rebuilt, nothing is queued and
    /// the remark says so.
    fn compare_with_previous_review(
        &self,
        records: &Records,
        latest: Option<&ThroughputReview>,
        measured: &Throughput,
    ) -> (String, Option<JobId>) {
        let Some(previous) = latest else {
            return (String::new(), None);
        };
        let growth = previous_stage_medians(records, previous)
            .map(|before| stage_growth(&before, &measured.median_stages));
        let mut parts = Vec::new();
        if let Some(grown) = growth.as_deref().filter(|g| !g.is_empty()) {
            let stages = grown
                .iter()
                .map(|g| format!("{} {:.1} to {:.1}", g.stage, g.previous_minutes, g.minutes))
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!(
                "median {:.1} to {:.1} min per landed job; stages that grew: {stages}",
                previous.window_median_minutes, measured.median_minutes
            ));
        }
        let mut slowdown_job = None;
        if is_slowdown(previous.window_median_minutes, measured.median_minutes) {
            match growth.as_deref() {
                None => parts.push(
                    "the median grew by more than 25% but the previous review's stage medians could not be rebuilt; no slowdown request queued"
                        .to_string(),
                ),
                Some([]) => parts.push(
                    "the median grew by more than 25% but no stage's median did; no slowdown request queued"
                        .to_string(),
                ),
                Some(grown) => {
                    let rise = measured.median_minutes - previous.window_median_minutes;
                    match slowdown_beyond_noise(
                        rise,
                        &measured.bars.median_minutes,
                        measured.landed_total,
                    ) {
                        // The rise is no more than the noise between recent windows: no request.
                        Err(within) => parts.push(within),
                        Ok(remark) => {
                            parts.extend(remark);
                            if let Some(open) = open_request_with_marker(records, SLOWDOWN_MARKER) {
                                parts.push(format!("slowdown request {} is still open", open.id()));
                            } else {
                                match slowdown_request(
                                    previous.window_median_minutes,
                                    measured.median_minutes,
                                    grown,
                                ) {
                                    Ok(text) => match self.queue(&text) {
                                        Ok((job, _)) => {
                                            parts.push(format!(
                                                "slowdown request {job} queued at high priority"
                                            ));
                                            slowdown_job = Some(job);
                                        }
                                        Err(e) => parts.push(format!(
                                            "the slowdown request could not be queued: {e:#}"
                                        )),
                                    },
                                    Err(e) => parts.push(format!(
                                        "the slowdown request could not be built: {e}"
                                    )),
                                }
                            }
                        }
                    }
                }
            }
        }
        (parts.join("; "), slowdown_job)
    }

    /// Runs the model once over a scratch workspace holding the throughput section and the design documents.
    /// Returns the recorded invocation (None when the model could not start) and its validated, redacted answer.
    fn ask_throughput(
        &self,
        head: &str,
        measured: &Throughput,
        trigger: ThroughputTrigger,
        now: Timestamp,
        open: Option<&JobRecords>,
    ) -> Result<(Option<Invocation>, Result<ThroughputOutput>)> {
        let jc = JobConfig::at(&self.git, head)?;
        let run_job = JobId::Id(RecordId::generate(now));
        let job_dir = self.state.job_dir(&run_job);
        let _cleanup = Cleanup(job_dir.clone());
        let workspace = job_dir.join("workspace");
        std::fs::create_dir_all(&workspace)?;
        let open_note = open.map_or_else(String::new, |job| {
            let title = job
                .request
                .as_ref()
                .and_then(|r| r.text.lines().find(|l| l.starts_with(THROUGHPUT_MARKER)))
                .unwrap_or_default();
            format!(
                "An earlier design request is still open: {} ({}). Propose a change that differs from it.\n\n",
                job.id(),
                one_line(title)
            )
        });
        std::fs::write(
            workspace.join(THROUGHPUT_FILE),
            format!(
                "# Throughput\n\nThis review ran because of: {}.\n\n{open_note}```\n{}\n```\n\nThe stages are minutes of the model \
                 invocations of that role, minutes the checks took, and, for landing and rebuild, the time from the \
                 end of the last invocation (the verifier) to the later of the landing outcome and the landing \
                 record (landings/), which is written after the CI wait, push and merge. Idle time and the time on \
                 jobs that did not land are time spent (invocation wall times, check durations and the landing \
                 wait), not the time a job stayed open. Each figure's basis is printed under it.\n",
                trigger.as_str(),
                measured.text()
            ),
        )?;
        let listing = self
            .git
            .run(["ls-tree", "-r", "--name-only", head, "--", "docs/design"])?;
        for path in listing.lines().filter(|p| p.ends_with(".md")) {
            let Some(bytes) = self.git.show_file(head, path)? else {
                continue;
            };
            let text: String = String::from_utf8_lossy(&bytes)
                .chars()
                .take(MAX_DOC_CHARS)
                .collect();
            let to = workspace.join(path.strip_prefix("docs/").unwrap_or(path));
            if let Some(parent) = to.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(to, text)?;
        }

        let composed: Composed = compose(&[
            CONTRACT.into(),
            Component::new("throughput.prompt", 1, THROUGHPUT_PROMPT),
        ]);
        let grant = ToolGrant {
            tools: vec!["Read".into(), "Grep".into(), "Glob".into()],
            commands: BTreeMap::new(),
            writes: false,
            permission_mode: "dontAsk".into(),
        };
        let brief = format!(
            "# Throughput review\n\nIn the current directory, `{THROUGHPUT_FILE}` holds the measurements and \
             `design/` holds Athena's design documents. Read them with Read, Grep and Glob; nothing else is \
             available and nothing may be changed.\n\nName the one biggest bottleneck and the architectural \
             change to the pipeline that removes it. Return `bottleneck`, `stage`, `change`, \
             `expected_minutes_saved` (minutes saved per landed job) and `rationale`."
        );
        // The review borrows the verifier's model and budget, as the steward, the audience and the auditor do.
        let spec = self.agent_spec(
            &jc,
            &run_job,
            1,
            Role::Steward,
            &jc.config.models.verifier,
            &composed,
            brief,
            &grant,
            &workspace,
            Side::Read,
            &BTreeMap::new(),
            jc.config.budget.verifier_usd,
            jc.config.budget.verifier_timeout_s,
            throughput_schema(),
        )?;
        let facts = crate::invocation::InvocationFacts {
            job: &run_job,
            spec: &spec,
            composed: &composed,
            config: &jc.config,
            fake: self.is_fake(),
        };
        let redactor = self.redactor(&jc.config, Some(&run_job))?;
        let (output, invocation) = match self.invoke_and_record(&facts, &redactor)? {
            Ok(done) => done,
            Err(error) => {
                return Ok((
                    None,
                    Err(anyhow::anyhow!(
                        "the throughput review could not run: {error}"
                    )),
                ));
            }
        };
        let answer = if output.outcome == InvocationOutcome::Success {
            serde_json::from_value::<ThroughputOutput>(output.structured.unwrap_or_default())
                .context("the model returned no valid throughput proposal")
        } else {
            Err(anyhow::anyhow!(
                "the model did not complete: {}",
                invocation.detail.as_deref().unwrap_or("no detail")
            ))
        };
        let answer = answer.and_then(|mut a| {
            a.validate()?;
            // What the model wrote goes into records, so it is redacted first.
            let mut words = vec![
                a.bottleneck.clone(),
                a.stage.clone(),
                a.change.clone(),
                a.rationale.clone(),
            ];
            redactor.redact_all(&mut words)?;
            let mut words = words.into_iter();
            a.bottleneck = words.next().unwrap_or_default();
            a.stage = words.next().unwrap_or_default();
            a.change = words.next().unwrap_or_default();
            a.rationale = words.next().unwrap_or_default();
            Ok(a)
        });
        Ok((Some(invocation), answer))
    }

    /// The error review backfill. Athena runs it at the start of every drain and in each sweep, and
    /// `athena hestia review --backfill` runs it on request. It reviews the repeated causes already in the
    /// records, which the review that runs after each new introspection never saw. See the module comment for the rules. Safe to run
    /// again: a cause that an open request covers is skipped, so a second run changes nothing.
    pub fn error_review_backfill(&self, ui: &dyn Ui) -> Result<BackfillReport> {
        let head = self.head()?;
        let records = Records::load(&self.git, &head)?;
        let config = crate::capability::JobConfig::at(&self.git, &head)?.config;
        let disabled = Stage::disabled(&config);
        let mut report = BackfillReport::default();
        for planned in plan_backfill(&records, &disabled) {
            let line = self.apply_step(planned, ui)?;
            report.lines.push(line);
        }
        Ok(report)
    }

    fn apply_step(&self, planned: Planned, ui: &dyn Ui) -> Result<BackfillLine> {
        let Planned {
            cause,
            priority,
            step,
        } = planned;
        let level = priority.map(|p| p.as_str().to_string());
        let n = cause.count();
        let mut line = BackfillLine {
            action: BackfillAction::Skipped,
            cause: cause.cause.clone(),
            introspections: cause.introspections.clone(),
            priority: level.clone(),
            job: None,
            replaced: None,
            reason: String::new(),
        };
        match step {
            Step::Skip { job, reason } => {
                line.job = job;
                line.reason = reason;
            }
            Step::Queue { text } => {
                let (job, _) = self.queue(&text)?;
                line.action = BackfillAction::Queued;
                line.job = Some(job.as_str().to_string());
                line.reason = format!(
                    "no open request addressed it; root-cause request queued citing {n} introspection(s)"
                );
            }
            Step::Requeue { old, text } => {
                let (job, _) = self.queue(&text)?;
                let why = format!(
                    "re-queued as {job} through the steward request builder, with acceptance, files and priority {}: the cause repeated in {} jobs",
                    level.as_deref().unwrap_or("normal"),
                    cause.jobs.len()
                );
                self.close(old.as_str(), &why, ui)
                    .with_context(|| format!("closing {old} after queuing {job} in its place"))?;
                line.action = BackfillAction::Requeued;
                line.job = Some(job.as_str().to_string());
                line.reason = format!("open request {old} addressed it; {why}");
                line.replaced = Some(old.as_str().to_string());
            }
            Step::Reprioritise { job } => {
                let to = level.as_deref().unwrap_or("normal");
                self.set_priority(job.as_str(), to)?;
                line.action = BackfillAction::Reprioritised;
                line.job = Some(job.as_str().to_string());
                line.reason = format!(
                    "queued request {job} already covers the cause; its priority is now {to}"
                );
            }
        }
        Ok(line)
    }

    /// The error review, run after `recorded` was committed as an introspection: groups it with the earlier
    /// introspections by cause and, when the cause now occurs in two or more distinct jobs and no open or
    /// landed request cites it, queues one high-priority root-cause request citing the introspection ids. Returns the
    /// job queued, if any. Never fails: a problem is a line to the user, since the review never blocks a job.
    pub fn error_review(&self, recorded: &IntrospectionRecord, ui: &dyn Ui) -> Option<JobId> {
        // A proposal to relax or remove a gate is not a cause: its evidence is the gate's own counts, and it
        // queues a review of that gate instead of a root-cause request.
        if recorded
            .prevention_kind
            .is_some_and(PreventionKind::loosens)
        {
            return match self.gate_review(recorded) {
                Ok(Some((job, count))) => {
                    ui.line(&format!(
                        "hestia     gate {} ({}); review request {job} queued at normal priority",
                        count.gate,
                        count.describe()
                    ));
                    Some(job)
                }
                Ok(None) => None,
                Err(error) => {
                    ui.line(&format!(
                        "hestia     the gate review could not run: {error:#}"
                    ));
                    None
                }
            };
        }
        match self.try_error_review(recorded) {
            Ok(Some((job, cause))) => {
                ui.line(&format!(
                    "hestia     repeated cause in {} jobs ({} introspections); root-cause request {job} queued at high priority",
                    cause.jobs.len(),
                    cause.count()
                ));
                Some(job)
            }
            Ok(None) => None,
            Err(error) => {
                ui.line(&format!(
                    "hestia     the error review could not run: {error:#}"
                ));
                None
            }
        }
    }

    /// The gate review of a recorded `relax` or `remove` introspection: queues one request that reviews the
    /// gate it names, citing the gate's counts, and returns the job and the counts. Queues nothing (None)
    /// when the introspection is not of those kinds, names no gate, names a gate that no later try overruled
    /// (checked again here, on the records at HEAD), when a request already cites the introspection, or when
    /// an open request already reviews the gate ([`open_gate_request`]); a gate whose review has closed may be
    /// reviewed again.
    pub fn gate_review(
        &self,
        recorded: &IntrospectionRecord,
    ) -> Result<Option<(JobId, GateCount)>> {
        let Some(kind) = recorded.prevention_kind.filter(|k| k.loosens()) else {
            return Ok(None);
        };
        if recorded.status != IntrospectionStatus::Recorded {
            return Ok(None);
        }
        let Some(gate) = recorded.prevention.as_deref().and_then(gate_of_prevention) else {
            return Ok(None);
        };
        let records = Records::load(&self.git, &self.head()?)?;
        let Some(count) = gate_counts(&records)
            .into_iter()
            .find(|c| c.gate == gate && c.ends >= 1 && c.disagreements >= 1)
        else {
            return Ok(None);
        };
        if addressing_request(&records, &[recorded.id.to_string()]).is_some()
            || open_gate_request(&records, gate).is_some()
        {
            return Ok(None);
        }
        let text = gate_request(&count, kind, recorded)
            .and_then(|text| plain_boundary_text(&text))
            .map_err(|e| anyhow::anyhow!("the gate request: {e}"))?;
        let (job, _) = self.queue(&text)?;
        Ok(Some((job, count)))
    }

    fn try_error_review(
        &self,
        recorded: &IntrospectionRecord,
    ) -> Result<Option<(JobId, RepeatedCause)>> {
        let records = Records::load(&self.git, &self.head()?)?;
        let Some(key) = cause_key(&records, recorded) else {
            return Ok(None);
        };
        let Some(cause) = repeated_causes(&records)
            .into_iter()
            .find(|c| c.key == key && c.introspections.contains(&recorded.id.to_string()))
        else {
            return Ok(None);
        };
        if cause.request.is_some() {
            return Ok(None);
        }
        let text = root_cause_request(&cause, &records)
            .and_then(|text| plain_boundary_text(&text))
            .map_err(|e| anyhow::anyhow!("the root-cause request: {e}"))?;
        let (job, _) = self.queue(&text)?;
        Ok(Some((job, cause)))
    }

    /// The records as `athena status` reads them: the `athena/records` worktree when it exists, else `records/`
    /// at HEAD.
    fn current_records(&self) -> Result<Records> {
        let head = self.head()?;
        let worktree = athena_data::paths::records_worktree(self.state.root());
        Records::load_current(&self.git, &head, &worktree)
    }

    /// The context of a duty pass started from `occasion`: the landing total on HEAD now and the time.
    pub fn duty_context(&self, occasion: Occasion) -> Result<DutyContext> {
        let head = self.head()?;
        Ok(DutyContext {
            occasion,
            landings: crate::code_health::landings(&self.git, &head)?.len() as u32,
            now: crate::now(),
        })
    }

    /// Runs the duties of `table` that are due under `ctx`, in table order, and returns their results. Which
    /// duties are due is decided from the sweeps already recorded ([`is_due`]); nothing is written here.
    pub fn execute_duties(
        &self,
        table: &[Duty],
        runner: &dyn DutyRunner,
        ctx: &DutyContext,
        ui: &dyn Ui,
    ) -> Result<DutyReport> {
        let records = self.current_records()?;
        let mut report = DutyReport::default();
        for duty in table {
            let past = past_runs(&records.hestia_sweeps, duty.id);
            if !is_due(duty, &past, ctx) {
                continue;
            }
            let outcome = runner.run(duty, ui, ctx.now);
            let outcome = match outcome {
                Ok(output) => {
                    report.lines.extend(output.lines);
                    Ok(output.check)
                }
                Err(error) => {
                    report.lines.push(duty_error_line(duty, &error));
                    Err(error)
                }
            };
            let (result, findings) = settle(duty.id, outcome);
            report.results.push(result);
            report.findings.extend(findings);
        }
        Ok(report)
    }

    /// Runs the due duties of `table` with `runner`, prints their lines, and commits one sweep that lists `checks`
    /// and the duties that ran, with the landing total of `ctx`. Commits a sweep even when no duty was due, so
    /// the record shows when the sweep ran.
    pub fn hestia_sweep_duties(
        &self,
        checks: &[Box<dyn Check>],
        ui: &dyn Ui,
        table: &[Duty],
        runner: &dyn DutyRunner,
        ctx: DutyContext,
    ) -> Result<Sweep> {
        let report = self.execute_duties(table, runner, &ctx, ui)?;
        for line in &report.lines {
            ui.line(line);
        }
        self.commit_sweep(checks, Some((report, ctx)))
    }

    /// Runs the duties of [`DUTIES`] that are due for a pass started from `occasion`, records the pass when any
    /// ran, and returns the lines to tell the user. Never fails: a problem is a line.
    pub fn run_due_duties(&self, occasion: Occasion, ui: &dyn Ui) -> Vec<String> {
        match self.git.head() {
            Ok(Some(_)) => {}
            Ok(None) => return Vec::new(),
            Err(e) => return vec![format!("hestia     duties: could not run: {e:#}")],
        }
        let ctx = match self.duty_context(occasion) {
            Ok(ctx) => ctx,
            Err(e) => return vec![format!("hestia     duties: could not run: {e:#}")],
        };
        let mut report =
            match self.execute_duties(&DUTIES, &AthenaDuties { athena: self }, &ctx, ui) {
                Ok(report) => report,
                Err(e) => return vec![format!("hestia     duties: could not run: {e:#}")],
            };
        let mut lines = std::mem::take(&mut report.lines);
        if !report.results.is_empty()
            && let Err(e) = self.commit_sweep(&[], Some((report, ctx)))
        {
            lines.push(format!(
                "hestia     the duty pass could not be recorded: {e:#}"
            ));
        }
        lines
    }

    /// Athena's own housekeeping, run at the start of every drain so that no person has to run
    /// `athena hestia review --backfill` or `athena score-jev --derive` (both commands stay available).
    /// Runs the drain-start duties of [`DUTIES`] (the error review backfill and the evidence-label derivation),
    /// records the pass, and returns one summary line for each, then the lines of [`Athena::extra_review_passes`].
    /// Both duties are idempotent: over unchanged records they change nothing. A duty that fails is reported in
    /// its line and does not stop the other or the caller.
    pub fn review_passes(&self, ui: &dyn Ui) -> Vec<String> {
        let mut lines = self.run_due_duties(Occasion::DrainStart, ui);
        lines.extend(self.extra_review_passes(ui));
        lines
    }

    /// The review passes that are not duties: the spec review and the throughput review.
    fn extra_review_passes(&self, ui: &dyn Ui) -> Vec<String> {
        let spec = match self.spec_review(ui) {
            Ok(r) => {
                let mut line = format!(
                    "hestia     spec: {} requirement(s) without a mechanism or a valid measure · queued {} · already open {} · left for a later pass {}",
                    r.gaps,
                    r.queued.len(),
                    r.already_open,
                    r.deferred
                );
                if !r.failed.is_empty() {
                    line.push_str(&format!(" · could not queue: {}", r.failed.join("; ")));
                }
                line
            }
            Err(e) => format!("hestia     spec: could not run: {e:#}"),
        };
        let mut lines = vec![spec];
        // The throughput review is due every ten landings or on a 20% slowdown, so most calls print nothing.
        match self.throughput_review(ui) {
            Ok(Some(r)) => {
                let outcome = throughput_outcome(&r);
                lines.push(format!(
                    "hestia     throughput review ({}): median {:.1} min per landed job over {} landings · {outcome}",
                    r.trigger.as_str(),
                    r.window_median_minutes,
                    r.landings
                ));
            }
            Ok(None) => {}
            Err(e) => lines.push(format!(
                "hestia     throughput review: could not run: {e:#}"
            )),
        }
        lines
    }

    /// The build measurement pass: runs [`Athena::measure_build`] when [`measure_build_due`] says so (no
    /// measurement exists, or `Cargo.lock` at HEAD is not the one the latest measurement built) and returns the
    /// new record. Returns None, and builds nothing, when it is not due, when HEAD has no `Cargo.lock`, in a
    /// nested run, with the kill switch on, or when free disk space is below `[disk] min_free_gb` (a line says
    /// so, and the next sweep tries again). A measurement that fails is an error, so the next sweep tries again.
    pub fn measure_build_pass(
        &self,
        builder: &dyn CargoBuilder,
        ui: &dyn Ui,
    ) -> Result<Option<BuildMeasurement>> {
        if self.nested.is_some() || self.state.registry().stopped() {
            return Ok(None);
        }
        let Some(head) = self.git.head()? else {
            return Ok(None);
        };
        let Some(lock_hash) = cargo_lock_blob(&self.git, &head)? else {
            return Ok(None);
        };
        let records = Records::load(&self.git, &head)?;
        if !measure_build_due(
            BuildMeasurement::latest(&records.build_measurements),
            &lock_hash,
        ) {
            return Ok(None);
        }
        let required = JobConfig::at(&self.git, &head)?
            .config
            .min_free_gb()
            .saturating_mul(GB);
        if let Some(free) = self.state.free_bytes()
            && free < required
        {
            ui.line(&format!(
                "hestia     build measurement due but not run: {} free, {} required",
                format_bytes(free),
                format_bytes(required)
            ));
            return Ok(None);
        }
        self.measure_build(builder, ui).map(Some)
    }

    /// The sweep with the build measurement pass in front: runs [`Athena::measure_build_pass`] with `builder`
    /// (its outcome goes to `ui`; a failure does not stop the sweep), then [`Athena::hestia_sweep`] over
    /// `checks`. The model-driven review passes are not run here; [`Athena::hestia_sweep_with`] runs them first.
    pub fn hestia_sweep_measuring(
        &self,
        checks: &[Box<dyn Check>],
        ui: &dyn Ui,
        builder: &dyn CargoBuilder,
    ) -> Result<Sweep> {
        self.report_build_measurement(ui, builder);
        self.hestia_sweep(checks)
    }

    /// Runs the build measurement pass with `builder` and tells `ui` what it did; a failure is reported and
    /// does not stop the sweep.
    fn report_build_measurement(&self, ui: &dyn Ui, builder: &dyn CargoBuilder) {
        match self.measure_build_pass(builder, ui) {
            Ok(Some(m)) => ui.line(&format!(
                "hestia     build measurement {} recorded for Cargo.lock {}",
                m.id,
                &m.cargo_lock_hash[..m.cargo_lock_hash.len().min(12)]
            )),
            Ok(None) => {}
            Err(e) => ui.line(&format!(
                "hestia     build measurement: could not run: {e:#}"
            )),
        }
    }

    /// The sweep of `athena hestia sweep`. Runs the duties of [`DUTIES`] that are due (a duty that ran within its
    /// cadence is not run again; see [`is_due`]) and `checks`, commits one report that lists them with the
    /// landing total, and returns it. The duties' summary lines go to `ui`, then the lines of
    /// [`Athena::extra_review_passes`], then the build measurement pass ([`Athena::measure_build_pass`]) with the
    /// real builder.
    pub fn hestia_sweep_with(&self, checks: &[Box<dyn Check>], ui: &dyn Ui) -> Result<Sweep> {
        if self.git.head()?.is_none() {
            return self.commit_sweep(checks, None);
        }
        let ctx = self.duty_context(Occasion::Sweep)?;
        let sweep =
            self.hestia_sweep_duties(checks, ui, &DUTIES, &AthenaDuties { athena: self }, ctx)?;
        for line in self.extra_review_passes(ui) {
            ui.line(&line);
        }
        self.report_build_measurement(ui, &CargoBuild);
        Ok(sweep)
    }

    /// Runs `checks`, commits the report as a record, and returns it. Prints nothing and runs no duty; the sweep
    /// that also runs the duties and the review passes is [`Athena::hestia_sweep_with`].
    pub fn hestia_sweep(&self, checks: &[Box<dyn Check>]) -> Result<Sweep> {
        self.commit_sweep(checks, None)
    }

    /// The reading rule check over the records at `head` (the `athena/records` worktree when it exists, as
    /// `athena status` reads them). A failure to read them is the check's error, which the sweep reports.
    fn reading_rules_check(&self, head: &str) -> ReadingRulesCheck {
        let worktree = athena_data::paths::records_worktree(self.state.root());
        let stats = Records::load_current(&self.git, head, &worktree)
            .map(|records| ReadingRuleStats::compute(&records))
            .map_err(|e| format!("{e:#}"));
        ReadingRulesCheck::new(stats)
    }

    /// Runs `checks`, adds the results of a duty pass when there is one (its occasion and landing total are
    /// stored in the record), and commits one report.
    fn commit_sweep(
        &self,
        checks: &[Box<dyn Check>],
        duties: Option<(DutyReport, DutyContext)>,
    ) -> Result<Sweep> {
        let known = match self.git.head()? {
            Some(head) => Records::load(&self.git, &head)?,
            None => bail!(
                "the repository has no commits yet; commit athena.toml and defaults.toml first"
            ),
        };
        let now = duties.as_ref().map_or_else(crate::now, |(_, ctx)| ctx.now);
        let mut id = RecordId::generate(now);
        while known.has_record_id(&id) {
            id = RecordId::generate(now);
        }
        let mut sweep = run_sweep(checks, id, now);
        if let Some((report, ctx)) = duties {
            sweep.checks.extend(report.results);
            sweep.findings.extend(report.findings);
            sweep.duty_pass = Some(DutyPass {
                occasion: ctx.occasion,
                landings: ctx.landings,
            });
        }
        sweep.validate().map_err(|e| anyhow::anyhow!(e))?;
        let record = AnyRecord::HestiaSweep(Box::new(sweep.clone()));
        let subject = format!(
            "Hestia sweep: {} check(s), {} finding(s)",
            sweep.checks.len(),
            sweep.findings.len()
        );
        self.commit_records(
            &[NewFile::record(&record)],
            &message(
                &subject,
                "A report of Hestia's checks. It lists every check that ran with its count; a check that failed to run is a finding.",
                &[("Athena-Record", record.canonical_path())],
            ),
        )?;
        Ok(sweep)
    }

    /// The newest recorded sweep, or None when none was recorded. A drain-start or landing duty pass lists only
    /// the duties that ran in it, so it does not replace the last full sweep; when every record is such a pass,
    /// the newest of them is returned.
    pub fn last_sweep(&self) -> Result<Option<Sweep>> {
        let Some(head) = self.git.head()? else {
            return Ok(None);
        };
        let records = Records::load(&self.git, &head)?;
        let full: Vec<Sweep> = records
            .hestia_sweeps
            .iter()
            .filter(|s| !s.is_duty_pass())
            .cloned()
            .collect();
        let from = if full.is_empty() {
            &records.hestia_sweeps
        } else {
            &full
        };
        Ok(Sweep::latest(from).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Quiet;
    use crate::state::StateDir;
    use athena_agent::fake::FakeAgentDriver;
    use athena_data::records::Build;
    use athena_git::Git;

    /// A check with a fixed answer.
    struct Fixed {
        id: &'static str,
        answer: std::result::Result<CheckOutput, &'static str>,
    }

    impl Check for Fixed {
        fn id(&self) -> &str {
            self.id
        }
        fn run(&self, _: Timestamp) -> Result<CheckOutput> {
            self.answer.clone().map_err(|e| anyhow::anyhow!(e))
        }
    }

    fn clean(id: &'static str, examined: u32) -> Box<dyn Check> {
        Box::new(Fixed {
            id,
            answer: Ok(CheckOutput {
                examined,
                findings: Vec::new(),
            }),
        })
    }

    fn finding(check: &str, subject: &str) -> Finding {
        Finding {
            check: check.into(),
            subject: subject.into(),
            evidence: "merged into main three weeks ago".into(),
            age_seconds: 1_814_400,
            state: FindingState::Open,
        }
    }

    fn rid(text: &str) -> RecordId {
        RecordId::parse(text).unwrap()
    }

    #[test]
    fn a_sweep_where_every_check_finds_nothing_lists_every_check_with_a_count_of_zero() {
        let checks = vec![clean("alpha", 3), clean("beta", 0), clean("gamma", 12)];
        let sweep = run_sweep(&checks, rid("20261026-090000-5abc"), crate::now());
        assert_eq!(
            sweep
                .checks
                .iter()
                .map(|c| (c.check.as_str(), c.count, c.examined, c.errored))
                .collect::<Vec<_>>(),
            [
                ("alpha", 0, 3, false),
                ("beta", 0, 0, false),
                ("gamma", 0, 12, false)
            ]
        );
        assert!(sweep.findings.is_empty());
        assert!(sweep.validate().is_ok());
    }

    #[test]
    fn a_check_that_errors_becomes_a_finding_and_the_other_checks_still_run() {
        let checks: Vec<Box<dyn Check>> = vec![
            clean("alpha", 3),
            Box::new(Fixed {
                id: "broken",
                answer: Err("cannot read the branch list"),
            }),
            Box::new(Fixed {
                id: "noisy",
                answer: Ok(CheckOutput {
                    examined: 5,
                    // A check cannot file a finding under another check's id.
                    findings: vec![finding("someone-else", "old-branch")],
                }),
            }),
            clean("omega", 1),
        ];
        let sweep = run_sweep(&checks, rid("20261026-090000-5abc"), crate::now());
        assert_eq!(
            sweep
                .checks
                .iter()
                .map(|c| (c.check.as_str(), c.count, c.errored))
                .collect::<Vec<_>>(),
            [
                ("alpha", 0, false),
                ("broken", 1, true),
                ("noisy", 1, false),
                ("omega", 0, false)
            ]
        );
        let broken: Vec<&Finding> = sweep
            .findings
            .iter()
            .filter(|f| f.check == "broken")
            .collect();
        assert_eq!(broken.len(), 1);
        assert_eq!(broken[0].subject, "broken");
        assert!(
            broken[0].evidence.contains("cannot read the branch list"),
            "{}",
            broken[0].evidence
        );
        assert_eq!(broken[0].age_seconds, 0);
        assert_eq!(broken[0].state, FindingState::Open);
        let noisy = sweep.findings.iter().find(|f| f.check == "noisy").unwrap();
        assert_eq!(noisy.subject, "old-branch");
        assert_eq!(sweep.findings.len(), 2);
        assert!(sweep.validate().is_ok());
    }

    #[test]
    fn a_finding_without_evidence_is_reported_as_a_failed_check() {
        let mut bad = finding("x", "branch");
        bad.evidence = " ".into();
        let checks: Vec<Box<dyn Check>> = vec![Box::new(Fixed {
            id: "sloppy",
            answer: Ok(CheckOutput {
                examined: 1,
                findings: vec![bad],
            }),
        })];
        let sweep = run_sweep(&checks, rid("20261026-090000-5abc"), crate::now());
        assert!(sweep.checks[0].errored);
        assert!(sweep.findings[0].evidence.contains("no evidence"));
        assert!(sweep.validate().is_ok());
    }

    #[test]
    fn the_skeleton_registers_no_checks() {
        assert!(default_checks().is_empty());
    }

    fn athena_in(dir: &std::path::Path, state: &std::path::Path) -> Athena {
        athena_with(
            dir,
            state,
            std::sync::Arc::new(FakeAgentDriver::deterministic()),
        )
    }

    fn athena_with(
        dir: &std::path::Path,
        state: &std::path::Path,
        driver: std::sync::Arc<dyn athena_agent::AgentDriver>,
    ) -> Athena {
        let git = Git::at(dir);
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value]).unwrap();
        }
        // These tests are about the stages a switched-off planner leaves dormant, so the repository's
        // `[job_plan]` (planning on for hard jobs) is turned off here.
        let repo_config = include_str!("../../../athena.toml");
        let on = "[job_plan]\nenabled = true\nmode = \"hard\"\n";
        assert!(repo_config.contains(on), "athena.toml plans hard jobs");
        std::fs::write(
            dir.join("athena.toml"),
            repo_config.replace(on, "[job_plan]\nenabled = false\n"),
        )
        .unwrap();
        std::fs::write(
            dir.join("defaults.toml"),
            include_str!("../../../defaults.toml"),
        )
        .unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "Start"]).unwrap();
        Athena {
            git,
            state: StateDir::new(state.join("repo"))
                .with_free_space(crate::test_support::plenty_of_space),
            driver,
            capabilities: vec![],
            build: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            athena_bin: "/bin/false".into(),
            nested: None,
            pinner: Box::new(crate::fake::CountingPinner::new(&"0".repeat(40))),
            jev: None,
        }
    }

    #[test]
    fn a_committed_sweep_is_read_back_and_the_last_one_is_the_newest() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        assert_eq!(athena.last_sweep().unwrap(), None);

        let first = athena.hestia_sweep(&[clean("alpha", 2)]).unwrap();
        assert_eq!(athena.last_sweep().unwrap(), Some(first.clone()));
        let broken: Vec<Box<dyn Check>> = vec![
            clean("alpha", 2),
            Box::new(Fixed {
                id: "broken",
                answer: Err("boom"),
            }),
        ];
        let second = athena.hestia_sweep(&broken).unwrap();
        assert_ne!(first.id, second.id);
        let last = athena.last_sweep().unwrap().unwrap();
        assert_eq!(last, second);
        assert_eq!(last.checks.len(), 2);
        assert!(last.findings[0].evidence.contains("boom"));
        let path = format!("records/hestia-sweeps/{}/{}.json", last.id.month(), last.id);
        assert!(dir.path().join(&path).exists(), "{path}");
    }

    // -----------------------------------------------------------------------------------------------------
    // Reading rule refusals

    use crate::status::reading_rule_tests::{job_id, reading_job, records_of};

    /// Records of first readings: for each `(rule, text, count)`, `count` first readings that rule refused with
    /// that text, then `accepted` first readings accepted at once. Job `n` started at `10 n` seconds.
    fn first_readings(refusals: &[(&str, &str, usize)], accepted: usize) -> Records {
        let mut invocations = Vec::new();
        let mut n = 0u32;
        for (rule, text, count) in refusals {
            for _ in 0..*count {
                n += 1;
                invocations.extend(reading_job(n, i64::from(n) * 10, Some((rule, text)), None));
            }
        }
        for _ in 0..accepted {
            n += 1;
            invocations.extend(reading_job(n, i64::from(n) * 10, None, None));
        }
        records_of(invocations)
    }

    fn reading_check(records: &Records) -> Box<dyn Check> {
        Box::new(ReadingRulesCheck::new(Ok(ReadingRuleStats::compute(
            records,
        ))))
    }

    /// 20 first readings: rule A refused 7 (35%), four of them with one text and three with a text each,
    /// rule B refused 6 (exactly 30%), rule C refused 2, and 5 were accepted.
    fn twenty_readings() -> Records {
        first_readings(
            &[
                ("rule_a", "A's usual text", 4),
                ("rule_a", "A other one", 1),
                ("rule_a", "A other two", 1),
                ("rule_a", "A other three", 1),
                ("rule_b", "B text", 6),
                ("rule_c", "C text", 2),
            ],
            5,
        )
    }

    #[test]
    fn the_flag_limit_and_the_minimum_window_are_named_constants() {
        assert_eq!(REFUSAL_FLAG_PERCENT, 30);
        assert_eq!(MIN_FIRST_READINGS, 10);
    }

    #[test]
    fn a_rule_refusing_more_than_thirty_percent_is_flagged_with_its_commonest_text_and_one_at_thirty_is_not()
     {
        let stats = ReadingRuleStats::compute(&twenty_readings()).unwrap();
        assert_eq!(stats.first_readings, 20);
        let findings = reading_rule_findings(&stats);
        assert_eq!(findings.len(), 1, "{findings:?}");
        let flagged = &findings[0];
        assert_eq!(flagged.subject, "rule_a");
        assert_eq!(flagged.state, FindingState::Open);
        for part in [
            "rule_a",
            "7 of the last 20",
            "35.0%",
            "30%",
            "A's usual text",
            "4 time",
        ] {
            assert!(
                flagged.evidence.contains(part),
                "{part}: {}",
                flagged.evidence
            );
        }
        assert!(
            !flagged.evidence.contains("A other"),
            "{}",
            flagged.evidence
        );
        // Rule B refused exactly 30% and rule C 10%: neither is named.
        assert!(
            findings
                .iter()
                .all(|f| f.subject != "rule_b" && f.subject != "rule_c")
        );

        // Through the sweep: the finding is filed under the check, which examined the 20 first readings.
        let sweep = run_sweep(
            &[reading_check(&twenty_readings())],
            rid("20261101-000000-aaaa"),
            crate::now(),
        );
        assert_eq!(sweep.checks.len(), 1);
        assert_eq!(sweep.checks[0].check, READING_RULES_CHECK_ID);
        assert_eq!(
            (
                sweep.checks[0].examined,
                sweep.checks[0].count,
                sweep.checks[0].errored
            ),
            (20, 1, false)
        );
        assert_eq!(sweep.findings[0].check, READING_RULES_CHECK_ID);
        assert_eq!(sweep.findings[0].subject, "rule_a");
    }

    #[test]
    fn the_limit_is_decided_on_the_share_of_the_window_just_over_and_just_under() {
        // 50 first readings, the most the window holds: 16 refusals is 32%, 15 is exactly 30%.
        for (refused, flagged) in [(16, true), (15, false), (14, false)] {
            let records = first_readings(&[("rule_a", "text", refused)], 50 - refused);
            let stats = ReadingRuleStats::compute(&records).unwrap();
            assert_eq!(stats.first_readings, 50);
            assert_eq!(
                reading_rule_findings(&stats).len(),
                usize::from(flagged),
                "{refused} of 50"
            );
        }
    }

    #[test]
    fn a_window_with_no_readings_or_too_few_gives_no_flag_and_no_claim_of_health() {
        let nine_all_refused = first_readings(&[("rule_a", "text", MIN_FIRST_READINGS - 1)], 0);
        let stats = ReadingRuleStats::compute(&nine_all_refused).unwrap();
        assert_eq!(stats.first_readings, 9);
        assert!(reading_rule_findings(&stats).is_empty());
        for records in [Records::default(), nine_all_refused] {
            let sweep = run_sweep(
                &[reading_check(&records)],
                rid("20261101-000000-aaaa"),
                crate::now(),
            );
            let result = &sweep.checks[0];
            assert_eq!(result.check, READING_RULES_CHECK_ID);
            // Nothing was examined, nothing found, and the check did not fail: no "all clear" is recorded.
            assert_eq!(
                (result.examined, result.count, result.errored),
                (0, 0, false)
            );
            assert!(sweep.findings.is_empty());
        }
        // The minimum is inclusive: ten first readings, all refused, flag the rule.
        let ten = first_readings(&[("rule_a", "text", MIN_FIRST_READINGS)], 0);
        let findings = reading_rule_findings(&ReadingRuleStats::compute(&ten).unwrap());
        assert_eq!(findings.len(), 1);
        let text = format!("{:?}", findings).to_lowercase();
        assert!(!text.contains("healthy"), "{text}");
    }

    #[test]
    fn records_that_cannot_be_read_make_the_reading_check_error_not_pass() {
        let check: Vec<Box<dyn Check>> =
            vec![Box::new(ReadingRulesCheck::new(Err("no records".into())))];
        let sweep = run_sweep(&check, rid("20261101-000000-aaaa"), crate::now());
        assert!(sweep.checks[0].errored);
        assert!(sweep.findings[0].evidence.contains("no records"));
    }

    #[test]
    fn the_sweep_command_path_flags_the_rule_from_committed_invocation_records() {
        use crate::status::records_worktree_tests::{commit_in_tree, job_record};
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        // No reader invocation recorded: the check examines nothing and finds nothing.
        let empty = athena.hestia_sweep_with(&[], &Lines::default()).unwrap();
        let result = empty
            .checks
            .iter()
            .find(|c| c.check == READING_RULES_CHECK_ID)
            .expect("the sweep runs the reading rule check");
        assert_eq!((result.examined, result.count), (0, 0));
        assert!(
            empty
                .findings
                .iter()
                .all(|f| f.check != READING_RULES_CHECK_ID)
        );

        let fixture = twenty_readings();
        let mut files = Vec::new();
        for n in 1..=20u32 {
            files.push(job_record(job_id(n).as_str(), "docs"));
        }
        for job in fixture.jobs.values() {
            for inv in &job.invocations {
                files.push(AnyRecord::Invocation(Box::new(inv.clone())));
            }
        }
        commit_in_tree(&athena, &files);
        // The first sweep found the duty never run; the second runs it again only once five landings have
        // passed since, so the landing total is advanced here (the repository holds no landed commit).
        let ctx = DutyContext {
            occasion: Occasion::Sweep,
            landings: 5,
            now: crate::now(),
        };
        let reading = [DUTIES[5]];
        let sweep = athena
            .hestia_sweep_duties(
                &[],
                &Lines::default(),
                &reading,
                &AthenaDuties { athena: &athena },
                ctx,
            )
            .unwrap();
        let result = sweep
            .checks
            .iter()
            .find(|c| c.check == READING_RULES_CHECK_ID)
            .unwrap();
        assert_eq!((result.examined, result.count), (20, 1));
        let flagged: Vec<&Finding> = sweep
            .findings
            .iter()
            .filter(|f| f.check == READING_RULES_CHECK_ID)
            .collect();
        assert_eq!(flagged.len(), 1);
        assert_eq!(flagged[0].subject, "rule_a");
        assert!(flagged[0].evidence.contains("A's usual text"));
        // The committed report holds the same finding.
        assert_eq!(athena.last_sweep().unwrap().unwrap(), sweep);
        // The plain sweep without the review passes does not run the check.
        let plain = athena.hestia_sweep(&[]).unwrap();
        assert!(plain.checks.is_empty());
    }

    // -----------------------------------------------------------------------------------------------------
    // Error review

    use athena_data::records::{
        ClassStatus, FailureClass, IntrospectionJev, IntrospectionTrigger, JevClassAnswer, Outcome,
        PreventionKind, Request,
    };

    fn at(n: u32) -> Timestamp {
        format!("2026-10-27T09:{:02}:{:02}Z", n / 60, n % 60)
            .parse()
            .unwrap()
    }

    fn rec_id(n: u32) -> RecordId {
        RecordId::parse(&format!("20261027-09{:02}{:02}-1abc", n / 60, n % 60)).unwrap()
    }

    fn job(letter: char) -> JobId {
        JobId::parse(&format!(
            "20261027-0800{:02}-{}bcd",
            letter as u32 - 'a' as u32,
            letter
        ))
        .unwrap()
    }

    fn class(n: u32, name: &str) -> FailureClass {
        FailureClass {
            id: rec_id(n),
            at: at(n),
            name: name.into(),
            description: "a class".into(),
            status: ClassStatus::Proposed,
            proposed_by: rec_id(n + 1),
        }
    }

    /// A recorded introspection `n` about `of`, in class `in_class`, with `cause`.
    fn introspection(
        n: u32,
        of: char,
        in_class: Option<&RecordId>,
        cause: &str,
    ) -> IntrospectionRecord {
        IntrospectionRecord {
            id: rec_id(n),
            at: at(n),
            trigger: IntrospectionTrigger::Rejection,
            job: Some(job(of)),
            intervention: None,
            class_of_work: "code".into(),
            category: "rejected".into(),
            what_happened: Some("the job was rejected".into()),
            failure_class: in_class.cloned(),
            new_class: false,
            earlier_cases: Vec::new(),
            decisions_matched: Vec::new(),
            cause: Some(cause.into()),
            prevention_kind: Some(PreventionKind::CodeCheck),
            prevention: Some("Add a code check.".into()),
            proposed_request: Some("Add the check".into()),
            evidence: None,
            status: IntrospectionStatus::Recorded,
            failure: None,
            flags: Vec::new(),
            other_causes: Vec::new(),
            open_hypotheses: Vec::new(),
            contradictions: Vec::new(),
            dropped_ids: Vec::new(),
            planning_stopped: false,
            from_jev: false,
            jev: None,
        }
    }

    /// Writes the records into the repository and commits them, as the introspection tests do.
    fn put(dir: &std::path::Path, athena: &Athena, records: Vec<AnyRecord>) {
        for record in &records {
            let file = NewFile::record(record);
            let path = dir.join(&file.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, &file.bytes).unwrap();
        }
        athena.git.run(["add", "-A"]).unwrap();
        athena
            .git
            .run(["commit", "-q", "-m", "Record for the test"])
            .unwrap();
    }

    fn put_introspection(dir: &std::path::Path, athena: &Athena, i: &IntrospectionRecord) {
        put(
            dir,
            athena,
            vec![AnyRecord::Introspection(Box::new(i.clone()))],
        );
    }

    fn records(athena: &Athena) -> Records {
        Records::load(&athena.git, &athena.head().unwrap()).unwrap()
    }

    /// One `[[requirement]]` of a fixture spec, owned by Hestia.
    fn req(id: &str, statement: &str, status: &str, mechanism: &str, measure: &str) -> String {
        format!(
            "[[requirement]]\nid = \"{id}\"\nstatement = \"{statement}\"\nkind = \"requirement\"\nstatus = \"{status}\"\nmechanism = \"{mechanism}\"\nowner = \"Hestia\"\nmeasure = \"{measure}\"\n\n"
        )
    }

    /// Commits a fixture `docs/spec.toml`.
    fn put_spec(dir: &std::path::Path, athena: &Athena, text: &str) {
        std::fs::create_dir_all(dir.join("docs")).unwrap();
        std::fs::write(dir.join(SPEC_FILE), text).unwrap();
        athena.git.run(["add", "-A"]).unwrap();
        athena
            .git
            .run(["commit", "-q", "-m", "Spec for the test"])
            .unwrap();
    }

    /// The text of the one request that cites the requirement, when exactly one does.
    fn spec_request_text(athena: &Athena, id: &str) -> String {
        let known = records(athena);
        let line = format!("{SPEC_LINE}{id}");
        let found: Vec<String> = known
            .request_jobs()
            .iter()
            .filter_map(|j| j.request.as_ref())
            .filter(|r| r.text.lines().any(|l| l == line))
            .map(|r| r.text.clone())
            .collect();
        assert_eq!(found.len(), 1, "requests for {id}: {found:?}");
        found[0].clone()
    }

    #[test]
    fn the_gap_rules_flag_no_mechanism_and_a_failing_measure_and_exempt_superseded() {
        let text = [
            req("R-1", "s", "active", "none", "a test; status: repeated"),
            req("R-2", "s", "active", "a path", "a test; status: nowhere"),
            req("R-3", "s", "active", "a path", "None"),
            req("R-4", "s", "active", "a path", "a test; status: not shown"),
            req("R-5", "s", "superseded", "none", "none"),
            req(
                "R-6",
                "s",
                "active",
                "a path",
                "a figure; status: throughput",
            ),
            req("R-7", "s", "active", "", "how it is shown"),
        ]
        .concat();
        let requirements = parse_spec(&text).unwrap();
        assert_eq!(requirements.len(), 7);
        assert!(!requirements[4].active);
        let gaps = spec_gaps(&requirements);
        let ids: Vec<&str> = gaps.iter().map(|g| g.id.as_str()).collect();
        assert_eq!(ids, ["R-1", "R-2", "R-3", "R-7"]);
        assert!(gaps[0].no_mechanism && gaps[0].measure_problem.is_none());
        assert!(!gaps[1].no_mechanism);
        assert!(
            gaps[1]
                .measure_problem
                .as_ref()
                .unwrap()
                .contains("nowhere")
        );
        assert!(
            gaps[2]
                .measure_problem
                .as_ref()
                .unwrap()
                .contains("blank or none")
        );
        assert!(gaps[3].no_mechanism && gaps[3].measure_problem.is_some());
        assert!(
            gaps[0]
                .next_line()
                .starts_with("requirement R-1 has no mechanism   # owner: Hestia")
        );
        assert!(
            gaps[1]
                .next_line()
                .starts_with("requirement R-2 has a failing measure: ")
        );
        assert!(parse_spec("this is not = = toml").is_err());
        assert_eq!(parse_spec("").unwrap(), Vec::new());
    }

    #[test]
    fn the_spec_review_queues_one_request_for_a_requirement_with_no_mechanism_and_none_while_it_is_open()
     {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let spec = [
            req(
                "R-1",
                "Athena checks the thing.",
                "active",
                "none",
                "a test; status: not shown",
            ),
            req(
                "R-2",
                "Another one.",
                "active",
                "a path",
                "a test; status: not shown",
            ),
        ]
        .concat();
        put_spec(dir.path(), &athena, &spec);

        let first = athena.spec_review(&crate::Quiet).unwrap();
        assert_eq!(first.gaps, 1);
        assert_eq!(first.queued.len(), 1, "{first:?}");
        assert_eq!(first.queued[0].0, "R-1");
        assert_eq!(records(&athena).request_jobs().len(), 1);
        // Built through the steward builder: priority, acceptance bullets and a files line.
        let text = spec_request_text(&athena, "R-1");
        assert!(text.starts_with("Priority: normal\n"), "{text}");
        assert!(text.contains("Acceptance:\n- "), "{text}");
        assert!(text.contains("\nFiles: docs/spec.toml, crates/"), "{text}");
        assert!(text.contains("Athena checks the thing."), "{text}");
        assert_eq!(crate::reading::request_text_acceptance(&text).len(), 3);
        assert!(spec_request_is_open(&records(&athena), "R-1"));
        assert!(!spec_request_is_open(&records(&athena), "R"));

        // A second run while the request is open queues nothing, and the pass line says so.
        let second = athena.spec_review(&crate::Quiet).unwrap();
        assert!(second.queued.is_empty(), "{second:?}");
        assert_eq!(second.already_open, 1);
        assert_eq!(records(&athena).request_jobs().len(), 1);
        let lines = athena.review_passes(&crate::Quiet);
        assert!(
            lines.iter().any(|l| l.starts_with(
                "hestia     spec: 1 requirement(s) without a mechanism or a valid measure · queued 0 · already open 1"
            )),
            "{lines:?}"
        );
        assert_eq!(records(&athena).request_jobs().len(), 1);

        // Once the request is closed the gap is still there, so it is asked for again.
        let job = first.queued[0].1.clone();
        athena
            .close(job.as_str(), "not wanted", &crate::Quiet)
            .unwrap();
        let third = athena.spec_review(&crate::Quiet).unwrap();
        assert_eq!(third.queued.len(), 1, "{third:?}");
        assert_eq!(records(&athena).request_jobs().len(), 2);

        // Filling the mechanism ends it: nothing is queued once the requirement has one.
        let fixed = spec.replace("\"none\"", "\"a path\"");
        put_spec(dir.path(), &athena, &(fixed + "\n"));
        let fourth = athena.spec_review(&crate::Quiet).unwrap();
        assert_eq!((fourth.gaps, fourth.queued.len()), (0, 0));
    }

    #[test]
    fn a_speed_requirement_is_high_never_urgent_and_a_statement_with_faster_is_not_enough() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let spec = [
            req(
                "R-1",
                "Athena gets measurably faster.",
                "active",
                "none",
                "minutes; status: throughput",
            ),
            req(
                "R-2",
                "Athena learns faster from its mistakes.",
                "active",
                "none",
                "causes; status: repeated",
            ),
            req(
                "R-3",
                "Checks run quickly.",
                "active",
                "none",
                "durations; status: checks",
            ),
        ]
        .concat();
        put_spec(dir.path(), &athena, &spec);
        let report = athena.spec_review(&crate::Quiet).unwrap();
        assert_eq!(report.queued.len(), 3);
        assert!(spec_request_text(&athena, "R-1").starts_with("Priority: high\n"));
        assert!(spec_request_text(&athena, "R-3").starts_with("Priority: high\n"));
        assert!(spec_request_text(&athena, "R-2").starts_with("Priority: normal\n"));
        let known = records(&athena);
        let jobs = known.request_jobs();
        assert!(jobs.iter().all(|j| j.priority() != Priority::Urgent));
        let high = jobs
            .iter()
            .filter(|j| j.priority() == Priority::High)
            .count();
        assert_eq!(high, 2);
    }

    #[test]
    fn a_failing_measure_is_flagged_and_a_valid_one_or_a_superseded_requirement_is_not() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        // No spec at all: nothing to review, no error.
        let none = athena.spec_review(&crate::Quiet).unwrap();
        assert_eq!((none.gaps, none.queued.len()), (0, 0));

        let spec = [
            req("R-1", "A.", "active", "a path", "a test; status: nowhere"),
            req("R-2", "B.", "active", "a path", "a test; status: not shown"),
            req("R-3", "C.", "superseded", "none", "none"),
            req(
                "R-4",
                "D.",
                "active",
                "a path",
                "a figure; status: keep rule",
            ),
        ]
        .concat();
        put_spec(dir.path(), &athena, &spec);
        let report = athena.spec_review(&crate::Quiet).unwrap();
        assert_eq!(report.queued.len(), 1, "{report:?}");
        assert_eq!(report.queued[0].0, "R-1");
        let text = spec_request_text(&athena, "R-1");
        assert!(
            text.contains("Give requirement R-1 a valid measure"),
            "{text}"
        );
        assert!(text.contains("nowhere"), "{text}");
        assert!(text.starts_with("Priority: normal\n"), "{text}");
    }

    #[test]
    fn one_pass_queues_at_most_five_requests_and_speed_comes_first() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let mut spec = String::new();
        for n in 1..=5 {
            spec += &req(
                &format!("G-{n}"),
                "Plain.",
                "active",
                "none",
                "x; status: repeated",
            );
        }
        spec += &req("G-6", "Quick.", "active", "none", "x; status: checks");
        spec += &req("G-7", "Quicker.", "active", "none", "x; status: throughput");
        put_spec(dir.path(), &athena, &spec);

        let first = athena.spec_review(&crate::Quiet).unwrap();
        let ids: Vec<&str> = first.queued.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, ["G-6", "G-7", "G-1", "G-2", "G-3"]);
        assert_eq!((first.gaps, first.deferred), (7, 2));
        let second = athena.spec_review(&crate::Quiet).unwrap();
        let ids: Vec<&str> = second.queued.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, ["G-4", "G-5"]);
        assert_eq!(second.already_open, 5);
        let third = athena.spec_review(&crate::Quiet).unwrap();
        assert!(third.queued.is_empty() && third.deferred == 0);
        assert_eq!(records(&athena).request_jobs().len(), 7);
    }

    #[test]
    fn athena_status_lists_requirements_without_a_mechanism() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        // A repository with no spec lists nothing and does not fail.
        assert!(athena.status().unwrap().spec_gaps.is_empty());

        let spec = [
            req("R-1", "A.", "active", "none", "a test; status: not shown"),
            req("R-2", "B.", "active", "a path", "a test; status: nowhere"),
            req("R-3", "C.", "active", "a path", "a test; status: not shown"),
        ]
        .concat();
        put_spec(dir.path(), &athena, &spec);
        let view = athena.status().unwrap();
        let ids: Vec<&str> = view.spec_gaps.iter().map(|g| g.id.as_str()).collect();
        assert_eq!(ids, ["R-1", "R-2"]);
        let none: Vec<&str> = view
            .spec_gaps
            .iter()
            .filter(|g| g.no_mechanism)
            .map(|g| g.id.as_str())
            .collect();
        assert_eq!(none, ["R-1"]);
        assert!(
            view.next
                .iter()
                .any(|l| l.starts_with("requirement R-1 has no mechanism")),
            "{:?}",
            view.next
        );
        assert!(
            view.next
                .iter()
                .any(|l| l.starts_with("requirement R-2 has a failing measure")),
            "{:?}",
            view.next
        );
        assert!(!view.next.iter().any(|l| l.contains("R-2 has no mechanism")));

        let fixed = spec
            .replace("none", "a path")
            .replace("nowhere", "repeated");
        put_spec(dir.path(), &athena, &fixed);
        let view = athena.status().unwrap();
        assert!(view.spec_gaps.is_empty(), "{:?}", view.spec_gaps);
        assert!(!view.next.iter().any(|l| l.starts_with("requirement ")));
    }

    #[test]
    fn the_real_spec_records_every_field_and_classes_the_acceleration_requirements_as_speed() {
        let requirements = parse_spec(include_str!("../../../docs/spec.toml")).unwrap();
        let active: Vec<&SpecRequirement> = requirements.iter().filter(|r| r.active).collect();
        assert!(active.len() >= 264, "{} active", active.len());
        for r in &active {
            assert!(!r.mechanism.trim().is_empty(), "{}: no mechanism", r.id);
            assert!(!blank_or_none(&r.owner), "{}: no owner", r.id);
            assert_eq!(measure_problem(&r.measure), None, "{}: {}", r.id, r.measure);
        }
        let by = |id: &str| requirements.iter().find(|r| r.id == id).unwrap();
        assert!(is_speed(by("OLY-A-22")) && is_speed(by("OLY-A-23")));
        assert!(
            !is_speed(by("OLY-7-04")),
            "its statement says faster in passing"
        );
        assert!(by("OLY-7-04").statement.contains("faster"));
        assert!(by("OLY-A-22").mechanism.contains("throughput_review"));

        // If the acceleration requirement ever had no mechanism, its request would be high.
        let mut acceleration = by("OLY-A-22").clone();
        acceleration.mechanism = "none".into();
        let gaps = spec_gaps(&[acceleration]);
        assert_eq!(gaps.len(), 1);
        let text = spec_request(&gaps[0]).unwrap();
        assert!(text.starts_with("Priority: high\n"), "{text}");
        assert!(text.contains("Spec requirement: OLY-A-22\n"), "{text}");
    }

    #[test]
    fn oly_b_set_04_no_longer_appears_in_the_spec_gap_list() {
        let requirements = parse_spec(include_str!("../../../docs/spec.toml")).unwrap();
        let r = requirements
            .iter()
            .find(|r| r.id == "OLY-B-SET-04")
            .unwrap()
            .clone();
        assert_eq!(measure_problem(&r.measure), None, "{}", r.measure);
        assert!(
            !spec_gaps(&requirements).iter().any(|g| g.id == r.id),
            "OLY-B-SET-04 is still a spec gap"
        );

        // The same entry without its mechanism is a gap.
        let mut without = r;
        without.mechanism = "none".into();
        let gaps = spec_gaps(&[without]);
        assert_eq!(gaps.len(), 1);
        assert!(gaps[0].no_mechanism);
    }

    #[test]
    fn oly_b_o1_01_no_longer_appears_in_the_spec_gap_list() {
        let requirements = parse_spec(include_str!("../../../docs/spec.toml")).unwrap();
        let r = requirements
            .iter()
            .find(|r| r.id == "OLY-B-O1-01")
            .expect("OLY-B-O1-01 is in the spec")
            .clone();
        assert_ne!(r.mechanism, "none");
        assert!(
            r.mechanism
                .contains("athena_gate::invariant::bootstrap_range"),
            "{}",
            r.mechanism
        );
        assert_eq!(r.owner, "Athena");
        assert_eq!(measure_problem(&r.measure), None, "{}", r.measure);
        assert!(
            !spec_gaps(&requirements).iter().any(|g| g.id == r.id),
            "OLY-B-O1-01 is still a spec gap"
        );

        // The same entry without its mechanism is a gap.
        let mut without = r;
        without.mechanism = "none".into();
        let gaps = spec_gaps(&[without]);
        assert_eq!(gaps.len(), 1);
        assert!(gaps[0].no_mechanism);
    }

    // -----------------------------------------------------------------------------------------------------
    // Gate review

    use crate::gate_counts::{fixtures as gate_fixtures, mark_prevention};

    /// A recorded relax or remove introspection `n` of the gate, about the job of the same number.
    fn loosening(n: u32, gate: &str, kind: PreventionKind) -> IntrospectionRecord {
        let mut i = introspection(n, 'a', None, "The gate is stricter than its rule.");
        i.job = Some(gate_fixtures::job_id(n));
        i.prevention_kind = Some(kind);
        i.prevention = Some(mark_prevention(gate, "Loosen the check."));
        i
    }

    /// `test` ended five jobs and a later try landed the same change each time; `fmt` ended two and was
    /// never overruled; `test-all` ended two and was overruled once.
    fn put_gate_chains(dir: &std::path::Path, athena: &Athena) {
        let mut records = gate_fixtures::chain("test", 1, 5, 5);
        records.extend(gate_fixtures::chain("fmt", 20, 2, 0));
        records.extend(gate_fixtures::chain("test-all", 40, 2, 1));
        put(dir, athena, records);
    }

    fn request_text(athena: &Athena, job: &JobId) -> String {
        records(athena)
            .jobs
            .get(job.as_str())
            .and_then(|j| j.request.as_ref())
            .map(|r| r.text.clone())
            .expect("the queued request")
    }

    /// The request jobs whose text holds the review line for `gate`.
    fn gate_requests(athena: &Athena, gate: &str) -> Vec<JobId> {
        let line = format!("Gate under review: {gate}.");
        records(athena)
            .request_jobs()
            .iter()
            .filter(|j| {
                j.request
                    .as_ref()
                    .is_some_and(|r| r.text.lines().any(|l| l == line))
            })
            .map(|j| j.id().clone())
            .collect()
    }

    #[test]
    fn gate_review_queues_one_request_with_counts() {
        for (kind, word) in [
            (PreventionKind::Relax, "relax"),
            (PreventionKind::Remove, "remove"),
        ] {
            let dir = tempfile::tempdir().unwrap();
            let state = tempfile::tempdir().unwrap();
            let athena = athena_in(dir.path(), state.path());
            put_gate_chains(dir.path(), &athena);
            let i = loosening(60, "test", kind);
            put_introspection(dir.path(), &athena, &i);

            let job = athena
                .error_review(&i, &crate::Quiet)
                .expect("the gate was overruled, so a review is queued");
            let text = request_text(&athena, &job);
            assert!(text.starts_with("Priority: normal\n"), "{text}");
            assert!(
                text.lines().any(|l| l == "Gate under review: test."),
                "{text}"
            );
            assert!(
                text.contains("ended 5 job(s), overruled 5 time(s)"),
                "{text}"
            );
            assert!(text.contains(i.id.as_str()), "{text}");
            assert!(text.contains(&format!("Proposal: {word}.")), "{text}");
            assert!(text.contains("Loosen the check."), "{text}");
            assert!(!text.contains("[gate:"), "the prefix is not copied: {text}");
            assert_eq!(gate_requests(&athena, "test"), [job]);
            // Only the Acceptance bullets are bullets, so `athena plan` finds acceptance and files.
            assert_eq!(crate::reading::request_text_acceptance(&text).len(), 2);
        }
    }

    #[test]
    fn gate_review_not_requeued_on_second_review() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        put_gate_chains(dir.path(), &athena);
        let i = loosening(60, "test", PreventionKind::Relax);
        put_introspection(dir.path(), &athena, &i);
        assert!(athena.error_review(&i, &crate::Quiet).is_some());
        assert!(athena.error_review(&i, &crate::Quiet).is_none());
        assert!(athena.gate_review(&i).unwrap().is_none());
        assert_eq!(gate_requests(&athena, "test").len(), 1);
    }

    #[test]
    fn gate_review_one_open_request_per_gate() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        put_gate_chains(dir.path(), &athena);
        let first = loosening(60, "test", PreventionKind::Relax);
        put_introspection(dir.path(), &athena, &first);
        let queued = athena.error_review(&first, &crate::Quiet).unwrap();

        // A second introspection of the same gate, in another job, queues nothing while the first is open.
        let second = loosening(61, "test", PreventionKind::Remove);
        put_introspection(dir.path(), &athena, &second);
        assert!(athena.error_review(&second, &crate::Quiet).is_none());
        assert_eq!(gate_requests(&athena, "test").len(), 1);

        // A gate whose name starts with this one is another gate.
        let other = loosening(62, "test-all", PreventionKind::Relax);
        put_introspection(dir.path(), &athena, &other);
        assert!(athena.error_review(&other, &crate::Quiet).is_some());
        assert_eq!(gate_requests(&athena, "test-all").len(), 1);
        assert_eq!(gate_requests(&athena, "test").len(), 1);

        // Once the first review is closed, a later introspection of the gate queues a new one.
        athena
            .close(queued.as_str(), "not needed", &crate::Quiet)
            .unwrap();
        let third = loosening(63, "test", PreventionKind::Relax);
        put_introspection(dir.path(), &athena, &third);
        assert!(athena.error_review(&third, &crate::Quiet).is_some());
        assert_eq!(gate_requests(&athena, "test").len(), 2);
    }

    #[test]
    fn gate_review_refuses_gate_with_no_disagreements() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        put_gate_chains(dir.path(), &athena);
        // `fmt` ended two jobs and no later try overruled it; `nothing` ended none.
        for (n, gate) in [(60, "fmt"), (61, "nothing")] {
            let i = loosening(n, gate, PreventionKind::Relax);
            put_introspection(dir.path(), &athena, &i);
            assert!(athena.gate_review(&i).unwrap().is_none(), "{gate}");
            assert!(athena.error_review(&i, &crate::Quiet).is_none(), "{gate}");
        }
        // A prevention with no gate prefix, and another kind, name no gate to review.
        let mut bare = loosening(62, "test", PreventionKind::Relax);
        bare.prevention = Some("Loosen the check.".into());
        put_introspection(dir.path(), &athena, &bare);
        assert!(athena.gate_review(&bare).unwrap().is_none());
        let mut other = loosening(63, "test", PreventionKind::CodeCheck);
        other.prevention = Some(mark_prevention("test", "Add a check."));
        put_introspection(dir.path(), &athena, &other);
        assert!(athena.gate_review(&other).unwrap().is_none());
        assert!(records(&athena).request_jobs().iter().all(|j| {
            !j.request
                .as_ref()
                .is_some_and(|r| r.text.contains("Gate under review"))
        }));
    }

    #[test]
    fn gate_request_text_has_no_bullet_body_lines_and_names_gate_and_counts() {
        let count = GateCount {
            gate: "test".into(),
            ends: 5,
            disagreements: 3,
        };
        let mut i = loosening(60, "test", PreventionKind::Relax);
        i.prevention = Some(mark_prevention(
            "test",
            "First line.\n- a bullet of the model's\n1. a numbered one",
        ));
        let text = gate_request(&count, PreventionKind::Relax, &i).unwrap();
        assert!(
            text.lines().any(|l| l == "Gate under review: test."),
            "{text}"
        );
        assert!(
            text.contains("ended 5 job(s), overruled 3 time(s)"),
            "{text}"
        );
        // The model's own lines are flattened into one line, so only the two Acceptance bullets are bullets.
        assert_eq!(crate::reading::request_text_acceptance(&text).len(), 2);
        assert!(text.contains("Files: crates/, docs/"), "{text}");
        assert!(!text.contains("athena.toml"), "{text}");
    }

    #[test]
    fn relax_introspection_is_in_no_repeated_cause_or_improvement() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        put_gate_chains(dir.path(), &athena);
        let c = class(1, "A check is too strict");
        put(
            dir.path(),
            &athena,
            vec![AnyRecord::FailureClass(c.clone())],
        );
        let mut first = loosening(60, "test", PreventionKind::Relax);
        first.failure_class = Some(c.id.clone());
        let mut second = loosening(61, "test", PreventionKind::Remove);
        second.failure_class = Some(c.id.clone());
        for i in [&first, &second] {
            put_introspection(dir.path(), &athena, i);
        }
        let known = records(&athena);
        assert_eq!(cause_key(&known, &first), None);
        assert_eq!(cause_key(&known, &second), None);
        // The same two as code checks do form a repeated cause.
        let mut as_check = first.clone();
        as_check.prevention_kind = Some(PreventionKind::CodeCheck);
        assert!(cause_key(&known, &as_check).is_some());
        assert!(repeated_causes(&known).is_empty());
        assert!(improvement_of(&known).items.is_empty());

        // Their review request is not an improvement item either: nothing links it to a cause.
        assert!(athena.error_review(&first, &crate::Quiet).is_some());
        let known = records(&athena);
        assert!(repeated_causes(&known).is_empty());
        assert!(improvement_of(&known).items.is_empty());
    }

    #[test]
    fn gate_review_does_not_hide_code_check_cause_from_backfill() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        put_gate_chains(dir.path(), &athena);
        let c = class(1, "The verdict is not read");
        put(
            dir.path(),
            &athena,
            vec![AnyRecord::FailureClass(c.clone())],
        );
        // The same cause as a code-check in two jobs: a repeated cause nobody has queued yet.
        let a = introspection(10, 'a', Some(&c.id), "The verdict is not read.");
        let b = introspection(11, 'b', Some(&c.id), "The verdict is not read.");
        // And a relax introspection of the same class in a third job, whose gate review is queued.
        let mut r = loosening(12, "test", PreventionKind::Relax);
        r.failure_class = Some(c.id.clone());
        r.cause = Some("The verdict is not read.".into());
        for i in [&a, &b, &r] {
            put_introspection(dir.path(), &athena, i);
        }
        assert!(athena.error_review(&r, &crate::Quiet).is_some());
        assert_eq!(gate_requests(&athena, "test").len(), 1);

        let report = athena.error_review_backfill(&crate::Quiet).unwrap();
        assert_eq!(report.count(BackfillAction::Queued), 1, "{report:?}");
        let cause = repeated_causes(&records(&athena));
        assert_eq!(cause.len(), 1);
        assert_eq!(
            cause[0].jobs.len(),
            2,
            "the relax record is not in the group"
        );
        assert!(cause[0].request.is_some());
    }

    fn landed(job: &JobId) -> AnyRecord {
        AnyRecord::Outcome(Box::new(Outcome {
            job: job.clone(),
            state: OutcomeState::Landed,
            reason: "landed".into(),
            at: at(500),
            totals: Some(crate::scoring::totals(&[], None)),
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        }))
    }

    #[test]
    fn two_introspections_of_one_cause_in_different_jobs_queue_one_high_request_and_a_third_queues_none()
     {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let c = class(1, "Reading ignores the earlier verdict");
        put(
            dir.path(),
            &athena,
            vec![AnyRecord::FailureClass(c.clone())],
        );

        let first = introspection(
            10,
            'a',
            Some(&c.id),
            "The reading does not load the verdict.",
        );
        put_introspection(dir.path(), &athena, &first);
        assert_eq!(athena.error_review(&first, &crate::Quiet), None);
        assert!(
            records(&athena).request_jobs().is_empty(),
            "one job is no repeat"
        );

        // The same cause in another job, worded differently: the class is what joins them.
        let second = introspection(20, 'b', Some(&c.id), "The earlier verdict is never read.");
        put_introspection(dir.path(), &athena, &second);
        let queued = athena
            .error_review(&second, &crate::Quiet)
            .expect("the second job makes it a repeat");

        let known = records(&athena);
        assert_eq!(known.request_jobs().len(), 1, "exactly one request");
        let request = known.jobs[queued.as_str()].request.as_ref().unwrap();
        assert!(
            request.text.starts_with("Priority: high\n"),
            "{}",
            request.text
        );
        assert_eq!(
            known.jobs[queued.as_str()].priority(),
            crate::plan::Priority::High
        );
        assert!(request.text.contains(first.id.as_str()), "{}", request.text);
        assert!(
            request.text.contains(second.id.as_str()),
            "{}",
            request.text
        );
        assert!(request.text.contains("Acceptance:\n- "), "{}", request.text);
        assert!(request.text.contains("root cause"), "{}", request.text);
        // The shared steward builder's own parse accepts the text it made.
        assert!(!crate::reading::request_text_acceptance(&request.text).is_empty());

        // A third introspection of the cause, in a third job, while that request is open.
        let third = introspection(30, 'c', Some(&c.id), "Nothing loads the earlier verdict.");
        put_introspection(dir.path(), &athena, &third);
        assert_eq!(athena.error_review(&third, &crate::Quiet), None);
        let known = records(&athena);
        assert_eq!(known.request_jobs().len(), 1, "still one request");
        let repeated = repeated_causes(&known);
        assert_eq!(repeated.len(), 1);
        assert_eq!(repeated[0].count(), 3);
        assert_eq!(repeated[0].jobs.len(), 3);
        assert_eq!(repeated[0].request.as_ref().unwrap().job, queued.as_str());

        // Once the request has landed, the cause still counts as addressed.
        put(dir.path(), &athena, vec![landed(&queued)]);
        let fourth = introspection(40, 'd', Some(&c.id), "Still no read of the verdict.");
        put_introspection(dir.path(), &athena, &fourth);
        assert_eq!(athena.error_review(&fourth, &crate::Quiet), None);
        assert_eq!(records(&athena).request_jobs().len(), 1);
    }

    /// The word for the one who runs Athena, built from fragments so no tracked file holds it.
    fn boundary_word() -> String {
        ["oper", "ator"].concat()
    }

    #[test]
    fn an_error_review_of_a_cause_with_a_boundary_word_queues_a_request_without_it() {
        let word = boundary_word();
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let c = class(1, "Reading ignores the earlier verdict");
        put(
            dir.path(),
            &athena,
            vec![AnyRecord::FailureClass(c.clone())],
        );
        let mut first = introspection(
            10,
            'a',
            Some(&c.id),
            &format!("The {word} is never told of the verdict."),
        );
        first.prevention = Some(format!("Tell the {word} at once."));
        put_introspection(dir.path(), &athena, &first);
        let second = introspection(20, 'b', Some(&c.id), "The earlier verdict is never read.");
        put_introspection(dir.path(), &athena, &second);
        // The seed carries the word, so the request would be refused at admission if it were copied as is.
        assert!(!athena_context::briefs::boundary_scan(first.cause.as_deref().unwrap()).is_empty());

        let queued = athena
            .error_review(&second, &crate::Quiet)
            .expect("the request is admitted");
        let known = records(&athena);
        let text = &known.jobs[queued.as_str()].request.as_ref().unwrap().text;
        assert!(
            athena_context::briefs::boundary_scan(text).is_empty(),
            "{text}"
        );
        assert!(crate::reading::scan_request_input(text).is_empty());
        assert!(text.contains("the person running Athena"), "{text}");
    }

    #[test]
    fn a_root_cause_request_for_a_cause_with_a_boundary_word_has_none() {
        let word = boundary_word();
        let path = ["/home", "/user/x"].concat();
        let mut records = Records::default();
        let mut one = introspection(
            10,
            'a',
            None,
            &format!("The {word} reads {path} and the {}.", word.to_uppercase()),
        );
        one.prevention = Some(format!("Ask the {word}."));
        records.introspections = vec![
            one.clone(),
            introspection(20, 'b', None, one.cause.as_deref().unwrap()),
        ];
        let causes = repeated_causes(&records);
        assert_eq!(causes.len(), 1);
        assert!(!athena_context::briefs::boundary_scan(&causes[0].cause).is_empty());

        let text = root_cause_request(&causes[0], &records).unwrap();
        assert!(
            athena_context::briefs::boundary_scan(&text).is_empty(),
            "{text}"
        );
        assert!(crate::reading::scan_request_input(&text).is_empty());
        assert!(text.contains("the person running Athena"), "{text}");
        assert!(text.contains("[local path]"), "{text}");
    }

    #[test]
    fn a_single_job_repeat_or_a_cited_cause_queues_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let c = class(1, "Planner ignores the cap");
        let d = class(3, "Check output is cut");
        put(
            dir.path(),
            &athena,
            vec![
                AnyRecord::FailureClass(c.clone()),
                AnyRecord::FailureClass(d.clone()),
            ],
        );

        // The same cause twice in one job is one job.
        let one = introspection(10, 'a', Some(&c.id), "The cap is not read.");
        let two = introspection(11, 'a', Some(&c.id), "The cap is not read.");
        put_introspection(dir.path(), &athena, &one);
        put_introspection(dir.path(), &athena, &two);
        assert_eq!(athena.error_review(&two, &crate::Quiet), None);
        assert!(repeated_causes(&records(&athena)).is_empty());
        assert!(records(&athena).request_jobs().is_empty());

        // A request that already cites an introspection of the cause covers the cause.
        let owner = introspection(20, 'x', Some(&d.id), "The output tail is cut.");
        put_introspection(dir.path(), &athena, &owner);
        let (cited, _) = athena
            .queue(&format!(
                "Make the check output whole\n\nSee introspection {}.\n\nAcceptance:\n- The output is whole\n\nFiles: crates/\n",
                owner.id
            ))
            .unwrap();
        let again = introspection(21, 'y', Some(&d.id), "The output tail is cut again.");
        put_introspection(dir.path(), &athena, &again);
        assert_eq!(athena.error_review(&again, &crate::Quiet), None);
        let known = records(&athena);
        assert_eq!(known.request_jobs().len(), 1, "no second request");
        assert_eq!(
            repeated_causes(&known)[0].request.as_ref().unwrap().job,
            cited.as_str()
        );

        // A failed introspection is no case, and neither is one with no job.
        let mut failed = introspection(30, 'p', Some(&c.id), "The cap is not read.");
        failed.status = IntrospectionStatus::Failed;
        let mut no_job = introspection(31, 'q', Some(&c.id), "The cap is not read.");
        no_job.job = None;
        put_introspection(dir.path(), &athena, &failed);
        put_introspection(dir.path(), &athena, &no_job);
        assert_eq!(athena.error_review(&failed, &crate::Quiet), None);
        assert_eq!(athena.error_review(&no_job, &crate::Quiet), None);
        assert_eq!(records(&athena).request_jobs().len(), 1);
    }

    #[test]
    fn a_cause_is_matched_by_the_registry_class_by_jevs_class_answer_or_by_its_text() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let c = class(1, "Reading ignores the earlier verdict");
        put(
            dir.path(),
            &athena,
            vec![AnyRecord::FailureClass(c.clone())],
        );

        // A recorded introspection always names a class, but it may name one that is not in the registry (an
        // id the registry does not hold). The registry then cannot place it, and Jev's answer does.
        let unlisted = rec_id(900);
        let first = introspection(10, 'a', Some(&c.id), "The verdict is not read.");
        let mut second = introspection(20, 'b', Some(&unlisted), "A verdict nobody loads.");
        second.jev = Some(IntrospectionJev {
            model: "typesafe-ai/jev".into(),
            class: Some(JevClassAnswer {
                choice: c.id.to_string(),
                probabilities: Default::default(),
                confidence: 0.9,
            }),
            repeat_probability: Some(0.5),
            context_prevention: None,
            input_tokens: 0,
            output_tokens: 0,
            cost_usd: None,
            error: None,
        });
        put_introspection(dir.path(), &athena, &first);
        put_introspection(dir.path(), &athena, &second);
        assert!(athena.error_review(&second, &crate::Quiet).is_some());

        // Jev's `new` is no registry class; with no class to place them, the same words (case and punctuation
        // aside) join.
        let mut text_a = introspection(30, 'e', Some(&unlisted), "Nothing runs the docs test!");
        let mut text_b = introspection(31, 'f', Some(&unlisted), "nothing runs the DOCS test");
        text_a.jev = None;
        text_b.jev = Some(IntrospectionJev {
            class: Some(JevClassAnswer {
                choice: "new".into(),
                probabilities: Default::default(),
                confidence: 0.5,
            }),
            ..second.jev.clone().unwrap()
        });
        put_introspection(dir.path(), &athena, &text_a);
        assert_eq!(athena.error_review(&text_a, &crate::Quiet), None);
        put_introspection(dir.path(), &athena, &text_b);
        assert!(athena.error_review(&text_b, &crate::Quiet).is_some());
        assert_eq!(records(&athena).request_jobs().len(), 2);

        // A different cause with no class stays alone.
        let other = introspection(40, 'g', Some(&unlisted), "The brief omits file names.");
        put_introspection(dir.path(), &athena, &other);
        assert_eq!(athena.error_review(&other, &crate::Quiet), None);
    }

    #[test]
    fn athena_status_lists_each_open_repeated_cause_with_its_count_and_its_request() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        assert!(athena.status().unwrap().repeated_causes.is_empty());

        let addressed = class(1, "Reading ignores the earlier verdict");
        let waiting = class(3, "Check output is cut");
        let done = class(5, "Planner ignores the cap");
        put(
            dir.path(),
            &athena,
            vec![
                AnyRecord::FailureClass(addressed.clone()),
                AnyRecord::FailureClass(waiting.clone()),
                AnyRecord::FailureClass(done.clone()),
            ],
        );
        // `addressed`: three introspections in three jobs, with a request queued by the review.
        let mut queued = None;
        for (n, of) in [(10, 'a'), (11, 'b'), (12, 'c')] {
            let i = introspection(n, of, Some(&addressed.id), "The verdict is not read.");
            put_introspection(dir.path(), &athena, &i);
            if let Some(job) = athena.error_review(&i, &crate::Quiet) {
                queued = Some(job);
            }
        }
        let queued = queued.expect("the second introspection queued the request");
        // `waiting`: two jobs, but no review ran, so no request addresses it.
        for (n, of) in [(20, 'd'), (21, 'e')] {
            let i = introspection(n, of, Some(&waiting.id), "The tail is cut.");
            put_introspection(dir.path(), &athena, &i);
        }
        // `done`: two jobs and a request that has landed, so it is no longer open.
        for (n, of) in [(30, 'f'), (31, 'g')] {
            let i = introspection(n, of, Some(&done.id), "The cap is not read.");
            put_introspection(dir.path(), &athena, &i);
            athena.error_review(&i, &crate::Quiet);
        }
        let done_job = records(&athena)
            .request_jobs()
            .into_iter()
            .find(|j| {
                j.request
                    .as_ref()
                    .is_some_and(|r| r.text.contains(rec_id(30).as_str()))
            })
            .map(|j| j.id().clone())
            .expect("the review queued a request for the cause");
        put(dir.path(), &athena, vec![landed(&done_job)]);

        let view = athena.status().unwrap();
        let shown: Vec<(usize, usize, Option<String>)> = view
            .repeated_causes
            .iter()
            .map(|c| {
                (
                    c.count(),
                    c.jobs.len(),
                    c.request.as_ref().map(|r| r.job.clone()),
                )
            })
            .collect();
        assert_eq!(
            shown,
            vec![(3, 3, Some(queued.as_str().to_string())), (2, 2, None)],
            "the landed cause is not listed"
        );
        assert_eq!(
            view.repeated_causes[0].request.as_ref().unwrap().state,
            "queued"
        );
        assert!(
            view.next
                .iter()
                .any(|n| n.contains("no root-cause request") && n.contains("The tail is cut.")),
            "{:?}",
            view.next
        );
    }

    #[test]
    fn the_root_cause_request_is_high_and_refuses_a_cause_that_reads_as_acceptance() {
        let cause = |text: &str| RepeatedCause {
            key: "cause:x".into(),
            class: None,
            cause: one_line(text),
            introspections: vec![rec_id(10).to_string(), rec_id(20).to_string()],
            jobs: vec![job('a').as_str().to_string(), job('b').as_str().to_string()],
            request: None,
        };
        // A cause with line breaks and a leading dash is flattened to one line and prefixed, so the builder
        // does not read it as an acceptance bullet.
        let text = root_cause_request(&cause("- first\n- second"), &Records::default()).unwrap();
        assert_eq!(
            crate::reading::request_text_acceptance(&text).len(),
            3,
            "{text}"
        );
        assert!(text.contains("Cause: - first - second\n"), "{text}");
        assert!(text.starts_with("Priority: high\n"), "{text}");
        assert!(!text.to_lowercase().contains("urgent"), "{text}");
    }

    #[test]
    fn the_root_cause_request_asks_for_missing_information_before_allowing_a_refusal_rule() {
        let cause = RepeatedCause {
            key: "cause:x".into(),
            class: None,
            cause: one_line("The planner lacked a fact"),
            introspections: vec![rec_id(10).to_string(), rec_id(20).to_string()],
            jobs: vec![job('a').as_str().to_string(), job('b').as_str().to_string()],
            request: None,
        };
        let records = Records {
            introspections: Vec::new(),
            ..Default::default()
        };
        let text = root_cause_request(&cause, &records).unwrap();
        let ask = text
            .find("First ask what information the failing stage lacked")
            .unwrap_or_else(|| panic!("{text}"));
        let supplied = text
            .find("given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives")
            .unwrap_or_else(|| panic!("{text}"));
        let rule = text
            .find("A refusal rule is allowed only when giving the stage that information cannot prevent the failure")
            .unwrap_or_else(|| panic!("{text}"));
        let share = text
            .find("state the expected share of past accepted readings the rule would refuse")
            .unwrap_or_else(|| panic!("{text}"));
        assert!(ask < supplied && supplied < rule && rule < share, "{text}");
        // The builder refuses a bullet-like body line, so the Ok above shows there is none; the acceptance
        // list is still the three lines of the template.
        assert_eq!(
            crate::reading::request_text_acceptance(&text).len(),
            3,
            "{text}"
        );
    }

    // -----------------------------------------------------------------------------------------------------
    // Backfill

    /// The live causes of the fixture backlog: class name, first introspection number, and how many jobs.
    const LIVE: [(&str, u32, u32); 7] = [
        ("Stalled producer burns its deadline", 100, 7),
        ("Readings larger than one producer run", 200, 6),
        ("Criteria need evidence the verifier cannot see", 300, 5),
        ("Reading omits the earlier verdict", 400, 4),
        ("Check output is cut", 500, 3),
        ("Order ignores owner priority", 600, 2),
        ("Brief omits file names", 700, 2),
    ];
    const DORMANT: &str = "Planner drops the acceptance of a request";

    /// A backlog of repeated causes, none of which the review ever saw: the seven in [`LIVE`], one about the
    /// planner (a stage athena.toml switches off), and one whose request has landed. An open steward request
    /// names the class of the second live cause in its words, cites no introspection and has no acceptance.
    /// Returns that request's job.
    fn backlog(dir: &std::path::Path, athena: &Athena) -> JobId {
        let mut all: Vec<(&str, u32, u32)> = LIVE.to_vec();
        all.push((DORMANT, 800, 3));
        all.push(("Footer is cut", 900, 2));
        let mut records = Vec::new();
        for (idx, (name, base, count)) in all.iter().enumerate() {
            let c = class(1 + 2 * idx as u32, name);
            for k in 0..*count {
                let letter = (b'a' + k as u8) as char;
                let i = introspection(base + k, letter, Some(&c.id), &format!("{name}."));
                records.push(AnyRecord::Introspection(Box::new(i)));
            }
            records.push(AnyRecord::FailureClass(c));
        }
        let existing = JobId::Id(rec_id(1500));
        records.push(AnyRecord::Request(Request {
            job: existing.clone(),
            text: "Split large readings\n\nReadings larger than one producer run keep failing on the deadline."
                .into(),
            received_at: at(1500),
            channel: "steward".into(),
        }));
        put(dir, athena, records);
        let (footer, _) = athena
            .queue(&format!(
                "Fix the footer\n\nSee introspection {}.\n\nAcceptance:\n- The footer is whole\n\nFiles: crates/\n",
                rec_id(900)
            ))
            .unwrap();
        put(dir, athena, vec![landed(&footer)]);
        existing
    }

    fn line_of<'a>(report: &'a BackfillReport, name: &str) -> &'a BackfillLine {
        report
            .lines
            .iter()
            .find(|l| l.cause.starts_with(name))
            .unwrap_or_else(|| panic!("no line for {name}"))
    }

    #[test]
    fn the_backfill_skips_dormant_requeues_an_existing_request_queues_the_rest_and_ranks_the_top_five_high_never_urgent()
     {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let existing = backlog(dir.path(), &athena);
        assert_eq!(records(&athena).request_jobs().len(), 2);

        let report = athena.error_review_backfill(&crate::Quiet).unwrap();

        assert_eq!(report.lines.len(), 9, "{}", report.render());
        assert_eq!(report.count(BackfillAction::Queued), 6);
        assert_eq!(report.count(BackfillAction::Requeued), 1);
        assert_eq!(report.count(BackfillAction::Reprioritised), 0);
        assert_eq!(report.count(BackfillAction::Skipped), 2);

        // Ranked by occurrences, then recency: the two causes of two jobs tie, and the newer one comes first.
        let ranked: Vec<&str> = report.lines[..7]
            .iter()
            .map(|l| l.cause.trim_end_matches('.'))
            .collect();
        let expected: Vec<&str> = [0, 1, 2, 3, 4, 6, 5].iter().map(|&i| LIVE[i].0).collect();
        assert_eq!(ranked, expected);
        for (rank, l) in report.lines[..7].iter().enumerate() {
            let want = if rank < 5 { "high" } else { "normal" };
            assert_eq!(l.priority.as_deref(), Some(want), "{}", l.cause);
        }

        // Dormant: skipped with the stage named, and nothing queued for it.
        let dormant = line_of(&report, DORMANT);
        assert_eq!(dormant.action, BackfillAction::Skipped);
        assert!(
            dormant
                .reason
                .starts_with("dormant: the planner stage is disabled in athena.toml"),
            "{}",
            dormant.reason
        );
        assert_eq!(dormant.priority, None);
        let landed_line = line_of(&report, "Footer is cut");
        assert_eq!(landed_line.action, BackfillAction::Skipped);
        assert!(
            landed_line.reason.contains("has landed"),
            "{}",
            landed_line.reason
        );

        let known = records(&athena);
        assert_eq!(
            known.request_jobs().len(),
            2 + 7,
            "one request per live cause"
        );
        assert!(
            !known.request_jobs().iter().any(|j| j
                .request
                .as_ref()
                .is_some_and(|r| r.text.contains(rec_id(800).as_str()))),
            "no request cites the dormant cause"
        );

        // Re-queued: the old request is closed with a reason, and the new one went through the builder.
        let requeued = line_of(&report, LIVE[1].0);
        assert_eq!(requeued.action, BackfillAction::Requeued);
        assert_eq!(requeued.replaced.as_deref(), Some(existing.as_str()));
        let new = requeued.job.as_deref().unwrap();
        let old = &known.jobs[existing.as_str()];
        assert_eq!(old.state(), JobState::Closed(OutcomeState::Dropped));
        let reason = &old.outcome.as_ref().unwrap().reason;
        assert!(reason.contains(&format!("re-queued as {new}")), "{reason}");
        let text = &known.jobs[new].request.as_ref().unwrap().text;
        assert!(
            text.starts_with("Priority: high\n\nSplit large readings\n"),
            "{text}"
        );
        assert!(text.contains("keep failing on the deadline."), "{text}");
        assert_eq!(
            crate::reading::request_text_acceptance(text).len(),
            3,
            "{text}"
        );
        assert!(text.ends_with("\nFiles: crates/"), "{text}");
        for n in 200..206 {
            assert!(text.contains(rec_id(n).as_str()), "cites {n}: {text}");
        }
        assert_eq!(known.jobs[new].state(), JobState::Queued);

        // Queued: every other live cause has a new request citing its introspection ids, at its rank's priority.
        for (rank, l) in report.lines[..7].iter().enumerate() {
            let cause = repeated_causes(&known)
                .into_iter()
                .find(|c| c.cause == l.cause)
                .unwrap();
            let address = cause.request.as_ref().expect("a request addresses it now");
            assert_eq!(Some(address.job.as_str()), l.job.as_deref());
            assert_eq!(address.state, "queued");
            let job = &known.jobs[address.job.as_str()];
            let want = if rank < 5 {
                Priority::High
            } else {
                Priority::Normal
            };
            assert_eq!(job.priority(), want, "{}", l.cause);
            let text = &job.request.as_ref().unwrap().text;
            for id in &cause.introspections {
                assert!(text.contains(id.as_str()), "{} cites {id}: {text}", l.cause);
            }
            if l.action == BackfillAction::Queued {
                assert!(
                    l.reason.contains("root-cause request queued"),
                    "{}",
                    l.reason
                );
                assert!(
                    text.contains("Fix the root cause of a repeated failure"),
                    "{text}"
                );
            }
        }
        let rendered = report.render();
        assert!(
            rendered.contains("queued 6 · re-queued 1 · priority set 0 · skipped 2"),
            "{rendered}"
        );
        assert!(
            rendered.contains("why: dormant: the planner stage"),
            "{rendered}"
        );
    }

    #[test]
    fn plan_backfill_over_more_than_five_causes_queues_every_request_at_high_or_lower() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        backlog(dir.path(), &athena);
        let plan = plan_backfill(&records(&athena), &[Stage::Planner]);
        let worked: Vec<&Planned> = plan.iter().filter(|p| p.priority.is_some()).collect();
        assert!(worked.len() > BACKFILL_HIGH, "{} causes", worked.len());
        for p in &worked {
            assert!(p.priority.unwrap() >= Priority::High, "{}", p.cause.cause);
            let text = match &p.step {
                Step::Queue { text } | Step::Requeue { text, .. } => text,
                _ => continue,
            };
            assert!(
                text.starts_with("Priority: high\n") || text.starts_with("Priority: normal\n"),
                "{text}"
            );
        }
        assert_eq!(
            worked
                .iter()
                .filter(|p| p.priority == Some(Priority::High))
                .count(),
            BACKFILL_HIGH
        );
    }

    /// Two introspections of one cause in two jobs, the review's request for it, and that request closed with
    /// `athena close` (so the outcome is a person's). Returns the class, the cause and the closed request.
    fn cause_whose_request_a_person_closed(
        dir: &std::path::Path,
        athena: &Athena,
    ) -> (FailureClass, JobId) {
        let c = class(1, "Reading ignores the earlier verdict");
        put(dir, athena, vec![AnyRecord::FailureClass(c.clone())]);
        let first = introspection(10, 'a', Some(&c.id), "The verdict is not read.");
        let second = introspection(20, 'b', Some(&c.id), "The earlier verdict is never read.");
        put_introspection(dir, athena, &first);
        put_introspection(dir, athena, &second);
        let queued = athena
            .error_review(&second, &crate::Quiet)
            .expect("the second job makes it a repeat");
        athena
            .close(queued.as_str(), "not wanted", &crate::Quiet)
            .unwrap();
        let known = records(athena);
        assert_eq!(
            known.jobs[queued.as_str()].state(),
            JobState::Closed(OutcomeState::Dropped)
        );
        (c, queued)
    }

    #[test]
    fn a_cause_a_person_closed_is_not_queued_again_until_it_recurs_after_the_close() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let (c, closed) = cause_whose_request_a_person_closed(dir.path(), &athena);

        // The backfill neither queues nor re-queues it.
        let report = athena.error_review_backfill(&crate::Quiet).unwrap();
        assert_eq!(
            report.count(BackfillAction::Queued),
            0,
            "{}",
            report.render()
        );
        assert_eq!(report.count(BackfillAction::Requeued), 0);
        assert!(report.changed_nothing(), "{}", report.render());
        assert_eq!(records(&athena).request_jobs().len(), 1);

        // An introspection recorded before the close does not count as a recurrence.
        let older = introspection(30, 'c', Some(&c.id), "Nothing reads the verdict.");
        put_introspection(dir.path(), &athena, &older);
        assert_eq!(athena.error_review(&older, &crate::Quiet), None);
        assert_eq!(records(&athena).request_jobs().len(), 1);

        // A new introspection of the cause, recorded after the close, queues it again, at high.
        let mut later = introspection(40, 'd', Some(&c.id), "The verdict is still not read.");
        later.at = "2999-01-01T00:00:00Z".parse().unwrap();
        put_introspection(dir.path(), &athena, &later);
        let queued = athena
            .error_review(&later, &crate::Quiet)
            .expect("the cause recurred after the close");
        assert_ne!(queued, closed);
        let known = records(&athena);
        assert_eq!(known.request_jobs().len(), 2);
        assert_eq!(known.jobs[queued.as_str()].priority(), Priority::High);
    }

    #[test]
    fn the_backfill_queues_a_cause_again_once_it_recurs_after_a_persons_close() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let (c, _) = cause_whose_request_a_person_closed(dir.path(), &athena);
        let mut later = introspection(40, 'd', Some(&c.id), "The verdict is still not read.");
        later.at = "2999-01-01T00:00:00Z".parse().unwrap();
        put_introspection(dir.path(), &athena, &later);

        let report = athena.error_review_backfill(&crate::Quiet).unwrap();
        assert_eq!(
            report.count(BackfillAction::Queued),
            1,
            "{}",
            report.render()
        );
        let known = records(&athena);
        assert_eq!(known.request_jobs().len(), 2);
        assert!(
            known
                .request_jobs()
                .iter()
                .all(|j| j.priority() != Priority::Urgent)
        );
    }

    #[test]
    fn a_cause_whose_request_hestia_closed_to_queue_it_again_is_still_queued() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let c = class(1, "Check output is cut");
        let (a, b) = (
            introspection(10, 'a', Some(&c.id), "The tail is cut."),
            introspection(11, 'b', Some(&c.id), "The tail is cut."),
        );
        put(
            dir.path(),
            &athena,
            vec![
                AnyRecord::FailureClass(c),
                AnyRecord::Introspection(Box::new(a.clone())),
                AnyRecord::Introspection(Box::new(b.clone())),
            ],
        );
        let text = StewardRequest::new(
            Some(Priority::Normal),
            "Keep the whole check output",
            &format!("See introspections {} and {}.", a.id, b.id),
            &["The output is whole".to_string()],
            &["crates/".to_string()],
        )
        .unwrap()
        .render();
        let (old, _) = athena.queue(&text).unwrap();
        // The reason Hestia writes when it closes a request to queue another in its place.
        athena
            .close(
                old.as_str(),
                "re-queued as 20261027-000000-aaaa through the steward request builder",
                &crate::Quiet,
            )
            .unwrap();
        assert!(!closed_by_person(&records(&athena).jobs[old.as_str()]));

        let report = athena.error_review_backfill(&crate::Quiet).unwrap();
        assert_eq!(
            report.count(BackfillAction::Queued),
            1,
            "{}",
            report.render()
        );
        assert_eq!(records(&athena).request_jobs().len(), 2);
    }

    #[test]
    fn the_backfill_does_not_lower_a_request_a_person_made_urgent() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let c = class(1, "Check output is cut");
        let (a, b) = (
            introspection(10, 'a', Some(&c.id), "The tail is cut."),
            introspection(11, 'b', Some(&c.id), "The tail is cut."),
        );
        put(
            dir.path(),
            &athena,
            vec![
                AnyRecord::FailureClass(c),
                AnyRecord::Introspection(Box::new(a.clone())),
                AnyRecord::Introspection(Box::new(b.clone())),
            ],
        );
        let text = StewardRequest::new(
            Some(Priority::Normal),
            "Keep the whole check output",
            &format!("See introspections {} and {}.", a.id, b.id),
            &["The output is whole".to_string()],
            &["crates/".to_string()],
        )
        .unwrap()
        .render();
        let (job, _) = athena.queue(&text).unwrap();
        athena.set_priority(job.as_str(), "urgent").unwrap();

        let report = athena.error_review_backfill(&crate::Quiet).unwrap();
        assert!(report.changed_nothing(), "{}", report.render());
        assert_eq!(
            records(&athena).jobs[job.as_str()].priority(),
            Priority::Urgent
        );
    }

    #[test]
    fn a_second_backfill_over_the_same_records_queues_requeues_and_closes_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        backlog(dir.path(), &athena);
        let first = athena.error_review_backfill(&crate::Quiet).unwrap();
        assert_eq!(first.count(BackfillAction::Queued), 6);
        assert!(!first.changed_nothing());

        let head = athena.head().unwrap();
        let jobs = records(&athena).request_jobs().len();
        let second = athena.error_review_backfill(&crate::Quiet).unwrap();

        assert_eq!(
            second.lines.len(),
            9,
            "the same nine causes are looked at again"
        );
        assert!(second.changed_nothing(), "{}", second.render());
        assert_eq!(second.count(BackfillAction::Queued), 0);
        assert_eq!(second.count(BackfillAction::Requeued), 0);
        assert_eq!(athena.head().unwrap(), head, "no commit was made");
        assert_eq!(records(&athena).request_jobs().len(), jobs);
        let covered = second
            .lines
            .iter()
            .filter(|l| l.reason.contains("already covers the cause"))
            .count();
        assert_eq!(covered, 7, "{}", second.render());
    }

    /// A UI that keeps the lines it is given.
    #[derive(Default)]
    struct Lines(std::sync::Mutex<Vec<String>>);

    impl Ui for Lines {
        fn line(&self, text: &str) {
            self.0.lock().unwrap().push(text.to_string());
        }
    }

    /// A builder that runs no cargo: it counts its builds and writes a small file into the target directory.
    #[derive(Default)]
    struct CountingBuilder(std::cell::Cell<u32>);

    impl CargoBuilder for CountingBuilder {
        fn sccache_installed(&self) -> bool {
            false
        }

        fn build(
            &self,
            _worktree: &std::path::Path,
            target_dir: &std::path::Path,
        ) -> Result<crate::disk::BuildRun> {
            self.0.set(self.0.get() + 1);
            std::fs::create_dir_all(target_dir)?;
            std::fs::write(target_dir.join("out"), [1u8; 64])?;
            Ok(crate::disk::BuildRun {
                seconds: 2.5,
                sccache: None,
                peak: None,
            })
        }
    }

    /// Commits `Cargo.lock` with `text` on top of the repository.
    fn commit_lock(dir: &std::path::Path, athena: &Athena, text: &str) {
        std::fs::write(dir.join("Cargo.lock"), text).unwrap();
        athena.git.run(["add", "-A"]).unwrap();
        athena
            .git
            .run(["commit", "-q", "-m", "Change the lock file"])
            .unwrap();
    }

    #[test]
    fn measure_build_is_due_with_no_record_or_a_different_lock_and_not_otherwise() {
        let record = |lock: &str| BuildMeasurement {
            id: RecordId::parse("20261029-090000-5abc").unwrap(),
            at: crate::now(),
            commit: "0".repeat(40),
            cargo_lock_hash: lock.into(),
            target_bytes: 1,
            source_bytes: 1,
            first_build_seconds: 1.0,
            second_build_seconds: 1.0,
            sccache: athena_data::records::SccacheFigures::NotInstalled,
            first_build_peak: None,
            second_build_peak: None,
        };
        assert!(measure_build_due(None, "aaa"));
        assert!(!measure_build_due(Some(&record("aaa")), "aaa"));
        assert!(measure_build_due(Some(&record("aaa")), "bbb"));
    }

    #[test]
    fn sweep_measures_build_when_no_record_exists_and_again_only_when_cargo_lock_changes() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        commit_lock(dir.path(), &athena, "version = 4\n");
        let builder = CountingBuilder::default();
        let ui = Lines::default();

        // No record exists: the sweep builds twice and records one measurement.
        athena.hestia_sweep_measuring(&[], &ui, &builder).unwrap();
        assert_eq!(builder.0.get(), 2);
        let known = records(&athena);
        assert_eq!(known.build_measurements.len(), 1);
        let first = known.build_measurements[0].clone();
        assert_eq!(first.target_bytes, 64);
        assert_eq!(
            Some(first.cargo_lock_hash.clone()),
            cargo_lock_blob(&athena.git, &athena.head().unwrap()).unwrap()
        );
        {
            let lines = ui.0.lock().unwrap();
            assert!(
                lines.iter().any(|l| l.contains("build measurement")),
                "{lines:?}"
            );
        }

        // The next sweep finds the record for this Cargo.lock and builds nothing, though HEAD moved.
        athena.hestia_sweep_measuring(&[], &ui, &builder).unwrap();
        assert_eq!(
            builder.0.get(),
            2,
            "no second measurement for the same lock"
        );
        assert_eq!(records(&athena).build_measurements.len(), 1);

        // A new Cargo.lock: the sweep measures again, and the newest record names the new lock.
        commit_lock(dir.path(), &athena, "version = 4\n# a dependency changed\n");
        athena.hestia_sweep_measuring(&[], &ui, &builder).unwrap();
        assert_eq!(builder.0.get(), 4);
        let known = records(&athena);
        assert_eq!(known.build_measurements.len(), 2);
        let newest = BuildMeasurement::latest(&known.build_measurements).unwrap();
        assert_ne!(newest.cargo_lock_hash, first.cargo_lock_hash);
        assert_eq!(
            Some(newest.cargo_lock_hash.clone()),
            cargo_lock_blob(&athena.git, &athena.head().unwrap()).unwrap()
        );

        // And once more with nothing changed: nothing runs.
        athena.hestia_sweep_measuring(&[], &ui, &builder).unwrap();
        assert_eq!(builder.0.get(), 4);
    }

    #[test]
    fn sweep_measures_nothing_without_a_cargo_lock_with_low_disk_or_when_a_build_fails() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let mut athena = athena_in(dir.path(), state.path());
        let builder = CountingBuilder::default();
        let ui = Lines::default();
        // No Cargo.lock at HEAD: nothing to build.
        athena.hestia_sweep_measuring(&[], &ui, &builder).unwrap();
        assert_eq!(builder.0.get(), 0);
        assert!(records(&athena).build_measurements.is_empty());

        // Free disk space below `[disk] min_free_gb`: the pass says so and builds nothing.
        commit_lock(dir.path(), &athena, "version = 4\n");
        let roomy = std::mem::replace(
            &mut athena.state,
            StateDir::new(state.path().join("repo")).with_free_space(|_| Ok(GB)),
        );
        athena.hestia_sweep_measuring(&[], &ui, &builder).unwrap();
        athena.state = roomy;
        assert_eq!(builder.0.get(), 0);
        assert!(
            ui.0.lock()
                .unwrap()
                .iter()
                .any(|l| l.contains("due but not run")),
        );

        // A failed measurement does not stop the sweep, writes no record and is tried again.
        struct Failing;
        impl CargoBuilder for Failing {
            fn sccache_installed(&self) -> bool {
                false
            }
            fn build(
                &self,
                _: &std::path::Path,
                _: &std::path::Path,
            ) -> Result<crate::disk::BuildRun> {
                bail!("cargo is broken")
            }
        }
        let sweep = athena.hestia_sweep_measuring(&[], &ui, &Failing).unwrap();
        assert!(sweep.checks.is_empty());
        assert!(records(&athena).build_measurements.is_empty());
        assert!(
            ui.0.lock()
                .unwrap()
                .iter()
                .any(|l| l.contains("could not run") && l.contains("cargo is broken")),
        );
        athena.hestia_sweep_measuring(&[], &ui, &builder).unwrap();
        assert_eq!(builder.0.get(), 2);
        assert_eq!(records(&athena).build_measurements.len(), 1);
    }

    #[test]
    fn a_sweep_runs_the_backfill_and_the_derivation_and_a_repeat_sweep_changes_nothing_else() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        backlog(dir.path(), &athena);
        let before = records(&athena).request_jobs().len();

        let ui = Lines::default();
        athena.hestia_sweep_with(&[clean("alpha", 1)], &ui).unwrap();
        let first = records(&athena);
        assert!(
            first.request_jobs().len() > before,
            "the sweep ran the backfill, which queued requests"
        );
        {
            let lines = ui.0.lock().unwrap();
            assert!(
                lines.iter().any(|l| l.contains("backfill: 9 repeated")),
                "{lines:?}"
            );
            assert!(
                lines.iter().any(|l| l.contains("evidence labels:")),
                "{lines:?}"
            );
        }

        let requests = first.request_jobs().len();
        let labels = first.class_labels.len();
        let sweeps = first.hestia_sweeps.len();
        let ui = Lines::default();
        athena.hestia_sweep_with(&[clean("alpha", 1)], &ui).unwrap();
        let second = records(&athena);
        assert_eq!(second.request_jobs().len(), requests, "nothing queued");
        assert_eq!(second.class_labels.len(), labels, "no label written");
        assert_eq!(second.hestia_sweeps.len(), sweeps + 1, "only the report");
        // The backfill and the derivation ran in the first sweep and are not due again at a sweep (they run at
        // every drain start), so the second prints only the spec pass.
        let lines = ui.0.lock().unwrap();
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(lines[0].contains("spec:"), "{lines:?}");
        let newest = Sweep::latest(&second.hestia_sweeps).unwrap();
        assert!(
            newest.checks.iter().all(|c| c.check == "alpha"),
            "{newest:?}"
        );
    }

    #[test]
    fn status_shows_each_repeated_causes_request_after_the_backfill_and_dormant_causes_apart() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        backlog(dir.path(), &athena);

        let before = athena.status().unwrap();
        assert_eq!(before.repeated_causes.len(), 7);
        assert!(before.repeated_causes.iter().all(|c| c.request.is_none()));
        assert_eq!(before.dormant_causes.len(), 1);
        assert_eq!(before.dormant_causes[0].stage, Stage::Planner);
        assert!(
            before
                .next
                .iter()
                .any(|n| n.contains("hestia review --backfill"))
        );

        athena.error_review_backfill(&crate::Quiet).unwrap();

        let after = athena.status().unwrap();
        assert_eq!(
            after.repeated_causes.len(),
            7,
            "the landed cause is not listed"
        );
        for c in &after.repeated_causes {
            let request = c.request.as_ref().unwrap_or_else(|| panic!("{}", c.cause));
            assert_eq!(request.state, "queued", "{}", c.cause);
        }
        assert!(
            after.repeated_causes.iter().all(|c| Stage::of(c).is_none()),
            "a dormant cause is not among them"
        );
        assert_eq!(after.dormant_causes.len(), 1);
        assert!(after.dormant_causes[0].cause.cause.starts_with(DORMANT));
        assert_eq!(after.dormant_causes[0].cause.request, None);
        assert!(
            !after
                .next
                .iter()
                .any(|n| n.contains("no root-cause request")),
            "{:?}",
            after.next
        );
    }

    #[test]
    fn a_queued_request_that_covers_a_cause_only_has_its_priority_set() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let c = class(1, "Check output is cut");
        let (a, b) = (
            introspection(10, 'a', Some(&c.id), "The tail is cut."),
            introspection(11, 'b', Some(&c.id), "The tail is cut."),
        );
        put(
            dir.path(),
            &athena,
            vec![
                AnyRecord::FailureClass(c),
                AnyRecord::Introspection(Box::new(a.clone())),
                AnyRecord::Introspection(Box::new(b.clone())),
            ],
        );
        // A well-formed request at normal priority that already cites both introspections.
        let text = StewardRequest::new(
            Some(Priority::Normal),
            "Keep the whole check output",
            &format!("See introspections {} and {}.", a.id, b.id),
            &["The output is whole".to_string()],
            &["crates/".to_string()],
        )
        .unwrap()
        .render();
        let (job, _) = athena.queue(&text).unwrap();

        let report = athena.error_review_backfill(&crate::Quiet).unwrap();

        assert_eq!(
            report.count(BackfillAction::Reprioritised),
            1,
            "{}",
            report.render()
        );
        assert_eq!(report.count(BackfillAction::Queued), 0);
        assert_eq!(report.count(BackfillAction::Requeued), 0);
        let known = records(&athena);
        assert_eq!(known.request_jobs().len(), 1, "no second request");
        assert_eq!(known.jobs[job.as_str()].state(), JobState::Queued);
        assert_eq!(known.jobs[job.as_str()].priority(), Priority::High);
        assert!(
            athena
                .error_review_backfill(&crate::Quiet)
                .unwrap()
                .changed_nothing()
        );
    }

    #[test]
    fn a_stage_is_dormant_only_while_athena_toml_switches_it_off() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let c = class(1, DORMANT);
        put(
            dir.path(),
            &athena,
            vec![
                AnyRecord::FailureClass(c.clone()),
                AnyRecord::Introspection(Box::new(introspection(
                    10,
                    'a',
                    Some(&c.id),
                    "Drops it.",
                ))),
                AnyRecord::Introspection(Box::new(introspection(
                    11,
                    'b',
                    Some(&c.id),
                    "Drops it.",
                ))),
            ],
        );
        let first = athena.error_review_backfill(&crate::Quiet).unwrap();
        assert_eq!(
            first.count(BackfillAction::Skipped),
            1,
            "{}",
            first.render()
        );
        assert!(records(&athena).request_jobs().is_empty());

        // Job planning on: the planner stage is no longer dormant, and the cause is queued.
        let config = std::fs::read_to_string(dir.path().join("athena.toml")).unwrap();
        assert!(config.contains("[job_plan]\nenabled = false"));
        std::fs::write(
            dir.path().join("athena.toml"),
            config.replace("[job_plan]\nenabled = false", "[job_plan]\nenabled = true"),
        )
        .unwrap();
        athena.git.run(["add", "-A"]).unwrap();
        athena
            .git
            .run(["commit", "-q", "-m", "Turn job planning on"])
            .unwrap();
        let second = athena.error_review_backfill(&crate::Quiet).unwrap();
        assert_eq!(
            second.count(BackfillAction::Queued),
            1,
            "{}",
            second.render()
        );
        assert_eq!(records(&athena).request_jobs().len(), 1);
        assert!(athena.status().unwrap().dormant_causes.is_empty());
    }

    #[test]
    fn the_stage_of_a_cause_is_read_from_the_words_of_its_class_and_cause() {
        let cause = |class: Option<&str>, text: &str| RepeatedCause {
            key: "cause:x".into(),
            class: class.map(String::from),
            cause: text.into(),
            introspections: Vec::new(),
            jobs: Vec::new(),
            request: None,
        };
        assert_eq!(
            Stage::of(&cause(None, "The critic approves a plan.")),
            Some(Stage::Critic)
        );
        assert_eq!(
            Stage::of(&cause(Some("Planner drops acceptance (x)"), "It drops.")),
            Some(Stage::Planner)
        );
        assert_eq!(Stage::of(&cause(None, "The producer stalls.")), None);
        assert_eq!(
            Stage::of(&cause(None, "Plan order ignores priority.")),
            None
        );
    }

    // -----------------------------------------------------------------------------------------------------
    // Throughput review

    use crate::status::planning_cost_tests::timed_job;
    use crate::status::waste_windows;
    use athena_agent::fake::FakeTurn;
    use std::sync::{Arc, Mutex};

    /// A fake model that answers every call with a proposal predicting `saving` minutes, and keeps the
    /// THROUGHPUT.md it was given on each call.
    fn throughput_model(saving: f64) -> (Arc<FakeAgentDriver>, Arc<Mutex<Vec<String>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let log = seen.clone();
        let driver = FakeAgentDriver::new(Box::new(move |spec| {
            assert_eq!(spec.role, Role::Steward);
            let section =
                std::fs::read_to_string(spec.workdir.join(THROUGHPUT_FILE)).unwrap_or_default();
            log.lock().unwrap().push(section);
            FakeTurn::output(serde_json::json!({
                "bottleneck": "The producer waits for the checks of an earlier job.",
                "stage": "producer",
                "change": "Drain landed jobs asynchronously while the next job is already running.",
                "expected_minutes_saved": saving,
                "rationale": "THROUGHPUT.md shows the producer stage dominates every landed job."
            }))
        }));
        (Arc::new(driver), seen)
    }

    fn producer_only(minutes: f64) -> StageMinutes {
        StageMinutes {
            producer: minutes,
            ..StageMinutes::default()
        }
    }

    /// Commits `count` landed jobs numbered from `from`, each `minutes` long, in September 2026 (before any
    /// review the test's clock makes), two hours apart.
    fn seed_landed(
        dir: &std::path::Path,
        athena: &Athena,
        from: usize,
        count: usize,
        minutes: f64,
    ) {
        seed_stages(dir, athena, from, count, producer_only(minutes));
    }

    /// Like [`seed_landed`], with the minutes of each stage given.
    fn seed_stages(
        dir: &std::path::Path,
        athena: &Athena,
        from: usize,
        count: usize,
        stages: StageMinutes,
    ) {
        let mut all = Vec::new();
        for g in from..from + count {
            let (day, hour) = (1 + g / 12, (g % 12) * 2);
            all.extend(timed_job(
                &format!("202609{day:02}-{hour:02}0000-t{g:03}"),
                format!("2026-09-{day:02}T{hour:02}:00:00Z")
                    .parse()
                    .unwrap(),
                stages,
                OutcomeState::Landed,
            ));
        }
        put(dir, athena, all);
    }

    /// Commits `count` landed jobs that ran no model, so `Throughput` counts them as landed and measures none.
    fn seed_unmeasured(dir: &std::path::Path, athena: &Athena, from: usize, count: usize) {
        seed_stages(dir, athena, from, count, StageMinutes::default());
    }

    /// Commits an open request whose title is a throughput design request, so a review finds one open.
    fn open_design_request_job(dir: &std::path::Path, athena: &Athena) {
        let mut job = timed_job(
            "20260905-000000-d000",
            "2026-09-05T00:00:00Z".parse().unwrap(),
            StageMinutes::default(),
            OutcomeState::Landed,
        );
        job.truncate(1);
        let AnyRecord::Request(request) = &mut job[0] else {
            panic!("a job starts with its request");
        };
        request.text = format!(
            "{THROUGHPUT_MARKER}Producer waits\n\nAcceptance:\n- A design note exists\n\nFiles: docs/design/\n"
        );
        put(dir, athena, job);
    }

    fn open_with_marker<'a>(known: &'a Records, marker: &str) -> Vec<&'a JobRecords> {
        known
            .request_jobs()
            .into_iter()
            .filter(|j| !matches!(j.state(), JobState::Closed(_)))
            .filter(|j| {
                j.request
                    .as_ref()
                    .is_some_and(|r| r.text.lines().any(|l| l.starts_with(marker)))
            })
            .collect()
    }

    fn open_design_requests(known: &Records) -> Vec<&JobRecords> {
        open_with_marker(known, THROUGHPUT_MARKER)
    }

    fn open_slowdown_requests(known: &Records) -> Vec<&JobRecords> {
        open_with_marker(known, SLOWDOWN_MARKER)
    }

    fn put_earlier_review(dir: &std::path::Path, athena: &Athena, landings: u32, median: f64) {
        put(
            dir,
            athena,
            vec![AnyRecord::ThroughputReview(Box::new(earlier_review(
                landings, median,
            )))],
        );
    }

    fn earlier_review(landings: u32, median: f64) -> ThroughputReview {
        ThroughputReview {
            id: rid("20260902-000000-r001"),
            at: "2026-09-02T00:00:00Z".parse().unwrap(),
            landings,
            window_median_minutes: median,
            trigger: ThroughputTrigger::Cadence,
            proposal: None,
            waste: None,
            followups: vec![],
            note: String::new(),
        }
    }

    #[test]
    fn a_throughput_review_is_due_at_ten_landings_and_on_a_rise_of_more_than_twenty_percent() {
        use ThroughputTrigger::{Cadence, Worsened};
        // No review yet: ten landings make the first one due, nine do not, however slow they were.
        assert_eq!(throughput_due(9, 500.0, None, None), None);
        assert_eq!(throughput_due(10, 5.0, None, None), Some(Cadence));
        // A review at 10 landings with a median of 10 minutes.
        let prior = earlier_review(10, 10.0);
        assert_eq!(
            throughput_due(19, 10.0, None, Some(&prior)),
            None,
            "nine more"
        );
        assert_eq!(throughput_due(20, 10.0, None, Some(&prior)), Some(Cadence));
        assert_eq!(throughput_due(15, 9.0, None, Some(&prior)), None, "faster");
        // Exactly 20% worse is not more than 20% worse; a little over is, once a job has landed since.
        assert_eq!(throughput_due(15, 12.0, None, Some(&prior)), None);
        assert_eq!(throughput_due(15, 12.1, None, Some(&prior)), Some(Worsened));
        assert_eq!(
            throughput_due(10, 12.1, None, Some(&prior)),
            None,
            "no new landing"
        );
        // Ten more landings are the cadence whatever the median did.
        assert_eq!(throughput_due(20, 12.1, None, Some(&prior)), Some(Cadence));
    }

    fn figures(attempts: f64, usd: f64, share: f64) -> WasteFigures {
        WasteFigures {
            attempts_per_landing: attempts,
            wasted_usd: usd,
            first_attempt_share: share,
        }
    }

    fn earlier_review_with_waste(
        landings: u32,
        median: f64,
        waste: WasteFigures,
    ) -> ThroughputReview {
        ThroughputReview {
            waste: Some(waste),
            ..earlier_review(landings, median)
        }
    }

    #[test]
    fn a_rise_in_wasted_work_makes_a_review_due_when_the_median_has_not_moved() {
        use ThroughputTrigger::Wasted;
        let prior = earlier_review_with_waste(10, 10.0, figures(1.5, 2.0, 0.8));
        let due = |landed, now| throughput_due(landed, 10.0, Some(now), Some(&prior));
        // Spend up from 2.00 to 2.50 with the same median: a trigger. The same inputs without the rise: none.
        assert_eq!(due(15, figures(1.5, 2.5, 0.8)), Some(Wasted));
        assert_eq!(due(15, figures(1.5, 2.0, 0.8)), None);
        // Needs a landing since the review; exactly 20% more is not more than 20%.
        assert_eq!(due(10, figures(1.5, 2.5, 0.8)), None, "no new landing");
        assert_eq!(due(15, figures(1.5, 2.4, 0.8)), None, "exactly 20%");
        // Attempts per landing and a falling first-attempt share each trigger alone.
        assert_eq!(due(15, figures(1.9, 2.0, 0.8)), Some(Wasted));
        assert_eq!(due(15, figures(1.5, 2.0, 0.6)), Some(Wasted));
        assert_eq!(
            due(15, figures(1.5, 2.0, 0.7)),
            None,
            "a fall of less than 20%"
        );
        assert_eq!(
            due(15, figures(1.2, 1.0, 0.9)),
            None,
            "better on every measure"
        );
        // A review written before the figures existed gives no waste trigger.
        let old = earlier_review(10, 10.0);
        assert_eq!(
            throughput_due(15, 10.0, Some(figures(9.0, 99.0, 0.0)), Some(&old)),
            None
        );
        // From a clean window a failed run alone starts nothing; with attempts per landing up it does.
        let clean = earlier_review_with_waste(10, 10.0, figures(1.0, 0.0, 1.0));
        let from_clean = |now| throughput_due(15, 10.0, Some(now), Some(&clean));
        assert_eq!(from_clean(figures(1.0, 0.5, 1.0)), None);
        assert_eq!(from_clean(figures(1.25, 0.5, 1.0)), Some(Wasted));
        assert_eq!(
            waste_rises(&figures(1.5, 2.0, 0.8), &figures(1.9, 2.5, 0.6)),
            vec![
                "attempts per landed change",
                "spent on attempts that did not land",
                "share landed on the first attempt (fell)"
            ]
        );
    }

    #[test]
    fn the_review_asks_the_model_at_ten_landings_and_not_at_nine() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let (model, seen) = throughput_model(6.0);
        let athena = athena_with(dir.path(), state.path(), model);
        seed_landed(dir.path(), &athena, 0, 9, 20.0);
        assert_eq!(athena.throughput_review(&crate::Quiet).unwrap(), None);
        assert!(
            seen.lock().unwrap().is_empty(),
            "no model call when not due"
        );
        assert!(records(&athena).throughput_reviews.is_empty());
        assert!(open_design_requests(&records(&athena)).is_empty());

        seed_landed(dir.path(), &athena, 9, 1, 20.0);
        let review = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        assert_eq!(review.trigger, ThroughputTrigger::Cadence);
        assert_eq!((review.landings, review.window_median_minutes), (10, 20.0));
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 1, "one model call");
        // The model was given the status section: ten landed jobs of twenty minutes each.
        assert!(seen[0].contains("10 landed"), "{}", seen[0]);
        assert!(seen[0].contains("20.0 min"), "{}", seen[0]);
    }

    #[test]
    fn a_review_queues_one_design_request_with_the_bottleneck_change_and_saving_and_notes_it_while_it_is_open()
     {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let (model, seen) = throughput_model(12.0);
        let athena = athena_with(dir.path(), state.path(), model);
        seed_landed(dir.path(), &athena, 0, 10, 20.0);

        let first = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        let known = records(&athena);
        let open = open_design_requests(&known);
        assert_eq!(open.len(), 1, "exactly one design request");
        let text = &open[0].request.as_ref().unwrap().text;
        for expected in [
            "Throughput design: The producer waits for the checks of an earlier job.",
            "Bottleneck: The producer waits for the checks of an earlier job.",
            "Proposed change: Drain landed jobs asynchronously while the next job is already running.",
            "Predicted saving: 12.0 minutes per landed job",
            first.id.as_str(),
            "Acceptance:\n- ",
        ] {
            assert!(text.contains(expected), "{expected:?} not in:\n{text}");
        }
        assert!(!crate::reading::request_text_acceptance(text).is_empty());
        let proposal = first.proposal.as_ref().expect("a proposal is recorded");
        assert_eq!(proposal.request_job.as_ref(), Some(open[0].id()));
        assert_eq!(proposal.expected_minutes_saved, 12.0);
        assert_eq!(proposal.priority, "high");
        assert_eq!(known.throughput_reviews, vec![first.clone()]);
        assert_eq!(known.steward_runs.len(), 1, "the invocation is recorded");

        // Ten more landings make a second review due while the request is still open: the review runs, the
        // model is asked and is told about the open request, and the note names it. Twenty landings are too
        // few to tell a change from the noise, so the proposal is recorded and no second request is queued.
        seed_landed(dir.path(), &athena, 10, 10, 20.0);
        let second = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        let second_proposal = second.proposal.as_ref().expect("the proposal is recorded");
        assert_eq!(second_proposal.request_job, None);
        assert!(second.note.contains("still open"), "{}", second.note);
        assert!(
            second
                .note
                .contains("design request: not enough history: 20 of 60 landings, none queued"),
            "{}",
            second.note
        );
        let known = records(&athena);
        assert_eq!(open_design_requests(&known).len(), 1, "still one request");
        assert_eq!(known.throughput_reviews.len(), 2);
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 2, "the model was asked again");
        assert!(seen[1].contains(open[0].id().as_str()), "{}", seen[1]);
        assert!(
            !seen[0].contains("An earlier design request is still open"),
            "{}",
            seen[0]
        );
        assert!(
            seen[1].contains("An earlier design request is still open"),
            "{}",
            seen[1]
        );
    }

    #[test]
    fn a_model_that_fails_still_gives_a_review_record_so_the_cadence_moves_on() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let driver = FakeAgentDriver::new(Box::new(|_| {
            FakeTurn::output(serde_json::json!({ "unexpected": true }))
        }));
        let athena = athena_with(dir.path(), state.path(), Arc::new(driver));
        seed_landed(dir.path(), &athena, 0, 10, 20.0);
        let review = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        assert_eq!(review.proposal, None);
        assert!(review.note.contains("no proposal"), "{}", review.note);
        let known = records(&athena);
        assert!(open_design_requests(&known).is_empty());
        assert_eq!(known.throughput_reviews.len(), 1);
        assert_eq!(known.steward_runs.len(), 1, "the invocation cost money");
        assert_eq!(
            athena.throughput_review(&crate::Quiet).unwrap(),
            None,
            "not due again"
        );
    }

    /// An earlier review R (10 landings, 20 minutes each, predicting 6 minutes saved by the request `J`),
    /// then `J` itself landing (its text cites R), the cited change `C` landing, and eight faster jobs.
    fn history_after_a_review(dir: &std::path::Path, athena: &Athena, change_lands: bool) {
        seed_landed(dir, athena, 0, 10, 20.0);
        let r = earlier_review(10, 20.0);
        let design = JobId::parse("20260902-020000-w000").unwrap();
        let mut review = r.clone();
        review.proposal = Some(ThroughputProposal {
            bottleneck: "The producer waits.".into(),
            stage: "producer".into(),
            change: "Drain asynchronously.".into(),
            expected_minutes_saved: 6.0,
            priority: "high".into(),
            request_job: Some(design),
        });
        let mut all = vec![AnyRecord::ThroughputReview(Box::new(review))];
        let minutes = [15.0, 15.0, 16.0, 14.0, 15.0, 16.0, 14.0, 15.0, 16.0, 14.0];
        for (i, m) in minutes.iter().enumerate() {
            let hour = 2 + 2 * i;
            let mut job = timed_job(
                &format!("20260902-{hour:02}0000-w{i:03}"),
                format!("2026-09-02T{hour:02}:00:00Z").parse().unwrap(),
                producer_only(*m),
                OutcomeState::Landed,
            );
            let text = match i {
                0 => format!(
                    "{THROUGHPUT_MARKER}Producer waits\n\nThroughput review {}.",
                    r.id
                ),
                1 => format!(
                    "Implement the asynchronous drain.\n\nCites throughput review {}.",
                    r.id
                ),
                _ => "An unrelated change.".to_string(),
            };
            let AnyRecord::Request(request) = &mut job[0] else {
                panic!("a job starts with its request");
            };
            request.text = text;
            if i == 1 && !change_lands {
                job.truncate(1);
            }
            all.extend(job);
        }
        put(dir, athena, all);
    }

    #[test]
    fn after_a_cited_change_lands_the_next_review_records_predicted_against_measured_minutes() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let (model, _) = throughput_model(3.0);
        let athena = athena_with(dir.path(), state.path(), model);
        history_after_a_review(dir.path(), &athena, true);

        // Pure: the design request J is not the change; C is, and the eight jobs after it are measured.
        let known = records(&athena);
        let followups = throughput_followups(&known, &landed_jobs(&known));
        assert_eq!(followups.len(), 1);
        let f = &followups[0];
        assert_eq!(f.review, rid("20260902-000000-r001"));
        assert_eq!(f.change_job.as_str(), "20260902-040000-w001");
        assert_eq!(f.predicted_minutes_saved, 6.0);
        assert_eq!(f.baseline_median_minutes, 20.0);
        assert_eq!(f.jobs_measured, 8);
        // 14, 14, 14, 15, 15, 16, 16, 16: the median is 15 minutes, so 5 minutes were saved.
        assert_eq!(f.measured_median_minutes, 15.0);
        assert_eq!(f.measured_minutes_saved, 5.0);

        // Twenty landings stand, ten beyond the earlier review: the next review is due and records it.
        let review = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        assert_eq!(review.followups, followups);
        let known = records(&athena);
        assert_eq!(known.throughput_reviews.len(), 2);
        assert_eq!(known.throughput_reviews[1].followups, followups);

        // The prediction is closed once: a third review records none for it.
        seed_landed(dir.path(), &athena, 10, 10, 15.0);
        let third = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        assert!(third.followups.is_empty(), "{:?}", third.followups);
    }

    #[test]
    fn a_cited_change_that_has_not_landed_records_no_follow_up() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let (model, _) = throughput_model(3.0);
        let athena = athena_with(dir.path(), state.path(), model);
        history_after_a_review(dir.path(), &athena, false);
        let known = records(&athena);
        assert!(throughput_followups(&known, &landed_jobs(&known)).is_empty());
    }

    #[test]
    fn request_priority_rises_with_the_expected_minutes_saved() {
        assert_eq!(priority_for_minutes(0.5), Priority::Low);
        assert_eq!(priority_for_minutes(2.0), Priority::Normal);
        assert_eq!(priority_for_minutes(5.0), Priority::High);
        assert_eq!(priority_for_minutes(10.0), Priority::High);
        assert_eq!(priority_for_minutes(25.0), Priority::High);
        let rank = |p: Priority| match p {
            Priority::Low => 0,
            Priority::Normal => 1,
            Priority::High => 2,
            Priority::Urgent => panic!("Hestia never queues urgent"),
        };
        let series: Vec<i32> = [0.0, 1.9, 2.0, 4.9, 5.0, 9.9, 10.0, 60.0]
            .iter()
            .map(|m| rank(priority_for_minutes(*m)))
            .collect();
        assert!(series.windows(2).all(|w| w[0] <= w[1]), "{series:?}");
        assert!(series.first() < series.last());

        // The queued request carries the priority its prediction set.
        for (saving, level, expected) in
            [(12.0, "high", Priority::High), (1.0, "low", Priority::Low)]
        {
            let dir = tempfile::tempdir().unwrap();
            let state = tempfile::tempdir().unwrap();
            let (model, _) = throughput_model(saving);
            let athena = athena_with(dir.path(), state.path(), model);
            seed_landed(dir.path(), &athena, 0, 10, 20.0);
            athena.throughput_review(&crate::Quiet).unwrap().unwrap();
            let known = records(&athena);
            let open = open_design_requests(&known);
            assert_eq!(open.len(), 1);
            let text = &open[0].request.as_ref().unwrap().text;
            assert!(text.starts_with(&format!("Priority: {level}\n")), "{text}");
            assert_eq!(open[0].priority(), expected);
        }
    }

    // -----------------------------------------------------------------------------------------------------
    // Throughput review: the comparison with the previous review

    /// Stage minutes of a job that took 60 minutes: the producer 45, the checks 10, the verifier 5.
    fn slow_stages() -> StageMinutes {
        StageMinutes {
            producer: 45.0,
            checks: 10.0,
            verifier: 5.0,
            ..StageMinutes::default()
        }
    }

    fn lines_of(ui: &Lines) -> String {
        ui.0.lock().unwrap().join("\n")
    }

    #[test]
    fn a_median_rise_from_25_to_60_with_a_design_request_open_names_the_grown_stages_and_queues_a_high_request()
     {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let (model, seen) = throughput_model(6.0);
        let athena = athena_with(dir.path(), state.path(), model);
        // Ten jobs of 25 minutes, a review that saw them, an open design request, then eleven of 60 minutes.
        seed_landed(dir.path(), &athena, 0, 10, 25.0);
        put_earlier_review(dir.path(), &athena, 10, 25.0);
        open_design_request_job(dir.path(), &athena);
        seed_stages(dir.path(), &athena, 10, 11, slow_stages());

        let ui = Lines(std::sync::Mutex::new(Vec::new()));
        let review = athena.throughput_review(&ui).unwrap().unwrap();
        assert_eq!(review.window_median_minutes, 60.0);

        // The open design request does not stop the review: the model is asked and the proposal is recorded,
        // but 21 landings give no noise estimate, so no second design request is queued.
        let proposal = review.proposal.as_ref().expect("a proposal is recorded");
        assert_eq!(proposal.request_job, None);
        assert!(review.note.contains("still open"), "{}", review.note);
        assert!(
            review
                .note
                .contains("design request: not enough history: 21 of 60 landings"),
            "{}",
            review.note
        );
        assert_eq!(seen.lock().unwrap().len(), 1, "the model was asked");
        let known = records(&athena);
        assert_eq!(open_design_requests(&known).len(), 1);

        // Every stage that grew is named, the one that gained most first, with its minutes.
        let producer = review.note.find("producer 25.0 to 45.0").expect("producer");
        let checks = review.note.find("checks 0.0 to 10.0").expect("checks");
        let verifier = review.note.find("verifier 0.0 to 5.0").expect("verifier");
        assert!(producer < checks && checks < verifier, "{}", review.note);
        assert!(
            review.note.contains("median 25.0 to 60.0"),
            "{}",
            review.note
        );
        assert!(
            lines_of(&ui).contains("producer 25.0 to 45.0"),
            "{}",
            lines_of(&ui)
        );

        // One high-priority request names the stage that grew most and its measured minutes.
        let open = open_slowdown_requests(&known);
        assert_eq!(open.len(), 1, "exactly one slowdown request");
        assert_eq!(open[0].priority(), Priority::High);
        let text = &open[0].request.as_ref().unwrap().text;
        assert!(text.starts_with("Priority: high\n"), "{text}");
        for expected in [
            "Throughput slowdown: producer grew from 25.0 to 45.0 min per landed job",
            "rose from 25.0 to 60.0",
            "checks 0.0 to 10.0",
            "verifier 0.0 to 5.0",
        ] {
            assert!(text.contains(expected), "{expected:?} not in:\n{text}");
        }
        assert!(
            review.note.contains(&format!(
                "slowdown request {} queued at high priority",
                open[0].id()
            )),
            "{}",
            review.note
        );
        assert!(known.throughput_reviews.contains(&review));

        // Ten slower jobs make a second review due while the slowdown request is open: the grown stage is
        // named again, and no second request is queued.
        seed_stages(
            dir.path(),
            &athena,
            21,
            10,
            StageMinutes {
                producer: 100.0,
                ..StageMinutes::default()
            },
        );
        let second = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        assert_eq!(second.window_median_minutes, 80.0);
        assert!(
            second.note.contains("producer 45.0 to 72.5"),
            "{}",
            second.note
        );
        assert!(
            second
                .note
                .contains(&format!("slowdown request {} is still open", open[0].id())),
            "{}",
            second.note
        );
        let known = records(&athena);
        assert_eq!(open_slowdown_requests(&known).len(), 1);
        assert_eq!(open_design_requests(&known).len(), 1);
        assert_eq!(
            seen.lock().unwrap().len(),
            2,
            "the model was asked each time"
        );
    }

    #[test]
    fn unmeasured_landed_jobs_before_the_earlier_review_do_not_hide_the_growth() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let (model, seen) = throughput_model(6.0);
        let athena = athena_with(dir.path(), state.path(), model);
        // Twelve landed jobs that ran no model, ten of 25 minutes: the review saw 22 landings. Then an open
        // design request and eleven jobs of 60 minutes.
        seed_unmeasured(dir.path(), &athena, 0, 12);
        seed_landed(dir.path(), &athena, 12, 10, 25.0);
        put_earlier_review(dir.path(), &athena, 22, 25.0);
        open_design_request_job(dir.path(), &athena);
        seed_stages(dir.path(), &athena, 22, 11, slow_stages());

        // The window that review saw is rebuilt from its ten measured jobs, not from the newest ones.
        let known = records(&athena);
        let before = previous_stage_medians(&known, &earlier_review(22, 25.0)).expect("a window");
        assert_eq!(before.producer, 25.0);
        assert_eq!(before.checks, 0.0);

        let review = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        assert_eq!(review.window_median_minutes, 60.0);
        assert!(
            review.note.contains("producer 25.0 to 45.0"),
            "{}",
            review.note
        );
        let known = records(&athena);
        let open = open_slowdown_requests(&known);
        assert_eq!(open.len(), 1, "{}", review.note);
        assert_eq!(open[0].priority(), Priority::High);
        assert_eq!(seen.lock().unwrap().len(), 1, "the model was asked");
    }

    #[test]
    fn a_flat_median_queues_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let (model, seen) = throughput_model(6.0);
        let athena = athena_with(dir.path(), state.path(), model);
        seed_landed(dir.path(), &athena, 0, 10, 25.0);
        put_earlier_review(dir.path(), &athena, 10, 25.0);
        open_design_request_job(dir.path(), &athena);
        seed_landed(dir.path(), &athena, 10, 10, 25.0);
        let before = records(&athena).request_jobs().len();

        let ui = Lines(std::sync::Mutex::new(Vec::new()));
        let review = athena.throughput_review(&ui).unwrap().unwrap();
        assert_eq!(review.window_median_minutes, 25.0);
        assert!(review.note.contains("still open"), "{}", review.note);
        assert!(
            !review.note.contains("grew") && !review.note.contains("slowdown"),
            "{}",
            review.note
        );
        assert_eq!(lines_of(&ui), "", "nothing was reported");
        let known = records(&athena);
        assert_eq!(known.request_jobs().len(), before, "no request was queued");
        assert!(open_slowdown_requests(&known).is_empty());
        assert_eq!(open_design_requests(&known).len(), 1);
        assert_eq!(seen.lock().unwrap().len(), 1, "the model was asked");

        // The same holds for equal stage medians in the pure comparison.
        assert!(stage_growth(&slow_stages(), &slow_stages()).is_empty());
    }

    #[test]
    fn a_rise_of_twenty_to_twenty_five_percent_names_stages_but_queues_no_slowdown_request() {
        for (minutes, queued) in [(24.5, false), (25.1, true)] {
            let dir = tempfile::tempdir().unwrap();
            let state = tempfile::tempdir().unwrap();
            let (model, _) = throughput_model(6.0);
            let athena = athena_with(dir.path(), state.path(), model);
            // A review that saw five jobs of 20 minutes, an open design request, then nine slower jobs.
            seed_landed(dir.path(), &athena, 0, 5, 20.0);
            put_earlier_review(dir.path(), &athena, 5, 20.0);
            open_design_request_job(dir.path(), &athena);
            seed_landed(dir.path(), &athena, 5, 9, minutes);

            let review = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
            assert_eq!(review.trigger, ThroughputTrigger::Worsened);
            assert_eq!(review.window_median_minutes, minutes);
            assert!(
                review
                    .note
                    .contains(&format!("producer 20.0 to {minutes:.1}")),
                "{minutes}: {}",
                review.note
            );
            let known = records(&athena);
            let open = open_slowdown_requests(&known);
            assert_eq!(
                open.len(),
                usize::from(queued),
                "{minutes}: {}",
                review.note
            );
            if queued {
                assert_eq!(open[0].priority(), Priority::High);
            }
        }
        assert!(!is_slowdown(20.0, 25.0), "exactly 25% is not more");
        assert!(is_slowdown(20.0, 25.1));
        assert!(!is_slowdown(20.0, 12.0));
        assert!(!is_slowdown(0.0, 10.0), "no ratio to nothing");
    }

    // -----------------------------------------------------------------------------------------------------
    // Throughput review: wasted work, the open request and the noise rule

    /// Commits one failed attempt, costing `usd`, after each landing in `slots` (as numbered by
    /// [`seed_landed`]): it starts 30 minutes after that landing started and runs 5 minutes, so it closes
    /// after the landing ends and before the next landing ends.
    fn seed_failed(dir: &std::path::Path, athena: &Athena, slots: &[usize], usd: f64) {
        let mut all = Vec::new();
        for &g in slots {
            let (day, hour) = (1 + g / 12, (g % 12) * 2);
            let mut job = timed_job(
                &format!("202609{day:02}-{hour:02}3000-f{g:03}"),
                format!("2026-09-{day:02}T{hour:02}:30:00Z")
                    .parse()
                    .unwrap(),
                producer_only(5.0),
                OutcomeState::Failed,
            );
            // The cost is on the invocation, its per-model usage and the outcome's totals, which must agree.
            let mut invocations = Vec::new();
            for record in &mut job {
                if let AnyRecord::Invocation(i) = record {
                    i.cost = athena_data::Quantity::reported(usd, athena_data::Unit::Usd, "test");
                    for usage in i.models_used.values_mut() {
                        usage.cost_usd = usd;
                    }
                    invocations.push((**i).clone());
                }
            }
            for record in &mut job {
                if let AnyRecord::Outcome(o) = record {
                    o.totals = Some(crate::scoring::totals(&invocations, None));
                }
            }
            all.extend(job);
        }
        put(dir, athena, all);
    }

    /// The landings after which a failed attempt goes: `counts[w]` of them in the `w`-th oldest window of
    /// twenty landings, each after a different landing.
    fn failed_slots(counts: [usize; 4]) -> Vec<usize> {
        (0..4)
            .flat_map(|w| (0..counts[w]).map(move |j| 20 * w + j))
            .collect()
    }

    /// Commits a review of `landings` landings and `median` minutes that stored `waste`, and, when `cites` is
    /// given, whose proposal queued that request.
    fn put_review(
        dir: &std::path::Path,
        athena: &Athena,
        landings: u32,
        median: f64,
        waste: Option<WasteFigures>,
        cites: Option<&str>,
    ) {
        let mut review = earlier_review(landings, median);
        review.waste = waste;
        review.proposal = cites.map(|job| ThroughputProposal {
            bottleneck: "The producer waits.".into(),
            stage: "producer".into(),
            change: "Drain asynchronously.".into(),
            expected_minutes_saved: 6.0,
            priority: "high".into(),
            request_job: Some(JobId::parse(job).unwrap()),
        });
        put(
            dir,
            athena,
            vec![AnyRecord::ThroughputReview(Box::new(review))],
        );
    }

    const OPEN_REQUEST: &str = "20260905-000000-d000";

    #[test]
    fn a_rise_in_wasted_spend_triggers_a_review_with_no_change_in_median_time() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let (model, seen) = throughput_model(6.0);
        let athena = athena_with(dir.path(), state.path(), model);
        // 80 landings of 10 minutes in four windows of twenty; failed attempts of 1 USD, oldest window
        // first: 2, 5, 2 and 10. The review at 75 landings stored 2.00 USD wasted, 1.5 attempts per
        // landing and every change on its first attempt.
        seed_landed(dir.path(), &athena, 0, 80, 10.0);
        seed_failed(dir.path(), &athena, &failed_slots([2, 5, 2, 10]), 1.0);
        put_review(
            dir.path(),
            &athena,
            75,
            10.0,
            Some(figures(1.5, 2.0, 1.0)),
            None,
        );

        let known = records(&athena);
        let attempts: Vec<usize> = waste_windows(&known, THROUGHPUT_WINDOW)
            .iter()
            .map(|w| w.attempts)
            .collect();
        assert_eq!(attempts, vec![30, 22, 25, 22]);
        let latest = ThroughputReview::latest(&known.throughput_reviews);
        assert_eq!(
            throughput_due(80, 10.0, None, latest),
            None,
            "the median is unchanged and only five landings stand beyond the review"
        );

        let review = athena
            .throughput_review(&crate::Quiet)
            .unwrap()
            .expect("the rise in wasted spend makes a review due");
        assert_eq!(review.trigger, ThroughputTrigger::Wasted);
        assert_eq!(review.window_median_minutes, 10.0);
        assert_eq!(review.waste.map(|w| w.wasted_usd), Some(10.0));
        assert_eq!(seen.lock().unwrap().len(), 1);
        // Spend rose by 8 USD against a noise of 3 USD, so one design request is queued.
        assert!(
            review
                .note
                .contains("spent on attempts that did not land up 8.00 USD (noise 3.00 USD"),
            "{}",
            review.note
        );
        assert_eq!(open_design_requests(&records(&athena)).len(), 1);
    }

    #[test]
    fn an_open_design_request_no_longer_stops_the_review() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let (model, seen) = throughput_model(6.0);
        let athena = athena_with(dir.path(), state.path(), model);
        seed_landed(dir.path(), &athena, 0, 10, 25.0);
        put_review(dir.path(), &athena, 10, 25.0, None, Some(OPEN_REQUEST));
        open_design_request_job(dir.path(), &athena);
        seed_stages(dir.path(), &athena, 10, 11, slow_stages());
        let before = records(&athena);
        assert_eq!(open_design_requests(&before).len(), 1);

        let review = athena
            .throughput_review(&crate::Quiet)
            .unwrap()
            .expect("the review runs although a design request is open");
        // The model was asked, and was told which request is open.
        let seen = seen.lock().unwrap();
        assert_eq!(seen.len(), 1);
        assert!(
            seen[0].contains(OPEN_REQUEST) && seen[0].contains("Producer waits"),
            "{}",
            seen[0]
        );
        // The record has the proposal, and the note names the open request and says what was queued.
        assert!(review.proposal.is_some());
        for expected in [
            OPEN_REQUEST,
            "still open; this review ran anyway",
            "design request: not enough history: 21 of 60 landings, none queued",
            "noise could not be estimated (21 of 60 landings); the 25% rule applied",
            "queued at high priority",
        ] {
            assert!(
                review.note.contains(expected),
                "{expected:?} not in {}",
                review.note
            );
        }
        let known = records(&athena);
        assert_eq!(
            open_design_requests(&known).len(),
            1,
            "no second design request"
        );
        assert_eq!(
            open_slowdown_requests(&known).len(),
            1,
            "the slowdown request is queued"
        );
    }

    /// 80 landings with failed attempts of 1 USD, `counts` oldest window first, and a review at 75 landings
    /// citing the open request `d000`, with stored figures `stored`. The median is 10 minutes throughout.
    fn eighty_landings(
        counts: [usize; 4],
        stored: WasteFigures,
        previous_landings: u32,
    ) -> (
        tempfile::TempDir,
        tempfile::TempDir,
        Athena,
        Arc<Mutex<Vec<String>>>,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let (model, seen) = throughput_model(6.0);
        let athena = athena_with(dir.path(), state.path(), model);
        seed_landed(dir.path(), &athena, 0, 80, 10.0);
        seed_failed(dir.path(), &athena, &failed_slots(counts), 1.0);
        open_design_request_job(dir.path(), &athena);
        put_review(
            dir.path(),
            &athena,
            previous_landings,
            10.0,
            Some(stored),
            Some(OPEN_REQUEST),
        );
        (dir, state, athena, seen)
    }

    #[test]
    fn a_change_within_the_noise_queues_no_request() {
        // Older windows hold 2, 5 and 2 failed attempts, so the spend noise is 3 USD and the attempts noise
        // 0.15. The newest holds 4: spend up 2 USD, attempts per landing up 0.1. Both are within the noise,
        // yet the stored 2.00 USD makes the spend trigger fire.
        let (_dir, _state, athena, seen) =
            eighty_landings([2, 5, 2, 4], figures(1.1, 2.0, 1.0), 75);
        let before = records(&athena).request_jobs().len();
        let review = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        assert_eq!(review.trigger, ThroughputTrigger::Wasted);
        assert_eq!(seen.lock().unwrap().len(), 1, "the model is asked");
        let proposal = review.proposal.as_ref().expect("the proposal is recorded");
        assert_eq!(proposal.request_job, None);
        for expected in [
            "still open",
            "design request: within noise, none queued",
            "spent on attempts that did not land up 2.00 USD (noise 3.00 USD from 2 difference(s)",
        ] {
            assert!(
                review.note.contains(expected),
                "{expected:?} not in {}",
                review.note
            );
        }
        let known = records(&athena);
        assert_eq!(known.request_jobs().len(), before, "nothing was queued");
        assert_eq!(open_design_requests(&known).len(), 1);
    }

    #[test]
    fn a_change_beyond_the_noise_queues_a_request_even_while_another_is_open() {
        // The newest window holds 10 failed attempts: spend up 8 USD against a noise of 3, attempts per
        // landing up 0.4 against 0.15.
        let (_dir, _state, athena, seen) =
            eighty_landings([2, 5, 2, 10], figures(1.5, 2.0, 1.0), 75);
        let review = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        assert_eq!(review.trigger, ThroughputTrigger::Wasted);
        assert_eq!(seen.lock().unwrap().len(), 1);
        for expected in [
            "still open",
            "design request: worse by more than the noise",
            "attempts per landed change up 0.40",
            "spent on attempts that did not land up 8.00 USD",
        ] {
            assert!(
                review.note.contains(expected),
                "{expected:?} not in {}",
                review.note
            );
        }
        let queued = review
            .proposal
            .as_ref()
            .unwrap()
            .request_job
            .clone()
            .expect("queued");
        let known = records(&athena);
        assert_eq!(open_design_requests(&known).len(), 2);
        let listed = improvement_of(&known).items;
        assert!(listed.contains(&OPEN_REQUEST.to_string()), "{listed:?}");
        assert!(listed.contains(&queued.as_str().to_string()), "{listed:?}");
    }

    #[test]
    fn one_failed_run_after_a_clean_history_is_within_the_noise() {
        // Three clean windows and one failed attempt in the newest: spend up 1 USD and attempts per landing
        // up 0.05 are one job's worth. The review at 70 landings is ten landings old, so Cadence fires.
        let (_dir, _state, athena, _seen) =
            eighty_landings([0, 0, 0, 1], figures(1.0, 0.0, 1.0), 70);
        let review = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        assert_eq!(review.trigger, ThroughputTrigger::Cadence);
        assert_eq!(review.proposal.as_ref().unwrap().request_job, None);
        assert!(
            review.note.contains("design request: within noise"),
            "{}",
            review.note
        );
        assert_eq!(open_design_requests(&records(&athena)).len(), 1);
        // Two failed runs are more than one job's worth.
        let (_dir, _state, athena, _seen) =
            eighty_landings([0, 0, 0, 2], figures(1.0, 0.0, 1.0), 70);
        let review = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        assert!(
            review.note.contains("worse by more than the noise"),
            "{}",
            review.note
        );
        assert_eq!(open_design_requests(&records(&athena)).len(), 2);
    }

    /// 80 landings: the medians of the four windows, oldest first, are `medians`; the review at 60 landings
    /// saw a median of 10 minutes.
    fn windows_of_minutes(medians: [f64; 4]) -> (tempfile::TempDir, tempfile::TempDir, Athena) {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let (model, _) = throughput_model(6.0);
        let athena = athena_with(dir.path(), state.path(), model);
        for (w, minutes) in medians.iter().enumerate() {
            seed_landed(dir.path(), &athena, 20 * w, 20, *minutes);
        }
        put_earlier_review(dir.path(), &athena, 60, 10.0);
        (dir, state, athena)
    }

    #[test]
    fn a_slowdown_within_the_noise_between_windows_queues_no_request() {
        // The older windows swing between 10 and 22 minutes, a noise of 12. The newest is 14: a rise of
        // 4 over the review's 10 minutes is more than 25%, and the producer stage grew, but it is within
        // the noise.
        let (_dir, _state, athena) = windows_of_minutes([10.0, 22.0, 10.0, 14.0]);
        let review = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        for expected in [
            "producer 10.0 to 14.0",
            "slowdown request: median rose 4.0 min over the previous review, within noise (noise 12.0 min from 2 difference(s)",
            "design request: within noise",
        ] {
            assert!(
                review.note.contains(expected),
                "{expected:?} not in {}",
                review.note
            );
        }
        let known = records(&athena);
        assert!(open_slowdown_requests(&known).is_empty());
        assert!(open_design_requests(&known).is_empty());

        // Flat older windows: the same rise is far above the noise and the one-minute floor.
        let (_dir, _state, athena) = windows_of_minutes([10.0, 10.0, 10.0, 14.0]);
        let review = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        assert!(
            review.note.contains("queued at high priority"),
            "{}",
            review.note
        );
        assert!(
            !review.note.contains("slowdown request: median rose"),
            "{}",
            review.note
        );
        assert!(
            review.note.contains("worse by more than the noise"),
            "{}",
            review.note
        );
        let known = records(&athena);
        assert_eq!(open_slowdown_requests(&known).len(), 1);
        assert_eq!(open_design_requests(&known).len(), 1);
    }

    fn measure(
        name: &'static str,
        unit: &'static str,
        now: f64,
        before: f64,
        higher_is_worse: bool,
        noise: Option<f64>,
        floor: f64,
    ) -> Measure {
        Measure {
            name,
            unit,
            now,
            before: Some(before),
            higher_is_worse,
            bar: Bar {
                noise,
                differences: if noise.is_some() { 2 } else { 0 },
                floor,
            },
        }
    }

    #[test]
    fn the_gate_queues_only_a_change_larger_than_the_noise_and_the_one_job_floor() {
        let attempts = |now: f64, noise: Option<f64>| {
            measure(
                "attempts per landed change",
                "",
                now,
                1.0,
                true,
                noise,
                0.05,
            )
        };
        let gate = |m: &[Measure]| request_gate(m, 60, false, false);

        let within = gate(&[attempts(1.1, Some(0.3))]);
        assert!(!within.queue);
        assert!(
            within.note.starts_with("design request: within noise"),
            "{}",
            within.note
        );
        let beyond = gate(&[attempts(1.5, Some(0.3))]);
        assert!(beyond.queue);
        for expected in [
            "attempts per landed change up 0.50",
            "noise 0.30",
            "2 difference(s)",
            "floor 0.05",
        ] {
            assert!(
                beyond.note.contains(expected),
                "{expected:?} in {}",
                beyond.note
            );
        }
        assert!(
            !gate(&[attempts(1.3, Some(0.3))]).queue,
            "equal to the noise is within it"
        );

        // A falling first-attempt share is worse; a rising one, or no change, is not.
        let share = |now: f64| {
            measure(
                "share landed on the first attempt",
                "",
                now,
                0.8,
                false,
                Some(0.1),
                0.05,
            )
        };
        let fell = gate(&[share(0.5)]);
        assert!(
            fell.queue && fell.note.contains("down 30 points"),
            "{}",
            fell.note
        );
        let better = gate(&[share(1.0), share(0.8)]);
        assert!(
            !better.queue && better.note.contains("no measure worsened"),
            "{}",
            better.note
        );

        // Flat older windows give noise 0: one job's worth is still within, two jobs' worth is not.
        assert!(!gate(&[attempts(1.05, Some(0.0))]).queue);
        assert!(gate(&[attempts(1.10, Some(0.0))]).queue);
        let spend = |now: f64| {
            measure(
                "spent on attempts that did not land",
                "USD",
                now,
                0.0,
                true,
                Some(0.0),
                1.0,
            )
        };
        assert!(!gate(&[spend(1.0)]).queue);
        assert!(gate(&[spend(2.0)]).queue);
        let median = |now: f64| {
            measure(
                "median minutes per landed job",
                "min",
                now,
                10.0,
                true,
                Some(0.0),
                1.0,
            )
        };
        assert!(!gate(&[median(10.5)]).queue);
        assert!(gate(&[median(12.0)]).queue);

        // Too few windows: nothing is queued, except by the very first review with no request open.
        let young = [attempts(5.0, None)];
        let none = request_gate(&young, 40, false, false);
        assert_eq!(
            none,
            Gate {
                queue: false,
                note: "design request: not enough history: 40 of 60 landings, none queued".into()
            }
        );
        assert!(request_gate(&young, 10, true, false).queue);
        assert!(!request_gate(&young, 10, true, true).queue);
        // A measure whose windows are not full is named as not judged beside one that is.
        let mixed = gate(&[attempts(1.5, Some(0.3)), median(30.0).with_no_noise()]);
        assert!(mixed.queue);
        assert!(
            mixed
                .note
                .contains("not judged, too few full windows: median minutes per landed job"),
            "{}",
            mixed.note
        );
    }

    impl Measure {
        fn with_no_noise(mut self) -> Measure {
            self.bar.noise = None;
            self.bar.differences = 0;
            self
        }
    }

    #[test]
    fn a_slowdown_is_beyond_the_noise_only_when_the_rise_exceeds_noise_and_the_floor() {
        let bar = |noise| Bar {
            noise,
            differences: 2,
            floor: 1.0,
        };
        assert_eq!(slowdown_beyond_noise(5.0, &bar(Some(2.0)), 80), Ok(None));
        let within = slowdown_beyond_noise(2.0, &bar(Some(2.0)), 80).unwrap_err();
        assert!(
            within.contains("within noise (noise 2.0 min from 2 difference(s), floor 1.0 min)"),
            "{within}"
        );
        // The floor holds when the older windows are flat.
        assert!(slowdown_beyond_noise(0.5, &bar(Some(0.0)), 80).is_err());
        assert_eq!(slowdown_beyond_noise(1.5, &bar(Some(0.0)), 80), Ok(None));
        // Without a noise estimate the 25% rule decides, and the remark says so.
        let remark = slowdown_beyond_noise(0.5, &bar(None), 21).unwrap().unwrap();
        assert!(
            remark.contains("noise could not be estimated (21 of 60 landings)"),
            "{remark}"
        );
    }

    #[test]
    fn stage_growth_lists_the_stages_that_grew_largest_first() {
        let before = StageMinutes {
            reading: 2.0,
            planning: 0.0,
            producer: 10.0,
            checks: 5.0,
            verifier: 4.0,
            landing_and_rebuild: 3.0,
        };
        let now = StageMinutes {
            reading: 4.0,
            planning: 3.0,
            producer: 20.0,
            checks: 5.0,
            verifier: 2.0,
            landing_and_rebuild: 5.0,
        };
        let grown = stage_growth(&before, &now);
        // Reading and landing both gained two minutes: pipeline order breaks the tie. Checks stayed and the
        // verifier shrank, so neither is named.
        assert_eq!(
            grown.iter().map(|g| g.stage).collect::<Vec<_>>(),
            vec!["producer", "planning", "reading", "landing and rebuild"]
        );
        assert_eq!(
            grown[1],
            StageGrowth {
                stage: "planning",
                previous_minutes: 0.0,
                minutes: 3.0
            }
        );
        assert_eq!(grown[0].added(), 10.0);
        // Shrinking stages alone give nothing, even if some other median rose.
        assert!(stage_growth(&now, &before).len() < grown.len());
        assert!(stage_growth(&now, &now).is_empty());
    }

    #[test]
    fn the_slowdown_request_names_the_biggest_stage_and_its_minutes_at_high_priority() {
        let growth = [
            StageGrowth {
                stage: "producer",
                previous_minutes: 25.0,
                minutes: 45.0,
            },
            StageGrowth {
                stage: "checks",
                previous_minutes: 0.0,
                minutes: 10.0,
            },
            StageGrowth {
                stage: "verifier",
                previous_minutes: 0.0,
                minutes: 5.0,
            },
        ];
        let text = slowdown_request(25.0, 60.0, &growth).unwrap();
        assert!(text.starts_with("Priority: high\n\n"), "{text}");
        assert!(
            text.contains(
                "Throughput slowdown: producer grew from 25.0 to 45.0 min per landed job"
            ),
            "{text}"
        );
        for expected in [
            "rose from 25.0 to 60.0",
            "a median of 45.0 minutes per landed job now, 25.0 before",
            "producer 25.0 to 45.0, checks 0.0 to 10.0, verifier 0.0 to 5.0",
        ] {
            assert!(text.contains(expected), "{expected:?} not in:\n{text}");
        }
        assert!(text.lines().any(|l| l.starts_with(SLOWDOWN_MARKER)));
        assert!(!text.lines().any(|l| l.starts_with(THROUGHPUT_MARKER)));
        assert!(!crate::reading::request_text_acceptance(&text).is_empty());
        // No review id: a landed job that cites one is read as the change that implements its proposal.
        assert!(!text.contains("20260"), "{text}");
        assert!(slowdown_request(25.0, 60.0, &[]).is_err());
    }

    #[test]
    fn review_subject_names_the_design_and_slowdown_requests() {
        let design = JobId::parse("20261104-100000-d000").unwrap();
        let slow = JobId::parse("20261104-100100-s000").unwrap();
        assert_eq!(
            review_subject(60.0, Some(&design), None),
            "Throughput review: median 60.0 min per landed job; design request 20261104-100000-d000"
        );
        assert_eq!(
            review_subject(60.0, None, None),
            "Throughput review: median 60.0 min per landed job; no request queued"
        );
        let only_slowdown = review_subject(60.0, None, Some(&slow));
        assert!(only_slowdown.contains("slowdown request 20261104-100100-s000"));
        assert!(!only_slowdown.contains("no request queued"));
        let both = review_subject(60.0, Some(&design), Some(&slow));
        assert!(both.contains("design request 20261104-100000-d000"));
        assert!(both.contains("slowdown request 20261104-100100-s000"));
    }

    #[test]
    fn throughput_outcome_shows_the_slowdown_with_and_without_a_design_request() {
        let mut review = earlier_review(21, 60.0);
        review.proposal = Some(ThroughputProposal {
            bottleneck: "The producer waits.".into(),
            stage: "producer".into(),
            change: "Drain asynchronously.".into(),
            expected_minutes_saved: 6.0,
            priority: "high".into(),
            request_job: Some(JobId::parse("20261104-100000-d000").unwrap()),
        });
        let design_only = "design request 20261104-100000-d000 queued at high priority, predicting 6.0 min saved per landed job";
        assert_eq!(throughput_outcome(&review), design_only);
        review.note = "slowdown request 20261104-100100-s000 queued at high priority".into();
        let both = throughput_outcome(&review);
        assert!(both.starts_with(design_only), "{both}");
        assert!(
            both.contains("slowdown request 20261104-100100-s000 queued"),
            "{both}"
        );
        review.proposal = None;
        assert!(throughput_outcome(&review).starts_with("no proposal: slowdown request"));
    }

    // -----------------------------------------------------------------------------------------------------
    // Improvement requests: the drain's guaranteed turn

    /// The request the error review queues for a cause recorded in `jobs` (letters), through the real path.
    fn queue_cause_in(
        dir: &std::path::Path,
        athena: &Athena,
        in_class: &FailureClass,
        jobs: &[char],
    ) -> JobId {
        let mut queued = None;
        for (n, of) in jobs.iter().enumerate() {
            let i = introspection(
                10 + n as u32,
                *of,
                Some(&in_class.id),
                "The verdict is not read.",
            );
            put_introspection(dir, athena, &i);
            if let Some(job) = athena.error_review(&i, &crate::Quiet) {
                queued = Some(job);
            }
        }
        queued.expect("the cause recurred, so the review queued a request")
    }

    #[test]
    fn improvement_of_counts_the_distinct_jobs_of_the_cause_a_request_addresses() {
        // A failure class that appears in introspections of three distinct jobs, and one that appears in two.
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let c = class(1, "Reading ignores the earlier verdict");
        put(
            dir.path(),
            &athena,
            vec![AnyRecord::FailureClass(c.clone())],
        );
        let queued = queue_cause_in(dir.path(), &athena, &c, &['a', 'b', 'c']);
        let found = improvement_of(&records(&athena));
        assert_eq!(found.items, [queued.as_str()]);
        assert_eq!(found.cause_jobs.get(queued.as_str()), Some(&3));

        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        put(
            dir.path(),
            &athena,
            vec![AnyRecord::FailureClass(c.clone())],
        );
        let queued = queue_cause_in(dir.path(), &athena, &c, &['a', 'b']);
        let found = improvement_of(&records(&athena));
        assert_eq!(found.items, [queued.as_str()]);
        assert_eq!(found.cause_jobs.get(queued.as_str()), Some(&2));
        // Two introspections of one job are one job: the count is of jobs.
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        put(
            dir.path(),
            &athena,
            vec![AnyRecord::FailureClass(c.clone())],
        );
        let queued = queue_cause_in(dir.path(), &athena, &c, &['a', 'b', 'b']);
        let found = improvement_of(&records(&athena));
        assert_eq!(found.cause_jobs.get(queued.as_str()), Some(&2));
    }

    #[test]
    fn improvement_of_feeds_the_plan_so_a_three_job_request_is_ordered_as_urgent() {
        use crate::plan::{Plan, PlanEntry, TierCaps};
        let build = |jobs: &[char]| {
            let dir = tempfile::tempdir().unwrap();
            let state = tempfile::tempdir().unwrap();
            let athena = athena_in(dir.path(), state.path());
            let c = class(1, "Reading ignores the earlier verdict");
            put(
                dir.path(),
                &athena,
                vec![AnyRecord::FailureClass(c.clone())],
            );
            let hestia = queue_cause_in(dir.path(), &athena, &c, jobs);
            let found = improvement_of(&records(&athena));
            let ready = |id: &str| PlanEntry {
                goal: format!("goal of {id}"),
                rung: Some(athena_data::roles::Ladder::Implementation),
                acceptance: vec![format!("{id} works")],
                touches: vec![format!("{id}/")],
                ..PlanEntry::bare(id, id)
            };
            // An older normal request of a person comes first by age.
            let entries = [ready("person"), ready(hestia.as_str())];
            let (plan, capped) =
                Plan::build_improving(&entries, &[], TierCaps::UNLIMITED, &[], &found);
            (hestia, plan, capped)
        };
        let (hestia, plan, capped) = build(&['a', 'b', 'c']);
        assert_eq!(plan.order[0], hestia.as_str(), "ordered ahead as urgent");
        assert_eq!(plan.promoted.len(), 1);
        assert_eq!(plan.promoted[0].jobs, 3);
        assert_eq!(capped[1].priority, Priority::Urgent);
        let (hestia, plan, capped) = build(&['a', 'b']);
        assert_eq!(plan.order[1], hestia.as_str(), "two jobs: not promoted");
        assert!(plan.promoted.is_empty());
        assert_eq!(capped[1].priority, Priority::Normal);
    }

    #[test]
    fn improvement_of_lists_every_kind_of_request_it_can_link_and_nothing_else() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        assert_eq!(improvement_of(&records(&athena)), Improvement::default());

        // A plain request of a person that cites nothing.
        let (plain, _) = athena
            .queue("Add a flag.\n\nAcceptance:\n- the flag works\n\nFiles: src/a.rs\n")
            .unwrap();
        // A spec-review request carries the requirement line; the same text without it is not one.
        let spec = |line: &str| {
            format!(
                "Cover the requirement.\n{line}\n\nAcceptance:\n- it is covered\n\nFiles: src/a.rs\n"
            )
        };
        let (spec_open, _) = athena.queue(&spec("Spec requirement: R-1")).unwrap();
        let (no_line, _) = athena.queue(&spec("Spec requirement R-1")).unwrap();
        let (closed_spec, _) = athena.queue(&spec("Spec requirement: R-2")).unwrap();
        athena
            .close(closed_spec.as_str(), "not wanted", &crate::Quiet)
            .unwrap();
        // A request that cites the introspections of a repeated cause is the one that addresses it.
        let c = class(1, "Reading ignores the earlier verdict");
        put(
            dir.path(),
            &athena,
            vec![AnyRecord::FailureClass(c.clone())],
        );
        let cause = queue_cause_in(dir.path(), &athena, &c, &['a', 'b', 'c']);

        let found = improvement_of(&records(&athena));
        let mut expected = vec![spec_open.as_str().to_string(), cause.as_str().to_string()];
        expected.sort();
        assert_eq!(found.items, expected);
        for not_listed in [&plain, &no_line, &closed_spec] {
            assert!(!found.items.contains(&not_listed.as_str().to_string()));
        }
        // Only the request that addresses a cause carries a count.
        assert_eq!(found.cause_jobs.len(), 1);
        assert_eq!(found.cause_jobs[cause.as_str()], 3);
        assert_eq!((found.since_turn, found.tried.len()), (0, 0));

        // The cause's request, once closed, is not listed.
        athena
            .close(cause.as_str(), "not wanted", &crate::Quiet)
            .unwrap();
        let found = improvement_of(&records(&athena));
        assert_eq!(found.items, [spec_open.as_str()]);
    }

    #[test]
    fn improvement_of_lists_the_throughput_design_request_and_not_the_slowdown_request() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let (model, _) = throughput_model(12.0);
        let athena = athena_with(dir.path(), state.path(), model);
        seed_landed(dir.path(), &athena, 0, 10, 20.0);
        let review = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        let design = review.proposal.unwrap().request_job.unwrap();
        let found = improvement_of(&records(&athena));
        assert_eq!(
            found.items,
            [design.as_str()],
            "linked by the recorded proposal"
        );
        assert!(found.cause_jobs.is_empty());
        // Closed, the design request is not listed.
        athena
            .close(design.as_str(), "not wanted", &crate::Quiet)
            .unwrap();
        assert!(improvement_of(&records(&athena)).items.is_empty());

        // The slowdown request has no recorded link, so it is not listed (and no title is matched).
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let (model, _) = throughput_model(6.0);
        let athena = athena_with(dir.path(), state.path(), model);
        seed_landed(dir.path(), &athena, 0, 10, 25.0);
        put_earlier_review(dir.path(), &athena, 10, 25.0);
        open_design_request_job(dir.path(), &athena);
        seed_stages(dir.path(), &athena, 10, 11, slow_stages());
        athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        let known = records(&athena);
        let slowdown = open_slowdown_requests(&known);
        assert_eq!(slowdown.len(), 1, "the review queued a slowdown request");
        assert!(improvement_of(&known).items.is_empty());
    }

    #[test]
    fn improvement_of_lists_open_steward_channel_requests() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let steward = job('s');
        put(
            dir.path(),
            &athena,
            vec![AnyRecord::Request(Request {
                job: steward.clone(),
                text: "Fix the cause.\n\nAcceptance:\n- it is fixed\n\nFiles: src/a.rs\n".into(),
                received_at: at(5),
                channel: crate::steward::REQUEST_CHANNEL.into(),
            })],
        );
        let (person, _) = athena
            .queue("Add a flag.\n\nAcceptance:\n- the flag works\n\nFiles: src/a.rs\n")
            .unwrap();
        let found = improvement_of(&records(&athena));
        assert_eq!(found.items, [steward.as_str()]);
        assert!(!found.items.contains(&person.as_str().to_string()));
    }

    // -----------------------------------------------------------------------------------------------------
    // Recurring duties

    /// A runner that does no work: it logs the ids it is asked to run, and fails the one it is told to.
    #[derive(Default)]
    struct CountingRunner {
        log: std::cell::RefCell<Vec<String>>,
        fail: std::cell::Cell<Option<&'static str>>,
    }

    impl CountingRunner {
        fn take(&self) -> Vec<String> {
            std::mem::take(&mut *self.log.borrow_mut())
        }
    }

    impl DutyRunner for CountingRunner {
        fn run(&self, duty: &Duty, _: &dyn Ui, _: Timestamp) -> Result<DutyOutput> {
            self.log.borrow_mut().push(duty.id.to_string());
            if self.fail.get() == Some(duty.id) {
                bail!("this duty is broken");
            }
            Ok(DutyOutput {
                check: CheckOutput {
                    examined: 1,
                    findings: Vec::new(),
                },
                lines: Vec::new(),
            })
        }
    }

    const fn test_duty(id: &'static str, cadence: Cadence) -> Duty {
        Duty {
            id,
            name: "A test duty",
            cadence,
            how: How::Runner,
            runs_in: "a test",
        }
    }

    const A_DRAIN: Duty = test_duty("a-drain", Cadence::DrainStart);
    const B_LANDING: Duty = test_duty("b-landing", Cadence::EveryLanding);
    const C_THIRD: Duty = test_duty("c-third", Cadence::EveryLandings(3));

    #[test]
    fn a_sweep_runs_each_due_duty_records_the_last_run_and_a_second_sweep_runs_none() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let table = [A_DRAIN, B_LANDING, C_THIRD];
        let runner = CountingRunner::default();
        let mut clock = 0;
        // One pass: the ids the runner was asked to run, and the sweep as committed and read back.
        let mut pass = |occasion: Occasion, landings: u32| {
            clock += 1;
            let ctx = DutyContext {
                occasion,
                landings,
                now: at(clock),
            };
            let sweep = athena
                .hestia_sweep_duties(&[], &Quiet, &table, &runner, ctx)
                .unwrap();
            let committed = Sweep::latest(&records(&athena).hestia_sweeps)
                .cloned()
                .unwrap();
            assert_eq!(committed, sweep, "the record read back is the one returned");
            let listed: Vec<String> = sweep.checks.iter().map(|c| c.check.clone()).collect();
            assert_eq!(runner.take(), listed, "the record lists what ran");
            assert_eq!(
                sweep.duty_pass,
                Some(DutyPass { occasion, landings }),
                "the pass stores its occasion and the landing total"
            );
            listed
        };
        // The first sweep finds every duty never run.
        assert_eq!(
            pass(Occasion::Sweep, 0),
            ["a-drain", "b-landing", "c-third"]
        );
        // A second sweep within the cadence runs none.
        assert!(pass(Occasion::Sweep, 0).is_empty());
        // One landing: only the per-landing duty. The drain-start duty has no landing cadence.
        assert_eq!(pass(Occasion::Sweep, 1), ["b-landing"]);
        // Three landings since the first run: the every-3-landings duty is due as well.
        assert_eq!(pass(Occasion::Sweep, 3), ["b-landing", "c-third"]);
        assert!(pass(Occasion::Sweep, 3).is_empty());
        // A drain start runs the drain-start duties and nothing else, every time.
        assert_eq!(pass(Occasion::DrainStart, 3), ["a-drain"]);
        assert_eq!(pass(Occasion::DrainStart, 3), ["a-drain"]);
        // A landing runs the landing duties that are due: one landing since c-third last ran is not enough.
        assert_eq!(pass(Occasion::Landing, 4), ["b-landing"]);
        assert_eq!(pass(Occasion::Landing, 6), ["b-landing", "c-third"]);
        // A duty that failed is not counted as run: it is due at the next pass that may run it.
        runner.fail.set(Some("a-drain"));
        assert_eq!(pass(Occasion::DrainStart, 6), ["a-drain"]);
        let failed = Sweep::latest(&records(&athena).hestia_sweeps)
            .cloned()
            .unwrap();
        assert!(failed.checks[0].errored);
        assert!(failed.findings[0].evidence.contains("this duty is broken"));
        runner.fail.set(None);
        assert_eq!(pass(Occasion::Sweep, 6), ["a-drain"]);
        assert!(pass(Occasion::Sweep, 6).is_empty());
    }

    #[test]
    fn the_duty_table_matches_the_design_doc() {
        let doc = include_str!("../../../docs/design/hestia.md");
        let section: Vec<&str> = doc
            .lines()
            .skip_while(|l| !l.starts_with("## 13. Recurring duties"))
            .skip(1)
            .take_while(|l| !l.starts_with("## "))
            .collect();
        assert!(
            !section.is_empty(),
            "the doc has a Recurring duties section"
        );
        let rows: Vec<Vec<&str>> = section
            .iter()
            .filter(|l| l.trim_start().starts_with('|'))
            .map(|l| {
                l.trim()
                    .trim_matches('|')
                    .split('|')
                    .map(str::trim)
                    .collect()
            })
            .skip(2)
            .collect();
        assert_eq!(rows.len(), DUTIES.len(), "one row for each duty");
        for (row, duty) in rows.iter().zip(DUTIES.iter()) {
            assert_eq!(
                row.len(),
                5,
                "id, duty, owner, cadence, where it runs: {row:?}"
            );
            assert_eq!(row[0].trim_matches('`'), duty.id);
            assert!(
                row[1].eq_ignore_ascii_case(duty.name),
                "{} is named {:?} in the doc",
                duty.id,
                row[1]
            );
            assert_eq!(row[2], "Hestia's sweep", "{}", duty.id);
            assert_eq!(row[3], duty.cadence.describe(), "{}", duty.id);
            assert!(!row[4].is_empty(), "{} says where it runs", duty.id);
        }
        // The six duties of the brief, by name.
        for name in [
            "error-review backlog pass",
            "evidence-label derivation",
            "code-health check",
            "periodic design review",
            "keep-rule threshold check",
            "reading-rule refusal-rate check",
        ] {
            assert!(
                DUTIES.iter().any(|d| d.name.eq_ignore_ascii_case(name)),
                "{name}"
            );
        }
    }

    #[test]
    fn a_landing_pass_does_not_hide_the_last_full_sweep() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let runner = CountingRunner::default();
        let ctx = |occasion, n| DutyContext {
            occasion,
            landings: 1,
            now: at(n),
        };
        // Only duty passes so far: the newest of them is the last sweep.
        let first = athena
            .hestia_sweep_duties(
                &[],
                &Quiet,
                &[B_LANDING],
                &runner,
                ctx(Occasion::Landing, 1),
            )
            .unwrap();
        assert_eq!(athena.last_sweep().unwrap(), Some(first));
        // A full sweep with a finding, then a landing pass that lists only the duty that ran.
        let found: Vec<Box<dyn Check>> = vec![Box::new(Fixed {
            id: "branches",
            answer: Ok(CheckOutput {
                examined: 2,
                findings: vec![finding("branches", "old-branch")],
            }),
        })];
        let full = athena
            .hestia_sweep_duties(&found, &Quiet, &[], &runner, ctx(Occasion::Sweep, 2))
            .unwrap();
        assert_eq!(full.findings.len(), 1);
        let landing = athena
            .hestia_sweep_duties(
                &[],
                &Quiet,
                &[B_LANDING],
                &runner,
                ctx(Occasion::Landing, 3),
            )
            .unwrap();
        assert!(landing.is_duty_pass());
        assert!(!full.is_duty_pass());
        assert_eq!(athena.last_sweep().unwrap(), Some(full));
        assert_eq!(records(&athena).hestia_sweeps.len(), 3);
    }

    #[test]
    fn duty_commits_and_interventions_do_not_change_the_landing_total() {
        use crate::code_health::tests::{land, set_up};
        let (_dir, _state, athena) = set_up();
        let total = |a: &Athena| {
            crate::code_health::landings(&a.git, &a.head().unwrap())
                .unwrap()
                .len()
        };
        let base = athena.head().unwrap();
        land(&athena, &[("a.txt", "a\n".into())], "j1", crate::now(), 2);
        let landed = athena.head().unwrap();
        assert_eq!(total(&athena), 1);
        // The host squashes the landing: another commit with the landed content, trailers kept, on the base.
        let tree = athena.git.tree_of(&landed).unwrap();
        let merged = athena
            .git
            .run([
                "commit-tree",
                tree.trim(),
                "-p",
                &base,
                "-m",
                "Land j1 (#7)\n\nAthena-Job: j1\nAthena-Verified-Content: fp-j1\n",
            ])
            .unwrap()
            .trim()
            .to_string();
        athena.git.run(["reset", "-q", "--hard", &merged]).unwrap();
        assert_eq!(
            total(&athena),
            1,
            "a squash that keeps the trailers is one landing"
        );

        let table = [B_LANDING];
        let runner = CountingRunner::default();
        // The first landing hook runs the per-landing duty.
        let ctx = athena.duty_context(Occasion::Landing).unwrap();
        assert_eq!(ctx.landings, 1);
        athena
            .hestia_sweep_duties(&[], &Quiet, &table, &runner, ctx)
            .unwrap();
        assert_eq!(runner.take(), ["b-landing"]);
        // The pass itself is a records-only commit, and a person's fix carries no trailers: neither is a landing.
        athena
            .git
            .run(["commit", "--allow-empty", "-q", "-m", "Fix by hand"])
            .unwrap();
        assert_eq!(total(&athena), 1);
        // The second hook for the same landing finds nothing due.
        let ctx = athena.duty_context(Occasion::Landing).unwrap();
        let report = athena
            .execute_duties(&table, &runner, &ctx, &Quiet)
            .unwrap();
        assert!(report.results.is_empty());
        assert!(runner.take().is_empty());
        // The next landing is counted.
        land(&athena, &[("b.txt", "b\n".into())], "j2", crate::now(), 1);
        assert_eq!(total(&athena), 2);
        let ctx = athena.duty_context(Occasion::Landing).unwrap();
        let report = athena
            .execute_duties(&table, &runner, &ctx, &Quiet)
            .unwrap();
        assert_eq!(report.results.len(), 1);
    }
}
