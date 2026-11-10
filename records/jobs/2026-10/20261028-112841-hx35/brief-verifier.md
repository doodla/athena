# Job 20261028-112841-hx35: verify "Keep partial work and retry once on producer 401 login expiry"

## Request (verbatim)

> Priority: high
> Treat an expired or failed model login as an environment stop. On 2026-10-28 a producer ran 30 minutes and then failed with 'API Error: 401 OAuth access token has expired'; the login refreshed by itself minutes later, but the job was recorded as failed and its partial work was not kept, so the retry started from scratch. Fix: recognise a 401 or authentication failure from the agent driver as an environment cause (like low disk and usage limits), keep the producer's partial work the way a timed-out producer's work is kept, and have the retry resume from it; retry the driver call once after a short wait before giving up.
> Acceptance:
> - A test with a fake driver returning the 401 text shows the job ends as an environment stop with the partial work kept, and athena retry resumes from it.
> - A test shows one retried driver call succeeds after a transient 401.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-agent/src/claude.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/run/mod.rs, CHANGELOG.md

## Reading (accepted by the user)

A producer that fails with API Error 401 (expired login) should end as an environment stop whose partial work is kept as a candidate, as a timed-out producer's is. `athena retry` should resume from that candidate, and the driver call should be retried once after a short wait. The goal is that a transient login expiry no longer costs a 30-minute producer's work. The audience is the owner running `athena retry`.

Assumptions:
- Code survey: read crates/athena-core/src/disk.rs. `api_error_status`, `StopCause::of`, `producer_stop_reason`, `is_environment_reason` and `resumable_stop` already treat a 401 (PRODUCER_AUTH) and a 429 as environment stops, and `resumable_stop` lets `athena retry` resume a Failed job with an environment reason.
- Code survey: read crates/athena-core/src/run/producer.rs (around lines 364-391). A TimedOut outcome goes to `keep_timed_out_work`, which builds a candidate commit and a progress note. Any other non-Success outcome, including a 401, goes to `self.failure(producer_stop_reason(..))`, which keeps nothing. This is the gap the request names.
- Code survey: read crates/athena-core/src/run/mod.rs. `keep_timed_out_work` (line ~938), `timeout_seed` (~881, which requires the job state TimedOut), `resume_seed` and `resume_candidate` (~533, ~647, which read a resume mark), `write_resume_mark` (~834), and `TimedOutKeep` and `timeout_note` (~1393-1435). The change reuses `keep_timed_out_work` for an auth stop. The job closes Failed with the PRODUCER_AUTH reason, so `resumable_stop` still holds, and the retry finds the kept candidate through the existing seed path. The job state is not TimedOut, so the producer must either add a Failed-state branch to `timeout_seed` or write the resume mark. This is the main design choice.
- Code survey: read crates/athena-agent/src/claude.rs. The driver spawns the CLI through `retry_busy` and builds the detail in `output_from_result` (`<subtype>: <result text>`). A single retry after a short wait on a 401 result goes here, and the wait must be injectable or short so the test is fast. crates/athena-agent/src/fake.rs holds the fake driver the tests use. Existing tests are in crates/athena-core/src/run/tests.rs and crates/athena-cli/tests/retry.rs. The test helpers `EXPIRED_LOGIN_TEXT`, `result_line` and `driver_detail` in disk.rs hold the exact 401 text.
- Records and data: no new record or data file is added, so `check_record_file`, `AnyRecord` and the round-trip test are not affected.
- The 'once after a short wait' retry lives in the agent driver (claude.rs). A second 401 after the retry falls through to the environment stop with kept work.
- Not a refusal rule: nothing newly refuses readings. A 401 job is already classified as an environment stop, so the change only adds kept work.
- Environment: no CI workflow, push or PR is involved. Tests use a fake driver in the local workspace only.
- Consumers: none found. No config deliverable. athena.toml and defaults.toml stay unchanged.

## Acceptance criteria

