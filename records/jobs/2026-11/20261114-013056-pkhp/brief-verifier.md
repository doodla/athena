# Job 20261114-013056-pkhp: verify "Add version-pin and hermetic-check defaults; widen reader budget"

## Request (verbatim)

> try again on 20261114-012353-pbvc

## Reading (accepted by the user)

Retry of job 20261114-012353-pbvc (planning stopped on minor critic findings; no candidate). Root cause: defaults.toml is silent on version-pin policy and ambient-tool checks, so the reader asked the user (jobs 20261016-005805-p806, 20261025-183644-t67q). Add two prose-only defaults naming the existing repin command, raise the reader budget to about 2200, and record decision D181.

Assumptions:
- Goal and audience: the reader role (and the user who currently gets asked about pins) must find written stances on version pins and hermetic checks in defaults.toml, so such requests stop needing a user answer.
- Previous result, job 20261014-placeholder is not involved; job 20261114-012353-pbvc failed in planning with only minor findings and wrote no code. Its four minor findings are each addressed: (1) both entries are marked prose-only; (2) the version-pins entry names `cargo run -p xtask -- repin` (D154), which already exists, so no command is built and its factual slip no longer applies; (3) D181 states it departs from D170's rule that budgets never rise to fit a set; (4) new lines avoid words the boundary scan bans, checked by running the policy and budgets tests.
- Earlier jobs 20261016-005805-p806 (patch-tolerant version acceptance in claude.rs, xtask, wiring.rs, live.rs, README.md) is dropped; job 20261025-183644-t67q (nextest, athena.toml tools pins, ci.yml, checks.rs, gh.rs, config.rs, run.rs) is a separate job. Neither candidate's files are kept.
- Code survey: crates/athena-data/src/defaults.rs (render prints only the first sentence, cut at the first '. '; REPO_DEFAULTS at ~line 89; test parses the real defaults.toml), crates/athena-context/src/lib.rs (reader budget at ~line 176), crates/athena-core/src/run_pin_tests.rs (reader budget literal 'of 2000' at ~line 1625), crates/athena-core budgets.rs (computes reader set from defaults.toml and compares with docs/system-facts.md), xtask/tests/policy.rs (boundary scan). No record or data file is added under crates/, so check_record_file and AnyRecord are unaffected. The property is measured directly by rendering the real defaults and counting tokens.
- Environment: none; no CI, push or records read is needed.
- Consumers of defaults.toml: crates/athena-data/src/defaults.rs (parse and render), crates/athena-context (prompt composition), budgets tests, docs/system-facts.md figures.
- Estimate (not measured): the two new rendered lines add about 140-145 tokens, so the reader set lands near 2140, under 2200.
- The first sentence of each new entry must carry the stance, since render prints only that sentence.

## Acceptance criteria

1. [content] defaults.toml has a new [[default]] entry id version-pins whose first sentence says pins are exact, never auto-floated, and changed only by the person running Athena through the existing `cargo run -p xtask -- repin` command; scoped to code, config and build and marked prose-only.
2. [content] defaults.toml has a new [[default]] entry id hermetic-checks whose first sentence says a check never depends on inherited environment or ambient tools, the runner passes every path a check needs, and any preferred-versus-fallback tool choice is recorded in the check output; marked prose-only. defaults.toml is the only governance file changed (athena.toml, workflows, xtask unchanged per git diff).
3. [content] The reader budget at crates/athena-context/src/lib.rs is raised to about 2200, and the literal at crates/athena-core/src/run_pin_tests.rs (~1625) and any figure in docs/system-facts.md match it.
4. [content] docs/decisions.md gets a new appended decision D181 with a Provenance bullet naming the root cause, jobs 20261016-005805-p806 and 20261025-183644-t67q, and stating it departs from D170's rule that budgets never rise to fit a set; no existing entry is edited. CHANGELOG.md has an entry naming the root cause (defaults silent on pin and ambient-tool policy). No repin command is built.
5. [behavior] Run `cargo test`: a new test in crates/athena-data/src/defaults.rs parses the real defaults.toml, asserts both ids exist with key phrases (exact, never auto-floated, repin, ambient), and asserts the rendered reader prompt contains both; it fails on the base tree and passes with the change. The reader prompt budget tests, budgets tests and xtask policy test also pass.
6. [behavior] Run `cargo fmt --check` and `cargo clippy` (warnings denied); both pass. Recorded checks on the final tree also count.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/defaults.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)

## Out of scope

