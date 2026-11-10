//! Brief topics, referenced jobs, request-text helpers and the plan entry.

use super::*;

/// The most jobs a request may reference; more would bloat the reader brief.
pub const MAX_REFERENCED_JOBS: usize = 3;

/// Full job ids (`YYYYMMDD-HHMMSS-xxxx`) named anywhere in `text`, in order, without repeats.
pub fn job_ids_in(text: &str) -> Vec<RecordId> {
    let mut ids: Vec<RecordId> = Vec::new();
    for word in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
        if let Ok(id) = RecordId::parse(word)
            && !ids.contains(&id)
        {
            ids.push(id);
        }
    }
    ids
}

/// The jobs a request depends on, in order, without repeats: the retry link of a "try again on <job>"
/// request, then every full job id on a line that starts with "Depends on" (any case; a quote mark `>`, a
/// list marker, bold marks or a number may come first, and a colon may follow). A job id anywhere else in the
/// text is a reference, not a dependency: [`job_ids_in`] still finds it for the reader.
pub fn dependency_ids_in(text: &str) -> Vec<RecordId> {
    const KEYWORD: &str = "depends on";
    let mut ids: Vec<RecordId> = Vec::new();
    let mut add = |id: RecordId| {
        if !ids.contains(&id) {
            ids.push(id);
        }
    };
    if let Some(id) = retry_reference(text)
        .ok()
        .flatten()
        .and_then(|reference| RecordId::parse(reference).ok())
    {
        add(id);
    }
    for line in text.lines() {
        let line = line.trim_start_matches(|c: char| c.is_whitespace() || "> -*+".contains(c));
        let digits = line.chars().take_while(char::is_ascii_digit).count();
        let line = match line[digits..].strip_prefix(['.', ')']) {
            Some(rest) if digits > 0 => rest.trim_start(),
            _ => line,
        };
        let Some(head) = line.get(..KEYWORD.len()) else {
            continue;
        };
        let rest = &line[KEYWORD.len()..];
        if head.eq_ignore_ascii_case(KEYWORD)
            && !rest.starts_with(|c: char| c.is_ascii_alphanumeric())
        {
            for id in job_ids_in(rest) {
                add(id);
            }
        }
    }
    ids
}

/// Whether a word of a request looks like a repository path: it has a directory part or a file extension.
pub(crate) fn looks_like_path(word: &str) -> bool {
    let w = word.trim_matches(|c: char| !(c.is_ascii_alphanumeric() || "/._-".contains(c)));
    !w.is_empty()
        && !w.starts_with('/')
        && !w.contains("..")
        && (w.contains('/')
            || [".rs", ".md", ".toml", ".json", ".yml"]
                .iter()
                .any(|ext| w.ends_with(ext)))
}

/// What planning looks up in the repository and its records, so `plan_entry` stays free of I/O.
pub struct Lookup<'a> {
    /// Whether a job with this full id is among the records.
    pub job_exists: &'a dyn Fn(&str) -> bool,
    /// Whether this path (a directory ends with `/`) exists at the base commit.
    pub path_exists: &'a dyn Fn(&str) -> bool,
    /// What BRIEF.md says the owner wants (see [`brief_topics`]); empty when there is no brief.
    pub brief: &'a [BriefTopic],
}

impl std::fmt::Debug for Lookup<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lookup").finish_non_exhaustive()
    }
}

/// One section of BRIEF.md that a request can serve: a requirement or an owner addendum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BriefTopic {
    /// The `###` heading.
    pub heading: String,
    /// The section's body. Kept only for addenda; a requirement is matched by its heading alone, because its
    /// long body would match nearly any request.
    pub text: String,
    /// The priority the owner gave the section: from a `Priority: <level>` line in an addendum's body, or
    /// high when the body says its work ranks above all other work or is a top priority. Normal otherwise.
    pub priority: Priority,
}

/// The priority an addendum's body states (see [`BriefTopic::priority`]).
pub(super) fn addendum_priority(body: &str) -> Priority {
    let stated = Priority::from_text(body);
    if stated != Priority::Normal {
        return stated;
    }
    let lower = body.to_lowercase();
    if [
        "ranks above all other work",
        "top priority",
        "highest priority",
    ]
    .iter()
    .any(|p| lower.contains(p))
    {
        Priority::High
    } else {
        Priority::Normal
    }
}

