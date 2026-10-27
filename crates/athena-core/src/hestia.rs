//! Hestia's sweep skeleton (`docs/design/hestia.md` section 9, job 1; decision D129).
//!
//! A sweep runs a list of [`Check`]s and records one report, an `athena.hestia-sweep/1` record: every check that
//! ran, with the number of things it looked at and the number of findings, and each finding. A check that
//! returns an error does not stop the sweep and does not look clean: [`run_sweep`] turns the error into a
//! finding for that check and marks the check as errored. Hestia reports and never gates work (D127).
//!
//! The trigger is an explicit command: `athena hestia sweep` runs and records a sweep, and `athena hestia last`
//! prints the newest one. Nothing runs a sweep on a timer or inside `athena drain`; D129 says why.
//!
//! This skeleton ships no checks. [`default_checks`] is empty, and the checks of the design's rows 1 to 8 are
//! later jobs that add themselves to it.
//!
//! Hestia also reviews errors across jobs (`docs/design/hestia.md`, "Error review"). Every failure gets an
//! introspection with a cause, but each one is written about one job. After an introspection is recorded,
//! [`Athena::error_review`] groups it with the earlier ones by cause ([`cause_key`]). A cause that has occurred
//! in two or more distinct jobs, and that no open or landed request cites, gets one urgent root-cause request,
//! built by [`crate::steward::StewardRequest`] and citing the introspection ids. [`repeated_causes`] lists the
//! repeated causes for `athena status`.

use crate::commits::{NewFile, message};
use crate::steward::StewardRequest;
use crate::store::{JobState, Records};
use crate::{Athena, Ui};
use anyhow::{Result, bail};
use athena_data::records::{
    AnyRecord, HestiaCheckResult, IntrospectionRecord, IntrospectionStatus, OutcomeState,
};
use athena_data::{JobId, RecordId};
use jiff::Timestamp;
use serde::Serialize;

pub use athena_data::records::HestiaCheckResult as CheckResult;
pub use athena_data::records::HestiaFinding as Finding;
pub use athena_data::records::HestiaFindingState as FindingState;
pub use athena_data::records::HestiaSweep as Sweep;

/// What a check saw when it ran without error.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckOutput {
    /// How many things it looked at (branches, requests, docs).
    pub examined: u32,
    pub findings: Vec<Finding>,
}

/// One Hestia check: a fact it looks at, and what is wrong with it.
pub trait Check {
    /// A short stable id, such as `leftover-branches`. Unique within a sweep.
    fn id(&self) -> &str;
    /// Looks at the system. `Err` means the check could not run, which is itself reported.
    fn run(&self, now: Timestamp) -> Result<CheckOutput>;
}

/// The checks a sweep runs when none are given. Empty in the skeleton; later jobs register theirs here.
pub fn default_checks() -> Vec<Box<dyn Check>> {
    Vec::new()
}

/// Runs every check, in order, and reports on all of them. A check that fails, or that returns a finding with
/// no subject or no evidence, becomes one open finding for that check, and the sweep goes on. Whatever check id
/// a check puts on its findings, they are filed under the check that returned them.
pub fn run_sweep(checks: &[Box<dyn Check>], id: RecordId, at: Timestamp) -> Sweep {
    let mut results = Vec::new();
    let mut findings = Vec::new();
    for check in checks {
        let check_id = check.id().to_string();
        let outcome = check.run(at).and_then(|output| {
            if output
                .findings
                .iter()
                .any(|f| f.subject.trim().is_empty() || f.evidence.trim().is_empty())
            {
                bail!("a finding came back with no subject or no evidence");
            }
            Ok(output)
        });
        match outcome {
            Ok(output) => {
                results.push(CheckResult {
                    check: check_id.clone(),
                    examined: output.examined,
                    count: output.findings.len() as u32,
                    errored: false,
                });
                findings.extend(output.findings.into_iter().map(|f| Finding {
                    check: check_id.clone(),
                    ..f
                }));
            }
            Err(error) => {
                results.push(HestiaCheckResult {
                    check: check_id.clone(),
                    examined: 0,
                    count: 1,
                    errored: true,
                });
                findings.push(Finding {
                    check: check_id.clone(),
                    subject: check_id,
                    evidence: format!("the check failed to run: {error:#}"),
                    age_seconds: 0,
                    state: FindingState::Open,
                });
            }
        }
    }
    Sweep {
        id,
        at,
        checks: results,
        findings,
    }
}

// ---------------------------------------------------------------------------------------------------------
// Error review

