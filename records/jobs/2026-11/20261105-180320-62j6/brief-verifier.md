# Job 20261105-180320-62j6: verify "Record peak memory and cores for cold builds; show in status"

## Request (verbatim)

> Measure agent memory, build memory, CPU and the slot-relevant sizes (step 1). Build step from docs/design/agent-and-build-slots.md section 8; follow that section for the files touched and how the effect is measured.
> 
> Acceptance:
> - the build-measurement record gains peak resident memory and cores used for a cold build; old records still read
> - athena status prints the new figures
> - tests cover the new fields and reading an old record

## Reading (accepted by the user)

Build step 1 of the slots design: each cold build in `athena measure-build` also records peak resident memory and cores used, stored as optional fields on `BuildMeasurement`, so old records still parse. The `build` line of `athena status` prints them for the maintainer sizing slots.

Assumptions:
- Code survey: read crates/athena-core/src/disk.rs (BuildRun, CargoBuilder, CargoBuild::build runs cargo via Command::output and times it; Athena::measure_build builds two cold worktrees and commits the record), crates/athena-data/src/records.rs (BuildMeasurement with deny_unknown_fields; validate; AnyRecord::BuildMeasurement; round-trip test a_build_measurement_has_a_path_a_schema_and_round_trips), crates/athena-core/src/status.rs (build_measurement_line, tests in build_measurement_tests), crates/athena-cli/tests/status_build_measurement.rs. Entry point is `athena measure-build`; status reads the latest record.
- Records validation: the schema tag athena.build-measurement/1 stays; new fields are Option with #[serde(default, skip_serializing_if = "Option::is_none")] so old records parse under deny_unknown_fields and the AnyRecord round-trip stays valid; validate() rejects non-finite or negative values. No new record or data file is added.
- Peak resident memory and cores are measured directly from the cargo process tree (e.g. wait4 rusage maxrss of the child plus CPU time divided by wall time as cores used), not estimated; the exact mechanism is the producer's choice, preferring an existing maintained crate over hand-rolled code. Peak memory and cores are recorded per cold build (first and second), or for the first at minimum.
- Audience is the maintainer sizing agent and build slots (section 7 of the design). The one-off sccache run in the design text uses code that already exists and needs no change. Agent memory measurement is read as out of this step's record-field scope beyond the build record, since the acceptance names only the build record.
- Changes to disk.rs fake builders in tests (FakeBuilder) and BuildMeasurement literals in hestia.rs, status.rs and the CLI test are needed to compile.

## Acceptance criteria

1. [content] BuildMeasurement in crates/athena-data/src/records.rs gains optional peak-resident-memory and cores-used fields with serde defaults, and validate() rejects negative or non-finite values.
2. [content] BuildRun and CargoBuild::build in crates/athena-core/src/disk.rs capture peak resident memory and cores used of each cold build from the cargo process, and measure_build stores them in the record.
3. [behavior] A test parses a JSON build-measurement record written without the new fields and it deserializes with those fields None; a test round-trips a record with them set. Run `cargo test -p athena-data`.
4. [behavior] build_measurement_line prints the peak memory and cores when present and still prints old records without them; tests in status.rs and crates/athena-cli/tests/status_build_measurement.rs check the printed text. Run `cargo test -p athena-core -p athena-cli`.
5. [behavior] A disk.rs test with a fake builder checks measure_build writes the builder's memory and core figures into the committed record. Run `cargo test -p athena-core`.
6. [behavior] `cargo test` passes for the workspace and `cargo clippy --all-targets` and `cargo fmt --check` are clean.
7. [content] The design doc docs/design/agent-and-build-slots.md or CHANGELOG.md notes the new fields briefly, if the repository convention records such changes.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-cli/tests/status_build_measurement.rs (modify)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)
- CHANGELOG.md (modify)
- docs/design/agent-and-build-slots.md (modify)

## Out of scope

- Running sccache as a separate one-off measurement (existing code already does it)
- Build steps 2 to 8 of the slots design (landing stage, slot queue, shared target dirs)
- Changes to athena.toml, defaults.toml or workflows
- Editing existing records under records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for new fields, old-record reading and status output; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check printed status text and deserialized values by running tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is memory and cores visible in the status build line.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Prefer an existing maintained crate or std/libc rusage for resource measurement.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to the build-measurement path.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 3c90ebe235fb → candidate 76c9ec2d3cad. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                       |  6 ++
 crates/athena-agent/src/procs.rs                   | 61 +++++++++++++++++-
 .../athena-cli/tests/status_build_measurement.rs   |  6 ++
 crates/athena-core/src/disk.rs                     | 50 +++++++++++++--
 crates/athena-core/src/hestia.rs                   |  6 ++
 crates/athena-core/src/status.rs                   | 41 +++++++++++-
 crates/athena-data/src/records.rs                  | 73 ++++++++++++++++++++++
 docs/design/agent-and-build-slots.md               |  3 +
 8 files changed, 239 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-180320-62j6/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-180320-62j6/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42124088268 for 76c9ec2d3cad2b4d0944fb6e6831ba0b494714f8)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42124088268 for 76c9ec2d3cad2b4d0944fb6e6831ba0b494714f8)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42124088268 for 76c9ec2d3cad2b4d0944fb6e6831ba0b494714f8)
- ci-result: `not a check` passed (Check results for 76c9ec2d3cad2b4d0944fb6e6831ba0b494714f8 come from the CI run https://github.com/doodla/athena/actions/runs/42124088268 (467s): fmt success, clippy success, test success.)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: planned files left unchanged without a reason: crates/athena-core/Cargo.toml, Cargo.lock)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo test` passes for the workspace and `cargo clippy --all-targets` and `cargo fmt --check` are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
