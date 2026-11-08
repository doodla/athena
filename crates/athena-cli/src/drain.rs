//! `athena drain`: works the queue itself (requests the steward queued and requests deferred with `athena ask --later`), within a budget the user sets. For each queued request
//! it reads and runs the job, lands it through the pull-request path (`athena land --pr`), and only then
//! rates the live page as its audience would. It stops at the first of the reasons in [`Stop`].
//!
//! - **Planning comes first.** The open requests are fleshed out and planned together
//!   ([`athena_core::plan`]): goal, requirements-ladder rung, acceptance, open questions, dependencies and
//!   touched paths. The loop works the items in plan order (a dependent after what it
//!   needs, then priority from a `Priority:` line in the request, then higher rung, then age as the tiebreak),
//!   not oldest first. An item whose plan lacks a goal, a rung or acceptance is
//!   refused and reported with a reason ([`Report::refused`]). The plan also groups items that could run in
//!   parallel ([`Report::plan_groups`]); [`drain_concurrent`] runs the items of one group side by side, up to
//!   `[parallel] max_concurrency` (default 1, which is [`drain`]'s one job at a time). The pass is
//!   deterministic code and calls no model of its own; nothing is defaulted into a plan.
//! - **Athena's own requests get a guaranteed turn.** Requests from the Steward and from Hestia's reviews
//!   ([`Steps::improvement`]) used to sort behind every request a person marked urgent or high. The plan now
//!   gives them at least one dispatch in three while a ready one is queued
//!   ([`athena_core::plan::Plan::build_improving`]); the drain counts its dispatches (`State::since_turn`) so each
//!   new plan keeps the gap. A cause recorded in three distinct jobs makes its request urgent. `athena plan`
//!   prints each `turn` and `promoted` item; the report lists the turns taken ([`Report::improvement_turns`]).
//! - **The drain follows the plan as it stands.** The drain does not keep the plan it built at its start. After
//!   each item (each group, under [`drain_concurrent`]) it reads the queue again and builds the plan again
//!   through the same function `athena plan` uses (`Drain::prepare`, which calls
//!   [`athena_core::plan::Plan::build_with`]), so a priority record, a job end or a request queued meanwhile
//!   changes what is worked next. [`batches_for`] only filters that plan's groups: it drops what this run has
//!   already tried and never reorders. Memory kept across these plans: the jobs the drain approved, the
//!   readings it made, and the items it refused to read, so nothing is approved, read or paid for twice.
//!   *Passed-over log line.* An item that is planned but not worked gets one line, `plan       <id> passed
//!   over: <reason>`, once per run: refused, held by a failed dependency, in a loop of dependencies, not
//!   ready (needs accepting), held for disk or budget, skipped, over the budget, over the job limit, or not
//!   reached because the run ended first (with the path it shares with the last group when that is why it
//!   waited).
//! - **A paused line is passed over, not failed.** [`Steps::paused`] says which queued items are on a line the
//!   scope guard has paused. `Drain::prepare` asks once for the whole queue, before it reads anything: such an
//!   item is not read, is held back with whatever depends on it (so `athena plan` lists it as `held`), and its
//!   one passed-over line, which names the line and how to lift the pause, is printed as soon as the plan is
//!   made. [`Drain::attempts`] asks again before each attempt ([`ItemEnd::Paused`]), and once more after the
//!   last failed attempt, for a line the item's own failed attempt has just paused. Neither is a failed
//!   attempt: an item whose second failure paused its line is passed over with the pause reason, and the
//!   batch goes on to the next queued item.
//! - **A flagged line is passed over after two failures.** A line the scope guard has only flagged (T1, T3 or
//!   T5, not a pause) is still dispatched, so its item gets both attempts. [`Steps::flagged`] is asked only
//!   after the last failed attempt, when no pause was found ([`ItemEnd::Flagged`]): the item is then passed
//!   over for the rest of the batch with the flag signals as the reason, and the batch goes on to the next
//!   queued item. Nothing is written.
//! - **An item that fails twice is passed over, not a reason to stop.** An item whose line is neither flagged
//!   nor paused and whose second attempt fails ([`ItemEnd::Failed`]) is passed over for the rest of the batch
//!   with the reason `failed twice; left for the scope guard and Hestia: <last reason>`, and the batch goes
//!   on to the next queued item. Both jobs stay in the records, so the scope guard (which pauses or flags a line that
//!   keeps failing) and Hestia (which lists repeated failures) see the failure as before; the drain adds no
//!   verdict of its own. The one exception is not about the item: when the closing reason of that second
//!   failure says the environment is at fault ([`athena_core::disk::StopCause`]: the producer's login
//!   expired, a usage limit was reached, or free disk space is low), every later item would fail the same
//!   way, so the run also ends with [`Stop::Environment`]. Before this rule an item that failed twice with no
//!   pause or flag ended the whole batch (a 4 Nov batch with $43 and 3 job slots left stopped on one such
//!   item).
//! - **Concurrency.** A group is worked by up to `max_concurrency` workers; the next group starts only when
//!   the group before it is done, so an item that depends on another starts after it has finished. Landing
//!   and rating go one at a time (a lock), and every model call's cap is held against the budget while it is
//!   in flight, so jobs side by side cannot overspend it. The first reason to stop ends the run: no worker
//!   starts another job, jobs already started are finished (a failure or a refusal stops no sibling that is
//!   in flight), and the first reason found is the one reported. A worker that panics is not isolated: the
//!   drain ends with the panic once the other workers have finished.
//! - **Read-ahead.** When [`Steps::read_ahead`] is above 0, with one job at a time as with several,
//!   [`drain_concurrent`] also reads, and for a hard job plans ([`Steps::plan_ahead`]), the next items up to
//!   `read_ahead` of them ahead of the batch that is running (`docs/design/async-drain.md` section 4). It
//!   reads an item ahead only when the paths the plan says it touches are disjoint from every job of the
//!   running batch that has not finished, by the rule the plan groups by: a path equals another or lies inside
//!   it as a directory, and the shared append-only files (`CHANGELOG.md`, `docs/decisions.md`, `README.md`)
//!   are ignored. An item whose touched paths are unknown is not read ahead. Reads and plans made ahead are
//!   charged to the budget like any other, start only when a read and a run both fit it, count against the
//!   job limit, and a reading is run as it is,
//!   never read a second time. *Stale readings:* a landing may change what a reading assumed. When the job
//!   starts, the paths landed since its reading began are compared with the paths the reading touches by the
//!   same rule; on a hit the drain notes it ([`Report::notes`]) and asks [`Steps::recheck`]. If the reading no
//!   longer holds, [`Steps::reread`] makes a fresh one, and a reading whose paths no landing touched is used
//!   as it is. How many readings are held or being made at once is bounded by [`Steps::read_ahead`]. With
//!   `read_ahead` 0 nothing is read ahead and the drain behaves as it always did. [`drain`] reads each item
//!   at its turn, because its steps are not `Sync` and so cannot be read from a second thread. The real steps
//!   do not turn this on yet.
//! - **Checks run again after a rebase only when it touched what the candidate changed.** Under the landing
//!   lock, before a verified job lands, the drain asks [`Steps::rebase`] what rebasing the job onto the latest
//!   main moved (`docs/design/async-drain.md` section 6, decision D145, build job 7). The pure
//!   [`rebase_needs_recheck`] decides: yes when a path the rebase touched overlaps a path the candidate changed
//!   (the rule of [`stale_paths`]) or is a `Cargo.toml` or `Cargo.lock` at any depth; otherwise the job lands
//!   with no checks run again. The checks are [`Steps::rerun_checks`], with [`Steps::rerun_cap`] held against
//!   the budget first; a cap that does not fit ends the run as [`Stop::BudgetSpent`] without a failure
//!   counted. A re-check that fails is owed again on the landing retry (the branch is then already moved, so
//!   `rebase` answers [`Rebased::Unchanged`]), and a job that fails it twice ends like any landing that fails
//!   twice. Every rebase that moved a branch is counted, and so is every one that needed a re-check: the share
//!   is [`Report::rebases`] and a line in [`Report::notes`] naming the rule. It is kept for the run only. With
//!   one job at a time nothing changes unless `rebase` answers `Moved`. The real steps keep the default
//!   `Unchanged` until the drain wiring of the landing rebase exists, so the share is not measured in a live
//!   drain yet; a rebase conflict is that wiring's matter.
//! - **The pass reads unread requests that are underspecified.** An unread request whose own words give no
//!   acceptance (or no rung) is read first, within the budget and job limit, and planned again from its
//!   reading. If the plan is still incomplete it is refused with the reason, so the queue
//!   does not fill with unread items that are refused merely for being unread; one that is specified after
//!   its reading runs without a second reading. [`plan`] runs the same pass for `athena plan` and works
//!   nothing, so its `--budget` only has to fit each read's cap, not a run after it.
//! - **A read request stays queued.** A request is open until it is run or closed. One that `athena plan` (or
//!   a person) has read waits in state proposed and is still in the queue, marked read, when its latest
//!   stored reading passes the text-only reading checks (`check_reading_text`; the check of the deliverable
//!   paths against the repository is not repeated): [`drain`] runs it without reading it again, in plan
//!   order, and a second `athena plan` lists it again. This holds for a request queued with `ask --later`
//!   or by the steward or Hestia, as for any other. A proposed request with no stored reading (its reader
//!   failed, or both its readings were refused) is queued unread: the drain reads it in plan order, runs it
//!   when the reading is accepted, and skips it after two empty reads. Two kinds of proposed request wait
//!   for a person to run `athena go` and are left out of the queue: one whose stored reading fails a
//!   reading check, and one that was read again after a settings change with a changed scope (a reading
//!   from `athena correct` is not held back). Only a request that was declined, is running, or has ended,
//!   and one on another channel, is also left out ([`athena_core::steward::queued_work`]).
//! - **An interrupted job is retried.** A job that was dispatched, has no closing record and has no process
//!   working on it (a machine restart killed it) is listed as work like a queued request, on a drained
//!   channel only ([`athena_core::steward::interrupted_work`]). Its first attempt is `athena retry`, which
//!   reads the request again as a new job; that job is then run. The interrupted job is left as it is, but a
//!   job that a retry already names is not listed again. A job whose process is still running is never listed.
//!   A job held for low disk space after its candidate was built (see "Low disk holds a job") is listed the
//!   same way, so the next pass resumes it with no one running `athena retry`.
//! - **The in-flight table.** The real steps keep a local file, `in-flight.json` in the state directory, with
//!   one row per job being worked: job id, stage and worktree ([`InFlight`]). The stage is reported by the run
//!   itself where it enters a stage (`athena_core::state::StageSink`, called by `Athena::go_with_stages`):
//!   producer (again on every send-back after a failed self-check), checks, verifier, and landing. The drain
//!   matches no output text to choose a stage. The worktree is the producer's live work tree
//!   (`StateDir::work_dir`), not the kept work of a timed out job. A rejected or failed job's row waits for
//!   its retry: the retry's reading moves the row to the new job and keeps its stage, and the retry's
//!   producer then sets it to producer again. The row is removed when the item's work ends, and after a
//!   landing. The drain reads the file at its start and notes any row left by an earlier run
//!   ([`Report::notes`]); it acts on none (recovery is later work). A missing file is empty. A file that
//!   cannot be read is moved aside to `in-flight.json.corrupt`, noted, and the drain goes on. A write that
//!   fails is a warning line, never a job failure.
//! - **A pull request waiting on checks is not a failure.** The landing waits while any check run on the pull
//!   request head is queued or in progress, and a poll that turns green merges in the same locked step: the
//!   one land call polls the checks and merges as soon as they are green (including after a merge the host
//!   refused for its repository rules before every check finished), so the job is [`Landing::Landed`] and
//!   is not left for a later call. Only a poll that ends with the pull request still open and its checks
//!   pending is [`Landing::Waiting`]: the job line says `waiting to merge`, the item is not tried again and
//!   does not count as a failure, and the run goes on to the next item. `athena land --pr` merges
//!   it once the checks have finished.
//! - **Stop at a job boundary.** [`Steps::stop_requested`] is asked before every job. A job that has started
//!   is finished (landed and rated) and no further job starts: [`Stop::Requested`]. Nothing is killed.
//!
//! - **Athena rebuilds itself after landing its own code.** A landing that changed `crates/`, `xtask/`,
//!   or any other path in [`BUILD_PATHS`] ([`touches_build`]) leaves the installed binary behind the
//!   checkout, and the next config read or landing could fail on it. So before the next job starts, the loop
//!   calls [`Steps::rebuild`] (the real one runs `setup.sh` in the checkout, which builds from the landed tree
//!   and installs the binary). If that fails the run ends with [`Stop::RebuildFailed`] and
//!   [`Report::rebuild_output`] holds the build output. If it succeeds the pass ends with [`Stop::Rebuilt`] and
//!   [`Report::restart`] holds what is left of the budget and job limit; [`follow_restart`] hands those to the
//!   new binary (`athena drain` again, see [`reexec_args`]), which carries on with the queue. A stop asked for
//!   meanwhile wins, and a run with no budget or no job limit left ends as such instead of restarting.
//! - **A landed config the running build cannot read is not a failure.** Right after a landing the drain asks
//!   [`Steps::config_after_landing`]. The config was readable when the run started, so if it is not now, the
//!   landing added something only a newer build understands (a new key, for instance). The job stays
//!   `landed`, no failure is counted, and the run ends with [`Stop::NeedsReinstall`] before another item
//!   starts; a note carries the read error and says to reinstall Athena and run the drain again. If a rebuild
//!   runs after that landing, the new binary reads the config and the stop is [`Stop::Rebuilt`].
//!   The root cause was that a landing never checked that the running build can read the config it lands, and
//!   the first fix stored the fact only as the stop reason, which keeps the first reason found: another
//!   worker's earlier stop hid it, and the page was rated against the unreadable config. The fact is now kept
//!   in the state: while it is set, no rating is taken, and the
//!   report ends with [`Stop::NeedsReinstall`] and the note whatever stop was stored first (only a failed
//!   rebuild keeps its own stop). A stored [`Stop::Rebuilt`] becomes a reinstall too, because that rebuild ran
//!   before the landing. `athena land` asks the same question (`JobConfig::unreadable_at`) of the merged
//!   commit.
//!
//! The loop ([`drain`]) holds every rule and calls the world through [`Steps`], so tests drive it with a
//! stub and no model, no GitHub and no network:
//!
//! - **Rate after landing.** A rating is only requested after a landing in this run, and the rating line
//!   records that it was ([`RatingLine::after_landing`]). A run that landed nothing never reports "target
//!   met": it shows the last recorded rating, marked as not taken after a landing, and says the page is
//!   unchanged.
//! - **Budget.** Every model call is guarded: the money already spent, by what the calls actually cost and not
//!   by what they were expected to cost, plus the call's cap must fit the budget ([`Budget::fits`]). That covers
//!   a job's reading, its run, the re-reading of a retry, and the rating. A job is started only when its
//!   reading and its run both fit, so a request is not left read and never run for want of money.
//! - **A rating alone does not end the run.** "Target met" needs two more things, both by code:
//!   1. *An outcome-changing landing.* A rating after landings that touched only other paths (for example
//!      docs or the loop's own code) rates a page that did not change. At least one landing in the run must
//!      have changed a path in [`OUTCOME_PATHS`]: `studies/`, the site generator (`xtask/`) or the publish
//!      workflow. Otherwise drain keeps running and the report says the landing did not change the outcome
//!      ([`Report::outcome_changed`]). The paths come from the landed job's candidate record.
//!   2. *A source check of all page text.* A reader who sees only the page cannot catch a wrong fact, so
//!      [`Steps::check_page`] must pass. The real check first runs the model-free floor
//!      ([`model_contradictions`] over each study's `model.json`), then reads every file of every study at the
//!      head commit except the structural ones (`athena_core::pin::STRUCTURAL_FILES`: the pin and the rule
//!      results) against that study's pinned source with one model call per study
//!      (`athena_core::Athena::check_study_text`). That covers `claims.json`, `intro.json` (intro, transition
//!      and glossary) and any text file added later. A contradicted or omitted statement fails it. So does a
//!      `behavior` problem: the checker also traces the study's model (`model.json`: states, where transitions
//!      end, every event that changes a counter, rules and their waivers) through the pinned code, and
//!      compares the page's rules with each other. Quote existence was not enough: an independent reader
//!      scored the page 3 where Athena's own check allowed 4, four times running, and found model errors
//!      (a reopened backlog item shown as finished although the source returns it to Queued, a counter
//!      that missed a failed close freeing the place, a red-PR rule contradicting a waiver on the same page).
//!      When it fails, drain keeps running and the report names the statement ([`Report::held_back`]).
//!
//!   The source check is a model call per study, so it is guarded like the others: the reserve is
//!   [`Steps::check_cap`] (the verifier's cap times the number of studies), it must fit the budget or drain
//!   stops as "budget spent", and what the calls cost is added to the spend ([`Check::cost`]).
//! - **Retry.** An item gets two attempts (an interrupted job's first attempt is its retry, see above). After a rejected or failed run the second attempt reads the job
//!   again with `athena retry`, then runs it. The retry's producer starts from the base, except after a
//!   rejection with every check passed and either only content criteria unmet or no unmet criterion and only
//!   documentation findings blocking, when it starts from the rejected
//!   candidate (`athena_core::reading::qualifies_for_tree_retry`). A run that ended any other way (no outcome, blocked) leaves
//!   the job proposed or blocked, which `athena retry` refuses, so the second attempt runs the same reading
//!   again. After a run that passed but did not land, it lands again. Two failures stop the loop, naming the
//!   item.
//! - **A configuration change.** A reading stays valid across a change that does not bear on it (for
//!   example `[parallel] max_concurrency`; see `athena_core::run::GoResult::Reread`). When a relevant change
//!   makes `athena go` read the job again, drain accepts the new reading if its scope (capability, class
//!   of work, deliverables) is unchanged and runs it in the same drain. If the scope changed, the item is
//!   left queued with the reason (outcome `needs accepting`, plus a note); that is not a failure and stops
//!   nothing.
//! - **Interrupted jobs give up their build directories.** At the start of a drain and before each run, drain
//!   removes `target/` of every job that is dispatched with no closing record, has no process in the registry
//!   and whose job lock is free ([`athena_core::disk::reclaim_interrupted_targets`]). Work trees, kept
//!   candidates, resume marks and records stay, so the retry resumes as before. A job that another command is
//!   running keeps its build directory. The sweep reads the records once per call; a job another command
//!   dispatches in that instant is protected by its job lock. This is [`Steps::reclaim_interrupted`].
//! - **Low disk holds a job.** When free disk space is below `[disk] min_free_gb` (default 10) at dispatch,
//!   before the producer (after the baseline checks), before the checks or before the verifier, even after idle jobs' build directories are removed,
//!   `athena go` returns [`athena_core::disk::LowDisk`]. The item ends held (outcome `environment: low disk`,
//!   with the free and required amounts, plus a note); that is not a failure, uses no attempt, and stops
//!   nothing. A hold at dispatch leaves the job queued, and a later drain runs it. A hold before the producer
//!   leaves the job dispatched with no candidate and no resume mark; the next pass retries it from the start,
//!   with the producer never having run. A hold after the candidate
//!   was built leaves the job dispatched with no process, so the next pass lists it as interrupted and
//!   resumes it from its kept candidate by itself, with no planner or producer run and no one running `athena
//!   retry`. That skips the rebuild only while the main branch has changed nothing outside `records/` since
//!   the job's base; otherwise the retry plans and produces again, as any retry does. Space that stays short
//!   holds the job again.
//! - **A build-slot wait holds a job.** Each local check takes one of `[drain.pipeline] max_check_slots` build
//!   slots ([`athena_core::slots`]). A check that waits longer than `build_wait_s` (default 1800) makes `athena
//!   go` return [`athena_core::disk::SlotWait`]. The item ends held (outcome `environment: build slot wait`, not
//!   `environment: low disk`, plus a note); that is not a failure, uses no attempt, and stops nothing. The job
//!   stays dispatched with no process and its kept candidate, so the next pass lists it as interrupted and
//!   resumes it by itself.
//! - **A short budget holds a job.** A job's own budget (`[budget] job_usd`) must cover everything still to run
//!   (`athena_core::reading::budget_reserve`). When it does not, `athena go` returns
//!   [`athena_core::reading::BudgetShort`] before spending anything. The item is left queued (outcome
//!   `budget short`, with the shortfall and the config line, plus a note); that is not a failure, is not
//!   retried, and the next item runs.
//! - **In-limit proposals are approved, approved jobs are worked.** An open proposal whose rung the role that
//!   raised it may change itself (its newest recorded definition decides; a role with no recorded definition
//!   has no limit) is approved as `athena approve-work` would, which reads the job and records the decision.
//!   Jobs that a work decision records as approved and that are still in state proposed are run, once, without
//!   a second reading; a closed job is never picked up again. An approval is a model call, so it is guarded by
//!   the budget and the job limit like any other. An approved job is not worked first: it enters the plan
//!   like any other item (a job approved in this pass gets its plan entry right after the approval) and is
//!   ordered by the same rules, dependency, then priority, then rung, then age. Its priority is normal unless
//!   a priority record (`athena priority`), the job's own words or the standing instruction of the role that
//!   raised it (a `Priority: <level>` line) say otherwise ([`athena_core::plan::proposal_priority`]), so an
//!   urgent queued request is worked before an old proposal, and the job limit and budget apply to it as to
//!   the rest.
//! - **People decide the rest.** Rejecting a proposal, and approving one above its role's limit (or from an
//!   unknown role), stay with people. Such proposals are listed as waiting, and [`Steps`] has no way to reject
//!   one or to approve one that is not in limit.

