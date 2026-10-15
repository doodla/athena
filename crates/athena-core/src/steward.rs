//! The steward: a role that runs in the background every few finished tasks, looks at what Athena has done,
//! and queues work items (`docs/design/roles-and-ladder.md`).
//!
//! The steward calls a model. Athena builds it a read-only workspace (the code, a copy of `records/`, and a
//! log of recent commits), starts it through the same driver and tool policy as every other role, and records
//! the invocation (role, usage, cost) beside the work items. The model reads, and reports work items as
//! structured output. It cannot change anything: its Bash is `git` reads and `gh` reads, enforced by the tool
//! policy (`athena_agent::policy`).
//!
//! Athena, not the model, decides what happens to a report. Each item is validated (known sources, existing
//! jobs, stable keys), items whose key is already recorded are dropped, and every remaining item goes through
//! [`decide`]: the requirements ladder (`athena_data::roles::may_change`) queues a request for a rung at or
//! below the role's limit, which then runs as a job under the normal checks and the independent verifier, and
//! only a proposal with its evidence above it.

use crate::Athena;
use crate::capability::{JobConfig, ToolGrant};
use crate::commits::{Committer, NewFile, message};
use crate::state::StateDir;
use crate::store::Records;
use anyhow::{Context, Result, bail};
use athena_agent::env::Side;
use athena_context::outputs::{
    RefKindOutput, RungOutput, StewardItemOutput, StewardKindOutput, StewardOutput, steward_schema,
};
use athena_context::{Composed, steward_system_prompt};
use athena_data::records::{
    AnyRecord, Invocation, InvocationOutcome, OutcomeState, RefKind, Reference, Request, Role,
    WorkItem, WorkItemKind,
};
use athena_data::roles::{Ladder, RoleDefinition, steward};
use athena_data::{JobId, RecordId};
use athena_gate::records::RecordSet;
use athena_git::Git;
use athena_git::snapshot::{Snapshot, SnapshotRepo};
use jiff::Timestamp;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

// ---------------------------------------------------------------------------------------------------------
// Cadence

fn counter_path(state: &StateDir) -> PathBuf {
    state.root().join("steward-count")
}

/// Finished tasks counted since the steward last started.
pub fn finished_since_run(state: &StateDir) -> u32 {
    std::fs::read_to_string(counter_path(state))
        .ok()
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(0)
}

/// Counts one finished task. Returns true, and starts the count again, when `cadence` tasks have finished.
pub fn tick(state: &StateDir, cadence: u32) -> Result<bool> {
    let _lock = state.lock_repository()?;
    let count = finished_since_run(state) + 1;
    let due = count >= cadence;
    std::fs::write(
        counter_path(state),
        format!("{}\n", if due { 0 } else { count }),
    )
    .context("writing the steward's counter")?;
    Ok(due)
}

// ---------------------------------------------------------------------------------------------------------
// Running in the background

static RUNNING: Mutex<Vec<JoinHandle<()>>> = Mutex::new(Vec::new());

/// Runs `work` on its own thread and returns at once.
pub fn spawn_background(work: impl FnOnce() + Send + 'static) {
    let handle = std::thread::spawn(work);
    RUNNING.lock().unwrap().push(handle);
}

/// Waits for every background run to end. A program calls it before it exits, so a run is not cut off;
/// the main flow never calls it.
pub fn wait_for_background() {
    let handles: Vec<JoinHandle<()>> = std::mem::take(&mut *RUNNING.lock().unwrap());
    for handle in handles {
        let _ = handle.join();
    }
}

fn error_path(state: &StateDir) -> PathBuf {
    state.root().join("steward-error.txt")
}

impl Athena {
    /// Called when a job finishes. A landed or rejected job counts toward the steward's cadence, and the
    /// steward starts in the background when its count is reached. This never waits for the steward. A nested
    /// run never starts it: no model call happens outside a job's own records and budget.
    pub(crate) fn note_finished(&self, state: OutcomeState, ui: &dyn crate::Ui) {
        if self.nested.is_some() || !matches!(state, OutcomeState::Landed | OutcomeState::Rejected)
        {
            return;
        }
        let role = steward(crate::now());
        match tick(&self.state, role.cadence_tasks) {
            Ok(false) => {}
            Ok(true) => {
                ui.line(&format!(
                    "steward    started in the background ({} finished tasks)",
                    role.cadence_tasks
                ));
                let git = self.git.clone();
                let state = self.state.clone();
                let driver = Arc::clone(&self.driver);
                let build = self.build.clone();
                let athena_bin = self.athena_bin.clone();
                spawn_background(move || {
                    // The steward needs the repository, the state directory, and the driver; it builds its own
                    // handle on them, because the capabilities of the command that started it stay with it.
                    let athena = Athena {
                        git,
                        state,
                        driver,
                        capabilities: Vec::new(),
                        build,
                        athena_bin,
                        nested: None,
                        pinner: Box::new(crate::pin::GitPinner),
                        jev: None,
                    };
                    let view = GitView { git: &athena.git };
                    let outcome = run(&athena, &view, &RequestQueue, &role, crate::now());
                    let path = error_path(&athena.state);
                    match outcome {
                        Ok(_) => {
                            let _ = std::fs::remove_file(path);
                        }
                        Err(error) => {
                            let _ = std::fs::write(path, format!("{error:#}\n"));
                        }
                    }
                });
            }
            Err(error) => ui.line(&format!("steward    not started: {error:#}")),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------
// What the steward looks at

/// A source the steward reads: a stable name, and where it lies in the steward's workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Source {
    pub name: &'static str,
    pub paths: &'static [&'static str],
}

/// Where the git log goes in the workspace.
pub const GIT_LOG_FILE: &str = "GIT-LOG.txt";

/// How many recent commits the git log file lists.
pub const GIT_LOG_COMMITS: u32 = 200;

/// The sources the steward reads in its workspace. The names are what a blind spot is keyed on, so they never
/// change with a failure's wording.
pub const SOURCES: [Source; 7] = [
    Source {
        name: "jobs",
        paths: &["records/jobs"],
    },
    Source {
        name: "events",
        paths: &[
            "records/interventions",
            "records/restarts",
            "records/host-errors",
            "records/resolutions",
        ],
    },
    Source {
        name: "introspections",
        paths: &["records/introspections"],
    },
    Source {
        name: "class-registry",
        paths: &["records/failure-classes"],
    },
    Source {
        name: "decisions",
        paths: &["records/decision-states", "records/default-expectations"],
    },
    Source {
        name: "git-log",
        paths: &[GIT_LOG_FILE],
    },
    Source {
        name: "code",
        paths: &["."],
    },
];

/// The one source read through `gh` rather than in the workspace.
pub const GITHUB_SOURCE: &str = "github";

/// Every name a blind spot may use for a source.
pub fn source_names() -> Vec<&'static str> {
    SOURCES
        .iter()
        .map(|s| s.name)
        .chain([GITHUB_SOURCE])
        .collect()
}

