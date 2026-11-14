//! Follow-ups named in a reading's `out_of_scope` list: when a job ends, each item that names separate work
//! ("as its own job", "as a follow-up request", "propose ... follow-up") is queued once as its own steward
//! request that cites the original job, so the work is not lost as text in the reading record.
//!
//! The words of a request come from the steward's own builder: this module only assembles the
//! [`ProposedChange::request`] string and hands it to [`RequestQueue`], which writes the request record on
//! [`REQUEST_CHANNEL`] after [`StewardRequest::from_change`] has checked the text. No record type, setting
//! or crate is added.
//!
//! Queued once per retry chain: a retry ("try again on <job>") gets a new reading whose wording drifts, so
//! each queued request carries a line `Out-of-scope item number: <first job of the chain> <n>`, and any request
//! on the steward channel that carries one for the chain stops later jobs of the chain from queueing more.

use crate::commits::NewFile;
use crate::reading::{dependency_ids_in, followed_up_ids, request_text_touches, retried_job};
use crate::steward::{Applier, ProposedChange, REQUEST_CHANNEL, RequestQueue};
use crate::store::Records;
use crate::{Athena, Ui, truncate_at_word};
use athena_data::JobId;
use athena_data::records::{Reading, Reference, WorkItemKind};
use athena_data::roles::Ladder;
use std::collections::BTreeSet;

/// The most follow-ups queued for one job and its retries.
pub const MAX_PER_JOB: usize = 5;

/// The longest title of a queued follow-up, in characters, cut at a word boundary.
const TITLE_MAX: usize = 90;

/// The longest context line taken from the original job's request, in characters.
const CONTEXT_MAX: usize = 120;

/// The most links of a retry chain followed to its first job.
const MAX_CHAIN: usize = 16;

/// The start of the line that marks a queued follow-up; the first job of the chain and the item's number in
/// the reading follow it.
///
/// The word "key" is avoided on purpose: the gate's secret scan reads `key: <random-looking id>` as a leaked
/// API key and refuses the commit.
pub const KEY_LINE: &str = "Out-of-scope item number:";

/// Words that may sit between "as" and "own" or "separate".
const DETERMINERS: [&str; 4] = ["a", "an", "its", "their"];
/// "as its own ..." and "as a separate ...".
const OWN_WORDS: [&str; 2] = ["own", "separate"];
/// What "own" or "separate" must be followed by to name work.
const WORK_NOUNS: [&str; 5] = ["job", "jobs", "request", "requests", "work"];
/// "as a follow-up", "as follow-ups".
const FOLLOW_UP_WORDS: [&str; 4] = ["follow-up", "follow-ups", "followup", "followups"];
/// "propose these ...".
const PROPOSE_WORDS: [&str; 2] = ["propose", "proposes"];
/// What a "propose" must reach, besides a follow-up word.
const PROPOSED_NOUNS: [&str; 4] = ["request", "requests", "job", "jobs"];
/// How many words after "propose" may hold the follow-up word or noun.
const PROPOSE_REACH: usize = 3;

/// An item as one line: whitespace collapsed, a leading `-`, `*` or `N.` marker dropped. Empty when nothing
/// is left.
fn normalise_item(item: &str) -> String {
    let line = item.split_whitespace().collect::<Vec<_>>().join(" ");
    let rest = line
        .strip_prefix("- ")
        .or_else(|| line.strip_prefix("* "))
        .or_else(|| {
            line.split_once(". ")
                .filter(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
                .map(|(_, rest)| rest)
        })
        .unwrap_or(&line);
    rest.trim().to_string()
}

/// The lowercase words of `text`: runs of letters, digits and `-`, so "follow-up:" is the word `follow-up`.
fn words_of(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !(c.is_alphanumeric() || c == '-'))
        .map(|w| w.trim_matches('-'))
        .filter(|w| !w.is_empty())
        .map(String::from)
        .collect()
}

