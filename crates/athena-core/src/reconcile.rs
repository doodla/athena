//! Reconciliation of landing records (`docs/design/records-history.md` §8.1, decision D111, build job 3 part 2).
//!
//! The crash window: the host merged a pull request, the process stopped, and no landing record exists. The
//! landing-intent record was written first, so the facts for the repair are local. [`reconcile`] walks main's
//! first-parent commits and, for each commit that no landing record names:
//!
//! 1. **Local first.** A landing intent whose head tree equals the commit's tree and whose expected base is the
//!    commit's first parent ([`athena_data::history::LandingIntent::matches`]) gives the landing record. The
//!    host is not asked.
//! 2. **Host only as a fallback.** With no matching intent, the host is asked which pull request merged the
//!    commit. The landing is written only if a landing intent records that pull request with the same head,
//!    because a landing names its intent and Athena does not invent one.
//! 3. Otherwise the commit is reported as *unattributed* and nothing is written. Athena never guesses: not from
//!    a nearby intent, not from a nearby pull request, and not when the host cannot answer.
//!
//! **The walk is bounded.** It starts strictly after the later of the last landing record's commit and the
//! expected base of the first landing intent on `athena/records`. Older commits, such as everything before the
//! cutover, are never examined. With no records branch, or no landing intent on it, there is nothing to
//! reconcile and the report says so.
//!
//! **Known gap.** A crashed landing that a later landing has already passed (so it lies before the last landing
//! record) is outside the bound and is not repaired. Widening the bound would let an abandoned intent, such as
//! one for a closed pull request, hold the walk open forever and cost a host call per commit on every run.
//!
//! The merge method is inferred from the commit and needs no host call: two parents is a merge; one parent and
//! the commit being the pull request's head is a rebase (a fast-forward or replay of the head itself); one
//! parent otherwise is a squash. A rebase that replays several commits leaves a landing that names one host
//! commit, so the other replayed commits are reported unattributed.
//!
//! Reconciliation never fails the command that runs it: every problem becomes a finding or a skipped reason.

use crate::Athena;
use crate::Ui;
use crate::gh::{
    GhClient, GitLandingRecords, LandingFacts, MergeMethod, PrHost, Runner, parse_remote,
};
use crate::status::StatusView;
use anyhow::{Context, Result, anyhow};
use athena_data::RecordId;
use athena_data::history::{DeclaredIntervention, Landing, LandingIntent};
use athena_git::Git;
use serde::Serialize;
use std::collections::BTreeSet;

/// The history records reconciliation reads from the records branch.
#[derive(Debug, Clone, Default)]
pub struct RecordsSnapshot {
    pub intents: Vec<LandingIntent>,
    pub landings: Vec<Landing>,
    pub declared: Vec<DeclaredIntervention>,
}

/// Where reconciliation reads the records and writes a landing record.
pub trait ReconcileRecords {
    /// The records on the remote's records branch; `None` when that branch does not exist.
    fn snapshot(&self) -> Result<Option<RecordsSnapshot>>;
    /// Writes the landing record for `facts` and returns its id.
    fn write_landing(&self, facts: &LandingFacts) -> Result<RecordId>;
}

/// The pull request the host says merged a commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergedPr {
    pub number: u64,
    pub head_sha: String,
}

/// What reconciliation may ask the host.
pub trait CommitHost {
    /// The pull request that merged `commit` on the host; `None` when the host names none. An error means the
    /// host could not answer.
    fn merged_pr(&self, commit: &str) -> Result<Option<MergedPr>>;
}

/// Where the facts of a repaired landing came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// A landing intent matched the commit's tree and parent; the host was not asked.
    Intent,
    /// The host named the pull request, and a landing intent records it.
    Host,
}

/// What reconciliation found for one main commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum Finding {
    /// The missing landing record was written.
    Repaired {
        commit: String,
        pr_number: u64,
        landing: String,
        source: Source,
    },
    /// No landing record names the commit and nothing could say which pull request made it. Nothing was written.
    Unattributed { commit: String, reason: String },
    /// The pull request is known but the landing record could not be written; the intent stays for the next run.
    WriteFailed {
        commit: String,
        pr_number: u64,
        reason: String,
    },
}