use athena_core::audience::{LivePage, Page, Target};
use athena_core::disk::StopCause;
use athena_core::gh::{Checks, GhCli, PrReport};
use athena_core::plan::{JobEnd, Plan, PlanEntry, Refusal, shared_path};
use athena_core::pr::LinkState;
use athena_core::state::{Stage, StageSink};
pub use athena_core::status::{BUILD_PATHS, touches_build};
use athena_core::store::Records;
use athena_core::{Athena, Ui, steward, work_decisions};
use athena_data::JobId;
use athena_data::records::{AcceptedBy, OutcomeState};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

/// The rating at which the page counts as good enough, unless the user says otherwise.
pub const DEFAULT_TARGET_RATING: u8 = 4;

/// The most jobs one drain starts, unless the user says otherwise.
pub const DEFAULT_MAX_JOBS: u32 = 5;

/// How many times one item is tried before the loop gives up on it.
const ATTEMPTS: u32 = 2;

/// What the user set.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    /// The most money to spend on model calls in this run, in US dollars.
    pub budget_usd: f64,
    /// The rating, from 1 to 5, at or above which the target is met.
    pub target_rating: u8,
    /// The most jobs to start. A retry of an item is a job.
    pub max_jobs: u32,
}

/// The most one model call of each kind may cost: the caps in `athena.toml`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Caps {
    pub read: f64,
    pub run: f64,
    pub rate: f64,
}

/// Money spent against a budget. What is added is what a call cost, which may be more than its cap.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Budget {
    total: f64,
    spent: f64,
    /// Caps reserved by calls that are in flight and have not yet reported their cost. With one job at a time
    /// this is zero whenever a call is about to start.
    held: f64,
}

impl Budget {
    pub fn new(total: f64) -> Budget {
        Budget {
            total,
            spent: 0.0,
            held: 0.0,
        }
    }

    /// Whether a call capped at `cap` may start: what is spent, plus the caps of the calls in flight, plus
    /// this cap must not exceed the budget.
    pub fn fits(&self, cap: f64) -> bool {
        self.spent + self.held + cap <= self.total + 1e-9
    }

    pub fn add(&mut self, cost: f64) {
        self.spent += cost.max(0.0);
    }

    /// Reserves the cap of a call that is starting, so a call running beside it counts it in [`Budget::fits`].
    pub fn hold(&mut self, cap: f64) {
        self.held += cap;
    }

    /// Gives a reservation back once the call has ended and its real cost was added.
    pub fn release(&mut self, cap: f64) {
        self.held = (self.held - cap).max(0.0);
    }

    pub fn spent(&self) -> f64 {
        self.spent
    }
}

/// A queued request the loop may work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Item {
    pub job: String,
    pub title: String,
    /// True when the job was already read (a job waiting in state proposed whose stored reading passes the
    /// reading checks, or an approved job), so it is run without a reading. A proposed request with no stored
    /// reading is not marked read: the drain reads it. A retry after a failed run still reads again.
    #[serde(skip)]
    pub read: bool,
    /// True when the job was interrupted (dispatched, no process, no closing record): its first attempt is
    /// `athena retry`, not a reading of the request.
    #[serde(skip)]
    pub interrupted: bool,
}

impl Item {
    /// A queued request that has not been read.
    pub fn queued(job: &str, title: &str) -> Item {
        Item {
            job: job.into(),
            title: title.into(),
            read: false,
            interrupted: false,
        }
    }
}

/// An open proposal. It is waiting for a person unless its rung is within its role's limit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Waiting {
    pub id: String,
    pub title: String,
    pub rung: String,
}

/// What is in the queue when the loop starts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QueueView {
    /// Jobs to work: approved jobs already read, then queued requests.
    pub items: Vec<Item>,
    /// Open proposals above their role's limit, or from an unknown role: they need a person.
    pub waiting: Vec<Waiting>,
    /// Open proposals within their role's limit: the loop approves them.
    pub approvable: Vec<Waiting>,
    /// The fleshed-out plan entry of each open request, oldest first (see [`athena_core::plan`]). The loop
    /// plans them together and works their items in plan order. An item with no entry keeps its place ahead
    /// of the planned items.
    pub plan: Vec<PlanEntry>,
}

/// The result of approving a proposal: the job that carries it out, which is read and waits to run.
#[derive(Debug, Clone, Default)]
pub struct Approval {
    pub job: Option<String>,
    pub cost: f64,
    pub error: Option<String>,
}

/// What to read: a queued request for the first time, or a failed job again (`athena retry`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadOf<'a> {
    Queued(&'a str),
    Retry(&'a str),
}

/// The result of a reading.
#[derive(Debug, Clone, Default)]
pub struct Read {
    /// The job that was read; a retry is a new job. None when the reading could not be made.
    pub job: Option<String>,
    pub cost: f64,
    /// Whether a capability fits, so the job can run.
    pub runnable: bool,
    pub error: Option<String>,
    /// True when the read was refused before any money was spent because the job's budget does not cover
    /// the whole path ([`athena_core::reading::BudgetShort`]). The error says by how much. The item is left
    /// queued: it is neither a failure nor tried again in this run, and the next item runs.
    pub held: bool,
}

/// The result of running a job to its end.
#[derive(Debug, Clone, Default)]
pub struct Run {
    /// None when the job did not reach an outcome.
    pub outcome: Option<OutcomeState>,
    pub reason: String,
    pub cost: f64,
    /// True when the job was read again and the new reading's scope differs from the one it replaces, so it
    /// is not run until a person accepts it. The job is still proposed: it is neither retried nor counted as
    /// a failure.
    pub needs_accepting: bool,
    /// True when the job was held for lack of free disk space (`environment: low disk`), because no build
    /// slot came free in time (`environment: build slot wait`), or because the job's budget does not cover
    /// the whole path still to run (`budget short`). The job is not retried in this run and is not counted as
    /// a failure; a later drain runs the same job once space or a build slot is free or the budget is raised.
    pub held: bool,
}

/// The paths whose change can change the explainer page: the studies it is written from (`studies/`, the
/// page sources), the site generator (`xtask/`) and the workflow that publishes the page. A landing counts
/// toward "target met" only if it changed at least one path under these.
pub const OUTCOME_PATHS: [&str; 3] = ["studies/", "xtask/", ".github/workflows/pages.yml"];

/// Whether a landing that changed `paths` can have changed the explainer page.
pub fn changes_outcome(paths: &[String]) -> bool {
    paths
        .iter()
        .any(|p| OUTCOME_PATHS.iter().any(|root| p.starts_with(root)))
}

/// What a rebuild did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rebuild {
    /// Athena was built from the landed tree and installed.
    Installed,
    /// The repository is not Athena's own source (it has no `setup.sh`), so there is nothing to rebuild.
    NotApplicable,
}

/// What is left of a run that stopped to rebuild: the new binary continues with this.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Restart {
    /// The budget not yet spent, in US dollars.
    pub budget_usd: f64,
    pub target_rating: u8,
    /// The job limit minus the jobs this pass started.
    pub max_jobs: u32,
}

impl Restart {
    pub fn limits(&self) -> Limits {
        Limits {
            budget_usd: self.budget_usd,
            target_rating: self.target_rating,
            max_jobs: self.max_jobs,
        }
    }
}

/// The arguments the new binary is run with to carry on: `athena drain` with the remaining budget and job
/// limit, and the same rating threshold.
pub fn reexec_args(limits: &Limits, json: bool) -> Vec<String> {
    let mut args = vec![
        "drain".to_string(),
        "--budget".into(),
        format!("{}", limits.budget_usd),
        "--rating-threshold".into(),
        limits.target_rating.to_string(),
        "--job-limit".into(),
        limits.max_jobs.to_string(),
    ];
    if json {
        args.push("--json".into());
    }
    args
}

/// The result of landing a verified job through the pull-request path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Landing {
    /// Merged and verified: the change is on the default branch. `changed` is every path the change touched.
    Landed { changed: Vec<String> },
    /// Anything short of that, with why. An open pull request is not a landing.
    NotLanded(String),
    /// The pull request is still open with its checks pending when the land step's poll ends: not a failure.
    /// A poll that turns green merges in the same locked step and is [`Landing::Landed`], never this. The
    /// drain reports the job as waiting to merge, does not try it again in this run, and goes on to the next
    /// item. `athena land --pr` (or a later run) merges it once every check has finished.
    Waiting(String),
}

/// The result of one audience rating of the live page.
#[derive(Debug, Clone, Default)]
pub struct Rate {
    pub rating: Option<u8>,
    pub cost: f64,
    pub error: Option<String>,
}

/// The result of checking the page against the pinned source, which a rating cannot do: a reader who sees only
/// the page cannot tell that a fact on it is wrong.
#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    pub passed: bool,
    /// What the check found, or what it covered when it passed.
    pub detail: String,
    /// What the check's model calls cost, in US dollars; zero for a check that made none.
    pub cost: f64,
}

impl Check {
    pub fn pass(detail: impl Into<String>) -> Check {
        Check {
            passed: true,
            detail: detail.into(),
            cost: 0.0,
        }
    }

    pub fn fail(detail: impl Into<String>) -> Check {
        Check {
            passed: false,
            detail: detail.into(),
            cost: 0.0,
        }
    }

    /// The same check, with what its model calls cost.
    pub fn costing(self, cost: f64) -> Check {
        Check { cost, ..self }
    }
}

/// The world the loop acts on. Every method that can spend money says what it cost.
pub trait Steps {
    fn caps(&self) -> anyhow::Result<Caps>;
    fn queue(&self) -> anyhow::Result<QueueView>;
    /// The latest rating recorded before this run, if any. It is stale: never a result of this run.
    fn previous_rating(&self) -> Option<u8>;
    fn read(&self, what: ReadOf<'_>) -> Read;
    /// Approves an in-limit open proposal by its id, as `athena approve-work` does. Refuses one that is not.
    fn approve(&self, proposal: &str) -> Approval;
    fn run(&self, job: &str) -> Run;
    /// Removes the build directories of interrupted jobs (dispatched, no process, free job lock), keeping their
    /// work trees and candidates. It is asked at the start of the drain and before each run, so interrupted jobs
    /// do not hold the disk. The default removes nothing, as for steps that have no state directory.
    fn reclaim_interrupted(&self) {}
    fn land(&self, job: &str) -> Landing;
    /// Why the configuration at the branch head cannot be read, asked only right after a [`Landing::Landed`].
    /// The configuration was readable when the run started, so a landing that makes it unreadable has brought
    /// in something this build does not understand (a key or section from a newer Athena). The default is
    /// that it can be read, as for steps with no configuration.
    fn config_after_landing(&self) -> Option<String> {
        None
    }
    fn rate(&self) -> Rate;
    /// Checks every text input of the page against the pinned source, with a model call per study. It is asked
    /// only when a rating reaches the target after a landing that changed the outcome, and the target is met
    /// only if it passes. It reports what it cost in [`Check::cost`].
    fn check_page(&self) -> Check;
    /// The most [`Steps::check_page`] may cost: the cap of one call times the number of studies it checks. The
    /// loop asks the budget for this much before it calls the check.
    fn check_cap(&self) -> f64;
    /// Builds and installs Athena from the tree a landing just advanced, the steps of `./setup.sh`. It is asked
    /// once after a landing that touched [`BUILD_PATHS`]. An error is the build output. The default builds
    /// nothing, as for a repository that is not Athena's own source.
    fn rebuild(&self) -> Result<Rebuild, String> {
        Ok(Rebuild::NotApplicable)
    }
    /// The plan entry of `job` as the records stand now. The planning pass asks for it after it has read an
    /// unread request, because a reading gives the entry its goal, acceptance and touched paths. None (the
    /// default) keeps the entry the queue view carried.
    fn plan_entry(&self, _job: &str) -> Option<PlanEntry> {
        None
    }
    /// How earlier jobs ended, for the plan's dependency rule: a dependency on a job that was rejected or failed
    /// holds the item unless a landed job is linked to it as its retry. The default is no history, so every
    /// dependency outside the queue counts as done.
    fn job_history(&self) -> Vec<JobEnd> {
        Vec::new()
    }
    /// How many items the plan keeps at urgent and at high (`[priority_caps]`); items over a cap are demoted one
    /// tier in memory when the plan is built. The default is no cap, so the plan follows the items' own tiers.
    fn priority_caps(&self) -> athena_core::plan::TierCaps {
        athena_core::plan::TierCaps::UNLIMITED
    }
    /// The queued jobs whose priority a person set with a priority record. A plan never demotes them. The
    /// default is none.
    fn person_set(&self) -> Vec<String> {
        Vec::new()
    }
    /// Athena's own improvement requests among the queue (the Steward's and Hestia's), which the plan gives a
    /// guaranteed turn: [`athena_core::hestia::improvement_of`]. Only `items` and `cause_jobs` are read; the
    /// drain fills in its own count of dispatches. The default is none, so the plan is the plain one.
    fn improvement(&self) -> athena_core::plan::Improvement {
        athena_core::plan::Improvement::default()
    }
    /// Whether the user asked the loop to stop. It is asked only between jobs, never during one, so a job
    /// that has started is finished (and landed and rated) first. Not asking to stop is the default.
    fn stop_requested(&self) -> bool {
        false
    }
    /// Athena's own housekeeping before the queue is read: the error review backfill and the evidence-label
    /// derivation, so no person has to run `athena hestia review --backfill` or `athena score-jev --derive`.
    /// Returns one summary line for each pass. Both are idempotent, and a pass that fails says so in its line
    /// without stopping the drain. The default runs nothing, as for steps that have no records.
    fn review_passes(&self) -> Vec<String> {
        Vec::new()
    }
    /// How many items ahead of the running batch [`drain_concurrent`] may read (and plan, for hard jobs) while
    /// jobs run, with one job at a time or several. 0 (the default) reads nothing ahead. [`drain`] reads each
    /// item at its turn whatever this says. The real steps keep the default until the drain's pipeline
    /// settings are wired in.
    fn read_ahead(&self) -> u32 {
        0
    }
    /// The most one [`Steps::plan_ahead`] call may cost; held against the budget while it runs.
    fn plan_cap(&self) -> f64 {
        0.0
    }
    /// Plans `job` after it was read ahead, if it is a hard job. `Some` means it is hard and was planned (with
    /// what that cost); `None` (the default) means there is nothing to plan.
    fn plan_ahead(&self, _job: &str) -> Option<Planned> {
        None
    }
    /// Whether the reading of `job`, made ahead, still holds now that the landings in `changed` touched paths
    /// it touches. Asked when the job starts. `true` (the default) keeps the reading.
    fn recheck(&self, _job: &str, _changed: &[String]) -> bool {
        true
    }
    /// A fresh reading of `job` whose earlier reading [`Steps::recheck`] said no longer holds. It must not run
    /// the job. The default refuses, so steps that override `recheck` to say no but cannot read again fail the
    /// job with that reason and never run it on a reading that was found stale.
    fn reread(&self, _job: &str) -> Read {
        Read {
            job: None,
            cost: 0.0,
            runnable: false,
            error: Some("this Steps cannot read an already-read job again".into()),
            held: false,
        }
    }
    /// What an earlier run left in the in-flight table, one line per leftover row, and a line when the file
    /// could not be read. Asked once, at the start of a drain. The drain only notes these lines. The default
    /// has no table.
    fn in_flight(&self) -> Vec<String> {
        Vec::new()
    }
    /// Told once when the work on an item ends, with the last job the item had (a retry is a new job), so the
    /// table row of a job that did not land can be removed. The default does nothing.
    fn item_over(&self, _job: &str) {}
    /// Rebases `job`'s branch onto the latest main before it lands and says what that moved. Asked under the
    /// landing lock, before [`Steps::land`]. The default does not rebase ([`Rebased::Unchanged`]). The real
    /// steps keep the default until the drain wiring of the landing rebase exists; a rebase conflict is also
    /// that wiring's matter (it holds the job), not this method's.
    fn rebase(&self, _job: &str) -> Rebased {
        Rebased::Unchanged
    }
    /// Runs the checks of `job` again after a rebase that touched what its candidate changed. Asked before
    /// [`Steps::land`], and again on the landing retry while the last answer was a failure. It reports what it
    /// cost in [`Check::cost`]. The default passes at no cost.
    fn rerun_checks(&self, _job: &str) -> Check {
        Check::pass("no checks to run again")
    }
    /// The most [`Steps::rerun_checks`] may cost; held against the budget while it runs. If it does not fit,
    /// the checks are not run and the run ends as [`Stop::BudgetSpent`].
    fn rerun_cap(&self) -> f64 {
        0.0
    }
    /// Which of `jobs` are on a line the scope guard has paused, each with the reason to show: the line, how
    /// to lift the pause and the pause file. Only a pause that still holds counts. The drain asks before it
    /// reads or runs an item, so an item on a paused line is passed over and never fails. The default has no
    /// paused line, as for steps that have no scope guard.
    fn paused(&self, _jobs: &[String]) -> Vec<(String, String)> {
        Vec::new()
    }
    /// Which of `jobs` are on a line the scope guard has only flagged (not paused), each with the reason to
    /// show: the line, the signals and why. Only a flag that still holds counts. Unlike [`Steps::paused`] this
    /// is asked only after an item's last failed attempt and never before a read, so a flagged line is still
    /// dispatched; an item that failed twice on such a line is passed over ([`ItemEnd::Flagged`]) instead of
    /// stopping the batch. The default has no flagged line, as for steps that have no scope guard.
    fn flagged(&self, _jobs: &[String]) -> Vec<(String, String)> {
        Vec::new()
    }
}

/// What rebasing a verified job's branch onto the latest main did, as far as the re-check rule needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rebased {
    /// The branch was already on the latest main, or there was nothing to rebase. Not counted as a rebase.
    Unchanged,
    /// The branch moved. `candidate` is the paths the candidate changed; `touched` the paths the rebase
    /// changed (what main gained since the candidate's base).
    Moved {
        candidate: Vec<String>,
        touched: Vec<String>,
    },
}

