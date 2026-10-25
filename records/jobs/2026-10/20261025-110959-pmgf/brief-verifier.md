# Job 20261025-110959-pmgf: verify "Wait for the early CI run before pushing the landing commit"

## Request (verbatim)

> Priority: urgent
> Cut A misses its cache by a race: the early pull request opens before Athena's checks (run.rs:1645), but its 'fmt, clippy, tests' CI run takes about 16 minutes while Athena's checks and verifier take about 14, so the landing commit is pushed and its CI run starts before the early run's pass is cached (PR #180: early run passed 10:46:49, landing run started 10:45:15 and ran the full 16 minutes). Before land pushes the landing commit, it waits for the early run's 'fmt, clippy, tests' on the candidate's commit to finish (with a bounded wait, for example 10 minutes, then it pushes anyway), so the landing run finds the cached pass. If the early run failed, land pushes as today and the full run decides.
> Acceptance:
> - A fake-host test shows land waiting for the candidate's check run to complete before pushing the landing commit, and pushing at once when it has already completed.
> - A test shows the bounded wait expiring pushes anyway, and a failed early run changes nothing about merging.
> - CHANGELOG.md records the next real landing's time, measured.
> Files: crates/athena-core/src/gh.rs, crates/athena-core/src/land.rs, CHANGELOG.md

## Reading (accepted by the user)

When land replaces the early-pushed head with the landing commit (gh.rs land_pr/replace_early), it first waits up to a bounded time (about 10 minutes) for the early commit's workflow run to complete, so the landing run hits the cached pass. If the wait expires or the early run failed, it pushes as today. Audience: Athena maintainers and anyone watching landing CI time.

Assumptions:
- The landing push the request describes is in gh.rs publish/land_pr via replace_early, not in land.rs, which holds planning and classification; land.rs is listed because the request names it, and likely stays unchanged.
- Completion is read from the existing PrHost::ci_runs (CiRunState status == completed) for the early commit. The request's 'fmt, clippy, tests' run is the commit's workflow run, so no new host call is needed; if per-run-name filtering is needed, gh.rs gets a small addition.
- Wait and sleep reuse the existing Poll sleep injection, with a new bounded-wait setting defaulting to 10 minutes.
- The criterion about the next real landing's time cannot be produced by the producer: it needs a landing that happens after this change. The CHANGELOG will say the measurement is pending, not carry a made-up number.

## Acceptance criteria

1. [behavior] A fake-host test (run with cargo test -p athena-core) shows land_pr, with the early head's CI run still in progress, polling ci_runs on the early commit and pushing the landing commit only after the run reports completed.
2. [behavior] A fake-host test shows that when the early run is already completed, the landing push happens at once with no sleep calls.
3. [behavior] A test shows that when the run never completes within the bounded wait, the landing commit is still pushed, and that the wait is bounded (default about 10 minutes, injectable so the test needs no real sleeping).
4. [behavior] A test shows that a failed (or timed-out) early run, and an early commit with no runs or a ci_runs error, change nothing about merging: land pushes and merges exactly as before and the full landing run decides.
5. [behavior] cargo test and cargo clippy --all-targets are clean, and cargo fmt --check passes; existing land_pr and publish tests still pass.
6. [content] The wait applies only when an early head exists and differs from the landing commit; the wait is logged or reported in a way a reader can see how long it waited and why it stopped.
7. [content] CHANGELOG.md has an entry for this change that says the next real landing's time is still to be measured. It must not state an invented number. The measured time is recorded only once a real landing has been observed, with its source (for example the PR number and the run timestamps).

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/test_support.rs (modify)

## Out of scope

- Changing CI workflows or the early-PR opening order at run.rs:1645
- Re-running or fixing failed early runs
- Changing merge method or the existing check-wait and re-run logic in wait_for_checks

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New fake-host tests; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check that push ordering and timing happen in fake-host tests, not that fields exist.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: CHANGELOG number must be measured and sourced, otherwise marked pending.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the landing path.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the race by waiting for the cache-producing run, not by shortening the CI.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Fake-host test: ci_runs on the early commit returns in_progress twice, then completed/success. A recording sleep and the fake remote share an event log. Run: `cargo test -p athena-core land_pr_waits_for_the_early_run_before_pushing`. Observes: The log shows three ci_runs calls on the early commit with sleeps between them, and the replace push logged after the last ci_runs call.
  Fails without the behavior: Without the wait the push is logged before any ci_runs call or sleep, so the ordering assertion fails.
- Criterion 2: Fake ci_runs returns a completed run on the first call. Run: `cargo test -p athena-core land_pr_pushes_at_once_when_the_early_run_is_completed`. Observes: One ci_runs call, zero sleep calls, the push happens, and the note says already completed.
  Fails without the behavior: A wait that sleeps before checking records a sleep and fails the zero-sleep assertion.
- Criterion 3: Fake host overrides early_run_wait to 60s; ci_runs always returns in_progress; recording sleep does not block. A second assertion checks the trait default is 10 minutes. Run: `cargo test -p athena-core land_pr_pushes_anyway_when_the_early_run_never_completes`. Observes: Recorded sleeps total at least 60s and at most 60s plus one step, the landing push still happens, and the note says timed out.
  Fails without the behavior: An unbounded wait never returns (the fake caps calls and fails); without the fall-through the landing commit is not pushed.
- Criterion 4: Table of cases (failed run, timed_out run, empty run list, ci_runs Err). Each runs land_pr through merge and compares merge calls and the report with a baseline run with no early-head wait. Run: `cargo test -p athena-core land_pr_early_run_failure_changes_nothing`. Observes: No sleeps, the push happens, merge is called with the landing commit, and the report matches the baseline apart from the note.
  Fails without the behavior: A wait that treats failure as pending sleeps to the bound; one that propagates the ci_runs error aborts the landing.
- Criterion 5: Run the whole suite, including the new tests of criteria 1-4 and the existing land_pr and publish tests, which stay unmodified because no struct literal changes. Then clippy and fmt. Run: `cargo test --locked`. Observes: All tests pass, then `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check` exit clean.
  Fails without the behavior: The new tests of criteria 1-4 are in this run and fail without the wait; a broken existing land_pr or publish test (for example one that now sleeps or errors on ci_runs) fails the same run.
- Criterion 6: Check the note for each stop reason, and cases with no early head and with early head equal to the landing commit. Run: `cargo test -p athena-core land_pr_reports_why_the_early_wait_stopped`. Observes: The note contains seconds waited and the reason (completed, already completed, no runs, ci_runs error, timed out). With no early head or equal heads there are zero ci_runs calls and no note.
  Fails without the behavior: A wait that runs unconditionally calls ci_runs in the no-early-head case and fails the zero-call assertion.
- Criterion 7: Inspect the added entry. Run: `git diff CHANGELOG.md`. Observes: The entry describes the wait and the race (PR #180: early run passed 10:46:49, landing run started 10:45:15), states the next real landing's time is still to be measured with no new number, and says to record it with its PR number and run timestamps once observed.

Planned files:

- crates/athena-core/src/gh.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/run.rs (modify)

## The change

Base 029f49018991 → candidate eddf20bdea8d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                 |   9 ++
 crates/athena-core/src/gh.rs | 306 +++++++++++++++++++++++++++++++++++++++++--
 2 files changed, 305 insertions(+), 10 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1222 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
