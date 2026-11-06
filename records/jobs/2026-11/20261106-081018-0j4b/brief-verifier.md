# Job 20261106-081018-0j4b: verify "Keep the failing section of a CI log, not its cleanup tail"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The check result stores the tail of the CI log, and the tail is post-job cleanup text. The failing t
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: failed-check-output-is-ci-noise-cause-of-test-failure-invisible (20261106-042835-rxkn)
> Cause: The check result stores the tail of the CI log, and the tail is post-job cleanup text. The failing test's output is cut off, so the record cannot be used to diagnose anything. Nothing extracts failing test names from the test runner, and th...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261106-042824-j07h (job 20261106-022423-vdtn); prevention it proposed: When the test check fails, Athena should capture and store the failing test names and assertion output from the test runner itself (for example the `failures:` section of cargo test), not the tail of the CI log. If no test name can be extra...
> Introspection 20261106-081000-96mr (job 20261106-045349-f6x6); prevention it proposed: When a test check fails, Athena should save the failing section of the log rather than its tail. That means the lines from "test result", "FAILED", "failures:" and "panicked at", with the test names and assertion messages. The save should a...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: `detail_from_job_logs` (crates/athena-core/src/gh/client.rs) tails the whole CI job log, which ends in runner post-job cleanup, and when no test name is parsed (e.g. a shard that did not finish, or an xtask test) the stored `output_tail` is only that noise. Fix it there: cut the post-job cleanup block before tailing and, when the tail has no failure evidence, keep the failure lines (`test result`, `FAILED`, `failures:`, `panicked at`) instead. This addresses jobs 20261106-022423-vdtn and 20261106-045349-f6x6 only as the evidence of the class; their candidate changes (config ceiling, mirrored_sources) are dropped.

Assumptions:
- Goal and audience: the stored check result is read by the verifier and by introspection, who must be able to see why a test check failed.
- Code survey: read crates/athena-core/src/gh/client.rs (detail_from_job_logs, failing_units, plain_log_line) and crates/athena-core/src/checks.rs (tail_lines, tail, failed_test_names, with_failure_detail, FAILED_TAIL_LINES). The entry point is CiSource::failure_detail, which fetches job logs and calls detail_from_job_logs; names are parsed first, then the whole log is tailed, then the FAILING_UNITS_PREFIX header is added only when names exist. The bug is that the tail is taken of the full log including post-job steps ('Post job cleanup', 'Cleaning up orphan processes', git config lines).
- No new record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected; CheckRun/FailureDetail fields stay unchanged.
- Job 20261106-022423-vdtn: its candidate (max_concurrency ceiling 8 to 16 in config.rs, docs, athena.toml, CHANGELOG) is unrelated to the cause and is dropped; its CI findings were the noise tail itself plus an xtask test flake, which this change does not address except for the output they showed. Its CHANGELOG nit is moot.
- Job 20261106-045349-f6x6: its candidate (mirrored_sources plan field, admission rule, prompts, docs) is unrelated and dropped; its findings concerned that work, and its drain-test CI failure showed the same cleanup tail, which this change fixes for later jobs.
- Not a new refusal rule: the change does not refuse readings, so no replay-refusal percentage applies.
- Tail is applied before redaction in with_failure_detail; the new trimming happens earlier, so redaction behavior is unchanged.

## Acceptance criteria

