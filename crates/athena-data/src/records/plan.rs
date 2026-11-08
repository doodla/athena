//! Job-plan, critique, baseline and replay records.

use super::*;

// ---------------------------------------------------------------------------------------------------------
// Job planning (`docs/design/job-planning.md` sections 3 to 5)

pub const JOB_PLAN_SCHEMA: &str = "athena.job-plan/1";
pub const CRITIQUE_SCHEMA: &str = "athena.critique/1";

/// An option the planner weighed and turned down.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RejectedOption {
    pub option: String,
    pub why: String,
}

/// One design choice in a plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanDecision {
    pub choice: String,
    pub reason: String,
    pub rejected: Vec<RejectedOption>,
}

/// A file the plan touches. Athena, not the planner, sets `governance` and `exists`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanFile {
    pub path: String,
    pub action: DeliverableAction,
    pub purpose: String,
    pub governance: bool,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanRisk {
    pub risk: String,
    pub mitigation: String,
}

/// What kind of thing relies on what a plan changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependentKind {
    /// Code that reads or calls the changed thing.
    Code,
    /// A rule: a decision, a verdict rule, an admission check or a gate check.
    Rule,
    /// A host setting, such as a workflow, a config key or a budget.
    HostSetting,
}

/// Something that reads or relies on what the plan changes, with the test that covers it or the reason it is
/// unaffected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanDependent {
    /// What the plan changes: a behavior, record, config key, check name or host setting.
    pub name: String,
    /// The code, rule or host setting that reads or relies on it.
    pub dependent: String,
    pub kind: DependentKind,
    /// The test that covers the dependent after the change.
    pub test: Option<String>,
    /// Why the dependent is unaffected.
    pub reason: Option<String>,
}

/// How one acceptance criterion will be shown to hold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Demonstration {
    /// The criterion's number in the reading, from 1.
    pub criterion: u32,
    pub method: String,
    /// The command or test that shows it.
    pub command: String,
    /// What running it observes.
    pub observes: String,
    /// What would fail if the behavior were missing; a behavior demonstration needs it.
    pub fails_without: Option<String>,
    /// The test seam the demonstration drives; a behavior demonstration needs it. It names the production
    /// entry point called, the fake or fixture used and how it is injected, and either "exists" or the plan
    /// step that adds it. Records written before the field existed lack it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seam: Option<String>,
}

/// `athena.job-plan/1`: one round of the planner's plan, at `<job>/job-plan-<round>.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobPlan {
    pub job: JobId,
    /// From 1.
    pub round: u32,
    pub at: Timestamp,
    /// The planner's invocation, as numbered in the job's `invocations/` directory.
    pub invocation_seq: u32,
    pub approach: String,
    pub decisions: Vec<PlanDecision>,
    pub files: Vec<PlanFile>,
    pub steps: Vec<String>,
    pub risks: Vec<PlanRisk>,
    /// Who relies on what the plan changes, each with a test or a reason it is unaffected. Athena refuses a
    /// new plan without it. Records written before the field existed lack it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependents: Vec<PlanDependent>,
    pub demonstrations: Vec<Demonstration>,
    pub open_questions: Vec<String>,
    /// The SHA-256 of the section of the producer's prompt that code renders from this plan.
    pub producer_section_sha256: String,
    /// Set on the first plan of a job that resumed planning from the kept plan of this earlier job, which
    /// stopped at the planning round cap. Absent on every other plan.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resumed_from: Option<JobId>,
}

impl JobPlan {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if self.round == 0 {
            return Err("a job plan's round starts at 1".into());
        }
        if self.approach.trim().is_empty() {
            return Err("a job plan states its approach".into());
        }
        if self.demonstrations.iter().any(|d| d.criterion == 0) {
            return Err("a demonstration names its criterion by number, from 1".into());
        }
        if self.producer_section_sha256.len() != 64
            || !self
                .producer_section_sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit())
        {
            return Err("producer_section_sha256 is a SHA-256 in hexadecimal".into());
        }
        Ok(())
    }
}

/// A critique's verdict on a plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CritiqueVerdict {
    Accept,
    Revise,
    Reject,
}

/// `athena.critique/1`: the critic's review of one plan round, at `<job>/critique-<round>.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Critique {
    pub job: JobId,
    /// The round of the plan reviewed, from 1.
    pub round: u32,
    pub at: Timestamp,
    /// The critic's invocation; none when Athena's own checks stopped the round before any critic call.
    pub invocation_seq: Option<u32>,
    pub agent_verdict: Option<CritiqueVerdict>,
    pub final_verdict: CritiqueVerdict,
    /// Findings from the critic and from Athena's own checks.
    pub findings: Vec<Finding>,
    /// Set when the critic judged that the accepted reading misread the request.
    pub request_reading_mismatch: Option<String>,
    pub summary: Option<String>,
}

