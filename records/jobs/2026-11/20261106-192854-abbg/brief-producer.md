# Job 20261106-192854-abbg: Keep the failing test section, not the CI cleanup tail, in failed checks

## Request (verbatim)

> Priority: normal
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

Root cause: a failed check's record keeps the last 200 lines of the log (checks.rs tail_lines in the local run, with_failure_detail, and gh/client.rs detail_from_job_logs). For a CI log those lines are post-job cleanup, which pushes the test runner's failures section out of the record. Fix it once in a shared function in checks.rs that drops post-job cleanup and keeps the failing section, and use it at all three sites. This retries jobs 20261106-022423-vdtn and 20261106-045349-f6x6 for the failure class only. Their own diffs are out of scope.

Assumptions:
- Goal and audience: the verifier, the next producer on a retry, and the person reading a failed check record must be able to see the failing test name and assertion output.
- Code survey: read crates/athena-core/src/checks.rs (tail, tail_lines, FAILED_TAIL_LINES, failed_test_names, with_failure_detail, run path around line 505-545), crates/athena-core/src/gh/client.rs (detail_from_job_logs, plain_log_line, failing_units), crates/athena-core/src/ci_result.rs (FailureDetail), and routing.rs. The entry points are the local check run in checks.rs and detail_from_job_logs, which with_failure_detail applies to a reused red CI run. Both keep a line tail and extract names only from `test ... FAILED` lines. Records validation is unchanged: no record or data file is added, so check_record_file, AnyRecord and the round-trip test are not affected. The property is measured directly on the stored output_tail of the CheckRun, not through a stand-in.
- Job 20261106-022423-vdtn: its failed test check came from CI noise unrelated to its diff (the xtask repin test and the cleanup tail). Its diff (config.rs, docs, CHANGELOG.md, athena.toml) is not redone here. Those files are out of scope and are not kept or changed by this job.
- Job 20261106-045349-f6x6: its mirrored_sources work (records.rs, briefs.rs, planning.rs, admission.rs, prompts and the other 17 changed files) is dropped from this job. Its findings of a stale target dir and an unreadable test log are the failure class this job fixes. Every file it changed is excluded in out_of_scope.
- The cleanup marker is the GitHub Actions line `Post job cleanup.` (also `##[group]Post job cleanup`). The section starts at the earliest runner marker (`failures:`, `panicked at`, `---- ... stdout ----`, `test result: FAILED`) when one exists. It is bounded by the existing FAILED_TAIL_LINES and FAILED_TAIL_BYTES. With no marker it falls back to the current tail of the cleanup-stripped text.
- Redaction still runs on the kept text; the new function only selects lines.

## Acceptance criteria

1. [content] The change description names the root cause: the failed-check record stores the last N lines of the log without regard to what they are, so CI post-job cleanup text displaces the test runner's failing section. It is fixed in one shared function in crates/athena-core/src/checks.rs that both the local run and the CI-log path (gh/client.rs detail_from_job_logs and with_failure_detail) call.
2. [behavior] A new unit test in checks.rs, run with `cargo test -p athena-core checks`, feeds a log with a `failures:` section, a `panicked at` line and a `test result: FAILED` line followed by more than 200 lines of post-job cleanup, as in the CI logs of jobs 20261106-022423-vdtn and 20261106-045349-f6x6. It asserts the kept output contains the failing test name and the assertion text and no `Post job cleanup` lines. The test fails on the base commit (verify with git stash or the fails-without run) and passes with the change.
3. [behavior] A test in gh/tests.rs, run with `cargo test -p athena-core gh`, builds a CI job log whose failing output has no parseable `test ... FAILED` names (for example a panic in an xtask test) followed by cleanup text. It asserts detail_from_job_logs returns an output_tail with the panic message and not the cleanup lines. It fails on the base and passes with the change.
4. [behavior] A log with no runner marker and no cleanup text keeps the same output as today (tail of the last FAILED_TAIL_LINES lines), and a passing check's kept output is unchanged. Existing tests, including red_reused_run_records_failing_tests_and_log_tail and the run/tests.rs reuse test, still pass under `cargo test --locked`.
5. [behavior] `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` pass.
6. [content] CHANGELOG.md has one entry under Fixed or Changed stating that a failed check record now keeps the failing test section instead of the log tail. It states only what the diff does.
7. [content] Not a change to refusal rules: no check is added that refuses or holds work, so no replay refusal-rate criterion applies.

## Expected deliverables

- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)

## Out of scope

- Job 20261106-022423-vdtn's diff: crates/athena-data/src/config.rs, docs/design/agent-and-build-slots.md, docs/design/async-drain.md, docs/design/parallel-jobs.md, athena.toml, and the max_concurrency ceiling change
- Job 20261106-045349-f6x6's diff: mirrored_sources and the prompt, admission, plan and records files it changed (crates/athena-context/prompts/planner.md and critic.md, briefs.rs, outputs.rs, admission.rs, reading.rs, run/mod.rs, run/planning.rs, run_pin_tests.rs, status.rs, store.rs, refusal_corpus.rs, records.rs, job_plan.rs, docs/design/job-planning.md, docs/system-facts.md)
- Fixing the flaky xtask repin test or the stale shared target dir
- Changes to records/, athena.toml, defaults.toml or CI workflows
- Changing the stored record schema (CheckRun fields)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The fix is Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: One shared selector fixes every path that stores a log tail.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The tests assert the stored output content.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the checks module and its CI-log caller.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier runs the tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the failing test text is in the stored record.
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

- path: <state>/jobs/20261106-192854-abbg/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-192854-abbg/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

The change description names the root cause: the failed-check record stores the last N lines of the log without regard to what they are, so CI post-job cleanup text displaces the test runner's failing section. It is fixed in one shared function in crates/athena-core/src/checks.rs that both the local run and the CI-log path (gh/client.rs detail_from_job_logs and with_failure_detail) call.
A new unit test in checks.rs, run with `cargo test -p athena-core checks`, feeds a log with a `failures:` section, a `panicked at` line and a `test result: FAILED` line followed by more than 200 lines of post-job cleanup, as in the CI logs of jobs 20261106-022423-vdtn and 20261106-045349-f6x6. It asserts the kept output contains the failing test name and the assertion text and no `Post job cleanup` lines. The test fails on the base commit (verify with git stash or the fails-without run) and passes with the change.
A test in gh/tests.rs, run with `cargo test -p athena-core gh`, builds a CI job log whose failing output has no parseable `test ... FAILED` names (for example a panic in an xtask test) followed by cleanup text. It asserts detail_from_job_logs returns an output_tail with the panic message and not the cleanup lines. It fails on the base and passes with the change.
A log with no runner marker and no cleanup text keeps the same output as today (tail of the last FAILED_TAIL_LINES lines), and a passing check's kept output is unchanged. Existing tests, including red_reused_run_records_failing_tests_and_log_tail and the run/tests.rs reuse test, still pass under `cargo test --locked`.
`cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` pass.
CHANGELOG.md has one entry under Fixed or Changed stating that a failed check record now keeps the failing test section instead of the log tail. It states only what the diff does.
Not a change to refusal rules: no check is added that refuses or holds work, so no replay refusal-rate criterion applies.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
