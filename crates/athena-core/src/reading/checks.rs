//! Reference, boundary, acceptance and governance checks on a reading.

use super::*;
use athena_context::briefs::change_limit;

/// Refuses a reading for a retried or follow-up job that leaves out the previous result or the change
/// scope. `priors` are the retried job and the jobs the request says it follows up (see
/// [`continued_jobs`]), never jobs it only cites as evidence; with none (a first attempt, or a request
/// that cites jobs as evidence) every reading passes, and so does one that declines the request or asks a question, which has no change to
/// scope yet.
///
/// - The previous result is reflected when the reading's own text (title, summary, assumptions,
///   departures and criteria) names each prior job by its id, so each finding is addressed or set aside
///   in words the user can read. The brief asks for exactly this.
/// - The change scope is stated when the reading lists at least one deliverable and at least one
///   out-of-scope item.
/// - The earlier job's files are accounted for when each file its candidate changed (the carried
///   `changed_files`) is covered by a deliverable (the same path, or a directory deliverable above it) or is
///   named in an out-of-scope item, as its own path or as a directory above it (see [`names_path_token`]).
///   Only deliverables and out-of-scope items count: the survey and other prose of a reading do not say
///   whether earlier work is kept or dropped. A prior with no recorded files asks for nothing.
///
/// The error names each missing part: "previous result", "change scope" and "the earlier job's files".
///
/// Root cause this part removes: the carry-over of an earlier job left out the files its candidate changed,
/// so a retry's reading could name the job and still ignore (keep or drop without saying) the work it did.
///
/// This check refuses more readings; the higher refusal rate is intended. A replay of accepted readings
/// (`replay`) holds a job that newly refuses more than 10% unless the job's criteria declare a share, as
/// "expected N% of newly refused readings", and give a reason when it is above 25%. The replay passes no
/// earlier attempts to the checks (see `check_reading_text`), so it cannot observe this part: the share a
/// job declares is an expectation, not a measurement.
pub fn check_prior_result_and_scope(reading: &Reading, priors: &[&EarlierAttempt]) -> Result<()> {
    if priors.is_empty() || reading.capability.is_none() || reading.question.is_some() {
        return Ok(());
    }
    let mut text = vec![
        reading.title.as_str(),
        reading.summary.as_str(),
        reading.departures.as_deref().unwrap_or_default(),
    ];
    text.extend(reading.assumptions.iter().map(String::as_str));
    text.extend(reading.acceptance_criteria.iter().map(|c| c.text.as_str()));
    let text = text.join("\n").to_lowercase();
    let unnamed: Vec<&str> = priors
        .iter()
        .map(|p| p.job.as_str())
        .filter(|id| !text.contains(&id.to_lowercase()))
        .collect();
    let mut missing: Vec<String> = Vec::new();
    if !unnamed.is_empty() {
        missing.push(format!(
            "the previous result: it does not name job {} in its summary, assumptions or criteria, \
             so it does not say how the earlier findings and end state are addressed or why they no longer apply",
            unnamed.join(", job ")
        ));
    }
    let mut scope: Vec<&str> = Vec::new();
    if reading.deliverables.is_empty() {
        scope.push("deliverables");
    }
    if reading.out_of_scope.iter().all(|i| i.trim().is_empty()) {
        scope.push("out_of_scope items");
    }
    if !scope.is_empty() {
        missing.push(format!(
            "the change scope: it lists no {}",
            scope.join(" and no ")
        ));
    }
    let ignored: Vec<String> = priors
        .iter()
        .filter_map(|p| {
            let files = unaccounted_files(p, reading);
            if files.is_empty() {
                return None;
            }
            let shown = files
                .iter()
                .take(MAX_FILES_NAMED)
                .copied()
                .collect::<Vec<_>>();
            let more = match files.len() - shown.len() {
                0 => String::new(),
                n => format!(" and {n} more"),
            };
            Some(format!("job {} changed {}{more}", p.job, shown.join(", ")))
        })
        .collect();
    if !ignored.is_empty() {
        missing.push(format!(
            "the earlier job's files: {}, but the reading neither lists them as deliverables nor excludes them \
             in out_of_scope, so it does not say whether that work is kept or dropped",
            ignored.join("; ")
        ));
    }
    if missing.is_empty() {
        return Ok(());
    }
    bail!(
        "the reading was refused because it leaves out {}; this job retries or follows up an earlier job, \
         so the reading must reflect that job's result and state the allowed change scope",
        missing.join("; and ")
    )
}

