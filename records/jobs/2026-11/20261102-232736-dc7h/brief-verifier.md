# Job 20261102-232736-dc7h: verify "Free writer build before checks disk gate; gate only before local run"

## Request (verbatim)

> try again on 20261102-061427-anxh

## Reading (accepted by the user)

Retry of job 20261102-061427-anxh (dispatched, never verified, no findings). In check_and_verify the up-front disk_gate runs before free_producer_build, so the job's own writer build can hold it; move the gate after that removal and apply it only just before a local check run, so a CI-supplied result never triggers it. Audience: the Athena operator, who wants fewer false disk holds.

Assumptions:
- Code survey: read crates/athena-core/src/run/mod.rs. check_and_verify calls disk_gate (about line 1414) when recorded_checks is none, before push_ci_branch. free_producer_build (called about line 1463, defined in crates/athena-core/src/run/checks.rs) runs later, in the Checks stage. disk_gate is in crates/athena-core/src/disk.rs. hold_for_disk is the existing hold path. Free disk is measured directly by the existing simulated-free-space hook in the drain tests, not through a stand-in. No record or data file is added, so the records allow-list is untouched (no check_record_file or AnyRecord change). The change adds no refusal rule beyond moving and narrowing an existing one, so it refuses fewer readings; expected 0% of newly refused readings.
- Retry of job 20261102-061427-anxh: that job had no recorded findings and was not verified. Its candidate's 10 changed files are listed as deliverables (kept as a starting point and re-verified) except binary.rs, test_support.rs and run/tests.rs, which are also kept as optional deliverables because the candidate edited them for test support. docs/architecture.md is kept to describe the new gate placement.
- The hold, when it still applies on the local-run path, behaves as today: it is committed with records and no outcome, and the next drain resumes it. Reusing the producer's approach (the first local exec calling the existing disk_gate) is allowed but not required.
- Priority high is noted. The user's original constraint stands: no standalone repo-wide fmt, clippy or test acceptance criterion.

## Acceptance criteria

1. [behavior] A new test in crates/athena-cli/tests/drain.rs (using with_free_space and simulated_free) has a writer build whose size takes simulated free disk below min_free_gb. The job is not held for disk, the build directory is removed first, and the checks run to a verdict. Run `cargo test -p athena-cli --test drain` and check that it passes.
2. [behavior] A new test with a GitHub origin and a green CI run for the candidate, with free disk below min_free_gb, shows the checks stage does not hold for low disk and prints the 'ci   checks reused' line. A companion case with no usable CI result shows the disk gate still holds the job before the local check run. Run `cargo test -p athena-cli --test drain` and check that both pass.
3. [content] In crates/athena-core/src/run/mod.rs check_and_verify, the up-front disk_gate call before the checks is gone. free_producer_build runs before any disk_gate in the checks stage, and the gate is called only on the local-run path. Check by reading the diff.
4. [content] CHANGELOG.md has a new entry saying the writer build is removed before the checks free-disk check and the requirement applies only before a local check run.
5. [content] The verifier-stage gate (verifier_environment_problem) and the dispatch and producer gates are unchanged in the diff, and the existing disk-hold tests in crates/athena-cli/tests/drain.rs still pass (`cargo test -p athena-cli --test drain`).
6. [content] docs/architecture.md, if it describes the checks-stage disk check, states the new order; no other claim there changes.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- docs/architecture.md (modify)

## Out of scope

