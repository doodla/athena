# Job 20261104-171135-911m: verify "Lift agent caps in athena.toml with job_usd covering their sum"

## Request (verbatim)

> try again on 20261104-140950-gzp9

## Reading (accepted by the user)

Retry of job 20261104-140950-gzp9: apply the nine cap values to athena.toml, raise [budget] job_usd to 50.00 so it covers the $47.00 required by the new caps (2 + 5×(3+3) + 10 + 5), and update the config.rs tests that pin the old values. Audience: the Athena owner and verifier.

Assumptions:
- Job 20261104-140950-gzp9 failed: its athena.toml-only edit (kept here as a deliverable) left job_usd 40.00 below the 47.00 required, and four config.rs tests pin old values. This retry fixes both: raises job_usd and updates those tests. The earlier out_of_scope on config.rs is dropped because the tests read athena.toml; the schema code itself stays unchanged.
- Code survey: crates/athena-data/src/config.rs. Config::budget_required = reader_usd + max_rounds*(planner_usd+critic_usd) + producer_usd + verifier_usd; check_budget refuses job_usd below it. Tests at ~lines 855-936 and ~1163 pin 0.75, max_rounds 3, job_usd 12.0 and the replace string 'job_usd = 12.00'. Other crates' tests (athena-cli/tests, athena-core) may read athena.toml and are checked by running the workspace tests.
- job_usd = 50.00 is chosen (47.00 needed plus headroom); the user's other eight values are as originally stated. CHANGELOG.md was a prior deliverable but is not required; listed with low probability.
- The change adds no new refusal rule; it only changes values, so no newly-refused-readings percentage applies.

## Acceptance criteria