/// How many rebases moved a branch in a drain, and how many of them needed the checks run again. Kept in
/// memory for the run and printed in the drain summary; no record is written.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct RebaseTally {
    pub total: u32,
    pub rechecked: u32,
}

/// The rule behind the share, as the summary line names it.
const RECHECK_RULE: &str = "measured by candidate paths and any Cargo.toml or Cargo.lock";

/// Whether a rebase that touched `touched` needs the checks of a candidate that changed `candidate` run
/// again: some touched path overlaps a candidate path by the rule of [`stale_paths`] (equal, or one holds the
/// other as a directory; the shared append-only files are ignored), or a touched path is a `Cargo.toml` or
/// `Cargo.lock` at any depth, because a manifest change can break a candidate that never touched it.
fn rebase_needs_recheck(candidate: &[String], touched: &[String]) -> bool {
    let manifest = |p: &String| {
        matches!(
            p.rsplit('/').next(),
            Some("Cargo.toml") | Some("Cargo.lock")
        )
    };
    !stale_paths(candidate, touched).is_empty() || touched.iter().any(manifest)
}

/// The file name of the in-flight table in the state directory.
pub const IN_FLIGHT_FILE: &str = "in-flight.json";

/// One row of the in-flight table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InFlightRow {
    pub job: String,
    pub stage: Stage,
    /// The producer's live work tree, `StateDir::work_dir` of the job.
    pub worktree: PathBuf,
}

/// What reading the table found.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InFlightRead {
    pub rows: Vec<InFlightRow>,
    /// Set when the file could not be read: where it was moved to, and why.
    pub corrupt: Option<String>,
}

/// The local in-flight table: a file of job id, stage and worktree, rewritten at each stage change. Every
/// update takes one process-wide lock, reads the whole file and writes it back through a temporary file and a
/// rename, so workers of [`drain_concurrent`] do not lose each other's rows.
#[derive(Debug, Clone)]
pub struct InFlight {
    path: PathBuf,
}

static TABLE_LOCK: Mutex<()> = Mutex::new(());

impl InFlight {
    /// The table in the state directory `state_root`.
    pub fn at(state_root: &Path) -> InFlight {
        InFlight {
            path: state_root.join(IN_FLIGHT_FILE),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The rows now. A missing file is an empty table. A file that is not a table is moved to
    /// `in-flight.json.corrupt` and reported in [`InFlightRead::corrupt`]; the rows are then empty.
    pub fn read(&self) -> InFlightRead {
        let _guard = TABLE_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        self.load()
    }

    fn load(&self) -> InFlightRead {
        let bytes = match std::fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return InFlightRead::default(),
            Err(e) => {
                return InFlightRead {
                    rows: Vec::new(),
                    corrupt: Some(format!("{} could not be read: {e}", self.path.display())),
                };
            }
        };
        match serde_json::from_slice::<Vec<InFlightRow>>(&bytes) {
            Ok(rows) => InFlightRead {
                rows,
                corrupt: None,
            },
            Err(why) => {
                let aside = self.path.with_extension("json.corrupt");
                let moved = match std::fs::rename(&self.path, &aside) {
                    Ok(()) => format!("moved to {}", aside.display()),
                    Err(e) => format!("could not be moved aside: {e}"),
                };
                InFlightRead {
                    rows: Vec::new(),
                    corrupt: Some(format!(
                        "{} is not a table of rows ({why}); {moved}",
                        self.path.display()
                    )),
                }
            }
        }
    }

    fn store(&self, rows: &[InFlightRow]) -> anyhow::Result<()> {
        if rows.is_empty() {
            return match std::fs::remove_file(&self.path) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
                _ => Ok(()),
            };
        }
        let temp = self.path.with_extension("json.tmp");
        std::fs::write(&temp, serde_json::to_vec_pretty(rows)?)?;
        std::fs::rename(&temp, &self.path)?;
        Ok(())
    }

    fn update(&self, change: impl FnOnce(&mut Vec<InFlightRow>)) -> anyhow::Result<()> {
        let _guard = TABLE_LOCK.lock().unwrap_or_else(PoisonError::into_inner);
        let mut rows = self.load().rows;
        change(&mut rows);
        self.store(&rows)
    }

    /// Writes the row of `row.job`, replacing the one it has.
    pub fn set(&self, row: InFlightRow) -> anyhow::Result<()> {
        self.update(|rows| {
            rows.retain(|r| r.job != row.job);
            rows.push(row);
        })
    }

    /// Gives the row of `old`, if there is one, to `new` with `worktree`, keeping its stage.
    pub fn move_row(&self, old: &str, new: &str, worktree: PathBuf) -> anyhow::Result<()> {
        self.update(|rows| {
            if let Some(at) = rows.iter().position(|r| r.job == old) {
                let mut row = rows.remove(at);
                rows.retain(|r| r.job != new);
                row.job = new.to_string();
                row.worktree = worktree;
                rows.push(row);
            }
        })
    }

    /// Removes the row of `job`, if there is one.
    pub fn remove(&self, job: &str) -> anyhow::Result<()> {
        self.update(|rows| rows.retain(|r| r.job != job))
    }
}

/// The sink the real steps give `Athena::go_with_stages`: each stage the run enters becomes the job's row.
struct TableSink<'a> {
    table: InFlight,
    athena: &'a Athena,
    ui: &'a dyn Ui,
}

impl StageSink for TableSink<'_> {
    fn enter(&self, job: &JobId, stage: Stage) {
        let row = InFlightRow {
            job: job.as_str().to_string(),
            stage,
            worktree: self.athena.state.work_dir(job),
        };
        if let Err(e) = self.table.set(row) {
            self.ui.line(&format!(
                "warning: could not write the in-flight table: {e:#}"
            ));
        }
    }
}

/// What planning a hard job ahead of its turn did ([`Steps::plan_ahead`]).
#[derive(Debug, Clone, Default)]
pub struct Planned {
    pub cost: f64,
    /// Why the job could not be planned. The reading made ahead is then dropped and the job is read again
    /// when it starts.
    pub error: Option<String>,
}

/// A reading made ahead of its job's turn, kept until the job starts.
#[derive(Debug, Clone)]
struct Ahead {
    read: Read,
    /// How many landings had been recorded when its reading began; only later ones can make it stale.
    seen: usize,
}

/// Where the reading of a job comes from when it starts.
#[derive(Default)]
enum Source {
    /// Read it as the drain always did.
    #[default]
    Read,
    /// A reading made ahead, which is run as it is.
    Given(Read),
    /// A reading made ahead that no longer holds: read the job again with [`Steps::reread`].
    Reread,
}

/// The shared append-only files that many jobs edit. The plan does not group by them
/// (`athena_core::plan`'s `SHARED_APPEND_ONLY`), so neither does read-ahead.
const SHARED_APPEND_ONLY: [&str; 3] = ["CHANGELOG.md", "docs/decisions.md", "README.md"];

/// The paths that count when comparing two sets: without the shared append-only files.
fn group_paths(paths: &[String]) -> Vec<&str> {
    paths
        .iter()
        .map(String::as_str)
        .filter(|p| !SHARED_APPEND_ONLY.contains(p))
        .collect()
}

/// Whether two paths overlap: equal, or one is a directory holding the other.
fn overlap(a: &str, b: &str) -> bool {
    let inside = |dir: &str, path: &str| {
        let dir = dir.trim_end_matches('/');
        path == dir || path.strip_prefix(dir).is_some_and(|r| r.starts_with('/'))
    };
    inside(a, b) || inside(b, a)
}

/// Whether two jobs may be in play side by side: both name some path besides the shared append-only files,
/// and nothing of that overlaps. This is the rule of `athena_core::plan` (`disjoint`, `overlap` and
/// `SHARED_APPEND_ONLY`), copied because those are private to that crate; a test checks the copy against
/// `Plan::build` so the two cannot drift apart.
fn paths_disjoint(a: &[String], b: &[String]) -> bool {
    let (pa, pb) = (group_paths(a), group_paths(b));
    !pa.is_empty() && !pb.is_empty() && !pa.iter().any(|x| pb.iter().any(|y| overlap(x, y)))
}

/// The paths in `changed` (what landings changed) that can make a reading that touches `touches` stale: the
/// ones that overlap a touched path, by the rule of [`paths_disjoint`]. A reading with no touched paths besides
/// the shared files is of unknown scope, so any landed path besides those counts. Sorted, without repeats.
fn stale_paths(touches: &[String], changed: &[String]) -> Vec<String> {
    let mine = group_paths(touches);
    let mut hit: Vec<String> = group_paths(changed)
        .into_iter()
        .filter(|p| mine.is_empty() || mine.iter().any(|m| overlap(m, p)))
        .map(String::from)
        .collect();
    hit.sort();
    hit.dedup();
    hit
}

/// The file whose presence asks a running drain to stop at the next job boundary, inside the repository's
/// state directory. `athena drain-stop` creates it and the next `athena drain` removes any left over.
pub fn stop_file(state_root: &std::path::Path) -> std::path::PathBuf {
    state_root.join("drain.stop")
}

/// Why the loop stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stop {
    /// A rating taken after a landing reached the target, and two more things hold. (1) At least one landing in
    /// this run changed what the explainer is built from ([`OUTCOME_PATHS`]: `studies/`, the site generator or
    /// the publish workflow); a landing that changed only other paths leaves the page as it was, so its rating
    /// proves nothing. (2) Every text input of every study (all files but those in
    /// `athena_core::pin::STRUCTURAL_FILES`: claims, the intro, its transition and glossary, and the rest)
    /// passed a model-backed check against the pinned source, which fails on a contradicted or omitted
    /// statement; the model-free self-consistency check of each model stays as a floor under it.
    TargetMet,
    /// The next call's cap did not fit what was left of the budget.
    BudgetSpent,
    /// The environment, not the item, is at fault: an item failed twice and the closing reason of its second
    /// failure says the producer's login expired, a usage limit was reached, or free disk space is low
    /// ([`athena_core::disk::StopCause`]). Every later item would fail the same way, so nothing more starts.
    /// An item that fails twice for any other reason is not this: it is passed over and
    /// the batch goes on, as it does for an item whose reader gave no reading twice, or whose line the
    /// scope guard has paused or flagged.
    Environment,
    /// Nothing is queued (or everything queued was worked). This is not success.
    QueueEmpty,
    /// The job limit was reached.
    JobLimit,
    /// The user asked the loop to stop (`athena drain-stop`). The job in progress was finished first and no
    /// further job was started; nothing was killed.
    Requested,
    /// A landing changed Athena's own code and Athena was rebuilt and installed. The pass ends so that the new
    /// binary can carry on with what is left ([`Report::restart`]); [`follow_restart`] does that.
    Rebuilt,
    /// A landing changed Athena's own code and the rebuild failed. Nothing more starts, and
    /// [`Report::rebuild_output`] holds the build output.
    RebuildFailed,
    /// A job landed and its `athena.toml` can no longer be read by the running build, because the landing
    /// added something only a newer build understands. The job stays landed and no failure is counted; nothing
    /// more starts, and the user reinstalls Athena and runs the drain again. It is reported whenever the
    /// landed config was unreadable and no rebuild followed, even when another worker stopped the run first
    /// (only [`Stop::RebuildFailed`] keeps its own place).
    NeedsReinstall,
}

impl Stop {
    pub const ALL: [Stop; 9] = [
        Stop::TargetMet,
        Stop::BudgetSpent,
        Stop::Environment,
        Stop::QueueEmpty,
        Stop::JobLimit,
        Stop::Requested,
        Stop::Rebuilt,
        Stop::RebuildFailed,
        Stop::NeedsReinstall,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Stop::TargetMet => "target met",
            Stop::BudgetSpent => "budget spent",
            Stop::Environment => "environment",
            Stop::QueueEmpty => "queue empty",
            Stop::JobLimit => "job limit",
            Stop::Requested => "stop requested",
            Stop::Rebuilt => "rebuilt",
            Stop::RebuildFailed => "rebuild failed",
            Stop::NeedsReinstall => "reinstall needed",
        }
    }

    pub fn meaning(self) -> &'static str {
        match self {
            Stop::TargetMet => {
                "a rating taken after a landing in this run reached the target rating, a landing in this run changed what the explainer is built from, and all of the page's text passed its check against the pinned source"
            }
            Stop::BudgetSpent => {
                "spend plus the next model call's cap would exceed the budget; the rest stays queued"
            }
            Stop::Environment => {
                "an item failed twice because the producer's login expired, a usage limit was reached or free disk space is low, so every later item would fail the same way; fix that and run the drain again (an item that fails twice for any other reason is passed over and does not stop the batch)"
            }
            Stop::QueueEmpty => "nothing was left to work; this is not success",
            Stop::JobLimit => "the job limit was reached",
            Stop::Requested => {
                "a stop was asked for; the job in progress was finished and no further job was started, so the rest stays queued"
            }
            Stop::Rebuilt => {
                "a landing changed Athena's own code, so Athena was rebuilt and installed; the new binary carries on with the remaining budget and job limit"
            }
            Stop::RebuildFailed => {
                "a landing changed Athena's own code and rebuilding it failed; the build output follows and the rest stays queued"
            }
            Stop::NeedsReinstall => {
                "a job landed and its athena.toml needs a newer Athena than the one running; the job is landed and not a failure, so reinstall Athena (./setup.sh) and run the drain again"
            }
        }
    }
}

/// One job the loop started, and what became of it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JobLine {
    /// The queued request this job works on.
    pub item: String,
    /// The job itself; a retry of an item is a job with another id.
    pub job: String,
    /// How the job ended: `passed` (landed on the local branch), `rejected`, `failed`, `blocked`, `not read`,
    /// `not run` or `no outcome`. For `not read`, `reason` is the reader's error.
    pub outcome: String,
    pub reason: Option<String>,
    pub cost_usd: f64,
    /// `landed`, `not landed: <why>`, `waiting to merge: <why>` (the pull request is open and waits on its
    /// checks; not a failure) or `not attempted`.
    pub landing: String,
}

/// A rating and where it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RatingLine {
    pub value: u8,
    /// True only for a rating this run took after at least one landing. False means the rating is the last one
    /// recorded before the run, of a page this run did not change.
    pub after_landing: bool,
    pub met_target: bool,
}

/// What a drain did.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Report {
    pub stop: Stop,
    pub jobs: Vec<JobLine>,
    pub spent_usd: f64,
    pub budget_usd: f64,
    pub target_rating: u8,
    /// Items landed in this run.
    pub landed: u32,
    /// True when nothing landed, so the page is as it was.
    pub page_unchanged: bool,
    /// Whether a landing in this run changed a path the explainer is built from ([`OUTCOME_PATHS`]). When
    /// something landed and this is false, the landings did not change the outcome.
    pub outcome_changed: bool,
    pub rating: Option<RatingLine>,
    /// Set when a rating reached the target but the check of the page against the pinned source failed, so
    /// drain kept running: what the check found.
    pub held_back: Option<String>,
    /// Queued requests not landed, left for a later run.
    pub queued: Vec<Item>,
    /// Open proposals waiting for a person. The loop did not touch them.
    pub waiting: Vec<Waiting>,
    /// The plan the loop followed: the planned items in groups. Items of one group have no dependency and
    /// touch disjoint paths; they run side by side up to `[parallel] max_concurrency`, group after group in
    /// this order (one job at a time when the bound is 1).
    pub plan_groups: Vec<Vec<String>>,
    /// How many items each of the plan's groups held when the run started, in group order. It measures how
    /// much parallel work the plan offered, so it is the same whatever `[parallel] max_concurrency` was.
    pub plan_group_sizes: Vec<usize>,
    /// Queued items that had no plan entry when the run started. They belong to no group: they are counted
    /// here and never as a group of one, so a lone planned item and an unplanned one stay apart.
    pub unplanned: usize,
    /// Items refused for design or code because their plan lacks a goal, a ladder rung or acceptance.
    pub refused: Vec<Refusal>,
    /// Anything that went wrong beside the stop reason, such as a rating that could not be taken.
    pub notes: Vec<String>,
    /// The build output, when `stop` is `rebuild-failed`.
    pub rebuild_output: Option<String>,
    /// What is left to carry on with, when `stop` is `rebuilt` ([`follow_restart`]).
    pub restart: Option<Restart>,
    /// The rebases before landing that moved the branch, and how many of them touched a path that needed the
    /// checks run again. Kept for this run only; also stated in [`Report::notes`] when there was one.
    pub rebases: RebaseTally,
    /// The improvement requests (Steward or Hestia) that took the guaranteed turn: dispatched in the place the
    /// plan gave them after two dispatches to other work ([`athena_core::plan::Plan::turns`]). Empty when
    /// none did.
    pub improvement_turns: Vec<ImprovementTaken>,
}

