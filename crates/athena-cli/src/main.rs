//! `athena`: an agent orchestration system's command line (design §3).
//!
//! Exit codes: 0 when the command did what it was asked (for `go`, the job landed); 2 when a job did not
//! land (rejected, failed, stale, or blocked) or a check found violations; 1 when Athena itself erred. For
//! `drain`, 0 means the target was met after a landing, and 2 any other stop.

mod commands;

use anyhow::Result;
use athena_cli::{display, drain, hooks, slot_exec, wiring};
use athena_core::reading::Proposal;
use athena_core::run::GoResult;
use athena_core::{Athena, Ui};
use athena_data::records::{AcceptedBy, OutcomeState};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use std::io::{BufRead, IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "athena",
    about = "Take a lazy request, show how it was read and what is predicted, run it on a sub-agent, verify it independently, and land a traceable commit."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Read a request and show the reading and predictions; on a terminal, Enter runs it.
    Ask {
        /// The request, in a line or two.
        #[arg(required = true, num_args = 1..)]
        request: Vec<String>,
        /// Run without waiting at the prompt (the reading is printed, but its acceptance stays unresolved).
        #[arg(long, conflicts_with = "later")]
        yes: bool,
        /// Only record the request, to be read and run later with `athena go`; no model is called.
        #[arg(long)]
        later: bool,
        #[arg(long)]
        json: bool,
    },
    /// Try a rejected, failed, timed out, stale or interrupted job again, starting from its introspection
    /// record (the proposed next request, the cause and the prevention) and shown as `ask` shows a reading;
    /// Enter runs it. A stale job keeps its verified candidate: the retry merges it onto the branch tip. A
    /// clean merge is checked and verified again with no planner or producer run; a conflict starts a producer
    /// from the conflicted merge. The retry starts from the base when the candidate cannot be merged (it is
    /// gone, or the branch no longer holds its base).
    Retry {
        job: String,
        /// Run without waiting at the prompt (the reading is printed, but its acceptance stays unresolved).
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        json: bool,
    },
    /// Write a pending defaults proposal for every failure class whose introspections span enough distinct
    /// jobs. Proposals are records; `defaults.toml` is not edited. `athena status` lists them.
    Propose {
        #[arg(long)]
        json: bool,
    },
    /// Build the labeled set from introspections that have a class label (a person's, or one derived from
    /// evidence), score Jev's recorded shadow answers against it (Brier skill over two references, sample
    /// size, interval) and give the promotion verdict of defaults-loop section 10.4. With too few labeled
    /// cases it says so and gives no verdict. Read-only unless --derive is given.
    ScoreJev {
        /// Cases recorded before this time (RFC 3339) calibrate and later ones test. Default: the earliest
        /// two thirds by time calibrate.
        #[arg(long)]
        cut: Option<String>,
        /// First derive class labels from what happened after each introspection (the retry that followed,
        /// recurrence after a prevention landed, findings on the same job) and commit them as new
        /// class-label records, each citing the records it rests on. Existing records are never edited, a
        /// person's label always wins, and a rerun adds nothing.
        #[arg(long)]
        derive: bool,
        #[arg(long)]
        json: bool,
    },
    /// Show one introspection (what happened, class, cause, prevention, Jev's shadow answer) and record your
    /// verdict on its class in the same step, as a class-label record. The introspection is not edited.
    /// Give exactly one of --right, --class and --new-class.
    Review {
        /// The introspection id (or a unique prefix or suffix of it).
        id: String,
        /// The class the introspection recorded is right.
        #[arg(long)]
        right: bool,
        /// The right class is this other class in the registry.
        #[arg(long, value_name = "CLASS_ID")]
        class: Option<String>,
        /// No registry class fits: add a class with this name (needs --description).
        #[arg(long, value_name = "NAME")]
        new_class: Option<String>,
        /// One line saying what the new class is.
        #[arg(long)]
        description: Option<String>,
        /// A one-line note kept with the verdict.
        #[arg(long)]
        note: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Accept an open defaults proposal: submit the ordinary job "apply defaults proposal <id>", which lists
    /// `defaults.toml` as a deliverable and is read and run like any request. Enter runs it.
    AcceptProposal {
        id: String,
        /// Run without waiting at the prompt (the reading is printed, but its acceptance stays unresolved).
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        json: bool,
    },
    /// Reject an open defaults proposal, with your reason. Its class is not proposed again until enough
    /// new introspections arrive.
    RejectProposal {
        id: String,
        #[arg(long)]
        why: String,
        #[arg(long)]
        json: bool,
    },
    /// Approve an open steward proposal (a work item above the steward's limit) by its id, as `athena status`
    /// lists it: submit an ordinary job whose request copies the proposal's title, rung, source and the
    /// content of each cited record, then read it like any request. Enter runs it.
    ApproveWork {
        id: String,
        /// Run without waiting at the prompt (the reading is printed, but its acceptance stays unresolved).
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        json: bool,
    },
    /// Reject an open steward proposal by its id, with your reason. The decision is appended as a record.
    RejectWork {
        id: String,
        #[arg(long)]
        why: String,
        #[arg(long)]
        json: bool,
    },
    /// Run the audience: fetch the live explainer page (or --url), have a model read it as a newcomer, and
    /// record a 1 to 5 rating with findings that cite their evidence. A rating of 2 or lower queues a work
    /// item. The page and the repository's files are not changed.
    Audience {
        /// Rate this page instead of the live explainer.
        #[arg(long)]
        url: Option<String>,
        /// Who reads it; default `newcomer`.
        #[arg(long)]
        reader: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Run the auditor on a job: a model judges the job's real outcome (its request and the files it changed)
    /// against BRIEF.md and the owner's goals, not against the job's own criteria, and records findings that
    /// each cite their evidence. Nothing is changed. Defaults to the latest job.
    Audit {
        /// The job to audit: a full id, a unique suffix, or `latest`.
        #[arg(default_value = "latest")]
        job: String,
        #[arg(long)]
        json: bool,
    },
    /// Hestia, the continuity role: sweep the system's upkeep checks into one recorded report, or print the
    /// last report. Reports only; nothing is fixed, deleted or blocked.
    Hestia {
        #[command(subcommand)]
        action: HestiaAction,
    },
    /// Measure the build: build the workspace cold in two separate git worktrees of HEAD, each with its own
    /// target directory, and commit the result as a build-measurement record: T (one cold target directory),
    /// S (one worktree's tracked files), the wall time of each build, and the sccache hit and miss counts of
    /// each build (or that sccache is not installed). It takes minutes and needs the disk for two builds. The
    /// Hestia sweep runs it when no record exists and when Cargo.lock changes; `athena status` shows the latest.
    MeasureBuild {
        #[arg(long)]
        json: bool,
    },
    /// Check a candidate `defaults.toml` against an open or accepted proposal and `defaults.toml` at HEAD:
    /// id, statement, version (base plus one) and parse. Exit 2 on a problem.
    CheckProposal {
        id: String,
        /// The candidate file to check.
        #[arg(long)]
        candidate: PathBuf,
    },
    /// Correct the latest reading in one line (or answer its question), and read again.
    Correct {
        job: String,
        #[arg(required = true, num_args = 1..)]
        line: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Set how soon a queued request is worked: urgent, high, normal or low. Appends a priority record; the
    /// request is not changed, and plan and status prefer the latest record over a `Priority:` line in its words.
    Priority {
        /// The queued request (a job id, or a unique prefix or suffix of it).
        job: String,
        /// One of: urgent, high, normal, low.
        level: String,
        #[arg(long)]
        json: bool,
    },
    /// Set what to work on now, or print the current focus. With text, appends a focus record (the latest one
    /// is the current focus) and prints it; with no text, prints the current focus. `athena status` shows it.
    Focus {
        /// What to focus on, in a few words. Leave it out to print the current focus.
        text: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Run a proposed job to its end, read a queued one, or retry the landing of a blocked one.
    Go {
        job: String,
        #[arg(long)]
        json: bool,
    },
    /// Push a verified job's branch, open a pull request, and wait for green checks. The merge is requested
    /// only when athena.toml sets `[landing] merge_via_rest = true`; run it again to see a merge.
    Land {
        job: String,
        /// Land through a pull request.
        #[arg(long, required = true)]
        pr: bool,
        #[arg(long)]
        json: bool,
    },
    /// Work the queue (the steward's requests and those deferred with `ask --later`) within a budget: for each queued request, read and run the job, land it
    /// through the pull-request path (as `athena land --pr` does), then rate the live page. Open proposals
    /// within their role's limit are approved and their jobs worked, as are jobs already approved. Proposals
    /// above the limit, or from an unknown role, are listed as waiting for a person; drain never rejects one.
    #[command(after_help = "\
Budget: before every model call (a reading, a run, the re-reading of a retry, a rating, the source check) the money spent so far, \
as the calls actually cost, plus the call's cap from athena.toml must fit --budget; otherwise nothing more starts.

Landing: only a merged, verified pull request counts as landed. Without `[landing] merge_via_rest = true` in \
athena.toml the pull request stays open and the item is not landed.

A rating counts toward the target only when it is taken after a landing in this run. A run that landed \
nothing says the page is unchanged and never reports `target met`.

`target met` needs two more things. (1) A landing in this run must have changed what the explainer is built \
from: a path under studies/, the site generator (xtask/) or the publish workflow; otherwise the report says \
the landing did not change the outcome. (2) All of the page's text (claims, the intro with its transition and \
glossary, and every other study file but the pin and the rule results) must pass a model-backed check against \
the pinned source, which fails on a contradicted or omitted statement and names it. That check is a model \
call per study: the verifier's cap times the number of studies must fit --budget, and its cost counts. Each \
study model's self-consistency check runs first, as a floor.

Stop reasons:
  target met        a rating taken after a landing reached --rating-threshold, a landing changed the outcome, \
and the page text passed the source check
  budget spent      spend plus the next call's cap would exceed --budget; the rest stays queued
  environment       an item failed twice because the producer's login expired or free \
disk space is low, so every later item would fail the same way; fix that and run the drain again. An item that \
fails twice for any other reason is passed over (\"failed twice; left for the scope guard and Hestia\") and the \
batch goes on to the next item
  usage limit       a job's reply said the weekly or session usage limit was reached; nothing more starts, and a \
note gives when the limit resets; run the drain again after that
  queue empty       nothing was left to work; this is not success
  job limit         --job-limit jobs were started
  stop requested    `athena drain-stop` was run; the job in progress was finished and no further job was started
  rebuilt           a landing changed a path in BUILD_PATHS (athena-core status.rs), so Athena was rebuilt \
and installed (the steps of ./setup.sh); the new binary carries on with the remaining --budget and --job-limit, \
so a run is not seen to end here
  rebuild failed    that rebuild failed; the build output is shown and the rest stays queued
  reinstall needed  a job landed and its athena.toml needs a newer Athena than the one running (a new key, for \
instance); the job is landed and not a failure; reinstall Athena (./setup.sh) and run the drain again

Order: planning comes first. Every open request is fleshed out (goal, requirements-ladder rung, acceptance, \
open questions, dependencies, the paths it touches) and all of them are planned together. Drain works the \
items in plan order (a dependent after what it needs, then priority urgent, high, normal or low from a \
`Priority:` line in the request's words or a later `athena priority <job> <level>` record, which wins, \
then higher rung, then age as the tiebreak; no priority means normal), not oldest \
first, and the report shows the plan's groups: items in one group have no dependency and touch disjoint paths, \
so they can run in parallel: drain runs up to `[parallel] max_concurrency` of them at once (athena.toml; \
default 1, which is one job at a time; at most {ceiling}). Landing and rating stay one at a time, and a group starts \
only when the group before it is done. Nothing is defaulted into a plan: acceptance \
comes only from explicit bullets in the request or a reading's criteria, and the rung from the proposal that made \
the job or from evidence in the request. The planning pass is deterministic code, not a model call. An unread \
request whose own words leave the plan incomplete is read first (within the budget) and planned again from its \
reading. An item whose plan still lacks a goal, a rung or acceptance is not worked and is reported with the \
reason. `athena plan` prints this plan without working anything.

Improvement turn: requests of Athena's own (the steward's, and Hestia's: the error review's request for a repeated \
cause, the throughput design request and the spec review's request) are guaranteed a turn. While one is ready, \
at most two other items run before it, however many requests a person marked urgent or high; it must have a \
complete plan and no unmet dependency, and `athena plan` prints a `turn` line for each. A request whose cause is \
recorded in three or more distinct jobs is raised to urgent for the plan (a `promoted` line), unless a person set \
its priority; the raise is made before `[priority_caps]` apply. The report has an `improvement` line for each turn taken.

One drain runs per repository. It writes its pid file (`drain.pid` in the state directory) when it starts and \
removes it when it ends, and a second `athena drain` while one is running is refused. It also refuses to start, \
and dispatches nothing, when this build is older than the landed code (the message says to run ./setup.sh). To see, stop or restart a drain use `athena drain-status`, `athena drain-stop --wait` \
and `athena drain-restart`, not `pkill` or `pgrep`.

Exit code: 0 only when the target was met; 2 for any other stop; 1 when Athena itself erred.")]
    Drain {
        /// The most to spend on model calls in this run, in US dollars.
        #[arg(long, value_name = "USD", value_parser = parse_budget)]
        budget: f64,
        /// The audience rating, 1 to 5, at or above which the target is met.
        #[arg(long, value_name = "N", default_value_t = drain::DEFAULT_TARGET_RATING, value_parser = clap::value_parser!(u8).range(1..=5))]
        rating_threshold: u8,
        /// The most jobs to start; a retry of an item is a job.
        #[arg(long, value_name = "N", default_value_t = drain::DEFAULT_MAX_JOBS, value_parser = clap::value_parser!(u32).range(1..))]
        job_limit: u32,
        #[arg(long)]
        json: bool,
    },
    /// Print the coordinated plan of the open requests without working any: the order, the parallel groups, and
    /// for each item its goal, requirements-ladder rung, acceptance, open questions, dependencies and touched
    /// paths, then the items held back, each with its reason. The planning pass is deterministic code, not a
    /// model call. It does read unread requests whose own words leave the plan incomplete (a reading is a model
    /// call, capped by `[budget] reader_usd` each and by --budget in all, with no run reserved); `--budget 0` reads
    /// nothing. A request it reads stays queued for `athena drain`. At most `[priority_caps]` items (default 3
    /// urgent, 10 high) keep their tier: the rest are demoted one tier for this plan only, each reported on a
    /// `note` line, and an item whose priority a person set with `athena priority` is never demoted. Requests of
    /// Athena's own (the steward's and Hestia's) get a guaranteed turn, at least one dispatch in three while one is
    /// ready: a `turn` line names each item that takes it, and a `promoted` line each request raised to urgent
    /// because its cause is recorded in three or more distinct jobs.
    Plan {
        /// The most to spend reading unread requests, in US dollars.
        #[arg(long, value_name = "USD", default_value_t = 2.0, value_parser = parse_read_budget)]
        budget: f64,
        #[arg(long)]
        json: bool,
    },
    /// Ask a running `athena drain` to stop at the next job boundary: the job in progress is finished (landed and
    /// rated as usual), no further job is started, and the drain reports `stop requested`. Nothing is killed;
    /// to kill running work use `athena stop`. With `--wait` it then waits until the drain has exited, found
    /// through the pid file `athena drain` keeps in the state directory, so use it instead of `pkill`, `pgrep`
    /// or a hand-written wait loop: those match command text, which their own command line contains.
    DrainStop {
        /// Wait until the drain has exited (it finishes its job first, which can take hours). Prints a progress
        /// line every 30 seconds. Exit code 2 if `--timeout` ran out first.
        #[arg(long)]
        wait: bool,
        /// With `--wait`, give up waiting after this many seconds; the stop stays asked for.
        #[arg(long, value_name = "SECS", requires = "wait", value_parser = clap::value_parser!(u64).range(1..))]
        timeout: Option<u64>,
    },
    /// Say whether an `athena drain` is running, and as which build, from the pid file in the state directory and
    /// nothing else: the process is checked by pid and start time, never by matching command text. Use it
    /// instead of `pgrep`, `pkill` or `ps | grep`, whose pattern matches their own command line. Reports why
    /// no drain counts as running (no pid file, a process that has exited, a pid another process now holds).
    /// Exit code: 0 when a drain is running, 2 when none is.
    DrainStatus {
        #[arg(long)]
        json: bool,
    },
    /// Restart the drain on a fresh build, in one command, instead of a hand-written `pkill`, wait and start:
    /// rebuild and install Athena (`./setup.sh`; a failed build leaves the running drain alone and starts
    /// nothing), ask the running drain to stop and wait for it to finish its job and exit, build again if a
    /// landing meanwhile left the installed binary behind the checkout, then start the new drain and print the
    /// build it runs. The old drain is found through the pid file, never by `pkill` or `pgrep`. The new drain
    /// runs in the background, in its own process group, and logs to `drain.log` in the state directory. It
    /// takes the same limits as `athena drain`; a stopped drain does not keep its remaining ones. A drain
    /// started by a build from before the pid file existed cannot be seen: run `athena drain-stop` first and
    /// let it finish. Exit code: 0 when the new drain is running, 2 when the old drain was still on its job
    /// at `--timeout` (nothing new was started), 1 on an error.
    DrainRestart {
        /// The most to spend on model calls in this run, in US dollars.
        #[arg(long, value_name = "USD", value_parser = parse_budget)]
        budget: f64,
        /// The audience rating, 1 to 5, at or above which the target is met.
        #[arg(long, value_name = "N", default_value_t = drain::DEFAULT_TARGET_RATING, value_parser = clap::value_parser!(u8).range(1..=5))]
        rating_threshold: u8,
        /// The most jobs to start; a retry of an item is a job.
        #[arg(long, value_name = "N", default_value_t = drain::DEFAULT_MAX_JOBS, value_parser = clap::value_parser!(u32).range(1..))]
        job_limit: u32,
        /// Give up waiting for the old drain to exit after this many seconds (by default there is no limit: a
        /// job can run for hours). The stop stays asked for and no new drain is started.
        #[arg(long, value_name = "SECS", value_parser = clap::value_parser!(u64).range(1..))]
        timeout: Option<u64>,
        /// The new drain prints its report as JSON, as `athena drain --json` does.
        #[arg(long)]
        json: bool,
    },
    /// Close a job that will not run or finish.
    Close {
        job: String,
        #[arg(long)]
        why: String,
        /// Let invocation records already committed that fail "usage records reconcile" (written
        /// before a usage fix) through the closing commit; they are named and left unmodified.
        #[arg(long)]
        allow_legacy_usage: bool,
        #[arg(long)]
        json: bool,
    },
    /// The state of the work, or one job's full trace.
    Status {
        job: Option<String>,
        #[arg(long)]
        json: bool,
        /// Read the records and history from an evidence snapshot directory (written at dispatch under a
        /// job's state directory) instead of this repository. Needs no repository and no records/ here.
        #[arg(long, value_name = "DIR", conflicts_with = "job")]
        snapshot: Option<PathBuf>,
    },
    /// Record an intervention: anything you did beyond what a lazy user would.
    Intervene {
        /// What happened, in one or two sentences.
        what: String,
        #[arg(long)]
        why: String,
        /// The class of work, from athena.toml's closed list.
        #[arg(long)]
        class: String,
        /// did-work-themselves, corrected-output, gave-detailed-instructions, overrode-decision,
        /// answered-question, carried-state, fixed-environment, mistake, or other.
        #[arg(long)]
        category: String,
        #[arg(long)]
        job: Option<String>,
        /// Commit your staged hand changes together with the record.
        #[arg(long)]
        with_staged: bool,
        #[arg(long)]
        json: bool,
    },
    /// Record a restart of your own context.
    Restart {
        /// What it took to get productive again.
        #[arg(long)]
        cost: String,
        #[arg(long)]
        minutes: Option<f64>,
        #[arg(long)]
        tokens: Option<f64>,
        /// The minutes and tokens were measured, not recalled.
        #[arg(long)]
        measured: bool,
        /// Anything that was lost (repeatable).
        #[arg(long)]
        lost: Vec<String>,
        /// What the new context read to resume (repeatable).
        #[arg(long)]
        via: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Check every record, reconciliation, and gate over the working state, or over the full history.
    ///
    /// With `--pr-head <BASE> <HEAD>`, the pre-merge check on a pull request: every commit in BASE..HEAD must
    /// be a verified job landing (prediction before outcome), a declared intervention before the last
    /// landing, or records only, and the head must hold the last landing's verified content. Exit 2 on a
    /// finding.
    ///
    /// With `--attribution`, the records-only check on main: every first-parent commit from the earliest
    /// landing-intent's expected base onward must be named by a matching landing record (a commit with a
    /// matching intent and no landing is pending for 15 minutes). It reads `athena/records` (fetch it first)
    /// and never a commit message. Exit 2 on a finding. It judges only the commits and records added since
    /// the newest checkpoint on `athena/records` (a full replay when there is none or its check version
    /// differs), and on a pass on the main branch pushes a new checkpoint to the `athena/records` branch on
    /// `origin`, so CI starts from it. The records it judged must already be on `origin`; otherwise no
    /// checkpoint is written and the reason is printed. It writes nothing when CI is set in the environment,
    /// with `--no-checkpoint`, or off the main branch.
    /// It prints the seconds taken and the commits and records judged.
    ///
    /// With `--host-rules`, the read-only check of the host rules: it reads the rulesets view of the GitHub
    /// origin (GET requests only) and reports, for `athena/records`, whether force pushes and deletion are
    /// forbidden and, for `main`, whether branches must be up to date before merging. Exit 2 when a required
    /// rule is not in force or cannot be read.
    Check {
        #[arg(long)]
        history: bool,
        #[arg(long, num_args = 2, value_names = ["BASE", "HEAD"], conflicts_with = "history")]
        pr_head: Option<Vec<String>>,
        #[arg(long, conflicts_with_all = ["history", "pr_head"])]
        attribution: bool,
        /// Read-only check that the host rules on `athena/records` and `main` are in force.
        #[arg(long, conflicts_with_all = ["history", "pr_head", "attribution"])]
        host_rules: bool,
        /// With `--attribution`: ignore checkpoints, judge everything from the start, and fail if the newest
        /// checkpoint names a commit the replay fails.
        #[arg(long, requires = "attribution")]
        full_replay: bool,
        /// With `--attribution`: never write a checkpoint.
        #[arg(long, requires = "attribution")]
        no_checkpoint: bool,
        #[arg(long)]
        json: bool,
    },
    /// Read-only backtest of decisions D87 and D88 over `docs/decisions.md`, its confirmed link list `docs/decision-links.toml` and `records/`; exit 2 on a finding.
    Backtest {
        #[arg(long)]
        json: bool,
    },
    /// Read-only: coverage per goal (rows, covered, partial, not covered, and the not-covered rows) computed
    /// from `docs/design/requirements-map.md` and the goal lists of `docs/design/strategy-view.md`. Exit 2
    /// when the two disagree on which rows exist.
    Strategy {
        #[arg(long)]
        json: bool,
    },
    /// Read-only: apply the reading rules of this build to a stored request and reading and print `accept`, or
    /// `refuse: <reason>`. Needs no repository and calls no model. Exit 0 on accept, 2 on refuse. Athena runs
    /// the base and the candidate builds of this command over its newest accepted readings before it verifies
    /// a candidate.
    CheckReading {
        /// The request: its `request.json` record, or the request text in a plain file.
        #[arg(long)]
        request: PathBuf,
        /// The reading: its `reading-<n>.json` record.
        #[arg(long)]
        reading: PathBuf,
    },
    /// The kill switch: terminate every running agent and check, and block new ones.
    Stop,
    /// Turn the kill switch off.
    Start,
    /// Entry points for hooks.
    #[command(hide = true)]
    Hook {
        #[command(subcommand)]
        hook: HookCommand,
    },
}

#[derive(Subcommand, Debug)]
enum HestiaAction {
    /// Run every registered check, record the report, and print it. The skeleton registers no checks yet, and
    /// then the report says so.
    Sweep {
        #[arg(long)]
        json: bool,
    },
    /// Print the last recorded sweep: each check with its count, and each finding. Says so when none is recorded.
    Last {
        #[arg(long)]
        json: bool,
    },
    /// Run the error review over the backlog, not only on a new introspection. With --backfill, once and safe to
    /// repeat: a repeated cause in a stage that athena.toml switches off is skipped as dormant; an open request
    /// that addresses a cause is queued again through the steward request builder and the old one closed; any
    /// other cause gets a new root-cause request. The five highest-ranked causes (most occurrences, then most
    /// recent) are urgent, the rest normal. Prints what was queued, re-queued and skipped, and why.
    Review {
        /// Sweep the repeated causes already in the records.
        #[arg(long, required = true)]
        backfill: bool,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug)]
enum HookCommand {
    /// Claude Code's PreToolUse hook.
    PreToolUse {
        #[arg(long)]
        policy: PathBuf,
    },
}

/// Progress lines on standard error, so `--json` output stays clean on standard output.
struct Terminal;

impl Ui for Terminal {
    fn line(&self, text: &str) {
        eprintln!("{text}");
    }
}

/// The text of `after_help` with the `{ceiling}` placeholder replaced by `ceiling`. A limit that help text
/// states comes from the constant that enforces it, never from a number typed into the text, so a change to the
/// constant cannot leave the help stale (decision D156).
fn drain_after_help(template: &str, ceiling: u32) -> String {
    template.replace("{ceiling}", &ceiling.to_string())
}

/// The command line as `main` runs it: the derived command with the `athena drain` help filled in from
/// `[parallel] max_concurrency`'s ceiling, [`athena_data::config::MAX_CONCURRENCY_CEILING`]. Anything that
/// renders the help builds the command here, not with `Cli::command()`, which leaves the placeholder unfilled.
fn cli_command() -> clap::Command {
    Cli::command().mut_subcommand("drain", |drain| {
        let template = drain
            .get_after_help()
            .map(ToString::to_string)
            .unwrap_or_default();
        drain.after_help(drain_after_help(
            &template,
            athena_data::config::MAX_CONCURRENCY_CEILING,
        ))
    })
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let name = args
        .first()
        .map(|a| a.rsplit('/').next().unwrap_or(a).to_string())
        .unwrap_or_default();
    if name == "pre-commit" || name == "commit-msg" {
        return ExitCode::from(hooks::git_hook(&name, &args[1..]) as u8);
    }
    // The producer's `cargo`: this binary linked as `cargo` takes a build slot, then runs the real one.
    if name == "cargo" {
        return ExitCode::from(slot_exec::run(&args[1..], &|key| std::env::var(key).ok()));
    }
    let version: &'static str =
        Box::leak(wiring::version_string(&wiring::build_info()).into_boxed_str());
    let matches = cli_command().version(version).get_matches();
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(error) => error.exit(),
    };
    let result = run(cli);
    // A background role (the steward) started by this command finishes before the process ends. The
    // command's own work is done by now, so nothing waited on it.
    athena_core::steward::wait_for_background();
    match result {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(1)
        }
    }
}

/// `athena check --host-rules`: reads the rulesets view of the repository `origin_url` names through `runner`
/// (GET only) and returns the text to print and the exit code (0 when every required rule is in force, else 2).
fn host_rules_command(
    runner: &dyn athena_core::gh::Runner,
    origin_url: &str,
    json: bool,
) -> Result<(String, u8)> {
    let results = athena_core::host_rules::check_origin(runner, origin_url)?;
    let missing = athena_core::host_rules::missing_required(&results);
    let code = if missing.is_empty() { 0 } else { 2 };
    let text = if json {
        let branches: Vec<_> = results
            .iter()
            .map(|r| serde_json::json!({ "branch": r.branch, "states": format!("{:?}", r.states) }))
            .collect();
        format!(
            "{}\n",
            serde_json::json!({ "branches": branches, "passed": missing.is_empty(), "missing": missing })
        )
    } else {
        display::host_rules_report(&results)
    };
    Ok((text, code))
}

/// `athena check-reading`: applies this build's reading rules (`athena_core::reading::check_reading_text`) to a
/// stored request and reading. Returns the line to print (`accept`, or `refuse: <reason>`) and the exit code (0,
/// or 2 on a refusal). A file that cannot be read, or a reading that is not a reading record, is an error and
/// never an `accept`.
fn check_reading_command(
    request: &std::path::Path,
    reading: &std::path::Path,
) -> Result<(String, u8)> {
    use anyhow::Context as _;
    let request_bytes =
        std::fs::read(request).with_context(|| format!("reading {}", request.display()))?;
    let request_text = match athena_data::records::AnyRecord::parse(
        &request.display().to_string(),
        &request_bytes,
    ) {
        Ok(athena_data::records::AnyRecord::Request(r)) => r.text,
        _ => String::from_utf8(request_bytes)
            .with_context(|| format!("{} is not text", request.display()))?,
    };
    let reading_bytes =
        std::fs::read(reading).with_context(|| format!("reading {}", reading.display()))?;
    let record =
        athena_data::records::AnyRecord::parse(&reading.display().to_string(), &reading_bytes)?;
    let athena_data::records::AnyRecord::Reading(parsed) = record else {
        anyhow::bail!("{} is not an athena.reading/1 record", reading.display());
    };
    let check = athena_core::reading::check_reading_text(&parsed, &request_text);
    let code = match check {
        athena_core::reading::ReadingCheck::Accept => 0,
        athena_core::reading::ReadingCheck::Refuse(_) => 2,
    };
    Ok((check.line(), code))
}

/// A budget in US dollars: a number above zero.
fn parse_budget(text: &str) -> Result<f64, String> {
    match text.trim().trim_start_matches('$').parse::<f64>() {
        Ok(value) if value.is_finite() && value > 0.0 => Ok(value),
        _ => Err(format!("{text:?} is not an amount above zero")),
    }
}

/// Like `parse_budget`, but zero is allowed: it means "read nothing".
fn parse_read_budget(text: &str) -> Result<f64, String> {
    match text.trim().trim_start_matches('$').parse::<f64>() {
        Ok(value) if value.is_finite() && value >= 0.0 => Ok(value),
        _ => Err(format!("{text:?} is not an amount of zero or more")),
    }
}

/// Prints one line on standard error when this binary's build commit is older than `HEAD` for the paths it is
/// built from (see `Athena::stale_build`). For the commands that run agents.
fn warn_if_stale(athena: &Athena) {
    if let Some(line) = athena.stale_build() {
        eprintln!("{line}");
    }
}

/// Replaces this process with the installed Athena run with `args`. It returns only when that failed. A
/// background role this command started finishes first, as it does when the command ends.
#[cfg(unix)]
fn reexec<T>(args: &[String]) -> Result<T> {
    use std::os::unix::process::CommandExt;
    athena_core::steward::wait_for_background();
    let binary = wiring::installed_binary();
    let error = std::process::Command::new(&binary).args(args).exec();
    Err(anyhow::anyhow!(
        "athena was rebuilt but {} could not be run to carry on: {error}; run `athena {}` yourself",
        binary.display(),
        args.join(" ")
    ))
}

fn open(writes: bool) -> Result<Athena> {
    wiring::open(&std::env::current_dir()?, writes, &|line| {
        eprintln!("{line}")
    })
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

/// `athena drain-status --json`: whether a drain runs, which one, and why not when none does.
fn drain_status_json(state: &drain::DrainState, stop_asked: bool) -> serde_json::Value {
    match state {
        drain::DrainState::Running(drain) => serde_json::json!({
            "running": true,
            "pid": drain.pid,
            "build": drain.build,
            "started_at": drain.at,
            "stop_asked": stop_asked,
        }),
        drain::DrainState::NotRunning(why) => serde_json::json!({
            "running": false,
            "reason": why.reason(),
            "stop_asked": stop_asked,
        }),
    }
}

fn proposal_json(p: &Proposal) -> serde_json::Value {
    serde_json::json!({
        "job": p.job.as_str(),
        "commit": p.commit,
        "reading": p.reading,
        "reader_error": p.reader_error,
        "spent_usd": p.spent_usd,
    })
}

/// Whether the environment says this is CI (`CI` set and not empty). CI's token is read-only, and the code
/// agrees: a CI run never writes a checkpoint.
fn ci_is_set() -> bool {
    std::env::var_os("CI").is_some_and(|v| !v.is_empty())
}

/// Why `check --attribution` must not write a checkpoint, or `None` when it may.
fn no_write_reason(no_checkpoint: bool, ci: bool, branch: Option<&str>) -> Option<&'static str> {
    if ci {
        Some("CI is read-only")
    } else if no_checkpoint {
        Some("--no-checkpoint")
    } else if branch != Some("main") {
        Some("HEAD is not on the main branch")
    } else {
        None
    }
}

/// The text of a run: the report, then the mode, the checkpoint and the seconds and counts.
fn run_text(run: &athena_gate::attribution::RunReport, no_write: Option<&str>) -> String {
    use std::fmt::Write as _;
    let mut out = display::attribution_report(&run.report);
    let _ = writeln!(out, "mode: {}", mode_text(run));
    out.push_str(&checkpoint_line(run, no_write));
    let _ = writeln!(
        out,
        "took {:.3} s; judged {} commit(s) and {} record(s)",
        run.seconds,
        run.commits_judged(),
        run.records_judged()
    );
    out
}

fn mode_text(run: &athena_gate::attribution::RunReport) -> String {
    match (&run.started_from, &run.fallback_reason) {
        (Some(cp), _) => format!(
            "incremental, from the checkpoint at main commit {} (check version {})",
            cp.main_sha, cp.verifier_version
        ),
        (None, Some(why)) => format!("full replay, because {why}"),
        (None, None) => run.mode.label().to_string(),
    }
}

fn checkpoint_line(run: &athena_gate::attribution::RunReport, no_write: Option<&str>) -> String {
    match (&run.checkpoint_written, &run.checkpoint_error, no_write) {
        (Some(cp), _, _) => format!(
            "checkpoint: written {} (main {}, records {}, check version {})\n",
            cp.id, cp.main_sha, cp.records_sha, cp.verifier_version
        ),
        (None, Some(error), _) => format!("checkpoint not written: {error}\n"),
        (None, None, Some(why)) => format!("checkpoint: not written ({why})\n"),
        (None, None, None) => {
            "checkpoint: not written (the run did not pass cleanly or judged nothing new)\n"
                .to_string()
        }
    }
}

/// The run as JSON: the attribution report's keys plus the mode, seconds and counts.
fn run_json(
    run: &athena_gate::attribution::RunReport,
    no_write: Option<&str>,
) -> serde_json::Value {
    let mut value = display::attribution_json(&run.report);
    if let Some(map) = value.as_object_mut() {
        map.insert("mode".into(), run.mode.label().into());
        map.insert("requested_mode".into(), run.requested.label().into());
        map.insert("fallback_reason".into(), run.fallback_reason.clone().into());
        map.insert("seconds".into(), run.seconds.into());
        map.insert("commits_judged".into(), run.commits_judged().into());
        map.insert("records_judged".into(), run.records_judged().into());
        map.insert(
            "checkpoint_written".into(),
            run.checkpoint_written
                .as_ref()
                .map(|cp| cp.id.to_string())
                .into(),
        );
        map.insert(
            "checkpoint_error".into(),
            run.checkpoint_error.clone().into(),
        );
        map.insert("read_only_reason".into(), no_write.into());
    }
    value
}

/// After a landing record: runs the attribution check incrementally on `landed` (the merged commit) with
/// checkpoint writing on. A pass pushes a checkpoint to the `athena/records` branch on `origin` (the remote
/// landing publishes its records to), so CI's incremental run can start from it; the verdict never changes
/// the landing's own outcome.
fn checkpoint_after_landing(
    git: &athena_git::Git,
    landed: &str,
    now: jiff::Timestamp,
) -> Result<athena_gate::attribution::RunReport> {
    use athena_gate::attribution::{Mode, Options, run_check};
    Ok(run_check(
        git,
        landed,
        now,
        &Options::new(Mode::Incremental, true),
    )?)
}

/// After a landing record: the reinstall note when this build cannot read the `athena.toml` of the merged
/// commit `merged`, else None. It reads the merged commit and not head, because head may predate the merge.
/// The job is landed either way; the note only tells the user the running build is older than what it landed.
fn reinstall_note_after_landing(git: &athena_git::Git, merged: &str) -> Option<String> {
    athena_core::capability::JobConfig::unreadable_at(git, merged)
        .map(|why| drain::reinstall_note(&why))
}

/// One note for `athena land` about the checkpoint after a landing.
fn checkpoint_note(run: &athena_gate::attribution::RunReport) -> String {
    match (&run.checkpoint_written, &run.checkpoint_error) {
        (Some(cp), _) => format!(
            "checkpoint written: main {}, records {}, check version {} ({:.3} s, {} commit(s), {} record(s) judged)\n",
            cp.main_sha,
            cp.records_sha,
            cp.verifier_version,
            run.seconds,
            run.commits_judged(),
            run.records_judged()
        ),
        (None, Some(error)) => format!("checkpoint not written: {error}\n"),
        (None, None) => format!(
            "checkpoint not written: the attribution check found {} problem(s) or a pending commit\n",
            run.report.findings.len()
        ),
    }
}

fn finished_code(state: OutcomeState) -> u8 {
    if state == OutcomeState::Landed { 0 } else { 2 }
}

/// 0 when the job landed or its pull request is open with green checks; 2 otherwise.
fn land_pr_code(report: &athena_core::gh::PrReport) -> u8 {
    use athena_core::gh::Checks;
    use athena_core::pr::LinkState;
    match (report.link.state, report.checks) {
        (LinkState::Landed, _) | (LinkState::Open, Some(Checks::Green)) => 0,
        _ => 2,
    }
}

fn go_and_report(athena: &Athena, job: &str, accepted_by: AcceptedBy, json: bool) -> Result<u8> {
    match athena.go(job, accepted_by, &Terminal)? {
        GoResult::Reread(p) => {
            if json {
                print_json(&proposal_json(&p))?;
            } else if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
                return prompt_loop(athena, *p);
            } else {
                if let Some(r) = &p.reading {
                    print!("{}", display::reading(p.job.as_str(), r, false));
                }
                print!("{}", display::follow_up(&p));
            }
            Ok(2)
        }
        GoResult::Finished(f) => {
            if json {
                print_json(&serde_json::json!({
                    "job": f.job.as_str(),
                    "state": f.state.as_str(),
                    "reason": f.reason,
                    "prediction_commit": f.prediction_commit,
                    "closing_commit": f.closing_commit,
                    "landed_commit": f.landed_commit,
                    "totals": f.totals,
                    "score": f.score,
                }))?;
            } else {
                print!("{}", display::finished(&f));
            }
            Ok(finished_code(f.state))
        }
    }
}

/// The interactive prompt after a reading: Enter runs it, `q` leaves it, anything else corrects it.
fn prompt_loop(athena: &Athena, mut proposal: Proposal) -> Result<u8> {
    let stdin = std::io::stdin();
    loop {
        let Some(reading) = proposal.reading.clone() else {
            print!("{}", display::follow_up(&proposal));
            return Ok(2);
        };
        print!(
            "{}",
            display::reading(proposal.job.as_str(), &reading, true)
        );
        print!("> ");
        std::io::stdout().flush()?;
        let mut line = String::new();
        if stdin.lock().read_line(&mut line)? == 0 {
            println!();
            print!("{}", display::follow_up(&proposal));
            return Ok(0);
        }
        let line = line.trim();
        if line == "q" || (line.is_empty() && reading.capability.is_none()) {
            print!("{}", display::follow_up(&proposal));
            return Ok(0);
        }
        if line.is_empty() {
            return go_and_report(athena, proposal.job.as_str(), AcceptedBy::User, false);
        }
        proposal = athena.correct(proposal.job.as_str(), line, &Terminal)?;
    }
}

/// Shows a fresh reading as `ask` does: with `--yes` it is printed and run without waiting; with
/// `--json` it is printed as data; on a terminal Enter accepts it. Nothing runs unless the user accepts.
fn show_proposal(athena: &Athena, proposal: Proposal, yes: bool, json: bool) -> Result<u8> {
    if yes {
        if let Some(r) = &proposal.reading
            && !json
        {
            print!("{}", display::reading(proposal.job.as_str(), r, false));
        }
        if proposal
            .reading
            .as_ref()
            .is_some_and(|r| r.capability.is_some())
        {
            return go_and_report(athena, proposal.job.as_str(), AcceptedBy::Flag, json);
        }
        if json {
            print_json(&proposal_json(&proposal))?;
        } else {
            print!("{}", display::follow_up(&proposal));
        }
        return Ok(2);
    }
    if json {
        print_json(&proposal_json(&proposal))?;
        return Ok(0);
    }
    if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
        return prompt_loop(athena, proposal);
    }
    if let Some(r) = &proposal.reading {
        print!("{}", display::reading(proposal.job.as_str(), r, false));
    }
    print!("{}", display::follow_up(&proposal));
    Ok(0)
}

fn run(cli: Cli) -> Result<u8> {
    match cli.command {
        Command::Ask {
            request,
            yes,
            later,
            json,
        } => commands::ask(request, yes, later, json),
        Command::Retry { job, yes, json } => commands::retry(job, yes, json),
        Command::Propose { json } => commands::propose(json),
        Command::ScoreJev { cut, derive, json } => commands::score_jev(cut, derive, json),
        Command::Review {
            id,
            right,
            class,
            new_class,
            description,
            note,
            json,
        } => commands::review(id, right, class, new_class, description, note, json),
        Command::AcceptProposal { id, yes, json } => {
            let athena = open(true)?;
            let proposal = athena.accept_proposal(&id, &Terminal)?;
            show_proposal(&athena, proposal, yes, json)
        }
        Command::RejectProposal { id, why, json } => commands::reject_proposal(id, why, json),
        Command::ApproveWork { id, yes, json } => {
            let athena = open(true)?;
            let proposal = athena.approve_work(&id, &Terminal)?;
            show_proposal(&athena, proposal, yes, json)
        }
        Command::RejectWork { id, why, json } => commands::reject_work(id, why, json),
        Command::Audience { url, reader, json } => commands::audience(url, reader, json),
        Command::Audit { job, json } => commands::audit(job, json),
        Command::Hestia { action } => commands::hestia(action),
        Command::MeasureBuild { json } => commands::measure_build(json),
        Command::CheckProposal { id, candidate } => commands::check_proposal(id, candidate),
        Command::Correct { job, line, json } => commands::correct(job, line, json),
        Command::Priority { job, level, json } => commands::priority(job, level, json),
        Command::Focus { text, json } => commands::focus(text, json),
        Command::Go { job, json } => commands::go(job, json),
        Command::Land { job, pr: _, json } => commands::land(job, json),
        Command::Drain {
            budget,
            rating_threshold,
            job_limit,
            json,
        } => commands::drain(budget, rating_threshold, job_limit, json),
        Command::Plan { budget, json } => commands::plan(budget, json),
        Command::DrainStop { wait, timeout } => commands::drain_stop(wait, timeout),
        Command::DrainStatus { json } => commands::drain_status(json),
        Command::DrainRestart {
            budget,
            rating_threshold,
            job_limit,
            timeout,
            json,
        } => commands::drain_restart(budget, rating_threshold, job_limit, timeout, json),
        Command::Close {
            job,
            why,
            allow_legacy_usage,
            json,
        } => commands::close(job, why, allow_legacy_usage, json),
        Command::Status {
            snapshot: Some(dir),
            json,
            ..
        } => commands::status_snapshot(dir, json),
        Command::Status { job, json, .. } => commands::status(job, json),
        Command::Intervene {
            what,
            why,
            class,
            category,
            job,
            with_staged,
            json,
        } => commands::intervene(
            athena_core::manage::InterveneArgs {
                what,
                why,
                class,
                category,
                job,
                with_staged,
            },
            json,
        ),
        Command::Restart {
            cost,
            minutes,
            tokens,
            measured,
            lost,
            via,
            json,
        } => commands::restart(
            athena_core::manage::RestartArgs {
                cost,
                minutes,
                tokens,
                measured,
                lost,
                via,
            },
            json,
        ),
        Command::Check {
            history,
            pr_head,
            attribution,
            host_rules,
            full_replay,
            no_checkpoint,
            json,
        } => commands::check(commands::CheckOptions {
            history,
            pr_head,
            attribution,
            host_rules,
            full_replay,
            no_checkpoint,
            json,
        }),
        Command::Backtest { json } => commands::backtest(json),
        Command::Strategy { json } => commands::strategy(json),
        Command::CheckReading { request, reading } => commands::check_reading(request, reading),
        Command::Stop => commands::stop(),
        Command::Start => commands::start(),
        Command::Hook {
            hook: HookCommand::PreToolUse { policy },
        } => Ok(hooks::pre_tool_use(&policy) as u8),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::JobId;
    use athena_data::RecordId;
    use athena_data::history::{
        HistoryRecord, Landing, LandingCommit, LandingIntent, LandingKind, MergeMethod,
    };
    use athena_git::Git;
    use athena_git::records_branch::{Change, RECORDS_REF, RecordsWriter};

    fn put(git: &Git, dir: &str, id: &RecordId, record: HistoryRecord) {
        RecordsWriter::new(git)
            .append(
                "record",
                &[Change::put(
                    format!("{dir}/{id}.json"),
                    serde_json::to_string(&record).unwrap(),
                )],
            )
            .unwrap();
    }

    /// A stored reading with this one acceptance criterion, as `reading-1.json`'s content.
    fn reading_json(criterion: &str) -> String {
        use athena_data::records::{
            AnyRecord, Criterion, CriterionKind, Deliverable, DeliverableAction, Forecast,
            Forecasts, Quantiles, Reading,
        };
        let forecast = Forecast {
            p_reading_accepted: 0.8,
            p_verification_passes: 0.7,
            p_user_intervenes: 0.1,
            cost_usd: Quantiles::new(0.2, 0.5, 1.5).unwrap(),
            wall_time_s: Quantiles::new(100.0, 300.0, 900.0).unwrap(),
        };
        AnyRecord::Reading(Box::new(Reading {
            job: JobId::parse("20261027-054151-ace0").unwrap(),
            n: 1,
            at: jiff::Timestamp::UNIX_EPOCH,
            invocation_seq: 1,
            base_commit: "a".repeat(40),
            config_sha256: "b".repeat(64),
            defaults_sha256: "c".repeat(64),
            defaults_version: 1,
            title: "Add a greeting".into(),
            summary: "Add a greeting.".into(),
            class_of_work: "feature".into(),
            capability: Some("change".into()),
            decline_reason: None,
            producer_model: "producer".into(),
            verifier_model: "verifier".into(),
            defaults_applied: vec![],
            assumptions: vec![],
            question: None,
            acceptance_criteria: vec![Criterion {
                text: criterion.into(),
                kind: CriterionKind::Behavior,
            }],
            deliverables: vec![Deliverable {
                path: "src/greeting.rs".into(),
                action: DeliverableAction::Modify,
                p: 0.9,
            }],
            out_of_scope: vec!["the changelog".into()],
            forecasts: Forecasts {
                reader: forecast.clone(),
                reference: forecast,
            },
            departures: None,
            urgent_reason: None,
            hard_reason: None,
            producer_effort: None,
        }))
        .to_json()
    }

    /// Writes a request and a reading to a scratch directory and returns the two paths.
    fn check_reading_fixture(
        dir: &std::path::Path,
        criterion: &str,
    ) -> (std::path::PathBuf, std::path::PathBuf) {
        let request = dir.join("request.txt");
        let reading = dir.join("reading-1.json");
        std::fs::write(&request, "add a greeting").unwrap();
        std::fs::write(&reading, reading_json(criterion)).unwrap();
        (request, reading)
    }

    #[test]
    fn check_reading_prints_accept_for_a_reading_the_rules_accept() {
        let dir = tempfile::tempdir().unwrap();
        let (request, reading) = check_reading_fixture(dir.path(), "cargo test --locked passes");
        assert_eq!(
            check_reading_command(&request, &reading).unwrap(),
            ("accept".to_string(), 0)
        );
    }

    #[test]
    fn check_reading_prints_refuse_and_the_reason_for_a_criterion_on_the_producers_report() {
        let dir = tempfile::tempdir().unwrap();
        let bad = "the producer's report lists the caller-check Grep";
        let (request, reading) = check_reading_fixture(dir.path(), bad);
        let (line, code) = check_reading_command(&request, &reading).unwrap();
        assert_eq!(code, 2);
        assert!(line.starts_with("refuse: "), "{line}");
        assert!(line.contains("criterion 1"), "{line}");
        assert!(line.contains(bad), "{line}");
        assert!(!line.contains('\n'), "{line}");
        // The request may also be its stored record; the answer is the same.
        let record = athena_data::records::AnyRecord::Request(athena_data::records::Request {
            job: JobId::parse("20261027-054151-ace0").unwrap(),
            text: "add a greeting".into(),
            received_at: jiff::Timestamp::UNIX_EPOCH,
            channel: "cli".into(),
        });
        std::fs::write(&request, record.to_json()).unwrap();
        assert_eq!(
            check_reading_command(&request, &reading).unwrap(),
            (line, 2)
        );
    }

    #[test]
    fn check_reading_applies_the_governance_rule_to_the_request_text() {
        // The request names athena.toml and the reading does not say whether the job changes it.
        let dir = tempfile::tempdir().unwrap();
        let (request, reading) = check_reading_fixture(dir.path(), "cargo test --locked passes");
        std::fs::write(&request, "add a greeting and change athena.toml").unwrap();
        let (line, code) = check_reading_command(&request, &reading).unwrap();
        assert_eq!(code, 2, "{line}");
        assert!(line.contains("athena.toml"), "{line}");
    }

    #[test]
    fn check_reading_with_an_unreadable_or_wrong_file_is_an_error_not_an_accept() {
        let dir = tempfile::tempdir().unwrap();
        let (request, reading) = check_reading_fixture(dir.path(), "cargo test --locked passes");
        let missing = dir.path().join("none.json");
        assert!(check_reading_command(&request, &missing).is_err());
        assert!(check_reading_command(&missing, &reading).is_err());
        // A request record is not a reading.
        let not_reading = dir.path().join("notes.json");
        std::fs::write(&not_reading, "{ \"schema\": \"nope\" }").unwrap();
        assert!(check_reading_command(&request, &not_reading).is_err());
    }

    struct FixtureRunner {
        replies: std::cell::RefCell<std::collections::VecDeque<&'static str>>,
        calls: std::cell::RefCell<Vec<Vec<String>>>,
    }

    impl FixtureRunner {
        fn new(replies: &[&'static str]) -> Self {
            FixtureRunner {
                replies: std::cell::RefCell::new(replies.iter().copied().collect()),
                calls: Default::default(),
            }
        }
    }

    impl athena_core::gh::Runner for FixtureRunner {
        fn gh(&self, args: &[String]) -> Result<String> {
            self.calls.borrow_mut().push(args.to_vec());
            Ok(self.replies.borrow_mut().pop_front().unwrap().to_string())
        }
    }

    const HOST_RULES_RECORDS: &str = r#"[{"type":"non_fast_forward"},{"type":"deletion"}]"#;
    const HOST_RULES_MAIN: &str = r#"[{"type":"required_status_checks","parameters":{"strict_required_status_checks_policy":true}}]"#;
    const HOST_RULES_URL: &str = "https://github.com/o/r.git";

    #[test]
    fn host_rules_in_force_exits_0_with_the_report() {
        let runner = FixtureRunner::new(&[HOST_RULES_RECORDS, HOST_RULES_MAIN]);
        let (text, code) = host_rules_command(&runner, HOST_RULES_URL, false).unwrap();
        assert_eq!(code, 0);
        let expected = "\
athena/records: force-push forbidden: in force
athena/records: deletion forbidden: in force
athena/records: up-to-date required: not in force
main: force-push forbidden: not in force
main: deletion forbidden: not in force
main: up-to-date required: in force
Only rulesets are seen; classic branch protection is not visible to this check, so set these rules as rulesets.
host rules: all required rules are in force
";
        assert_eq!(text, expected);
        for call in runner.calls.borrow().iter() {
            assert_eq!(call.len(), 2, "GET only, no flags: {call:?}");
            assert_eq!(call[0], "api");
        }
    }

    #[test]
    fn host_rules_not_in_force_exits_2_naming_branch_and_rule() {
        let runner = FixtureRunner::new(&[r#"[{"type":"non_fast_forward"}]"#, "[]"]);
        let (text, code) = host_rules_command(&runner, HOST_RULES_URL, false).unwrap();
        assert_eq!(code, 2);
        assert!(text.contains(
            "  - required rule not in force: athena/records: deletion forbidden: not in force\n"
        ));
        assert!(
            text.contains(
                "  - required rule not in force: main: up-to-date required: not in force\n"
            )
        );
        assert!(text.ends_with("host rules FAIL: 2 required rule(s) not in force\n"));
        // A non-GitHub origin is an error, not a pass.
        assert!(host_rules_command(&runner, "/srv/repo", false).is_err());
    }

    #[test]
    fn host_rules_is_a_check_flag_that_excludes_the_other_modes() {
        let parsed = Cli::try_parse_from(["athena", "check", "--host-rules"]).unwrap();
        assert!(matches!(
            parsed.command,
            Command::Check {
                host_rules: true,
                ..
            }
        ));
        assert!(Cli::try_parse_from(["athena", "check", "--host-rules", "--history"]).is_err());
    }

    #[test]
    fn check_reading_is_a_subcommand_that_takes_a_request_and_a_reading() {
        let parsed = Cli::try_parse_from([
            "athena",
            "check-reading",
            "--request",
            "r.txt",
            "--reading",
            "reading-1.json",
        ])
        .unwrap();
        assert!(matches!(parsed.command, Command::CheckReading { .. }));
        assert!(Cli::try_parse_from(["athena", "check-reading", "--request", "r.txt"]).is_err());
    }

    #[test]
    fn hestia_review_takes_backfill_and_refuses_to_run_without_it() {
        let parsed = Cli::try_parse_from(["athena", "hestia", "review", "--backfill"]).unwrap();
        assert!(matches!(
            parsed.command,
            Command::Hestia {
                action: HestiaAction::Review {
                    backfill: true,
                    json: false
                }
            }
        ));
        assert!(Cli::try_parse_from(["athena", "hestia", "review"]).is_err());
        assert!(
            Cli::try_parse_from(["athena", "hestia", "review", "--backfill", "--json"]).is_ok()
        );
    }

    #[test]
    fn measure_build_is_a_subcommand_with_its_own_help() {
        let parsed = Cli::try_parse_from(["athena", "measure-build"]).unwrap();
        assert!(matches!(
            parsed.command,
            Command::MeasureBuild { json: false }
        ));
        assert!(matches!(
            Cli::try_parse_from(["athena", "measure-build", "--json"])
                .unwrap()
                .command,
            Command::MeasureBuild { json: true }
        ));
        let help = Cli::command().render_help().to_string();
        assert!(help.contains("measure-build"), "{help}");
        let mut command = Cli::command();
        let sub = command
            .find_subcommand_mut("measure-build")
            .unwrap()
            .render_help()
            .to_string();
        assert!(sub.contains("worktrees"), "{sub}");
    }

    #[test]
    fn retry_help_names_stale_jobs_the_merge_and_the_base_fallback() {
        let mut command = Cli::command();
        let help = command
            .find_subcommand_mut("retry")
            .expect("a retry subcommand")
            .render_long_help()
            .to_string();
        // Wrapping in the rendered help must not matter.
        let help = help.split_whitespace().collect::<Vec<_>>().join(" ");
        for needle in [
            "stale",
            "merges it onto the branch tip",
            "checked and verified again with no planner or producer run",
            "a conflict starts a producer from the conflicted merge",
            "The retry starts from the base when the candidate cannot be merged",
        ] {
            assert!(help.contains(needle), "{needle:?} missing from:\n{help}");
        }
        assert!(!help.contains("a rejected or failed job again"), "{help}");
    }

    /// The long help of `athena drain` as a person reads it.
    fn drain_help(command: &mut clap::Command) -> String {
        command
            .find_subcommand_mut("drain")
            .expect("a drain subcommand")
            .render_long_help()
            .to_string()
    }

    #[test]
    fn drain_help_states_the_ceiling_the_config_enforces() {
        use athena_data::config::MAX_CONCURRENCY_CEILING;
        let help = drain_help(&mut cli_command());
        let stated = format!("at most {MAX_CONCURRENCY_CEILING})");
        assert!(help.contains(&stated), "{stated:?} missing from:\n{help}");
        assert!(!help.contains("{ceiling}"), "the placeholder survived");
    }

    #[test]
    fn drain_help_follows_the_ceiling_it_is_built_from() {
        // The text the derive carries is a template with a placeholder, not a number: with a hard-coded
        // number the filled text would not change when the ceiling does.
        let mut derived = Cli::command();
        let template = derived
            .find_subcommand_mut("drain")
            .and_then(|d| d.get_after_help())
            .expect("drain has after_help")
            .to_string();
        assert!(template.contains("at most {ceiling})"), "{template}");
        for ceiling in [3, 13, 64] {
            let filled = drain_after_help(&template, ceiling);
            assert!(filled.contains(&format!("at most {ceiling})")), "{ceiling}");
            assert!(!filled.contains("{ceiling}"));
            for stale in (1..=64).filter(|n| *n != ceiling) {
                assert!(
                    !filled.contains(&format!("at most {stale})")),
                    "{ceiling}: {stale}"
                );
            }
        }
        // Everything but the number is the same text.
        assert_eq!(
            drain_after_help(&template, 7).replace("at most 7)", "at most {ceiling})"),
            template
        );
    }

    #[test]
    fn the_new_commands_are_in_the_help_and_the_code_does_no_text_matching() {
        let top = Cli::command().render_help().to_string();
        assert!(top.contains("drain-status"), "{top}");
        assert!(top.contains("drain-restart"), "{top}");
        let help_of = |name: &str| {
            cli_command()
                .find_subcommand_mut(name)
                .unwrap_or_else(|| panic!("no subcommand {name}"))
                .render_long_help()
                .to_string()
        };
        for name in ["drain-status", "drain-restart", "drain-stop"] {
            let help = help_of(name);
            assert!(
                help.contains("pkill") && help.contains("pgrep"),
                "{name}: {help}"
            );
            assert!(help.contains("instead of"), "{name}: {help}");
        }
        assert!(help_of("drain-stop").contains("--wait"));
        let restart = help_of("drain-restart");
        assert!(
            restart.contains("background") && restart.contains("drain.log"),
            "{restart}"
        );
        assert!(help_of("drain").contains("drain-status"));
        // The flags parse, and a timeout without a wait is refused.
        assert!(matches!(
            Cli::try_parse_from(["athena", "drain-stop", "--wait", "--timeout", "30"])
                .unwrap()
                .command,
            Command::DrainStop {
                wait: true,
                timeout: Some(30)
            }
        ));
        assert!(Cli::try_parse_from(["athena", "drain-stop", "--timeout", "30"]).is_err());
        assert!(Cli::try_parse_from(["athena", "drain-restart", "--budget", "5"]).is_ok());
        assert!(
            Cli::try_parse_from(["athena", "drain-restart"]).is_err(),
            "a budget is required"
        );

        // No code that finds a process by matching text. The needles are built here so that this file does
        // not contain them. Only the code before each file's first test module is read.
        let sources = [
            ("drain.rs", include_str!("drain.rs")),
            ("main.rs", include_str!("main.rs")),
            ("wiring.rs", include_str!("wiring.rs")),
        ];
        let command = |program: &str| format!("Command::new(\"{program}\")");
        let reads_command_text = format!("/{}", "cmdline");
        for (file, source) in sources {
            let code = source.split("#[cfg(test)]").next().unwrap();
            for program in ["pgrep", "pkill", "pidof"] {
                assert!(!code.contains(&command(program)), "{file} runs {program}");
            }
            assert!(
                !code.contains(&reads_command_text),
                "{file} reads command text"
            );
            // Every `ps` call looks one process up by pid.
            for (at, _) in code.match_indices(&command("ps")) {
                let call: String = code[at..].chars().take(200).collect();
                assert!(
                    call.contains("\"-p\""),
                    "{file}: a ps call without -p: {call}"
                );
            }
        }
    }

    #[test]
    fn the_changelog_and_readme_describe_the_drain_lifecycle_commands() {
        // Line breaks in the files do not matter: compare with runs of white space as single spaces.
        let flat = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
        let changelog = include_str!("../../../CHANGELOG.md");
        let unreleased = flat(
            changelog
                .split("\n## [0.1.0]")
                .next()
                .expect("the changelog has an Unreleased section"),
        );
        for needle in [
            "drain-status",
            "drain-restart",
            "drain-stop --wait",
            "pid file",
            "refuses to start when its build is older than the landed code",
            "One-time transition",
            "Estimate, not measured",
        ] {
            assert!(
                unreleased.contains(needle),
                "CHANGELOG Unreleased lacks {needle:?}"
            );
        }
        let readme = include_str!("../../../README.md");
        let stopping = flat(
            readme
                .split("**Stopping a drain.**")
                .nth(1)
                .expect("the README has a section on stopping a drain"),
        );
        assert!(
            stopping.contains("athena drain-stop --wait"),
            "the stop section lacks --wait"
        );
        for needle in [
            "athena drain-status",
            "athena drain-restart",
            "drain.pid",
            "drain.log",
            "refuses to start",
            "One-time transition",
        ] {
            assert!(stopping.contains(needle), "README lacks {needle:?}");
        }
    }

    #[test]
    fn no_write_reasons_cover_ci_the_flag_and_other_branches() {
        assert_eq!(
            no_write_reason(false, true, Some("main")),
            Some("CI is read-only")
        );
        assert_eq!(
            no_write_reason(true, false, Some("main")),
            Some("--no-checkpoint")
        );
        assert!(no_write_reason(false, false, Some("topic")).is_some());
        assert!(no_write_reason(false, false, None).is_some());
        assert_eq!(no_write_reason(false, false, Some("main")), None);
    }

    #[test]
    fn the_reinstall_note_after_a_landing_reads_the_merged_commit_not_head() {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        let config = include_str!("../../../athena.toml");
        let write_and_commit = |config: &str, message: &str| {
            std::fs::write(dir.path().join("athena.toml"), config).unwrap();
            std::fs::write(
                dir.path().join("defaults.toml"),
                include_str!("../../../defaults.toml"),
            )
            .unwrap();
            git.run(["add", "-A"]).unwrap();
            git.run(["commit", "-q", "-m", message]).unwrap();
            git.head().unwrap().unwrap()
        };
        let good = write_and_commit(config, "readable");
        let bad = write_and_commit(
            &format!("brand_new_key = 1\n{config}"),
            "adds a field this build does not know",
        );
        // Head is a later commit that reads fine, as when the merged commit is not head.
        let head = write_and_commit(config, "later, readable");
        assert_eq!(git.head().unwrap().unwrap(), head);

        let note =
            reinstall_note_after_landing(&git, &bad).expect("the merged commit is unreadable");
        assert!(note.contains("Reinstall Athena"), "{note}");
        assert!(note.contains("brand_new_key"), "{note}");
        assert_eq!(reinstall_note_after_landing(&git, &good), None);
        assert_eq!(reinstall_note_after_landing(&git, &head), None);
    }

    #[test]
    fn checkpoint_after_landing_writes_a_checkpoint_naming_the_landed_commit() {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        std::fs::write(dir.path().join("a.txt"), "a\n").unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "init"]).unwrap();
        let base = git.head().unwrap().unwrap();
        std::fs::write(dir.path().join("b.txt"), "b\n").unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "squash"]).unwrap();
        let landed = git.head().unwrap().unwrap();

        let now: jiff::Timestamp = "2026-10-24T10:00:00Z".parse().unwrap();
        let intent_id = RecordId::from_parts(now, 1);
        let intent = LandingIntent {
            id: intent_id.clone(),
            created_at: now,
            pr_number: 1,
            head_sha: "1".repeat(40),
            head_tree_sha: git.tree_of(&landed).unwrap(),
            expected_base: base,
            job_ids: vec![JobId::parse("20261024-100000-aaaa").unwrap()],
        };
        put(
            &git,
            "landing-intents",
            &intent_id,
            HistoryRecord::LandingIntent(intent),
        );
        let landing_id = RecordId::from_parts(now, 2);
        let landing = Landing {
            id: landing_id.clone(),
            created_at: now,
            pr_number: 1,
            head_sha: "1".repeat(40),
            host_commit: landed.clone(),
            merge_method: MergeMethod::Squash,
            commits: vec![LandingCommit {
                sha: "1".repeat(40),
                tree: "2".repeat(40),
                kind: LandingKind::RecordsOnly,
                job: None,
                verification: None,
            }],
            intent: intent_id,
        };
        put(
            &git,
            "landings",
            &landing_id,
            HistoryRecord::Landing(landing),
        );
        let before = git.rev_parse(RECORDS_REF).unwrap().unwrap();
        // The records the landing wrote have reached the remote, which the checkpoint is pushed to.
        let remote_dir = tempfile::tempdir().unwrap();
        let remote = Git::isolated(remote_dir.path());
        remote.run(["init", "-q", "--bare", "-b", "main"]).unwrap();
        git.run([
            "remote",
            "add",
            "origin",
            remote_dir.path().to_str().unwrap(),
        ])
        .unwrap();
        git.push_branch("origin", "athena/records", &before)
            .unwrap();

        let run = checkpoint_after_landing(&git, &landed, now).unwrap();
        assert!(run.report.passed(), "{:?}", run.report.findings);
        let cp = run
            .checkpoint_written
            .clone()
            .expect("a checkpoint is written");
        assert_eq!(cp.main_sha, landed);
        assert_eq!(cp.records_sha, before);
        assert_eq!(cp.verifier_version, athena_gate::attribution::CHECK_VERSION);
        assert_ne!(git.rev_parse(RECORDS_REF).unwrap().unwrap(), before);
        assert!(checkpoint_note(&run).starts_with("checkpoint written"));
        // The checkpoint is on the remote records branch, read from the remote.
        let remote_tip = remote.rev_parse(RECORDS_REF).unwrap().unwrap();
        assert_eq!(remote_tip, git.rev_parse(RECORDS_REF).unwrap().unwrap());
        assert_eq!(remote.parents(&remote_tip).unwrap(), vec![before.clone()]);
        assert!(
            remote
                .show_file(&remote_tip, &format!("checkpoints/{}.json", cp.id))
                .unwrap()
                .is_some()
        );

        // A landing the records do not name writes nothing and says so.
        std::fs::write(dir.path().join("c.txt"), "c\n").unwrap();
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "hand-made"]).unwrap();
        let rogue = git.head().unwrap().unwrap();
        let tip = git.rev_parse(RECORDS_REF).unwrap().unwrap();
        let run = checkpoint_after_landing(&git, &rogue, now).unwrap();
        assert!(!run.report.passed() && run.checkpoint_written.is_none());
        assert!(checkpoint_note(&run).starts_with("checkpoint not written"));
        assert_eq!(git.rev_parse(RECORDS_REF).unwrap().unwrap(), tip);
    }
}
