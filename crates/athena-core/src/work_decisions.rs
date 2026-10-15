//! Deciding a steward proposal (`docs/design/roles-and-ladder.md`).
//!
//! The steward queues a change above its own limit only as a work item of kind proposal. A work item cites its
//! source and evidence by reference (a job id, a path, a commit), and readers cannot see `records/`, so a job
//! whose request only pointed at the work item would be read blind.
//!
//! `athena approve-work <id>` therefore builds the request from the work item itself ([`approve_request`]): its
//! title, rung, source and evidence, with the content of every cited record copied into the text. The request
//! goes through the usual reading, like `athena accept-proposal`. `athena reject-work <id> --why "..."` records
//! the reason. Work items are never edited, so each decision is an appended `athena.work-decision/1` record
//! that names the work item; an item with a decision is no longer open ([`open_proposals`]).

use crate::commits::{NewFile, message};
use crate::store::Records;
use crate::{Athena, Ui};
use anyhow::{Result, bail};
use athena_data::RecordId;
use athena_data::records::{
    AnyRecord, RefKind, Reference, WorkDecision, WorkDecisionState, WorkItem, WorkItemKind,
};
use athena_git::Git;
use jiff::Timestamp;

/// The most characters of one cited file copied into a request.
pub const MAX_FILE_CHARS: usize = 6000;

/// The most entries of a cited directory listed in a request.
pub const MAX_LISTED: usize = 60;

/// The decision recorded for `item`, if any. The newest by time wins if there are several.
pub fn decision_of<'a>(records: &'a Records, item: &WorkItem) -> Option<&'a WorkDecision> {
    records
        .work_decisions
        .iter()
        .filter(|d| d.work_item == item.id)
        .max_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)))
}

/// The work items of kind proposal that no decision names, oldest first.
pub fn open_proposals(records: &Records) -> Vec<&WorkItem> {
    let mut open: Vec<&WorkItem> = records
        .work_items
        .iter()
        .filter(|w| w.kind == WorkItemKind::Proposal && decision_of(records, w).is_none())
        .collect();
    open.sort_by(|a, b| (a.at, &a.id).cmp(&(b.at, &b.id)));
    open
}

/// The id in `text`: a bare id, or the path of the work item's record (the form `athena status` and the
/// records tree show).
fn parse_id(text: &str) -> Result<RecordId> {
    let name = text.trim().rsplit('/').next().unwrap_or("");
    let name = name.strip_suffix(".json").unwrap_or(name);
    RecordId::parse(name).map_err(|_| anyhow::anyhow!("{text:?} is not a work item id"))
}

/// The open proposal with this id, or why there is none: unknown, not a proposal, or already decided.
pub fn find_open<'a>(records: &'a Records, text: &str) -> Result<&'a WorkItem> {
    let id = parse_id(text)?;
    let Some(item) = records.work_items.iter().find(|w| w.id == id) else {
        bail!("no work item {id}; `athena status` lists the open proposals");
    };
    if item.kind != WorkItemKind::Proposal {
        bail!("work item {id} is not a proposal; only proposals are decided");
    }
    match decision_of(records, item) {
        None => Ok(item),
        Some(d) => match &d.decision {
            WorkDecisionState::Approved { job } => {
                bail!("work item {id} was already approved, as job {job}")
            }
            WorkDecisionState::Rejected { reason } => {
                bail!("work item {id} was already rejected: {reason}")
            }
        },
    }
}

/// The record that approves `item`, carried out by `job`.
pub fn approval(
    item: &WorkItem,
    job: athena_data::JobId,
    record: RecordId,
    now: Timestamp,
) -> WorkDecision {
    WorkDecision {
        id: record,
        at: now,
        work_item: item.id.clone(),
        decision: WorkDecisionState::Approved { job },
    }
}

/// The record that rejects `item`, with the person's reason.
pub fn rejection(
    item: &WorkItem,
    reason: &str,
    record: RecordId,
    now: Timestamp,
) -> Result<WorkDecision> {
    if reason.trim().is_empty() {
        bail!("a rejection gives a reason");
    }
    Ok(WorkDecision {
        id: record,
        at: now,
        work_item: item.id.clone(),
        decision: WorkDecisionState::Rejected {
            reason: reason.trim().to_string(),
        },
    })
}

fn clipped(text: &str, max: usize) -> String {
    let total = text.chars().count();
    if total <= max {
        return text.trim_end().to_string();
    }
    let head: String = text.chars().take(max).collect();
    format!(
        "{}\n[clipped: {} more characters]",
        head.trim_end(),
        total - max
    )
}

