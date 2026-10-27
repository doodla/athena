//! Wiring: the capabilities this build knows, the driver, and the repository Athena works in.

use anyhow::{Context, Result, bail};
use athena_agent::claude::ClaudeCodeDriver;
use athena_agent::fake::FakeAgentDriver;
use athena_agent::procs::Registry;
use athena_agent::{
    AgentDriver, AgentInvocation, AgentOutput, DriverError, UsageSink, nested_marker,
};
use athena_core::Athena;
use athena_core::capability::Capability;
use athena_core::pin::GitPinner;
use athena_core::state::StateDir;
use athena_data::records::{Build, DriverInfo};
use athena_git::Git;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Every capability this build registers. Adding a kind of work means adding it here.
pub fn capabilities() -> Vec<Box<dyn Capability>> {
    vec![
        Box::new(athena_cap_change::Change),
        Box::new(athena_cap_study::Study),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_capabilities_include_change_and_study_without_duplicates() {
        let ids: Vec<&str> = capabilities().iter().map(|c| c.id()).collect();
        assert!(ids.contains(&"change"), "{ids:?}");
        assert!(ids.contains(&"study"), "{ids:?}");
        let unique: std::collections::BTreeSet<&&str> = ids.iter().collect();
        assert_eq!(unique.len(), ids.len(), "duplicate ids: {ids:?}");
    }
}

/// How long `athena land --pr` waits for a pull request's checks: thirty looks, twenty seconds apart.
pub fn checks_poll(sleep: &dyn Fn(std::time::Duration)) -> athena_core::gh::Poll<'_> {
    athena_core::gh::Poll {
        attempts: 30,
        delay: std::time::Duration::from_secs(20),
        sleep,
    }
}

/// The running binary's version and source commit, embedded at build time (D45). A build that names no
/// commit counts as dirty.
pub fn build_info() -> Build {
    let commit = option_env!("ATHENA_BUILD_COMMIT").filter(|c| !c.is_empty());
    Build {
        version: env!("CARGO_PKG_VERSION").to_string(),
        source_commit: commit.unwrap_or("unknown").to_string(),
        dirty: commit.is_none() || option_env!("ATHENA_BUILD_DIRTY") != Some("false"),
    }
}

/// Where setup installs Athena (`$ATHENA_HOME/bin/athena`). After a rebuild this is the new binary, whereas
/// the running one may have been replaced on disk.
pub fn installed_binary() -> PathBuf {
    athena_data::paths::athena_home().join("bin/athena")
}

/// The text `athena --version` shows after the name: the package version, the source commit, and a
/// `dirty` marker when the build does not name a clean commit.
pub fn version_string(build: &Build) -> String {
    let dirty = if build.dirty { ", dirty" } else { "" };
    format!("{} (commit {}{dirty})", build.version, build.source_commit)
}

/// The Claude Code binary setup chose: `$ATHENA_HOME/config.toml`'s `claude_bin`, else `claude` on `PATH`.
pub fn claude_bin() -> PathBuf {
    let config = athena_data::paths::athena_home().join("config.toml");
    if let Ok(text) = std::fs::read_to_string(&config)
        && let Ok(value) = text.parse::<toml::Table>()
        && let Some(bin) = value.get("claude_bin").and_then(|v| v.as_str())
    {
        return PathBuf::from(bin);
    }
    PathBuf::from("claude")
}

/// The real driver, created on first use, so commands that run no agent never start Claude Code.
#[derive(Debug)]
pub struct DeferredDriver {
    pinned: String,
    registry: Registry,
    driver: OnceLock<Result<ClaudeCodeDriver, String>>,
}

impl DeferredDriver {
    pub fn new(pinned: String, registry: Registry) -> Self {
        DeferredDriver {
            pinned,
            registry,
            driver: OnceLock::new(),
        }
    }
}

impl AgentDriver for DeferredDriver {
    fn info(&self) -> DriverInfo {
        DriverInfo {
            name: "claude-code".into(),
            version: self.pinned.clone(),
        }
    }

    fn invoke(
        &self,
        spec: &AgentInvocation,
        sink: &mut dyn UsageSink,
    ) -> Result<AgentOutput, DriverError> {
        let driver = self.driver.get_or_init(|| {
            ClaudeCodeDriver::new(claude_bin(), &self.pinned, self.registry.clone())
                .map_err(|e| e.to_string())
        });
        match driver {
            Ok(driver) => driver.invoke(spec, sink),
            Err(message) => Err(DriverError::Spawn(message.clone())),
        }
    }
}

/// Opens Athena for the repository containing `cwd`. Under the nested-run marker, commands that would
/// call agents or commit (`writes`) run in a scratch copy with the fake driver instead (D47).
pub fn open(cwd: &Path, writes: bool, announce: &dyn Fn(&str)) -> Result<Athena> {
    let (major, minor) = Git::version()?;
    if (major, minor) < athena_git::MIN_GIT_VERSION {
        bail!(
            "git {major}.{minor} is too old; Athena needs 2.40 or later (run ./setup.sh to check prerequisites)"
        );
    }
    let user = Git::discover(cwd).context("Athena runs inside a git repository")?;
    athena_git::attrs::refuse_info_attributes(&user)?;
    let athena_bin = std::env::current_exe().context("locating the athena binary")?;
    let build = build_info();
    if let Some(outer) = nested_marker()
        && writes
    {
        let scratch = athena_core::nested::prepare(&user)?;
        announce(&format!(
            "nested run (ATHENA_NESTED={outer}): working in a scratch copy at {} with fake agents; the repository it was started in is untouched",
            scratch.repo.display()
        ));
        let state = StateDir::new(scratch.state.clone());
        let driver = FakeAgentDriver::deterministic().with_registry(state.registry());
        return Ok(Athena {
            git: Git::at(&scratch.repo),
            state,
            driver: std::sync::Arc::new(driver),
            capabilities: capabilities(),
            build,
            athena_bin,
            nested: Some(outer),
            pinner: Box::new(GitPinner),
            jev: None,
        });
    }
    let state = StateDir::for_repository(&user.common_dir()?);
    // The driver only needs the pinned Claude Code version, and is built when an agent first runs. A
    // configuration that does not parse must not stop `check` and `status` from reporting it; commands
    // that run agents read the configuration again, and refuse it there.
    let config = match athena_gate::config_at(&user, user.head()?.as_deref()) {
        Ok(config) => config,
        Err(athena_gate::GateError::Config(_)) if !writes => athena_gate::builtin_config(),
        Err(other) => return Err(other.into()),
    };
    let driver = DeferredDriver::new(config.agents.claude_version.clone(), state.registry());
    Ok(Athena {
        git: user,
        state,
        driver: std::sync::Arc::new(driver),
        capabilities: capabilities(),
        build,
        athena_bin,
        nested: nested_marker(),
        pinner: Box::new(GitPinner),
        jev: athena_core::jev::shadow_from_env(),
    })
}
