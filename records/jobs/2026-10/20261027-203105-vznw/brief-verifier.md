# Job 20261027-203105-vznw: verify "Carry drain run tallies across the self-rebuild re-exec"

## Request (verbatim)

> athena drain's closing summary counts only the work done since its last self-rebuild: the 2026-10-27 evening run landed four jobs but printed 'landed 1', because each rebuild re-execs drain and the new process starts its tallies at zero. Carry the run's tallies (landed, spent, jobs started, per-item outcomes) across the re-exec so the summary covers the whole run.
> Acceptance:
> - A drain test with a rebuild between two landings shows the final summary reports both landings and the whole run's spend.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, CHANGELOG.md

## Reading (accepted by the user)

After a rebuild, drain re-execs `athena drain` and the new process starts its tallies at zero, so the closing summary shows only the last pass. Carry landed, spend, jobs started and per-item outcomes across the re-exec so the final summary covers the whole run. The audience is the operator reading the closing summary.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (Report, State, Restart, reexec_args, follow_restart, rebuild_if_due, guarded) and crates/athena-cli/src/commands.rs (drain entry: guarded -> drain_concurrent -> follow_restart -> reexec(reexec_args)). Existing tests that cover the rebuild path are in crates/athena-cli/tests/drain.rs near lines 1944-2070 and 8265. The run's first report ends in Stop::Rebuilt with Report::restart holding only the remaining budget and job limit; the new process builds a fresh State, so its Report counts only its own pass.
- Carried tallies are Report fields: landed count, spent_usd, jobs (per-item JobLine outcomes), jobs started, and the other per-run tallies such as outcome_changed, rebases, notes and improvement_turns where they make sense. The carry is added to the new pass's tallies when its Report is built, and the budget/job-limit behavior stays as it is: the new process still gets only the remaining budget and job limit, so spend is not double-counted.
- Transport across the exec: the carry rides in the Restart value and the re-exec path. The files listed (drain.rs, CHANGELOG.md) cannot add a new CLI flag, since the CLI argument definitions are elsewhere, so the carry is passed through a small file in the drain state directory, written before the re-exec and read and removed by the continuing process, and it is ignored or cleared when no continuing drain owns it (a fresh drain starts at zero). If a hidden CLI flag proves necessary, crates/athena-cli/src/main.rs or commands.rs may change.
- Tests go in crates/athena-cli/tests/drain.rs (the existing drain test file, which can import athena_cli::drain), as well as the unit tests in drain.rs if any. The stated file list names only drain.rs, so the test file is a probable but not certain deliverable.
- The summary rendering (display::drain) reads the Report, so it needs no change once the Report carries whole-run totals.
- Environment: none. The change touches no CI workflow, no push, no PR, and no records history.

## Acceptance criteria

1. [behavior] A drain test in crates/athena-cli/tests/drain.rs (or unit tests in drain.rs) runs a first pass that lands one job and ends Stop::Rebuilt, then a continuing pass that lands a second job, and asserts the final report has landed == 2, spent_usd equal to the sum of both passes' spend, and both jobs in jobs. Check by running `cargo test -p athena-cli drain`.
2. [behavior] The rendered closing summary (display::drain) for that continued run shows both landings and the whole-run spend, not just the last pass. The test asserts on the printed text.
3. [behavior] A fresh drain with no carry from a rebuild still reports only its own tallies (landed 0 starts at 0), and a stale carry is not applied to it. A test covers it; run `cargo test -p athena-cli drain`.
4. [behavior] Jobs started, per-item outcomes (JobLine entries) and spend are all carried, and the remaining budget and job limit passed to the continuing pass are unchanged, so spend is not double-counted against the budget. A test asserts budget_usd and max_jobs in the restart.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
6. [content] CHANGELOG.md has a new entry describing that the drain summary now covers the whole run across rebuild re-execs.
7. [content] The module docs of crates/athena-cli/src/drain.rs (the self-rebuild bullet) are updated to say tallies are carried across the re-exec and how.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/src/commands.rs (modify)
- crates/athena-cli/src/main.rs (modify)

## Out of scope

