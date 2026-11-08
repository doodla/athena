//! Retry and carry-over: the earlier attempt a "try again on" request builds on.

use super::*;

pub(super) const RETRY_PREFIX: &str = "try again on ";

/// Detects "try again on <job>" at the start of `request` and builds the earlier attempt from the
/// rejected, failed, timed out or stale job's request, final verification and (for a failed, stale or
/// unverified job) outcome reason. Returns `None` when the request is not that form.
/// The carried text goes through `redactor`.
pub fn earlier_attempt(
    records: &Records,
    request: &str,
    redactor: &Redactor,
) -> Result<Option<EarlierAttempt>> {
    earlier_attempt_with(records, request, redactor, &|_| false)
}

/// Like [`earlier_attempt`], with `running` saying whether a process still works on a job. A dispatched job
/// that no process works on was interrupted (for example by a restart) and can be tried again, though it
/// may have no verification or outcome record; one that is running is refused.
pub fn earlier_attempt_with(
    records: &Records,
    request: &str,
    redactor: &Redactor,
    running: &dyn Fn(&str) -> bool,
) -> Result<Option<EarlierAttempt>> {
    let Some(reference) = retry_reference(request)? else {
        return Ok(None);
    };
    let id = records.resolve(reference)?;
    let job = records.jobs.get(id.as_str()).expect("resolved");
    let state = job.state();
    let interrupted = state == JobState::Dispatched && !running(id.as_str());
    if state == JobState::Dispatched && !interrupted {
        bail!("job {id} is running; wait for it to finish before trying it again");
    }
    if !interrupted
        && !matches!(
            state,
            JobState::Closed(
                OutcomeState::Rejected
                    | OutcomeState::Failed
                    | OutcomeState::TimedOut
                    | OutcomeState::Stale
            )
        )
    {
        bail!(
            "job {id} is {}; only a rejected or failed job can be tried again (so can a timed out or stale one), or one that was interrupted",
            state.name()
        );
    }
    if job.request.is_none() {
        bail!("job {id} is {} but has no request record", state.name());
    }
    if !interrupted && job.verification.is_none() && job.outcome.is_none() {
        bail!(
            "job {id} is {} but has no verification or outcome record",
            state.name()
        );
    }
    let mut attempt = carry(job, redactor)?.expect("has a request");
    // When the retried job was itself a retry, carry the original request instead of the retry phrase,
    // and the corrections given to the jobs it retried (oldest first) ahead of its own.
    let earlier = earlier_jobs(records, job);
    if let Some(original) = earlier.last() {
        let mut texts = vec![original.request.as_ref().expect("checked").text.clone()];
        let inherited: Vec<&str> = earlier
            .iter()
            .rev()
            .flat_map(|j| j.corrections.iter().map(|c| c.text.as_str()))
            .collect();
        texts.extend(inherited.iter().map(|t| t.to_string()));
        redactor.redact_all(&mut texts)?;
        attempt.request = texts.remove(0);
        texts.append(&mut attempt.corrections);
        attempt.corrections = texts;
    }
    attempt.introspection = retry_cause(records, job, redactor)?;
    Ok(Some(attempt))
}

/// The job's own introspection of its rejection or failure that can seed a retry: the latest recorded
/// one with a proposed request. A proposal to relax or remove a gate never seeds a retry: it is for the
/// review Hestia queues, and a producer must not be invited to weaken the check it is judged by.
pub(crate) fn retry_introspection<'a>(
    records: &'a Records,
    job: &JobRecords,
) -> Option<&'a IntrospectionRecord> {
    records.introspections.iter().rev().find(|i| {
        i.job.as_ref() == Some(job.id())
            && matches!(
                i.trigger,
                IntrospectionTrigger::Rejection | IntrospectionTrigger::Failure
            )
            && i.status == IntrospectionStatus::Recorded
            && !i.prevention_kind.is_some_and(PreventionKind::loosens)
            && i.proposed_request.is_some()
            && i.cause.is_some()
            && i.prevention.is_some()
    })
}

/// The longest one-line account of an earlier case that is carried, in characters.
pub(super) const CASE_WHAT_CHARS: usize = 160;

/// The cause, prevention, proposed request and earlier cases from `job`'s introspection record, redacted;
/// `None` when the job has no such record. Earlier cases are the ids the record stored; classes are not
/// matched again.
pub(super) fn retry_cause(
    records: &Records,
    job: &JobRecords,
    redactor: &Redactor,
) -> Result<Option<RetryCause>> {
    let Some(record) = retry_introspection(records, job) else {
        return Ok(None);
    };
    let kind_and_text = |kind: Option<PreventionKind>, text: &str| match kind {
        Some(kind) => format!("{}: {text}", kind.as_str()),
        None => text.to_string(),
    };
    let class = record.failure_class.as_ref().map(|id| {
        let name = records
            .classes
            .iter()
            .find(|c| &c.id == id)
            .map_or("unknown class", |c| c.name.as_str());
        format!("{name} ({id})")
    });
    // Everything is redacted in one pass: the cause fields first, then each case's text.
    let mut texts = vec![
        record.cause.clone().unwrap_or_default(),
        kind_and_text(
            record.prevention_kind,
            record.prevention.as_deref().unwrap_or_default(),
        ),
        record.proposed_request.clone().unwrap_or_default(),
    ];
    let mut cases = Vec::new();
    for id in &record.earlier_cases {
        let Some(case) = records
            .introspections
            .iter()
            .find(|c| &c.id == id && !c.prevention_kind.is_some_and(PreventionKind::loosens))
        else {
            continue;
        };
        let what = case
            .what_happened
            .as_deref()
            .and_then(|w| w.lines().find(|l| !l.trim().is_empty()))
            .unwrap_or("(no account recorded)")
            .trim()
            .chars()
            .take(CASE_WHAT_CHARS)
            .collect::<String>();
        let prevention = case
            .prevention
            .as_deref()
            .map(|p| kind_and_text(case.prevention_kind, p));
        texts.push(what);
        texts.push(prevention.clone().unwrap_or_default());
        cases.push((case, prevention.is_some()));
    }
    redactor.redact_all(&mut texts)?;
    let mut texts = texts.into_iter();
    let cause = texts.next().expect("three leading texts");
    let prevention = texts.next().expect("three leading texts");
    let proposed_request = texts.next().expect("three leading texts");
    let earlier_cases = cases
        .into_iter()
        .map(|(case, has_prevention)| {
            let what = texts.next().expect("two texts per case");
            let text = texts.next().expect("two texts per case");
            EarlierCase {
                id: case.id.to_string(),
                job: case.job.as_ref().map(ToString::to_string),
                what,
                prevention: has_prevention.then_some(text),
            }
        })
        .collect();
    Ok(Some(RetryCause {
        introspection: record.id.to_string(),
        class,
        cause,
        prevention,
        proposed_request,
        earlier_cases,
    }))
}

