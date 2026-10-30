# Job 20261030-151810-0n3b: Raise job_usd budget from 8.00 to 12.00 with a covering test

## Request (verbatim)

> Priority: urgent
> Raise [budget] job_usd in athena.toml from 8.00 to 12.00. Hard jobs read, plan and critique before the producer runs; on 2026-10-30 the drain-order fix spent $2.50 on reading and planning, then stopped because $5.50 was left and the planner, producer and verifier caps need $5.75. Any hard job whose reading or planning takes a second round hits this, so it fails before writing code.
> Acceptance:
> - athena.toml has job_usd = 12.00 and nothing else in [budget] changes.
> - A test that loads the repository's athena.toml shows job_usd is 12.00 and covers the per-stage caps (reader, planner, producer, verifier) summed with two reading rounds and two planning rounds.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: athena.toml, CHANGELOG.md

## Reading (accepted by the user)

Change [budget] job_usd in athena.toml from 8.00 to 12.00 so hard jobs with a second reading or planning round can still run the producer and verifier. Add a test over the repository's athena.toml that pins 12.00 and checks the summed per-stage caps fit, plus a CHANGELOG.md entry. The audience is whoever runs hard jobs and reviews the budget.

Assumptions:
- Goal: hard jobs must not stop for budget after two reading and two planning rounds; audience is the maintainer who runs hard jobs.
- The only value change in athena.toml is job_usd under [budget]; reader_usd 0.50, producer_usd 3.00, verifier_usd 2.00 and [job_plan] planner_usd 0.75, critic_usd 0.75, max_rounds 3 stay as they are (read in athena.toml).
- Summed caps with two reading and two planning rounds: 2*0.50 + 2*(0.75+0.75) + 3.00 + 2.00 = 9.00, which is within 12.00 and above the old 8.00 only once the extra rounds count; the test computes this from the parsed config rather than hard-coding it.
- Several tests include athena.toml with include_str!; run_pin_tests.rs line 870 does .replace("job_usd = 8.00", ...) and would silently stop tightening the budget, so it must be updated to "job_usd = 12.00" to keep that test meaningful.
- The new test goes in an existing test module that already parses the repository's athena.toml (for example crates/athena-data/src/config.rs with REPO_CONFIG); the producer picks the field names from the Config struct.
- Code survey: read crates/athena-core/src/run_pin_tests.rs (config() at line 79 loads athena.toml via include_str!, line 870 replaces job_usd), and grepped crates/athena-data/src/config.rs (REPO_CONFIG at line 706) and other crates for include_str! of athena.toml. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. The change adds no refusal rule; it loosens a limit, so no newly-refused-readings percentage applies.

## Acceptance criteria

1. [content] athena.toml [budget] has job_usd = 12.00 and reader_usd = 0.50, producer_usd = 3.00, verifier_usd = 2.00 are unchanged; git diff of athena.toml shows only that one line changed.
2. [content] A test in crates/ parses the repository's athena.toml (include_str!) and asserts job_usd == 12.00, and asserts that 2 x reader cap + 2 x (planner cap + critic cap) + producer cap + verifier cap is at most job_usd, with the caps read from the parsed config.
3. [content] crates/athena-core/src/run_pin_tests.rs replaces "job_usd = 12.00" instead of "job_usd = 8.00" in the tight-budget test, so that test still tightens the budget.
4. [content] CHANGELOG.md has a new entry stating job_usd rose from 8.00 to 12.00 and why (second reading or planning round on a hard job exhausted the budget).
5. [behavior] Run cargo test: the new budget test and the whole suite pass, including when_the_budget_runs_out_before_the_rerun_the_reason_reports_the_budget.
6. [behavior] Run cargo clippy --all-targets -- -D warnings and cargo fmt --check: both are clean.
7. [behavior] The new test fails if athena.toml's job_usd is set back to 8.00 (verifier checks by reading the assertion against the diff).

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)

## Out of scope

- Changing reader_usd, producer_usd, verifier_usd or any [job_plan] value.
- Changing how budgets are enforced or how rounds are counted in code.
- Editing records/ or any other config file.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus cargo test and clippy -D warnings criteria.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the budget value and that the summed caps fit, not just that the file changed.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces the budget relation so it cannot drift silently.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused config change with its test and changelog.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The test is written in Rust.
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

21 more qualifying classes are left out.

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

athena.toml [budget] has job_usd = 12.00 and reader_usd = 0.50, producer_usd = 3.00, verifier_usd = 2.00 are unchanged; git diff of athena.toml shows only that one line changed.
A test in crates/ parses the repository's athena.toml (include_str!) and asserts job_usd == 12.00, and asserts that 2 x reader cap + 2 x (planner cap + critic cap) + producer cap + verifier cap is at most job_usd, with the caps read from the parsed config.
crates/athena-core/src/run_pin_tests.rs replaces "job_usd = 12.00" instead of "job_usd = 8.00" in the tight-budget test, so that test still tightens the budget.
CHANGELOG.md has a new entry stating job_usd rose from 8.00 to 12.00 and why (second reading or planning round on a hard job exhausted the budget).
Run cargo test: the new budget test and the whole suite pass, including when_the_budget_runs_out_before_the_rerun_the_reason_reports_the_budget.
Run cargo clippy --all-targets -- -D warnings and cargo fmt --check: both are clean.
The new test fails if athena.toml's job_usd is set back to 8.00 (verifier checks by reading the assertion against the diff).

## Planned files left unchanged

Every file in the plan and in the expected deliverables must be changed, or you must say why it needs no change. Before you report, compare your edits with those lists. For each listed file you did not change, either change it now or add an entry to `unchanged_planned_files` in your report with its path and the reason. A listed file that is neither changed nor explained is sent back once, then flagged to the verifier as a gap.

## Report

End with the producer report as your structured output.
