# Job 20261030-060410-mavp: verify "Defer introspections while the kill switch is on; run on start"

## Request (verbatim)

> When the kill switch is on, have Athena defer an introspection (keep it pending and run it after `athena start`) instead of starting the model and recording status failed. Rerun the two introspections that already failed this way, for jobs 20261029-142858-5c0p and 20261017-155259-7mjc. I read both records: each failed with 'the kill switch is on', and neither job has any other introspection. Acceptance: a test shows an introspection triggered while the kill switch is on is left pending, not failed, and runs once the switch is off; fmt, clippy -D warnings and the tests pass.
> 
> Acceptance:
> - The change is made as described: Introspections start while the kill switch is on, are recorded as failed, and are never rerun, so two failed jobs have no diagnosis
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/introspections/2026-10/20261030-013540-p86y.json

## Reading (accepted by the user)

When the kill switch is on, `Athena::introspect` queues the event in the state directory instead of invoking the model and committing a failed record. `athena start` then runs the queued introspections. This also reruns the two kill-switch failures, for jobs 20261029-142858-5c0p and 20261017-155259-7mjc, through that queue.

Assumptions:
- Code survey: I read crates/athena-core/src/introspect.rs. `introspect_job` and `introspect_intervention` go through `introspect`, which builds a Failed record and commits it even when `driver.invoke` errors. I read crates/athena-core/src/manage.rs, where `start()` calls `state.registry().start()`, and crates/athena-cli/src/commands.rs, where `start()` is the `athena start` entry point. I read crates/athena-data/src/records/introspection.rs, where `IntrospectionStatus` has only Recorded and Failed. `registry().stopped()` is already used in status.rs. The existing test file is crates/athena-cli/tests/introspection.rs. Measurement is direct: a test with the kill switch on and a fake driver counts model calls and committed records.
- Design: no new record status. A deferred event is a pending file under the state root (`introspect-pending/`), not a record, so the records allow-list, `AnyRecord`, `check_record_file` and the round-trip test are untouched. `athena start`, only when the switch was on, drains the queue by calling `introspect` per event and deletes each entry after it is committed. An entry whose run fails stays recorded as failed, as today, and is not requeued, so a failing item cannot loop.
- Interpretation: the 'kill switch is on' check is read from `state.registry().stopped()` before the model is invoked. The same deferral applies to job and intervention events. A trigger that is already queued is not queued twice.
- Term pending: an introspection event that was triggered, has no introspection record yet, and is waiting in the state-directory queue.
- Retry of 20261029-142858-5c0p: its planning stopped because the kill switch was turned on, so it produced no candidate and has no files to keep. It is addressed by queueing its introspection for rerun, not by redoing its build work. Retry of 20261017-155259-7mjc: it was rejected because `cargo test --locked` failed with no exit code. Its introspection is rerun the same way. Its seven changed CI-routing files are dropped and excluded in out_of_scope.
- The rerun of the two failed introspections needs the model, which the producer cannot call. It is done by the owner running `athena start` after the change lands, and the change provides a way to enqueue those two events. The producer does not edit records/ (records/introspections/2026-10/20261030-013540-p86y.json is read only), and the old failed records stay as history.
- Environment: a local state directory, read and write for the producer. No network, push, PR or CI workflow is used. The tests use the fake driver in crates/athena-agent, and git commits are made inside the test repository only.
- Consumers: none. No config file changes.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/introspection.rs triggers `introspect_job` (a rejected or failed job) while the kill switch is on. It asserts that the fake driver was not invoked, no introspection record was committed with status failed, and one event is pending. Evidence: `cargo test` shows it passing.
2. [behavior] The same test, or a second one, turns the switch off through the real `start` path (`Athena::start` or the `athena start` command). It asserts that the pending event now yields exactly one committed introspection record with status recorded, and that the queue is then empty. Evidence: `cargo test`.
3. [behavior] A test covers an intervention-triggered introspection while the switch is on. It is deferred and then run on start, and a second `athena start` does not run it again. Evidence: `cargo test`.
4. [content] The diff to crates/athena-core/src/introspect.rs (and the `start` path in crates/athena-core/src/manage.rs or crates/athena-cli/src/commands.rs) shows the stopped check happening before `driver.invoke`, with a queue of pending events and a drain run when the switch is turned off.
5. [content] CHANGELOG.md has a new entry saying that introspections are deferred while the kill switch is on and run after `athena start`.
6. [behavior] No unrelated behavior changes. The existing tests in crates/athena-cli/tests/introspection.rs and the other crates pass unchanged, and `the_kill_switch_blocks_new_work` still passes. Evidence: `cargo test`.
7. [behavior] `cargo fmt --check`, `cargo clippy` (with -D warnings as configured) and `cargo test` pass, as recorded by Athena on the final tree.
8. [content] The change does not edit anything under records/, docs/decisions.md or athena.toml. If a decision is recorded, it is a new appended decision with a Provenance bullet.

