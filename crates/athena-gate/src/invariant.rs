//! The governed-content invariant (design §7.3, D39) and the prediction-trailer check (§7.4 check 7, D07).
//!
//! Let X be a commit and R the rules of its first parent. If fingerprint(X; R) equals the first parent's
//! fingerprint, X touched only records or owner paths; it must name its work with an `Athena-Job:`,
//! `Athena-Intervention:`, or `Athena-Record:` trailer, unless it touched only owner paths. If the
//! fingerprint changed, X must be exactly one of:
//!
//! - (a) a landing commit of a job that Athena read, dispatched, and verified (D55): it carries
//!   `Athena-Job`, `Athena-Prediction`, and an `Athena-Verified-Content` equal to its fingerprint; the
//!   job's `job.json` is on the parent commit and the prediction commit, an ancestor, added the accepted
//!   reading; X adds the job's candidate, verification, and landed outcome (or, after `blocked`, its
//!   `outcome-landed.json`); the candidate and the verification both hold X's fingerprint, the final
//!   verdict is pass, the verifier invocation the verification names is recorded, succeeded, used the
//!   job's verifier model, and reconciles, as do the outcome's totals; and no earlier commit landed that
//!   job;
//! - (b) a declared intervention: `Athena-Intervention: <id>`, and X adds that intervention record;
//! - (c) a bootstrap commit: `Athena-Job: bootstrap`, while the bootstrap range is open, or at or before
//!   the commit that `records/jobs/bootstrap/outcome.json` names.
//!
//! A merge passes when its fingerprint equals one parent's; otherwise it needs (a) or (b).

use crate::jobs::JobFiles;
use crate::{Change, Judged, Result, Rule, Violation, reconcile, short, trailer_values};
use athena_data::records::{AnyRecord, Invocation, InvocationOutcome, OutcomeState, Role, Verdict};
use athena_data::{JobId, RecordId};
use athena_git::Git;
use athena_git::attrs::fingerprint;
use std::collections::BTreeSet;

pub const JOB: &str = "Athena-Job";
pub const INTERVENTION: &str = "Athena-Intervention";
pub const RECORD: &str = "Athena-Record";
pub const VERIFIED_CONTENT: &str = "Athena-Verified-Content";
pub const PREDICTION: &str = "Athena-Prediction";

/// The path of the record that closes the bootstrap range.
pub const BOOTSTRAP_OUTCOME: &str = "records/jobs/bootstrap/outcome.json";

/// What a history walk carries from commit to commit.
#[derive(Debug, Default)]
pub struct HistoryState {
    /// The tip of the history being checked, where the bootstrap outcome is looked up.
    pub tip: String,
    /// Jobs already landed by an earlier commit in the walk.
    pub landed: BTreeSet<String>,
}

/// The bootstrap range: open until `records/jobs/bootstrap/outcome.json` names its last commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootstrapRange {
    Open,
    ClosedThrough(String),
}

/// The state of the bootstrap range as recorded in `treeish`.
pub fn bootstrap_range(git: &Git, treeish: &str) -> Result<BootstrapRange> {
    match git.show_file(treeish, BOOTSTRAP_OUTCOME)? {
        None => Ok(BootstrapRange::Open),
        Some(bytes) => match AnyRecord::parse(BOOTSTRAP_OUTCOME, &bytes) {
            Ok(AnyRecord::Outcome(outcome)) => match &outcome.through_commit {
                Some(through) => Ok(BootstrapRange::ClosedThrough(through.clone())),
                None => Ok(BootstrapRange::ClosedThrough(String::new())),
            },
            _ => Ok(BootstrapRange::ClosedThrough(String::new())),
        },
    }
}

