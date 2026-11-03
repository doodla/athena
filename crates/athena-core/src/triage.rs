//! The steward's triage duty: at the start of every drain, one model call sets priorities from the current
//! focus.
//!
//! The call reads the focus (`athena focus`), the open items (title, rung, current priority and who set it,
//! age, dependencies) and the failure classes of the last two days. It returns a priority for each item it
//! would change, with a one-line reason tied to the focus or to a repeated failure, and it may propose
//! closing an item.
//!
//! What Athena does with the answer, by mechanism and not by the model's good manners:
//!
//! - A change is written as a priority record whose `set_by` is the steward and whose `reason` is the model's
//!   line, so the setter can be read back from the records.
//! - An item whose latest priority record names no role was set by a person. It is never changed.
//! - The caps (`[priority_caps]`) are never exceeded: every open item already at urgent or high counts,
//!   including the ones a person set, and a change into a full tier is skipped.
//! - Nothing is closed. A proposal to close an item is recorded as a steward work item for a person to
//!   decide with `athena approve-work`; this module writes no outcome record of any kind.
//!
//! A failed call is an error to the caller, which prints it and goes on: triage never stops a drain.

use crate::Athena;
use crate::capability::{JobConfig, ToolGrant};
use crate::commits::{Committer, NewFile, message};
use crate::plan::{PlanEntry, Priority, TierCaps};
use crate::steward::{Cleanup, failure_classes_since, pending_invocations, stash_invocation};
use crate::store::{JobRecords, Records};
use anyhow::{Result, bail};
use athena_agent::env::Side;
use athena_context::briefs::plain_boundary_text;
use athena_context::steward_system_prompt;
use athena_data::records::{
    AnyRecord, InvocationOutcome, PriorityRecord, Reference, Role, WorkItem, WorkItemKind,
};
use athena_data::roles::{self, Ladder};
use athena_data::{JobId, RecordId};
use jiff::Timestamp;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// The heading the triage prompt starts with. The deterministic fake recognises a triage prompt by it.
pub const HEADING: &str = athena_agent::fake::TRIAGE_HEADING;

/// The role name written as `set_by` on a priority record triage wrote.
pub const AUTHOR: &str = "steward";

/// The longest reason kept, in characters.
const MAX_REASON: usize = 200;

/// The most characters of an item's title shown to the model.
const MAX_TITLE: usize = 120;

/// How far back the failure classes the model sees reach, in days.
const FAILURE_WINDOW_DAYS: i64 = 2;

/// The prefix of the key of a work item that proposes closing a job.
const CLOSE_KEY_PREFIX: &str = "triage-close:";

/// Who set an open item's current priority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Setter {
    /// A person: the latest priority record names no role. Triage never changes such an item.
    Person,
    /// A role, by name, through an earlier triage.
    Role(String),
    /// No priority record: the priority comes from the request's words and its reading.
    Words,
}

impl Setter {
    fn label(&self) -> String {
        match self {
            Setter::Person => "a person".into(),
            Setter::Role(role) => format!("the {role} role"),
            Setter::Words => "the request's own words".into(),
        }
    }
}

/// Whether a person set the job's priority: its latest priority record names no role. A job with no priority
/// record was set by nobody, so it is not a person's.
pub fn set_by_person(job: &JobRecords) -> bool {
    job.priorities
        .iter()
        .max_by_key(|p| p.n)
        .is_some_and(PriorityRecord::is_by_person)
}

fn setter_of(job: Option<&JobRecords>) -> Setter {
    match job.and_then(|j| j.priorities.iter().max_by_key(|p| p.n)) {
        None => Setter::Words,
        Some(p) => match &p.set_by {
            None => Setter::Person,
            Some(role) => Setter::Role(role.clone()),
        },
    }
}

/// One open item as the model sees it and as the caps count it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenItem {
    pub id: String,
    pub title: String,
    /// The requirements rung, or None when planning has not found it.
    pub rung: Option<Ladder>,
    /// The priority the item has now.
    pub priority: Priority,
    pub setter: Setter,
    pub age_days: i64,
    pub depends_on: Vec<String>,
}

