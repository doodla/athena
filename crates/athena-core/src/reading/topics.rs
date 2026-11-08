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

/// The jobs a request names by full id, other than `skip` (the retried job): each one's request and
/// final findings, redacted. Ids that match no job (or a job without a request) are ignored. At most
/// [`MAX_REFERENCED_JOBS`] are carried; jobs the request says it follows up come first so the cap never
/// drops one. Carrying a job only shows it to the reader: [`continued_jobs`] decides which of them the
/// reading must account for.
pub fn referenced_jobs(
    records: &Records,
    request: &str,
    skip: Option<&str>,
    redactor: &Redactor,
) -> Result<Vec<EarlierAttempt>> {
    let mut out = Vec::new();
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
    Ok(out)
}