/// The sections of `brief` (the text of BRIEF.md) that a request can serve: every `###` heading, and for those
/// under `## Addenda` the body too. An addendum that states a priority carries it, so planning can rank the
/// work that serves it ahead of default order.
pub fn brief_topics(brief: &str) -> Vec<BriefTopic> {
    let mut topics: Vec<BriefTopic> = Vec::new();
    let mut in_addenda = false;
    let mut in_topic = false;
    for line in brief.lines() {
        if let Some(h) = line.strip_prefix("### ") {
            topics.push(BriefTopic {
                heading: h.trim().to_string(),
                text: String::new(),
                priority: Priority::Normal,
            });
            in_topic = true;
        } else if line.starts_with('#') {
            in_addenda = line.trim_end() == "## Addenda";
            in_topic = false;
        } else if in_topic
            && in_addenda
            && let Some(t) = topics.last_mut()
        {
            t.text.push_str(line);
            t.text.push('\n');
        }
    }
    for t in &mut topics {
        t.priority = addendum_priority(&t.text);
    }
    topics
}

/// Words too common in BRIEF.md to say which section a request serves.
pub(super) const COMMON_WORDS: [&str; 30] = [
    "that", "this", "with", "from", "have", "they", "them", "will", "should", "would", "could",
    "about", "which", "there", "their", "athena", "system", "what", "when", "also", "into", "make",
    "work", "then", "than", "each", "only", "some", "just", "need",
];

/// The words of `text` that can tie a request to a brief section: lowercase, a trailing `s` dropped, at least
/// four letters (or `ci`), and neither common nor one of [`IMPLEMENTATION_WORDS`].
pub(super) fn topic_words(text: &str) -> std::collections::BTreeSet<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .map(|w| w.strip_suffix('s').unwrap_or(w))
        .filter(|w| {
            (w.len() >= 4 || *w == "ci")
                && !w.chars().all(|c| c.is_ascii_digit())
                && !COMMON_WORDS.contains(w)
                && !IMPLEMENTATION_WORDS.contains(w)
        })
        .map(String::from)
        .collect()
}

/// The topic of `topics` that `hay` shares the most words with, when it shares at least two.
pub(super) fn serves_topic<'t>(hay: &str, topics: &'t [BriefTopic]) -> Option<&'t BriefTopic> {
    let words = topic_words(hay);
    topics
        .iter()
        .map(|t| {
            let theirs = topic_words(&format!("{} {}", t.heading, t.text));
            (words.intersection(&theirs).count(), t)
        })
        .filter(|(shared, _)| *shared >= 2)
        .max_by_key(|(shared, _)| *shared)
        .map(|(_, t)| t)
}

/// Words that say a request is about code, tests or docs, not which requirement it serves. They are never a
/// rung on their own: the rung comes from what the request serves in BRIEF.md and its addenda. They are
/// ignored when a request is matched to a brief section, since they are everywhere in a brief about software.
pub(super) const IMPLEMENTATION_WORDS: [&str; 18] = [
    "readme",
    "changelog",
    "docs",
    "test",
    "tests",
    "code",
    "bug",
    "fix",
    "typo",
    "refactor",
    "rename",
    "function",
    "command",
    "crate",
    "module",
    "clippy",
    "lint",
    "cli",
];

/// The rung a request serves, found from what it serves in BRIEF.md and the owner's addenda; `None` when
/// nothing says. Deterministic and cheap, and a heuristic, in this order:
///
/// 1. A phrase naming a higher rung (an owner goal, a brief change, a design decision, a user default).
/// 2. The request or its reading shares at least two significant words with a section of `brief` (a
///    requirement heading or an addendum, see [`brief_topics`]): it serves the brief, so
///    BriefRequirements. This is how the owner's latest priorities rank above plain implementation work.
/// 3. A touched path or a reading for the `change` capability: Implementation, as plain work that serves no
///    particular brief section.
///
/// The [`IMPLEMENTATION_WORDS`] are not evidence. It is never the answer by default.
pub fn infer_rung(
    text: &str,
    reading: Option<&Reading>,
    touches: &[String],
    brief: &[BriefTopic],
) -> Option<athena_data::roles::Ladder> {
    use athena_data::roles::Ladder;
    let mut hay = text.to_lowercase();
    if let Some(r) = reading {
        hay.push(' ');
        hay.push_str(&r.summary.to_lowercase());
    }
    let has = |phrases: &[&str]| phrases.iter().any(|p| hay.contains(p));
    if has(&["owner goal", "owner's goal"]) {
        Some(Ladder::OwnerGoals)
    } else if has(&[
        "brief requirement",
        "change the brief",
        "amend the brief",
        "update the brief",
    ]) {
        Some(Ladder::BriefRequirements)
    } else if has(&["design decision", "record a decision", "new decision"]) {
        Some(Ladder::DesignDecisions)
    } else if has(&["user default", "defaults-conventions"]) {
        Some(Ladder::DefaultsConventions)
    } else if serves_topic(&hay, brief).is_some() {
        Some(Ladder::BriefRequirements)
    } else if !touches.is_empty()
        || reading.is_some_and(|r| r.capability.as_deref() == Some("change"))
    {
        Some(Ladder::Implementation)
    } else {
        None
    }
}

