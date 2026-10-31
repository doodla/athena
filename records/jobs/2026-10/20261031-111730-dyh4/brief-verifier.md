# Job 20261031-111730-dyh4: verify "Retry low-disk hold-and-resume; fix scope-guard test and decisions entry"

## Request (verbatim)

> try again on 20261031-110024-4qd3

## Reading (accepted by the user)

Retry of job 20261031-110024-4qd3 (rejected): keep its hold-and-auto-resume work, so low disk after idle-build cleanup before the checks or the verifier holds the job resumable, not failed. Fix its two findings: the stale scope_guard test and the failing backtest from the docs/decisions.md entry. Audience: the maintainer running drains and the verifier running cargo test.

Assumptions:
- Code survey: read crates/athena-core/src/scope_guard.rs (test a_low_disk_job_is_read_as_an_environment_attempt, line 1375, still asserts Failed and Verdict::Environment, panics because go() now returns the LowDisk hold error); crates/athena-cli/src/backtest.rs line 337 (the backtest emits 'supersession phrases without a target id', so the decisions.md entry the candidate added near line 3875 uses such a phrase). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected. The property is measured by running the real drain and run tests with ATHENA_FREE_BYTES, not a stand-in.
- Job 20261031-110024-4qd3 findings addressed: (a) blocker, stale scope_guard test: rewrite it so go() returns Err LowDisk with no attempt recorded and no outcome, then after space is freed the retry or next go resumes the kept candidate as resumed and lands; (b) failing backtest: reword the docs/decisions.md entry so any supersession phrase names a target decision id, or has none; (c) major, criterion 8 unevidenced: the full cargo test --locked now passes and the recorded check shows it.
- Job 20261031-110024-4qd3's candidate changed 16 files. All 16 are kept as deliverables: the hold design is sound, and only the test and the docs fix are new work. The retry starts from the base, so the producer redoes the whole change, guided by the earlier approach: Athena::hold_for_disk commits the attempt's records without a Score or Outcome, writes a hold marker with free and required amounts, and returns LowDisk; drain maps LowDisk to Held with no attempt used; push_closing_records is shared from close.rs.
- This is a code change in Rust; no refusal rule is added to readings, so no newly-refused percentage applies.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test drain`. A test with simulated low space (ATHENA_FREE_BYTES) after the build shows the job ends held, not Failed, with no `athena retry`, no verifier or producer verdict recorded, and outcome text naming `environment: low disk` with free and required amounts. It fails on pre-change code.
2. [behavior] Run `cargo test -p athena-cli --test drain`. A test shows a later drain pass, after simulated free space is raised, resumes the held job from its kept candidate with no new producer run and no `athena retry`, and the job finishes.
3. [behavior] Run `cargo test -p athena-cli --test drain`. A test shows space that stays short across passes leaves the job held and uses no drain attempt.
4. [behavior] Run `cargo test -p athena-core`. Tests show a low-disk hold before the checks and before the verifier is not counted as a work failure in introspection, failure classes or the scope guard. The updated scope_guard test a_low_disk_job_is_read_as_an_environment_attempt passes by asserting go returns the LowDisk hold with no attempt recorded and the later resume lands as resumed. Idle-build cleanup still runs first, and removes only Athena-owned finished-job directories.
5. [behavior] Run `cargo test -p athena-cli --test binary`. `athena status` shows the held job with its low-disk reason and the amounts.
6. [behavior] Run `cargo test -p athena-cli --test backtest`. It passes, including the_real_log_has_no_condition_a_finding_against_the_confirmed_list, so the docs/decisions.md entry carries no supersession phrase without a target id.
7. [content] Grep of crates/athena-core/src/run/mod.rs, run/verification.rs and disk.rs shows the low-disk path no longer builds a failure with a `before the checks` or `before the verifier` message telling the person to run `athena retry`.
8. [content] CHANGELOG.md has an entry for hold-and-auto-resume; module docs in crates/athena-core/src/disk.rs and crates/athena-cli/src/drain.rs describe it instead of fail-and-retry; docs/decisions.md records the call.
9. [behavior] Run `cargo test --locked`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`; all pass with no failing test anywhere in the workspace.
10. [content] Governance files are unchanged: athena.toml, defaults.toml, .github/workflows, Cargo.toml, Cargo.lock and rust-toolchain.toml are not in the diff.

