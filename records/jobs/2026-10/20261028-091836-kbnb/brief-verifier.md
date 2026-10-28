# Job 20261028-091836-kbnb: verify "Remove target/produce after checks pass; resume retry from kept"

## Request (verbatim)

> Priority: urgent
> Keep a finished job from failing twice on disk. A job that passes its own checks can still stop before the verifier with under 10 GB free, because its producer build directory (target/produce) is still on disk; its retry then plans and builds again from scratch. Fix both.
> (1) Disk: once Athena's own checks have passed, remove the job's target/produce before the gate that runs before the verifier; keep it when a check fails (the send-back needs it).
> (2) Resume: when a job stops for an environment cause after its candidate is built (use the existing disk::is_environment_reason, and treat an interrupted process the same way), athena retry resumes from the kept candidate at the stage that stopped, with no new planner or producer run. The retry still runs the reader.
> Tests: the low-disk test must clear the gate before the checks and stop only at the gate before the verifier (enough free space for the first gate, less for the second). Show resume by counting planner and producer invocations in the retry (zero), not by counting reads of the free-space function.
> Acceptance:
> - A test shows target/produce is gone after checks pass and before the verifier gate, and kept when a check fails.
> - A test stops a job on low disk at the gate before the verifier and shows athena retry runs no planner or producer and goes on to the verifier with the kept candidate.
> - A test shows an interrupted job with a built candidate resumes the same way.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/reading.rs, CHANGELOG.md

## Reading (accepted by the user)

Two fixes for the job pipeline. (1) After Athena's own checks pass, delete the job's target/produce before the pre-verifier disk gate, and keep it when a check fails. (2) When a job stops after its candidate is built for an environment reason (disk::is_environment_reason, or an interrupt), athena retry re-reads but skips planner and producer and resumes at the stage that stopped. The audience is Athena operators and the verifier.

Assumptions:
- Resume state is recorded with the job so the retry can find the kept candidate and the stage that stopped; reading.rs is where retry reads the earlier job.
- Interrupted-process stops are treated like environment-cause stops only when a built candidate exists.
- The existing test helpers can count planner and producer invocations with fake agents.

## Acceptance criteria

1. [behavior] A test shows target/produce is absent after checks pass and before the verifier gate, and present when a check fails. Run: cargo test.
2. [behavior] A test stops a job on low disk at the gate before the verifier, with enough free space for the first gate and less for the second. It then shows athena retry runs zero planner and zero producer invocations and proceeds to the verifier with the kept candidate. Run: cargo test.
3. [behavior] A test shows an interrupted job with a built candidate resumes the same way: zero planner and producer runs in the retry, and the reader still runs. Run: cargo test.
4. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.
5. [content] CHANGELOG.md has an entry describing the cleanup of target/produce and the resume-from-candidate retry.
6. [content] The resume tests count planner and producer invocations, not free-space function reads (check with Grep in the test code).

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/test_support.rs (modify)

## Out of scope

- Changing the thresholds of the disk gates
- Changes to athena.toml, defaults.toml or workflows
- Cleaning other target directories

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for cleanup, low-disk resume, and interrupt resume; cargo test and clippy pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests count planner and producer invocations in the retry (zero) and check target/produce is absent or present.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Behavior is enforced in code and tests, not prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix both the disk retention and the redo-from-scratch retry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test in `produce_and_verify_order_tests` (run/mod.rs). It drives `Athena::ask` then `Athena::go` (production entry points) with the fake driver and a temporary git repo. The Producer turn in the Script plants a file under `<state>/jobs/<id>/target/produce`. A new `test_support::recording_plenty_space` free-space function (added in step 10) is installed with `StateDir::with_free_space` (seam exists today). It returns plenty of space and records, per state root, whether `jobs/*/verify` and `jobs/*/target/produce` exist on every call. Case A uses a capability whose check is `true`, case B one whose check is `false`, via a new `athena_with` constructor that `athena` calls. The check command is a parameter of the capability; no job logic sits in the double. Run: `cargo test -p athena-core produce_dir_is_removed_after_checks_pass_and_kept_when_a_check_fails`. Observes: Case A: at the pre-verifier gate call (verify dir exists) produce is absent. Case B: at that call produce is present, and the job still reaches the verifier or send-back.
  Fails without the behavior: Without the removal step, the probe records produce present at the pre-verifier gate in the passing case and the first assertion fails. If removal also ran for failing checks, the second case fails on the missing directory.
