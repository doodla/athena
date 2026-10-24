//! Record types for the records branch (`athena/records`), schema and validation only.
//!
//! Design: `docs/design/records-history.md` (decision D111), build job 1 of §10. These six types are the
//! per-record JSON files the design adds: `landing-intent`, `landing`, declared `intervention` (§5.3),
//! `checkpoint` (§9), `legacy-exempt` and `import-manifest` (§4).
//!
//! This module is deliberately separate from [`crate::records`]: it is not part of
//! [`crate::records::AnyRecord`], so the `records/` routing and the history check on main are unchanged. Nothing
//! reads or writes the records branch yet; the writer (§8.2) and the host-rules check are later jobs.
//!
//! The declared intervention uses the schema tag `athena.declared-intervention/1` and the Rust name
//! [`DeclaredIntervention`], because `athena.intervention/1` and `InterventionCategory` already name a different
//! record in [`crate::records`].

use crate::id::{JobId, RecordId};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};

/// Why a history record was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum HistoryError {
    /// The text is not valid JSON for any history record (unknown field, wrong or missing schema tag, bad type).
    #[error("not a valid history record: {0}")]
    Parse(String),
    /// A required text field is empty or only whitespace.
    #[error("{0} must not be empty")]
    Empty(&'static str),
    /// A value that must be a 40-character lowercase hex sha is not.
    #[error("{0} must be 40 lowercase hex characters")]
    BadSha(&'static str),
    /// A value that must be a 64-character lowercase hex SHA-256 is not.
    #[error("{0} must be 64 lowercase hex characters")]
    BadSha256(&'static str),
    /// A list that must have at least one entry is empty.
    #[error("{0} must have at least one entry")]
    NoEntries(&'static str),
    /// A pull request number is zero.
    #[error("pr_number must be greater than zero")]
    BadPrNumber,
    /// A landing commit of this kind must carry the named field and does not.
    #[error("a {kind} commit must carry {field}")]
    MissingForKind {
        /// The commit kind, as written in JSON.
        kind: &'static str,
        /// The missing field.
        field: &'static str,
    },
    /// A landing commit of this kind must not carry the named field and does.
    #[error("a {kind} commit must not carry {field}")]
    NotAllowedForKind {
        /// The commit kind, as written in JSON.
        kind: &'static str,
        /// The forbidden field.
        field: &'static str,
    },
}

fn is_lower_hex(text: &str, len: usize) -> bool {
    text.len() == len && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn check_text(value: &str, field: &'static str) -> Result<(), HistoryError> {
    if value.trim().is_empty() {
        Err(HistoryError::Empty(field))
    } else {
        Ok(())
    }
}

fn check_sha(value: &str, field: &'static str) -> Result<(), HistoryError> {
    if is_lower_hex(value, 40) {
        Ok(())
    } else {
        Err(HistoryError::BadSha(field))
    }
}

fn check_sha256(value: &str, field: &'static str) -> Result<(), HistoryError> {
    if is_lower_hex(value, 64) {
        Ok(())
    } else {
        Err(HistoryError::BadSha256(field))
    }
}

fn check_pr(pr_number: u64) -> Result<(), HistoryError> {
    if pr_number == 0 {
        Err(HistoryError::BadPrNumber)
    } else {
        Ok(())
    }
}

/// Written before a merge so the check can recognize the incoming commit (§5.2, §5.3; D111).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LandingIntent {
    /// Id of this record.
    pub id: RecordId,
    /// When the intent was written.
    pub created_at: Timestamp,
    /// Pull request number.
    pub pr_number: u64,
    /// Sha of the pull request's head commit.
    pub head_sha: String,
    /// Tree sha of the head commit.
    pub head_tree_sha: String,
    /// Main's tip when the intent was written.
    pub expected_base: String,
    /// Jobs whose work the pull request carries.
    pub job_ids: Vec<JobId>,
}

impl LandingIntent {
    /// Checks the fields against §5.3.
    pub fn validate(&self) -> Result<(), HistoryError> {
        check_pr(self.pr_number)?;
        check_sha(&self.head_sha, "head_sha")?;
        check_sha(&self.head_tree_sha, "head_tree_sha")?;
        check_sha(&self.expected_base, "expected_base")?;
        if self.job_ids.is_empty() {
            return Err(HistoryError::NoEntries("job_ids"));
        }
        Ok(())
    }

    /// Whether a main commit with tree `tree` and first parent `parent` is the merge this intent announced:
    /// the tree is the head's tree and the parent is the expected base (§8.1 step 2). Both must hold.
    pub fn matches(&self, tree: &str, parent: &str) -> bool {
        self.head_tree_sha == tree && self.expected_base == parent
    }
}

/// How the host combined the pull request into main (§5.3; D111).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeMethod {
    /// One new commit holding the whole pull request.
    Squash,
    /// A merge commit.
    Merge,
    /// The pull request's commits replayed onto main.
    Rebase,
}

/// What kind of commit a pull request commit is, which decides the records that cover it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandingKind {
    /// Produced by a job; carries the job and its verification record.
    JobLanding,
    /// A human commit covered by a declared intervention; claims no job.
    DeclaredIntervention,
    /// A commit that touches only records; claims no job.
    RecordsOnly,
}

impl LandingKind {
    /// The kind as written in JSON.
    pub fn as_str(self) -> &'static str {
        match self {
            LandingKind::JobLanding => "job_landing",
            LandingKind::DeclaredIntervention => "declared_intervention",
            LandingKind::RecordsOnly => "records_only",
        }
    }
}

