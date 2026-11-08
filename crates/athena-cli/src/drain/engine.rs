//! The drain engine's shared state and its entry points: `drain`, `drain_concurrent`, `plan` and `follow_restart`.

use super::*;

/// Everything the workers of one drain share, behind one lock. A lock is never held across a call into
/// [`Steps`], so a slow model call never blocks another worker's bookkeeping.
pub(super) struct State {
    pub(super) budget: Budget,
    pub(super) jobs: Vec<JobLine>,
    pub(super) started: u32,
    pub(super) landed: Vec<String>,
    /// Landings in this run that changed a path in [`OUTCOME_PATHS`].
    pub(super) outcome_landings: u32,
    pub(super) rating: Option<RatingLine>,
    pub(super) held_back: Option<String>,
    pub(super) notes: Vec<String>,
    /// Set by a landing that touched [`BUILD_PATHS`] and cleared by the rebuild that follows it.
    pub(super) rebuild_due: bool,
    /// The build output of a failed rebuild.
    pub(super) rebuild_output: Option<String>,
    /// Why the configuration could not be read after a landing ([`Steps::config_after_landing`]). This is the
    /// one fact the rating and the report key on, whatever the stored stop reason is (the stop keeps only the
    /// first reason, so another worker's earlier stop must not hide it). Cleared by a rebuild that runs after
    /// the landing, because the installed build then reads the landed config.
    pub(super) unreadable_config: Option<String>,
    /// What the planning pass spent reading an item, charged to that item's job line when it runs.
    pub(super) read_cost: std::collections::BTreeMap<String, f64>,
    /// Why the run is ending, once any worker has found a reason (the first reason found stands). No worker
    /// starts a job once this is set; jobs already started are finished.
    pub(super) stop: Option<Stop>,
    // Read-ahead. All of this stays empty unless `drain_concurrent` runs and `Steps::read_ahead` is above 0.
    /// Readings made ahead and not yet claimed by a job that starts.
    pub(super) ahead: BTreeMap<String, Ahead>,
    /// Items the read-ahead is reading or planning right now.
    pub(super) reading: BTreeSet<String>,
    /// Items the read-ahead has already tried once, so a dropped reading is not made again.
    pub(super) pumped: BTreeSet<String>,
    /// Items that have started a job, so the money their job needs is held.
    pub(super) begun: BTreeSet<String>,
    /// Items whose work is over (landed, failed or left queued).
    pub(super) done: BTreeSet<String>,
    /// The paths of each landing in this run, in order.
    pub(super) landings: Vec<Vec<String>>,
    /// The paths each item's plan entry says it touches.
    pub(super) touches: BTreeMap<String, Vec<String>>,
    /// The index of the batch that is running.
    pub(super) batch: usize,
    /// The batches are over (or the run is ending): the read-ahead stops.
    pub(super) pump_done: bool,
    /// The read-ahead has ended, so no worker waits on a reading that will not finish.
    pub(super) pump_gone: bool,
    /// Items this run has handed to a worker; a new plan never offers them again.
    pub(super) tried: BTreeSet<String>,
    /// How many items this run has handed to a worker.
    pub(super) dispatched: u32,
    /// How many items in a row the run has handed to a worker that were not improvement requests: the count
    /// the plan's guaranteed turn is measured from ([`athena_core::plan::Improvement::since_turn`]).
    pub(super) since_turn: usize,
    /// The improvement requests the last plan knew ([`Steps::improvement`]).
    pub(super) improving: BTreeSet<String>,
    /// The improvement requests this run handed to a worker in the place the plan's guaranteed turn gave them.
    pub(super) improvement_turns: Vec<ImprovementTaken>,
    /// Items that have a "passed over" line, so none gets two.
    pub(super) passed_over: BTreeSet<String>,
    /// Items the last plan held back because their line is paused (or they wait on one that is), with the
    /// reason. Replaced by each plan: a lifted pause frees the item.
    pub(super) pause_held: BTreeMap<String, String>,
    /// What each new plan of this run needs to remember from the ones before.
    pub(super) carry: Carry,
    /// Workers whose run passed and whose job is not landed yet, in the order the runs passed.
    pub(super) queue: LandingQueue,
    /// The rebases of this run's landings that moved the branch, and how many of them needed a re-check.
    pub(super) rebases: RebaseTally,
    /// Jobs whose checks after a rebase are still owed: decided at the rebase, cleared when the re-check
    /// passes. The landing retry asks `Steps::rebase` again, and a branch already moved answers `Unchanged`,
    /// so without this a candidate whose re-check failed would land unchecked on the second try.
    pub(super) owing: BTreeSet<String>,
}

