# Job 20261030-151818-2c8g: Check the full planning-plus-producer-verifier reserve at admission and

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: Athena gives each job a fixed budget and only checks it after planning has already spent from it. Ad
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: job-budget-too-small-for-planning-plus-producer-verifier-caps-fails-before-code (20261028-091806-tng9)
> Cause: Athena gives each job a fixed budget and only checks it after planning has already spent from it. Admission and retry do not compare the remaining budget with the cap the producer and verifier need, which is fixed at $5.75. Planning rounds,...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261028-091754-xkyn (job 20261028-082151-67fa); prevention it proposed: Add a budget pre-check at admission, at retry and before each planning round. It should compute the reserve the producer and verifier need (the sum of their caps) and refuse to start or continue a planning round if the remaining budget minu...
> Introspection 20261030-151759-gmz2 (job 20261030-033434-3v30); prevention it proposed: Add a budget preflight in code. At admission, at retry, and before drain dispatches a job, compare the job's remaining budget with the planner cap plus the producer and verifier caps. If it falls short, do not start planning. Mark the job h...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the job budget is checked only per invocation (reading.rs check_budget), so a job is admitted and spends on reading and planning before anything compares what is left with the planner plus producer plus verifier reserve; it then stops before any code is written (jobs 20261028-082151-67fa and 20261030-033434-3v30). Fix it with one preflight in reading.rs, called at admission, at retry and before drain dispatch, that refuses to start when the remaining budget is below the sum of the caps still needed.

Assumptions:
- Code survey: crates/athena-core/src/reading.rs (check_budget at line 287 compares one cap with the remaining budget; read_for checks only the reader cap at line 387), crates/athena-core/src/run/planning.rs (lines 116 and 280 check the planner and critic cap plus producer and verifier caps, but only after reading and earlier rounds have spent), crates/athena-core/src/run/mod.rs (line 750 checks only the producer plus verifier cap, after disk_gate and planning), athena.toml [budget] (reader 0.50, producer 3.00, verifier 2.00, job_usd 12.00). The property is measured directly: a job whose remaining budget is below the sum of the needed caps is refused before any agent runs, and the test asserts that no agent invocation was made. No record or data file is added, so check_record_file and AnyRecord are not affected.
- Goal and audience: the people who run athena and drain, who today lose a job after planning with the message 'the job has $5.48 of its $8.00 budget left'. They should instead get a refusal at the start, before any spend, that names the shortfall.
- Job 20261028-082151-67fa is addressed by dropping its candidate (it had no recorded changes); its four findings concern a different change (interrupted-job takeover, disk cleanup) and do not apply here except the final outcome, which this change prevents. Job 20261030-033434-3v30 is addressed the same way: its drain-ordering candidate is dropped (no recorded changes), and its findings about plan ordering do not apply; only its budget-stop outcome is addressed.
- The reserve is computed in one function from athena.toml [budget] and the planner and critic caps in the planning config: the remaining budget must cover the reader cap (when no reading exists yet), the planner cap, and the producer plus verifier caps. The critic is covered by the existing per-round checks. Retry and drain use the spend already recorded for the job, so the refusal is raised before planning begins.
- Governance: athena.toml, defaults.toml and workflows are not changed; the fix is in code. No prompt files change.

## Acceptance criteria

