# Job 20261104-153614-pzfw: verify "Reuse main's newest green CI run with the base's fingerprint"

## Request (verbatim)

> Baseline reuse never finds main's result. In crates/athena-core/src/run/checks.rs the default-branch path asks main's CI for a run on the base's own content commit, but that commit exists only on the working branch: main holds a different squash commit with the same governed content. So every job since the fix still re-ran the 20-minute local baseline ('main: no usable CI result for 609bd60... on main', 4 Nov 15:35 UTC, while main's tip fe1df4f had a green ci run). Change the default-branch path to take main's newest green CI run whose commit has the same governed-content fingerprint as the base, instead of looking up the base's own commit.
> 
> Acceptance:
> - a test where main's tip is a squash commit with the base's fingerprint and a green CI run, and the base is a different commit: the baseline is reused from that run
> - a test where main's tip has a different fingerprint: the baseline runs locally as before
> - the existing baseline-reuse tests still pass

## Reading (accepted by the user)

In `reused_baseline` (crates/athena-core/src/run/checks.rs), the default-branch path stops asking main's CI about the base's own content commit. It now takes main's tip (the local remote-tracking ref of the default branch) and uses its green CI run when that commit has the same governed-content fingerprint as the base. Otherwise the baseline runs locally. The audience is the job runner and maintainers, who need the 20-minute baseline skipped.

Assumptions:
- Code survey: read crates/athena-core/src/run/checks.rs (reused_baseline: step 1 is the landed job's candidate run; step 2 is the default-branch push run looked up at content_commit(base) via green_run, after a fingerprint(base) check; baseline_stage/take_baseline falls back to a local run on Err), crates/athena-core/src/run/mod.rs (with_ci_source_on reads one run with no wait, via a test seam), crates/athena-core/src/reconcile.rs (local_default_branch reads origin/HEAD) and crates/athena-core/src/run/tests.rs (commit_landed_squash, ci_reuse_run_with, CiReplies::only_for, baseline_reads, and the three existing baseline_* tests around lines 2915-3100). No record or data file is added, so the records allow-list, check_record_file and AnyRecord are untouched.
- Main's tip is resolved from the local remote-tracking ref (origin/<default branch>) with no network fetch. The request says 'newest green CI run' on main, so the simplest reading is main's tip first, then a bounded walk back over its first-parent ancestors (a small cap) until a commit with the base's fingerprint is found. Each candidate goes through the existing green_run, so the finished, green and covers-fmt/clippy/test rules, and the no-wait one-read rule, all still apply.
- Fingerprints are compared with the existing fingerprint(git, commit, commit) function against the base's own fingerprint, as step 1 does. If the commit is not in the local object store, that case yields a reason string and falls back to a local run.
- The existing default-branch test (baseline_reused_from_the_default_branch_run_runs_nothing_locally) uses a main run on the commit under the records. It should keep passing, since that commit is main's tip with the same fingerprint, and may need a small setup adjustment so origin/main points at it.
- No workflow, athena.toml or defaults.toml change is needed.

## Acceptance criteria

1. [behavior] A new test in crates/athena-core/src/run/tests.rs has main's tip as a squash commit that is not the base's content commit but has the same governed-content fingerprint, with a green CI run on main for that tip. It asserts the baseline has reused_from set to the run address, all checks are reused and passed, baseline_ran is false, and the host read was for the squash SHA on branch main. Run `cargo test -p athena-core baseline_reused`.
2. [behavior] A new test has main's tip with a different governed-content fingerprint, even if it has a green run. It asserts the baseline runs locally (baseline_ran true, reused_from None) and the output line gives the fingerprint-mismatch reason. Run `cargo test -p athena-core baseline_runs_locally`.
3. [behavior] The existing baseline tests still pass unchanged in meaning: baseline_reused_from_the_landed_candidates_run_runs_nothing_locally, baseline_reused_from_the_default_branch_run_runs_nothing_locally, baseline_runs_locally_when_no_green_ci_result_exists, and the ci_reuse_* tests. Run `cargo test -p athena-core`.
4. [content] In crates/athena-core/src/run/checks.rs, the default-branch step of reused_baseline no longer passes the base's own content commit to green_run on the default branch. It passes main's commit chosen by matching fingerprint with the base's, and the doc comment of reused_baseline describes this.
5. [behavior] `cargo clippy --all-targets` and `cargo fmt --check` are clean, and the change touches only the reuse path, so the landed-candidate path and the local fallback are unchanged.

## Expected deliverables

- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/reconcile.rs (modify)

## Out of scope

- Changing the landed-candidate path (step 1) of reused_baseline.
- Fetching or polling the host for new main commits, or waiting for an unfinished run.
- Changing CI workflow files, athena.toml or defaults.toml.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the match and mismatch cases; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the baseline being reused or run locally, not that wiring exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The lookup key is changed from the working-branch commit to a fingerprint match on main, rather than patching one case.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the default-branch step.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo test commands the verifier runs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 27ee7e14f96d → candidate 467da5cbf453. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  5 +++
 crates/athena-core/src/run/checks.rs | 33 +++++++++-----
 crates/athena-core/src/run/tests.rs  | 85 ++++++++++++++++++++++++++++++++++++
 3 files changed, 112 insertions(+), 11 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41892229021 for 467da5cbf4537e93cb27df9c24e46e45591d42fa)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41892229021 for 467da5cbf4537e93cb27df9c24e46e45591d42fa)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41892229021 for 467da5cbf4537e93cb27df9c24e46e45591d42fa)
- ci-result: `not a check` passed (Check results for 467da5cbf4537e93cb27df9c24e46e45591d42fa come from the CI run https://github.com/doodla/athena/actions/runs/41892229021 (669s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo clippy --all-targets` and `cargo fmt --check` are clean, and the change touches only the reuse path, so the landed-candidate path and the local fallback are unchanged.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
