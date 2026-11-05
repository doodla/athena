//! A job's records judged as a set (design §5.4; M27).
//!
//! Records are append-only, but appending is not enough: a hand-written record added next to a job's own
//! could rewrite what the job's records say. So, for every record a change adds:
//!
//! - a score must equal the score that the job's records in the change's tree imply, recomputed with the
//!   scorer Athena writes them with ([`athena_score::score_records`]), under the same rules;
//! - a re-score must name an intervention that the same change adds and that names the same job;
//! - a landed outcome (`outcome.json` or `outcome-landed.json`) comes only with a valid landing commit of
//!   its own job (the governed-content invariant's case a); a squash that lands a chain of jobs is a valid
//!   landing of each of them;
//! - an outcome after `blocked` (`outcome-landed.json`, `outcome-abandoned.json`) needs the job's
//!   `blocked` outcome on the parent commit, or in the same commit when a squash folds the job into one.
//!
//! A commit being made is also held to provenance ([`provenance`], design §16.2): a job it closes must
//! carry the request, reader invocation, dispatch fields, brief and ordering of times that Athena's own
//! writers leave. Records are unsigned, so this refuses records written by hand that lack or contradict
//! that evidence; it does not stop a forger who writes all of it consistently.

use crate::invariant::{self, HistoryState};
use crate::{Change, Judged, Result, Rule, Violation};
use athena_data::records::{
    AnyRecord, Candidate, Correction, Intervention, Invocation, Job, Outcome, OutcomeState,
    Reading, Role, Score, Verification,
};
use athena_data::{JobId, RecordId};
use athena_git::Git;
use athena_score::{JobFacts, RulesVersion};
use std::collections::{BTreeMap, BTreeSet};

/// The records of one job in one tree. Files that do not parse, or that sit away from their canonical
/// path, are left out: the content checks report them.
#[derive(Debug, Default, Clone)]
pub struct JobFiles {
    pub readings: Vec<Reading>,
    pub corrections: Vec<Correction>,
    pub job: Option<Job>,
    /// Every invocation record, with its path, in order of `seq`.
    pub invocations: Vec<(String, Invocation)>,
    pub candidate: Option<Candidate>,
    pub verification: Option<Verification>,
    /// `outcome.json`.
    pub outcome: Option<Outcome>,
    /// The outcome after `blocked`: `outcome-landed.json` or `outcome-abandoned.json`.
    pub later_outcome: Option<Outcome>,
}

impl JobFiles {
    /// Loads the records of `job` in `treeish`.
    pub fn load(git: &Git, treeish: &str, job: &JobId) -> Result<JobFiles> {
        let entries: Vec<_> = git
            .ls_tree_under(treeish, &job.dir())?
            .into_iter()
            .filter(|e| e.kind == "blob" && e.path.ends_with(".json"))
            .collect();
        let oids: Vec<String> = entries.iter().map(|e| e.oid.clone()).collect();
        let blobs = git.cat_blobs(&oids)?;
        let mut files = JobFiles::default();
        for (entry, bytes) in entries.into_iter().zip(blobs) {
            let Ok(record) = AnyRecord::parse(&entry.path, &bytes) else {
                continue;
            };
            if record.canonical_path() != entry.path {
                continue;
            }
            match record {
                AnyRecord::Reading(r) => files.readings.push(*r),
                AnyRecord::Correction(c) => files.corrections.push(c),
                AnyRecord::Job(j) => files.job = Some(*j),
                AnyRecord::Invocation(i) => files.invocations.push((entry.path, *i)),
                AnyRecord::Candidate(c) | AnyRecord::Candidate2(c) => files.candidate = Some(*c),
                AnyRecord::Verification(v) => files.verification = Some(*v),
                AnyRecord::Outcome(o) if o.after_blocked => files.later_outcome = Some(*o),
                AnyRecord::Outcome(o) => files.outcome = Some(*o),
                _ => {}
            }
        }
        files.readings.sort_by_key(|r| r.n);
        files.corrections.sort_by_key(|c| c.n);
        files.invocations.sort_by_key(|(_, i)| i.seq);
        Ok(files)
    }

