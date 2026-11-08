# Job 20261028-000831-pm4f: verify "Make drain and status never skip a dispatched job with no outcome"

## Request (verbatim)

> Find out why these six dispatched jobs have job.json and brief-producer.md but no producer invocation and no outcome.json: 20261026-232359-msh3, 20261026-232423-hvyb, 20261027-054148-g63s, 20261027-123701-77zh, 20261027-054206-w1y3 and 20261027-054209-z7sj. All were dispatched on 2026-10-27, and later jobs (for example 20261027-122909-vvvm and 20261027-225544-q7fe) were dispatched and reached an outcome after them. Outcome: every dispatched job ends with an outcome record, either finished, resumed or closed with a stated reason, and drain does not move past a dispatched job while it has no outcome. Close each of the six with an outcome that cites the commit if its work already landed through another job (for example the Transport-row move and the run.rs rename). Otherwise re-queue it. Acceptance: (1) a test dispatches a job, stops it before the producer runs, and shows that the next drain either resumes it or writes an outcome with a reason; (2) athena status lists any dispatched job with no outcome; (3) the six jobs above have outcome records. Do not change unrelated stages.
> 
> Acceptance:
> - The change is made as described: Six dispatched jobs have a producer brief but no producer run and no outcome; later jobs were dispatched and finished past them
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: job.json, brief-producer.md, outcome.json, run.rs

## Reading (accepted by the user)

Goal: stop dispatched jobs that stop before the producer runs from sitting with no outcome. The maintainer (the audience) should see them in `athena status`, and the next drain should resume them or close them with a reason. The six named jobs (msh3, hvyb, g63s, 77zh, w1y3, z7sj) then get outcomes: closed with a commit citation where the work landed elsewhere, otherwise re-queued.

