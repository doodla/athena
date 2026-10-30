# Job 20261030-214204-x2fz: verify "Add one shared budget-reserve preflight covering all planning rounds"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the added Unreleased entry in the diff. Run: `git diff -- CHANGELOG.md`. Observes: The added entry names the root cause: budget checked only per invocation after reading and planning had spent, with a reserve that left out the critic and later rounds. It cites 20261028-082151-67fa, 20261030-033434-3v30 and 20261030-151818-2c8g. It states that the higher refusal at admission is intended because the shipped 12.00 covers the 10.00 reserve.
- Criterion 2: Static check after a build. Grep for the single arithmetic and its callers. Run: `cargo build -p athena-core -p athena-cli`. Observes: After the build, Grep `fn budget_reserve` under crates/ finds one definition, in reading.rs. Grep `\.preflight\(` finds calls in reading.rs (first_reading, retry), run/mod.rs (dispatch Dispatch and Run stages) and run/planning.rs (Planner and Critic stages). drain.rs only downcasts `BudgetShort`. Grep for `planner_usd +`, `critic_usd +`, `producer_usd +` in run/planning.rs and run/mod.rs finds no sums.
- Criterion 3: Core test in run_pin_tests.rs. Entry point: `Athena::ask`, the production admission path through `first_reading`. Guards: `[job_plan] enabled = true` appended to `config(true)`, which gives the shipped caps (reader 0.50, planner and critic 0.75, max_rounds 3, producer 3.00, verifier 2.00) and a reserve of 10.00. `job_usd = 12.00` is replaced by `8.00`. Seam: the existing `athena_with(..., tweak)` hook (run_pin_tests.rs:153) is called for every invocation including the reader. The test passes a tweak that increments a shared counter and leaves the turn alone; the counter is the invocation count and holds no logic under test. The seam exists today. Run: `cargo test -p athena-core admission_refuses_a_job_short_of_the_whole_planning_reserve`. Observes: `ask` returns Err whose text contains the shortfall $2.00, the reserve $10.00 and `[budget] job_usd` in athena.toml. The counter is 0 and `git.head()` is unchanged.
  Fails without the behavior: Without the admission preflight in first_reading, the old per-invocation check passes at 8.00, the reader runs (counter 1) and the Err assertion fails.
- Criterion 4: Three core tests in `cargo test -p athena-core`. (a) Real-round table in run/tests.rs. Entry points: `Athena::ask` then `Athena::go` on `repo_with_planning(max_rounds)` with `job_usd` 12.00, which reaches the production critic check in run/planning.rs. Seam: the existing `planning_script` and `athena_with`, wrapped by a closure that only sets `turn.cost_usd` for the Planner role; the fake driver does not enforce the cap, so a planner turn can cost more than $0.75. The reader and critic cost 0.01. Rows (max_rounds, planner cost, left at the round-1 critic, expected): (3, 6.24, 5.75, critic not called); (3, 3.25, 8.74, not called); (3, 3.24, 8.75 = the full multi-round Critic{1} reserve, called); (1, 6.25, 5.74, not called); (1, 6.24, 5.75 = the single-round reserve, called). `go` returns Finished(Failed) naming the shortfall, and the plan record stands for the stopped rows. (b) Unit table in reading.rs `mod tests` on the shipped-config `JobConfig`: Admission − reader − planner equals Critic{1} for max_rounds 1..4; Admission is Err at job_usd 5.75 for 3 rounds and Ok at 7.00 for 1 round. This is arithmetic only and is not the proof that planning.rs calls the shared check. (c) Retry test in run/tests.rs. A planned job fails in planning: `revising_critique`, 3 rounds, the pattern of the test at run/tests.rs:651. A shared reader-cost cell in the cost wrapper is then set to 4.0. `Athena::retry(job)` returns Ok: its admission starts at 0, which shows that no spend from the old job is carried over. `go(new job)` returns Err naming the shortfall (left 8.00 against the 9.50 dispatch reserve), the Planner invocation count does not rise and the head is unchanged. The original job in the same test is the funded case: it ran its planner and critic. Run: `cargo test -p athena-core preflight_and_round_checks_agree && cargo test -p athena-core round_one_critic_check_follows_the_shared_reserve && cargo test -p athena-core a_retry_is_judged_on_its_own_spend_and_stopped_before_planning`. Observes: (a) The critic call count is 0 in the 5.75, 8.74 and 5.74 rows and 1 in the 8.75 and 5.75 single-round rows; the refused rows' reasons contain the shortfall. (b) The identity and the admission verdicts hold. (c) The retry is admitted, `go` is refused with `BudgetShort` and no planner is called.
  Fails without the behavior: (a) On the base commit planning.rs checks `critic_usd + producer + verifier` = 5.75, so the critic is called in the (3, 6.24) and (3, 3.25) rows and the test fails. (c) Without the Dispatch preflight, the old 5.00 check passes at 8.00 and the planner is called. Reverting to a carry-over rule would make the `retry` call Err.
