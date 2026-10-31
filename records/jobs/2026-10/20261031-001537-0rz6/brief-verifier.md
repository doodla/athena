# Job 20261031-001537-0rz6: verify "Split CI into parallel fmt, clippy and sharded test jobs with xtask"

## Request (verbatim)

> Priority: urgent
> Split CI's test run across many GitHub runners so a full check of one commit takes minutes, not 16
> 
> The `fmt, clippy, tests` job in .github/workflows/ci.yml runs everything on one runner, about 16 minutes on a cold cache (docs/decisions.md D125). GitHub runners are free and parallel, so use many of them. This is the first step of moving Athena's checks off the local machine (docs/design/ci-routing.md); the request to read the result back into a job is separate.
> 
> Change ci.yml so that fmt, clippy and the tests are separate jobs that run at the same time, and the tests are split into a matrix of shards (start with 6) that together run every test exactly once. Keep the existing cache layers, the fingerprint pass marker (D121) and the SHA pinning of actions. Give every job a timeout-minutes. Add one final job named `ci` that needs all the others and fails if any failed, so a single check name still says whether a commit is green.
> 
> The producer cannot run GitHub Actions. Acceptance is checked from the files and local tests only; the real run time is measured later by a throughput review.
> 
> Acceptance:
> - ci.yml runs fmt, clippy and the test shards as separate jobs, plus a final `ci` job that needs them all; every job has timeout-minutes.
> - An xtask test (or a test in an existing crate) reads ci.yml and fails if the shard list is not a partition: the test command each shard runs, taken together, covers every workspace test target exactly once.
> - The shard split is done by a command the repository owns (an xtask subcommand), so the same split can be run locally with `cargo run -p xtask -- test-shard <k> <n>`; running all n shards locally runs the same tests as `cargo test --workspace`.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: .github/workflows/ci.yml, xtask/, CHANGELOG.md

## Reading (accepted by the user)

Replace the single `checks` job in ci.yml with parallel fmt, clippy and a 6-shard test matrix, plus a final `ci` gate job. Add an `xtask test-shard <k> <n>` subcommand that partitions workspace test targets, and a test that reads ci.yml and checks the shards form an exact partition. The audience is Athena's maintainers, who need a fast single required check.

Assumptions:
- Code survey: read .github/workflows/ci.yml and xtask/src/main.rs (entry `main` dispatches subcommands; `ci()` at ~line 536 runs fmt --all --check, clippy --all-targets --locked -D warnings, then `cargo test --locked` in sequence). xtask/src/ci_cache.rs holds fingerprint and ci-cache check/mark/report. No record or data file is added under crates/, so check_record_file and AnyRecord are not touched; xtask/tests/ already holds integration tests.
- The D121 pass marker stays: the fingerprint and gate steps run once in the fmt/clippy/test jobs' shared prelude (or a small first job whose outputs the others need); the marker is written only by the `ci` job or a job that needs all others, after all succeeded, so a failed shard never leaves a marker. Producer picks the least invasive design and documents it in comments.
- Sharding unit is the workspace test target (each lib, bin, integration test file), enumerated deterministically by xtask (e.g. cargo metadata), assigned by index modulo n; shard k runs `cargo test --locked -p <pkg> --test <name>` / `--lib` for its targets, including doctests assigned to exactly one shard. All n shards together equal `cargo test --workspace`.
- Cache layers keep their keys; each job gets its own target cache key (target-fmt/clippy/test) to avoid collisions, with restore-only on every event and save on default branch only. Existing history, attribution and pr-head jobs are unchanged apart from nothing.
- The `ci` gate uses `if: always()` and fails unless every needed job result is `success`; the old `checks` name is replaced by `ci` as the required check (branch protection is outside the repo).

## Acceptance criteria

1. [content] Grep of .github/workflows/ci.yml shows separate jobs for fmt, clippy and a test job with `strategy.matrix` of 6 shards, plus a job with `name: ci` whose `needs` lists fmt, clippy and the test job and which uses `if: always()` and fails when any needed result is not success; every job has `timeout-minutes`.
2. [content] All `uses:` lines in ci.yml remain pinned by 40-hex commit SHA, the three cache layers (toolchain, registry, target) and the D121 fingerprint/ci-pass lookup, gate and mark steps are still present, and the pass marker is saved only after all checks succeeded.
3. [behavior] An xtask test reads .github/workflows/ci.yml, extracts each shard's test command and the shard count, and fails when the shards do not partition the workspace test targets exactly once; run `cargo test -p xtask` and see it pass, and a negative case inside the test (a mutated shard list with a dropped or duplicated shard) is shown to fail the check.
4. [behavior] `cargo run -p xtask -- test-shard <k> <n>` exists; for n=6 and k in 0..5 (or the documented index base) the selected targets are disjoint and their union equals the target set of `cargo test --workspace`; a test in xtask asserts this, and invalid k/n (k>=n, n=0) exits with an error. Run `cargo test -p xtask`.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.
6. [content] CHANGELOG.md has an entry describing the parallel jobs, the shard matrix, the `ci` gate job and `xtask test-shard`.
7. [content] Outcome check: the real run time is not claimed; comments or the changelog state it is to be measured later by the throughput review, and no number is presented as measured.

