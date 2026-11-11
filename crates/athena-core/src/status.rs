//! `athena status`: the state of the work, derived from the records at `HEAD` and the state directory
//! (design §3.6). Nothing about the state of the work lives in a hand-maintained file (D35).

use crate::Athena;
use crate::manage::pending_quantities;
use crate::state::format_bytes;
use crate::store::{JobRecords, JobState, Records};
use anyhow::Result;
use athena_data::records::{
    Build, BuildMeasurement, Critique, CritiqueVerdict, InterventionSource, Invocation, JobPlan,
    OutcomeState, Reading, Role, SccacheFigures, text_cites_job,
};
use athena_gate::invariant::{BOOTSTRAP_OUTCOME, BootstrapRange, bootstrap_range};
use athena_git::Git;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

mod planning;
mod throughput;

pub use planning::*;
pub use throughput::*;

/// The paths the installed binary is built from: Athena's own code and the files that build and install it. A
/// change to one of them after the build commit leaves the installed binary stale.
///
/// The CI `athena-bin` cache key hashes the same files, so it mirrors this list by hand. The mirrors registry
/// (`crate::mirrors`) compares the two both ways and names the side to fix, so edit them together.
pub const BUILD_PATHS: [&str; 9] = [
    "crates/",
    "xtask/",
    "rust-toolchain.toml",
    "Cargo.toml",
    "Cargo.lock",
    "athena.toml",
    "defaults.toml",
    "setup.sh",
    "docs/system-facts.md",
];

/// Whether any of `paths` is one the installed binary is built from.
pub fn touches_build(paths: &[String]) -> bool {
    paths.iter().any(|p| {
        BUILD_PATHS
            .iter()
            .any(|root| p == root || (root.ends_with('/') && p.starts_with(root)))
    })
}

/// The one line that says the binary built from `build_commit` is older than `head` for [`BUILD_PATHS`], or
/// None when it is not: the build commit is not in this repository (a build that names no commit, or another
/// repository's) or is not an ancestor of `head`, or none of those paths changed since it. Changes to
/// anything else, such as `docs/` or `records/`, never make a build stale.
pub fn stale_build_line(git: &Git, build_commit: &str, head: &str) -> Option<String> {
    if !git.commit_exists(build_commit).ok()? {
        return None;
    }
    git.run(["merge-base", "--is-ancestor", build_commit, head])
        .ok()?;
    let mut args = vec!["diff", "--name-only", build_commit, head, "--"];
    args.extend(BUILD_PATHS);
    let changed = git
        .run(args)
        .ok()?
        .lines()
        .filter(|l| !l.is_empty())
        .count();
    if changed == 0 {
        return None;
    }
    let short = |c: &str| c[..c.len().min(12)].to_string();
    Some(format!(
        "stale build: this athena was built from {}, and {changed} file(s) under {} changed by {}; run ./setup.sh",
        short(build_commit),
        BUILD_PATHS.join(", "),
        short(head)
    ))
}

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
    /// "environment: low disk: 3.0 GB free, 10.0 GB required" for a proposed job whose dispatch was held for
    /// lack of disk space, or for an interrupted job held for it after its candidate was built; absent
    /// otherwise. A proposed job is still queued to run, not rejected; a held interrupted job is resumed from
    /// its kept candidate by the next drain.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<String>,
    /// The id of the retry that stands for this job, for an interrupted job that was already retried
    /// ([`crate::reading::retries_by_job`]); absent otherwise. `athena retry` refuses such a job.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retried_as: Option<String>,
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

/// How many of the most recent first readings the per-rule refusal counts cover.
pub const READING_WINDOW: usize = 50;

/// What one reading rule refused in the window ([`ReadingRuleStats`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RuleRefusals {
    /// The name of the rule: its check function, or `check_reading_size`, or `other` for reader output that
    /// was not a valid reading.
    pub rule: String,
    /// First readings this rule refused.
    pub first_refused: usize,
    /// Second readings (the reading made again with a refusal as its correction) this rule refused.
    pub second_refused: usize,
    /// The refusal text this rule gave most often to a first reading; a tie goes to the most recent.
    pub top_text: String,
    /// How many first readings got `top_text`.
    pub top_text_count: usize,
}

/// How often each reading rule refused a reading, over the last [`READING_WINDOW`] first readings. A refused
/// first reading is read again once, so a first refusal costs a reader call, and a second refusal fails the
/// request. A reading is refused by the first rule that refuses it, so each refusal is counted under exactly one
/// rule, and a rule that comes later in the order is hidden whenever an earlier one refuses the same reading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReadingRuleStats {
    /// First readings in the window.
    pub first_readings: usize,
    /// Of those, the ones that were read again (the first was refused and the reader ran a second time).
    pub second_readings: usize,
    /// Rules that refused at least one reading in the window, most first refusals first, then by name.
    pub rules: Vec<RuleRefusals>,
}

impl ReadingRuleStats {
    /// The stats over the newest [`READING_WINDOW`] first readings (reader invocations recorded as attempt 1,
    /// ordered by start time, then job id and sequence), each paired with the reading made again in the same
    /// job at the next sequence number. `None` when no reader invocation has a recorded reading attempt, which
    /// is also the case for records written before the attempt was recorded.
    pub fn compute(records: &Records) -> Option<Self> {
        let attempt = |inv: &Invocation| inv.reading_attempt.as_ref().map(|a| a.attempt);
        let mut firsts: Vec<(&Invocation, Option<&Invocation>)> = Vec::new();
        for job in records.jobs.values() {
            let readers = || job.invocations.iter().filter(|i| i.role == Role::Reader);
            for first in readers().filter(|i| attempt(i) == Some(1)) {
                let second = readers().find(|i| i.seq == first.seq + 1 && attempt(i) == Some(2));
                firsts.push((first, second));
            }
        }
        firsts.sort_by(|(a, _), (b, _)| {
            (a.started_at, a.job.as_str(), a.seq).cmp(&(b.started_at, b.job.as_str(), b.seq))
        });
        let window = &firsts[firsts.len().saturating_sub(READING_WINDOW)..];
        if window.is_empty() {
            return None;
        }
        // Per rule: its row and how often each first-refusal text came, with the position of the latest.
        let mut rows: BTreeMap<String, RuleRow> = BTreeMap::new();
        let refusal_of =
            |inv: &Invocation| inv.reading_attempt.as_ref().and_then(|a| a.refusal.clone());
        let mut second_readings = 0;
        for (position, (first, second)) in window.iter().enumerate() {
            if let Some(refusal) = refusal_of(first) {
                let (counts, texts) = rows
                    .entry(refusal.rule.clone())
                    .or_insert_with(|| (empty_row(&refusal.rule), BTreeMap::new()));
                counts.first_refused += 1;
                let seen = texts.entry(refusal.text).or_insert((0, position));
                seen.0 += 1;
                seen.1 = position;
            }
            let Some(second) = second else { continue };
            second_readings += 1;
            if let Some(refusal) = refusal_of(second) {
                let (counts, _) = rows
                    .entry(refusal.rule.clone())
                    .or_insert_with(|| (empty_row(&refusal.rule), BTreeMap::new()));
                counts.second_refused += 1;
            }
        }
        let mut rules: Vec<RuleRefusals> = rows
            .into_values()
            .map(|(mut counts, texts)| {
                if let Some((text, (count, _))) = texts
                    .into_iter()
                    .max_by_key(|(_, (count, latest))| (*count, *latest))
                {
                    counts.top_text = text;
                    counts.top_text_count = count;
                }
                counts
            })
            .collect();
        rules.sort_by(|a, b| {
            b.first_refused
                .cmp(&a.first_refused)
                .then_with(|| a.rule.cmp(&b.rule))
        });
        Some(Self {
            first_readings: window.len(),
            second_readings,
            rules,
        })
    }

    /// The lines `athena status` prints: the window, then one row for each rule that refused a reading with its
    /// first-refusal and second-refusal counts. A window where nothing was refused says so.
    pub fn text(&self) -> String {
        let mut out = format!(
            "readings    last {} first readings ({} read again): refused per rule, first / second\n",
            self.first_readings, self.second_readings
        );
        if self.rules.is_empty() {
            out.push_str("            no rule refused a reading in this window\n");
        }
        for r in &self.rules {
            out.push_str(&format!(
                "            {}: {} first · {} second\n",
                r.rule, r.first_refused, r.second_refused
            ));
        }
        out
    }
}

/// A rule's row while counting, with how often each first-refusal text came and the window position of its
/// latest occurrence.
type RuleRow = (RuleRefusals, BTreeMap<String, (usize, usize)>);

/// A rule's row before anything is counted under it.
fn empty_row(rule: &str) -> RuleRefusals {
    RuleRefusals {
        rule: rule.to_string(),
        first_refused: 0,
        second_refused: 0,
        top_text: String::new(),
        top_text_count: 0,
    }
}

/// Where a requirement's measure can appear in `athena status`: the label `athena status` prints at the start
/// of a line, and the [`StatusView`] field that holds what it prints. A measure in `docs/spec.toml` ends with
/// `status: <label>` using one of these, or with `status: not shown` and names the test or check that shows
/// it. The Hestia spec pass flags a measure whose label is not here, and the `xtask` spec test requires the
/// marker.
pub const STATUS_MEASURE_LABELS: &[(&str, &str)] = &[
    ("throughput", "throughput"),
    ("trend", "class_trend"),
    ("checks", "check_durations"),
    ("planning", "planning_cost"),
    ("keep rule", "keep_rule"),
    ("producers", "producer_settings"),
    ("specialization", "specialization"),
    ("mistakes", "mistakes"),
    ("repeated", "repeated_causes"),
    ("code health", "code_health"),
    ("audit", "audit"),
    ("audience", "audience"),
    ("requirement", "spec_gaps"),
    ("readings", "reading_rules"),
];

/// The line `athena status` prints for the latest build measurement (`athena measure-build`): T (one cold
/// target directory), S (one worktree's sources), the wall time of each cold build and the compile-cache
/// counts, with the commit and time it was taken. With no measurement it says so and how to make one.
pub fn build_measurement_line(latest: Option<&BuildMeasurement>) -> String {
    let Some(m) = latest else {
        return "build      measurement: none yet (run `athena measure-build`, or the Hestia sweep makes one)"
            .into();
    };
    let cache = match m.sccache {
        SccacheFigures::NotInstalled => "sccache not installed".to_string(),
        SccacheFigures::Measured { first, second } => format!(
            "sccache build 1 {} hits / {} misses, build 2 {} hits / {} misses",
            first.hits, first.misses, second.hits, second.misses
        ),
    };
    // Whole-build totals over every process of the build, not the figures of one process; absent in old
    // records.
    let peaks = if m.first_build_peak.is_none() && m.second_build_peak.is_none() {
        String::new()
    } else {
        let memory = |p: Option<athena_data::records::BuildPeak>| {
            p.map_or("n/a".to_string(), |p| format!("{:.2} GB", p.memory_gb))
        };
        let processes = |p: Option<athena_data::records::BuildPeak>| {
            p.map_or("n/a".to_string(), |p| p.processes.to_string())
        };
        format!(
            " · build total peak memory build 1 {}, build 2 {} · peak concurrent processes build 1 {}, build 2 {}",
            memory(m.first_build_peak),
            memory(m.second_build_peak),
            processes(m.first_build_peak),
            processes(m.second_build_peak)
        )
    };
    format!(
        "build      measurement: T {} target (one cold target directory) · S {} source (one worktree) · build 1 {:.2}s, build 2 {:.2}s{peaks} · {cache} · commit {} at {}",
        format_bytes(m.target_bytes),
        format_bytes(m.source_bytes),
        m.first_build_seconds,
        m.second_build_seconds,
        &m.commit[..m.commit.len().min(12)],
        m.at
    )
}