    /// The invocation records, in order of `seq`.
    pub fn invocation_records(&self) -> Vec<Invocation> {
        self.invocations.iter().map(|(_, i)| i.clone()).collect()
    }
}

/// The interventions in `treeish` that name `job`.
pub fn interventions_for(git: &Git, treeish: &str, job: &JobId) -> Result<Vec<Intervention>> {
    let entries: Vec<_> = git
        .ls_tree_under(treeish, "records/interventions")?
        .into_iter()
        .filter(|e| e.kind == "blob")
        .collect();
    let oids: Vec<String> = entries.iter().map(|e| e.oid.clone()).collect();
    let blobs = git.cat_blobs(&oids)?;
    Ok(entries
        .iter()
        .zip(blobs)
        .filter_map(
            |(entry, bytes)| match AnyRecord::parse(&entry.path, &bytes) {
                Ok(AnyRecord::Intervention(i)) if i.job.as_ref() == Some(job) => Some(i),
                _ => None,
            },
        )
        .collect())
}

/// The score that `job`'s records in `treeish` imply under `rules`, made at `at`; a re-score when `cause`
/// is set.
pub fn expected_score(
    git: &Git,
    treeish: &str,
    job: &JobId,
    cause: Option<RecordId>,
    at: jiff::Timestamp,
    rules: RulesVersion,
) -> Result<Score> {
    let files = JobFiles::load(git, treeish, job)?;
    // Only the interventions that existed when the score was made count. A squash folds a job's closing
    // score and its later interventions into one tree. The closing score was made before those
    // interventions, with `user_intervenes` still unresolved (false and provisional); recomputing it from
    // the whole tree would count them and resolve that event to true, so the recorded closing score, and
    // each re-score derived from it, would differ in `events[user_intervenes].outcome`, `.provisional` and
    // `.brier`. A re-score is the closing score as of its own `at` with that event resolved true by
    // `rescore`, so a forged re-score (for example zeroed Brier values) still differs.
    let interventions = interventions_for(git, treeish, job)?;
    let interventions: Vec<&Intervention> = interventions.iter().filter(|i| i.at <= at).collect();
    let invocations = files.invocation_records();
    let closing = athena_score::score_records(
        rules,
        &JobFacts {
            job,
            readings: &files.readings,
            corrections: &files.corrections,
            dispatched: files.job.as_ref(),
            invocations: &invocations,
            candidate: files.candidate.as_ref(),
            verification: files.verification.as_ref(),
            wall_time_execution_s: files
                .outcome
                .as_ref()
                .and_then(|o| o.totals.as_ref())
                .and_then(|t| t.wall_time_execution.as_ref())
                .and_then(|q| q.value()),
            interventions: &interventions,
        },
        None,
        at,
    );
    Ok(match cause {
        Some(cause) => athena_score::rescore(&closing, cause, at),
        None => closing,
    })
}

/// The first place where two scores differ, comparing numbers to within a relative 1e-9.
pub fn score_difference(recorded: &Score, expected: &Score) -> Option<String> {
    let recorded = serde_json::to_value(recorded).ok()?;
    let expected = serde_json::to_value(expected).ok()?;
    first_difference("", &recorded, &expected)
}

fn first_difference(
    path: &str,
    recorded: &serde_json::Value,
    expected: &serde_json::Value,
) -> Option<String> {
    use serde_json::Value;
    let describe = |r: &Value, e: &Value| {
        let field = if path.is_empty() { "the score" } else { path };
        Some(format!("{field} is {r}, but the records give {e}"))
    };
    match (recorded, expected) {
        (Value::Number(r), Value::Number(e)) => {
            let (r_f, e_f) = (r.as_f64()?, e.as_f64()?);
            let scale = r_f.abs().max(e_f.abs()).max(1.0);
            if (r_f - e_f).abs() <= 1e-9 * scale {
                None
            } else {
                describe(recorded, expected)
            }
        }
        (Value::Object(r), Value::Object(e)) => {
            for key in r.keys().chain(e.keys()) {
                let child = format!("{path}.{key}");
                match (r.get(key), e.get(key)) {
                    (Some(rv), Some(ev)) => {
                        if let Some(found) = first_difference(&child, rv, ev) {
                            return Some(found);
                        }
                    }
                    (rv, ev) => {
                        return Some(format!(
                            "{child} is {}, but the records give {}",
                            rv.map_or("absent".into(), Value::to_string),
                            ev.map_or("absent".into(), Value::to_string)
                        ));
                    }
                }
            }
            None
        }
        (Value::Array(r), Value::Array(e)) => {
            if r.len() != e.len() {
                return Some(format!(
                    "{path} has {} entries, but the records give {}",
                    r.len(),
                    e.len()
                ));
            }
            r.iter()
                .zip(e)
                .enumerate()
                .find_map(|(i, (rv, ev))| first_difference(&format!("{path}[{i}]"), rv, ev))
        }
        (r, e) if r == e => None,
        _ => describe(recorded, expected),
    }
}

