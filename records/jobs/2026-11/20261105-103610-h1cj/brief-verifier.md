# Job 20261105-103610-h1cj: verify "Read ahead queued items while a single job runs, up to the limit"

## Request (verbatim)

> Read and plan the next queued requests while the current job runs. Reading and planning are model calls with no build, but today a batch reads and plans an item only when it reaches it. While jobs run, read and plan the next queued items ahead, up to a limit, so a job is ready to dispatch when a slot frees.
> 
> Files changed: crates/athena-cli/src/drain.rs and its tests.
> 
> Acceptance:
> - a test where a batch runs one job and has three queued items: the readings of the next items are made while the first job runs
> - a test where the look-ahead limit is 1: only one item is read ahead
> - an item whose look-ahead reading is refused is reported as today, and the batch still runs its current job

## Reading (accepted by the user)

Read-ahead (reading and, for hard jobs, planning the next queued items while a job runs) already exists in crates/athena-cli/src/drain.rs, but only for two or more concurrent jobs. It is off for `drain` and for a bound of 1, so a batch running one job reads each next item only when it reaches it. I will make read-ahead work with one job at a time, limited by `Steps::read_ahead`, so the next job is ready when the slot frees. The audience is the person running a batch and the verifier who runs the tests.

Assumptions:
- Code survey: I read crates/athena-cli/src/drain.rs. The entry points are `drain` (calls run_serial) and `drain_concurrent` (calls run_serial when workers == 1, else run_parallel). `drain_with` plans, takes batches_for(...).remove(0), truncates it to one item when one_at_a_time, marks it tried, then calls run(d, vec![now], rest). `run_parallel` returns early to run_batches when `window == 0 || workers < 2`; otherwise it spawns `read_ahead_pump`. `pick_ahead` chooses items in later batches. `read_ahead_pump` calls `Steps::read`, then `plan_ahead`, and stores an `Ahead`. A refused or erroring reading is dropped with a note and is read again by `claim`/`attempts` when the item starts. Reading holds `caps.read` against the budget.
- The change is in drain.rs and its tests only. The module doc, which says nothing is read ahead with one job at a time, must be updated. The existing test `read_ahead_is_off_with_a_bound_of_one` (read_ahead_tests module) asserts the old behavior for bound 1 and must change. Its window-0 case stays valid.
- Design: when only one job runs at a time and `Steps::read_ahead()` > 0, the pump runs beside the single worker. When drain_with truncates a batch to one item, the dropped remainder is passed on in `later` so it can be read ahead. The pump keeps its existing guards: window, job limit, budget fit for a read plus a run, disjoint touched paths, and stop. `Steps::read_ahead()` is the look-ahead limit, so no new config is needed. The default stays 0 in the trait, and the real steps do not turn it on, so athena.toml is not touched.
- No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. The change adds no refusal rule and does not tighten any, so no newly-refused-readings percentage applies. A reading refused during look-ahead is not a new refusal: it is reported as today, when the item starts.
- Tests use the existing `World` fake in read_ahead_tests, which logs `read x`, `plan_ahead x`, `land x`, and gates calls. The property is measured through the log order of `Steps` calls and the report, not through stand-ins like flags.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/src/drain.rs: one job at a time (drain, or drain_concurrent with bound 1), a queue of one running job plus three more queued items, window 3. While the first job is gated in `run`, the log shows `read` for the next items before `land` of the first job. Verified with `cargo test -p athena-cli read_ahead`.
2. [behavior] A new test with look-ahead limit 1 and the same queue: while the first job runs, exactly one later item is read (`world.count("read c") == 0` and `read b` is logged, or the equivalent for the fake's item names), and the next is read only after the held reading is claimed. Verified with `cargo test -p athena-cli read_ahead`.
3. [behavior] A new test where the look-ahead reading of a queued item is refused (not runnable or an error) while the first job runs: the batch still lands the first job, the refused item is reported as it is today (same line, or same stop or reading-refused outcome as when read at its turn), and the pump's drop note appears in `report.notes`. Verified with `cargo test -p athena-cli read_ahead`.
4. [behavior] The existing read-ahead tests still pass, with `read_ahead_is_off_with_a_bound_of_one` rewritten so that window 0 still means no reading ahead for drain and for drain_concurrent(1). Every item is read after the job before it landed, and `plan_ahead` is never called. Verified with `cargo test -p athena-cli` and `cargo clippy --all-targets -- -D warnings`.
5. [content] The module doc in crates/athena-cli/src/drain.rs (the Read-ahead bullet) no longer says nothing is read ahead with one job at a time. It says read-ahead applies there too and is bounded by `Steps::read_ahead`. `cargo fmt --check` passes.
6. [content] Only crates/athena-cli/src/drain.rs changes, plus an optional CHANGELOG.md line if the repository's convention requires one. No governance file (athena.toml, defaults.toml, workflows) is changed.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Turning read-ahead on in the real steps or setting a non-zero default for `read_ahead` in athena.toml
- Changing how readings are refused, or the refusal rules
- Changing landing, rating, or rebase behavior
- Changes outside crates/athena-cli/src/drain.rs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in drain.rs.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Three new tests; cargo test and clippy must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo test commands the verifier runs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the order of read, run, and land calls in the logs, not the existence of wiring.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to one file.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The module doc and test comments are updated for an outside reader.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The look-ahead limit is enforced by the existing `pick_ahead` window check and covered by a test.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that the next job's reading exists before the slot frees, not that code was added.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Two new tests in `read_ahead_tests`. Test A, `read_ahead_reads_the_next_items_while_a_single_job_runs`: a chain a, b, c, d (each depends on the one before, each touches its own file), `World::new(3, ..)`, `gate("a", ["read b","read c","read d"])`. Test B, `read_ahead_reads_independent_items_while_a_single_job_runs`: a, b, c, d with no dependencies, each touching only its own file (and no shared directory), `World::new(3, ..)`, the same gate. Both run through `concurrent(&world, 1)`. Test B has a comment saying the items sit in one plan group, so only the `drain_with` split-off lets the pump see them. Run: `cargo test -p athena-cli read_ahead_reads_the_next_items_while_a_single_job_runs && cargo test -p athena-cli read_ahead_reads_independent_items_while_a_single_job_runs`. Observes: In both tests the log shows `read b`, `read c` and `read d` before `end a` and before `land a`, and each item is read exactly once. `report.landed == 4` and `world.widest <= 4`. Later jobs run on the readings made ahead, so no `read` entry appears a second time.
  Fails without the behavior: Without the `drain_concurrent` change, bound 1 calls `run_serial`, no pump exists, and `read b` appears only after `land a`, so `at("read b") < at("end a")` fails (the gate gives up after 5 s). Without the `drain_with` split-off, test B fails while test A still passes: the independent items are dropped from `later`, so the reads come only after `end a`.
  Test seam: Production entry point: `drain_concurrent(&world, limits(20.0), &Quiet, 1)` via the existing `concurrent(&world, 1)` helper (drain.rs:4745), which reaches `drain_with`, `run_parallel` and `read_ahead_pump`. Guards: window `World.window` through `Steps::read_ahead`, job limit 20, budget fit (read 0.5 + run 1.0 against 20.0), `paths_disjoint`, and the unstarted-jobs rule. Fake: the `World` struct (drain.rs:4470), passed as `&(dyn Steps + Sync)`; its `gate`, `log`, `at` and `count` helpers exist. The new tests are added in plan step 7.
- Criterion 2: New test `read_ahead_with_a_limit_of_one_reads_one_item_ahead`. The chain a, b, c, d with `World::new(1, ..)`, `gate("a", ["read b"])` and `gate("b", ["read c"])`, run through `concurrent(&world, 1)`. The gate keeps a running 200 ms after `read b` appears, which is long enough for a pump that ignored the limit to read c as well. Run: `cargo test -p athena-cli read_ahead_with_a_limit_of_one_reads_one_item_ahead`. Observes: The entries before `end a` that start with `read ` are exactly `read a` and `read b`, so `world.count("read c") == 0` at that point. `at("end a") < at("read c")`, so c is read only after b's reading was claimed. `at("read c") < at("end b")`, so c is read while b runs. `world.widest <= 2`, `report.landed == 4`, and each item is read once.
  Fails without the behavior: Without the change, nothing is read ahead and `read b` is missing before `end a`. With a pump that ignored `held >= window`, `read c` would also appear before `end a` and the exact-reads assertion fails.
  Test seam: Production entry point: `drain_concurrent` with bound 1 through `concurrent(&world, 1)`. The guard under test is `held >= window` in `pick_ahead` (drain.rs:2656), with `st.ahead` carried between waves. Fake: the existing `World` with `window` 1, injected as `&(dyn Steps + Sync)`. Plan step 7 adds the test.
- Criterion 3: New test `read_ahead_of_a_refused_reading_is_reported_as_when_it_starts`. Items a, b, c where b and c depend only on a and touch their own files. A new `World.refuses` holds `b`; the fake's `read` returns `runnable: false` and an error for it (still logging `read b`). `gate("a", ["read b"])`. The test runs one world with window 1 and an identical world with window 0, both through `concurrent(&world, 1)`. Run: `cargo test -p athena-cli read_ahead_of_a_refused_reading_is_reported_as_when_it_starts`. Observes: With window 1, `read b` is logged before `end a`, and `land a` is logged and `report.landed >= 1`, so the batch still runs its current job. `report.notes` has the drop note 'the reading made ahead of b was dropped'. The job line for b (`outcome`, `reason`, `landing`) and the skipped or refused note for b are the same as in the window 0 report, which has no drop note. The test comment states what the producer observed for b's outcome, since the code does not settle it.
  Fails without the behavior: Without the change, no reading is made ahead, so `read b` is not before `end a` and the drop note is absent. If a refused look-ahead reading stopped the batch, `land a` would be missing.
  Test seam: Production entry point: `drain_concurrent` with bound 1. The refusal handling is production code: the pump (drain.rs:2721-2770) and `run_job`/`claim`. Fake: `World` plus the new `refuses` field and its branch in `read`, which only returns a refused `Read` and holds no drain logic. Plan step 6 adds the field; step 7 adds the test.
- Criterion 4: Rewrite `read_ahead_is_off_with_a_bound_of_one` (drain.rs:5025) as `read_ahead_is_off_with_a_window_of_zero`. Cases: `drain_concurrent` bound 1 with window 0, `drain` with window 0, bound 2 with window 0, and `drain` with window 1 (comment: `drain` is serial because its steps are not `Sync`, so it never reads ahead). Each case asserts `land a < read b`, `land b < read c`, `plan_ahead b` never called and no stale note. Then run the whole suite and clippy. Run: `cargo test -p athena-cli && cargo clippy --all-targets --locked -- -D warnings`. Observes: All tests pass: the existing read-ahead and stale-reading tests, the tests that call `drain` with stubs, and the `drain_concurrent` bound-1 tests in crates/athena-cli/tests/drain.rs. Clippy reports no warnings.
  Fails without the behavior: If the pump ran with a window of 0, `read b` would come before `land a` and `plan_ahead b` would be logged, so the rewritten test fails. If the change broke the 2-worker path, `read_ahead_reads_and_plans_the_next_hard_job_while_the_current_one_runs` and the stale-reading tests fail.
  Test seam: Production entry points `drain` (drain.rs:2778) and `drain_concurrent` (drain.rs:2789). Fake: the existing `World`. The test exists and plan step 8 edits it.
- Criterion 5: Read the module doc Read-ahead bullet at the top of drain.rs, and the `drain_concurrent` doc, and run the format check. Run: `cargo fmt --all --check`. Observes: The fmt check exits 0. The bullet's first sentence no longer says read-ahead needs more than one job. It says [`drain_concurrent`] reads ahead when [`Steps::read_ahead`] is above 0, with one job at a time too, bounded by `read_ahead`. It says `drain` reads each item at its turn because its steps are not `Sync`. It no longer says that with one job at a time nothing is read ahead. The `drain_concurrent` doc says a bound of 1 runs like `drain` except that it reads ahead when `Steps::read_ahead` is above 0.
- Criterion 6: List the changed files after the producer run. Run: `git status && git diff --stat`. Observes: Only crates/athena-cli/src/drain.rs and CHANGELOG.md are modified. athena.toml, defaults.toml, workflows and Cargo files are not.

Planned files:

- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 43b367459c3d → candidate d4907149927d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                   |   6 ++
 crates/athena-cli/src/drain.rs | 229 +++++++++++++++++++++++++++++++++++++----
 2 files changed, 215 insertions(+), 20 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-103610-h1cj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-103610-h1cj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42071793087 for d4907149927d200a0521bd8dbba565745bfc9a3f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42071793087 for d4907149927d200a0521bd8dbba565745bfc9a3f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42071793087 for d4907149927d200a0521bd8dbba565745bfc9a3f)
