//! Reading the records at a commit, and deriving state from them (design §5.1 principle 3).
//!
//! No record holds a mutable status. A job's state is a function of which of its records exist.

use anyhow::{Result, anyhow, bail};
use athena_data::records::{
    AnyRecord, Candidate, Correction, Intervention, Invocation, Job, Outcome, OutcomeState,
    Reading, Request, Resolution, Restart, Score, Verification,
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
        self.invocations.iter().map(|i| i.cost.best_lower()).sum()
    }

    fn add(&mut self, path: &str, record: AnyRecord) {
        match record {
            AnyRecord::Request(r) => self.request = Some(r),
            AnyRecord::Reading(r) => self.readings.push(*r),
            AnyRecord::Correction(c) => self.corrections.push(c),
            AnyRecord::Job(j) => self.job = Some(*j),
            AnyRecord::Invocation(i) => self.invocations.push(*i),
            AnyRecord::Candidate(c) => self.candidate = Some(*c),
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

/// Every record at one commit.
#[derive(Debug, Clone, Default)]
pub struct Records {
    pub jobs: BTreeMap<String, JobRecords>,
    pub interventions: Vec<Intervention>,
    pub restarts: Vec<Restart>,
    pub resolutions: Vec<Resolution>,
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
        AnyRecord::Candidate(r) => Some(&r.job),
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
    }
}
