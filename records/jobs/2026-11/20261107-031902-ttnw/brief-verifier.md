# Job 20261107-031902-ttnw: verify "Give Steward and Hestia requests a guaranteed one-in-three drain turn"

## Request (verbatim)

> Give Athena's own improvement requests a guaranteed turn in the drain.
> 
> Requests from the Steward and from Hestia's error review often name the right cause, but they rarely run: of 32 Steward-sourced requests filed 4 to 6 Nov, 5 ran, because requests a person marks urgent or high always sort first.
> 
> Change the drain's ordering so that:
> - at least one job in every three dispatched comes from Athena's own improvement requests (Steward or Hestia) when any are queued and their reading passes
> - an improvement request whose cause has been recorded in three or more distinct jobs is treated as urgent
> - the plan and the drain report show which jobs took the improvement turn
> 
> Acceptance:
> - a test: with six person-filed urgent requests and two Steward requests queued, the first three dispatches include at least one Steward request
> - a test: a Hestia request whose failure class appears in three distinct jobs is ordered as urgent
> - a test: with no improvement requests queued, ordering is unchanged
> - athena plan prints which items take the improvement turn
> - all tests pass; CHANGELOG states only what the diff does
> 
> Files changed: the drain's ordering and planning code, the plan display, tests, CHANGELOG.md.

## Reading (accepted by the user)

Change the drain's ordering so at least one in every three dispatches is a Steward- or Hestia-sourced improvement request (when queued and its reading passes), and a request whose cause recurs in three or more distinct jobs counts as urgent. The plan and drain report show which items took the improvement turn. Audience: the maintainer reviewing plan and drain output.