/// The state of the work.
#[derive(Debug, Clone, Serialize)]
pub struct StatusView {
    pub repository: String,
    pub branch: Option<String>,
    pub head: String,
    pub athena: Build,
    /// Set when the installed binary's build commit is older than `head` for the paths it is built from
    /// ([`stale_build_line`]): the line `athena status` and the agent-running commands print.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_build: Option<String>,
    pub driver: String,
    pub kill_switch_on: bool,
    pub nested: Option<String>,
    /// The current focus (`athena focus`), in the words it was set in; absent until one is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus: Option<String>,
    pub bootstrap: Bootstrap,
    pub jobs_by_state: BTreeMap<String, usize>,
    pub recent: Vec<JobLine>,
    pub open: Vec<JobLine>,
    /// First-round accept and stop counts over planned jobs; absent when no job was planned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planning: Option<FirstRoundCounts>,
    /// Planning rounds and cost per landed job, before and after the first resumed planning; absent when no
    /// planned job landed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planning_cost: Option<PlanningCost>,
    /// The job-planning keep rule (job-planning.md section 9, D110) over planned dispatched jobs; absent when
    /// no job was planned and dispatched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_rule: Option<KeepRule>,
    /// First-pass verification rate, landed share and cost per landed job by producer model and effort; empty
    /// when no dispatched job ran a producer.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub producer_settings: Vec<ProducerSettingStats>,
    /// Per class of work, whether a non-default producer setting beat the class's most-used one; empty when no
    /// dispatched job ran a producer.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub specialization: Vec<ClassSpecialization>,
    /// Median and slowest `test` check duration over the newest jobs, against the check time limit; absent
    /// when no job ran a measured test check or the configuration cannot be read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_durations: Option<CheckDurations>,
    /// Per gate (a failed check or an admission rule): the jobs it ended and how often a later try landed the
    /// same change (`crate::gate_counts`). The most overruled gate comes first; absent when no gate has ended
    /// a job.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gate_counts: Vec<crate::gate_counts::GateCount>,
    /// Per reading rule: the first readings it refused and the second readings it refused, over the last
    /// [`READING_WINDOW`] first readings. Absent when no reader invocation has recorded a reading attempt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reading_rules: Option<ReadingRuleStats>,
    /// Hestia's recurring duties: each with its cadence, last run, next due point and whether it is overdue
    /// (`crate::hestia::DUTIES`, `docs/design/hestia.md` section 13).
    pub duties: Vec<crate::hestia::DutyStanding>,
    /// Landed jobs per day, median minutes per landed job by stage, idle time between jobs and time spent on
    /// jobs that did not land, computed from the job records; absent until a request job has landed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub throughput: Option<Throughput>,
    /// The latest build measurement as one printed line ([`build_measurement_line`]), or the line that says
    /// there is none yet.
    pub build_measurement: String,
    /// Classes of work whose minutes per landed job fell or rose between their older and newer landed jobs,
    /// with the job after which a rising class got worse; absent until a class has enough landed jobs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_trend: Option<crate::class_trend::ClassTrend>,
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
    /// The code-health table (`crate::code_health`); absent when the sources could not be measured.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_health: Option<crate::code_health::CodeHealth>,
    /// Lines of work the scope guard flagged or paused, with their signals T1 to T5 (`crate::scope_guard`).
    /// Pauses first. Empty when no mark file exists.
    pub lines: Vec<crate::scope_guard::LineMark>,
    /// The open repeated causes from Hestia's error review (`crate::hestia::repeated_causes`): causes that
    /// occurred in two or more distinct jobs and whose root-cause request has not landed, each with its count
    /// and the request that addresses it. Empty when there are none. Causes in a stage that `athena.toml` has
    /// switched off are not here; see `dormant_causes`.
    pub repeated_causes: Vec<crate::hestia::RepeatedCause>,
    /// The open repeated causes whose stage (planner or critic) `athena.toml` has switched off: listed apart,
    /// never queued by the error review backfill that Athena runs at drain start and in each sweep.
    pub dormant_causes: Vec<crate::hestia::DormantCause>,
    /// The active requirements of `docs/spec.toml` with no mechanism, or with a measure that is blank or names
    /// a label `athena status` does not print (`crate::hestia::spec_gaps`). Empty when every active requirement
    /// has both, or when the repository has no spec. Athena queues one request for each at the next drain or
    /// sweep.
    pub spec_gaps: Vec<crate::hestia::SpecGap>,
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
                Some(PreventionKind::Relax) => "relax a gate",
                Some(PreventionKind::Remove) => "remove a gate",
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
        hold: None,
        retried_as: None,
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

/// What one pass over the request jobs collects for [`StatusView`]: counts by state, the newest lines, the open
/// lines and the first planning round's outcome counts.
#[derive(Default)]
struct JobScan {
    jobs_by_state: BTreeMap<String, usize>,
    recent: Vec<JobLine>,
    open: Vec<JobLine>,
    first_round: Option<FirstRoundCounts>,
}

/// Interventions counted by category and by class of work, in that order.
fn intervention_counts(records: &Records) -> (BTreeMap<String, usize>, BTreeMap<String, usize>) {
    let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_class: BTreeMap<String, usize> = BTreeMap::new();
    for i in &records.interventions {
        *by_category.entry(i.category.as_str().into()).or_default() += 1;
        *by_class.entry(i.class_of_work.clone()).or_default() += 1;
    }
    (by_category, by_class)
}

/// The forecast headline of each class of work: the reader's and the reference's mean Brier score over the
/// latest score of every job of that class.
fn class_forecasts(records: &Records) -> Vec<ClassHeadline> {
    let mut per_class: BTreeMap<String, Vec<&athena_data::records::Score>> = BTreeMap::new();
    for job in records.request_jobs() {
        if let (Some(score), Some(class)) = (job.latest_score(), job.class()) {
            per_class.entry(class.to_string()).or_default().push(score);
        }
    }
    per_class
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
        .collect()
}

/// The quantities that are not known yet, each with the resolution record that settles it, if any.
fn pending_views(records: &Records, set: &athena_gate::records::RecordSet) -> Vec<Pending> {
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
    pending_quantities(set)
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
        .collect()
}

/// The bootstrap section of the status view for a range of unverified commits.
fn bootstrap_view(range: &BootstrapRange) -> Bootstrap {
    match range {
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
    }
}

/// The next-step lines for the defaults proposals and the work proposals waiting for a decision.
fn proposal_next_lines(
    proposals: &[ProposalView],
    work_proposals: &[WorkProposalView],
) -> Vec<String> {
    let mut next = Vec::new();
    for p in proposals {
        let id = &p.id;
        next.push(format!("athena accept-proposal {id}   # runs a job that edits defaults.toml · or: athena reject-proposal {id} --why \"...\""));
    }
    for w in work_proposals {
        let id = &w.id;
        next.push(format!("athena approve-work {id}   # submits a job that carries the proposal's cited content · or: athena reject-work {id} --why \"...\""));
    }
    next
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

    /// The stale-build line when this binary's build commit is older than `HEAD` for the paths it is built
    /// from, else None ([`stale_build_line`]). Never an error: a repository that cannot be read has no line.
    pub fn stale_build(&self) -> Option<String> {
        stale_build_line(&self.git, &self.build.source_commit, &self.head().ok()?)
    }

    /// The mistakes in the records at `HEAD`, per class and per cause, with the prevention proposed.
    pub fn mistakes(&self) -> Result<MistakeReport> {
        let head = self.head()?;
        Ok(mistake_report(&Records::load(&self.git, &head)?))
    }

    /// Walk every request job once: count jobs by state, keep the ten most recent non-queued lines and the open
    /// ones, and count the first planning round's outcomes. Marks the hold of a proposed or interrupted job.
    fn scan_jobs(&self, records: &Records) -> JobScan {
        let mut scan = JobScan::default();
        let retries = crate::reading::retries_by_job(records);
        for job in records.request_jobs() {
            if let Some(accepted) = first_round_accepted(job)
                && let Some(state) = plan_state(job)
            {
                let counts = scan.first_round.get_or_insert(FirstRoundCounts {
                    accepted: 0,
                    stopped: 0,
                });
                // A round-1 revise sends the plan back; only a job whose planning ended in round 1 stopped there.
                if accepted {
                    counts.accepted += 1;
                } else if !state.accepted && state.round == 1 {
                    counts.stopped += 1;
                }
            }
            let state = self.state_name(job);
            *scan.jobs_by_state.entry(state.clone()).or_default() += 1;
            let mut line = job_line(job, &state);
            if state == "interrupted" {
                line.retried_as = retries.get(job.id().as_str()).cloned();
            }
            // An interrupted job that was already retried is not resumed again, so its hold is not shown.
            if state == "proposed" || (state == "interrupted" && line.retried_as.is_none()) {
                line.hold = crate::disk::read_any_hold(&self.state, job.id()).map(|h| h.text());
            }
            // Queued requests have not run; they show under open work.
            if scan.recent.len() < 10 && state != "queued" {
                scan.recent.push(line.clone());
            }
            if matches!(
                state.as_str(),
                "queued" | "proposed" | "declined" | "interrupted" | "running" | "blocked"
            ) {
                scan.open.push(line);
            }
        }
        scan
    }

    /// The next-step lines for the open jobs, one per job that has a next step, in the order of `open`.
    fn open_job_next_lines(&self, records: &Records, open: &[JobLine]) -> Vec<String> {
        let mut next = Vec::new();
        for line in open {
            let id = &line.id;
            match line.state.as_str() {
                "queued" => next.push(format!("athena go {id}   # reads it first, then asks to run it · or: athena close {id} --why \"...\"")),
                "proposed" => next.push(format!("athena go {id}   # or: athena correct {id} \"<line>\" · athena close {id} --why \"...\"")),
                "declined" => next.push(format!("athena correct {id} \"<line>\"   # or: athena close {id} --why \"...\"")),
                "interrupted" => match &line.retried_as {
                    Some(new) => {
                        let state = records
                            .jobs
                            .get(new.as_str())
                            .map_or("unknown", |j| j.state().name());
                        let follow_up = crate::reading::retry_follow_up(
                            records,
                            new,
                            self.running(new),
                        );
                        next.push(format!("retried as {new} ({state})   # {follow_up} · or: athena close {id} --why \"...\" (commits what the interrupted job left)"));
                    }
                    None => next.push(format!("athena retry {id}   # reads it again to resume · or: athena close {id} --why \"...\" (commits what the interrupted job left)")),
                },
                "blocked" => next.push(format!("athena go {id}   # after resolving local edits; or: athena close {id} --why \"...\"")),
                _ => {}
            }
        }
        next
    }

    /// The state of the work.
    pub fn status(&self) -> Result<StatusView> {
        self.state.registry().reap();
        let head = self.head()?;
        // The records come from the `athena/records` worktree when it exists, else from `records/` at HEAD
        // (D111). The per-class forecasts below are computed from the same `records`.
        let worktree = athena_data::paths::records_worktree(self.state.root());
        let records = Records::load_current(&self.git, &head, &worktree)?;
        let set = athena_gate::records::RecordSet::load_current(&self.git, &head, &worktree)?;
        let scan = self.scan_jobs(&records);
        let (by_category, by_class) = intervention_counts(&records);
        let pending = pending_views(&records, &set);
        let bootstrap = bootstrap_view(&bootstrap_range(&self.git, &head)?);
        let kill_switch_on = self.state.registry().stopped();
        let mut next = Vec::new();
        if kill_switch_on {
            next.push("athena start   # the kill switch is on; new work is blocked".into());
        }
        next.extend(self.open_job_next_lines(&records, &scan.open));
        // Marks that no longer hold under the current attempts are not shown.
        let lines = self.live_marks(&records);
        for l in lines.iter().filter(|l| l.is_pause()) {
            next.push(format!(
                "line {} is paused ({})   # {}",
                l.line,
                l.signal_names(),
                l.lift
            ));
        }
        let open_causes: Vec<crate::hestia::RepeatedCause> =
            crate::hestia::repeated_causes(&records)
                .into_iter()
                .filter(crate::hestia::RepeatedCause::is_open)
                .collect();
        // The stages athena.toml switches off are read from the configuration at HEAD; an unreadable
        // configuration leaves every cause live.
        let job_config = crate::capability::JobConfig::at(&self.git, &head).ok();
        let disabled = job_config
            .as_ref()
            .map(|jc| crate::hestia::Stage::disabled(&jc.config))
            .unwrap_or_default();
        let check_durations = job_config
            .as_ref()
            .and_then(|jc| check_durations(&records, jc.config.budget.check_timeout_s));
        let (repeated_causes, dormant_causes) =
            crate::hestia::split_dormant(open_causes, &disabled);
        for c in repeated_causes.iter().filter(|c| c.request.is_none()) {
            next.push(format!(
                "repeated cause in {} jobs has no root-cause request   # {} · Athena queues it at the next drain or sweep (athena hestia review --backfill does it now)",
                c.jobs.len(),
                c.cause
            ));
        }
        // A spec that cannot be read or parsed lists nothing here; the sweep's spec pass reports it.
        let spec_gaps = crate::hestia::spec_gaps_at(&self.git, &head).unwrap_or_default();
        next.extend(spec_gaps.iter().map(|g| g.next_line()));
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
        let work_proposals = work_proposal_views(&records);
        next.extend(proposal_next_lines(&proposals, &work_proposals));
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
        let code_health = crate::code_health::compute(&self.git, &head, &records, crate::now())
            .ok()
            .map(|health| health.shown());
        let stale_build = stale_build_line(&self.git, &self.build.source_commit, &head);
        let duty_ctx = crate::hestia::DutyContext {
            occasion: crate::hestia::Occasion::Sweep,
            landings: crate::code_health::landings(&self.git, &head)?.len() as u32,
            now: crate::now(),
        };
        let duties = crate::hestia::duty_standings(
            &crate::hestia::DUTIES,
            &records.hestia_sweeps,
            &records.code_health_baselines,
            &duty_ctx,
        );
        Ok(StatusView {
            repository: self.git.dir().display().to_string(),
            branch: self.git.current_branch()?,
            head,
            athena: self.build.clone(),
            stale_build,
            driver: format!("{} {}", self.driver.info().name, self.driver.info().version),
            kill_switch_on,
            nested: self.nested.clone(),
            focus: records.current_focus().map(|f| f.text.clone()),
            bootstrap,
            jobs_by_state: scan.jobs_by_state,
            recent: scan.recent,
            open: scan.open,
            planning: scan.first_round,
            planning_cost: planning_cost(&records),
            keep_rule: keep_rule(&records),
            producer_settings: producer_settings(&records),
            specialization: specialization(&records),
            check_durations,
            gate_counts: crate::gate_counts::gate_counts(&records),
            reading_rules: ReadingRuleStats::compute(&records),
            duties,
            throughput: Throughput::compute(&records),
            build_measurement: build_measurement_line(BuildMeasurement::latest(
                &records.build_measurements,
            )),
            class_trend: crate::class_trend::ClassTrend::compute(&records),
            interventions_by_category: by_category,
            interventions_by_class: by_class,
            mistakes: mistake_report(&records),
            open_interventions,
            restarts: records.restarts.len(),
            forecasts: class_forecasts(&records),
            pending,
            unreviewed_introspections: unreviewed.len(),
            proposals,
            work_proposals,
            audience: audience_view(&records),
            audit: audit_view(&records),
            code_health,
            lines,
            repeated_causes,
            dormant_causes,
            spec_gaps,
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
        let mut commits: Vec<String> = self
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
        if crate::gh::cutover_job_for(&self.git, &head).is_some() {
            // After C0 (D111) commits carry no trailers: list those that touch the job's records, then the
            // host commit of the job's landing record, which is the squash commit on the default branch.
            let mut more = self
                .git
                .run(["log", "--format=%h %s", &head, "--", &id.dir()])?
                .lines()
                .map(str::to_string)
                .collect::<Vec<_>>();
            if let Ok(Some(host)) = athena_gate::attribution::cutover_commit(&self.git, id.as_str())
                && self.git.commit_exists(&host)?
            {
                let line = self
                    .git
                    .run(["log", "-1", "--format=%h %s", &host])?
                    .trim()
                    .to_string();
                more.push(line);
            }
            for line in more {
                if !commits.contains(&line) {
                    commits.push(line);
                }
            }
        }
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
            other_causes: Vec::new(),
            open_hypotheses: Vec::new(),
            contradictions: Vec::new(),
            dropped_ids: Vec::new(),
            planning_stopped: false,
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
    fn a_relax_or_remove_prevention_is_named_for_what_it_proposes() {
        use InterventionCategory::Mistake;
        let mut records = Records::default();
        records.interventions = vec![
            intervention(1, "docs", Mistake, "a"),
            intervention(2, "docs", Mistake, "b"),
        ];
        records.introspections = vec![
            introspection(
                11,
                1,
                "The fmt check is too strict.",
                PreventionKind::Relax,
                "[gate: fmt] Loosen it.",
            ),
            introspection(
                12,
                2,
                "The lint adds nothing.",
                PreventionKind::Remove,
                "[gate: lint] Drop it.",
            ),
        ];
        let report = mistake_report(&records);
        let kinds: BTreeMap<&str, &str> = report
            .proposals
            .iter()
            .map(|p| (p.cause.as_str(), p.kind.as_str()))
            .collect();
        assert_eq!(kinds["The fmt check is too strict."], "relax a gate");
        assert_eq!(kinds["The lint adds nothing."], "remove a gate");
    }

    #[test]
    fn the_report_is_empty_with_no_mistakes() {
        assert_eq!(mistake_report(&Records::default()).total, 0);
    }

    #[test]
    fn every_measure_label_names_a_status_view_field_and_a_printed_line() {
        let here = include_str!("status.rs");
        let display = include_str!("../../athena-cli/src/display.rs");
        // The `requirement` line is a next step built where the spec gaps are found.
        let hestia = include_str!("hestia.rs");
        for (label, field) in STATUS_MEASURE_LABELS {
            assert!(
                here.contains(&format!("    pub {field}:")),
                "StatusView has no field {field} for the label {label:?}"
            );
            let printed = format!("\"{label} ");
            let is_printed = [here, display, hestia]
                .iter()
                .any(|source| source.contains(&printed));
            assert!(
                is_printed,
                "`athena status` prints no line starting {label:?}"
            );
        }
    }
}

#[cfg(test)]
mod build_measurement_tests {
    use super::*;
    use crate::commits::{NewFile, message};
    use athena_data::RecordId;
    use athena_data::records::{AnyRecord, SccacheCounts};

    fn measurement(
        id: &str,
        hours_ago: i64,
        target: u64,
        sccache: SccacheFigures,
    ) -> BuildMeasurement {
        BuildMeasurement {
            id: RecordId::parse(id).unwrap(),
            at: crate::now()
                .checked_sub(jiff::SignedDuration::from_hours(hours_ago))
                .unwrap(),
            commit: "0123456789abcdef0123456789abcdef01234567".into(),
            cargo_lock_hash: "89abcdef0123456789abcdef0123456789abcdef".into(),
            target_bytes: target,
            source_bytes: 2_500_000,
            first_build_seconds: 61.5,
            second_build_seconds: 12.25,
            sccache,
            first_build_peak: None,
            second_build_peak: None,
        }
    }

    fn commit(athena: &Athena, m: &BuildMeasurement) {
        let record = AnyRecord::BuildMeasurement(Box::new(m.clone()));
        athena
            .commit_records(
                &[NewFile::record(&record)],
                &message(
                    "Build measurement",
                    "",
                    &[("Athena-Record", record.canonical_path())],
                ),
            )
            .unwrap();
    }

    #[test]
    fn the_line_shows_t_s_both_times_and_the_cache_counts_or_that_there_is_no_cache() {
        let measured = measurement(
            "20261029-090000-5abc",
            0,
            4_100_000_000,
            SccacheFigures::Measured {
                first: SccacheCounts { hits: 0, misses: 9 },
                second: SccacheCounts { hits: 4, misses: 5 },
            },
        );
        let line = build_measurement_line(Some(&measured));
        for want in [
            "T 4.1 GB",
            "S 2.5 MB",
            "build 1 61.50s",
            "build 2 12.25s",
            "sccache build 1 0 hits / 9 misses",
            "build 2 4 hits / 5 misses",
            "commit 0123456789ab",
        ] {
            assert!(line.contains(want), "{want:?} not in {line}");
        }
        let plain = measurement("20261029-090000-5abc", 0, 5, SccacheFigures::NotInstalled);
        let line = build_measurement_line(Some(&plain));
        assert!(line.contains("sccache not installed"), "{line}");
        assert!(!line.contains("hits"), "{line}");
        // An old record without peaks prints no peak text at all.
        assert!(!line.contains("peak"), "{line}");
        let mut peaked = measured;
        peaked.first_build_peak = Some(athena_data::records::BuildPeak {
            memory_gb: 3.25,
            processes: 6,
        });
        peaked.second_build_peak = Some(athena_data::records::BuildPeak {
            memory_gb: 2.5,
            processes: 4,
        });
        let line = build_measurement_line(Some(&peaked));
        for want in [
            "build total peak memory build 1 3.25 GB, build 2 2.50 GB",
            "peak concurrent processes build 1 6, build 2 4",
        ] {
            assert!(line.contains(want), "{want:?} not in {line}");
        }
        let none = build_measurement_line(None);
        assert!(none.contains("measurement: none yet"), "{none}");
        assert!(none.contains("athena measure-build"), "{none}");
    }

    #[test]
    fn build_measurement_status_shows_the_latest_record_and_a_clear_line_when_there_is_none() {
        let (_dir, _state, athena) = crate::code_health::tests::set_up();
        let before = athena.status().unwrap();
        assert!(
            before.build_measurement.contains("none yet"),
            "{}",
            before.build_measurement
        );

        let older = measurement(
            "20261029-080000-1abc",
            5,
            1_000_000,
            SccacheFigures::NotInstalled,
        );
        let newer = measurement(
            "20261029-090000-2abc",
            1,
            7_300_000_000,
            SccacheFigures::Measured {
                first: SccacheCounts { hits: 0, misses: 3 },
                second: SccacheCounts { hits: 1, misses: 2 },
            },
        );
        // Committed newest first: the latest is picked by time, not by file order.
        commit(&athena, &newer);
        commit(&athena, &older);

        let view = athena.status().unwrap();
        let line = &view.build_measurement;
        assert!(line.contains("T 7.3 GB"), "{line}");
        assert!(line.contains("S 2.5 MB"), "{line}");
        assert!(line.contains("build 1 61.50s, build 2 12.25s"), "{line}");
        assert!(line.contains("build 2 1 hits / 2 misses"), "{line}");
        assert!(!line.contains("not installed"), "{line}");
        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["build_measurement"], line.as_str());
    }
}

/// Status, its per-class forecasts and `land_pr`'s job lookup read the `athena/records` worktree when it
/// exists and `records/` at HEAD only when it is absent (D111). The fixtures here give the two sources
/// different job ids and different classes, so a test shows which one was read.
#[cfg(test)]
pub(crate) mod records_worktree_tests {
    use super::*;
    use crate::commits::{NewFile, message};
    use athena_data::JobId;
    use athena_data::records::{AnyRecord, JobKind, Owner, Score};

    /// The job committed under `records/` at HEAD.
    pub(crate) const TREE_JOB: &str = "20261010-153012-k7q2";
    /// The job that exists only in the worktree.
    pub(crate) const WORKTREE_JOB: &str = "20261011-090000-z4d8";

    /// A dispatched request job of `class`.
    pub(crate) fn job_record(id: &str, class: &str) -> AnyRecord {
        AnyRecord::Job(Box::new(athena_data::records::Job {
            id: JobId::parse(id).unwrap(),
            kind: JobKind::Request,
            title: "A job".into(),
            class_of_work: Some(class.into()),
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
            dispatch_branch: None,
            allowed_paths: None,
        }))
    }

    /// A closing score for the job, with no events, so the job counts in its class's forecast.
    pub(crate) fn score_record(id: &str) -> AnyRecord {
        AnyRecord::Score(Box::new(Score {
            job: JobId::parse(id).unwrap(),
            at: "2026-10-12T10:00:00Z".parse().unwrap(),
            rules: athena_score::RULES.into(),
            cause: None,
            events: vec![],
            deliverables: None,
            quantities: vec![],
            models: None,
        }))
    }

    /// Commits the records under `records/` at HEAD.
    pub(crate) fn commit_in_tree(athena: &Athena, records: &[AnyRecord]) {
        let files: Vec<NewFile> = records.iter().map(NewFile::record).collect();
        athena
            .commit_records(&files, &message("Records in the tree", "", &[]))
            .unwrap();
    }

    /// Writes the records into the worktree directory, creating it.
    pub(crate) fn write_in_worktree(athena: &Athena, records: &[AnyRecord]) {
        let root = athena_data::paths::records_worktree(athena.state.root());
        for record in records {
            let path = root.join(record.canonical_path());
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, record.to_json()).unwrap();
        }
    }

    pub(crate) fn remove_worktree(athena: &Athena) {
        std::fs::remove_dir_all(athena_data::paths::records_worktree(athena.state.root())).unwrap();
    }

    /// The ids status lists, over the recent and the open lines, sorted.
    fn listed(view: &StatusView) -> Vec<String> {
        let mut ids: Vec<String> = view
            .recent
            .iter()
            .chain(&view.open)
            .map(|l| l.id.clone())
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }

    #[test]
    fn status_records_worktree_decides_which_jobs_are_listed() {
        let (_dir, _state, athena) = crate::code_health::tests::set_up();
        commit_in_tree(&athena, &[job_record(TREE_JOB, "class-tree")]);
        write_in_worktree(&athena, &[job_record(WORKTREE_JOB, "class-wt")]);

        let view = athena.status().unwrap();
        assert_eq!(listed(&view), vec![WORKTREE_JOB.to_string()]);
        assert_eq!(view.jobs_by_state.values().sum::<usize>(), 1);
        assert!(
            view.invalid_records.is_empty(),
            "{:?}",
            view.invalid_records
        );

        remove_worktree(&athena);
        let view = athena.status().unwrap();
        assert_eq!(listed(&view), vec![TREE_JOB.to_string()]);
        assert_eq!(view.jobs_by_state.values().sum::<usize>(), 1);
        assert!(
            view.invalid_records.is_empty(),
            "{:?}",
            view.invalid_records
        );
    }

    #[test]
    fn forecast_records_worktree_decides_which_classes_are_headlined() {
        let (_dir, _state, athena) = crate::code_health::tests::set_up();
        commit_in_tree(
            &athena,
            &[job_record(TREE_JOB, "class-tree"), score_record(TREE_JOB)],
        );
        write_in_worktree(
            &athena,
            &[
                job_record(WORKTREE_JOB, "class-wt"),
                score_record(WORKTREE_JOB),
            ],
        );
        let classes = |view: &StatusView| -> Vec<String> {
            view.forecasts.iter().map(|f| f.class.clone()).collect()
        };

        let view = athena.status().unwrap();
        assert_eq!(classes(&view), vec!["class-wt".to_string()]);
        assert_eq!(view.forecasts[0].scored_jobs, 1);
        assert!(
            view.invalid_records.is_empty(),
            "{:?}",
            view.invalid_records
        );

        remove_worktree(&athena);
        let view = athena.status().unwrap();
        assert_eq!(classes(&view), vec!["class-tree".to_string()]);
        assert_eq!(view.forecasts[0].scored_jobs, 1);
    }
}

#[cfg(test)]
pub(crate) mod planning_cost_tests {
    use super::*;
    use athena_data::records::{
        AnyRecord, DriverInfo, InvocationCaps, InvocationContext, InvocationOutcome, ModelUsage,
        Request, Tokens,
    };
    use athena_data::{JobId, Quantity, Unit};

    fn id(text: &str) -> JobId {
        JobId::parse(text).unwrap()
    }

    pub(crate) fn invocation(job: &JobId, seq: u32, role: Role, usd: f64) -> Invocation {
        let tokens = |v: f64| Quantity::measured(v, Unit::Tokens, "result.usage");
        Invocation {
            job: job.clone(),
            seq,
            role,
            driver: DriverInfo {
                name: "claude-code".into(),
                version: "2.1.299".into(),
            },
            athena: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            model: "m".into(),
            effort: None,
            context: InvocationContext {
                components: vec![],
                prompt_sha256: "0".repeat(64),
                prompt_tokens: Quantity::estimated(Some(1.0), Unit::Tokens, "estimate", None, None)
                    .unwrap(),
                host_baseline: Quantity::estimated(Some(600.0), Unit::Tokens, "E3", None, None)
                    .unwrap(),
                tools: vec![],
            },
            caps: InvocationCaps {
                max_budget_usd: 3.0,
                timeout_s: 1800,
            },
            started_at: jiff::Timestamp::UNIX_EPOCH,
            ended_at: jiff::Timestamp::UNIX_EPOCH,
            wall_time: Quantity::measured(60.0, Unit::Seconds, "athena clock"),
            tokens: Tokens {
                input: tokens(1.0),
                output: tokens(1.0),
                cache_read: tokens(1.0),
                cache_write: tokens(1.0),
            },
            models_used: BTreeMap::from([(
                "m".to_string(),
                ModelUsage {
                    input: 1,
                    output: 1,
                    cache_read: 1,
                    cache_write: 1,
                    cost_usd: usd,
                },
            )]),
            cost: Quantity::reported(usd, Unit::Usd, "Claude Code total_cost_usd"),
            outcome: InvocationOutcome::Success,
            detail: None,
            turns: None,
            commands_run: vec![],
            policy_denials: vec![],
            reading_attempt: None,
        }
    }

    fn plan(job: &JobId, round: u32, resumed_from: Option<JobId>) -> JobPlan {
        JobPlan {
            job: job.clone(),
            round,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: 1,
            approach: "Approach.".into(),
            decisions: vec![],
            files: vec![],
            steps: vec![],
            risks: vec![],
            dependents: vec![],
            demonstrations: vec![],
            open_questions: vec![],
            producer_section_sha256: "0".repeat(64),
            resumed_from,
        }
    }

    #[test]
    fn the_status_title_skips_a_leading_priority_line() {
        for (text, title) in [
            ("Priority: urgent\n\nFix the widget", "Fix the widget"),
            ("  priority: high\nFix the widget", "Fix the widget"),
            ("Fix the widget\nmore", "Fix the widget"),
            ("Priority: high\nPriority: low", "Priority: high"),
        ] {
            let j = job(
                "20260101-000000-aaaa",
                text,
                "2026-01-01T00:00:00Z",
                1,
                None,
                OutcomeState::Landed,
                1.0,
            );
            assert_eq!(job_line(&j, "closed").title, title, "{text:?}");
        }
    }

    /// A job with `rounds` plans, each costing `usd` for the planner and the same for the critic, a producer
    /// invocation that must not count, a request text, and the outcome.
    fn job(
        text: &str,
        request: &str,
        dispatched: &str,
        rounds: u32,
        resumed_from: Option<&str>,
        state: OutcomeState,
        usd: f64,
    ) -> JobRecords {
        let job_id = id(text);
        let mut j = JobRecords {
            id: Some(job_id.clone()),
            request: Some(Request {
                job: job_id.clone(),
                text: request.into(),
                received_at: jiff::Timestamp::UNIX_EPOCH,
                channel: "cli".into(),
            }),
            ..JobRecords::default()
        };
        let mut stub: athena_data::records::Job = serde_json::from_value(serde_json::json!({
            "id": text, "kind": "request", "title": "t", "class_of_work": null,
            "capability": null, "accepted_reading": null, "accepted_by": null,
            "owner": { "role": "producer", "capability": null, "model": null, "note": null },
            "verifier_model": null, "caps": null, "base_commit": null, "config_sha256": null,
            "defaults_sha256": null, "athena": null, "driver": null, "brief_sha256": null,
            "dispatched_at": null, "request_path": null, "request_sha256": null, "cost": null,
            "tokens": null, "note": null, "pin": null
        }))
        .unwrap();
        stub.dispatched_at = Some(dispatched.parse().unwrap());
        j.job = Some(stub);
        for round in 1..=rounds {
            let from = (round == 1).then(|| resumed_from.map(id)).flatten();
            j.plans.push(plan(&job_id, round, from));
            j.invocations
                .push(invocation(&job_id, 2 * round, Role::Planner, usd));
            j.invocations
                .push(invocation(&job_id, 2 * round + 1, Role::Critic, usd));
        }
        j.invocations
            .push(invocation(&job_id, 99, Role::Producer, 50.0));
        j.outcome = Some(athena_data::records::Outcome {
            job: job_id,
            state,
            reason: "r".into(),
            at: jiff::Timestamp::UNIX_EPOCH,
            totals: None,
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        });
        j
    }

    fn records(jobs: Vec<JobRecords>) -> Records {
        let mut records = Records::default();
        for j in jobs {
            records.jobs.insert(j.id().as_str().to_string(), j);
        }
        records
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    const OLD_STOP: &str = "20261020-100000-aaaa";
    const OLD_LANDED: &str = "20261021-100000-bbbb";
    const NEW_STOP: &str = "20261026-100000-cccc";
    const NEW_LANDED: &str = "20261026-110000-dddd";

    fn fixture(resumed: bool) -> Records {
        let landed = OutcomeState::Landed;
        let failed = OutcomeState::Failed;
        records(vec![
            // Before: a cap stop of three rounds at 0.2 USD a round, then a fresh plan that landed.
            job(
                OLD_STOP,
                "add x",
                "2026-10-20T10:00:00Z",
                3,
                None,
                failed,
                0.1,
            ),
            job(
                OLD_LANDED,
                &format!("try again on {OLD_STOP}"),
                "2026-10-21T10:00:00Z",
                1,
                None,
                landed,
                0.2,
            ),
            // After: a cap stop, then a retry that resumed from it with one round of revision.
            job(
                NEW_STOP,
                "add y",
                "2026-10-26T10:00:00Z",
                3,
                None,
                failed,
                0.1,
            ),
            job(
                NEW_LANDED,
                &format!("try again on {NEW_STOP}"),
                "2026-10-26T11:00:00Z",
                if resumed { 2 } else { 3 },
                resumed.then_some(NEW_STOP),
                landed,
                if resumed { 0.05 } else { 0.1 },
            ),
            // Excluded: landed without planning, and planned without landing.
            job(
                "20261022-100000-eeee",
                "add z",
                "2026-10-22T10:00:00Z",
                0,
                None,
                landed,
                0.0,
            ),
            job(
                "20261022-110000-ffff",
                "add w",
                "2026-10-22T11:00:00Z",
                2,
                None,
                failed,
                9.0,
            ),
        ])
    }

    #[test]
    fn planning_cost_sums_each_chain_at_its_landed_job_and_splits_at_the_first_resumed_plan() {
        let cost = planning_cost(&fixture(true)).expect("landed planned jobs");
        assert_eq!(
            cost.first_resumed,
            Some((NEW_LANDED.to_string(), "2026-10-26T11:00:00Z".to_string()))
        );
        // Before: one landed planned job (the unplanned landed job and the unlanded one are left out). Its
        // chain is 3 + 1 rounds and 0.6 + 0.4 USD of planner and critic; its own are 1 round and 0.4 USD.
        let before = cost.before;
        assert_eq!(before.jobs, 1);
        assert!(close(before.mean_rounds, 4.0), "{before:?}");
        assert!(close(before.mean_cost_usd, 1.0), "{before:?}");
        assert!(close(before.own_mean_rounds, 1.0), "{before:?}");
        assert!(close(before.own_mean_cost_usd, 0.4), "{before:?}");
        // After: the resumed retry. Its chain is 3 + 2 rounds and 0.6 + 0.2 USD; its own are 2 and 0.2.
        let after = cost.after;
        assert_eq!(after.jobs, 1);
        assert!(close(after.mean_rounds, 5.0), "{after:?}");
        assert!(close(after.mean_cost_usd, 0.8), "{after:?}");
        assert!(close(after.own_mean_rounds, 2.0), "{after:?}");
        assert!(close(after.own_mean_cost_usd, 0.2), "{after:?}");
    }

    #[test]
    fn planning_cost_follows_a_three_job_chain_once_and_stops_at_a_cycle() {
        let landed = OutcomeState::Landed;
        let failed = OutcomeState::Failed;
        let a = "20261020-100000-aaaa";
        let b = "20261021-100000-bbbb";
        let c = "20261022-100000-cccc";
        let chain = records(vec![
            job(
                a,
                &format!("try again on {c}"),
                "2026-10-20T10:00:00Z",
                1,
                None,
                failed,
                0.1,
            ),
            job(
                b,
                &format!("try again on {a}"),
                "2026-10-21T10:00:00Z",
                2,
                None,
                failed,
                0.1,
            ),
            job(
                c,
                &format!("try again on {b}"),
                "2026-10-22T10:00:00Z",
                3,
                None,
                landed,
                0.1,
            ),
        ]);
        let cost = planning_cost(&chain).expect("a landed planned job");
        assert_eq!(
            cost.before.jobs, 1,
            "the chain counts once, at its landed job"
        );
        assert!(close(cost.before.mean_rounds, 6.0), "{cost:?}");
        assert!(close(cost.before.mean_cost_usd, 1.2), "{cost:?}");
    }

    #[test]
    fn planning_cost_is_absent_when_no_planned_job_landed() {
        let none = records(vec![job(
            OLD_STOP,
            "add x",
            "2026-10-20T10:00:00Z",
            3,
            None,
            OutcomeState::Failed,
            0.1,
        )]);
        assert_eq!(planning_cost(&none), None);
    }

    #[test]
    fn planning_cost_has_no_after_group_when_no_plan_was_resumed() {
        let cost = planning_cost(&fixture(false)).expect("landed planned jobs");
        assert_eq!(cost.first_resumed, None);
        assert_eq!(cost.after, PlanningGroup::default());
        assert_eq!(cost.before.jobs, 2);
        // Chains of 4 and 6 rounds, costing 1.0 and 1.2 USD.
        assert!(close(cost.before.mean_rounds, 5.0), "{cost:?}");
        assert!(close(cost.before.mean_cost_usd, 1.1), "{cost:?}");
    }

    fn critique(job: &str, round: u32, verdict: CritiqueVerdict) -> Critique {
        Critique {
            job: id(job),
            round,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: None,
            agent_verdict: Some(verdict),
            final_verdict: verdict,
            findings: vec![],
            request_reading_mismatch: None,
            summary: None,
        }
    }

    /// A planned, dispatched job costing `usd` in all, closed as `state`, whose critiques give `verdicts` in turn.
    fn planned(
        i: usize,
        state: OutcomeState,
        usd: f64,
        verdicts: &[CritiqueVerdict],
    ) -> JobRecords {
        let text = format!("20261001-00{:02}{:02}-abcd", i / 60, i % 60);
        let mut j = job(
            &text,
            &format!("request {i}"),
            "2026-10-01T10:00:00Z",
            1,
            None,
            state,
            0.0,
        );
        j.invocations = vec![invocation(&id(&text), 99, Role::Producer, usd)];
        for (n, v) in verdicts.iter().enumerate() {
            j.critiques.push(critique(&text, n as u32 + 1, *v));
        }
        j
    }

    /// `n` planned jobs of which `landed` landed (accepted in round 1); the rest were stopped in planning.
    fn fleet(n: usize, landed: usize, usd: f64) -> Records {
        records(
            (0..n)
                .map(|i| {
                    if i < landed {
                        planned(i, OutcomeState::Landed, usd, &[CritiqueVerdict::Accept])
                    } else {
                        planned(i, OutcomeState::Failed, usd, &[CritiqueVerdict::Reject])
                    }
                })
                .collect(),
        )
    }

    #[test]
    fn keep_rule_flags_28_landed_of_51_planned_as_a_breach_with_the_measures() {
        use CritiqueVerdict::{Accept, Reject, Revise};
        let jobs = (0..51)
            .map(|i| match i {
                // 20 landed after an accept in round 1, 8 after a revise first.
                0..20 => planned(i, OutcomeState::Landed, 1.0, &[Accept]),
                20..28 => planned(i, OutcomeState::Landed, 1.0, &[Revise, Accept]),
                // 23 not landed; stopped in planning at round 1.
                _ => planned(i, OutcomeState::Failed, 1.0, &[Reject]),
            })
            .collect();
        let rule = keep_rule(&records(jobs)).expect("planned jobs");
        assert_eq!((rule.planned, rule.landed), (51, 28));
        assert!(close(rule.share, 28.0 / 51.0), "{rule:?}");
        assert!(close(rule.total_cost_usd, 51.0), "{rule:?}");
        assert!(
            close(rule.cost_per_landed_usd.unwrap(), 51.0 / 28.0),
            "{rule:?}"
        );
        assert_eq!(
            (rule.first_round_accepted, rule.first_round_planned),
            (20, 51)
        );
        assert!(
            close(rule.first_round_accept_rate.unwrap(), 20.0 / 51.0),
            "{rule:?}"
        );
        assert_eq!(rule.verdict, KeepVerdict::Breach);

        let json = serde_json::to_value(&rule).unwrap();
        assert_eq!(json["planned"], 51);
        assert_eq!(json["landed"], 28);
        assert_eq!(json["baseline_share"], 0.64);
        assert_eq!(json["verdict"], "breach");
        assert!((json["share"].as_f64().unwrap() - 0.549).abs() < 1e-3);
        assert!(json["cost_per_landed_usd"].is_f64());
        assert!(json["first_round_accept_rate"].is_f64());
    }

    #[test]
    fn keep_rule_counts_a_job_stopped_in_planning_without_a_dispatch_as_planned_and_not_landed() {
        let mut stopped = planned(1, OutcomeState::Failed, 0.5, &[CritiqueVerdict::Reject]);
        stopped.job = None;
        stopped.outcome = None;
        let rule = keep_rule(&records(vec![
            planned(0, OutcomeState::Landed, 1.5, &[CritiqueVerdict::Accept]),
            stopped,
        ]))
        .unwrap();
        assert_eq!((rule.planned, rule.landed), (2, 1));
        // All cost of the planned jobs over the landed ones.
        assert!(close(rule.cost_per_landed_usd.unwrap(), 2.0), "{rule:?}");
    }

    #[test]
    fn keep_rule_early_branches_keep_at_the_baseline_and_breach_below_it() {
        // 30 planned: 20 of 30 is 0.667 (keep); 19 of 30 is 0.633 (breach).
        assert_eq!(
            keep_rule(&fleet(30, 20, 1.0)).unwrap().verdict,
            KeepVerdict::Keep
        );
        assert_eq!(
            keep_rule(&fleet(30, 19, 1.0)).unwrap().verdict,
            KeepVerdict::Breach
        );
        // Exactly the baseline: 16 of 25 is 0.64, kept.
        assert_eq!(
            keep_rule(&fleet(25, 16, 1.0)).unwrap().verdict,
            KeepVerdict::Keep
        );
        // Between the thresholds the early rule continues.
        assert_eq!(
            keep_rule(&fleet(60, 30, 1.0)).unwrap().verdict,
            KeepVerdict::Breach
        );
    }

    #[test]
    fn keep_rule_decision_branches_at_100_planned_jobs() {
        let verdict = |landed, usd| keep_rule(&fleet(100, landed, usd)).unwrap().verdict;
        // 72% landed at $1.39 a landed job ($100 in all): keep.
        assert_eq!(verdict(72, 1.0), KeepVerdict::Keep);
        // 72% landed but $2.78 a landed job ($200 in all): inconclusive.
        assert_eq!(verdict(72, 2.0), KeepVerdict::Inconclusive);
        // 64% landed: turn off. 65% landed: inconclusive.
        assert_eq!(verdict(64, 1.0), KeepVerdict::TurnOff);
        assert_eq!(verdict(65, 1.0), KeepVerdict::Inconclusive);
        assert_eq!(verdict(10, 1.0), KeepVerdict::TurnOff);
    }

    /// A dispatched job with no planning whose producer ran with `model` and `effort`, costing `usd`, with one
    /// verifier run that gave `verdict`.
    fn producer_job(
        text: &str,
        model: &str,
        effort: Option<&str>,
        state: OutcomeState,
        verdict: athena_data::records::Verdict,
        usd: f64,
    ) -> JobRecords {
        let mut j = job(text, "r", "2026-10-26T10:00:00Z", 0, None, state, usd);
        let job_id = id(text);
        j.invocations.clear();
        let mut producer = invocation(&job_id, 1, Role::Producer, usd);
        producer.model = model.into();
        producer.effort = effort.map(str::to_string);
        j.invocations.push(producer);
        j.invocations
            .push(invocation(&job_id, 2, Role::Verifier, 0.0));
        j.verification = Some(athena_data::records::Verification {
            job: job_id,
            candidate_commit: "0".repeat(40),
            verified_fingerprint: String::new(),
            tool_config_from_base: vec![],
            checks: vec![],
            check_cleanup: vec![],
            verifier_invocation: Some(2),
            criteria: vec![],
            findings: vec![],
            request_reading_mismatch: None,
            agent_verdict: Some(verdict),
            final_verdict: verdict,
            reasons: vec![],
            inputs: vec![],
            summary: None,
        });
        j
    }

    #[test]
    fn producer_settings_split_first_pass_landed_share_and_cost_by_model_and_effort() {
        use athena_data::records::Verdict::{Fail, Pass};
        let landed = OutcomeState::Landed;
        let failed = OutcomeState::Failed;
        let stats = producer_settings(&records(vec![
            producer_job(
                "20261026-100000-aaaa",
                "s",
                Some("medium"),
                landed,
                Pass,
                1.0,
            ),
            producer_job(
                "20261026-100001-aaab",
                "s",
                Some("medium"),
                failed,
                Fail,
                3.0,
            ),
            producer_job("20261026-100002-aaac", "s", Some("high"), failed, Fail, 2.0),
            producer_job("20261026-100003-aaad", "s", None, landed, Pass, 0.5),
        ]));
        let get = |effort: &str| {
            stats
                .iter()
                .find(|s| s.model == "s" && s.effort == effort)
                .unwrap_or_else(|| panic!("no bucket for {effort}: {stats:?}"))
        };
        assert_eq!(stats.len(), 3);
        let medium = get("medium");
        assert_eq!((medium.jobs, medium.verified, medium.first_pass), (2, 2, 1));
        assert_eq!(medium.first_pass_rate, Some(0.5));
        assert_eq!(medium.landed, 1);
        assert!(close(medium.landed_share, 0.5));
        assert!(close(medium.cost_per_landed_usd.unwrap(), 4.0));
        // A bucket with no landed job has a zero share and no cost per landed job, not a division by zero.
        let high = get("high");
        assert_eq!((high.jobs, high.landed), (1, 0));
        assert_eq!(high.landed_share, 0.0);
        assert_eq!(high.cost_per_landed_usd, None);
        assert_eq!(high.first_pass_rate, Some(0.0));
        // A producer invocation recorded before effort was kept gets its own bucket.
        assert_eq!(get(EFFORT_UNRECORDED).landed, 1);
    }

    fn test_run(seconds: f64, basis: &str) -> athena_data::records::CheckRun {
        queued_test_run(seconds, basis, None)
    }

    fn queued_test_run(
        seconds: f64,
        basis: &str,
        queued_s: Option<f64>,
    ) -> athena_data::records::CheckRun {
        athena_data::records::CheckRun {
            name: "test".into(),
            command: vec!["cargo".into(), "test".into()],
            exit_code: Some(0),
            passed: true,
            tests_ran: Some(10),
            duration: Quantity::measured(seconds, Unit::Seconds, basis),
            output_tail: String::new(),
            note: None,
            failed_tests: Vec::new(),
            reused: false,
            revert_for: Vec::new(),
            queued_s,
        }
    }

    /// 24 request jobs, oldest first: the 2 oldest took 5000 s (outside the window of 20); job 12 has no test
    /// check; the newest has the 0 s placeholder of a check that did not run; job 15 took `slowest` s; the
    /// others took 100 s plus their number.
    fn seeded_durations(slowest: f64) -> Records {
        let jobs = (0..24)
            .map(|n: u32| {
                let text = format!("20261026-1000{n:02}-aa{n:02}");
                let mut j = producer_job(
                    &text,
                    "s",
                    None,
                    OutcomeState::Landed,
                    athena_data::records::Verdict::Pass,
                    0.0,
                );
                let checks = match n {
                    0 | 1 => vec![test_run(5000.0, "athena monotonic clock")],
                    12 => vec![],
                    15 => vec![test_run(slowest, "athena monotonic clock")],
                    23 => vec![test_run(0.0, "not run")],
                    _ => vec![test_run(100.0 + n as f64, "athena monotonic clock")],
                };
                j.verification.as_mut().unwrap().checks = checks;
                j
            })
            .collect();
        records(jobs)
    }

    #[test]
    fn check_durations_report_median_slowest_and_the_eighty_percent_flag() {
        let limit = 1800;
        // The 20 counted jobs are numbers 2 to 22 but 12: durations 102 to 111, 113 to 122 with job 15 slow.
        // The median is the mean of 111 and 113. The placeholder, the missing check and the two old jobs are out.
        let over = check_durations(&seeded_durations(1500.0), limit).unwrap();
        assert_eq!(over.jobs, 20);
        assert!(close(over.median_s, 112.0), "{over:?}");
        assert!(close(over.slowest_s, 1500.0));
        assert_eq!(over.slowest_job, "20261026-100015-aa15");
        assert_eq!(over.limit_s, limit);
        assert!(over.flagged, "1500 s is over 80% of 1800 s");
        let line = over.text();
        assert!(line.contains("median 112s"), "{line}");
        assert!(
            line.contains("slowest 1500s (20261026-100015-aa15)"),
            "{line}"
        );
        assert!(line.contains("of 1800s limit over last 20 jobs"), "{line}");
        assert!(line.contains("over 80% of the limit"), "{line}");

        // Under the threshold: reported, not flagged.
        let under = check_durations(&seeded_durations(1000.0), limit).unwrap();
        assert!(!under.flagged);
        assert!(close(under.slowest_s, 1000.0));
        assert!(!under.text().contains("80%"), "{}", under.text());

        // Exactly 80% of the limit is not over it.
        let edge = check_durations(&seeded_durations(1440.0), limit).unwrap();
        assert!(!edge.flagged);
    }

    #[test]
    fn check_durations_need_a_measured_test_run() {
        assert_eq!(check_durations(&Records::default(), 1800), None);
        let mut only_placeholder = producer_job(
            "20261026-100000-aaaa",
            "s",
            None,
            OutcomeState::Landed,
            athena_data::records::Verdict::Pass,
            0.0,
        );
        only_placeholder.verification.as_mut().unwrap().checks = vec![test_run(0.0, "not run")];
        assert_eq!(
            check_durations(&records(vec![only_placeholder]), 1800),
            None
        );
    }

    fn classed_job(
        text: &str,
        class: &str,
        effort: &str,
        state: OutcomeState,
        verdict: athena_data::records::Verdict,
    ) -> JobRecords {
        let mut j = producer_job(text, "s", Some(effort), state, verdict, 1.0);
        j.job.as_mut().unwrap().class_of_work = Some(class.into());
        j
    }

    #[test]
    fn specialization_flags_only_a_class_where_an_enough_used_setting_beats_the_default() {
        use athena_data::records::Verdict::{Fail, Pass};
        let (landed, failed) = (OutcomeState::Landed, OutcomeState::Failed);
        let mut jobs = Vec::new();
        let mut n = 0;
        let mut add = |class: &str, effort: &str, state, verdict| {
            n += 1;
            jobs.push(classed_job(
                &format!("20261026-1000{n:02}-aa{n:02}"),
                class,
                effort,
                state,
                verdict,
            ));
        };
        // "code": medium (default, 4 jobs) lands 1 of 4; high (3 jobs) lands 3 of 3.
        for _ in 0..3 {
            add("code", "medium", failed, Fail);
        }
        add("code", "medium", landed, Pass);
        for _ in 0..3 {
            add("code", "high", landed, Pass);
        }
        // "docs": the same, but the better setting has only 2 jobs.
        for _ in 0..3 {
            add("docs", "medium", failed, Fail);
        }
        for _ in 0..2 {
            add("docs", "high", landed, Pass);
        }
        // "tests": the other setting has enough jobs and does no better.
        for _ in 0..4 {
            add("tests", "medium", landed, Pass);
        }
        for _ in 0..3 {
            add("tests", "high", failed, Fail);
        }
        let result = specialization(&records(jobs));
        let verdict = |c: &str| {
            &result
                .iter()
                .find(|r| r.class == c)
                .unwrap_or_else(|| panic!("no class {c}: {result:?}"))
                .verdict
        };
        assert_eq!(
            verdict("code"),
            &SpecializationVerdict::Helps {
                model: "s".into(),
                effort: "high".into()
            }
        );
        assert_eq!(verdict("docs"), &SpecializationVerdict::NotEnoughJobs);
        assert_eq!(verdict("tests"), &SpecializationVerdict::NoBetterSetting);
        let code = result.iter().find(|r| r.class == "code").unwrap();
        assert_eq!(code.baseline_effort, "medium");
        assert_eq!(code.settings.len(), 2);
    }

    #[test]
    fn specialization_groups_unclassified_jobs_and_is_empty_without_a_producer() {
        use athena_data::records::Verdict::Pass;
        assert!(specialization(&Records::default()).is_empty());
        let r = specialization(&records(vec![producer_job(
            "20261026-100000-aaaa",
            "s",
            Some("medium"),
            OutcomeState::Landed,
            Pass,
            1.0,
        )]));
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].class, "unclassified");
        assert_eq!(r[0].verdict, SpecializationVerdict::NotEnoughJobs);
    }

    #[test]
    fn producer_settings_are_empty_without_a_dispatched_producer() {
        assert!(producer_settings(&Records::default()).is_empty());
        let mut queued = job(
            "20261026-100000-aaaa",
            "r",
            "2026-10-26T10:00:00Z",
            0,
            None,
            OutcomeState::Landed,
            1.0,
        );
        queued.invocations.clear();
        assert!(producer_settings(&records(vec![queued])).is_empty());
    }

    #[test]
    fn keep_rule_with_nothing_landed_divides_by_nothing() {
        let rule = keep_rule(&fleet(5, 0, 1.0)).unwrap();
        assert_eq!(rule.landed, 0);
        assert_eq!(rule.share, 0.0);
        assert_eq!(rule.cost_per_landed_usd, None);
        assert_eq!(rule.verdict, KeepVerdict::Breach);
        assert!(serde_json::to_value(&rule).unwrap()["cost_per_landed_usd"].is_null());
        assert_eq!(
            keep_rule(&fleet(100, 0, 1.0)).unwrap().verdict,
            KeepVerdict::TurnOff
        );
        assert_eq!(keep_rule(&Records::default()), None);
    }

    /// A later introspection about `job`, whose cause is `cause`.
    fn failure_cause(
        record: &str,
        job: &str,
        cause: &str,
    ) -> athena_data::records::IntrospectionRecord {
        use athena_data::records::{
            IntrospectionRecord, IntrospectionStatus, IntrospectionTrigger, PreventionKind,
        };
        IntrospectionRecord {
            id: athena_data::RecordId::parse(record).unwrap(),
            at: jiff::Timestamp::UNIX_EPOCH,
            trigger: IntrospectionTrigger::Failure,
            job: Some(id(job)),
            intervention: None,
            class_of_work: "change".into(),
            category: "failed".into(),
            what_happened: Some("a drain stopped".into()),
            failure_class: None,
            new_class: false,
            earlier_cases: Vec::new(),
            decisions_matched: Vec::new(),
            cause: Some(cause.into()),
            prevention_kind: Some(PreventionKind::CodeCheck),
            prevention: Some("p".into()),
            proposed_request: Some("r".into()),
            evidence: None,
            status: IntrospectionStatus::Recorded,
            failure: None,
            flags: Vec::new(),
            other_causes: Vec::new(),
            open_hypotheses: Vec::new(),
            contradictions: Vec::new(),
            dropped_ids: Vec::new(),
            planning_stopped: false,
            from_jev: false,
            jev: None,
        }
    }

    #[test]
    fn later_harm_counts_landings_cited_as_a_failure_revert_or_fix_for_planned_and_unplanned_jobs()
    {
        const P_FAILED: &str = "20261020-100000-a1a1";
        const P_CLEAN: &str = "20261020-110000-b2b2";
        const U_FAILED: &str = "20261021-100000-c3c3";
        const U_REVERTED: &str = "20261021-110000-d4d4";
        const U_FIXED: &str = "20261021-120000-e5e5";
        const U_CLEAN: &str = "20261021-130000-f6f6";
        const U_NOT_LANDED: &str = "20261021-140000-g7g7";
        let landed = OutcomeState::Landed;
        // A dispatched job with no plan record, whose request is `request`.
        let unplanned = |text: &str, request: &str, state| {
            job(text, request, "2026-10-21T10:00:00Z", 0, None, state, 0.0)
        };
        let planned_job = |text: &str| {
            let mut j = job(text, "tidy", "2026-10-20T10:00:00Z", 1, None, landed, 0.0);
            j.critiques.push(critique(text, 1, CritiqueVerdict::Accept));
            j
        };
        let mut records = records(vec![
            planned_job(P_FAILED),
            planned_job(P_CLEAN),
            unplanned(U_FAILED, "tidy", landed),
            unplanned(U_REVERTED, "tidy", landed),
            unplanned(U_FIXED, "tidy", landed),
            unplanned(U_CLEAN, "tidy", landed),
            unplanned(U_NOT_LANDED, "tidy", OutcomeState::Failed),
            // Later jobs: one asks to revert a landing, one to fix another. A retry of a job that did not land,
            // and a request that cites a landing without asking for either, are no cause.
            unplanned(
                "20261022-100000-h8h8",
                &format!("Revert job {U_REVERTED}: it blocks the drain."),
                OutcomeState::Failed,
            ),
            unplanned(
                "20261022-110000-j9j9",
                &format!("Fix the rule that job {U_FIXED} landed."),
                OutcomeState::Failed,
            ),
            unplanned(
                "20261022-120000-k1k1",
                &format!("try again on {U_NOT_LANDED}; background: {U_CLEAN}"),
                OutcomeState::Failed,
            ),
        ]);
        records.introspections = vec![
            // A later failure names a planned landing and an unplanned one as its cause.
            failure_cause(
                "20261022-130000-m2m2",
                "20261022-100000-h8h8",
                &format!("The refusal rule landed by {P_FAILED} matched words and held the drain."),
            ),
            failure_cause(
                "20261022-140000-n3n3",
                "20261022-110000-j9j9",
                &format!("{U_FAILED} changed the gate."),
            ),
            // An introspection of the landing's own job, and one older than the landing, are no later cause.
            failure_cause(
                "20261022-150000-p4p4",
                P_CLEAN,
                &format!("{P_CLEAN} itself."),
            ),
            failure_cause(
                "20261019-100000-q5q5",
                "20261022-100000-h8h8",
                &format!("{U_CLEAN} earlier."),
            ),
        ];

        let harm = later_harm(&records);
        let p = harm.planned;
        assert_eq!((p.jobs, p.landed), (2, 2), "{harm:?}");
        assert_eq!(
            (p.cited, p.failures, p.reverts, p.fixes),
            (1, 1, 0, 0),
            "{harm:?}"
        );
        assert_eq!(p.share, Some(1.0));
        let u = harm.unplanned;
        // Eight unplanned dispatched jobs: four landings, the job that did not land, and the three later jobs.
        assert_eq!((u.jobs, u.landed), (8, 4), "{harm:?}");
        assert_eq!(
            (u.cited, u.failures, u.reverts, u.fixes),
            (3, 1, 1, 1),
            "{harm:?}"
        );
        assert!(close(u.share.unwrap(), 4.0 / 8.0), "{harm:?}");

        // It rides on the keep rule, beside the landed share, and in its JSON.
        let rule = keep_rule(&records).expect("planned jobs");
        assert_eq!(rule.later_harm, harm);
        let json = serde_json::to_value(&rule).unwrap();
        assert_eq!(json["later_harm"]["planned"]["cited"], 1);
        assert_eq!(json["later_harm"]["unplanned"]["reverts"], 1);
        assert_eq!(json["later_harm"]["unplanned"]["fixes"], 1);
        assert_eq!(json["later_harm"]["unplanned"]["failures"], 1);
    }

    /// The records of one request job whose stages took the minutes in `m`, starting at `start`: the reader,
    /// the planner (when `m.planning` is above zero), the producer, the checks (a verification with one
    /// `test` check, when `m.checks` is above zero), the verifier, then `m.landing_and_rebuild` minutes before
    /// the outcome `state`. A stage of zero minutes is left out, so a job can fail early.
    pub(crate) fn timed_job(
        text: &str,
        start: jiff::Timestamp,
        m: StageMinutes,
        state: OutcomeState,
    ) -> Vec<AnyRecord> {
        use athena_data::records::{Outcome, Request, Verdict, Verification};
        let job_id = id(text);
        let after = |t: jiff::Timestamp, minutes: f64| {
            t.checked_add(jiff::SignedDuration::from_secs_f64(minutes * 60.0))
                .unwrap()
        };
        let mut records = vec![AnyRecord::Request(Request {
            job: job_id.clone(),
            text: "A request.".into(),
            received_at: start,
            channel: "cli".into(),
        })];
        let mut cursor = start;
        let mut seq = 0;
        let mut invocations: Vec<Invocation> = Vec::new();
        let mut run = |role: Role, minutes: f64, cursor: &mut jiff::Timestamp| {
            if minutes > 0.0 {
                seq += 1;
                let mut i = invocation(&job_id, seq, role, 0.0);
                i.started_at = *cursor;
                i.ended_at = after(*cursor, minutes);
                *cursor = i.ended_at;
                Some(i)
            } else {
                None
            }
        };
        invocations.extend(run(Role::Reader, m.reading, &mut cursor));
        invocations.extend(run(Role::Planner, m.planning, &mut cursor));
        invocations.extend(run(Role::Producer, m.producer, &mut cursor));
        if m.checks > 0.0 {
            // The checks run between the producer and the verifier, outside any invocation.
            let mut check = test_run(m.checks * 60.0, "athena monotonic clock");
            check.name = "test".into();
            cursor = after(cursor, m.checks);
            records.push(AnyRecord::Verification(Box::new(Verification {
                job: job_id.clone(),
                candidate_commit: "0".repeat(40),
                verified_fingerprint: String::new(),
                tool_config_from_base: vec![],
                checks: vec![check],
                check_cleanup: vec![],
                verifier_invocation: None,
                criteria: vec![],
                findings: vec![],
                request_reading_mismatch: None,
                agent_verdict: Some(Verdict::Pass),
                final_verdict: Verdict::Pass,
                reasons: vec![],
                inputs: vec![],
                summary: None,
            })));
        }
        invocations.extend(run(Role::Verifier, m.verifier, &mut cursor));
        let totals = crate::scoring::totals(&invocations, None);
        records.extend(
            invocations
                .into_iter()
                .map(|i| AnyRecord::Invocation(Box::new(i))),
        );
        records.push(AnyRecord::Outcome(Box::new(Outcome {
            job: job_id,
            state,
            reason: "r".into(),
            at: after(cursor, m.landing_and_rebuild),
            totals: Some(totals),
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        })));
        records
    }

    /// The in-memory `Records` of records made by [`timed_job`].
    fn loaded(all: Vec<AnyRecord>) -> Records {
        let mut records = Records::default();
        for record in all {
            let job = match &record {
                AnyRecord::Request(r) => &r.job,
                AnyRecord::Invocation(r) => &r.job,
                AnyRecord::Verification(r) => &r.job,
                AnyRecord::Outcome(r) => &r.job,
                other => panic!("not a timed job record: {other:?}"),
            };
            let entry = records.jobs.entry(job.as_str().to_string()).or_default();
            entry.id.get_or_insert_with(|| job.clone());
            match record {
                AnyRecord::Request(r) => entry.request = Some(r),
                AnyRecord::Invocation(r) => entry.invocations.push(*r),
                AnyRecord::Verification(r) => entry.verification = Some(*r),
                AnyRecord::Outcome(r) => entry.outcome = Some(*r),
                _ => unreachable!(),
            }
        }
        records
    }

    fn minutes(
        reading: f64,
        planning: f64,
        producer: f64,
        checks: f64,
        verifier: f64,
        landing: f64,
    ) -> StageMinutes {
        StageMinutes {
            reading,
            planning,
            producer,
            checks,
            verifier,
            landing_and_rebuild: landing,
        }
    }

    /// A landed job whose verification checks carry the given `queued_s` values, one check per value.
    fn landed_job_with_waits(text: &str, start: &str, waits: &[Option<f64>]) -> Vec<AnyRecord> {
        let mut records = timed_job(
            text,
            start.parse().unwrap(),
            minutes(1.0, 0.0, 5.0, 2.0, 3.0, 1.0),
            OutcomeState::Landed,
        );
        for record in &mut records {
            if let AnyRecord::Verification(v) = record {
                v.checks = waits
                    .iter()
                    .enumerate()
                    .map(|(n, queued)| {
                        let mut check = queued_test_run(60.0, "athena monotonic clock", *queued);
                        check.name = format!("check-{n}");
                        check
                    })
                    .collect();
            }
        }
        records
    }

    #[test]
    fn the_throughput_section_sums_build_wait() {
        let mut all = landed_job_with_waits(
            "20261029-000000-aaab",
            "2026-10-29T00:00:00Z",
            &[Some(12.0), None],
        );
        all.extend(landed_job_with_waits(
            "20261029-010000-aaac",
            "2026-10-29T01:00:00Z",
            &[Some(30.0)],
        ));
        let text = Throughput::compute(&loaded(all))
            .expect("landed jobs")
            .text();
        let line = text
            .lines()
            .find(|l| l.trim_start().starts_with("build wait"))
            .unwrap_or_else(|| panic!("no build wait line in:\n{text}"));
        assert!(
            line.contains("42.0 s over 2 check(s), longest 30.0 s"),
            "{line}"
        );
        let basis = text.lines().skip_while(|l| l != &line).nth(1).unwrap();
        assert!(basis.contains("athena monotonic clock"), "{basis}");
        assert!(basis.trim_start().starts_with("basis:"), "{basis}");
    }

    #[test]
    fn no_recorded_wait_gives_no_build_wait_line() {
        let all = landed_job_with_waits("20261029-000000-aaab", "2026-10-29T00:00:00Z", &[None]);
        let throughput = Throughput::compute(&loaded(all)).expect("a landed job");
        assert_eq!(throughput.build_wait, None);
        assert!(!throughput.text().contains("build wait"));
    }

    #[test]
    fn throughput_reports_landed_per_day_stage_medians_idle_time_and_time_on_jobs_that_did_not_land()
     {
        let t = |s: &str| -> jiff::Timestamp { s.parse().unwrap() };
        let landed = OutcomeState::Landed;
        let mut all = Vec::new();
        // A lands on the 28th. B and C land on the 29th, and C ran while B ran.
        all.extend(timed_job(
            "20261028-230000-aaaa",
            t("2026-10-28T23:00:00Z"),
            minutes(2.0, 0.0, 10.0, 4.0, 6.0, 3.0),
            landed,
        ));
        all.extend(timed_job(
            "20261029-000000-aaab",
            t("2026-10-29T00:00:00Z"),
            minutes(4.0, 6.0, 20.0, 6.0, 8.0, 5.0),
            landed,
        ));
        all.extend(timed_job(
            "20261029-003000-aaac",
            t("2026-10-29T00:30:00Z"),
            minutes(3.0, 0.0, 15.0, 5.0, 5.0, 4.0),
            landed,
        ));
        // Two jobs that did not land: 30 minutes and 10 minutes of work.
        all.extend(timed_job(
            "20261029-030000-aaad",
            t("2026-10-29T03:00:00Z"),
            minutes(2.0, 0.0, 20.0, 0.0, 0.0, 8.0),
            OutcomeState::Failed,
        ));
        all.extend(timed_job(
            "20261029-040000-aaae",
            t("2026-10-29T04:00:00Z"),
            minutes(1.0, 0.0, 0.0, 0.0, 0.0, 9.0),
            OutcomeState::Rejected,
        ));
        let throughput = Throughput::compute(&loaded(all)).expect("jobs landed");

        assert_eq!(throughput.landed_total, 3);
        assert_eq!(
            throughput.landed_per_day,
            vec![
                DayCount {
                    day: "2026-10-28".into(),
                    landed: 1
                },
                DayCount {
                    day: "2026-10-29".into(),
                    landed: 2
                },
            ]
        );
        assert_eq!(throughput.window_jobs, 3);
        // Totals 25, 49 and 32 minutes: the median is 32, not the sum of the stage medians.
        assert_eq!(throughput.median_minutes, 32.0);
        assert_eq!(
            throughput.median_stages,
            minutes(3.0, 0.0, 15.0, 5.0, 6.0, 4.0)
        );
        // Gaps: A ends 23:25, B starts 00:00 (35); B and C overlap (0); C ends 01:02, D starts 03:00 (118);
        // D's invocations end 03:22 (the closing record after them is not time spent), the rejected job
        // starts 04:00 (38).
        assert_eq!(throughput.idle_minutes, 191.0);
        assert_eq!(throughput.unlanded_jobs, 2);
        assert_eq!(throughput.unlanded_minutes, 22.0 + 1.0);

        let text = throughput.text();
        for expected in [
            "3 landed (2026-10-28 1 · 2026-10-29 2)",
            "median per landed job over the last 3 measured: 32.0 min",
            "reading 3.0 · planning 0.0 · producer 15.0 · checks 5.0 · verifier 6.0 · landing and rebuild 4.0",
            "idle between jobs 191.0 min",
            "did not land: 2 job(s), 23.0 min",
        ] {
            assert!(text.contains(expected), "{expected:?} not in:\n{text}");
        }
    }

    fn t(s: &str) -> jiff::Timestamp {
        s.parse().unwrap()
    }

    /// A landing record, written at `created_at`, whose one commit is the job landing of `job`.
    fn landing_of(job: &str, created_at: jiff::Timestamp) -> athena_data::history::Landing {
        use athena_data::history::{Landing, LandingCommit, LandingKind, MergeMethod};
        Landing {
            id: athena_data::RecordId::generate(created_at),
            created_at,
            pr_number: 7,
            head_sha: "a".repeat(40),
            host_commit: "b".repeat(40),
            merge_method: MergeMethod::Squash,
            commits: vec![LandingCommit {
                sha: "a".repeat(40),
                tree: "c".repeat(40),
                kind: LandingKind::JobLanding,
                job: Some(id(job)),
                verification: None,
            }],
            intent: athena_data::RecordId::generate(created_at),
        }
    }

    const LANDED_JOB: &str = "20261029-100000-mnd1";

    /// One landed job: verifier ends 10:17, the outcome is set 0.1 minute later.
    fn landed_job_records() -> Records {
        loaded(timed_job(
            LANDED_JOB,
            t("2026-10-29T10:00:00Z"),
            minutes(0.0, 0.0, 10.0, 2.0, 5.0, 0.1),
            OutcomeState::Landed,
        ))
    }

    #[test]
    fn the_landing_stage_runs_to_the_landing_record_written_after_the_outcome() {
        let mut records = landed_job_records();
        // The landing record is written 6 minutes after the verifier ended: the CI wait, push and merge.
        records.landings = vec![landing_of(LANDED_JOB, t("2026-10-29T10:23:00Z"))];
        let landed = landed_jobs(&records);
        assert_eq!(landed.len(), 1);
        assert!(landed[0].landing_found);
        assert!((landed[0].stages.landing_and_rebuild - 6.0).abs() < 1e-6);
        let throughput = Throughput::compute(&records).unwrap();
        assert!((throughput.median_stages.landing_and_rebuild - 6.0).abs() < 1e-6);
        assert_eq!(throughput.window_with_landing_record, 1);
        // The wait for the landing record is time spent, so it is not idle time.
        assert_eq!(throughput.idle_minutes, 0.0);
    }

    #[test]
    fn a_landing_record_of_another_job_does_not_extend_the_landing_stage() {
        let mut records = landed_job_records();
        records.landings = vec![landing_of(
            "20261029-110000-pth1",
            t("2026-10-29T10:23:00Z"),
        )];
        let landed = landed_jobs(&records);
        assert!(!landed[0].landing_found);
        assert!((landed[0].stages.landing_and_rebuild - 0.1).abs() < 1e-6);
    }

    /// A scratch repository holding `records` at their canonical paths, `tree_landings` under `landings/`,
    /// and `branch_landings` on the local records branch only, as landings are written after a merge.
    fn scratch_repo(
        records: &[AnyRecord],
        tree_landings: &[athena_data::history::Landing],
        branch_landings: &[athena_data::history::Landing],
    ) -> (tempfile::TempDir, athena_git::Git) {
        use athena_data::history::HistoryRecord;
        let dir = tempfile::tempdir().unwrap();
        let git = athena_git::Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        let put = |path: &str, text: &str| {
            let full = dir.path().join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, text).unwrap();
        };
        for record in records {
            put(&record.canonical_path(), &record.to_json());
        }
        let landing_json = |l: &athena_data::history::Landing| {
            serde_json::to_string(&HistoryRecord::Landing(l.clone())).unwrap()
        };
        for l in tree_landings {
            put(&format!("landings/{}.json", l.id), &landing_json(l));
        }
        put("README.md", "scratch\n");
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "Records\n\nAthena-Job: bootstrap"])
            .unwrap();
        let writer = athena_git::records_branch::RecordsWriter::new(&git);
        for l in branch_landings {
            writer
                .append(
                    "landing",
                    &[athena_git::records_branch::Change::put(
                        format!("landings/{}.json", l.id),
                        landing_json(l),
                    )],
                )
                .unwrap();
        }
        (dir, git)
    }

    #[test]
    fn the_snapshot_gives_the_same_throughput_section_as_the_main_checkout() {
        let landed = OutcomeState::Landed;
        let jobs = [
            ("20261029-100000-mnd1", "2026-10-29T10:00:00Z", landed),
            ("20261029-120000-mnd2", "2026-10-29T12:00:00Z", landed),
            ("20261029-140000-mnd3", "2026-10-29T14:00:00Z", landed),
            (
                "20261029-160000-mnd4",
                "2026-10-29T16:00:00Z",
                OutcomeState::Failed,
            ),
        ];
        let mut records = Vec::new();
        for (job, start, state) in jobs {
            records.extend(timed_job(
                job,
                t(start),
                minutes(1.0, 0.0, 10.0, 2.0, 5.0, 0.1),
                state,
            ));
        }
        // Two landing records are in the tree and the third is on the records branch only: the snapshot
        // must hold all three, the same two sources the checkout is read from.
        // A job landing names the verification it rests on, or the record is not acceptable.
        let landing = |job: &str, at: &str| {
            let mut l = landing_of(job, t(at));
            l.commits[0].verification = Some(athena_data::RecordId::generate(t(at)));
            l
        };
        let tree = [
            landing(jobs[0].0, "2026-10-29T10:23:00Z"),
            landing(jobs[1].0, "2026-10-29T12:25:00Z"),
        ];
        let branch = [landing(jobs[2].0, "2026-10-29T14:27:00Z")];
        let (_dir, git) = scratch_repo(&records, &tree, &branch);

        let from_checkout = Records::load(&git, "HEAD").unwrap();
        let expected = Throughput::compute(&from_checkout)
            .expect("jobs landed")
            .text();
        assert!(
            expected.contains("landing record (landings/) found for each"),
            "the fixture's landing records must be read: {expected}"
        );
        assert_eq!(from_checkout.landings.len(), 3);

        let out = tempfile::tempdir().unwrap();
        let dest = out.path().join("evidence");
        crate::snapshot::write(&git, "HEAD", &dest).unwrap();
        struct Unlock(std::path::PathBuf);
        impl Drop for Unlock {
            fn drop(&mut self) {
                let _ = crate::snapshot::unlock(&self.0);
            }
        }
        let _guard = Unlock(dest.clone());
        let view = crate::snapshot::status(&dest).unwrap();
        let from_snapshot = view.throughput.expect("jobs landed in the snapshot").text();
        assert_eq!(from_snapshot, expected);
        assert_eq!(view.landing_records, 3);
        assert_eq!(view.jobs_by_state.get("landed"), Some(&3));
        assert_eq!(view.jobs_by_state.get("failed"), Some(&1));
        assert!(
            view.invalid_records.is_empty(),
            "{:?}",
            view.invalid_records
        );
        assert_eq!(view.history.commits, 1);
        // The same records, assembled from the directory and from git.
        let from_dir = Records::load_dir(&dest).unwrap();
        assert_eq!(from_dir.jobs.len(), from_checkout.jobs.len());
        assert_eq!(
            from_dir.landings.iter().map(|l| &l.id).collect::<Vec<_>>(),
            from_checkout
                .landings
                .iter()
                .map(|l| &l.id)
                .collect::<Vec<_>>()
        );

        // A snapshot without its landing records reads differently, so the equality above is not vacuous.
        let bare = tempfile::tempdir().unwrap();
        let bare_dest = bare.path().join("evidence");
        let (_other, plain) = scratch_repo(&records, &[], &[]);
        crate::snapshot::write(&plain, "HEAD", &bare_dest).unwrap();
        let _bare_guard = Unlock(bare_dest.clone());
        let without = crate::snapshot::status(&bare_dest)
            .unwrap()
            .throughput
            .unwrap()
            .text();
        assert_ne!(without, expected);
        assert!(
            without.contains("no landing record (landings/) found"),
            "{without}"
        );
    }

    #[test]
    fn a_landed_job_with_no_landing_record_reports_the_outcome_time_and_says_so() {
        let records = landed_job_records();
        let landed = landed_jobs(&records);
        assert!(!landed[0].landing_found);
        assert!((landed[0].stages.landing_and_rebuild - 0.1).abs() < 1e-6);
        let text = Throughput::compute(&records).unwrap().text();
        assert!(
            text.contains(
                "no landing record (landings/) found, so landing ends at the landing outcome"
            ),
            "{text}"
        );
    }

    #[test]
    fn did_not_land_counts_the_time_spent_not_the_days_the_job_stayed_open() {
        let mut all = Vec::new();
        all.extend(timed_job(
            "20261029-100000-mnd1",
            t("2026-10-29T10:00:00Z"),
            minutes(0.0, 0.0, 10.0, 0.0, 0.0, 0.0),
            OutcomeState::Landed,
        ));
        // Runs 20 minutes and is closed three days later.
        all.extend(timed_job(
            "20261029-120000-pqn1",
            t("2026-10-29T12:00:00Z"),
            minutes(0.0, 0.0, 20.0, 0.0, 0.0, 3.0 * 24.0 * 60.0),
            OutcomeState::Abandoned,
        ));
        let throughput = Throughput::compute(&loaded(all)).unwrap();
        assert_eq!(throughput.unlanded_jobs, 1);
        assert_eq!(throughput.unlanded_minutes, 20.0);
    }

    #[test]
    fn idle_time_is_the_gap_between_time_spent_whatever_stays_open() {
        let mut all = Vec::new();
        all.extend(timed_job(
            "20261029-000000-jdk1",
            t("2026-10-29T00:00:00Z"),
            minutes(0.0, 0.0, 10.0, 0.0, 0.0, 0.0),
            OutcomeState::Landed,
        ));
        // Works 01:00 to 01:20, is closed three days later: its open days do not cover the next job.
        all.extend(timed_job(
            "20261029-010000-jdk2",
            t("2026-10-29T01:00:00Z"),
            minutes(0.0, 0.0, 20.0, 0.0, 0.0, 3.0 * 24.0 * 60.0),
            OutcomeState::Abandoned,
        ));
        all.extend(timed_job(
            "20261029-020000-jdk3",
            t("2026-10-29T02:00:00Z"),
            minutes(0.0, 0.0, 10.0, 0.0, 0.0, 0.0),
            OutcomeState::Landed,
        ));
        // Gaps: 00:10 to 01:00 (50) and 01:20 to 02:00 (40).
        let throughput = Throughput::compute(&loaded(all)).unwrap();
        assert_eq!(throughput.idle_minutes, 90.0);
    }

    #[test]
    fn every_printed_figure_states_its_basis_and_the_landing_basis_names_the_landing_record() {
        let mut records = landed_job_records();
        records.landings = vec![landing_of(LANDED_JOB, t("2026-10-29T10:23:00Z"))];
        let text = Throughput::compute(&records).unwrap().text();
        let lines: Vec<&str> = text.lines().collect();
        // Each figure line is followed by the line that states how it was measured.
        for (figure, basis) in [
            ("throughput 1 landed", "basis: landed jobs counted"),
            ("median per landed job", "basis: invocation wall times"),
            (
                "idle between jobs",
                "basis: gaps between merged busy intervals",
            ),
            ("did not land", "basis: time spent"),
            (
                "attempts per landed change",
                "basis: landings plus attempts",
            ),
            (
                "spent on attempts that did not land",
                "basis: the cost recorded",
            ),
            (
                "landed on the first attempt",
                "basis: landed changes whose request",
            ),
        ] {
            let at = lines
                .iter()
                .position(|l| l.trim_start().starts_with(figure))
                .unwrap_or_else(|| panic!("{figure:?} not in:\n{text}"));
            assert!(
                lines[at + 1].contains(basis),
                "{figure:?} has no basis in:\n{text}"
            );
        }
        assert!(
            text.contains("landing record (landings/) found for each"),
            "{text}"
        );
        assert!(text.contains("placed by estimate"), "{text}");
        assert!(!text.contains("have no record"), "{text}");
    }

    #[test]
    fn throughput_is_absent_before_a_landing_and_its_medians_cover_only_the_newest_twenty() {
        assert_eq!(Throughput::compute(&Records::default()), None);
        let mut all = Vec::new();
        for n in 0..22u32 {
            // The two oldest jobs took 100 minutes; the newest twenty took 10.
            let producer = if n < 2 { 100.0 } else { 10.0 };
            all.extend(timed_job(
                &format!("20261029-1000{n:02}-ab{n:02}"),
                format!("2026-10-29T{:02}:00:00Z", n).parse().unwrap(),
                minutes(0.0, 0.0, producer, 0.0, 0.0, 0.0),
                OutcomeState::Landed,
            ));
        }
        let throughput = Throughput::compute(&loaded(all)).unwrap();
        assert_eq!(throughput.landed_total, 22);
        assert_eq!(throughput.window_jobs, THROUGHPUT_WINDOW);
        assert_eq!(throughput.median_minutes, 10.0);
    }

    /// The start of landing `n` in the waste fixtures: two hours apart from 2026-10-01.
    fn landing_start(n: usize) -> jiff::Timestamp {
        t("2026-10-01T00:00:00Z")
            .checked_add(jiff::SignedDuration::from_secs(2 * 3600 * n as i64))
            .unwrap()
    }

    /// The start of failed attempt `j` after landing `n`: 30 minutes after it started, 10 minutes apart.
    fn failed_start(n: usize, j: usize) -> jiff::Timestamp {
        landing_start(n)
            .checked_add(jiff::SignedDuration::from_secs(30 * 60 + 600 * j as i64))
            .unwrap()
    }

    fn landed_id(n: usize) -> String {
        athena_data::RecordId::from_parts(landing_start(n), 0)
            .as_str()
            .to_string()
    }

    fn failed_id(n: usize, j: usize) -> String {
        athena_data::RecordId::from_parts(failed_start(n, j), 1)
            .as_str()
            .to_string()
    }

    /// `landings` landed jobs of `landed_minutes` producer minutes (landing `n` starts at hour `2n`), and
    /// `failed(n)` failed attempts of 5 producer minutes after landing `n`, each costing `usd`. Each failed
    /// attempt closes after landing `n` ends and before landing `n + 1` ends, so it belongs to landing `n`'s window.
    fn waste_records(
        landings: usize,
        landed_minutes: f64,
        failed: impl Fn(usize) -> usize,
        usd: f64,
    ) -> Records {
        let mut all = Vec::new();
        for n in 0..landings {
            all.extend(timed_job(
                &landed_id(n),
                landing_start(n),
                minutes(0.0, 0.0, landed_minutes, 0.0, 0.0, 0.0),
                OutcomeState::Landed,
            ));
            for j in 0..failed(n) {
                all.extend(timed_job(
                    &failed_id(n, j),
                    failed_start(n, j),
                    minutes(0.0, 0.0, 5.0, 0.0, 0.0, 0.0),
                    OutcomeState::Failed,
                ));
            }
        }
        let mut records = loaded(all);
        for job in records.jobs.values_mut() {
            if job
                .outcome
                .as_ref()
                .is_some_and(|o| o.state == OutcomeState::Failed)
            {
                for i in &mut job.invocations {
                    i.cost = Quantity::reported(usd, Unit::Usd, "test");
                }
            }
        }
        records
    }

    /// Makes landing `n` ask to try again on failed attempt `j` of landing `m`.
    fn retry(records: &mut Records, n: usize, m: usize, j: usize) {
        let job = records.jobs.get_mut(&landed_id(n)).unwrap();
        job.request.as_mut().unwrap().text = format!("try again on {}", failed_id(m, j));
    }

    #[test]
    fn half_of_the_attempts_not_landing() {
        // Twenty landings and twenty failed attempts: forty attempts, half of them did not land.
        let mut records = waste_records(20, 20.0, |_| 1, 0.5);
        // Ten of the landings retry the attempt that failed before them, so they are not first attempts.
        for n in 1..=10 {
            retry(&mut records, n, n - 1, 0);
        }
        let windows = waste_windows(&records, THROUGHPUT_WINDOW);
        assert_eq!(windows.len(), 1, "twenty landings are one window");
        let w = &windows[0];
        assert_eq!((w.landings, w.unlanded_attempts, w.attempts), (20, 20, 40));
        assert_eq!(w.attempts_per_landing(), 2.0);
        assert_eq!(w.first_attempt_share(), 0.5);
        assert_eq!(w.wasted_usd, 10.0);

        let throughput = Throughput::compute(&records).unwrap();
        assert_eq!(throughput.waste, *w);
        assert_eq!(throughput.waste_previous, None);
        assert_eq!(throughput.waste.attempts_per_landing(), 2.0);
        assert_eq!(throughput.waste.first_attempt_share(), 0.5);
        let text = throughput.text();
        assert!(
            text.contains("attempts per landed change: 2.00 over the last 20 landing(s)"),
            "{text}"
        );
        assert!(
            text.contains("spent on attempts that did not land: 10.00 USD"),
            "{text}"
        );
        assert!(text.contains("landed on the first attempt: 50%"), "{text}");
        assert!(
            !text.contains("landings before"),
            "no earlier window yet: {text}"
        );

        // A closed job that never ran a model is not an attempt.
        let mut with_idle = waste_records(20, 20.0, |_| 1, 0.5);
        for n in 1..=10 {
            retry(&mut with_idle, n, n - 1, 0);
        }
        let idle = failed_id(0, 5);
        let mut idle_records = timed_job(
            &idle,
            failed_start(0, 5),
            StageMinutes::default(),
            OutcomeState::Failed,
        );
        idle_records.retain(|r| !matches!(r, AnyRecord::Invocation(_)));
        for record in idle_records {
            let AnyRecord::Outcome(o) = record else {
                continue;
            };
            let job = with_idle.jobs.entry(idle.clone()).or_default();
            job.id = Some(id(&idle));
            job.outcome = Some(*o);
        }
        assert!(with_idle.jobs[&idle].invocations.is_empty());
        let after = &waste_windows(&with_idle, THROUGHPUT_WINDOW)[0];
        assert_eq!(after.attempts, 40, "a job that never ran is not an attempt");
        assert_eq!(
            Throughput::compute(&with_idle).unwrap().unlanded_jobs,
            20,
            "and it was not counted as time on jobs that did not land"
        );
    }

    #[test]
    fn with_no_failed_attempts_every_landing_is_the_first_attempt_and_nothing_is_wasted() {
        let records = waste_records(20, 20.0, |_| 0, 0.5);
        let w = &waste_windows(&records, THROUGHPUT_WINDOW)[0];
        assert_eq!(w.attempts_per_landing(), 1.0);
        assert_eq!(w.first_attempt_share(), 1.0);
        assert_eq!(w.wasted_usd, 0.0);
        let text = Throughput::compute(&records).unwrap().text();
        assert!(text.contains("attempts per landed change: 1.00"), "{text}");
        assert!(
            text.contains("spent on attempts that did not land: 0.00 USD"),
            "{text}"
        );
        assert!(text.contains("landed on the first attempt: 100%"), "{text}");
    }

    #[test]
    fn a_window_with_fewer_than_twenty_landings_is_reported_with_its_landing_count() {
        let records = waste_records(7, 20.0, |n| usize::from(n % 2 == 0), 1.0);
        let windows = waste_windows(&records, THROUGHPUT_WINDOW);
        assert_eq!(windows.len(), 1, "no window before a partial one");
        assert_eq!(windows[0].landings, 7);
        assert_eq!(windows[0].unlanded_attempts, 4);
        let text = Throughput::compute(&records).unwrap().text();
        assert!(text.contains("over the last 7 landing(s)"), "{text}");
        assert_eq!(
            Throughput::compute(&records).unwrap().bars.wasted_usd.noise,
            None
        );
    }

    #[test]
    fn a_job_still_open_is_neither_a_landing_nor_wasted() {
        let mut records = waste_records(3, 20.0, |_| 0, 0.5);
        let open = failed_id(1, 0);
        let mut open_job = timed_job(
            &open,
            failed_start(1, 0),
            minutes(0.0, 0.0, 5.0, 0.0, 0.0, 0.0),
            OutcomeState::Failed,
        );
        open_job.retain(|r| !matches!(r, AnyRecord::Outcome(_)));
        for record in open_job {
            let job = records.jobs.entry(open.clone()).or_default();
            job.id.get_or_insert_with(|| id(&open));
            match record {
                AnyRecord::Request(r) => job.request = Some(r),
                AnyRecord::Invocation(i) => job.invocations.push(*i),
                _ => {}
            }
        }
        let w = &waste_windows(&records, THROUGHPUT_WINDOW)[0];
        assert_eq!((w.landings, w.attempts), (3, 3));
    }

    #[test]
    fn windows_of_wasted_work_are_cut_by_landing_and_the_bars_come_from_the_older_windows() {
        // Four windows of 20 landings; the failed attempts per window, oldest first, are 2, 5, 2 and 10, each
        // costing 1 USD, all in the window's first landing.
        let per_window = [2usize, 5, 2, 10];
        let records = waste_records(
            80,
            20.0,
            |n| if n % 20 == 0 { per_window[n / 20] } else { 0 },
            1.0,
        );
        let windows = waste_windows(&records, THROUGHPUT_WINDOW);
        let attempts: Vec<usize> = windows.iter().map(|w| w.attempts).collect();
        assert_eq!(attempts, vec![30, 22, 25, 22], "newest first");

        let throughput = Throughput::compute(&records).unwrap();
        assert_eq!(throughput.waste_previous.as_ref().unwrap().wasted_usd, 2.0);
        // Noise of the spend: |2 - 5| and |5 - 2|, so 3 from two differences; the floor is one attempt's cost.
        let spend = throughput.bars.wasted_usd;
        assert_eq!(spend.noise, Some(3.0));
        assert_eq!(spend.differences, 2);
        assert_eq!(spend.floor, 1.0);
        // Attempts per landing: 1.10, 1.25, 1.10; the noise is the larger gap, 0.15.
        let attempts_bar = throughput.bars.attempts_per_landing;
        assert!((attempts_bar.noise.unwrap() - 0.15).abs() < 1e-9);
        assert_eq!(attempts_bar.floor, 0.05);
        // The shares and medians are flat, so their noise is zero and only the one-job floor is left.
        assert_eq!(throughput.bars.first_attempt_share.noise, Some(0.0));
        assert_eq!(throughput.bars.median_minutes.noise, Some(0.0));
        assert_eq!(throughput.bars.median_minutes.floor, 1.0);
        assert_eq!(throughput.bars.median_minutes.bar(), Some(1.0));
        assert_eq!(throughput.median_previous, Some(20.0));

        // The text puts the earlier window beside the newest.
        let text = throughput.text();
        assert!(text.contains("attempts per landed change: 1.50 over the last 20 landing(s); the 20 landings before: 1.10"), "{text}");
        assert!(text.contains("spent on attempts that did not land: 10.00 USD over the last 20 landing(s); the 20 landings before: 2.00 USD"), "{text}");

        // With three windows the noise rests on one difference; with two there is none.
        let three = Throughput::compute(&waste_records(
            60,
            20.0,
            |n| {
                if n % 20 == 0 {
                    per_window[n / 20 + 1]
                } else {
                    0
                }
            },
            1.0,
        ))
        .unwrap();
        assert_eq!(three.bars.wasted_usd.noise, Some(3.0));
        assert_eq!(three.bars.wasted_usd.differences, 1);
        let two = Throughput::compute(&waste_records(40, 20.0, |_| 1, 1.0)).unwrap();
        assert_eq!(two.bars.wasted_usd.noise, None);
        assert!(two.waste_previous.is_some());
        let measures = two.measures();
        assert_eq!(measures.len(), 4);
        assert_eq!(measures[2].worsening(), Some(0.0));
    }

    #[test]
    fn landed_jobs_among_first_ranks_all_landed_jobs_and_keeps_the_measured_ones() {
        let landed = OutcomeState::Landed;
        let mut all = Vec::new();
        // Three landed jobs that ran no model land first, then four measured ones.
        for n in 0..3u32 {
            all.extend(timed_job(
                &format!("20261029-0{n}0000-aa{n:02}"),
                format!("2026-10-29T0{n}:00:00Z").parse().unwrap(),
                StageMinutes::default(),
                landed,
            ));
        }
        for n in 3..7u32 {
            all.extend(timed_job(
                &format!("20261029-0{n}0000-aa{n:02}"),
                format!("2026-10-29T0{n}:00:00Z").parse().unwrap(),
                minutes(0.0, 0.0, 10.0 * f64::from(n), 0.0, 0.0, 0.0),
                landed,
            ));
        }
        let records = loaded(all);
        let throughput = Throughput::compute(&records).unwrap();
        assert_eq!(throughput.landed_total, 7);
        assert_eq!(landed_jobs(&records).len(), 4);

        // The first five landings are the three unmeasured jobs and the first two measured ones. Cutting
        // `landed_jobs` by five would instead keep four measured jobs.
        let first = landed_jobs_among_first(&records, 5);
        let ids: Vec<&str> = first.iter().map(|j| j.id.as_str()).collect();
        assert_eq!(ids, vec!["20261029-030000-aa03", "20261029-040000-aa04"]);
        assert_eq!(landed_jobs_among_first(&records, 7).len(), 4);
        assert!(landed_jobs_among_first(&records, 3).is_empty());
        assert!(landed_jobs_among_first(&records, 0).is_empty());

        // The per-stage median is the one `Throughput::compute` reports.
        let window = landed_jobs(&records);
        assert_eq!(StageMinutes::median_of(&window), throughput.median_stages);
        assert_eq!(StageMinutes::median_of(&[]), StageMinutes::default());
    }
}