/// Where the steward reads Athena's state. An error is a blind spot: the steward says so, and goes on.
pub trait View: Sync {
    /// Every record, where it lives.
    fn records(&self) -> Result<Records>;
    /// Writes the read-only workspace into `dir`, which must not exist yet. Returns the sources, by name, that
    /// could not be written.
    fn workspace(&self, dir: &Path) -> Result<Vec<&'static str>>;
}

/// The real view: the records and code at the current commit.
#[derive(Debug)]
pub struct GitView<'a> {
    pub git: &'a Git,
}

impl GitView<'_> {
    fn head(&self) -> Result<String> {
        self.git
            .head()?
            .context("the repository has no commits yet")
    }
}

impl View for GitView<'_> {
    fn records(&self) -> Result<Records> {
        Records::load(self.git, &self.head()?)
    }

    /// The workspace holds the visible files of the commit as a one-commit repository of its own (what the
    /// producer sees: no private paths, no `records/`), a copy of `records/`, and `GIT-LOG.txt`, the subjects
    /// of the last commits. The full object history is not given: private paths stay hidden.
    fn workspace(&self, dir: &Path) -> Result<Vec<&'static str>> {
        let head = self.head()?;
        let mut unreadable = Vec::new();
        if Snapshot::create(self.git, &head, &head, dir, SnapshotRepo::Base).is_err() {
            unreadable.push("code");
            std::fs::create_dir_all(dir)?;
        }
        match RecordSet::load(self.git, &head) {
            Ok(set) => {
                for file in &set.files {
                    let full = dir.join(&file.path);
                    if let Some(parent) = full.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(full, &file.bytes)?;
                }
            }
            Err(_) => unreadable.extend(["jobs", "events", "introspections", "class-registry"]),
        }
        // A source with no records yet is an empty directory, not a missing one.
        for path in SOURCES.iter().flat_map(|s| s.paths) {
            if path.starts_with("records/") {
                std::fs::create_dir_all(dir.join(path))?;
            }
        }
        let log = self.git.run([
            "log",
            "-n",
            &GIT_LOG_COMMITS.to_string(),
            "--date=short",
            "--format=%h %ad %s",
            &head,
        ]);
        match log {
            Ok(text) => std::fs::write(dir.join(GIT_LOG_FILE), text)?,
            Err(_) => unreadable.push("git-log"),
        }
        unreadable.dedup();
        Ok(unreadable)
    }
}

/// The path that names the records as a source.
pub const RECORDS_SOURCE: &str = "records/";

// ---------------------------------------------------------------------------------------------------------
// Changes the steward proposes

/// A change the steward wants to make to one rung of the ladder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposedChange {
    pub kind: WorkItemKind,
    /// Stable for the same finding, so the steward does not raise it twice.
    pub key: String,
    pub title: String,
    pub rung: Ladder,
    pub source: Reference,
    pub evidence: Vec<Reference>,
    /// The request queued when the steward may change this rung itself.
    pub request: String,
}

/// A request queued to apply a change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedRequest {
    pub job: JobId,
    pub file: NewFile,
}

/// How a change is applied. It never edits files: it queues a request, which runs as a job under Athena's
/// checks and the independent verifier.
pub trait Applier {
    fn apply(&self, change: &ProposedChange, taken: &BTreeSet<String>) -> Result<QueuedRequest>;
}

/// The real applier: a queued request, as `athena ask --later` writes one.
#[derive(Debug)]
pub struct RequestQueue;

impl Applier for RequestQueue {
    fn apply(&self, change: &ProposedChange, taken: &BTreeSet<String>) -> Result<QueuedRequest> {
        let mut id = RecordId::generate(crate::now());
        while taken.contains(id.as_str()) {
            id = RecordId::generate(crate::now());
        }
        let job = JobId::Id(id);
        let request = Request {
            job: job.clone(),
            text: change.request.trim().to_string(),
            received_at: crate::now(),
            channel: "steward".into(),
        };
        Ok(QueuedRequest {
            job,
            file: NewFile::record(&AnyRecord::Request(request)),
        })
    }
}

