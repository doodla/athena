//! The records `athena land --pr` writes to the `athena/records` branch around a merge: the landing-intent and
//! landing records, the records-history cutover, and the reconcile hooks over them.

use super::MergeMethod;
use crate::land::pr_commits;
use crate::reconcile::{ReconcileRecords, RecordsSnapshot};
use crate::records_import::{MANIFEST_PATH, import_at_cutover};
use anyhow::{Context, Result, bail};
use athena_data::history::{self, HistoryRecord, Landing, LandingCommit, LandingIntent};
use athena_data::{JobId, RecordId};
use athena_git::Git;
use athena_git::records_branch::{Change, RecordsWriter};

/// The records `land_pr` writes around the merge (§5.2, §5.3, §8 of `docs/design/records-history.md`).
pub trait LandingRecords {
    /// The commit the remote's default branch `branch` holds now: the intent's expected base, and the lower
    /// bound of the pull request's commits.
    fn base_tip(&self, branch: &str) -> Result<String>;
    /// Writes the landing-intent record for the pull request, **before** the merge, and returns its id. The
    /// commits in `expected_base..head` are checked first; one without a verification is an error and
    /// nothing is written. The intent of an earlier run for the same pull request and head is reused.
    fn intent(&self, pr_number: u64, head: &str, expected_base: &str) -> Result<RecordId>;
    /// Writes the landing record, **after** the merge was confirmed, and returns its id. Its records commit
    /// takes `head` as a second parent (§6.1).
    fn landing(&self, landing: &LandingFacts) -> Result<RecordId>;
    /// Runs after the landing record was written, with the host commit the merge made. The records-history
    /// cutover hooks in here (`docs/design/records-history.md` §4 and §10, D111): when the landed commit's
    /// `athena.toml` names a `[history] cutover_job`, the import at that job's `C0` and the legacy-exempt
    /// record are written if they are not on the records branch yet. Returns a line to show when something was
    /// written. The default does nothing.
    fn cutover(&self, _host_commit: &str) -> Result<Option<String>> {
        Ok(None)
    }
}

/// What is known about a merge once the host reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LandingFacts {
    pub intent: RecordId,
    pub expected_base: String,
    pub pr_number: u64,
    pub head: String,
    pub host_commit: String,
    pub merge_method: MergeMethod,
}

/// The verification record that covers a commit of `job`. A verification has no id of its own: there is
/// exactly one per job, stored under the job's directory, so the job's id names it (§5.3). The bootstrap job
/// has no such id, so a commit of it cannot be recorded and the error names the commit. If a verification
/// ever gets an id of its own, this is the one place to change.
pub fn verification_id(job: &JobId, commit: &str) -> Result<RecordId> {
    match job {
        JobId::Id(id) => Ok(id.clone()),
        JobId::Bootstrap => bail!(
            "commit {commit} belongs to the bootstrap job, which has no verification record to name in a \
             landing record"
        ),
    }
}

/// Directory of landing-intent files on the records branch.
const INTENT_DIR: &str = "landing-intents";
/// Directory of landing files on the records branch.
const LANDING_DIR: &str = "landings";

fn history_method(method: MergeMethod) -> history::MergeMethod {
    match method {
        MergeMethod::Merge => history::MergeMethod::Merge,
        MergeMethod::Squash => history::MergeMethod::Squash,
        MergeMethod::Rebase => history::MergeMethod::Rebase,
    }
}

/// [`LandingRecords`] over a git repository and one of its remotes: records go to the remote's
/// `athena/records` branch through [`RecordsWriter::append_to_remote`] (§8.2), which builds on the remote
/// tip and moves the local branch only after the push succeeded.
#[derive(Debug)]
pub struct GitLandingRecords<'a> {
    git: &'a Git,
    remote: String,
}

impl<'a> GitLandingRecords<'a> {
    /// The real sink for `git` and the remote named `remote`. The remote must exist, so a landing cannot go
    /// ahead with nowhere to put its records (§5.2).
    pub fn for_remote(git: &'a Git, remote: &str) -> Result<Self> {
        if remote.trim().is_empty() {
            bail!("no remote is named, so landing records have nowhere to go");
        }
        git.run(["remote", "get-url", remote]).with_context(|| {
            format!("remote {remote:?} is not configured, so landing records have nowhere to go")
        })?;
        Ok(GitLandingRecords {
            git,
            remote: remote.to_string(),
        })
    }

