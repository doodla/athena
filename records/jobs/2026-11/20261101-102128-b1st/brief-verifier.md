# Job 20261101-102128-b1st: verify "Reclaim interrupted jobs' build dirs and gate disk before the producer"

## Request (verbatim)

> try again on 20261030-185437-1tbs

## Reading (accepted by the user)

Retry of job 20261030-185437-1tbs (failed in planning: its test fixture include could not compile in a lib unit test). Reclaim target/ of interrupted (Dispatched, no registry entry, free lock) jobs at drain start and before each run, keeping work trees and candidates, and hold the job as an environment stop on low disk before the producer. Audience: whoever runs `athena drain` unattended.

Assumptions:
- Code Survey: Code survey: read crates/athena-core/src/disk.rs (sweep_idle_targets at :160 uses try_lock_job and remove_target and skips only `keep`; free-bytes hook via FREE_BYTES_ENV at :130; disk_gate at :446), crates/athena-core/src/run/mod.rs (disk_gate calls at :749 and :1301, before dispatch), crates/athena-core/src/run/close.rs (hold_for_disk :176), crates/athena-core/src/state.rs (remove_target :251, with_free_space :224), crates/athena-cli/src/drain.rs (drain :2273, drain_concurrent :2284, drain_with :2332) and the integration tests crates/athena-cli/tests/drain.rs with its Fixture. No record or data file is added, so no records allow-list change.
- Job 20261030-185437-1tbs: its candidate changed no files and nothing is kept. Its planning findings are addressed as follows. (major) The #[path] include of tests/common/mod.rs into a drain.rs unit test cannot compile (it names athena_cli::wiring::capabilities); fixed by putting the tests in the existing integration test file crates/athena-cli/tests/drain.rs, which already uses Fixture and can import athena_cli. (minor) Criterion 2 passed on old code because the dispatch gate already holds before the agents; settled by reading the code: the producer is entered in run/producer.rs, so the change adds a disk_gate before the producer there, holding through hold_for_disk, and the user is not asked. (nit) Do not rely on a bare Job stub in Records::load; the disk.rs test uses a real ask+go flow with a killed producer if a stub is not accepted.
- Existing code: reuse crates/athena-core/src/disk.rs sweep_idle_targets and the lock/remove logic, crates/athena-core/src/state.rs remove_target and try_lock_job, crates/athena-core/src/run/close.rs hold_for_disk and crates/athena-core/src/run/mod.rs disk_gate. Extend them; do not build a parallel mechanism.
- Interrupted means: records show Dispatched, there is no registry entry (process gone), and the job lock is free. The lock is held during removal and the drain's own jobs are skipped. Only target/ is removed.
- The scope widens beyond the earlier file list to crates/athena-core/src/run/producer.rs (and possibly run/mod.rs) because the pre-producer gate belongs there (root-cause default). The earlier list's athena-cli drain.rs, athena-core disk.rs and CHANGELOG.md stay in scope.

## Acceptance criteria

1. [behavior] A new integration test in crates/athena-cli/tests/drain.rs, run with `cargo test -p athena-cli --test drain`, enters through the drain entry point with an interrupted Dispatched job's target/ present and asserts target/ is gone after drain start while the job's work tree and candidate remain.
2. [behavior] A new test, run with `cargo test`, makes a job's lock held or its process alive (registry entry) and asserts its target/ is kept, and asserts a sweep before a later run reclaims a newly interrupted job's target/.
3. [behavior] A new test, run with `cargo test`, simulates low free disk only at the pre-producer point (via the free-bytes hook or with_free_space) and asserts the job is held as an environment stop (low disk), the status says so, and the producer is never started.
4. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
5. [content] Grep of the diff shows a disk gate before the producer is invoked in crates/athena-core/src/run/producer.rs (or its caller) that holds through hold_for_disk, and no test fixture is pulled in by #[path] from tests/common into a src unit test.
6. [content] CHANGELOG.md has a new entry describing the reclaim of interrupted jobs' build directories and the pre-producer disk check; `git diff` shows it. Module docs in drain.rs and disk.rs describe the new behavior.
7. [content] The diff does not touch athena.toml or defaults.toml, does not change the min_free_gb default, and does not delete work trees or candidates.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)

## Out of scope