/// Whether `job` has an introspection record that `athena retry` can seed a retry from.
pub fn has_retry_introspection(records: &Records, job: &JobRecords) -> bool {
    retry_introspection(records, job).is_some()
}

/// The job a request of the form "try again on <job>" retries; `None` when it is not that form or names no
/// job.
pub fn retried_job(records: &Records, request: &str) -> Option<JobId> {
    let reference = retry_reference(request).ok().flatten()?;
    records.resolve(reference).ok()
}

/// How each request job stands (landed; rejected, failed, timed out or stale; or any other state), with the
/// job its request retries ("try again on <job>", see [`retried_job`]). This is the history [`crate::plan::Plan::build_with`]
/// reads to tell a dependency that landed (or has a landed retry) from one that did not. A link to the job
/// itself is ignored.
pub fn job_history(records: &Records) -> Vec<JobEnd> {
    records
        .request_jobs()
        .into_iter()
        .map(|job| {
            let ending = match job.state() {
                JobState::Closed(OutcomeState::Landed) => Ending::Landed,
                JobState::Closed(
                    state @ (OutcomeState::Rejected
                    | OutcomeState::Failed
                    | OutcomeState::TimedOut
                    | OutcomeState::Stale),
                ) => Ending::Failed(state.as_str().replace('_', " ")),
                _ => Ending::Other,
            };
            let id = job.id().as_str().to_string();
            let retry_of = job
                .request
                .as_ref()
                .and_then(|r| retried_job(records, &r.text))
                .map(|j| j.as_str().to_string())
                .filter(|j| *j != id);
            JobEnd {
                id,
                ending,
                retry_of,
            }
        })
        .collect()
}

/// For each job that has a retry ("try again on <job>"), the newest retry that still stands for the work.
///
/// Built on [`job_history`], so the request form is read in one place ([`retried_job`]). A retry closed
/// without running (dropped, declined, abandoned, stale or closed by the bootstrap) does not stand: the
/// work it was meant to do is still undone, so it does not hide the retried job.
pub fn retries_by_job(records: &Records) -> BTreeMap<String, String> {
    retries_in(records, &job_history(records))
}

/// [`retries_by_job`] over a history the caller already built, so one pass of [`job_history`] can serve
/// several questions.
fn retries_in(records: &Records, history: &[JobEnd]) -> BTreeMap<String, String> {
    let mut retries = BTreeMap::new();
    // `job_history` lists the newest job first, so the first retry kept for a job is its newest.
    for end in history {
        let Some(retried) = end.retry_of.clone() else {
            continue;
        };
        let unrun = matches!(
            records.jobs.get(end.id.as_str()).map(JobRecords::state),
            Some(JobState::Closed(
                OutcomeState::Dropped
                    | OutcomeState::Declined
                    | OutcomeState::Abandoned
                    | OutcomeState::Stale
                    | OutcomeState::Closed
            ))
        );
        if !unrun {
            retries.entry(retried).or_insert_with(|| end.id.clone());
        }
    }
    retries
}

/// How many rejected, failed or timed-out attempts one retry lineage may have. At this many, Athena's drain
/// stops retrying the lineage and queues one narrower request instead (see [`lineage_stop`]).
pub const LINEAGE_ATTEMPT_LIMIT: usize = 3;

/// Every job of the retry lineage `job` belongs to, oldest first: the chain of "try again on <job>" requests
/// through `job`, from the first request that is not a retry to the newest retry that still stands (see
/// [`retries_in`]). Empty when `job` retries nothing and has no retry. A cycle or a chain longer than
/// [`MAX_RETRY_DEPTH`] ends the walk where it is.
fn lineage_chain(history: &[JobEnd], retries: &BTreeMap<String, String>, job: &str) -> Vec<String> {
    let back: BTreeMap<&str, &str> = history
        .iter()
        .filter_map(|e| Some((e.id.as_str(), e.retry_of.as_deref()?)))
        .collect();
    if !back.contains_key(job) && !retries.contains_key(job) {
        return Vec::new();
    }
    let mut seen = BTreeSet::from([job.to_string()]);
    let mut newest = job.to_string();
    for _ in 0..MAX_RETRY_DEPTH {
        let Some(next) = retries.get(&newest) else {
            break;
        };
        if !seen.insert(next.clone()) {
            break;
        }
        newest = next.clone();
    }
    let mut seen = BTreeSet::from([newest.clone()]);
    let mut chain = vec![newest.clone()];
    let mut current = newest;
    for _ in 0..MAX_RETRY_DEPTH {
        let Some(previous) = back.get(current.as_str()) else {
            break;
        };
        if !seen.insert((*previous).to_string()) {
            break;
        }
        chain.push((*previous).to_string());
        current = (*previous).to_string();
    }
    chain.reverse();
    chain
}

/// Whether `job` ended as an attempt that counts toward [`LINEAGE_ATTEMPT_LIMIT`]: rejected, failed or timed
/// out. A job that landed, was dropped, is stale or has not run does not count.
fn is_failed_attempt(records: &Records, job: &str) -> bool {
    matches!(
        records.jobs.get(job).map(JobRecords::state),
        Some(JobState::Closed(
            OutcomeState::Rejected | OutcomeState::Failed | OutcomeState::TimedOut
        ))
    )
}

