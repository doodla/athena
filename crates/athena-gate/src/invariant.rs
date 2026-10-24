//! The governed-content invariant (design §7.3, D39) and the prediction-trailer check (§7.4 check 7, D07).
//!
//! Let X be a commit and R the rules of its first parent. If fingerprint(X; R) equals the first parent's
//! fingerprint, X touched only records or owner paths; it must name its work with an `Athena-Job:`,
//! `Athena-Intervention:`, or `Athena-Record:` trailer, unless it touched only owner paths. If the
//! fingerprint changed, X must be exactly one of:
//!
//! - (a) a landing commit of a job that Athena read, dispatched, and verified (D55). The job is the one
//!   whose closing records X adds: X adds its candidate, verification, and landed outcome (or, after
//!   `blocked`, its `outcome-landed.json`). Nothing is taken from trailers or ancestry, so a merge, a
//!   squash merge, and a rebase merge all land the same way. The landed tree holds the job's `job.json`,
//!   naming an accepted reading, and that reading; the reading's recorded time and invocation number
//!   precede the dispatch and every producer and verifier invocation the records list (prediction before
//!   work, proved from the records); the candidate and the verification both hold X's fingerprint, the
//!   final verdict is pass, the verifier invocation the verification names is recorded, succeeded, used
//!   the job's verifier model, and reconciles, as do the outcome's totals; and the job has not landed
//!   before: its landed outcome is not on the parent, and (in a history walk) no earlier commit validly
//!   landed it. A squash may carry the jobs of a chain (D69): then each job passes all of this except that
//!   only the last holds X's fingerprint, and the jobs' `athena.candidate/2` records must link the parent's
//!   content to X's (see `chain_problem`). Trailers are only hints that must agree: an `Athena-Verified-Content` that is not X's
//!   fingerprint, or an `Athena-Job` naming another request job, refuses the landing; `Athena-Prediction`
//!   is ignored;
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
use std::collections::{BTreeMap, BTreeSet};

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
    /// The `[history] exempt` commit ids, read from the tip's configuration. Each commit is otherwise
    /// judged by its parent's configuration, which cannot hold an entry added later, so an adopter
    /// could never excuse history from before Athena. Validation still stops an entry from covering
    /// a commit made after it.
    pub exempt: Vec<String>,
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

/// Whether `change` is a commit on the `[history] exempt` list. The invariant and the job-records check
/// both skip such a commit (D66, D67 and the decision that followed them): an exemption excuses the
/// commit's content and the job records it adds, which are judged as part of the same landing.
///
/// The history check reads the exempt list from the tip's configuration, not the judged commit's
/// parent's: an entry is added after the commit it excuses, so the parent's list never holds it.
/// A per-commit check has no tip and uses the parent's list. An exemption names one commit in full
/// (the configuration refuses anything shorter), and only a commit that existed before the entry was
/// added (the content check), so it cannot cover a later one.
pub fn is_exempt(change: &Change, judged: &Judged, history: Option<&HistoryState>) -> bool {
    let exempt = match history {
        Some(state) => &state.exempt,
        None => &judged.config.history.exempt,
    };
    change
        .commit
        .as_deref()
        .is_some_and(|id| exempt.iter().any(|e| e == id))
}

