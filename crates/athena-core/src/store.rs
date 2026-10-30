//! Reading the records at a commit, and deriving state from them (design §5.1 principle 3).
//!
//! No record holds a mutable status. A job's state is a function of which of its records exist.

use crate::plan::Priority;
use anyhow::{Result, anyhow, bail};
use athena_data::records::{
    AnyRecord, Baseline, Candidate, Correction, Critique, DecisionState, DecisionStateKind,
    DefaultExpectations, FailureClass, HostError, Intervention, IntrospectionRecord, Invocation,
    Job, JobPlan, Outcome, OutcomeState, PriorityRecord, ProposalRecord, Reading, Replay, Request,
    Resolution, Restart, Role, Score, Verification, fold_decision_states,
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
    /// The user's priority records, by `n` (`athena.priority/1`); the latest wins.
    pub priorities: Vec<PriorityRecord>,
    pub job: Option<Job>,
    pub invocations: Vec<Invocation>,
    pub candidate: Option<Candidate>,
    pub verification: Option<Verification>,
    pub outcome: Option<Outcome>,
    /// An outcome after `blocked`: the late landing, or the abandonment.
    pub later_outcome: Option<Outcome>,
    pub scores: Vec<Score>,
    /// The job's plans, by round (`athena.job-plan/1`).
    pub plans: Vec<JobPlan>,
    /// The critiques of those plans, by round (`athena.critique/1`).
    pub critiques: Vec<Critique>,
    /// The checks run on the base commit before the producer (`athena.baseline/1`). Recorded only.
    pub baseline: Option<Baseline>,
    /// The replay of the newest accepted readings through the candidate's reading rules (`athena.replay/1`).
    pub replay: Option<Replay>,
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

    /// How soon the job should be worked, in order of precedence: the latest priority record
    /// (`athena priority`); a `Priority:` line in the request's words; urgent when the latest reading gave an
    /// urgent reason; else normal. Plan and status both read it here, so they cannot disagree.
    pub fn priority(&self) -> Priority {
        if let Some(p) = self
            .priorities
            .iter()
            .max_by_key(|p| p.n)
            .and_then(|p| Priority::parse(&p.priority))
        {
            return p;
        }
        let from_words = Priority::from_text(self.request.as_ref().map_or("", |r| r.text.as_str()));
        if from_words != Priority::Normal {
            return from_words;
        }
        if self
            .latest_reading()
            .is_some_and(|r| r.urgent_reason.is_some())
        {
            return Priority::Urgent;
        }
        Priority::Normal
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
            (None, Some(req)) => {
                crate::truncate_at_word(req.text.lines().next().unwrap_or(""), crate::SUBJECT_MAX)
            }
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
            AnyRecord::Priority(p) => self.priorities.push(p),
            AnyRecord::Job(j) => self.job = Some(*j),
            AnyRecord::Invocation(i) => self.invocations.push(*i),
            AnyRecord::Candidate(c) | AnyRecord::Candidate2(c) => self.candidate = Some(*c),
            AnyRecord::Verification(v) => self.verification = Some(*v),
            AnyRecord::Outcome(o) if o.after_blocked => self.later_outcome = Some(*o),
            AnyRecord::Outcome(o) => self.outcome = Some(*o),
            AnyRecord::Score(s) => self.scores.push(*s),
            AnyRecord::JobPlan(p) => self.plans.push(*p),
            AnyRecord::Critique(c) => self.critiques.push(*c),
            AnyRecord::Baseline(b) => self.baseline = Some(*b),
            AnyRecord::Replay(r) => self.replay = Some(*r),
            other => {
                let _ = (path, other);
            }
        }
    }

    fn sort(&mut self) {
        self.readings.sort_by_key(|r| r.n);
        self.corrections.sort_by_key(|c| c.n);
        self.priorities.sort_by_key(|p| p.n);
        self.invocations.sort_by_key(|i| i.seq);
        self.scores.sort_by_key(|s| s.at);
        self.plans.sort_by_key(|p| p.round);
        self.critiques.sort_by_key(|c| c.round);
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
    /// Every class-label record (`crate::labeled`), in file order.
    pub class_labels: Vec<athena_data::records::ClassLabel>,
    /// Every recorded role definition, in file order.
    pub roles: Vec<athena_data::roles::RoleDefinition>,
    /// Every work item a role queued, in file order.
    pub work_items: Vec<athena_data::records::WorkItem>,
    /// Every decision on a work item (`crate::work_decisions`), in file order.
    pub work_decisions: Vec<athena_data::records::WorkDecision>,
    /// The steward's model invocations, oldest first. A steward run is no job, so its invocation is kept apart
    /// from `jobs` and never shows as work to do.
    pub steward_runs: Vec<Invocation>,
    /// Every audience rating, oldest first; the last is the latest.
    pub audience_ratings: Vec<athena_data::records::AudienceRating>,
    /// The audience's model invocations, oldest first, kept apart from `jobs` as the steward's are.
    pub audience_runs: Vec<Invocation>,
    /// Every audit of a job's outcome (`crate::auditor`), oldest first.
    pub audits: Vec<athena_data::records::Audit>,
    /// Every Hestia sweep (`crate::hestia`), oldest first; the last is the latest.
    pub hestia_sweeps: Vec<athena_data::records::HestiaSweep>,
    /// The auditor's model invocations, oldest first, kept apart from `jobs` as the audience's are.
    pub auditor_runs: Vec<Invocation>,
    /// Every code-health baseline (`crate::code_health`), oldest first; the last is the latest.
    pub code_health_baselines: Vec<athena_data::records::CodeHealthBaseline>,
    /// Every throughput review (`crate::hestia`), oldest first; the last is the latest.
    pub throughput_reviews: Vec<athena_data::records::ThroughputReview>,
    /// Every build measurement (`crate::disk`), oldest first; the last is the latest.
    pub build_measurements: Vec<athena_data::records::BuildMeasurement>,
    /// Landing records (`landings/`, written after each merge), oldest first. Read from the tree itself and
    /// from the local records branch; one that cannot be read is left out, so a missing branch reads as none.
    pub landings: Vec<athena_data::history::Landing>,
    /// Committer time of each commit on the default branch's first-parent chain, by sha. Used to date a landing
    /// record by the commit it names, not by when the record was written.
    pub main_commit_times: std::collections::BTreeMap<String, jiff::Timestamp>,
    /// For a job id, the earliest commit on that chain whose message names the job: its merge commit.
    pub main_job_commits: std::collections::BTreeMap<String, jiff::Timestamp>,
    /// For a job id, the committer time of the job's own candidate commit, when that commit is in the repository.
    pub job_commit_times: std::collections::BTreeMap<String, jiff::Timestamp>,
    /// Files that are not acceptable records, by path.
    pub invalid: Vec<String>,
}