/// An improvement request that the drain handed to a worker in its guaranteed turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ImprovementTaken {
    pub job: String,
    pub title: String,
    /// Which dispatch of the run it was, counted from 1.
    pub dispatch: u32,
    /// True when the request ranked later and was moved up to the turn; false when it already stood there.
    pub moved: bool,
}

impl Report {
    /// The target was met by a rating taken after a landing. The only way to be true.
    pub fn target_met(&self) -> bool {
        self.stop == Stop::TargetMet && self.rating.is_some_and(|r| r.after_landing && r.met_target)
    }
}

// The rest of the drain lives in child modules by responsibility. Each is re-exported here, so every path
// under `crate::drain` is the one it was before the split.
mod engine;
mod item_work;
mod lander;
mod lifecycle;
mod real_steps;
mod rounds;
pub use engine::*;
pub use lander::*;
pub use lifecycle::*;
pub use real_steps::*;

/// `batches_for` takes the drain's order from the plan: it filters and never reorders.
#[cfg(test)]
mod batches_tests {
    use super::*;
    use athena_data::roles::Ladder;

    fn entry(id: &str, rung: Ladder, priority: athena_core::plan::Priority) -> PlanEntry {
        PlanEntry {
            goal: format!("goal of {id}"),
            rung: Some(rung),
            acceptance: vec![format!("{id} works")],
            touches: vec![format!("src/{id}.rs")],
            priority,
            ..PlanEntry::bare(id, &format!("title of {id}"))
        }
    }

    fn ids(batches: &[Vec<Item>]) -> Vec<String> {
        batches.iter().flatten().map(|i| i.job.clone()).collect()
    }

    #[test]
    fn batches_for_flattened_equal_the_plan_order_and_tried_removes_only_that_item() {
        use athena_core::plan::Priority::{High, Low, Normal};
        let mut entries = vec![
            entry("old", Ladder::Implementation, Normal),
            entry("low", Ladder::DesignDecisions, Low),
            entry("new", Ladder::Implementation, High),
            entry("mid", Ladder::DesignDecisions, Normal),
        ];
        entries[3].depends_on = vec!["old".into()];
        let plan = Plan::build(&entries);
        let mut items: Vec<Item> = entries
            .iter()
            .map(|e| Item::queued(&e.id, &e.title))
            .collect();
        // An item with no plan entry comes first, alone.
        items.push(Item::queued("bare", "a request with no plan entry"));
        let all = batches_for(&items, &entries, &plan, &BTreeSet::new());
        assert_eq!(all[0].len(), 1);
        assert_eq!(all[0][0].job, "bare");
        let planned: Vec<Vec<String>> = all[1..]
            .iter()
            .map(|b| b.iter().map(|i| i.job.clone()).collect())
            .collect();
        assert_eq!(planned, plan.groups);
        assert_eq!(ids(&all[1..]), plan.order);
        // What is tried drops out and the rest keeps the plan's order.
        let tried: BTreeSet<String> = [plan.order[1].clone(), "bare".to_string()].into();
        let rest = batches_for(&items, &entries, &plan, &tried);
        let mut expected = plan.order.clone();
        expected.remove(1);
        assert_eq!(ids(&rest), expected);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_core::Quiet;
    use std::cell::RefCell;

    /// A stub that logs every call and costs what the test says.
    struct Stub {
        calls: RefCell<Vec<String>>,
        read_cost: f64,
        run_cost: f64,
        outcome: OutcomeState,
    }

    impl Steps for Stub {
        fn caps(&self) -> anyhow::Result<Caps> {
            Ok(Caps {
                read: 0.5,
                run: 1.0,
                rate: 0.5,
            })
        }
        fn queue(&self) -> anyhow::Result<QueueView> {
            Ok(QueueView {
                items: vec![Item::queued("j1", "t")],
                ..QueueView::default()
            })
        }
        fn approve(&self, _: &str) -> Approval {
            Approval::default()
        }
        fn previous_rating(&self) -> Option<u8> {
            None
        }
        fn read(&self, what: ReadOf<'_>) -> Read {
            self.calls.borrow_mut().push(format!("read {what:?}"));
            Read {
                job: Some("job-a".into()),
                cost: self.read_cost,
                runnable: true,
                error: None,
                held: false,
            }
        }
        fn run(&self, job: &str) -> Run {
            self.calls.borrow_mut().push(format!("run {job}"));
            Run {
                outcome: Some(self.outcome),
                reason: String::new(),
                cost: self.run_cost,
                ..Run::default()
            }
        }
        fn land(&self, job: &str) -> Landing {
            self.calls.borrow_mut().push(format!("land {job}"));
            Landing::Landed {
                changed: vec!["studies/s/claims.json".into()],
            }
        }
        fn check_page(&self) -> Check {
            Check::pass("stub")
        }
        fn check_cap(&self) -> f64 {
            0.5
        }
        fn rate(&self) -> Rate {
            self.calls.borrow_mut().push("rate".into());
            Rate {
                rating: Some(3),
                cost: 0.1,
                error: None,
            }
        }
    }

    /// Steps that only list a queue of `(id, priority)` entries, oldest first, with urgent capped at 3.
    struct Capped {
        queue: Vec<(&'static str, athena_core::plan::Priority)>,
        person_set: Vec<String>,
    }

    impl Steps for Capped {
        fn caps(&self) -> anyhow::Result<Caps> {
            Ok(Caps {
                read: 0.5,
                run: 1.0,
                rate: 0.5,
            })
        }
        fn queue(&self) -> anyhow::Result<QueueView> {
            Ok(QueueView {
                items: self
                    .queue
                    .iter()
                    .map(|(id, _)| Item::queued(id, id))
                    .collect(),
                plan: self
                    .queue
                    .iter()
                    .map(|(id, priority)| PlanEntry {
                        goal: format!("goal of {id}"),
                        rung: Some(athena_data::roles::Ladder::Implementation),
                        acceptance: vec![format!("{id} works")],
                        touches: vec![format!("src/{id}.rs")],
                        priority: *priority,
                        ..PlanEntry::bare(id, id)
                    })
                    .collect(),
                ..QueueView::default()
            })
        }
        fn priority_caps(&self) -> athena_core::plan::TierCaps {
            athena_core::plan::TierCaps {
                urgent: 3,
                high: 10,
            }
        }
        fn person_set(&self) -> Vec<String> {
            self.person_set.clone()
        }
        fn approve(&self, _: &str) -> Approval {
            Approval::default()
        }
        fn previous_rating(&self) -> Option<u8> {
            None
        }
        fn read(&self, _: ReadOf<'_>) -> Read {
            panic!("every entry is complete, so nothing is read");
        }
        fn run(&self, _: &str) -> Run {
            panic!("planning runs nothing");
        }
        fn land(&self, _: &str) -> Landing {
            panic!("planning lands nothing");
        }
        fn check_page(&self) -> Check {
            Check::pass("stub")
        }
        fn check_cap(&self) -> f64 {
            0.5
        }
        fn rate(&self) -> Rate {
            panic!("planning rates nothing");
        }
    }

    #[test]
    fn a_plan_demotes_over_the_cap_with_one_note_each_and_no_false_contradiction() {
        use athena_core::plan::Priority::{High, Urgent};
        let steps = Capped {
            queue: vec![
                ("h0", High),
                ("u1", Urgent),
                ("u2", Urgent),
                ("u3", Urgent),
                ("u4", Urgent),
                ("u5", Urgent),
            ],
            person_set: vec!["u5".into()],
        };
        let report = plan(&steps, 1.0, &Quiet).unwrap();
        // The person-set item keeps its tier; the two newest others drop. They are planned after the older
        // high item, which would read as a contradiction if the order were judged by their original tier.
        assert_eq!(report.plan.order[..3], ["u1", "u2", "u5"]);
        let demoted: Vec<&str> = report.plan.demoted.iter().map(|d| d.id.as_str()).collect();
        assert_eq!(demoted, ["u3", "u4"]);
        let lines: Vec<&String> = report
            .notes
            .iter()
            .filter(|n| n.starts_with("demoted "))
            .collect();
        assert_eq!(lines.len(), 2, "{:?}", report.notes);
        assert!(lines[0].starts_with("demoted u3 · u3 from urgent to high"));
        assert!(
            !report.notes.iter().any(|n| n.contains("contradicts")),
            "{:?}",
            report.notes
        );
    }

    fn limits(budget: f64) -> Limits {
        Limits {
            budget_usd: budget,
            target_rating: 4,
            max_jobs: 10,
        }
    }

    /// Steps with an empty queue that log when the review passes run and when the queue is read.
    struct Housekeeping {
        calls: RefCell<Vec<&'static str>>,
    }

    impl Steps for Housekeeping {
        fn review_passes(&self) -> Vec<String> {
            self.calls.borrow_mut().push("review");
            vec![
                "backfill: summary".into(),
                "evidence labels: summary".into(),
            ]
        }
        fn caps(&self) -> anyhow::Result<Caps> {
            Ok(Caps {
                read: 0.5,
                run: 1.0,
                rate: 0.5,
            })
        }
        fn queue(&self) -> anyhow::Result<QueueView> {
            self.calls.borrow_mut().push("queue");
            Ok(QueueView::default())
        }
        fn approve(&self, _: &str) -> Approval {
            Approval::default()
        }
        fn previous_rating(&self) -> Option<u8> {
            None
        }
        fn read(&self, _: ReadOf<'_>) -> Read {
            unreachable!("nothing is queued")
        }
        fn run(&self, _: &str) -> Run {
            unreachable!("nothing is queued")
        }
        fn land(&self, _: &str) -> Landing {
            unreachable!("nothing is queued")
        }
        fn check_page(&self) -> Check {
            Check::pass("stub")
        }
        fn check_cap(&self) -> f64 {
            0.5
        }
        fn rate(&self) -> Rate {
            unreachable!("nothing is queued")
        }
    }

    /// A UI that keeps its lines.
    #[derive(Default)]
    struct Kept(Mutex<Vec<String>>);

    impl Ui for Kept {
        fn line(&self, text: &str) {
            self.0.lock().unwrap().push(text.to_string());
        }
    }

    #[test]
    fn a_drain_runs_the_review_passes_before_it_reads_the_queue_and_prints_a_line_for_each() {
        let steps = Housekeeping {
            calls: RefCell::new(Vec::new()),
        };
        let ui = Kept::default();
        drain(&steps, limits(3.0), &ui).unwrap();
        assert_eq!(*steps.calls.borrow(), ["review", "queue"]);
        let lines = ui.0.lock().unwrap();
        assert_eq!(
            *lines,
            ["backfill: summary", "evidence labels: summary"],
            "one line per pass"
        );
    }

    #[test]
    fn a_call_starts_only_when_spend_plus_its_cap_fits() {
        let budget = Budget::new(2.0);
        assert!(budget.fits(2.0), "exactly the budget fits");
        assert!(!budget.fits(2.01));
        let mut spent = Budget::new(2.0);
        spent.add(1.5);
        assert!(spent.fits(0.5));
        assert!(!spent.fits(0.51));
        // A call that cost more than its cap leaves less room, never more.
        let mut over = Budget::new(2.0);
        over.add(2.5);
        assert!(!over.fits(0.0001));
    }

    #[test]
    fn nothing_starts_after_a_job_cost_more_than_its_cap_not_even_the_retry_reading() {
        // Budget 3.0; the reading (cap 0.5) cost 0.5 and the run (cap 1.0) cost 2.2 and was rejected. Spent is
        // 2.7, so the retry's reading (0.5) plus its run (1.0) cannot fit, and neither is started.
        let stub = Stub {
            calls: RefCell::new(Vec::new()),
            read_cost: 0.5,
            run_cost: 2.2,
            outcome: OutcomeState::Rejected,
        };
        let report = drain(&stub, limits(3.0), &Quiet).unwrap();
        assert_eq!(report.stop, Stop::BudgetSpent);
        assert_eq!(
            *stub.calls.borrow(),
            ["read Queued(\"j1\")", "run job-a"],
            "no retry reading, no second run, no rating, no landing"
        );
        assert!((report.spent_usd - 2.7).abs() < 1e-9);
        assert_eq!(report.queued.len(), 1, "the item stays queued");
    }

    #[test]
    fn a_reading_that_used_up_the_room_for_the_run_stops_before_the_run() {
        // Budget 1.4: the reading (cap 0.5) fits with the run (cap 1.0)? 1.5 > 1.4, so the job never starts.
        let stub = Stub {
            calls: RefCell::new(Vec::new()),
            read_cost: 0.1,
            run_cost: 0.1,
            outcome: OutcomeState::Landed,
        };
        let report = drain(&stub, limits(1.4), &Quiet).unwrap();
        assert_eq!(report.stop, Stop::BudgetSpent);
        assert!(stub.calls.borrow().is_empty(), "{:?}", stub.calls.borrow());
        assert_eq!(report.spent_usd, 0.0);
    }

    fn model(changed_by: &[&str]) -> serde_json::Value {
        serde_json::json!({
            "counters": [{"id": "places-held", "changed_by": changed_by}],
            "transitions": [
                {"id": "t1", "retry": null},
                {"id": "t2", "retry": {"changes": ["places-held falls when another worker records its ready PR or is cleaned up"]}}
            ]
        })
    }

    #[test]
    fn a_counter_whose_changed_by_omits_a_transition_its_retry_changes_is_a_contradiction() {
        let found = model_contradictions(&model(&["t1", "t3"]));
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("places-held") && found[0].contains("t2"));
        assert!(model_contradictions(&model(&["t1", "t3", "t4"])).is_empty());
    }

    #[test]
    fn the_shipped_firstmate_model_is_consistent() {
        let text = include_str!("../../../studies/firstmate-work-handling/model.json");
        let model: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(model_contradictions(&model), Vec::<String>::new());
    }

    #[test]
    fn only_paths_the_explainer_is_built_from_change_the_outcome() {
        let paths = |p: &[&str]| p.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        for yes in [
            "studies/firstmate-work-handling/intro.json",
            "xtask/src/site.rs",
            ".github/workflows/pages.yml",
        ] {
            assert!(changes_outcome(&paths(&["README.md", yes])), "{yes}");
        }
        for no in [
            "README.md",
            "crates/athena-cli/src/drain.rs",
            "docs/decisions.md",
            "studies-notes.md",
            ".github/workflows/ci.yml",
        ] {
            assert!(!changes_outcome(&paths(&[no])), "{no}");
        }
        assert!(!changes_outcome(&[]));
    }

    #[test]
    fn a_study_file_added_later_is_page_text_and_a_study_without_a_pin_cannot_be_checked() {
        let file = |name: &str, body: &str| (name.to_string(), body.as_bytes().to_vec());
        let pin = format!(
            r#"{{"repository_url": "https://example.com/r.git", "sha": "{}"}}"#,
            "b".repeat(40)
        );
        let files = vec![
            file("pin.json", &pin),
            file("validation.json", "{}"),
            file("claims.json", "[]"),
            file("notes.md", "a new text file"),
        ];
        let study = study_text("s", &files).unwrap();
        let names: Vec<&str> = study.files.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, ["claims.json", "notes.md"]);
        assert!(study_text("s", &files[1..]).is_err(), "no pin");
        let bad = vec![file("pin.json", &pin), file("blob.bin", "")];
        let mut bad = bad;
        bad[1].1 = vec![0xff, 0xfe];
        assert!(study_text("s", &bad).unwrap_err().contains("not text"));
    }

    #[test]
    fn every_stop_reason_has_a_label_and_a_meaning() {
        for stop in Stop::ALL {
            assert!(!stop.label().is_empty() && !stop.meaning().is_empty());
        }
        assert!(Stop::QueueEmpty.meaning().contains("not success"));
        assert!(Stop::NeedsReinstall.meaning().contains("reinstall"));
    }

    // ---- The drain's life cycle: pid file, status, stop, restart, stale-build refusal ----

    /// A child process that is killed and reaped when the test ends, however it ends.
    struct Reaped(std::process::Child);

    impl Reaped {
        fn id(&self) -> u32 {
            self.0.id()
        }
    }

    impl Drop for Reaped {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }

    /// A process that sleeps, with `arg0` as the first word of its command line.
    fn sleeper(arg0: &str) -> Reaped {
        use std::os::unix::process::CommandExt;
        Reaped(
            std::process::Command::new("sleep")
                .arg0(arg0)
                .arg("60")
                .spawn()
                .unwrap(),
        )
    }

    /// The pid record of a live process, with its real start marker.
    fn record_of(pid: u32, build: &str) -> PidRecord {
        PidRecord {
            pid,
            started: RealProcs.start_marker(pid).expect("the process is alive"),
            build: build.into(),
            at: "2026-11-05T20:00:00Z".into(),
        }
    }

    const BUILD: &str = "athena 0.0.0 (commit abcdef123456)";

    /// A wait with short polls.
    fn quick(limit: Option<Duration>) -> StopWait {
        StopWait {
            limit,
            poll: Duration::from_millis(20),
            notice: Duration::from_secs(30),
            start: Duration::from_secs(10),
        }
    }