## Expected deliverables

- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-cli/src/commands.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Any edit to records/, including records/introspections/2026-10/20261030-013540-p86y.json. The old failed records stay.
- The seven files the rejected job 20261017-155259-7mjc changed (ci_result.rs, gh.rs, routing.rs, run.rs, run_ci_tests.rs, docs/design/ci-routing.md, CHANGELOG.md CI entry) are dropped, not kept.
- The build work of job 20261029-142858-5c0p (per-job worktree start, disk bound). It produced no candidate and is not redone here.
- Changing how the kill switch blocks agents and checks, or deferring anything other than introspections.
- A new IntrospectionStatus variant or any change to the record schema.
- Changes to athena.toml, defaults.toml or .github/workflows.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new passing tests plus cargo test and clippy
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria check pending-then-run behavior through the real start path
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change, with no new record type
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The deferral is enforced in code and tested, not left as prose
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test `an_introspection_triggered_while_the_kill_switch_is_on_is_left_pending` in crates/athena-cli/tests/introspection.rs. Set up with `setup(Scenario::default(), &model)` after `model.say(reply(...))`. Commit a rejected job's records with the existing `chain_request`, `chain_candidate` and `chain_outcome(.., Rejected, ..)` through `commit_records`. Call `athena.stop()`, then `athena.introspect_job(&job, &Lines::default())`. Assert: result is None; `introspections(&fx)` is empty (none with status failed); `model.prompts` is empty and the queued reply is unused; `athena.pending_introspections()` has length 1; the `Lines` output has a 'deferred' line. Run: `cargo test --locked -p athena-cli --test introspection an_introspection_triggered_while_the_kill_switch_is_on_is_left_pending`. Observes: The test passes: no record is committed, the scripted model saw no prompt, and exactly one event is pending.
  Fails without the behavior: Without the deferral `introspect` runs: the fake driver returns `DriverError::Stopped` (fake.rs:92) and a failed record 'the model could not run: the kill switch is on' is committed (introspect.rs:1434), so the empty-records and one-pending asserts fail.
  Test seam: Production entry point `Athena::introspect_job` (introspect.rs:1272), with the switch set by the real `Athena::stop` (manage.rs:499) through the registry file. The `Model` scripted driver and `Fixture` exist (introspection.rs:37-105) and are injected through `fx.athena(model.script(..))` with `FakeAgentDriver::with_registry` (common/mod.rs:179-182); exists. The accessor `pending_introspections` is added by step 3. The guard (`stopped()`) and the queue are production code; the double supplies only the model reply.
- Criterion 2: Second test `a_pending_introspection_runs_once_when_start_turns_the_switch_off`. After the deferral, call `athena.start_with(&ui)` (the real start path; `athena.start()` is the same with `Quiet`). Assert: it returns true; `introspections(&fx)` has exactly one record, status Recorded, with `job` equal to the rejected job and trigger Rejection; `model.prompts` has length 1; the pending list is empty. Then `athena.start()` again: record count still 1. Also run `athena_bin(&fx).args(["start"])` (introspection.rs:292) once against the same state to cover the command wiring in commands.rs. Run: `cargo test --locked -p athena-cli --test introspection a_pending_introspection_runs_once_when_start_turns_the_switch_off`. Observes: After start the single record is Recorded and the queue is empty. A second start adds nothing.
  Fails without the behavior: Without the drain, start only removes the stop file: the record count stays 0 and the pending list stays at 1, so the Recorded and empty-queue asserts fail.
  Test seam: Production entry points `Athena::start_with` / `Athena::start` (step 5) and, in the CLI check, `athena start` (commands.rs:649, wired in step 6). The same `Model`/`Fixture` seam as criterion 1 (exists); the scripted model replies once. `start_with` is added by step 5.
