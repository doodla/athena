# Job 20261111-213010-8jgz: verify "Match later_similar_task on class of work, not any later landed job"

## Request (verbatim)

> try again on 20261029-233102-c04w

## Reading (accepted by the user)

Retry of job 20261029-233102-c04w (OLY-10-03 later-similar-task mechanism): redo the status check so "prevented" needs a later landed job of the same class of work, using the stored class_of_work, and prove it over real records. Fixes the verifier's major finding (any later job counted) and both minors.

Assumptions:
- Prior job 20261029-233102-c04w: its candidate files are kept as deliverables (spec.toml, introspection.rs, status.rs, display.rs, status_build_measurement.rs, CHANGELOG.md) and redone from base; the producer reworks later_similar_task. Findings addressed: (major) any later landed job counted, fixed by matching class_of_work; (minor) single later job and early return, fixed by requiring a minimum count of later same-class jobs and checking every accepted proposal; (minor) failed-ranks-before-rejected guess, fixed by ranking only on stages the records state and treating infrastructure failures and other categories as not shown/equal rank, with the rationale in the doc comment.
- Goal and audience: an owner running `athena status` must see a truthful OLY-10-03 line; a verifier judges it against real records. Unrelated later jobs must never produce 'shown'.
- Code survey: read crates/athena-data/src/records.rs (Job.class_of_work: Option<String> at line 607; AnyRecord at line 1089; check_record_file at line 1415), crates/athena-core/src/status.rs (landed_job_ids at line 382, which feeds the status measures) and the prior introspection.rs function. This job adds no record or data file and does not change a record schema, so check_record_file and AnyRecord need no change and records.rs is not a deliverable; existing record round-trip tests keep covering the schemas. Class comes from JobRecords.job in crates/athena-core/src/store.rs; the proposal's applies_to lists the classes. The measure is computed directly from those records.
- Mechanism owner stays Apollo; the measure ends with `status: not shown` or a label athena status prints.
- Defaults applied: outcome-check and no-empty-success. A job with no class, no records or no later job gives not shown, never shown.
- Environment: none; records are read through the existing store accessors, no push, CI or history read.

## Acceptance criteria

