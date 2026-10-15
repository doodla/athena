//! `athena drain`: works the steward's queue itself, within a budget the user sets. For each queued request
//! it reads and runs the job, lands it through the pull-request path (`athena land --pr`), and only then
//! rates the live page as its audience would. It stops at the first of five reasons ([`Stop`]).
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
//! - **Retry.** An item gets two attempts. After a rejected or failed run the second attempt reads the job
//!   again with `athena retry`, then runs it. After a run that passed but did not land, it lands again. Two
//!   failures stop the loop, naming the item.
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
use athena_core::pr::LinkState;
use athena_core::store::Records;
use athena_core::{Athena, Ui, steward, work_decisions};
use athena_data::records::{AcceptedBy, OutcomeState};
use serde::Serialize;

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
}

impl Budget {
    pub fn new(total: f64) -> Budget {
        Budget { total, spent: 0.0 }
    }

    /// Whether a call capped at `cap` may start: what is spent plus the cap must not exceed the budget.
    pub fn fits(&self, cap: f64) -> bool {
        self.spent + cap <= self.total + 1e-9
    }

    pub fn add(&mut self, cost: f64) {
        self.spent += cost.max(0.0);
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
}

impl Item {
    /// A queued request that has not been read.
    pub fn queued(job: &str, title: &str) -> Item {
        Item {
            job: job.into(),
            title: title.into(),
            read: false,
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
}

/// The result of landing a verified job through the pull-request path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Landing {
    /// Merged and verified: the change is on the default branch.
    Landed,
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
}

/// Why the loop stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stop {
    /// A rating taken after a landing reached the target.
    TargetMet,
    /// The next call's cap did not fit what was left of the budget.
    BudgetSpent,
    /// One item failed twice.
    RepeatedFailure,
    /// Nothing is queued (or everything queued was worked). This is not success.
    QueueEmpty,
    /// The job limit was reached.
    JobLimit,
}

impl Stop {
    pub const ALL: [Stop; 5] = [
        Stop::TargetMet,
        Stop::BudgetSpent,
        Stop::RepeatedFailure,
        Stop::QueueEmpty,
        Stop::JobLimit,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Stop::TargetMet => "target met",
            Stop::BudgetSpent => "budget spent",
            Stop::RepeatedFailure => "repeated failure",
            Stop::QueueEmpty => "queue empty",
            Stop::JobLimit => "job limit",
        }
    }

    pub fn meaning(self) -> &'static str {
        match self {
            Stop::TargetMet => {
                "a rating taken after a landing in this run reached the target rating"
            }
            Stop::BudgetSpent => {
                "spend plus the next model call's cap would exceed the budget; the rest stays queued"
            }
            Stop::RepeatedFailure => "the same item failed twice; the item is named",
            Stop::QueueEmpty => "nothing was left to work; this is not success",
            Stop::JobLimit => "the job limit was reached",
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
    pub rating: Option<RatingLine>,
    /// Queued requests not landed, left for a later run.
    pub queued: Vec<Item>,
    /// Open proposals waiting for a person. The loop did not touch them.
    pub waiting: Vec<Waiting>,
    /// Anything that went wrong beside the stop reason, such as a rating that could not be taken.
    pub notes: Vec<String>,
}

impl Report {
    /// The target was met by a rating taken after a landing. The only way to be true.
    pub fn target_met(&self) -> bool {
        self.stop == Stop::TargetMet && self.rating.is_some_and(|r| r.after_landing && r.met_target)
    }
}

struct Drain<'a> {
    steps: &'a dyn Steps,
    ui: &'a dyn Ui,
    caps: Caps,
    limits: Limits,
    budget: Budget,
    jobs: Vec<JobLine>,
    started: u32,
    landed: Vec<String>,
    rating: Option<RatingLine>,
    notes: Vec<String>,
}

enum ItemEnd {
    Landed,
    Failed,
    Budget,
    JobLimit,
}

enum RunStep {
    /// The job passed; its line is at this index.
    Passed(String, usize),
    /// The job did not pass. `ran` is whether it was run (so a retry can re-read it).
    Failed {
        job: String,
        ran: bool,
    },
    Budget,
}

