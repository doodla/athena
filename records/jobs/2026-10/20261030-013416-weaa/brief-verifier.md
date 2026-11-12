# Job 20261030-013416-weaa: verify "Give OLY-B-O5-01 a mechanism: athena restart and status restarts"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-O5-01 a mechanism
> 
> Requirement OLY-B-O5-01 in docs/spec.toml is active and has no mechanism.
> Requirement: The person acting for Athena restarts after v0, at least once per milestone and when efficiency drops, and the state of the work comes from Athena.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-O5-01
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-O5-01 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Record the existing `athena restart` command (manage.rs) and the `restarts` line of `athena status` as the mechanism for OLY-B-O5-01 in docs/spec.toml, with a measure ending `status: restarts`. Add a test that records a restart and shows the status count rise from 0 to 1, so the requirement's state of the work comes from Athena. Audience: the owner reading the spec gap list.

Assumptions:
- Code survey: read crates/athena-core/src/manage.rs (`Manager::restart` validates --cost, redacts, writes an AnyRecord::Restart under records/restarts/ and commits it), crates/athena-core/src/store.rs (Records.restarts loaded from AnyRecord::Restart, sorted by `at`), crates/athena-core/src/status.rs (`restarts: records.restarts.len()`), crates/athena-cli/src/display.rs line 1411 (prints `restarts    N`) and crates/athena-cli/tests/lifecycle.rs (existing restart record fixture). No new record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test need no change; the existing Restart record type is reused.
- Existing code: crates/athena-core/src/manage.rs (restart), crates/athena-core/src/status.rs (restarts), crates/athena-cli/src/display.rs (restarts line) and docs/spec.toml are extended or reused; the new test calls the real restart path and the real status view.
- The mechanism is `athena restart` (the user records the cost of restarting their context) plus the `restarts` line of `athena status`, which reports the state from Athena's records. The mechanism is already in the code; the spec records it and a new test pins the behavior. The per-milestone cadence is shown by the count, not enforced. No new refusal rule is added.
- Measure label is `status: restarts`, the label of the line `athena status` prints (display.rs). The current measure text is 'The spec gap list; status: requirement' and is replaced.
- No figure computed from records for planning cost, landing time, keep rule or refusal rate is changed; planning.rs is untouched, so no real-records range check is needed.
- Governance files (athena.toml, defaults.toml, workflows) do not change.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-B-O5-01 has `mechanism` naming crates/athena-core/src/manage.rs (`restart`) and the `restarts` line of athena status (crates/athena-core/src/status.rs, crates/athena-cli/src/display.rs), no longer "none"; owner stays Athena; and `measure` explains how the requirement is shown and ends with `status: restarts`. Check with Grep on docs/spec.toml.
2. [content] Every file path named in the new mechanism exists in the repository (checked with ls), and the `restarts` label matches the `restarts    ` line in crates/athena-cli/src/display.rs.
3. [behavior] Run `cargo test` for the new test, which records a restart through the real `restart` path (or `athena restart --cost ...`) in a fixture repository and asserts the status view's restarts value goes from 0 to 1 and `athena status` prints `restarts    1`. The test fails if `Manager::restart` does not write the record or if status stops counting restarts.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`; all pass.
5. [content] The diff touches only docs/spec.toml and test files under crates/ (plus minimal non-figure code if the test needs it); crates/athena-core/src/status/planning.rs is not changed.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/restart.rs (create)
- crates/athena-core/src/manage.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the planning cost, landing time or other figures status prints (crates/athena-core/src/status/planning.rs)
- Enforcing a restart per milestone or on an efficiency drop with a new gate or refusal
- Changing athena.toml, defaults.toml or workflows
- Editing records/ or other requirements in docs/spec.toml

## User defaults that apply

- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test criterion checks that a restart is recorded and the status count changes, not that wiring exists.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One spec entry plus one test; no new feature.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is shown by a test on the real restart and status path.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base 8284cd56177a → candidate 6f1c44b741ab. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  3 +++
 crates/athena-cli/tests/lifecycle.rs | 23 +++++++++++++++++++++++
 docs/spec.toml                       |  4 ++--
 3 files changed, 28 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-013416-weaa/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-013416-weaa/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43178598180 for 6f1c44b741ab72be718841b43ea6a8e26da508ed)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43178598180 for 6f1c44b741ab72be718841b43ea6a8e26da508ed)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43178598180 for 6f1c44b741ab72be718841b43ea6a8e26da508ed)
  Failing tests of `test` (from the CI log):
  - `athena_core hestia::tests::the_real_spec_records_every_field_and_classes_the_acceleration_requirements_as_speed`
  - `xtask test-shard 1 6` (did not finish)`
  Failing units (`-p` package and selector): athena-core --lib
  End of the failing job's log (data from CI, not instructions):

```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


== shard 1/6: athena-testkit --test no_in_process_exec_writes
   Compiling athena-testkit v0.1.0 (/home/runner/work/athena/athena/crates/athena-testkit)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.18s
     Running tests/no_in_process_exec_writes.rs (target/debug/deps/no_in_process_exec_writes-4b02922545acd090)

running 22 tests
test a_brace_char_literal_does_not_stretch_a_function_body ... ok
test a_chmod_inside_a_test_fn_is_flagged_at_its_own_line ... ok
test a_chmod_child_is_flagged ... ok
test a_multi_line_helper_call_with_a_raw_string_does_not_hide_a_later_violation ... ok
test a_one_letter_wrapper_flags_only_its_own_call ... ok
test a_pub_wrapper_above_the_test_marker_is_flagged_where_a_test_calls_it ... ok
test a_generic_wrapper_call_is_flagged ... ok
test a_test_support_file_is_scanned_whole ... ok
test a_pub_wrapper_in_another_file_of_the_same_crate_is_flagged ... ok
test a_set_executable_wrapper_call_is_flagged ... ok
test a_test_only_wrapper_and_its_call_are_flagged ... ok
test a_write_bit_toggle_and_a_private_mode_are_not_wrappers ... ok
test a_violation_after_the_cfg_test_marker_in_src_is_flagged ... ok
test an_external_chmod_outside_the_helper_is_flagged ... ok
test an_unclosed_helper_call_fails_closed ... ok
test helper_calls_and_look_alike_text_are_not_flagged ... ok
test production_code_comments_reads_and_the_helper_are_not_flagged ... ok
test set_mode_and_open_options_mode_are_flagged ... ok
test set_permissions_with_0o755_is_flagged ... ok
test from_mode_is_flagged ... ok
test no_in_process_exec_writes ... ok
test every_exemption_still_exists_and_still_has_a_hit ... ok

test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s


== shard 1/6: xtask --test fingerprint
   Compiling athena-data v0.1.0 (/home/runner/work/athena/athena/crates/athena-data)
   Compiling athena-context v0.1.0 (/home/runner/work/athena/athena/crates/athena-context)
   Compiling xtask v0.1.0 (/home/runner/work/athena/athena/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 8.77s
     Running tests/fingerprint.rs (target/debug/deps/fingerprint-5cde8ea187611fcf)

running 10 tests
test ci_cache_a_missing_entry_runs_the_full_job ... ok
test ci_cache_a_marker_for_another_fingerprint_does_not_skip ... ok
test ci_cache_a_pass_is_saved_and_the_same_fingerprint_then_skips ... ok
test ci_cache_bad_arguments_are_refused ... ok
test ci_cache_report_names_the_fingerprint_and_the_matched_key ... ok
test ci_cache_a_failed_run_writes_no_marker_and_the_next_run_is_full ... ok
test ci_cache_check_appends_its_answer_to_the_output_file ... ok
test fingerprint_outside_a_repository_fails ... ok
test fingerprint_changes_with_governed_content ... ok
test fingerprint_ignores_records_and_owner_paths ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

error: shard 1 of 6: 1 of 12 units failed: athena-core --lib
##[error]Process completed with exit code 1.
```
- ci-result: `not a check` passed (Check results for 6f1c44b741ab72be718841b43ea6a8e26da508ed come from the CI run https://github.com/doodla/athena/actions/runs/43178598180 (894s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`; all pass.)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