1. [behavior] Test in crates/athena-data/src/records/introspection.rs: an accepted proposal plus enough later landed jobs of the same class_of_work with no recurrence gives prevented. Run `cargo test -p athena-data introspection`.
2. [behavior] Test: a later landed job of a different class, or with no class, gives not shown; no later job gives not shown; a pending, rejected or pre-acceptance case gives not shown. Same run; the tests fail if the check ignores class.
3. [behavior] Test: a same-class recurrence at a stage the records show to be earlier gives detected earlier; same or later stage gives not shown; a recurrence on any accepted proposal blocks prevented. Same run.
4. [behavior] A test in crates/athena-core/src/status.rs runs the measure over fixture records holding both a same-class and a different-class later job and requires different answers, and the crates/athena-cli test asserts the athena status line equals the StatusView field. Run `cargo test -p athena-core status` and `cargo test -p athena-cli status`.
5. [content] The doc comment on later_similar_task in crates/athena-data/src/records/introspection.rs no longer says records carry no class for a landed job, and states the class match, the minimum count and the stage ranking.
6. [content] CHANGELOG.md has an entry that records the measure's output over the real records with one example of each outcome (prevented, not prevented by different class, no later job); Grep shows it.
7. [content] docs/spec.toml OLY-10-03 names the existing function path and the status line function as mechanism, keeps owner Apollo, and the measure ends in `status: not shown` or a printed athena status label; Grep shows it.
8. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-data/src/records/introspection.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/status_build_measurement.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing OLY-10-01, OLY-10-02 or other requirements
- Changing the proposal, evaluation, introspection or job record schemas; crates/athena-data/src/records.rs is not changed
- Changing athena.toml, defaults.toml or CI workflows
- Writing real records or running a live demonstration job
- The generic standing code-check for all status measures proposed by the earlier introspection (separate job)

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the line is true only for same-class later jobs, over fixtures and recorded real-record output.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: No records, no later job or no class gives not shown.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base 1cfbaf6cc806 → candidate a37357573285. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                       |  17 +
 crates/athena-cli/src/display.rs                   |   1 +
 .../athena-cli/tests/status_build_measurement.rs   |  23 ++
 crates/athena-core/src/status.rs                   | 117 +++++++
 crates/athena-data/src/records/introspection.rs    | 371 +++++++++++++++++++++
 docs/spec.toml                                     |   4 +-
 6 files changed, 531 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261111-213010-8jgz/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-213010-8jgz/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43108047668 for a37357573285ba4b944fa1956bb738ef24a4d00b)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43108047668 for a37357573285ba4b944fa1956bb738ef24a4d00b)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43108047668 for a37357573285ba4b944fa1956bb738ef24a4d00b)
  Failing tests of `test` (from the CI log):
  - `drain parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn`
  - `xtask test-shard 3 6` (did not finish)`
  Failing units (`-p` package and selector): athena-cli --test drain
  End of the failing job's log (data from CI, not instructions):

```text
test records_branch::tests::two_writers_on_threads_both_land ... ok
test records_branch::tests::writing_the_same_bytes_again_is_a_no_op_that_returns_the_tip ... ok
test records_branch::tests::when_every_attempt_is_rejected_the_writer_stops_with_a_clear_error ... ok
test snapshot::tests::a_history_snapshot_lists_visible_commits_without_hidden_paths_or_a_repository ... ok
test records_branch::tests::sync_from_resets_a_local_branch_that_is_ahead_or_diverged_and_the_next_write_succeeds ... ok
test snapshot::tests::a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree ... ok
test snapshot::tests::a_snapshot_seeded_from_a_kept_directory_copies_its_files_but_not_its_git ... ok
test snapshot::tests::a_walk_survives_entries_removed_after_the_directory_was_listed ... ok
test snapshot::tests::a_seeded_snapshot_diffs_against_the_base_and_notices_an_untouched_seed ... ok
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
test tests::log_history_reads_hash_date_subject_and_paths_newest_first ... ok
test tests::a_remote_branch_is_fetched_without_moving_any_local_ref_and_set_ref_moves_one ... ok
test tests::merge_tree_report_fails_on_a_bad_revision ... ok
test tests::merge_tree_detects_conflicts ... ok
test tests::merge_tree_report_lists_a_modify_delete_conflict_without_marker_lines ... ok
test tests::merge_tree_report_of_a_clean_merge_is_the_merged_tree ... ok
test tests::merge_tree_report_of_a_conflict_keeps_the_tree_and_lists_the_path_once ... ok
test tests::piping_over_a_megabyte_through_an_echoing_child_does_not_deadlock ... ok
test tests::merge_tree_report_reads_a_path_with_a_space_exactly ... ok
test tests::replay_keeps_a_message_without_trailers_verbatim ... ok
test tests::replay_reports_a_conflict_and_changes_nothing ... ok
test tests::reset_keep_moves_the_branch_and_keeps_unrelated_local_edits ... ok
test tests::replay_carries_later_commits_onto_a_squash_and_keeps_their_messages ... ok
test tests::trailers_are_parsed_by_git ... ok
test tests::show_file_on_a_directory_is_none_not_an_error ... ok
test snapshot::tests::a_history_snapshot_is_capped_at_the_limit_counted_after_exclusion ... ok

test result: ok. 62 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.84s


== shard 3/6: athena-score --doc
   Compiling athena-data v0.1.0 (/home/runner/work/athena/athena/crates/athena-data)
   Compiling athena-score v0.1.0 (/home/runner/work/athena/athena/crates/athena-score)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 7.44s
   Doc-tests athena_score

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: shard 3 of 6: 1 of 12 units failed: athena-cli --test drain
##[error]Process completed with exit code 1.
```
- ci-result: `not a check` passed (Check results for a37357573285ba4b944fa1956bb738ef24a4d00b come from the CI run https://github.com/doodla/athena/actions/runs/43108047668 (855s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass.)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