#[cfg(test)]
pub(crate) mod reading_rule_tests {
    use super::*;
    use athena_data::JobId;
    use athena_data::records::{ReadingAttempt, ReadingRefusal};

    pub(crate) const RULE_A: &str = "check_governance_coverage";
    pub(crate) const RULE_B: &str = "check_code_survey";

    /// A refusal as a rule name and its text; `None` is an accepted reading.
    pub(crate) type Refusal<'a> = Option<(&'a str, &'a str)>;

    pub(crate) fn job_id(n: u32) -> JobId {
        JobId::parse(&format!("20261001-{n:06}-aaaa")).unwrap()
    }

    /// A reader invocation recorded as `attempt` (1 or 2) of its run, started `at` seconds after the epoch.
    pub(crate) fn reading_invocation(
        n: u32,
        seq: u32,
        at: i64,
        attempt: u32,
        refusal: Refusal,
    ) -> Invocation {
        let mut inv = planning_cost_tests::invocation(&job_id(n), seq, Role::Reader, 0.02);
        inv.started_at = jiff::Timestamp::from_second(at).unwrap();
        inv.reading_attempt = Some(ReadingAttempt {
            attempt,
            refusal: refusal.map(|(rule, text)| ReadingRefusal {
                rule: rule.into(),
                text: text.into(),
            }),
        });
        inv
    }

