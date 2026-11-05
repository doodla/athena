# Job 20261017-205547-cgzb: verify "Refuse a second retry of an interrupted job; show "retried as" in status"

## Request (verbatim)

> Stop `athena retry` from duplicating work on an interrupted job that was already retried, and test the restart case. After `athena retry <id>` on an interrupted job, the old job stays dispatched, `athena status` still tells the user to run `athena retry <id>` (crates/athena-cli/src/status.rs, the interrupted hint), and retry does not check for an existing retry, so following the hint starts the same work twice. Also, no test covers what a container restart leaves behind: a registry entry whose process is dead; the reap() calls in crates/athena-core/src/reading.rs and crates/athena-cli/src/drain.rs are what turn such a job into interrupted. Found by an independent spot-check of 8c20336.
> Plan: (1) A retry job's request names the job it retries ("try again on <id>"); use that to find an existing retry. (2) `athena status` shows "retried as <new id>" for such a job instead of the retry hint. (3) `athena retry <id>` refuses when a retry already exists, naming it. (4) Add a test that registers a process id that is not running for a dispatched job and shows the job is reported interrupted and is accepted by retry.
> Acceptance:
> - cargo test: after one retry of an interrupted job, a second `athena retry` of the same job fails with a message naming the first retry, and no new job is created.
> - cargo test: `athena status` for the retried job shows "retried as <new id>" and no longer suggests `athena retry`.
> - cargo test: a dispatched job whose registered process is dead is listed as interrupted and retry accepts it.
> - Existing retry and drain tests pass unchanged; clippy with -D warnings and fmt --check are clean; CHANGELOG.md has an entry.
> Out of scope: closing or rewriting the old job's records.

## Reading (accepted by the user)

