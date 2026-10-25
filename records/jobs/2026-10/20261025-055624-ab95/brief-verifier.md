# Job 20261025-055624-ab95: verify "Skip 'fmt, clippy, tests' CI when the governed fingerprint already"

## Request (verbatim)

> Priority: high
> Speed first, cut A part 1 (job 20261025-034731-2am0 timed out as one job): make the 'fmt, clippy, tests' CI job reuse its own earlier pass for unchanged code. Add an xtask command that prints the governed-content fingerprint of the checked-out tree (reusing athena-gate's fingerprint, which ignores records/ and owner paths). In .github/workflows/ci.yml, before setup, look the fingerprint up in actions/cache; on a hit of a passing entry, report it and skip the build and checks; otherwise run the full job and save the entry only after it passes. GitHub scopes caches by branch, so entries saved on a pull-request branch serve later runs on that branch, and main's shared caches stay written only from main. Keep the job name 'fmt, clippy, tests'.
> Acceptance:
> - An xtask test shows the fingerprint unchanged by a records-only or owner-path change and changed by a governed change.
> - The workflow saves an entry only after a full pass, and a missing or failed entry runs the full job (shown by a test of the step script or the workflow logic).
> - docs/decisions.md records the mechanism, the measured landing median before (15.4 minutes over 33 jobs, 2026-10-23 to 25) and the alternatives.
> Files: xtask/src/main.rs, .github/workflows/ci.yml, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Add `xtask fingerprint` (printing athena-git's governed-content fingerprint of HEAD) and make the 'fmt, clippy, tests' job look that fingerprint up in actions/cache before setup. A hit skips setup and checks; a miss runs everything and saves a marker only after a pass. Audience: the owner and the verifier judging CI speed and soundness.

Assumptions:
- Part A1 only: no change to athena land or gh.rs and no early push of the candidate; the after-measurement is deferred.
- The fingerprint is computed from the checked-out HEAD with rules from that same commit, as athena-git::attrs::fingerprint(git, HEAD, HEAD) does.
- xtask currently depends only on athena-data and policy forbids other edges; reusing athena-git needs an xtask->athena-git edge, so docs/architecture.md and xtask/Cargo.toml are updated to allow it. This is the one governance-adjacent change.
- The step script logic is testable by putting it in xtask (e.g. a subcommand deciding hit/miss and writing the marker) so the workflow only calls it.
- Cache key is like ci-pass-<fingerprint>; GitHub's branch scoping gives the described PR/main behavior, and the pass entry is saved by a step that runs only on success (not default-branch-only).
- The workflow runs xtask with the runner's preinstalled cargo before setup.sh; no change to athena.toml or defaults.toml.

## Acceptance criteria

1. [behavior] Run `cargo test -p xtask`: a test builds a temporary git repo and shows the printed fingerprint is unchanged by a records-only change and by an owner-path change, and changes after a governed change.
2. [behavior] Run `cargo test -p xtask`: a test of the step script/workflow logic shows a missing entry or a failed run leads to the full job and no save, a hit of a passing entry skips the build and checks, and the save happens only after the checks pass.
3. [content] .github/workflows/ci.yml keeps the job name 'fmt, clippy, tests'; the cache lookup (actions/cache/restore, key from the xtask fingerprint) comes before 'Set up'; setup, the checks and the existing cache steps are skipped on a hit; the new save step runs only when the checks succeeded.
4. [content] Saving is done on pull-request branches for their own fingerprint entries, while the shared toolchain/registry/target caches are still saved only from the default branch; the entry is written under a key that includes the fingerprint and a marker whose presence means 'passed' (a failed run never writes one).
5. [content] The hit step reports the reused result in the log and step summary (fingerprint and matched key), so a reader can see why the build was skipped.
6. [content] The fingerprint comes from athena-git's `fingerprint` (not a reimplementation); any dependency or architecture-doc change this needs keeps `dependency_direction_matches_the_architecture` passing.
7. [content] docs/decisions.md has an entry giving the mechanism, the measured landing median before (15.4 minutes over 33 jobs, 2026-10-23 to 25), the alternatives considered, and that the after-figure is not yet measured; CHANGELOG.md has a matching entry.
8. [behavior] Run `cargo test` and `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: all pass.

## Expected deliverables

- xtask/src/main.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- docs/architecture.md (modify)
- xtask/tests/fingerprint.rs (create)
- xtask/tests/policy.rs (modify)

## Out of scope

- Changing athena land or crates/athena-core/src/gh.rs, or pushing the candidate early for CI.
- Caching for the 'the full history' job or other jobs.
- Measuring the landing time after the change.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Fingerprint command, script logic and tests are all Rust in xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for the fingerprint and the hit/miss/save logic; cargo test and clippy criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing reuse and rerun behavior, not just wiring.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: decisions.md cites the measured 15.4 min over 33 jobs and labels the after-figure unmeasured.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Uses actions/cache and the existing athena-git fingerprint.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only part A1; early push is left out.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Pass-only save is enforced by step conditions and tested.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that unchanged governed content skips the job while governed changes run it.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration test xtask/tests/fingerprint.rs builds a temp git repo (with the attribute rules the repo uses for owner paths, and a records/ dir), runs the built xtask binary `fingerprint` in it via CARGO_BIN_EXE_xtask, and compares outputs: equal after adding a records/ file, equal after changing an owner-path file, different after changing a governed file. Run: `cargo test -p xtask --test fingerprint`. Observes: Test names such as fingerprint_ignores_records_and_owner_paths and fingerprint_changes_with_governed_content pass.
  Fails without the behavior: If the command hashed the whole tree, or records/owner changes leaked in, the equality asserts fail; if governed changes were ignored, the inequality assert fails.
- Criterion 2: Tests of `ci-cache` in the same file/unit tests: empty or missing dir -> skip=false; marker with another fingerprint -> false; marker with the right fingerprint -> true; `mark --outcome failure` (or cancelled) writes no marker, `--outcome success` writes one that `check` then accepts; and a sequence test: miss -> run -> failed outcome -> still miss; success -> hit. A policy.rs test parses ci.yml and asserts the save step has no condition beyond success, comes after the mark step, which comes after the checks step. Run: `cargo test -p xtask`. Observes: All ci_cache_* tests and the workflow-order test pass.
  Fails without the behavior: If mark wrote on failure, or check skipped on a missing/foreign marker, the corresponding asserts fail.
- Criterion 3: A policy.rs test reads ci.yml: job name is 'fmt, clippy, tests'; index of the cache/restore step with key ci-pass- precedes 'Set up'; the existing restores, report, setup, checks and shared saves carry `steps.gate.outputs.skip != 'true'`; the pass-save step follows the checks step. Run: `cargo test -p xtask --test policy`. Observes: The new workflow-structure test passes.
  Fails without the behavior: Removing a skip condition or reordering the lookup after setup fails the test.
- Criterion 4: The same policy.rs test asserts the pass-save step has no default-branch condition and a key starting ci-pass- plus the fingerprint output, that the three shared saves still contain the default-branch condition, and that the marker is written only by `ci-cache mark` after checks (covered by criterion 2 tests). Run: `cargo test -p xtask --test policy`. Observes: Assertions on keys and conditions pass.
  Fails without the behavior: A save step gated to the default branch, or shared saves opened to PRs, fails the assertions.
- Criterion 5: A test of `ci-cache check`/a report subcommand output: on a hit it prints and appends to the summary file (path from an argument) the fingerprint and matched key; the policy test checks the hit-report step passes the matched-key output and runs only on skip. Run: `cargo test -p xtask`. Observes: Output contains the fingerprint and key; the summary file has them.
  Fails without the behavior: If the report omitted the fingerprint or key, the contains asserts fail.
- Criterion 6: Inspect with cargo tree that xtask depends on athena-git, and run the existing graph test; a grep-free check is that main.rs calls athena_git::attrs::fingerprint. Run: `cargo tree -p xtask`. Observes: athena-git appears under xtask; dependency_direction_matches_the_architecture passes in cargo test -p xtask --test policy.
  Fails without the behavior: Without the architecture.md row update, dependency_direction_matches_the_architecture fails on the undocumented xtask -> git edge.
- Criterion 7: Read the new docs/decisions.md and CHANGELOG.md entries; the existing policy tests that check decisions and changelog format must still pass. Run: `cargo test -p xtask --test policy`. Observes: Entries state the mechanism, 15.4 minutes over 33 jobs (2026-10-23 to 25), the alternatives, and 'after-figure not yet measured'; format tests pass.
- Criterion 8: Run the full gates. Run: `cargo test`. Observes: All pass; also cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean.
  Fails without the behavior: Any new warning, format drift or failing test fails the run.

Planned files:

- xtask/src/main.rs (modify)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- docs/architecture.md (modify)
- .github/workflows/ci.yml (modify)
- xtask/tests/fingerprint.rs (create)
- xtask/tests/policy.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## The change

Base 96240cb59548 → candidate d48e613703f2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 .github/workflows/ci.yml   |  67 +++++++++++-
 CHANGELOG.md               |   9 ++
 Cargo.lock                 |   1 +
 docs/architecture.md       |   2 +-
 docs/decisions.md          |  44 ++++++++
 xtask/Cargo.toml           |   1 +
 xtask/src/ci_cache.rs      | 113 ++++++++++++++++++++
 xtask/src/main.rs          |  10 +-
 xtask/tests/fingerprint.rs | 256 +++++++++++++++++++++++++++++++++++++++++++++
 xtask/tests/policy.rs      | 116 +++++++++++++++++++-
 10 files changed, 611 insertions(+), 8 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1199 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
