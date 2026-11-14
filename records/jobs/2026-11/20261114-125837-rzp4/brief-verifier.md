# Job 20261114-125837-rzp4: verify "Retry 20261114-045625-aymy: run checks before verifier, send failures"

## Request (verbatim)

> try again on 20261114-045625-aymy

## Reading (accepted by the user)

Retry of job 20261114-045625-aymy, which died in planning at the 600s deadline with no code written. Same fix, planned as one narrow change: before the verifier, Athena runs the configured checks on the producer's tree, and a failure goes into the existing single send-back brief, so the producer sees it before the verdict.

Assumptions:
- Goal and audience: the goal is that a producer sees a failing recorded check (fmt, clippy, test) before the verifier does, so one repair pass replaces a whole-job rejection. The audience is Athena maintainers and the verifier. The typed request 'try again on 20261114-045625-aymy' takes precedence over the introspection's proposed planner-deadline request, which is a separate job and out of scope.
- Previous job 20261114-045625-aymy: it failed in planning (planner deadline of 600s) with 5 outcome reasons and no candidate, so no files are kept and all its deliverables are re-planned from the base. Its plan-critique findings are addressed as follows: (1) athena-cli tests whose test check is `test -f README.md && ...` and that delete README.md (retry.rs, introspection.rs, steward.rs, code_health_hooks.rs, lifecycle.rs:588) get one extra producer pass; the producer must run the full `cargo test -p athena-cli` and fix any assertion on producer counts or report order that breaks, without weakening it. (2) The in-loop check run sets GIT_INDEX_FILE to a temporary index built with `git add -A`, as Snapshot::work_tree_tree does, so `git ls-files` sees new files. (3) The producer.md change is net-zero bytes, and the producer gives the exact final text of each changed line.
- To keep the plan inside the 600s planner deadline the change is deliberately narrow: no new loop or limit, one in-loop check run reusing the existing check runner, one new appended decision, no edits to reader or verifier prompts.
- Code survey: read crates/athena-core/src/run/send_back.rs (send_back_text, send_back_text_scoped, unmet criteria from criteria_check at lines 11-50 and 92-166), crates/athena-core/src/run/producer.rs (send-back loop near lines 292, 573 and 614), crates/athena-core/src/run/mod.rs:1591 (PRODUCER_SEND_BACKS = 1) and crates/athena-core/src/run/tests.rs (fake producer tests). The entry point is the producer loop in producer.rs. The checks currently run in crates/athena-core/src/run/checks.rs only at verification. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched. The property is measured directly by driving the real producer loop with a fake producer, not through a stand-in.
- Field meaning: no new field, CLI argument, record key or config key is added. The stored invocation detail reuses the existing send-back problem text.
- Existing code: the change extends crates/athena-core/src/run/checks.rs (run_check, which clears the environment and sets ctx.env), crates/athena-core/src/run/send_back.rs and crates/athena-context/src/briefs.rs, and follows docs/decisions.md D115 (recorded checks on the same tree count as evidence).
- No refusal rule is added; the change only adds information to the existing send-back. Expected 0% of newly refused readings.
- Environment: none; local cargo only, with no CI workflow, push or records access.

## Acceptance criteria

