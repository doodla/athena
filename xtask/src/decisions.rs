//! The owner decision briefs page: `decisions.html`, built beside `index.html` by `cargo run -p xtask -- site`.
//!
//! The page lists the Steward's open proposals (work items of kind proposal that no work decision names) read
//! from the records branch `athena/records`. Each proposal becomes a short brief: its title, rung, source,
//! evidence, a recommendation, and a link to the record on GitHub at the branch tip. The style is the owner's
//! (`docs/conventions/owner-style.md`): plain words, nothing invented.
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
    AnyRecord, RefKind, Reference, WorkItem, WorkItemKind, check_record_file,
};
use athena_data::roles::{Ladder, RoleDefinition, may_change};
use athena_git::Git;
use athena_git::records_branch::{RECORDS_BRANCH, RecordsTip, resolve_tip};
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

/// The most words of prose one brief may show, counting every displayed line.
pub const MAX_WORDS: usize = 120;

/// One proposal as a short brief. Every field is text the page shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Brief {
    title: String,
    rung_line: String,
    role_line: String,
    source_line: String,
    /// One line per cited record, or the single line `Evidence: not recorded`.
    evidence_lines: Vec<String>,
    recommendation: String,
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

    fn html(&self) -> String {
        let mut out = format!("<article class=\"card\"><h3>{}</h3>", escape(&self.title));
        for line in [&self.rung_line, &self.role_line, &self.source_line] {
            let _ = write!(out, "<p>{}</p>", escape(line));
        }
        out.push_str("<ul class=\"evidence\">");
        for line in &self.evidence_lines {
            let _ = write!(out, "<li>{}</li>", escape(line));
        }
        let _ = write!(
            out,
            "</ul><p class=\"recommendation\"><strong>{}</strong></p>\
             <p>Record: <a href=\"{}\">{}</a></p></article>",
            escape(&self.recommendation),
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
