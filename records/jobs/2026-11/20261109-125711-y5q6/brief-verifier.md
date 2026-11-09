# Job 20261109-125711-y5q6: verify "Make the reader state which agent roles a "Athena knows X" request"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: "Athena knows X" is ambiguous between documentation for readers and behavior built into the agents.
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: knowledge-request-read-as-docs-for-humans-not-agent-behavior (20261109-091813-hhqq)
> Cause: "Athena knows X" is ambiguous between documentation for readers and behavior built into the agents. Nothing makes the reader ask which one is meant. It also does not check which existing rules would stop the agents acting on the stated fact...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261109-091753-tt8c (job 20261109-080749-p0fs); prevention it proposed: Add a reader default: when a request says Athena or its agents should know, understand or act on something, the reading must state which roles and prompts must change, and must name any governing rule (such as the ladder rule in roles-and-l...
> Introspection 20261109-125651-wz8f (job 20261109-080220-zkhy); prevention it proposed: When a request changes what a role should decide or do, the reading step must list the rules that bound that role's actions (ladder rungs, budgets, role limits) and put any blocking rule into scope as a deliverable. Where a rule is owner-go...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause (class knowledge-request-read-as-docs-for-humans-not-agent-behavior, 20261109-091813-hhqq): the reader prompt never makes the reader decide whether "Athena should know/act on X" means docs or agent behavior, nor list the rules that bound the affected role. Fix it in crates/athena-context/prompts/reader.md, within the 71-token reader budget, with a test on the composed prompt. The audience is the reader role, plus the verifiers who judge its readings. Retries 20261109-080749-p0fs and 20261109-080220-zkhy.

Assumptions:
- Code survey: I read crates/athena-context/prompts/reader.md, where the reader rules live. crates/athena-core/src/reading.rs and crates/athena-cli/tests/budgets.rs compose it as READER_FRAME. Prompt edits are mirror-checked by crates/athena-core/src/mirrors.rs. The change adds no record or data file, so check_record_file, AnyRecord and the round-trip test are not involved. The budget is measured directly by composing the real prompt.
- Term ladder: the ordered rungs of change size or authority that cap what a role may decide, per docs/design/roles-and-ladder.md; a rung limit is one of the rules the reader must list.
- Fixing at the reader prompt matches both introspections' proposals. The steward-side failures in job 20261109-080220-zkhy and the README failure in 20261109-080749-p0fs are instances; their code and doc candidates are not kept.
- The prompt edit may need a docs/decisions.md entry, so it is listed with moderate probability.
- Placing the rule in defaults.toml was rejected: it is a governance file, and the reader budget counts defaults too.

## Acceptance criteria

1. [content] crates/athena-context/prompts/reader.md gains one rule in 'Reading the request': when a request says Athena or its agents should know, understand or act on something, the reading must state whether it means docs or agent behavior, name the roles and prompts that change, list the rules that bound those roles (ladder rungs, budgets, role limits, admission rules) and list as deliverables any blocking rule that is not owner-only. Verify with git diff.
2. [behavior] A new unit test in crates/athena-context/src/lib.rs asserts the composed reader prompt contains the docs-versus-agent-behavior rule and the rule-listing requirement. It fails on the pre-change reader.md and passes after. Run: cargo test -p athena-context.
3. [behavior] The budget check in crates/athena-cli/tests/budgets.rs still passes: the reader set stays at or under 2000 tokens (71 left), so the producer must trim existing reader.md words if needed. Run: cargo test -p athena-cli --test budgets.
4. [content] The change description (CHANGELOG.md [Unreleased]) names the root cause, the ambiguity between docs for readers and built-in agent behavior, and says the fix is in the reader prompt, not in jobs 20261109-080749-p0fs or 20261109-080220-zkhy. Their candidates' README.md, CHANGELOG.md and steward changes are dropped, so the verifier can check the prior findings do not recur in the diff.
5. [behavior] Mirror and policy checks pass for the prompt edit: cargo test (whole workspace), including athena-core mirrors tests and the xtask policy test, plus cargo fmt --check and cargo clippy, all pass.
6. [content] No new refusal rule is added; the change is prose-only, marked as such in CHANGELOG.md. It does not edit defaults.toml, athena.toml or workflows, which Athena changes only via its proposal path, so no past readings are newly refused (expected 0% of newly refused readings).

## Expected deliverables

- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- README.md, docs/architecture.md and the steward prompt or code (crates/athena-core/src/steward.rs, crates/athena-data/src/roles.rs, crates/athena-data/src/records.rs): the earlier candidates are dropped
- defaults.toml, athena.toml, workflows, BRIEF.md and records/
- A code check that refuses readings
- Changing role budgets

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is in the reader prompt that produced both misreadings, not in either job's output.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A test on the composed reader prompt, plus fmt, clippy and the whole test suite.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The test checks the composed prompt text the reader actually receives.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One rule in one prompt file, within the token budget.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff of reader.md. Run: `git diff -- crates/athena-context/prompts/reader.md`. Observes: One new bullet on its own line under '## Reading the request'. It says to state docs or agent behavior, name the roles and prompts that change, list bounding rules (ladder rungs, budgets, role limits, admission), and make blocking non-owner-only rules deliverables. The only removed text is the filler 'Users write lazily.' folded into the first bullet; no rule is lost.
- Criterion 2: Run the new unit test `the_reader_must_say_docs_or_agent_behavior_and_list_bounding_rules` in the tests module of crates/athena-context/src/lib.rs. Entry point: `compose(&[CONTRACT.into(), READER_FRAME.into()])`, the same call `components_are_recorded_with_hashes_and_estimates` uses. It slices the text between '## Reading the request' and '## Forecasts' and asserts each element phrase and the clause 'blocking non-owner-only rules' is inside the slice. No guards or preconditions. The FRAME_ANCHORS test (lib.rs:643) also checks the new anchor. Run: `cargo test -p athena-context`. Observes: The new test and the anchor test pass with the change. On the base reader.md none of the phrases exist, so the test fails.
  Fails without the behavior: The pre-change reader.md has no rule about docs versus agent behavior, so the phrase assertions fail. A bullet that dropped 'blocking' or 'admission' also fails.
  Test seam: `READER_FRAME` (lib.rs:38) and `compose` (lib.rs:142) exist today. The real include_str! prompt is composed with no double or injection. Plan step 2 adds the test and the anchor entry.
- Criterion 3: Run the budgets test. `computed_rows` composes contract, READER_FRAME, defaults and catalog; `check_budget` rejects a set over 2000 tokens; `system_facts_budgets_match_computed` compares the facts line with the computed size. Run: `cargo test -p athena-cli --test budgets`. Observes: The reader set is at most 2000 tokens and the updated docs/system-facts.md line equals the computed size. If the first edit overshoots, the failure names the computed value, and the producer shortens the new bullet and reruns.
  Fails without the behavior: Without the system-facts update the drift test fails, and a rule over 71 tokens of headroom fails `check_budget`.
  Test seam: `computed_rows` and `system_facts_budgets_match_computed` in crates/athena-cli/tests/budgets.rs exist today and compose the real prompts with no double. No new seam.
- Criterion 4: Read the diff. Run: `git diff -- CHANGELOG.md docs/decisions.md`. Observes: The [Unreleased] 'Changed' bullet names the root cause ("Athena knows X" is ambiguous between documentation for readers and behavior built into the agents; nothing made the reader ask which, or list the rules that bound the role). It says the fix is in the reader prompt, not in jobs 20261109-080749-p0fs or 20261109-080220-zkhy, marks the change prose-only, and gives the reader size before and after. D170 follows D169 with a Provenance bullet. `git diff --stat` shows no README.md or steward file.
- Criterion 5: Run the whole workspace tests, including athena-core mirrors tests (`prompt structure`, `decision entries`, crates/athena-core/src/mirrors.rs, `check_all` at line 1117), crates/athena-cli/tests/mirrors.rs and the xtask policy test. Then run `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings`. Run: `cargo test --locked`. Observes: All pass. The new bullet starts its own line with '- ', the D170 entry is appended with a `## D170 · title` heading and a Provenance bullet, and the boundary scan finds no banned words or email-like text in the added text.
  Fails without the behavior: A glued bullet, an edit to an earlier decision line, or a missing Provenance bullet fails the mirror tests.
  Test seam: The mirror registry and the git-diff-based tests exist today and need no new fixture.
- Criterion 6: Read the changed-file list. Run: `git diff --stat`. Observes: Changed files: reader.md, athena-context/src/lib.rs, CHANGELOG.md, docs/decisions.md and docs/system-facts.md (one number) only. defaults.toml, athena.toml, .github/workflows/ and every parser or reading-check source are unchanged, and the CHANGELOG bullet says prose-only. No refusal rule changes, so no past reading is newly refused (0 expected).

Planned files:

- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/src/lib.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/system-facts.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 3d3fb7afb93c → candidate 13a89809eb6b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            | 10 ++++++++++
 crates/athena-context/prompts/reader.md |  3 ++-
 crates/athena-context/src/lib.rs        | 25 +++++++++++++++++++++++++
 docs/decisions.md                       | 30 ++++++++++++++++++++++++++++++
 docs/system-facts.md                    |  2 +-
 5 files changed, 68 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-125711-y5q6/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-125711-y5q6/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42626835753 for 13a89809eb6b1f8c677266857c64c60ed7a068f7)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42626835753 for 13a89809eb6b1f8c677266857c64c60ed7a068f7)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42626835753 for 13a89809eb6b1f8c677266857c64c60ed7a068f7)
