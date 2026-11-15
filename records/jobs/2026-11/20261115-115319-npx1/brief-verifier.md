# Job 20261115-115319-npx1: verify "Queue jobs for a single build-folder slot so two can run at once"

## Request (verbatim)

> From Sri (the owner), word for word: "send both" (this is part 2 of 2).
> 
> Background (not Sri's own words; added so the request can be read on its own). Part 1 sets max_concurrency = 1 as a stopgap because two jobs each keep a 15-24 GB build folder on a disk that fits one (see the part 1 request).
> Part 2, the lasting fix: let two jobs run at once again, but queue for disk, not only for running builds. Only one job at a time may hold a build folder; a second job may read and plan, and waits for the folder to be freed before its producer starts building, instead of being held or discarded for low disk. When this lands and is shown to keep free space above the low-disk gate with two jobs, set max_concurrency back to 2.

## Reading (accepted by the user)

Add a disk-slot gate so only one job at a time holds a build folder. A second job may read and plan, and waits for the slot before its producer builds, instead of being held or discarded for low disk. With the gate in place, max_concurrency goes back to 2. The audience is the owner running the drain on a disk that fits one build folder.

Assumptions:
- Code survey: read crates/athena-core/src/admission.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/state.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/commands.rs and crates/athena-data/src/config.rs. Grep shows max_concurrency and build-slot handling in these files. The entry point is the drain in crates/athena-cli/src/drain.rs. Admission and the disk gate run before a job is started. The producer build step is where the new wait goes. Direct measurement is the count of jobs holding a build folder at the same moment, not a proxy. The producer should confirm the exact functions before editing. Any new record or data file under crates/ must be covered by check_record_file, AnyRecord or the round-trip test; prefer no new record type.
- Term ladder: not used. Build-folder slot means a single exclusive hold on the right to create or keep a per-job build directory (target folder). It is taken before the producer builds and freed when the job lands, is rejected or is discarded.
- Reading of the request: planning, reading and verification need no slot. Only the producer build phase waits. A job waiting for the slot is neither discarded nor held for low disk. If free space is below the low-disk gate for a reason other than another job's folder, existing behavior stays.
- Field meaning: max_concurrency in athena.toml (line 205, live value 2 in this copy) bounds concurrent jobs. Its meaning is unchanged. Part 1 may already have set it to 1. This job ends with it at 2. Consumers: crates/athena-data/src/config.rs, crates/athena-cli/src/drain.rs, crates/athena-core/src/admission.rs.
- The slot count is one, fixed by design, and the reason is that the disk fits one 15-24 GB build folder. Whether a config key for the slot count is added is left to the planner, with default 1.
- Environment: none. No CI workflow, push or records read is needed. Verification uses cargo test, clippy and fmt on the working copy.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs (or crates/athena-core/src/run/tests.rs) drains two jobs with max_concurrency = 2. It asserts that both jobs reach planning together, that the second producer starts only after the first job frees its build folder, and that the peak count of build-folder holders is 1. Both jobs land, and neither is discarded or held for low disk. Run with cargo test; the test fails if the slot gate is removed.
2. [behavior] A test shows a job waiting for the slot does not count as a low-disk hold or discard. With a simulated free-space value below the gate while another job holds the folder, the waiting job stays queued and is later started. Run with cargo test.
3. [behavior] A test shows the slot is freed on every exit path (landed, rejected, failed or discarded job). A second job that was waiting then proceeds. Run with cargo test.
4. [content] The slot gate is enforced in code, in crates/athena-core/src/admission.rs, crates/athena-core/src/disk.rs or crates/athena-cli/src/drain.rs, and is not prose only. A Grep shows it called from the producer-start path.
5. [content] athena.toml has max_concurrency = 2 at line 205 in the final tree (git diff shows it unchanged or restored from 1 to 2). The CHANGELOG.md entry says the stopgap is lifted.
6. [content] A new appended entry in docs/decisions.md, with a Provenance bullet, records the design: one build-folder slot, the alternatives considered (a global max_concurrency of 1, a free-disk check only), and why. docs/design/parallel-jobs.md or docs/design/agent-and-build-slots.md is updated to describe the disk slot. No existing decisions entry is edited.
7. [behavior] cargo test, cargo clippy and cargo fmt --check pass on the final tree, with no new clippy warnings.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-data/src/config.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/design/agent-and-build-slots.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the low-disk gate threshold itself.
- Running a live drain on the owner's machine to prove free space; tests simulate it.
- Adding more than one build-folder slot or a cleanup of old build folders.
- Edits to records/ or existing docs/decisions.md entries.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for the slot gate; cargo test and clippy criteria.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Gate enforced in code, not prose.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the drain with two jobs and check peak slot holders.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: A new decision entry records alternatives.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Gate on disk use directly rather than limiting job count.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: One core test over two real `Athena::go` calls on two scoped threads, as the drain's workers call it (drain.rs:4705-4721). The repository is `repo_with_planning(1)`, so both jobs plan. The check is a command that writes a file into `$CARGO_TARGET_DIR`, so the real baseline leaves a real `target/verify`. The planner script of each job marks itself inside planning and waits (10 s bound) until both are inside. The producer wrapper logs start and end. The first producer waits (10 s bound) for a waiter on the folder queue, then sleeps 300 ms, so the second job's wait is at least 300 ms. The verifier wrapper logs and writes `target/verify/blob`. A sampler thread polls `jobs/*/target/*` every 2 ms and counts jobs that have a real (non-link) directory there. The test times each `go` with its own `Instant`. Run: `cargo test -p athena-core two_jobs_plan_together_and_build_one_after_the_other`. Observes: Both planners were inside planning at once. The log reads first-start, first-end, first-verifier, second-start. The first producer saw a waiter. The peak number of jobs holding a real build folder is 1. Both jobs end `Finished` with state Landed; neither returned `LowDisk` or `SlotWait`, and neither has a hold marker. The second job's stored `totals.wall_time_execution` value plus 0.3 s is at most its own measured `go` time (the folder wait is left out), and the first job's value is positive.
  Fails without the behavior: Remove the `take_build_folder` call in `dispatch`: the first producer waits its bound for a waiter that never appears, `second-start` comes before `first-end`, and the peak holders is 2, so the log, waiter and peak assertions fail. Remove `ctx.clock.exclude(..)`: the second job's `wall_time_execution` is about its whole `go` time less a few milliseconds, so the 0.3 s assertion fails.
  Test seam: Production entry point `Athena::go` (run/mod.rs `go_job` -> `dispatch` -> `produce_and_verify`; the job lock is taken at mod.rs:456). Fixture: `athena_with` (run/tests.rs:191) with `repo_with_planning(1)` (293), a new check constant that writes into `$CARGO_TARGET_DIR`, `crate::test_support::plenty_of_space`, and a script that wraps the planning script with `counting(counts, Some(root))` (3114) and `each_job_its_own_edit` (6579). The wrappers only block, log and write files; they hold none of the gate or the clock. The queue probe is `build_folder_queue(&athena.state)` and `RunClock` is the clock, both added by this plan (steps 3 and 4). Every other piece exists today.
- Criterion 2: Two tests with a simulated disk. The free-space function is `capacity` minus the file lengths summed by its own walker over `<root>/jobs/*/target` (an independent walker, not `dir_bytes`), minus a ballast file in the second test. Test one: capacity 12 GB, minimum 10 GB. Job A's producer writes a sparse 8 GB file under `target/produce` and waits (10 s bound) for a waiter on the folder queue; the main thread starts job B's `go` once that file exists. Test two (`a_job_short_of_disk_for_its_own_reasons_is_still_held_at_dispatch`): capacity 12 GB, A's folder 2 GB, and A's producer also writes a 5 GB ballast file in the state root and removes it only after B's `go` has returned. Run: `cargo test -p athena-core a_job_waiting_for_the_build_folder_is_not_held_for_low_disk`. Observes: Test one: free space is 4 GB (below 10 GB) while A builds. B prints the `disk` credit line, which names A as the holder, and no `held` line. B stays queued at the folder, then starts after A's close and lands. B's `read_hold` is None. A also lands with no hold marker, because B owns no folder while A builds. Test two: B's `go` returns `LowDisk` with free 5 GB, the `held       environment: low disk` line is printed, and B's hold marker exists. B's producer never ran. A lands once the ballast is gone. A third unit test in disk.rs (`the_credit_counts_only_the_slot_holder`) shows a locked job that is not named in the holder file adds 0 bytes, and so does a holder file naming a job whose lock is free.
  Fails without the behavior: Without `dispatch_disk_gate`, B's `go` returns `LowDisk` at the dispatch gate (mod.rs:1088) and the test's `expect` fails. Without the slot, B's producer starts inside A's window and A's producer never sees a waiter. If the credit counted every locked job, or counted unconditionally, the second test and the holder test fail.
  Test seam: Entry point `Athena::go` on two threads. Fixture: `athena_with(.., free_space_fn, script)` (191). The new `fn(&Path) -> io::Result<u64>` free-space functions are injected through `StateDir::with_free_space` (state.rs:233), modelled on `low_once_marked` (6758). The producer wrapper around `counting` writes the sparse files. The credit lives in `dispatch_disk_gate` and the holder file in `BuildFolder`, both added by plan step 3; the doubles hold none of it.
- Criterion 3: For each ending, run job one through `go`, assert the folder queue is free (`take_slot_soon`, 6433) with `waiting() == 0`, that `jobs/<id>/target` is gone and that the holder file is gone, then run a second job to Landed. Endings: landed (`PASSES`); rejected (`FAILS_ON_EDIT`, 104); failed with no change (a producer that edits nothing, the empty-diff failure in `no_change_reason`); held for low disk before the producer (`low_once_marked`, as the test at 6234); and the kill switch set through `registry().stop_file()` while the job waits for a folder a probe holds, which returns an error with no ticket left. A separate test, `a_resumed_job_takes_the_build_folder_before_its_checks`, repeats the setup of the test that ends at 6406: it holds a job for low disk before its checks, holds the folder with a probe, runs the retry's `go` on a thread and asserts no check ran while the probe holds. After the probe drops the job resumes and lands. Run: `cargo test -p athena-core the_build_folder_is_freed_on_every_way_a_job_ends`. Observes: After each ending the slot is free, the queue is empty, the job's build folder and the holder file are gone, and the next job proceeds. The resumed job's checks start only after the folder is free. The code has no job state named discard (no match in lib.rs, state.rs, disk.rs, run/mod.rs, run/close.rs, store.rs or manage.rs); rejected, failed, held and killed are the not-landed endings.
  Fails without the behavior: If the guard were leaked (forgotten, or dropped only on the success path), the next `try_acquire` stays `None` and the following job's `go` hangs past the test's 10 s bound. If the `Drop` did not remove `target`, the absence assertion fails for the held and killed endings.
  Test seam: Entry points `Athena::go` and `Athena::retry`. Fixture: `athena_with`, `PASSES`, `FAILS_ON_EDIT`, `low_once_marked`, `counting`, `check_seam` (the thread-local fake check, used at 6488), `registry().stop_file()` (used at run/tests.rs:686). A probe holder comes from `build_folder_queue(&athena.state).try_acquire()`, as `slot_queue` is used at 6409 and 6787. All exist except `build_folder_queue`, added in step 3.
- Criterion 4: Read the diff. `Athena::take_build_folder` and `dispatch_disk_gate` are defined in disk.rs. `dispatch` in run/mod.rs calls `take_build_folder_or_hold` before `resume_candidate` and before `produce_and_verify`, calls `drop(folder)` after `finish_dispatched`, and uses `dispatch_disk_gate` in place of `disk_gate` at the dispatch gate. The criterion-1 test fails if that call is removed, which shows the gate is code and not prose. Run: `git diff -- crates/athena-core/src/disk.rs crates/athena-core/src/run/mod.rs crates/athena-core/src/state.rs`. Observes: The diff shows the call ahead of `baseline_stage` and the producer. run/mod.rs is outside the listed deliverables, and it has to be: planning runs inside `dispatch` (`plan_round`, mod.rs:1366) and the slot must come after it, while drain.rs reaches the run only through the blocking `go` (drain.rs:4705-4721) and admission.rs runs before dispatch. The edit there is the call sites and the `RunClock` type, about 45 lines.
- Criterion 5: The new test parses `include_str!("../../../athena.toml")` with `Config::parse` (the pattern at config.rs:1819-1830) and asserts `parallel.max_concurrency == 2`. The verifier also runs `git diff -- athena.toml CHANGELOG.md`. Run: `cargo test -p athena-cli --test drain the_live_config_allows_two_jobs_at_once`. Observes: The shipped config parses with 2. In the diff only the three comment lines above `[parallel]` change (or nothing, if the base already reads 2), so line 205 still reads `max_concurrency = 2`. The CHANGELOG `### Changed` bullet under `[Unreleased]` says the stopgap is lifted.
  Fails without the behavior: A live `max_concurrency = 1` (the stopgap left in place) fails the equality assertion.
  Test seam: Reads the shipped file through `Config::parse` (athena_data::config, exists), with no double.
- Criterion 6: Append `## D188 · ...` after D187 (decisions.md:5474, the last entry) with Case, Decision, Alternatives, Tradeoff, Reversal cost and Provenance, in the fields D187 uses (5476-5516). Alternatives name a global `max_concurrency` of 1, a free-disk check only, the release before the checks, a second take before the verifier, reuse of the check slots, the T estimate, a queue probe, a credit for every locked job, and keeping the wait in `wall_time_execution_s`. The Tradeoff names the planner spend of a job that plans before it is held, the credit's stale-holder case, and the builds that stay outside the slot: the drain's self-rebuild after a landing (`rebuild_if_due`, drain.rs:2650, which runs `setup.sh` through `run_setup`, drain.rs:3654) and the check slots' warm target directories (`slot_target`, state.rs:351, up to `slot_max_gb` = 8 GB each, config.rs:168). It writes no `Amends` or `Supersedes` phrase, so `link_problems` (D187) needs no link, and `docs/decision-links.toml` lists no governed path this change edits. Add `## 11. The build-folder slot` to agent-and-build-slots.md before `## Questions for the maintainer` (line 529): what the slot guards, where it is taken and freed, the dispatch rule and holder file, the fixed count of 1, how it differs from the check slots, how a waiting job appears, the exclusion of the wait from the wall-time figure, and the same two builds outside the slot. Also run `cargo test -p athena-core governed` and `cargo test -p athena-cli --test backtest`. Run: `git diff -- docs/decisions.md docs/design/agent-and-build-slots.md`. Observes: The decisions.md diff shows only added lines, with a Provenance bullet. The design doc has the new section. The decision-log checks pass.
- Criterion 7: Run the three Athena checks as separate commands (`cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`). Also run `cargo test -p athena-core --test refusal_corpus`: the corpus names `max_concurrency` only in question text (refusal_corpus.rs:3470 and 3482) and no refusal or matching rule changes, so it is a regression guard. Run: `cargo test --locked`. Observes: All pass. The changed test at run/tests.rs:6459 passes in its new form, and no helper is left unused (`recorded_queued_s` is still used at 6916 and 7137).
  Fails without the behavior: Any failing test, lint or format fault in the added code.
  Test seam: The existing check commands, no double.

Planned files:

- crates/athena-core/src/state.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/scoring.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-data/src/config.rs (modify)
- athena.toml (modify)
- docs/design/agent-and-build-slots.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 556625fe8918 → candidate 6fe01ec8dfd8. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   1 +
 athena.toml                          |   4 +-
 crates/athena-cli/src/drain.rs       |   8 +
 crates/athena-cli/tests/drain.rs     |   9 +
 crates/athena-cli/tests/lifecycle.rs |  11 +-
 crates/athena-core/src/disk.rs       | 237 ++++++++++
 crates/athena-core/src/run/mod.rs    |  83 +++-
 crates/athena-core/src/run/tests.rs  | 853 ++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/scoring.rs    |   2 +-
 crates/athena-core/src/state.rs      |  26 ++
 crates/athena-data/src/config.rs     |   5 +-
 docs/decisions.md                    |  62 +++
 docs/design/agent-and-build-slots.md |  63 +++
 13 files changed, 1337 insertions(+), 27 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261115-115319-npx1/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261115-115319-npx1/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43731642566 for 6fe01ec8dfd876f9b98b3aa7e45b0dfb5e3f74c0)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43731642566 for 6fe01ec8dfd876f9b98b3aa7e45b0dfb5e3f74c0)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43731642566 for 6fe01ec8dfd876f9b98b3aa7e45b0dfb5e3f74c0)
