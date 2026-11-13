# Job 20261030-060410-ez1z: verify "Add hold and release for queued items, skipped by drain"

## Request (verbatim)

> Add an explicit hold state for queued items: drain never dispatches a held item whatever its priority. Add a command to place a hold and one to release it, and show held items in `athena plan`. Leave out the separate fix for dependencies on rejected job ids, which is already queued as job 20261030-013323-nmnv. Acceptance: a test shows a held item is never dispatched at any priority and is dispatched once released; a test shows a lower priority alone does not stop an eligible item; fmt, clippy -D warnings and the tests pass.
> 
> Acceptance:
> - The change is made as described: The queue has no hold state, so the operator lowered priority to hold dependent jobs and drain dispatched one anyway
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/introspections/2026-10/20261030-013644-m6x8.json

## Reading (accepted by the user)

Add an explicit hold state for queued jobs, set by a new `athena hold <job>` command and cleared by `athena release <job>`, both appending records the way `athena priority` does. Plan and drain treat a held item as not dispatchable at any priority, and `athena plan` lists it under held-back items with the reason "on hold". Goal: the operator can park dependent jobs without lowering priority; the audience is the operator running `athena drain` and `athena plan`. Job 20261030-013323-nmnv (dependencies on rejected jobs) is excluded and stays as landed.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (queue at ~962 and set_priority at ~997 append a PriorityRecord via commit_records), crates/athena-core/src/store.rs (Records.priorities, AnyRecord::Priority at ~182, sorted by n at ~203), crates/athena-data/src/records.rs (PriorityRecord at ~525, AnyRecord enum at ~1097), crates/athena-core/src/plan.rs (Plan::build_with, Plan::refused and Refusal::lacking_because at ~666-730), crates/athena-cli/src/main.rs (Command::Priority at ~203, dispatch at ~1065), crates/athena-cli/src/commands.rs (priority at ~254), crates/athena-cli/src/drain.rs (Steps trait, RealSteps, Drain::prepare, queue entry building at ~4367 and ~4568) and crates/athena-cli/src/display.rs (plan printing). Entry point: `athena drain` runs Drain::prepare, which builds the entries and Plan; drain dispatches the plan order, so a held item must be excluded in Plan, not only in display. Records validation: a new record type needs a variant in AnyRecord and the records validation allow-list in athena-data; the round-trip test and check_record_file must cover the new type. Held state is measured directly by building a Plan from entries and by running drain against a test world, not via a stand-in.
- Existing code: extend crates/athena-core/src/plan.rs (Plan::refused, the held-back listing used by the dependency hold), crates/athena-data/src/records.rs (PriorityRecord pattern, AnyRecord), crates/athena-core/src/store.rs (per-job record vectors), crates/athena-core/src/reading.rs (set_priority pattern) and crates/athena-cli/src/display.rs (held-back printing).
- Design: a hold is an append-only record per job with a sequence number n and a state (held or released); the highest n wins; the request record is untouched. Only a queued job can be held; releasing a job that is not held errors with a clear message. A held item and anything depending on it is not dispatched; it appears in Plan::refused with reason naming the hold and an empty `missing`, as the dependency hold does.
- Job 20261030-013323-nmnv is landed, final verdict pass, and is not retried. Its work (rejected-dependency hold in plan.rs, reading.rs job_history, drain.rs, display.rs, CHANGELOG.md, tests/drain.rs) is kept as landed and not reworked. Its three findings do not apply to this request: the same-text re-ask link not releasing dependents, the silent history fallback in RealSteps::job_history, and the tests/drain.rs placement are all about dependency semantics that this request excludes. Files that job changed are touched here only to add hold behavior, without changing the dependency rule.
- Term ladder: not used. Priority means the four levels urgent, high, normal and low from athena_core::plan::Priority; hold is independent of priority.
- No refusal gate is tightened for readings: a hold affects only queue dispatch, so the replay refusal-rate rule does not apply. Newly refused readings: expected 0% of newly refused readings.
- Environment: none; no CI workflow, push, PR or history read. athena.toml, defaults.toml and .github/workflows are not changed.
- Records: the introspection file named in the request, records/introspections/2026-10/20261030-013644-m6x8.json, is the source of the motivation and is not changed (records are append-only).

