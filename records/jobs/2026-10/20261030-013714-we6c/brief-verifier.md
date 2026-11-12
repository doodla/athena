# Job 20261030-013714-we6c: verify "Give OLY-B-RUN-02 a mechanism: one-command pinned idempotent setup"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-B-RUN-02 a mechanism
> 
> Requirement OLY-B-RUN-02 in docs/spec.toml is active and has no mechanism.
> Requirement: Environment changes are repeatable: every setup step is committed, idempotent and version-pinned, and a fresh environment reaches a working state with one command.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-B-RUN-02
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-B-RUN-02 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Record the existing committed setup path (setup.sh handing off to `xtask setup`) as OLY-B-RUN-02's mechanism in docs/spec.toml, with a measure ending `status: not shown` that names a new test. Add that test in xtask/tests/, which checks the spec entry and that the setup path is version-pinned and idempotent. The audience is the Athena owner and the verifier, who run cargo test.

Assumptions:
- Code Survey placeholder removed below; see the next entry.
- Code survey: read crates/athena-core/src/preflight.rs (OLY-B-RUN-01 precedent: its spec entry is checked by a unit test that calls hestia::parse_spec, checks mechanism and owner, requires the measure to end `status: not shown`, and requires spec_gaps to omit the id), crates/athena-core/src/hestia.rs (spec_gaps flags an active requirement whose mechanism is blank or `none` or whose measure_problem is Some; measure_problem accepts `status: not shown...` or a label in status::STATUS_MEASURE_LABELS; the allow-list is that label list), xtask/tests/spec.rs (the spec parse requires id, statement, kind, status; mechanism, owner and measure default to empty), setup.sh (a pinned rustup 1.28.2 with sha256 per platform, then `exec cargo run --locked --release -p xtask -- setup`), xtask/src/main.rs (entry `setup` at line 181: idempotent steps, pinned gitleaks and sccache, refuses inside a job), and rust-toolchain.toml (channel 1.97.0). Resolution order in setup: cargo is found on PATH or in CARGO_HOME, then rustup-init is fetched and verified only if cargo is missing, then xtask setup runs. The change adds no record or data file under crates/, so check_record_file and AnyRecord are not touched. The property is measured directly by reading the committed files in a test, not through a stand-in; the test must fail while the spec says `none`.
- The mechanism is already built: setup.sh plus `xtask setup` is the one command. The job records it as the mechanism and adds the test that proves each property, rather than building a new path (buy-first, small).
- The measure uses `status: not shown` because no `athena status` line reports setup repeatability.
- No governance file changes: rust-toolchain.toml, setup.sh, athena.toml and Cargo files are only read. The change adds no refusal rule.
- Consumers: none found; no config file is a deliverable.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-B-RUN-02 has `mechanism` naming setup.sh and xtask/src/main.rs (not `none`), keeps owner `Athena`, and has a `measure` that names the new test and ends with `status: not shown`.
2. [content] A new test file xtask/tests/setup_repeatable.rs exists and asserts: the spec entry's mechanism names files that exist; setup.sh pins RUSTUP_VERSION with a sha256 check; rust-toolchain.toml channel is an exact x.y.z version; setup.sh ends in `cargo run --locked ... -- setup` as the single command.
3. [behavior] Run `cargo test -p xtask --test setup_repeatable`: it passes. Reverting the docs/spec.toml mechanism to `none` (checked by a unit case that parses a copy with the mechanism blanked and finds a gap) makes the test's gap assertion fail, so the test fails without the mechanism.
4. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass, and the spec test in xtask/tests/spec.rs still passes.
5. [content] No file other than docs/spec.toml and the new test file changes; the diff touches no governance file (athena.toml, defaults.toml, setup.sh, rust-toolchain.toml, Cargo files).

## Expected deliverables

