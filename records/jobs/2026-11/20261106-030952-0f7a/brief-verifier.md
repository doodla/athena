# Job 20261106-030952-0f7a: verify "Retry: producer cargo build/test/clippy/check take a build slot"

## Request (verbatim)

> try again on 20261106-002344-jn5t

## Reading (accepted by the user)

Retry of job 20261106-002344-jn5t (build step 4): an `athena`-linked `cargo` wrapper first on the producer PATH holds a build slot for build, test, clippy and check, finding the subcommand after global flags via the policy's shared list. Fixes the prior failure by registering the new test file in xtask/shard-timings.txt and adding the fmt/metadata no-slot test; audience is the maintainer and verifier.

Assumptions:
- Code survey: read crates/athena-agent/src/policy.rs (CARGO_GLOBAL_FLAGS, check_cargo), crates/athena-agent/src/env.rs (producer PATH), crates/athena-core/src/slots.rs (SlotQueue, Class::Producer, try_acquire), crates/athena-core/src/run/producer.rs and crates/athena-cli/src/main.rs (entry point dispatching on argv[0]). Slot wait is measured by a test with a real SlotQueue and a fake cargo. No record or data file is added under crates/, so check_record_file/AnyRecord are unaffected.
- Retry of job 20261106-002344-jn5t: its candidate design is kept (athena binary linked as `cargo`, dispatch on argv[0]); the findings are addressed as: (a) blocker, missing line in xtask/shard-timings.txt: add `athena-cli --test slot_exec <seconds>`; (b) major, criterion 4 evidence: the fmt/metadata no-slot test is kept and criteria cite runnable commands; (c) minor, files outside the list: crates/athena-cli/src/lib.rs, crates/athena-core/src/run/mod.rs and run/tests.rs are now listed as deliverables; (d) nit, no `build-slot` subcommand: the argv[0] deviation is accepted and the design doc says so.
- The earlier introspection's proposal for a general early check that every test file is registered in shard-timings.txt is a separate job, not part of this retry; here the file is simply registered by hand. The unlisted athena-agent/src/lib.rs and athena-cli/Cargo.toml were not changed by the candidate and change only if needed.
- Existing xtask test xtask/tests/ci_shards.rs checked_in_timings_cover_every_workspace_test_unit is the guard for the registration.

## Acceptance criteria

