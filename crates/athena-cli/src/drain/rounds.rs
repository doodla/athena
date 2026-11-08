//! The drain's rounds: building the engine, rebuilds, and the waves of items run side by side.

use super::*;

impl<S: Steps + ?Sized, U: Ui + ?Sized> Drain<'_, S, U> {
    pub(super) fn new<'a>(steps: &'a S, ui: &'a U, caps: Caps, limits: Limits) -> Drain<'a, S, U> {
        Drain {
            steps,
            ui,
            caps,
            limits,
            state: Mutex::new(State {
                budget: Budget::new(limits.budget_usd),
                jobs: Vec::new(),
                started: 0,
                landed: Vec::new(),
                outcome_landings: 0,
                rating: None,
                held_back: None,
                notes: Vec::new(),
                rebuild_due: false,
                rebuild_output: None,
                unreadable_config: None,
                read_cost: Default::default(),
                stop: None,
                ahead: BTreeMap::new(),
                reading: BTreeSet::new(),
                pumped: BTreeSet::new(),
                begun: BTreeSet::new(),
                done: BTreeSet::new(),
                landings: Vec::new(),
                touches: BTreeMap::new(),
                batch: 0,
                pump_done: false,
                pump_gone: false,
                tried: BTreeSet::new(),
                dispatched: 0,
                since_turn: 0,
                improving: BTreeSet::new(),
                improvement_turns: Vec::new(),
                passed_over: BTreeSet::new(),
                pause_held: BTreeMap::new(),
                carry: Carry::default(),
                queue: LandingQueue::default(),
                rebases: RebaseTally::default(),
                owing: BTreeSet::new(),
            }),
            landing: Mutex::new(()),
            wake: Condvar::new(),
        }
    }

    /// Rebuilds and installs Athena when a landing touched [`BUILD_PATHS`] since the last rebuild. Returns
    /// whether it installed a new build, or None when the rebuild failed (the run is then ended with
    /// [`Stop::RebuildFailed`], which replaces any other reason: the installed binary is stale, and the build
    /// output is what the user needs). Landings wait while it builds, because a landing moves the checkout
    /// the build reads.
    pub(super) fn rebuild_if_due(&self) -> Option<bool> {
        let _one_at_a_time = self.landing.lock().unwrap_or_else(PoisonError::into_inner);
        if !std::mem::take(&mut self.st().rebuild_due) {
            return Some(false);
        }
        self.ui.line(
            "drain      the landing changed Athena's own code; rebuilding and installing it (setup.sh)",
        );
        match self.steps.rebuild() {
            Ok(Rebuild::Installed) => {
                self.ui.line("drain      Athena rebuilt and installed");
                Some(true)
            }
            Ok(Rebuild::NotApplicable) => Some(false),
            Err(output) => {
                self.ui.line("drain      rebuild failed; stopping");
                let mut st = self.st();
                st.rebuild_output = Some(output);
                st.stop = Some(Stop::RebuildFailed);
                None
            }
        }
    }

    /// What a worker does when its item has ended: a landed item is followed by a rating, and a reason to
    /// stop ends the run for every worker. One item's failure stops no job already in flight.
    pub(super) fn finish(&self, item: &Item, end: ItemEnd) {
        match end {
            ItemEnd::Landed => {
                let Some(rebuilt) = self.rebuild_if_due() else {
                    return;
                };
                // Keyed on the fact that the landed config is unreadable, not on the stop reason: the stop
                // keeps the first reason found, so another worker's earlier stop must not hide this.
                let unreadable = self.st().unreadable_config.is_some();
                if unreadable {
                    // Rating reads the config this running build cannot parse, so it is skipped.
                    if rebuilt {
                        // The installed binary is now the newer build, which reads the landed config: the
                        // reinstall advice no longer applies. Only a reinstall stop becomes the restart;
                        // another worker's earlier reason stands.
                        let mut st = self.st();
                        st.unreadable_config = None;
                        if st.stop == Some(Stop::NeedsReinstall) {
                            st.stop = Some(Stop::Rebuilt);
                        }
                    }
                    return;
                }
                if let Some(reason) = self.rate() {
                    self.set_stop(reason);
                }
                // The first reason found stands; a run that is ending anyway has still been rebuilt.
                if rebuilt {
                    self.set_stop(Stop::Rebuilt);
                }
            }
            ItemEnd::Failed => {
                // One item's double failure is the scope guard's and Hestia's to judge: record it and go on.
                // A cause in the environment (the login, the usage limit, the disk) is not about this item:
                // the next item would fail the same way, so the run ends.
                let reason = self.last_reason(&item.job, "its last attempt did not pass");
                // An attempt that passed but did not land has its landing text on its line, not its run's.
                let shown = self
                    .last_landing_failure(&item.job)
                    .unwrap_or_else(|| reason.clone());
                self.pass_over(
                    &item.job,
                    &format!("failed twice; left for the scope guard and Hestia: {shown}"),
                );
                if matches!(
                    StopCause::of(&reason),
                    StopCause::ProducerAuth | StopCause::UsageLimit | StopCause::LowDisk
                ) {
                    self.set_stop(Stop::Environment);
                }
            }
            ItemEnd::Budget => {
                self.pass_over(
                    &item.job,
                    &self.last_reason(&item.job, "the budget did not cover its read and run"),
                );
                self.set_stop(Stop::BudgetSpent);
            }
            ItemEnd::JobLimit => {
                self.pass_over(&item.job, "over the job limit");
                self.set_stop(Stop::JobLimit);
            }
            ItemEnd::Stop => {
                self.pass_over(&item.job, "run ended first: stop requested");
                self.set_stop(Stop::Requested);
            }
            ItemEnd::Idle => {
                self.pass_over(&item.job, "run ended first: another job ended the run")
            }
            ItemEnd::NeedsAccepting => self.pass_over(
                &item.job,
                &format!(
                    "not ready: {}",
                    self.last_reason(&item.job, "its reading needs accepting")
                ),
            ),
            ItemEnd::Held => self.pass_over(
                &item.job,
                &format!(
                    "held: {}",
                    self.last_reason(&item.job, "it was held and a later pass runs it")
                ),
            ),
            ItemEnd::Waiting => self.pass_over(&item.job, &self.last_landing(&item.job)),
            ItemEnd::Paused(reason) | ItemEnd::Flagged(reason) => {
                self.pass_over(&item.job, &reason)
            }
            ItemEnd::Skipped => self.pass_over(
                &item.job,
                &format!(
                    "skipped: {}",
                    self.last_reason(&item.job, "the reader gave no reading twice")
                ),
            ),
        }
    }

    /// Takes items from `queue`, oldest first, until it is empty or the run has ended.
    pub(super) fn take_until_done(&self, queue: &Mutex<VecDeque<Item>>) {
        loop {
            if self.stopped() {
                return;
            }
            let next = queue
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .pop_front();
            let Some(item) = next else { return };
            let made = self.claim(&item);
            let end = self.work(&item, made);
            self.st().done.insert(item.job.clone());
            self.wake.notify_all();
            self.finish(&item, end);
        }
    }

    /// A worker takes `item`. If the read-ahead is reading it right now, waits for that reading to end (it is
    /// never read twice), then takes the reading made ahead, if there is one.
    pub(super) fn claim(&self, item: &Item) -> Option<Ahead> {
        let mut st = self.st();
        while st.reading.contains(&item.job) && !st.pump_gone {
            st = self.pause(st);
        }
        st.ahead.remove(&item.job)
    }

    /// Waits a short while for a change of the shared state. Waiters check their condition again afterwards, so
    /// a wake-up that comes early or not at all costs only a few milliseconds.
    pub(super) fn pause<'g>(&self, guard: MutexGuard<'g, State>) -> MutexGuard<'g, State> {
        self.wake
            .wait_timeout(guard, Duration::from_millis(20))
            .unwrap_or_else(PoisonError::into_inner)
            .0
    }

    /// One job at a time, in order, on the calling thread.
    pub(super) fn run_serial(&self, batches: Vec<Vec<Item>>) {
        for batch in batches {
            self.take_until_done(&Mutex::new(VecDeque::from(batch)));
        }
    }
}