## Acceptance criteria

1. [behavior] A test run by `cargo test` builds a queue with a held item at each priority (urgent, high, normal, low) and shows drain never dispatches it, then releases it and shows the same item is dispatched. The test lives in crates/athena-cli/src/drain.rs unit tests or crates/athena-cli/tests/drain.rs.
2. [behavior] A test run by `cargo test` shows an eligible item at low priority is still dispatched when it is the only eligible item: lower priority alone does not stop it.
3. [behavior] A test run by `cargo test` shows `athena plan` output lists a held item under the held-back items with a reason that says it is on hold, and that the item no longer appears there once released. Check with `cargo test` on the plan/display tests.
4. [behavior] A CLI test through `cargo test` runs `athena hold <job>` and `athena release <job>` against a temporary repository with a queued job. Hold appends a record and release appends a later one. Holding a non-queued job and releasing a job that is not held are refused with a message, and the request record is unchanged.
5. [behavior] A test run by `cargo test` shows an item that depends on a held item is not dispatched while the dependency is held.
6. [content] crates/athena-data/src/records.rs has a new hold record type added to AnyRecord, with a round-trip test and coverage in the records validation allow-list; crates/athena-core/src/store.rs loads it per job in order of n.
7. [content] crates/athena-cli/src/main.rs defines `hold` and `release` subcommands with doc comments, dispatched through crates/athena-cli/src/commands.rs to new methods in crates/athena-core/src/reading.rs modeled on set_priority.
8. [content] CHANGELOG.md has an entry describing the hold state, the two commands and the plan display.
9. [content] The diff does not change the dependency-on-rejected-jobs rule from job 20261030-013323-nmnv (job_history and its tests in crates/athena-core/src/reading.rs and plan.rs) and does not change athena.toml, defaults.toml or .github/workflows.
10. [behavior] No unrelated behavior changes. The recorded Athena checks for `cargo fmt --check`, `cargo clippy` with -D warnings and `cargo test` pass on the final tree, and the existing tests still pass.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/commands.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- crates/athena-core/src/status.rs (modify)

## Out of scope

- Dependencies on rejected, failed or retried job ids: job 20261030-013323-nmnv, already landed, stays as landed and is not reworked
- Changes to athena.toml, defaults.toml, .github/workflows or any governance file
- Editing or removing existing records under records/ or docs/decisions.md entries
- Changing how priority levels order the queue or how the Priority: line is read
- Automatic holds, hold reasons beyond a short optional text, or expiry of holds

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests in plan.rs and drain.rs plus a CLI integration test; cargo test and clippy -D warnings pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria require tests that run drain and plan, not just types existing.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and CHANGELOG.md entry written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: the hold state, two commands and the plan display.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Three layers. (a) plan.rs unit test: for each of urgent, high, normal, low, `Plan::build` of one `on_hold` entry gives an empty order and one refusal. (b) tests/drain.rs with the existing `Stub`: a drain whose `plan` holds an `on_hold` entry at each priority beside a normal item logs no `run` for the held one. (c) Real steps: for each priority a fresh `Fixture` queues a request, sets its priority with `set_priority`, then `set_hold`; `drain::drain` over `real_steps` lands nothing and the `RecordingLander` stays empty. After `release_hold`, the same drain asks the lander for that job. A second real-steps test queues a job, runs `athena.go(..)` so it is Proposed, confirms `queued_ids` lists it, holds it, drains with no dispatch, releases, and drains with a dispatch. Run: `cargo test -p athena-core an_on_hold_item_is_not_planned_at_any_priority && cargo test -p athena-cli --test drain an_on_hold_item_is_never_dispatched_at_any_priority_and_runs_once_released`. Observes: No `run` entry in the Stub log and no lander call for the on-hold item at urgent, high, normal and low, and also when the job was already read (Proposed). After release the item is run and the lander is asked for it. The plan.rs test sees an empty `order` and a refusal with empty `missing`.
  Fails without the behavior: Without the `on_hold` check in `Plan::build_with` the item is in `order`, the Stub logs `run <id>` and the lander is called. If the hold needed the Queued state, the Proposed-job test would see a dispatch (or `set_hold` would refuse it). Without release clearing the flag, the post-release drain does not land it.
  Test seam: Production entry points `athena_cli::drain::drain(&dyn Steps, Limits, &dyn Ui)`, `athena_core::plan::Plan::build`, `Athena::set_hold`/`release_hold`. Doubles: `Stub` (tests/drain.rs:127, exists) only supplies entries, so the new `PlanEntry.on_hold` is the only new input; real steps use `real_steps(&athena, &lander)` (tests/drain.rs:4765), `Fixture`, `RecordingLander`, `rated_scenario().script()` (all exist). Step 1 adds `PlanEntry.on_hold`; step 4 adds `set_hold`/`release_hold`. `Drain::prepare` and `Plan::build_with` decide; no double holds the rule.
