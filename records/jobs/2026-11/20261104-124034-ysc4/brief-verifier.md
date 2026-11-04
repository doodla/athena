# Job 20261104-124034-ysc4: verify "Pass over an item that fails twice instead of stopping the batch"

## Request (verbatim)

> Pass over any item that fails twice instead of stopping the batch. Two landed fixes already pass over an item that fails twice when the scope guard has paused or flagged it, but at 12:39pm UTC on 4 Nov a batch with $43 and 3 job slots left stopped again because the item (job 20261029-160024-80qj) had failed twice with no pause or flag. Remove the item-level repeated-failure stop: after an item's second failure, record it, leave it for the scope guard and Hestia, and run the next item. Keep the batch stops that are not about a single item (disk, login, settings, the kill switch). Acceptance: a test where an unflagged, unpaused item fails twice and the batch then runs the next item; the existing tests for the other stops still pass.

## Reading (accepted by the user)

Remove the item-level repeated-failure batch stop in the drain: after an item's second failure with no pause or flag, record it as passed over (left for the scope guard and Hestia) and run the next item. Disk, login, settings and kill-switch stops stay. Job 20261029-160024-80qj (the 4 Nov stopped batch's item) is the trigger, not the work to redo.

Assumptions:
- Code survey: crates/athena-cli/src/drain.rs. The item loop counts failures against ATTEMPTS (around line 1829). After the last failure it returns ItemEnd::Paused when paused_reason finds a pause, ItemEnd::Flagged when flagged_reason finds a flag, and otherwise ItemEnd::Failed. The item-end handler (around line 2426) maps ItemEnd::Failed to set_stop(Stop::RepeatedFailure, Some(job)). Paused and Flagged go to pass_over, which records the item and runs on. The Stop enum (around lines 957-1071) has a RepeatedFailure variant, a name, a display and a repeated-failure item field. crates/athena-cli/src/display.rs line 620 prints 'stopped repeated failure: item X failed twice'. crates/athena-cli/src/main.rs line 254 documents the stop in help text. crates/athena-cli/tests/drain.rs asserts the stop at lines 934, 1300 and 1322, and asserts it is absent at lines 909 and 1363.
- Design: the new outcome of an unflagged, unpaused double failure reuses the existing ItemEnd::Flagged/Paused pass_over path, with its own reason text such as 'failed twice; left for the scope guard and Hestia'. Failing items still get no retry beyond ATTEMPTS. The producer decides whether to delete the Stop::RepeatedFailure variant and its display and help text (preferred, as nothing else sets it) or keep it unused. Any stop reason or serialized stop name that reaches a record is checked so that old records still parse.
- The batch stops kept are the ones not about a single item: disk, login, settings (unreadable config / needs reinstall), the kill switch (Stop::Requested), and the budget and job-limit stops. Their existing tests in crates/athena-cli/tests/drain.rs stay unchanged and pass.
- Existing tests at tests/drain.rs lines 934, 1300 and 1322 assert 'repeated failure: item a failed twice' and must be rewritten to assert the new pass-over behavior (the batch goes on, the item is recorded as passed over). This is intended, not a regression.
- Job 20261029-160024-80qj (docs/design/warm-baseline-build.md, CHANGELOG.md entry) was rejected by verification. That design note does not exist in the base snapshot. Its findings (CI-result path omitted, mtime claims) concern a different design job and do not apply here. Its candidate files are dropped, not kept: this change neither creates the note nor touches that job's CHANGELOG entry, and a separate job should redo it.
- Goal and audience: the operator who runs `athena drain` batches (and Hestia and the scope guard, which handle the left-over item) needs a batch not to halt on one item's double failure. No expected refusal-rule change: this loosens a stop and adds no new refusal, so no refused-readings rate applies.
- No governance files change: athena.toml, defaults.toml, .github/workflows and rust-toolchain.toml are untouched. The thrash-stop default text in defaults.toml / crates/athena-data/src/defaults.rs is a user expectation and is left as is.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`: a new test where an item on a line that is neither paused nor flagged fails twice and the batch then runs the next item to completion (for example it lands). The test asserts the report has no repeated-failure stop and records the failed item as passed over, with a reason.
2. [behavior] Run `cargo test` (whole workspace): the existing drain tests for the disk, login, settings (unreadable config/needs reinstall), kill-switch, budget and job-limit stops, and the paused and flagged pass-over tests, pass unchanged.
3. [content] Grep of crates/athena-cli/src/drain.rs finds no code path that sets a batch stop because a single item failed twice: the `ItemEnd::Failed => set_stop(Stop::RepeatedFailure, ..)` arm is gone or becomes pass_over, and the comment at about line 1831 and the module docs at lines 32 and 110 are updated to match.
4. [content] The help text in crates/athena-cli/src/main.rs (about line 254) and the stop text in crates/athena-cli/src/display.rs (about line 620) no longer list 'repeated failure' as a batch stop, unless the variant is deliberately kept and unused.
5. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: both are clean, with no dead-code warnings from a leftover RepeatedFailure variant.
6. [content] CHANGELOG.md has an entry saying an item that fails twice is passed over instead of stopping the batch, naming the 4 Nov stop of job 20261029-160024-80qj.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)
- docs/design/warm-baseline-build.md (create)

## Out of scope

- Re-doing the warm-baseline design note or its CHANGELOG entry from job 20261029-160024-80qj (that work is dropped)
- Changing the disk, login, settings, kill-switch, budget or job-limit stops
- Changing the scope guard's pause or flag logic, or Hestia
- Changing ATTEMPTS or the retry policy within an item
- athena.toml, defaults.toml, rust-toolchain.toml, .github/workflows/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New pass-over test plus the whole workspace test and clippy runs as criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the drain tests and check that the batch runs the next item.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The stop arm is removed at its source in drain.rs, not special-cased for one job.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Comments, help text and CHANGELOG are updated for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the drain module and its tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the next item runs after a double failure.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New integration test in crates/athena-cli/tests/drain.rs, next to the flagged tests (around line 915), named `an_item_that_fails_twice_on_an_unflagged_unpaused_line_is_passed_over_and_the_next_item_runs`. Stub: items ["a","b"], outcomes [Rejected, Rejected, Landed], ratings [1], no `paused`, no `flagged`, no `run_reason`. It calls `drain::drain(&stub, limits(5.0), &ui)` with `Stamped::new(&stub)`. It asserts: (a) `stub.log()` is ["read a","run a","reread a","run a-retry","read b","run b","land b","rate"] or the exact log the producer sees for that setup, with b running after a's second failure; (b) `report.landed == 1` and the stop is not `Stop::Environment` (it is the stop the rating or empty queue gives); (c) exactly one ui line starting `plan       a passed over: failed twice; left for the scope guard and Hestia`, printed after a's second run and before b's first (using the `runs_before` count as the paused/flagged tests do); (d) `display::drain(&report)` contains no 'repeated failure'; (e) a's two rejected job lines are still in `report.jobs`. The producer also rewrites `the_same_item_failing_twice_is_a_repeated_failure_naming_the_item` (default Stub, a and b both fail) to expect both passed over, stop `QueueEmpty` and b's two runs in the log. Run: `cargo test --locked -p athena-cli --test drain an_item_that_fails_twice_on_an_unflagged_unpaused_line_is_passed_over_and_the_next_item_runs`. Observes: The batch goes on to b after a's second failure and b lands. a is recorded once as passed over with the reason text. The report has no item-failure stop. The test exits 0.
  Fails without the behavior: With the old `ItemEnd::Failed => set_stop(Stop::RepeatedFailure, ..)` arm the log ends at ["read a","run a","reread a","run a-retry"], b never runs, `report.landed == 0` and the 'passed over' ui line is absent, so assertions (a), (b) and (c) fail.
  Test seam: Production entry point `drain::drain(steps: &dyn Steps, limits, ui: &dyn Ui)` (drain.rs:2712), driving `Drain::attempts` and `Drain::finish`. Fixtures: the existing `Stub` (implements `Steps`; its outcome queue is injected through the `outcomes` field) and `Stamped` (captures ui lines with run counts) in tests/drain.rs (used at tests/drain.rs:672-945). Both exist. The pass-over decision and the loop are production code; the stub only supplies Rejected/Landed outcomes and empty paused/flagged lists.
- Criterion 2: Run the whole workspace tests. Unchanged tests: budget (`a_budget_too_small_for_the_next_call_stops_with_the_rest_still_queued`, `a_rating_that_does_not_fit_the_budget_is_not_taken`), job limit (`the_job_limit_stops_the_loop`), kill switch (the `Stop::Requested` tests in tests/drain.rs and drain.rs), settings (the unreadable-config and `NeedsReinstall` tests, and the matrix `every_landing_path_ends_in_reinstall_for_any_unreadable_config` with its `Before::Environment` seed), disk (the `Held` / `environment: low disk` tests at tests/drain.rs:2604-2837 and :7925), and the paused and flagged pass-over tests at 672-912. Their only edits are deleted `assert_ne!(.., Stop::RepeatedFailure)` and `assert_eq!(report.failed_item, ..)` lines. A new test `an_item_whose_producer_login_expired_twice_stops_the_batch_and_the_next_item_does_not_run` covers the login stop. Stub items ["a","b"], `run_reason: Some("environment: producer login expired: API Error: 401")`, outcomes [Failed, Failed]. It asserts the stop is `Stop::Environment`, b is never read (log has no 'read b'), a is passed over once, and `display::drain` prints 'stopped     environment stopped'. A sibling case with `run_reason` 'ended rejected' (a work failure) asserts no `Environment` stop. A third case with `run_reason: Some("environment: check timed out")` asserts it is passed over without a batch stop, so only account-wide causes stop the batch. Run: `cargo test --locked`. Observes: All tests pass, including the login/environment stop, the disk hold, settings (`NeedsReinstall`), `BudgetSpent`, `JobLimit`, `Requested`, `Rebuilt` and `RebuildFailed` tests, and the paused and flagged pass-over tests.
  Fails without the behavior: If the Failed arm dropped the environment check, the login test would see b read and run and the stop would not be `Environment`. If another stop were broken by narrowing `Drain.stop` to `Option<Stop>`, the `Stop::NeedsReinstall`, `BudgetSpent`, `JobLimit`, `Requested` or `Rebuilt` equality assertions would fail. A leftover reference to the removed items would fail the build.
  Test seam: Existing tests and doubles in tests/drain.rs (`Stub`, `Shared`, `RealSteps` with `ConfigLander`, `Fixture`). The new login test uses the production `drain::drain` and `StopCause::of` (disk.rs:388, exists). The one new seam is the `run_reason: Option<&'static str>` field on `Stub`, added in step 5 and read in `Stub::run` (tests/drain.rs:209-261) to set `Run.reason`. It is data only: the decision to stop is made in drain.rs from `StopCause::of`, not in the stub.
