# Job 20261029-110953-bw7k: verify "Make timed-out checks environment stops; add status check durations"

## Request (verbatim)

> try again on 20261029-084645-sza7

## Reading (accepted by the user)

Retry of job 20261029-084645-sza7 (rejected because it deferred part 3). Goal: the maintainer stops random rejections like 20261026-232434-6wfq. This job does all three parts: check_timeout_s = 1800, a timed-out check ends as an environment stop that prints duration against the limit, and athena status reports median and slowest test-check durations with an 80% flag.

Assumptions:
- Code survey: read crates/athena-core/src/checks.rs (run_check kills at deadline with note 'killed at the deadline of Ns', exit code None), crates/athena-core/src/disk.rs (is_environment_reason line 212, resumable_stop line 221), crates/athena-core/src/run/mod.rs (check_and_verify; check timeout used at lines 1107, 1395, 1844) and crates/athena-core/src/status.rs (status() line 1120), and crates/athena-data/src/config.rs (check_timeout_s: u64 line 234, unchanged). No new record or data file under crates/ is added, so no check_record_file or AnyRecord change; the status durations are computed from existing stored check runs of jobs.
- Job 20261029-084645-sza7 findings addressed: (a) major, reading deferred part (3): fixed, status.rs and the status test are now deliverables; (b) minor, a timeout hid a real failure in another check: fixed by holding the job only when every failed check timed out, and rejecting when any check failed for real; (c) minor, no check record kept on the early return: accepted as in the low-disk stop, the closing reason carries duration and limit, so no new record field; (d) nit, test seam under cfg(test) follows the replay_seam pattern.
- The candidate of 20261029-084645-sza7 changed CHANGELOG.md, athena.toml, checks.rs, disk.rs and run/mod.rs; all five are kept as the starting approach (shared deadline note, timed_out(&CheckRun), CHECK_TIMEOUT reason in disk.rs) and re-done from the base, with status.rs added.
- The user's reply governs: all three parts are done in this job with no follow-up split.
- A 'test check' in status means the check named test in the stored check runs; jobs with no such check are skipped; the limit is the configured check_timeout_s.

## Acceptance criteria

1. [content] Grep athena.toml shows check_timeout_s = 1800, and crates/athena-data/src/config.rs still declares check_timeout_s: u64 with validation unchanged.
2. [behavior] Run cargo test --locked: a test with a fake check runner whose check is killed at its deadline (exit code None, deadline note) shows the job closes as an environment stop (disk::is_environment_reason true, resumable_stop true), not Rejected.
3. [behavior] Run cargo test --locked: a companion test shows a check failing with a real nonzero exit code is still a failed check leading to rejection, and a run where one check timed out and another failed for real is still rejected.
4. [behavior] Run cargo test --locked: a test shows the timeout stop reason and printed line contain the measured duration and the limit (for example '905s of 900s limit').
5. [behavior] Run cargo test --locked: a status test with seeded jobs shows athena status prints the median and slowest test-check durations over the last 20 jobs against the limit, flags the slowest when it exceeds 80% of the limit, and does not flag it when under.
6. [content] Grep crates/athena-core/src/disk.rs shows is_environment_reason recognises the check-timeout reason, with a unit test in the same file.
7. [behavior] Run cargo test --locked: the repository config test that loads athena.toml passes with 1800.
8. [behavior] Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked: all pass.
9. [content] CHANGELOG.md has an entry describing the 1800 s limit, timeout-as-environment-stop and the status durations report, citing job 20261026-232434-6wfq.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/status.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the schema or validation of check_timeout_s in crates/athena-data/src/config.rs
- Speeding up the test suite or changing other timeouts (reader, producer, verifier)
- Re-running or reopening job 20261026-232434-6wfq
- Adding a new field to CheckRun or other records
- Editing defaults.toml, workflows, or records/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust plus athena.toml.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy -D warnings and cargo test pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test, clippy and fmt runs.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: A kill at the deadline is classified apart from a real failure, rather than only raising the limit.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check job end state and printed status output, not wiring.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The timeout classification is enforced by code and tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a near-limit run is held and retried, not rejected.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change over the files named in the request.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check with the Grep tool on athena.toml and crates/athena-data/src/config.rs, plus the diff stat. Run: `git diff --stat`. Observes: athena.toml has `check_timeout_s = 1800`; config.rs still declares `pub check_timeout_s: u64` (around line 234) and crates/athena-data/src/config.rs does not appear in the diff, so its validation is unchanged.
- Criterion 2: Job-level test `a_check_killed_at_its_deadline_closes_as_an_environment_stop` in `produce_and_verify_order_tests` (crates/athena-core/src/run/mod.rs). Entry point: `Athena::go` on a job from `athena.ask(..)` in the existing harness (`athena_with`, `Docs` capability, fake agent script, `repo()` loading athena.toml). Preconditions: `test_support::plenty_of_space`, checks not resumed, not nested, no usable baseline. Seam: `check_seam::install(..)`, a cfg(test) module that step 4 adds to run/mod.rs (it does not exist today; modelled on `replay_seam` in lib.rs, which exists), read in `SnapExec::exec` (exists, around line 969). The fake returns only a CheckRun with exit_code None, passed false, note `checks::deadline_note(1800)` and duration 1805 s. `held_for_timeout`, the reason builder, `failure` and the state come from production code, not the fake. Run: `cargo test --locked -p athena-core a_check_killed_at_its_deadline_closes_as_an_environment_stop`. Observes: The job closes Failed (not Rejected); `disk::is_environment_reason(&reason)` is true; `disk::resumable_stop(JobState::Closed(Failed), Some(&reason))` is true; the verifier call counter is 0.
  Fails without the behavior: Without the `held_for_timeout` early return the job goes on to the verifier, `judge` sees a failed check and the state is Rejected, so the Failed, environment and resumable assertions fail.
