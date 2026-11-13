# Job 20261113-023315-gk4a: verify "Retry OLY-B-A13-01: tokens-per-class status line, drain-safe"

## Request (verbatim)

> try again on 20261030-033234-vm1m

## Reading (accepted by the user)

Retry of job 20261030-033234-vm1m: re-deliver the context-load mechanism (new athena-core module, `context` line in `athena status`, spec entry ending `status: context`) and fix its findings: unrelated-looking drain failure, D43 evidence, tighter mistake figures, and an end-to-end display test. Audience is the Athena owner reading `athena status` and the spec gap list.

Assumptions:
- Prior job 20261030-033234-vm1m is retried from the base; its candidate is re-made, keeping all nine changed files as deliverables (including xtask/shard-timings.txt, which the ci_shards test requires for the new test file).
- Findings addressed: (1) check-test failure on athena-cli drain parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn was not reproduced and the change touches no drain code; the producer runs `cargo test -p athena-cli --test drain` and reports the result, and if it still fails on the final tree it is investigated rather than dismissed as flaky. (2) D43: criteria name each command as its own simple command. (3) shard-timings.txt is now a listed deliverable. (4) mistake figures: count only mistakes whose job is among the counted closed jobs, so numerator and denominator cover the same set; unattached mistakes are not folded in. (5) the display test is made end to end where feasible.
- The introspection's proposal to change the test check's kept output (failing-test extraction, base-tree rerun) is a separate change to the check pipeline; it is out of scope here. The typed request, 'try again', takes precedence.
- Code survey: crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS at ~305, mistake_report at ~595, StatusView), crates/athena-core/src/hestia.rs (spec_gaps ~1291, label acceptance ~1230), crates/athena-cli/src/display.rs (status printing); the new module needs lib.rs registration. It reads existing records only and adds no new record type or data file, so check_record_file and AnyRecord are not touched.
- Existing code: reuse mistake_report in crates/athena-core/src/status.rs and the existing StatusView/STATUS_MEASURE_LABELS and spec_gaps in crates/athena-core/src/hestia.rs; extend them rather than building parallel paths.
- Current docs/spec.toml entry OLY-B-A13-01 has mechanism `none` and measure `The spec gap list; status: requirement` (read at line 2866).
- Minimum landed jobs per class is 3, as before; thin classes yield no figure.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-A13-01 has a mechanism naming crates/athena-core/src/context_load.rs and its test, owner `Athena`, and a measure ending with `status: context`; `context` is in STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs.
2. [content] crates/athena-core/src/context_load.rs is registered in crates/athena-core/src/lib.rs; it groups tokens loaded per landed job by class of work with landed share and mistakes for the same class, counting only mistakes of the counted jobs. StatusView has a matching field and crates/athena-cli/src/display.rs prints a line starting with `context`.
3. [behavior] Run `cargo test -p athena-core --test context_compression`: tests with known token counts across two classes assert exact per-class and outcome figures; a thin class yields no figure; setting the mechanism to `none` makes spec_gaps report OLY-B-A13-01 while the real entry is not a gap. These fail without the module.
4. [behavior] Run `cargo test -p athena-cli`: display tests assert the `context` line appears in status output when data exists and is absent when it does not, and `--test drain` passes including parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test --locked` each as its own command, or cite Athena-recorded passes on the final tree; all pass, including the ci_shards timings test with the line for context_compression in xtask/shard-timings.txt.
6. [content] CHANGELOG.md records the job and a new appended decision with a Provenance bullet in docs/decisions.md records tokens-per-landed-job as the compression measure; no existing decisions.md or records/ entry is edited.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/context_load.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/tests/context_compression.rs (create)
- xtask/shard-timings.txt (modify)

## Out of scope

- Changing prompts or prompt token budgets
- Actually reducing the context any role loads
- Editing or annotating existing docs/decisions.md or records/ entries
- Changing athena.toml, defaults.toml or workflows
- Changing the test check's kept-output extraction or base-tree rerun (separate change)
- Changes to drain code

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the printed status line and per-class figures, not file existence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and test criteria included.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name cargo test commands.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A content test in the new core test file parses the real spec with hestia::parse_spec(include_str!("../../../docs/spec.toml")) (same depth as status.rs:1634) and finds OLY-B-A13-01. It asserts the mechanism is not `none` and contains `context_load.rs` and `context_compression`, the owner is `Athena`, the measure ends with `status: context`, hestia::measure_problem(&req.measure) is None, and STATUS_MEASURE_LABELS contains ("context", "context_load"). A second command, `cargo test -p athena-core every_measure_label_names_a_status_view_field_and_a_printed_line` (status.rs:1689), checks that StatusView has `pub context_load:` and that a `"context ` literal is printed in display.rs. Run: `cargo test -p athena-core --test context_compression`. Observes: Both commands pass: the spec entry names the module and its test, and the label maps to a StatusView field and a printed line.
  Fails without the behavior: Without the spec edit, `oly_b_a13_01_is_met_by_context_load_and_shown_by_the_context_line` fails: the mechanism is `none` and the measure has no `status: context`. Without the label entry, `measure_problem` returns Some for the label and the STATUS_MEASURE_LABELS assert fails.
  Test seam: Production entry points hestia::parse_spec, hestia::measure_problem and the STATUS_MEASURE_LABELS const. The fixture is the real docs/spec.toml read at compile time; nothing is faked. The seam exists today (hestia.rs:1172, 1219; status.rs:305).
