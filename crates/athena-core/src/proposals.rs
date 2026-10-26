//! Defaults proposals (`docs/design/defaults-loop.md` sections 8 and 9; build order step 3).
//!
//! When recorded introspections of one failure class span at least [`REPEAT_THRESHOLD`] distinct jobs, code
//! writes an `athena.defaults-proposal/1` record: a drafted default (id, statement, `applies_to`), the version
//! bump, and the introspections it cites. Every proposal is validated by building the full candidate
//! `defaults.toml` text and parsing it with [`DefaultsModel::parse`], by checking that the id is unique among
//! the model and the pending proposals, that the version is the base plus one, and that every cited record
//! passes the evidence-after-decision rule ([`after_start`]).
//!
//! **Athena never writes `defaults.toml`.** A proposal is a record. Accepting one submits an ordinary job
//! request that lists `defaults.toml` as a deliverable ([`accept_request`]); the landing is a verified one, and
//! [`apply_check`] is the test of the candidate that job produces. A test asserts that this file's code, above
//! its test module, opens nothing for writing.
//!
//! Rejecting a proposal appends a `rejected` record with the user's reason. The class is then suppressed until
//! [`REPEAT_THRESHOLD`] more distinct jobs have cases newer than the rejection (section 9). Without Jev the
//! statement and scope are drafted by code from the class entry and the cases' cause text, and the record's
//! `notes` say that no assumptions, signals or probabilities exist yet. No language model is called.

use crate::capability::JobConfig;
use crate::commits::{NewFile, message};
use crate::introspect::{CaseRow, Records, Starts, after_start, read_records};
use crate::store::Records as GitRecords;
use crate::{Athena, Ui};
use anyhow::{Result, bail};
use athena_data::records::{
    AnyRecord, ProposalKind, ProposalRecord, ProposalSource, ProposalStatus, ProposalVersion,
};
use athena_data::{DefaultsModel, JobId, RecordId, sha256_hex};
use athena_gate::DEFAULTS_PATH;
use jiff::Timestamp;
use std::collections::{BTreeMap, BTreeSet};

/// The distinct jobs of one failure class needed before a default is proposed. A design choice, movable by
/// calibration (`defaults-loop.md` section 7.2: N = 3), and not a setting in `athena.toml`.
pub const REPEAT_THRESHOLD: usize = 3;

/// What this build leaves out, written into every proposal.
const NOTES: &str = "Drafted by code from the failure class and the cause text of its cases. No Jev: the assumptions, \
signals and evidence probabilities of the design are not recorded.";

/// The newest record of each default id, oldest id first. A proposal's status changes by appending a record
/// with the same `id`, so the newest by `(at, record)` is current.
pub fn current(records: &[ProposalRecord]) -> Vec<&ProposalRecord> {
    let mut newest: BTreeMap<&str, &ProposalRecord> = BTreeMap::new();
    for p in records {
        let better = newest
            .get(p.id.as_str())
            .is_none_or(|old| (p.at, &p.record) > (old.at, &old.record));
        if better {
            newest.insert(p.id.as_str(), p);
        }
    }
    newest.into_values().collect()
}

/// The proposals that are open: current and pending.
pub fn open(records: &[ProposalRecord]) -> Vec<&ProposalRecord> {
    current(records)
        .into_iter()
        .filter(|p| p.status == ProposalStatus::Pending)
        .collect()
}

/// Lower case words of letters and digits joined by `-`: the id drafted from a class name.
pub fn slug(name: &str) -> String {
    name.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join("-")
}

fn at_of(case: &CaseRow) -> Option<Timestamp> {
    case.at.parse().ok()
}

/// The `source` text of the default's entry: the records that pulled it in.
fn source_text(p: &ProposalRecord) -> String {
    let cited: Vec<String> = p
        .sources
        .iter()
        .map(|s| format!("introspection {} (job {})", s.introspection, s.job))
        .collect();
    format!("defaults proposal {}: {}", p.id, cited.join(", "))
}