impl OpenItem {
    /// The item for a plan entry: the entry has the rung, the priority and the dependencies; the records say
    /// who set the priority and when the request came in.
    pub fn of_entry(entry: &PlanEntry, records: &Records, now: Timestamp) -> OpenItem {
        let job = records.jobs.get(&entry.id);
        let age_days = job
            .and_then(|j| j.request.as_ref())
            .map_or(0, |r| {
                (now.as_second() - r.received_at.as_second()) / 86_400
            })
            .max(0);
        OpenItem {
            id: entry.id.clone(),
            title: entry.title.clone(),
            rung: entry.rung,
            priority: entry.priority,
            setter: setter_of(job),
            age_days,
            depends_on: entry.depends_on.clone(),
        }
    }
}

/// A priority change triage will write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub item: String,
    pub from: Priority,
    pub to: Priority,
    /// One line.
    pub reason: String,
}

impl Change {
    /// The line the drain prints for this change.
    pub fn line(&self) -> String {
        format!(
            "triage     {} {} -> {}: {}",
            self.item,
            self.from.as_str(),
            self.to.as_str(),
            self.reason
        )
    }
}

/// A proposal to close an item. It becomes a work item for a person to decide; nothing is closed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseProposal {
    pub item: String,
    /// The item's title, for the work item's title.
    pub title: String,
    /// One line.
    pub reason: String,
}

impl CloseProposal {
    /// The key of the work item, stable for the same job so a later drain does not propose it twice.
    pub fn key(&self) -> String {
        format!("{CLOSE_KEY_PREFIX}{}", self.item)
    }

    /// The line the drain prints for this proposal.
    pub fn line(&self) -> String {
        format!(
            "triage     {} proposed for closing (a work item for a person to decide): {}",
            self.item, self.reason
        )
    }
}

/// What a validated answer asks for.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Proposed {
    pub changes: Vec<Change>,
    pub closes: Vec<CloseProposal>,
}

/// The JSON schema of the model's answer.
pub fn schema() -> Value {
    let level = json!({ "type": "string", "enum": athena_data::records::PRIORITY_LEVELS });
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["changes", "close"],
        "properties": {
            "changes": {
                "type": "array",
                "description": "A priority for each item you would change. Empty when nothing should change.",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["item", "priority", "reason"],
                    "properties": {
                        "item": { "type": "string", "description": "The item's id." },
                        "priority": level,
                        "reason": { "type": "string", "description": "One line, tied to the focus or to a repeated failure." }
                    }
                }
            },
            "close": {
                "type": "array",
                "description": "Items you think should be closed. A person decides; nothing is closed for you.",
                "items": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["item", "reason"],
                    "properties": {
                        "item": { "type": "string", "description": "The item's id." },
                        "reason": { "type": "string", "description": "One line." }
                    }
                }
            }
        }
    })
}