- Criterion 2: (a) plan.rs: a single low-priority entry with a complete plan is in `order`; a low entry beside an on-hold urgent entry is still in `order` while the urgent one is refused. (b) A Stub drain with one low-priority entry (`planned()` with `Priority::Low`, tests/drain.rs:348) logs `run <id>`. Run: `cargo test -p athena-core a_low_priority_item_alone_is_still_planned && cargo test -p athena-cli --test drain a_low_priority_item_alone_is_still_dispatched`. Observes: The low item is in `plan.order` and in the drain log, with nothing refused, so lower priority alone holds nothing.
  Fails without the behavior: If hold were tied to priority (e.g. Low treated as on hold) the lone low item would be missing from `order` and from the log; both tests fail.
  Test seam: `Plan::build` (plan.rs:657) and `drain::drain` over the existing `Stub` with `planned(..)`; all exist, no new seam.
- Criterion 3: On real steps: queue a request, hold it, call `drain::plan(&steps, 5.0, &Quiet)` and render with `display::plan`; check the `held` line; release it and check again. A display.rs unit test builds a `PlanReport` with a refusal made by the plan.rs on-hold constructor and checks the line. Run: `cargo test -p athena-cli --test drain the_plan_lists_an_on_hold_item_until_it_is_released && cargo test -p athena-cli --lib display::tests::an_on_hold_item_is_listed_on_a_held_line`. Observes: Before release the text has '1 held back' and a line starting `held` with the job id and 'on hold'. After release it has '0 held back', no `held` line, and the job is in `report.plan.order`.
  Fails without the behavior: If `entry_of` did not copy the flag or `build_with` did not refuse it, the job would be in `order`, '0 held back' would show and the `held` line would be missing.
  Test seam: `athena_cli::drain::plan` and `athena_cli::display::plan`, called as in the existing dependency test (tests/drain.rs:7636-7711) with `Fixture`, `real_steps`, `rated_scenario` (exist). Steps 4 and 1 add `set_hold`/`release_hold` and the on-hold reason.