/// Checks the invariant for `change`. `history` is set during a history walk.
pub fn check(
    git: &Git,
    change: &Change,
    judged: &Judged,
    history: Option<&HistoryState>,
) -> Result<Vec<Violation>> {
    let commit = change.commit.as_deref();
    if let Some(id) = commit
        && judged
            .config
            .history
            .exempt
            .iter()
            .any(|e| !e.is_empty() && id.starts_with(e.as_str()))
    {
        return Ok(Vec::new());
    }
    let trailers = change.trailers(git)?;
    let fail = |detail: String, repair: &str| {
        vec![Violation::new(Rule::Invariant, detail, repair).in_commit(commit)]
    };

    let fp = fingerprint(git, &change.tree, &judged.rules)?;
    let parent_fps: Vec<String> = if change.parents.is_empty() {
        vec![fingerprint(git, &judged.base, &judged.rules)?]
    } else {
        change
            .parents
            .iter()
            .map(|p| fingerprint(git, p, &judged.rules))
            .collect::<std::result::Result<_, _>>()?
    };

    if change.parents.len() > 1 {
        if parent_fps.contains(&fp) {
            return Ok(Vec::new());
        }
        if landing(git, change, judged, &trailers, &fp, history)?.is_ok()
            || intervention(judged, &trailers).is_ok()
        {
            return Ok(Vec::new());
        }
        return Ok(fail(
            "a merge introduces governed content that neither parent has".into(),
            "merge only clean histories, or land the combined change through a job",
        ));
    }

    if fp == parent_fps[0] {
        // Records, owner paths, or nothing changed.
        let only_owner = judged.diff.iter().all(|d| judged.class(&d.path).is_owner());
        if only_owner {
            return Ok(Vec::new());
        }
        return Ok(names_existing_work(git, change, &trailers)?
            .map(|problem| {
                fail(
                    problem,
                    "commit records through athena, or add an Athena-Job, Athena-Intervention, or Athena-Record \
                     trailer naming the work they belong to",
                )
            })
            .unwrap_or_default());
    }

    // Governed content changed.
    let landing_result = landing(git, change, judged, &trailers, &fp, history)?;
    if landing_result.is_ok() {
        return Ok(Vec::new());
    }
    if intervention(judged, &trailers).is_ok() {
        return Ok(Vec::new());
    }
    let jobs = trailer_values(&trailers, JOB);
    if jobs.contains(&JobId::BOOTSTRAP) {
        let tip = history
            .map(|h| h.tip.as_str())
            .unwrap_or(change.tree.as_str());
        match bootstrap_range(git, tip)? {
            BootstrapRange::Open => return Ok(Vec::new()),
            BootstrapRange::ClosedThrough(through) => {
                if let Some(id) = commit
                    && !through.is_empty()
                    && git.rev_parse(&through)?.is_some()
                    && git.is_ancestor(id, &through)?
                {
                    return Ok(Vec::new());
                }
                return Ok(fail(
                    format!(
                        "Athena-Job: bootstrap after the bootstrap range closed at {}",
                        short(&through)
                    ),
                    "governed changes now go through `athena ask`, or are declared with `athena intervene --with-staged`",
                ));
            }
        }
    }
    let why = match landing_result {
        Err(Some(reason)) => reason,
        _ => "governed content changed without a verified landing, a declared intervention, or the open bootstrap range"
            .to_string(),
    };
    Ok(fail(
        why,
        "land changes through `athena ask`/`athena go`, or stage hand work and run \
         `athena intervene \"<what>\" --why ... --class ... --category ... --with-staged`",
    ))
}

/// The job a commit validly lands (case a), if it is a landing commit at all.
pub fn landing_job(
    git: &Git,
    change: &Change,
    judged: &Judged,
    history: Option<&HistoryState>,
) -> Result<Option<JobId>> {
    let trailers = change.trailers(git)?;
    let fp = fingerprint(git, &change.tree, &judged.rules)?;
    Ok(landing(git, change, judged, &trailers, &fp, history)?.ok())
}

