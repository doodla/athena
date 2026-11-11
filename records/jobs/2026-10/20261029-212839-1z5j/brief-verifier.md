# Job 20261029-212839-1z5j: verify "Check plan claims against code before the critic; send back open test"

## Request (verbatim)

> Change the planner's brief, and add a deterministic plan check where one is possible, so that before a plan goes to the critic every claim about existing behavior is checked against the code. That covers what a named function does on the failure or retry path, which existing tests outside the deliverables the change will break, and which environment, paths and tools the sandboxed checks provide. Any open question that the code can settle must be settled, with the file and line cited. Add a test with a fake planner. It should show that a plan which leaves existing-test breakage as an open question is sent back without using up a critic round.
> 
> Acceptance:
> - The change is made as described: Plans rely on wrong claims about existing functions, tests or the check environment; the critic catches them only in the last round and the job fails before any code
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: records/failure-classes/2026-10/20261024-115027-s1ww.json

## Reading (accepted by the user)

Tighten the planner brief so each claim about existing behavior (a named function's failure and retry path, existing tests the change breaks outside its deliverables, the environment, paths and tools the verifier's checks provide) is checked in code with file and line. Extend the deterministic plan check in admission.rs so a plan that leaves existing-test breakage as an open question is sent back mechanically, and add a fake-planner test showing this uses no critic round. Audience: planners and the critic, who now catch these errors only in the last round.

Assumptions:
- Goal: planners stop relying on unverified claims about existing code; audience is the planner and critic roles and whoever reads failed planning jobs.
- Code survey: read crates/athena-core/src/admission.rs (plan_findings at line 175, open_questions_findings at 524, the User:/Host: prefix rule at 549), crates/athena-core/src/run/planning.rs (lines 211-247: major findings return RoundEnd::Mechanical before critic_round, bounded by MAX_MECHANICAL_PASSES), crates/athena-context/prompts/planner.md, crates/athena-agent/src/fake.rs (fake planner) and crates/athena-cli/tests/job_plan.rs (mechanical-revision tests at lines 821 and 872). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. The property is measured directly: the critic call count and round number after a plan with such an open question.
- Existing code: extends crates/athena-core/src/admission.rs open_questions_findings and plan_findings, crates/athena-context/prompts/planner.md, and the mechanical-revision path in crates/athena-core/src/run/planning.rs; none is created by this job.
- The refusal rule extension is narrow: it refuses a plan whose open question (even one prefixed User:/Host:) asks whether existing tests break or what a named function does, since those are searchable. Expected 3% of newly refused readings (it applies to plans, not readings, so the replay of readings is unaffected); the higher refusal rate is intended because those plans spent critic rounds on impact the planner can find by searching.
- Prompt budget: planner prompt has 95 tokens left, so the planner.md edit adds one compact bullet (under about 90 tokens) and replaces wording where possible. Only the planner role gets new brief content; the critic is not changed.
- The deterministic check can only cover the open-question form and the existing 'producer to check' deferral; whether a function claim is true cannot be checked by code, so that part stays in the brief and the critic.

## Acceptance criteria

1. [content] crates/athena-context/prompts/planner.md has a bullet requiring each claim about existing behavior (a named function's failure and retry path, existing tests outside the deliverables that the change breaks, the environment, paths and tools the verifier's checks provide) to be checked in code and cited with file and line, and open questions the code can settle to be settled; the planner prompt stays within its token budget (git diff shows the added lines).
2. [content] crates/athena-core/src/admission.rs plan_findings or open_questions_findings gives a major finding for an open question about whether existing tests break, even with a User: or Host: prefix, and its doc comment lists the rule (git diff shows it).
3. [behavior] A test in crates/athena-cli/tests/job_plan.rs using the fake planner and fake critic shows a first plan leaving existing-test breakage as an open question is sent back with no critic call and round 1 not used, then the revised plan reaches the critic as round 1; run `cargo test -p athena-cli --test job_plan`.
4. [behavior] Unit tests in admission.rs cover the new rule: a test-breakage question is refused and a genuine User: decision question still passes; run `cargo test -p athena-core admission`.
5. [behavior] No unrelated behavior changes: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass, including the existing mechanical-revision tests.
6. [content] CHANGELOG.md has an entry for the new plan check and brief change, and no files other than the listed deliverables change (git diff --stat).