- Criterion 3: New test `an_intervention_introspection_is_deferred_run_on_start_and_not_run_twice`. Call `athena.stop()`, then the existing helper `intervene(&athena, "...")` (introspection.rs:122), which commits the intervention and calls `introspect_intervention` (manage.rs:400). Assert: the intervention is committed, no introspection record, one pending, `model.prompts` empty. Call `athena.start_with(&ui)`: one record, status Recorded, `intervention` equal to the returned id, trigger Intervention, queue empty. Call `athena.start()` again: it returns false, the record count is still 1 and `model.prompts` has length 1. Run: `cargo test --locked -p athena-cli --test introspection an_intervention_introspection_is_deferred_run_on_start_and_not_run_twice`. Observes: Deferred while stopped, recorded once on start, and a second start does not rerun it.
  Fails without the behavior: Without deferral the intervention gets a failed record while stopped (zero pending, one failed record). Without file deletion after the commit, the second start calls the model again and `model.prompts` is 2.
  Test seam: Production entry points `Athena::intervene` (manage.rs) leading to `introspect_intervention`, then `Athena::start_with` / `start`. Fixture, scripted model and `intervene` helper exist (introspection.rs:101-138); the pending accessor and `start_with` are added by steps 3 and 5.
- Criterion 4: Read the diff: `run_or_defer` in introspect.rs calls `self.state.registry().stopped()` before the call to `introspect`, so before `driver.invoke` (introspect.rs:1433). The queue is `StateDir::introspect_pending_dir` plus `defer`, `pending_introspections` and `run_pending_introspections`. `start_with` in manage.rs calls the drain after `registry().start()`, and `start()` in commands.rs calls `start_with`. The test `a_kill_switch_failed_introspection_is_requeued_by_start_and_rerun_once` seeds a failed record with the old kill-switch text and one with 'model unavailable', and shows that only the first is rerun. Run: `git diff crates/athena-core/src/introspect.rs crates/athena-core/src/manage.rs crates/athena-core/src/state.rs crates/athena-cli/src/commands.rs`. Observes: The stopped check sits before the model call in `run_or_defer`; a queue directory and a drain are present; `start_with` calls the drain. The behavior is shown by the tests of criteria 1-3 and by the requeue test (cargo test --locked -p athena-cli --test introspection a_kill_switch_failed_introspection_is_requeued_by_start_and_rerun_once).
- Criterion 5: Read CHANGELOG.md under `## [Unreleased]` / `### Added`: one new bullet saying introspections are deferred while the kill switch is on and run after `athena start`, that `athena start` also reruns an introspection that failed with 'the kill switch is on' and has no other introspection (the cases of jobs 20261029-142858-5c0p and 20261017-155259-7mjc), and that no record schema changed. Run: `git diff CHANGELOG.md`. Observes: A single added bullet in Unreleased naming the deferral and `athena start`.
- Criterion 6: Run the whole suite. The existing tests in introspection.rs, lifecycle.rs (`the_kill_switch_blocks_new_work`, 1719) and the other crates must pass unchanged. `a_failed_introspection_is_recorded_never_blocks_and_is_not_introspected` (introspection.rs:1032) runs with the switch off, so it still gets a failed record from 'model unavailable'. The diff of existing test bodies is empty; only new tests are added. Run: `cargo test --locked`. Observes: All tests pass.
  Fails without the behavior: A changed `start` signature would fail to compile at lifecycle.rs:1725; a deferral that also applied with the switch off would fail the failed-introspection test.
  Test seam: Existing tests; no seam added. `Athena::start()` keeps its signature (step 5), so lifecycle.rs:1725 compiles unchanged. Exists.