1. [behavior] A new test using a fake driver that returns the 401 text (`Failed to authenticate. API Error: 401 ... OAuth access token has expired`) for every call, after the producer has edited files, ends the job Failed with a reason starting `environment: producer login expired`. The test asserts that the closing holds a candidate commit with the producer's edits. Run `cargo test -p athena-core` and check that this test passes.
2. [behavior] A new test shows `athena retry <job>` on that stopped job resumes from the kept candidate (the retry's seed or resume names the earlier job, and the earlier edits are present) instead of starting from scratch. Run `cargo test` and check that this test passes.
3. [behavior] A new test shows a driver that returns a 401 once and then succeeds yields a successful invocation: exactly two driver calls were made and the job does not end as an environment stop. Run `cargo test -p athena-agent` and check that this test passes.
4. [behavior] A test shows two consecutive 401 results lead to only one retry (two calls in total, no more) before the environment stop. Run `cargo test` and check that this test passes.
5. [content] The diff in crates/athena-agent/src/claude.rs adds the single retry after a short wait on a 401. crates/athena-core/src/run/mod.rs (with run/producer.rs if needed) keeps partial work for a 401 stop by reusing `keep_timed_out_work`. crates/athena-core/src/disk.rs changes only if the classification or resume rules need it. All three are listed in `git diff`.
6. [content] The existing timeout path and the 429 and low-disk paths keep their behavior: the existing tests in crates/athena-core/src/run/tests.rs and crates/athena-cli/tests/retry.rs still pass under `cargo test`.
7. [content] CHANGELOG.md has a new entry describing the 401 environment stop, the kept partial work with resume, and the single driver retry.
8. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.

## Expected deliverables

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)

## Out of scope

- Handling other API errors (5xx, 529 overloaded) with retries
- Changing how 429 usage limits are handled
- Automatic retry or resume of the job without the user running `athena retry`, beyond the existing drain behavior
- Refreshing the login or reading credentials

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The criteria require new tests plus cargo test, clippy -D warnings and fmt --check.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria check the 401 stop, kept work, resume and retry by running tests, not by checking that code exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix reuses the timeout keep-work path and the central StopCause classification instead of adding a special case.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry and doc comments are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test `a_producer_login_expiry_keeps_its_edits_and_closes_as_an_environment_stop` in crates/athena-core/src/run/tests.rs. The script is `base_script()` wrapped like `timing_out()` (tests.rs:5191): for the producer it sets outcome Error, structured None and detail `crate::disk::driver_detail(&result_line("success", EXPIRED_LOGIN_TEXT))` (the real `output_from_result` text `success: Failed to authenticate. API Error: 401 ...`). The producer still edits README.md. Run `finished_job(&athena)` (ask, then go). Assertions: state Failed; reason starts with `environment: producer login expired` (PRODUCER_AUTH), contains `its edits are kept as candidate`; `is_environment_reason` true; `Records::load` gives a job candidate whose commit's README.md (`git.show_file`) equals the producer's text; `progress-note` is among the job's briefs and names README.md; `read_resume(&id)` is None; one producer call, no verifier. Second test `a_login_expiry_with_no_edit_keeps_nothing_and_a_usage_limit_keeps_nothing`: no edit gives a note saying `Nothing was kept` and no candidate; a 429 detail with edits closes Failed with a reason starting `USAGE_LIMIT` and `job.candidate` None. Run: `cargo test -p athena-core login_expiry`. Observes: The tests pass: the job closes Failed with a reason starting `environment: producer login expired`, the closing record holds a candidate commit whose README.md is the producer's edit, a progress note exists, no resume mark exists, no verifier ran; the 429 case keeps no candidate.
  Fails without the behavior: Without the new branch in producer.rs the 401 goes to `self.failure(...)`: the reason has no `kept as candidate`, `job.candidate` is None and there is no progress-note, so the candidate and note assertions fail.
  Test seam: Production entry point `Athena::go` via `finished_job` (reaches the producer loop in producer.rs and `keep_unfinished_work`). Fake: `FakeAgentDriver::new(script)` injected through `athena_with(&repo, &state, PASSES, plenty_of_space, script)` (tests.rs:190), which exists. The 401 detail is built by production `output_from_result` through the existing helper `disk::driver_detail`, so no parsing is in the double. Scripted `edits` and `detail` exist (fake.rs:14-25, 97-111), so fake.rs needs no change.