- Changing the dispatch, producer or verifier-stage disk gates
- Changing the min_free_gb default or config
- Changing CI-result resolution or routing rules
- Governance files such as athena.toml, defaults.toml and workflow files

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that the false hold no longer happens and that the local-run hold still does.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests in drain.rs cover both behaviors.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the drain tests rather than checking that wiring exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The gate order is fixed, not the single job's symptom.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the checks-stage gate.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New test `low_disk_from_the_writer_build_is_cleared_before_the_checks_gate_so_the_job_is_not_held`. Fixture is `disk_fixture()` (min_free_gb 10), built through `go_with_disk`-style wiring. The scripted Producer closure plants `<state>/repo/jobs/<spec.job>/target/produce/debug/big`, as `counting` does in run/tests.rs. The free-space reader is the existing `athena_core::test_support::low_once_built`, installed with `with_free_space`: it reports 3 GB while any `jobs/*/target/produce` exists and plenty otherwise. The test calls `athena.go(job, AcceptedBy::User, &lines)`. Run: `cargo test -p athena-cli --test drain`. Observes: The result is `GoResult::Finished`, not a LowDisk Err. `jobs/<id>/target/produce` is gone. The lines contain `disk       removed the producer's build directory` before the `checks     ` line. No line starts with `held       `.
  Fails without the behavior: With the old order, the up-front disk_gate runs while `target/produce` still exists, so it reads 3 GB and holds the job. `go` returns Err(LowDisk) and no `checks     ` line is printed, so the test fails.
  Test seam: Production entry point: `Athena::go`. Fake: `StateDir::with_free_space(test_support::low_once_built)` plus the scripted Producer closure that plants the build directory. The free-space function only reads the filesystem and holds no gate logic. Both exist today (drain.rs `go_with_disk`, test_support.rs 706). The test itself is added in step 7.
- Criterion 2: Three new tests, all using `Fixture::with_config(with_min_free_gb(&config_with_change_checks(fmt, clippy, test), 10))`. The `fmt`, `clippy` and `test` checks are trivial `sh -c` commands, so `route.reuse` is true. Free space comes from a drain.rs reader that returns 3 GB while any `target/produce` exists, or while a `verify` dir exists and `test_support::verifier_stage_reached` is false. Otherwise it returns plenty. So space is low through the writer build and the checks stage, then recovers. (a) `low_disk_with_a_green_ci_run_does_not_hold_the_checks_stage`: `git remote add origin https://github.com/owner/repo`. The new `test_support` installer puts in `EarlyFake` (remote and PR host) and a fake gh Runner that answers a green run for the candidate sha and green jobs. It uses the reply shape of `CiReplies` in run/tests.rs 2446-2530. The test calls `athena.go`. (b1) `low_disk_with_no_ci_result_holds_before_the_local_run`: no origin, so NoCi. (b2) `low_disk_with_a_github_origin_and_no_ci_run_holds_before_the_local_run`: origin, with the fake gh answering `workflow_runs: []`. Run: `cargo test -p athena-cli --test drain`. Observes: (a) `go` returns Finished with no LowDisk error. A line starts `ci         checks reused from` and a line starts `checks     `. No `held       ` line. The EarlyFake recorded a CI-branch push and nothing touched a network. (b1) and (b2) `go` returns Err downcastable to LowDisk (3 GB free, 10 GB required). No `checks     ` line. `read_hold` is Some. The job is Dispatched with no outcome. `jobs/<id>/verify` is gone. After the reader is made plentiful, `athena.retry` plus `go` reach the checks and land, as in the existing `low_disk_before_the_checks_holds_the_job_and_a_retry_with_space_runs_the_checks`.
  Fails without the behavior: (a) With the gate before the CI read, the writer build still exists at the old gate, so the job is held (LowDisk Err) and the `ci         checks reused` line never prints. (b) If the gate were dropped from the local path, the checks would run at 3 GB, `go` would not return LowDisk, and the hold assertions fail.
  Test seam: Production entry point: `Athena::go`, with `with_ci_source`, `routing::run_routed_with_record`, `SnapExec::exec` and `hold_for_disk` all real. Fakes: `EarlyFake` (exists in test_support.rs 264-444) and a fake gh Runner, both injected by the new `test_support` installer, which sets the widened thread-local seams. Steps 3-5 add the widened seams, the installer, the Runner and `verifier_stage_reached`. The `with_free_space` hook exists today.