/// Whether a line of a request is an explicit acceptance bullet: it starts with `-`, `*` or a number and a dot.
pub(super) fn is_bullet(line: &str) -> bool {
    line.starts_with("- ")
        || line.starts_with("* ")
        || line
            .split_once(". ")
            .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// The acceptance of a request that has not been read: its lines that start with `-`, `*` or a number and a
/// dot. This is the parse `athena plan` uses.
pub fn request_text_acceptance(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| is_bullet(l))
        .map(String::from)
        .collect()
}

/// One banned boundary word found in a request's own text, before the job is admitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputFinding {
    /// Always `request`: only the request's own text is scanned at admission.
    pub source: String,
    pub line: usize,
    pub term: String,
}

impl std::fmt::Display for InputFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.source, self.line, self.term)
    }
}

/// The boundary scan (the same one `cargo test -p xtask --test policy` runs, from the same word lists) over
/// the request's own text only. Files the request names are not read here: the owner's brief legitimately
/// uses boundary words, and a file is scanned when a candidate changes it, not when it is named.
pub fn scan_request_input(text: &str) -> Vec<InputFinding> {
    athena_context::briefs::boundary_scan(text)
        .into_iter()
        .map(|hit| InputFinding {
            source: "request".to_string(),
            line: hit.line,
            term: hit.term,
        })
        .collect()
}

/// Refuses a request whose own text holds a banned boundary word, so the words are reworded before a job
/// exists. Files the request names are not scanned. Nothing is edited: the message lists each source, line
/// and word.
pub(super) fn refuse_banned_input(text: &str) -> Result<()> {
    let findings = scan_request_input(text);
    if findings.is_empty() {
        return Ok(());
    }
    let list: Vec<String> = findings.iter().map(|f| format!("  {f}")).collect();
    bail!(
        "the request was not admitted: its own text holds words Athena's boundary scan rejects \
         (source:line: word). Reword them and ask again:\n{}",
        list.join("\n")
    );
}

/// What a request that has not been read touches: its path-like words, without repeats. This is the parse
/// `athena plan` uses.
pub fn request_text_touches(text: &str) -> Vec<String> {
    let mut touches: Vec<String> = Vec::new();
    for word in text.split_whitespace().filter(|w| looks_like_path(w)) {
        let path = word
            .trim_matches(|c: char| !(c.is_ascii_alphanumeric() || "/._-".contains(c)))
            .to_string();
        if !touches.contains(&path) {
            touches.push(path);
        }
    }
    touches
}

