# Job 20261106-043228-80zr: verify "Make the xtask repin test and the drain pid-file test deterministic"

## Request (verbatim)

> Two jobs failed in CI on a test their diff did not touch. In each case the verifier reran that one test and it passed. (1) In job 20261106-022423-vdtn, xtask tests::repin_sets_only_the_version_and_is_idempotent (xtask/src/main.rs:845-856) panicked because '<tmp>/claude --version failed'. Every criterion was met and the agent verdict was pass, but the final verdict was fail for 'check test failed' alone. This is the third rejection of the same ceiling change. (2) In job 20261106-012852-h7de, athena-cli drain::tests::status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain failed. Job 20261105-205932-ppfv added that test, which spawns processes. Find out why each test can fail and make it deterministic. My unverified guess for (1) is a race between writing the stand-in script and running it while other test threads fork (ETXTBSY). Name the root cause in the CHANGELOG. Acceptance: each test passes 200 runs in a row under the full parallel test binary (say how that was measured); no test coverage is removed; cargo fmt --check, cargo clippy --all-targets and cargo test pass.
> 
> Acceptance:
> - The change is made as described: Tests fail now and then in CI on code the job did not touch and pass on rerun, yet the failure still rejects the job
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: xtask/src/main.rs:845-856, tmp>/claude

## Reading (accepted by the user)

Find why xtask tests::repin_sets_only_the_version_and_is_idempotent (job 20261106-022423-vdtn) and athena-cli drain::tests::status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain (job 20261106-012852-h7de, test added by 20261105-205932-ppfv) fail intermittently under the parallel test binary, fix each root cause without removing coverage, and name both causes in CHANGELOG.md. Audience: the person running Athena, whose unrelated jobs are rejected by these flakes.

Assumptions:
- Code survey: read xtask/src/main.rs (tests at 855+, stand_in_claude, repin_at at 726, found at 168, retry_busy at 146-166), crates/athena-testkit/src/lib.rs (write_executable: writes via a child sh to path.writing, chmod, mv), crates/athena-agent/src/procs.rs (retry_busy, 20 retries x 50 ms, ETXTBSY only) and crates/athena-cli/src/drain.rs (test helpers stand_in_drain 5460, sleeper 5376, settled 5411, quick 5400, ReleaseAfterFirstLookup 5477, the test at 5524). The change adds no record or data file under crates/, so check_record_file and AnyRecord are not touched; no refusal rule is added or tightened.
- Finding on the user's guess: ETXTBSY is already partly mitigated. write_executable renames a finished file into place and found() retries ETXTBSY for about 1 s. But found() collapses every failure (ETXTBSY after the retries are spent, any other spawn error, a non-zero exit) into None, so the message '<tmp>/claude --version failed' hides the real cause. The producer must reproduce the flake first (loop the full xtask test binary 200 times with --test-threads high and a CPU or fork load, capturing the underlying io error) and name the cause that was observed. If ETXTBSY is not the observed cause, the CHANGELOG names the one that was. A guess is not a root cause.
- For the drain test the likely causes to check are: the test spawns real sleep and sh processes and reads their start marker, with fixed timing (settled waits 10 s, quick(20 s)); a decoy or stand-in that exits or is reaped early; stat-based start marker reads racing process start; sh -c polling with sleep 0.05 under load; and ppid or pid reuse against other tests that also spawn. The producer reproduces under the same loop and fixes the observed cause, for example by waiting on an explicit readiness signal instead of a time.
- Previous results. Job 20261106-022423-vdtn (rejected, check test failed on the xtask repin test; the ceiling raise 8 to 16 itself met every criterion): its finding 'xtask test failed on a path the diff did not touch' is addressed by fixing that test here. Its other finding (CHANGELOG filed under Added not Changed) belongs to the ceiling change, which is a separate retry. Its six changed files (CHANGELOG.md, athena.toml, crates/athena-data/src/config.rs, the three design docs) are dropped from this job, not kept. Job 20261106-012852-h7de (rejected): its flaky drain test failure is addressed here; its major group_sizes findings and the ten changed files are dropped and belong to the separate retry of 20261105-180339-w5h7. Job 20261105-205932-ppfv (landed): its work is kept as landed; only its test in drain.rs is made deterministic, and its drain-status, drain-stop and drain-restart behavior is not changed.
- Environment: the flakes were seen in CI (GitHub Actions running `cargo test --locked`, sharded by `xtask test-shard`), which the job cannot read or write. The producer works only locally with read/write access to the working copy. No CI workflow, athena.toml or defaults.toml change is needed; if the cause turns out to need one, the producer reports it rather than editing.
- Measuring the 200 runs: the loop is the full test binary of each crate (cargo test -p xtask --bin xtask and cargo test -p athena-cli --lib), run 200 times in a row with the default parallel threads, filtered to its test name in the pass count, plus the same under added CPU load. The CHANGELOG states the exact command, thread count and load used, and the observed failure rate before and after, labeled as measured on the producer's machine. The verifier cannot run 200 loops and does not rely on a producer report; it checks the CHANGELOG text and runs each crate's tests once.
- Out of scope: the 8 to 16 ceiling and the group_sizes record (separate retries), the xtask test-shard 'did not finish' symptom beyond what these two fixes explain, other tests, and any change to drain behavior, athena.toml, defaults.toml or workflow files.