## Expected deliverables

- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- xtask/src/main.rs (modify)
- xtask/src/shard.rs (create)
- xtask/tests/ci_shards.rs (create)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- CHANGELOG.md (modify)
- docs/design/ci-routing.md (modify)

## Out of scope

- Reading CI results back into a job (a separate request)
- Measuring real GitHub run time or changing branch protection settings
- Changing the history, attribution and pr-head jobs
- Changes to athena.toml, defaults.toml, setup.sh or rust-toolchain.toml
- Tests that call real models

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The shard command and its test are Rust in xtask.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New xtask tests; cargo test and clippy -D warnings must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The partition rule is enforced by a test that reads ci.yml.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the partition and the shard command by running tests.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion states the run time is not claimed and is measured later.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to ci.yml plus xtask.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Use cargo metadata and the existing test runner rather than adding a sharding tool; record the call in a comment.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Workflow comments and changelog written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Tests `workflow_jobs_*` in xtask/tests/ci_shards.rs read the real ci.yml with std::fs, split it into job blocks (two-space-indent keys under `jobs:`) and assert: jobs fmt, clippy, test exist; test has strategy.matrix.shard of 6 entries; the job with `name: ci` has needs containing gate, fmt, clippy, test, has `if: always()`, and its run step exits 1 unless each need is success (or skipped when gate.outputs.skip == 'true'); every job block has timeout-minutes. Text mutations (remove `if: always()`, drop one needs entry, strip one timeout-minutes) are asserted to be rejected. Run: `cargo test -p xtask --test ci_shards -- workflow_jobs`. Observes: Tests pass on the new ci.yml; each mutated copy is rejected with a message naming the missing item.
  Fails without the behavior: If ci.yml lacked the separate jobs, the 6-shard matrix, the ci job's needs / if: always() / result check, or any job's timeout-minutes, one of these tests fails.
  Test seam: Reads the real .github/workflows/ci.yml via std::fs with line regexes over job blocks; no double; same style as existing checks_job_steps in xtask/tests/policy.rs (exists). The new test file is added by plan step 6.
- Criterion 2: Re-pointed D121 tests in policy.rs use job_steps(job_id). In gate: fingerprint < lookup < decide, lookup is cache/restore keyed by the fingerprint. Each compute job has `needs: gate` and the skip condition, and restores toolchain, registry and target caches. `pass` needs gate, fmt, clippy, test, has no always/failure/cancelled, runs ci-cache mark then cache/save of ci-pass with no default-branch condition. Shared-cache saves stay default-branch-only. In ci_shards.rs, `workflow_actions_are_pinned` asserts every `uses:` matches @[0-9a-f]{40}, and `workflow_caches` asserts the three cache restores in fmt, clippy and test. Run: `cargo test -p xtask --test policy && cargo test -p xtask --test ci_shards -- workflow_actions workflow_caches`. Observes: All pass; existing policy assertions are kept, moved to the job that now holds each step.
  Fails without the behavior: An unpinned uses, a removed cache layer, dropped gate/lookup/mark steps, or always() / a default-branch condition on the mark or save step makes the pinned-actions test or the re-pointed policy tests fail.
  Test seam: Real ci.yml read via the helper in policy.rs, generalised from checks_job_steps() (exists) to job_steps(job_id) (step 7). workflow_actions_are_pinned and workflow_caches are added in step 6.