1. [content] The CHANGELOG entry (CHANGELOG.md, under Fixed) and code comment name the root cause: the stored tail was taken of the whole CI log, which ends with post-job cleanup, and failure lines were not selected.
2. [behavior] Run `cargo test -p athena-core detail_from_job_logs`: a new test feeds a job log with a failing test and a trailing post-job cleanup block (Post job cleanup, git config --unset-all, Cleaning up orphan processes, a Node.js deprecation warning) and asserts output_tail contains the `FAILED`/`panicked at` lines and no cleanup text. It fails on the base commit and passes with the change.
3. [behavior] Run `cargo test -p athena-core detail_from_job_logs`: a new test with a log that yields no parsed test names (a shard that did not finish, or non-cargo test output) followed by cleanup asserts output_tail keeps the lines containing `test result`, `FAILED`, `failures:` or `panicked at` rather than only cleanup text. It fails before the change and passes after, so it covers the class and not one job.
4. [behavior] Run `cargo test -p athena-core checks`: existing tests for tail_lines, 200-line and byte caps, FAILING_UNITS_PREFIX and redaction still pass, and a log with no failure lines and no cleanup keeps its plain tail as before.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`: all pass.
6. [content] No files outside crates/athena-core (and CHANGELOG.md) change; the 20261106-022423-vdtn and 20261106-045349-f6x6 candidate files are not reintroduced.

## Expected deliverables

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- The max_concurrency ceiling change from job 20261106-022423-vdtn (config.rs, docs, athena.toml): dropped
- The mirrored_sources plan field, admission rule and prompt changes from job 20261106-045349-f6x6 (records.rs, briefs.rs, admission.rs, planner/critic prompts, etc.): dropped
- Fixing the flaky xtask repin test or drain tests
- Editing records/, athena.toml, defaults.toml or CI workflows
- Changing the local (non-CI) failed-check tail behavior beyond what the shared helper needs

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: fix in detail_from_job_logs where the stored tail is built, with a test on the class
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: new tests plus fmt, clippy, cargo test criterion
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: tests assert the stored output content, not wiring
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: change is Rust only
- small: Prefer one focused change; size is a design choice, not a wish list. Here: one focused change in the log-to-detail path
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: verifier runs the named cargo tests
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: criteria check that the failure text is kept and cleanup text dropped
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check of the diff: the CHANGELOG Fixed bullet and the doc comments on without_post_job_cleanup, failure_tail and detail_from_job_logs name the cause (the stored tail was taken of the whole CI log, which ends with post-job cleanup, and failure lines were not selected). Run: `git diff -- CHANGELOG.md crates/athena-core/src/gh/client.rs crates/athena-core/src/checks.rs`. Observes: A bullet under `## [Unreleased]` / `### Fixed` in CHANGELOG.md, and doc comments in client.rs and checks.rs, that state the root cause and that detail_from_job_logs is where it is fixed.
- Criterion 2: New test `detail_from_job_logs_drops_the_post_job_cleanup_tail` in gh/tests.rs. Log built with the existing `stamped` helper (gh/tests.rs:376): `Running unittests ...`, `running 1 test`, `test tests::a ... FAILED`, `failures:`, `---- tests::a stdout ----`, `thread 'tests::a' panicked at src/lib.rs:9:5:`, an assertion line, ` left: 1`, ` right: 2`, a blank, `failures:`, `    tests::a`, a blank, `test result: FAILED. 0 passed; 1 failed`, then `Post job cleanup.`, `[command]/usr/bin/git config --local --unset-all http.https://github.com/.extraheader`, `Cleaning up orphan processes`, `##[warning]Node.js 20 actions are deprecated`. It calls `detail_from_job_logs(&[("test (shard 1)".to_string(), "failure".to_string(), Some(log))])`, the signature at client.rs:248-250, as job_log_parser_strips_stamps_keeps_the_indent_and_names_the_units does (gh/tests.rs:675-680). Asserts on the returned FailureDetail: output_tail contains `panicked at`, `test tests::a ... FAILED`, ` right: 2` and `test result: FAILED`; contains none of `Post job cleanup`, `unset-all`, `Cleaning up orphan processes`, `Node.js`; the last 600 chars and the last 60 lines of output_tail each contain `panicked at`; failed_tests is ["athena_core tests::a"]. Run: `cargo test -p athena-core detail_from_job_logs`. Observes: The test passes: the stored tail ends at the failure and carries the whole panic block, with no cleanup text, and the 600-char and 60-line ends hold the panic.
  Fails without the behavior: On the base commit the tail is the last 200 lines of the whole log, so the cleanup text is present and the not-contains assertions fail.
  Test seam: Production entry point detail_from_job_logs (pub, gh/client.rs:248), called with a vec of (name, conclusion, Some(log)). The log is real input built by the existing `stamped` helper; no double. Guards: it returns None only when no log is readable, and the log is readable. The test is added in plan step 5.
