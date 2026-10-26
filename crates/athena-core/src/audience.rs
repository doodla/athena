//! The audience: a role that reads a shipped outcome as its audience would, rates it from 1 to 5, and records
//! the rating with findings that each cite their evidence (`docs/design/audience-role.md`).
//!
//! The audience only reads and records. Athena fetches the page, writes it into a workspace of one file, and
//! starts the model through the same driver and tool policy as every other role, with `Read`, `Grep` and
//! `Glob` and no commands. The model's report is validated before it becomes an `athena.audience-rating/1`
//! record. A rating at or below [`LOW_RATING`] also queues a work item, a proposal a person decides with
//! `athena approve-work` or `athena reject-work`. The latest rating is shown to the steward in its brief and
//! in `athena status`.

use crate::Athena;
use crate::capability::{JobConfig, ToolGrant};
use crate::commits::{Committer, NewFile, message};
use crate::store::Records;
use anyhow::{Context, Result, bail};
use athena_agent::env::Side;
use athena_context::outputs::{AudienceOutput, audience_schema};
use athena_context::{Composed, audience_system_prompt};
use athena_data::records::{
    AnyRecord, AudienceFinding, AudienceRating, InvocationOutcome, RATING_RANGE, Reference, Role,
    WorkItem, WorkItemKind,
};
use athena_data::roles::{self, Ladder, RoleDefinition};
use athena_data::{JobId, RecordId};
use jiff::Timestamp;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// The live explainer page (`docs/design/explainer-site.md`). This is the only place the default lives. To
/// rate another page, give `athena audience --url`, or add a [`Target`] constructor beside [`Target::explainer`].
pub const EXPLAINER_URL: &str = "https://doodla.github.io/athena/";

/// The reader the explainer is written for.
pub const NEWCOMER: &str = "newcomer";

/// A rating at or below this raises a work item.
pub const LOW_RATING: u8 = 2;

/// The file in the audience's workspace that holds the page as fetched.
pub const PAGE_FILE: &str = "PAGE.html";

/// The most of a page the audience is given, in characters.
pub const MAX_PAGE_CHARS: usize = 400_000;

/// The most findings, and the longest finding text or source, that a rating keeps.
pub const MAX_FINDINGS: usize = 20;
const MAX_TEXT: usize = 1_000;

/// What is read, and by whom.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub url: String,
    pub audience: String,
}

impl Target {
    /// The live explainer page, read as a newcomer.
    pub fn explainer() -> Target {
        Target {
            url: EXPLAINER_URL.into(),
            audience: NEWCOMER.into(),
        }
    }
}

/// Where the page comes from. Tests give a stub, so no test reaches the network.
pub trait Page: Sync {
    fn fetch(&self, url: &str) -> Result<String>;
}

/// The real page source: a GET through the one wrapper for external calls (`crate::gh`), which follows
/// redirects and fails on an HTTP error.
#[derive(Debug)]
pub struct LivePage;

impl Page for LivePage {
    fn fetch(&self, url: &str) -> Result<String> {
        crate::gh::CurlHttp::default().get_text(url)
    }
}

/// The latest rating in the records, if any.
pub fn latest(records: &Records) -> Option<&AudienceRating> {
    records.audience_ratings.last()
}

/// Whether a rating raises a work item.
pub fn is_low(rating: u8) -> bool {
    rating <= LOW_RATING
}

/// The key of the work item for one rating, so a rating raises it once.
pub fn work_item_key(rating: &AudienceRating) -> String {
    format!("audience:{}", rating.id)
}

/// The work item a low rating raises: a proposal on the implementation rung that cites the rating record.
pub fn work_item(rating: &AudienceRating, id: RecordId, now: Timestamp) -> WorkItem {
    let record =
        Reference::path(AnyRecord::AudienceRating(Box::new(rating.clone())).canonical_path());
    WorkItem {
        id,
        at: now,
        role: roles::AUDIENCE.into(),
        kind: WorkItemKind::Proposal,
        key: work_item_key(rating),
        title: format!(
            "A {} rated {} {}/5; address its findings",
            rating.audience, rating.target, rating.rating
        ),
        rung: Ladder::Implementation,
        source: record.clone(),
        evidence: vec![record],
    }
}