/// The rejected, failed or timed-out attempts of the retry lineage `job` belongs to (see [`lineage_chain`]),
/// oldest first. Empty when `job` is in no lineage.
pub fn lineage_attempts(records: &Records, job: &str) -> Vec<JobId> {
    let history = job_history(records);
    let retries = retries_in(records, &history);
    lineage_chain(&history, &retries, job)
        .iter()
        .filter(|id| is_failed_attempt(records, id))
        .filter_map(|id| records.jobs.get(id.as_str()).map(|j| j.id().clone()))
        .collect()
}

/// Words that carry no meaning of their own in a finding's summary; [`same_summary`] ignores them so that
/// two short, different findings are not paired for the words they share.
const SUMMARY_STOP_WORDS: [&str; 27] = [
    "the", "a", "an", "and", "or", "is", "are", "was", "were", "be", "of", "to", "in", "on", "for",
    "no", "not", "it", "its", "this", "that", "with", "as", "by", "at", "has", "have",
];

/// The words of `text` that carry meaning: lower case, digits dropped, split on anything that is not a
/// letter, stop words removed.
fn content_words(text: &str) -> Vec<String> {
    crate::scope_guard::normalise_criterion(text)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty() && !SUMMARY_STOP_WORDS.contains(w))
        .map(String::from)
        .collect()
}

/// Whether two finding summaries say the same thing, across independent verifier runs: their content words
/// are equal, or one run of them contains the other (at least 2 words), or at least 2 content words are shared
/// and they are more than half of the shorter list.
pub(super) fn same_summary(a: &str, b: &str) -> bool {
    let (wa, wb) = (content_words(a), content_words(b));
    if wa.is_empty() || wb.is_empty() {
        return crate::scope_guard::normalise_criterion(a)
            == crate::scope_guard::normalise_criterion(b);
    }
    if wa == wb {
        return true;
    }
    let (short, long) = if wa.len() <= wb.len() {
        (&wa, &wb)
    } else {
        (&wb, &wa)
    };
    if short.len() >= 2 && long.windows(short.len()).any(|w| w == short.as_slice()) {
        return true;
    }
    let shared = short.iter().filter(|w| long.contains(w)).count();
    shared >= 2 && shared * 2 > short.len()
}

/// The file of a finding's location: a trailing `:line`, `:line-line` or `:line:col` and a `#anchor` removed.
pub(super) fn location_path(location: &str) -> &str {
    let mut path = location.trim();
    if let Some((head, _)) = path.split_once('#') {
        path = head;
    }
    for _ in 0..2 {
        match path.rsplit_once(':') {
            Some((head, tail))
                if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit() || c == '-') =>
            {
                path = head;
            }
            _ => break,
        }
    }
    path
}

/// Whether two unmet criteria are the same one: the scope guard's T4 comparison does not say they differ
/// (`Yes` or `Maybe`). Criteria of one lineage come from one acceptance list.
pub(super) fn same_criterion(a: &str, b: &str) -> bool {
    crate::scope_guard::compare_criteria(a, b) != crate::scope_guard::T4Match::No
}

/// Whether two blocking findings are the same one. Both located: the same file and the same summary
/// ([`same_summary`]). Either unlocated: the summaries must be equal after normalising (the T4 comparison
/// says `Yes`) or have the same content words, so punctuation does not hide a repeat but a partial overlap
/// is not enough.
pub(super) fn same_finding(a: (&str, Option<&str>), b: (&str, Option<&str>)) -> bool {
    match (a.1, b.1) {
        (Some(la), Some(lb)) => location_path(la) == location_path(lb) && same_summary(a.0, b.0),
        _ => {
            let (wa, wb) = (content_words(a.0), content_words(b.0));
            crate::scope_guard::compare_criteria(a.0, b.0) == crate::scope_guard::T4Match::Yes
                || (!wa.is_empty() && wa == wb)
        }
    }
}

/// One thing a failed attempt was held to account for: an unmet criterion or a blocking finding.
#[derive(Clone)]
struct Observed {
    criterion: bool,
    text: String,
    location: Option<String>,
}

impl Observed {
    fn same_as(&self, other: &Observed) -> bool {
        if self.criterion != other.criterion {
            return false;
        }
        if self.criterion {
            return same_criterion(&self.text, &other.text);
        }
        same_finding(
            (&self.text, self.location.as_deref()),
            (&other.text, other.location.as_deref()),
        )
    }

    /// The item on one line, as the replacement request words it.
    fn line(&self) -> String {
        let text = clip(&self.text, 200);
        match (&self.location, self.criterion) {
            (_, true) => format!("Unmet criterion: {text}"),
            (Some(location), false) => format!("Finding at {}: {text}", clip(location, 80)),
            (None, false) => format!("Finding: {text}"),
        }
    }

    /// The acceptance bullet that asks for the item to be put right.
    fn acceptance(&self) -> String {
        let text = clip(&self.text, 200);
        if self.criterion {
            format!("Meet: {text}")
        } else {
            format!("Resolve: {text}")
        }
    }
}

/// `text` on one line, cut to at most `max` characters.
fn clip(text: &str, max: usize) -> String {
    let one: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    one.chars().take(max).collect()
}

/// What a failed attempt was held to account for: its unmet criteria and blocking findings from the final
/// verification. An attempt with no verification contributes the first line of its closing reason.
fn observed_in(job: &JobRecords) -> Vec<Observed> {
    let Some(verification) = job.verification.as_ref() else {
        return job
            .outcome
            .as_ref()
            .and_then(|o| o.reason.lines().find(|l| !l.trim().is_empty()))
            .map(|line| Observed {
                criterion: false,
                text: line.trim().to_string(),
                location: None,
            })
            .into_iter()
            .collect();
    };
    let unmet = verification
        .criteria
        .iter()
        .filter(|c| !c.met)
        .map(|c| Observed {
            criterion: true,
            text: c.criterion.clone(),
            location: None,
        });
    let findings = verification
        .findings
        .iter()
        .filter(|f| crate::verify::finding_blocks(f))
        .map(|f| Observed {
            criterion: false,
            text: f.summary.clone(),
            location: f.location.clone(),
        });
    unmet.chain(findings).collect()
}