/// One commit of a pull request, with the records that cover it (§5.3; D111).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LandingCommit {
    /// Commit sha.
    pub sha: String,
    /// Tree sha of the commit.
    pub tree: String,
    /// What kind of commit this is. Required.
    pub kind: LandingKind,
    /// The job that produced the commit; required for `job_landing`, forbidden otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job: Option<JobId>,
    /// The verification record that covers the commit; required for `job_landing`, forbidden otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<RecordId>,
}

impl LandingCommit {
    /// Checks the sha fields and that `job` and `verification` match `kind`.
    pub fn validate(&self) -> Result<(), HistoryError> {
        check_sha(&self.sha, "commits.sha")?;
        check_sha(&self.tree, "commits.tree")?;
        let kind = self.kind.as_str();
        if self.kind == LandingKind::JobLanding {
            if self.job.is_none() {
                return Err(HistoryError::MissingForKind { kind, field: "job" });
            }
            if self.verification.is_none() {
                return Err(HistoryError::MissingForKind {
                    kind,
                    field: "verification",
                });
            }
        } else {
            if self.job.is_some() {
                return Err(HistoryError::NotAllowedForKind { kind, field: "job" });
            }
            if self.verification.is_some() {
                return Err(HistoryError::NotAllowedForKind {
                    kind,
                    field: "verification",
                });
            }
        }
        Ok(())
    }
}

/// Written after the merge; names the commit the host made on main (§5.3; D111).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Landing {
    /// Id of this record.
    pub id: RecordId,
    /// When the landing was recorded.
    pub created_at: Timestamp,
    /// Pull request number.
    pub pr_number: u64,
    /// Sha of the pull request's head commit.
    pub head_sha: String,
    /// The commit the host made on main.
    pub host_commit: String,
    /// How the host merged.
    pub merge_method: MergeMethod,
    /// The pull request's commits in order, oldest first.
    pub commits: Vec<LandingCommit>,
    /// Id of the landing-intent record this landing fulfills.
    pub intent: RecordId,
}

impl Landing {
    /// Checks the fields against §5.3.
    pub fn validate(&self) -> Result<(), HistoryError> {
        check_pr(self.pr_number)?;
        check_sha(&self.head_sha, "head_sha")?;
        check_sha(&self.host_commit, "host_commit")?;
        if self.commits.is_empty() {
            return Err(HistoryError::NoEntries("commits"));
        }
        for commit in &self.commits {
            commit.validate()?;
        }
        Ok(())
    }
}

/// A human commit on main or in a pull request, declared by a person (§5.3, §6; D111).
///
/// Not the same record as [`crate::records::Intervention`]; see the module note on the schema tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeclaredIntervention {
    /// Id of this record.
    pub id: RecordId,
    /// When the declaration was recorded.
    pub created_at: Timestamp,
    /// Who declared it.
    pub declarer: String,
    /// Why the commit was made by hand.
    pub reason: String,
    /// The commit being declared.
    pub commit_sha: String,
}

