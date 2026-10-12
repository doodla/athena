//! Pinning: the core resolves a study's repository ref to one commit, and mounts that tree read-only.
//!
//! A study reads a system "at one pinned commit". The core, not an agent, picks that commit: [`Pinner`]
//! turns the URL and ref from the capability's configuration into a full 40-hex commit id and checks the
//! tree out. [`mount`] then copies the tree into an agent's working directory as read-only files.
//!
//! Read-only is enforced by file permissions (every file and directory loses its write bits, so nothing in
//! the tree can be edited, created, or removed by a process that honors them) plus the agents' own policy;
//! it is not an operating-system mount. The mount directory is listed in the working directory's git
//! `info/exclude`, so it never appears in a candidate's change list.

use anyhow::{Context, Result, anyhow, bail};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The directory, inside an agent's working directory, that holds the pinned tree.
pub const MOUNT_DIR: &str = "pinned-source";

/// Resolves a repository ref to a commit and checks that commit's tree out.
pub trait Pinner: Send + Sync {
    /// Resolves `git_ref` of the repository at `url` to a full 40-hex commit id, and writes that commit's
    /// tree (without any `.git`) into `dest`, which must not exist yet. Returns the commit id.
    fn pin(&self, url: &str, git_ref: &str, dest: &Path) -> Result<String>;
}

/// The real pinner: `git ls-remote` to resolve, then a shallow fetch of that commit and a checkout.
#[derive(Debug, Default, Clone, Copy)]
pub struct GitPinner;

