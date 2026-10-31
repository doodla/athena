# Job 20261031-142738-g437: verify "Make the CI result fetch read only the push run on the job's CI branch"

## Request (verbatim)

> try again on 20261031-001542-anbv

## Reading (accepted by the user)

Retry of job 20261031-001542-anbv, cut to its first slice so one run can finish. Change only the GitHub CI result fetch (`parse_ci_run` and the `CiSource` impl in gh/client.rs, plus tests in gh/tests.rs) so it returns just the `push` run on branch `athena/ci/<job id>`, read from the aggregate `fmt, clippy, tests` job. The goal is a trustworthy exact-tree CI result for the Athena maintainer; wiring it into the run is a follow-up.

Assumptions:
- Job 20261031-001542-anbv failed in planning (round 3 of 3) and its candidate changed no files, so none of its work is kept. Its major finding (the newest `ci` run may be a pull_request run that tests a merge with main) is fixed in this slice by selecting only event `push` on branch `athena/ci/<job id>`. Its minor finding that the reread path calls `self.dispatch` from inside `dispatch` does not apply yet, because this slice does not touch dispatch; it moves to the follow-up that wires the run. Its six scope findings are resolved by listing gh/tests.rs here and moving the run/* files and gh/early.rs to follow-ups.
- Code survey: crates/athena-core/src/gh/client.rs. `parse_ci_run` (about line 519) picks the newest run named `ci` by created_at with no event or branch filter. It copies the conclusion of the one step starting with `cargo fmt --check` to all three checks. `impl CiSource for GhClient::fetch` (about line 570) queries `actions/runs?head_sha=..&per_page=20`. crates/athena-core/src/ci_result.rs: `CiSource::fetch(&self, sha)` and `resolve` check only `run.sha == sha`. .github/workflows/ci.yml has an aggregate job `ci` named `fmt, clippy, tests` (about line 400) next to the separate fmt, clippy and test jobs. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected. The property is measured directly by feeding fixture JSON for both runs to the parser and the source.
- The branch is passed by a new constructor or wrapper, for example `GhClient::ci_source(branch)`, so the `CiSource` trait signature stays unchanged. It adds `&event=push&branch=<branch>` to the runs query, and the parser also filters `event == push` and `head_branch == branch` as a second guard.
- Kept to one producer run: no governance file, workflow, athena.toml or prompt change. This is not a refusal rule, so no refusal-rate criterion applies.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core gh`. A new fixture test has two runs for the same SHA, a `pull_request` run with the newer created_at and a different conclusion, and a `push` run on `athena/ci/<job id>`. It shows that only the push run's result is returned.
2. [behavior] Run `cargo test -p athena-core gh`. A new test where only a `pull_request` run exists, or a push run exists on another branch, shows that the source returns no result (None), so `resolve` reports it missing and never uses that run.
3. [behavior] Run `cargo test -p athena-core gh`. A new test shows that the HTTP path built by the CI source includes `event=push` and the branch name. Use the recording transport in gh/tests.rs if present, or assert on the built query.
4. [behavior] Run `cargo test -p athena-core gh`. A new fixture test shows that a failed aggregate `fmt, clippy, tests` job gives red steps, and a successful one gives green steps for all three check names. The current behaviour of copying the `cargo fmt --check` step to all three checks is gone (Grep for `CI_CHECK_STEP` in client.rs).
5. [content] crates/athena-core/src/gh/client.rs filters runs on `event` and `head_branch`. The other changed files are crates/athena-core/src/gh/tests.rs and, only if needed, ci_result.rs call sites. No change appears in run/mod.rs, ci.yml, athena.toml or CHANGELOG.md.
6. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.

## Expected deliverables

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Follow-up request A: wire `resolve` and `run_routed` into the run in run/mod.rs, replacing `NoCi`. This covers the verifier reuse statement, the CI wait bound in athena.toml and config.rs, and the local fallback.
- Follow-up request B: push `athena/ci/<job id>` after admission and delete it at close (athena-git, run/mod.rs, and the reread path calling `dispatch` recursively so the delete runs once). This also covers run/ci.rs, run/verification.rs, run/ci_tests.rs, run/tests.rs and gh/early.rs.
- Follow-up request C: make .github/workflows/ci.yml run on `athena/ci/**` pushes, update docs/design/ci-routing.md section 10 items 4 and 5 and the 'either run serves' text, and add the CHANGELOG.md entry.
- Changing what the CI workflow checks, giving the producer GitHub access, or making real network calls in tests.
- Files the failed job's candidate changed: none were recorded, so nothing is kept or dropped.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change and its tests are Rust in crates/athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests cover the filter; fmt, clippy and the full test suite must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused slice of the request; the rest is named in out_of_scope as follow-ups.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run fixture tests with competing runs, not just check that code exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The parser and source are fixed, not just the one call site.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments on the changed functions state which run is trusted and why.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Fixture test with two runs for one SHA, driven through the production parser and the production source. The Runner fake returns canned JSON and records paths. Run: `cargo test -p athena-core gh`. Observes: `ci_push_run_wins_over_newer_pull_request_run`: the runs reply holds a pull_request run (newer created_at, conclusion failure) and a push run on `athena/ci/<job id>` (older, success). `parse_ci_run` and `ci_source(branch).fetch(sha)` both return the push run's address, conclusion `success` and three `success` steps. The recorded jobs request names the push run's id. The reverse case returns the push run's red result.
  Fails without the behavior: With the old newest-`ci`-run selection the pull_request run is returned, so the address and conclusion assertions fail.
  Test seam: Production entry points: `GhClient::ci_source(branch)` with `CiSource::fetch` (steps 3 and 4 add them in gh/client.rs) and `parse_ci_run` (exists; the signature changes in step 3). Fake: `CiRunner` in gh/tests.rs, an implementation of the existing `Runner` trait (gh/transport.rs), injected through `GhClient::new(&runner, repo)` like `FakeRunner`. Step 5 adds the fake. It returns canned strings only; selection and verdict logic stay in production code.
- Criterion 2: Fixture tests where the only candidates are a pull_request run, or a push run on another branch, checked at the parser, the source and `resolve`. Run: `cargo test -p athena-core gh`. Observes: `ci_push_run_ignores_other_events_and_branches`: `parse_ci_run` returns None and `fetch` returns Ok(None) in both cases, though the canned reply (which ignores the query) holds a green run for the right SHA. `resolve` with the `CiBranchSource` and a failing `LocalChecks` fake returns `Outcome::Failed` with `Source::Local`, and the local fake was called once.
  Fails without the behavior: Without the event and head_branch filter, `fetch` returns the green pull_request run, `resolve` returns Verified with `Source::Ci`, and the local call count stays 0, so the assertions fail.
  Test seam: Production entry points: `CiBranchSource::fetch` via `GhClient::ci_source` (step 4), `parse_ci_run` (step 3) and `ci_result::resolve` (exists, unchanged). Fakes: `CiRunner` and a `LocalChecks` fake with a call counter (step 5), injected as arguments to `GhClient::new` and `resolve`. The canned reply ignores the query on purpose, so the parser-side guard is what is tested.
- Criterion 3: A recording Runner fake captures the args of the runs request made by the production source; the test asserts on the path. Run: `cargo test -p athena-core gh`. Observes: `ci_push_run_query_names_event_and_branch`: the recorded `gh api` args hold a path starting `repos/example/project/actions/runs?` that contains `head_sha=<sha>`, `event=push` and `branch=athena/ci/<job id>`. The next recorded call is `actions/runs/<id>/jobs?filter=latest&per_page=100`.
  Fails without the behavior: If `fetch` still built `actions/runs?head_sha=..&per_page=20`, the `event=push` and `branch=` assertions fail.
  Test seam: Production entry point: `CiBranchSource::fetch` through `GhClient::ci_source` (step 4). Fake: `CiRunner` (step 5) records args in a `RefCell<Vec<Vec<String>>>`, like `FakeRunner.calls` (exists); it is injected via `GhClient::new`.
- Criterion 4: Fixture tests on jobs replies built from the job names in ci.yml, driven through `parse_ci_run` and `fetch`; plus a Grep for the removed constant. Run: `cargo test -p athena-core gh`. Observes: `ci_push_run_verdict_comes_from_the_aggregate_job`: a failed `clippy` job with a failed aggregate job gives failure with clippy `failure` and fmt `success`. A failed `fmt` job with a failed aggregate gives red. A cancelled `fmt` job with a failed aggregate, and a failed gate, give `unusable`, so `resolve` falls back to local. A successful aggregate gives three `success` steps, also with skipped check jobs. A run failed only by `the full history` with a successful aggregate is `success`. An aggregate job in progress gives finished=false. An aggregate `cancelled` gives conclusion `cancelled` and no steps. No aggregate job gives `unusable`. A job named `fmt` is not the aggregate. Then `Grep CI_CHECK_STEP crates/athena-core/src/gh/client.rs` finds no match.
  Fails without the behavior: With the old parser (the step starting `cargo fmt --check`) the fixtures have no such step, so steps are empty and the assertions fail; a failure-always rule would fail the cancelled-`fmt` and failed-gate cases; the `history`-only fixture would give a failure conclusion.
  Test seam: Production entry points: `parse_ci_run` and `CiBranchSource::fetch` (steps 2 to 4). Fixtures: JSON built with `json!` in gh/tests.rs (existing style), served by `CiRunner` for the fetch path (step 5). The fakes return canned values only.
- Criterion 5: Content check: Grep the filter in client.rs and list the changed files in the diff. Run: `git diff --stat`. Observes: The diff lists only crates/athena-core/src/gh/client.rs and crates/athena-core/src/gh/tests.rs. `Grep` for `event` and `head_branch` in client.rs finds the filter in `pick_ci_run`. No change in run/mod.rs, ci.yml, athena.toml, CHANGELOG.md or ci_result.rs.
- Criterion 6: Run the three Athena checks on the final tree. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. The build shows that nothing else used the removed `impl CiSource for GhClient`, `CI_CHECK_STEP` or the old `parse_ci_run` signature, and that the xtask policy and boundary tests accept the new test text.
  Fails without the behavior: A caller left on the old `parse_ci_run(runs, jobs)` signature, an unused import or constant, or an unformatted file would fail the build, clippy or fmt.
  Test seam: No injection; the whole workspace.

Planned files:

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 9e1118042896 → candidate 0188fe1244e0. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 crates/athena-core/src/gh/client.rs | 258 ++++++++++++++++-----
 crates/athena-core/src/gh/tests.rs  | 451 +++++++++++++++++++++++++++++++++++-
 2 files changed, 645 insertions(+), 64 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1825 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
