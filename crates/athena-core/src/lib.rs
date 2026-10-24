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
//! - [`records_import`]: the importer that copies `records/` at a commit onto the `athena/records` branch and
//!   checks the copy; not wired into any command;
//! - [`reconcile`]: the reconcile step of `docs/design/records-history.md` §8.1, run at the start of
//!   `athena land` and `athena status`: it writes a missing landing record from its landing intent, or reports
//!   the main commit as unattributed;
//! - [`admission`], [`toolconfig`], [`checks`], [`verify`], [`land`]: the verification protocol;
//! - [`scoring`]: forecasts resolved and scored;
//! - [`status`]: the state of the work, from the repository alone;
//! - [`nested`]: Athena running inside one of its own jobs;
//! - [`pr`]: pull-request landing in owned mode: the link store, the push guard, the merged-content check;
//! - [`ci_result`]: the CI result record keyed by commit SHA, its stale, missing and red rules, and the local
//!   fallback (`docs/design/ci-routing.md` 5.3);
//! - [`routing`]: the checks chosen by the plan's job kind, and where each result comes from: a reused CI
//!   result for the candidate SHA or a local run (`docs/design/ci-routing.md` 3.2);
//! - [`gh`]: `athena land --pr`: push the job's branch, open the pull request, wait for green checks, and merge
//!   over REST only when the opt-in setting is on;
//! - [`introspect`]: the introspection record of an event, the class registry, and the reading code shared with
//!   `athena backtest`;
//! - [`proposals`]: proposed defaults from repeated introspections, their validation, and the apply-as-a-job
//!   check; `defaults.toml` is never written by Athena;
//! - [`labeled`]: the labeled set of reviewed introspections and the scoring of Jev's recorded shadow answers
//!   against it, read-only;
//! - [`review`]: `athena review`, a person's verdict on an introspection's class, committed as a class-label
//!   record (and a class record for a new class); the introspection is never edited;
//! - [`watcher`]: decisions put under review by exact matching of host-error records, run at landing;
//! - [`steward`]: the first role, run in the background every five finished tasks. It calls a model, read-only,
//!   over a workspace of Athena's code, records and git log (and GitHub through `gh` reads), and queues the
//!   work items the model reports after Athena has validated them; the requirements ladder decides, for each,
//!   whether a request is queued or only a proposal; [`steward::queued_work`] lists the queued requests that
//!   `athena drain` (in the `athena-cli` crate) works, lands through the pull-request path, and rates after;
//! - [`work_decisions`]: `athena approve-work` and `athena reject-work`, a person's decision on a steward
//!   proposal, recorded as an appended work-decision record; approving submits a job whose request carries
//!   the proposal's cited content in full;
//! - [`audience`]: `athena audience`, a role that reads the live explainer page as a newcomer, rates it from 1
//!   to 5, and records the rating with findings that each cite their evidence; a rating of 2 or lower queues
//!   a work item, and the steward's brief and `athena status` show the latest rating;
//! - [`auditor`]: `athena audit`, a role that judges a job's real outcome against `BRIEF.md` and the owner's
//!   goals, never against the job's own acceptance criteria, and records findings that each cite their
//!   evidence; the prompt is a fixed constant, protected from jobs by nothing else until the warden request
//!   (prose-only);
//! - [`pin`]: a study's repository ref resolved to a commit by the core, and mounted read-only;
//! - [`plan`]: planning, the first stage: every open request fleshed out and all of them planned together into
//!   one ordered plan with parallel groups, which `athena drain` follows; and the plan-stage audit's tracing
//!   of logged gaps to stages.

pub mod admission;
pub mod audience;
pub mod auditor;
pub mod capability;
pub mod chain;
pub mod checks;
pub mod ci_result;
pub mod commits;
pub mod fake;
pub mod gh;
pub mod host_rules;
pub mod introspect;
pub mod invocation;
pub mod jev;
pub mod labeled;
pub mod land;
pub mod manage;
pub mod nested;
pub mod pin;
pub mod plan;
pub mod pr;
pub mod proposals;
pub mod reading;
pub mod reconcile;
pub mod records_import;
pub mod review;
pub mod routing;
pub mod run;
#[cfg(test)]
mod run_pin_tests;
pub mod scoring;
pub mod state;
pub mod status;
pub mod steward;
pub mod store;
pub mod toolconfig;
pub mod verify;
pub mod watcher;
pub mod work_decisions;