/// What became of a proposed change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// The role may change the rung itself: a request was queued for it.
    Applied(QueuedRequest),
    /// The rung is above the role's limit: only the work item was queued.
    Proposed,
}

/// Decides a proposed change by the ladder. The work item is returned in both cases, so the finding is
/// recorded once; an applied change cites the queued request in its evidence.
pub fn decide(
    role: &RoleDefinition,
    change: &ProposedChange,
    applier: &dyn Applier,
    taken: &BTreeSet<String>,
    now: Timestamp,
) -> Result<(Decision, WorkItem)> {
    let mut evidence = change.evidence.clone();
    let decision = if role.may_change(change.rung) {
        let queued = applier.apply(change, taken)?;
        evidence.push(Reference::job(&queued.job));
        Decision::Applied(queued)
    } else {
        Decision::Proposed
    };
    let item = WorkItem {
        id: RecordId::generate(now),
        at: now,
        role: role.name.clone(),
        kind: change.kind,
        key: change.key.clone(),
        title: change.title.clone(),
        rung: change.rung,
        source: change.source.clone(),
        evidence,
    };
    Ok((decision, item))
}

// ---------------------------------------------------------------------------------------------------------
// Findings

/// The key of the blind spot for a source. It is made from the source's name alone, never from the text of
/// an error, so the same failure on the next run has the same key and is not queued again.
pub fn blind_spot_key(source: &str) -> String {
    format!("blind-spot:{source}")
}

/// The change for a source that could not be read. It is on the lowest rung, so the role queues a request to
/// find out why and make the source readable.
pub fn blind_spot_change(source: &str, title: &str) -> ProposedChange {
    ProposedChange {
        kind: WorkItemKind::BlindSpot,
        key: blind_spot_key(source),
        title: title.to_string(),
        rung: Ladder::Implementation,
        source: Reference::path(source),
        evidence: Vec::new(),
        request: format!(
            "The steward could not read {source}: {title}. Find out why, and make it readable, or record why \
             it cannot be."
        ),
    }
}

/// Jobs that ended rejected or failed, grouped by class of work. Only groups of two or more repeat. The
/// steward is given this count as an input; it decides what the repeats mean.
pub fn repeated_failures(records: &Records) -> BTreeMap<String, Vec<JobId>> {
    let mut by_class: BTreeMap<String, Vec<JobId>> = BTreeMap::new();
    for job in records.request_jobs() {
        let ended_badly = job
            .outcome
            .as_ref()
            .is_some_and(|o| matches!(o.state, OutcomeState::Rejected | OutcomeState::Failed));
        let class = job.job.as_ref().and_then(|j| j.class_of_work.clone());
        if let (true, Some(class)) = (ended_badly, class) {
            by_class.entry(class).or_default().push(job.id().clone());
        }
    }
    by_class.retain(|_, jobs| jobs.len() >= 2);
    for jobs in by_class.values_mut() {
        jobs.sort();
    }
    by_class
}

// ---------------------------------------------------------------------------------------------------------
// The model's report, validated

/// The most work items one run may queue.
pub const MAX_ITEMS: usize = 20;

const MAX_TITLE: usize = 200;
const MAX_KEY: usize = 120;
const MAX_REQUEST: usize = 4000;

fn ladder(rung: RungOutput) -> Ladder {
    match rung {
        RungOutput::OwnerGoals => Ladder::OwnerGoals,
        RungOutput::BriefRequirements => Ladder::BriefRequirements,
        RungOutput::DesignDecisions => Ladder::DesignDecisions,
        RungOutput::DefaultsConventions => Ladder::DefaultsConventions,
        RungOutput::Implementation => Ladder::Implementation,
    }
}

/// Checks a reference the model gave: a job must exist, a commit must look like one, a path must stay inside
/// the workspace.
fn check_reference(
    kind: RefKindOutput,
    value: &str,
    records: &Records,
) -> Result<Reference, String> {
    let value = value.trim();
    match kind {
        RefKindOutput::Job => match JobId::parse(value) {
            Ok(id) if records.jobs.contains_key(id.as_str()) => Ok(Reference::job(&id)),
            _ => Err(format!("{value:?} is not a job in the records")),
        },
        RefKindOutput::Commit => {
            if (7..=40).contains(&value.len())
                && value
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                Ok(Reference::commit(value))
            } else {
                Err(format!("{value:?} is not a commit id"))
            }
        }
        RefKindOutput::Path => {
            if value.is_empty()
                || value.starts_with('/')
                || value.starts_with('~')
                || value.contains('\n')
                || value.split('/').any(|part| part == "..")
            {
                Err(format!("{value:?} is not a path inside the workspace"))
            } else {
                Ok(Reference::path(value))
            }
        }
    }
}

fn check_key(key: &str) -> Result<String, String> {
    let key = key.trim();
    if key.is_empty() || key.len() > MAX_KEY {
        return Err(format!("a key is 1 to {MAX_KEY} characters"));
    }
    if !key
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b":_./,-".contains(&b))
    {
        return Err(format!(
            "{key:?}: a key has letters, digits, and : _ . / , - only"
        ));
    }
    if key.starts_with("blind-spot:") {
        return Err(format!(
            "{key:?}: blind-spot keys are made by Athena from the source's name"
        ));
    }
    Ok(key.to_string())
}

/// What validation made of a report.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Validated {
    pub changes: Vec<ProposedChange>,
    /// Why each item that was dropped was dropped.
    pub rejected: Vec<String>,
}

