//! The owner decision cards page: `decisions.html`, built beside `index.html` by `cargo run -p xtask -- site`.
//!
//! The page lists the Steward's open proposals (work items of kind proposal that no work decision names) read
//! from the records branch `athena/records`. Each proposal becomes one card. The top of the card is what the
//! owner needs to decide, in this order: one yes/no question, an "If yes" line, an "If no" line, a cost and
//! risk line, and a "Who decides" line. Below it, a collapsed `<details>` block holds the evidence: the rung,
//! the raising role, the source, the cited records and a link to the record on GitHub at the branch tip. The
//! style is the owner's (`docs/conventions/owner-style.md`): plain words, nothing invented.
//!
//! A proposal record holds a title, a rung, a source and evidence, and nothing else: it has no question, no
//! outcome, no cost, no risk and no recommendation. So the card builds its text from those fields only:
//!
//! - A title that begins with a verb ("Retry a failed check once") is read as the change itself, and the
//!   question, the "If yes" line and the "If no" line are built around it and around what the rung changes.
//! - Any other title states a problem, not a change. The card says so plainly, and asks whether to start work
//!   on the problem instead of faking a change.
//! - The cost and risk line says "not recorded" and adds only what the record shows: what the rung reaches and
//!   how much evidence backs the proposal.
//! - No card recommends yes or no, because no record holds a recommendation. The "Who decides" line comes from
//!   the raising role's recorded limit: above it, or from a role with no recorded definition, the owner must
//!   decide; within it, Athena may make the change itself.
//!
//! Records are read with `athena_data::records::check_record_file`, so they are the same records every other
//! reader sees and no new file format exists.
//!
//! [`Brief::check`] refuses a card of more than [`MAX_WORDS`] words, a card without exactly one question, a
//! card without a "Who decides" line, and a card whose text above the details block holds Athena's internal
//! words (rung names, role names, job, work item, proposal, blind spot, commit, path). Words in a proposal's
//! title are reworded to plain ones first. The page lists a refused or unreadable record as not shown; it never
//! stops the rest of the site from being built. A checkout without the records branch gets a page that says so
//! and lists no proposals.

use crate::site::{ATHENA_URL, CSS, DECISIONS_PAGE, escape, source_url};
use anyhow::{Result, bail};
use athena_data::RecordId;
use athena_data::records::{
    AnyRecord, RefKind, Reference, WorkItem, WorkItemKind, check_record_file,
};
use athena_data::roles::{Ladder, RoleDefinition, may_change};
use athena_git::Git;
use athena_git::records_branch::{RECORDS_BRANCH, RecordsTip, resolve_tip};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

/// The most words of prose one card may show, counting every displayed line, the evidence block included.
pub const MAX_WORDS: usize = 170;

/// The prefix of the line that says who decides.
const OWNER_PREFIX: &str = "Who decides:";

/// First words that make a title read as a change ("Retry a failed check once"), in lower case.
const VERBS: &[&str] = &[
    "accept", "add", "allow", "ask", "avoid", "block", "change", "check", "close", "count",
    "demand", "drop", "end", "fix", "give", "keep", "let", "limit", "list", "log", "make", "merge",
    "move", "name", "open", "prevent", "raise", "read", "record", "refuse", "reject", "remove",
    "rename", "report", "require", "reuse", "run", "send", "set", "show", "skip", "split", "start",
    "stop", "tell", "treat", "use", "verify", "warn", "write", "retry", "rerun", "stamp", "tag",
    "teach", "print", "pin", "cap", "clear", "cut", "move", "stage",
];

