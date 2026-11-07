# Job 20261106-192854-abbg: verify "Keep the failing test section, not the CI cleanup tail, in failed checks"

## Request (verbatim)

> Priority: normal
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

Root cause: a failed check's record keeps the last 200 lines of the log (checks.rs tail_lines in the local run, with_failure_detail, and gh/client.rs detail_from_job_logs). For a CI log those lines are post-job cleanup, which pushes the test runner's failures section out of the record. Fix it once in a shared function in checks.rs that drops post-job cleanup and keeps the failing section, and use it at all three sites. This retries jobs 20261106-022423-vdtn and 20261106-045349-f6x6 for the failure class only. Their own diffs are out of scope.

Assumptions:
- Goal and audience: the verifier, the next producer on a retry, and the person reading a failed check record must be able to see the failing test name and assertion output.
- Code survey: read crates/athena-core/src/checks.rs (tail, tail_lines, FAILED_TAIL_LINES, failed_test_names, with_failure_detail, run path around line 505-545), crates/athena-core/src/gh/client.rs (detail_from_job_logs, plain_log_line, failing_units), crates/athena-core/src/ci_result.rs (FailureDetail), and routing.rs. The entry points are the local check run in checks.rs and detail_from_job_logs, which with_failure_detail applies to a reused red CI run. Both keep a line tail and extract names only from `test ... FAILED` lines. Records validation is unchanged: no record or data file is added, so check_record_file, AnyRecord and the round-trip test are not affected. The property is measured directly on the stored output_tail of the CheckRun, not through a stand-in.
- Job 20261106-022423-vdtn: its failed test check came from CI noise unrelated to its diff (the xtask repin test and the cleanup tail). Its diff (config.rs, docs, CHANGELOG.md, athena.toml) is not redone here. Those files are out of scope and are not kept or changed by this job.
- Job 20261106-045349-f6x6: its mirrored_sources work (records.rs, briefs.rs, planning.rs, admission.rs, prompts and the other 17 changed files) is dropped from this job. Its findings of a stale target dir and an unreadable test log are the failure class this job fixes. Every file it changed is excluded in out_of_scope.
- The cleanup marker is the GitHub Actions line `Post job cleanup.` (also `##[group]Post job cleanup`). The section starts at the earliest runner marker (`failures:`, `panicked at`, `---- ... stdout ----`, `test result: FAILED`) when one exists. It is bounded by the existing FAILED_TAIL_LINES and FAILED_TAIL_BYTES. With no marker it falls back to the current tail of the cleanup-stripped text.
- Redaction still runs on the kept text; the new function only selects lines.

## Acceptance criteria

1. [content] The change description names the root cause: the failed-check record stores the last N lines of the log without regard to what they are, so CI post-job cleanup text displaces the test runner's failing section. It is fixed in one shared function in crates/athena-core/src/checks.rs that both the local run and the CI-log path (gh/client.rs detail_from_job_logs and with_failure_detail) call.
2. [behavior] A new unit test in checks.rs, run with `cargo test -p athena-core checks`, feeds a log with a `failures:` section, a `panicked at` line and a `test result: FAILED` line followed by more than 200 lines of post-job cleanup, as in the CI logs of jobs 20261106-022423-vdtn and 20261106-045349-f6x6. It asserts the kept output contains the failing test name and the assertion text and no `Post job cleanup` lines. The test fails on the base commit (verify with git stash or the fails-without run) and passes with the change.
3. [behavior] A test in gh/tests.rs, run with `cargo test -p athena-core gh`, builds a CI job log whose failing output has no parseable `test ... FAILED` names (for example a panic in an xtask test) followed by cleanup text. It asserts detail_from_job_logs returns an output_tail with the panic message and not the cleanup lines. It fails on the base and passes with the change.
4. [behavior] A log with no runner marker and no cleanup text keeps the same output as today (tail of the last FAILED_TAIL_LINES lines), and a passing check's kept output is unchanged. Existing tests, including red_reused_run_records_failing_tests_and_log_tail and the run/tests.rs reuse test, still pass under `cargo test --locked`.
5. [behavior] `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` pass.
6. [content] CHANGELOG.md has one entry under Fixed or Changed stating that a failed check record now keeps the failing test section instead of the log tail. It states only what the diff does.
7. [content] Not a change to refusal rules: no check is added that refuses or holds work, so no replay refusal-rate criterion applies.