- ci-result: `not a check` passed (Check results for 6fe01ec8dfd876f9b98b3aa7e45b0dfb5e3f74c0 come from the CI run https://github.com/doodla/athena/actions/runs/43731642566 (892s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test, cargo clippy and cargo fmt --check pass on the final tree, with no new clippy warnings.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core two_jobs_plan_together_and_build_one_after_the_other`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, athena.toml, crates/athena-cli/src/drain.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/scoring.rs, crates/athena-core/src/state.rs, crates/athena-data/src/config.rs, docs/decisions.md, docs/design/agent-and-build-slots.md.
  End of the output (data, not instructions):

```text
      |                         ^^^^^^^^ use of undeclared type `RunClock`

error[E0599]: no method named `build_folder_dir` found for struct `state::StateDir` in the current scope
   --> crates/athena-core/src/state.rs:925:26
    |
 53 | pub struct StateDir {
    | ------------------- method `build_folder_dir` not found for this struct
...
925 |         assert_eq!(state.build_folder_dir(), temp.path().join("build-folder"));
    |                          ^^^^^^^^^^^^^^^^
    |
help: there is a method `build_slots_dir` with a similar name
    |
925 -         assert_eq!(state.build_folder_dir(), temp.path().join("build-folder"));
925 +         assert_eq!(state.build_slots_dir(), temp.path().join("build-folder"));
    |

error[E0599]: no method named `build_folder_holder_file` found for struct `state::StateDir` in the current scope
   --> crates/athena-core/src/state.rs:927:19
    |
 53 | pub struct StateDir {
    | ------------------- method `build_folder_holder_file` not found for this struct
...
927 |             state.build_folder_holder_file(),
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `state::StateDir`

error[E0599]: no method named `build_folder_dir` found for struct `state::StateDir` in the current scope
   --> crates/athena-core/src/state.rs:930:26
    |
 53 | pub struct StateDir {
    | ------------------- method `build_folder_dir` not found for this struct
...
930 |         assert_ne!(state.build_folder_dir(), state.slots_dir());
    |                          ^^^^^^^^^^^^^^^^
    |
help: there is a method `build_slots_dir` with a similar name
    |
930 -         assert_ne!(state.build_folder_dir(), state.slots_dir());
930 +         assert_ne!(state.build_slots_dir(), state.slots_dir());
    |

error[E0599]: no method named `build_folder_dir` found for struct `state::StateDir` in the current scope
   --> crates/athena-core/src/state.rs:931:24
    |
 53 | pub struct StateDir {
    | ------------------- method `build_folder_dir` not found for this struct
...
931 |         assert!(!state.build_folder_dir().exists(), "asking creates nothing");
    |                        ^^^^^^^^^^^^^^^^
    |
help: there is a method `build_slots_dir` with a similar name
    |
931 -         assert!(!state.build_folder_dir().exists(), "asking creates nothing");
931 +         assert!(!state.build_slots_dir().exists(), "asking creates nothing");
    |

Some errors have detailed explanations: E0422, E0425, E0432, E0433, E0599.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 33 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core a_job_waiting_for_the_build_folder_is_not_held_for_low_disk`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, athena.toml, crates/athena-cli/src/drain.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/scoring.rs, crates/athena-core/src/state.rs, crates/athena-data/src/config.rs, docs/decisions.md, docs/design/agent-and-build-slots.md.
  End of the output (data, not instructions):

```text
11364 |             let clock = RunClock::start();
      |                         ^^^^^^^^ use of undeclared type `RunClock`

error[E0599]: no method named `build_folder_dir` found for struct `state::StateDir` in the current scope
   --> crates/athena-core/src/state.rs:925:26
    |
 53 | pub struct StateDir {
    | ------------------- method `build_folder_dir` not found for this struct
...
925 |         assert_eq!(state.build_folder_dir(), temp.path().join("build-folder"));
    |                          ^^^^^^^^^^^^^^^^
    |
help: there is a method `build_slots_dir` with a similar name
    |
925 -         assert_eq!(state.build_folder_dir(), temp.path().join("build-folder"));
925 +         assert_eq!(state.build_slots_dir(), temp.path().join("build-folder"));
    |

error[E0599]: no method named `build_folder_holder_file` found for struct `state::StateDir` in the current scope
   --> crates/athena-core/src/state.rs:927:19
    |
 53 | pub struct StateDir {
    | ------------------- method `build_folder_holder_file` not found for this struct
...
927 |             state.build_folder_holder_file(),
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `state::StateDir`

error[E0599]: no method named `build_folder_dir` found for struct `state::StateDir` in the current scope
   --> crates/athena-core/src/state.rs:930:26
    |
 53 | pub struct StateDir {
    | ------------------- method `build_folder_dir` not found for this struct
...
930 |         assert_ne!(state.build_folder_dir(), state.slots_dir());
    |                          ^^^^^^^^^^^^^^^^
    |
help: there is a method `build_slots_dir` with a similar name
    |
930 -         assert_ne!(state.build_folder_dir(), state.slots_dir());
930 +         assert_ne!(state.build_slots_dir(), state.slots_dir());
    |

error[E0599]: no method named `build_folder_dir` found for struct `state::StateDir` in the current scope
   --> crates/athena-core/src/state.rs:931:24
    |
 53 | pub struct StateDir {
    | ------------------- method `build_folder_dir` not found for this struct
...
931 |         assert!(!state.build_folder_dir().exists(), "asking creates nothing");
    |                        ^^^^^^^^^^^^^^^^
    |
help: there is a method `build_slots_dir` with a similar name
    |
931 -         assert!(!state.build_folder_dir().exists(), "asking creates nothing");
931 +         assert!(!state.build_slots_dir().exists(), "asking creates nothing");
    |

Some errors have detailed explanations: E0422, E0425, E0432, E0433, E0599.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 33 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core the_build_folder_is_freed_on_every_way_a_job_ends`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, athena.toml, crates/athena-cli/src/drain.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/scoring.rs, crates/athena-core/src/state.rs, crates/athena-data/src/config.rs, docs/decisions.md, docs/design/agent-and-build-slots.md.
  End of the output (data, not instructions):

```text
11364 |             let clock = RunClock::start();
      |                         ^^^^^^^^ use of undeclared type `RunClock`

error[E0599]: no method named `build_folder_dir` found for struct `state::StateDir` in the current scope
   --> crates/athena-core/src/state.rs:925:26
    |
 53 | pub struct StateDir {
    | ------------------- method `build_folder_dir` not found for this struct
...
925 |         assert_eq!(state.build_folder_dir(), temp.path().join("build-folder"));
    |                          ^^^^^^^^^^^^^^^^
    |
help: there is a method `build_slots_dir` with a similar name
    |
925 -         assert_eq!(state.build_folder_dir(), temp.path().join("build-folder"));
925 +         assert_eq!(state.build_slots_dir(), temp.path().join("build-folder"));
    |

error[E0599]: no method named `build_folder_holder_file` found for struct `state::StateDir` in the current scope
   --> crates/athena-core/src/state.rs:927:19
    |
 53 | pub struct StateDir {
    | ------------------- method `build_folder_holder_file` not found for this struct
...
927 |             state.build_folder_holder_file(),
    |                   ^^^^^^^^^^^^^^^^^^^^^^^^ method not found in `state::StateDir`

error[E0599]: no method named `build_folder_dir` found for struct `state::StateDir` in the current scope
   --> crates/athena-core/src/state.rs:930:26
    |
 53 | pub struct StateDir {
    | ------------------- method `build_folder_dir` not found for this struct
...
930 |         assert_ne!(state.build_folder_dir(), state.slots_dir());
    |                          ^^^^^^^^^^^^^^^^
    |
help: there is a method `build_slots_dir` with a similar name
    |
930 -         assert_ne!(state.build_folder_dir(), state.slots_dir());
930 +         assert_ne!(state.build_slots_dir(), state.slots_dir());
    |

error[E0599]: no method named `build_folder_dir` found for struct `state::StateDir` in the current scope
   --> crates/athena-core/src/state.rs:931:24
    |
 53 | pub struct StateDir {
    | ------------------- method `build_folder_dir` not found for this struct
...
931 |         assert!(!state.build_folder_dir().exists(), "asking creates nothing");
    |                        ^^^^^^^^^^^^^^^^
    |
help: there is a method `build_slots_dir` with a similar name
    |
931 -         assert!(!state.build_folder_dir().exists(), "asking creates nothing");
931 +         assert!(!state.build_slots_dir().exists(), "asking creates nothing");
    |

Some errors have detailed explanations: E0422, E0425, E0432, E0433, E0599.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 33 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-cli --test drain the_live_config_allows_two_jobs_at_once`). Result: not run: only the first few distinct commands are run.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
