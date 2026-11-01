# Job 20261030-195907-hs8s: verify "Free producer build before checks; keep failed check output tails"

## Request (verbatim)

> Priority: urgent
> Two gaps in the checks stage, seen on 2026-10-30 when the read-only evidence snapshot job failed its tests after 493s (about half the usual time) with 9.3 GB free:
> 1. The producer's build directory is removed only after the checks pass, so during the checks the job holds two full builds (20 GB) and a disk allowance of about 40 GB runs out mid-test.
> 2. When a check fails, its output is not kept: the drain log says only 'test FAILED' and the job's records hold no failing test name or message, so neither a retry nor a person can tell a real failure from an environment one.
> Change: remove the producer's build directory before the checks start (the checks build in their own directory), and record each failed check's last 200 lines of output in the job's records and print the failing test names in the drain log.
> Acceptance:
> - A test through the real checks path shows the producer's build directory is gone when the checks start and the candidate's work tree is kept.
> - A test with a failing check shows its failing test name in the drain log line and its output tail in the job's committed records.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/checks.rs, crates/athena-core/src/disk.rs, crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

Goal: stop the job holding two builds during checks, and make failed checks diagnosable for retries and people. Remove the producer's build directory before checks start (keeping the candidate work tree), then save each failed check's last 200 output lines in the job's records and print failing test names in the drain log.

Assumptions:
- Code survey: read crates/athena-core/src/run/checks.rs (CheckRun exec at line 20, baseline checks with athena_agent::env::build), crates/athena-core/src/state.rs (existing test removing_the_producer_build_directory_keeps_the_rest_of_target), crates/athena-agent/src/env.rs (per-job build directories), crates/athena-cli/tests/drain.rs (existing low-disk tests that remove build dirs before checks). Entry: drain dispatches the job, the producer runs, then the checks stage runs; the removal must go between producer end and the first check. Record allow-list: if the output tail is stored as a new record or data file, the producer must find the validation covering it (check_record_file, AnyRecord or its round-trip test) and extend it; preferably it extends an existing record. The tail is measured directly from the real check output captured by the checks path, not a stand-in.
- Reuses the existing helper that removes the producer build directory in crates/athena-core/src/disk.rs and state.rs rather than adding new deletion logic.
- Failing test names are parsed from cargo test output lines like 'test x::y ... FAILED' and printed in the drain log line; falls back to the check name if none parse.
- Audience: people and automated retries reading the drain log and job records.
- No rule is added that refuses readings, so no refusal-rate criterion applies.

## Acceptance criteria

1. [behavior] Run cargo test: a new test through the real checks path (not a unit stub) asserts the producer's build directory does not exist when the first check starts and the candidate's work tree still exists afterwards.
2. [behavior] Run cargo test: a new test with a deliberately failing check asserts the drain log line contains the failing test name (not just 'test FAILED') and the job's committed records contain the last 200 lines of that check's output (a longer output is truncated to its final 200 lines).
3. [content] git diff shows the CHANGELOG.md entry describing both changes, and changes limited to crates/athena-core/src/run/checks.rs, crates/athena-core/src/disk.rs, crates/athena-cli/src/drain.rs, tests, and CHANGELOG.md (plus any record type needed).
4. [behavior] Run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test: all pass.
5. [behavior] Run cargo test: existing drain tests for low disk and interrupted-job reclaim still pass, showing the work tree and candidate are kept.

## Expected deliverables

- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/state.rs (modify)

## Out of scope

- Changing athena.toml, defaults.toml, or workflow files
- Changing disk allowance sizes or thresholds
- Reworking how checks build in their own directory
- Retry policy changes based on the recorded output

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy -D warnings criteria.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria exercised by running cargo test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the directory is gone and the output tail recorded, not wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Removes the build before checks rather than raising the disk allowance.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Changelog and clear log lines.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the named files.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test in crates/athena-core/src/run/tests.rs. It drives `Athena::go` with `athena_with(&repo,&state,check,recording_plenty_space,counting(counts,Some(root)))`. The `counting` fake producer plants `jobs/<id>/target/produce/debug/big`. `check` is a leaked `&'static [&'static str]`: an `sh -c` script that exits 1 if any `<root>/jobs/*/target/produce` exists. It also exits 1 unless `<root>/jobs/*/verify` and README.md in its cwd hold the candidate text ('A demo'). The test asserts the job is Landed, the ui lines show `disk       removed the producer's build directory` before `checks     `, and the committed candidate record has a candidate commit. Run: `cargo test --locked -p athena-core run::tests::producer_build_directory_is_gone_when_the_first_check_starts_and_the_candidate_is_kept`. Observes: Landed with a passing check. The check itself saw the producer build directory absent and the candidate's tree present. The removal line precedes the `checks     ` line.
  Fails without the behavior: Without the early removal, `target/produce` still exists when the first check starts. The real check command exits 1, the job is rejected and the test fails. Before the change the removal ran only after all checks passed.
  Test seam: Entry is `Athena::go` through `check_and_verify` and `SnapExec::exec` to `run_check`, which runs the real `sh` command. The seams `athena_with`, `counting`, `Lines` and `recording_plenty_space` exist (read in run/tests.rs and test_support.rs). The script path is injected via `Box::leak(format!(..))`. Nothing is added to production code for the test, and no check double is used.