/// Whether `text` is a full lowercase 40-hex commit id.
pub fn is_full_sha(text: &str) -> bool {
    text.len() == 40 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn git(dir: Option<&Path>, args: &[&str]) -> Result<String> {
    let mut command = Command::new("git");
    if let Some(dir) = dir {
        command.arg("-C").arg(dir);
    }
    let output = command
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .output()
        .context("running git")?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.first().copied().unwrap_or(""),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Picks the commit `git_ref` names from `git ls-remote` output (annotated tags use their peeled commit).
fn pick(listing: &str, git_ref: &str) -> Option<String> {
    let wanted = [
        git_ref.to_string(),
        format!("refs/heads/{git_ref}"),
        format!("refs/tags/{git_ref}^{{}}"),
        format!("refs/tags/{git_ref}"),
    ];
    let lines: Vec<(&str, &str)> = listing.lines().filter_map(|l| l.split_once('\t')).collect();
    wanted.iter().find_map(|name| {
        lines
            .iter()
            .find(|(sha, n)| n == name && is_full_sha(sha))
            .map(|(sha, _)| sha.to_string())
    })
}

impl Pinner for GitPinner {
    fn pin(&self, url: &str, git_ref: &str, dest: &Path) -> Result<String> {
        for (what, text) in [("repository URL", url), ("ref", git_ref)] {
            if text.trim().is_empty() || text.starts_with('-') {
                bail!("the {what} {text:?} is not usable");
            }
        }
        if dest.exists() {
            bail!("pin destination {} already exists", dest.display());
        }
        let sha = if is_full_sha(git_ref) {
            git_ref.to_string()
        } else {
            let listing = git(None, &["ls-remote", url, git_ref])?;
            pick(&listing, git_ref)
                .ok_or_else(|| anyhow!("the ref {git_ref:?} was not found in {url}"))?
        };
        let scratch = tempfile::tempdir().context("creating a scratch repository")?;
        let repo = scratch.path();
        git(Some(repo), &["init", "-q"])?;
        git(
            Some(repo),
            &["fetch", "-q", "--depth", "1", "--no-tags", url, &sha],
        )
        .with_context(|| format!("fetching {sha} from {url}"))?;
        // An annotated tag resolves to its own object; the pin is the commit it points at.
        let fetched = git(Some(repo), &["rev-parse", "FETCH_HEAD^{commit}"])?
            .trim()
            .to_string();
        if !is_full_sha(&fetched) || (is_full_sha(git_ref) && fetched != sha) {
            bail!("fetched {fetched:?} where {git_ref} was expected");
        }
        let sha = fetched;
        std::fs::create_dir_all(dest)?;
        let prefix = format!("--prefix={}/", dest.display());
        git(Some(repo), &["read-tree", "FETCH_HEAD"])
            .and_then(|_| git(Some(repo), &["checkout-index", "-a", &prefix]))
            .with_context(|| format!("checking out {sha}"))?;
        Ok(sha)
    }
}

/// Copies the pinned tree at `src` into `workdir/MOUNT_DIR`, hides it from the working directory's git,
/// and makes it read-only. Returns the mount path.
pub fn mount(src: &Path, workdir: &Path) -> Result<PathBuf> {
    let dest = workdir.join(MOUNT_DIR);
    if dest.exists() {
        bail!("{} already exists", dest.display());
    }
    copy_tree(src, &dest)?;
    let info = workdir.join(".git").join("info");
    if info.is_dir() {
        let exclude = info.join("exclude");
        let mut text = std::fs::read_to_string(&exclude).unwrap_or_default();
        text.push_str(&format!("/{MOUNT_DIR}/\n"));
        std::fs::write(&exclude, text)?;
    }
    make_read_only(&dest)?;
    Ok(dest)
}

fn copy_tree(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        let kind = entry.file_type()?;
        if kind.is_dir() {
            copy_tree(&from, &to)?;
        } else if kind.is_symlink() {
            copy_symlink(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn copy_symlink(from: &Path, to: &Path) -> Result<()> {
    std::os::unix::fs::symlink(std::fs::read_link(from)?, to)?;
    Ok(())
}

#[cfg(not(unix))]
fn copy_symlink(_from: &Path, _to: &Path) -> Result<()> {
    Ok(())
}

/// Removes the write bits of every file and directory under `path`, and of `path` itself.
pub fn make_read_only(path: &Path) -> Result<()> {
    let meta = std::fs::symlink_metadata(path)?;
    if meta.file_type().is_symlink() {
        return Ok(());
    }
    if meta.is_dir() {
        for entry in std::fs::read_dir(path)? {
            make_read_only(&entry?.path())?;
        }
    }
    set_writable(path, false)
}

#[cfg(unix)]
fn set_writable(path: &Path, writable: bool) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(path)?.permissions();
    let mode = permissions.mode();
    permissions.set_mode(if writable {
        mode | 0o200
    } else {
        mode & !0o222
    });
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_writable(path: &Path, writable: bool) -> Result<()> {
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_readonly(!writable);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

fn make_writable(path: &Path) {
    let Ok(meta) = std::fs::symlink_metadata(path) else {
        return;
    };
    if meta.file_type().is_symlink() {
        return;
    }
    let _ = set_writable(path, true);
    if meta.is_dir()
        && let Ok(entries) = std::fs::read_dir(path)
    {
        for entry in entries.flatten() {
            make_writable(&entry.path());
        }
    }
}

/// Removes a directory tree that may hold read-only mounts. Missing paths are fine.
pub fn remove_tree(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    make_writable(path);
    std::fs::remove_dir_all(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(dir: &Path, args: &[&str]) -> String {
        git(Some(dir), args).unwrap()
    }

    /// A local repository with two commits on `main` and an annotated tag on the first; returns both ids.
    fn origin() -> (tempfile::TempDir, String, String) {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        run(p, &["init", "-q", "-b", "main"]);
        for (k, v) in [
            ("user.name", "Test User"),
            ("user.email", "noreply@example.com"),
            ("commit.gpgsign", "false"),
            ("tag.gpgsign", "false"),
        ] {
            run(p, &["config", k, v]);
        }
        std::fs::create_dir_all(p.join("src")).unwrap();
        std::fs::write(p.join("src/lib.rs"), "// one\n").unwrap();
        run(p, &["add", "-A"]);
        run(p, &["commit", "-q", "-m", "one"]);
        let first = run(p, &["rev-parse", "HEAD"]).trim().to_string();
        run(p, &["tag", "-a", "-m", "release", "v1"]);
        std::fs::write(p.join("src/lib.rs"), "// two\n").unwrap();
        run(p, &["commit", "-q", "-am", "two"]);
        let second = run(p, &["rev-parse", "HEAD"]).trim().to_string();
        (dir, first, second)
    }

    fn url(dir: &tempfile::TempDir) -> String {
        format!("file://{}", dir.path().display())
    }

    #[test]
    fn a_branch_a_tag_and_a_commit_resolve_to_their_commits() {
        let (dir, first, second) = origin();
        let out = tempfile::tempdir().unwrap();
        for (git_ref, expected, text) in [
            ("main", &second, "// two\n"),
            ("v1", &first, "// one\n"),
            (first.as_str(), &first, "// one\n"),
        ] {
            let dest = out.path().join(format!("t-{}", &expected[..8]));
            let _ = remove_tree(&dest);
            let sha = GitPinner.pin(&url(&dir), git_ref, &dest).unwrap();
            assert_eq!(&sha, expected, "ref {git_ref}");
            assert!(is_full_sha(&sha));
            assert_eq!(
                std::fs::read_to_string(dest.join("src/lib.rs")).unwrap(),
                text
            );
            assert!(!dest.join(".git").exists(), "the tree carries no history");
        }
    }

    #[test]
    fn a_missing_ref_is_an_error() {
        let (dir, _, _) = origin();
        let out = tempfile::tempdir().unwrap();
        let dest = out.path().join("tree");
        let error = GitPinner
            .pin(&url(&dir), "no-such-branch", &dest)
            .unwrap_err();
        assert!(error.to_string().contains("not found"), "{error:#}");
        assert!(!dest.exists());
        assert!(GitPinner.pin(&url(&dir), "--upload-pack=x", &dest).is_err());
    }

    #[test]
    fn the_mount_is_read_only_and_invisible_to_git() {
        let (dir, _, _) = origin();
        let out = tempfile::tempdir().unwrap();
        let src = out.path().join("pin");
        GitPinner.pin(&url(&dir), "main", &src).unwrap();
        let work = out.path().join("work");
        std::fs::create_dir_all(work.join(".git/info")).unwrap();
        let mounted = mount(&src, &work).unwrap();
        assert_eq!(mounted, work.join(MOUNT_DIR));
        assert_eq!(
            std::fs::read_to_string(mounted.join("src/lib.rs")).unwrap(),
            "// two\n"
        );
        for path in [
            mounted.clone(),
            mounted.join("src"),
            mounted.join("src/lib.rs"),
        ] {
            assert!(
                std::fs::metadata(&path).unwrap().permissions().readonly(),
                "{} is read-only",
                path.display()
            );
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            for path in [mounted.join("src"), mounted.join("src/lib.rs")] {
                let mode = std::fs::metadata(&path).unwrap().permissions().mode();
                assert_eq!(mode & 0o222, 0, "{} has no write bit", path.display());
            }
        }
        let exclude = std::fs::read_to_string(work.join(".git/info/exclude")).unwrap();
        assert!(exclude.contains(&format!("/{MOUNT_DIR}/")));
        remove_tree(&work).unwrap();
        assert!(!work.exists(), "a read-only mount can still be cleaned up");
    }
}