- ci-result: `not a check` passed (Check results for 13a89809eb6b1f8c677266857c64c60ed7a068f7 come from the CI run https://github.com/doodla/athena/actions/runs/42626835753 (566s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Mirror and policy checks pass for the prompt edit: cargo test (whole workspace), including athena-core mirrors tests and the xtask policy test, plus cargo fmt --check and cargo clippy, all pass.)
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/reader.md, docs/decisions.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
test outputs::tests::the_audience_schema_requires_a_rating_and_findings_with_sources ... ok
test outputs::tests::the_auditor_schema_requires_a_summary_and_findings_with_evidence ... ok
test outputs::tests::outputs_round_trip ... ok
test outputs::tests::schemas_require_every_field_and_forbid_others ... ok
test outputs::tests::the_plan_and_critic_outputs_reject_an_extra_property_at_the_top_and_in_a_nested_item ... ok
test outputs::tests::the_critic_verdict_is_exactly_accept_revise_or_reject ... ok
test outputs::tests::the_reading_output_parses_an_urgent_reason_and_null_and_the_schema_states_the_rule ... ok
test briefs::tests::with_no_qualifying_class_there_is_no_class_heading ... ok
test outputs::tests::the_plan_schema_requires_dependents_and_a_plan_without_the_key_reads_as_empty ... ok
test outputs::tests::the_reading_schema_is_narrowed_to_the_closed_lists ... ok
test outputs::tests::the_plan_and_critic_schemas_forbid_extra_properties_at_every_level ... ok
test tests::an_over_budget_contract_fails ... ok
test tests::external_service_calls_need_a_named_state_and_a_wait ... ok
test tests::components_are_recorded_with_hashes_and_estimates ... ok
test outputs::tests::the_steward_schema_requires_every_field_of_an_item ... ok
test outputs::tests::the_steward_schema_says_a_brief_is_for_an_environment_or_token_need_not_a_rung ... ok
test tests::frames_name_the_reference_doc_as_optional_and_it_exists ... FAILED
test tests::headroom_rows_name_shared_components_in_every_set_that_composes_them ... ok
test tests::reader_checks_for_governance_files_and_producer_stops_on_unlisted_ones ... ok
test tests::landing_rules_need_a_commit_shape_table_and_a_realistic_fixture ... ok
test tests::producer_and_verifier_frames_leave_room_for_capability_sections ... ok
test tests::planner_and_critic_frames_are_short_and_fit_the_budgets ... ok
test tests::the_contract_is_within_its_budget ... ok
test tests::the_contract_keeps_one_rule_per_line_as_measured ... ok
test tests::the_audience_prompt_follows_the_contract_and_fits_its_budget ... ok
test tests::the_auditor_prompt_follows_the_contract_and_fits_its_budget ... ok
test tests::the_verifier_frame_limits_snapshot_runs_to_the_snapshot ... ok
test tests::the_real_steward_prompt_fits_its_budget ... ok
test tests::the_reader_must_say_docs_or_agent_behavior_and_list_bounding_rules ... FAILED
test tests::the_steward_prompt_follows_the_contract_and_fits_its_budget ... ok

failures:

---- tests::frames_name_the_reference_doc_as_optional_and_it_exists stdout ----

thread 'tests::frames_name_the_reference_doc_as_optional_and_it_exists' (18010) panicked at crates/athena-context/src/lib.rs:649:13:
reader frame lacks: docs or agent behavior
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- tests::the_reader_must_say_docs_or_agent_behavior_and_list_bounding_rules stdout ----

thread 'tests::the_reader_must_say_docs_or_agent_behavior_and_list_bounding_rules' (18020) panicked at crates/athena-context/src/lib.rs:687:13:
the reading rules lack: docs or agent behavior


failures:
    tests::frames_name_the_reference_doc_as_optional_and_it_exists
    tests::the_reader_must_say_docs_or_agent_behavior_and_list_bounding_rules

test result: FAILED. 129 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p athena-context --lib`
   Doc-tests athena_context

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-context --lib`
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --test budgets`, exit code 0. Result: passed without the fix: the revert did not make the test fail.
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/reader.md, docs/decisions.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
   Compiling toml v0.9.12+spec-1.1.0
   Compiling serde_derive_internals v0.30.0
   Compiling fastrand v2.5.0
   Compiling once_cell v1.21.4
   Compiling regex-syntax v0.8.11
   Compiling tempfile v3.27.0
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling thiserror v2.0.21
   Compiling schemars_derive v1.2.2
   Compiling anyhow v1.0.104
   Compiling hashbrown v0.17.1
   Compiling regex-automata v0.4.18
   Compiling equivalent v1.0.2
   Compiling indexmap v2.14.2
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling regex v1.13.1
   Compiling utf8parse v0.2.2
   Compiling anstyle-parse v1.0.0
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling colorchoice v1.0.5
   Compiling anstyle-query v1.1.5
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle v1.0.14
   Compiling anstream v1.0.0
   Compiling clap_lex v1.1.1
   Compiling heck v0.5.0
   Compiling strsim v0.11.1
   Compiling clap_derive v4.6.7
   Compiling clap_builder v4.6.7
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling clap v4.6.7
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 37.00s
     Running tests/budgets.rs (<job>/target/verify/debug/deps/budgets-3c161170e6563139)

running 7 tests
test an_over_budget_role_set_fails ... ok
test every_role_set_is_within_its_budget ... ok
test system_facts_budgets_match_computed ... ok
test system_facts_crate_drift_is_rejected ... ok
test system_facts_drift_is_rejected ... ok
test system_facts_crate_lines_match_manifests ... ok
test the_reader_headroom_rows_equal_the_budget_figures_and_name_shared_files ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/reader.md, docs/decisions.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text

running 2 tests
test status_snapshot_names_a_missing_directory_and_refuses_a_job_argument ... ok
test status_snapshot_reads_records_and_history_from_the_snapshot_with_no_records_in_the_current_directory ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.31s

     Running tests/steward.rs (<job>/target/verify/debug/deps/steward-6e27dbfadc238955)

running 12 tests
test a_missing_token_reaches_the_owner_with_its_brief_and_a_plain_gap_does_not ... ok
test a_plain_github_blind_spot_recorded_first_does_not_hide_a_later_token_need ... ok
test an_unreadable_source_becomes_a_blind_spot_work_item_and_calls_no_model ... ok
test a_second_run_with_the_same_keys_queues_nothing_again ... ok
test a_source_athena_cannot_write_becomes_a_blind_spot_the_model_is_told_about ... ok
test empty_output_makes_no_commit_and_its_invocation_waits_for_the_next_one ... ok
test github_being_unavailable_is_one_blind_spot_whatever_the_error_says ... ok
test a_nested_run_makes_no_model_call ... ok
test items_that_fail_validation_are_reported_and_nothing_is_queued_for_them ... ok
test scripted_output_with_a_real_gap_commits_the_work_items_and_a_queued_request ... ok
test the_fake_steward_reads_every_named_source_in_a_workspace_without_private_files ... ok
test the_fifth_finished_task_starts_the_steward_and_the_fourth_does_not ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 23.92s

     Running tests/trailer_stop.rs (<job>/target/verify/debug/deps/trailer_stop-96dfd117ad765ec7)

running 4 tests
test retry_landing_without_a_cutover_job_still_writes_trailers ... ok
test retry_landing_after_c0_searches_no_trailer_and_writes_none ... ok
test post_c0_job_is_read_by_the_trace_and_the_auditor ... ok
test post_c0_landing_through_land_pr ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 19.93s

     Running tests/version.rs (<job>/target/verify/debug/deps/version-692b629f5c0131c4)

running 4 tests
test clean_build_names_its_commit ... ok
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
    `-p athena-cli --test mirrors`
    `-p athena-context --lib`
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
