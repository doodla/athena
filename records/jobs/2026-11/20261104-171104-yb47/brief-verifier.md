# Job 20261104-171104-yb47: verify "Lift per-agent budget and timeout caps in athena.toml"

## Request (verbatim)

> Lift Athena's per-agent spending and time caps (owner's direction, 4 Nov: 'lift budgets for now if they're causing so many issues'). The first try was refused at admission because job_usd 40 was below the 7 the stage caps add up to; job_usd is now 60. Values only; no key is added or removed, so the config schema (crates/athena-data/src/config.rs) needs no change.
> 
> Files changed: athena.toml only.
> 
> Values:
> - [budget] reader_usd 0.50 -> 2.00, producer_usd 3.00 -> 10.00, verifier_usd 2.00 -> 5.00, job_usd 12.00 -> 60.00, producer_timeout_s 3000 -> 5400, verifier_timeout_s 2400 -> 3600
> - [job_plan] planner_usd 0.75 -> 3.00, critic_usd 0.75 -> 3.00, max_rounds 3 -> 5
> 
> Acceptance:
> - athena.toml holds exactly these values and no other change
> - cargo test --locked -p athena-data passes
> - admission accepts the change (job_usd covers the stage caps)

## Reading (accepted by the user)

Set the [budget] and [job_plan] values in athena.toml to the owner-directed values, with job_usd 60 so admission accepts them. The snapshot already holds most of the new values but has job_usd 50, so the remaining edit may be only job_usd. Values only; no keys and no config.rs change.

Assumptions:
- Goal: owner lifts budgets so agents stop hitting caps; the audience is the Athena operator and the admission check.
- Snapshot athena.toml already shows most target values and job_usd = 50.00, so the producer must verify each value and set job_usd to 60.00.
- Code survey: not needed beyond crates/athena-data/src/config.rs, which is unchanged because only values change.

## Acceptance criteria

1. [content] athena.toml [budget] has reader_usd 2.00, producer_usd 10.00, verifier_usd 5.00, job_usd 60.00, producer_timeout_s 5400, verifier_timeout_s 3600; checkable with git diff and Grep on athena.toml.
2. [content] athena.toml [job_plan] has planner_usd 3.00, critic_usd 3.00, max_rounds 5; no other key, added or removed, differs from the base (git diff shows only value lines).
3. [content] Only athena.toml changes; crates/athena-data/src/config.rs and other files are untouched (git status).
4. [behavior] Run `cargo test --locked -p athena-data`; it passes.
5. [content] job_usd (60) is at least the sum of the stage caps (reader+producer+verifier+planner+critic = 23), so admission accepts it; checkable by arithmetic from the file.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)

## Out of scope