    /// The history records under `dir` on the remote's records branch (the local branch is first reset to
    /// it, see [`RecordsWriter::sync_from`]).
    fn existing(&self, dir: &str) -> Result<Vec<HistoryRecord>> {
        let Some(tip) = RecordsWriter::new(self.git).sync_from(&self.remote)? else {
            return Ok(Vec::new());
        };
        self.records_under(&tip, dir)
    }

    /// The history records under `dir` at the records commit `tip`.
    fn records_under(&self, tip: &str, dir: &str) -> Result<Vec<HistoryRecord>> {
        let oids: Vec<String> = self
            .git
            .ls_tree_under(tip, dir)?
            .into_iter()
            .map(|e| e.oid)
            .collect();
        self.git
            .cat_blobs(&oids)?
            .iter()
            .map(|bytes| {
                HistoryRecord::parse(&String::from_utf8_lossy(bytes))
                    .context("an existing record on the records branch is not valid")
            })
            .collect()
    }

    /// The pull request's commits, each with its kind and, for a job landing, the job and the verification
    /// record that covers it, all read from the commit's content (§5.3).
    pub(super) fn commits(&self, expected_base: &str, head: &str) -> Result<Vec<LandingCommit>> {
        let commits: Vec<LandingCommit> = pr_commits(self.git, expected_base, head)?
            .into_iter()
            .map(|c| LandingCommit {
                sha: c.sha,
                tree: c.tree,
                kind: c.kind,
                job: c.job,
                verification: c.verification,
            })
            .collect();
        for commit in &commits {
            commit.validate()?;
        }
        Ok(commits)
    }

    /// Validates `record`, then writes it as `<dir>/<id>.json` in one commit with `extra_parents`.
    fn write(
        &self,
        dir: &str,
        id: &RecordId,
        record: &HistoryRecord,
        message: &str,
        extra_parents: &[&str],
    ) -> Result<()> {
        record.validate()?;
        let bytes = serde_json::to_vec_pretty(record)?;
        RecordsWriter::new(self.git).append_to_remote(
            &self.remote,
            message,
            &[Change::put(format!("{dir}/{id}.json"), bytes)],
            extra_parents,
        )?;
        Ok(())
    }
}