/// The items that repeated across the attempts (`attempts`, oldest first): those seen in at least 2 distinct
/// attempts, each with its wording from the newest attempt that had it. When nothing repeated, the blocking
/// items of the newest attempt, and `false`.
fn repeated_items(records: &Records, attempts: &[JobId]) -> (Vec<Observed>, bool) {
    let mut clusters: Vec<(Observed, BTreeSet<usize>)> = Vec::new();
    let mut last: Vec<Observed> = Vec::new();
    for (n, id) in attempts.iter().enumerate() {
        let items = records
            .jobs
            .get(id.as_str())
            .map(observed_in)
            .unwrap_or_default();
        for item in &items {
            match clusters.iter_mut().find(|(c, _)| c.same_as(item)) {
                Some((latest, seen)) => {
                    *latest = item.clone();
                    seen.insert(n);
                }
                None => clusters.push((item.clone(), BTreeSet::from([n]))),
            }
        }
        last = items;
    }
    let repeated: Vec<Observed> = clusters
        .into_iter()
        .filter(|(_, seen)| seen.len() >= 2)
        .map(|(item, _)| item)
        .collect();
    if repeated.is_empty() {
        (last, false)
    } else {
        (repeated, true)
    }
}

/// What the drain does in place of the next attempt of a stopped lineage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NarrowerAction {
    /// A narrower request for this lineage is already recorded: this job. Nothing more is queued.
    Queued(JobId),
    /// Queue a request with this text.
    Queue(String),
    /// No narrower request is made, for this reason; the lineage is still stopped and a person decides.
    Refused(String),
}

/// A retry lineage that has reached [`LINEAGE_ATTEMPT_LIMIT`] failed attempts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineageStop {
    /// The first request of the lineage.
    pub root: JobId,
    /// The rejected, failed or timed-out attempts, oldest first.
    pub attempts: Vec<JobId>,
    /// The findings that repeated, one line each (or the newest attempt's blocking findings when none did,
    /// see `repeated_found`).
    pub repeated: Vec<String>,
    /// Whether `repeated` holds findings seen in at least 2 attempts (`true`) or only the newest attempt's.
    pub repeated_found: bool,
    pub action: NarrowerAction,
}

/// The text that marks a request as the narrower request of the lineage rooted at `root`.
fn narrower_marker(root: &str) -> String {
    format!("Lineage {root}:")
}

/// The start of the title of every narrower request. A lineage whose root starts like this is not narrowed
/// again: the narrower request itself failed, and a person decides.
const NARROWER_TITLE: &str = "Re-scope or split ";

/// The narrower request for the lineage rooted at `root`, from the items that repeated.
fn narrower_request(
    records: &Records,
    root: &JobId,
    attempts: &[JobId],
    items: &[Observed],
    repeated_found: bool,
) -> Result<String, String> {
    let root_text = records
        .jobs
        .get(root.as_str())
        .and_then(|j| j.request.as_ref())
        .map_or("", |r| r.text.as_str());
    let root_bullets = request_text_acceptance(root_text).len();
    let room = if root_bullets > 1 {
        root_bullets - 1
    } else {
        1
    };
    let (kept, left) = items.split_at(items.len().min(room));
    let first_line = |text: &str| clip(text.lines().next().unwrap_or(""), 80);
    let subject = items
        .first()
        .map_or_else(|| first_line(root_text), |i| clip(&i.text, 80));
    let title = format!("{NARROWER_TITLE}{root}: {subject}");
    let ids: Vec<&str> = attempts.iter().map(JobId::as_str).collect();
    let mut body = format!(
        "{} failed attempts {}.\n",
        narrower_marker(root.as_str()),
        ids.join(", ")
    );
    body.push_str(&format!(
        "Athena stopped retrying this request after {} failed attempts and queued this narrower request in place of another retry. Do less than the original: meet only what is asked below, or split the rest into later requests.\n",
        attempts.len()
    ));
    body.push_str(if repeated_found {
        "Findings that repeated across the attempts:\n"
    } else {
        "No finding repeated across the attempts. Blocking findings of the last attempt:\n"
    });
    for item in kept {
        body.push_str(&item.line());
        body.push('\n');
    }
    if !left.is_empty() {
        body.push_str("Left for a later request: ");
        let rest: Vec<String> = left.iter().map(|i| clip(&i.text, 80)).collect();
        body.push_str(&rest.join("; "));
        body.push('\n');
    }
    let acceptance: Vec<String> = if kept.is_empty() {
        vec![format!(
            "Name the smallest part of {root} that can pass its checks, and make only that change"
        )]
    } else {
        kept.iter().map(Observed::acceptance).collect()
    };
    let mut found: Vec<String> = Vec::new();
    for item in items {
        for token in item
            .location
            .as_deref()
            .unwrap_or("")
            .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
        {
            let path = location_path(token);
            if looks_like_path(path) && !found.iter().any(|f| f == path) {
                found.push(path.to_string());
            }
        }
    }
    let wanted = request_text_touches(root_text);
    let files: Vec<String> = if wanted.is_empty() {
        if found.is_empty() {
            vec!["records/".to_string()]
        } else {
            found
        }
    } else {
        let overlap: Vec<String> = found
            .iter()
            .filter(|f| wanted.contains(f))
            .cloned()
            .collect();
        if overlap.is_empty() { wanted } else { overlap }
    };
    crate::steward::StewardRequest::new(None, &title, &body, &acceptance, &files)
        .map(|r| r.render())
}