- Building a repin command: it already exists (D154, xtask/src/main.rs).
- Patch-tolerant version acceptance in claude.rs, xtask claude_accepts, wiring.rs, live.rs, README.md (job 20261016-005805-p806 candidate): dropped.
- cargo-nextest adoption, athena.toml [tools] pins, ci.yml, checks.rs, gh.rs, config.rs, run.rs (job 20261025-183644-t67q): separate job.
- Editing athena.toml, workflows or other prompt files.
- Changing planning or critic acceptance rules (the introspection's design change): separate job.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Behavior criterion checks the real defaults render into the reader prompt.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: New entries marked prose-only since nothing enforces them.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus fmt, clippy, test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in defaults.toml where the policy gap originates.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff. Optionally run `git show HEAD:defaults.toml` to see the base has no such id. Run: `git diff -- defaults.toml`. Observes: A new `[[default]]` with id `version-pins` and applies_to `["code", "config", "build"]`. Its first sentence is exactly: Version pins are exact and never auto-floated; only the person running Athena moves one, and the Claude Code pin (`claude_version`) moves through the existing `cargo run -p xtask -- repin` command. It has no '. ' before that point. A later sentence says 'Prose-only'. Its source names jobs 20261016-005805-p806 and 20261025-183644-t67q and D181. The `repin` command is named, and `git diff --stat` shows no xtask file.
- Criterion 2: Read `git diff -- defaults.toml` for the hermetic-checks entry, and `git diff --stat` for the file list. Run: `git diff --stat`. Observes: A new `hermetic-checks` entry with applies_to `["code", "config", "build", "tests"]`. Its first sentence is exactly: A check never depends on inherited environment or ambient tools; the runner passes every path a check needs, and any preferred-versus-fallback tool choice is recorded in the check output. A later sentence says 'Prose-only'. The stat lists exactly the seven deliverables: defaults.toml, crates/athena-data/src/defaults.rs, crates/athena-context/src/lib.rs, crates/athena-core/src/run_pin_tests.rs, docs/decisions.md, CHANGELOG.md and docs/system-facts.md. It lists no athena.toml, .github/workflows, xtask, rust-toolchain.toml or docs/decision-links.toml path.
- Criterion 3: Read `git diff -- crates/athena-context/src/lib.rs crates/athena-core/src/run_pin_tests.rs docs/system-facts.md`. Then run `cargo test --locked -p athena-cli --test budgets`, `cargo test --locked -p athena-context` and `cargo test --locked -p athena-core --lib run_pin_tests`. Run: `cargo test --locked -p athena-cli --test budgets`. Observes: `BudgetSet::Reader` returns 2_200 in lib.rs. The comment above the producer line no longer claims the reader budget never rises. run_pin_tests.rs:1625 takes the figure from `BudgetSet::Reader.budget()`. The `budget reader` line reads `<computed size> of 2200`. `system_facts_budgets_match_computed` and `the_reader_headroom_rows_equal_the_budget_figures_and_name_shared_files` (budgets.rs:125, :396) pass, as does the `the_reading_brief_carries_the_token_headroom_of_the_reader_and_verifier_sets` test (run_pin_tests.rs:1581). The computed size is estimated at about 2156 until measured, and `check_budget` returns an error above 2200.
- Criterion 4: Read the diff. Then run `cargo test --locked -p athena-cli --test backtest`, `cargo test --locked -p athena-core --test refusal_corpus` and `cargo test --locked -p athena-core --test buy_first`. Do not cite a result without running them. Run: `git diff -- docs/decisions.md CHANGELOG.md`. Observes: The decisions.md diff is additions only, at the end after D180's last line. It adds `## D181 · …` with Case, Decision, Alternatives, Tradeoff, Reversal cost and `- **Provenance:**` bullets. The Provenance bullet names the root cause (defaults.toml was silent on version-pin and ambient-tool policy), jobs 20261016-005805-p806 and 20261025-183644-t67q, the retry of 20261114-012353-pbvc, the measured reader size, and says the entry departs from D170's rule that budgets never rise to fit a set, with D93 as precedent. D181 also names the pages.yml floating tags as a known gap and proposes a follow-up job. It contains no 'amends', 'supersedes' or 'superseded by'. The CHANGELOG diff is one added entry at the top of Unreleased/Added. It names the same root cause, the budget raise from 2000 to 2200, D181, and that `cargo run -p xtask -- repin` already exists (D154). No repin code is added and no xtask path is in `git diff --stat`.
- Criterion 5: Add the test to `mod tests` in crates/athena-data/src/defaults.rs. It runs `DefaultsModel::parse(REPO_DEFAULTS)` and gets `version-pins` and `hermetic-checks` by id. It asserts applies_to `[code, config, build]` and `[code, config, build, tests]`. It asserts the pins statement contains 'exact', 'never auto-floated', 'repin' and 'Claude Code pin (`claude_version`)'. It asserts the hermetic statement contains 'ambient', 'inherited environment', 'every path' and 'preferred-versus-fallback'. It asserts both full statements contain 'Prose-only'. It asserts that `model.render()` contains both exact lines `- version-pins [code, config, build]: <first sentence>\n` and `- hermetic-checks [code, config, build, tests]: <first sentence>\n`. Then run the targeted test, `cargo test --locked -p athena-cli --test budgets`, `cargo test --locked -p athena-core --lib run_pin_tests`, `cargo test --locked -p xtask --test policy`, `cargo test --locked -p athena-cli --test backtest`, and finally the whole suite with `cargo test --locked`. Run: `cargo test --locked -p athena-data pin_and_hermetic_check_defaults_reach_the_reader`. Observes: The new test passes. The budgets, run_pin_tests headroom, boundary-scan and backtest tests also pass, and the whole `cargo test --locked` passes.
  Fails without the behavior: On the base tree defaults.toml has neither id, so the first `model.get("version-pins").unwrap()` panics and the test fails. If a first sentence held a '. ', or the stance sat after the first sentence, `render()` would print a shorter line and the exact-line assertions would fail. If the version-pins sentence claimed repin for every pin, the exact-line assertion would not match the agreed text.
  Test seam: Production entry points `DefaultsModel::parse` (defaults.rs:36) and `DefaultsModel::render` (defaults.rs:58). Guards: `deny_unknown_fields` (defaults.rs:7, :16), the duplicate-id check (:41) and the empty statement or source check (:44). The fixture is `REPO_DEFAULTS = include_str!("../../../defaults.toml")` at defaults.rs:89, the real file with no double. The seam exists today. The plan adds only the test (step 2). `render` is the text the reader loads (reading.rs:241-253 `reader_system_prompt`, and budgets.rs:34-39).
- Criterion 6: Run `cargo fmt --all --check`, then `cargo clippy --all-targets --locked -- -D warnings`. Recorded checks on the final tree (fmt, clippy, test) also count. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: Both commands exit 0 with no diff and no warnings.
  Fails without the behavior: A badly formatted new test or the reworded comment would fail `cargo fmt --all --check`. An unused import or a needless borrow in the changed `format!` call in run_pin_tests.rs would fail clippy.
  Test seam: The cargo commands are the production checks Athena runs (the 'Checks Athena will run' list). No fixture is used. The seam exists.

Planned files:

- defaults.toml (modify)
- crates/athena-data/src/defaults.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/system-facts.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

no replay: a check failed

## The change

Base 5224ad65b221 → candidate d978280dfdc2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  1 +
 crates/athena-context/src/lib.rs        |  8 ++++---
 crates/athena-core/src/run_pin_tests.rs |  5 +++--
 crates/athena-data/src/defaults.rs      | 37 +++++++++++++++++++++++++++++++++
 defaults.toml                           | 12 +++++++++++
 docs/decisions.md                       | 34 ++++++++++++++++++++++++++++++
 docs/system-facts.md                    |  2 +-
 7 files changed, 93 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-013056-pkhp/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-013056-pkhp/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43544517183 for d978280dfdc23b138e84b73dd2825cf4572ec275)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43544517183 for d978280dfdc23b138e84b73dd2825cf4572ec275)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43544517183 for d978280dfdc23b138e84b73dd2825cf4572ec275)
  Failing tests of `test` (from the CI log):
  - `drain parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn`
  - `xtask test-shard 3 6` (did not finish)`
  Failing units (`-p` package and selector): athena-cli --test drain
  End of the failing job's log (data from CI, not instructions):

```text
test records_branch::tests::bad_paths_and_empty_batches_are_refused_before_anything_is_written ... ok
test records_branch::tests::audit_since_reads_only_the_commits_after_it_and_the_helpers_find_the_new_files ... ok
test records_branch::tests::existing_paths_cannot_be_modified_deleted_or_renamed ... ok
test records_branch::tests::resolve_tip_prefers_the_descendant_and_reports_divergence ... ok
test records_branch::tests::the_first_commit_is_an_orphan_unrelated_to_main ... ok
test records_branch::tests::stale_tip_is_rejected_then_replayed ... ok
test records_branch::tests::a_remote_that_moves_on_every_attempt_ends_in_a_bounded_error_and_the_local_branch_stays ... ok
test records_branch::tests::two_writers_on_threads_both_land ... ok
test records_branch::tests::writing_the_same_bytes_again_is_a_no_op_that_returns_the_tip ... ok
test records_branch::tests::when_every_attempt_is_rejected_the_writer_stops_with_a_clear_error ... ok
test snapshot::tests::a_history_snapshot_lists_visible_commits_without_hidden_paths_or_a_repository ... ok
test records_branch::tests::sync_from_resets_a_local_branch_that_is_ahead_or_diverged_and_the_next_write_succeeds ... ok
test snapshot::tests::a_snapshot_seeded_from_a_kept_directory_copies_its_files_but_not_its_git ... ok
test snapshot::tests::a_locked_snapshot_refuses_edits_and_commits_and_keeps_head_and_tree ... ok
test snapshot::tests::a_seeded_snapshot_diffs_against_the_base_and_notices_an_untouched_seed ... ok
test snapshot::tests::a_walk_survives_entries_removed_after_the_directory_was_listed ... ok
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
test tests::a_push_url_on_another_repository_is_refused ... ok
test tests::a_relaxed_squash_text_accepts_no_job_trailer_and_refuses_two ... ok
test tests::a_push_git_itself_declines_stays_a_git_failure_not_a_refusal ... ok
test tests::a_squash_text_splits_the_title_and_keeps_one_job_trailer_last ... ok
test tests::an_origin_with_a_second_push_url_on_another_repository_is_refused ... ok
test tests::an_origin_with_two_fetch_urls_and_no_push_url_is_refused ... ok
test tests::a_remote_branch_is_fetched_without_moving_any_local_ref_and_set_ref_moves_one ... ok
test tests::commit_exists_tells_a_present_commit_from_an_absent_one_and_a_path ... ok
test tests::diff_tree_reports_raw_changes ... ok
test tests::commit_with_records_adds_the_records_on_top_of_the_base ... ok
…