impl LandingRecords for GitLandingRecords<'_> {
    fn base_tip(&self, branch: &str) -> Result<String> {
        self.git
            .fetch_remote_branch(&self.remote, branch)?
            .with_context(|| {
                format!(
                    "the remote {} has no branch {branch}, so the expected base cannot be recorded",
                    self.remote
                )
            })
    }

    fn intent(&self, pr_number: u64, head: &str, expected_base: &str) -> Result<RecordId> {
        let commits = self.commits(expected_base, head)?;
        for record in self.existing(INTENT_DIR)? {
            if let HistoryRecord::LandingIntent(found) = record
                && found.pr_number == pr_number
                && found.head_sha == head
            {
                return Ok(found.id);
            }
        }
        let mut job_ids: Vec<JobId> = Vec::new();
        for job in commits.iter().filter_map(|c| c.job.as_ref()) {
            if !job_ids.contains(job) {
                job_ids.push(job.clone());
            }
        }
        if job_ids.is_empty() {
            bail!(
                "no commit between the remote base {expected_base} and the PR head {head} lands a job, so \
                 a landing intent has no job to name; nothing was recorded"
            );
        }
        let now = crate::now();
        let intent = LandingIntent {
            id: RecordId::generate(now),
            created_at: now,
            pr_number,
            head_sha: head.to_string(),
            head_tree_sha: self.git.tree_of(head)?,
            expected_base: expected_base.to_string(),
            job_ids,
        };
        let id = intent.id.clone();
        self.write(
            INTENT_DIR,
            &id,
            &HistoryRecord::LandingIntent(intent),
            &format!("Landing intent for pull request #{pr_number}"),
            &[],
        )?;
        Ok(id)
    }

    fn landing(&self, facts: &LandingFacts) -> Result<RecordId> {
        let commits = self.commits(&facts.expected_base, &facts.head)?;
        for record in self.existing(LANDING_DIR)? {
            if let HistoryRecord::Landing(found) = record
                && found.intent == facts.intent
            {
                return Ok(found.id);
            }
        }
        let now = crate::now();
        let landing = Landing {
            id: RecordId::generate(now),
            created_at: now,
            pr_number: facts.pr_number,
            head_sha: facts.head.clone(),
            host_commit: facts.host_commit.clone(),
            merge_method: history_method(facts.merge_method),
            commits,
            intent: facts.intent.clone(),
        };
        let id = landing.id.clone();
        self.write(
            LANDING_DIR,
            &id,
            &HistoryRecord::Landing(landing),
            &format!("Landing record for pull request #{}", facts.pr_number),
            &[facts.head.as_str()],
        )?;
        Ok(id)
    }

    fn cutover(&self, host_commit: &str) -> Result<Option<String>> {
        // The landed commit's own `athena.toml` names the cutover job: the file that adds the setting is part
        // of the merge that lands it. A commit that cannot be fetched cannot be read; the reconcile catch-up
        // then finishes the work from the default branch's tip.
        if !self
            .git
            .ensure_commit(&self.remote, host_commit)
            .unwrap_or(false)
        {
            return Ok(None);
        }
        let Some(job) = cutover_job_for(self.git, host_commit) else {
            return Ok(None);
        };
        Ok(self
            .run_cutover(&job)?
            .filter(CutoverRun::wrote)
            .map(|run| run.describe()))
    }
}

/// What one run of the cutover did (`docs/design/records-history.md` §4 and §10, D111).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutoverRun {
    /// `C0`: the host commit of the cutover job's landing record.
    pub c0: String,
    /// The records commit that imported `records/` at `c0` with the manifest, when this run wrote it.
    pub import: Option<String>,
    /// Whether this run wrote the legacy-exempt record.
    pub exempt_written: bool,
}

impl CutoverRun {
    /// Whether this run changed the records branch.
    pub fn wrote(&self) -> bool {
        self.import.is_some() || self.exempt_written
    }

    /// A line for a person.
    pub fn describe(&self) -> String {
        let mut did = Vec::new();
        if let Some(commit) = &self.import {
            did.push(format!(
                "imported records/ at C0 ({commit} holds the files and the manifest)"
            ));
        }
        if self.exempt_written {
            did.push("wrote the legacy-exempt record".to_string());
        }
        format!("cutover: {} for C0 {}", did.join(" and "), self.c0)
    }
}

/// Reads `[history] cutover_job` from the `athena.toml` at `commit`. `None` when the commit has no
/// `athena.toml`, it does not parse, or it sets no cutover job. The built-in configuration is not consulted:
/// the setting is only ever what a commit says.
///
/// The cutover job is read from the landed commit (after a merge) or the default branch's tip (in the
/// reconcile catch-up), never from the checkout a landing started in: the job that adds the setting lands
/// with a checkout that does not have it yet.
pub fn cutover_job_for(git: &Git, commit: &str) -> Option<String> {
    let bytes = git.show_file(commit, "athena.toml").ok()??;
    athena_data::config::Config::parse(&String::from_utf8_lossy(&bytes))
        .ok()?
        .history
        .cutover_job
}

