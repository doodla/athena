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
//! - **Concurrency.** A group is worked by up to `max_concurrency` workers; the next group starts only when
//!   the group before it is done, so an item that depends on another starts after it has finished. Landing
//!   and rating go one at a time (a lock), and every model call's cap is held against the budget while it is
//!   in flight, so jobs side by side cannot overspend it. The first reason to stop ends the run: no worker
//!   starts another job, jobs already started are finished (a failure or a refusal stops no sibling that is
//!   in flight), and the first reason found is the one reported. A worker that panics is not isolated: the
//!   drain ends with the panic once the other workers have finished.
//! - **Read-ahead.** When more than one job may run at once and [`Steps::read_ahead`] is above 0,
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
//!   as it is. With one job at a time (`max_in_flight` 1, so [`drain`] or a bound of 1), or `read_ahead` 0,
//!   nothing is read ahead and the drain behaves as it always did. The real steps do not turn this on yet.
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
//!   request head is queued or in progress. A pull request that is still open with its checks pending, such
//!   as one whose merge the host refused for its repository rules before every check finished, is
//!   [`Landing::Waiting`]: the job line says `waiting to merge`, the item is not tried again and does not
//!   count toward [`Stop::RepeatedFailure`], and the run goes on to the next item. `athena land --pr` merges
//!   it once the checks have finished.
//! - **Stop at a job boundary.** [`Steps::stop_requested`] is asked before every job. A job that has started
//!   is finished (landed and rated) and no further job starts: [`Stop::Requested`]. Nothing is killed.
//!
//! - **Athena rebuilds itself after landing its own code.** A landing that changed `crates/`, `xtask/`,
//!   `Cargo.toml`, `Cargo.lock` or `setup.sh` ([`touches_build`]) leaves the installed binary behind the
//!   checkout, and the next config read or landing could fail on it. So before the next job starts, the loop
//!   calls [`Steps::rebuild`] (the real one runs `setup.sh` in the checkout, which builds from the landed tree
//!   and installs the binary). If that fails the run ends with [`Stop::RebuildFailed`] and
//!   [`Report::rebuild_output`] holds the build output. If it succeeds the pass ends with [`Stop::Rebuilt`] and
//!   [`Report::restart`] holds what is left of the budget and job limit; [`follow_restart`] hands those to the
//!   new binary (`athena drain` again, see [`reexec_args`]), which carries on with the queue. A stop asked for
//!   meanwhile wins, and a run with no budget or no job limit left ends as such instead of restarting.
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
    /// True when dispatch was held for lack of free disk space (`environment: low disk`), or because the
    /// job's budget does not cover the whole path still to run (`budget short`). The job is still proposed
    /// and was not run: it is neither retried nor counted as a failure, and a later drain runs the same job
    /// once space is free or the budget is raised.
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
    /// The pull request is open and the host holds the merge until its checks finish: not a failure. The
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
    /// more than one job runs at once. 0 (the default) reads nothing ahead. The real steps keep the default
    /// until the drain's pipeline settings are wired in.
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
    /// One item failed twice. An item whose reader gave no reading twice is not this: it is skipped, with its
    /// reason, and the next item runs.
    RepeatedFailure,
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
}

