//! `athena status`: the state of the work, derived from the records at `HEAD` and the state directory
//! (design §3.6). Nothing about the state of the work lives in a hand-maintained file (D35).

use crate::Athena;
use crate::manage::pending_quantities;
use crate::store::{JobRecords, JobState, Records};
use anyhow::Result;
use athena_data::records::{Build, Invocation, OutcomeState, Reading};
use athena_gate::invariant::{BOOTSTRAP_OUTCOME, BootstrapRange, bootstrap_range};
use serde::Serialize;
use std::collections::BTreeMap;

/// One job, in one line.
#[derive(Debug, Clone, Serialize)]
pub struct JobLine {
    pub id: String,
    pub state: String,
    pub title: String,
    pub class: Option<String>,
    pub cost_usd: f64,
}

/// One class's forecast headline.
#[derive(Debug, Clone, Serialize)]
pub struct ClassHeadline {
    pub class: String,
    pub scored_jobs: usize,
    pub reader_mean_brier: Option<f64>,
    pub reference_mean_brier: Option<f64>,
}

/// A quantity that is not known yet.
#[derive(Debug, Clone, Serialize)]
pub struct Pending {
    pub record: String,
    pub field: String,
    pub basis: String,
    pub resolved_by: Option<String>,
}

/// The bootstrap's range of unverified commits.
#[derive(Debug, Clone, Serialize)]
pub struct Bootstrap {
    pub range_open: bool,
    pub through_commit: Option<String>,
    pub note: String,
}

/// The state of the work.
#[derive(Debug, Clone, Serialize)]
pub struct StatusView {
    pub repository: String,
    pub branch: Option<String>,
    pub head: String,
    pub athena: Build,
    pub driver: String,
    pub kill_switch_on: bool,
    pub nested: Option<String>,
    pub bootstrap: Bootstrap,
    pub jobs_by_state: BTreeMap<String, usize>,
    pub recent: Vec<JobLine>,
    pub open: Vec<JobLine>,
    pub interventions_by_category: BTreeMap<String, usize>,
    pub interventions_by_class: BTreeMap<String, usize>,
    pub restarts: usize,
    pub forecasts: Vec<ClassHeadline>,
    pub pending: Vec<Pending>,
    pub invalid_records: Vec<String>,
    pub next: Vec<String>,
}

/// One invocation in a job's trace.
#[derive(Debug, Clone, Serialize)]
pub struct InvocationLine {
    pub seq: u32,
    pub role: String,
    pub model: String,
    pub outcome: String,
    pub cost: String,
    pub tokens: String,
    pub wall_time_s: Option<f64>,
    pub commands_run: usize,
    pub denials: usize,
}

/// A job's full trace: request → prediction → owner → model → cost → verification → outcome → score.
#[derive(Debug, Clone, Serialize)]
pub struct JobTrace {
    pub id: String,
    pub state: String,
    pub request: Option<String>,
    pub readings: Vec<Reading>,
    pub corrections: Vec<String>,
    pub accepted_reading: Option<u32>,
    pub owner: Option<String>,
    pub verifier: Option<String>,
    pub invocations: Vec<InvocationLine>,
    pub candidate: Option<serde_json::Value>,
    pub verification: Option<serde_json::Value>,
    pub outcome: Option<serde_json::Value>,
    pub later_outcome: Option<serde_json::Value>,
    pub score: Option<serde_json::Value>,
    pub interventions: Vec<String>,
    pub commits: Vec<String>,
    pub directory: String,
}

fn job_line(job: &JobRecords, state: &str) -> JobLine {
    JobLine {
        id: job.id().as_str().to_string(),
        state: state.to_string(),
        title: job.title(),
        class: job.class().map(str::to_string),
        cost_usd: job.spent_usd(),
    }
}

fn quantity_text(q: &athena_data::Quantity) -> String {
    match (q.value(), q.lower(), q.upper()) {
        (Some(v), _, _) => format!("{v} {} ({:?})", q.unit(), q.provenance()).to_lowercase(),
        (None, l, u) => format!(
            "[{}, {}] {} ({:?})",
            l.map_or("?".into(), |v| v.to_string()),
            u.map_or("?".into(), |v| v.to_string()),
            q.unit(),
            q.provenance()
        )
        .to_lowercase(),
    }
}

