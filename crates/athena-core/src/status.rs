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
    /// Introspections no class-label record names yet (`crate::labeled::unreviewed`); `athena review` clears
    /// them one at a time.
    pub unreviewed_introspections: usize,
    /// Open defaults proposals (`athena.defaults-proposal/1`): pending ones only.
    pub proposals: Vec<ProposalView>,
    /// Open steward proposals (work items of kind proposal with no decision), decided by id with
    /// `athena approve-work` or `athena reject-work`.
    pub work_proposals: Vec<WorkProposalView>,
    pub invalid_records: Vec<String>,
    pub next: Vec<String>,
}

/// An open defaults proposal as `athena status` lists it (`defaults-loop.md` section 9).
#[derive(Debug, Clone, Serialize)]
pub struct ProposalView {
    pub id: String,
    pub kind: String,
    pub statement: String,
    pub applies_to: Vec<String>,
    pub version_from: u32,
    pub version_to: u32,
    /// The failure class's name and id.
    pub failure_class: String,
    /// The proposal record that holds it.
    pub record: String,
    /// The introspections it cites, each with its job.
    pub sources: Vec<String>,
}

/// An open steward proposal as `athena status` lists it.
#[derive(Debug, Clone, Serialize)]
pub struct WorkProposalView {
    pub id: String,
    pub title: String,
    pub rung: String,
    /// The source and the evidence, each as `kind value`.
    pub source: String,
    pub evidence: Vec<String>,
}

fn work_proposal_views(records: &Records) -> Vec<WorkProposalView> {
    let show =
        |r: &athena_data::records::Reference| format!("{:?} {}", r.kind, r.value).to_lowercase();
    crate::work_decisions::open_proposals(records)
        .into_iter()
        .map(|w| WorkProposalView {
            id: w.id.to_string(),
            title: w.title.clone(),
            rung: w.rung.as_str().into(),
            source: show(&w.source),
            evidence: w.evidence.iter().map(show).collect(),
        })
        .collect()
}