impl Stop {
    pub const ALL: [Stop; 8] = [
        Stop::TargetMet,
        Stop::BudgetSpent,
        Stop::RepeatedFailure,
        Stop::QueueEmpty,
        Stop::JobLimit,
        Stop::Requested,
        Stop::Rebuilt,
        Stop::RebuildFailed,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Stop::TargetMet => "target met",
            Stop::BudgetSpent => "budget spent",
            Stop::RepeatedFailure => "repeated failure",
            Stop::QueueEmpty => "queue empty",
            Stop::JobLimit => "job limit",
            Stop::Requested => "stop requested",
            Stop::Rebuilt => "rebuilt",
            Stop::RebuildFailed => "rebuild failed",
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
            Stop::Rebuilt => {
                "a landing changed Athena's own code, so Athena was rebuilt and installed; the new binary carries on with the remaining budget and job limit"
            }
            Stop::RebuildFailed => {
                "a landing changed Athena's own code and rebuilding it failed; the build output follows and the rest stays queued"
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
    /// The build output, when `stop` is `rebuild-failed`.
    pub rebuild_output: Option<String>,
    /// What is left to carry on with, when `stop` is `rebuilt` ([`follow_restart`]).
    pub restart: Option<Restart>,
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
    /// Set by a landing that touched [`BUILD_PATHS`] and cleared by the rebuild that follows it.
    rebuild_due: bool,
    /// The build output of a failed rebuild.
    rebuild_output: Option<String>,
    /// What the planning pass spent reading an item, charged to that item's job line when it runs.
    read_cost: std::collections::BTreeMap<String, f64>,
    /// Why the run is ending, once any worker has found a reason (the first reason found stands), and the
    /// item that failed twice when that is the reason. No worker starts a job once this is set; jobs already
    /// started are finished.
    stop: Option<(Stop, Option<String>)>,
    // Read-ahead. All of this stays empty unless more than one job runs at once and `Steps::read_ahead` is
    // above 0.
    /// Readings made ahead and not yet claimed by a job that starts.
    ahead: BTreeMap<String, Ahead>,
    /// Items the read-ahead is reading or planning right now.
    reading: BTreeSet<String>,
    /// Items the read-ahead has already tried once, so a dropped reading is not made again.
    pumped: BTreeSet<String>,
    /// Items that have started a job, so the money their job needs is held.
    begun: BTreeSet<String>,
    /// Items whose work is over (landed, failed or left queued).
    done: BTreeSet<String>,
    /// The paths of each landing in this run, in order.
    landings: Vec<Vec<String>>,
    /// The paths each item's plan entry says it touches.
    touches: BTreeMap<String, Vec<String>>,
    /// The index of the batch that is running.
    batch: usize,
    /// The batches are over (or the run is ending): the read-ahead stops.
    pump_done: bool,
    /// The read-ahead has ended, so no worker waits on a reading that will not finish.
    pump_gone: bool,
    /// Items this run has handed to a worker; a new plan never offers them again.
    tried: BTreeSet<String>,
    /// Items that have a "passed over" line, so none gets two.
    passed_over: BTreeSet<String>,
    /// What each new plan of this run needs to remember from the ones before.
    carry: Carry,
    /// Workers whose run passed and whose job is not landed yet, in the order the runs passed.
    queue: LandingQueue,
}

/// What a drain keeps from one plan to the next, merged into each fresh queue view by `Drain::prepare`. A
/// fresh view may not list a job the drain approved, and must not make it read or pay for an item twice.
#[derive(Clone, Default)]
struct Carry {
    /// Jobs approved in this run, with their plan entries.
    approved: Vec<(Item, Option<PlanEntry>)>,
    /// Items already read by the planning pass: whether the item counts as read, and its entry after reading.
    prepared: BTreeMap<String, (bool, PlanEntry)>,
    /// Items the planning pass refused after trying to read them.
    forced: Vec<Refusal>,
}

/// Sets a flag of the shared state and wakes every waiter when it is dropped, so it happens on every way out
/// of a scope, a panic included.
struct FlagOnDrop<'a> {
    state: &'a Mutex<State>,
    wake: &'a Condvar,
    set: fn(&mut State),
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
struct LandingQueue {
    /// The ticket the next join gets.
    next: u64,
    /// The tickets in line, the head first.
    line: VecDeque<u64>,
}

impl LandingQueue {
    /// Takes a place at the back of the line and returns its ticket.
    fn join(&mut self) -> u64 {
        let ticket = self.next;
        self.next += 1;
        self.line.push_back(ticket);
        ticket
    }

    /// Whether `ticket` is first in line.
    fn is_head(&self, ticket: u64) -> bool {
        self.line.front() == Some(&ticket)
    }

    /// Gives up the place of `ticket`, wherever it is in the line. A ticket that is not in line is ignored.
    fn leave(&mut self, ticket: u64) {
        self.line.retain(|t| *t != ticket);
    }
}

/// A place in the [`LandingQueue`] of a drain. Dropping it gives the place up and wakes every waiter, so the
/// next in line goes on whichever way the holder leaves its scope, a panic included.
struct LandingSlot<'a> {
    state: &'a Mutex<State>,
    wake: &'a Condvar,
    ticket: u64,
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
enum Pick {
    /// Nothing is left to read ahead.
    Done,
    /// Something could be read, but not yet.
    Wait,
    /// Read the item at this index of the upcoming items.
    Start(usize),
}

struct Drain<'a, S: ?Sized, U: ?Sized> {
    steps: &'a S,
    ui: &'a U,
    caps: Caps,
    limits: Limits,
    state: Mutex<State>,
    /// Held while a job is landed and while the page is rated: landings and ratings go one at a time even
    /// when jobs run side by side, because each pushes a branch and rates the page as it then stands. Workers
    /// join the landing queue when their run passes, and only the head of the queue takes this lock.
    landing: Mutex<()>,
    /// Woken when a reading made ahead is stored, a job ends or a landing is recorded.
    wake: Condvar,
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
    /// Dispatch was held for lack of free disk space, or the job's budget did not cover the whole path; the
    /// item stays queued, and nothing else stops.
    Held,
    /// The item's reader gave no reading twice. It is skipped with its reason and the next item runs; this
    /// is not a failed run, so it does not stop the drain.
    Skipped,
    /// The job passed and its pull request is open, waiting on checks that have not finished. It is not
    /// retried and is not a failure: the next item runs, and the job merges on a later `athena land --pr`.
    Waiting,
}

/// What one call of [`Drain::land`] came to.
enum LandStep {
    Landed,
    Waiting,
    NotLanded,
}

/// The outcome of an item held because its job's budget does not cover the whole path still to run; also the
/// start of the reason a run held for it gives.
const BUDGET_SHORT: &str = "budget short";

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
    /// The job was held for lack of free disk space or of budget. It is not a failure and not tried again in
    /// this run.
    Held,
    /// A queued job's reader gave no reading, with this reason. The job is recorded without a reading; its
    /// next attempt reads it again. It is not a failure of a run.
    Unread(String),
    Budget,
    JobLimit,
    /// Another worker had already ended the run.
    Ended,
}