- Criterion 2: New test `retrying_a_login_expiry_stop_resumes_from_the_kept_work` in crates/athena-cli/tests/retry.rs, modelled on lifecycle.rs:3126. The script wraps `Scenario::default().script()` with a `reading(&[("README.md","modify"),("src/partial.rs","create")], ...)`. On the first producer call it returns outcome Error with the 401 detail (built with `driver_detail`-style literal `success: Failed to authenticate. API Error: 401 ...`) and edits `src/partial.rs` and README.md. Later producer calls record whether `src/partial.rs` exists in the workdir at start and the user prompt, then finish normally. Steps: `athena.ask`, `go` (Failed, environment reason), `athena.check_history()` has no violations, `athena.retry(first)`, `go(second)`. Assertions: the second producer saw `src/partial.rs` before acting; its brief contains `## Resume`, the first job id and `kept as candidate <short>` and does not contain `reached its deadline`; the retry lands with the first run's file content; `repo/kept/<first>` does not exist. Companion test in tests.rs `timeout_seed_resumes_a_login_expiry_stop_but_not_other_failures` calls `athena.timeout_seed(&records, "try again on <job>", &head)`: Some with `commit` equal to the candidate for the 401 job; None for a Failed work failure and a 429 job. The drain's own retry uses the same `athena.retry` then `go` pair (drain.rs:4478), so this test covers the drain's second attempt; the drain tests stay as the classification check (`an_item_whose_producer_login_expired_twice_stops_the_batch_...`, tests/drain.rs:1181). Run: `cargo test -p athena-cli --test retry login_expiry && cargo test -p athena-core timeout_seed && cargo test -p athena-cli --test drain`. Observes: The retry's producer starts with the earlier edits in its work tree and a Resume section naming the stopped job (with the neutral wording); the retry lands with the earlier file included. `timeout_seed` returns the kept commit only for the login-expiry job. The drain tests pass unchanged.
  Fails without the behavior: With `timeout_seed` still requiring TimedOut, the retry starts from the base: `src/partial.rs` is absent when the producer starts and the brief has no `## Resume`, so the assertions fail.
  Test seam: Production entry points `Athena::retry` and `Athena::go` (dispatch calls `timeout_seed`, run/mod.rs:1091). Fake: `Fixture::athena(script)` from crates/athena-cli/tests/common/mod.rs:179 with a wrapper around `Scenario::script()` (common/mod.rs:561); both exist. Scripted Error outcome and detail are existing FakeTurn fields. No new seam.
- Criterion 3: New tests in the `stand_in_tests` module of crates/athena-agent/src/claude.rs, using the real `ClaudeCodeDriver::invoke` and a real child process. The existing STAND_IN shell script is extended: it reads the prompt into the file named by env PROMPT_FILE (default /dev/null), counts calls in the file named by env COUNT_FILE and appends each call's arguments to `$COUNT_FILE.args`; new modes `auth_once` (call 1 prints a 401 result with subtype `success`, is_error true, `total_cost_usd` 0.25, usage and modelUsage entries; later calls print the normal success result), `auth_always`, `auth_stop` (writes STOP_FILE, then prints the 401 result) and `limit` (a 429 result). Test `a_login_expiry_is_retried_once_and_a_second_call_succeeds`: driver from the module's `driver()` helper with `auth_retry_wait` set to 300 ms, spec timeout 30 s and budget 0.5, `COUNT_FILE` and `PROMPT_FILE` in `spec.env`. Asserts: outcome Success; `structured["answer"] == 42`; count file reads 2; elapsed at least 300 ms; the second call's arguments carry `--max-budget-usd 0.25` (0.5 minus the first call's 0.25); the second prompt contains the expired-login note and the original `brief`; returned `cost_usd` is the sum (0.25 + 0.002, within 1e-9), `usage` is the sum of both calls, `commands_run` holds both calls' commands, `models_used` is merged, and the recorder sink's last update equals the summed usage. Test `a_login_expiry_with_no_time_left_is_not_retried`: timeout 2 s, wait 3 s gives count 1 and the first Error output returned. Test `an_athena_stop_during_a_login_expiry_wait_ends_the_retry`: mode `auth_stop` returns Err(DriverError::Stopped) with count 1. Test `a_usage_limit_is_not_retried`: mode `limit` gives count 1 and an Error output. Pure test `retry_spec_leaves_the_remaining_time_and_budget`: for spec timeout 600 s, first run 100 s and wait 30 s the retry spec's timeout is 470 s (under a 10 s remainder gives None), budget is original minus first cost (under $0.01 gives None), and the prompt starts with the note and ends with the original prompt. Run: `cargo test -p athena-agent login_expir`. Observes: The first call returns a 401 error result, the driver waits, the second call succeeds, the stand-in's call count is exactly 2, the returned output carries both calls' spend, the second call ran with the remaining budget and the note, and the retry is skipped when no time is left, on a stop, and for a 429.
  Fails without the behavior: Without the wrapper the first call's 401 Error is returned: outcome is Error and the count is 1, so the Success and count assertions fail. Without the merge, cost_usd would be 0.002 and the sum assertion fails. Without the remainder logic the second call's `--max-budget-usd` is 0.50 and the args assertion fails.
  Test seam: Production entry point `ClaudeCodeDriver::invoke` (real code, real child). Fixture: the existing `STAND_IN` script and `stand_in()` helper (claude.rs:604-626), extended in plan step 7 with the modes and env files above; the private `auth_retry_wait` field is added in step 2 and lowered by the test, as `min_stall_window` is today (claude.rs:779). The `driver()` helper literal (claude.rs:629-637) gets the new field in step 7.