impl Critique {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if self.round == 0 {
            return Err("a critique's round starts at 1".into());
        }
        if self.final_verdict == CritiqueVerdict::Accept
            && self
                .findings
                .iter()
                .any(|f| matches!(f.severity, Severity::Blocker | Severity::Major))
        {
            return Err("a critique that accepts cannot hold a blocker or major finding".into());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------------------
// The baseline of checks

pub const BASELINE_SCHEMA: &str = "athena.baseline/1";

/// `athena.baseline/1`: the job's configured checks run on the base commit, before the first producer
/// invocation, at `<job>/baseline.json`. It records which checks already fail before the producer touches
/// anything. At verification, a check that failed here and fails again is marked pre-existing and does not
/// fail the verdict, unless both runs name their failing tests and the candidate fails a test the base did
/// not (see [`CheckRun::failed_tests`]).
///
/// A check that fails, times out or cannot start is recorded as not passed, with a note; none of these stops
/// the job. When the baseline could not run at all, `checks` is empty and `note` says why.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    pub job: JobId,
    /// The commit the checks ran on: the job's base commit.
    pub base_commit: String,
    /// The sequence number the first producer invocation gets. The baseline was taken before it.
    pub before_invocation: u32,
    /// One entry per configured check. No path-skipping applies, since no change list exists yet.
    pub checks: Vec<CheckRun>,
    /// Why the baseline is empty or partial, if it is.
    pub note: Option<String>,
    /// The address of the GitHub run whose green result stands in for the checks, when the baseline was
    /// reused and not run: a finished green run for the same governed content as the base commit. Every
    /// check then has `reused` set and passed. Absent (and not written) when the checks ran here, and in files
    /// written before this field existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reused_from: Option<String>,
}

/// What a verification check's note starts with when the same check already failed on the base commit.
pub const PRE_EXISTING_NOTE: &str = "pre-existing: this check already failed on the base commit";

impl Baseline {
    /// True only when this baseline ran without a note and holds a failed entry named `name`. A baseline
    /// with a note (it could not run, or is partial), an empty one, or one without that check says nothing
    /// about the check, so it is never read as a reason to excuse a failure.
    pub fn failed_before(&self, name: &str) -> bool {
        self.note.is_none() && self.checks.iter().any(|c| c.name == name && !c.passed)
    }

    /// The failing test names recorded for the check `name` on the base: empty when the baseline has a
    /// note, holds no failed entry of that name, or recorded no names for it (see
    /// [`CheckRun::failed_tests`]). The check-name rule of [`Baseline::failed_before`] is the fallback.
    pub fn failed_tests_before(&self, name: &str) -> &[String] {
        if self.note.is_some() {
            return &[];
        }
        self.checks
            .iter()
            .find(|c| c.name == name && !c.passed)
            .map_or(&[], |c| c.failed_tests.as_slice())
    }

    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if self.base_commit.trim().is_empty() {
            return Err("a baseline names the base commit it ran on".into());
        }
        let mut seen = std::collections::BTreeSet::new();
        for c in &self.checks {
            if !seen.insert(c.name.as_str()) {
                return Err(format!("a baseline lists check {} twice", c.name));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------------------------------------
// The replay of accepted readings

pub const REPLAY_SCHEMA: &str = "athena.replay/1";

/// One request that the candidate's reading rules refuse and the base's rules accepted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReplayRefusal {
    /// The job whose accepted reading is now refused.
    pub job: JobId,
    /// The candidate's refusal text.
    pub reason: String,
}

/// `athena.replay/1`: the newest accepted readings run through the base build and the candidate build of
/// `athena check-reading`, at `<job>/replay.json`. It lists the requests the candidate refuses and the base
/// accepted. A candidate that newly refuses more than a tenth of the replayed readings is held, unless its
/// acceptance criteria state that a higher refusal rate is intended; the record keeps the list either way.
///
/// When the replay could not run (no history, a build or probe failure, a failed check), `no_replay` says why,
/// `total` is 0 and the list is empty. That is a recorded absence, never a pass.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Replay {
    pub job: JobId,
    /// The base commit the base build was made from.
    pub base_commit: String,
    /// The candidate commit the candidate build was made from.
    pub candidate_commit: String,
    /// The most readings one replay uses.
    pub window: u32,
    /// How many accepted readings were replayed.
    pub total: u32,
    /// The requests the candidate refuses and the base accepted.
    pub newly_refused: Vec<ReplayRefusal>,
    /// Whether the job is held for the refusal rate.
    pub held: bool,
    /// Whether the criteria state that a higher refusal rate is intended.
    pub intended: bool,
    /// Why no replay ran, if none did.
    pub no_replay: Option<String>,
}

impl Replay {
    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        if self.window == 0 {
            return Err("a replay's window is above 0".into());
        }
        if self.newly_refused.len() > self.total as usize {
            return Err("a replay cannot refuse more requests than it replayed".into());
        }
        if self
            .no_replay
            .as_deref()
            .is_some_and(|r| r.trim().is_empty())
        {
            return Err("a replay that did not run says why".into());
        }
        if self.no_replay.is_some() && (self.total != 0 || !self.newly_refused.is_empty()) {
            return Err("a replay that did not run holds no readings and no refusals".into());
        }
        if self.held && self.newly_refused.is_empty() {
            return Err("a held replay lists the requests it refuses".into());
        }
        if self.held && self.intended {
            return Err("a replay whose criteria intend the refusals is not held".into());
        }
        Ok(())
    }
}