/// The fewest distinct jobs a cause must occur in before it is a repeat.
pub const REPEAT_JOBS: usize = 2;

/// The longest cause text shown in a title or a request line.
const CAUSE_CAP: usize = 240;

/// The request that addresses a repeated cause, and where it stands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AddressingRequest {
    pub job: String,
    /// The request's state: `queued`, `landed`, and so on (`JobState::name`).
    pub state: String,
    /// True when the request landed, so the cause is no longer open.
    pub landed: bool,
}

/// A cause that occurred in two or more distinct jobs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RepeatedCause {
    /// The grouping key: see [`cause_key`].
    pub key: String,
    /// The failure class's name and id, when the introspections resolve to a registry class.
    pub class: Option<String>,
    /// The cause as the earliest introspection of the group words it, on one line.
    pub cause: String,
    /// The introspections of the group, oldest first.
    pub introspections: Vec<String>,
    /// The distinct jobs they were about, in order of first occurrence.
    pub jobs: Vec<String>,
    /// An open or landed request that cites one of the introspections; none when no request addresses it yet.
    pub request: Option<AddressingRequest>,
}

impl RepeatedCause {
    /// How many introspections record this cause.
    pub fn count(&self) -> usize {
        self.introspections.len()
    }

    /// Whether the cause still needs work: no request addresses it, or the request has not landed.
    pub fn is_open(&self) -> bool {
        !self.request.as_ref().is_some_and(|r| r.landed)
    }
}

/// The failure class an introspection belongs to: the class its record names when the class registry holds
/// it, else the class Jev chose when that is a registry class (`new` and unknown ids are not).
fn class_of(records: &Records, i: &IntrospectionRecord) -> Option<RecordId> {
    let known = |id: &RecordId| records.classes.iter().any(|c| &c.id == id);
    i.failure_class.clone().filter(known).or_else(|| {
        let choice = i.jev.as_ref()?.class.as_ref()?.choice.as_str();
        RecordId::parse(choice).ok().filter(known)
    })
}

/// The one line of text of a cause, cut to [`CAUSE_CAP`] characters.
fn one_line(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= CAUSE_CAP {
        return flat;
    }
    let cut: String = flat.chars().take(CAUSE_CAP).collect();
    format!("{cut}...")
}

/// The key two introspections share when they have the same cause: the failure class they resolve to
/// ([`class_of`], through the class registry or Jev's class answer), and, for one that resolves to none, its
/// cause text with case and punctuation ignored ([`crate::introspect::normalise`]). `None` for an
/// introspection that is not a case: one that failed, has no job, or has no cause.
pub fn cause_key(records: &Records, i: &IntrospectionRecord) -> Option<String> {
    let cause = i.cause.as_deref()?;
    if i.status != IntrospectionStatus::Recorded || i.job.is_none() {
        return None;
    }
    if let Some(class) = class_of(records, i) {
        return Some(format!("class:{class}"));
    }
    let text = crate::introspect::normalise(cause);
    (!text.is_empty()).then(|| format!("cause:{text}"))
}

/// The request that addresses a group of introspections: an open or landed request job whose text cites one of
/// their ids. An open one is preferred to a landed one. A request that was rejected, failed or closed does not
/// address the cause, so the review may queue another.
fn addressing_request(records: &Records, introspections: &[String]) -> Option<AddressingRequest> {
    let mut found: Option<AddressingRequest> = None;
    for job in records.request_jobs() {
        let state = job.state();
        let landed = state == JobState::Closed(OutcomeState::Landed);
        if matches!(state, JobState::Closed(_)) && !landed {
            continue;
        }
        let cites = job
            .request
            .as_ref()
            .is_some_and(|r| introspections.iter().any(|id| r.text.contains(id.as_str())));
        if !cites {
            continue;
        }
        if found.as_ref().is_none_or(|f| f.landed && !landed) {
            found = Some(AddressingRequest {
                job: job.id().as_str().to_string(),
                state: state.name().to_string(),
                landed,
            });
        }
    }
    found
}

