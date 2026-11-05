# Job 20261017-205547-cgzb: Refuse a second retry of an interrupted job; show "retried as" in status

## Request (verbatim)

> Stop `athena retry` from duplicating work on an interrupted job that was already retried, and test the restart case. After `athena retry <id>` on an interrupted job, the old job stays dispatched, `athena status` still tells the user to run `athena retry <id>` (crates/athena-cli/src/status.rs, the interrupted hint), and retry does not check for an existing retry, so following the hint starts the same work twice. Also, no test covers what a container restart leaves behind: a registry entry whose process is dead; the reap() calls in crates/athena-core/src/reading.rs and crates/athena-cli/src/drain.rs are what turn such a job into interrupted. Found by an independent spot-check of 8c20336.
> Plan: (1) A retry job's request names the job it retries ("try again on <id>"); use that to find an existing retry. (2) `athena status` shows "retried as <new id>" for such a job instead of the retry hint. (3) `athena retry <id>` refuses when a retry already exists, naming it. (4) Add a test that registers a process id that is not running for a dispatched job and shows the job is reported interrupted and is accepted by retry.
> Acceptance:
> - cargo test: after one retry of an interrupted job, a second `athena retry` of the same job fails with a message naming the first retry, and no new job is created.
> - cargo test: `athena status` for the retried job shows "retried as <new id>" and no longer suggests `athena retry`.
> - cargo test: a dispatched job whose registered process is dead is listed as interrupted and retry accepts it.
> - Existing retry and drain tests pass unchanged; clippy with -D warnings and fmt --check are clean; CHANGELOG.md has an entry.
> Out of scope: closing or rewriting the old job's records.

## Reading (accepted by the user)

For an interrupted (dispatched, no live process) job that already has a retry job ("try again on <id>"), `athena retry` refuses naming the existing retry, and `athena status` shows "retried as <new id>" instead of the `athena retry` hint. Adds tests for the double retry and for a restart leaving a dead registry entry. Audience: the user following status hints and the reviewer of 8c20336.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (Athena::retry at ~1101 calls registry().reap(), then resumable_job, then ask; earlier_attempt_with at ~1327 decides interrupted = Dispatched && !running; retried_job and job_history at ~1497-1535 already map a request 'try again on <id>' to the job it retries via records.resolve), crates/athena-core/src/status.rs (running() at 1646 checks registry entries; state_name 1650 yields 'interrupted'; status() reaps then builds the 'next' hints, the interrupted hint at ~1791), crates/athena-agent/src/procs.rs (reap at 177 removes dead groups), and crates/athena-cli/tests/retry.rs (interrupted() helper ~401, running-job test ~442 registers a live sleeper). The status code is in athena-core, not athena-cli as the request says.
- Existing code: reuse reading.rs retried_job / job_history / retry_reference and the RETRY_PREFIX request form; extend status.rs state_name/next hints and Athena::retry. No new record or data file is added, so AnyRecord and check_record_file are unaffected.
- Assumption: the refusal applies only when the job being retried is interrupted (as the request describes); retries of rejected or failed jobs behave as today so existing retry tests pass unchanged. An existing retry is any request job whose text resolves to the old job id, other than itself.
- Assumption: 'retried as <new id>' replaces the retry hint for that job in the status 'next' list and shows in the job line/state text; the close hint may remain. If several retries exist, the newest is named.
- Assumption: the dead-process test registers a pid of an exited process through the registry for a dispatched job, calls status/list and retry, and does not touch a real container.
- Refusal rule: this refuses more retries only for interrupted jobs already retried; expected 5% of newly refused readings (rare, duplicate retries only).

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test retry`: a test retries an interrupted job once, then a second `athena retry` of the same job returns an error whose message contains the first retry's job id, and the count of jobs in the records is unchanged after the second attempt.
2. [behavior] Run `cargo test`: a test shows `athena status` for the retried interrupted job contains 'retried as <new id>' and its next-step list contains no `athena retry <old id>` line.
3. [behavior] Run `cargo test`: a test registers a process id that is not running for a dispatched job; status lists the job as 'interrupted' and `athena retry` accepts it and creates a proposal.
4. [behavior] Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: all pass, with the existing retry.rs and drain.rs tests unmodified in their assertions.
5. [content] Athena::retry in crates/athena-core/src/reading.rs finds an existing retry through retried_job/job_history (the 'try again on <id>' form) rather than a new ad hoc parse, and the refusal text names the existing retry's id.
6. [content] CHANGELOG.md has an entry describing the duplicate-retry refusal, the 'retried as' status line and the restart test.
7. [content] The change refuses more retries only for already-retried interrupted jobs; expected 5% of newly refused readings.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/drain.rs (modify)

## Out of scope

- Closing or rewriting the old job's records
- Changing retry behavior for rejected, failed or timed-out jobs
- Governance files (athena.toml, defaults.toml, workflows)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test, clippy -D warnings and fmt --check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests exercising double retry, status text and dead-process interruption.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name commands the verifier can run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check lives in retry itself, not only in the status hint.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in reading.rs, status.rs and tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Messages and changelog written for outside readers.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Outcome is that following the hint no longer duplicates work.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 43 cases in 43 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a check that runs before the verifier. For each behavior criterion, it requires a recorded command and its output in the producer's report. For "fails without the fix" criteria, Athena itself should run the new tests against the pre-change source, for example by stashing the production diff and ...
- 21 cases in 19 jobs:
> Title: low-disk-ends-job-failed-no-automatic-cleanup
> Prevention: Add a disk budget that every build start must pass, whether it comes from a drain or from a hand-run command. Before starting, it takes free disk minus the writer minimum minus the estimated size of builds already running. If that is not enough, the new start waits or is refused with a message that ...
- 12 cases in 10 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer deadline hit non-destructive. On timeout, commit the working tree to a work-in-progress branch, record it in the job's records, and mark the job held or resumable, not failed. A retry then resumes from that work. As a second part, count deadline losses per week and have Athena automa...
- 11 cases in 9 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: Build a revert-run capability into Athena's checks. For any criterion tagged 'fails without the fix', Athena itself reverts the named production change on a scratch copy of the final tree. It runs the named test there and records the command, the exit status and the failing output under 'Checks Athe...
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

28 more qualifying classes are left out.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

Before you finish, run `cargo test -p xtask --test policy` on your changed files. It is the same scan Athena runs, and it names each file, line and word it rejects. Reword any banned word it finds, then run it again until it passes. It reads only tracked files, so it does not see a file you created: reread each new file against the words above.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

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

- path: <state>/jobs/20261017-205547-cgzb/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261017-205547-cgzb/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

Run `cargo test -p athena-cli --test retry`: a test retries an interrupted job once, then a second `athena retry` of the same job returns an error whose message contains the first retry's job id, and the count of jobs in the records is unchanged after the second attempt.
Run `cargo test`: a test shows `athena status` for the retried interrupted job contains 'retried as <new id>' and its next-step list contains no `athena retry <old id>` line.
Run `cargo test`: a test registers a process id that is not running for a dispatched job; status lists the job as 'interrupted' and `athena retry` accepts it and creates a proposal.
Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: all pass, with the existing retry.rs and drain.rs tests unmodified in their assertions.
Athena::retry in crates/athena-core/src/reading.rs finds an existing retry through retried_job/job_history (the 'try again on <id>' form) rather than a new ad hoc parse, and the refusal text names the existing retry's id.
CHANGELOG.md has an entry describing the duplicate-retry refusal, the 'retried as' status line and the restart test.
The change refuses more retries only for already-retried interrupted jobs; expected 5% of newly refused readings.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
