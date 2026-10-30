# Job 20261030-013143-as0j: verify "Add drain in-flight table with live worktree and real stage hooks"

## Request (verbatim)

> Priority: urgent
> Async drain build job A3 (docs/design/async-drain.md, decision D145), third attempt. Add the in-flight table (job 3): a local file of job id, stage and worktree, written at each stage change and read at drain start. Depends on 20261029-142849-04d9. With max_in_flight = 1 drain behaves as today.
> Lessons from the two rejected attempts (20261029-142853-x4nr, 20261030-005008-tf2s):
> - The worktree recorded must be the producer's live work tree, not the kept-work directory used for timed-out jobs.
> - The stage must come from where the run actually enters each stage (producer, checks, verifier, landing), never by matching a display or progress string; when the producer is sent back after checks or the verifier, the table must say producer again.
> Acceptance:
> - Tests through the real entry point show the row's worktree is the producer's live work tree, and that a send-back moves the row from checks (or verifier) back to producer.
> - With max_in_flight = 1 the existing drain tests pass unchanged.
> - fmt, clippy with -D warnings and the test suite pass.
> - CHANGELOG.md has an entry, and any wording in docs/design/async-drain.md about stage changes matches the behavior.
> Files: crates/athena-cli/src/drain.rs, crates/athena-core/src/state.rs, crates/athena-core/src/run/producer.rs, CHANGELOG.md

## Reading (accepted by the user)

Build job A3 (third attempt) adds a local in-flight table (job id, stage, worktree) written at each stage change and read at drain start. It fixes the findings of rejected jobs 20261029-142853-x4nr and 20261030-005008-tf2s: the live work tree is recorded, and stages come from the run pipeline's own entry points. A send-back returns the row to producer. Audience: later crash-recovery jobs and the drain verifier.