/// The stop for each of `jobs` whose retry lineage has [`LINEAGE_ATTEMPT_LIMIT`] or more rejected, failed or
/// timed-out attempts, with the job it was asked for. One pass of [`job_history`] serves all of `jobs`, and
/// jobs of one lineage share one stop.
///
/// The stop holds the findings that repeated across the attempts and what to do in place of another retry:
/// the narrower re-scope or split request to queue, the one already queued for the lineage (found by the
/// `Lineage <root>:` line of its text), or a refusal when the lineage's own root is a narrower request.
pub fn lineage_stops(records: &Records, jobs: &[String]) -> Vec<(String, LineageStop)> {
    let history = job_history(records);
    let retries = retries_in(records, &history);
    let mut by_root: BTreeMap<String, Option<LineageStop>> = BTreeMap::new();
    let mut stops = Vec::new();
    for job in jobs {
        let chain = lineage_chain(&history, &retries, job);
        let Some(root) = chain.first().and_then(|r| records.jobs.get(r.as_str())) else {
            continue;
        };
        let root = root.id().clone();
        let attempts: Vec<JobId> = chain
            .iter()
            .filter(|id| is_failed_attempt(records, id))
            .filter_map(|id| records.jobs.get(id.as_str()).map(|j| j.id().clone()))
            .collect();
        if attempts.len() < LINEAGE_ATTEMPT_LIMIT {
            continue;
        }
        let key = format!("{root}:{}", attempts.len());
        let stop = by_root
            .entry(key)
            .or_insert_with(|| Some(stop_of(records, &root, attempts)))
            .clone();
        if let Some(stop) = stop {
            stops.push((job.clone(), stop));
        }
    }
    stops
}

/// [`lineage_stops`] for one job: `None` while its lineage has fewer than [`LINEAGE_ATTEMPT_LIMIT`] failed
/// attempts.
pub fn lineage_stop(records: &Records, job: &str) -> Option<LineageStop> {
    lineage_stops(records, &[job.to_string()])
        .pop()
        .map(|(_, stop)| stop)
}

fn stop_of(records: &Records, root: &JobId, attempts: Vec<JobId>) -> LineageStop {
    let (items, repeated_found) = repeated_items(records, &attempts);
    let marker = narrower_marker(root.as_str());
    let existing = records
        .request_jobs()
        .into_iter()
        .filter(|j| j.id() != root)
        .find(|j| j.request.as_ref().is_some_and(|r| r.text.contains(&marker)))
        .map(|j| j.id().clone());
    let root_text = records
        .jobs
        .get(root.as_str())
        .and_then(|j| j.request.as_ref())
        .map_or("", |r| r.text.as_str());
    let action = if let Some(job) = existing {
        NarrowerAction::Queued(job)
    } else if root_text.trim_start().starts_with(NARROWER_TITLE) {
        NarrowerAction::Refused(format!(
            "{root} is itself a narrower request, so it is not narrowed again"
        ))
    } else {
        match narrower_request(records, root, &attempts, &items, repeated_found) {
            Ok(text) => NarrowerAction::Queue(text),
            Err(why) => {
                NarrowerAction::Refused(format!("the narrower request could not be built: {why}"))
            }
        }
    };
    LineageStop {
        root: root.clone(),
        attempts,
        repeated: items.iter().map(Observed::line).collect(),
        repeated_found,
        action,
    }
}

/// What to do next about `retry`, a retry job that stands for an interrupted job (see [`retries_by_job`]),
/// worded to follow "retried as <job> (<state>);". `running` says whether a process works on the retry.
pub fn retry_follow_up(records: &Records, retry: &str, running: bool) -> String {
    let Some(job) = records.jobs.get(retry) else {
        return "see `athena status`".into();
    };
    match job.state() {
        JobState::Queued | JobState::Proposed | JobState::Blocked => {
            format!("run `athena go {retry}` to carry it out")
        }
        JobState::Declined => {
            format!("answer it with `athena correct {retry} \"<line>\"`")
        }
        JobState::Dispatched if running => "it is running; wait for it".into(),
        JobState::Dispatched => {
            format!("it was interrupted too; run `athena retry {retry}`")
        }
        JobState::Closed(
            OutcomeState::Rejected | OutcomeState::Failed | OutcomeState::TimedOut,
        ) => format!("it did not land; run `athena retry {retry}`"),
        JobState::Closed(OutcomeState::Landed) => "it landed; nothing is left to do".into(),
        JobState::Closed(_) => "see `athena status`".into(),
    }
}

/// The job a request of the form "try again on <job>" may resume from its kept candidate: one that failed for
/// an environment cause, or was interrupted (dispatched, and `running` says no process works on it; a job
/// held for low disk space after its candidate was built is one, and the drain retries it by itself). See
/// [`crate::disk::resumable_stop`]. Whether a candidate was kept is for the caller to find out.
pub fn resumable_job(
    records: &Records,
    request: &str,
    running: &dyn Fn(&str) -> bool,
) -> Option<JobId> {
    let id = retried_job(records, request)?;
    let job = records.jobs.get(id.as_str())?;
    let state = job.state();
    if state == JobState::Dispatched && running(id.as_str()) {
        return None;
    }
    let reason = job.outcome.as_ref().map(|o| o.reason.as_str());
    crate::disk::resumable_stop(state, reason).then_some(id)
}

/// Whether a retry of a job may start its producer from the job's rejected candidate instead of the base.
///
/// True only when all of these hold, read from the job's records and never from prose:
/// - the job was rejected (not failed, interrupted, blocked, landed or dropped);
/// - it has a candidate with a commit and a tree and no admission violations;
/// - its verification recorded at least one check, every check passed, and the verifier found no mismatch
///   between the request and the reading;
/// - no behavior criterion is unmet or cites a command that was not seen running; and
/// - either at least one criterion is unmet and every unmet criterion is of kind content, or no criterion is
///   unmet, at least one finding is a blocker or major, and every such blocking finding cites only non-code
///   files (Markdown, a CHANGELOG, or anything under `docs/`; see
///   [`finding_is_content_only`](crate::verify::finding_is_content_only)). Minor and nit findings are ignored.
///
/// Otherwise a rejection that names no unmet criterion and no blocking finding has an unknown cause, so it
/// does not qualify, and a blocking finding that cites a code file (or no recognizable path) restarts from
/// the base. The new candidate is verified in full either way.
pub fn qualifies_for_tree_retry(
    state: JobState,
    candidate: Option<&athena_data::records::Candidate>,
    verification: Option<&athena_data::records::Verification>,
) -> bool {
    if state != JobState::Closed(OutcomeState::Rejected) {
        return false;
    }
    let (Some(candidate), Some(verification)) = (candidate, verification) else {
        return false;
    };
    if candidate.candidate_commit.is_none()
        || candidate.candidate_tree.is_none()
        || !candidate.violations.is_empty()
    {
        return false;
    }
    if verification.checks.is_empty()
        || !verification.checks.iter().all(|c| c.passed)
        || verification.request_reading_mismatch.is_some()
    {
        return false;
    }
    let behavior_trouble = verification
        .criteria
        .iter()
        .any(|c| c.kind == CriterionKind::Behavior && (!c.met || !c.commands_verified));
    if behavior_trouble {
        return false;
    }
    let mut unmet = verification.criteria.iter().filter(|c| !c.met).peekable();
    if unmet.peek().is_some() {
        return unmet.all(|c| c.kind == CriterionKind::Content);
    }
    let mut blocking = verification
        .findings
        .iter()
        .filter(|f| crate::verify::finding_blocks(f))
        .peekable();
    blocking.peek().is_some() && blocking.all(crate::verify::finding_is_content_only)
}