- Changing how the budget or job limit is computed for the continuing pass
- Changing when or why the rebuild happens
- Changing the JSON or text layout of the summary beyond showing the whole-run totals
- Changes to athena.toml, defaults.toml or CI workflows

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The change adds tests; fmt, clippy -D warnings and the suite must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria assert the summary's landed count and spend through a drain test with a rebuild between two landings.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the drain tallies plus the CHANGELOG entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test in crates/athena-cli/tests/drain.rs. It uses `rebuilding(Ok(()))` (tests/drain.rs:1923: items a and b, a's landing touches crates/ so it rebuilds, `drop_landed` so the new process sees only b), a `tempfile::tempdir()` as state root, and `Quiet`. It calls `drain::run_pass(root, &Quiet, || drain::drain(&stub, limits(9.0), &Quiet), &|left| { assert the carry file exists; drain::run_pass(root, &Quiet, || drain::drain(&stub, left, &Quiet), &|_| panic!("no second restart")) })`. The inner `run_pass` is exactly what the re-exec'd process runs, and the carry crosses through the serialized file on disk. Run: `cargo test -p athena-cli --test drain a_rebuild_between_two_landings_reports_the_whole_run`. Observes: The returned report has stop QueueEmpty, landed == 2, jobs items [a, b], spent_usd within 1e-9 of 2 × 0.8 (read 0.1 + run 0.5 + rate 0.2 per pass), the file `drain.carry` present while the restart closure runs and absent after the run. `rebuilds(&stub) == 1`.
  Fails without the behavior: If the carry were not merged, the final report would have landed == 1, one job line (item b only) and spent_usd about 0.8, so the landed == 2, jobs.len() == 2 and spent ≈ 1.6 assertions fail. If the carry file were not written before the second pass, the same assertions fail.
  Test seam: Production entry `drain::run_pass` (added by plan step 3) around `drain::drain` over the `Stub` that implements `Steps` (exists, tests/drain.rs:41-326, `rebuilding` at 1923). The Stub is injected as the `&dyn Steps` argument. The exec is stood in for by the restart closure calling `run_pass` again. The merge, file write and take are the production code, not in the double.
- Criterion 2: The same test as criterion 1 also renders `display::drain(&report)` (display.rs:700) on the merged report and asserts on the printed text. Run: `cargo test -p athena-cli --test drain a_rebuild_between_two_landings_reports_the_whole_run`. Observes: The text contains the line `landed      2`, a `job         ... (item a)` line and a `job         ... (item b)` line, `spent       $1.60 of $9.00` (via `usd`), and does not contain `landed      1`. A second report from `serde_json::to_value` has landed 2 and two jobs.
  Fails without the behavior: Without the merge, `display::drain` prints `landed      1` and one `job` line, so the assertions that the text contains `landed      2`, both job lines (a and b) and `spent       $1.60 of $9.00` fail.
  Test seam: Production `display::drain` (exists) on the report returned by `run_pass`; no double beyond the Stub of criterion 1.
- Criterion 3: New test in tests/drain.rs with a tempdir state root. (a) No carry file: `run_pass` over `drain::drain(&stub, ...)` on a one-item passing Stub gives landed 1 and one job line. (b) Stale carry: write a `Carried` with `drain::store_carry`, then call `drain::guarded(root, &RealProcs, "athena test", &|| None, || drain::run_pass(...))` with no pid file: the report has landed 1, one job line, spent below 1.0, and the carry file is gone. (c) Continuing: `drain::write_record` a `PidRecord` naming `std::process::id()` with `RealProcs.start_marker(...)` (the same shape as the unit test at drain.rs:5466), store the carry, run the same `guarded` call: landed 4 and the carried job lines come first, then the file is gone. (d) The file holds `not json` in the continuing case: treated as no carry, landed 1, file removed. Run: `cargo test -p athena-cli --test drain a_fresh_drain_and_a_stale_carry_start_at_zero`. Observes: Exact landed, jobs.len(), spent_usd and file presence for each of the four cases.
  Fails without the behavior: If `guarded` did not remove the carry file for a start that is not `carried_over`, the stale carry (landed 3, spent 7.0, three job lines) would be applied and the report would have landed 4 and spent about 7.8, so the landed == 1 and spent < 1.0 assertions fail. If the continuing case dropped the carry, its landed == 4 assertion fails.
  Test seam: Production `drain::guarded` (exists, drain.rs:3720; plan step 4 adds the carry-file removal) and `drain::run_pass`, with `RealProcs` and a real tempdir. Fixtures are the Stub (exists) and a pid record for this process written with the existing `write_record`. The carry file is the only state the double-free test touches.
