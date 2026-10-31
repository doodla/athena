# Job 20261030-214156-d6y1: verify "Reject a job budget below the summed stage caps when config loads"

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: Athena gives each job a fixed budget and only checks it after planning has already spent from it. Ad
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: job-budget-too-small-for-planning-plus-producer-verifier-caps-fails-before-code (20261028-091806-tng9)
> Cause: Athena gives each job a fixed budget and only checks it after planning has already spent from it. Admission and retry do not compare the remaining budget with the cap the producer and verifier need, which is fixed at $5.75. Planning rounds,...
> Seen 6 time(s) in 5 jobs. The introspections that record it:
> Introspection 20261028-091754-xkyn (job 20261028-082151-67fa); prevention it proposed: Add a budget pre-check at admission, at retry and before each planning round. It should compute the reserve the producer and verifier need (the sum of their caps) and refuse to start or continue a planning round if the remaining budget minu...
> Introspection 20261030-151759-gmz2 (job 20261030-033434-3v30); prevention it proposed: Add a budget preflight in code. At admission, at retry, and before drain dispatches a job, compare the job's remaining budget with the planner cap plus the producer and verifier caps. If it falls short, do not start planning. Mark the job h...
> Introspection 20261030-153131-skzk (job 20261030-151827-48rm); prevention it proposed: Add one check, run at admission, when the queue picks a job, and before any retry or re-ask. It compares the job's remaining budget with the planning cost plus the producer and verifier caps. If the budget is short, it holds the job before ...
> Introspection 20261030-153335-xdtk (job 20261030-151810-0n3b); prevention it proposed: Add a code check that runs when a job fails at its budget cap, and again before a retry. It compares the remaining budget with the stage caps. If the cap is the cause, drain should hold that job with a clear reason, record the cap as the bl...
> Introspection 20261030-165437-3n07 (job 20261030-033434-3v30); prevention it proposed: Add a check at admission, before each retry and after each planning round. It compares the job's remaining budget with the producer cap plus the verifier cap. If the budget is short, it holds the job with a clear "budget below stage caps" r...
> Introspection 20261030-214133-cp86 (job 20261030-151818-2c8g); prevention it proposed: Define the reserve in one function that sums the stage caps from config, as planner + critic per round for max_rounds, plus producer and verifier. Use that same function in admission, retry, dispatch and the per-round check. Add a test that...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: `[budget] job_usd` was never required to cover the reader, every planning round, the producer and the verifier, so jobs 20261028-082151-67fa, 20261030-033434-3v30 and 20261030-151827-48rm stopped for budget before any code was written. The base tree already has `budget_reserve` and `preflight` and a 12.00 job budget; this change adds the missing origin check, which is config validation that refuses a `job_usd` below the reserve, and tests it.

Assumptions:
- Goal and audience: whoever runs Athena and judges the change wants no job to fail before code because its budget cannot cover planning plus producer and verifier caps. Failure-class jobs: 20261028-082151-67fa, 20261030-033434-3v30, 20261030-151827-48rm and the jobs of the other three introspections.
- Code survey: crates/athena-core/src/reading.rs defines ReserveStage, budget_reserve (the one sum of caps from config), BudgetShort and Athena::preflight, called at admission (ask, queued read, line ~976 and ~1043), at dispatch (run/mod.rs:775), before each planner and critic call (run/planning.rs:114, 273) and before the run (run/mod.rs:943); crates/athena-cli/src/drain.rs downcasts BudgetShort and holds the item. crates/athena-data/src/config.rs holds Config::parse and validation, whose tests (~745-753) check the shipped budget but nothing refuses a bad one at load. No record or data file is added, so check_record_file and AnyRecord are not touched. The property is measured directly by parsing a TOML with a small job_usd, not through a stand-in.
- The earlier three jobs ended with no candidate (none recorded, no verdict), so no earlier work is kept or dropped; their findings (interrupted takeover, drain order, replan) concern other changes and do not apply to this budget cause. Their stated need, a budget check before planning spends, is already met by preflight in the base tree; this job adds the config-load origin check.
- Config validation uses budget_reserve's formula through one shared function (no second copy of the sum); athena.toml is not edited because its 12.00 already covers the 10.00 admission reserve.
- Refusal rule: the shipped athena.toml and the config test fixtures must still parse; expected 0% of newly refused readings, since this refuses configs and not readings.

