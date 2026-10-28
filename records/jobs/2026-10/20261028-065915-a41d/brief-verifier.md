# Job 20261028-065915-a41d: verify "Don't count environment stops as failed attempts in the scope guard"

## Request (verbatim)

> Priority: high
> The scope guard paused the replay line (20261028-034820-v3rf) after an attempt whose work was complete: it passed planning, built, and passed every check, then stopped only because the disk was 0.2 GB short before the verifier. The guard counted that environment stop like a failed attempt and its spend like waste. Fix: an attempt that ended on an environment cause (low disk, usage limit, interrupted process) does not count toward the guard's attempt or round signals, and when a retry resumes a kept candidate the guard forecasts only the stages left, not a fresh job.
> Acceptance:
> - A test shows an attempt ending on low disk does not count toward a pause, and a failed verification does.
> - A test shows a retry resuming a kept candidate is judged against the cost of its remaining stages.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/scope_guard.rs, CHANGELOG.md

## Reading (accepted by the user)

The scope guard (crates/athena-core/src/scope_guard.rs) currently turns every non-landed, non-rejected closed job into a Failed attempt that counts toward T1 retries, T2 planning rounds, T3 cost and T5 time. Change it so an attempt that ended on an environment cause (low disk, usage limit, interrupted process) is excluded from the attempt and round signals, and so a retry that resumes a kept candidate is judged against the cost of its remaining stages only. Audience: the operator whose line was paused. Job 20261028-034820-v3rf is the paused line, not a job to redo: its five planning-critique findings (replay.md refused by the records check, test override placement, no skip switch, Records::load question) concern the replay feature and are not touched by this guard fix. The line is not edited, only unblocked.

Assumptions:
- Job 20261028-034820-v3rf's findings belong to the replay feature; this change only makes the guard stop counting its environment-stopped attempt, so the findings need no fix here and its own retry is out of scope.
- An environment stop is identified from the job's recorded outcome (state and reason text, e.g. low disk, usage limit, interruption) and the existing attempt_of mapping; no new record fields are added to athena-data.
- An environment-stopped attempt is excluded from T1 attempt count, T2 planning round and plan growth, T4 rejections and T5 minutes; a failed verification (Rejected) still counts.
- For a retry resuming a kept candidate, T3 compares spend against the p90 forecast scaled to the stages not yet done, and spend already made on the kept stages is not counted against it.
- Stage cost shares are fixed constants in scope_guard.rs, labelled as estimates.
- Thresholds and athena.toml are unchanged, so no governance file changes.

## Acceptance criteria

1. [behavior] A unit test in crates/athena-core/src/scope_guard.rs builds a line whose earlier attempt ended on low disk and shows evaluate_before_attempt gives no T1/T2/T5 pause or flag from it, while the same line with a Rejected verification does. Run: cargo test -p athena-core scope_guard.
2. [behavior] A unit test in scope_guard.rs shows a retry that resumes a kept candidate (planning, build and checks done) is judged against the cost of its remaining stages: spend that would pause as a fresh job does not pause, and the same spend on a non-resuming retry does. Run: cargo test -p athena-core scope_guard.
3. [behavior] The existing scope_guard tests (t4_pause_via_go_and_ask, retry_refuses_paused_line, t3_pause_from_reading_forecast and the rest) still pass, so real rejections and cost overruns still pause lines. Run: cargo test.
4. [behavior] cargo fmt --check and cargo clippy --all-targets -- -D warnings are clean.
5. [content] CHANGELOG.md has an entry naming the scope guard change: environment-ended attempts are not counted, and resumed retries are forecast on remaining stages.
6. [content] Each new constant or rule in scope_guard.rs has a doc comment stating its basis or marking it an estimate, and the diff touches only crates/athena-core/src/scope_guard.rs and CHANGELOG.md.

## Expected deliverables

- crates/athena-core/src/scope_guard.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/mod.rs (modify)

## Out of scope

- Changing athena.toml, defaults.toml or workflows
- Changing the guard thresholds or the T4 same-criterion rule
- Implementing the replay feature of job 20261028-034820-v3rf or retrying that job
- Changing the disk check or how outcomes are recorded in athena-data
- Unpausing or editing existing pause records

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests are required, and cargo test and clippy -D warnings must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The new rule is enforced in code by the guard's signal functions, with tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix changes how the guard classifies attempts, not just this one paused line.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria run the tests and check pause and no-pause results.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a complete attempt stopped on the environment no longer pauses the line.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The change is one focused edit to the guard plus a changelog entry.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments explain the new rule and any estimate.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test `environment_stop_is_not_counted_but_a_rejection_is` in `mod tests` of scope_guard.rs. Entry point: the public `evaluate_before_attempt`; no seam or fake is needed. Line A is two `att(Verdict::Environment, ...)` attempts with enough minutes, plan growth and rounds that raw counting would trip T1, T2 and T5. Line B swaps in `rej(...)` with the same costs and minutes. A second test, `a_low_disk_job_is_read_as_an_environment_attempt`, runs in `watcher_tests` through production `Athena::ask` and `go` with a new `rig_with_space` helper (added in step 6; today `rig` hard-codes `plenty_of_space`) that passes the `FreeSpace` fn to `StateDir::with_free_space`, and with the existing reader `crate::test_support::low_once_verifying`. It checks `request_records` yields `Verdict::Environment` and no `pause` or `flag` file exists. If the rig cannot reproduce the stop, drop that test and cover `attempt_of` with a `JobRecords` fixture. Run: `cargo test -p athena-core scope_guard`. Observes: Line A: t1, t2 and t5 are `Level::None`, and `evaluate_before_attempt` gives no pause or flag from them. Line B: the matching signals fire (T1 flags, T2 and T5 pause, T4 pauses on a repeated criterion). The end-to-end job closes `failed` with a reason starting `environment: low disk` and writes no pause or flag file.
  Fails without the behavior: Without the Environment classification and the filter in t1/t2/t5, line A reaches the same levels as line B and the first assertion (no pause) fails. Without T4 counting Rejected, the rejected control would not pause.
