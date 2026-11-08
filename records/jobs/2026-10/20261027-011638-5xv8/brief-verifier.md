# Job 20261027-011638-5xv8: verify "Record crate placement facts and add a test-placement reading rule"

## Request (verbatim)

> Add the crate dependency facts that matter for test placement to docs/system-facts.md (which crates have library targets, which may depend on which, and where xtask code must be tested), and add a reading-brief rule to place each test in a crate that can reach the target. Merge failure classes 20261026-155850-0ksw and 20261026-170231-4fcf into one. Acceptance: system-facts drift test covers the new facts.
> 
> Acceptance:
> - The change is made as described: Reading or plan puts a test in a crate that cannot import the code under test (e.g. xtask has no library target); found late, fixed by hand; registry holds two classes for one failure
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: docs/system-facts.md

## Reading (accepted by the user)

Add a crate-dependency fact (library targets, allowed dependency directions, where xtask code is tested) to docs/system-facts.md, add a ninth reading-brief rule that a test must sit in a crate that can reach the code under test, and extend the system-facts drift test to cover the new facts. The failure-class merge lives in records/, which no job changes, so it is left out. The goal is to stop readings and plans from putting tests where they cannot import the target; the audience is the planner, critic and reader, plus the verifier.

Assumptions:
- Code survey: read crates/athena-context/src/briefs.rs (READING_RULES, an array of 8; SYSTEM_FACTS include_str; the test the_system_facts_state_every_fact_with_a_source_and_stay_short with its 5,000-byte cap and marker list), crates/athena-cli/tests/budgets.rs (check_system_facts and system_facts_drift_is_rejected), crates/athena-core/src/reading/checks.rs and surveys.rs (check_code_survey order). No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. Resolution and probing are untouched.
- Goal: the planner, critic and reader stop placing tests in a crate that cannot import the code under test, such as xtask, which has no library target. The facts file is read by the planner and critic; the new rule is read by the reader.
- The facts must come from the Cargo.toml files. Only athena-cli declares a [[bin]] (named athena); xtask is a binary-only package with a name 'xtask' and its tests sit in xtask/tests/policy.rs plus unit tests inside xtask/src/main.rs. The producer must read each crates/*/Cargo.toml [dependencies] to state the allowed dependency directions rather than guess.
- The new facts bullet follows the file's form: a bold title and a 'Source' naming Cargo.toml paths. The file stays under 5,000 bytes. The new fact stays free of the banned-word scan list and of email-like text.
- The drift test extension is in crates/athena-cli/tests/budgets.rs: a check that reads the crate facts lines (a fixed line form such as '- crate <name>: lib|bin-only; depends on <list>') and compares them with cargo metadata or the Cargo.toml files, rejecting a doctored line, a missing crate and an unknown crate. This mirrors the budget-line check.
- The reading-brief rule is added to READING_RULES in briefs.rs as a ninth line, with its array length and any test or count that asserts 8 updated. The reader prompt is at 1997 of 2000 tokens, and the rules render in the reader brief rather than the always-loaded set, so the producer must run the budgets test to confirm no set exceeds its budget. If the rule is enforced by a check, it may be marked prose-only; this reading assumes prose-only and says so in the rule's text.
- Failure classes 20261026-155850-0ksw and 20261026-170231-4fcf live in the records registry. records/ is absent from the snapshot and never changes, so the merge is outside this job. Neither id appears in any tracked file (searched with Grep).
- Docs/system-facts.md is embedded by include_str, so the budget lines stay accurate; budgets are not changed. CHANGELOG.md gets one Unreleased line.

## Acceptance criteria

1. [content] docs/system-facts.md has a new bullet with a 'Source' naming Cargo.toml paths. It states which crates have library targets, that athena-cli is the only crate with a binary and that xtask has no library target, which crates may depend on which, and that xtask code is tested in xtask/tests/ or in unit tests inside xtask/src/main.rs. A reader can check each claim against the Cargo.toml files.
2. [content] READING_RULES in crates/athena-context/src/briefs.rs has a new line saying to place each test in a crate that can reach the code under test (for example, no integration test for a binary-only crate like xtask), and the array length and every test that counts the rules are updated. A Grep of briefs.rs shows the line.
3. [behavior] Run cargo test: crates/athena-cli/tests/budgets.rs has a drift test that covers the new crate facts lines. It fails when a facts line is doctored, missing or names an unknown crate, and it passes on the real file. The existing system_facts_budgets_match_computed and system_facts_drift_is_rejected tests still pass.
4. [behavior] Run cargo test: the_system_facts_state_every_fact_with_a_source_and_stay_short passes with the new bullet (markers added for it, file under 5,000 bytes), and the reader prompt-budget tests pass.
5. [behavior] Run cargo fmt --check, cargo clippy and cargo test: all pass with no unrelated behavior changes.
6. [content] The diff changes nothing under records/ and no governance files (athena.toml, defaults.toml, workflows); the failure-class merge is not attempted.

## Expected deliverables

- docs/system-facts.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/reading.rs (modify)

## Out of scope

- Merging failure classes 20261026-155850-0ksw and 20261026-170231-4fcf: they live in the records registry (records/), which no job changes; the user must do it with the records tooling.
- A mechanical check that refuses a misplaced test; the new rule is prose-only.
- Changing budgets, athena.toml, defaults.toml, or any workflow.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The drift test is Rust in the existing budgets.rs test file.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New facts lines are covered by the drift test; cargo test and clippy must pass.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The new facts name Cargo.toml sources.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs fmt --check, clippy and test.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Facts are enforced by the drift test; the reading rule is marked prose-only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change; the records merge is excluded.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The rule targets the whole class of unreachable test placement, not the one xtask case.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check by diff plus the positive drift test. The diff shows the bullet. The test compares every crate line with the Cargo.toml files, so each claim in the lines can be checked against the manifests. Run: `cargo test -p athena-cli --test budgets system_facts_crate_lines_match_manifests`. Observes: Passes on the real file, with one line per workspace member (12). `xtask` is `bin`. `cli` is `lib+bin` and is the only line with `bin` and `lib`. Every other crate is `lib`. Each `deps` and `dev` list equals the manifest. `git diff -- docs/system-facts.md` shows the bullet. The bullet says that cli is the only crate with a binary, that xtask has no library target, and that xtask code is tested in `xtask/tests/` or in unit tests in `xtask/src/` (main.rs or its modules). It also says that a crate reaches only crates that do not depend on it (Cargo refuses cycles). It ends with a Source naming the Cargo.toml paths.
- Criterion 2: Grep of briefs.rs for the new line, plus the existing rule-count test run with the new length. Run: `cargo test -p athena-context the_reader_brief_names_each_reading_rule_in_one_short_line`. Observes: Passes with `READING_RULES: [&str; 9]`, `assert_eq!(READING_RULES.len(), 9)`, the nine lines each at most 220 bytes, and the measured section cap. `Grep 'Test placement' crates/athena-context/src/briefs.rs` shows the line and the doc comment that now says a `(prose only)` line has no check and causes no refusal. `cargo test -p athena-context the_reader_brief_tells_the_reader_to_write_a_code_survey` also passes, since it splits the same section.
- Criterion 3: Behavior test. `check_crate_facts(text, rows)` is fed the real `SYSTEM_FACTS` and doctored copies made with `str::replace`. `rows` come from `crate_rows()`, which reads the real Cargo.toml files. Run: `cargo test -p athena-cli --test budgets system_facts`. Observes: `system_facts_crate_lines_match_manifests` passes on the real file. `system_facts_crate_drift_is_rejected` first asserts that the real file holds `crate_line(row)` for each row. It then asserts an error for each doctored copy: a flipped target (`lib` to `bin`), an added dep, a removed dep, a doctored `dev` list (a dev name removed or added), a removed line (missing crate), an appended `- crate steward: lib` line (unknown crate), a duplicated line and a malformed line. The filter also runs the existing `system_facts_budgets_match_computed` and `system_facts_drift_is_rejected`, which still pass because their `- budget ` filter never matches `- crate `.
  Fails without the behavior: Without `check_crate_facts`, or if it accepted a doctored line, the `unwrap_err()` calls in the negative test panic. Without the bullet, the positive test fails with a message naming the crate that has no line.
  Test seam: Production entry point: `check_crate_facts(text, rows)` in crates/athena-cli/tests/budgets.rs, the pattern of the existing `check_system_facts` (called by `system_facts_budgets_match_computed`). Plan step 3 adds it, with `crate_rows()` and `crate_line()`. Guards: a workspace member with no line, a line for a non-member, a repeated line and a line of the wrong form are all errors. Fixture: `athena_context::briefs::SYSTEM_FACTS` (an `include_str!` of docs/system-facts.md, which exists) with string edits. Truth: `crate_rows()` reads the root and member Cargo.toml files through CARGO_MANIFEST_DIR. The manifests are the oracle, and no double holds logic.
- Criterion 4: Run the existing facts-file tests and the budget tests with the new bullet and markers in place. Run: `cargo test -p athena-context the_system_facts`. Observes: `the_system_facts_state_every_fact_with_a_source_and_stay_short` passes: the new markers are found, there are at least 12 bullets, each has 'Source', and `SYSTEM_FACTS.len() < 5_000`. `the_system_facts_name_the_boundary_source_and_hold_no_banned_word_or_email` passes. `planner_and_critic_briefs_carry_the_system_facts_whole` passes. Then `cargo test -p athena-cli --test budgets` shows `every_role_set_is_within_its_budget` and the reader headroom test passing, with the reader line still 1997 of 2000.
  Fails without the behavior: A bullet that lacks a marker, a 'Source' or room fails the first test. A banned word or '@' in the bullet fails the second. A stale count (`>= 11`) would not fail, which is why step 5 raises it to 12 so a lost bullet is noticed.
  Test seam: Production entry point: the `SYSTEM_FACTS` constant (briefs.rs line 1764, an `include_str!` of docs/system-facts.md), read by the existing tests at briefs.rs lines 3747, 3768 and 3810, and by `computed_rows` and `check_budget` in budgets.rs. These exist today. The real file is the fixture, and plan step 5 edits the marker list.
- Criterion 5: Run the three checks Athena runs, in order. Run: `cargo test --locked`. Observes: `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings` are clean. `cargo test --locked` passes in every crate, including athena-core (reading.rs and the replay tests), xtask (policy.rs and the unit tests) and athena-cli drain.rs. No Cargo.toml or Cargo.lock change.
  Fails without the behavior: A rustfmt difference, a clippy warning in the new test code (an unneeded clone or an unused helper), a stale rule count or a facts file of 5,000 bytes or more fails the run.
  Test seam: Production entry points: the repository's whole test suite and the three checks. The seam exists today, and no double is added.
- Criterion 6: Inspect the diff. Run: `git diff --stat`. Observes: Only docs/system-facts.md, crates/athena-context/src/briefs.rs, crates/athena-cli/tests/budgets.rs and CHANGELOG.md appear. Nothing under records/, athena.toml, defaults.toml, .github/workflows/ or any Cargo file.

Planned files:

- docs/system-facts.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 471a03c72290 → candidate 68f5a9a750a9. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  10 ++
 crates/athena-cli/tests/budgets.rs  | 187 ++++++++++++++++++++++++++++++++++++
 crates/athena-context/src/briefs.rs |  28 ++++--
 docs/system-facts.md                |  17 +++-
 4 files changed, 234 insertions(+), 8 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-011638-5xv8/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-011638-5xv8/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42483909474 for 68f5a9a750a9824be7b0ae77c18b94b0eadc72f6)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42483909474 for 68f5a9a750a9824be7b0ae77c18b94b0eadc72f6)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42483909474 for 68f5a9a750a9824be7b0ae77c18b94b0eadc72f6)
- ci-result: `not a check` passed (Check results for 68f5a9a750a9824be7b0ae77c18b94b0eadc72f6 come from the CI run https://github.com/doodla/athena/actions/runs/42483909474 (767s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo fmt --check, cargo clippy and cargo test: all pass with no unrelated behavior changes.)
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --test budgets system_facts`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, docs/system-facts.md.
  End of the output (data, not instructions):

```text
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling regex v1.13.1
   Compiling utf8parse v0.2.2
   Compiling anstyle-parse v1.0.0
   Compiling syn v2.0.119
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling anstyle-query v1.1.5
   Compiling anstyle v1.0.14
   Compiling is_terminal_polyfill v1.70.2
   Compiling colorchoice v1.0.5
   Compiling anstream v1.0.0
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling strsim v0.11.1
   Compiling clap_lex v1.1.1
   Compiling heck v0.5.0
   Compiling clap_builder v4.6.7
   Compiling clap_derive v4.6.7
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling clap v4.6.7
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 42.78s
     Running tests/budgets.rs (<job>/target/verify/debug/deps/budgets-3c161170e6563139)

running 4 tests
test system_facts_budgets_match_computed ... ok
test system_facts_crate_lines_match_manifests ... FAILED
test system_facts_drift_is_rejected ... ok
test system_facts_crate_drift_is_rejected ... FAILED

failures:

---- system_facts_crate_lines_match_manifests stdout ----

thread 'system_facts_crate_lines_match_manifests' (25398) panicked at crates/athena-cli/tests/budgets.rs:278:5:
the facts file has no crate line for data
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- system_facts_crate_drift_is_rejected stdout ----

thread 'system_facts_crate_drift_is_rejected' (25397) panicked at crates/athena-cli/tests/budgets.rs:303:9:
the file has no line `- crate data: lib`


failures:
    system_facts_crate_drift_is_rejected
    system_facts_crate_lines_match_manifests

test result: FAILED. 2 passed; 2 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-cli --test budgets`
error: 1 target failed:
    `-p athena-cli --test budgets`
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context the_system_facts`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, docs/system-facts.md.
  End of the output (data, not instructions):

```text
   Compiling unicode-ident v1.0.26
   Compiling libc v0.2.190
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling proc-macro2 v1.0.107
   Compiling winnow v0.7.15
   Compiling quote v1.0.47
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling syn v3.0.7
   Compiling getrandom v0.3.4
   Compiling toml v0.9.12+spec-1.1.0
   Compiling serde_derive_internals v0.30.0
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling ref-cast-impl v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling ref-cast v1.0.27
   Compiling thiserror v2.0.21
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 17.44s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_context-6da838b4f8c2a29c)

running 3 tests
test briefs::tests::the_system_facts_state_every_fact_with_a_source_and_stay_short ... FAILED
test briefs::tests::the_system_facts_name_the_boundary_source_and_hold_no_banned_word_or_email ... ok
test briefs::tests::planner_and_critic_briefs_carry_the_system_facts_whole ... ok

failures:

---- briefs::tests::the_system_facts_state_every_fact_with_a_source_and_stay_short stdout ----

thread 'briefs::tests::the_system_facts_state_every_fact_with_a_source_and_stay_short' (26115) panicked at crates/athena-context/src/briefs.rs:3801:13:
the facts omit "Crate reach"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    briefs::tests::the_system_facts_state_every_fact_with_a_source_and_stay_short

test result: FAILED. 2 passed; 1 failed; 0 ignored; 0 measured; 121 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-context --lib`
error: 1 target failed:
    `-p athena-context --lib`
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, docs/system-facts.md.
  End of the output (data, not instructions):

```text
test drain_passes_over_an_item_whose_second_failure_pauses_its_line_and_runs_the_next_item ... ok
test an_interrupted_job_whose_retry_gave_no_reading_is_read_again_not_retried_twice ... ok
test drain_order_follows_the_plan_as_the_queue_changes ... ok
test drain_lists_and_retries_an_interrupted_job_and_leaves_one_on_another_channel_alone ... ok
test drain_start_reclaims_the_build_directory_of_an_interrupted_job_and_keeps_its_work_and_candidate ... ok
test drain_real_steps_pass_over_the_item_of_a_line_that_is_flagged_after_two_failures ... ok
test every_file_of_the_shipped_study_is_page_text_except_the_two_structural_ones ... ok
test every_item_failing_twice_is_passed_over_and_none_stops_the_batch ... ok
test drain_real_steps_do_not_hold_an_item_for_a_stored_pause_that_no_longer_holds ... ok
test drain_real_steps_pass_over_the_item_of_a_line_that_is_still_paused_without_reading_it ... ok
test every_role_runs_at_its_configured_model_and_effort_and_the_record_names_both ... ok
test in_flight_bound_does_not_apply_with_max_in_flight_one ... ok
test in_flight_bound_holds_a_start_in_a_drain_without_a_failure_and_a_later_drain_runs_it ... ok
test in_flight_bound_lets_a_start_through_when_the_disk_fits_the_jobs_allowed ... ok
test in_flight_bound_makes_the_extra_start_wait_and_a_later_start_runs_the_same_job ... ok
test drain_works_the_requests_a_plan_read_in_plan_order_without_reading_them_again ... ok
test in_flight_bound_never_holds_a_job_when_no_other_is_in_flight ... ok
test in_flight_row_goes_producer_checks_verifier_and_is_producer_again_after_a_self_check_send_back ... ok
test items_are_worked_in_plan_order_not_oldest_first ... ok
test in_flight_row_names_the_producers_live_work_tree ... ok
test low_disk_after_the_build_holds_the_job_and_does_not_fail_it ... ok
test in_flight_row_goes_when_an_item_fails_twice ... ok
test in_flight_row_moves_from_verifier_back_to_producer_when_the_rejected_job_is_retried ... ok
test low_disk_after_the_checks_holds_the_job_before_the_verifier_runs ... ok
test low_disk_at_dispatch_that_cleanup_does_not_cure_holds_the_job_queued_and_not_rejected ... ok
test low_disk_at_dispatch_removes_idle_build_directories_first_and_then_dispatches ... ok
test low_disk_after_the_producer_removes_idle_build_directories_before_the_checks_run ... ok
test low_disk_from_the_writer_build_is_cleared_before_the_checks_gate_so_the_job_is_not_held ... ok
test low_disk_with_a_green_ci_run_does_not_hold_the_checks_stage ... ok
test only_changes_to_code_and_the_files_that_build_it_touch_the_build ... ok
test parallel::a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn ... ok
test parallel::a_landing_with_an_unreadable_config_after_another_workers_stop_is_landed_and_ends_with_reinstall ... ok
test parallel::a_rebuild_after_the_unreadable_landing_restarts_and_leaves_another_workers_stop_alone ... ok
test parallel::a_stop_starts_no_further_item_and_finishes_those_in_flight ... ok
test parallel::an_item_that_depends_on_another_starts_only_after_it_has_finished ... ok
test parallel::an_item_without_a_plan_entry_is_unplanned_and_not_a_group_of_one ... ok
test low_disk_before_the_checks_holds_the_job_and_a_retry_with_space_runs_the_checks ... ok
test parallel::group_sizes_are_recorded_with_a_bound_of_one ... ok
test parallel::group_sizes_are_recorded_with_a_bound_of_two ... ok
…

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_score

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_testkit

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 2 targets failed:
    `-p athena-cli --test budgets`
    `-p athena-context --lib`
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