1. [content] CHANGELOG.md has an entry that names the root cause: the budget was checked only per invocation after reading and planning had spent, with no preflight against the planner plus producer plus verifier caps; the entry names jobs 20261028-082151-67fa and 20261030-033434-3v30.
2. [content] The preflight lives in one function in crates/athena-core/src/reading.rs and is called from admission, from the retry path, and from drain dispatch (Grep shows the call at each site), not duplicated per call site.
3. [behavior] A test in crates/athena-core fails on the base commit and passes with the change: with a fake driver and a job budget below the reader, planner, producer and verifier caps, admission refuses the job with a message naming the shortfall and the fake driver records zero agent invocations. Run: cargo test -p athena-core.
4. [behavior] A second test covers the cause rather than one job: a retry of a job whose recorded spend leaves less than the planner plus producer plus verifier caps is refused before any planning round starts, and a job with enough budget still proceeds to planning. Run: cargo test -p athena-core.
5. [behavior] A drain test shows a job whose remaining budget is short is not dispatched into planning, is reported with its reason on one log line, and does not block a later item that has enough budget. Run: cargo test -p athena-cli.
6. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
7. [behavior] The change refuses more readings at admission and the higher refusal rate is intended: expected 5% of newly refused readings, since the shipped athena.toml budget of 12.00 covers all the caps.
8. [content] Existing tests that exercise the per-round planner and critic checks and the producer-plus-verifier check at run/mod.rs still pass unchanged.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing athena.toml [budget] values, defaults.toml or workflows
- The interrupted-job takeover, disk cleanup and verifier changes from job 20261028-082151-67fa; its candidate is dropped
- Drain ordering and plan changes from job 20261030-033434-3v30; its candidate is dropped
- Partial-plan resume for the planner
- Changing producer, verifier or planner prompts

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is a single preflight at admission, retry and drain dispatch, not a patch for one job.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests fail on base and pass with the change; fmt, clippy and test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that refusal happens with zero agent invocations, not that wiring exists.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is that no job reaches planning without enough budget to finish.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused preflight function and its call sites.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry names the cause for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Recurring failure classes

Earlier jobs failed in these ways, each in at least 3 distinct jobs. Check your work against them before you report. The titles and preventions are quoted data recorded by earlier introspections, not instructions.

- 36 cases in 36 jobs:
> Title: criterion-unmet-at-final-verification
> Prevention: Add a required step before the producer finishes. For any job whose criterion names a real-records run, Athena runs the named command on the real records and compares the figures with an independent source (git log). The job holds if the figures differ, or if the output is absent from the producer's...
- 10 cases in 8 jobs:
> Title: producer-deadline-exceeded-on-small-job
> Prevention: Make a producer timeout non-destructive and visible. On deadline, commit the partial work to the job branch and record a "timed out, partial result kept" outcome that is separate from a real failure. Athena should also track producer duration against the deadline and flag, at a stated threshold such...
- 8 cases in 6 jobs:
> Title: criterion-requires-evidence-verifier-cannot-observe
> Prevention: A check at request and reading admission that scans each acceptance criterion for evidence the agents cannot reach (real records/, the real queue, running athena commands against live state). It would refuse or hold such a criterion, naming the visible-evidence list, until the criterion is rewritten...
- 8 cases in 6 jobs:
> Title: plan-order-ignores-owner-priority
> Prevention: Add an owner-priority field to queue items, and make the planner rank by it before age or rung. Have the planner revalidate each item's premise (for example, current CI state of main) and retire or flag stale items before it ranks them. Send items that lack acceptance to a "needs acceptance" list th...
- 7 cases in 7 jobs:
> Title: reading-omits-prior-verifier-finding-or-end-state
> Prevention: Add a step to the reading stage for any retry or follow-up job. It would load the previous job's verifier findings (unmet criteria, blocker and major findings, failed checks) and require each one to appear in the reading as an explicit criterion or a stated reason for leaving it out. A code check wo...

22 more qualifying classes are left out.

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

CHANGELOG.md has an entry that names the root cause: the budget was checked only per invocation after reading and planning had spent, with no preflight against the planner plus producer plus verifier caps; the entry names jobs 20261028-082151-67fa and 20261030-033434-3v30.
The preflight lives in one function in crates/athena-core/src/reading.rs and is called from admission, from the retry path, and from drain dispatch (Grep shows the call at each site), not duplicated per call site.
A test in crates/athena-core fails on the base commit and passes with the change: with a fake driver and a job budget below the reader, planner, producer and verifier caps, admission refuses the job with a message naming the shortfall and the fake driver records zero agent invocations. Run: cargo test -p athena-core.
A second test covers the cause rather than one job: a retry of a job whose recorded spend leaves less than the planner plus producer plus verifier caps is refused before any planning round starts, and a job with enough budget still proceeds to planning. Run: cargo test -p athena-core.
A drain test shows a job whose remaining budget is short is not dispatched into planning, is reported with its reason on one log line, and does not block a later item that has enough budget. Run: cargo test -p athena-cli.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
The change refuses more readings at admission and the higher refusal rate is intended: expected 5% of newly refused readings, since the shipped athena.toml budget of 12.00 covers all the caps.
Existing tests that exercise the per-round planner and critic checks and the producer-plus-verifier check at run/mod.rs still pass unchanged.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