Assumptions:
- Code survey: I read crates/athena-cli/src/drain.rs (queue view around line 4190-4232, which adds steward::interrupted_work items), crates/athena-core/src/steward.rs (interrupted_work, lines 443-480, restricted to DRAINED_CHANNELS and skipping jobs that a later request retries), crates/athena-core/src/status.rs (lines 885-975, where a Dispatched job with no process is shown as 'interrupted' with a hint) and crates/athena-core/src/reading/retry.rs (interrupted detection, lines 20-60). Entry point: drain builds the queue view, then retries interrupted items via `athena retry`. No new record or data file is added, so check_record_file, AnyRecord and the round-trip test are not touched.
- Likely cause, unverified because records/ is absent from my copy: interrupted_work only covers the drained channels and leaves out any job that a later request names as a retry. A dispatched job on another channel, or one whose 'retry' was itself abandoned, is therefore never resumed or closed, and drain moves past it. The producer must confirm this cause against the code and fix it where it originates. If it differs, fix the actual cause.
- Referenced jobs 20261026-232359-msh3 and 20261026-232423-hvyb: their G2 (Transport row move to gh/transport.rs) and R1 (run.rs to run/mod.rs) work already landed through a re-ask and a retry. The repository shows crates/athena-core/src/run/mod.rs and gh/ exist. Their candidates changed no files, so the work is neither kept nor dropped here. The outcomes cite the landing commit. Job 20261027-054148-g63s (reading-scope sizing; deliverables reading.rs, outputs.rs, config.rs, CHANGELOG.md) was never run and has no candidate; its work is not in the tree unless a later job landed it, so it is re-queued rather than closed, and those files are not deliverables here. The producer must check git log for each of 77zh, w1y3 and z7sj the same way, then close or re-queue.
- Recording the six outcomes is an operator action (`athena close <job> --why ...` or `athena retry <job>`) run against the live records. records/ is never edited by the change, so the diff cannot contain them. This part is checked outside the diff and needs the live records.
- Environment: no CI workflow, push or PR is needed. The producer works in the local checkout. Closing the six jobs needs write access to the live job records and read access to git history; neither is in the sandbox copy.
- Standing defaults apply: tested (cargo test and clippy clean), verified (exercise the behavior), root-cause (fix the class, not six instances), small (one focused change).

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs (or crates/athena-core/src/steward.rs unit tests) dispatches a job, stops it before the producer runs (job.json and brief-producer.md present, no invocation, no outcome), then runs a drain. It asserts the job is resumed or gets an outcome record with a stated reason. It covers a job on a non-drained channel and a job named by a later retry request. It fails on the old code and passes on the new. Run: cargo test --locked -p athena-cli --test drain.
2. [behavior] A test shows that drain does not end its run past a dispatched job that has no outcome and no process: that job is resumed, closed with a reason, or reported as held with a reason. Run: cargo test --locked.
3. [behavior] `athena status` lists every dispatched job with no outcome and no process, whatever its channel, with its job id and a next-step hint. A test in crates/athena-core/src/status.rs or crates/athena-cli/tests/ builds records with such a job and asserts the status text or line contains it. Run: cargo test --locked.
4. [content] The six jobs 20261026-232359-msh3, 20261026-232423-hvyb, 20261027-054148-g63s, 20261027-123701-77zh, 20261027-054206-w1y3 and 20261027-054209-z7sj each end with an outcome record (finished, resumed or closed with a reason). Closed jobs whose work landed cite the commit hash found in git log; the others are re-queued. This is checked against the live records (see assumptions); the diff does not contain it.
5. [content] CHANGELOG.md has an entry that names the cause (a dispatched job with no outcome was skipped by drain and not shown by status) and the fix.
6. [behavior] No unrelated behavior changes: existing tests pass, and the producer, verifier and planning stages are untouched. Run: cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/reading/retry.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Moving other structure-pass rows (G3 onward) or redoing the Transport or run/ moves, which already landed
- The reading-scope sizing work of job 20261027-054148-g63s (reading.rs, outputs.rs, config.rs): it is re-queued, not built here
- Editing records/ files, athena.toml, defaults.toml or workflows
- Changing producer or verifier timeouts, planning, or any stage other than drain and status handling of dispatched jobs with no outcome

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for drain and status; fmt, clippy and test are acceptance criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the drain/status gap for the whole class, not the six jobs one by one.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that stop a job before the producer and check the drain and status results.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that every dispatched job ends with an outcome, not that code paths exist.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to drain, steward and status.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 5)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Four layers, each stopping a dispatched job before the producer runs and driving a production entry point. (1) Core: `Athena::go` with `repo_with_planning(1)`; the planner-role script plants the blocking `records` file so the first invocation fails after the Dispatch commit. (2) Drain, real steps: a cli job and a web job stopped with `interrupt` (asserted dispatched, with job.json and brief-producer.md, no invocation, no outcome before the drain), then `drain::drain`; and the planner-role error job in a planning-on Fixture, asserted after the drain. (3) A job whose lock is held with no registry entry. (4) steward `settle_*` and `interrupted_work` unit tests, plus the manage.rs `recover_traces` unit tests. Run: `cargo test --locked -p athena-core run::tests::an_error_after_dispatch && cargo test --locked -p athena-core manage::tests::recover_traces && cargo test --locked -p athena-core steward::tests && cargo test --locked -p athena-cli --test drain -- interrupted after_dispatch`. Observes: Core: `go` returns Finished with state Failed and a reason starting `environment: stopped by an error after dispatch`; `crate::disk::is_environment_reason(&done.reason)` holds, `StopCause::of` is OtherEnvironment, `scope_guard::verdict_of` is `Some(Verdict::Environment)` and `resumable_stop` accepts the job; the job has job.json and brief-producer.md, a recovered planner invocation and no producer invocation; a low-disk hold, a kill-switch stop and a job already closed at HEAD are left as they were (original Err returned, no second outcome). manage.rs: `recover_traces` on a directory with no traces returns nothing; with a live record and a usage file it returns both; `recover_interrupted` still synthesizes the Missing producer for a job with no traces. Drain: the cli and the web original both end Closed(Abandoned) with a reason starting `re-queued as <retry>`; the planner-error job ends Closed(Failed) in the same drain with an `environment:` reason in the report, at most two failed jobs exist for the request, a second drain dispatches nothing, and `steps.paused`/`steps.flagged` return nothing for its line. The lock-held job is not retried, not named in a `held <job>` note and stays Dispatched. Steward: request-less is Close, closed retry is Close, open retry is Hold only when `final_pass` is true, dropped retry gives no settle but is listed by `interrupted_work`, running or lock-held gives nothing, chain Z-A-B closes both, `final_pass` false gives no Hold line, a job in `reported` gets no Hold line.
  Fails without the behavior: Without the dispatch wrapper `go` returns Err and the job stays dispatched with no outcome, so the core and drain error tests fail. Without the new `interrupted_work` the web job is not listed and stays dispatched, a job whose retry was dropped is never re-listed, and a job whose retry landed stays dispatched. Without the lock probe the live off-channel job is retried and the lock test fails. Without the `environment:` prefix the `is_environment_reason` and `Verdict::Environment` assertions fail. Without the Dispatched-at-HEAD check, the unit test that calls `close_after_error` on a job already closed at HEAD writes a second outcome. The phantom-producer case is covered only by the `recover_traces` unit test, which fails if the synthesis moves into it.
  Test seam: Entry points `Athena::go` (run/mod.rs:402) and `drain::drain(&steps, limits(100.0), &Quiet)` with `RealSteps` from `real_steps(&athena, &lander)` (tests/drain.rs:4365, exists). Fixtures that exist: `athena_with`, `repo_with_planning`, `planning_script` (run/tests.rs:187, 287, 1189), `Fixture::new()`, `killing_producer`, `interrupt`, `request_on` (tests/drain.rs:5094, 5108, 4968), the steward `add` helper (steward.rs:2118-2216), `StateDir::try_lock_job`. Doubles: scripted agent and `RecordingLander` only; the error close, listing, settle and lock probe are production code. Steps that add seams: the dispatch wrapper and `close_after_error` (step 5); `recover_traces` (step 3); `Athena::has_process` (step 4); `steward::settle_interrupted` (step 6); the `Steps` hook (step 7); a planning-on Fixture builder in tests/drain.rs following job_plan.rs:85 (step 10).