Assumptions:
- Code survey: read crates/athena-core/src/plan.rs (PlanEntry, Priority, Plan::build_with, Plan::build_capped, which orders by effective priority, then rung, then age, with person-set items protected from caps), crates/athena-cli/src/drain.rs (plan_entry, priority_caps, PlanReport, the drain loop), crates/athena-cli/src/display.rs (plan() prints the PlanReport), crates/athena-core/src/hestia.rs (REPEAT_JOBS, cause_key, repeated_causes, root_cause_request) and crates/athena-core/src/steward.rs. The entry point is the drain's plan build; ordering runs after the reading check (missing() and held items), so only requests whose reading passes can take the turn.
- Existing code: extend crates/athena-core/src/plan.rs (PlanEntry, Plan) and reuse crates/athena-core/src/hestia.rs repeated_causes and cause_key for the three-distinct-jobs rule; the existing repeat threshold there is two jobs, so a separate threshold of three is added for the urgent promotion rather than changing it.
- No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected. No refusal rule is added: the change reorders work and refuses no readings, so no refusal-rate figure is needed.
- Improvement requests are identified by their recorded source (Steward or Hestia) on the request, set at filing; the producer finds the existing field or adds a minimal marker on PlanEntry. The promotion to urgent is a derived in-memory priority, not written to records, and a person-set priority is never lowered.
- The turn is a slot rule: in each window of three consecutive dispatches in plan order, if the window has no improvement item and one is ready (dependencies met), the best-ranked improvement item moves into the window. Dependencies and disjoint groups are still respected.
- No athena.toml, defaults.toml or workflow change is needed; the one-in-three ratio is a constant in code.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a new test queues six person-filed urgent requests and two Steward requests, builds the plan, and asserts the first three dispatched items include at least one Steward request.
2. [behavior] Run `cargo test -p athena-core`: a new test gives a Hestia request whose failure class appears in introspections of three distinct jobs and asserts it is ordered as urgent; a companion case with two distinct jobs asserts it is not promoted.
3. [behavior] Run `cargo test`: a new test with no Steward or Hestia requests queued asserts the plan order is identical to the order before the change, and the existing plan tests still pass.
4. [behavior] Run `cargo test -p athena-cli`: a test of the `athena plan` output (display::plan or the CLI drain test) asserts it names each item that takes the improvement turn, and that the drain report lists them.
5. [content] An improvement request whose reading fails (missing goal, rung or acceptance) or whose dependency is unmet is not given a turn; a test or the diff in plan.rs shows the turn is taken only from ready, passing entries.
6. [behavior] Run `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test`: all pass.
7. [content] CHANGELOG.md has an entry for the guaranteed improvement turn, the three-job urgent promotion and the plan and report display, and each claim matches the diff, with nothing claimed beyond it.
8. [content] The changed files are limited to the ordering and planning code, the plan display, tests and CHANGELOG.md, with no change to athena.toml or defaults.toml.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the existing two-job repeat threshold in hestia.rs for filing root-cause requests
- Changing how the Steward or Hestia file requests or what they say
- Changing priority caps ([priority_caps]) or person-set priority semantics
- Editing athena.toml, defaults.toml or workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for each rule; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise ordering and display output.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome checked is that improvement requests get dispatched, shown by the ordering tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to ordering, planning and display.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The ordering rule is fixed at the plan sort rather than by raising individual requests' priority.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the named cargo commands.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and CHANGELOG written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in plan.rs build 8 entries: six `Priority::Urgent` person-filed ids and two `Priority::Normal` Steward ids, with empty dependencies and touches. Case A uses `TierCaps::UNLIMITED`. Case B uses `TierCaps { urgent: 3, high: 10 }` with the six ids in `person_set`, so all six stay urgent. `Plan::build_improving` gets `Improvement { items: [s1, s2], since_turn: 0, .. }`. More cases: `since_turn` 2 puts an improvement item first; `since_turn` 1 puts it second; a tried improvement item is not counted. A simulation builds the plan from the full order, then removes the first dispatched id and rebuilds with the advanced `since_turn` and tried set, for 8 dispatches, and asserts the dispatch order equals the first build's `order`. Run: `cargo test -p athena-core improvement_turn`. Observes: `plan.order[..3]` contains a Steward id in cases A and B. `plan.turns` names both Steward ids at positions 3 and 6 (1-based) with `moved: true`. The same inputs through `Plan::build_capped` leave both Steward ids last. The simulated dispatch order equals the first plan's order. No more than two non-improvement ids sit between turns.
  Fails without the behavior: Without the gap rule the order is the six urgent ids and then both Steward ids, so the first-three assertion fails. Without `since_turn`, the replan simulation and the drain-level test in criterion 4 fail because the item is pushed back on each replan.
  Test seam: Production entry point `Plan::build_improving` (added by step 2). The `Improvement` value is a plain argument and no double is used. Plan entries are built with the struct-literal helper that the existing plan.rs tests use (exists). Nothing under test is inside a fixture.
- Criterion 2: Two layers. (a) hestia.rs `improvement_of` tests use the existing harness `athena_in`, `class`, `introspection` and `put` (hestia.rs:2508-2567). The error-review request is filed through the real path `athena.error_review_backfill` (as the tests at hestia.rs:3630-3712 do), so it lands on channel `cli` through `Athena::queue`. A failure class with introspections in jobs a, b, c gives `cause_jobs == 3`; a class with only a, b gives 2. A throughput record built with `request_job` set to an open request job (the way the existing throughput tests put a `ThroughputReview`) is listed, with no cause count; with the request closed it is not listed. A request queued with `athena.queue` whose text holds the line `Spec requirement: R1` is listed; the same text with the line absent is not. A plain person request citing nothing is not listed. A steward-channel `Request` (built as the fixture at hestia.rs:3354 does) is listed. A request closed by `athena close` is not listed. A slowdown request built with `slowdown_request` and queued is asserted NOT listed, which documents the gap. (b) plan.rs `repeat_urgent` tests pass `Improvement { cause_jobs: {h: 3} }` and an older normal entry to `build_improving`. A 3-job request is returned at Urgent, with `plan.promoted` holding `{jobs: 3, from: Normal}` and `h` ordered first. The 2-job case stays normal with `promoted` empty and the order by age. A `person_set` item is not promoted. With the urgent tier full of three person-set items, a promoted item is demoted to high and shows in both `plan.promoted` and `plan.demoted`. Run: `cargo test -p athena-core repeat_urgent && cargo test -p athena-core improvement_of`. Observes: The job filed by the real Hestia path is found as an improvement request with 3 distinct jobs, and the same cause with 2 jobs reports 2. In the plan the 3-job request is Urgent and the 2-job one is normal. Throughput and spec requests are listed through their recorded links. The slowdown request is not.
  Fails without the behavior: Without the promotion the 3-job request stays normal and sorts after an older normal item, so the urgent assertion fails. A threshold of 2 fails the two-job companion. A channel-only marker fails the `improvement_of` tests, because the requests are on channel `cli`. Without the `request_job` and spec-line branches, the throughput and spec cases fail.
  Test seam: Production entry points `hestia::improvement_of` (added by step 4) and `Plan::build_improving` (step 2). Fixtures are a temporary git repository and `Athena` from the existing hestia test helpers (exist), injected by value; the error-review filing is the real `error_review_backfill`. `improvement_of`, `repeated_causes` and `spec_request_is_open`'s line test run unmodified in behavior.
