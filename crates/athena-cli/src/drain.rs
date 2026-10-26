//! `athena drain`: works the queue itself (requests the steward queued and requests deferred with `athena ask --later`), within a budget the user sets. For each queued request
//! it reads and runs the job, lands it through the pull-request path (`athena land --pr`), and only then
//! rates the live page as its audience would. It stops at the first of six reasons ([`Stop`]).
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
//! - **Concurrency.** A group is worked by up to `max_concurrency` workers; the next group starts only when
//!   the group before it is done, so an item that depends on another starts after it has finished. Landing
//!   and rating go one at a time (a lock), and every model call's cap is held against the budget while it is
//!   in flight, so jobs side by side cannot overspend it. The first reason to stop ends the run: no worker
//!   starts another job, jobs already started are finished (a failure or a refusal stops no sibling that is
//!   in flight), and the first reason found is the one reported. A worker that panics is not isolated: the
//!   drain ends with the panic once the other workers have finished.
//! - **The pass reads unread requests that are underspecified.** An unread request whose own words give no
//!   acceptance (or no rung) is read first, within the budget and job limit, and planned again from its
//!   reading. If the plan is still incomplete it is refused with the reason, so the queue
//!   does not fill with unread items that are refused merely for being unread; one that is specified after
//!   its reading runs without a second reading. [`plan`] runs the same pass for `athena plan` and works
//!   nothing, so its `--budget` only has to fit each read's cap, not a run after it.
//! - **A read request stays queued.** A request is open until it is run or closed. One that `athena plan` (or
//!   a person) has read waits in state proposed and is still in the queue, marked read: [`drain`] runs it
//!   without reading it again, in plan order, and a second `athena plan` lists it again. Only a request
//!   that was declined, is running, or has ended, and one on another channel, is left out
//!   ([`athena_core::steward::queued_work`]).
//! - **An interrupted job is retried.** A job that was dispatched, has no closing record and has no process
//!   working on it (a machine restart killed it) is listed as work like a queued request, on a drained
//!   channel only ([`athena_core::steward::interrupted_work`]). Its first attempt is `athena retry`, which
//!   reads the request again as a new job; that job is then run. The interrupted job is left as it is, but a
//!   job that a retry already names is not listed again. A job whose process is still running is never listed.
//! - **Stop at a job boundary.** [`Steps::stop_requested`] is asked before every job. A job that has started
//!   is finished (landed and rated) and no further job starts: [`Stop::Requested`]. Nothing is killed.
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
//! - **Low disk holds a job.** When free disk space is below `[disk] min_free_gb` (default 10) at dispatch,
//!   even after idle jobs' build directories are removed, `athena go` returns [`athena_core::disk::LowDisk`].
//!   The item is left queued (outcome `environment: low disk`, with the free and required amounts, plus a
//!   note); that is not a failure, is not retried, and stops nothing. A later drain runs the same job.
//! - **In-limit proposals are approved, approved jobs are worked.** An open proposal whose rung the role that
//!   raised it may change itself (its newest recorded definition decides; a role with no recorded definition
//!   has no limit) is approved as `athena approve-work` would, which reads the job and records the decision.
//!   Jobs that a work decision records as approved and that are still in state proposed are run, once, without
//!   a second reading; a closed job is never picked up again. An approval is a model call, so it is guarded by
//!   the budget and the job limit like any other.
//! - **People decide the rest.** Rejecting a proposal, and approving one above its role's limit (or from an
//!   unknown role), stay with people. Such proposals are listed as waiting, and [`Steps`] has no way to reject
//!   one or to approve one that is not in limit.

use athena_core::audience::{LivePage, Page, Target};
use athena_core::gh::GhCli;
use athena_core::plan::{Plan, PlanEntry, Refusal};
use athena_core::pr::LinkState;
use athena_core::store::Records;
use athena_core::{Athena, Ui, steward, work_decisions};
use athena_data::records::{AcceptedBy, OutcomeState};
use serde::Serialize;
use std::collections::VecDeque;
use std::sync::{Mutex, MutexGuard, PoisonError};

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
    /// True when the job was already read (an approved job waiting in state proposed), so it is run without a
    /// reading. A retry after a failed run still reads again.
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
    /// True when dispatch was held for lack of free disk space (`environment: low disk`). The job is still
    /// proposed and was not run: it is neither retried nor counted as a failure, and a later drain runs the
    /// same job once space is free.
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

