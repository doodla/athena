//! Throughput-review, build-measurement and Hestia-sweep records.

use super::*;

/// The schema tag of a throughput review.
pub const THROUGHPUT_REVIEW_SCHEMA: &str = "athena.throughput-review/1";

/// Why a throughput review ran.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThroughputTrigger {
    /// Ten more landings since the previous review (or the first ten ever).
    Cadence,
    /// The median minutes per landed job grew by more than 20% against the previous review. A rise of more
    /// than 25% also gets a slowdown request; every review compares the stages, whatever the trigger.
    Worsened,
    /// Wasted work rose by more than 20% against the previous review: attempts per landed change, money
    /// spent on attempts that did not land, or the share of changes that landed on the first attempt fell.
    Wasted,
}

impl ThroughputTrigger {
    pub fn as_str(self) -> &'static str {
        match self {
            ThroughputTrigger::Cadence => "cadence",
            ThroughputTrigger::Worsened => "worsened",
            ThroughputTrigger::Wasted => "wasted",
        }
    }
}

/// The three measures of wasted work over a window of landings, as a throughput review saw them.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WasteFigures {
    /// Attempts (landings plus attempts that did not land) per landing; at least 1.
    pub attempts_per_landing: f64,
    /// Money spent on attempts that did not land, in US dollars.
    pub wasted_usd: f64,
    /// The share of landed changes that landed on their first attempt, from 0 to 1.
    pub first_attempt_share: f64,
}

impl WasteFigures {
    /// Whether the figures are numbers a review can hold.
    pub fn is_valid(&self) -> bool {
        self.attempts_per_landing.is_finite()
            && self.attempts_per_landing >= 1.0
            && self.wasted_usd.is_finite()
            && self.wasted_usd >= 0.0
            && self.first_attempt_share.is_finite()
            && (0.0..=1.0).contains(&self.first_attempt_share)
    }
}

/// The top proposal of a throughput review: the biggest wait and the change that removes it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThroughputProposal {
    /// The biggest bottleneck, in one or two sentences.
    pub bottleneck: String,
    /// The stage it sits in (reading, planning, producer, checks, verifier, landing or idle).
    pub stage: String,
    /// The architectural change that removes it.
    pub change: String,
    /// The prediction: minutes saved per landed job once the change lands.
    pub expected_minutes_saved: f64,
    /// The request priority the prediction set (`urgent`, `high`, `normal` or `low`).
    pub priority: String,
    /// The design request this review queued; None when the change was not beyond the noise between recent
    /// windows or the request could not be queued (the review's note says which).
    pub request_job: Option<JobId>,
}

/// What a later review measured for a change an earlier review cited: the prediction set against the minutes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThroughputFollowup {
    /// The review whose prediction this closes.
    pub review: RecordId,
    /// The landed job that cited the review.
    pub change_job: JobId,
    /// Minutes saved per landed job that review predicted.
    pub predicted_minutes_saved: f64,
    /// The median minutes per landed job when that review ran.
    pub baseline_median_minutes: f64,
    /// The median minutes per landed job over the jobs that landed after the change.
    pub measured_median_minutes: f64,
    /// Baseline minus measured: positive is faster.
    pub measured_minutes_saved: f64,
    /// How many landed jobs the measured median covers.
    pub jobs_measured: u32,
}

/// `athena.throughput-review/1`: one throughput review. `landings` and `window_median_minutes` are the
/// cadence point the next review counts from; the proposal is the prediction; `followups` close earlier
/// predictions. Reviews are never edited; the latest is the newest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThroughputReview {
    pub id: RecordId,
    pub at: Timestamp,
    /// Landed request jobs counted when the review ran.
    pub landings: u32,
    /// The median minutes per landed job over the newest landed jobs when the review ran.
    pub window_median_minutes: f64,
    pub trigger: ThroughputTrigger,
    /// None when the model gave no usable proposal.
    pub proposal: Option<ThroughputProposal>,
    /// The wasted-work figures over the newest window of landings when the review ran; the next review
    /// compares with them. None on a review written before they were measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waste: Option<WasteFigures>,
    /// Predictions this review closed.
    pub followups: Vec<ThroughputFollowup>,
    /// Why there is no proposal or no request, or any other remark; empty when there is nothing to say. It also
    /// names the stages whose median minutes grew since the previous review and the slowdown request the
    /// review queued, when it queued one.
    pub note: String,
}

