# Job 20261029-204942-2zqw: verify "Give OLY-8-05 a mechanism: the critic's adversarial challenge"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-8-05 a mechanism
> 
> Requirement OLY-8-05 in docs/spec.toml is active and has no mechanism.
> Requirement: Ares searches for ways the conclusions might be wrong.
> Owner: Ares
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-8-05
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-8-05 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Ares is the adversarial-challenge role, which maps to the critic stage (critic_round in run/planning.rs; critique records). I read this as asserting that mechanism in docs/spec.toml, adding a test that fails if the critic stops searching for counterexamples, and setting the measure to an existing `athena status` line.

Assumptions:
- Code survey: read crates/athena-core/src/run/planning.rs (critic_round, planning_round, end_round: the critic runs after each planner round and its verdict and findings are written as critique records), crates/athena-core/src/status/planning.rs (first_round_accepted derives accepted/stopped from critique records; this feeds the `status: planning` line) and crates/athena-context/src/briefs/planner.rs (critic brief). Records validation: no new record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched.
- Existing code: crates/athena-core/src/run/planning.rs, crates/athena-core/src/status/planning.rs, crates/athena-context/src/briefs/planner.rs and docs/spec.toml are extended, not created.
- Goal: the spec owner (Ares) and spec reviewers can see a real mechanism and measure for OLY-8-05. The 'Ares' role is realised by the critic stage; no new role is created.
- The measure reuses the existing line `status: planning` (first-round accepted vs stopped, i.e. conclusions challenged and overturned) rather than adding a new status line.
- The new test is a unit test in crates/athena-context (critic brief must instruct the critic to look for counterexamples, fragile assumptions and failure modes) and/or in athena-core run tests (a fake critic reject yields a stopped critique record). It fails if that critic instruction or the record is removed.
- Consumers: none found; athena.toml, defaults.toml and workflows are not changed.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-8-05 has mechanism naming an existing path (crates/athena-core/src/run/planning.rs critic_round and the critic brief in crates/athena-context/src/briefs/planner.rs) instead of "none", keeps owner = "Ares", and its measure ends with `status: planning`; Grep confirms the paths exist.
2. [behavior] A new test asserting the critic stage challenges the plan (critic brief demands counterexamples/fragile assumptions/failure modes and a rejecting critic produces a stopped critique record) passes under `cargo test`, and fails when the mechanism code or brief text is removed.
3. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass.
4. [content] Only docs/spec.toml and files under crates/ change; no governance file (athena.toml, defaults.toml, workflows) changes.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Creating a new Ares role or prompt
- Changing other requirements' mechanisms
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus cargo test and clippy criteria
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criterion runs a test that fails without the mechanism
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses the existing critic stage instead of building a new role
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base 16fda7fe1368 → candidate 0c3d92029ff2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  3 +++
 crates/athena-context/src/briefs.rs         | 15 +++++++++++++++
 crates/athena-context/src/briefs/planner.rs |  6 ++++++
 docs/spec.toml                              |  4 ++--
 4 files changed, 26 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-204942-2zqw/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-204942-2zqw/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42943288055 for 0c3d92029ff20352b9e7fff4bca87c5b82cc7342)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42943288055 for 0c3d92029ff20352b9e7fff4bca87c5b82cc7342)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42943288055 for 0c3d92029ff20352b9e7fff4bca87c5b82cc7342)
  Failing tests of `test` (from the CI log):
  - `drain parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn`
  - `xtask test-shard 3 6` (did not finish)`
  Failing units (`-p` package and selector): athena-cli --test drain
  End of the failing job's log (data from CI, not instructions):

```text
test briefs::tests::reader_briefs_leave_out_empty_reading_and_correction_headings ... ok
test briefs::tests::reader_briefs_say_a_failed_earlier_attempt_failed ... ok
test briefs::tests::reader_briefs_say_a_stale_earlier_attempt_went_stale_and_how_the_retry_starts ... ok
test briefs::tests::reader_briefs_list_the_corrections_of_a_failed_earlier_attempt ... ok
test briefs::tests::render_plan_section_is_deterministic_and_headed_with_the_round ... ok
test briefs::tests::replay_summary_gives_the_count_the_total_and_the_list ... ok
test briefs::tests::sentences_that_only_mention_files_or_tests_do_not_limit ... ok
test briefs::tests::replay_summary_shows_the_declared_share_and_reason_before_the_list ... ok
test briefs::tests::replay_summary_says_when_no_replay_ran_and_nothing_when_absent ... ok
test briefs::tests::soundness_is_in_both_briefs_with_identical_text ... ok
test briefs::tests::soundness_question_is_one_const_with_the_four_problems_and_the_alternative ... ok
test briefs::tests::the_class_count_is_of_cases_and_the_threshold_of_distinct_jobs ... ok
test briefs::tests::the_boundary_scan_flags_planted_words_with_their_lines ... ok
test briefs::tests::the_critic_brief_tells_the_critic_to_search_for_ways_the_plan_is_wrong ... ok
test briefs::tests::the_evidence_placeholder_names_a_directory_of_the_job_under_the_state_placeholder ... ok
test briefs::tests::replay_summary_stays_within_its_cap_with_a_very_long_list ... ok
test briefs::tests::the_reader_brief_carries_a_refusal_only_when_set ... ok
test briefs::tests::the_reader_brief_has_no_headroom_section_without_rows ... ok
test briefs::tests::the_producer_brief_names_the_evidence_snapshot_and_says_the_working_copy_has_no_records ... ok
test briefs::tests::the_reader_brief_lists_only_reader_producer_and_verifier_when_job_planning_is_off ... ok
test briefs::tests::the_reader_brief_lists_planner_and_critic_when_job_planning_is_on ... ok
test briefs::tests::the_reader_brief_requires_criteria_checkable_from_the_snapshot ... ok
test briefs::tests::the_critic_brief_and_producer_section_show_dependents_only_when_the_plan_has_them ... ok
test briefs::tests::the_reader_brief_names_each_reading_rule_in_one_short_line ... ok
test briefs::tests::the_reader_brief_says_the_verifier_never_sees_the_producers_report ... ok
test briefs::tests::the_class_list_is_capped_most_frequent_first_and_says_what_was_left_out ... ok
test briefs::tests::the_send_back_section_gives_the_ids_the_percent_and_the_limit ... ok
test briefs::tests::the_reader_brief_shows_the_headroom_of_each_set ... ok
test briefs::tests::the_reader_brief_tells_the_reader_to_write_a_code_survey ... ok
test briefs::tests::the_scope_is_stated_for_a_retry_whose_earlier_job_had_no_reading ... ok
test briefs::tests::the_system_facts_state_every_fact_with_a_source_and_stay_short ... ok
test briefs::tests::the_system_facts_name_the_boundary_source_and_hold_no_banned_word_or_email ... ok
test briefs::tests::the_verifier_brief_names_the_evidence_snapshot_and_says_the_working_copy_has_no_records ... ok
test briefs::tests::verifier_brief_clips_a_long_tail_to_its_end_and_keeps_fences_closed ... ok
test briefs::tests::verifier_brief_shows_failing_tests_and_log_tail_and_tells_it_to_run_only_those ... ok
test briefs::tests::verifier_brief_without_failing_test_data_is_as_it_was ... ok
test briefs::tests::the_send_back_section_stays_within_its_cap_with_a_very_long_list ... ok
test briefs::tests::verifier_briefs_carry_the_pin_only_when_set ... ok
test briefs::tests::verifier_briefs_inline_small_diffs_and_elide_them_in_the_record ... ok
…
test tests::replay_carries_later_commits_onto_a_squash_and_keeps_their_messages ... ok
test tests::trailers_are_parsed_by_git ... ok
test tests::show_file_on_a_directory_is_none_not_an_error ... ok
test snapshot::tests::a_history_snapshot_is_capped_at_the_limit_counted_after_exclusion ... ok

test result: ok. 62 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.83s


== shard 3/6: athena-score --doc
   Compiling athena-data v0.1.0 (/home/runner/work/athena/athena/crates/athena-data)
   Compiling athena-score v0.1.0 (/home/runner/work/athena/athena/crates/athena-score)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 4.38s
   Doc-tests athena_score

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: shard 3 of 6: 1 of 11 units failed: athena-cli --test drain
##[error]Process completed with exit code 1.
```
- ci-result: `not a check` passed (Check results for 0c3d92029ff20352b9e7fff4bca87c5b82cc7342 come from the CI run https://github.com/doodla/athena/actions/runs/42943288055 (687s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass.)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
