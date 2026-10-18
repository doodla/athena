# Job 20261018-100259-24wj: verify "Make land --pr wait for failed runs, rerun once, name failing test"

## Request (verbatim)

> try again on 20261018-095724-dqht

## Reading (accepted by the user)

Retry of the failed job: in land_pr's bounded check wait, when checks go Red, keep waiting until the failed workflow runs complete, rerun failed jobs once via GitHub's rerun-failed-jobs endpoint, wait again, then merge if green or report the failing test's name if red. Audience: the operator who wants unattended landings.

Assumptions:
- The retry lives inside land_pr's check wait in crates/athena-core/src/gh.rs and reuses HostOperation::ReadChecks and the existing host-call retry and error-log path.
- The merge gate is unchanged.
- The total wait across the first wait, the rerun and the second wait stays within a bound (about twice the existing 30 looks of 20 s).
- Rerun staleness is detected with run_attempt from the actions/runs reply, with a look allowance as fallback.
- GitHub refuses rerun-failed-jobs on a run still in progress, so the code waits for completion first.
- Docs go in docs/design/native-pr-landing.md and CHANGELOG.md.
- The file list is carried from the earlier attempt; I did not re-read the repository.

## Acceptance criteria

1. [behavior] A FakeHost test where the first read is Red while the run is still in progress shows no rerun is requested until the run completes, then exactly one rerun. Total reads stay within 2 x attempts. The test fails without the wait-for-completion behavior. Run `cargo test -p athena-core gh`.
2. [behavior] A test with the read sequence [Red, Red, Pending, Red] asserts exact counts: first wait 1 read, second wait 3 reads, 1 rerun, no second rerun. A stale run_attempt (unchanged after the rerun) is treated as not yet restarted. Run `cargo test -p athena-core gh`.
3. [behavior] A test where the rerun turns green shows land_pr reports Green and merges when merge_via_rest is on. Run `cargo test -p athena-core gh`.
4. [behavior] A test where the second result is Red shows no merge, no further rerun, and the PrReport carries the failing test's name. Checks still pending after the bound give Pending with no rerun and no merge. Run `cargo test -p athena-core gh`.
5. [content] display::land_pr and the JSON report name the failing test and say the failed job was re-run once. The failure message names the test whether or not a rerun happened.
6. [content] GhClient calls the rerun-failed-jobs endpoint through the existing HostOperation retry and redaction path. A parser test on a sample payload covers the failing test name. The plan or docs list, for each new GitHub call, the state it requires and what GitHub returns when that state is missing.
7. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass with no warnings.
8. [content] docs/design/native-pr-landing.md and CHANGELOG.md state the wait, the single rerun, the completion precondition and the failing-test report.

## Expected deliverables

- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- docs/design/native-pr-landing.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/external_calls.rs (modify)

## Out of scope

- Changing the merge gate
- Retrying more than once
- Changing the Pages rerun behavior
- Changes to athena.toml, defaults.toml or workflows

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests plus cargo test, clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check reruns, merges and reported names by test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and CHANGELOG entry for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the premature-Red-before-run-complete cause, not one instance.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuses the existing host-call path, not new machinery.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change inside land_pr's check wait.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in gh.rs. FakeHost scripts checks [Red, ...] and ci_runs in_progress for the first looks, then completed; it records how many ci_runs looks had happened at each rerun_failed. attempts >= 4. Run: `cargo test -p athena-core gh`. Observes: Zero reruns while the run is in progress; exactly one rerun_failed, made only after the completed look; check reads <= 2 x attempts.
  Fails without the behavior: Without the wait for completion the rerun fires at the first Red with the run still in progress, so the recorded run state is in_progress and the test fails.
- Criterion 2: Unit test with checks [Red, Red, Pending, Red]; ci_runs attempt 1 until rerun, then 1 on the first look afterwards (stale), 2 from the next. Run: `cargo test -p athena-core gh`. Observes: Phase 1 makes 1 read, phase 3 makes 3 reads, 4 in all; one rerun_failed; stale Red not taken as final; report Red with reruns 1; no second rerun.
  Fails without the behavior: Without the run_attempt staleness check, phase 3 stops at the second read (stale Red) and the 3-read assertion fails.