- Criterion 2: Core test `class_figures_use_only_the_counted_jobs` seeds records and asserts ContextLoad::compute groups landed jobs by class and gives each class its landed share and its mistakes from the counted jobs only. It asserts shown mistakes + left out == mistake_report(&records).total. A source-text test asserts lib.rs contains `pub mod context_load;`. The display half is the end-to-end test run under criterion 4; a second command here is `cargo test -p athena-cli status_prints_the_context_line_from_seeded_records_and_not_without_them`. Run: `cargo test -p athena-core --test context_compression`. Observes: Exact per-class counts, shares and mistake counts (figures in criterion 3). An unattached mistake, a mistake on a job of another class and a mistake on a job not yet closed are absent from the class figures and present in the left-out count. The printed status text starts a line with `context`.
  Fails without the behavior: Without the module the test file does not compile. Without the StatusView field or the status.rs build site, the every_measure_label test and the end-to-end test of criterion 4 fail.
  Test seam: Production entry points Records::load_dir (store.rs:364, which walks <dir>/records) and ContextLoad::compute (added by step 2). Fixture: a tempdir of record files written with AnyRecord::canonical_path and to_json (records.rs:1266, 1271): Job (JSON stub as at status.rs:2178), Invocation and Outcome (builders copied from tests/status_role_need.rs:19-94) and Intervention. They go through the real loader. The test file is added by step 5; the loader exists today.
- Criterion 3: Tests with known token counts. Class `code`: landed jobs loading 1000, 2000 and 3000 tokens (input + cache_read + cache_write, with large output values that must not count), expected mean 2000.0. One failed `code` job, so landed 3 of 4 closed, share 0.75. One mistake on a landed `code` job and one on the failed one, so 2 mistakes. Class `docs`: landed jobs of 500, 1500 and 1000, mean 1000.0, share 1.0, 0 mistakes. Class `study`: 2 landed jobs, so no figure; thin_classes == 1 and the class is absent from `classes`. Records with only thin classes give compute == None. An unattached mistake and a mistake on a still-dispatched job give mistakes_left_out == 2. A landed job with no token value is counted in the without-tokens note and does not lower the mean. A gap test takes the real OLY-B-A13-01 entry, asserts spec_gaps has no entry for it, sets mechanism = "none" on a clone and asserts spec_gaps(&[clone]) returns exactly one gap with no_mechanism (same pattern as status.rs:1653). Run: `cargo test -p athena-core --test context_compression`. Observes: The tests pass with the exact numbers above; failure messages print the computed lines() text.
  Fails without the behavior: The tests do not compile without athena_core::context_load. If tokens were summed wrongly (output included, or all classes pooled) the exact-figure asserts fail. If the 3-job floor were missing the thin-class assert fails. If the spec entry or label were missing the gap test fails.
  Test seam: Production entries ContextLoad::compute, ContextLoad::lines, hestia::spec_gaps and mistake_report. Fixture: the record files of criterion 2 and the real spec text. No double is used and no logic under test is in a fixture. compute is added by step 2, spec_gaps exists today (hestia.rs:1291), and the test file is added by step 5.