/// The result of landing a verified job through the pull-request path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Landing {
    /// Merged and verified: the change is on the default branch. `changed` is every path the change touched.
    Landed { changed: Vec<String> },
    /// Anything short of that, with why. An open pull request is not a landing.
    NotLanded(String),
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
    fn land(&self, job: &str) -> Landing;
    fn rate(&self) -> Rate;
    /// Checks every text input of the page against the pinned source, with a model call per study. It is asked
    /// only when a rating reaches the target after a landing that changed the outcome, and the target is met
    /// only if it passes. It reports what it cost in [`Check::cost`].
    fn check_page(&self) -> Check;
    /// The most [`Steps::check_page`] may cost: the cap of one call times the number of studies it checks. The
    /// loop asks the budget for this much before it calls the check.
    fn check_cap(&self) -> f64;
    /// The plan entry of `job` as the records stand now. The planning pass asks for it after it has read an
    /// unread request, because a reading gives the entry its goal, acceptance and touched paths. None (the
    /// default) keeps the entry the queue view carried.
    fn plan_entry(&self, _job: &str) -> Option<PlanEntry> {
        None
    }
    /// Whether the user asked the loop to stop. It is asked only between jobs, never during one, so a job
    /// that has started is finished (and landed and rated) first. Not asking to stop is the default.
    fn stop_requested(&self) -> bool {
        false
    }
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
    /// One item failed twice.
    RepeatedFailure,
    /// Nothing is queued (or everything queued was worked). This is not success.
    QueueEmpty,
    /// The job limit was reached.
    JobLimit,
    /// The user asked the loop to stop (`athena drain-stop`). The job in progress was finished first and no
    /// further job was started; nothing was killed.
    Requested,
}

