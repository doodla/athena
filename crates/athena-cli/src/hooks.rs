//! Entry points that other programs call: git's `pre-commit` and `commit-msg` hooks (symlinks to this
//! binary, dispatched on its own name, D18 and E13), and Claude Code's `PreToolUse` hook.

use anyhow::{Context, Result};
use athena_agent::policy::{Decision, Policy, evaluate};
use athena_gate::{Change, Scope, gate};
use athena_git::Git;
use std::io::Read;
use std::path::Path;

/// Runs the gate as a git hook. Returns the process exit code.
pub fn git_hook(name: &str, args: &[String]) -> i32 {
    match run_git_hook(name, args) {
        Ok(0) => 0,
        Ok(n) => {
            eprintln!(
                "athena {name}: the commit was refused ({n} violation(s)). Fix them and commit again."
            );
            1
        }
        Err(error) => {
            eprintln!(
                "athena {name}: the gate could not reach a verdict, so the commit is refused: {error:#}"
            );
            1
        }
    }
}

fn run_git_hook(name: &str, args: &[String]) -> Result<usize> {
    let cwd = std::env::current_dir()?;
    let git = Git::discover(&cwd)?;
    let (change, scope) = match name {
        "pre-commit" => (Change::staged(&git, None)?, Scope::PreCommit),
        "commit-msg" => {
            let file = args.first().context("commit-msg needs the message file")?;
            let message =
                std::fs::read_to_string(file).with_context(|| format!("reading {file}"))?;
            (Change::staged(&git, Some(message))?, Scope::CommitMsg)
        }
        other => anyhow::bail!("athena does not handle the {other} hook"),
    };
    let violations = gate(&git, &change, scope)?;
    for v in &violations {
        eprintln!("  - {v}");
    }
    Ok(violations.len())
}

/// Claude Code's `PreToolUse` hook: reads the tool call from stdin and prints a deny decision, or
/// nothing for no objection. It fails closed: anything it cannot read is denied.
pub fn pre_tool_use(policy_file: &Path) -> i32 {
    let decision = (|| -> Result<Decision> {
        let policy: Policy = serde_json::from_str(&std::fs::read_to_string(policy_file)?)?;
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        let event: serde_json::Value = serde_json::from_str(&input)?;
        let tool = event
            .get("tool_name")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let tool_input = event
            .get("tool_input")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        let cwd = event.get("cwd").and_then(|v| v.as_str()).map(Path::new);
        Ok(evaluate(&policy, tool, &tool_input, cwd))
    })()
    .unwrap_or_else(|e| {
        Decision::Deny(format!(
            "the hook could not read this tool call ({e}), so it is denied"
        ))
    });
    if let Some(output) = decision.hook_output() {
        println!("{output}");
    }
    0
}
