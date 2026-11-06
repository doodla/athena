# Job 20261106-180606-ymst: verify "Retry: producer cargo build/test/clippy/check take a build slot"

## Request (verbatim)

> try again on 20261106-030952-0f7a

## Reading (accepted by the user)

Retry of job 20261106-030952-0f7a (stale: branch moved during verification). Keep its verified candidate (athena-linked `cargo` wrapper first on the producer PATH, holding a build slot for build/test/clippy/check, subcommand found after global flags via the shared policy list) and merge it onto the tip; audience is the maintainer and verifier.

Assumptions:
- Job 20261106-030952-0f7a: its candidate passed final verification but went stale because governed content moved on the branch (290f31101ef0). The candidate is kept and merged onto the tip; no behavior change is intended beyond resolving merge conflicts, if any.
- Job 20261106-030952-0f7a finding 1 (stale artifact in the verifier's shared target dir, E0432 no CargoShim): environment-only, not a candidate defect; the candidate's CI passed and a release rebuild passed. No code fix.
- Job 20261106-030952-0f7a finding 2 (a later command leaves the wait note, so the job stays held): documented conservative choice in the design doc; kept as is, not changed.
- Job 20261106-030952-0f7a finding 3 (SIGKILL of the wrapper frees the slot while the child cargo may still run): documented trade-off, since the workspace denies unsafe and so cannot exec; kept as is.
- Job 20261106-030952-0f7a finding 4 (outcome: governed content changed since dispatch): addressed by this retry merging onto the new tip and re-verifying.
- Code survey: read crates/athena-agent/src/policy.rs (CARGO_GLOBAL_FLAGS at line 477, check_cargo walk near line 625) and listed crates/athena-cli/src and tests. The entry point is main.rs, which dispatches on argv[0]==cargo to slot_exec::run. The wrapper takes a Class::Producer slot via athena-core/src/slots.rs for build/test/clippy/check, and puts a shim dir first on the producer PATH in athena-agent/src/env.rs. The slot hold is measured by direct tests against a fake cargo. No record or data file is added under crates/, so check_record_file and AnyRecord are not involved.
- Governance: xtask/shard-timings.txt is a deliverable (new slot_exec test unit needs a line). athena.toml, defaults.toml and workflows are not changed.

## Acceptance criteria

1. [behavior] Run `cargo test --locked`: the slot_exec tests pass, including one with one slot and two producer processes where the second blocks while the first holds the slot, then runs, and afterwards try_acquire succeeds.
2. [behavior] Run `cargo test --locked`: a test runs `cargo -q build` and `cargo --locked test` through the wrapper against a fake cargo and asserts both held a slot while running and freed it after; it fails if the subcommand lookup reads only args[0].
3. [behavior] Run `cargo test --locked`: a test where no slot frees within the wait limit asserts the documented message ('no build slot free for ... s; try again'), a non-failure environment exit, and that the job is recorded as held, not failed; the real cargo is never started.
4. [behavior] Run `cargo test --locked`: a test shows `cargo fmt` and `cargo metadata` run through the wrapper without taking a slot even when every slot is held.
5. [behavior] Run `cargo test -p xtask --test ci_shards`: checked_in_timings_cover_every_workspace_test_unit passes. Grep shows xtask/shard-timings.txt has an `athena-cli --test slot_exec` line.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked` from the repo root on the merged tree: all pass.
7. [content] CARGO_GLOBAL_FLAGS in crates/athena-agent/src/policy.rs is the single shared list, Grep shows the wrapper uses it, and no second copy exists; check_cargo behavior is unchanged and its existing tests pass.
8. [content] crates/athena-agent/src/env.rs puts the wrapper directory first on the producer PATH only; the diff leaves verifier commands and environments unchanged.
9. [content] CHANGELOG.md has an entry for producer cargo commands taking a build slot. The diff touches no governance files (athena.toml, defaults.toml, workflows) and no records/.

## Expected deliverables

- crates/athena-cli/src/slot_exec.rs (create)
- crates/athena-cli/tests/slot_exec.rs (create)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-agent/src/env.rs (modify)
- crates/athena-agent/src/policy.rs (modify)
- crates/athena-agent/src/lib.rs (modify)
- crates/athena-core/src/slots.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/Cargo.toml (modify)
- xtask/shard-timings.txt (modify)
- CHANGELOG.md (modify)
- docs/design/agent-and-build-slots.md (modify)

## Out of scope

- A general early check for unregistered test files and better recorded test-check output (separate job)
- Changing the verifier's commands, SnapExec or the local check path
- Slot-owned warm target directories (build step 5)
- Recording queued_s per producer command or the peak-slots statistic
- Changing the policy's allowed commands beyond sharing the global-flag list
- The landing rebuild slot
- Changing the hold-note semantics or SIGKILL behavior noted in findings 2 and 3

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require cargo test, clippy and fmt --check to pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Reuses the verified candidate; scope limited to wrapper, PATH install and tests.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria are exercised by cargo test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check slots held and freed, waits, and held-not-failed state.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Slot gating is enforced by the wrapper on the producer PATH.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The global-flag list is shared, so flag forms cannot skip the queue.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is the slot actually being held, tested directly.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration test in the new crates/athena-cli/tests/slot_exec.rs. A one-slot SlotQueue is built over a temp dir. `install_cargo_shim` builds the producer env; each child is started with env_clear plus that env. Two processes spawn `<shim>/cargo build`, with a fake cargo later on PATH that appends start and end lines to a log and blocks on a release file. The test waits (polling, bounded 10 s) until `queue.waiting()==1`, asserts the second process has logged no start and `try_acquire` is None, then releases. It asserts the log order first-start, first-end, second-start, and that `try_acquire` succeeds afterwards (bounded retry as `take_soon` in slots.rs). Run: `cargo test --locked`. Observes: The test passes: the log shows the second start strictly after the first end, and the slot is acquirable afterwards.
  Fails without the behavior: With no slot in the wrapper both fake cargos start together and the waiting()==1 and ordering assertions fail. A wrapper that does not release the slot fails the final try_acquire.
  Test seam: Production entry point: the `athena` binary run as `cargo` through the symlink (main.rs argv[0] dispatch into slot_exec::run). Fixtures: temp state dir, install_cargo_shim for the env, a /bin/sh fake cargo that only logs and blocks, and SlotQueue::new over the same dir as observer (exists, slots.rs). slot_exec, the dispatch and install_cargo_shim are added by plan steps 1 to 5.
- Criterion 2: Same test file. One test runs `cargo -q build` and one runs `cargo --locked test` through the shim. The fake cargo logs its args and blocks on a release file. While it is blocked the test asserts `queue.try_acquire()` is None. After release it asserts exit 0, that the args reached the fake unchanged, and that the slot is free again. A third test sets ATHENA_SLOT_HELD to a different dir path in the child's env (the case where a producer's own HELD leaks into a test) and still sees the slot held; a fourth sets it to the wrapper's own slot dir and sees the nested cargo run without a slot and without deadlock on a one-slot queue. Run: `cargo test --locked`. Observes: Both gated commands hold the only slot while the fake runs and free it after; args pass through unchanged. The foreign HELD value does not disable the slot, and the matching one does.
  Fails without the behavior: A lookup that reads only args[0] sees `-q` or `--locked`, takes no slot, and `try_acquire` succeeds while the fake runs. A plain HELD flag would fail the foreign-HELD case.
  Test seam: As criterion 1: one-slot queue so any held slot is visible from outside; fake cargo is a shell script holding no slot logic. All added by steps 1 to 5.
- Criterion 3: Three tests. (a) CLI file: the test holds the only slot with `try_acquire`, installs the shim with a small wait (a `CargoShim` whose build_wait_s is 1, so `producer_slot_wait_s` gives 1), runs `cargo build`, and asserts exit 75, stderr containing 'no build slot free for 1 s; try again', the fake cargo's start marker absent, and `read_wait_note` returning SlotWait{limit_s:1}. (b) env.rs unit test `the_wrapper_wait_is_below_the_bash_timeout`: `producer_slot_wait_s(1800, 600000)` is 540, below 600; `(30, 600000)` is 30; `(1800, 50000)` is 1; and `install_cargo_shim` with the shipped numbers sets ATHENA_SLOT_WAIT_S to 540. (c) run/tests.rs `a_producer_slot_wait_note_holds_the_job_and_does_not_fail_it`: a script for Role::Producer writes the note with `slots::write_wait_note` to `spec.env["ATHENA_SLOT_NOTE"]`; `Athena::go` returns an error that downcasts to SlotWait, state is Dispatched with no outcome or score, `read_slot_hold` is Some, `read_hold` is None, and the 'held       environment: build slot wait' line is printed. A later `retry` lands the job once no note is written. Run: `cargo test --locked`. Observes: The wrapper test shows exit 75, the message, an unstarted real cargo and a written note. The env test shows the default wait is below the Bash timeout. The run test shows the job held, not failed.
  Fails without the behavior: Without the exit-75 path the wrapper starts cargo or exits 1. Without the cap, (b) fails (1800 not below 600) and the hold is unreachable in real runs. Without the note read in producer_loop, (c) gets a failed or landed job instead of a SlotWait hold.
  Test seam: (a) as criterion 1 with the slot held from outside. (b) `env::producer_slot_wait_s` and `install_cargo_shim`, added by step 2. (c) Production entry point `Athena::go` through producer_loop and produce_and_verify; helpers `athena_with`/`counting` and the FakeAgentDriver script closure that receives the invocation env exist (run/tests.rs, fake.rs). The double only writes the note, which is what the wrapper does in (a); reading and holding the note is production code from steps 4 to 6.
- Criterion 4: CLI test with a one-slot queue whose only slot the test holds throughout. It runs `cargo fmt --check` and `cargo metadata` through the shim with a short wait. For each it asserts the fake cargo ran, exit 0, elapsed time under the wait limit, and no wait note written. Run: `cargo test --locked`. Observes: Both commands run to completion at once while every slot is held.
  Fails without the behavior: If every subcommand took a slot both would block for the wait, exit 75 and never start the fake cargo.
  Test seam: As criterion 1 plus a slot held from outside; added by the same steps.
- Criterion 5: Add the line `athena-cli --test slot_exec <seconds>` (an estimate) to xtask/shard-timings.txt, between `review` and `status_build_measurement`, then run the xtask check that every workspace test unit has a timing. Run: `cargo test -p xtask --test ci_shards`. Observes: checked_in_timings_cover_every_workspace_test_unit passes; Grep on xtask/shard-timings.txt shows the slot_exec line.
  Fails without the behavior: Without the line the new test unit has no timing and the coverage test fails.
  Test seam: Production entry point: the xtask ci_shards test reading the workspace test targets and the checked-in file; both exist. Step 8 adds the line.
- Criterion 6: Run the three Athena checks on the final tree from the repo root: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. Run: `cargo fmt --check`. Observes: All three exit 0 (clippy and test are the same commands as in the brief).
  Fails without the behavior: Any formatting, lint or test regression in the merged tree fails one of the three.
  Test seam: The repository's own cargo commands; nothing is injected.
- Criterion 7: policy.rs unit test `the_shared_global_flag_list_finds_the_subcommand_check_cargo_accepts` loops over CARGO_GLOBAL_FLAGS. For each flag it asserts `cargo_subcommand([flag, "build"]) == Some("build")` and that `check_cargo` allows `cargo <flag> build`; an unlisted flag gives None and check_cargo refuses it. Existing check_cargo tests run unchanged. Grep for CARGO_GLOBAL_FLAGS shows one definition and the wrapper reaching it through `policy::cargo_subcommand`. Run: `cargo test -p athena-agent --lib`. Observes: The new test and every existing policy test pass; the constant changes only in visibility.
  Fails without the behavior: If the wrapper kept its own list the agreement test would not cover its lookup; if check_cargo changed, its existing message tests would fail.
  Test seam: Production entry points `policy::check_cargo` via the Policy built as the existing policy tests do (exists) and `policy::cargo_subcommand` (step 1).
- Criterion 8: (b) is the demonstration: a run/tests.rs test records `spec.env` by role over a full job through `Athena::go` with the FakeAgentDriver script closure; the producer env's PATH starts with `<job_dir>/agent/bin` and has the cargo symlink, the reader and verifier envs have PATH equal to the parent's and no ATHENA_SLOT_* keys. (a) An env.rs unit test `the_cargo_shim_goes_first_on_the_producer_path_only` checks install_cargo_shim on a Side::Produce env (shim first, stale link replaced, variables set); it is described as a regression guard, and the Verify/Read half is dropped because `build` is unchanged and cannot fail (finding 5). Inspect `git diff` of env.rs to confirm `build` is unchanged. Run: `cargo test --locked`. Observes: The producer PATH starts with the shim dir; reader and verifier PATH values are the parent's.
  Fails without the behavior: If the shim were installed in `build`, or for every side, the reader and verifier assertions in the run test fail.
  Test seam: `Athena::go` with the FakeAgentDriver script closure (exists); `install_cargo_shim` and its call in producer.rs are added by steps 2 and 5.
- Criterion 9: Add a CHANGELOG.md entry under [Unreleased] / Added and inspect the diff file list. Run: `git diff --stat`. Observes: The stat lists CHANGELOG.md and the deliverables only; no athena.toml, defaults.toml, workflow or records/ path.
  Fails without the behavior: A missing entry or a governance file in the stat shows the criterion is unmet.
  Test seam: No code seam; the diff of the producer's work tree. Step 9 adds the entry.

Planned files:

- crates/athena-cli/src/slot_exec.rs (create)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/slot_exec.rs (create)
- crates/athena-agent/src/policy.rs (modify)
- crates/athena-agent/src/env.rs (modify)
- crates/athena-agent/src/lib.rs (modify)
- crates/athena-core/src/slots.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/Cargo.toml (modify)
- xtask/shard-timings.txt (modify)
- CHANGELOG.md (modify)
- docs/design/agent-and-build-slots.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 6ff1fd7332eb → candidate 5316504b1c5f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  13 +-
 crates/athena-agent/src/env.rs         | 243 +++++++++++++++++++++++++++
 crates/athena-agent/src/policy.rs      | 113 +++++++++++--
 crates/athena-cli/src/lib.rs           |   1 +
 crates/athena-cli/src/main.rs          |   6 +-
 crates/athena-cli/src/slot_exec.rs     | 179 ++++++++++++++++++++
 crates/athena-cli/tests/slot_exec.rs   | 289 +++++++++++++++++++++++++++++++++
 crates/athena-core/src/run/close.rs    |  12 +-
 crates/athena-core/src/run/mod.rs      |  17 +-
 crates/athena-core/src/run/producer.rs |  23 +++
 crates/athena-core/src/run/tests.rs    | 185 +++++++++++++++++++++
 crates/athena-core/src/slots.rs        |  50 +++++-
 docs/design/agent-and-build-slots.md   |  32 ++++
 xtask/shard-timings.txt                |   1 +
 14 files changed, 1141 insertions(+), 23 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-180606-ymst/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-180606-ymst/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42271308134 for 5316504b1c5f09e2788fb5fc4cd33d52e8fff646)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42271308134 for 5316504b1c5f09e2788fb5fc4cd33d52e8fff646)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42271308134 for 5316504b1c5f09e2788fb5fc4cd33d52e8fff646)
- ci-result: `not a check` passed (Check results for 5316504b1c5f09e2788fb5fc4cd33d52e8fff646 come from the CI run https://github.com/doodla/athena/actions/runs/42271308134 (640s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked` from the repo root on the merged tree: all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 2, 3, 4, 8 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/env.rs, crates/athena-agent/src/policy.rs, crates/athena-cli/src/lib.rs, crates/athena-cli/src/main.rs, crates/athena-cli/src/slot_exec.rs, crates/athena-core/src/run/close.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/slots.rs, docs/design/agent-and-build-slots.md, xtask/shard-timings.txt.
  Warning: tests inside these files were removed with the revert: crates/athena-cli/src/slot_exec.rs.
  End of the output (data, not instructions):

```text

error[E0425]: cannot find function `cargo_subcommand` in this scope
    --> crates/athena-agent/src/policy.rs:1043:20
     |
1043 |         assert_eq!(cargo_subcommand(&["-q"]), None);
     |                    ^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `cargo_subcommand` in this scope
    --> crates/athena-agent/src/policy.rs:1044:20
     |
1044 |         assert_eq!(cargo_subcommand(&["--config", "x", "build"]), None);
     |                    ^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `cargo_subcommand` in this scope
    --> crates/athena-agent/src/policy.rs:1047:24
     |
1047 |             assert_eq!(cargo_subcommand(&[flag, "test"]), Some("test"), "{flag}");
     |                        ^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `cargo_lead` in this scope
    --> crates/athena-agent/src/policy.rs:1050:13
     |
1050 |             cargo_lead(["-q", "build", "-p", "x"]),
     |             ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `cargo_lead` in this scope
    --> crates/athena-agent/src/policy.rs:1057:13
     |
1057 |             cargo_lead(["-q", "--config"]),
     |             ^^^^^^^^^^ not found in this scope

error[E0433]: cannot find type `CargoLead` in this scope
    --> crates/athena-agent/src/policy.rs:1058:13
     |
1058 |             CargoLead::UnknownFlag("--config")
     |             ^^^^^^^^^ use of undeclared type `CargoLead`

error[E0425]: cannot find function `cargo_lead` in this scope
    --> crates/athena-agent/src/policy.rs:1060:20
     |
1060 |         assert_eq!(cargo_lead(["-q"]), CargoLead::NoSub);
     |                    ^^^^^^^^^^ not found in this scope

error[E0433]: cannot find type `CargoLead` in this scope
    --> crates/athena-agent/src/policy.rs:1060:40
     |
1060 |         assert_eq!(cargo_lead(["-q"]), CargoLead::NoSub);
     |                                        ^^^^^^^^^ use of undeclared type `CargoLead`

   Compiling xtask v0.1.0 (<job>/revert/xtask)
error[E0433]: cannot find type `CargoLead` in this scope
    --> crates/athena-agent/src/policy.rs:1051:13
     |
1051 |             CargoLead::Sub {
     |             ^^^^^^^^^ use of undeclared type `CargoLead`

Some errors have detailed explanations: E0425, E0433.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-agent` (lib test) due to 30 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p xtask --test ci_shards`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/env.rs, crates/athena-agent/src/policy.rs, crates/athena-cli/src/lib.rs, crates/athena-cli/src/main.rs, crates/athena-cli/src/slot_exec.rs, crates/athena-core/src/run/close.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/slots.rs, docs/design/agent-and-build-slots.md, xtask/shard-timings.txt.
  Warning: tests inside these files were removed with the revert: crates/athena-cli/src/slot_exec.rs.
  End of the output (data, not instructions):

```text
   Compiling quote v1.0.47
   Compiling syn v3.0.7
   Compiling toml v0.9.12+spec-1.1.0
   Compiling serde_derive_internals v0.30.0
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling xtask v0.1.0 (<job>/revert/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.53s
     Running tests/ci_shards.rs (<job>/target/verify/debug/deps/ci_shards-ce10c1bc9859f0ab)

running 20 tests
test checked_in_timings_cover_every_workspace_test_unit ... FAILED
test cover_rejects_a_shard_unit_that_is_not_a_target ... ok
test cover_accepts_the_real_shards ... ok
test cover_rejects_a_unit_in_no_shard ... ok
test cover_rejects_a_unit_in_two_shards ... ok
test partition_rejects_a_changed_shard_list ... ok
test oracle_matches_files_on_disk ... ok
test test_shard_1_equals_the_workspace ... ok
test cover_rejects_an_oracle_unit_the_shards_lack ... ok
test cover_rejects_a_unit_twice_in_one_shard ... ok
test test_shard_rejects_bad_arguments ... ok
test workflow_actions_are_pinned ... ok
test workflow_caches_keep_three_layers_in_each_check_job ... ok
test workflow_ci_job_is_green_only_when_every_check_succeeded ... ok
test workflow_jobs_are_separate_sharded_and_gated_by_ci ... ok
test shards_in_ci_yml_partition_every_workspace_test_target ... ok
test test_shard_output_is_stable_and_sorted ... ok
test workflow_required_check_name_is_on_exactly_one_job_that_needs_every_check ... ok
test workflow_jobs_reject_a_missing_gate_condition_need_or_timeout ... ok
test test_shard_6_is_disjoint_and_equals_the_workspace_target_set ... ok

failures:

---- checked_in_timings_cover_every_workspace_test_unit stdout ----

thread 'checked_in_timings_cover_every_workspace_test_unit' (4821) panicked at xtask/tests/ci_shards.rs:420:5:
assertion `left == right` failed
  left: {"athena-agent --doc", "athena-agent --lib", "athena-cap-change --doc", "athena-cap-change --lib", "athena-cap-study --doc", "athena-cap-study --lib", "athena-cli --bin athena", "athena-cli --doc", "athena-cli --lib", "athena-cli --test audience", "athena-cli --test auditor", "athena-cli --test backtest", "athena-cli --test binary", "athena-cli --test budgets", "athena-cli --test code_health_hooks", "athena-cli --test cutover_fixture", "athena-cli --test drain", "athena-cli --test external_calls", "athena-cli --test focus", "athena-cli --test history_pr_head", "athena-cli --test introspection", "athena-cli --test job_plan", "athena-cli --test labeled", "athena-cli --test lifecycle", "athena-cli --test live", "athena-cli --test mirrors", "athena-cli --test ontology", "athena-cli --test proposals", "athena-cli --test reconcile", "athena-cli --test retry", "athena-cli --test review", "athena-cli --test status_build_measurement", "athena-cli --test status_durations", "athena-cli --test status_snapshot", "athena-cli --test steward", "athena-cli --test trailer_stop", "athena-cli --test version", "athena-cli --test watcher", "athena-cli --test work_decisions", "athena-context --doc", "athena-context --lib", "athena-core --doc", "athena-core --lib", "athena-core --test external_calls_guard", "athena-core --test refusal_corpus", "athena-data --doc", "athena-data --lib", "athena-data --test repo_config", "athena-gate --doc", "athena-gate --lib", "athena-git --doc", "athena-git --lib", "athena-score --doc", "athena-score --lib", "xtask --bin xtask", "xtask --test ci_shards", "xtask --test fingerprint", "xtask --test policy", "xtask --test site_page", "xtask --test spec", "xtask --test walltime"}
 right: {"athena-agent --doc", "athena-agent --lib", "athena-cap-change --doc", "athena-cap-change --lib", "athena-cap-study --doc", "athena-cap-study --lib", "athena-cli --bin athena", "athena-cli --doc", "athena-cli --lib", "athena-cli --test audience", "athena-cli --test auditor", "athena-cli --test backtest", "athena-cli --test binary", "athena-cli --test budgets", "athena-cli --test code_health_hooks", "athena-cli --test cutover_fixture", "athena-cli --test drain", "athena-cli --test external_calls", "athena-cli --test focus", "athena-cli --test history_pr_head", "athena-cli --test introspection", "athena-cli --test job_plan", "athena-cli --test labeled", "athena-cli --test lifecycle", "athena-cli --test live", "athena-cli --test mirrors", "athena-cli --test ontology", "athena-cli --test proposals", "athena-cli --test reconcile", "athena-cli --test retry", "athena-cli --test review", "athena-cli --test slot_exec", "athena-cli --test status_build_measurement", "athena-cli --test status_durations", "athena-cli --test status_snapshot", "athena-cli --test steward", "athena-cli --test trailer_stop", "athena-cli --test version", "athena-cli --test watcher", "athena-cli --test work_decisions", "athena-context --doc", "athena-context --lib", "athena-core --doc", "athena-core --lib", "athena-core --test external_calls_guard", "athena-core --test refusal_corpus", "athena-data --doc", "athena-data --lib", "athena-data --test repo_config", "athena-gate --doc", "athena-gate --lib", "athena-git --doc", "athena-git --lib", "athena-score --doc", "athena-score --lib", "xtask --bin xtask", "xtask --test ci_shards", "xtask --test fingerprint", "xtask --test policy", "xtask --test site_page", "xtask --test spec", "xtask --test walltime"}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    checked_in_timings_cover_every_workspace_test_unit

test result: FAILED. 19 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.01s

error: test failed, to rerun pass `-p xtask --test ci_shards`
error: 1 target failed:
    `-p xtask --test ci_shards`
```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --check`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 7 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-agent --lib`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/env.rs, crates/athena-agent/src/policy.rs, crates/athena-cli/src/lib.rs, crates/athena-cli/src/main.rs, crates/athena-cli/src/slot_exec.rs, crates/athena-core/src/run/close.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/slots.rs, docs/design/agent-and-build-slots.md, xtask/shard-timings.txt.
  Warning: tests inside these files were removed with the revert: crates/athena-cli/src/slot_exec.rs.
  End of the output (data, not instructions):

```text
1042 |         assert_eq!(cargo_subcommand(&[]), None);
     |                    ^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `cargo_subcommand` in this scope
    --> crates/athena-agent/src/policy.rs:1043:20
     |
1043 |         assert_eq!(cargo_subcommand(&["-q"]), None);
     |                    ^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `cargo_subcommand` in this scope
    --> crates/athena-agent/src/policy.rs:1044:20
     |
1044 |         assert_eq!(cargo_subcommand(&["--config", "x", "build"]), None);
     |                    ^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `cargo_subcommand` in this scope
    --> crates/athena-agent/src/policy.rs:1047:24
     |
1047 |             assert_eq!(cargo_subcommand(&[flag, "test"]), Some("test"), "{flag}");
     |                        ^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `cargo_lead` in this scope
    --> crates/athena-agent/src/policy.rs:1050:13
     |
1050 |             cargo_lead(["-q", "build", "-p", "x"]),
     |             ^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `cargo_lead` in this scope
    --> crates/athena-agent/src/policy.rs:1057:13
     |
1057 |             cargo_lead(["-q", "--config"]),
     |             ^^^^^^^^^^ not found in this scope

error[E0433]: cannot find type `CargoLead` in this scope
    --> crates/athena-agent/src/policy.rs:1058:13
     |
1058 |             CargoLead::UnknownFlag("--config")
     |             ^^^^^^^^^ use of undeclared type `CargoLead`

error[E0425]: cannot find function `cargo_lead` in this scope
    --> crates/athena-agent/src/policy.rs:1060:20
     |
1060 |         assert_eq!(cargo_lead(["-q"]), CargoLead::NoSub);
     |                    ^^^^^^^^^^ not found in this scope

error[E0433]: cannot find type `CargoLead` in this scope
    --> crates/athena-agent/src/policy.rs:1060:40
     |
1060 |         assert_eq!(cargo_lead(["-q"]), CargoLead::NoSub);
     |                                        ^^^^^^^^^ use of undeclared type `CargoLead`

error[E0433]: cannot find type `CargoLead` in this scope
    --> crates/athena-agent/src/policy.rs:1051:13
     |
1051 |             CargoLead::Sub {
     |             ^^^^^^^^^ use of undeclared type `CargoLead`

Some errors have detailed explanations: E0425, E0433.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-agent` (lib test) due to 30 previous errors
```
- fails-without-9: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 9 (`git diff --stat`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