- Criterion 3: A plan.rs test builds a mixed entry set (urgent, high, normal, low, with dependencies and touches, and a refused and a held item). It asserts `Plan::build_improving(.., Improvement::default())` equals `Plan::build_capped(..)` in plan and entries, for `TierCaps::UNLIMITED` and the default caps (3, 10). A second case passes improvement ids that are not among the entries and asserts the same. A third has improvement ids present but all refused, and the order equals the `build_capped` order. A tests/drain.rs test drives the drain with a `Stub` that has no improvement items and asserts the run order is the pre-change plan order. Run: `cargo test --locked`. Observes: Plan, order, groups, refused and demoted are equal, and `turns` and `promoted` are empty. The whole existing suite still passes.
  Fails without the behavior: If the new code reordered non-improvement work, the equality assertion and the existing plan and drain tests (such as `items_are_worked_in_plan_order_not_oldest_first`) would fail.
  Test seam: `Plan::build_capped` (exists) is the reference, and `Plan::build_improving` (step 2) is the code under test. The `Stub` in tests/drain.rs (exists) gets only the default `Steps::improvement()`, which returns an empty value.
- Criterion 4: tests/drain.rs: `Stub` gets an `improvement: Improvement` field (items and cause counts) and a `Steps::improvement` override that returns it. It holds 6 person-filed urgent entries and 2 improvement entries, one with cause count 3, with `person_set` and caps left at the `Steps` defaults. `drain::plan(&stub, 5.0, &Quiet)` gives a `PlanReport`, and `display::plan(&report)` is asserted to contain a `turn` line and a `promoted` line, each naming the id and title. A promoted-then-demoted case asserts the same line shows the demotion. A second test runs `drain::drain` with `max_jobs` 8 and asserts the `run` log order equals the `PlanReport` order from `drain::plan`, that `report.improvement_turns` names the improvement items that ran as turns, and that `display::drain(&report)` has an `improvement` line naming each with its job. The concurrent case runs `drain::drain_concurrent(&shared, limits(..), &Quiet, 3)` with the `Shared` double (tests/drain.rs:7183) after step 9 gives it the `improvement` field and override; it has 6 independent urgent items and 2 improvement items, and asserts an improvement id is among the first three `start` log entries and that no more than two non-improvement items start before it. A display.rs unit test builds a `PlanReport` and a `Report` by hand with non-empty `turns` and `improvement_turns`, and an empty one that prints neither line. Run: `cargo test -p athena-cli improvement`. Observes: The plan text contains a line like `turn        <id> · <title> takes the improvement turn` and a line like `promoted    <id> · <title>: cause recorded in 3 jobs, raised from normal to urgent`. The drain text names each item that took a turn. With no improvement items neither line appears, and the existing plan-text needles still match.
  Fails without the behavior: Without `turns`, `promoted`, the `Report` field and the display lines, the text has none of the needles and `report.improvement_turns` does not compile. With `build_capped` in place of `build_improving`, the concurrent first-three assertion fails, because all six urgent items start first.
  Test seam: Production entry points `drain::plan`, `drain::drain`, `drain::drain_concurrent`, `display::plan` and `display::drain` (all exist). The injection is the `Stub` and the `Shared` double, each with a new `improvement` field and `Steps::improvement` override (added by steps 6 and 9; `Shared` is `Sync` because `Improvement` is plain data). The doubles only report which ids are improvement requests and their cause counts; the ordering and the report are production code.