fn invocation_line(i: &Invocation) -> InvocationLine {
    let t = &i.tokens;
    InvocationLine {
        seq: i.seq,
        role: i.role.as_str().into(),
        model: i.model.clone(),
        outcome: format!("{:?}", i.outcome).to_lowercase(),
        cost: quantity_text(&i.cost),
        tokens: format!(
            "in {} · out {} · cache read {} · cache write {} ({:?})",
            t.input.best_lower(),
            t.output.best_lower(),
            t.cache_read.best_lower(),
            t.cache_write.best_lower(),
            t.output.provenance()
        )
        .to_lowercase(),
        wall_time_s: i.wall_time.value(),
        commands_run: i.commands_run.len(),
        denials: i.policy_denials.len(),
    }
}

impl Athena {
    fn running(&self, job: &str) -> bool {
        self.state.registry().entries().iter().any(|e| e.job == job)
    }

    fn state_name(&self, job: &JobRecords) -> String {
        match job.state() {
            JobState::Dispatched if self.running(job.id().as_str()) => "running".into(),
            JobState::Dispatched => "interrupted".into(),
            other => other.name().into(),
        }
    }

    /// The state of the work.
    pub fn status(&self) -> Result<StatusView> {
        self.state.registry().reap();
        let head = self.head()?;
        let records = Records::load(&self.git, &head)?;
        let set = athena_gate::records::RecordSet::load(&self.git, &head)?;
        let mut jobs_by_state: BTreeMap<String, usize> = BTreeMap::new();
        let mut recent = Vec::new();
        let mut open = Vec::new();
        for job in records.request_jobs() {
            let state = self.state_name(job);
            *jobs_by_state.entry(state.clone()).or_default() += 1;
            if recent.len() < 10 {
                recent.push(job_line(job, &state));
            }
            if matches!(
                state.as_str(),
                "proposed" | "declined" | "interrupted" | "running" | "blocked"
            ) {
                open.push(job_line(job, &state));
            }
        }
        let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
        let mut by_class: BTreeMap<String, usize> = BTreeMap::new();
        for i in &records.interventions {
            *by_category.entry(i.category.as_str().into()).or_default() += 1;
            *by_class.entry(i.class_of_work.clone()).or_default() += 1;
        }
        let mut per_class: BTreeMap<String, Vec<&athena_data::records::Score>> = BTreeMap::new();
        for job in records.request_jobs() {
            if let (Some(score), Some(class)) = (job.latest_score(), job.class()) {
                per_class.entry(class.to_string()).or_default().push(score);
            }
        }
        let forecasts = per_class
            .into_iter()
            .map(|(class, scores)| {
                let agg = athena_score::aggregate(&scores);
                let mean = |name: &str| {
                    agg.per_forecaster
                        .get(name)
                        .filter(|t| t.events > 0)
                        .map(|t| t.event_brier_sum / t.events as f64)
                };
                ClassHeadline {
                    class,
                    scored_jobs: scores.len(),
                    reader_mean_brier: mean("reader"),
                    reference_mean_brier: mean("reference"),
                }
            })
            .collect();
        let resolutions: BTreeMap<(String, String), String> = records
            .resolutions
            .iter()
            .map(|r| {
                (
                    (r.target_path.clone(), r.target_field.clone()),
                    format!("records/resolutions/{}/{}.json", r.id.month(), r.id),
                )
            })
            .collect();
        let pending: Vec<Pending> = pending_quantities(&set)
            .into_iter()
            .map(|(record, field, basis)| {
                let resolved_by = resolutions.get(&(record.clone(), field.clone())).cloned();
                Pending {
                    record,
                    field,
                    basis,
                    resolved_by,
                }
            })
            .collect();
        let range = bootstrap_range(&self.git, &head)?;
        let bootstrap = match &range {
            BootstrapRange::Open => Bootstrap {
                range_open: true,
                through_commit: None,
                note: format!(
                    "OPEN: commits labeled Athena-Job: bootstrap may still change governed content unverified. The bootstrap \
                     closes its range by adding {BOOTSTRAP_OUTCOME}, naming its last commit (docs/records.md)."
                ),
            },
            BootstrapRange::ClosedThrough(through) => Bootstrap {
                range_open: false,
                through_commit: Some(through.clone()),
                note: "closed: governed content changes only through verified landings and declared interventions".into(),
            },
        };
        let kill_switch_on = self.state.registry().stopped();
        let mut next = Vec::new();
        if kill_switch_on {
            next.push("athena start   # the kill switch is on; new work is blocked".into());
        }
        for line in &open {
            let id = &line.id;
            match line.state.as_str() {
                "proposed" => next.push(format!("athena go {id}   # or: athena correct {id} \"<line>\" · athena close {id} --why \"...\"")),
                "declined" => next.push(format!("athena correct {id} \"<line>\"   # or: athena close {id} --why \"...\"")),
                "interrupted" => next.push(format!("athena close {id} --why \"...\"   # commits what the interrupted job left")),
                "blocked" => next.push(format!("athena go {id}   # after resolving local edits; or: athena close {id} --why \"...\"")),
                _ => {}
            }
        }
        if bootstrap.range_open {
            next.push(format!("close the bootstrap range: commit {BOOTSTRAP_OUTCOME} (docs/records.md, \"Closing the bootstrap\")"));
        }
        for p in pending.iter().filter(|p| p.resolved_by.is_none()) {
            next.push(format!("resolve {}#{}: add an athena.resolution/1 record (docs/records.md, \"Resolving a pending quantity\")", p.record, p.field));
        }
        if records.request_jobs().is_empty() {
            next.push("athena ask \"<a one- or two-line request>\"".into());
        }
        Ok(StatusView {
            repository: self.git.dir().display().to_string(),
            branch: self.git.current_branch()?,
            head,
            athena: self.build.clone(),
            driver: format!("{} {}", self.driver.info().name, self.driver.info().version),
            kill_switch_on,
            nested: self.nested.clone(),
            bootstrap,
            jobs_by_state,
            recent,
            open,
            interventions_by_category: by_category,
            interventions_by_class: by_class,
            restarts: records.restarts.len(),
            forecasts,
            pending,
            invalid_records: records.invalid.clone(),
            next,
        })
    }