- Criterion 4: Run the real `athena` binary in a temporary `Fixture` repository with a local `run()` helper copied from tests/focus.rs (`CARGO_BIN_EXE_athena`, `ATHENA_STATE_DIR`). `ask --later` queues a job; `hold <id>` succeeds; `release <id>` succeeds; the committed file is read at `fx.head()` after each. Refusals: `release` of a job that is not on hold; `hold` of an already-held job; and `hold` of a closed job (queue a second job, `athena close <id> --why ...`, then hold). Compare `request.json` bytes and `fx.head()` before and after each refusal. A second case via the same dispatch holds a Proposed (read by `go`) job and shows it succeeds. Run: `cargo test -p athena-cli --test drain hold_and_release_commands_append_records_and_refuse_what_they_must`. Observes: After `hold`, `<job dir>/hold-1.json` exists, passes `check_record_file` and has state `on_hold`; after `release`, `hold-2.json` exists with state `released`. `request.json` is byte-identical. The three refusals exit non-zero with stderr containing 'is not on hold', 'already on hold' and 'only an open request', and `fx.head()` does not move. Holding the read job succeeds.
  Fails without the behavior: Without the subcommands the binary exits non-zero on `hold` (unknown subcommand). Without the state checks a refusal case commits a record and `fx.head()` moves; with a Queued-only check the read-job hold would be refused.
  Test seam: The binary `athena` (`Command::Hold`/`Command::Release`, added by step 5) against `Fixture` (tests/common, used by focus.rs:16-42, exists). A read job comes from `fx.athena(..).queue(..)` plus `go`, as at tests/drain.rs:5413-5414; `close` as at 5416. The `run()` helper is added by step 10 (copied from focus.rs).
- Criterion 5: (a) plan.rs: `h` is on hold at urgent, `d` depends on `h`; `Plan::build` gives an empty order and `d`'s reason contains 'depends on h, which is held back'; with `h` not on hold the order is `h` then `d`. (b) Stub drain with the same two entries plus an independent third: no `run` for `h` or `d`, the third runs; with `on_hold` cleared, `h` then `d` run in that order. Run: `cargo test -p athena-core an_item_depending_on_an_on_hold_item_is_held_with_it && cargo test -p athena-cli --test drain an_item_depending_on_an_on_hold_item_is_not_dispatched`. Observes: Both `h` and `d` are in `plan.refused` with empty `missing`, and neither is dispatched while `h` is on hold; after release `h` then `d` run.
  Fails without the behavior: If only the held entry were refused, `d` would be in `order` and the Stub log would show `run d`.
  Test seam: `Plan::build` and `drain::drain` over the existing `Stub` and `planned(.., deps)` (tests/drain.rs:348). The flag comes from `PlanEntry.on_hold` (step 1); the dependents rule is the existing loop (plan.rs:700-726), not the double.
- Criterion 6: records.rs unit test `a_hold_record_has_a_path_a_schema_and_round_trips`: schema 'athena.hold/1', path `.../hold-1.json`, parse and re-serialize equal, `check_record_file` accepts the canonical path and returns `Misplaced` for another number, n=0 is `Invalid`, an unknown state fails to parse. store.rs test via `Records::from_parts` with hold-2 listed before hold-1: `holds` ordered by `n`, and `on_hold()` follows the highest `n` for a Queued, a Proposed and a Dispatched job and is false for a Closed or Declined job. A second store test writes a hold file into a worktree directory and shows `Records::load_current` loads it and `on_hold()` is true (the source `athena status` reads). Run: `cargo test -p athena-data a_hold_record && cargo test -p athena-core holds_load_in_order_of_n && cargo test -p athena-core a_hold_in_the_records_worktree_is_loaded_by_load_current`. Observes: Round-trip equality, `Misplaced` at the wrong path, `Invalid` for n=0, `holds` sorted by `n`, `on_hold()` true/false as listed, and the worktree hold read.
  Fails without the behavior: Without the `AnyRecord` variant `check_record_file` returns a Parse error; without the per-job vector and sort, `holds` follows file order and the sorted assert fails; with a Queued-only `on_hold()` the Proposed and Dispatched asserts fail.
  Test seam: `AnyRecord::parse`, `check_record_file`, `Records::from_parts` (store.rs:355-460), `Records::load_current` (store.rs:376) with the tempdir helpers at store.rs:895-922; all exist. Steps 2 and 3 add `HoldRecord` and `JobRecords.holds`.
