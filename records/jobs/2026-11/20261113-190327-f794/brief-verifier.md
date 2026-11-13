# Job 20261113-190327-f794: verify "Design direct-to-main landing to retire the long-lived landing branch"

## Request (verbatim)

> Priority: high
> 
> Throughput design: The landing-and-rebuild stage is 2310.0 of the 2342.1 median minutes per landed job (THROUGHPUT.md).
> 
> Athena's throughput review 20261113-190159-mapn named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: The landing-and-rebuild stage is 2310.0 of the 2342.1 median minutes per landed job (THROUGHPUT.md). The cause is the one long-lived local landing branch: every pull request is cut from it and carries all earlier unlanded jobs' commits, so ...
> Stage: landing and rebuild
> Proposed change: Retire the long-lived local landing branch as a staging area. When a job's verifier passes, fetch host main and build that job's single landing commit directly on the current tip by a three-way merge from the job's recorded base (Git::merge...
> Predicted saving: 5.0 minutes per landed job; the median was 2342.1 minutes per landed job when the review ran.
> Why: THROUGHPUT.md (measured): the median per landed job is 2342.1 min. Landing and rebuild is 2310.0 of it; the other five stages are small (producer 24.0, verifier 2.8, planning 2.7, checks 0.8, reading 0.6). The design notes show the cause. T...
> 
> A change that implements this design must cite throughput review 20261113-190159-mapn in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for throughput review 20261113-190159-mapn: build each job's single landing commit on fetched host main by a three-way merge from its recorded base, retiring the long-lived local landing branch. The note states the predicted 5.0 min per landed job saving and how the next throughput review measures it. The audience is Athena's owner and the next throughput review.

Assumptions:
- Existing code: crates/athena-git/src/lib.rs, crates/athena-core/src/land.rs, crates/athena-core/src/pr.rs, docs/design/direct-main-merge-tree-landing.md, docs/design/per-job-main-landing.md, docs/design/continuous-landing.md, docs/design/native-pr-landing.md; these are read and not created by this job.
- Goal and audience: a design note for the owner and for the next throughput review that scores the prediction.
- The job is a docs-only design; no code, athena.toml, defaults.toml or workflow file changes. Prior design notes on direct-main landing exist and the new note must say how it differs from or builds on them.
- Throughput review 20261113-190159-mapn and THROUGHPUT.md are cited as given in the request; I could not re-measure them. Figures are quoted from the request.
- The 5.0 minute saving is the request's prediction; the note states it and labels it an estimate.
- Environment: none reached; the job only edits files in the repository.
- Defaults applied: designed (alternatives and risks written down), evidence (cite sources, label estimates), strangers, public.

## Acceptance criteria

1. [content] A new note under docs/design/ (diff shows the added file) describes retiring the long-lived local landing branch and building each job's single landing commit on fetched host main by a three-way merge from the job's recorded base, and explains how this removes the landing-and-rebuild wait (2310.0 of 2342.1 median minutes per landed job).
2. [content] The design has an Existing code section that reuses or extends crates/athena-git/src/lib.rs, crates/athena-core/src/land.rs and docs/design/direct-main-merge-tree-landing.md, naming which functions or notes it builds on and what changes.
3. [content] The note states the predicted saving, 5.0 minutes per landed job, labeled as an estimate against the 2342.1 median, and says how the next throughput review measures it: the median minutes per landed job in THROUGHPUT.md for jobs landed after the change, compared with the prediction.
4. [content] The note cites throughput review 20261113-190159-mapn, and states that a change implementing it must cite that id in its request text.
5. [content] The note lists at least two alternatives considered with why each was rejected, and the risks, such as merge conflicts at landing and the failure path.
6. [content] CHANGELOG.md records the job in a new entry; docs/decisions.md and records/ are not edited except by a new appended decision with a Provenance bullet if one is added.
7. [behavior] Run cargo fmt --check, cargo clippy and cargo test; each passes, or Athena's recorded check on the same tree passed, showing the docs-only change breaks nothing.

## Expected deliverables