- Criterion 5: plan.rs tests: (a) an improvement entry with no acceptance has `missing()` non-empty and is in `refused`; a ready improvement item behind it takes the turn. (b) An improvement entry depends on a job that `history` shows failed with no landed retry; it is held and in `refused`. (c) An improvement entry depends on an in-plan item that would sit at or after the target slot: it is skipped for that slot and takes a later one once its dependency has run; every dependency precedes its dependent in `order`. (d) With no ready improvement item, `turns` is empty and the order equals the `build_capped` order. (e) A moved turn item produces no `priority_violations` entry. Run: `cargo test -p athena-core improvement_turn_skips`. Observes: `plan.turns` never names a refused, held or unready item. In every case a dependency precedes its dependent.
  Fails without the behavior: If the walk accepted any improvement item, the refused or held item would appear in `plan.turns` or the dependent would run before its dependency, and the assertions fail.
  Test seam: `Plan::build_improving` with plain entries and `history` values (the `JobEnd` constructors that the existing hold tests in plan.rs use, which exist). The candidate filter is the production walk in plan.rs (step 2), not a double.
- Criterion 6: Run the three gate commands over the workspace. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0, including `system_facts_budgets_match_computed` (no prompt or brief text changes).
  Fails without the behavior: A new `Steps` method, `Plan` field, `Report` field or `State` field not wired through the struct literals (the `Report` literal at drain.rs:3057 and the `State` literal in `Drain::new`) would not compile or would fail clippy.
  Test seam: The cargo toolchain over the whole workspace; it exists.
- Criterion 7: Read the new entry under `## [Unreleased]` / `### Added` and match each sentence to the diff. It states: the root cause (the drain ordered by person-set priority alone); the guaranteed turn (at most two other dispatches between improvement requests while a ready one is queued) for open steward-channel requests, the open request that cites a repeated cause's introspections (a person's request that cites them counts too), the throughput design request recorded as a proposal's `request_job`, and the open spec-review request; that the throughput slowdown request is not covered because no record links it; the raise to urgent for a cause recorded in three or more distinct jobs, made before `[priority_caps]` so a full urgent tier can demote it again, and never applied to a priority a person set; the `turn` and `promoted` lines in `athena plan`; the improvement line in the drain report; the new `Steps::improvement` method. Run: `git diff CHANGELOG.md`. Observes: Every sentence maps to a changed function or test, and nothing is claimed about the filing threshold, Hestia or Steward text, or caps.
  Fails without the behavior: An entry that claimed more than the diff (for example the slowdown request being covered, or a change to the filing threshold or caps) would not map to a changed function or test.
  Test seam: Not a test; a reading of the diff by the verifier with `git diff`.
- Criterion 8: List the changed paths and compare them with the deliverables. Run: `git status`. Observes: Only crates/athena-core/src/{plan,hestia,steward}.rs, crates/athena-cli/src/{drain,display,main}.rs, crates/athena-cli/tests/drain.rs and CHANGELOG.md changed. athena.toml, defaults.toml and workflows are not touched.
  Fails without the behavior: A changed athena.toml, defaults.toml or workflow would show in the listed paths.
  Test seam: Not a test; `git status` over the working tree.

Planned files:

- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 5aec315ccdae → candidate d1b6a5e67a50. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  23 ++
 crates/athena-cli/src/display.rs  | 202 ++++++++++++-
 crates/athena-cli/src/drain.rs    |  88 +++++-
 crates/athena-cli/src/main.rs     |  12 +-
 crates/athena-cli/tests/drain.rs  | 225 +++++++++++++-
 crates/athena-core/src/hestia.rs  | 319 +++++++++++++++++++-
 crates/athena-core/src/plan.rs    | 617 +++++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/steward.rs |  63 ++++
 8 files changed, 1533 insertions(+), 16 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261107-031902-ttnw/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-031902-ttnw/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42338283026 for d1b6a5e67a5099661c7246d01074b9eaa1f7b5cf)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42338283026 for d1b6a5e67a5099661c7246d01074b9eaa1f7b5cf)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42338283026 for d1b6a5e67a5099661c7246d01074b9eaa1f7b5cf)