impl Stop {
    pub const ALL: [Stop; 6] = [
        Stop::TargetMet,
        Stop::BudgetSpent,
        Stop::RepeatedFailure,
        Stop::QueueEmpty,
        Stop::JobLimit,
        Stop::Requested,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Stop::TargetMet => "target met",
            Stop::BudgetSpent => "budget spent",
            Stop::RepeatedFailure => "repeated failure",
            Stop::QueueEmpty => "queue empty",
            Stop::JobLimit => "job limit",
            Stop::Requested => "stop requested",
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
            Stop::RepeatedFailure => "the same item failed twice; the item is named",
            Stop::QueueEmpty => "nothing was left to work; this is not success",
            Stop::JobLimit => "the job limit was reached",
            Stop::Requested => {
                "a stop was asked for; the job in progress was finished and no further job was started, so the rest stays queued"
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
    /// `not run` or `no outcome`.
    pub outcome: String,
    pub reason: Option<String>,
    pub cost_usd: f64,
    /// `landed`, `not landed: <why>` or `not attempted`.
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
    /// The item that failed twice, when `stop` is `repeated-failure`.
    pub failed_item: Option<String>,
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
    /// Items refused for design or code because their plan lacks a goal, a ladder rung or acceptance.
    pub refused: Vec<Refusal>,
    /// Anything that went wrong beside the stop reason, such as a rating that could not be taken.
    pub notes: Vec<String>,
}

impl Report {
    /// The target was met by a rating taken after a landing. The only way to be true.
    pub fn target_met(&self) -> bool {
        self.stop == Stop::TargetMet && self.rating.is_some_and(|r| r.after_landing && r.met_target)
    }
}

/// Everything the workers of one drain share, behind one lock. A lock is never held across a call into
/// [`Steps`], so a slow model call never blocks another worker's bookkeeping.
struct State {
    budget: Budget,
    jobs: Vec<JobLine>,
    started: u32,
    landed: Vec<String>,
    /// Landings in this run that changed a path in [`OUTCOME_PATHS`].
    outcome_landings: u32,
    rating: Option<RatingLine>,
    held_back: Option<String>,
    notes: Vec<String>,
    /// What the planning pass spent reading an item, charged to that item's job line when it runs.
    read_cost: std::collections::BTreeMap<String, f64>,
    /// Why the run is ending, once any worker has found a reason (the first reason found stands), and the
    /// item that failed twice when that is the reason. No worker starts a job once this is set; jobs already
    /// started are finished.
    stop: Option<(Stop, Option<String>)>,
}

struct Drain<'a, S: ?Sized, U: ?Sized> {
    steps: &'a S,
    ui: &'a U,
    caps: Caps,
    limits: Limits,
    state: Mutex<State>,
    /// Held while a job is landed and while the page is rated: landings and ratings go one at a time even
    /// when jobs run side by side, because each pushes a branch and rates the page as it then stands.
    landing: Mutex<()>,
}

enum ItemEnd {
    Landed,
    Failed,
    Budget,
    JobLimit,
    /// A stop was asked for and no further job was started.
    Stop,
    /// Another worker had already ended the run, so nothing was started.
    Idle,
    /// The item's reading was made again with a different scope; it stays queued, and nothing else stops.
    NeedsAccepting,
    /// Dispatch was held for lack of free disk space; the item stays queued, and nothing else stops.
    Held,
}

enum RunStep {
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
    /// The job was held at dispatch for lack of free disk space. It is not a failure and not tried again in
    /// this run.
    Held,
    Budget,
    JobLimit,
    /// Another worker had already ended the run.
    Ended,
}

impl<S: Steps + ?Sized, U: Ui + ?Sized> Drain<'_, S, U> {
    fn st(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn push(&self, line: JobLine) -> usize {
        self.ui.line(&format!(
            "drain      {} → {} ({})",
            line.item, line.job, line.outcome
        ));
        let mut st = self.st();
        st.jobs.push(line);
        st.jobs.len() - 1
    }

    fn note(&self, text: String) {
        self.st().notes.push(text);
    }

    /// Ends the run for `stop`, unless another worker already found a reason.
    fn set_stop(&self, stop: Stop, failed: Option<String>) {
        let mut st = self.st();
        if st.stop.is_none() {
            st.stop = Some((stop, failed));
        }
    }

    fn stopped(&self) -> bool {
        self.st().stop.is_some()
    }

    /// Reads and runs one job. Starts only when the reading and the run both fit the budget, counting the
    /// caps that jobs already in flight hold.
    fn run_job(&self, item: &Item, from: ReadOf<'_>) -> RunStep {
        // An approved job was read when it was approved; its first attempt is only a run.
        let pre_read = item.read && matches!(from, ReadOf::Queued(_));
        let (ReadOf::Queued(target) | ReadOf::Retry(target)) = from;
        let needed = if pre_read {
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
            st.budget.hold(needed);
        }
        let read = if pre_read {
            Read {
                job: Some(target.to_string()),
                cost: 0.0,
                runnable: true,
                error: None,
            }
        } else {
            self.steps.read(from)
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
                self.push(line);
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
                line.outcome = athena_core::disk::LOW_DISK.into();
                self.note(format!(
                    "item {} is left queued: {}",
                    item.job,
                    line.reason
                        .as_deref()
                        .unwrap_or("free disk space is below the minimum")
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

    fn land(&self, job: &str, at: usize) -> bool {
        let _one_at_a_time = self.landing.lock().unwrap_or_else(PoisonError::into_inner);
        match self.steps.land(job) {
            Landing::Landed { changed } => {
                let outcome = changes_outcome(&changed);
                {
                    let mut st = self.st();
                    st.jobs[at].landing = "landed".into();
                    if outcome {
                        st.outcome_landings += 1;
                    }
                }
                if !outcome {
                    self.ui.line(&format!(
                        "drain      {job} landed but changed nothing the explainer is built from"
                    ));
                }
                true
            }
            Landing::NotLanded(why) => {
                self.ui.line(&format!("drain      {job} not landed: {why}"));
                self.st().jobs[at].landing = format!("not landed: {why}");
                false
            }
        }
    }

    /// Works one item through its attempts: read and run, land, and on a failure try once more.
    fn work(&self, item: &Item) -> ItemEnd {
        // Once the job is read, an attempt that is not a retry only runs it.
        let mut item = item.clone();
        let item = &mut item;
        let mut last_job = item.job.clone();
        // An interrupted job has no finished attempt to run: its first attempt reads it again.
        let mut retry = item.interrupted;
        let mut passed: Option<(String, usize)> = None;
        let mut failures = 0;
        loop {
            if passed.is_none() {
                // A job boundary: the previous attempt, if any, is over and the next has not started.
                if self.steps.stop_requested() {
                    return ItemEnd::Stop;
                }
                let from = if retry {
                    ReadOf::Retry(&last_job)
                } else {
                    ReadOf::Queued(&last_job)
                };
                match self.run_job(item, from) {
                    RunStep::Budget => return ItemEnd::Budget,
                    RunStep::JobLimit => return ItemEnd::JobLimit,
                    RunStep::Ended => return ItemEnd::Idle,
                    RunStep::NeedsAccepting => return ItemEnd::NeedsAccepting,
                    RunStep::Held => return ItemEnd::Held,
                    RunStep::Passed(job, at) => passed = Some((job, at)),
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
                            last_job = job;
                            item.read = true;
                        }
                    }
                }
            }
            if let Some((job, at)) = passed.clone()
                && self.land(&job, at)
            {
                self.st().landed.push(item.job.clone());
                return ItemEnd::Landed;
            }
            failures += 1;
            if failures >= ATTEMPTS {
                return ItemEnd::Failed;
            }
        }
    }

    /// Approves the proposals the ladder lets their role change. Each approval reads a job, so it must leave
    /// room for that job's run and stay under the job limit. Returns the jobs it made, and the proposals it
    /// left open (with a note saying why).
    fn approve_all(&self, proposals: Vec<Waiting>) -> (Vec<Item>, Vec<Waiting>) {
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
    fn prepare(&self, view: &mut QueueView, reserve: f64) -> Plan {
        let mut forced: Vec<Refusal> = Vec::new();
        for idx in 0..view.items.len() {
            // An interrupted job was read before it ran; its retry reads the request again.
            if view.items[idx].read || view.items[idx].interrupted {
                continue;
            }
            let id = view.items[idx].job.clone();
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
                self.note(format!(
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
                forced.push(r);
                continue;
            }
            view.items[idx].read = read.runnable;
            if !read.runnable && view.plan[pos].missing().is_empty() {
                let mut r = Refusal::lacking(&id, &title, vec!["capability".into()]);
                r.reason =
                    "it was read, but no capability fits the request, so there is nothing to run"
                        .into();
                forced.push(r);
            } else if !view.plan[pos].missing().is_empty() {
                self.note(format!(
                    "item {id} was read, but its plan is still incomplete; correct or answer it with `athena correct {id}`"
                ));
            }
        }
        let entries: Vec<PlanEntry> = view
            .plan
            .iter()
            .filter(|e| !forced.iter().any(|f| f.id == e.id))
            .cloned()
            .collect();
        let mut plan = Plan::build(&entries);
        plan.refused.extend(forced);
        plan
    }

    /// Rates the page after a landing. Returns the stop reason when the rating ends the run.
    fn rate(&self) -> Option<Stop> {
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

impl<S: Steps + ?Sized, U: Ui + ?Sized> Drain<'_, S, U> {
    fn new<'a>(steps: &'a S, ui: &'a U, caps: Caps, limits: Limits) -> Drain<'a, S, U> {
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
                read_cost: Default::default(),
                stop: None,
            }),
            landing: Mutex::new(()),
        }
    }

    /// What a worker does when its item has ended: a landed item is followed by a rating, and a reason to
    /// stop ends the run for every worker. One item's failure stops no job already in flight.
    fn finish(&self, item: &Item, end: ItemEnd) {
        match end {
            ItemEnd::Landed => {
                if let Some(reason) = self.rate() {
                    self.set_stop(reason, None);
                }
            }
            ItemEnd::Failed => self.set_stop(Stop::RepeatedFailure, Some(item.job.clone())),
            ItemEnd::Budget => self.set_stop(Stop::BudgetSpent, None),
            ItemEnd::JobLimit => self.set_stop(Stop::JobLimit, None),
            ItemEnd::Stop => self.set_stop(Stop::Requested, None),
            ItemEnd::Idle | ItemEnd::NeedsAccepting | ItemEnd::Held => {}
        }
    }

    /// Takes items from `queue`, oldest first, until it is empty or the run has ended.
    fn take_until_done(&self, queue: &Mutex<VecDeque<Item>>) {
        loop {
            if self.stopped() {
                return;
            }
            let next = queue
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .pop_front();
            let Some(item) = next else { return };
            let end = self.work(&item);
            self.finish(&item, end);
        }
    }

    /// One job at a time, in order, on the calling thread.
    fn run_serial(&self, batches: Vec<Vec<Item>>) {
        for batch in batches {
            self.take_until_done(&Mutex::new(VecDeque::from(batch)));
        }
    }
}

impl<S: Steps + Sync + ?Sized, U: Ui + Sync + ?Sized> Drain<'_, S, U> {
    /// Runs each batch with up to `workers` jobs at once; the next batch starts when the batch before it is
    /// done. A batch holds items that no dependency links and whose touched paths are disjoint.
    fn run_parallel(&self, batches: Vec<Vec<Item>>, workers: usize) {
        for batch in batches {
            if self.stopped() {
                return;
            }
            let count = workers.min(batch.len()).max(1);
            let queue = Mutex::new(VecDeque::from(batch));
            std::thread::scope(|scope| {
                for _ in 0..count {
                    scope.spawn(|| self.take_until_done(&queue));
                }
            });
        }
    }
}

/// Works the queue within `limits`, one job at a time. See the module text for the rules.
pub fn drain(steps: &dyn Steps, limits: Limits, ui: &dyn Ui) -> anyhow::Result<Report> {
    let caps = steps.caps()?;
    let d = Drain::new(steps, ui, caps, limits);
    drain_with(&d, steps, |d, batches| d.run_serial(batches))
}

/// Works the queue like [`drain`], running up to `max_concurrency` jobs at once. The jobs of one group of the
/// plan (no dependency between them, disjoint touched paths) run side by side; the next group starts when
/// the group before it is done, so an item that depends on another starts only after it has finished. Landing
/// and rating stay one at a time. With `max_concurrency` 1 this is [`drain`]. A worker that panics ends the
/// drain with that panic once the other workers have finished their jobs; its item is not isolated.
pub fn drain_concurrent(
    steps: &(dyn Steps + Sync),
    limits: Limits,
    ui: &(dyn Ui + Sync),
    max_concurrency: u32,
) -> anyhow::Result<Report> {
    let caps = steps.caps()?;
    let d = Drain::new(steps, ui, caps, limits);
    let workers = max_concurrency.max(1) as usize;
    drain_with(&d, steps, move |d, batches| {
        if workers == 1 {
            d.run_serial(batches);
        } else {
            d.run_parallel(batches, workers);
        }
    })
}

fn drain_with<'a, S: Steps + ?Sized, U: Ui + ?Sized>(
    d: &Drain<'a, S, U>,
    steps: &S,
    run: impl FnOnce(&Drain<'a, S, U>, Vec<Vec<Item>>),
) -> anyhow::Result<Report> {
    let limits = d.limits;
    let view = steps.queue()?;
    let previous = steps.previous_rating();
    let mut view = view;
    let (approved, left) = d.approve_all(std::mem::take(&mut view.approvable));
    // Jobs already approved come first, then the ones approved now, then queued requests.
    let (read, unread): (Vec<Item>, Vec<Item>) = view.items.drain(..).partition(|i| i.read);
    view.items = read.into_iter().chain(approved).chain(unread).collect();
    view.waiting.extend(left);
    // Planning comes first: the open requests are planned together, and the items are worked in plan order,
    // not oldest first. Items with no plan entry keep their place ahead of the planned ones; items whose plan
    // is incomplete, or that wait on each other in a loop, are not worked.
    let reserve = d.caps.read + d.caps.run;
    let plan = d.prepare(&mut view, reserve);
    // Items with no plan entry keep their place ahead of the planned ones, each alone in its batch. The
    // planned items follow, one batch per plan group; the groups, flattened, are the plan order.
    let mut batches: Vec<Vec<Item>> = view
        .items
        .iter()
        .filter(|i| !view.plan.iter().any(|e| e.id == i.job))
        .cloned()
        .map(|i| vec![i])
        .collect();
    batches.extend(plan.groups.iter().map(|group| {
        group
            .iter()
            .filter_map(|id| view.items.iter().find(|i| &i.job == id).cloned())
            .collect()
    }));
    for refusal in &plan.refused {
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
    run(d, batches);
    let st = d.st();
    let (stop, failed_item) = st.stop.clone().unwrap_or((Stop::QueueEmpty, None));
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
        failed_item,
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
        refused: plan.refused,
        notes: st.notes.clone(),
        jobs: st.jobs.clone(),
    })
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

// ---------------------------------------------------------------------------------------------------------
// The real world

/// How a verified job is landed.
pub trait Lander {
    fn land(&self, athena: &Athena, job: &str, ui: &dyn Ui) -> Landing;
}

/// Lands through the pull-request path of `athena land --pr`: push the job's branch, open the pull request,
/// wait for green checks, merge only when `[landing] merge_via_rest` is on. Only a merged, verified pull
/// request is a landing.
#[derive(Debug)]
pub struct PrLander;

impl Lander for PrLander {
    fn land(&self, athena: &Athena, job: &str, ui: &dyn Ui) -> Landing {
        let report = athena.land_pr(
            job,
            &GhCli,
            crate::wiring::checks_poll(&std::thread::sleep),
            ui,
        );
        match report {
            // The paths it changed are read from the job's candidate record by `RealSteps::land`.
            Ok(report) if report.link.state == LinkState::Landed => Landing::Landed {
                changed: Vec::new(),
            },
            Ok(report) => Landing::NotLanded(format!(
                "pull request {} is {:?}, checks {:?}; it is landed once it is merged",
                report.link.pr_number, report.link.state, report.checks
            )),
            Err(error) => Landing::NotLanded(format!("{error:#}")),
        }
    }
}

/// Where a study model contradicts itself about its counters, as one sentence each. A retry edge that says a
/// counter changes (a text in `retry.changes` that names the counter) means the counter is taken by one
/// transition and freed by each event the text names; the events are its alternatives, split on " or ". The
/// counter's `changed_by` must then list at least one transition for taking and one for each such event. The
/// check reads only the model, so it needs no model call and no network. It is a floor, not a proof: the
/// pinned source is still the authority, and the study's own validation rules still apply.
pub fn model_contradictions(model: &serde_json::Value) -> Vec<String> {
    let list = |v: &serde_json::Value, key: &str| -> Vec<serde_json::Value> {
        v.get(key)
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default()
    };
    let mut out = Vec::new();
    for counter in list(model, "counters") {
        let Some(id) = counter.get("id").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let changed_by = list(&counter, "changed_by").len();
        for t in list(model, "transitions") {
            let tid = t
                .get("id")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("?");
            let Some(retry) = t.get("retry").filter(|r| !r.is_null()) else {
                continue;
            };
            for text in list(retry, "changes")
                .iter()
                .filter_map(serde_json::Value::as_str)
                .filter(|text| text.contains(id))
            {
                let needed = 1 + text.split(" or ").count();
                if changed_by < needed {
                    out.push(format!(
                        "counter {id} lists {changed_by} transition(s) in changed_by, but {tid}'s retry says \"{text}\", which needs {needed}: one that takes a place and one for each way it is freed"
                    ));
                }
            }
        }
    }
    out
}

/// One study directory's files: each file's path relative to the directory, with its bytes.
pub type StudyFiles = Vec<(String, Vec<u8>)>;

/// The files of each study directory under `studies/` at `commit`, with the study's name. Studies are in name
/// order.
pub fn study_files(
    git: &athena_git::Git,
    commit: &str,
) -> anyhow::Result<Vec<(String, StudyFiles)>> {
    let mut studies: std::collections::BTreeMap<String, StudyFiles> =
        std::collections::BTreeMap::new();
    for entry in git.ls_tree_under(commit, "studies")? {
        let Some((name, file)) = entry
            .path
            .strip_prefix("studies/")
            .and_then(|rest| rest.split_once('/'))
        else {
            continue;
        };
        if let Some(bytes) = git.show_file(commit, &entry.path)? {
            studies
                .entry(name.to_string())
                .or_default()
                .push((file.to_string(), bytes));
        }
    }
    Ok(studies.into_iter().collect())
}

/// A study's page text and pin, from its files. Every file but those in `athena_core::pin::STRUCTURAL_FILES`
/// is page text, so a text file added to a study is checked without anyone listing it. Errs, saying why, when
/// the study has no usable pin or a page file is not text.
pub fn study_text(
    name: &str,
    files: &[(String, Vec<u8>)],
) -> Result<athena_core::pin::StudyText, String> {
    let pin = files
        .iter()
        .find(|(f, _)| f == "pin.json")
        .ok_or("it has no pin.json, so there is no source to check against")?;
    let pin: serde_json::Value =
        serde_json::from_slice(&pin.1).map_err(|e| format!("pin.json is not valid JSON: {e}"))?;
    let field = |key: &str| {
        pin.get(key)
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
            .ok_or(format!("pin.json has no {key}"))
    };
    let sha = field("sha")?;
    if !athena_core::pin::is_full_sha(&sha) {
        return Err(format!("pin.json's sha {sha:?} is not a full commit id"));
    }
    let mut text = Vec::new();
    for (file, bytes) in files
        .iter()
        .filter(|(f, _)| athena_core::pin::is_page_text(f))
    {
        let body = String::from_utf8(bytes.clone())
            .map_err(|_| format!("{file} is not text, so it cannot be checked"))?;
        text.push(athena_core::pin::TextFile {
            name: file.clone(),
            text: body,
        });
    }
    Ok(athena_core::pin::StudyText {
        name: name.to_string(),
        repository_url: field("repository_url")?,
        sha,
        files: text,
    })
}

/// The steps against a real repository.
pub struct RealSteps<'a> {
    pub athena: &'a Athena,
    // `Sync` so that [`drain_concurrent`] can share the steps between workers.
    pub ui: &'a (dyn Ui + Sync),
    pub page: &'a dyn Page,
    pub lander: &'a (dyn Lander + Sync),
}

impl std::fmt::Debug for RealSteps<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealSteps")
            .field("athena", &self.athena)
            .finish_non_exhaustive()
    }
}