/// Checks the job records `change` adds against the rest of their job (M27), and, for a commit being made,
/// the cross-links that Athena's own writers always leave ([`provenance`]).
///
/// `history` is `None` for a commit being made (the commit-msg hook, Athena's plumbing commits and
/// `athena close`: `gate` in lib.rs) and `Some` for the frozen walk over commits already made
/// ([`crate::history`]). Only a commit being made is held to provenance: records that older builds wrote
/// carry less of it, and a commit already made cannot be excused afterwards. A future caller that passes
/// `None` for commits already made gets provenance too.
pub fn check(
    git: &Git,
    change: &Change,
    judged: &Judged,
    history: Option<&HistoryState>,
) -> Result<Vec<Violation>> {
    let mut violations = check_records(git, change, judged, history)?;
    if history.is_none() {
        violations.extend(provenance(git, change, judged, history)?);
    }
    Ok(violations)
}

/// The start of every [`provenance`] detail, so that a reader of a gate report sees which kind of refusal
/// it is.
pub const PROVENANCE: &str = "provenance: ";

/// The dispatch fields a landed job's `job.json` must hold: those the provenance rules compare, and no more.
fn missing_dispatch_fields(job: &Job) -> Vec<&'static str> {
    let mut missing = Vec::new();
    let mut need = |present: bool, name: &'static str| {
        if !present {
            missing.push(name);
        }
    };
    need(job.accepted_reading.is_some(), "accepted_reading");
    need(job.accepted_by.is_some(), "accepted_by");
    need(job.caps.is_some(), "caps");
    need(job.base_commit.is_some(), "base_commit");
    need(job.config_sha256.is_some(), "config_sha256");
    need(job.defaults_sha256.is_some(), "defaults_sha256");
    need(job.athena.is_some(), "athena");
    need(job.driver.is_some(), "driver");
    need(job.brief_sha256.is_some(), "brief_sha256");
    need(job.dispatched_at.is_some(), "dispatched_at");
    missing
}

