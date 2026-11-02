# Job 20261031-184832-4gah: verify "Pass over queued items on a paused line instead of failing them"

## Request (verbatim)

> Priority: high
> Skip, do not fail, a queued item whose line the scope guard has paused
> 
> On 2026-10-31 three drains stopped on items of the paused line 20261030-060751-5389 (gdgv, then 7ym7, then 487a): each item ended 'no outcome · not attempted · line ... is paused' twice, which counts as a repeated failure and stops the drain before any other work.
> 
> An item on a paused line is not a failure of that item. In athena drain, pass over it before reading it, print one line naming the line and how to lift the pause, and go on to the next item; it must not count toward the repeated-failure stop. athena plan should list such items as held, with the same reason.
> 
> Acceptance:
> - A drain test with a pause file for a line and a queued item on that line shows the item is passed over with one printed line, is not read, does not count as a failure, and the next item runs.
> - A plan test shows the item listed as held with the pause reason.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.

## Reading (accepted by the user)

`athena drain` passes over a queued item whose line the scope guard has paused (before reading it), prints one line naming the line and how to lift the pause, and runs the next item without counting a failure. `athena plan` lists such items as held with the same reason. The audience is whoever runs drains; it is the follow-up to job 20261030-060751-5389, whose work is unrelated and dropped.

