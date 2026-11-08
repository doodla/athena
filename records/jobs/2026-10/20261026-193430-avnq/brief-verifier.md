# Job 20261026-193430-avnq: verify "Split status rendering out of display.rs into a submodule"

## Request (verbatim)

> Structural pass: crates/athena-cli/src/display.rs
> 
> Code health found crates/athena-cli/src/display.rs over a limit: 1504 production lines with 9 landings in the last 7 days, over the limit of 1500 lines with 5 landings in a week.
> 
> Measured: 1504 production lines (test modules excluded); longest function status at 197 lines; 9 landings in the last 7 days; 18 failure or rejection citations.
> limits: 2000 production lines per file; 200 lines per function; 1500 lines with 5 or more landings in 7 days.
> 
> Restructure the file so it is under its limits: split it by responsibility or break up the long function, without changing behavior. Existing tests keep passing.
> 
> Acceptance:
> - The file is under the limit it breached, as measured by code health: 1504 production lines with 9 landings in the last 7 days, over the limit of 1500 lines with 5 landings in a week
> - No behavior change; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-cli/src/display.rs

## Reading (accepted by the user)

Reduce crates/athena-cli/src/display.rs below 1500 production lines (target ~1100) by moving the status-rendering group (plan_suffix, sections, snapshot_status, status, code_health_section, introspection, roughly lines 992–1473) into crates/athena-cli/src/display/status_view.rs, re-exported so callers and tests are unchanged. Audience: the code-health gate and maintainers; no behavior change.

Assumptions:
- Code survey: read crates/athena-cli/src/display.rs item layout (pure string-rendering functions, tests module from line ~1943) and listed crates/athena-cli/src/; display.rs is a plain module file so a sibling directory display/ can hold a submodule without renaming.
- Moving the status group (lines ~992–1473) drops about 480 lines; the split is by responsibility (status view rendering vs. other report rendering).
- Shared helpers (pct, usd, duration, etc.) stay in display.rs and are used by the submodule via super::; visibility adjusted to pub(super) as needed.
- Longest function status (197 lines) is under the 200 limit and is left alone.

## Acceptance criteria

1. [content] crates/athena-cli/src/display.rs has fewer than 1500 production lines (test module excluded), and the moved functions live in a new submodule file under crates/athena-cli/src/display/.
2. [content] Public functions of display (status, snapshot_status, etc.) keep their paths and signatures via `pub use`/`use` re-exports; no caller outside display changes; function bodies are moved verbatim.
3. [behavior] Run `cargo test -p athena-cli`: all existing display tests pass unchanged.
4. [behavior] Run `cargo fmt --check` and `cargo clippy --all-targets`: both clean.
5. [content] No governance files (athena.toml, defaults.toml, Cargo configs) are changed; no test is deleted or weakened.

## Expected deliverables

- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/display/status_view.rs (create)
- crates/athena-cli/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Behavior or output format changes
- Splitting other files or raising limits
- Refactoring the status function itself

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy, fmt criteria
- small: Prefer one focused change; size is a design choice, not a wish list. Here: one focused move
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: tests run to show unchanged behavior
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base 160ea297dff5 → candidate 748a4c1cc4a0. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 |   3 +
 crates/athena-cli/src/display.rs             | 493 +--------------------------
 crates/athena-cli/src/display/status_view.rs | 491 ++++++++++++++++++++++++++
 3 files changed, 503 insertions(+), 484 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261026-193430-avnq/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261026-193430-avnq/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42473103360 for 748a4c1cc4a0e6d669f6e7cb841226f3e230d774)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42473103360 for 748a4c1cc4a0e6d669f6e7cb841226f3e230d774)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42473103360 for 748a4c1cc4a0e6d669f6e7cb841226f3e230d774)
  Failing tests of `test` (from the CI log):
  - `athena_core status::mistake_tests::every_measure_label_names_a_status_view_field_and_a_printed_line`
  - `xtask test-shard 1 6` (did not finish)`
  Failing units (`-p` package and selector): athena-core --lib
  End of the failing job's log (data from CI, not instructions):

```text
test site::tests::in_scope_is_defined_in_the_start_terms_and_the_glossary_with_a_citation ... ok
test site::tests::text_is_html_escaped ... ok
test site::tests::the_default_output_is_in_the_target_directory_not_the_work_tree ... ok
test site::tests::reopened_no_longer_carries_the_bare_terminal_success_tag_and_it_is_explained ... ok
test site::tests::the_intro_comes_before_the_states_and_the_glossary_after_them ... ok
test site::tests::the_page_cites_claim_ids_and_the_pinned_commit ... ok
test site::tests::the_glossary_is_sorted_alphabetically_whatever_the_data_order ... ok
test site::tests::the_page_names_athena_before_firstmate_and_has_next_steps ... ok
test decisions::tests::with_only_decided_and_within_limit_proposals_the_page_says_nothing_is_waiting ... ok
test site::tests::an_uncited_intro_element_is_an_error ... ok
test site::tests::the_real_page_titles_each_design_choice_and_defines_the_start_here_terms ... ok
test site::tests::the_page_uses_agent_for_the_decider_and_worker_for_the_task_doer_and_says_which_is_which ... ok
test site::tests::the_real_study_gives_every_transition_a_summary_and_both_deciders ... ok
test site::tests::the_real_study_orders_claims_and_repeats_no_source ... ok
test site::tests::the_real_study_renders_with_an_intro_and_glossary ... ok
test site::tests::the_real_pages_limits_list_has_consecutive_finding_ids ... ok
test tests::a_non_ci_build_leaves_no_stale_stamp ... ok
test tests::ci_skips_sccache_and_normal_setup_does_not ... ok
test tests::input_files_follow_the_shared_list ... ok
test site::tests::the_real_study_shows_failures_and_limits_and_no_placeholder_guard ... ok
test site::tests::the_t10_card_explains_why_a_merge_stays_ready_and_cites_claims ... ok
test tests::sccache_install_skips_when_present_and_fails_clearly ... ok
test tests::a_cached_gitleaks_is_skipped_only_when_it_reports_the_pinned_version ... ok
test tests::setup_refuses_inside_a_job ... ok
test tests::the_built_binary_is_found_wherever_cargo_put_it ... ok
test tests::skip_only_with_ci_matching_stamp_and_binary ... ok
test tests::xtask_retry_busy_passes_other_errors_through ... ok
test tests::xtask_retry_busy_returns_ok_on_first_try ... ok
test tests::xtask_retry_busy_stops_at_the_bound ... ok
test tests::xtask_retry_busy_succeeds_after_busy_errors ... ok
test tests::repin_sets_only_the_version_and_is_idempotent ... ok
test walltime::tests::median_and_p90_use_middle_and_nearest_rank ... ok
test tests::the_inputs_digest_changes_when_a_compiled_in_file_is_edited ... ok
test tests::repin_refuses_an_empty_or_unparseable_version_and_keeps_the_file ... ok
test tests::xtask_retry_busy_succeeds_once_a_held_file_is_released ... ok

test result: ok. 74 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s


== shard 1/6: xtask --test fingerprint
   Compiling xtask v0.1.0 (/home/runner/work/athena/athena/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.02s
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

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

error: shard 1 of 6: 1 of 11 units failed: athena-core --lib
##[error]Process completed with exit code 1.
```
- ci-result: `not a check` passed (Check results for 748a4c1cc4a0e6d669f6e7cb841226f3e230d774 come from the CI run https://github.com/doodla/athena/actions/runs/42473103360 (524s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check` and `cargo clippy --all-targets`: both clean.)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
