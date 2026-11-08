# Job 20261027-082810-q9j5: verify "Stop a retry lineage after 3 failed attempts and queue a narrower"

## Request (verbatim)

> Add a documented default: once one retry lineage has 3 rejected or failed attempts, Athena stops retrying it. It records the findings that repeated and queues one narrower re-scope or split request in place of another retry. The operator should no longer have to stop or split lineages by hand. Add a test showing the 4th attempt is not dispatched.
> 
> Acceptance:
> - The change is made as described: No limit on attempts per lineage: the operator stopped and split by hand lineages that ran 3 to 5 rejected attempts
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/interventions/2026-10/20261026-180231-tf2p.json

## Reading (accepted by the user)

Add a documented default: when a retry lineage reaches 3 rejected or failed attempts, Athena dispatches no 4th attempt, records the repeated findings, and queues one narrower re-scope/split request instead. Goal: the maintainer no longer stops or splits runaway lineages by hand; the audience is the maintainer and the verifier, who check a test showing the 4th attempt is not dispatched.

Assumptions:
- Code survey: read crates/athena-core/src/reading/retry.rs (earlier_attempt, retries_by_job, job_history, retried_job) and the crates/athena-cli/src/drain.rs module doc and Steps/RunStep types; the drain today gives each item two attempts and a retry is a new job linked by 'try again on <job>'. The new cap counts rejected or failed jobs along that chain, so it is measured directly on job records, not through a stand-in. No new record file is added under crates/, so check_record_file/AnyRecord are untouched.
- Existing code: extends crates/athena-core/src/reading/retry.rs, crates/athena-cli/src/drain.rs and defaults.toml; reuses the finding text from retry_findings in retry.rs.
- The cited intervention record is not readable here; I take its stated facts from the request: lineages ran 3 to 5 rejected attempts before manual stop or split.
- The limit applies across manual and drain retries of one lineage; 'attempt' means a job ending rejected or failed.
- The replacement request is queued through the drain's existing request queue, once per lineage (no duplicates on later passes).
- Term lineage: a chain of jobs where each is a retry of the previous one.

## Acceptance criteria