impl DeclaredIntervention {
    /// Checks the fields against §5.3.
    pub fn validate(&self) -> Result<(), HistoryError> {
        check_text(&self.declarer, "declarer")?;
        check_text(&self.reason, "reason")?;
        check_sha(&self.commit_sha, "commit_sha")
    }
}

/// A passing point the incremental check starts from (§8, §9; D111).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    /// Id of this record.
    pub id: RecordId,
    /// When the checkpoint was written.
    pub created_at: Timestamp,
    /// The last main sha that passed.
    pub main_sha: String,
    /// The records-branch sha at that point.
    pub records_sha: String,
    /// Version of the verifier that produced the pass.
    pub verifier_version: String,
}

impl Checkpoint {
    /// Checks the fields against §9.
    pub fn validate(&self) -> Result<(), HistoryError> {
        check_sha(&self.main_sha, "main_sha")?;
        check_sha(&self.records_sha, "records_sha")?;
        check_text(&self.verifier_version, "verifier_version")
    }
}

/// One commit exempted from the legacy rule, with the reason (§4; D111).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyExemptEntry {
    /// Full sha of the exempt commit.
    pub sha: String,
    /// Why it is exempt.
    pub reason: String,
}

/// The pre-cutover exempt list, moved into the records history (§4; D111).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LegacyExempt {
    /// Id of this record.
    pub id: RecordId,
    /// When the list was recorded.
    pub created_at: Timestamp,
    /// The exempt commits.
    pub entries: Vec<LegacyExemptEntry>,
}

impl LegacyExempt {
    /// Checks the fields against §4.
    pub fn validate(&self) -> Result<(), HistoryError> {
        if self.entries.is_empty() {
            return Err(HistoryError::NoEntries("entries"));
        }
        for entry in &self.entries {
            check_sha(&entry.sha, "entries.sha")?;
            check_text(&entry.reason, "entries.reason")?;
        }
        Ok(())
    }
}

/// One imported file: its source path, git blob id at `C0`, and SHA-256 of its contents (§4; D111).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportedPath {
    /// Path of the file under `records/` at `C0`.
    pub path: String,
    /// Git blob id at `C0`.
    pub blob_id: String,
    /// SHA-256 of the file's contents.
    pub sha256: String,
}

/// The manifest written with the import, making completeness checkable at any time (§4; D111).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportManifest {
    /// Id of this record.
    pub id: RecordId,
    /// When the import was recorded.
    pub created_at: Timestamp,
    /// The cutover commit `C0` on main.
    pub c0: String,
    /// Every imported file.
    pub paths: Vec<ImportedPath>,
}

impl ImportManifest {
    /// Checks the fields against §4.
    pub fn validate(&self) -> Result<(), HistoryError> {
        check_sha(&self.c0, "c0")?;
        if self.paths.is_empty() {
            return Err(HistoryError::NoEntries("paths"));
        }
        for entry in &self.paths {
            check_text(&entry.path, "paths.path")?;
            check_sha(&entry.blob_id, "paths.blob_id")?;
            check_sha256(&entry.sha256, "paths.sha256")?;
        }
        Ok(())
    }
}

/// Any of the six history records, tagged by `schema` (§5.3; D111).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "schema")]
pub enum HistoryRecord {
    /// See [`LandingIntent`].
    #[serde(rename = "athena.landing-intent/1")]
    LandingIntent(LandingIntent),
    /// See [`Landing`].
    #[serde(rename = "athena.landing/1")]
    Landing(Landing),
    /// See [`DeclaredIntervention`].
    #[serde(rename = "athena.declared-intervention/1")]
    DeclaredIntervention(DeclaredIntervention),
    /// See [`Checkpoint`].
    #[serde(rename = "athena.checkpoint/1")]
    Checkpoint(Checkpoint),
    /// See [`LegacyExempt`].
    #[serde(rename = "athena.legacy-exempt/1")]
    LegacyExempt(LegacyExempt),
    /// See [`ImportManifest`].
    #[serde(rename = "athena.import-manifest/1")]
    ImportManifest(ImportManifest),
}

