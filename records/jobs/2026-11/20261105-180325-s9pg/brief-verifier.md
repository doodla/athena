# Job 20261105-180325-s9pg: verify "Enforce max_check_slots as a counting build-slot queue for local checks"

## Request (verbatim)

> Enforce max_check_slots as a counting build-slot queue around local checks (step 3). Build step from docs/design/agent-and-build-slots.md section 8; follow that section for the files touched and how the effect is measured.
> 
> Acceptance:
> - local checks take a build slot from a counting queue of max_check_slots entries and free it when done, including when the job crashes
> - a test where two jobs need local checks with one slot: the second waits and then runs
> - a test where a slot holder's process dies: the slot frees without manual cleanup

## Reading (accepted by the user)

Add a lock-file counting semaphore (new slots.rs) of max_check_slots entries that SnapExec::exec takes around each local check and releases on drop or process death. Add build_wait_s/build_age_promote_s config, the resumable "environment: build slot wait" hold, and a queued_s field plus a "build wait" status line, as design section 8 step 3 says. Audience: the maintainer running the drain.

Assumptions:
- Code survey: read crates/athena-core/src/run/checks.rs (SnapExec::exec runs the disk gate once, then run_check per check; the entry point), crates/athena-core/src/state.rs (lock_file, try_lock_job use std File::try_lock; the lock dies with the process), crates/athena-core/src/disk.rs (LowDisk hold, write_hold/read_hold) and crates/athena-data/src/config.rs (PipelineConfig.max_check_slots, validated 1..=MAX_CHECK_SLOTS_CEILING). The slot is taken after refuse_if_stopped and the disk gate, before run_check, and held until the snapshot is restored.
- Code survey: the effect is measured directly by a queued_s field on each check in verification.json (CheckRun in crates/athena-data/src/records.rs, optional so old records still read; its round-trip/AnyRecord tests must still pass) and a 'build wait' line in the throughput section of crates/athena-core/src/status.rs. No new record file type is added.
- Only the local-check part of step 3 is in scope; the producer wrapper (step 4), landing rebuild (step 7) and slot-owned target directories (step 5) are later steps. Priority classes and ageing are implemented in slots.rs as the design says, with check/rerun as the only class used now.
- A limit of 1 reproduces today's one-build behaviour. Test hooks may use the existing check_seam and a tempdir state; a 'process dies' test uses a child process (or a separate File handle dropped) holding a slot, since OS file locks release on exit.
- athena.toml and defaults.toml need no change: new config fields have in-code defaults (build_wait_s 1800, build_age_promote_s 600).

## Acceptance criteria

1. [behavior] `cargo test -p athena-core` passes including a new slots test where two jobs need local checks with max_check_slots = 1: the second waits while the first holds the slot, then runs after release (asserted by order and a nonzero queued_s for the second).
2. [behavior] `cargo test -p athena-core` includes a test where a slot holder's process dies (a child process holding the slot lock is killed, or a holder is dropped on panic/error) and a waiter then acquires the slot with no manual cleanup or sweeper.
3. [behavior] A test with max_check_slots = 2 shows at most 2 holders at once and a third request waits.
4. [behavior] A test shows a check request waiting past build_wait_s returns a resumable hold with reason `environment: build slot wait`, uses no attempt, and is not recorded as a work failure.
5. [content] crates/athena-core/src/slots.rs exists and SnapExec::exec in crates/athena-core/src/run/checks.rs acquires a guard from it before run_check and drops it after the snapshot restore, on every return path (guard released by Drop).
6. [content] PipelineConfig in crates/athena-data/src/config.rs gains build_wait_s (default 1800) and build_age_promote_s (default 600) with validation and tests; unknown keys are still refused.
7. [behavior] Each check in verification.json carries an optional queued_s (old records still deserialize), and `cargo test` covers a 'build wait' line in the status throughput section that sums it.
8. [behavior] `cargo clippy --all-targets` and `cargo fmt --check` are clean, and the whole `cargo test` passes with the default max_check_slots = 1.
9. [content] No change to athena.toml, defaults.toml, workflows, or records/; CHANGELOG.md gets an entry for the change.

## Expected deliverables

- crates/athena-core/src/slots.rs (create)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/state.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/tests.rs (modify)
- docs/design/agent-and-build-slots.md (modify)

## Out of scope

