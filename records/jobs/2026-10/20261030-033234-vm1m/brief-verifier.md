# Job 20261030-033234-vm1m: verify "Measure tokens loaded per class of work in athena status"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-A13-01 a mechanism
> 
> Requirement OLY-B-A13-01 in docs/spec.toml is active and has no mechanism.
> Requirement: Context compression is an explicit goal: Athena measures tokens loaded per class of work and whether outcomes held.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-A13-01
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-A13-01 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Give OLY-B-A13-01 a real mechanism: a new athena-core module computes tokens loaded per landed job by class of work, beside whether outcomes held (landed share and mistakes per class). `athena status` prints it as a `context` line, and docs/spec.toml names the module and ends the measure with `status: context`. Goal is context compression; the audience is the Athena owner reading `athena status` and the spec gap list.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, StatusView, mistake_report, landed_jobs), crates/athena-core/src/class_trend.rs (the per-class pattern to mirror: analyze/compute/lines, prints nothing on a thin history), crates/athena-core/src/invocation.rs (tokens_lower_bound, prompt_tokens), crates/athena-core/tests/learnings_recorded.rs (the spec-mechanism test pattern using parse_spec, spec_gaps, measure_problem), and crates/athena-cli/src/display.rs (where the trend section is printed). The new code reads the existing token facts on recorded invocations; it adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not touched.
- Existing code: extend crates/athena-core/src/status.rs (STATUS_MEASURE_LABELS, StatusView), crates/athena-core/src/class_trend.rs (pattern), crates/athena-cli/src/display.rs (printing), and docs/spec.toml; the new module is the only file created under crates/ besides a test.
- The status label is `context`, so the measure ends with `status: context`. Adding it to STATUS_MEASURE_LABELS (so the Hestia spec pass accepts it) is part of the change.
- Outcomes held means landed share and mistakes for the same class of work, shown beside tokens per class. It is not a new quality judgement.
- A class with too few landed jobs prints no figure, as the trend line does, so a thin history is not shown as success.
- Not a refusal rule: nothing is newly refused, so no refusal-rate criterion applies.
- Consumers: none found for new config; athena.toml and defaults.toml are not changed.
- Environment: none. No CI, push, PR or records-branch reach is needed.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-A13-01 has a mechanism naming the new module path under crates/athena-core/src/ and its test, owner `Athena`, and a measure that ends with `status: context`. The `context` label is in STATUS_MEASURE_LABELS in crates/athena-core/src/status.rs.
2. [content] A new module in crates/athena-core/src/ is registered in lib.rs. It groups tokens loaded per landed job by class of work and reports, for the same class, the landed share and mistake count. `StatusView` has a matching field, and crates/athena-cli/src/display.rs prints a line starting with `context`.
3. [behavior] Run `cargo test -p athena-core`. A unit test builds records with known token counts across two classes and asserts the exact per-class figures and the outcome figures. A second test asserts that a class below the minimum job count yields no figure. A third test asserts that dropping the mechanism (setting it to `none`) makes `spec_gaps` report OLY-B-A13-01, and that the real spec entry is not a gap. These tests fail without the module and pass with it.
4. [behavior] Run `cargo test -p athena-cli`. A display test asserts the `context` line appears in `athena status` output when the data exists and is absent, rather than showing success, when it does not.
5. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test` as separate commands, or point to the Athena-recorded passes on the final tree. All pass.
6. [content] CHANGELOG.md records the job, and a new appended decision with a Provenance bullet in docs/decisions.md records the choice of tokens-per-landed-job as the compression measure. No existing decisions.md or records/ entry is edited.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/context_load.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/tests/context_compression.rs (create)

## Out of scope

- Changing prompts or prompt token budgets
- Actually reducing the context any role loads
- Editing or annotating existing docs/decisions.md or records/ entries
- Changing athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has unit tests, and fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check exact per-class figures and the status line, not that files exist.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: A class with too few jobs shows nothing rather than success.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The requirement is enforced by a test on the spec entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content test on the real files, following spec_entry_has_a_real_mechanism (learnings_recorded.rs:289-309). It reads docs/spec.toml with hestia::parse_spec, finds OLY-B-A13-01 and asserts: the mechanism contains `crates/athena-core/src/context_load.rs` and `crates/athena-core/tests/context_compression.rs`; the owner is `Athena`; the measure ends `status: context`; measure_problem is None; STATUS_MEASURE_LABELS contains ("context", "context_load"). The existing status.rs test every_measure_label_names_a_status_view_field_and_a_printed_line (status.rs:1688-1708) then requires `pub context_load:` in status.rs and `"context ` in status.rs, display.rs or hestia.rs. Run: `cargo test -p athena-core --test context_compression spec_entry_names_the_module_its_test_and_a_shown_label && cargo test -p athena-core --lib every_measure_label_names_a_status_view_field_and_a_printed_line && cargo test -p xtask --test spec`. Observes: All pass: the entry's mechanism is no longer none and names the module and test, the owner is Athena, the measure ends `status: context`, the label, field and printed line exist, and the xtask spec rules accept the entry.
  Fails without the behavior: With the mechanism left `none` or the measure missing `status: context`, the content test fails. Without the label in STATUS_MEASURE_LABELS, the content test and measure_problem fail. Without the StatusView field or the display literal, the label test fails.
  Test seam: Production entry points hestia::parse_spec, hestia::measure_problem and status::STATUS_MEASURE_LABELS (exist, hestia.rs:1172 and 1219; status.rs:305). Fixture: the real docs/spec.toml read from the workspace root, no fake. The test is added by plan step 7.
