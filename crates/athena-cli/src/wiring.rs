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

    /// A program that answers `--version` with `reply` and the exit status `status`.
    fn version_stand_in(dir: &Path, reply: &str, status: u8) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join("claude");
        let script = format!("#!/bin/sh\necho \"{reply}\"\nexit {status}\n");
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    #[test]
    fn deferred_driver_reports_the_version_that_ran_after_it_is_built() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let bin = version_stand_in(dir.path(), "2.1.301 (Claude Code)", 0);
        let driver = DeferredDriver::for_test("2.1.299", Registry::new(state.path()), bin, None);
        assert_eq!(driver.info().version, "2.1.299");
        driver.probe();
        let info = driver.info();
        assert_eq!(info.name, "claude-code");
        assert_eq!(info.version, "2.1.301");
    }

    #[test]
    fn deferred_driver_reports_the_pin_when_the_binary_cannot_report() {
        let dir = tempfile::tempdir().unwrap();
        let state = tempfile::tempdir().unwrap();
        let bin = version_stand_in(dir.path(), "broken", 1);
        let driver = DeferredDriver::for_test("2.1.299", Registry::new(state.path()), bin, None);
        driver.probe();
        assert_eq!(driver.info().version, "2.1.299");
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
    /// The pinned Claude Code version, the tested minimum (D154).
    pinned: String,
    registry: Registry,
    driver: OnceLock<Result<ClaudeCodeDriver, String>>,
    /// Tests give a stand-in binary and a nested-run marker here instead of the real ones.
    #[cfg(test)]
    test_override: Option<(PathBuf, Option<String>)>,
}

impl DeferredDriver {
    pub fn new(pinned: String, registry: Registry) -> Self {
        DeferredDriver {
            pinned,
            registry,
            driver: OnceLock::new(),
            #[cfg(test)]
            test_override: None,
        }
    }

    /// A driver that runs the program at `bin` and reads the nested-run marker as `marker`.
    #[cfg(test)]
    fn for_test(pinned: &str, registry: Registry, bin: PathBuf, marker: Option<String>) -> Self {
        DeferredDriver {
            test_override: Some((bin, marker)),
            ..DeferredDriver::new(pinned.to_string(), registry)
        }
    }

    /// The real driver, built on first call. This is what `invoke` runs first.
    fn inner(&self) -> &Result<ClaudeCodeDriver, String> {
        self.driver.get_or_init(|| {
            let (bin, marker) = (claude_bin(), nested_marker());
            #[cfg(test)]
            let (bin, marker) = self.test_override.clone().unwrap_or((bin, marker));
            ClaudeCodeDriver::with_marker(bin, &self.pinned, self.registry.clone(), marker)
                .map_err(|e| e.to_string())
        })
    }

    /// Builds the real driver now, so `info` reports the version the binary reports.
    #[cfg(test)]
    fn probe(&self) {
        let _ = self.inner();
    }
}

impl AgentDriver for DeferredDriver {
    /// The version the binary reported once the driver has been built, else the pinned version. The
    /// invocation records read this after `invoke`, so they name the version that ran.
    fn info(&self) -> DriverInfo {
        match self.driver.get() {
            Some(Ok(driver)) => driver.info(),
            _ => DriverInfo {
                name: "claude-code".into(),
                version: self.pinned.clone(),
            },
        }
    }

    fn invoke(
        &self,
        spec: &AgentInvocation,
        sink: &mut dyn UsageSink,
    ) -> Result<AgentOutput, DriverError> {
        match self.inner() {
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