- Producer cargo wrapper and build-slot subcommand (step 4)
- Landing rebuild slot use (step 7)
- Slot-owned warm target directories (step 5)
- Changes to athena.toml or defaults.toml
- Changing the sample metrics in records/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implementation and tests are Rust, using std file locks as state.rs does.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New slots.rs tests; cargo test passes and clippy is clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests exercise waiting, release on crash, and the queued_s output.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is measured by queued_s summed in the 'build wait' status line.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only step 3; steps 4 to 7 out of scope.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The limit is enforced by lock files, not prose.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A full-job test in run/tests.rs, two_jobs_with_one_build_slot_run_one_after_the_other. Setup: repo_with_config("") (default max_check_slots = 1), athena_with over a tempdir state and plenty_of_space, then two athena.ask calls to get two proposals. Inside std::thread::scope, two threads call the shared &Athena `go` (the drain does the same through RealSteps). Each thread installs its own thread-local check_seam fake. The fake appends to a shared Mutex<Vec<&str>> log and uses an `inside` AtomicUsize. The first fake call (chosen by an AtomicBool swap) logs 'first-start', then waits (bounded 10 s) until `SlotQueue::waiting()` on a clone of athena.state is at least 1, then logs 'first-end' and returns a passing CheckRun. The second fake logs 'second' and asserts inside == 0 on entry. A slots.rs unit test with two threads and one slot shows the same order for plain acquire calls. A second test in run/tests.rs, a_check_that_waited_checks_the_disk_again, uses a free-space fn pointer that reports low once a file in the state dir exists; the holder writes that file just before it drops the slot. The job then returns the LowDisk hold instead of running its check. Run: `cargo test -p athena-core two_jobs_with_one_build_slot && cargo test -p athena-core a_check_that_waited_checks_the_disk_again && cargo test -p athena-core slots::tests::the_second_request_waits_then_runs`. Observes: The log reads first-start, first-end, second. The waiting job's check in verification.json has queued_s > 0 (loaded via Records) and the first job's check has a small queued_s. Both go calls return Ok. The waited-gate test returns the LowDisk hold, with the verifier not run.
  Fails without the behavior: Without the slot in SnapExec::exec, the second fake runs while the first waits, so the `inside == 0` assertion fails, or the first fake times out at 10 s because no ticket ever appears, and queued_s is None. Without the re-gate after a wait, the disk test runs the check instead of holding.
  Test seam: Production entry points: Athena::go -> SnapExec::exec -> SlotQueue::acquire (real lock files in the tempdir state). Fakes: the existing thread-local check_seam::install (run/mod.rs, used in run/tests.rs near lines 1983, 2666 and 3062) supplies only the CheckRun, and the existing free-space fn pointer via athena_with. athena_with, repo_with_config and counting exist. SlotQueue, waiting() and Slot::waited() are added by plan steps 4 and 6. The coordination (ticket seen) reads the real queue directory; no logic under test is in a double.
- Criterion 2: A unit test in slots.rs, a_killed_holder_frees_its_slot, re-executes the test binary (std::env::current_exe with `--exact slots::tests::slot_holder_child --nocapture`, env var naming the slot dir). The child acquires the only slot, prints HELD and sleeps. The parent reads HELD, then asserts a non-blocking try_acquire on the same dir is refused with the child alive. It then calls Child::kill (SIGKILL) and wait, and a blocking acquire succeeds with no file removed and no cleanup call. A second test, a_panicking_holder_frees_its_slot, panics in a thread that holds a Slot and the waiter then acquires. A third shows a holder dropped on an Err return frees the slot. Run: `cargo test -p athena-core slots::tests::a_killed_holder_frees_its_slot && cargo test -p athena-core slots::tests::a_panicking_holder_frees_its_slot`. Observes: The slot is refused while the child lives and acquired after the kill within a poll or two, with no manual cleanup. The same holds after a panic.
  Fails without the behavior: If the slot were a created/removed file or pid file with no sweeper, the waiter would time out after the kill. If the slot did not exclude, the refusal assertion before the kill would fail.
  Test seam: Production entry points: SlotQueue::acquire and try_acquire on a tempdir. The seam is a real child process, running as the same user and environment as production, so nothing mocks the lock. The child's test body returns at once unless the env var is set. SlotQueue::new(dir, slots, wait, promote) and with_poll are added by plan step 4; StateDir::slots_dir by step 1.