impl GitLandingRecords<'_> {
    /// Runs the cutover for `job`, the configured `[history] cutover_job`, against the remote's records
    /// branch (the local branch is first reset to it). `C0` is the host commit of the landing record that
    /// lands `job`; `None` when there is none yet. Each of the two records is written only when it is not on
    /// the branch, each tested by its file: `import-manifest.json` for the import of `records/` at `C0`, and
    /// `legacy-exempt.json` for the legacy-exempt record, built from `[history] exempt` of `athena.toml` at
    /// `C0`. When both are missing they go in one commit, so the branch never shows an import without its
    /// exempt record. A second run writes nothing.
    pub fn run_cutover(&self, job: &str) -> Result<Option<CutoverRun>> {
        let Some(tip) = RecordsWriter::new(self.git).sync_from(&self.remote)? else {
            return Ok(None);
        };
        let Some(c0) = athena_gate::attribution::cutover_commit_at(self.git, &tip, job)? else {
            return Ok(None);
        };
        if !self.git.ensure_commit(&self.remote, &c0)? {
            bail!(
                "C0 {c0}, the host commit of the landing of job {job}, is not available from remote {}",
                self.remote
            );
        }
        let has = |path: &str| -> Result<bool> { Ok(self.git.show_file(&tip, path)?.is_some()) };
        let has_manifest = has(MANIFEST_PATH)?;
        let exempt = if has(athena_gate::legacy::LEGACY_EXEMPT_PATH)? {
            None
        } else {
            let now = crate::now();
            let config = athena_gate::config_at(self.git, Some(&c0))?;
            let record = athena_gate::legacy::legacy_exempt_record(
                &config.history.exempt,
                RecordId::generate(now),
                now,
            )?;
            Some(athena_gate::legacy::legacy_exempt_change(&record)?)
        };
        let mut run = CutoverRun {
            c0: c0.clone(),
            import: None,
            exempt_written: false,
        };
        if !has_manifest {
            let extra: Vec<Change> = exempt.iter().cloned().collect();
            run.import = Some(import_at_cutover(self.git, &self.remote, &c0, &extra)?);
            run.exempt_written = exempt.is_some();
        } else if let Some(change) = exempt {
            RecordsWriter::new(self.git).append_to_remote(
                &self.remote,
                &format!("Legacy-exempt record for the cutover at {c0}"),
                &[change],
                &[],
            )?;
            run.exempt_written = true;
        }
        Ok(Some(run))
    }
}

/// The cutover catch-up of the reconcile step (`athena land` and `athena status` run it first): reads
/// `[history] cutover_job` from `tip`, the fetched tip of the default branch, and runs the cutover for it, so
/// a landing whose own run stopped before the import still gets it. The lines are for a person; nothing is
/// returned when there is nothing to do, and a failure is a line, never an error.
pub fn cutover_catch_up(sink: &GitLandingRecords<'_>, tip: &str) -> Vec<String> {
    let Some(job) = cutover_job_for(sink.git, tip) else {
        return Vec::new();
    };
    match sink.run_cutover(&job) {
        Ok(Some(run)) if run.wrote() => vec![run.describe()],
        Ok(_) => Vec::new(),
        Err(problem) => vec![format!(
            "cutover: the import for job {job} did not finish ({problem:#}); it is tried again next time"
        )],
    }
}

/// Directory of declared-intervention files on the records branch (§5.3). Nothing writes it yet; a missing
/// directory reads as empty.
const DECLARED_DIR: &str = "declared-interventions";

impl ReconcileRecords for GitLandingRecords<'_> {
    fn snapshot(&self) -> Result<Option<RecordsSnapshot>> {
        let Some(tip) = RecordsWriter::new(self.git).sync_from(&self.remote)? else {
            return Ok(None);
        };
        let mut snapshot = RecordsSnapshot::default();
        for record in self.records_under(&tip, INTENT_DIR)? {
            if let HistoryRecord::LandingIntent(intent) = record {
                snapshot.intents.push(intent);
            }
        }
        for record in self.records_under(&tip, LANDING_DIR)? {
            if let HistoryRecord::Landing(landing) = record {
                snapshot.landings.push(landing);
            }
        }
        for record in self.records_under(&tip, DECLARED_DIR)? {
            if let HistoryRecord::DeclaredIntervention(declared) = record {
                snapshot.declared.push(declared);
            }
        }
        Ok(Some(snapshot))
    }

    fn write_landing(&self, facts: &LandingFacts) -> Result<RecordId> {
        // The head and the base must be present to list the pull request's commits.
        for commit in [&facts.head, &facts.expected_base] {
            if !self.git.ensure_commit(&self.remote, commit)? {
                bail!(
                    "commit {commit} is not available from remote {}",
                    self.remote
                );
            }
        }
        self.landing(facts)
    }
}
