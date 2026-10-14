//! Reading the records at a commit, and deriving state from them (design §5.1 principle 3).
//!
//! No record holds a mutable status. A job's state is a function of which of its records exist.

use anyhow::{Result, anyhow, bail};
use athena_data::records::{
    AnyRecord, Candidate, Correction, DecisionState, DecisionStateKind, DefaultExpectations,
    FailureClass, HostError, Intervention, IntrospectionRecord, Invocation, Job, Outcome,
    OutcomeState, ProposalRecord, Reading, Request, Resolution, Restart, Score, Verification,
    fold_decision_states,
};
use athena_data::{JobId, RecordId};
use athena_gate::records::RecordSet;
use athena_git::Git;
use std::collections::BTreeMap;

/// Every record of one job.
#[derive(Debug, Clone, Default)]
pub struct JobRecords {
    pub id: Option<JobId>,
    pub request: Option<Request>,
    pub readings: Vec<Reading>,
    pub corrections: Vec<Correction>,
    pub job: Option<Job>,
    pub invocations: Vec<Invocation>,
    pub candidate: Option<Candidate>,
    pub verification: Option<Verification>,
    pub outcome: Option<Outcome>,
    /// An outcome after `blocked`: the late landing, or the abandonment.
    pub later_outcome: Option<Outcome>,
    pub scores: Vec<Score>,
    pub briefs: BTreeMap<String, String>,
}

/// A job's state, derived from its records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobState {
    /// Requested for later and not read yet (`athena ask --later`); `go` reads it.
    Queued,
    /// Read (or being read); waiting for `go`, a correction, or `close`.
    Proposed,
    /// No capability fits; waiting for a correction or `close`.
    Declined,
    /// Dispatched with no closing record: running, or interrupted if no process owns it.
    Dispatched,
    /// Verified, but the landing was refused by local edits; `go` retries it.
    Blocked,
    /// A terminal outcome.
    Closed(OutcomeState),
}

impl JobState {
    pub fn name(self) -> &'static str {
        match self {
            JobState::Queued => "queued",
            JobState::Proposed => "proposed",
            JobState::Declined => "declined",
            JobState::Dispatched => "dispatched",
            JobState::Blocked => "blocked",
            JobState::Closed(state) => state.as_str(),
        }
    }
}

impl JobRecords {
    pub fn id(&self) -> &JobId {
        self.id.as_ref().expect("job records always carry their id")
    }

    pub fn latest_reading(&self) -> Option<&Reading> {
        self.readings.last()
    }

    pub fn reading(&self, n: u32) -> Option<&Reading> {
        self.readings.iter().find(|r| r.n == n)
    }

    pub fn state(&self) -> JobState {
        if let Some(later) = &self.later_outcome {
            return JobState::Closed(later.state);
        }
        if let Some(outcome) = &self.outcome {
            return if outcome.state == OutcomeState::Blocked {
                JobState::Blocked
            } else {
                JobState::Closed(outcome.state)
            };
        }
        if self.job.is_some() {
            return JobState::Dispatched;
        }
        if self.readings.is_empty() && self.invocations.is_empty() {
            return JobState::Queued;
        }
        match self.latest_reading() {
            Some(r) if r.capability.is_none() => JobState::Declined,
            _ => JobState::Proposed,
        }
    }

    /// The authoritative score: the latest by time.
    pub fn latest_score(&self) -> Option<&Score> {
        self.scores.iter().max_by_key(|s| s.at)
    }

    /// The next invocation sequence number.
    pub fn next_seq(&self) -> u32 {
        self.invocations.iter().map(|i| i.seq).max().unwrap_or(0) + 1
    }

    /// The title to show: the latest reading's, else the request's first line.
    pub fn title(&self) -> String {
        match (self.latest_reading(), &self.request) {
            (Some(r), _) => r.title.clone(),
            (None, Some(req)) => req
                .text
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(72)
                .collect(),
            _ => String::new(),
        }
    }

    /// The class of work, once read.
    pub fn class(&self) -> Option<&str> {
        self.job
            .as_ref()
            .and_then(|j| j.class_of_work.as_deref())
            .or_else(|| self.latest_reading().map(|r| r.class_of_work.as_str()))
    }