/// Planning, the first stage: fleshes out one open request into a plan entry (design: planning first).
/// Deterministic code; it calls no model and fills nothing in by default.
///
/// - Once the request has been read, the goal is the reading's summary, the acceptance is its criteria (only
///   when the reading chose a capability: a declined reading has no acceptance), the open question is its
///   question and what it touches is its deliverables.
/// - Before it is read, the request's own words stand in: the first line is the goal; only the lines that
///   start with `-`, `*` or a number are the acceptance (the first line is **not** copied into it); path-like
///   words are what it touches.
/// - The rung is `rung_hint` when the caller knows it (a steward proposal carries it), else [`infer_rung`]
///   from what the request and the reading serve in `lookup.brief`, else `None`, which the plan flags.
/// - A request depends only on the jobs [`dependency_ids_in`] finds: a full job id on a line that starts with
///   "Depends on", or the retry link of a "try again on <job>" request. A job id anywhere else in the text
///   ("after <job>", evidence it cites) is a reference only: the reader is shown it, but it never holds the
///   item. A request never depends on itself.
/// - Open questions are also filled by lookup: every touched path that does not exist (a file a reading
///   creates is excepted), and every dependency that is not among the records.
/// - A read request whose design names no existing code it changes or reuses, and does not list only files it
///   creates, gets `code_problem` set ([`crate::plan::code_reference_problem`]), so the plan refuses it before
///   production. This is the early form of the check the verifier made only at the final verdict.
/// - A read request whose design modifies `athena.toml` or `defaults.toml` without a path under
///   `crates/athena-data/` gets `schema_problem` set ([`crate::plan::schema_reference_problem`]), so the plan
///   refuses it before production instead of admission refusing an unknown field after it.
pub fn plan_entry(
    job: &JobRecords,
    rung_hint: Option<athena_data::roles::Ladder>,
    lookup: &Lookup<'_>,
) -> crate::plan::PlanEntry {
    let text = job.request.as_ref().map_or("", |r| r.text.as_str());
    let first = text.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let mut entry = crate::plan::PlanEntry::bare(job.id().as_str(), &job.title());
    entry.priority = job.priority();
    entry.depends_on = dependency_ids_in(text)
        .into_iter()
        .map(|id| id.as_str().to_string())
        .filter(|id| id != job.id().as_str())
        .collect();
    let mut creates: Vec<String> = Vec::new();
    match job.latest_reading() {
        Some(reading) => {
            entry.goal = if reading.summary.trim().is_empty() {
                first.trim().to_string()
            } else {
                reading.summary.clone()
            };
            if reading.capability.is_some() {
                entry.acceptance = reading
                    .acceptance_criteria
                    .iter()
                    .map(|c| c.text.clone())
                    .collect();
            }
            entry.open_questions = reading.question.iter().cloned().collect();
            entry.touches = reading
                .deliverables
                .iter()
                .map(|d| d.path.clone())
                .collect();
            creates = reading
                .deliverables
                .iter()
                .filter(|d| d.action == DeliverableAction::Create)
                .map(|d| d.path.clone())
                .collect();
        }
        None => {
            entry.goal = first.trim().to_string();
            entry.acceptance = request_text_acceptance(text);
            entry.touches = request_text_touches(text);
        }
    }
    entry.rung =
        rung_hint.or_else(|| infer_rung(text, job.latest_reading(), &entry.touches, lookup.brief));
    // The owner's stated priority reaches the plan: a request serving an addendum that states one is ranked
    // at least that high. A priority record from `athena priority` is the user's word and is left alone.
    if job.priorities.is_empty() {
        let mut hay = text.to_string();
        if let Some(r) = job.latest_reading() {
            hay.push(' ');
            hay.push_str(&r.summary);
        }
        if let Some(topic) = serves_topic(&hay, lookup.brief)
            && topic.priority < entry.priority
        {
            entry.priority = topic.priority;
        }
    }
    for path in &entry.touches {
        if !creates.contains(path) && !(lookup.path_exists)(path) {
            entry.open_questions.push(format!(
                "the touched path {path} does not exist at the base commit"
            ));
        }
    }
    // The early check that the design names the existing code it changes or reuses (or says it is wholly
    // new): it runs here, when the plan is made, not at the final verdict. A declined reading has no design.
    if let Some(reading) = job.latest_reading()
        && reading.capability.is_some()
    {
        entry.code_problem =
            crate::plan::code_reference_problem(&entry.touches, &creates, lookup.path_exists);
        // The same stage checks that a configuration edit comes with the schema that must accept it.
        let changes: Vec<(String, bool)> = reading
            .deliverables
            .iter()
            .map(|d| (d.path.clone(), d.action == DeliverableAction::Delete))
            .collect();
        entry.schema_problem = crate::plan::schema_reference_problem(&changes);
    }
    for id in &entry.depends_on {
        if !(lookup.job_exists)(id) {
            entry
                .open_questions
                .push(format!("the referenced job {id} is not among the records"));
        }
    }
    entry
}

/// Words that may sit between a follow-up phrase and the job id it names ("follows up on job <id>").
pub(super) const FOLLOW_UP_FILLER: [&str; 6] = ["on", "of", "to", "job", "the", "earlier"];