- Criterion 5: Core test in run/tests.rs on a real stored job. Entry points: `Athena::ask` then `Athena::go` on `repo_with_planning(3)` (mode all, the planning capability), `job_usd` 12.00, so admission passes at 10.00. A wrapper over `planning_script` sets the Reader turn's `cost_usd` to 4.00, so the reader spends 4.00 and left is 8.00. That is at least producer+verifier (5.00) and below the dispatch reserve (3×1.50 + 5.00 = 9.50). The `Calls` vector from `planning_script` records every invocation. The seam exists today. Run: `cargo test -p athena-core dispatch_holds_a_job_that_cannot_afford_every_planning_round`. Observes: `ask` records [Reader]. `go` returns Err whose text contains the shortfall $1.50 and `[budget] job_usd`. `calls` has no Planner or Critic, and `athena.head()` is the same before and after `go`, so no prediction commit was made.
  Fails without the behavior: Without the Dispatch preflight, the old producer+verifier check (5.00) passes at 8.00, `go` calls the planner, `calls` contains Role::Planner and the test fails.
- Criterion 6: Drain test in job_plan.rs (athena-cli) through the real drain. Entry points: `athena_cli::drain::drain` over `RealSteps { athena, ui, page, lander }`, which calls `Athena::go` (queued re-read) and `Athena::run`. Setup: `Fixture::with_config(with_job_usd(test_config(), "6.00"))`, unplanned reserve 5.50. A wrapper over `Scenario::default().script()` turns item A's first reader call into `turn.structured = None` with `cost_usd = 0.60`, and counts reader calls by prompt (the pattern of `reader_fails_for_first_request`, tests/drain.rs:2109). `athena.queue(A)` records A with no reading and $0.60 spent; `athena.queue(B)` reads normally for $0.01. The drain then re-reads A through `go` → `first_reading(prior = Some)` with before 0.60, so left is 5.40 against 5.50. `StubPage` (tests/drain.rs:1325), a recording Lander and a small `real_steps` constructor are copied locally into job_plan.rs, because tests/common does not hold them; the producer reads their bodies in tests/drain.rs. `Read.held`, the `BudgetShort` mapping and the Held handling for reads are added in the steps. The rest of the seam exists today. Run: `cargo test -p athena-cli --test job_plan a_short_job_is_skipped_with_its_reason_and_the_next_item_runs`. Observes: `report.notes` and the `Lines` log each hold exactly one entry of the form `item <A> is left queued: …`, and it carries the shortfall `$0.10` and `[budget] job_usd`. A's reader count is 1 (the queue-time read; the drain pass adds none). B is read, run and landed through the recording Lander. `report.stop` is not `Stop::RepeatedFailure`.
  Fails without the behavior: With the first_reading preflight removed, A's reader runs again (count 2), the A note is missing and the assertions fail. If `BudgetShort` were not mapped to held, A would be a Failed item, not a left-queued one, and the 'left queued' assertion fails.
- Criterion 7: Run the three Athena checks. Existing tests get budget and cap values only, with no assertion, script or control-flow edits: lifecycle.rs `a_job_over_budget_is_refused_before_its_next_invocation` (job_usd 4.00 → 5.60, above the 5.50 unplanned reserve); job_plan.rs `a_cap_between_the_producer_alone_and_planning_plus_the_producer…` (add `reader_usd = 0.01` and `max_rounds = 1` beside job_usd 5.025, reserve 0.01 + 0.01 + 5.00 = 5.02); run_pin_tests.rs `when_the_budget_runs_out_before_the_rerun…` (`reader_usd = 0.10`, so reserve 0.10 + 0.50 + 2.00 = 2.60); run/tests.rs `repo_with_planning(5)` → `repo_with_planning(4)` (reserve 11.50, still above the two mechanical passes the test relies on). Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three pass. `git diff` on the four existing tests shows only budget and cap numbers changed.
  Fails without the behavior: Reverting any of the four edits makes the test refuse at admission and fail at its first assertion. Removing a call site or breaking the arithmetic makes the new tests for criteria 3-6 fail in the same run.
- Criterion 8: Unit test in reading.rs `mod tests` on the `JobConfig` built from the shipped athena.toml, plus the criterion 3 test. Entry points: `budget_reserve(Admission)` and `Athena::preflight`. Precondition: the shipped `[budget]` is 0.50/3.00/2.00/12.00 and `[job_plan]` is 0.75/0.75/max_rounds 3. The 10% figure is a forecast and is not tested; the test shows only that the shipped 12.00 covers the 10.00 reserve and that 8.00 is refused. Run: `cargo test -p athena-core shipped_budget_covers_the_reserve`. Observes: `budget_reserve(Admission)` equals 10.00 within 1e-9. `preflight(Admission, 0)` is Ok at job_usd 12.00 and Err at 8.00 with shortfall $2.00.
  Fails without the behavior: A reserve that left out the critic and later rounds would be 5.50 or 7.00, not 10.00, and both the equality and the 8.00 refusal fail.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 10% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 748e87c99431 → candidate 1864315b1c7c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  17 +++
 crates/athena-cli/src/drain.rs          |  75 ++++++++++--
 crates/athena-cli/tests/drain.rs        |   2 +
 crates/athena-cli/tests/job_plan.rs     | 113 +++++++++++++++++-
 crates/athena-cli/tests/lifecycle.rs    |   4 +-
 crates/athena-core/src/reading.rs       | 202 ++++++++++++++++++++++++++++++++
 crates/athena-core/src/run/mod.rs       |  35 ++----
 crates/athena-core/src/run/planning.rs  |  16 +--
 crates/athena-core/src/run/tests.rs     | 189 +++++++++++++++++++++++++++++-
 crates/athena-core/src/run_pin_tests.rs |  56 ++++++++-
 10 files changed, 654 insertions(+), 55 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1753 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass, with edits to existing tests config-only.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