    /// The state once the drain stops showing as running (a killed child that is not yet reaped takes a
    /// moment to become a zombie), or after ten seconds.
    fn settled(root: &Path) -> DrainState {
        for _ in 0..500 {
            let state = drain_state(root, &RealProcs);
            if !matches!(state, DrainState::Running(_)) {
                return state;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        drain_state(root, &RealProcs)
    }

    #[test]
    fn the_start_marker_is_read_by_pid_from_the_stat_text_or_the_ps_text() {
        // A command name with spaces and parentheses must not shift the fields.
        let stat =
            "1234 (a b) c) S 1 1234 1234 0 -1 4194560 100 0 0 0 0 0 0 0 20 0 1 0 987654 1000 100";
        assert_eq!(linux_marker(stat).as_deref(), Some("987654"));
        assert_eq!(linux_marker(&stat.replace(") S 1", ") Z 1")), None);
        assert_eq!(linux_marker("garbage"), None);
        assert_eq!(
            ps_marker("Thu Nov  5 20:59:32 2026 Ss\n").as_deref(),
            Some("Thu Nov 5 20:59:32 2026")
        );
        assert_eq!(ps_marker("Thu Nov  5 20:59:32 2026 Z+\n"), None);
        assert_eq!(ps_marker(""), None);
        let own = RealProcs.start_marker(std::process::id());
        assert!(own.is_some());
        assert_eq!(own, RealProcs.start_marker(std::process::id()));
    }

    #[test]
    fn commit_in_version_reads_the_commit_athena_version_prints() {
        assert_eq!(
            commit_in_version("athena 0.1.0 (commit abc123def456)\n").as_deref(),
            Some("abc123def456")
        );
        assert_eq!(
            commit_in_version("athena 0.1.0 (commit abc123def456, dirty)").as_deref(),
            Some("abc123def456")
        );
        assert_eq!(
            commit_in_version("athena 0.1.0 (commit unknown, dirty)"),
            None
        );
        assert_eq!(commit_in_version("athena 0.1.0"), None);
    }

    /// A stand-in drain whose command line reads `athena drain ...`. It exits only when both the stop file and
    /// the release file exist, so a test decides when it ends.
    fn stand_in_drain(root: &Path, release: &Path) -> Reaped {
        use std::os::unix::process::CommandExt;
        let script = r#"while [ ! -e "$1" ] || [ ! -e "$2" ]; do sleep 0.05; done"#;
        Reaped(
            std::process::Command::new("sh")
                .arg0("athena drain --budget 5")
                .args(["-c", script, "sh"])
                .arg(stop_file(root))
                .arg(release)
                .spawn()
                .unwrap(),
        )
    }

    /// Real processes, except that the first lookup, once it has read the process, creates the release file.
    /// The lookup that `stop_and_wait` makes before it waits therefore sees the stand-in alive, and the
    /// stand-in can only exit after the wait has begun.
    struct ReleaseAfterFirstLookup {
        release: PathBuf,
        released: std::sync::atomic::AtomicBool,
    }

    impl Procs for ReleaseAfterFirstLookup {
        fn start_marker(&self, pid: u32) -> Option<String> {
            let marker = RealProcs.start_marker(pid);
            if !self
                .released
                .swap(true, std::sync::atomic::Ordering::SeqCst)
            {
                std::fs::write(&self.release, "go\n").unwrap();
            }
            marker
        }
    }

    #[test]
    fn a_stand_in_drain_that_exited_before_the_waiting_stop_reads_as_dead_not_exited() {
        // The shape the stop test had before the release file: the stand-in is gone by the time the waiting
        // call looks, so the call finds no running drain. A dead pid with no earlier sighting is
        // indistinguishable from a crashed or stale drain, so it stays NotRunning(Dead).
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let release = root.join("release");
        let mut drain_child = stand_in_drain(root, &release);
        let record = record_of(drain_child.id(), BUILD);
        write_record(root, &record).unwrap();
        let asked = stop_and_wait(root, &RealProcs, None, &Quiet).unwrap();
        assert_eq!(asked, StopOutcome::Asked(record.clone()));
        std::fs::write(&release, "go\n").unwrap();
        drain_child.0.wait().unwrap();
        let waited = stop_and_wait(
            root,
            &RealProcs,
            Some(&quick(Some(Duration::from_secs(20)))),
            &Quiet,
        )
        .unwrap();
        assert_eq!(
            waited,
            StopOutcome::NotRunning(NotRunning::Dead(record.pid))
        );
    }

    #[test]
    fn status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let release = root.join("release");
        // Two processes whose command lines read `athena drain ...`. Only one is named in the pid file.
        let drain_child = stand_in_drain(root, &release);
        let decoy = sleeper("athena drain --budget 9");
        let record = record_of(drain_child.id(), BUILD);
        write_record(root, &record).unwrap();

        match drain_state(root, &RealProcs) {
            DrainState::Running(found) => assert_eq!(found, record),
            other => panic!("expected the drain to be running: {other:?}"),
        }
        let status = crate::display::drain_status(&drain_state(root, &RealProcs), false);
        assert!(
            status.contains(&format!("pid {}", drain_child.id())),
            "{status}"
        );
        assert!(
            status.contains(BUILD) && status.contains("running"),
            "{status}"
        );
        assert!(!status.contains(&decoy.id().to_string()), "{status}");

        // A stop with no wait only asks.
        let asked = stop_and_wait(root, &RealProcs, None, &Quiet).unwrap();
        assert_eq!(asked, StopOutcome::Asked(record.clone()));
        assert!(stop_file(root).exists());
        // Asking again with a wait returns once the drain has exited. The stand-in exits at the stop file only
        // once the release file exists, and the release is made after this call has seen it running.
        let waited = stop_and_wait(
            root,
            &ReleaseAfterFirstLookup {
                release: release.clone(),
                released: std::sync::atomic::AtomicBool::new(false),
            },
            Some(&quick(Some(Duration::from_secs(20)))),
            &Quiet,
        )
        .unwrap();
        assert_eq!(waited, StopOutcome::Exited(record));
        assert!(matches!(
            settled(root),
            DrainState::NotRunning(NotRunning::Dead(_))
        ));
        let mut decoy = decoy;
        assert!(
            decoy.0.try_wait().unwrap().is_none(),
            "the decoy was not touched"
        );
    }

    #[test]
    fn a_missing_dead_or_recycled_pid_file_is_not_running_and_signals_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let status = |state: &DrainState| crate::display::drain_status(state, false);

        // No file.
        let state = drain_state(root, &RealProcs);
        assert_eq!(state, DrainState::NotRunning(NotRunning::Missing));
        assert_eq!(state.exit_code(), 2);
        assert!(status(&state).contains("no drain is running"));
        let stopped = stop_and_wait(root, &RealProcs, None, &Quiet).unwrap();
        assert_eq!(stopped, StopOutcome::NotRunning(NotRunning::Missing));
        assert!(stop_file(root).exists(), "the stop is still asked for");

        // A file that is not a pid record.
        std::fs::write(pid_file(root), "not json").unwrap();
        let state = drain_state(root, &RealProcs);
        assert!(
            matches!(state, DrainState::NotRunning(NotRunning::Unreadable(_))),
            "{state:?}"
        );
        assert_eq!(state.exit_code(), 2);

        // A process that exited and was reaped.
        let mut gone = sleeper("athena drain");
        let record = record_of(gone.id(), BUILD);
        gone.0.kill().unwrap();
        gone.0.wait().unwrap();
        write_record(root, &record).unwrap();
        let state = drain_state(root, &RealProcs);
        assert_eq!(state, DrainState::NotRunning(NotRunning::Dead(record.pid)));
        assert!(status(&state).contains("has exited"));

        // A process that exited and is not yet reaped (a zombie) is not the drain either.
        let zombie = sleeper("athena drain");
        let record = record_of(zombie.id(), BUILD);
        let mut zombie = zombie;
        zombie.0.kill().unwrap();
        write_record(root, &record).unwrap();
        assert_eq!(
            settled(root),
            DrainState::NotRunning(NotRunning::Dead(record.pid))
        );
        drop(zombie);

        // A pid that another process holds now: the file's marker is from an earlier process.
        let mut other = sleeper("athena drain --budget 5");
        let recycled = PidRecord {
            started: "1".into(),
            ..record_of(other.id(), BUILD)
        };
        write_record(root, &recycled).unwrap();
        let state = drain_state(root, &RealProcs);
        assert_eq!(
            state,
            DrainState::NotRunning(NotRunning::Recycled(other.id()))
        );
        assert_eq!(state.exit_code(), 2);
        assert!(status(&state).contains("different process"));
        let stopped = stop_and_wait(
            root,
            &RealProcs,
            Some(&quick(Some(Duration::from_secs(5)))),
            &Quiet,
        )
        .unwrap();
        assert_eq!(
            stopped,
            StopOutcome::NotRunning(NotRunning::Recycled(other.id()))
        );
        assert!(
            other.0.try_wait().unwrap().is_none(),
            "the other process was not signalled"
        );
    }

    #[test]
    fn a_stale_binary_start_is_refused_and_dispatches_no_job() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        std::fs::write(stop_file(root), "stop\n").unwrap();
        let stub = Stub {
            calls: RefCell::new(Vec::new()),
            read_cost: 0.1,
            run_cost: 0.1,
            outcome: OutcomeState::Landed,
        };
        let stale = || {
            Some(
                "stale build: this athena was built from aaaaaaaaaaaa, and 3 file(s) under crates/, xtask/, Cargo.toml, Cargo.lock or setup.sh changed by bbbbbbbbbbbb; run ./setup.sh"
                    .to_string(),
            )
        };
        let error = guarded(root, &RealProcs, BUILD, &stale, || {
            drain(&stub, limits(5.0), &Quiet)
        })
        .unwrap_err()
        .to_string();
        assert!(error.contains("refusing to start the drain"), "{error}");
        assert!(error.contains("nothing was dispatched"), "{error}");
        assert!(
            error.contains("aaaaaaaaaaaa") && error.contains("bbbbbbbbbbbb"),
            "{error}"
        );
        assert!(error.contains("./setup.sh"), "{error}");
        assert!(stub.calls.borrow().is_empty(), "{:?}", stub.calls.borrow());
        assert!(!pid_file(root).exists());
        assert!(stop_file(root).exists(), "a refused start changes nothing");

        // The same start with a build that is not stale goes ahead.
        guarded(root, &RealProcs, BUILD, &|| None, || {
            drain(&stub, limits(5.0), &Quiet)
        })
        .unwrap();
        assert!(!stub.calls.borrow().is_empty());
        assert!(
            !pid_file(root).exists(),
            "the pid file is removed when the run ends"
        );
    }

    #[test]
    fn the_pid_file_is_written_at_start_removed_on_exit_and_a_second_live_start_is_refused() {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        let own = std::process::id();
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let run = |root: &Path, f: &mut dyn FnMut()| {
            guarded(root, &RealProcs, BUILD, &|| None, || {
                f();
                Ok(())
            })
        };

        // (1) Written at start, removed at exit.
        let mut inside = None;
        run(root, &mut || {
            inside = std::fs::read_to_string(pid_file(root)).ok();
        })
        .unwrap();
        let written: PidRecord =
            serde_json::from_str(&inside.expect("written while running")).unwrap();
        assert_eq!((written.pid, written.build.as_str()), (own, BUILD));
        assert_eq!(Some(written.started), RealProcs.start_marker(own));
        assert!(!pid_file(root).exists());

        // (2) A live drain is named in the file: the second start is refused and changes nothing.
        let live = sleeper("athena drain");
        let first = record_of(live.id(), BUILD);
        write_record(root, &first).unwrap();
        std::fs::write(stop_file(root), "stop\n").unwrap();
        let mut ran = false;
        let error = run(root, &mut || ran = true).unwrap_err().to_string();
        assert!(
            error.contains(&format!("a drain is already running (pid {}", live.id())),
            "{error}"
        );
        assert!(!ran);
        assert!(
            stop_file(root).exists(),
            "the first drain's pending stop is kept"
        );
        let kept: PidRecord =
            serde_json::from_str(&std::fs::read_to_string(pid_file(root)).unwrap()).unwrap();
        assert_eq!(kept, first, "the first drain's file is not touched");
        drop(live);

        // (3) The file names this very process: the drain carrying on after its own rebuild. It is taken
        // over and the stop file, which belongs to this run, stays.
        write_record(root, &record_of(own, "athena 0.0.0 (commit 000000000000)")).unwrap();
        let mut stop_seen = false;
        run(root, &mut || stop_seen = stop_file(root).exists()).unwrap();
        assert!(stop_seen, "a stop asked for before the exec is kept");
        assert!(!pid_file(root).exists());

        // (4) A panic still removes the file.
        let caught = catch_unwind(AssertUnwindSafe(|| {
            let _ = run(root, &mut || panic!("the drain panicked"));
        }));
        assert!(caught.is_err());
        assert!(!pid_file(root).exists());

        // (5) A leftover naming a dead process is replaced, and an old stop file is removed.
        let mut dead = sleeper("athena drain");
        write_record(root, &record_of(dead.id(), BUILD)).unwrap();
        dead.0.kill().unwrap();
        dead.0.wait().unwrap();
        let mut seen = (None, true);
        run(root, &mut || {
            seen = (
                std::fs::read_to_string(pid_file(root)).ok(),
                stop_file(root).exists(),
            )
        })
        .unwrap();
        assert!(seen.0.unwrap().contains(&format!("\"pid\": {own}")));
        assert!(!seen.1, "a stop left by an earlier run is removed");

        // (6) No file at all, and an old stop file.
        std::fs::write(stop_file(root), "stop\n").unwrap();
        let mut stop_seen = true;
        run(root, &mut || stop_seen = stop_file(root).exists()).unwrap();
        assert!(!stop_seen);
    }

    /// The `Lifecycle` of the restart tests: it logs each step to a file that the old drain's script also
    /// writes to, so the order of events across processes is the order in the file.
    struct Rec {
        root: PathBuf,
        rebuilds: RefCell<VecDeque<Result<Rebuild, String>>>,
        /// The count of the checkout that the installed binary was built from.
        built: std::cell::Cell<usize>,
        /// The spawned drain exits at once instead of taking the pid file.
        exits_at_once: bool,
        children: RefCell<Vec<Reaped>>,
    }

    impl Rec {
        fn new(root: &Path, rebuilds: Vec<Result<Rebuild, String>>) -> Rec {
            Rec {
                root: root.to_path_buf(),
                rebuilds: RefCell::new(rebuilds.into()),
                built: std::cell::Cell::new(0),
                exits_at_once: false,
                children: RefCell::new(Vec::new()),
            }
        }
        fn event(&self, what: &str) {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(self.root.join("events"))
                .unwrap();
            writeln!(file, "{what}").unwrap();
        }
        fn events(&self) -> Vec<String> {
            std::fs::read_to_string(self.root.join("events"))
                .unwrap_or_default()
                .lines()
                .map(String::from)
                .collect()
        }
        /// The checkout moved on when the old drain landed something as it ended.
        fn head(&self) -> usize {
            usize::from(self.root.join("landed").exists())
        }
    }

    impl Lifecycle for Rec {
        fn state_root(&self) -> &Path {
            &self.root
        }
        fn procs(&self) -> &dyn Procs {
            &RealProcs
        }
        fn rebuild(&self) -> Result<Rebuild, String> {
            self.event("rebuild");
            let result = self
                .rebuilds
                .borrow_mut()
                .pop_front()
                .expect("a scripted rebuild");
            if result.is_ok() {
                self.built.set(self.head());
            }
            result
        }
        fn installed_stale(&self) -> Option<String> {
            (self.built.get() < self.head())
                .then(|| "stale build: the installed binary is behind the checkout".to_string())
        }
        fn spawn(&self) -> Result<Spawned, String> {
            self.event("spawn");
            let log = log_file(&self.root);
            if self.exits_at_once {
                std::fs::write(&log, "refusing to start the drain: boom\n").unwrap();
                let child = Reaped(std::process::Command::new("true").spawn().unwrap());
                let pid = child.id();
                self.children.borrow_mut().push(child);
                return Ok(Spawned { pid, log });
            }
            let child = sleeper("athena drain --budget 5");
            let pid = child.id();
            write_record(&self.root, &record_of(pid, BUILD)).unwrap();
            self.children.borrow_mut().push(child);
            Ok(Spawned { pid, log })
        }
    }

    /// An old drain that leaves when the stop file appears, writing `old exited` first and, when `lands`,
    /// moving the checkout on as its last act.
    fn old_drain(root: &Path, lands: bool) -> Reaped {
        use std::os::unix::process::CommandExt;
        let script = r#"while [ ! -e "$1" ]; do sleep 0.05; done; echo 'old exited' >> "$2"; if [ "$3" = land ]; then : > "$4"; fi"#;
        let child = Reaped(
            std::process::Command::new("sh")
                .arg0("athena drain --budget 5")
                .args(["-c", script, "sh"])
                .arg(stop_file(root))
                .arg(root.join("events"))
                .arg(if lands { "land" } else { "no" })
                .arg(root.join("landed"))
                .spawn()
                .unwrap(),
        );
        write_record(
            root,
            &record_of(child.id(), "athena 0.0.0 (commit 111111111111)"),
        )
        .unwrap();
        child
    }

    fn installed() -> Result<Rebuild, String> {
        Ok(Rebuild::Installed)
    }

    #[test]
    fn restart_rebuilds_then_waits_for_the_old_drain_then_starts_and_names_the_build() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let old = old_drain(root, false);
        let life = Rec::new(root, vec![installed()]);
        let ui = Kept::default();
        let outcome = restart(&life, &quick(Some(Duration::from_secs(20))), &ui).unwrap();
        assert_eq!(life.events(), ["rebuild", "old exited", "spawn"]);
        let RestartOutcome::Started(report) = outcome else {
            panic!("expected a started drain: {outcome:?}");
        };
        assert_eq!(report.old.as_ref().map(|o| o.pid), Some(old.id()));
        assert_eq!(report.new.build, BUILD);
        assert_eq!(
            Some(report.new.pid),
            life.children.borrow().first().map(Reaped::id)
        );
        let text = crate::display::drain_restart(&RestartOutcome::Started(report));
        assert!(text.contains(BUILD), "{text}");
        assert!(!ui.0.lock().unwrap().is_empty(), "progress is told");
    }

    #[test]
    fn restart_with_a_failed_rebuild_starts_nothing_and_leaves_the_old_drain_alone() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let mut old = old_drain(root, false);
        let life = Rec::new(root, vec![Err("setup.sh failed: boom".into())]);
        let error = format!("{:#}", restart(&life, &quick(None), &Quiet).unwrap_err());
        assert!(
            error.contains("boom") && error.contains("left alone"),
            "{error}"
        );
        assert_eq!(life.events(), ["rebuild"]);
        assert!(!stop_file(root).exists(), "no stop was asked for");
        assert!(
            old.0.try_wait().unwrap().is_none(),
            "the old drain still runs"
        );
        assert!(life.children.borrow().is_empty(), "nothing was spawned");
    }

    #[test]
    fn restart_builds_again_when_the_old_drain_landed_before_it_exited() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let _old = old_drain(root, true);
        let life = Rec::new(root, vec![installed(), installed()]);
        let outcome = restart(&life, &quick(Some(Duration::from_secs(20))), &Quiet).unwrap();
        assert!(matches!(outcome, RestartOutcome::Started(_)));
        assert_eq!(life.events(), ["rebuild", "old exited", "rebuild", "spawn"]);
    }

    #[test]
    fn restart_starts_nothing_when_the_second_rebuild_fails() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let _old = old_drain(root, true);
        let life = Rec::new(root, vec![installed(), Err("second build broke".into())]);
        let error = format!(
            "{:#}",
            restart(&life, &quick(Some(Duration::from_secs(20))), &Quiet).unwrap_err()
        );
        assert!(error.contains("second build broke"), "{error}");
        assert!(
            error.contains("no drain is running") && error.contains("./setup.sh"),
            "{error}"
        );
        assert_eq!(life.events(), ["rebuild", "old exited", "rebuild"]);
        assert!(life.children.borrow().is_empty());
    }

    #[test]
    fn restart_that_times_out_says_the_stop_is_asked_for_and_starts_nothing() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let busy = sleeper("athena drain --budget 5");
        write_record(root, &record_of(busy.id(), BUILD)).unwrap();
        let life = Rec::new(root, vec![installed()]);
        let outcome = restart(&life, &quick(Some(Duration::from_millis(200))), &Quiet).unwrap();
        let RestartOutcome::StillStopping(old, _) = &outcome else {
            panic!("expected a timeout: {outcome:?}");
        };
        assert_eq!(old.pid, busy.id());
        let text = crate::display::drain_restart(&outcome);
        assert!(
            text.contains("still asked for") && text.contains("no new drain was started"),
            "{text}"
        );
        assert!(text.contains("athena drain-restart"), "{text}");
        assert!(stop_file(root).exists(), "the stop stays asked for");
        assert_eq!(life.events(), ["rebuild"], "no second rebuild and no spawn");
    }

    #[test]
    fn restart_with_no_pid_file_says_an_older_drain_cannot_be_seen_and_writes_no_stop_file() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let life = Rec::new(root, vec![installed()]);
        let ui = Kept::default();
        let outcome = restart(&life, &quick(None), &ui).unwrap();
        let RestartOutcome::Started(report) = outcome else {
            panic!("expected a started drain: {outcome:?}");
        };
        assert!(report.old.is_none());
        assert_eq!(life.events(), ["rebuild", "spawn"]);
        assert!(!stop_file(root).exists());
        let lines = ui.0.lock().unwrap().join("\n");
        assert!(lines.contains("before the pid file existed"), "{lines}");
    }

    #[test]
    fn restart_names_the_log_when_the_new_drain_exits_at_once() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let mut life = Rec::new(root, vec![installed()]);
        life.exits_at_once = true;
        let error = format!("{:#}", restart(&life, &quick(None), &Quiet).unwrap_err());
        assert!(error.contains("drain.log"), "{error}");
        assert!(
            error.contains("refusing to start the drain: boom"),
            "{error}"
        );
        assert!(error.contains("no drain is running"), "{error}");
    }
}