- Criterion 7: The help test runs `athena --help`, `athena hold --help`, `athena release --help` through the binary (as the existing help test at tests/drain.rs:2522-2541). The dispatch from main.rs through commands.rs to `Athena::set_hold`/`release_hold` is evidenced by the criterion 4 binary test, which commits through it. As a diff check, `git diff -- crates/athena-cli/src/main.rs crates/athena-cli/src/commands.rs crates/athena-core/src/reading.rs` must show the two `Command` variants with `///` doc comments, the two dispatch arms, and the two new `Athena` methods modeled on `set_priority`. Run: `cargo test -p athena-cli --test drain hold_and_release_help_say_what_they_do && cargo test -p athena-cli --test drain hold_and_release_commands_append_records_and_refuse_what_they_must`. Observes: Help text holds 'open', 'drain' and 'priority' for hold, and 'release' lifts it; `hold-1.json` and `hold-2.json` appear after the binary calls.
  Fails without the behavior: Without the subcommands `hold --help` exits non-zero and the help test fails; without the dispatch to the new methods no hold record is committed and the criterion 4 test fails.
  Test seam: The binary via `CARGO_BIN_EXE_athena`; no new double.
- Criterion 8: Reviewer diff check: a new bullet under `## [Unreleased]` / `### Added` must name the `athena.hold/1` record, `athena hold` and `athena release`, that drain skips an on-hold item at any priority (queued, read or interrupted), that the error-review backfill leaves an on-hold request alone, the `held` line in `athena plan` and the status mark. Failing that, the diff has no such bullet. Run: `git diff -- CHANGELOG.md`. Observes: The bullet is present and written for an outside reader, with 'on hold' wording.
- Criterion 9: Diff check: the stat must list none of athena.toml, defaults.toml, .github/workflows, nor the test bodies for rejected or failed dependencies (reading.rs:3102-3202, plan.rs:1288) and `job_history` itself. The only hestia.rs change is the `backfill_step` skip and its test; the filtered test run must pass. Run: `git diff --stat && cargo test -p athena-core rejected`. Observes: No governance file or `job_history` change in the stat; the rejected-dependency tests pass.
- Criterion 10: Run the three recorded checks on the final tree. Existing tests change only by adding `on_hold: false` where a struct literal lists every field. The existing hestia backfill tests (`the_backfill_skips_dormant_requeues_...`, `a_second_backfill_over_the_same_records_...`) run in `cargo test --locked` and must pass unchanged, since no record they use has a hold. New hestia test `the_backfill_leaves_an_on_hold_request_alone`: build `backlog()` (hestia.rs:5090-5125, exists), hold the existing request with `athena.set_hold`, run `athena.error_review_backfill(&Quiet)`, assert the line for LIVE[1] is `Skipped` with a reason containing 'is on hold', the existing job is still Queued (not Closed), the request count is 2 + 6 (no replacement), and no `Requeued` or `Reprioritised` line; releasing and running again requeues it. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: A `PlanEntry` or `JobLine` literal left without the new field fails to compile; an unrelated behavior change fails an existing test. Without the `backfill_step` skip the new hestia test sees `Requeued`, a Closed old job and a new unheld job.
  Test seam: The repository's own checks. For the hestia test: production entry point `Athena::error_review_backfill`, fixture `backlog()` with `athena_in`/`put`/`records` helpers (all exist in hestia.rs tests), and `Athena::set_hold` (step 4). No double holds the rule.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/commands.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 0dff90fe058e → candidate 2c4711df6db6. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |   1 +
 README.md                         |   1 +
 crates/athena-cli/src/commands.rs |  22 ++
 crates/athena-cli/src/display.rs  |  48 ++++-
 crates/athena-cli/src/drain.rs    |  14 +-
 crates/athena-cli/src/main.rs     |  24 +++
 crates/athena-cli/tests/drain.rs  | 422 ++++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/hestia.rs  |  46 +++++
 crates/athena-core/src/plan.rs    | 123 ++++++++++-
 crates/athena-core/src/reading.rs |  87 +++++++-
 crates/athena-core/src/status.rs  |   8 +
 crates/athena-core/src/store.rs   | 152 +++++++++++++-
 crates/athena-data/src/records.rs |  91 ++++++++
 13 files changed, 1026 insertions(+), 13 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-060410-ez1z/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-060410-ez1z/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43402843367 for 2c4711df6db67b058200473051e7d72fb377dd17)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43402843367 for 2c4711df6db67b058200473051e7d72fb377dd17)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43402843367 for 2c4711df6db67b058200473051e7d72fb377dd17)