- Criterion 4: A display test in display.rs's test module, `status_prints_the_context_line_from_seeded_records_and_not_without_them`. It makes a temp git repo with the demo bootstrap commit (athena.toml via include_str!("../../../athena.toml") and defaults.toml, as Fixture::with_config does, tests/common/mod.rs:131). With no jobs committed it asserts display::status(&athena.status().unwrap()) has no line starting `context`. It then commits seeded records (3 landed `code` jobs of 1000, 2000 and 3000 tokens, one failed `code` job with a mistake, 3 landed `docs` jobs) and asserts a line starting `context ` and a `code:` line containing `2000 tokens per landed job over 3 landed jobs`, `landed 3 of 4 closed (75%)` and `2 mistakes in those 4 jobs`, plus a `docs:` line with `1000 tokens`. A third state with only two landed jobs per class asserts the line is absent again. A separate command, `cargo test -p athena-cli --test drain`, must pass, including parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn. The producer reports its result; a failure on the final tree is investigated, not dismissed. Run: `cargo test -p athena-cli`. Observes: The printed status text contains the exact `context` lines when data exists and none when it does not. The drain test passes, or its failure is reported with the cause.
  Fails without the behavior: Without the printing in display.rs the test fails: no `context` line appears for the seeded data. Without the absence guard (None when thin) the no-data and thin-data states fail because a `context` line appears.
  Test seam: Production entries Athena::status (builds the view at status.rs:1285) and athena_cli::display::status (display.rs:1258). Fixture: a tempdir git repo plus athena_core::test_support::make_athena(&git, state_root, vec![], script), where the script panics if called because status runs no agent; it is injected by constructing the Athena directly. Seeded record files are committed under records/ and read by the real Records::load path (status.rs:1180). make_athena exists (test_support.rs:893) and is enabled for athena-cli by Cargo.toml:31. The seeding helpers are added by step 6.
- Criterion 5: Each runs as its own simple command: `cargo fmt --check`; then `cargo clippy --all-targets`; then `cargo test --locked`. A fourth, `cargo test -p xtask --test ci_shards`, shows the timings test alone. Citing Athena-recorded passes on the final tree is also accepted. Athena's own checks are `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`. Run: `cargo fmt --check`. Observes: All pass, and xtask/shard-timings.txt contains `athena-core --test context_compression 5.00` (an estimate, like the rest of the file).
  Fails without the behavior: A formatting slip in the new files fails `cargo fmt --check`. A lint fails `cargo clippy --all-targets`. A missing timings line makes checked_in_timings_cover_every_workspace_test_unit (ci_shards.rs:411) fail because the new test target has no entry.
  Test seam: No double: the repository's own check commands over the final tree. The ci_shards test reads the real xtask/shard-timings.txt and the workspace target set (ci_shards.rs:414-420); it exists today.
- Criterion 6: The diff must show only added lines: a CHANGELOG bullet under `## [Unreleased]` / `### Added` naming OLY-B-A13-01, context_load.rs, the `context` line, the 3-job floor and the shard-timings line; and a new `## D175 · ...` entry at the end of decisions.md with Case, Decision (tokens per landed job as the compression measure, the input + cache_read + cache_write definition, mistakes scoped to the counted closed jobs), Alternatives, Tradeoff, Reversal cost and a Provenance bullet. A second command, `cargo test -p athena-cli --test mirrors`, shows the decision-log structure check still passes. `git diff --stat` shows no records/ path. Run: `git diff -- docs/decisions.md CHANGELOG.md`. Observes: Only additions in both files, a Provenance bullet in D175, and no edit to D01 to D174 or to records/.
  Fails without the behavior: Without the new entry the diff shows no D175 text and no Provenance bullet. An in-place edit of an old entry would show as removed lines above line 5061 and be refused by the append-only rule.
  Test seam: No double. git diff over the working tree and the existing mirror and decision-structure tests (D162 registry); both exist today.

Planned files:

- crates/athena-core/src/context_load.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/spec.toml (modify)
- crates/athena-core/tests/context_compression.rs (create)
- xtask/shard-timings.txt (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base bbd98c6e4f6c → candidate d75029d9f8c6. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                    |   1 +
 crates/athena-cli/src/display.rs                | 255 ++++++++++++++++
 crates/athena-core/src/context_load.rs          | 172 +++++++++++
 crates/athena-core/src/lib.rs                   |   3 +
 crates/athena-core/src/status.rs                |   6 +
 crates/athena-core/tests/context_compression.rs | 370 ++++++++++++++++++++++++
 docs/decisions.md                               |  31 ++
 docs/spec.toml                                  |   4 +-
 xtask/shard-timings.txt                         |   1 +
 9 files changed, 841 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261113-023315-gk4a/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-023315-gk4a/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43356801900 for d75029d9f8c6dfd0425d7fde0ef5ca6b519f2c4f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43356801900 for d75029d9f8c6dfd0425d7fde0ef5ca6b519f2c4f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43356801900 for d75029d9f8c6dfd0425d7fde0ef5ca6b519f2c4f)
