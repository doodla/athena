//! The pre-merge check on a pull request's head (`docs/design/records-history.md` §5.1 and §10, D111).
//!
//! `athena check --pr-head <base> <head>` passes only when every commit in `base..head` is, by its content
//! alone (see [`crate::land::classify_commit`]):
//!
//! - a job landing whose verification passed and whose prediction (the accepted reading) precedes the work,
//! - a declared intervention made before the last job landing, or
//! - records only;
//!
//! and the head's governed content equals the content the last job landing verified, because the commits
//! after that landing are records only. A declared intervention after the last landing (or with no landing
//! at all) fails: a squash of it is not a valid landing until the cutover. Every offending commit is listed.

use crate::land::{PrCommit, classify_commit};
use anyhow::{Context, Result};
use athena_data::history::LandingKind;
use athena_data::records::Verdict;
use athena_gate::invariant::prediction_problem;
use athena_gate::jobs::JobFiles;
use athena_git::Git;
use athena_git::attrs::fingerprint;
use serde::Serialize;

/// One commit that stops the pull request from passing, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Offence {
    pub sha: String,
    pub reason: String,
}

/// The result of the check: the classified commits (those that could be classified) and every offence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrReport {
    pub base: String,
    pub head: String,
    pub commits: Vec<PrCommit>,
    pub offences: Vec<Offence>,
}

impl PrReport {
    pub fn passed(&self) -> bool {
        self.offences.is_empty()
    }
}

/// Checks the pull request `base..head`; see the module documentation.
pub fn check_pr_head(git: &Git, base: &str, head: &str) -> Result<PrReport> {
    let listing = git
        .run(["rev-list", "--reverse", &format!("{base}..{head}")])
        .with_context(|| format!("could not list the commits between {base} and {head}"))?;
    let shas: Vec<&str> = listing
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let mut report = PrReport {
        base: base.to_string(),
        head: head.to_string(),
        commits: vec![],
        offences: vec![],
    };
    if shas.is_empty() {
        report.offences.push(Offence {
            sha: head.to_string(),
            reason: format!("the PR head {head} adds no commit to the base {base}"),
        });
        return Ok(report);
    }

    let classified: Vec<(&str, Result<PrCommit>)> = shas
        .iter()
        .map(|sha| (*sha, classify_commit(git, base, head, sha)))
        .collect();
    let last_landing = classified
        .iter()
        .rposition(|(_, c)| c.as_ref().is_ok_and(|c| c.kind == LandingKind::JobLanding));

    for (i, (sha, result)) in classified.iter().enumerate() {
        let mut offend = |reason: String| {
            report.offences.push(Offence {
                sha: (*sha).to_string(),
                reason,
            });
        };
        let commit = match result {
            Ok(commit) => commit,
            Err(error) => {
                offend(format!("{error:#}"));
                continue;
            }
        };
        match commit.kind {
            LandingKind::JobLanding => {
                if let Some(reason) = landing_problem(git, commit)? {
                    offend(reason);
                }
            }
            LandingKind::DeclaredIntervention => {
                if last_landing.is_none_or(|l| i > l) {
                    offend(
                        "a declared intervention after the last job landing (or with no job landing): a \
                         squash of it is not a valid landing until the cutover"
                            .into(),
                    );
                }
            }
            LandingKind::RecordsOnly => {}
        }
    }
    report.commits = classified.into_iter().filter_map(|(_, c)| c.ok()).collect();

    // The head holds the last landing's verified content, since only records-only commits follow it.
    if report.offences.is_empty()
        && let Some(landing) = report
            .commits
            .iter()
            .rfind(|c| c.kind == LandingKind::JobLanding)
    {
        let job = landing.job.clone().context("a job landing names its job")?;
        let verified = JobFiles::load(git, &landing.tree, &job)?
            .verification
            .context("a job landing holds its verification")?
            .verified_fingerprint;
        let parent = git
            .run(["rev-parse", &format!("{}^", landing.sha)])?
            .trim()
            .to_string();
        let actual = fingerprint(git, head, &parent)?;
        if actual != verified {
            report.offences.push(Offence {
                sha: landing.sha.clone(),
                reason: format!(
                    "the head's content ({actual}) differs from the content this job landing verified \
                     ({verified}); only records-only commits may follow a job landing"
                ),
            });
        }
    }
    Ok(report)
}