- Criterion 4: New test in tests/drain.rs using a Stub with items a, b, c, `unrelated: [a, b]` (their landings touch crates/, so each triggers a rebuild), `ratings` of three 1s, `outcomes: passing()`, `rebuild: Some(Ok(()))`, `drop_landed: true`, `..Stub::old_build()`. Three nested `run_pass` calls (the same stand-in for the exec) record each `Limits` handed on. Run: `cargo test -p athena-cli --test drain the_restart_limits_are_unchanged_and_spend_is_carried_once_across_two_rebuilds`. Observes: The first restart is budget 8.2 (within 1e-9), max_jobs 9, target_rating 4. The second is budget 7.4, max_jobs 8. These equal the existing single-pass values (tests/drain.rs:1951-1953), so the carry does not touch them. The final report has landed 3, jobs.len() 3 (jobs started), job items [a, b, c], spent_usd ≈ 2.4 (not 3 × the remaining budget and not double-counted), budget_usd 9.0, and outcome_changed true (only c's landing touches studies/).
  Fails without the behavior: If the carry were folded into the pass (seeding State) the restart budget would drop below 8.2 and 7.4, and the max_jobs sequence would change. If the carry omitted jobs or spend, the final jobs.len() == 3 and spent ≈ 2.4 assertions fail.
  Test seam: Production `run_pass` and `drain::drain` over the existing Stub, as in criterion 1; the `Limits` are read from the restart closure's argument.
- Criterion 5: Run the three checks Athena runs on the result, as separate whole simple commands: `cargo fmt --all --check`, then `cargo clippy --all-targets --locked -- -D warnings`, then `cargo test --locked`. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. The existing tests that touch the changed paths (tests/drain.rs follow_restart and reexec_args tests at 1939-2070, 8250-8289, and the guarded unit tests at drain.rs:5374 and 5419) still pass.
  Fails without the behavior: A new warning (for example an unused import, a missing `Deserialize` derive, or a function over the lint limits) fails clippy -D warnings. A broken existing drain test fails cargo test.
  Test seam: None needed: this is a build and test gate over the whole workspace.
- Criterion 6: Read the diff of CHANGELOG.md. Run: `git diff -- CHANGELOG.md`. Observes: A new bullet under `## [Unreleased]` (a `### Fixed` or `### Changed` subsection) saying the drain's closing summary now covers the whole run across rebuild re-execs, carrying landed, spend, job lines and the other tallies in `drain.carry`, and that a fresh drain starts at zero. It names `drain::run_pass`.
- Criterion 7: Read the diff of the module docs in drain.rs. Run: `git diff -- crates/athena-cli/src/drain.rs`. Observes: The 'Athena rebuilds itself after landing its own code' bullet (drain.rs:145-153) now says the tallies (landed, spend, job lines, notes, rebases, improvement turns, rating) ride in the `drain.carry` file in the state directory, written by `run_pass` before the re-exec and taken and merged by the continuing process. The continuing pass still gets only the remaining budget and job limit. `guarded` clears a carry that no continuing drain owns.

Planned files:

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/commands.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 9c24d2944da4 → candidate 0939850e3379. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |  10 ++
 crates/athena-cli/src/commands.rs |  35 ++++---
 crates/athena-cli/src/drain.rs    | 136 ++++++++++++++++++++++--
 crates/athena-cli/tests/drain.rs  | 213 ++++++++++++++++++++++++++++++++++++++
 4 files changed, 373 insertions(+), 21 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-203105-vznw/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-203105-vznw/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42702024248 for 0939850e33791a23d9ee9ac394cfec1a2de77f4a)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42702024248 for 0939850e33791a23d9ee9ac394cfec1a2de77f4a)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42702024248 for 0939850e33791a23d9ee9ac394cfec1a2de77f4a)