/// The user prompt: the focus, the open items, the recent failure classes, and the rules.
pub fn brief(
    focus: Option<&str>,
    items: &[OpenItem],
    failures: &BTreeMap<String, Vec<JobId>>,
    caps: TierCaps,
) -> String {
    let mut text = format!("{HEADING}\n\n");
    text.push_str(
        "You set the order Athena works its open items in. You have no tools: everything you need is below.\n\n",
    );
    text.push_str("## Current focus\n\n");
    match focus {
        Some(focus) => text.push_str(&format!("{}\n\n", focus.trim())),
        None => text.push_str("No focus has been set. Tie a change to a repeated failure, or leave things as they are.\n\n"),
    }
    text.push_str("## Open items\n\n");
    for item in items {
        let title = crate::truncate_at_word(&one_line(&item.title), MAX_TITLE);
        let deps = if item.depends_on.is_empty() {
            "none".to_string()
        } else {
            item.depends_on.join(", ")
        };
        text.push_str(&format!(
            "- {}: {title}; rung {}; priority {} (set by {}); {} day(s) old; depends on {deps}\n",
            item.id,
            item.rung.map_or("not found yet", Ladder::as_str),
            item.priority.as_str(),
            item.setter.label(),
            item.age_days,
        ));
    }
    text.push_str(&format!(
        "\n## Failure classes in the last {FAILURE_WINDOW_DAYS} days (jobs that ended rejected or failed)\n\n"
    ));
    if failures.is_empty() {
        text.push_str("None.\n");
    }
    for (class, jobs) in failures {
        let ids: Vec<&str> = jobs.iter().map(JobId::as_str).collect();
        text.push_str(&format!(
            "- class `{class}`: {} job(s): {}\n",
            jobs.len(),
            ids.join(", ")
        ));
    }
    text.push_str(&format!(
        "\n## Your answer\n\n\
         Return `changes`: a priority (urgent, high, normal or low) for each item you would change, with a \
         one-line `reason` that ties the change to the focus or to a repeated failure. Leave an item out to \
         leave it as it is. Do not repeat an item's current priority. Items marked as set by a person are \
         never changed. At most {} items stay at urgent and {} at high; Athena counts every item that is \
         there now, and skips a change into a full tier.\n\n\
         Return `close` for an item that should not be worked at all, with a one-line `reason`. Nothing is \
         closed for you: Athena records a proposal for a person to decide. Both lists may be empty.\n",
        cap_text(caps.urgent),
        cap_text(caps.high),
    ));
    text
}

fn cap_text(cap: usize) -> String {
    if cap == usize::MAX {
        "any number".into()
    } else {
        cap.to_string()
    }
}

/// `text` as one line: runs of white space become one space.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A reason as stored: one line, at most [`MAX_REASON`] characters, and free of words the boundary scan
/// rejects. None when nothing is left or the text cannot be made plain.
fn clean_reason(text: &str) -> Option<String> {
    let line = crate::truncate_at_word(&one_line(text), MAX_REASON);
    let plain = plain_boundary_text(&line).ok()?;
    (!plain.trim().is_empty()).then_some(plain)
}

/// What the model's answer asks for, once the rules are applied. An entry that is malformed, names an item
/// that is not open, has no reason, or does not change anything is ignored. Pure: it reads only its inputs.
///
/// - An item set by a person is never changed.
/// - An item is changed at most once, by the first entry that names it.
/// - A change into urgent or high is skipped when the tier is already full. The count starts from every open
///   item at that tier, whoever set it, and follows each change as it is applied, in the model's order.
pub fn plan_changes(answer: &Value, items: &[OpenItem], caps: TierCaps) -> Proposed {
    let mut proposed = Proposed::default();
    let by_id: BTreeMap<&str, &OpenItem> = items.iter().map(|i| (i.id.as_str(), i)).collect();
    let in_tier = |tier: Priority| items.iter().filter(|i| i.priority == tier).count();
    let (mut urgent, mut high) = (in_tier(Priority::Urgent), in_tier(Priority::High));
    let mut touched: BTreeSet<&str> = BTreeSet::new();
    for entry in entries(answer, "changes") {
        let (Some(id), Some(level), Some(reason)) = (
            entry.get("item").and_then(Value::as_str),
            entry.get("priority").and_then(Value::as_str),
            entry.get("reason").and_then(Value::as_str),
        ) else {
            continue;
        };
        let Some(item) = by_id.get(id.trim()) else {
            continue;
        };
        let (Some(to), Some(reason)) = (Priority::parse(level), clean_reason(reason)) else {
            continue;
        };
        if item.setter == Setter::Person
            || to == item.priority
            || touched.contains(item.id.as_str())
        {
            continue;
        }
        if (to == Priority::Urgent && urgent >= caps.urgent)
            || (to == Priority::High && high >= caps.high)
        {
            continue;
        }
        for (tier, count) in [(item.priority, -1_i64), (to, 1)] {
            match tier {
                Priority::Urgent => urgent = urgent.saturating_add_signed(count as isize),
                Priority::High => high = high.saturating_add_signed(count as isize),
                _ => {}
            }
        }
        touched.insert(item.id.as_str());
        proposed.changes.push(Change {
            item: item.id.clone(),
            from: item.priority,
            to,
            reason,
        });
    }
    let mut proposed_close: BTreeSet<&str> = BTreeSet::new();
    for entry in entries(answer, "close") {
        let (Some(id), Some(reason)) = (
            entry.get("item").and_then(Value::as_str),
            entry.get("reason").and_then(Value::as_str),
        ) else {
            continue;
        };
        let (Some(item), Some(reason)) = (by_id.get(id.trim()), clean_reason(reason)) else {
            continue;
        };
        if !proposed_close.insert(item.id.as_str()) {
            continue;
        }
        proposed.closes.push(CloseProposal {
            item: item.id.clone(),
            title: item.title.clone(),
            reason,
        });
    }
    proposed
}

