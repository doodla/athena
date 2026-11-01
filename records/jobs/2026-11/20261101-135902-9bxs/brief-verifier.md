# Job 20261101-135902-9bxs: verify "Judge PRs by required checks; let held-job records pass head check"

## Request (verbatim)

> Two problems stop drain landings from counting, both seen on the disk fix's pull request.
> 1. The drain reports a landed job as 'not landed: pull request is Open, checks Red' when the only red check is one GitHub does not require (the pull request head check, marked not yet required in .github/workflows/ci.yml), and then stops the run for a repeated failure. Judge the pull request by the checks main requires, as recorded for the required-check names; a red optional check is a note, not a failure.
> 2. The pull request head check (crates/athena-gate/src/invariant.rs, 'adds the records of job ... without its verification') refuses the records a job writes when it is held for low disk, because a held job has no verification yet. A commit that only records a hold, or a held job's progress, must pass that check, as other records-only commits do.
> - A drain test: a pull request whose required check is green and an optional check is red is reported landed or waiting, never failed, and the run does not stop.
> - An invariant test: a commit adding a held job's records with no verification passes the pull request head check; a commit adding a finished job's records with no verification still fails.
> - CHANGELOG.md gets an entry.
> Priority: high

## Reading (accepted by the user)

Fix two blockers for drain landings, for the maintainer running the drain. (1) The drain judges a pull request only by the required check ("fmt, clippy, tests"), so a red optional check such as the pull request head check is a note, not a failure. (2) The pre-merge classification no longer returns NoVerification for a held job's records, while a finished job's closing records without verification still fail.