/// Whether an out_of_scope item names separate work to do, by whole-word directive phrases. A bare
/// "follow-ups", "proposal" or "separate request" does not match, since those are plain words in this
/// repository's own text.
///
/// - "as" + optional determiner + ("own" or "separate") + a work noun: "as its own job";
/// - "as" + optional determiner + a follow-up word: "as a follow-up request";
/// - "propose" with a follow-up word, request or job within three words.
pub fn is_follow_up(item: &str) -> bool {
    let words = words_of(item);
    let at = |i: usize| words.get(i).map(String::as_str).unwrap_or("");
    let is_in = |word: &str, list: &[&str]| list.contains(&word);
    (0..words.len()).any(|i| {
        let word = at(i);
        if word == "as" {
            let j = if is_in(at(i + 1), &DETERMINERS) {
                i + 2
            } else {
                i + 1
            };
            return (is_in(at(j), &OWN_WORDS) && is_in(at(j + 1), &WORK_NOUNS))
                || is_in(at(j), &FOLLOW_UP_WORDS);
        }
        is_in(word, &PROPOSE_WORDS)
            && (i + 1..=i + PROPOSE_REACH)
                .any(|k| is_in(at(k), &FOLLOW_UP_WORDS) || is_in(at(k), &PROPOSED_NOUNS))
    })
}

/// The first job of the retry chain that ends at `closing`: it follows the "try again on <job>" link of each
/// job's request, as the retry preflight does, and stops at a job with no link, an unknown job, a cycle or
/// after [`MAX_CHAIN`] links.
fn chain_root(records: &Records, closing: &JobId) -> JobId {
    let mut chain = vec![closing.clone()];
    for _ in 0..MAX_CHAIN {
        let current = chain.last().expect("the chain starts with one job");
        let Some(text) = records
            .jobs
            .get(current.as_str())
            .and_then(|j| j.request.as_ref())
            .map(|r| r.text.as_str())
        else {
            break;
        };
        let Some(next) = retried_job(records, text) else {
            break;
        };
        if chain.contains(&next) || !records.jobs.contains_key(next.as_str()) {
            break;
        }
        chain.push(next);
    }
    chain.pop().expect("the chain is never empty")
}

/// Whether a line of context can go in a request without changing what the request means: it names no job as
/// a dependency or a continuation, and no path (a path word would be read as a file the request touches).
fn is_plain_context(line: &str) -> bool {
    dependency_ids_in(line).is_empty()
        && followed_up_ids(line).is_empty()
        && request_text_touches(line).is_empty()
}

/// What `queue` reads: the reading's items, the jobs involved, the original job's words and the steward
/// requests that already exist.
#[derive(Debug)]
pub struct Input<'a> {
    /// The reading's `out_of_scope` list.
    pub items: &'a [String],
    /// The job that is closing.
    pub closing: &'a JobId,
    /// The first job of the closing job's retry chain (the closing job itself when it retries nothing).
    pub root: &'a JobId,
    /// The root's title, shown on the `Original job` line when it holds no path or job link.
    pub root_title: &'a str,
    /// The first line of the root's request, shown the same way.
    pub root_request_line: &'a str,
    /// The text of every request on the steward channel that exists now.
    pub existing: &'a [String],
    /// The ids of every job that exists, so a new id never collides.
    pub taken: &'a BTreeSet<String>,
}

/// One follow-up that was queued.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Queued {
    pub job: JobId,
    /// The item's number in the reading, from 1.
    pub number: usize,
    pub title: String,
}

/// What `queue` decided.
#[derive(Debug, Default)]
pub struct Plan {
    /// The request records to commit, one per queued follow-up.
    pub files: Vec<NewFile>,
    pub jobs: Vec<Queued>,
    /// Items that were not queued although they name separate work: over the cap, or a request that could
    /// not be built. Each is one line of reason.
    pub skipped: Vec<String>,
    /// Some request already carries this chain's key, so nothing was queued.
    pub already_queued: bool,
}

/// The title of the request: the item cut at a word boundary, so it never ends inside a word. A cut that
/// would leave a path-like fragment the item does not hold gives a generic title instead.
fn title_of(item: &str, number: usize, root: &JobId) -> String {
    let title = truncate_at_word(item, TITLE_MAX);
    let item_files = request_text_touches(item);
    if request_text_touches(&title)
        .iter()
        .all(|f| item_files.contains(f))
    {
        title
    } else {
        format!("Follow-up {number} from the out_of_scope list of job {root}")
    }
}