/// The in-flight table file: what a missing, a good and a bad file give, and how rows change.
#[cfg(test)]
mod in_flight_tests {
    use super::*;

    fn row(job: &str, stage: Stage) -> InFlightRow {
        InFlightRow {
            job: job.into(),
            stage,
            worktree: PathBuf::from(format!("/state/jobs/{job}/work")),
        }
    }

    #[test]
    fn a_missing_file_is_an_empty_table_and_stays_missing() {
        let dir = tempfile::tempdir().unwrap();
        let table = InFlight::at(dir.path());
        assert_eq!(table.read(), InFlightRead::default());
        assert!(!table.path().exists());
    }

    #[test]
    fn a_set_row_is_replaced_by_the_next_set_for_the_same_job() {
        let dir = tempfile::tempdir().unwrap();
        let table = InFlight::at(dir.path());
        table.set(row("a", Stage::Producer)).unwrap();
        table.set(row("b", Stage::Checks)).unwrap();
        table.set(row("a", Stage::Verifier)).unwrap();
        let rows = table.read().rows;
        assert_eq!(rows.len(), 2);
        assert!(rows.contains(&row("a", Stage::Verifier)));
        assert!(rows.contains(&row("b", Stage::Checks)));
        // The file holds the job, the stage as a word and the worktree.
        let text = std::fs::read_to_string(table.path()).unwrap();
        assert!(text.contains("\"stage\": \"verifier\"") && text.contains("\"worktree\""));
    }

    #[test]
    fn a_moved_row_keeps_its_stage_and_takes_the_new_job_and_worktree() {
        let dir = tempfile::tempdir().unwrap();
        let table = InFlight::at(dir.path());
        table.set(row("old", Stage::Verifier)).unwrap();
        table.set(row("other", Stage::Producer)).unwrap();
        table
            .move_row("old", "new", PathBuf::from("/state/jobs/new/work"))
            .unwrap();
        let rows = table.read().rows;
        assert!(rows.contains(&InFlightRow {
            job: "new".into(),
            stage: Stage::Verifier,
            worktree: PathBuf::from("/state/jobs/new/work"),
        }));
        assert!(rows.contains(&row("other", Stage::Producer)));
        assert_eq!(rows.len(), 2);
        // Moving a job that has no row changes nothing.
        table.move_row("absent", "x", PathBuf::from("/x")).unwrap();
        assert_eq!(table.read().rows.len(), 2);
    }

    #[test]
    fn removing_the_last_row_removes_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let table = InFlight::at(dir.path());
        table.set(row("a", Stage::Landing)).unwrap();
        table.remove("b").unwrap();
        assert_eq!(table.read().rows.len(), 1);
        table.remove("a").unwrap();
        assert!(table.read().rows.is_empty());
        assert!(!table.path().exists());
        // Removing from a missing file is not an error.
        table.remove("a").unwrap();
    }

    #[test]
    fn a_file_that_is_not_a_table_is_moved_aside_and_reported_once() {
        let dir = tempfile::tempdir().unwrap();
        let table = InFlight::at(dir.path());
        std::fs::write(table.path(), "[{\"job\": 1}").unwrap();
        let found = table.read();
        assert!(found.rows.is_empty());
        let why = found.corrupt.expect("a corrupt note");
        assert!(why.contains("in-flight.json.corrupt"), "{why}");
        assert!(!table.path().exists());
        assert_eq!(
            std::fs::read_to_string(dir.path().join("in-flight.json.corrupt")).unwrap(),
            "[{\"job\": 1}"
        );
        // The next read finds an empty table, and a write starts a fresh one.
        assert_eq!(table.read(), InFlightRead::default());
        table.set(row("a", Stage::Producer)).unwrap();
        assert_eq!(table.read().rows, [row("a", Stage::Producer)]);
    }

    #[test]
    fn a_write_into_a_bad_file_keeps_the_bad_file_aside_and_writes_the_row() {
        let dir = tempfile::tempdir().unwrap();
        let table = InFlight::at(dir.path());
        std::fs::write(table.path(), "garbage").unwrap();
        table.set(row("a", Stage::Checks)).unwrap();
        assert_eq!(table.read().rows, [row("a", Stage::Checks)]);
        assert_eq!(
            std::fs::read_to_string(dir.path().join("in-flight.json.corrupt")).unwrap(),
            "garbage"
        );
    }
}

/// Read-ahead and the stale-reading re-check, driven through the real entry points ([`drain_concurrent`] and
/// [`drain`]) with a fake [`Steps`] that only logs calls and supplies data. The pump, the window, the claim and
/// the order of the drain's calls are the production code. The tests assert on the order of the log, which the
/// dependency chain and the window force, and not on elapsed time; the fake's waits are capped, so a missing
/// behavior fails an assertion instead of hanging.
#[cfg(test)]
mod read_ahead_tests {
    use super::*;
    use athena_core::Quiet;
    use athena_data::roles::Ladder;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Instant;

