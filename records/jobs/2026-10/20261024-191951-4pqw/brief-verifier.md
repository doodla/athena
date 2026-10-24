# Job 20261024-191951-4pqw: verify "Add `athena check --pr-head` pre-merge check and non-blocking CI job"

## Request (verbatim)

> Build job 4, part 1, of the records-history plan (docs/design/records-history.md sections 5.1 and 10, decision D111): a pre-merge check on a pull request's head, as a command (for example athena check --pr-head <base> <head>) that passes only when every commit between the base and the head classifies (via land.rs pr_commits) as a job landing whose verification passed and whose prediction precedes its outcome, a declared intervention, or records-only, and the head's content equals the last job landing's verified content plus only records-only commits after it; otherwise it fails naming each offending commit. Add it to .github/workflows/ci.yml as a new job on pull requests that does not block merging yet (it becomes required at the cutover, job 7), and list .github/workflows/ci.yml as a governance deliverable. Tests: a head with an unverified commit fails; a head shaped like a real Athena pull request passes; a head with a declared intervention after the job landing fails, since a squash of it is not a valid landing until the cutover.

## Reading (accepted by the user)

Adds a command (`athena check --pr-head <base> <head>`) that classifies every commit in base..head with `land::pr_commits` and passes only if each is a verified job landing with prediction before outcome, a declared intervention, or records-only, and the head's content equals the last job landing's verified content plus records-only commits. A new non-required pull_request job in ci.yml runs it. The audience is maintainers and CI reviewing Athena PRs before the job-7 cutover.

Assumptions:
- The command is a new `--pr-head <base> <head>` option on the existing `Check` subcommand (conflicting with --history), since `athena check` already exists; a separate subcommand is acceptable if clap makes that cleaner.
- The core logic lives in athena-core (land.rs or a small new module) so tests can build git fixtures with the existing land.rs test fixtures.
- 'Prediction precedes outcome' is checked within the head's tree/history using the records the job landing carries, per design section 5.1/10; the producer reads the design doc for the exact rule.
- The CI job is simply not added to branch protection; nothing in athena.toml or defaults.toml changes.
- Goal: a pre-merge gate for Athena PRs; audience is the maintainer and CI reading pass/fail output.

## Acceptance criteria

1. [behavior] Run `cargo test`: a test builds a PR range containing a commit that changes governed content with no verification or intervention, and the pr-head check fails and names that commit's sha.
2. [behavior] Run `cargo test`: a test builds a head shaped like a real Athena PR (job landing with verification, prediction before outcome, optional records-only commits after) and the check passes.
3. [behavior] Run `cargo test`: a test with a declared-intervention commit after the job landing fails, naming that commit and saying a squash of it is not a valid landing until the cutover.
4. [behavior] Run `cargo test`: a test shows a job landing whose prediction does not precede its outcome, and a head whose content differs from the last landing's verified content plus records-only commits, each fail naming the offending commit; every offending commit is listed, not only the first.
5. [content] `athena check --pr-head <base> <head>` exists in the CLI, uses `land::pr_commits` for classification, exits non-zero on failure and zero on pass, and the existing `check` and `check --history` behavior is unchanged.
6. [content] .github/workflows/ci.yml gains a new job that runs only on pull_request, passes the PR base and head shas (with full history fetched) to the command, and is commented as not yet required (becomes required at the cutover, job 7). Existing jobs are unchanged.
7. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both clean; `cargo test` passes fully.

## Expected deliverables

- crates/athena-core/src/land.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- .github/workflows/ci.yml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/pr_check.rs (create)
- crates/athena-cli/src/display.rs (modify)
- README.md (modify)

## Out of scope

- Making the new CI job a required status (job 7 cutover).
- Records-branch ordering checks or changes to how landings are written (job 4 part 2 and later).
- Changing pr_commits classification rules.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Three requested tests plus prediction-order and content-mismatch cases; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the check on built PR shapes and assert pass/fail and named commits.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a CI job and command, marked not-yet-required.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One command, one CI job, no changes to pr_commits.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Error messages name commits and the reason; changelog entry written for outside readers.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt --check.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code in Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test on a git fixture: a PR range holding one commit that edits a.txt with no job records or intervention. The test runs the check and asserts it fails and the report names that commit's sha and the reason. Run: `cargo test --locked pr_check::tests::an_unverified_commit_fails_and_is_named`. Observes: Report is a failure whose offence list contains exactly that sha (and its 'no verification covers it' reason).
  Fails without the behavior: With no per-commit classification error collected, the check would pass or report no sha.