- Criterion 3: Three parts. (a) Job-level `a_real_nonzero_check_failure_is_still_rejected` through `Athena::go` with no seam installed: the real `run_check` runs a capability check `false` (a real exit 1). (b) Unit test `held_for_timeout_needs_a_new_failure_and_every_new_failure_timed_out` in checks.rs, calling `held_for_timeout`, the function `check_and_verify` calls, on seeded CheckRun values (pure records, no seam; `Baseline` literals use its public fields, with note None). The mixed case is shown here outright because `run_local` stops at the first failure. Cases: [timeout, exit 1] and [exit 1, timeout] give None (these are the rejected 'one timed out, another failed for real' sets); [timeout] gives Some; all-pass gives None; a failure with the kill-switch note gives None; pre-existing timeout alone (baseline failed `test`) gives None; pre-existing real failure plus new timeout gives Some; pre-existing timeout plus new real failure gives None. (c) A real-kill test `a_real_deadline_kill_is_classified_as_a_timeout` in checks.rs runs `run_check` with a 300 ms limit (as the existing baseline timeout test does) and asserts `timed_out`, so the shared note is shown to match a real kill. Run: `cargo test --locked -p athena-core a_real_nonzero_check_failure_is_still_rejected held_for_timeout_needs_a_new_failure_and_every_new_failure_timed_out a_real_deadline_kill_is_classified_as_a_timeout`. Observes: A real nonzero exit leaves the job Rejected with a non-environment reason; every set containing a new real failure is not held (so judge rejects it as today); pre-existing failures never cause a hold or a rejection by themselves; a real kill is classified as a timeout.
  Fails without the behavior: If the hold ignored real failures, the mixed cases would return Some and a real failure would be retried instead of rejected. If it ignored the baseline, the pre-existing timeout alone would return Some (a passing job turned into a failing stop) and the pre-existing failure plus new timeout case would return None.
- Criterion 4: Pure unit test in disk.rs `check_timeout_reason_names_the_duration_and_the_limit` calling `check_timeout_reason("test", 905, 900, job)`, plus the criterion 2 job test, which reads the stored closing reason and the printed line from the harness `Lines` UI. Seam as in criterion 2. Run: `cargo test --locked -p athena-core check_timeout_reason_names_the_duration_and_the_limit a_check_killed_at_its_deadline_closes_as_an_environment_stop`. Observes: The unit test finds `905s of 900s limit` and `athena retry` in the reason. The job test finds `1805s of 1800s limit` in both the stored closing reason and the printed `failed     ` line, the limit coming from `jc.config.budget.check_timeout_s` as loaded from athena.toml.
  Fails without the behavior: If the reason omitted the duration or the limit, or used another format, the `905s of 900s limit` and `1805s of 1800s limit` substring assertions fail.