impl Finding {
    /// One line for a person.
    pub fn line(&self) -> String {
        match self {
            Finding::Repaired {
                commit,
                pr_number,
                landing,
                source,
            } => {
                let how = match source {
                    Source::Intent => "from its landing intent",
                    Source::Host => "from the pull request the host named",
                };
                format!(
                    "reconcile: wrote landing record {landing} for main commit {commit} (pull request #{pr_number}), {how}"
                )
            }
            Finding::Unattributed { commit, reason } => format!(
                "reconcile: main commit {commit} is unattributed: no landing record names it ({reason}); nothing was written"
            ),
            Finding::WriteFailed {
                commit,
                pr_number,
                reason,
            } => format!(
                "reconcile: main commit {commit} is pull request #{pr_number} but its landing record could not be written ({reason}); it will be tried again"
            ),
        }
    }
}

/// The result of one reconciliation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct ReconcileReport {
    /// How many main commits were examined.
    pub examined: usize,
    pub findings: Vec<Finding>,
    /// Why nothing was reconciled, when that is so: no records branch, no landing intent, no remote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped: Option<String>,
}

impl ReconcileReport {
    fn skipped(reason: impl Into<String>) -> Self {
        ReconcileReport {
            skipped: Some(reason.into()),
            ..ReconcileReport::default()
        }
    }

    /// Whether there is anything to tell a person.
    pub fn is_quiet(&self) -> bool {
        self.findings.is_empty() && self.skipped.is_none()
    }

    /// The lines for a person: the skipped reason, then each finding.
    pub fn lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if let Some(reason) = &self.skipped {
            lines.push(format!("reconcile: nothing to do ({reason})"));
        }
        lines.extend(self.findings.iter().map(Finding::line));
        lines
    }

    /// Whether any commit is still unattributed.
    pub fn has_unattributed(&self) -> bool {
        self.findings
            .iter()
            .any(|f| matches!(f, Finding::Unattributed { .. }))
    }
}

/// Main's first-parent commits, oldest first, each with all its parents.
fn first_parent_line(git: &Git, main_tip: &str) -> Result<Vec<(String, Vec<String>)>> {
    let listing = git.run(["rev-list", "--first-parent", "--parents", main_tip])?;
    let mut line: Vec<(String, Vec<String>)> = listing
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let mut shas = l.split_whitespace().map(str::to_string);
            let commit = shas.next().unwrap_or_default();
            (commit, shas.collect())
        })
        .collect();
    line.reverse();
    Ok(line)
}

/// The merge method of a main commit (see the module text).
fn merge_method(parents: usize, commit: &str, head: &str) -> MergeMethod {
    if parents >= 2 {
        MergeMethod::Merge
    } else if commit == head {
        MergeMethod::Rebase
    } else {
        MergeMethod::Squash
    }
}