/// `text` on one line, for a line of the drain's log.
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

impl<S: Steps + ?Sized, U: Ui + ?Sized> Drain<'_, S, U> {
    fn st(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn push(&self, line: JobLine) -> usize {
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

    fn note(&self, text: String) {
        self.st().notes.push(text);
    }

    /// A note that a new plan of the same run would only repeat.
    fn note_once(&self, text: String) {
        let mut st = self.st();
        if !st.notes.contains(&text) {
            st.notes.push(text);
        }
    }

    /// Logs that the planned item `id` is not worked, with the reason: one line per item per run.
    fn pass_over(&self, id: &str, reason: &str) {
        let first = self.st().passed_over.insert(id.to_string());
        if first {
            self.ui.line(&format!(
                "plan       {id} passed over: {}",
                one_line(reason)
            ));
        }
    }

    /// The landing text on the last job line of `item`; a waiting job's line says why it waits to merge.
    fn last_landing(&self, item: &str) -> String {
        let st = self.st();
        st.jobs
            .iter()
            .rev()
            .find(|l| l.item == item && l.landing.starts_with("waiting to merge"))
            .map(|l| l.landing.clone())
            .unwrap_or_else(|| "waiting to merge: its pull request waits on checks".to_string())
    }

    /// The reason on the last job line of `item`, else `default`.
    fn last_reason(&self, item: &str, default: &str) -> String {
        let st = self.st();
        st.jobs
            .iter()
            .rev()
            .find(|l| l.item == item)
            .and_then(|l| l.reason.clone())
            .unwrap_or_else(|| default.to_string())
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
    fn run_job(&self, item: &Item, from: ReadOf<'_>, source: Source) -> RunStep {
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
                // counted as a failed run.
                let unread = matches!(from, ReadOf::Queued(_))
                    && !pre_read
                    && read.job.is_some()
                    && read.error.is_some();
                let reason = line.reason.clone().unwrap_or_default();
                self.push(line);
                if unread {
                    return RunStep::Unread(reason);
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
                line.outcome = if run.reason.starts_with(BUDGET_SHORT) {
                    BUDGET_SHORT.into()
                } else {
                    athena_core::disk::LOW_DISK.into()
                };
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

    /// Takes the next place in the landing queue. The place is kept until the returned slot is dropped.
    fn enter_queue(&self) -> LandingSlot<'_> {
        let ticket = self.st().queue.join();
        LandingSlot {
            state: &self.state,
            wake: &self.wake,
            ticket,
        }
    }

    /// Returns when `slot` is the head of the landing queue. A worker that has to wait says so once, outside
    /// the state lock; the head returns at once and says nothing.
    fn await_head(&self, job: &str, slot: &LandingSlot<'_>) {
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

    fn land(&self, job: &str, at: usize) -> LandStep {
        let _one_at_a_time = self.landing.lock().unwrap_or_else(PoisonError::into_inner);
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
    fn work(&self, item: &Item, ahead: Option<Ahead>) -> ItemEnd {
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
    fn attempts(&self, item: &Item, mut ahead: Option<Ahead>, last_job: &mut String) -> ItemEnd {
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
                    RunStep::Unread(reason) => {
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
                    LandStep::NotLanded => {}
                }
            }
            failures += 1;
            if failures >= ATTEMPTS {
                return ItemEnd::Failed;
            }
        }
    }

    /// The stale-reading re-check, made when the job of a reading made ahead starts. A landing since the
    /// reading began may have changed a path the reading touches; if none did, the reading is used as it is.
    /// Otherwise the drain notes which paths and asks [`Steps::recheck`]: a reading that still holds is kept,
    /// and one that does not is replaced by [`Steps::reread`].
    fn vet(&self, item: &Item, made: Ahead) -> Source {
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
        for idx in 0..view.items.len() {
            // An interrupted job was read before it ran; its retry reads the request again.
            if view.items[idx].read || view.items[idx].interrupted {
                continue;
            }
            let id = view.items[idx].job.clone();
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
        let entries: Vec<PlanEntry> = view
            .plan
            .iter()
            .filter(|e| !forced.iter().any(|f| f.id == e.id))
            .cloned()
            .collect();
        let mut plan = Plan::build_with(&entries, &self.steps.job_history());
        for contradiction in plan.priority_violations(&entries) {
            self.note_once(format!("plan order contradicts priority: {contradiction}"));
        }
        plan.refused.extend(forced);
        plan
    }

    /// Records that the planning pass read `id`, so a later plan of the run neither reads nor pays for it again.
    fn remember_reading(&self, id: &str, read: bool, entry: &PlanEntry, refusal: Option<&Refusal>) {
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
    fn sweep(&self, view: &QueueView, plan: &Plan, wave: &[String]) {
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
        let why = self
            .st()
            .stop
            .as_ref()
            .map_or(Stop::QueueEmpty.label(), |(stop, _)| stop.label());
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
                rebuild_due: false,
                rebuild_output: None,
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
                passed_over: BTreeSet::new(),
                carry: Carry::default(),
                queue: LandingQueue::default(),
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
    fn rebuild_if_due(&self) -> Option<bool> {
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
                st.stop = Some((Stop::RebuildFailed, None));
                None
            }
        }
    }

    /// What a worker does when its item has ended: a landed item is followed by a rating, and a reason to
    /// stop ends the run for every worker. One item's failure stops no job already in flight.
    fn finish(&self, item: &Item, end: ItemEnd) {
        match end {
            ItemEnd::Landed => {
                let Some(rebuilt) = self.rebuild_if_due() else {
                    return;
                };
                if let Some(reason) = self.rate() {
                    self.set_stop(reason, None);
                }
                // The first reason found stands; a run that is ending anyway has still been rebuilt.
                if rebuilt {
                    self.set_stop(Stop::Rebuilt, None);
                }
            }
            ItemEnd::Failed => self.set_stop(Stop::RepeatedFailure, Some(item.job.clone())),
            ItemEnd::Budget => {
                self.pass_over(
                    &item.job,
                    &self.last_reason(&item.job, "the budget did not cover its read and run"),
                );
                self.set_stop(Stop::BudgetSpent, None);
            }
            ItemEnd::JobLimit => {
                self.pass_over(&item.job, "over the job limit");
                self.set_stop(Stop::JobLimit, None);
            }
            ItemEnd::Stop => {
                self.pass_over(&item.job, "run ended first: stop requested");
                self.set_stop(Stop::Requested, None);
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
                    self.last_reason(&item.job, "free disk space is below the minimum")
                ),
            ),
            ItemEnd::Waiting => self.pass_over(&item.job, &self.last_landing(&item.job)),
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
            let made = self.claim(&item);
            let end = self.work(&item, made);
            self.st().done.insert(item.job.clone());
            self.wake.notify_all();
            self.finish(&item, end);
        }
    }

    /// A worker takes `item`. If the read-ahead is reading it right now, waits for that reading to end (it is
    /// never read twice), then takes the reading made ahead, if there is one.
    fn claim(&self, item: &Item) -> Option<Ahead> {
        let mut st = self.st();
        while st.reading.contains(&item.job) && !st.pump_gone {
            st = self.pause(st);
        }
        st.ahead.remove(&item.job)
    }

    /// Waits a short while for a change of the shared state. Waiters check their condition again afterwards, so
    /// a wake-up that comes early or not at all costs only a few milliseconds.
    fn pause<'g>(&self, guard: MutexGuard<'g, State>) -> MutexGuard<'g, State> {
        self.wake
            .wait_timeout(guard, Duration::from_millis(20))
            .unwrap_or_else(PoisonError::into_inner)
            .0
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
    ///
    /// With [`Steps::read_ahead`] above 0, one more thread reads (and plans) the items of the batches to come
    /// while the batch runs; see the module text.
    ///
    /// `batches` is one wave (the drain plans again before the next): the batch to run. `later` is the
    /// batches of the same plan after it; it only chooses what to read ahead and is never dispatched.
    fn run_parallel(&self, batches: Vec<Vec<Item>>, later: Vec<Vec<Item>>, workers: usize) {
        let window = self.steps.read_ahead() as usize;
        if window == 0 || workers < 2 {
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

    fn run_batches(&self, batches: Vec<Vec<Item>>, workers: usize) {
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
    fn pick_ahead(&self, st: &State, upcoming: &[(usize, Item)], window: usize) -> Pick {
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
    fn read_ahead_pump(&self, upcoming: &[(usize, Item)], window: usize) {
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

/// Works the queue within `limits`, one job at a time. See the module text for the rules.
pub fn drain(steps: &dyn Steps, limits: Limits, ui: &dyn Ui) -> anyhow::Result<Report> {
    let caps = steps.caps()?;
    let d = Drain::new(steps, ui, caps, limits);
    drain_with(&d, steps, true, |d, wave, _| d.run_serial(wave))
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
    drain_with(&d, steps, workers == 1, move |d, wave, later| {
        if workers == 1 {
            d.run_serial(wave);
        } else {
            d.run_parallel(wave, later, workers);
        }
    })
}

/// The batches to work from `plan`, in the order to work them: items with no plan entry first, each alone,
/// then the plan's groups in the plan's own order, without what this run has `tried`. It only filters; the
/// order is the plan's, built by [`Plan::build_with`] in `Drain::prepare` (the function `athena plan` calls).
fn batches_for(
    items: &[Item],
    entries: &[PlanEntry],
    plan: &Plan,
    tried: &BTreeSet<String>,
) -> Vec<Vec<Item>> {
    let mut batches: Vec<Vec<Item>> = items
        .iter()
        .filter(|i| !tried.contains(&i.job) && !entries.iter().any(|e| e.id == i.job))
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

fn drain_with<'a, S: Steps + ?Sized, U: Ui + ?Sized>(
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
    // The plan is built again before each wave (one item, or one group under `drain_concurrent`) from the
    // queue as it then stands, by the same `prepare` as `athena plan`. `tried` is what this run has already
    // handed to a worker, so the loop ends.
    let first_plan = plan.clone();
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
            now.truncate(1);
        }
        wave = now.iter().map(|i| i.job.clone()).collect();
        d.st().tried.extend(wave.iter().cloned());
        run(d, vec![now], batches);
        if d.stopped() {
            break;
        }
        match steps.queue() {
            Ok(mut next) => {
                next.waiting = std::mem::take(&mut view.waiting);
                view = next;
                plan = d.prepare(&mut view, reserve);
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
    let (mut stop, failed_item) = st.stop.clone().unwrap_or((Stop::QueueEmpty, None));
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
        rebuild_output: st.rebuild_output.clone(),
        restart,
    })
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

// ---------------------------------------------------------------------------------------------------------
// The real world

/// How a verified job is landed.
pub trait Lander {
    fn land(&self, athena: &Athena, job: &str, ui: &dyn Ui) -> Landing;
}

/// Lands through the pull-request path of `athena land --pr`: push the job's branch, open the pull request,
/// wait for every check run to finish and be green, merge only when `[landing] merge_via_rest` is on. Only a
/// merged, verified pull request is a landing; one that waits on unfinished checks is [`Landing::Waiting`].
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
        landing_from_report(report)
    }
}

/// What the drain makes of the result of `athena land --pr` for one job. A merged pull request is a landing.
/// An open one whose checks are still pending (the host holds its merge until they finish, including a merge it
/// refused for its repository rules) is [`Landing::Waiting`], not a failure. Anything else is
/// [`Landing::NotLanded`].
fn landing_from_report(report: anyhow::Result<PrReport>) -> Landing {
    match report {
        // The paths it changed are read from the job's candidate record by `RealSteps::land`.
        Ok(report) if report.link.state == LinkState::Landed => Landing::Landed {
            changed: Vec::new(),
        },
        Ok(report)
            if report.link.state == LinkState::Open && report.checks == Some(Checks::Pending) =>
        {
            Landing::Waiting(format!(
                "pull request {} is open and its checks have not finished; it merges once they have",
                report.link.pr_number
            ))
        }
        Ok(report) => Landing::NotLanded(format!(
            "pull request {} is {:?}, checks {:?}; it is landed once it is merged",
            report.link.pr_number, report.link.state, report.checks
        )),
        Err(error) => Landing::NotLanded(format!("{error:#}")),
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

/// Builds and installs Athena from the checkout at `dir` by running its `setup.sh`, which is the one command
/// that does it (`./setup.sh`). A checkout with no `setup.sh` is not Athena's own source: nothing is built.
/// A failure is the exit status followed by what the script printed.
pub fn run_setup(dir: &std::path::Path) -> Result<Rebuild, String> {
    let script = dir.join("setup.sh");
    if !script.is_file() {
        return Ok(Rebuild::NotApplicable);
    }
    let out = std::process::Command::new("sh")
        .arg(&script)
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .output()
        .map_err(|e| format!("could not run {}: {e}", script.display()))?;
    if out.status.success() {
        return Ok(Rebuild::Installed);
    }
    Err(format!(
        "{} failed ({})\n{}{}",
        script.display(),
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    ))
}

/// The default priority of the work a role raises (it applies when the job's own words give none): the `Priority: <level>` line in the standing instruction of
/// the role's newest recorded definition, normal when the role is not recorded or states none.
fn role_priority(records: &Records, role: &str) -> athena_core::plan::Priority {
    records
        .roles
        .iter()
        .filter(|r| r.name == role)
        .max_by_key(|r| r.version)
        .map_or(athena_core::plan::Priority::Normal, |r| {
            athena_core::plan::Priority::from_text(&r.prompt)
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
        // The proposal this job carries out, if it was made by approving one.
        let proposal = records.work_items.iter().find(|work| {
            work_decisions::decision_of(records, work).is_some_and(|decision| {
                matches!(
                    &decision.decision,
                    athena_data::records::WorkDecisionState::Approved { job: made }
                        if made.as_str() == job
                )
            })
        });
        let hint = proposal.map(|work| work.rung);
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
        let mut entry = athena_core::reading::plan_entry(
            record,
            hint,
            &athena_core::reading::Lookup {
                job_exists: &job_exists,
                path_exists: &path_exists,
                brief: &brief,
            },
        );
        // A job made from a proposal ranks like any other item: normal unless a priority record, its words or
        // the role that raised it say otherwise.
        if let Some(work) = proposal {
            entry.priority = athena_core::plan::proposal_priority(
                (!record.priorities.is_empty()).then(|| record.priority()),
                entry.priority,
                role_priority(records, &work.role),
            );
        }
        Some(entry)
    }

    fn audience_cost(&self) -> f64 {
        self.records()
            .map(|r| r.audience_runs.iter().map(|i| i.cost.best_lower()).sum())
            .unwrap_or(0.0)
    }

    fn table(&self) -> InFlight {
        InFlight::at(self.athena.state.root())
    }

    /// A table write that fails is a warning line: the table is bookkeeping and never fails a job.
    fn table_result(&self, result: anyhow::Result<()>) {
        if let Err(e) = result {
            self.ui.line(&format!(
                "warning: could not write the in-flight table: {e:#}"
            ));
        }
    }
}

impl Steps for RealSteps<'_> {
    fn review_passes(&self) -> Vec<String> {
        self.athena.review_passes(self.ui)
    }

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

    fn job_history(&self) -> Vec<JobEnd> {
        self.records()
            .map(|r| athena_core::reading::job_history(&r))
            .unwrap_or_default()
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
        if let ReadOf::Retry(old) = what {
            // The retry is a new job. A row waiting for it (the checks or verifier the old job reached) goes
            // to the new job, keeping its stage, until the new job's producer enters; a retry that cannot run
            // leaves no row.
            let moved = match &result {
                Ok(p)
                    if p.reader_error.is_none()
                        && p.reading.as_ref().is_some_and(|r| r.capability.is_some()) =>
                {
                    self.table()
                        .move_row(old, p.job.as_str(), self.athena.state.work_dir(&p.job))
                }
                _ => self.table().remove(old),
            };
            self.table_result(moved);
        }
        match result {
            Ok(p) => Read {
                runnable: p.reading.as_ref().is_some_and(|r| r.capability.is_some()),
                error: p.reader_error.clone(),
                cost: p.spent_usd,
                job: Some(p.job.as_str().to_string()),
                held: false,
            },
            Err(error) => Read {
                job: None,
                cost: match what {
                    ReadOf::Queued(job) => (self.job_cost(job) - before).max(0.0),
                    ReadOf::Retry(_) => 0.0,
                },
                runnable: false,
                error: Some(format!("{error:#}")),
                held: error
                    .downcast_ref::<athena_core::reading::BudgetShort>()
                    .is_some(),
            },
        }
    }

    fn reclaim_interrupted(&self) {
        self.athena.state.registry().reap();
        self.athena.reclaim_interrupted(self.ui);
    }

    fn run(&self, job: &str) -> Run {
        let before = self.job_cost(job);
        // The run reports each stage it enters; the table holds the latest.
        let sink = TableSink {
            table: self.table(),
            athena: self.athena,
            ui: self.ui,
        };
        let result = self
            .athena
            .go_with_stages(job, AcceptedBy::Flag, self.ui, &sink);
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
            Err(error) => {
                let short = error
                    .downcast_ref::<athena_core::reading::BudgetShort>()
                    .is_some();
                Run {
                    outcome: None,
                    reason: if short {
                        format!("{BUDGET_SHORT}: {error:#}")
                    } else {
                        format!("{error:#}")
                    },
                    cost,
                    needs_accepting: false,
                    held: short || error.downcast_ref::<athena_core::disk::LowDisk>().is_some(),
                }
            }
        }
    }

    fn land(&self, job: &str) -> Landing {
        // `go` does not land a verified job; this is where the drain enters the landing stage.
        if let Ok(id) = JobId::parse(job) {
            TableSink {
                table: self.table(),
                athena: self.athena,
                ui: self.ui,
            }
            .enter(&id, Stage::Landing);
        }
        let landing = match self.lander.land(self.athena, job, self.ui) {
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
        };
        // The landing is over, landed or not: a landing that failed is tried again by the drain's next
        // attempt, which sets the row again.
        self.table_result(self.table().remove(job));
        landing
    }

    fn in_flight(&self) -> Vec<String> {
        let found = self.table().read();
        let mut lines: Vec<String> = found
            .rows
            .iter()
            .map(|r| {
                format!(
                    "the in-flight table holds a row left by an earlier run: job {} was at stage {} in {}; it is not acted on",
                    r.job,
                    r.stage.as_str(),
                    r.worktree.display()
                )
            })
            .collect();
        lines.extend(found.corrupt.map(|why| {
            format!("the in-flight table was not read and the drain goes on without it: {why}")
        }));
        lines
    }

    fn item_over(&self, job: &str) {
        self.table_result(self.table().remove(job));
    }

    fn rebuild(&self) -> Result<Rebuild, String> {
        run_setup(self.athena.git.dir())
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
        stop_after: Option<String>,
        /// Jobs whose landing is a merge the host holds until checks finish.
        waiting: BTreeSet<String>,
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
                stop_after: None,
                waiting: BTreeSet::new(),
                reads: AtomicUsize::new(0),
                ended: AtomicUsize::new(0),
                widest: AtomicUsize::new(0),
            }
        }

        fn waits(mut self, job: &str) -> World {
            self.waiting.insert(job.into());
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
            Landing::Landed {
                changed: self
                    .landed
                    .get(job)
                    .cloned()
                    .unwrap_or_else(|| vec![format!("src/{job}.rs"), "CHANGELOG.md".into()]),
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
        assert!(report.failed_item.is_none());
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

    #[test]
    fn read_ahead_is_off_with_a_bound_of_one() {
        // One job at a time, through both entry points, and a window of 0 with two jobs at a time.
        let runs: [(&str, u32, u32); 3] = [
            ("drain_concurrent 1", 1, 1),
            ("drain", 1, 1),
            ("window 0", 0, 2),
        ];
        for (how, window, bound) in runs {
            let world = World::chain(window);
            let report = if how == "drain" {
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