- Criterion 2: New test in run/tests.rs through `Athena::go`. The check is a real `sh -c` that prints 300 numbered lines, then `test demo::the_failing_one ... FAILED`, then 5 more lines, and exits 1. The test asserts the `checks     ` ui line contains `demo::the_failing_one`. It then runs `Records::load(&athena.git,&athena.head().unwrap())`, finds the job's verification check run with `passed == false`, and asserts that its `output_tail` has exactly 200 lines, ends with the last printed line, contains the FAILED line, and does not contain the line numbered before the window. Unit tests in core checks.rs cover the line builder with `failed_tests` names, the fallback parse, the name-less fallback and a many-names cap. They also cover `run_check` with a 250-line failing `sh` spec, a passing spec keeping the byte tail, and the byte ceiling. Run: `cargo test --locked -p athena-core run::tests::a_failed_check_logs_its_failing_test_and_records_its_last_200_lines`. Observes: The drain line names the failing test. The committed record has the last 200 lines of a longer output and the earlier lines are cut.
  Fails without the behavior: Without the log change the line is `checks     probe FAILED (..s)` with no test name. Without the 200-line tail the record holds at most 4096 bytes and lacks line 101 or the failure line. The test fails on either.
  Test seam: Entry is `Athena::go` with `athena_with(.., FAILING_CHECK, plenty_of_space, base_script())`. The fixtures `Lines` and `Records::load` exist. The check is a real subprocess. Production helpers added in step 2 and 3 are the code under test. The unit tests use the existing `sh_spec` helper in checks.rs tests.
- Criterion 3: Run `git diff --stat` and `git diff CHANGELOG.md`. Check that the changed files are only crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs (call sites), crates/athena-core/src/checks.rs, crates/athena-core/src/disk.rs (doc comment), crates/athena-core/src/run/tests.rs (tests), and CHANGELOG.md. Run: `git diff --stat`. Observes: One new CHANGELOG entry under Unreleased names both changes: the producer build removed before the checks, and the failed-check output tail kept with failing test names in the drain log. No config, workflow, toml or record-type file appears.
- Criterion 4: Run the three Athena check commands in order. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. The updated `produce_dir_is_removed_before_the_checks_whether_they_pass_or_fail` test and the new tests pass with the whole suite.
  Fails without the behavior: A formatting, lint or test failure anywhere in the workspace fails the command.
  Test seam: The commands are the existing fmt, clippy and test checks. No new seam is needed.
- Criterion 5: Run the unchanged existing drain tests in crates/athena-cli/tests/drain.rs (read at lines 1509-1640 and 3463). Also run `cargo test --locked -p athena-core run::tests::` for the core low-disk and resume tests. Run: `cargo test --locked -p athena-cli --test drain`. Observes: The tests still pass: idle build directories are swept, the hold-and-resume path works, and an interrupted job keeps its work and candidate.
  Fails without the behavior: If the early removal broke the disk gate or removed the work or candidate, `low_disk_*`, `low_disk_before_the_checks_holds_the_job_and_a_retry_with_space_runs_the_checks` or `drain_start_reclaims_the_build_directory_of_an_interrupted_job_and_keeps_its_work_and_candidate` would fail.
  Test seam: The existing fixtures `disk_fixture`, `go_with_disk`, `plant_idle_job` and `low_once_built` exist and are untouched.

Planned files:

- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 3538ec208eaa → candidate 8601736dd116. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  12 +++
 crates/athena-core/src/checks.rs     | 184 ++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/disk.rs       |   8 +-
 crates/athena-core/src/run/checks.rs |  17 ++++
 crates/athena-core/src/run/mod.rs    |  30 ++----
 crates/athena-core/src/run/tests.rs  | 134 +++++++++++++++++++++++--
 6 files changed, 348 insertions(+), 37 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41317480416 for 8601736dd11630d2813f49654d96a7c2567ac409)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41317480416 for 8601736dd11630d2813f49654d96a7c2567ac409)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41317480416 for 8601736dd11630d2813f49654d96a7c2567ac409)
- ci-result: `not a check` passed (Check results for 8601736dd11630d2813f49654d96a7c2567ac409 come from the CI run https://github.com/doodla/athena/actions/runs/41317480416 (566s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
