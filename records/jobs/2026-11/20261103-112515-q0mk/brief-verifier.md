# Job 20261103-112515-q0mk: verify "Make a verifier timeout a distinct unjudged outcome, retried once"

## Request (verbatim)

> try again on 20261027-054228-k9ex

## Reading (accepted by the user)

Retry of job 20261027-054228-k9ex (dispatched, never verified, no findings recorded): its candidate work in 7 files is kept as the starting point and finished with a rerun of a timed-out verifier plus tests. The root cause is that a verifier killed at its deadline was recorded as the generic `verifier_failure` and failed the verdict like a judged failure, so criteria were rejected unjudged (jobs 20261010-154005-jsdj, 20261018-120736-90w3, 20261018-133651-1ncn). Audience: the Athena operator reading verdicts.

Assumptions:
- Code survey: I read crates/athena-core/src/run/verification.rs (verification_stage sets a plain-string verifier_failure at the budget, snapshot lock, could-not-run, non-Success outcome and no-valid-verdict points, and already reruns once on a system_fault), crates/athena-core/src/verify.rs (VerdictInputs.verifier_failure: Option<String>; judge pushes 'the verifier did not complete: ..' and makes the verdict fail when it is set) and crates/athena-core/src/run/send_back.rs (constructs VerdictInputs with verifier_failure: None). The property is measured directly by calling judge with the new failure value and by running verification_stage with a stub verifier. The change adds no record or data file under crates/, so check_record_file and AnyRecord are unaffected. It adds no refusal rule.
- Job 20261027-054228-k9ex: its candidate changed 7 files, and all are kept as deliverables. It typed verifier_failure as Incomplete or TimedOut and gave the timeout an 'unjudged' reason. The candidate was never verified and recorded no findings, so there is nothing to fix. What was missing is that its rerun on timeout and its behavior tests are not shown, so this retry adds them.
- A timeout is detected from the verifier invocation's recorded detail or outcome. The first timeout triggers one rerun, and the second leaves the result unjudged, not a pass.
- The verdict stays non-passing when unjudged, but the reason text says 'unjudged', not 'rejected', and does not add 'returned no verdict'.
- Previous out-of-scope items stay: no change to athena.toml, defaults.toml, workflows or the verifier_timeout_s value, and no per-criterion saving or deadline scaling.

## Acceptance criteria