/// Internal words that are replaced in titles, and refused in the text above the details block. Each pair is
/// the word and its plain replacement; the replacement never holds a listed word.
fn jargon() -> Vec<(String, String)> {
    let mut table: Vec<(String, String)> = [
        ("work items", "items"),
        ("work item", "item"),
        ("blind spots", "gaps"),
        ("blind spot", "gap"),
        ("proposals", "suggestions"),
        ("proposal", "suggestion"),
        ("jobs", "tasks"),
        ("job", "task"),
        ("commits", "saved changes"),
        ("commit", "saved change"),
        ("paths", "files"),
        ("path", "file"),
        ("steward", "helper"),
        ("scribe", "helper"),
        ("verifier", "helper"),
        ("producer", "helper"),
        ("planner", "helper"),
        ("critic", "helper"),
        ("introspector", "helper"),
    ]
    .iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect();
    for rung in Ladder::ALL {
        table.push((rung.as_str().to_string(), rung.as_str().replace('-', " ")));
    }
    table
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

/// The byte offset of the first whole-word, case-insensitive match of the lower-case ASCII `needle`.
fn find_word(text: &str, needle: &str) -> Option<usize> {
    let hay = text.to_ascii_lowercase();
    let bytes = hay.as_bytes();
    let mut from = 0;
    while let Some(pos) = hay[from..].find(needle) {
        let start = from + pos;
        let end = start + needle.len();
        let before = start == 0 || !is_word_byte(bytes[start - 1]);
        let after = end >= bytes.len() || !is_word_byte(bytes[end]);
        if before && after {
            return Some(start);
        }
        from = start + 1;
    }
    None
}

/// The first internal word that `text` holds.
fn jargon_in(text: &str) -> Option<String> {
    jargon()
        .into_iter()
        .map(|(word, _)| word)
        .find(|word| find_word(text, word).is_some())
}

/// `text` with every internal word replaced by its plain one, keeping a leading capital.
fn plain_words(text: &str) -> String {
    let mut out = text.to_string();
    for (word, plain) in jargon() {
        while let Some(start) = find_word(&out, &word) {
            let end = start + word.len();
            let capital = out[start..end].starts_with(|c: char| c.is_ascii_uppercase());
            let replacement = if capital {
                upper_first(&plain)
            } else {
                plain.clone()
            };
            out.replace_range(start..end, &replacement);
        }
    }
    out
}

fn upper_first(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|c| c.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn lower_first(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(|c| c.to_lowercase().chain(chars).collect())
        .unwrap_or_default()
}

/// What a change at `rung` changes, in plain words.
fn area(rung: Ladder) -> &'static str {
    match rung {
        Ladder::OwnerGoals => "your goals for Athena",
        Ladder::BriefRequirements => "the written requirements",
        Ladder::DesignDecisions => "the recorded design",
        Ladder::DefaultsConventions => "the default rules and conventions",
        Ladder::Implementation => "the code, tests or docs",
    }
}

/// How far a change at `rung` reaches, in plain words. This is the shape of the ladder, not a measured risk.
fn reach(rung: Ladder) -> &'static str {
    match rung {
        Ladder::OwnerGoals => "It changes your goals, so everything below them depends on it.",
        Ladder::BriefRequirements => {
            "It changes the requirements that the design and code must meet."
        }
        Ladder::DesignDecisions => "It changes the design that the rules and code follow.",
        Ladder::DefaultsConventions => "It changes the rules that new work follows.",
        Ladder::Implementation => "It changes only code, tests or docs.",
    }
}

/// The word for what a reference points at, in the evidence block.
fn kind_word(kind: RefKind) -> &'static str {
    match kind {
        RefKind::Job => "job",
        RefKind::Path => "path",
        RefKind::Commit => "commit",
    }
}

fn reference_text(reference: &Reference) -> String {
    format!("{} {}", kind_word(reference.kind), reference.value)
}

/// How many pieces of evidence there are, by kind, in plain words.
fn evidence_summary(evidence: &[Reference]) -> String {
    let count = |kind: RefKind| evidence.iter().filter(|e| e.kind == kind).count();
    let mut parts = Vec::new();
    for (kind, one, many) in [
        (RefKind::Path, "file", "files"),
        (RefKind::Job, "earlier task", "earlier tasks"),
        (RefKind::Commit, "saved change", "saved changes"),
    ] {
        let n = count(kind);
        if n > 0 {
            parts.push(format!("{n} {}", if n == 1 { one } else { many }));
        }
    }
    if parts.is_empty() {
        "none recorded".to_string()
    } else {
        parts.join(", ")
    }
}

/// The highest rung the role called `name` may change itself, by its newest recorded version.
fn role_limit(roles: &[RoleDefinition], name: &str) -> Option<Ladder> {
    roles
        .iter()
        .filter(|r| r.name == name)
        .max_by_key(|r| r.version)
        .map(|r| r.highest_rung)
}

/// The "Who decides" line, from the raising role's recorded limit. It gives no advice on the merits.
fn owner_line(item: &WorkItem, roles: &[RoleDefinition]) -> String {
    match role_limit(roles, &item.role) {
        None => format!(
            "{OWNER_PREFIX} you must decide. The limit of the role that raised it is not recorded."
        ),
        Some(limit) if may_change(limit, item.rung) => format!(
            "{OWNER_PREFIX} Athena may change this itself. The role that raised it may change up to {}.",
            area(limit)
        ),
        Some(limit) => format!(
            "{OWNER_PREFIX} you must decide. The role that raised it may change only up to {}.",
            area(limit)
        ),
    }
}

/// One proposal as a card. Every field is text the page shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Brief {
    /// The title in plain words, without a question mark or closing full stop.
    title: String,
    /// Set when the title states a problem and not a change; the plain statement of that.
    lead: Option<String>,
    question: String,
    yes_line: String,
    no_line: String,
    cost_line: String,
    owner_line: String,
    rung_line: String,
    role_line: String,
    source_line: String,
    /// One line per cited record, or the single line `Evidence: not recorded`.
    evidence_lines: Vec<String>,
    /// The record's path on the records branch.
    record_path: String,
    /// The link to that record on GitHub at the branch tip.
    record_url: String,
}

