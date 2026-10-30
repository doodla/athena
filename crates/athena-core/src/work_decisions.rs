//! Deciding a steward proposal (`docs/design/roles-and-ladder.md`).
//!
//! The steward queues a change above its own limit only as a work item of kind proposal. A work item cites its
//! source and evidence by reference (a job id, a path, a commit), and readers cannot see `records/`, so a job
//! whose request only pointed at the work item would be read blind.
//!
//! `athena approve-work <id>` therefore builds the request from the work item itself ([`approve_request`]): its
//! title, rung, source and evidence, with the content of every cited record copied into the text. Words the
//! admission scan rejects are reworded first, so a proposal is never stuck at admission. The request goes
//! through the usual reading, like `athena accept-proposal`. `athena reject-work <id> --why "..."` records
//! the reason. Work items are never edited, so each decision is an appended `athena.work-decision/1` record
//! that names the work item; an item with a decision is no longer open ([`open_proposals`]).

use crate::commits::{NewFile, message};
use crate::store::Records;
use crate::{Athena, Ui};
use anyhow::{Result, bail};
use athena_context::briefs::plain_boundary_text;
use athena_data::RecordId;
use athena_data::records::{
    AnyRecord, RefKind, Reference, WorkDecision, WorkDecisionState, WorkItem, WorkItemKind,
};
use athena_data::roles::{Ladder, may_change};
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

/// The highest rung the role called `name` may change itself, by its newest recorded version. None when no
/// definition of that role is recorded.
pub fn role_limit(records: &Records, name: &str) -> Option<Ladder> {
    records
        .roles
        .iter()
        .filter(|r| r.name == name)
        .max_by_key(|r| r.version)
        .map(|r| r.highest_rung)
}

/// Whether the role that raised `item` may change the item's rung itself. A role with no recorded
/// definition may not.
pub fn in_limit(records: &Records, item: &WorkItem) -> bool {
    role_limit(records, &item.role).is_some_and(|limit| may_change(limit, item.rung))
}

