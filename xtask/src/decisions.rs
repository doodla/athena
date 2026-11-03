//! The owner decision briefs page: `decisions.html`, written next to `index.html` by `xtask site`.
//!
//! One short brief per open Steward proposal (a work item of kind proposal that no decision names). A brief
//! uses only the proposal's own fields: title, rung, source and evidence. The recommendation reads the
//! limit of the role that raised the proposal from the role records, with the same rule `athena-core` uses
//! to split proposals (the newest version of the role by name, then `may_change` on its highest rung).
//!
//! Records are read from the records branch through `athena-git` and parsed with `check_record_file`, so no
//! new file format is read. A checkout without the records branch lists no proposals, and the page says so
//! rather than reporting that nothing waits. The owner's style for the text is in
//! `docs/conventions/owner-style.md`.

use crate::site::{ATHENA_URL, CSS, escape, source_url};
use anyhow::{Context, Result, anyhow, bail};
use athena_data::records::{
    AnyRecord, RefKind, Reference, WorkItem, WorkItemKind, check_record_file,
};
use athena_data::roles::{Ladder, RoleDefinition, may_change};
use athena_git::Git;
use athena_git::records_branch::{RecordsTip, resolve_tip};
use std::fmt::Write as _;
use std::path::Path;

/// The most words of prose one brief may hold. A longer one is refused, not cut.
pub const MAX_WORDS: usize = 120;

/// The file the page is written to, next to `index.html`.
pub const PAGE: &str = "decisions.html";

/// A link base: the GitHub repository and the commit its links point at.
#[derive(Debug, Clone, Copy)]
pub struct Pin<'a> {
    pub base: &'a str,
    pub sha: &'a str,
}

/// The records a page is built from.
#[derive(Debug, Default)]
pub struct Waiting {
    pub items: Vec<WorkItem>,
    pub decided: Vec<athena_data::RecordId>,
    pub roles: Vec<RoleDefinition>,
}

/// What reading the checkout found.
#[derive(Debug)]
pub enum Found {
    /// Neither the local records branch nor its remote-tracking ref exists.
    NoRecordsBranch,
    /// The records branch at `tip`, with what it holds.
    At { tip: String, waiting: Waiting },
}

/// One brief, with every line already worked out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Brief {
    pub id: String,
    pub title: String,
    pub rung: Ladder,
    pub role: String,
    pub source: String,
    pub evidence: Vec<String>,
    pub recommendation: String,
    /// Where the work item lives on the records branch.
    pub path: String,
}

fn reference_text(r: &Reference) -> String {
    let kind = match r.kind {
        RefKind::Job => "job",
        RefKind::Path => "path",
        RefKind::Commit => "commit",
    };
    format!("{kind} {}", r.value)
}

/// The recommendation line for a proposal, from the limit of the role that raised it.
pub fn recommendation(item: &WorkItem, roles: &[RoleDefinition]) -> String {
    let rung = item.rung.as_str();
    let role = &item.role;
    let limit = roles
        .iter()
        .filter(|r| &r.name == role)
        .max_by_key(|r| r.version)
        .map(|r| r.highest_rung);
    match limit {
        None => format!(
            "The {role} role is not recorded, so Athena cannot tell whether rung {rung} is within its limit. Decide yourself."
        ),
        Some(limit) if may_change(limit, item.rung) => format!(
            "Rung {rung} is within the {role} role's limit ({}). Athena approves it without you; check only if you disagree.",
            limit.as_str()
        ),
        Some(_) if item.rung < Ladder::HIGHEST_SELF_CHANGE => {
            format!("Rung {rung} is in the top two; no role may change it. Decide yourself.")
        }
        Some(limit) => format!(
            "Rung {rung} is above the {role} role's limit ({}). Decide yourself.",
            limit.as_str()
        ),
    }
}

/// The briefs for the open proposals, oldest first.
pub fn briefs(waiting: &Waiting) -> Vec<Brief> {
    let mut open: Vec<&WorkItem> = waiting
        .items
        .iter()
        .filter(|w| w.kind == WorkItemKind::Proposal && !waiting.decided.contains(&w.id))
        .collect();
    open.sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
    open.into_iter()
        .map(|w| Brief {
            id: w.id.to_string(),
            title: w.title.trim().to_string(),
            rung: w.rung,
            role: w.role.clone(),
            source: reference_text(&w.source),
            evidence: w.evidence.iter().map(reference_text).collect(),
            recommendation: recommendation(w, &waiting.roles),
            path: format!("records/work-items/{}/{}.json", w.id.month(), w.id),
        })
        .collect()
}

fn word_count(texts: &[&str]) -> usize {
    texts.iter().map(|t| t.split_whitespace().count()).sum()
}

