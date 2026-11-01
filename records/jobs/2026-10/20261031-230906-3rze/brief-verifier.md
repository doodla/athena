# Job 20261031-230906-3rze: verify "Push each candidate to athena/ci/<job id> and delete it at close"

## Request (verbatim)

> Priority: urgent
> Push each job's candidate to a CI branch on GitHub, and delete it when the job closes
> 
> First of two small jobs replacing 20261031-150034-17c3, which failed twice: planning grew to 28 items and the writer hit its 3000s limit. This part only pushes; nothing reads the CI result yet.
> 
> 1. Right after the candidate commit is admitted, push it to branch athena/ci/<job id> on the origin remote (through the existing gh::Remote push), and make .github/workflows/ci.yml run on pushes to athena/ci/**.
> 2. Delete athena/ci/<job id> from the remote when the job closes, whatever the outcome.
> 3. With no GitHub origin, do nothing and say so in one log line.
> 
> Tests must not reach the network: fake the remote, and for any test with an https github.com origin also install early_seam with recording fakes, as run_pin_tests.rs does.
> 
> Acceptance:
> - A test through Athena::go shows the CI branch is pushed after admission and deleted at close, through recording fakes.
> - A test shows a repository with no GitHub origin pushes nothing.
> - ci.yml has a push trigger for athena/ci/**, and the required-check job keeps its name 'fmt, clippy, tests'.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run/mod.rs, crates/athena-core/src/gh/early.rs, .github/workflows/ci.yml, crates/athena-core/src/run/tests.rs, CHANGELOG.md

## Reading (accepted by the user)

Reads job 20261031-150034-17c3's first half: after the candidate is admitted, push it to origin branch athena/ci/<job id> through gh::Remote, and delete that branch when the job closes. ci.yml also runs on pushes to athena/ci/**. Nothing reads the CI result yet. With no GitHub origin, one log line says so and nothing is pushed. The audience is the maintainer, who judges it by the tests and CI.

Assumptions:
- Code survey: I read crates/athena-core/src/gh/remote.rs (the Remote trait has remote_head, push, replace and commits_after, but no delete), crates/athena-core/src/gh/early.rs (early_pr skips non-https-github origins silently, and under cfg(test) early_pr_to uses early_seam::current(); check_and_verify calls early_pr), and crates/athena-core/src/run_pin_tests.rs (the existing early_seam install pattern; this path exists, there is no run/run_pin_tests.rs). The entry point is Athena::go in crates/athena-core/src/run/mod.rs. No record or data file is added, so no records validation or AnyRecord change is needed. The behavior is measured directly by recording fakes that log push and delete calls.
- Prior job 20261031-150034-17c3 is addressed by splitting. This job keeps only its push and delete parts (items 1, 2 and 5 of that request, without resolve, athena.toml wait bound, verify::reuse_statement or checks-stage wiring). Its findings about the CI result and reuse statement no longer apply because nothing reads the CI result here. The major finding (real network via early_pr) is fixed: every test with an https github.com origin also installs early_seam with recording fakes, as run_pin_tests.rs does. The ci.yml trigger comment finding is fixed by updating the comment. The history job finding does not apply because nothing reads the result. That job's candidate recorded no changed files, so no earlier work is kept. Its other planned files (ci.rs, ci_tests.rs, checks.rs, verification.rs, close.rs, routing.rs, verify.rs, athena-git, config.rs, athena.toml, docs/design/ci-routing.md) are dropped.
- Add a Remote::delete_branch method with a default that returns an error, as replace does, implemented for GitRemote with git push origin --delete. This needs crates/athena-core/src/gh/remote.rs, which the request did not list. The fake remote in tests records it.
- The push is a forced push (replace-style or a plain refspec), so a branch left by a killed job does not block it. A push or delete failure is one warning line and does not change the job's outcome. Delete runs in a close path that covers every outcome. The skip log line is emitted once per job when origin is missing or not an https github.com URL, for both push and delete.
- athena.toml, defaults.toml and Cargo/clippy config do not change. .github/workflows/ci.yml changes only the push trigger and its comment; the job named 'fmt, clippy, tests' keeps its name.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a test in crates/athena-core/src/run/tests.rs drives Athena::go with a recording fake Remote and early_seam installed with recording fakes. It shows branch athena/ci/<job id> is pushed with the admitted candidate commit after admission and deleted at close, in that order, and that the early pull request fake was also called, so the two pushes are separate.
2. [behavior] Run `cargo test -p athena-core`: a test through Athena::go with no origin, or a non-GitHub origin, records zero pushes and zero deletes on the fake remote, and the UI output has exactly one line saying CI branch push is skipped because there is no GitHub origin.
3. [behavior] Run `cargo test -p athena-core`: a test shows the CI branch is deleted at close when the job ends in failure or rejection, not only on success.
4. [content] Grep .github/workflows/ci.yml: the push trigger lists athena/ci/** as well as main, the comment above it describes these as Athena candidate CI runs, and the job name 'fmt, clippy, tests' is unchanged.
5. [content] Grep the new tests: every test with an https://github.com origin installs early_seam with recording fakes, and no test reaches the network.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.
7. [content] CHANGELOG.md has an entry for the CI branch push and delete, saying that nothing reads the CI result yet.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-core/src/gh/remote.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/gh/mod.rs (modify)

## Out of scope

- Reading or using the CI result in the checks stage, calling resolve, the athena.toml wait bound, or the verifier reuse statement (the second job).
- Changing what the CI workflow checks beyond its trigger and comment.
- Real network calls in tests, or giving the producer GitHub access.
- Changing ci_result.rs, gh/client.rs, the local check implementation, athena.toml, or docs/design/ci-routing.md.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust plus one workflow YAML.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for push/delete and the no-origin case, with fmt, clippy -D warnings and cargo test passing.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the commands to run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only push and delete; the result reading is left to the second job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check recorded push and delete calls through Athena::go.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Branch cleanup is enforced in code at close and checked by test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry and the code comments are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A test in run/tests.rs (module produce_and_verify_order_tests) builds the repo with `origin` = https://github.com/owner/repo, installs `early_seam` with an `EarlyFake` (probe = agents run so far), and drives `Athena::go`. It asserts: `ci_calls` kinds are [push, delete]; the push branch is `athena/ci/<job>` with the same commit as the early push; that commit exists in the repository and is a candidate, not the landed one; the push's probe is 1 (only the producer had run) and the delete's is 2 (the verifier had finished); the early `open_pr` call exists; the fake's CI head ends None. Run: `cargo test -p athena-core ci_branch_is_pushed_after_admission_and_deleted_at_close`. Observes: Recorded CI push and delete calls in order with the candidate commit, plus the separate early push and open_pr.
  Fails without the behavior: Without the push in check_and_verify the fake records no CI push; without the delete in finish_dispatched it records no CI delete; if the CI push reused the early pull request's push, the CI-branch list would be empty and `of("push")` would hold one non-CI push.
  Test seam: Production entry `Athena::go` (exists). The guards are a github.com https origin and `nested` None. The fake is `test_support::EarlyFake`, injected through `crate::gh::early_seam::install(fake.clone(), fake.clone())` (exists, used in run_pin_tests.rs). Step 4 extends `EarlyFake` with CI-branch recording and `delete_branch`; steps 2 and 3 add the production calls.
- Criterion 2: A test in run/tests.rs loops origins [none, a local path, https://gitlab.example.com/owner/repo, https://example.com/github.com/owner/repo], each with the seam installed and a `Lines` UI. It runs `Athena::go` to completion and asserts: the fake's `ci_calls` and `calls` are empty, and exactly one UI line contains 'CI branch push is skipped because there is no GitHub origin'. Run: `cargo test -p athena-core no_github_origin_pushes_no_ci_branch`. Observes: No recorded calls on the fake, and one skip line in the UI.
  Fails without the behavior: Without the origin check the push site would reach the fake (a recorded call) or git; without the skip line the line count is 0, and a second log site would make it 2.
  Test seam: `Athena::go` (exists); `EarlyFake` plus `early_seam::install` as in criterion 1; the origin differs per iteration by `git remote add origin` on the test repo (the pattern in run_pin_tests.rs `early_run`).
- Criterion 3: A test in run/tests.rs drives `Athena::go` twice with a github origin and the seam. First with check `FAILS_ON_EDIT`, which ends Rejected. Second with `check_seam::install` returning `killed_at_the_deadline`, which ends Failed. For each it asserts the final state, that `ci_calls` kinds are [push, delete], and that the fake's CI head is None. It also checks a delete that fails (fake `fail_ci`) gives one warning line and the same state. Run: `cargo test -p athena-core ci_branch_is_deleted_when_the_job_is_rejected_or_fails`. Observes: A recorded CI delete after a Rejected and a Failed close.
  Fails without the behavior: If delete ran only on success, the rejected and failed runs would end with a push and no delete, and the remote head would still be set.
  Test seam: `Athena::go`; `athena_with(.., FAILS_ON_EDIT, ..)` and `check_seam::install` both exist in run/tests.rs; `EarlyFake` and `early_seam` as above.
- Criterion 4: Read the diff of the workflow. Run: `git diff -- .github/workflows/ci.yml`. Observes: `push: branches: [main, "athena/ci/**"]`, a comment above it saying pushes to athena/ci/** are Athena candidate CI runs, and no change to the line `name: fmt, clippy, tests`.
- Criterion 5: Read the diff. Every new test builds its run through one helper, `ci_run(origin, check, ..)`, which installs `early_seam` with an `EarlyFake` before calling `Athena::go` and holds the guard until the end. No new test calls `go` another way, and nothing sets a github origin without that helper. `cargo test -p athena-core ci_` runs without network access. Run: `git diff -- crates/athena-core/src/run/tests.rs`. Observes: The one helper that adds an origin also installs the seam; no `GhCli` or `GitRemote` path is reachable in the new tests.
- Criterion 6: Run the three Athena checks. This also runs the existing run_pin_tests.rs tests, which must keep passing unchanged. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three commands exit 0.
  Fails without the behavior: A missing `delete_branch` impl, an unused import, or an existing test broken by the skip line (a no-origin test that asserts exact UI lines) fails the suite.
  Test seam: All tests as they run under `cargo test`; no new seam.
- Criterion 7: Read the diff of CHANGELOG.md. Run: `git diff -- CHANGELOG.md`. Observes: A new bullet under `## [Unreleased]` / `### Added` saying each candidate is pushed to athena/ci/<job id> and the branch deleted at close, that ci.yml runs on those pushes, that with no GitHub origin it logs one line and does nothing, and that nothing reads the CI result yet.

Planned files:

- crates/athena-core/src/gh/remote.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- .github/workflows/ci.yml (modify)
- crates/athena-core/src/test_support.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 120218c6ea31 → candidate 2499d9429798. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 .github/workflows/ci.yml               |   5 +-
 CHANGELOG.md                           |   9 ++
 crates/athena-core/src/gh/early.rs     |  85 ++++++++++++-
 crates/athena-core/src/gh/mod.rs       |   2 +-
 crates/athena-core/src/gh/remote.rs    |  15 +++
 crates/athena-core/src/gh/tests.rs     |  37 ++++++
 crates/athena-core/src/run/mod.rs      |  22 +++-
 crates/athena-core/src/run/tests.rs    | 221 ++++++++++++++++++++++++++++++++-
 crates/athena-core/src/test_support.rs |  80 ++++++++++--
 9 files changed, 454 insertions(+), 22 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1869 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