/// The text of one request, in the form `StewardRequest::from_change` reads: the title, body lines, the
/// `Acceptance:` bullets. Paths come from the item only, since `from_change` reads every path-like word of
/// the text; context lines are added only when they hold none.
fn request_text(input: &Input<'_>, item: &str, number: usize, title: &str) -> String {
    let (closing, root) = (input.closing, input.root);
    let mut original = format!("Original job: {root}");
    let with_title = format!("{original} \u{2014} {}", input.root_title.trim());
    if !input.root_title.trim().is_empty() && is_plain_context(&with_title) {
        original = with_title;
    }
    let mut lines = vec![
        title.to_string(),
        format!("Queued from the out_of_scope list of job {closing}."),
        original,
    ];
    let request_line = truncate_at_word(input.root_request_line.trim(), CONTEXT_MAX);
    let context = format!("Original request: {request_line}");
    if !request_line.is_empty() && is_plain_context(&context) {
        lines.push(context);
    }
    lines.push(format!("Item: {item}"));
    lines.push(format!("{KEY_LINE} {root} {number}"));
    lines.push("Acceptance:".to_string());
    lines.push("- The named work is done, with no unrelated change".to_string());
    lines.push("- fmt, clippy and test pass".to_string());
    lines.join("\n")
}

/// Whether any existing request carries a key line for `root`.
fn chain_has_marker(existing: &[String], root: &JobId) -> bool {
    let start = format!("{KEY_LINE} {root} ");
    existing
        .iter()
        .any(|text| text.lines().any(|l| l.trim_start().starts_with(&start)))
}

/// Queues the follow-ups of a reading: a request per distinct item that names separate work, at most
/// [`MAX_PER_JOB`], and none when a request already carries the chain's key. An item that does not make a
/// valid request is reported in `skipped`, never an error, so the close that calls this never fails.
pub fn queue(input: &Input<'_>) -> Plan {
    let mut plan = Plan::default();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut wanted: Vec<(usize, String)> = Vec::new();
    for (index, raw) in input.items.iter().enumerate() {
        let item = normalise_item(raw);
        if item.is_empty() || !is_follow_up(&item) || !seen.insert(item.to_lowercase()) {
            continue;
        }
        wanted.push((index + 1, item));
    }
    if wanted.is_empty() {
        return plan;
    }
    if chain_has_marker(input.existing, input.root) {
        plan.already_queued = true;
        return plan;
    }
    let mut taken = input.taken.clone();
    for (number, item) in wanted {
        if plan.jobs.len() >= MAX_PER_JOB {
            plan.skipped.push(format!(
                "item {number} is over the limit of {MAX_PER_JOB} follow-ups: {}",
                truncate_at_word(&item, TITLE_MAX)
            ));
            continue;
        }
        let title = title_of(&item, number, input.root);
        let change = ProposedChange {
            kind: WorkItemKind::Proposal,
            key: format!("out-of-scope:{}:{number}", input.root),
            title: title.clone(),
            rung: Ladder::Implementation,
            source: Reference::job(input.closing),
            evidence: vec![Reference::job(input.root)],
            request: request_text(input, &item, number, &title),
            brief: None,
        };
        match RequestQueue.apply(&change, &taken) {
            Ok(queued) => {
                taken.insert(queued.job.as_str().to_string());
                plan.files.push(queued.file);
                plan.jobs.push(Queued {
                    job: queued.job,
                    number,
                    title,
                });
            }
            Err(e) => plan
                .skipped
                .push(format!("item {number} was not queued: {e}")),
        }
    }
    plan
}