== shard 3/6: xtask --test walltime
   Compiling athena-data v0.1.0 (/home/runner/work/athena/athena/crates/athena-data)
   Compiling athena-context v0.1.0 (/home/runner/work/athena/athena/crates/athena-context)
   Compiling xtask v0.1.0 (/home/runner/work/athena/athena/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 6.37s
     Running tests/walltime.rs (target/debug/deps/walltime-cd3c22a0e62c495c)

running 6 tests
test missing_timestamps_are_stated_not_printed_as_zero ... ok
test failed_and_rejected_jobs_are_reported_apart ... ok
test median_and_p90_of_dispatch_to_verified_time ... ok
test no_records_is_stated ... ok
test the_window_is_the_last_n_jobs_by_dispatch ... ok
test fewer_than_twenty_jobs_is_stated_and_twenty_is_not_flagged ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

error: shard 3 of 6: 1 of 13 units failed: athena-cli --test drain
##[error]Process completed with exit code 1.
```
- ci-result: `not a check` passed (Check results for d978280dfdc23b138e84b73dd2825cf4572ec275 come from the CI run https://github.com/doodla/athena/actions/runs/43544517183 (701s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check` and `cargo clippy` (warnings denied); both pass. Recorded checks on the final tree also count.)
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-data pin_and_hermetic_check_defaults_reach_the_reader`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/lib.rs, defaults.toml, docs/decisions.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
   Compiling version_check v0.9.5
   Compiling quote v1.0.47
   Compiling generic-array v0.14.7
   Compiling typenum v1.20.1
   Compiling zmij v1.0.23
   Compiling libc v0.2.190
   Compiling cfg-if v1.0.5
   Compiling thiserror v2.0.21
   Compiling serde_json v1.0.151
   Compiling syn v3.0.7
   Compiling getrandom v0.3.4
   Compiling winnow v1.0.4
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling serde v1.0.229
   Compiling digest v0.10.7
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling serde_spanned v1.1.2
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling winnow v0.7.15
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling cpufeatures v0.2.17
   Compiling jiff-core v0.1.1
   Compiling itoa v1.0.18
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling memchr v2.8.3
   Compiling toml v0.9.12+spec-1.1.0
   Compiling jiff v0.2.38
   Compiling sha2 v0.10.9
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.71s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_data-62eb92b0c349858b)

running 1 test
test defaults::tests::pin_and_hermetic_check_defaults_reach_the_reader ... FAILED

failures:

---- defaults::tests::pin_and_hermetic_check_defaults_reach_the_reader stdout ----

thread 'defaults::tests::pin_and_hermetic_check_defaults_reach_the_reader' (29776) panicked at crates/athena-data/src/defaults.rs:150:46:
called `Option::unwrap()` on a `None` value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    defaults::tests::pin_and_hermetic_check_defaults_reach_the_reader

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 219 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-data --lib`
     Running tests/repo_config.rs (<job>/target/verify/debug/deps/repo_config-bf3c3e8d6e6af283)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-data --lib`
```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo clippy --all-targets --locked -- -D warnings`). Result: not run: not a plain cargo test command.

## Failing tests from the CI run

Athena read these failing tests from the CI log of the run it reused. Run only the failing tests (`cargo test -p <crate> <name>`) unless you have a reason to run more; do not rerun the whole suite to find them.

A name has the form `<binary> <test>`: the binary is the library crate (`athena_core`), an integration-test file, or `doc-tests <crate>`. The failing-units line gives the `-p` package and the selector for it (`--lib`, `--test <file>`, `--doc`), so the command reads `cargo test -p athena-core --lib <test>`. A name in parentheses means the list is incomplete: run the whole unit or shard it names.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