For an interrupted (dispatched, no live process) job that already has a retry job ("try again on <id>"), `athena retry` refuses naming the existing retry, and `athena status` shows "retried as <new id>" instead of the `athena retry` hint. Adds tests for the double retry and for a restart leaving a dead registry entry. Audience: the user following status hints and the reviewer of 8c20336.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (Athena::retry at ~1101 calls registry().reap(), then resumable_job, then ask; earlier_attempt_with at ~1327 decides interrupted = Dispatched && !running; retried_job and job_history at ~1497-1535 already map a request 'try again on <id>' to the job it retries via records.resolve), crates/athena-core/src/status.rs (running() at 1646 checks registry entries; state_name 1650 yields 'interrupted'; status() reaps then builds the 'next' hints, the interrupted hint at ~1791), crates/athena-agent/src/procs.rs (reap at 177 removes dead groups), and crates/athena-cli/tests/retry.rs (interrupted() helper ~401, running-job test ~442 registers a live sleeper). The status code is in athena-core, not athena-cli as the request says.
- Existing code: reuse reading.rs retried_job / job_history / retry_reference and the RETRY_PREFIX request form; extend status.rs state_name/next hints and Athena::retry. No new record or data file is added, so AnyRecord and check_record_file are unaffected.
- Assumption: the refusal applies only when the job being retried is interrupted (as the request describes); retries of rejected or failed jobs behave as today so existing retry tests pass unchanged. An existing retry is any request job whose text resolves to the old job id, other than itself.
- Assumption: 'retried as <new id>' replaces the retry hint for that job in the status 'next' list and shows in the job line/state text; the close hint may remain. If several retries exist, the newest is named.
- Assumption: the dead-process test registers a pid of an exited process through the registry for a dispatched job, calls status/list and retry, and does not touch a real container.
- Refusal rule: this refuses more retries only for interrupted jobs already retried; expected 5% of newly refused readings (rare, duplicate retries only).

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test retry`: a test retries an interrupted job once, then a second `athena retry` of the same job returns an error whose message contains the first retry's job id, and the count of jobs in the records is unchanged after the second attempt.
2. [behavior] Run `cargo test`: a test shows `athena status` for the retried interrupted job contains 'retried as <new id>' and its next-step list contains no `athena retry <old id>` line.
3. [behavior] Run `cargo test`: a test registers a process id that is not running for a dispatched job; status lists the job as 'interrupted' and `athena retry` accepts it and creates a proposal.
4. [behavior] Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: all pass, with the existing retry.rs and drain.rs tests unmodified in their assertions.
5. [content] Athena::retry in crates/athena-core/src/reading.rs finds an existing retry through retried_job/job_history (the 'try again on <id>' form) rather than a new ad hoc parse, and the refusal text names the existing retry's id.
6. [content] CHANGELOG.md has an entry describing the duplicate-retry refusal, the 'retried as' status line and the restart test.
7. [content] The change refuses more retries only for already-retried interrupted jobs; expected 5% of newly refused readings.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/drain.rs (modify)

## Out of scope

- Closing or rewriting the old job's records
- Changing retry behavior for rejected, failed or timed-out jobs
- Governance files (athena.toml, defaults.toml, workflows)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test, clippy -D warnings and fmt --check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests exercising double retry, status text and dead-process interruption.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name commands the verifier can run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check lives in retry itself, not only in the status hint.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in reading.rs, status.rs and tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Messages and changelog written for outside readers.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Outcome is that following the hint no longer duplicates work.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration test. `setup(&Model::default())`, `interrupted(&fx,&athena,"remove the readme")`, `athena.retry(&job,&Lines::default())` once (Ok, job N). Record `Records::load(&fx.git,&fx.head()).unwrap().request_jobs().len()` and `fx.head()`. Call `athena.retry(&job, ..)` again. Run: `cargo test -p athena-cli --test retry a_second_retry_of_an_interrupted_job_is_refused_naming_the_first`. Observes: The second call is Err. The message contains N (the first retry's full id), the old job id, and 'already retried'. `request_jobs().len()` and `fx.head()` are equal before and after the second call, so no job or record was created.
  Fails without the behavior: Without the check in `Athena::retry` the second call returns Ok with a new proposal; `unwrap_err` panics and the job count rises by one.
  Test seam: Production entry point `Athena::retry` (reading.rs 1101), which reads the registry (`running`) and the records at HEAD. Fixture `setup` with the scripted `Model` and the `interrupted()` helper exist today (retry.rs 102, 401-415). No double is added.
- Criterion 2: (a) retry.rs `status_shows_retried_as_and_no_retry_hint`: make an interrupted job, retry it once (job N, left proposed), call `athena.status()`. (b) retry.rs `status_names_the_follow_up_of_a_rejected_retry`: reject the retry, then check the status line. (c) display.rs unit test `status_line_shows_retried_as`: build a `JobLine` with `retried_as: Some(..)` and call `plan_suffix`. Run: `cargo test -p athena-cli retried_as`. Observes: (a) The old job is in `open` with state 'interrupted' and `retried_as == Some(N)`; one `next` entry contains `retried as N` and `athena go N`; no `next` entry contains `athena retry <old>`. (b) the entry names `athena retry <new>`, not the old id. (c) The suffix contains ` · retried as N`; the existing suffix assertions (display.rs ~2047-2051) are unchanged.
  Fails without the behavior: Without the status change `retried_as` is None and `next` still holds `athena retry <old>`; the display test fails without the suffix.
  Test seam: Production entry points `Athena::status()` (status.rs 1671) and `plan_suffix` (display.rs 888). Fixture and `interrupted()` exist; the `JobLine.retried_as` field, `retries_by_job` and `retry_follow_up` are added by plan steps 2-5.
- Criterion 3: retry.rs `a_dispatched_job_with_a_dead_registered_process_is_interrupted_and_accepted_by_retry`: make an interrupted job, spawn `true` in its own process group, `wait()`, register that pid for the job, assert `athena.running(&job)` (the dead entry is present), call `athena.status()`, then `athena.retry`. tests/drain.rs `a_job_with_a_dead_registered_process_is_listed_and_retried_by_the_drain`: same registration after `interrupt(&fx,&athena,&cli)`, then `real_steps(&athena,&lander).queue()` and `drain::drain(&steps, limits(100.0), &Quiet)`. Run: `cargo test -p athena-cli --test retry a_dispatched_job_with_a_dead_registered_process && cargo test -p athena-cli --test drain a_job_with_a_dead_registered_process_is_listed_and_retried_by_the_drain`. Observes: retry.rs: after `status()` the job is in `open` with state 'interrupted' and the registry has no entry; `retry` returns Ok with a reading and the new job's request is `try again on <job>`. drain: `queue().items` lists the job with `interrupted == true`; the report has a 'passed' line for a new job whose request is `try again on <cli>`; the registry has no entry for the job afterwards.
  Fails without the behavior: Without the `reap()` calls in `status()`, `retry()` and the drain's queue, the dead entry counts as running: status says 'running', retry fails with 'is running', and the drain lists nothing. These tests pass on the base code for the same reason as the existing live-process test: they are regression tests for the restart case, which had none.
  Test seam: Entry points `Athena::status`, `Athena::retry` (retry.rs) and `drain::drain` with `real_steps` (tests/drain.rs 4124, `RealSteps::queue` at drain.rs 3496 calls `registry().reap()`). The fixture is a really exited child registered through `athena.state.registry().register`; mirrors the live-sleeper test at retry.rs 442-465. All seams exist today; the tests are added by steps 6 and 7.
- Criterion 4: Run the three checks Athena runs. Then `git diff -- crates/athena-cli/tests/retry.rs crates/athena-cli/tests/drain.rs` to see that only new tests were appended and no existing assertion line changed. Includes the new drain test `an_interrupted_job_whose_retry_gave_no_reading_is_read_again_not_retried_twice` and the existing `an_interrupted_job_is_retried_first_and_then_run` (4833), `drain_lists_and_retries_an_interrupted_job_and_leaves_one_on_another_channel_alone` (4881) and `a_reader_that_gives_no_reading_is_read_once_more_then_skipped_with_its_error` (3495). Run: `cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`. Observes: All three exit 0. The new drain test: `interrupt()` a job, then arm a reader that returns no structured reading once (a wrapper around `killing_producer` with an `AtomicBool` armed after the interrupt); run `drain::drain`. The retry job is recorded unread, then read again through `go`: exactly one job has the request `try again on <cli>`, the report has a 'passed' line for it, and `report.landed == 1`.
  Fails without the behavior: Without the drain change the second attempt calls `athena.retry(<old>)` again and is refused (item ends failed, nothing landed); without the guard it would create a second retry job. The test fails in both cases.
  Test seam: Whole-workspace checks, plus `drain::drain` with `real_steps` (exists, tests/drain.rs 4124) and `reader_fails_for_first_request`-style script wrapping (pattern at 3476-3492; the armed-once reader script is a new helper added by step 7). No logic is placed in a double: the script only withholds the reader's structured output.
- Criterion 5: Read the diff. Run: `git diff -- crates/athena-core/src/reading.rs`. Observes: `retries_by_job` iterates `job_history(records)`, which uses `retried_job`; the diff has no new use of `RETRY_PREFIX` or `retry_reference` for parsing, and no call to `Records::resolve` on request text. `Athena::retry` calls `retries_by_job` and the `bail!` text contains the retry's id and state. It also shows the helper skips retries closed unrun (finding 1).
- Criterion 6: Read the diff. Run: `git diff -- CHANGELOG.md`. Observes: An entry under [Unreleased] (the Fixed list, matching its neighbours) that names: the refusal of a second `athena retry` of an interrupted job (message names the first retry), 'retried as <id>' in status in place of the retry hint, a retry closed unrun not counting, the drain reading an unread retry job again, and the new restart tests with a dead registry entry.
- Criterion 7: Run the corpus (untouched) and the new test that rejects a job, retries it twice and expects both retries to be accepted; run `cargo test -p athena-cli --test retry a_dropped_retry_does_not_block_a_new_retry` (retry an interrupted job, `athena.close(<retry>, "...", ..)` while proposed, retry the old job again). Run: `cargo test -p athena-core --test refusal_corpus && cargo test -p athena-cli --test retry a_rejected_job_can_be_retried_twice`. Observes: The corpus passes unchanged. The rejected-job double retry returns Ok twice. After the retry is dropped, retrying the old job is Ok and status shows `athena retry <old>` again. So the only new refusals are interrupted jobs with a standing retry. I cannot run these from the plan; no result is cited. Input forms and cases (tests/refusal_corpus.rs): this change adds no text rule and keeps `retry_reference` and `RETRY_PREFIX`, so no corpus case moves; the nearest cases are `retry-complaint-reading-names-the-job` and `retry-complaint-reading-omits-the-job` (lines ~351 and ~369), which concern a reading's text. Forms of the new guard and their tests: interrupted with no retry (retry.rs 418, existing); interrupted with a standing retry (new double-retry test); interrupted with a dropped retry (new test); interrupted and running (retry.rs 442); interrupted with a dead registered process (new); rejected or failed job retried twice (new); retry of a retry (guard keys on the job being retried; the status (b) test retries a rejected retry).
  Fails without the behavior: If the guard applied to rejected or failed jobs, `a_rejected_job_can_be_retried_twice` fails; if dropped retries counted, `a_dropped_retry_does_not_block_a_new_retry` fails; if the reading gate changed, a corpus expectation moves.
  Test seam: Entry points `Athena::retry` and `Athena::status` (exist); fixtures as for criterion 1; `reject(..)` helper exists (retry.rs 114); `Athena::close` exists (manage.rs 74).

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 342a3c185a1b → candidate 8f506c9d6ff6. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  16 +++
 crates/athena-cli/src/display.rs  |  23 ++++-
 crates/athena-cli/src/drain.rs    |  30 ++++--
 crates/athena-cli/tests/drain.rs  | 103 ++++++++++++++++++++
 crates/athena-cli/tests/retry.rs  | 199 ++++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/reading.rs |  77 ++++++++++++++-
 crates/athena-core/src/status.rs  |  28 +++++-
 7 files changed, 463 insertions(+), 13 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261017-205547-cgzb/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261017-205547-cgzb/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42000195278 for 8f506c9d6ff6525e4d25e96e49162b252363ab0a)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42000195278 for 8f506c9d6ff6525e4d25e96e49162b252363ab0a)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42000195278 for 8f506c9d6ff6525e4d25e96e49162b252363ab0a)
- ci-result: `not a check` passed (Check results for 8f506c9d6ff6525e4d25e96e49162b252363ab0a come from the CI run https://github.com/doodla/athena/actions/runs/42000195278 (602s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: all pass, with the existing retry.rs and drain.rs tests unmodified in their assertions.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --test retry a_second_retry_of_an_interrupted_job_is_refused_naming_the_first`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/drain.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/status.rs.
  End of the output (data, not instructions):

