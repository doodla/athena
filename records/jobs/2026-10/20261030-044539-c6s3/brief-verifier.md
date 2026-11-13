# Job 20261030-044539-c6s3: verify "Give OLY-B-S6-01 a critique ledger mechanism and test"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-S6-01 a mechanism
> 
> Requirement OLY-B-S6-01 in docs/spec.toml is active and has no mechanism.
> Requirement: A living, machine-readable ledger holds critiques, improvements and open questions, and Athena is critiqued to the same standard as the others.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-S6-01
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-S6-01 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a committed machine-readable ledger docs/critique-ledger.toml of critiques, improvements and open questions (including entries critiquing Athena itself), plus a validating parser in athena-core and a test. Record the module as the mechanism for OLY-B-S6-01 in docs/spec.toml with an owner and a measure ending `status: not shown` naming the test. Audience: the verifier and spec-gap readers.

Assumptions:
- Code survey: read crates/athena-core/src/iris.rs (committed TOML ledger plus strict parse_ledger, the pattern to follow), crates/athena-core/tests/learnings_recorded.rs (spec-requirement mechanism test template using parse_spec, spec_gaps, measure_problem from athena_core::hestia, and a measure ending `status: not shown` naming the test command), and the crates/athena-core/src/lib.rs module list. The new module is entered by tests only (no CLI path), so there is no resolution/probing order; the property is measured directly by parsing the real docs/critique-ledger.toml and the real docs/spec.toml entry, not through a stand-in.
- Existing code: crates/athena-core/src/iris.rs (ledger parse pattern), crates/athena-core/src/lib.rs (module registration), docs/spec.toml (requirement entry at OLY-B-S6-01), crates/athena-core/tests/learnings_recorded.rs (test shape). The new criteria extend these patterns rather than replace them.
- The ledger is a data file docs/critique-ledger.toml, not under crates/, so records validation (check_record_file, AnyRecord) does not cover it; the new module's own strict parser rejects unknown keys, and the real file is parsed by the test.
- Ledger entry shape (assumption): [[entry]] with id, kind (critique | improvement | open_question), subject (e.g. athena, or another named component), status (open | resolved), text, and source. The ledger is 'living' because entries are appended and statuses updated, never required to be empty or closed.
- Term ledger: a committed TOML file of entries that code parses and validates; the 'same standard' means Athena-subject entries must satisfy the same validation as entries about others, and at least one entry has subject athena.
- The measure ends `status: not shown` because no `athena status` line reports the ledger; adding a status line is out of scope. No athena.toml, defaults.toml or workflow change is needed. Consumers: none found (no config deliverable).

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-S6-01 has mechanism naming the new module path under crates/athena-core/src/ and docs/critique-ledger.toml (no longer "none"), owner "Athena", and a measure ending `status: not shown` that names the test command `cargo test -p athena-core --test critique_ledger`.
2. [content] docs/critique-ledger.toml exists with [[entry]] tables covering all three kinds (critique, improvement, open_question), each with id, kind, subject, status, text and source, and at least one entry with subject athena (visible in the diff).
3. [content] The new module is registered in crates/athena-core/src/lib.rs with a doc line, and the module parser rejects unknown keys, unknown kinds or statuses, blank text, and duplicate ids.
4. [behavior] Run `cargo test -p athena-core --test critique_ledger`: it parses the real docs/critique-ledger.toml and the real docs/spec.toml entry and passes; the tests include negative cases (duplicate id, bad kind, no athena-subject entry, missing kind coverage) that fail validation, so the test fails if the ledger or its validator is removed.
5. [behavior] Run `cargo test` for the workspace, `cargo clippy` for the workspace and `cargo fmt --check`; each passes (or the same-tree recorded passing checks per D115).
6. [content] CHANGELOG.md gains an entry for this job; no existing entry of docs/decisions.md or anything under records/ is edited.

## Expected deliverables

- docs/spec.toml (modify)
- docs/critique-ledger.toml (create)
- crates/athena-core/src/critique_ledger.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/tests/critique_ledger.rs (create)
- CHANGELOG.md (modify)
- docs/design/requirements-map.md (modify)

## Out of scope

