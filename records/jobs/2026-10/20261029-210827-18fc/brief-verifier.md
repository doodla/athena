# Job 20261029-210827-18fc: verify "Add 'athena measure-build' with Hestia sweep trigger and status display"

## Request (verbatim)

> Priority: urgent
> Async-drain build job A1 again, as code (docs/design/async-drain.md, decision D145). A producer cannot take these measurements: its restricted working copy cannot run sccache or two full builds, and job 20261029-142844-ynv3 returned with no numbers twice. Instead add 'athena measure-build', which Athena runs in its own environment: it builds the workspace cold in two separate worktrees with different target directories, records T (one cold target/ size), S (worktree source size), the wall time of each build, and sccache hit and miss counts if sccache is installed (or that it is not), and writes them as a typed measurement record. The Hestia sweep runs it once when no measurement record exists and again when Cargo.lock changes, and athena status shows the latest values. Mark decision D145 accepted, decided by the brain under Sri's delegation (2026-10-29).
> Acceptance:
> - A test with a fake builder shows measure-build writes a measurement record with T, S, both build times and the cache figures, and that the record round-trips through the records schema.
> - A test shows the sweep runs it when no record exists and not again until Cargo.lock changes.
> - athena status shows the latest measurement.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry; docs/decisions.md marks D145 accepted.
> Files: crates/athena-cli/src/main.rs, crates/athena-core/src/disk.rs, crates/athena-data/src/records.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/status.rs, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Retries job 20261029-142844-ynv3 (A1, D145) as code: the previous docs-only attempt measured nothing and was rejected. Add `athena measure-build`, which Athena runs itself to do two cold worktree builds, record T, S, build times and sccache counts as a typed record. The Hestia sweep triggers it when no record exists or Cargo.lock changed, and `athena status` shows the latest values. Audience: Sri and the async-drain build jobs.