1. [behavior] Run `cargo test --locked`: the slot_exec tests pass, including one with one slot and two producer processes where the second blocks while the first holds the slot, then runs, and afterwards try_acquire succeeds.
2. [behavior] Run `cargo test --locked`: a test runs `cargo -q build` and `cargo --locked test` through the wrapper against a fake cargo and asserts both held a slot while running and freed it after; it fails if the subcommand lookup reads only args[0].
3. [behavior] Run `cargo test --locked`: a test where no slot frees within the wait limit asserts the documented message ('no build slot free for ... s; try again'), a non-failure environment exit, and that the job is recorded as held (`environment: build slot wait`), not failed; the real cargo is never started.
4. [behavior] Run `cargo test --locked`: a test shows `cargo fmt` and `cargo metadata` run through the wrapper without taking a slot even when every slot is held.
5. [behavior] Run `cargo test -p xtask --test ci_shards`: checked_in_timings_cover_every_workspace_test_unit passes. Grep shows xtask/shard-timings.txt has an `athena-cli --test slot_exec` line.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked` from the repo root: all pass. Evidence cites these commands.
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

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy criteria.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands to run.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check slots taken and freed, not that wiring exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The shard-timings registration is fixed and the shared flag list avoids drift.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: The early-check infrastructure is left out of this retry.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration test with the real `athena` binary run as `cargo` through a symlink in a temp shim dir (target `env!("CARGO_BIN_EXE_athena")`). A fake `cargo` shell script sits after the shim dir on PATH: it writes `started-<tag>`, waits for `release-<tag>`, then exits 0. One slot, a 30 s wait, and the slots dir of a real `SlotQueue` in a tempdir. Start producer process A (`cargo build`) and wait for `started-A`. Start B and poll until `queue.waiting() == 1`, bounded at 10 s. Assert B has no `started-B` and `try_acquire()` is None. Release A, wait for `started-B`, release B. After both exit assert `try_acquire()` is Some. Run: `cargo test --locked -p athena-cli --test slot_exec a_second_producer_cargo_waits_for_the_slot_then_runs`. Observes: While A holds the only slot B is queued and has not run; after A's release B runs; both exit 0 and the slot is free again.
  Fails without the behavior: If the wrapper took no slot, B would write `started-B` while A held and `waiting() == 1` would never be reached. If the slot were not freed at exit, the final `try_acquire` would be None.
  Test seam: Entry point: `slot_exec::run` through the `athena` binary as `cargo` (argv[0] dispatch added by step 5). Fixtures: a real `SlotQueue` over a tempdir (test-created, read with `SlotQueue::try_acquire`, which exists, slots.rs:168) and a fake cargo script that only records and waits. They are injected by building the child's env with the production `CargoShim::apply` (step 3). The fake holds none of the slot logic. The seam is added by steps 3-5 and 9.
- Criterion 2: Same fixtures, one slot. Run `cargo -q build`, then `cargo --locked test`, through the shim. The fake records its argv, writes `started`, waits for `release`. At each `started` assert `try_acquire()` is None. After exit assert `try_acquire()` is Some and the fake received exactly the original argv. The policy unit tests assert `cargo_subcommand(&["-q","build"]) == Some("build")`, `["--locked","test"] == Some("test")`, `["-vv","check"] == Some("check")`, and None for `["--config","x"]` and `[]`. Existing check_cargo tests run in the same module. Run: `cargo test --locked -p athena-cli --test slot_exec global_flags_before_the_subcommand_still_hold_a_slot && cargo test --locked -p athena-agent policy`. Observes: Both commands hold the slot while the fake runs and free it after; the real cargo gets the arguments unchanged.
  Fails without the behavior: A lookup of only args[0] sees `-q` or `--locked`, takes no slot, and `try_acquire` returns Some while the fake runs, failing the assert.
  Test seam: Entry points: `slot_exec::run` via the binary as `cargo`, and `athena_agent::policy::cargo_subcommand` (added by step 2). Fixtures as in criterion 1. The seam is added by steps 2-5 and 9.
- Criterion 3: (a) CLI test: settings are one slot and a 1 s wait; the test holds the slot with `try_acquire()` and runs `cargo build` through the shim. Assert exit code 75, stderr exactly `no build slot free for 1 s; try again`, no `started` file from the fake, and `slots::read_wait_note` at the note path returns a SlotWait with `limit_s == 1`. (b) Core test: `athena_with` plus `counting(..)` drive a fake producer whose script, when role is Producer, writes the note with the production helper `slots::write_wait_note` at the path in `spec.env`, as the wrapper would. Call `athena.go(..)`. Assert the error downcasts to SlotWait, the `held       environment: build slot wait` line was printed, the records show no outcome and no score, `read_slot_hold` equals the wait, the verifier ran 0 times, and a retry after the note is gone runs the producer again and lands. A second case: a producer that wrote the note, then a later cargo got a slot and cleared it, is not held. (c) Env unit test: parse the shipped athena.toml (`include_str!("../../../athena.toml")`, `Config::parse`, config.rs:616) and assert `CargoShim::new(..).wait_s` is below `bash_default_timeout_ms / 1000` and equals 300; a small build_wait_s (such as 1) is kept as is. Run: `cargo test --locked -p athena-cli --test slot_exec a_wait_past_the_limit_exits_75_and_never_starts_cargo && cargo test --locked -p athena-core a_producer_whose_last_cargo_waited_out_the_slot_holds_the_job && cargo test --locked -p athena-agent the_shims_wait_is_below_the_bash_timeout_of_the_shipped_config`. Observes: The wrapper gives the documented message and a non-failure exit, writes a note, and never starts cargo. A job whose producer ended on that note is held as `environment: build slot wait`, not failed, and resumes. In production the wrapper's limit is under the Bash timeout, so this path is reachable before Bash kills the command.
  Fails without the behavior: Without the wrapper's note, the CLI test finds no note. Without the `producer_loop` check, `go` runs the checks and verifier on unbuilt work and the core test sees a verifier run and a closed outcome. Without the cap, the env test fails (1800 is not below 600).
  Test seam: Entry points: `slot_exec::run` through the binary (CLI); `Athena::go` with `producer_loop` and `produce_and_verify` (core, signatures unchanged); `CargoShim::new` (env). Doubles: a real SlotQueue plus a fake cargo script (CLI); the existing `FakeAgentDriver` script via `athena_with`/`counting` (tests.rs:187 and :2152), which writes the note through the production helper so the hold logic is not in the double (core). The shipped config file is the fixture for (c). Steps 3, 4, 5, 6 and 7 add the seam; `athena_with`, `counting` and `slot_queue` exist.
- Criterion 4: One slot, a 1 s wait. The test holds the slot with `try_acquire()` and runs `cargo fmt --check` and `cargo metadata --no-deps` through the shim. The fake cargo writes its argv and exits 0. Assert both exit 0, the fake ran with the right argv, no wait note exists, and the slot is still held by the test at the end. Run: `cargo test --locked -p athena-cli --test slot_exec fmt_and_metadata_take_no_slot_even_when_every_slot_is_held`. Observes: Both commands run to completion with every slot held and take no slot.
  Fails without the behavior: If every subcommand took a slot, both would wait 1 s, exit 75, never reach the fake and write a note, failing the asserts.
  Test seam: Same as criterion 1: `slot_exec::run` through the binary as `cargo`, a real SlotQueue held by the test, a fake cargo script, env from `CargoShim::apply`. Added by steps 3-5 and 9.
- Criterion 5: Run the existing guard, which compares the units in xtask/shard-timings.txt with the workspace's test units. Then use the Grep tool on xtask/shard-timings.txt for `athena-cli --test slot_exec` (plain grep is not on the verifier's command list). Run: `cargo test --locked -p xtask --test ci_shards checked_in_timings_cover_every_workspace_test_unit`. Observes: The guard passes with the new test file registered, and the file has the line `athena-cli --test slot_exec 8.00`.
  Fails without the behavior: Without the line the set of timed units lacks `athena-cli --test slot_exec` and the equality assert (ci_shards.rs:420) fails.
  Test seam: Entry point: the existing test `checked_in_timings_cover_every_workspace_test_unit` (ci_shards.rs:411), which reads the real file; it exists. Step 10 adds the line.
- Criterion 6: Run the three CI commands from the repo root and cite their output. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: A fmt, lint or test failure anywhere in the workspace, including the new tests and the shard guard, fails the matching command.
  Test seam: The commands are the checks Athena runs; no seam is needed.
- Criterion 7: Read the diff: `CARGO_GLOBAL_FLAGS` is `pub`, one new helper holds the leading-flag walk, `check_cargo` and `cargo_subcommand` both call it, and `check_cargo`'s error strings are unchanged. Use the Grep tool on crates/athena-cli/src/slot_exec.rs: it calls `cargo_subcommand` and has no flag literal list (`"--locked"` and `"--offline"` are absent). Run the policy tests: the existing check_cargo cases (`listed_git_and_cargo_commands_are_allowed`, the `--target-dir` and `--manifest-path` denials, an unknown global flag error text, `cargo` without a subcommand) plus the new `cargo_subcommand` tests. Run: `git diff -- crates/athena-agent/src/policy.rs && cargo test --locked -p athena-agent policy`. Observes: One list and one walk; check_cargo behavior and its tests are unchanged and pass.
  Fails without the behavior: A second list in slot_exec.rs would appear in the diff or the Grep. A changed error string or walk in check_cargo breaks its existing tests.
  Test seam: Entry points: `check_cargo` (via `Policy::check`) and `cargo_subcommand`. Both are production code and no double is used. Step 2 adds the helper.
- Criterion 8: The diff shows `build` and `EnvSpec` unchanged, with only `CargoShim` and its helpers added. The core test (narrowed to producer versus verifier, finding 3) uses the existing fake agent Script, which receives an `AgentInvocation`: for the Producer role it records `spec.env`, and for the Verifier role it does too. After `go` it asserts the producer's PATH starts with `<job_dir>/shim` and has the slot variables, and the verifier's env has none of those variables and a PATH equal to the parent's. Checks are not run through the agent driver (checks.rs:556, :826 call `env::build` and run through SnapExec), so for them an env unit test asserts `env::build(..)` output has no `ATHENA_CARGO_SHIM_DIR` or slot variable and PATH is the parent's. The diff shows no change to checks.rs. Run: `git diff -- crates/athena-agent/src/env.rs && cargo test --locked -p athena-core only_the_producer_gets_the_cargo_shim_first_on_path && cargo test --locked -p athena-agent build_adds_no_shim_variables`. Observes: Only the producer's PATH changes; verifier and check environments are as before.
  Fails without the behavior: If the shim went on every role's PATH, or into `env::build`, the verifier and `build` assertions fail.
  Test seam: Entry points: `Athena::go`, then `producer_loop` (where the shim is applied) and `verification.rs`'s `agent_spec` call for the verifier; and `env::build`. Double: the existing `FakeAgentDriver` Script capturing `spec.env` (exists); the capture is added by step 8.
- Criterion 9: Read the diff's file list and the CHANGELOG hunk. `CHANGELOG.md` has an entry under [Unreleased] / Added saying producer cargo build, test, clippy and check take a build slot, with the root cause and the held-job behavior and the cap on the wrapper's wait. The list has no athena.toml, defaults.toml, `.github/workflows` file or path under `records/`. Run: `git diff --stat`. Observes: The changelog entry is present and no governance file or records path appears.
  Fails without the behavior: A missing entry or a touched governance file would show in the diff.
  Test seam: The diff itself; no seam is needed. Step 11 writes the entry.

Planned files:

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

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 3234e9d408e5 → candidate 9f979e468a64. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  12 ++
 crates/athena-agent/src/env.rs         | 243 +++++++++++++++++++++++++++
 crates/athena-agent/src/policy.rs      | 113 +++++++++++--
 crates/athena-cli/src/lib.rs           |   1 +
 crates/athena-cli/src/main.rs          |   6 +-
 crates/athena-cli/src/slot_exec.rs     | 179 ++++++++++++++++++++
 crates/athena-cli/tests/slot_exec.rs   | 289 +++++++++++++++++++++++++++++++++
 crates/athena-core/src/run/mod.rs      |  15 +-
 crates/athena-core/src/run/producer.rs |  23 +++
 crates/athena-core/src/run/tests.rs    | 185 +++++++++++++++++++++
 crates/athena-core/src/slots.rs        |  50 +++++-
 docs/design/agent-and-build-slots.md   |  32 ++++
 xtask/shard-timings.txt                |   1 +
 13 files changed, 1133 insertions(+), 16 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261106-030952-0f7a/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261106-030952-0f7a/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42158900636 for 9f979e468a6454e7443cb32cac31d2dc1f628f3c)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42158900636 for 9f979e468a6454e7443cb32cac31d2dc1f628f3c)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42158900636 for 9f979e468a6454e7443cb32cac31d2dc1f628f3c)
- ci-result: `not a check` passed (Check results for 9f979e468a6454e7443cb32cac31d2dc1f628f3c come from the CI run https://github.com/doodla/athena/actions/runs/42158900636 (639s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test --locked` from the repo root: all pass. Evidence cites these commands.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli --test slot_exec a_second_producer_cargo_waits_for_the_slot_then_runs`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/env.rs, crates/athena-agent/src/policy.rs, crates/athena-cli/src/lib.rs, crates/athena-cli/src/main.rs, crates/athena-cli/src/slot_exec.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/slots.rs, docs/design/agent-and-build-slots.md, xtask/shard-timings.txt.
  Warning: tests inside these files were removed with the revert: crates/athena-cli/src/slot_exec.rs.
  End of the output (data, not instructions):

```text
   Compiling indexmap v2.14.2
   Compiling schemars v1.2.2
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling utf8parse v0.2.2
   Compiling anstyle-parse v1.0.0
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle v1.0.14
   Compiling colorchoice v1.0.5
   Compiling anstyle-query v1.1.5
   Compiling anstream v1.0.0
   Compiling heck v0.5.0
   Compiling clap_lex v1.1.1
   Compiling strsim v0.11.1
   Compiling clap_builder v4.6.7
   Compiling clap_derive v4.6.7
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling clap v4.6.7
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0432]: unresolved import `athena_agent::env::CargoShim`
 --> crates/athena-cli/tests/slot_exec.rs:8:5
  |