/// What a producer retry seeded from a rejected candidate starts from and is told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeRetrySeed {
    /// The rejected job.
    pub job: JobId,
    /// The base commit the rejected job's candidate was made against.
    pub base_commit: String,
    /// The rejected candidate's commit, whose visible files seed the working directory.
    pub candidate_commit: String,
    /// The unmet criteria with the verifier's evidence, redacted.
    pub unmet: Vec<String>,
    /// The verification's findings, redacted.
    pub findings: Vec<String>,
    /// The blocker and major findings among `findings`: those the retry must address.
    pub must_address: Vec<String>,
}

/// The seed for a request of the form "try again on <job>" when [`qualifies_for_tree_retry`] holds for that
/// job; `None` for any other request or job. The caller must still check that the visible base has not
/// changed since `base_commit` before it seeds from the candidate.
pub fn tree_retry_seed(
    records: &Records,
    request: &str,
    redactor: &Redactor,
) -> Result<Option<TreeRetrySeed>> {
    let Some(id) = retried_job(records, request) else {
        return Ok(None);
    };
    let Some(job) = records.jobs.get(id.as_str()) else {
        return Ok(None);
    };
    if !qualifies_for_tree_retry(
        job.state(),
        job.candidate.as_ref(),
        job.verification.as_ref(),
    ) {
        return Ok(None);
    }
    let (Some(candidate), Some(verification)) = (&job.candidate, &job.verification) else {
        return Ok(None);
    };
    let Some(candidate_commit) = candidate.candidate_commit.clone() else {
        return Ok(None);
    };
    let mut texts: Vec<String> = verification
        .criteria
        .iter()
        .filter(|c| !c.met)
        .map(|c| format!("{}: {}", c.criterion, c.evidence))
        .collect();
    let unmet_count = texts.len();
    texts.extend(verification.findings.iter().map(finding_text));
    redactor.redact_all(&mut texts)?;
    let findings = texts.split_off(unmet_count);
    // The blocking findings are picked from the redacted list by position, so both retry paths show the
    // same text for the same finding.
    let must_address = verification
        .findings
        .iter()
        .zip(&findings)
        .filter(|(f, _)| crate::verify::finding_blocks(f))
        .map(|(_, text)| text.clone())
        .collect();
    Ok(Some(TreeRetrySeed {
        job: id,
        base_commit: candidate.base_commit.clone(),
        candidate_commit,
        unmet: texts,
        findings,
        must_address,
    }))
}

/// One finding as a retry brief shows it: severity, location, summary and evidence.
pub(super) fn finding_text(f: &Finding) -> String {
    let location = f
        .location
        .as_deref()
        .map_or(String::new(), |l| format!(" ({l})"));
    format!(
        "{}{location}: {} {}",
        format!("{:?}", f.severity).to_lowercase(),
        f.summary,
        f.evidence
    )
    .trim_end()
    .to_string()
}

/// The start of the closing reason of a job whose candidate still changed files outside the job's allowed
/// paths when the check before verification ran. The reason names each file and the criterion that limits
/// the files.
pub(crate) const SCOPE_REASON_PREFIX: &str = "files outside the allowed paths:";

/// The job a request "try again on <job>" retries, and every blocker and major finding of that job's final
/// verification, redacted, in the form [`tree_retry_seed`] uses. Findings have no resolved state, so none is
/// left out because a later round might have fixed it. `None` when the request is not a retry, the job has
/// no verification, or no finding blocks.
///
/// Two kinds of job have no verification and still have a finding: a job the decision check closed before
/// the verifier started ([`athena_data::decision_links::GATE_REASON_PREFIX`]), and a job the allowed-paths
/// check closed there ([`SCOPE_REASON_PREFIX`]). Its closing reason names the decisions the change left
/// unreconciled, or the files outside the allowed paths, and the fix, so that reason, redacted, is the one
/// finding.
pub fn retry_findings(
    records: &Records,
    request: &str,
    redactor: &Redactor,
) -> Result<Option<(JobId, Vec<String>)>> {
    let Some(id) = retried_job(records, request) else {
        return Ok(None);
    };
    let job = records.jobs.get(id.as_str());
    let Some(verification) = job.and_then(|j| j.verification.as_ref()) else {
        let Some(outcome) = job
            .and_then(|j| j.outcome.as_ref())
            .filter(|o| o.state == OutcomeState::Failed)
            .filter(|o| {
                o.reason
                    .starts_with(athena_data::decision_links::GATE_REASON_PREFIX)
                    || o.reason.starts_with(SCOPE_REASON_PREFIX)
            })
        else {
            return Ok(None);
        };
        let mut texts = vec![outcome.reason.clone()];
        redactor.redact_all(&mut texts)?;
        return Ok(Some((id, texts)));
    };
    let mut texts: Vec<String> = verification
        .findings
        .iter()
        .filter(|f| crate::verify::finding_blocks(f))
        .map(finding_text)
        .collect();
    if texts.is_empty() {
        return Ok(None);
    }
    redactor.redact_all(&mut texts)?;
    Ok(Some((id, texts)))
}

/// The kept plan of a job that stopped at the planning round cap, and the critic's findings on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanningResumeSeed {
    /// The job that stopped at the cap.
    pub job: JobId,
    /// Its last plan, from its `job-plan-<round>.json`.
    pub plan: athena_data::records::JobPlan,
    /// Every finding of the critique of that plan, from its `critique-<round>.json`.
    pub findings: Vec<Finding>,
}

