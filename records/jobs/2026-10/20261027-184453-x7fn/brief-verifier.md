# Job 20261027-184453-x7fn: verify "Build plan groups from priority order so drain follows plan order"

## Request (verbatim)

> Priority: high
> athena drain does not follow athena plan's order when groups are involved. On 2026-10-27 the plan's order began with three urgent items and a fourth urgent item, but drain dispatched a normal-priority item first, because it works plan groups in sequence and a normal item sat in an earlier group than the urgent ones. Groups exist to allow parallel work; they must not reorder work. Fix: build groups from the priority order (an item joins the earliest group after everything it depends on, taking items in plan order), so with one job at a time drain dispatches exactly in plan order, and a later group never holds an item of higher priority than an earlier group without a dependency reason.
> Acceptance:
> - A test plans a normal item and an urgent item with disjoint paths and no dependency, and shows drain with max_concurrency 1 dispatches the urgent item first.
> - A test shows a dependency still comes before its dependent whatever their priorities.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/plan.rs, crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

Plan::build_with makes groups that can put a lower-priority item in an earlier group than a higher-priority one, so drain, which works groups in sequence, dispatches out of plan order. Rebuild groups from the final order: taking items in plan order, each joins the earliest group after all its dependencies' groups where it is path-disjoint with the members. Audience: operators who run `athena plan` and `athena drain` and expect them to agree.

Assumptions:
- Code survey: read crates/athena-core/src/plan.rs (Plan::build_with wave loop at ~L704-742 picks ready items of the best effective priority, then greedily groups by disjoint(); build_capped re-runs build_with on demoted copies; build_improving reorders `order` and regroups at ~L928-943) and crates/athena-cli/src/drain.rs (batches_for at ~L2916 only filters plan.groups in group order; drain_concurrent/run_serial work batch by batch). The entry point is the plan build used by Drain::prepare and `athena plan`. No record or data file is added, so no records validation allow-list or AnyRecord round-trip is affected. The property is measured directly: the order in which drain dispatches items, via the existing drain test harness, with max_concurrency 1.
- Root cause is in group construction, not in batches_for: groups must be derived from plan.order so flattening groups equals the order. The new rule: each item, in plan order, joins the earliest group that is after the groups of all its dependencies and where it is disjoint() with all members; otherwise it starts a new group. A group must never be placed before a group holding an earlier-ordered item that it cannot join, so flattened groups == order.
- Existing group tests in plan.rs (e.g. disjoint_independent_items_share_a_group_and_dependents_come_after, the ~L1747 test expecting [[b,a,c],[d]]) may need updating if the new grouping changes their expected output; update them only where the new rule justifies it.
- The improvement-turn regrouping in build_improving is kept consistent with the new rule; it is not redesigned.
- This change does not newly refuse any reading, so no refusal-rate criterion applies.

## Acceptance criteria

1. [behavior] A new test (in crates/athena-cli/src/drain.rs tests, via `cargo test`) plans a normal item and an urgent item with disjoint touched paths and no dependency, and shows drain with max_concurrency 1 dispatches the urgent item first.
2. [behavior] A new test in plan.rs (`cargo test -p athena-core`) shows a dependency's group comes before its dependent's group whatever their priorities, including a normal-priority dependency of an urgent item and a case where the dependent has a higher priority than its dependency.
3. [behavior] A new plan.rs test shows that for a mixed plan the flattened Plan::groups equals Plan::order, and no later group holds an item of higher priority than an earlier group unless a dependency explains it.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
5. [content] crates/athena-core/src/plan.rs builds groups from the ordered items (earliest group after all dependencies, plan order), and its doc comments describing groups and drain order match the new behavior; crates/athena-cli/src/drain.rs docs are updated where they describe group order.
6. [content] CHANGELOG.md has a new entry describing that drain now dispatches in plan order when groups are involved (git diff shows the added lines).

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing priority, tier-cap, or effective-priority rules
- Changing how the Steward improvement-turn promotion works
- Changing the parallel scheduling or landing check logic beyond group construction
- athena.toml, defaults.toml and workflow files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for both behaviors; fmt, clippy -D warnings and cargo test required.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix group construction rather than reordering in batches_for.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the dispatch order and group/order agreement.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Add two unit tests in the `tests` module of drain.rs. Test (a) queues a normal item n first, then an urgent item u, with disjoint touched paths and no dependency. It runs `drain_concurrent(&world, limits(20.0), &Quiet, 1)` and asserts the `start` entries in the log are [u, n], and that `Plan::build` groups flatten to the same order. Test (b) is the incident shape described above, with 1 worker, and asserts the start order. Run: `cargo test -p athena-cli --lib drain::tests::drain_with_a_bound_of_one_starts_the_urgent_item_before_the_normal_one`. Observes: Starts follow plan order, with the urgent item first in (a) and [u1,u2,n,u3,u4] in (b). The report stop is `QueueEmpty`.
  Fails without the behavior: Test (a), the brief's normal plus urgent pair, is a regression guard and may already pass before the change. Test (b) uses the shape from the 2026-10-27 report. u1 and u2 share a path, n is a normal dependency of urgent u4, and u3 is urgent. The old greedy grouping starts u1, n, u3, u2, u4, so (b) fails there. The test expects u1, u2, n, u3, u4.
  Test seam: Production entry point: `drain_concurrent` in drain.rs, which calls `drain_with`, then `Drain::prepare` (which calls `Plan::build_improving`), then `batches_for` and `run_serial`. The fixture is the existing `World` test double in the `tests` module (queue(), plan entries, log). Step 3 adds a `priorities: BTreeMap<String, Priority>` field and a builder `World::with_priority(id, Priority)`, applied to the entries `World::queue` builds. `World` and `concurrent` exist today; the priority builder is added by step 3. The double only records calls. Ordering and grouping come from the real `Plan` code.