/// The causes that occurred in [`REPEAT_JOBS`] or more distinct jobs, with their introspections and the
/// request that addresses each. Ordered by the time of the earliest introspection of each cause.
pub fn repeated_causes(records: &Records) -> Vec<RepeatedCause> {
    let mut groups: Vec<(String, Vec<&IntrospectionRecord>)> = Vec::new();
    for i in &records.introspections {
        let Some(key) = cause_key(records, i) else {
            continue;
        };
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, members)) => members.push(i),
            None => groups.push((key, vec![i])),
        }
    }
    groups
        .into_iter()
        .filter_map(|(key, members)| {
            let mut jobs: Vec<String> = Vec::new();
            for job in members.iter().filter_map(|i| i.job.as_ref()) {
                if !jobs.iter().any(|j| j == job.as_str()) {
                    jobs.push(job.as_str().to_string());
                }
            }
            if jobs.len() < REPEAT_JOBS {
                return None;
            }
            let introspections: Vec<String> = members.iter().map(|i| i.id.to_string()).collect();
            let class = class_of(records, members[0]).map(|id| {
                let name = records
                    .classes
                    .iter()
                    .find(|c| c.id == id)
                    .map_or("unknown class", |c| c.name.as_str());
                format!("{name} ({id})")
            });
            Some(RepeatedCause {
                request: addressing_request(records, &introspections),
                cause: one_line(members[0].cause.as_deref().unwrap_or("")),
                key,
                class,
                introspections,
                jobs,
            })
        })
        .collect()
}

/// The text of the urgent root-cause request for a repeated cause, built with [`StewardRequest`]. No body line
/// starts like a bullet: the builder refuses one, since `athena plan` would read it as acceptance.
pub fn root_cause_request(cause: &RepeatedCause, records: &Records) -> Result<String, String> {
    let mut body = String::from(
        "The same cause has produced failures in more than one job. Find the cause where it originates and fix \
         it there, so the whole class is prevented; do not work around one instance.\n\n",
    );
    if let Some(class) = &cause.class {
        body.push_str(&format!("Failure class: {class}\n"));
    }
    body.push_str(&format!("Cause: {}\n", cause.cause));
    body.push_str(&format!(
        "Seen {} time(s) in {} jobs. The introspections that record it:\n",
        cause.count(),
        cause.jobs.len()
    ));
    for id in &cause.introspections {
        let i = records.introspections.iter().find(|i| i.id.as_str() == id);
        let job = i.and_then(|i| i.job.as_ref()).map_or("?", JobId::as_str);
        let prevention = i
            .and_then(|i| i.prevention.as_deref())
            .map(one_line)
            .unwrap_or_default();
        body.push_str(&format!(
            "Introspection {id} (job {job}); prevention it proposed: {prevention}\n"
        ));
    }
    let acceptance = [
        "The root cause is named in the change description and fixed where it originates, not at one instance"
            .to_string(),
        "A test fails without the change and passes with it, for the cause and not only for one job".to_string(),
        "fmt, clippy and test pass".to_string(),
    ];
    let title: String = cause.cause.chars().take(100).collect();
    StewardRequest::new(
        Some(crate::plan::Priority::Urgent),
        &format!("Fix the root cause of a repeated failure: {title}"),
        &body,
        &acceptance,
        &["crates/".to_string()],
    )
    .map(|r| r.render())
}

impl Athena {
    /// The error review, run after `recorded` was committed as an introspection: groups it with the earlier
    /// introspections by cause and, when the cause now occurs in two or more distinct jobs and no open or
    /// landed request cites it, queues one urgent root-cause request citing the introspection ids. Returns the
    /// job queued, if any. Never fails: a problem is a line to the user, since the review never blocks a job.
    pub fn error_review(&self, recorded: &IntrospectionRecord, ui: &dyn Ui) -> Option<JobId> {
        match self.try_error_review(recorded) {
            Ok(Some((job, cause))) => {
                ui.line(&format!(
                    "hestia     repeated cause in {} jobs ({} introspections); root-cause request {job} queued at urgent priority",
                    cause.jobs.len(),
                    cause.count()
                ));
                Some(job)
            }
            Ok(None) => None,
            Err(error) => {
                ui.line(&format!(
                    "hestia     the error review could not run: {error:#}"
                ));
                None
            }
        }
    }

    fn try_error_review(
        &self,
        recorded: &IntrospectionRecord,
    ) -> Result<Option<(JobId, RepeatedCause)>> {
        let records = Records::load(&self.git, &self.head()?)?;
        let Some(key) = cause_key(&records, recorded) else {
            return Ok(None);
        };
        let Some(cause) = repeated_causes(&records)
            .into_iter()
            .find(|c| c.key == key && c.introspections.contains(&recorded.id.to_string()))
        else {
            return Ok(None);
        };
        if cause.request.is_some() {
            return Ok(None);
        }
        let text = root_cause_request(&cause, &records)
            .map_err(|e| anyhow::anyhow!("the root-cause request: {e}"))?;
        let (job, _) = self.queue(&text)?;
        Ok(Some((job, cause)))
    }