/// The landing records under `landings/` in `treeish`; an absent directory or an unreadable file counts nothing.
fn landings_in(git: &Git, treeish: &str) -> Vec<athena_data::history::Landing> {
    let Ok(entries) = git.ls_tree_under(treeish, "landings") else {
        return Vec::new();
    };
    let oids: Vec<String> = entries
        .into_iter()
        .filter(|e| e.kind == "blob")
        .map(|e| e.oid)
        .collect();
    let Ok(blobs) = git.cat_blobs(&oids) else {
        return Vec::new();
    };
    blobs
        .iter()
        .filter_map(|bytes| {
            match athena_data::history::HistoryRecord::parse(&String::from_utf8_lossy(bytes)) {
                Ok(athena_data::history::HistoryRecord::Landing(l)) => Some(l),
                _ => None,
            }
        })
        .collect()
}

fn job_of(record: &AnyRecord) -> Option<&JobId> {
    match record {
        AnyRecord::Request(r) => Some(&r.job),
        AnyRecord::Reading(r) => Some(&r.job),
        AnyRecord::Correction(r) => Some(&r.job),
        AnyRecord::Priority(r) => Some(&r.job),
        AnyRecord::Job(r) => Some(&r.id),
        AnyRecord::Invocation(r) => Some(&r.job),
        AnyRecord::Candidate(r) | AnyRecord::Candidate2(r) => Some(&r.job),
        AnyRecord::Verification(r) => Some(&r.job),
        AnyRecord::Outcome(r) => Some(&r.job),
        AnyRecord::Score(r) => Some(&r.job),
        AnyRecord::JobPlan(r) => Some(&r.job),
        AnyRecord::Critique(r) => Some(&r.job),
        AnyRecord::Baseline(r) => Some(&r.job),
        AnyRecord::Replay(r) => Some(&r.job),
        _ => None,
    }
}