/// What a drain keeps from one plan to the next, merged into each fresh queue view by `Drain::prepare`. A
/// fresh view may not list a job the drain approved, and must not make it read or pay for an item twice.
#[derive(Clone, Default)]
pub(super) struct Carry {
    /// Jobs approved in this run, with their plan entries.
    pub(super) approved: Vec<(Item, Option<PlanEntry>)>,
    /// Items already read by the planning pass: whether the item counts as read, and its entry after reading.
    pub(super) prepared: BTreeMap<String, (bool, PlanEntry)>,
    /// Items the planning pass refused after trying to read them.
    pub(super) forced: Vec<Refusal>,
}

/// Sets a flag of the shared state and wakes every waiter when it is dropped, so it happens on every way out
/// of a scope, a panic included.
pub(super) struct FlagOnDrop<'a> {
    pub(super) state: &'a Mutex<State>,
    pub(super) wake: &'a Condvar,
    pub(super) set: fn(&mut State),
}

impl Drop for FlagOnDrop<'_> {
    fn drop(&mut self) {
        (self.set)(&mut self.state.lock().unwrap_or_else(PoisonError::into_inner));
        self.wake.notify_all();
    }
}

/// The first-in, first-out line of workers whose run passed and whose job is not yet landed. It is a plain
/// value: it holds no lock and starts no thread, so it is driven by the state lock of the drain and by unit
/// tests alike. The line is ordered by `join`, which a worker calls when its run passes, so the order is the
/// order runs passed.
#[derive(Default)]
pub(super) struct LandingQueue {
    /// The ticket the next join gets.
    pub(super) next: u64,
    /// The tickets in line, the head first.
    pub(super) line: VecDeque<u64>,
}

impl LandingQueue {
    /// Takes a place at the back of the line and returns its ticket.
    pub(super) fn join(&mut self) -> u64 {
        let ticket = self.next;
        self.next += 1;
        self.line.push_back(ticket);
        ticket
    }

    /// Whether `ticket` is first in line.
    pub(super) fn is_head(&self, ticket: u64) -> bool {
        self.line.front() == Some(&ticket)
    }

    /// Gives up the place of `ticket`, wherever it is in the line. A ticket that is not in line is ignored.
    pub(super) fn leave(&mut self, ticket: u64) {
        self.line.retain(|t| *t != ticket);
    }
}

/// A place in the [`LandingQueue`] of a drain. Dropping it gives the place up and wakes every waiter, so the
/// next in line goes on whichever way the holder leaves its scope, a panic included.
pub(super) struct LandingSlot<'a> {
    pub(super) state: &'a Mutex<State>,
    pub(super) wake: &'a Condvar,
    pub(super) ticket: u64,
}

impl Drop for LandingSlot<'_> {
    fn drop(&mut self) {
        self.state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .queue
            .leave(self.ticket);
        self.wake.notify_all();
    }
}

/// What the read-ahead does next.
pub(super) enum Pick {
    /// Nothing is left to read ahead.
    Done,
    /// Something could be read, but not yet.
    Wait,
    /// Read the item at this index of the upcoming items.
    Start(usize),
}

pub(super) struct Drain<'a, S: ?Sized, U: ?Sized> {
    pub(super) steps: &'a S,
    pub(super) ui: &'a U,
    pub(super) caps: Caps,
    pub(super) limits: Limits,
    pub(super) state: Mutex<State>,
    /// Held while a job is landed and while the page is rated: landings and ratings go one at a time even
    /// when jobs run side by side, because each pushes a branch and rates the page as it then stands. Workers
    /// join the landing queue when their run passes, and only the head of the queue takes this lock.
    pub(super) landing: Mutex<()>,
    /// Woken when a reading made ahead is stored, a job ends or a landing is recorded.
    pub(super) wake: Condvar,
}