1. [content] defaults.toml has a new entry (own id, statement, source naming records/interventions/2026-10/20261026-180231-tf2p.json) stating: at 3 rejected or failed attempts in one retry lineage, Athena stops retrying, records the repeated findings and queues one narrower re-scope or split request; and states whether it is enforced by mechanism.
2. [content] A lineage-length helper in crates/athena-core/src/reading/retry.rs (built on the existing retries_by_job/job_history chain) counts the rejected or failed attempts in a lineage, and the limit 3 is a named constant or config value, visible in the diff.
3. [behavior] A new test in crates/athena-cli/tests/drain.rs (or retry.rs) builds a lineage with 3 rejected attempts and shows the drain starts no 4th attempt; run `cargo test` and the named test passes.
4. [behavior] The same or a sibling test shows exactly one narrower re-scope/split request is queued in place of the retry, and that its text carries the findings that repeated across the attempts; run `cargo test`.
5. [content] docs (docs/design/async-drain.md or the retry section of the drain module doc in crates/athena-cli/src/drain.rs) documents the 3-attempt lineage limit and the replacement request; CHANGELOG.md has an entry.
6. [behavior] Lineages with fewer than 3 failed attempts behave as before: the existing tests in crates/athena-cli/tests/retry.rs and drain.rs pass unchanged; run `cargo test`.
7. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` all pass.
8. [content] The diff changes no file under records/ and no governance file other than defaults.toml.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/reading/retry.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- Changing the two-attempt per-item rule inside a single drain run
- Editing records/ or the intervention record itself
- Changing the thrash-stop default text beyond cross-referencing
- Prompt file changes

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The limit is enforced in drain code, not prose.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests exercise the 4th attempt not being dispatched.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog explain the limit.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to retry counting, drain and a default.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Caps the lineage, not the single instance.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 5)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A test in crates/athena-cli/tests/drain.rs parses the repository's defaults.toml with `DefaultsModel::parse(include_str!("../../../defaults.toml"))`. It asserts that `get("lineage-limit")` exists, its source contains `records/interventions/2026-10/20261026-180231-tf2p.json`, its statement contains `3 rejected or failed attempts`, `findings`, `narrower` and `enforced by mechanism`, and `render()` holds a line starting `- lineage-limit [*]: `. A second command shows the reader set still equals the recorded line. Run: `cargo test -p athena-cli --test drain the_lineage_limit_default_names_its_source_and_states_it_is_enforced_by_mechanism && cargo test -p athena-cli --test budgets`. Observes: The default parses with the new id, source and statement. `system_facts_budgets_match_computed` passes with docs/system-facts.md unchanged: the computed reader size is still 1997 of 2000.
  Fails without the behavior: Without the entry, `get("lineage-limit")` is None and the test fails. If the reader size drifted from 1997 because the entry is not offset by the `buy-first` split, `system_facts_budgets_match_computed` fails with the computed value in its message.
  Test seam: `athena_data::DefaultsModel::parse` over the real defaults.toml (exists; same include path as crates/athena-cli/tests/budgets.rs). The entry is added by plan step 10 and the test by step 13.
- Criterion 2: Unit tests in crates/athena-core/src/reading.rs `mod tests` (helpers `records_with`, `outcome`, `finding`, `two_majors_and_a_minor`) build in-memory `Records` with a chain j1 <- j2 <- j3 of "try again on" requests. They assert: `LINEAGE_ATTEMPT_LIMIT == 3`; `lineage_attempts` returns the failed ids oldest first from any job of the chain and from an unrun queued item that retries j3; a landed job, a dropped or stale retry and a job in another lineage are not counted; rejected, failed and timed-out each count; a job in no lineage returns empty; a cyclic request does not loop; `retries_by_job` returns the same map as before for the same records; `lineage_stops` over several jobs gives the same answers as `lineage_stop` job by job. Run: `cargo test -p athena-core lineage`. Observes: The constant is 3, and the count over the `job_history`/`retries_in` chain matches the records for each shape. The constant and helper are visible in the diff of retry.rs.
  Fails without the behavior: Without the helper the test module does not compile. With a counting bug (stale counted, forward walk missing, queued item not walking back) one shape assertion fails.
  Test seam: `athena_core::reading::lineage_attempts`, `lineage_stop(s)` and `retries_by_job` over in-memory `Records` built by the existing `records_with` helper (exists); the functions are added by plan steps 2 to 4 and the tests by step 5.
- Criterion 3: Three tests in crates/athena-cli/tests/drain.rs call `drain::drain(&real_steps(&athena, &lander), limits, &Lines)` over a `Scenario` whose verdict is `fail` with every criterion met and one major finding `the description lists no callers` (a Fail verdict or blocking finding gives final_verdict Fail, verify.rs:649-672; no unmet criterion, so the scope guard's T4 does not pause). (A) `a_lineage_with_three_rejected_attempts_gets_no_fourth_attempt`: run j1 by hand to Rejected; `athena.retry(j1)` and run to Rejected j2; queue `try again on <j2>` = q; drain with `max_jobs: 2`. q runs (its chain is 2, so `prepare` does not hold it) and is the 3rd failure. Assert j1, j2 and q are `Closed(Rejected)`; `lineage_attempts(&records, q)` is exactly [j1, j2, q]; no request job has text `try again on <q>`; `steps.paused(&[q])` is empty; nothing landed; the passed-over line names the lineage; no `over the job limit` line for q. (B) `a_queued_retry_of_a_lineage_with_three_failures_is_not_read_or_run`: hand-reject j1, j2 and j3, queue `try again on <j3>` = q; record `Scenario.invocations.lock().len()`; drain with `max_jobs: 2`. Assert the invocation count is unchanged (no Reader or other call), q has no job of its own and is `Closed(Dropped)` with a reason naming the narrower request, no job has text `try again on <q>`, and exactly one `plan       <q> passed over:` line names the lineage and the narrower request. (C) `a_lineage_that_reaches_three_failures_on_the_items_second_attempt_queues_the_narrower_request` (see criterion 4). Run: `cargo test -p athena-cli --test drain a_lineage_with_three_rejected_attempts_gets_no_fourth_attempt && cargo test -p athena-cli --test drain a_queued_retry_of_a_lineage_with_three_failures_is_not_read_or_run`. Observes: Exactly three rejected jobs in the lineage and none a retry of the third, with `max_jobs` 2 so the cap could not have stopped it (a 4th attempt would be job 2); the output says the lineage was stopped. A queued 4th is never read or run: zero new agent invocations and the item is dropped.
  Fails without the behavior: (A) fails without point 2 (after `failures += 1`): q's first failure leaves `failures` at 1, so the drain reads and runs `try again on <q>`, a job exists, and the lineage-equals-3 and no-retry-of-q assertions fail. (B) fails without the `prepare` hold: q is read for planning (an invocation is recorded), then run, so the invocation-count and `Closed(Dropped)` assertions fail. Plan step 15 removes each in turn and expects exactly these results.
  Test seam: Production entry `drain::drain` over `RealSteps` (exists; `real_steps`, `Lines`, `RecordingLander` exist in tests/drain.rs). The fake agent from `Fixture::athena(Scenario.script())` (tests/common/mod.rs:179,562, exists) only plays the rejecting verifier and holds no limit logic; its `invocations` log (exists, line 431) is read by the test. Hand attempts go through `Athena::go` and `Athena::retry` (exist). The all-met-plus-major verdict helper is added by step 13; the `Steps` methods, `ItemEnd::LineageStopped`, the `prepare` hold and point 2 by steps 6 to 9.
- Criterion 4: (a) `the_lineage_stop_queues_one_narrower_request_that_carries_the_repeated_findings` on fixture (A): load `Records`; select requests whose text contains `Lineage <j1>:`. Assert exactly one, channel `cli`, text not starting `try again on`, title starting `Re-scope or split `, containing `the description lists no callers` and the three failed job ids, `request_text_acceptance` non-empty, bullet count no larger than the root's (strictly smaller when the root has several) and none equal to a root bullet, and its `Files:` inside the root's touches. Then call `steps.lineage_stops(&[q])` and `steps.queue_narrower(&stop)` again on the same `RealSteps` and assert the returned id is the queued one and `fx.head()` is unchanged. (b) `a_lineage_that_reaches_three_failures_on_the_items_second_attempt_queues_the_narrower_request`: hand-reject j1, queue `try again on <j1>` = q, drain with `max_jobs: 2`. q runs (2nd failure), its retry r runs (3rd); point 2 stops before the `failures >= ATTEMPTS` return. Assert exactly one marker request, the output has no `failed twice`, no job retries r, and the passed-over line names the lineage. (c) In (B), call `lineage_stops(&[q])` and `queue_narrower` a second time after the drain and assert one marker request and an unchanged head (q is Dropped, so nothing is closed again). (d) A unit test in crates/athena-cli/src/drain.rs `mod tests` with a small local `Steps` stub modelled on the in-file `Stub` (line 4648), whose `lineage_stops` returns a stop for two item ids sharing one root and whose `queue_narrower` counts calls: calling `Drain::narrow` for both items leaves the count at 1 and both get the same result (finding 3). (e) Core unit tests in reading.rs `mod tests`: `repeated_findings_match_across_line_numbers_and_wording` (locations `src/a.rs:10` and `src/a.rs:42`; summaries `The description lists no callers.` and `description lists no callers`; listed as repeated, shown with the later wording); `findings_with_different_paths_or_unrelated_summaries_do_not_repeat` (same summary at `src/a.rs` and `src/b.rs`; same path with `the description lists no callers` vs `unit tests for the parser are missing`; and `Tests are missing` vs `Docs are missing` at one path, which must NOT repeat, finding 4); `unmet_criteria_repeat_across_case_digits_and_whitespace` plus an unrelated criterion; an unlocated finding repeats only on a Yes; a finding in 2 of 3 attempts is listed, one in 1 is not; the `none repeated` wording; a root with no files and no located findings still builds a request with files `records/`; the request is accepted by `request_text_acceptance`/`request_text_touches`; the marker lookup returns Queued; a root that is itself a `Re-scope or split ` request gives Refused and queues nothing. Run: `cargo test -p athena-cli --test drain the_lineage_stop_queues_one_narrower_request_that_carries_the_repeated_findings && cargo test -p athena-cli --test drain a_lineage_that_reaches_three_failures_on_the_items_second_attempt_queues_the_narrower_request && cargo test -p athena-cli --test drain a_queued_retry_of_a_lineage_with_three_failures_is_not_read_or_run && cargo test -p athena-cli --lib narrow_queues_once_per_lineage_root && cargo test -p athena-core lineage_stop`. Observes: One narrower, differently worded request carries the repeated finding; a direct second queue call adds none; two items of one lineage queue it once in the drain state. The core tests show repeats are found on reworded and renumbered input and are not found for unrelated findings or for `Tests are missing` vs `Docs are missing`.
  Fails without the behavior: Without queuing, the marker count of 1 fails. Without the marker lookup the second `queue_narrower` call writes a second request and the unchanged-head assertion fails. Without the `narrowed` guard the unit test (d) sees 2 calls. If findings were not carried or acceptance were copied from the root, the text and bullet assertions fail. Without point 2, (b) ends `ItemEnd::Failed` (drain.rs:1951-1954) with no marker request and says `failed twice`. With exact-equality matching the renumbered/reworded test fails; with the Maybe-based rule the `Tests/Docs are missing` pair fails; with a match-everything rule the contrasting-pair test fails. Without the second-generation rule the unit test queues a request. Drain test (a) uses the identical fake summary each time, so it shows carrying and queuing only; the matching is shown by the unit tests.
  Test seam: `RealSteps::lineage_stops`, `RealSteps::queue_narrower` and `RealSteps::drop_stopped` (added by steps 6 to 8) over a real `Athena` and fixture repository (exist); the stop and request text come from `athena_core::reading::lineage_stop(s)` (step 4) computed from the fixture's own records, using `athena_core::scope_guard::compare_criteria` and `normalise_criterion` (exist, pub at scope_guard.rs:190, 201). The fake agent only plays the rejecting verifier. For (d) the stub only supplies a stop and counts calls; `Drain::narrow`, whose once-per-root logic is under test, is the production code. The unit tests build `Records` with the existing helpers.
- Criterion 5: Read the diff of the documentation files. CHANGELOG.md has an Unreleased/Added entry naming the 3-attempt limit, the root cause, both stop points (the planning hold and the own-attempt stop), how repeats are matched (T4 for criteria, the stricter stop-word rule for findings), the narrower request and how it is narrowed, the drop of the tripping queued item, once-per-lineage queuing, the second-generation rule, the hand-retry override, the tests, the default id, that every reading made before the defaults edit is read again at its next run, and that the reader now sees a shorter first sentence for `buy-first` (a behavior change made to hold the reader set at 1997 of 2000). The drain.rs module doc has a bullet on the same. async-drain.md section 10's failure table has a row for it with a sentence below. Run: `git diff -- CHANGELOG.md docs/design/async-drain.md crates/athena-cli/src/drain.rs`. Observes: The three files hold the described entries, each stating the limit of 3, the narrower request and that a hand `athena retry` is not limited.
- Criterion 6: Run the whole suite, which includes the unchanged tests in crates/athena-cli/tests/retry.rs and drain.rs. Every existing drain test with fewer than 3 failed attempts in a lineage is unaffected: the in-file and tests/drain.rs stubs use the `Steps` defaults (`lineage_stops` empty, `drop_stopped` Ok(false)); the real-steps flag/pause tests have a lineage of 2. The new tests/retry.rs test `a_hand_retry_past_three_failed_attempts_is_still_allowed` builds three rejected attempts with `reject_then_ask_again`/`run_retry` and the all-met-plus-major verdict (no T4 pause), then calls `athena.retry` on the third and runs the proposal to a finished job. `scope_guard` tests also run because `compare_criteria` and `normalise_criterion` are called, not changed. Run: `cargo test --locked`. Observes: All tests pass; the diff of test files shows only added tests.
  Fails without the behavior: If the stop fired below 3 failures, `a_rejected_job_can_be_retried_twice` (tests/retry.rs) and the one-retry drain tests would fail. If a hand retry were limited, the new retry.rs test fails.
  Test seam: Existing tests with their existing fakes through `Athena::retry`, `Athena::go` and `drain::drain` (exist). The new retry.rs test is added in plan step 14.
- Criterion 7: Run the three checks Athena runs, in this order: fmt, clippy, test. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: Each exits 0: formatting clean, no clippy warning (no missed `ItemEnd::LineageStopped` arm, no unused item), and every test passes, including the new lineage tests, `system_facts_budgets_match_computed` against the unchanged facts line, and the defaults tests.
  Fails without the behavior: Unformatted code fails fmt; a missed `ItemEnd` arm or unused item fails clippy; a missing hold, stop or marker check, or a reader size off 1997, fails `cargo test --locked`.
  Test seam: Cargo over the whole workspace (exists). No fake or fixture.
- Criterion 8: List the changed files. No path under records/ appears, and the only governance file changed is defaults.toml. The eight paths are the eight listed deliverables; docs/system-facts.md, docs/design/context-engineer.md, athena.toml, workflows and tool or lint configuration are not touched. Run: `git diff --stat`. Observes: Eight paths, none under records/; defaults.toml is the only governance file.

Planned files:

- defaults.toml (modify)
- crates/athena-core/src/reading/retry.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 5813a704b74d → candidate 02e5afb228ee. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  24 ++
 crates/athena-cli/src/drain.rs          | 336 +++++++++++++++++++++-
 crates/athena-cli/tests/drain.rs        | 331 +++++++++++++++++++++
 crates/athena-cli/tests/retry.rs        |  42 +++
 crates/athena-core/src/reading.rs       | 465 ++++++++++++++++++++++++++++++
 crates/athena-core/src/reading/retry.rs | 489 +++++++++++++++++++++++++++++++-
 defaults.toml                           |   8 +-
 docs/design/async-drain.md              |   3 +
 8 files changed, 1688 insertions(+), 10 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-082810-q9j5/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-082810-q9j5/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42509421091 for 02e5afb228eea632e79e9bea6e0337d9d161272c)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42509421091 for 02e5afb228eea632e79e9bea6e0337d9d161272c)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42509421091 for 02e5afb228eea632e79e9bea6e0337d9d161272c)
- ci-result: `not a check` passed (Check results for 02e5afb228eea632e79e9bea6e0337d9d161272c come from the CI run https://github.com/doodla/athena/actions/runs/42509421091 (556s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy` and `cargo test` all pass.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test -p athena-cli --test drain the_lineage_limit_default_names_its_source_and_states_it_is_enforced_by_mechanism && cargo test -p athena-cli --test budgets`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core lineage`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/reading/retry.rs, defaults.toml, docs/design/async-drain.md.
  End of the output (data, not instructions):