impl Brief {
    /// The card of `item`, which is stored at `record_path` on the records branch whose tip is `tip`.
    /// Refused when it fails [`Brief::check`].
    pub fn new(
        item: &WorkItem,
        record_path: &str,
        tip: &str,
        roles: &[RoleDefinition],
    ) -> Result<Brief> {
        let title = plain_words(
            item.title
                .replace('?', "")
                .trim()
                .trim_end_matches(['.', '!', ':', ';', ' ']),
        );
        let first = title
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let changes = area(item.rung);
        let (lead, question, yes_line, no_line) = if VERBS.contains(&first.as_str()) {
            let act = lower_first(&title);
            (
                None,
                format!("Should Athena {act}, and so change {changes}?"),
                format!("If yes: Athena will {act}. This changes {changes}."),
                format!(
                    "If no: Athena does not {act}. {} stay as they are.",
                    upper_first(changes)
                ),
            )
        } else {
            (
                Some(format!(
                    "The record states a problem, not a change: {title}. It does not say what to do."
                )),
                format!("Should Athena start work on this problem, and so change {changes}?"),
                format!("If yes: Athena starts work on this problem. It may change {changes}."),
                format!(
                    "If no: nothing is done about it. {} stay as they are.",
                    upper_first(changes)
                ),
            )
        };
        let evidence_lines = if item.evidence.is_empty() {
            vec!["Evidence: not recorded".to_string()]
        } else {
            item.evidence
                .iter()
                .map(|e| format!("Evidence: {}", reference_text(e)))
                .collect()
        };
        let brief = Brief {
            title,
            lead,
            question,
            yes_line,
            no_line,
            cost_line: format!(
                "Cost and risk: not recorded. {} Backed by: {}.",
                reach(item.rung),
                evidence_summary(&item.evidence)
            ),
            owner_line: owner_line(item, roles),
            rung_line: format!("Rung: {}", item.rung.as_str()),
            role_line: format!("Raised by: {}", item.role),
            source_line: format!("Source: {}", reference_text(&item.source)),
            evidence_lines,
            record_path: record_path.to_string(),
            record_url: source_url(ATHENA_URL, tip, record_path, 0, 0),
        };
        brief.check()?;
        Ok(brief)
    }

    /// The lines shown above the details block, in page order.
    fn top_lines(&self) -> Vec<&str> {
        let mut lines: Vec<&str> = self.lead.iter().map(String::as_str).collect();
        lines.extend([
            self.question.as_str(),
            &self.yes_line,
            &self.no_line,
            &self.cost_line,
            &self.owner_line,
        ]);
        lines
    }

    /// Every line of prose the card displays, in page order.
    fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = self.top_lines().into_iter().map(String::from).collect();
        lines.extend([
            self.rung_line.clone(),
            self.role_line.clone(),
            self.source_line.clone(),
        ]);
        lines.extend(self.evidence_lines.iter().cloned());
        lines.push(format!("Record: {}", self.record_path));
        lines
    }

    /// The words of prose the card displays.
    fn words(&self) -> usize {
        self.lines()
            .iter()
            .map(|l| l.split_whitespace().count())
            .sum()
    }

    /// Refuses a card that is over [`MAX_WORDS`] words, lacks a "Who decides" line, does not have exactly one
    /// question, or holds an internal word above the details block.
    fn check(&self) -> Result<()> {
        let owner_rest = self.owner_line.strip_prefix(OWNER_PREFIX).map(str::trim);
        if owner_rest.is_none_or(str::is_empty) {
            bail!(
                "the card of {:?} has no decision-owner line (\"{OWNER_PREFIX} ...\")",
                self.title
            );
        }
        let top = self.top_lines();
        let marks: usize = top.iter().map(|l| l.matches('?').count()).sum();
        if marks != 1 || !self.question.ends_with('?') {
            bail!(
                "the card of {:?} must hold exactly one question, ending in a question mark",
                self.title
            );
        }
        for line in top {
            if let Some(word) = jargon_in(line) {
                bail!(
                    "the card of {:?} uses the internal word {word:?} where the owner reads it",
                    self.title
                );
            }
        }
        let words = self.words();
        if words > MAX_WORDS {
            bail!(
                "the card of {:?} is {words} words, over the {MAX_WORDS} the owner can take in",
                self.title
            );
        }
        Ok(())
    }

    fn html(&self) -> String {
        let mut out = String::from("<article class=\"card\">");
        if let Some(lead) = &self.lead {
            let _ = write!(out, "<p class=\"lead\">{}</p>", escape(lead));
        }
        let _ = write!(
            out,
            "<h3>{}</h3><p class=\"yes\">{}</p><p class=\"no\">{}</p><p class=\"cost\">{}</p>\
             <p class=\"owner\"><strong>{}</strong></p>",
            escape(&self.question),
            escape(&self.yes_line),
            escape(&self.no_line),
            escape(&self.cost_line),
            escape(&self.owner_line)
        );
        out.push_str("<details><summary>Evidence and record</summary>");
        for line in [&self.rung_line, &self.role_line, &self.source_line] {
            let _ = write!(out, "<p>{}</p>", escape(line));
        }
        out.push_str("<ul class=\"evidence\">");
        for line in &self.evidence_lines {
            let _ = write!(out, "<li>{}</li>", escape(line));
        }
        let _ = write!(
            out,
            "</ul><p>Record: <a href=\"{}\">{}</a></p></details></article>",
            escape(&self.record_url),
            escape(&self.record_path)
        );
        out
    }
}

/// An open proposal and where its record is.
#[derive(Debug)]
struct Found {
    path: String,
    item: WorkItem,
}

