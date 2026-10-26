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

thread_local! {
    static STATE_ROOT_OVERRIDE: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// Runs `f` with [`state_root`] answering `root` on this thread only. Tests use it to keep a records worktree
/// under a temporary directory without changing the process environment, which the workspace forbids doing
/// from safe code and which parallel tests would share.
pub fn with_state_root<T>(root: &Path, f: impl FnOnce() -> T) -> T {
    let previous = STATE_ROOT_OVERRIDE.with(|o| o.borrow_mut().replace(root.to_path_buf()));
    struct Restore(Option<PathBuf>);
    impl Drop for Restore {
        fn drop(&mut self) {
            let previous = self.0.take();
            STATE_ROOT_OVERRIDE.with(|o| *o.borrow_mut() = previous);
        }
    }
    let _restore = Restore(previous);
    f()
}

/// The root of all per-repository state directories.
pub fn state_root() -> PathBuf {
    if let Some(root) = STATE_ROOT_OVERRIDE.with(|o| o.borrow().clone()) {
        return root;
    }
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

/// Where the checkout of the records branch `athena/records` lives, inside a repository's state directory
/// (`<state root>/<name>-<key>/records-worktree`, D111).
pub fn records_worktree_dir(repo_state_dir: &Path) -> PathBuf {
    repo_state_dir.join("records-worktree")
}

/// The records worktree of the repository whose common git directory is `common_git_dir`, when it exists and
/// is a checkout of `athena/records`; `None` otherwise. Readers of records use it when present and fall back to
/// the in-tree `records/` (or the commit they were given) when it is `None`.
pub fn records_worktree(common_git_dir: &Path) -> Option<PathBuf> {
    let dir = records_worktree_dir(&repo_state_dir(common_git_dir));
    is_records_checkout(&dir).then_some(dir)
}

/// Whether `dir` is a git work tree whose `HEAD` is the branch `athena/records`. A linked work tree has `.git`
/// as a file naming its git directory; that directory's `HEAD` is read.
fn is_records_checkout(dir: &Path) -> bool {
    let dot_git = dir.join(".git");
    let git_dir = if dot_git.is_file() {
        let Ok(text) = std::fs::read_to_string(&dot_git) else {
            return false;
        };
        let Some(target) = text.trim().strip_prefix("gitdir:") else {
            return false;
        };
        let target = PathBuf::from(target.trim());
        if target.is_absolute() {
            target
        } else {
            dir.join(target)
        }
    } else if dot_git.is_dir() {
        dot_git
    } else {
        return false;
    };
    std::fs::read_to_string(git_dir.join("HEAD"))
        .is_ok_and(|head| head.trim() == "ref: refs/heads/athena/records")
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
    fn a_records_checkout_is_recognised_by_its_head() {
        let dir = std::env::temp_dir().join(format!("athena-paths-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let admin = dir.join("admin");
        std::fs::create_dir_all(&admin).unwrap();
        let wt = dir.join("wt");
        std::fs::create_dir_all(&wt).unwrap();
        assert!(!is_records_checkout(&wt), "no .git at all");
        std::fs::write(wt.join(".git"), format!("gitdir: {}\n", admin.display())).unwrap();
        std::fs::write(admin.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        assert!(!is_records_checkout(&wt), "a checkout of another branch");
        std::fs::write(admin.join("HEAD"), "ref: refs/heads/athena/records\n").unwrap();
        assert!(is_records_checkout(&wt));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_records_worktree_lies_in_the_repository_state_dir() {
        let common = Path::new("/nonexistent/three/.git");
        assert_eq!(
            records_worktree_dir(&repo_state_dir(common)),
            repo_state_dir(common).join("records-worktree")
        );
        assert_eq!(records_worktree(common), None);
    }

    #[test]
    fn repo_state_dirs_are_keyed_by_repository() {
        let a = repo_state_dir(Path::new("/nonexistent/one/.git"));
        let b = repo_state_dir(Path::new("/nonexistent/two/.git"));
        assert_ne!(a, b);
        assert!(a.file_name().unwrap().to_string_lossy().starts_with("one-"));
    }
}