- Criterion 2: Compilation plus tests. The integration test imports athena_core::context_load::ContextLoad, so it does not compile unless lib.rs has `pub mod context_load;`. Unit tests in context_load.rs check analyze grouping (tokens per landed job, landed share, mistakes, mistakes per closed job) and the lines() text. The status.rs unit test status_carries_the_context_load_for_a_class_with_enough_landed_jobs shows the StatusView field is filled by Athena::status(). The display.rs test shows the printed line starts with `context`. Run: `cargo test -p athena-core --lib context_load && cargo test -p athena-core --lib status_carries_the_context_load && cargo test -p athena-cli --lib context_line`. Observes: Tests compile and pass. Per-class figures include tokens per landed job, landed share and mistake figures. The StatusView field holds them and display prints a line that starts with `context`.
  Fails without the behavior: Without `pub mod context_load;` the tests do not compile. Without the compute call in the StatusView literal the status.rs test finds None. Without the printer call the display test finds no `context` line.
  Test seam: Production entry points ContextLoad::compute and analyze (added by step 2), Athena::status() (exists, status.rs:1174; context_load is built in the literal at status.rs:1285, step 4) and display::context_section (added by step 5). Fixture for the status.rs test: crate::code_health::tests::set_up() and records::commit_in_tree (exist, used at status.rs:4283 and 4282), injected by committing AnyRecord values into the scratch repository, so Athena::status() loads them with the real Records::load_current. No logic sits in the fixture.
