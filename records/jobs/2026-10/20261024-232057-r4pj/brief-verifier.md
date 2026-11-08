# Job 20261024-232057-r4pj: verify "Trim verifier and study producer prompts to free 100+ tokens each"

## Request (verbatim)

> Free prompt-budget room before more instructions are added: the change verifier composes at 1298 of its 1300-token budget, the study verifier at 1296 and the study producer at 1276 (cargo test --locked -p athena-cli --test budgets -- --nocapture prints them). Remove text in those prompts and their shared frames that repeats what the briefs already say or that no current rule needs.
> Acceptance:
> - The budgets test passes and each of those three sets is at least 100 tokens under its budget.
> - No brief test or behavior criterion changes meaning.
> - docs/decisions.md lists what was removed and why.
> Files: crates/athena-context/prompts, crates/athena-cap-change/prompts, crates/athena-cap-study/prompts, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Cut repeated or rule-less text from the change verifier, study verifier and study producer prompts and the shared frames so each set sits at least 100 tokens under its 1300 budget, without changing any brief rule. Record the removals in docs/decisions.md and CHANGELOG.md, and refresh the budget lines in docs/system-facts.md that the facts test checks.

Assumptions:
- Goal: free prompt-budget room for later instructions; audience is maintainers adding rules and the verifier checking the diff.
- Code survey: read crates/athena-cli/tests/budgets.rs; it composes each role set from the prompt files (crates/athena-context/prompts, crates/athena-cap-change/prompts, crates/athena-cap-study/prompts) and calls check_budget; system_facts_budgets_match_computed compares docs/system-facts.md budget lines to computed sizes. No record or data file is added.
- The producer is the study producer (cap-study/prompts/producer.md at 1257 by the headroom table, 1276 per the request) plus frame producer; the 'at least 100 under' target is measured by the budgets test output, not by the table.
- Shared frames (athena-context producer.md and verifier.md) also feed the change producer and the planner/critic; trims there only shrink them, but docs/system-facts.md budget lines for producer, verifier, planner and critic must be updated to computed values or system_facts_budgets_match_computed fails.
- Contract.md is shared by every set including the reader; trimming it is allowed but not required, and the reader set must not grow.
- Existing tests asserting prompt phrases (brief tests) must still pass unmodified; the producer greps them before deleting any sentence.
- docs/system-facts.md is not named in the request but is required by the facts test; it is listed as a deliverable.

## Acceptance criteria

1. [behavior] Run `cargo test --locked -p athena-cli --test budgets -- --nocapture`: it passes, and the printed sizes for the change verifier, the study verifier and the study producer are each at most 1200.
2. [behavior] Run `cargo test --locked` for the workspace: all tests pass, including every prompt/brief test, with no test file modified in the diff, and `cargo clippy --locked --all-targets` is clean.
3. [content] The diff touches only prompt .md files under crates/athena-context/prompts, crates/athena-cap-change/prompts and crates/athena-cap-study/prompts, plus docs/decisions.md, CHANGELOG.md and docs/system-facts.md; no brief test, behavior criterion or rule is reworded in meaning (removed text is repetition of brief content or backs no current rule).
4. [content] docs/decisions.md has a new entry listing each removed passage (or group) and why it was removable, citing the file; a reader can match each bullet to the diff.
5. [content] CHANGELOG.md has an entry describing the prompt trim and the new headroom.
6. [content] docs/system-facts.md budget lines equal the computed sizes (system_facts_budgets_match_computed passes) and budgets are unchanged.
7. [content] Standing defaults: nothing secret or personal is committed; the change is one focused edit with text written for an outside reader.

## Expected deliverables

- crates/athena-cap-change/prompts/verifier.md (modify)
- crates/athena-cap-study/prompts/verifier.md (modify)
- crates/athena-cap-study/prompts/producer.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-context/prompts/contract.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)
- crates/athena-cap-change/prompts/producer.md (modify)

## Out of scope