impl Athena {
    /// The request records for the follow-ups of `reading`, for the closing commit of the job `closing`.
    /// Nothing is read from the records unless an item names separate work. A failure to read the records
    /// is one line to `ui` and no files: it never fails the close.
    pub(crate) fn follow_up_files(
        &self,
        closing: &JobId,
        reading: &Reading,
        ui: &dyn Ui,
    ) -> Vec<NewFile> {
        if !reading
            .out_of_scope
            .iter()
            .any(|item| is_follow_up(&normalise_item(item)))
        {
            return Vec::new();
        }
        let records = match self.head().and_then(|head| Records::load(&self.git, &head)) {
            Ok(records) => records,
            Err(e) => {
                ui.line(&format!("follow-ups were not queued: {e}"));
                return Vec::new();
            }
        };
        let root = chain_root(&records, closing);
        let existing: Vec<String> = records
            .jobs
            .values()
            .filter_map(|j| j.request.as_ref())
            .filter(|r| r.channel == REQUEST_CHANNEL)
            .map(|r| r.text.clone())
            .collect();
        let root_job = records.jobs.get(root.as_str());
        let root_title = root_job.map(|j| j.title()).unwrap_or_default();
        let root_request_line = root_job
            .and_then(|j| j.request.as_ref())
            .map(|r| crate::plan::title_line(&r.text).to_string())
            .unwrap_or_default();
        let taken: BTreeSet<String> = records.jobs.keys().cloned().collect();
        let plan = queue(&Input {
            items: &reading.out_of_scope,
            closing,
            root: &root,
            root_title: &root_title,
            root_request_line: &root_request_line,
            existing: &existing,
            taken: &taken,
        });
        if plan.already_queued {
            ui.line(&format!("follow-ups already queued for {root}"));
        }
        for queued in &plan.jobs {
            ui.line(&format!(
                "follow-up queued {}: {}",
                queued.job, queued.title
            ));
        }
        for skipped in &plan.skipped {
            ui.line(&format!("follow-up skipped: {skipped}"));
        }
        plan.files
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reading::scan_request_input;
    use athena_data::records::{AnyRecord, Request};

    const CLOSING: &str = "20261114-052244-qf6e";
    const ROOT: &str = "20261114-041123-hvyh";

    fn id(text: &str) -> JobId {
        JobId::parse(text).unwrap()
    }

    /// Runs `queue` over `items` with the given existing request texts.
    fn run_with(items: &[&str], existing: &[String], root_title: &str, root_line: &str) -> Plan {
        let items: Vec<String> = items.iter().map(|s| s.to_string()).collect();
        let (closing, root) = (id(CLOSING), id(ROOT));
        queue(&Input {
            items: &items,
            closing: &closing,
            root: &root,
            root_title,
            root_request_line: root_line,
            existing,
            taken: &BTreeSet::new(),
        })
    }

    fn run(items: &[&str]) -> Plan {
        run_with(items, &[], "", "")
    }

    /// The request text of each queued file.
    fn texts(plan: &Plan) -> Vec<String> {
        plan.files
            .iter()
            .map(|f| match AnyRecord::parse(&f.path, &f.bytes).unwrap() {
                AnyRecord::Request(Request { text, channel, .. }) => {
                    assert_eq!(channel, REQUEST_CHANNEL);
                    text
                }
                other => panic!("not a request: {other:?}"),
            })
            .collect()
    }

    /// The out_of_scope items of the job this change retries (the planner-deadline brief).
    const PLAIN: [&str; 6] = [
        "Making the planner survive its deadline (saving draft plans, a planning-timeout state, resume on retry, sizing requests before dispatch): a separate request from introspection 20261114-052207-pj25.",
        "Backfilling the follow-ups already lost from job 20261114-034034-fgwz, or fixing its other items (queue order, October backlog hold, closing 20261017-205552-y1bh and 20261030-060410-9e5h, plan critic memory, review findings 1 and 3-13).",
        "The three verifier findings on 20261114-034034-fgwz in crates/athena-core/src/hestia.rs; hestia.rs stays unchanged.",
        "New refusal rules, changes to the reading prompts (the reader prompt is at its token limit), athena.toml, defaults.toml, workflows, docs/decisions.md and records/.",
        "Reprioritizing queued follow-ups ahead of or behind other work.",
        "athena.toml, defaults.toml, workflows, docs/decisions.md and records/",
    ];

    #[test]
    fn followups_queue_one_request_per_follow_up_item_pure() {
        for item in PLAIN {
            assert!(run(&[item]).files.is_empty(), "{item}");
        }
        let items = [
            "Propose these as follow-up requests: queue order and the backlog hold",
            "Plan critic memory runs as its own job",
            "Propose these as follow-up requests: closing 20261017-205552-y1bh and 20261030-060410-9e5h",
            "- Backlog hold runs as its own request",
            "2. Treat the retry state as a separate job",
        ];
        for item in items {
            let plan = run(&[item]);
            assert_eq!(plan.files.len(), 1, "{item}");
            assert_eq!(plan.jobs.len(), 1, "{item}");
        }
        let plan = run(&[PLAIN[0], items[0], PLAIN[3], items[1], items[2], ""]);
        assert_eq!(plan.files.len(), 3);
        let texts = texts(&plan);
        for text in &texts {
            assert!(text.contains(CLOSING), "{text}");
            assert!(text.contains(ROOT), "{text}");
            assert!(text.contains(&format!("{KEY_LINE} {ROOT} ")), "{text}");
            assert!(!crate::reading::request_text_acceptance(text).is_empty());
            assert!(!request_text_touches(text).is_empty());
        }
        assert!(
            texts[2].contains("20261017-205552-y1bh") && texts[2].contains("20261030-060410-9e5h")
        );
        // The number is the item's place in the reading, from 1.
        assert!(texts[0].contains(&format!("{KEY_LINE} {ROOT} 2")));
        assert!(texts[1].contains(&format!("{KEY_LINE} {ROOT} 4")));
    }

    #[test]
    fn followups_ignore_plain_exclusions_and_an_empty_out_of_scope_pure() {
        let plan = run(&[
            "athena.toml, defaults.toml, workflows, docs/decisions.md and records/",
            "the changelog",
            "Changes to the proposal record format",
            "A separate request about the planner",
            "Queue order and the backlog hold",
            "",
            "   ",
        ]);
        assert!(plan.files.is_empty() && plan.jobs.is_empty());
        assert!(plan.skipped.is_empty() && !plan.already_queued);
        let none = run(&[]);
        assert!(none.files.is_empty() && none.skipped.is_empty() && !none.already_queued);
    }

    #[test]
    fn followups_are_queued_once_and_capped_pure() {
        let three = [
            "Propose queue order as a follow-up request",
            "Critic memory runs as its own job",
            "Backlog hold runs as its own request",
        ];
        let first = run(&three);
        assert_eq!(first.files.len(), 3);
        let existing = texts(&first);
        // The same items again, and different wording under the same chain, queue nothing.
        let again = run_with(&three, &existing, "", "");
        assert!(again.already_queued && again.files.is_empty());
        let reworded = run_with(
            &[
                "Planner deadline runs as its own job",
                "Propose a hold as a follow-up",
            ],
            &existing,
            "",
            "",
        );
        assert!(reworded.already_queued && reworded.files.is_empty());
        // A reading with no follow-up item is not "already queued": there is nothing to report.
        assert!(!run_with(&["Other files"], &existing, "", "").already_queued);
        // A marker for another chain does not stop this one.
        let other = vec![format!("Title\n\n{KEY_LINE} 20260101-000000-abcd 1\n")];
        assert_eq!(run_with(&three, &other, "", "").files.len(), 3);
        // Seven distinct items queue five and report two.
        let seven: Vec<String> = (1..=7)
            .map(|n| format!("Piece {n} runs as its own job"))
            .collect();
        let refs: Vec<&str> = seven.iter().map(String::as_str).collect();
        let capped = run(&refs);
        assert_eq!(capped.files.len(), MAX_PER_JOB);
        assert_eq!(capped.skipped.len(), 2);
        // The same item twice (case and spacing aside) is queued once.
        let repeated = run(&["Memory runs as its own job", "memory  runs as its own JOB"]);
        assert_eq!(repeated.files.len(), 1);
    }

    #[test]
    fn followups_files_come_from_the_item_only_pure() {
        let item = "Plan critic memory runs as its own job";
        let plan = run_with(
            &[item],
            &[],
            "Edit athena.toml and crates/athena-core/src/run.rs",
            "Change docs/decisions.md and src/x.rs please",
        );
        let text = &texts(&plan)[0];
        assert_eq!(request_text_touches(text), vec!["records/".to_string()]);
        assert!(text.contains("Files: records/"), "{text}");
        assert!(!text.contains("athena.toml") && !text.contains("Original request"));
        // The item names a file: only it is a file.
        let item = "Rename it in crates/athena-core/src/steward.rs as its own job";
        let text = &texts(&run_with(&[item], &[], "Edit athena.toml", ""))[0];
        assert_eq!(
            request_text_touches(text),
            vec!["crates/athena-core/src/steward.rs".to_string()]
        );
        // A title cut at the 90th character never ends inside a word, so a path sits whole or not at all.
        let long = format!(
            "{} src/some/very/long/path/name.rs runs as its own job",
            "word ".repeat(17)
        );
        let text = &texts(&run(&[&long]))[0];
        assert_eq!(request_text_touches(text), request_text_touches(&long));
        let title = text.lines().next().unwrap();
        assert!(
            title.chars().count() <= TITLE_MAX && !title.contains("src/"),
            "{title}"
        );
        // One word longer than the cut falls back to a generic title when its cut part reads as a path.
        let word = format!("{}/{} runs as its own job", "d".repeat(85), "e".repeat(30));
        let text = &texts(&run(&[&word]))[0];
        assert!(text.starts_with("Follow-up 1 from"), "{text}");
        // Clean context lines are kept.
        let text = &texts(&run_with(
            &[item],
            &[],
            "Queue the follow-ups",
            "Queue follow-ups of jobs",
        ))[0];
        assert!(text.contains("Original job: 20261114-041123-hvyh \u{2014} Queue the follow-ups"));
        assert!(text.contains("Original request: Queue follow-ups of jobs"));
    }

    #[test]
    fn followups_text_is_a_reference_not_a_dependency_pure() {
        let hostile = format!("Follow up on job {CLOSING} with Depends on {CLOSING}");
        let plan = run_with(
            &["Propose these as follow-up requests: closing 20261017-205552-y1bh"],
            &[],
            &hostile,
            &format!("try again on {CLOSING}"),
        );
        let records_free = texts(&plan);
        for text in &records_free {
            assert!(dependency_ids_in(text).is_empty(), "{text}");
            assert!(followed_up_ids(text).is_empty(), "{text}");
            assert!(!text.contains("Follow up on"), "{text}");
            assert!(text.contains(&format!("Original job: {ROOT}")));
        }
    }

    #[test]
    fn followups_reword_banned_words_pure() {
        let word = athena_context::briefs::boundary_words()
            .into_iter()
            .next()
            .unwrap();
        let fragment = athena_context::briefs::boundary_fragments()
            .into_iter()
            .next()
            .unwrap();
        let reworded = format!("The {word} runs as its own job");
        let plan = run(&[&reworded]);
        assert_eq!(plan.files.len(), 1);
        for text in texts(&plan) {
            assert!(scan_request_input(&text).is_empty(), "{text}");
        }
        // A path that holds a banned fragment cannot be reworded: that item is skipped, the rest is queued.
        let bad_path = format!("a{fragment}/x.rs runs as its own job");
        let plan = run(&[&bad_path, "Memory runs as its own job"]);
        assert_eq!(plan.files.len(), 1);
        assert_eq!(plan.skipped.len(), 1);
    }

    #[test]
    fn the_matcher_reads_whole_directive_phrases() {
        for yes in [
            "Runs as its own job",
            "as their own requests",
            "Do this as a separate job",
            "queued as a follow-up",
            "as follow-ups",
            "proposes a follow-up",
        ] {
            assert!(is_follow_up(yes), "{yes}");
        }
        for no in [
            "follow-ups",
            "the proposal",
            "a separate request",
            "as an example",
            "proposal for a new rule",
            "propose",
            "Reprioritizing queued follow-ups ahead of other work",
        ] {
            assert!(!is_follow_up(no), "{no}");
        }
    }
}
