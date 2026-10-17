//! The auditor: a role that judges a job's real outcome against `BRIEF.md` and the owner's goals, and records
//! findings that each cite their evidence (`docs/design/roles-and-ladder.md`).
//!
//! The auditor exists so that the yardstick a job is judged by is not the job's own. The verifier judges a
//! change against the acceptance criteria the reader wrote for that job. The auditor is never given those
//! criteria, the job's verification, or any other record about the job. It is given `BRIEF.md` (in its brief),
//! a short description of the request and its state (`OUTCOME.md`), and the files the job actually changed,
//! as they stood in the job's commit (`outcome/`). It reads with `Read`, `Grep` and `Glob` and no commands.
//!
//! The model's report is validated before it becomes an `athena.audit/1` record. A finding with no evidence
//! refuses the whole report, so a record never holds a claim with nothing behind it. The invocation is recorded
//! whether or not the report is usable, because it cost money either way.
//!
//! **What protects the auditor's prompt, and what does not.** The prompt is a constant in
//! `athena_data::roles` (`AUDITOR_PROMPT`). That is the only protection: a job that edits that file changes
//! the yardstick it is judged by. The warden role, a separate request, will enforce the protection by
//! mechanism. Until then the protection is prose-only: it is written down here and in the design document, and
//! no code checks it.

use crate::Athena;
use crate::capability::{JobConfig, ToolGrant};
use crate::commits::{Committer, NewFile, message};
use crate::store::{JobRecords, Records};
use anyhow::{Context, Result, bail};
use athena_agent::env::Side;
use athena_context::outputs::{AuditorOutput, auditor_schema};
use athena_context::{Composed, auditor_system_prompt};
use athena_data::records::{AnyRecord, Audit, AuditFinding, Invocation, InvocationOutcome, Role};
use athena_data::roles::{self, RoleDefinition};
use athena_data::{JobId, RecordId};
use jiff::Timestamp;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

/// The file in the auditor's workspace that describes the request and its state.
pub const OUTCOME_FILE: &str = "OUTCOME.md";

/// The directory in the auditor's workspace that holds the files the job changed.
pub const OUTCOME_DIR: &str = "outcome";

/// The most changed files, and the most characters of each, that the auditor is given.
pub const MAX_FILES: usize = 40;
pub const MAX_FILE_CHARS: usize = 60_000;

/// The most findings, and the longest finding text or evidence, that an audit keeps.
pub const MAX_FINDINGS: usize = 20;
const MAX_TEXT: usize = 1_000;

/// The auditor's user prompt. It carries `BRIEF.md` in full and names the job; it never carries the job's
/// acceptance criteria.
pub fn brief(brief_md: &str, job: &JobId) -> String {
    format!(
        "# Audit run\n\nJudge the real outcome of job {job} against BRIEF.md and the owner's goals in it. \
         Do not judge it against anything the job said about itself. In the current directory, `{OUTCOME_FILE}` \
         describes the request and how the job ended, and `{OUTCOME_DIR}/` holds the files the job changed, as \
         they stood in its commit. Read them with Read, Grep and Glob. Nothing else is available and nothing \
         may be changed.\n\n\
         ## Your report\n\nReturn `summary` (one or two sentences: does the outcome serve BRIEF.md and the \
         owner's goals?) and `findings`. Each finding has `text` (one or two sentences) and `evidence` (a file \
         and line, or a passage quoted from the outcome). A finding without evidence is dropped, and a report \
         with one is refused.\n\n\
         ## BRIEF.md\n\n{brief_md}\n"
    )
}

/// What the auditor is told about the job: the request, how it ended, and the files it changed. No criteria,
/// no verification, no findings of the verifier.
pub fn outcome_text(job: &JobRecords, changed: &[String]) -> String {
    let mut text = format!("# Outcome of job {}\n\n", job.id());
    text.push_str(&format!("Title: {}\n\n", job.title()));
    text.push_str("## Request\n\n");
    match &job.request {
        Some(request) => text.push_str(request.text.trim()),
        None => text.push_str("(no request is recorded)"),
    }
    text.push_str("\n\n## How it ended\n\n");
    match job.later_outcome.as_ref().or(job.outcome.as_ref()) {
        Some(outcome) => {
            text.push_str(&format!("{}: {}\n", outcome.state.as_str(), outcome.reason))
        }
        None => text.push_str("no outcome is recorded\n"),
    }
    text.push_str("\n## Files changed\n\n");
    if changed.is_empty() {
        text.push_str("none\n");
    }
    for path in changed {
        text.push_str(&format!("- {path}\n"));
    }
    text
}

