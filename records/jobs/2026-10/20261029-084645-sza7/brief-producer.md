# Job 20261029-084645-sza7: Raise check timeout to 1800s; treat a timed-out check as an environment

## Request (verbatim)

> Priority: urgent
> The test check is now cut off at random. [budget] check_timeout_s in athena.toml is 900 seconds, and the full suite (1617 tests) takes 750 to 940 seconds, so a run near the limit is killed partway and recorded as a failed test check: on 2026-10-29 job 20261026-232434-6wfq was rejected with 'check test failed' after only 591 tests ran, exit code none, 905 s, while the verifier's own full run of the same tree passed. Fix: (1) raise check_timeout_s to 1800 in athena.toml, with the schema accepting it unchanged; (2) record a check killed by its time limit as an environment stop (the existing environment-reason path), not a failed check, so the job is held and retried rather than rejected, and print the measured duration against the limit; (3) report in athena status the median and slowest test-check durations over the last 20 jobs against the limit, flagged when the slowest exceeds 80% of it.
> Acceptance:
> - A test with a fake check runner that exceeds its time limit shows the job ends as an environment stop, not a rejection, and a real test failure is still a failed check.
> - A test shows status reports the median and slowest test-check durations and flags one over 80% of the limit.
> - athena.toml has check_timeout_s = 1800 and the repository config test passes.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: athena.toml, crates/athena-core/src/run/mod.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/status.rs, CHANGELOG.md

## Reading (accepted by the user)

Goal: stop random rejections of jobs (such as 20261026-232434-6wfq) whose full test check is killed near the 900 s limit. This change does parts (1) and (2): raise check_timeout_s to 1800 in athena.toml, and make a check killed at its deadline an environment stop (held and retried, not rejected) that prints the measured duration against the limit. Part (3), the athena status durations report, is a follow-up request.

Assumptions:
- Code survey: read crates/athena-core/src/checks.rs (run_check polls to a deadline, kills the process group and sets note 'killed at the deadline of Ns', exit_code None, passed false), crates/athena-core/src/disk.rs (is_environment_reason matches a reason prefix; resumable_stop resumes only Failed with an environment reason), crates/athena-core/src/run/mod.rs (line ~554 holds only Failed plus environment reason; checks run via SnapExec with timeout from budget.check_timeout_s at ~1107 and ~1395; the check summary line prints 'checks ... (Ns)' at ~1447), crates/athena-data/src/config.rs (check_timeout_s: u64, so 1800 needs no schema change). Entry point is produce_and_verify; baseline, then checks, then verification. The duration is measured directly from CheckRun.duration, not a stand-in. No new record or data file is added, so no AnyRecord or check_record_file change.
- The timeout is detected from the CheckRun the runner returns: a check killed at its deadline is told apart from a real failure by a single shared marker or helper in checks.rs (the existing deadline note), not by a new record field. The fake-runner test supplies such a CheckRun.
- The environment stop reuses the existing environment-reason path: the job closes Failed with a reason that disk.rs is_environment_reason accepts (a new reason prefix for 'check timed out'), so resumable_stop and athena retry treat it as held and resumable.
- Rejected job 20261026-232434-6wfq is not re-run by this change; it is addressed by fixing the root cause (limit and classification) so its failure mode no longer rejects. Its own candidate was a different refactor and is not touched.
- Part (3), status median and slowest durations with an 80% flag, is a separate follow-up so one run fits inside verifier_timeout_s.

## Acceptance criteria

1. [content] athena.toml has check_timeout_s = 1800 (Grep athena.toml), and crates/athena-data/src/config.rs still declares check_timeout_s: u64 unchanged.
2. [behavior] Run cargo test --locked: a new test with a fake check runner whose check is killed at its deadline (exit code None, deadline note) shows the job ends closed as an environment stop (disk::is_environment_reason true on its reason, resumable_stop true), not Rejected; the test asserts this.
3. [behavior] Run cargo test --locked: a companion test shows a fake check that fails with a real nonzero exit code is still a failed check that leads to rejection, not an environment stop.
4. [behavior] Run cargo test --locked: a test shows the timeout stop message and printed line contain the measured duration and the limit (for example '905s of 900s limit'), and the repository config test (the one loading athena.toml) passes with 1800.
5. [content] crates/athena-core/src/disk.rs recognises the new check-timeout reason in is_environment_reason (Grep disk.rs), with a unit test in the same file for it.
6. [behavior] Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked: all pass.
7. [content] CHANGELOG.md has an entry describing the 1800 s limit and the timeout-as-environment-stop behavior, citing job 20261026-232434-6wfq.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Part (3): the athena status report of median and slowest test-check durations over the last 20 jobs with an 80% flag, and crates/athena-core/src/status.rs; submit as a follow-up request
- Changing the schema or validation of check_timeout_s in crates/athena-data/src/config.rs
- Speeding up the test suite or changing other timeouts (reader, producer, verifier)
- Re-running or reopening job 20261026-232434-6wfq
- Adding a new field to CheckRun or other records
- Editing defaults.toml, workflows, or records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests plus cargo test, clippy -D warnings and fmt to pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Status reporting split into a follow-up so this is one focused change.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise timeout versus real failure with a fake runner rather than checking wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes both the limit and the misclassification of a killed check as a failed one.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The classification is enforced in code and tests, not prose.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry and printed message name the duration and limit for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 33 cases in 33 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a repo test or xtask check that reads the gh module's use and path references and fails on any reference from a lower row (transport, remote, client, pages, records) to a higher row, and on any cycle among rows. The check should run with the job's other checks so the producer sees the violation ...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...
- 7 cases in 7 jobs:
> Title: reading-scope-exceeds-one-job-deadline
> Prevention: Add a retry-time scope check. When a job has failed or been rejected twice, compare the reading's deliverables to the earlier attempts' readings. If the set of files or readers grew, or the attempt count reaches a limit (for example 3), stop the retry. Have the planner split the request into the sma...

19 more qualifying classes are left out.

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

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

athena.toml has check_timeout_s = 1800 (Grep athena.toml), and crates/athena-data/src/config.rs still declares check_timeout_s: u64 unchanged.
Run cargo test --locked: a new test with a fake check runner whose check is killed at its deadline (exit code None, deadline note) shows the job ends closed as an environment stop (disk::is_environment_reason true on its reason, resumable_stop true), not Rejected; the test asserts this.
Run cargo test --locked: a companion test shows a fake check that fails with a real nonzero exit code is still a failed check that leads to rejection, not an environment stop.
Run cargo test --locked: a test shows the timeout stop message and printed line contain the measured duration and the limit (for example '905s of 900s limit'), and the repository config test (the one loading athena.toml) passes with 1800.
crates/athena-core/src/disk.rs recognises the new check-timeout reason in is_environment_reason (Grep disk.rs), with a unit test in the same file for it.
Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked: all pass.
CHANGELOG.md has an entry describing the 1800 s limit and the timeout-as-environment-stop behavior, citing job 20261026-232434-6wfq.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