impl<'a> RealSteps<'a> {
    /// The live explainer page, landed through the pull-request path.
    pub fn live(athena: &'a Athena, ui: &'a (dyn Ui + Sync)) -> RealSteps<'a> {
        RealSteps {
            athena,
            ui,
            page: &LivePage,
            lander: &PrLander,
        }
    }

    fn records(&self) -> anyhow::Result<Records> {
        Records::load(&self.athena.git, &self.athena.head()?)
    }

    fn job_cost(&self, job: &str) -> f64 {
        self.records()
            .ok()
            .and_then(|r| r.jobs.get(job).map(|j| j.spent_usd()))
            .unwrap_or(0.0)
    }

    /// The plan entry of `job` at `head`: the rung of the proposal that made it when there is one, and lookups
    /// of the paths and referenced jobs it names.
    fn entry_of(&self, records: &Records, head: &str, job: &str) -> Option<PlanEntry> {
        let record = records.jobs.get(job)?;
        let hint = records.work_items.iter().find_map(|work| {
            let decision = work_decisions::decision_of(records, work)?;
            match &decision.decision {
                athena_data::records::WorkDecisionState::Approved { job: made }
                    if made.as_str() == job =>
                {
                    Some(work.rung)
                }
                _ => None,
            }
        });
        let job_exists = |id: &str| records.jobs.contains_key(id);
        // A lookup that fails counts the path as present, so a git error never invents an open question.
        let path_exists = |path: &str| self.athena.git.path_exists(head, path).unwrap_or(true);
        // The requirements and addenda a request can serve; no brief at the base commit means none.
        let brief = self
            .athena
            .git
            .show_file(head, "BRIEF.md")
            .ok()
            .flatten()
            .map(|b| athena_core::reading::brief_topics(&String::from_utf8_lossy(&b)))
            .unwrap_or_default();
        Some(athena_core::reading::plan_entry(
            record,
            hint,
            &athena_core::reading::Lookup {
                job_exists: &job_exists,
                path_exists: &path_exists,
                brief: &brief,
            },
        ))
    }

    fn audience_cost(&self) -> f64 {
        self.records()
            .map(|r| r.audience_runs.iter().map(|i| i.cost.best_lower()).sum())
            .unwrap_or(0.0)
    }
}

