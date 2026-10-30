//! The checks stages of a job: the baseline run on the base commit and the executor that runs one routed
//! check in the verifier's snapshot (structure-pass §3.3, stages S6 and S7).

use super::*;

/// Runs one routed check in the verifier's snapshot, then puts the snapshot back as it was prepared.
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
}

impl crate::routing::Exec for SnapExec<'_> {
    fn exec(&self, spec: &crate::capability::CheckSpec) -> Result<CheckRun> {
        self.athena.refuse_if_stopped()?;
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
}

impl Athena {
    /// Runs the capability's configured checks on a fresh snapshot of the base commit and returns the
    /// baseline record. It never fails: a snapshot or environment error, or a nested run, gives a record
    /// with no checks and a note saying why. `before_invocation` is the sequence number the first
    /// producer invocation will get. Verification reads the result to mark pre-existing failures.
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
