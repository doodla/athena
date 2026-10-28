# Job 20261028-065915-a41d: Don't count environment stops as failed attempts in the scope guard

## Request (verbatim)

> Priority: high
> The scope guard paused the replay line (20261028-034820-v3rf) after an attempt whose work was complete: it passed planning, built, and passed every check, then stopped only because the disk was 0.2 GB short before the verifier. The guard counted that environment stop like a failed attempt and its spend like waste. Fix: an attempt that ended on an environment cause (low disk, usage limit, interrupted process) does not count toward the guard's attempt or round signals, and when a retry resumes a kept candidate the guard forecasts only the stages left, not a fresh job.
> Acceptance:
> - A test shows an attempt ending on low disk does not count toward a pause, and a failed verification does.
> - A test shows a retry resuming a kept candidate is judged against the cost of its remaining stages.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/scope_guard.rs, CHANGELOG.md

## Reading (accepted by the user)

The scope guard (crates/athena-core/src/scope_guard.rs) currently turns every non-landed, non-rejected closed job into a Failed attempt that counts toward T1 retries, T2 planning rounds, T3 cost and T5 time. Change it so an attempt that ended on an environment cause (low disk, usage limit, interrupted process) is excluded from the attempt and round signals, and so a retry that resumes a kept candidate is judged against the cost of its remaining stages only. Audience: the operator whose line was paused. Job 20261028-034820-v3rf is the paused line, not a job to redo: its five planning-critique findings (replay.md refused by the records check, test override placement, no skip switch, Records::load question) concern the replay feature and are not touched by this guard fix. The line is not edited, only unblocked.

Assumptions:
- Job 20261028-034820-v3rf's findings belong to the replay feature; this change only makes the guard stop counting its environment-stopped attempt, so the findings need no fix here and its own retry is out of scope.
- An environment stop is identified from the job's recorded outcome (state and reason text, e.g. low disk, usage limit, interruption) and the existing attempt_of mapping; no new record fields are added to athena-data.
- An environment-stopped attempt is excluded from T1 attempt count, T2 planning round and plan growth, T4 rejections and T5 minutes; a failed verification (Rejected) still counts.
- For a retry resuming a kept candidate, T3 compares spend against the p90 forecast scaled to the stages not yet done, and spend already made on the kept stages is not counted against it.
- Stage cost shares are fixed constants in scope_guard.rs, labelled as estimates.
- Thresholds and athena.toml are unchanged, so no governance file changes.

## Acceptance criteria

1. [behavior] A unit test in crates/athena-core/src/scope_guard.rs builds a line whose earlier attempt ended on low disk and shows evaluate_before_attempt gives no T1/T2/T5 pause or flag from it, while the same line with a Rejected verification does. Run: cargo test -p athena-core scope_guard.
2. [behavior] A unit test in scope_guard.rs shows a retry that resumes a kept candidate (planning, build and checks done) is judged against the cost of its remaining stages: spend that would pause as a fresh job does not pause, and the same spend on a non-resuming retry does. Run: cargo test -p athena-core scope_guard.
3. [behavior] The existing scope_guard tests (t4_pause_via_go_and_ask, retry_refuses_paused_line, t3_pause_from_reading_forecast and the rest) still pass, so real rejections and cost overruns still pause lines. Run: cargo test.
4. [behavior] cargo fmt --check and cargo clippy --all-targets -- -D warnings are clean.
5. [content] CHANGELOG.md has an entry naming the scope guard change: environment-ended attempts are not counted, and resumed retries are forecast on remaining stages.
6. [content] Each new constant or rule in scope_guard.rs has a doc comment stating its basis or marking it an estimate, and the diff touches only crates/athena-core/src/scope_guard.rs and CHANGELOG.md.

## Expected deliverables

- crates/athena-core/src/scope_guard.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/mod.rs (modify)

## Out of scope

- Changing athena.toml, defaults.toml or workflows
- Changing the guard thresholds or the T4 same-criterion rule
- Implementing the replay feature of job 20261028-034820-v3rf or retrying that job
- Changing the disk check or how outcomes are recorded in athena-data
- Unpausing or editing existing pause records

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests are required, and cargo test and clippy -D warnings must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The new rule is enforced in code by the guard's signal functions, with tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix changes how the guard classifies attempts, not just this one paused line.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria run the tests and check pause and no-pause results.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a complete attempt stopped on the environment no longer pauses the line.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The change is one focused edit to the guard plus a changelog entry.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments explain the new rule and any estimate.
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

A unit test in crates/athena-core/src/scope_guard.rs builds a line whose earlier attempt ended on low disk and shows evaluate_before_attempt gives no T1/T2/T5 pause or flag from it, while the same line with a Rejected verification does. Run: cargo test -p athena-core scope_guard.
A unit test in scope_guard.rs shows a retry that resumes a kept candidate (planning, build and checks done) is judged against the cost of its remaining stages: spend that would pause as a fresh job does not pause, and the same spend on a non-resuming retry does. Run: cargo test -p athena-core scope_guard.
The existing scope_guard tests (t4_pause_via_go_and_ask, retry_refuses_paused_line, t3_pause_from_reading_forecast and the rest) still pass, so real rejections and cost overruns still pause lines. Run: cargo test.
cargo fmt --check and cargo clippy --all-targets -- -D warnings are clean.
CHANGELOG.md has an entry naming the scope guard change: environment-ended attempts are not counted, and resumed retries are forecast on remaining stages.
Each new constant or rule in scope_guard.rs has a doc comment stating its basis or marking it an estimate, and the diff touches only crates/athena-core/src/scope_guard.rs and CHANGELOG.md.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