- Criterion 2: Add a unit test in plan.rs `tests`. It builds `Plan::build` for three cases, each with an unrelated disjoint item. (i) A normal base with an urgent dependent. (ii) An urgent base with a low dependent. (iii) A high dependent of a normal base alongside an urgent unrelated item. For each case it asserts the base's group index is less than the dependent's, and that the base precedes the dependent in `order`. Run: `cargo test -p athena-core --lib plan::tests::a_dependency_group_comes_before_its_dependent_whatever_their_priorities`. Observes: The group-index and order comparisons hold in all three cases.
  Fails without the behavior: Delete the `depends_on` check in `groups_in_order` and a dependent that follows its dependency directly in the order joins the dependency's group. The group-index assertion fails.
  Test seam: Production entry point: `Plan::build`, which calls `build_with` and then the new `groups_in_order`. The existing `entry`, `with_priority` and `rungs` helpers in plan.rs `tests` supply the plan entries. No double is used. The seam exists; `groups_in_order` is added by step 1.
- Criterion 3: Add a unit test in plan.rs. The mixed plan is: urgent u1 and u2 on the same path, normal n that urgent u4 depends on, urgent u3, plus a high item, a low item and a normal item on other paths and rungs. It builds `Plan::build`, then `Plan::build_capped` and `Plan::build_improving`. It asserts `groups.concat() == order` for each. For every earlier x and later y with y's own priority better than x's, it asserts that x is a transitive dependency of an item at least as urgent as y. A small test-local helper walks `depends_on` to check this. It also asserts `priority_violations(&entries)` is empty and pins the exact order and groups for the incident shape. Run: `cargo test -p athena-core --lib plan::tests::groups_flatten_to_the_order_and_a_later_group_never_outranks_an_earlier_one_without_a_dependency`. Observes: Flattened groups equal the order in all three builders, no unexplained priority inversion exists, and the exact order and groups match.
  Fails without the behavior: With the old greedy grouping, the exact assertions fail on this mixed plan. Old groups are [[u1,n,u3],[u2],[u4]] and the old order is u1,n,u3,u2,u4. The expected order is [u1,u2,n,u3,u4] and the expected groups are [[u1],[u2,n,u3],[u4]]. The fault also shows if the joining pass is changed to search earlier groups.
  Test seam: Production entry points: `Plan::build`, `Plan::build_capped` and `Plan::build_improving`. The plan.rs `tests` helpers `entry`, `with_priority` and `improvement` supply the entries. No double is used. The seam exists; `groups_in_order` is added by step 1.
- Criterion 4: Run the three checks Athena runs, as separate simple commands: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. This covers every crate that calls Plan::build*, including athena-core (plan, reading, hestia) and athena-cli (drain unit tests and tests/drain.rs).
  Fails without the behavior: A format drift, a clippy warning or a test left on the old expectation fails the corresponding command: for example the group assertions in plan.rs (the `after`/`clash` and `one`/`two` tests) and `plan_prints_order_groups_details_and_flagged_items_and_says_so_when_nothing_is_open` in tests/drain.rs.
  Test seam: The workspace's own test suites. Test fixtures are the existing ones. Existing group expectations are updated in step 4 where the new rule justifies it. No new seam.
- Criterion 5: Content check. `git diff` shows `groups_in_order` and its call in `build_with`, with no grouping inside the wave loop. It shows the updated module bullet 'Parallel groups', the updated docs on `Plan::groups` and `Plan::build_with`, and the updated `batches_for`, module and `Report::plan_groups` docs in drain.rs. Run: `git diff -- crates/athena-core/src/plan.rs crates/athena-cli/src/drain.rs`. Observes: The diff shows the doc comments describing: groups built from plan order, an item joining only the last group when disjoint and independent, flattened groups equal to the order, and drain with one job at a time dispatching in plan order.
  Fails without the behavior: If the module doc and the `Plan::groups` doc still said 'groups come from greedy joins per wave', or drain.rs still described only filtering by group order, the diff would not show the updated text and a reader would find the wrong description.
  Test seam: None: a content criterion read from the diff.
