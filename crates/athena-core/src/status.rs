//! `athena status`: the state of the work, derived from the records at `HEAD` and the state directory
//! (design §3.6). Nothing about the state of the work lives in a hand-maintained file (D35).

use crate::Athena;
use crate::manage::pending_quantities;
use crate::store::{JobRecords, JobState, Records};
use anyhow::Result;
use athena_data::records::{
    Build, Critique, CritiqueVerdict, InterventionSource, Invocation, JobPlan, OutcomeState,
    Reading,
};
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
    /// "plan accepted r<n>" or "plan stopped r<n>", for a job that went through job planning; absent otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    /// The job's priority: the latest priority record, else the request's words, else urgent if the reading
    /// gave an urgent reason, else normal (`JobRecords::priority`).
    pub priority: crate::plan::Priority,
}

/// How job planning ended for one job: the round of the last critique and whether it let the job go on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanState {
    pub accepted: bool,
    pub round: u32,
}

impl PlanState {
    /// "plan accepted r2" or "plan stopped r3".
    pub fn text(self) -> String {
        format!(
            "plan {} r{}",
            if self.accepted { "accepted" } else { "stopped" },
            self.round
        )
    }
}

/// The plan state of a job: that of its last critique. A critique that accepts stands unless it also says the
/// reading misread the request, which stops the job. A job with no critique has no plan state.
pub fn plan_state(job: &JobRecords) -> Option<PlanState> {
    let last = job.critiques.last()?;
    Some(PlanState {
        accepted: last.final_verdict == CritiqueVerdict::Accept
            && last.request_reading_mismatch.is_none(),
        round: last.round,
    })
}

/// How many planned jobs the first planning round accepted and stopped.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct FirstRoundCounts {
    pub accepted: usize,
    pub stopped: usize,
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
    /// First-round accept and stop counts over planned jobs; absent when no job was planned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planning: Option<FirstRoundCounts>,
    pub interventions_by_category: BTreeMap<String, usize>,
    pub interventions_by_class: BTreeMap<String, usize>,
    /// Mistakes per class and per cause, with the prevention proposed. Apart from `interventions_by_category`,
    /// which counts each mistake under its own category as before.
    pub mistakes: MistakeReport,
    /// Interventions that call for follow-up and that no later request job cites yet (see
    /// [`open_interventions`]). They are not in `open`, which lists jobs.
    pub open_interventions: Vec<OpenIntervention>,
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
    /// The latest audience rating (`crate::audience`), if one is recorded.
    pub audience: Option<AudienceView>,
    /// The latest audit of a job's outcome (`crate::auditor`), if one is recorded.
    pub audit: Option<AuditView>,
    pub invalid_records: Vec<String>,
    pub next: Vec<String>,
    /// The reconciliation that ran before this view was built (`Athena::status_with_reconcile`); absent from
    /// [`Athena::status`], which asks nothing of the remote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reconcile: Option<crate::reconcile::ReconcileReport>,
}

/// An intervention that still calls for follow-up, as `athena status` lists it under open work.
#[derive(Debug, Clone, Serialize)]
pub struct OpenIntervention {
    pub id: String,
    pub what: String,
    pub category: String,
    /// The job the intervention was recorded against, if any.
    pub job: Option<String>,
    /// The command that starts the follow-up.
    pub next: String,
}

/// The interventions that call for follow-up: those recorded by the user (`source` `user`) that no later
/// request job cites. A request job cites an intervention when its request text names the intervention's
/// record id and it was received at or after the intervention was recorded. Interventions Athena observed
/// itself (a correction of a reading) are already applied to the job that took them, so they are not listed.
/// Oldest first.
pub fn open_interventions(records: &Records) -> Vec<OpenIntervention> {
    records
        .interventions
        .iter()
        .filter(|i| i.source == InterventionSource::User)
        .filter(|i| {
            !records.request_jobs().iter().any(|job| {
                job.request
                    .as_ref()
                    .is_some_and(|r| r.received_at >= i.at && r.text.contains(i.id.as_str()))
            })
        })
        .map(|i| OpenIntervention {
            id: i.id.to_string(),
            what: i.what.clone(),
            category: i.category.as_str().into(),
            job: i.job.as_ref().map(|j| j.as_str().to_string()),
            next: format!(
                "athena ask \"<request that addresses intervention {}>\"",
                i.id
            ),
        })
        .collect()
}