pub(super) enum ItemEnd {
    Landed,
    /// The item failed twice on a line that is neither paused nor flagged. It is passed over for the rest of
    /// the batch, left for the scope guard and Hestia, and the next item runs. Only an environment cause of
    /// the failure also ends the run ([`Stop::Environment`]).
    Failed,
    Budget,
    JobLimit,
    /// A stop was asked for and no further job was started.
    Stop,
    /// Another worker had already ended the run, so nothing was started.
    Idle,
    /// The item's reading was made again with a different scope; it stays queued, and nothing else stops.
    NeedsAccepting,
    /// Dispatch was held for lack of free disk space, or the job's budget did not cover the whole path; the
    /// item stays queued, and nothing else stops.
    Held,
    /// The item's reader gave no reading twice. It is skipped with its reason and the next item runs; this
    /// is not a failed run, so it does not stop the drain.
    Skipped,
    /// The job passed and its pull request is open, with checks that had not finished when the land step's
    /// poll ended. (Checks that finish green during the poll merge in that same locked step and are not
    /// this.) It is not retried and is not a failure: the next item runs, and the job merges on a later
    /// `athena land --pr`.
    Waiting,
    /// The item's line is paused by the scope guard, with the reason. It is passed over before it is read or
    /// run: not a failed attempt. The batch goes on to the next item.
    Paused(String),
    /// The item failed twice on a line the scope guard has flagged but not paused, with the flag reason. It is
    /// passed over for the rest of the batch, and the batch goes on to the next item.
    Flagged(String),
}

/// What one call of [`Drain::land`] came to.
pub(super) enum LandStep {
    Landed,
    Waiting,
    NotLanded,
    /// The checks owed after a rebase could not start: their cap does not fit what is left of the budget.
    Budget,
}

/// The outcome of an item held because its job's budget does not cover the whole path still to run; also the
/// start of the reason a run held for it gives.
pub(super) const BUDGET_SHORT: &str = "budget short";

pub(super) enum RunStep {
    /// The job passed; its line is at this index.
    Passed(String, usize),
    /// The job did not pass. `retry` is whether it ended rejected or failed, the only states `athena retry`
    /// accepts; the next attempt then reads it again. `read` is whether the job was read, so that when it
    /// is not retried its next attempt is a run of that reading and not a second reading.
    Failed {
        job: String,
        retry: bool,
        read: bool,
    },
    /// The job was read again with a different scope and waits for a person to accept it. It is not a
    /// failure and not tried again.
    NeedsAccepting,
    /// The job was held for lack of free disk space or of budget. It is not a failure and not tried again in
    /// this run.
    Held,
    /// A job's reader gave no reading, with this reason. The job is recorded without a reading; its next
    /// attempt reads it again. It is not a failure of a run. `retried` is the new job when the read was a
    /// retry of an interrupted job: that retry job exists, so the next attempt reads it and does not retry
    /// the interrupted job a second time.
    Unread {
        reason: String,
        retried: Option<String>,
    },
    Budget,
    JobLimit,
    /// Another worker had already ended the run.
    Ended,
}

/// `text` on one line, for a line of the drain's log.
pub(super) fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Works the queue within `limits`, one job at a time. See the module text for the rules.
pub fn drain(steps: &dyn Steps, limits: Limits, ui: &dyn Ui) -> anyhow::Result<Report> {
    let caps = steps.caps()?;
    let d = Drain::new(steps, ui, caps, limits);
    drain_with(&d, steps, true, |d, wave, _| d.run_serial(wave))
}

/// Works the queue like [`drain`], running up to `max_concurrency` jobs at once. The jobs of one group of the
/// plan (no dependency between them, disjoint touched paths) run side by side; the next group starts when
/// the group before it is done, so an item that depends on another starts only after it has finished. Landing
/// and rating stay one at a time. With `max_concurrency` 1 it runs like [`drain`], one job at a time, except
/// that it reads (and plans) the next items while the job runs when [`Steps::read_ahead`] is above 0. A worker
/// that panics ends the drain with that panic once the other workers have finished their jobs; its item is
/// not isolated.
pub fn drain_concurrent(
    steps: &(dyn Steps + Sync),
    limits: Limits,
    ui: &(dyn Ui + Sync),
    max_concurrency: u32,
) -> anyhow::Result<Report> {
    let caps = steps.caps()?;
    let d = Drain::new(steps, ui, caps, limits);
    let workers = max_concurrency.max(1) as usize;
    drain_with(&d, steps, workers == 1, move |d, wave, later| {
        if workers == 1 && d.steps.read_ahead() == 0 {
            d.run_serial(wave);
        } else {
            d.run_parallel(wave, later, workers);
        }
    })
}