/// Validates the steward's items against the records and the role's limit. An invalid item is dropped with a
/// reason and does not stop the others. A blind spot's key and rung are Athena's: the key is made from the
/// source's name, so it does not depend on the model's wording.
pub fn validate_items(
    items: Vec<StewardItemOutput>,
    role: &RoleDefinition,
    records: &Records,
) -> Validated {
    let mut out = Validated::default();
    for (n, item) in items.into_iter().enumerate() {
        if n >= MAX_ITEMS {
            out.rejected.push(format!(
                "item {}: a run queues at most {MAX_ITEMS} items",
                n + 1
            ));
            continue;
        }
        match validate_item(item, role, records) {
            Ok(change) => out.changes.push(change),
            Err(reason) => out.rejected.push(format!("item {}: {reason}", n + 1)),
        }
    }
    out
}

fn validate_item(
    item: StewardItemOutput,
    role: &RoleDefinition,
    records: &Records,
) -> Result<ProposedChange, String> {
    let title = item.title.trim().to_string();
    if title.is_empty() || title.contains('\n') || title.chars().count() > MAX_TITLE {
        return Err(format!(
            "a title is one line of 1 to {MAX_TITLE} characters"
        ));
    }
    if item.kind == StewardKindOutput::BlindSpot {
        let source = item.source.value.trim();
        if !source_names().contains(&source) {
            return Err(format!(
                "{source:?} is not a source; the sources are {}",
                source_names().join(", ")
            ));
        }
        return Ok(blind_spot_change(source, &title));
    }
    let key = check_key(&item.key)?;
    if item.evidence.is_empty() {
        return Err(format!(
            "{key}: an item that is not a blind spot cites its evidence"
        ));
    }
    let source = check_reference(item.source.kind, &item.source.value, records)
        .map_err(|e| format!("{key}: source: {e}"))?;
    let evidence = item
        .evidence
        .iter()
        .map(|e| check_reference(e.kind, &e.value, records))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("{key}: evidence: {e}"))?;
    let rung = ladder(item.rung);
    let request = item.request.unwrap_or_default().trim().to_string();
    if role.may_change(rung) && request.is_empty() {
        return Err(format!(
            "{key}: rung {} is at or below your limit, so the item needs a request to queue",
            rung.as_str()
        ));
    }
    if request.chars().count() > MAX_REQUEST {
        return Err(format!(
            "{key}: a request is at most {MAX_REQUEST} characters"
        ));
    }
    Ok(ProposedChange {
        kind: match item.kind {
            StewardKindOutput::RepeatedFailure => WorkItemKind::RepeatedFailure,
            _ => WorkItemKind::Proposal,
        },
        key,
        title,
        rung,
        source,
        evidence,
        request,
    })
}

// ---------------------------------------------------------------------------------------------------------
// The brief

/// The steward's user prompt: what is in its workspace, what Athena counted, and what is already queued.
pub fn brief(
    role: &RoleDefinition,
    readable: &[&str],
    unreadable: &[&str],
    failures: &BTreeMap<String, Vec<JobId>>,
    known_keys: &BTreeSet<String>,
) -> String {
    let mut text = String::from("# Steward run\n\n");
    text.push_str(
        "You work read-only in the current directory, which holds Athena's code, a copy of `records/`, and \
         `GIT-LOG.txt` (recent commit subjects; the full history is not available). Inspect it with Read, \
         Grep, Glob, `git log|show|diff|status`, and `ls`. Read GitHub with `gh run list|view`, \
         `gh pr list|view`, and `gh api <endpoint>` (GET only). Nothing else runs.\n\n",
    );
    text.push_str(&format!(
        "## Your limit\n\nYou may change rungs at or below `{}` yourself: for those, give a `request` that \
         says what to do, and Athena queues it as a job. Above that, Athena queues only a proposal with your \
         evidence. The top two rungs are never changed by a role.\n\n",
        role.highest_rung.as_str()
    ));
    text.push_str("## Sources\n\n");
    for source in SOURCES.iter().filter(|s| readable.contains(&s.name)) {
        text.push_str(&format!("- {}: {}\n", source.name, source.paths.join(", ")));
    }
    text.push_str(&format!(
        "\n## GitHub\n\nThe source `{GITHUB_SOURCE}` is CI runs, pull requests, and deploys, read with `gh`. If \
         `gh` fails or has no access (no `GH_TOKEN` or `GITHUB_TOKEN` set), report a blind spot with source `{GITHUB_SOURCE}`.\n"
    ));
    if !unreadable.is_empty() {
        text.push_str(&format!(
            "\nAthena could not write these sources and has already reported them: {}.\n",
            unreadable.join(", ")
        ));
    }
    text.push_str(
        "\nFor any other source you cannot read, report a blind spot whose `source.value` is that source's \
         name. Its key is made from the name.\n\n",
    );
    text.push_str("## Counted by Athena (an input, not a finding)\n\n");
    if failures.is_empty() {
        text.push_str("No class of work has two or more rejected or failed jobs.\n");
    }
    for (class, jobs) in failures {
        let ids: Vec<&str> = jobs.iter().map(JobId::as_str).collect();
        text.push_str(&format!(
            "- class `{class}`: {} jobs ended rejected or failed: {}\n",
            jobs.len(),
            ids.join(", ")
        ));
    }
    text.push_str("\n## Already queued (do not report these again)\n\n");
    if known_keys.is_empty() {
        text.push_str("Nothing yet.\n");
    }
    for key in known_keys.iter().rev().take(200) {
        text.push_str(&format!("- {key}\n"));
    }
    text.push_str(
        "\n## Your report\n\nReturn `items`: at most 20, empty when there is nothing to report. Each cites \
         its source and evidence by reference (a job id that exists, a path in the workspace, or a commit \
         id); copy no data. A key is stable for the same finding.\n",
    );
    text
}