/// Finds main's first-parent commits after the bound that no landing record names, and writes the missing
/// landing records (see the module text). `main_tip` is the tip of the default branch.
pub fn reconcile(
    git: &Git,
    main_tip: &str,
    records: &dyn ReconcileRecords,
    host: &dyn CommitHost,
) -> Result<ReconcileReport> {
    let Some(snapshot) = records.snapshot()? else {
        return Ok(ReconcileReport::skipped(
            "the records branch athena/records does not exist",
        ));
    };
    let Some(first) = snapshot
        .intents
        .iter()
        .min_by_key(|i| (i.created_at, i.id.to_string()))
    else {
        return Ok(ReconcileReport::skipped(
            "the records branch holds no landing intent",
        ));
    };
    let line = first_parent_line(git, main_tip)?;
    let position = |sha: &str| line.iter().position(|(c, _)| c == sha);
    let Some(base_at) = position(&first.expected_base) else {
        return Ok(ReconcileReport::skipped(format!(
            "the first landing intent's expected base {} is not on main's first-parent line",
            first.expected_base
        )));
    };
    let last_landing_at = snapshot
        .landings
        .iter()
        .filter_map(|l| position(&l.host_commit))
        .max();
    let start = base_at.max(last_landing_at.unwrap_or(base_at)) + 1;

    let named: BTreeSet<&str> = snapshot
        .landings
        .iter()
        .map(|l| l.host_commit.as_str())
        .chain(snapshot.declared.iter().map(|d| d.commit_sha.as_str()))
        .collect();
    let mut fulfilled: BTreeSet<String> = snapshot
        .landings
        .iter()
        .map(|l| l.intent.to_string())
        .collect();

    let mut report = ReconcileReport::default();
    for (commit, parents) in &line[start..] {
        if named.contains(commit.as_str()) {
            continue;
        }
        report.examined += 1;
        let tree = git.tree_of(commit)?;
        let parent = parents.first().map(String::as_str);
        let open = |i: &&LandingIntent| !fulfilled.contains(&i.id.to_string());
        let matching: Vec<&LandingIntent> = snapshot
            .intents
            .iter()
            .filter(open)
            .filter(|i| parent.is_some_and(|p| i.matches(&tree, p)))
            .collect();
        let (intent, source) = match matching.as_slice() {
            [intent] => (*intent, Source::Intent),
            [] => match host.merged_pr(commit) {
                Err(error) => {
                    report.findings.push(Finding::Unattributed {
                        commit: commit.clone(),
                        reason: format!("host unavailable: {error:#}"),
                    });
                    continue;
                }
                Ok(None) => {
                    report.findings.push(Finding::Unattributed {
                        commit: commit.clone(),
                        reason: "no landing intent matches its tree and parent, and the host names no merged pull request for it".into(),
                    });
                    continue;
                }
                Ok(Some(pr)) => {
                    match snapshot
                        .intents
                        .iter()
                        .filter(open)
                        .find(|i| i.pr_number == pr.number && i.head_sha == pr.head_sha)
                    {
                        Some(intent) => (intent, Source::Host),
                        None => {
                            report.findings.push(Finding::Unattributed {
                                commit: commit.clone(),
                                reason: format!(
                                    "the host names pull request #{} with head {}, but no unfulfilled landing intent records that pull request and head",
                                    pr.number, pr.head_sha
                                ),
                            });
                            continue;
                        }
                    }
                }
            },
            several => {
                report.findings.push(Finding::Unattributed {
                    commit: commit.clone(),
                    reason: format!(
                        "{} landing intents match its tree and parent, so none is chosen",
                        several.len()
                    ),
                });
                continue;
            }
        };
        let facts = LandingFacts {
            intent: intent.id.clone(),
            expected_base: intent.expected_base.clone(),
            pr_number: intent.pr_number,
            head: intent.head_sha.clone(),
            host_commit: commit.clone(),
            merge_method: merge_method(parents.len(), commit, &intent.head_sha),
        };
        match records.write_landing(&facts) {
            Ok(landing) => {
                fulfilled.insert(intent.id.to_string());
                report.findings.push(Finding::Repaired {
                    commit: commit.clone(),
                    pr_number: intent.pr_number,
                    landing: landing.to_string(),
                    source,
                });
            }
            Err(error) => report.findings.push(Finding::WriteFailed {
                commit: commit.clone(),
                pr_number: intent.pr_number,
                reason: format!("{error:#}"),
            }),
        }
    }
    Ok(report)
}

/// The host when no client can be built: every question fails with the reason, so the commit is reported
/// unattributed.
struct NoHost(String);

impl CommitHost for NoHost {
    fn merged_pr(&self, _: &str) -> Result<Option<MergedPr>> {
        Err(anyhow!("{}", self.0))
    }
}

/// The default branch from the local `origin/HEAD`, without asking the host.
fn local_default_branch(git: &Git, remote: &str) -> Option<String> {
    let exit = git
        .exit(
            [
                "symbolic-ref",
                "--short",
                &format!("refs/remotes/{remote}/HEAD"),
            ],
            None,
            &[],
        )
        .ok()?;
    if exit.code != Some(0) {
        return None;
    }
    let text = String::from_utf8_lossy(&exit.stdout).trim().to_string();
    text.strip_prefix(&format!("{remote}/"))
        .filter(|b| !b.is_empty())
        .map(str::to_string)
}