- Criterion 2: Unit test `resumed_retry_is_judged_on_remaining_stages` in `mod tests`. Entry points: public `t3_cost`, `t3_basis` and `evaluate_before_attempt`; no seam. Attempts: an Environment attempt (cost 1.4) then an attempt with `resumed: true` (cost 0.1) against p90 1.0, so spend counted is 0.1 against forecast 1.0 x `VERIFIER_SHARE`. A second case where the resumed attempt costs more than 1.5 x p90 x the share must pause, so T3 stays active. Control: the same costs with `resumed: false` give spend 1.5 against p90 1.0 and pause. A `request_records` case with fixture `JobRecords` built in the test (retry target environment-failed, no Producer invocation) checks `resumed` is true; the same with a Producer invocation gives false. Run: `cargo test -p athena-core scope_guard`. Observes: Resumed: `Level::None`. Resumed with spend over the remaining-stage budget: `Level::Pause`. Control with `resumed: false`: `Level::Pause`. `resumed` true only for a resumable target with no Producer invocation.
  Fails without the behavior: Without `resumed` and the T3 basis, the resumed line's total spend 1.5 reaches 1.5 x p90 and pauses, so the first assertion fails. Without the check that the next attempt resumed, the non-resuming control would not pause either.
- Criterion 3: Run the whole suite. Entry points are the existing production paths those tests already call: `Athena::ask` and `go` with `watch_line` in `watcher_tests` (`t4_pause_via_go_and_ask`, `retry_refuses_paused_line`, `t3_pause_from_reading_forecast`) and the `mod tests` cases on `evaluate_before_attempt`. Their fixtures and rig are unchanged; only the `att` helper gains `resumed: false`, so they act as an unchanged regression oracle for the new rule. These existing tests contain Rejected and plain Failed attempts and no environment reason. Run: `cargo test --locked`. Observes: All tests pass: a Rejected verification on the same criterion still pauses on T4, a paused line still refuses a retry, and a cost overrun made of non-environment attempts still pauses on T3 against the full p90.
  Fails without the behavior: If the new rule were too broad (for example Environment matched any `failed` job or Rejected were filtered like Environment, or the resumed scaling applied to every retry), `t4_pause_via_go_and_ask`, `retry_refuses_paused_line` or `t3_pause_from_reading_forecast` would no longer pause and fail their assertions. If the new variant or field broke a match or constructor, the suite would fail to compile.
- Criterion 4: Run the two checks Athena runs, as two whole commands: `cargo fmt --all --check`, then `cargo clippy --all-targets --locked -- -D warnings`. They operate on the final tree, including the new variant, field, constants, `t3_basis` and the new tests. Nothing is injected. Run: `cargo fmt --all --check`. Observes: Both exit 0 with no diff output and no warnings.
  Fails without the behavior: An unformatted edit makes `cargo fmt --all --check` exit non-zero with a diff. A clippy finding in the new code (an unused constant such as `PRODUCER_SHARE` that nothing reads, a match not covering `Verdict::Environment`, needless borrows in `counted`, or an unused helper) makes clippy with -D warnings exit non-zero. So the check fails if the change is left unformatted or leaves dead code.
- Criterion 5: Read the diff: one bullet under `## [Unreleased]` in the file's existing style. It names the scope guard, states that attempts ended by the environment (low disk, interruption) do not count toward the attempt, round and time signals, states that a retry resuming a kept candidate is forecast on its remaining stages, and names job 20261028-034820-v3rf as the case. Run: `git diff CHANGELOG.md`. Observes: The entry exists under Unreleased and names both changes.
- Criterion 6: Check the stat lists only the two files, then read `git diff crates/athena-core/src/scope_guard.rs`. Every new item (`PRODUCER_SHARE`, `VERIFIER_SHARE`, `ENVIRONMENT_PREFIX`, `Verdict::Environment`, `resumed`, `t3_basis`) has a doc comment stating its basis (`disk::LOW_DISK`, `resumable_stop`, the `cost_usd` forecast) or marking it an estimate. A unit test asserts `crate::disk::LOW_DISK.starts_with(ENVIRONMENT_PREFIX)`, tying the prefix to its source. Run: `git diff --stat`. Observes: Only crates/athena-core/src/scope_guard.rs and CHANGELOG.md are listed, and every new item carries a doc comment.

Planned files:

- crates/athena-core/src/scope_guard.rs (modify)
- CHANGELOG.md (modify)

## The change

Base f1b76f593bf2 → candidate f1788571c87b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |  15 ++
 crates/athena-core/src/scope_guard.rs | 254 +++++++++++++++++++++++++++++++---
 2 files changed, 253 insertions(+), 16 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1537 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check and cargo clippy --all-targets -- -D warnings are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
