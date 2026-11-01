# Job 20261101-164247-5mrg: verify "Retry 9bxs: judge PRs by required check; held-job records pass head"

## Request (verbatim)

> try again on 20261101-135902-9bxs
> The design passed review; only one test fixture was missed. In crates/athena-cli/tests/external_calls.rs the fake host still returns a check run with no name, so land_pr_with_a_one_off_500_on_each_call_is_retried_and_lands stays Open. Name that check run 'fmt, clippy, tests' as the gh/tests.rs fixtures now are, and search all test fakes for other unnamed check runs.
> - crates/athena-cli/tests/external_calls.rs may change, as may crates/athena-core/src/gh/mod.rs and crates/athena-gate/src/landing_tests.rs.
> - Return the optional-check notes from the same read as the verdict, not a second API call.
> - Do not list a repository-wide fmt, clippy or test run as its own acceptance criterion; the checks stage covers it.
> Priority: urgent

## Reading (accepted by the user)

Retry of job 20261101-135902-9bxs (rejected: external_calls.rs fake host returned an unnamed check run, so the new classifier read it as Pending). Redo both fixes for the drain maintainer: judge a PR only by the required check "fmt, clippy, tests" (optional red checks become notes, returned from the same read as the verdict), let a held job's records pass the PR head check, and name the fake's check run in external_calls.rs and any other test fake.

