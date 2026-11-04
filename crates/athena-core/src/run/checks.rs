//! The checks stages of a job: the baseline run on the base commit and the executor that runs one routed
//! check in the verifier's snapshot (structure-pass §3.3, stages S6 and S7).

use super::*;

/// Runs one routed check in the verifier's snapshot, then puts the snapshot back as it was prepared.
///
/// It is called only when a check really runs here, so it is also where the free-disk requirement of the
/// checks is applied: once, just before the first check starts. A CI result that stands in for the checks, a
/// resumed job and a skipped check never reach it. Low space comes back as a [`crate::disk::LowDisk`] error
/// that `check_and_verify` turns into a hold.
pub(super) struct SnapExec<'a> {
    pub(super) athena: &'a Athena,
    pub(super) snapshot: &'a Snapshot,
    pub(super) prepared: &'a str,
    pub(super) env: &'a std::collections::BTreeMap<String, String>,
    pub(super) registry: &'a athena_agent::procs::Registry,
    pub(super) job: &'a str,
    pub(super) timeout: std::time::Duration,
    pub(super) redactor: &'a Redactor,
    pub(super) cleanup: std::cell::RefCell<Vec<String>>,
    /// The job the checks belong to, for the disk gate.
    pub(super) id: &'a JobId,
    pub(super) ui: &'a dyn Ui,
    pub(super) min_free_gb: u64,
    /// Whether the disk gate has been applied yet.
    pub(super) gated: std::cell::Cell<bool>,
}

impl crate::routing::Exec for SnapExec<'_> {
    fn exec(&self, spec: &crate::capability::CheckSpec) -> Result<CheckRun> {
        self.athena.refuse_if_stopped()?;
        if !self.gated.replace(true)
            && let Some(low) = self.athena.disk_gate(self.min_free_gb, self.id, self.ui)
        {
            return Err(low.into());
        }
        #[cfg(test)]
        let seamed = check_seam::current().map(|fake| fake(spec));
        #[cfg(not(test))]
        let seamed: Option<CheckRun> = None;
        let run = match seamed {
            Some(run) => run,
            None => run_check(
                spec,
                &CheckContext {
                    dir: self.snapshot.dir(),
                    env: self.env,
                    timeout: self.timeout,
                    registry: self.registry,
                    job: self.job,
                    redactor: self.redactor,
                },
            )?,
        };
        self.cleanup
            .borrow_mut()
            .extend(self.snapshot.restore_to(self.prepared)?);
        Ok(run)
    }

    fn redact(&self, text: &str) -> Result<String> {
        Ok(self.redactor.redact(text)?)
    }
}

/// The path of `relative` inside the scratch copy at `root`, refused when `root` or a directory between
/// holds a symbolic link there (the candidate may contain one), and with a link at the file itself removed,
/// so a write never lands outside the copy.
fn scratch_target(root: &std::path::Path, relative: &str) -> Result<PathBuf> {
    let mut at = root.to_path_buf();
    let parts: Vec<&str> = relative.split('/').collect();
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() || *part == ".." || *part == "." {
            bail!("the path {relative} is not a plain relative path");
        }
        at.push(part);
        let Ok(meta) = std::fs::symlink_metadata(&at) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            if i + 1 < parts.len() {
                bail!("the path {relative} passes through a symbolic link");
            }
            std::fs::remove_file(&at)?;
        }
    }
    Ok(at)
}

/// Whether the demonstrations run in this build. A production build always runs them. A test build, or one
/// with the `test-support` feature (the integration tests run planned jobs with fake agents), runs them only
/// when a test installed [`demo_seam`], so no test starts a real `cargo test` by accident.
#[cfg(any(test, feature = "test-support"))]
fn demo_runs_here() -> bool {
    demo_seam::current().is_some()
}

/// Whether the demonstrations run in this build: always, in production.
#[cfg(not(any(test, feature = "test-support")))]
fn demo_runs_here() -> bool {
    true
}

/// The command to run for a parsed `cargo test ...`: as parsed, except that an installed [`demo_seam`] maps it.
fn mapped_argv(argv: &[String]) -> Vec<String> {
    #[cfg(any(test, feature = "test-support"))]
    if let Some(map) = demo_seam::current() {
        return map(argv);
    }
    argv.to_vec()
}

/// The seam tests use to change the program a demonstration runs, so the scratch copy, the revert, the process
/// and the classification stay production code while no real cargo runs. It maps the parsed argv to the argv
/// that runs, and its being installed is what makes a test or `test-support` build run demonstrations at all.
/// It is per thread, so parallel tests do not see each other's mappers. Production builds do not have it.
#[cfg(any(test, feature = "test-support"))]
pub(crate) mod demo_seam {
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Maps the parsed argv of a demonstration command to the argv that runs.
    pub type Mapper = Rc<dyn Fn(&[String]) -> Vec<String>>;