impl Steps for RealSteps<'_> {
    fn caps(&self) -> anyhow::Result<Caps> {
        let jc = athena_core::capability::JobConfig::at(&self.athena.git, &self.athena.head()?)?;
        Ok(Caps {
            read: jc.config.budget.reader_usd,
            run: jc.config.budget.job_usd,
            rate: jc.config.budget.verifier_usd,
        })
    }

    fn queue(&self) -> anyhow::Result<QueueView> {
        let records = self.records()?;
        let waiting = |items: Vec<&athena_data::records::WorkItem>| -> Vec<Waiting> {
            items
                .into_iter()
                .map(|w| Waiting {
                    id: w.id.to_string(),
                    title: w.title.clone(),
                    rung: w.rung.as_str().to_string(),
                })
                .collect()
        };
        let (within, above) = work_decisions::split_open(&records);
        let item = |w: steward::QueuedWork| Item {
            job: w.job.as_str().to_string(),
            title: w.title,
            read: w.read,
            interrupted: false,
        };
        let mut items: Vec<Item> = steward::approved_work(&records)
            .into_iter()
            .map(item)
            .collect();
        // A job a work decision approved can also sit on a drained channel: it is listed once.
        let queued: Vec<Item> = steward::queued_work(&records)
            .into_iter()
            .map(item)
            .filter(|i| !items.iter().any(|a| a.job == i.job))
            .collect();
        items.extend(queued);
        // A job a restart interrupted is worked by retrying it. The registry is cleaned first, so a job whose
        // process is gone counts as interrupted.
        self.athena.state.registry().reap();
        items.extend(
            steward::interrupted_work(&records, &|j| self.athena.running(j))
                .into_iter()
                .map(|w| Item {
                    interrupted: true,
                    ..item(w)
                }),
        );
        // Planning: every open request is fleshed out. Nothing is defaulted: a job that came from a steward
        // proposal serves the proposal's rung; any other request's rung is inferred or flagged.
        let head = self.athena.head()?;
        let plan = items
            .iter()
            .filter_map(|i| self.entry_of(&records, &head, &i.job))
            .collect();
        Ok(QueueView {
            items,
            waiting: waiting(above),
            approvable: waiting(within),
            plan,
        })
    }

    fn stop_requested(&self) -> bool {
        stop_file(self.athena.state.root()).exists()
    }

    fn plan_entry(&self, job: &str) -> Option<PlanEntry> {
        let records = self.records().ok()?;
        let head = self.athena.head().ok()?;
        self.entry_of(&records, &head, job)
    }

    fn approve(&self, proposal: &str) -> Approval {
        match self.athena.approve_in_limit(proposal, self.ui) {
            Ok(p) => Approval {
                job: Some(p.job.as_str().to_string()),
                cost: p.spent_usd,
                error: p.reader_error.clone(),
            },
            Err(error) => Approval {
                job: None,
                cost: 0.0,
                error: Some(format!("{error:#}")),
            },
        }
    }

    fn previous_rating(&self) -> Option<u8> {
        self.records()
            .ok()?
            .audience_ratings
            .last()
            .map(|r| r.rating)
    }

    fn read(&self, what: ReadOf<'_>) -> Read {
        let (result, before) = match what {
            ReadOf::Queued(job) => (
                match self.athena.go(job, AcceptedBy::Flag, self.ui) {
                    Ok(athena_core::run::GoResult::Reread(p)) => Ok(*p),
                    Ok(athena_core::run::GoResult::Finished(_)) => {
                        Err(anyhow::anyhow!("the request was already read and run"))
                    }
                    Err(error) => Err(error),
                },
                self.job_cost(job),
            ),
            ReadOf::Retry(job) => (self.athena.retry(job, self.ui).map(|(p, _)| p), 0.0),
        };
        match result {
            Ok(p) => Read {
                runnable: p.reading.as_ref().is_some_and(|r| r.capability.is_some()),
                error: p.reader_error.clone(),
                cost: p.spent_usd,
                job: Some(p.job.as_str().to_string()),
            },
            Err(error) => Read {
                job: None,
                cost: match what {
                    ReadOf::Queued(job) => (self.job_cost(job) - before).max(0.0),
                    ReadOf::Retry(_) => 0.0,
                },
                runnable: false,
                error: Some(format!("{error:#}")),
            },
        }
    }

    fn run(&self, job: &str) -> Run {
        let before = self.job_cost(job);
        let result = self.athena.go(job, AcceptedBy::Flag, self.ui);
        let cost = (self.job_cost(job) - before).max(0.0);
        match result {
            Ok(athena_core::run::GoResult::Finished(f)) => Run {
                outcome: Some(f.state),
                reason: f.reason,
                cost,
                needs_accepting: false,
                held: false,
            },
            Ok(athena_core::run::GoResult::Reread(p)) => Run {
                outcome: None,
                reason: format!(
                    "the request was read again after a configuration change and the new reading's scope differs from the earlier one{}; the reading needs accepting (`athena go {job}`)",
                    p.reader_error
                        .as_ref()
                        .map_or(String::new(), |e| format!(" ({e})"))
                ),
                cost,
                needs_accepting: true,
                held: false,
            },
            Err(error) => Run {
                outcome: None,
                reason: format!("{error:#}"),
                cost,
                needs_accepting: false,
                held: error.downcast_ref::<athena_core::disk::LowDisk>().is_some(),
            },
        }
    }

    fn land(&self, job: &str) -> Landing {
        match self.lander.land(self.athena, job, self.ui) {
            // What the landing changed is what the job's candidate changed; a landing whose candidate cannot be
            // read is taken to have changed nothing the page is built from.
            Landing::Landed { .. } => Landing::Landed {
                changed: self
                    .records()
                    .ok()
                    .and_then(|r| {
                        r.jobs.get(job).and_then(|j| {
                            j.candidate
                                .as_ref()
                                .map(|c| c.changes.iter().map(|e| e.path.clone()).collect())
                        })
                    })
                    .unwrap_or_default(),
            },
            other => other,
        }
    }

    fn check_cap(&self) -> f64 {
        let studies = self
            .athena
            .head()
            .and_then(|head| study_files(&self.athena.git, &head))
            .map_or(1, |s| s.len().max(1));
        let one = self.caps().map_or(0.0, |c| c.rate);
        one * studies as f64
    }

    fn check_page(&self) -> Check {
        let studies = match self
            .athena
            .head()
            .and_then(|head| study_files(&self.athena.git, &head))
        {
            Ok(studies) => studies,
            Err(error) => return Check::fail(format!("the studies could not be read: {error:#}")),
        };
        if studies.is_empty() {
            return Check::fail(
                "no study was found under studies/, so there is no page text to check",
            );
        }
        // The floor: each model must be consistent with itself. It costs nothing, so it goes first.
        let mut problems = Vec::new();
        for (name, files) in &studies {
            if let Some((_, bytes)) = files.iter().find(|(f, _)| f == "model.json") {
                match serde_json::from_slice::<serde_json::Value>(bytes) {
                    Ok(model) => problems.extend(
                        model_contradictions(&model)
                            .into_iter()
                            .map(|p| format!("studies/{name}/model.json: {p}")),
                    ),
                    Err(error) => problems.push(format!(
                        "studies/{name}/model.json: not valid JSON: {error}"
                    )),
                }
            }
        }
        if !problems.is_empty() {
            return Check::fail(problems.join("; "));
        }
        // The source check: every page text file of every study, against that study's pinned source.
        let mut cost = 0.0;
        let mut files_checked = 0;
        for (name, files) in &studies {
            let study = match study_text(name, files) {
                Ok(study) => study,
                Err(why) => {
                    problems.push(format!("studies/{name}: {why}"));
                    continue;
                }
            };
            files_checked += study.files.len();
            let report = self.athena.check_study_text(&study);
            cost += report.cost_usd;
            problems.extend(report.problems);
        }
        if problems.is_empty() {
            Check::pass(format!(
                "{} study(ies), {files_checked} page text file(s), match the pinned source and are consistent with themselves",
                studies.len()
            ))
            .costing(cost)
        } else {
            Check::fail(problems.join("; ")).costing(cost)
        }
    }

    fn rate(&self) -> Rate {
        let before = self.audience_cost();
        let result = self.athena.audience(self.page, &Target::explainer());
        let cost = (self.audience_cost() - before).max(0.0);
        match result {
            Ok(Some(rated)) => Rate {
                rating: Some(rated.rating.rating),
                cost,
                error: None,
            },
            Ok(None) => Rate {
                rating: None,
                cost,
                error: Some("a nested run makes no rating".into()),
            },
            Err(error) => Rate {
                rating: None,
                cost,
                error: Some(format!("{error:#}")),
            },
        }
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

    fn limits(budget: f64) -> Limits {
        Limits {
            budget_usd: budget,
            target_rating: 4,
            max_jobs: 10,
        }
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
    }
}