## Expected deliverables

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)

## Out of scope

- Changing the critic prompt, the reader or the producer
- Changing MAX_MECHANICAL_PASSES or critic round limits
- Editing records/ files, including records/failure-classes/2026-10/20261024-115027-s1ww.json
- Proving a claim about a function true by static analysis

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests in admission.rs and a fake-planner test; cargo test, clippy and fmt --check must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criterion runs the fake-planner test and checks that no critic call was made.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the brief and the plan check.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The test-breakage rule is enforced by plan_findings, not only prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 4)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff, then run the frame and budget tests that pin planner.md: `cargo test -p athena-context` and `cargo test -p athena-cli --test budgets`. `ls -l crates/athena-context/prompts/planner.md` must show 2383 bytes (net change 0). Run: `git diff -- crates/athena-context/prompts/planner.md`. Observes: Line 5 reads '- Read the code the request touches first; cite file:line for each claim about it (failure and retry paths, tests it breaks outside the deliverables, the checks' environment, paths, tools); settle what code can; name only what you saw.' Line 9 is merged into line 8 and the 'Never write "producer to check"' sentence stays. Lines 7, 12 and 16 are unchanged. The frame tests (lib.rs:398-408 under 800 tokens, :481-545 wording, :563-650 anchors, briefs.rs:1582-1634) and `system_facts_budgets_match_computed` (budgets.rs:124) pass with docs/system-facts.md unchanged.
- Criterion 2: Read the diff of admission.rs. Run: `git diff -- crates/athena-core/src/admission.rs`. Observes: `open_questions_findings` has a third branch returning a major finding at 'open questions' for a test-breakage or named-function failure/retry question, with or without a User:/Host: prefix. The prefix is stripped before the exemptions are checked. A decision form or a host-environment word in the body exempts the question. The doc comments of `open_questions_findings` and `plan_findings` list the rule, the exemptions, the known misses (rewordings, unbackticked names, 'should existing tests ...', a question naming a platform or toolchain) and the refusal-rate estimate, labelled an estimate and not a measurement.
- Criterion 3: New test `athena_sends_a_plan_whose_open_question_asks_about_existing_tests_back_with_no_critic_call`, next to job_plan.rs:851. It builds `Scenario::default()` with `plans = [asks, good_plan()]` and `critiques = [critique_of("accept", &[], None)]`, where `asks` is `good_plan()` with `open_questions = ["User: do any existing tests break when the prompt grows?"]`. It runs `ask` then `go` as at job_plan.rs:864-870. Run: `cargo test -p athena-cli --test job_plan`. Observes: The job lands. The planner is called 2 times and the critic 1 time. `briefs.len() == 2`, and `briefs[1]` contains 'settled by searching the code', the question and 'Revision'. `critique_at(&fx, &dir, 1).invocation_seq.is_some()`, so critique 1 is the critic's own and the mechanical pass did not use round 1. The first plan never reached the critic.
  Fails without the behavior: Without the new branch the User:-prefixed question passes (has_addressee is true and says_code_answers is false, admission.rs:528-535). The critic accepts the first plan, the job lands after 1 planner call, the planner-count assertion (2) fails and `briefs` has one entry.
  Test seam: Production entry point: `Athena::go`, which reaches `plan_findings` at planning.rs:217 and the `RoundEnd::Mechanical` return before `critic_round` (:249). Fake: the scripted `Scenario` in crates/athena-cli/tests/common/mod.rs, injected by `fx.athena(scenario.script())` with `Fixture::with_config(&planned_config())`. Exists (read at job_plan.rs:791-883). Only the planner's output is scripted; `plan_findings` is real.