1. [content] crates/athena-core/src/run/producer.rs runs the configured checks (via the existing runner in crates/athena-core/src/run/checks.rs) on the producer's tree before the verifier, with GIT_INDEX_FILE set to a temporary index of the work tree. A failure's command, exit status and output tail are added to the existing send-back problems and rendered by send_back_text in crates/athena-core/src/run/send_back.rs. The stored invocation detail names the check failure.
2. [content] The send-back stays bounded by PRODUCER_SEND_BACKS in crates/athena-core/src/run/mod.rs (diff shows no new loop and no changed limit). A check still failing after the send-back is passed to the verifier as flagged, not hidden.
3. [behavior] A new test in crates/athena-core/src/run/tests.rs drives the real producer loop with a fake producer whose tree fails a configured check on the first pass and passes after the send-back. Run `cargo test -p athena-core`: it passes, and the brief of the second producer call contains the failed command and its output tail. On the base tree it fails because no send-back happens.
4. [behavior] A second test in crates/athena-core/src/run/tests.rs shows that when all checks pass on the first pass there is exactly one producer call and no send-back, and a test shows that a file added by the producer and listed only by `git ls-files` is seen by the in-loop check. Run `cargo test -p athena-core`.
5. [behavior] The athena-cli tests that delete README.md and expect Rejected (retry.rs, introspection.rs, steward.rs, code_health_hooks.rs, lifecycle.rs:588) still pass with the extra producer pass. Run `cargo test -p athena-cli`.
6. [content] crates/athena-context/prompts/producer.md and the producer brief in crates/athena-context/src/briefs.rs tell the producer that Athena runs the configured checks before the verifier and sends a failure back once. A test in briefs.rs asserts that text. The producer.md edit is net-zero bytes and the prompt budgets in crates/athena-cli/tests/budgets.rs still pass.
7. [content] docs/decisions.md has a new appended entry with a Provenance bullet that names the root cause: Athena's recorded checks ran only at verification, so the producer could not see a failing gate before the verdict. It also names the temporary-index choice. No existing entry is edited. CHANGELOG.md records the change, and docs/decision-links.toml gets a link only if the entry amends or supersedes an earlier one.
8. [behavior] No reading is newly refused (expected 0% of newly refused readings). Run `cargo test` for the workspace.
9. [behavior] cargo fmt --check, cargo clippy --all-targets and cargo test pass on the final tree (Athena's recorded checks on the same tree, D115).

## Expected deliverables

- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/prompts/producer.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/decision-links.toml (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)

## Out of scope

- Adding a new repair loop or raising PRODUCER_SEND_BACKS
- Editing the reader, verifier prompts or the reader budget, or the stale 1,800 doc mentions from jobs 20261014-170031-2r9b, 20261014-172342-rqgb and 20261014-174937-hrzn (that work is dropped, not kept)
- Changing athena.toml, defaults.toml or workflow files
- Edits to records/ or past decision entries
- Adding any rule that refuses a reading
- Making the planner survive its deadline (the introspection's proposed request); it is a separate job

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The criteria check the producer seeing a failed check before the verdict, through a test on the real producer loop.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo fmt --check, clippy and test pass on the final tree.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix goes in the producer loop, where the missing information originates, not at one job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The behavior is enforced by code and tests, with no prose-only rule.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 4)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: The test of criterion 3 also loads `Records` and asserts that the first producer invocation's `detail` contains `self-check failed: check probe failed before the verifier`, and that every marker line ending `/work` has `index=unset` and `target=` ending `/target/produce`. Unit tests in send_back.rs (`mod check_failure_tests`): `the_send_back_names_the_failed_check` renders `append_check_failures` on a `## Sent back` brief with one `CheckFailure { name: "probe", command: "sh -c …", exit: "exit code 1", tail: "MISSING-FIXED-TXT" }` and asserts the command, `exit code 1`, the tail and that the leading sentence follows the heading; an empty slice returns the text unchanged. `the_exposed_index_lists_new_files_and_is_restored_on_drop` (tests.rs) builds a snapshot with `Snapshot::create`, adds `NEW.txt`, runs `git ls-files` (no NEW.txt), builds the `ExposedIndex`, runs `git ls-files` (lists NEW.txt), drops it, runs it again (no NEW.txt) and compares the index bytes with the original. `an_error_in_the_in_loop_run_is_a_skip_line_and_only_the_kill_switch_leaves` calls `Athena::settle_loop_checks` with an arbitrary error (skip line, `Ok`) and with the stop file set (`Err` naming the kill switch). Read the diff for the call in `producer_loop`, `GIT_INDEX_FILE` only in the `git add -A` of `ExposedIndex::new`, and the checks run through `SnapExec` and `run_check`: `git diff -- crates/athena-core/src/run`. Also run `cargo test -p athena-core the_send_back_names_the_failed_check`, `cargo test -p athena-core the_exposed_index_lists_new_files` and `cargo test -p athena-core an_error_in_the_in_loop_run`. Run: `cargo test -p athena-core a_failing_check_is_sent_back_before_the_verifier_and_the_fix_reaches_it`. Observes: The tests pass. The second producer brief has the check section with the command, the exit status and the tail; the first invocation's stored detail names the failed check; every in-loop marker line is in `target/produce` with no `GIT_INDEX_FILE`; the temporary index is installed for the checks and gone after; an error in the in-loop run is a skip line, the kill switch still ends the job.
  Fails without the behavior: With no in-loop run the producer count is 1, there is no second prompt and no stored `check probe failed before the verifier` text, so the test panics on `prompts[1]`. With no `ExposedIndex` the new-file test fails. With a leaked `GIT_INDEX_FILE` the marker line shows `index=` set and the test fails. If every error were propagated, or the kill switch swallowed, the settle test fails.
  Test seam: Production entry point: `Athena::go`, which reaches `producer_loop` (producer.rs:272). Fixture: `athena_with(&repo, &state, <marked NEEDS_FIXED_FILE>, plenty_of_space, script)` (tests.rs:191), with the real `run_check` running `sh -c` (no `check_seam` installed; the in-loop path never reads it anyway). The fake agent script wraps `editing(..)` (tests.rs:3066) and adds `FIXED.txt` only when the prompt has `## Sent back`, the device of `reconciling_on_send_back` (tests.rs:3120). The agents only edit files and return reports; the checks are real processes. The seam exists today; step 8 adds `NEEDS_FIXED_FILE`, `marked_check` and the wrapper. The unit tests call `append_check_failures`, `ExposedIndex::new` and `settle_loop_checks` directly (the last on an `Athena` from `athena_with` and a `Lines` UI) and need no seam; step 4 adds `settle_loop_checks`.
- Criterion 2: Same fixture with a producer that never creates FIXED.txt and a wrapper recording every verifier `user_prompt`. It asserts producer count exactly 2, verifier count exactly 1, state Rejected, a UI line starting `producer   report still incomplete after the send-back: check probe failed before the verifier`, the second producer invocation's stored detail naming the check, and a verifier prompt containing `producer-report-gaps` and `check probe failed`. `low_disk_while_the_producer_build_exists_skips_the_in_loop_checks` uses `crate::test_support::low_once_built` (test_support.rs:830) with `counting(counts, Some(root))` (the producer leaves `target/produce`, tests.rs:2984) and a failing check: the in-loop gate sees 3 GB, a `producer   checks not run before the verifier:` line appears, the producer count is 1, no `producer   sent back` line appears and verification judges the check (verifier count 1). `a_busy_build_slot_skips_the_in_loop_checks_without_waiting` (repo config `build_wait_s = 1` as at tests.rs:5293, a holder from `slot_queue(&athena).try_acquire()`, a failing check) asserts the line `producer   checks not run before the verifier: no build slot is free`, producer count 1, no `producer   sent back`, and that `go` returns the verification-time `SlotWait` error (the hold of 5292). Read `git diff -- crates/athena-core/src/run/mod.rs`: the `PRODUCER_SEND_BACKS: u32 = 1` line (mod.rs:1591) is unchanged apart from its doc comment and no loop is added. Run: `cargo test -p athena-core a_check_still_failing_after_the_send_back_is_flagged_to_the_verifier`. Observes: Exactly one send-back. The still-failing check appears in the incomplete-report line and reaches the verifier as a gap; verification then rejects on its own recorded check. A low disk or a busy slot skips the in-loop run, never waits and does not send the producer back.
  Fails without the behavior: Without the `report_problems` entry the incomplete-report line is missing and the verifier prompt has no gap. If the loop were unbounded the producer count would pass 2. If the disk gate were skipped (`gated = true`) the low-disk test would send the producer back for a space fault and see count 2. If the in-loop run linked the slot target, the marker's `target=` would end in `/target/verify`. If the in-loop run queued for the slot, the busy-slot test would wait out `build_wait_s` twice and not show the skip line before the hold.
  Test seam: Production entry point: `Athena::go` (producer_loop). Fixture: `athena_with` with the marked `NEEDS_FIXED_FILE` and a script built on `counting(counts, None)` and `editing(..)`; `Counts::all()` (tests.rs:2970) gives the producer and verifier counts. The low-disk test uses `low_once_built`, which exists today and reads `jobs/*/target/produce`; the busy-slot test uses `slot_queue(&athena)` (exists, tests.rs:5312) and `repo_with_config` (exists, tests.rs:5293). New consts and wrappers are added in step 8.
- Criterion 3: `repo()`, `athena_with(.., marked NEEDS_FIXED_FILE, ..)`, a script that records every producer prompt and adds FIXED.txt on a send-back; `athena.ask("describe the demo", &Quiet)` then `athena.go(job, AcceptedBy::User, &ui)`. It asserts state Landed; producer count 2 and verifier count 1; `prompts[0]` has no `## Sent back`; `prompts[1]` has `## Sent back`, `test -f FIXED.txt`, `exit code 1` and the tail `MISSING-FIXED-TXT`; a UI line starting `producer   sent back: check probe failed before the verifier`; and exactly two marker lines ending `/work` (the failing pass and the fixed pass), each with `target=…/target/produce`. `NEEDS_FIXED_FILE` passes on the base (README has no `A demo`), so the base run does not make it pre-existing. Run `cargo test -p athena-core` for the whole crate as well. Run: `cargo test -p athena-core a_failing_check_is_sent_back_before_the_verifier_and_the_fix_reaches_it`. Observes: The second producer call's brief names the failed command and its tail; after the fix the verifier runs once and the job lands.
  Fails without the behavior: On the base tree no send-back happens: the check first fails at verification, the job is Rejected, the producer count is 1 and `prompts[1]` is an index panic, so the test fails.
  Test seam: Production entry point: `Athena::go` (producer.rs:272 via run/mod.rs). Fake: the `FakeAgentDriver` script (tests.rs:191-220), injected through `athena_with`; it only edits files and returns the report, and does not run the checks. The checks are the real `run_check` on a real `sh`. The check command is configuration, not logic under test. The seam exists today; step 8 adds the const, `marked_check` and the wrapper script.
- Criterion 4: Tests, each also asserting from the marker that the in-loop run happened on the producer tree. (a) `all_checks_passing_means_one_producer_call_and_no_send_back`: marked `true`, `editing(counts, prompts, vec![])`; producer count 1, verifier count 1, one prompt without `## Sent back`, no UI line starting `producer   sent back`, exactly one `/work` marker line. (b) `a_file_the_producer_added_is_seen_by_the_in_loop_check`: check body `if test -f NEW.txt; then git ls-files --error-unmatch NEW.txt || exit 1; fi` (passes on the base, which has no NEW.txt); the producer adds NEW.txt through `extra`; producer count 1; the `/work` marker line has `new=yes`. (c) `a_check_that_runs_git_in_a_fresh_repository_passes_in_the_loop`: body runs `git init`, `git add` and `git -c user.name=t -c user.email=t commit` in `mktemp -d` when NEW.txt exists; producer count 1; the `/work` line has `index=unset`. (d) `a_check_failure_that_already_fails_on_the_base_is_not_sent_back`: check `sh -c "exit 1"` fails on the base and the candidate; producer count 1. (e) `a_tool_configuration_change_skips_the_in_loop_checks`: the producer adds `rustfmt.toml` through `extra`; check body `test ! -f rustfmt.toml`; producer count 1, no `producer   sent back` line, a `producer   checks not run before the verifier: the change holds check-tool configuration (rustfmt.toml)` line and no `/work` marker line. (f) the guard test of criterion 1. The CI-reuse tests that the real in-loop command could break are re-run with the edited `MARKS_BASELINE_OUTSIDE`: `cargo test -p athena-core baseline_`. Run `cargo test -p athena-core` for all of them. Run: `cargo test -p athena-core all_checks_passing_means_one_producer_call_and_no_send_back`. Observes: Passing checks give exactly one producer call and no send-back. A producer-added file is listed by `git ls-files` inside the in-loop check, the check env holds no `GIT_INDEX_FILE`, a fresh repo's git works inside the check, a base failure is not sent back, a tool-config change skips the run, and the `baseline_` tests still show `baseline_ran` false where the baseline was reused and true where it ran.
  Fails without the behavior: If the in-loop run always sent back, (a) would see producer count 2. Without the index swap, (b) fails in the loop (the snapshot's own index lacks NEW.txt) and the producer count is 2. If `GIT_INDEX_FILE` were exported, (c)'s marker shows `index=` set (and the nested commit fails on missing blobs). If the base filter were missing, (d) would see count 2. If the tool-config skip were missing, (e) would see count 2. If the in-loop run were skipped, every test's marker assertion for one `/work` line fails (which the bare count of 1 would not catch). Without the `*/baseline` guard on `MARKS_BASELINE_OUTSIDE`, the four `baseline_` tests that assert `!run.baseline_ran` (tests.rs:6945, 6988, 7041, 7131) fail because the in-loop run touches `<state>/jobs/baseline-ran`.
  Test seam: Production entry point: `Athena::go`. Fixture: `athena_with`, `editing`, `Counts`, `leaked_sh_check` (tests.rs:191, 3066, 2951, 4363); all exist today. Step 8 adds `marked_check` and the consts and edits `MARKS_BASELINE_OUTSIDE`. The CI-reuse harness `ci_reuse_run_with` (tests.rs:6739) already installs `check_seam`, `early_seam` and `ci_seam`; the in-loop run does not read the first.
- Criterion 5: Run the whole athena-cli test suite on the final tree. `test_config()` (common/mod.rs:62) has a `test` check of the form `test -f README.md && …`, so every scenario that deletes README.md fails it in the loop, gets one send-back, deletes the file again (athena-agent/src/fake.rs:107-109 ignores a missing file), goes to the verifier flagged, and is rejected as before. The producer fixes only an assertion on producer counts, call indexes or report order, in lifecycle.rs or retry.rs, without weakening it (per-call scripts at lifecycle.rs:656-678 and retry.rs:628, 1329). A break in another file (drain.rs, introspection.rs, steward.rs, code_health_hooks.rs) is reported and not edited (a file outside the deliverables fails the scope check). Also run `cargo test -p athena-cli --test lifecycle` and `cargo test -p athena-cli --test drain`. Run: `cargo test -p athena-cli`. Observes: All athena-cli tests pass, including those that delete README.md and expect Rejected (lifecycle.rs:588, 864; retry.rs:106, 238, 320, 989; introspection.rs:96; steward.rs:151, 188; code_health_hooks.rs:117; drain.rs:9631), the tool-config tests lifecycle.rs:905 and 947, and the disk tests drain.rs:3078 and the `gate_run` family.
  Fails without the behavior: If the in-loop run broke the cli fixtures, a test that asserts Rejected after a README deletion, or the stage order at lifecycle.rs:1589, would fail on a changed producer count, report order or retry brief. A missing tool-config skip fails `a_cargo_config_runner_in_the_candidate_is_set_aside_for_the_checks` (lifecycle.rs:947) with a Rejected state.
  Test seam: Production entry point: `Athena::go` through the cli `Fixture` with `test_config()` and the `Scenario` scripts (crates/athena-cli/tests/common/mod.rs). They exist today and are run, not changed, except for a count or order assertion that needs the extra producer call.
- Criterion 6: New test in briefs.rs, modelled on `producer_briefs_tell_the_producer_to_run_the_mirror_check` (briefs.rs:2293): it builds a `producer_brief` (briefs.rs:675) and asserts that the `## Checks before the verifier` section says Athena usually runs the configured checks (fmt, clippy and the test suite) after the producer reports and before the verifier, sends a failure back once with its command, exit status and output tail, flags one still failing to the verifier, and may skip the run (low disk, a busy build slot, an error, a tool-configuration change), and that it sits after `## Planned files left unchanged` and before `## Report`. producer.md is edited in step 7, net-zero: `wc -c crates/athena-context/prompts/producer.md` must still print 1102. Then run `cargo test -p athena-cli --test budgets` and `cargo test -p athena-context` (`frames_name_the_reference_doc_as_optional_and_it_exists`, lib.rs:628-645, needs the phrase `if docs/agent-reference.md exists in the working copy` once in the producer set). Run: `cargo test -p athena-context producer_briefs_say_athena_runs_the_checks_before_the_verifier`. Observes: The brief test passes. The budget tests pass with `budget producer: 1043 of 1200` unchanged. The producer.md byte count is 1102.
  Fails without the behavior: With no new section the brief test fails on the missing text. A producer.md that changed size would fail `system_facts_budgets_match_computed`; a line 12 that lost the pointer phrase would fail the reference-doc test.
  Test seam: Production entry point: `producer_brief` (briefs.rs:675) and the budget tests. No fake. Step 9 adds the section and the test.
- Criterion 7: Read `git diff -- docs/decisions.md CHANGELOG.md docs/decision-links.toml`. docs/decisions.md ends with a new `## D182 · …` entry (after D181, file end at line 5257). Its Provenance bullet names the root cause (Athena's recorded checks ran only at verification, so the producer could not see a failing gate before the verdict) and the temporary-index choice (built with `GIT_INDEX_FILE` for one `git add -A`, swapped into the snapshot's `.git/index`, restored on drop, never exported to the checks), and states that this departs from the literal wording of criterion 1. Its Decision says the in-loop run uses the producer's own build directory and a non-waiting slot, and why. The Tradeoff says the in-loop result is advisory (the producer tree keeps its own tool configuration and verification uses the base's; a tool-config change skips the run), that a busy slot or low disk skips it (with the labelled skip-rate estimate), and gives the labelled wall-time estimate (about one check run per pass, median 1107 s, n = 6, warm-baseline-build.md:42) and the two follow-ups (reuse of a passing in-loop result at verification; a queued slot wait). D115 and D156 stand. No earlier line changes. CHANGELOG.md gets one bullet at the top of `### Added` under `## [Unreleased]` (CHANGELOG.md:8, 16). docs/decision-links.toml is unchanged. Run: `cargo test -p athena-cli --test mirrors`. Observes: A pure append to decisions.md, one changelog bullet, an unchanged links file, and a passing mirrors test.
  Fails without the behavior: An entry that breaks the structure rules, or is not an append, fails the mirrors registry check on the finished diff (D162). A used `amends` or `supersedes` phrase would need a `[[link]]` and fail the decision-links backtest.
  Test seam: Production entry point: the mirror registry (`mirrors::check_all`, run by the `mirrors` test and by `producer_loop`'s `mirror_drift_in`). No fake. The seam exists today.
- Criterion 8: Run the full workspace tests. The plan adds or changes no refusal or matching rule (no parser, stem list, stop list or reading check), so there are no input forms to map and no corpus case is added. `crates/athena-core/tests/refusal_corpus.rs` runs as part of the workspace run; this plan cites no case ids and no result for it. Run: `cargo test`. Observes: The workspace passes. The share of newly refused readings stays at 0%, because no reading code changes.
  Fails without the behavior: If the change made any reading refuse, `refusal_corpus` or a reading test would fail.
  Test seam: Production entry point: the whole workspace test run. No fake.
- Criterion 9: Run Athena's three recorded checks on the final tree: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`. This is also what the in-loop run now does on the producer's tree before the verifier (D115). Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: All three exit 0 on the final tree.
  Fails without the behavior: A formatting, lint or test fault in the new code fails the matching check.
  Test seam: Production entry point: the three checks as written. No fake.

Planned files:

- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/prompts/producer.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/decision-links.toml (modify)
- crates/athena-cli/tests/budgets.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 036407fe95d3 → candidate 109c4716ce2f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              |   1 +
 crates/athena-cli/tests/retry.rs          |  10 +-
 crates/athena-context/prompts/producer.md |   4 +-
 crates/athena-context/src/briefs.rs       |  53 ++++
 crates/athena-core/src/run/checks.rs      | 219 ++++++++++++-
 crates/athena-core/src/run/mod.rs         |  17 +-
 crates/athena-core/src/run/producer.rs    |  34 +-
 crates/athena-core/src/run/send_back.rs   | 131 ++++++++
 crates/athena-core/src/run/tests.rs       | 494 +++++++++++++++++++++++++++++-
 docs/decisions.md                         |  62 ++++
 10 files changed, 1001 insertions(+), 24 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-125837-rzp4/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-125837-rzp4/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43607778455 for 109c4716ce2fb82d0b328b21e2985fdc26e05f50)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43607778455 for 109c4716ce2fb82d0b328b21e2985fdc26e05f50)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43607778455 for 109c4716ce2fb82d0b328b21e2985fdc26e05f50)