    /// The reader invocations of job `n`: a first reading started at `at`, and its re-read when `second` is
    /// given (`Some(None)` is a re-read that was accepted).
    pub(crate) fn reading_job(
        n: u32,
        at: i64,
        first: Refusal,
        second: Option<Refusal>,
    ) -> Vec<Invocation> {
        let mut out = vec![reading_invocation(n, 1, at, 1, first)];
        if let Some(second) = second {
            out.push(reading_invocation(n, 2, at + 1, 2, second));
        }
        out
    }

    pub(crate) fn records_of(invocations: Vec<Invocation>) -> Records {
        let mut records = Records::default();
        for inv in invocations {
            records
                .jobs
                .entry(inv.job.as_str().to_string())
                .or_insert_with(|| JobRecords {
                    id: Some(inv.job.clone()),
                    ..JobRecords::default()
                })
                .invocations
                .push(inv);
        }
        records
    }

    /// Ten first readings. Rule A refuses three of them: the first of these is accepted when read again, the
    /// second is refused again by A and the third is accepted when read again. Rule B refuses one, which is
    /// accepted when read again. Six are accepted at once.
    fn rule_fixture() -> Records {
        let mut all = Vec::new();
        all.extend(reading_job(1, 10, Some((RULE_A, "a one")), Some(None)));
        all.extend(reading_job(
            2,
            20,
            Some((RULE_A, "a two")),
            Some(Some((RULE_A, "a two"))),
        ));
        all.extend(reading_job(3, 30, Some((RULE_A, "a two")), Some(None)));
        all.extend(reading_job(4, 40, Some((RULE_B, "b one")), Some(None)));
        for n in 5..=10 {
            all.extend(reading_job(n, 10 * i64::from(n), None, None));
        }
        records_of(all)
    }