/// Validates the model's report into an audit record. A finding with no evidence, or an empty summary, is
/// refused.
pub fn validate_report(
    report: AuditorOutput,
    target: &JobId,
    id: RecordId,
    now: Timestamp,
) -> Result<Audit> {
    if report.findings.len() > MAX_FINDINGS {
        bail!("an audit keeps at most {MAX_FINDINGS} findings");
    }
    if report.summary.chars().count() > MAX_TEXT
        || report
            .findings
            .iter()
            .any(|f| f.text.chars().count() > MAX_TEXT || f.evidence.chars().count() > MAX_TEXT)
    {
        bail!("a summary, a finding's text and its evidence are at most {MAX_TEXT} characters");
    }
    let audit = Audit {
        id,
        at: now,
        target: target.clone(),
        summary: report.summary.trim().to_string(),
        findings: report
            .findings
            .into_iter()
            .map(|f| AuditFinding {
                text: f.text.trim().to_string(),
                evidence: f.evidence.trim().to_string(),
            })
            .collect(),
    };
    audit.validate().map_err(|e| anyhow::anyhow!(e))?;
    Ok(audit)
}

/// What a run recorded.
#[derive(Debug, Clone, PartialEq)]
pub struct Audited {
    pub audit: Audit,
    /// The auditor's recorded model invocation.
    pub invocation: Invocation,
}

impl Audited {
    /// Where the invocation record lives.
    pub fn invocation_path(&self) -> String {
        AnyRecord::Invocation(Box::new(self.invocation.clone())).canonical_path()
    }
}

/// Removes a run's directory in the state directory when the run ends, however it ends.
struct Cleanup(PathBuf);

impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A repository path that is safe to join under the workspace: relative, with no `..`.
fn is_plain_relative(path: &str) -> bool {
    !path.is_empty()
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

/// Reads the job's changed files (at most `MAX_FILES`) from the commit that holds them.
///
/// That is `candidate` when it exists in this repository. A fresh clone does not have candidate commits, so
/// otherwise it is the newest commit on `head`'s history that carries the job's `Athena-Job` trailer and holds
/// at least one of the files (the squash commit that landed the job). A file the job deleted reads as nothing.
/// When the job changed files and none can be read, this fails naming the job and the commits tried.
fn read_changed_files(
    athena: &Athena,
    head: &str,
    job: &JobId,
    candidate: Option<&str>,
    changed: &[String],
) -> Result<Vec<(String, Vec<u8>)>> {
    let wanted: Vec<&String> = changed.iter().take(MAX_FILES).collect();
    if wanted.is_empty() {
        return Ok(Vec::new());
    }
    let read_from = |commit: &str| -> Result<Vec<(String, Vec<u8>)>> {
        let mut files = Vec::new();
        for path in &wanted {
            if let Some(bytes) = athena.git.show_file(commit, path)? {
                files.push(((*path).clone(), bytes));
            }
        }
        Ok(files)
    };
    let mut tried: Vec<String> = Vec::new();
    match candidate {
        Some(c) if athena.git.commit_exists(c)? => {
            let files = read_from(c)?;
            if files.is_empty() {
                bail!("{}", refusal(job, &[format!("{c} (candidate)")]));
            }
            return Ok(files);
        }
        Some(c) => tried.push(format!("{c} (candidate, not in this repository)")),
        None => tried.push("no candidate commit recorded".to_string()),
    }
    for squash in athena
        .git
        .log_grep(head, &format!("^Athena-Job: {}$", job.as_str()))?
    {
        let files = read_from(&squash)?;
        if !files.is_empty() {
            return Ok(files);
        }
        tried.push(format!("{squash} (Athena-Job trailer)"));
    }
    bail!("{}", refusal(job, &tried))
}

/// The refusal for a job whose changed files cannot be read.
fn refusal(job: &JobId, tried: &[String]) -> String {
    format!(
        "cannot audit job {job}: it changed files but none could be read; commits tried: {}. \
         Nothing was paid for and no audit was recorded",
        tried.join(", ")
    )
}

/// One audit run over the job `reference`: reads `BRIEF.md` and the job's changed files at the head commit's
/// records, asks the model, and commits the audit, the invocation record and the role definition (when it is
/// not yet recorded) in one records commit. A nested run makes no model call and records nothing.
pub fn run(
    athena: &Athena,
    role: &RoleDefinition,
    reference: &str,
    now: Timestamp,
) -> Result<Option<Audited>> {
    if athena.nested.is_some() {
        return Ok(None);
    }
    athena.refuse_if_stopped()?;
    let head = athena.head()?;
    let records = Records::load(&athena.git, &head)?;
    let target = records.resolve(reference)?;
    let job = records
        .jobs
        .get(target.as_str())
        .with_context(|| format!("no records for job {target}"))?;
    let Some(outcome) = job.later_outcome.as_ref().or(job.outcome.as_ref()) else {
        bail!("job {target} has no outcome yet; there is nothing to audit");
    };
    let brief_md = athena
        .git
        .show_file(&head, "BRIEF.md")?
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .context("BRIEF.md is not in the repository at HEAD")?;

    // The commit the job's files are read from: the candidate's, else the outcome's. A job with neither
    // changed nothing the auditor can read.
    let commit = job
        .candidate
        .as_ref()
        .and_then(|c| c.candidate_commit.clone())
        .or_else(|| outcome.candidate_commit.clone());
    let changed: Vec<String> = job
        .candidate
        .as_ref()
        .map(|c| {
            c.changes
                .iter()
                .map(|e| e.path.clone())
                .filter(|p| is_plain_relative(p) && !p.starts_with("records/"))
                .collect()
        })
        .unwrap_or_default();

    // Read the job's files before anything is created or paid for, so a refusal costs nothing.
    let files = read_changed_files(athena, &head, &target, commit.as_deref(), &changed)?;

    let jc = JobConfig::at(&athena.git, &head)?;
    let run_job = JobId::Id(RecordId::generate(now));
    let job_dir = athena.state.job_dir(&run_job);
    let _cleanup = Cleanup(job_dir.clone());
    let workspace = job_dir.join("workspace");
    std::fs::create_dir_all(workspace.join(OUTCOME_DIR))?;
    std::fs::write(
        workspace.join(OUTCOME_FILE),
        outcome_text(job, &changed[..changed.len().min(MAX_FILES)]),
    )?;
    for (path, bytes) in &files {
        let text: String = String::from_utf8_lossy(bytes)
            .chars()
            .take(MAX_FILE_CHARS)
            .collect();
        let to = workspace.join(OUTCOME_DIR).join(path);
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(to, text)?;
    }

    let composed: Composed = auditor_system_prompt(&role.prompt, role.version);
    let grant = ToolGrant {
        tools: vec!["Read".into(), "Grep".into(), "Glob".into()],
        commands: BTreeMap::new(),
        writes: false,
        permission_mode: "dontAsk".into(),
    };
    // The auditor borrows the verifier's model and budget, as the steward and the audience do.
    let spec = athena.agent_spec(
        &jc,
        &run_job,
        1,
        Role::Auditor,
        &jc.config.models.verifier,
        &composed,
        brief(&brief_md, &target),
        &grant,
        &workspace,
        Side::Read,
        &BTreeMap::new(),
        jc.config.budget.verifier_usd,
        jc.config.budget.verifier_timeout_s,
        auditor_schema(),
    )?;
    let facts = crate::invocation::InvocationFacts {
        job: &run_job,
        spec: &spec,
        composed: &composed,
        config: &jc.config,
        fake: athena.is_fake(),
    };
    let redactor = athena.redactor(&jc.config, Some(&run_job))?;
    let (output, invocation) = match athena.invoke_and_record(&facts, &redactor)? {
        Ok(done) => done,
        Err(error) => bail!("the auditor could not run: {error}"),
    };

    // The invocation cost money whatever the model said, so it is recorded even when the report is unusable.
    let report = if output.outcome == InvocationOutcome::Success {
        serde_json::from_value::<AuditorOutput>(output.structured.clone().unwrap_or_default())
            .context("the auditor returned no valid report")
    } else {
        Err(anyhow::anyhow!(
            "the auditor did not complete: {}",
            invocation.detail.as_deref().unwrap_or("no detail")
        ))
    };
    let mut id = RecordId::generate(crate::now());
    while records.has_record_id(&id) {
        id = RecordId::generate(crate::now());
    }
    let audit = report
        .and_then(|r| validate_report(r, &target, id, now))
        .and_then(|mut audit| {
            // What the model wrote goes into records, so it is redacted first.
            let mut words: Vec<String> = vec![audit.summary.clone()];
            words.extend(
                audit
                    .findings
                    .iter()
                    .flat_map(|f| [f.text.clone(), f.evidence.clone()]),
            );
            redactor.redact_all(&mut words)?;
            let mut words = words.into_iter();
            audit.summary = words.next().unwrap_or_default();
            for f in audit.findings.iter_mut() {
                f.text = words.next().unwrap_or_default();
                f.evidence = words.next().unwrap_or_default();
            }
            audit.validate().map_err(|e| anyhow::anyhow!(e))?;
            Ok(audit)
        });

    let mut files: Vec<NewFile> = Vec::new();
    let mut trailers: Vec<(&str, String)> = Vec::new();
    let recorded = records
        .roles
        .iter()
        .any(|r| r.name == role.name && r.version == role.version);
    if !recorded {
        let record = AnyRecord::Role(Box::new(role.clone()));
        trailers.push(("Athena-Record", record.canonical_path()));
        files.push(NewFile::record(&record));
    }
    if let Ok(audit) = &audit {
        let record = AnyRecord::Audit(Box::new(audit.clone()));
        trailers.push(("Athena-Record", record.canonical_path()));
        files.push(NewFile::record(&record));
    }
    let invocation_record = AnyRecord::Invocation(Box::new(invocation.clone()));
    trailers.push(("Athena-Record", invocation_record.canonical_path()));
    files.push(NewFile::record(&invocation_record));
    let subject = match &audit {
        Ok(a) => format!(
            "Audit: job {} with {} finding(s)",
            a.target,
            a.findings.len()
        ),
        Err(_) => "Audit: no usable report".to_string(),
    };
    let committer = Committer {
        git: &athena.git,
        state: &athena.state,
    };
    committer.commit_records(
        &files,
        &message(
            &subject,
            "A judgement of one job's real outcome against BRIEF.md and the owner's goals. Each finding cites its evidence.",
            &trailers,
        ),
    )?;
    audit.map(|audit| Some(Audited { audit, invocation }))
}

impl Athena {
    /// `athena audit`: one auditor run over the job `reference` (`latest`, a full id, or a unique suffix).
    pub fn audit(&self, reference: &str) -> Result<Option<Audited>> {
        run(self, &roles::auditor(crate::now()), reference, crate::now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_context::outputs::AuditorFindingOutput;

    fn at() -> Timestamp {
        "2026-10-17T12:00:00Z".parse().unwrap()
    }

    fn id() -> RecordId {
        RecordId::parse("20261017-120000-1abc").unwrap()
    }

    fn job() -> JobId {
        JobId::parse("20261016-120000-2def").unwrap()
    }

    fn report(evidence: &str) -> AuditorOutput {
        AuditorOutput {
            summary: "The outcome serves the goal.".into(),
            findings: vec![AuditorFindingOutput {
                text: "The command exists.".into(),
                evidence: evidence.into(),
            }],
        }
    }

    #[test]
    fn a_report_becomes_an_audit_only_when_every_finding_cites_evidence() {
        let ok = validate_report(report("main.rs:12"), &job(), id(), at()).unwrap();
        assert_eq!(ok.target, job());
        assert_eq!(ok.findings[0].evidence, "main.rs:12");
        assert!(validate_report(report("  "), &job(), id(), at()).is_err());
        let mut no_summary = report("x");
        no_summary.summary = " ".into();
        assert!(validate_report(no_summary, &job(), id(), at()).is_err());
    }

    #[test]
    fn the_brief_carries_the_brief_and_names_the_job() {
        let text = brief("# Brief\nOWNER-GOAL-CANARY\n", &job());
        assert!(text.contains("OWNER-GOAL-CANARY"));
        assert!(text.contains(job().as_str()));
        assert!(!text.contains("Acceptance criteria"));
    }

    #[test]
    fn only_plain_relative_paths_are_written_into_the_workspace() {
        assert!(is_plain_relative("src/lib.rs"));
        for bad in ["", "/etc/passwd", "../x", "a/../../x"] {
            assert!(!is_plain_relative(bad), "{bad}");
        }
    }
}
