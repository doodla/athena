# Job 20261106-191125-zy9w: Retry: store failure lines, not CI cleanup, in failed-check tail

## Request (verbatim)

> try again on 20261106-081018-0j4b

## Reading (accepted by the user)

Retry of job 20261106-081018-0j4b (stale: branch moved). Re-apply its verified fix so detail_from_job_logs cuts post-job cleanup and keeps failure lines, merged onto the new tip, and address its findings, mainly by showing the new tests fail on base.

Assumptions:
- Job 20261106-081018-0j4b passed verification but went stale (branch moved, e49abd98ebd1 touched governed content); its candidate is merged onto the tip. Its four changed files are kept as deliverables.
- Finding 1 (new tests not shown failing on base because they referenced missing helpers): addressed by putting the gh tests in crates/athena-core/src/gh/tests.rs against only the public detail_from_job_logs, so they compile on base and fail on assertions. Helper unit tests in checks.rs may stay but are not the base-failure evidence.
- Finding 2 (failure block keeps only marker plus 4 lines): fixed by extending a block until a blank line or next `----` header, within the n/2 cap. Finding 3 (moved lines look like last output): kept as a documented trade-off; the separator line stays and the comment states it.
- Code survey: read crates/athena-core/src/gh/client.rs (detail_from_job_logs, line 248, tails the whole job log) and crates/athena-core/src/checks.rs (tail_lines, line 193, plus its tests). No record or data file is added, so records validation, check_record_file and AnyRecord are not touched. The property is measured directly by calling detail_from_job_logs on a synthetic log and inspecting output_tail.
- Audience: Athena maintainers and later introspections diagnosing failed CI checks. The dropped candidates from 20261106-022423-vdtn (config ceiling) and 20261106-045349-f6x6 (mirrored_sources) stay dropped.
- If the merge conflicts with the tip, the producer resolves it from the conflicted merge; the shared child-process script writer change is untouched.

## Acceptance criteria

1. [content] CHANGELOG.md under Fixed and a code comment in crates/athena-core/src/gh/client.rs or checks.rs name the root cause: the stored tail was taken of the whole CI log, which ends with post-job cleanup, and failure lines were not selected.
2. [behavior] Run `cargo test -p athena-core detail_from_job_logs`: a test in gh/tests.rs with a failing test followed by a post-job cleanup block asserts output_tail contains the FAILED/panicked at lines and no cleanup text. It compiles on base using only existing public items and fails on assertions there.
3. [behavior] Run `cargo test -p athena-core detail_from_job_logs`: a test with a log yielding no parsed test names, followed by cleanup, asserts output_tail keeps lines with `test result`, `FAILED`, `failures:` or `panicked at`.
4. [behavior] Run `cargo test -p athena-core detail_from_job_logs`: a test with a multi-line panic message longer than 4 lines asserts the whole message survives in output_tail.
5. [behavior] Run `cargo test -p athena-core`: existing tail_lines, 200-line and byte cap, FAILING_UNITS_PREFIX and redaction tests pass, and a log with no failure lines and no cleanup keeps its plain tail.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`: all pass.
7. [content] Only crates/athena-core/src/gh/client.rs, gh/tests.rs, checks.rs and CHANGELOG.md change; no config, records, workflow or governance files change.

## Expected deliverables

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- The max_concurrency ceiling change from job 20261106-022423-vdtn (config.rs, docs, athena.toml): dropped
- The mirrored_sources plan field, admission rule and prompt changes from job 20261106-045349-f6x6: dropped
- Fixing the flaky xtask repin test or drain tests
- Editing records/, athena.toml, defaults.toml or CI workflows
- Changing the local (non-CI) failed-check tail behavior beyond what the shared helper needs
- Reworking the e49abd98ebd1 child-process script writer change

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in crates/athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy, test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix in detail_from_job_logs, where the tail is taken, not per job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests on synthetic logs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in four files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Independent verifier reruns the cargo checks.
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

- path: <state>/jobs/20261106-191125-zy9w/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-191125-zy9w/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## criteria_check: copy the line

Report exactly one criteria_check entry per acceptance criterion, in the order below, including criteria that were already satisfied before you began. For each entry copy the line below as the entry's criterion text, character for character; do not retype, shorten, merge or reword it. Give its status (met or unmet) and your evidence: what you read or ran. A criterion with no entry is sent back.

A criterion that only Athena's own recorded checks can show (cargo fmt --check, cargo clippy, the full or workspace cargo test) is not yours to run: report it with evidence that starts with "left to Athena's checks" and it is not sent back; the verifier judges it from the recorded checks. Targeted-test criteria and every other criterion must still be reported met with evidence of what you ran or read.

CHANGELOG.md under Fixed and a code comment in crates/athena-core/src/gh/client.rs or checks.rs name the root cause: the stored tail was taken of the whole CI log, which ends with post-job cleanup, and failure lines were not selected.
Run `cargo test -p athena-core detail_from_job_logs`: a test in gh/tests.rs with a failing test followed by a post-job cleanup block asserts output_tail contains the FAILED/panicked at lines and no cleanup text. It compiles on base using only existing public items and fails on assertions there.
Run `cargo test -p athena-core detail_from_job_logs`: a test with a log yielding no parsed test names, followed by cleanup, asserts output_tail keeps lines with `test result`, `FAILED`, `failures:` or `panicked at`.
Run `cargo test -p athena-core detail_from_job_logs`: a test with a multi-line panic message longer than 4 lines asserts the whole message survives in output_tail.
Run `cargo test -p athena-core`: existing tail_lines, 200-line and byte cap, FAILING_UNITS_PREFIX and redaction tests pass, and a log with no failure lines and no cleanup keeps its plain tail.
Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`: all pass.
Only crates/athena-core/src/gh/client.rs, gh/tests.rs, checks.rs and CHANGELOG.md change; no config, records, workflow or governance files change.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