```text
     |
2229 -             let NarrowerAction::Queue(text) = lineage_stop(&records, &lid(3)).unwrap().action
2229 +             let NarrowerAction::Queue(text) = lineage_job(&records, &lid(3)).unwrap().action
     |

error[E0425]: cannot find function `lineage_stop` in this scope
    --> crates/athena-core/src/reading.rs:2235:24
     |
1790 |         fn lineage_job(id: &str, request: &str, ended: Option<OutcomeState>) -> JobRecords {
     |         ---------------------------------------------------------------------------------- similarly named function `lineage_job` defined here
...
2235 |             let stop = lineage_stop(&lineage_records(jobs), &lid(3)).unwrap();
     |                        ^^^^^^^^^^^^
     |
help: a function with a similar name exists
     |
2235 -             let stop = lineage_stop(&lineage_records(jobs), &lid(3)).unwrap();
2235 +             let stop = lineage_job(&lineage_records(jobs), &lid(3)).unwrap();
     |

error[E0433]: cannot find type `NarrowerAction` in this scope
    --> crates/athena-core/src/reading.rs:2106:17
     |
2106 |             let NarrowerAction::Queue(text) = &stop.action else {
     |                 ^^^^^^^^^^^^^^ use of undeclared type `NarrowerAction`

error[E0433]: cannot find type `NarrowerAction` in this scope
    --> crates/athena-core/src/reading.rs:2161:17
     |
2161 |             let NarrowerAction::Queue(text) = stop.action else {
     |                 ^^^^^^^^^^^^^^ use of undeclared type `NarrowerAction`

error[E0433]: cannot find type `NarrowerAction` in this scope
    --> crates/athena-core/src/reading.rs:2187:17
     |
2187 |             let NarrowerAction::Queue(text) = stop.action else {
     |                 ^^^^^^^^^^^^^^ use of undeclared type `NarrowerAction`

error[E0433]: cannot find type `NarrowerAction` in this scope
    --> crates/athena-core/src/reading.rs:2212:17
     |
2212 |             let NarrowerAction::Queue(text) = lineage_stop(&records, &lid(3)).unwrap().action
     |                 ^^^^^^^^^^^^^^ use of undeclared type `NarrowerAction`

error[E0433]: cannot find type `NarrowerAction` in this scope
    --> crates/athena-core/src/reading.rs:2237:40
     |
2237 |                 matches!(&stop.action, NarrowerAction::Refused(why) if why.contains("narrower request")),
     |                                        ^^^^^^^^^^^^^^ use of undeclared type `NarrowerAction`

error[E0433]: cannot find type `NarrowerAction` in this scope
    --> crates/athena-core/src/reading.rs:2229:17
     |
2229 |             let NarrowerAction::Queue(text) = lineage_stop(&records, &lid(3)).unwrap().action
     |                 ^^^^^^^^^^^^^^ use of undeclared type `NarrowerAction`

Some errors have detailed explanations: E0425, E0433.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 38 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test -p athena-cli --test drain a_lineage_with_three_rejected_attempts_gets_no_fourth_attempt && cargo test -p athena-cli --test drain a_queued_retry_of_a_lineage_with_three_failures_is_not_read_or_run`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-cli --test drain the_lineage_stop_queues_one_narrower_request_that_carries_the_repeated_findings && cargo test -p athena-cli --test drain a_lineage_that_reaches_three_failures_on_the_items_second_attempt_queues_the_narrower_request && cargo test -p athena-cli --test drain a_queued_retry_of_a_lineage_with_three_failures_is_not_read_or_run && cargo test -p athena-cli --lib narrow_queues_once_per_lineage_root && cargo test -p athena-core lineage_stop`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 6 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/reading/retry.rs, defaults.toml, docs/design/async-drain.md.
  End of the output (data, not instructions):

```text
     |

