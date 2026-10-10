//! Athena's commit and record gates (design §7.4).
//!
//! The gate is one function with one decision point, [`gate`], run in three places:
//!
//! - the `pre-commit` and `commit-msg` git hooks, for every commit made in a set-up clone;
//! - in process, for every commit Athena builds with plumbing (plumbing commits run no hooks);
//! - CI, over every commit in the history ([`history::check_history`]).
//!
//! Every check judges a change by the rules of its parent commit, never by rules the change itself makes
//! (D38). The checks are: secrets ([`secrets`]), append-only and valid records, size, and personal data
//! ([`content`]), the governed-content invariant and the prediction trailer ([`invariant`]), and
//! reconciliation of usage records ([`reconcile`]). [`redact`] scrubs agent free text before it is recorded.

pub mod content;
pub mod history;
pub mod invariant;
pub mod reconcile;
pub mod records;
pub mod redact;
pub mod secrets;

use athena_data::Config;
use athena_git::attrs::{PathClass, classify};
use athena_git::{DiffEntry, Git, GitError};
use std::collections::BTreeMap;
use std::fmt;

/// The configuration compiled into Athena, used to judge commits whose parent has no `athena.toml`.
pub const BUILTIN_CONFIG: &str = include_str!("../../../athena.toml");

/// The repository-relative path of the configuration.
pub const CONFIG_PATH: &str = "athena.toml";
/// The repository-relative path of the defaults model.
pub const DEFAULTS_PATH: &str = "defaults.toml";

/// The rule a violation breaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rule {
    Secrets,
    AppendOnly,
    ValidRecords,
    Size,
    PersonalData,
    Configuration,
    Invariant,
    Prediction,
    Reconciliation,
}

impl Rule {
    pub fn name(self) -> &'static str {
        match self {
            Rule::Secrets => "no secrets",
            Rule::AppendOnly => "records are append-only",
            Rule::ValidRecords => "records are valid and in place",
            Rule::Size => "files stay small",
            Rule::PersonalData => "no personal data",
            Rule::Configuration => "configuration parses",
            Rule::Invariant => {
                "governed content changes only through a verified landing, a declared intervention, or the open bootstrap range"
            }
            Rule::Prediction => "the prediction precedes the outcome",
            Rule::Reconciliation => "usage records reconcile",
        }
    }
}

/// One broken rule, with where it was broken and how to repair it.
#[derive(Debug, Clone, PartialEq)]
pub struct Violation {
    pub rule: Rule,
    pub commit: Option<String>,
    pub path: Option<String>,
    pub detail: String,
    pub repair: String,
}

impl Violation {
    pub fn new(rule: Rule, detail: impl Into<String>, repair: impl Into<String>) -> Self {
        Violation {
            rule,
            commit: None,
            path: None,
            detail: detail.into(),
            repair: repair.into(),
        }
    }
    pub fn at_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
    pub fn in_commit(mut self, commit: Option<&str>) -> Self {
        self.commit = commit.map(|c| c.chars().take(12).collect());
        self
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.rule.name(), self.detail)?;
        if let Some(commit) = &self.commit {
            write!(f, " (commit {commit})")?;
        }
        if !self.repair.is_empty() {
            write!(f, ". Repair: {}", self.repair)?;
        }
        Ok(())
    }
}

