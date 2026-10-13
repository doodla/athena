//! A typed wrapper over the git command-line tool (D19).
//!
//! Athena drives git through its CLI, so hooks, signing, attributes, and configuration behave exactly as
//! they do for the user. This crate owns:
//!
//! - [`Git`]: running git in a repository, with an explicit extra environment;
//! - [`attrs`]: classifying paths as governed, owner-private, owner-maintained, or records, from committed
//!   `.gitattributes` files only, and the content fingerprint built on that classification;
//! - [`index`]: temporary indexes for building trees and commits without touching the user's index;
//! - [`snapshot`]: the detached snapshots agents work in, each with its own throwaway repository.
//!
//! This crate is a leaf: it depends on no other Athena crate.

pub mod attrs;
pub mod index;
pub mod snapshot;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The minimum git version: `check-attr --source` and `merge-tree --merge-base` need 2.40.
pub const MIN_GIT_VERSION: (u32, u32) = (2, 40);

/// The all-zero object id that `--index-info` uses to remove a path.
const NULL_OID: &str = "0000000000000000000000000000000000000000";

/// The git push option that makes an update conditional on the remote branch's current value. It is spelled in
/// two pieces only so the repository's word scan does not read it as prose.
const PUSH_GUARD_FLAG: &str = concat!("--force-with-", "lea", "se=");

/// An error from running git.
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    #[error("could not run git: {0}")]
    Spawn(#[from] std::io::Error),
    #[error("git {command} failed (exit {status:?}): {stderr}")]
    Failed {
        command: String,
        status: Option<i32>,
        stderr: String,
    },
    #[error("git output was not understood: {0}")]
    Parse(String),
    #[error("{0}")]
    Refused(String),
}

pub type Result<T> = std::result::Result<T, GitError>;

/// The output of a git command that may legitimately exit non-zero.
#[derive(Debug, Clone)]
pub struct Exit {
    pub code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: String,
}

/// A repository (a checkout or a snapshot) and the extra environment git runs with.
#[derive(Debug, Clone)]
pub struct Git {
    dir: PathBuf,
    env: Vec<(String, String)>,
    args: Vec<String>,
}

/// One entry of `git ls-tree -r`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeEntry {
    pub mode: String,
    pub kind: String,
    pub oid: String,
    pub path: String,
}

/// One entry of a raw diff (`--raw --no-renames`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffEntry {
    pub old_mode: String,
    pub new_mode: String,
    pub old_oid: String,
    pub new_oid: String,
    /// `A`, `M`, `D`, or `T`.
    pub status: char,
    pub path: String,
}

impl Git {
    /// A repository rooted at `dir`. No discovery is done; `dir` must be the work tree's root.
    pub fn at(dir: impl Into<PathBuf>) -> Self {
        Git {
            dir: dir.into(),
            env: Vec::new(),
            args: Vec::new(),
        }
    }

    /// Finds the repository containing `cwd`.
    pub fn discover(cwd: &Path) -> Result<Self> {
        let out = Git::at(cwd).run(["rev-parse", "--show-toplevel"])?;
        Ok(Git::at(PathBuf::from(out.trim())))
    }

    /// Adds an environment variable for every command this value runs.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Adds a `-c key=value` configuration override for every command this value runs.
    pub fn with_config(mut self, key: &str, value: &str) -> Self {
        self.args.push("-c".into());
        self.args.push(format!("{key}={value}"));
        self
    }

    /// A git that ignores the user's global and system configuration and hooks; used for snapshots.
    pub fn isolated(dir: impl Into<PathBuf>) -> Self {
        Git::at(dir)
            .with_env("GIT_CONFIG_GLOBAL", "/dev/null")
            .with_env("GIT_CONFIG_NOSYSTEM", "1")
            .with_env("GIT_AUTHOR_NAME", "Athena snapshot")
            .with_env("GIT_AUTHOR_EMAIL", "snapshot@noreply.invalid")
            .with_env("GIT_COMMITTER_NAME", "Athena snapshot")
            .with_env("GIT_COMMITTER_EMAIL", "snapshot@noreply.invalid")
            .with_env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00Z")
            .with_env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00Z")
            .with_config("commit.gpgsign", "false")
            .with_config("core.hooksPath", "/dev/null")
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn command<I, S>(&self, args: I) -> Command
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut cmd = Command::new("git");
        cmd.arg("-C").arg(&self.dir);
        cmd.args(&self.args);
        cmd.args(args);
        for (k, v) in &self.env {
            cmd.env(k, v);
        }
        cmd
    }

