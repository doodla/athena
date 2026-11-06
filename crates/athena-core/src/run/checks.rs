//! The checks stages of a job: the baseline run on the base commit and the executor that runs one routed
//! check in the verifier's snapshot (structure-pass §3.3, stages S6 and S7).

use super::*;

/// The decision log, and the list that links its decisions to the code they govern.
const DECISION_LOG: &str = "docs/decisions.md";
const DECISION_LINKS: &str = "docs/decision-links.toml";

/// Runs one routed check in the verifier's snapshot, then puts the snapshot back as it was prepared.
///
/// It is called only when a check really runs here, so it is also where the free-disk requirement of the
/// checks is applied: once, just before the first check starts. A CI result that stands in for the checks, a
/// resumed job and a skipped check never reach it. Low space comes back as a [`crate::disk::LowDisk`] error
/// that `check_and_verify` turns into a hold.
///
/// Each check also takes a build slot ([`crate::slots`], `[drain.pipeline] max_check_slots` of them) after
/// that gate and holds it until the snapshot is restored. The slot is a guard: every way out of `exec` (an
/// error, a panic, a normal return) drops it, and a process that dies loses its lock file, so no path
/// leaves a slot taken. A wait longer than `build_wait_s` comes back as a [`crate::disk::SlotWait`] error that
/// `check_and_verify` turns into a hold, and the time waited is stored on the check as `queued_s`. A request
/// that waited reads the free disk space again, since the earlier reading may be a long wait old.
///
/// The slot also owns a warm target directory, `<state>/build-slots/<n>/target`, that outlives every job. For
/// the length of one command the job's `CARGO_TARGET_DIR` is a link to it; afterwards the link goes and the
/// workspace crates' output is cleaned out of the directory, so third-party crates stay compiled for the next
/// job that gets the slot. A directory over `slot_max_gb` is deleted and rebuilt cold
/// (`docs/design/agent-and-build-slots.md` section 5.3).
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
    /// The build slots every check takes one of.
    pub(super) slots: crate::slots::SlotQueue,
    /// The job's own directory, inside which the build path is linked to the slot's directory.
    pub(super) job_dir: &'a std::path::Path,
    /// `[drain.pipeline] slot_max_gb`: the size above which a slot's target directory is deleted.
    pub(super) slot_max_gb: u64,
}

impl crate::routing::Exec for SnapExec<'_> {
    fn exec(&self, spec: &crate::capability::CheckSpec) -> Result<CheckRun> {
        self.athena.refuse_if_stopped()?;
        if !self.gated.replace(true)
            && let Some(low) = self.athena.disk_gate(self.min_free_gb, self.id, self.ui)
        {
            return Err(low.into());
        }
        // The slot lives to the end of this call, past the snapshot restore: it is the guard that frees it.
        let slot = self.slots.acquire(crate::slots::Class::Check, &|| {
            self.athena.refuse_if_stopped()
        })?;
        if slot.waited()
            && let Some(low) = self.athena.disk_gate(self.min_free_gb, self.id, self.ui)
        {
            return Err(low.into());
        }
        // The slot's warm target directory is linked at the job's build path for this one command only.
        let link = self.link_slot_target(&slot);
        #[cfg(test)]
        let seamed = check_seam::current().map(|fake| fake(spec));
        #[cfg(not(test))]
        let seamed: Option<CheckRun> = None;
        let ran = match seamed {
            Some(run) => Ok(run),
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
            ),
        };
        // Every way out of the command: unlink, then drop the workspace crates' output from the slot.
        if let Some(link) = link {
            drop(link);
            self.clean_slot_target(&slot);
        }
        let mut run = ran?;
        self.cleanup
            .borrow_mut()
            .extend(self.snapshot.restore_to(self.prepared)?);
        run.queued_s = Some(slot.queued().as_secs_f64());
        Ok(run)
    }

    fn redact(&self, text: &str) -> Result<String> {
        Ok(self.redactor.redact(text)?)
    }
}

/// The bytes `slot_max_gb` gigabytes come to. A test build can shrink the gigabyte ([`gb_seam`]) so a test
/// shows the deletion without writing one.
pub(super) fn slot_limit_bytes(slot_max_gb: u64) -> u64 {
    #[cfg(test)]
    let gb = gb_seam::current().unwrap_or(crate::disk::GB);
    #[cfg(not(test))]
    let gb = crate::disk::GB;
    slot_max_gb.saturating_mul(gb)
}