8 | use athena_agent::env::CargoShim;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `CargoShim` in `env`

error[E0425]: cannot find function `wait_note_path` in module `slots`
  --> crates/athena-cli/tests/slot_exec.rs:45:22
   |
45 |         note: slots::wait_note_path(&job_dir),
   |                      ^^^^^^^^^^^^^^ not found in `slots`

error[E0425]: cannot find function `read_wait_note` in module `slots`
   --> crates/athena-cli/tests/slot_exec.rs:229:33
    |
229 |     let wait: SlotWait = slots::read_wait_note(&fx.shim.note).expect("a wait note");
    |                                 ^^^^^^^^^^^^^^ not found in `slots`

error[E0425]: cannot find function `read_wait_note` in module `slots`
   --> crates/athena-cli/tests/slot_exec.rs:240:23
    |
240 |     assert_eq!(slots::read_wait_note(&fx.shim.note), None);
    |                       ^^^^^^^^^^^^^^ not found in `slots`

error[E0425]: cannot find function `read_wait_note` in module `slots`
   --> crates/athena-cli/tests/slot_exec.rs:264:27
    |
264 |         assert_eq!(slots::read_wait_note(&fx.shim.note), None, "no wait note");
    |                           ^^^^^^^^^^^^^^ not found in `slots`