/// A fenced block, long enough a fence that the text cannot close it.
fn fenced(text: &str) -> String {
    let mut fence = String::from("```");
    while text.contains(&fence) {
        fence.push('`');
    }
    format!("{fence}\n{text}\n{fence}")
}

/// What a job record says, as text: its request, how it was read, and how it ended.
fn job_text(records: &Records, id: &str) -> String {
    let Some(job) = records.jobs.get(id) else {
        return "This job is not in the records.".into();
    };
    let mut out = String::new();
    match &job.request {
        Some(request) => out.push_str(&format!(
            "Request:\n{}\n",
            fenced(&clipped(&request.text, MAX_FILE_CHARS))
        )),
        None => out.push_str("The job has no recorded request.\n"),
    }
    if let Some(reading) = job.latest_reading() {
        out.push_str(&format!(
            "Read as: {} ({}). {}\n",
            reading.title.trim(),
            reading.class_of_work,
            reading.summary.trim()
        ));
    }
    match &job.outcome {
        Some(outcome) => out.push_str(&format!(
            "Outcome: {}. {}\n",
            outcome.state.as_str(),
            outcome.reason.trim()
        )),
        None => out.push_str("Outcome: none recorded yet.\n"),
    }
    out
}

/// What a path holds at `head`: the file's text, a directory's listing, or a note that it is not in the
/// repository (a file only the steward's workspace had, or a name such as `github`).
fn path_text(git: &Git, head: &str, path: &str) -> String {
    let wanted = path.trim().trim_end_matches('/');
    let Ok(entries) = git.ls_tree(head) else {
        return "The repository could not be listed.".into();
    };
    if let Some(entry) = entries.iter().find(|e| e.path == wanted) {
        return match git.show_file(head, &entry.path) {
            Ok(Some(bytes)) => fenced(&clipped(&String::from_utf8_lossy(&bytes), MAX_FILE_CHARS)),
            _ => "The file could not be read.".into(),
        };
    }
    let prefix = if wanted.is_empty() || wanted == "." {
        String::new()
    } else {
        format!("{wanted}/")
    };
    let under: Vec<&str> = entries
        .iter()
        .filter(|e| e.path.starts_with(&prefix))
        .map(|e| e.path.as_str())
        .collect();
    if under.is_empty() {
        return "This path is not in the repository at the current commit.".into();
    }
    let mut listing = under
        .iter()
        .take(MAX_LISTED)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    if under.len() > MAX_LISTED {
        listing.push_str(&format!(
            "\n[{} more files not listed]",
            under.len() - MAX_LISTED
        ));
    }
    format!(
        "A directory of {} file(s). Their paths:\n{}",
        under.len(),
        fenced(&listing)
    )
}

fn commit_text(git: &Git, sha: &str) -> String {
    match git.run(["show", "-s", "--format=%H%n%s%n%n%b", sha]) {
        Ok(text) => fenced(&clipped(&text, MAX_FILE_CHARS)),
        Err(_) => "This commit is not in the repository.".into(),
    }
}

/// A reference and the content it points at.
fn render(reference: &Reference, records: &Records, git: &Git, head: &str) -> String {
    match reference.kind {
        RefKind::Job => format!(
            "job {}\n{}",
            reference.value,
            job_text(records, &reference.value)
        ),
        RefKind::Path => format!(
            "path {}\n{}",
            reference.value,
            path_text(git, head, &reference.value)
        ),
        RefKind::Commit => format!(
            "commit {}\n{}",
            reference.value,
            commit_text(git, &reference.value)
        ),
    }
}

/// The request text of the ordinary job that carries out an approved proposal. It states the work item's
/// title, rung, source and evidence, and copies the content of each cited record into the text, because the
/// reader cannot open `records/`.
pub fn approve_request(item: &WorkItem, records: &Records, git: &Git, head: &str) -> String {
    let mut out = format!(
        "{title}\n\n\
         A person approved this proposal from the steward role. Carry the work out as an ordinary change.\n\
         Everything the proposal cites is copied below; nothing here points at a file you cannot open.\n\n\
         Proposal: {id} (key {key}), queued at {at} by the {role} role.\n\
         Rung of the requirements ladder: {rung}.\n\n\
         ## Source\n\n{source}\n\n## Evidence\n",
        title = item.title.trim(),
        id = item.id,
        key = item.key,
        at = item.at,
        role = item.role,
        rung = item.rung.as_str(),
        source = render(&item.source, records, git, head),
    );
    for (n, reference) in item.evidence.iter().enumerate() {
        out.push_str(&format!(
            "\n### {}. {}\n",
            n + 1,
            render(reference, records, git, head)
        ));
    }
    out
}

impl Athena {
    fn fresh_decision_id(&self, known: &Records) -> RecordId {
        let mut id = RecordId::generate(crate::now());
        while known.has_record_id(&id) {
            id = RecordId::generate(crate::now());
        }
        id
    }

