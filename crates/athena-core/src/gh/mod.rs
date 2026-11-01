//! `athena land --pr`: push the job's branch, open a pull request, wait for green checks, and, only when the
//! opt-in setting `[landing] merge_via_rest` is on, ask the host to merge (`docs/design/native-pr-landing.md`).
//!
//! The host client runs the `gh` program and only its `api` subcommand, with REST paths such as
//! `repos/{owner}/{repo}/pulls`. It uses no other `gh` subcommand and no query-language endpoint. The program
//! sits behind [`Runner`], and the host behind [`PrHost`], so the logic is tested without a network.
//!
//! With the setting off (the default) the command stops at an open pull request and reports whether its
//! checks are green. A later run of the same command looks the pull request up by its stored number and, if
//! it was merged, runs the merged-content check of [`crate::pr`]: the job is landed on a matching
//! fingerprint, and `merged-unverified` otherwise. The merged commit is never found by reading trailers.

#[cfg(test)]
use crate::Ui;
#[cfg(test)]
use crate::pr::{
    Ancestry, Content, Host, LinkState, PrLink, load_early, load_early_link, load_link, store_early,
};
#[cfg(test)]
use crate::state::StateDir;
#[cfg(test)]
use anyhow::{Result, bail};
#[cfg(test)]
use athena_data::JobId;
#[cfg(test)]
use athena_data::records::WorkPhase;
#[cfg(test)]
use athena_git::{CommitText, Git};
#[cfg(test)]
use std::time::Duration;

/// The only host this slice supports.
const HOST: &str = "github.com";

mod client;
mod early;
mod landing;
mod pages;
mod records;
mod remote;
mod transport;

pub use client::{
    CI_WORKFLOW, Checks, CiRunState, EARLY_RUN_WAIT, GhClient, MergeMethod, MergeMethods, NewPr,
    PrDraft, PrHost, classify_checks, failing_test_name, parse_check_run, parse_ci_run,
    parse_ci_runs, parse_merged_pr, parse_remote,
};
#[cfg(test)]
use early::EARLY_RUN_JOB;
pub use early::{CI_BRANCH_PREFIX, EarlyPr, ci_branch, open_early_pr};
pub use landing::{
    LandPr, Poll, PrReport, confirm_existing_merge, land_pr, landed_commit, landed_commit_for,
    landed_commit_from_records, squash_text_for, work_branch,
};
pub use pages::{
    PAGES_WAIT_DELAY, PAGES_WAIT_LOOKS, PAGES_WORKFLOW, PagesHost, PagesReport, PagesRun,
    watch_pages,
};
pub use records::{
    CutoverRun, GitLandingRecords, LandingFacts, LandingRecords, cutover_catch_up, cutover_job_for,
    verification_id,
};
pub use remote::{GitRemote, Remote};
pub use transport::{
    CurlHttp, GhCli, GhFailure, HostErrorLog, HttpPost, HttpReply, Retry, Runner, is_transient,
    parse_status, retry_transient, transient_text,
};

/// The arguments of a plain read of a REST path: `gh api <path>`. `gh api` defaults to GET, so no method
/// override and no field or body argument is added.
pub fn get_args(path: &str) -> Vec<String> {
    vec!["api".to_string(), path.to_string()]
}

/// The seam tests use to give `Athena::go` a fake remote and host in place of `gh` and git.
#[cfg(test)]
pub(crate) mod early_seam {
    use super::{PrHost, Remote};
    use std::cell::RefCell;
    use std::rc::Rc;

    type Pair = (Rc<dyn Remote>, Rc<dyn PrHost>);

    thread_local! {
        static SEAM: RefCell<Option<Pair>> = const { RefCell::new(None) };
    }

    /// Clears the seam when dropped.
    pub struct Guard;

    impl Drop for Guard {
        fn drop(&mut self) {
            SEAM.with(|s| *s.borrow_mut() = None);
        }
    }

    pub fn install(remote: Rc<dyn Remote>, host: Rc<dyn PrHost>) -> Guard {
        SEAM.with(|s| *s.borrow_mut() = Some((remote, host)));
        Guard
    }

    pub fn current() -> Option<Pair> {
        SEAM.with(|s| s.borrow().clone())
    }
}

/// What follows a merge: the order of the branch resync, the Pages watch and the code-health check.
#[cfg(test)]
mod after_merge_tests;

#[cfg(test)]
mod tests;