    thread_local! {
        static SEAM: RefCell<Option<Mapper>> = const { RefCell::new(None) };
    }

    /// Clears the seam when dropped. Only this crate's own tests install a mapper, so a `test-support` build
    /// of it never has one and runs no demonstration.
    #[cfg(test)]
    pub struct Guard;

    #[cfg(test)]
    impl Drop for Guard {
        fn drop(&mut self) {
            SEAM.with(|s| *s.borrow_mut() = None);
        }
    }

    #[cfg(test)]
    pub fn install(map: Mapper) -> Guard {
        SEAM.with(|s| *s.borrow_mut() = Some(map));
        Guard
    }

    pub fn current() -> Option<Mapper> {
        SEAM.with(|s| s.borrow().clone())
    }
}

impl Athena {
    /// Removes the producer's build directory before the candidate's checks start. The checks build in their
    /// own directory, so this one is idle from here on, and it is the largest thing on the disk: left in place
    /// the job would hold two full builds while the checks compile and run. Only the build goes; the
    /// candidate and its work tree stay. A failure to remove is a warning line only. It runs before the
    /// free-disk check of the checks ([`SnapExec`]), so the job's own build never holds that check.
    pub(super) fn free_producer_build(&self, ctx: &RunContext<'_>, id: &JobId) {
        match self.state.remove_produce(id) {
            Ok(0) => {}
            Ok(bytes) => ctx.ui.line(&format!(
                "disk       removed the producer's build directory: {}",
                crate::state::format_bytes(bytes)
            )),
            Err(why) => ctx.ui.line(&format!(
                "warning: could not remove the producer's build directory: {why}"
            )),
        }
    }

    /// Runs the capability's configured checks on a fresh snapshot of the base commit and returns the
    /// baseline record. It never fails: a snapshot or environment error, or a nested run, gives a record
    /// with no checks and a note saying why. `before_invocation` is the sequence number the first
    /// producer invocation will get. Verification reads the result to mark pre-existing failures. The checks
    /// run through `run_check`, as the candidate's do, so each failed run records the names of its failing
    /// tests and verification compares test by test, not by check name alone.
    pub(super) fn take_baseline(
        &self,
        ctx: &RunContext<'_>,
        job_dir: &std::path::Path,
        seq: u32,
    ) -> Baseline {
        let mut baseline = Baseline {
            job: ctx.id().clone(),
            base_commit: ctx.prediction.clone(),
            before_invocation: seq,
            checks: Vec::new(),
            note: None,
        };
        if self.nested.is_some() {
            baseline.note =
                Some("not run: a nested run skips checks; the outer job runs its own".into());
            return baseline;
        }
        let dir = job_dir.join("baseline");
        let outcome = self.baseline_checks(ctx, job_dir, &dir);
        let _ = crate::pin::remove_tree(&dir);
        match outcome {
            Ok(checks) => baseline.checks = checks,
            Err(e) => baseline.note = Some(format!("the baseline could not run: {e:#}")),
        }
        baseline
    }

    /// Runs the accepted plan's "fails without the fix" demonstrations and returns one record each, to go
    /// to the verifier under the checks. The verifier gets no producer report and cannot change files, so
    /// it could never see this run otherwise.
    ///
    /// Each distinct plain `cargo test ...` command runs once (at most [`crate::verify::MAX_DEMONSTRATION_RUNS`]) in a
    /// scratch copy of the candidate, `<job_dir>/revert`, where the changed non-test files are back at their
    /// base versions and the tests are kept. The copy is removed on every path; the candidate, its commit
    /// and the job repository are only read. It never fails the job: a problem is a "not run" record, and no
    /// record is a check, so the recorded checks still decide the verdict. A nested run, a job with no plan
    /// and a plan with no such demonstration return nothing. In a test build, or one with the `test-support`
    /// feature, it returns nothing unless a test installed [`demo_seam`], so no real cargo runs there.
    pub(super) fn demonstration_records(
        &self,
        ctx: &RunContext<'_>,
        job_dir: &std::path::Path,
        plan: Option<&JobPlan>,
        (base, candidate): (&str, &str),
        capability_env: &std::collections::BTreeMap<String, String>,
    ) -> Vec<CheckResultView> {
        let Some(plan) = plan else {
            return Vec::new();
        };
        let groups = crate::verify::demonstration_groups(&plan.demonstrations);
        if self.nested.is_some() || groups.is_empty() || !demo_runs_here() {
            return Vec::new();
        }
        let dir = job_dir.join("revert");
        let outcome = self.demonstration_runs(
            ctx,
            job_dir,
            &dir,
            &groups,
            (base, candidate),
            capability_env,
        );
        let _ = crate::pin::remove_tree(&dir);
        match outcome {
            Ok(records) => records,
            Err(e) => groups
                .iter()
                .map(|g| {
                    crate::verify::demonstration_not_run(
                        g,
                        &format!("the scratch copy could not be made or run: {e:#}"),
                    )
                })
                .collect(),
        }
    }