- Changing any budget value, test, or athena.toml/defaults.toml
- Adding new rules or instructions to the prompts
- Trimming the reader, planner or critic prompts beyond what shared frames imply

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the budgets test output and that rules remain, not that edits were made.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: decisions.md entry written for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run cargo test and clippy.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused prompt-trim change.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The budgets test enforces the headroom.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base bbc99bbd23e8 → candidate 6c05807b8cb3. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 |  5 +++++
 crates/athena-cap-change/prompts/verifier.md |  5 ++---
 crates/athena-cap-study/prompts/verifier.md  |  2 +-
 crates/athena-context/prompts/producer.md    |  9 ++++-----
 crates/athena-context/prompts/verifier.md    |  4 +---
 docs/decisions.md                            | 13 +++++++++++++
 docs/system-facts.md                         |  4 ++--
 7 files changed, 28 insertions(+), 14 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261024-232057-r4pj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261024-232057-r4pj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42430281422 for 6c05807b8cb363de9cbf65da2dcada5e8992a96c)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42430281422 for 6c05807b8cb363de9cbf65da2dcada5e8992a96c)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42430281422 for 6c05807b8cb363de9cbf65da2dcada5e8992a96c)
  Failing tests of `test` (from the CI log):
  - `athena_context tests::the_verifier_frame_limits_snapshot_runs_to_the_snapshot`
  - `xtask test-shard 3 6` (did not finish)`
  Failing units (`-p` package and selector): athena-context --lib
  End of the failing job's log (data from CI, not instructions):

```text
test records_branch::tests::the_first_commit_is_an_orphan_unrelated_to_main ... ok
test records_branch::tests::a_remote_that_moves_on_every_attempt_ends_in_a_bounded_error_and_the_local_branch_stays ... ok
test records_branch::tests::two_writers_on_threads_both_land ... ok
test records_branch::tests::writing_the_same_bytes_again_is_a_no_op_that_returns_the_tip ... ok
test records_branch::tests::when_every_attempt_is_rejected_the_writer_stops_with_a_clear_error ... ok
test records_branch::tests::sync_from_resets_a_local_branch_that_is_ahead_or_diverged_and_the_next_write_succeeds ... ok
test snapshot::tests::a_history_snapshot_lists_visible_commits_without_hidden_paths_or_a_repository ... ok
test snapshot::tests::a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree ... ok
test snapshot::tests::a_seeded_snapshot_diffs_against_the_base_and_notices_an_untouched_seed ... ok
test snapshot::tests::a_walk_survives_entries_removed_after_the_directory_was_listed ... ok
test snapshot::tests::a_snapshot_seeded_from_a_kept_directory_copies_its_files_but_not_its_git ... ok
test snapshot::tests::a_snapshot_without_a_seed_is_never_unchanged_since_seed ... ok
test snapshot::tests::plain_snapshots_have_no_history_file ... ok
test snapshot::tests::producer_edits_become_a_change_list ... ok
test snapshot::tests::an_unlocked_snapshot_takes_edits_and_commits ... ok
test snapshot::tests::restore_returns_a_work_tree_to_a_tree ... ok
test snapshot::tests::snapshots_hold_only_what_agents_may_see ... ok
test snapshot::tests::snapshot_git_cannot_reach_the_users_history_or_hidden_content ... ok
test tests::a_child_that_exits_without_reading_is_not_an_error ... ok
test snapshot::tests::verifier_snapshots_carry_base_then_candidate ... ok
test tests::a_guarded_push_replaces_the_branch_when_the_remote_is_where_expected ... ok
test tests::a_guarded_push_is_rejected_and_changes_nothing_when_the_remote_moved ... ok
test tests::a_relaxed_squash_text_accepts_no_job_trailer_and_refuses_two ... ok
test tests::commit_exists_tells_a_present_commit_from_an_absent_one_and_a_path ... ok
test tests::a_squash_text_splits_the_title_and_keeps_one_job_trailer_last ... ok
test tests::diff_tree_reports_raw_changes ... ok
test tests::commit_with_records_adds_the_records_on_top_of_the_base ... ok
test tests::git_is_recent_enough ... ok
test tests::ensure_commit_fetches_a_missing_commit_without_moving_any_ref ... ok
test tests::log_first_parent_reads_hash_time_subject_and_trailers_newest_first ... ok
test tests::a_remote_branch_is_fetched_without_moving_any_local_ref_and_set_ref_moves_one ... ok
test tests::log_history_reads_hash_date_subject_and_paths_newest_first ... ok
test tests::merge_tree_report_fails_on_a_bad_revision ... ok
test tests::merge_tree_detects_conflicts ... ok
test tests::merge_tree_report_lists_a_modify_delete_conflict_without_marker_lines ... ok
test tests::merge_tree_report_of_a_clean_merge_is_the_merged_tree ... ok
test tests::piping_over_a_megabyte_through_an_echoing_child_does_not_deadlock ... ok
test tests::merge_tree_report_of_a_conflict_keeps_the_tree_and_lists_the_path_once ... ok
test tests::merge_tree_report_reads_a_path_with_a_space_exactly ... ok
test tests::replay_keeps_a_message_without_trailers_verbatim ... ok
test tests::replay_reports_a_conflict_and_changes_nothing ... ok
test tests::reset_keep_moves_the_branch_and_keeps_unrelated_local_edits ... ok
test tests::replay_carries_later_commits_onto_a_squash_and_keeps_their_messages ... ok
test tests::trailers_are_parsed_by_git ... ok
test tests::show_file_on_a_directory_is_none_not_an_error ... ok
test snapshot::tests::a_history_snapshot_is_capped_at_the_limit_counted_after_exclusion ... ok

test result: ok. 61 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.01s


== shard 3/6: athena-testkit --doc
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
   Doc-tests athena_testkit

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: shard 3 of 6: 1 of 11 units failed: athena-context --lib
##[error]Process completed with exit code 1.
```
- ci-result: `not a check` passed (Check results for 6c05807b8cb363de9cbf65da2dcada5e8992a96c come from the CI run https://github.com/doodla/athena/actions/runs/42430281422 (756s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --locked` for the workspace: all tests pass, including every prompt/brief test, with no test file modified in the diff, and `cargo clippy --locked --all-targets` is clean.)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