impl<S: Steps + Sync + ?Sized, U: Ui + Sync + ?Sized> Drain<'_, S, U> {
    /// Runs each batch with up to `workers` jobs at once; the next batch starts when the batch before it is
    /// done. A batch holds items that no dependency links and whose touched paths are disjoint.
    ///
    /// With [`Steps::read_ahead`] above 0, one more thread reads (and plans) the items of the batches to come
    /// while the batch runs; see the module text. This holds for one worker as well as for several.
    ///
    /// `batches` is one wave (the drain plans again before the next): the batch to run. `later` is the
    /// batches of the same plan after it; it only chooses what to read ahead and is never dispatched.
    pub(super) fn run_parallel(
        &self,
        batches: Vec<Vec<Item>>,
        later: Vec<Vec<Item>>,
        workers: usize,
    ) {
        let window = self.steps.read_ahead() as usize;
        if window == 0 {
            return self.run_batches(batches, workers);
        }
        let upcoming: Vec<(usize, Item)> = batches
            .iter()
            .chain(&later)
            .enumerate()
            .flat_map(|(n, batch)| batch.iter().cloned().map(move |item| (n, item)))
            .collect();
        {
            // A new wave: the read-ahead of the last one has ended. Readings it made stay for their jobs.
            let mut st = self.st();
            st.pump_done = false;
            st.pump_gone = false;
            st.batch = 0;
        }
        std::thread::scope(|scope| {
            scope.spawn(|| self.read_ahead_pump(&upcoming, window));
            // However the batches end (all run, a reason to stop, a worker's panic), the read-ahead is told to
            // end, so the scope can close.
            let _end = FlagOnDrop {
                state: &self.state,
                wake: &self.wake,
                set: |st| st.pump_done = true,
            };
            self.run_batches(batches, workers);
        });
    }

    pub(super) fn run_batches(&self, batches: Vec<Vec<Item>>, workers: usize) {
        for (n, batch) in batches.into_iter().enumerate() {
            if self.stopped() {
                return;
            }
            self.st().batch = n;
            let count = workers.min(batch.len()).max(1);
            let queue = Mutex::new(VecDeque::from(batch));
            std::thread::scope(|scope| {
                for _ in 0..count {
                    scope.spawn(|| self.take_until_done(&queue));
                }
            });
        }
    }

    /// What the read-ahead does next. It considers the items of the batches after the running one, in plan
    /// order, that it has not tried and that are not read already (nor interrupted). It starts one only when
    /// every job of the running batch has started, fewer than `window` readings are held or being made, the job limit leaves room for another job and the
    /// budget fits a read and a run (as a job is started only when both fit, so nothing is read for want of
    /// money to run it), and the item's touched paths are disjoint from every job of the running batch that has
    /// not finished (an item whose paths are unknown is not read ahead beside a job in flight).
    pub(super) fn pick_ahead(&self, st: &State, upcoming: &[(usize, Item)], window: usize) -> Pick {
        let touches = |item: &Item| st.touches.get(&item.job).map_or(&[][..], Vec::as_slice);
        let held = st.ahead.len() + st.reading.len();
        // The running batch starts its own jobs first: a read made ahead must not take the money, or the
        // place in the job limit, that one of them has not yet held.
        let unstarted = upcoming.iter().any(|(b, i)| {
            *b <= st.batch && !st.begun.contains(&i.job) && !st.done.contains(&i.job)
        });
        let no_room = unstarted
            || held >= window
            || st.started as usize + held >= self.limits.max_jobs as usize
            || !st.budget.fits(self.caps.read + self.caps.run);
        let mut any = false;
        for (at, (batch, item)) in upcoming.iter().enumerate() {
            if *batch <= st.batch || item.read || item.interrupted || st.pumped.contains(&item.job)
            {
                continue;
            }
            any = true;
            if no_room {
                break;
            }
            let clear = upcoming
                .iter()
                .filter(|(b, other)| *b <= st.batch && !st.done.contains(&other.job))
                .all(|(_, other)| paths_disjoint(touches(item), touches(other)));
            if clear {
                return Pick::Start(at);
            }
        }
        if any { Pick::Wait } else { Pick::Done }
    }

    /// The read-ahead thread: reads, and for a hard job plans, items of the batches to come while the running
    /// batch works, never holding more than `window` readings that no job has claimed. Every read and plan
    /// holds its cap against the budget while it runs and adds what it cost, as any other call does, and no
    /// lock is held across a call into [`Steps`]. It ends when there is nothing left to read ahead, a stop was
    /// asked for, or the batches are over.
    pub(super) fn read_ahead_pump(&self, upcoming: &[(usize, Item)], window: usize) {
        // If this thread ends in any way, a worker must not wait for a reading it will never finish.
        let _gone = FlagOnDrop {
            state: &self.state,
            wake: &self.wake,
            set: |st| st.pump_gone = true,
        };
        loop {
            let (at, seen) = {
                let mut st = self.st();
                loop {
                    if st.pump_done || st.stop.is_some() {
                        return;
                    }
                    match self.pick_ahead(&st, upcoming, window) {
                        Pick::Done => return,
                        Pick::Wait => st = self.pause(st),
                        Pick::Start(at) => {
                            let job = upcoming[at].1.job.clone();
                            st.pumped.insert(job.clone());
                            st.reading.insert(job);
                            st.budget.hold(self.caps.read);
                            break (at, st.landings.len());
                        }
                    }
                }
            };
            let job = upcoming[at].1.job.as_str();
            if self.steps.stop_requested() {
                let mut st = self.st();
                st.reading.remove(job);
                st.budget.release(self.caps.read);
                drop(st);
                self.wake.notify_all();
                return;
            }
            let read = self.steps.read(ReadOf::Queued(job));
            let usable = matches!(
                (&read.job, &read.error, read.runnable),
                (Some(_), None, true)
            );
            {
                let mut st = self.st();
                st.budget.release(self.caps.read);
                st.budget.add(read.cost);
                *st.read_cost.entry(job.to_string()).or_insert(0.0) += read.cost;
            }
            let mut problem = (!usable).then(|| {
                read.error
                    .clone()
                    .unwrap_or_else(|| "no capability fits the request".into())
            });
            if usable {
                // A hard job is planned too. The plan is paid for like the reading; with no room for its cap
                // it is left for when the job starts.
                let cap = self.steps.plan_cap();
                let room = {
                    let mut st = self.st();
                    let room = st.budget.fits(cap);
                    if room {
                        st.budget.hold(cap);
                    }
                    room
                };
                if room {
                    let planned = self.steps.plan_ahead(job);
                    let mut st = self.st();
                    st.budget.release(cap);
                    if let Some(planned) = planned {
                        st.budget.add(planned.cost);
                        *st.read_cost.entry(job.to_string()).or_insert(0.0) += planned.cost;
                        problem = planned.error;
                    }
                }
            }
            {
                let mut st = self.st();
                match problem {
                    None => {
                        st.ahead.insert(job.to_string(), Ahead { read, seen });
                    }
                    Some(why) => st.notes.push(format!(
                        "the reading made ahead of {job} was dropped ({why}); it is read again when it starts"
                    )),
                }
                st.reading.remove(job);
            }
            self.wake.notify_all();
        }
    }
}