- Criterion 3: Unit test with checks [Red, Pending, Green], a completed failed run, merge_via_rest true; second case with it false. Run: `cargo test -p athena-core gh`. Observes: checks Green, reruns 1, merges 1 and merge_requested true when on; merges 0 when off.
  Fails without the behavior: Without the rerun, checks stay Red, no merge happens and the Green and merge assertions fail.
- Criterion 4: Two unit tests: (a) checks [Red, Red(restarted), Red] with a failed job log containing `test gh::tests::x ... FAILED`; (b) checks always Pending. Run: `cargo test -p athena-core gh`. Observes: (a) merges 0, exactly one rerun_failed, report Red, failing_test Some("gh::tests::x"). (b) report Pending, reruns 0, 0 rerun calls, merges 0, reads equal attempts.
  Fails without the behavior: Without the single-rerun cap (a) calls rerun_failed more than once; without the name lookup failing_test is None; without the Pending path (b) a rerun or merge would occur.
- Criterion 5: Display unit tests on constructed PrReports: Red with test and rerun, Red with test and no rerun, Green after rerun, Red with unknown name; serde_json::to_value check of JSON fields. Run: `cargo test -p athena-cli display`. Observes: Text names the test and says the failed job was re-run once; Red message names the test when reruns is 0; JSON has failing_test and reruns.
  Fails without the behavior: Without the new display branches the Red text stays 'a check failed; nothing was merged' and the name assertions fail.
- Criterion 6: gh.rs parser test on a sample log (timestamp-prefixed line, `---- name stdout ----` fallback, rejected odd name); external_calls.rs test drives rerun_failed through the Injecting runner; docs table lists each new call with required state and reply when missing (rerun-failed-jobs: refused while the run is in progress, 404 for unknown run, empty 201 on success; the exact in-progress status is to be confirmed from GitHub docs). Run: `cargo test --locked`. Observes: Runner sees POST repos/example/project/actions/runs/{id}/rerun-failed-jobs; a one-off 500 is retried with a retry line; a refusal is recorded in the host error log under read-checks; parser returns expected names.
  Fails without the behavior: If the call bypassed api, no retry line or log entry would appear and the injected-500 test fails; with no parser the name assertions fail.
- Criterion 7: Run the full suite, clippy and fmt as the checks Athena runs; the new tests are part of the suite, so they must pass for it to be green. Run: `cargo test --locked`. Observes: Tests exit 0 including the new gh and display tests; then `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check` exit 0 with no warnings.
  Fails without the behavior: If any new test, a trait implementor (FakeHost or GhClient) or the new PrReport fields were left inconsistent, the workspace would fail to compile or a test would fail; an unused item, needless clone or unformatted code would make clippy -D warnings or fmt --check exit non-zero.
- Criterion 8: Inspect the diff of both files. Run: `git diff --stat -- docs/design/native-pr-landing.md CHANGELOG.md`. Observes: native-pr-landing.md has a section on the wait on Red, completion precondition, single rerun, look budget and failing-test report, plus the table of new calls; CHANGELOG.md has an entry under the unreleased heading.

Planned files:

- crates/athena-core/src/gh.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-cli/tests/external_calls.rs (modify)
- docs/design/native-pr-landing.md (modify)
- CHANGELOG.md (modify)

## The change

Base ae4e1fde6433 → candidate f86063c1b383. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              |   7 +
 crates/athena-cli/src/display.rs          |  79 ++++-
 crates/athena-cli/tests/external_calls.rs |  64 +++-
 crates/athena-core/src/gh.rs              | 481 +++++++++++++++++++++++++++++-
 docs/design/native-pr-landing.md          |  38 +++
 5 files changed, 656 insertions(+), 13 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (964 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
