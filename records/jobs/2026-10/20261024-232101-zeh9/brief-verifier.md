# Job 20261024-232101-zeh9: verify "Order athena plan by request priority before age"

## Request (verbatim)

> athena plan orders requests only by dependencies and age, so a drain works the oldest requests first however much a newer one matters: on 2026-10-24 the records-history cutover (D111 job 7) sat behind about 30 older requests. Let a request carry a priority (for example a line 'Priority: high' in its words, or a field set by a command) that plan orders by before age, and show it in plan and status.
> Acceptance:
> - A test shows a high-priority request ordered before older normal ones while dependencies are still respected.
> - athena plan and athena status show each request's priority.
> - A request without one keeps today's order.
> Files: crates/athena-core/src/plan.rs, crates/athena-cli/src/display.rs, CHANGELOG.md, docs/decisions.md

## Reading (accepted by the user)

Add a priority (high/normal/low) to each plan entry, read from a "Priority: high" line in the request's words. Plan orders ready items by priority, then rung, then age, never ahead of dependencies. Priority is shown in `athena plan` and `athena status`. The audience is the owner who drains the queue and needs urgent work to run first.

Assumptions:
- Priority is a closed set: high, normal (default), low. High sorts first, low last.
- Priority is parsed from a line 'Priority: <level>' in the request text; no new CLI command is added, since the request offered the words as one option.
- Priority is the primary sort key among ready items; dependency order stays strictly first, and the sort is stable, so equal priorities keep today's rung-then-age order.
- The status view needs the priority too, so a small change outside the listed files (status data and reading.rs plan_entry) is likely.
- Existing PlanEntry struct literals in tests use ..PlanEntry::bare so a new field defaults cleanly.

## Acceptance criteria

1. [behavior] A plan.rs unit test builds a newer high-priority entry and several older normal ones, and shows the high one ordered first. Run `cargo test -p athena-core plan`.
2. [behavior] A test shows a high-priority entry that depends on an older normal one still comes after it, so dependencies win over priority. Run `cargo test -p athena-core plan`.
3. [behavior] A test shows that entries with no priority keep the existing order (rung, then age). The existing plan tests pass unchanged. Run `cargo test`.
4. [behavior] A display test shows `athena plan` output has a priority line per item, and the `athena status` request lines show priority. Run `cargo test -p athena-cli`.
5. [behavior] A test shows a request whose text has a line 'Priority: high' (case-insensitive) is read as high priority, and a request without such a line as normal.
6. [content] The plan.rs module doc names the new ordering key: priority, then rung, then age, with dependencies first.
7. [content] CHANGELOG.md has an entry for request priority. docs/decisions.md records the choice, with alternatives considered (words-in-request versus a command-set field) and the cutover motivation.
8. [behavior] `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.

## Expected deliverables

- crates/athena-core/src/plan.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- A CLI command to set priority after submission
- Changing how rungs or dependencies are inferred
- Changing the parallel grouping logic
- Reordering already-running jobs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for ordering and parsing; cargo test and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the observed order and printed output, not that the field exists.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: decisions.md entry records alternatives.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes in Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change, no new command.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in plan.rs: n1,n2,n3 normal (older) plus newer high h, same rung, no dependencies; assert Plan::build order. Run: `cargo test -p athena-core plan`. Observes: order == [h, n1, n2, n3].
  Fails without the behavior: Without the priority key the newer high entry is last and the order assertion fails.
- Criterion 2: Unit test: h high depends_on n1 (older normal), plus unrelated older normal n2 and n3; assert order is exactly [n1, h, n2, n3] (n1 before h, and h before n2 because the prerequisite inherits the priority). A second case: low-priority item with a high dependent is pulled before normals. Run: `cargo test -p athena-core plan`. Observes: n1 precedes h always, and h precedes the unrelated older normals.
  Fails without the behavior: If priority overrode dependencies h would precede n1; if priority applied only within a wave, h would land after n2 and n3 and the exact-order assertion fails.
- Criterion 3: New plan.rs test with no priorities, varied rungs and ages, asserting rung-then-age order and groups equal to the existing behavior; existing plan and drain tests run unchanged. Run: `cargo test`. Observes: Order is rung then age; all tests pass.
  Fails without the behavior: If the default order changed, the no-priority test and the existing tests fail.
- Criterion 4: drain.rs plan display test with a high and a normal entry asserts a 'priority' line for each item; display.rs test builds JobLines with high and normal priority and asserts the high line contains 'priority high' and the normal line has no priority suffix (absence means normal). Run: `cargo test -p athena-cli`. Observes: Plan output has a priority line per item; status request lines show non-normal priority.
  Fails without the behavior: Without the display change the priority needles are absent and assertions fail.
- Criterion 5: Tests for Priority::from_text ('Priority: high', 'priority: HIGH', indented, absent, 'Priority: urgent', prose mentioning priority) and reading::plan_entry on a request with the line vs without. Run: `cargo test -p athena-core`. Observes: High for the line in any case; Normal otherwise.
  Fails without the behavior: Without the parser every request reads as normal and the high assertions fail.
- Criterion 6: Read the module doc diff. Run: `git diff crates/athena-core/src/plan.rs`. Observes: The Order bullet names priority, then rung, then age, with dependencies first, and explains the inherited priority and the best-priority wave.
- Criterion 7: Review the diff. Run: `git diff CHANGELOG.md docs/decisions.md`. Observes: A CHANGELOG entry for request priority; a decisions.md entry with words-in-request versus command-field alternatives, the wave-only alternative, and the 2026-10-24 cutover motivation.
- Criterion 8: Run clippy with warnings denied and the format check over all code including new tests and field. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: No warnings; then `cargo fmt --all --check` also exits 0 with no diff.
  Fails without the behavior: An unused item, an unformatted line or a missed Priority field in a struct literal would make clippy or fmt exit non-zero.

Planned files:

- crates/athena-core/src/plan.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## The change

Base 6d300c84e1fe → candidate 3db40cbb2aaa. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |   6 ++
 crates/athena-cli/src/display.rs  |  34 ++++++++-
 crates/athena-cli/src/drain.rs    |   5 +-
 crates/athena-cli/src/main.rs     |   3 +-
 crates/athena-cli/tests/drain.rs  |  16 ++--
 crates/athena-core/src/plan.rs    | 155 +++++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/reading.rs |  17 +++++
 crates/athena-core/src/status.rs  |   5 ++
 docs/decisions.md                 |  35 +++++++++
 9 files changed, 263 insertions(+), 13 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1165 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