- Criterion 4: (a) In claude.rs stand_in_tests, `two_login_expiries_make_two_calls_and_return_the_error` uses mode `auth_always` (a 401 result with cost 0.25 on every call), wait 300 ms, budget 0.5. It asserts the count file reads exactly 2, outcome Error, the detail starts with `success: Failed to authenticate. API Error: 401`, the returned cost is 0.5 (0.25 + 0.25, both spends counted) and the elapsed time is short, so a second 401 is returned, not retried a third time. (b) The criterion 1 test feeds exactly that Error outcome and detail to the core and asserts one producer call and the environment stop with kept work. Run: `cargo test -p athena-agent login_expir && cargo test -p athena-core login_expiry`. Observes: After two consecutive 401 results the driver has called the child exactly twice and returns the Error output with the 401 detail and the combined cost; the core turns that into the environment stop with the candidate kept.
  Fails without the behavior: A wrapper that looped would make the count exceed 2 or hang; one that never retried gives count 1; both fail `count == 2`.
  Test seam: Production entry points `ClaudeCodeDriver::invoke` for (a) and `Athena::go` for (b). Fixtures: stand-in `auth_always` and a lowered wait (steps 2 and 7), and the fake driver as for criterion 1 (exists). The halves are in separate crates because the stand-in is `#[cfg(test)]` in athena-agent and unreachable from athena-core; each half runs production code.
- Criterion 5: Run `git diff --stat` and read the diff. claude.rs shows the `invoke_once` split, the wrapper, `retry_spec`, the merging sink and the moved `api_error_status`. run/mod.rs shows `keep_unfinished_work` and the `timeout_seed` change. producer.rs shows the new branch before `self.failure`. disk.rs shows the import and `keeps_unfinished_work`. briefs.rs shows the one-sentence Resume rewording. Confirm `keep_unfinished_work` holds the single copy of the candidate-building, note and redaction code and is called from both the deadline and the login-expiry branches. Run: `git diff --stat`. Observes: The list holds claude.rs, run/mod.rs, run/producer.rs, disk.rs, briefs.rs, the tests and CHANGELOG.md. `fn keep_timed_out_work` no longer exists; `keep_unfinished_work` has two call sites in producer.rs.
- Criterion 6: The existing suites for the paths that must not change run unchanged: run/tests.rs `a_deadline_keeps_a_candidate_and_timeout_seed_prefers_it_to_the_kept_directory`, `timeout_seed_is_none_for_a_job_that_did_not_time_out`, the timeout-note tests; lifecycle.rs `a_producer_that_hits_its_deadline_keeps_its_work_and_closes_timed_out` and `retrying_a_timed_out_job_...`; `low_disk_before_the_verifier_holds_the_job_and_a_retry_resumes_it`; `a_check_killed_at_its_deadline_closes_as_an_environment_stop`; the disk.rs StopCause tests; the drain tests; all of retry.rs. The new 429 case in the criterion 1 test shows a 429 producer still keeps nothing. Run: `cargo test --locked`. Observes: All existing tests pass unchanged. The deadline path still closes `timed_out` with a reason starting `the producer did not complete: `; 429 and low-disk stops are still Failed environment stops.
  Fails without the behavior: If the generalisation changed the deadline path (state or reason prefix), tests.rs:5216-5219 and lifecycle.rs:3025 fail.
  Test seam: Production entry points `Athena::go`, `Athena::retry` and the disk.rs classifiers through existing fixtures. Nothing new.