- Criterion 2: Drain with two interrupted cli jobs and `Limits{max_jobs: 1,..}` against `RealSteps`; Stub-driven tests for the hook, the reported set and the plan-entry exemption. Run: `cargo test --locked -p athena-cli --test drain -- drain_leaves_no_dispatched_job drain_works_an_interrupted_item drain_notes_what_settling`. Observes: After the drain every request job is Closed, or is Dispatched and named in `report.notes` by exactly one line starting `held <job>:` with a reason and the `athena retry <job>` hint. The first job is retried and its original closed; the second is not worked and is held with a reason. A job the drain already passed over as held (disk-hold marker) gets no second `held` line. Stub: an interrupted item with an incomplete plan entry is retried (log `reread a`, `run a-retry`) with a note that the retry reads the request again; the start call (`final_pass=false`, `reported` empty) and the end call (`true`, `reported` = ids from `passed_over` and `st.jobs`) reach `Report::notes`; the start call never returns a Hold line. A job whose lock is held is not named in a held line.
  Fails without the behavior: Without the final-pass Hold the second job stays dispatched with no note. Without the `reported` set the held-by-disk job is named twice. Without the plan-entry exemption the item is refused as not worked and never re-read. Without the hook no settle note appears.
  Test seam: `drain::drain` against `Stub` (tests/drain.rs:127, exists) extended with a `settled` field and a `settle_interrupted` impl added in step 10 (the double records the arguments it is called with and returns scripted lines; it holds no selection logic); `Limits.max_jobs` exists (drain.rs:288-294; `limits()` at tests/drain.rs:332); real-steps fixture as in criterion 1. Selection and the Hold rules are steward code; the exemption, the `reported` set and the notes are drain.rs code added in steps 7-8.
