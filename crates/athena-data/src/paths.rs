//! Where Athena keeps machine-local things (design §2.4).
//!
//! - The install directory, `$ATHENA_HOME` (default `${XDG_DATA_HOME:-~/.local/share}/athena`), holds the
//!   adopted `athena` binary and pinned tools.
//! - The state directory, `$ATHENA_STATE_DIR` (default `${XDG_STATE_HOME:-~/.local/state}/athena`), holds a
//!   subdirectory per repository with snapshots, live records, locks, and the kill switch.
//!
//! Nothing here is committed. Once a job closes, nothing that matters lives only on the machine.

use std::path::{Path, PathBuf};

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn env_dir(key: &str) -> Option<PathBuf> {
    std::env::var_os(key)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

/// The install directory.
pub fn athena_home() -> PathBuf {
    env_dir("ATHENA_HOME").unwrap_or_else(|| {
        env_dir("XDG_DATA_HOME")
            .unwrap_or_else(|| home().join(".local/share"))
            .join("athena")
    })
}

/// The root of all per-repository state directories.
pub fn state_root() -> PathBuf {
    env_dir("ATHENA_STATE_DIR").unwrap_or_else(|| {
        env_dir("XDG_STATE_HOME")
            .unwrap_or_else(|| home().join(".local/state"))
            .join("athena")
    })
}

/// The state directory for one repository, keyed by its git common directory.
pub fn repo_state_dir(common_git_dir: &Path) -> PathBuf {
    let canonical = common_git_dir
        .canonicalize()
        .unwrap_or_else(|_| common_git_dir.to_path_buf());
    let text = canonical.to_string_lossy();
    let name = canonical
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "repo".into());
    let key = &crate::sha256_hex(text.as_bytes())[..12];
    state_root().join(format!("{name}-{key}"))
}

/// The path of a pinned tool's binary inside the install directory.
pub fn tool_path(tool: &str, version: &str) -> PathBuf {
    athena_home()
        .join("tools")
        .join(tool)
        .join(version)
        .join(tool)
}

/// The install root of a pinned sccache (the `--root` given to `cargo install`).
pub fn sccache_root(home: &Path, version: &str) -> PathBuf {
    home.join("tools").join("sccache").join(version)
}

/// The sccache binary for the one pinned version, under the given install directory. Setup installs to
/// this path and the wrapper setting derives from it, so the two cannot name different versions.
pub fn sccache_binary_in(home: &Path, version: &str) -> PathBuf {
    sccache_root(home, version).join("bin").join("sccache")
}

/// The sccache binary for the pinned version under this machine's install directory.
pub fn sccache_binary(version: &str) -> PathBuf {
    sccache_binary_in(&athena_home(), version)
}

/// The platform key used in tool pins, such as `linux_x64`.
pub fn platform_key() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Some("linux_x64"),
        ("linux", "aarch64") => Some("linux_arm64"),
        ("macos", "x86_64") => Some("darwin_x64"),
        ("macos", "aarch64") => Some("darwin_arm64"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_paths_are_versioned() {
        let path = tool_path("gitleaks", "8.30.1");
        assert!(path.ends_with("tools/gitleaks/8.30.1/gitleaks"));
    }

    #[test]
    fn the_sccache_binary_derives_from_the_one_pinned_version() {
        let home = Path::new("/h/athena");
        let binary = sccache_binary_in(home, "0.10.0");
        assert_eq!(
            binary,
            Path::new("/h/athena/tools/sccache/0.10.0/bin/sccache")
        );
        // The binary lies under the install root that `cargo install --root` receives.
        assert!(binary.starts_with(sccache_root(home, "0.10.0")));
        assert_ne!(binary, sccache_binary_in(home, "0.11.0"));
    }

    #[test]
    fn repo_state_dirs_are_keyed_by_repository() {
        let a = repo_state_dir(Path::new("/nonexistent/one/.git"));
        let b = repo_state_dir(Path::new("/nonexistent/two/.git"));
        assert_ne!(a, b);
        assert!(a.file_name().unwrap().to_string_lossy().starts_with("one-"));
    }
}
