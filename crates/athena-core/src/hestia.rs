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

use crate::commits::{NewFile, message};
use crate::plan::Priority;
use crate::steward::StewardRequest;
use crate::store::{JobRecords, JobState, Records};
use crate::{Athena, Ui};
use anyhow::{Context, Result, bail};
use athena_data::records::{
    AnyRecord, HestiaCheckResult, IntrospectionRecord, IntrospectionStatus, OutcomeState,
};
use athena_data::{JobId, RecordId};
use jiff::Timestamp;
use serde::Serialize;
use std::collections::BTreeSet;

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

impl Athena {
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
        vec![backfill, derive]
    }

    /// Runs `checks`, commits the report as a record, and returns it. The review passes of
    /// [`Athena::review_passes`] run first and their summary lines go to `ui`.
    pub fn hestia_sweep_with(&self, checks: &[Box<dyn Check>], ui: &dyn Ui) -> Result<Sweep> {
        if self.git.head()?.is_some() {
            for line in self.review_passes(ui) {
                ui.line(&line);
            }
        }
        self.hestia_sweep(checks)
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
            driver: std::sync::Arc::new(FakeAgentDriver::deterministic()),
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
        assert_eq!(ui.0.lock().unwrap().len(), 2, "one line per pass");
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
}