/// The size of a gigabyte in a test, per thread, for [`slot_limit_bytes`].
#[cfg(test)]
pub(crate) mod gb_seam {
    use std::cell::Cell;

    thread_local! {
        static GB: Cell<Option<u64>> = const { Cell::new(None) };
    }

    /// Restores the real gigabyte when dropped.
    #[derive(Debug)]
    pub struct Guard;

    impl Drop for Guard {
        fn drop(&mut self) {
            GB.with(|g| g.set(None));
        }
    }

    pub fn install(bytes: u64) -> Guard {
        GB.with(|g| g.set(Some(bytes)));
        Guard
    }

    pub fn current() -> Option<u64> {
        GB.with(Cell::get)
    }
}

impl SnapExec<'_> {
    /// Links the job's build path (`CARGO_TARGET_DIR`) to `slot`'s warm target directory. A slot directory
    /// over `slot_max_gb` is deleted and made again empty first. The directory outlives the job; only the
    /// link goes when the returned value is dropped. A failure here is a warning line and the command builds
    /// in the job's own directory, cold, as it did before slots owned directories. No build path in the
    /// environment, or none inside the job's directory, links nothing.
    fn link_slot_target(&self, slot: &crate::slots::Slot) -> Option<athena_agent::env::BuildLink> {
        let path = self.env.get("CARGO_TARGET_DIR")?;
        let limit = slot_limit_bytes(self.slot_max_gb);
        let state = &self.athena.state;
        let prepared = match state.prepare_slot_target(slot.index(), limit) {
            Ok(prepared) => prepared,
            Err(e) => {
                self.ui.line(&format!(
                    "warning: build slot {} has no usable target directory: {e:#}",
                    slot.index()
                ));
                return None;
            }
        };
        if let Some(bytes) = prepared.deleted_bytes {
            self.ui.line(&format!(
                "disk       build slot {} target directory held {}, over its {} GB limit: deleted, the next build is cold",
                slot.index(),
                crate::state::format_bytes(bytes),
                self.slot_max_gb
            ));
        }
        match athena_agent::env::BuildLink::create(
            self.job_dir,
            std::path::Path::new(path),
            &prepared.dir,
        ) {
            Ok(link) => Some(link),
            Err(e) => {
                self.ui
                    .line(&format!("warning: could not link the build slot: {e}"));
                None
            }
        }
    }

    /// Removes the workspace crates' output from `slot`'s target directory, keeping third-party output
    /// ([`crate::slots::clean_workspace_output`]). When that fails the directory is deleted whole instead, so
    /// the next job never builds on output it cannot trust.
    fn clean_slot_target(&self, slot: &crate::slots::Slot) {
        let target = self.athena.state.slot_target(slot.index());
        if let Err(e) = crate::slots::clean_workspace_output(self.snapshot.dir(), &target) {
            self.ui.line(&format!(
                "warning: could not clean build slot {}, so its target directory is deleted: {e:#}",
                slot.index()
            ));
            if let Err(e) = self.athena.state.reset_slot_target(slot.index()) {
                self.ui.line(&format!(
                    "warning: could not delete build slot {}'s target directory: {e:#}",
                    slot.index()
                ));
            }
        }
    }
}

/// How many commits of records only the baseline lookup steps over to find the commit the host has.
const MAX_RECORDS_ONLY_COMMITS: usize = 20;

/// An executor that runs nothing. The baseline lookup uses the call the candidate's checks use, which runs
/// the checks locally when no CI result is usable; here that must not happen, so the run fails and the
/// caller reads the failure as "no result".
struct RefuseExec;

impl crate::routing::Exec for RefuseExec {
    fn exec(&self, spec: &crate::capability::CheckSpec) -> Result<CheckRun> {
        bail!("the baseline lookup does not run {}", spec.name)
    }

    fn redact(&self, _: &str) -> Result<String> {
        bail!("the baseline lookup records no log text")
    }
}

