//! Athena's core: deterministic Rust that owns the job lifecycle and holds no model context (D01).
//!
//! A job moves through `ask` (the reader's reading and forecasts, committed), `go` (the prediction
//! commit, the producer in a detached snapshot, the candidate built from its change list, admission, the
//! checks, the independent verifier, and the landing), and closes with its records and score committed
//! under its trailer. Every command that creates records commits them before it exits (D40).
//!
//! Modules:
//! - [`capability`]: the seam between the core and a kind of work;
//! - [`state`]: the per-repository state directory;
//! - [`store`]: records at a commit, and state derived from them;
//! - [`commits`]: records and landing commits built with plumbing, gated in process, fast-forwarded;
//! - [`reading`], [`run`], [`manage`]: the commands;
//! - [`admission`], [`toolconfig`], [`checks`], [`verify`], [`land`]: the verification protocol;
//! - [`scoring`]: forecasts resolved and scored;
//! - [`status`]: the state of the work, from the repository alone;
//! - [`nested`]: Athena running inside one of its own jobs.

pub mod admission;
pub mod capability;
pub mod checks;
pub mod commits;
pub mod invocation;
pub mod land;
pub mod manage;
pub mod nested;
pub mod reading;
pub mod run;
pub mod scoring;
pub mod state;
pub mod status;
pub mod store;
pub mod toolconfig;
pub mod verify;

use anyhow::{Result, anyhow, bail};
use athena_agent::AgentDriver;
use athena_data::records::Build;
use athena_git::Git;
use capability::Capability;
use state::StateDir;
use std::path::PathBuf;

/// Where progress lines go: the terminal for people, a buffer in tests.
pub trait Ui {
    fn line(&self, text: &str);
}

/// A UI that prints nothing.
#[derive(Debug, Default)]
pub struct Quiet;

impl Ui for Quiet {
    fn line(&self, _: &str) {}
}

/// One command's view of Athena: the repository, its state directory, the driver, and the capabilities.
pub struct Athena {
    pub git: Git,
    pub state: StateDir,
    pub driver: Box<dyn AgentDriver>,
    pub capabilities: Vec<Box<dyn Capability>>,
    /// The running binary's version and source.
    pub build: Build,
    /// The binary the agents' hook runs.
    pub athena_bin: PathBuf,
    /// Set in a nested run: the outer job's id.
    pub nested: Option<String>,
}

impl std::fmt::Debug for Athena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Athena")
            .field("git", &self.git)
            .field("state", &self.state)
            .field("driver", &self.driver.info())
            .field(
                "capabilities",
                &self.capabilities.iter().map(|c| c.id()).collect::<Vec<_>>(),
            )
            .field("nested", &self.nested)
            .finish()
    }
}

impl Athena {
    pub fn capability(&self, id: &str) -> Result<&dyn Capability> {
        self.capabilities
            .iter()
            .find(|c| c.id() == id)
            .map(|c| c.as_ref())
            .ok_or_else(|| anyhow!("no capability {id:?} is registered in this build"))
    }

    /// The current commit; Athena needs at least one.
    pub fn head(&self) -> Result<String> {
        match self.git.head()? {
            Some(head) => Ok(head),
            None => bail!(
                "the repository has no commits yet; commit athena.toml and defaults.toml first"
            ),
        }
    }

    /// Refuses to run when the kill switch is on.
    pub fn refuse_if_stopped(&self) -> Result<()> {
        if self.state.registry().stopped() {
            bail!(
                "the kill switch is on: `athena stop` was run. Run `athena start` to allow new work"
            );
        }
        Ok(())
    }
}

/// The current time.
pub fn now() -> jiff::Timestamp {
    jiff::Timestamp::now()
}