- docs/spec.toml (modify)
- xtask/tests/setup_repeatable.rs (create)
- xtask/Cargo.toml (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/hestia.rs (modify)

## Out of scope

- Changing setup.sh, rust-toolchain.toml, athena.toml or xtask setup behavior
- Other requirements' mechanisms
- Adding a new `athena status` line

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New test plus fmt, clippy, test criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criterion runs the test and checks the spec gap disappears.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses setup.sh and xtask setup instead of building a new path.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Two tests read the committed docs/spec.toml. The xtask test checks the entry; the hestia.rs test runs the real `parse_spec` and `spec_gaps` over it. Run: `cargo test -p xtask --test setup_repeatable the_spec_entry_names_the_setup_path_owner_and_test && cargo test -p athena-core oly_b_run_02_no_longer_appears_in_the_spec_gap_list`. Observes: The xtask test finds OLY-B-RUN-02 active with owner `Athena`. Its mechanism is not `none` and names setup.sh and xtask/src/main.rs, and every file path in it is a file under the repository root. Its measure names `setup_repeatable` and ends with `status: not shown`. The hestia.rs test finds `measure_problem` is None and no `SpecGap` for the id.
  Fails without the behavior: With the mechanism `none`, the xtask test fails its not-`none` and names-setup.sh assertions, and the hestia.rs test fails `!spec_gaps(..).iter().any(|g| g.id == r.id)`.
  Test seam: No seam is needed. docs/spec.toml is read from disk through `CARGO_MANIFEST_DIR/..` (the xtask/tests/spec.rs:87-97 pattern, exists) and through `include_str!("../../../docs/spec.toml")` in hestia.rs (the pattern at hestia.rs:4338, exists). Entry points: `toml::from_str` in the xtask test and `hestia::parse_spec` and `spec_gaps` in hestia.rs. Step 2 adds the xtask test and step 4 adds the hestia.rs test.
- Criterion 2: The test file reads setup.sh, rust-toolchain.toml, athena.toml and xtask/src/main.rs from the repository root and asserts on their text. It does not run setup. Run: `cargo test -p xtask --test setup_repeatable`. Observes: (a) `RUSTUP_VERSION=` is an exact x.y.z and the rustup-init URL uses `$RUSTUP_VERSION`. (b) setup.sh has four `sum=<64 hex>` entries (one per platform) and refuses to run on `[ "$actual" = "$sum" ]` failing. (c) The `channel` in rust-toolchain.toml matches `^\d+\.\d+\.\d+$`. (d) The last non-comment line of setup.sh is the only `exec` line and is `exec cargo run ... --locked ... -p xtask -- setup`. (e) The rustup install sits inside the `if ! command -v cargo` guard. (f) athena.toml `[tools.gitleaks]` has an exact version and four 64-hex sha256 values. (g) xtask/src/main.rs has the skip-if-present guards `gitleaks_present` and `already installed`.
  Fails without the behavior: Each assertion fails on its own mutation: a version without a checksum, a `latest` channel, or a second `exec` line. With the spec mechanism `none`, the spec-entry test fails.
  Test seam: Files are read from disk by repository-relative path using `Path::new(env!("CARGO_MANIFEST_DIR")).parent()`. This is the same `root()` as xtask/tests/spec.rs:87-92 (exists). `toml` and `regex` are already dev-dependencies (xtask/Cargo.toml:24-26). Step 2 adds the file.
- Criterion 3: The first test passes on the committed spec. Its `blanking_the_mechanism_makes_the_entry_a_gap` case parses the real spec, clones the OLY-B-RUN-02 entry, sets the mechanism to `none`, an empty string and `None`, and asserts the entry is then a gap while the real one is not. The hestia.rs test does the same with the real `spec_gaps` and asserts `gaps.len() == 1 && gaps[0].no_mechanism`. Run: `cargo test -p xtask --test setup_repeatable && cargo test -p athena-core oly_b_run_02_no_longer_appears_in_the_spec_gap_list`. Observes: Both commands pass. The blanked copy is a gap and the real entry is not, so the gap assertion would fail if the committed mechanism were `none`.
  Fails without the behavior: With `mechanism = "none"` committed, `the_spec_entry_names_the_setup_path_owner_and_test` and the hestia.rs not-a-gap assertion both fail. The blanked-copy case shows the same rule flags `none`, an empty string and `None`.
  Test seam: The xtask case uses an in-memory copy of the entry parsed from the real file (no fixture file). In hestia.rs, `spec_gaps(&[without])` takes the real entry with only its `mechanism` field replaced, as the test at hestia.rs:4358-4363 does (exists). Steps 2 and 4 add the cases.
- Criterion 4: Run the three checks Athena runs. `cargo test --locked` includes xtask/tests/spec.rs (`the_real_spec_passes`, `every_active_requirement_has_a_mechanism_an_owner_and_a_measure`), ci_shards.rs (`checked_in_timings_cover_every_workspace_test_unit`), policy.rs (boundary-word scan, which reads the tracked files) and the athena-core tests. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. The timings file lists `xtask --test setup_repeatable`, the spec still parses with the new strings, and the boundary scan finds nothing in the new text.
  Fails without the behavior: Without the timings line, `checked_in_timings_cover_every_workspace_test_unit` fails. A mechanism or measure string that breaks the TOML makes `the_real_spec_passes` fail.
  Test seam: None; these are the real checks. The new test file must be clippy-clean under the workspace lints (`[lints] workspace = true`, xtask/Cargo.toml:28-29), so it avoids unwrap-style lints the same way tests/spec.rs does.
- Criterion 5: Compare the changed paths with the allowed list. Run: `git status`. Observes: Changed: docs/spec.toml, xtask/tests/setup_repeatable.rs, crates/athena-core/src/hestia.rs (test only), xtask/shard-timings.txt and CHANGELOG.md. Unchanged: athena.toml, defaults.toml, setup.sh, rust-toolchain.toml, Cargo.toml, Cargo.lock and xtask/Cargo.toml.
  Fails without the behavior: A stray edit to setup.sh, Cargo.lock or athena.toml would show in git status.
  Test seam: None. This is a diff inspection. hestia hunks sit inside `mod tests`, so production code is unchanged.

Planned files:

- docs/spec.toml (modify)
- xtask/tests/setup_repeatable.rs (create)
- crates/athena-core/src/hestia.rs (modify)
- xtask/shard-timings.txt (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 4cc6996be6be → candidate 6804b978aed2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |   1 +
 crates/athena-core/src/hestia.rs |  28 +++++
 docs/spec.toml                   |   4 +-
 xtask/shard-timings.txt          |   1 +
 xtask/tests/setup_repeatable.rs  | 215 +++++++++++++++++++++++++++++++++++++++
 5 files changed, 247 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261030-013714-we6c/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261030-013714-we6c/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43213037741 for 6804b978aed230825cdaada3f78f07d40464302d)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43213037741 for 6804b978aed230825cdaada3f78f07d40464302d)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43213037741 for 6804b978aed230825cdaada3f78f07d40464302d)