    /// `athena approve-work`: submits the ordinary job that carries out the proposal, through the usual
    /// reading, and then records the work item as approved by that job. Returns the reading.
    pub fn approve_work(&self, id: &str, ui: &dyn Ui) -> Result<crate::reading::Proposal> {
        let head = self.head()?;
        let known = Records::load(&self.git, &head)?;
        let item = find_open(&known, id)?.clone();
        let request = approve_request(&item, &known, &self.git, &head);
        let reading = self.ask(&request, ui)?;
        let known = Records::load(&self.git, &self.head()?)?;
        let next = approval(
            &item,
            reading.job.clone(),
            self.fresh_decision_id(&known),
            crate::now(),
        );
        let record = AnyRecord::WorkDecision(next);
        let path = record.canonical_path();
        self.commit_records(
            &[NewFile::record(&record)],
            &message(
                &format!("Approve work item {}", item.id),
                "The work item is carried out by the job named here. Its request copies the content of the \
                 work item's source and evidence.",
                &[
                    ("Athena-Record", path),
                    ("Athena-Job", reading.job.as_str().to_string()),
                ],
            ),
        )?;
        Ok(reading)
    }

    /// `athena reject-work`: appends a rejection record with the reason.
    pub fn reject_work(&self, id: &str, reason: &str) -> Result<WorkDecision> {
        let head = self.head()?;
        let known = Records::load(&self.git, &head)?;
        let item = find_open(&known, id)?;
        let next = rejection(item, reason, self.fresh_decision_id(&known), crate::now())?;
        let record = AnyRecord::WorkDecision(next.clone());
        let path = record.canonical_path();
        self.commit_records(
            &[NewFile::record(&record)],
            &message(
                &format!("Reject work item {}", item.id),
                "The person's reason is in the record.",
                &[("Athena-Record", path)],
            ),
        )?;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::roles::Ladder;

    fn at() -> Timestamp {
        "2026-10-15T00:00:00Z".parse().unwrap()
    }

    fn item(kind: WorkItemKind) -> WorkItem {
        WorkItem {
            id: RecordId::parse("20261015-003903-khz6").unwrap(),
            at: at(),
            role: "steward".into(),
            kind,
            key: "k".into(),
            title: "Change the thing".into(),
            rung: Ladder::DesignDecisions,
            source: Reference::path("records/jobs"),
            evidence: vec![Reference::path("README.md")],
        }
    }

    #[test]
    fn only_undecided_proposals_are_open_and_a_second_decision_is_refused() {
        let mut records = Records::default();
        records.work_items.push(item(WorkItemKind::Proposal));
        let mut other = item(WorkItemKind::BlindSpot);
        other.id = RecordId::parse("20261015-003904-aaaa").unwrap();
        records.work_items.push(other);
        assert_eq!(open_proposals(&records).len(), 1);
        assert!(find_open(&records, "20261015-003903-khz6").is_ok());
        // The path form is accepted too.
        assert!(
            find_open(
                &records,
                "records/work-items/2026-10/20261015-003903-khz6.json"
            )
            .is_ok()
        );
        assert!(find_open(&records, "20261015-003904-aaaa").is_err());
        assert!(find_open(&records, "20261015-009999-zzzz").is_err());
        assert!(find_open(&records, "nonsense").is_err());

        let decided = rejection(
            &records.work_items[0],
            " Too big. ",
            RecordId::parse("20261015-010000-bbbb").unwrap(),
            at(),
        )
        .unwrap();
        assert_eq!(
            decided.decision,
            WorkDecisionState::Rejected {
                reason: "Too big.".into()
            }
        );
        records.work_decisions.push(decided);
        assert!(open_proposals(&records).is_empty());
        let error = find_open(&records, "20261015-003903-khz6").unwrap_err();
        assert!(error.to_string().contains("already rejected"), "{error}");
    }

    #[test]
    fn a_rejection_needs_a_reason() {
        let w = item(WorkItemKind::Proposal);
        let id = RecordId::parse("20261015-010000-bbbb").unwrap();
        assert!(rejection(&w, "  ", id, at()).is_err());
    }

    #[test]
    fn long_text_is_clipped_and_fences_cannot_be_closed_by_the_text() {
        let text = "x".repeat(MAX_FILE_CHARS + 5);
        assert!(clipped(&text, MAX_FILE_CHARS).ends_with("[clipped: 5 more characters]"));
        assert_eq!(clipped("short\n", MAX_FILE_CHARS), "short");
        let block = fenced("a ``` b");
        assert!(block.starts_with("````\n") && block.ends_with("\n````"));
    }
}
