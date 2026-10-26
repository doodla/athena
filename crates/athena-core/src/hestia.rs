//! Hestia's sweep skeleton (`docs/design/hestia.md` section 9, job 1; decision D129).
//!
//! A sweep runs a list of [`Check`]s and records one report, an `athena.hestia-sweep/1` record: every check that
//! ran, with the number of things it looked at and the number of findings, and each finding. A check that
//! returns an error does not stop the sweep and does not look clean: [`run_sweep`] turns the error into a
//! finding for that check and marks the check as errored. Hestia reports and never gates work (D127).
//!
//! The trigger is an explicit command: `athena hestia sweep` runs and records a sweep, and `athena hestia last`
//! prints the newest one. Nothing runs a sweep on a timer or inside `athena drain`; D129 says why.
//!
//! This skeleton ships no checks. [`default_checks`] is empty, and the checks of the design's rows 1 to 8 are
//! later jobs that add themselves to it.

use crate::Athena;
use crate::commits::{NewFile, message};
use crate::store::Records;
use anyhow::{Result, bail};
use athena_data::RecordId;
use athena_data::records::{AnyRecord, HestiaCheckResult};
use jiff::Timestamp;

pub use athena_data::records::HestiaCheckResult as CheckResult;
pub use athena_data::records::HestiaFinding as Finding;
pub use athena_data::records::HestiaFindingState as FindingState;
pub use athena_data::records::HestiaSweep as Sweep;

/// What a check saw when it ran without error.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CheckOutput {
    /// How many things it looked at (branches, requests, docs).
    pub examined: u32,
    pub findings: Vec<Finding>,
}

/// One Hestia check: a fact it looks at, and what is wrong with it.
pub trait Check {
    /// A short stable id, such as `leftover-branches`. Unique within a sweep.
    fn id(&self) -> &str;
    /// Looks at the system. `Err` means the check could not run, which is itself reported.
    fn run(&self, now: Timestamp) -> Result<CheckOutput>;
}

/// The checks a sweep runs when none are given. Empty in the skeleton; later jobs register theirs here.
pub fn default_checks() -> Vec<Box<dyn Check>> {
    Vec::new()
}

/// Runs every check, in order, and reports on all of them. A check that fails, or that returns a finding with
/// no subject or no evidence, becomes one open finding for that check, and the sweep goes on. Whatever check id
/// a check puts on its findings, they are filed under the check that returned them.
pub fn run_sweep(checks: &[Box<dyn Check>], id: RecordId, at: Timestamp) -> Sweep {
    let mut results = Vec::new();
    let mut findings = Vec::new();
    for check in checks {
        let check_id = check.id().to_string();
        let outcome = check.run(at).and_then(|output| {
            if output
                .findings
                .iter()
                .any(|f| f.subject.trim().is_empty() || f.evidence.trim().is_empty())
            {
                bail!("a finding came back with no subject or no evidence");
            }
            Ok(output)
        });
        match outcome {
            Ok(output) => {
                results.push(CheckResult {
                    check: check_id.clone(),
                    examined: output.examined,
                    count: output.findings.len() as u32,
                    errored: false,
                });
                findings.extend(output.findings.into_iter().map(|f| Finding {
                    check: check_id.clone(),
                    ..f
                }));
            }
            Err(error) => {
                results.push(HestiaCheckResult {
                    check: check_id.clone(),
                    examined: 0,
                    count: 1,
                    errored: true,
                });
                findings.push(Finding {
                    check: check_id.clone(),
                    subject: check_id,
                    evidence: format!("the check failed to run: {error:#}"),
                    age_seconds: 0,
                    state: FindingState::Open,
                });
            }
        }
    }
    Sweep {
        id,
        at,
        checks: results,
        findings,
    }
}