- Deleting work trees or candidates of interrupted jobs
- Changing the min_free_gb default or athena.toml / defaults.toml
- Changing how closed and idle jobs are reclaimed
- Automatic retry policy changes for held jobs
- Including tests/common/mod.rs into src unit tests

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus cargo test and clippy -D warnings.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Reclaims at the source and gates the producer itself, not only the checks.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria exercise drain and the gate through tests.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by the gate and sweep code, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in a few files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the earlier job's request and failure from the brief.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration test in crates/athena-cli/tests/drain.rs entering through `drain::drain(&RealSteps, limits, &Quiet)`, with an interrupted job's `target/` planted. Run: `cargo test -p athena-cli --test drain an_interrupted_jobs_build_directory_is_reclaimed_at_drain_start`. Observes: Setup: `Fixture::new()`, `killing_producer(kill)`, `request_on(&athena, "web", ..)` and `interrupt()` make a real Dispatched job with no outcome on a channel the drain does not work. The test plants `jobs/<id>/target/debug/big` (100 KB), `jobs/<id>/work/main.rs` and `kept/<id>/work/kept.rs` (via `state.kept_work_dir`). No registry entry, no lock held. After `drain` returns: `jobs/<id>/target` is gone; `work/main.rs` and the kept file keep their bytes; Records still show the job Dispatched with its records intact; the ui `Lines` contains `disk       removed build directory of job <id>`; the report has no jobs run, so only the drain-start call can have removed it.
  Fails without the behavior: Without the `steps.reclaim_interrupted(&[])` call in `drain_with`, `target/` stays and the assert fails. A whole-directory removal fails the work-file and kept-file asserts. A Records filter on closed jobs leaves the dispatched job's directory.
  Test seam: Entry point `drain::drain` with `RealSteps` (exists, drain.rs:2759; `real_steps_with` at tests/drain.rs:2573 builds it with a given ui). Fixture, Script, `request_on`, `interrupt`, `killing_producer` exist in tests/drain.rs. `Steps::reclaim_interrupted` and `Athena::reclaim_interrupted` are added by steps 4, 5 and 7. No double holds the rule.
- Criterion 2: Three tests: (a) integration guards of the real rule through `Athena::reclaim_interrupted`, including the later-run sweep; (b) integration wiring through the recording `Stub`, asserting the keep-list choice; (c) disk.rs unit test of lock and keep list on a bare StateDir. Run: `cargo test -p athena-cli --test drain the_reclaim_keeps_live_locked_closed_and_own_jobs && cargo test -p athena-cli --test drain drain_reclaims_at_start_and_before_each_run && cargo test -p athena-core the_interrupted_sweep_skips_locked_and_kept_jobs`. Observes: (a) Jobs made interrupted with `request_on` + `interrupt`, each with planted `target/` and `work/`: A no process; B with a live `sleep 60` child (`std::process::Command` + `process_group(0)`, `athena.state.registry().register(pgid, B, "test")`, kill-and-wait guard); C with its lock held by `athena.state.try_lock_job`; E on the keep list; plus D, a job closed via queue + go + `athena.close(.., "no longer wanted", &Quiet)` as in tests/drain.rs:3308, with a `target/` planted after the close. `athena.reclaim_interrupted(&[E], &lines)` removes only A's `target/`; B, C, D and E keep theirs; every `work/` stays. A second call with the lock released and B's child killed (registry reaped) removes B and C. A job interrupted after the first sweep is removed by the next sweep (the later-run case). (b) Stub with `interrupted: ["x"]` and a second item: `reclaimed` has one call `(0, [])` before the first `run`, then per run a call whose `own` contains that run's `read.job` ('x-retry' for the retried item) and the jobs of lines already pushed, and does NOT contain the interrupted source 'x' (it is swept on purpose). (c) bare StateDir: locked and keep-listed jobs keep `target/`, the others lose it, `work/` stays.
  Fails without the behavior: Ignoring the registry removes B's target; ignoring the lock removes C's; filtering on closed instead of Dispatched removes D's and keeps A's; ignoring the keep list removes E's. A drain that reclaims only at start gives (b) one call, not one per run; dropping the `run_job` call does the same; putting `item.job` on the list fails the 'x' not-in-own assert.
  Test seam: Entry points `Athena::reclaim_interrupted` (added by step 4) and `drain::drain` over the Stub (exists; Stub gets one new RefCell field in step 9). `Registry::register` (athena-agent procs.rs:149), `StateDir::try_lock_job` (state.rs:136), `Athena::close` exist. The rule under test is production code; the Stub only records calls.