- Criterion 7: Run the three checks Athena runs on the final tree, in this order: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. The command given is the test run, which includes the three new behavior tests and the requeue test; the verifier also runs the other two as separate whole commands. The producer runs `cargo fmt --all` first. Run: `cargo test --locked`. Observes: fmt prints nothing and exits 0; clippy prints no warning and exits 0; test shows every test passing, including the new deferral, start-drain, intervention and requeue tests and the unchanged `the_kill_switch_blocks_new_work`.
  Fails without the behavior: Each check fails on a defect this change could carry. fmt fails on an unformatted file in the touched crates. Clippy `-D warnings` fails on an unused helper (for example `pending_introspections` or `defer` left unused) or a needless clone. `cargo test` fails if any new test is red: with no deferral, the criterion 1 and 3 tests see a failed record and zero pending; with no drain, the criterion 2 test sees zero records after start. It also fails if `start` changed signature, because lifecycle.rs:1725 would not compile.
  Test seam: The production gate commands Athena runs (cargo fmt, cargo clippy, cargo test over the workspace) run in the producer's working tree. No fake is injected by the command itself. The behavior under test is driven by the `Model` scripted driver and `Fixture` in crates/athena-cli/tests/introspection.rs (exists, lines 37-105), injected through `fx.athena(model.script(..))` with `FakeAgentDriver::with_registry` (common/mod.rs:179-182). The tests of criteria 1-3 are added by step 7; the fixture exists.
- Criterion 8: Inspect the diff and status: no path under records/, no athena.toml, no defaults.toml, no workflow. docs/decisions.md changes only by lines added after the last decision (D176 at line 5094), with a Provenance bullet. Run: `git status`. Observes: Modified paths are only the listed deliverables (introspect.rs, manage.rs, state.rs, commands.rs, tests/introspection.rs, CHANGELOG.md, docs/decisions.md). `git diff docs/decisions.md` shows only added lines at the end.

Planned files:

- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/commands.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8238eadfd32f → candidate 1cb91434285a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |   1 +
 crates/athena-cli/src/commands.rs        |   2 +-
 crates/athena-cli/tests/introspection.rs | 205 +++++++++++++++++++++++++++++++
 crates/athena-core/src/introspect.rs     | 173 +++++++++++++++++++++++++-
 crates/athena-core/src/manage.rs         |  13 +-
 crates/athena-core/src/state.rs          |   6 +
 docs/decisions.md                        |  34 +++++
 7 files changed, 428 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-060410-mavp/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-060410-mavp/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43390183538 for 1cb91434285ab39bbfd06636b94360bf8d032440)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43390183538 for 1cb91434285ab39bbfd06636b94360bf8d032440)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43390183538 for 1cb91434285ab39bbfd06636b94360bf8d032440)