/// The most files of one earlier job that a refusal names.
pub(super) const MAX_FILES_NAMED: usize = 5;

/// The paths of the files `prior`'s candidate changed that the reading neither lists nor excludes: a file is
/// accounted for when a deliverable is the file itself or a directory above it (compared by whole path
/// components), or when an out-of-scope item names the file or a directory above it as a whole path token
/// ([`names_path_token`]). Case is ignored. Free text such as the summary or the assumptions never counts.
pub(super) fn unaccounted_files<'a>(prior: &'a EarlierAttempt, reading: &Reading) -> Vec<&'a str> {
    let deliverables: Vec<String> = reading
        .deliverables
        .iter()
        .map(|d| d.path.trim().trim_end_matches('/').to_lowercase())
        .collect();
    let out_of_scope: Vec<String> = reading
        .out_of_scope
        .iter()
        .map(|item| item.to_lowercase())
        .collect();
    prior
        .changed_files
        .iter()
        // A carried entry is `status path`; the path may itself hold spaces.
        .filter_map(|entry| entry.split_once(' ').map(|(_, path)| path))
        .filter(|path| {
            let file = path.to_lowercase();
            let covered = deliverables.iter().any(|d| {
                file == *d
                    || file
                        .strip_prefix(d.as_str())
                        .is_some_and(|r| r.starts_with('/'))
            });
            let mut names = vec![file.clone()];
            let mut ancestor = file.as_str();
            while let Some((parent, _)) = ancestor.rsplit_once('/') {
                names.push(parent.to_string());
                names.push(format!("{parent}/"));
                ancestor = parent;
            }
            let excluded = out_of_scope
                .iter()
                .any(|item| names.iter().any(|name| names_path_token(item, name)));
            !(covered || excluded)
        })
        .collect()
}

/// Whether `item` holds `token` as a whole path token (both lowercase). A token starts at the start of the
/// text or after white space, a backtick, a quote, `(`, `[`, `{`, `,` or `;`. It ends at the end of the text, or
/// before white space, a closing quote, a backtick, `)`, `]`, `}`, `,`, `;`, `:` or `*`, or before a `.`, `!` or `?`
/// that is itself followed by the end of the text or white space. A `/` or a letter or digit next to the token
/// never matches, so `src/a.rs` is not found in `crates/x/src/a.rs`, and `src/a` is not found in `src/a.rs`.
pub(super) fn names_path_token(item: &str, token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    let starts = |c: char| c.is_whitespace() || "`\"'([{,;\u{2018}\u{201c}".contains(c);
    let ends = |c: char| c.is_whitespace() || "`\"')]},;:*\u{2019}\u{201d}".contains(c);
    item.match_indices(token).any(|(at, _)| {
        let before_ok = item[..at].chars().next_back().is_none_or(starts);
        let rest = &item[at + token.len()..];
        let mut after = rest.chars();
        let after_ok = match after.next() {
            None => true,
            Some(c) if ends(c) => true,
            Some('.' | '!' | '?') => after.next().is_none_or(char::is_whitespace),
            Some(_) => false,
        };
        before_ok && after_ok
    })
}

/// The fields of a reading whose text can tell the writer what to put into a tracked file: the acceptance
/// criteria of kind `content`, each with the label a refusal names it by. The deliverable paths are not here:
/// [`check_boundary_and_repo_files`] scans them as paths.
///
/// A `content` criterion describes what a file must say, so a boundary word in it can be copied into a file.
/// A `behavior` criterion names a command to run or a result to observe. The title, summary, assumptions,
/// out-of-scope list, defaults and the reasons and question are prose in a record, and records are outside the
/// policy scan, so they are not listed. Criterion kind is an imperfect signal in two ways: a `content`
/// criterion that describes a document without quoting text is still scanned, and a `behavior` criterion that
/// quotes output is not (the writer's own scan and the policy scan at verification then catch a word that
/// reaches a tracked file).
pub(super) fn file_bound_text_fields(reading: &Reading) -> Vec<(String, &str)> {
    reading
        .acceptance_criteria
        .iter()
        .enumerate()
        .filter(|(_, criterion)| criterion.kind == CriterionKind::Content)
        .map(|(i, criterion)| (format!("acceptance_criteria[{i}]"), criterion.text.as_str()))
        .collect()
}

/// Whether `path` is a file in `files`, or a directory with at least one file under it.
pub(super) fn path_exists_in(files: &BTreeSet<String>, path: &str) -> bool {
    if files.contains(path) {
        return true;
    }
    let prefix = format!("{}/", path.trim_end_matches('/'));
    files
        .range(prefix.clone()..)
        .next()
        .is_some_and(|f| f.starts_with(&prefix))
}

