# Job 20261030-151810-0n3b: verify "Raise job_usd budget from 8.00 to 12.00 with a covering test"

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

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff. Entry point: the athena.toml file itself. Run: `git diff -- athena.toml`. Observes: Exactly one changed line in [budget]: `-job_usd = 8.00` and `+job_usd = 12.00`. reader_usd = 0.50, producer_usd = 3.00 and verifier_usd = 2.00 are not in the diff.
- Criterion 2: Read the new test in crates/athena-data/src/config.rs and run it. It calls Config::parse(REPO_CONFIG). That constant exists today at config.rs line 706, via include_str!("../../../athena.toml"), so no seam is added. Run: `cargo test --locked -p athena-data the_repository_budget`. Observes: The test passes. In the source it asserts `budget.job_usd == 12.0` and `2*reader_usd + 2*(planner_usd + critic_usd) + producer_usd + verifier_usd <= job_usd`, with every cap read from the parsed config (job_plan unwrapped).
  Fails without the behavior: Not a behavior criterion. The test is added in step 2 and run here.
- Criterion 3: Read the diff. Run: `git diff -- crates/athena-core/src/run_pin_tests.rs`. Observes: Line 870 changes to `.replace("job_usd = 12.00", "job_usd = 2.60")`. Nothing else in the file changes.
- Criterion 4: Read the diff. Run: `git diff -- CHANGELOG.md`. Observes: A new bullet under [Unreleased] (a Changed heading) saying job_usd went from 8.00 to 12.00, and that a second reading or planning round on a hard job exhausted the budget before the producer and verifier could run.
- Criterion 5: Run the whole suite. Production entry points: Config::parse (athena-data) and the run in run_pin_tests.rs through its existing config()/run_with helpers, which exist today. Run: `cargo test --locked`. Observes: All tests pass, including the new budget test, when_the_budget_runs_out_before_the_rerun_the_reason_reports_the_budget and the_default_plan_budget_affords_a_read_where_drain_would_need_a_read_and_a_run.
  Fails without the behavior: If athena.toml were left at 8.00 the new test fails on the job_usd == 12.0 assertion. If run_pin_tests.rs kept the 8.00 pattern, when_the_budget_runs_out_before_the_rerun_the_reason_reports_the_budget would run with job_usd = 12.00 and runs.len() would be 2, so its assert_eq!(runs.len(), 1) fails. If drain.rs line 2855 kept 8.0, the_default_plan_budget_affords_a_read_where_drain_would_need_a_read_and_a_run fails.
- Criterion 6: Run clippy, then `cargo fmt --all --check`, both as in the checks Athena will run. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Both exit 0 with no warnings and no formatting diff.
  Fails without the behavior: Not applicable. This is a lint check, so any warning in the new test code fails it.
- Criterion 7: Read the test's equality assertion in config.rs against the athena.toml diff. It is an exact equality on the parsed job_usd, not a lower bound. Run: `cargo test --locked -p athena-data the_repository_budget`. Observes: The assertion compares the parsed job_usd to 12.0 with assert_eq!. It would fail on the old value and on any other value.
  Fails without the behavior: With athena.toml at job_usd = 8.00, the `job_usd == 12.0` assertion fails (8.0 != 12.0). The verifier confirms this by reading the assertion against the diff, and does not edit athena.toml.

Planned files:

- athena.toml (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 70cdad6f739a → candidate 0b6d38e63c1b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  5 +++++
 athena.toml                             |  2 +-
 crates/athena-cli/tests/drain.rs        |  2 +-
 crates/athena-core/src/run_pin_tests.rs |  2 +-
 crates/athena-data/src/config.rs        | 17 +++++++++++++++++
 5 files changed, 25 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1742 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo clippy --all-targets -- -D warnings and cargo fmt --check: both are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
