# Job 20261103-231510-20z0: verify "Pass over an item that fails twice while flagged, don't stop batch"

## Request (verbatim)

> Extend the landed fix 'Don't stop a batch on repeated failure of an item already paused' to items the scope guard has only flagged: after an item fails twice, pause it and go on to the next item instead of stopping the batch. At 11:13pm UTC on 3 Nov a batch stopped with money and slots left because item 20261103-193428-df2m was flagged (T1, T3, T5), not paused. Acceptance: a test where an item fails twice and is flagged, and the batch then runs the next item.

## Reading (accepted by the user)

Extends the earlier fix (a batch no longer stops on repeated failure of an item whose line the scope guard paused) to lines the guard only flagged (T1/T3/T5), as for item 20261103-193428-df2m. After two failed attempts on a flagged line, the drain passes the item over with a reason and runs the next item; the audience is the operator running `athena drain` batches. Previous job 20261103-193428-df2m (a different fix, job-slot counting) is related by id only, so its work is not carried here.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (Drain::attempts around lines 1735-1830, where failures >= ATTEMPTS returns ItemEnd::Failed unless paused_reason finds a pause; Steps::paused at ~650; RealSteps::paused at ~3339) and crates/athena-core/src/scope_guard.rs (Athena::line_pause at 782 returns only Level::Pause marks; live_marks at 796 also holds flags). The cause: line_pause ignores flags, so a flagged line that fails twice yields ItemEnd::Failed and the repeated-failure stop. No record or data file is added, so the records allow-list is untouched; the property is measured by driving Drain with stub Steps in crates/athena-cli/tests/drain.rs.
- Reading of 'pause it': after the second failure on a flagged line the drain passes the item over for the rest of the batch (as ItemEnd::Paused does today), with a note naming the flag signals; it does not write a new pause file. Simplest route: a new defaulted Steps method (e.g. flagged) backed by Athena::live_marks, consulted at the ATTEMPTS point alongside paused_reason.
- The earlier landed fix's tests in tests/drain.rs for paused lines stay passing; an unflagged, unpaused item failing twice still stops the batch as 'repeated failure'.
- Job 20261103-193428-df2m is only the example incident (the item flagged T1, T3, T5); its own verification findings (uncited command evidence for a behavior criterion) concern a different change and are addressed here by naming runnable commands in each behavior criterion.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs: an item fails twice on a line that is flagged (not paused), and the batch then runs the next item, which is recorded as run/landed, with no 'repeated failure' stop. Run `cargo test -p athena-cli --test drain` and see it pass.
2. [behavior] A test shows an item that fails twice with no flag and no pause still ends the batch with the repeated-failure stop naming it. Run `cargo test -p athena-cli --test drain`.
3. [behavior] The passed-over flagged item is reported with a line naming it and its flag reason, and the batch summary does not count it as a failed item stopping the run. Checked by the new test's assertions on output under `cargo test -p athena-cli --test drain`.
4. [behavior] Existing paused-line drain tests still pass; `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` are clean.
5. [content] drain.rs (Drain::attempts, the stop-reasons text and the module docs near line 26 and 944) says a line that is flagged or paused is passed over after two failures; Grep for 'flagged' in crates/athena-cli/src/drain.rs shows the new rule documented where the old paused rule is.
6. [content] CHANGELOG.md has an entry stating that a batch no longer stops on repeated failure of a flagged item, citing the 3 Nov incident with item 20261103-193428-df2m.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- Job 20261103-193428-df2m's change (re-read 'needs accepting' not counting toward --job-limit): not kept or redone here; its files drain.rs, tests/drain.rs, main.rs and CHANGELOG.md are only touched here for this request's change, and its 'needs accepting' slot logic is not part of this job.
- Changing the scope guard's T1-T5 thresholds or when a flag or pause is recorded.
- Changing the stop on repeated failure for items whose line is neither flagged nor paused.
- Writing or lifting pause files on disk beyond what the existing watcher already does.
- Changing athena.toml, defaults.toml or workflow files.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change and test in Rust under crates/athena-cli.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt --check must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: A drain test exercises two failures on a flagged line then the next item running.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The stop decision at the ATTEMPTS failure point in Drain::attempts is changed for flagged lines, not just this item.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in drain.rs and its tests, plus a changelog line.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test using the `Stub` Steps double. `Stub` gets a new field `flagged: Vec<(&'static str, &'static str)>` and a `Steps::flagged` impl that returns the configured pairs whose job is in the asked list. Items `a`, `b`; outcomes Rejected, Rejected, Landed; `flagged: vec![("a", flag_text("a"))]` and no pause. It calls `drain::drain(&stub, limits(9.0), &ui)` with the `Stamped` Ui. Run: `cargo test -p athena-cli --test drain drain_passes_over_an_item_that_fails_twice_on_a_flagged_line_and_runs_the_next_item`. Observes: `stub.log()` equals `[read a, run a, reread a, run a-retry, read b, run b, land b, rate]`. `report.stop` is `QueueEmpty` and not `RepeatedFailure`. `report.failed_item` is `None`. `report.landed` is 1. `report.jobs` holds `b` as run and landed.
  Fails without the behavior: Without the `flagged` consult at the ATTEMPTS point, `a` ends as `ItemEnd::Failed`, `report.stop` is `Stop::RepeatedFailure`, `report.failed_item` is `Some("a")`, and the log never contains `read b`/`run b`/`land b`, so the test fails.
  Test seam: Production entry point `drain::drain` (crates/athena-cli/src/drain.rs) reaches `Drain::attempts`. The fake is the existing `Stub` in tests/drain.rs, injected as `&dyn Steps` argument. The new `flagged` field and method on `Stub` are added in step 6; the `Steps::flagged` trait method is added in step 3. The stub's `flagged` only returns configured data; the decision logic stays in `Drain::attempts`.