    /// The sum of every invocation's cost (best value or lower bound).
    pub fn spent_usd(&self) -> f64 {
        positive_zero(self.invocations.iter().map(|i| i.cost.best_lower()).sum())
    }

    fn add(&mut self, path: &str, record: AnyRecord) {
        match record {
            AnyRecord::Request(r) => self.request = Some(r),
            AnyRecord::Reading(r) => self.readings.push(*r),
            AnyRecord::Correction(c) => self.corrections.push(c),
            AnyRecord::Job(j) => self.job = Some(*j),
            AnyRecord::Invocation(i) => self.invocations.push(*i),
            AnyRecord::Candidate(c) | AnyRecord::Candidate2(c) => self.candidate = Some(*c),
            AnyRecord::Verification(v) => self.verification = Some(*v),
            AnyRecord::Outcome(o) if o.after_blocked => self.later_outcome = Some(*o),
            AnyRecord::Outcome(o) => self.outcome = Some(*o),
            AnyRecord::Score(s) => self.scores.push(*s),
            other => {
                let _ = (path, other);
            }
        }
    }

    fn sort(&mut self) {
        self.readings.sort_by_key(|r| r.n);
        self.corrections.sort_by_key(|c| c.n);
        self.invocations.sort_by_key(|i| i.seq);
        self.scores.sort_by_key(|s| s.at);
    }
}

/// `value`, with a negative zero (what summing no values gives) made positive, so nothing shows `$-0.00`.
pub fn positive_zero(value: f64) -> f64 {
    if value == 0.0 { 0.0 } else { value }
}

/// Every record at one commit.
#[derive(Debug, Clone, Default)]
pub struct Records {
    pub jobs: BTreeMap<String, JobRecords>,
    pub interventions: Vec<Intervention>,
    pub restarts: Vec<Restart>,
    pub resolutions: Vec<Resolution>,
    /// Refused host calls, oldest first.
    pub host_errors: Vec<HostError>,
    /// Changes of a decision's state, oldest first; fold them with [`Records::decision_state`].
    pub decision_states: Vec<DecisionState>,
    /// What each default assumes and watches, oldest first.
    pub default_expectations: Vec<DefaultExpectations>,
    /// Introspections, oldest first.
    pub introspections: Vec<IntrospectionRecord>,
    /// The class registry: one record per class, oldest first.
    pub classes: Vec<FailureClass>,
    /// Every defaults-proposal record, oldest first; fold them with `proposals::current`.
    pub proposals: Vec<ProposalRecord>,
    /// Files that are not acceptable records, by path.
    pub invalid: Vec<String>,
}

fn job_of(record: &AnyRecord) -> Option<&JobId> {
    match record {
        AnyRecord::Request(r) => Some(&r.job),
        AnyRecord::Reading(r) => Some(&r.job),
        AnyRecord::Correction(r) => Some(&r.job),
        AnyRecord::Job(r) => Some(&r.id),
        AnyRecord::Invocation(r) => Some(&r.job),
        AnyRecord::Candidate(r) | AnyRecord::Candidate2(r) => Some(&r.job),
        AnyRecord::Verification(r) => Some(&r.job),
        AnyRecord::Outcome(r) => Some(&r.job),
        AnyRecord::Score(r) => Some(&r.job),
        _ => None,
    }
}

