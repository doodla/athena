# Job 20261029-084645-sza7: verify "Raise check timeout to 1800s; treat a timed-out check as an environment"

## Request (verbatim)

> Priority: urgent
> The test check is now cut off at random. [budget] check_timeout_s in athena.toml is 900 seconds, and the full suite (1617 tests) takes 750 to 940 seconds, so a run near the limit is killed partway and recorded as a failed test check: on 2026-10-29 job 20261026-232434-6wfq was rejected with 'check test failed' after only 591 tests ran, exit code none, 905 s, while the verifier's own full run of the same tree passed. Fix: (1) raise check_timeout_s to 1800 in athena.toml, with the schema accepting it unchanged; (2) record a check killed by its time limit as an environment stop (the existing environment-reason path), not a failed check, so the job is held and retried rather than rejected, and print the measured duration against the limit; (3) report in athena status the median and slowest test-check durations over the last 20 jobs against the limit, flagged when the slowest exceeds 80% of it.
> Acceptance:
> - A test with a fake check runner that exceeds its time limit shows the job ends as an environment stop, not a rejection, and a real test failure is still a failed check.
> - A test shows status reports the median and slowest test-check durations and flags one over 80% of the limit.
> - athena.toml has check_timeout_s = 1800 and the repository config test passes.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: athena.toml, crates/athena-core/src/run/mod.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/status.rs, CHANGELOG.md

## Reading (accepted by the user)

Goal: stop random rejections of jobs (such as 20261026-232434-6wfq) whose full test check is killed near the 900 s limit. This change does parts (1) and (2): raise check_timeout_s to 1800 in athena.toml, and make a check killed at its deadline an environment stop (held and retried, not rejected) that prints the measured duration against the limit. Part (3), the athena status durations report, is a follow-up request.

Assumptions:
- Code survey: read crates/athena-core/src/checks.rs (run_check polls to a deadline, kills the process group and sets note 'killed at the deadline of Ns', exit_code None, passed false), crates/athena-core/src/disk.rs (is_environment_reason matches a reason prefix; resumable_stop resumes only Failed with an environment reason), crates/athena-core/src/run/mod.rs (line ~554 holds only Failed plus environment reason; checks run via SnapExec with timeout from budget.check_timeout_s at ~1107 and ~1395; the check summary line prints 'checks ... (Ns)' at ~1447), crates/athena-data/src/config.rs (check_timeout_s: u64, so 1800 needs no schema change). Entry point is produce_and_verify; baseline, then checks, then verification. The duration is measured directly from CheckRun.duration, not a stand-in. No new record or data file is added, so no AnyRecord or check_record_file change.
- The timeout is detected from the CheckRun the runner returns: a check killed at its deadline is told apart from a real failure by a single shared marker or helper in checks.rs (the existing deadline note), not by a new record field. The fake-runner test supplies such a CheckRun.
- The environment stop reuses the existing environment-reason path: the job closes Failed with a reason that disk.rs is_environment_reason accepts (a new reason prefix for 'check timed out'), so resumable_stop and athena retry treat it as held and resumable.
- Rejected job 20261026-232434-6wfq is not re-run by this change; it is addressed by fixing the root cause (limit and classification) so its failure mode no longer rejects. Its own candidate was a different refactor and is not touched.
- Part (3), status median and slowest durations with an 80% flag, is a separate follow-up so one run fits inside verifier_timeout_s.

## Acceptance criteria