/// The open proposals, oldest first, split by the ladder: those the raising role may change itself, and those
/// it may not (above its limit, or raised by a role with no recorded definition). Only the first kind is
/// approved without a person.
pub fn split_open(records: &Records) -> (Vec<&WorkItem>, Vec<&WorkItem>) {
    open_proposals(records)
        .into_iter()
        .partition(|w| in_limit(records, w))
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
///
/// The copied content comes from files, jobs and commits that may hold a word the admission scan rejects (a
/// proposal about a recurring duty of the person running Athena is a case), and a request holding one is
/// refused, so the proposal could never be worked. The finished text therefore goes through
/// [`plain_boundary_text`], which rewords such words. It is an error only when a word cannot be reworded.
pub fn approve_request(
    item: &WorkItem,
    records: &Records,
    git: &Git,
    head: &str,
) -> Result<String> {
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
    plain_boundary_text(&out).map_err(|why| {
        anyhow::anyhow!(
            "the request for work item {} cannot be made admissible: {why}",
            item.id
        )
    })
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
        let request = approve_request(&item, &known, &self.git, &head)?;
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

    /// Approves an open proposal like [`Athena::approve_work`], but only when the role that raised it may
    /// change its rung itself; any other proposal is refused and stays open for a person.
    pub fn approve_in_limit(&self, id: &str, ui: &dyn Ui) -> Result<crate::reading::Proposal> {
        let known = Records::load(&self.git, &self.head()?)?;
        let item = find_open(&known, id)?;
        if !in_limit(&known, item) {
            bail!(
                "work item {} (rung {}, raised by {}) is above that role's limit, so a person decides it",
                item.id,
                item.rung.as_str(),
                item.role
            );
        }
        self.approve_work(id, ui)
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
    fn a_proposal_is_in_limit_only_for_a_recorded_role_whose_limit_covers_its_rung() {
        let mut records = Records::default();
        let mut low = item(WorkItemKind::Proposal);
        low.rung = Ladder::Implementation;
        let mut high = item(WorkItemKind::Proposal);
        high.id = RecordId::parse("20261015-003904-aaaa").unwrap();
        let mut stranger = low.clone();
        stranger.id = RecordId::parse("20261015-003905-bbbb").unwrap();
        stranger.role = "ghost".into();
        records.work_items.extend([low, high, stranger]);
        // No role is recorded yet, so nothing is in limit.
        let (within, above) = split_open(&records);
        assert!(within.is_empty());
        assert_eq!(above.len(), 3);
        records.roles.push(athena_data::roles::steward(at()));
        let (within, above) = split_open(&records);
        assert_eq!(within.len(), 1);
        assert_eq!(within[0].rung, Ladder::Implementation);
        assert_eq!(above.len(), 2, "above the limit, and an unknown role");
    }

    /// A repository with `athena.toml`, `defaults.toml` and the files given, and an Athena whose reader
    /// declines every request (so no capability is needed). Returns the directories that keep it alive.
    fn repo_with(files: &[(&str, &str)]) -> (tempfile::TempDir, tempfile::TempDir, Athena) {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let git = Git::at(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value]).unwrap();
        }
        let mut all = vec![
            ("athena.toml", include_str!("../../../athena.toml")),
            ("defaults.toml", include_str!("../../../defaults.toml")),
        ];
        all.extend_from_slice(files);
        for (path, text) in all {
            let full = dir.path().join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, text).unwrap();
        }
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "Start"]).unwrap();
        let script = Box::new(|_: &athena_agent::AgentInvocation| {
            athena_agent::fake::FakeTurn::output(serde_json::json!({
                "title": "Describe the duty",
                "summary": "Describe the duty in the notes.",
                "class_of_work": "docs",
                "capability": null,
                "decline_reason": "not a request",
                "producer_model": "claude-haiku-5-5",
                "defaults_applied": [{ "id": "strangers", "how": "plain words" }],
                "assumptions": [],
                "question": null,
                "acceptance_criteria": [{ "text": "The notes describe the duty.", "kind": "content" }],
                "deliverables": [{ "path": "docs/duty.md", "action": "modify", "p": 0.9 }],
                "out_of_scope": ["Other files"],
                "forecast": {
                    "p_reading_accepted": 0.8,
                    "p_verification_passes": 0.7,
                    "p_user_intervenes": 0.1,
                    "cost_usd": { "p10": 0.05, "p50": 0.2, "p90": 0.8 },
                    "wall_time_s": { "p10": 5.0, "p50": 30.0, "p90": 300.0 },
                    "departures": null
                }
            }))
        });
        let athena = crate::test_support::make_athena(&git, state.path(), vec![], script);
        (dir, state, athena)
    }

    #[test]
    fn approve_work_succeeds_on_a_proposal_whose_text_holds_a_boundary_word() {
        let word = ["oper", "ator"].concat();
        let note = format!("The {word} must be told when a duty recurs.\n");
        let (dir, _state, athena) = repo_with(&[("docs/duty.md", &note)]);
        let mut proposal = item(WorkItemKind::Proposal);
        proposal.title = format!("Tell the {word} about a recurring duty");
        proposal.source = Reference::path("docs/duty.md");
        proposal.evidence = vec![Reference::path("docs/")];
        let record = AnyRecord::WorkItem(Box::new(proposal.clone()));
        let file = NewFile::record(&record);
        let full = dir.path().join(&file.path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, &file.bytes).unwrap();
        athena.git.run(["add", "-A"]).unwrap();
        athena
            .git
            .run(["commit", "-q", "-m", "Seed a steward proposal"])
            .unwrap();

        // The seed fails admission when its text is used as it stands: this is the 2026-10-29 failure.
        let raw = format!("{}\n\n{note}", proposal.title);
        assert!(!crate::reading::scan_request_input(&raw).is_empty());
        let refused = athena.ask(&raw, &crate::Quiet).unwrap_err();
        assert!(refused.to_string().contains("not admitted"), "{refused}");

        let reading = athena
            .approve_work(&proposal.id.to_string(), &crate::Quiet)
            .expect("the proposal is approved");
        let known = Records::load(&athena.git, &athena.head().unwrap()).unwrap();
        let text = &known.jobs[reading.job.as_str()]
            .request
            .as_ref()
            .unwrap()
            .text;
        assert!(
            athena_context::briefs::boundary_scan(text).is_empty(),
            "{text}"
        );
        assert!(crate::reading::scan_request_input(text).is_empty());
        assert!(text.contains("the person running Athena"), "{text}");
        // The reading the job was given passes the admission checks too.
        crate::reading::check_boundary_and_repo_files(reading.reading.as_ref().unwrap(), None)
            .unwrap();
        // The decision names the job.
        let [decision] = known.work_decisions.as_slice() else {
            panic!("one decision")
        };
        assert_eq!(
            decision.decision,
            WorkDecisionState::Approved {
                job: reading.job.clone()
            }
        );
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