/// Whether `text` can name a commit safely on a git command line: 40 or 64 hex digits.
fn is_full_id(text: &str) -> bool {
    matches!(text.len(), 40 | 64) && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Judges whether the records a commit closes a job with carry the evidence Athena's own writers always
/// leave (design §16.2). Records are not signed, so this cannot tell a careful forgery from the real thing;
/// it refuses the hand-written job whose links are missing, copied or out of order.
///
/// It applies to a request job that this commit closes: a closing score (not a re-score) or any outcome is
/// added to its directory. Each reading of the job needs (R1):
///
/// - `request.json`, received at or before the reading;
/// - the reader invocation the reading names (`invocation_seq`), ended at or before the reading;
/// - when the reading's base commit is held, a `defaults.toml` there that hashes to its `defaults_sha256`.
///
/// A job whose outcome is `landed` also needs (R2) a `job.json` with the dispatch fields, whose config and
/// defaults hashes equal the accepted reading's, and a `brief-producer.md` whose hash is `brief_sha256`;
/// and (R4, [`dispatch_order_problem`]) records that happened in the order of a real run.
///
/// Reading-before-work and the presence of a successful verifier invocation are judged by
/// [`invariant::prediction_problem`] and the landing check in `invariant`, which run in the same gate scope;
/// they are not repeated here.
///
/// Like [`check`], it judges a commit being made. A history walk does not call it.
pub fn provenance(
    git: &Git,
    change: &Change,
    judged: &Judged,
    history: Option<&HistoryState>,
) -> Result<Vec<Violation>> {
    if invariant::is_exempt(change, judged, history) {
        return Ok(Vec::new());
    }
    let commit = change.commit.as_deref();
    let fail = |path: &str, detail: String, repair: &str| {
        Violation::new(Rule::JobRecords, format!("{PROVENANCE}{detail}"), repair)
            .at_path(path)
            .in_commit(commit)
    };
    // The jobs this commit closes, each with the landed outcome record's path and value when it adds one.
    let mut closed: BTreeMap<JobId, Option<(String, Outcome)>> = BTreeMap::new();
    for d in judged
        .diff
        .iter()
        .filter(|d| d.status == 'A' && d.path.starts_with("records/jobs/"))
    {
        // A merge brings in records its other parents already have; those were judged where they were
        // added.
        let mut from_another_parent = false;
        for parent in change.parents.iter().skip(1) {
            if git.show_file(parent, &d.path)?.is_some() {
                from_another_parent = true;
            }
        }
        if from_another_parent {
            continue;
        }
        let bytes = git.cat_blob(&d.new_oid)?;
        let Ok(record) = AnyRecord::parse(&d.path, &bytes) else {
            continue;
        };
        if record.canonical_path() != d.path {
            continue;
        }
        match record {
            AnyRecord::Score(score) if score.cause.is_none() && score.job.is_request() => {
                closed.entry(score.job.clone()).or_insert(None);
            }
            AnyRecord::Outcome(outcome) if outcome.job.is_request() => {
                let landed = (outcome.state == OutcomeState::Landed)
                    .then(|| (d.path.clone(), Outcome::clone(&outcome)));
                let slot = closed.entry(outcome.job.clone()).or_insert(None);
                if landed.is_some() {
                    *slot = landed;
                }
            }
            _ => {}
        }
    }

    let mut violations = Vec::new();
    for (job, landed) in &closed {
        let dir = job.dir();
        let files = JobFiles::load(git, &change.tree, job)?;
        let request = git
            .show_file(&change.tree, &format!("{dir}/request.json"))?
            .and_then(|bytes| AnyRecord::parse(&format!("{dir}/request.json"), &bytes).ok())
            .and_then(|record| match record {
                AnyRecord::Request(r) => Some(r),
                _ => None,
            });
        for reading in &files.readings {
            let n = reading.n;
            let path = format!("{dir}/reading-{n}.json");
            match &request {
                None => violations.push(fail(
                    &path,
                    format!(
                        "{path} has no request record: {dir}/request.json is not in the tree, so nothing shows the request it answers was received"
                    ),
                    "a proposal is made by `athena ask`, which commits the request with its reading; a hand-written proposal cannot be closed here",
                )),
                Some(request) if request.received_at > reading.at => violations.push(fail(
                    &path,
                    format!(
                        "{path} was recorded at {}, before its request was received at {} ({dir}/request.json)",
                        reading.at, request.received_at
                    ),
                    "a reading follows its request; `athena ask` writes them in that order",
                )),
                Some(_) => {}
            }
            match files
                .invocations
                .iter()
                .find(|(_, i)| i.seq == reading.invocation_seq)
            {
                Some((inv_path, inv)) if inv.role == Role::Reader => {
                    if inv.ended_at > reading.at {
                        violations.push(fail(
                            &path,
                            format!(
                                "{path} was recorded at {}, before the reader invocation it names ended at {} ({inv_path})",
                                reading.at, inv.ended_at
                            ),
                            "a reading is made from the reader's output; `athena ask` records the invocation first",
                        ));
                    }
                }
                other => violations.push(fail(
                    &path,
                    match other {
                        Some((inv_path, inv)) => format!(
                            "{path} names invocation {} as its reader, but there is no reader invocation with that number: {inv_path} is a {:?} invocation",
                            reading.invocation_seq, inv.role
                        ),
                        None => format!(
                            "{path} names invocation {} as its reader, but there is no reader invocation with that number in {dir}",
                            reading.invocation_seq
                        ),
                    },
                    "the reader's invocation record is committed with its reading by `athena ask`; run `athena ask` for a new proposal",
                )),
            }
            if is_full_id(&reading.base_commit) && git.commit_exists(&reading.base_commit)? {
                let actual = git
                    .show_file(&reading.base_commit, crate::DEFAULTS_PATH)?
                    .map(|bytes| athena_data::sha256_hex(&bytes));
                if actual.as_deref() != Some(reading.defaults_sha256.as_str()) {
                    violations.push(fail(
                        &path,
                        format!(
                            "{path} records defaults_sha256 {}, which does not match {} at its base commit {} ({})",
                            reading.defaults_sha256,
                            crate::DEFAULTS_PATH,
                            crate::short(&reading.base_commit),
                            actual.as_deref().unwrap_or("the file is absent there")
                        ),
                        "a reading records the hash of the defaults it was made under; `athena ask` computes it from the base commit",
                    ));
                }
            }
        }

        let Some((outcome_path, outcome)) = landed else {
            continue;
        };
        let job_path = format!("{dir}/job.json");
        let Some(dispatched) = &files.job else {
            violations.push(fail(
                outcome_path,
                format!(
                    "{outcome_path} says the job landed, but the job has no job.json ({job_path} is not in the tree), so no dispatch is recorded"
                ),
                "a job is dispatched by `athena go`, which commits job.json before any work starts",
            ));
            continue;
        };
        let missing = missing_dispatch_fields(dispatched);
        if !missing.is_empty() {
            violations.push(fail(
                &job_path,
                format!(
                    "{job_path} lacks {}, which `athena go` records when it dispatches",
                    missing.join(", ")
                ),
                "a job is dispatched by `athena go`; a job.json written by hand or by an older build cannot land",
            ));
        }
        if let Some(accepted) = dispatched
            .accepted_reading
            .and_then(|n| files.readings.iter().find(|r| r.n == n))
        {
            for (name, recorded, read) in [
                (
                    "config_sha256",
                    &dispatched.config_sha256,
                    &accepted.config_sha256,
                ),
                (
                    "defaults_sha256",
                    &dispatched.defaults_sha256,
                    &accepted.defaults_sha256,
                ),
            ] {
                if recorded.as_deref().is_some_and(|r| r != read) {
                    violations.push(fail(
                        &job_path,
                        format!(
                            "{job_path} {name} {} differs from the accepted reading's {read} ({dir}/reading-{}.json)",
                            recorded.as_deref().unwrap_or_default(),
                            accepted.n
                        ),
                        "`athena go` copies these hashes from the accepted reading; do not edit job.json",
                    ));
                }
            }
        }
        if let Some(expected) = &dispatched.brief_sha256 {
            let brief_path = format!("{dir}/brief-producer.md");
            match git.show_file(&change.tree, &brief_path)? {
                None => violations.push(fail(
                    &job_path,
                    format!(
                        "{job_path} records brief_sha256 {expected}, but {brief_path} is not in the tree"
                    ),
                    "`athena go` commits the producer brief with job.json; run `athena go` for this request",
                )),
                Some(bytes) => {
                    let actual = athena_data::sha256_hex(&bytes);
                    if &actual != expected {
                        violations.push(fail(
                            &brief_path,
                            format!(
                                "{brief_path} hashes to {actual}, but {job_path} records brief_sha256 {expected}"
                            ),
                            "`athena go` writes the brief and its hash together; run `athena go` for this request",
                        ));
                    }
                }
            }
        }
        for problem in
            dispatch_order_problem(&files, dispatched.dispatched_at, outcome_path, outcome)
        {
            violations.push(fail(
                outcome_path,
                problem,
                "`athena go` records each step as it happens, in order; a landed job whose records say otherwise was not run by Athena",
            ));
        }
    }
    Ok(violations)
}

/// Why the times in a landed job's records are not those of a real run, one reason per fault.
///
/// Every invocation other than the reader's starts at or after `dispatched_at`; the verifier named by the
/// verification starts at or after the end of the latest producer invocation with a lower number; and the
/// landed outcome is recorded at or after the verifier ended. All comparisons are one way, so equal times
/// pass. This is the complement of [`invariant::prediction_problem`] (the reading comes before the
/// dispatch and before every non-reader invocation) and of the landing check in `invariant` (the
/// verification names a successful verifier invocation): when the verification names no verifier
/// invocation that the records hold, the verifier orderings are skipped and that check refuses it.
pub fn dispatch_order_problem(
    files: &JobFiles,
    dispatched_at: Option<jiff::Timestamp>,
    outcome_path: &str,
    outcome: &Outcome,
) -> Vec<String> {
    let mut problems = Vec::new();
    if let Some(dispatched_at) = dispatched_at {
        for (path, inv) in files
            .invocations
            .iter()
            .filter(|(_, i)| i.role != Role::Reader)
        {
            if inv.started_at < dispatched_at {
                problems.push(format!(
                    "{path} started at {}, before the job was dispatched at {dispatched_at}",
                    inv.started_at
                ));
            }
        }
    }
    let verifier = files
        .verification
        .as_ref()
        .and_then(|v| v.verifier_invocation)
        .and_then(|seq| files.invocations.iter().find(|(_, i)| i.seq == seq))
        .filter(|(_, i)| i.role == Role::Verifier);
    let Some((verifier_path, verifier)) = verifier else {
        return problems;
    };
    let producer = files
        .invocations
        .iter()
        .filter(|(_, i)| i.role == Role::Producer && i.seq < verifier.seq)
        .max_by_key(|(_, i)| i.seq);
    if let Some((producer_path, producer)) = producer
        && verifier.started_at < producer.ended_at
    {
        problems.push(format!(
            "{verifier_path} started at {}, before the producer invocation ended at {} ({producer_path})",
            verifier.started_at, producer.ended_at
        ));
    }
    if outcome.at < verifier.ended_at {
        problems.push(format!(
            "{outcome_path} was recorded at {}, before the verifier invocation ended at {} ({verifier_path})",
            outcome.at, verifier.ended_at
        ));
    }
    problems
}

/// The M27 checks of [`check`] on their own: the score recompute, the re-score and outcome rules. A history
/// walk calls this and not [`provenance`].
pub fn check_records(
    git: &Git,
    change: &Change,
    judged: &Judged,
    history: Option<&HistoryState>,
) -> Result<Vec<Violation>> {
    // An exempt commit is excused here too: its records arrived with the content the exemption excuses,
    // and a landed outcome cannot be a valid landing of a commit the invariant does not judge.
    if invariant::is_exempt(change, judged, history) {
        return Ok(Vec::new());
    }
    let commit = change.commit.as_deref();
    let mut violations = Vec::new();
    let fail = |path: &str, detail: String, repair: &str| {
        Violation::new(Rule::JobRecords, detail, repair)
            .at_path(path)
            .in_commit(commit)
    };
    let added = |path: &str| {
        judged
            .diff
            .iter()
            .any(|d| d.status == 'A' && d.path == path)
    };
    // The jobs this commit validly lands: one, or every job of a chain.
    let mut landed_jobs: Option<BTreeSet<JobId>> = None;
    for d in judged
        .diff
        .iter()
        .filter(|d| d.status == 'A' && d.path.starts_with("records/jobs/"))
    {
        // A merge brings in records its other parents already have; those were judged where they were
        // added.
        let mut from_another_parent = false;
        for parent in change.parents.iter().skip(1) {
            if git.show_file(parent, &d.path)?.is_some() {
                from_another_parent = true;
            }
        }
        if from_another_parent {
            continue;
        }
        let bytes = git.cat_blob(&d.new_oid)?;
        let Ok(record) = AnyRecord::parse(&d.path, &bytes) else {
            continue;
        };
        if record.canonical_path() != d.path {
            continue;
        }
        match record {
            AnyRecord::Score(score) => {
                if let Some(cause) = &score.cause {
                    let path = format!("records/interventions/{}/{cause}.json", cause.month());
                    let names_job = added(&path)
                        && interventions_for(git, &change.tree, &score.job)?
                            .iter()
                            .any(|i| &i.id == cause);
                    if !names_job {
                        violations.push(fail(
                            &d.path,
                            format!(
                                "{} is a re-score caused by intervention {cause}, which this commit does not add for job {}",
                                d.path,
                                score.job.as_str()
                            ),
                            "re-scores are written by `athena intervene --job`, in the commit that adds the intervention",
                        ));
                        continue;
                    }
                }
                let Some(rules) = RulesVersion::from_recorded(&score.rules) else {
                    violations.push(fail(
                        &d.path,
                        format!(
                            "{} was scored under rules {:?}, which this Athena cannot recompute",
                            d.path, score.rules
                        ),
                        "scores are written by athena, under a rules version it knows and recomputes them with",
                    ));
                    continue;
                };
                let expected = expected_score(
                    git,
                    &change.tree,
                    &score.job,
                    score.cause.clone(),
                    score.at,
                    rules,
                )?;
                if let Some(difference) = score_difference(&score, &expected) {
                    violations.push(fail(
                        &d.path,
                        format!(
                            "{} does not follow from job {}'s records: {difference}",
                            d.path,
                            score.job.as_str()
                        ),
                        "scores are written by athena from the job's records; a hand-written score is refused",
                    ));
                }
            }
            AnyRecord::Outcome(outcome) if outcome.job.is_request() => {
                let dir = outcome.job.dir();
                if outcome.after_blocked {
                    // The blocked outcome is on the parent, or arrives in this very commit when a squash
                    // or rebase merge folds the whole job into one.
                    let is_blocked = |treeish: &str| -> Result<bool> {
                        Ok(JobFiles::load(git, treeish, &outcome.job)?
                            .outcome
                            .is_some_and(|o| o.state == OutcomeState::Blocked))
                    };
                    let blocked = is_blocked(&judged.base)? || is_blocked(&change.tree)?;
                    if !blocked
                        || !matches!(
                            outcome.state,
                            OutcomeState::Landed | OutcomeState::Abandoned
                        )
                    {
                        violations.push(fail(
                            &d.path,
                            format!(
                                "{} follows a blocked outcome, but {dir}/outcome.json on the parent commit is not blocked",
                                d.path
                            ),
                            "only a blocked job lands later (`athena go`) or is abandoned (`athena close`)",
                        ));
                        continue;
                    }
                }
                if outcome.state == OutcomeState::Landed {
                    if landed_jobs.is_none() {
                        landed_jobs = Some(invariant::landing_job(git, change, judged, history)?);
                    }
                    if !landed_jobs
                        .as_ref()
                        .is_some_and(|jobs| jobs.contains(&outcome.job))
                    {
                        violations.push(fail(
                            &d.path,
                            format!(
                                "{} says job {} landed, but this commit is not a valid landing of that job",
                                d.path,
                                outcome.job.as_str()
                            ),
                            "a job lands only through athena, in a commit that carries the verified change",
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::landing_tests::{chain_repo, land_chain};

    const STEP_1: &str = "pub fn a() { /* one */ }\n";
    const STEP_2: &str = "pub fn a() { /* two */ }\n";

    fn check_head(git: &Git) -> Vec<Violation> {
        let id = git.rev_parse("HEAD").unwrap().unwrap();
        let change = Change::of_commit(git, &id).unwrap();
        let judged = Judged::new(git, &change).unwrap();
        check_records(git, &change, &judged, None).unwrap()
    }

    #[test]
    fn every_landed_outcome_of_a_chain_squash_is_accepted() {
        let (dir, git, links) = chain_repo(&[STEP_1, STEP_2]);
        land_chain(dir.path(), &git, &links, STEP_2, &|_, _| {});
        assert_eq!(check_head(&git), vec![]);
    }

    #[test]
    fn the_landed_outcomes_of_a_broken_chain_are_refused() {
        let (dir, git, mut links) = chain_repo(&[STEP_1, STEP_2]);
        links[1].base_fp = "e".repeat(64);
        land_chain(dir.path(), &git, &links, STEP_2, &|_, _| {});
        let found = check_head(&git);
        assert_eq!(found.len(), 2, "one per landed outcome: {found:?}");
        assert!(
            found
                .iter()
                .all(|v| v.detail.contains("not a valid landing of that job"))
        );
    }
}
