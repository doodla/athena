# Job 20261030-013405-6nnh: verify "Give OLY-B-GOAL-03 a mechanism: forecasts committed before dispatch"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-GOAL-03 a mechanism
> 
> Requirement OLY-B-GOAL-03 in docs/spec.toml is active and has no mechanism.
> Requirement: Athena predicts outcomes before it dispatches work.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-GOAL-03
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-GOAL-03 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Record the existing pre-dispatch forecast path (reader forecasts committed in reading.rs and scored per class in status.rs) as the mechanism for OLY-B-GOAL-03 in docs/spec.toml, with measure ending `status: forecasts`. Add a "forecasts" entry to STATUS_MEASURE_LABELS and a test that fails without the pre-dispatch forecast. Audience: Athena's maintainers and the Hestia spec pass.

Assumptions:
- Code survey: read docs/spec.toml (OLY-B-GOAL-01..04 entries, line ~2089), crates/athena-core/src/reading.rs (commits reading with forecasts before any work; refuses oversize wall-time forecast in build_reading), crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS at line 305, class_forecasts, StatusView.forecasts, test every_measure_label_names_a_status_view_field_and_a_printed_line at ~1630), crates/athena-cli/src/display.rs (forecasts line at ~1390), crates/athena-core/src/hestia.rs (spec pass at ~1230). No new record or data file under crates/ is added, so check_record_file/AnyRecord round-trip is unaffected. The property is measured directly: the reading's forecasts exist in the committed reading before dispatch, and status prints per-class scored forecasts.
- Existing code: crates/athena-core/src/reading.rs, crates/athena-core/src/status.rs, crates/athena-cli/src/display.rs, crates/athena-core/src/hestia.rs, docs/spec.toml; the change extends them and creates no new mechanism of its own where one already exists.
- The `forecasts` label is not yet in STATUS_MEASURE_LABELS; the existing label test requires the source to contain a literal `"forecasts ` string, so the display line needs a literal beginning with that text (the current code builds the label through a variable).
- The change adds no refusal rule; it refuses no new readings (expected 0% of newly refused readings).
- Governance files such as athena.toml and defaults.toml are not changed; no workflow or Environment reach is needed.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-GOAL-03 has a mechanism naming existing paths (crates/athena-core/src/reading.rs and crates/athena-core/src/status.rs) instead of "none", keeps owner Athena, and a measure ending `status: forecasts`; check by Grep of docs/spec.toml.
2. [content] STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs contains ("forecasts", "forecasts") and the `athena status` display prints a line starting with that label (crates/athena-cli/src/display.rs); check by Grep and diff.
3. [behavior] A new test fails without the mechanism and passes with it: it builds a reading without or with forecasts before dispatch (or a status view with scored forecasts) and asserts the forecast is present/committed and that the `forecasts` line is printed; run `cargo test` and confirm the test passes, and that removing the forecast makes it fail.
4. [behavior] `cargo test` (including every_measure_label_names_a_status_view_field_and_a_printed_line and the xtask spec test), `cargo clippy` and `cargo fmt --check` all pass.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing other requirements' mechanisms or measures in docs/spec.toml
- Changing forecast scoring or the reference forecast logic
- Edits to athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus cargo test and clippy pass criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test asserts forecasts are committed/printed, not just that wiring exists.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Measure ends in a status label that status actually prints.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Reuse the existing forecast path; one focused change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the new status.rs test, which parses the real docs/spec.toml, and read the entry with Grep. Run: `cargo test -p athena-core oly_b_goal_03`. Observes: The test passes. Grep of docs/spec.toml for `id = "OLY-B-GOAL-03"` with -A 4 shows a mechanism naming crates/athena-core/src/reading.rs and crates/athena-core/src/status.rs instead of "none", owner Athena, and a measure ending `status: forecasts`.
  Fails without the behavior: Reverting the mechanism to "none" or dropping the `status: forecasts` suffix fails the test (asserts on mechanism text, owner, measure_problem None, absence from spec_gaps; a clone with mechanism "none" is a gap).
  Test seam: Production entry points: crate::hestia::parse_spec (hestia.rs) over include_str!("../../../docs/spec.toml"), crate::hestia::measure_problem (hestia.rs:1219) and crate::hestia::spec_gaps; fixture is the real spec file (compiled in by include_str!) and a clone of the entry with mechanism set to "none". Exists (the functions are read in hestia.rs); the test is added by plan step 5.