## Expected deliverables

- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)

## Out of scope

- Job 20261106-022423-vdtn's diff: crates/athena-data/src/config.rs, docs/design/agent-and-build-slots.md, docs/design/async-drain.md, docs/design/parallel-jobs.md, athena.toml, and the max_concurrency ceiling change
- Job 20261106-045349-f6x6's diff: mirrored_sources and the prompt, admission, plan and records files it changed (crates/athena-context/prompts/planner.md and critic.md, briefs.rs, outputs.rs, admission.rs, reading.rs, run/mod.rs, run/planning.rs, run_pin_tests.rs, status.rs, store.rs, refusal_corpus.rs, records.rs, job_plan.rs, docs/design/job-planning.md, docs/system-facts.md)
- Fixing the flaky xtask repin test or the stale shared target dir
- Changes to records/, athena.toml, defaults.toml or CI workflows
- Changing the stored record schema (CheckRun fields)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The fix is Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: One shared selector fixes every path that stores a log tail.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The tests assert the stored output content.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the checks module and its CI-log caller.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: An independent verifier runs the tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the failing test text is in the stored record.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff. `failure_section` is defined in checks.rs with a doc comment naming the root cause (the old rule kept the last N lines whatever they were, so cleanup displaced the failing section). The three former `tail(&tail_lines(..))` sites call it. Run: `git diff -- crates/athena-core/src/checks.rs crates/athena-core/src/gh/client.rs CHANGELOG.md`. Observes: One new shared function in checks.rs; run_check, with_failure_detail and detail_from_job_logs call it; the old tail-by-count expressions are gone from those three sites; the CHANGELOG bullet states the change.
- Criterion 2: New tests in checks.rs `mod tests`. (a) `failure_section_drops_trailing_cleanup_and_keeps_the_failing_section`: calls `failure_section(text, FAILED_TAIL_LINES, FAILED_TAIL_BYTES)` on cargo output (`failures:`, `---- t stdout ----`, `thread 't' panicked at … assertion failed: left == right`, `test result: FAILED. 0 passed; 1 failed`, `##[error]Process completed with exit code 101.`) followed by 250 lines of `Post Run …` / `Post job cleanup.` / `Cleaning up orphan processes`. Asserts the test name and assertion text are present and `Post job cleanup` is absent. (b) `a_failed_local_run_keeps_the_failing_section_not_trailing_cleanup`: `run_script(&sh_spec(..))` with a script that echoes the failing section and then 250 `Post job cleanup.` lines and exits 1; asserts `output_tail` holds the test name and panic text and no `Post job cleanup`. (c) `a_cleanup_looking_line_before_the_failing_section_is_kept`: a `Post job cleanup.` line at line 3 and the failing section after it, then trailing cleanup; the early line and the section are both kept. Run: `cargo test -p athena-core checks`. Observes: All pass with the change. On the base commit (a) cannot call the function, and (b) fails: the last 200 lines are all cleanup, so the panic text is missing and `Post job cleanup` is present.
  Fails without the behavior: The kept output holds only cleanup lines: the test-name and assertion-text assertions fail and the no-`Post job cleanup` assertion fails.
  Test seam: Production entry points `checks::failure_section` (added by step 2) and `checks::run_check` through the existing `run_script` and `sh_spec` helpers (exist, checks.rs 736 and its use at 764). The fixture is the shell script text passed to sh_spec and string literals; no test double.