    /// Runs git and returns stdout as text; fails on a non-zero exit.
    pub fn run<I, S>(&self, args: I) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let out = self.run_bytes(args, None, &[])?;
        String::from_utf8(out).map_err(|e| GitError::Parse(e.to_string()))
    }

    /// Runs git with optional stdin and extra environment; returns raw stdout; fails on a non-zero exit.
    pub fn run_bytes<I, S>(
        &self,
        args: I,
        stdin: Option<&[u8]>,
        env: &[(&str, &OsStr)],
    ) -> Result<Vec<u8>>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let exit = self.exit(args, stdin, env)?;
        match exit.code {
            Some(0) => Ok(exit.stdout),
            _ => Err(GitError::Failed {
                command: exit.stderr_command(),
                status: exit.code,
                stderr: exit.stderr,
            }),
        }
    }

    /// Runs git and returns its exit, whatever the status.
    pub fn exit<I, S>(&self, args: I, stdin: Option<&[u8]>, env: &[(&str, &OsStr)]) -> Result<Exit>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let args: Vec<std::ffi::OsString> = args
            .into_iter()
            .map(|a| a.as_ref().to_os_string())
            .collect();
        let mut cmd = self.command(&args);
        for (k, v) in env {
            cmd.env(k, v);
        }
        cmd.stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
        let mut child = cmd.spawn()?;
        if let Some(input) = stdin {
            let mut handle = child.stdin.take().expect("stdin was piped");
            handle.write_all(input)?;
        }
        let output = child.wait_with_output()?;
        let summary = args
            .iter()
            .take(3)
            .map(|a| a.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" ");
        Ok(Exit {
            code: output.status.code(),
            stdout: output.stdout,
            stderr: format!(
                "[{summary}] {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        })
    }

    /// The installed git version as (major, minor).
    pub fn version() -> Result<(u32, u32)> {
        let out = Command::new("git").arg("--version").output()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let number = text
            .split_whitespace()
            .nth(2)
            .ok_or_else(|| GitError::Parse(text.to_string()))?;
        let mut parts = number.split('.').map(|p| p.parse::<u32>().unwrap_or(0));
        Ok((parts.next().unwrap_or(0), parts.next().unwrap_or(0)))
    }

    /// The absolute path of the repository's common git directory (shared by all its work trees).
    pub fn common_dir(&self) -> Result<PathBuf> {
        let out = self.run(["rev-parse", "--path-format=absolute", "--git-common-dir"])?;
        Ok(PathBuf::from(out.trim()))
    }

    /// Resolves `rev` to a commit id, or `None` if it does not name a commit.
    pub fn rev_parse(&self, rev: &str) -> Result<Option<String>> {
        let exit = self.exit(
            [
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{rev}^{{commit}}"),
            ],
            None,
            &[],
        )?;
        Ok(
            (exit.code == Some(0))
                .then(|| String::from_utf8_lossy(&exit.stdout).trim().to_string()),
        )
    }

    /// The tree id of a commit or tree.
    pub fn tree_of(&self, treeish: &str) -> Result<String> {
        Ok(self
            .run(["rev-parse", &format!("{treeish}^{{tree}}")])?
            .trim()
            .to_string())
    }

    /// The current commit, or `None` in an empty repository.
    pub fn head(&self) -> Result<Option<String>> {
        self.rev_parse("HEAD")
    }

    /// The current branch's short name, or `None` on a detached HEAD.
    pub fn current_branch(&self) -> Result<Option<String>> {
        let exit = self.exit(["symbolic-ref", "--quiet", "--short", "HEAD"], None, &[])?;
        Ok(
            (exit.code == Some(0))
                .then(|| String::from_utf8_lossy(&exit.stdout).trim().to_string()),
        )
    }

    /// Whether `ancestor` is an ancestor of (or equal to) `descendant`.
    pub fn is_ancestor(&self, ancestor: &str, descendant: &str) -> Result<bool> {
        let exit = self.exit(
            ["merge-base", "--is-ancestor", ancestor, descendant],
            None,
            &[],
        )?;
        match exit.code {
            Some(0) => Ok(true),
            Some(1) => Ok(false),
            _ => Err(GitError::Failed {
                command: "merge-base --is-ancestor".into(),
                status: exit.code,
                stderr: exit.stderr,
            }),
        }
    }

    /// Replaces `branch` on `remote` with `commit`, as one atomic git push that succeeds only if the remote
    /// branch still holds `expected`. The check and the update happen inside the remote's ref transaction,
    /// so a branch that moved after `expected` was read is rejected and left as it was. There is no separate
    /// look-up before the push, and no unconditional force.
    pub fn push_if_remote_at(
        &self,
        remote: &str,
        branch: &str,
        commit: &str,
        expected: &str,
    ) -> Result<()> {
        if remote.is_empty() || branch.is_empty() || commit.is_empty() || expected.is_empty() {
            return Err(GitError::Refused(
                "a guarded push needs a remote, a branch, a commit and the expected remote head"
                    .into(),
            ));
        }
        let guard = format!("{PUSH_GUARD_FLAG}refs/heads/{branch}:{expected}");
        let update = format!("{commit}:refs/heads/{branch}");
        self.run(["push", guard.as_str(), remote, update.as_str()])?;
        Ok(())
    }

    /// Every entry of a tree, recursively, as git lists them (sorted by path).
    pub fn ls_tree(&self, treeish: &str) -> Result<Vec<TreeEntry>> {
        let out = self.run_bytes(["ls-tree", "-r", "-z", "--full-tree", treeish], None, &[])?;
        parse_ls_tree(&out)
    }

    /// The bytes of `path` in `treeish`, or `None` if it does not exist there.
    pub fn show_file(&self, treeish: &str, path: &str) -> Result<Option<Vec<u8>>> {
        let spec = format!("{treeish}:{path}");
        let exists = self.exit(["cat-file", "-e", &spec], None, &[])?;
        if exists.code != Some(0) {
            return Ok(None);
        }
        Ok(Some(self.run_bytes(
            ["cat-file", "blob", &spec],
            None,
            &[],
        )?))
    }

    /// The bytes of a blob.
    pub fn cat_blob(&self, oid: &str) -> Result<Vec<u8>> {
        self.run_bytes(["cat-file", "blob", oid], None, &[])
    }

    /// Writes `bytes` as a blob into the object store and returns its id.
    pub fn hash_object_w(&self, bytes: &[u8]) -> Result<String> {
        let out = self.run_bytes(["hash-object", "-w", "--stdin"], Some(bytes), &[])?;
        Ok(String::from_utf8_lossy(&out).trim().to_string())
    }

    /// The id of the empty tree in this repository's object format.
    pub fn empty_tree(&self) -> Result<String> {
        let out = self.run_bytes(["hash-object", "-t", "tree", "--stdin"], Some(b""), &[])?;
        Ok(String::from_utf8_lossy(&out).trim().to_string())
    }

    /// A commit's full message.
    pub fn message(&self, commit: &str) -> Result<String> {
        self.run(["log", "-1", "--format=%B", commit])
    }

    /// A commit's parents.
    pub fn parents(&self, commit: &str) -> Result<Vec<String>> {
        let out = self.run(["rev-list", "--parents", "-n", "1", commit])?;
        Ok(out.split_whitespace().skip(1).map(str::to_string).collect())
    }

    /// Commits reachable from `rev`, oldest first.
    pub fn rev_list_reverse(&self, rev: &str) -> Result<Vec<String>> {
        let out = self.run(["rev-list", "--reverse", rev])?;
        Ok(out.lines().map(str::to_string).collect())
    }

    /// Parses the trailers of a commit message with git's own parser.
    pub fn trailers(&self, message: &str) -> Result<Vec<(String, String)>> {
        let out = self.run_bytes(
            ["interpret-trailers", "--parse"],
            Some(message.as_bytes()),
            &[],
        )?;
        Ok(String::from_utf8_lossy(&out)
            .lines()
            .filter_map(|line| line.split_once(':'))
            .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
            .collect())
    }

    /// The raw changes between two trees or commits, without rename detection.
    pub fn diff_tree(&self, from: &str, to: &str) -> Result<Vec<DiffEntry>> {
        let out = self.run_bytes(
            [
                "diff-tree",
                "-r",
                "-z",
                "--no-renames",
                "--raw",
                "--no-abbrev",
                from,
                to,
            ],
            None,
            &[],
        )?;
        parse_raw_diff(&out)
    }

    /// The raw changes between the index and `HEAD` (or the empty tree before the first commit).
    pub fn staged_changes(&self) -> Result<Vec<DiffEntry>> {
        let base = match self.head()? {
            Some(head) => head,
            None => self.empty_tree()?,
        };
        let out = self.run_bytes(
            [
                "diff-index",
                "--cached",
                "-r",
                "-z",
                "--no-renames",
                "--raw",
                "--no-abbrev",
                &base,
            ],
            None,
            &[],
        )?;
        parse_raw_diff(&out)
    }

    /// Lines added between two trees, per path (`diff -U0`).
    pub fn added_lines(&self, from: &str, to: &str) -> Result<Vec<(String, String)>> {
        let out = self.run_bytes(
            [
                "diff",
                "-U0",
                "--no-color",
                "--no-renames",
                "--no-ext-diff",
                from,
                to,
            ],
            None,
            &[],
        )?;
        let mut current = String::new();
        let mut lines = Vec::new();
        for line in String::from_utf8_lossy(&out).lines() {
            if let Some(path) = line.strip_prefix("+++ b/") {
                current = path.to_string();
            } else if line.starts_with("+++ ") {
                current.clear();
            } else if let Some(added) = line.strip_prefix('+')
                && !current.is_empty()
            {
                lines.push((current.clone(), added.to_string()));
            }
        }
        Ok(lines)
    }

    /// A configuration value, or `None` if unset.
    pub fn config_get(&self, key: &str) -> Result<Option<String>> {
        let exit = self.exit(["config", "--get", key], None, &[])?;
        Ok(
            (exit.code == Some(0))
                .then(|| String::from_utf8_lossy(&exit.stdout).trim().to_string()),
        )
    }

    /// Whether commits should be signed, per the user's configuration.
    pub fn signs_commits(&self) -> Result<bool> {
        Ok(self
            .config_get("commit.gpgsign")?
            .is_some_and(|v| matches!(v.as_str(), "true" | "yes" | "on" | "1")))
    }

    /// Creates a commit object from a tree and parents; signs it with `-S` when `sign` is set.
    pub fn commit_tree(
        &self,
        tree: &str,
        parents: &[&str],
        message: &str,
        sign: bool,
    ) -> Result<String> {
        let mut args: Vec<String> = vec!["commit-tree".into(), tree.into()];
        for p in parents {
            args.push("-p".into());
            args.push((*p).into());
        }
        args.push(if sign {
            "-S".into()
        } else {
            "--no-gpg-sign".into()
        });
        args.push("-F".into());
        args.push("-".into());
        let out = self.run_bytes(&args, Some(message.as_bytes()), &[])?;
        Ok(String::from_utf8_lossy(&out).trim().to_string())
    }

    /// A commit on top of `base` whose tree is `base`'s with everything under `records/` replaced by the
    /// `records/` of `records_from`. Governed content, and so the fingerprint, is that of `base`. When the
    /// trees come out equal, `base` itself is returned and no commit is made.
    pub fn commit_with_records(
        &self,
        base: &str,
        records_from: &str,
        message: &str,
        sign: bool,
    ) -> Result<String> {
        let is_record = |e: &TreeEntry| e.path.starts_with(attrs::RECORDS_PREFIX);
        let index = index::TempIndex::from_tree(self, base)?;
        let mut edits: Vec<index::IndexEdit> = self
            .ls_tree(base)?
            .into_iter()
            .filter(is_record)
            .map(|e| index::IndexEdit::Remove { path: e.path })
            .collect();
        edits.extend(
            self.ls_tree(records_from)?
                .into_iter()
                .filter(is_record)
                .map(|e| index::IndexEdit::Put {
                    mode: e.mode,
                    oid: e.oid,
                    path: e.path,
                }),
        );
        index.apply(&edits)?;
        let tree = index.write_tree()?;
        if tree == self.tree_of(base)? {
            return Ok(base.to_string());
        }
        self.commit_tree(&tree, &[base], message, sign)
    }

    /// Three-way merges `theirs` into `ours` from `base` without touching any work tree.
    /// Returns the merged tree, or `None` on a conflict.
    pub fn merge_tree(&self, base: &str, ours: &str, theirs: &str) -> Result<Option<String>> {
        let exit = self.exit(
            [
                "merge-tree",
                "--write-tree",
                &format!("--merge-base={base}"),
                ours,
                theirs,
            ],
            None,
            &[],
        )?;
        match exit.code {
            Some(0) => Ok(String::from_utf8_lossy(&exit.stdout)
                .lines()
                .next()
                .map(str::to_string)),
            Some(1) => Ok(None),
            _ => Err(GitError::Failed {
                command: "merge-tree".into(),
                status: exit.code,
                stderr: exit.stderr,
            }),
        }
    }

    /// Fast-forwards the current branch (and work tree) to `commit`; refuses rather than overwrite edits.
    pub fn merge_ff_only(&self, commit: &str) -> Result<std::result::Result<(), String>> {
        let exit = self.exit(
            ["merge", "--ff-only", "--quiet", "--no-edit", commit],
            None,
            &[],
        )?;
        Ok(match exit.code {
            Some(0) => Ok(()),
            _ => Err(exit.stderr),
        })
    }

    /// Writes the current index as a tree (used by the commit-msg hook) and returns its id.
    pub fn write_index_tree(&self) -> Result<String> {
        Ok(self.run(["write-tree"])?.trim().to_string())
    }

    /// Lists tracked paths (`git ls-files -z`).
    pub fn ls_files(&self) -> Result<Vec<String>> {
        let out = self.run_bytes(["ls-files", "-z"], None, &[])?;
        Ok(out
            .split(|b| *b == 0)
            .filter(|c| !c.is_empty())
            .map(|c| String::from_utf8_lossy(c).into_owned())
            .collect())
    }

    /// Every entry of `treeish` under `dir` (recursively), with full paths.
    pub fn ls_tree_under(&self, treeish: &str, dir: &str) -> Result<Vec<TreeEntry>> {
        let out = self.run_bytes(
            ["ls-tree", "-r", "-z", "--full-tree", treeish, "--", dir],
            None,
            &[],
        )?;
        parse_ls_tree(&out)
    }

    /// Whether `path` (a file or a directory) exists in `treeish`.
    pub fn path_exists(&self, treeish: &str, path: &str) -> Result<bool> {
        let spec = format!("{treeish}:{}", path.trim_end_matches('/'));
        Ok(self.exit(["cat-file", "-e", &spec], None, &[])?.code == Some(0))
    }

    /// The sizes in bytes of blobs, by id (`cat-file --batch-check`).
    pub fn blob_sizes(&self, oids: &[String]) -> Result<BTreeMap<String, u64>> {
        let mut sizes = BTreeMap::new();
        if oids.is_empty() {
            return Ok(sizes);
        }
        let input = oids.iter().map(|o| format!("{o}\n")).collect::<String>();
        let out = self.run_bytes(
            ["cat-file", "--batch-check=%(objectname) %(objectsize)"],
            Some(input.as_bytes()),
            &[],
        )?;
        for line in String::from_utf8_lossy(&out).lines() {
            if let Some((oid, size)) = line.split_once(' ')
                && let Ok(size) = size.trim().parse::<u64>()
            {
                sizes.insert(oid.to_string(), size);
            }
        }
        Ok(sizes)
    }

    /// The contents of many blobs in one `cat-file --batch` call, in the order given.
    pub fn cat_blobs(&self, oids: &[String]) -> Result<Vec<Vec<u8>>> {
        if oids.is_empty() {
            return Ok(Vec::new());
        }
        let input = oids.iter().map(|o| format!("{o}\n")).collect::<String>();
        let out = self.run_bytes(["cat-file", "--batch"], Some(input.as_bytes()), &[])?;
        let mut blobs = Vec::with_capacity(oids.len());
        let mut rest = out.as_slice();
        for oid in oids {
            let newline = rest
                .iter()
                .position(|b| *b == b'\n')
                .ok_or_else(|| GitError::Parse("cat-file --batch: truncated".into()))?;
            let header = String::from_utf8_lossy(&rest[..newline]).into_owned();
            rest = &rest[newline + 1..];
            let mut fields = header.split(' ');
            let (_, kind, size) = (fields.next(), fields.next(), fields.next());
            if kind != Some("blob") {
                return Err(GitError::Parse(format!("{oid} is not a blob: {header}")));
            }
            let size: usize = size
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| GitError::Parse(header.clone()))?;
            if rest.len() < size + 1 {
                return Err(GitError::Parse(format!(
                    "cat-file --batch: short read for {oid}"
                )));
            }
            blobs.push(rest[..size].to_vec());
            rest = &rest[size + 1..];
        }
        Ok(blobs)
    }

    /// Commits reachable from `rev` whose message matches the extended regular expression `pattern`.
    pub fn log_grep(&self, rev: &str, pattern: &str) -> Result<Vec<String>> {
        let out = self.run([
            "log",
            "--format=%H",
            "-E",
            &format!("--grep={pattern}"),
            rev,
        ])?;
        Ok(out.lines().map(str::to_string).collect())
    }

    /// The paths a commit adds relative to its first parent (or to the empty tree for a root commit).
    pub fn added_paths(&self, commit: &str) -> Result<Vec<String>> {
        let base = match self.parents(commit)?.first() {
            Some(parent) => parent.clone(),
            None => self.empty_tree()?,
        };
        Ok(self
            .diff_tree(&base, commit)?
            .into_iter()
            .filter(|d| d.status == 'A')
            .map(|d| d.path)
            .collect())
    }
}