Some errors have detailed explanations: E0425, E0432.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-cli` (test "slot_exec") due to 5 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test --locked -p athena-cli --test slot_exec global_flags_before_the_subcommand_still_hold_a_slot && cargo test --locked -p athena-agent policy`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test --locked -p athena-cli --test slot_exec a_wait_past_the_limit_exits_75_and_never_starts_cargo && cargo test --locked -p athena-core a_producer_whose_last_cargo_waited_out_the_slot_holds_the_job && cargo test --locked -p athena-agent the_shims_wait_is_below_the_bash_timeout_of_the_shipped_config`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli --test slot_exec fmt_and_metadata_take_no_slot_even_when_every_slot_is_held`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/env.rs, crates/athena-agent/src/policy.rs, crates/athena-cli/src/lib.rs, crates/athena-cli/src/main.rs, crates/athena-cli/src/slot_exec.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/slots.rs, docs/design/agent-and-build-slots.md, xtask/shard-timings.txt.
  Warning: tests inside these files were removed with the revert: crates/athena-cli/src/slot_exec.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0432]: unresolved import `athena_agent::env::CargoShim`
 --> crates/athena-cli/tests/slot_exec.rs:8:5
  |
8 | use athena_agent::env::CargoShim;
  |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `CargoShim` in `env`