    fn own(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    fn entry(id: &str, touches: &[&str], deps: &[&str]) -> PlanEntry {
        PlanEntry {
            goal: format!("goal of {id}"),
            rung: Some(Ladder::Implementation),
            acceptance: vec![format!("{id} works")],
            depends_on: own(deps),
            touches: own(touches),
            ..PlanEntry::bare(id, &format!("title of {id}"))
        }
    }

    fn limits(budget: f64) -> Limits {
        Limits {
            budget_usd: budget,
            target_rating: 4,
            max_jobs: 20,
        }
    }

    /// Steps that log every call. Reading costs 0.1, a run 0.3, a plan made ahead 0.2.
    struct World {
        log: Mutex<Vec<String>>,
        /// Each item with the paths its plan says it touches and what it depends on, oldest first.
        items: Vec<(String, Vec<String>, Vec<String>)>,
        window: u32,
        /// Items whose `plan_ahead` says they are hard and were planned.
        hard: Vec<String>,
        /// What a landing of a job changed; by default its own source file and the changelog.
        landed: BTreeMap<String, Vec<String>>,
        /// The job a reading names, when it is not the item.
        named: BTreeMap<String, String>,
        /// `run(job)` waits (at most 5 s) until these log entries exist, then keeps running for 200 ms.
        gate: BTreeMap<String, Vec<String>>,
        recheck_holds: bool,
        /// Items whose reading comes back not runnable, with an error, however often it is made.
        refuses: BTreeSet<String>,
        stop_after: Option<String>,
        /// Jobs whose landing is a merge the host holds until checks finish.
        waiting: BTreeSet<String>,
        /// Jobs whose land step ends with this pull request state and checks, as `land_pr` reports them once
        /// its poll is over. The landing is what the production [`landing_from_report`] makes of that report.
        reports: BTreeMap<String, (LinkState, Option<Checks>)>,
        reads: AtomicUsize,
        ended: AtomicUsize,
        /// The most readings made but not yet through a run, seen when a read started.
        widest: AtomicUsize,
    }

    impl World {
        fn new(window: u32, items: &[(&str, &[&str], &[&str])]) -> World {
            World {
                log: Mutex::new(Vec::new()),
                items: items
                    .iter()
                    .map(|(id, touches, deps)| (id.to_string(), own(touches), own(deps)))
                    .collect(),
                window,
                hard: Vec::new(),
                landed: BTreeMap::new(),
                named: BTreeMap::new(),
                gate: BTreeMap::new(),
                recheck_holds: true,
                refuses: BTreeSet::new(),
                stop_after: None,
                waiting: BTreeSet::new(),
                reports: BTreeMap::new(),
                reads: AtomicUsize::new(0),
                ended: AtomicUsize::new(0),
                widest: AtomicUsize::new(0),
            }
        }

        fn waits(mut self, job: &str) -> World {
            self.waiting.insert(job.into());
            self
        }

        /// Every reading of `item` is refused: not runnable, with an error.
        fn refusing(mut self, item: &str) -> World {
            self.refuses.insert(item.into());
            self
        }

        /// The job's single land call ends with a pull request in `state` whose checks read `checks`.
        fn ends_with(mut self, job: &str, state: LinkState, checks: Checks) -> World {
            self.reports.insert(job.into(), (state, Some(checks)));
            self
        }

        /// a, then b, then c, each touching only its own file and the changelog.
        fn chain(window: u32) -> World {
            World::new(
                window,
                &[
                    ("a", &["src/a.rs", "CHANGELOG.md"], &[]),
                    ("b", &["src/b.rs", "CHANGELOG.md"], &["a"]),
                    ("c", &["src/c.rs", "CHANGELOG.md"], &["b"]),
                ],
            )
        }

        fn gate(mut self, job: &str, until: &[&str]) -> World {
            self.gate.insert(job.into(), own(until));
            self
        }

        fn hard(mut self, item: &str) -> World {
            self.hard.push(item.into());
            self
        }

        fn lands(mut self, job: &str, changed: &[&str]) -> World {
            self.landed.insert(job.into(), own(changed));
            self
        }

        fn names(mut self, item: &str, job: &str) -> World {
            self.named.insert(item.into(), job.into());
            self
        }

        fn log(&self, entry: String) {
            self.log.lock().unwrap().push(entry);
        }

        fn entries(&self) -> Vec<String> {
            self.log.lock().unwrap().clone()
        }

        fn logged(&self, entry: &str) -> bool {
            self.log.lock().unwrap().iter().any(|e| e == entry)
        }

        fn count(&self, entry: &str) -> usize {
            self.entries().iter().filter(|e| *e == entry).count()
        }

        /// The place of `entry` in the log; fails the test when it was never logged.
        fn at(&self, entry: &str) -> usize {
            self.entries()
                .iter()
                .position(|e| e == entry)
                .unwrap_or_else(|| panic!("{entry:?} was never logged: {:?}", self.entries()))
        }
    }

    impl Steps for World {
        fn caps(&self) -> anyhow::Result<Caps> {
            Ok(Caps {
                read: 0.5,
                run: 1.0,
                rate: 0.5,
            })
        }
        fn queue(&self) -> anyhow::Result<QueueView> {
            Ok(QueueView {
                items: self
                    .items
                    .iter()
                    .map(|(id, _, _)| Item::queued(id, "t"))
                    .collect(),
                plan: self
                    .items
                    .iter()
                    .map(|(id, touches, deps)| PlanEntry {
                        touches: touches.clone(),
                        depends_on: deps.clone(),
                        ..entry(id, &[], &[])
                    })
                    .collect(),
                ..QueueView::default()
            })
        }
        fn previous_rating(&self) -> Option<u8> {
            None
        }
        fn approve(&self, _: &str) -> Approval {
            Approval::default()
        }
        fn read(&self, what: ReadOf<'_>) -> Read {
            let ReadOf::Queued(item) = what else {
                panic!("only a first reading is expected, got {what:?}");
            };
            self.log(format!("read {item}"));
            let made = self.reads.fetch_add(1, Ordering::SeqCst) + 1;
            let open = made.saturating_sub(self.ended.load(Ordering::SeqCst));
            self.widest.fetch_max(open, Ordering::SeqCst);
            if self.refuses.contains(item) {
                return Read {
                    job: None,
                    cost: 0.1,
                    runnable: false,
                    error: Some(format!("{item} cannot be read")),
                    held: false,
                };
            }
            Read {
                job: Some(
                    self.named
                        .get(item)
                        .map_or(item, String::as_str)
                        .to_string(),
                ),
                cost: 0.1,
                runnable: true,
                error: None,
                held: false,
            }
        }
        fn run(&self, job: &str) -> Run {
            self.log(format!("start {job}"));
            if let Some(until) = self.gate.get(job) {
                let give_up = Instant::now() + Duration::from_secs(5);
                while !until.iter().all(|e| self.logged(e)) && Instant::now() < give_up {
                    std::thread::sleep(Duration::from_millis(5));
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            self.log(format!("end {job}"));
            self.ended.fetch_add(1, Ordering::SeqCst);
            Run {
                outcome: Some(OutcomeState::Landed),
                cost: 0.3,
                ..Run::default()
            }
        }
        fn land(&self, job: &str) -> Landing {
            self.log(format!("land {job}"));
            if self.waiting.contains(job) {
                return Landing::Waiting("the host holds the merge until checks finish".into());
            }
            let landing = match self.reports.get(job) {
                Some((state, checks)) => landing_from_report(Ok(PrReport {
                    link: athena_core::pr::PrLink {
                        pr_number: 7,
                        host: "github.com".into(),
                        repository: "example/project".into(),
                        head: "abc".into(),
                        job: job.into(),
                        state: *state,
                        merged_commit: (*state == LinkState::Landed).then(|| "def".into()),
                    },
                    checks: *checks,
                    merge_requested: *state == LinkState::Landed,
                    reruns: 0,
                    failing_test: None,
                    warnings: Vec::new(),
                })),
                None => Landing::Landed {
                    changed: Vec::new(),
                },
            };
            match landing {
                // As `RealSteps::land` does, take the paths from the job's record, here the scripted ones.
                Landing::Landed { .. } => {
                    Landing::Landed {
                        changed: self.landed.get(job).cloned().unwrap_or_else(|| {
                            vec![format!("src/{job}.rs"), "CHANGELOG.md".into()]
                        }),
                    }
                }
                other => other,
            }
        }
        fn rate(&self) -> Rate {
            Rate {
                rating: Some(1),
                ..Rate::default()
            }
        }
        fn check_page(&self) -> Check {
            Check::pass("fake")
        }
        fn check_cap(&self) -> f64 {
            0.5
        }
        fn stop_requested(&self) -> bool {
            self.stop_after
                .as_ref()
                .is_some_and(|job| self.logged(&format!("end {job}")))
        }
        fn read_ahead(&self) -> u32 {
            self.window
        }
        fn plan_cap(&self) -> f64 {
            0.2
        }
        fn plan_ahead(&self, item: &str) -> Option<Planned> {
            self.log(format!("plan_ahead {item}"));
            self.hard.iter().any(|h| h == item).then(|| {
                self.log(format!("plan {item}"));
                Planned {
                    cost: 0.2,
                    error: None,
                }
            })
        }
        fn recheck(&self, item: &str, changed: &[String]) -> bool {
            self.log(format!("recheck {item} [{}]", changed.join(",")));
            self.recheck_holds
        }
        fn reread(&self, item: &str) -> Read {
            self.log(format!("reread {item}"));
            Read {
                job: Some(format!("{item}-fresh")),
                cost: 0.1,
                runnable: true,
                error: None,
                held: false,
            }
        }
    }

    fn concurrent(world: &World, bound: u32) -> Report {
        drain_concurrent(world, limits(20.0), &Quiet, bound).unwrap()
    }

    #[test]
    fn a_job_waiting_to_merge_is_not_a_failure_and_the_run_goes_on_to_the_next_job() {
        // Independent items: a's pull request waits on its checks, b lands.
        let world =
            World::new(0, &[("a", &["src/a.rs"], &[]), ("b", &["src/b.rs"], &[])]).waits("a");
        let report = concurrent(&world, 2);
        assert_eq!(report.stop, Stop::QueueEmpty, "{:?}", report.jobs);
        // a is landed once, not tried again; b was worked and landed.
        assert_eq!(world.count("land a"), 1);
        assert!(world.logged("land b"));
        assert_eq!(report.landed, 1);
        let a = report.jobs.iter().find(|j| j.item == "a").unwrap();
        assert!(a.landing.starts_with("waiting to merge"), "{}", a.landing);
        assert!(!a.landing.starts_with("not landed"), "{}", a.landing);
        let b = report.jobs.iter().find(|j| j.item == "b").unwrap();
        assert_eq!(b.landing, "landed");
    }

    // The next three tests drive the drain's own land step. The scripted pull request state is what `land_pr`
    // reports when its poll is over: the poll itself (and the merge on a green read inside it) is tested in
    // `athena_core::gh`, where `land_pr` runs. Here the drain must take that one call as the whole landing.

    #[test]
    fn a_job_whose_checks_turn_green_during_the_poll_lands_in_the_same_step() {
        // The checks were pending when the poll began and green partway through: the one land call merged.
        let world = World::new(0, &[("a", &["src/a.rs"], &[])]).ends_with(
            "a",
            LinkState::Landed,
            Checks::Green,
        );
        let report = concurrent(&world, 2);
        assert_eq!(report.stop, Stop::QueueEmpty, "{:?}", report.jobs);
        assert_eq!(world.count("land a"), 1, "no second land call");
        assert_eq!(report.landed, 1);
        let a = report.jobs.iter().find(|j| j.item == "a").unwrap();
        assert_eq!(a.landing, "landed");
        assert!(
            report
                .jobs
                .iter()
                .all(|j| !j.landing.contains("waiting to merge")),
            "{:?}",
            report.jobs
        );
    }

    #[test]
    fn a_job_whose_checks_are_still_pending_at_the_end_of_the_poll_is_waiting_and_not_retried() {
        let world = World::new(0, &[("a", &["src/a.rs"], &[]), ("b", &["src/b.rs"], &[])])
            .ends_with("a", LinkState::Open, Checks::Pending);
        let report = concurrent(&world, 2);
        assert_eq!(report.stop, Stop::QueueEmpty, "{:?}", report.jobs);
        assert_eq!(world.count("land a"), 1, "not tried again");
        assert_eq!(report.landed, 1, "only b landed");
        let a = report.jobs.iter().find(|j| j.item == "a").unwrap();
        assert!(a.landing.starts_with("waiting to merge"), "{}", a.landing);
        assert!(!a.landing.starts_with("not landed"), "{}", a.landing);
    }

    #[test]
    fn a_job_whose_checks_are_red_at_the_end_of_the_poll_is_not_landed() {
        let world = World::new(0, &[("a", &["src/a.rs"], &[])]).ends_with(
            "a",
            LinkState::Open,
            Checks::Red,
        );
        let report = concurrent(&world, 2);
        // A failed landing is tried twice, like any failed job; a waiting one is tried once.
        assert_eq!(world.count("land a"), 2);
        assert_eq!(report.landed, 0);
        let a = report.jobs.iter().find(|j| j.item == "a").unwrap();
        assert!(a.landing.starts_with("not landed"), "{}", a.landing);
        assert!(!a.landing.contains("waiting to merge"), "{}", a.landing);
    }

    #[test]
    fn a_merge_refused_for_rule_violations_while_checks_are_pending_is_reported_as_waiting() {
        let report = |state, checks, merge_requested| {
            Ok(PrReport {
                link: athena_core::pr::PrLink {
                    pr_number: 7,
                    host: "github.com".into(),
                    repository: "example/project".into(),
                    head: "abc".into(),
                    job: "j".into(),
                    state,
                    merged_commit: None,
                },
                checks,
                merge_requested,
                reruns: 0,
                failing_test: None,
                warnings: vec!["Repository rule violations found".into()],
            })
        };
        // The refused merge: open, checks pending, merge asked for.
        let waiting = landing_from_report(report(LinkState::Open, Some(Checks::Pending), true));
        assert!(matches!(waiting, Landing::Waiting(_)), "{waiting:?}");
        // Red checks, or an unread state, are still a failed landing.
        for checks in [Some(Checks::Red), Some(Checks::Green), None] {
            let landing = landing_from_report(report(LinkState::Open, checks, false));
            assert!(matches!(landing, Landing::NotLanded(_)), "{landing:?}");
        }
        let landing = landing_from_report(Err(anyhow::anyhow!("boom")));
        assert!(matches!(landing, Landing::NotLanded(_)), "{landing:?}");
        let landed = landing_from_report(report(LinkState::Landed, Some(Checks::Green), true));
        assert!(matches!(landed, Landing::Landed { .. }), "{landed:?}");
    }

    #[test]
    fn read_ahead_reads_and_plans_the_next_hard_job_while_the_current_one_runs() {
        // a runs until b has been read and planned; b runs until c has been read, and c is not hard.
        let world = World::chain(1)
            .hard("b")
            .gate("a", &["read b", "plan b"])
            .gate("b", &["read c", "plan_ahead c"]);
        let report = concurrent(&world, 2);
        assert_eq!(report.landed, 3, "{:?} {:?}", report.jobs, report.notes);
        assert_eq!(report.stop, Stop::QueueEmpty);
        // b is read and planned while a runs.
        assert!(world.at("start a") < world.at("end a"));
        assert!(world.at("read b") < world.at("plan_ahead b"));
        assert!(world.at("plan_ahead b") < world.at("plan b"));
        assert!(world.at("plan b") < world.at("end a"));
        // c is read only after b took b's place in the window (a ended), and while b runs.
        assert!(world.at("end a") < world.at("read c"));
        assert!(world.at("read c") < world.at("end b"));
        // c is not hard: the drain asked, and nothing was planned for it.
        assert!(world.logged("plan_ahead c"));
        assert!(!world.logged("plan c"));
        // Each item is read once; the ahead readings were run as they were.
        for item in ["a", "b", "c"] {
            assert_eq!(world.count(&format!("read {item}")), 1, "{item}");
        }
        // The running job plus one reading ahead is the most there ever is.
        assert!(world.widest.load(Ordering::SeqCst) <= 2);
        // The money: three readings, three runs, and the plan of b only.
        let expected = 3.0 * 0.1 + 3.0 * 0.3 + 0.2;
        assert!(
            (report.spent_usd - expected).abs() < 1e-9,
            "{} vs {expected}",
            report.spent_usd
        );
        // The ahead cost is on the job's line.
        assert!((report.jobs[1].cost_usd - (0.1 + 0.2 + 0.3)).abs() < 1e-9);
    }

    #[test]
    fn read_ahead_never_runs_further_ahead_than_the_window() {
        let world = World::new(
            2,
            &[
                ("a", &["src/a.rs"], &[]),
                ("b", &["src/b.rs"], &["a"]),
                ("c", &["src/c.rs"], &["b"]),
                ("d", &["src/d.rs"], &["c"]),
            ],
        )
        .gate("a", &["read b", "read c"]);
        let report = concurrent(&world, 2);
        assert_eq!(report.landed, 4);
        // b and c are read while a runs; d waits for a place in the window, which b gives up once it starts.
        assert!(world.at("read b") < world.at("end a"));
        assert!(world.at("read c") < world.at("end a"));
        assert!(world.at("end a") < world.at("read d"));
        // Running job plus two readings ahead.
        assert!(world.widest.load(Ordering::SeqCst) <= 3);
    }

    #[test]
    fn read_ahead_is_not_skipped_for_jobs_that_share_only_changelog() {
        // Every job edits the changelog; that must not stop the next one from being read.
        let world = World::chain(1).gate("a", &["read b"]);
        let report = concurrent(&world, 2);
        assert_eq!(report.landed, 3);
        assert!(world.at("read b") < world.at("end a"));
    }

    #[test]
    fn read_ahead_skips_an_item_that_shares_paths_with_a_job_in_flight() {
        // b is inside the directory a edits and e's paths are unknown: neither is read beside a. c is clear.
        let world = World::new(
            1,
            &[
                ("a", &["src/shared"], &[]),
                ("b", &["src/shared/x.rs"], &["a"]),
                ("c", &["src/c.rs"], &["a"]),
                ("e", &[], &["a"]),
            ],
        )
        .gate("a", &["read c"]);
        let report = concurrent(&world, 2);
        assert_eq!(report.landed, 4, "{:?} {:?}", report.jobs, report.notes);
        assert!(world.at("read c") < world.at("end a"));
        assert!(world.at("end a") < world.at("read b"));
        assert!(world.at("end a") < world.at("read e"));
        for item in ["a", "b", "c", "e"] {
            assert_eq!(world.count(&format!("read {item}")), 1, "{item}");
        }
    }

    #[test]
    fn read_ahead_paths_rule_matches_the_plan_grouping() {
        let cases: [(&[&str], &[&str]); 11] = [
            (&["src/x"], &["src/x/y.rs"]),
            (&["src/x/y.rs"], &["src/x"]),
            (&["src/x/"], &["src/x/y.rs"]),
            (&["src/x"], &["src/xy.rs"]),
            (&["src/a.rs"], &["src/a.rs"]),
            (&["src/a.rs"], &["src/b.rs"]),
            (&["CHANGELOG.md"], &["src/b.rs"]),
            (&["CHANGELOG.md", "src/a.rs"], &["CHANGELOG.md", "src/b.rs"]),
            (&["docs/decisions.md"], &["README.md"]),
            (&[], &["src/b.rs"]),
            (
                &["README.md", "src/a/"],
                &["src/a/b.rs", "docs/decisions.md"],
            ),
        ];
        for (a, b) in cases {
            // Plan::build puts two items with no dependency in one group exactly when it finds them disjoint.
            let plan = Plan::build(&[entry("x", a, &[]), entry("y", b, &[])]);
            assert_eq!(
                paths_disjoint(&own(a), &own(b)),
                plan.groups.len() == 1,
                "{a:?} against {b:?}: {:?}",
                plan.groups
            );
        }
    }

    #[test]
    fn read_ahead_ends_with_the_drain_when_a_stop_is_requested() {
        let mut world = World::new(
            2,
            &[
                ("a", &["src/a.rs"], &[]),
                ("b", &["src/b.rs"], &["a"]),
                ("c", &["src/c.rs"], &["b"]),
                ("d", &["src/d.rs"], &["c"]),
            ],
        )
        .gate("a", &["read b", "read c"]);
        world.stop_after = Some("a".into());
        let started = Instant::now();
        let report = concurrent(&world, 2);
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "the drain did not end promptly"
        );
        assert_eq!(report.stop, Stop::Requested);
        assert_eq!(report.landed, 1);
        // Nothing is read once the stop was asked for (a ended, then the stop was seen), and b never ran.
        let end = world.at("end a");
        assert!(
            world
                .entries()
                .iter()
                .enumerate()
                .all(|(at, e)| !e.starts_with("read ") || at < end),
            "{:?}",
            world.entries()
        );
        assert!(!world.logged("start b"));
        // The readings made ahead and never used are reported.
        assert!(
            report
                .notes
                .iter()
                .any(|n| n.contains("made ahead but not used")),
            "{:?}",
            report.notes
        );
    }

    /// a, b, c and d, each depending on the one before and touching only its own file.
    fn long_chain(window: u32) -> World {
        World::new(
            window,
            &[
                ("a", &["src/a.rs"], &[]),
                ("b", &["src/b.rs"], &["a"]),
                ("c", &["src/c.rs"], &["b"]),
                ("d", &["src/d.rs"], &["c"]),
            ],
        )
    }

    #[test]
    fn read_ahead_reads_the_next_items_while_a_single_job_runs() {
        // One job at a time (a bound of 1), window 3. a is held until b, c and d have all been read, so
        // the log order read b, read c, read d, end a shows the next items are read while a runs.
        let world = long_chain(3).gate("a", &["read b", "read c", "read d"]);
        let report = concurrent(&world, 1);
        assert_eq!(report.landed, 4, "{:?} {:?}", report.jobs, report.notes);
        assert_eq!(report.stop, Stop::QueueEmpty);
        for item in ["b", "c", "d"] {
            assert!(
                world.at(&format!("read {item}")) < world.at("end a"),
                "{item}"
            );
            assert!(
                world.at(&format!("read {item}")) < world.at("land a"),
                "{item}"
            );
        }
        // Later jobs run on the readings made ahead: no item is read twice, and a is read first.
        for item in ["a", "b", "c", "d"] {
            assert_eq!(world.count(&format!("read {item}")), 1, "{item}");
        }
        // One job at a time, plus at most three readings held.
        assert!(world.widest.load(Ordering::SeqCst) <= 4);
        assert!(world.at("land a") < world.at("start b"));
    }

    #[test]
    fn read_ahead_reads_independent_items_while_a_single_job_runs() {
        // No dependencies and disjoint files, so the plan puts a, b, c and d in one group. With one job at a
        // time only a runs now; the rest of its group must still be offered to the read-ahead.
        let world = World::new(
            3,
            &[
                ("a", &["src/a.rs"], &[]),
                ("b", &["src/b.rs"], &[]),
                ("c", &["src/c.rs"], &[]),
                ("d", &["src/d.rs"], &[]),
            ],
        )
        .gate("a", &["read b", "read c", "read d"]);
        let report = concurrent(&world, 1);
        assert_eq!(report.landed, 4, "{:?} {:?}", report.jobs, report.notes);
        let first_start = world
            .entries()
            .into_iter()
            .find(|e| e.starts_with("start "));
        assert_eq!(first_start.as_deref(), Some("start a"));
        for item in ["b", "c", "d"] {
            assert!(
                world.at(&format!("read {item}")) < world.at("end a"),
                "{item}"
            );
        }
        for item in ["a", "b", "c", "d"] {
            assert_eq!(world.count(&format!("read {item}")), 1, "{item}");
        }
        assert!(world.widest.load(Ordering::SeqCst) <= 4);
    }

    #[test]
    fn read_ahead_with_a_limit_of_one_reads_one_item_ahead() {
        // Window 1. a is held until b is read, and then 200 ms more, long enough for a reader that ignored
        // the limit to read c as well. c is read only once b's reading is claimed (b starts), while b runs.
        let world = long_chain(1).gate("a", &["read b"]).gate("b", &["read c"]);
        let report = concurrent(&world, 1);
        assert_eq!(report.landed, 4, "{:?} {:?}", report.jobs, report.notes);
        let before_end_of_a: Vec<String> = world.entries()[..world.at("end a")]
            .iter()
            .filter(|e| e.starts_with("read "))
            .cloned()
            .collect();
        assert_eq!(before_end_of_a, ["read a", "read b"]);
        assert_eq!(world.count("read c"), 1);
        assert!(world.at("end a") < world.at("read c"));
        assert!(world.at("read c") < world.at("end b"));
        for item in ["a", "b", "c", "d"] {
            assert_eq!(world.count(&format!("read {item}")), 1, "{item}");
        }
        // The running job plus one reading ahead.
        assert!(world.widest.load(Ordering::SeqCst) <= 2);
    }

    #[test]
    fn read_ahead_of_a_refused_reading_is_reported_as_when_it_starts() {
        // a, then b and c, which depend only on a. b's reading is refused every time. The same queue is run
        // with a window of 1 (b is read while a runs, and the reading is dropped) and with a window of 0
        // (b is read only at its turn).
        let build = |window: u32| {
            World::new(
                window,
                &[
                    ("a", &["src/a.rs"], &[]),
                    ("b", &["src/b.rs"], &["a"]),
                    ("c", &["src/c.rs"], &["a"]),
                ],
            )
            .refusing("b")
        };
        let ahead = build(1).gate("a", &["read b"]);
        let report = concurrent(&ahead, 1);
        // b was read while a ran, and a still ran and landed.
        assert!(ahead.at("read b") < ahead.at("end a"));
        assert!(ahead.at("end a") < ahead.at("land a"));
        assert!(report.landed >= 1, "{:?} {:?}", report.jobs, report.notes);
        assert!(
            report
                .notes
                .iter()
                .any(|n| n.contains("the reading made ahead of b was dropped")),
            "{:?}",
            report.notes
        );
        // The refused reading is read again at b's turn, exactly as many times as with no read-ahead (observed:
        // twice, the drain tries a refused item once more), so the look-ahead reading is the only extra one.
        let turn = build(0);
        let plain = concurrent(&turn, 1);
        assert_eq!(turn.count("read b"), 2, "{:?}", turn.entries());
        assert_eq!(ahead.count("read b"), 3, "{:?}", ahead.entries());
        assert!(
            plain.notes.iter().all(|n| !n.contains("made ahead")),
            "{:?}",
            plain.notes
        );
        let line = |report: &Report, item: &str| {
            report
                .jobs
                .iter()
                .find(|j| j.item == item)
                .map(|j| (j.outcome.clone(), j.reason.clone(), j.landing.clone()))
        };
        let b = line(&report, "b");
        assert!(b.is_some(), "{:?}", report.jobs);
        assert_eq!(b, line(&plain, "b"), "{:?} {:?}", report.jobs, plain.jobs);
        assert_eq!(report.stop, plain.stop);
        assert_eq!(report.landed, plain.landed);
        let kept = |report: &Report| -> Vec<String> {
            report
                .notes
                .iter()
                .filter(|n| !n.contains("made ahead"))
                .cloned()
                .collect()
        };
        assert_eq!(kept(&report), kept(&plain));
    }

    #[test]
    fn read_ahead_is_off_with_a_window_of_zero() {
        // A window of 0 reads nothing ahead: with one job at a time through both entry points, and with two.
        // `drain` takes steps that are not `Sync`, so it never reads ahead, whatever the window says.
        let runs: [(&str, u32, u32); 4] = [
            ("drain_concurrent 1, window 0", 0, 1),
            ("drain, window 0", 0, 1),
            ("drain_concurrent 2, window 0", 0, 2),
            ("drain, window 1", 1, 1),
        ];
        for (how, window, bound) in runs {
            let world = World::chain(window);
            let report = if how.starts_with("drain,") {
                drain(&world, limits(20.0), &Quiet).unwrap()
            } else {
                concurrent(&world, bound)
            };
            assert_eq!(report.landed, 3, "{how}");
            // Each item is read when its turn comes, after the job before it ended and was landed.
            assert!(world.at("land a") < world.at("read b"), "{how}");
            assert!(world.at("land b") < world.at("read c"), "{how}");
            assert_eq!(world.count("plan_ahead b"), 0, "{how}");
            assert!(
                report.notes.iter().all(|n| !n.contains("stale")),
                "{how}: {:?}",
                report.notes
            );
        }
    }

    #[test]
    fn read_ahead_must_fit_the_budget() {
        // 1.5 is exactly what a's reading and run cap (0.5 + 1.0) need. While a runs, another reading and run
        // do not fit beside the run's held cap, and after a they do not fit what is left either, so b is
        // never read and the drain stops for lack of budget.
        let world = World::chain(1).gate("a", &[]);
        let report = drain_concurrent(&world, limits(1.5), &Quiet, 2).unwrap();
        assert_eq!(report.stop, Stop::BudgetSpent, "{:?}", report.notes);
        assert_eq!(report.landed, 1);
        assert_eq!(world.count("read b"), 0, "{:?}", world.entries());
    }

    /// a, then b which depends on it. The plan says a touches only src/a.rs, and b only src/b.rs.
    fn pair() -> World {
        World::new(
            1,
            &[
                ("a", &["src/a.rs", "CHANGELOG.md"], &[]),
                ("b", &["src/b.rs", "CHANGELOG.md"], &["a"]),
            ],
        )
        .gate("a", &["read b"])
    }

    fn stale_notes(report: &Report) -> Vec<&String> {
        report
            .notes
            .iter()
            .filter(|n| n.contains("stale"))
            .collect()
    }

    #[test]
    fn stale_reading_is_rechecked_when_its_job_starts() {
        // The plan underestimated a: it changed src/b.rs, which b's reading touches.
        let mut world = pair().lands("a", &["src/b.rs", "CHANGELOG.md"]);
        world.recheck_holds = false;
        let report = concurrent(&world, 2);
        assert_eq!(report.landed, 2, "{:?} {:?}", report.jobs, report.notes);
        assert!(
            world.at("read b") < world.at("land a"),
            "read ahead of a's landing"
        );
        assert!(world.at("land a") < world.at("recheck b [src/b.rs]"));
        assert!(world.at("recheck b [src/b.rs]") < world.at("reread b"));
        assert!(world.at("reread b") < world.at("start b-fresh"));
        assert_eq!(world.count("recheck b [src/b.rs]"), 1);
        assert_eq!(world.count("read b"), 1, "the drain never reads b again");
        assert!(!world.logged("start b"), "the stale reading is not run");
        let notes = stale_notes(&report);
        assert_eq!(notes.len(), 1, "{:?}", report.notes);
        assert!(notes[0].contains("reading of b") && notes[0].contains("src/b.rs"));
        // The job line is the fresh job's, and carries both readings and the run.
        assert_eq!(report.jobs[1].item, "b");
        assert_eq!(report.jobs[1].job, "b-fresh");
        assert!((report.jobs[1].cost_usd - (0.1 + 0.1 + 0.3)).abs() < 1e-9);
    }

    #[test]
    fn stale_reading_kept_when_recheck_says_it_holds() {
        let world = pair().lands("a", &["src/b.rs", "CHANGELOG.md"]);
        let report = concurrent(&world, 2);
        assert_eq!(report.landed, 2);
        assert_eq!(world.count("recheck b [src/b.rs]"), 1);
        assert_eq!(world.count("reread b"), 0);
        assert!(world.logged("start b"));
        assert_eq!(world.count("read b"), 1);
        assert_eq!(stale_notes(&report).len(), 1, "{:?}", report.notes);
    }

    #[test]
    fn stale_reading_not_marked_when_paths_are_untouched() {
        // a changed only what the plan said it would.
        let world = pair().lands("a", &["src/a.rs", "CHANGELOG.md"]);
        let report = concurrent(&world, 2);
        assert_eq!(report.landed, 2);
        assert!(world.at("read b") < world.at("land a"));
        assert!(!world.entries().iter().any(|e| e.starts_with("recheck")));
        assert_eq!(world.count("reread b"), 0);
        assert!(stale_notes(&report).is_empty(), "{:?}", report.notes);
        assert_eq!(world.count("read b"), 1);
        assert!(world.logged("start b"));
    }

    #[test]
    fn stale_reading_not_marked_when_only_shared_files_landed() {
        // Both the landing and the reading touch the changelog; every job does, so it is not a clash.
        let world = pair().lands("a", &["CHANGELOG.md"]);
        let report = concurrent(&world, 2);
        assert_eq!(report.landed, 2);
        assert!(!world.entries().iter().any(|e| e.starts_with("recheck")));
        assert_eq!(world.count("reread b"), 0);
        assert!(stale_notes(&report).is_empty(), "{:?}", report.notes);
        assert!(world.logged("start b"));
    }

    #[test]
    fn stale_reading_runs_the_job_the_reading_named() {
        let world = pair().names("b", "b-2");
        let report = concurrent(&world, 2);
        assert_eq!(report.landed, 2);
        assert!(world.logged("start b-2"));
        assert!(!world.logged("start b"));
        assert_eq!(report.jobs[1].item, "b");
        assert_eq!(report.jobs[1].job, "b-2");
    }

    #[test]
    fn a_reading_with_unknown_paths_is_stale_for_any_landing_but_the_shared_files() {
        let p = |paths: &[&str]| own(paths);
        assert_eq!(
            stale_paths(&p(&[]), &p(&["src/a.rs", "CHANGELOG.md", "src/a.rs"])),
            ["src/a.rs"]
        );
        assert!(stale_paths(&p(&["CHANGELOG.md"]), &p(&["README.md"])).is_empty());
        assert_eq!(
            stale_paths(&p(&["src/dir/"]), &p(&["src/dir/x.rs", "src/other.rs"])),
            ["src/dir/x.rs"]
        );
    }

    fn needs(candidate: &[&str], touched: &[&str]) -> bool {
        rebase_needs_recheck(&own(candidate), &own(touched))
    }

    #[test]
    fn rebase_needs_recheck_when_a_touched_path_is_a_candidate_path() {
        assert!(needs(&["src/a.rs"], &["src/a.rs"]));
        assert!(needs(&["src/a.rs", "src/b.rs"], &["docs/x.md", "src/b.rs"]));
    }

    #[test]
    fn rebase_needs_recheck_when_a_directory_holds_the_other_path() {
        assert!(needs(&["src/dir/"], &["src/dir/x.rs"]));
        assert!(needs(&["src/dir/x.rs"], &["src/dir"]));
    }

    #[test]
    fn rebase_needs_recheck_when_a_workspace_manifest_changed() {
        assert!(needs(&["src/a.rs"], &["Cargo.toml"]));
        assert!(needs(&["src/a.rs"], &["Cargo.lock"]));
        assert!(needs(&["src/a.rs"], &["crates/x/Cargo.toml"]));
    }

    #[test]
    fn rebase_needs_no_recheck_when_no_candidate_path_is_touched() {
        assert!(!needs(&["src/a.rs"], &["docs/other.md"]));
        assert!(!needs(&["src/a.rs"], &["src/ab.rs", "src/a.rs.bak"]));
        assert!(!needs(&["src/a.rs"], &[]));
        // A file merely named like a manifest is not one.
        assert!(!needs(&["src/a.rs"], &["docs/Cargo.toml.md"]));
    }

    #[test]
    fn rebase_needs_no_recheck_when_only_shared_append_only_files_are_touched() {
        assert!(!needs(&["src/a.rs"], &["CHANGELOG.md", "README.md"]));
        assert!(!needs(&["CHANGELOG.md"], &["CHANGELOG.md"]));
    }

    #[test]
    fn rebase_needs_recheck_when_the_candidate_paths_are_unknown_and_a_real_path_is_touched() {
        assert!(needs(&[], &["src/a.rs"]));
        assert!(!needs(&[], &["CHANGELOG.md"]));
    }
}

/// The landing queue: the pure [`LandingQueue`], the [`LandingSlot`] guard, and two workers driven through
/// [`drain_concurrent`]. The drain tests share one ordered log between the fake [`Steps`] and the recording
/// [`Ui`], and assert the whole log. A worker that is held back says so on the [`Ui`] ("waits in the landing
/// queue"), and the fake's land calls wait for that line, so the order does not depend on how the `landing`
/// lock is won. A wait is capped at 5 s and then logs "gate timed out", so a missing behavior fails the
/// assertion on the log instead of hanging the test.
#[cfg(test)]
mod landing_queue_tests {
    use super::*;
    use athena_data::roles::Ladder;
    use std::time::Instant;

    const WAITS: &str = "waits in the landing queue";

    #[test]
    fn joins_are_served_in_order() {
        let mut queue = LandingQueue::default();
        let (a, b, c) = (queue.join(), queue.join(), queue.join());
        assert!(queue.is_head(a) && !queue.is_head(b) && !queue.is_head(c));
        queue.leave(a);
        assert!(!queue.is_head(a) && queue.is_head(b) && !queue.is_head(c));
        queue.leave(b);
        assert!(queue.is_head(c));
        queue.leave(c);
        assert!(!queue.is_head(c), "an empty line has no head");
    }

    #[test]
    fn a_member_that_leaves_while_not_at_the_head_is_removed() {
        let mut queue = LandingQueue::default();
        let (a, b, c) = (queue.join(), queue.join(), queue.join());
        queue.leave(b);
        assert!(queue.is_head(a), "the head is not disturbed");
        queue.leave(a);
        assert!(queue.is_head(c), "the member that left is not served");
        assert!(!queue.is_head(b));
    }

    #[test]
    fn leaving_twice_is_harmless() {
        let mut queue = LandingQueue::default();
        let (a, b) = (queue.join(), queue.join());
        queue.leave(b);
        queue.leave(b);
        assert!(queue.is_head(a));
        queue.leave(a);
        queue.leave(a);
        let c = queue.join();
        assert!(queue.is_head(c), "tickets are never reused");
    }

    /// Entries in the order they happened, shared by the fake steps and the recording ui.
    #[derive(Default)]
    struct Log(Mutex<Vec<String>>);

    impl Log {
        fn push(&self, entry: impl Into<String>) {
            self.0.lock().unwrap().push(entry.into());
        }

        fn entries(&self) -> Vec<String> {
            self.0.lock().unwrap().clone()
        }

        /// Waits until `entry` is in the log; after 5 s logs "gate timed out" and goes on.
        fn wait_for(&self, entry: &str) {
            let give_up = Instant::now() + Duration::from_secs(5);
            while !self.entries().iter().any(|e| e == entry) {
                if Instant::now() >= give_up {
                    self.push("gate timed out");
                    return;
                }
                std::thread::sleep(Duration::from_millis(2));
            }
        }
    }

    /// A ui that logs the drain's "waits in the landing queue" lines, as "<job> waits in the landing queue".
    struct Watch<'a>(&'a Log);

    impl Ui for Watch<'_> {
        fn line(&self, text: &str) {
            if text.contains(WAITS) {
                self.0.push(text.trim_start_matches("drain").trim());
            }
        }
    }