/// What the records branch holds that the page needs.
#[derive(Debug)]
pub struct Snapshot {
    tip: String,
    proposals: Vec<Found>,
    roles: Vec<RoleDefinition>,
    /// Records that could not be read, each as `path: reason`.
    unreadable: Vec<String>,
}

/// Where the page's records came from.
#[derive(Debug)]
pub enum Records {
    /// The branch is not in this checkout, or could not be read; the text says why.
    Missing(String),
    Read(Snapshot),
}

/// Reads the proposals, the decisions on them and the role definitions from the records branch at `tip`.
fn read_snapshot(git: &Git, tip: &str) -> Result<Snapshot> {
    let mut proposals = Vec::new();
    let mut decided: BTreeSet<RecordId> = BTreeSet::new();
    let mut roles = Vec::new();
    let mut unreadable = Vec::new();
    for dir in [
        "records/work-items",
        "records/work-decisions",
        "records/roles",
    ] {
        for entry in git.ls_tree_under(tip, dir)? {
            if entry.kind != "blob" {
                continue;
            }
            let bytes = git.cat_blob(&entry.oid)?;
            match check_record_file(&entry.path, &bytes) {
                Ok(Some(AnyRecord::WorkItem(item))) if item.kind == WorkItemKind::Proposal => {
                    proposals.push(Found {
                        path: entry.path,
                        item: *item,
                    })
                }
                Ok(Some(AnyRecord::WorkDecision(d))) => {
                    decided.insert(d.work_item);
                }
                Ok(Some(AnyRecord::Role(role))) => roles.push(*role),
                Ok(_) => {}
                Err(e) => unreadable.push(format!("{}: {e}", entry.path)),
            }
        }
    }
    proposals.retain(|p| !decided.contains(&p.item.id));
    proposals.sort_by(|a, b| (a.item.at, &a.item.id).cmp(&(b.item.at, &b.item.id)));
    Ok(Snapshot {
        tip: tip.to_string(),
        proposals,
        roles,
        unreadable,
    })
}

/// Finds the records branch under `root` and reads it. A checkout that is not a git repository, has no
/// records branch, or cannot be read gives [`Records::Missing`], so the rest of the site is still built.
pub fn locate(root: &Path) -> Records {
    let git = Git::at(root);
    match resolve_tip(&git) {
        Ok(RecordsTip::At { tip, .. }) => match read_snapshot(&git, &tip) {
            Ok(snapshot) => Records::Read(snapshot),
            Err(e) => Records::Missing(format!("it could not be read: {e}")),
        },
        Ok(RecordsTip::Missing) => Records::Missing(format!(
            "this checkout has no {RECORDS_BRANCH} branch (fetch it before building the site)"
        )),
        Ok(RecordsTip::Diverged { .. }) => Records::Missing(
            "the local and fetched copies of it differ, and neither contains the other".to_string(),
        ),
        Err(e) => Records::Missing(format!("git could not be asked: {e}")),
    }
}

/// The page for `records`.
pub fn render(records: &Records) -> String {
    let mut body = String::from(
        "<header><h1>Proposals waiting for your decision</h1>\
         <p class=\"lede\">Each card is one yes or no question, with what happens either way and who decides. \
         Open \"Evidence and record\" to see where it came from. This page only shows them; it decides \
         nothing.</p></header>",
    );
    match records {
        Records::Missing(why) => {
            let _ = write!(
                body,
                "<section><h2>No proposals listed</h2><p>The records branch ({}) is missing here, so no \
                 proposals are listed: {}.</p></section>",
                escape(RECORDS_BRANCH),
                escape(why)
            );
        }
        Records::Read(snapshot) => {
            let mut shown = String::new();
            let mut count = 0;
            let mut refused = snapshot.unreadable.clone();
            for found in &snapshot.proposals {
                match Brief::new(&found.item, &found.path, &snapshot.tip, &snapshot.roles) {
                    Ok(brief) => {
                        count += 1;
                        shown.push_str(&brief.html());
                    }
                    Err(e) => refused.push(format!("{}: {e}", found.path)),
                }
            }
            if count == 0 {
                body.push_str("<section><h2>No proposals waiting</h2><p>No proposal is waiting for a decision.</p></section>");
            } else {
                let _ = write!(
                    body,
                    "<section><h2>{count} waiting</h2><div class=\"cards\">{shown}</div></section>"
                );
            }
            if !refused.is_empty() {
                body.push_str("<section><h2>Not shown</h2><p>These records could not be turned into a card:</p><ul>");
                for line in &refused {
                    let _ = write!(body, "<li>{}</li>", escape(line));
                }
                body.push_str("</ul></section>");
            }
            let _ = write!(
                body,
                "<footer>Records branch {} at <code>{}</code>.</footer>",
                escape(RECORDS_BRANCH),
                escape(&snapshot.tip)
            );
        }
    }
    format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>Proposals waiting for your decision</title><style>{CSS}</style></head><body><main>{body}</main></body></html>\n"
    )
}