    fn demonstration_runs(
        &self,
        ctx: &RunContext<'_>,
        job_dir: &std::path::Path,
        dir: &std::path::Path,
        groups: &[crate::verify::DemonstrationGroup],
        (base, candidate): (&str, &str),
        capability_env: &std::collections::BTreeMap<String, String>,
    ) -> Result<Vec<CheckResultView>> {
        use crate::verify::{demonstration_not_run, demonstration_view};
        let jc = ctx.jc;
        let mut records: Vec<Option<CheckResultView>> = Vec::new();
        let mut to_run = Vec::new();
        for g in groups {
            if g.argv.is_none() {
                records.push(Some(demonstration_not_run(
                    g,
                    "not a plain cargo test command",
                )));
            } else if to_run.len() >= crate::verify::MAX_DEMONSTRATION_RUNS {
                records.push(Some(demonstration_not_run(
                    g,
                    "only the first few distinct commands are run",
                )));
            } else {
                to_run.push(g);
                records.push(None);
            }
        }
        if to_run.is_empty() {
            return Ok(records.into_iter().flatten().collect());
        }
        let (snapshot, _, _) =
            self.verifier_snapshot(dir, base, candidate, ctx.pin_dir.as_deref())?;
        let changed: Vec<(char, String)> = self
            .git
            .diff_tree(base, candidate)?
            .into_iter()
            .map(|e| (e.status, e.path))
            .collect();
        let revert = crate::verify::revert_plan(
            &changed,
            &|p| self.git.show_file(base, p).ok().flatten(),
            &|p| self.git.show_file(candidate, p).ok().flatten(),
        );
        for (path, bytes) in &revert.write {
            let target = scratch_target(snapshot.dir(), path)?;
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&target, bytes)?;
        }
        for path in &revert.remove {
            let target = scratch_target(snapshot.dir(), path)?;
            match std::fs::remove_file(&target) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(e.into()),
                _ => {}
            }
        }
        let env = athena_agent::env::build(
            &EnvSpec {
                agents: &jc.config.agents,
                job: ctx.id().as_str(),
                job_dir,
                snapshot: snapshot.dir(),
                side: Side::Verify,
                capability_env,
            },
            |key| std::env::var(key).ok(),
        )
        .map_err(anyhow::Error::msg)?;
        let registry = self.state.registry();
        let timeout = std::time::Duration::from_secs(jc.config.budget.check_timeout_s);
        let mut ran = to_run.into_iter();
        for slot in records.iter_mut().filter(|s| s.is_none()) {
            let Some(g) = ran.next() else { break };
            let spec = crate::capability::CheckSpec {
                name: format!("fails-without-{}", g.criteria.first().copied().unwrap_or(0)),
                run: mapped_argv(g.argv.as_deref().unwrap_or_default()),
                kind: athena_data::config::CheckKind::Test,
                skip_when_only: None,
            };
            let run = run_check(
                &spec,
                &CheckContext {
                    dir: snapshot.dir(),
                    env: &env,
                    timeout,
                    registry: &registry,
                    job: ctx.id().as_str(),
                    redactor: ctx.redactor,
                },
            )?;
            let mut record = demonstration_view(g, &run, &revert);
            // The record names the command as the plan wrote it, not the one a test seam mapped it to.
            if let Some(shown) = record.demonstration.as_mut() {
                shown.command = g.command.clone();
            }
            *slot = Some(record);
        }
        Ok(records.into_iter().flatten().collect())
    }

    fn baseline_checks(
        &self,
        ctx: &RunContext<'_>,
        job_dir: &std::path::Path,
        dir: &std::path::Path,
    ) -> Result<Vec<CheckRun>> {
        let jc = ctx.jc;
        if dir.exists() {
            crate::pin::remove_tree(dir)?;
        }
        let p = ctx.prediction.clone();
        let snapshot = Snapshot::create(&self.git, &p, &p, dir, SnapshotRepo::Base)?;
        if let Some(pin_dir) = &ctx.pin_dir {
            mount(pin_dir, snapshot.dir())?;
        }
        let env = athena_agent::env::build(
            &EnvSpec {
                agents: &jc.config.agents,
                job: ctx.id().as_str(),
                job_dir,
                snapshot: snapshot.dir(),
                side: Side::Verify,
                capability_env: &ctx.capability.env(jc),
            },
            |key| std::env::var(key).ok(),
        )
        .map_err(anyhow::Error::msg)?;
        let registry = self.state.registry();
        Ok(crate::checks::baseline_runs(
            &ctx.capability.checks(jc),
            &CheckContext {
                dir: snapshot.dir(),
                env: &env,
                timeout: std::time::Duration::from_secs(jc.config.budget.check_timeout_s),
                registry: &registry,
                job: ctx.id().as_str(),
                redactor: ctx.redactor,
            },
        ))
    }
}