- Criterion 3: A unit test in slots.rs, two_slots_admit_two_holders_and_the_third_waits, with slots = 2 and three threads calling acquire on one dir. A shared AtomicUsize counts live holders and records the peak. try_acquire is also called while two are held and then after one drops. Separate tests cover class order (a later Check waiter overtakes an earlier Producer waiter), FIFO within a class, and ageing (a Producer older than build_age_promote_s is promoted past a Check). Run: `cargo test -p athena-core slots::tests::two_slots_admit_two_holders_and_the_third_waits`. Observes: The peak of concurrent holders is 2, never 3. The third request returns only after a release, with queued above zero. try_acquire is refused with two held and succeeds after a drop.
  Fails without the behavior: A limit of 1 or no limit fails the peak == 2 or the 'third waits' assertion; no ordering fails the class tests.
  Test seam: Production entry points: SlotQueue::acquire/try_acquire with real threads and tempdir lock files. There is no double. Ageing tests build queues with small promote and poll values through the constructor and with_poll (plan step 4).
- Criterion 4: A full-job test in run/tests.rs, a_check_that_waits_past_build_wait_s_holds_the_job, with repo_with_config("\n[drain.pipeline]\nbuild_wait_s = 1\n"). A real Slot from SlotQueue::acquire on a clone of athena.state is held for the whole first go. Assertions: (a) go returns an error that downcasts to disk::SlotWait; (b) the ui has a line starting `held       environment: build slot wait`; (c) the job is JobState::Dispatched with no outcome, no score and no verification, and the counts show producer 1 and verifier 0; (d) read_slot_hold is Some and read_hold is None; (e) a second `go` on the held job returns an error whose text contains `build slot wait` and not `was interrupted` (finding 11); (f) athena.status() shows the job's `hold` equal to the slot-wait text; (g) is_environment_reason(&text) is true; (h) after the holder drops, athena.retry then go resumes with no producer rerun and lands. A drain test in crates/athena-cli/tests/drain.rs, a_build_slot_wait_holds_the_item_and_is_not_labelled_low_disk, uses Fixture::with_config of config_with_change_checks plus `[drain.pipeline] build_wait_s = 1`, with a held Slot on athena.state, then drain::drain(&real_steps, work_everything(), &Quiet). Run: `cargo test -p athena-core a_check_that_waits_past_build_wait_s_holds_the_job && cargo test -p athena-cli --test drain a_build_slot_wait_holds_the_item`. Observes: The core assertions (a)-(h) above. In the drain test: jobs[0].outcome == "environment: build slot wait" (not "environment: low disk"), landed == 0, the item is attempted once, display::drain text contains the slot-wait reason and no `athena retry` advice, the run does not stop as an environment stop, and the records show Dispatched with no outcome. After the holder drops, a second drain pass resumes the same job and lands it.
  Fails without the behavior: Without the SlotWait hold, the wait never times out, or the timeout surfaces as a plain error or a failed outcome, so (a), (c) and the drain outcome fail. Without read_any_hold, (e) gives 'was interrupted' and (f) is None. Without the drain reason switch, the outcome reads `environment: low disk`.
  Test seam: Production entry points: Athena::go -> SnapExec::exec -> SlotQueue::acquire -> Athena::hold_for_slot_wait, and the drain's RealSteps run. Fixtures that exist today: repo_with_config, athena_with, counting and plenty_of_space in run/tests.rs; Fixture::with_config, config_with_change_checks, real_steps, Lines, work_everything and rated_scenario in crates/athena-cli/tests/drain.rs. Core tests also use the existing check_seam; the drain test runs the real `sh -c true` checks (as gate_run does). The holder is the real SlotQueue (a pub module), and build_wait_s reaches the code through the committed config via JobConfig, as check_timeout_s does. No test clock: 1 s real, the smallest valid value. The hold, read_any_hold and SlotQueue are added by plan steps 2, 4, 7 and 9.