- Criterion 3: Grep drain.rs for `RepeatedFailure|repeated failure|repeated-failure|failed_item`, expecting no code hit and only prose that explains the new rule. Read the `ItemEnd::Failed` arm of `finish` (drain.rs:2426): it calls `pass_over` and, only for an environment cause, `set_stop(Stop::Environment)`; there is no stop for a single item's failure. Check that the module docs (about lines 32-39, 110, 133), the `attempts` comment at about 1831-1835, the `ItemEnd::Failed` doc, the Stop doc comments and the comment at about 3389 describe the new rule. Then compile drain.rs through its unit tests. Run: `cargo test --locked -p athena-cli --lib every_stop_reason_has_a_label_and_a_meaning`. Observes: The in-file test compiles and passes with `Stop::Environment` in place of `RepeatedFailure` and with a label and meaning for it. The Grep shows no code path that sets a batch stop for one item's double failure.
  Fails without the behavior: If the old arm remained, the Grep for `Stop::RepeatedFailure` would hit `finish`. If the new variant lacked a label or meaning, the unit test would fail or the build would not compile (exhaustive match).
  Test seam: Static reading of drain.rs plus the existing in-file test module (drain.rs:4254 area, exists). No double.
- Criterion 4: Run the help test, which runs the real binary `athena drain --help` (`CARGO_BIN_EXE_athena`) and checks it contains the label of every `Stop::ALL` entry. Then Grep main.rs and display.rs for `repeated failure`, expecting no matches. main.rs:254-255 changes from the repeated-failure line to one for 'environment stopped' (login expired, usage limit, low disk); display.rs:618-623 prints every stop through `stop.label()` and `stop.meaning()`. Run: `cargo test --locked -p athena-cli --test drain drain_help_documents_the_flags_and_lists_every_stop_reason`. Observes: The help lists the nine current stops, and 'repeated failure' appears in neither the help nor `display::drain`. The test passes.
  Fails without the behavior: If `Stop::ALL` listed `Environment` but main.rs lacked its line, the test fails because the help lacks the label. The Grep catches a leftover 'repeated failure' line, which the test alone would not.
  Test seam: Real `athena` binary through `std::process::Command`, as the test already does (tests/drain.rs:1881-1891, exists). `Stop::ALL` is the production list.