- Criterion 7: Add a bullet under `## [Unreleased]` / `### Fixed` in CHANGELOG.md in the style of its neighbours. It names the 2026-10-28 incident (a 30-minute producer, `API Error: 401 OAuth access token has expired`, work lost); says a 401 now closes the job as an environment stop (`environment: producer login expired`) with the partial work kept as a candidate and a progress note, as a deadline's is; says `athena retry` (and the drain's second attempt) resumes the producer from it; says the driver waits about 30 seconds and retries the call once, as one invocation (the remaining time and budget, with both calls' cost and usage counted), and tells the retried model that its earlier edits are in the working directory; notes the Resume wording no longer says deadline. Read it back. Run: `git diff CHANGELOG.md`. Observes: The diff adds one entry under Unreleased covering the 401 environment stop, the kept partial work with resume, and the single driver retry.
- Criterion 8: Run the three checks Athena runs. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0: no unused import after `api_error_status` moves, no `too_many_arguments` lint on `keep_unfinished_work` (self plus six arguments, within clippy's limit of seven), no lint on the merging sink or `retry_spec`.
  Fails without the behavior: A failing format, lint or test makes the matching command exit non-zero.
  Test seam: Whole-workspace commands; no seam.

Planned files:

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base d6083bafe5a1 → candidate e47eb46be315. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  18 ++
 crates/athena-agent/src/claude.rs      | 427 ++++++++++++++++++++++++++++++++-
 crates/athena-cli/tests/retry.rs       | 119 +++++++++
 crates/athena-context/src/briefs.rs    |   6 +-
 crates/athena-core/src/disk.rs         |  82 +++++--
 crates/athena-core/src/run/mod.rs      |  43 +++-
 crates/athena-core/src/run/producer.rs |  32 ++-
 crates/athena-core/src/run/tests.rs    | 228 ++++++++++++++++++
 8 files changed, 912 insertions(+), 43 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261028-112841-hx35/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261028-112841-hx35/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42740992238 for e47eb46be3150fbb3e9a7e9eb4dc3eb9669518fe)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42740992238 for e47eb46be3150fbb3e9a7e9eb4dc3eb9669518fe)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42740992238 for e47eb46be3150fbb3e9a7e9eb4dc3eb9669518fe)