- ci-result: `not a check` passed (Check results for 1cb91434285ab39bbfd06636b94360bf8d032440 come from the CI run https://github.com/doodla/athena/actions/runs/43390183538 (903s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy` (with -D warnings as configured) and `cargo test` pass, as recorded by Athena on the final tree.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli --test introspection an_introspection_triggered_while_the_kill_switch_is_on_is_left_pending`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/commands.rs, crates/athena-core/src/introspect.rs, crates/athena-core/src/manage.rs, crates/athena-core/src/state.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
1586 |     assert!(athena.start_with(&ui).unwrap(), "the switch was on");
     |                    ^^^^^^^^^^
     |
help: there is a method `start` with a similar name, but with different arguments
    --> crates/athena-core/src/manage.rs:504:5
     |
 504 |     pub fn start(&self) -> Result<bool> {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `pending_introspections` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1595:20
     |
1595 |     assert!(athena.pending_introspections().is_empty(), "queue is empty");
     |                    ^^^^^^^^^^^^^^^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no method named `pending_introspections` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1622:26
     |
1622 |     let pending = athena.pending_introspections();
     |                          ^^^^^^^^^^^^^^^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no method named `start_with` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1628:20
     |
1628 |     assert!(athena.start_with(&Lines::default()).unwrap());
     |                    ^^^^^^^^^^
     |
help: there is a method `start` with a similar name, but with different arguments
    --> crates/athena-core/src/manage.rs:504:5
     |
 504 |     pub fn start(&self) -> Result<bool> {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `pending_introspections` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1636:20
     |
1636 |     assert!(athena.pending_introspections().is_empty());
     |                    ^^^^^^^^^^^^^^^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no method named `start_with` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1699:21
     |
1699 |     assert!(!athena.start_with(&Lines::default()).unwrap());
     |                     ^^^^^^^^^^
     |
help: there is a method `start` with a similar name, but with different arguments
    --> crates/athena-core/src/manage.rs:504:5
     |
 504 |     pub fn start(&self) -> Result<bool> {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `pending_introspections` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1711:20
     |
1711 |     assert!(athena.pending_introspections().is_empty());
     |                    ^^^^^^^^^^^^^^^^^^^^^^ method not found in `athena_core::Athena`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `athena-cli` (test "introspection") due to 9 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli --test introspection a_pending_introspection_runs_once_when_start_turns_the_switch_off`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/commands.rs, crates/athena-core/src/introspect.rs, crates/athena-core/src/manage.rs, crates/athena-core/src/state.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
     |
1586 |     assert!(athena.start_with(&ui).unwrap(), "the switch was on");
     |                    ^^^^^^^^^^
     |
help: there is a method `start` with a similar name, but with different arguments
    --> crates/athena-core/src/manage.rs:504:5
     |
 504 |     pub fn start(&self) -> Result<bool> {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `pending_introspections` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1595:20
     |
1595 |     assert!(athena.pending_introspections().is_empty(), "queue is empty");
     |                    ^^^^^^^^^^^^^^^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no method named `pending_introspections` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1622:26
     |
1622 |     let pending = athena.pending_introspections();
     |                          ^^^^^^^^^^^^^^^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no method named `start_with` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1628:20
     |
1628 |     assert!(athena.start_with(&Lines::default()).unwrap());
     |                    ^^^^^^^^^^
     |
help: there is a method `start` with a similar name, but with different arguments
    --> crates/athena-core/src/manage.rs:504:5
     |
 504 |     pub fn start(&self) -> Result<bool> {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `pending_introspections` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1636:20
     |
1636 |     assert!(athena.pending_introspections().is_empty());
     |                    ^^^^^^^^^^^^^^^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no method named `start_with` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1699:21
     |
1699 |     assert!(!athena.start_with(&Lines::default()).unwrap());
     |                     ^^^^^^^^^^
     |
help: there is a method `start` with a similar name, but with different arguments
    --> crates/athena-core/src/manage.rs:504:5
     |
 504 |     pub fn start(&self) -> Result<bool> {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `pending_introspections` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1711:20
     |
1711 |     assert!(athena.pending_introspections().is_empty());
     |                    ^^^^^^^^^^^^^^^^^^^^^^ method not found in `athena_core::Athena`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `athena-cli` (test "introspection") due to 9 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli --test introspection an_intervention_introspection_is_deferred_run_on_start_and_not_run_twice`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/commands.rs, crates/athena-core/src/introspect.rs, crates/athena-core/src/manage.rs, crates/athena-core/src/state.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
     |
1586 |     assert!(athena.start_with(&ui).unwrap(), "the switch was on");
     |                    ^^^^^^^^^^
     |
help: there is a method `start` with a similar name, but with different arguments
    --> crates/athena-core/src/manage.rs:504:5
     |
 504 |     pub fn start(&self) -> Result<bool> {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `pending_introspections` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1595:20
     |
1595 |     assert!(athena.pending_introspections().is_empty(), "queue is empty");
     |                    ^^^^^^^^^^^^^^^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no method named `pending_introspections` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1622:26
     |
1622 |     let pending = athena.pending_introspections();
     |                          ^^^^^^^^^^^^^^^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no method named `start_with` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1628:20
     |
1628 |     assert!(athena.start_with(&Lines::default()).unwrap());
     |                    ^^^^^^^^^^
     |
help: there is a method `start` with a similar name, but with different arguments
    --> crates/athena-core/src/manage.rs:504:5
     |
 504 |     pub fn start(&self) -> Result<bool> {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `pending_introspections` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1636:20
     |
1636 |     assert!(athena.pending_introspections().is_empty());
     |                    ^^^^^^^^^^^^^^^^^^^^^^ method not found in `athena_core::Athena`

error[E0599]: no method named `start_with` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1699:21
     |
1699 |     assert!(!athena.start_with(&Lines::default()).unwrap());
     |                     ^^^^^^^^^^
     |
help: there is a method `start` with a similar name, but with different arguments
    --> crates/athena-core/src/manage.rs:504:5
     |
 504 |     pub fn start(&self) -> Result<bool> {
     |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `pending_introspections` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/introspection.rs:1711:20
     |
1711 |     assert!(athena.pending_introspections().is_empty());
     |                    ^^^^^^^^^^^^^^^^^^^^^^ method not found in `athena_core::Athena`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `athena-cli` (test "introspection") due to 9 previous errors
```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6, 7 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