- Criterion 2: Fixture: a records-only reading commit, a dispatch commit with job.json and an accepted reading recorded before it, an intervention before the landing, a landing commit whose verification passed and whose verified_fingerprint is the real fingerprint, then a records-only commit. Run the check. Run: `cargo test --locked pr_check::tests::a_head_shaped_like_an_athena_pull_request_passes`. Observes: The report passes with no offences; the classified commits are RecordsOnly…, JobLanding, RecordsOnly.
  Fails without the behavior: A check that rejected valid shapes (e.g. an intervention before the landing, or trailing records-only) would fail this test.
- Criterion 3: Same valid head, plus an intervention commit (a governed edit with an intervention record) after the landing. Assert it fails, names that commit's sha, and the message contains 'a squash of it is not a valid landing until the cutover'. Run: `cargo test --locked pr_check::tests::a_declared_intervention_after_the_landing_fails`. Observes: Failure naming the intervention sha with the squash wording; the landing is not named.
  Fails without the behavior: Without the after-landing rule the head would pass.
- Criterion 4: Three tests. (a) A landing whose reading is recorded after the dispatch or an invocation start fails naming the landing sha. (b) A landing whose verified_fingerprint differs from the head's content fails naming that landing sha. (c) One range with an unverified commit, a late-prediction landing and a trailing intervention reports all three shas, not only the first. Run: `cargo test --locked pr_check::tests`. Observes: Each failure message contains the expected sha(s); in (c) the offence list has three entries in commit order.
  Fails without the behavior: Stopping at the first error (as `pr_commits` does) would omit shas in (c); skipping the prediction or content rule would make (a) or (b) pass.
- Criterion 5: Inspect the diff. `Check` gains a `pr_head` option conflicting with `history`; its branch calls `athena_core::pr_check`, which classifies through `land::classify_commit`/`pr_commits`, and returns 2 on failure and 0 on pass. The `history` and working-state branches are untouched. `pr_commits` tests still pass. Run: `git diff -- crates/athena-cli/src/main.rs crates/athena-core/src/land.rs`. Observes: Only additions in the Check arm plus the behaviour-preserving extraction in land.rs; the existing land.rs `pr_commits` tests pass unchanged under `cargo test --locked`.
- Criterion 6: Inspect the diff: a new `pr-head` job gated by `if: github.event_name == 'pull_request'`, fetch-depth 0, running `athena check --pr-head "$BASE" "$HEAD"` with env from `github.event.pull_request.base.sha` and `.head.sha`, commented as not yet required (becomes required at the cutover, job 7). The diff shows no changes to the `checks` or `history` jobs. Run: `git diff -- .github/workflows/ci.yml`. Observes: The diff is purely additive and contains the comment text.
- Criterion 7: Run clippy, then `cargo fmt --all --check`, then `cargo test --locked`. Run: `cargo clippy --all-targets --locked -- -D warnings`. Observes: All three exit 0 with no warnings and no failing tests.
  Fails without the behavior: A lint, format drift or a broken existing test would make a command exit non-zero.

Planned files:

- crates/athena-core/src/pr_check.rs (create)
- crates/athena-core/src/land.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- .github/workflows/ci.yml (modify)
- CHANGELOG.md (modify)
- docs/design/records-history.md (modify)
- README.md (modify)

## The change

Base 3feaed9edd7a → candidate 01bdc007a590. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 .github/workflows/ci.yml            |  22 ++
 CHANGELOG.md                        |  11 +
 README.md                           |   1 +
 crates/athena-cli/src/display.rs    |  18 ++
 crates/athena-cli/src/main.rs       |  32 ++-
 crates/athena-core/src/land.rs      | 109 ++++----
 crates/athena-core/src/lib.rs       |   1 +
 crates/athena-core/src/pr_check.rs  | 539 ++++++++++++++++++++++++++++++++++++
 crates/athena-gate/src/invariant.rs |   2 +-
 docs/design/records-history.md      |   9 +
 10 files changed, 691 insertions(+), 53 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1115 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