/// The cause text under which a mistake is counted when no recorded introspection gives one.
pub const NO_CAUSE: &str = "no cause recorded";

/// Prevention work proposed for the mistakes that share one cause.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MistakeProposal {
    pub cause: String,
    /// `check` (a code check), `knowledge` (a default the readers are given) or `design change`.
    pub kind: String,
    /// What would have prevented the mistakes of this cause.
    pub prevention: String,
    /// The introspections that propose it.
    pub introspections: Vec<String>,
    /// How many mistakes it would have prevented.
    pub mistakes: usize,
}

/// The mistakes in the records: counted per class of work and per cause, apart from the intervention
/// categories. A mistake is an intervention for which [`athena_data::records::is_mistake`] holds: category
/// `mistake`, or a legacy record with category `other` whose text starts "Mistake:". Records are not rewritten.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct MistakeReport {
    pub total: usize,
    pub by_class: BTreeMap<String, usize>,
    /// The cause the recorded introspection gives, grouped by [`crate::introspect::normalise`]; a mistake with
    /// no recorded introspection counts under [`NO_CAUSE`].
    pub by_cause: BTreeMap<String, usize>,
    /// One proposal per cause.
    pub proposals: Vec<MistakeProposal>,
}

/// Counts the mistakes in `records` and gathers the prevention proposed for them, one proposal per cause.
pub fn mistake_report(records: &Records) -> MistakeReport {
    use athena_data::records::{IntrospectionStatus, PreventionKind};
    let mut report = MistakeReport::default();
    // Normalised cause -> (display text, proposal).
    let mut groups: BTreeMap<String, MistakeProposal> = BTreeMap::new();
    for i in records.interventions.iter().filter(|i| i.is_mistake()) {
        report.total += 1;
        *report.by_class.entry(i.class_of_work.clone()).or_default() += 1;
        let recorded = records
            .introspections
            .iter()
            .rfind(|n| n.intervention.as_ref() == Some(&i.id))
            .filter(|n| n.status == IntrospectionStatus::Recorded);
        let Some(n) = recorded else {
            *report.by_cause.entry(NO_CAUSE.into()).or_default() += 1;
            continue;
        };
        let cause = n.cause.as_deref().unwrap_or("").trim();
        let key = crate::introspect::normalise(cause);
        if key.is_empty() {
            *report.by_cause.entry(NO_CAUSE.into()).or_default() += 1;
            continue;
        }
        let proposal = groups.entry(key).or_insert_with(|| MistakeProposal {
            cause: cause.to_string(),
            kind: match n.prevention_kind {
                Some(PreventionKind::CodeCheck) => "check",
                Some(PreventionKind::Default) => "knowledge",
                Some(PreventionKind::DesignChange) => "design change",
                None => "unknown",
            }
            .into(),
            prevention: n.prevention.clone().unwrap_or_default(),
            introspections: Vec::new(),
            mistakes: 0,
        });
        proposal.mistakes += 1;
        let id = n.id.to_string();
        if !proposal.introspections.contains(&id) {
            proposal.introspections.push(id);
        }
    }
    for p in groups.into_values() {
        *report.by_cause.entry(p.cause.clone()).or_default() += p.mistakes;
        report.proposals.push(p);
    }
    report
}

/// The latest audience rating as `athena status` lists it.
#[derive(Debug, Clone, Serialize)]
pub struct AudienceView {
    pub id: String,
    pub target: String,
    pub audience: String,
    pub rating: u8,
    pub at: String,
    /// Each finding with the evidence it cites.
    pub findings: Vec<String>,
}

fn audience_view(records: &Records) -> Option<AudienceView> {
    crate::audience::latest(records).map(|r| AudienceView {
        id: r.id.to_string(),
        target: r.target.clone(),
        audience: r.audience.clone(),
        rating: r.rating,
        at: r.at.to_string(),
        findings: r
            .findings
            .iter()
            .map(|f| format!("{} (evidence: {})", f.text, f.source))
            .collect(),
    })
}