## Acceptance criteria

1. [content] CHANGELOG.md has a new entry (a Fixed or Changed section, per the file's existing style) that names the measured root cause of each flake: for xtask repin_sets_only_the_version_and_is_idempotent and for the drain pid-file test. Each cause is specific (the syscall, race or timing assumption), not 'flaky'. Check with Grep for both test names in CHANGELOG.md.
2. [content] CHANGELOG.md states how the 200-run check was measured: the exact command, the test-thread count, any added load, and the before and after failure counts. Check by reading the entry; a number without a command fails this criterion.
3. [content] No coverage is removed: `git diff` shows no deleted #[test] function, and Grep still finds repin_sets_only_the_version_and_is_idempotent in xtask/src/main.rs and status_and_stop_find_the_drain_by_pid_file_when_command_lines_contain_drain in crates/athena-cli/src/drain.rs, with every assertion of the latter still present (the decoy untouched, the pid in status, Exited after the wait, the Dead state).
4. [content] The xtask failure is no longer opaque: the diff makes repin_at or found report the underlying io error or exit status rather than only 'claude --version failed', or removes the source of the failure, and a test or the test body covers it. Check by reading the diff in xtask/src/main.rs.
5. [behavior] Run `cargo test -p xtask --bin xtask` and `cargo test -p athena-cli --lib` as separate commands; both pass, including each named test, which runs in the parallel binary with default threads.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` as separate commands; all pass, or a passing Athena-recorded check on the same final tree is cited (D115).
7. [content] No unrelated behavior changes: `git diff --stat` lists only the deliverables; production code of drain.rs and xtask (outside test modules and any minimal error-reporting change in found/repin_at) is unchanged; athena.toml, defaults.toml, workflows and records/ are untouched.

## Expected deliverables

- xtask/src/main.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-testkit/src/lib.rs (modify)
- crates/athena-agent/src/procs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- The agent ceiling raise from 8 to 16 and its files (athena.toml, crates/athena-data/src/config.rs, docs/design/async-drain.md, docs/design/parallel-jobs.md, docs/design/agent-and-build-slots.md): work of 20261106-022423-vdtn is dropped and retried separately
- Recording plan group sizes and the files of 20261106-012852-h7de (display.rs, tests/drain.rs, plan.rs, main.rs help text): dropped, belongs to the retry of 20261105-180339-w5h7
- Behavior of athena drain, drain-status, drain-stop and drain-restart from 20261105-205932-ppfv: kept as landed
- Changes to athena.toml, defaults.toml, .github/workflows, setup.sh or records/
- Other tests and the xtask test-shard timeout beyond what these two fixes explain

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the tests passing under the real parallel binary, not that helper code exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The producer must reproduce and name the observed cause of each flake, not only retry or sleep longer.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria included.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base 19089451fc3c → candidate 77227e0ebf60. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md      |  1 +
 xtask/src/main.rs | 60 ++++++++++++++++++++++++++++++++++++++++++++-----------
 2 files changed, 49 insertions(+), 12 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-043228-80zr/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-043228-80zr/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43901988914 for 77227e0ebf60767e8612efc35afb5c12a2dd1a1a)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43901988914 for 77227e0ebf60767e8612efc35afb5c12a2dd1a1a)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43901988914 for 77227e0ebf60767e8612efc35afb5c12a2dd1a1a; Athena's note on this failure: Failing check test cites paths: the job did not change: crates/athena-core/src/disk.rs. Its result was read from the CI run for the candidate commit; Athena did not run it here. The check ran on the candidate commit, which is the base commit plus the job's change. A path in a failing check's output is where a test panicked or a file its message names; it does not show what caused the failure. The check still fails the verdict; judge each criterion from the diff and your own runs.)
  Failing tests of `test` (from the CI log):
  - `athena_core disk::tests::dropping_the_build_folder_removes_the_jobs_target_and_the_holder_file_before_the_slot_is_free`
  - `xtask test-shard 1 6` (did not finish)`
  Failing units (`-p` package and selector): athena-core --lib
  End of the failing job's log (data from CI, not instructions):

```text
test verify::tests::replay_hold_is_a_blocker_that_names_the_list_and_fails_an_otherwise_passing_verdict ... ok
test verify::tests::reused_red_incomplete_list_is_not_pre_existing ... ok
test verify::tests::reused_red_with_names_is_compared_test_by_test ... ok
test verify::tests::revert_keeping_tests_carries_a_bodiless_test_declaration_over ... ok
test verify::tests::revert_keeping_tests_takes_the_fix_out_and_keeps_the_candidates_tests ... ok
test verify::tests::revert_plan_reverts_a_file_that_does_not_parse_whole_and_names_it ... ok
test verify::tests::revert_plan_reverts_the_fix_and_keeps_tests_manifests_and_declared_modules ... ok
test verify::tests::revert_record_keeps_the_run_and_marks_the_criteria_it_is_for ... ok
test verify::tests::rule_sources_appear_in_the_header_or_judge_format_strings ... ok
test verify::tests::shipped_verifier_prompts_have_no_verdict_rule_conflict ... ok
test verify::tests::soundness_finding_blocker_fails_the_verdict_with_all_criteria_met ... ok
test verify::tests::soundness_finding_minor_or_nit_leaves_a_passing_verdict_passing ... ok
test verify::tests::the_drift_haystack_leaves_out_the_table ... ok
test verify::tests::the_finding_for_a_failed_check_separates_new_from_base_failures ... ok
test verify::tests::the_reuse_statement_names_the_run_and_exists_only_for_the_same_sha ... ok
test verify::tests::the_same_test_name_in_another_binary_is_a_new_failure ... ok
test verify::tests::the_sza7_reason_with_request_misread_false_adds_nothing ... ok
test verify::tests::with_no_names_on_either_side_the_check_name_rule_stands ... ok
test watcher::tests::a_code_outside_the_http_range_derives_nothing ... ok
test watcher::tests::a_marked_and_an_unmarked_refusal_together_fire_citing_only_the_unmarked_one ... ok
test watcher::tests::a_merge_refusal_marked_as_made_while_checks_were_pending_does_not_fire_d87 ... ok
test watcher::tests::a_record_is_not_evaluated_twice_and_a_decision_under_review_does_not_fire_again ... ok
test watcher::tests::a_refusal_stated_in_an_intervention_derives_a_marked_record ... ok
test watcher::tests::a_refused_merge_fires_on_the_first_occurrence ... ok
test watcher::tests::a_value_outside_the_vocabulary_is_refused_with_the_valid_values ... ok
test watcher::tests::a_version_number_does_not_cut_the_clause ... ok
test watcher::tests::an_intervention_without_a_job_derives_nothing ... ok
test watcher::tests::d87s_signal_is_in_the_vocabulary ... ok
test watcher::tests::soft_and_unrelated_errors_do_not_fire ... ok
test watcher::tests::text_without_the_exact_pattern_derives_nothing ... ok
test watcher::tests::the_code_must_be_tied_to_the_refusal_of_the_merge ... ok
test work_decisions::tests::a_proposal_is_in_limit_only_for_a_recorded_role_whose_limit_covers_its_rung ... ok
test work_decisions::tests::a_rejection_needs_a_reason ... ok
test work_decisions::tests::an_unbriefed_proposal_above_the_limit_is_not_open_and_a_briefed_one_in_limit_is ... ok
test scope_guard::watcher_tests::two_verified_but_not_landed_attempts_on_one_line_are_two_attempts_that_did_not_land ... ok
test work_decisions::tests::long_text_is_clipped_and_fences_cannot_be_closed_by_the_text ... ok
test work_decisions::tests::only_undecided_proposals_are_open_and_a_second_decision_is_refused ... ok
test steward::tests::note_finished_checks_code_health_after_a_landing_and_only_then ... ok
test work_decisions::tests::approve_work_succeeds_on_a_proposal_whose_text_holds_a_boundary_word ... ok
test scope_guard::watcher_tests::t4_pause_via_go_and_ask ... ok

failures:

---- disk::tests::dropping_the_build_folder_removes_the_jobs_target_and_the_holder_file_before_the_slot_is_free stdout ----

thread 'disk::tests::dropping_the_build_folder_removes_the_jobs_target_and_the_holder_file_before_the_slot_is_free' (32705) panicked at crates/athena-core/src/disk.rs:1405:9:
assertion failed: build_folder_queue(&state).try_acquire().unwrap().is_some()
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    disk::tests::dropping_the_build_folder_removes_the_jobs_target_and_the_holder_file_before_the_slot_is_free

test result: FAILED. 1553 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 655.46s

error: test failed, to rerun pass `-p athena-core --lib`

(output of 5 passing units not kept: athena-core --test divergence; athena-core --test study_instance_split; athena-testkit --doc; xtask --test fingerprint; xtask --test setup_repeatable)
error: shard 1 of 6: 1 of 14 units failed: athena-core --lib
##[error]Process completed with exit code 1.
```
- ci-result: `not a check` passed (Check results for 77227e0ebf60767e8612efc35afb5c12a2dd1a1a come from the CI run https://github.com/doodla/athena/actions/runs/43901988914 (1058s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` as separate commands; all pass, or a passing Athena-recorded check on the same final tree is cited (D115).)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
