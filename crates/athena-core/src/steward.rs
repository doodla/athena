//! The steward: a role that runs in the background every few finished tasks, looks at what Athena has done,
//! and queues work items (`docs/design/roles-and-ladder.md`).
//!
//! It reads Athena's own state where it lives, through a [`View`], and records only what would otherwise be
//! lost: work items that cite their source and evidence by reference, and the role's own definition. For each
//! thing it could not read, it queues a blind-spot item that names the source.
//!
//! What it may change itself is decided by the requirements ladder (`athena_data::roles::may_change`): a
//! proposed change on a rung at or below the role's limit is applied, by queueing a request that goes through
//! the normal checks and the independent verifier; a change above the limit is queued as a proposal with its
//! evidence and not applied.

use crate::commits::{Committer, NewFile, message};
use crate::state::StateDir;
use crate::store::Records;
use crate::{Athena, Ui};
use anyhow::{Context, Result};
use athena_data::records::{
    AnyRecord, OutcomeState, RefKind, Reference, Request, WorkItem, WorkItemKind,
};
use athena_data::roles::{Ladder, RoleDefinition, steward};
use athena_data::{JobId, RecordId};
use athena_git::Git;
use jiff::Timestamp;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Mutex;
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
    /// steward starts in the background when its count is reached. This never waits for the steward.
    pub(crate) fn note_finished(&self, state: OutcomeState, ui: &dyn Ui) {
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
                spawn_background(move || {
                    let committer = Committer {
                        git: &git,
                        state: &state,
                    };
                    let view = GitView { git: &git };
                    let outcome = run(&committer, &view, &RequestQueue, &role, crate::now());
                    let path = error_path(&state);
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

/// Where the steward reads Athena's state. An error is a blind spot: the steward says so, and goes on.
pub trait View: Sync {
    /// Every record, where it lives.
    fn records(&self) -> Result<Records>;
}

/// The real view: the records at the current commit.
#[derive(Debug)]
pub struct GitView<'a> {
    pub git: &'a Git,
}

impl View for GitView<'_> {
    fn records(&self) -> Result<Records> {
        let head = self
            .git
            .head()?
            .context("the repository has no commits yet")?;
        Records::load(self.git, &head)
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

/// The blind spot for a source that could not be read.
pub fn blind_spot(role: &RoleDefinition, source: &str, title: &str, now: Timestamp) -> WorkItem {
    WorkItem {
        id: RecordId::generate(now),
        at: now,
        role: role.name.clone(),
        kind: WorkItemKind::BlindSpot,
        key: format!("blind-spot:{source}"),
        title: title.to_string(),
        rung: Ladder::Implementation,
        source: Reference::path(source),
        evidence: Vec::new(),
    }
}

/// Jobs that ended rejected or failed, grouped by class of work. Only groups of two or more repeat.
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

/// The change the steward proposes for a class that keeps failing: a convention for that class of work, a
/// default or convention being the rung where a repeat is prevented.
fn change_for(class: &str, jobs: &[JobId]) -> ProposedChange {
    let ids: Vec<&str> = jobs.iter().map(JobId::as_str).collect();
    ProposedChange {
        kind: WorkItemKind::RepeatedFailure,
        key: format!("repeated-failure:{class}:{}", ids.join(",")),
        title: format!(
            "{} jobs of class {class} ended rejected or failed",
            jobs.len()
        ),
        rung: Ladder::DefaultsConventions,
        source: Reference::path("records/jobs"),
        evidence: jobs.iter().map(Reference::job).collect(),
        request: format!(
            "Jobs of the class {class} keep ending rejected or failed: {}. Read their records, find what they \
             share, and add the convention or check that would have stopped it.",
            ids.join(", ")
        ),
    }
}

/// What a run did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    /// Work items queued, by key.
    pub queued: Vec<String>,
    /// Requests queued to apply a change the role may make itself.
    pub applied: Vec<JobId>,
}

/// One steward run: reads through `view`, queues work items, and commits them with the role definition
/// (when it is not yet recorded) in one records commit. Does nothing, and commits nothing, when there is
/// nothing to add.
pub fn run(
    committer: &Committer<'_>,
    view: &dyn View,
    applier: &dyn Applier,
    role: &RoleDefinition,
    now: Timestamp,
) -> Result<Report> {
    let mut items: Vec<WorkItem> = Vec::new();
    let mut requests: Vec<QueuedRequest> = Vec::new();
    let mut files: Vec<NewFile> = Vec::new();
    let mut known_keys: BTreeSet<String> = BTreeSet::new();
    let mut role_recorded = false;

    match view.records() {
        Ok(records) => {
            known_keys = records.work_items.iter().map(|w| w.key.clone()).collect();
            let mut taken_jobs: BTreeSet<String> = records.jobs.keys().cloned().collect();
            role_recorded = records
                .roles
                .iter()
                .any(|r| r.name == role.name && r.version == role.version);
            for path in &records.invalid {
                items.push(blind_spot(
                    role,
                    path,
                    "A record file could not be read as a record",
                    now,
                ));
            }
            for (class, jobs) in repeated_failures(&records) {
                let change = change_for(&class, &jobs);
                if known_keys.contains(&change.key) {
                    continue;
                }
                let (decision, item) = decide(role, &change, applier, &taken_jobs, now)?;
                if let Decision::Applied(queued) = decision {
                    taken_jobs.insert(queued.job.as_str().to_string());
                    requests.push(queued);
                }
                items.push(item);
            }
        }
        Err(_) => items.push(blind_spot(
            role,
            RECORDS_SOURCE,
            "The records could not be read",
            now,
        )),
    }

    let mut seen = known_keys;
    items.retain(|item| seen.insert(item.key.clone()));
    if items.is_empty() && requests.is_empty() && role_recorded {
        return Ok(Report::default());
    }
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
    let subject = format!(
        "Steward: {} work item(s), {} request(s) queued",
        items.len(),
        requests.len()
    );
    committer.commit_records(
        &files,
        &message(
            &subject,
            "Work items from the steward's look at everything Athena has done. Each cites its source and \
             evidence by reference.",
            &trailers,
        ),
    )?;
    Ok(Report {
        queued: items.iter().map(|i| i.key.clone()).collect(),
        applied: requests.into_iter().map(|q| q.job).collect(),
    })
}

/// True when a reference is a job.
pub fn is_job(reference: &Reference) -> bool {
    reference.kind == RefKind::Job
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn a_blind_spot_names_its_source() {
        let role = steward(at());
        let item = blind_spot(&role, RECORDS_SOURCE, "The records could not be read", at());
        assert_eq!(item.kind, WorkItemKind::BlindSpot);
        assert_eq!(item.source, Reference::path("records/"));
        assert_eq!(item.key, "blind-spot:records/");
        assert!(item.validate().is_ok());
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
