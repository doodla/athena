//! `athena`: an agent orchestration system's command line (design §3).
//!
//! Exit codes: 0 when the command did what it was asked (for `go`, the job landed); 2 when a job did not
//! land (rejected, failed, stale, or blocked) or a check found violations; 1 when Athena itself erred. For
//! `drain`, 0 means the target was met after a landing, and 2 any other stop.

use anyhow::Result;
use athena_cli::{display, drain, hooks, wiring};
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
    /// Try a rejected or failed job again, starting from its introspection record (the proposed next
    /// request, the cause and the prevention) and shown as `ask` shows a reading; Enter runs it.
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
    /// Read-only: build the labeled set from introspections a reviewer labeled, score Jev's recorded shadow
    /// answers against it (Brier skill over two references, sample size, interval) and give the promotion
    /// verdict of defaults-loop section 10.4. With too few labeled cases it says so and gives no verdict.
    ScoreJev {
        /// Cases recorded before this time (RFC 3339) calibrate and later ones test. Default: the earliest
        /// two thirds by time calibrate.
        #[arg(long)]
        cut: Option<String>,
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
  repeated failure  the same item failed twice; it is named
  queue empty       nothing was left to work; this is not success
  job limit         --job-limit jobs were started
  stop requested    `athena drain-stop` was run; the job in progress was finished and no further job was started

Order: planning comes first. Every open request is fleshed out (goal, requirements-ladder rung, acceptance, \
open questions, dependencies, the paths it touches) and all of them are planned together. Drain works the \
items in plan order (higher rung first, a dependent after what it needs, age as the tiebreak), not oldest \
first, and the report shows the plan's groups: items in one group have no dependency and touch disjoint paths, \
so they could run in parallel; drain still runs one job at a time. Nothing is defaulted into a plan: acceptance \
comes only from explicit bullets in the request or a reading's criteria, and the rung from the proposal that made \
the job or from evidence in the request. The planning pass is deterministic code, not a model call. An unread \
request whose own words leave the plan incomplete is read first (within the budget) and planned again from its \
reading. An item whose plan still lacks a goal, a rung or acceptance is not worked and is reported with the \
reason. `athena plan` prints this plan without working anything.

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
    /// call, capped by `[budget] reader_usd` each and by --budget in all); `--budget 0` reads nothing.
    Plan {
        /// The most to spend reading unread requests, in US dollars.
        #[arg(long, value_name = "USD", default_value_t = 2.0, value_parser = parse_read_budget)]
        budget: f64,
        #[arg(long)]
        json: bool,
    },
    /// Ask a running `athena drain` to stop at the next job boundary: the job in progress is finished (landed and
    /// rated as usual), no further job is started, and the drain reports `stop requested`. Nothing is killed;
    /// to kill running work use `athena stop`.
    DrainStop,
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
        /// answered-question, carried-state, fixed-environment, or other.
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
    Check {
        #[arg(long)]
        history: bool,
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

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let name = args
        .first()
        .map(|a| a.rsplit('/').next().unwrap_or(a).to_string())
        .unwrap_or_default();
    if name == "pre-commit" || name == "commit-msg" {
        return ExitCode::from(hooks::git_hook(&name, &args[1..]) as u8);
    }
    let version: &'static str =
        Box::leak(wiring::version_string(&wiring::build_info()).into_boxed_str());
    let matches = Cli::command().version(version).get_matches();
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

fn open(writes: bool) -> Result<Athena> {
    wiring::open(&std::env::current_dir()?, writes, &|line| {
        eprintln!("{line}")
    })
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
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
        } => {
            let athena = open(true)?;
            if later {
                let (job, commit) = athena.queue(&request.join(" "))?;
                if json {
                    print_json(&serde_json::json!({
                        "job": job.as_str(),
                        "state": "queued",
                        "commit": commit,
                    }))?;
                } else {
                    print!("{}", display::queued(job.as_str(), &commit));
                }
                return Ok(0);
            }
            let proposal = athena.ask(&request.join(" "), &Terminal)?;
            show_proposal(&athena, proposal, yes, json)
        }
        Command::Retry { job, yes, json } => {
            let athena = open(true)?;
            let (proposal, from_introspection) = athena.retry(&job, &Terminal)?;
            if !from_introspection {
                eprintln!(
                    "job {job} has no introspection record with a proposed request; retrying as 'try again on' does, from its request and findings"
                );
            }
            show_proposal(&athena, proposal, yes, json)
        }
        Command::Propose { json } => {
            let athena = open(true)?;
            let generated = athena.propose(&Terminal)?;
            if json {
                print_json(&serde_json::json!({
                    "proposals": generated.proposals,
                    "refused": generated.refusals.iter().map(|r| serde_json::json!({
                        "failure_class": r.failure_class,
                        "reasons": r.reasons,
                    })).collect::<Vec<_>>(),
                }))?;
            }
            Ok(0)
        }
        Command::ScoreJev { cut, json } => {
            let cut = cut
                .map(|text| {
                    text.parse::<jiff::Timestamp>().map_err(|e| {
                        anyhow::anyhow!("--cut {text:?} is not an RFC 3339 time such as 2026-10-01T00:00:00Z: {e}")
                    })
                })
                .transpose()?;
            let athena = open(false)?;
            let report = athena.score_jev(cut)?;
            if json {
                print_json(&report)?;
            } else {
                print!("{}", display::jev_score(&report));
            }
            Ok(0)
        }
        Command::Review {
            id,
            right,
            class,
            new_class,
            description,
            note,
            json,
        } => {
            let verdict = athena_core::review::Verdict::from_flags(
                right,
                class.as_deref(),
                new_class.as_deref(),
                description.as_deref(),
            )?;
            let athena = open(true)?;
            let review = athena.review(&id, &verdict, note.as_deref())?;
            if json {
                print_json(&review)?;
            } else {
                print!("{}", display::review(&review));
            }
            Ok(0)
        }
        Command::AcceptProposal { id, yes, json } => {
            let athena = open(true)?;
            let proposal = athena.accept_proposal(&id, &Terminal)?;
            show_proposal(&athena, proposal, yes, json)
        }
        Command::RejectProposal { id, why, json } => {
            let athena = open(true)?;
            let rejected = athena.reject_proposal(&id, &why)?;
            if json {
                print_json(&rejected)?;
            } else {
                println!(
                    "rejected {}; it is not proposed again until new cases arrive",
                    rejected.id
                );
            }
            Ok(0)
        }
        Command::ApproveWork { id, yes, json } => {
            let athena = open(true)?;
            let proposal = athena.approve_work(&id, &Terminal)?;
            show_proposal(&athena, proposal, yes, json)
        }
        Command::RejectWork { id, why, json } => {
            let athena = open(true)?;
            let rejected = athena.reject_work(&id, &why)?;
            if json {
                print_json(&rejected)?;
            } else {
                println!("rejected work item {}", rejected.work_item);
            }
            Ok(0)
        }
        Command::Audience { url, reader, json } => {
            let athena = open(true)?;
            let mut target = athena_core::audience::Target::explainer();
            if let Some(url) = url {
                target.url = url;
            }
            if let Some(reader) = reader {
                target.audience = reader;
            }
            let Some(rated) = athena.audience(&athena_core::audience::LivePage, &target)? else {
                return Ok(0);
            };
            if json {
                print_json(&serde_json::json!({
                    "rating": rated.rating,
                    "work_item": rated.work_item,
                }))?;
            } else {
                println!(
                    "{}/5 from a {} reading {}",
                    rated.rating.rating, rated.rating.audience, rated.rating.target
                );
                for f in &rated.rating.findings {
                    println!("  - {} (evidence: {})", f.text, f.source);
                }
                if let Some(item) = &rated.work_item {
                    println!("queued work item {}: {}", item.id, item.title);
                }
            }
            Ok(0)
        }
        Command::CheckProposal { id, candidate } => {
            let athena = open(false)?;
            let text = std::fs::read_to_string(&candidate)?;
            let problems = athena.check_proposal(&id, &text)?;
            for problem in &problems {
                println!("  - {problem}");
            }
            if problems.is_empty() {
                println!("{} applies proposal {id}", candidate.display());
            }
            Ok(if problems.is_empty() { 0 } else { 2 })
        }
        Command::Correct { job, line, json } => {
            let athena = open(true)?;
            let proposal = athena.correct(&job, &line.join(" "), &Terminal)?;
            if json {
                print_json(&proposal_json(&proposal))?;
                return Ok(0);
            }
            if std::io::stdin().is_terminal() && std::io::stdout().is_terminal() {
                return prompt_loop(&athena, proposal);
            }
            if let Some(r) = &proposal.reading {
                print!("{}", display::reading(proposal.job.as_str(), r, false));
            }
            print!("{}", display::follow_up(&proposal));
            Ok(0)
        }
        Command::Go { job, json } => {
            let athena = open(true)?;
            go_and_report(&athena, &job, AcceptedBy::User, json)
        }
        Command::Land { job, pr: _, json } => {
            let athena = open(true)?;
            let report = athena.land_pr(
                &job,
                &athena_core::gh::GhCli,
                wiring::checks_poll(&std::thread::sleep),
                &Terminal,
            )?;
            if json {
                print_json(&report)?;
            } else {
                print!("{}", display::land_pr(&report));
            }
            Ok(land_pr_code(&report))
        }
        Command::Drain {
            budget,
            rating_threshold,
            job_limit,
            json,
        } => {
            let athena = open(true)?;
            // A stop asked for before this run started belongs to an earlier run.
            let _ = std::fs::remove_file(drain::stop_file(athena.state.root()));
            let report = drain::drain(
                &drain::RealSteps::live(&athena, &Terminal),
                drain::Limits {
                    budget_usd: budget,
                    target_rating: rating_threshold,
                    max_jobs: job_limit,
                },
                &Terminal,
            )?;
            if json {
                print_json(&report)?;
            } else {
                print!("{}", display::drain(&report));
            }
            Ok(if report.target_met() { 0 } else { 2 })
        }
        Command::Plan { budget, json } => {
            let athena = open(true)?;
            let report = drain::plan(
                &drain::RealSteps::live(&athena, &Terminal),
                budget,
                &Terminal,
            )?;
            if json {
                print_json(&report)?;
            } else {
                print!("{}", display::plan(&report));
            }
            Ok(0)
        }
        Command::DrainStop => {
            let athena = open(false)?;
            let file = drain::stop_file(athena.state.root());
            std::fs::create_dir_all(athena.state.root())?;
            std::fs::write(&file, "stop at the next job boundary\n")?;
            println!(
                "stop asked for: a running drain finishes its current job and starts no further one"
            );
            Ok(0)
        }
        Command::Close {
            job,
            why,
            allow_legacy_usage,
            json,
        } => {
            let athena = open(true)?;
            let closed = athena.close_with(&job, &why, allow_legacy_usage, &Terminal)?;
            if json {
                print_json(
                    &serde_json::json!({ "job": closed.job.as_str(), "state": closed.state.as_str(), "commit": closed.commit }),
                )?;
            }
            Ok(0)
        }
        Command::Status { job, json } => {
            let athena = open(false)?;
            match job {
                Some(job) => {
                    let trace = athena.trace(&job)?;
                    if json {
                        print_json(&trace)?
                    } else {
                        print!("{}", display::trace(&trace))
                    }
                }
                None => {
                    let view = athena.status()?;
                    if json {
                        print_json(&view)?
                    } else {
                        print!("{}", display::status(&view))
                    }
                }
            }
            Ok(0)
        }
        Command::Intervene {
            what,
            why,
            class,
            category,
            job,
            with_staged,
            json,
        } => {
            let athena = open(true)?;
            let done = athena.intervene(
                &athena_core::manage::InterveneArgs {
                    what,
                    why,
                    class,
                    category,
                    job,
                    with_staged,
                },
                &Terminal,
            )?;
            if json {
                print_json(&serde_json::json!({
                    "intervention": done.intervention,
                    "commit": done.commit,
                    "rescore": done.rescore,
                }))?;
            } else if let Some(s) = &done.rescore {
                println!("re-scored  {} · {}", s.job.as_str(), display::score_line(s));
            }
            Ok(0)
        }
        Command::Restart {
            cost,
            minutes,
            tokens,
            measured,
            lost,
            via,
            json,
        } => {
            let athena = open(true)?;
            let (restart, commit) = athena.restart(
                &athena_core::manage::RestartArgs {
                    cost,
                    minutes,
                    tokens,
                    measured,
                    lost,
                    via,
                },
                &Terminal,
            )?;
            if json {
                print_json(&serde_json::json!({ "restart": restart, "commit": commit }))?;
            }
            Ok(0)
        }
        Command::Check { history, json } => {
            let athena = open(false)?;
            let report = if history {
                athena.check_history()?
            } else {
                athena.check_working()?
            };
            if json {
                print_json(&serde_json::json!({
                    "commits": report.commits,
                    "records": report.records,
                    "violations": report.violations.iter().map(|v| v.to_string()).collect::<Vec<_>>(),
                }))?;
            } else {
                for v in &report.violations {
                    println!("  - {v}");
                }
                if history {
                    println!(
                        "checked {} commits and {} record files: {} violation(s)",
                        report.commits,
                        report.records,
                        report.violations.len()
                    );
                } else {
                    println!(
                        "checked {} record files at HEAD: {} violation(s)",
                        report.records,
                        report.violations.len()
                    );
                }
            }
            Ok(if report.violations.is_empty() { 0 } else { 2 })
        }
        Command::Backtest { json } => {
            let report = athena_cli::backtest::run(&std::env::current_dir()?)?;
            if json {
                print_json(&report)?;
            } else {
                print!("{}", athena_cli::backtest::render(&report));
            }
            Ok(if report.passed() { 0 } else { 2 })
        }
        Command::Strategy { json } => {
            let report = athena_cli::strategy::run(&std::env::current_dir()?)?;
            if json {
                print_json(&report)?;
            } else {
                print!("{}", athena_cli::strategy::render(&report));
            }
            Ok(if report.errors.is_empty() { 0 } else { 2 })
        }
        Command::Stop => {
            let athena = open(false)?;
            let n = athena.stop()?;
            println!(
                "kill switch on: terminated {n} process group(s); new agents and checks are blocked until `athena start`"
            );
            Ok(0)
        }
        Command::Start => {
            let athena = open(false)?;
            let was = athena.start()?;
            println!(
                "{}",
                if was {
                    "kill switch off: new work may start"
                } else {
                    "the kill switch was already off"
                }
            );
            Ok(0)
        }
        Command::Hook {
            hook: HookCommand::PreToolUse { policy },
        } => Ok(hooks::pre_tool_use(&policy) as u8),
    }
}
