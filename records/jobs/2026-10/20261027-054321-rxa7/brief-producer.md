# Job 20261027-054321-rxa7: Verify the whole verifier-snapshot lock holds before the verifier runs

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The verifier runs with write access to the same snapshot it judges, and nothing enforces that the sn
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: verifier-snapshot-mutated-during-verification (20261015-033453-s2ry)
> Cause: The verifier runs with write access to the same snapshot it judges, and nothing enforces that the snapshot is read-only or compares a fingerprint before and after. The only detection is a late finding at the end, so a verification run can i...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261015-033445-dtwk (job 20261015-031159-xx7n); prevention it proposed: Run the verifier against a read-only snapshot, for example a read-only mount or a copy with write permissions removed. Hash the snapshot before and after verification. If the hashes differ, discard that verdict and automatically re-run veri...
> Introspection 20261015-040355-pwpg (job 20261015-033631-x3a1); prevention it proposed: Run the verifier against a read-only copy or mount of the snapshot, so writes fail at the filesystem level. Also record a content fingerprint of the snapshot before verification and compare it after. If it differs, discard that verdict and ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: `Snapshot::lock` in crates/athena-git/src/snapshot.rs treats the lock as held after probing only two places (creating a file at the root, appending to .git/HEAD). The immutable attribute is applied best-effort with `chattr -R`, so a partial lock can pass the probe while other paths (objects, refs, work-tree files) stay writable. The verifier can then change HEAD or the tree, as in jobs 20261015-031159-xx7n and 20261015-033631-x3a1. The fix makes the lock check every directory and file in the tree and fail if any accepts a write, so a verifier never starts on a partly locked snapshot. The audience is the Athena operator, who gets verdicts that are not wrongly voided.

Assumptions:
- Code survey: I read crates/athena-git/src/snapshot.rs (`Snapshot::lock`, `probe_write`, `set_tree_writable`, `set_immutable`, `work_tree_tree`, and the tests at about lines 755-790) and crates/athena-core/src/run/verification.rs (the verifier loop at about lines 198-346 locks, invokes, unlocks, compares HEAD and tree to `prepared`, and reruns once). The entry point is `verification.rs` calling `vsnap.lock()` before `invoke_and_record`. The change adds no record or data file under crates/, so no `check_record_file`, `AnyRecord` or round-trip test is involved. The property is measured directly, by attempting writes at every path of a real locked snapshot rather than through a stand-in.
- Environment: none. The change is local code and tests. It touches no CI workflow, push or PR, and no records or history reads.
- The rerun-once and tree-comparison logic in verification.rs already exists and stays as it is. The leftover cause is that `lock()` can report success while part of the snapshot is still writable, so the fix belongs in `lock()`.
- Job 20261015-031159-xx7n (the firstmate explainer rating) was rejected only because of the snapshot-change blocker. Its minor and nit findings about the page content are not addressed here. Its candidate files (CHANGELOG.md, docs/design/explainer-site.md, studies/firstmate-work-handling/intro.json, xtask/src/site.rs) are dropped from this change. Job 20261015-033631-x3a1 was a retry with the same blocker. Its candidate files (the four above plus studies/firstmate-work-handling/findings.json and model.json) are also dropped. The retry of those content jobs is separate work.
- Tests must run as whatever user the test runner is. A test of the new check should make part of the tree writable after a partial lock, for example by restoring a write bit on one nested directory or file, and assert that the check reports it. It must not depend on `chattr` being available.

## Acceptance criteria

1. [content] The `lock()` doc comment or the CHANGELOG.md [Unreleased] entry names the root cause: a lock probed at only two places can be taken for held while other paths in the snapshot remain writable, so the verifier could change its snapshot.
2. [content] In crates/athena-git/src/snapshot.rs, `Snapshot::lock` checks every directory and file under the snapshot, `.git` included, with a check that writes nothing permanent. The check does not stop at the first two probes. If any path would accept a write, `lock()` unlocks and returns an error naming that path.
3. [behavior] Run `cargo test -p athena-git`. A new test partly unlocks a nested directory or file of a locked snapshot, or simulates a partial lock, and asserts the full-tree check reports that path. The test fails when `lock()` is reverted to the two-place probe and passes with the change. It does not rely on `chattr`.
4. [behavior] Run `cargo test -p athena-git` and `cargo test -p athena-core`. The existing tests pass: `a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree` in athena-git and the snapshot-change rerun tests in crates/athena-core/src/run_pin_tests.rs.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass.
6. [content] CHANGELOG.md has an [Unreleased] entry describing the fix, since it is a user-visible change in verification behavior.

## Expected deliverables

- crates/athena-git/src/snapshot.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)

## Out of scope

- Retrying the firstmate explainer content: studies/firstmate-work-handling/*, xtask/src/site.rs, docs/design/explainer-site.md. The earlier candidates' changes to these files are dropped.
- Changing the rerun-once policy or the tree comparison in verification.rs, except to surface a lock failure.
- Hashing gitignored files such as target/ in the before/after fingerprint.
- Changes to records/, governance files (athena.toml, defaults.toml, workflows) or prompts.

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in `Snapshot::lock`, which all snapshot locking goes through, not in one job's verifier run.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A new test fails on the old lock and passes on the new one. fmt, clippy and test must pass.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the lock check and its tests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that a partly locked snapshot is refused, not that a function exists.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check behind the work is that a snapshot reported as locked refuses writes everywhere.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
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
- 11 cases in 11 jobs:
> Title: reading-or-provenance-mismatch-caught-only-at-final-verdict
> Prevention: Add a code check that runs before the final verdict. For a job whose criteria name specific job ids, the check reads the live records and lists, per id, whether an outcome record exists and whether a closed job cites a commit hash found in git log. It reports any gap to the producer while it can sti...

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

- path: <state>/jobs/20261027-054321-rxa7/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054321-rxa7/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The `lock()` doc comment or the CHANGELOG.md [Unreleased] entry names the root cause: a lock probed at only two places can be taken for held while other paths in the snapshot remain writable, so the verifier could change its snapshot.
In crates/athena-git/src/snapshot.rs, `Snapshot::lock` checks every directory and file under the snapshot, `.git` included, with a check that writes nothing permanent. The check does not stop at the first two probes. If any path would accept a write, `lock()` unlocks and returns an error naming that path.
Run `cargo test -p athena-git`. A new test partly unlocks a nested directory or file of a locked snapshot, or simulates a partial lock, and asserts the full-tree check reports that path. The test fails when `lock()` is reverted to the two-place probe and passes with the change. It does not rely on `chattr`.
Run `cargo test -p athena-git` and `cargo test -p athena-core`. The existing tests pass: `a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree` in athena-git and the snapshot-change rerun tests in crates/athena-core/src/run_pin_tests.rs.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass.
CHANGELOG.md has an [Unreleased] entry describing the fix, since it is a user-visible change in verification behavior.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