Assumptions:
- Code survey: read crates/athena-core/src/gh/client.rs (classify_checks at ~766 reads all check runs and combined status; CI_AGGREGATE_JOB = "fmt, clippy, tests" at 516), crates/athena-core/src/gh/early.rs (EARLY_RUN_JOB duplicates the same name at line 25), crates/athena-cli/tests/external_calls.rs (line 90 fake returns a check run with no name), crates/athena-gate/src/invariant.rs (Uncovered::NoVerification at ~278/327). No record or data file is added, so check_record_file/AnyRecord/round-trip tests are not affected. Behavior is measured directly by tests that call the classifier, land_pr and classify_pre_merge.
- Job 20261101-135902-9bxs: its discarded candidate is not in the repo, so the whole fix is redone. Findings addressed: (1) blocker, unnamed check run in external_calls.rs: fix by naming it 'fmt, clippy, tests' and searching every test fake for other unnamed check runs; (2) major, uncited cargo test: the checks stage covers the workspace gate, so no repo-wide run is a criterion; (3) minor, extra files gh/mod.rs and landing_tests.rs: now listed deliverables; (4) minor, optional_failures making two extra API calls: instead return optional-check notes together with the verdict from one read; (5) nit, drain dropping report.warnings: kept as is, since landing output is accepted.
- Kept from the earlier candidate (redone from the base): CHANGELOG.md, gh/client.rs, gh/early.rs, gh/landing.rs, gh/mod.rs, gh/tests.rs, land.rs, invariant.rs, landing_tests.rs, docs/decisions.md. The run/mod.rs entry from the earlier scope was not changed by the candidate, so it is a low-probability deliverable.
- The pre-verdict gate proposed by the introspection is not part of this request; the user's typed request takes precedence and asks only for the fixture fix.
- Required-check name comes from the existing CI_AGGREGATE_JOB constant, with EARLY_RUN_JOB referring to it, so there is one source.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-cli --test external_calls`: land_pr_with_a_one_off_500_on_each_call_is_retried_and_lands ends Landed; the fake host's check run in crates/athena-cli/tests/external_calls.rs is named 'fmt, clippy, tests'.
2. [content] Grep of crates/**/tests*.rs, tests/ and *_tests.rs for check_runs fakes shows no check run without a name; every fake carries 'fmt, clippy, tests' or is intentionally another name.
3. [behavior] Run `cargo test -p athena-core gh`: a PR whose required check 'fmt, clippy, tests' is green and an optional check (the pull request head) is red is reported landed or waiting, never failed, with a note in the report warnings; a red required check is still Red and a missing or pending required check stays Pending.
4. [content] The optional-check notes come from the same check-runs and status read as the verdict (one classify call returning both); Grep of the diff shows no second check_runs_reply or status_reply call for notes.
5. [behavior] Run `cargo test -p athena-gate`: a commit adding a held job's records with no verification passes classify_pre_merge, and a commit adding a finished job's closing records with no verification still returns Uncovered::NoVerification.
6. [content] CHANGELOG.md has an entry describing both fixes (required-checks-only judgment; held-job records pass the pull request head check).
7. [content] Grep of the diff shows 'fmt, clippy, tests' defined once (CI_AGGREGATE_JOB) and reused, not duplicated as a second list.
8. [content] Expected 0% of newly refused readings: the change only narrows refusals.

## Expected deliverables

- crates/athena-cli/tests/external_calls.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/land.rs (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/run/mod.rs (modify)

## Out of scope

- Making the pull request head check required, or editing .github/workflows/ci.yml
- Changing the disk-hold logic or the low-disk threshold
- Changes to the drain's thrash-stop rule for genuine repeated failures
- Changes to records/ or GitHub branch-protection settings
- The pre-verdict workspace gate proposed in the introspection
- Surfacing warnings in crates/athena-cli/src/drain.rs output
- Listing a repository-wide fmt, clippy or test run as an acceptance criterion; the checks stage covers it

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the optional-red, required-red, pending and held-job cases.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria name the cargo test commands and observable outcomes.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Searches all test fakes for unnamed check runs, not just the one that surfaced.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Fixture fix plus the earlier focused change, no extra features.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry and doc comments written for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Run the existing integration test. It drives `athena_core::gh::land_pr` through a real `GhClient` over the file's `Injecting` Runner fake, which injects one 500 per path and serves canned JSON. Run: `cargo test -p athena-cli --test external_calls land_pr_with_a_one_off_500_on_each_call_is_retried_and_lands`. Observes: The report's link state is Landed. The check-runs body at external_calls.rs:90 reads `{"name": "fmt, clippy, tests", "status": "completed", "conclusion": "success"}` with no id. That it reaches Landed with no id also shows the classifier does not need an id.
  Fails without the behavior: With the fake's check run unnamed, the classifier finds no required run and reports Pending, land_pr never merges, and the link stays Open, so the Landed assertion fails. If the classifier went through `parse_check_run`, the named but id-less run would be dropped and the test would fail the same way.
  Test seam: Production entry point `athena_core::gh::land_pr` with a `GhClient` over the `Injecting` Runner in crates/athena-cli/tests/external_calls.rs (exists, read). The fixture edit is plan step 5; nothing is injected into the logic under test.
- Criterion 2: Content check by Grep, then the test run. Grep `check_runs|check-runs` over every `*.rs` file. The fakes found: external_calls.rs:90, gh/tests.rs:86, :675, :680 and :2192. gh/tests.rs:1662 already names its runs. Every other hit is production code or a doc comment. The producer reruns the Grep after editing; the verifier checks each fake object with `git diff -- crates/athena-cli/tests/external_calls.rs crates/athena-core/src/gh/tests.rs`. Run: `cargo test -p athena-core gh`. Observes: Each fake check-run object in test files carries a `name`: 'fmt, clippy, tests', or a deliberately different optional name in the new tests (such as 'pull request head'). No `check_runs` literal in a test file has a run without one. The gh tests, which wait on these fakes, pass.
- Criterion 3: A new table test over `classify_checks(runs, status)` with JSON inputs: (i) required green plus optional 'pull request head' red gives Green and one note naming that check; (ii) required red gives Red; (iii) required in_progress gives Pending; (iv) only an optional green run, no required run, gives Pending; (v) a failed legacy status with required green gives Green plus a note; (vi) a named, id-less, completed-success run 'fmt, clippy, tests' gives Green (this case would catch the id trap); (vii) two runs with the required name, an older red and a newer green, give Green; (viii) the required run skipped gives Green, as before. Then end to end: `land_pr` through a real `GhClient` on a new `Reporting` Runner fake whose check-runs reply is the required run green plus the optional run red. With `merge_via_rest` true the report is Landed with the note in `warnings`. With it false the report is waiting (Open) with `checks == Some(Green)`, never Red. A required-red reply still gives `checks == Some(Red)`. Run: `cargo test -p athena-core gh`. Observes: Verdicts and notes as listed. The note names the optional check and its conclusion. The red optional check never yields Red.
  Fails without the behavior: Without the required-check rule, the optional red run makes `classify_checks` return Red, so cases (i) and (v) and the Landed test fail. Without the name-only pick, case (vi) fails. Without the Pending rule, (iii) and (iv) fail.
  Test seam: Production entry points `classify_checks` and `athena_core::gh::land_pr` via `Rig::run_with` (exists, tests.rs:949), the way `land_through` does at tests.rs:2218. The `Reporting` Runner fake serves canned HTTP-shaped JSON only and holds no classification logic; plan step 6 adds it.
- Criterion 4: A test over the `Reporting` fake, which logs the paths it serves, calls `GhClient::check_report("abc")` once and asserts the logged calls. Also review `git diff -- crates/athena-core/src/gh/client.rs` for one `classify_checks` call that returns the `CheckReport`, and no new `check_runs_reply`, `status_reply` or `optional_failures` call. Run: `cargo test -p athena-core gh`. Observes: Exactly one GET of `commits/abc/check-runs` and one of `commits/abc/status`, and a report that holds both the verdict (Green) and the note for the red optional check.
  Fails without the behavior: If notes came from a second read, the call-count assertion fails. If `classify_checks` stopped returning both, the test does not compile.
  Test seam: Production entry point `GhClient::check_report` over the `Reporting` Runner. `GhClient::api` takes any Runner today (exists); the method is added by plan step 2 and the fake by step 6.
- Criterion 5: Two new tests in landing_tests.rs on a temp git repo, using the existing helpers `owner_repo`, `write_records`, `commit`, `opening`, `closing`, `candidate`. (a) Commit `opening(times)` plus `AnyRecord::Candidate2(Box::new(candidate(fp)))` only, the record `hold_for_disk` writes, with no verification and no outcome, then call `invariant::classify_pre_merge`: Ok, kind RecordsOnly, job None. (b) Commit `closing(fp)` with the Verification record removed, so the Outcome is still there: `Err(Uncovered::NoVerification(job))`. The existing `closing_jobs`, `landing` and `pr_commits` tests keep passing. Run: `cargo test -p athena-gate`. Observes: (a) is Ok and not a job landing; (b) is exactly `Uncovered::NoVerification` for that job id.
  Fails without the behavior: Without the held-job rule, (a) returns `Err(Uncovered::NoVerification(job))` and fails. Without the outcome guard in `held`, (b) also passes the check and fails its assertion.
  Test seam: Production entry point `athena_gate::invariant::classify_pre_merge(&Git, sha)`. Fixtures are the landing_tests.rs helpers and real git commits in a temp repo (exist, read). No double; the tests are added by plan step 4.
- Criterion 6: Inspect the diff. Run: `git diff -- CHANGELOG.md`. Observes: One entry under Unreleased / Fixed, written for outside readers, names both fixes and the root cause of each: the pull request is judged by the 'fmt, clippy, tests' check only, with optional failures as warnings; the records of a held job pass the pull request head check while a finished job's records still need verification.
- Criterion 7: Inspect the diff for literals equal to `fmt, clippy, tests` in non-test code. Run: `git diff -- crates/athena-core/src/gh/client.rs crates/athena-core/src/gh/early.rs`. Observes: The literal appears once, at `CI_AGGREGATE_JOB` in client.rs. `EARLY_RUN_JOB` is `super::client::CI_AGGREGATE_JOB` and `classify_checks` uses the constant. The only other literal outside athena-core test fakes is the commented one in the athena-cli integration test, which cannot see a private constant.
- Criterion 8: No parser, stem list, stop list or reading check is touched, so there are no new input forms and I map no corpus case ids; I did not read the corpus. The producer and verifier run the corpus test to confirm refusals are unchanged. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: The test passes as before, so no refusals were added.

Planned files:

- crates/athena-cli/tests/external_calls.rs (modify)
- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/mod.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- crates/athena-core/src/land.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base c74f3ee40a13 → candidate eee246c8be77. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              |  12 ++
 crates/athena-cli/tests/external_calls.rs |   5 +-
 crates/athena-core/src/gh/client.rs       |  78 ++++++++--
 crates/athena-core/src/gh/early.rs        |  31 ++--
 crates/athena-core/src/gh/landing.rs      |   1 +
 crates/athena-core/src/gh/mod.rs          |   6 +-
 crates/athena-core/src/gh/tests.rs        | 234 ++++++++++++++++++++++++++++--
 crates/athena-core/src/land.rs            |   4 +-
 crates/athena-gate/src/invariant.rs       |  23 ++-
 crates/athena-gate/src/landing_tests.rs   |  41 ++++++
 docs/decisions.md                         |  40 +++++
 11 files changed, 430 insertions(+), 45 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41331044109 for eee246c8be77cf067dbf3a1f0ef2a8888d440acd)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41331044109 for eee246c8be77cf067dbf3a1f0ef2a8888d440acd)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41331044109 for eee246c8be77cf067dbf3a1f0ef2a8888d440acd)
- ci-result: `not a check` passed (Check results for eee246c8be77cf067dbf3a1f0ef2a8888d440acd come from the CI run https://github.com/doodla/athena/actions/runs/41331044109 (583s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