/// Refuses a reading whose deliverable paths or `content` acceptance criteria hold a word the boundary scan
/// rejects, or whose deliverables disagree with the files at the base commit.
///
/// Root cause: a reading was checked only for the previous result and change scope, the evidence each
/// criterion rests on, the code survey and the governance files. Nothing compared its text with the boundary
/// word list or its deliverables with the repository's files at the base commit. A reading that named a
/// banned word, a path that does not exist, or a `create` of a file that already exists was accepted, and the
/// fault showed up only later, in the producer's run or at verification. The check is deterministic and
/// calls no model; it runs last in [`check_reading_acceptance`], so a refusal reaches the reader as its one
/// correction, as the other refusals do.
///
/// Only the places a word can reach a tracked file go through `athena_context::briefs::boundary_scan`: every
/// deliverable path, and every acceptance criterion of kind `content` (text that says what a file must hold;
/// see [`file_bound_text_fields`]). Not scanned: the title, summary, assumptions, out-of-scope list, defaults
/// applied, the decline, hard and urgent reasons, the question, the departures and `behavior` criteria. A
/// reading is a record, records are outside the boundary scan the policy test runs, and the writer already
/// gets the word list and runs that scan on its own files (D73), so a word in prose cannot fail a file check
/// and refusing it only sent the reader round again (D153). The title, summary and criterion texts also
/// reach landing commit messages and the pull request title, which no scan covers; that gap is recorded in
/// D153, not closed here.
///
/// `repo_files` is the set of file paths at the base commit. When it is `Some`
/// and the reading is a change with no open question, a `modify` or `delete` of a path that is neither a file
/// nor a directory holding files is refused, and so is a `create` of a path that is already a file. A
/// `modify` of `CHANGELOG.md` is allowed when the file is absent, because Athena adds that deliverable
/// itself ([`ensure_changelog`]) and the reader cannot repair it. With `None` (a replay with no repository)
/// only the words are checked.
pub fn check_boundary_and_repo_files(
    reading: &Reading,
    repo_files: Option<&BTreeSet<String>>,
) -> Result<()> {
    let mut problems: Vec<String> = Vec::new();
    for (label, text) in file_bound_text_fields(reading) {
        for hit in athena_context::briefs::boundary_scan(text) {
            problems.push(format!("{label}: the banned word {:?}", hit.term));
        }
    }
    for d in &reading.deliverables {
        for hit in athena_context::briefs::boundary_scan(&d.path) {
            problems.push(format!(
                "deliverable path {}: the banned word {:?}",
                d.path, hit.term
            ));
        }
    }
    let mut path_problems = 0;
    if let Some(files) = repo_files
        && reading.capability.is_some()
        && reading.question.is_none()
    {
        for d in &reading.deliverables {
            let message = match d.action {
                DeliverableAction::Create if files.contains(&d.path) => Some(format!(
                    "deliverable {} is a create, but the file already exists at the base commit \
                     (use modify)",
                    d.path
                )),
                DeliverableAction::Modify | DeliverableAction::Delete
                    if !path_exists_in(files, &d.path)
                        && !(d.action == DeliverableAction::Modify && d.path == CHANGELOG_PATH) =>
                {
                    let verb = if d.action == DeliverableAction::Modify {
                        "modify"
                    } else {
                        "delete"
                    };
                    Some(format!(
                        "deliverable {} is a {verb}, but no such file exists at the base commit \
                         (use create for a new file, or name the path that exists)",
                        d.path
                    ))
                }
                _ => None,
            };
            if let Some(message) = message {
                problems.push(message);
                path_problems += 1;
            }
        }
    }
    if problems.is_empty() {
        return Ok(());
    }
    let word_problems = problems.len() - path_problems;
    let repair = match (word_problems > 0, path_problems > 0) {
        (true, true) => {
            "reword the deliverable path or content acceptance criterion without the banned words, and \
             make each deliverable's action match the files"
        }
        (true, false) => {
            "reword the deliverable path or content acceptance criterion without the banned words"
        }
        _ => "make each deliverable's action match the files at the base commit",
    };
    bail!(
        "the reading was refused because its deliverable paths, content acceptance criteria or \
         deliverables fail a check Athena runs on every reading: {}; {repair}",
        problems.join("; ")
    )
}

/// Top-level directories a reading may name a repo path under, besides those found in the snapshot.
pub(super) const KNOWN_TOP_DIRS: [&str; 5] = ["crates", "docs", "xtask", "studies", ".github"];