impl ThroughputReview {
    /// The most recent of `reviews`, by time and then id; None when there are none.
    pub fn latest(reviews: &[ThroughputReview]) -> Option<&ThroughputReview> {
        reviews
            .iter()
            .max_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)))
    }

    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        let good = |v: f64| v.is_finite() && v >= 0.0;
        if !good(self.window_median_minutes) {
            return Err("a throughput review's median is a finite, non-negative number".into());
        }
        if self.waste.is_some_and(|w| !w.is_valid()) {
            return Err(
                "a throughput review's wasted-work figures are finite, with at least one attempt per landing, \
                 non-negative spend and a first-attempt share from 0 to 1"
                    .into(),
            );
        }
        if let Some(p) = &self.proposal {
            if p.bottleneck.trim().is_empty() || p.change.trim().is_empty() {
                return Err("a throughput proposal names its bottleneck and its change".into());
            }
            if !good(p.expected_minutes_saved) {
                return Err("expected minutes saved is a finite, non-negative number".into());
            }
            if !PRIORITY_LEVELS.contains(&p.priority.as_str()) {
                return Err(format!(
                    "priority {:?} is not one of {}",
                    p.priority,
                    PRIORITY_LEVELS.join(", ")
                ));
            }
        }
        for f in &self.followups {
            if !good(f.predicted_minutes_saved)
                || !good(f.baseline_median_minutes)
                || !good(f.measured_median_minutes)
                || !f.measured_minutes_saved.is_finite()
            {
                return Err("a throughput follow-up holds finite minutes".into());
            }
            if f.jobs_measured == 0 {
                return Err("a throughput follow-up measures at least one landed job".into());
            }
        }
        Ok(())
    }
}

/// The schema tag of a build measurement.
pub const BUILD_MEASUREMENT_SCHEMA: &str = "athena.build-measurement/1";

/// What `sccache` counted during one build: compile requests it answered from its cache and those it had
/// to compile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SccacheCounts {
    pub hits: u64,
    pub misses: u64,
}

/// The compile-cache figures of a build measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SccacheFigures {
    /// `sccache` was not on the PATH, so both builds compiled everything themselves. This is not a count of
    /// zero hits.
    NotInstalled,
    /// The counts of each build; the second build running after the first shows whether the cache answers
    /// across worktrees.
    Measured {
        first: SccacheCounts,
        second: SccacheCounts,
    },
}

/// The whole-build peaks of one cold build: totals over every process of the build, not the figures of one
/// process.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildPeak {
    /// The highest resident memory, summed over all processes of the build at one moment, in decimal
    /// gigabytes.
    pub memory_gb: f64,
    /// The highest number of build processes alive at one moment.
    pub processes: u32,
}

/// `athena.build-measurement/1`: the size and time of a cold workspace build, measured by
/// `athena measure-build` in two separate worktrees with different target directories. Measurements are
/// never edited; the latest is the newest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildMeasurement {
    pub id: RecordId,
    pub at: Timestamp,
    /// The commit whose tree was built.
    pub commit: String,
    /// The git blob id of `Cargo.lock` at that commit; a new measurement is due when it changes.
    pub cargo_lock_hash: String,
    /// T: bytes in one cold `target/` directory after the first build.
    pub target_bytes: u64,
    /// S: bytes of the tracked files of one worktree.
    pub source_bytes: u64,
    /// Wall time of the first cold build, in seconds.
    pub first_build_seconds: f64,
    /// Wall time of the second cold build, in seconds.
    pub second_build_seconds: f64,
    pub sccache: SccacheFigures,
    /// Peak memory and process count of the first cold build; absent in records made before they were
    /// measured, or where the platform gave no process list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_build_peak: Option<BuildPeak>,
    /// The same for the second cold build.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub second_build_peak: Option<BuildPeak>,
}

impl BuildMeasurement {
    /// The most recent of `measurements`, by time and then id; None when there are none.
    pub fn latest(measurements: &[BuildMeasurement]) -> Option<&BuildMeasurement> {
        measurements
            .iter()
            .max_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)))
    }

    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        let good = |v: f64| v.is_finite() && v >= 0.0;
        if !good(self.first_build_seconds) || !good(self.second_build_seconds) {
            return Err("a build measurement's times are finite, non-negative seconds".into());
        }
        if self.commit.trim().is_empty() || self.cargo_lock_hash.trim().is_empty() {
            return Err("a build measurement names its commit and its Cargo.lock hash".into());
        }
        for peak in [self.first_build_peak, self.second_build_peak]
            .into_iter()
            .flatten()
        {
            if !good(peak.memory_gb) {
                return Err("a build's peak memory is a finite, non-negative size".into());
            }
            if peak.processes == 0 {
                return Err("a build's peak process count is at least one".into());
            }
        }
        Ok(())
    }
}