```text
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling toml v0.9.12+spec-1.1.0
   Compiling jiff v0.2.38
   Compiling rustix v1.1.5
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
   Compiling hashbrown v0.17.1
   Compiling anyhow v1.0.104
   Compiling dyn-clone v1.0.20
   Compiling equivalent v1.0.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling schemars v1.2.2
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling utf8parse v0.2.2
   Compiling anstyle-parse v1.0.0
   Compiling syn v2.0.119
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle v1.0.14
   Compiling anstyle-query v1.1.5
   Compiling colorchoice v1.0.5
   Compiling anstream v1.0.0
   Compiling clap_lex v1.1.1
   Compiling heck v0.5.0
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling strsim v0.11.1
   Compiling clap_builder v4.6.7
   Compiling clap_derive v4.6.7
   Compiling clap v4.6.7
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0609]: no field `retried_as` on type `&athena_core::status::JobLine`
   --> crates/athena-cli/tests/retry.rs:959:21
    |
959 |     assert_eq!(line.retried_as.as_deref(), Some(retry.as_str()));
    |                     ^^^^^^^^^^ unknown field
    |
    = note: available fields are: `id`, `state`, `title`, `class`, `cost_usd` ... and 3 others

For more information about this error, try `rustc --explain E0609`.
error: could not compile `athena-cli` (test "retry") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli retried_as`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/drain.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/status.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0560]: struct `athena_core::status::JobLine` has no field named `retried_as`
    --> crates/athena-cli/src/display.rs:2047:13
     |
2047 |             retried_as: None,
     |             ^^^^^^^^^^ `athena_core::status::JobLine` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `athena_core::status::JobLine` has no field named `retried_as`
    --> crates/athena-cli/src/display.rs:2067:13
     |
2067 |             retried_as: Some("20260101-000000-abcd".into()),
     |             ^^^^^^^^^^ `athena_core::status::JobLine` does not have this field
     |
     = note: all struct fields are already assigned

For more information about this error, try `rustc --explain E0560`.
error: could not compile `athena-cli` (lib test) due to 2 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test -p athena-cli --test retry a_dispatched_job_with_a_dead_registered_process && cargo test -p athena-cli --test drain a_job_with_a_dead_registered_process_is_listed_and_retried_by_the_drain`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test --locked && cargo clippy --all-targets --locked -- -D warnings && cargo fmt --all --check`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test -p athena-core --test refusal_corpus && cargo test -p athena-cli --test retry a_rejected_job_can_be_retried_twice`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
