# Job 20261026-024220-94p9: verify "Build scope-guard running-line watcher (job 5): flag, pause, refuse"

## Request (verbatim)

> try again on 20261026-015419-4j37

## Reading (accepted by the user)

Retry of the failed watcher job. After each attempt of a line, evaluate T1–T5 from scope_guard.rs, record flags and pause records, show them in athena status, and make dispatch and Athena::retry refuse a paused line. Audience is the owner running Athena. Demonstrations use states the code accepts, per the user's two decisions.

Assumptions:
- Pauses only stop the next dispatch; no running process is killed.
- T3 p90 is read from the job's reading forecasts (user decision).
- Demonstrations avoid calling go on a Dispatched job (user decision).
- reading.rs is edited for the Athena::retry refusal.

## Acceptance criteria

1. [behavior] A test with two rejections on the same unmet criterion, with the third request made via ask('try again on <id>') (not retry), shows the third attempt is not dispatched and a pause record exists; a control with no pause file dispatches. Run cargo test.
2. [behavior] A test with rejections on differing criteria records a flag only: no pause record, and the next attempt is dispatched. Run cargo test.
3. [behavior] A test shows Athena::retry on a paused line refuses with the pause reason, and dispatch finds the line head by rebuilding RequestRecords and calling group_lines, then looking up the pause file by the line's first id. Run cargo test.
4. [behavior] A T3 test passes a real positive forecast p90 dollars read from the job's reading forecasts, pauses the line, and fails if the pause is removed. Run cargo test.
5. [behavior] A test registers a real child process (e.g. sleep) as a line's running job, drives the watcher to a pause on that line, and asserts the child is still alive and its registry entry unchanged. Run cargo test.
6. [behavior] A test shows athena status output lists flagged and paused lines with their signal (T1–T5). Run cargo test.
7. [content] A pause record names the signal, the numbers that tripped it, and what would lift it (a measurement, or a narrowed request for T4).
8. [content] CHANGELOG.md has an entry for the watcher; docs/design/scope-guard.md reflects job 5 and states that p90 comes from the reading's forecasts and is used by t3_cost.
9. [behavior] cargo clippy is clean, cargo fmt --check passes, and cargo test passes.

## Expected deliverables

- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/design/scope-guard.md (modify)

## Out of scope

- Changing T1–T5 thresholds or rules
- Killing or interrupting running jobs
- Lifting pauses automatically

## User defaults that apply

- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing refusal, with controls that dispatch without the pause.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt --check criterion.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the listed files.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check lines being stopped, not code existing.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Test in scope_guard.rs using the existing fake-driver harness (CutoverFixture::athena with Script, or run_pin_tests.rs). Produce the first and second rejections on the same unmet criterion text through `athena.go(..)`, with no direct `watch_line` call. Assert the pause file exists once the second `go` returns. Then call `athena.ask("try again on <id2>")` and `athena.go(third)`. Assert Err with the pause reason, no Job record for the third job, and the pause file still present. Control: delete the pause file, and `go` on the same ask returns Finished (dispatched). Run: `cargo test --locked scope_guard::tests::t4_pause_via_go_and_ask`. Observes: The pause appears only through the go hook. The third attempt is refused while the file exists and dispatched once it is gone.
  Fails without the behavior: Without the `watch_line` hook in `go`, no pause file exists after the second rejection and the pause assertion fails. Without the dispatch check, the third job is dispatched and the refusal assertion fails. If the control also refused, the control assertion fails.
- Criterion 2: Test in scope_guard.rs. Produce two rejections with different unmet criterion texts through `go`. Assert a flag file with a signal exists and no pause file exists. Then `ask` and `go` the next attempt and assert it is dispatched (Finished). Run: `cargo test --locked scope_guard::tests::differing_criteria_flag_only`. Observes: A flag file, no pause file, and the next attempt dispatched.
  Fails without the behavior: If differing criteria paused the line, the no-pause-file assertion fails. If no flag were written, the flag assertion fails. If dispatch refused, the dispatched assertion fails.