## Acceptance criteria

1. [content] CHANGELOG.md has an entry naming the root cause (job_usd never required to cover reader, planning rounds, producer and verifier caps) and the config-load check, and says earlier jobs 20261028-082151-67fa, 20261030-033434-3v30 and 20261030-151827-48rm are addressed by it.
2. [content] crates/athena-data/src/config.rs computes the required budget from the configured caps (reader, max_rounds of planner plus critic, producer, verifier) in one function that crates/athena-core/src/reading.rs budget_reserve reuses or agrees with; Grep shows no second hand-written sum of those caps.
3. [behavior] Run `cargo test -p athena-data`: a new test parses a config with job_usd below the reserve and fails with an error naming job_usd and the shortfall; the same test fails when the check is removed, and a config at exactly the reserve parses.
4. [behavior] Run `cargo test -p athena-core`: the shipped athena.toml still parses and shipped_budget_covers_the_reserve passes; a test changes a cap (planner_usd or max_rounds) and shows the required budget changes with no other code change.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.
6. [content] The change adds no edit to athena.toml, defaults.toml or .github/workflows, and does not touch crates/athena-cli/src/drain.rs ordering or run/mod.rs takeover code.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing athena.toml [budget] values (12.00 already covers the reserve)
- Drain ordering and passed-over logging from jobs 20261030-033434-3v30 and 20261030-151827-48rm (crates/athena-cli/src/drain.rs, plan.rs)
- The interrupted-job takeover and disk changes from job 20261028-082151-67fa (run/mod.rs, disk.rs, manage.rs)
- Changing verifier or planner prompts
- Rewriting preflight call sites, which already exist

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check sits where the budget is defined and loaded, so no job can start under an unaffordable budget.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests that fail without the change; fmt, clippy and test must pass.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check a too-small budget being refused, not that the code exists.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by config validation, not prose.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused check in config plus tests.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check of the diff. Run: `git diff CHANGELOG.md`. Observes: The Unreleased/Fixed entry names the root cause (`[budget] job_usd` never required to cover the reader, planning rounds, producer and verifier caps) and the new `Config::check_budget`, run when an athena.toml change is admitted (not at parse, so history stays readable). It says runtime preflight remains the guard for running jobs. It names 20261028-082151-67fa, 20261030-033434-3v30 and 20261030-151827-48rm as addressed. A Grep of CHANGELOG.md for each id and for `job_usd` finds all of them in the new entry.
- Criterion 2: Grep of non-test code for a second hand-written sum, then Read of `budget_reserve`. The existing stage test pins the values. Run: `cargo test -p athena-core every_stage_counts_only_what_is_still_to_run_and_takes_its_caps_from_config`. Observes: Grep over crates/ for `producer_usd *\+` and `\+ *[a-z_.]*verifier_usd`, outside `mod tests`, matches only `Config::run_usd` in config.rs. `budget_reserve` has no `+` of producer and verifier caps and calls `budget_required`, `planning_usd`, `critic_usd` and `run_usd`. The stage test, which carries its own expected sums, passes, so the shared function agrees with the earlier values.
- Criterion 3: New unit test `job_usd_below_the_reserve_is_refused_by_the_budget_check` in config.rs. Real TOML is parsed, no double. Entry points: `Config::parse`, then `Config::check_budget`. Preconditions: the text must change, so each `replace` of `job_usd = 12.00` is followed by an assertion that the text differs from REPO_CONFIG (the producer reads the exact line first). Run: `cargo test -p athena-data job_usd_below_the_reserve`. Observes: (a) job_usd 9.99 parses and `check_budget` returns `Err(ConfigError::BudgetBelowReserve{..})`, whose text contains `[budget] job_usd`, `$0.01 short` and `$10.00`. (b) Exactly 10.00 returns Ok. (c) 12.00 returns Ok. (d) An 8.00 config with the shipped planning caps still passes `Config::parse`. (e) The `unplanned` config at 5.49 is refused and at 5.50 is Ok. (f) Raising `max_rounds` in the parsed config moves the threshold by one planner+critic pair.
  Fails without the behavior: If `check_budget` is a no-op or ignores the [job_plan] caps, the Err assertions for 9.99 (planned) and 5.49 (unplanned) fail. If the check were moved into `Config::parse`, the assertion that the 8.00 config still parses fails.
  Test seam: `Config::parse` and `Config::check_budget` in crates/athena-data/src/config.rs. REPO_CONFIG and `unplanned` exist today in the config.rs tests module (read, lines 706-721). Step 2 adds `check_budget`; the test is added in step 4.