/// Lowercase phrases that make a sentence state that a path is absent or is to be created. A path named in
/// such a sentence is not a claim that the path exists.
pub(super) const ABSENCE_PHRASES: [&str; 10] = [
    "not exist",
    "n't exist",
    "absent",
    "missing",
    "no such",
    "not yet",
    "be created",
    "new file",
    "creates",
    "to create",
];

/// The sentences of `text`, split at a full stop followed by white space or the end, at a newline and at
/// a semicolon.
pub(super) fn sentences(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        let ends = match c {
            '\n' | ';' => true,
            '.' => chars.peek().is_none_or(|(_, next)| next.is_whitespace()),
            _ => false,
        };
        if ends {
            out.push(&text[start..i]);
            start = i + c.len_utf8();
        }
    }
    out.push(&text[start..]);
    out
}

/// The repo-path tokens in `sentence`: each maximal run of letters, digits and `_ . / -` that holds a `/`,
/// with trailing `.` and `/` trimmed. A run is skipped when it starts with `/` or `./`, holds `..` or an empty
/// segment (an absolute path, a relative path, an ellipsis), or sits next to a glob or placeholder character.
/// A `:` ends a run, so `path:12` yields `path`, and a URL yields nothing (its run starts with `//`).
pub(super) fn path_tokens(sentence: &str) -> Vec<String> {
    const NEIGHBOURS: &str = "*<>{}$~%@";
    let is_path_char = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-');
    let chars: Vec<char> = sentence.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        if !is_path_char(chars[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < chars.len() && is_path_char(chars[i]) {
            i += 1;
        }
        let before = start.checked_sub(1).map(|j| chars[j]);
        let after = chars.get(i).copied();
        if before.is_some_and(|c| NEIGHBOURS.contains(c))
            || after.is_some_and(|c| NEIGHBOURS.contains(c))
        {
            continue;
        }
        let run: String = chars[start..i].iter().collect();
        let token = run.trim_end_matches(['.', '/']);
        if token.contains('/')
            && !run.starts_with('/')
            && !run.starts_with("./")
            && !run.contains("..")
            && !run.contains("//")
        {
            out.push(token.to_string());
        }
    }
    out
}

/// Refuses a reading whose summary, assumptions or acceptance criteria rely on a repo path that is not in the
/// snapshot at the base commit and that the reading neither creates nor says is absent.
///
/// Root cause: reading acceptance never compared the reading's references with the snapshot. It checked the
/// deliverable paths ([`check_boundary_and_repo_files`]) and nothing else, so a reading whose text said an
/// earlier job's file, role or prompt already exists was accepted as written. The earlier job might be
/// queued, unmerged or never produced, and the critic or the producer found the gap later. Jobs
/// 20261026-042558-5p65, 20261026-064950-38jx and 20261026-065228-02zf each planned on a design-review role
/// or a finding state that was not in the snapshot; the check stops that class for any path, not for those
/// jobs.
///
/// A path token (see [`path_tokens`]) whose first segment is a top-level directory of the snapshot or one of
/// [`KNOWN_TOP_DIRS`] must be a file or a directory holding files in `repo_files`, a `create` deliverable (or
/// a directory above one), `CHANGELOG.md` (Athena adds it itself), or be named in a sentence with one of
/// [`ABSENCE_PHRASES`]. Slash-less names such as `reading.rs` are not checked: they are shorthand for a file
/// elsewhere. With `repo_files` `None` (a replay with no repository), or a reading that declines or asks a
/// question, nothing is checked. The check is deterministic and calls no model.
pub fn check_reading_references(
    reading: &Reading,
    repo_files: Option<&BTreeSet<String>>,
) -> Result<()> {
    let Some(files) = repo_files else {
        return Ok(());
    };
    if reading.capability.is_none() || reading.question.is_some() {
        return Ok(());
    }
    let mut top_dirs: BTreeSet<&str> = KNOWN_TOP_DIRS.into_iter().collect();
    top_dirs.extend(
        files
            .iter()
            .filter_map(|f| f.split_once('/').map(|(d, _)| d)),
    );
    let created: Vec<&str> = reading
        .deliverables
        .iter()
        .filter(|d| d.action == DeliverableAction::Create)
        .map(|d| d.path.as_str())
        .collect();
    let is_created = |path: &str| {
        created
            .iter()
            .any(|c| *c == path || c.starts_with(&format!("{path}/")))
    };
    let mut fields: Vec<(String, &str)> = vec![("summary".into(), reading.summary.as_str())];
    for (i, text) in reading.assumptions.iter().enumerate() {
        fields.push((format!("assumptions[{i}]"), text.as_str()));
    }
    for (i, c) in reading.acceptance_criteria.iter().enumerate() {
        fields.push((format!("acceptance_criteria[{i}]"), c.text.as_str()));
    }
    let mut missing: Vec<String> = Vec::new();
    for (label, text) in fields {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        for sentence in sentences(text) {
            let lower = sentence.to_lowercase();
            if ABSENCE_PHRASES.iter().any(|p| lower.contains(p)) {
                continue;
            }
            for token in path_tokens(sentence) {
                let top = token.split('/').next().unwrap_or_default();
                if top_dirs.contains(top)
                    && token != CHANGELOG_PATH
                    && !path_exists_in(files, &token)
                    && !is_created(&token)
                    && seen.insert(token.clone())
                {
                    missing.push(format!("{label}: {token}"));
                }
            }
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    bail!(
        "the reading was refused because its text relies on repo paths that are not in the snapshot at the \
         base commit: {}; an earlier job's file, role or prompt is not there until that job lands. Repair: \
         list each missing artifact as a create deliverable if this change adds it, or say in the text that \
         it does not exist, or name the path that does exist",
        missing.join("; ")
    )
}

/// Every check a built reading must pass before it is accepted: the previous result and change scope of a
/// retried or followed-up job ([`check_prior_result_and_scope`]), then the evidence each criterion rests on
/// ([`check_criteria_evidence`]), then the code survey a code or tests reading with a behavior criterion
/// rests on ([`check_code_survey`]), then the realistic-branch fixture a commit classification, history or
/// attribution rule is tested on ([`check_history_fixture`]), then the governance files the outcome needs
/// ([`check_governance_coverage`]), then the observable state a delivery request needs
/// ([`check_outcome_state`]), then the place, permissions and data reach of each action or source the reading
/// assigns ([`check_environment_reach`]), then the structural terms the request uses and the reading leaves
/// undefined ([`check_defined_terms`]), then the files a criterion that limits which files may change leaves
/// out, those Athena always writes and the readers of a changed config ([`check_scope_limit`]), then the
/// deliverable paths and `content` criteria against the boundary
/// words and the deliverables against the files at the base commit ([`check_boundary_and_repo_files`]; `repo_files` is `None` when
/// there is no repository to compare with), then the repo paths its text relies on against the same files
/// ([`check_reading_references`]). It applies to a first attempt as well as a retry. It runs
/// [`refusing_check`] and drops the name of the check that refused.
pub fn check_reading_acceptance(
    reading: &Reading,
    request: &str,
    attempt: Option<&EarlierAttempt>,
    referenced: &[EarlierAttempt],
    repo_files: Option<&BTreeSet<String>>,
) -> Result<()> {
    match refusing_check(reading, request, attempt, referenced, repo_files) {
        None => Ok(()),
        Some((_, error)) => Err(error),
    }
}

/// Runs the checks of [`check_reading_acceptance`], in its order, and returns the name of the first one that
/// refuses together with its error; `None` when the reading is accepted.
///
/// Root cause this seam serves: refusal and matching rules (the parts parser, stems, stop lists and the
/// reading checks) were specified by reasoning and hand-checked on a few cases, and nothing executed them
/// against real request and reading shapes before they were planned or landed, so a gap (numbered `N.` and
/// `N)` parts, stop-list and stem order, a job cited but not relied on, a brand word read as a symbol) showed
/// up only in critic rounds. Jobs 20261029-111202-2bpc and 20261030-060751-5389 each failed in planning that
/// way. The corpus in `crates/athena-core/tests/refusal_corpus.rs` runs real shapes through this function and
/// asserts which check refuses each one, and it fails when a `step!` line below has no corpus case in which
/// that check refuses. A new check is added as one `step!(check_name(..))` line here, never as a direct call
/// in [`check_reading_acceptance`].
///
/// The names come from the called functions (`stringify!`), so no second list can drift from the real one.
/// Order, short-circuit and error text are those of the gate, so this adds no refusal.
pub fn refusing_check(
    reading: &Reading,
    request: &str,
    attempt: Option<&EarlierAttempt>,
    referenced: &[EarlierAttempt],
    repo_files: Option<&BTreeSet<String>>,
) -> Option<(&'static str, anyhow::Error)> {
    macro_rules! step {
        ($name:ident($($arg:expr),*)) => {
            if let Err(error) = $name($($arg),*) {
                return Some((stringify!($name), error));
            }
        };
    }
    step!(check_prior_result_and_scope(
        reading,
        &continued_jobs(request, attempt, referenced)
    ));
    step!(check_criteria_evidence(reading));
    step!(check_code_survey(reading));
    step!(check_history_fixture(reading));
    step!(check_design_existing_code(reading));
    step!(check_governance_coverage(reading, request));
    step!(check_outcome_state(reading, request));
    step!(check_environment_reach(reading));
    step!(check_defined_terms(reading, request));
    step!(check_scope_limit(reading));
    step!(check_boundary_and_repo_files(reading, repo_files));
    step!(check_reading_references(reading, repo_files));
    None
}

/// Governance paths a request can name, and the whole-word lowercase phrase that names each. A path ending in
/// `/` stands for any file under that directory; its phrase has no trailing `/`, so a file path under the
/// directory (`.github/workflows/ci.yml`) matches as well as the directory. A request word such as "deploy"
/// or "config" names no path and requires nothing.
pub(super) const GOVERNANCE_BY_NAMED_PATH: [(&str, &str); 5] = [
    (".github/workflows/", ".github/workflows"),
    ("athena.toml", "athena.toml"),
    ("defaults.toml", "defaults.toml"),
    ("rust-toolchain.toml", "rust-toolchain.toml"),
    ("setup.sh", "setup.sh"),
];

/// Deliverables whose change implies a governance file changes with them, and why.
pub(super) const GOVERNANCE_BY_DELIVERABLE: [(&str, &str, &str); 1] = [(
    "crates/athena-data/src/config.rs",
    "athena.toml",
    "a new or changed configuration section must be set in athena.toml, or its schema test fails",
)];

/// Files Athena writes in every change reading whatever the request says, and why. A criterion that limits
/// which files may change must let the job write each of them.
pub(super) const DEFAULT_WRITTEN_FILES: [(&str, &str); 1] = [(
    CHANGELOG_PATH,
    "Athena always adds it to a change reading, and the job must record itself there",
)];

/// Configuration files that many tests and tools read, so changing one can break a file the request never named.
pub(super) const CONFIG_FILES: [&str; 2] = ["athena.toml", "defaults.toml"];

/// Whether the reading covers the governance `path` (a directory when it ends in `/`): it lists a governance
/// deliverable there and names it in a criterion, or it names the path in `out_of_scope`.
pub(super) fn governance_is_covered(reading: &Reading, path: &str) -> bool {
    let out_of_scope = reading
        .out_of_scope
        .iter()
        .any(|item| item.contains(path.trim_end_matches('/')));
    if out_of_scope {
        return true;
    }
    if path.ends_with('/') {
        reading
            .deliverables
            .iter()
            .any(|d| d.path.starts_with(path) && governance_change_is_listed(reading, &d.path))
    } else {
        governance_change_is_listed(reading, path)
    }
}

/// Refuses a reading that leaves out a governance file its outcome needs, or lists one without naming it.
///
/// Root cause this check removes: reading acceptance had no mapping from the request's outcome to governance
/// files, so a reading for "publish to GitHub Pages" (which needs a workflow), a new configuration section
/// (which needs `athena.toml`) or CI routing (which needs `athena.toml`) was accepted without them, and the
/// omission was found only at admission, after the producer had finished. Jobs 20261014-082328-g6vy,
/// 20261015-014012-akwq, 20261017-044557-3htd and 20261017-114155-ksq0 each failed that way. Admission has one
/// definition of a governance file (`is_governance`) and of a listed one (`governance_change_is_listed`); this
/// check applies the same two to the reading, where the reader can still repair it, for every reading with a
/// capability and a deliverable, first attempt or retry. It costs no model call.
///
/// It refuses when:
/// - a deliverable is a governance file that no criterion names;
/// - a deliverable in `GOVERNANCE_BY_DELIVERABLE` is listed without its governance file; or
/// - the request names a path of `GOVERNANCE_BY_NAMED_PATH` (the file itself, or a file under the directory)
///   and the reading neither lists and names that governance file nor names it in `out_of_scope`.
///
/// A request word that only suggests a governance file (deploy, config, CI, gate) requires nothing: the
/// earlier word table refused ordinary requests that merely used a common word, and the reader was not told
/// why, so the next read failed the same way (`Athena::read_for` now reads once more with the refusal text).
/// The error names each missing path and why, and says to list it as a deliverable and name it in a
/// criterion, or to name it in `out_of_scope`.
pub fn check_governance_coverage(reading: &Reading, request: &str) -> Result<()> {
    if reading.capability.is_none() || reading.question.is_some() || reading.deliverables.is_empty()
    {
        return Ok(());
    }
    let mut missing: Vec<(String, String)> = Vec::new();
    let mut need = |path: &str, why: String| {
        if !missing.iter().any(|(p, _)| p == path) {
            missing.push((path.to_string(), why));
        }
    };
    for d in &reading.deliverables {
        if is_governance(&d.path) && !governance_change_is_listed(reading, &d.path) {
            need(
                &d.path,
                "it is listed as a deliverable but no criterion names it".into(),
            );
        }
        for (trigger, path, why) in GOVERNANCE_BY_DELIVERABLE {
            if d.path == trigger && !governance_is_covered(reading, path) {
                need(path, format!("{trigger} is a deliverable, and {why}"));
            }
        }
    }
    // An underscore joins a word (`athena_gate` is one identifier), so it is not a boundary.
    let words = request.to_lowercase().replace('_', "x");
    for (path, phrase) in GOVERNANCE_BY_NAMED_PATH {
        if has_phrase(&words, phrase) && !governance_is_covered(reading, path) {
            need(
                path,
                format!(
                    "the request names {phrase:?}, so the reading must say whether the job changes it"
                ),
            );
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    let paths: Vec<String> = missing
        .iter()
        .map(|(path, why)| format!("{path} ({why})"))
        .collect();
    bail!(
        "the reading was refused because the outcome needs governance files it does not cover: {}; \
         list each missing path as a deliverable and name it in an acceptance criterion, or, if this job must \
         not change it, name it in out_of_scope; admission refuses a governance change that is not both listed \
         and named, and only after the producer has finished",
        paths.join("; ")
    )
}

/// Refuses a change reading whose criterion limits which files may change but leaves out a file the job
/// must change anyway: one Athena always writes, or one that reads a changed config.
///
/// Root cause this check removes: a scope limit written from the named files without listing default-written
/// files and config consumers. The reader wrote "only these files may change" from the files the request
/// named, and nothing at the reading gate compared that limit with the files Athena always writes
/// (`CHANGELOG.md`, see [`DEFAULT_WRITTEN_FILES`]) or with the tests and tools that load a changed config
/// (`athena.toml`, `defaults.toml`). Verification builds its allowed list from the same limit
/// (`AcceptedReading::allowed_paths`), so the job failed there, after the producer had finished. Failure
/// class scope-criterion-too-narrow-misses-dependent-tests; jobs 20261018-045437-7z81 (the tests that
/// load the real `athena.toml` were not allowed to change) and 20261023-052543-9bw7 (the limit left out
/// `CHANGELOG.md`).
///
/// The limit is read by `change_limit` in athena-context, the matcher that enforcement uses, and only the
/// first limiting criterion counts, as in enforcement. It refuses when:
/// - the limit names its allowed files (not "no other file is changed") and those words neither include
///   `CHANGELOG.md` nor say `the deliverables`. A `CHANGELOG.md` deliverable alone does not repair the text,
///   because `ensure_changelog` adds it before the gate in production; or
/// - a deliverable is `athena.toml` or `defaults.toml` (limit named or bare) and no `Consumers:` assumption
///   answers who loads it. The line must name only deliverable paths (a config file itself is ignored) and
///   either name at least one or say "none found", "unchanged" or "no change". A consumer path that is not a
///   deliverable is refused whatever else the line says, because with a limit the deliverable list is the
///   allowed list.
///
/// The error names each missing file, why, and the repair. Only criteria are read, as in enforcement, and the
/// repo is not searched: the check forces the question and checks that every named consumer is a deliverable,
/// but it cannot tell that `Consumers: none found` followed a search. Known misses: a bare form that really
/// forbids `CHANGELOG.md`, a negation inside the allowed words, and a second limiting criterion.
///
/// This check refuses more readings; the higher refusal rate is intended: a scope limit that omits files
/// Athena always writes fails verification. The share is expected 3% of newly refused readings, an estimate
/// and not a measurement; the replay of accepted readings measures it.
pub fn check_scope_limit(reading: &Reading) -> Result<()> {
    if reading.capability.as_deref() != Some("change") || reading.question.is_some() {
        return Ok(());
    }
    let Some(limit) = reading
        .acceptance_criteria
        .iter()
        .find_map(|c| change_limit(&c.text))
    else {
        return Ok(());
    };
    let mut missing: Vec<String> = Vec::new();
    let names_deliverables = limit
        .allowed
        .windows(2)
        .any(|pair| pair[0] == "the" && pair[1] == "deliverables");
    if !limit.bare && !names_deliverables {
        for (file, why) in DEFAULT_WRITTEN_FILES {
            if !limit.allowed.iter().any(|w| *w == file.to_lowercase()) {
                missing.push(format!(
                    "{file} ({why}) is not in the files the limiting criterion allows; add {file} to its \
                     list, or write \"the deliverables\" there"
                ));
            }
        }
    }
    let deliverables: Vec<String> = reading
        .deliverables
        .iter()
        .map(|d| normalized_path(&d.path))
        .collect();
    let configs: Vec<&str> = CONFIG_FILES
        .into_iter()
        .filter(|c| deliverables.iter().any(|d| d == c))
        .collect();
    if !configs.is_empty() {
        let lines: Vec<&str> = reading
            .assumptions
            .iter()
            .map(|a| a.trim())
            .filter(|a| a.to_lowercase().starts_with("consumers:"))
            .collect();
        match lines.iter().find_map(|l| consumers_problem(l, &deliverables)) {
            Some(problem) => missing.push(format!(
                "{} is a deliverable and a criterion limits which files may change, so the files that load it \
                 must be allowed to change with it, but {problem}",
                configs.join(" and ")
            )),
            None if lines.is_empty() => missing.push(format!(
                "{} is a deliverable and a criterion limits which files may change, so the tests and tools \
                 that load it must be allowed to change with it, but the reading has no `Consumers:` assumption",
                configs.join(" and ")
            )),
            None => {}
        }
    }
    if missing.is_empty() {
        return Ok(());
    }
    bail!(
        "the reading was refused because a criterion that limits which files may change leaves out files the \
         job must change: {}; with a limit the deliverable list is the allowed list, and verification fails a \
         diff outside it, after the producer has finished. Repair: for a config, write an assumption \
         `Consumers: <files that load it, each also a deliverable>`, or `Consumers: none found`, or \
         `Consumers: ... unchanged`; leave off a consumer that needs no change",
        missing.join("; ")
    )
}

/// A path without surrounding blanks and a leading `./`.
fn normalized_path(path: &str) -> String {
    let path = path.trim();
    path.strip_prefix("./").unwrap_or(path).to_string()
}

/// Why a `Consumers:` line does not answer who loads the changed config, or `None` when it does: every
/// path-like word on it (other than a config file itself) is one of `deliverables` or under a deliverable
/// that ends in `/`, and the line names at least one such path or says "none found", "unchanged" or
/// "no change".
fn consumers_problem(line: &str, deliverables: &[String]) -> Option<String> {
    let mut named: Vec<String> = Vec::new();
    let mut outside: Vec<String> = Vec::new();
    for word in line
        .get("consumers:".len()..)
        .unwrap_or("")
        .split_whitespace()
    {
        let word = word.trim_start_matches(|c: char| "([{'\"`".contains(c));
        let word = word.strip_prefix("./").unwrap_or(word);
        let word = word.trim_end_matches(|c: char| ".,;:)]}'\"`".contains(c));
        if !is_path_like(word) || CONFIG_FILES.contains(&word) {
            continue;
        }
        let covered = deliverables
            .iter()
            .any(|d| d == word || (d.ends_with('/') && word.starts_with(d.as_str())));
        let list = if covered { &mut named } else { &mut outside };
        if !list.iter().any(|w| w == word) {
            list.push(word.to_string());
        }
    }
    if !outside.is_empty() {
        return Some(format!(
            "the `Consumers:` line names {} that {} not a deliverable; list each as a deliverable, or leave it \
             off the line if it needs no change",
            outside.join(", "),
            if outside.len() == 1 { "is" } else { "are" }
        ));
    }
    let lower = line.to_lowercase();
    let says_none = ["none found", "unchanged", "no change"]
        .iter()
        .any(|phrase| lower.contains(phrase));
    (named.is_empty() && !says_none).then(|| {
        "the `Consumers:` line names no deliverable path and does not say \"none found\", \"unchanged\" or \
         \"no change\""
            .to_string()
    })
}

/// Whether a word of a `Consumers:` line looks like a file path: it has a `/` (but is not "and/or") or ends
/// with a file extension of the repo's files.
fn is_path_like(word: &str) -> bool {
    word != "and/or"
        && (word.contains('/')
            || [".rs", ".toml", ".md", ".json", ".yml", ".yaml"]
                .iter()
                .any(|ext| word.ends_with(ext)))
}