- Criterion 3: Three tests in crates/athena-core/tests/context_compression.rs using public API. (a) tokens_per_landed_job_and_outcomes_are_exact_for_two_classes: write records into a temp dir's records/ (AnyRecord::to_json at canonical_path: Job or Reading with class_of_work, invocations with output 5000 each, an Outcome with totals, interventions) and call Records::load_dir, then assert records.invalid is empty and ContextLoad::compute(&records). docs: d1 landed (invocations 350 and 50, so 400), d2 landed 900, d3 landed 1700, d4 failed 600 (loaded = input + cache_read + cache_write; output 5000 each). Expected for docs: closed 4, landed 3, tokens 3600, 1200.0 per landed job, landed share 75%, mistakes 2 (two Mistake interventions counted, one CorrectedOutput not), 0.50 per closed job. code: c1..c3 landed with 2000, 3000, 4000: 3000.0 per landed job, 100%, mistakes 0, 0.00. An auditor run under its own generated id with 100000 tokens sits in auditor_runs (asserted len 1) and changes nothing. The test also asserts the lines() text `docs: 1200 tokens per landed job · 3 of 4 closed jobs landed (75%) · 2 mistake(s) recorded, 0.50 per closed job`. (b) a_class_below_the_minimum_yields_no_figure: a `config` class with 2 landed jobs is absent from classes; records holding only that class give None. (c) the_spec_entry_is_not_a_gap_and_none_makes_it_one: as spec_check_refuses_a_none_mechanism (learnings_recorded.rs:311-318), the real entry is absent from hestia::spec_gaps and a copy with mechanism `none` yields a gap for OLY-B-A13-01. Unit tests in context_load.rs pin a lower-bound Quantity counted at its bound. Expected numbers are written from the constants above, not computed by the code. Run: `cargo test -p athena-core --test context_compression && cargo test -p athena-core --lib context_load`. Observes: Exact figures 1200.0 and 3000.0, shares 75% and 100%, mistakes 2 and 0, rates 0.50 and 0.00, no `config` class, the auditor run not counted, no gap for the real entry, and a gap for the `none` copy.
  Fails without the behavior: Without the module the file does not compile. If output tokens were counted, every figure changes and the 1200.0 assertion fails. If cache tokens were dropped the d1 and d2 totals change. If the minimum were not enforced `config` would appear. If a mistake of CorrectedOutput category were counted, mistakes would read 3. With the mechanism left `none` in the real spec, test (c) fails.
  Test seam: Production entry points athena_core::context_load::ContextLoad::compute (added by step 2), store::Records::load_dir (exists, store.rs:364) and hestia::{parse_spec, spec_gaps} (exist, hestia.rs:1172 and 1291). Fixture: record files written with AnyRecord::to_json and canonical_path (exist, records.rs:1266 and 1271) into a tempfile dir (tempfile is a dev-dependency of athena-core, Cargo.toml:24). No fake; the loader applies check_record_file. Job and Invocation field lists are copied from status.rs timed_job (status.rs:2989) and the Invocation fixtures in role_need.rs tests. The Outcome of a request job needs totals (records.rs:1255).
- Criterion 4: Display tests in crates/athena-cli/src/display.rs mod tests, beside the trend test (display.rs:2285-2306). `a_context_line_is_printed_with_its_figures` builds a ContextLoad (pub fields) with docs: 3 landed of 4 closed, 1200.0 per landed job, 2 mistakes and calls context_section(Some(&load)): the text starts with `context     ` and contains `docs: 1200 tokens per landed job · 3 of 4 closed jobs landed (75%)`. `no_context_line_without_data` asserts context_section(None) == "". The status() printer calls `out.push_str(&context_section(v.context_load.as_ref()))`. The no-data case through the full path is the status.rs unit test: a scratch repository with no records gives context_load == None, with 2 landed docs jobs still None, and with 3 landed plus 1 failed gives Some with the exact figures; the JSON of the view has `context_load` only then. Run: `cargo test -p athena-cli --lib context_line && cargo test -p athena-core --lib status_carries_the_context_load`. Observes: The `context` line text appears with exact figures when data exists, and the section is the empty string when it does not. Through Athena::status() the field is None with no records or a thin class and Some with the exact figures at 3 landed jobs.
  Fails without the behavior: Without the printer the display test finds no `context` text. If the section printed a success line for None, the empty-string assertion fails. If the minimum were not enforced, the 2-landed-job phase of the status.rs test would find Some.
  Test seam: Production entry points display::context_section (added by step 5) and Athena::status() (exists, status.rs:1174). Fixture for display: a hand-built athena_core::context_load::ContextLoad (pub fields, as the trend test builds ClassTrend at display.rs:2295). Fixture for status: code_health::tests::set_up and commit_in_tree (exist, status.rs:4281-4303). The tests are added by steps 4 and 5. The main `athena status` caller of the printer is not run end to end by a cli test; the call is one line and the label test requires the `"context ` literal.
- Criterion 5: Run the three checks Athena runs as separate commands on the final tree. cargo test --locked includes xtask/tests/ci_shards.rs checked_in_timings_cover_every_workspace_test_unit (needs the one new timings line), the status.rs label test, the boundary and personal-data scans over new and edited files, and the decisions append-only and mirror checks. clippy runs with -D warnings over all targets, so unused code in the new module fails it. Run: `cargo fmt --all --check`. Observes: Then `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`, each as its own command. All three exit 0, with no failing test in any crate (core, cli, xtask).
  Fails without the behavior: Without the `athena-core --test context_compression 5.00` line, checked_in_timings_cover_every_workspace_test_unit fails (assert_eq at ci_shards.rs:420). Without the status.rs field or the display.rs `"context ` literal, the label test fails. A formatting slip fails fmt. A dead helper or unused import fails clippy.
  Test seam: Production entry points: the three Athena check commands on the working tree. Fixture: the real workspace, no fake. ci_shards reads xtask/shard-timings.txt and the workspace's test units (exist, ci_shards.rs:411-421). Step 9 runs them after steps 2-8.
