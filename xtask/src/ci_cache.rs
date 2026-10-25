//! The 'fmt, clippy, tests' CI job's reuse of its own earlier pass (docs/decisions.md, D121).
//!
//! The workflow looks a cache entry up under a key made from the governed-content fingerprint. The entry is
//! a directory holding one marker file, which names the fingerprint and exists only if the checks passed.
//! This module holds the decisions, so `cargo test` covers them and the workflow only calls them:
//!
//! - `fingerprint`: prints `athena_git::attrs::fingerprint` of `HEAD` under the rules committed at `HEAD`;
//! - `ci-cache check <dir> <fingerprint> [--output <file>]`: prints `skip=true` only if `<dir>` holds a marker
//!   naming that fingerprint, else `skip=false`; with `--output` it also appends the line to that file;
//! - `ci-cache mark <dir> <fingerprint> --outcome <outcome>`: writes the marker only if the outcome is
//!   `success`; any other outcome writes nothing;
//! - `ci-cache report <fingerprint> <matched-key> [--summary <file>]`: says why the build was skipped.

use anyhow::{Context, Result, bail};
use athena_git::{Git, attrs};
use std::io::Write;
use std::path::Path;

pub const USAGE: &str = "cargo run -p xtask -- fingerprint\n       \
     cargo run -p xtask -- ci-cache check <dir> <fingerprint> [--output <file>]\n       \
     cargo run -p xtask -- ci-cache mark <dir> <fingerprint> --outcome <outcome>\n       \
     cargo run -p xtask -- ci-cache report <fingerprint> <matched-key> [--summary <file>]";

/// The name of the marker file; its presence, with the right content, means 'the checks passed'.
const MARKER: &str = "passed";

/// `fingerprint`: the governed-content fingerprint of the repository containing the current directory.
pub fn fingerprint_command() -> Result<()> {
    let cwd = std::env::current_dir().context("cannot read the current directory")?;
    let git = Git::discover(&cwd).context("not inside a git repository")?;
    let fingerprint = attrs::fingerprint(&git, "HEAD", "HEAD")?;
    println!("{fingerprint}");
    Ok(())
}

/// Whether the checks can be skipped: the restored directory holds a marker naming this fingerprint.
pub fn should_skip(dir: &Path, fingerprint: &str) -> bool {
    !fingerprint.is_empty()
        && std::fs::read_to_string(dir.join(MARKER)).is_ok_and(|text| text.trim() == fingerprint)
}

/// Writes the marker if and only if `outcome` is `success`. Returns whether it wrote.
pub fn mark(dir: &Path, fingerprint: &str, outcome: &str) -> Result<bool> {
    if outcome != "success" || fingerprint.is_empty() {
        return Ok(false);
    }
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join(MARKER), format!("{fingerprint}\n"))?;
    Ok(true)
}

/// The text that says why the build was skipped.
pub fn report_text(fingerprint: &str, matched_key: &str) -> String {
    format!(
        "fmt, clippy, tests: reused an earlier pass; setup, build and checks skipped\n\
         - governed fingerprint: {fingerprint}\n\
         - matched cache key: {matched_key}\n"
    )
}

fn append(file: &Path, text: &str) -> Result<()> {
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
        .with_context(|| format!("cannot open {}", file.display()))?;
    f.write_all(text.as_bytes())?;
    Ok(())
}

/// The value after `flag`, if the flag is present.
fn flag_value<'a>(args: &'a [String], flag: &str) -> Result<Option<&'a str>> {
    match args.iter().position(|a| a == flag) {
        None => Ok(None),
        Some(i) => match args.get(i + 1) {
            Some(v) => Ok(Some(v.as_str())),
            None => bail!("{flag} needs a value\n{USAGE}"),
        },
    }
}

pub fn command(args: &[String]) -> Result<()> {
    match args {
        [verb, dir, fingerprint, rest @ ..] if verb == "check" => {
            let line = format!("skip={}", should_skip(Path::new(dir), fingerprint));
            println!("{line}");
            if let Some(file) = flag_value(rest, "--output")? {
                append(Path::new(file), &format!("{line}\n"))?;
            }
            Ok(())
        }
        [verb, dir, fingerprint, rest @ ..] if verb == "mark" => {
            let Some(outcome) = flag_value(rest, "--outcome")? else {
                bail!("mark needs --outcome\n{USAGE}");
            };
            if mark(Path::new(dir), fingerprint, outcome)? {
                println!("marked {fingerprint} as passed");
            } else {
                println!("not marked: the outcome was '{outcome}', not 'success'");
            }
            Ok(())
        }
        [verb, fingerprint, key, rest @ ..] if verb == "report" => {
            let text = report_text(fingerprint, key);
            print!("{text}");
            if let Some(file) = flag_value(rest, "--summary")? {
                append(Path::new(file), &text)?;
            }
            Ok(())
        }
        _ => bail!("{USAGE}"),
    }
}