1. [content] athena.toml has check_timeout_s = 1800 (Grep athena.toml), and crates/athena-data/src/config.rs still declares check_timeout_s: u64 unchanged.
2. [behavior] Run cargo test --locked: a new test with a fake check runner whose check is killed at its deadline (exit code None, deadline note) shows the job ends closed as an environment stop (disk::is_environment_reason true on its reason, resumable_stop true), not Rejected; the test asserts this.
3. [behavior] Run cargo test --locked: a companion test shows a fake check that fails with a real nonzero exit code is still a failed check that leads to rejection, not an environment stop.
4. [behavior] Run cargo test --locked: a test shows the timeout stop message and printed line contain the measured duration and the limit (for example '905s of 900s limit'), and the repository config test (the one loading athena.toml) passes with 1800.
5. [content] crates/athena-core/src/disk.rs recognises the new check-timeout reason in is_environment_reason (Grep disk.rs), with a unit test in the same file for it.
6. [behavior] Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked: all pass.
7. [content] CHANGELOG.md has an entry describing the 1800 s limit and the timeout-as-environment-stop behavior, citing job 20261026-232434-6wfq.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Part (3): the athena status report of median and slowest test-check durations over the last 20 jobs with an 80% flag, and crates/athena-core/src/status.rs; submit as a follow-up request
- Changing the schema or validation of check_timeout_s in crates/athena-data/src/config.rs
- Speeding up the test suite or changing other timeouts (reader, producer, verifier)
- Re-running or reopening job 20261026-232434-6wfq
- Adding a new field to CheckRun or other records
- Editing defaults.toml, workflows, or records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests plus cargo test, clippy -D warnings and fmt to pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Status reporting split into a follow-up so this is one focused change.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise timeout versus real failure with a fake runner rather than checking wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes both the limit and the misclassification of a killed check as a failed one.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The classification is enforced in code and tests, not prose.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry and printed message name the duration and limit for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check plus a config test. Grep athena.toml for `check_timeout_s = 1800`; Grep crates/athena-data/src/config.rs for `pub check_timeout_s: u64` (file unchanged; git diff shows no change). A new test in run/mod.rs (step 6c) parses the repository file with `athena_data::Config::parse(include_str!("../../../../athena.toml"))` and asserts budget.check_timeout_s == 1800. No seam needed; the repository file is the fixture. Run: `cargo test --locked -p athena-core repository_check_timeout_is_1800 && cargo test --locked -p athena-data`. Observes: The test passes with 1800 and the existing athena-data config tests still pass, so the schema accepts it unchanged.
  Fails without the behavior: If athena.toml still said 900 the config test asserting 1800 fails; if the schema type changed, the athena-data tests would not compile or pass.
- Criterion 2: New test in produce_and_verify_order_tests, run through the production entry point `Athena::go` (-> produce_and_verify -> check_and_verify) with the existing athena_with/base_script/Docs fixtures (fake agent driver, temp git repo, plenty_of_space gate). Preconditions: planning off, disk gate passes. The fake check runner is a thread-local seam (new cfg(test) module `check_seam` in run/mod.rs, added in plan step 4; it does not exist today) installed with a Drop guard and consulted at the top of SnapExec::exec. It returns a CheckRun with passed false, exit_code None, duration 905.0 s, note = checks::deadline_note(900s) built by the production helper. The seam holds no classification logic. After go, read Finished and assert state == Failed, disk::is_environment_reason(&reason), resumable_stop(Closed(Failed), Some(&reason)), state != Rejected, no verifier invocation (counting()), and read_resume(job) is still Some at ResumeStage::Checks. Run: `cargo test --locked -p athena-core a_check_killed_at_its_deadline_is_an_environment_stop`. Observes: The job closes Failed with an `environment: check timed out` reason, is resumable, kept its resume mark, and the verifier never ran.
  Fails without the behavior: If check_and_verify did not classify a timed-out run, the job would reach verification and end Rejected ('check probe failed'); the assertions on Failed state, is_environment_reason and resumable_stop would fail.
- Criterion 3: Companion test via `Athena::go` with the same fixtures and the same check_seam (added in step 4). The seam returns a failed CheckRun with exit_code Some(1), passed false and an ordinary note. Assert state == Rejected, reason is 'check probe failed', !is_environment_reason(reason) and !resumable_stop. A unit test in checks.rs also asserts timed_out is false for exit 3, a pass, the kill-switch note, a spawn failure and a skipped run. The existing test produce_dir_is_removed_after_checks_pass_and_kept_when_a_check_fails (a real `sh -c test` failure ending Rejected) is a regression guard. Run: `cargo test --locked -p athena-core a_real_failing_check_is_still_a_rejection`. Observes: A genuine nonzero-exit check still produces the Rejected outcome, not an environment stop.
  Fails without the behavior: If the classifier treated every failed check (or every exit_code None) as a timeout, this job would end Failed with an environment reason instead of Rejected, and the test would fail.