## Expected deliverables

- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Automatic proposing after introspection (job 20261025-033738-yz2d): main.rs, introspect.rs, proposals.rs and their tests; rerun separately
- The budget-cap hold for planner, producer and verifier stages; a separate job
- Test-suite speedup and other work of job 20261025-202447-xgk9
- Guarding combined disk use of parallel jobs, or planning around disk
- Changing the [disk] min_free_gb default or config schema, or editing athena.toml, defaults.toml or workflows
- Deleting committed records or touching a running job's files in the cleanup
- Removing `athena retry` or its handling of old failed environment jobs

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt criteria.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run drain, binary and backtest tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the stale test and the decisions entry that caused the failures.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check hold, resume and status behavior by running tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is that a full-disk drain never ends a job failed.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New drain test `low_disk_after_the_build_holds_the_job_and_does_not_fail_it`, with a variant for the hold before the verifier. It builds the disk fixture over the real steps. The producer step of the scripted scenario writes the simulated free-space file with 3 GB (the `go_with_disk` pattern); free space is high at dispatch. The verifier variant sets it low once `jobs/<id>/verify` exists (`low_once_verifying`). It runs `drain::drain(&steps, work_everything(), &Quiet)` once, loads the records and reads the hold marker. Run: `cargo test -p athena-cli --test drain low_disk_after_the_build_holds_the_job`. Observes: The item ends held: outcome text contains `environment: low disk`, `3.0 GB` and `10.0 GB`, and `failed_item` is None. The job is `JobState::Dispatched` with no Outcome and no Score. No Verifier invocation exists for the job. No line or reason contains `athena retry`. `disk::read_hold` returns the amounts. With an idle build present, the `disk       removed build directory` line comes before the `held` line.
  Fails without the behavior: Before the change the same run closes the job `OutcomeState::Failed` with a reason containing `athena retry <job>`, so the Dispatched, no-Outcome and no-retry-text assertions fail.
  Test seam: Production entry `drain::drain` over `real_steps(&athena, &lander)` with the scripted `Scenario` driver. Free space is injected with `athena.state.with_free_space(simulated_free)`, a file read at each gate so it can change mid-run. Both exist today (tests/drain.rs `go_with_disk` pattern and `rated_scenario().script()`). No new seam.