impl Athena {
    /// Reconciles landing records for the remote `origin` (see the module text). It never fails: a problem
    /// becomes a skipped reason or a finding. The host is asked only for a commit no intent matches, and for the
    /// default branch only when `origin/HEAD` is not set locally.
    pub fn reconcile(&self, runner: &dyn Runner, ui: &dyn Ui) -> ReconcileReport {
        match self.try_reconcile(runner, ui) {
            Ok(report) => report,
            Err(error) => ReconcileReport::skipped(format!("{error:#}")),
        }
    }

    fn try_reconcile(&self, runner: &dyn Runner, ui: &dyn Ui) -> Result<ReconcileReport> {
        let remote = "origin";
        let url = self
            .git
            .run(["remote", "get-url", remote])
            .context("no remote named origin")?;
        let sink = GitLandingRecords::for_remote(&self.git, remote)?;
        let client = parse_remote(&url).ok().map(|(_, repository)| GhClient {
            ui: Some(ui),
            ..GhClient::new(runner, repository)
        });
        let branch = match (local_default_branch(&self.git, remote), &client) {
            (Some(branch), _) => branch,
            (None, Some(client)) => client
                .default_branch()
                .context("origin/HEAD is not set and the host did not name the default branch")?,
            (None, None) => {
                return Ok(ReconcileReport::skipped(
                    "origin/HEAD is not set and the remote is not a supported host",
                ));
            }
        };
        let Some(tip) = self.git.fetch_remote_branch(remote, &branch)? else {
            return Ok(ReconcileReport::skipped(format!(
                "the remote has no branch {branch}"
            )));
        };
        match &client {
            Some(client) => reconcile(&self.git, &tip, &sink, client),
            None => reconcile(
                &self.git,
                &tip,
                &sink,
                &NoHost("the remote is not a supported host".into()),
            ),
        }
    }