- docs/design/direct-main-single-commit-landing.md (create)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Implementing the landing change in crates/ code
- Editing athena.toml, defaults.toml or workflows
- Editing existing entries of docs/decisions.md or anything under records/

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Note includes alternatives and risks.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Figures cite THROUGHPUT.md and review id; the saving is labeled an estimate.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The saving is tied to the next review's measured median.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base 623745018239 → candidate d5b2a7571646. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                     |   1 +
 docs/decisions.md                                |  20 ++++
 docs/design/direct-main-single-commit-landing.md | 142 +++++++++++++++++++++++
 3 files changed, 163 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261113-190327-f794/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261113-190327-f794/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43492550291 for d5b2a75716464eb16b60dffeeadb0897ff174fa7)
  Failing tests of `test` (from the CI log):
  - `drain parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn`
  - `xtask test-shard 3 6` (did not finish)`
  Failing units (`-p` package and selector): athena-cli --test drain
  End of the failing job's log (data from CI, not instructions):

```text
test records_branch::tests::audit_passes_a_normal_history_with_extra_parents_and_reports_a_dropped_or_changed_file ... ok
test records_branch::tests::existing_paths_cannot_be_modified_deleted_or_renamed ... ok
test records_branch::tests::audit_since_reads_only_the_commits_after_it_and_the_helpers_find_the_new_files ... ok
test records_branch::tests::resolve_tip_prefers_the_descendant_and_reports_divergence ... ok
test records_branch::tests::the_first_commit_is_an_orphan_unrelated_to_main ... ok
test records_branch::tests::a_remote_that_moves_on_every_attempt_ends_in_a_bounded_error_and_the_local_branch_stays ... ok
test records_branch::tests::stale_tip_is_rejected_then_replayed ... ok
test records_branch::tests::two_writers_on_threads_both_land ... ok
test records_branch::tests::writing_the_same_bytes_again_is_a_no_op_that_returns_the_tip ... ok
test records_branch::tests::when_every_attempt_is_rejected_the_writer_stops_with_a_clear_error ... ok
test snapshot::tests::a_history_snapshot_lists_visible_commits_without_hidden_paths_or_a_repository ... ok
test records_branch::tests::sync_from_resets_a_local_branch_that_is_ahead_or_diverged_and_the_next_write_succeeds ... ok
test snapshot::tests::a_seeded_snapshot_diffs_against_the_base_and_notices_an_untouched_seed ... ok
test snapshot::tests::a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree ... ok
test snapshot::tests::a_walk_survives_entries_removed_after_the_directory_was_listed ... ok
test snapshot::tests::a_snapshot_seeded_from_a_kept_directory_copies_its_files_but_not_its_git ... ok
test snapshot::tests::a_snapshot_without_a_seed_is_never_unchanged_since_seed ... ok
test snapshot::tests::plain_snapshots_have_no_history_file ... ok
test snapshot::tests::an_unlocked_snapshot_takes_edits_and_commits ... ok
test snapshot::tests::producer_edits_become_a_change_list ... ok
test snapshot::tests::restore_returns_a_work_tree_to_a_tree ... ok
test snapshot::tests::snapshot_git_cannot_reach_the_users_history_or_hidden_content ... ok
test tests::a_child_that_exits_without_reading_is_not_an_error ... ok
test snapshot::tests::snapshots_hold_only_what_agents_may_see ... ok
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
…
running 14 tests
test tests::a_cost_known_only_as_a_bound_scores_provisional_and_a_firm_cost_does_not ... ok
test tests::a_provisional_quantity_does_not_count_in_aggregates ... ok
test tests::a_rules_string_selects_its_version_and_an_unknown_one_selects_none ... ok
test tests::brier_values ... ok
test tests::padding_deliverables_never_lowers_the_score ... ok
test tests::omitting_a_changed_path_never_helps ... ok
test tests::brier_is_strictly_proper_on_a_grid ... ok
test tests::skill_is_withheld_below_the_minimum_job_count ... ok
test tests::reference_uses_priors_without_history_and_moves_with_it ... ok
test tests::vague_forecasts_score_worse_than_sharp_calibrated_ones ... ok
test tests::v1_scores_a_bound_only_cost_as_not_provisional_and_records_v1 ... ok
test tests::wis_median_term_is_proper_for_the_median ... ok
test tests::wis_rewards_tight_correct_intervals_and_punishes_misses ... ok
test tests::zero_cost_is_floored_not_undefined ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: shard 3 of 6: 1 of 14 units failed: athena-cli --test drain
##[error]Process completed with exit code 1.
```
- ci-result: `not a check` passed (Check results for d5b2a75716464eb16b60dffeeadb0897ff174fa7 come from the CI run https://github.com/doodla/athena/actions/runs/43492550291 (917s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo fmt --check, cargo clippy and cargo test; each passes, or Athena's recorded check on the same tree passed, showing the docs-only change breaks nothing.)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