- Criterion 2: The existing test `the_same_item_failing_twice_is_a_repeated_failure_naming_the_item` (tests/drain.rs:1160) is kept; it uses a `Stub` with no `paused` and no `flagged`. A new twin, `drain_still_stops_on_repeated_failure_when_another_item_is_flagged_but_this_one_is_not`, has items `a`, `b`, `flagged: [("b", ..)]`, and both of `a`'s attempts rejected. Run: `cargo test -p athena-cli --test drain the_same_item_failing_twice_is_a_repeated_failure_naming_the_item`. Observes: `report.stop == Stop::RepeatedFailure`, `failed_item == Some("a")`, the log ends at `run a-retry`, `b` is not started, and `display::drain` contains `repeated failure: item a failed twice`. The twin shows a flag on another item does not suppress the stop.
  Fails without the behavior: If the new check passed over every twice-failed item instead of only flagged or paused ones, this test and the new unflagged test would see `Stop::QueueEmpty`, no failed item and `b` started, so they would fail.
  Test seam: `drain::drain` with the existing `Stub`; no new seam beyond the `flagged` field added in step 6. `Stub::flagged` returns only entries for jobs asked about, as for `paused`.
- Criterion 3: The same new test as criterion 1 asserts on output. The `Stamped` Ui collects each printed line with the number of runs started so far; the test also calls `display::drain(&report)`. Run: `cargo test -p athena-cli --test drain drain_passes_over_an_item_that_fails_twice_on_a_flagged_line_and_runs_the_next_item`. Observes: Exactly one line starts `plan       a passed over: line a is flagged (T1, T3, T5)` and contains the stub's flag reason text, and it was printed after 2 runs and before `b` ran. `report.failed_item` is `None`. `display::drain` text has no `repeated failure` and no `failed twice`. `report.queued` still lists `a`.
  Fails without the behavior: Without `ItemEnd::Flagged` handled by `pass_over` in `finish`, no `passed over` line naming `a` and its flag text is printed, and with the old code the stop is `RepeatedFailure`, so the summary counts `a` as the failed item and the test fails.
  Test seam: `drain::drain` with `Stub` and `Stamped` Ui (both exist, tests/drain.rs:105 and 562). The flag text is built by a new test helper `flag_text(line)` beside `pause_text`, as the stub's configured reason. The production wording (built from `LineMark`) is checked by criterion 4's real-steps test.