    /// [`Athena::status`] with a reconciliation first, whose report is in the view.
    pub fn status_with_reconcile(&self, runner: &dyn Runner, ui: &dyn Ui) -> Result<StatusView> {
        let report = self.reconcile(runner, ui);
        let mut view = self.status()?;
        view.reconcile = Some(report);
        Ok(view)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use athena_data::history::{HistoryRecord, MergeMethod as RecordedMethod};
    use athena_git::records_branch::{Change, RECORDS_REF, RecordsWriter};
    use std::cell::RefCell;

    use crate::gh::LandingRecords;

    const JOB: &str = "20261024-114125-7z2t";

    enum Answer {
        Down,
        Nothing,
        Pr(u64, String),
    }

    struct FakeHost {
        answer: Answer,
        calls: RefCell<Vec<String>>,
    }

    impl FakeHost {
        fn new(answer: Answer) -> Self {
            FakeHost {
                answer,
                calls: RefCell::new(Vec::new()),
            }
        }
        fn calls(&self) -> usize {
            self.calls.borrow().len()
        }
    }

    impl CommitHost for FakeHost {
        fn merged_pr(&self, commit: &str) -> Result<Option<MergedPr>> {
            self.calls.borrow_mut().push(commit.to_string());
            match &self.answer {
                Answer::Down => Err(anyhow!("connection refused")),
                Answer::Nothing => Ok(None),
                Answer::Pr(number, head) => Ok(Some(MergedPr {
                    number: *number,
                    head_sha: head.clone(),
                })),
            }
        }
    }

    /// A work repository with a bare `origin`; main holds one commit, pushed.
    struct World {
        _bare_dir: tempfile::TempDir,
        bare: Git,
        _work_dir: tempfile::TempDir,
        git: Git,
    }

    impl World {
        fn new() -> Self {
            let bare_dir = tempfile::tempdir().unwrap();
            let bare = Git::isolated(bare_dir.path());
            bare.run(["init", "-q", "--bare", "-b", "main"]).unwrap();
            let work_dir = tempfile::tempdir().unwrap();
            let git = Git::isolated(work_dir.path());
            git.run(["init", "-q", "-b", "main"]).unwrap();
            git.run(["remote", "add", "origin", bare_dir.path().to_str().unwrap()])
                .unwrap();
            let world = World {
                _bare_dir: bare_dir,
                bare,
                _work_dir: work_dir,
                git,
            };
            world.human("start");
            world
        }

        fn tip(&self) -> String {
            self.git.rev_parse("refs/heads/main").unwrap().unwrap()
        }

        fn sink(&self) -> GitLandingRecords<'_> {
            GitLandingRecords::for_remote(&self.git, "origin").unwrap()
        }

        /// A commit by hand on main, pushed.
        fn human(&self, file: &str) -> String {
            std::fs::write(self.git.dir().join(file), file).unwrap();
            self.git.run(["add", "-A"]).unwrap();
            self.git
                .run(["commit", "-q", "-m", &format!("human {file}")])
                .unwrap();
            self.git.run(["push", "-q", "origin", "main"]).unwrap();
            self.tip()
        }

        /// A pull request head one commit above `base`, with a job trailer, pushed as branch `pr<number>`.
        fn pr_head(&self, number: u64, base: &str) -> String {
            self.git.run(["checkout", "-q", "--detach", base]).unwrap();
            let file = format!("pr{number}.txt");
            std::fs::write(self.git.dir().join(&file), &file).unwrap();
            self.git.run(["add", "-A"]).unwrap();
            self.git
                .run([
                    "commit",
                    "-q",
                    "-m",
                    &format!("Pull request {number}\n\nAthena-Job: {JOB}"),
                ])
                .unwrap();
            let head = self.git.head().unwrap().unwrap();
            self.git
                .run([
                    "push",
                    "-q",
                    "origin",
                    &format!("{head}:refs/heads/pr{number}"),
                ])
                .unwrap();
            self.git.run(["checkout", "-q", "main"]).unwrap();
            head
        }

        /// A landing intent for a new pull request on `base`; returns (head, intent id).
        fn intent(&self, number: u64, base: &str) -> (String, RecordId) {
            let head = self.pr_head(number, base);
            let id = self.sink().intent(number, &head, base).unwrap();
            (head, id)
        }

        /// What the host does on a squash merge: one commit with the head's tree on `parent`, pushed as main.
        fn squash(&self, head: &str, parent: &str) -> String {
            let tree = self.git.tree_of(head).unwrap();
            let commit = self
                .git
                .commit_tree(&tree, &[parent], "Squash", false)
                .unwrap();
            self.git
                .run(["update-ref", "refs/heads/main", &commit])
                .unwrap();
            self.git.run(["reset", "-q", "--hard", "main"]).unwrap();
            self.git.run(["push", "-q", "origin", "main"]).unwrap();
            commit
        }

        fn run(&self, host: &FakeHost) -> ReconcileReport {
            reconcile(&self.git, &self.tip(), &self.sink(), host).unwrap()
        }

        fn records_tip(&self) -> Option<String> {
            self.bare.rev_parse(RECORDS_REF).unwrap()
        }

        fn landings(&self) -> Vec<Landing> {
            match self.sink().snapshot().unwrap() {
                Some(snapshot) => snapshot.landings,
                None => Vec::new(),
            }
        }
    }

