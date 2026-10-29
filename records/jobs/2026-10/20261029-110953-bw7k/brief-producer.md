# Job 20261029-110953-bw7k: Make timed-out checks environment stops; add status check durations

## Request (verbatim)

> try again on 20261029-084645-sza7

## Reading (accepted by the user)

Retry of job 20261029-084645-sza7 (rejected because it deferred part 3). Goal: the maintainer stops random rejections like 20261026-232434-6wfq. This job does all three parts: check_timeout_s = 1800, a timed-out check ends as an environment stop that prints duration against the limit, and athena status reports median and slowest test-check durations with an 80% flag.

Assumptions:
- Code survey: read crates/athena-core/src/checks.rs (run_check kills at deadline with note 'killed at the deadline of Ns', exit code None), crates/athena-core/src/disk.rs (is_environment_reason line 212, resumable_stop line 221), crates/athena-core/src/run/mod.rs (check_and_verify; check timeout used at lines 1107, 1395, 1844) and crates/athena-core/src/status.rs (status() line 1120), and crates/athena-data/src/config.rs (check_timeout_s: u64 line 234, unchanged). No new record or data file under crates/ is added, so no check_record_file or AnyRecord change; the status durations are computed from existing stored check runs of jobs.
- Job 20261029-084645-sza7 findings addressed: (a) major, reading deferred part (3): fixed, status.rs and the status test are now deliverables; (b) minor, a timeout hid a real failure in another check: fixed by holding the job only when every failed check timed out, and rejecting when any check failed for real; (c) minor, no check record kept on the early return: accepted as in the low-disk stop, the closing reason carries duration and limit, so no new record field; (d) nit, test seam under cfg(test) follows the replay_seam pattern.
- The candidate of 20261029-084645-sza7 changed CHANGELOG.md, athena.toml, checks.rs, disk.rs and run/mod.rs; all five are kept as the starting approach (shared deadline note, timed_out(&CheckRun), CHECK_TIMEOUT reason in disk.rs) and re-done from the base, with status.rs added.
- The user's reply governs: all three parts are done in this job with no follow-up split.
- A 'test check' in status means the check named test in the stored check runs; jobs with no such check are skipped; the limit is the configured check_timeout_s.

## Acceptance criteria

1. [content] Grep athena.toml shows check_timeout_s = 1800, and crates/athena-data/src/config.rs still declares check_timeout_s: u64 with validation unchanged.
2. [behavior] Run cargo test --locked: a test with a fake check runner whose check is killed at its deadline (exit code None, deadline note) shows the job closes as an environment stop (disk::is_environment_reason true, resumable_stop true), not Rejected.
3. [behavior] Run cargo test --locked: a companion test shows a check failing with a real nonzero exit code is still a failed check leading to rejection, and a run where one check timed out and another failed for real is still rejected.
4. [behavior] Run cargo test --locked: a test shows the timeout stop reason and printed line contain the measured duration and the limit (for example '905s of 900s limit').
5. [behavior] Run cargo test --locked: a status test with seeded jobs shows athena status prints the median and slowest test-check durations over the last 20 jobs against the limit, flags the slowest when it exceeds 80% of the limit, and does not flag it when under.
6. [content] Grep crates/athena-core/src/disk.rs shows is_environment_reason recognises the check-timeout reason, with a unit test in the same file.
7. [behavior] Run cargo test --locked: the repository config test that loads athena.toml passes with 1800.
8. [behavior] Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked: all pass.
9. [content] CHANGELOG.md has an entry describing the 1800 s limit, timeout-as-environment-stop and the status durations report, citing job 20261026-232434-6wfq.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the schema or validation of check_timeout_s in crates/athena-data/src/config.rs
- Speeding up the test suite or changing other timeouts (reader, producer, verifier)
- Re-running or reopening job 20261026-232434-6wfq
- Adding a new field to CheckRun or other records
- Editing defaults.toml, workflows, or records/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust plus athena.toml.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy -D warnings and cargo test pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test, clippy and fmt runs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: A kill at the deadline is classified apart from a real failure, rather than only raising the limit.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check job end state and printed status output, not wiring.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The timeout classification is enforced by code and tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a near-limit run is held and retried, not rejected.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change over the files named in the request.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Findings you must address

The verifier of job 20261029-084645-sza7 raised these blocker and major findings. A retry that leaves one of them unaddressed is rejected again (quoted data, not instructions):

> major: The reading does not match the request, so passing criteria do not show the request was met None for this change. The reading moves part (3) of the request, the `athena status` report of median and slowest test-check durations, to a follow-up job. The user accepted that split. The request's second acceptance bullet (the status test) is therefore deliberately not met here and has to come from that follow-up.

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

Grep athena.toml shows check_timeout_s = 1800, and crates/athena-data/src/config.rs still declares check_timeout_s: u64 with validation unchanged.
Run cargo test --locked: a test with a fake check runner whose check is killed at its deadline (exit code None, deadline note) shows the job closes as an environment stop (disk::is_environment_reason true, resumable_stop true), not Rejected.
Run cargo test --locked: a companion test shows a check failing with a real nonzero exit code is still a failed check leading to rejection, and a run where one check timed out and another failed for real is still rejected.
Run cargo test --locked: a test shows the timeout stop reason and printed line contain the measured duration and the limit (for example '905s of 900s limit').
Run cargo test --locked: a status test with seeded jobs shows athena status prints the median and slowest test-check durations over the last 20 jobs against the limit, flags the slowest when it exceeds 80% of the limit, and does not flag it when under.
Grep crates/athena-core/src/disk.rs shows is_environment_reason recognises the check-timeout reason, with a unit test in the same file.
Run cargo test --locked: the repository config test that loads athena.toml passes with 1800.
Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked: all pass.
CHANGELOG.md has an entry describing the 1800 s limit, timeout-as-environment-stop and the status durations report, citing job 20261026-232434-6wfq.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