- ci-result: `not a check` passed (Check results for d1b6a5e67a5099661c7246d01074b9eaa1f7b5cf come from the CI run https://github.com/doodla/athena/actions/runs/42338283026 (581s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` and `cargo test`: all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core improvement_turn`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/plan.rs, crates/athena-core/src/steward.rs.
  End of the output (data, not instructions):

```text
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no associated function or constant named `build_improving` found for struct `plan::Plan` in the current scope
    --> crates/athena-core/src/plan.rs:2069:36
     |
 410 | pub struct Plan {
     | --------------- associated function or constant `build_improving` not found for this struct
...
2069 |         let (plan, capped) = Plan::build_improving(
     |                                    ^^^^^^^^^^^^^^^ associated function or constant not found in `plan::Plan`
     |
note: if you're trying to build a new `plan::Plan` consider using one of the following associated functions:
      plan::Plan::build
      plan::Plan::build_with
    --> crates/athena-core/src/plan.rs:548:5
     |
 548 |     pub fn build(entries: &[PlanEntry]) -> Plan {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
 560 |     pub fn build_with(entries: &[PlanEntry], history: &[JobEnd]) -> Plan {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0560]: struct `plan::Plan` has no field named `turns`
    --> crates/athena-core/src/plan.rs:2079:13
     |
2079 |             turns: Vec::new(),
     |             ^^^^^ `plan::Plan` does not have this field
     |
     = note: available fields are: `order`, `groups`, `refused`, `cyclic`, `demoted`

error[E0599]: no associated function or constant named `build_improving` found for struct `plan::Plan` in the current scope
    --> crates/athena-core/src/plan.rs:2090:42
     |
 410 | pub struct Plan {
     | --------------- associated function or constant `build_improving` not found for this struct
...
2090 |         let (mut broken, capped) = Plan::build_improving(
     |                                          ^^^^^^^^^^^^^^^ associated function or constant not found in `plan::Plan`
     |
note: if you're trying to build a new `plan::Plan` consider using one of the following associated functions:
      plan::Plan::build
      plan::Plan::build_with
    --> crates/athena-core/src/plan.rs:548:5
     |
 548 |     pub fn build(entries: &[PlanEntry]) -> Plan {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
 560 |     pub fn build_with(entries: &[PlanEntry], history: &[JobEnd]) -> Plan {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0425]: cannot find function `improvement_requests` in this scope
    --> crates/athena-core/src/steward.rs:2135:13
     |
2135 |             improvement_requests(&records),
     |             ^^^^^^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0422, E0425, E0433, E0560, E0599.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 42 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-core repeat_urgent && cargo test -p athena-core improvement_of`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/plan.rs, crates/athena-core/src/steward.rs.
  End of the output (data, not instructions):

```text

error[E0599]: no associated function or constant named `build_improving` found for struct `plan::Plan` in the current scope
    --> crates/athena-core/src/plan.rs:2069:36
     |
 410 | pub struct Plan {
     | --------------- associated function or constant `build_improving` not found for this struct
...
2069 |         let (plan, capped) = Plan::build_improving(
     |                                    ^^^^^^^^^^^^^^^ associated function or constant not found in `plan::Plan`
     |
note: if you're trying to build a new `plan::Plan` consider using one of the following associated functions:
      plan::Plan::build
      plan::Plan::build_with
    --> crates/athena-core/src/plan.rs:548:5
     |
 548 |     pub fn build(entries: &[PlanEntry]) -> Plan {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
 560 |     pub fn build_with(entries: &[PlanEntry], history: &[JobEnd]) -> Plan {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0560]: struct `plan::Plan` has no field named `turns`
    --> crates/athena-core/src/plan.rs:2079:13
     |
2079 |             turns: Vec::new(),
     |             ^^^^^ `plan::Plan` does not have this field
     |
     = note: available fields are: `order`, `groups`, `refused`, `cyclic`, `demoted`

error[E0599]: no associated function or constant named `build_improving` found for struct `plan::Plan` in the current scope
    --> crates/athena-core/src/plan.rs:2090:42
     |
 410 | pub struct Plan {
     | --------------- associated function or constant `build_improving` not found for this struct
...
2090 |         let (mut broken, capped) = Plan::build_improving(
     |                                          ^^^^^^^^^^^^^^^ associated function or constant not found in `plan::Plan`
     |
note: if you're trying to build a new `plan::Plan` consider using one of the following associated functions:
      plan::Plan::build
      plan::Plan::build_with
    --> crates/athena-core/src/plan.rs:548:5
     |
 548 |     pub fn build(entries: &[PlanEntry]) -> Plan {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
...
 560 |     pub fn build_with(entries: &[PlanEntry], history: &[JobEnd]) -> Plan {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0425]: cannot find function `improvement_requests` in this scope
    --> crates/athena-core/src/steward.rs:2135:13
     |
2135 |             improvement_requests(&records),
     |             ^^^^^^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0422, E0425, E0433, E0560, E0599.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 42 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli improvement`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/plan.rs, crates/athena-core/src/steward.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0432]: unresolved imports `athena_core::plan::ImprovementTurn`, `athena_core::plan::RepeatPromotion`
    --> crates/athena-cli/src/display.rs:1908:43
     |