/// Checks the invariant for `change`. `history` is set during a history walk.
pub fn check(
    git: &Git,
    change: &Change,
    judged: &Judged,
    history: Option<&HistoryState>,
) -> Result<Vec<Violation>> {
    let commit = change.commit.as_deref();
    if is_exempt(change, judged, history) {
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

/// The jobs a commit validly lands (case a): one job, or every job of a chain. Empty when the commit is
/// not a valid landing.
pub fn landing_job(
    git: &Git,
    change: &Change,
    judged: &Judged,
    history: Option<&HistoryState>,
) -> Result<BTreeSet<JobId>> {
    let trailers = change.trailers(git)?;
    let fp = fingerprint(git, &change.tree, &judged.rules)?;
    Ok(landing(git, change, judged, &trailers, &fp, history)?.unwrap_or_default())
}

/// The jobs whose closing records (`candidate.json`, `verification.json`, `outcome.json`, or
/// `outcome-landed.json`) `judged` adds. A commit that adds none does not claim to land a job.
///
/// A job whose recorded outcome in the commit's tree is not landed (rejected, blocked, or failed) is left
/// out. A squash ships the records of such a job, but its work never reached the branch, so it is no link
/// of the chain: counting it would make its landed retry, which names it as `previous_job`, look like a
/// fork. A job with no outcome record at all stays in, so that the landing check can refuse it.
///
/// This is the single answer to "which jobs does this commit claim to land": the landing check uses it, and
/// so does the classification of a pull request's commits (`athena_core::land::pr_commits`).
pub fn closing_jobs(git: &Git, change: &Change, judged: &Judged) -> Result<BTreeSet<JobId>> {
    const CLOSING: [&str; 4] = [
        "candidate.json",
        "verification.json",
        "outcome.json",
        "outcome-landed.json",
    ];
    let mut jobs = BTreeSet::new();
    for d in judged.diff.iter().filter(|d| d.status == 'A') {
        let mut parts = d.path.split('/');
        if let (Some("records"), Some("jobs"), Some(_month), Some(id), Some(name), None) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) && CLOSING.contains(&name)
            && let Ok(job) = JobId::parse(id)
            && job.is_request()
            && d.path == format!("{}/{name}", job.dir())
            && !not_landed(&JobFiles::load(git, &change.tree, &job)?)
        {
            jobs.insert(job);
        }
    }
    Ok(jobs)
}

/// Whether `files` record an outcome and none of the recorded outcomes is landed.
fn not_landed(files: &JobFiles) -> bool {
    let outcomes: Vec<_> = [&files.outcome, &files.later_outcome]
        .into_iter()
        .flatten()
        .collect();
    !outcomes.is_empty() && outcomes.iter().all(|o| o.state != OutcomeState::Landed)
}

/// `Ok(jobs)` when the commit is a valid landing of `jobs` (case a): one job, or a chain of several that
/// a squash folded into one commit (design D69); `Err(Some(reason))` when it claims to be one but is not;
/// `Err(None)` when it does not claim to be one.
///
/// The jobs are identified by the closing records the commit adds, never by trailers; the proof that each
/// was read before its work is in the landed records (see [`prediction_problem`]); and ancestry is not
/// used, so a squash or rebase merge, which leaves no ancestor to point at, lands the same way a merge
/// does. Trailers are hints that must agree with the records: a verified-content value that is not the
/// landed content, or an `Athena-Job` that names a request job the commit does not land, refuses the
/// landing.
///
/// One job is judged by [`job_problem`] against the landed content, as it always was. Several jobs must
/// first form one chain ([`chain_problem`]); then every job passes [`job_problem`], and only the last has
/// to hold the landed content.
fn landing(
    git: &Git,
    change: &Change,
    judged: &Judged,
    trailers: &[(String, String)],
    fp: &str,
    history: Option<&HistoryState>,
) -> Result<std::result::Result<BTreeSet<JobId>, Option<String>>> {
    let refuse = |reason: String| Ok(Err(Some(reason)));
    let jobs = closing_jobs(git, change, judged)?;
    if jobs.is_empty() {
        return Ok(Err(None));
    }
    for claimed in trailer_values(trailers, VERIFIED_CONTENT) {
        if claimed != fp {
            return refuse(format!(
                "Athena-Verified-Content {claimed} does not match the landed content {fp}"
            ));
        }
    }
    for named in trailer_values(trailers, JOB) {
        if let Ok(named) = JobId::parse(named)
            && named.is_request()
            && !jobs.contains(&named)
        {
            return refuse(match jobs.iter().next() {
                Some(only) if jobs.len() == 1 => format!(
                    "the Athena-Job trailer names {}, but the commit adds the records of {}",
                    named.as_str(),
                    only.as_str()
                ),
                _ => format!(
                    "the Athena-Job trailer names {}, which is not one of the jobs whose records the commit adds",
                    named.as_str()
                ),
            });
        }
    }
    let Some(parent) = change.parents.first() else {
        return refuse("a root commit cannot land a job".into());
    };
    if jobs.len() == 1 {
        let job = jobs.iter().next().expect("one job");
        return match job_problem(git, change, judged, parent, job, Some(fp), history)? {
            Some(reason) => refuse(reason),
            None => Ok(Ok(jobs)),
        };
    }
    let order = match chain_problem(git, change, judged, parent, &jobs, fp)? {
        Ok(order) => order,
        Err(reason) => return refuse(reason),
    };
    for job in &order {
        // Only the last job has to hold the landed content; the link checks tie the others to it.
        let landed = (order.last() == Some(job)).then_some(fp);
        if let Some(reason) = job_problem(git, change, judged, parent, job, landed, history)? {
            return refuse(reason);
        }
    }
    Ok(Ok(jobs))
}

/// Whether `path` is a file that classifies paths (a `.gitattributes`).
fn is_rules_file(path: &str) -> bool {
    path.rsplit('/').next() == Some(".gitattributes")
}

/// Why `jobs`, the several jobs a commit adds closing records for, do not form one chain from the parent's
/// content to the landed content `fp`; otherwise the jobs in chain order (design D69, §4.2 steps 2-5).
///
/// The order comes from the `previous_job` each `athena.candidate/2` records, never from the fingerprints,
/// which may repeat. Every fingerprint in a chain is computed under the parent's rules, so a chain that
/// changes a rules file is refused.
fn chain_problem(
    git: &Git,
    change: &Change,
    judged: &Judged,
    parent: &str,
    jobs: &BTreeSet<JobId>,
    fp: &str,
) -> Result<std::result::Result<Vec<JobId>, String>> {
    if change.parents.len() > 1 {
        return Ok(Err(
            "a chain landing is refused on a commit with more than one parent".into(),
        ));
    }
    let mut candidates = BTreeMap::new();
    for job in jobs {
        match JobFiles::load(git, &change.tree, job)?.candidate {
            Some(c) if c.base_fingerprint.is_some() => {
                candidates.insert(job.clone(), c);
            }
            _ => {
                return Ok(Err(format!(
                    "a chain landing needs athena.candidate/2 in {}/candidate.json",
                    job.dir()
                )));
            }
        }
    }
    for (job, c) in &candidates {
        if c.changes.iter().any(|e| is_rules_file(&e.path)) {
            return Ok(Err(format!(
                "job {} changes a path classification rules file, so its fingerprints are not comparable; land it alone",
                job.as_str()
            )));
        }
    }
    if let Some(d) = judged.diff.iter().find(|d| is_rules_file(&d.path)) {
        return Ok(Err(format!(
            "the commit changes {}, a path classification rules file, so the fingerprints of its jobs are not comparable; land the jobs one at a time",
            d.path
        )));
    }

    let parent_fp = fingerprint(git, parent, &judged.rules)?;
    // The job that follows each job, and the first sign of a fork.
    let mut followers: BTreeMap<&JobId, &JobId> = BTreeMap::new();
    for (job, c) in &candidates {
        if let Some(previous) = &c.previous_job
            && let Some(other) = followers.insert(previous, job)
        {
            return Ok(Err(format!(
                "jobs {} and {} both follow {}, so they do not form one chain",
                other.as_str(),
                job.as_str(),
                previous.as_str()
            )));
        }
    }
    let not_linked = |job: &JobId, previous: &JobId| {
        Err(format!(
            "job {} is not linked to the others: its previous job {} is not in this commit",
            job.as_str(),
            previous.as_str()
        ))
    };
    // The head has no previous job, one that already landed on the parent, or one that did not land.
    let mut heads = Vec::new();
    for (job, c) in &candidates {
        match &c.previous_job {
            None => heads.push(job),
            Some(previous) if candidates.contains_key(previous) => {}
            Some(previous) => {
                // A previous job that is recorded as not landed (a rejected try this job retries) is
                // outside the chain; the head's base must still equal the parent's content.
                if landed_at(git, parent, previous, &parent_fp)?
                    || not_landed(&JobFiles::load(git, &change.tree, previous)?)
                {
                    heads.push(job);
                } else {
                    return Ok(not_linked(job, previous));
                }
            }
        }
    }
    let no_head = || {
        Err(format!(
            "no job in the commit starts from the parent's content {parent_fp}"
        ))
    };
    let head = match heads.as_slice() {
        [] => return Ok(no_head()),
        [head] => *head,
        [first, second, ..] => {
            return Ok(Err(format!(
                "jobs {} and {} both start a chain, so they do not form one chain",
                first.as_str(),
                second.as_str()
            )));
        }
    };
    if candidates[head].base_fingerprint.as_deref() != Some(parent_fp.as_str()) {
        return Ok(no_head());
    }

    let mut order = vec![head.clone()];
    while let Some(next) = followers.get(&&order[order.len() - 1]) {
        order.push((*next).clone());
    }
    if let Some((job, previous)) = candidates
        .iter()
        .find(|(job, _)| !order.contains(job))
        .and_then(|(job, c)| c.previous_job.as_ref().map(|p| (job, p)))
    {
        return Ok(not_linked(job, previous));
    }
    for pair in order.windows(2) {
        let (earlier, later) = (&candidates[&pair[0]], &candidates[&pair[1]]);
        if later.base_fingerprint != earlier.fingerprint {
            return Ok(Err(format!(
                "job {} starts from {} but its previous job {} ended at {}",
                pair[1].as_str(),
                later.base_fingerprint.as_deref().unwrap_or("nothing"),
                pair[0].as_str(),
                earlier.fingerprint.as_deref().unwrap_or("nothing")
            )));
        }
    }
    let last = &order[order.len() - 1];
    if candidates[last].fingerprint.as_deref() != Some(fp) {
        return Ok(Err(format!(
            "the last job {} holds {}, not the landed content {fp}",
            last.as_str(),
            candidates[last].fingerprint.as_deref().unwrap_or("nothing")
        )));
    }
    Ok(Ok(order))
}

/// Whether `job` landed on `treeish` with content `fingerprint`: its closing records are there, and its
/// candidate holds that fingerprint. A `/1` candidate counts; only its fingerprint is read.
fn landed_at(git: &Git, treeish: &str, job: &JobId, fingerprint: &str) -> Result<bool> {
    let files = JobFiles::load(git, treeish, job)?;
    let landed = [&files.outcome, &files.later_outcome]
        .into_iter()
        .flatten()
        .any(|o| o.state == OutcomeState::Landed);
    Ok(landed
        && files.verification.is_some()
        && files
            .candidate
            .is_some_and(|c| c.fingerprint.as_deref() == Some(fingerprint)))
}

/// Why `job`, one of the jobs `change` lands, is not validly landed by it, if it is not. `landed` is the
/// content the job's candidate and verification must hold: the landed content for a single job and for the
/// last job of a chain. For an earlier job of a chain it is `None`, and the job only has to hold what it
/// verified (the chain's links tie that to the next job).
fn job_problem(
    git: &Git,
    change: &Change,
    judged: &Judged,
    parent: &str,
    job: &JobId,
    landed: Option<&str>,
    history: Option<&HistoryState>,
) -> Result<Option<String>> {
    let refuse = |reason: String| Ok(Some(reason));
    let dir = job.dir();

    // The job was read and dispatched, and the reading came before the work, as its records show.
    let before = JobFiles::load(git, parent, job)?;
    let after = JobFiles::load(git, &change.tree, job)?;
    let Some(dispatched) = &after.job else {
        return refuse(format!(
            "{dir}/job.json is not in the landed tree, so no dispatched job lands here"
        ));
    };
    let Some(n) = dispatched.accepted_reading else {
        return refuse(format!("{dir}/job.json names no accepted reading"));
    };
    if let Some(problem) = prediction_problem(&after, job, n) {
        return refuse(problem);
    }

    // It lands with its own closing records.
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
    // After a blocked outcome, which the parent holds, or which a squash brings in with the rest.
    let retried = added("outcome-landed.json")
        && after
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
    if let Some(fp) = landed {
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
    } else if candidate.fingerprint.as_deref() != Some(verified.verified_fingerprint.as_str()) {
        return refuse(format!(
            "{dir}/verification.json verified {}, not the content {dir}/candidate.json holds",
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

    // No second landing: the parent's records show none, and (in a history walk) no earlier commit did.
    let landed_before = [&before.outcome, &before.later_outcome]
        .into_iter()
        .flatten()
        .any(|o| o.state == OutcomeState::Landed);
    let walked = history.is_some_and(|state| state.landed.contains(job.as_str()));
    if landed_before || walked {
        return refuse(format!("job {} has already landed", job.as_str()));
    }
    Ok(None)
}

/// Why the records of a landed job do not show that its accepted reading `n` came before the work, if
/// they do not. The reading must be in the tree, and neither its time nor its invocation number may
/// come after the dispatch, a producer invocation, or a verifier invocation that the records list.
/// Nothing here depends on the history being kept: a squash holds the same records.
pub fn prediction_problem(files: &JobFiles, job: &JobId, n: u32) -> Option<String> {
    let dir = job.dir();
    let Some(reading) = files.readings.iter().find(|r| r.n == n && &r.job == job) else {
        return Some(format!("{dir}/reading-{n}.json is not in the landed tree"));
    };
    if let Some(dispatched_at) = files.job.as_ref().and_then(|j| j.dispatched_at)
        && reading.at > dispatched_at
    {
        return Some(format!(
            "{dir}/reading-{n}.json was recorded at {}, after the job was dispatched at {dispatched_at}",
            reading.at
        ));
    }
    for (path, inv) in &files.invocations {
        if inv.role == Role::Reader {
            continue;
        }
        if reading.at > inv.started_at || reading.invocation_seq >= inv.seq {
            return Some(format!(
                "{dir}/reading-{n}.json (recorded at {}, invocation {}) does not precede {path} (started at {}), \
                 so the prediction was not made before the work",
                reading.at, reading.invocation_seq, inv.started_at
            ));
        }
    }
    None
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

/// The interventions whose record `judged` adds, at `records/interventions/<month>/<id>.json`. This reads
/// the content only; [`check`] additionally requires an `Athena-Intervention` trailer (case b).
pub fn added_interventions(judged: &Judged) -> Vec<RecordId> {
    let mut found = Vec::new();
    for d in judged.diff.iter().filter(|d| d.status == 'A') {
        let mut parts = d.path.split('/');
        if let (Some("records"), Some("interventions"), Some(month), Some(file), None) = (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) && let Some(stem) = file.strip_suffix(".json")
            && let Ok(id) = RecordId::parse(stem)
            && id.month() == month
        {
            found.push(id);
        }
    }
    found
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
    fn added_interventions_lists_the_records_a_commit_adds_whatever_its_message_says() {
        let (dir, git) = owner_repo();
        let (path, text) = intervention_record("20261010-160000-abcd");
        // A record under the wrong month is not an intervention record at its canonical path.
        let wrong = "records/interventions/2026-01/20261010-160001-abcd.json";
        write(dir.path(), &[(&path, &text), (wrong, "{}")]);
        commit(&git, "no trailers at all\n");
        let id = git.rev_parse("HEAD").unwrap().unwrap();
        let change = Change::of_commit(&git, &id).unwrap();
        let judged = Judged::new(&git, &change).unwrap();
        assert_eq!(
            added_interventions(&judged),
            vec![RecordId::parse("20261010-160000-abcd").unwrap()]
        );
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
                .contains("job.json is not in the landed tree")
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