1. [content] The change description names the root cause: a verifier timeout was recorded as the generic `verifier_failure` in crates/athena-core/src/run/verification.rs and failed the verdict in crates/athena-core/src/verify.rs exactly like a judged failure, so criteria were rejected unjudged.
2. [content] A Grep of the diff in crates/athena-core/src/verify.rs and crates/athena-core/src/run/verification.rs shows a distinct timeout variant whose reason says 'unjudged' and is separate from 'the verifier did not complete'; it never yields a pass.
3. [behavior] Run `cargo test -p athena-core`. A new test where the verifier times out on the first run and succeeds on the second ends with the second run's verdict. A second test where it times out twice ends with the unjudged reason, not as a judged rejection. Both fail on the base commit and pass on the change.
4. [behavior] Run `cargo test -p athena-core`. A new test shows that a non-timeout verifier failure, a failing check, or a judged fail verdict still fails the verdict with its existing reason.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked`; all pass with no warnings.
6. [content] outcome-check: the tests assert the verdict and reason recorded for a timed-out verifier, not only that types or wiring exist.
7. [content] CHANGELOG.md has an entry for the new outcome, and docs/records.md and docs/design/athena-v0.md are updated where they describe the verifier-failure reason.

## Expected deliverables

- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/records.md (modify)
- docs/design/athena-v0.md (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)

## Out of scope

- Re-landing the candidate work of jobs 20261010-154005-jsdj, 20261018-120736-90w3 and 20261018-133651-1ncn (run.rs trailer, docs/spec.toml, xtask/tests/spec.rs, docs edits)
- Changing athena.toml, defaults.toml or workflows, including the verifier_timeout_s value
- Saving each criterion's verdict as it is reached
- Making the deadline scale with the cargo work
- Changing the gate's reading of records

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is where the timeout is recorded and judged, not at one job.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test criteria.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Tests assert the recorded verdict and reason.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run named cargo test commands.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the change description (the CHANGELOG [Unreleased] 'Fixed' entry and the doc-comment on `VerifierFailure`) in the diff. Run: `git diff -- crates/athena-core/src/verify.rs crates/athena-core/src/run/verification.rs crates/athena-core/src/run/mod.rs CHANGELOG.md`. Observes: The CHANGELOG entry and the enum's doc-comment name verification.rs, verify.rs, the generic `verifier_failure`, and the fact that criteria were rejected unjudged like a judged failure.
- Criterion 2: Read the diff and `judge`. Look for `VerifierFailure::TimedOut`, the word 'unjudged', and the separate 'the verifier did not complete' text for `Incomplete`. The unit test `a_verifier_that_times_out_is_unjudged_never_a_pass` in verify.rs gives `judge` a passing `output` together with `TimedOut` and asserts a Fail verdict. Run: `git diff -- crates/athena-core/src/verify.rs crates/athena-core/src/run/verification.rs`. Observes: A distinct `TimedOut` variant whose reason says 'unjudged' and not 'did not complete'. `pass` still requires `verifier_failure.is_none()`, so no failure value yields a pass.
- Criterion 3: Run two new tests in run_pin_tests.rs beside `faulty_verifier`. The tweak `timing_out_verifier(runs, n)` sets `turn.outcome = InvocationOutcome::TimedOut`, `turn.structured = None` and `turn.detail = Some("deadline of 1s reached")` on the first `n` verifier invocations. Test A, `a_verifier_that_times_out_once_is_rerun_and_the_second_verdict_decides` (n=1), asserts runs.len()==2, finished.state==Landed, verification.summary==Some("Fine."), empty verification.reasons, and that `verifier_invocation` is the second run's seq. Test B, `a_verifier_that_times_out_twice_is_unjudged_not_rejected_as_judged` (n=2), asserts runs.len()==2 (no third run), state Rejected, `agent_verdict` None, reasons containing 'unjudged' and neither 'did not complete' nor 'returned no verdict', each criterion's evidence containing 'unjudged', and finished.reason containing 'unjudged'. The verifier checks the output lists both named tests, not '0 tests'. Run: `cargo test -p athena-core a_verifier_that_times_out`. Observes: The recorded state, summary, reasons, evidence and invocation seq after a timed-out first run and a good second run, and after two timed-out runs.
  Fails without the behavior: On the base commit the stage runs the verifier once and records 'the verifier did not complete: ..'. Test A fails (runs.len() is 1 and the state is Rejected, not Landed). Test B fails on the reason ('unjudged' is absent, 'did not complete' is present).
  Test seam: Production entry point: `Athena::go`, which reaches `verification_stage` in run/verification.rs. The tests call it through the existing `run_with` helper in run_pin_tests.rs. Guards and preconditions: a study job with a pinned `CountingPinner`, and the fake agent driver (athena-agent/src/fake.rs). Fixture: a scripted `Tweak` closure passed to `athena_with` that makes the fake return a `FakeTurn` with `outcome = TimedOut`. `run_with`, `Tweak` and the fake's TimedOut support exist (read in run_pin_tests.rs and fake.rs). Step 5 adds `timing_out_verifier`. The verdict logic is not in the double: it only sets the outcome.
- Criterion 4: Three tests share the substring `non_timeout`. In verify.rs, `non_timeout_incomplete_failure_keeps_the_did_not_complete_reason` passes `VerifierFailure::Incomplete("x")` to `judge` and expects Fail, reason 'the verifier did not complete: x', and no 'unjudged'. In verify.rs, `non_timeout_failing_check_and_judged_fail_keep_their_reasons` expects a failing `check(false)` and an `output` with verdict fail each to fail the verdict with the existing reason text and not say 'unjudged'. In run_pin_tests.rs, `non_timeout_verifier_error_is_not_rerun` scripts `InvocationOutcome::Error` with a detail and expects runs.len()==1, Rejected, a reason containing 'did not complete' and not 'unjudged'. The existing test `a_verifier_write_is_refused_by_the_lock_and_does_not_reach_the_work` keeps its 'could not run' reason. The verifier checks that all three named tests appear in the output. Run: `cargo test -p athena-core non_timeout`. Observes: The existing reasons and the Fail verdict are kept for a non-timeout verifier failure, a failing check and a judged fail verdict, with no rerun for an error.
  Fails without the behavior: If the timeout branch swallowed other failures, `Incomplete` would lose 'the verifier did not complete: ..'. A failing check or judged fail would stop failing the verdict, or an error would be rerun (runs.len()==2). These tests would then fail.
  Test seam: Production entry points: `crate::verify::judge` for the two unit tests, with inputs built from the existing `criteria()`, `check()` and `output()` helpers in the verify.rs test module (they exist); and `Athena::go` via `run_with` plus a `faulty_verifier`-style `Tweak` for the Error case (the Tweak pattern exists). The only fixture is a scripted fake turn that sets the outcome. Step 3 adds the unit tests and step 5 adds the pin test.
- Criterion 5: The verifier runs three plain commands, one at a time, each on its own on the final tree, with no chaining or grouping: (a) `cargo fmt --all --check`; (b) `cargo clippy --all-targets --locked -- -D warnings`; (c) `cargo test --locked` (the command listed here). Clippy runs with `-D warnings`, so any warning is a failure. Run: `cargo test --locked`. Observes: (a) exits 0 with no diff output; (b) exits 0 with no warnings; (c) exits 0 and reports no failed tests across the workspace, including the new ones.
  Fails without the behavior: fmt fails if the new code or tests are unformatted. clippy fails on an unused `VerifierFailure` import in run/mod.rs, a match that does not cover both variants, or any other warning. Both clippy and test fail to compile if a `VerdictInputs` constructor (send_back.rs line 270 or the verify.rs test constructors) still holds the old `Option<String>`. test fails if any new or existing test breaks.
  Test seam: Production entry points are the cargo commands themselves, run over the workspace in the working tree. No fake or fixture is used and nothing is injected. The checks are fixed by Athena's own check configuration and exist today. The workspace compiles the changed `VerdictInputs` constructors, the new `VerifierFailure` imports in run/mod.rs, the display.rs change and the new tests. Step 8 runs each command separately and fixes what it shows.
- Criterion 6: Read the new tests from criterion 3 and run them with the broader filter `times_out`. They must assert recorded values (`finished.state`, `verification.reasons`, `criteria[].evidence`, `summary`, `agent_verdict`, and the Fail verdict from `judge`), not only that the enum or wiring exists. Run: `cargo test -p athena-core times_out`. Observes: Each assertion reads the Finished or Verification record or the Judgment. No test only constructs the enum. The filter matches both the run_pin_tests.rs tests and the verify.rs unit test `a_verifier_that_times_out_is_unjudged_never_a_pass`.
  Fails without the behavior: A test that only built `VerifierFailure::TimedOut` would still pass on a build without the rerun or the 'unjudged' wording. These tests fail on the base commit because the recorded state and reasons differ.
  Test seam: Same as criterion 3: `Athena::go` through `run_with` with a scripted fake `TimedOut` turn (`run_with` exists; `timing_out_verifier` added in step 5), plus `judge` called directly in verify.rs (exists; unit test added in step 3).
- Criterion 7: Check the diff for a CHANGELOG [Unreleased] 'Fixed' entry, a sentence in docs/records.md at the verification record description (about line 96), and sentences in docs/design/athena-v0.md at the 'verifier judges exactly what will land' row (about line 787) and the verification.json row (about line 611). Each says a verifier that hits its deadline is rerun once and, if it times out again, the criteria are unjudged and the verdict does not pass. Run: `git diff -- CHANGELOG.md docs/records.md docs/design/athena-v0.md`. Observes: The three files contain the new outcome and wording ('unjudged', rerun once).

Planned files:

- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- docs/records.md (modify)
- docs/design/athena-v0.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base eb8db729660a → candidate 5fe66d86c1ee. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  11 +++
 crates/athena-cli/src/display.rs           |  22 +++--
 crates/athena-core/src/run/mod.rs          |   4 +-
 crates/athena-core/src/run/verification.rs |  58 +++++++++---
 crates/athena-core/src/run_pin_tests.rs    | 105 +++++++++++++++++++++-
 crates/athena-core/src/verify.rs           | 140 +++++++++++++++++++++++++++--
 docs/design/athena-v0.md                   |   4 +-
 docs/records.md                            |   2 +-
 8 files changed, 317 insertions(+), 29 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41633934663 for 5fe66d86c1ee28f56361cf1543f6701c975ba3bc)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41633934663 for 5fe66d86c1ee28f56361cf1543f6701c975ba3bc)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41633934663 for 5fe66d86c1ee28f56361cf1543f6701c975ba3bc)
- ci-result: `not a check` passed (Check results for 5fe66d86c1ee28f56361cf1543f6701c975ba3bc come from the CI run https://github.com/doodla/athena/actions/runs/41633934663 (495s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked`; all pass with no warnings.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
