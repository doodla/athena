//! The drain's work on one item: running a job, landing it through the queue, the attempts and the plan pass.

use super::*;

impl<S: Steps + ?Sized, U: Ui + ?Sized> Drain<'_, S, U> {
    pub(super) fn st(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn push(&self, line: JobLine) -> usize {
        // A job that was not read says why on its line: the reader's error is what the person needs.
        let why = match (&line.reason, line.outcome.as_str()) {
            (Some(reason), "not read") => format!(": {}", one_line(reason)),
            _ => String::new(),
        };
        self.ui.line(&format!(
            "drain      {} → {} ({}){why}",
            line.item, line.job, line.outcome
        ));
        let mut st = self.st();
        st.jobs.push(line);
        st.jobs.len() - 1
    }

    pub(super) fn note(&self, text: String) {
        self.st().notes.push(text);
    }

    /// A note that a new plan of the same run would only repeat.
    pub(super) fn note_once(&self, text: String) {
        let mut st = self.st();
        if !st.notes.contains(&text) {
            st.notes.push(text);
        }
    }

    /// Logs that the planned item `id` is not worked, with the reason: one line per item per run.
    pub(super) fn pass_over(&self, id: &str, reason: &str) {
        let first = self.st().passed_over.insert(id.to_string());
        if first {
            self.ui.line(&format!(
                "plan       {id} passed over: {}",
                one_line(reason)
            ));
        }
    }

    /// The landing text on the last job line of `item`; a waiting job's line says why it waits to merge.
    pub(super) fn last_landing(&self, item: &str) -> String {
        let st = self.st();
        st.jobs
            .iter()
            .rev()
            .find(|l| l.item == item && l.landing.starts_with("waiting to merge"))
            .map(|l| l.landing.clone())
            .unwrap_or_else(|| "waiting to merge: its pull request waits on checks".to_string())
    }

    /// The landing text of the last job line of `item` when that line's landing did not merge
    /// (`not landed: <why>`), else none.
    pub(super) fn last_landing_failure(&self, item: &str) -> Option<String> {
        let st = self.st();
        st.jobs
            .iter()
            .rev()
            .find(|l| l.item == item)
            .filter(|l| l.landing.starts_with("not landed"))
            .map(|l| l.landing.clone())
    }

    /// The reason on the last job line of `item`, else `default`.
    pub(super) fn last_reason(&self, item: &str, default: &str) -> String {
        let st = self.st();
        st.jobs
            .iter()
            .rev()
            .find(|l| l.item == item)
            .and_then(|l| l.reason.clone())
            .unwrap_or_else(|| default.to_string())
    }

    /// Ends the run for `stop`, unless another worker already found a reason.
    pub(super) fn set_stop(&self, stop: Stop) {
        let mut st = self.st();
        if st.stop.is_none() {
            st.stop = Some(stop);
        }
    }

    pub(super) fn stopped(&self) -> bool {
        self.st().stop.is_some()
    }

    /// Reads and runs one job. Starts only when the reading and the run both fit the budget, counting the
    /// caps that jobs already in flight hold.
    pub(super) fn run_job(&self, item: &Item, from: ReadOf<'_>, source: Source) -> RunStep {
        // An approved job was read when it was approved; its first attempt is only a run. So is a job whose
        // reading was made ahead and still holds.
        let pre_read = item.read && matches!(from, ReadOf::Queued(_));
        let (ReadOf::Queued(target) | ReadOf::Retry(target)) = from;
        let needed = if pre_read || matches!(source, Source::Given(_)) {
            self.caps.run
        } else {
            self.caps.read + self.caps.run
        };
        {
            let mut st = self.st();
            if st.stop.is_some() {
                return RunStep::Ended;
            }
            if st.started >= self.limits.max_jobs {
                return RunStep::JobLimit;
            }
            if !st.budget.fits(needed) {
                return RunStep::Budget;
            }
            st.started += 1;
            st.begun.insert(item.job.clone());
            st.budget.hold(needed);
        }
        let read = match source {
            Source::Given(read) => read,
            Source::Reread => self.steps.reread(target),
            Source::Read if pre_read => Read {
                job: Some(target.to_string()),
                cost: 0.0,
                runnable: true,
                error: None,
                held: false,
            },
            Source::Read => self.steps.read(from),
        };
        let runnable = matches!(
            (&read.job, &read.error, read.runnable),
            (Some(_), None, true)
        );
        let (planned_read_cost, room) = {
            let mut st = self.st();
            st.budget.release(needed);
            st.budget.add(read.cost);
            let planned = st.read_cost.remove(&item.job).unwrap_or(0.0);
            // The reading's real cost may have used up the room the run needs.
            let room = runnable && st.budget.fits(self.caps.run);
            if room {
                st.budget.hold(self.caps.run);
            }
            (planned, room)
        };
        let mut line = JobLine {
            item: item.job.clone(),
            job: read.job.clone().unwrap_or_else(|| item.job.clone()),
            outcome: "not read".into(),
            reason: None,
            cost_usd: read.cost + planned_read_cost,
            landing: "not attempted".into(),
        };
        let job = match (&read.job, &read.error, read.runnable) {
            (Some(job), None, true) => job.clone(),
            (_, error, _) => {
                line.reason = Some(error.clone().unwrap_or_else(|| {
                    "no capability fits the request, so there is nothing to run".into()
                }));
                // Refused before any money was spent because the budget does not cover the whole path: the
                // item stays queued with its reason on one line, and the next item runs.
                if read.held {
                    line.outcome = BUDGET_SHORT.into();
                    self.note(format!(
                        "item {} is left queued: {}",
                        item.job,
                        one_line(line.reason.as_deref().unwrap_or("its budget is short"))
                    ));
                    self.push(line);
                    return RunStep::Held;
                }
                // A queued job whose reader failed is recorded with no reading: it is read once more, not
                // counted as a failed run. So is a retry whose reader failed: the retry job is recorded,
                // and its next read is of that job, not a second retry of the interrupted one.
                let unread = ((matches!(from, ReadOf::Queued(_)) && !pre_read)
                    || matches!(from, ReadOf::Retry(_)))
                    && read.job.is_some()
                    && read.error.is_some();
                let retried = matches!(from, ReadOf::Retry(_))
                    .then(|| read.job.clone())
                    .flatten();
                let reason = line.reason.clone().unwrap_or_default();
                self.push(line);
                if unread {
                    return RunStep::Unread { reason, retried };
                }
                return RunStep::Failed {
                    job: item.job.clone(),
                    retry: false,
                    read: false,
                };
            }
        };
        if !room {
            line.outcome = "not run".into();
            line.reason = Some("the budget did not leave room for the run".into());
            self.push(line);
            return RunStep::Budget;
        }
        // A job interrupted since the last sweep gives up its build directory before the next one builds.
        self.steps.reclaim_interrupted();
        let run = self.steps.run(&job);
        {
            let mut st = self.st();
            st.budget.release(self.caps.run);
            st.budget.add(run.cost);
        }
        line.cost_usd += run.cost;
        line.reason = (!run.reason.is_empty()).then(|| run.reason.clone());
        match run.outcome {
            Some(OutcomeState::Landed) => {
                line.outcome = "passed".into();
                let at = self.push(line);
                RunStep::Passed(job, at)
            }
            _ if run.needs_accepting => {
                line.outcome = "needs accepting".into();
                self.note(format!(
                    "item {} is left queued: {}",
                    item.job,
                    line.reason
                        .as_deref()
                        .unwrap_or("its reading needs accepting")
                ));
                self.push(line);
                RunStep::NeedsAccepting
            }
            _ if run.held => {
                // The reason says which hold it is, so a slot wait is never counted as low disk.
                let (outcome, fallback) = if run.reason.starts_with(BUDGET_SHORT) {
                    (
                        BUDGET_SHORT,
                        "the job's budget does not cover the whole path",
                    )
                } else if run.reason.starts_with(athena_core::disk::BUILD_SLOT_WAIT) {
                    (
                        athena_core::disk::BUILD_SLOT_WAIT,
                        "no build slot became free in time",
                    )
                } else {
                    (
                        athena_core::disk::LOW_DISK,
                        "free disk space is below the minimum",
                    )
                };
                line.outcome = outcome.into();
                self.note(format!(
                    "item {} is left queued: {}",
                    item.job,
                    line.reason.as_deref().unwrap_or(fallback)
                ));
                self.push(line);
                RunStep::Held
            }
            other => {
                line.outcome = other.map_or("no outcome", OutcomeState::as_str).to_string();
                self.push(line);
                RunStep::Failed {
                    job,
                    retry: matches!(
                        other,
                        Some(
                            OutcomeState::Rejected | OutcomeState::Failed | OutcomeState::TimedOut
                        )
                    ),
                    read: true,
                }
            }
        }
    }

    /// Takes the next place in the landing queue. The place is kept until the returned slot is dropped.
    pub(super) fn enter_queue(&self) -> LandingSlot<'_> {
        let ticket = self.st().queue.join();
        LandingSlot {
            state: &self.state,
            wake: &self.wake,
            ticket,
        }
    }

    /// Returns when `slot` is the head of the landing queue. A worker that has to wait says so once, outside
    /// the state lock; the head returns at once and says nothing.
    pub(super) fn await_head(&self, job: &str, slot: &LandingSlot<'_>) {
        if self.st().queue.is_head(slot.ticket) {
            return;
        }
        self.ui
            .line(&format!("drain      {job} waits in the landing queue"));
        let mut st = self.st();
        while !st.queue.is_head(slot.ticket) {
            st = self.pause(st);
        }
    }

    /// The re-check rule after a rebase, run under the landing lock before the job is landed. Asks
    /// [`Steps::rebase`]; a rebase that moved the branch is counted, and one that touched a path the candidate
    /// changed ([`rebase_needs_recheck`]) puts the job in `owing`. A job in `owing` has its checks run again
    /// ([`Steps::rerun_checks`]) before it lands. None means the landing goes on; Some ends this attempt.
    pub(super) fn recheck_after_rebase(&self, job: &str, at: usize) -> Option<LandStep> {
        if let Rebased::Moved { candidate, touched } = self.steps.rebase(job) {
            let needed = rebase_needs_recheck(&candidate, &touched);
            let mut st = self.st();
            st.rebases.total += 1;
            if needed {
                st.rebases.rechecked += 1;
                st.owing.insert(job.to_string());
            }
        }
        if !self.st().owing.contains(job) {
            return None;
        }
        let cap = self.steps.rerun_cap();
        {
            let mut st = self.st();
            if !st.budget.fits(cap) {
                let why =
                    "not landed: the re-check after the rebase does not fit the budget".to_string();
                st.jobs[at].landing = why.clone();
                st.jobs[at].reason = Some(why.clone());
                drop(st);
                self.ui.line(&format!("drain      {job} {why}"));
                return Some(LandStep::Budget);
            }
            st.budget.hold(cap);
        }
        self.ui.line(&format!(
            "drain      {job} re-running checks after the rebase"
        ));
        let check = self.steps.rerun_checks(job);
        {
            let mut st = self.st();
            st.budget.release(cap);
            st.budget.add(check.cost);
            if check.passed {
                st.owing.remove(job);
            }
        }
        if check.passed {
            return None;
        }
        let why = format!("not landed: checks failed after rebase: {}", check.detail);
        self.ui.line(&format!("drain      {job} {why}"));
        self.st().jobs[at].landing = why;
        Some(LandStep::NotLanded)
    }

    pub(super) fn land(&self, job: &str, at: usize) -> LandStep {
        let _one_at_a_time = self.landing.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(step) = self.recheck_after_rebase(job, at) {
            return step;
        }
        match self.steps.land(job) {
            Landing::Landed { changed } => {
                let outcome = changes_outcome(&changed);
                let build = touches_build(&changed);
                {
                    let mut st = self.st();
                    st.jobs[at].landing = "landed".into();
                    st.rebuild_due |= build;
                    if outcome {
                        st.outcome_landings += 1;
                    }
                    st.landings.push(changed);
                }
                // Root cause of a landed job being recorded as failed: a landing never checked that the
                // running build can read the config it lands. A job can add a config field with the code
                // that reads it, and the build that keeps running is the older one. Every `Steps::land`
                // funnels through here under the landing lock, so this is the one place the landed head is
                // read. The configuration was readable when the run started, so if it is not now, the
                // landing is what changed it. A newer schema cannot be told cheaply from other damage, so
                // any read error is taken as "the landed config is newer than this build". The fact is kept
                // in `unreadable_config`, which the rating and the report read; the stop reason is only the
                // signal that ends the run, and it keeps the first reason found, so it cannot carry the fact.
                if let Some(why) = self.steps.config_after_landing() {
                    self.ui.line(&format!(
                        "drain      {job} landed, but its athena.toml cannot be read by this build; stopping, reinstall Athena"
                    ));
                    self.st().unreadable_config = Some(why);
                    self.set_stop(Stop::NeedsReinstall);
                }
                self.wake.notify_all();
                if !outcome {
                    self.ui.line(&format!(
                        "drain      {job} landed but changed nothing the explainer is built from"
                    ));
                }
                LandStep::Landed
            }
            Landing::NotLanded(why) => {
                self.ui.line(&format!("drain      {job} not landed: {why}"));
                self.st().jobs[at].landing = format!("not landed: {why}");
                LandStep::NotLanded
            }
            Landing::Waiting(why) => {
                self.ui
                    .line(&format!("drain      {job} waiting to merge: {why}"));
                self.st().jobs[at].landing = format!("waiting to merge: {why}");
                LandStep::Waiting
            }
        }
    }

    /// Works one item through its attempts: read and run, land, and on a failure try once more.
    ///
    /// `ahead` is the reading the read-ahead made of this item, if it did; it is used by the first attempt.
    pub(super) fn work(&self, item: &Item, ahead: Option<Ahead>) -> ItemEnd {
        let mut last_job = item.job.clone();
        let end = self.attempts(item, ahead, &mut last_job);
        // Whatever the end, the item's table row goes with it: the job it last had, and the item's own.
        self.steps.item_over(&last_job);
        if last_job != item.job {
            self.steps.item_over(&item.job);
        }
        end
    }

    /// The attempts of [`Drain::work`]; `last_job` ends as the last job the item had.
    pub(super) fn attempts(
        &self,
        item: &Item,
        mut ahead: Option<Ahead>,
        last_job: &mut String,
    ) -> ItemEnd {
        // Once the job is read, an attempt that is not a retry only runs it.
        let mut item = item.clone();
        let item = &mut item;
        // An interrupted job has no finished attempt to run: its first attempt reads it again.
        let mut retry = item.interrupted;
        // The job that passed, its line, and its place in the landing queue. The place is kept through both
        // land calls below and given up when this function returns or unwinds.
        let mut passed: Option<(String, usize, LandingSlot<'_>)> = None;
        let mut failures = 0;
        // Reads that gave no reading. One is read again; a second skips the item.
        let mut unread = 0;
        loop {
            if passed.is_none() {
                // A job boundary: the previous attempt, if any, is over and the next has not started.
                if self.steps.stop_requested() {
                    return ItemEnd::Stop;
                }
                // A line the scope guard paused (possibly by this item's own last attempt) refuses a read
                // or run with the pause text. That is not a failure of the item: pass it over unread.
                if let Some(reason) = self.paused_reason(item) {
                    return ItemEnd::Paused(reason);
                }
                let from = if retry {
                    ReadOf::Retry(last_job.as_str())
                } else {
                    ReadOf::Queued(last_job.as_str())
                };
                let source = match ahead.take() {
                    Some(made) => self.vet(item, made),
                    None => Source::Read,
                };
                match self.run_job(item, from, source) {
                    RunStep::Budget => return ItemEnd::Budget,
                    RunStep::JobLimit => return ItemEnd::JobLimit,
                    RunStep::Ended => return ItemEnd::Idle,
                    RunStep::NeedsAccepting => return ItemEnd::NeedsAccepting,
                    RunStep::Held => return ItemEnd::Held,
                    RunStep::Unread { reason, retried } => {
                        // A retry job was made: read that job next, not a second retry of the old one.
                        if let Some(job) = retried {
                            *last_job = job;
                            retry = false;
                        }
                        unread += 1;
                        if unread >= 2 {
                            self.note(format!(
                                "item {} is skipped: the reader gave no reading twice: {}",
                                item.job,
                                one_line(&reason)
                            ));
                            return ItemEnd::Skipped;
                        }
                        continue;
                    }
                    RunStep::Passed(job, at) => passed = Some((job, at, self.enter_queue())),
                    RunStep::Failed {
                        job,
                        retry: again,
                        read,
                    } => {
                        // Only a rejected or failed job is read again (`athena retry`); a job that ended
                        // any other way is still proposed or blocked, so its next attempt runs it.
                        // An interrupted job whose retry could not be read is still interrupted.
                        retry = again || (item.interrupted && !read);
                        if read {
                            *last_job = job;
                            item.read = true;
                        }
                    }
                }
            }
            if let Some((job, at, slot)) = &passed {
                self.await_head(job, slot);
                match self.land(job, *at) {
                    LandStep::Landed => {
                        self.st().landed.push(item.job.clone());
                        return ItemEnd::Landed;
                    }
                    // Not a failed attempt: nothing is retried, and no failure is counted.
                    LandStep::Waiting => return ItemEnd::Waiting,
                    // The run ends for the budget; the job is not tried again and no failure is counted.
                    LandStep::Budget => return ItemEnd::Budget,
                    LandStep::NotLanded => {}
                }
            }
            failures += 1;
            if failures >= ATTEMPTS {
                // The last failure may itself have paused the line. That is the scope guard's doing: pass
                // the item over and let the batch go on. A line the guard has only flagged (a flag still
                // dispatches, so the item had both attempts) is passed over the same way, with the flag
                // reason. An item whose line is neither paused nor flagged is passed over too (see
                // `ItemEnd::Failed`): one item's failures are not a reason to end the batch.
                if let Some(reason) = self.paused_reason(item) {
                    return ItemEnd::Paused(reason);
                }
                return match self.flagged_reason(item) {
                    Some(reason) => ItemEnd::Flagged(reason),
                    None => ItemEnd::Failed,
                };
            }
        }
    }

    /// The reason the scope guard has only flagged the line of `item`, if it has.
    pub(super) fn flagged_reason(&self, item: &Item) -> Option<String> {
        self.steps
            .flagged(std::slice::from_ref(&item.job))
            .into_iter()
            .next()
            .map(|(_, reason)| reason)
    }

    /// The reason the scope guard has paused the line of `item`, if it has.
    pub(super) fn paused_reason(&self, item: &Item) -> Option<String> {
        self.steps
            .paused(std::slice::from_ref(&item.job))
            .into_iter()
            .next()
            .map(|(_, reason)| reason)
    }

    /// The stale-reading re-check, made when the job of a reading made ahead starts. A landing since the
    /// reading began may have changed a path the reading touches; if none did, the reading is used as it is.
    /// Otherwise the drain notes which paths and asks [`Steps::recheck`]: a reading that still holds is kept,
    /// and one that does not is replaced by [`Steps::reread`].
    pub(super) fn vet(&self, item: &Item, made: Ahead) -> Source {
        // The plan entry as the records stand now, else the one the queue view carried.
        let touches = match self.steps.plan_entry(&item.job) {
            Some(entry) => entry.touches,
            None => self
                .st()
                .touches
                .get(&item.job)
                .cloned()
                .unwrap_or_default(),
        };
        let landed: Vec<String> = {
            let st = self.st();
            st.landings
                .iter()
                .skip(made.seen)
                .flatten()
                .cloned()
                .collect()
        };
        // What was paid for the reading was charged when it was made.
        let read = Read {
            cost: 0.0,
            ..made.read
        };
        let hit = stale_paths(&touches, &landed);
        if hit.is_empty() {
            return Source::Given(read);
        }
        self.note(format!(
            "the reading of {} is stale: it was made before a landing changed {}, which it touches; checking that it still holds",
            item.job,
            hit.join(", ")
        ));
        if self.steps.recheck(&item.job, &hit) {
            Source::Given(read)
        } else {
            self.note(format!(
                "the reading of {} no longer holds; reading it again",
                item.job
            ));
            Source::Reread
        }
    }

    /// Approves the proposals the ladder lets their role change. Each approval reads a job, so it must leave
    /// room for that job's run and stay under the job limit. Returns the jobs it made, and the proposals it
    /// left open (with a note saying why).
    pub(super) fn approve_all(&self, proposals: Vec<Waiting>) -> (Vec<Item>, Vec<Waiting>) {
        let mut approved = Vec::new();
        let mut left = Vec::new();
        for proposal in proposals {
            let room = {
                let st = self.st();
                (st.started as usize + approved.len()) < self.limits.max_jobs as usize
                    && st.budget.fits(self.caps.read + self.caps.run)
            };
            if !room {
                self.note(format!(
                    "proposal {} is within its role's limit but was not approved: no room in the job limit or the budget",
                    proposal.id
                ));
                left.push(proposal);
                continue;
            }
            let approval = self.steps.approve(&proposal.id);
            self.st().budget.add(approval.cost);
            match (approval.job, approval.error) {
                (Some(job), None) => {
                    self.ui
                        .line(&format!("drain      approved {} → {job}", proposal.id));
                    approved.push(Item {
                        job,
                        title: proposal.title,
                        read: true,
                        interrupted: false,
                    });
                }
                (_, error) => {
                    self.note(format!(
                        "proposal {} could not be approved: {}",
                        proposal.id,
                        error.unwrap_or_else(|| "no job was made".into())
                    ));
                    left.push(proposal);
                }
            }
        }
        (approved, left)
    }

    /// The planning pass. Reads every unread request whose plan from its own words is incomplete, so that the
    /// reading (not a default) can fill the plan: without this an unread vague request would be refused for
    /// being unread and stay in the queue forever. Each such read must fit the budget and the job limit like
    /// any other. An item whose plan is still incomplete after its reading is refused with the reason; one
    /// whose plan is complete is marked read and runs without a second reading. Unread items whose plan is
    /// already complete are left for the loop to read when it works them, in plan order.
    ///
    /// Returns the coordinated plan; its `refused` also holds the items that could not be read.
    ///
    /// `reserve` is what must fit the budget besides what is spent before an item is read: the read cap plus
    /// the run cap for `drain`, which goes on to run the item, and the read cap alone for `plan`, which runs
    /// nothing.
    pub(super) fn prepare(&self, view: &mut QueueView, reserve: f64) -> Plan {
        // A view made after this run began is joined with what the run remembers: the jobs it approved (a
        // queue may not list them), the readings it made and the items it could not read.
        let carry = self.st().carry.clone();
        let absent: Vec<Item> = carry
            .approved
            .iter()
            .map(|(item, _)| item)
            .filter(|item| !view.items.iter().any(|i| i.job == item.job))
            .cloned()
            .collect();
        if !absent.is_empty() {
            let (read, unread): (Vec<Item>, Vec<Item>) = view.items.drain(..).partition(|i| i.read);
            view.items = read.into_iter().chain(absent).chain(unread).collect();
        }
        let mut entry_added = false;
        for entry in carry.approved.iter().filter_map(|(_, e)| e.as_ref()) {
            if !view.plan.iter().any(|p| p.id == entry.id) {
                view.plan.push(entry.clone());
                entry_added = true;
            }
        }
        if entry_added {
            let items = &view.items;
            let place = |id: &str| items.iter().position(|i| i.job == id).unwrap_or(usize::MAX);
            view.plan.sort_by_key(|e| place(&e.id));
        }
        for item in &mut view.items {
            if let Some((read, _)) = carry.prepared.get(&item.job) {
                item.read = *read;
            }
        }
        for entry in &mut view.plan {
            if let Some((_, stored)) = carry.prepared.get(&entry.id)
                && !entry.missing().is_empty()
            {
                *entry = stored.clone();
            }
        }
        let mut forced: Vec<Refusal> = carry.forced.clone();
        // Items on a line the scope guard has paused: asked once for the whole queue, before any read. Such an
        // item is not read to fill its plan, and is held back below.
        let paused: BTreeMap<String, String> = {
            let ids: Vec<String> = view.items.iter().map(|i| i.job.clone()).collect();
            self.steps.paused(&ids).into_iter().collect()
        };
        for idx in 0..view.items.len() {
            // An interrupted job was read before it ran; its retry reads the request again.
            if view.items[idx].read || view.items[idx].interrupted {
                continue;
            }
            let id = view.items[idx].job.clone();
            if paused.contains_key(&id) {
                continue;
            }
            // Read by an earlier plan of this run: not read or paid for again.
            if carry.prepared.contains_key(&id) {
                continue;
            }
            let Some(pos) = view.plan.iter().position(|e| e.id == id) else {
                continue;
            };
            if view.plan[pos].missing().is_empty() {
                continue;
            }
            if self.steps.stop_requested() {
                break;
            }
            let room = {
                let st = self.st();
                st.started < self.limits.max_jobs && st.budget.fits(reserve)
            };
            if !room {
                self.note_once(format!(
                    "item {id} was not read to fill its plan: no room in the job limit or the budget"
                ));
                continue;
            }
            self.ui.line(&format!(
                "plan       {id} is underspecified ({}); reading it",
                view.plan[pos].missing().join(", ")
            ));
            let read = self.steps.read(ReadOf::Queued(&id));
            {
                let mut st = self.st();
                st.budget.add(read.cost);
                st.read_cost.insert(id.clone(), read.cost);
            }
            if let Some(entry) = self.steps.plan_entry(&id) {
                view.plan[pos] = entry;
            }
            let title = view.plan[pos].title.clone();
            if read.job.is_none() || read.error.is_some() {
                let why = read.error.as_deref().unwrap_or("no job was made");
                let mut r = Refusal::lacking(&id, &title, vec!["reading".into()]);
                r.reason = format!("the request could not be read: {why}");
                self.remember_reading(&id, false, &view.plan[pos], Some(&r));
                forced.push(r);
                continue;
            }
            view.items[idx].read = read.runnable;
            let mut refusal = None;
            if !read.runnable && view.plan[pos].missing().is_empty() {
                let mut r = Refusal::lacking(&id, &title, vec!["capability".into()]);
                r.reason =
                    "it was read, but no capability fits the request, so there is nothing to run"
                        .into();
                refusal = Some(r.clone());
                forced.push(r);
            } else if !view.plan[pos].missing().is_empty() {
                self.note(format!(
                    "item {id} was read, but its plan is still incomplete; correct or answer it with `athena correct {id}`"
                ));
            }
            self.remember_reading(&id, view.items[idx].read, &view.plan[pos], refusal.as_ref());
        }
        // A paused item is held back, and so is everything that depends on it, as `Plan::build_with` holds the
        // dependents of an item it holds: a dependency on an id outside the entries would count as met.
        let mut held: BTreeMap<String, String> = paused
            .into_iter()
            .filter(|(id, _)| !forced.iter().any(|f| &f.id == id))
            .collect();
        loop {
            let more: Vec<(String, String)> = view
                .plan
                .iter()
                .filter(|e| !held.contains_key(&e.id) && !forced.iter().any(|f| f.id == e.id))
                .filter_map(|e| {
                    let on = e.depends_on.iter().find(|d| held.contains_key(*d))?;
                    Some((
                        e.id.clone(),
                        format!("depends on {on}, which is held back: its line is paused"),
                    ))
                })
                .collect();
            if more.is_empty() {
                break;
            }
            held.extend(more);
        }
        let entries: Vec<PlanEntry> = view
            .plan
            .iter()
            .filter(|e| !forced.iter().any(|f| f.id == e.id) && !held.contains_key(&e.id))
            .cloned()
            .collect();
        // Items over a priority cap are demoted one tier in memory; the order is judged against the tiers it
        // was built from, so a demotion is not reported as a contradiction.
        // Athena's own improvement requests get a guaranteed turn: the run's count of dispatches since the last
        // one, and what it has dispatched, decide whose place is due.
        let mut improvement = self.steps.improvement();
        {
            let mut st = self.st();
            improvement.since_turn = st.since_turn;
            improvement.tried = st.tried.clone();
            st.improving = improvement.items.iter().cloned().collect();
        }
        let (mut plan, entries) = Plan::build_improving(
            &entries,
            &self.steps.job_history(),
            self.steps.priority_caps(),
            &self.steps.person_set(),
            &improvement,
        );
        for contradiction in plan.priority_violations(&entries) {
            self.note_once(format!("plan order contradicts priority: {contradiction}"));
        }
        for d in &plan.demoted {
            let title = entries
                .iter()
                .find(|e| e.id == d.id)
                .map_or("", |e| e.title.as_str());
            self.note_once(format!(
                "demoted {} · {title} from {} to {}: over the {} cap of {}",
                d.id,
                d.from.as_str(),
                d.to.as_str(),
                d.from.as_str(),
                d.cap
            ));
        }
        plan.refused.extend(forced);
        // In queue order, so the list does not depend on the map.
        for (id, title) in view.items.iter().map(|i| (&i.job, &i.title)) {
            if let Some(reason) = held.get(id) {
                let title = view
                    .plan
                    .iter()
                    .find(|e| &e.id == id)
                    .map_or(title.as_str(), |e| e.title.as_str());
                plan.refused.push(Refusal::held(id, title, reason));
            }
        }
        self.st().pause_held = held;
        plan
    }

    /// Counts the items of `wave` as dispatched, in wave order: an improvement request restarts the count the
    /// plan's guaranteed turn is measured from, any other item adds to it. An improvement request that `plan`
    /// (the plan the wave came from) gave a turn is recorded for the report with its title from `entries`.
    pub(super) fn note_dispatch(&self, wave: &[String], plan: &Plan, entries: &[PlanEntry]) {
        let mut st = self.st();
        for id in wave {
            st.dispatched += 1;
            if !st.improving.contains(id) {
                st.since_turn += 1;
                continue;
            }
            st.since_turn = 0;
            if let Some(turn) = plan.turns.iter().find(|t| &t.id == id) {
                let title = entries
                    .iter()
                    .find(|e| &e.id == id)
                    .map_or_else(String::new, |e| e.title.clone());
                let dispatch = st.dispatched;
                st.improvement_turns.push(ImprovementTaken {
                    job: id.clone(),
                    title,
                    dispatch,
                    moved: turn.moved,
                });
            }
        }
    }

    /// One "passed over" line for each item the last plan held for a paused line, unless this run has worked it.
    /// It names the line and how to lift the pause, and is printed as soon as the plan is made, before any other
    /// item runs.
    pub(super) fn pass_over_paused(&self) {
        let (held, tried) = {
            let st = self.st();
            (st.pause_held.clone(), st.tried.clone())
        };
        for (id, reason) in held.iter().filter(|(id, _)| !tried.contains(*id)) {
            self.pass_over(id, reason);
        }
    }

    /// Records that the planning pass read `id`, so a later plan of the run neither reads nor pays for it again.
    pub(super) fn remember_reading(
        &self,
        id: &str,
        read: bool,
        entry: &PlanEntry,
        refusal: Option<&Refusal>,
    ) {
        let mut st = self.st();
        st.carry
            .prepared
            .insert(id.to_string(), (read, entry.clone()));
        st.carry.forced.extend(refusal.cloned());
    }

    /// At the end of the run, one "passed over" line for every item of the last plan that was not worked and
    /// has none yet: the refused (which includes the held and the unreadable), the cyclic, an item of the last
    /// wave that never started, and the planned items the run ended before reaching. `wave` is the ids of
    /// the last wave worked. An item is logged only now, not when first seen, because a new plan can make it
    /// workable.
    pub(super) fn sweep(&self, view: &QueueView, plan: &Plan, wave: &[String]) {
        let tried = self.st().tried.clone();
        for refusal in &plan.refused {
            if !tried.contains(&refusal.id) {
                self.pass_over(&refusal.id, &refusal.reason);
            }
        }
        for id in plan.cyclic.iter().filter(|id| !tried.contains(*id)) {
            self.pass_over(
                id,
                &format!(
                    "it waits on other items in a loop ({}), so none of them is worked",
                    plan.cyclic.join(", ")
                ),
            );
        }
        let why = self.st().stop.map_or(Stop::QueueEmpty.label(), Stop::label);
        let not_started: Vec<&String> = {
            let st = self.st();
            wave.iter().filter(|id| !st.done.contains(*id)).collect()
        };
        for id in not_started {
            self.pass_over(id, &format!("run ended first: {why}"));
        }
        let entry_of = |id: &str| view.plan.iter().find(|e| e.id == id);
        let last_wave: Vec<&PlanEntry> = wave.iter().filter_map(|id| entry_of(id)).collect();
        for batch in batches_for(&view.items, &view.plan, plan, &tried) {
            for item in batch {
                let mut reason = format!("run ended first: {why}");
                let clash = entry_of(&item.job).and_then(|mine| {
                    last_wave
                        .iter()
                        .find_map(|other| shared_path(mine, other).map(|p| (p, other.id.clone())))
                });
                if let Some((path, other)) = clash {
                    reason.push_str(&format!(
                        "; it shares {path} with {other}, so it was waiting for it"
                    ));
                }
                self.pass_over(&item.job, &reason);
            }
        }
    }

    /// Rates the page after a landing. Returns the stop reason when the rating ends the run.
    pub(super) fn rate(&self) -> Option<Stop> {
        let _one_at_a_time = self.landing.lock().unwrap_or_else(PoisonError::into_inner);
        {
            let mut st = self.st();
            if !st.budget.fits(self.caps.rate) {
                return Some(Stop::BudgetSpent);
            }
            st.budget.hold(self.caps.rate);
        }
        let rate = self.steps.rate();
        {
            let mut st = self.st();
            st.budget.release(self.caps.rate);
            st.budget.add(rate.cost);
        }
        match rate.rating {
            Some(value) => {
                // The only place a rating of this run is made, and it is made after a landing.
                let (after_landing, outcome_landings) = {
                    let mut st = self.st();
                    let after_landing = !st.landed.is_empty();
                    st.held_back = None;
                    st.rating = Some(RatingLine {
                        value,
                        after_landing,
                        met_target: false,
                    });
                    (after_landing, st.outcome_landings)
                };
                let rated_enough = after_landing && value >= self.limits.target_rating;
                if !rated_enough {
                    return None;
                }
                // A rating after landings that left the page's sources alone says nothing new about the page.
                if outcome_landings == 0 {
                    self.ui.line(&format!(
                        "drain      rating {value} reached the target but no landing changed the outcome"
                    ));
                    return None;
                }
                // A rating cannot catch a wrong fact, so it ends the run only with a passing source check. The
                // check is a model call per study, so its whole cap must fit before it starts.
                let cap = self.steps.check_cap();
                {
                    let mut st = self.st();
                    if !st.budget.fits(cap) {
                        st.notes.push(format!(
                            "the source check was not made: its cap of ${cap:.2} does not fit what is left of the budget"
                        ));
                        return Some(Stop::BudgetSpent);
                    }
                    st.budget.hold(cap);
                }
                let check = self.steps.check_page();
                {
                    let mut st = self.st();
                    st.budget.release(cap);
                    st.budget.add(check.cost);
                }
                if !check.passed {
                    self.ui.line(&format!(
                        "drain      rating {value} reached the target but the source check failed: {}",
                        check.detail
                    ));
                    self.st().held_back = Some(check.detail);
                    return None;
                }
                self.st().rating = Some(RatingLine {
                    value,
                    after_landing,
                    met_target: true,
                });
                Some(Stop::TargetMet)
            }
            None => {
                self.note(format!(
                    "the page could not be rated: {}",
                    rate.error.as_deref().unwrap_or("no rating")
                ));
                None
            }
        }
    }
}