Assumptions:
- Code survey: I read crates/athena-data/src/records.rs (AnyRecord at line 2512, check_record_file at 2819), crates/athena-core/src/hestia.rs (run_sweep at 99, Check trait), crates/athena-core/src/disk.rs, crates/athena-core/src/status.rs (BUILD_PATHS includes Cargo.lock) and crates/athena-cli/src/main.rs. The new record kind must be added to AnyRecord and accepted by check_record_file, and the existing records round-trip test must cover it.
- Retry of job 20261029-142844-ynv3: its finding was that nothing was measured. This job addresses it by building the tool that measures, so Athena produces the numbers, not a producer. Its docs-only candidate files are kept: CHANGELOG.md and docs/decisions.md (D145 already accepted by the brain under Sri's delegation, 2026-10-29, and the A1 entry is rewritten to say the measurement is automated and not yet run). docs/design/async-drain.md keeps its accepted status line, with no further change expected.
- The builder is behind a trait so tests use a fake; the real builder runs cargo build in two git worktrees with different CARGO_TARGET_DIRs, and reads sccache --show-stats only if sccache is on PATH, otherwise records 'not installed'.
- The sweep decides to run using the stored Cargo.lock hash in the latest measurement record versus the current one. No athena.toml change is expected.

## Acceptance criteria

1. [behavior] Run `cargo test`: a test in crates/athena-core/src/disk.rs drives measure-build with a fake builder and asserts the written record has T, S, both build wall times and the sccache hit/miss counts, and also covers the sccache-not-installed case.
2. [behavior] Run `cargo test`: the new measurement record round-trips through check_record_file / AnyRecord in crates/athena-data/src/records.rs (serialize, validate, parse back equal).
3. [behavior] Run `cargo test`: a test in crates/athena-core/src/hestia.rs shows run_sweep runs the measurement when no record exists, does not run it on the next sweep, and runs it again after the Cargo.lock hash changes.
4. [behavior] Run `cargo test`: a test in crates/athena-core/src/status.rs shows `athena status` output contains the latest T, S, build times and cache figures, and a clear line when no measurement exists.
5. [content] crates/athena-cli/src/main.rs registers a `measure-build` subcommand, shown in the help text, that calls the core function; Grep finds it.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
7. [content] CHANGELOG.md has an entry for `athena measure-build`, the sweep trigger and the status line, and does not claim any number was measured.
8. [content] docs/decisions.md marks D145 accepted, decided by the brain under Sri's delegation (2026-10-29), and the A1 measurements entry says the numbers come from `athena measure-build` records (no invented figures).
9. [content] athena.toml, defaults.toml and .github/workflows are unchanged, and no existing drain test is modified.

## Expected deliverables

- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/status.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/async-drain.md (modify)
- crates/athena-core/src/lib.rs (modify)

## Out of scope

- Actually running the measurement or recording measured numbers in docs (Athena does that later)
- Any athena.toml change or re-enabling sccache
- Implementing read-ahead, per-job worktrees or max_in_flight
- Changing drain behavior or existing drain tests
- Finding the cause of D84's 0 sccache hits beyond recording the hit/miss counts

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code is Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for each module touched; fmt, clippy -D warnings, test pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the record contents, sweep behavior and status output, not wiring.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and code comments written for outside readers.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Sweep trigger is enforced in code and tested.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Test in disk.rs. Entry point: `Athena::measure_build(&dyn CargoBuilder, &dyn Ui)` on an Athena over a temp git repo with a committed Cargo.lock and two tracked files of known sizes, built with the existing `crate::code_health::tests::set_up` pattern. Guard and precondition: HEAD has a Cargo.lock. Seam: a `FakeBuilder` in the test module, passed as `&dyn CargoBuilder`; the trait is added in step 4 and does not exist today. The fake writes a file of known length into the target_dir it is given and returns fixed seconds (such as 61.5 and 12.25) and sccache counts. Worktree creation, sizing, hashing and record writing are the real production code. Two tests: sccache installed, and `sccache_installed() == false`. `parse_sccache_stats` has unit tests for real-looking text and garbage. Run: `cargo test -p athena-core disk::tests::measure_build`. Observes: After `Records::load`, `build_measurements` holds one record with `target_bytes` equal to the written file length, `source_bytes` equal to the sum of the tracked file sizes, `first_build_seconds` 61.5, `second_build_seconds` 12.25, and `sccache` equal to Measured{first, second} (or NotInstalled). The fake saw two distinct worktree paths and two distinct target dirs. After the call neither worktree nor target dir exists, also after a fake that returns an error.
  Fails without the behavior: Without the behavior no record is committed, or T, S, the build times or the cache figures differ from the fixture, and an assertion fails. In the not-installed case, a record that claims counts fails the `NotInstalled` assertion. If the logic were missing and only the fake returned a record, the sizes asserted from real files would not match.
- Criterion 2: Test in records.rs beside `a_throughput_review_has_a_path_a_schema_and_round_trips`. Entry points: `AnyRecord::to_json`, `canonical_path`, `schema` and `check_record_file(path, bytes)`. No seam: a constructed `BuildMeasurement` value, in both the `Measured` and `NotInstalled` variants. A second test calls `validate`/`check_record_file` with a NaN or negative time and an empty lock hash. Run: `cargo test -p athena-data build_measurement`. Observes: `check_record_file(&record.canonical_path(), json.as_bytes())` returns `Ok(Some(parsed))` and `parsed == record`. The schema is `athena.build-measurement/1`. The path matches `records/build-measurements/<yyyy-mm>/<id>.json`. Bad numbers are refused with `RecordError::Invalid`, and a record at the wrong path with `Misplaced`.
  Fails without the behavior: If the `AnyRecord` variant is missing, `check_record_file` returns a parse or unknown-file error and the round trip fails. If the canonical path is wrong it returns `Misplaced`. If `validate` is missing, the bad-number case is accepted and its assertion fails.
- Criterion 3: Test in hestia.rs. Entry point: `Athena::hestia_sweep_measuring(&[], &Quiet, &CountingBuilder)`, the function `hestia_sweep_with` calls after its review passes. `run_sweep` itself is a pure function over `Check`s with no records access, so the trigger is tested at this sweep entry. The split is added in step 8; it does not exist today. Guards and preconditions: HEAD has a Cargo.lock, and free disk is above `jc.config.min_free_gb()` (a real temp dir). Seam: a `CountingBuilder` with a `Cell<u32>` call counter, defined in the test and injected as `&dyn CargoBuilder`; it writes a small file into its target dir and returns fixed seconds. Steps: sweep once with no record; sweep again; commit a changed Cargo.lock; sweep a third time. `measure_build_due` also has pure unit tests. A repo with no Cargo.lock at HEAD is swept and gets zero calls. Run: `cargo test -p athena-core hestia::tests::sweep_measures_build`. Observes: After sweep 1 the builder was called twice (two builds), one record exists, and the output has a `build measurement` line. After sweep 2 the count is unchanged and there is still one record. After the Cargo.lock change, sweep 3 raises the count by two, there are two records, and the newer one has the new `cargo_lock_hash`.
  Fails without the behavior: If the sweep never runs the measurement, the builder call count stays 0 after sweep 1. If it ignores the stored hash, the count is 2 after sweep 2, or stays at 2 after the lock change. Each fails an assertion. If store.rs had no load arm, `Records::load` would panic in this test.
- Criterion 4: Test in status.rs. Entry points: `Athena::status()` on a temp repo, and the pure `build_measurement_line(Option<&BuildMeasurement>)`. The test commits two records through `commit_records` (the newer must win) and reads `StatusView.build_measurement`. A second case has no record. A display.rs test (step 10) renders `display::status(&view)` and checks that the line is in the printed text. No fake is needed; `StatusView.build_measurement` and `build_measurement_line` are added in step 9. Run: `cargo test -p athena-core status::tests::build_measurement`. Observes: The line contains the newest record's T and S (through `format_bytes`), both build times, and the hit and miss counts, or the words 'sccache not installed'. It names the short commit and the time. With no record it reads a clear line, such as 'build measure: none yet (athena measure-build, or the Hestia sweep, makes one)'.
  Fails without the behavior: If `StatusView` carries no measurement line, the test finds no `build measure` line. If the status shows an older record, the values differ from the newest record's and the assertions fail.
- Criterion 5: Grep `MeasureBuild` and `measure-build` in crates/athena-cli/src/main.rs; run the parse test in main.rs using `Cli::try_parse_from(["athena","measure-build"])`, like the existing `hestia_review_takes_backfill...` test; and run the help. The match arm calls `open(true)?.measure_build(&CargoBuild, &Terminal)`, with `CargoBuild` the real builder from disk.rs. The subcommand and its arm are added in step 10. A second command, `cargo run -p athena-cli -- --help`, is run to see the listing. Run: `cargo test -p athena-cli measure_build`. Observes: The Grep finds the enum variant and the arm that calls the core function. `--help` lists `measure-build` with its doc comment. The parse test passes.
  Fails without the behavior: Without the subcommand, clap rejects `measure-build`, the parse test fails, and the Grep for `MeasureBuild` finds nothing.
- Criterion 6: Run the three Athena checks as three separate commands: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, and `cargo test --locked`. This entry names the last one. The full suite includes the budgets and policy tests, which scan CHANGELOG and docs. Run: `cargo test --locked`. Observes: All three exit 0. No existing test is edited; the new tests only add to the touched files.
  Fails without the behavior: Without the work done: `cargo fmt --all --check` exits non-zero on any unformatted new code. `cargo clippy --all-targets --locked -- -D warnings` fails on a warning, or on a non-exhaustive match when the `AnyRecord::BuildMeasurement` variant is added without handling. `cargo test --locked` fails if a new test fails, or if `Records::load` panics because store.rs lacks the arm, or if a policy or budget test finds a banned word or a stale line in the docs.
- Criterion 7: Read the CHANGELOG.md diff and Grep it for `measure-build`. A policy test in the suite scans it for banned words. Run: `git diff -- CHANGELOG.md`. Observes: An Unreleased 'Added' entry names `athena measure-build`, the sweep trigger (no record, or Cargo.lock changed) and the status line. It says the numbers come from running the command and that none has been recorded yet. The earlier A1 bullet that calls the A1 entry 'not a measurement' is reworded. No figure such as '6 GB' appears as a result.
- Criterion 8: Read the diff and Grep decisions.md for the D145 heading and for `athena.build-measurement/1`. At the base snapshot the D145 heading already carries the accepted marking; the producer keeps it exactly and rewrites the A1 measurements entry and the 'Effect on D145' bullet. Run: `git diff -- docs/decisions.md`. Observes: The D145 heading says accepted, decided by the brain under Sri's delegation (2026-10-29). The A1 entry names `athena measure-build` and the record schema, says the figures come from those records (written by the sweep or by hand, shown by `athena status`), says none is recorded yet, and contains no measured number. Estimates stay labelled as estimates.
- Criterion 9: Run `git status` and `git diff --stat`, and check the changed paths. Run: `git status`. Observes: The changed files are exactly the plan's list. athena.toml, defaults.toml, .github/ and the drain tests (crates/athena-cli/src/drain.rs and crates/athena-cli/tests/) show no diff. The touched test modules only gain new tests.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- docs/decisions.md (modify)
- docs/design/async-drain.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 289c423fa502 → candidate 88e767a79112. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                       |  18 +-
 crates/athena-cli/src/display.rs                   |   1 +
 crates/athena-cli/src/main.rs                      |  41 ++
 .../athena-cli/tests/status_build_measurement.rs   |  63 +++
 crates/athena-core/src/disk.rs                     | 546 ++++++++++++++++++++-
 crates/athena-core/src/hestia.rs                   | 254 +++++++++-
 crates/athena-core/src/lib.rs                      |   5 +-
 crates/athena-core/src/status.rs                   | 152 +++++-
 crates/athena-core/src/store.rs                    |   7 +
 crates/athena-data/src/records.rs                  | 167 +++++++
 docs/decisions.md                                  |  53 +-
 docs/design/async-drain.md                         |   6 +-
 12 files changed, 1271 insertions(+), 42 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1709 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