- ci-result: `not a check` passed (Check results for d75029d9f8c6dfd0425d7fde0ef5ca6b519f2c4f come from the CI run https://github.com/doodla/athena/actions/runs/43356801900 (756s): fmt success, clippy success, test success.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 2, 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test context_compression`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-core/src/context_load.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/status.rs, docs/decisions.md, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling getrandom v0.3.4
   Compiling cfg-if v1.0.5
   Compiling crypto-common v0.1.7
   Compiling block-buffer v0.10.4
   Compiling thiserror v2.0.21
   Compiling winnow v1.0.4
   Compiling serde_json v1.0.151
   Compiling serde v1.0.229
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling digest v0.10.7
   Compiling winnow v0.7.15
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling cpufeatures v0.2.17
   Compiling itoa v1.0.18
   Compiling sha2 v0.10.9
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling bitflags v2.13.2
   Compiling linux-raw-sys v0.12.1
   Compiling jiff v0.2.38
   Compiling ref-cast v1.0.27
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling equivalent v1.0.2
   Compiling dyn-clone v1.0.20
   Compiling hashbrown v0.17.1
   Compiling anyhow v1.0.104
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
error[E0432]: unresolved import `athena_core::context_load`
 --> crates/athena-core/tests/context_compression.rs:8:18
  |
8 | use athena_core::context_load::{ContextLoad, MIN_CLASS_JOBS};
  |                  ^^^^^^^^^^^^ could not find `context_load` in `athena_core`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `athena-core` (test "context_compression") due to 1 previous error
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-core/src/context_load.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/status.rs, docs/decisions.md, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
running 4 tests
test the_old_way_job_landed_with_its_trailers_and_was_read_dispatched_and_verified ... ok
test the_fixture_has_a_bare_remote_a_bootstrap_commit_and_names_the_cutover_job ... ok
test c0_resolves_from_the_cutover_landing_and_the_import_and_exempt_record_are_on_the_records_branch ... ok
test the_history_and_attribution_checks_pass_on_the_fixtures_main ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.38s

     Running tests/design_surface.rs (<job>/target/verify/debug/deps/design_surface-781a316b062528bf)

running 2 tests
test the_inventory_at_head_lists_every_command_in_help_and_every_section_athena_toml_uses ... ok
test the_inventory_at_head_is_identical_when_computed_twice ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s

     Running tests/drain.rs (<job>/target/verify/debug/deps/drain-b948bdfa3c2650c2)

running 202 tests
test a_budget_too_small_for_the_next_call_stops_with_the_rest_still_queued ... ok
test a_check_timeout_or_a_work_failure_failing_an_item_twice_does_not_stop_the_batch ... ok
test a_checker_that_skips_model_json_is_rejected_even_with_no_problems ... ok
test a_declined_job_stays_out_of_the_queue ... ok
test a_closed_job_stays_out_of_the_queue ... ok
test a_failed_exec_leaves_no_carry_behind ... ok
test a_failed_job_that_is_not_a_usage_limit_does_not_stop_the_drain_as_one ... ok
test a_failed_rebuild_stops_the_drain_as_rebuild_failed_with_the_build_output ... ok
test a_file_the_checker_did_not_read_fails_the_check ... ok
test a_fresh_drain_and_a_stale_carry_start_at_zero ... ok
test a_drain_runs_the_backfill_and_the_derivation_at_its_start_and_a_second_drain_writes_nothing ... ok
test a_job_a_work_decision_approved_is_listed_once ... ok
test a_job_that_does_not_land_twice_is_passed_over_with_its_landing_text_and_the_next_item_runs ... ok
test a_change_to_max_concurrency_alone_does_not_invalidate_a_reading ... ok
test a_landing_failure_is_a_failure_and_no_rating_is_taken_as_success ... ok
test a_landing_outside_the_explainers_sources_cannot_meet_the_target_even_at_the_rating ... ok
test a_landing_that_changes_athenas_code_rebuilds_once_and_the_next_item_runs_under_the_new_build ... ok
test a_landing_that_changes_only_docs_or_records_rebuilds_nothing_and_the_run_goes_on ... ok
test a_landing_whose_config_the_running_build_cannot_read_ends_the_run_with_a_reinstall_note_not_a_failure ... ok
test a_landing_whose_config_the_running_build_cannot_read_is_recorded_landed_and_ends_with_reinstall ... ok
…
     Running tests/work_decisions.rs (<job>/target/verify/debug/deps/work_decisions-e471e5320a629b3a)

running 5 tests
test status_lists_the_open_proposal_with_its_id_and_nothing_else ... ok
test an_unknown_id_a_blind_spot_and_an_empty_reason_are_refused ... ok
test the_work_commands_exist ... ok
test rejecting_records_the_reason_and_closes_the_proposal ... ok
test approving_queues_a_job_whose_request_carries_the_cited_content ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.72s

   Doc-tests athena_cli

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 2 targets failed:
    `-p athena-cli --lib`
    `-p athena-cli --test mirrors`
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --check`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff -- docs/decisions.md CHANGELOG.md`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