use anyhow::{Result, anyhow, bail};
use athena_agent::AgentDriver;
use athena_data::records::Build;
use athena_git::Git;
use capability::Capability;
use state::StateDir;
use std::path::PathBuf;
use std::sync::Arc;

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
    /// Shared, so the steward can use it from its background thread.
    pub driver: Arc<dyn AgentDriver>,
    pub capabilities: Vec<Box<dyn Capability>>,
    /// The running binary's version and source.
    pub build: Build,
    /// The binary the agents' hook runs.
    pub athena_bin: PathBuf,
    /// Set in a nested run: the outer job's id.
    pub nested: Option<String>,
    /// Resolves and checks out a study's repository ref; only capabilities that pin ever call it.
    pub pinner: Box<dyn pin::Pinner>,
    /// Jev, asked in shadow beside the introspection (`jev`). None means it is not asked. Its answers are
    /// recorded and never decide anything.
    pub jev: Option<Box<dyn jev::Evaluate>>,
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

    /// A redactor for a job's records: secrets, private addresses, and this machine's paths (the job's
    /// directory, the state directory, and the home directory, which names the user).
    pub fn redactor(
        &self,
        config: &athena_data::Config,
        job: Option<&athena_data::JobId>,
    ) -> Result<athena_gate::redact::Redactor> {
        let mut redactor =
            athena_gate::redact::Redactor::new(config)?.with_path(self.state.root(), "<state>");
        if let Some(job) = job {
            redactor = redactor.with_path(&self.state.job_dir(job), "<job>");
        }
        if let Some(home) = std::env::var_os("HOME") {
            redactor = redactor.with_path(std::path::Path::new(&home), "~");
        }
        Ok(redactor)
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

/// The longest generated commit subject or job title, in characters.
pub const SUBJECT_MAX: usize = 72;

/// Cuts `text` to at most `max` characters, at a word boundary.
///
/// Text that already fits is returned unchanged. Otherwise the cut falls at the last whitespace within
/// `max` characters, so no word is split, and trailing whitespace is dropped. A single word longer than
/// `max` has no boundary to cut at, so it is cut at `max` characters.
pub fn truncate_at_word(text: &str, max: usize) -> String {
    let Some((end, _)) = text.char_indices().nth(max) else {
        return text.to_string();
    };
    let head = &text[..end];
    if text[end..].starts_with(char::is_whitespace) {
        return head.trim_end().to_string();
    }
    match head.rfind(char::is_whitespace) {
        Some(at) if !head[..at].trim_end().is_empty() => head[..at].trim_end().to_string(),
        _ => head.to_string(),
    }
}

#[cfg(test)]
mod truncate_tests {
    use super::{SUBJECT_MAX, truncate_at_word};

    #[test]
    fn text_within_the_limit_is_unchanged() {
        assert_eq!(truncate_at_word("short text", 72), "short text");
        let exact = "a".repeat(72);
        assert_eq!(truncate_at_word(&exact, 72), exact);
    }

    #[test]
    fn over_limit_text_is_cut_at_the_last_whitespace() {
        let text =
            "one two three four five six seven eight nine ten eleven twelve thirteen fourteen";
        let cut = truncate_at_word(text, SUBJECT_MAX);
        assert!(cut.chars().count() <= SUBJECT_MAX);
        assert!(text.starts_with(&cut));
        assert!(cut.ends_with("thirteen"), "{cut:?}");
        // A limit that lands exactly on a space keeps the whole word before it.
        assert_eq!(truncate_at_word("abc def ghi", 7), "abc def");
        assert_eq!(truncate_at_word("abc def ghi", 6), "abc");
    }

    #[test]
    fn a_single_over_long_word_is_hard_cut() {
        let word = "x".repeat(100);
        assert_eq!(truncate_at_word(&word, 72), "x".repeat(72));
        assert_eq!(
            truncate_at_word(&format!(" {word}"), 72).chars().count(),
            72
        );
    }

    #[test]
    fn multibyte_text_does_not_panic() {
        let text = "héllo wörld ".repeat(20);
        let cut = truncate_at_word(&text, 72);
        assert!(cut.chars().count() <= 72);
        assert!(cut.ends_with("wörld") || cut.ends_with("héllo"), "{cut:?}");
        let wide = "日本語".repeat(40);
        assert_eq!(truncate_at_word(&wide, 72).chars().count(), 72);
    }
}