- Criterion 5: Tests in status.rs on `check_durations(&Records, limit_s)` and `CheckDurations::text()`, the functions `Athena::status()` calls to fill `StatusView.check_durations`; fixture: `Records` seeded like the existing status.rs tests that seed `verification`. Seed 22 request jobs, each with a `test` check of a chosen measured duration (basis 'measured'); the oldest 2 have huge durations to show only the newest 20 count; one job has no `test` check (skipped); one newest job has a skipped/placeholder `test` entry (0 s, basis 'not run') to show it is excluded from both the count and the median. Case A: slowest 1500 s of 1800 gives flagged true and text with median, slowest and limit. Case B: slowest 1000 s gives no flag. Case C: exactly 1440 s (80%) is not flagged. A display.rs test `status_prints_the_check_durations_line` calls `display::status` and asserts the line is printed when `check_durations` is Some and absent when None (it builds the view with the same helper the existing display status tests use, if one exists; otherwise it takes a view from a seeded `Athena::status()`). Run: `cargo test --locked -p athena-core check_durations_report_median_slowest_and_the_eighty_percent_flag && cargo test --locked -p athena-cli status_prints_the_check_durations_line`. Observes: The text shows median and slowest test-check durations over the last 20 jobs against the limit, flagged exactly when slowest exceeds 80% of the limit; the 'not run' placeholder does not lower the median or count as a job; `athena status` output contains the line.
  Fails without the behavior: Without the computation there is no line. With a window of all jobs the median and slowest differ from the expected values. With a wrong threshold, case B or C flips. If 'not run' placeholders were counted, the seeded median would be lower than expected. Without the display edit the display test finds no line.
- Criterion 6: Content check: Grep `CHECK_TIMEOUT` in crates/athena-core/src/disk.rs to see `is_environment_reason` contains `reason.starts_with(CHECK_TIMEOUT)`. Unit test `the_check_timeout_reason_is_an_environment_reason` in the disk.rs tests module. Pure functions, no seam. Run: `cargo test --locked -p athena-core the_check_timeout_reason_is_an_environment_reason`. Observes: True for the built reason and the bare prefix; false for 'the verifier returned no valid verdict'; `resumable_stop(Closed(Failed), Some(reason))` true and `Closed(Rejected)` false; the reason starts with `scope_guard::ENVIRONMENT_PREFIX`.
  Fails without the behavior: Without the recognition in `is_environment_reason` the assertions fail, and the criterion 2 job test sees `resumable_stop` false.
- Criterion 7: Run the full suite as one plain command. Tests that parse athena.toml through the unchanged validation include `committed` in routing.rs (`Config::parse(include_str!("../../../athena.toml"))`, line 577), the invocation.rs test at line 376, and the `repo()` helper in run/mod.rs that every job test uses. The producer names the one it relied on in its report. Run: `cargo test --locked`. Observes: All tests that parse athena.toml pass with check_timeout_s = 1800.
  Fails without the behavior: If 1800 broke a validation bound in config.rs, `Config::parse` would fail and the routing, invocation and job tests would fail.
- Criterion 8: Run the three Athena checks as three separate plain commands in this order: fmt, clippy, test. Run: `cargo fmt --all --check`. Observes: Each of `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` exits 0.
  Fails without the behavior: Any formatting, lint or test regression (including a StatusView literal missing the new field) fails one of the three.
- Criterion 9: Content check: Grep CHANGELOG.md for `20261026-232434-6wfq`, `1800` and `athena status`, and read the diff. Run: `git diff CHANGELOG.md`. Observes: One entry under Unreleased, in the file's style, states the 1800 s limit, that a timed-out check ends the job as an environment stop resumable with `athena retry`, that pre-existing failures are not counted, that `athena status` reports median and slowest test-check durations with the 80% flag, and cites job 20261026-232434-6wfq.

Planned files:

- athena.toml (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 96417de756fa → candidate d1564b963d18. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  14 ++
 athena.toml                                 |   2 +-
 crates/athena-cli/src/display.rs            |   3 +
 crates/athena-cli/tests/status_durations.rs |  92 ++++++++++++++
 crates/athena-core/src/checks.rs            | 191 +++++++++++++++++++++++++++-
 crates/athena-core/src/disk.rs              |  53 +++++++-
 crates/athena-core/src/run/mod.rs           | 191 ++++++++++++++++++++++++++--
 crates/athena-core/src/status.rs            | 190 ++++++++++++++++++++++++++-
 8 files changed, 715 insertions(+), 21 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1628 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