1. [content] athena.toml [budget] reads reader_usd 2.00, producer_usd 10.00, verifier_usd 5.00, job_usd 50.00 (at least 47.00), producer_timeout_s 5400, verifier_timeout_s 3600; [job_plan] planner_usd 3.00, critic_usd 3.00, max_rounds 5; git diff of athena.toml shows only those value substitutions, no key added or removed, reader_timeout_s, check_timeout_s, planner_timeout_s, critic_timeout_s unchanged.
2. [behavior] Run `cargo test --locked -p athena-data`; it passes, including the repository-config budget tests updated to the new values and a test that Config::parse(REPO_CONFIG).check_budget() is ok.
3. [behavior] Run `cargo test --locked --workspace`; it passes, so no other test pinning old athena.toml values remains broken; and `cargo clippy --locked --workspace --all-targets` is clean.
4. [content] Changes to crates/athena-data/src/config.rs are confined to the test module (git diff shows no change outside #[cfg(test)]); the schema and check logic are untouched.
5. [content] job_usd (50.00) is no less than reader_usd + max_rounds*(planner_usd+critic_usd) + producer_usd + verifier_usd = 47.00, so admission's budget check passes on the new config.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-core/src/admission.rs (modify)

## Out of scope

- Changing config schema or non-test logic in crates/athena-data/src/config.rs
- Changing keys not listed (reader_timeout_s, check_timeout_s, planner_timeout_s, critic_timeout_s)
- Editing defaults.toml or workflows
- Adding a new command to compute minimum job_usd

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check admission budget passes and tests run, not just that values were edited.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the job_usd-below-sum invariant and the pinned tests rather than only the nine values.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run cargo test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single config change plus the tests that pin it.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff of athena.toml. Exactly nine value-only changed lines and nothing added or removed. The new config.rs test the_repository_budget_caps_cover_their_sum asserts each value by parsing REPO_CONFIG, including timeouts 5400/3600 and the unchanged reader_timeout_s 300, check_timeout_s 1800, planner/critic_timeout_s 600. Run: `git diff -- athena.toml`. Observes: Only the nine listed keys change value; the new test passes, which also fails if reader_timeout_s, check_timeout_s, planner_timeout_s or critic_timeout_s moved.
- Criterion 2: Run the athena-data tests. The updated tests parse the real athena.toml through the production entry points Config::parse and Config::check_budget, via the existing REPO_CONFIG = include_str!(athena.toml) constant (config.rs:838). The budget-threshold test has with_job_usd replace 'job_usd = 50.00' with 46.99 (required $47.00, $0.01 short), 47.00 (accepted) and 16.99/17.00 (planning off); the more-rounds case needs 53.00. Run: `cargo test --locked -p athena-data`. Observes: All tests pass, including Config::parse(REPO_CONFIG).check_budget() is_ok, the 47.00 requirement and the refusal one cent below it.
  Fails without the behavior: With athena.toml at the new values but config.rs tests unchanged, the_repository_config_plans_hard_jobs_only_with_the_designs_caps (asserts 0.75 / 3) and the_repository_budget_covers_two_reading_and_two_planning_rounds (asserts job_usd 12.0) fail. With job_usd left at 40.00, the new test and job_usd_below_the_reserve_is_refused_by_the_budget_check (which asserts check_budget ok on REPO_CONFIG, config.rs:909) fail.
  Test seam: Config::parse and Config::check_budget with the REPO_CONFIG fixture; exists today (config.rs:838, 819). The new test is added in step 3.
- Criterion 3: Run the whole workspace tests, then `cargo clippy --locked --workspace --all-targets`. Fakes are the existing scripted agent fakes (Scenario, costing(), CountingPinner). Each updated test drives production code: Athena::ask/go, drain::drain/plan, admission checks and budget_reserve. They read athena.toml by include_str!; edited numbers appear only as expected values. Run: `cargo test --locked --workspace`. Observes: Every test passes; clippy exits 0 with no warnings.
  Fails without the behavior: Without the test edits, these fail on the new athena.toml: admission a_job_usd_below_the_stage_caps_is_a_configuration_violation (replace of 12.00 hits nothing, so assert_ne fails), reading shipped_budget_covers_the_reserve (expects 10.0), run_pin_tests admission_refuses_a_job_short_of_the_whole_planning_reserve, run/tests round_one_critic_check_follows_the_shared_reserve and dispatch_holds_a_job_that_cannot_afford_every_planning_round, lifecycle a_job_over_budget_is_refused_before_its_next_invocation, job_plan tests, and drain tests using real_steps.
  Test seam: Existing: Athena::ask / go via Fixture::with_config and with_job_usd (tests/common/mod.rs:96), run/tests.rs repo_with_planning, drain::drain with RealSteps. No new seam.
- Criterion 4: Read the diff. Every hunk lies after the `#[cfg(test)]` line at config.rs:833; no hunk touches the lines before it (schema, budget_required, check_budget). Run: `git diff -- crates/athena-data/src/config.rs`. Observes: No hunk above the cfg(test) module; the schema and check logic are unchanged.
- Criterion 5: The new test parses REPO_CONFIG and asserts budget_required() == 47.00 within 1e-9 and equals reader + max_rounds*(planner+critic) + producer + verifier, that job_usd >= that, and that check_budget() is ok. The same invariant is exercised on the athena-core side by reading.rs shipped_budget_covers_the_reserve through budget_reserve(ReserveStage::Admission) and admission's configuration check (admission.rs:616) on BASE_CONFIG. Run: `cargo test --locked -p athena-data the_repository_budget_caps_cover_their_sum`. Observes: The sum is 47.00, job_usd is 50.00, check_budget is ok; admission passes on the shipped file.
  Fails without the behavior: If job_usd were below 47.00 (for example the failed first attempt's 40.00), check_budget returns BudgetBelowReserve and the test fails; the admission test in athena-core also sees the shipped file refused.
  Test seam: Config::budget_required / check_budget with the REPO_CONFIG fixture; exists. The new test is added in step 3.

Planned files:

- athena.toml (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 3aaf21d4e8ba → candidate 2db54282584f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  8 +++++
 athena.toml                             | 18 +++++-----
 crates/athena-cli/tests/drain.rs        | 44 +++++++++++------------
 crates/athena-cli/tests/job_plan.rs     | 16 ++++-----
 crates/athena-cli/tests/lifecycle.rs    | 23 ++++++++----
 crates/athena-core/src/admission.rs     |  4 +--
 crates/athena-core/src/reading.rs       | 17 +++++----
 crates/athena-core/src/run/tests.rs     | 35 +++++++++---------
 crates/athena-core/src/run_pin_tests.rs | 23 ++++++------
 crates/athena-data/src/config.rs        | 64 +++++++++++++++++++++++----------
 10 files changed, 150 insertions(+), 102 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41882292905 for 2db54282584fca9e0f48a2e54ab0c855693e5367)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41882292905 for 2db54282584fca9e0f48a2e54ab0c855693e5367)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41882292905 for 2db54282584fca9e0f48a2e54ab0c855693e5367)