- Criterion 2: New test in `produce_and_verify_order_tests`. `Athena::ask`, then `go` with `test_support::low_once_verifying` (added in step 10): plenty of space until `jobs/*/verify` exists, 3 GB after, unless a `freed` file exists in the state root. That clears the gate before the checks and stops at the gate before the verifier. The Script closure (exists today) is wrapped to count Planner, Producer, Reader and Verifier invocations in AtomicUsize counters. The test asserts the first job ends Failed with `is_environment_reason`, removes the stop condition, calls `Athena::retry`, then `go` on the proposal's job. Run: `cargo test -p athena-core low_disk_before_the_verifier_resumes_without_planner_or_producer`. Observes: Before the retry: producer 1, verifier 0. In the retry: planner 0, producer 0 (stays 1), reader 1, verifier 1. The retry ends Landed with `verified_fingerprint` equal to the first job's candidate fingerprint. No `checks` UI line in the retry (resumed at stage Verifier). The first job's `kept/<job>/resume.json` is gone.
  Fails without the behavior: Without the resume path the retry runs the planner/producer again, so the producer count is 2 and the assertion that it is 0 more fails. Without the mark the retry cannot find the candidate.
- Criterion 3: New test in the same module. The Ui given to `go` (Ui::line seam exists today) panics when it sees a line starting `checks     `; `go` is wrapped in `std::panic::catch_unwind(AssertUnwindSafe(..))`. That simulates a process dying after the candidate is built: the job lock is dropped and the job stays Dispatched with no outcome. Counters wrap the Script as in criterion 2. The test asserts the old job's state is `JobState::Dispatched`, then calls `retry` and `go` with a normal Ui. Run: `cargo test -p athena-core interrupted_job_with_a_candidate_resumes_without_planner_or_producer`. Observes: In the retry: planner 0, producer 0 (total stays 1), reader 1, verifier 1. The job lands and the landed README content is the first producer's edit. The first job's mark is removed.
  Fails without the behavior: Without the interrupted-job handling no resume seed is built for a Dispatched job that never closed, and the retry runs the producer (count 2, expected 1 total). Without the mark written at candidate build there is nothing to resume.
- Criterion 4: Run the three Athena checks as whole commands in sequence (each also separately runnable by the verifier). They call no production entry point of their own; they gate the whole change. The new unit tests in state.rs (mark round trip, `remove_produce` keeps the rest of `target/`), disk.rs (`resumable_stop`: Failed with and without an environment reason, Rejected, Landed, interrupted) and reading.rs (resume job lookup) run inside `cargo test`. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0 on the full workspace, with the three behavior tests of criteria 1-3, the new unit tests and the existing characterization tests (`produce_and_verify_order_tests`, drain.rs `low_disk_after_the_checks_fails_the_job_as_the_environment_before_the_verifier_runs`) all passing.
  Fails without the behavior: `cargo test --locked` fails if the cleanup or resume behavior is missing (the new criterion 1-3 tests and unit tests fail), or if the split of `produce_and_verify` changed the existing order (characterization tests fail). `cargo clippy -D warnings` fails if new helpers (`resumable_stop`, `remove_produce`, resume mark methods) are left unused or badly written. `cargo fmt --check` fails if the new code is unformatted.
- Criterion 5: Read the CHANGELOG.md diff. Run: `git diff -- CHANGELOG.md`. Observes: A new entry under the existing Unreleased heading, in the file's entry format, says target/produce is removed once Athena's checks pass and kept when one fails, and that `athena retry` resumes from the kept candidate after an environment or interrupt stop with no planner or producer run and the reader still run.
- Criterion 6: Read the added test lines (Grep over added lines gives the same). Check the resume tests assert on counters bumped in the Script for `Role::Planner` and `Role::Producer`. Run: `git diff -- crates/athena-core/src/run/mod.rs crates/athena-core/src/test_support.rs`. Observes: Assertions on the planner and producer counters in the retry equal 0 (producer total stays 1). The free-space function is used only to simulate low space and, for criterion 1, to record directory presence. No assertion counts its reads.

Planned files:

- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- CHANGELOG.md (modify)

## The change

Base c310b1d291e6 → candidate 845a3f728dc0. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  13 +
 crates/athena-core/src/disk.rs         |  44 ++
 crates/athena-core/src/manage.rs       |   2 +
 crates/athena-core/src/reading.rs      |  34 +-
 crates/athena-core/src/run/mod.rs      | 725 +++++++++++++++++++++++++++++++--
 crates/athena-core/src/state.rs        | 150 +++++++
 crates/athena-core/src/test_support.rs |  62 +++
 7 files changed, 988 insertions(+), 42 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1533 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