    /// A job's full trace.
    pub fn trace(&self, job_ref: &str) -> Result<JobTrace> {
        let head = self.head()?;
        let records = Records::load(&self.git, &head)?;
        let id = records.resolve(job_ref)?;
        let job = records.jobs.get(id.as_str()).expect("resolved");
        let to_value = |v: &dyn erased::ToValue| v.to_value();
        let commits = self
            .git
            .run([
                "log",
                "--format=%h %s",
                "-E",
                &format!("--grep=^Athena-Job: {}$", id.as_str()),
                &head,
            ])?
            .lines()
            .map(str::to_string)
            .collect();
        Ok(JobTrace {
            id: id.as_str().to_string(),
            state: self.state_name(job),
            request: job.request.as_ref().map(|r| r.text.clone()),
            readings: job.readings.clone(),
            corrections: job
                .corrections
                .iter()
                .map(|c| format!("reading {}: {}", c.n, c.text))
                .collect(),
            accepted_reading: job.job.as_ref().and_then(|j| j.accepted_reading),
            owner: job.job.as_ref().map(|j| {
                format!(
                    "{} · capability {} · model {}",
                    j.owner.role,
                    j.owner.capability.clone().unwrap_or_default(),
                    j.owner.model.clone().unwrap_or_default()
                )
            }),
            verifier: job.job.as_ref().and_then(|j| j.verifier_model.clone()),
            invocations: job.invocations.iter().map(invocation_line).collect(),
            candidate: job.candidate.as_ref().map(|c| to_value(c)),
            verification: job.verification.as_ref().map(|v| to_value(v)),
            outcome: job.outcome.as_ref().map(|o| to_value(o)),
            later_outcome: job.later_outcome.as_ref().map(|o| to_value(o)),
            score: job.latest_score().map(|s| to_value(s)),
            interventions: records
                .interventions_for(&id)
                .iter()
                .map(|i| format!("{} [{}] {}", i.id, i.category.as_str(), i.what))
                .collect(),
            commits,
            directory: id.dir(),
        })
    }
}

/// Whether a state is terminal.
pub fn is_terminal(state: OutcomeState) -> bool {
    !matches!(state, OutcomeState::Blocked)
}

mod erased {
    /// Serializes any record to a JSON value for the trace.
    pub trait ToValue {
        fn to_value(&self) -> serde_json::Value;
    }
    impl<T: serde::Serialize> ToValue for T {
        fn to_value(&self) -> serde_json::Value {
            serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
        }
    }
}