error[E0425]: cannot find function `lineage_stop` in this scope
    --> crates/athena-core/src/reading.rs:2235:24
     |
1790 |         fn lineage_job(id: &str, request: &str, ended: Option<OutcomeState>) -> JobRecords {
     |         ---------------------------------------------------------------------------------- similarly named function `lineage_job` defined here
...
2235 |             let stop = lineage_stop(&lineage_records(jobs), &lid(3)).unwrap();
     |                        ^^^^^^^^^^^^
     |
help: a function with a similar name exists
     |
2235 -             let stop = lineage_stop(&lineage_records(jobs), &lid(3)).unwrap();
2235 +             let stop = lineage_job(&lineage_records(jobs), &lid(3)).unwrap();
     |

   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0433]: cannot find type `NarrowerAction` in this scope
    --> crates/athena-core/src/reading.rs:2106:17
     |
2106 |             let NarrowerAction::Queue(text) = &stop.action else {
     |                 ^^^^^^^^^^^^^^ use of undeclared type `NarrowerAction`

error[E0433]: cannot find type `NarrowerAction` in this scope
    --> crates/athena-core/src/reading.rs:2161:17
     |
2161 |             let NarrowerAction::Queue(text) = stop.action else {
     |                 ^^^^^^^^^^^^^^ use of undeclared type `NarrowerAction`

error[E0433]: cannot find type `NarrowerAction` in this scope
    --> crates/athena-core/src/reading.rs:2187:17
     |
2187 |             let NarrowerAction::Queue(text) = stop.action else {
     |                 ^^^^^^^^^^^^^^ use of undeclared type `NarrowerAction`

error[E0433]: cannot find type `NarrowerAction` in this scope
    --> crates/athena-core/src/reading.rs:2212:17
     |
2212 |             let NarrowerAction::Queue(text) = lineage_stop(&records, &lid(3)).unwrap().action
     |                 ^^^^^^^^^^^^^^ use of undeclared type `NarrowerAction`

error[E0433]: cannot find type `NarrowerAction` in this scope
    --> crates/athena-core/src/reading.rs:2237:40
     |
2237 |                 matches!(&stop.action, NarrowerAction::Refused(why) if why.contains("narrower request")),
     |                                        ^^^^^^^^^^^^^^ use of undeclared type `NarrowerAction`

error[E0433]: cannot find type `NarrowerAction` in this scope
    --> crates/athena-core/src/reading.rs:2229:17
     |
2229 |             let NarrowerAction::Queue(text) = lineage_stop(&records, &lid(3)).unwrap().action
     |                 ^^^^^^^^^^^^^^ use of undeclared type `NarrowerAction`

Some errors have detailed explanations: E0425, E0433.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 38 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