- Criterion 5: Content check plus a full-job test, the_build_slot_is_free_after_a_panic_in_the_check_and_after_a_normal_run. The test installs a check_seam fake that panics (it is called after the slot is taken in exec), runs go under std::panic::catch_unwind(AssertUnwindSafe(..)), then asserts SlotQueue::try_acquire on a clone of athena.state returns Some. A second part runs a normal job to completion and asserts try_acquire succeeds again. Reading exec shows `let slot = self.slots.acquire(...)` before run_check and live past restore_to. Run: `ls crates/athena-core/src/slots.rs && cargo test -p athena-core the_build_slot_is_free_after_a_panic_in_the_check`. Observes: slots.rs exists. After the panic unwinds out of exec, the single slot is free at once, and it is free after a normal run too.
  Fails without the behavior: If the guard were leaked (mem::forget) or released only on the success path, try_acquire after the unwind would be refused. flock conflicts between separate open files in one process, so the leak is visible. The panic happens after acquire, so the test cannot pass by an early return before the slot.
  Test seam: Production entry point: Athena::go -> SnapExec::exec. Fakes: the existing check_seam fake (it is the thing that panics) and athena_with with plenty_of_space, both existing. try_acquire is added by plan step 4. A wrong-order variant (a disk-gate return) is deliberately not used, since that returns before the slot is taken.
- Criterion 6: Config tests in config.rs: the defaults give 1800 and 600, explicit values parse, a zero for either key is refused naming the key, and an unknown key under [drain.pipeline] is still refused. bad_drain_pipeline_settings_are_refused_naming_the_key gains `build_wait_s = 0` and `build_age_promote_s = 0` next to the unchanged `surprise = 1`. drain_pipeline_defaults_keep_todays_behavior asserts the new defaults. reading_inputs_ignore_a_non_default_drain_pipeline keeps passing. Run: `cargo test -p athena-data config::`. Observes: The defaults tuple includes (1800, 600). A zero value gives ConfigError::DrainPipeline("build_wait_s") or ("build_age_promote_s"). `surprise` is still refused, and the digest and reading-inputs equality hold.
  Fails without the behavior: Without the fields the defaults assertion does not compile. Without validation the zero cases parse and the expect_err fails.
  Test seam: Production entry point: Config::parse with the REPO_CONFIG fixture (exists in config.rs tests). No double. PipelineConfig stays Copy with u64 fields.
- Criterion 7: Two tests. In records.rs, a CheckRun JSON without queued_s deserializes with None and re-serializes without the key, and one with queued_s 12.5 round-trips; the AnyRecord verification tests still pass. In status.rs, the_throughput_section_sums_build_wait builds landed jobs whose verification checks carry queued_s 12.0 and 30.0, plus a reused check with None, and calls Throughput::compute(...).text(). A second status test checks that no queued_s anywhere gives no `build wait` line. Run: `cargo test -p athena-data records:: && cargo test -p athena-core status::tests::the_throughput_section_sums_build_wait`. Observes: Old JSON reads. The text has a line starting `build wait` with total 42 s, the count of waited checks and the basis 'athena monotonic clock'. With no data there is no such line, so existing throughput text assertions are unchanged.
  Fails without the behavior: Without the field, the round-trip with queued_s fails to compile. Without the summation, the `build wait` line is absent and the contains assertion fails.
  Test seam: Production entry points: AnyRecord::parse, serde on CheckRun, and Throughput::compute/text over Records built by the existing loaded(...), timed_job and test_run helpers in status.rs tests. No double. Plan step 5 adds the field, step 8 the line.
- Criterion 8: Run the three Athena checks. Every existing test runs through Athena::go at the default max_check_slots = 1 with an uncontended queue, and every CheckRun literal across crates is updated (the compiler reports any missed). Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. The LowDisk hold tests, the drain tests, status and records tests behave as before.
  Fails without the behavior: A missing queued_s: None literal fails compilation. A slot that blocked at limit 1 even when uncontended would hang the existing job tests.
  Test seam: These are the production commands Athena runs on the result. State dirs are tempdirs, so slot files are isolated per test.
- Criterion 9: Inspect the diff: nothing under athena.toml, defaults.toml, .github, workflows or records/, and a CHANGELOG.md bullet under Unreleased > Added. Run: `git diff --stat -- athena.toml defaults.toml .github records`. Observes: The first command prints nothing. `git diff -- CHANGELOG.md` shows a new bullet under `## [Unreleased]` / `### Added` naming the build-slot queue, the two config keys, the `build slot wait` hold, queued_s and the 'build wait' status line.
  Fails without the behavior: A forgotten changelog shows an empty diff for CHANGELOG.md. An accidental governance edit shows in the stat.
  Test seam: Not a behavior criterion, so no seam. It is a content check on the repository diff.

Planned files:

- crates/athena-core/src/slots.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/status_durations.rs (modify)
- CHANGELOG.md (modify)
- docs/design/agent-and-build-slots.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b562d63ef2cd → candidate f8da8d6710f7. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  10 +
 crates/athena-cli/src/drain.rs              |  45 +-
 crates/athena-cli/tests/drain.rs            |  69 +++
 crates/athena-cli/tests/status_durations.rs |   1 +
 crates/athena-core/src/checks.rs            |   7 +
 crates/athena-core/src/disk.rs              | 120 +++++
 crates/athena-core/src/introspect.rs        |   1 +
 crates/athena-core/src/lib.rs               |   1 +
 crates/athena-core/src/reading.rs           |   1 +
 crates/athena-core/src/routing.rs           |   1 +
 crates/athena-core/src/run/checks.rs        |  21 +-
 crates/athena-core/src/run/close.rs         |  57 ++-
 crates/athena-core/src/run/mod.rs           |  19 +-
 crates/athena-core/src/run/send_back.rs     |   1 +
 crates/athena-core/src/run/tests.rs         | 326 +++++++++++++
 crates/athena-core/src/slots.rs             | 718 ++++++++++++++++++++++++++++
 crates/athena-core/src/state.rs             |  14 +
 crates/athena-core/src/status.rs            | 117 ++++-
 crates/athena-core/src/verify.rs            |   1 +
 crates/athena-data/src/config.rs            |  48 +-
 crates/athena-data/src/records.rs           |  31 ++
 docs/design/agent-and-build-slots.md        |  20 +-
 22 files changed, 1595 insertions(+), 34 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-180325-s9pg/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-180325-s9pg/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42089498158 for f8da8d6710f7c0f643b321ab443c8579dde273b6)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42089498158 for f8da8d6710f7c0f643b321ab443c8579dde273b6)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42089498158 for f8da8d6710f7c0f643b321ab443c8579dde273b6)