/// The full candidate `defaults.toml`: `base` with the proposal applied and the version raised to `to`. The
/// base's comments and layout are kept. This builds text in memory; it writes nothing.
pub fn build_candidate(base: &str, p: &ProposalRecord) -> Result<String> {
    use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, value};
    let mut doc: DocumentMut = base.parse()?;
    let base_version = doc.get("version").and_then(Item::as_integer);
    if base_version != Some(i64::from(p.version.from)) {
        bail!(
            "the proposal is built on version {}, but defaults.toml is at version {base_version:?}",
            p.version.from
        );
    }
    doc["version"] = value(i64::from(p.version.to));
    let applies: Array = p.applies_to.iter().map(String::as_str).collect();
    let tables = doc
        .entry("default")
        .or_insert(Item::ArrayOfTables(ArrayOfTables::new()))
        .as_array_of_tables_mut()
        .ok_or_else(|| anyhow::anyhow!("`default` in defaults.toml is not an array of tables"))?;
    match p.kind {
        ProposalKind::Add => {
            let mut table = Table::new();
            table["id"] = value(p.id.as_str());
            table["statement"] = value(p.statement.as_str());
            table["applies_to"] = value(applies);
            table["source"] = value(source_text(p));
            tables.push(table);
        }
        ProposalKind::Amend => {
            let Some(table) = tables
                .iter_mut()
                .find(|t| t.get("id").and_then(Item::as_str) == Some(p.id.as_str()))
            else {
                bail!("there is no default {:?} to amend", p.id);
            };
            let old = table
                .get("source")
                .and_then(Item::as_str)
                .unwrap_or("")
                .to_string();
            table["statement"] = value(p.statement.as_str());
            table["applies_to"] = value(applies);
            table["source"] = value(format!("{old}; {}", source_text(p)));
        }
    }
    Ok(doc.to_string())
}

/// Validates a proposal against the base `defaults.toml`, the records and the decisions' start times, and
/// returns the candidate text it hashes to. Every reason it is refused is listed.
///
/// - The candidate built from `base` must parse with [`DefaultsModel::parse`], and hash to `candidate_sha256`.
/// - The version is the base's plus one.
/// - The id is the model's for `amend`, absent from it for `add`, and not the id of another pending proposal.
/// - Every source is a recorded case of the proposal's class, in the job it names; it is later than the start
///   of the decision (the evidence-after-decision rule, `starts` keyed by the default's id); and the sources
///   span at least [`REPEAT_THRESHOLD`] distinct jobs.
pub fn validate(
    p: &ProposalRecord,
    base: &str,
    records: &Records,
    starts: &Starts,
) -> Result<String, Vec<String>> {
    let mut problems = Vec::new();
    let model = match DefaultsModel::parse(base) {
        Ok(model) => Some(model),
        Err(error) => {
            problems.push(format!("the base defaults.toml does not parse: {error}"));
            None
        }
    };
    if let Some(model) = &model {
        if p.version.from != model.version || p.version.to != model.version + 1 {
            problems.push(format!(
                "the version is {} to {}, not the base's {} plus one",
                p.version.from, p.version.to, model.version
            ));
        }
        match (p.kind, model.get(&p.id)) {
            (ProposalKind::Add, Some(_)) => {
                problems.push(format!("default {:?} already exists; amend it", p.id))
            }
            (ProposalKind::Amend, None) => {
                problems.push(format!("default {:?} does not exist; add it", p.id))
            }
            _ => {}
        }
    }
    if open(&records.proposals)
        .iter()
        .any(|o| o.id == p.id && o.failure_class != p.failure_class && o.record != p.record)
    {
        problems.push(format!(
            "default id {:?} is used by a pending proposal",
            p.id
        ));
    }
    for source in &p.sources {
        let case = records
            .cases
            .iter()
            .find(|c| c.id == source.introspection.as_str());
        let Some(case) = case else {
            problems.push(format!(
                "source {} is not a recorded introspection",
                source.introspection
            ));
            continue;
        };
        if case.job.as_deref() != Some(source.job.as_str())
            || case.failure_class != p.failure_class.as_str()
            || at_of(case) != Some(source.at)
        {
            problems.push(format!(
                "source {} does not match the record's job, class and time",
                source.introspection
            ));
        } else if !after_start(starts, &p.id, Some(source.at)) {
            problems.push(format!(
                "source {} is not later than the start of the decision {}",
                source.introspection, p.id
            ));
        }
    }
    let jobs: BTreeSet<&JobId> = p.sources.iter().map(|s| &s.job).collect();
    if jobs.len() < REPEAT_THRESHOLD {
        problems.push(format!(
            "the sources span {} distinct job(s); at least {REPEAT_THRESHOLD} are needed",
            jobs.len()
        ));
    }
    let candidate = build_candidate(base, p);
    match &candidate {
        Err(error) => problems.push(format!("no candidate defaults.toml: {error:#}")),
        Ok(text) => {
            if let Err(error) = DefaultsModel::parse(text) {
                problems.push(format!("the candidate does not parse: {error}"));
            }
            if sha256_hex(text.as_bytes()) != p.candidate_sha256 {
                problems.push("candidate_sha256 is not the hash of the candidate".into());
            }
        }
    }
    match (problems.is_empty(), candidate) {
        (true, Ok(text)) => Ok(text),
        _ => Err(problems),
    }
}

/// A class that has repeated enough but whose proposal was refused, with the reasons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub failure_class: String,
    pub reasons: Vec<String>,
}