    #[test]
    fn crash_between_merge_and_landing_is_repaired_from_the_intent_alone() {
        let world = World::new();
        let base = world.tip();
        let (head, intent) = world.intent(7, &base);
        // The host merged and the process stopped before the landing record.
        let merged = world.squash(&head, &base);
        assert!(world.landings().is_empty());

        let host = FakeHost::new(Answer::Down);
        let report = world.run(&host);

        assert_eq!(host.calls(), 0, "the host is not asked");
        assert_eq!(report.findings.len(), 1, "{report:?}");
        let Finding::Repaired {
            commit,
            pr_number,
            source,
            ..
        } = &report.findings[0]
        else {
            panic!("{report:?}");
        };
        assert_eq!((commit, *pr_number, *source), (&merged, 7, Source::Intent));
        let landings = world.landings();
        assert_eq!(landings.len(), 1);
        assert_eq!(landings[0].host_commit, merged);
        assert_eq!(landings[0].intent, intent);
        assert_eq!(landings[0].head_sha, head);
        assert_eq!(landings[0].merge_method, RecordedMethod::Squash);
        landings[0].validate().unwrap();

        // A second run finds the commit named: nothing new, no host call.
        let again = world.run(&host);
        assert!(again.findings.is_empty(), "{again:?}");
        assert_eq!(world.landings().len(), 1);
        assert_eq!(host.calls(), 0);
    }

    #[test]
    fn merge_method_follows_the_commit_shape() {
        assert_eq!(merge_method(2, "m", "h"), MergeMethod::Merge);
        assert_eq!(merge_method(1, "h", "h"), MergeMethod::Rebase);
        assert_eq!(merge_method(1, "s", "h"), MergeMethod::Squash);
    }

    #[test]
    fn unavailable_host_gives_unattributed_and_writes_nothing() {
        let world = World::new();
        let base = world.tip();
        world.intent(1, &base);
        let stray = world.human("stray");
        let before = world.records_tip();

        let host = FakeHost::new(Answer::Down);
        let report = world.run(&host);

        assert_eq!(host.calls(), 1);
        assert_eq!(report.findings.len(), 1, "{report:?}");
        let Finding::Unattributed { commit, reason } = &report.findings[0] else {
            panic!("{report:?}");
        };
        assert_eq!(commit, &stray);
        assert!(reason.contains("host unavailable"), "{reason}");
        assert!(report.findings[0].line().contains("unattributed"));
        assert!(world.landings().is_empty());
        assert_eq!(
            world.records_tip(),
            before,
            "the records branch is unchanged"
        );
    }

    #[test]
    fn no_intent_and_no_pull_request_is_reported_never_guessed() {
        let world = World::new();
        let base = world.tip();
        // Nearby intents whose tree and base differ from the stray commit's.
        world.intent(1, &base);
        let (head_b, _) = world.intent(2, &base);
        let stray = world.human("stray");
        let before = world.records_tip();

        let none = FakeHost::new(Answer::Nothing);
        let report = world.run(&none);
        assert_eq!(none.calls(), 1);
        assert!(
            matches!(&report.findings[..], [Finding::Unattributed { commit, .. }] if commit == &stray),
            "{report:?}"
        );

        // The host names a pull request that no intent records.
        let unknown = FakeHost::new(Answer::Pr(99, "f".repeat(40)));
        let report = world.run(&unknown);
        assert!(
            matches!(&report.findings[..], [Finding::Unattributed { reason, .. }] if reason.contains("#99")),
            "{report:?}"
        );

        // The host names a nearby pull request, but with a head the intent does not record.
        let wrong_head = FakeHost::new(Answer::Pr(1, head_b.clone()));
        let report = world.run(&wrong_head);
        assert!(
            matches!(&report.findings[..], [Finding::Unattributed { .. }]),
            "{report:?}"
        );

        assert!(world.landings().is_empty());
        assert_eq!(world.records_tip(), before);
    }

