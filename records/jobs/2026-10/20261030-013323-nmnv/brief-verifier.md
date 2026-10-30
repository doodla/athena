# Job 20261030-013323-nmnv: verify "Hold items whose dependency was rejected with no landed retry"

## Request (verbatim)

> Priority: urgent
> athena plan (and so drain's order) treats a dependency on a rejected job as met. Example: 20261029-142858-5c0p says it depends on 20261029-142853-x4nr, which was rejected twice and re-asked as 20261030-013143-as0j, yet the plan ordered 5c0p first and would have worked it before its dependency existed.
> Change: a dependency is met only when the named job landed, or when a later job that landed is linked to it as its retry. A dependency on a rejected or failed job with no landed successor holds the dependent item back, with the reason shown under the plan's held-back items.
> Acceptance:
> - A test through athena plan shows an item that depends on a rejected job is held back with that reason, and is no longer held once a retry of that job landed.
> - Existing plan and drain tests pass.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry.
> Files: crates/athena-core/src/plan.rs, CHANGELOG.md

## Reading (accepted by the user)

Fix `athena plan` and drain ordering so a dependency counts as met only when the named job landed, or a landed job is linked as its retry. An item depending on a rejected or failed job with no landed successor is held back, with the reason under the plan's held-back items. Goal: stop jobs like 20261029-142858-5c0p running before their dependency 20261029-142853-x4nr has a landed successor (20261030-013143-as0j). Audience: the owner reading the plan and the drain.

Assumptions:
- Code survey: crates/athena-core/src/plan.rs is the entry point. `Plan::build` (line ~418) gets only the open entries. A dependency on an id outside them counts as done (`!known.contains`), so a rejected or landed job is treated alike. `Plan::refused` (a `Refusal` with a reason) is what the display lists as held-back items (crates/athena-cli/src/display.rs ~581). crates/athena-cli/src/drain.rs (~1469 `Plan::build`, ~2354 `entry_of`) builds the entries from records, and crates/athena-core/src/reading.rs `plan_entry` with its `Lookup` fills `depends_on`. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched.
- Design: `PlanEntry` gets a field (default empty) listing each dependency that is unmet and why, such as rejected or failed with no landed successor. `Plan::build` moves such an entry, and anything that depends on it, to `refused` with a reason naming the dependency, instead of ordering it. `entry_of` in drain.rs (and `reading::plan_entry` if needed) fills the field from job records: end state landed means met; else a job whose retry-of link names it and which landed means met. Tests in plan.rs use `Plan::build` with that field. The spec's Files line names only plan.rs and CHANGELOG.md, but populating it from records needs drain.rs and reading.rs, so they are listed at lower probability.
- Previous jobs. 20261029-142858-5c0p (A4, failed, never verified, no candidate): not redone here. This change only stops it being ordered ahead of its dependency; its own work stays for a later run. 20261029-142853-x4nr (A3, rejected): its candidate (drain.rs, state.rs, CHANGELOG.md) is dropped. Its findings no longer apply because 20261030-013143-as0j, which landed, fixed the worktree path and the stages. 20261030-013143-as0j (landed, pass): kept as landed, including drain.rs, state.rs, run/mod.rs, run/producer.rs, tests/drain.rs, tests/lifecycle.rs, CHANGELOG.md and docs/design/async-drain.md. Its minor finding (a moved row keeps the verifier stage) is a separate issue and left alone.
- A job counts as the retry of another by the existing retry link in the job records (the same link that `RealSteps::read`'s Retry branch uses); no new link is invented.
- A failed or rejected dependency with a landed retry is met. A dependency on a job id that does not exist keeps today's behavior (counts as done), since the request only concerns rejected and failed jobs.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/plan.rs, run with `cargo test -p athena-core plan`, builds a plan in which item B depends on a rejected job with no landed retry. B is absent from `order` and present in `refused` with a reason that names the rejected dependency.
2. [behavior] A new test in the same module shows B is not held and is ordered after the retry once a retry of that job has landed. Run `cargo test -p athena-core plan`.
3. [behavior] A test through the real `athena plan` or drain entry in crates/athena-cli/tests (run with `cargo test -p athena-cli`) shows an item depending on a rejected job listed under the held-back items with the reason, and no longer held after a landed retry is linked.
4. [behavior] A dependent of a held item is held as well and is not ordered. Existing plan.rs and drain tests pass unchanged: `cargo test`.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
6. [content] CHANGELOG.md has an entry stating that a dependency on a rejected or failed job without a landed retry now holds the dependent item, with the reason shown.
7. [content] The doc comments on `Plan::build` and the module header in plan.rs describe the new rule instead of 'an id not among the entries counts as done' for rejected or failed jobs.
8. [content] The diff touches no athena.toml, defaults.toml, .github/workflows or records files.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)

## Out of scope

- Reworking or re-running 20261029-142858-5c0p (A4); its work stays for a later job
- 20261029-142853-x4nr's candidate changes to drain.rs, state.rs and CHANGELOG.md: dropped, superseded by 20261030-013143-as0j
- Changes to files 20261030-013143-as0j landed: run/mod.rs, run/producer.rs, state.rs, tests/drain.rs, tests/lifecycle.rs and docs/design/async-drain.md stay as landed
- The in-flight row stage issue from 20261030-013143-as0j's verification
- Changes to athena.toml, defaults.toml, .github/workflows
- Creating new retry links or changing how retries are recorded

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in crates/athena-core and crates/athena-cli.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy -D warnings are criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the held and released ordering through Plan::build and the plan entry, not that fields exist.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands the verifier runs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The rule is fixed in Plan::build, which both `athena plan` and the drain use, not in one caller.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the dependency rule.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a dependent is not ordered before a landed successor exists.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New unit test in plan.rs calling the production entry `Plan::build_with(&entries, &history)`. Entries come from the existing test helper `entry(...)` (exists today): a ready entry `b` with `depends_on` ["x"]; `x` is not an entry. `history` is plain data: one `JobEnd` for `x`, rejected, no retry link. No double. `JobEnd`, `Ending` and `build_with` are added by plan steps 1 and 3. A control call `Plan::build(&entries)` with no history still orders `b`. Run: `cargo test -p athena-core plan::tests::a_dependency_on_a_rejected_job_with_no_landed_retry_holds_the_item_and_its_dependents`. Observes: `b` is not in `plan.order`; `plan.refused` has one item with id `b`, an empty `missing`, and a reason containing `x` and 'rejected'.
  Fails without the behavior: With `build_with` ignoring the history (today's rule, an id outside the entries counts as done), `b` is in `order` and `refused` is empty, so both assertions fail.
- Criterion 2: New unit test in plan.rs, same entry point `Plan::build_with`, plain-data history. Case 1: `x` rejected, `r1` rejected with retry link to `x`, `r2` landed with link to `r1`. Case 2: `x` rejected and `r` landed linked to `x`, where `r` is also an open entry that `b` depends on. Each case calls `build_with` before and after the landed job is added. A landed job linked to a different job must not release `b`. A cycle in the links must end. Fixtures are passed as arguments; helper `entry` exists today. Run: `cargo test -p athena-core plan::tests::a_dependency_is_met_once_a_retry_of_it_landed_and_the_item_is_ordered_after_it`. Observes: Before the landed retry exists `b` is refused. After, `b` is in `order`, `refused` is empty, and `b` comes after `r` where `r` is an entry. The unrelated landed job leaves `b` held.
  Fails without the behavior: If the retry link is not followed, `b` stays in `refused` after the landed retry is added, so the not-held and order assertions fail. If a landed retry were ignored, the chain case fails too.
- Criterion 3: New test appended to crates/athena-cli/tests/drain.rs, calling production `drain::plan(&real_steps(&athena, &lander), 5.0, &Quiet)` and `display::plan(&report)`. Seam: `RealSteps` reading real records via `Records::load` (exists today); fixture `Fixture::new()` with `rated_scenario().script()` and real `athena.queue`/`athena.go` (exist today). Setup: job `x` is queued and run to landed, then its Outcome record is rewritten to rejected and committed with `commit_records_to`. Job `b` is queued with text naming `x`, a priority line and acceptance bullets. Then `athena.queue("try again on <x>")` creates the retry, driven to landed by a real `go`; if that is not feasible, a landed Outcome re-keyed to the retry job is committed, and the producer must check `Records::load` accepts it. The reader `reading::job_history` and the `Steps::job_history` override are added by plan steps 5 and 6. The tests/job_plan.rs file is not used because it lacks these helpers. Run: `cargo test -p athena-cli --test drain the_plan_holds_an_item_whose_dependency_was_rejected_until_a_retry_landed`. Observes: With `x` rejected and no landed retry: `report.plan.order` lacks `b`; `report.plan.refused` has `b` with a reason naming `x`; the `display::plan` text has a `held` line for `b` with that reason and the header counts 1 held back. After the landed retry: `b` is in `order`, `refused` has no `b`, and no `held` line for `b` appears.
  Fails without the behavior: Without the `Steps::job_history` override and `reading::job_history`, `drain::plan` passes no history, `b` is ordered, and the hold and display assertions fail.
- Criterion 4: New unit test `a_dependent_of_a_held_item_is_held_too` calling `Plan::build_with` with `b` (depends on rejected `x`), `c` (depends on `b`), `d` (depends on `c`) and an unrelated `e`. The existing plan.rs and tests/drain.rs tests run unchanged in the full `cargo test --locked`; no existing test is edited. Run: `cargo test -p athena-core plan:: && cargo test --locked`. Observes: `order` is exactly [`e`]; `refused` has `b`, `c`, `d` in that order, with 'depends on b, which is held back' on `c` and a matching reason on `d`. The full test run has no failure in existing tests.
  Fails without the behavior: If only direct dependents were held, or the hold ran after ordering, `d` or `c` would be ordered and the order assertion fails. If `Plan::build` semantics changed, an existing plan or drain test fails.
- Criterion 5: Run the three Athena checks on the finished diff. They cover the new code only because the new tests, the `JobEnd` fields, `build_with`, the `Steps` default method and the display change all must compile, be used and pass. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0: no format diff, no clippy warning (including dead-code on `JobEnd`, `Ending` and `Refusal::held`), and every test passes including the new ones.
  Fails without the behavior: If the rule were not wired in, `cargo test --locked` fails on the new criterion 1-4 tests, and `clippy -D warnings` fails on the unused `JobEnd`, `Ending` or `job_history` items (dead_code); if the code is unformatted, `fmt --check` fails.
- Criterion 6: Read the CHANGELOG.md diff. Run: `git diff -- CHANGELOG.md`. Observes: A new bullet under `## [Unreleased]` (Fixed) says that a dependency on a rejected or failed job with no landed retry now holds the dependent item in `athena plan` and drain, with the reason under the held-back items.
- Criterion 7: Read the plan.rs diff for the module header and the doc comments of `Plan::build` and `Plan::build_with`. Run: `git diff -- crates/athena-core/src/plan.rs`. Observes: The header's Order bullet no longer says an unlisted dependency counts as done for rejected or failed jobs. It states the rule: met when landed or when a landed job is linked as its retry, otherwise held with a reason. Both function docs say the same. Only unknown, open, dropped, declined and abandoned ids still count as done.
- Criterion 8: Check the changed-file list. Run: `git status`. Observes: Changed files are only plan.rs, reading.rs, drain.rs, display.rs, tests/drain.rs and CHANGELOG.md. None is athena.toml, defaults.toml, a file under .github/workflows or a records file.

Planned files:

- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base a2f22a122f72 → candidate 09d91e9049ec. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  10 ++
 crates/athena-cli/src/display.rs  |  26 ++--
 crates/athena-cli/src/drain.rs    |  16 ++-
 crates/athena-cli/tests/drain.rs  |  78 +++++++++++
 crates/athena-core/src/plan.rs    | 278 +++++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/reading.rs |  37 ++++-
 6 files changed, 430 insertions(+), 15 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1737 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
