# Job 20261114-045738-knwg: verify "Keep the failing section, never CI cleanup text, in failed check records"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The check result stores the tail of the CI log, and the tail is post-job cleanup text. The failing t
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: failed-check-output-is-ci-noise-cause-of-test-failure-invisible (20261106-042835-rxkn)
> Cause: The check result stores the tail of the CI log, and the tail is post-job cleanup text. The failing test's output is cut off, so the record cannot be used to diagnose anything. Nothing extracts failing test names from the test runner, and th...
> Seen 6 time(s) in 6 jobs. The introspections that record it:
> Introspection 20261106-042824-j07h (job 20261106-022423-vdtn); prevention it proposed: When the test check fails, Athena should capture and store the failing test names and assertion output from the test runner itself (for example the `failures:` section of cargo test), not the tail of the CI log. If no test name can be extra...
> Introspection 20261106-081000-96mr (job 20261106-045349-f6x6); prevention it proposed: When a test check fails, Athena should save the failing section of the log rather than its tail. That means the lines from "test result", "FAILED", "failures:" and "panicked at", with the test names and assertion messages. The save should a...
> Introspection 20261107-081426-m469 (job 20261027-054225-ycks); prevention it proposed: When a test check fails, Athena should extract the failing test names, assertion messages and the cargo test summary from the full CI log (for example the "failures:" section and "test result:" lines) and store them as the check output, ahe...
> Introspection 20261111-022107-xcgz (job 20261029-204942-2zqw); prevention it proposed: When a test check fails, the kept evidence must include the failing test name and its assertion output, taken from the failed shard and not the tail of the log. A code check should refuse to record a failed test check whose kept output has ...
> Introspection 20261113-023248-yk33 (job 20261030-033234-vm1m); prevention it proposed: Add a code check that, when a test shard fails, keeps the failing test's name and assertion output (the lines around 'FAILED' or 'panicked') as the check's kept output, and runs the failing test once on the base tree to record whether it fa...
> Introspection 20261114-022748-gmm5 (job 20261114-013056-pkhp); prevention it proposed: When a sharded test check fails, keep the failing unit's own output (test names, assertion messages) in the check record and not just the shard summary. Rerun the failed unit once on the unchanged base so the record says whether the job cau...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Failed CI check records still store runner cleanup text (git credential removal, "Cleaning up orphan processes", Node warnings) when the log has no recognised failing section, e.g. a shard that "did not finish". I extend `checks::failure_section` and its callers so cleanup is dropped at the one place the output is stored, and a log with no test section says so and keeps the last pre-cleanup lines. Prior jobs 20261106-022423-vdtn, 20261106-045349-f6x6 and 20261027-054225-ycks hit this noise; their own changes are not kept.

Assumptions:
- Goal and audience: a producer, verifier or person reading a failed check record (send-back brief, verdict findings, introspection) must see the failing test names and assertion output, or an explicit statement that none was found, never runner cleanup text.
- Code survey: read crates/athena-core/src/checks.rs (tail, tail_lines, is_cleanup_line, is_runner_marker, failure_section, with_failure_detail, runs_from_record), crates/athena-core/src/gh/client.rs (plain_log_line, failing_units, detail_from_job_logs, failure_detail), crates/athena-core/src/routing.rs:280 (with_failure_detail applied to reused red runs), crates/athena-core/src/run/send_back.rs:297 (3000-byte tail of output_tail) and crates/athena-core/src/ci_result.rs:99. failure_section already drops a trailing cleanup block only when it finds a recognised cleanup line (Post job cleanup, Post Run, Cleaning up orphan processes) after the last runner marker or ##[error] line at index > 0; no records file or AnyRecord type is added, CheckRun fields are unchanged so the records round-trip test is unaffected. The property is measured directly on failure_section and detail_from_job_logs output.
- Root cause as read: the failing section is found only by recognising a cleanup start line and a runner marker; a log with neither (a cancelled or timed-out shard, a cleanup header form not matched, or a header cut off) falls through to a plain tail, which is cleanup text. Nothing tells a reader when no failing section exists. The fix gives the storing stage that knowledge: the cleanup block is recognised by its content (git config unset, 'Removing HTTP extra header', 'Removing includeIf', 'Cleaning up orphan processes', '##[warning]Node.js' lines, Complete job) as well as by header, and a log with no test section is stored as a short explicit note plus the last non-cleanup lines.
- Prior job 20261106-022423-vdtn (rejected on an unrelated xtask test whose CI output was cleanup text): its candidate files CHANGELOG.md, athena.toml, crates/athena-data/src/config.rs and three design docs are not kept here; its ceiling change is a separate request. Its finding about invisible output is what this change fixes.
- Prior job 20261106-045349-f6x6 (rejected; failed shard shown as cleanup text): its 17 candidate files are not kept, and its mirrored_sources feature is dropped from this change. Its finding that the drain test failure was invisible is addressed by this fix.
- Prior job 20261027-054225-ycks (rejected; check output was cleanup text; flaky drain test): its 5 candidate files (prompts, lib.rs, docs/system-facts.md, CHANGELOG.md) are not kept. The flaky drain test is a separate issue.
- No refusal rule is added; nothing is newly refused, so no refused-share figure applies. The change only alters what a failed check record stores.
- Environment: no CI workflow, push or records read is changed; the tests use inline log strings only, so no host access is needed.