/// The phrases that say a request continues a named job.
pub(super) const FOLLOW_UP_PHRASES: [&str; 5] = [
    "follows up",
    "follow up",
    "follow-up",
    "following up",
    "followup",
];

/// The job ids a request says it follows up: the id that comes right after a phrase such as "follows up
/// job <id>", "follow-up of <id>" or "following up on <id>" (case-insensitive; only the words `on`, `of`,
/// `to`, `job`, `the` and `earlier` may sit between the phrase and the id). Every other id in the text is
/// evidence the request cites, not a job it continues. In order, without repeats.
pub fn followed_up_ids(text: &str) -> Vec<RecordId> {
    // ASCII lowercasing keeps byte offsets, so positions in `lower` index `text` too.
    let lower = text.to_ascii_lowercase();
    let mut found: Vec<(usize, RecordId)> = Vec::new();
    for phrase in FOLLOW_UP_PHRASES {
        for (at, _) in lower.match_indices(phrase) {
            let rest = &text[at + phrase.len()..];
            let words = rest.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'));
            for word in words.filter(|w| !w.is_empty()) {
                if FOLLOW_UP_FILLER.contains(&word.to_ascii_lowercase().as_str()) {
                    continue;
                }
                if let Ok(id) = RecordId::parse(word) {
                    found.push((at, id));
                }
                break;
            }
        }
    }
    found.sort_by_key(|(at, _)| *at);
    let mut ids: Vec<RecordId> = Vec::new();
    for (_, id) in found {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    ids
}

/// The earlier jobs a reading must account for: the retried job (`attempt`, from the job's `previous_job`
/// link) and those of the carried `referenced` jobs that the request says it follows up (see
/// [`followed_up_ids`]). Jobs the request only cites as evidence are left out, so their results need not be
/// restated; the reader still sees them in its brief.
pub fn continued_jobs<'a>(
    request: &str,
    attempt: Option<&'a EarlierAttempt>,
    referenced: &'a [EarlierAttempt],
) -> Vec<&'a EarlierAttempt> {
    let followed = followed_up_ids(request);
    attempt
        .into_iter()
        .chain(
            referenced
                .iter()
                .filter(|r| followed.iter().any(|id| id.as_str() == r.job)),
        )
        .collect()
}

/// The most jobs that the retried job's introspection may add to the reader brief, after the jobs the
/// request names.
pub const MAX_CITED_JOBS: usize = 4;

/// The job ids an introspection cites, in order, without repeats: first the job of each earlier case, then
/// the ids written in the cause, the prevention and the proposed request, then those in the texts of the
/// earlier cases.
pub fn introspection_cited_ids(cause: &RetryCause) -> Vec<RecordId> {
    let mut ids: Vec<RecordId> = Vec::new();
    let mut add = |id: RecordId| {
        if !ids.contains(&id) {
            ids.push(id);
        }
    };
    for job in cause.earlier_cases.iter().filter_map(|c| c.job.as_deref()) {
        if let Ok(id) = RecordId::parse(job) {
            add(id);
        }
    }
    for text in [&cause.cause, &cause.prevention, &cause.proposed_request] {
        job_ids_in(text).into_iter().for_each(&mut add);
    }
    for case in &cause.earlier_cases {
        job_ids_in(&case.what).into_iter().for_each(&mut add);
        if let Some(prevention) = &case.prevention {
            job_ids_in(prevention).into_iter().for_each(&mut add);
        }
    }
    ids
}