/// Writes `decisions.html` into `out`, from the records branch of the repository at `root`.
pub fn write(root: &Path, out: &Path) -> Result<()> {
    let page = out.join(DECISIONS_PAGE);
    std::fs::write(&page, render(&locate(root)))?;
    println!("wrote {}", page.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::JobId;
    use athena_git::records_branch::{Change, RecordsWriter};

    const TIP: &str = "0123456789abcdef0123456789abcdef01234567";

    fn at() -> jiff::Timestamp {
        "2026-10-15T00:39:03Z".parse().unwrap()
    }

    fn role(name: &str, limit: Ladder) -> RoleDefinition {
        RoleDefinition {
            name: name.into(),
            version: 1,
            at: at(),
            goal: "goal".into(),
            prompt: "prompt".into(),
            cadence_tasks: 3,
            highest_rung: limit,
        }
    }

    fn item(id: &str, role: &str, title: &str, rung: Ladder, source: Reference) -> WorkItem {
        WorkItem {
            id: RecordId::parse(id).unwrap(),
            at: at(),
            role: role.into(),
            kind: WorkItemKind::Proposal,
            key: format!("key-{id}"),
            title: title.into(),
            rung,
            source,
            evidence: vec![Reference::path("README.md")],
        }
    }

    fn path_of(item: &WorkItem) -> String {
        AnyRecord::WorkItem(Box::new(item.clone())).canonical_path()
    }

    fn brief(item: &WorkItem, roles: &[RoleDefinition]) -> Brief {
        Brief::new(item, &path_of(item), TIP, roles).unwrap()
    }

    /// Two proposals that differ in title, rung, source kind and value, evidence and role.
    fn two_fixtures() -> (WorkItem, WorkItem) {
        let job = JobId::parse("20261014-101010-abcd").unwrap();
        let mut first = item(
            "20261015-003903-khz6",
            "steward",
            "Retry a failed check once",
            Ladder::DesignDecisions,
            Reference::path("docs/design/retry.md"),
        );
        first.evidence = vec![Reference::job(&job)];
        let mut second = item(
            "20261016-120000-mnp4",
            "scribe",
            "Name an owner for reviews",
            Ladder::BriefRequirements,
            Reference::job(&job),
        );
        second.evidence = vec![
            Reference::commit("abc1234"),
            Reference::path("docs/reviews.md"),
        ];
        (first, second)
    }

    /// A problem statement, as the Steward often records one: no verb first.
    fn problem_fixture() -> WorkItem {
        let mut third = item(
            "20261017-080000-qrst",
            "steward",
            "Two tasks failed the same way",
            Ladder::Implementation,
            Reference::commit("def5678"),
        );
        third.evidence = vec![
            Reference::path("a.md"),
            Reference::path("b.md"),
            Reference::path("c.md"),
        ];
        third
    }

    #[test]
    fn question_and_both_outcomes_differ_per_proposal_and_use_its_own_facts() {
        let (first, second) = two_fixtures();
        let roles = [
            role("steward", Ladder::DesignDecisions),
            role("scribe", Ladder::Implementation),
        ];
        let third = problem_fixture();
        let (a, b, c) = (
            brief(&first, &roles),
            brief(&second, &roles),
            brief(&third, &roles),
        );
        for (x, y) in [(&a, &b), (&a, &c), (&b, &c)] {
            assert_ne!(x.question, y.question);
            assert_ne!(x.yes_line, y.yes_line);
            assert_ne!(x.no_line, y.no_line);
            assert_ne!(x.cost_line, y.cost_line);
        }
        // Each line holds a fact from its own proposal: its title words, what its rung changes, its evidence.
        assert!(a.question.contains("retry a failed check once"));
        assert!(a.question.contains("the recorded design"));
        assert!(a.yes_line.contains("retry a failed check once"));
        assert!(a.no_line.contains("retry a failed check once"));
        assert!(a.cost_line.contains("1 earlier task"));
        assert!(b.question.contains("name an owner for reviews"));
        assert!(b.yes_line.contains("the written requirements"));
        assert!(b.no_line.contains("name an owner for reviews"));
        assert!(b.cost_line.contains("1 file, 1 saved change"));
        assert!(c.question.contains("the code, tests or docs"));
        assert!(c.cost_line.contains("3 files"));
        assert!(c.cost_line.contains("only code, tests or docs"));
        // A problem statement is said to be one, with its words, and is not turned into a fake change.
        let lead = c.lead.as_deref().unwrap();
        assert!(
            lead.contains("problem, not a change")
                && lead.contains("Two tasks failed the same way")
        );
        assert!(!c.question.contains("Two tasks"));
        assert!(a.lead.is_none() && b.lead.is_none());
        // No question is the title with a fixed prefix.
        for (brief, item) in [(&a, &first), (&b, &second)] {
            let title = item.title.as_str();
            for fixed in [
                "Go ahead with this: ",
                "Should Athena ",
                "Should Athena do this: ",
            ] {
                for t in [title.to_string(), lower_first(title)] {
                    assert_ne!(brief.question, format!("{fixed}{t}?"));
                }
            }
            assert!(
                !brief
                    .question
                    .ends_with(&format!("{}?", lower_first(title)))
            );
            assert!(!brief.question.ends_with(&format!("{title}?")));
        }
        // The proposal's own record fields still show in the details.
        assert_eq!(a.rung_line, "Rung: design-decisions");
        assert_eq!(a.source_line, "Source: path docs/design/retry.md");
        assert_eq!(b.source_line, "Source: job 20261014-101010-abcd");
        assert_eq!(
            b.evidence_lines,
            ["Evidence: commit abc1234", "Evidence: path docs/reviews.md"]
        );
    }

    #[test]
    fn above_the_limit_the_owner_must_decide_and_no_card_recommends() {
        let rung = Ladder::DefaultsConventions;
        let roles = [
            role("steward", Ladder::DefaultsConventions),
            role("scribe", Ladder::Implementation),
        ];
        let mk =
            |id: &str, who: &str| item(id, who, "Change a rule", rung, Reference::path("a.md"));
        let within = brief(&mk("20261015-003903-khz6", "steward"), &roles);
        let above = brief(&mk("20261015-003904-khz7", "scribe"), &roles);
        let unknown = brief(&mk("20261015-003905-khz8", "ghost"), &roles);
        assert!(
            above.owner_line.to_lowercase().contains("you must decide"),
            "{}",
            above.owner_line
        );
        assert!(
            above
                .owner_line
                .contains("only up to the code, tests or docs")
        );
        assert!(
            unknown
                .owner_line
                .to_lowercase()
                .contains("you must decide")
        );
        assert!(unknown.owner_line.contains("not recorded"));
        assert!(within.owner_line.contains("Athena may change this itself"));
        assert!(!within.owner_line.to_lowercase().contains("must decide"));
        // The newest recorded version of a role is the one that counts.
        let mut newer = role("scribe", Ladder::DefaultsConventions);
        newer.version = 2;
        let both = [role("scribe", Ladder::Implementation), newer];
        let later = brief(&mk("20261015-003904-khz7", "scribe"), &both);
        assert!(later.owner_line.contains("may change this itself"));
        // No record holds a recommendation, so no card shows one, whatever the limit.
        for card in [&within, &above, &unknown, &later] {
            let html = card.html().to_lowercase();
            assert!(!html.contains("recommend"), "{html}");
            assert!(!html.contains("recommend no"));
        }
    }

    /// The offset of `needle` in `html`.
    fn at_offset(html: &str, needle: &str) -> usize {
        html.find(needle)
            .unwrap_or_else(|| panic!("{needle} not in {html}"))
    }

    #[test]
    fn the_decision_comes_first_and_the_evidence_sits_in_a_closed_details_block() {
        let (first, second) = two_fixtures();
        let roles = [
            role("steward", Ladder::DesignDecisions),
            role("scribe", Ladder::Implementation),
        ];
        for it in [&first, &second] {
            let html = brief(it, &roles).html();
            let question = at_offset(&html, "?</h3>");
            let yes = at_offset(&html, "If yes:");
            let no = at_offset(&html, "If no:");
            let cost = at_offset(&html, "Cost and risk:");
            let owner = at_offset(&html, "Who decides:");
            let details = at_offset(&html, "<details>");
            let end = at_offset(&html, "</details>");
            assert!(!html.contains("<details open"));
            assert!(question < yes && yes < no && no < cost && cost < owner);
            let id = it.id.to_string();
            let mut evidence_starts = vec![
                at_offset(&html, "Rung:"),
                at_offset(&html, "Raised by:"),
                at_offset(&html, "Source:"),
                at_offset(&html, "Evidence:"),
                at_offset(&html, &id),
                at_offset(&html, "href="),
            ];
            evidence_starts.extend(it.evidence.iter().map(|e| at_offset(&html, &e.value)));
            for start in evidence_starts {
                assert!(owner < start, "{html}");
                assert!(details < start && start < end, "{html}");
            }
            // Nothing of the evidence is outside the block.
            assert_eq!(html.matches("Rung:").count(), 1);
            assert_eq!(html.matches("Raised by:").count(), 1);
            assert_eq!(html.matches("Source:").count(), 1);
        }
    }

    #[test]
    fn a_page_of_cards_stays_short_has_one_question_each_and_refuses_internal_words() {
        let (first, second) = two_fixtures();
        let roles = [
            role("steward", Ladder::DesignDecisions),
            role("scribe", Ladder::Implementation),
        ];
        let third = problem_fixture();
        // Real-looking titles from the Steward, full of internal words.
        let mut jargon_title = item(
            "20261018-090000-vwxy",
            "steward",
            "Stop the Steward job from missing a blind spot in a commit path?",
            Ladder::DefaultsConventions,
            Reference::path("a.md"),
        );
        jargon_title.evidence = vec![Reference::path("a.md")];
        let mut rung_title = jargon_title.clone();
        rung_title.id = RecordId::parse("20261018-090001-vwxz").unwrap();
        rung_title.title =
            "The proposal rung design-decisions is unclear to a work item reader".into();
        let proposals = [first, second, third, jargon_title, rung_title]
            .into_iter()
            .map(|item| Found {
                path: path_of(&item),
                item,
            })
            .collect();
        let page = render(&Records::Read(Snapshot {
            tip: TIP.into(),
            proposals,
            roles: roles.to_vec(),
            unreadable: Vec::new(),
        }));
        assert!(page.contains("5 waiting"), "{page}");
        let cards: Vec<&str> = page.split("<article").skip(1).collect();
        assert_eq!(cards.len(), 5);
        for card in cards {
            let card = card.split("</article>").next().unwrap();
            let mut text = String::new();
            let mut inside = false;
            for c in card.chars() {
                match c {
                    '<' => {
                        inside = true;
                        text.push(' ')
                    }
                    '>' => inside = false,
                    c if !inside => text.push(c),
                    _ => {}
                }
            }
            assert!(text.split_whitespace().count() <= MAX_WORDS + 5, "{text}");
            assert_eq!(text.matches('?').count(), 1, "{text}");
            // Above the details block no internal word shows.
            let top = card.split("<details>").next().unwrap().to_lowercase();
            for word in [
                "steward",
                "scribe",
                "job",
                "work item",
                "proposal",
                "blind spot",
                "commit",
                "path",
                "design-decisions",
                "defaults-conventions",
            ] {
                assert!(find_word(&top, word).is_none(), "{word} in {top}");
            }
        }
        assert!(page.contains("stop the Helper task from missing a gap in a saved change file"));
        assert!(!page.contains("Not shown"), "{page}");
        assert!(page.contains("design decisions"));
    }

    #[test]
    fn a_card_that_holds_an_internal_word_is_refused() {
        let roles = [role("steward", Ladder::DesignDecisions)];
        let ok = brief(
            &item(
                "20261015-003903-khz6",
                "steward",
                "Change a rule",
                Ladder::DesignDecisions,
                Reference::path("a.md"),
            ),
            &roles,
        );
        assert!(ok.check().is_ok());
        for (field, word) in [
            ("question", "Should Athena rerun the job?"),
            ("yes", "If yes: the steward does it."),
            ("no", "If no: the proposal stays open."),
            ("cost", "Cost and risk: a commit changes."),
            ("owner", "Who decides: the path is yours."),
        ] {
            let mut bad = ok.clone();
            match field {
                "question" => bad.question = word.into(),
                "yes" => bad.yes_line = word.into(),
                "no" => bad.no_line = word.into(),
                "cost" => bad.cost_line = word.into(),
                _ => bad.owner_line = word.into(),
            }
            let err = bad.check().unwrap_err();
            assert!(format!("{err}").contains("internal word"), "{field}: {err}");
        }
        // A title made only of internal words is reworded into plain ones, not shown as it was.
        let mut raw = item(
            "20261015-003903-khz6",
            "steward",
            "Verifier job blocks work item",
            Ladder::Implementation,
            Reference::path("a.md"),
        );
        raw.evidence = Vec::new();
        let plain = brief(&raw, &roles);
        assert_eq!(plain.title, "Helper task blocks item");
        // Two questions, or none, are refused.
        let mut two = ok.clone();
        two.yes_line = "If yes: why?".into();
        assert!(format!("{}", two.check().unwrap_err()).contains("exactly one question"));
        let mut none = ok.clone();
        none.question = "Do it.".into();
        assert!(none.check().is_err());
    }

    #[test]
    fn a_card_over_the_word_limit_or_without_a_decision_owner_is_refused() {
        let roles = [role("steward", Ladder::DesignDecisions)];
        let mut long = item(
            "20261015-003903-khz6",
            "steward",
            "Change a rule",
            Ladder::DesignDecisions,
            Reference::path("a.md"),
        );
        let ok = Brief::new(&long, &path_of(&long), TIP, &roles).unwrap();
        assert!(ok.words() <= MAX_WORDS);
        // The extra words are in evidence lines, not the title: every displayed line is counted.
        long.evidence = (0..40)
            .map(|n| Reference::path(format!("docs/long/file-{n}.md")))
            .collect();
        let err = Brief::new(&long, &path_of(&long), TIP, &roles).unwrap_err();
        assert!(format!("{err}").contains("words"), "{err}");
        // A title alone that is too long is refused too.
        let mut wordy = long.clone();
        wordy.evidence = vec![Reference::path("a.md")];
        wordy.title = "word ".repeat(MAX_WORDS);
        assert!(Brief::new(&wordy, &path_of(&wordy), TIP, &roles).is_err());
        // A card whose decision-owner line is empty or has no prefix is refused.
        let mut bare = ok.clone();
        bare.owner_line = String::new();
        let err = bare.check().unwrap_err();
        assert!(format!("{err}").contains("no decision-owner line"), "{err}");
        bare.owner_line = "Who decides:".into();
        assert!(bare.check().is_err());
        bare.owner_line = "Athena may change this.".into();
        assert!(bare.check().is_err());
        assert!(ok.check().is_ok());
    }

    #[test]
    fn a_proposal_with_no_evidence_says_so() {
        let mut bare = item(
            "20261015-003903-khz6",
            "steward",
            "Change a rule",
            Ladder::DesignDecisions,
            Reference::path("a.md"),
        );
        bare.evidence = Vec::new();
        let brief = brief(&bare, &[]);
        assert_eq!(brief.evidence_lines, ["Evidence: not recorded"]);
        assert!(brief.cost_line.contains("Backed by: none recorded."));
        assert!(brief.html().contains("<li>Evidence: not recorded</li>"));
    }

    fn init_repo() -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        std::fs::write(dir.path().join("a.txt"), "a").unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "init"]).unwrap();
        (dir, git)
    }

    fn put(path: String, record: AnyRecord) -> Change {
        Change::put(path, record.to_json())
    }

    #[test]
    fn the_page_links_every_record_to_github_at_the_tip_and_lists_only_open_proposals() {
        let (dir, git) = init_repo();
        let (first, second) = two_fixtures();
        let mut third = first.clone();
        third.id = RecordId::parse("20261017-080000-qrst").unwrap();
        third.title = "Remove the already decided rule".into();
        let decision = athena_data::records::WorkDecision {
            id: RecordId::parse("20261018-090000-vwxy").unwrap(),
            at: at(),
            work_item: third.id.clone(),
            decision: athena_data::records::WorkDecisionState::Rejected {
                reason: "no".into(),
            },
        };
        let steward = role("steward", Ladder::DesignDecisions);
        let mut changes = Vec::new();
        for item in [&first, &second, &third] {
            changes.push(put(
                path_of(item),
                AnyRecord::WorkItem(Box::new(item.clone())),
            ));
        }
        let decision = AnyRecord::WorkDecision(decision);
        changes.push(put(decision.canonical_path(), decision));
        let steward = AnyRecord::Role(Box::new(steward));
        changes.push(put(steward.canonical_path(), steward));
        changes.push(Change::put(
            "records/work-items/2026-10/bad.json",
            b"{}".to_vec(),
        ));
        let tip = RecordsWriter::new(&git)
            .append("records", &changes)
            .unwrap();

        let html = render(&locate(dir.path()));
        assert!(html.contains("2 waiting"), "{html}");
        assert!(html.contains("retry a failed check once"));
        assert!(html.contains("name an owner for reviews"));
        assert!(!html.contains("already decided"));
        // The unreadable record is listed as not shown and does not stop the rest.
        assert!(html.contains("Not shown") && html.contains("bad.json"));
        let hrefs: Vec<&str> = html
            .split("href=\"")
            .skip(1)
            .map(|s| s.split('"').next().unwrap())
            .collect();
        assert_eq!(hrefs.len(), 2);
        for href in hrefs {
            assert!(href.starts_with(&format!("{ATHENA_URL}/")), "{href}");
            assert!(href.contains(&tip), "{href}");
            assert!(href.contains("/records/work-items/2026-10/"), "{href}");
        }
        // Oldest first.
        assert!(html.find("retry a failed").unwrap() < html.find("name an owner").unwrap());
    }

    #[test]
    fn a_checkout_without_the_records_branch_lists_no_proposals_and_says_so() {
        let (dir, _git) = init_repo();
        let html = render(&locate(dir.path()));
        assert!(html.contains("No proposals listed"), "{html}");
        assert!(html.contains("athena/records"));
        assert!(html.contains("is missing here"));
        assert!(!html.contains("<article"));
        assert!(!html.contains("href="));
        // A directory that is not a repository is the same.
        let plain = tempfile::tempdir().unwrap();
        let html = render(&locate(plain.path()));
        assert!(html.contains("No proposals listed") && !html.contains("<article"));
    }

    #[test]
    fn a_records_branch_with_no_open_proposal_says_so() {
        let (dir, git) = init_repo();
        let steward = AnyRecord::Role(Box::new(role("steward", Ladder::DesignDecisions)));
        RecordsWriter::new(&git)
            .append("role", &[put(steward.canonical_path(), steward)])
            .unwrap();
        let html = render(&locate(dir.path()));
        assert!(html.contains("No proposal is waiting for a decision"));
    }

    /// Whether the workflow text fetches the records branch in a line before the site render step. Comment
    /// lines are ignored.
    fn fetches_records_before_render(workflow: &str) -> bool {
        let lines: Vec<&str> = workflow
            .lines()
            .map(str::trim)
            .filter(|l| !l.starts_with('#'))
            .collect();
        let fetch = lines
            .iter()
            .position(|l| l.contains("git fetch") && l.contains("refs/heads/athena/records"));
        let render = lines
            .iter()
            .position(|l| l.contains("cargo run --locked -p xtask -- site"));
        matches!((fetch, render), (Some(f), Some(r)) if f < r)
    }

    #[test]
    fn the_pages_workflow_fetches_the_records_branch_before_it_renders_the_site() {
        let fetch = "- run: git fetch --depth=1 origin +athena/records:refs/heads/athena/records\n";
        let render = "- run: cargo run --locked -p xtask -- site\n";
        assert!(fetches_records_before_render(&format!("{fetch}{render}")));
        assert!(!fetches_records_before_render(render));
        assert!(!fetches_records_before_render(&format!("{render}{fetch}")));
        assert!(!fetches_records_before_render(&format!(
            "# {fetch}{render}"
        )));
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../.github/workflows/pages.yml");
        let workflow = std::fs::read_to_string(path).unwrap();
        assert!(
            fetches_records_before_render(&workflow),
            "pages.yml must fetch athena/records into refs/heads/athena/records before the site step"
        );
    }
}