error[E0425]: cannot find function `wait_note_path` in module `slots`
  --> crates/athena-cli/tests/slot_exec.rs:45:22
   |
45 |         note: slots::wait_note_path(&job_dir),
   |                      ^^^^^^^^^^^^^^ not found in `slots`

error[E0425]: cannot find function `read_wait_note` in module `slots`
   --> crates/athena-cli/tests/slot_exec.rs:229:33
    |
229 |     let wait: SlotWait = slots::read_wait_note(&fx.shim.note).expect("a wait note");
    |                                 ^^^^^^^^^^^^^^ not found in `slots`

error[E0425]: cannot find function `read_wait_note` in module `slots`
   --> crates/athena-cli/tests/slot_exec.rs:240:23
    |
240 |     assert_eq!(slots::read_wait_note(&fx.shim.note), None);
    |                       ^^^^^^^^^^^^^^ not found in `slots`

error[E0425]: cannot find function `read_wait_note` in module `slots`
   --> crates/athena-cli/tests/slot_exec.rs:264:27
    |
264 |         assert_eq!(slots::read_wait_note(&fx.shim.note), None, "no wait note");
    |                           ^^^^^^^^^^^^^^ not found in `slots`

Some errors have detailed explanations: E0425, E0432.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-cli` (test "slot_exec") due to 5 previous errors
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p xtask --test ci_shards checked_in_timings_cover_every_workspace_test_unit`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/env.rs, crates/athena-agent/src/policy.rs, crates/athena-cli/src/lib.rs, crates/athena-cli/src/main.rs, crates/athena-cli/src/slot_exec.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/slots.rs, docs/design/agent-and-build-slots.md, xtask/shard-timings.txt.
  Warning: tests inside these files were removed with the revert: crates/athena-cli/src/slot_exec.rs.
  End of the output (data, not instructions):