- Criterion 3: Test in scope_guard.rs. Build the line a→b, where b retries a, with both jobs rejected and reading scripted. Control first: with no pause file, `athena.retry("<b>")` returns Ok. Then write the pause file named by a's id and call `retry("<b>")` again. Assert Err whose text contains the pause signal and reason, and that the records tip is unchanged. Also assert `line_pause` for job b (rebuilt RequestRecords, then group_lines, then lookup by the first id) returns the pause written under a. Run: `cargo test --locked scope_guard::tests::retry_refuses_paused_line`. Observes: The control retry succeeds. The same retry with the pause file is refused with the pause reason and commits nothing.
  Fails without the behavior: Without the `line_pause` check in `retry`, the call reaches `ask` and returns Ok, so the Err assertion fails. If `line_pause` used the job's own id instead of the line's first id, the retry of b is not refused and the test fails.
- Criterion 4: Test in scope_guard.rs. The reading records carry `forecasts.reader.cost_usd` with a positive p90 (1.0). One rejected job spent 1.6, with a single attempt, so only T3 holds. Run it through `go` and assert the pause file names T3 with 1.6 and 1.0. Assert the next `go` is refused. Then remove the pause file and assert dispatch proceeds. A second assertion sets p90 to 0 (a reading with no usable forecast) and shows no T3 pause. Run: `cargo test --locked scope_guard::tests::t3_pause_from_reading_forecast`. Observes: The T3 pause comes from the reading's p90. Removing the file lets dispatch succeed.
  Fails without the behavior: If the pause were removed, the dispatch assertion fails. If p90 were not read from the reading, spend of 1.6 against p90 of 1.0 would give no T3 pause and the pause assertion fails.
- Criterion 5: Test in scope_guard.rs. Spawn `sleep 60` with `std::os::unix::process::CommandExt::process_group(0)` so the child leads its own group. Register `child.id() as i32` as the pgid in the registry (athena_agent::procs `Registry::register(pgid, job, what)`) for the line's running job. Snapshot the entries. Drive the watcher to a T4 pause on that line, through `go` on a different job of the line or through `watch_line`. Assert the pause file exists, `child.try_wait()` is Ok(None), and the registry entries equal the snapshot. A Drop guard kills and waits the child even when an assertion fails. Run: `cargo test --locked scope_guard::tests::pause_leaves_running_child_alone`. Observes: A pause exists, the child is alive, and the registry is unchanged.
  Fails without the behavior: If the watcher killed or reaped the process, `try_wait` returns Some or the registry entry is gone or changed, and the assertions fail.
- Criterion 6: Tests in status.rs and display.rs. Write one flag file (T1) and one pause file (T4) under the state `scope/` directory. `athena.status()` must list both with level and signals. `display::status` output must contain 'flagged' with T1, 'paused' with T4, and each line's first id. A control status with no files shows no Lines section. Run: `cargo test --locked status_lists_flagged_and_paused_lines`. Observes: Status text names each flagged or paused line with its T1–T5 signal.
  Fails without the behavior: Without `StatusView.lines` and its rendering, the status text lacks the signal names and the assertions fail.
- Criterion 7: Test in scope_guard.rs. Read the JSON pause file for a T4 pause and for a T3 pause. Assert the signal name, the numbers (rejection count and matched criterion for T4, spend and p90 for T3), and the lift text ('narrow' for T4, 'measurement' for T3). Run: `cargo test --locked scope_guard::tests::pause_record_content`. Observes: The signals, numbers and lift fields hold the expected text.
  Fails without the behavior: If the numbers or lift text were omitted, the field assertions fail.
- Criterion 8: Read the diff for CHANGELOG.md and docs/design/scope-guard.md. Only git diff, status, log and show are allowed commands; use them to locate the entries. Run: `git diff`. Observes: The diff shows an Unreleased CHANGELOG entry for the running-line watcher. It shows scope-guard.md marking job 5 built, with the pause file location and the manual lift, and stating that p90 comes from the reading's forecasts (`forecasts.reader.cost_usd.p90()`) and is used by `t3_cost`.
- Criterion 9: Run the three checks Athena runs. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: Any format, lint or test failure, including the budgets test, fails the command.

Planned files:

- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/design/scope-guard.md (modify)

## The change

Base 6e136ba8fc0f → candidate 32ef4512b0f0. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |   7 +
 crates/athena-cli/src/display.rs      |  55 ++
 crates/athena-core/src/reading.rs     |   4 +
 crates/athena-core/src/run.rs         |   6 +
 crates/athena-core/src/scope_guard.rs | 925 +++++++++++++++++++++++++++++++++-
 crates/athena-core/src/status.rs      |  13 +
 docs/design/scope-guard.md            |  28 +-
 7 files changed, 1032 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1270 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