- ci-result: `not a check` passed (Check results for 2db54282584fca9e0f48a2e54ab0c855693e5367 come from the CI run https://github.com/doodla/athena/actions/runs/41882292905 (690s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --locked --workspace`; it passes, so no other test pinning old athena.toml values remains broken; and `cargo clippy --locked --workspace --all-targets` is clean.)
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-data`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, athena.toml.
  End of the output (data, not instructions):

```text
ootstrap (361a786) and fails the records-based landing check.\n# ee4b0dd1aa6fcd81289dc988d0f7bd38fd7a40ef is the squash of PR #8, which carried several jobs' landings in one\n# commit; the landing check refuses that shape. Excused by the user's choice of option (a) in D66: this one\n# commit's content skips the invariant, and its jobs' own records still get the per-record checks.\n# f2a8453380196b95f2e2cc629ffa969af57e6c85 is the squash of PR #9. Most of that PR's jobs predate\n# athena.candidate/2, so the chain landing rule (D69) cannot link them (D70).\n# 5a734782e76daac9b12bd8833c97a73927c8cf94 is the squash of PR #14. It changes records/ but names no job,\n# so the per-job records check cannot tie its content to a job (D76).\n# 39962ed02fec9da2d87cd4a3e287121ed229cd0f is the squash of PR #130. It carried two jobs' landings:\n# 20261010-154004-60r4 (its PR #129 was closed unmerged) and 20261017-121143-sg0j. The chain between them is\n# broken, so the history check finds that neither landing is valid in that commit. Excused as ee4b0dd (D66)\n# and f2a8453 (D70) were: this one commit's content skips the invariant, and each job's own records still get\n# the per-record checks.\n# 02837e0 is the squash of PR #149 (job 20261018-145008-k7fq), merged by hand around athena land. Its\n# content carries two hand changes beside the job's verified content, so it is not a valid landing. Excused\n# with Sri's approval of 2026-10-23 (BRIEF.md addendum of that date) as 39962ed was. The commit that adds this\n# entry reverts the job's own content, to re-land through Athena; the two hand changes stay.\nexempt = [\"49c6ac1524677b8091081115fb5d410ed6d2b2ea\", \"64ecdc9aa4fd24cbe4d4884c864d7e9041585c3d\", \"ee4b0dd1aa6fcd81289dc988d0f7bd38fd7a40ef\", \"f2a8453380196b95f2e2cc629ffa969af57e6c85\", \"5a734782e76daac9b12bd8833c97a73927c8cf94\", \"39962ed02fec9da2d87cd4a3e287121ed229cd0f\", \"02837e0d47783e9a4af0fd139d5b11adbcbb217b\"]\n\n[records]\nmax_file_bytes = 1048576\n\n[personal_data]\n# Email addresses allowed in committed content. An entry ending in \"@\" allows that local part (\"noreply@\");\n# one starting with \"@\" allows that domain; one starting with \".\" allows every domain under that suffix.\n# These are no-reply addresses, the documentation domains, and the reserved top-level domains (RFC 2606),\n# none of which can belong to a person.\nemail_allow = [\"noreply@\", \"@example.com\", \"@example.org\", \"@example.net\", \".invalid\", \".test\", \".example\"]\n\n# Athena asks the host to merge a pull request once its checks are green, so `athena land --pr` finishes\n# the landing without a manual merge step. Red or pending checks never trigger a merge.\n[landing]\nmerge_via_rest = true\n\n# How many jobs `athena drain` runs at once: the independent items of one plan group (no dependency, disjoint\n# touched paths). 1 is one job at a time, as before; the schema allows 1 to 8. Left at 1 here, so adding the\n# section changes nothing until the user raises it.\n[parallel]\nmax_concurrency = 1\n\n# How many queued items `athena plan` and `athena drain` treat as urgent and as high. An item over a cap is\n# demoted one tier, in memory, for that plan only: no request or priority record is rewritten, and an item whose\n# priority a person set with a priority record is never demoted. These are the defaults, written out.\n[priority_caps]\nurgent = 3\nhigh = 10\n\n# Job planning (docs/design/job-planning.md): a planner and a critic run before the producer of a change job.\n# The caps are the design's. `mode = \"hard\"` plans only a job whose reading is marked hard, the marking that also\n# escalates the producer: a job that adds or changes a rule that can refuse or hold work, or changes landing, the\n# history check, records, the run pipeline or the drain. Every other job runs the producer and verifier only.\n# `mode = \"all\"` plans every change job. `enabled = false` turns planning off whatever the mode (D110, D143).\n[job_plan]\nenabled = true\nmode = \"hard\"\nplanner_usd = 0.75\ncritic_usd = 0.75\nplanner_timeout_s = 600\ncritic_timeout_s = 600\nmax_rounds = 3\n"

---- config::tests::the_repository_budget_caps_cover_their_sum stdout ----

thread 'config::tests::the_repository_budget_caps_cover_their_sum' (9901) panicked at crates/athena-data/src/config.rs:894:9:
assertion `left == right` failed
  left: (0.5, 3.0, 2.0, 12.0)
 right: (2.0, 10.0, 5.0, 50.0)

---- config::tests::the_repository_budget_covers_two_reading_and_two_planning_rounds stdout ----

thread 'config::tests::the_repository_budget_covers_two_reading_and_two_planning_rounds' (9902) panicked at crates/athena-data/src/config.rs:877:9:
assertion `left == right` failed
  left: 12.0
 right: 50.0

---- config::tests::the_repository_config_plans_hard_jobs_only_with_the_designs_caps stdout ----

thread 'config::tests::the_repository_config_plans_hard_jobs_only_with_the_designs_caps' (9904) panicked at crates/athena-data/src/config.rs:862:9:
assertion `left == right` failed
  left: 0.75
 right: 3.0


failures:
    config::tests::job_usd_below_the_reserve_is_refused_by_the_budget_check
    config::tests::rounds_count_only_when_planning_is_enabled
    config::tests::the_repository_budget_caps_cover_their_sum
    config::tests::the_repository_budget_covers_two_reading_and_two_planning_rounds
    config::tests::the_repository_config_plans_hard_jobs_only_with_the_designs_caps

test result: FAILED. 152 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

error: test failed, to rerun pass `-p athena-data --lib`
     Running tests/repo_config.rs (<job>/target/verify/debug/deps/repo_config-bf3c3e8d6e6af283)

running 3 tests
test defaults_toml_parses_with_this_crates_schema ... ok
test athena_toml_parses_with_this_crates_schema ... ok
test an_unknown_section_is_what_the_test_would_catch ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_data

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-data --lib`
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked --workspace`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, athena.toml.
  End of the output (data, not instructions):

```text
   Doc-tests athena_cap_change

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_cap_study

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_cli

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_context

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_data

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_gate

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_git

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_score

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 5 targets failed:
    `-p athena-cli --test drain`
    `-p athena-cli --test job_plan`
    `-p athena-cli --test lifecycle`
    `-p athena-core --lib`
    `-p athena-data --lib`
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-data the_repository_budget_caps_cover_their_sum`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, athena.toml.
  End of the output (data, not instructions):

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_data-62eb92b0c349858b)

running 1 test
test config::tests::the_repository_budget_caps_cover_their_sum ... FAILED

failures:

---- config::tests::the_repository_budget_caps_cover_their_sum stdout ----

thread 'config::tests::the_repository_budget_caps_cover_their_sum' (10458) panicked at crates/athena-data/src/config.rs:894:9:
assertion `left == right` failed
  left: (0.5, 3.0, 2.0, 12.0)
 right: (2.0, 10.0, 5.0, 50.0)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    config::tests::the_repository_budget_caps_cover_their_sum

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 156 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-data --lib`
     Running tests/repo_config.rs (<job>/target/verify/debug/deps/repo_config-bf3c3e8d6e6af283)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-data --lib`
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