- Criterion 3: New tests in gh/tests.rs using the existing `stamped` helper (gh/tests.rs 376). (a) `cleanup_after_an_xtask_panic_does_not_displace_it`: a stamped CI log with no `Running` line and no `test … FAILED` line, containing `thread 'repin' panicked at xtask/src/…: pin mismatch`, `test result: FAILED. 0 passed; 1 failed`, then 250 stamped `Post job cleanup.` lines. Calls `detail_from_job_logs(&[("test (shard 1)", "failure", Some(log))])`; asserts `output_tail` contains the panic message, has no `Post job cleanup`, and `failed_tests` is empty. (b) `two_shards_each_keep_their_failing_section`: two logs, each with a failing section of about 150 lines (a distinct panic message on each) followed by cleanup; asserts both messages are in output_tail and the total is at most 200 lines. Run: `cargo test -p athena-core gh`. Observes: output_tail holds the panic message(s) and no cleanup lines. On the base the 200-line tail of (a) is all cleanup, and in (b) the joined tail drops the first shard's message.
  Fails without the behavior: The panic message is absent from output_tail and cleanup lines are present in (a); in (b) the first shard's message is absent. The assertions fail.
  Test seam: Production entry point `gh::client::detail_from_job_logs` (re-exported in gh/mod.rs:45), whose input is a plain `(String, String, Option<String>)` slice; existing tests already call it at gh/tests.rs 680 and 709. The `stamped` helper exists. No new seam.
- Criterion 4: Unit tests in checks.rs: `failure_section` on 300 numbered lines with no marker and no cleanup equals `tail(&tail_lines(text, 200), FAILED_TAIL_BYTES)`; a marker inside the last 200 lines and no cleanup gives the same output as the old rule; a passing run still gives the byte tail. With the window marker-free and an earlier marker plus 250 lines of unrecognised trailing text, the output starts at the marker, has a `…` line and ends with the final lines, one of them `error[E0425]` (so demonstration_view's `output_has("error[E")` still holds). With 40 lines of 3000 bytes each after a marker, the output is within max_bytes plus the ellipsis and still starts with the marker line. A `with_failure_detail` test with a `failing units:` first line plus 300 lines keeps that line first. Existing tests also run: `a_failed_run_keeps_its_last_200_lines_and_a_passing_run_its_byte_tail`, `a_failed_runs_kept_lines_stop_at_the_byte_ceiling`, `a_reused_failed_check_takes_the_redacted_names_and_tail_and_logs_the_names`, `red_reused_run_records_failing_tests_and_log_tail`, the run/tests.rs reuse test (about 3481-3499) and the verify.rs and briefs.rs tests. Run: `cargo test --locked`. Observes: All pass. Marker-free, cleanup-free output equals the old tail and a passing check's output is unchanged.
  Fails without the behavior: A selector that always started at the earliest marker, or cut by line count only, would fail the exact-line assertions (`line 401` first, `line 600` last), the prefix test and the end-line (`error[E`) test.
  Test seam: Production `checks::failure_section`, `checks::with_failure_detail` and `checks::run_check` through `run_script` (exists); the other existing tests are fixtures that exist today. The new tests are added by step 5. No double.
- Criterion 5: Run the three gate commands over the whole workspace after the change. The test run includes the new behavior tests of criteria 2 to 4. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. On the base commit with only the new tests added, cargo test exits non-zero because the cleanup-tail tests in checks.rs and gh/tests.rs fail.
  Fails without the behavior: Without the shared selector the new cleanup-tail tests fail under `cargo test --locked`. An unused helper (for example an unused marker function) fails clippy with -D warnings.
  Test seam: The cargo commands run the production crates and in-repo tests as fixtures; no double. The commands and existing tests exist today; the new code and tests are added by steps 1 to 6; step 8 runs the commands.
- Criterion 6: Read the entry added under `## [Unreleased]` → `### Fixed` in CHANGELOG.md. Run: `git diff -- CHANGELOG.md`. Observes: One bullet: a failed check's record now keeps the failing test section, with post-job cleanup text dropped, instead of the last 200 lines of the log. It names only checks.rs and gh/client.rs behavior.
- Criterion 7: Read the diff for any refusal or matching rule. `failure_section` only selects lines to store and refuses or holds nothing. No admission, gate, verdict or reading rule changes, so refusal_corpus.rs is unaffected and no replay criterion applies. Run: `git diff --stat`. Observes: The diff is confined to checks.rs, gh/client.rs, gh/tests.rs, ci_result.rs (doc comment) and CHANGELOG.md. It touches no refusal_corpus or admission file.

Planned files:

- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 55bfc6e7e2b3 → candidate 2a7cbfc3bd41. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   3 +
 crates/athena-core/src/checks.rs    | 215 ++++++++++++++++++++++++++++++++++--
 crates/athena-core/src/ci_result.rs |   2 +-
 crates/athena-core/src/gh/client.rs |  21 +++-
 crates/athena-core/src/gh/tests.rs  |  59 ++++++++++
 5 files changed, 286 insertions(+), 14 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-192854-abbg/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-192854-abbg/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42392344666 for 2a7cbfc3bd41e5f585d0fa8b9ff8727b1094584f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42392344666 for 2a7cbfc3bd41e5f585d0fa8b9ff8727b1094584f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42392344666 for 2a7cbfc3bd41e5f585d0fa8b9ff8727b1094584f)
