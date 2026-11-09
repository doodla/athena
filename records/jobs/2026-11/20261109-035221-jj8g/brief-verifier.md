# Job 20261109-035221-jj8g: verify "Rewrite agent prompts leaner and rebase the prompt token budgets"

## Request (verbatim)

> Hmm. I kinda want to rethink that budget and instructions. What does the model actually need to know? Knowledge can be in skills or reference docs or whatever as well.
> 
>  Those instructions look kinda mid

## Reading (accepted by the user)

Reading "that budget and instructions" as the fixed prompt token budgets (crates/athena-context/src/lib.rs) and the always-loaded role prompts. The job rewrites the prompts to hold only what each model needs to act, moves reference knowledge into a docs/ file agents can read on demand, and resets the budgets to match. The audience is the maintainer, who finds the prompts mediocre.

Assumptions:
- Term ladder: none used; 'budget' means the fixed estimated-token budget per always-loaded prompt set (BudgetSet::budget in crates/athena-context/src/lib.rs), not the [budget] USD caps in athena.toml.
- Goal: prompts that give each role only what it needs to act correctly, with lookup-style knowledge moved to a reference doc. Audience: the maintainer, plus the agents who read the prompts.
- Code survey: crates/athena-context/src/lib.rs defines BudgetSet budgets (contract 350, reader 2000, producer/verifier 1300, others 600), compose() and check_budget(). crates/athena-context/src/briefs.rs has tests that quote headroom rows such as '1298 of 1300'. Prompts live in crates/athena-context/prompts/*.md and crates/athena-cap-change/prompts/ and crates/athena-cap-study/prompts/. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched. Token use is measured directly with estimate_tokens on the composed text through check_budget.
- Existing code: crates/athena-context/src/lib.rs, crates/athena-context/prompts/, crates/athena-cap-change/prompts/, crates/athena-cap-study/prompts/ are extended, not recreated.
- Consumers: none found for athena.toml changes; the budget values live in lib.rs, so athena.toml and defaults.toml stay unchanged.
- The rewrite keeps every rule that a check enforces; prose-only guidance may be shortened or moved. The reference doc is read by agents with the Read tool.
- Only enabled roles (reader, planner, critic, producer, verifier) get new content.

## Acceptance criteria

1. [behavior] `cargo test --locked` passes, including the budget tests in crates/athena-context (every set within its budget) and any updated headroom-row tests in crates/athena-context/src/briefs.rs.
2. [behavior] `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check` are clean.
3. [content] `git diff` shows the always-loaded prompt files under crates/athena-context/prompts/ and the capability prompts rewritten. For each role, the diff drops text that restates defaults, catalog or facts already in the brief, and keeps the role's job, output contract and hard rules.
4. [content] A reference doc under docs/ holds the knowledge moved out of the prompts, and each rewritten prompt that depends on it names its path. The doc exists and the pointer path matches it.
5. [content] The BudgetSet::budget values in crates/athena-context/src/lib.rs are changed only where the rewrite justifies it; the diff gives a reason, and no budget rises without a matching cut elsewhere.
6. [content] Every rule the old prompts stated that a check enforces (for example the reader's Code survey and criterion rules) is still stated in the new prompts or in the referenced doc. Grep of the old and new text shows each is present.
7. [content] Outcome check: the diff includes a measured before-and-after token count per prompt set, taken from check_budget or the headroom rows. The sets are leaner, or the extra headroom is stated.
8. [content] CHANGELOG.md has an entry for the prompt and budget change.

## Expected deliverables

- crates/athena-context/prompts/contract.md (modify)
- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-cap-change/prompts/producer.md (modify)
- crates/athena-cap-change/prompts/verifier.md (modify)
- crates/athena-cap-study/prompts/producer.md (modify)
- crates/athena-cap-study/prompts/verifier.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/agent-reference.md (create)
- CHANGELOG.md (modify)

## Out of scope

- The USD [budget] caps in athena.toml and defaults.toml
- Changing which roles exist or the pipeline's logic
- Study-capability behavior beyond prompt wording
- records/ and owner files

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion requires measured before/after token counts and the retention of enforced rules.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test and clippy to pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run cargo test, clippy and fmt --check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Limited to prompts, budget constants and one reference doc.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Read 'that budget and instructions' from the repo as the prompt token budgets and role prompts.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The reference doc and the prompts are written for outside readers.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Budgets stay enforced by check_budget; prose-only guidance may move to the doc.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 4)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the whole suite. Entry points are the existing check_budget (lib.rs:222) and compose (lib.rs:142), reached through athena_cli::wiring::capabilities() in crates/athena-cli/tests/budgets.rs, which builds the real sets from the compiled-in prompt files. Tests that run: every_role_set_is_within_its_budget, system_facts_budgets_match_computed, system_facts_drift_is_rejected, the athena-context wording and ratchet tests, the new frames_name_the_reference_doc_as_optional_and_it_exists, the_reading_brief_carries_the_token_headroom_of_the_reader_and_verifier_sets (run_pin_tests.rs:1581), shipped_verifier_prompts_have_no_verdict_rule_conflict (verify.rs), the athena-cli mirrors target, and the xtask policy boundary scan on the new doc. No double: the prompt files and budgets are the real ones, injected by include_str!. The briefs.rs headroom-row tests are unchanged and still pass because they use a synthetic row. Run: `cargo test --locked`. Observes: Every set is at or under its budget; the facts lines equal the computed sizes; every pinned phrase and severity term is present in its frame; the pointer test passes; no verdict-rule conflict; the headroom-row pin holds (or, if the forced pin edit was needed, the rendered row still matches); the new doc passes the boundary scan.
  Fails without the behavior: A set over the new Producer or Verifier budget fails every_role_set_is_within_its_budget; a stale facts line fails system_facts_budgets_match_computed; a dropped anchor or severity line fails a lib.rs wording test or the new test; a banned word in the doc fails the xtask policy test.
  Test seam: check_budget, compose, athena_cli::wiring::capabilities() and the *_FRAME constants exist (read in lib.rs and budgets.rs). The anchor test is added by step 8. No double.
- Criterion 2: Run fmt in check mode, then clippy with warnings denied as a second command. The only Rust edits are in lib.rs (two budget constants and comments, ratcheted tests, one new test) and, only if the pin fails, one assertion in run_pin_tests.rs. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings`. Observes: Both commands exit 0 with no diff or warning output.
  Fails without the behavior: An unformatted edit to lib.rs fails fmt; an unused variable or needless borrow in the new test fails clippy.
  Test seam: The Athena checks named in the brief (fmt, clippy); they exist.
- Criterion 3: Read the diff of the ten prompt files. For each role, confirm each removed line restates the output schema, the brief, the catalog or a default, and the retained lines are the job, the output points the schema lacks, the severity definitions and the hard rules. Run: `git diff -- crates/athena-context/prompts crates/athena-cap-change/prompts crates/athena-cap-study/prompts`. Observes: All ten files show net byte reductions (the context prompts also gain one pointer line each in five files). No deleted line states a rule a check enforces (compare with the criterion 6 anchors). The critic and verifier severity lines are still present.
- Criterion 4: The test (added by step 8) calls compose() on the real reader, planner, critic, producer and verifier sets, the text the agent receives. It asserts each composed set contains the conditional pointer ('if docs/agent-reference.md exists') exactly once, so a repository without the file is told nothing false. It asserts every anchor in its explicit list (rule phrases plus the four severity terms) is in its composed frame. It asserts the study and change capability sections carry no pointer. It asserts CARGO_MANIFEST_DIR/../../docs/agent-reference.md is a file; this covers Athena's own tree only and the test comment says so. It does not assert anything about anchor phrases in the doc. The limit is also stated in the doc header and the CHANGELOG, and the User question asks about compiling the doc in. The test's file-exists assertion is the demonstration of the doc's existence; no separate ls. Run: `cargo test -p athena-context frames_name_the_reference_doc_as_optional_and_it_exists`. Observes: The test passes: each of the five composed sets has the conditional pointer once; the capability sections have none; every anchor is in its frame; the doc file exists at the path the pointer names.
  Fails without the behavior: A misspelled pointer, a missing doc, an unconditional pointer, a frame that omits the pointer, or an anchor or severity term absent from a frame fails the test.
  Test seam: compose (lib.rs:142) over the real *_FRAME constants and capability sections exists. The anchor list and test are added by step 8. No double.
- Criterion 5: Read the lib.rs diff. Only BudgetSet::Producer and BudgetSet::Verifier change, each with a comment giving the rule by relation (largest set, 90%, section 8) and no figure. Contract 350, Reader 2000 and the 600 values are unchanged, and no value rises. Then run the budgets test with output shown and compare each printed size with its budget. Run: `git diff -- crates/athena-context/src/lib.rs`. Observes: The Producer and Verifier values go down from 1300 and the comments give the reason. A second command, 'cargo test -p athena-cli --test budgets every_role_set_is_within_its_budget -- --nocapture', prints sizes each at or under budget, with the largest set at or below 90% of its budget unless the shortfall is stated in the comment.
- Criterion 6: (a) Read the old text with git show HEAD:<path> and grep the new files for each anchor. Reader: 'Check whether the outcome needs a change to athena.toml, defaults.toml', '.github/workflows/', 'list that file as a deliverable and name it in an acceptance criterion'. Producer: 'governance file', 'does not list as a deliverable', 'stop and report it in open_issues; do not edit it', 'Never mark a requirement row Covered'. Verifier: the REQUIRED_FIELDS sentences, the five snapshot phrases and the severity line. Planner: external-service, commit-shape, seam, refusal_corpus and dependents wording. Critic: major-finding wording and the severity line. (b) The reader's Code survey, evidence, governance and terms rules live in READING_RULES and CODE_SURVEY_GUIDE (briefs/reader.rs:287-314), are not edited, and are pinned by briefs.rs:3541-3560. The criterion rules stay in the reader frame. (c) The existing and new tests pin the rest. A second command, 'cargo test -p athena-core --lib verify', runs the verdict-rule check on the three verifier prompts. Run: `cargo test -p athena-context`. Observes: All wording tests in lib.rs and briefs.rs pass, including the new anchor list. The verify tests report no verdict-rule conflict.
  Fails without the behavior: Deleting any pinned phrase fails its assertion, for example 'planner frame lacks: ...' or 'D43: no instruction names the field `commands`'.
  Test seam: PLANNER_FRAME, CRITIC_FRAME, PRODUCER_FRAME, READER_FRAME and VERIFIER_FRAME text, check_verifier_prompt (verify.rs:1263) and READING_RULES all exist.
- Criterion 7: The budgets test prints the measured sets after the change. Compare with the base figures in docs/system-facts.md at the base commit (contract 326, reader 1997, producer 1257, verifier 1298, planner 1217, critic 974). The producer records both columns per set in one CHANGELOG bullet; docs/system-facts.md shows the after figures. The Contract and Reader lines state their extra headroom instead of a smaller budget. Run: `cargo test -p athena-cli --test budgets every_role_set_is_within_its_budget -- --nocapture`. Observes: The printed 'estimated tokens' line lists the sets; producer, verifier, planner and critic are under their base figures. 'git diff -- docs/system-facts.md CHANGELOG.md' shows the same after figures in both.
  Fails without the behavior: A set over its new budget makes check_budget(...).unwrap() panic; facts lines that differ from the computed values fail system_facts_budgets_match_computed.
  Test seam: check_budget over compose of the real constants and athena_cli::wiring::capabilities() exists (budgets.rs:339-393).
- Criterion 8: Read the CHANGELOG diff. Run: `git diff -- CHANGELOG.md`. Observes: One bullet under '## [Unreleased]' / '### Changed' names the rewritten prompts, the new reference doc and its Athena-repo-only limit, the new Producer and Verifier budgets with the 90% rule, the before and after tokens per set, the forced docs/system-facts.md edit, a forced run_pin_tests.rs edit if one was made, and a sentence that the figures in docs/design/context-engineer.md are historical. No other change to the file.

Planned files:

- docs/agent-reference.md (create)
- crates/athena-context/prompts/contract.md (modify)
- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/prompts/producer.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-cap-change/prompts/producer.md (modify)
- crates/athena-cap-change/prompts/verifier.md (modify)
- crates/athena-cap-study/prompts/producer.md (modify)
- crates/athena-cap-study/prompts/verifier.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/system-facts.md (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 7de0cd936b11 → candidate ff6ace71b721. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                 |  23 ++++++
 crates/athena-cap-change/prompts/producer.md |   2 +-
 crates/athena-cap-change/prompts/verifier.md |   4 +-
 crates/athena-cap-study/prompts/producer.md  |   2 +-
 crates/athena-cap-study/prompts/verifier.md  |   6 +-
 crates/athena-context/prompts/contract.md    |   4 +-
 crates/athena-context/prompts/critic.md      |  16 ++--
 crates/athena-context/prompts/planner.md     |  19 ++---
 crates/athena-context/prompts/producer.md    |  13 ++--
 crates/athena-context/prompts/reader.md      |  16 ++--
 crates/athena-context/prompts/verifier.md    |  14 ++--
 crates/athena-context/src/lib.rs             | 110 +++++++++++++++++++++++++--
 docs/agent-reference.md                      |  97 +++++++++++++++++++++++
 docs/system-facts.md                         |  12 +--
 14 files changed, 271 insertions(+), 67 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-035221-jj8g/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-035221-jj8g/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42563765403 for ff6ace71b721e733ef3ff3703d5f5c496f962ceb)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42563765403 for ff6ace71b721e733ef3ff3703d5f5c496f962ceb)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42563765403 for ff6ace71b721e733ef3ff3703d5f5c496f962ceb)
- ci-result: `not a check` passed (Check results for ff6ace71b721e733ef3ff3703d5f5c496f962ceb come from the CI run https://github.com/doodla/athena/actions/runs/42563765403 (579s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test --locked` passes, including the budget tests in crates/athena-context (every set within its budget) and any updated headroom-row tests in crates/athena-context/src/briefs.rs.; `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check` are clean.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-cap-change/prompts/producer.md, crates/athena-cap-change/prompts/verifier.md, crates/athena-cap-study/prompts/producer.md, crates/athena-cap-study/prompts/verifier.md, crates/athena-context/prompts/contract.md, crates/athena-context/prompts/critic.md, crates/athena-context/prompts/planner.md, crates/athena-context/prompts/producer.md, crates/athena-context/prompts/reader.md, crates/athena-context/prompts/verifier.md, crates/athena-context/src/lib.rs, docs/agent-reference.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
test disk::tests::a_build_slot_wait_is_an_environment_stop_that_uses_no_attempt ... ok
test disk::tests::a_disk_below_the_reserve_and_check_slots_fits_none_and_zero_cost_is_unbounded ... ok
test disk::tests::a_failure_of_the_work_is_never_an_environment_stop ... ok
test disk::tests::a_hold_marker_round_trips_and_clears ... ok
test disk::tests::a_probe_failure_hold_has_no_free_bytes_and_a_marker_names_the_amounts ... ok
test disk::tests::a_producer_stop_on_an_expired_login_is_resumable ... ok
test disk::tests::a_simulated_reading_is_used_only_when_it_is_a_whole_number ... ok
test disk::tests::a_slot_wait_marker_shares_the_file_and_each_reader_reads_only_its_own_kind ... ok
test disk::tests::a_start_waits_only_when_the_disk_fits_fewer_than_max_in_flight_and_others_fill_it ... ok
test disk::tests::check_timeout_reason_names_the_duration_and_the_limit ... ok
test disk::tests::df_output_is_read_as_available_kibibytes ... ok
test disk::tests::directory_bytes_count_files_below_it ... ok
test disk::tests::every_kind_of_environment_stop_is_classified_by_its_own_text ... ok
test disk::tests::interrupted_targets_are_reclaimed_but_running_jobs_keep_theirs ... ok
test design_surface::tests::the_inventory_is_read_from_the_commit_and_not_the_working_tree ... ok
test design_surface::tests::the_inventory_lists_commands_flags_keys_schemas_roles_and_crates ... ok
test disk::tests::measure_build_records_nothing_and_cleans_up_when_a_build_fails ... ok
test disk::tests::measure_build_refuses_installed_sccache_without_counts_and_a_repository_without_a_lock ... ok
test disk::tests::one_classifier_decides_environment_reasons ... ok
test disk::tests::only_an_environment_failure_or_an_interrupted_job_is_resumable ... ok
test disk::tests::sccache_totals_are_read_from_its_text_and_garbage_is_refused ... ok
test disk::tests::the_check_timeout_reason_is_an_environment_reason ... ok
test disk::tests::the_figures_are_those_of_the_latest_record_not_the_first ... ok
test disk::tests::the_limit_is_floor_of_free_less_reserve_and_check_slots_over_t_plus_s ... ok
test disk::tests::the_real_driver_detail_of_an_expired_login_is_an_api_error_401 ... ok
test disk::tests::the_real_reader_reports_some_space_for_a_real_directory ... ok
test disk::tests::the_sweep_removes_target_keeps_source_and_skips_locked_jobs ... ok
test disk::tests::the_verifier_environment_check_holds_the_job_for_any_job ... ok
test disk::tests::the_write_probe_fails_where_nothing_can_be_written ... ok
test disk::tests::the_write_probe_succeeds_in_a_normal_workspace_and_leaves_nothing_behind ... ok
test disk::tests::with_no_record_the_design_estimates_are_used ... ok
test evidence_labels::tests::a_blocking_verifier_finding_is_not_evidence_for_the_rejection_it_was_shown_to ... ok
test evidence_labels::tests::a_case_with_no_evidence_gets_no_label_and_is_counted_as_unlabeled ... ok
test code_health::tests::structural_pass_queues_nothing_for_a_file_with_an_open_request ... ok
test evidence_labels::tests::a_class_that_kept_recurring_or_stopped_after_a_prevention_landed_gives_labels ... ok
test evidence_labels::tests::a_retry_that_failed_for_another_class_or_is_still_open_gives_no_label ... ok
test evidence_labels::tests::a_person_s_label_is_skipped_and_a_rerun_writes_nothing_and_edits_nothing ... ok
test evidence_labels::tests::a_retry_that_followed_the_proposed_request_and_landed_gives_a_label_citing_it ... ok
test evidence_labels::tests::a_stopped_after_prevention_label_is_not_rewritten_when_more_jobs_close ... ok
…
   Doc-tests athena_git

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_score

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_testkit

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-context --lib`
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context frames_name_the_reference_doc_as_optional_and_it_exists`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-cap-change/prompts/producer.md, crates/athena-cap-change/prompts/verifier.md, crates/athena-cap-study/prompts/producer.md, crates/athena-cap-study/prompts/verifier.md, crates/athena-context/prompts/contract.md, crates/athena-context/prompts/critic.md, crates/athena-context/prompts/planner.md, crates/athena-context/prompts/producer.md, crates/athena-context/prompts/reader.md, crates/athena-context/prompts/verifier.md, crates/athena-context/src/lib.rs, docs/agent-reference.md, docs/system-facts.md.
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
   Compiling schemars v1.2.2
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 16.74s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_context-6da838b4f8c2a29c)

running 1 test
test tests::frames_name_the_reference_doc_as_optional_and_it_exists ... FAILED

failures:

---- tests::frames_name_the_reference_doc_as_optional_and_it_exists stdout ----

thread 'tests::frames_name_the_reference_doc_as_optional_and_it_exists' (27776) panicked at crates/athena-context/src/lib.rs:607:13:
assertion `left == right` failed: the reader set names the optional reference doc once, conditionally
  left: 0
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::frames_name_the_reference_doc_as_optional_and_it_exists

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 127 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-context --lib`
error: 1 target failed:
    `-p athena-context --lib`
```
- fails-without-6: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 6 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-cap-change/prompts/producer.md, crates/athena-cap-change/prompts/verifier.md, crates/athena-cap-study/prompts/producer.md, crates/athena-cap-study/prompts/verifier.md, crates/athena-context/prompts/contract.md, crates/athena-context/prompts/critic.md, crates/athena-context/prompts/planner.md, crates/athena-context/prompts/producer.md, crates/athena-context/prompts/reader.md, crates/athena-context/prompts/verifier.md, crates/athena-context/src/lib.rs, docs/agent-reference.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
test briefs::tests::the_send_back_section_stays_within_its_cap_with_a_very_long_list ... ok
test briefs::tests::with_no_qualifying_class_there_is_no_class_heading ... ok
test outputs::tests::the_auditor_schema_requires_a_summary_and_findings_with_evidence ... ok
test outputs::tests::the_critic_verdict_is_exactly_accept_revise_or_reject ... ok
test outputs::tests::schemas_require_every_field_and_forbid_others ... ok
test outputs::tests::outputs_round_trip ... ok
test outputs::tests::the_plan_and_critic_outputs_reject_an_extra_property_at_the_top_and_in_a_nested_item ... ok
test outputs::tests::the_audience_schema_requires_a_rating_and_findings_with_sources ... ok
test outputs::tests::the_plan_and_critic_schemas_forbid_extra_properties_at_every_level ... ok
test outputs::tests::the_reading_output_parses_an_urgent_reason_and_null_and_the_schema_states_the_rule ... ok
test outputs::tests::the_reading_schema_is_narrowed_to_the_closed_lists ... ok
test outputs::tests::the_plan_schema_requires_dependents_and_a_plan_without_the_key_reads_as_empty ... ok
test tests::an_over_budget_contract_fails ... ok
test tests::external_service_calls_need_a_named_state_and_a_wait ... ok
test tests::components_are_recorded_with_hashes_and_estimates ... ok
test tests::frames_name_the_reference_doc_as_optional_and_it_exists ... FAILED
test outputs::tests::the_steward_schema_requires_every_field_of_an_item ... ok
test tests::landing_rules_need_a_commit_shape_table_and_a_realistic_fixture ... ok
test tests::planner_and_critic_frames_are_short_and_fit_the_budgets ... FAILED
test tests::reader_checks_for_governance_files_and_producer_stops_on_unlisted_ones ... ok
test tests::producer_and_verifier_frames_leave_room_for_capability_sections ... ok
test tests::the_contract_is_within_its_budget ... ok
test tests::the_auditor_prompt_follows_the_contract_and_fits_its_budget ... ok
test tests::headroom_rows_name_shared_components_in_every_set_that_composes_them ... ok
test tests::the_contract_keeps_one_rule_per_line_as_measured ... ok
test tests::the_verifier_frame_limits_snapshot_runs_to_the_snapshot ... ok
test tests::the_audience_prompt_follows_the_contract_and_fits_its_budget ... ok
test tests::the_steward_prompt_follows_the_contract_and_fits_its_budget ... ok

failures:

---- tests::frames_name_the_reference_doc_as_optional_and_it_exists stdout ----

thread 'tests::frames_name_the_reference_doc_as_optional_and_it_exists' (28033) panicked at crates/athena-context/src/lib.rs:607:13:
assertion `left == right` failed: the reader set names the optional reference doc once, conditionally
  left: 0
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- tests::planner_and_critic_frames_are_short_and_fit_the_budgets stdout ----

thread 'tests::planner_and_critic_frames_are_short_and_fit_the_budgets' (28036) panicked at crates/athena-context/src/lib.rs:406:13:
frame.planner is short


failures:
    tests::frames_name_the_reference_doc_as_optional_and_it_exists
    tests::planner_and_critic_frames_are_short_and_fit_the_budgets

test result: FAILED. 126 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p athena-context --lib`
   Doc-tests athena_context

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-context --lib`
```
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test -p athena-cli --test budgets every_role_set_is_within_its_budget -- --nocapture`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
