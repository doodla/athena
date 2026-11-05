# Job 20261105-103552-6s9v: verify "Raise [parallel] max_concurrency from 1 to 2 in athena.toml"

## Request (verbatim)

> Raise [parallel] max_concurrency in athena.toml from 1 to 2, so a batch runs two independent jobs at once (the parallel drain already supports up to 8). The disk check already holds a job when free disk is low, so two builds cannot fill the disk.
> 
> Files changed: athena.toml only (one value; no new key, so the config schema does not change).
> 
> Acceptance:
> - athena.toml has max_concurrency = 2 and no other change
> - cargo test --locked -p athena-data passes

## Reading (accepted by the user)

Change the single value max_concurrency in the [parallel] table of athena.toml from 1 to 2 so a batch runs two independent jobs at once. The audience is whoever runs batches. No other file or key changes.

Assumptions:
- Goal: let a batch run two independent jobs concurrently; the drain supports up to 8 and the disk check holds jobs when disk is low, as the request states.
- The max_concurrency key at athena.toml line 195 is the one under [parallel].
- Governance file athena.toml is a deliverable and named in the criteria; defaults.toml and workflows need no change.
- Code survey: no code change is made; crates/ is not edited, so no behavior beyond the cargo test criterion applies.

## Acceptance criteria

1. [content] Grep of athena.toml finds max_concurrency = 2 at the [parallel] table (line about 195) and no line with max_concurrency = 1.
2. [content] git diff shows athena.toml as the only changed file, with exactly one changed line: max_concurrency = 1 becomes max_concurrency = 2, and no new key.
3. [behavior] Run cargo test --locked -p athena-data; it passes.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)

## Out of scope

- Changing the parallel drain code or its maximum of 8
- Changing the disk check
- Adding config keys or changing the config schema
- defaults.toml and .github/workflows

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One value changed in one file.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test --locked -p athena-data and checks the diff.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the value and the diff scope.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

no replay: a check failed

## The change

Base ead62e58cf23 → candidate 324d0c665a1a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 athena.toml | 2 +-
 1 file changed, 1 insertion(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-103552-6s9v/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-103552-6s9v/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42059062290 for 324d0c665a1a3ab2184fb3c771949bb5f645d2cd)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42059062290 for 324d0c665a1a3ab2184fb3c771949bb5f645d2cd)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42059062290 for 324d0c665a1a3ab2184fb3c771949bb5f645d2cd)
  Failing tests of `test` (from the CI log):
  - `drain a_change_to_max_concurrency_alone_does_not_invalidate_a_reading`
  - `drain parallel::every_landing_path_ends_in_reinstall_for_any_unreadable_config`
  - `drain unreadable_at_reports_what_the_running_schema_rejects`
  - `xtask test-shard 3 6` (did not finish)`
  End of the failing job's log (data from CI, not instructions):

```text
test records_branch::tests::writing_the_same_bytes_again_is_a_no_op_that_returns_the_tip ... ok
test records_branch::tests::when_every_attempt_is_rejected_the_writer_stops_with_a_clear_error ... ok
test records_branch::tests::sync_from_resets_a_local_branch_that_is_ahead_or_diverged_and_the_next_write_succeeds ... ok
test snapshot::tests::a_history_snapshot_lists_visible_commits_without_hidden_paths_or_a_repository ... ok
test snapshot::tests::a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree ... ok
test snapshot::tests::a_snapshot_seeded_from_a_kept_directory_copies_its_files_but_not_its_git ... ok
test snapshot::tests::a_walk_survives_entries_removed_after_the_directory_was_listed ... ok
test snapshot::tests::a_seeded_snapshot_diffs_against_the_base_and_notices_an_untouched_seed ... ok
test snapshot::tests::a_snapshot_without_a_seed_is_never_unchanged_since_seed ... ok
test snapshot::tests::plain_snapshots_have_no_history_file ... ok
test snapshot::tests::an_unlocked_snapshot_takes_edits_and_commits ... ok
test snapshot::tests::producer_edits_become_a_change_list ... ok
test snapshot::tests::restore_returns_a_work_tree_to_a_tree ... ok
test snapshot::tests::snapshot_git_cannot_reach_the_users_history_or_hidden_content ... ok
test snapshot::tests::snapshots_hold_only_what_agents_may_see ... ok
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
test tests::a_remote_branch_is_fetched_without_moving_any_local_ref_and_set_ref_moves_one ... ok
test tests::log_first_parent_reads_hash_time_subject_and_trailers_newest_first ... ok
test tests::log_history_reads_hash_date_subject_and_paths_newest_first ... ok
test tests::piping_over_a_megabyte_through_an_echoing_child_does_not_deadlock ... ok
test tests::merge_tree_detects_conflicts ... ok
test tests::replay_keeps_a_message_without_trailers_verbatim ... ok
test tests::replay_reports_a_conflict_and_changes_nothing ... ok
test tests::replay_carries_later_commits_onto_a_squash_and_keeps_their_messages ... ok
test tests::reset_keep_moves_the_branch_and_keeps_unrelated_local_edits ... ok
test tests::trailers_are_parsed_by_git ... ok
test tests::show_file_on_a_directory_is_none_not_an_error ... ok
test snapshot::tests::a_history_snapshot_is_capped_at_the_limit_counted_after_exclusion ... ok

test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.26s

error: shard 3 of 6: 1 of 10 units failed: athena-cli --test drain
##[error]Process completed with exit code 1.
Post job cleanup.
[command]/usr/bin/git version
git version 2.56.0
Temporarily overriding HOME='/home/runner/work/_temp/2c6ee0ab-9f94-4371-8387-a1853c25f8ef' before making global git config changes
Adding repository directory to the temporary git global config as a safe directory
[command]/usr/bin/git config --global --add safe.directory /home/runner/work/athena/athena
Removing SSH command configuration
[command]/usr/bin/git config --local --name-only --get-regexp core\.sshCommand
[command]/usr/bin/git submodule foreach --recursive sh -c "git config --local --name-only --get-regexp 'core\.sshCommand' && git config --local --unset-all 'core.sshCommand' || :"
Removing HTTP extra header
[command]/usr/bin/git config --local --name-only --get-regexp http\.https\:\/\/github\.com\/\.extraheader
[command]/usr/bin/git submodule foreach --recursive sh -c "git config --local --name-only --get-regexp 'http\.https\:\/\/github\.com\/\.extraheader' && git config --local --unset-all 'http.https://github.com/.extraheader' || :"
Removing includeIf entries pointing to credentials config files
[command]/usr/bin/git config --local --name-only --get-regexp ^includeIf\.gitdir:
[command]/usr/bin/git submodule foreach --recursive git config --local --show-origin --name-only --get-regexp remote.origin.url
Cleaning up orphan processes
##[warning]Node.js 20 is deprecated. The following actions target Node.js 20 but are being forced to run on Node.js 24: actions/cache/restore@5a3ec84eff668545956fd18022155c47e93e2684. For more information see: https://github.blog/changelog/2025-09-19-deprecation-of-node-20-on-github-actions-runners/
```
- ci-result: `not a check` passed (Check results for 324d0c665a1a3ab2184fb3c771949bb5f645d2cd come from the CI run https://github.com/doodla/athena/actions/runs/42059062290 (535s): fmt success, clippy success, test failure.)

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