- Criterion 4: Add admission.rs unit tests next to :1206 and :1223, using the existing helper `open_question_findings_for`, which calls the real `plan_findings`. Test 1 `a_user_or_host_question_about_existing_tests_or_a_retry_path_is_refused`: 'User: do any existing tests break when the prompt grows?', 'Host: which other tests fail if the budget line changes?', '**Host:** which other tests fail if the budget line changes?', '**User:** Does the existing test suite break?', 'user: what does `land` do on the retry path?', 'User: what happens on failure in `run_job`?', 'User: do tests outside the deliverables fail?', 'User: do existing tests that expect the old wording break?', 'User: may existing tests fail after this?'. Each yields exactly 1 major Athena finding at 'open questions' whose summary contains the question and 'settled by searching the code'. Test 2 `a_decision_or_host_fact_that_mentions_tests_or_failure_is_not_refused` expects no findings for: 'User: Should the CHANGELOG entry go under Added or Changed?', 'User: which failing test should block landing?', 'User: should any test that fails on Windows be marked ignored?', 'Host: does any test fail on the production host's older toolchain?', 'Host: do existing tests fail on the production host's older toolchain?', 'User: does it matter whether the CLI exits 1 or 2 on error?', 'User: should other tests that fail be skipped?', 'User: is it acceptable if existing tests break?', 'User: what does the user expect `athena land` to print on failure?', 'User: may the change edit tests in `crates/athena-cli/tests/job_plan.rs`?'. Test 3: an unprefixed 'Do existing tests break?' still yields only the 'no addressee' finding. Run: `cargo test -p athena-core admission`. Observes: All new and existing admission unit tests pass. The Host: refuse case and the Host: toolchain accept case together show the prefix does not exempt a question.
  Fails without the behavior: Without the third branch, test 1's questions yield no finding and its one-finding assertion fails. If the host word were matched on the prefix, the Host: refuse cases would fail. If 'expect' or 'may' exempted anywhere, the last two test 1 questions would fail. Without the decision, host-word and named-function narrowing, test 2's counterexamples are refused and it fails.
  Test seam: Production entry point: `plan_findings` through the existing helper `open_question_findings_for` in the admission.rs test module (about :1190). Inputs are plain strings; no fake. Exists.
- Criterion 5: Run the full suite, plus `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test -p athena-core --test refusal_corpus`, `cargo test -p athena-cli --test budgets` and `cargo test -p athena-context`. Run: `cargo test --locked`. Observes: Everything passes: the existing mechanical-revision tests (job_plan.rs:791 and :851, run/tests.rs:1450), `open_question_cases_get_the_expected_outcome` with the new cases, and `system_facts_budgets_match_computed` against the unchanged facts line. I read the case ids and have not seen a run.
  Fails without the behavior: An over-broad rule would refuse an accepted case (user-decision, user-scope-question-citing-a-path, host-fact-marked-in-bold, or the admission.rs:1225-1229 list) and fail the corpus or admission tests. A planner.md size change that moves the token estimate fails `system_facts_budgets_match_computed`; a frame over 800 tokens fails lib.rs:408.
  Test seam: All existing test targets; nothing is injected. The existing suite is the seam.