- Criterion 3: Two new tests in gh/tests.rs, both through detail_from_job_logs. (a) `detail_from_job_logs_keeps_failure_lines_when_no_test_is_named`: the log has no `Running` or `Doc-tests` line, so failed_test_names yields no names. It holds `thread 'x' panicked at src/main.rs:3:5:`, `assertion failed: ok`, `test result: FAILED. 0 passed; 1 failed`, `failures:`, then 250 progress lines `compiling crate_N` and the cleanup block; conclusion `cancelled`. Asserts: failed_tests is empty; output_tail contains `panicked at`, `assertion failed: ok`, `test result: FAILED` and `failures:`; no cleanup text; the last 600 chars and last 60 lines contain `panicked at`. (b) `detail_from_job_logs_keeps_a_failing_unit_behind_passing_units` (finding 1): unit 1 `Running unittests src/lib.rs (target/debug/deps/athena_core-0f3a9b)` fails `tests::a` with a full assert_eq panic block (message, ` left:`, ` right:`), then unit 2 `Running` prints 250 lines `test t_N ... ok` and `test result: ok. 250 passed; 0 failed`, then cleanup. Asserts: failed_tests names `athena_core tests::a`; output_tail contains `panicked at`, ` right:` and `test result: FAILED`; the last 600 chars and the last 60 lines contain `panicked at`; no cleanup text; output_tail does not contain `test result: ok` as its last line's only evidence (the passing summary is not selected as a marker). Run: `cargo test -p athena-core detail_from_job_logs`. Observes: Both tests pass: the failure lines are kept in output_tail, at its end, when the failing output was pushed out of the last 200 lines, both when no name is parsed and when a passing unit follows the failing one.
  Fails without the behavior: On the base commit the last 200 lines are progress or passing-unit lines plus cleanup, so the `panicked at` / `test result: FAILED` assertions fail. With the broad `test result` marker (the rejected design), test (b) would fail, because the passing summary in the tail would count as evidence.
  Test seam: Production entry point detail_from_job_logs (gh/client.rs:248) with logs from the existing `stamped` helper (gh/tests.rs:376); no double. Guard: failed_test_names reads names from `Running`/`Doc-tests` sections (checks.rs:119-163), so (a) deliberately has none and (b) has one. Both tests are added in plan step 5.
- Criterion 4: Existing tests run unchanged: tail_lines (checks.rs ~754), the 200-line cap (~639), the byte cap (~783), red_reused_run_records_failing_tests_and_log_tail (gh/tests.rs:493, starts with the units header, ends with the last log line), the redaction test (:536), reused_red_degrades_when_the_log_lists_no_failures (:577, no-marker no-cleanup log keeps its plain tail and ends with `build broke`), and job_log_parser_strips_stamps_keeps_the_indent_and_names_the_units (:673). New unit tests in checks.rs call the pure helpers: `without_post_job_cleanup` (cut at each marker; no marker returns the text unchanged; marker on the first line keeps the log; a test line merely containing the marker text mid-line is not a cut) and `failure_tail` (no marker returns exactly tail_lines; all markers in the end window returns exactly tail_lines; a `test result: ok` only log returns the plain tail; a marker outside the window returns the section last and respects the n cap and n/2 section cap; ` right:` of an assert_eq block is kept). Run: `cargo test -p athena-core checks`. Observes: All checks tests pass, including the old tail and cap tests and the new equality tests, and the gh tests listed above still pass under `cargo test -p athena-core gh`.
  Fails without the behavior: If failure_tail or the cut altered ordinary logs (no markers, or markers already at the end), the plain-tail equality tests and the existing cap tests would fail; if a passing `test result: ok` counted as a marker, the passing-only test would fail.
  Test seam: Pure functions `without_post_job_cleanup` and `failure_tail` in checks.rs, called directly with string inputs; no double. The helpers are added in plan steps 1-3 and their unit tests in step 6; tail_lines, tail and the existing tests exist today.
- Criterion 5: Run the three CI checks on the result. `cargo test --locked` runs every test binary, including the three new detail_from_job_logs tests and the new checks.rs unit tests, so it is the end-to-end proof that the behavior holds and nothing else broke. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0; the test summary lists the three new detail_from_job_logs tests as passed.
  Fails without the behavior: On the base commit with only the new tests added, `cargo test --locked` exits non-zero because the detail_from_job_logs tests fail on the cleanup-tail and missing-failure assertions; with the helpers present but not wired into detail_from_job_logs the same tests still fail. Unformatted code or a clippy warning in the helpers fails the first two commands.
  Test seam: Entry points are the cargo toolchain over the workspace; the production code under test is detail_from_job_logs called by the new tests in gh/tests.rs. Fixture: the `stamped` log builder (exists, gh/tests.rs:376) passed as the `Some(log)` element; no fakes. The tests are added in steps 5-6 and step 8 runs the commands.
- Criterion 6: Inspect the changed paths. Run: `git status`. Observes: Only crates/athena-core/src/gh/client.rs, crates/athena-core/src/gh/tests.rs, crates/athena-core/src/checks.rs and CHANGELOG.md changed; no config.rs, records.rs, briefs.rs, admission.rs, docs or athena.toml.

Planned files:

- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base cb3ba2ba255b → candidate 4a2130065f67. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   7 ++
 crates/athena-core/src/checks.rs    | 156 ++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/gh/client.rs |  10 ++-
 crates/athena-core/src/gh/tests.rs  | 129 +++++++++++++++++++++++++++++
 4 files changed, 300 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-081018-0j4b/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-081018-0j4b/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42214357668 for 4a2130065f67f1de07bddde2fff324fd05aa4a9d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42214357668 for 4a2130065f67f1de07bddde2fff324fd05aa4a9d)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42214357668 for 4a2130065f67f1de07bddde2fff324fd05aa4a9d)
- ci-result: `not a check` passed (Check results for 4a2130065f67f1de07bddde2fff324fd05aa4a9d come from the CI run https://github.com/doodla/athena/actions/runs/42214357668 (608s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --locked`: all pass.)
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2, 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core detail_from_job_logs`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/checks.rs, crates/athena-core/src/gh/client.rs.
  End of the output (data, not instructions):

```text
626 |         assert!(kept.contains(FAILURE_SECTION_SEPARATOR));
    |                               ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `without_post_job_cleanup` in this scope
   --> crates/athena-core/src/checks.rs:596:24
    |
596 |             assert_eq!(without_post_job_cleanup(&log), "a\nb\n", "{marker}");
    |                        ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `without_post_job_cleanup` in this scope
   --> crates/athena-core/src/checks.rs:598:20
    |
598 |         assert_eq!(without_post_job_cleanup("a\nb\n"), "a\nb\n");
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `without_post_job_cleanup` in this scope
   --> crates/athena-core/src/checks.rs:601:20
    |
601 |         assert_eq!(without_post_job_cleanup(first), first);
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `without_post_job_cleanup` in this scope
   --> crates/athena-core/src/checks.rs:604:20
    |
604 |         assert_eq!(without_post_job_cleanup(mid), mid);
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_tail` in this scope
   --> crates/athena-core/src/checks.rs:610:20
    |
610 |         assert_eq!(failure_tail(&plain, 200), tail_lines(&plain, 200));
    |                    ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_tail` in this scope
   --> crates/athena-core/src/checks.rs:612:20
    |
612 |         assert_eq!(failure_tail(&passing, 200), tail_lines(&passing, 200));
    |                    ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_tail` in this scope
   --> crates/athena-core/src/checks.rs:614:20
    |
614 |         assert_eq!(failure_tail(&at_end, 200), tail_lines(&at_end, 200));
    |                    ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_tail` in this scope
   --> crates/athena-core/src/checks.rs:624:20
    |
624 |         let kept = failure_tail(&text, 200);
    |                    ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_tail` in this scope
   --> crates/athena-core/src/checks.rs:634:20
    |
634 |         let kept = failure_tail(&many, 40);
    |                    ^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 10 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core checks`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/checks.rs, crates/athena-core/src/gh/client.rs.
  End of the output (data, not instructions):

```text
    |
626 |         assert!(kept.contains(FAILURE_SECTION_SEPARATOR));
    |                               ^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `without_post_job_cleanup` in this scope
   --> crates/athena-core/src/checks.rs:596:24
    |
596 |             assert_eq!(without_post_job_cleanup(&log), "a\nb\n", "{marker}");
    |                        ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `without_post_job_cleanup` in this scope
   --> crates/athena-core/src/checks.rs:598:20
    |
598 |         assert_eq!(without_post_job_cleanup("a\nb\n"), "a\nb\n");
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `without_post_job_cleanup` in this scope
   --> crates/athena-core/src/checks.rs:601:20
    |
601 |         assert_eq!(without_post_job_cleanup(first), first);
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `without_post_job_cleanup` in this scope
   --> crates/athena-core/src/checks.rs:604:20
    |
604 |         assert_eq!(without_post_job_cleanup(mid), mid);
    |                    ^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_tail` in this scope
   --> crates/athena-core/src/checks.rs:610:20
    |
610 |         assert_eq!(failure_tail(&plain, 200), tail_lines(&plain, 200));
    |                    ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_tail` in this scope
   --> crates/athena-core/src/checks.rs:612:20
    |
612 |         assert_eq!(failure_tail(&passing, 200), tail_lines(&passing, 200));
    |                    ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_tail` in this scope
   --> crates/athena-core/src/checks.rs:614:20
    |
614 |         assert_eq!(failure_tail(&at_end, 200), tail_lines(&at_end, 200));
    |                    ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_tail` in this scope
   --> crates/athena-core/src/checks.rs:624:20
    |
624 |         let kept = failure_tail(&text, 200);
    |                    ^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_tail` in this scope
   --> crates/athena-core/src/checks.rs:634:20
    |
634 |         let kept = failure_tail(&many, 40);
    |                    ^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 10 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