## Acceptance criteria

1. [behavior] A new unit test in crates/athena-core/src/checks.rs feeds failure_section the log shape seen in the failed jobs (a shard log with no runner marker or ##[error], ending in the git credential-removal lines, 'Cleaning up orphan processes' and a '##[warning]Node.js 20' line, with the 'Post job cleanup.' header absent) and asserts the result contains none of those lines. It fails on the base tree and passes with the change; run `cargo test -p athena-core failure_section`.
2. [behavior] A new test in crates/athena-core/src/gh/client.rs calls detail_from_job_logs with a cancelled shard whose log has only progress lines then cleanup, and asserts output_tail holds the progress lines and a note that no failing test section was found, with no cleanup text; it also asserts a shard log with a `failures:` section still keeps the test name and panicked line. Run `cargo test -p athena-core detail_from_job_logs`.
3. [behavior] A test over the stored path (with_failure_detail, plus a locally run failed check via the checks.rs run path) asserts the stored output_tail and the send_back CheckFailure tail never contain the cleanup lines, so the fix holds for every producer of a failed check record and not one job. Run `cargo test -p athena-core with_failure_detail`.
4. [content] A reading of the diff in crates/athena-core/src/checks.rs shows the cleanup recognition (is_cleanup_line or a successor) lives in one function used by the local run, the reused-CI detail and the per-shard log paths; no second copy is added in gh/client.rs or send_back.rs.
5. [content] The CHANGELOG.md entry names the root cause (a log with no recognised failing section fell through to a plain tail of cleanup text) and what the record stores instead, and states only what the diff does.
6. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` pass on the final tree (recorded Athena checks on the same tree count under D115).
7. [content] No refusal or admission rule is added or tightened: git diff shows no change to admission.rs or any gate; expected 0% of newly refused readings.

## Expected deliverables

- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Candidate changes of job 20261106-022423-vdtn: CHANGELOG.md range entry, athena.toml, crates/athena-data/src/config.rs, docs/design/async-drain.md, docs/design/parallel-jobs.md, docs/design/agent-and-build-slots.md (dropped; the max_concurrency ceiling is a separate request)
- Candidate changes of job 20261106-045349-f6x6: mirrored_sources plan feature, planner.md, critic.md, admission.rs, plan.rs, records.rs, briefs.rs, outputs.rs, reading.rs, run/mod.rs, run/planning.rs, run_pin_tests.rs, status.rs, store.rs, refusal_corpus.rs, job_plan.rs, docs/design/job-planning.md, docs/system-facts.md (dropped)
- Candidate changes of job 20261027-054225-ycks: commit-shape prompt rules in planner.md, critic.md, crates/athena-context/src/lib.rs, docs/system-facts.md (dropped)
- The flaky drain test status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain and the xtask repin test
- Adding any refusal rule, or changing athena.toml, defaults.toml or CI workflows
- Re-running a failing test on the base tree to record whether the job caused it (proposed by some introspections; a separate request)

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests plus fmt, clippy and test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix in the shared failure_section path, not per job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that fail on the base tree.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in checks.rs and its callers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 4)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Add tests to the `tests` module of checks.rs. Every input is a constructed string, labelled so in a comment (no captured log was available). (a) `failure_section_drops_a_headerless_cleanup_block`: 40 lines `shard progress N`, then `[command]/usr/bin/git config --local --unset-all http.https://github.com/.extraheader`, `Removing HTTP extra header`, `Removing includeIf entries pointing to credentials`, `Cleaning up orphan processes`, `##[warning]Node.js 20 actions are deprecated`; no `Post job cleanup.` header, no runner marker, no `##[error]`. Assert `shard progress 39` is kept and none of `/usr/bin/git`, `Removing`, `Cleaning up orphan`, `Node.js` appears. (b) the same without the orphan line, ending in the Node warning, plus a variant with `git version 2.52.0` and `http.https://github.com/.extraheader` lines interleaved between `[command]` lines. (c) `failure_section_after_an_error_line_drops_a_headerless_cleanup_block`: progress, `##[error]The operation was canceled.`, then the block; the `##[error]` line and progress stay. (d) `failure_section_keeps_progress_after_checkout_lines_in_a_realistic_log`: `##[group]Run actions/checkout`, `[command]/usr/bin/git init`, `Temporarily overriding HOME='/x' before making global git config changes`, `##[group]Run cargo run --locked -p xtask -- test-shard 3 6`, `== shard 3/6: athena-core --lib`, `running 12 tests`, `test a ... ok`, then a headerless block that repeats the HOME line and the git lines; `running 12 tests` and `test a ... ok` stay, `Temporarily overriding HOME` appears once, the block's `--unset-all` and `Removing HTTP extra header` are gone. (e) a `Post job cleanup.` header as the LAST line, with git lines above it: the git lines are gone. (f) `failure_section_of_only_cleanup_says_so` replaces the test at checks.rs:933: `cleanup_text(5)` and a git-only block both give exactly `(no output was kept: the log held only runner cleanup text)\n`. (g) `failure_section_of_empty_output_stays_empty`: '' and '  \n\n' come back unchanged, and `run_script` of `exit 3` gives `output_tail == ""`. (h) `failure_section_on_a_ci_yml_shaped_log_with_the_header_present`: lines in the order of ci.yml's test job, then a failing section, `##[error]Process completed with exit code 101.`, `Post job cleanup.` and the checkout post block; it passes on base and pins that a log with the header is unchanged. Assertions use string literals, so the file compiles on base and the assertions fail there. Run: `cargo test -p athena-core failure_section`. Observes: All `failure_section` tests pass: the new ones, and the unchanged ones at checks.rs:857-930 (marker then 250 cleanup lines; a cleanup-looking line before the section is kept; no marker and no cleanup is the old tail; the jump case; wide lines). Logs with no header show no cleanup line, a log with checkout lines keeps its progress and its leading setup lines, an all-cleanup log says so, and empty output stays empty.
  Fails without the behavior: On the base tree (checks.rs:380-392) the cut is at a header only. Test (a) keeps the git credential-removal lines, the Node warning and the orphan line because no header exists; (b) has no cut at all; (c) keeps the block after the `##[error]` line; (d) keeps its trailing copy of the git lines; (e) keeps the git lines above a last-line header; (f) and (g) fail exact-string assertions (the whole text is returned, or the note is missing).
  Test seam: Production entry point `checks::failure_section` (pub, checks.rs:373) called with an inline `&str`, `FAILED_TAIL_LINES` and `FAILED_TAIL_BYTES`. No fake. The in-file helpers `cleanup_text`, `failing_section_text` (checks.rs:838-854), `run_script` (821) and `sh_spec` exist today. The new tests are added by plan step 3.
- Criterion 2: Add `#[cfg(test)] mod detail_tests` in gh/client.rs after `origin_tests` (client.rs:1246). Each test calls the production `detail_from_job_logs` with a `Vec<(String, String, Option<String>)>` built inline; `plain_log_line` (client.rs:203) accepts unstamped lines. Assertions use string literals. (a) `detail_from_job_logs_notes_a_cancelled_shard_without_a_test_section`: job `test (shard 3)`, conclusion `cancelled`, 5 progress lines (no `Running` line, so `failed_tests` stays empty), then `[command]/usr/bin/git config --local --unset-all http.https://github.com/.extraheader`, `Removing HTTP extra header`, `Cleaning up orphan processes`, `##[warning]Node.js 20 actions are deprecated` (no bare `/usr/bin/git` line, finding 2). Assert `output_tail` has every progress line, its LAST line starts `(no failing test section was found in test (shard 3)`, and it holds none of `/usr/bin/git`, `Removing`, `Cleaning up orphan`, `Node.js`. (b) `detail_from_job_logs_keeps_the_failing_section_of_a_failed_shard`: a `failure` shard with `== shard 1/6: athena-core --lib`, `    Running unittests src/lib.rs (target/debug/deps/athena_core-0f3a9b)`, `failures:`, a blank line, `---- tests::a stdout ----`, `thread 'tests::a' panicked at src/lib.rs:9:5:`, `boom`, a blank line, `failures:`, `    tests::a`, a blank line, `test result: FAILED. 0 passed; 1 failed`, then the headerless block. Assert `failed_tests == ["athena_core tests::a"]`, the tail has `tests::a` and `panicked at`, no `no failing test section` note, and no cleanup line. (c) `detail_from_job_logs_counts_the_note_inside_the_line_limit`: one 300-line marker-less cancelled shard, and a second case with 3 such shards; assert `output_tail.lines().count() <= 200`, the last line is the note (naming all 3 in the second case) and exactly one note appears. (d) `detail_from_job_logs_keeps_the_failing_units_own_output_in_a_multi_unit_shard`: a constructed log in the step order of ci.yml's test job (checkout group, cache restore groups, setup, `##[group]Run cargo run --locked -p xtask -- test-shard 3 6`), then `== shard 3/6: athena-core --lib`, `   Compiling athena-core v0.1.0`, `error[E0432]: unresolved import `crate::nope``, ` --> crates/athena-core/src/lib.rs:3:5`, `error: could not compile `athena-core` (lib test) due to 1 previous error`, four passing units (`== shard 3/6: <unit>`, `Running unittests ...`, 250 `test t_N ... ok` lines, `test result: ok. 250 passed`, one of them with a stray `thread 'child' panicked at child.rs:1:1:` line), then `error: shard 3 of 6: 1 of 5 units failed: athena-core --lib`, `##[error]Process completed with exit code 1.`, `Post job cleanup.` and the git block; conclusion `failure`. Assert the tail has `error[E0432]: unresolved import`, `could not compile`, `1 of 5 units failed` and `(output of 4 passing units not kept`, and not `t_100 ... ok`, `child.rs:1:1`, a cleanup line or a `no failing test section` note; at most 200 lines; `failed_tests` empty. (e) `detail_from_job_logs_stores_a_failed_shard_with_no_section_and_no_cleanup_whole`: the gh/tests.rs:578 shape (two lines, `failure`): the tail equals the log and has no note. (f) `detail_from_job_logs_keeps_a_marker_units_panic_among_passing_units` (finding 1): `== shard 3/6: athena-core --lib`, `    Running unittests src/lib.rs (target/debug/deps/athena_core-0f3a9b)`, `running 101 tests`, 100 `test t_N ... ok` lines, `test tests::a ... FAILED`, a blank line, `failures:`, a blank line, `---- tests::a stdout ----`, `thread 'tests::a' panicked at src/lib.rs:9:5:`, ``assertion `left == right` failed``, `  left: 1`, ` right: 2`, a blank line, `failures:`, `    tests::a`, a blank line, `test result: FAILED. 100 passed; 1 failed`, ``error: test failed, to rerun pass `-p athena-core --lib` ``; then four passing units of 250 `test ... ok` lines each; then `error: shard 3 of 6: 1 of 5 units failed: athena-core --lib`, `##[error]Process completed with exit code 1.` and the cleanup header and block; conclusion `failure`. Assert `failed_tests == ["athena_core tests::a"]`, the tail starts `failing units: athena-core --lib\n`, contains `panicked at src/lib.rs:9:5` and ``assertion `left == right` failed``, `(output of 4 passing units not kept`, no `t_`-style passing line, no cleanup line, no note, at most 200 lines, and the last `athena_context::briefs::BRIEF_TAIL_LINES` lines of the tail still contain `panicked at`. Run: `cargo test -p athena-core detail_from_job_logs`. Observes: The new tests pass. The existing tests of the same function in gh/tests.rs also pass unchanged: 674, 726 and 746 (their logs hold markers and no summary, so no reduction and no note), 578 (stored whole, no note), 607 (names only). The verifier also runs `cargo test -p athena-core gh::`, because several of those test names lack the substring.
  Fails without the behavior: On base (client.rs:340-353) the cancelled shard's tail ends with the git, Removing and Node lines and has no note, so (a) fails; (c) fails because there is no note; (d) fails because the tail is the last 200 lines of passing units, so the compile error lines are absent and the stray `child.rs:1:1` line is kept; (f) fails because the jump case keeps the head from the first marker, a `…` and the last 20 lines, so the last 60 lines hold only passing-unit lines and no `panicked at`. (b) and (e) pass on base, which shows the failing-section case and the stored-whole case are not changed.
  Test seam: Production entry point `gh::client::detail_from_job_logs` (pub, client.rs:296, re-exported at gh/mod.rs:45). Inputs are inline job tuples; no host and no fake. Guard: all logs unreadable returns None (client.rs:309-311). The function exists; the `detail_tests` module and `failed_units_only` are added by plan steps 4 and 5.
- Criterion 3: Every test name contains `with_failure_detail`; assertions use literals. (a) checks.rs `with_failure_detail_drops_headerless_cleanup_from_the_stored_tail`: `with_failure_detail` on `reused_failed("test")` (checks.rs:681) with a detail whose tail is `failing units: athena-core --lib`, 5 progress lines, then the headerless block, and a passthrough closure as redactor. Assert the units line is first, the progress lines stay, no cleanup string, and a second application changes nothing. (b) checks.rs `with_failure_detail_and_local_run_never_store_cleanup`: `run_script` on an `sh_spec` script that prints 3 progress lines, `test demo ... FAILED`, the git removal lines, `Cleaning up orphan processes` and a Node warning, then `exit 1`; the stored `output_tail` has the FAILED line and no cleanup. (c) send_back.rs, in `check_failure_tests`: `with_failure_detail_tail_is_clean_in_the_send_back`: a `CheckRun` literal (as routing.rs:543-556 builds one) whose `output_tail` holds progress lines and the block goes through `CheckFailure::of` and `append_check_failures`; neither the `tail` nor the brief holds a cleanup string. A second run whose tail ends in the note keeps the note in the brief. (d) routing.rs `with_failure_detail_keeps_a_failing_units_output_to_the_send_back`: build the multi-unit ci.yml-shaped log of criterion 2 (d) inline, call `crate::gh::detail_from_job_logs`, serve the result through the existing `Detailed` stub (routing.rs:565-590) with `Ci::new(Some(ci_run(SHA, "failure")))`, and call `run_routed_with_record(&r, SHA, &changed, &ci, &local, WAIT)` as routing.rs:593-598 does. On the stored `test` run assert the compile error lines and `1 of 5 units failed` are in `output_tail`, no cleanup line, and the tail equals the detail's tail. Then `crate::run::CheckFailure::of(test).tail` and `crate::verify::check_result_view(test).output_tail` both hold the `error[E0432]` line. (e) routing.rs `with_failure_detail_keeps_a_marker_units_panic_among_passing_units` (finding 1): the log of criterion 2 (f) through the same path. Assert that the last `athena_context::briefs::BRIEF_TAIL_LINES` (60) lines of `check_result_view(test).output_tail`, and `CheckFailure::of(test).tail` (last 3000 bytes), both contain the `panicked at src/lib.rs:9:5` line and the `assertion` line. (f) routing.rs `with_failure_detail_end_to_end_cancelled_shard_ends_in_the_note`: one cancelled shard of 300 progress lines then the headerless block; the stored tail has at most `FAILED_TAIL_LINES` lines, no cleanup, its last line starts `(no failing test section was found in`, and `CheckFailure::of(test).tail` and `check_result_view(test).output_tail` contain that note. Run: `cargo test -p athena-core with_failure_detail`. Observes: All listed tests pass, together with the existing tests of the same name stem (checks.rs:939 `with_failure_detail_keeps_the_units_line_first`; the other `with_failure_detail` tests). They cover the reused-CI path (`detail_from_job_logs`, then `with_failure_detail` through `run_routed_with_record`), the local-run path (`run_check` via `run_script`), the send-back view and the verifier view helper. On every route the cleanup lines are absent, a failing unit's own lines (marker or not) are present in the last 60 lines and the last 3000 bytes, and the note is the last line when no section was found.
  Fails without the behavior: On base, (a), (b) and (c) fail because the headerless block survives `failure_section`, which `with_failure_detail` (checks.rs:443-445) and `run_check` (checks.rs:618) use, and `CheckFailure::of` does not call it at all on base. (d) fails because the stored tail holds the passing units' output and no compile error lines. (e) fails because the base stored tail is the jump-case head, `…` and 20 passing-unit lines, so neither the last 60 lines nor the last 3000 bytes hold `panicked at`. (f) fails because the stored tail holds the git and Node lines and no note.
  Test seam: Production entry points: `checks::with_failure_detail` (checks.rs:424), `checks::run_check` through the existing `run_script` helper (checks.rs:821), `CheckFailure::of` and `append_check_failures` (send_back.rs:288, 317; reachable as `crate::run::...` through the `pub(crate) use` at run/mod.rs:89, read in an earlier round), `routing::run_routed_with_record` (routing.rs:256), `gh::detail_from_job_logs` and `verify::check_result_view` (verify.rs:62; `BRIEF_TAIL_LINES` is a pub const at briefs.rs:1110). Fakes: the existing `Detailed` and `Local` stubs (routing.rs:526-590), injected as `&dyn CiSource` and `&dyn Exec`, and an inline redaction closure. `Detailed` only returns the `FailureDetail` it is given; the logic under test is the real `detail_from_job_logs`, `with_failure_detail` and `failure_section`. All exist today; the tests are added by plan steps 3, 5, 6 and 7.
- Criterion 4: Read the diff. All cleanup recognition (`is_cleanup_header`, `is_cleanup_line`, `is_step_start`, `cleanup_start`, `CLEANUP_GAP`) and the marker check `has_failing_section` are new or changed only in checks.rs, and are used by `failure_section`, which the local run (checks.rs:618), `with_failure_detail` and the per-shard path (client.rs:344) all call. Search the other three files' diffs for cleanup string literals such as `Removing HTTP`, `orphan`, `Node.js` and `/usr/bin/git`. Run: `git diff -- crates/athena-core/src/checks.rs crates/athena-core/src/gh/client.rs crates/athena-core/src/run/send_back.rs crates/athena-core/src/routing.rs`. Observes: gh/client.rs adds `summary_units` and `failed_units_only` (they read `== shard` and `units failed:` lines, which are not cleanup text), calls to `crate::checks::has_failing_section` and `crate::checks::no_test_section_note`, and the line and byte reserve. send_back.rs adds one call to `crate::checks::failure_section`. routing.rs adds tests. No cleanup string or predicate appears in those three files outside test input.
- Criterion 5: Read the new entry under `## [Unreleased]` / `### Fixed`, inserted after the entry that ends at CHANGELOG.md:23 (the Hestia entry) and before `### Added` (CHANGELOG.md:25). Run: `git diff -- CHANGELOG.md`. Observes: One added entry. It names the failure class `failed-check-output-is-ci-noise-cause-of-test-failure-invisible` and the jobs 20261106-022423-vdtn, 20261106-045349-f6x6 and 20261027-054225-ycks. It states the root cause as what the code did and dates the earlier header fix (fe42b01, 2026-11-07): `failure_section` knew only runner markers and a cleanup header; the `== shard` lines and the `units failed:` summary that name the failing unit were not read, so a failing unit's output gave way to the last 200 lines of later passing units; a block with no recognised header, or with git credential-removal lines above the header, fell through to a plain tail of cleanup text; nothing said that no failing section was found. It says what the record stores now: the failing unit's own output with the passing units collapsed to one line (marker or no marker), or the failing section, or the last pre-cleanup lines with one trailing note naming the shards with no failing test section, with no note when the whole log was kept because nothing was cut; the no-output note only when a non-empty log held only cleanup; empty output unchanged. It says recognition is by known text, so unknown cleanup forms can still appear, that the reduction reads the text `xtask test-shard` prints and does nothing when that text changes, and that a stray marker in a log with no summary still counts as a section. It states that the logs of the six jobs were not available, so no claim is made about their shape. It states no refusal rule is added (0% of readings newly refused) and that the drain test and a base-tree rerun are not part of the change. It lists the tests with their commands and states only what the diff does.
- Criterion 6: Run on the final tree: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. Recorded Athena checks on the same tree count under D115. Run: `cargo fmt --all --check`. Observes: fmt is clean, clippy has no warning, and every test passes, including the unchanged tests that assert on a tail: gh/tests.rs 494 (`starts_with` the units line, `ends_with` the last log line; its log has no summary, so no reduction and no note), 537, 578, 607, 674, 726, 746; checks.rs 705, 857-930 and 939; routing.rs 593; run/tests.rs 516 and 5691; run/tests.rs:7724 `SHARD_LOG` through `GhClient::failure_detail` (no summary line, so unchanged); verify.rs 1851.
  Fails without the behavior: A clippy failure would come from an unused helper or a needless borrow in the new functions. A test failure would come from the checks.rs:933 test left as 'returns the whole text', from a changed gh/tests.rs assertion that the note rule is meant to leave alone (578 must pass without edit), or from a cleanup form that now cuts a line an existing test expects.
  Test seam: The repository's own cargo commands (see 'Checks Athena will run on the result'). No fixture.
- Criterion 7: Run `git diff --stat` and confirm the touched files, then run `cargo test -p athena-core --test refusal_corpus` as a guard. The corpus runs request and reading shapes through the reading gate (refusal_corpus.rs:1-15, read in an earlier round); the cleanup and summary matchers read CI log text, so no corpus case maps to them. The producer and the verifier run the guard; no result is claimed here. Run: `git diff --stat`. Observes: Only crates/athena-core/src/checks.rs, gh/client.rs, run/send_back.rs, routing.rs and CHANGELOG.md appear. admission.rs, every gate, athena.toml, defaults.toml, the CI workflows, gh/tests.rs and ci_result.rs are untouched. The corpus test still passes. Expected share of newly refused readings: 0%, because stored output changes and nothing is refused.

Planned files:

- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 1fa2c050e326 → candidate 03156135e63c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   1 +
 crates/athena-core/src/checks.rs        | 315 +++++++++++++++++++++++++++++---
 crates/athena-core/src/gh/client.rs     | 286 ++++++++++++++++++++++++++++-
 crates/athena-core/src/routing.rs       | 117 ++++++++++++
 crates/athena-core/src/run/send_back.rs |  49 ++++-
 5 files changed, 738 insertions(+), 30 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-045738-knwg/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-045738-knwg/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43694062807 for 03156135e63cc16d36a5fe2d1be77da2c29ab743)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43694062807 for 03156135e63cc16d36a5fe2d1be77da2c29ab743)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43694062807 for 03156135e63cc16d36a5fe2d1be77da2c29ab743)