fn parse_ls_tree(out: &[u8]) -> Result<Vec<TreeEntry>> {
    out.split(|b| *b == 0)
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| {
            let text = String::from_utf8_lossy(chunk);
            let (meta, path) = text
                .split_once('\t')
                .ok_or_else(|| GitError::Parse(text.to_string()))?;
            let mut fields = meta.split(' ');
            Ok(TreeEntry {
                mode: fields.next().unwrap_or_default().to_string(),
                kind: fields.next().unwrap_or_default().to_string(),
                oid: fields.next().unwrap_or_default().to_string(),
                path: path.to_string(),
            })
        })
        .collect()
}

impl Exit {
    fn stderr_command(&self) -> String {
        self.stderr
            .split(']')
            .next()
            .unwrap_or("")
            .trim_start_matches('[')
            .to_string()
    }
}

fn parse_raw_diff(out: &[u8]) -> Result<Vec<DiffEntry>> {
    let mut entries = Vec::new();
    let mut parts = out.split(|b| *b == 0).filter(|c| !c.is_empty());
    while let Some(meta) = parts.next() {
        let meta = String::from_utf8_lossy(meta);
        let path = parts
            .next()
            .ok_or_else(|| GitError::Parse(format!("raw diff without a path: {meta}")))?;
        let fields: Vec<&str> = meta.trim_start_matches(':').split(' ').collect();
        if fields.len() < 5 {
            return Err(GitError::Parse(format!("unexpected raw diff line: {meta}")));
        }
        entries.push(DiffEntry {
            old_mode: fields[0].to_string(),
            new_mode: fields[1].to_string(),
            old_oid: fields[2].to_string(),
            new_oid: fields[3].to_string(),
            status: fields[4].chars().next().unwrap_or('?'),
            path: String::from_utf8_lossy(path).into_owned(),
        });
    }
    Ok(entries)
}

