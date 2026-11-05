//! The owner decision briefs page: `decisions.html`, built beside `index.html` by `cargo run -p xtask -- site`.
//!
//! The page lists only the proposals that wait for the owner: work items of kind proposal that no work
//! decision names and whose rung is above the raising role's recorded limit (or whose role has no recorded
//! definition). They are read from the records branch `athena/records`. A proposal the role may change itself
//! is not listed. With none left, the page says "Nothing waiting for you". Work decisions are read from both
//! places: the records branch and the tree of main (the checkout's HEAD). Approve-work and reject-work record
//! their decisions under `records/work-decisions/` on main, which the records branch does not hold, so
//! reading the branch alone left every proposal looking undecided. Each proposal becomes a short brief: its
//! title, rung, source, evidence, a recommendation, and a link to the record on GitHub at the branch tip. The
//! style is the owner's (`docs/conventions/owner-style.md`): plain words, nothing invented.
//!
//! The words of a card are written by the steward, not by this page: when it raises a proposal above its
//! limit it stores five plain-language fields in the work item (question, if_yes, if_no, cost, recommendation).
//! The page shows them first, as stored, and puts the rung, source, evidence and record link in a collapsed
//! section. A proposal without them shows its title and "No brief written yet". The page only escapes and lays
//! out stored text; it never rewrites a title.
//!
//! Records are read with `athena_data::records::check_record_file`, so they are the same records every other
//! reader sees and no new file format exists. A brief shows only what the proposal's fields hold: there is no
//! cost, risk or approval wording. The recommendation comes from the raising role's recorded limit: a proposal
//! at or below it is something the role may change itself; one above it, or from a role with no recorded
//! definition, is for the owner to decide.
//!
//! A brief of more than [`MAX_WORDS`] words, or without a recommendation line, is refused. The page lists a
//! refused or unreadable record as not shown; it never stops the rest of the site from being built. A checkout
//! without the records branch gets a page that says so and lists no proposals.

use crate::site::{ATHENA_URL, CSS, DECISIONS_PAGE, escape, source_url};
use anyhow::{Result, bail};
use athena_data::RecordId;
use athena_data::records::{
    AnyRecord, OwnerBrief, RefKind, Reference, WorkItem, WorkItemKind, check_record_file,
};
use athena_data::roles::{Ladder, RoleDefinition, may_change};
use athena_git::Git;
use athena_git::records_branch::{RECORDS_BRANCH, RecordsTip, resolve_tip};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

/// The most words of prose one brief may show, counting every displayed line.
pub const MAX_WORDS: usize = 160;

/// The tree of main in the checkout (its HEAD), where approve-work and reject-work record their decisions.
const MAIN_TREE: &str = "HEAD";

/// What a card says when the steward wrote no brief for the proposal.
const NO_BRIEF: &str = "No brief written yet";

/// One proposal as a short brief. Every field is text the page shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Brief {
    title: String,
    rung_line: String,
    role_line: String,
    source_line: String,
    /// One line per cited record, or the single line `Evidence: not recorded`.
    evidence_lines: Vec<String>,
    /// The derived recommendation line, from the raising role's limit.
    recommendation: String,
    /// The five fields the steward wrote, shown as stored. None for a proposal raised without them.
    owner: Option<OwnerBrief>,
    /// The record's path on the records branch.
    record_path: String,
    /// The link to that record on GitHub at the branch tip.
    record_url: String,
}

/// The word for what a reference points at.
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

/// The highest rung the role called `name` may change itself, by its newest recorded version.
fn role_limit(roles: &[RoleDefinition], name: &str) -> Option<Ladder> {
    roles
        .iter()
        .filter(|r| r.name == name)
        .max_by_key(|r| r.version)
        .map(|r| r.highest_rung)
}

/// Whether `item` is for the owner to decide: its rung is above the raising role's recorded limit, or the
/// role has no recorded definition. A proposal the role may change itself is not listed.
fn needs_owner(item: &WorkItem, roles: &[RoleDefinition]) -> bool {
    role_limit(roles, &item.role).is_none_or(|limit| !may_change(limit, item.rung))
}