- Criterion 6: Diff review. CHANGELOG.md gains one bullet under `## [Unreleased]` / `### Added` naming OLY-B-A13-01, the module, the `context` label, the `context_load` field and the tests. docs/decisions.md gains a final `## D175 · ...` section after D174 with Case, Decision, Alternatives, Tradeoff, Reversal cost and a Provenance bullet (Read: the files above; Measured: the new tests on fixtures; Not measured: figures on real records, and whether a smaller context holds outcomes). D175 states which invocation kinds are counted (reader, producer, verifier, planner, critic of closed request jobs) and not counted (steward, audience, auditor), the scopes of the mistake figures, and names the real-records check as a follow-up. Run: `git diff -- CHANGELOG.md docs/decisions.md records`. Observes: Only added lines in the two files, no change to any existing D-entry, and an empty diff for records/. The append-only and mirror checks in cargo test --locked back this up.
  Fails without the behavior: An edit to an existing decisions.md entry fails the append-only check in cargo test --locked; a missing Provenance bullet fails the decisions format check.
  Test seam: Production entry points: git diff, and the xtask policy and decisions checks run by cargo test --locked (exist). Fixture: the working tree; no fake.

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

no replay: a check failed

## The change

Base b7778fcef84f → candidate 3ed9f8ffe236. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                    |   1 +
 crates/athena-cli/src/display.rs                |  47 +++
 crates/athena-core/src/context_load.rs          | 236 +++++++++++++++
 crates/athena-core/src/lib.rs                   |   1 +
 crates/athena-core/src/status.rs                |  56 ++++
 crates/athena-core/tests/context_compression.rs | 383 ++++++++++++++++++++++++
 docs/decisions.md                               |  44 +++
 docs/spec.toml                                  |   4 +-
 xtask/shard-timings.txt                         |   1 +
 9 files changed, 771 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-033234-vm1m/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-033234-vm1m/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43350354938 for 3ed9f8ffe23658eae33cd5a4fc402bde5e821927)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43350354938 for 3ed9f8ffe23658eae33cd5a4fc402bde5e821927)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43350354938 for 3ed9f8ffe23658eae33cd5a4fc402bde5e821927)
  Failing tests of `test` (from the CI log):
  - `drain parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn`
  - `xtask test-shard 3 6` (did not finish)`
  Failing units (`-p` package and selector): athena-cli --test drain
  End of the failing job's log (data from CI, not instructions):

```text
test records_branch::tests::when_every_attempt_is_rejected_the_writer_stops_with_a_clear_error ... ok
test snapshot::tests::a_history_snapshot_lists_visible_commits_without_hidden_paths_or_a_repository ... ok
test records_branch::tests::sync_from_resets_a_local_branch_that_is_ahead_or_diverged_and_the_next_write_succeeds ... ok
test snapshot::tests::a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree ... ok
test snapshot::tests::a_seeded_snapshot_diffs_against_the_base_and_notices_an_untouched_seed ... ok
test snapshot::tests::a_walk_survives_entries_removed_after_the_directory_was_listed ... ok
test snapshot::tests::a_snapshot_seeded_from_a_kept_directory_copies_its_files_but_not_its_git ... ok
test snapshot::tests::a_snapshot_without_a_seed_is_never_unchanged_since_seed ... ok
test snapshot::tests::plain_snapshots_have_no_history_file ... ok
test snapshot::tests::an_unlocked_snapshot_takes_edits_and_commits ... ok
test snapshot::tests::producer_edits_become_a_change_list ... ok
test snapshot::tests::restore_returns_a_work_tree_to_a_tree ... ok
test snapshot::tests::snapshot_git_cannot_reach_the_users_history_or_hidden_content ... ok
test snapshot::tests::snapshots_hold_only_what_agents_may_see ... ok
test tests::a_child_that_exits_without_reading_is_not_an_error ... ok
test snapshot::tests::verifier_snapshots_carry_base_then_candidate ... ok
test tests::a_guarded_push_is_rejected_and_changes_nothing_when_the_remote_moved ... ok
test tests::a_fetch_and_a_push_url_for_the_same_repository_are_accepted_in_any_spelling ... ok
test tests::a_guarded_push_replaces_the_branch_when_the_remote_is_where_expected ... ok
test tests::a_missing_origin_is_refused_and_writes_nothing ... ok
test tests::a_relaxed_squash_text_accepts_no_job_trailer_and_refuses_two ... ok
test tests::a_push_url_on_another_repository_is_refused ... ok
test tests::a_push_git_itself_declines_stays_a_git_failure_not_a_refusal ... ok
test tests::a_squash_text_splits_the_title_and_keeps_one_job_trailer_last ... ok
test tests::an_origin_with_a_second_push_url_on_another_repository_is_refused ... ok
test tests::an_origin_with_two_fetch_urls_and_no_push_url_is_refused ... ok
test tests::commit_exists_tells_a_present_commit_from_an_absent_one_and_a_path ... ok
test tests::a_remote_branch_is_fetched_without_moving_any_local_ref_and_set_ref_moves_one ... ok
test tests::diff_tree_reports_raw_changes ... ok
test tests::an_origin_with_two_push_urls_is_refused_even_when_both_name_its_own_repository ... ok
test tests::commit_with_records_adds_the_records_on_top_of_the_base ... ok
test tests::git_is_recent_enough ... ok
test tests::ensure_commit_fetches_a_missing_commit_without_moving_any_ref ... ok
test tests::log_first_parent_reads_hash_time_subject_and_trailers_newest_first ... ok
test tests::log_history_reads_hash_date_subject_and_paths_newest_first ... ok
test tests::merge_tree_report_fails_on_a_bad_revision ... ok
test tests::merge_tree_detects_conflicts ... ok
test tests::merge_tree_report_lists_a_modify_delete_conflict_without_marker_lines ... ok
test tests::merge_tree_report_of_a_clean_merge_is_the_merged_tree ... ok
…
== shard 3/6: xtask --test setup_repeatable
   Compiling athena-data v0.1.0 (/home/runner/work/athena/athena/crates/athena-data)
   Compiling athena-context v0.1.0 (/home/runner/work/athena/athena/crates/athena-context)
   Compiling xtask v0.1.0 (/home/runner/work/athena/athena/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.18s
     Running tests/setup_repeatable.rs (target/debug/deps/setup_repeatable-d67a8262a122879f)

running 7 tests
test setup_sh_ends_in_one_locked_cargo_run ... ok
test setup_sh_pins_rustup_and_checks_its_sha256 ... ok
test the_rustup_install_runs_only_when_cargo_is_missing ... ok
test the_toolchain_channel_is_an_exact_version ... ok
test the_other_pins_are_exact ... ok
test the_spec_entry_names_the_setup_path_owner_and_test ... ok
test blanking_the_mechanism_makes_the_entry_a_gap ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

error: shard 3 of 6: 1 of 13 units failed: athena-cli --test drain
##[error]Process completed with exit code 1.
```
- ci-result: `not a check` passed (Check results for 3ed9f8ffe23658eae33cd5a4fc402bde5e821927 come from the CI run https://github.com/doodla/athena/actions/runs/43350354938 (894s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy` and `cargo test` as separate commands, or point to the Athena-recorded passes on the final tree. All pass.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test -p athena-core --test context_compression spec_entry_names_the_module_its_test_and_a_shown_label && cargo test -p athena-core --lib every_measure_label_names_a_status_view_field_and_a_printed_line && cargo test -p xtask --test spec`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-core --lib context_load && cargo test -p athena-core --lib status_carries_the_context_load && cargo test -p athena-cli --lib context_line`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test -p athena-core --test context_compression && cargo test -p athena-core --lib context_load`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-cli --lib context_line && cargo test -p athena-core --lib status_carries_the_context_load`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff -- CHANGELOG.md docs/decisions.md records`). Result: not run: not a plain cargo test command.

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