- ci-result: `not a check` passed (Check results for 109c4716ce2fb82d0b328b21e2985fdc26e05f50 come from the CI run https://github.com/doodla/athena/actions/runs/43607778455 (781s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: No reading is newly refused (expected 0% of newly refused readings). Run `cargo test` for the workspace.; cargo fmt --check, cargo clippy --all-targets and cargo test pass on the final tree (Athena's recorded checks on the same tree, D115).)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core a_failing_check_is_sent_back_before_the_verifier_and_the_fix_reaches_it`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/producer.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   --> crates/athena-core/src/run/send_back.rs:332:19
    |
332 |         let out = append_check_failures(send_back_text("brief", &[], &[]), &[f]);
    |                   ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0599]: no method named `settle_loop_checks` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:4832:14
     |
4831 | /         athena
4832 | |             .settle_loop_checks(&ui, Err(anyhow::anyhow!("boom")))
     | |             -^^^^^^^^^^^^^^^^^^ method not found in `Athena`
     | |_____________|
     |
     |
    ::: crates/athena-core/src/lib.rs:229:1
     |
 229 |   pub struct Athena {
     |   ----------------- method `settle_loop_checks` not found for this struct

error[E0599]: no method named `settle_loop_checks` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:4834:16
     |
4834 |         athena.settle_loop_checks(&ui, Ok(())).unwrap();
     |                ^^^^^^^^^^^^^^^^^^ method not found in `Athena`
     |
    ::: crates/athena-core/src/lib.rs:229:1
     |
 229 | pub struct Athena {
     | ----------------- method `settle_loop_checks` not found for this struct

error[E0599]: no method named `settle_loop_checks` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:4846:14
     |
4845 |           let error = athena
     |  _____________________-
4846 | |             .settle_loop_checks(&ui, Err(anyhow::anyhow!("boom")))
     | |             -^^^^^^^^^^^^^^^^^^ method not found in `Athena`
     | |_____________|
     |
     |
    ::: crates/athena-core/src/lib.rs:229:1
     |
 229 |   pub struct Athena {
     |   ----------------- method `settle_loop_checks` not found for this struct

error[E0599]: no method named `settle_loop_checks` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:4849:24
     |
4849 |         assert!(athena.settle_loop_checks(&ui, Ok(())).is_err());
     |                        ^^^^^^^^^^^^^^^^^^ method not found in `Athena`
     |
    ::: crates/athena-core/src/lib.rs:229:1
     |
 229 | pub struct Athena {
     | ----------------- method `settle_loop_checks` not found for this struct

Some errors have detailed explanations: E0422, E0425, E0433, E0599.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 11 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core a_check_still_failing_after_the_send_back_is_flagged_to_the_verifier`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/producer.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
error[E0425]: cannot find function `append_check_failures` in this scope
   --> crates/athena-core/src/run/send_back.rs:332:19
    |
332 |         let out = append_check_failures(send_back_text("brief", &[], &[]), &[f]);
    |                   ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0599]: no method named `settle_loop_checks` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:4832:14
     |
4831 | /         athena
4832 | |             .settle_loop_checks(&ui, Err(anyhow::anyhow!("boom")))
     | |             -^^^^^^^^^^^^^^^^^^ method not found in `Athena`
     | |_____________|
     |
     |
    ::: crates/athena-core/src/lib.rs:229:1
     |
 229 |   pub struct Athena {
     |   ----------------- method `settle_loop_checks` not found for this struct

error[E0599]: no method named `settle_loop_checks` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:4834:16
     |
4834 |         athena.settle_loop_checks(&ui, Ok(())).unwrap();
     |                ^^^^^^^^^^^^^^^^^^ method not found in `Athena`
     |
    ::: crates/athena-core/src/lib.rs:229:1
     |
 229 | pub struct Athena {
     | ----------------- method `settle_loop_checks` not found for this struct

error[E0599]: no method named `settle_loop_checks` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:4846:14
     |
4845 |           let error = athena
     |  _____________________-
4846 | |             .settle_loop_checks(&ui, Err(anyhow::anyhow!("boom")))
     | |             -^^^^^^^^^^^^^^^^^^ method not found in `Athena`
     | |_____________|
     |
     |
    ::: crates/athena-core/src/lib.rs:229:1
     |
 229 |   pub struct Athena {
     |   ----------------- method `settle_loop_checks` not found for this struct

error[E0599]: no method named `settle_loop_checks` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:4849:24
     |
4849 |         assert!(athena.settle_loop_checks(&ui, Ok(())).is_err());
     |                        ^^^^^^^^^^^^^^^^^^ method not found in `Athena`
     |
    ::: crates/athena-core/src/lib.rs:229:1
     |
 229 | pub struct Athena {
     | ----------------- method `settle_loop_checks` not found for this struct

Some errors have detailed explanations: E0422, E0425, E0433, E0599.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 11 previous errors
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core all_checks_passing_means_one_producer_call_and_no_send_back`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/producer.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/send_back.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
error[E0425]: cannot find function `append_check_failures` in this scope
   --> crates/athena-core/src/run/send_back.rs:332:19
    |
332 |         let out = append_check_failures(send_back_text("brief", &[], &[]), &[f]);
    |                   ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0599]: no method named `settle_loop_checks` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:4832:14
     |
4831 | /         athena
4832 | |             .settle_loop_checks(&ui, Err(anyhow::anyhow!("boom")))
     | |             -^^^^^^^^^^^^^^^^^^ method not found in `Athena`
     | |_____________|
     |
     |
    ::: crates/athena-core/src/lib.rs:229:1
     |
 229 |   pub struct Athena {
     |   ----------------- method `settle_loop_checks` not found for this struct

error[E0599]: no method named `settle_loop_checks` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:4834:16
     |
4834 |         athena.settle_loop_checks(&ui, Ok(())).unwrap();
     |                ^^^^^^^^^^^^^^^^^^ method not found in `Athena`
     |
    ::: crates/athena-core/src/lib.rs:229:1
     |
 229 | pub struct Athena {
     | ----------------- method `settle_loop_checks` not found for this struct

error[E0599]: no method named `settle_loop_checks` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:4846:14
     |
4845 |           let error = athena
     |  _____________________-
4846 | |             .settle_loop_checks(&ui, Err(anyhow::anyhow!("boom")))
     | |             -^^^^^^^^^^^^^^^^^^ method not found in `Athena`
     | |_____________|
     |
     |
    ::: crates/athena-core/src/lib.rs:229:1
     |
 229 |   pub struct Athena {
     |   ----------------- method `settle_loop_checks` not found for this struct

error[E0599]: no method named `settle_loop_checks` found for struct `Athena` in the current scope
    --> crates/athena-core/src/run/tests.rs:4849:24
     |
4849 |         assert!(athena.settle_loop_checks(&ui, Ok(())).is_err());
     |                        ^^^^^^^^^^^^^^^^^^ method not found in `Athena`
     |
    ::: crates/athena-core/src/lib.rs:229:1
     |
 229 | pub struct Athena {
     | ----------------- method `settle_loop_checks` not found for this struct

Some errors have detailed explanations: E0422, E0425, E0433, E0599.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 11 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-cli`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-context producer_briefs_say_athena_runs_the_checks_before_the_verifier`). Result: not run: only the first few distinct commands are run.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test -p athena-cli --test mirrors`). Result: not run: only the first few distinct commands are run.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo test`). Result: not run: only the first few distinct commands are run.
- fails-without-9: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 9 (`cargo clippy --all-targets --locked -- -D warnings`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
