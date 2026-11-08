//! One handler per `athena` subcommand with a body of any size. `run` in `main.rs` destructures the parsed
//! command and calls these; each returns the exit code.

use super::*;

pub(crate) fn ask(request: Vec<String>, yes: bool, later: bool, json: bool) -> Result<u8> {
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

pub(crate) fn retry(job: String, yes: bool, json: bool) -> Result<u8> {
    let athena = open(true)?;
    let (proposal, from_introspection) = athena.retry(&job, &Terminal)?;
    if !from_introspection {
        eprintln!(
            "job {job} has no introspection record with a proposed request; retrying as 'try again on' does, from its request and findings"
        );
    }
    show_proposal(&athena, proposal, yes, json)
}

pub(crate) fn propose(json: bool) -> Result<u8> {
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

pub(crate) fn score_jev(cut: Option<String>, derive: bool, json: bool) -> Result<u8> {
    let cut = cut
        .map(|text| {
            text.parse::<jiff::Timestamp>().map_err(|e| {
                anyhow::anyhow!(
                    "--cut {text:?} is not an RFC 3339 time such as 2026-10-01T00:00:00Z: {e}"
                )
            })
        })
        .transpose()?;
    let athena = open(derive)?;
    let derivation = derive.then(|| athena.derive_labels()).transpose()?;
    let mut report = athena.score_jev(cut)?;
    report.derivation = derivation;
    if json {
        print_json(&report)?;
    } else {
        print!("{}", display::jev_score(&report));
    }
    Ok(0)
}

pub(crate) fn review(
    id: String,
    right: bool,
    class: Option<String>,
    new_class: Option<String>,
    description: Option<String>,
    note: Option<String>,
    json: bool,
) -> Result<u8> {
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

pub(crate) fn reject_proposal(id: String, why: String, json: bool) -> Result<u8> {
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

pub(crate) fn reject_work(id: String, why: String, json: bool) -> Result<u8> {
    let athena = open(true)?;
    let rejected = athena.reject_work(&id, &why)?;
    if json {
        print_json(&rejected)?;
    } else {
        println!("rejected work item {}", rejected.work_item);
    }
    Ok(0)
}

pub(crate) fn audience(url: Option<String>, reader: Option<String>, json: bool) -> Result<u8> {
    let athena = open(true)?;
    warn_if_stale(&athena);
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

pub(crate) fn audit(job: String, json: bool) -> Result<u8> {
    let athena = open(true)?;
    warn_if_stale(&athena);
    // The mistake report does not depend on a job audit: it prints when none is produced, and when the
    // audit errors.
    let mistakes = athena.mistakes()?;
    let audited = match athena.audit(&job) {
        Ok(audited) => audited,
        Err(error) => {
            if json {
                print_json(&serde_json::json!({ "mistakes": mistakes }))?;
            } else {
                print!("{}", display::mistakes(&mistakes));
            }
            return Err(error);
        }
    };
    if json {
        print_json(&serde_json::json!({
            "mistakes": mistakes,
            "audit": audited.as_ref().map(|a| &a.audit),
            "invocation": audited.as_ref().map(|a| a.invocation_path()),
        }))?;
    } else {
        print!("{}", display::audit_report(&mistakes, audited.as_ref()));
    }
    Ok(0)
}

pub(crate) fn hestia_review(json: bool) -> Result<u8> {
    let report = open(true)?.error_review_backfill(&Terminal)?;
    if json {
        print_json(&report)?;
    } else {
        print!("{}", report.render());
    }
    Ok(0)
}

pub(crate) fn hestia(action: HestiaAction) -> Result<u8> {
    let (sweep, json) = match action {
        HestiaAction::Review { json, .. } => return hestia_review(json),
        HestiaAction::Sweep { json } => {
            let athena = open(true)?;
            let sweep =
                athena.hestia_sweep_with(&athena_core::hestia::default_checks(), &Terminal)?;
            (Some(sweep), json)
        }
        HestiaAction::Last { json } => (open(false)?.last_sweep()?, json),
    };
    if json {
        print_json(&sweep)?;
    } else {
        print!("{}", athena_cli::hestia_report(sweep.as_ref()));
    }
    Ok(0)
}

pub(crate) fn measure_build(json: bool) -> Result<u8> {
    let measurement = open(true)?.measure_build(&athena_core::disk::CargoBuild, &Terminal)?;
    if json {
        print_json(&measurement)?;
    }
    Ok(0)
}

pub(crate) fn check_proposal(id: String, candidate: PathBuf) -> Result<u8> {
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

pub(crate) fn correct(job: String, line: Vec<String>, json: bool) -> Result<u8> {
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

pub(crate) fn priority(job: String, level: String, json: bool) -> Result<u8> {
    let athena = open(true)?;
    let (id, priority) = athena.set_priority(&job, &level)?;
    if json {
        print_json(&serde_json::json!({ "job": id.as_str(), "priority": priority }))?;
    } else {
        println!("{id} is now {} priority", priority.as_str());
    }
    Ok(0)
}

pub(crate) fn focus(text: Option<String>, json: bool) -> Result<u8> {
    let athena = open(true)?;
    let current = match text {
        Some(text) => Some(athena.set_focus(&text)?),
        None => athena.focus()?,
    };
    match (current, json) {
        (Some(focus), true) => print_json(&focus)?,
        (Some(focus), false) => println!("{}", focus.text),
        (None, true) => print_json(&serde_json::Value::Null)?,
        (None, false) => println!("no focus is set; set one with: athena focus \"<text>\""),
    }
    Ok(0)
}

pub(crate) fn go(job: String, json: bool) -> Result<u8> {
    let athena = open(true)?;
    warn_if_stale(&athena);
    go_and_report(&athena, &job, AcceptedBy::User, json)
}

pub(crate) fn land(job: String, json: bool) -> Result<u8> {
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
    // After a landing record, the attribution check; a pass writes a checkpoint. The note goes to
    // stderr so `--json` output stays whole, and nothing here changes the exit code.
    if report.link.state == athena_core::pr::LinkState::Landed
        && let Some(merged) = &report.link.merged_commit
        && matches!(athena.git.commit_exists(merged), Ok(true))
    {
        // A landing never checked that the running build can read the config it lands, so a new
        // field in the merged athena.toml surfaced later as a failed job. The merged commit is read,
        // not head, which may predate the merge.
        if let Some(note) = reinstall_note_after_landing(&athena.git, merged) {
            eprintln!("{note}");
        }
        if !ci_is_set() {
            match checkpoint_after_landing(&athena.git, merged, jiff::Timestamp::now()) {
                Ok(run) => eprint!("{}", checkpoint_note(&run)),
                Err(error) => eprintln!("checkpoint not written: {error}"),
            }
        }
    }
    Ok(land_pr_code(&report))
}

pub(crate) fn drain(budget: f64, rating_threshold: u8, job_limit: u32, json: bool) -> Result<u8> {
    let athena = open(true)?;
    let build = format!("athena {}", wiring::version_string(&athena.build));
    // One drain per state directory: this refuses a build older than the landed code and a second
    // live drain, and clears a stop left by an earlier run (unless this is the same drain carrying on
    // after its own rebuild). The pid file is removed when the run ends.
    let report = drain::guarded(
        athena.state.root(),
        &drain::RealProcs,
        &build,
        &|| athena.stale_build(),
        || {
            // `[parallel] max_concurrency` from the head commit's athena.toml; 1 is one job at a time.
            let max_concurrency =
                athena_core::capability::JobConfig::at(&athena.git, &athena.head()?)?
                    .config
                    .parallel
                    .max_concurrency;
            let report = drain::drain_concurrent(
                &drain::RealSteps::live(&athena, &Terminal),
                drain::Limits {
                    budget_usd: budget,
                    target_rating: rating_threshold,
                    max_jobs: job_limit,
                },
                &Terminal,
                max_concurrency,
            )?;
            // After a landing that changed Athena's own code and a successful rebuild, the newly
            // installed binary carries on with the remaining budget and job limit. This returns only
            // if that failed.
            drain::follow_restart(report, &Terminal, &|left| {
                reexec(&drain::reexec_args(&left, json))
            })
        },
    )?;
    if json {
        print_json(&report)?;
    } else {
        print!("{}", display::drain(&report));
    }
    Ok(if report.target_met() { 0 } else { 2 })
}

pub(crate) fn plan(budget: f64, json: bool) -> Result<u8> {
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

pub(crate) fn drain_stop(wait: bool, timeout: Option<u64>) -> Result<u8> {
    let athena = open(false)?;
    let waiting = wait.then(|| drain::StopWait::new(timeout.map(std::time::Duration::from_secs)));
    let outcome = drain::stop_and_wait(
        athena.state.root(),
        &drain::RealProcs,
        waiting.as_ref(),
        &Terminal,
    )?;
    print!("{}", display::drain_stop(&outcome));
    Ok(match outcome {
        drain::StopOutcome::TimedOut(..) => 2,
        _ => 0,
    })
}

pub(crate) fn drain_status(json: bool) -> Result<u8> {
    let athena = open(false)?;
    let root = athena.state.root();
    let state = drain::drain_state(root, &drain::RealProcs);
    let stop_asked = drain::stop_file(root).exists();
    if json {
        print_json(&drain_status_json(&state, stop_asked))?;
    } else {
        print!("{}", display::drain_status(&state, stop_asked));
    }
    Ok(state.exit_code())
}

pub(crate) fn drain_restart(
    budget: f64,
    rating_threshold: u8,
    job_limit: u32,
    timeout: Option<u64>,
    json: bool,
) -> Result<u8> {
    let athena = open(false)?;
    let life = drain::RealLifecycle {
        athena: &athena,
        limits: drain::Limits {
            budget_usd: budget,
            target_rating: rating_threshold,
            max_jobs: job_limit,
        },
        json,
    };
    let waiting = drain::StopWait::new(timeout.map(std::time::Duration::from_secs));
    let outcome = drain::restart(&life, &waiting, &Terminal)?;
    print!("{}", display::drain_restart(&outcome));
    Ok(match outcome {
        drain::RestartOutcome::Started(_) => 0,
        drain::RestartOutcome::StillStopping(..) => 2,
    })
}

pub(crate) fn close(job: String, why: String, allow_legacy_usage: bool, json: bool) -> Result<u8> {
    let athena = open(true)?;
    let closed = athena.close_with(&job, &why, allow_legacy_usage, &Terminal)?;
    if json {
        print_json(
            &serde_json::json!({ "job": closed.job.as_str(), "state": closed.state.as_str(), "commit": closed.commit }),
        )?;
    }
    Ok(0)
}

/// Handled before `open`: a snapshot is read on its own, with no repository.
pub(crate) fn status_snapshot(dir: PathBuf, json: bool) -> Result<u8> {
    let view = athena_core::snapshot::status(&dir)?;
    if json {
        print_json(&view)?
    } else {
        print!("{}", display::snapshot_status(&view))
    }
    Ok(0)
}

pub(crate) fn status(job: Option<String>, json: bool) -> Result<u8> {
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
            let view = athena.status_with_reconcile(&athena_core::gh::GhCli, &Terminal)?;
            if json {
                print_json(&view)?
            } else {
                print!("{}", display::status(&view))
            }
        }
    }
    Ok(0)
}

pub(crate) fn intervene(args: athena_core::manage::InterveneArgs, json: bool) -> Result<u8> {
    let athena = open(true)?;
    let done = athena.intervene(&args, &Terminal)?;
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

pub(crate) fn restart(args: athena_core::manage::RestartArgs, json: bool) -> Result<u8> {
    let athena = open(true)?;
    let (restart, commit) = athena.restart(&args, &Terminal)?;
    if json {
        print_json(&serde_json::json!({ "restart": restart, "commit": commit }))?;
    }
    Ok(0)
}

/// The options of `athena check`, as parsed.
pub(crate) struct CheckOptions {
    pub(crate) history: bool,
    pub(crate) pr_head: Option<Vec<String>>,
    pub(crate) attribution: bool,
    pub(crate) host_rules: bool,
    pub(crate) full_replay: bool,
    pub(crate) no_checkpoint: bool,
    pub(crate) json: bool,
}

pub(crate) fn check(options: CheckOptions) -> Result<u8> {
    let CheckOptions {
        history,
        pr_head,
        attribution,
        host_rules,
        full_replay,
        no_checkpoint,
        json,
    } = options;
    let athena = open(false)?;
    if host_rules {
        let url = athena.git.run(["remote", "get-url", "origin"])?;
        let (text, code) = host_rules_command(&athena_core::gh::GhCli, &url, json)?;
        print!("{text}");
        return Ok(code);
    }
    if attribution {
        use athena_gate::attribution::{Mode, Options, run_check};
        let tip = athena
            .git
            .head()?
            .ok_or_else(|| anyhow::anyhow!("the repository has no commits"))?;
        let no_write = no_write_reason(
            no_checkpoint,
            ci_is_set(),
            athena.git.current_branch()?.as_deref(),
        );
        let mode = if full_replay {
            Mode::FullReplay
        } else {
            Mode::Incremental
        };
        let options = Options::new(mode, no_write.is_none());
        let run = run_check(&athena.git, &tip, jiff::Timestamp::now(), &options)?;
        if json {
            print_json(&run_json(&run, no_write))?;
        } else {
            print!("{}", run_text(&run, no_write));
        }
        return Ok(if run.report.passed() { 0 } else { 2 });
    }
    if let Some(range) = pr_head {
        let report = athena_core::pr_check::check_pr_head(&athena.git, &range[0], &range[1])?;
        if json {
            print_json(&serde_json::json!({
                "base": report.base,
                "head": report.head,
                "commits": report.commits.len(),
                "passed": report.passed(),
                "offences": report.offences.iter().map(|o| serde_json::json!({
                    "sha": o.sha,
                    "reason": o.reason,
                })).collect::<Vec<_>>(),
            }))?;
        } else {
            print!("{}", display::pr_head_report(&report));
        }
        return Ok(if report.passed() { 0 } else { 2 });
    }
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
        for note in &report.notes {
            println!("note: {note}");
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

pub(crate) fn backtest(json: bool) -> Result<u8> {
    let report = athena_cli::backtest::run(&std::env::current_dir()?)?;
    if json {
        print_json(&report)?;
    } else {
        print!("{}", athena_cli::backtest::render(&report));
    }
    Ok(if report.passed() { 0 } else { 2 })
}

pub(crate) fn strategy(json: bool) -> Result<u8> {
    let report = athena_cli::strategy::run(&std::env::current_dir()?)?;
    if json {
        print_json(&report)?;
    } else {
        print!("{}", athena_cli::strategy::render(&report));
    }
    Ok(if report.errors.is_empty() { 0 } else { 2 })
}

pub(crate) fn check_reading(request: PathBuf, reading: PathBuf) -> Result<u8> {
    let (line, code) = check_reading_command(&request, &reading)?;
    println!("{line}");
    Ok(code)
}

pub(crate) fn stop() -> Result<u8> {
    let athena = open(false)?;
    let n = athena.stop()?;
    println!(
        "kill switch on: terminated {n} process group(s); new agents and checks are blocked until `athena start`"
    );
    Ok(0)
}

pub(crate) fn start() -> Result<u8> {
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