impl HistoryRecord {
    /// The schema tag this record is written with.
    pub fn schema(&self) -> &'static str {
        match self {
            Self::LandingIntent(_) => "athena.landing-intent/1",
            Self::Landing(_) => "athena.landing/1",
            Self::DeclaredIntervention(_) => "athena.declared-intervention/1",
            Self::Checkpoint(_) => "athena.checkpoint/1",
            Self::LegacyExempt(_) => "athena.legacy-exempt/1",
            Self::ImportManifest(_) => "athena.import-manifest/1",
        }
    }

    /// Checks the record's fields.
    pub fn validate(&self) -> Result<(), HistoryError> {
        match self {
            Self::LandingIntent(r) => r.validate(),
            Self::Landing(r) => r.validate(),
            Self::DeclaredIntervention(r) => r.validate(),
            Self::Checkpoint(r) => r.validate(),
            Self::LegacyExempt(r) => r.validate(),
            Self::ImportManifest(r) => r.validate(),
        }
    }

    /// Parses JSON text and validates the result.
    pub fn parse(text: &str) -> Result<Self, HistoryError> {
        let record: Self =
            serde_json::from_str(text).map_err(|e| HistoryError::Parse(e.to_string()))?;
        record.validate()?;
        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA_A: &str = "0123456789abcdef0123456789abcdef01234567";
    const SHA_B: &str = "fedcba9876543210fedcba9876543210fedcba98";
    const SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    fn rid() -> RecordId {
        RecordId::parse("20261024-033758-c2h3").unwrap()
    }

    fn jid() -> JobId {
        JobId::parse("20261024-033758-c2h3").unwrap()
    }

    fn now() -> Timestamp {
        "2026-10-24T03:37:58Z".parse().unwrap()
    }

    fn intent() -> LandingIntent {
        LandingIntent {
            id: rid(),
            created_at: now(),
            pr_number: 12,
            head_sha: SHA_A.into(),
            head_tree_sha: SHA_B.into(),
            expected_base: SHA_A.into(),
            job_ids: vec![jid()],
        }
    }

    fn landing() -> Landing {
        Landing {
            id: rid(),
            created_at: now(),
            pr_number: 12,
            head_sha: SHA_A.into(),
            host_commit: SHA_B.into(),
            merge_method: MergeMethod::Squash,
            commits: vec![LandingCommit {
                sha: SHA_A.into(),
                tree: SHA_B.into(),
                kind: LandingKind::JobLanding,
                job: Some(jid()),
                verification: Some(rid()),
            }],
            intent: rid(),
        }
    }

    fn declared() -> DeclaredIntervention {
        DeclaredIntervention {
            id: rid(),
            created_at: now(),
            declarer: "maintainer".into(),
            reason: "fixed a typo by hand".into(),
            commit_sha: SHA_A.into(),
        }
    }

    fn checkpoint() -> Checkpoint {
        Checkpoint {
            id: rid(),
            created_at: now(),
            main_sha: SHA_A.into(),
            records_sha: SHA_B.into(),
            verifier_version: "0.1.0".into(),
        }
    }

    fn legacy() -> LegacyExempt {
        LegacyExempt {
            id: rid(),
            created_at: now(),
            entries: vec![LegacyExemptEntry {
                sha: SHA_A.into(),
                reason: "predates the rule".into(),
            }],
        }
    }

    fn manifest() -> ImportManifest {
        ImportManifest {
            id: rid(),
            created_at: now(),
            c0: SHA_A.into(),
            paths: vec![ImportedPath {
                path: "records/jobs/bootstrap/job.json".into(),
                blob_id: SHA_B.into(),
                sha256: SHA256.into(),
            }],
        }
    }

    fn all() -> Vec<HistoryRecord> {
        vec![
            HistoryRecord::LandingIntent(intent()),
            HistoryRecord::Landing(landing()),
            HistoryRecord::DeclaredIntervention(declared()),
            HistoryRecord::Checkpoint(checkpoint()),
            HistoryRecord::LegacyExempt(legacy()),
            HistoryRecord::ImportManifest(manifest()),
        ]
    }

    fn json_of(record: &HistoryRecord) -> serde_json::Value {
        serde_json::to_value(record).unwrap()
    }

    #[test]
    fn every_type_round_trips_with_its_schema_tag() {
        for record in all() {
            let text = crate::to_pretty_json(&record).unwrap();
            let tag = format!("\"schema\": \"{}\"", record.schema());
            assert!(text.contains(&tag), "{text}");
            assert_eq!(HistoryRecord::parse(&text).unwrap(), record);
        }
    }

    #[test]
    fn an_intent_matches_only_on_head_tree_and_expected_base_together() {
        let i = intent();
        assert!(i.matches(SHA_B, SHA_A));
        assert!(!i.matches(SHA_A, SHA_A), "wrong tree");
        assert!(!i.matches(SHA_B, SHA_B), "wrong parent");
    }

    #[test]
    fn valid_examples_pass_validation() {
        for record in all() {
            record.validate().unwrap();
        }
    }

    #[test]
    fn empty_required_fields_are_rejected() {
        let mut d = declared();
        d.declarer = "  ".into();
        assert_eq!(d.validate(), Err(HistoryError::Empty("declarer")));
        let mut d = declared();
        d.reason.clear();
        assert_eq!(d.validate(), Err(HistoryError::Empty("reason")));
        let mut c = checkpoint();
        c.verifier_version.clear();
        assert!(c.validate().is_err());
        let mut l = legacy();
        l.entries[0].reason.clear();
        assert!(l.validate().is_err());
        l.entries.clear();
        assert_eq!(l.validate(), Err(HistoryError::NoEntries("entries")));
        let mut i = intent();
        i.job_ids.clear();
        assert!(i.validate().is_err());
        let mut l = landing();
        l.commits.clear();
        assert!(l.validate().is_err());
        let mut m = manifest();
        m.paths[0].path.clear();
        assert!(m.validate().is_err());
        m.paths.clear();
        assert!(m.validate().is_err());
    }

    fn landing_with(kind: LandingKind, job: bool, verification: bool) -> Landing {
        let mut l = landing();
        l.commits[0].kind = kind;
        l.commits[0].job = job.then(jid);
        l.commits[0].verification = verification.then(rid);
        l
    }

    #[test]
    fn job_landing_needs_job_and_verification() {
        landing_with(LandingKind::JobLanding, true, true)
            .validate()
            .unwrap();
        assert_eq!(
            landing_with(LandingKind::JobLanding, false, true).validate(),
            Err(HistoryError::MissingForKind {
                kind: "job_landing",
                field: "job"
            })
        );
        assert_eq!(
            landing_with(LandingKind::JobLanding, true, false).validate(),
            Err(HistoryError::MissingForKind {
                kind: "job_landing",
                field: "verification"
            })
        );
    }

    #[test]
    fn other_kinds_must_not_claim_a_job_or_verification() {
        for kind in [LandingKind::DeclaredIntervention, LandingKind::RecordsOnly] {
            landing_with(kind, false, false).validate().unwrap();
            assert_eq!(
                landing_with(kind, true, false).validate(),
                Err(HistoryError::NotAllowedForKind {
                    kind: kind.as_str(),
                    field: "job"
                })
            );
            assert_eq!(
                landing_with(kind, false, true).validate(),
                Err(HistoryError::NotAllowedForKind {
                    kind: kind.as_str(),
                    field: "verification"
                })
            );
        }
    }

    #[test]
    fn each_landing_kind_round_trips_and_omits_absent_fields() {
        for (kind, text) in [
            (LandingKind::JobLanding, "job_landing"),
            (LandingKind::DeclaredIntervention, "declared_intervention"),
            (LandingKind::RecordsOnly, "records_only"),
        ] {
            let job = kind == LandingKind::JobLanding;
            let record = HistoryRecord::Landing(landing_with(kind, job, job));
            let value = json_of(&record);
            let commit = &value["commits"][0];
            assert_eq!(commit["kind"], text);
            assert_eq!(commit.get("job").is_some(), job);
            assert_eq!(commit.get("verification").is_some(), job);
            let parsed = HistoryRecord::parse(&crate::to_pretty_json(&record).unwrap()).unwrap();
            assert_eq!(parsed, record);
        }
    }

    #[test]
    fn malformed_landing_commits_fail_to_parse() {
        let base = json_of(&HistoryRecord::Landing(landing()));
        let parse = |v: &serde_json::Value| HistoryRecord::parse(&v.to_string());
        parse(&base).unwrap();
        let mut unknown_kind = base.clone();
        unknown_kind["commits"][0]["kind"] = "squashed".into();
        assert!(matches!(parse(&unknown_kind), Err(HistoryError::Parse(_))));
        let mut missing_kind = base.clone();
        missing_kind["commits"][0]
            .as_object_mut()
            .unwrap()
            .remove("kind");
        assert!(matches!(parse(&missing_kind), Err(HistoryError::Parse(_))));
        let mut extra = base.clone();
        extra["commits"][0]["extra"] = 1.into();
        assert!(matches!(parse(&extra), Err(HistoryError::Parse(_))));
    }

    #[test]
    fn zero_pr_number_is_rejected() {
        let mut i = intent();
        i.pr_number = 0;
        assert_eq!(i.validate(), Err(HistoryError::BadPrNumber));
        let mut l = landing();
        l.pr_number = 0;
        assert_eq!(l.validate(), Err(HistoryError::BadPrNumber));
    }

    #[test]
    fn malformed_shas_are_rejected() {
        let upper = SHA_A.to_uppercase();
        let short = &SHA_A[..39];
        let nonhex = format!("{}g", &SHA_A[..39]);
        for bad in [upper.as_str(), short, nonhex.as_str(), ""] {
            let mut i = intent();
            i.head_sha = bad.into();
            assert_eq!(i.validate(), Err(HistoryError::BadSha("head_sha")), "{bad}");
            let mut i = intent();
            i.head_tree_sha = bad.into();
            assert!(i.validate().is_err());
            let mut i = intent();
            i.expected_base = bad.into();
            assert!(i.validate().is_err());
            let mut l = landing();
            l.host_commit = bad.into();
            assert!(l.validate().is_err());
            let mut l = landing();
            l.commits[0].tree = bad.into();
            assert!(l.validate().is_err());
            let mut d = declared();
            d.commit_sha = bad.into();
            assert!(d.validate().is_err());
            let mut c = checkpoint();
            c.records_sha = bad.into();
            assert!(c.validate().is_err());
            let mut e = legacy();
            e.entries[0].sha = bad.into();
            assert!(e.validate().is_err());
            let mut m = manifest();
            m.c0 = bad.into();
            assert!(m.validate().is_err());
            let mut m = manifest();
            m.paths[0].blob_id = bad.into();
            assert!(m.validate().is_err());
        }
    }

    #[test]
    fn malformed_sha256_is_rejected() {
        for bad in [
            SHA256.to_uppercase(),
            SHA256[..63].to_string(),
            SHA_A.to_string(),
        ] {
            let mut m = manifest();
            m.paths[0].sha256 = bad;
            assert_eq!(m.validate(), Err(HistoryError::BadSha256("paths.sha256")));
        }
    }

    #[test]
    fn parse_rejects_invalid_values_after_deserializing() {
        let mut value = json_of(&HistoryRecord::Checkpoint(checkpoint()));
        value["main_sha"] = "ABC".into();
        let err = HistoryRecord::parse(&value.to_string()).unwrap_err();
        assert_eq!(err, HistoryError::BadSha("main_sha"));
    }

    #[test]
    fn unknown_fields_are_rejected_at_every_level() {
        for record in all() {
            let mut value = json_of(&record);
            value["extra"] = true.into();
            assert!(matches!(
                HistoryRecord::parse(&value.to_string()),
                Err(HistoryError::Parse(_))
            ));
        }
        let mut value = json_of(&HistoryRecord::Landing(landing()));
        value["commits"][0]["extra"] = 1.into();
        assert!(HistoryRecord::parse(&value.to_string()).is_err());
    }

    #[test]
    fn wrong_or_missing_schema_tag_is_rejected() {
        for record in all() {
            let mut value = json_of(&record);
            value["schema"] = "athena.nonsense/1".into();
            assert!(HistoryRecord::parse(&value.to_string()).is_err());
            let mut value = json_of(&record);
            value["schema"] = format!("{}x", record.schema()).into();
            assert!(HistoryRecord::parse(&value.to_string()).is_err());
            value.as_object_mut().unwrap().remove("schema");
            assert!(HistoryRecord::parse(&value.to_string()).is_err());
        }
    }

    #[test]
    fn existing_intervention_schema_is_not_a_history_record() {
        let mut value = json_of(&HistoryRecord::DeclaredIntervention(declared()));
        value["schema"] = "athena.intervention/1".into();
        assert!(HistoryRecord::parse(&value.to_string()).is_err());
    }

    #[test]
    fn merge_method_is_snake_case_and_closed() {
        let mut value = json_of(&HistoryRecord::Landing(landing()));
        assert_eq!(value["merge_method"], "squash");
        value["merge_method"] = "fast_forward".into();
        assert!(HistoryRecord::parse(&value.to_string()).is_err());
    }
}