impl Records {
    /// Reads the commit times that date landings from git: the default branch's first-parent chain (`main`, else
    /// `treeish`), and the candidate commit of each landed job. Anything git cannot show is left out, so the
    /// status line reports no evidence for that job instead of guessing.
    fn load_commit_times(&mut self, git: &Git, treeish: &str) {
        let parse = |t: &str| t.parse::<jiff::Timestamp>().ok();
        for rev in ["main", treeish] {
            let Ok(commits) = git.log_commit_times(rev, 5000) else {
                continue;
            };
            for (sha, time, message) in &commits {
                let Some(time) = parse(time) else { continue };
                self.main_commit_times.insert(sha.clone(), time);
                for id in self.jobs.keys() {
                    if message.contains(id.as_str()) {
                        let e = self.main_job_commits.entry(id.clone()).or_insert(time);
                        *e = (*e).min(time);
                    }
                }
            }
            break;
        }
        for (id, job) in &self.jobs {
            if let Some(sha) = job
                .candidate
                .as_ref()
                .and_then(|c| c.candidate_commit.as_ref())
                && let Some(time) = git.commit_time(sha).as_deref().and_then(parse)
            {
                self.job_commit_times.insert(id.clone(), time);
            }
        }
    }

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
                AnyRecord::ClassLabel(l) => records.class_labels.push(l),
                AnyRecord::Role(r) => records.roles.push(*r),
                AnyRecord::WorkItem(w) => records.work_items.push(*w),
                AnyRecord::WorkDecision(d) => records.work_decisions.push(d),
                AnyRecord::AudienceRating(a) => records.audience_ratings.push(*a),
                AnyRecord::Audit(a) => records.audits.push(*a),
                AnyRecord::HestiaSweep(s) => records.hestia_sweeps.push(*s),
                AnyRecord::CodeHealthBaseline(b) => records.code_health_baselines.push(b),
                AnyRecord::ThroughputReview(t) => records.throughput_reviews.push(*t),
                AnyRecord::BuildMeasurement(m) => records.build_measurements.push(*m),
                AnyRecord::Invocation(i) if i.role == Role::Auditor => {
                    records.auditor_runs.push(*i);
                }
                AnyRecord::Invocation(i) if i.role == Role::Steward => {
                    records.steward_runs.push(*i);
                }
                AnyRecord::Invocation(i) if i.role == Role::Audience => {
                    records.audience_runs.push(*i);
                }
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
        let mut landings = landings_in(git, treeish);
        if let Ok(Some(tip)) = athena_git::records_branch::RecordsWriter::new(git).tip() {
            landings.extend(landings_in(git, &tip));
        }
        landings.sort_by(|a, b| (a.created_at, &a.id).cmp(&(b.created_at, &b.id)));
        landings.dedup_by(|a, b| a.id == b.id);
        records.landings = landings;
        records.load_commit_times(git, treeish);
        records.steward_runs.sort_by_key(|i| i.started_at);
        records.audience_runs.sort_by_key(|i| i.started_at);
        records.auditor_runs.sort_by_key(|i| i.started_at);
        records
            .audits
            .sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
        records
            .hestia_sweeps
            .sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
        records
            .audience_ratings
            .sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
        records
            .code_health_baselines
            .sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
        records
            .throughput_reviews
            .sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
        records
            .build_measurements
            .sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
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
            || self.work_items.iter().any(|r| &r.id == id)
            || self.work_decisions.iter().any(|r| &r.id == id)
            || self.hestia_sweeps.iter().any(|r| &r.id == id)
            || self.throughput_reviews.iter().any(|r| &r.id == id)
            || self.build_measurements.iter().any(|r| &r.id == id)
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

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::records::{
        Critique, CritiqueVerdict, DeliverableAction, Demonstration, Finding, FindingSource,
        JobKind, JobPlan, Owner, PlanFile, Replay, Severity,
    };

    fn job_id() -> JobId {
        JobId::parse("20261010-153012-k7q2").unwrap()
    }