/// A CI source for the baseline lookup. It forwards the run and never reads a failing job's log: a red run
/// is not reused as a baseline, so its log is not needed.
///
/// It also decides what is worth waiting for. The read loop ([`crate::ci_result::resolve`]) keeps reading
/// while a run is unfinished or absent, up to its bound. Here only a run that has been seen in progress is
/// waited for: until then an absent run is reported as an error, which the loop reads as "no result" and
/// stops at, so a run that never appeared does not hold the baseline for the whole bound.
struct WaitsWhileRunning<'a> {
    inner: &'a dyn crate::ci_result::CiSource,
    /// Whether a read has shown the run for the asked commit in progress.
    running: std::cell::Cell<bool>,
}

impl<'a> WaitsWhileRunning<'a> {
    fn new(inner: &'a dyn crate::ci_result::CiSource) -> Self {
        WaitsWhileRunning {
            inner,
            running: std::cell::Cell::new(false),
        }
    }
}

impl crate::ci_result::CiSource for WaitsWhileRunning<'_> {
    fn fetch(&self, sha: &str) -> Result<Option<crate::ci_result::CiRun>> {
        let read = self.inner.fetch(sha)?;
        match &read {
            Some(run) if run.sha == sha && !run.finished => self.running.set(true),
            None if !self.running.get() => bail!("there is no run for {sha} to wait for"),
            _ => {}
        }
        Ok(read)
    }
}

/// The job a base commit landed, as its closing `verification.json` says.
struct LandedJob {
    job: JobId,
    /// The candidate commit the job verified: the commit its CI run was for.
    candidate: String,
    /// The fingerprint of that candidate's governed content, as verified.
    fingerprint: String,
    /// Every path the landing changed.
    changed: Vec<String>,
}

/// The request job a path names when it is `<job dir>/verification.json`.
fn verification_job(path: &str) -> Option<JobId> {
    let (dir, name) = path.rsplit_once('/')?;
    if name != "verification.json" {
        return None;
    }
    let job = JobId::parse(dir.rsplit('/').next()?).ok()?;
    (job.is_request() && dir == job.dir()).then_some(job)
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

    /// Clears the seam when dropped. This crate's tests and, through `test-support`, other crates' tests
    /// install a mapper; a build where none did runs no demonstration.
    #[derive(Debug)]
    pub struct Guard;

    impl Drop for Guard {
        fn drop(&mut self) {
            SEAM.with(|s| *s.borrow_mut() = None);
        }
    }

    pub fn install(map: Mapper) -> Guard {
        SEAM.with(|s| *s.borrow_mut() = Some(map));
        Guard
    }

    pub fn current() -> Option<Mapper> {
        SEAM.with(|s| s.borrow().clone())
    }
}

impl Athena {
    /// The decisions of `docs/decisions.md` that the change reverses or may reverse and does not reconcile:
    /// the closing reason when there are any, `None` when the change is fine. The base's `[[governs]]` list in
    /// `docs/decision-links.toml` names each decision and the paths it governs; a change that touches one of
    /// them must add a log entry that amends or supersedes the decision (or says it was checked and stands)
    /// with a matching `[[link]]` ([`athena_data::decision_links::unreconciled`]). It runs before the pull
    /// request, the checks and the verifier, so a reversal is found at the producer's step and not at the
    /// final verdict. A base with no list governs nothing; a base list that cannot be read is a warning line
    /// and the check is skipped, so a change that repairs the list can pass.
    pub(super) fn unreconciled_decisions(
        &self,
        ctx: &RunContext<'_>,
        base: &str,
        candidate: &str,
        changed: &[String],
    ) -> Option<String> {
        use athena_data::decision_links::{GATE_REASON_PREFIX, unreconciled};
        let text = |commit: &str, path: &str| -> String {
            self.git
                .show_file(commit, path)
                .ok()
                .flatten()
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
                .unwrap_or_default()
        };
        let base_links = text(base, DECISION_LINKS);
        if base_links.is_empty() {
            return None;
        }
        let found = match unreconciled(
            &base_links,
            &text(candidate, DECISION_LINKS),
            &text(base, DECISION_LOG),
            &text(candidate, DECISION_LOG),
            changed,
        ) {
            Ok(found) => found,
            Err(why) => {
                ctx.ui.line(&format!(
                    "warning: {DECISION_LINKS} could not be read, so decisions are not checked: {why}"
                ));
                return None;
            }
        };
        if found.is_empty() {
            return None;
        }
        let each: Vec<String> = found
            .iter()
            .map(|u| format!("{} ({})", u.decision, u.touched.join(", ")))
            .collect();
        Some(format!(
            "{GATE_REASON_PREFIX} the change touches code that {DECISION_LOG} records decisions for: {}. \
             Add an entry to {DECISION_LOG} that amends or supersedes each decision (or says it was checked \
             and stands), and a matching [[link]] in {DECISION_LINKS} whose older is the decision and whose \
             newer is the new entry.",
            each.join("; ")
        ))
    }

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