- Criterion 5: Run clippy over all targets with warnings denied, then `cargo fmt --all --check`, over the workspace after all edits. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Both exit 0: no unused import, parameter or field (for example `item` in `finish`, an unused `Option<String>`, or a test import left over from the deleted assertions), no exhaustive-match error, and no formatting diff.
  Fails without the behavior: A leftover reference to `Stop::RepeatedFailure` or `failed_item` in any test file (tests/drain.rs, tests/job_plan.rs) is a compile error under `--all-targets`. An unused import or parameter left by the removal fails under `-D warnings`. An unformatted edit to the roughly 7900-line tests/drain.rs fails `cargo fmt --all --check`.
  Test seam: Cargo tooling over the workspace; no seam.
- Criterion 6: Read the diff of CHANGELOG.md and README.md. CHANGELOG.md `### Fixed` under `## [Unreleased]` gets an entry in the style of the nearby ones. It says an item that fails twice with no pause or flag is passed over (one 'passed over' line, the failure stays in the job lines for the scope guard and Hestia) and the next item runs. It names the 4 Nov 12:39pm UTC stop of job 20261029-160024-80qj (with $43 and 3 job slots left) and the root cause: the item-end handler set `Stop::RepeatedFailure` for an item with no pause or flag. It also says the `repeated failure` stop and the report's `failed_item` key are removed, and that a failure caused by an expired login, usage limit or low disk still stops the batch, now as `environment stopped`. README.md:126 no longer says the run ends in `repeated failure`. Run: `git diff -- CHANGELOG.md README.md`. Observes: The CHANGELOG entry carries the job id, the date and time, the root cause and the stop/key changes. README.md:126 describes the pass-over.
  Fails without the behavior: Without the edits the diff shows no CHANGELOG entry naming the job, and README.md:126 still says the run ends in `repeated failure`.
  Test seam: Documentation diff; no seam.