Assumptions:
- Code survey: read crates/athena-core/src/gh/client.rs (PrHost::checks at ~405 calls classify_checks at ~766, which turns any failed check-run into Checks::Red and has no notion of required names; CI_AGGREGATE_JOB = "fmt, clippy, tests" at ~516), crates/athena-core/src/gh/early.rs (EARLY_RUN_JOB), crates/athena-core/src/gh/landing.rs (~169-340, the landing wait and the 'checks red' standing text), and crates/athena-gate/src/invariant.rs (closing_jobs ~234, Uncovered::NoVerification ~278, classify_pre_merge ~311, which returns NoVerification when a job's closing records lack verification.json). The entry points are the landing wait in gh/landing.rs and classify_pre_merge; the producer must locate how the drain maps Red to 'not landed' and the repeated-failure stop.
- The required-check name set is what main requires: the single job named 'fmt, clippy, tests' (see xtask/tests/ci_shards.rs REQUIRED_CHECK_NAME). The producer should reuse that existing constant or recorded name instead of adding a second list; optional checks (the pull request head, attribution of main) are ignored when judging red, and appear only as a note.
- A 'held job' is one whose state is a low-disk hold (crates/athena-core/src/disk.rs, state.rs). The invariant fix must identify it from the job's own records in the commit tree (for example a hold record present and no outcome or candidate record), so a finished job (candidate or outcome records) without verification still yields NoVerification.
- No record or data file is added under crates/, so the records validation allow-list (check_record_file, AnyRecord round-trip) is not touched, unless the producer finds the hold marker is a new record kind, which this reading does not expect.
- The change adds no new refusal; it narrows two existing ones, so the replay refusal rate should fall. Expected 0% of newly refused readings.
- Governance: .github/workflows/ci.yml, athena.toml and defaults.toml need no change; the 'not yet required' marking stays as is.

## Acceptance criteria

1. [behavior] A drain/gh test (cargo test -p athena-core) where the required check 'fmt, clippy, tests' is green and an optional check (the pull request head) is red reports landed or waiting, never failed, and the run does not stop for a repeated failure.
2. [behavior] A drain test (cargo test -p athena-core) shows that a red required check is still reported as failed, and a pending required check stays waiting, so the behavior is not weakened.
3. [behavior] An invariant test (cargo test -p athena-gate) shows a commit adding a held job's records with no verification passes classify_pre_merge (no Uncovered::NoVerification), and a commit adding a finished job's closing records with no verification still returns NoVerification.
4. [content] The red-optional-check case is surfaced as a note in the drain or landing output (visible in the test's assertion on the report), not as the 'not landed: pull request is Open, checks Red' error.
5. [content] CHANGELOG.md has an entry that describes both fixes (required-checks-only judgment, held-job records pass the pull request head check).
6. [behavior] cargo test, cargo clippy --all-targets and cargo fmt --check pass for the workspace.
7. [content] The required check names are taken from one existing source (the 'fmt, clippy, tests' constant in the code), and not duplicated as an unrelated second list; verified by Grep of the diff.
8. [content] Expected 0% of newly refused readings: the change only narrows refusals, so the higher refusal rate is not intended.

## Expected deliverables

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Making the pull request head check required, or editing .github/workflows/ci.yml
- Changing the disk-hold logic itself or the low-disk threshold
- Changes to the drain's thrash-stop rule for genuine repeated failures
- Changes to records/ or branch-protection settings on GitHub

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new drain and invariant tests and passing cargo test, clippy, fmt.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check outcomes (landed or waiting report, pass or fail of the head check) through named tests.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Judge by required checks generally, and classify held jobs by their records, not one instance.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Both rules are enforced in code and tests, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change covering the two stated problems.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: A new test in gh/tests.rs, `a_red_optional_check_does_not_fail_the_landing`. It runs `land_pr` through `Rig::run_with` with a `GhClient` over a new scripted runner. The runner serves check-runs `[{name:"fmt, clippy, tests", success}, {name:"pull request head", failure}]`, an unmerged pull request until the merge PUT, then a merged one with commit `merged1`. It uses `merge_via_rest` true, and also false for the waiting case. Run: `cargo test -p athena-core a_red_optional_check_does_not_fail_the_landing`. Observes: The report has `checks == Some(Checks::Green)`, `reruns == 0`, `link.state == Landed` and `merge_requested`. Nothing is `Red`, so `PrLander::land` returns `Landing::Landed` and drain.rs:1537 `failures` is never incremented. With the setting off the report is Open and Green, and nothing failed. The stop is reached only through `NotLanded`, drain.rs:1445-1449 and 1537-1539, which this path never takes.
  Fails without the behavior: Without required-only judgment, `classify_checks` returns Red for the failed 'pull request head' run, the wait reruns, the report is Red and Open and nothing merges, so the Landed and Green assertions fail.
  Test seam: Production entry point `land_pr` (gh/landing.rs) with `GhClient::checks`/`classify_checks`. Fixture: a new scripted `Runner` impl in gh/tests.rs, injected as `&dyn PrHost` via `Rig::run_with(host, ...)`, which takes `&dyn PrHost` today. The new runner is added by plan step 5. `Rig`, `FakeContent` and `GhClient::new(&runner, ..)` exist (tests.rs:56-93, 915-991).
- Criterion 2: Two more tests with the same scripted runner and `land_pr`. In one the required run concludes failure, with a log text for `failing_test`. In the other the required run is `in_progress`, and in a variant it is absent while the optional run is green. A unit test on `classify_checks` pins the three JSON shapes. Run: `cargo test -p athena-core required_check`. Observes: A red required run gives `checks == Some(Red)`, no merge, `reruns` 0 or 1 as today, and the pull request stays Open. A pending or absent required run gives Pending after exactly `attempts` reads, no merge and no rerun. The existing tests at tests.rs:1026-1116 still pass.
  Fails without the behavior: If the classifier ignored the required check, or treated 'absent' as green, the red case would be Green or merged and the pending case would merge, so the Red and Pending assertions fail.
  Test seam: Same as criterion 1: `land_pr` plus `classify_checks` through a scripted `Runner` (added by step 5). The existing fixtures at tests.rs:86 and 677 return unnamed check-runs and must be renamed to 'fmt, clippy, tests' in step 5.
- Criterion 3: New tests in crates/athena-gate/src/landing_tests.rs, using `owner_repo()`, `write_records`, `closing()`/`candidate()`/`outcome()` and a commit helper. One commit adds a held job's `candidate.json` and an invocation record, with no outcome and no verification. `classify_pre_merge(git, sha)` is expected to return `Ok(PreMergeCommit)` with kind `RecordsOnly`. Another commit adds `candidate.json` and a Landed `outcome.json` with no verification, and is expected to return `Err(Uncovered::NoVerification(job))`. Run: `cargo test -p athena-gate held_job`. Observes: The held commit is `Ok` and `RecordsOnly` with `job == None`, and its `clause()` text is never produced. The finished commit is `Err(NoVerification)` and `clause()` contains 'without its verification'. The existing land.rs test `a_landing_without_a_verification_is_refused` still passes.
  Fails without the behavior: Without the held-job rule, the held commit returns `Err(NoVerification)` (invariant.rs:323-328) and the Ok assertion fails.
  Test seam: Production entry point `classify_pre_merge` (invariant.rs:311) over a real temp git repository. The fixtures in landing_tests.rs exist (lines 42-310). The commit helper or the test-only record builders for a candidate-only and an outcome-only commit are added by step 8.
- Criterion 4: The criterion-1 test also asserts on `report.warnings`. It is the PrReport field the athena land output prints. It is checked by exact content. Run: `cargo test -p athena-core a_red_optional_check_does_not_fail_the_landing`. Observes: `warnings` has one entry containing 'pull request head' and 'not required', and no entry or error contains 'checks Red' or 'not landed'. No `Err` is returned.
  Fails without the behavior: Without the note, `warnings` is empty and the contains assertion fails. Without the fix the call returns Red and no note.
  Test seam: `PrReport.warnings` from `land_pr`, populated by `PrHost::optional_failures` on `GhClient` (added by step 4). The default trait method keeps the other hosts unchanged (test_support.rs:207 and 420, tests.rs:845 and 1599).
- Criterion 5: Read the new entry under `## [Unreleased]` in CHANGELOG.md (`### Fixed`). It names both fixes and says that the pull request head check stays optional. Run: `git diff -- CHANGELOG.md`. Observes: One added bullet covers: the drain judges a pull request by the required 'fmt, clippy, tests' check and reports other red checks as notes, and records of a held job pass the pull request head check (`classify_pre_merge`).
- Criterion 6: Run the three workspace checks. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. The renamed fixtures and the new `optional_failures` default method compile in every `PrHost` impl.
  Fails without the behavior: A fixture still using an unnamed check-run would read Pending and break the client test at tests.rs:106. A trait change without a default would fail to compile for the other hosts.
  Test seam: The workspace build and test commands, which exist.
- Criterion 7: Inspect the diff. The literal 'fmt, clippy, tests' is added nowhere in non-test code. client.rs has the one constant, early.rs refers to it, and the new classifier and optional-failure reader use it. No second list of names is added. Run: `git diff -- crates/athena-core/src/gh/client.rs crates/athena-core/src/gh/early.rs`. Observes: The `EARLY_RUN_JOB` definition becomes a reference to `CI_AGGREGATE_JOB`. No new string literal of the check name appears outside tests.
- Criterion 8: Both changes only narrow refusals. They touch the landing wait and the pre-merge classification, not a reading parser, a stem list or a stop list. No refusal rule is added, so no new corpus case is needed. I did not read crates/athena-core/tests/refusal_corpus.rs, so I cite no case id. The producer and the verifier run the corpus to confirm no reading refusal changed. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: The corpus test passes unchanged, so no reading is newly refused.

Planned files:

- crates/athena-core/src/gh/client.rs (modify)
- crates/athena-core/src/gh/early.rs (modify)
- crates/athena-core/src/gh/landing.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)

## Replay of accepted readings

no replay: a check failed

## The change

Base a74df6a991f8 → candidate 470b06497a0b. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  16 +++
 crates/athena-core/src/gh/client.rs     | 134 ++++++++++++++-------
 crates/athena-core/src/gh/early.rs      |   5 +-
 crates/athena-core/src/gh/landing.rs    |  10 +-
 crates/athena-core/src/gh/mod.rs        |   4 +-
 crates/athena-core/src/gh/tests.rs      | 200 ++++++++++++++++++++++++++++++--
 crates/athena-core/src/land.rs          |   4 +-
 crates/athena-gate/src/invariant.rs     |  30 +++--
 crates/athena-gate/src/landing_tests.rs |  69 +++++++++++
 docs/decisions.md                       |  33 ++++++
 10 files changed, 441 insertions(+), 64 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41324232864 for 470b06497a0b2f420383dfec60509f4b5e0b7f6f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41324232864 for 470b06497a0b2f420383dfec60509f4b5e0b7f6f)
- test: `cargo test --locked` failed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41324232864 for 470b06497a0b2f420383dfec60509f4b5e0b7f6f)
- ci-result: `not a check` passed (Check results for 470b06497a0b2f420383dfec60509f4b5e0b7f6f come from the CI run https://github.com/doodla/athena/actions/runs/41324232864 (580s): fmt success, clippy success, test failure.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test, cargo clippy --all-targets and cargo fmt --check pass for the workspace.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