/// The batches to work from `plan`, in the order to work them: items with no plan entry first, each alone,
/// then the plan's groups in the plan's own order, without what this run has `tried`. It only filters; the
/// order is the plan's, built by [`Plan::build_with`] in `Drain::prepare` (the function `athena plan` calls).
pub(super) fn batches_for(
    items: &[Item],
    entries: &[PlanEntry],
    plan: &Plan,
    tried: &BTreeSet<String>,
) -> Vec<Vec<Item>> {
    let mut batches: Vec<Vec<Item>> = items
        .iter()
        .filter(|i| {
            !tried.contains(&i.job)
                && !entries.iter().any(|e| e.id == i.job)
                // An item the plan held back (a paused line) is not worked, entry or not.
                && !plan.refused.iter().any(|r| r.id == i.job)
        })
        .cloned()
        .map(|i| vec![i])
        .collect();
    batches.extend(
        plan.groups
            .iter()
            .map(|group| {
                group
                    .iter()
                    .filter(|id| !tried.contains(*id))
                    .filter_map(|id| items.iter().find(|i| &i.job == id).cloned())
                    .collect::<Vec<Item>>()
            })
            .filter(|group| !group.is_empty()),
    );
    batches
}

pub(super) fn drain_with<'a, S: Steps + ?Sized, U: Ui + ?Sized>(
    d: &Drain<'a, S, U>,
    steps: &S,
    one_at_a_time: bool,
    mut run: impl FnMut(&Drain<'a, S, U>, Vec<Vec<Item>>, Vec<Vec<Item>>),
) -> anyhow::Result<Report> {
    let limits = d.limits;
    // Interrupted jobs' build directories first: they are the disk a run would otherwise need.
    steps.reclaim_interrupted();
    // Before the queue is read, so a request the backfill queues is worked in this drain.
    for line in steps.review_passes() {
        d.ui.line(&line);
    }
    // What an earlier run left in the in-flight table is reported and left alone: recovery is later work.
    for line in steps.in_flight() {
        d.note(line);
    }
    let view = steps.queue()?;
    let previous = steps.previous_rating();
    let mut view = view;
    let (approved, left) = d.approve_all(std::mem::take(&mut view.approvable));
    // Jobs already approved come first, then the ones approved now, then queued requests. This is only their
    // age order (the tiebreak of the plan); it does not put them ahead of anything.
    let (read, unread): (Vec<Item>, Vec<Item>) = view.items.drain(..).partition(|i| i.read);
    // A job approved now enters the plan like one approved earlier. Root cause of the starvation of urgent
    // requests: it had no plan entry, so it kept its place ahead of every planned item and used the job slots.
    let any_approved = !approved.is_empty();
    for item in &approved {
        let entry = steps.plan_entry(&item.job);
        match &entry {
            Some(entry) => view.plan.push(entry.clone()),
            None => d.note(format!(
                "approved job {} has no plan entry, so it cannot be ordered with the rest",
                item.job
            )),
        }
        // Every later plan of this run puts the approved job back, as a fresh queue may not list it.
        d.st().carry.approved.push((item.clone(), entry));
    }
    view.items = read.into_iter().chain(approved).chain(unread).collect();
    if any_approved {
        // The plan's entries are oldest first, as the items are.
        let place = |id: &str| {
            view.items
                .iter()
                .position(|i| i.job == id)
                .unwrap_or(usize::MAX)
        };
        view.plan.sort_by_key(|e| place(&e.id));
    }
    view.waiting.extend(left);
    // Planning comes first: the open requests are planned together, and the items are worked in plan order,
    // not oldest first. Items with no plan entry keep their place ahead of the planned ones; items whose plan
    // is incomplete, or that wait on each other in a loop, are not worked.
    let reserve = d.caps.read + d.caps.run;
    let plan = d.prepare(&mut view, reserve);
    // Items on a paused line get their one "passed over" line now, so nothing else prints their reason.
    d.pass_over_paused();
    let first_paused: BTreeSet<String> = d.st().pause_held.keys().cloned().collect();
    for refusal in plan
        .refused
        .iter()
        .filter(|r| !first_paused.contains(&r.id))
    {
        d.note(format!(
            "item {} is not worked: {}",
            refusal.id, refusal.reason
        ));
    }
    if !plan.cyclic.is_empty() {
        d.note(format!(
            "items {} wait on each other, so none of them is worked",
            plan.cyclic.join(", ")
        ));
    }
    // The plan is built again before each wave (one item, or one group under `drain_concurrent`) from the
    // queue as it then stands, by the same `prepare` as `athena plan`. `tried` is what this run has already
    // handed to a worker, so the loop ends.
    let first_plan = plan.clone();
    // Measured from the plan the run started with, before any item is worked, for both paths.
    let plan_group_sizes: Vec<usize> = first_plan.groups.iter().map(Vec::len).collect();
    let unplanned = view
        .items
        .iter()
        .filter(|i| !view.plan.iter().any(|e| e.id == i.job))
        .count();
    let mut plan = plan;
    let mut wave: Vec<String> = Vec::new();
    loop {
        {
            let mut st = d.st();
            for entry in &view.plan {
                st.touches.insert(entry.id.clone(), entry.touches.clone());
            }
        }
        let mut batches = {
            let st = d.st();
            batches_for(&view.items, &view.plan, &plan, &st.tried)
        };
        if batches.is_empty() || d.stopped() {
            break;
        }
        let mut now = batches.remove(0);
        if one_at_a_time {
            // One item runs. The rest of its group is not dispatched now (a later wave takes it); it is
            // passed on as `later` so the read-ahead can read it while the one item runs.
            let rest = now.split_off(now.len().min(1));
            if !rest.is_empty() {
                batches.insert(0, rest);
            }
        }
        wave = now.iter().map(|i| i.job.clone()).collect();
        d.st().tried.extend(wave.iter().cloned());
        d.note_dispatch(&wave, &plan, &view.plan);
        run(d, vec![now], batches);
        if d.stopped() {
            break;
        }
        match steps.queue() {
            Ok(mut next) => {
                next.waiting = std::mem::take(&mut view.waiting);
                view = next;
                plan = d.prepare(&mut view, reserve);
                d.pass_over_paused();
            }
            Err(error) => {
                d.note(format!(
                    "the queue could not be read again, so the plan is not rebuilt: {error}"
                ));
                break;
            }
        }
    }
    d.sweep(&view, &plan, &wave);
    let plan = first_plan;
    let mut st = d.st();
    // A reading made ahead that no job used (the run ended first) was paid for; say so.
    for (item, made) in std::mem::take(&mut st.ahead) {
        let job = made.read.job.as_deref().unwrap_or("?");
        st.notes.push(format!(
            "the reading of {item} (job {job}) was made ahead but not used: the run ended first"
        ));
    }
    if st.rebases.total > 0 {
        let line = format!(
            "rebases: {} of {} needed a re-check ({RECHECK_RULE})",
            st.rebases.rechecked, st.rebases.total
        );
        st.notes.push(line);
    }
    let mut stop = st.stop.unwrap_or(Stop::QueueEmpty);
    // The landed config being unreadable is a fact of its own, and the stop keeps only the first reason, so
    // it is read here from the state and not from the stop. Whatever else ended the run, the user's first
    // action is to reinstall: a stored `Rebuilt` was built before that landing, so it would restart onto the
    // old schema. A failed rebuild keeps its own stop, because its build output is what the user needs.
    if let Some(why) = st.unreadable_config.clone()
        && stop != Stop::RebuildFailed
    {
        stop = Stop::NeedsReinstall;
        st.notes.push(reinstall_note(&why));
    }
    // A rebuilt Athena carries on with what is left, unless there is nothing left or a stop was asked for.
    let mut restart = None;
    if stop == Stop::Rebuilt {
        let jobs_left = limits.max_jobs.saturating_sub(st.started);
        let budget_left = limits.budget_usd - st.budget.spent();
        if steps.stop_requested() {
            stop = Stop::Requested;
        } else if jobs_left == 0 {
            stop = Stop::JobLimit;
        } else if budget_left <= 0.0 {
            stop = Stop::BudgetSpent;
        } else {
            restart = Some(Restart {
                budget_usd: budget_left,
                target_rating: limits.target_rating,
                max_jobs: jobs_left,
            });
        }
    }
    let landed = st.landed.len() as u32;
    // With nothing landed the only rating there is, if any, is the stale one, and it says so.
    let rating = st.rating.or_else(|| {
        previous.map(|value| RatingLine {
            value,
            after_landing: false,
            met_target: false,
        })
    });
    Ok(Report {
        stop,
        spent_usd: st.budget.spent(),
        budget_usd: limits.budget_usd,
        target_rating: limits.target_rating,
        landed,
        page_unchanged: landed == 0,
        outcome_changed: st.outcome_landings > 0,
        rating,
        held_back: st.held_back.clone(),
        queued: view
            .items
            .into_iter()
            .filter(|i| !st.landed.contains(&i.job))
            .collect(),
        waiting: view.waiting,
        plan_groups: plan.groups,
        plan_group_sizes,
        unplanned,
        refused: plan
            .refused
            .into_iter()
            .filter(|r| !first_paused.contains(&r.id))
            .collect(),
        notes: st.notes.clone(),
        jobs: st.jobs.clone(),
        rebuild_output: st.rebuild_output.clone(),
        restart,
        rebases: st.rebases.clone(),
        improvement_turns: st.improvement_turns.clone(),
    })
}