    #[test]
    fn host_names_the_merged_pull_request_when_no_intent_matches() {
        let world = World::new();
        let base = world.tip();
        // Pull request 1 landed and has its landing record.
        let (head1, _) = world.intent(1, &base);
        let landed = world.squash(&head1, &base);
        let host = FakeHost::new(Answer::Nothing);
        assert_eq!(world.run(&host).findings.len(), 1);
        assert_eq!(world.landings().len(), 1);

        // Pull request 7 was opened on the landed commit; then a person committed to main; then the host
        // squashed pull request 7 onto that commit, so the intent's expected base no longer matches.
        let (head7, intent7) = world.intent(7, &landed);
        let by_hand = world.human("by-hand");
        let declared = DeclaredIntervention {
            id: RecordId::generate(crate::now()),
            created_at: crate::now(),
            declarer: "maintainer".into(),
            reason: "fixed by hand".into(),
            commit_sha: by_hand.clone(),
        };
        RecordsWriter::new(&world.git)
            .append_to_remote(
                "origin",
                "Declare an intervention",
                &[Change::put(
                    format!("declared-interventions/{}.json", declared.id),
                    serde_json::to_vec_pretty(&HistoryRecord::DeclaredIntervention(declared))
                        .unwrap(),
                )],
                &[],
            )
            .unwrap();
        let merged = world.squash(&head7, &by_hand);

        let host = FakeHost::new(Answer::Pr(7, head7.clone()));
        let report = world.run(&host);

        assert_eq!(
            host.calls(),
            1,
            "only the unexplained commit is asked about: {report:?}"
        );
        assert_eq!(host.calls.borrow()[0], merged);
        assert_eq!(report.findings.len(), 1, "{report:?}");
        assert!(matches!(
            &report.findings[0],
            Finding::Repaired { pr_number: 7, source: Source::Host, commit, .. } if commit == &merged
        ));
        let landings = world.landings();
        assert_eq!(landings.len(), 2, "the first landing is untouched");
        let seven = landings.iter().find(|l| l.pr_number == 7).unwrap();
        assert_eq!(seven.intent, intent7);
        assert_eq!(seven.host_commit, merged);
        assert!(landings.iter().all(|l| l.host_commit != by_hand));
    }

    #[test]
    fn bound_skips_older_commits_and_absent_or_empty_records() {
        // A: only commits after the first intent's base and after the last landing are examined.
        let world = World::new();
        let before_everything = world.human("before-1");
        let base = world.human("before-2");
        let (head_a, _) = world.intent(1, &base);
        // Pull request 1 merged but crashed before its landing; a later landing passes over it (known gap).
        let crashed = world.squash(&head_a, &base);
        let (head_b, intent_b) = world.intent(2, &crashed);
        let landed = world.squash(&head_b, &crashed);
        world
            .sink()
            .landing(&LandingFacts {
                intent: intent_b,
                expected_base: crashed.clone(),
                pr_number: 2,
                head: head_b,
                host_commit: landed.clone(),
                merge_method: MergeMethod::Squash,
            })
            .unwrap();
        let after = world.human("after");

        let host = FakeHost::new(Answer::Nothing);
        let report = world.run(&host);
        assert_eq!(report.examined, 1, "{report:?}");
        assert_eq!(*host.calls.borrow(), vec![after.clone()]);
        let commits: Vec<&str> = report
            .findings
            .iter()
            .map(|f| match f {
                Finding::Unattributed { commit, .. } => commit.as_str(),
                other => panic!("{other:?}"),
            })
            .collect();
        assert_eq!(commits, [after.as_str()]);
        for older in [&before_everything, &base, &crashed, &landed] {
            assert!(!host.calls.borrow().contains(older));
        }

        // B: no records branch at all.
        let bare_world = World::new();
        bare_world.human("stray");
        let host = FakeHost::new(Answer::Down);
        let report = bare_world.run(&host);
        assert!(report.findings.is_empty());
        assert!(
            report
                .skipped
                .as_deref()
                .unwrap()
                .contains("does not exist")
        );
        assert_eq!(host.calls(), 0);
        assert_eq!(bare_world.records_tip(), None);

        // C: a records branch without any landing intent.
        let empty_world = World::new();
        RecordsWriter::new(&empty_world.git)
            .append_to_remote(
                "origin",
                "A note",
                &[Change::put(
                    "notes/readme.txt".to_string(),
                    b"hello".to_vec(),
                )],
                &[],
            )
            .unwrap();
        empty_world.human("stray");
        let before = empty_world.records_tip();
        let host = FakeHost::new(Answer::Down);
        let report = empty_world.run(&host);
        assert!(report.findings.is_empty());
        assert!(
            report
                .skipped
                .as_deref()
                .unwrap()
                .contains("no landing intent")
        );
        assert_eq!(host.calls(), 0);
        assert_eq!(empty_world.records_tip(), before);
    }
}
