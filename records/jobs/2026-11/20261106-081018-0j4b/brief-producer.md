# Job 20261106-081018-0j4b: Keep the failing section of a CI log, not its cleanup tail

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The check result stores the tail of the CI log, and the tail is post-job cleanup text. The failing t
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: failed-check-output-is-ci-noise-cause-of-test-failure-invisible (20261106-042835-rxkn)
> Cause: The check result stores the tail of the CI log, and the tail is post-job cleanup text. The failing test's output is cut off, so the record cannot be used to diagnose anything. Nothing extracts failing test names from the test runner, and th...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261106-042824-j07h (job 20261106-022423-vdtn); prevention it proposed: When the test check fails, Athena should capture and store the failing test names and assertion output from the test runner itself (for example the `failures:` section of cargo test), not the tail of the CI log. If no test name can be extra...
> Introspection 20261106-081000-96mr (job 20261106-045349-f6x6); prevention it proposed: When a test check fails, Athena should save the failing section of the log rather than its tail. That means the lines from "test result", "FAILED", "failures:" and "panicked at", with the test names and assertion messages. The save should a...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: `detail_from_job_logs` (crates/athena-core/src/gh/client.rs) tails the whole CI job log, which ends in runner post-job cleanup, and when no test name is parsed (e.g. a shard that did not finish, or an xtask test) the stored `output_tail` is only that noise. Fix it there: cut the post-job cleanup block before tailing and, when the tail has no failure evidence, keep the failure lines (`test result`, `FAILED`, `failures:`, `panicked at`) instead. This addresses jobs 20261106-022423-vdtn and 20261106-045349-f6x6 only as the evidence of the class; their candidate changes (config ceiling, mirrored_sources) are dropped.

Assumptions:
- Goal and audience: the stored check result is read by the verifier and by introspection, who must be able to see why a test check failed.
- Code survey: read crates/athena-core/src/gh/client.rs (detail_from_job_logs, failing_units, plain_log_line) and crates/athena-core/src/checks.rs (tail_lines, tail, failed_test_names, with_failure_detail, FAILED_TAIL_LINES). The entry point is CiSource::failure_detail, which fetches job logs and calls detail_from_job_logs; names are parsed first, then the whole log is tailed, then the FAILING_UNITS_PREFIX header is added only when names exist. The bug is that the tail is taken of the full log including post-job steps ('Post job cleanup', 'Cleaning up orphan processes', git config lines).
- No new record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected; CheckRun/FailureDetail fields stay unchanged.
- Job 20261106-022423-vdtn: its candidate (max_concurrency ceiling 8 to 16 in config.rs, docs, athena.toml, CHANGELOG) is unrelated to the cause and is dropped; its CI findings were the noise tail itself plus an xtask test flake, which this change does not address except for the output they showed. Its CHANGELOG nit is moot.
- Job 20261106-045349-f6x6: its candidate (mirrored_sources plan field, admission rule, prompts, docs) is unrelated and dropped; its findings concerned that work, and its drain-test CI failure showed the same cleanup tail, which this change fixes for later jobs.
- Not a new refusal rule: the change does not refuse readings, so no replay-refusal percentage applies.
- Tail is applied before redaction in with_failure_detail; the new trimming happens earlier, so redaction behavior is unchanged.

## Acceptance criteria

1. [content] The CHANGELOG entry (CHANGELOG.md, under Fixed) and code comment name the root cause: the stored tail was taken of the whole CI log, which ends with post-job cleanup, and failure lines were not selected.
2. [behavior] Run `cargo test -p athena-core detail_from_job_logs`: a new test feeds a job log with a failing test and a trailing post-job cleanup block (Post job cleanup, git config --unset-all, Cleaning up orphan processes, a Node.js deprecation warning) and asserts output_tail contains the `FAILED`/`panicked at` lines and no cleanup text. It fails on the base commit and passes with the change.
3. [behavior] Run `cargo test -p athena-core detail_from_job_logs`: a new test with a log that yields no parsed test names (a shard that did not finish, or non-cargo test output) followed by cleanup asserts output_tail keeps the lines containing `test result`, `FAILED`, `failures:` or `panicked at` rather than only cleanup text. It fails before the change and passes after, so it covers the class and not one job.
4. [behavior] Run `cargo test -p athena-core checks`: existing tests for tail_lines, 200-line and byte caps, FAILING_UNITS_PREFIX and redaction still pass, and a log with no failure lines and no cleanup keeps its plain tail as before.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`: all pass.
6. [content] No files outside crates/athena-core (and CHANGELOG.md) change; the 20261106-022423-vdtn and 20261106-045349-f6x6 candidate files are not reintroduced.

## Expected deliverables

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- The max_concurrency ceiling change from job 20261106-022423-vdtn (config.rs, docs, athena.toml): dropped
- The mirrored_sources plan field, admission rule and prompt changes from job 20261106-045349-f6x6 (records.rs, briefs.rs, admission.rs, planner/critic prompts, etc.): dropped
- Fixing the flaky xtask repin test or drain tests
- Editing records/, athena.toml, defaults.toml or CI workflows
- Changing the local (non-CI) failed-check tail behavior beyond what the shared helper needs

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: fix in detail_from_job_logs where the stored tail is built, with a test on the class
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: new tests plus fmt, clippy, cargo test criterion
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: tests assert the stored output content, not wiring
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: change is Rust only
- small: Prefer one focused change; size is a design choice, not a wish list. Here: one focused change in the log-to-detail path
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: verifier runs the named cargo tests
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: criteria check that the failure text is kept and cleanup text dropped
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
- 9 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-focus record, a short ordered list of themes, each with a tag or keyword set, kept in the repo and changed only by the owner through a logged command. Queue ranking would derive each item's effective priority from the focus tags: items on the top theme rank urgent, off-theme root-cause ...

32 more qualifying classes are left out.

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

- path: <state>/jobs/20261106-081018-0j4b/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-081018-0j4b/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The CHANGELOG entry (CHANGELOG.md, under Fixed) and code comment name the root cause: the stored tail was taken of the whole CI log, which ends with post-job cleanup, and failure lines were not selected.
Run `cargo test -p athena-core detail_from_job_logs`: a new test feeds a job log with a failing test and a trailing post-job cleanup block (Post job cleanup, git config --unset-all, Cleaning up orphan processes, a Node.js deprecation warning) and asserts output_tail contains the `FAILED`/`panicked at` lines and no cleanup text. It fails on the base commit and passes with the change.
Run `cargo test -p athena-core detail_from_job_logs`: a new test with a log that yields no parsed test names (a shard that did not finish, or non-cargo test output) followed by cleanup asserts output_tail keeps the lines containing `test result`, `FAILED`, `failures:` or `panicked at` rather than only cleanup text. It fails before the change and passes after, so it covers the class and not one job.
Run `cargo test -p athena-core checks`: existing tests for tail_lines, 200-line and byte caps, FAILING_UNITS_PREFIX and redaction still pass, and a log with no failure lines and no cleanup keeps its plain tail as before.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`: all pass.
No files outside crates/athena-core (and CHANGELOG.md) change; the 20261106-022423-vdtn and 20261106-045349-f6x6 candidate files are not reintroduced.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