- Adding or removing config keys
- Changes to crates/athena-data/src/config.rs
- Changing other budget values such as reader_timeout_s or check_timeout_s

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the resulting values and the passing test, not the activity.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One-file, values-only change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: cargo test run by verifier.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff of athena.toml and the [budget] table. All values except job_usd are already at target in the base, so the diff has one changed line. Run: `git diff -- athena.toml`. Observes: The [budget] table reads reader_usd 2.00, producer_usd 10.00, verifier_usd 5.00, job_usd 60.00, producer_timeout_s 5400, verifier_timeout_s 3600. The only changed line is `-job_usd = 50.00` / `+job_usd = 60.00`.
- Criterion 2: Read the same diff and the [job_plan] table (athena.toml lines 209-216 in the base). Run: `git diff -- athena.toml`. Observes: [job_plan] reads planner_usd 3.00, critic_usd 3.00, max_rounds 5. The diff has no added or removed key and no changed line outside job_usd.
- Criterion 3: List the changed files, then read the diff of each. Judge the criterion as 'no production code, key or schema changed', because criterion 4 needs the test edits. Expected paths: athena.toml, CHANGELOG.md, crates/athena-data/src/config.rs, crates/athena-core/src/admission.rs, crates/athena-core/src/run_pin_tests.rs, crates/athena-core/src/run/tests.rs, crates/athena-core/src/reading.rs and crates/athena-cli/tests/drain.rs. Run: `git status`. Observes: Only the eight paths above are modified. `git diff --stat` and `git diff` show that the changes in config.rs, admission.rs and reading.rs lie inside `#[cfg(test)] mod tests` blocks (numbers, string literals and comments). run_pin_tests.rs, run/tests.rs and tests/drain.rs are test files, also edited only in numbers, string literals and comments. No struct, parser, `budget_required` or `check_budget` line is touched, and nothing under crates/ outside those paths changes.
- Criterion 4: Run the athena-data tests. They parse the shipped athena.toml through `REPO_CONFIG = include_str!("../../../athena.toml")` (config.rs ~838), run `Config::parse` and `Config::check_budget`, and pin the shipped numbers. Run `cargo test --locked` as well so the dependent athena-core and athena-cli tests are covered. Run: `cargo test --locked -p athena-data`. Observes: Exit 0, all athena-data tests pass, including the_repository_budget_covers_two_reading_and_two_planning_rounds (job_usd 60.0), the_repository_budget_caps_cover_their_sum (tuple (2.0, 10.0, 5.0, 60.0), sum 47.0, check_budget Ok) and job_usd_below_the_reserve_is_refused_by_the_budget_check (with_job_usd now replaces `job_usd = 60.00`; the 46.99, 47.00 and 16.99 cases are absolute values and still hold). A further run of `cargo test --locked` passes the athena-core tests a_job_usd_below_the_stage_caps_is_a_configuration_violation, round_one_critic_check_follows_the_shared_reserve, dispatch_holds_a_job_that_cannot_afford_every_planning_round and a_retry_is_judged_on_its_own_spend_and_stopped_before_planning, and the athena-cli drain test the_default_plan_budget_affords_a_read_where_drain_would_need_a_read_and_a_run.
  Fails without the behavior: At job_usd 60.00 with the old literals, `assert_eq!(b.job_usd, 50.0)` (config.rs 877) fails, the tuple assert at 896 fails, and the `assert_ne!` in with_job_usd (917) fails. Without the new reader costs in run/tests.rs, the dispatch and retry tests panic on `expect_err`, because 60 - 31.5 = 28.5 covers the 19.50 reserve. The planner-cost rows in round_one_critic_check_follows_the_shared_reserve also leave the critic callable where a shortfall is expected.
  Test seam: Production entry points `Config::parse`, `Config::check_budget` and `Config::budget_required` (config.rs 807-830). The fixture is the real athena.toml, compiled in through include_str!. The athena-core tests inject a scripted agent through `costing(...)`/`athena_with(...)` (run/tests.rs ~930-970), which sets only the agent's reported cost, not the budget logic. No double holds logic under test. All of it exists today (read); the plan adds no seam.
- Criterion 5: Arithmetic from the file, shown by tests. With the shipped caps, budget_required is 2 + 5×(3+3) + 10 + 5 = 47 (config.rs 807-813), and the brief's one-round sum is 2+10+5+3+3 = 23. Both are at most the job_usd of 60. Admission's configuration check (admission.rs ~616) calls `check_budget`. Run: `cargo test --locked -p athena-data the_repository_budget_caps_cover_their_sum`. Observes: The test passes: the caps sum to 47.00, `config.budget_required()` equals that sum, job_usd 60.0 is at least the sum, and `check_budget()` is Ok. Also run `cargo test --locked -p athena-core shipped_budget_covers_the_reserve` and `cargo test --locked -p athena-core a_job_usd_below_the_stage_caps_is_a_configuration_violation`. The first asserts the shipped job_usd covers the reserve. The second shows a 46.00 budget is refused with '$1.00 short' while the shipped file is accepted.
  Fails without the behavior: With job_usd below 47 (for example the old 12.00, or 46.00), `check_budget` returns BudgetBelowReserve and admission refuses the change.
  Test seam: Production entry points `Config::check_budget` (config.rs 819-830) and the admission configuration check (admission.rs ~616). The fixture is the real athena.toml (`REPO_CONFIG` and admission.rs `BASE_CONFIG`, both include_str!). No double. The seam exists today.