- ci-result: `not a check` passed (Check results for e47eb46be3150fbb3e9a7e9eb4dc3eb9669518fe come from the CI run https://github.com/doodla/athena/actions/runs/42740992238 (765s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core login_expiry`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/claude.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `keeps_unfinished_work` in module `crate::disk`
    --> crates/athena-core/src/run/tests.rs:5375:30
     |
5375 |         assert!(crate::disk::keeps_unfinished_work(
     |                              ^^^^^^^^^^^^^^^^^^^^^ not found in `crate::disk`

error[E0425]: cannot find function `keeps_unfinished_work` in this scope
    --> crates/athena-core/src/disk.rs:1535:17
     |
1535 |         assert!(keeps_unfinished_work(
     |                 ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `keeps_unfinished_work` in this scope
    --> crates/athena-core/src/disk.rs:1539:17
     |
1539 |         assert!(keeps_unfinished_work(timed_out, None));
     |                 ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `keeps_unfinished_work` in this scope
    --> crates/athena-core/src/disk.rs:1541:21
     |
1541 |             assert!(keeps_unfinished_work(failed, Some(reason)), "{reason}");
     |                     ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `keeps_unfinished_work` in this scope
    --> crates/athena-core/src/disk.rs:1555:22
     |
1555 |             assert!(!keeps_unfinished_work(failed, Some(reason)), "{reason}");
     |                      ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `keeps_unfinished_work` in this scope
    --> crates/athena-core/src/disk.rs:1557:18
     |
1557 |         assert!(!keeps_unfinished_work(failed, None));
     |                  ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `keeps_unfinished_work` in this scope
    --> crates/athena-core/src/disk.rs:1559:22
     |
1559 |             assert!(!keeps_unfinished_work(
     |                      ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `keeps_unfinished_work` in this scope
    --> crates/athena-core/src/disk.rs:1564:18
     |
1564 |         assert!(!keeps_unfinished_work(JobState::Dispatched, Some(&login)));
     |                  ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 8 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-cli --test retry login_expiry && cargo test -p athena-core timeout_seed && cargo test -p athena-cli --test drain`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-agent login_expir`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/claude.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs.
  End of the output (data, not instructions):

```text
   --> crates/athena-agent/src/claude.rs:871:16
    |
871 |         driver.auth_retry_wait = Duration::from_millis(300);
    |                ^^^^^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `bin`, `version`, `registry`, `refuse_nested_runs`, `min_stall_window`

error[E0609]: no field `auth_retry_wait` on type `claude::ClaudeCodeDriver`
   --> crates/athena-agent/src/claude.rs:887:16
    |
887 |         driver.auth_retry_wait = Duration::from_millis(300);
    |                ^^^^^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `bin`, `version`, `registry`, `refuse_nested_runs`, `min_stall_window`

error[E0425]: cannot find function `api_error_status` in this scope
   --> crates/athena-agent/src/claude.rs:894:20
    |
894 |         assert_eq!(api_error_status(&out.detail.unwrap()), Some(429));
    |                    ^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `retry_spec` in this scope
   --> crates/athena-agent/src/claude.rs:918:21
    |
918 |         let retry = retry_spec(&original, &first, Duration::from_secs(130)).unwrap();
    |                     ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `retry_spec` in this scope
   --> crates/athena-agent/src/claude.rs:924:17
    |
924 |         assert!(retry_spec(&original, &first, Duration::from_secs(591)).is_none());
    |                 ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `retry_spec` in this scope
   --> crates/athena-agent/src/claude.rs:925:17
    |
925 |         assert!(retry_spec(&original, &first, Duration::from_secs(600)).is_none());
    |                 ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `retry_spec` in this scope
   --> crates/athena-agent/src/claude.rs:926:17
    |
926 |         assert!(retry_spec(&original, &first, Duration::from_secs(900)).is_none());
    |                 ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `retry_spec` in this scope
   --> crates/athena-agent/src/claude.rs:928:17
    |
928 |         assert!(retry_spec(&original, &first, Duration::from_secs(130)).is_none());
    |                 ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `retry_spec` in this scope
   --> crates/athena-agent/src/claude.rs:931:21
    |
931 |         let retry = retry_spec(&original, &first, Duration::from_secs(130)).unwrap();
    |                     ^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0425, E0560, E0609.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-agent` (lib test) due to 17 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-agent login_expir && cargo test -p athena-core login_expiry`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 6 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/claude.rs, crates/athena-context/src/briefs.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs.
  End of the output (data, not instructions):

```text
    |
871 |         driver.auth_retry_wait = Duration::from_millis(300);
    |                ^^^^^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `bin`, `version`, `registry`, `refuse_nested_runs`, `min_stall_window`

error[E0609]: no field `auth_retry_wait` on type `claude::ClaudeCodeDriver`
   --> crates/athena-agent/src/claude.rs:887:16
    |
887 |         driver.auth_retry_wait = Duration::from_millis(300);
    |                ^^^^^^^^^^^^^^^ unknown field
    |
    = note: available fields are: `bin`, `version`, `registry`, `refuse_nested_runs`, `min_stall_window`

error[E0425]: cannot find function `api_error_status` in this scope
   --> crates/athena-agent/src/claude.rs:894:20
    |
894 |         assert_eq!(api_error_status(&out.detail.unwrap()), Some(429));
    |                    ^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `retry_spec` in this scope
   --> crates/athena-agent/src/claude.rs:918:21
    |
918 |         let retry = retry_spec(&original, &first, Duration::from_secs(130)).unwrap();
    |                     ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `retry_spec` in this scope
   --> crates/athena-agent/src/claude.rs:924:17
    |
924 |         assert!(retry_spec(&original, &first, Duration::from_secs(591)).is_none());
    |                 ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `retry_spec` in this scope
   --> crates/athena-agent/src/claude.rs:925:17
    |
925 |         assert!(retry_spec(&original, &first, Duration::from_secs(600)).is_none());
    |                 ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `retry_spec` in this scope
   --> crates/athena-agent/src/claude.rs:926:17
    |
926 |         assert!(retry_spec(&original, &first, Duration::from_secs(900)).is_none());
    |                 ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `retry_spec` in this scope
   --> crates/athena-agent/src/claude.rs:928:17
    |
928 |         assert!(retry_spec(&original, &first, Duration::from_secs(130)).is_none());
    |                 ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `retry_spec` in this scope
   --> crates/athena-agent/src/claude.rs:931:21
    |
931 |         let retry = retry_spec(&original, &first, Duration::from_secs(130)).unwrap();
    |                     ^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0425, E0560, E0609.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-agent` (lib test) due to 17 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