/// An error that stops the gate from reaching a verdict. The gate fails closed on every one of them.
#[derive(Debug, thiserror::Error)]
pub enum GateError {
    #[error(transparent)]
    Git(#[from] GitError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("gitleaks {version} is not installed at {path}; run ./setup.sh to install it")]
    GitleaksMissing { version: String, path: String },
    #[error("gitleaks failed: {0}")]
    Gitleaks(String),
    #[error("{0}")]
    Config(String),
}

pub type Result<T> = std::result::Result<T, GateError>;

/// The configuration that judges changes on top of `commit`: its `athena.toml`, or the built-in one when
/// the commit has none. A configuration that does not parse is an error.
pub fn config_at(git: &Git, commit: Option<&str>) -> Result<Config> {
    let text = match commit {
        Some(commit) => git.show_file(commit, CONFIG_PATH)?,
        None => None,
    };
    match text {
        Some(bytes) => Config::parse(&String::from_utf8_lossy(&bytes)).map_err(|e| {
            GateError::Config(format!(
                "{CONFIG_PATH} at {}: {e}",
                short(commit.unwrap_or(""))
            ))
        }),
        None => Ok(builtin_config()),
    }
}

/// The configuration compiled into this build.
pub fn builtin_config() -> Config {
    Config::parse(BUILTIN_CONFIG).expect("the built-in athena.toml is validated by tests")
}

/// The first twelve characters of a commit id, for messages.
pub fn short(id: &str) -> String {
    id.chars().take(12).collect()
}

/// A change to judge: a commit that exists, or one about to be made from the index.
#[derive(Debug, Clone)]
pub struct Change {
    pub parents: Vec<String>,
    pub tree: String,
    /// The commit message, when known (not yet in `pre-commit`).
    pub message: Option<String>,
    /// The commit id, when the commit exists.
    pub commit: Option<String>,
}

impl Change {
    /// An existing commit.
    pub fn of_commit(git: &Git, commit: &str) -> Result<Change> {
        Ok(Change {
            parents: git.parents(commit)?,
            tree: git.tree_of(commit)?,
            message: Some(git.message(commit)?),
            commit: Some(commit.to_string()),
        })
    }

    /// The commit the index would make now, on top of `HEAD` (and `MERGE_HEAD` during a merge).
    pub fn staged(git: &Git, message: Option<String>) -> Result<Change> {
        let mut parents: Vec<String> = git.head()?.into_iter().collect();
        let merge_head = git.common_dir()?.join("MERGE_HEAD");
        if let Ok(text) = std::fs::read_to_string(merge_head) {
            parents.extend(
                text.lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty()),
            );
        }
        Ok(Change {
            parents,
            tree: git.write_index_tree()?,
            message,
            commit: None,
        })
    }

    /// The tree-ish the change is diffed against: its first parent, or the empty tree.
    pub fn base(&self, git: &Git) -> Result<String> {
        match self.parents.first() {
            Some(parent) => Ok(parent.clone()),
            None => Ok(git.empty_tree()?),
        }
    }

    /// The trailers of the message, parsed by git, with comment lines removed first.
    pub fn trailers(&self, git: &Git) -> Result<Vec<(String, String)>> {
        let Some(message) = &self.message else {
            return Ok(Vec::new());
        };
        let cleaned: String = message
            .lines()
            .filter(|l| !l.starts_with('#'))
            .map(|l| format!("{l}\n"))
            .collect();
        Ok(git.trailers(&cleaned)?)
    }
}

/// Values of one trailer key.
pub fn trailer_values<'a>(trailers: &'a [(String, String)], key: &str) -> Vec<&'a str> {
    trailers
        .iter()
        .filter(|(k, _)| k.eq_ignore_ascii_case(key))
        .map(|(_, v)| v.as_str())
        .collect()
}

/// What a change looks like under its parent's rules.
#[derive(Debug)]
pub struct Judged {
    pub config: Config,
    /// The tree-ish the change is diffed against.
    pub base: String,
    /// The commit (or tree) whose committed rules classify paths: the first parent, or the empty tree.
    pub rules: String,
    pub diff: Vec<DiffEntry>,
    pub classes: BTreeMap<String, PathClass>,
    /// Set when the parent's configuration did not parse and the built-in one judged instead.
    pub config_error: Option<String>,
}

impl Judged {
    pub fn new(git: &Git, change: &Change) -> Result<Judged> {
        let base = change.base(git)?;
        let rules = base.clone();
        let (config, config_error) =
            match config_at(git, change.parents.first().map(String::as_str)) {
                Ok(config) => (config, None),
                Err(GateError::Config(message)) => (builtin_config(), Some(message)),
                Err(other) => return Err(other),
            };
        let diff = git.diff_tree(&base, &change.tree)?;
        let paths: Vec<String> = diff.iter().map(|d| d.path.clone()).collect();
        let classes = classify(git, &rules, &paths)?;
        Ok(Judged {
            config,
            base,
            rules,
            diff,
            classes,
            config_error,
        })
    }

