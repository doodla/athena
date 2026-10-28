# Job 20261028-081326-zzxg: Free producer build dir before verifier; resume retry from kept

## Request (verbatim)

> Priority: urgent
> Two gaps make a finished job fail twice on disk: on 2026-10-28 a job passed planning, built, and passed every Athena check, then stopped before the verifier with 9.5 to 9.8 GB free against the 10 GB minimum, and its retry re-planned and rebuilt from scratch for about $5 instead of resuming. Fix both. (1) Disk: once Athena's own checks have passed, remove the job's producer build directory (target/produce) before the verifier starts, since nothing reads it after the checks; the verifier then has the space. (2) Resume: when a job stops for an environment cause (low disk, usage limit, interrupted process) after its candidate is built, athena retry resumes from the kept candidate at the stage that stopped, with no new planning or producer run, the same way a timed-out producer's work is kept.
> Acceptance:
> - A test shows the producer build directory is gone after checks pass and before the verifier runs, and is kept when checks fail (for the send-back).
> - A test stops a job on low disk before the verifier and shows athena retry runs only the verifier on the kept candidate, with no planner or producer invocation.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/reading.rs, CHANGELOG.md

## Reading (accepted by the user)

Fix two gaps for operators of Athena: after Athena's own checks pass, delete the job's target/produce before the verifier starts (kept if checks fail, for send-back); and make `athena retry` resume a job stopped for an environment cause after build from the kept candidate at the stopped stage, with no planner or producer run.

Assumptions:
- Goal: Athena operators get jobs that finish and retries that resume cheaply.
- The existing timed-out-producer keep/resume path can be generalized to later stages.
- No governance file changes are needed.

## Acceptance criteria

1. [behavior] A test in crates/athena-core shows target/produce is absent after checks pass and before the verifier runs, and still present when checks fail (send-back). Run: cargo test
2. [behavior] A test stops a job on low disk before the verifier, then runs retry and asserts only the verifier ran on the kept candidate, with zero planner and zero producer invocations. Run: cargo test
3. [content] Retry resume logic in crates/athena-core/src/run/mod.rs and reading.rs treats low disk, usage limit and interrupted process after the candidate is built as resumable at the stopped stage, using the same keep mechanism as a timed-out producer.
4. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
5. [content] CHANGELOG.md has an entry describing both fixes.
6. [content] Outcome check: a job that passes checks with 9.5 GB free no longer stops on disk for lack of target/produce space, and its retry does not re-plan or rebuild.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/disk.rs (modify)

## Out of scope

- Changing the 10 GB minimum free-disk threshold
- Changing verifier behavior or prompts
- Cleaning other directories

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added; fmt, clippy -D warnings, cargo test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the dir removal and the retry skipping planner/producer by test.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Both the disk use and the resume gap are fixed.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion on the job finishing and retry not re-running.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Limited to the named files.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 31 cases in 31 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a test in the site build that renders the firstmate study page and asserts the output. It must contain the glossary entry and the next-step link before the first transition card. It must not contain any f2 or f3 model-limit note. The producer must run it, and a failure must block handover. More ...
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

17 more qualifying classes are left out.

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

A test in crates/athena-core shows target/produce is absent after checks pass and before the verifier runs, and still present when checks fail (send-back). Run: cargo test
A test stops a job on low disk before the verifier, then runs retry and asserts only the verifier ran on the kept candidate, with zero planner and zero producer invocations. Run: cargo test
Retry resume logic in crates/athena-core/src/run/mod.rs and reading.rs treats low disk, usage limit and interrupted process after the candidate is built as resumable at the stopped stage, using the same keep mechanism as a timed-out producer.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
CHANGELOG.md has an entry describing both fixes.
Outcome check: a job that passes checks with 9.5 GB free no longer stops on disk for lack of target/produce space, and its retry does not re-plan or rebuild.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
