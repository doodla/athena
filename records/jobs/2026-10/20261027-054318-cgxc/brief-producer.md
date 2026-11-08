# Job 20261027-054318-cgxc: Close stacked earlier PRs when a later PR lands their commits

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Athena lets a job be built on an earlier unmerged job's branch, but it doesn't record that dependenc
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: stacked-job-content-lands-via-later-pr (20261015-201512-vm2y)
> Cause: Athena lets a job be built on an earlier unmerged job's branch, but it doesn't record that dependency or track the earlier pull request. When the later pull request merges, nothing links it to the earlier ones whose commits it carries. The ...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261017-125701-mkgp (job 20261010-154004-60r4); prevention it proposed: Record the base job or pull request whenever a job is cut on top of an unmerged job. After any pull request merges, check each recorded earlier pull request that is still open. If all of its commits are now reachable from main, close it aut...
> Introspection 20261017-141752-2ffq (job 20261010-154007-jbt9); prevention it proposed: Record the base job or branch when a job is cut on top of unmerged work. After any pull request merges, run a check that finds open pull requests whose head commits are all reachable from main. Close each one automatically with a comment na...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: a job can be cut on an earlier unmerged job's branch, but the pull request link (`PrLink` in crates/athena-core/src/pr.rs) records no base job or earlier PR, and nothing runs after a merge to reconcile open PRs whose commits main now carries. The fix records the base job on the link and, after a merge, closes each recorded earlier open PR whose head is reachable from the merged commit, with a comment naming the PR that carried it. This follows up jobs 20261010-154004-60r4 and 20261010-154007-jbt9, whose commits landed this way.