/// `Ok(job)` when the commit is a valid landing of `job` (case a); `Err(Some(reason))` when it claims to
/// be one but is not; `Err(None)` when it does not claim to be one.
fn landing(
    git: &Git,
    change: &Change,
    judged: &Judged,
    trailers: &[(String, String)],
    fp: &str,
    history: Option<&HistoryState>,
) -> Result<std::result::Result<JobId, Option<String>>> {
    let claimed = trailer_values(trailers, VERIFIED_CONTENT);
    let Some(claimed) = claimed.first() else {
        return Ok(Err(None));
    };
    let refuse = |reason: String| Ok(Err(Some(reason)));
    if *claimed != fp {
        return refuse(format!(
            "Athena-Verified-Content {claimed} does not match the landed content {fp}"
        ));
    }
    let Some(job) = trailer_values(trailers, JOB)
        .first()
        .and_then(|j| JobId::parse(j).ok())
        .filter(JobId::is_request)
    else {
        return refuse("a landing commit needs an Athena-Job trailer naming its job".into());
    };
    let Some(parent) = change.parents.first() else {
        return refuse("a root commit cannot land a job".into());
    };
    let dir = job.dir();

    // The job was read and dispatched before it landed.
    let before = JobFiles::load(git, parent, &job)?;
    let Some(dispatched) = &before.job else {
        return refuse(format!(
            "{dir}/job.json is not on the parent commit, so no dispatched job lands here"
        ));
    };
    let Some(n) = dispatched.accepted_reading else {
        return refuse(format!("{dir}/job.json names no accepted reading"));
    };
    let Some(prediction) = trailer_values(trailers, PREDICTION).first().copied() else {
        return refuse(
            "a landing commit needs an Athena-Prediction trailer naming the commit that added its accepted reading"
                .into(),
        );
    };
    if let Some(problem) = prediction_problem(git, parent, &job, n, prediction)? {
        return refuse(problem);
    }

    // It lands with its own closing records.
    let after = JobFiles::load(git, &change.tree, &job)?;
    let added = |name: &str| {
        let path = format!("{dir}/{name}");
        judged
            .diff
            .iter()
            .any(|d| d.status == 'A' && d.path == path)
    };
    let fresh = added("candidate.json")
        && added("verification.json")
        && added("outcome.json")
        && after
            .outcome
            .as_ref()
            .is_some_and(|o| o.state == OutcomeState::Landed);
    let retried = added("outcome-landed.json")
        && before
            .outcome
            .as_ref()
            .is_some_and(|o| o.state == OutcomeState::Blocked);
    let (outcome_path, outcome) = match (fresh, retried) {
        (true, _) => (format!("{dir}/outcome.json"), after.outcome.as_ref()),
        (false, true) => (
            format!("{dir}/outcome-landed.json"),
            after.later_outcome.as_ref(),
        ),
        (false, false) => {
            return refuse(format!(
                "a landing commit adds {dir}/candidate.json, verification.json, and a landed outcome.json, \
                 or outcome-landed.json after a blocked outcome"
            ));
        }
    };
    let (Some(candidate), Some(verified), Some(outcome)) =
        (&after.candidate, &after.verification, outcome)
    else {
        return refuse(format!(
            "{dir} lacks the candidate, verification, or outcome record a landing needs"
        ));
    };
    if candidate.fingerprint.as_deref() != Some(fp) {
        return refuse(format!(
            "{dir}/candidate.json does not hold the landed content {fp}"
        ));
    }
    if verified.verified_fingerprint != fp {
        return refuse(format!(
            "{dir}/verification.json verified {}, not the landed content {fp}",
            verified.verified_fingerprint
        ));
    }
    if verified.final_verdict != Verdict::Pass {
        return refuse(format!(
            "{dir}/verification.json records a final verdict of fail"
        ));
    }
    if candidate.candidate_commit.as_deref() != Some(verified.candidate_commit.as_str()) {
        return refuse(format!(
            "{dir}/verification.json and candidate.json name different candidate commits"
        ));
    }

    // An independent verifier ran, and its usage reconciles.
    let Some(seq) = verified.verifier_invocation else {
        return refuse(format!(
            "{dir}/verification.json names no verifier invocation"
        ));
    };
    let Some((path, invocation)) = after.invocations.iter().find(|(_, i)| i.seq == seq) else {
        return refuse(format!("{dir} has no record of verifier invocation {seq}"));
    };
    if invocation.role != Role::Verifier || invocation.outcome != InvocationOutcome::Success {
        return refuse(format!("{path} is not a successful verifier invocation"));
    }
    if dispatched.verifier_model.as_deref() != Some(invocation.model.as_str()) {
        return refuse(format!(
            "{path} used {}, not the job's verifier model",
            invocation.model
        ));
    }
    if let Some(v) = reconcile::check_invocation(path, invocation).first() {
        return refuse(v.detail.clone());
    }
    let invocations: Vec<&Invocation> = after.invocations.iter().map(|(_, i)| i).collect();
    if let Some(v) = reconcile::check_outcome(&outcome_path, outcome, &invocations).first() {
        return refuse(v.detail.clone());
    }

    let already = match history {
        Some(state) => state.landed.contains(job.as_str()),
        None => earlier_landing(git, parent, job.as_str())?,
    };
    if already {
        return refuse(format!("job {} has already landed", job.as_str()));
    }
    Ok(Ok(job))
}