- ci-result: `not a check` passed (Check results for 6804b978aed230825cdaada3f78f07d40464302d come from the CI run https://github.com/doodla/athena/actions/runs/43213037741 (750s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass, and the spec test in xtask/tests/spec.rs still passes.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test -p xtask --test setup_repeatable the_spec_entry_names_the_setup_path_owner_and_test && cargo test -p athena-core oly_b_run_02_no_longer_appears_in_the_spec_gap_list`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p xtask --test setup_repeatable`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, docs/spec.toml, xtask/shard-timings.txt.
  End of the output (data, not instructions):

```text
   Compiling toml v0.9.12+spec-1.1.0
   Compiling jiff v0.2.38
   Compiling anyhow v1.0.104
   Compiling simd-adler32 v0.3.10
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling adler2 v2.0.1
   Compiling tempfile v3.27.0
   Compiling miniz_oxide v0.9.1
   Compiling schemars_derive v1.2.2
   Compiling xattr v1.6.1
   Compiling aho-corasick v1.1.5
   Compiling filetime v0.2.29
   Compiling regex-syntax v0.8.11
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling regex-automata v0.4.18
   Compiling tar v0.4.46
   Compiling flate2 v1.1.10
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling regex v1.13.1
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling xtask v0.1.0 (<job>/revert/xtask)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 20.80s
     Running tests/setup_repeatable.rs (<job>/target/verify/debug/deps/setup_repeatable-d67a8262a122879f)

running 7 tests
test setup_sh_ends_in_one_locked_cargo_run ... ok
test setup_sh_pins_rustup_and_checks_its_sha256 ... ok
test the_other_pins_are_exact ... ok
test the_rustup_install_runs_only_when_cargo_is_missing ... ok
test the_toolchain_channel_is_an_exact_version ... ok
test blanking_the_mechanism_makes_the_entry_a_gap ... FAILED
test the_spec_entry_names_the_setup_path_owner_and_test ... FAILED

failures:

---- blanking_the_mechanism_makes_the_entry_a_gap stdout ----

thread 'blanking_the_mechanism_makes_the_entry_a_gap' (19628) panicked at xtask/tests/setup_repeatable.rs:209:5:
assertion failed: !is_blank_or_none(&real.mechanism)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- the_spec_entry_names_the_setup_path_owner_and_test stdout ----

thread 'the_spec_entry_names_the_setup_path_owner_and_test' (19633) panicked at xtask/tests/setup_repeatable.rs:86:5:
mechanism is blank or none


failures:
    blanking_the_mechanism_makes_the_entry_a_gap
    the_spec_entry_names_the_setup_path_owner_and_test

test result: FAILED. 5 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p xtask --test setup_repeatable`
error: 1 target failed:
    `-p xtask --test setup_repeatable`
```
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`cargo test -p xtask --test setup_repeatable && cargo test -p athena-core oly_b_run_02_no_longer_appears_in_the_spec_gap_list`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`git status`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