Assumptions:
- Code survey: read crates/athena-core/src/pr.rs (PrLink holds pr_number, host, repository, head, job, state, merged_commit; no base job; Host trait has only merged_commit; observe_merge/check_merged/apply_outcome record the merge outcome) and crates/athena-core/src/gh/landing.rs (land_pr publishes, observes the merge, merges over REST, then writes the landing record and cutover; it never looks at other jobs' links). Entry point is land_pr, run after publish and merge confirmation. The property is measured directly by unit tests with fake Host and Ancestry implementations, as the existing pr.rs tests do.
- Existing code: crates/athena-core/src/pr.rs (PrLink, store_link, load_link, Host, Ancestry, observe_merge) and crates/athena-core/src/gh/landing.rs (land_pr, confirm_existing_merge) are extended, not replaced. PrLink is a state-directory file (pr-link.json), not a records-branch record, so no AnyRecord or check_record_file change is expected. New PrLink fields use serde default so old links still parse.
- Environment: the host is GitHub through the existing gh client in crates/athena-core/src/gh; closing a pull request and commenting on it are new write calls on the same token that already merges. The tests use fakes and need no network. Data reached: the job state directory's pr-link.json files and the remote's main history.
- Reading of the cause: the originating gap is that a stacked job's dependency is never recorded, not that one job's PR was left open. The class fix is a recorded base (the earlier job's id, from the branch the job is cut on) plus a post-merge reconciliation. The reconciliation closes an earlier PR only when it is still open and every commit of its head is reachable from the merged commit. Otherwise it leaves the PR alone.
- Both referenced jobs are already landed with passing verification. Their own findings are not reopened and do not apply here: for 20261010-154004-60r4 the display.rs, lifecycle-reason and deliverable-list findings; for 20261010-154007-jbt9 the weak manage.rs subject test. Their landed code is kept, and none of their files are deliverables.
- Audience: Athena maintainers who stack jobs and review the PR list. The outcome they check is that no stale open PR remains after the later one merges.

## Acceptance criteria

1. [content] The change description names the root cause: a job built on an unmerged job's branch has no recorded dependency and no post-merge reconciliation of the earlier PRs it carries. The fix is in crates/athena-core/src/pr.rs (the link) and crates/athena-core/src/gh/landing.rs (land_pr), not in one job's data. Check with git diff.
2. [content] PrLink gains a defaulted field recording the base job (or earlier PR). A test in pr.rs shows an old pr-link.json without the field still loads (git diff shows the test).
3. [behavior] A new unit test of the reconciliation (run cargo test -p athena-core) shows an open earlier PR whose head is reachable from the merged commit is closed with a comment naming the merging PR. It fails on the pre-change code, which has no such step.
4. [behavior] A new unit test shows an earlier open PR with a commit not reachable from the merged commit is left open, and an already merged or closed PR is not touched. This guards against closing live work.
5. [behavior] A test is generic over the cause: two different earlier jobs stacked in a chain (A, then B on A, then C on B) are both closed when C lands, with no job id hard-coded. Run cargo test -p athena-core.
6. [behavior] A failure to close an earlier PR is reported as a warning in PrReport.warnings and does not undo or fail the merge or the landing record. Check with a unit test using a failing fake host (cargo test -p athena-core).
7. [content] CHANGELOG.md has an entry for the change, and docs describing the PR link or landing (docs/ file that mentions pr-link) are updated if they list the link's fields. Check with git diff and grep.
8. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.

## Expected deliverables

- crates/athena-core/src/pr.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)
- docs/records.md (modify)

## Out of scope

- Automatically detecting the base job from git history for jobs already landed
- Changing how the stacked branch is cut or how rebases work (rebase_job_branch, plan_rebase)
- Closing PRs by any rule other than all commits reachable from the merged commit
- Fixing the earlier jobs' own minor findings (display.rs provisional cost display, lifecycle test reason, manage.rs subject test input)
- Files changed by jobs 20261010-154004-60r4 and 20261010-154007-jbt9: claude.rs, scoring.rs, records.rs, athena-score lib.rs, lifecycle.rs, manage.rs, reading.rs, store.rs, athena-core lib.rs. Their landed work is kept unchanged.
- Governance files: athena.toml, defaults.toml and workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests, with clippy and fmt clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the missing dependency record and the missing post-merge reconciliation, not one job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that fail without the change.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that no stale open PR remains after a later PR lands its commits.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the PR link and landing code.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier runs cargo fmt, clippy and test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and the changelog written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 22 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk-budget check to Athena. Before a drain batch starts, and before each stage, it would total the free disk against the expected build size of running and held work. It would also count stray target folders from hand runs. When space is short it would reclaim what is safe to reclaim, such as...
- 12 cases in 10 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Add a code check before the final verdict. For every behavior criterion that names commands, it confirms the job's recorded checks include a run of those commands against the final tree, and it attaches that run record as the evidence the verifier sees. When the diff touches only docs, Athena should...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 10 cases in 10 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check at reading acceptance: when a request names a file-size or line limit, the check measures the named file in the current snapshot, adds a criterion for the limit itself, and records the measured number. The producer is then required to re-measure the file at the end and report the fi...

33 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Files kept by hand

If `crates/athena-cli/tests/mirrors.rs` exists in your working directory, run `cargo test -p athena-cli --test mirrors` before you finish. A file kept by hand as a mirror of a source (the test timings, the decision log, the CI cache key against the build paths, the ontology) is compared with that source by content, and a failure names the file and the fix. A new test file needs its line in xtask/shard-timings.txt, and docs/decisions.md only gains text after its last line.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Replay of accepted readings while you work

When your change touches code (a path under `crates/`, or Cargo.toml or Cargo.lock), Athena replays the newest accepted readings through the base build and your candidate build before it judges your report. If your change makes the reading rules refuse more than 10% of them, or more than the share the acceptance criteria declare as "N% of newly refused readings" plus 5 points, your work is sent back with the list of newly refused job ids and the measured share. A rule that is too broad shows up there. Narrow the rule so it refuses what it is meant to refuse; do not hide the cases. A replay that cannot run says `no replay` and never counts as a pass.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-054318-cgxc/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054318-cgxc/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The change description names the root cause: a job built on an unmerged job's branch has no recorded dependency and no post-merge reconciliation of the earlier PRs it carries. The fix is in crates/athena-core/src/pr.rs (the link) and crates/athena-core/src/gh/landing.rs (land_pr), not in one job's data. Check with git diff.
PrLink gains a defaulted field recording the base job (or earlier PR). A test in pr.rs shows an old pr-link.json without the field still loads (git diff shows the test).
A new unit test of the reconciliation (run cargo test -p athena-core) shows an open earlier PR whose head is reachable from the merged commit is closed with a comment naming the merging PR. It fails on the pre-change code, which has no such step.
A new unit test shows an earlier open PR with a commit not reachable from the merged commit is left open, and an already merged or closed PR is not touched. This guards against closing live work.
A test is generic over the cause: two different earlier jobs stacked in a chain (A, then B on A, then C on B) are both closed when C lands, with no job id hard-coded. Run cargo test -p athena-core.
A failure to close an earlier PR is reported as a warning in PrReport.warnings and does not undo or fail the merge or the landing record. Check with a unit test using a failing fake host (cargo test -p athena-core).
CHANGELOG.md has an entry for the change, and docs describing the PR link or landing (docs/ file that mentions pr-link) are updated if they list the link's fields. Check with git diff and grep.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