/// Why `prediction` is not the commit that added reading `n` of `job` before `parent`, if it is not.
fn prediction_problem(
    git: &Git,
    parent: &str,
    job: &JobId,
    n: u32,
    prediction: &str,
) -> Result<Option<String>> {
    let Some(predicted) = git.rev_parse(prediction)? else {
        return Ok(Some(format!(
            "Athena-Prediction {prediction} is not a commit"
        )));
    };
    if !git.is_ancestor(&predicted, parent)? {
        return Ok(Some(format!(
            "the prediction commit {} is not an ancestor",
            short(&predicted)
        )));
    }
    let reading = format!("{}/reading-{n}.json", job.dir());
    if !git.added_paths(&predicted)?.contains(&reading) {
        return Ok(Some(format!(
            "the prediction commit {} did not add {reading}",
            short(&predicted)
        )));
    }
    Ok(None)
}

/// Whether a commit reachable from `rev` already landed `job`.
pub fn earlier_landing(git: &Git, rev: &str, job: &str) -> Result<bool> {
    for commit in git.log_grep(rev, &format!("^{JOB}: {}$", regex::escape(job)))? {
        let trailers = git.trailers(&git.message(&commit)?)?;
        if !trailer_values(&trailers, VERIFIED_CONTENT).is_empty() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `Ok` when the commit declares an intervention and adds its record (case b).
fn intervention(judged: &Judged, trailers: &[(String, String)]) -> std::result::Result<(), ()> {
    for value in trailer_values(trailers, INTERVENTION) {
        if let Ok(id) = RecordId::parse(value) {
            let path = format!("records/interventions/{}/{}.json", id.month(), id.as_str());
            if judged
                .diff
                .iter()
                .any(|d| d.status == 'A' && d.path == path)
            {
                return Ok(());
            }
        }
    }
    Err(())
}

/// `None` when a records-only commit names existing work; otherwise the problem.
fn names_existing_work(
    git: &Git,
    change: &Change,
    trailers: &[(String, String)],
) -> Result<Option<String>> {
    if change.message.is_none() {
        return Ok(None);
    }
    for value in trailer_values(trailers, JOB) {
        if let Ok(job) = JobId::parse(value)
            && git.path_exists(&change.tree, &job.dir())?
        {
            return Ok(None);
        }
    }
    for value in trailer_values(trailers, INTERVENTION) {
        if let Ok(id) = RecordId::parse(value)
            && git.path_exists(
                &change.tree,
                &format!("records/interventions/{}/{}.json", id.month(), id.as_str()),
            )?
        {
            return Ok(None);
        }
    }
    for value in trailer_values(trailers, RECORD) {
        if git.path_exists(&change.tree, value)? {
            return Ok(None);
        }
    }
    Ok(Some(
        "a commit that changes records names no job, intervention, or record that exists".into(),
    ))
}

/// The prediction trailer: `Athena-Prediction` names an earlier commit that added the job's accepted reading.
pub fn check_prediction(git: &Git, change: &Change) -> Result<Vec<Violation>> {
    let commit = change.commit.as_deref();
    let trailers = change.trailers(git)?;
    let mut violations = Vec::new();
    for prediction in trailer_values(&trailers, PREDICTION) {
        let fail = |detail: String| {
            Violation::new(
                Rule::Prediction,
                detail,
                "close jobs through athena, which names the commit that added the accepted reading",
            )
            .in_commit(commit)
        };
        let Some(job) = trailer_values(&trailers, JOB)
            .first()
            .and_then(|j| JobId::parse(j).ok())
        else {
            violations.push(fail(
                "Athena-Prediction without an Athena-Job trailer".into(),
            ));
            continue;
        };
        let Some(parent) = change.parents.first() else {
            violations.push(fail("a root commit cannot follow its prediction".into()));
            continue;
        };
        let job_path = format!("{}/job.json", job.dir());
        let accepted = match git.show_file(&change.tree, &job_path)? {
            Some(bytes) => match AnyRecord::parse(&job_path, &bytes) {
                Ok(AnyRecord::Job(j)) => j.accepted_reading,
                _ => None,
            },
            None => None,
        };
        let Some(n) = accepted else {
            violations.push(fail(format!("{job_path} names no accepted reading")));
            continue;
        };
        if let Some(problem) = prediction_problem(git, parent, &job, n, prediction)? {
            violations.push(fail(problem));
        }
    }
    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    fn judge(git: &Git, rev: &str) -> Vec<Violation> {
        let id = git.rev_parse(rev).unwrap().unwrap();
        let change = Change::of_commit(git, &id).unwrap();
        let judged = Judged::new(git, &change).unwrap();
        check(git, &change, &judged, None).unwrap()
    }

    fn owner_repo() -> (tempfile::TempDir, Git) {
        repo(&[
            (
                ".gitattributes",
                "/OWNER.md athena-exclude\n/SPEC.md athena-readonly\n",
            ),
            ("src/lib.rs", "pub fn a() {}\n"),
        ])
    }

    fn intervention_record(id: &str) -> (String, String) {
        let rid = RecordId::parse(id).unwrap();
        let path = format!("records/interventions/{}/{id}.json", rid.month());
        let text = format!(
            "{{\n  \"schema\": \"athena.intervention/1\",\n  \"id\": \"{id}\",\n  \"at\": \"2026-10-10T15:30:12Z\",\n  \"what\": \"Fixed it by hand\",\n  \"why\": \"Faster\",\n  \"class_of_work\": \"code\",\n  \"category\": \"did-work-themselves\",\n  \"job\": null,\n  \"source\": \"user\",\n  \"with_staged_changes\": true\n}}\n"
        );
        (path, text)
    }

    #[test]
    fn a_hand_change_without_an_intervention_is_refused() {
        // M6 (blocks): a hand change without an intervention record.
        let (dir, git) = owner_repo();
        write(
            dir.path(),
            &[("src/lib.rs", "pub fn a() { println!(); }\n")],
        );
        commit(&git, "hand edit\n");
        assert_eq!(judge(&git, "HEAD").len(), 1);
    }

    #[test]
    fn a_code_change_carrying_an_earlier_jobs_trailer_is_refused() {
        // M6 (blocks): a code change carrying an earlier job's trailer.
        let (dir, git) = owner_repo();
        write(
            dir.path(),
            &[(
                "records/jobs/2026-10/20261010-153012-k7q2/request.json",
                "{}",
            )],
        );
        commit(&git, "records\n\nAthena-Job: 20261010-153012-k7q2\n");
        write(dir.path(), &[("src/lib.rs", "pub fn a() { todo!() }\n")]);
        commit(&git, "sneak\n\nAthena-Job: 20261010-153012-k7q2\n");
        assert_eq!(judge(&git, "HEAD").len(), 1);
    }

    #[test]
    fn an_intervention_commit_with_its_record_passes() {
        // M6 (allows): an intervention commit with its record.
        let (dir, git) = owner_repo();
        let (path, text) = intervention_record("20261010-160000-abcd");
        write(
            dir.path(),
            &[
                ("src/lib.rs", "pub fn a() { /* by hand */ }\n"),
                (&path, &text),
            ],
        );
        commit(
            &git,
            "Fix by hand\n\nAthena-Intervention: 20261010-160000-abcd\n",
        );
        assert_eq!(judge(&git, "HEAD"), vec![]);
    }

    #[test]
    fn an_intervention_trailer_without_its_record_is_refused() {
        let (dir, git) = owner_repo();
        write(
            dir.path(),
            &[("src/lib.rs", "pub fn a() { /* by hand */ }\n")],
        );
        commit(
            &git,
            "Fix by hand\n\nAthena-Intervention: 20261010-160000-abcd\n",
        );
        assert_eq!(judge(&git, "HEAD").len(), 1);
    }

    #[test]
    fn records_only_and_owner_only_commits_pass() {
        // M6 (allows): a records-only commit; a commit touching only owner paths.
        let (dir, git) = owner_repo();
        write(dir.path(), &[("OWNER.md", "mine\n"), ("SPEC.md", "spec\n")]);
        commit(&git, "owner edits, no trailer\n");
        assert_eq!(judge(&git, "HEAD"), vec![]);
        let (path, text) = intervention_record("20261010-170000-abcd");
        write(dir.path(), &[(&path, &text)]);
        commit(
            &git,
            "record\n\nAthena-Intervention: 20261010-170000-abcd\n",
        );
        assert_eq!(judge(&git, "HEAD"), vec![]);
    }

    #[test]
    fn a_records_only_commit_must_name_existing_work() {
        let (dir, git) = owner_repo();
        let (path, text) = intervention_record("20261010-170000-abcd");
        write(dir.path(), &[(&path, &text)]);
        commit(&git, "record without a trailer\n");
        assert_eq!(judge(&git, "HEAD").len(), 1);
    }

    #[test]
    fn a_clean_merge_passes_and_a_merge_with_new_content_is_refused() {
        // M6 (allows): a clean merge. (blocks): a merge that introduces new governed content.
        let (dir, git) = owner_repo();
        let base = git.head().unwrap().unwrap();
        let (path, text) = intervention_record("20261010-180000-abcd");
        write(
            dir.path(),
            &[("src/lib.rs", "pub fn a() { 1; }\n"), (&path, &text)],
        );
        commit(&git, "hand\n\nAthena-Intervention: 20261010-180000-abcd\n");
        git.run(["checkout", "-q", "-b", "side", &base]).unwrap();
        write(dir.path(), &[("OWNER.md", "owner\n")]);
        commit(&git, "owner\n");
        git.run(["checkout", "-q", "main"]).unwrap();
        git.run(["merge", "-q", "--no-ff", "-m", "merge side", "side"])
            .unwrap();
        assert_eq!(judge(&git, "HEAD"), vec![]);

        git.run(["checkout", "-q", "-b", "evil", &base]).unwrap();
        write(dir.path(), &[("src/other.rs", "pub fn b() {}\n")]);
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "side code\n\nAthena-Job: bootstrap"])
            .unwrap();
        git.run(["checkout", "-q", "main"]).unwrap();
        git.run(["merge", "-q", "--no-ff", "--no-commit", "evil"])
            .unwrap();
        write(dir.path(), &[("src/extra.rs", "pub fn c() {}\n")]);
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "merge with extra content"])
            .unwrap();
        assert_eq!(judge(&git, "HEAD").len(), 1);
    }

    #[test]
    fn bootstrap_commits_pass_while_the_range_is_open_and_not_after() {
        // M6 (blocks): a bootstrap commit after the range is closed.
        let (dir, git) = owner_repo();
        write(dir.path(), &[("src/lib.rs", "pub fn a() { 2; }\n")]);
        let in_range = commit(&git, "bootstrap work\n\nAthena-Job: bootstrap\n");
        assert_eq!(judge(&git, "HEAD"), vec![]);
        let outcome = format!(
            "{{\n  \"schema\": \"athena.outcome/1\",\n  \"job\": \"bootstrap\",\n  \"state\": \"closed\",\n  \"reason\": \"accepted\",\n  \"at\": \"2026-10-10T18:00:00Z\",\n  \"totals\": null,\n  \"branch\": null,\n  \"base_commit\": null,\n  \"candidate_commit\": null,\n  \"after_blocked\": false,\n  \"through_commit\": \"{in_range}\"\n}}\n"
        );
        write(dir.path(), &[(BOOTSTRAP_OUTCOME, &outcome)]);
        commit(&git, "close the bootstrap\n\nAthena-Job: bootstrap\n");
        assert_eq!(judge(&git, "HEAD"), vec![]);
        assert_eq!(
            judge(&git, &in_range),
            vec![],
            "commits inside the range still pass"
        );
        write(dir.path(), &[("src/lib.rs", "pub fn a() { 3; }\n")]);
        commit(&git, "late bootstrap work\n\nAthena-Job: bootstrap\n");
        assert_eq!(judge(&git, "HEAD").len(), 1);
    }

    fn verification_record(job: &str, fingerprint: &str) -> String {
        format!(
            "{{\n  \"schema\": \"athena.verification/1\",\n  \"job\": \"{job}\",\n  \"candidate_commit\": \"{c}\",\n  \"verified_fingerprint\": \"{fingerprint}\",\n  \"tool_config_from_base\": [],\n  \"checks\": [],\n  \"check_cleanup\": [],\n  \"verifier_invocation\": null,\n  \"criteria\": [],\n  \"findings\": [],\n  \"request_reading_mismatch\": null,\n  \"agent_verdict\": \"pass\",\n  \"final_verdict\": \"pass\",\n  \"reasons\": [],\n  \"inputs\": [],\n  \"summary\": null\n}}\n",
            c = "0".repeat(40)
        )
    }

    #[test]
    fn a_landing_whose_content_differs_from_its_verification_is_refused() {
        // M7 (blocks): a landing commit whose content differs from its verification (the CI audit).
        let job = "20261010-153012-k7q2";
        let dir_of = |job: &str| JobId::parse(job).unwrap().dir();
        let (dir, git) = owner_repo();
        let base = git.head().unwrap().unwrap();
        write(
            dir.path(),
            &[("src/lib.rs", "pub fn a() { /* landed */ }\n")],
        );
        git.run(["add", "-A"]).unwrap();
        let tree = git.write_index_tree().unwrap();
        let landed = fingerprint(&git, &tree, &base).unwrap();
        let verified_elsewhere = fingerprint(&git, &base, &base).unwrap();
        write(
            dir.path(),
            &[(
                &format!("{}/verification.json", dir_of(job)),
                &verification_record(job, &verified_elsewhere),
            )],
        );
        commit(
            &git,
            &format!("Land\n\nAthena-Job: {job}\nAthena-Verified-Content: {verified_elsewhere}\n"),
        );
        assert_eq!(
            judge(&git, "HEAD").len(),
            1,
            "the trailer does not match the landed content"
        );

        let (dir, git) = owner_repo();
        let base = git.head().unwrap().unwrap();
        write(
            dir.path(),
            &[("src/lib.rs", "pub fn a() { /* landed */ }\n")],
        );
        git.run(["add", "-A"]).unwrap();
        let tree = git.write_index_tree().unwrap();
        let landed_here = fingerprint(&git, &tree, &base).unwrap();
        assert_eq!(landed, landed_here);
        write(
            dir.path(),
            &[(
                &format!("{}/verification.json", dir_of(job)),
                &verification_record(job, &landed_here),
            )],
        );
        commit(
            &git,
            &format!("Land\n\nAthena-Job: {job}\nAthena-Verified-Content: {landed_here}\n"),
        );
        // A matching verification record alone is not a landing: no job was read, dispatched, or
        // verified (acceptance verification 1, M1). Real landings pass in the lifecycle tests.
        let found = judge(&git, "HEAD");
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(
            found[0]
                .detail
                .contains("job.json is not on the parent commit")
        );
    }

    #[test]
    fn a_prediction_trailer_must_name_an_ancestor_that_added_the_reading() {
        // M23 (blocks): a closing commit whose prediction commit is not an ancestor, or did not add the
        // accepted reading.
        let job = "20261010-153012-k7q2";
        let dir = JobId::parse(job).unwrap().dir();
        let (root, git) = owner_repo();
        let unrelated = git.head().unwrap().unwrap();
        write(root.path(), &[(&format!("{dir}/reading-1.json"), "{}")]);
        let prediction = commit(&git, &format!("Read\n\nAthena-Job: {job}\n"));
        let job_record = format!(
            "{{\"schema\":\"athena.job/1\",\"id\":\"{job}\",\"kind\":\"request\",\"title\":\"t\",\"class_of_work\":\"docs\",\"capability\":\"change\",\"accepted_reading\":1,\"accepted_by\":\"user\",\"owner\":{{\"role\":\"producer\",\"capability\":null,\"model\":null,\"note\":null}},\"verifier_model\":null,\"caps\":null,\"base_commit\":null,\"config_sha256\":null,\"defaults_sha256\":null,\"athena\":null,\"driver\":null,\"brief_sha256\":null,\"dispatched_at\":null,\"request_path\":null,\"request_sha256\":null,\"cost\":null,\"tokens\":null,\"note\":null}}"
        );
        write(root.path(), &[(&format!("{dir}/job.json"), &job_record)]);
        commit(&git, &format!("Dispatch\n\nAthena-Job: {job}\n"));
        write(root.path(), &[(&format!("{dir}/outcome.json"), "{}")]);
        let good = commit(
            &git,
            &format!("Close\n\nAthena-Job: {job}\nAthena-Prediction: {prediction}\n"),
        );
        assert_eq!(
            check_prediction(&git, &Change::of_commit(&git, &good).unwrap()).unwrap(),
            vec![]
        );
        write(root.path(), &[(&format!("{dir}/score.json"), "{}")]);
        let wrong = commit(
            &git,
            &format!("Close\n\nAthena-Job: {job}\nAthena-Prediction: {unrelated}\n"),
        );
        assert_eq!(
            check_prediction(&git, &Change::of_commit(&git, &wrong).unwrap())
                .unwrap()
                .len(),
            1,
            "it did not add the reading"
        );
        git.run(["checkout", "-q", "-b", "other", &unrelated])
            .unwrap();
        write(root.path(), &[(&format!("{dir}/job.json"), &job_record)]);
        let elsewhere = commit(
            &git,
            &format!("Close\n\nAthena-Job: {job}\nAthena-Prediction: {prediction}\n"),
        );
        assert_eq!(
            check_prediction(&git, &Change::of_commit(&git, &elsewhere).unwrap())
                .unwrap()
                .len(),
            1,
            "not an ancestor"
        );
    }

    #[test]
    fn exempt_commits_are_skipped() {
        let (_dir, git) = owner_repo();
        let root = git.head().unwrap().unwrap();
        let change = Change::of_commit(&git, &root).unwrap();
        let mut judged = Judged::new(&git, &change).unwrap();
        judged.config.history.exempt = vec![root.clone()];
        assert_eq!(check(&git, &change, &judged, None).unwrap(), vec![]);
    }
}