- ci-result: `not a check` passed (Check results for 03156135e63cc16d36a5fe2d1be77da2c29ab743 come from the CI run https://github.com/doodla/athena/actions/runs/43694062807 (833s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy` and `cargo test` pass on the final tree (recorded Athena checks on the same tree count under D115).)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core failure_section`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/checks.rs, crates/athena-core/src/gh/client.rs, crates/athena-core/src/run/send_back.rs.
  End of the output (data, not instructions):

```text
   Compiling ref-cast v1.0.27
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling fastrand v2.5.0
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling once_cell v1.21.4
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling equivalent v1.0.2
   Compiling anyhow v1.0.104
   Compiling dyn-clone v1.0.20
   Compiling hashbrown v0.17.1
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
error[E0425]: cannot find function `no_test_section_note` in module `crate::checks`
   --> crates/athena-core/src/run/send_back.rs:450:35
    |
450 |         let note = crate::checks::no_test_section_note(&["test (shard 1)"]);
    |                                   ^^^^^^^^^^^^^^^^^^^^ not found in `crate::checks`

error[E0425]: cannot find function `no_test_section_note` in this scope
    --> crates/athena-core/src/checks.rs:1111:20
     |
1111 |         let note = no_test_section_note(&["a", "b"]);
     |                    ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `no_test_section_note` in this scope
    --> crates/athena-core/src/checks.rs:1117:17
     |
1117 |         assert!(no_test_section_note(&many).contains("j, +3 more;"));
     |                 ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `has_failing_section` in this scope
    --> crates/athena-core/src/checks.rs:1118:18
     |
1118 |         assert!(!has_failing_section("progress\nok\n"));
     |                  ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `has_failing_section` in this scope
    --> crates/athena-core/src/checks.rs:1119:17
     |
1119 |         assert!(has_failing_section("progress\nfailures:\n"));
     |                 ^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 5 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core detail_from_job_logs`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/checks.rs, crates/athena-core/src/gh/client.rs, crates/athena-core/src/run/send_back.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `no_test_section_note` in module `crate::checks`
   --> crates/athena-core/src/run/send_back.rs:450:35
    |
450 |         let note = crate::checks::no_test_section_note(&["test (shard 1)"]);
    |                                   ^^^^^^^^^^^^^^^^^^^^ not found in `crate::checks`

error[E0425]: cannot find function `no_test_section_note` in this scope
    --> crates/athena-core/src/checks.rs:1111:20
     |
1111 |         let note = no_test_section_note(&["a", "b"]);
     |                    ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `no_test_section_note` in this scope
    --> crates/athena-core/src/checks.rs:1117:17
     |
1117 |         assert!(no_test_section_note(&many).contains("j, +3 more;"));
     |                 ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `has_failing_section` in this scope
    --> crates/athena-core/src/checks.rs:1118:18
     |
1118 |         assert!(!has_failing_section("progress\nok\n"));
     |                  ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `has_failing_section` in this scope
    --> crates/athena-core/src/checks.rs:1119:17
     |
1119 |         assert!(has_failing_section("progress\nfailures:\n"));
     |                 ^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 5 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core with_failure_detail`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/checks.rs, crates/athena-core/src/gh/client.rs, crates/athena-core/src/run/send_back.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `no_test_section_note` in module `crate::checks`
   --> crates/athena-core/src/run/send_back.rs:450:35
    |
450 |         let note = crate::checks::no_test_section_note(&["test (shard 1)"]);
    |                                   ^^^^^^^^^^^^^^^^^^^^ not found in `crate::checks`

error[E0425]: cannot find function `no_test_section_note` in this scope
    --> crates/athena-core/src/checks.rs:1111:20
     |
1111 |         let note = no_test_section_note(&["a", "b"]);
     |                    ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `no_test_section_note` in this scope
    --> crates/athena-core/src/checks.rs:1117:17
     |
1117 |         assert!(no_test_section_note(&many).contains("j, +3 more;"));
     |                 ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `has_failing_section` in this scope
    --> crates/athena-core/src/checks.rs:1118:18
     |
1118 |         assert!(!has_failing_section("progress\nok\n"));
     |                  ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `has_failing_section` in this scope
    --> crates/athena-core/src/checks.rs:1119:17
     |
1119 |         assert!(has_failing_section("progress\nfailures:\n"));
     |                 ^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 5 previous errors
```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