fn proposal_views(records: &Records) -> Vec<ProposalView> {
    crate::proposals::open(&records.proposals)
        .into_iter()
        .map(|p| ProposalView {
            id: p.id.clone(),
            kind: p.kind.as_str().into(),
            statement: p.statement.clone(),
            applies_to: p.applies_to.clone(),
            version_from: p.version.from,
            version_to: p.version.to,
            failure_class: format!(
                "{} ({})",
                records
                    .classes
                    .iter()
                    .find(|c| c.id == p.failure_class)
                    .map_or("unknown class", |c| c.name.as_str()),
                p.failure_class
            ),
            record: p.record.to_string(),
            sources: p
                .sources
                .iter()
                .map(|s| format!("introspection {} (job {})", s.introspection, s.job))
                .collect(),
        })
        .collect()
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

/// An introspection as `athena status <job>` shows it (`defaults-loop.md` section 3.2).
#[derive(Debug, Clone, Serialize)]
pub struct IntrospectionView {
    pub id: String,
    pub trigger: String,
    pub intervention: Option<String>,
    pub what_happened: Option<String>,
    /// The class's name and id; `proposed` marks a class this introspection introduced.
    pub failure_class: Option<String>,
    pub new_class: bool,
    /// Earlier cases, each with its job where it has one.
    pub earlier_cases: Vec<String>,
    pub decisions_matched: Vec<String>,
    pub cause: Option<String>,
    pub prevention_kind: Option<String>,
    pub prevention: Option<String>,
    pub proposed_request: Option<String>,
    pub status: String,
    pub failure: Option<String>,
    pub flags: Vec<String>,
    pub dropped_ids: Vec<String>,
    /// False: the class and prevention kind came from a language model; Jev runs in shadow only (see `jev`).
    pub from_jev: bool,
    pub cost_usd: Option<f64>,
    /// Jev's shadow answers, when it was asked. They decide nothing.
    pub jev: Option<athena_data::records::IntrospectionJev>,
}

/// The introspections of one job, oldest first, with class names and the jobs of earlier cases.
fn introspection_views(records: &Records, job: &athena_data::JobId) -> Vec<IntrospectionView> {
    let class_name = |id: &athena_data::RecordId| {
        let name = records
            .classes
            .iter()
            .find(|c| &c.id == id)
            .map_or("unknown class", |c| c.name.as_str());
        format!("{name} ({id})")
    };
    records
        .introspections
        .iter()
        .filter(|i| i.job.as_ref() == Some(job))
        .map(|i| IntrospectionView {
            id: i.id.to_string(),
            trigger: i.trigger.as_str().into(),
            intervention: i.intervention.as_ref().map(ToString::to_string),
            what_happened: i.what_happened.clone(),
            failure_class: i.failure_class.as_ref().map(class_name),
            new_class: i.new_class,
            earlier_cases: i
                .earlier_cases
                .iter()
                .map(|case| {
                    let job = records
                        .introspections
                        .iter()
                        .find(|c| &c.id == case)
                        .and_then(|c| c.job.as_ref());
                    match job {
                        Some(job) => format!("{case} (job {job})"),
                        None => case.to_string(),
                    }
                })
                .collect(),
            decisions_matched: i.decisions_matched.clone(),
            cause: i.cause.clone(),
            prevention_kind: i.prevention_kind.map(|k| k.as_str().into()),
            prevention: i.prevention.clone(),
            proposed_request: i.proposed_request.clone(),
            status: i.status.as_str().into(),
            failure: i.failure.clone(),
            flags: i.flags.clone(),
            dropped_ids: i.dropped_ids.clone(),
            from_jev: i.from_jev,
            cost_usd: i.evidence.as_ref().map(|e| e.cost_usd),
            jev: i.jev.clone(),
        })
        .collect()
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
    /// What went wrong or was changed, and why the system allowed it (`athena.introspection/1`).
    pub introspections: Vec<IntrospectionView>,
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

/// A number without float noise: at most six decimals, trailing zeros removed.
fn number(v: f64) -> String {
    let text = format!("{v:.6}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text.is_empty() {
        "0".into()
    } else {
        text.to_string()
    }
}

fn quantity_text(q: &athena_data::Quantity) -> String {
    match (q.value(), q.lower(), q.upper()) {
        (Some(v), _, _) => {
            format!("{} {} ({:?})", number(v), q.unit(), q.provenance()).to_lowercase()
        }
        (None, l, u) => format!(
            "[{}, {}] {} ({:?})",
            l.map_or("?".into(), number),
            u.map_or("?".into(), number),
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
            // Queued requests have not run; they show under open work.
            if recent.len() < 10 && state != "queued" {
                recent.push(job_line(job, &state));
            }
            if matches!(
                state.as_str(),
                "queued" | "proposed" | "declined" | "interrupted" | "running" | "blocked"
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
                "queued" => next.push(format!("athena go {id}   # reads it first, then asks to run it · or: athena close {id} --why \"...\"")),
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
        let proposals = proposal_views(&records);
        for p in &proposals {
            let id = &p.id;
            next.push(format!("athena accept-proposal {id}   # runs a job that edits defaults.toml · or: athena reject-proposal {id} --why \"...\""));
        }
        let work_proposals = work_proposal_views(&records);
        for w in &work_proposals {
            let id = &w.id;
            next.push(format!("athena approve-work {id}   # submits a job that carries the proposal's cited content · or: athena reject-work {id} --why \"...\""));
        }
        let unreviewed = crate::labeled::unreviewed(&records.introspections, &records.class_labels);
        if let Some(oldest) = unreviewed.first() {
            next.push(format!(
                "athena review {}   # {} introspection(s) have no class label; one of: --right · --class <id> · --new-class <name> --description \"...\"",
                oldest.id,
                unreviewed.len()
            ));
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
            unreviewed_introspections: unreviewed.len(),
            proposals,
            work_proposals,
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
            introspections: introspection_views(&records, &id),
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