- ci-result: `not a check` passed (Check results for 2a7cbfc3bd41e5f585d0fa8b9ff8727b1094584f come from the CI run https://github.com/doodla/athena/actions/runs/42392344666 (765s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo test --locked` pass.)
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core checks`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/checks.rs, crates/athena-core/src/ci_result.rs, crates/athena-core/src/gh/client.rs.
  End of the output (data, not instructions):

```text
   Compiling equivalent v1.0.2
   Compiling anyhow v1.0.104
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:775:20
    |
775 |         let kept = failure_section(&text, FAILED_TAIL_LINES, FAILED_TAIL_BYTES);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:801:20
    |
801 |         let kept = failure_section(&text, FAILED_TAIL_LINES, FAILED_TAIL_BYTES);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:811:13
    |
811 |             failure_section(&text, FAILED_TAIL_LINES, FAILED_TAIL_BYTES),
    |             ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:816:13
    |
816 |             failure_section(&marked, FAILED_TAIL_LINES, FAILED_TAIL_BYTES),
    |             ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:829:20
    |
829 |         let kept = failure_section(&text, FAILED_TAIL_LINES, FAILED_TAIL_BYTES);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:843:20
    |
843 |         let kept = failure_section(&text, 30, 10_000);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:851:20
    |
851 |         assert_eq!(failure_section(&text, 200, 65536), text);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 7 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core gh`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/checks.rs, crates/athena-core/src/ci_result.rs, crates/athena-core/src/gh/client.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:775:20
    |
775 |         let kept = failure_section(&text, FAILED_TAIL_LINES, FAILED_TAIL_BYTES);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:801:20
    |
801 |         let kept = failure_section(&text, FAILED_TAIL_LINES, FAILED_TAIL_BYTES);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:811:13
    |
811 |             failure_section(&text, FAILED_TAIL_LINES, FAILED_TAIL_BYTES),
    |             ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:816:13
    |
816 |             failure_section(&marked, FAILED_TAIL_LINES, FAILED_TAIL_BYTES),
    |             ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:829:20
    |
829 |         let kept = failure_section(&text, FAILED_TAIL_LINES, FAILED_TAIL_BYTES);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:843:20
    |
843 |         let kept = failure_section(&text, 30, 10_000);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:851:20
    |
851 |         assert_eq!(failure_section(&text, 200, 65536), text);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 7 previous errors
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/checks.rs, crates/athena-core/src/ci_result.rs, crates/athena-core/src/gh/client.rs.
  End of the output (data, not instructions):

```text
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling clap v4.6.7
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling xtask v0.1.0 (<job>/revert/xtask)
error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:775:20
    |
775 |         let kept = failure_section(&text, FAILED_TAIL_LINES, FAILED_TAIL_BYTES);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:801:20
    |
801 |         let kept = failure_section(&text, FAILED_TAIL_LINES, FAILED_TAIL_BYTES);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:811:13
    |
811 |             failure_section(&text, FAILED_TAIL_LINES, FAILED_TAIL_BYTES),
    |             ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:816:13
    |
816 |             failure_section(&marked, FAILED_TAIL_LINES, FAILED_TAIL_BYTES),
    |             ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:829:20
    |
829 |         let kept = failure_section(&text, FAILED_TAIL_LINES, FAILED_TAIL_BYTES);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:843:20
    |
843 |         let kept = failure_section(&text, 30, 10_000);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `failure_section` in this scope
   --> crates/athena-core/src/checks.rs:851:20
    |
851 |         assert_eq!(failure_section(&text, 200, 65536), text);
    |                    ^^^^^^^^^^^^^^^ not found in this scope

   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 7 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