- Criterion 3: Test `shards_in_ci_yml_partition_every_workspace_test_target` calls check_partition(real ci.yml text): parse the matrix list and n, run the built `xtask test-shard k n --dry-run` for each listed k, then check_cover against the independent cargo-metadata oracle; expect Ok. Negatives, each asserted Err: (a) shape mutations of the ci.yml text (drop shard 5 with n=6; duplicate a shard; change the command's n to 5), each asserted to differ from the original; (b) cover-only cases calling check_cover directly with real per-shard unit sets: a unit removed (error names it missing), a unit copied into a second shard (error names it duplicated), an oracle with one extra synthetic unit, a shard unit absent from the oracle. A positive control checks check_cover returns Ok for the unmodified sets. Run: `cargo test -p xtask --test ci_shards -- partition`. Observes: The positive test passes on the repository's ci.yml; each negative passes because it is rejected, the cover-only ones by check_cover with the unit named.
  Fails without the behavior: If the cover comparison always returned Ok, the cover-only negatives fail; if the shape rule were missing, the text-mutation cases fail; if the real shards missed or repeated a unit, the positive test fails.
  Test seam: Production entry point: the built xtask binary, `test-shard <k> <n> --dry-run`, via env!("CARGO_BIN_EXE_xtask") (pattern in xtask/tests/fingerprint.rs, exists). Fixtures: the real ci.yml, in-memory mutated strings, and shard/oracle sets passed as arguments to the pure check_cover (step 6). The oracle is the test's own cargo metadata call plus the by-path filesystem cross-check, with no logic imported from shard.rs. The subcommand and --dry-run are added by steps 2-4.
- Criterion 4: In ci_shards.rs, `test_shard_6_is_disjoint_and_equals_the_workspace_target_set` runs test-shard k 6 --dry-run for k in 0..=5 and asserts the unit lines are pairwise disjoint and their union equals the oracle; `test_shard_1_equals_the_workspace` does the same for n=1; `oracle_matches_files_on_disk` maps every src/lib.rs, src/main.rs, src/bin/*.rs and tests/*.rs (and */main.rs) of each member to a metadata target by src_path and requires its unit in the oracle (passes for athena-cli's renamed bin `athena`); `test_shard_rejects_bad_arguments` runs `6 6`, `0 0`, `3 2`, `x 6` and no arguments and asserts non-zero exit and an error message. Unit tests in shard.rs feed a hand-written metadata JSON to plan_units: examples and benches excluded; bin/test targets get --bin/--test; a lib with doctest=false has no --doc unit; select over n=1, 6 and n > unit count is disjoint and complete. Run: `cargo test -p xtask`. Observes: All xtask tests pass, including existing ones. `cargo run -p xtask -- test-shard 0 6 --dry-run` prints a stable sorted list, and the six shards together equal the workspace target set.
  Fails without the behavior: If test-shard were missing, overlapped, dropped any unit (for example doctests), or accepted k>=n or n=0, the disjoint/union and bad-argument tests fail. If a target were renamed in a way the oracle mishandles, oracle_matches_files_on_disk fails.
  Test seam: Production entry point: the xtask binary (main dispatch to shard::command) run via CARGO_BIN_EXE_xtask against the real workspace. For unit tests, shard::plan_units(&serde_json::Value) takes the metadata JSON as an argument so a fixture is injected. Both added by steps 2-4; integration tests by step 6.
- Criterion 5: Run the three commands Athena runs, in order. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0; cargo test also runs the new ci_shards tests and the re-pointed policy tests.
  Fails without the behavior: A formatting difference, a clippy warning in shard.rs or the new tests, or a failing test (including the re-pointed policy tests) makes a command exit non-zero.
  Test seam: The repository's own commands; no double.
- Criterion 6: Read the diff. An entry under `## [Unreleased]` names: the separate fmt, clippy and test jobs; the 6-shard matrix; the final `ci` gate job and its handling of a reused pass; the `pass` job that writes the D121 marker; `cargo run -p xtask -- test-shard <k> <n>` with --dry-run; the ci_shards partition test; the old check name `fmt, clippy, tests` that no longer exists (branch protection needs `ci`); the known EARLY_RUN_JOB effect; the partial test-cache known effect. Run: `git diff -- CHANGELOG.md`. Observes: The entry exists and mentions each item.
  Fails without the behavior: An empty CHANGELOG.md diff, or an entry missing one of the named items.
- Criterion 7: Read the diff. The ci.yml header comment, the CHANGELOG entry and the ci-routing.md section each say the run time of the new layout is not measured and is to be measured later by the throughput review. The only numbers quoted are the D125 baseline (about 16 minutes, labelled as the earlier measurement) and the timeout ceilings (labelled as ceilings). Run: `git diff -- .github/workflows/ci.yml CHANGELOG.md docs/design/ci-routing.md`. Observes: No new timing is claimed; the throughput review is named as the later measure.
  Fails without the behavior: Any text presenting a measured run time for the new layout would show as a minutes figure without a source in the diff.

Planned files:

- .github/workflows/ci.yml (modify)
- xtask/src/shard.rs (create)
- xtask/src/main.rs (modify)
- xtask/src/ci_cache.rs (modify)
- xtask/tests/fingerprint.rs (modify)
- xtask/tests/ci_shards.rs (create)
- xtask/tests/policy.rs (modify)
- xtask/Cargo.toml (modify)
- Cargo.lock (modify)
- CHANGELOG.md (modify)
- docs/design/ci-routing.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base e10e7fdfa0d0 → candidate ae0e76ec2c01. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 .github/workflows/ci.yml   | 347 ++++++++++++++++++++++---
 CHANGELOG.md               |  24 ++
 docs/design/ci-routing.md  |  49 +++-
 xtask/Cargo.toml           |   2 +-
 xtask/src/ci_cache.rs      |   5 +-
 xtask/src/main.rs          |  11 +-
 xtask/src/shard.rs         | 287 +++++++++++++++++++++
 xtask/tests/ci_shards.rs   | 615 +++++++++++++++++++++++++++++++++++++++++++++
 xtask/tests/fingerprint.rs |   2 +-
 xtask/tests/policy.rs      | 143 ++++++++---
 10 files changed, 1394 insertions(+), 91 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1810 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