/// What to tell the user when a landing made the configuration unreadable for the running build: the job is
/// landed, no failure is counted, and the user reinstalls Athena. `why` is the read error. Shared by the drain
/// and by `athena land`.
pub fn reinstall_note(why: &str) -> String {
    format!(
        "the landed athena.toml could not be read by this build ({why}); the job is landed and no failure is counted. Reinstall Athena (./setup.sh in the checkout) and run `athena drain` again"
    )
}

/// Carries a drain across a rebuild: when `report` ended in [`Stop::Rebuilt`], `restart` is called with what is
/// left of the budget and job limit and its result is the run's result. The real `restart` replaces this
/// process with the newly installed binary (`athena drain` with [`reexec_args`]) and returns only if that
/// failed. Any other report is returned as it is.
pub fn follow_restart(
    report: Report,
    ui: &dyn Ui,
    restart: &dyn Fn(Limits) -> anyhow::Result<Report>,
) -> anyhow::Result<Report> {
    let Some(left) = report.restart else {
        return Ok(report);
    };
    ui.line(&format!(
        "drain      continuing with the new build: ${:.2} and {} job(s) left",
        left.budget_usd, left.max_jobs
    ));
    restart(left.limits())
}

/// The coordinated plan of the open requests, as `athena plan` prints it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PlanReport {
    /// Every open request's entry, oldest first.
    pub entries: Vec<PlanEntry>,
    /// The order, the parallel groups, the flagged items and any loop of dependencies.
    pub plan: Plan,
    /// What the planning pass spent reading unread requests, in US dollars.
    pub spent_usd: f64,
    pub notes: Vec<String>,
}

/// `athena plan`: runs the same planning pass as `drain` (reading the unread requests whose plan is incomplete,
/// within `read_budget_usd`) and returns the plan without working anything.
pub fn plan(steps: &dyn Steps, read_budget_usd: f64, ui: &dyn Ui) -> anyhow::Result<PlanReport> {
    let caps = steps.caps()?;
    let mut view = steps.queue()?;
    let limits = Limits {
        budget_usd: read_budget_usd,
        target_rating: DEFAULT_TARGET_RATING,
        max_jobs: u32::MAX,
    };
    let d = Drain::new(steps, ui, caps, limits);
    // Planning runs nothing, so only the read has to fit `--budget`.
    let reserve = d.caps.read;
    let plan = d.prepare(&mut view, reserve);
    if !plan.cyclic.is_empty() {
        d.note(format!(
            "items {} wait on each other, so none of them is worked",
            plan.cyclic.join(", ")
        ));
    }
    let st = d.st();
    Ok(PlanReport {
        entries: view.plan,
        plan,
        spent_usd: st.budget.spent(),
        notes: st.notes.clone(),
    })
}