/// One brief as HTML. A brief with no recommendation, or with more than [`MAX_WORDS`] words of prose, is an
/// error.
pub fn render_brief(brief: &Brief, pin: Pin) -> Result<String> {
    if brief.recommendation.trim().is_empty() {
        bail!("the brief for {} has no recommendation line", brief.id);
    }
    let evidence_line = match brief.evidence.split_first() {
        None => "Evidence: not recorded.".to_string(),
        Some((first, [])) => format!("First evidence: {first}."),
        Some((first, rest)) => format!("First evidence: {first} (and {} more).", rest.len()),
    };
    let rung_line = format!("Rung: {}. Raised by: {}.", brief.rung.as_str(), brief.role);
    let source_line = format!("Source: {}.", brief.source);
    let words = word_count(&[
        &brief.title,
        &rung_line,
        &source_line,
        &evidence_line,
        &brief.recommendation,
    ]);
    if words > MAX_WORDS {
        bail!(
            "the brief for {} has {words} words of prose; one screen holds at most {MAX_WORDS}",
            brief.id
        );
    }
    let link = source_url(pin.base, pin.sha, &brief.path, 0, 0);
    Ok(format!(
        "<article class=\"card brief\"><h3>{title}</h3>\
         <p class=\"recommendation\"><strong>{rec}</strong></p>\
         <p>{rung}</p><p>{source}</p><p>{evidence}</p>\
         <p>Record: <a href=\"{link}\">{id}</a></p>\
         <p>Then run <code>athena approve-work {id}</code> or <code>athena reject-work {id} --why \"...\"</code></p>\
         </article>",
        title = escape(&brief.title),
        rec = escape(&brief.recommendation),
        rung = escape(&rung_line),
        source = escape(&source_line),
        evidence = escape(&evidence_line),
        link = escape(&link),
        id = escape(&brief.id),
    ))
}

/// The whole page for what the checkout holds.
pub fn render_page(found: &Found) -> Result<String> {
    let mut body = String::from(
        "<header><h1>Proposals waiting for the owner</h1>\
         <p class=\"lede\">One short brief for each open Steward proposal. Each uses only the proposal's own fields.</p></header>",
    );
    let footer = match found {
        Found::NoRecordsBranch => {
            body.push_str(
                "<section><p class=\"empty\">The records branch is not in this checkout, so no proposals \
                 could be listed. This does not mean none are waiting.</p></section>",
            );
            "No records branch was found.".to_string()
        }
        Found::At { tip, waiting } => {
            let pin = Pin {
                base: ATHENA_URL,
                sha: tip,
            };
            let list = briefs(waiting);
            if list.is_empty() {
                body.push_str(
                    "<section><p class=\"empty\">No proposals are waiting.</p></section>",
                );
            } else {
                let _ = write!(
                    body,
                    "<section><p>{} waiting, oldest first.</p><div class=\"cards\">",
                    list.len()
                );
                for brief in &list {
                    body.push_str(&render_brief(brief, pin)?);
                }
                body.push_str("</div></section>");
            }
            format!(
                "Read from the records branch at commit <code>{}</code>.",
                escape(tip)
            )
        }
    };
    let _ = write!(body, "<footer>{footer}</footer>");
    Ok(format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>Proposals waiting for the owner</title><style>{CSS}</style></head><body><main>{body}</main></body></html>\n"
    ))
}

/// Reads the work items, work decisions and role records from the records branch of `git`.
pub fn read(git: &Git) -> Result<Found> {
    let tip = match resolve_tip(git).context("cannot resolve the records branch")? {
        RecordsTip::Missing => return Ok(Found::NoRecordsBranch),
        RecordsTip::At { tip, .. } => tip,
        RecordsTip::Diverged { local, tracking } => bail!(
            "the records branch has diverged (local {local}, remote {tracking}); fetch it before building"
        ),
    };
    let mut entries = Vec::new();
    for dir in [
        "records/work-items",
        "records/work-decisions",
        "records/roles",
    ] {
        entries.extend(
            git.ls_tree_under(&tip, dir)
                .with_context(|| format!("cannot list {dir} at {tip}"))?
                .into_iter()
                .filter(|e| e.kind == "blob" && e.path.ends_with(".json")),
        );
    }
    let oids: Vec<String> = entries.iter().map(|e| e.oid.clone()).collect();
    let blobs = git.cat_blobs(&oids).context("cannot read the records")?;
    let mut waiting = Waiting::default();
    for (entry, bytes) in entries.iter().zip(blobs) {
        let record = check_record_file(&entry.path, &bytes).map_err(|e| anyhow!("{e}"))?;
        match record {
            Some(AnyRecord::WorkItem(item)) => waiting.items.push(*item),
            Some(AnyRecord::WorkDecision(d)) => waiting.decided.push(d.work_item),
            Some(AnyRecord::Role(role)) => waiting.roles.push(*role),
            _ => {}
        }
    }
    Ok(Found::At { tip, waiting })
}