    #[test]
    fn each_rule_is_reported_with_its_first_and_second_refusal_counts() {
        let stats = ReadingRuleStats::compute(&rule_fixture()).unwrap();
        assert_eq!(stats.first_readings, 10);
        assert_eq!(stats.second_readings, 4);
        let counts: Vec<(&str, usize, usize)> = stats
            .rules
            .iter()
            .map(|r| (r.rule.as_str(), r.first_refused, r.second_refused))
            .collect();
        assert_eq!(counts, vec![(RULE_A, 3, 1), (RULE_B, 1, 0)]);
        // The text refused most often is kept per rule.
        assert_eq!(stats.rules[0].top_text, "a two");
        assert_eq!(stats.rules[0].top_text_count, 2);
        assert_eq!(stats.rules[1].top_text, "b one");
        let text = stats.text();
        assert!(
            text.starts_with("readings    last 10 first readings (4 read again)"),
            "{text}"
        );
        assert!(
            text.contains(&format!("{RULE_A}: 3 first · 1 second")),
            "{text}"
        );
        assert!(
            text.contains(&format!("{RULE_B}: 1 first · 0 second")),
            "{text}"
        );
    }

    #[test]
    fn a_second_refusal_by_another_rule_counts_under_that_rule() {
        let records = records_of(reading_job(
            1,
            10,
            Some((RULE_A, "a")),
            Some(Some((RULE_B, "b"))),
        ));
        let stats = ReadingRuleStats::compute(&records).unwrap();
        let counts: Vec<(&str, usize, usize)> = stats
            .rules
            .iter()
            .map(|r| (r.rule.as_str(), r.first_refused, r.second_refused))
            .collect();
        assert_eq!(counts, vec![(RULE_A, 1, 0), (RULE_B, 0, 1)]);
    }