- ci-result: `not a check` passed (Check results for d4907149927d200a0521bd8dbba565745bfc9a3f come from the CI run https://github.com/doodla/athena/actions/runs/42071793087 (629s): fmt success, clippy success, test success.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test -p athena-cli read_ahead_reads_the_next_items_while_a_single_job_runs && cargo test -p athena-cli read_ahead_reads_independent_items_while_a_single_job_runs`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli read_ahead_with_a_limit_of_one_reads_one_item_ahead`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs.
  End of the output (data, not instructions):

```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.00s

     Running tests/review.rs (<job>/target/verify/debug/deps/review-e2b31eef9d3c5417)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s

     Running tests/status_build_measurement.rs (<job>/target/verify/debug/deps/status_build_measurement-86be37d815dff1dd)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

     Running tests/status_durations.rs (<job>/target/verify/debug/deps/status_durations-a24da118fb9e96d2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

     Running tests/status_snapshot.rs (<job>/target/verify/debug/deps/status_snapshot-8e4f5018c0b5ea3b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/steward.rs (<job>/target/verify/debug/deps/steward-98cf1b798d3221f9)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

     Running tests/trailer_stop.rs (<job>/target/verify/debug/deps/trailer_stop-985b0e3d0a0c45f2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/version.rs (<job>/target/verify/debug/deps/version-55574b07738a66c3)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/watcher.rs (<job>/target/verify/debug/deps/watcher-5380c34aed879c72)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/work_decisions.rs (<job>/target/verify/debug/deps/work_decisions-a47b7f34852351f5)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-cli --lib`
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli read_ahead_of_a_refused_reading_is_reported_as_when_it_starts`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs.
  End of the output (data, not instructions):

```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.00s

     Running tests/review.rs (<job>/target/verify/debug/deps/review-e2b31eef9d3c5417)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s

     Running tests/status_build_measurement.rs (<job>/target/verify/debug/deps/status_build_measurement-86be37d815dff1dd)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

     Running tests/status_durations.rs (<job>/target/verify/debug/deps/status_durations-a24da118fb9e96d2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

     Running tests/status_snapshot.rs (<job>/target/verify/debug/deps/status_snapshot-8e4f5018c0b5ea3b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/steward.rs (<job>/target/verify/debug/deps/steward-98cf1b798d3221f9)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 0.00s

     Running tests/trailer_stop.rs (<job>/target/verify/debug/deps/trailer_stop-985b0e3d0a0c45f2)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/version.rs (<job>/target/verify/debug/deps/version-55574b07738a66c3)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/watcher.rs (<job>/target/verify/debug/deps/watcher-5380c34aed879c72)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/work_decisions.rs (<job>/target/verify/debug/deps/work_decisions-a47b7f34852351f5)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-cli --lib`
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-cli && cargo clippy --all-targets --locked -- -D warnings`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