- Criterion 4: In the criterion-2 test, additionally assert that the Finished reason and a line captured by the Lines UI (starting `failed     `) contain `905s of 1800s limit` (test config from the repository athena.toml via repo(), now 1800; duration 905 from the fake seam). A disk.rs unit test checks `check_timeout_reason` text directly: contains the check name, '905s of 900s limit', 'not the work' and `athena retry <job>`. The criterion-1 config test covers the 1800 load. Run: `cargo test --locked -p athena-core check_timeout`. Observes: Both the closing reason and the printed `failed` line carry the measured duration against the limit; the config loads with 1800.
  Fails without the behavior: Without the duration and limit in the reason, the substring assertions on the closing reason, the UI line and the disk.rs text test fail.
- Criterion 5: Grep disk.rs for `CHECK_TIMEOUT` and its use in is_environment_reason. New unit test in disk.rs `a_check_timeout_reason_is_an_environment_reason` builds `check_timeout_reason("test", 905.0, 900, &id(1))` and asserts is_environment_reason true, resumable_stop(Closed(Failed), Some(&r)) true, resumable_stop(Closed(Rejected), Some(&r)) false, and that 'check test failed' and an admission-refused reason are not environment reasons. No seam; pure functions. Run: `cargo test --locked -p athena-core is_environment_reason`. Observes: The reason is classified as environment and only a Failed close is resumable.
  Fails without the behavior: Without the new prefix in is_environment_reason, the assertion that the timeout reason is an environment reason, and resumable_stop for a Failed job with it, fail.
- Criterion 6: Run the three Athena checks in order after `cargo fmt --all` has been applied (step 8). They exercise the whole workspace including the new tests of criteria 2-5 and 7's unaffected suite; the cfg(test) seam is guarded by cfg(test) on both definition and use so non-test builds have no dead code. No seam of its own. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0: formatting is canonical, clippy reports no warning (no dead code from timed_out, check_timeout_reason, deadline_note or the seam; no unused imports), and every test, old and new, passes.
  Fails without the behavior: Without the classification the new timeout tests in `cargo test --locked` fail (job ends Rejected); a helper added but not wired into check_and_verify (timed_out or check_timeout_reason unused) makes `clippy -D warnings` fail with a dead-code error; unformatted new code makes `cargo fmt --all --check` exit non-zero; a broken existing test, such as a hard-coded 900 limit, fails `cargo test --locked`.
- Criterion 7: Content check: Grep CHANGELOG.md for `20261026-232434-6wfq` and `check timed out`. The [Unreleased] entry names the 1800 s limit, the timeout-as-environment-stop (`environment: check timed out`, held, `athena retry`), the printed duration against the limit, the root cause (900 s limit below the 750-940 s suite time and a killed check recorded as failed) and notes the status report is a follow-up. Run: `cargo test --locked -p athena-core`. Observes: The entry exists and cites the job; the existing tests that read CHANGELOG.md (if any) still pass.
  Fails without the behavior: Without the entry, Grep for the job id in CHANGELOG.md finds nothing (the content criterion is unmet).

Planned files:

- athena.toml (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base bae9ceb4f80f → candidate 77c68c950881. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  16 +++
 athena.toml                       |   2 +-
 crates/athena-core/src/checks.rs  |  78 ++++++++++++++-
 crates/athena-core/src/disk.rs    |  48 ++++++++-
 crates/athena-core/src/run/mod.rs | 202 ++++++++++++++++++++++++++++++++++++++
 5 files changed, 338 insertions(+), 8 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1622 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