    /// Runs `checks`, commits the report as a record, and returns it.
    pub fn hestia_sweep(&self, checks: &[Box<dyn Check>]) -> Result<Sweep> {
        let known = match self.git.head()? {
            Some(head) => Records::load(&self.git, &head)?,
            None => bail!(
                "the repository has no commits yet; commit athena.toml and defaults.toml first"
            ),
        };
        let now = crate::now();
        let mut id = RecordId::generate(now);
        while known.has_record_id(&id) {
            id = RecordId::generate(now);
        }
        let sweep = run_sweep(checks, id, now);
        sweep.validate().map_err(|e| anyhow::anyhow!(e))?;
        let record = AnyRecord::HestiaSweep(Box::new(sweep.clone()));
        let subject = format!(
            "Hestia sweep: {} check(s), {} finding(s)",
            sweep.checks.len(),
            sweep.findings.len()
        );
        self.commit_records(
            &[NewFile::record(&record)],
            &message(
                &subject,
                "A report of Hestia's checks. It lists every check that ran with its count; a check that failed to run is a finding.",
                &[("Athena-Record", record.canonical_path())],
            ),
        )?;
        Ok(sweep)
    }

    /// The newest recorded sweep, or None when none was recorded.
    pub fn last_sweep(&self) -> Result<Option<Sweep>> {
        let Some(head) = self.git.head()? else {
            return Ok(None);
        };
        let records = Records::load(&self.git, &head)?;
        Ok(Sweep::latest(&records.hestia_sweeps).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateDir;
    use athena_agent::fake::FakeAgentDriver;
    use athena_data::records::Build;
    use athena_git::Git;

    /// A check with a fixed answer.
    struct Fixed {
        id: &'static str,
        answer: std::result::Result<CheckOutput, &'static str>,
    }

    impl Check for Fixed {
        fn id(&self) -> &str {
            self.id
        }
        fn run(&self, _: Timestamp) -> Result<CheckOutput> {
            self.answer.clone().map_err(|e| anyhow::anyhow!(e))
        }
    }

    fn clean(id: &'static str, examined: u32) -> Box<dyn Check> {
        Box::new(Fixed {
            id,
            answer: Ok(CheckOutput {
                examined,
                findings: Vec::new(),
            }),
        })
    }

    fn finding(check: &str, subject: &str) -> Finding {
        Finding {
            check: check.into(),
            subject: subject.into(),
            evidence: "merged into main three weeks ago".into(),
            age_seconds: 1_814_400,
            state: FindingState::Open,
        }
    }

    fn rid(text: &str) -> RecordId {
        RecordId::parse(text).unwrap()
    }

    #[test]
    fn a_sweep_where_every_check_finds_nothing_lists_every_check_with_a_count_of_zero() {
        let checks = vec![clean("alpha", 3), clean("beta", 0), clean("gamma", 12)];
        let sweep = run_sweep(&checks, rid("20261026-090000-5abc"), crate::now());
        assert_eq!(
            sweep
                .checks
                .iter()
                .map(|c| (c.check.as_str(), c.count, c.examined, c.errored))
                .collect::<Vec<_>>(),
            [
                ("alpha", 0, 3, false),
                ("beta", 0, 0, false),
                ("gamma", 0, 12, false)
            ]
        );
        assert!(sweep.findings.is_empty());
        assert!(sweep.validate().is_ok());
    }

    #[test]
    fn a_check_that_errors_becomes_a_finding_and_the_other_checks_still_run() {
        let checks: Vec<Box<dyn Check>> = vec![
            clean("alpha", 3),
            Box::new(Fixed {
                id: "broken",
                answer: Err("cannot read the branch list"),
            }),
            Box::new(Fixed {
                id: "noisy",
                answer: Ok(CheckOutput {
                    examined: 5,
                    // A check cannot file a finding under another check's id.
                    findings: vec![finding("someone-else", "old-branch")],
                }),
            }),
            clean("omega", 1),
        ];
        let sweep = run_sweep(&checks, rid("20261026-090000-5abc"), crate::now());
        assert_eq!(
            sweep
                .checks
                .iter()
                .map(|c| (c.check.as_str(), c.count, c.errored))
                .collect::<Vec<_>>(),
            [
                ("alpha", 0, false),
                ("broken", 1, true),
                ("noisy", 1, false),
                ("omega", 0, false)
            ]
        );
        let broken: Vec<&Finding> = sweep
            .findings
            .iter()
            .filter(|f| f.check == "broken")
            .collect();
        assert_eq!(broken.len(), 1);
        assert_eq!(broken[0].subject, "broken");
        assert!(
            broken[0].evidence.contains("cannot read the branch list"),
            "{}",
            broken[0].evidence
        );
        assert_eq!(broken[0].age_seconds, 0);
        assert_eq!(broken[0].state, FindingState::Open);
        let noisy = sweep.findings.iter().find(|f| f.check == "noisy").unwrap();
        assert_eq!(noisy.subject, "old-branch");
        assert_eq!(sweep.findings.len(), 2);
        assert!(sweep.validate().is_ok());
    }

    #[test]
    fn a_finding_without_evidence_is_reported_as_a_failed_check() {
        let mut bad = finding("x", "branch");
        bad.evidence = " ".into();
        let checks: Vec<Box<dyn Check>> = vec![Box::new(Fixed {
            id: "sloppy",
            answer: Ok(CheckOutput {
                examined: 1,
                findings: vec![bad],
            }),
        })];
        let sweep = run_sweep(&checks, rid("20261026-090000-5abc"), crate::now());
        assert!(sweep.checks[0].errored);
        assert!(sweep.findings[0].evidence.contains("no evidence"));
        assert!(sweep.validate().is_ok());
    }

    #[test]
    fn the_skeleton_registers_no_checks() {
        assert!(default_checks().is_empty());
    }

    fn athena_in(dir: &std::path::Path, state: &std::path::Path) -> Athena {
        let git = Git::at(dir);
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value]).unwrap();
        }
        std::fs::write(
            dir.join("athena.toml"),
            include_str!("../../../athena.toml"),
        )
        .unwrap();
        std::fs::write(
            dir.join("defaults.toml"),
            include_str!("../../../defaults.toml"),
        )
        .unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "Start"]).unwrap();
        Athena {
            git,
            state: StateDir::new(state.join("repo"))
                .with_free_space(crate::test_support::plenty_of_space),
            driver: std::sync::Arc::new(FakeAgentDriver::deterministic()),
            capabilities: vec![],
            build: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            athena_bin: "/bin/false".into(),
            nested: None,
            pinner: Box::new(crate::fake::CountingPinner::new(&"0".repeat(40))),
            jev: None,
        }
    }

    #[test]
    fn a_committed_sweep_is_read_back_and_the_last_one_is_the_newest() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        assert_eq!(athena.last_sweep().unwrap(), None);

        let first = athena.hestia_sweep(&[clean("alpha", 2)]).unwrap();
        assert_eq!(athena.last_sweep().unwrap(), Some(first.clone()));
        let broken: Vec<Box<dyn Check>> = vec![
            clean("alpha", 2),
            Box::new(Fixed {
                id: "broken",
                answer: Err("boom"),
            }),
        ];
        let second = athena.hestia_sweep(&broken).unwrap();
        assert_ne!(first.id, second.id);
        let last = athena.last_sweep().unwrap().unwrap();
        assert_eq!(last, second);
        assert_eq!(last.checks.len(), 2);
        assert!(last.findings[0].evidence.contains("boom"));
        let path = format!("records/hestia-sweeps/{}/{}.json", last.id.month(), last.id);
        assert!(dir.path().join(&path).exists(), "{path}");
    }

    // -----------------------------------------------------------------------------------------------------
    // Error review

    use athena_data::records::{
        ClassStatus, FailureClass, IntrospectionJev, IntrospectionTrigger, JevClassAnswer, Outcome,
        PreventionKind,
    };

    fn at(n: u32) -> Timestamp {
        format!("2026-10-27T09:{:02}:{:02}Z", n / 60, n % 60)
            .parse()
            .unwrap()
    }

    fn rec_id(n: u32) -> RecordId {
        RecordId::parse(&format!("20261027-09{:02}{:02}-1abc", n / 60, n % 60)).unwrap()
    }

    fn job(letter: char) -> JobId {
        JobId::parse(&format!(
            "20261027-0800{:02}-{}bcd",
            letter as u32 - 'a' as u32,
            letter
        ))
        .unwrap()
    }

    fn class(n: u32, name: &str) -> FailureClass {
        FailureClass {
            id: rec_id(n),
            at: at(n),
            name: name.into(),
            description: "a class".into(),
            status: ClassStatus::Proposed,
            proposed_by: rec_id(n + 1),
        }
    }

    /// A recorded introspection `n` about `of`, in class `in_class`, with `cause`.
    fn introspection(
        n: u32,
        of: char,
        in_class: Option<&RecordId>,
        cause: &str,
    ) -> IntrospectionRecord {
        IntrospectionRecord {
            id: rec_id(n),
            at: at(n),
            trigger: IntrospectionTrigger::Rejection,
            job: Some(job(of)),
            intervention: None,
            class_of_work: "code".into(),
            category: "rejected".into(),
            what_happened: Some("the job was rejected".into()),
            failure_class: in_class.cloned(),
            new_class: false,
            earlier_cases: Vec::new(),
            decisions_matched: Vec::new(),
            cause: Some(cause.into()),
            prevention_kind: Some(PreventionKind::CodeCheck),
            prevention: Some("Add a code check.".into()),
            proposed_request: Some("Add the check".into()),
            evidence: None,
            status: IntrospectionStatus::Recorded,
            failure: None,
            flags: Vec::new(),
            dropped_ids: Vec::new(),
            planning_stopped: false,
            from_jev: false,
            jev: None,
        }
    }

    /// Writes the records into the repository and commits them, as the introspection tests do.
    fn put(dir: &std::path::Path, athena: &Athena, records: Vec<AnyRecord>) {
        for record in &records {
            let file = NewFile::record(record);
            let path = dir.join(&file.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, &file.bytes).unwrap();
        }
        athena.git.run(["add", "-A"]).unwrap();
        athena
            .git
            .run(["commit", "-q", "-m", "Record for the test"])
            .unwrap();
    }

    fn put_introspection(dir: &std::path::Path, athena: &Athena, i: &IntrospectionRecord) {
        put(
            dir,
            athena,
            vec![AnyRecord::Introspection(Box::new(i.clone()))],
        );
    }

    fn records(athena: &Athena) -> Records {
        Records::load(&athena.git, &athena.head().unwrap()).unwrap()
    }

    fn landed(job: &JobId) -> AnyRecord {
        AnyRecord::Outcome(Box::new(Outcome {
            job: job.clone(),
            state: OutcomeState::Landed,
            reason: "landed".into(),
            at: at(500),
            totals: Some(crate::scoring::totals(&[], None)),
            branch: None,
            base_commit: None,
            candidate_commit: None,
            after_blocked: false,
            through_commit: None,
        }))
    }

    #[test]
    fn two_introspections_of_one_cause_in_different_jobs_queue_one_urgent_request_and_a_third_queues_none()
     {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let c = class(1, "Reading ignores the earlier verdict");
        put(
            dir.path(),
            &athena,
            vec![AnyRecord::FailureClass(c.clone())],
        );

        let first = introspection(
            10,
            'a',
            Some(&c.id),
            "The reading does not load the verdict.",
        );
        put_introspection(dir.path(), &athena, &first);
        assert_eq!(athena.error_review(&first, &crate::Quiet), None);
        assert!(
            records(&athena).request_jobs().is_empty(),
            "one job is no repeat"
        );

        // The same cause in another job, worded differently: the class is what joins them.
        let second = introspection(20, 'b', Some(&c.id), "The earlier verdict is never read.");
        put_introspection(dir.path(), &athena, &second);
        let queued = athena
            .error_review(&second, &crate::Quiet)
            .expect("the second job makes it a repeat");

        let known = records(&athena);
        assert_eq!(known.request_jobs().len(), 1, "exactly one request");
        let request = known.jobs[queued.as_str()].request.as_ref().unwrap();
        assert!(
            request.text.starts_with("Priority: urgent\n"),
            "{}",
            request.text
        );
        assert_eq!(
            known.jobs[queued.as_str()].priority(),
            crate::plan::Priority::Urgent
        );
        assert!(request.text.contains(first.id.as_str()), "{}", request.text);
        assert!(
            request.text.contains(second.id.as_str()),
            "{}",
            request.text
        );
        assert!(request.text.contains("Acceptance:\n- "), "{}", request.text);
        assert!(request.text.contains("root cause"), "{}", request.text);
        // The shared steward builder's own parse accepts the text it made.
        assert!(!crate::reading::request_text_acceptance(&request.text).is_empty());

        // A third introspection of the cause, in a third job, while that request is open.
        let third = introspection(30, 'c', Some(&c.id), "Nothing loads the earlier verdict.");
        put_introspection(dir.path(), &athena, &third);
        assert_eq!(athena.error_review(&third, &crate::Quiet), None);
        let known = records(&athena);
        assert_eq!(known.request_jobs().len(), 1, "still one request");
        let repeated = repeated_causes(&known);
        assert_eq!(repeated.len(), 1);
        assert_eq!(repeated[0].count(), 3);
        assert_eq!(repeated[0].jobs.len(), 3);
        assert_eq!(repeated[0].request.as_ref().unwrap().job, queued.as_str());

        // Once the request has landed, the cause still counts as addressed.
        put(dir.path(), &athena, vec![landed(&queued)]);
        let fourth = introspection(40, 'd', Some(&c.id), "Still no read of the verdict.");
        put_introspection(dir.path(), &athena, &fourth);
        assert_eq!(athena.error_review(&fourth, &crate::Quiet), None);
        assert_eq!(records(&athena).request_jobs().len(), 1);
    }

    #[test]
    fn a_single_job_repeat_or_a_cited_cause_queues_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let c = class(1, "Planner ignores the cap");
        let d = class(3, "Check output is cut");
        put(
            dir.path(),
            &athena,
            vec![
                AnyRecord::FailureClass(c.clone()),
                AnyRecord::FailureClass(d.clone()),
            ],
        );

        // The same cause twice in one job is one job.
        let one = introspection(10, 'a', Some(&c.id), "The cap is not read.");
        let two = introspection(11, 'a', Some(&c.id), "The cap is not read.");
        put_introspection(dir.path(), &athena, &one);
        put_introspection(dir.path(), &athena, &two);
        assert_eq!(athena.error_review(&two, &crate::Quiet), None);
        assert!(repeated_causes(&records(&athena)).is_empty());
        assert!(records(&athena).request_jobs().is_empty());

        // A request that already cites an introspection of the cause covers the cause.
        let owner = introspection(20, 'x', Some(&d.id), "The output tail is cut.");
        put_introspection(dir.path(), &athena, &owner);
        let (cited, _) = athena
            .queue(&format!(
                "Make the check output whole\n\nSee introspection {}.\n\nAcceptance:\n- The output is whole\n\nFiles: crates/\n",
                owner.id
            ))
            .unwrap();
        let again = introspection(21, 'y', Some(&d.id), "The output tail is cut again.");
        put_introspection(dir.path(), &athena, &again);
        assert_eq!(athena.error_review(&again, &crate::Quiet), None);
        let known = records(&athena);
        assert_eq!(known.request_jobs().len(), 1, "no second request");
        assert_eq!(
            repeated_causes(&known)[0].request.as_ref().unwrap().job,
            cited.as_str()
        );

        // A failed introspection is no case, and neither is one with no job.
        let mut failed = introspection(30, 'p', Some(&c.id), "The cap is not read.");
        failed.status = IntrospectionStatus::Failed;
        let mut no_job = introspection(31, 'q', Some(&c.id), "The cap is not read.");
        no_job.job = None;
        put_introspection(dir.path(), &athena, &failed);
        put_introspection(dir.path(), &athena, &no_job);
        assert_eq!(athena.error_review(&failed, &crate::Quiet), None);
        assert_eq!(athena.error_review(&no_job, &crate::Quiet), None);
        assert_eq!(records(&athena).request_jobs().len(), 1);
    }

    #[test]
    fn a_cause_is_matched_by_the_registry_class_by_jevs_class_answer_or_by_its_text() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        let c = class(1, "Reading ignores the earlier verdict");
        put(
            dir.path(),
            &athena,
            vec![AnyRecord::FailureClass(c.clone())],
        );

        // A recorded introspection always names a class, but it may name one that is not in the registry (an
        // id the registry does not hold). The registry then cannot place it, and Jev's answer does.
        let unlisted = rec_id(900);
        let first = introspection(10, 'a', Some(&c.id), "The verdict is not read.");
        let mut second = introspection(20, 'b', Some(&unlisted), "A verdict nobody loads.");
        second.jev = Some(IntrospectionJev {
            model: "typesafe-ai/jev".into(),
            class: Some(JevClassAnswer {
                choice: c.id.to_string(),
                probabilities: Default::default(),
                confidence: 0.9,
            }),
            repeat_probability: Some(0.5),
            context_prevention: None,
            input_tokens: 0,
            output_tokens: 0,
            cost_usd: None,
            error: None,
        });
        put_introspection(dir.path(), &athena, &first);
        put_introspection(dir.path(), &athena, &second);
        assert!(athena.error_review(&second, &crate::Quiet).is_some());

        // Jev's `new` is no registry class; with no class to place them, the same words (case and punctuation
        // aside) join.
        let mut text_a = introspection(30, 'e', Some(&unlisted), "Nothing runs the docs test!");
        let mut text_b = introspection(31, 'f', Some(&unlisted), "nothing runs the DOCS test");
        text_a.jev = None;
        text_b.jev = Some(IntrospectionJev {
            class: Some(JevClassAnswer {
                choice: "new".into(),
                probabilities: Default::default(),
                confidence: 0.5,
            }),
            ..second.jev.clone().unwrap()
        });
        put_introspection(dir.path(), &athena, &text_a);
        assert_eq!(athena.error_review(&text_a, &crate::Quiet), None);
        put_introspection(dir.path(), &athena, &text_b);
        assert!(athena.error_review(&text_b, &crate::Quiet).is_some());
        assert_eq!(records(&athena).request_jobs().len(), 2);

        // A different cause with no class stays alone.
        let other = introspection(40, 'g', Some(&unlisted), "The brief omits file names.");
        put_introspection(dir.path(), &athena, &other);
        assert_eq!(athena.error_review(&other, &crate::Quiet), None);
    }

    #[test]
    fn athena_status_lists_each_open_repeated_cause_with_its_count_and_its_request() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        assert!(athena.status().unwrap().repeated_causes.is_empty());

        let addressed = class(1, "Reading ignores the earlier verdict");
        let waiting = class(3, "Check output is cut");
        let done = class(5, "Planner ignores the cap");
        put(
            dir.path(),
            &athena,
            vec![
                AnyRecord::FailureClass(addressed.clone()),
                AnyRecord::FailureClass(waiting.clone()),
                AnyRecord::FailureClass(done.clone()),
            ],
        );
        // `addressed`: three introspections in three jobs, with a request queued by the review.
        let mut queued = None;
        for (n, of) in [(10, 'a'), (11, 'b'), (12, 'c')] {
            let i = introspection(n, of, Some(&addressed.id), "The verdict is not read.");
            put_introspection(dir.path(), &athena, &i);
            if let Some(job) = athena.error_review(&i, &crate::Quiet) {
                queued = Some(job);
            }
        }
        let queued = queued.expect("the second introspection queued the request");
        // `waiting`: two jobs, but no review ran, so no request addresses it.
        for (n, of) in [(20, 'd'), (21, 'e')] {
            let i = introspection(n, of, Some(&waiting.id), "The tail is cut.");
            put_introspection(dir.path(), &athena, &i);
        }
        // `done`: two jobs and a request that has landed, so it is no longer open.
        for (n, of) in [(30, 'f'), (31, 'g')] {
            let i = introspection(n, of, Some(&done.id), "The cap is not read.");
            put_introspection(dir.path(), &athena, &i);
            athena.error_review(&i, &crate::Quiet);
        }
        let done_job = records(&athena)
            .request_jobs()
            .into_iter()
            .find(|j| {
                j.request
                    .as_ref()
                    .is_some_and(|r| r.text.contains(rec_id(30).as_str()))
            })
            .map(|j| j.id().clone())
            .expect("the review queued a request for the cause");
        put(dir.path(), &athena, vec![landed(&done_job)]);

        let view = athena.status().unwrap();
        let shown: Vec<(usize, usize, Option<String>)> = view
            .repeated_causes
            .iter()
            .map(|c| {
                (
                    c.count(),
                    c.jobs.len(),
                    c.request.as_ref().map(|r| r.job.clone()),
                )
            })
            .collect();
        assert_eq!(
            shown,
            vec![(3, 3, Some(queued.as_str().to_string())), (2, 2, None)],
            "the landed cause is not listed"
        );
        assert_eq!(
            view.repeated_causes[0].request.as_ref().unwrap().state,
            "queued"
        );
        assert!(
            view.next
                .iter()
                .any(|n| n.contains("no root-cause request") && n.contains("The tail is cut.")),
            "{:?}",
            view.next
        );
    }

    #[test]
    fn the_root_cause_request_is_urgent_and_refuses_a_cause_that_reads_as_acceptance() {
        let cause = |text: &str| RepeatedCause {
            key: "cause:x".into(),
            class: None,
            cause: one_line(text),
            introspections: vec![rec_id(10).to_string(), rec_id(20).to_string()],
            jobs: vec![job('a').as_str().to_string(), job('b').as_str().to_string()],
            request: None,
        };
        // A cause with line breaks and a leading dash is flattened to one line and prefixed, so the builder
        // does not read it as an acceptance bullet.
        let text = root_cause_request(&cause("- first\n- second"), &Records::default()).unwrap();
        assert_eq!(
            crate::reading::request_text_acceptance(&text).len(),
            3,
            "{text}"
        );
        assert!(text.contains("Cause: - first - second\n"), "{text}");
    }
}