/// The latest audit as `athena status` lists it.
#[derive(Debug, Clone, Serialize)]
pub struct AuditView {
    pub id: String,
    /// The job whose outcome was judged.
    pub job: String,
    pub summary: String,
    pub at: String,
    /// Each finding with the evidence it cites.
    pub findings: Vec<String>,
}

fn audit_view(records: &Records) -> Option<AuditView> {
    records.audits.last().map(|a| AuditView {
        id: a.id.to_string(),
        job: a.target.as_str().to_string(),
        summary: a.summary.clone(),
        at: a.at.to_string(),
        findings: a
            .findings
            .iter()
            .map(|f| format!("{} (evidence: {})", f.text, f.evidence))
            .collect(),
    })
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
    /// The job's plan state, plans and critiques by round; all absent for a job that was not planned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub plans: Vec<JobPlan>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub critiques: Vec<Critique>,
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
        plan: plan_state(job).map(PlanState::text),
        priority: job.priority(),
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
    /// Whether a process of the registry still works on `job`.
    pub fn running(&self, job: &str) -> bool {
        self.state.registry().entries().iter().any(|e| e.job == job)
    }

    fn state_name(&self, job: &JobRecords) -> String {
        match job.state() {
            JobState::Dispatched if self.running(job.id().as_str()) => "running".into(),
            JobState::Dispatched => "interrupted".into(),
            other => other.name().into(),
        }
    }

    /// The mistakes in the records at `HEAD`, per class and per cause, with the prevention proposed.
    pub fn mistakes(&self) -> Result<MistakeReport> {
        let head = self.head()?;
        Ok(mistake_report(&Records::load(&self.git, &head)?))
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
        let mut first_round: Option<FirstRoundCounts> = None;
        for job in records.request_jobs() {
            if let Some(first) = job.critiques.first()
                && let Some(state) = plan_state(job)
            {
                let counts = first_round.get_or_insert(FirstRoundCounts {
                    accepted: 0,
                    stopped: 0,
                });
                let accepted = first.final_verdict == CritiqueVerdict::Accept
                    && first.request_reading_mismatch.is_none();
                // A round-1 revise sends the plan back; only a job whose planning ended in round 1 stopped there.
                if accepted {
                    counts.accepted += 1;
                } else if !state.accepted && state.round == 1 {
                    counts.stopped += 1;
                }
            }
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
                "interrupted" => next.push(format!("athena retry {id}   # reads it again to resume · or: athena close {id} --why \"...\" (commits what the interrupted job left)")),
                "blocked" => next.push(format!("athena go {id}   # after resolving local edits; or: athena close {id} --why \"...\"")),
                _ => {}
            }
        }
        let open_interventions = open_interventions(&records);
        for i in &open_interventions {
            next.push(format!(
                "{}   # follow up intervention {} ({})",
                i.next, i.id, i.category
            ));
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
            planning: first_round,
            interventions_by_category: by_category,
            interventions_by_class: by_class,
            mistakes: mistake_report(&records),
            open_interventions,
            restarts: records.restarts.len(),
            forecasts,
            pending,
            unreviewed_introspections: unreviewed.len(),
            proposals,
            work_proposals,
            audience: audience_view(&records),
            audit: audit_view(&records),
            invalid_records: records.invalid.clone(),
            next,
            reconcile: None,
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
            plan: plan_state(job).map(PlanState::text),
            plans: job.plans.clone(),
            critiques: job.critiques.clone(),
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

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod mistake_tests {
    use super::*;
    use athena_data::RecordId;
    use athena_data::records::{
        Intervention, InterventionCategory, IntrospectionRecord, IntrospectionStatus,
        IntrospectionTrigger, PreventionKind,
    };
    use jiff::Timestamp;

    fn at() -> Timestamp {
        "2026-10-17T12:00:00Z".parse().unwrap()
    }

    fn rid(n: u32) -> RecordId {
        RecordId::parse(&format!("20261017-1200{n:02}-1abc")).unwrap()
    }

    fn intervention(
        n: u32,
        class: &str,
        category: InterventionCategory,
        what: &str,
    ) -> Intervention {
        Intervention {
            id: rid(n),
            at: at(),
            what: what.into(),
            why: "why".into(),
            class_of_work: class.into(),
            category,
            job: None,
            source: InterventionSource::User,
            with_staged_changes: false,
        }
    }

    fn introspection(
        n: u32,
        of: u32,
        cause: &str,
        kind: PreventionKind,
        prevention: &str,
    ) -> IntrospectionRecord {
        IntrospectionRecord {
            id: rid(n),
            at: at(),
            trigger: IntrospectionTrigger::Intervention,
            job: None,
            intervention: Some(rid(of)),
            class_of_work: "docs".into(),
            category: "mistake".into(),
            what_happened: Some("it happened".into()),
            failure_class: None,
            new_class: false,
            earlier_cases: Vec::new(),
            decisions_matched: Vec::new(),
            cause: Some(cause.into()),
            prevention_kind: Some(kind),
            prevention: Some(prevention.into()),
            proposed_request: Some("do it".into()),
            evidence: None,
            status: IntrospectionStatus::Recorded,
            failure: None,
            flags: Vec::new(),
            dropped_ids: Vec::new(),
            from_jev: false,
            jev: None,
        }
    }

    #[test]
    fn mistakes_count_per_class_and_per_cause_and_same_cause_gives_one_proposal() {
        use InterventionCategory::Mistake;
        let mut records = Records::default();
        records.interventions = vec![
            intervention(1, "docs", Mistake, "skipped the check"),
            intervention(2, "docs", Mistake, "skipped it again"),
            intervention(3, "code", Mistake, "wrong file"),
            intervention(4, "code", Mistake, "no introspection"),
            intervention(5, "code", InterventionCategory::CorrectedOutput, "not one"),
        ];
        records.introspections = vec![
            introspection(
                11,
                1,
                "Nothing runs the docs test.",
                PreventionKind::CodeCheck,
                "Run it.",
            ),
            introspection(
                12,
                2,
                "nothing runs the docs test",
                PreventionKind::CodeCheck,
                "Run it.",
            ),
            introspection(
                13,
                3,
                "The brief omits file names.",
                PreventionKind::Default,
                "Name files.",
            ),
        ];
        let report = mistake_report(&records);
        assert_eq!(report.total, 4);
        assert_eq!(report.by_class["docs"], 2);
        assert_eq!(report.by_class["code"], 2);
        assert_eq!(report.by_cause["Nothing runs the docs test."], 2);
        assert_eq!(report.by_cause["The brief omits file names."], 1);
        assert_eq!(report.by_cause[NO_CAUSE], 1);
        assert_eq!(report.proposals.len(), 2, "same cause, one proposal");
        let shared = report
            .proposals
            .iter()
            .find(|p| p.mistakes == 2)
            .expect("the shared cause");
        assert_eq!(shared.kind, "check");
        assert_eq!(shared.introspections.len(), 2);
        let other = report.proposals.iter().find(|p| p.mistakes == 1).unwrap();
        assert_eq!(other.kind, "knowledge");
        assert_eq!(other.prevention, "Name files.");
    }

    #[test]
    fn a_legacy_other_record_starting_mistake_colon_is_counted_and_not_rewritten() {
        use InterventionCategory::Other;
        let mut records = Records::default();
        records.interventions = vec![
            intervention(1, "docs", Other, "Mistake: forgot the changelog"),
            intervention(2, "docs", Other, "A mistake: not counted"),
        ];
        let before = records.interventions.clone();
        let report = mistake_report(&records);
        assert_eq!(report.total, 1);
        assert_eq!(report.by_class["docs"], 1);
        assert_eq!(records.interventions, before, "the records are unchanged");
        assert_eq!(records.interventions[0].category, Other);
    }

    #[test]
    fn a_failed_introspection_gives_no_cause() {
        let mut records = Records::default();
        records.interventions = vec![intervention(1, "docs", InterventionCategory::Mistake, "x")];
        let mut n = introspection(11, 1, "c", PreventionKind::Default, "p");
        n.status = IntrospectionStatus::Failed;
        n.failure = Some("no".into());
        records.introspections = vec![n];
        let report = mistake_report(&records);
        assert_eq!(report.by_cause[NO_CAUSE], 1);
        assert!(report.proposals.is_empty());
    }

    #[test]
    fn the_report_is_empty_with_no_mistakes() {
        assert_eq!(mistake_report(&Records::default()).total, 0);
    }
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