// ---------------------------------------------------------------------------------------------------------
// Recording the invocation

fn pending_dir(state: &StateDir) -> PathBuf {
    state.root().join("steward-pending")
}

/// Keeps an invocation's record until a commit carries it. A run that queues nothing makes no commit, so
/// its record waits for the next one.
fn stash_invocation(state: &StateDir, record: &Invocation) -> Result<()> {
    let dir = pending_dir(state);
    std::fs::create_dir_all(&dir)?;
    let any = AnyRecord::Invocation(Box::new(record.clone()));
    std::fs::write(
        dir.join(format!("{}.json", record.job.as_str())),
        any.to_json(),
    )
    .context("keeping the steward's invocation record")
}

/// The invocation records waiting for a commit, with the files that hold them.
fn pending_invocations(state: &StateDir) -> Vec<(PathBuf, NewFile)> {
    let Ok(entries) = std::fs::read_dir(pending_dir(state)) else {
        return Vec::new();
    };
    let mut found: Vec<(PathBuf, NewFile)> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let bytes = std::fs::read(entry.path()).ok()?;
            let record: AnyRecord = serde_json::from_slice(&bytes).ok()?;
            matches!(&record, AnyRecord::Invocation(i) if i.role == Role::Steward)
                .then(|| (entry.path(), NewFile::record(&record)))
        })
        .collect();
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
}

/// Removes a run's directory in the state directory when the run ends, however it ends.
struct Cleanup(PathBuf);

impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// ---------------------------------------------------------------------------------------------------------
// A run

/// What a run did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    /// Work items queued, by key.
    pub queued: Vec<String>,
    /// Requests queued to apply a change the role may make itself.
    pub applied: Vec<JobId>,
    /// Items of the model's report that failed validation, with the reason.
    pub rejected: Vec<String>,
}

/// What the model said, validated.
struct Consulted {
    changes: Vec<ProposedChange>,
    rejected: Vec<String>,
}

/// Runs the model once over a fresh workspace, records the invocation, and validates its report.
fn consult(
    athena: &Athena,
    view: &dyn View,
    role: &RoleDefinition,
    records: &Records,
    now: Timestamp,
) -> Result<(Consulted, Vec<&'static str>)> {
    let head = athena.head()?;
    let jc = JobConfig::at(&athena.git, &head)?;
    // The run is no job, but it has an id of its own: its scratch directory and its invocation record are
    // named by it.
    let job = JobId::Id(RecordId::generate(now));
    let job_dir = athena.state.job_dir(&job);
    let _cleanup = Cleanup(job_dir.clone());
    let workspace = job_dir.join("workspace");
    let unreadable = view.workspace(&workspace)?;
    let readable: Vec<&str> = SOURCES
        .iter()
        .map(|s| s.name)
        .filter(|name| !unreadable.contains(name))
        .collect();
    let known_keys: BTreeSet<String> = records.work_items.iter().map(|w| w.key.clone()).collect();
    let user_prompt = brief(
        role,
        &readable,
        &unreadable,
        &repeated_failures(records),
        &known_keys,
    );
    let composed: Composed = steward_system_prompt(&role.prompt, role.version);
    let grant = ToolGrant {
        tools: vec!["Read".into(), "Grep".into(), "Glob".into(), "Bash".into()],
        commands: BTreeMap::from([
            (
                "git".to_string(),
                ["status", "diff", "log", "show"].map(String::from).to_vec(),
            ),
            (
                "gh".to_string(),
                ["run", "pr", "api"].map(String::from).to_vec(),
            ),
            ("ls".to_string(), vec!["*".to_string()]),
        ]),
        writes: false,
        permission_mode: "dontAsk".into(),
    };
    // The steward borrows the verifier's model and budget (docs/design/roles-and-ladder.md).
    let spec = athena.agent_spec(
        &jc,
        &job,
        1,
        Role::Steward,
        &jc.config.models.verifier,
        &composed,
        user_prompt,
        &grant,
        &workspace,
        Side::Read,
        &BTreeMap::new(),
        jc.config.budget.verifier_usd,
        jc.config.budget.verifier_timeout_s,
        steward_schema(),
    )?;
    let facts = crate::invocation::InvocationFacts {
        job: &job,
        spec: &spec,
        composed: &composed,
        config: &jc.config,
        fake: athena.is_fake(),
    };
    let redactor = athena.redactor(&jc.config, Some(&job))?;
    let (output, record) = match athena.invoke_and_record(&facts, &redactor)? {
        Ok(done) => done,
        Err(error) => bail!("the steward could not run: {error}"),
    };
    // The invocation cost money whatever the model said, so its record is kept before anything else.
    stash_invocation(&athena.state, &record)?;
    if output.outcome != InvocationOutcome::Success {
        bail!(
            "the steward did not complete: {}",
            record.detail.as_deref().unwrap_or("no detail")
        );
    }
    let report: StewardOutput = serde_json::from_value(output.structured.unwrap_or_default())
        .context("the steward returned no valid report")?;
    let mut validated = validate_items(report.items, role, records);
    // What the model wrote goes into records, so it is redacted first.
    let mut text: Vec<String> = validated
        .changes
        .iter()
        .flat_map(|c| [c.title.clone(), c.request.clone()])
        .collect();
    redactor.redact_all(&mut text)?;
    let mut text = text.into_iter();
    for change in validated.changes.iter_mut() {
        change.title = text.next().unwrap_or_default();
        change.request = text.next().unwrap_or_default();
    }
    Ok((
        Consulted {
            changes: validated.changes,
            rejected: validated.rejected,
        },
        unreadable,
    ))
}