- Criterion 3: Read the diff. Then run `Grep` for `disk_gate(` under crates/athena-core/src/run to list the call sites. Run: `git diff -- crates/athena-core/src/run/mod.rs crates/athena-core/src/run/checks.rs`. Observes: In `check_and_verify` the block `if recorded_checks.is_none() && let Some(low) = self.disk_gate(...)` before `push_ci_branch` is removed. `free_producer_build` is still called at the start of the Checks stage. The only new `disk_gate` call is in `SnapExec::exec`, after `refuse_if_stopped`, behind a once flag. The dispatch (~782), producer (~1341) and verifier call sites are unchanged.
- Criterion 4: Read the diff. Run: `git diff -- CHANGELOG.md`. Observes: One new bullet under `## [Unreleased]` / `### Changed` saying that the writer build is removed before the checks free-disk check, and that the free-disk requirement now applies only just before a local check run, so a CI-supplied result is never held for it. No other CHANGELOG line changes.
- Criterion 5: This is a regression check, not a new behavior. Read the diff to confirm the dispatch gate, the producer gate, `verifier_environment_problem` and the verifier-stage hold in verification.rs are unchanged. `disk.rs` has at most a doc-comment change, and the `gate` and `disk_gate` code is untouched. Then run the drain binary: the existing disk tests must keep passing. These are the `low_disk_*` tests, `low_disk_after_the_checks_holds_the_job_before_the_verifier_runs`, `low_disk_after_the_build_holds_the_job_and_does_not_fail_it` and `the_real_steps_hold_a_job_on_low_disk_without_a_failure_and_a_later_drain_runs_the_same_job`. Run: `git diff -- crates/athena-core/src/run/mod.rs crates/athena-core/src/run/verification.rs crates/athena-core/src/disk.rs && cargo test -p athena-cli --test drain`. Observes: The gate code in those files is untouched, and every pre-existing disk-hold test in drain.rs passes alongside the new ones. The drain.rs `low_once_verifying` helper now calls `verifier_stage_reached`, so it still turns low only after the checks passed.
- Criterion 6: Read the diff of the paragraph on `[disk] min_free_gb` (about lines 40-43). Run: `git diff -- docs/architecture.md`. Observes: Only the 'before the checks' clause changes, to say the producer's build directory is removed first and the free-disk requirement applies just before a local check run. The rest of the paragraph and file is unchanged. That includes the failed-job and `athena retry` wording for the later gates, which already differs from the hold behavior and is not this job's to fix.

Planned files:

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/disk.rs (modify)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base ff72a719f312 → candidate 472309a3eadb. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                           |   7 +
 crates/athena-cli/tests/binary.rs      |  19 ++-
 crates/athena-cli/tests/drain.rs       | 260 +++++++++++++++++++++++++++++++--
 crates/athena-core/src/disk.rs         |  10 +-
 crates/athena-core/src/gh/early.rs     |   6 +-
 crates/athena-core/src/gh/mod.rs       |   2 +-
 crates/athena-core/src/run/checks.rs   |  19 ++-
 crates/athena-core/src/run/mod.rs      |  82 ++++++-----
 crates/athena-core/src/run/tests.rs    |  42 ++++--
 crates/athena-core/src/test_support.rs | 132 +++++++++++++++--
 docs/architecture.md                   |   5 +-
 11 files changed, 496 insertions(+), 88 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41553579164 for 472309a3eadbd336a4767a37d9a17300a4e8185c)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41553579164 for 472309a3eadbd336a4767a37d9a17300a4e8185c)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41553579164 for 472309a3eadbd336a4767a37d9a17300a4e8185c)
- ci-result: `not a check` passed (Check results for 472309a3eadbd336a4767a37d9a17300a4e8185c come from the CI run https://github.com/doodla/athena/actions/runs/41553579164 (463s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