- Criterion 2: New test `a_later_pass_resumes_the_held_job_from_its_kept_candidate`, continuing the held state of criterion 1. It records the job count and the reader, planner and producer call counts after pass 1, raises free space with `set_free(&athena, 500_000_000_000)`, and runs a second `drain::drain`. The counts come from a call-counting wrapper around the script that only counts and delegates to the scenario. Run: `cargo test -p athena-cli --test drain a_later_pass_resumes_the_held_job`. Observes: Pass 2 lands the work (the RecordingLander holds the new job's id) and the item is not failed. Exactly one new job exists (the retry B). Exactly one more reader call, no planner call, no producer call, so the producer invocation count equals pass 1's. B's reading did not need accepting. Output has `resume     job <A> kept its candidate`. The test never runs the `athena retry` command. A's hold marker is gone.
  Fails without the behavior: Without the hold, A is closed Failed and left for `athena retry`. With a hold but no resume from the kept candidate, the producer or planner count rises or the job never lands.
  Test seam: `drain::drain` with RealSteps (it calls `Athena::retry` internally for an interrupted job, drain.rs:2896), the call-counting script wrapper (test code that only counts and delegates), and `set_free`. All exist today in tests/drain.rs.
- Criterion 3: New test `space_that_stays_short_leaves_the_job_held_and_uses_no_attempt`. After pass 1 holds job A, free space stays at 3 GB and two more drains run. Job ids are followed from `Records::load`. Run: `cargo test -p athena-cli --test drain space_that_stays_short_leaves_the_job_held`. Observes: Pass 1: held, `failed_item` None, no attempt counted. Pass 2: exactly one new job B with one more reader call; B is Proposed with a hold marker and amounts (held at dispatch); no planner or producer call; the report says held, stop reason is not `RepeatedFailure`. Pass 3: no new job, no new reader call, B still held, A and B's records unchanged. Landed is 0 in each pass.
  Fails without the behavior: If a hold counted as a failed attempt the drain would retry in the same pass (two jobs in one pass) or end with `Stop::RepeatedFailure`. If pass 3 created a job, the job count check fails.
  Test seam: `drain::drain` over real steps with `set_free` and the call-counting script wrapper; all exist today in tests/drain.rs. Job list read via `Records::load`.
- Criterion 4: (a) Rewrite scope_guard test `a_low_disk_job_is_read_as_an_environment_attempt`: `r.go(&a)` returns an Err that downcasts to `disk::LowDisk`; `request_records` shows `requests[0].attempts` empty, no outcome, no pause or flag file. After the `freed` file appears, `r.athena.retry(a)` then `r.go(&retry.job)` lands, and `requests[1].attempts[0].resumed` is true with `Verdict::Accepted`. (b) Rewrite the low-disk gate tests in run/tests.rs (~1465-1490, ~1610-1630): a hold before the checks and before the verifier return LowDisk, record no Outcome, add no introspection record and no failure class, and run the idle sweep first, removing only Athena job directories (a planted idle job's target goes; its source and an unrelated state-root directory stay). (c) Add disk.rs unit tests for the hold marker contents, the probe-failure hold (free 0, probe error printed) and `settle_resume_marks` clearing the consumed job's hold. Run: `cargo test -p athena-core`. Observes: go returns Err(LowDisk) with the amounts; zero attempts for the held job; the retry job's attempt is `resumed` and Accepted; introspections and failure classes are empty; the `disk       removed build directory of job` line comes before `held`; the unrelated directory is untouched.
  Fails without the behavior: Before the change go returns Ok(Finished Failed) and `attempts[0].verdict` is Environment, so the unwrap_err, empty-attempts and empty-introspection assertions fail.
  Test seam: `Athena::go` and `Athena::retry` through `rig_with_space(..., test_support::low_once_verifying)` in scope_guard.rs, and `Athena::state.with_free_space` in run/tests.rs. Both exist today. test_support.rs gains a free-space helper (high at dispatch, low at a named gate) in plan step 9 only if `low_once_verifying` cannot reach the gate before the checks.
- Criterion 5: New binary test for a held job. The hold is made through athena_core in the test's own repo and state directory (in-process run with `with_free_space` low after the producer, or `disk::write_hold` on a Dispatched fixture job if the harness cannot share state). It then spawns the built `athena status` against the same repo and state, with `ATHENA_FREE_BYTES` set to a fixed low value where a fixed value is enough, and reads stdout. Run: `cargo test -p athena-cli --test binary`. Observes: stdout shows the job as interrupted and held with `environment: low disk: <free> free, <required> required` and a hint that the next drain resumes it, not `athena retry`.
  Fails without the behavior: Status fills the hold only for a `proposed` job today (status.rs ~1659), so a Dispatched held job prints no reason.
  Test seam: The spawned `athena` binary (production entry; binary.rs ~158-206 already sets ATHENA_FREE_BYTES). The held state is built with `Athena::state.with_free_space` or the `disk::write_hold` fallback (exists, disk.rs:172). Whether binary.rs can share state with an in-process Athena is checked in plan step 11.
- Criterion 6: Run the backtest suite, including `the_real_log_has_no_condition_a_finding_against_the_confirmed_list`, against the reworded docs/decisions.md. Run: `cargo test -p athena-cli --test backtest`. Observes: The suite passes: the new entry carries no supersession phrase without a target decision id.
  Fails without the behavior: The previous candidate's entry raised the 'supersession phrases without a target id' finding (backtest.rs ~337), failing that test.
  Test seam: Production entry is the backtest test harness in crates/athena-cli/tests/backtest.rs, which reads the real docs/decisions.md directly. No fake or injection; the file under test is the real one. Exists today.
- Criterion 7: Content check by grep over the three files. Run: `grep -n "athena retry\|before the checks\|before the verifier\|before_verifier_reason" crates/athena-core/src/run/mod.rs crates/athena-core/src/run/verification.rs crates/athena-core/src/disk.rs`. Observes: No line builds a failure with `before the checks` or `before the verifier` telling the person to run `athena retry`. `before_verifier_reason` is gone and both `self.failure(...)` calls at the gates are gone. Comments describing a hold are fine; the `check_timeout_reason` text may keep `athena retry` in disk.rs (outside this job).
  Fails without the behavior: The base has `failure(... before the checks ... athena retry {id})` in mod.rs (~1191) and uses `before_verifier_reason` in verification.rs (~139).
- Criterion 8: Content check: read the new CHANGELOG Unreleased entry, the //! docs of disk.rs and drain.rs, and the new decisions.md entry. Run: `grep -n "hold" CHANGELOG.md crates/athena-core/src/disk.rs crates/athena-cli/src/drain.rs docs/decisions.md`. Observes: CHANGELOG describes hold-and-auto-resume. disk.rs module docs say the second and third gates hold and a later drain resumes from the kept candidate only while main has changed nothing outside records/. drain.rs docs say a held job is left held with no attempt used and the next pass resumes it. decisions.md records the call, including that 'no `athena retry`' means no person runs the command. The disk.rs bullets no longer say 'fails with an environment: low disk reason'.
  Fails without the behavior: The base docs still describe fail-and-retry.
- Criterion 9: Run the three Athena check commands unchanged over the whole workspace after all changes. The tests inside `cargo test --locked` are the ones rewritten or added for criteria 1-6; no extra test is added for this criterion. Run: `cargo test --locked`. Observes: All workspace tests pass, including the rewritten scope_guard, run/tests.rs, drain.rs and binary.rs tests and the backtest, with no failing test anywhere. Then `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check` (also whole commands) exit clean.
  Fails without the behavior: At the base, once go holds, scope_guard `a_low_disk_job_is_read_as_an_environment_attempt` panics on the LowDisk error, and the old drain and run tests asserting Failed fail; the docs/decisions.md entry of the earlier candidate fails the backtest. Any of these left unfixed makes `cargo test --locked` fail.
  Test seam: Production entry points: the cargo test harness (`cargo test --locked`), clippy and rustfmt over the whole workspace; nothing is stubbed. Preconditions: Cargo.lock unchanged so `--locked` resolves; a clean working tree with every plan step applied. Fixtures are those of each included test, which already exist: `with_free_space` file seam and `rig_with_space` in-process, and `ATHENA_FREE_BYTES` for the spawned binary in tests/binary.rs. This criterion injects nothing itself and adds no seam; the only helper any test may add is the optional test_support.rs free-space helper from plan step 9.
- Criterion 10: List the diff of the job's branch against the base. Run: `git diff --stat`. Observes: No athena.toml, defaults.toml, .github/workflows, Cargo.toml, Cargo.lock or rust-toolchain.toml in the diff.
  Fails without the behavior: A governance file in the diff.

Planned files:

- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base d5421ca15c9f → candidate e260bb7dd6bf. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  14 ++
 crates/athena-cli/src/display.rs           |  11 +-
 crates/athena-cli/src/drain.rs             |  14 +-
 crates/athena-cli/tests/binary.rs          |  76 +++++++
 crates/athena-cli/tests/drain.rs           | 309 ++++++++++++++++++++++++-----
 crates/athena-core/src/disk.rs             | 114 +++++++----
 crates/athena-core/src/reading.rs          |   5 +-
 crates/athena-core/src/run/close.rs        | 107 ++++++----
 crates/athena-core/src/run/mod.rs          |  29 ++-
 crates/athena-core/src/run/tests.rs        | 141 +++++++++++--
 crates/athena-core/src/run/verification.rs |  15 +-
 crates/athena-core/src/scope_guard.rs      |  17 +-
 crates/athena-core/src/state.rs            |   4 +-
 crates/athena-core/src/status.rs           |   6 +-
 crates/athena-core/src/test_support.rs     |  20 ++
 docs/decisions.md                          |  40 ++++
 16 files changed, 737 insertions(+), 185 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1817 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --locked`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`; all pass with no failing test anywhere in the workspace.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