- Criterion 2: Run the existing label test and the new display test; diff status.rs and display.rs to see the table entry and the "forecasts   " literal. Run: `cargo test -p athena-core every_measure_label_names_a_status_view_field_and_a_printed_line`. Observes: The label test passes with ('forecasts','forecasts'): StatusView has `    pub forecasts:` (status.rs:475) and display.rs holds `"forecasts `. Then `cargo test -p athena-cli forecast_headline` shows the rendered first row starts with `forecasts `.
  Fails without the behavior: Without the display.rs literal the label test fails with "`athena status` prints no line starting \"forecasts\""; without the table entry the display test still passes but the Grep/diff check fails.
  Test seam: Production entry points: the label test (status.rs:1630) reads include_str! of status.rs, display.rs and hestia.rs; forecast_headline_lines in display.rs, which status() calls. Fixture: ClassHeadline literals passed straight to the helper; no double. The label test exists; the helper and its test are added by plan step 3.
- Criterion 3: Three unit tests over real code paths. Reading test: build_sized returns Ok with the full forecast and Err without the `forecast` key. Spec test as in criterion 1. Display test: the rendered rows. Run: `cargo test -p athena-core a_reading_commits_the_reader_forecast_and_a_reference_before_dispatch`. Observes: Passes with the change: Reading.forecasts.reader.p_reading_accepted == 0.8, p_verification_passes == 0.7, p_user_intervenes == 0.1, cost p50 == 0.2, wall p50 == the input, the reference forecast validates, and the output without `forecast` is refused with 'parsing the reading'. Also run `cargo test -p athena-core oly_b_goal_03` and `cargo test -p athena-cli forecast_headline`; the display test shows row one `forecasts   docs: 3 scored · mean event Brier reader 0.120 · reference 0.250`, row two starting with 12 spaces, a None Brier printed as `-`.
  Fails without the behavior: Reading test: without the `forecast` object build_reading returns 'parsing the reading'; if build_reading stopped carrying the forecast into Reading.forecasts, the equality assertions on the reader values fail. Spec test: mechanism reverted to "none" or label dropped fails it. Display test: dropping the label or the per-class scores fails it.
  Test seam: Production entry points: Athena::build_reading (reading.rs:767) through the existing build_sized helper (reading.rs:1692; make_athena with a script that panics if an agent runs; Records::default(); shipped athena.toml and defaults.toml), injected as plain JSON from sized_reading; forecast_headline_lines (display.rs, step 3); hestia::parse_spec/measure_problem. build_sized and sized_reading exist (read at reading.rs:1666-1726); the helper is added by step 3 and the tests by steps 3-5. No double holds logic under test; fixtures are plain data.
- Criterion 4: Run the three checks Athena runs, in full, over the whole workspace, plus the two suites that read the edited spec entry and sources: the xtask spec test and the athena-core label test. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. Format of the edited files is clean, clippy has no warnings, and every test passes, including every_measure_label_names_a_status_view_field_and_a_printed_line (status.rs:1630), xtask/tests/spec.rs (check_how_it_is_met at line 194: non-empty mechanism, owner not none, measure ending `status: <label>`), the hestia real-spec tests (hestia.rs:4286 and :4315) and the three new tests. Run `cargo test -p xtask --test spec` separately to see the spec test alone.
  Fails without the behavior: Without the display.rs literal the label test fails for 'forecasts'. If the spec measure lost its `status:` label, check_how_it_is_met fails (xtask/tests/spec.rs:204-212). Without the spec edit the existing real-spec tests still pass (mechanism "none" is allowed there); the new status.rs test is what fails.
  Test seam: Production entry points: cargo's fmt, clippy and test over the workspace (the checks Athena runs), including xtask/tests/spec.rs check (reads the real docs/spec.toml and the brief) and the label test (status.rs:1630). Fixtures: the real repository files (docs/spec.toml, display.rs, status.rs, hestia.rs via include_str! or fs read); no fake or double and nothing injected. Exists today (read at xtask/tests/spec.rs:194-213 and status.rs:1630-1649); the new tests and the display literal are added by plan steps 3-5.