/// One steward run: reads through `view`, asks the model, queues work items through [`decide`], and commits
/// them with the role definition (when it is not yet recorded) and the invocation records in one records
/// commit. Does nothing, and commits nothing, when there is nothing to queue. A nested run makes no model
/// call and does nothing.
pub fn run(
    athena: &Athena,
    view: &dyn View,
    applier: &dyn Applier,
    role: &RoleDefinition,
    now: Timestamp,
) -> Result<Report> {
    if athena.nested.is_some() {
        return Ok(Report::default());
    }
    let mut candidates: Vec<ProposedChange> = Vec::new();
    let mut rejected: Vec<String> = Vec::new();
    let mut known_keys: BTreeSet<String> = BTreeSet::new();
    let mut taken_jobs: BTreeSet<String> = BTreeSet::new();
    let mut role_recorded = false;

    match view.records() {
        Ok(records) => {
            known_keys = records.work_items.iter().map(|w| w.key.clone()).collect();
            taken_jobs = records.jobs.keys().cloned().collect();
            taken_jobs.extend(
                records
                    .steward_runs
                    .iter()
                    .map(|i| i.job.as_str().to_string()),
            );
            role_recorded = records
                .roles
                .iter()
                .any(|r| r.name == role.name && r.version == role.version);
            for path in &records.invalid {
                candidates.push(blind_spot_change(
                    path,
                    "A record file could not be read as a record",
                ));
            }
            let (consulted, unreadable) = consult(athena, view, role, &records, now)?;
            for source in unreadable {
                candidates.push(blind_spot_change(
                    source,
                    "Athena could not write it to the steward's workspace",
                ));
            }
            candidates.extend(consulted.changes);
            rejected = consulted.rejected;
        }
        Err(_) => candidates.push(blind_spot_change(
            RECORDS_SOURCE,
            "The records could not be read",
        )),
    }

    let mut items: Vec<WorkItem> = Vec::new();
    let mut requests: Vec<QueuedRequest> = Vec::new();
    let mut seen = known_keys;
    for change in candidates {
        if !seen.insert(change.key.clone()) {
            continue;
        }
        let (decision, item) = decide(role, &change, applier, &taken_jobs, now)?;
        item.validate()
            .map_err(|reason| anyhow::anyhow!("work item {}: {reason}", item.key))?;
        if let Decision::Applied(queued) = decision {
            taken_jobs.insert(queued.job.as_str().to_string());
            requests.push(queued);
        }
        items.push(item);
    }
    if items.is_empty() {
        return Ok(Report {
            rejected,
            ..Report::default()
        });
    }

    let mut files: Vec<NewFile> = Vec::new();
    let mut trailers: Vec<(&str, String)> = Vec::new();
    if !role_recorded {
        let record = AnyRecord::Role(Box::new(role.clone()));
        trailers.push(("Athena-Record", record.canonical_path()));
        files.push(NewFile::record(&record));
    }
    for item in &items {
        let record = AnyRecord::WorkItem(Box::new(item.clone()));
        trailers.push(("Athena-Record", record.canonical_path()));
        files.push(NewFile::record(&record));
    }
    for queued in &requests {
        trailers.push(("Athena-Job", queued.job.as_str().to_string()));
        files.push(queued.file.clone());
    }
    let waiting = pending_invocations(&athena.state);
    for (_, file) in &waiting {
        trailers.push(("Athena-Record", file.path.clone()));
        files.push(file.clone());
    }
    let subject = format!(
        "Steward: {} work item(s), {} request(s) queued",
        items.len(),
        requests.len()
    );
    let committer = Committer {
        git: &athena.git,
        state: &athena.state,
    };
    committer.commit_records(
        &files,
        &message(
            &subject,
            "Work items from the steward's model-backed look at everything Athena has done. Each cites its \
             source and evidence by reference.",
            &trailers,
        ),
    )?;
    for (path, _) in waiting {
        let _ = std::fs::remove_file(path);
    }
    Ok(Report {
        queued: items.iter().map(|i| i.key.clone()).collect(),
        applied: requests.into_iter().map(|q| q.job).collect(),
        rejected,
    })
}