impl Athena {
    /// Runs `checks`, commits the report as a record, and returns it.
    pub fn hestia_sweep(&self, checks: &[Box<dyn Check>]) -> Result<Sweep> {
        let known = match self.git.head()? {
            Some(head) => Records::load(&self.git, &head)?,
            None => bail!(
                "the repository has no commits yet; commit athena.toml and defaults.toml first"
            ),
        };
        let now = crate::now();
        let mut id = RecordId::generate(now);
        while known.has_record_id(&id) {
            id = RecordId::generate(now);
        }
        let sweep = run_sweep(checks, id, now);
        sweep.validate().map_err(|e| anyhow::anyhow!(e))?;
        let record = AnyRecord::HestiaSweep(Box::new(sweep.clone()));
        let subject = format!(
            "Hestia sweep: {} check(s), {} finding(s)",
            sweep.checks.len(),
            sweep.findings.len()
        );
        self.commit_records(
            &[NewFile::record(&record)],
            &message(
                &subject,
                "A report of Hestia's checks. It lists every check that ran with its count; a check that failed to run is a finding.",
                &[("Athena-Record", record.canonical_path())],
            ),
        )?;
        Ok(sweep)
    }

    /// The newest recorded sweep, or None when none was recorded.
    pub fn last_sweep(&self) -> Result<Option<Sweep>> {
        let Some(head) = self.git.head()? else {
            return Ok(None);
        };
        let records = Records::load(&self.git, &head)?;
        Ok(Sweep::latest(&records.hestia_sweeps).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::StateDir;
    use athena_agent::fake::FakeAgentDriver;
    use athena_data::records::Build;
    use athena_git::Git;

    /// A check with a fixed answer.
    struct Fixed {
        id: &'static str,
        answer: std::result::Result<CheckOutput, &'static str>,
    }

    impl Check for Fixed {
        fn id(&self) -> &str {
            self.id
        }
        fn run(&self, _: Timestamp) -> Result<CheckOutput> {
            self.answer.clone().map_err(|e| anyhow::anyhow!(e))
        }
    }

    fn clean(id: &'static str, examined: u32) -> Box<dyn Check> {
        Box::new(Fixed {
            id,
            answer: Ok(CheckOutput {
                examined,
                findings: Vec::new(),
            }),
        })
    }

    fn finding(check: &str, subject: &str) -> Finding {
        Finding {
            check: check.into(),
            subject: subject.into(),
            evidence: "merged into main three weeks ago".into(),
            age_seconds: 1_814_400,
            state: FindingState::Open,
        }
    }

    fn rid(text: &str) -> RecordId {
        RecordId::parse(text).unwrap()
    }

    #[test]
    fn a_sweep_where_every_check_finds_nothing_lists_every_check_with_a_count_of_zero() {
        let checks = vec![clean("alpha", 3), clean("beta", 0), clean("gamma", 12)];
        let sweep = run_sweep(&checks, rid("20261026-090000-5abc"), crate::now());
        assert_eq!(
            sweep
                .checks
                .iter()
                .map(|c| (c.check.as_str(), c.count, c.examined, c.errored))
                .collect::<Vec<_>>(),
            [
                ("alpha", 0, 3, false),
                ("beta", 0, 0, false),
                ("gamma", 0, 12, false)
            ]
        );
        assert!(sweep.findings.is_empty());
        assert!(sweep.validate().is_ok());
    }

    #[test]
    fn a_check_that_errors_becomes_a_finding_and_the_other_checks_still_run() {
        let checks: Vec<Box<dyn Check>> = vec![
            clean("alpha", 3),
            Box::new(Fixed {
                id: "broken",
                answer: Err("cannot read the branch list"),
            }),
            Box::new(Fixed {
                id: "noisy",
                answer: Ok(CheckOutput {
                    examined: 5,
                    // A check cannot file a finding under another check's id.
                    findings: vec![finding("someone-else", "old-branch")],
                }),
            }),
            clean("omega", 1),
        ];
        let sweep = run_sweep(&checks, rid("20261026-090000-5abc"), crate::now());
        assert_eq!(
            sweep
                .checks
                .iter()
                .map(|c| (c.check.as_str(), c.count, c.errored))
                .collect::<Vec<_>>(),
            [
                ("alpha", 0, false),
                ("broken", 1, true),
                ("noisy", 1, false),
                ("omega", 0, false)
            ]
        );
        let broken: Vec<&Finding> = sweep
            .findings
            .iter()
            .filter(|f| f.check == "broken")
            .collect();
        assert_eq!(broken.len(), 1);
        assert_eq!(broken[0].subject, "broken");
        assert!(
            broken[0].evidence.contains("cannot read the branch list"),
            "{}",
            broken[0].evidence
        );
        assert_eq!(broken[0].age_seconds, 0);
        assert_eq!(broken[0].state, FindingState::Open);
        let noisy = sweep.findings.iter().find(|f| f.check == "noisy").unwrap();
        assert_eq!(noisy.subject, "old-branch");
        assert_eq!(sweep.findings.len(), 2);
        assert!(sweep.validate().is_ok());
    }

    #[test]
    fn a_finding_without_evidence_is_reported_as_a_failed_check() {
        let mut bad = finding("x", "branch");
        bad.evidence = " ".into();
        let checks: Vec<Box<dyn Check>> = vec![Box::new(Fixed {
            id: "sloppy",
            answer: Ok(CheckOutput {
                examined: 1,
                findings: vec![bad],
            }),
        })];
        let sweep = run_sweep(&checks, rid("20261026-090000-5abc"), crate::now());
        assert!(sweep.checks[0].errored);
        assert!(sweep.findings[0].evidence.contains("no evidence"));
        assert!(sweep.validate().is_ok());
    }

    #[test]
    fn the_skeleton_registers_no_checks() {
        assert!(default_checks().is_empty());
    }

    fn athena_in(dir: &std::path::Path, state: &std::path::Path) -> Athena {
        let git = Git::at(dir);
        git.run(["init", "-q", "-b", "main"]).unwrap();
        for (key, value) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("core.hooksPath", "/dev/null"),
        ] {
            git.run(["config", key, value]).unwrap();
        }
        std::fs::write(
            dir.join("athena.toml"),
            include_str!("../../../athena.toml"),
        )
        .unwrap();
        std::fs::write(
            dir.join("defaults.toml"),
            include_str!("../../../defaults.toml"),
        )
        .unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "Start"]).unwrap();
        Athena {
            git,
            state: StateDir::new(state.join("repo"))
                .with_free_space(crate::test_support::plenty_of_space),
            driver: std::sync::Arc::new(FakeAgentDriver::deterministic()),
            capabilities: vec![],
            build: Build {
                version: "0.1.0".into(),
                source_commit: "0".repeat(40),
                dirty: false,
            },
            athena_bin: "/bin/false".into(),
            nested: None,
            pinner: Box::new(crate::fake::CountingPinner::new(&"0".repeat(40))),
            jev: None,
        }
    }

    #[test]
    fn a_committed_sweep_is_read_back_and_the_last_one_is_the_newest() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let athena = athena_in(dir.path(), state.path());
        assert_eq!(athena.last_sweep().unwrap(), None);

        let first = athena.hestia_sweep(&[clean("alpha", 2)]).unwrap();
        assert_eq!(athena.last_sweep().unwrap(), Some(first.clone()));
        let broken: Vec<Box<dyn Check>> = vec![
            clean("alpha", 2),
            Box::new(Fixed {
                id: "broken",
                answer: Err("boom"),
            }),
        ];
        let second = athena.hestia_sweep(&broken).unwrap();
        assert_ne!(first.id, second.id);
        let last = athena.last_sweep().unwrap().unwrap();
        assert_eq!(last, second);
        assert_eq!(last.checks.len(), 2);
        assert!(last.findings[0].evidence.contains("boom"));
        let path = format!("records/hestia-sweeps/{}/{}.json", last.id.month(), last.id);
        assert!(dir.path().join(&path).exists(), "{path}");
    }
}