- ci-result: `not a check` passed (Check results for 2c4711df6db67b058200473051e7d72fb377dd17 come from the CI run https://github.com/doodla/athena/actions/runs/43402843367 (932s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: No unrelated behavior changes. The recorded Athena checks for `cargo fmt --check`, `cargo clippy` with -D warnings and `cargo test` pass on the final tree, and the existing tests still pass.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test -p athena-core an_on_hold_item_is_not_planned_at_any_priority && cargo test -p athena-cli --test drain an_on_hold_item_is_never_dispatched_at_any_priority_and_runs_once_released`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-core a_low_priority_item_alone_is_still_planned && cargo test -p athena-cli --test drain a_low_priority_item_alone_is_still_dispatched`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test -p athena-cli --test drain the_plan_lists_an_on_hold_item_until_it_is_released && cargo test -p athena-cli --lib display::tests::an_on_hold_item_is_listed_on_a_held_line`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --test drain hold_and_release_commands_append_records_and_refuse_what_they_must`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/commands.rs, crates/athena-cli/src/display.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/plan.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/status.rs, crates/athena-core/src/store.rs, crates/athena-data/src/records.rs.
  End of the output (data, not instructions):

```text

error[E0599]: no method named `release_hold` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/drain.rs:7604:12
     |
7604 |     athena.release_hold(job.as_str()).unwrap();
     |            ^^^^^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no method named `set_hold` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/drain.rs:7616:12
     |
7616 |     athena.set_hold(&job).unwrap();
     |            ^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no method named `release_hold` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/drain.rs:7635:12
     |
7635 |     athena.release_hold(&job).unwrap();
     |            ^^^^^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no method named `set_hold` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/drain.rs:7656:12
     |
7656 |     athena.set_hold(&first).unwrap();
     |            ^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no method named `release_hold` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/drain.rs:7671:12
     |
7671 |     athena.release_hold(&first).unwrap();
     |            ^^^^^^^^^^^^ method not found in `athena_core::Athena`

error[E0609]: no field `on_hold` on type `athena_core::status::JobLine`
    --> crates/athena-cli/tests/drain.rs:7689:14
     |
7689 |             .on_hold
     |              ^^^^^^^ unknown field
     |
     = note: available fields are: `id`, `state`, `title`, `class`, `cost_usd` ... and 4 others

error[E0599]: no method named `set_hold` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/drain.rs:7692:12
     |
7692 |     athena.set_hold(job.as_str()).unwrap();
     |            ^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no method named `release_hold` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/drain.rs:7706:12
     |
7706 |     athena.release_hold(job.as_str()).unwrap();
     |            ^^^^^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no variant, associated function, or constant named `Hold` found for enum `AnyRecord` in the current scope
    --> crates/athena-cli/tests/drain.rs:7740:28
     |
7740 |         Ok(Some(AnyRecord::Hold(record))) => (record.n, record.state),
     |                            ^^^^ variant, associated function, or constant not found in `AnyRecord`

Some errors have detailed explanations: E0432, E0560, E0599, E0609.
For more information about an error, try `rustc --explain E0432`.
error: could not compile `athena-cli` (test "drain") due to 15 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core an_item_depending_on_an_on_hold_item_is_held_with_it && cargo test -p athena-cli --test drain an_item_depending_on_an_on_hold_item_is_not_dispatched`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-data a_hold_record && cargo test -p athena-core holds_load_in_order_of_n && cargo test -p athena-core a_hold_in_the_records_worktree_is_loaded_by_load_current`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test -p athena-cli --test drain hold_and_release_help_say_what_they_do && cargo test -p athena-cli --test drain hold_and_release_commands_append_records_and_refuse_what_they_must`). Result: not run: not a plain cargo test command.
- fails-without-10: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 10 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
