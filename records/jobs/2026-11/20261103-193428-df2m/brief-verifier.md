# Job 20261103-193428-df2m: verify "Don't count needs-accepting re-reads toward the drain job limit"

## Request (verbatim)

> Don't spend a batch's job slots on re-reads that end in 'needs accepting'. A batch with $22 and 4 job slots left just ended with nothing landed: all 4 slots went to old requests (from 10-16 Oct) that were read again after a settings change and came back 'needs accepting' at $0.06-0.18 each. A re-read that only produces 'needs accepting' should not count toward --job-limit, and the batch should go on to the next item. Acceptance: a test where a batch with job limit 1 meets a re-read that needs accepting and then still runs the next item.

## Reading (accepted by the user)

When a drain re-reads an old request after a settings change and the new reading needs accepting, that attempt must not use one of the --job-limit slots, so the drain goes on to the next item. Audience: the operator running `athena drain` with a job limit and budget. The fix is in the drain's job counter, with a regression test using job limit 1.

Assumptions:
- Code survey: crates/athena-cli/src/drain.rs. In `run_job` (~line 1418) the order is: check stop, check `st.started >= limits.max_jobs` (JobLimit), check budget, then `st.started += 1`. It then reads or re-reads (`Source::Reread` calls `steps.reread`), then `steps.run`. `run.needs_accepting` yields `RunStep::NeedsAccepting` (~line 1537), which `attempts` maps to `ItemEnd::NeedsAccepting`, and `finish` passes the item over (~line 2411). Nothing gives the slot back, so the next item hits JobLimit. `approve_all` (~1889), `prepare` (~2008), `pick_ahead` (~2555) and the restart arithmetic (~2896) also read `st.started`. The real needs-accepting source is RealSteps::run (~line 3566). Fix: release the slot (`st.started -= 1`, saturating) when the step ends as NeedsAccepting. Money spent stays counted in the budget. The job-limit check is the one place that measures the limit.
- Code survey: crates/athena-cli/tests/drain.rs. The existing `a_run_that_needs_accepting_leaves_the_item_queued_without_a_failure` (~1305) and `a_reread_whose_scope_changed_is_left_for_a_person_and_is_not_a_failure` (~2933) cover the outcome with a stub that sets `needs_accepting`. `the_job_limit_stops_the_loop` (~1422) shows how to set `max_jobs: 1`. The new test reuses that Stub pattern: item a needs accepting, item b is runnable.
- No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. No refusal rule is added or tightened, so no refusal-rate criterion applies.
- Only the NeedsAccepting outcome releases its slot. Failed, held and passed attempts keep counting, and the job-limit stop for those is unchanged.
- A change to docs or CHANGELOG is optional and not required. athena.toml, defaults.toml and workflows need no change.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs (Grep for `needs_accepting` near `max_jobs: 1`) drains with job limit 1 where item `a` ends as needs accepting and item `b` is runnable. It asserts `jobs[0].outcome == "needs accepting"`, `landed == 1`, and `stop != Stop::JobLimit`. Run `cargo test -p athena-cli --test drain` and it passes.
2. [behavior] The same test, run against the unfixed `run_job` counter (as in the pre-change source in `git diff`), would stop with `Stop::JobLimit` before `b` runs. The assertions show it fails without the fix: `b` is asserted to land, which the old counter prevents.
3. [content] `git diff` of crates/athena-cli/src/drain.rs shows the `RunStep::NeedsAccepting` path in `run_job` giving the started-job slot back (a decrement of `st.started` under the state lock). The JobLimit check and the other outcomes (Failed, Held, Passed) are unchanged.
4. [behavior] A job limit that still binds for ordinary jobs: `cargo test -p athena-cli --test drain` still passes `the_job_limit_stops_the_loop`, `an_urgent_queued_request_is_worked_before_an_approved_in_limit_proposal`, and the two existing needs-accepting tests, with no existing assertions weakened (`git diff` shows no removed asserts in them).
5. [behavior] `cargo test` passes for the workspace and `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean.
6. [content] Doc comments or help text in crates/athena-cli/src/drain.rs that describe the job limit (the `--job-limit` help and the module docs near ~line 180) say a re-read that only needs accepting does not count toward it. A Grep for that wording finds it.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Avoiding the re-read itself, or changing which config changes trigger a re-read.
- Changing the money budget accounting; the cost of the re-read still counts against the budget.
- Changing how the other outcomes (failed, held, budget short) count toward the job limit.
- Auto-accepting readings whose scope differs.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New regression test plus cargo test and clippy clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the slot counting in run_job, not a special case for old requests.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criterion runs a drain with job limit 1 and checks that the next item lands.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a batch lands the next item after a needs-accepting re-read.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test on the drain tests.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused counter change with one test.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the new test. It calls the production `drain::drain` with `Limits { max_jobs: 1, ..limits(9.0) }` and checks the report. Run: `cargo test -p athena-cli --test drain a_needs_accepting_reread_does_not_use_a_job_slot`. Observes: The test passes. It asserts `report.jobs[0].outcome == "needs accepting"`, `report.landed == 1`, `report.stop != Stop::JobLimit`, and that the stub log has `run a`, then `run b`, then `land b`.
  Fails without the behavior: Without the decrement, `st.started` is 1 after a, so b's `run_job` returns `RunStep::JobLimit` at line 1433. The drain stops with `Stop::JobLimit` and `landed == 0`, so the landed and stop assertions fail.
  Test seam: Production entry point `drain::drain(&dyn Steps, Limits, &Ui)`, which reaches `run_job`. Fixture: the existing `Stub` in tests/drain.rs, injected as `&stub`, with items a and b, `needs_accepting: vec!["a"]`, `outcomes: passing()` and `ratings: RefCell::new(vec![3])`. The stub only returns the `Run` the real step would; the slot counting runs in production `run_job`. Exists: I read `Stub::run` (~223) and `the_job_limit_stops_the_loop` (~1422). Step 2 adds only the test function.
- Criterion 2: Temporarily remove the decrement in the working tree, run the new test and see it fail, then restore the decrement and see it pass. The failing run is shown by the producer in its report. The verifier can repeat it, or read that the test's assertions (`landed == 1`, `stop != Stop::JobLimit`) cannot hold under the old counter. Run: `cargo test -p athena-cli --test drain a_needs_accepting_reread_does_not_use_a_job_slot`. Observes: With the decrement removed the test fails with `landed == 0` and `Stop::JobLimit`. With it restored the test passes.
  Fails without the behavior: The old counter keeps the slot, so b is refused at line 1433 and the landed and stop assertions fail.
  Test seam: Same as criterion 1: `drain::drain` with the `Stub` injected as `&stub`. Exists today. The removal is a temporary edit to `run_job` made by the producer in step 5, not a double.
- Criterion 3: Read the diff of drain.rs. Run: `git diff -- crates/athena-cli/src/drain.rs`. Observes: In the `_ if run.needs_accepting` arm of `run_job`, the diff shows a lock of the state (`self.st()`) and `st.started = st.started.saturating_sub(1)`. The `if st.started >= self.limits.max_jobs` check, the `st.started += 1` line, and the Passed, Held and `other` arms show no change.
- Criterion 4: Run the whole drain test file, then read the diff of tests/drain.rs to check that it only adds lines. Run: `cargo test -p athena-cli --test drain`. Observes: `the_job_limit_stops_the_loop`, `an_urgent_queued_request_is_worked_before_an_approved_in_limit_proposal`, `a_run_that_needs_accepting_leaves_the_item_queued_without_a_failure`, `a_reread_whose_scope_changed_is_left_for_a_person_and_is_not_a_failure` and `passed_over_item_is_logged_once_with_its_reason` all pass. `git diff -- crates/athena-cli/tests/drain.rs` shows no removed `assert` lines.
  Fails without the behavior: If the release were applied to every outcome, `the_job_limit_stops_the_loop` (Passed outcomes, max_jobs 1) would not stop at `Stop::JobLimit` and would fail. So these tests show the limit still binds for ordinary jobs.
  Test seam: Production `drain::drain` with the existing `Stub` injected as `&stub`, and the `Fixture`/`RealSteps` path (RealSteps::run, drain.rs ~3566) for the real-step tests. Both exist today; no step adds a seam.
- Criterion 5: Run the workspace gates in turn: `cargo test --locked`, then `cargo clippy --all-targets --locked -- -D warnings`, then `cargo fmt --all --check`. The test run includes the new test, which exercises the production `drain::drain` path with the `Stub`. Run: `cargo test --locked`. Observes: Every test passes, including `a_needs_accepting_reread_does_not_use_a_job_slot` and the budget and doc-scan tests. Clippy and fmt report nothing.
  Fails without the behavior: Without the decrement the new test fails, so `cargo test --locked` goes red with `landed == 0` and `Stop::JobLimit`. A badly formatted or lint-failing change fails the clippy and fmt commands.
  Test seam: The workspace test run reaches the production `drain::drain` through the `Stub` in tests/drain.rs, injected as `&stub`, and through the existing `RealSteps` tests. The seam exists today (read in tests/drain.rs and drain.rs); the new test is added in step 2.
- Criterion 6: Read the diff, then Grep for `does not count` in drain.rs and main.rs. Run: `git diff -- crates/athena-cli/src/drain.rs crates/athena-cli/src/main.rs`. Observes: The diff adds wording that a re-read that only needs accepting does not count toward the job limit: in the drain.rs module docs (the configuration-change bullet ~176-181), in the `Limits::max_jobs` doc (~253), and in main.rs in the `job limit` stop-reasons line (~256) and the `--job-limit` doc (~288). Grep finds the phrase in both files.

Planned files:

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8e4d74266164 → candidate d1172e67b540. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  5 +++++
 crates/athena-cli/src/drain.rs   | 12 ++++++++++--
 crates/athena-cli/src/main.rs    |  5 +++--
 crates/athena-cli/tests/drain.rs | 32 ++++++++++++++++++++++++++++++++
 4 files changed, 50 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41728184733 for d1172e67b5406bc70da72611680f1e7b1e5ba9b4)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41728184733 for d1172e67b5406bc70da72611680f1e7b1e5ba9b4)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41728184733 for d1172e67b5406bc70da72611680f1e7b1e5ba9b4)
- ci-result: `not a check` passed (Check results for d1172e67b5406bc70da72611680f1e7b1e5ba9b4 come from the CI run https://github.com/doodla/athena/actions/runs/41728184733 (613s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test` passes for the workspace and `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