- Criterion 4: Run the full suite and the lint and format checks. The paused-line tests (drain.rs tests at 606, 654, 691, 724, 776, 827, 859 and the real-steps one at 4493) are untouched. Two tests are added: a real-steps drain test (reusing `rejected_twice(rejecting_verdict(), rejecting_second_criterion())`, which produces a flagged line) and a `scope_guard` unit test for `line_flag` (holds while flagged, `None` for a paused or unflagged line, `None` once the records no longer reach a flag). Run: `cargo test -p athena-cli --test drain && cargo test -p athena-core scope_guard && cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`. Observes: All pass and clippy and fmt are clean. In the real-steps test: `report.failed_item` is `None`, `steps.flagged(&[job])` returns `[(job, text)]` with the text naming the line and the flag signals, and a following `drain` lists `plan       <job> passed over: line <job> is flagged`.
  Fails without the behavior: Without the updated test at tests/drain.rs:4534 the suite fails, because its `failed_item == Some(job)` assertion contradicts the new behavior. Without `line_flag` returning the mark, the new real-steps test `drain_real_steps_pass_over_the_item_of_a_line_that_is_flagged_after_two_failures` fails, because `RealSteps::flagged` returns nothing and the batch stops.
  Test seam: `RealSteps` built by `real_steps_with(&athena, &lander, &ui)`, with the fake agent `Scenario`/`phased` fixtures (all exist, tests/drain.rs:4474-4490) and `athena.state`. Production entry points: `drain::drain`, `RealSteps::flagged`, `Athena::line_flag`. The test double is only the fake agent; the flag file is written by the real watcher (`try_watch_line`).
- Criterion 5: Content check. The test run makes sure the docs edit compiled (doc links such as [`ItemEnd::Flagged`] and [`Steps::flagged`] resolve). Grep for `flagged` in drain.rs. Run: `cargo test -p athena-cli && grep -n flagged crates/athena-cli/src/drain.rs`. Observes: `flagged` appears in the module docs bullet next to the paused bullet (around lines 26-33), the `Drain::attempts` comment at the ATTEMPTS point, the `Stop::RepeatedFailure` doc (around 943-946), and `Stop::meaning` for RepeatedFailure (around 1004), each saying a line that is flagged or paused is passed over after two failures. The stop-reasons text in main.rs (line 254) says the same.
- Criterion 6: Content check on CHANGELOG.md. The new bullet goes in `### Fixed` under Unreleased, directly after the paused-line entry at line 170-175. Run: `grep -n 20261103-193428-df2m CHANGELOG.md`. Observes: One bullet saying a batch no longer stops on repeated failure of a flagged item (T1/T3/T5), naming the 3 Nov 11:13pm UTC incident with item 20261103-193428-df2m, the root cause (`line_pause` returns only pauses) and that an unflagged unpaused item that fails twice still stops the batch.

Planned files:

- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b91c2320aa14 → candidate 26e596186a81. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |   8 ++
 crates/athena-cli/src/drain.rs        |  63 +++++++++--
 crates/athena-cli/src/main.rs         |   3 +-
 crates/athena-cli/tests/drain.rs      | 191 +++++++++++++++++++++++++++++++++-
 crates/athena-core/src/scope_guard.rs |  66 +++++++++++-
 5 files changed, 318 insertions(+), 13 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41752872718 for 26e596186a8130f3e9df38b9709d06fed998b5a6)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41752872718 for 26e596186a8130f3e9df38b9709d06fed998b5a6)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41752872718 for 26e596186a8130f3e9df38b9709d06fed998b5a6)
- ci-result: `not a check` passed (Check results for 26e596186a8130f3e9df38b9709d06fed998b5a6 come from the CI run https://github.com/doodla/athena/actions/runs/41752872718 (610s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Existing paused-line drain tests still pass; `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