impl Drain<'_> {
    fn push(&mut self, line: JobLine) -> usize {
        self.ui.line(&format!(
            "drain      {} → {} ({})",
            line.item, line.job, line.outcome
        ));
        self.jobs.push(line);
        self.jobs.len() - 1
    }

    /// Reads and runs one job. Starts only when the reading and the run both fit the budget.
    fn run_job(&mut self, item: &Item, from: ReadOf<'_>) -> RunStep {
        // An approved job was read when it was approved; its first attempt is only a run.
        let pre_read = item.read && matches!(from, ReadOf::Queued(_));
        let needed = if pre_read {
            self.caps.run
        } else {
            self.caps.read + self.caps.run
        };
        if !self.budget.fits(needed) {
            return RunStep::Budget;
        }
        self.started += 1;
        let read = if pre_read {
            Read {
                job: Some(item.job.clone()),
                cost: 0.0,
                runnable: true,
                error: None,
            }
        } else {
            self.steps.read(from)
        };
        self.budget.add(read.cost);
        let mut line = JobLine {
            item: item.job.clone(),
            job: read.job.clone().unwrap_or_else(|| item.job.clone()),
            outcome: "not read".into(),
            reason: None,
            cost_usd: read.cost,
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
                    ran: false,
                };
            }
        };
        // The reading's real cost may have used up the room the run needs.
        if !self.budget.fits(self.caps.run) {
            line.outcome = "not run".into();
            line.reason = Some("the budget did not leave room for the run".into());
            self.push(line);
            return RunStep::Budget;
        }
        let run = self.steps.run(&job);
        self.budget.add(run.cost);
        line.cost_usd += run.cost;
        line.reason = (!run.reason.is_empty()).then(|| run.reason.clone());
        match run.outcome {
            Some(OutcomeState::Landed) => {
                line.outcome = "passed".into();
                let at = self.push(line);
                RunStep::Passed(job, at)
            }
            other => {
                line.outcome = other.map_or("no outcome", OutcomeState::as_str).to_string();
                self.push(line);
                RunStep::Failed { job, ran: true }
            }
        }
    }

    fn land(&mut self, job: &str, at: usize) -> bool {
        match self.steps.land(job) {
            Landing::Landed => {
                self.jobs[at].landing = "landed".into();
                true
            }
            Landing::NotLanded(why) => {
                self.ui.line(&format!("drain      {job} not landed: {why}"));
                self.jobs[at].landing = format!("not landed: {why}");
                false
            }
        }
    }

    /// Works one item through its attempts: read and run, land, and on a failure try once more.
    fn work(&mut self, item: &Item) -> ItemEnd {
        let mut last_job = item.job.clone();
        let mut retry = false;
        let mut passed: Option<(String, usize)> = None;
        let mut failures = 0;
        loop {
            if passed.is_none() {
                if self.started >= self.limits.max_jobs {
                    return ItemEnd::JobLimit;
                }
                let from = if retry {
                    ReadOf::Retry(&last_job)
                } else {
                    ReadOf::Queued(&item.job)
                };
                match self.run_job(item, from) {
                    RunStep::Budget => return ItemEnd::Budget,
                    RunStep::Passed(job, at) => passed = Some((job, at)),
                    RunStep::Failed { job, ran } => {
                        retry = ran;
                        if ran {
                            last_job = job;
                        }
                    }
                }
            }
            if let Some((job, at)) = passed.clone()
                && self.land(&job, at)
            {
                self.landed.push(item.job.clone());
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
    fn approve_all(&mut self, proposals: Vec<Waiting>) -> (Vec<Item>, Vec<Waiting>) {
        let mut approved = Vec::new();
        let mut left = Vec::new();
        for proposal in proposals {
            if self.started + approved.len() as u32 >= self.limits.max_jobs
                || !self.budget.fits(self.caps.read + self.caps.run)
            {
                self.notes.push(format!(
                    "proposal {} is within its role's limit but was not approved: no room in the job limit or the budget",
                    proposal.id
                ));
                left.push(proposal);
                continue;
            }
            let approval = self.steps.approve(&proposal.id);
            self.budget.add(approval.cost);
            match (approval.job, approval.error) {
                (Some(job), None) => {
                    self.ui
                        .line(&format!("drain      approved {} → {job}", proposal.id));
                    approved.push(Item {
                        job,
                        title: proposal.title,
                        read: true,
                    });
                }
                (_, error) => {
                    self.notes.push(format!(
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

    /// Rates the page after a landing. Returns the stop reason when the rating ends the run.
    fn rate(&mut self) -> Option<Stop> {
        if !self.budget.fits(self.caps.rate) {
            return Some(Stop::BudgetSpent);
        }
        let rate = self.steps.rate();
        self.budget.add(rate.cost);
        match rate.rating {
            Some(value) => {
                // The only place a rating of this run is made, and it is made after a landing.
                let after_landing = !self.landed.is_empty();
                let met_target = after_landing && value >= self.limits.target_rating;
                self.rating = Some(RatingLine {
                    value,
                    after_landing,
                    met_target,
                });
                met_target.then_some(Stop::TargetMet)
            }
            None => {
                self.notes.push(format!(
                    "the page could not be rated: {}",
                    rate.error.as_deref().unwrap_or("no rating")
                ));
                None
            }
        }
    }
}

/// Works the queue within `limits`. See the module text for the rules.
pub fn drain(steps: &dyn Steps, limits: Limits, ui: &dyn Ui) -> anyhow::Result<Report> {
    let caps = steps.caps()?;
    let view = steps.queue()?;
    let previous = steps.previous_rating();
    let mut d = Drain {
        steps,
        ui,
        caps,
        limits,
        budget: Budget::new(limits.budget_usd),
        jobs: Vec::new(),
        started: 0,
        landed: Vec::new(),
        rating: None,
        notes: Vec::new(),
    };
    let mut view = view;
    let (approved, left) = d.approve_all(std::mem::take(&mut view.approvable));
    // Jobs already approved come first, then the ones approved now, then queued requests.
    let (read, unread): (Vec<Item>, Vec<Item>) = view.items.drain(..).partition(|i| i.read);
    view.items = read.into_iter().chain(approved).chain(unread).collect();
    view.waiting.extend(left);
    let mut stop = Stop::QueueEmpty;
    let mut failed_item = None;
    for item in &view.items {
        match d.work(item) {
            ItemEnd::Landed => {
                if let Some(reason) = d.rate() {
                    stop = reason;
                    break;
                }
            }
            ItemEnd::Failed => {
                stop = Stop::RepeatedFailure;
                failed_item = Some(item.job.clone());
                break;
            }
            ItemEnd::Budget => {
                stop = Stop::BudgetSpent;
                break;
            }
            ItemEnd::JobLimit => {
                stop = Stop::JobLimit;
                break;
            }
        }
    }
    let landed = d.landed.len() as u32;
    // With nothing landed the only rating there is, if any, is the stale one, and it says so.
    let rating = d.rating.or_else(|| {
        previous.map(|value| RatingLine {
            value,
            after_landing: false,
            met_target: false,
        })
    });
    Ok(Report {
        stop,
        failed_item,
        spent_usd: d.budget.spent(),
        budget_usd: limits.budget_usd,
        target_rating: limits.target_rating,
        landed,
        page_unchanged: landed == 0,
        rating,
        queued: view
            .items
            .into_iter()
            .filter(|i| !d.landed.contains(&i.job))
            .collect(),
        waiting: view.waiting,
        notes: d.notes,
        jobs: d.jobs,
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
            Ok(report) if report.link.state == LinkState::Landed => Landing::Landed,
            Ok(report) => Landing::NotLanded(format!(
                "pull request {} is {:?}, checks {:?}; it is landed once it is merged",
                report.link.pr_number, report.link.state, report.checks
            )),
            Err(error) => Landing::NotLanded(format!("{error:#}")),
        }
    }
}

/// The steps against a real repository.
pub struct RealSteps<'a> {
    pub athena: &'a Athena,
    pub ui: &'a dyn Ui,
    pub page: &'a dyn Page,
    pub lander: &'a dyn Lander,
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
    pub fn live(athena: &'a Athena, ui: &'a dyn Ui) -> RealSteps<'a> {
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
        let item = |w: steward::QueuedWork, read: bool| Item {
            job: w.job.as_str().to_string(),
            title: w.title,
            read,
        };
        let mut items: Vec<Item> = steward::approved_work(&records)
            .into_iter()
            .map(|w| item(w, true))
            .collect();
        items.extend(
            steward::queued_work(&records)
                .into_iter()
                .map(|w| item(w, false)),
        );
        Ok(QueueView {
            items,
            waiting: waiting(above),
            approvable: waiting(within),
        })
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
            },
            Ok(athena_core::run::GoResult::Reread(_)) => Run {
                outcome: None,
                reason: "the request was read again and needs accepting".into(),
                cost,
            },
            Err(error) => Run {
                outcome: None,
                reason: format!("{error:#}"),
                cost,
            },
        }
    }

    fn land(&self, job: &str) -> Landing {
        self.lander.land(self.athena, job, self.ui)
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
            }
        }
        fn land(&self, job: &str) -> Landing {
            self.calls.borrow_mut().push(format!("land {job}"));
            Landing::Landed
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

    #[test]
    fn every_stop_reason_has_a_label_and_a_meaning() {
        for stop in Stop::ALL {
            assert!(!stop.label().is_empty() && !stop.meaning().is_empty());
        }
        assert!(Stop::QueueEmpty.meaning().contains("not success"));
    }
}