/// The steward's brief section, and the one `athena status` echoes: the latest rating and its findings.
pub fn brief_section(latest: Option<&AudienceRating>) -> String {
    let mut text = String::from("## Audience rating\n\n");
    let Some(r) = latest else {
        text.push_str("No audience rating is recorded yet (`athena audience` records one).\n");
        return text;
    };
    text.push_str(&format!(
        "The latest rating: {}/5 from a {} reading {} at {}.\n",
        r.rating, r.audience, r.target, r.at
    ));
    for finding in &r.findings {
        text.push_str(&format!(
            "- {} (evidence: {})\n",
            finding.text, finding.source
        ));
    }
    if is_low(r.rating) {
        text.push_str(&format!(
            "A rating of {LOW_RATING} or lower has already raised the work item `{}`; do not report it again.\n",
            work_item_key(r)
        ));
    }
    text
}

/// The audience's user prompt.
pub fn brief(target: &Target) -> String {
    format!(
        "# Audience run\n\nYou are a {audience}: someone meeting {url} for the first time. The page as \
         fetched is the file `{PAGE_FILE}` in the current directory; read it with Read and Grep. Nothing \
         else is available and nothing may be changed.\n\n\
         ## Your report\n\nReturn `rating`, a whole number from {lo} to {hi}, and `findings`. Each finding \
         has `text` (one or two sentences) and `source` (a passage quoted from the page, or the heading \
         where you saw it). A finding without a source is dropped. Rate how well the page serves a \
         {audience}: could they say what it is, why it matters, and what to do next?\n",
        audience = target.audience,
        url = target.url,
        lo = RATING_RANGE.start(),
        hi = RATING_RANGE.end(),
    )
}

/// Validates the model's report into a rating record. A rating out of range is refused; a finding without
/// evidence is refused too, so the record never holds a claim with nothing behind it.
pub fn validate_report(
    report: AudienceOutput,
    target: &Target,
    id: RecordId,
    now: Timestamp,
) -> Result<AudienceRating> {
    if report.findings.len() > MAX_FINDINGS {
        bail!("a rating keeps at most {MAX_FINDINGS} findings");
    }
    if report
        .findings
        .iter()
        .any(|f| f.text.chars().count() > MAX_TEXT || f.source.chars().count() > MAX_TEXT)
    {
        bail!("a finding's text and source are at most {MAX_TEXT} characters");
    }
    let rating = AudienceRating {
        id,
        at: now,
        target: target.url.clone(),
        audience: target.audience.clone(),
        rating: report.rating,
        findings: report
            .findings
            .into_iter()
            .map(|f| AudienceFinding {
                text: f.text.trim().to_string(),
                source: f.source.trim().to_string(),
            })
            .collect(),
    };
    rating.validate().map_err(|e| anyhow::anyhow!(e))?;
    Ok(rating)
}

/// What a run recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rated {
    pub rating: AudienceRating,
    /// The work item a low rating raised.
    pub work_item: Option<WorkItem>,
}

/// Removes a run's directory in the state directory when the run ends, however it ends.
struct Cleanup(PathBuf);

impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// One audience run: fetches the page, asks the model for a rating, and commits the rating, the invocation
/// record, the role definition (when it is not yet recorded) and, for a low rating, a work item, in one
/// records commit. A nested run makes no model call and records nothing.
pub fn run(
    athena: &Athena,
    page: &dyn Page,
    role: &RoleDefinition,
    target: &Target,
    now: Timestamp,
) -> Result<Option<Rated>> {
    if athena.nested.is_some() {
        return Ok(None);
    }
    athena.refuse_if_stopped()?;
    let head = athena.head()?;
    let records = Records::load_current(&athena.git, &head)?;
    let text = page.fetch(&target.url)?;
    let text: String = text.chars().take(MAX_PAGE_CHARS).collect();
    if text.trim().is_empty() {
        bail!("{} returned an empty page", target.url);
    }

    let jc = JobConfig::at(&athena.git, &head)?;
    let job = JobId::Id(RecordId::generate(now));
    let job_dir = athena.state.job_dir(&job);
    let _cleanup = Cleanup(job_dir.clone());
    let workspace = job_dir.join("workspace");
    std::fs::create_dir_all(&workspace)?;
    std::fs::write(workspace.join(PAGE_FILE), &text)?;

    let composed: Composed = audience_system_prompt(&role.prompt, role.version);
    let grant = ToolGrant {
        tools: vec!["Read".into(), "Grep".into(), "Glob".into()],
        commands: BTreeMap::new(),
        writes: false,
        permission_mode: "dontAsk".into(),
    };
    // The audience borrows the verifier's model and budget, as the steward does.
    let spec = athena.agent_spec(
        &jc,
        &job,
        1,
        Role::Audience,
        &jc.config.models.verifier,
        &composed,
        brief(target),
        &grant,
        &workspace,
        Side::Read,
        &BTreeMap::new(),
        jc.config.budget.verifier_usd,
        jc.config.budget.verifier_timeout_s,
        audience_schema(),
    )?;
    let facts = crate::invocation::InvocationFacts {
        job: &job,
        spec: &spec,
        composed: &composed,
        config: &jc.config,
        fake: athena.is_fake(),
    };
    let redactor = athena.redactor(&jc.config, Some(&job))?;
    let (output, invocation) = match athena.invoke_and_record(&facts, &redactor)? {
        Ok(done) => done,
        Err(error) => bail!("the audience could not run: {error}"),
    };

    let mut files: Vec<NewFile> = Vec::new();
    let mut trailers: Vec<(&str, String)> = Vec::new();
    // The invocation cost money whatever the model said, so it is recorded even when the report is unusable.
    let report = if output.outcome == InvocationOutcome::Success {
        serde_json::from_value::<AudienceOutput>(output.structured.clone().unwrap_or_default())
            .context("the audience returned no valid report")
    } else {
        Err(anyhow::anyhow!(
            "the audience did not complete: {}",
            invocation.detail.as_deref().unwrap_or("no detail")
        ))
    };
    let invocation = AnyRecord::Invocation(Box::new(invocation));

    let mut taken: Vec<RecordId> = Vec::new();
    let mut fresh = |records: &Records| loop {
        let id = RecordId::generate(crate::now());
        if !records.has_record_id(&id) && !taken.contains(&id) {
            taken.push(id.clone());
            return id;
        }
    };
    let rated = match report.and_then(|r| validate_report(r, target, fresh(&records), now)) {
        Ok(mut rating) => {
            // What the model wrote goes into records, so it is redacted first.
            let mut words: Vec<String> = rating
                .findings
                .iter()
                .flat_map(|f| [f.text.clone(), f.source.clone()])
                .collect();
            redactor.redact_all(&mut words)?;
            let mut words = words.into_iter();
            for f in rating.findings.iter_mut() {
                f.text = words.next().unwrap_or_default();
                f.source = words.next().unwrap_or_default();
            }
            rating.validate().map_err(|e| anyhow::anyhow!(e))?;
            let item = is_low(rating.rating).then(|| work_item(&rating, fresh(&records), now));
            Ok(Rated {
                rating,
                work_item: item,
            })
        }
        Err(error) => Err(error),
    };

    let recorded = records
        .roles
        .iter()
        .any(|r| r.name == role.name && r.version == role.version);
    if !recorded {
        let record = AnyRecord::Role(Box::new(role.clone()));
        trailers.push(("Athena-Record", record.canonical_path()));
        files.push(NewFile::record(&record));
    }
    if let Ok(rated) = &rated {
        let record = AnyRecord::AudienceRating(Box::new(rated.rating.clone()));
        trailers.push(("Athena-Record", record.canonical_path()));
        files.push(NewFile::record(&record));
        if let Some(item) = &rated.work_item {
            item.validate()
                .map_err(|reason| anyhow::anyhow!("work item {}: {reason}", item.key))?;
            let record = AnyRecord::WorkItem(Box::new(item.clone()));
            trailers.push(("Athena-Record", record.canonical_path()));
            files.push(NewFile::record(&record));
        }
    }
    trailers.push(("Athena-Record", invocation.canonical_path()));
    files.push(NewFile::record(&invocation));
    let subject = match &rated {
        Ok(r) => format!(
            "Audience: {} rated {}/5",
            r.rating.audience, r.rating.rating
        ),
        Err(_) => "Audience: no usable rating".to_string(),
    };
    let committer = Committer {
        git: &athena.git,
        state: &athena.state,
    };
    committer.commit_records(
        &files,
        &message(
            &subject,
            "A rating of the live page by a reader of one kind. Each finding cites its evidence.",
            &trailers,
        ),
    )?;
    rated.map(Some)
}

