# Job 20261105-230157-e1j3: verify "Measure whole-build peak memory and cores in measure-build"

## Request (verbatim)

> try again on 20261105-180320-62j6

## Reading (accepted by the user)

Retry of job 20261105-180320-62j6: keep its optional record fields, status line, docs and tests, but replace the per-process wait4 ru_maxrss measure. Peak memory becomes the highest summed memory of all build processes at one moment, and cores the highest concurrent build process count. Both include sccache-server-spawned compilers, and status labels them as build totals.

Assumptions:
- Goal and audience: the maintainer sizing agent and build slots (docs/design/agent-and-build-slots.md section 7.2) needs the whole-build peak, and judges it from `athena status`.
- Measured quantity: scope = every process of the cold build including compilers run by an sccache server or a detached wrapper; aggregation = maximum over time of the SUM of resident memory of concurrent processes, not a per-process maximum; cores = maximum number of build processes running at the same moment. Consumer = section 7.2 slot sizing.
- Code survey: read crates/athena-core/src/disk.rs (BuildRun line 789, CargoBuilder trait line 803, CargoBuild::build line 818 running `sccache --zero-stats` and RUSTC_WRAPPER=sccache, measure_build line 981, fake builder in tests near line 1537), crates/athena-core/src/status.rs (build_measurement_line line 1132), crates/athena-data/src/records.rs (BuildMeasurement line 2275, validate), crates/athena-core/src/hestia.rs (fake BuildRun at lines 3769 and 3901). Entry is `athena measure-build` -> Athena::measure_build -> CargoBuilder::build, twice (first and second cold build). Records validation is the existing BuildMeasurement validate and the existing AnyRecord round-trip test; no new record file is added, so check_record_file needs no change. The property is measured directly by sampling the build's processes (summing memory from /proc or reading a cgroup v2 memory.peak), not through wait4 ru_maxrss of the cargo child.
- Previous job 20261105-180320-62j6 findings: (1) sccache compilers escape wait4 -> fixed by measuring all processes of the build including sccache-server children, tested with a detaching wrapper; (2) largest-single-process figure -> fixed by summing concurrent processes and labelling status as build totals; (3) procs.rs was an unplanned file -> its wait4/ChildUsage wrapper is dropped or replaced, so procs.rs is a deliverable that may be edited back; (4) old-record test built JSON from the serializer -> the test uses a hand-written old-format JSON literal; (5) agent memory not measured -> stays out of scope, the design doc note says a later step measures it. Cargo.toml and Cargo.lock change only if a process-inspection crate is adopted (buy-first), else stay unchanged.
- The sampler needs a pure, testable core (a function from process snapshots to total memory and count) so the fake-build test can use real child processes with known memory and concurrency without needing sccache installed.

## Acceptance criteria

1. [behavior] A disk.rs or procs.rs test runs a fake build that starts several concurrent child processes of known memory use, one launched through a wrapper that detaches (setsid or double fork, as an sccache server's compiler would be). It asserts the recorded peak memory is at least the known total of the concurrent processes and the cores equal the known concurrency. Run `cargo test -p athena-core -p athena-agent`.
2. [behavior] The same test fails when the figure is reduced to the largest single process, or when the detached wrapper's processes are left out. A companion test asserts a per-process-maximum figure is below the known total, so the test fails if the measure regresses. Run `cargo test -p athena-core -p athena-agent`.
3. [behavior] A test in crates/athena-data/src/records.rs parses a hand-written old-format build-measurement JSON literal without the new fields and the fields are None. Another test round-trips a record with them set, and validate() rejects negative or non-finite values. Run `cargo test -p athena-data`.
4. [behavior] build_measurement_line prints peak memory and cores labelled as build totals (for example 'build total peak memory' and 'peak concurrent processes') and still prints old records without them. Tests in status.rs and crates/athena-cli/tests/status_build_measurement.rs check the text. Run `cargo test -p athena-core -p athena-cli`.
5. [behavior] A measure_build test with a fake builder checks the committed record holds the builder's memory and core figures. Run `cargo test -p athena-core`.
6. [content] CargoBuild::build in crates/athena-core/src/disk.rs measures all processes of the cold build under sccache too (RUSTC_WRAPPER=sccache with its server), by summing resident memory of the build tree plus sccache-server descendants or by reading a cgroup v2 memory.peak. A code comment states the measured quantity and why wait4 is not used.
7. [content] CHANGELOG.md states only what the diff does, with no claim of per-process maximum, and docs/design/agent-and-build-slots.md describes the figures as build totals. The diff touches only build measurement code, the status line, their tests and these two docs.
8. [behavior] `cargo test` passes for the workspace, `cargo clippy --all-targets` is clean and `cargo fmt --check` is clean.

## Expected deliverables

- crates/athena-core/src/disk.rs (modify)
- crates/athena-agent/src/procs.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-cli/tests/status_build_measurement.rs (modify)
- CHANGELOG.md (modify)
- docs/design/agent-and-build-slots.md (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)

## Out of scope

- Measuring agent memory (a later step)
- Build steps 2 to 8 of the slots design (landing stage, slot queue, shared target dirs)
- Changes to athena.toml, defaults.toml or workflows
- Editing existing records under records/
- Running sccache as a separate one-off measurement

## User defaults that apply

- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run a fake build with known concurrency and check the recorded figures.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Workspace test, clippy and fmt criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Replaces the wrong quantity rather than relabelling it.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Test fails if the measure shrinks to a single process or omits the wrapper.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Change limited to measurement code, status line, tests and docs.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Adopt a maintained process-inspection crate if it fits, else /proc sampling or cgroup, recorded in a code comment.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base dbb4b097269d → candidate a129bc810985. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                       |  10 +
 crates/athena-agent/src/procs.rs                   | 285 +++++++++++++++++++++
 .../athena-cli/tests/status_build_measurement.rs   |  12 +-
 crates/athena-core/src/disk.rs                     |  73 +++++-
 crates/athena-core/src/hestia.rs                   |   3 +
 crates/athena-core/src/status.rs                   |  41 ++-
 crates/athena-data/src/records.rs                  | 102 ++++++++
 docs/design/agent-and-build-slots.md               |   6 +
 8 files changed, 525 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-230157-e1j3/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-230157-e1j3/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42127514378 for a129bc8109853808a6b5c57c3e697d545844fd31)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42127514378 for a129bc8109853808a6b5c57c3e697d545844fd31)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42127514378 for a129bc8109853808a6b5c57c3e697d545844fd31)
- ci-result: `not a check` passed (Check results for a129bc8109853808a6b5c57c3e697d545844fd31 come from the CI run https://github.com/doodla/athena/actions/runs/42127514378 (509s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test` passes for the workspace, `cargo clippy --all-targets` is clean and `cargo fmt --check` is clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
