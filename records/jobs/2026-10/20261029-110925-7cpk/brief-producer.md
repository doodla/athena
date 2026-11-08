# Job 20261029-110925-7cpk: Refuse landing with a clear error when the working copy is off-branch

## Request (verbatim)

> Before drain lands a verified job, check that the working copy it commits to is on the expected branch (HEAD not detached, branch matches the drain branch). If it is not, stop with a clear error that names the actual HEAD state and the expected branch, instead of failing as a missing verification. Acceptance: (1) a test that detaches HEAD in the drain working copy shows landing refuses with a message naming the detached HEAD and the expected branch, and lands nothing; (2) a test where HEAD is on the expected branch lands as before; (3) fmt, clippy and the existing tests pass.
> 
> Acceptance:
> - The change is made as described: Drain's landing failed twice and stopped after a hand command detached HEAD in its working copy; nothing checks the branch before landing
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/introspections/2026-10/20261029-045649-k845.json

## Reading (accepted by the user)

Before a verified job lands, Athena checks that the working copy is on a branch (HEAD not detached) and that it is the expected branch (the one the job was dispatched from, which drain works on). If not, landing stops with an error naming the actual HEAD state and the expected branch, lands nothing, and no longer surfaces as a missing verification. Audience: the operator running drain.

Assumptions:
- Code survey: read crates/athena-core/src/land.rs (plan() already returns Plan::OtherBranch for a branch mismatch or detached HEAD against the job's recorded dispatch_branch), crates/athena-core/src/run/close.rs (land_or_block takes the repository lock, computes head, calls plan; OtherBranch is turned into a stale outcome or a bail when the branch lacks the job's records), crates/athena-cli/src/commands.rs (drain entry, open(true)), crates/athena-cli/src/drain.rs (drain_with, Steps landing) and crates/athena-git/src/lib.rs (Git::current_branch returns None when detached). The records validation allow-list and AnyRecord are untouched because no record or data file is added.
- Reading of 'drain branch': the branch the job was dispatched from (job record dispatch_branch), which is the branch the drain works on. No separate drain-branch setting exists in the code I read. If the job record has no dispatch branch (old record), the expected branch is the branch the drain started on only if cheaply available; otherwise the check only rejects a detached HEAD.
- Today a detached HEAD at landing is routed through Plan::OtherBranch into a recorded stale outcome or a generic bail, which the drain reports as a failed/missing verification. The fix is a precondition before landing (early in land_or_block, before any commit is built) that bails with a message such as 'working copy HEAD is detached at <short sha>; expected branch `main`; nothing landed'. The existing tests a_branch_job_finished_on_a_detached_head_is_refused and a_job_dispatched_on_a_detached_head_never_lands in land.rs may need adjusting only if the message or path changes; their intent stays.
- Tests go in crates/athena-cli/tests/lifecycle.rs or crates/athena-core (whichever fixture can detach HEAD with git and run the landing); they must exercise the real landing path, not just plan().
- Out of scope: the --pr landing path, rebase logic, stale-detection rules, and drain stop/retry policy.

## Acceptance criteria

1. [behavior] A new test detaches HEAD in the working copy of a verified job and attempts landing: it fails with an error containing 'detached' and the expected branch name (e.g. `main`), and afterwards the branch tip is unchanged and no landing commit exists. Check: cargo test, naming the test.
2. [behavior] A new test with HEAD on the expected branch lands the verified job as before (landing commit present, tip advanced). Check: cargo test.
3. [behavior] The error for the detached case is raised before any closing commit or stale outcome is built, and does not read as a missing verification: the message does not contain 'verification' wording about a missing record. Check: diff of crates/athena-core/src/run/close.rs / land.rs and the test assertion.
4. [behavior] cargo fmt --check, cargo clippy (all targets, warnings denied as configured) and cargo test for the whole workspace pass, including the existing land.rs and lifecycle/drain tests.
5. [content] The diff changes only landing precondition code and its tests, plus a CHANGELOG.md line if the repo convention requires one; no other landing behavior (stale rules, merge-tree, fingerprint) changes.
6. [content] The refusal narrows landing only for an off-branch working copy; expected 0% of newly refused readings, since past readings are not affected by the branch check and the higher refusal of off-branch landings is intended.

## Expected deliverables

- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- The --pr landing path (gh/landing.rs)
- Changing stale-detection, rebase or fingerprint rules
- Editing records/introspections/2026-10/20261029-045649-k845.json (a source note, not a deliverable)
- Governance files such as athena.toml and defaults.toml

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for detached and on-branch landing; fmt, clippy, test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that detach HEAD and check nothing landed.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Check sits at the landing precondition rather than patching the symptom message.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused precondition change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
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

- path: <state>/jobs/20261029-110925-7cpk/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-110925-7cpk/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Make your evidence checkable

Every acceptance criterion must be checkable from the snapshot, the diff or the commands the verifier runs, never from the producer's report or change description, which the verifier does not see. Prefer a test in your change that shows the property. Add a new file for evidence only when a deliverable or a criterion names it. Your report is still where you state what you ran or read; the rule only means the verifier does not rely on it. If a criterion can be shown only from your report, report it unmet and begin its evidence with `not checkable by the verifier`; do not invent a file.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

A new test detaches HEAD in the working copy of a verified job and attempts landing: it fails with an error containing 'detached' and the expected branch name (e.g. `main`), and afterwards the branch tip is unchanged and no landing commit exists. Check: cargo test, naming the test.
A new test with HEAD on the expected branch lands the verified job as before (landing commit present, tip advanced). Check: cargo test.
The error for the detached case is raised before any closing commit or stale outcome is built, and does not read as a missing verification: the message does not contain 'verification' wording about a missing record. Check: diff of crates/athena-core/src/run/close.rs / land.rs and the test assertion.
cargo fmt --check, cargo clippy (all targets, warnings denied as configured) and cargo test for the whole workspace pass, including the existing land.rs and lifecycle/drain tests.
The diff changes only landing precondition code and its tests, plus a CHANGELOG.md line if the repo convention requires one; no other landing behavior (stale rules, merge-tree, fingerprint) changes.
The refusal narrows landing only for an off-branch working copy; expected 0% of newly refused readings, since past readings are not affected by the branch check and the higher refusal of off-branch landings is intended.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