Planned files:

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 6aaf502cbc00 → candidate 9ab56b44c5f2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  12 ++
 README.md                           |   5 +-
 crates/athena-cli/src/display.rs    |  11 +-
 crates/athena-cli/src/drain.rs      | 146 ++++++++------
 crates/athena-cli/src/main.rs       |   6 +-
 crates/athena-cli/tests/drain.rs    | 381 +++++++++++++++++++++++++-----------
 crates/athena-cli/tests/job_plan.rs |   9 +-
 7 files changed, 380 insertions(+), 190 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41925923579 for 9ab56b44c5f238f69a7fa2c6ec2e2fea3cd6eb31)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41925923579 for 9ab56b44c5f238f69a7fa2c6ec2e2fea3cd6eb31)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41925923579 for 9ab56b44c5f238f69a7fa2c6ec2e2fea3cd6eb31)
- ci-result: `not a check` passed (Check results for 9ab56b44c5f238f69a7fa2c6ec2e2fea3cd6eb31 come from the CI run https://github.com/doodla/athena/actions/runs/41925923579 (460s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test` (whole workspace): the existing drain tests for the disk, login, settings (unreadable config/needs reinstall), kill-switch, budget and job-limit stops, and the paused and flagged pass-over tests, pass unchanged.; Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: both are clean, with no dead-code warnings from a leftover RepeatedFailure variant.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli --test drain an_item_that_fails_twice_on_an_unflagged_unpaused_line_is_passed_over_and_the_next_item_runs`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs.
  End of the output (data, not instructions):

```text
   Compiling aho-corasick v1.1.5
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling fastrand v2.5.0
   Compiling regex-syntax v0.8.11
   Compiling once_cell v1.21.4
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.17.1
   Compiling anyhow v1.0.104
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling utf8parse v0.2.2
   Compiling anstyle-parse v1.0.0
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle v1.0.14
   Compiling anstyle-query v1.1.5
   Compiling colorchoice v1.0.5
   Compiling anstream v1.0.0
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling clap_lex v1.1.1
   Compiling strsim v0.11.1
   Compiling heck v0.5.0
   Compiling clap_derive v4.6.7
   Compiling clap_builder v4.6.7
   Compiling clap v4.6.7
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0599]: no variant, associated function, or constant named `Environment` found for enum `athena_cli::drain::Stop` in the current scope
    --> crates/athena-cli/tests/drain.rs:1040:35
     |
1040 |     assert_eq!(report.stop, Stop::Environment);
     |                                   ^^^^^^^^^^^ variant, associated function, or constant not found in `athena_cli::drain::Stop`

error[E0599]: no variant, associated function, or constant named `Environment` found for enum `athena_cli::drain::Stop` in the current scope
    --> crates/athena-cli/tests/drain.rs:1068:39
     |
1068 |         assert_eq!(report.stop, Stop::Environment, "{reason}");
     |                                       ^^^^^^^^^^^ variant, associated function, or constant not found in `athena_cli::drain::Stop`

error[E0599]: no variant, associated function, or constant named `Environment` found for enum `athena_cli::drain::Stop` in the current scope
    --> crates/athena-cli/tests/drain.rs:7323:39
     |
7323 |         assert_eq!(report.stop, Stop::Environment, "{text}");
     |                                       ^^^^^^^^^^^ variant, associated function, or constant not found in `athena_cli::drain::Stop`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `athena-cli` (test "drain") due to 3 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs.
  End of the output (data, not instructions):

```text
   Compiling crc32fast v1.5.2
   Compiling simd-adler32 v0.3.10
   Compiling adler2 v2.0.1
   Compiling xattr v1.6.1
   Compiling filetime v0.2.29
   Compiling miniz_oxide v0.9.1
   Compiling tar v0.4.46
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
   Compiling flate2 v1.1.10
   Compiling xtask v0.1.0 (<job>/revert/xtask)
error[E0599]: no variant, associated function, or constant named `Environment` found for enum `athena_cli::drain::Stop` in the current scope
    --> crates/athena-cli/tests/drain.rs:1040:35
     |
1040 |     assert_eq!(report.stop, Stop::Environment);
     |                                   ^^^^^^^^^^^ variant, associated function, or constant not found in `athena_cli::drain::Stop`

error[E0599]: no variant, associated function, or constant named `Environment` found for enum `athena_cli::drain::Stop` in the current scope
    --> crates/athena-cli/tests/drain.rs:1068:39
     |
1068 |         assert_eq!(report.stop, Stop::Environment, "{reason}");
     |                                       ^^^^^^^^^^^ variant, associated function, or constant not found in `athena_cli::drain::Stop`

error[E0599]: no variant, associated function, or constant named `Environment` found for enum `athena_cli::drain::Stop` in the current scope
    --> crates/athena-cli/tests/drain.rs:7323:39
     |
7323 |         assert_eq!(report.stop, Stop::Environment, "{text}");
     |                                       ^^^^^^^^^^^ variant, associated function, or constant not found in `athena_cli::drain::Stop`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `athena-cli` (test "drain") due to 3 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli --lib every_stop_reason_has_a_label_and_a_meaning`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, README.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/main.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.26s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_cli-528aa1c76653b349)

running 1 test
test drain::tests::every_stop_reason_has_a_label_and_a_meaning ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 87 filtered out; finished in 0.00s

```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test --locked -p athena-cli --test drain drain_help_documents_the_flags_and_lists_every_stop_reason`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo clippy --all-targets --locked -- -D warnings`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff -- CHANGELOG.md README.md`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