    #[test]
    fn a_tie_between_refusal_texts_goes_to_the_most_recent() {
        let mut all = Vec::new();
        all.extend(reading_job(1, 10, Some((RULE_A, "older")), None));
        all.extend(reading_job(2, 20, Some((RULE_A, "newer")), None));
        let stats = ReadingRuleStats::compute(&records_of(all)).unwrap();
        assert_eq!(stats.rules[0].top_text, "newer");
        assert_eq!(stats.rules[0].top_text_count, 1);
    }

    /// 55 first readings. In job order `n`, job 1 is the newest by start time and job 55 the oldest, so the
    /// window cuts by time and not by job id.
    fn many_readings(refused: impl Fn(u32) -> bool) -> Records {
        let mut all = Vec::new();
        for n in 1..=55u32 {
            let first = refused(n).then_some(("check_x", "x refused"));
            all.extend(reading_job(n, 1000 - i64::from(n) * 10, first, None));
        }
        records_of(all)
    }

    #[test]
    fn only_the_newest_fifty_first_readings_are_counted() {
        // The five oldest by time (jobs 51 to 55) are refused; they fall outside the window.
        let stats = ReadingRuleStats::compute(&many_readings(|n| n > 50)).unwrap();
        assert_eq!(stats.first_readings, READING_WINDOW);
        assert_eq!(stats.rules, vec![]);
        // Five of the newest (jobs 1 to 5) are refused as well: exactly those five count.
        let stats = ReadingRuleStats::compute(&many_readings(|n| n > 50 || n <= 5)).unwrap();
        assert_eq!(stats.first_readings, 50);
        assert_eq!(stats.rules.len(), 1);
        assert_eq!(stats.rules[0].first_refused, 5);
        assert_eq!(stats.rules[0].second_refused, 0);
    }