/// The recommendation line, from the raising role's recorded limit.
fn recommendation(item: &WorkItem, roles: &[RoleDefinition]) -> String {
    let rung = item.rung.as_str();
    match role_limit(roles, &item.role) {
        None => format!(
            "Recommendation: Decide yourself. The role is not recorded ({} has no definition), so its limit is unknown.",
            item.role
        ),
        Some(limit) if may_change(limit, item.rung) => format!(
            "Recommendation: the {} role may change this itself. {rung} is within its limit ({}).",
            item.role,
            limit.as_str()
        ),
        Some(limit) => format!(
            "Recommendation: Decide yourself. {rung} is above the limit of the {} role ({}).",
            item.role,
            limit.as_str()
        ),
    }
}

impl Brief {
    /// The brief of `item`, which is stored at `record_path` on the records branch whose tip is `tip`.
    /// Refused when it is too long or has no recommendation.
    pub fn new(
        item: &WorkItem,
        record_path: &str,
        tip: &str,
        roles: &[RoleDefinition],
    ) -> Result<Brief> {
        let evidence_lines = if item.evidence.is_empty() {
            vec!["Evidence: not recorded".to_string()]
        } else {
            item.evidence
                .iter()
                .map(|e| format!("Evidence: {}", reference_text(e)))
                .collect()
        };
        let brief = Brief {
            title: item.title.clone(),
            rung_line: format!("Rung: {}", item.rung.as_str()),
            role_line: format!("Raised by: {}", item.role),
            source_line: format!("Source: {}", reference_text(&item.source)),
            evidence_lines,
            recommendation: recommendation(item, roles),
            owner: item.brief.clone(),
            record_path: record_path.to_string(),
            record_url: source_url(ATHENA_URL, tip, record_path, 0, 0),
        };
        brief.check()?;
        Ok(brief)
    }

    /// Every line of prose the brief displays, in page order.
    fn lines(&self) -> Vec<String> {
        let mut lines = vec![
            self.title.clone(),
            self.rung_line.clone(),
            self.role_line.clone(),
            self.source_line.clone(),
        ];
        lines.extend(self.evidence_lines.iter().cloned());
        match &self.owner {
            Some(o) => lines.extend([
                o.question.clone(),
                o.if_yes.clone(),
                o.if_no.clone(),
                o.cost.clone(),
                o.recommendation.clone(),
            ]),
            None => lines.push(NO_BRIEF.to_string()),
        }
        lines.push(self.recommendation.clone());
        lines.push(format!("Record: {}", self.record_path));
        lines
    }

    /// The words of prose the brief displays.
    fn words(&self) -> usize {
        self.lines()
            .iter()
            .map(|l| l.split_whitespace().count())
            .sum()
    }

    /// Refuses a brief that is over [`MAX_WORDS`] words or has no recommendation line.
    fn check(&self) -> Result<()> {
        if !self.recommendation.starts_with("Recommendation:")
            || self.recommendation.trim() == "Recommendation:"
        {
            bail!("the brief of {:?} has no recommendation line", self.title);
        }
        let words = self.words();
        if words > MAX_WORDS {
            bail!(
                "the brief of {:?} is {words} words, over the {MAX_WORDS} the owner can take in",
                self.title
            );
        }
        Ok(())
    }