1908 |         use athena_core::plan::{Demotion, ImprovementTurn, Plan, Priority, RepeatPromotion};
     |                                           ^^^^^^^^^^^^^^^                  ^^^^^^^^^^^^^^^ no `RepeatPromotion` in `plan`
     |                                           |
     |                                           no `ImprovementTurn` in `plan`

error[E0422]: cannot find struct, variant or union type `ImprovementTaken` in module `crate::drain`
    --> crates/athena-cli/src/display.rs:1999:27
     |
1999 |             crate::drain::ImprovementTaken {
     |                           ^^^^^^^^^^^^^^^^ not found in `crate::drain`

error[E0422]: cannot find struct, variant or union type `ImprovementTaken` in module `crate::drain`
    --> crates/athena-cli/src/display.rs:2005:27
     |
2005 |             crate::drain::ImprovementTaken {
     |                           ^^^^^^^^^^^^^^^^ not found in `crate::drain`

error[E0560]: struct `drain::Report` has no field named `improvement_turns`
    --> crates/athena-cli/src/display.rs:1902:13
     |
1902 |             improvement_turns: Vec::new(),
     |             ^^^^^^^^^^^^^^^^^ `drain::Report` does not have this field
     |
     = note: all struct fields are already assigned

error[E0609]: no field `turns` on type `athena_core::plan::Plan`
    --> crates/athena-cli/src/display.rs:1938:14
     |
1938 |         plan.turns = vec![ImprovementTurn {
     |              ^^^^^ unknown field
     |
     = note: available fields are: `order`, `groups`, `refused`, `cyclic`, `demoted`

error[E0609]: no field `promoted` on type `athena_core::plan::Plan`
    --> crates/athena-cli/src/display.rs:1943:14
     |
1943 |         plan.promoted = vec![RepeatPromotion {
     |              ^^^^^^^^ unknown field
     |
     = note: available fields are: `order`, `groups`, `refused`, `cyclic`, `demoted`

error[E0609]: no field `improvement_turns` on type `drain::Report`
    --> crates/athena-cli/src/display.rs:1998:16
     |
1998 |         report.improvement_turns = vec![
     |                ^^^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `stop`, `jobs`, `spent_usd`, `budget_usd`, `target_rating` ... and 15 others

Some errors have detailed explanations: E0422, E0432, E0560, E0609.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-cli` (lib test) due to 7 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core improvement_turn_skips`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`git diff CHANGELOG.md`). Result: not run: not a plain cargo test command.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`git status`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