/// The entries of the answer's list `name`; none when it is missing or is no list.
fn entries<'a>(answer: &'a Value, name: &str) -> impl Iterator<Item = &'a Value> {
    answer
        .get(name)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

/// The work item for a close proposal: the steward's, a proposal on the brief-requirements rung. That rung is
/// above the steward's limit, so it can only ever be a proposal for a person.
fn proposal_work_item(proposal: &CloseProposal, job: &JobId, now: Timestamp) -> WorkItem {
    let title = format!(
        "Close \"{}\"? {}",
        one_line(&proposal.title),
        proposal.reason
    );
    let title = crate::truncate_at_word(&title, 200);
    WorkItem {
        id: RecordId::generate(now),
        at: now,
        role: AUTHOR.into(),
        kind: WorkItemKind::Proposal,
        key: proposal.key(),
        title: plain_boundary_text(&title).unwrap_or(title),
        rung: Ladder::BriefRequirements,
        source: Reference::path("records/jobs"),
        evidence: vec![Reference::job(job)],
    }
}

impl Athena {
    /// One triage pass over the open items `entries` (the plan entries of the queue). Makes one model call as
    /// the steward, writes the priority records and the close-proposal work items it asks for and the rules
    /// allow, in one records commit, and returns one line for each. Returns no lines and makes no call for a
    /// nested run or when nothing is open.
    ///
    /// An error means the call failed or its answer was unusable; nothing was written.
    pub fn triage(&self, entries: &[PlanEntry], caps: TierCaps) -> Result<Vec<String>> {
        if self.nested.is_some() || entries.is_empty() {
            return Ok(Vec::new());
        }
        let now = crate::now();
        let head = self.head()?;
        let jc = JobConfig::at(&self.git, &head)?;
        let records = Records::load(&self.git, &head)?;
        let items: Vec<OpenItem> = entries
            .iter()
            .map(|e| OpenItem::of_entry(e, &records, now))
            .collect();
        let since =
            Timestamp::from_second(now.as_second() - FAILURE_WINDOW_DAYS * 86_400).unwrap_or(now);
        let failures = failure_classes_since(&records, since);
        let focus = records.current_focus().map(|f| f.text.clone());
        let user_prompt = brief(focus.as_deref(), &items, &failures, caps);

        let role = roles::steward(now);
        let composed = steward_system_prompt(&role.prompt, role.version);
        // The call is no job, but it has an id of its own: its scratch directory and its invocation record
        // are named by it.
        let job = JobId::Id(RecordId::generate(now));
        let job_dir = self.state.job_dir(&job);
        let _cleanup = Cleanup(job_dir.clone());
        let workspace = job_dir.join("workspace");
        std::fs::create_dir_all(&workspace)?;
        let grant = ToolGrant {
            tools: Vec::new(),
            commands: BTreeMap::new(),
            writes: false,
            permission_mode: "dontAsk".into(),
        };
        // The steward borrows the verifier's model and budget (docs/design/roles-and-ladder.md).
        let spec = self.agent_spec(
            &jc,
            &job,
            1,
            Role::Steward,
            &jc.config.models.verifier,
            &composed,
            user_prompt,
            &grant,
            &workspace,
            Side::Read,
            &BTreeMap::new(),
            jc.config.budget.verifier_usd,
            jc.config.budget.verifier_timeout_s,
            schema(),
        )?;
        let facts = crate::invocation::InvocationFacts {
            job: &job,
            spec: &spec,
            composed: &composed,
            config: &jc.config,
            fake: self.is_fake(),
        };
        let redactor = self.redactor(&jc.config, Some(&job))?;
        let (output, record) = match self.invoke_and_record(&facts, &redactor)? {
            Ok(done) => done,
            Err(error) => bail!("the steward could not run: {error}"),
        };
        // The call cost money whatever the model said, so its record is kept before anything else.
        stash_invocation(&self.state, &record)?;
        if output.outcome != InvocationOutcome::Success {
            bail!(
                "the steward did not complete: {}",
                record.detail.as_deref().unwrap_or("no detail")
            );
        }
        let answer = output.structured.unwrap_or_default();
        if !answer.is_object() {
            bail!("the steward returned no valid triage answer");
        }

        let mut proposed = plan_changes(&answer, &items, caps);
        // What the model wrote goes into records, so it is redacted first.
        let mut reasons: Vec<String> = proposed
            .changes
            .iter()
            .map(|c| c.reason.clone())
            .chain(proposed.closes.iter().map(|c| c.reason.clone()))
            .collect();
        redactor.redact_all(&mut reasons)?;
        let mut reasons = reasons.into_iter();
        for change in proposed.changes.iter_mut() {
            change.reason = reasons.next().unwrap_or_default();
        }
        for close in proposed.closes.iter_mut() {
            close.reason = reasons.next().unwrap_or_default();
        }
        // A job with a close proposal already on record is not proposed again.
        let known: BTreeSet<&str> = records.work_items.iter().map(|w| w.key.as_str()).collect();
        proposed
            .closes
            .retain(|c| !known.contains(c.key().as_str()));
        if proposed.changes.is_empty() && proposed.closes.is_empty() {
            // The invocation record waits for the next steward commit.
            return Ok(Vec::new());
        }

        let mut files: Vec<NewFile> = Vec::new();
        let mut trailers: Vec<(&str, String)> = Vec::new();
        let mut lines: Vec<String> = Vec::new();
        if !records
            .roles
            .iter()
            .any(|r| r.name == role.name && r.version == role.version)
        {
            let record = AnyRecord::Role(Box::new(role.clone()));
            trailers.push(("Athena-Record", record.canonical_path()));
            files.push(NewFile::record(&record));
        }
        for change in &proposed.changes {
            let job_id = JobId::parse(&change.item).map_err(|e| anyhow::anyhow!("{e}"))?;
            let n = records
                .jobs
                .get(&change.item)
                .map_or(0, |j| j.priorities.iter().map(|p| p.n).max().unwrap_or(0))
                + 1;
            let priority = PriorityRecord {
                job: job_id,
                n,
                at: now,
                priority: change.to.as_str().into(),
                set_by: Some(AUTHOR.into()),
                reason: Some(change.reason.clone()),
            };
            priority
                .validate()
                .map_err(|e| anyhow::anyhow!("priority record for {}: {e}", change.item))?;
            trailers.push(("Athena-Job", change.item.clone()));
            files.push(NewFile::record(&AnyRecord::Priority(priority)));
            lines.push(change.line());
        }
        for close in &proposed.closes {
            let job_id = JobId::parse(&close.item).map_err(|e| anyhow::anyhow!("{e}"))?;
            let item = proposal_work_item(close, &job_id, now);
            item.validate()
                .map_err(|e| anyhow::anyhow!("work item {}: {e}", item.key))?;
            let record = AnyRecord::WorkItem(Box::new(item));
            trailers.push(("Athena-Record", record.canonical_path()));
            files.push(NewFile::record(&record));
            lines.push(close.line());
        }
        let waiting = pending_invocations(&self.state);
        for (_, file) in &waiting {
            trailers.push(("Athena-Record", file.path.clone()));
            files.push(file.clone());
        }
        let subject = format!(
            "Steward triage: {} priority change(s), {} close proposal(s)",
            proposed.changes.len(),
            proposed.closes.len()
        );
        let committer = Committer {
            git: &self.git,
            state: &self.state,
        };
        committer.commit_records(
            &files,
            &message(
                &subject,
                "Priority records set by the steward's triage of the open items against the current focus, \
                 each with its reason, and work items for any item it proposed closing. Nothing was closed.",
                &trailers,
            ),
        )?;
        for (path, _) in waiting {
            let _ = std::fs::remove_file(path);
        }
        Ok(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, priority: Priority, setter: Setter) -> OpenItem {
        OpenItem {
            id: id.into(),
            title: format!("Title of {id}"),
            rung: Some(Ladder::Implementation),
            priority,
            setter,
            age_days: 1,
            depends_on: Vec::new(),
        }
    }

    fn change(id: &str, level: &str, reason: &str) -> Value {
        json!({ "item": id, "priority": level, "reason": reason })
    }

    const CAPS: TierCaps = TierCaps { urgent: 2, high: 3 };

    fn applied(proposed: &Proposed) -> Vec<(&str, Priority)> {
        proposed
            .changes
            .iter()
            .map(|c| (c.item.as_str(), c.to))
            .collect()
    }

    #[test]
    fn changes_keep_their_order_and_carry_the_reason() {
        let items = [
            item("a", Priority::Normal, Setter::Words),
            item("b", Priority::Normal, Setter::Words),
        ];
        let answer = json!({
            "changes": [change("b", "high", "serves the focus"), change("a", "low", "off the focus")],
            "close": []
        });
        let proposed = plan_changes(&answer, &items, CAPS);
        assert_eq!(
            applied(&proposed),
            [("b", Priority::High), ("a", Priority::Low)]
        );
        assert_eq!(proposed.changes[0].reason, "serves the focus");
        assert_eq!(proposed.changes[0].from, Priority::Normal);
        assert_eq!(
            proposed.changes[0].line(),
            "triage     b normal -> high: serves the focus"
        );
    }

    #[test]
    fn an_item_a_person_set_is_never_changed() {
        let items = [
            item("p", Priority::Low, Setter::Person),
            item("s", Priority::Low, Setter::Role("steward".into())),
        ];
        let answer = json!({ "changes": [change("p", "urgent", "x"), change("s", "high", "y")] });
        assert_eq!(
            applied(&plan_changes(&answer, &items, CAPS)),
            [("s", Priority::High)]
        );
    }

    #[test]
    fn a_change_into_a_full_tier_is_skipped_and_person_set_items_count() {
        // One urgent item is a person's; the cap of urgent is 2, so only one more fits.
        let items = [
            item("p", Priority::Urgent, Setter::Person),
            item("a", Priority::Normal, Setter::Words),
            item("b", Priority::Normal, Setter::Words),
            item("c", Priority::Normal, Setter::Words),
        ];
        let answer = json!({ "changes": [
            change("a", "urgent", "1"), change("b", "urgent", "2"), change("c", "urgent", "3"),
        ]});
        assert_eq!(
            applied(&plan_changes(&answer, &items, CAPS)),
            [("a", Priority::Urgent)]
        );
    }

    #[test]
    fn moving_an_item_out_of_a_tier_makes_room_for_the_next() {
        let items = [
            item("u1", Priority::Urgent, Setter::Words),
            item("u2", Priority::Urgent, Setter::Words),
            item("n", Priority::Normal, Setter::Words),
        ];
        let answer = json!({ "changes": [
            change("n", "urgent", "full"), change("u1", "high", "make room"), change("n", "urgent", "again"),
        ]});
        // The first entry is skipped (full); u1 moves down; the second entry for `n` now fits.
        assert_eq!(
            applied(&plan_changes(&answer, &items, CAPS)),
            [("u1", Priority::High), ("n", Priority::Urgent)]
        );
    }

    #[test]
    fn invalid_entries_are_ignored() {
        let items = [item("a", Priority::Normal, Setter::Words)];
        let answer = json!({ "changes": [
            change("nope", "high", "unknown item"),
            change("a", "critical", "bad level"),
            change("a", "high", "   "),
            change("a", "normal", "no change"),
            json!({ "item": "a", "priority": "high" }),
            json!("not an object"),
            change("a", "HIGH", "case does not matter"),
            change("a", "low", "an item changes once"),
        ]});
        let proposed = plan_changes(&answer, &items, CAPS);
        assert_eq!(applied(&proposed), [("a", Priority::High)]);
        assert_eq!(plan_changes(&json!({}), &items, CAPS), Proposed::default());
        assert_eq!(
            plan_changes(&json!(null), &items, CAPS),
            Proposed::default()
        );
    }

    #[test]
    fn a_reason_is_one_line() {
        let items = [item("a", Priority::Normal, Setter::Words)];
        let long = "word ".repeat(100);
        let answer = json!({ "changes": [change("a", "high", &format!("first\nsecond  {long}"))] });
        let reason = &plan_changes(&answer, &items, CAPS).changes[0].reason;
        assert!(!reason.contains('\n') && reason.starts_with("first second word"));
        assert!(reason.chars().count() <= MAX_REASON, "{reason}");
    }

    #[test]
    fn a_close_proposal_is_a_proposal_and_changes_no_priority() {
        let items = [
            item("a", Priority::Normal, Setter::Words),
            item("p", Priority::Low, Setter::Person),
        ];
        let answer = json!({ "changes": [], "close": [
            { "item": "a", "reason": "off the focus" },
            { "item": "a", "reason": "twice" },
            { "item": "p", "reason": "a person's item can still be proposed" },
            { "item": "nope", "reason": "unknown" },
        ]});
        let proposed = plan_changes(&answer, &items, CAPS);
        assert!(proposed.changes.is_empty());
        let ids: Vec<&str> = proposed.closes.iter().map(|c| c.item.as_str()).collect();
        assert_eq!(ids, ["a", "p"]);
        assert_eq!(proposed.closes[0].key(), "triage-close:a");

        let job = JobId::parse("20261031-082446-vq7x").unwrap();
        let work = proposal_work_item(&proposed.closes[0], &job, crate::now());
        assert_eq!(work.role, "steward");
        assert_eq!(work.kind, WorkItemKind::Proposal);
        assert_eq!(work.rung, Ladder::BriefRequirements);
        assert!(!roles::steward(crate::now()).may_change(work.rung));
        assert_eq!(work.evidence, [Reference::job(&job)]);
        work.validate().unwrap();
    }

    #[test]
    fn the_brief_carries_the_focus_the_items_the_failures_and_the_caps() {
        let mut dependent = item("b", Priority::High, Setter::Person);
        dependent.depends_on = vec!["a".into()];
        let items = [item("a", Priority::Normal, Setter::Words), dependent];
        let failures = BTreeMap::from([(
            "docs".to_string(),
            vec![JobId::parse("20261031-082446-vq7x").unwrap()],
        )]);
        let text = brief(Some("speed first"), &items, &failures, CAPS);
        assert!(text.starts_with(HEADING));
        assert!(text.contains("speed first"));
        assert!(text.contains("- a: Title of a; rung implementation; priority normal (set by the request's own words); 1 day(s) old; depends on none"), "{text}");
        assert!(
            text.contains("- b: Title of b;")
                && text.contains("(set by a person)")
                && text.contains("depends on a"),
            "{text}"
        );
        assert!(text.contains("class `docs`: 1 job(s): 20261031-082446-vq7x"));
        assert!(
            text.contains("At most 2 items stay at urgent and 3 at high"),
            "{text}"
        );
        let none = brief(None, &items, &BTreeMap::new(), TierCaps::UNLIMITED);
        assert!(
            none.contains("No focus has been set") && none.contains("None."),
            "{none}"
        );
    }

    #[test]
    fn this_module_writes_no_outcome_and_no_close_record() {
        let source = include_str!("triage.rs");
        let code = source.split("#[cfg(test)]").next().unwrap();
        // Spelled in pieces so this test does not contain what it looks for.
        for forbidden in [
            concat!("AnyRecord::", "Outcome"),
            concat!("AnyRecord::", "Close"),
            concat!("Outcome", "State"),
        ] {
            assert!(!code.contains(forbidden), "{forbidden}");
        }
    }
}