/// The seed for a request of the form "try again on <job>" when that job failed in planning because its plan
/// still needed revising in the last round; `None` for any other request, or a job that stopped for any other
/// reason (a reject, a misread reading, a timeout, a failed verification, an agent failure). The plan and
/// findings were redacted when they were recorded.
pub fn planning_resume_seed(records: &Records, request: &str) -> Option<PlanningResumeSeed> {
    let id = retried_job(records, request)?;
    let job = records.jobs.get(id.as_str())?;
    let outcome = job.outcome.as_ref()?;
    if outcome.state != OutcomeState::Failed {
        return None;
    }
    let why = outcome
        .reason
        .strip_prefix(athena_data::records::PLANNING_STOPPED_PREFIX)?;
    if !why
        .trim_start()
        .starts_with(athena_data::records::PLANNING_CAP_REASON)
    {
        return None;
    }
    let plan = job.plans.iter().max_by_key(|p| p.round)?;
    let critique = job.critiques.iter().find(|c| c.round == plan.round)?;
    if critique.final_verdict != CritiqueVerdict::Revise
        || critique.request_reading_mismatch.is_some()
    {
        return None;
    }
    Some(PlanningResumeSeed {
        job: id,
        plan: plan.clone(),
        findings: critique.findings.clone(),
    })
}

/// The reference after "try again on " at the start of `request`; `None` when it is not that form.
pub(super) fn retry_reference(request: &str) -> Result<Option<&str>> {
    let text = request.trim();
    if text.eq_ignore_ascii_case(RETRY_PREFIX.trim_end()) {
        bail!("say which job to try again on: 'try again on <job>'");
    }
    let Some(head) = text.get(..RETRY_PREFIX.len()) else {
        return Ok(None);
    };
    if !head.eq_ignore_ascii_case(RETRY_PREFIX) {
        return Ok(None);
    }
    let reference = text[RETRY_PREFIX.len()..]
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_end_matches(['.', ',', ';', ':', '!', '?']);
    if reference.is_empty() {
        bail!("say which job to try again on: 'try again on <job>'");
    }
    Ok(Some(reference))
}

/// The most retries followed back to an original request.
pub(super) const MAX_RETRY_DEPTH: usize = 16;

/// The jobs `job` is a retry of, nearest first, back to the first request that is not a retry. Empty
/// when `job` is not a retry; when the chain breaks (unknown job, no request, a cycle, or too deep),
/// the jobs reached so far are returned.
pub(super) fn earlier_jobs<'a>(records: &'a Records, job: &'a JobRecords) -> Vec<&'a JobRecords> {
    let mut current = job;
    let mut seen = vec![current.id().as_str().to_string()];
    let mut chain = Vec::new();
    for _ in 0..MAX_RETRY_DEPTH {
        let Some(request) = current.request.as_ref() else {
            break;
        };
        let Some(reference) = retry_reference(&request.text).ok().flatten() else {
            break;
        };
        let Some(next) = records
            .resolve(reference)
            .ok()
            .and_then(|id| records.jobs.get(id.as_str()))
        else {
            break;
        };
        if next.request.is_none() || seen.iter().any(|s| s == next.id().as_str()) {
            break;
        }
        seen.push(next.id().as_str().to_string());
        chain.push(next);
        current = next;
    }
    chain
}

/// The most bytes of one failed check's output that are carried; the end of the output is kept.
pub(super) const CARRIED_CHECK_OUTPUT_BYTES: usize = 1024;

/// The most changed files of one earlier job that are carried; the rest are counted, not named.
pub(super) const CARRIED_CHANGED_FILES: usize = 30;

/// The most characters of the producer's account that are carried; the start is kept.
pub(super) const CARRIED_APPROACH_CHARS: usize = 800;

/// The most measured-check lines that are carried.
pub(super) const CARRIED_CHECK_LINES: usize = 20;

/// The parts of [`EarlierAttempt`] that say what the earlier job did.
pub(super) struct CarriedWork {
    changed_files: Vec<String>,
    changed_files_omitted: usize,
    approach: Option<String>,
    checks: Vec<String>,
}

/// What the earlier job did, from the records it already has: the files its candidate changed (`status
/// path`, at most [`CARRIED_CHANGED_FILES`]), the producer's own summary (at most
/// [`CARRIED_APPROACH_CHARS`] characters) and one line per measured check (at most
/// [`CARRIED_CHECK_LINES`], no output text). All of it is redacted. A file whose name redaction altered is
/// dropped from the list and counted in the returned number, with the files past the cap, so no reading is
/// ever asked to repeat a redaction marker.
pub(super) fn carried_work(job: &JobRecords, redactor: &Redactor) -> Result<CarriedWork> {
    use athena_data::records::ChangeStatus;
    let raw_files: Vec<String> = job
        .candidate
        .iter()
        .flat_map(|c| &c.changes)
        .map(|c| {
            let status = match c.status {
                ChangeStatus::Added => "added",
                ChangeStatus::Modified => "modified",
                ChangeStatus::Deleted => "deleted",
                ChangeStatus::TypeChanged => "type-changed",
            };
            format!("{status} {}", c.path)
        })
        .collect();
    let summary = job
        .candidate
        .as_ref()
        .and_then(|c| c.producer_report.as_ref())
        .map(|r| r.summary.trim().to_string())
        .filter(|s| !s.is_empty());
    let checks: Vec<String> = job
        .verification
        .iter()
        .flat_map(|v| &v.checks)
        .take(CARRIED_CHECK_LINES)
        .map(|c| {
            let exit = c.exit_code.map_or("no exit code".to_string(), |code| {
                format!("exit code {code}")
            });
            let tests = c
                .tests_ran
                .map_or(String::new(), |n| format!(", {n} tests ran"));
            // A revert run is Athena's "fails without the fix" demonstration: a nonzero exit is the
            // expected result, so it is neither passed nor failed.
            if !c.revert_for.is_empty() {
                return format!("{}: revert run ({exit}{tests})", c.name);
            }
            let result = if c.passed { "passed" } else { "failed" };
            format!("{}: {result} ({exit}{tests})", c.name)
        })
        .collect();
    // One redaction pass over the files, the summary and the check lines, so a secret that spans them is
    // found; the summary is shortened after redaction so a cut never leaves half a secret.
    let files = raw_files.len();
    let mut texts = raw_files.clone();
    texts.extend(summary.clone());
    texts.extend(checks);
    redactor.redact_all(&mut texts)?;
    let mut rest = texts.split_off(files);
    let approach = summary.map(|_| rest.remove(0));
    let approach = approach.map(|s| crate::truncate_at_word(&s, CARRIED_APPROACH_CHARS));
    let kept: Vec<String> = texts
        .into_iter()
        .zip(&raw_files)
        .filter(|(redacted, raw)| redacted == *raw)
        .map(|(_, raw)| raw.clone())
        .take(CARRIED_CHANGED_FILES)
        .collect();
    let omitted = files - kept.len();
    Ok(CarriedWork {
        changed_files: kept,
        changed_files_omitted: omitted,
        approach,
        checks: rest,
    })
}

