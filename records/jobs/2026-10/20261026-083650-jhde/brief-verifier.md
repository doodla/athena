# Job 20261026-083650-jhde: verify "Make tests independent of real free disk by setting the minimum to 0"

## Request (verbatim)

> Priority: urgent
> The disk guard (9102f8f, default minimum 10 GB free) makes tests depend on the machine's real free space: athena-cli tests/binary.rs a_nested_run_works_in_a_scratch_copy_with_fake_agents failed in job 20261026-064950-38jx with 'environment: low disk: 9.0 GB free, 10.0 GB required', because the job's own build had used about 19 GB of the session's allowance. Tests must not depend on real free disk: let the minimum be set through the test fixtures' config (for example 0 in test configs) or inject the free-space reading, so the guard is still tested with simulated values and nested runs in tests are unaffected.
> Acceptance:
> - The nested-run test passes with free disk below 10 GB (shown by a test that simulates low free space for the nested run's environment and still passes because the test config sets the minimum to 0).
> - The disk guard's own tests still show a hold and a fail-fast with simulated low space.
> Files: crates/athena-core/src/disk.rs, crates/athena-cli/tests/common/mod.rs, crates/athena-cli/tests/binary.rs, CHANGELOG.md

## Reading (accepted by the user)

The shared test config (test_config in tests/common/mod.rs) gets [disk] min_free_gb = 0, so CLI tests, including the nested-run test, no longer depend on the machine's free space. A new test makes the nested run see low free space (a minimum above any real free space is not simulated; the reading is injected or the guard is exercised with simulated values) and shows it still passes. The disk guard's own tests keep showing a hold and a fail-fast with simulated values. The audience is the maintainers and verifier running cargo test on small-disk sandboxes.

Assumptions:
- Fixture::new writes test_config() as the fixture repo's athena.toml, and the nested scratch copy inherits it, so setting [disk] min_free_gb = 0 there reaches the nested run.
- Drain tests that exercise the hold already set their own minimum explicitly; they must keep doing so after the default becomes 0 (drain.rs may need a small edit if they rely on the default 10).
- Simulating low free space for the nested run is done by an injectable free-space reading in disk.rs (e.g. an env override read only when set) or equivalent; the production default of 10 GB is unchanged.
- athena.toml is not changed: the minimum is set in the test config only.

## Acceptance criteria

1. [behavior] Run `cargo test --locked -p athena-cli --test binary`: a_nested_run_works_in_a_scratch_copy_with_fake_agents passes, and a companion test simulates low free space (below 10 GB) in the nested run's environment and still passes because the test config sets min_free_gb to 0.
2. [behavior] Run `cargo test --locked -p athena-core disk`: the disk guard's tests still show a hold (job stays queued, 'environment: low disk') and a fail-fast with simulated low free space.
3. [content] crates/athena-cli/tests/common/mod.rs test_config sets [disk] min_free_gb = 0 with a comment explaining why, and the production default (10 GB in athena-data config) is unchanged.
4. [behavior] Run `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check`: all pass, and no test in the suite reads the machine's real free disk to decide its result.
5. [content] CHANGELOG.md has an entry stating that tests no longer depend on real free disk space and how the minimum or free-space reading is set in tests.

## Expected deliverables

- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Out of scope

- Changing the production default minimum of 10 GB
- The code-health findings from job 20261026-064950-38jx (hooks test file, baseline file, table header width)
- Changing athena.toml or defaults.toml

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass; the guard stays tested with simulated values.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the nested-run test and the disk tests rather than checking that config lines exist.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The shared test config is fixed, not just the one failing test.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands the verifier runs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change limited to the four listed files plus drain.rs only if needed.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the binary test file. a_nested_run_works_in_a_scratch_copy_with_fake_agents passes; the new companion test runs the same nested ask/go with ATHENA_FREE_BYTES=9000000000 and passes; a control with min_free_gb = 10 and the same env fails with the low-disk text. Run: `cargo test --locked -p athena-cli --test binary`. Observes: All tests in binary pass; the companion shows exit 0 and the landing commit under 9 GB simulated; the control run exits non-zero mentioning 'environment: low disk: 9.0 GB free, 10.0 GB required'.
  Fails without the behavior: Without min_free_gb = 0 in test_config the companion test fails with 'environment: low disk: 9.0 GB free, 10.0 GB required'; without the ATHENA_FREE_BYTES override the simulation has no effect and the negative-control assertion (min 10 must fail with low disk) fails.
- Criterion 2: Run core disk unit tests (including the new override tests); also run the drain tests that assert hold and fail-fast with simulated 3 GB: cargo test --locked -p athena-cli --test drain low_disk, and cargo test --locked -p athena-cli --test drain the_real_steps_hold. Run: `cargo test --locked -p athena-core disk`. Observes: Core disk tests pass; the drain hold test shows the job Proposed with 'environment: low disk: 3.0 GB free, 10.0 GB required'; the before-checks test shows a Failed job with a retry hint.
  Fails without the behavior: If the guard stopped comparing against the minimum, the hold test (queued, 3.0 GB free, 10.0 GB required) and the fail-fast test would fail; the new disk.rs unit tests fail if the override is ignored or garbage is not ignored.
- Criterion 3: Read common/mod.rs test_config for the [disk] min_free_gb = 0 block and its comment; run the athena-data config tests, which still assert min_free_gb() == 10 for an absent [disk] table; git diff shows no change to athena-data. Run: `cargo test --locked -p athena-data config`. Observes: Config tests pass with default 10; the diff touches only the listed files.
- Criterion 4: Run cargo test --locked, cargo clippy --all-targets --locked -- -D warnings, and cargo fmt --all --check. CLI tests use min 0 or simulated readings. Run: `cargo test --locked`. Observes: All three commands exit 0.
  Fails without the behavior: A test that still read real df with a nonzero minimum would flip with machine space; none remain (the_real_reader test only asserts > 0).
- Criterion 5: Inspect the diff: an entry under [Unreleased] stating tests no longer depend on real free disk, that test_config sets [disk] min_free_gb = 0, and that ATHENA_FREE_BYTES simulates the reading. Run: `git diff CHANGELOG.md`. Observes: The new bullet is present in the Unreleased section.

Planned files:

- crates/athena-core/src/disk.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)

## The change

Base 06140253a36c → candidate 6b8ee02ada93. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                          |  6 +++
 crates/athena-cli/tests/binary.rs     | 80 +++++++++++++++++++++++++++++------
 crates/athena-cli/tests/common/mod.rs | 16 ++++++-
 crates/athena-cli/tests/drain.rs      | 16 ++++---
 crates/athena-core/src/disk.rs        | 24 ++++++++++-
 5 files changed, 121 insertions(+), 21 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1282 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check`: all pass, and no test in the suite reads the machine's real free disk to decide its result.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