- Criterion 6: Read the stat and the CHANGELOG diff. Run: `git diff --stat`. Observes: CHANGELOG.md `## [Unreleased]` / `### Changed` has an entry for the new plan check (an open question about existing-test breakage or a named function's failure/retry path is sent back without a critic call, even with a User:/Host: prefix) and for the planner brief change. It says fake.rs was deliberately left unchanged and that the short form 'name only what you saw' was kept. The files changed are planner.md, admission.rs, job_plan.rs, refusal_corpus.rs, planning.rs (comment only) and CHANGELOG.md. No records/ file, fake.rs or docs/system-facts.md changes.

Planned files:

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-agent/src/fake.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 9ad246a753b8 → candidate 6656a653341d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  10 ++
 crates/athena-cli/tests/job_plan.rs        |  39 ++++++
 crates/athena-context/prompts/planner.md   |   7 +-
 crates/athena-core/src/admission.rs        | 184 ++++++++++++++++++++++++++++-
 crates/athena-core/src/run/planning.rs     |   5 +-
 crates/athena-core/tests/refusal_corpus.rs |  93 +++++++++++++++
 6 files changed, 327 insertions(+), 11 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-212839-1z5j/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-212839-1z5j/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42962704084 for 6656a653341de5fef95917b1b3e3e490a3369248)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42962704084 for 6656a653341de5fef95917b1b3e3e490a3369248)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42962704084 for 6656a653341de5fef95917b1b3e3e490a3369248)