/// Why a job landing does not stand (verification failed, or the prediction does not precede the work), if
/// it does not.
fn landing_problem(git: &Git, landing: &PrCommit) -> Result<Option<String>> {
    let job = landing.job.clone().context("a job landing names its job")?;
    let files = JobFiles::load(git, &landing.tree, &job)?;
    match &files.verification {
        None => return Ok(Some(format!("job {job} has no verification record"))),
        Some(v) if v.final_verdict != Verdict::Pass => {
            return Ok(Some(format!(
                "the verification of job {job} did not pass (final verdict fail)"
            )));
        }
        Some(_) => {}
    }
    let Some(n) = files.job.as_ref().and_then(|j| j.accepted_reading) else {
        return Ok(Some(format!(
            "job {job} has no job.json naming an accepted reading, so no prediction precedes its work"
        )));
    };
    Ok(prediction_problem(&files, &job, n))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::land::fixtures::{self, File, plain};
    use athena_data::records::{
        AnyRecord, Build, Criterion, CriterionKind, DefaultApplied, Deliverable, DeliverableAction,
        DriverInfo, Forecast, Forecasts, Invocation, InvocationCaps, InvocationContext,
        InvocationOutcome, ModelUsage, OutcomeState, Quantiles, Reading, Role, Tokens,
        Verification,
    };
    use athena_data::{JobId, Quantity, RecordId, Unit};
    use std::collections::BTreeMap;

    const JOB: &str = "20261024-115923-02gf";

    fn ts(text: &str) -> jiff::Timestamp {
        text.parse().unwrap()
    }

    fn job_id() -> JobId {
        JobId::parse(JOB).unwrap()
    }

    fn file(record: AnyRecord) -> File {
        (record.canonical_path(), record.to_json())
    }

    /// A repository with an excluded owner path; returns the base the pull request is cut from.
    fn pr_repo() -> (tempfile::TempDir, Git, String) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        fixtures::commit(
            &git,
            &[
                plain(".gitattributes", "/PRIVATE.md athena-exclude\n"),
                plain("a.txt", "a\n"),
            ],
            "base",
        );
        let base = git.head().unwrap().unwrap();
        (dir, git, base)
    }

    /// The reading, recorded at `at`.
    fn reading(at: &str) -> File {
        let forecast = Forecast {
            p_reading_accepted: 0.8,
            p_verification_passes: 0.8,
            p_user_intervenes: 0.15,
            cost_usd: Quantiles::new(0.2, 0.45, 1.1).unwrap(),
            wall_time_s: Quantiles::new(120.0, 240.0, 540.0).unwrap(),
        };
        file(AnyRecord::Reading(Box::new(Reading {
            job: job_id(),
            n: 1,
            at: ts(at),
            invocation_seq: 1,
            base_commit: "0".repeat(40),
            config_sha256: "a".repeat(64),
            defaults_sha256: "b".repeat(64),
            defaults_version: 1,
            title: "Change a".into(),
            summary: "Change a.".into(),
            class_of_work: "code".into(),
            capability: Some("change".into()),
            decline_reason: None,
            producer_model: "m".into(),
            verifier_model: "v".into(),
            defaults_applied: vec![DefaultApplied {
                id: "tested".into(),
                how: "a test".into(),
            }],
            assumptions: vec![],
            question: None,
            acceptance_criteria: vec![Criterion {
                text: "a changes".into(),
                kind: CriterionKind::Content,
            }],
            deliverables: vec![Deliverable {
                path: "a.txt".into(),
                action: DeliverableAction::Modify,
                p: 0.95,
            }],
            out_of_scope: vec![],
            forecasts: Forecasts {
                reader: forecast.clone(),
                reference: forecast,
            },
            departures: None,
            urgent_reason: None,
            hard_reason: None,
            producer_effort: None,
        })))
    }

    fn dispatch(at: &str) -> File {
        let text = format!(
            "{{\"schema\":\"athena.job/1\",\"id\":\"{JOB}\",\"kind\":\"request\",\"title\":\"t\",\"class_of_work\":\"code\",\"capability\":\"change\",\"accepted_reading\":1,\"accepted_by\":\"user\",\"owner\":{{\"role\":\"producer\",\"capability\":null,\"model\":null,\"note\":null}},\"verifier_model\":\"v\",\"caps\":null,\"base_commit\":null,\"config_sha256\":null,\"defaults_sha256\":null,\"athena\":null,\"driver\":null,\"brief_sha256\":null,\"dispatched_at\":\"{at}\",\"request_path\":null,\"request_sha256\":null,\"cost\":null,\"tokens\":null,\"note\":null}}"
        );
        file(serde_json::from_str::<AnyRecord>(&text).unwrap())
    }

    fn invocation(seq: u32, role: Role, start: &str) -> File {
        let tokens = |v: f64| Quantity::measured(v, Unit::Tokens, "result.usage");
        file(AnyRecord::Invocation(Box::new(Invocation {
            job: job_id(),
            seq,
            role,
            driver: DriverInfo {
                name: "claude-code".into(),
                version: "2.1.299".into(),
            },
            athena: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            model: "m".into(),
            effort: None,
            context: InvocationContext {
                components: vec![],
                prompt_sha256: "0".repeat(64),
                prompt_tokens: Quantity::estimated(Some(1.0), Unit::Tokens, "estimate", None, None)
                    .unwrap(),
                host_baseline: Quantity::estimated(Some(600.0), Unit::Tokens, "E3", None, None)
                    .unwrap(),
                tools: vec![],
            },
            caps: InvocationCaps {
                max_budget_usd: 3.0,
                timeout_s: 1800,
            },
            started_at: ts(start),
            ended_at: ts("2026-10-24T13:00:00Z"),
            wall_time: Quantity::measured(60.0, Unit::Seconds, "athena clock"),
            tokens: Tokens {
                input: tokens(10.0),
                output: tokens(20.0),
                cache_read: tokens(30.0),
                cache_write: tokens(40.0),
            },
            models_used: BTreeMap::from([(
                "m".to_string(),
                ModelUsage {
                    input: 10,
                    output: 20,
                    cache_read: 30,
                    cache_write: 40,
                    cost_usd: 0.1,
                },
            )]),
            cost: Quantity::reported(0.1, Unit::Usd, "Claude Code total_cost_usd"),
            outcome: InvocationOutcome::Success,
            detail: None,
            turns: Some(3),
            commands_run: vec![],
            policy_denials: vec![],
        })))
    }

    fn verification(fingerprint: &str, verdict: Verdict) -> File {
        file(AnyRecord::Verification(Box::new(Verification {
            job: job_id(),
            candidate_commit: "c".repeat(40),
            verified_fingerprint: fingerprint.to_string(),
            tool_config_from_base: vec![],
            checks: vec![],
            check_cleanup: vec![],
            verifier_invocation: None,
            criteria: vec![],
            findings: vec![],
            request_reading_mismatch: None,
            agent_verdict: Some(verdict),
            final_verdict: verdict,
            reasons: vec![],
            inputs: vec![],
            summary: None,
        })))
    }

    /// Records that open the job: the reading (at `reading_at`), the dispatch (at 12:00), a producer
    /// invocation (from 12:30). One commit.
    fn open_job(git: &Git, reading_at: &str) -> String {
        fixtures::commit(
            git,
            &[
                reading(reading_at),
                dispatch("2026-10-24T12:00:00Z"),
                invocation(2, Role::Producer, "2026-10-24T12:30:00Z"),
            ],
            "Open",
        )
    }

    /// A landing commit that edits a.txt and adds closing records verified against the real fingerprint of
    /// the commit's own content (computed after a first commit, then amended in).
    fn land(git: &Git, verdict: Verdict) -> String {
        land_verifying(git, verdict, None)
    }

    /// As [`land`], but the verification may record `claimed` instead of the real fingerprint.
    fn land_verifying(git: &Git, verdict: Verdict, claimed: Option<&str>) -> String {
        let parent = git.head().unwrap().unwrap();
        let mut files = vec![
            fixtures::candidate(&job_id()),
            fixtures::outcome(&job_id(), OutcomeState::Landed, false),
            verification(&"f".repeat(64), verdict),
            plain("a.txt", "landed by a job\n"),
        ];
        let first = fixtures::commit(git, &files, "Landing");
        let real = fingerprint(git, &first, &parent).unwrap();
        files[2] = verification(claimed.unwrap_or(&real), verdict);
        for (path, text) in &files {
            std::fs::write(git.dir().join(path), text).unwrap();
        }
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "--amend", "-m", "Landing"])
            .unwrap();
        let sha = git.head().unwrap().unwrap();
        assert_eq!(fingerprint(git, &sha, &parent).unwrap(), real);
        sha
    }

    fn records_only(git: &Git, name: &str) -> String {
        fixtures::commit(
            git,
            &[plain(
                &format!("records/introspections/2026-10/{name}.json"),
                "{}\n",
            )],
            "Records",
        )
    }

    fn intervene(git: &Git, text: &str) -> String {
        let id = RecordId::parse("20261024-121000-abcd").unwrap();
        fixtures::commit(
            git,
            &[
                plain("a.txt", text),
                plain(
                    &format!("records/interventions/{}/{id}.json", id.month()),
                    "{}\n",
                ),
            ],
            "Intervention",
        )
    }

    fn check(git: &Git, base: &str) -> PrReport {
        let head = git.head().unwrap().unwrap();
        check_pr_head(git, base, &head).unwrap()
    }

    fn offending(report: &PrReport) -> Vec<&str> {
        report.offences.iter().map(|o| o.sha.as_str()).collect()
    }

    #[test]
    fn an_unverified_commit_fails_and_is_named() {
        let (_dir, git, base) = pr_repo();
        let bad = fixtures::commit(&git, &[plain("a.txt", "hand edit\n")], "Hand edit");
        let report = check(&git, &base);
        assert!(!report.passed());
        assert_eq!(offending(&report), [bad.as_str()]);
        assert!(
            report.offences[0]
                .reason
                .contains("no verification covers it"),
            "{:?}",
            report.offences
        );
    }

    #[test]
    fn a_head_shaped_like_an_athena_pull_request_passes() {
        let (_dir, git, base) = pr_repo();
        records_only(&git, "20261024-110000-aaaa");
        open_job(&git, "2026-10-24T11:30:00Z");
        intervene(&git, "by hand\n");
        land(&git, Verdict::Pass);
        records_only(&git, "20261024-140000-bbbb");
        let report = check(&git, &base);
        assert!(report.passed(), "{:?}", report.offences);
        let kinds: Vec<_> = report.commits.iter().map(|c| c.kind).collect();
        assert_eq!(
            kinds,
            [
                LandingKind::RecordsOnly,
                LandingKind::RecordsOnly,
                LandingKind::DeclaredIntervention,
                LandingKind::JobLanding,
                LandingKind::RecordsOnly,
            ]
        );
    }

    #[test]
    fn a_declared_intervention_after_the_landing_fails() {
        let (_dir, git, base) = pr_repo();
        open_job(&git, "2026-10-24T11:30:00Z");
        let landing = land(&git, Verdict::Pass);
        let late = intervene(&git, "by hand, after the landing\n");
        let report = check(&git, &base);
        assert!(!report.passed());
        assert_eq!(offending(&report), [late.as_str()]);
        assert!(!offending(&report).contains(&landing.as_str()));
        assert!(
            report.offences[0]
                .reason
                .contains("a squash of it is not a valid landing until the cutover"),
            "{:?}",
            report.offences
        );
    }

    #[test]
    fn a_landing_whose_prediction_follows_the_work_fails() {
        let (_dir, git, base) = pr_repo();
        open_job(&git, "2026-10-24T12:45:00Z");
        let landing = land(&git, Verdict::Pass);
        let report = check(&git, &base);
        assert_eq!(offending(&report), [landing.as_str()]);
        assert!(
            report.offences[0].reason.contains("reading-1.json"),
            "{:?}",
            report.offences
        );
    }

    #[test]
    fn a_landing_whose_verification_failed_fails() {
        let (_dir, git, base) = pr_repo();
        open_job(&git, "2026-10-24T11:30:00Z");
        let landing = land(&git, Verdict::Fail);
        let report = check(&git, &base);
        assert_eq!(offending(&report), [landing.as_str()]);
        assert!(report.offences[0].reason.contains("did not pass"));
    }

    #[test]
    fn a_head_whose_content_differs_from_the_verified_content_fails() {
        let (_dir, git, base) = pr_repo();
        open_job(&git, "2026-10-24T11:30:00Z");
        // The verification claims content other than what the landing commit holds.
        let landing = land_verifying(&git, Verdict::Pass, Some(&"e".repeat(64)));
        records_only(&git, "20261024-140000-bbbb");
        let report = check(&git, &base);
        assert!(!report.passed());
        assert!(
            report.offences.iter().any(|o| o.sha == landing
                && o.reason
                    .contains("differs from the content this job landing verified")),
            "{:?}",
            report.offences
        );
    }

    #[test]
    fn every_offending_commit_is_listed_in_order() {
        let (_dir, git, base) = pr_repo();
        let unverified = fixtures::commit(&git, &[plain("a.txt", "hand edit\n")], "Hand edit");
        open_job(&git, "2026-10-24T12:45:00Z");
        let landing = land(&git, Verdict::Pass);
        let late = intervene(&git, "after\n");
        let report = check(&git, &base);
        assert_eq!(
            offending(&report),
            [unverified.as_str(), landing.as_str(), late.as_str()]
        );
    }

    #[test]
    fn an_empty_range_fails() {
        let (_dir, git, base) = pr_repo();
        let report = check_pr_head(&git, &base, &base).unwrap();
        assert!(!report.passed());
    }
}