Assumptions:
- Code survey: crates/athena-cli/src/drain.rs: `Drain::attempts` (about line 1642) runs `run_job`, and a paused line reaches it as a not-attempted result that `failures += 1` counts toward `ATTEMPTS`, so `ItemEnd::Failed` and `Stop::RepeatedFailure` follow. `ItemEnd::Held` and `ItemEnd::Skipped` are the existing non-failure ends, and `pass_over` (about line 2214) prints the one-line notes. `plan(steps, ...)` at about line 2748 builds `PlanReport`. crates/athena-core/src/scope_guard.rs: `Athena::line_pause(records, job_id)` returns the `LineMark` while the pause still holds, `pause_refusal` gives the text with the lift instruction and pause file, and `pause_path` gives the file. No record or data file is added under crates/, so `check_record_file`, `AnyRecord` and the round-trip test are not touched. The property is measured directly: the number of `read` calls and failure counts on a fake `Steps`, and the printed notes, in crates/athena-cli/tests/drain.rs and the drain tests module.
- Job 20261030-060751-5389 is a failed root-cause job on reading references. Its deliverables (reading.rs, CHANGELOG.md, docs/decisions.md) concern a different cause. Its work is dropped, not kept: this change does not touch reading.rs or docs/decisions.md, and it has no candidate files to keep. Its findings (Part 3 false positives, symbol-shape rule, crate-relative paths) do not apply to this request. The two jobs only share the paused line.
- The pause check is a new Steps method, or an extension of an existing one, that the real implementation backs with `line_pause`. The fake Steps used in tests lets a test mark a line paused.
- The passed-over item ends as a new non-failure ItemEnd, or the existing Held/Skipped path. The drain's final report lists it as passed over, with the same reason shown for `athena plan`.
- This adds no new refusal of readings, so no newly-refused-readings percentage applies. It removes a failure count; the drain still stops on a real repeated failure of any other item.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli drain`. A new drain test queues an item on a line with a pause file, then a second ordinary item. It asserts that the paused item is not read (zero read calls for it), that exactly one printed line names the line and how to lift the pause, that no failure is counted and the stop is not `repeated-failure`, and that the next item runs and lands.
2. [behavior] Run `cargo test -p athena-cli drain`. A new drain test queues the same paused-line item three times, followed by an ordinary item. It asserts that the drain does not stop on repeated failure and that the ordinary item still runs.
3. [behavior] Run `cargo test -p athena-cli plan`. A new plan test with a pause file for a line shows its queued item listed as held, with the pause reason and lift instruction from `pause_refusal`. The same test shows a queued item on an unpaused line is not listed as held.
4. [content] CHANGELOG.md has an entry that says drain passes over items on a paused line, prints one line, does not count them as failures, and plan lists them as held.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass.
6. [content] A stale pause (one `line_pause` no longer reports) does not hold an item. Pass-over uses `Athena::line_pause`, not just the existence of the pause file, as shown by the diff in the real Steps implementation.
7. [content] Job 20261030-060751-5389's candidate files and deliverables (reading.rs, docs/decisions.md) are unchanged in the diff.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing crates/athena-core/src/reading.rs or the reading-reference check of job 20261030-060751-5389; that job's work is dropped
- Changing how the scope guard decides to pause or lift a line
- Changing athena.toml, defaults.toml, or planner and critic prompts
- Changing how `athena go` or other commands handle a paused line

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New drain and plan tests; fmt, clippy -D warnings and test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The paused line is stopped before it is read and counted as a non-failure end, rather than hiding the failure in the stop rule.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that check reads, printed lines, failure counts and the next item running.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is that a drain no longer stops on a paused line and does other work.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria name commands the verifier can run.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced in code and tests, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in drain and plan.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test in crates/athena-cli/tests/drain.rs with `drain` in its name. It builds a `Stub` with items a (on the paused line) and b, `outcomes: passing()` and `paused: [("a", reason)]`, where the reason text holds the line id, a lift step and a pause file name. It runs `drain::drain(&stub, limits(9.0), &rec)`, where `rec` is a recording `Ui`. Each recorded line is stored with the number of `run ` entries in `stub.log()` at the time it was printed. It then renders `display::drain(&report)`. Run: `cargo test -p athena-cli drain`. Observes: `stub.log()` has no `read a` and no `run a`. Exactly one line across the recorded Ui lines and the rendered report contains the line id and the lift text. That line is `plan       a passed over: …`, and it was recorded before `run b`. `report.notes` has no `item a is not worked` note, and `report.refused` has no `a`. `report.stop` is `Stop::QueueEmpty` and `report.failed_item` is None. `stub.log()` has `read b`, `run b` and `land b`. There is no job line for a.
  Fails without the behavior: Without the pause check, `Stub::read` logs `read a`. `a` ends `Failed` after two attempts, the stop is `RepeatedFailure` with `failed_item` a, and `b` never runs. With the hold but without the filter, the reason appears three times and the count assertion fails.
  Test seam: Production entry point is `drain::drain`, which runs `Drain::prepare`, `drain_with` and `Drain::attempts`. The fake is the existing `Stub` in tests/drain.rs (struct at line 40, `impl Steps` at line 105, `limits` at line 270). Step 5 adds its `paused` field and a `Steps::paused` override that returns only what the test configured. Step 1 adds the trait method, which has a default. Step 5 also adds the recording Ui. If `Ui` requires `Sync`, the recorder keeps its lines in a `Mutex`.
- Criterion 2: Two new tests in tests/drain.rs with `drain` in their names, both calling `drain::drain`. (a) Items a1, a2 and a3 are all paused, with plan entries, then an ordinary item c. `a2` depends on `a1`, which covers the dependents rule. (b) A mid-run case: the `Stub` gets `pause_on_run: Option<(job, item)>`. Item x has no plan entry. Its first run is rejected, and the stub then marks x paused, as the watcher would. The `Steps::paused` override reads that live set. Run: `cargo test -p athena-cli drain`. Observes: (a) The stop is not `RepeatedFailure`. There is no `read` for a1, a2 or a3. Each has one `passed over` line, and `c` is read, run and landed. (b) There is exactly one `run x`, no `reread x`, no `RepeatedFailure`, one `passed over` line for x, and `c` runs and lands.
  Fails without the behavior: Without the hold and the guard, the first paused item reaches `attempts`, ends `Failed`, and stops the drain with `RepeatedFailure`, so `read c` and `run c` never occur. In (b), without `ItemEnd::Paused`, x is retried and fails twice.
  Test seam: `drain::drain` is the entry point. The existing `Stub` gains the `paused` and `pause_on_run` fields in step 5. `Steps::paused` is added in step 1. Production code decides what to do with the answer, and the stub only reports which lines are paused.
- Criterion 3: New tests in tests/drain.rs with `plan` in their names, calling `drain::plan(&stub, 5.0, &Quiet)` and `display::plan(&report)`. The first uses a `Stub` with plan entries for paused a, ordinary b, and c that depends on a. It has `paused: [("a", reason)]`, where the reason is the text a real `pause_refusal` returns. The second uses `RealSteps`, built on the existing `Fixture` and `Scenario` in tests/common as the other RealSteps tests in the file are. A pause is written with `pause_path` and a `LineMark`, or scripted with two rejections of one criterion if `Scenario` allows. The test checks that `drain::plan` through `RealSteps` lists the item as held with `athena.pause_refusal(&mark)` as its reason. Run: `cargo test -p athena-cli plan`. Observes: `report.plan.refused` has a, with empty `missing` and the stub's reason (line, lift step and pause file). It also has c, with the reason "depends on a, which is held back…". `report.plan.order` and the groups hold b and not a or c. The rendered text contains `held        a · <title>: <reason>` and a `held` line for c, and none for b. `stub.log()` has no `read a`. The item on the unpaused line is planned and not listed as held.
  Fails without the behavior: Without the hold in `prepare`, a stays in `plan.order` and in the groups, `plan.refused` is empty, and there is no `held` line.
  Test seam: `drain::plan(&dyn Steps, f64, &dyn Ui)` and `display::plan`, both existing. The fakes are the `Stub` with the `paused` field from step 5 and, for the real path, the existing `RealSteps` with `Fixture` and `Scenario`. Step 3 adds `RealSteps::paused`.
- Criterion 4: Add a bullet under `## [Unreleased]`, `### Fixed` in CHANGELOG.md, after reading how the neighbouring entries are written. It says that `athena drain` passes over a queued item whose line the scope guard has paused, before reading it. It prints one line naming the line and how to lift the pause. The item is not a failed attempt, so it no longer ends the drain with `repeated failure`. `athena plan` lists such items, and items that depend on them, as `held` with the same reason. Run: `git diff -- CHANGELOG.md`. Observes: The diff shows the entry, and it names all four points: pass over, one printed line, not a failure, and plan lists it as held.
- Criterion 5: Run the three Athena checks. `Steps::paused` has a default, so the other `Steps` fakes in tests/ and in drain.rs's own test module keep compiling. The only new enum variant is `ItemEnd::Paused`, and `finish` handles it. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three commands exit 0.
  Fails without the behavior: A clippy warning (such as an unused value or a match that is not exhaustive) or any regression in the existing drain tests, `batches_for` tests or budget tests fails this.
  Test seam: Existing check commands. No new seam.