    fn plan(round: u32) -> JobPlan {
        JobPlan {
            job: job_id(),
            round,
            at: "2026-10-10T15:30:12Z".parse().unwrap(),
            invocation_seq: 1,
            approach: "Add the records.".into(),
            decisions: vec![],
            files: vec![PlanFile {
                path: "a.md".into(),
                action: DeliverableAction::Create,
                purpose: "Hold it.".into(),
                governance: false,
                exists: false,
            }],
            steps: vec![],
            risks: vec![],
            demonstrations: vec![Demonstration {
                criterion: 1,
                method: "test".into(),
                command: "cargo test".into(),
                observes: "pass".into(),
                fails_without: None,
            }],
            open_questions: vec![],
            producer_section_sha256: "d".repeat(64),
            resumed_from: None,
        }
    }

    fn critique(round: u32, verdict: CritiqueVerdict) -> Critique {
        Critique {
            job: job_id(),
            round,
            at: "2026-10-10T15:30:12Z".parse().unwrap(),
            invocation_seq: Some(2),
            agent_verdict: Some(verdict),
            final_verdict: verdict,
            findings: vec![Finding {
                severity: Severity::Minor,
                summary: "Vague.".into(),
                evidence: "step 1".into(),
                location: None,
                source: FindingSource::Critic,
            }],
            request_reading_mismatch: None,
            summary: None,
        }
    }

    fn dispatched() -> Job {
        Job {
            id: job_id(),
            kind: JobKind::Request,
            title: "Add a file".into(),
            class_of_work: None,
            capability: None,
            accepted_reading: None,
            accepted_by: None,
            owner: Owner {
                role: "athena".into(),
                capability: None,
                model: None,
                note: None,
            },
            verifier_model: None,
            caps: None,
            base_commit: None,
            config_sha256: None,
            defaults_sha256: None,
            athena: None,
            driver: None,
            brief_sha256: None,
            dispatched_at: None,
            request_path: None,
            request_sha256: None,
            cost: None,
            tokens: None,
            note: None,
            pin: None,
        }
    }

    #[test]
    fn plans_and_critiques_are_filed_under_their_job_and_do_not_close_it() {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::at(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value]).unwrap();
        }
        let records = [
            AnyRecord::Job(Box::new(dispatched())),
            // Written out of order to show that loading sorts by round.
            AnyRecord::Critique(Box::new(critique(2, CritiqueVerdict::Accept))),
            AnyRecord::JobPlan(Box::new(plan(2))),
            AnyRecord::JobPlan(Box::new(plan(1))),
            AnyRecord::Critique(Box::new(critique(1, CritiqueVerdict::Revise))),
        ];
        for record in &records {
            let path = dir.path().join(record.canonical_path());
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, record.to_json()).unwrap();
        }
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "Plan a job"]).unwrap();

        let loaded = Records::load(&git, "HEAD").unwrap();
        assert!(loaded.invalid.is_empty(), "{:?}", loaded.invalid);
        let job = &loaded.jobs[job_id().as_str()];
        assert_eq!(
            job.plans.iter().map(|p| p.round).collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(
            job.critiques.iter().map(|c| c.round).collect::<Vec<_>>(),
            [1, 2]
        );
        assert_eq!(job.critiques[0].final_verdict, CritiqueVerdict::Revise);
        assert!(job.outcome.is_none());
        assert_eq!(job.state(), JobState::Dispatched);
    }

    #[test]
    fn replay_record_loads_into_its_job() {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::at(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value]).unwrap();
        }
        let replay = Replay {
            job: job_id(),
            base_commit: "a".repeat(40),
            candidate_commit: "b".repeat(40),
            window: 50,
            total: 0,
            newly_refused: vec![],
            held: false,
            intended: false,
            no_replay: Some("no replay: no accepted readings in the history".into()),
        };
        for record in [
            AnyRecord::Job(Box::new(dispatched())),
            AnyRecord::Replay(Box::new(replay.clone())),
        ] {
            let path = dir.path().join(record.canonical_path());
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, record.to_json()).unwrap();
        }
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "Replay"]).unwrap();

        let loaded = Records::load(&git, "HEAD").unwrap();
        assert!(loaded.invalid.is_empty(), "{:?}", loaded.invalid);
        assert_eq!(loaded.jobs[job_id().as_str()].replay, Some(replay));
    }
}