- ci-result: `not a check` passed (Check results for 6656a653341de5fef95917b1b3e3e490a3369248 come from the CI run https://github.com/doodla/athena/actions/runs/42962704084 (765s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: No unrelated behavior changes: `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass, including the existing mechanical-revision tests.)
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --test job_plan`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/planner.md, crates/athena-core/src/admission.rs, crates/athena-core/src/run/planning.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 31.27s
     Running tests/job_plan.rs (<job>/target/verify/debug/deps/job_plan-9cf7fa351a29623a)

running 30 tests
test a_cap_between_the_producer_alone_and_planning_plus_the_producer_stops_the_job_before_the_producer ... ok
test a_file_outside_the_deliverables_is_a_minor_finding_that_does_not_send_the_plan_back ... ok
test a_plan_citing_a_command_outside_the_verifiers_grant_gets_an_athena_critique_and_no_critic_call ... ok
test a_plan_that_edits_an_unlisted_governance_file_gets_an_athena_critique_and_no_critic_call ... ok
test a_retry_of_a_job_stopped_in_planning_carries_the_last_critiques_findings ... ok
test a_reject_a_misread_reading_or_a_revise_in_the_last_round_stops_the_job ... ok
test a_hard_reading_with_planning_enabled_for_every_job_and_one_with_planning_off_follow_the_mode ... ok
test a_plan_that_lists_a_directory_gets_a_major_athena_finding_and_the_job_goes_to_the_next_round ... ok
test a_second_round_accept_shows_as_plan_accepted_r2_and_is_not_a_first_round_accept ... ok
test a_send_back_keeps_the_plan_section_and_its_hash_matches_the_plan_record ... ok
test after_a_revise_the_second_planner_prompt_quotes_the_first_finding_and_the_job_lands ... ok
test an_enabled_job_runs_the_planner_and_critic_before_the_producer_and_commits_their_records ... ok
test an_easy_job_with_a_fails_without_criterion_is_planned_in_hard_mode_and_the_verifier_sees_the_record ... ok
test a_short_job_is_skipped_with_its_reason_and_the_next_item_runs ... ok
test an_unplanned_jobs_status_text_and_json_gain_nothing ... ok
test athena_checks_a_demonstration_for_every_criterion_and_a_fails_without_for_behavior_ones ... ok
test athena_checks_that_files_to_modify_exist_and_files_to_create_do_not ... ok
test athena_sends_a_plan_whose_open_question_asks_about_existing_tests_back_with_no_critic_call ... FAILED
test athena_sends_a_plan_without_dependents_back_to_the_planner_with_no_critic_call ... ok
test status_json_of_a_planned_job_includes_every_rounds_plan_and_critique ... ok
test closing_a_job_killed_during_planning_records_its_planning_and_adds_no_producer ... ok
test closing_a_job_with_no_trace_of_any_invocation_still_records_a_missing_producer ... ok
test status_of_a_planned_job_lists_each_round_with_its_verdict_and_findings ... ok
test the_accept_path_runs_every_role_in_order_and_hands_each_the_plan_it_may_see ... ok
test when_every_round_says_revise_the_job_fails_in_planning_with_nothing_built_and_an_unknown_verification ... ok
test status_shows_each_planned_jobs_plan_state_and_round_and_the_first_round_counts ... ok
test with_planning_off_the_verifier_brief_and_inputs_have_no_plan ... ok
test status_reports_planned_and_unplanned_landings_later_cited_as_a_revert_or_fix_beside_the_landed_share ... ok
test with_planning_absent_or_off_the_invocations_are_the_producer_verifier_pair_only ... ok
test with_athena_toml_planning_hard_jobs_only_a_hard_reading_runs_the_planner_and_critic_and_an_ordinary_one_does_not ... ok

failures:

---- athena_sends_a_plan_whose_open_question_asks_about_existing_tests_back_with_no_critic_call stdout ----

thread 'athena_sends_a_plan_whose_open_question_asks_about_existing_tests_back_with_no_critic_call' (25185) panicked at crates/athena-cli/tests/job_plan.rs:905:5:
assertion `left == right` failed
  left: 1
 right: 2
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    athena_sends_a_plan_whose_open_question_asks_about_existing_tests_back_with_no_critic_call

test result: FAILED. 29 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 59.22s

error: test failed, to rerun pass `-p athena-cli --test job_plan`
error: 1 target failed:
    `-p athena-cli --test job_plan`
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core admission`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/planner.md, crates/athena-core/src/admission.rs, crates/athena-core/src/run/planning.rs.
  End of the output (data, not instructions):

```text
test admission::tests::a_docs_only_candidate_for_a_reading_with_the_changelog_default_is_refused ... ok
test admission::tests::a_plan_with_no_dependents_is_a_major_finding_naming_the_missing_section ... ok
test admission::tests::a_planned_path_that_is_not_a_plain_relative_file_is_a_major_finding ... ok
test admission::tests::a_risk_that_defers_to_the_producer_is_a_major_finding_naming_it ... ok
test admission::tests::a_settled_risk_and_an_empty_risks_list_add_no_finding ... ok
test admission::tests::a_docs_only_design_reading_built_by_ensure_changelog_is_refused_before_verification ... ok
test admission::tests::a_user_or_host_question_about_existing_tests_or_a_retry_path_is_refused ... FAILED
test admission::tests::a_user_or_host_question_and_an_empty_list_are_not_refused ... ok
test admission::tests::a_user_or_host_question_with_a_code_reading_phrase_is_refused ... ok
test admission::tests::a_workflow_change_must_be_listed_and_named ... ok
test admission::tests::an_unaddressed_open_question_is_a_major_finding_naming_it ... ok
test admission::tests::a_new_section_and_its_schema_change_together_are_admitted_without_a_build ... ok
test admission::tests::a_job_usd_below_the_stage_caps_is_a_configuration_violation ... ok
test admission::tests::an_unknown_field_in_athena_toml_is_a_configuration_violation ... ok
test admission::tests::an_unprefixed_question_about_existing_tests_keeps_only_the_no_addressee_finding ... ok
test admission::tests::ci_reuse_replay_the_seen_commands_filter_turns_up_in_a_search_and_must_be_named ... ok
test admission::tests::governance_files_are_recognized ... ok
test admission::tests::lints_tables_are_compared_alone ... ok
test admission::tests::a_study_reading_or_a_reading_without_the_default_needs_no_changelog ... ok
test admission::tests::an_unknown_field_in_defaults_toml_is_a_configuration_violation ... ok
test admission::tests::an_unknown_section_without_a_schema_change_is_still_refused ... ok
test gate_counts::tests::an_admission_rule_that_ended_a_rejected_or_failed_job_is_a_gate ... ok
test admission::tests::the_changelog_is_demanded_even_when_the_reading_leaves_it_out_of_scope ... ok
test admission::tests::parsing_configuration_and_untouched_configuration_add_no_violation ... ok
test admission::tests::plan_admission_and_the_gate_agree_on_when_the_schema_judges_a_config_edit ... ok
test run::tests::produce_and_verify_order_tests::a_request_with_a_banned_word_is_refused_before_admission_and_a_clean_one_is_admitted ... ok
test run_pin_tests::admission_refuses_a_job_short_of_the_whole_planning_reserve ... ok
test run::tests::produce_and_verify_order_tests::ci_branch_is_pushed_after_admission_and_deleted_at_close ... ok

failures:

---- admission::tests::a_user_or_host_question_about_existing_tests_or_a_retry_path_is_refused stdout ----

thread 'admission::tests::a_user_or_host_question_about_existing_tests_or_a_retry_path_is_refused' (18757) panicked at crates/athena-core/src/admission.rs:1237:13:
assertion `left == right` failed: User: do any existing tests break when the prompt grows?: []
  left: 0
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    admission::tests::a_user_or_host_question_about_existing_tests_or_a_retry_path_is_refused

test result: FAILED. 37 passed; 1 failed; 0 ignored; 0 measured; 1288 filtered out; finished in 6.07s

error: test failed, to rerun pass `-p athena-core --lib`
     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-a2cb0acbd401486e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 23 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/planner.md, crates/athena-core/src/admission.rs, crates/athena-core/src/run/planning.rs.
  End of the output (data, not instructions):

```text
test a_duplicate_term_is_flagged ... ok
test a_sense_that_does_not_pass_both_ways_is_not_a_checked_claim ... ok
test a_sense_marker_equal_to_the_term_name_or_empty_is_flagged ... ok
test a_sense_in_a_file_the_term_does_not_scan_is_flagged ... ok
test a_source_that_does_not_exist_is_flagged ... ok
test a_sound_file_has_no_problems ... ok
test a_sound_fixture_has_no_claim_problems ... ok
test a_term_with_empty_sources_is_flagged ... ok
test a_term_without_a_checked_claim_is_flagged ... ok
test a_stale_excuse_is_flagged ... ok
test a_term_without_a_sources_field_is_flagged ... ok
test a_well_formed_excuse_covers_an_identifying_line_that_is_not_a_sense ... ok
test an_excuse_covering_more_than_one_file_is_flagged ... ok
test an_excuse_marker_that_is_only_the_term_name_is_flagged ... ok
test an_excuse_without_a_marker_is_flagged ... ok
test an_excuse_without_a_reason_is_flagged ... ok
test an_overbroad_excuse_is_flagged ... ok
test both_parsers_read_the_same_terms ... ok
test athena_needs_its_role_and_its_program_sense ... ok
test hermes_with_only_the_tool_sense_is_flagged_until_both_senses_are_listed ... ok
test identifying_lines_are_bullets_headings_and_table_rows ... ok
test markers_match_whole_words_only ... ok
test the_ontology_defines_the_seven_seeded_terms_with_their_meaning ... ok
test the_ontology_file_is_sound ... ok
test the_real_file_claims_are_checked_against_the_sources ... ok
test a_sense_whose_marker_is_gone_from_the_source_is_flagged_for_each_seed_term ... ok
test altered_definition_is_flagged_for_each_seed_term ... ok

test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s

     Running tests/proposals.rs (<job>/target/verify/debug/deps/proposals-c73dbd2d25c34aa3)

running 5 tests
test checking_a_candidate_applies_the_proposal_check_to_defaults_at_head ... ok
test the_proposal_commands_exist ... ok
test status_lists_open_proposals_with_their_sources_and_omits_decided_ones ... ok
test a_rejected_class_is_proposed_again_only_after_enough_newer_jobs ... ok
test accepting_submits_a_job_that_lists_defaults_toml_and_edits_nothing ... ok

…
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_score

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_testkit

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 3 targets failed:
    `-p athena-cli --test job_plan`
    `-p athena-core --lib`
    `-p athena-core --test refusal_corpus`
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