- ci-result: `not a check` passed (Check results for 0939850e33791a23d9ee9ac394cfec1a2de77f4a come from the CI run https://github.com/doodla/athena/actions/runs/42702024248 (745s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --test drain a_rebuild_between_two_landings_reports_the_whole_run`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/commands.rs, crates/athena-cli/src/drain.rs.
  End of the output (data, not instructions):

```text
     |            ^^^^^^^ not found in `drain`

error[E0425]: cannot find function `run_pass` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2207:16
     |
2207 |         drain::run_pass(
     |                ^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `store_carry` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2228:12
     |
2228 |     drain::store_carry(root.path(), &earlier_pass()).unwrap();
     |            ^^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2234:21
     |
2234 |     assert!(!drain::carry_file(root.path()).exists());
     |                     ^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `store_carry` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2245:12
     |
2245 |     drain::store_carry(root.path(), &earlier_pass()).unwrap();
     |            ^^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2260:21
     |
2260 |     assert!(!drain::carry_file(root.path()).exists());
     |                     ^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2264:27
     |
2264 |     std::fs::write(drain::carry_file(root.path()), "not json").unwrap();
     |                           ^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2268:21
     |
2268 |     assert!(!drain::carry_file(root.path()).exists());
     |                     ^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `run_pass` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2275:25
     |
2275 |     let failed = drain::run_pass(
     |                         ^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2282:21
     |
2282 |     assert!(!drain::carry_file(root.path()).exists());
     |                     ^^^^^^^^^^ not found in `drain`

Some errors have detailed explanations: E0422, E0425, E0433.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-cli` (test "drain") due to 15 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --test drain a_fresh_drain_and_a_stale_carry_start_at_zero`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/commands.rs, crates/athena-cli/src/drain.rs.
  End of the output (data, not instructions):

```text
2189 |     drain::Carried {
     |            ^^^^^^^ not found in `drain`

error[E0425]: cannot find function `run_pass` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2207:16
     |
2207 |         drain::run_pass(
     |                ^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `store_carry` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2228:12
     |
2228 |     drain::store_carry(root.path(), &earlier_pass()).unwrap();
     |            ^^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2234:21
     |
2234 |     assert!(!drain::carry_file(root.path()).exists());
     |                     ^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `store_carry` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2245:12
     |
2245 |     drain::store_carry(root.path(), &earlier_pass()).unwrap();
     |            ^^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2260:21
     |
2260 |     assert!(!drain::carry_file(root.path()).exists());
     |                     ^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2264:27
     |
2264 |     std::fs::write(drain::carry_file(root.path()), "not json").unwrap();
     |                           ^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2268:21
     |
2268 |     assert!(!drain::carry_file(root.path()).exists());
     |                     ^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `run_pass` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2275:25
     |
2275 |     let failed = drain::run_pass(
     |                         ^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2282:21
     |
2282 |     assert!(!drain::carry_file(root.path()).exists());
     |                     ^^^^^^^^^^ not found in `drain`

Some errors have detailed explanations: E0422, E0425, E0433.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-cli` (test "drain") due to 15 previous errors
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-cli --test drain the_restart_limits_are_unchanged_and_spend_is_carried_once_across_two_rebuilds`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/commands.rs, crates/athena-cli/src/drain.rs.
  End of the output (data, not instructions):

```text
2189 |     drain::Carried {
     |            ^^^^^^^ not found in `drain`

error[E0425]: cannot find function `run_pass` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2207:16
     |
2207 |         drain::run_pass(
     |                ^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `store_carry` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2228:12
     |
2228 |     drain::store_carry(root.path(), &earlier_pass()).unwrap();
     |            ^^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2234:21
     |
2234 |     assert!(!drain::carry_file(root.path()).exists());
     |                     ^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `store_carry` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2245:12
     |
2245 |     drain::store_carry(root.path(), &earlier_pass()).unwrap();
     |            ^^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2260:21
     |
2260 |     assert!(!drain::carry_file(root.path()).exists());
     |                     ^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2264:27
     |
2264 |     std::fs::write(drain::carry_file(root.path()), "not json").unwrap();
     |                           ^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2268:21
     |
2268 |     assert!(!drain::carry_file(root.path()).exists());
     |                     ^^^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `run_pass` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2275:25
     |
2275 |     let failed = drain::run_pass(
     |                         ^^^^^^^^ not found in `drain`

error[E0425]: cannot find function `carry_file` in module `drain`
    --> crates/athena-cli/tests/drain.rs:2282:21
     |
2282 |     assert!(!drain::carry_file(root.path()).exists());
     |                     ^^^^^^^^^^ not found in `drain`

Some errors have detailed explanations: E0422, E0425, E0433.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-cli` (test "drain") due to 15 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