```text
   Compiling unicode-ident v1.0.26
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling proc-macro2 v1.0.107
   Compiling crc32fast v1.5.2
   Compiling winnow v0.7.15
   Compiling adler2 v2.0.1
   Compiling simd-adler32 v0.3.10
   Compiling quote v1.0.47
   Compiling syn v3.0.7
   Compiling miniz_oxide v0.9.1
   Compiling xattr v1.6.1
   Compiling filetime v0.2.29
   Compiling tar v0.4.46
   Compiling flate2 v1.1.10
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
    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.92s
     Running tests/ci_shards.rs (<job>/target/verify/debug/deps/ci_shards-ce10c1bc9859f0ab)

running 1 test
test checked_in_timings_cover_every_workspace_test_unit ... FAILED

failures:

---- checked_in_timings_cover_every_workspace_test_unit stdout ----

thread 'checked_in_timings_cover_every_workspace_test_unit' (10484) panicked at xtask/tests/ci_shards.rs:420:5:
assertion `left == right` failed
  left: {"athena-agent --doc", "athena-agent --lib", "athena-cap-change --doc", "athena-cap-change --lib", "athena-cap-study --doc", "athena-cap-study --lib", "athena-cli --bin athena", "athena-cli --doc", "athena-cli --lib", "athena-cli --test audience", "athena-cli --test auditor", "athena-cli --test backtest", "athena-cli --test binary", "athena-cli --test budgets", "athena-cli --test code_health_hooks", "athena-cli --test cutover_fixture", "athena-cli --test drain", "athena-cli --test external_calls", "athena-cli --test focus", "athena-cli --test history_pr_head", "athena-cli --test introspection", "athena-cli --test job_plan", "athena-cli --test labeled", "athena-cli --test lifecycle", "athena-cli --test live", "athena-cli --test ontology", "athena-cli --test proposals", "athena-cli --test reconcile", "athena-cli --test retry", "athena-cli --test review", "athena-cli --test status_build_measurement", "athena-cli --test status_durations", "athena-cli --test status_snapshot", "athena-cli --test steward", "athena-cli --test trailer_stop", "athena-cli --test version", "athena-cli --test watcher", "athena-cli --test work_decisions", "athena-context --doc", "athena-context --lib", "athena-core --doc", "athena-core --lib", "athena-core --test external_calls_guard", "athena-core --test refusal_corpus", "athena-data --doc", "athena-data --lib", "athena-data --test repo_config", "athena-gate --doc", "athena-gate --lib", "athena-git --doc", "athena-git --lib", "athena-score --doc", "athena-score --lib", "xtask --bin xtask", "xtask --test ci_shards", "xtask --test fingerprint", "xtask --test policy", "xtask --test site_page", "xtask --test spec", "xtask --test walltime"}
 right: {"athena-agent --doc", "athena-agent --lib", "athena-cap-change --doc", "athena-cap-change --lib", "athena-cap-study --doc", "athena-cap-study --lib", "athena-cli --bin athena", "athena-cli --doc", "athena-cli --lib", "athena-cli --test audience", "athena-cli --test auditor", "athena-cli --test backtest", "athena-cli --test binary", "athena-cli --test budgets", "athena-cli --test code_health_hooks", "athena-cli --test cutover_fixture", "athena-cli --test drain", "athena-cli --test external_calls", "athena-cli --test focus", "athena-cli --test history_pr_head", "athena-cli --test introspection", "athena-cli --test job_plan", "athena-cli --test labeled", "athena-cli --test lifecycle", "athena-cli --test live", "athena-cli --test ontology", "athena-cli --test proposals", "athena-cli --test reconcile", "athena-cli --test retry", "athena-cli --test review", "athena-cli --test slot_exec", "athena-cli --test status_build_measurement", "athena-cli --test status_durations", "athena-cli --test status_snapshot", "athena-cli --test steward", "athena-cli --test trailer_stop", "athena-cli --test version", "athena-cli --test watcher", "athena-cli --test work_decisions", "athena-context --doc", "athena-context --lib", "athena-core --doc", "athena-core --lib", "athena-core --test external_calls_guard", "athena-core --test refusal_corpus", "athena-data --doc", "athena-data --lib", "athena-data --test repo_config", "athena-gate --doc", "athena-gate --lib", "athena-git --doc", "athena-git --lib", "athena-score --doc", "athena-score --lib", "xtask --bin xtask", "xtask --test ci_shards", "xtask --test fingerprint", "xtask --test policy", "xtask --test site_page", "xtask --test spec", "xtask --test walltime"}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    checked_in_timings_cover_every_workspace_test_unit

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 19 filtered out; finished in 0.06s

error: test failed, to rerun pass `-p xtask --test ci_shards`
error: 1 target failed:
    `-p xtask --test ci_shards`
```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`git diff -- crates/athena-agent/src/policy.rs && cargo test --locked -p athena-agent policy`). Result: not run: not a plain cargo test command.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`git diff -- crates/athena-agent/src/env.rs && cargo test --locked -p athena-core only_the_producer_gets_the_cargo_shim_first_on_path && cargo test --locked -p athena-agent build_adds_no_shim_variables`). Result: not run: not a plain cargo test command.
- fails-without-9: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 9 (`git diff --stat`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