- Adding an `athena status` line for the ledger
- Changing athena.toml, defaults.toml or CI workflows
- Editing docs/decisions.md existing entries or records/
- Automatically generating ledger entries from reviews or runs

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion runs the test that parses the real ledger and spec entry, with negative cases.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has tests; fmt, clippy, test criterion included.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Validation is enforced by code and test, not prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base f5b3b7d7d3ce → candidate 527c99f2a0b8. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |   1 +
 crates/athena-core/src/critique_ledger.rs   | 140 +++++++++++++++++++++++++++
 crates/athena-core/src/lib.rs               |   4 +
 crates/athena-core/tests/critique_ledger.rs | 144 ++++++++++++++++++++++++++++
 docs/critique-ledger.toml                   |  35 +++++++
 docs/design/requirements-map.md             |   2 +-
 docs/spec.toml                              |   4 +-
 xtask/shard-timings.txt                     |   1 +
 8 files changed, 328 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-044539-c6s3/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-044539-c6s3/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43384292005 for 527c99f2a0b831e31d60416ee7eb073d1fcd2965)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43384292005 for 527c99f2a0b831e31d60416ee7eb073d1fcd2965)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43384292005 for 527c99f2a0b831e31d60416ee7eb073d1fcd2965)
  Failing tests of `test` (from the CI log):
  - `athena_cli strategy::tests::strategy_view_table_matches_the_map`
  - `xtask test-shard 1 6` (did not finish)`
  Failing units (`-p` package and selector): athena-cli --lib
  End of the failing job's log (data from CI, not instructions):

```text
test admission::tests::a_user_or_host_question_and_an_empty_list_are_not_refused ... ok
test admission::tests::a_user_or_host_question_with_a_code_reading_phrase_is_refused ... ok
test admission::tests::a_workflow_change_must_be_listed_and_named ... ok
test admission::tests::an_unaddressed_open_question_is_a_major_finding_naming_it ... ok
test admission::tests::a_new_section_and_its_schema_change_together_are_admitted_without_a_build ... ok
test admission::tests::a_study_reading_or_a_reading_without_the_default_needs_no_changelog ... ok
test admission::tests::a_job_usd_below_the_stage_caps_is_a_configuration_violation ... ok
test admission::tests::an_unprefixed_question_about_existing_tests_keeps_only_the_no_addressee_finding ... ok
test admission::tests::an_unknown_field_in_athena_toml_is_a_configuration_violation ... ok
test admission::tests::governance_files_are_recognized ... ok
test admission::tests::lints_tables_are_compared_alone ... ok
test admission::tests::an_unknown_field_in_defaults_toml_is_a_configuration_violation ... ok
test admission::tests::ci_reuse_replay_the_seen_commands_filter_turns_up_in_a_search_and_must_be_named ... ok
test admission::tests::an_unknown_section_without_a_schema_change_is_still_refused ... ok
test audience::tests::a_report_becomes_a_rating_only_when_it_is_in_range_and_cites_evidence ... ok
test audience::tests::only_a_rating_of_two_or_lower_is_low ... ok
test audience::tests::the_brief_section_shows_the_rating_and_findings_or_says_there_is_none ... ok
test audience::tests::the_explainer_is_the_default_target ... ok
test audience::tests::the_real_page_source_refuses_a_non_http_url ... ok
test audience::tests::the_work_item_cites_the_rating_record_and_validates ... ok
test auditor::tests::a_report_becomes_an_audit_only_when_every_finding_cites_evidence ... ok
test auditor::tests::only_plain_relative_paths_are_written_into_the_workspace ... ok
test auditor::tests::the_brief_carries_the_brief_and_names_the_job ... ok
test admission::tests::the_changelog_is_demanded_even_when_the_reading_leaves_it_out_of_scope ... ok
test chain::tests::no_landed_job_means_no_previous_job ... ok
test admission::tests::parsing_configuration_and_untouched_configuration_add_no_violation ... ok
test admission::tests::plan_admission_and_the_gate_agree_on_when_the_schema_judges_a_config_edit ... ok
test checks::tests::a_check_without_a_skip_list_always_runs ... ok
test checks::tests::a_cleanup_looking_line_before_the_failing_section_is_kept ... ok
test checks::tests::a_crashed_binary_is_marked_as_not_finished ... ok
test checks::tests::a_detail_is_not_applied_to_a_passed_or_local_check_or_when_redaction_fails ... ok
test checks::tests::a_doc_test_failure_is_named_under_its_crate ... ok
test checks::tests::a_docs_only_change_skips_and_says_so ... ok
test chain::tests::the_latest_landed_job_is_named_and_hand_changes_are_skipped ... ok
test checks::tests::a_failed_local_run_keeps_the_failing_section_not_trailing_cleanup ... ok
test checks::tests::a_failed_runs_kept_lines_stop_at_the_byte_ceiling ... ok
test checks::tests::a_list_that_disagrees_with_the_failed_count_is_marked_as_not_finished ... ok
test checks::tests::a_failed_run_keeps_its_last_200_lines_and_a_passing_run_its_byte_tail ... ok
test checks::tests::a_record_for_the_same_sha_stands_in_for_the_checks_and_another_sha_does_not ... ok
…


== shard 1/6: xtask --test setup_repeatable
   Compiling xtask v0.1.0 (/home/runner/work/athena/athena/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.38s
     Running tests/setup_repeatable.rs (target/debug/deps/setup_repeatable-d67a8262a122879f)

running 7 tests
test setup_sh_ends_in_one_locked_cargo_run ... ok
test setup_sh_pins_rustup_and_checks_its_sha256 ... ok
test the_rustup_install_runs_only_when_cargo_is_missing ... ok
test the_toolchain_channel_is_an_exact_version ... ok
test the_other_pins_are_exact ... ok
test blanking_the_mechanism_makes_the_entry_a_gap ... ok
test the_spec_entry_names_the_setup_path_owner_and_test ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

error: shard 1 of 6: 1 of 14 units failed: athena-cli --lib
##[error]Process completed with exit code 1.
```
- ci-result: `not a check` passed (Check results for 527c99f2a0b831e31d60416ee7eb073d1fcd2965 come from the CI run https://github.com/doodla/athena/actions/runs/43384292005 (737s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test` for the workspace, `cargo clippy` for the workspace and `cargo fmt --check`; each passes (or the same-tree recorded passing checks per D115).)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