- Criterion 3: Real-steps test: interrupt a cli job and a web job; add a third dispatched job with a disk-hold marker (`go_with_disk`, tests/drain.rs:2424); add a fourth whose lock is held with no registry entry. Call `athena.status()` and `display::status(&view)`; then run the drain. Run: `cargo test --locked -p athena-cli --test drain -- status_lists_every_dispatched_job && cargo test --locked -p athena-cli --test retry -- dispatched`. Observes: The cli, web and held jobs are in `view.open` with state interrupted and an `athena retry <id>` next line, and `display::status` contains each id. The lock-held job is listed with state running and no retry hint. After the drain the cli and web originals are not in `open` (closed with a reason). Existing tests/retry.rs tests 1031, 1072, 1110, 1139 keep passing (no lock is held there).
  Fails without the behavior: The post-drain half fails on old code (originals still shown as interrupted, nothing closed them). The lock-held half fails if `state_name` keeps the registry-only test.
  Test seam: `Athena::status()` (status.rs:985) and `display::status` on the real `Fixture` with `killing_producer`; `interrupt`, `request_on`, `go_with_disk` exist. The lock comes from `StateDir::try_lock_job` (exists). `Athena::has_process` is added by step 4.
- Criterion 4: Operator step against the live records, done BEFORE the new build is deployed or drained. Today's `athena status` lists every dispatched job with no outcome as interrupted. For each of msh3, hvyb, g63s, 77zh, w1y3, z7sj read brief-producer.md and `git log`; where the work landed (HISTORY.txt shows 5874b1f for the Transport row move and 612c27f for the run.rs rename, both 2026-10-27, as candidates only) run `athena close <job> --why "landed through <job id> as <hash>"` with the hash from git log; otherwise `athena retry <job>`. Then close other old interrupted jobs the drain should not resurrect, then deploy. Preconditions: write access to the live records and the full git log (neither is in the snapshot). Run: `cargo run --locked -p athena-cli -- status`. Observes: None of the six ids is an unretried entry in `open`; each has an outcome.json (closed with the hash in the reason) or a standing retry shown as `retried as <job>`. Status lists no other interrupted job that the operator did not choose to leave.
- Criterion 5: Read the new entry under [Unreleased] ### Fixed in CHANGELOG.md and the new D167 in docs/decisions.md, and run the decision-log structure checks (D162 `DecisionEntries`, the athena-cli log guard) through the mirrors test target. Run: `git diff -- CHANGELOG.md docs/decisions.md && cargo test --locked -p athena-cli --test mirrors`. Observes: The CHANGELOG entry names the causes (a run that errored after the Dispatch commit left a dispatched job with no outcome; drain listed interrupted jobs only for the drained channels and hid any job a later request retried, even a dropped one; nothing closed an original once its retry ended; status showed a job another process runs as interrupted), the fixes (error close with an `environment:` reason; all channels with liveness from the registry and the job lock; settle step with end-of-drain hold lines; plan-entry exemption), the failure class, the six job ids, the operator order (close before the first drain on the new build), that reading/retry.rs is unchanged and status only gained the lock-aware running test, that D167 replaces the interrupted-job sentence of D104, and that it supersedes the older 'drained channels only' entry. It uses the words dispatched, no outcome, drain and status. D167 is appended after D166 with a Provenance bullet and the mirrors target passes.
- Criterion 6: Run the three Athena checks over the workspace. The only existing expectation that changes is the off-channel drain test (tests/drain.rs:5122), rewritten. Guards: steward test `queued_work_still_leaves_other_channels_alone`; the hold tests in run/tests.rs (3550, 3609, 3986, 4098, 4190, 4645, 4694, 4806) still expect Err; tests/retry.rs close tests show `athena close` is unchanged by the split; reading.rs:8355 and refusal_corpus.rs:1598 still read their decision entries. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: fmt, clippy and test pass. `git diff --stat` shows no change under producer, verifier, planner, critic, agent or context code, nor reading/retry.rs, hestia.rs, disk.rs or records/.
  Fails without the behavior: If the channel filter were removed from `queued_work` the steward test and tests/drain.rs:5334 fail; if the error close also closed holds the run/tests.rs hold tests fail; if the split changed `athena close`, tests/retry.rs fails.
  Test seam: The three cargo commands over the workspace. Fixtures: `Stub`, `Fixture`, the steward `add` helper and the run/tests.rs fixtures; all exist.

Planned files:

- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/manage.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base de12b17e36c6 → candidate f25c6a19e05f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  39 ++++
 crates/athena-cli/src/drain.rs      | 100 ++++++++-
 crates/athena-cli/tests/drain.rs    | 399 ++++++++++++++++++++++++++++++++--
 crates/athena-core/src/manage.rs    |  71 +++---
 crates/athena-core/src/run/mod.rs   | 135 +++++++++---
 crates/athena-core/src/run/tests.rs | 155 +++++++++++++
 crates/athena-core/src/status.rs    |  15 +-
 crates/athena-core/src/steward.rs   | 418 ++++++++++++++++++++++++++++++++++--
 docs/decisions.md                   |  52 +++++
 9 files changed, 1281 insertions(+), 103 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261028-000831-pm4f/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261028-000831-pm4f/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42530014704 for f25c6a19e05f753e5df42cb27a5bfad1e25fff27)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42530014704 for f25c6a19e05f753e5df42cb27a5bfad1e25fff27)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42530014704 for f25c6a19e05f753e5df42cb27a5bfad1e25fff27)
- ci-result: `not a check` passed (Check results for f25c6a19e05f753e5df42cb27a5bfad1e25fff27 come from the CI run https://github.com/doodla/athena/actions/runs/42530014704 (553s): fmt success, clippy success, test success.)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: acceptance criteria not met or not reported: "The six jobs 20261026-232359-msh3, 20261026-232423-hvyb, 20261027-054148-g63s, 20261027-123701-77zh, 20261027-054206-w1y3 and 20261027-054209-z7sj each end with an outcome record (finished, resumed or closed with a reason). Closed jobs whose work landed cite the commit hash found in git log; the others are re-queued. This is checked against the live records (see assumptions); the diff does not contain it.")
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: No unrelated behavior changes: existing tests pass, and the producer, verifier and planning stages are untouched. Run: cargo fmt --all --check, cargo clippy --all-targets --locked -- -D warnings and cargo test --locked.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test --locked -p athena-core run::tests::an_error_after_dispatch && cargo test --locked -p athena-core manage::tests::recover_traces && cargo test --locked -p athena-core steward::tests && cargo test --locked -p athena-cli --test drain -- interrupted after_dispatch`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli --test drain -- drain_leaves_no_dispatched_job drain_works_an_interrupted_item drain_notes_what_settling`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/manage.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/status.rs, crates/athena-core/src/steward.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling tempfile v3.27.0
   Compiling schemars_derive v1.2.2
   Compiling regex-automata v0.4.18
   Compiling anyhow v1.0.104
   Compiling equivalent v1.0.2
   Compiling dyn-clone v1.0.20
   Compiling hashbrown v0.17.1
   Compiling schemars v1.2.2
   Compiling regex v1.13.1
   Compiling indexmap v2.14.2
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling utf8parse v0.2.2
   Compiling anstyle-parse v1.0.0
   Compiling syn v2.0.119
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling colorchoice v1.0.5
   Compiling anstyle v1.0.14
   Compiling anstyle-query v1.1.5
   Compiling anstream v1.0.0
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling strsim v0.11.1
   Compiling clap_lex v1.1.1
   Compiling heck v0.5.0
   Compiling clap_builder v4.6.7
   Compiling clap_derive v4.6.7
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling clap v4.6.7
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0407]: method `settle_interrupted` is not a member of trait `Steps`
   --> crates/athena-cli/tests/drain.rs:172:5
    |
172 |       fn settle_interrupted(&self, final_pass: bool, reported: &BTreeSet<String>) -> Vec<String> {
    |       ^  ------------------ help: there is an associated function with a similar name: `reclaim_interrupted`
    |  _____|
    | |
173 | |         self.settled
174 | |             .borrow_mut()
175 | |             .push((final_pass, reported.clone()));
...   |
181 | |         lines.iter().map(|l| l.to_string()).collect()
182 | |     }
    | |_____^ not a member of trait `Steps`

error[E0599]: no method named `has_process` found for struct `athena_core::Athena` in the current scope
    --> crates/athena-cli/tests/drain.rs:5459:20
     |
5459 |     assert!(athena.has_process(&job));
     |                    ^^^^^^^^^^^ method not found in `athena_core::Athena`

Some errors have detailed explanations: E0407, E0599.
For more information about an error, try `rustc --explain E0407`.
error: could not compile `athena-cli` (test "drain") due to 2 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test --locked -p athena-cli --test drain -- status_lists_every_dispatched_job && cargo test --locked -p athena-cli --test retry -- dispatched`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