/// The schema tag of a Hestia sweep.
pub const HESTIA_SWEEP_SCHEMA: &str = "athena.hestia-sweep/1";

/// Where a Hestia finding stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HestiaFindingState {
    /// Reported and not yet seen to be fixed.
    Open,
    /// Seen to be fixed by a later sweep.
    Resolved,
}

impl HestiaFindingState {
    pub fn as_str(self) -> &'static str {
        match self {
            HestiaFindingState::Open => "open",
            HestiaFindingState::Resolved => "resolved",
        }
    }
}

/// One thing a Hestia check reports (`docs/design/hestia.md` section 9, job 1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HestiaFinding {
    /// The id of the check that reported it.
    pub check: String,
    /// What the finding is about: a branch, a path, a request, a binary.
    pub subject: String,
    /// What the check saw, or the error it hit. Never empty.
    pub evidence: String,
    /// How long the problem has stood, in whole seconds; 0 for one first seen in this sweep.
    pub age_seconds: u64,
    pub state: HestiaFindingState,
}

/// What one check did in one sweep. A check that ran and found nothing is listed with a count of 0.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HestiaCheckResult {
    pub check: String,
    /// How many things the check looked at.
    pub examined: u32,
    /// How many findings the check reported; equals the sweep's findings for this check.
    pub count: u32,
    /// True when the check failed to run. Its error is then one of its findings, so it never looks clean.
    pub errored: bool,
}

/// `athena.hestia-sweep/1`: one run of Hestia's checks. Sweeps are never edited; the latest is the newest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HestiaSweep {
    pub id: RecordId,
    pub at: Timestamp,
    /// Every check that was registered, in order, with its count. A drain-start or landing pass
    /// (`duty_pass`) lists only the duties that ran in it.
    pub checks: Vec<HestiaCheckResult>,
    pub findings: Vec<HestiaFinding>,
    /// Set by a pass over Hestia's recurring duties: where it was started from and the landing total at that
    /// moment, which is what "every N landings" counts from. A duty ran in this pass when its id is listed in
    /// `checks` and not errored. Absent in sweeps written before duties were scheduled.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duty_pass: Option<DutyPass>,
}

/// Where a pass over the recurring duties was started from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DutyOccasion {
    /// The start of a drain.
    DrainStart,
    /// A job landed.
    Landing,
    /// An explicit `athena hestia sweep`.
    Sweep,
}

/// The mark a duty pass leaves in a sweep record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DutyPass {
    pub occasion: DutyOccasion,
    /// How many landings stood on the branch when the pass ran.
    pub landings: u32,
}

impl HestiaSweep {
    /// Whether this record is a drain-start or landing pass, which lists only the duties that ran in it and so
    /// is not a full sweep.
    pub fn is_duty_pass(&self) -> bool {
        self.duty_pass
            .is_some_and(|p| matches!(p.occasion, DutyOccasion::DrainStart | DutyOccasion::Landing))
    }

    /// The most recent of `sweeps`, by time and then id; None when there are none.
    pub fn latest(sweeps: &[HestiaSweep]) -> Option<&HestiaSweep> {
        sweeps
            .iter()
            .max_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)))
    }

    /// Why this record is not acceptable, if it is not.
    pub fn validate(&self) -> Result<(), String> {
        for (i, c) in self.checks.iter().enumerate() {
            if c.check.trim().is_empty() {
                return Err("every check has an id".into());
            }
            if self.checks[..i].iter().any(|p| p.check == c.check) {
                return Err(format!("check {} is listed twice", c.check));
            }
            let reported = self.findings.iter().filter(|f| f.check == c.check).count();
            if reported != c.count as usize {
                return Err(format!(
                    "check {} counts {} finding(s) but the sweep holds {reported}",
                    c.check, c.count
                ));
            }
        }
        for f in &self.findings {
            if !self.checks.iter().any(|c| c.check == f.check) {
                return Err(format!(
                    "a finding names check {}, which is not listed",
                    f.check
                ));
            }
            if f.subject.trim().is_empty() || f.evidence.trim().is_empty() {
                return Err("every finding has a subject and evidence".into());
            }
        }
        Ok(())
    }
}