/// What one run of the generator produced.
#[derive(Debug, Clone, Default)]
pub struct Generated {
    pub proposals: Vec<ProposalRecord>,
    pub refusals: Vec<Refusal>,
}

fn sentence(text: &str) -> String {
    let text = text.trim();
    if text.ends_with(['.', '!', '?']) {
        text.to_string()
    } else {
        format!("{text}.")
    }
}

/// Groups the recorded cases by failure class and proposes a default for each class whose cases span at least
/// [`REPEAT_THRESHOLD`] distinct jobs. Cases without a job do not count, and repeats within one job count
/// once. A class with a pending proposal is skipped. A class whose latest proposal was accepted or rejected
/// counts only the cases newer than that decision. `base` is the `defaults.toml` text the candidate is built
/// on. Nothing is written: the caller commits the records returned.
pub fn generate(
    base: &str,
    records: &Records,
    starts: &Starts,
    now: Timestamp,
    mut new_id: impl FnMut() -> RecordId,
) -> Result<Generated> {
    let model = DefaultsModel::parse(base)?;
    let mut by_class: BTreeMap<&str, Vec<&CaseRow>> = BTreeMap::new();
    for case in records.cases.iter().filter(|c| c.job.is_some()) {
        by_class.entry(&case.failure_class).or_default().push(case);
    }
    let mut out = Generated::default();
    let mut taken: BTreeSet<String> = open(&records.proposals)
        .iter()
        .map(|p| p.id.clone())
        .collect();
    for (class, cases) in by_class {
        let Ok(class_id) = RecordId::parse(class) else {
            continue;
        };
        if open(&records.proposals)
            .iter()
            .any(|p| p.failure_class == class_id)
        {
            continue;
        }
        // A decision on this class: only newer cases are evidence for a new proposal.
        let decided = records
            .proposals
            .iter()
            .filter(|p| p.failure_class == class_id)
            .filter(|p| {
                matches!(
                    p.status,
                    ProposalStatus::Rejected { .. } | ProposalStatus::Accepted { .. }
                )
            })
            .map(|p| p.at)
            .max();
        let live: Vec<&CaseRow> = cases
            .into_iter()
            .filter(|c| decided.is_none_or(|d| at_of(c).is_some_and(|at| at > d)))
            .collect();
        let jobs: BTreeSet<&str> = live.iter().filter_map(|c| c.job.as_deref()).collect();
        if jobs.len() < REPEAT_THRESHOLD {
            continue;
        }
        let Some(entry) = records.classes.iter().find(|c| c.id == class) else {
            out.refusals.push(Refusal {
                failure_class: class.to_string(),
                reasons: vec!["the class is not in the registry".into()],
            });
            continue;
        };
        let id = slug(&entry.name);
        let newest = live.last().expect("a live class has cases");
        let drafted = format!(
            "{} {}",
            sentence(&format!("Prevent this failure: {}", entry.name)),
            sentence(&format!(
                "{} The recorded cause: {}",
                sentence(&entry.description),
                newest.cause.trim()
            )),
        );
        let (kind, statement) = match model.get(&id) {
            Some(existing) => (
                ProposalKind::Amend,
                format!("{} {drafted}", existing.statement.trim()),
            ),
            None => (ProposalKind::Add, drafted),
        };
        let applies_to: Vec<String> = live
            .iter()
            .map(|c| c.class_of_work.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let sources = live
            .iter()
            .map(|c| {
                Ok(ProposalSource {
                    introspection: RecordId::parse(&c.id)?,
                    intervention: records
                        .case_interventions
                        .get(&c.id)
                        .map(|i| RecordId::parse(i))
                        .transpose()?,
                    job: JobId::parse(c.job.as_deref().expect("live cases have a job"))?,
                    at: at_of(c).ok_or_else(|| anyhow::anyhow!("case {} has no time", c.id))?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let mut proposal = ProposalRecord {
            record: new_id(),
            at: now,
            id: id.clone(),
            kind,
            statement,
            applies_to,
            version: ProposalVersion {
                from: model.version,
                to: model.version + 1,
            },
            sources,
            failure_class: class_id,
            status: ProposalStatus::Pending,
            candidate_sha256: String::new(),
            notes: NOTES.into(),
        };
        if taken.contains(&id) {
            out.refusals.push(Refusal {
                failure_class: class.to_string(),
                reasons: vec![format!("default id {id:?} is used by a pending proposal")],
            });
            continue;
        }
        match build_candidate(base, &proposal) {
            Ok(text) => proposal.candidate_sha256 = sha256_hex(text.as_bytes()),
            Err(error) => {
                out.refusals.push(Refusal {
                    failure_class: class.to_string(),
                    reasons: vec![format!("{error:#}")],
                });
                continue;
            }
        }
        match validate(&proposal, base, records, starts) {
            Ok(_) => {
                taken.insert(id);
                out.proposals.push(proposal);
            }
            Err(reasons) => out.refusals.push(Refusal {
                failure_class: class.to_string(),
                reasons,
            }),
        }
    }
    Ok(out)
}

/// The record that rejects `proposal`, with the user's reason. It is appended, never an edit.
pub fn rejection(
    proposal: &ProposalRecord,
    reason: &str,
    record: RecordId,
    now: Timestamp,
) -> Result<ProposalRecord> {
    if reason.trim().is_empty() {
        bail!("a rejection gives a reason");
    }
    let mut next = proposal.clone();
    next.record = record;
    next.at = now;
    next.status = ProposalStatus::Rejected {
        reason: reason.trim().to_string(),
    };
    Ok(next)
}

/// The record that marks `proposal` accepted by the job that applies it.
pub fn acceptance(
    proposal: &ProposalRecord,
    job: JobId,
    record: RecordId,
    now: Timestamp,
) -> ProposalRecord {
    let mut next = proposal.clone();
    next.record = record;
    next.at = now;
    next.status = ProposalStatus::Accepted { job };
    next
}

/// The request text of the ordinary job that applies a proposal. It names `defaults.toml` as the deliverable
/// and states what the verifier checks ([`apply_check`]); the edit is the producer's, never Athena's.
pub fn accept_request(p: &ProposalRecord) -> String {
    format!(
        "apply defaults proposal {id}\n\n\
         Deliverable: defaults.toml (modify). Make one change to it and touch no other file.\n\
         - {verb} the default with id = \"{id}\", statement = \"{statement}\", applies_to = [{applies}].\n\
         - Set version to {to} (it is {from} now).\n\
         - Set its source to cite the proposal's records: {source}.\n\
         - Leave every other entry as it is.\n\
         The file must parse as a defaults model. Proposal record: {record}; candidate hash: {sha}.",
        id = p.id,
        verb = match p.kind {
            ProposalKind::Add => "Add",
            ProposalKind::Amend => "Amend",
        },
        statement = p.statement,
        applies = p
            .applies_to
            .iter()
            .map(|a| format!("\"{a}\""))
            .collect::<Vec<_>>()
            .join(", "),
        to = p.version.to,
        from = p.version.from,
        source = source_text(p),
        record = p.record,
        sha = p.candidate_sha256,
    )
}

/// The apply-as-a-job check (section 9): whether `candidate` is the `defaults.toml` that applies `p` to
/// `base`. The id and statement equal the proposal's, the version is the base's plus one, every other entry is
/// unchanged, and [`DefaultsModel::parse`] accepts the file. Returns every problem found; empty means it
/// applies the proposal.
pub fn apply_check(p: &ProposalRecord, base: &str, candidate: &str) -> Vec<String> {
    let mut problems = Vec::new();
    let candidate = match DefaultsModel::parse(candidate) {
        Ok(model) => model,
        Err(error) => return vec![format!("the candidate does not parse: {error}")],
    };
    let base = match DefaultsModel::parse(base) {
        Ok(model) => model,
        Err(error) => return vec![format!("the base does not parse: {error}")],
    };
    if candidate.version != base.version + 1 || p.version.from != base.version {
        problems.push(format!(
            "the version is {}, not the base's {} plus one",
            candidate.version, base.version
        ));
    }
    match candidate.get(&p.id) {
        None => problems.push(format!("the candidate has no default with id {:?}", p.id)),
        Some(entry) if entry.statement.trim() != p.statement.trim() => {
            problems.push(format!("the statement of {:?} is not the proposal's", p.id))
        }
        Some(_) => {}
    }
    match (p.kind, base.get(&p.id)) {
        (ProposalKind::Add, Some(_)) => problems.push(format!("{:?} is already in the base", p.id)),
        (ProposalKind::Amend, None) => problems.push(format!("{:?} is not in the base", p.id)),
        _ => {}
    }
    for old in base.defaults.iter().filter(|d| d.id != p.id) {
        if candidate.get(&old.id) != Some(old) {
            problems.push(format!("the default {:?} was changed or removed", old.id));
        }
    }
    let added = candidate
        .defaults
        .iter()
        .filter(|d| d.id != p.id && base.get(&d.id).is_none())
        .count();
    if added > 0 {
        problems.push(format!("{added} other default(s) were added"));
    }
    problems
}

impl Athena {
    fn base_defaults(&self, head: &str) -> Result<String> {
        let bytes = self
            .git
            .show_file(head, DEFAULTS_PATH)?
            .ok_or_else(|| anyhow::anyhow!("{DEFAULTS_PATH} is missing at HEAD"))?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    /// The time of the latest commit that changed `defaults.toml`: the start of every default's decision.
    fn defaults_start(&self, head: &str, model: &DefaultsModel) -> Result<Starts> {
        let text = self
            .git
            .run(["log", "-1", "--format=%cI", head, "--", DEFAULTS_PATH])?;
        let mut starts = Starts::new();
        if let Ok(at) = text.trim().parse::<Timestamp>() {
            for d in &model.defaults {
                starts.insert(d.id.clone(), at);
            }
        }
        Ok(starts)
    }

    fn fresh_record_id(&self, known: &GitRecords) -> RecordId {
        let mut id = RecordId::generate(crate::now());
        while known.has_record_id(&id) {
            id = RecordId::generate(crate::now());
        }
        id
    }

    /// `athena propose`: reads the introspection records, writes a pending proposal for every class that has
    /// repeated across enough distinct jobs, and commits them. Returns what was written and what was refused.
    pub fn propose(&self, ui: &dyn Ui) -> Result<Generated> {
        let head = self.head()?;
        let jc = JobConfig::at(&self.git, &head)?;
        let base = self.base_defaults(&head)?;
        let known = GitRecords::load(&self.git, &head)?;
        let records = read_records(self.git.dir())?;
        let starts = self.defaults_start(&head, &jc.defaults)?;
        let mut used: Vec<RecordId> = Vec::new();
        let generated = generate(&base, &records, &starts, crate::now(), || {
            let mut id = self.fresh_record_id(&known);
            while used.contains(&id) {
                id = self.fresh_record_id(&known);
            }
            used.push(id.clone());
            id
        })?;
        for refusal in &generated.refusals {
            ui.line(&format!(
                "proposal for class {} refused: {}",
                refusal.failure_class,
                refusal.reasons.join("; ")
            ));
        }
        if generated.proposals.is_empty() {
            ui.line("no class has repeated across enough distinct jobs");
            return Ok(generated);
        }
        let files: Vec<NewFile> = generated
            .proposals
            .iter()
            .map(|p| NewFile::record(&AnyRecord::DefaultsProposal(Box::new(p.clone()))))
            .collect();
        let trailers: Vec<(&str, String)> = generated
            .proposals
            .iter()
            .map(|p| {
                (
                    "Athena-Record",
                    AnyRecord::DefaultsProposal(Box::new(p.clone())).canonical_path(),
                )
            })
            .collect();
        self.commit_records(
            &files,
            &message(
                "Propose defaults from repeated introspections",
                "Each proposal cites the introspections of one failure class across distinct jobs. It is a record: defaults.toml is not edited.",
                &trailers,
            ),
        )?;
        for p in &generated.proposals {
            ui.line(&format!("proposed default {} ({})", p.id, p.kind.as_str()));
        }
        Ok(generated)
    }

    fn pending(&self, records: &GitRecords, id: &str) -> Result<ProposalRecord> {
        match open(&records.proposals).into_iter().find(|p| p.id == id) {
            Some(p) => Ok(p.clone()),
            None => bail!("no pending proposal {id:?}; `athena status` lists them"),
        }
    }

    /// `athena reject-proposal`: appends a `rejected` record with the reason. The class is suppressed until
    /// [`REPEAT_THRESHOLD`] more distinct jobs have cases newer than this record.
    pub fn reject_proposal(&self, id: &str, reason: &str) -> Result<ProposalRecord> {
        let head = self.head()?;
        let known = GitRecords::load(&self.git, &head)?;
        let pending = self.pending(&known, id)?;
        let next = rejection(&pending, reason, self.fresh_record_id(&known), crate::now())?;
        let record = AnyRecord::DefaultsProposal(Box::new(next.clone()));
        let path = record.canonical_path();
        self.commit_records(
            &[NewFile::record(&record)],
            &message(
                &format!("Reject defaults proposal {id}"),
                "The user's reason is in the record. The class is not proposed again until new cases arrive.",
                &[("Athena-Record", path)],
            ),
        )?;
        Ok(next)
    }

    /// `athena accept-proposal`: submits the ordinary job request that applies the proposal, through the usual
    /// reading, and then records the proposal as accepted with that job. `defaults.toml` is not touched here;
    /// the job edits it and lands like any other. Returns the reading.
    pub fn accept_proposal(&self, id: &str, ui: &dyn Ui) -> Result<crate::reading::Proposal> {
        let head = self.head()?;
        let known = GitRecords::load(&self.git, &head)?;
        let pending = self.pending(&known, id)?;
        let reading = self.ask(&accept_request(&pending), ui)?;
        let known = GitRecords::load(&self.git, &self.head()?)?;
        let next = acceptance(
            &pending,
            reading.job.clone(),
            self.fresh_record_id(&known),
            crate::now(),
        );
        let record = AnyRecord::DefaultsProposal(Box::new(next));
        let path = record.canonical_path();
        self.commit_records(
            &[NewFile::record(&record)],
            &message(
                &format!("Accept defaults proposal {id}"),
                "The proposal is applied by the job named here, which lists defaults.toml as a deliverable.",
                &[
                    ("Athena-Record", path),
                    ("Athena-Job", reading.job.as_str().to_string()),
                ],
            ),
        )?;
        Ok(reading)
    }

    /// `athena check-proposal`: runs [`apply_check`] on `candidate` against `defaults.toml` at HEAD.
    pub fn check_proposal(&self, id: &str, candidate: &str) -> Result<Vec<String>> {
        let head = self.head()?;
        let known = GitRecords::load(&self.git, &head)?;
        let proposal = current(&known.proposals)
            .into_iter()
            .find(|p| p.id == id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("no proposal {id:?}"))?;
        Ok(apply_check(
            &proposal,
            &self.base_defaults(&head)?,
            candidate,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::records::{
        ClassStatus, FailureClass, IntrospectionRecord, IntrospectionStatus, IntrospectionTrigger,
        PreventionKind,
    };
    use std::path::Path;

    const BASE: &str = "# header comment\nversion = 1\n\n[[default]]\nid = \"small\"\nstatement = \"Prefer one focused change.\"\napplies_to = [\"*\"]\nsource = \"user\"\n";
    const CLASS: &str = "20261001-000000-c001";

    fn ts(text: &str) -> Timestamp {
        text.parse().unwrap()
    }

    fn rid(day: u32, n: u32) -> RecordId {
        RecordId::from_parts(ts(&format!("2026-10-{day:02}T00:00:00Z")), n)
    }

    fn write_record(root: &Path, record: &AnyRecord) {
        let path = root.join(record.canonical_path());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, record.to_json()).unwrap();
    }

    fn write_class(root: &Path, class: &str, name: &str) {
        write_record(
            root,
            &AnyRecord::FailureClass(FailureClass {
                id: RecordId::parse(class).unwrap(),
                at: ts("2026-10-01T00:00:00Z"),
                name: name.into(),
                description: "A new field has no reader.".into(),
                status: ClassStatus::Proposed,
                proposed_by: rid(1, 1),
            }),
        );
    }

    /// One recorded introspection of `class`, on `day` (the date of the record), in job number `job`.
    fn write_case(root: &Path, class: &str, day: u32, n: u32, job: u32) -> RecordId {
        let id = rid(day, n);
        write_record(
            root,
            &AnyRecord::Introspection(Box::new(IntrospectionRecord {
                id: id.clone(),
                at: ts(&format!("2026-10-{day:02}T00:00:00Z")),
                trigger: IntrospectionTrigger::Correction,
                job: Some(JobId::parse(&format!("202610{day:02}-000000-j{job:03}")).unwrap()),
                intervention: Some(rid(day, n + 500)),
                class_of_work: "code".into(),
                category: "corrected-output".into(),
                what_happened: Some("Added a field with no reader.".into()),
                failure_class: Some(RecordId::parse(class).unwrap()),
                new_class: false,
                earlier_cases: vec![],
                decisions_matched: vec![],
                cause: Some("Nothing ties a field to its first use.".into()),
                prevention_kind: Some(PreventionKind::Default),
                prevention: Some("Add the field with its reader.".into()),
                proposed_request: Some("Add the reader.".into()),
                evidence: None,
                status: IntrospectionStatus::Recorded,
                failure: None,
                flags: vec![],
                dropped_ids: vec![],
                planning_stopped: false,
                from_jev: false,
                jev: None,
            })),
        );
        id
    }

    fn fixture() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        write_class(dir.path(), CLASS, "Field with no use");
        dir
    }

    fn run(dir: &Path, starts: &Starts) -> Generated {
        let records = read_records(dir).unwrap();
        let mut n = 900;
        generate(BASE, &records, starts, ts("2026-10-20T00:00:00Z"), || {
            n += 1;
            rid(20, n)
        })
        .unwrap()
    }

    fn pending_proposal(dir: &Path) -> ProposalRecord {
        for day in [2, 3, 4] {
            write_case(dir, CLASS, day, 1, day);
        }
        run(dir, &Starts::new()).proposals.remove(0)
    }

    #[test]
    fn cases_of_a_class_in_fewer_than_n_jobs_yield_no_proposal() {
        let dir = fixture();
        write_case(dir.path(), CLASS, 2, 1, 2);
        write_case(dir.path(), CLASS, 3, 1, 3);
        let generated = run(dir.path(), &Starts::new());
        assert!(generated.proposals.is_empty() && generated.refusals.is_empty());
    }

    #[test]
    fn repeats_within_one_job_do_not_count() {
        let dir = fixture();
        for n in 1..=5 {
            write_case(dir.path(), CLASS, 2, n, 2);
        }
        write_case(dir.path(), CLASS, 3, 1, 3);
        assert!(run(dir.path(), &Starts::new()).proposals.is_empty());
    }

    #[test]
    fn n_distinct_jobs_yield_one_pending_proposal_citing_those_records() {
        let dir = fixture();
        let cited: Vec<RecordId> = [2, 3, 4]
            .iter()
            .map(|d| write_case(dir.path(), CLASS, *d, 1, *d))
            .collect();
        let generated = run(dir.path(), &Starts::new());
        let [p] = generated.proposals.as_slice() else {
            panic!("one proposal, got {:?}", generated.proposals)
        };
        assert_eq!(p.id, "field-with-no-use");
        assert_eq!(p.kind, ProposalKind::Add);
        assert_eq!(p.status, ProposalStatus::Pending);
        assert_eq!(p.version, ProposalVersion { from: 1, to: 2 });
        assert_eq!(p.applies_to, ["code"]);
        assert_eq!(
            p.sources
                .iter()
                .map(|s| s.introspection.clone())
                .collect::<Vec<_>>(),
            cited
        );
        assert!(p.sources.iter().all(|s| s.intervention.is_some()));
        assert!(
            p.statement
                .contains("Nothing ties a field to its first use.")
        );
        // The candidate is the base with the entry added and the version raised; the comment survives.
        let candidate = build_candidate(BASE, p).unwrap();
        assert!(candidate.starts_with("# header comment\nversion = 2\n"));
        let model = DefaultsModel::parse(&candidate).unwrap();
        assert_eq!(model.defaults.len(), 2);
        assert_eq!(sha256_hex(candidate.as_bytes()), p.candidate_sha256);
        assert!(model.get(&p.id).unwrap().source.contains(cited[0].as_str()));
        // Written the way records are: it round-trips and sits at its canonical path.
        let record = AnyRecord::DefaultsProposal(Box::new(p.clone()));
        assert!(AnyRecord::parse("x", record.to_json().as_bytes()).is_ok());
        // A class with a pending proposal is not proposed again.
        write_record(dir.path(), &record);
        assert!(run(dir.path(), &Starts::new()).proposals.is_empty());
    }

    #[test]
    fn a_candidate_that_fails_to_parse_is_refused() {
        let dir = fixture();
        let mut p = pending_proposal(dir.path());
        let records = read_records(dir.path()).unwrap();
        assert!(validate(&p, BASE, &records, &Starts::new()).is_ok());
        p.statement = "  ".into();
        let problems = validate(&p, BASE, &records, &Starts::new()).unwrap_err();
        assert!(
            problems.iter().any(|m| m.contains("does not parse")),
            "{problems:?}"
        );
        // A base that is not a defaults model gives no candidate.
        let p = pending_proposal(dir.path());
        assert!(validate(&p, "version = 'x'", &records, &Starts::new()).is_err());
    }

    #[test]
    fn evidence_that_is_not_later_than_the_decision_is_refused() {
        let dir = fixture();
        for day in [2, 3, 4] {
            write_case(dir.path(), CLASS, day, 1, day);
        }
        // An amend of "small": the decision started on the 3rd, so the cases of the 2nd and 3rd are no evidence.
        write_class(dir.path(), CLASS, "small");
        let mut starts = Starts::new();
        starts.insert("small".into(), ts("2026-10-03T00:00:00Z"));
        let generated = run(dir.path(), &starts);
        assert!(generated.proposals.is_empty());
        let [refusal] = generated.refusals.as_slice() else {
            panic!("one refusal, got {:?}", generated.refusals)
        };
        assert!(
            refusal.reasons.iter().any(|r| r.contains("not later than")),
            "{refusal:?}"
        );
        // With the decision before all the evidence, the amend is proposed.
        starts.insert("small".into(), ts("2026-10-01T12:00:00Z"));
        let generated = run(dir.path(), &starts);
        let [p] = generated.proposals.as_slice() else {
            panic!("an amend, got {:?}", generated.refusals)
        };
        assert_eq!(p.kind, ProposalKind::Amend);
        assert!(p.statement.starts_with("Prefer one focused change."));
        let candidate = DefaultsModel::parse(&build_candidate(BASE, p).unwrap()).unwrap();
        assert_eq!(candidate.defaults.len(), 1);
    }

    #[test]
    fn an_id_used_by_a_pending_proposal_or_the_model_is_refused() {
        let dir = fixture();
        let p = pending_proposal(dir.path());
        let mut clash = p.clone();
        clash.record = rid(21, 1);
        clash.failure_class = RecordId::parse("20261001-000000-c002").unwrap();
        write_record(dir.path(), &AnyRecord::DefaultsProposal(Box::new(clash)));
        let records = read_records(dir.path()).unwrap();
        let problems = validate(&p, BASE, &records, &Starts::new()).unwrap_err();
        assert!(problems.iter().any(|m| m.contains("pending proposal")));
        let mut add_existing = p.clone();
        add_existing.id = "small".into();
        assert!(
            validate(&add_existing, BASE, &records, &Starts::new())
                .unwrap_err()
                .iter()
                .any(|m| m.contains("already exists"))
        );
    }

    #[test]
    fn a_rejection_suppresses_the_class_until_n_newer_distinct_jobs_arrive() {
        let dir = fixture();
        let p = pending_proposal(dir.path());
        // The proposal was written on the 20th; the rejection is later.
        let rejected = rejection(&p, "Too broad.", rid(21, 7), ts("2026-10-21T00:00:00Z")).unwrap();
        assert!(rejection(&p, " ", rid(21, 8), ts("2026-10-21T00:00:00Z")).is_err());
        write_record(
            dir.path(),
            &AnyRecord::DefaultsProposal(Box::new(p.clone())),
        );
        write_record(
            dir.path(),
            &AnyRecord::DefaultsProposal(Box::new(rejected.clone())),
        );
        let records = read_records(dir.path()).unwrap();
        assert!(open(&records.proposals).is_empty());
        assert!(run(dir.path(), &Starts::new()).proposals.is_empty());
        // Two newer jobs are not enough, and neither are old cases.
        write_case(dir.path(), CLASS, 22, 1, 22);
        write_case(dir.path(), CLASS, 23, 1, 23);
        write_case(dir.path(), CLASS, 23, 2, 23);
        assert!(run(dir.path(), &Starts::new()).proposals.is_empty());
        // The third newer job brings the proposal back, citing only the newer cases.
        write_case(dir.path(), CLASS, 24, 1, 24);
        let generated = run(dir.path(), &Starts::new());
        let [again] = generated.proposals.as_slice() else {
            panic!("a new proposal, got {:?}", generated.refusals)
        };
        assert!(again.sources.iter().all(|s| s.at > rejected.at));
        assert_eq!(again.status, ProposalStatus::Pending);
    }

    #[test]
    fn current_is_the_newest_record_of_each_id() {
        let dir = fixture();
        let p = pending_proposal(dir.path());
        let accepted = acceptance(
            &p,
            JobId::parse("20261021-000000-j001").unwrap(),
            rid(21, 2),
            ts("2026-10-21T00:00:00Z"),
        );
        let all = vec![accepted.clone(), p.clone()];
        assert_eq!(current(&all), vec![&accepted]);
        assert!(open(&all).is_empty());
        assert_eq!(open(std::slice::from_ref(&p)), vec![&p]);
    }

    #[test]
    fn the_apply_check_accepts_the_matching_candidate_and_refuses_the_others() {
        let dir = fixture();
        let p = pending_proposal(dir.path());
        let good = build_candidate(BASE, &p).unwrap();
        assert!(apply_check(&p, BASE, &good).is_empty());
        let said = |candidate: &str, needle: &str| {
            let problems = apply_check(&p, BASE, candidate);
            assert!(
                problems.iter().any(|m| m.contains(needle)),
                "{needle:?} not in {problems:?}"
            );
        };
        said(
            &good.replace(&p.statement, "Something else entirely."),
            "statement",
        );
        said(&good.replace(&p.id, "other-id"), "no default with id");
        said(&good.replace("version = 2", "version = 3"), "version");
        said(&good.replace("version = 2", "version = 1"), "version");
        said(
            &good.replace("[[default]]\nid", "[[default]]\nidd"),
            "parse",
        );
        said(
            &good.replace("Prefer one focused change.", "Prefer big changes."),
            "was changed",
        );
        said("not toml at all [", "parse");
    }

    #[test]
    fn accepting_makes_a_request_that_lists_defaults_toml_as_a_deliverable() {
        let dir = fixture();
        let p = pending_proposal(dir.path());
        let text = accept_request(&p);
        assert!(text.starts_with("apply defaults proposal field-with-no-use"));
        assert!(text.contains("Deliverable: defaults.toml (modify)"));
        assert!(text.contains(&p.statement) && text.contains(&p.candidate_sha256));
        assert!(text.contains("version to 2"));
    }

    #[test]
    fn the_proposal_path_never_opens_defaults_toml_for_writing() {
        let source = include_str!("proposals.rs");
        let code = &source[..source.find("#[cfg(test)]").unwrap()];
        for forbidden in [
            "File::create",
            "OpenOptions",
            "fs::write",
            "fs::rename",
            "fs::copy",
            "fs::remove",
            "write_all",
            "set_len",
        ] {
            assert!(
                !code.contains(forbidden),
                "the proposal path uses {forbidden}"
            );
        }
        // Everything that leaves this module goes through the one records commit.
        assert!(code.contains("commit_records"));
    }
}