/// The job's request, final verification findings and reasons, and each failed check's name, command,
/// exit code and output tail, redacted; the files its candidate changed, the producer's account and the
/// measured checks (see [`carried_work`]). `None` without a request.
pub(super) fn carry(job: &JobRecords, redactor: &Redactor) -> Result<Option<EarlierAttempt>> {
    let Some(earlier) = &job.request else {
        return Ok(None);
    };
    let mut texts = vec![earlier.text.clone()];
    if let Some(verification) = &job.verification {
        texts.extend(verification.findings.iter().map(finding_text));
        texts.extend(verification.reasons.iter().map(|r| format!("reason: {r}")));
        texts.extend(verification.checks.iter().filter(|c| !c.passed).map(|c| {
            let exit = c.exit_code.map_or("no exit code".to_string(), |code| {
                format!("exit code {code}")
            });
            format!(
                "failed check {} (`{}`, {exit}), output:\n{}",
                c.name,
                c.command.join(" "),
                tail(&c.output_tail, CARRIED_CHECK_OUTPUT_BYTES)
            )
        }));
    }
    // A job stopped in planning has no verification: the findings of the critique that stopped it say what
    // was wrong with the plan, so a retry carries them and not only the outcome reason (review F11).
    if let Some(critique) = job
        .critiques
        .last()
        .filter(|c| c.final_verdict != CritiqueVerdict::Accept)
    {
        if let Some(how) = &critique.request_reading_mismatch {
            texts.push(format!(
                "plan critique, round {}: the reading misread the request: {how}",
                critique.round
            ));
        }
        texts.extend(critique.findings.iter().map(|f| {
            let location = f
                .location
                .as_deref()
                .map_or(String::new(), |l| format!(" ({l})"));
            format!(
                "plan critique, round {}, {}{location}: {} {}",
                critique.round,
                format!("{:?}", f.severity).to_lowercase(),
                f.summary,
                f.evidence
            )
            .trim_end()
            .to_string()
        }));
    }
    // A job closed before verification has only its outcome reason to say why; a failed job's reason
    // (for example an admission refusal) is carried even when it was verified.
    let failed = job
        .outcome
        .as_ref()
        .is_some_and(|o| matches!(o.state, OutcomeState::Failed | OutcomeState::TimedOut));
    // A stale job was verified and then could not land, so its reason (what moved on the branch) is
    // carried too.
    let stale = job
        .outcome
        .as_ref()
        .is_some_and(|o| o.state == OutcomeState::Stale);
    if let Some(outcome) = &job.outcome
        && (failed || stale || job.verification.is_none())
        && !outcome.reason.trim().is_empty()
    {
        texts.push(format!("outcome: {}", outcome.reason));
    }
    redactor.redact_all(&mut texts)?;
    let request = texts.remove(0);
    // The final reading and the user's corrections and answers, redacted like the rest.
    let final_reading = job.latest_reading();
    let criteria = final_reading.map_or(0, |r| r.acceptance_criteria.len());
    let mut extra: Vec<String> = Vec::new();
    if let Some(reading) = final_reading {
        extra.push(reading.summary.clone());
        extra.extend(reading.acceptance_criteria.iter().map(|c| {
            let kind = match c.kind {
                CriterionKind::Content => "content",
                CriterionKind::Behavior => "behavior",
            };
            format!("[{kind}] {}", c.text)
        }));
        extra.extend(reading.deliverables.iter().map(|d| {
            let action = match d.action {
                DeliverableAction::Create => "create",
                DeliverableAction::Modify => "modify",
                DeliverableAction::Delete => "delete",
            };
            format!("{action} {}", d.path)
        }));
        extra.extend(reading.out_of_scope.iter().cloned());
    }
    let deliverable_count = final_reading.map_or(0, |r| r.deliverables.len());
    let lead = extra.len();
    extra.extend(job.corrections.iter().map(|c| c.text.clone()));
    redactor.redact_all(&mut extra)?;
    let corrections = extra.split_off(lead);
    let mut extra = extra.into_iter();
    let reading_summary = final_reading.and_then(|_| extra.next());
    let reading_criteria: Vec<String> = extra.by_ref().take(criteria).collect();
    let work = carried_work(job, redactor)?;
    Ok(Some(EarlierAttempt {
        introspection: None,
        job: job.id().as_str().to_string(),
        request,
        failed,
        findings: texts,
        reading_summary,
        reading_criteria,
        reading_deliverables: extra.by_ref().take(deliverable_count).collect(),
        reading_out_of_scope: extra.collect(),
        end_state: job.outcome.as_ref().map_or_else(
            || job.state().name().to_string(),
            |o| o.state.as_str().to_string(),
        ),
        verdict: job
            .verification
            .as_ref()
            .map(|v| format!("{:?}", v.final_verdict).to_lowercase()),
        corrections,
        changed_files: work.changed_files,
        changed_files_omitted: work.changed_files_omitted,
        approach: work.approach,
        checks: work.checks,
    }))
}
