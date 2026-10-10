//! Nested runs: Athena exercising Athena (design §7.6, D47; M25).
//!
//! The core sets `ATHENA_NESTED=<outer job>` in every agent and check environment. Under that marker, a
//! command that would call agents or commit never touches the repository it was started in: it copies
//! that working tree (committed and uncommitted files, so a producer can exercise its own edits) into a
//! fresh scratch repository, runs there with a scratch state directory and the fake driver, and says
//! where. The real driver refuses to start under the marker, so no model call happens outside a job's
//! records and budget.

use anyhow::{Context, Result};
use athena_git::Git;
use std::path::{Path, PathBuf};

/// Where a nested run works.
#[derive(Debug, Clone)]
pub struct Scratch {
    pub root: PathBuf,
    pub repo: PathBuf,
    pub state: PathBuf,
}

/// Copies the working tree of `source` into a fresh scratch repository with one commit.
pub fn prepare(source: &Git) -> Result<Scratch> {
    let root = match std::env::var_os("ATHENA_SCRATCH_DIR").filter(|v| !v.is_empty()) {
        Some(dir) => {
            std::fs::create_dir_all(&dir)?;
            tempfile::Builder::new()
                .prefix("athena-nested-")
                .tempdir_in(dir)?
                .keep()
        }
        None => tempfile::Builder::new()
            .prefix("athena-nested-")
            .tempdir()?
            .keep(),
    };
    let repo = root.join("repo");
    let state = root.join("state");
    std::fs::create_dir_all(&repo)?;
    std::fs::create_dir_all(&state)?;
    let listing = source.run_bytes(
        [
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ],
        None,
        &[],
    )?;
    for raw in listing.split(|b| *b == 0).filter(|c| !c.is_empty()) {
        let path = String::from_utf8_lossy(raw).into_owned();
        let from = source.dir().join(&path);
        let Ok(meta) = std::fs::symlink_metadata(&from) else {
            continue;
        };
        let to = repo.join(&path);
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if meta.file_type().is_symlink() {
            std::os::unix::fs::symlink(std::fs::read_link(&from)?, &to)?;
        } else if meta.is_file() {
            std::fs::copy(&from, &to).with_context(|| format!("copying {path}"))?;
        }
    }
    let git = Git::at(&repo);
    git.run(["init", "-q", "-b", "nested"])?;
    for (key, value) in [
        ("user.name", "Athena nested run"),
        ("user.email", "noreply@example.invalid"),
        ("commit.gpgsign", "false"),
        ("core.hooksPath", "/dev/null"),
    ] {
        git.run(["config", key, value])?;
    }
    git.run(["add", "-A"])?;
    git.run([
        "commit",
        "-q",
        "--no-verify",
        "-m",
        "A copy of the working tree for a nested Athena run",
    ])?;
    Ok(Scratch { root, repo, state })
}

/// Whether `path` lies inside `dir`.
pub fn is_inside(path: &Path, dir: &Path) -> bool {
    match (path.canonicalize(), dir.canonicalize()) {
        (Ok(p), Ok(d)) => p.starts_with(d),
        _ => false,
    }
}