Assumptions:
- Code survey: read crates/athena-cli/src/drain.rs (Steps trait ~line 423, RealSteps::run at ~2353 calling Athena::go, RealSteps::land ~2387, Stub/World test steps), crates/athena-core/src/run/mod.rs (Athena::go at 261, PRODUCER_SEND_BACKS = 1 at 1034, Flow::Go handling ~1183 where checks start), crates/athena-core/src/run/producer.rs (work tree is job_dir.join("work")) and crates/athena-core/src/state.rs (StateDir, job_dir, kept_work_dir). The entry point for tests is the drain over RealSteps/Athena::go. No record or data file under crates/ is added to records; the table is a local file in the state root, so check_record_file and AnyRecord are not touched.
- Job 20261029-142849-04d9 (A2, landed): config.rs, CHANGELOG.md and its design-doc note are kept as landed. Its minor findings (design doc line 21 says no [drain.pipeline] table exists; error message lacks range) are not part of this job. If the docs/design/async-drain.md edit touches line 21, fixing it is fine but not required.
- Job 20261029-142853-x4nr (rejected): its wrong-path finding (kept/<id>/work) is fixed by recording the live work tree through one StateDir helper that producer.rs also uses. Its missing-checks-stage finding is fixed by a stage report from the pipeline. Its minor finding about corrupt-table overwrite is addressed by renaming a bad file aside, or noted as left to the recovery job. Its design-doc note finding is addressed by a job 3 implemented note. Its candidate files (drain.rs, state.rs, CHANGELOG.md) are kept as deliverables, redone from scratch.
- Job 20261030-005008-tf2s (rejected retry): its display-string stage matching (StageWatch matching a UI line starting with "producer   ") is dropped. Stages are reported by Athena::go through a callback or observer called where run/mod.rs enters producer, checks, verifier and landing, so a send-back reports producer again. Its docs wording about 'each stage change' is made to match the behavior. Its files (drain.rs, state.rs, producer.rs, CHANGELOG.md, async-drain.md, tests/drain.rs) are kept as deliverables or excluded in out_of_scope as listed.
- Because the callback is added to Athena::go, existing callers (the single-job go command) pass a no-op and behave as before. The default max_in_flight = 1 keeps the drain's visible behavior unchanged.
- Nothing acts on leftover rows; crash recovery is later work.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli drain` (and the athena-core tests). A test through the drain over RealSteps and Athena::go shows the in-flight row's worktree equals the producer's live work tree (<state>/jobs/<id>/work, the path producer.rs uses), not <state>/kept/<id>/work.
2. [behavior] A test through the real entry point forces a producer send-back after a failed self-check, and another after a verifier send-back. It shows the row's stage goes checks (or verifier) back to producer, and the row reads producer during the second producer invocation. Run with `cargo test`.
3. [content] Grep shows the stage is set from the run pipeline entry points (a callback or observer invoked in crates/athena-core/src/run/mod.rs and producer.rs for producer, checks, verifier and landing). No code in drain.rs matches a UI/display line such as "producer   " to choose a stage.
4. [behavior] The table row is written at each stage change and removed when the job finishes or fails. The drain reads the file at start and reports leftover rows without acting on them. A missing file is empty; a corrupt file is noted and the drain continues. Verify with `cargo test` tests for these cases.
5. [behavior] With max_in_flight = 1 the existing drain tests in crates/athena-cli/src/drain.rs and tests/drain.rs pass unchanged: `git diff` shows no edits to existing test assertions, and `cargo test` passes.
6. [behavior] `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` all pass.
7. [content] CHANGELOG.md has an entry for the in-flight table, and its wording about stage changes matches the behavior (a send-back returns the row to producer). Any wording in docs/design/async-drain.md about stage changes (sections 7 and 11) matches the behavior, and it notes build job 3 is implemented.
8. [content] athena.toml, defaults.toml and .github/workflows are unchanged, because the outcome needs no configuration change.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)

## Out of scope

- Acting on leftover table rows (producer retry, landing re-check): later build jobs
- Running more than one producer at once, or raising max_in_flight to 3 (build job 10)
- Reading [drain.pipeline] values in the drain runtime
- Changes to crates/athena-data/src/config.rs (A2 job 20261029-142849-04d9 kept as landed)
- Changes to athena.toml, defaults.toml, .github/workflows
- Disk-guard, check-slot, read-ahead, conflict-return behavior

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added; fmt, clippy -D warnings and the suite must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the worktree path and the send-back stage through the real entry point.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria name commands the verifier can run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change; stage report plus table only.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Stage is reported by the pipeline, not inferred from display text; one shared work-tree helper.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the table content, not that the code exists.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test in crates/athena-cli/tests/drain.rs, added in step 9. Production entry point: `drain::drain` over `RealSteps`, which calls the real `Athena::go_with_stages` (added in step 3), with `RecordingLander` and `StubPage` as in the existing drain tests. Seam: the script closure of the existing `Fixture::athena(Script)` (used by `two_phases` in retry.rs). The test wraps it so that on `Role::Producer` it reads `<state>/in-flight.json` and records the row plus `spec.workdir`. The seam exists today; the wrapper is test code and only observes. Precondition: a one-item queue with a passing scenario. Run: `cargo test -p athena-cli --test drain in_flight_row_names_the_producers_live_work_tree`. Observes: During the producer invocation the row's job is the job under run, the stage is producer, the worktree equals `<state root>/jobs/<id>/work` and equals the producer's `spec.workdir`, the directory exists at that moment, and it is not under `<state root>/kept/`.
  Fails without the behavior: If the row recorded `kept_work_dir` or any path other than `StateDir::work_dir`, the equality with the producer's working directory fails. With no row written, the read finds no row and the test fails.
- Criterion 2: Tests in tests/drain.rs through `drain::drain` over `RealSteps` (real `Athena::go_with_stages`). Seams: the script closure of `Fixture::athena(Script)` snapshots the table file at each agent call (exists today); a recording `Ui` is a trigger only, production never reads it (the `Ui` trait exists today). (a) Send-back: `Scenario.report` marks one criterion unmet (like `unmet_criterion_report` in lifecycle.rs), so the producer is invoked twice; the row is read at both producer invocations. (b) Verifier rejection: a `two_phases`-style script whose first attempt gets a failing verdict (as `first_attempt(verdict_unmet(..))` in retry.rs) and whose second passes; the row is read at the checks line, the verifier invocation, and the retry's producer invocation. A core-level companion runs in crates/athena-cli/tests/lifecycle.rs (`cargo test -p athena-cli --test lifecycle stage_report`): `athena.go_with_stages` with a recording `StageSink` (a plain Vec recorder, no logic) on the send-back scenario. Run: `cargo test -p athena-cli --test drain in_flight_`. Observes: (a) Both producer invocations see stage producer with the same worktree. (b) The row reads checks at the checks line, verifier at the verifier invocation, then producer, with the retry job's id and the retry's work path, at the second producer invocation; there is one row per item, not two. The lifecycle test sees the sequence producer, producer, checks, verifier.
  Fails without the behavior: A pipeline report that fires only at the first producer entry leaves the second producer read showing the stale or missing stage. A retry that removes the row instead of moving it leaves no verifier/checks row to move to producer. Either way the test fails.
- Criterion 3: Content check by reading the diff (the verifier may also use the Grep tool, since grep is not in the allowed shell list). No behavior is executed. Run: `git diff -- crates/athena-core/src/run/mod.rs crates/athena-core/src/run/producer.rs crates/athena-cli/src/drain.rs`. Observes: `stages.enter(.., Stage::Producer)` in producer.rs; `Stage::Checks` and `Stage::Verifier` in the check and verify path and `Stage::Landing` in the Blocked arm of mod.rs; `Stage::Landing` in `RealSteps::land` in drain.rs. No non-test code in drain.rs contains the literals "producer   ", "checks     " or "verifier   "; test code uses them only as read triggers.
- Criterion 4: Unit tests in drain.rs for `InFlight` (read, write, move, remove on a temp dir) and integration tests in tests/drain.rs through `drain::drain` over `RealSteps` (entry point exists today; `in_flight`/`item_over` hooks are added in steps 6-8). (a) After a drain of a passing job and of a failing job (two rejections) the file has no rows. (b) A pre-written file with a leftover row: `drain` returns a note naming job and stage, the file is untouched, and no run starts for it. (c) No file: no note. (d) Garbage in the file: a corrupt note, the file moved to `in-flight.json.corrupt`, and the drain finishes the queue normally. The row is read from the script closure and the Ui trigger as in criterion 2. Run: `cargo test -p athena-cli in_flight`. Observes: Row present at each stage change and absent after `item_over` and after `land`. Leftover rows appear in `Report.notes` without any run call for them. A corrupt file is noted and the drain returns its normal result.
  Fails without the behavior: No row removal leaves rows in the final file. Acting on a leftover row would show a run call. Failing hard on a corrupt file would return an error from `drain`. Treating a missing file as an error would break (c).
- Criterion 5: Run all athena-cli tests: the existing unit tests in drain.rs (lib) and the integration tests in tests/drain.rs, at the default max_in_flight = 1, which go through `drain::drain`, the Stub/World `Steps` and `RealSteps` struct literals. Then inspect `git diff -- crates/athena-cli/src/drain.rs crates/athena-cli/tests/drain.rs` for removed or changed assertion lines. Precondition: new `Steps` methods have default bodies and `RealSteps` has no new field, so the existing code compiles untouched. Run: `cargo test -p athena-cli`. Observes: Every pre-existing drain test passes. The diff shows only added test functions and helpers plus production changes, with no modified or deleted assertion line.
  Fails without the behavior: If the change altered drain behavior at max_in_flight = 1 (extra `Report.notes` entries, a table write failure failing a job, a changed control flow in `work`, or a required new field/trait method), an existing drain test would fail or no longer compile, and an edited assertion would show in the diff.
- Criterion 6: Run the three gates as three separate commands: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, and `cargo test --workspace --locked` (the command named above). They run on the final tree after all steps; no seam or fixture is involved. Run: `cargo test --workspace --locked`. Observes: All three exit 0: no formatting diff, no clippy warning (including unused items from the new sink code and default methods), and every test in every crate passes, including the new tests.
  Fails without the behavior: An unformatted file fails fmt. An unused or needlessly complex item fails clippy under -D warnings. A regression in athena-core or athena-cli (for example the `go` wrapper or `RunContext` change breaking an existing lifecycle test) fails the test run.
- Criterion 7: Read the diff. Content check, no execution. Run: `git diff -- CHANGELOG.md docs/design/async-drain.md`. Observes: An Unreleased CHANGELOG entry says the stage is reported by the run itself at producer, checks, verifier and landing, that a self-check send-back and a retry return the row to producer, and that the worktree is the producer's live work tree. In async-drain.md the section 7 "written at each stage change" wording names those four stages and what a send-back does; section 11/12 notes build job 3 is implemented, with the single-file location and corrupt-file handling. Line 21 is fixed only if touched.
- Criterion 8: Check that no governance file changed. Run: `git status --short athena.toml defaults.toml .github/workflows`. Observes: No output: nothing modified, added or deleted in those paths.

Planned files:

- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8831ae3633ac → candidate 8424361a2b40. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  13 ++
 crates/athena-cli/src/drain.rs         | 389 +++++++++++++++++++++++++++++-
 crates/athena-cli/tests/drain.rs       | 415 ++++++++++++++++++++++++++++++++-
 crates/athena-cli/tests/lifecycle.rs   |  44 ++++
 crates/athena-core/src/run/mod.rs      |  63 ++++-
 crates/athena-core/src/run/producer.rs |  10 +-
 crates/athena-core/src/state.rs        |  67 ++++++
 docs/design/async-drain.md             |  19 +-
 8 files changed, 988 insertions(+), 32 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1733 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