/// The page for the repository at `root`. A directory that is not a git repository has no records branch.
pub fn page(root: &Path) -> Result<String> {
    let found = if root.join(".git").exists() {
        read(&Git::at(root))?
    } else {
        Found::NoRecordsBranch
    };
    render_page(&found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::RecordId;
    use athena_data::records::WorkDecision;
    use athena_data::records::WorkDecisionState;
    use athena_git::records_branch::{Change, RecordsWriter};

    const TIP: &str = "0123456789abcdef0123456789abcdef01234567";

    fn item(id: &str, role: &str, rung: Ladder, title: &str, evidence: Vec<Reference>) -> WorkItem {
        WorkItem {
            id: RecordId::parse(id).unwrap(),
            at: "2026-10-14T12:00:00Z".parse().unwrap(),
            role: role.into(),
            kind: WorkItemKind::Proposal,
            key: format!("key-{id}"),
            title: title.into(),
            rung,
            source: Reference::path("docs/design/x.md"),
            evidence,
        }
    }

    fn role(name: &str, version: u32, limit: Ladder) -> RoleDefinition {
        RoleDefinition {
            name: name.into(),
            version,
            at: "2026-10-14T00:00:00Z".parse().unwrap(),
            goal: "g".into(),
            prompt: "p".into(),
            cadence_tasks: 5,
            highest_rung: limit,
        }
    }

    fn pin() -> Pin<'static> {
        Pin {
            base: ATHENA_URL,
            sha: TIP,
        }
    }

    fn fixtures() -> Waiting {
        Waiting {
            items: vec![
                item(
                    "20261014-120000-1abc",
                    "steward",
                    Ladder::DefaultsConventions,
                    "Name the owner style",
                    vec![Reference::commit("abc1234")],
                ),
                item(
                    "20261015-120000-2def",
                    "steward",
                    Ladder::DesignDecisions,
                    "Change the ladder",
                    vec![],
                ),
            ],
            decided: vec![],
            roles: vec![role("steward", 2, Ladder::DefaultsConventions)],
        }
    }

    #[test]
    fn briefs_from_different_proposals_differ_and_carry_no_fixed_cost_risk_or_approve() {
        let list = briefs(&fixtures());
        assert_eq!(list.len(), 2);
        let a = render_brief(&list[0], pin()).unwrap();
        let b = render_brief(&list[1], pin()).unwrap();
        assert_ne!(a, b);
        assert!(a.contains("Name the owner style") && !a.contains("Change the ladder"));
        assert!(a.contains("Rung: defaults-conventions.") && b.contains("Rung: design-decisions."));
        assert!(a.contains("First evidence: commit abc1234."));
        assert!(b.contains("Evidence: not recorded."));
        for html in [&a, &b] {
            for word in ["Approve", "cost", "risk", "Cost", "Risk"] {
                assert!(!html.contains(word), "{word}");
            }
            assert!(html.contains("athena approve-work "));
            assert!(html.contains("athena reject-work "));
        }
    }

    #[test]
    fn the_recommendation_follows_the_limit_in_the_role_records() {
        let roles = vec![
            role("steward", 1, Ladder::Implementation),
            role("steward", 2, Ladder::DefaultsConventions),
            role("reader", 1, Ladder::Implementation),
        ];
        let rung = Ladder::DefaultsConventions;
        let at = |role: &str, rung: Ladder| {
            recommendation(
                &item("20261014-120000-1abc", role, rung, "t", vec![]),
                &roles,
            )
        };
        let within = at("steward", rung);
        let above = at("reader", rung);
        assert_eq!(
            within,
            "Rung defaults-conventions is within the steward role's limit (defaults-conventions). Athena approves it without you; check only if you disagree."
        );
        assert_eq!(
            above,
            "Rung defaults-conventions is above the reader role's limit (implementation). Decide yourself."
        );
        assert!(!within.contains("Decide yourself"));
        assert_eq!(
            at("nobody", rung),
            "The nobody role is not recorded, so Athena cannot tell whether rung defaults-conventions is within its limit. Decide yourself."
        );
        assert_eq!(
            at("steward", Ladder::BriefRequirements),
            "Rung brief-requirements is in the top two; no role may change it. Decide yourself."
        );
    }

    #[test]
    fn every_record_link_is_absolute_on_the_athena_base_with_the_records_tip() {
        let found = Found::At {
            tip: TIP.into(),
            waiting: fixtures(),
        };
        let html = render_page(&found).unwrap();
        let hrefs: Vec<&str> = html
            .split("href=\"")
            .skip(1)
            .map(|s| s.split('"').next().unwrap())
            .collect();
        assert_eq!(hrefs.len(), 2);
        for href in hrefs {
            assert!(
                href.starts_with(&format!("{ATHENA_URL}/blob/{TIP}/records/work-items/")),
                "{href}"
            );
        }
        assert!(html.contains(TIP));
    }

    #[test]
    fn a_long_brief_and_a_brief_without_a_recommendation_are_refused() {
        let mut brief = briefs(&fixtures()).remove(0);
        brief.title = vec!["word"; 130].join(" ");
        let err = render_brief(&brief, pin()).unwrap_err();
        assert!(format!("{err:#}").contains("words of prose"), "{err:#}");
        brief.title = vec!["word"; 60].join(" ");
        assert!(render_brief(&brief, pin()).is_ok());
        brief.recommendation = "  ".into();
        let err = render_brief(&brief, pin()).unwrap_err();
        assert!(format!("{err:#}").contains("no recommendation"), "{err:#}");
        // A page holding such a brief is refused, not shown without it.
        let mut waiting = fixtures();
        waiting.items[0].title = vec!["word"; 130].join(" ");
        let found = Found::At {
            tip: TIP.into(),
            waiting,
        };
        assert!(render_page(&found).is_err());
    }

    #[test]
    fn decided_proposals_and_other_kinds_are_not_listed() {
        let mut waiting = fixtures();
        waiting.decided.push(waiting.items[0].id.clone());
        waiting.items.push(WorkItem {
            kind: WorkItemKind::BlindSpot,
            ..item(
                "20261016-120000-3ghj",
                "steward",
                Ladder::Implementation,
                "blind",
                vec![],
            )
        });
        let list = briefs(&waiting);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "Change the ladder");
    }

    fn records_repo() -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        (dir, git)
    }

    fn put(record: AnyRecord) -> Change {
        let path = record.canonical_path();
        Change::put(path, serde_json::to_vec_pretty(&record).unwrap())
    }

    #[test]
    fn a_checkout_without_the_records_branch_says_so_and_lists_nothing() {
        let (_dir, git) = records_repo();
        let found = read(&git).unwrap();
        assert!(matches!(found, Found::NoRecordsBranch));
        let html = render_page(&found).unwrap();
        assert!(html.contains("The records branch is not in this checkout"));
        assert!(html.contains("does not mean none are waiting"));
        assert!(!html.contains("No proposals are waiting."));
        assert!(!html.contains("<article"));
        // A directory that is not a repository is the same.
        let plain = tempfile::tempdir().unwrap();
        assert!(
            page(plain.path())
                .unwrap()
                .contains("The records branch is not in this checkout")
        );
    }

    #[test]
    fn open_proposals_are_read_from_the_records_branch_with_role_records() {
        let (_dir, git) = records_repo();
        let w = fixtures();
        let decision = WorkDecision {
            id: RecordId::parse("20261016-090000-4jkm").unwrap(),
            at: "2026-10-16T09:00:00Z".parse().unwrap(),
            work_item: w.items[1].id.clone(),
            decision: WorkDecisionState::Rejected {
                reason: "no".into(),
            },
        };
        let changes = [
            put(AnyRecord::WorkItem(Box::new(w.items[0].clone()))),
            put(AnyRecord::WorkItem(Box::new(WorkItem {
                evidence: vec![Reference::commit("def5678")],
                ..w.items[1].clone()
            }))),
            put(AnyRecord::WorkDecision(decision)),
            put(AnyRecord::Role(Box::new(w.roles[0].clone()))),
        ];
        let tip = RecordsWriter::new(&git)
            .append("records", &changes)
            .unwrap();
        let found = read(&git).unwrap();
        let Found::At {
            tip: read_tip,
            waiting,
        } = &found
        else {
            panic!("{found:?}")
        };
        assert_eq!(read_tip, &tip);
        let list = briefs(waiting);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "Name the owner style");
        assert!(
            list[0]
                .recommendation
                .contains("within the steward role's limit")
        );
        let html = render_page(&found).unwrap();
        assert!(html.contains(&format!(
            "{ATHENA_URL}/blob/{tip}/records/work-items/2026-10/"
        )));
    }

    #[test]
    fn a_branch_with_no_open_proposal_says_none_are_waiting() {
        let html = render_page(&Found::At {
            tip: TIP.into(),
            waiting: Waiting::default(),
        })
        .unwrap();
        assert!(html.contains("No proposals are waiting."));
    }
}