/// True when a reference is a job.
pub fn is_job(reference: &Reference) -> bool {
    reference.kind == RefKind::Job
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_context::outputs::StewardRefOutput;
    use std::cell::RefCell;

    fn at() -> Timestamp {
        "2026-10-14T12:00:00Z".parse().unwrap()
    }

    fn change(rung: Ladder) -> ProposedChange {
        ProposedChange {
            kind: WorkItemKind::Proposal,
            key: format!("proposal:{}", rung.as_str()),
            title: "A change".into(),
            rung,
            source: Reference::path("records/jobs"),
            evidence: vec![Reference::job(
                &JobId::parse("20261010-153012-k7q2").unwrap(),
            )],
            request: "Make the change.".into(),
        }
    }

    /// An applier that counts its calls and writes nothing.
    #[derive(Default)]
    struct Counting(RefCell<u32>);

    impl Applier for Counting {
        fn apply(&self, _: &ProposedChange, _: &BTreeSet<String>) -> Result<QueuedRequest> {
            *self.0.borrow_mut() += 1;
            Ok(QueuedRequest {
                job: JobId::parse("20261014-120000-aaaa").unwrap(),
                file: NewFile::text("records/x", ""),
            })
        }
    }

    #[test]
    fn a_change_above_the_limit_is_queued_with_evidence_and_not_applied() {
        let role = steward(at());
        for rung in [
            Ladder::OwnerGoals,
            Ladder::BriefRequirements,
            Ladder::DesignDecisions,
        ] {
            let applier = Counting::default();
            let (decision, item) =
                decide(&role, &change(rung), &applier, &BTreeSet::new(), at()).unwrap();
            assert_eq!(decision, Decision::Proposed, "{rung:?}");
            assert_eq!(*applier.0.borrow(), 0, "{rung:?} is not applied");
            assert_eq!(item.rung, rung);
            assert_eq!(item.evidence.len(), 1, "the evidence is cited");
            assert!(item.validate().is_ok());
        }
    }

    #[test]
    fn a_change_on_a_lower_rung_is_applied() {
        let role = steward(at());
        for rung in [Ladder::DefaultsConventions, Ladder::Implementation] {
            let applier = Counting::default();
            let (decision, item) =
                decide(&role, &change(rung), &applier, &BTreeSet::new(), at()).unwrap();
            assert!(matches!(decision, Decision::Applied(_)), "{rung:?}");
            assert_eq!(*applier.0.borrow(), 1);
            assert!(
                item.evidence.iter().any(is_job),
                "the item cites the queued request"
            );
        }
    }

    #[test]
    fn a_role_with_a_higher_limit_still_cannot_change_the_top_two_rungs() {
        let mut role = steward(at());
        role.highest_rung = Ladder::OwnerGoals;
        let applier = Counting::default();
        let (decision, _) = decide(
            &role,
            &change(Ladder::BriefRequirements),
            &applier,
            &BTreeSet::new(),
            at(),
        )
        .unwrap();
        assert_eq!(decision, Decision::Proposed);
        assert_eq!(*applier.0.borrow(), 0);
    }

    #[test]
    fn a_blind_spot_names_its_source_and_its_key_ignores_the_failure_text() {
        let role = steward(at());
        let a = blind_spot_change("github", "gh: HTTP 401 at 12:00:01");
        let b = blind_spot_change("github", "gh: HTTP 502 at 12:07:44, try again");
        assert_eq!(a.key, "blind-spot:github");
        assert_eq!(a.key, b.key, "the key does not carry error text");
        assert_ne!(a.title, b.title);
        let (decision, item) =
            decide(&role, &a, &Counting::default(), &BTreeSet::new(), at()).unwrap();
        assert!(matches!(decision, Decision::Applied(_)));
        assert_eq!(item.kind, WorkItemKind::BlindSpot);
        assert_eq!(item.source, Reference::path("github"));
        assert!(item.validate().is_ok());
    }

    fn records_with_job(id: &str) -> Records {
        let mut records = Records::default();
        records.jobs.insert(
            id.to_string(),
            crate::store::JobRecords {
                id: Some(JobId::parse(id).unwrap()),
                ..Default::default()
            },
        );
        records
    }

    fn item(kind: StewardKindOutput, key: &str, rung: RungOutput) -> StewardItemOutput {
        StewardItemOutput {
            kind,
            key: key.into(),
            title: "Two jobs failed the same way".into(),
            rung,
            source: StewardRefOutput {
                kind: RefKindOutput::Path,
                value: "records/jobs".into(),
            },
            evidence: vec![StewardRefOutput {
                kind: RefKindOutput::Job,
                value: "20261010-153012-k7q2".into(),
            }],
            request: Some("Add the convention.".into()),
        }
    }

    #[test]
    fn a_valid_item_becomes_a_proposed_change() {
        let role = steward(at());
        let records = records_with_job("20261010-153012-k7q2");
        let valid = validate_items(
            vec![item(
                StewardKindOutput::RepeatedFailure,
                "repeat:docs",
                RungOutput::DefaultsConventions,
            )],
            &role,
            &records,
        );
        assert!(valid.rejected.is_empty(), "{:?}", valid.rejected);
        let change = &valid.changes[0];
        assert_eq!(change.kind, WorkItemKind::RepeatedFailure);
        assert_eq!(change.rung, Ladder::DefaultsConventions);
        assert_eq!(change.request, "Add the convention.");
        assert_eq!(
            change.evidence,
            vec![Reference::job(
                &JobId::parse("20261010-153012-k7q2").unwrap()
            )]
        );
    }

    #[test]
    fn invalid_items_are_dropped_with_a_reason_and_do_not_stop_the_rest() {
        let role = steward(at());
        let records = records_with_job("20261010-153012-k7q2");
        let mut no_evidence = item(
            StewardKindOutput::Proposal,
            "a",
            RungOutput::DesignDecisions,
        );
        no_evidence.evidence.clear();
        let mut unknown_job = item(
            StewardKindOutput::Proposal,
            "b",
            RungOutput::DesignDecisions,
        );
        unknown_job.evidence[0].value = "20200101-000000-zzzz".into();
        let mut bad_path = item(
            StewardKindOutput::Proposal,
            "c",
            RungOutput::DesignDecisions,
        );
        bad_path.source.value = "../outside".into();
        let mut bad_commit = item(
            StewardKindOutput::Proposal,
            "d",
            RungOutput::DesignDecisions,
        );
        bad_commit.evidence[0] = StewardRefOutput {
            kind: RefKindOutput::Commit,
            value: "not-a-sha".into(),
        };
        let mut no_request = item(
            StewardKindOutput::RepeatedFailure,
            "e",
            RungOutput::Implementation,
        );
        no_request.request = None;
        let reserved = item(
            StewardKindOutput::Proposal,
            "blind-spot:github",
            RungOutput::DesignDecisions,
        );
        let spaced = item(
            StewardKindOutput::Proposal,
            "has space",
            RungOutput::DesignDecisions,
        );
        let mut multiline = item(
            StewardKindOutput::Proposal,
            "f",
            RungOutput::DesignDecisions,
        );
        multiline.title = "one\ntwo".into();
        let good = item(
            StewardKindOutput::Proposal,
            "good",
            RungOutput::DesignDecisions,
        );
        let valid = validate_items(
            vec![
                no_evidence,
                unknown_job,
                bad_path,
                bad_commit,
                no_request,
                reserved,
                spaced,
                multiline,
                good,
            ],
            &role,
            &records,
        );
        assert_eq!(valid.rejected.len(), 8, "{:#?}", valid.rejected);
        assert_eq!(valid.changes.len(), 1);
        assert_eq!(valid.changes[0].key, "good");
        // A proposal above the limit needs no request.
        let mut proposal = item(
            StewardKindOutput::Proposal,
            "p",
            RungOutput::DesignDecisions,
        );
        proposal.request = None;
        assert_eq!(
            validate_items(vec![proposal], &role, &records)
                .changes
                .len(),
            1
        );
    }

    #[test]
    fn a_blind_spot_must_name_a_known_source_and_its_key_is_made_from_the_name() {
        let role = steward(at());
        let records = Records::default();
        let mut spot = item(
            StewardKindOutput::BlindSpot,
            "whatever the model wrote",
            RungOutput::OwnerGoals,
        );
        spot.source = StewardRefOutput {
            kind: RefKindOutput::Path,
            value: "github".into(),
        };
        spot.evidence.clear();
        spot.title = "gh said: HTTP 401 at 12:00:01".into();
        let valid = validate_items(vec![spot.clone()], &role, &records);
        let change = &valid.changes[0];
        assert_eq!(change.key, "blind-spot:github");
        assert_eq!(change.rung, Ladder::Implementation, "the rung is Athena's");
        assert_eq!(change.source, Reference::path("github"));
        spot.source.value = "the moon".into();
        let invalid = validate_items(vec![spot], &role, &records);
        assert!(invalid.changes.is_empty());
        assert_eq!(invalid.rejected.len(), 1);
    }

    #[test]
    fn a_run_queues_at_most_twenty_items() {
        let role = steward(at());
        let records = records_with_job("20261010-153012-k7q2");
        let items: Vec<StewardItemOutput> = (0..25)
            .map(|n| {
                item(
                    StewardKindOutput::Proposal,
                    &format!("k{n}"),
                    RungOutput::DesignDecisions,
                )
            })
            .collect();
        let valid = validate_items(items, &role, &records);
        assert_eq!(valid.changes.len(), MAX_ITEMS);
        assert_eq!(valid.rejected.len(), 5);
    }

    #[test]
    fn the_brief_names_the_sources_the_failures_and_the_queued_keys() {
        let role = steward(at());
        let failures = BTreeMap::from([(
            "docs".to_string(),
            vec![
                JobId::parse("20261010-153012-k7q2").unwrap(),
                JobId::parse("20261011-153012-k7q3").unwrap(),
            ],
        )]);
        let known = BTreeSet::from(["blind-spot:github".to_string()]);
        let names: Vec<&str> = SOURCES.iter().map(|s| s.name).collect();
        let text = brief(&role, &names, &["git-log"], &failures, &known);
        for name in [
            "jobs",
            "events",
            "introspections",
            "class-registry",
            "decisions",
        ] {
            assert!(text.contains(&format!("- {name}: records/")), "{name}");
        }
        assert!(text.contains("- git-log: GIT-LOG.txt"));
        assert!(text.contains("class `docs`: 2 jobs"));
        assert!(text.contains("- blind-spot:github"));
        assert!(
            text.contains("could not write these sources and has already reported them: git-log")
        );
        assert!(text.contains("`defaults-conventions`"));
        let readable: Vec<&str> = names.iter().copied().filter(|n| *n != "git-log").collect();
        let without = brief(
            &role,
            &readable,
            &["git-log"],
            &BTreeMap::new(),
            &BTreeSet::new(),
        );
        assert!(!without.contains("- git-log: GIT-LOG.txt"));
        assert!(without.contains("No class of work"));
    }

    #[test]
    fn the_counter_fires_on_the_fifth_task_and_not_the_fourth() {
        let dir = tempfile::tempdir().unwrap();
        let state = StateDir::new(dir.path().join("repo"));
        for n in 1..=4 {
            assert!(!tick(&state, 5).unwrap(), "task {n}");
            assert_eq!(finished_since_run(&state), n);
        }
        assert!(tick(&state, 5).unwrap(), "the fifth");
        assert_eq!(finished_since_run(&state), 0, "and the count starts again");
        assert!(!tick(&state, 5).unwrap());
    }

    #[test]
    fn a_background_run_does_not_block_the_caller() {
        let (release, wait) = std::sync::mpsc::channel::<()>();
        let (done, finished) = std::sync::mpsc::channel::<()>();
        spawn_background(move || {
            wait.recv().unwrap();
            done.send(()).unwrap();
        });
        // We are here while the run is still waiting: spawning did not block.
        assert!(finished.try_recv().is_err());
        release.send(()).unwrap();
        finished.recv().unwrap();
        wait_for_background();
    }
}
