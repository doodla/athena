# Job 20261029-041417-xkyn: verify "Make Hestia's sweep own and schedule Athena's recurring duties"

## Request (verbatim)

> Priority: high
> Give Athena's recurring duties an owner and a schedule, so none depends on a person remembering. The duties: the error-review backlog pass, the evidence-label derivation, the code-health check, the periodic design review and the threshold checks (keep rule, reading-rule refusal rates). Make Hestia's sweep the owner: one table in docs/design/hestia.md lists each duty, its cadence (every drain start, every landing, or every N landings) and where it runs, and the sweep runs each one on that cadence and records when it last ran. athena status shows each duty with its last run and next due point, and flags any duty overdue.
> Acceptance:
> - A test shows the sweep runs each listed duty on its cadence and records the last run, and a second sweep within the cadence runs none.
> - A test shows status flags a duty whose last run is past its cadence.
> - docs/design/hestia.md lists every duty with owner and cadence.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/hestia.rs, crates/athena-core/src/status.rs, docs/design/hestia.md, CHANGELOG.md

## Reading (accepted by the user)

Add a duty table (docs/design/hestia.md and a matching code table in hestia.rs) listing the error-review pass, evidence-label derivation, code-health check, periodic design review and threshold checks with cadence (drain start, landing, every N landings) and where each runs. The sweep runs each due duty and records its last run; `athena status` shows last run and next due for each duty and flags overdue ones. Audience: the owner/operator reading status and design doc.

Assumptions:
- Code survey: read crates/athena-core/src/hestia.rs (Check trait, default_checks, run_sweep, ReadingRulesCheck), crates/athena-core/src/status.rs (StatusView, ReadingRuleStats, status()) and the record types in crates/athena-data/src/records.rs (AnyRecord, HestiaSweep). run_sweep runs all checks every time with no cadence; last-run state has to be persisted, most likely as a new field or record in athena-data; if a new record file is added it must pass check_record_file, AnyRecord and the round-trip test.
- Last-run is stored as records (a duty-run record or a field on the sweep record) and 'every N landings' is counted from landed jobs in records; the exact mechanism is the producer's call after reading the code.
- Duties are run through the existing Check trait or a thin Duty wrapper; existing implementations of the error review, throughput and reading-rule checks are reused, and duties with no implementation yet are registered with a clear 'not yet automated' outcome rather than invented.
- No change to athena.toml, defaults.toml or workflows is needed; none is a deliverable.
- Cadence N values are chosen by the producer and documented in the table.

## Acceptance criteria

1. [behavior] A test in crates/athena-core/src/hestia.rs, run by `cargo test -p athena-core`, builds a duty table with a drain-start, a per-landing and an every-N-landings duty. It shows one sweep runs each due duty and records its last run, and a second sweep within the cadence runs none.
2. [behavior] A test in crates/athena-core/src/status.rs, run by `cargo test -p athena-core`, shows the status view flags a duty whose last run is past its cadence as overdue and does not flag a duty within its cadence. It also shows each duty with last run and next due point.
3. [content] docs/design/hestia.md has one table listing every duty (error-review backlog pass, evidence-label derivation, code-health check, periodic design review, keep-rule threshold check, reading-rule refusal-rate check) with owner Hestia's sweep, cadence and where it runs. Grep of the file finds each duty name.
4. [content] The duty table in hestia.rs lists the same duties and cadences as the table in docs/design/hestia.md, checked by a diff read or a test comparing ids.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
6. [content] CHANGELOG.md has an entry describing the duty schedule, last-run records and status overdue flag.
7. [content] Any new or changed record type under crates/athena-data is covered by `check_record_file`, `AnyRecord` and the round-trip test in records.rs, and old records still parse.

## Expected deliverables

- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/status.rs (modify)
- docs/design/hestia.md (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- Building new implementations of duties that do not exist yet beyond registering them
- Changing the threshold values of the keep rule or reading-rule refusal flags
- Changes to athena.toml, defaults.toml or CI workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests and fmt/clippy/test criteria included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests exercising cadence and overdue behavior.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Design doc table and changelog written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test `a_sweep_runs_each_due_duty_records_the_last_run_and_a_second_sweep_runs_none` in hestia.rs `mod tests`, in a temp repository from `athena_in(dir, state)`. The table holds three test duties: A DrainStart, B EveryLanding, C EveryLandings(3). A `CountingRunner` implements `DutyRunner`, logs the ids it is asked to run, and can be told to fail one id. The test calls `athena.hestia_sweep_duties(&[], &Quiet, &table, &runner, ctx)` with an injected `DutyContext { occasion, landings, now }`. After each call it loads the committed sweeps and reads the newest. Steps:
(1) Sweep, landings 0: A, B, C run; the record lists three non-errored checks and `duty_pass.landings == 0`.
(2) Sweep, landings 0: none run; the record lists no checks.
(3) Sweep, landings 1: B only.
(4) Sweep, landings 3: B and C.
(5) Sweep, landings 3: none.
(6) DrainStart occasion: A only.
(7) Landing occasion, landings 4: B only.
(8) The runner fails A at a DrainStart. A is recorded errored. A Sweep then re-runs A only. The next Sweep runs none. Run: `cargo test -p athena-core a_sweep_runs_each_due_duty`. Observes: The runner's call log matches each expectation above. The committed records, read back from git, list exactly the ids that ran, with the landing total at that time. The second sweep's record lists no checks.
  Fails without the behavior: A sweep that ignores cadence would run all three at step 2. One that records no check ids or totals would re-run everything. A wrong landing count would run the wrong duties at steps 3, 4 and 7. An errored run counted as a run would not retry at step 8.
  Test seam: Production entry `Athena::hestia_sweep_duties` (added by step 6). Fixture `athena_in` exists (hestia.rs:2984). The double `CountingRunner` implements the production trait `DutyRunner` (step 4) and only logs ids and fails on request. `DutyContext` carries injected landings and `now`, so no landed commits and no clock are needed. The commit and the read-back run through the real `commit_sweep` and `Records::load_current`, which exist.
- Criterion 2: Tests in status.rs `mod tests`.
Case A calls the pure `crate::hestia::duty_standings(&DUTIES, sweeps, baselines, ctx)` with `HestiaSweep` literals that carry `duty_pass`, built at injected times. Checked: an EveryLanding duty recorded at total 4 with the total now 5 is overdue; an EveryLandings(5) duty recorded at 0 with 4 landings is not overdue and says 'after 1 more landing'; at 5 it is overdue; a never-run duty is overdue with last run None; a drain-start duty that ran is not overdue however old; a drain-start duty whose newest attempt errored is overdue and shows `last_failed_at`; design-review with a baseline at 2 and total 11 is not overdue, and at 12 it is.
Case B calls `Athena::status()` over a repository where a sweep was committed with `commit_in_tree`. Checked: `view.duties` lists all six ids with last run and next due text; the duties in the sweep are not overdue and the rest are; `serde_json::to_value(&view)` has `duties`. Run: `cargo test -p athena-core duty_standings`. Observes: Every duty has a last run (or 'never') and a next due text. Exactly the late duties have `overdue == true` and the in-cadence ones false. The JSON view carries the list.
  Fails without the behavior: If overdue were never computed the late assertions fail. If it were always true the in-cadence assertions fail. If `status()` did not build the field, case B finds no `duties`.
  Test seam: Production entries `crate::hestia::duty_standings` and `Athena::status()` (status.rs:1129; the field is added by step 7). Sweep fixture: `status::records_worktree_tests::commit_in_tree` with `athena_in`, both existing. In case A the landing total, baselines and `now` are plain arguments, injected directly. The cases with landings go through the pure function because the real repository in case B has no landings, and nothing inside it is mocked.
- Criterion 3: Show the new section 13 of docs/design/hestia.md with the diff and with the test of criterion 4. The test also asserts every owner cell is "Hestia's sweep", the six ids appear in order, and the duty names (error-review backlog pass, evidence-label derivation, code-health check, periodic design review, keep-rule threshold check, reading-rule refusal-rate check) appear in the row text. Run: `git diff -- docs/design/hestia.md`. Observes: One table with columns id, duty, owner, cadence, where it runs. Its six rows are error-review, evidence-labels, code-health, design-review, keep-rule and reading-rule-refusals. Every owner cell is Hestia's sweep. The cadences read: every drain start (two rows), every landing, every 10 landings (two rows), every 5 landings.
  Fails without the behavior: A missing or renamed row fails `the_duty_table_matches_the_design_doc`.
  Test seam: Content criterion. The test in criterion 4 reads the real file through `include_str!("../../../docs/design/hestia.md")`. No double.
- Criterion 4: Unit test `the_duty_table_matches_the_design_doc` in hestia.rs. It finds the line `## 13. Recurring duties`, takes the table rows up to the next `## ` heading, and compares each row's id (backticks stripped) and cadence cell with `DUTIES[i].id` and `DUTIES[i].cadence.describe()`. It also asserts the row count equals `DUTIES.len()`. Run: `cargo test -p athena-core the_duty_table_matches_the_design_doc`. Observes: Doc ids in order equal the code ids, and each cadence text equals the code's description.
  Fails without the behavior: A renamed id or a changed N on either side makes the comparison fail.
  Test seam: Production values `DUTIES` and `Cadence::describe` (step 2) against the real doc (step 8). No double.
- Criterion 5: Run the three gate commands over the workspace. The changed paths are driven by existing fixtures. `athena_in` in hestia.rs drives `hestia_sweep_with` and `review_passes` (:3228, :5020, :3515). The cli `Fixture` with `real_steps_with` in crates/athena-cli/tests/drain.rs drives `Athena::review_passes` through `drain::drain` (:4583, :5909 and the other drains). `Fixture` and `run_job` in crates/athena-cli/tests/code_health_hooks.rs drive `code_health_after_landing`, as does `set_up` in gh/after_merge_tests.rs (:58). Also run `cargo test -p athena-data`, `-p athena-core` and `-p athena-cli`. Run: `cargo test --locked`. Observes: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` each exit 0.
  Fails without the behavior: A sweep that no longer runs a check an existing test expects, a changed drain or code-health line, a missing `duty_pass` in a `HestiaSweep` literal, or a `StatusView` built without `duties` fails the build or a test.
  Test seam: Existing production entries listed above. No double is added. The fakes are the existing agent drivers (`Scenario`, `FakeAgentDriver`), `RecordingLander` and the `Pages` fake, all of which exist.
- Criterion 6: Read the CHANGELOG.md entry under `## [Unreleased]`, `### Added`. It names the duty table (`DUTIES`, hestia.md section 13), the `duty_pass` field of the sweep record (old sweeps still parse), that drain start, landing and `athena hestia sweep` run the due duties, the `athena status` duties section with last run, next due and the overdue flag, and the cost of one extra records-only commit per drain start or landing that ran a duty. Run: `git diff -- CHANGELOG.md`. Observes: One new entry that mentions the duty schedule, the last-run records and the overdue flag.
  Fails without the behavior: No entry means the diff is empty.
  Test seam: Content criterion; the diff of the real file. No double.
- Criterion 7: Tests in records.rs extend `sweep_with` and `a_hestia_sweep_has_a_path_a_schema_and_round_trips` (:2576) and add `a_hestia_sweep_with_a_duty_pass_round_trips_and_an_old_sweep_still_parses`. A sweep with `duty_pass { occasion: landing, landings: 7 }` and checks named like duties (one errored with its finding) round-trips through `AnyRecord::parse` and `to_json`. `check_record_file` accepts its canonical path. `validate` still refuses a repeated check id. A literal sweep JSON as written before this change (no `duty_pass`) parses with `duty_pass == None`. JSON with an unknown field is still refused. The three occasion names serialize as drain_start, landing and sweep. A hestia.rs test for `is_duty_pass` covers the three occasions. Run: `cargo test -p athena-data hestia_sweep`. Observes: The new sweep round-trips and passes `check_record_file`. The old JSON parses. A duplicate check id is refused.
  Fails without the behavior: If the new field broke old records, the old-JSON assertion fails. If `check_record_file` or `AnyRecord` did not cover it, the path assertion fails.
  Test seam: Production `AnyRecord::parse`, `check_record_file` and `HestiaSweep::validate` (records.rs:1232, :1333; throughput.rs:345), all existing. Fixture `sweep_with` exists (records.rs:2547) and gains the field. No double.

Planned files:

- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/status.rs (modify)
- docs/design/hestia.md (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-data/src/records/throughput.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/code_health.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/gh/after_merge_tests.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 5db2b056fd56 → candidate 241391d970bb. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                   |   17 +
 crates/athena-cli/src/display.rs               |   57 ++
 crates/athena-cli/src/lib.rs                   |    1 +
 crates/athena-cli/src/main.rs                  |   10 +-
 crates/athena-cli/tests/drain.rs               |   46 +-
 crates/athena-cli/tests/trailer_stop.rs        |    3 +
 crates/athena-core/src/code_health.rs          |   58 +-
 crates/athena-core/src/gh/after_merge_tests.rs |   63 ++
 crates/athena-core/src/hestia.rs               | 1063 +++++++++++++++++++++---
 crates/athena-core/src/status.rs               |  222 +++++
 crates/athena-core/src/test_support.rs         |    3 +
 crates/athena-data/src/records.rs              |   55 ++
 crates/athena-data/src/records/throughput.rs   |   36 +-
 docs/design/hestia.md                          |   58 +-
 14 files changed, 1560 insertions(+), 132 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-041417-xkyn/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-041417-xkyn/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42791931760 for 241391d970bb515ddc34c770773bcd8be1a2dac4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42791931760 for 241391d970bb515ddc34c770773bcd8be1a2dac4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42791931760 for 241391d970bb515ddc34c770773bcd8be1a2dac4)
- ci-result: `not a check` passed (Check results for 241391d970bb515ddc34c770773bcd8be1a2dac4 come from the CI run https://github.com/doodla/athena/actions/runs/42791931760 (684s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core a_sweep_runs_each_due_duty`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/code_health.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records/throughput.rs, docs/design/hestia.md.
  End of the output (data, not instructions):

```text
     |                         ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:6960:25
     |
6960 |         assert_eq!(pass(Occasion::DrainStart, 6), ["a-drain"]);
     |                         ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:6967:25
     |
6967 |         assert_eq!(pass(Occasion::Sweep, 6), ["a-drain"]);
     |                         ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:6968:22
     |
6968 |         assert!(pass(Occasion::Sweep, 6).is_empty());
     |                      ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7048:21
     |
7048 |                 ctx(Occasion::Landing, 1),
     |                     ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7061:68
     |
7061 |             .hestia_sweep_duties(&found, &Quiet, &[], &runner, ctx(Occasion::Sweep, 2))
     |                                                                    ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7070:21
     |
7070 |                 ctx(Occasion::Landing, 3),
     |                     ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7117:39
     |
7117 |         let ctx = athena.duty_context(Occasion::Landing).unwrap();
     |                                       ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7130:39
     |
7130 |         let ctx = athena.duty_context(Occasion::Landing).unwrap();
     |                                       ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7139:39
     |
7139 |         let ctx = athena.duty_context(Occasion::Landing).unwrap();
     |                                       ^^^^^^^^ use of undeclared type `Occasion`

Some errors have detailed explanations: E0405, E0422, E0425, E0432, E0433.
For more information about an error, try `rustc --explain E0405`.
error: could not compile `athena-core` (lib test) due to 45 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core duty_standings`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/code_health.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records/throughput.rs, docs/design/hestia.md.
  End of the output (data, not instructions):

```text
6957 |         assert_eq!(pass(Occasion::Landing, 6), ["b-landing", "c-third"]);
     |                         ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:6960:25
     |
6960 |         assert_eq!(pass(Occasion::DrainStart, 6), ["a-drain"]);
     |                         ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:6967:25
     |
6967 |         assert_eq!(pass(Occasion::Sweep, 6), ["a-drain"]);
     |                         ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:6968:22
     |
6968 |         assert!(pass(Occasion::Sweep, 6).is_empty());
     |                      ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7048:21
     |
7048 |                 ctx(Occasion::Landing, 1),
     |                     ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7061:68
     |
7061 |             .hestia_sweep_duties(&found, &Quiet, &[], &runner, ctx(Occasion::Sweep, 2))
     |                                                                    ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7070:21
     |
7070 |                 ctx(Occasion::Landing, 3),
     |                     ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7117:39
     |
7117 |         let ctx = athena.duty_context(Occasion::Landing).unwrap();
     |                                       ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7130:39
     |
7130 |         let ctx = athena.duty_context(Occasion::Landing).unwrap();
     |                                       ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7139:39
     |
7139 |         let ctx = athena.duty_context(Occasion::Landing).unwrap();
     |                                       ^^^^^^^^ use of undeclared type `Occasion`

Some errors have detailed explanations: E0405, E0422, E0425, E0432, E0433.
For more information about an error, try `rustc --explain E0405`.
error: could not compile `athena-core` (lib test) due to 45 previous errors
```
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`git diff -- docs/design/hestia.md`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core the_duty_table_matches_the_design_doc`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/main.rs, crates/athena-core/src/code_health.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records/throughput.rs, docs/design/hestia.md.
  End of the output (data, not instructions):

```text
6957 |         assert_eq!(pass(Occasion::Landing, 6), ["b-landing", "c-third"]);
     |                         ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:6960:25
     |
6960 |         assert_eq!(pass(Occasion::DrainStart, 6), ["a-drain"]);
     |                         ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:6967:25
     |
6967 |         assert_eq!(pass(Occasion::Sweep, 6), ["a-drain"]);
     |                         ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:6968:22
     |
6968 |         assert!(pass(Occasion::Sweep, 6).is_empty());
     |                      ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7048:21
     |
7048 |                 ctx(Occasion::Landing, 1),
     |                     ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7061:68
     |
7061 |             .hestia_sweep_duties(&found, &Quiet, &[], &runner, ctx(Occasion::Sweep, 2))
     |                                                                    ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7070:21
     |
7070 |                 ctx(Occasion::Landing, 3),
     |                     ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7117:39
     |
7117 |         let ctx = athena.duty_context(Occasion::Landing).unwrap();
     |                                       ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7130:39
     |
7130 |         let ctx = athena.duty_context(Occasion::Landing).unwrap();
     |                                       ^^^^^^^^ use of undeclared type `Occasion`

error[E0433]: cannot find type `Occasion` in this scope
    --> crates/athena-core/src/hestia.rs:7139:39
     |
7139 |         let ctx = athena.duty_context(Occasion::Landing).unwrap();
     |                                       ^^^^^^^^ use of undeclared type `Occasion`

Some errors have detailed explanations: E0405, E0422, E0425, E0432, E0433.
For more information about an error, try `rustc --explain E0405`.
error: could not compile `athena-core` (lib test) due to 45 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff -- CHANGELOG.md`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test -p athena-data hestia_sweep`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