impl Records {
    /// Loads every record in `treeish`.
    pub fn load(git: &Git, treeish: &str) -> Result<Records> {
        let set = RecordSet::load(git, treeish)?;
        let mut records = Records::default();
        for file in &set.files {
            if file.error.is_some() {
                records.invalid.push(file.path.clone());
                continue;
            }
            if let Some(name) = file.path.strip_suffix(".md")
                && let Some((dir, brief)) = name.rsplit_once('/')
                && let Some(job) = dir.rsplit('/').next().and_then(|id| JobId::parse(id).ok())
            {
                let entry = records.jobs.entry(job.as_str().to_string()).or_default();
                entry.id.get_or_insert(job);
                entry.briefs.insert(
                    brief.to_string(),
                    String::from_utf8_lossy(&file.bytes).into_owned(),
                );
                continue;
            }
            let Some(record) = file.record.clone() else {
                continue;
            };
            match record {
                AnyRecord::Intervention(i) => records.interventions.push(i),
                AnyRecord::Restart(r) => records.restarts.push(r),
                AnyRecord::Resolution(r) => records.resolutions.push(r),
                AnyRecord::HostError(h) => records.host_errors.push(h),
                AnyRecord::DecisionState(d) => records.decision_states.push(d),
                AnyRecord::DefaultExpectations(e) => records.default_expectations.push(e),
                AnyRecord::Introspection(i) => records.introspections.push(*i),
                AnyRecord::FailureClass(c) => records.classes.push(c),
                AnyRecord::DefaultsProposal(p) => records.proposals.push(*p),
                other => {
                    let job = job_of(&other)
                        .expect("every other record names its job")
                        .clone();
                    let entry = records.jobs.entry(job.as_str().to_string()).or_default();
                    entry.id.get_or_insert(job);
                    entry.add(&file.path, other);
                }
            }
        }
        for job in records.jobs.values_mut() {
            job.sort();
        }
        records.interventions.sort_by_key(|i| i.at);
        records.restarts.sort_by_key(|r| r.at);
        records
            .host_errors
            .sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
        records
            .decision_states
            .sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
        records
            .default_expectations
            .sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
        records
            .introspections
            .sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
        records
            .classes
            .sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
        records
            .proposals
            .sort_by(|a, b| (a.at, &a.record).cmp(&(b.at, &b.record)));
        Ok(records)
    }

    /// Jobs other than the bootstrap, newest first.
    pub fn request_jobs(&self) -> Vec<&JobRecords> {
        let mut jobs: Vec<&JobRecords> =
            self.jobs.values().filter(|j| j.id().is_request()).collect();
        jobs.sort_by(|a, b| b.id().as_str().cmp(a.id().as_str()));
        jobs
    }

    /// Resolves a job reference: a full id, a unique suffix, or `latest`.
    pub fn resolve(&self, reference: &str) -> Result<JobId> {
        if reference == "latest" {
            return self
                .request_jobs()
                .first()
                .map(|j| j.id().clone())
                .ok_or_else(|| {
                    anyhow!("there are no jobs yet; start one with `athena ask \"<request>\"`")
                });
        }
        if let Ok(id) = JobId::parse(reference)
            && self.jobs.contains_key(id.as_str())
        {
            return Ok(id);
        }
        let matches: Vec<&JobRecords> = self
            .jobs
            .values()
            .filter(|j| j.id().as_str().ends_with(reference))
            .collect();
        match matches.as_slice() {
            [one] => Ok(one.id().clone()),
            [] => bail!("no job matches {reference:?}; `athena status` lists them"),
            many => bail!(
                "{reference:?} matches {} jobs ({}); give more of the id",
                many.len(),
                many.iter()
                    .map(|j| j.id().as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }

    /// Interventions that name a job.
    pub fn interventions_for(&self, job: &JobId) -> Vec<&Intervention> {
        self.interventions
            .iter()
            .filter(|i| i.job.as_ref() == Some(job))
            .collect()
    }

    /// Whether a record id is already used.
    pub fn has_record_id(&self, id: &RecordId) -> bool {
        self.interventions.iter().any(|i| &i.id == id)
            || self.restarts.iter().any(|r| &r.id == id)
            || self.resolutions.iter().any(|r| &r.id == id)
            || self.host_errors.iter().any(|r| &r.id == id)
            || self.decision_states.iter().any(|r| &r.id == id)
            || self.default_expectations.iter().any(|r| &r.id == id)
            || self.introspections.iter().any(|r| &r.id == id)
            || self.classes.iter().any(|r| &r.id == id)
            || self.proposals.iter().any(|r| &r.record == id)
    }

    /// The current state of each decision, folded from its append-only state records.
    pub fn decision_state(&self) -> BTreeMap<String, DecisionStateKind> {
        fold_decision_states(&self.decision_states)
    }

    /// The newest `athena.default-expectations/1` record for a default, if any.
    pub fn expectations_for(&self, default_id: &str) -> Option<&DefaultExpectations> {
        self.default_expectations
            .iter()
            .rfind(|e| e.default_id == default_id)
    }
}