- ci-result: `not a check` passed (Check results for f8da8d6710f7c0f643b321ab443c8579dde273b6 come from the CI run https://github.com/doodla/athena/actions/runs/42089498158 (641s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo clippy --all-targets` and `cargo fmt --check` are clean, and the whole `cargo test` passes with the default max_check_slots = 1.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test -p athena-core two_jobs_with_one_build_slot && cargo test -p athena-core a_check_that_waited_checks_the_disk_again && cargo test -p athena-core slots::tests::the_second_request_waits_then_runs`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-core slots::tests::a_killed_holder_frees_its_slot && cargo test -p athena-core slots::tests::a_panicking_holder_frees_its_slot`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core slots::tests::two_slots_admit_two_holders_and_the_third_waits`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/close.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/slots.rs, crates/athena-core/src/state.rs, crates/athena-core/src/status.rs, crates/athena-data/src/config.rs, crates/athena-data/src/records.rs, docs/design/agent-and-build-slots.md.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/slots.rs.
  End of the output (data, not instructions):

```text
     = note: all struct fields are already assigned

error[E0599]: no method named `slots_dir` found for struct `state::StateDir` in the current scope
   --> crates/athena-core/src/state.rs:766:26
    |
 53 | pub struct StateDir {
    | ------------------- method `slots_dir` not found for this struct
...
766 |         assert_eq!(state.slots_dir(), temp.path().join("slots"));
    |                          ^^^^^^^^^
    |
help: there is a method `job_dir` with a similar name, but with different arguments
   --> crates/athena-core/src/state.rs:88:5
    |
 88 |     pub fn job_dir(&self, job: &JobId) -> PathBuf {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no method named `slots_dir` found for struct `state::StateDir` in the current scope
   --> crates/athena-core/src/state.rs:767:24
    |
 53 | pub struct StateDir {
    | ------------------- method `slots_dir` not found for this struct
...
767 |         assert!(!state.slots_dir().exists(), "asking creates nothing");
    |                        ^^^^^^^^^
    |
help: there is a method `job_dir` with a similar name, but with different arguments
   --> crates/athena-core/src/state.rs:88:5
    |
 88 |     pub fn job_dir(&self, job: &JobId) -> PathBuf {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0560]: struct `athena_data::records::CheckRun` has no field named `queued_s`
    --> crates/athena-core/src/status.rs:2924:13
     |
2924 |             queued_s,
     |             ^^^^^^^^ `athena_data::records::CheckRun` does not have this field
     |
     = note: all struct fields are already assigned

error[E0609]: no field `build_wait` on type `status::Throughput`
    --> crates/athena-core/src/status.rs:3358:31
     |
3358 |         assert_eq!(throughput.build_wait, None);
     |                               ^^^^^^^^^^ unknown field
     |
     = note: available fields are: `landed_total`, `landed_per_day`, `window_jobs`, `median_minutes`, `median_stages` ... and 4 others

error[E0560]: struct `athena_data::records::CheckRun` has no field named `queued_s`
    --> crates/athena-core/src/verify.rs:1504:13
     |
1504 |             queued_s: None,
     |             ^^^^^^^^ `athena_data::records::CheckRun` does not have this field
     |
     = note: all struct fields are already assigned

Some errors have detailed explanations: E0422, E0425, E0433, E0560, E0599, E0609.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 37 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core a_check_that_waits_past_build_wait_s_holds_the_job && cargo test -p athena-cli --test drain a_build_slot_wait_holds_the_item`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`ls crates/athena-core/src/slots.rs && cargo test -p athena-core the_build_slot_is_free_after_a_panic_in_the_check`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 6 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-data config::`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/drain.rs, crates/athena-core/src/checks.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/close.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/slots.rs, crates/athena-core/src/state.rs, crates/athena-core/src/status.rs, crates/athena-data/src/config.rs, crates/athena-data/src/records.rs, docs/design/agent-and-build-slots.md.
  Warning: tests inside these files were removed with the revert: crates/athena-core/src/slots.rs.
  End of the output (data, not instructions):

```text
error[E0609]: no field `build_age_promote_s` on type `PipelineConfig`
    --> crates/athena-data/src/config.rs:1134:19
     |
1134 |                 p.build_age_promote_s
     |                   ^^^^^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `max_in_flight`, `max_check_slots`, `read_ahead`, `reserve_disk_gb`, `max_conflict_returns`

error[E0609]: no field `build_wait_s` on type `PipelineConfig`
    --> crates/athena-data/src/config.rs:1187:32
     |
1187 |                 set.pipeline().build_wait_s,
     |                                ^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `max_in_flight`, `max_check_slots`, `read_ahead`, `reserve_disk_gb`, `max_conflict_returns`

error[E0609]: no field `build_age_promote_s` on type `PipelineConfig`
    --> crates/athena-data/src/config.rs:1188:32
     |
1188 |                 set.pipeline().build_age_promote_s
     |                                ^^^^^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `max_in_flight`, `max_check_slots`, `read_ahead`, `reserve_disk_gb`, `max_conflict_returns`

error[E0560]: struct `records::CheckRun` has no field named `queued_s`
    --> crates/athena-data/src/records.rs:4510:13
     |
4510 |             queued_s: None,
     |             ^^^^^^^^ `records::CheckRun` does not have this field
     |
     = note: all struct fields are already assigned

error[E0609]: no field `queued_s` on type `records::CheckRun`
    --> crates/athena-data/src/records.rs:4528:24
     |
4528 |         assert_eq!(old.queued_s, None);
     |                        ^^^^^^^^ unknown field
     |
     = note: available fields are: `name`, `command`, `exit_code`, `passed`, `tests_ran` ... and 6 others

error[E0609]: no field `queued_s` on type `records::CheckRun`
    --> crates/athena-data/src/records.rs:4535:15
     |
4535 |         check.queued_s = Some(12.5);
     |               ^^^^^^^^ unknown field
     |
     = note: available fields are: `name`, `command`, `exit_code`, `passed`, `tests_ran` ... and 6 others

error[E0609]: no field `queued_s` on type `records::CheckRun`
    --> crates/athena-data/src/records.rs:4539:25
     |
4539 |         assert_eq!(back.queued_s, Some(12.5));
     |                         ^^^^^^^^ unknown field
     |
     = note: available fields are: `name`, `command`, `exit_code`, `passed`, `tests_ran` ... and 6 others

Some errors have detailed explanations: E0560, E0609.
For more information about an error, try `rustc --explain E0560`.
error: could not compile `athena-data` (lib test) due to 8 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test -p athena-data records:: && cargo test -p athena-core status::tests::the_throughput_section_sums_build_wait`). Result: not run: not a plain cargo test command.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-9: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 9 (`git diff --stat -- athena.toml defaults.toml .github records`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