    /// Takes the baseline: the capability's configured checks on the base commit, as a record.
    ///
    /// When a green GitHub run exists for the base's governed content, that result stands in for the checks
    /// and nothing runs here (see [`Athena::reused_baseline`]); the record then names the run in
    /// `reused_from` and every check is marked reused. When the default branch's run for that content is
    /// still in progress, this waits for it (up to the `ci_wait_s` bound). Otherwise the checks run on a fresh snapshot of the
    /// base commit, as they always did, and a `ci         baseline runs locally: <reason>` line says why.
    ///
    /// It never fails: a snapshot or environment error, or a nested run, gives a record with no checks and a
    /// note saying why, and a failed lookup of the CI result only means the local run. `before_invocation` is
    /// the sequence number the first producer invocation will get. Verification reads the result to mark
    /// pre-existing failures. The checks run through `run_check`, as the candidate's do, so each failed run
    /// records the names of its failing tests and verification compares test by test, not by check name
    /// alone.
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
            reused_from: None,
        };
        if self.nested.is_some() {
            baseline.note =
                Some("not run: a nested run skips checks; the outer job runs its own".into());
            return baseline;
        }
        match self.reused_baseline(ctx) {
            Ok((checks, address)) => {
                ctx.ui.line(&format!(
                    "ci         baseline reused from {address} for {}",
                    ctx.prediction
                ));
                baseline.checks = checks;
                baseline.reused_from = Some(address);
                return baseline;
            }
            Err(Some(reason)) => ctx
                .ui
                .line(&format!("ci         baseline runs locally: {reason}")),
            Err(None) => {}
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
    /// to the verifier under the checks, plus one stored [`CheckRun`] for each run that happened
    /// ([`crate::verify::revert_record`]), for the verification record's checks. The verifier gets no
    /// producer report and cannot change files, so it could never see this run otherwise; the stored record
    /// is what lets the verdict count the run as seen running (D43).
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
    ) -> (Vec<CheckResultView>, Vec<CheckRun>) {
        let Some(plan) = plan else {
            return (Vec::new(), Vec::new());
        };
        let groups = crate::verify::demonstration_groups(&plan.demonstrations);
        if self.nested.is_some() || groups.is_empty() || !demo_runs_here() {
            return (Vec::new(), Vec::new());
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
            Err(e) => (
                groups
                    .iter()
                    .map(|g| {
                        crate::verify::demonstration_not_run(
                            g,
                            &format!("the scratch copy could not be made or run: {e:#}"),
                        )
                    })
                    .collect(),
                Vec::new(),
            ),
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
    ) -> Result<(Vec<CheckResultView>, Vec<CheckRun>)> {
        use crate::verify::{
            demonstration_name, demonstration_not_run, demonstration_view, revert_record,
        };
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
            return Ok((records.into_iter().flatten().collect(), Vec::new()));
        }
        let mut stored: Vec<CheckRun> = Vec::new();
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
                name: demonstration_name(g),
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
            stored.push(revert_record(g, &run));
        }
        Ok((records.into_iter().flatten().collect(), stored))
    }

    /// Looks for a green GitHub run that stands in for the baseline, so the base commit is not tested again
    /// on this machine when its governed content was just tested there. `Ok` holds the reused checks and the
    /// run's address. `Err` is why nothing was reused: `Some(reason)` for the output line, `None` when no
    /// lookup was possible (no GitHub origin), which prints nothing. It runs no check. The landed
    /// candidate's run (1 below) is finished by the time its job has landed, so it is read once. The run for
    /// the default branch's tip (2 below) is often still in progress right after a landing, so that run is
    /// waited for: read every [`crate::routing::CI_POLL`] (20 s), for at most the `ci_wait_s` of the
    /// `[budget]` (the bound the candidate's own checks wait by), and reused if it passes. A run that is
    /// missing, failed, cancelled, or still in progress when the wait runs out gives the local run.
    ///
    /// Both lookups go through the call the candidate's checks use ([`crate::routing::run_routed_with_record`],
    /// which calls [`crate::ci_result::resolve`]), so a run counts only when it is finished, green, covers
    /// `fmt`, `clippy` and `test`, and is for the exact commit asked for:
    ///
    /// The commit looked up is the base's content commit ([`Athena::content_commit`]): the base, or the
    /// commit under the job's own records commits, which is the one the host has.
    ///
    /// 1. The landed candidate's run. The content commit is the squash of a landed job, which carries that
    ///    job's `verification.json`. When the fingerprint it recorded for the candidate equals the base's own
    ///    ([`fingerprint`] of the base under its own rules, the value a candidate record keeps as
    ///    `base_fingerprint`), the base has the governed content that run tested. The run is read on the
    ///    job's CI branch for the candidate commit. The squash's changed paths go to `resolve`, so a landing
    ///    that edited a workflow file, which controlled the checker, is not trusted.
    /// 2. The push run for the tip of the repository's default branch (the local `origin/<branch>` ref,
    ///    the branch named by `origin/HEAD`), when the tip's governed-content fingerprint equals the
    ///    base's. The host holds a different (squash) commit for the base's content, so the lookup is by
    ///    fingerprint, not by the base's own commit. A tip with other content reads nothing.
    ///
    /// The `config` route is used: it never skips a check by path, so a reused baseline holds the same
    /// checks a local one does.
    fn reused_baseline(
        &self,
        ctx: &RunContext<'_>,
    ) -> std::result::Result<(Vec<CheckRun>, String), Option<String>> {
        if !self.has_github_origin() {
            return Err(None);
        }
        let route = crate::routing::route(Some("config"), ctx.capability.checks(ctx.jc), &[]);
        if !route.reuse {
            return Err(Some(
                "the configured checks do not cover fmt, clippy and test".into(),
            ));
        }
        let base = ctx.prediction.as_str();
        let own = match fingerprint(&self.git, base, base) {
            Ok(own) => own,
            Err(e) => {
                return Err(Some(format!(
                    "the base's fingerprint could not be read: {e:#}"
                )));
            }
        };
        let content = match self.content_commit(base) {
            Ok(content) => content,
            Err(e) => {
                return Err(Some(format!("the base's history could not be read: {e:#}")));
            }
        };
        let mut why: Vec<String> = Vec::new();
        match self.landed_job(&content) {
            Ok(Some(landed)) if landed.fingerprint == own => {
                let branch = crate::gh::ci_branch(&landed.job);
                match self.green_run(&route, &landed.candidate, &landed.changed, &branch, 0) {
                    Ok(hit) => return Ok(hit),
                    Err(reason) => why.push(format!("landed job {}: {reason}", landed.job)),
                }
            }
            Ok(Some(landed)) => why.push(format!(
                "landed job {}: the fingerprint it verified differs from the base's",
                landed.job
            )),
            Ok(None) => why.push("the base is not the landing of one job".into()),
            Err(e) => why.push(format!("the base's landing could not be read: {e:#}")),
        }
        match crate::reconcile::local_default_branch(&self.git, "origin") {
            // The default branch holds its own commit for the base's content (a squash), so its tip is
            // looked up, not the base's commit. Its run is taken only when the tip has the base's
            // governed content.
            Some(branch) => {
                let tracking = format!("refs/remotes/origin/{branch}");
                match self.git.rev_parse(&tracking) {
                    Ok(Some(tip)) if fingerprint(&self.git, &tip, &tip).ok() == Some(own) => {
                        let wait_s = ctx.jc.config.budget.ci_wait_s;
                        match self.green_run(&route, &tip, &[], &branch, wait_s) {
                            Ok(hit) => return Ok(hit),
                            Err(reason) => why.push(format!("{branch}: {reason}")),
                        }
                    }
                    Ok(Some(tip)) => why.push(format!(
                        "{branch}: its tip {tip} has other governed content than the base"
                    )),
                    Ok(None) | Err(_) => why.push(format!(
                        "{branch}: its tip is not known here, so its run is not read"
                    )),
                }
            }
            None => why.push("origin/HEAD is not set, so the default branch is not read".into()),
        }
        Err(Some(why.join("; ")))
    }

    /// The commit that carries `base`'s governed content on the host: `base` itself, or the nearest ancestor
    /// reached by stepping over commits that change only files under `records/`. A job's own commits of
    /// records sit on the branch above the last landing (the base is one of them), and the host has never
    /// seen them. The walk stops at a commit with other than one parent, and after
    /// [`MAX_RECORDS_ONLY_COMMITS`] steps.
    fn content_commit(&self, base: &str) -> Result<String> {
        let mut commit = base.to_string();
        for _ in 0..MAX_RECORDS_ONLY_COMMITS {
            let parents = self.git.parents(&commit)?;
            let [parent] = parents.as_slice() else {
                break;
            };
            let diff = self.git.diff_tree(parent, &commit)?;
            if !diff.iter().all(|e| e.path.starts_with("records/")) {
                break;
            }
            commit = parent.clone();
        }
        Ok(commit)
    }

    /// The closing records of the job a base commit landed: the one `verification.json` the commit adds
    /// under a request job's directory. `None` for a commit with other parents than one, or that adds no
    /// such file or more than one, which is not the landing of a single job.
    fn landed_job(&self, base: &str) -> Result<Option<LandedJob>> {
        let parents = self.git.parents(base)?;
        let [parent] = parents.as_slice() else {
            return Ok(None);
        };
        let entries = self.git.diff_tree(parent, base)?;
        let closing: Vec<&str> = entries
            .iter()
            .filter(|e| e.status == 'A' && verification_job(&e.path).is_some())
            .map(|e| e.path.as_str())
            .collect();
        let [path] = closing.as_slice() else {
            return Ok(None);
        };
        let Some(job) = verification_job(path) else {
            return Ok(None);
        };
        let Some(bytes) = self.git.show_file(base, path)? else {
            return Ok(None);
        };
        let AnyRecord::Verification(v) = AnyRecord::parse(path, &bytes)? else {
            return Ok(None);
        };
        Ok(Some(LandedJob {
            job,
            candidate: v.candidate_commit,
            fingerprint: v.verified_fingerprint,
            changed: entries.iter().map(|e| e.path.clone()).collect(),
        }))
    }

    /// The checks from a green run for `sha` on `branch`, and the run's address; or why there is none.
    ///
    /// With `wait_s` of 0 it makes one read. With more, a run that a read shows in progress is waited for:
    /// it is read again every [`crate::routing::CI_POLL`] (20 s) until it finishes, for at most
    /// [`crate::routing::ci_polls`]`(wait_s)` reads, so `wait_s` is the bound. A run that is not there on
    /// the first read, a failed or cancelled one, and one still in progress when the reads run out are not
    /// waited for any longer. A failing check executor stands in for the local run, so a missing,
    /// unfinished, red or untrusted result is an `Err` and nothing runs here.
    fn green_run(
        &self,
        route: &crate::routing::Route,
        sha: &str,
        changed: &[String],
        branch: &str,
        wait_s: u64,
    ) -> std::result::Result<(Vec<CheckRun>, String), String> {
        let routed = self.with_ci_source_on(branch, wait_s, |ci, wait| {
            crate::routing::run_routed_with_record(
                route,
                sha,
                changed,
                &WaitsWhileRunning::new(ci),
                &RefuseExec,
                wait,
            )
        });
        match routed {
            Ok((runs, Some(record)))
                if record.source == crate::ci_result::Source::Ci
                    && record.all_passed()
                    && runs.iter().all(|r| r.passed) =>
            {
                Ok((
                    runs,
                    record.run_address.unwrap_or_else(|| "the CI run".into()),
                ))
            }
            Ok((_, Some(record))) => Err(format!(
                "the CI run {} for {sha} did not pass",
                record.run_address.as_deref().unwrap_or("")
            )),
            Ok((_, None)) | Err(_) => {
                let workflow = changed
                    .iter()
                    .any(|p| p.starts_with(crate::ci_result::WORKFLOW_DIR));
                Err(if workflow {
                    format!("{sha} changed a workflow file, so its CI run is not trusted")
                } else {
                    format!("no usable CI result for {sha} on {branch}")
                })
            }
        }
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