    /// The card: the title and the steward's five fields first, then the evidence, source and record link in
    /// a collapsed section. The stored text is escaped and laid out as it was written.
    fn html(&self) -> String {
        let mut out = format!("<article class=\"card\"><h3>{}</h3>", escape(&self.title));
        match &self.owner {
            Some(o) => {
                let _ = write!(
                    out,
                    "<p class=\"question\"><strong>{}</strong></p>\
                     <p>If yes: {}</p><p>If no: {}</p><p>Cost: {}</p>\
                     <p class=\"recommendation\"><strong>Recommendation: {}</strong></p>",
                    escape(&o.question),
                    escape(&o.if_yes),
                    escape(&o.if_no),
                    escape(&o.cost),
                    escape(&o.recommendation)
                );
            }
            None => {
                let _ = write!(out, "<p class=\"no-brief\">{NO_BRIEF}</p>");
            }
        }
        out.push_str("<details><summary>Evidence and record</summary>");
        for line in [&self.rung_line, &self.role_line, &self.source_line] {
            let _ = write!(out, "<p>{}</p>", escape(line));
        }
        out.push_str("<ul class=\"evidence\">");
        for line in &self.evidence_lines {
            let _ = write!(out, "<li>{}</li>", escape(line));
        }
        out.push_str("</ul>");
        if self.owner.is_none() {
            let _ = write!(out, "<p>{}</p>", escape(&self.recommendation));
        }
        let _ = write!(
            out,
            "<p>Record: <a href=\"{}\">{}</a></p></details></article>",
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
    // Approve and reject decisions are committed to main, not to the records branch, so main's tree is read
    // too. A checkout with no commit, or no decisions directory on main, adds nothing.
    for entry in git
        .ls_tree_under(MAIN_TREE, "records/work-decisions")
        .unwrap_or_default()
    {
        if entry.kind != "blob" {
            continue;
        }
        let bytes = git.cat_blob(&entry.oid)?;
        match check_record_file(&entry.path, &bytes) {
            Ok(Some(AnyRecord::WorkDecision(d))) => {
                decided.insert(d.work_item);
            }
            Ok(_) => {}
            Err(e) => unreadable.push(format!("{}: {e}", entry.path)),
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
         <p class=\"lede\">Each card is one proposal. It shows what it changes, where it came from, the \
         evidence, and a recommendation. This page only shows them; it decides nothing.</p></header>",
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
                if !needs_owner(&found.item, &snapshot.roles) {
                    continue;
                }
                match Brief::new(&found.item, &found.path, &snapshot.tip, &snapshot.roles) {
                    Ok(brief) => {
                        count += 1;
                        shown.push_str(&brief.html());
                    }
                    Err(e) => refused.push(format!("{}: {e}", found.path)),
                }
            }
            if count == 0 {
                body.push_str("<section><p>Nothing waiting for you</p></section>");
            } else {
                let _ = write!(
                    body,
                    "<section><h2>{count} waiting</h2><div class=\"cards\">{shown}</div></section>"
                );
            }
            if !refused.is_empty() {
                body.push_str("<section><h2>Not shown</h2><p>These records could not be turned into a brief:</p><ul>");
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
            brief: None,
        }
    }

    fn owner_brief() -> OwnerBrief {
        OwnerBrief {
            question: "Should the steward retry a failed check once?".into(),
            if_yes: "A failed check runs a second time.".into(),
            if_no: "A failed check stops the job at once.".into(),
            cost: "About one more minute per failure.".into(),
            recommendation: "Yes: most failures here pass on the second run.".into(),
        }
    }

    #[test]
    fn a_steward_brief_is_shown_before_any_path_and_the_paths_are_collapsed() {
        let roles = [role("steward", Ladder::DefaultsConventions)];
        let mut proposal = item(
            "20261015-003903-khz6",
            "steward",
            "Retry a failed check once",
            Ladder::DesignDecisions,
            Reference::path("docs/design/retry.md"),
        );
        proposal.evidence = vec![Reference::path("docs/evidence-one.md")];
        proposal.brief = Some(owner_brief());
        let html = brief(&proposal, &roles).html();
        let first_path = html.find("docs/design/retry.md").unwrap();
        let first_path = first_path.min(html.find("docs/evidence-one.md").unwrap());
        let b = owner_brief();
        for text in [&b.question, &b.if_yes, &b.if_no, &b.cost, &b.recommendation] {
            let at = html
                .find(text.as_str())
                .unwrap_or_else(|| panic!("{text}: {html}"));
            assert!(at < first_path, "{text} comes after a path");
        }
        assert!(!html.contains("No brief written yet"));
        let details = html.find("<details>").unwrap();
        assert!(details < first_path);
        assert!(first_path < html.find("</details>").unwrap());
        assert!(html.contains("<p>Source: path docs/design/retry.md</p>"));
        // The derived limit line is replaced by the steward's recommendation.
        assert!(!html.contains("Decide yourself"));
    }

    #[test]
    fn a_proposal_without_a_brief_shows_its_title_and_says_none_is_written() {
        let proposal = item(
            "20261015-003903-khz6",
            "steward",
            "Retry a failed check once",
            Ladder::DesignDecisions,
            Reference::path("docs/design/retry.md"),
        );
        let html = brief(&proposal, &[]).html();
        assert!(html.contains("<h3>Retry a failed check once</h3>"));
        assert!(html.contains("No brief written yet"));
        assert!(html.find("No brief written yet").unwrap() < html.find("<details>").unwrap());
        // The whole page renders, and the card is not listed as refused.
        let snapshot = Snapshot {
            tip: TIP.into(),
            proposals: vec![Found {
                path: path_of(&proposal),
                item: proposal,
            }],
            roles: Vec::new(),
            unreadable: Vec::new(),
        };
        let page = render(&Records::Read(snapshot));
        assert!(page.contains("1 waiting") && page.contains("No brief written yet"));
        assert!(!page.contains("Not shown"));
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

    #[test]
    fn each_brief_shows_its_own_proposal_and_invents_nothing() {
        let (first, second) = two_fixtures();
        let roles = [
            role("steward", Ladder::DesignDecisions),
            role("scribe", Ladder::Implementation),
        ];
        let (a, b) = (brief(&first, &roles), brief(&second, &roles));
        assert_eq!(a.title, "Retry a failed check once");
        assert_eq!(a.rung_line, "Rung: design-decisions");
        assert_eq!(a.source_line, "Source: path docs/design/retry.md");
        assert_eq!(a.evidence_lines, ["Evidence: job 20261014-101010-abcd"]);
        assert_eq!(b.title, "Name an owner for reviews");
        assert_eq!(b.rung_line, "Rung: brief-requirements");
        assert_eq!(b.source_line, "Source: job 20261014-101010-abcd");
        assert_eq!(
            b.evidence_lines,
            ["Evidence: commit abc1234", "Evidence: path docs/reviews.md"]
        );
        // A fixed source line would equal in both; each field differs between the briefs.
        assert_ne!(a.title, b.title);
        assert_ne!(a.rung_line, b.rung_line);
        assert_ne!(a.source_line, b.source_line);
        assert_ne!(a.evidence_lines, b.evidence_lines);
        let (html_a, html_b) = (a.html(), b.html());
        assert!(html_a.contains("<p>Source: path docs/design/retry.md</p>"));
        assert!(!html_a.contains("Source: job"));
        assert!(html_b.contains("<p>Source: job 20261014-101010-abcd</p>"));
        assert!(!html_b.contains("Source: path"));
        for html in [html_a, html_b] {
            let lower = html.to_lowercase();
            for word in ["cost", "risk", "approve"] {
                assert!(!lower.contains(word), "{word}");
            }
        }
    }

    #[test]
    fn the_recommendation_follows_the_raising_roles_recorded_limit() {
        // Same rung, three roles: one within its limit, one above it, one not recorded.
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
        assert_eq!(
            within.recommendation,
            "Recommendation: the steward role may change this itself. defaults-conventions is within its limit (defaults-conventions)."
        );
        assert_eq!(
            above.recommendation,
            "Recommendation: Decide yourself. defaults-conventions is above the limit of the scribe role (implementation)."
        );
        assert_eq!(
            unknown.recommendation,
            "Recommendation: Decide yourself. The role is not recorded (ghost has no definition), so its limit is unknown."
        );
        assert!(!within.recommendation.contains("Decide yourself"));
        // The newest recorded version of a role is the one that counts.
        let mut newer = role("scribe", Ladder::DefaultsConventions);
        newer.version = 2;
        let both = [role("scribe", Ladder::Implementation), newer];
        let item = mk("20261015-003904-khz7", "scribe");
        assert!(brief(&item, &both).recommendation.contains("may change"));
    }

    #[test]
    fn a_brief_over_the_word_limit_or_without_a_recommendation_is_refused() {
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
        long.evidence = (0..60)
            .map(|n| Reference::path(format!("docs/long/file-{n}.md")))
            .collect();
        let err = Brief::new(&long, &path_of(&long), TIP, &roles).unwrap_err();
        assert!(format!("{err}").contains("words"), "{err}");
        // A title alone that is too long is refused too.
        let mut wordy = long.clone();
        wordy.evidence = vec![Reference::path("a.md")];
        wordy.title = "word ".repeat(MAX_WORDS);
        assert!(Brief::new(&wordy, &path_of(&wordy), TIP, &roles).is_err());
        // A brief whose recommendation line is empty is refused.
        let mut bare = ok.clone();
        bare.recommendation = String::new();
        let err = bare.check().unwrap_err();
        assert!(format!("{err}").contains("no recommendation"), "{err}");
        bare.recommendation = "Recommendation:".into();
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
        third.title = "Already decided".into();
        let decision = athena_data::records::WorkDecision {
            id: RecordId::parse("20261018-090000-vwxy").unwrap(),
            at: at(),
            work_item: third.id.clone(),
            decision: athena_data::records::WorkDecisionState::Rejected {
                reason: "no".into(),
            },
        };
        // The steward's limit is below the first proposal's rung, so that proposal is for the owner.
        let steward = role("steward", Ladder::DefaultsConventions);
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
        assert!(html.contains("Retry a failed check once"));
        assert!(html.contains("Name an owner for reviews"));
        assert!(!html.contains("Already decided"));
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
        assert!(html.find("Retry a failed").unwrap() < html.find("Name an owner").unwrap());
    }

    /// Commits `record` to main of the repository (not to the records branch).
    fn commit_to_main(dir: &Path, git: &Git, record: &AnyRecord) {
        let path = dir.join(record.canonical_path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, record.to_json()).unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "decide"]).unwrap();
    }

    fn decision_of(id: &str, work_item: &RecordId, approved: bool) -> AnyRecord {
        use athena_data::records::{WorkDecision, WorkDecisionState};
        AnyRecord::WorkDecision(WorkDecision {
            id: RecordId::parse(id).unwrap(),
            at: at(),
            work_item: work_item.clone(),
            decision: if approved {
                WorkDecisionState::Approved {
                    job: JobId::parse("20261030-081746-a1he").unwrap(),
                }
            } else {
                WorkDecisionState::Rejected {
                    reason: "not now".into(),
                }
            },
        })
    }

    /// Two proposals on the records branch; decisions are committed to main only.
    fn decided_on_main(approve: bool, reject: bool) -> (tempfile::TempDir, WorkItem, WorkItem) {
        let (dir, git) = init_repo();
        let (first, second) = two_fixtures();
        let changes: Vec<Change> = [&first, &second]
            .iter()
            .map(|i| put(path_of(i), AnyRecord::WorkItem(Box::new((*i).clone()))))
            .collect();
        RecordsWriter::new(&git)
            .append("records", &changes)
            .unwrap();
        if approve {
            let d = decision_of("20261030-081746-a1he", &first.id, true);
            commit_to_main(dir.path(), &git, &d);
        }
        if reject {
            let d = decision_of("20261030-081747-b2jf", &second.id, false);
            commit_to_main(dir.path(), &git, &d);
        }
        (dir, first, second)
    }

    /// A regression guard: the base already drops proposals decided on main, so this passes there too.
    #[test]
    fn proposals_decided_on_main_drop_off_the_page() {
        let (dir, first, second) = decided_on_main(true, true);
        // The decisions are not on the records branch.
        let git = Git::isolated(dir.path());
        let tip = match resolve_tip(&git).unwrap() {
            RecordsTip::At { tip, .. } => tip,
            other => panic!("{other:?}"),
        };
        assert!(
            git.ls_tree_under(&tip, "records/work-decisions")
                .unwrap()
                .is_empty()
        );
        let html = render(&locate(dir.path()));
        assert!(!html.contains(first.id.as_str()), "{html}");
        assert!(!html.contains(second.id.as_str()), "{html}");
        assert!(!html.contains("<article"), "{html}");
        assert!(html.contains("Nothing waiting for you"));
    }

    #[test]
    fn a_proposal_with_no_decision_on_main_is_still_listed() {
        // Only the first proposal is approved; the second has no decision anywhere.
        let (dir, first, second) = decided_on_main(true, false);
        let html = render(&locate(dir.path()));
        assert_eq!(html.matches("<article").count(), 1, "{html}");
        assert!(html.contains("1 waiting"));
        assert!(html.contains("Name an owner for reviews"));
        assert!(html.contains(second.id.as_str()));
        assert!(!html.contains(first.id.as_str()));
    }

    /// A proposal by the steward (limit: defaults-conventions) on `rung`, titled `title`.
    fn steward_item(id: &str, title: &str, rung: Ladder) -> WorkItem {
        item(id, "steward", title, rung, Reference::path("a.md"))
    }

    /// A temp repository whose records branch holds `items` and the steward role. An approval is written
    /// where approve-work writes it (the records branch); a rejection where reject-work writes it (the
    /// current branch, main).
    fn page_with(items: &[&WorkItem], approved: &[&WorkItem], rejected: &[&WorkItem]) -> String {
        let (dir, git) = init_repo();
        let steward = AnyRecord::Role(Box::new(role("steward", Ladder::DefaultsConventions)));
        let mut changes = vec![put(steward.canonical_path(), steward)];
        for i in items {
            changes.push(put(path_of(i), AnyRecord::WorkItem(Box::new((*i).clone()))));
        }
        for (n, i) in approved.iter().enumerate() {
            let d = decision_of(&format!("20261027-074524-ap{n:02}"), &i.id, true);
            changes.push(put(d.canonical_path(), d));
        }
        RecordsWriter::new(&git)
            .append("records", &changes)
            .unwrap();
        for (n, i) in rejected.iter().enumerate() {
            let d = decision_of(&format!("20261027-074525-rj{n:02}"), &i.id, false);
            commit_to_main(dir.path(), &git, &d);
        }
        render(&locate(dir.path()))
    }

    #[test]
    fn with_only_decided_and_within_limit_proposals_the_page_says_nothing_is_waiting() {
        let approved = steward_item(
            "20261015-003901-aaaa",
            "Approved one",
            Ladder::DesignDecisions,
        );
        let rejected = steward_item(
            "20261015-003902-bbbb",
            "Rejected one",
            Ladder::DesignDecisions,
        );
        let within = steward_item(
            "20261015-003903-cccc",
            "Within limit",
            Ladder::DefaultsConventions,
        );
        let html = page_with(&[&approved, &rejected, &within], &[&approved], &[&rejected]);
        assert!(html.contains("Nothing waiting for you"), "{html}");
        assert!(!html.contains("<article"), "{html}");
        assert!(!html.contains("waiting</h2>"), "{html}");
        for title in ["Approved one", "Rejected one", "Within limit"] {
            assert!(!html.contains(title), "{title}");
        }
    }

    #[test]
    fn only_the_open_above_limit_proposal_is_a_card() {
        let approved = steward_item(
            "20261015-003901-aaaa",
            "Approved one",
            Ladder::DesignDecisions,
        );
        let rejected = steward_item(
            "20261015-003902-bbbb",
            "Rejected one",
            Ladder::DesignDecisions,
        );
        let above = steward_item(
            "20261015-003903-cccc",
            "Above limit",
            Ladder::DesignDecisions,
        );
        let below = steward_item(
            "20261015-003904-dddd",
            "Below limit",
            Ladder::Implementation,
        );
        let html = page_with(
            &[&approved, &rejected, &above, &below],
            &[&approved],
            &[&rejected],
        );
        assert_eq!(html.matches("<article").count(), 1, "{html}");
        assert!(html.contains("<h2>1 waiting</h2>"), "{html}");
        assert!(html.contains("Above limit"));
        for title in ["Approved one", "Rejected one", "Below limit"] {
            assert!(!html.contains(title), "{title}");
        }
        assert!(!html.contains("Nothing waiting for you"));
    }

    #[test]
    fn a_proposal_from_a_role_with_no_recorded_definition_is_still_listed() {
        let (dir, git) = init_repo();
        let ghost = item(
            "20261015-003903-cccc",
            "ghost",
            "Unknown limit",
            Ladder::Implementation,
            Reference::path("a.md"),
        );
        RecordsWriter::new(&git)
            .append(
                "records",
                &[put(
                    path_of(&ghost),
                    AnyRecord::WorkItem(Box::new(ghost.clone())),
                )],
            )
            .unwrap();
        let html = render(&locate(dir.path()));
        assert_eq!(html.matches("<article").count(), 1, "{html}");
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
        assert!(html.contains("Nothing waiting for you"));
        assert!(!html.contains("No proposals listed"));
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
