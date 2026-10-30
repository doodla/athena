# Job 20261030-214204-x2fz: Add one shared budget-reserve preflight covering all planning rounds

## Request (verbatim)

> try again on 20261030-151818-2c8g

## Reading (accepted by the user)

Retry of job 20261030-151818-2c8g, which failed because its plan's reserve omitted the critic cap and later planning rounds. Add one function in reading.rs computing reader + max_rounds×(planner+critic) + producer + verifier from config, used at admission, retry, drain dispatch and before each planning round, so a job is held with its shortfall before spend.

Assumptions:
- Goal: stop jobs that cannot afford the whole planning-plus-producer-verifier path before money is spent; audience is the person running Athena and the verifier.
- Job 20261030-151818-2c8g failed in plan critique round 1 (no candidate, nothing kept). Its major finding (reserve omitted critic and later rounds) is fixed by counting reader + max_rounds×(planner+critic) + producer + verifier; its minor findings are fixed by making the dispatch test discriminate, testing the real preflight on a real job, recomputing the test numbers, and listing lifecycle.rs and job_plan.rs as deliverables for config-only edits.
- Code survey: read crates/athena-core/src/reading.rs (check_budget at ~287, first_reading ~841, retry ~909), crates/athena-core/src/run/planning.rs (round loop at 41, critic check at ~283), crates/athena-core/src/run/mod.rs (dispatch ~685), crates/athena-cli/src/drain.rs, and tests in crates/athena-cli/tests/lifecycle.rs and job_plan.rs. Admission runs first_reading, then dispatch, then the planning rounds. No new record or data file is added, so check_record_file and AnyRecord are not touched. The property is measured by comparing the preflight verdict with the per-round check at the same budget, with a fake driver counting invocations.
- The reserve reads caps from config (reader, planner_usd, critic_usd, max_rounds, producer, verifier), so changing a cap in athena.toml changes it without a code change; athena.toml itself is not edited.
- Existing tests whose budgets sit near the new reserve get config-only edits to their budget values.

## Acceptance criteria

1. [content] CHANGELOG.md has an entry naming the root cause: budget checked only per invocation after reading and planning had spent, with a reserve that left out the critic and later rounds; it names jobs 20261028-082151-67fa, 20261030-033434-3v30 and 20261030-151818-2c8g.
2. [content] One reserve function in crates/athena-core/src/reading.rs sums reader, max_rounds×(planner+critic), producer and verifier from config; Grep shows it called from admission, retry, drain dispatch and the per-round check in run/planning.rs, with no duplicated arithmetic.
3. [behavior] Core test: with a fake driver and the recorded $8.00 budget against the shipped caps (reserve 10.00), admission refuses with a message naming the shortfall and the config line, and the driver records zero invocations. Run: cargo test -p athena-core.
4. [behavior] Core test: the preflight and the per-round critic check agree at $5.75, at the single-round total and at the full multi-round total; a retry whose recorded spend leaves less than the reserve is refused before planning, and a job with enough budget proceeds. Run: cargo test -p athena-core.
5. [behavior] Core test of the real preflight on a real stored job, with spend chosen so left ≥ producer+verifier but < the full reserve: base reaches the planner, the change records no invocation. Run: cargo test -p athena-core.
6. [behavior] Drain test through the real preflight path: a short job is skipped with its reason on one log line and the next item with enough budget still runs; it fails if the check is removed. Run: cargo test -p athena-cli.
7. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass, with edits to existing tests config-only.
8. [behavior] The change refuses more readings at admission and the higher refusal rate is intended: expected 10% of newly refused readings, since the shipped athena.toml budget of 12.00 covers the 10.00 reserve.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing athena.toml [budget] values, defaults.toml or workflows
- Interrupted-job takeover, disk cleanup and verifier changes from job 20261028-082151-67fa (candidate dropped)
- Drain ordering and plan changes from job 20261030-033434-3v30 (candidate dropped)
- Partial-plan resume for the planner
- Changing producer, verifier or planner prompts
- Letting a retry raise a job's budget

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes in Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added; cargo test and clippy must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Single shared reserve function at the origin, not per-site patches.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests with a fake driver counting invocations.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Outcome is no job reaching planning without enough budget.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced in code at four sites.
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

CHANGELOG.md has an entry naming the root cause: budget checked only per invocation after reading and planning had spent, with a reserve that left out the critic and later rounds; it names jobs 20261028-082151-67fa, 20261030-033434-3v30 and 20261030-151818-2c8g.
One reserve function in crates/athena-core/src/reading.rs sums reader, max_rounds×(planner+critic), producer and verifier from config; Grep shows it called from admission, retry, drain dispatch and the per-round check in run/planning.rs, with no duplicated arithmetic.
Core test: with a fake driver and the recorded $8.00 budget against the shipped caps (reserve 10.00), admission refuses with a message naming the shortfall and the config line, and the driver records zero invocations. Run: cargo test -p athena-core.
Core test: the preflight and the per-round critic check agree at $5.75, at the single-round total and at the full multi-round total; a retry whose recorded spend leaves less than the reserve is refused before planning, and a job with enough budget proceeds. Run: cargo test -p athena-core.
Core test of the real preflight on a real stored job, with spend chosen so left ≥ producer+verifier but < the full reserve: base reaches the planner, the change records no invocation. Run: cargo test -p athena-core.
Drain test through the real preflight path: a short job is skipped with its reason on one log line and the next item with enough budget still runs; it fails if the check is removed. Run: cargo test -p athena-cli.
cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass, with edits to existing tests config-only.
The change refuses more readings at admission and the higher refusal rate is intended: expected 10% of newly refused readings, since the shipped athena.toml budget of 12.00 covers the 10.00 reserve.

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