- Criterion 6: New `RealSteps` test in tests/drain.rs with `drain` in its name, using the existing `Fixture` and `Scenario`. Write a `LineMark` with level pause at `athena_core::scope_guard::pause_path(&athena.state, first)` for a line whose records do not reach a pause, so the stored pause is stale. If `Scenario` can script two rejections of different criteria (a flag), the line has real attempts that yield a flag and not a pause, as `a_stored_pause_that_no_longer_holds_is_not_returned_and_does_not_block_a_retry` does in scope_guard.rs. The test comment says which of the two cases it covers. A companion case scripts a real pause and asserts that `steps.paused(&[job])` returns `pause_refusal`'s text. The diff of `RealSteps::paused` is read to confirm it calls `line_pause` and `pause_refusal` and does not call `Path::exists`. Run: `cargo test -p athena-cli drain`. Observes: `steps.paused(&[job])` is empty for the stale pause file, which exists on disk, and non-empty for a pause that still holds.
  Fails without the behavior: If `RealSteps::paused` tested only that the pause file exists, the stale case would return a reason and the assertion would fail.
  Test seam: `RealSteps::paused`, added by step 3. The fixture and scenario helpers exist in tests/common. `LineMark` and `pause_path` are public in athena_core::scope_guard.
- Criterion 7: List the changed paths after the work. Run: `git diff --stat`. Observes: Only crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs and CHANGELOG.md appear. crates/athena-core/src/reading.rs and docs/decisions.md do not.

Planned files:

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 0eba7ea2930a → candidate be126fdb0a05. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  10 ++
 crates/athena-cli/src/drain.rs   | 130 +++++++++++++-
 crates/athena-cli/tests/drain.rs | 372 +++++++++++++++++++++++++++++++++++++++
 3 files changed, 508 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41384478389 for be126fdb0a052ddaea2a4772b5f0d33465b593f7)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41384478389 for be126fdb0a052ddaea2a4772b5f0d33465b593f7)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41384478389 for be126fdb0a052ddaea2a4772b5f0d33465b593f7)
- ci-result: `not a check` passed (Check results for be126fdb0a052ddaea2a4772b5f0d33465b593f7 come from the CI run https://github.com/doodla/athena/actions/runs/41384478389 (586s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
