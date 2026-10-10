//! `athena`: an agent orchestration system's command line (design §3).
//!
//! Exit codes: 0 when the command did what it was asked (for `go`, the job landed); 2 when a job did not
//! land (rejected, failed, stale, or blocked) or a check found violations; 1 when Athena itself erred.

use anyhow::Result;
use athena_cli::{display, hooks, wiring};
use athena_core::reading::Proposal;
use athena_core::run::GoResult;
use athena_core::{Athena, Ui};
use athena_data::records::{AcceptedBy, OutcomeState};
use clap::{Parser, Subcommand};
use std::io::{BufRead, IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "athena",
    version,
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
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        json: bool,
    },
    /// Correct the latest reading in one line (or answer its question), and read again.
    Correct {
        job: String,
        #[arg(required = true, num_args = 1..)]
        line: Vec<String>,
        #[arg(long)]
        json: bool,
    },
    /// Run a proposed job to its end, or retry the landing of a blocked one.
    Go {
        job: String,
        #[arg(long)]
        json: bool,
    },
    /// Close a job that will not run or finish.
    Close {
        job: String,
        #[arg(long)]
        why: String,
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
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(1)
        }
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

fn go_and_report(athena: &Athena, job: &str, accepted_by: AcceptedBy, json: bool) -> Result<u8> {
    match athena.go(job, accepted_by, &Terminal)? {
        GoResult::Reread(p) => {
            if json {
                print_json(&proposal_json(&p))?;
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

fn run(cli: Cli) -> Result<u8> {
    match cli.command {
        Command::Ask { request, yes, json } => {
            let athena = open(true)?;
            let proposal = athena.ask(&request.join(" "), &Terminal)?;
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
                    return go_and_report(&athena, proposal.job.as_str(), AcceptedBy::Flag, json);
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
                return prompt_loop(&athena, proposal);
            }
            if let Some(r) = &proposal.reading {
                print!("{}", display::reading(proposal.job.as_str(), r, false));
            }
            print!("{}", display::follow_up(&proposal));
            Ok(0)
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
        Command::Close { job, why, json } => {
            let athena = open(true)?;
            let closed = athena.close(&job, &why, &Terminal)?;
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