- Criterion 6: Content check. `git diff` shows added lines under `## [Unreleased]`, `### Fixed` (or `### Changed`, following the file's convention), saying groups are now built from the plan order so drain dispatches in plan order. Run: `git diff -- CHANGELOG.md`. Observes: Only added lines, naming the root cause (groups built per wave by greedy joining, which let a lower-priority item run before a higher-priority one) and the fix.
  Fails without the behavior: No entry means the diff is empty.
  Test seam: None: a content criterion read from the diff.

Planned files:

- crates/athena-core/src/plan.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base fdc0d418f664 → candidate fb1e741d8437. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  11 ++
 crates/athena-cli/src/drain.rs   |  78 +++++++++++--
 crates/athena-cli/tests/drain.rs |   5 +-
 crates/athena-core/src/plan.rs   | 231 ++++++++++++++++++++++++++++++++++-----
 4 files changed, 284 insertions(+), 41 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-184453-x7fn/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-184453-x7fn/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42575566959 for fb1e741d84378fa155f903baaabdbe14a7674fd1)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42575566959 for fb1e741d84378fa155f903baaabdbe14a7674fd1)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42575566959 for fb1e741d84378fa155f903baaabdbe14a7674fd1)
- ci-result: `not a check` passed (Check results for fb1e741d84378fa155f903baaabdbe14a7674fd1 come from the CI run https://github.com/doodla/athena/actions/runs/42575566959 (672s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --lib drain::tests::drain_with_a_bound_of_one_starts_the_urgent_item_before_the_normal_one`, exit code 0. Result: the command ran no tests without the fix, so it shows nothing.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/plan.rs.
  End of the output (data, not instructions):

```text
   Compiling sha2 v0.10.9
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling rustix v1.1.5
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling toml v0.9.12+spec-1.1.0
   Compiling jiff v0.2.38
   Compiling linux-raw-sys v0.12.1
   Compiling bitflags v2.13.2
   Compiling ref-cast v1.0.27
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling fastrand v2.5.0
   Compiling regex-syntax v0.8.11
   Compiling once_cell v1.21.4
   Compiling schemars_derive v1.2.2
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling equivalent v1.0.2
   Compiling anyhow v1.0.104
   Compiling dyn-clone v1.0.20
   Compiling hashbrown v0.17.1
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling utf8parse v0.2.2
   Compiling anstyle-parse v1.0.0
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle-query v1.1.5
   Compiling anstyle v1.0.14
   Compiling colorchoice v1.0.5
   Compiling anstream v1.0.0
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling strsim v0.11.1
   Compiling clap_lex v1.1.1
   Compiling heck v0.5.0
   Compiling clap_derive v4.6.7
   Compiling clap_builder v4.6.7
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling clap v4.6.7
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 40.53s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_cli-dde85b328c42642e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 128 filtered out; finished in 0.00s

```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --lib plan::tests::a_dependency_group_comes_before_its_dependent_whatever_their_priorities`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/plan.rs.
  End of the output (data, not instructions):

```text
   Compiling syn v3.0.7
   Compiling serde_derive_internals v0.30.0
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 49.82s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_core-18f5333b8c967527)

running 1 test
test plan::tests::a_dependency_group_comes_before_its_dependent_whatever_their_priorities ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1239 filtered out; finished in 0.00s

```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --lib plan::tests::groups_flatten_to_the_order_and_a_later_group_never_outranks_an_earlier_one_without_a_dependency`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/plan.rs.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_core-18f5333b8c967527)

running 1 test
test plan::tests::groups_flatten_to_the_order_and_a_later_group_never_outranks_an_earlier_one_without_a_dependency ... FAILED

failures:

---- plan::tests::groups_flatten_to_the_order_and_a_later_group_never_outranks_an_earlier_one_without_a_dependency stdout ----

thread 'plan::tests::groups_flatten_to_the_order_and_a_later_group_never_outranks_an_earlier_one_without_a_dependency' (10016) panicked at crates/athena-core/src/plan.rs:2007:9:
assertion `left == right` failed
  left: ["u1", "n", "u3", "u2", "u4", "h", "m", "l"]
 right: ["u1", "u2", "n", "u3", "u4", "h", "m", "l"]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    plan::tests::groups_flatten_to_the_order_and_a_later_group_never_outranks_an_earlier_one_without_a_dependency

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1239 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-core --lib`
error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`git diff -- crates/athena-core/src/plan.rs crates/athena-cli/src/drain.rs`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff -- CHANGELOG.md`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