    #[test]
    fn a_re_read_belongs_to_the_window_of_its_first_reading() {
        // Job 55 is the oldest: outside the window, so its re-read is not counted either.
        let mut all = Vec::new();
        for n in 1..=55u32 {
            let at = 1000 - i64::from(n) * 10;
            all.extend(reading_job(n, at, Some(("check_x", "x")), Some(None)));
        }
        let stats = ReadingRuleStats::compute(&records_of(all)).unwrap();
        assert_eq!(stats.first_readings, 50);
        assert_eq!(stats.second_readings, 50);
        assert_eq!(stats.rules[0].first_refused, 50);
    }

    #[test]
    fn records_without_a_reading_attempt_give_no_stats() {
        assert_eq!(ReadingRuleStats::compute(&Records::default()), None);
        let mut plain = planning_cost_tests::invocation(&job_id(1), 1, Role::Reader, 0.02);
        plain.reading_attempt = None;
        assert_eq!(ReadingRuleStats::compute(&records_of(vec![plain])), None);
        // A re-read alone, with no first reading before it, is not a first reading.
        let orphan = reading_invocation(1, 2, 10, 2, Some((RULE_A, "a")));
        assert_eq!(ReadingRuleStats::compute(&records_of(vec![orphan])), None);
    }

    #[test]
    fn a_window_where_nothing_was_refused_is_reported_as_such() {
        let stats = ReadingRuleStats::compute(&records_of(reading_job(1, 10, None, None))).unwrap();
        assert_eq!(stats.first_readings, 1);
        assert!(stats.rules.is_empty());
        assert!(stats.text().contains("no rule refused a reading"));
    }

