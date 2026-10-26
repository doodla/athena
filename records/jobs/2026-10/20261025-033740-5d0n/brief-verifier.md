# Job 20261025-033740-5d0n: verify "Add job-planning keep-rule evaluation and breach line to athena status"

## Request (verbatim)

> Priority: high
> Have athena status evaluate the job-planning keep rule (docs/design/job-planning.md section 9, decision D110) and flag it: the landed share of planned jobs is 28 of 51 (55%), below the 64% baseline the rule holds planning to through the first 30 planned jobs, and nothing reported it. Compute the landed share of planned dispatched jobs, cost per landed job and the first-round accept rate, compare them with the rule at its thresholds (30 and 100 planned jobs), and show a breach in status with the numbers.
> Acceptance:
> - A status test with fixture records shows the measures and a breach line when the share is below the baseline.
> - athena status --json carries the measures.
> Files: crates/athena-core/src/status.rs, crates/athena-cli/src/display.rs, CHANGELOG.md

## Reading (accepted by the user)

Add a pure keep-rule evaluation in status.rs computing, over planned dispatched jobs, the landed share (stopped-in-planning counts as not landed), cost per landed job and the first-round accept rate. It compares them with job-planning.md §9 at the 30 and 100 planned-job thresholds, adds the result to StatusView (so `--json` carries it), and display.rs prints the measures and a breach line. The audience is the operator reading status; the real data case is 28/51 = 55% against the 64% baseline.

Assumptions:
- The 28 of 51 figure is from the request; the implementation computes it from records rather than hard-coding it, and fixtures reproduce that shape.
- Planned jobs means request jobs with at least one job-plan record and a dispatch; a job stopped in planning counts as dispatched and not landed.
- Cost per landed job is total cost of planned jobs divided by landed ones, compared with the $2.50 ceiling at 100 planned jobs; the first-round accept rate reuses the existing FirstRoundCounts logic.
- Through 30 planned jobs a share below 0.64 is a breach; at 100 the rule's keep / turn off / inconclusive outcomes apply.
- The rule only reports; it does not turn planning off in athena.toml.

## Acceptance criteria

1. [behavior] A status.rs unit test builds fixture records with 51 planned dispatched jobs, 28 landed, and asserts the measures (landed 28, planned 51, share about 0.549, cost per landed job, first-round accept rate) and a breach verdict because the share is below 0.64. Run: cargo test -p athena-core.
2. [behavior] Fixture tests cover the rule's branches: at most 30 planned jobs with share at or above 0.64 reads keep; below 0.64 reads breach; at 100 or more planned jobs, share at least 0.72 with cost per landed job at most $2.50 reads keep, share at or below 0.64 reads turn off, otherwise inconclusive; zero landed gives no division by zero. Run: cargo test -p athena-core.
3. [behavior] A display.rs test shows the rendered status text contains the measures with their numbers (28 of 51, 55%, baseline 64%, cost per landed job, first-round accept rate) and a breach line when breached, and no breach line when kept. Run: cargo test -p athena-cli.
4. [behavior] The StatusView serialization (what `athena status --json` prints) contains a keep-rule object with the measures and verdict; a test asserts the JSON keys and values. Run: cargo test.
5. [content] The denominator counts planned dispatched jobs, a job stopped in planning counts as not landed, and cost per landed job divides all cost of those jobs by landed jobs, as section 9 defines it; the thresholds (30, 100, 64%, 72%, $2.50) are named constants citing job-planning.md section 9 and D110.
6. [content] CHANGELOG.md has an entry for the keep-rule evaluation in status.
7. [behavior] cargo test, cargo clippy --all-targets and cargo fmt --check all pass.

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- Automatically disabling planning in athena.toml
- Changing the keep rule's thresholds or job-planning.md
- Hestia checks that schedule the evaluation

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture tests in both crates; cargo test and clippy criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the measures and breach line appear in rendered status and JSON.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is a breach line with numbers shown when the share is below baseline.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single focused change in the three named files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria runnable by the verifier with cargo test.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in the planning_cost_tests module of status.rs. It builds 51 planned jobs (28 Landed, 23 Failed or stopped in planning) with known costs and critiques, then calls keep_rule. Run: `cargo test -p athena-core keep_rule`. Observes: planned 51, landed 28, share about 0.549, cost per landed equal to the total cost of the 51 jobs divided by 28, the expected first-round accept rate, and verdict Breach.
  Fails without the behavior: With no keep-rule evaluation the function does not exist. If stopped jobs were dropped from the denominator, planned would not be 51 and the share would not be 0.549.
- Criterion 2: Table-style fixture tests, built with a small generator of N jobs with L landed. They cover 30 planned at 64% or more (keep), 30 planned below 64% (breach), 100 planned with share 0.72 or more and cost at most $2.50 (keep), 100 planned with share 0.72 or more and cost above $2.50 (inconclusive), 100 planned with share at most 0.64 (turn_off), 100 planned with share between 0.64 and 0.72 (inconclusive), and 0 landed (no panic, cost per landed None). Run: `cargo test -p athena-core keep_rule`. Observes: The expected verdict for each branch, and no NaN or infinity in the zero-landed result.
  Fails without the behavior: A wrong threshold comparison, such as < instead of <= at 0.64, or an unguarded division would fail one of these cases.
- Criterion 3: Unit test in display.rs tests. It renders keep_rule_lines for a breach KeepRule (28 of 51, 0.549, baseline 0.64) and for a keep KeepRule. Run: `cargo test -p athena-cli keep_rule`. Observes: The breach text contains '28 of 51', '55%', '64%', 'cost per landed job', 'first-round accept rate' and a line starting 'BREACH'. The keep text contains the measures and no 'BREACH' line.
  Fails without the behavior: With no rendering the measures and the breach line are absent from the status text. If the line were always printed, the keep case would fail.
- Criterion 4: A status.rs test serializes the KeepRule from the 51-job fixture with serde_json::to_value. The StatusView field is `keep_rule`, an Option<KeepRule> that serializes whenever there is a planned job. Run: `cargo test`. Observes: The keys planned, landed, share, cost_per_landed_usd, first_round_accept_rate, baseline_share and verdict, with verdict equal to "breach" and planned equal to 51.
  Fails without the behavior: If the field were not serialized, or the keys differed, the key and value assertions would fail.
- Criterion 5: The tests of criteria 1 and 2 include a stopped-in-planning job in the denominator. A review of the constants shows their doc comments cite job-planning.md §9 and D110. Run: `cargo test -p athena-core keep_rule`. Observes: A job stopped in planning counts as planned and not landed, and cost per landed equals the total cost of all planned jobs divided by landed.
  Fails without the behavior: Excluding stopped jobs or dividing only landed jobs' cost changes the asserted numbers.
- Criterion 6: Inspect the diff. Run: `git diff -- CHANGELOG.md`. Observes: An entry under Unreleased > Added describing the keep-rule evaluation, the breach line and the `keep_rule` key in `athena status --json`.
- Criterion 7: Run the three Athena checks (each as its own cargo command). Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: A format, lint or test failure would exit non-zero.

Planned files:

- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)

## The change

Base 5f8ec7635fb5 → candidate 9599052ca8f0. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   8 ++
 crates/athena-cli/src/display.rs | 109 ++++++++++++++-
 crates/athena-core/src/status.rs | 280 ++++++++++++++++++++++++++++++++++++++-
 3 files changed, 393 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1359 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test, cargo clippy --all-targets and cargo fmt --check all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