- Criterion 3: Integration test through `drain::drain` with `RealSteps` and the production dispatch, producer and retry path. Space is low only after dispatch; a second drain with space free checks the hold's lifecycle. Run: `cargo test -p athena-cli --test drain a_job_is_held_before_the_producer_when_the_disk_fills_after_dispatch`. Observes: Setup: `disk_fixture()` (10 GB required), a Script counting `Role::Producer` calls in an `Arc<AtomicUsize>`, `athena.state.with_free_space(simulated_free)` with no `simulated_free` file (plenty), `queue_steward_request`, and a `Ui` wrapper over `Lines` that writes 3_000_000_000 into `<state>/simulated_free` on the first line starting `baseline`. First drain: `drain(&RealSteps{ui: &wrapper, ..}, work_everything(), &Quiet)`. Asserts: the item's outcome is `environment: low disk`, display text has `3.0 GB` and `10.0 GB`, `failed_item` is None, nothing landed; producer counter is 0; Records show the job H Dispatched with a job record, no outcome, no score (so it passed the dispatch gate); ui lines ordered `dispatched`, `baseline`, then `held       environment: low disk`; `disk::read_hold(H)` gives 3 GB / 10 GB; `display::status(&athena.status().unwrap())` shows `environment: low disk: 3.0 GB free, 10.0 GB required` on H; `athena.go(H)` errors with text that says it is read again and not 'resumes it from its kept candidate'; no line tells anyone to `athena retry`. Lifecycle (finding 1): delete `<state>/simulated_free` (plenty), run a second drain on the same Fixture: it retries H as a new job H2 and runs the producer (counter 1); then `disk::read_hold(H)` is None and the status text no longer shows H as held.
  Fails without the behavior: Without `producer_disk_gate` the producer runs (counter is 1) and the job is held only later by the check gate, so the first counter assert fails. If the hold were counted as a failure or committed an outcome, the outcome and Records asserts fail. Without `release_superseded_hold` the second-drain `read_hold(H)` is still Some and the lifecycle assert fails; without the `go` message change the message assert fails.
  Test seam: Entry point `drain::drain` with `RealSteps` (exists), `StateDir::with_free_space` on the public `Athena.state` (exists, used at tests/drain.rs:1498 and 1753), fake driver Script counting calls (pattern at tests/drain.rs:1851), Ui wrapper defined in the test and injected via `RealSteps.ui`. The gate is added by steps 1 and 2; the lifecycle by step 3. The wrapper only writes a file at the production `baseline` line and holds no gate logic.
- Criterion 4: The three Athena checks over the whole workspace after all changes. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All exit 0. Every existing `Steps` implementor compiles against the defaulted `reclaim_interrupted`; the existing hold tests (`low_disk_*`, `the_real_steps_hold_a_job_on_low_disk_*`, `low_disk_after_the_build_holds_*`) and run/tests.rs disk tests (including the ones at tests.rs:1675, :1794, :1825 that read the hold marker) still pass with the new gate; no unused-import or dead-code warning.
  Fails without the behavior: A missed fmt fails the first command; a badly shaped helper fails clippy; a `Steps` method without a default fails compilation; removing the reclaim calls or the gate turns the criterion 1, 2 and 3 tests red.
  Test seam: The three cargo commands on the real workspace. No fake. Step 14 runs them.
- Criterion 5: Read the diff for the gate and for any `#[path]` include. Run: `git diff -- crates/athena-core/src/run/producer.rs crates/athena-core/src/run/mod.rs crates/athena-cli/src`. Observes: The diff adds `producer_disk_gate` in run/producer.rs and, in `produce_and_verify`, a call after `baseline_stage` and before `producer_snapshot` that returns `Err(self.hold_for_disk(ctx, closing, low))`. A search of the diff output for `#[path` and `tests/common` finds no added line in any src file.
- Criterion 6: Read the CHANGELOG and module-doc diffs. Run: `git diff -- CHANGELOG.md crates/athena-cli/src/drain.rs crates/athena-core/src/disk.rs crates/athena-core/src/run/close.rs`. Observes: CHANGELOG.md gains an Unreleased bullet (Fixed group if present): root cause (an interrupted job's build directory was reclaimed only at dispatch or under low disk, so after a container restart it stayed until the disk filled and the next job failed), the fix (reclaim at drain start and before each run, work tree and candidate kept, the drain's own jobs skipped), the new disk check just before the producer that holds the job as an environment stop (not a failure, no attempt used), and that the retry spends the hold. No email or host name. drain.rs `//!` (interrupted-job bullet and 'Low disk holds a job' bullet), disk.rs `//!` (place list, 'Freeing and resuming') and the `hold_for_disk` doc describe the reclaim, its guards (Dispatched, no registry entry, lock free, own jobs kept, only `target/`) and the pre-producer check as a fourth place.
- Criterion 7: List changed paths and search the diff. Run: `git diff --stat`. Observes: Only crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs, crates/athena-core/src/disk.rs, crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/close.rs (doc only) and CHANGELOG.md appear. No athena.toml or defaults.toml, no `min_free_gb` default line changed, and no removal of `work` or candidate paths (the only deletion calls added are `remove_target` and `clear_hold`).

Planned files:

- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 53060afb405a → candidate ea539d9d3b63. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |  13 ++
 crates/athena-cli/src/drain.rs         |  41 ++++-
 crates/athena-cli/tests/drain.rs       | 300 +++++++++++++++++++++++++++++++++
 crates/athena-core/src/disk.rs         | 140 +++++++++++++--
 crates/athena-core/src/run/close.rs    |  10 +-
 crates/athena-core/src/run/mod.rs      |  37 ++++
 crates/athena-core/src/run/producer.rs |   8 +
 7 files changed, 528 insertions(+), 21 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41301595930 for ea539d9d3b63a9974fc53e6c09da401577ab4354)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41301595930 for ea539d9d3b63a9974fc53e6c09da401577ab4354)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41301595930 for ea539d9d3b63a9974fc53e6c09da401577ab4354)
- ci-result: `not a check` passed (Check results for ea539d9d3b63a9974fc53e6c09da401577ab4354 come from the CI run https://github.com/doodla/athena/actions/runs/41301595930 (597s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