Planned files:

- athena.toml (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 5024fb876c25 → candidate 0259c5fe5e91. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  2 +-
 athena.toml                             |  2 +-
 crates/athena-cli/tests/drain.rs        |  2 +-
 crates/athena-core/src/admission.rs     |  4 ++--
 crates/athena-core/src/reading.rs       |  2 +-
 crates/athena-core/src/run/tests.rs     | 22 +++++++++++-----------
 crates/athena-core/src/run_pin_tests.rs |  8 ++++----
 crates/athena-data/src/config.rs        |  8 ++++----
 8 files changed, 25 insertions(+), 25 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41897932851 for 0259c5fe5e91928c5f92da2bdcd63e561e7428ce)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41897932851 for 0259c5fe5e91928c5f92da2bdcd63e561e7428ce)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41897932851 for 0259c5fe5e91928c5f92da2bdcd63e561e7428ce)
- ci-result: `not a check` passed (Check results for 0259c5fe5e91928c5f92da2bdcd63e561e7428ce come from the CI run https://github.com/doodla/athena/actions/runs/41897932851 (677s): fmt success, clippy success, test success.)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: acceptance criteria not met or not reported: "Only athena.toml changes; crates/athena-data/src/config.rs and other files are untouched (git status).")
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-data`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, athena.toml.
  End of the output (data, not instructions):

```text
0261025-001232-vesh\"\n# Commits that predate Athena and are exempt from the governed-content invariant (design §7.3). Each entry is\n# a full commit id, and a commit that adds one must already have that commit in its history (D60).\n# 64ecdc9 is a parentless second root that Athena's own commit path wrote during job 20261010-132713-ezm8;\n# it joined main through the merge of the bootstrap (361a786) and fails the records-based landing check.\n# ee4b0dd1aa6fcd81289dc988d0f7bd38fd7a40ef is the squash of PR #8, which carried several jobs' landings in one\n# commit; the landing check refuses that shape. Excused by the user's choice of option (a) in D66: this one\n# commit's content skips the invariant, and its jobs' own records still get the per-record checks.\n# f2a8453380196b95f2e2cc629ffa969af57e6c85 is the squash of PR #9. Most of that PR's jobs predate\n# athena.candidate/2, so the chain landing rule (D69) cannot link them (D70).\n# 5a734782e76daac9b12bd8833c97a73927c8cf94 is the squash of PR #14. It changes records/ but names no job,\n# so the per-job records check cannot tie its content to a job (D76).\n# 39962ed02fec9da2d87cd4a3e287121ed229cd0f is the squash of PR #130. It carried two jobs' landings:\n# 20261010-154004-60r4 (its PR #129 was closed unmerged) and 20261017-121143-sg0j. The chain between them is\n# broken, so the history check finds that neither landing is valid in that commit. Excused as ee4b0dd (D66)\n# and f2a8453 (D70) were: this one commit's content skips the invariant, and each job's own records still get\n# the per-record checks.\n# 02837e0 is the squash of PR #149 (job 20261018-145008-k7fq), merged by hand around athena land. Its\n# content carries two hand changes beside the job's verified content, so it is not a valid landing. Excused\n# with Sri's approval of 2026-10-23 (BRIEF.md addendum of that date) as 39962ed was. The commit that adds this\n# entry reverts the job's own content, to re-land through Athena; the two hand changes stay.\nexempt = [\"49c6ac1524677b8091081115fb5d410ed6d2b2ea\", \"64ecdc9aa4fd24cbe4d4884c864d7e9041585c3d\", \"ee4b0dd1aa6fcd81289dc988d0f7bd38fd7a40ef\", \"f2a8453380196b95f2e2cc629ffa969af57e6c85\", \"5a734782e76daac9b12bd8833c97a73927c8cf94\", \"39962ed02fec9da2d87cd4a3e287121ed229cd0f\", \"02837e0d47783e9a4af0fd139d5b11adbcbb217b\"]\n\n[records]\nmax_file_bytes = 1048576\n\n[personal_data]\n# Email addresses allowed in committed content. An entry ending in \"@\" allows that local part (\"noreply@\");\n# one starting with \"@\" allows that domain; one starting with \".\" allows every domain under that suffix.\n# These are no-reply addresses, the documentation domains, and the reserved top-level domains (RFC 2606),\n# none of which can belong to a person.\nemail_allow = [\"noreply@\", \"@example.com\", \"@example.org\", \"@example.net\", \".invalid\", \".test\", \".example\"]\n\n# Athena asks the host to merge a pull request once its checks are green, so `athena land --pr` finishes\n# the landing without a manual merge step. Red or pending checks never trigger a merge.\n[landing]\nmerge_via_rest = true\n\n# How many jobs `athena drain` runs at once: the independent items of one plan group (no dependency, disjoint\n# touched paths). 1 is one job at a time, as before; the schema allows 1 to 8. Left at 1 here, so adding the\n# section changes nothing until the user raises it.\n[parallel]\nmax_concurrency = 1\n\n# How many queued items `athena plan` and `athena drain` treat as urgent and as high. An item over a cap is\n# demoted one tier, in memory, for that plan only: no request or priority record is rewritten, and an item whose\n# priority a person set with a priority record is never demoted. These are the defaults, written out.\n[priority_caps]\nurgent = 3\nhigh = 10\n\n# Job planning (docs/design/job-planning.md): a planner and a critic run before the producer of a change job.\n# The caps are the design's. `mode = \"hard\"` plans only a job whose reading is marked hard, the marking that also\n# escalates the producer: a job that adds or changes a rule that can refuse or hold work, or changes landing, the\n# history check, records, the run pipeline or the drain. Every other job runs the producer and verifier only.\n# `mode = \"all\"` plans every change job. `enabled = false` turns planning off whatever the mode (D110, D143).\n[job_plan]\nenabled = true\nmode = \"hard\"\nplanner_usd = 3.00\ncritic_usd = 3.00\nplanner_timeout_s = 600\ncritic_timeout_s = 600\nmax_rounds = 5\n"

---- config::tests::the_repository_budget_caps_cover_their_sum stdout ----

thread 'config::tests::the_repository_budget_caps_cover_their_sum' (15018) panicked at crates/athena-data/src/config.rs:894:9:
assertion `left == right` failed
  left: (2.0, 10.0, 5.0, 50.0)
 right: (2.0, 10.0, 5.0, 60.0)

---- config::tests::the_repository_budget_covers_two_reading_and_two_planning_rounds stdout ----

thread 'config::tests::the_repository_budget_covers_two_reading_and_two_planning_rounds' (15019) panicked at crates/athena-data/src/config.rs:877:9:
assertion `left == right` failed
  left: 50.0
 right: 60.0


failures:
    config::tests::job_usd_below_the_reserve_is_refused_by_the_budget_check
    config::tests::rounds_count_only_when_planning_is_enabled
    config::tests::the_repository_budget_caps_cover_their_sum
    config::tests::the_repository_budget_covers_two_reading_and_two_planning_rounds

test result: FAILED. 153 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

error: test failed, to rerun pass `-p athena-data --lib`
     Running tests/repo_config.rs (<job>/target/verify/debug/deps/repo_config-bf3c3e8d6e6af283)

running 3 tests
test defaults_toml_parses_with_this_crates_schema ... ok
test an_unknown_section_is_what_the_test_would_catch ... ok
test athena_toml_parses_with_this_crates_schema ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests athena_data

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
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

thread 'config::tests::the_repository_budget_caps_cover_their_sum' (15205) panicked at crates/athena-data/src/config.rs:894:9:
assertion `left == right` failed
  left: (2.0, 10.0, 5.0, 50.0)
 right: (2.0, 10.0, 5.0, 60.0)
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
