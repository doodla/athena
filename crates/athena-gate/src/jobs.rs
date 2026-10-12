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

use crate::invariant::{self, HistoryState};
use crate::{Change, Judged, Result, Rule, Violation};
use athena_data::records::{
    AnyRecord, Candidate, Correction, Intervention, Invocation, Job, Outcome, OutcomeState,
    Reading, Score, Verification,
};
use athena_data::{JobId, RecordId};
use athena_git::Git;
use athena_score::JobFacts;
use std::collections::BTreeSet;

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

/// The score that `job`'s records in `treeish` imply, made at `at`; a re-score when `cause` is set.
pub fn expected_score(
    git: &Git,
    treeish: &str,
    job: &JobId,
    cause: Option<RecordId>,
    at: jiff::Timestamp,
) -> Result<Score> {
    let files = JobFiles::load(git, treeish, job)?;
    let interventions = interventions_for(git, treeish, job)?;
    let interventions: Vec<&Intervention> = interventions.iter().collect();
    let invocations = files.invocation_records();
    let closing = athena_score::score_records(
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

/// Checks the job records `change` adds against the rest of their job (M27).
pub fn check(
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
                if score.rules != athena_score::RULES {
                    violations.push(fail(
                        &d.path,
                        format!(
                            "{} was scored under rules {:?}, which this Athena cannot recompute",
                            d.path, score.rules
                        ),
                        "scores are written by athena, under the rules it recomputes them with",
                    ));
                    continue;
                }
                let expected =
                    expected_score(git, &change.tree, &score.job, score.cause.clone(), score.at)?;
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
        check(git, &change, &judged, None).unwrap()
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