impl Athena {
    /// `athena audience`: one audience run over `target`.
    pub fn audience(&self, page: &dyn Page, target: &Target) -> Result<Option<Rated>> {
        run(
            self,
            page,
            &roles::audience(crate::now()),
            target,
            crate::now(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_context::outputs::AudienceFindingOutput;

    fn at() -> Timestamp {
        "2026-10-15T12:00:00Z".parse().unwrap()
    }

    fn id() -> RecordId {
        RecordId::parse("20261015-120000-1abc").unwrap()
    }

    fn report(rating: u8, source: &str) -> AudienceOutput {
        AudienceOutput {
            rating,
            findings: vec![AudienceFindingOutput {
                text: "The page never says what to do next.".into(),
                source: source.into(),
            }],
        }
    }

    #[test]
    fn the_explainer_is_the_default_target() {
        let target = Target::explainer();
        assert_eq!(target.url, EXPLAINER_URL);
        assert_eq!(target.audience, "newcomer");
    }

    #[test]
    fn a_report_becomes_a_rating_only_when_it_is_in_range_and_cites_evidence() {
        let target = Target::explainer();
        let ok = validate_report(report(4, "heading: Start here"), &target, id(), at()).unwrap();
        assert_eq!(ok.rating, 4);
        assert_eq!(ok.target, EXPLAINER_URL);
        for bad in [0, 6] {
            assert!(validate_report(report(bad, "x"), &target, id(), at()).is_err());
        }
        assert!(validate_report(report(3, "  "), &target, id(), at()).is_err());
    }

    #[test]
    fn only_a_rating_of_two_or_lower_is_low() {
        assert!(is_low(1) && is_low(2));
        assert!(!is_low(3) && !is_low(5));
    }

    #[test]
    fn the_work_item_cites_the_rating_record_and_validates() {
        let rating = validate_report(
            report(2, "heading: Start"),
            &Target::explainer(),
            id(),
            at(),
        )
        .unwrap();
        let item = work_item(
            &rating,
            RecordId::parse("20261015-120001-2def").unwrap(),
            at(),
        );
        assert_eq!(item.validate(), Ok(()));
        assert_eq!(item.role, "audience");
        assert_eq!(item.key, "audience:20261015-120000-1abc");
        assert_eq!(
            item.evidence,
            vec![Reference::path(
                "records/audience-ratings/2026-10/20261015-120000-1abc.json"
            )]
        );
    }

    #[test]
    fn the_brief_section_shows_the_rating_and_findings_or_says_there_is_none() {
        assert!(brief_section(None).contains("No audience rating"));
        let rating = validate_report(
            report(2, "heading: Start"),
            &Target::explainer(),
            id(),
            at(),
        )
        .unwrap();
        let text = brief_section(Some(&rating));
        assert!(text.contains("2/5"));
        assert!(text.contains("never says what to do next"));
        assert!(text.contains("evidence: heading: Start"));
        assert!(text.contains("audience:20261015-120000-1abc"));
        let fine = AudienceRating {
            rating: 4,
            ..rating
        };
        assert!(!brief_section(Some(&fine)).contains("already raised"));
    }

    #[test]
    fn the_real_page_source_refuses_a_non_http_url() {
        assert!(LivePage.fetch("file:///etc/hostname").is_err());
    }
}