/// The null object id used to remove entries through `--index-info`.
pub fn null_oid() -> &'static str {
    NULL_OID
}

#[cfg(test)]
pub(crate) mod testutil {
    use super::*;
    use std::fs;

    /// A fresh repository with an isolated configuration and one commit of `files`.
    pub fn repo_with(files: &[(&str, &str)]) -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        write_files(dir.path(), files);
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", "init"]).unwrap();
        (dir, git)
    }

    pub fn write_files(root: &Path, files: &[(&str, &str)]) {
        for (path, content) in files {
            let full = root.join(path);
            fs::create_dir_all(full.parent().unwrap()).unwrap();
            fs::write(full, content).unwrap();
        }
    }

    pub fn commit_all(git: &Git, message: &str) -> String {
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "-m", message]).unwrap();
        git.head().unwrap().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::testutil::*;
    use super::*;

    #[test]
    fn git_is_recent_enough() {
        let (major, minor) = Git::version().unwrap();
        assert!(
            (major, minor) >= MIN_GIT_VERSION,
            "git {major}.{minor} is older than 2.40"
        );
    }

    #[test]
    fn commit_with_records_adds_the_records_on_top_of_the_base() {
        let (dir, git) = repo_with(&[("a.txt", "a")]);
        let init = git.head().unwrap().unwrap();
        // The candidate: changed code, no records.
        git.run(["checkout", "-q", "-b", "cand"]).unwrap();
        write_files(dir.path(), &[("a.txt", "changed")]);
        let candidate = commit_all(&git, "candidate");
        // The local branch: other code, and records.
        git.run(["checkout", "-q", "main"]).unwrap();
        write_files(
            dir.path(),
            &[
                ("a.txt", "elsewhere"),
                ("records/jobs/x/outcome.json", "{}"),
                ("records/jobs/x/verification.json", "{}"),
            ],
        );
        commit_all(&git, "records");
        let head = git.head().unwrap().unwrap();
        assert_ne!(init, head);

        let work = git
            .commit_with_records(&candidate, &head, "work", false)
            .unwrap();
        assert_ne!(work, candidate);
        assert_eq!(git.parents(&work).unwrap(), vec![candidate.clone()]);
        assert!(git.is_ancestor(&candidate, &work).unwrap());
        let files: Vec<(String, Vec<u8>)> = git
            .ls_tree(&work)
            .unwrap()
            .into_iter()
            .map(|e| {
                let blob = git.cat_blob(&e.oid).unwrap();
                (e.path, blob)
            })
            .collect();
        let paths: Vec<&str> = files.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(
            paths,
            vec![
                "a.txt",
                "records/jobs/x/outcome.json",
                "records/jobs/x/verification.json"
            ]
        );
        assert_eq!(files[0].1, b"changed", "code is the candidate's");
        let rules = &init;
        assert_eq!(
            attrs::fingerprint(&git, &work, rules).unwrap(),
            attrs::fingerprint(&git, &candidate, rules).unwrap()
        );
        // Nothing to add: the base comes back unchanged.
        assert_eq!(
            git.commit_with_records(&candidate, &candidate, "work", false)
                .unwrap(),
            candidate
        );
    }

    /// A bare remote and a clone of it with one commit pushed to `main`.
    fn with_bare_remote() -> (tempfile::TempDir, tempfile::TempDir, Git, Git) {
        let bare_dir = tempfile::tempdir().unwrap();
        let bare = Git::isolated(bare_dir.path());
        bare.run(["init", "-q", "--bare", "-b", "main"]).unwrap();
        let (dir, git) = repo_with(&[("a.txt", "a")]);
        let url = bare_dir.path().to_str().unwrap().to_string();
        git.run(["remote", "add", "origin", &url]).unwrap();
        git.run(["push", "-q", "origin", "main"]).unwrap();
        (bare_dir, dir, bare, git)
    }

    #[test]
    fn a_guarded_push_replaces_the_branch_when_the_remote_is_where_expected() {
        let (_b, dir, bare, git) = with_bare_remote();
        let first = git.head().unwrap().unwrap();
        git.run(["push", "-q", "origin", "main:refs/heads/work"])
            .unwrap();
        // A history that does not contain `first`'s successor: rewrite, not fast-forward.
        git.run(["checkout", "-q", "-b", "other", &first]).unwrap();
        write_files(dir.path(), &[("a.txt", "other")]);
        let rewritten = commit_all(&git, "rewritten");
        git.push_if_remote_at("origin", "work", &rewritten, &first)
            .unwrap();
        assert_eq!(
            bare.rev_parse("refs/heads/work").unwrap().as_deref(),
            Some(rewritten.as_str())
        );
    }

    #[test]
    fn a_guarded_push_is_rejected_and_changes_nothing_when_the_remote_moved() {
        let (_b, dir, bare, git) = with_bare_remote();
        let first = git.head().unwrap().unwrap();
        git.run(["push", "-q", "origin", "main:refs/heads/work"])
            .unwrap();
        write_files(dir.path(), &[("a.txt", "moved")]);
        let moved = commit_all(&git, "moved");
        git.run(["push", "-q", "origin", "main:refs/heads/work"])
            .unwrap();
        git.run(["checkout", "-q", "-b", "other", &first]).unwrap();
        write_files(dir.path(), &[("a.txt", "other")]);
        let mine = commit_all(&git, "mine");
        // `first` is stale: the remote branch holds `moved`.
        assert!(
            git.push_if_remote_at("origin", "work", &mine, &first)
                .is_err()
        );
        assert_eq!(
            bare.rev_parse("refs/heads/work").unwrap().as_deref(),
            Some(moved.as_str())
        );
        assert!(git.push_if_remote_at("origin", "", &mine, &moved).is_err());
    }

    #[test]
    fn trailers_are_parsed_by_git() {
        let (_dir, git) = repo_with(&[("a.txt", "a")]);
        let message = "Subject\n\nBody.\n\nAthena-Job: 20261010-153012-k7q2\nAthena-Class: docs\n";
        let trailers = git.trailers(message).unwrap();
        assert!(trailers.contains(&("Athena-Job".into(), "20261010-153012-k7q2".into())));
        assert!(trailers.contains(&("Athena-Class".into(), "docs".into())));
    }

    #[test]
    fn diff_tree_reports_raw_changes() {
        let (dir, git) = repo_with(&[("a.txt", "a"), ("b.txt", "b")]);
        let first = git.head().unwrap().unwrap();
        write_files(dir.path(), &[("a.txt", "changed")]);
        std::fs::remove_file(dir.path().join("b.txt")).unwrap();
        write_files(dir.path(), &[("c.txt", "new")]);
        let second = commit_all(&git, "change");
        let changes = git.diff_tree(&first, &second).unwrap();
        let summary: Vec<(char, &str)> = changes
            .iter()
            .map(|c| (c.status, c.path.as_str()))
            .collect();
        assert_eq!(
            summary,
            vec![('M', "a.txt"), ('D', "b.txt"), ('A', "c.txt")]
        );
        let added = git.added_lines(&first, &second).unwrap();
        assert!(added.contains(&("c.txt".into(), "new".into())));
    }

    #[test]
    fn merge_tree_detects_conflicts() {
        let (dir, git) = repo_with(&[("a.txt", "base\n")]);
        let base = git.head().unwrap().unwrap();
        write_files(dir.path(), &[("a.txt", "ours\n")]);
        let ours = commit_all(&git, "ours");
        git.run(["checkout", "-q", "-b", "other", &base]).unwrap();
        write_files(dir.path(), &[("a.txt", "theirs\n")]);
        let theirs = commit_all(&git, "theirs");
        assert_eq!(git.merge_tree(&base, &ours, &theirs).unwrap(), None);
    }
}