    #[test]
    fn athena_status_reports_the_counts_from_the_committed_invocation_records() {
        use super::records_worktree_tests::{commit_in_tree, job_record};
        use athena_data::records::AnyRecord;
        let (_dir, _state, athena) = crate::code_health::tests::set_up();
        // With no reader invocation recorded, the field is absent.
        assert!(athena.status().unwrap().reading_rules.is_none());
        let mut files = Vec::new();
        for n in 1..=10u32 {
            files.push(job_record(job_id(n).as_str(), "docs"));
        }
        let fixture = rule_fixture();
        for job in fixture.jobs.values() {
            for inv in &job.invocations {
                files.push(AnyRecord::Invocation(Box::new(inv.clone())));
            }
        }
        commit_in_tree(&athena, &files);
        let view = athena.status().unwrap();
        let stats = view
            .reading_rules
            .clone()
            .expect("the reading rules are reported");
        assert_eq!(stats, ReadingRuleStats::compute(&fixture).unwrap());
        assert_eq!(
            (stats.rules[0].first_refused, stats.rules[0].second_refused),
            (3, 1)
        );
        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["reading_rules"]["first_readings"], 10);
    }

    // -----------------------------------------------------------------------------------------------------
    // Hestia's recurring duties

    use crate::hestia::{DUTIES, DutyContext, DutyStanding, Occasion, duty_standings};
    use athena_data::records::{
        AnyRecord, CodeHealthBaseline, DutyPass, HestiaCheckResult, HestiaFinding,
        HestiaFindingState, HestiaSweep,
    };

    fn duty_time(second: u32) -> jiff::Timestamp {
        format!("2026-10-27T09:00:{second:02}Z").parse().unwrap()
    }

    /// A duty pass at `second` with the landing total `landings`, listing `ran` as (duty id, errored).
    fn duty_sweep(second: u32, landings: u32, ran: &[(&str, bool)]) -> HestiaSweep {
        HestiaSweep {
            id: athena_data::RecordId::parse(&format!("20261027-0900{second:02}-aaaa")).unwrap(),
            at: duty_time(second),
            checks: ran
                .iter()
                .map(|(id, errored)| HestiaCheckResult {
                    check: (*id).into(),
                    examined: 1,
                    count: u32::from(*errored),
                    errored: *errored,
                })
                .collect(),
            findings: ran
                .iter()
                .filter(|(_, errored)| *errored)
                .map(|(id, _)| HestiaFinding {
                    check: (*id).into(),
                    subject: (*id).into(),
                    evidence: "the check failed to run: broken".into(),
                    age_seconds: 0,
                    state: HestiaFindingState::Open,
                })
                .collect(),
            duty_pass: Some(DutyPass {
                occasion: Occasion::Sweep,
                landings,
            }),
        }
    }

    fn baseline(second: u32, landings: u32, review_queued: bool) -> CodeHealthBaseline {
        CodeHealthBaseline {
            id: athena_data::RecordId::parse(&format!("20261027-0901{second:02}-bbbb")).unwrap(),
            at: duty_time(second),
            landings,
            review_queued,
        }
    }

    fn standing<'a>(all: &'a [DutyStanding], id: &str) -> &'a DutyStanding {
        all.iter().find(|s| s.id == id).unwrap()
    }

    fn standings(
        sweeps: &[HestiaSweep],
        baselines: &[CodeHealthBaseline],
        landings: u32,
    ) -> Vec<DutyStanding> {
        let ctx = DutyContext {
            occasion: Occasion::Sweep,
            landings,
            now: duty_time(59),
        };
        duty_standings(&DUTIES, sweeps, baselines, &ctx)
    }

    #[test]
    fn duty_standings_flag_a_duty_past_its_cadence_and_not_one_within_it() {
        let sweeps = [
            duty_sweep(
                1,
                0,
                &[
                    ("error-review", false),
                    ("evidence-labels", false),
                    ("keep-rule", false),
                    ("reading-rule-refusals", false),
                ],
            ),
            // The newest attempt of the evidence labels failed.
            duty_sweep(2, 4, &[("evidence-labels", true), ("code-health", false)]),
        ];
        let baselines = [baseline(1, 2, true)];
        let all = standings(&sweeps, &baselines, 5);
        assert_eq!(
            all.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
            DUTIES.iter().map(|d| d.id).collect::<Vec<_>>(),
            "every duty is shown, in table order"
        );

        // A drain-start duty that ran is never overdue by age, and says when it runs next.
        let review = standing(&all, "error-review");
        assert_eq!(review.last_run, Some(duty_time(1)));
        assert!(!review.overdue);
        assert_eq!(review.next_due, "at the next drain start");
        // One whose newest attempt failed is overdue and shows when it failed; it still has its last good run.
        let labels = standing(&all, "evidence-labels");
        assert!(labels.overdue);
        assert_eq!(labels.last_failed_at, Some(duty_time(2)));
        assert_eq!(labels.last_run, Some(duty_time(1)));
        assert_eq!(labels.next_due, "due now (last attempt failed)");
        // A per-landing duty recorded at 4 landings with 5 now is past its cadence.
        let health = standing(&all, "code-health");
        assert!(health.overdue);
        assert_eq!(health.last_run, Some(duty_time(2)));
        assert!(
            health.next_due.starts_with("due now"),
            "{}",
            health.next_due
        );
        // Every 10 landings, recorded at 0, with 5 now: within the cadence.
        let keep = standing(&all, "keep-rule");
        assert!(!keep.overdue);
        assert_eq!(keep.next_due, "after 5 more landings");
        // Every 5 landings, recorded at 0: 5 now is past it, 4 is not.
        assert!(standing(&all, "reading-rule-refusals").overdue);
        let early = standings(&sweeps, &baselines, 4);
        let reading = standing(&early, "reading-rule-refusals");
        assert!(!reading.overdue);
        assert_eq!(reading.next_due, "after 1 more landing");
        // The same total with the duty never run: overdue with no last run.
        let none = standings(&[], &baselines, 4);
        let never = standing(&none, "reading-rule-refusals");
        assert!(never.overdue);
        assert_eq!(never.last_run, None);
        assert_eq!(never.next_due, "due now (never run)");
        // An old sweep that carries no duty pass counts as no run.
        let mut old = duty_sweep(3, 0, &[("keep-rule", false)]);
        old.duty_pass = None;
        assert!(standing(&standings(&[old], &baselines, 0), "keep-rule").overdue);
    }

    #[test]
    fn the_design_review_is_counted_from_the_code_health_baseline() {
        // A baseline at 2 landings that queued a review: 11 landings is 9 later, within the cadence of 10.
        let reviewed = [baseline(1, 2, true)];
        let within = standings(&[], &reviewed, 11);
        let review = standing(&within, "design-review");
        assert!(!review.overdue);
        assert_eq!(review.last_run, Some(duty_time(1)));
        assert_eq!(review.next_due, "after 1 more landing");
        // At 12 landings the review is due.
        assert!(standing(&standings(&[], &reviewed, 12), "design-review").overdue);
        // A first baseline queued no review: never reviewed, and not overdue until the cadence passes.
        let first = [baseline(2, 3, false)];
        let fresh = standings(&[], &first, 4);
        let review = standing(&fresh, "design-review");
        assert_eq!(review.last_run, None);
        assert!(!review.overdue);
        assert!(standing(&standings(&[], &first, 13), "design-review").overdue);
        // No baseline at all: the code-health check has not run, so nothing counts the review yet.
        assert!(standing(&standings(&[], &[], 0), "design-review").overdue);
    }

    #[test]
    fn status_shows_each_duty_with_its_last_run_and_next_due_point_and_flags_the_late_ones() {
        use super::records_worktree_tests::commit_in_tree;
        let (_dir, _state, athena) = crate::code_health::tests::set_up();
        // Nothing recorded yet: every duty is shown, and every duty is overdue because none has run.
        let view = athena.status().unwrap();
        assert_eq!(view.duties.len(), DUTIES.len());
        assert!(
            view.duties
                .iter()
                .all(|d| d.overdue && d.last_run.is_none())
        );
        // A sweep that ran the drain-start duties and the code-health check, with no landing yet.
        let sweep = duty_sweep(
            1,
            0,
            &[
                ("error-review", false),
                ("evidence-labels", false),
                ("code-health", false),
            ],
        );
        commit_in_tree(&athena, &[AnyRecord::HestiaSweep(Box::new(sweep.clone()))]);
        let view = athena.status().unwrap();
        for id in ["error-review", "evidence-labels", "code-health"] {
            let duty = standing(&view.duties, id);
            assert_eq!(duty.last_run, Some(sweep.at), "{id}");
            assert!(!duty.overdue, "{id} ran within its cadence");
            assert!(
                !duty.next_due.starts_with("due now"),
                "{id}: {}",
                duty.next_due
            );
        }
        for id in ["keep-rule", "reading-rule-refusals", "design-review"] {
            let duty = standing(&view.duties, id);
            assert!(duty.overdue, "{id} never ran");
            assert_eq!(duty.last_run, None, "{id}");
        }
        let json = serde_json::to_value(&view).unwrap();
        let listed = json["duties"].as_array().unwrap();
        assert_eq!(listed.len(), DUTIES.len());
        assert_eq!(listed[0]["id"], "error-review");
        assert_eq!(listed[0]["overdue"], false);
        assert_eq!(listed[3]["id"], "design-review");
        assert_eq!(listed[3]["overdue"], true);
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