    /// One scripted land call: the log entry to wait for before it returns (if any), and whether it lands.
    type LandCall = (Option<&'static str>, bool);

    /// Steps for two items, a and b, with disjoint touched paths, so one group and two workers. Each `run`
    /// passes, after waiting for its gate. Each `land` logs "land <job>" and follows its script.
    struct Lane {
        log: Log,
        run_gate: BTreeMap<&'static str, &'static str>,
        lands: BTreeMap<&'static str, Vec<LandCall>>,
        calls: Mutex<BTreeMap<String, usize>>,
    }

    impl Lane {
        fn new() -> Lane {
            Lane {
                log: Log::default(),
                run_gate: BTreeMap::new(),
                lands: BTreeMap::new(),
                calls: Mutex::new(BTreeMap::new()),
            }
        }

        fn run_waits_for(mut self, job: &'static str, entry: &'static str) -> Lane {
            self.run_gate.insert(job, entry);
            self
        }

        fn land_calls(mut self, job: &'static str, calls: &[LandCall]) -> Lane {
            self.lands.insert(job, calls.to_vec());
            self
        }
    }

    impl Steps for Lane {
        fn caps(&self) -> anyhow::Result<Caps> {
            Ok(Caps {
                read: 0.5,
                run: 1.0,
                rate: 0.5,
            })
        }
        fn queue(&self) -> anyhow::Result<QueueView> {
            let entry = |id: &str| PlanEntry {
                goal: format!("goal of {id}"),
                rung: Some(Ladder::Implementation),
                acceptance: vec![format!("{id} works")],
                touches: vec![format!("src/{id}.rs")],
                ..PlanEntry::bare(id, &format!("title of {id}"))
            };
            Ok(QueueView {
                items: vec![Item::queued("a", "t"), Item::queued("b", "t")],
                plan: vec![entry("a"), entry("b")],
                ..QueueView::default()
            })
        }
        fn previous_rating(&self) -> Option<u8> {
            None
        }
        fn approve(&self, _: &str) -> Approval {
            Approval::default()
        }
        fn read(&self, what: ReadOf<'_>) -> Read {
            let ReadOf::Queued(item) = what else {
                panic!("only a first reading is expected, got {what:?}");
            };
            Read {
                job: Some(item.to_string()),
                cost: 0.1,
                runnable: true,
                error: None,
                held: false,
            }
        }
        fn run(&self, job: &str) -> Run {
            if let Some(entry) = self.run_gate.get(job) {
                self.log.wait_for(entry);
            }
            Run {
                outcome: Some(OutcomeState::Landed),
                cost: 0.3,
                ..Run::default()
            }
        }
        fn land(&self, job: &str) -> Landing {
            self.log.push(format!("land {job}"));
            let call = {
                let mut calls = self.calls.lock().unwrap();
                let n = calls.entry(job.to_string()).or_default();
                *n += 1;
                *n - 1
            };
            let (gate, lands) = self.lands[job][call];
            if let Some(entry) = gate {
                self.log.wait_for(entry);
            }
            if lands {
                Landing::Landed {
                    changed: vec![format!("src/{job}.rs"), "CHANGELOG.md".into()],
                }
            } else {
                Landing::NotLanded("scripted".into())
            }
        }
        fn rate(&self) -> Rate {
            Rate {
                rating: Some(1),
                ..Rate::default()
            }
        }
        fn check_page(&self) -> Check {
            Check::pass("fake")
        }
        fn check_cap(&self) -> f64 {
            0.5
        }
    }

    fn limits() -> Limits {
        Limits {
            budget_usd: 20.0,
            target_rating: 4,
            max_jobs: 20,
        }
    }

    /// Drives the lane with two workers and returns how many items landed.
    fn two_workers(lane: &Lane) -> usize {
        let ui = Watch(&lane.log);
        let report = drain_concurrent(lane, limits(), &ui, 2).unwrap();
        report.landed as usize
    }

    #[test]
    fn the_head_keeps_its_place_while_its_slot_is_held() {
        let lane = Lane::new();
        let ui = Watch(&lane.log);
        let d = Drain::new(&lane, &ui, lane.caps().unwrap(), limits());
        let first = d.enter_queue();
        let second = d.enter_queue();
        // The same slot serves both land calls of a retry; the member behind it never becomes head.
        for _call in 0..2 {
            assert!(d.st().queue.is_head(first.ticket));
            assert!(!d.st().queue.is_head(second.ticket));
        }
        drop(first);
        assert!(d.st().queue.is_head(second.ticket));
    }

    #[test]
    fn dropping_a_middle_slot_keeps_the_others_in_order() {
        let lane = Lane::new();
        let ui = Watch(&lane.log);
        let d = Drain::new(&lane, &ui, lane.caps().unwrap(), limits());
        let first = d.enter_queue();
        let middle = d.enter_queue();
        let last = d.enter_queue();
        let middle_ticket = middle.ticket;
        drop(middle);
        assert!(d.st().queue.is_head(first.ticket));
        drop(first);
        assert!(d.st().queue.is_head(last.ticket));
        assert!(!d.st().queue.is_head(middle_ticket));
    }

    #[test]
    fn a_slot_dropped_by_a_panic_leaves() {
        let lane = Lane::new();
        let ui = Watch(&lane.log);
        let d = Drain::new(&lane, &ui, lane.caps().unwrap(), limits());
        let waiting = d.enter_queue();
        let gone = d.enter_queue();
        let gone_ticket = gone.ticket;
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _held = gone;
            panic!("a worker panics while it holds a place");
        }));
        assert!(outcome.is_err());
        assert!(!d.st().queue.is_head(gone_ticket));
        drop(waiting);
        let next = d.enter_queue();
        assert!(d.st().queue.is_head(next.ticket), "nothing is left in line");
    }

    #[test]
    fn a_waiting_worker_does_not_land_between_the_heads_two_land_calls() {
        // a passes first and its first land call is refused. b passes while a is in that call, so b is behind
        // a: it says so, and a's first land call returns only after that. b lands after a's second call.
        let lane = Lane::new()
            .run_waits_for("b", "land a")
            .land_calls(
                "a",
                &[(Some("b waits in the landing queue"), false), (None, true)],
            )
            .land_calls("b", &[(None, true)]);
        let landed = two_workers(&lane);
        assert_eq!(
            lane.log.entries(),
            ["land a", "b waits in the landing queue", "land a", "land b"]
        );
        assert_eq!(landed, 2);
    }

    #[test]
    fn two_workers_land_in_the_order_they_passed() {
        // a is the first item and starts first, but its run ends only once b is in its first land call, so b
        // passed first. a passes behind b, says it waits, and lands after b.
        let lane = Lane::new()
            .run_waits_for("a", "land b")
            .land_calls("a", &[(None, true)])
            .land_calls("b", &[(Some("a waits in the landing queue"), true)]);
        let landed = two_workers(&lane);
        assert_eq!(
            lane.log.entries(),
            ["land b", "a waits in the landing queue", "land a"]
        );
        assert_eq!(landed, 2);
    }

    #[test]
    fn a_failed_head_releases_its_place() {
        // Both of a's land calls fail. b waits behind a, then lands once a has left.
        let lane = Lane::new()
            .run_waits_for("b", "land a")
            .land_calls(
                "a",
                &[(Some("b waits in the landing queue"), false), (None, false)],
            )
            .land_calls("b", &[(None, true)]);
        let landed = two_workers(&lane);
        assert_eq!(
            lane.log.entries(),
            ["land a", "b waits in the landing queue", "land a", "land b"]
        );
        assert_eq!(landed, 1);
    }
}