Planned files:

- docs/spec.toml (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base e08a304fc399 → candidate 25fa4c82a388. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  7 +++++
 crates/athena-cli/src/display.rs  | 57 ++++++++++++++++++++++++++++++---------
 crates/athena-core/src/reading.rs | 17 ++++++++++++
 crates/athena-core/src/status.rs  | 29 ++++++++++++++++++++
 docs/spec.toml                    |  4 +--
 5 files changed, 100 insertions(+), 14 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-013405-6nnh/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-013405-6nnh/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43160423250 for 25fa4c82a388a060af1daa96c52da729e983f7d1)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43160423250 for 25fa4c82a388a060af1daa96c52da729e983f7d1)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43160423250 for 25fa4c82a388a060af1daa96c52da729e983f7d1)
- ci-result: `not a check` passed (Check results for 25fa4c82a388a060af1daa96c52da729e983f7d1 come from the CI run https://github.com/doodla/athena/actions/runs/43160423250 (902s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test` (including every_measure_label_names_a_status_view_field_and_a_printed_line and the xtask spec test), `cargo clippy` and `cargo fmt --check` all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core oly_b_goal_03`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-core/src/status.rs, docs/spec.toml.
  End of the output (data, not instructions):

```text
   Compiling regex-automata v0.4.18
   Compiling tempfile v3.27.0
   Compiling schemars_derive v1.2.2
   Compiling hashbrown v0.17.1
   Compiling anyhow v1.0.104
   Compiling equivalent v1.0.2
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 54.27s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_core-18f5333b8c967527)

running 1 test
test status::mistake_tests::oly_b_goal_03_is_met_by_the_forecast_path_and_shown_by_the_forecasts_line ... FAILED

failures:

---- status::mistake_tests::oly_b_goal_03_is_met_by_the_forecast_path_and_shown_by_the_forecasts_line stdout ----

thread 'status::mistake_tests::oly_b_goal_03_is_met_by_the_forecast_path_and_shown_by_the_forecasts_line' (27817) panicked at crates/athena-core/src/status.rs:1655:9:
none
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    status::mistake_tests::oly_b_goal_03_is_met_by_the_forecast_path_and_shown_by_the_forecasts_line

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1371 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p athena-core --lib`
     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-a2cb0acbd401486e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/model_roster.rs (<job>/target/verify/debug/deps/model_roster-798f4afe333501c6)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core every_measure_label_names_a_status_view_field_and_a_printed_line`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-core/src/status.rs, docs/spec.toml.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.05s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_core-18f5333b8c967527)

running 1 test
test status::mistake_tests::every_measure_label_names_a_status_view_field_and_a_printed_line ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1371 filtered out; finished in 0.01s

     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-a2cb0acbd401486e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/model_roster.rs (<job>/target/verify/debug/deps/model_roster-798f4afe333501c6)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s

```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core a_reading_commits_the_reader_forecast_and_a_reference_before_dispatch`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-core/src/status.rs, docs/spec.toml.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_core-18f5333b8c967527)

running 1 test
test reading::tests::a_reading_commits_the_reader_forecast_and_a_reference_before_dispatch ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1371 filtered out; finished in 0.47s

     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-a2cb0acbd401486e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/model_roster.rs (<job>/target/verify/debug/deps/model_roster-798f4afe333501c6)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s

```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