- Criterion 4: In reading.rs tests, extend `shipped_budget_covers_the_reserve`. The shipped config parses via the existing `shipped_config()`. `job_usd >= budget_required()`, `check_budget()` is Ok, and `budget_required()` equals `budget_reserve(&config, ReserveStage::Admission)`. Add `required_budget_follows_the_configured_caps`. It starts from the parsed shipped config and clones it with `planner_usd += 1.0`, then with `max_rounds += 1`, with no other code change. `budget_required` and `budget_reserve(Admission)` both rise by `max_rounds * 1.0` for the planner change and by one planner+critic pair for the rounds change. With `job_usd` set to the new requirement minus 0.01, `check_budget` is refused; at exactly the requirement it is accepted. Run: `cargo test -p athena-core budget`. Observes: The shipped athena.toml still parses and covers its reserve. The required budget tracks changed caps in both the helper and `budget_reserve`, and the check follows the changed cap.
  Fails without the behavior: If `budget_reserve` kept its own sum, its equality with `budget_required` breaks once a cap is changed. If `check_budget` ignored the [job_plan] caps, the refusal at requirement minus 0.01 fails.
  Test seam: `athena_core::reading::budget_reserve`, `Config::budget_required` and `Config::check_budget`, driven on a real `Config` from `shipped_config()`, which exists in the reading.rs tests (~3652, read earlier). Step 5 adds the new test and the `Config` helpers. Mutating the parsed struct is the input, not a double.
- Criterion 5: Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` as three separate commands. The test that discriminates is the new admission.rs test, which calls the admission entry through the existing helper `admit_files(&[("athena.toml", &text)])` and filters with `configuration(..)`. The text is BASE_CONFIG with `job_usd = 12.00` replaced by `job_usd = 9.00`, after asserting the replace changed it. Only athena.toml is in the change set, so `schema_changed` is false. Run: `cargo test --locked admission`. Observes: fmt, clippy and the full suite pass. The 9.00 candidate gives exactly one `configuration` violation for path athena.toml. Its detail contains `[budget] job_usd` and `$1.00 short`, plus `raise job_usd or lower a cap`, and does not contain `does not parse`. BASE_CONFIG at 12.00 and an `# edited` copy give none. The existing parse-error tests still pass, which shows the parse violation is unchanged. The runtime BudgetShort tests in run/tests.rs and run_pin_tests.rs pass unchanged, because `JobConfig::at` and `preflight` are untouched.
  Fails without the behavior: The new admission test `a_job_usd_below_the_stage_caps_is_a_configuration_violation` fails if the `check_budget` call in admission.rs is removed (no violation is produced for the 9.00 candidate). Unformatted code or a clippy warning fails the other two commands. A fixture that breaks the new `shipped_budget_covers_the_reserve` assertions fails the suite.
  Test seam: The existing `admit_files` helper and `configuration` filter in the admission.rs tests (read, lines 824-859) and the BASE_CONFIG constant (line 567) exist today. Step 6 adds the production call in the candidate config check and the test. There is no double; real git and Config parsing.
- Criterion 6: Inspect the list of changed files. Run: `git diff --stat`. Observes: Changed files are only crates/athena-data/src/config.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/admission.rs and CHANGELOG.md. athena.toml, defaults.toml, .github/workflows, crates/athena-cli/src/drain.rs, crates/athena-core/src/run/mod.rs, capability.rs and run_pin_tests.rs are absent from the list.

Planned files:

- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 5e2b184fe6ef → candidate b79e7ad4a58e. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  13 +++++
 crates/athena-core/src/admission.rs |  35 +++++++++++-
 crates/athena-core/src/reading.rs   |  63 ++++++++++++++++-----
 crates/athena-data/src/config.rs    | 108 ++++++++++++++++++++++++++++++++++++
 4 files changed, 203 insertions(+), 16 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1821 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