/// The past jobs a reading may rely on: each one's request, criteria, deliverables and final findings,
/// redacted. First the jobs the request names by full id (at most [`MAX_REFERENCED_JOBS`]; those it says
/// it follows up come first so the cap never drops one), then the jobs the retried job's introspection
/// cites (at most [`MAX_CITED_JOBS`], not already carried). The retried job itself is left out. Ids that
/// match no job (or a job without a request) are ignored.
///
/// Root cause this fixes: ids that an introspection cited were never resolved to the deliverables recorded
/// for those jobs, so a reading built its fixtures and thresholds from an assumed shape that was invented.
/// The reader now sees the recorded shape of every job the request or the introspection cites. Ids the
/// reader introduces in its own reading are not resolved here.
///
/// Carrying a job only shows it to the reader: [`continued_jobs`] decides which of them the reading must
/// account for.
pub fn referenced_jobs(
    records: &Records,
    request: &str,
    attempt: Option<&EarlierAttempt>,
    redactor: &Redactor,
) -> Result<Vec<EarlierAttempt>> {
    let skip = attempt.map(|a| a.job.as_str());
    let mut out: Vec<EarlierAttempt> = Vec::new();
    let mut ids = followed_up_ids(request);
    for id in job_ids_in(request) {
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    for id in ids {
        if out.len() >= MAX_REFERENCED_JOBS {
            break;
        }
        if skip == Some(id.as_str()) {
            continue;
        }
        if let Some(job) = records.jobs.get(id.as_str())
            && let Some(carried) = carry(job, redactor)?
        {
            out.push(carried);
        }
    }
    let cited = attempt
        .and_then(|a| a.introspection.as_ref())
        .map(introspection_cited_ids)
        .unwrap_or_default();
    let mut added = 0;
    for id in cited {
        if added >= MAX_CITED_JOBS {
            break;
        }
        if skip == Some(id.as_str()) || out.iter().any(|j| j.job == id.as_str()) {
            continue;
        }
        if let Some(job) = records.jobs.get(id.as_str())
            && let Some(carried) = carry(job, redactor)?
        {
            out.push(carried);
            added += 1;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reading::tests::scoped_reading;
    use athena_context::briefs::EarlierCase;

    fn job_with_reading(
        id: &str,
        request: &str,
        deliverables: &[(&str, DeliverableAction)],
    ) -> JobRecords {
        let job = JobId::parse(id).unwrap();
        let mut reading = scoped_reading("a reading", &[], &[]);
        reading.job = job.clone();
        reading.deliverables = deliverables
            .iter()
            .map(|(path, action)| Deliverable {
                path: (*path).into(),
                action: *action,
                p: 0.9,
            })
            .collect();
        JobRecords {
            id: Some(job.clone()),
            request: Some(athena_data::records::Request {
                job,
                text: request.into(),
                received_at: jiff::Timestamp::UNIX_EPOCH,
                channel: "cli".into(),
            }),
            readings: vec![reading],
            ..JobRecords::default()
        }
    }

    fn id_of(n: usize) -> String {
        format!("20261110-00000{n}-aaaa")
    }

    fn records_of(jobs: Vec<JobRecords>) -> Records {
        let mut records = Records::default();
        for job in jobs {
            records.jobs.insert(job.id().as_str().to_string(), job);
        }
        records
    }

    fn attempt_citing(job: &str, cause: RetryCause) -> EarlierAttempt {
        EarlierAttempt {
            job: job.into(),
            request: "the retried request".into(),
            introspection: Some(cause),
            ..EarlierAttempt::default()
        }
    }

    fn redactor() -> Redactor {
        Redactor::new(&athena_gate::builtin_config()).unwrap()
    }

    const A_LIST: [&str; 2] = ["modify crates/x/a.rs", "create crates/x/b.rs"];

    fn a_records() -> Records {
        records_of(vec![
            job_with_reading(
                &id_of(1),
                "job a",
                &[
                    ("crates/x/a.rs", DeliverableAction::Modify),
                    ("crates/x/b.rs", DeliverableAction::Create),
                ],
            ),
            job_with_reading(
                &id_of(2),
                "job b",
                &[("crates/y.rs", DeliverableAction::Modify)],
            ),
        ])
    }

    #[test]
    fn referenced_jobs_returns_a_job_cited_only_by_the_introspection() {
        let records = a_records();
        let a = id_of(1);
        let b = id_of(2);
        let causes = [
            RetryCause {
                earlier_cases: vec![EarlierCase {
                    id: "i1".into(),
                    job: Some(a.clone()),
                    ..EarlierCase::default()
                }],
                ..RetryCause::default()
            },
            RetryCause {
                cause: format!("it repeated what happened in {a}"),
                ..RetryCause::default()
            },
            RetryCause {
                prevention: format!("rule: check against {a}"),
                ..RetryCause::default()
            },
        ];
        for cause in causes {
            let attempt = attempt_citing(&b, cause);
            let got =
                referenced_jobs(&records, "try again on it", Some(&attempt), &redactor()).unwrap();
            assert_eq!(got.len(), 1, "{got:?}");
            assert_eq!(got[0].job, a);
            assert_eq!(got[0].reading_deliverables, A_LIST);
        }
    }

    #[test]
    fn referenced_jobs_skips_the_retried_job_and_unknown_ids() {
        let records = a_records();
        let (a, b) = (id_of(1), id_of(2));
        let attempt = attempt_citing(
            &b,
            RetryCause {
                cause: format!("{b} {} {a}", id_of(9)),
                ..RetryCause::default()
            },
        );
        let got = referenced_jobs(&records, "x", Some(&attempt), &redactor()).unwrap();
        assert_eq!(
            got.iter().map(|j| j.job.as_str()).collect::<Vec<_>>(),
            [a.as_str()]
        );
        let without = referenced_jobs(&records, "x", None, &redactor()).unwrap();
        assert!(without.is_empty());
        let named = referenced_jobs(&records, &format!("see {a}"), None, &redactor()).unwrap();
        assert_eq!(named.len(), 1);
    }

    #[test]
    fn referenced_jobs_lists_request_named_jobs_first_and_never_drops_one() {
        // Jobs 1..=3 and 8 are named by the request; 4..=7 and 9 are cited by the introspection.
        let jobs: Vec<JobRecords> = (1..=9)
            .map(|n| {
                let path = format!("crates/n{n}.rs");
                job_with_reading(
                    &id_of(n),
                    "r",
                    &[(path.as_str(), DeliverableAction::Modify)],
                )
            })
            .chain([job_with_reading("20261110-000059-bbbb", "retried", &[])])
            .collect();
        let records = records_of(jobs);
        let cited: Vec<String> = [4, 5, 1, 6, 7, 9, 8].iter().map(|n| id_of(*n)).collect();
        let attempt = attempt_citing(
            "20261110-000059-bbbb",
            RetryCause {
                cause: cited.join(" "),
                ..RetryCause::default()
            },
        );
        let request = format!("see {} {} {} {}", id_of(1), id_of(2), id_of(3), id_of(8));
        let got = referenced_jobs(&records, &request, Some(&attempt), &redactor()).unwrap();
        let ids: Vec<&str> = got.iter().map(|j| j.job.as_str()).collect();
        let want: Vec<String> = [1, 2, 3, 4, 5, 6, 7].iter().map(|n| id_of(*n)).collect();
        assert_eq!(ids, want, "request-named first, then cited");
        assert_eq!(got.len(), MAX_REFERENCED_JOBS + MAX_CITED_JOBS);
        assert_eq!(got[3].reading_deliverables, ["modify crates/n4.rs"]);
        assert_eq!(got[4].reading_deliverables, ["modify crates/n5.rs"]);
        assert_ne!(got[3].reading_deliverables, got[4].reading_deliverables);

        // The request cap dropped the fourth request-named job; it is carried when the introspection cites it.
        let attempt = attempt_citing(
            "20261110-000059-bbbb",
            RetryCause {
                cause: id_of(8),
                ..RetryCause::default()
            },
        );
        let got = referenced_jobs(&records, &request, Some(&attempt), &redactor()).unwrap();
        let ids: Vec<&str> = got.iter().map(|j| j.job.as_str()).collect();
        assert_eq!(ids, [id_of(1), id_of(2), id_of(3), id_of(8)]);
    }

    #[test]
    fn referenced_jobs_doc_and_changelog_name_the_cause() {
        let marker = concat!("pub fn ", "referenced_jobs(");
        let source = include_str!("topics.rs");
        let at = source.find(marker).unwrap();
        let mut doc: Vec<&str> = Vec::new();
        for line in source[..at].lines().rev() {
            if line.trim_start().starts_with("///") {
                doc.push(line);
            } else if !doc.is_empty() {
                break;
            }
        }
        let doc = doc.join("\n");
        let phrases = [concat!("never ", "resolved"), concat!("in", "vented")];
        for phrase in phrases {
            assert!(doc.contains(phrase), "doc lacks {phrase}: {doc}");
        }
        let changelog = include_str!("../../../../CHANGELOG.md");
        let from = changelog.find("## [Unreleased]").unwrap();
        let rest = &changelog[from + 5..];
        let section = &rest[..rest.find("\n## [").unwrap_or(rest.len())];
        for needle in [
            "20261109-075120-7zqw",
            "20261029-111121-fzx0",
            "20261110-124307-bn3c",
        ] {
            assert!(section.contains(needle), "changelog lacks {needle}");
        }
        for phrase in phrases {
            assert!(section.contains(phrase), "changelog lacks {phrase}");
        }
    }
}