    pub fn class(&self, path: &str) -> PathClass {
        self.classes
            .get(path)
            .copied()
            .unwrap_or(PathClass::Governed)
    }
}

/// Where the gate runs; it decides which checks apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// The `pre-commit` hook: secrets in the staged changes, and the content checks.
    PreCommit,
    /// The `commit-msg` hook: the checks that need the message.
    CommitMsg,
    /// Athena's own plumbing commits: every check.
    Commit,
}

/// The gate: every check that applies in `scope`, judged by the parent's rules. One decision point.
pub fn gate(git: &Git, change: &Change, scope: Scope) -> Result<Vec<Violation>> {
    let judged = Judged::new(git, change)?;
    let mut violations = Vec::new();
    if let Some(error) = &judged.config_error {
        violations.push(
            Violation::new(
                Rule::Configuration,
                error.clone(),
                "fix athena.toml in a commit of its own",
            )
            .in_commit(change.commit.as_deref()),
        );
    }
    match scope {
        Scope::PreCommit => {
            violations.extend(secrets::check_staged(git, &judged.config)?);
            violations.extend(content::check(git, change, &judged, None)?);
        }
        Scope::CommitMsg => {
            violations.extend(invariant::check(git, change, &judged, None)?);
        }
        Scope::Commit => {
            let commit = change
                .commit
                .as_deref()
                .ok_or_else(|| GateError::Config("the commit scope needs a commit".into()))?;
            violations.extend(secrets::check_commit(
                git,
                &judged.config,
                change.parents.first().map(String::as_str),
                commit,
            )?);
            violations.extend(content::check(git, change, &judged, None)?);
            violations.extend(invariant::check(git, change, &judged, None)?);
            violations.extend(invariant::check_prediction(git, change)?);
            violations.extend(reconcile::check_tree(git, &change.tree)?);
        }
    }
    Ok(violations)
}

#[cfg(test)]
pub(crate) mod testutil {
    use athena_git::Git;
    use std::path::Path;

    /// A repository with an isolated configuration and a fixed identity, and one commit of `files`.
    pub fn repo(files: &[(&str, &str)]) -> (tempfile::TempDir, Git) {
        let dir = tempfile::tempdir().unwrap();
        let git = Git::isolated(dir.path());
        git.run(["init", "-q", "-b", "main"]).unwrap();
        write(dir.path(), files);
        commit(&git, "init\n\nAthena-Job: bootstrap\n");
        (dir, git)
    }

    pub fn write(root: &Path, files: &[(&str, &str)]) {
        for (path, content) in files {
            let full = root.join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(full, content).unwrap();
        }
    }

    pub fn commit(git: &Git, message: &str) -> String {
        git.run(["add", "-A"]).unwrap();
        git.run(["commit", "-q", "--allow-empty", "-m", message])
            .unwrap();
        git.head().unwrap().unwrap()
    }

    /// A token in the shape of a GitHub personal access token, built at run time so that no secret-shaped
    /// string exists in the source.
    pub fn fake_token() -> String {
        let body: String = "Zq8rT2vK9mW4xN7bP3cH6jL1sD5fG0aY"
            .chars()
            .chain("uE2i".chars())
            .collect();
        format!("{}{}", ["gh", "p_"].concat(), body)
    }

    /// An email address that is not on the allow-list, built at run time.
    pub fn private_email() -> String {
        format!("{}@{}", "jane.doe", ["corp", "mail", "com"].join("."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_configuration_parses() {
        let config = builtin_config();
        assert!(!config.history.exempt.is_empty());
    }

    #[test]
    fn violations_name_the_rule_and_the_repair() {
        let v = Violation::new(
            Rule::AppendOnly,
            "records/a.json was modified",
            "write a new record instead",
        )
        .in_commit(Some("0123456789abcdef"));
        let text = v.to_string();
        assert!(text.starts_with("records are append-only: records/a.json was modified"));
        assert!(text.contains("commit 0123456789ab"));
        assert!(text.contains("Repair: write a new record instead"));
    }
}
