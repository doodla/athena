//! The format step of the producer stage: Athena runs `cargo fmt` on the producer's work tree.
//!
//! Root cause it answers: the producer's grant allows `cargo fmt`, but only prose asked the producer to run
//! it, and nothing made the tree formatted before the configured `fmt` check ran. A job whose only defect was
//! rustfmt layout was rejected for layout, not for its work. Now the layout is fixed by code: `cargo fmt --all`
//! rewrites the tree, then `cargo fmt --all --check` confirms it.
//!
//! The step is Athena's own code, not a command an agent runs, so the agent policy that denies `cargo fmt`
//! without `--check` to roles that cannot write is unchanged. It runs both commands through
//! [`crate::checks::run_check`], so they get the same deadline, process-group kill, kill switch, redaction and
//! output tail as the configured checks.
//!
//! Where it is called: the producer loop's own text says "The format step belongs at the end of producer_loop,
//! before the change list is taken". The call is the last producer-stage act of every pass: right after the
//! producer agent returned and its report was parsed, and before the loop reads the change list. "After the
//! producer" means after each producer invocation returns. The loop's exit, the replay the loop measures, the
//! candidate, the checks and the verifier all come after it, so each sees the formatted tree, and a send-back
//! pass formats again (the step is idempotent).
//!
//! It runs only when the work tree has a `Cargo.toml` and the change list holds a Rust file that is not
//! deleted, so a job that changes no Rust does not grow its diff by layout drift in files it never touched.
//! An error the formatter cannot fix (a file that does not parse) stays in the tree: the configured `fmt` check
//! fails on it and reports it as before, so the gate is not relaxed.

use crate::capability::CheckSpec;
use crate::checks::{CheckContext, run_check, timed_out};
use anyhow::Result;
use athena_data::config::CheckKind;
use athena_data::records::CheckRun;
use athena_git::DiffEntry;
use std::collections::BTreeSet;
use std::path::Path;

/// What the format step did to a work tree.
#[derive(Debug, PartialEq)]
pub(super) enum Formatted {
    /// The step did not apply, for this reason. Nothing ran.
    Skipped(&'static str),
    /// `cargo fmt --all` ran and `cargo fmt --all --check` then passed.
    Done,
    /// `cargo fmt --all --check` still fails after the rewrite. `output` is that run's own failure text.
    Unfixed {
        output: String,
        note: Option<String>,
    },
    /// The kill switch was set or a run hit its deadline; the job must stop, with this reason.
    Stopped(String),
}

fn spec(args: &[&str]) -> CheckSpec {
    CheckSpec {
        name: "format".to_string(),
        run: args.iter().map(|a| a.to_string()).collect(),
        kind: CheckKind::Command,
        skip_when_only: None,
    }
}

/// Why `run` ended the step, when it did: it was killed at its deadline, or the kill switch is set.
fn stop_reason(run: &CheckRun, ctx: &CheckContext<'_>) -> Option<String> {
    if timed_out(run) {
        return run.note.clone();
    }
    ctx.registry.stopped().then(|| {
        run.note
            .clone()
            .unwrap_or_else(|| "the kill switch is on".to_string())
    })
}

/// Formats the Rust in `dir` with `cargo fmt --all`, then checks it with `cargo fmt --all --check`.
/// `changes` is the producer's change list, which decides whether the step applies at all.
pub(super) fn format_work_tree(
    dir: &Path,
    changes: &[DiffEntry],
    ctx: &CheckContext<'_>,
) -> Result<Formatted> {
    if !dir.join("Cargo.toml").is_file() {
        return Ok(Formatted::Skipped("the work tree has no Cargo.toml"));
    }
    if !changes
        .iter()
        .any(|c| c.status != 'D' && c.path.ends_with(".rs"))
    {
        return Ok(Formatted::Skipped("no Rust file changed"));
    }
    if ctx.registry.stopped() {
        return Ok(Formatted::Stopped("the kill switch is on".to_string()));
    }
    // Whether the rewrite passed or failed does not matter: the check below says whether the tree is clean.
    let rewrite = run_check(&spec(&["cargo", "fmt", "--all"]), ctx)?;
    if let Some(reason) = stop_reason(&rewrite, ctx) {
        return Ok(Formatted::Stopped(reason));
    }
    let check = run_check(&spec(&["cargo", "fmt", "--all", "--check"]), ctx)?;
    if let Some(reason) = stop_reason(&check, ctx) {
        return Ok(Formatted::Stopped(reason));
    }
    Ok(if check.passed {
        Formatted::Done
    } else {
        Formatted::Unfixed {
            output: check.output_tail,
            note: check.note,
        }
    })
}

/// The paths in `after` that `before` does not list, sorted: the files the formatter added to the change list
/// beyond the producer's own.
pub(super) fn paths_added_by_format(before: &[DiffEntry], after: &[DiffEntry]) -> Vec<String> {
    let known: BTreeSet<&str> = before.iter().map(|c| c.path.as_str()).collect();
    after
        .iter()
        .map(|c| c.path.as_str())
        .filter(|p| !known.contains(p))
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// The line that says why the formatter could not fix the tree: the first line that starts with `error`, else
/// the first line of the output, else the run's note.
pub(super) fn unfixed_summary(output: &str, note: Option<&str>) -> String {
    output
        .lines()
        .find(|l| l.starts_with("error"))
        .or_else(|| output.lines().find(|l| !l.trim().is_empty()))
        .map(str::to_string)
        .or_else(|| note.map(str::to_string))
        .unwrap_or_else(|| "no output".to_string())
}
