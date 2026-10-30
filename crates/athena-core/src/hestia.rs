//! Hestia's sweep skeleton (`docs/design/hestia.md` section 9, job 1; decision D129).
//!
//! A sweep runs a list of [`Check`]s and records one report, an `athena.hestia-sweep/1` record: every check that
//! ran, with the number of things it looked at and the number of findings, and each finding. A check that
//! returns an error does not stop the sweep and does not look clean: [`run_sweep`] turns the error into a
//! finding for that check and marks the check as errored. Hestia reports and never gates work (D127).
//!
//! The trigger is an explicit command: `athena hestia sweep` runs and records a sweep, and `athena hestia last`
//! prints the newest one. Nothing runs a sweep on a timer or inside `athena drain`; D129 says why.
//!
//! This skeleton ships no checks. [`default_checks`] is empty, and the checks of the design's rows 1 to 8 are
//! later jobs that add themselves to it.
//!
//! The sweep also reads Athena's own spec. Every active requirement in `docs/spec.toml` records a mechanism, an
//! owner and a measure. [`Athena::spec_review`] (a review pass, see [`Athena::review_passes`], because a [`Check`]
//! cannot reach the records or the queue) queues one request through [`StewardRequest`] for each active
//! requirement whose mechanism is `none` or whose measure is blank or names a label `athena status` does not
//! print ([`measure_problem`]). The request is urgent when the requirement is about speed ([`is_speed`]), and an
//! open request that holds the line `Spec requirement: <id>` stops a second one. At most
//! [`SPEC_REQUESTS_PER_PASS`] are queued in one pass, speed first. `athena status` lists the same gaps
//! ([`spec_gaps`]).
//!
//! Hestia also reviews errors across jobs (`docs/design/hestia.md`, "Error review"). Every failure gets an
//! introspection with a cause, but each one is written about one job. After an introspection is recorded,
//! [`Athena::error_review`] groups it with the earlier ones by cause ([`cause_key`]). A cause that has occurred
//! in two or more distinct jobs, and that no open or landed request cites, gets one urgent root-cause request,
//! built by [`crate::steward::StewardRequest`] and citing the introspection ids. [`repeated_causes`] lists the
//! repeated causes for `athena status`.
//!
//! The review only reacts to a new introspection, so causes that repeated before it existed have no request.
//! The backfill ([`Athena::error_review_backfill`]) sweeps them. Athena runs it at the start of every drain and in
//! each sweep ([`Athena::review_passes`]); `athena hestia review --backfill` runs it on request: a cause whose stage is
//! switched off in `athena.toml` is dormant and skipped; an open request that already addresses a cause is
//! re-queued through the same builder and the old one closed; any other cause gets a new root-cause request.
//! The five highest-ranked causes (most occurrences, then most recent) are urgent and the rest normal. A second
//! run finds every cause covered and changes nothing.
//!
//! The throughput review (`docs/design/hestia.md`, "Throughput review") asks a different question: not what
//! failed but what the pipeline waits on. It runs in [`Athena::review_passes`] when [`throughput_due`] says so:
//! every [`crate::code_health::REVIEW_EVERY`] landings, or when the median minutes per landed job grew by more
//! than 20% since the previous review. A strong model is given the status throughput section
//! ([`crate::status::Throughput`]) and `docs/design`, and names the biggest bottleneck and the architectural
//! change that removes it, with the minutes it expects that change to save per landed job. The top proposal is
//! queued as a design request through [`StewardRequest`], at a priority set by those minutes
//! ([`priority_for_minutes`]), and the prediction is kept in an `athena.throughput-review/1` record. A later
//! review compares the minutes measured after a change that cites the review with the prediction
//! ([`throughput_followups`]).
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
use crate::plan::Priority;
use crate::state::format_bytes;
use crate::status::{LandedJob, Throughput, landed_jobs, median};
use crate::steward::StewardRequest;
use crate::store::{JobRecords, JobState, Records};
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_agent::env::Side;
use athena_context::{CONTRACT, Component, Composed, compose};
use athena_data::records::{
    AnyRecord, BuildMeasurement, HestiaCheckResult, IntrospectionRecord, IntrospectionStatus,
    Invocation, InvocationOutcome, OutcomeState, Role, ThroughputFollowup, ThroughputProposal,
    ThroughputReview, ThroughputTrigger,
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
        let check_id = check.id().to_string();
        let outcome = check.run(at).and_then(|output| {
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
            Ok(output) => {
                results.push(CheckResult {
                    check: check_id.clone(),
                    examined: output.examined,
                    count: output.findings.len() as u32,
                    errored: false,
                });
                findings.extend(output.findings.into_iter().map(|f| Finding {
                    check: check_id.clone(),
                    ..f
                }));
            }
            Err(error) => {
                results.push(HestiaCheckResult {
                    check: check_id.clone(),
                    examined: 0,
                    count: 1,
                    errored: true,
                });
                findings.push(Finding {
                    check: check_id.clone(),
                    subject: check_id,
                    evidence: format!("the check failed to run: {error:#}"),
                    age_seconds: 0,
                    state: FindingState::Open,
                });
            }
        }
    }
    Sweep {
        id,
        at,
        checks: results,
        findings,
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
/// introspection that is not a case: one that failed, has no job, or has no cause.
pub fn cause_key(records: &Records, i: &IntrospectionRecord) -> Option<String> {
    let cause = i.cause.as_deref()?;
    if i.status != IntrospectionStatus::Recorded || i.job.is_none() {
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
/// address the cause, so the review may queue another.
fn addressing_request(records: &Records, introspections: &[String]) -> Option<AddressingRequest> {
    let mut found: Option<AddressingRequest> = None;
    for job in records.request_jobs() {
        let state = job.state();
        let landed = state == JobState::Closed(OutcomeState::Landed);
        if matches!(state, JobState::Closed(_)) && !landed {
            continue;
        }
        let cites = job
            .request
            .as_ref()
            .is_some_and(|r| introspections.iter().any(|id| r.text.contains(id.as_str())));
        if !cites {
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
    found
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

/// The text of the urgent root-cause request for a repeated cause, built with [`StewardRequest`]. No body line
/// starts like a bullet: the builder refuses one, since `athena plan` would read it as acceptance.
pub fn root_cause_request(cause: &RepeatedCause, records: &Records) -> Result<String, String> {
    root_cause_request_at(cause, records, Priority::Urgent)
}

/// [`root_cause_request`] at the given priority: the one place the text of a root-cause request is built.
fn root_cause_request_at(
    cause: &RepeatedCause,
    records: &Records,
    priority: Priority,
) -> Result<String, String> {
    let mut body = String::from(
        "The same cause has produced failures in more than one job. Find the cause where it originates and fix \
         it there, so the whole class is prevented; do not work around one instance.\n\n",
    );
    if let Some(class) = &cause.class {
        body.push_str(&format!("Failure class: {class}\n"));
    }
    body.push_str(&format!("Cause: {}\n", cause.cause));
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
    let title: String = cause.cause.chars().take(100).collect();
    StewardRequest::new(
        Some(priority),
        &format!("Fix the root cause of a repeated failure: {title}"),
        &body,
        &acceptance,
        &["crates/".to_string()],
    )
    .map(|r| r.render())
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
/// requirement's request is urgent.
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
/// urgent for a requirement about speed and normal otherwise.
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
        Priority::Urgent
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
    let line = format!("{SPEC_LINE}{id}");
    records.request_jobs().iter().any(|job| {
        !matches!(job.state(), JobState::Closed(_))
            && job
                .request
                .as_ref()
                .is_some_and(|r| r.text.lines().any(|l| l.trim() == line))
    })
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
    /// failing measure, queues one request through the steward request builder, urgent when the requirement is
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

/// How many of the highest-ranked causes a backfill makes urgent. The rest are normal.
pub const BACKFILL_URGENT: usize = 5;

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
        .map(crate::introspect::normalise)
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
/// [`BACKFILL_URGENT`] are urgent. The plan lists the ranked causes first, in rank order.
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
            let priority = if rank < BACKFILL_URGENT {
                Priority::Urgent
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
/// means the last proposal is still being worked, and the review queues no second one.
pub const THROUGHPUT_MARKER: &str = "Throughput design: ";

/// Expected minutes saved per landed job at or above which a design request is urgent.
pub const URGENT_MINUTES: f64 = 10.0;
/// ... high.
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
/// one job landed since the previous review and the median minutes per landed job is more than
/// [`WORSEN_RATIO`] times the one that review recorded. `landed` and `median` are the status throughput
/// section's `landed_total` and `median_minutes`.
pub fn throughput_due(
    landed: usize,
    median: f64,
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
    let worse = previous.window_median_minutes > 0.0
        && median / previous.window_median_minutes > WORSEN_RATIO;
    (landed > before && worse).then_some(ThroughputTrigger::Worsened)
}

/// The priority a design request gets from the minutes per landed job it is expected to save: more minutes,
/// higher priority (speed first).
pub fn priority_for_minutes(expected_minutes_saved: f64) -> Priority {
    if expected_minutes_saved >= URGENT_MINUTES {
        Priority::Urgent
    } else if expected_minutes_saved >= HIGH_MINUTES {
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

/// The open request job, if any, whose text has a line that starts with [`THROUGHPUT_MARKER`]. A closed one
/// (landed, rejected, failed or abandoned) does not count.
fn open_throughput_request(records: &Records) -> Option<&JobRecords> {
    records.request_jobs().into_iter().find(|j| {
        !matches!(j.state(), JobState::Closed(_))
            && j.request
                .as_ref()
                .is_some_and(|r| r.text.lines().any(|l| l.starts_with(THROUGHPUT_MARKER)))
    })
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
    /// architectural change that removes it, queues one design request for the proposal (none while an earlier
    /// one is open), and records the review with the prediction and the follow-ups it closes. Returns the
    /// record, or None when the review is not due, the run is nested, or the kill switch is on. A model that
    /// fails or answers badly still gives a record, without a proposal, so the cadence moves on; the invocation
    /// is recorded either way, since it cost money.
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
        let Some(trigger) = throughput_due(measured.landed_total, measured.median_minutes, latest)
        else {
            return Ok(None);
        };
        let now = crate::now();
        let mut id = RecordId::generate(now);
        while records.has_record_id(&id) {
            id = RecordId::generate(now);
        }
        let followups = throughput_followups(&records, &landed_jobs(&records));

        let mut note = String::new();
        let mut proposal = None;
        let mut invocation: Option<Invocation> = None;
        if let Some(open) = open_throughput_request(&records) {
            note = format!(
                "throughput design request {} is still open; the model was not asked for another",
                open.id()
            );
        } else {
            let (made, answer) = self.ask_throughput(&head, &measured, trigger, now)?;
            invocation = made;
            match answer {
                Ok(answer) => {
                    let priority = priority_for_minutes(answer.expected_minutes_saved);
                    let request_job =
                        match throughput_request(&id, &answer, measured.median_minutes) {
                            Ok(text) => match self.queue(&text) {
                                Ok((job, _)) => Some(job),
                                Err(e) => {
                                    note = format!("the design request could not be queued: {e:#}");
                                    None
                                }
                            },
                            Err(e) => {
                                note = format!("the design request could not be built: {e}");
                                None
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
                Err(e) => note = format!("no proposal: {e:#}"),
            }
        }

        let review = ThroughputReview {
            id,
            at: now,
            landings: u32::try_from(measured.landed_total).unwrap_or(u32::MAX),
            window_median_minutes: measured.median_minutes,
            trigger,
            proposal,
            followups,
            note,
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
        let subject = match review
            .proposal
            .as_ref()
            .and_then(|p| p.request_job.as_ref())
        {
            Some(job) => format!(
                "Throughput review: median {:.1} min per landed job; design request {job}",
                review.window_median_minutes
            ),
            None => format!(
                "Throughput review: median {:.1} min per landed job; no request queued",
                review.window_median_minutes
            ),
        };
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
        Ok(Some(review))
    }

    /// Runs the model once over a scratch workspace holding the throughput section and the design documents.
    /// Returns the recorded invocation (None when the model could not start) and its validated, redacted answer.
    fn ask_throughput(
        &self,
        head: &str,
        measured: &Throughput,
        trigger: ThroughputTrigger,
        now: Timestamp,
    ) -> Result<(Option<Invocation>, Result<ThroughputOutput>)> {
        let jc = JobConfig::at(&self.git, head)?;
        let run_job = JobId::Id(RecordId::generate(now));
        let job_dir = self.state.job_dir(&run_job);
        let _cleanup = Cleanup(job_dir.clone());
        let workspace = job_dir.join("workspace");
        std::fs::create_dir_all(&workspace)?;
        std::fs::write(
            workspace.join(THROUGHPUT_FILE),
            format!(
                "# Throughput\n\nThis review ran because of: {}.\n\n```\n{}\n```\n\nThe stages are minutes of the model \
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
    /// landed request cites it, queues one urgent root-cause request citing the introspection ids. Returns the
    /// job queued, if any. Never fails: a problem is a line to the user, since the review never blocks a job.
    pub fn error_review(&self, recorded: &IntrospectionRecord, ui: &dyn Ui) -> Option<JobId> {
        match self.try_error_review(recorded) {
            Ok(Some((job, cause))) => {
                ui.line(&format!(
                    "hestia     repeated cause in {} jobs ({} introspections); root-cause request {job} queued at urgent priority",
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
            .map_err(|e| anyhow::anyhow!("the root-cause request: {e}"))?;
        let (job, _) = self.queue(&text)?;
        Ok(Some((job, cause)))
    }

    /// Athena's own housekeeping, run at the start of every drain and in every sweep so that no person has to
    /// run `athena hestia review --backfill` or `athena score-jev --derive` (both commands stay available).
    /// Runs the error review backfill, then the evidence-label derivation, and returns one summary line for
    /// each. Both are idempotent: over unchanged records they write nothing. A pass that fails is reported in
    /// its line and does not stop the other or the caller.
    pub fn review_passes(&self, ui: &dyn Ui) -> Vec<String> {
        let backfill = match self.error_review_backfill(ui) {
            Ok(r) => format!(
                "hestia     backfill: {} repeated cause(s) · queued {} · re-queued {} · priority set {} · skipped {}",
                r.lines.len(),
                r.count(BackfillAction::Queued),
                r.count(BackfillAction::Requeued),
                r.count(BackfillAction::Reprioritised),
                r.count(BackfillAction::Skipped)
            ),
            Err(e) => format!("hestia     backfill: could not run: {e:#}"),
        };
        let derive = match self.derive_labels() {
            Ok(s) => format!(
                "hestia     evidence labels: {} derived · {} already derived · {} reviewed by a person · {} without evidence",
                s.derived, s.already_derived, s.reviewed, s.no_evidence
            ),
            Err(e) => format!("hestia     evidence labels: could not run: {e:#}"),
        };
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
        let mut lines = vec![backfill, derive, spec];
        // The throughput review is due every ten landings or on a 20% slowdown, so most calls print nothing.
        match self.throughput_review(ui) {
            Ok(Some(r)) => {
                let outcome = match r.proposal.as_ref() {
                    Some(p) => match &p.request_job {
                        Some(job) => format!(
                            "design request {job} queued at {} priority, predicting {:.1} min saved per landed job",
                            p.priority, p.expected_minutes_saved
                        ),
                        None => format!("proposal recorded, no request queued: {}", r.note),
                    },
                    None => format!("no proposal: {}", r.note),
                };
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
        self.hestia_sweep(checks)
    }

    /// Runs `checks`, commits the report as a record, and returns it. The review passes of
    /// [`Athena::review_passes`] run first and their summary lines go to `ui`, then the build measurement
    /// pass ([`Athena::measure_build_pass`]) with the real builder.
    pub fn hestia_sweep_with(&self, checks: &[Box<dyn Check>], ui: &dyn Ui) -> Result<Sweep> {
        if self.git.head()?.is_some() {
            for line in self.review_passes(ui) {
                ui.line(&line);
            }
        }
        self.hestia_sweep_measuring(checks, ui, &CargoBuild)
    }

    /// Runs `checks`, commits the report as a record, and returns it. Prints nothing; the sweep that also runs
    /// the review passes is [`Athena::hestia_sweep_with`].
    pub fn hestia_sweep(&self, checks: &[Box<dyn Check>]) -> Result<Sweep> {
        let known = match self.git.head()? {
            Some(head) => Records::load(&self.git, &head)?,
            None => bail!(
                "the repository has no commits yet; commit athena.toml and defaults.toml first"
            ),
        };
        let now = crate::now();
        let mut id = RecordId::generate(now);
        while known.has_record_id(&id) {
            id = RecordId::generate(now);
        }
        let sweep = run_sweep(checks, id, now);
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

    /// The newest recorded sweep, or None when none was recorded.
    pub fn last_sweep(&self) -> Result<Option<Sweep>> {
        let Some(head) = self.git.head()? else {
            return Ok(None);
        };
        let records = Records::load(&self.git, &head)?;
        Ok(Sweep::latest(&records.hestia_sweeps).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn a_speed_requirement_is_urgent_and_a_statement_with_faster_is_not_enough() {
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
        assert!(spec_request_text(&athena, "R-1").starts_with("Priority: urgent\n"));
        assert!(spec_request_text(&athena, "R-3").starts_with("Priority: urgent\n"));
        assert!(spec_request_text(&athena, "R-2").starts_with("Priority: normal\n"));
        let known = records(&athena);
        let urgent = known
            .request_jobs()
            .iter()
            .filter(|j| j.priority() == Priority::Urgent)
            .count();
        assert_eq!(urgent, 2);
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

        // If the acceleration requirement ever had no mechanism, its request would be urgent.
        let mut acceleration = by("OLY-A-22").clone();
        acceleration.mechanism = "none".into();
        let gaps = spec_gaps(&[acceleration]);
        assert_eq!(gaps.len(), 1);
        let text = spec_request(&gaps[0]).unwrap();
        assert!(text.starts_with("Priority: urgent\n"), "{text}");
        assert!(text.contains("Spec requirement: OLY-A-22\n"), "{text}");
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
    fn two_introspections_of_one_cause_in_different_jobs_queue_one_urgent_request_and_a_third_queues_none()
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
            request.text.starts_with("Priority: urgent\n"),
            "{}",
            request.text
        );
        assert_eq!(
            known.jobs[queued.as_str()].priority(),
            crate::plan::Priority::Urgent
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
    fn the_root_cause_request_is_urgent_and_refuses_a_cause_that_reads_as_acceptance() {
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
    fn the_backfill_skips_dormant_requeues_an_existing_request_queues_the_rest_and_ranks_the_top_five_urgent()
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
            let want = if rank < 5 { "urgent" } else { "normal" };
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
            text.starts_with("Priority: urgent\n\nSplit large readings\n"),
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
                Priority::Urgent
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
        assert_eq!(ui.0.lock().unwrap().len(), 3, "one line per pass");
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
        assert_eq!(known.jobs[job.as_str()].priority(), Priority::Urgent);
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

    use crate::status::StageMinutes;
    use crate::status::planning_cost_tests::timed_job;
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
        let mut all = Vec::new();
        for g in from..from + count {
            let (day, hour) = (1 + g / 12, (g % 12) * 2);
            all.extend(timed_job(
                &format!("202609{day:02}-{hour:02}0000-t{g:03}"),
                format!("2026-09-{day:02}T{hour:02}:00:00Z")
                    .parse()
                    .unwrap(),
                producer_only(minutes),
                OutcomeState::Landed,
            ));
        }
        put(dir, athena, all);
    }

    fn open_design_requests(known: &Records) -> Vec<&JobRecords> {
        known
            .request_jobs()
            .into_iter()
            .filter(|j| !matches!(j.state(), JobState::Closed(_)))
            .filter(|j| {
                j.request
                    .as_ref()
                    .is_some_and(|r| r.text.lines().any(|l| l.starts_with(THROUGHPUT_MARKER)))
            })
            .collect()
    }

    fn earlier_review(landings: u32, median: f64) -> ThroughputReview {
        ThroughputReview {
            id: rid("20260902-000000-r001"),
            at: "2026-09-02T00:00:00Z".parse().unwrap(),
            landings,
            window_median_minutes: median,
            trigger: ThroughputTrigger::Cadence,
            proposal: None,
            followups: vec![],
            note: String::new(),
        }
    }

    #[test]
    fn a_throughput_review_is_due_at_ten_landings_and_on_a_rise_of_more_than_twenty_percent() {
        use ThroughputTrigger::{Cadence, Worsened};
        // No review yet: ten landings make the first one due, nine do not, however slow they were.
        assert_eq!(throughput_due(9, 500.0, None), None);
        assert_eq!(throughput_due(10, 5.0, None), Some(Cadence));
        // A review at 10 landings with a median of 10 minutes.
        let prior = earlier_review(10, 10.0);
        assert_eq!(throughput_due(19, 10.0, Some(&prior)), None, "nine more");
        assert_eq!(throughput_due(20, 10.0, Some(&prior)), Some(Cadence));
        assert_eq!(throughput_due(15, 9.0, Some(&prior)), None, "faster");
        // Exactly 20% worse is not more than 20% worse; a little over is, once a job has landed since.
        assert_eq!(throughput_due(15, 12.0, Some(&prior)), None);
        assert_eq!(throughput_due(15, 12.1, Some(&prior)), Some(Worsened));
        assert_eq!(
            throughput_due(10, 12.1, Some(&prior)),
            None,
            "no new landing"
        );
        // Ten more landings are the cadence whatever the median did.
        assert_eq!(throughput_due(20, 12.1, Some(&prior)), Some(Cadence));
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
    fn a_review_queues_one_design_request_with_the_bottleneck_change_and_saving_and_none_while_it_is_open()
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
        assert_eq!(proposal.priority, "urgent");
        assert_eq!(known.throughput_reviews, vec![first.clone()]);
        assert_eq!(known.steward_runs.len(), 1, "the invocation is recorded");

        // Ten more landings make a second review due while the request is still open: no second request,
        // and the model is not asked again.
        seed_landed(dir.path(), &athena, 10, 10, 20.0);
        let second = athena.throughput_review(&crate::Quiet).unwrap().unwrap();
        assert_eq!(second.proposal, None);
        assert!(second.note.contains("still open"), "{}", second.note);
        let known = records(&athena);
        assert_eq!(open_design_requests(&known).len(), 1, "still one request");
        assert_eq!(known.throughput_reviews.len(), 2);
        assert_eq!(seen.lock().unwrap().len(), 1, "the model was called once");
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
        assert!(review.note.starts_with("no proposal"), "{}", review.note);
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
        assert_eq!(priority_for_minutes(10.0), Priority::Urgent);
        assert_eq!(priority_for_minutes(25.0), Priority::Urgent);
        let rank = |p: Priority| match p {
            Priority::Low => 0,
            Priority::Normal => 1,
            Priority::High => 2,
            Priority::Urgent => 3,
        };
        let series: Vec<i32> = [0.0, 1.9, 2.0, 4.9, 5.0, 9.9, 10.0, 60.0]
            .iter()
            .map(|m| rank(priority_for_minutes(*m)))
            .collect();
        assert!(series.windows(2).all(|w| w[0] <= w[1]), "{series:?}");
        assert!(series.first() < series.last());

        // The queued request carries the priority its prediction set.
        for (saving, level, expected) in [
            (12.0, "urgent", Priority::Urgent),
            (1.0, "low", Priority::Low),
        ] {
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
}
