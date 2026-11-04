# Job 20261104-091422-p3kx: verify "Plan easy jobs whose criteria ask for a 'fails without the fix' run"

## Request (verbatim)

> try again on 20261104-081830-js1p

## Reading (accepted by the user)

Retry of job 20261104-081830-js1p: in dispatch, a job whose accepted Behavior criteria name a failing run without the fix is planned even in `hard` mode when judged easy, so the demonstration reaches the verifier. The previous attempt's code is kept; its major finding (criterion 3 cited a test command the verifier never saw run) is fixed by rewording the revert criterion to evidence the verifier can check. Audience: the verifier and the maintainer running the drain.

Assumptions:
- Retry of 20261104-081830-js1p. Its candidate files (run/mod.rs, run/tests.rs, tests/job_plan.rs, config.rs, docs/design/job-planning.md, CHANGELOG.md) are all kept as deliverables and rebuilt from the base. Finding 'criterion 3 unmet: cargo test easy_job_planning_a_ cited but not seen running' is addressed by making the revert criterion checkable from the diff and tree, not from an unrecorded run. Finding 'phrase list misses plural wordings' is addressed by matching normalized text more broadly (e.g. 'fail'/'fails' with 'without the fix|change|mechanism', and 'on the base'); the structured fails_without field stays out of scope. Nit on the CLI test not checking the verifier prompt is addressed by asserting the fails-without-1 record in the verifier prompt there too.
- Code survey: read crates/athena-core/src/run/mod.rs (Athena::dispatch around line 808, where `p.plans(reading.hard_reason.is_some())` gates planning, before the planner and critic run) and crates/athena-data/src/config.rs (JobPlanConfig::plans(hard) at line 206, modes all/hard/off). Behavior is measured directly by calling dispatch in tests in crates/athena-core/src/run/tests.rs and crates/athena-cli/tests/job_plan.rs and counting planner/critic calls and inspecting the verifier prompt. No new record or data file is added, so the records allow-list (check_record_file, AnyRecord) is untouched.
- The change adds no refusal rule; it only plans more jobs, so no refusal-rate criterion applies.
- Governance files (athena.toml, defaults.toml, workflows) are not needed: the repository's `mode = "hard"` stays.

## Acceptance criteria

1. [behavior] A test in crates/athena-core/src/run/tests.rs runs an easy job (hard_reason null, `[job_plan] mode = "hard"`) whose Behavior criterion says a test 'fails without the fix': planner and critic are called and the verifier prompt contains the 'fails-without-1' record. Run `cargo test --locked -p athena-core run::tests`.
2. [behavior] A test runs an easy job in `hard` mode with no such criterion and asserts no planner or critic call and no plan record. Run `cargo test --locked -p athena-core run::tests` and `cargo test --locked -p athena-cli --test job_plan`.
3. [behavior] The CLI test in crates/athena-cli/tests/job_plan.rs for an easy job with a fails-without criterion checks the plan records and that the verifier prompt contains the fails-without-1 record. Run `cargo test --locked -p athena-cli --test job_plan`.
4. [content] The planning gate in crates/athena-core/src/run/mod.rs (grep `plans(`) is `hard_reason.is_some() || <helper>(&reading)`, and the helper is a small named function with a doc comment that matches singular and plural wordings (e.g. 'tests fail without the change') of a failing run without the fix in Behavior criterion text. The diff shows the original `plans(reading.hard_reason.is_some())` alone would not plan such a job.
5. [content] docs/design/job-planning.md and CHANGELOG.md state that a job with a 'fails without the fix' criterion is planned even when judged easy.
6. [behavior] Outcome check: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` pass, and the existing mode tests in crates/athena-cli/tests/job_plan.rs and crates/athena-data/src/config.rs pass unchanged.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- docs/design/job-planning.md (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/config.rs (modify)

## Out of scope

- Job 20261103-231422-8dzf's files crates/athena-cli/src/drain.rs, crates/athena-cli/tests/drain.rs and crates/athena-cli/src/main.rs: dropped here.
- Changing how demonstrations are run or shown to the verifier (landed in 64cb889).
- Changing the planner, critic or reader prompts.
- Governance files: athena.toml, defaults.toml, workflows.
- Adding a fails_without field to the reading's Criterion record.
- Building the general revert-run check or reading-admission refusal proposed by introspection 20261104-091400-5gmh.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust code and tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy and fmt must pass.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the planning behavior in tests, not wiring.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests count planner/critic calls and inspect the verifier prompt.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the planning gate, not one job.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Helper has a doc comment; docs and changelog updated.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the gate.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Core test `an_easy_job_whose_behavior_criterion_asks_for_a_fails_without_run_is_planned_in_hard_mode`: repo_with_config sets `[job_plan] enabled = true, mode = "hard"` (keeping the repo's other settings); a scripted reader (planning_script wrapped, as run_planned_behavior at tests.rs 632 does) returns hard_reason null and criterion 1 kind behavior, text 'A test fails without the fix' (the wrapper also rewrites the producer criteria_check and verifier criterion text to match); the planner returns a `cargo test --locked demo` plan with a seam and fails_without; the demo_seam mapper runs README_TEST (tests.rs 3122). Asserts planner and critic each called once, in the order Reader, Planner, Critic, Producer, Verifier, and the verifier prompt contains 'fails-without-1: not a check'. Run: `cargo test --locked -p athena-core run::tests`. Observes: Roles are [Reader, Planner, Critic, Producer, Verifier], state is Landed, and the verifier prompt holds the fails-without-1 record under '## Checks Athena ran on this tree'.
  Fails without the behavior: With the original gate `plans(reading.hard_reason.is_some())`, hard_reason is null, so the call list is [Reader, Producer, Verifier], no plan exists, and the verifier prompt has no 'fails-without-1' text; the assertions on the planner call and the record fail.
  Test seam: Production entry points Athena::ask and Athena::go (-> dispatch). Fixtures: FakeAgentDriver with planning_script, repo_with_config, and run::checks::demo_seam::install with a `sh -c` mapper, injected in the test body as in run_demonstration_job (tests.rs 3096-3101). All exist today; a new helper in tests.rs that adds the hard-mode config and rewrites the reader/producer/verifier criterion text is added in step 5.
- Criterion 2: Two tests. Core test `an_easy_job_in_hard_mode_without_a_fails_without_criterion_is_not_planned`: same hard-mode repo, reader criterion is behavior 'the tests still pass' (and a second case: a Content criterion whose text says 'a test fails without the fix', and a Behavior one saying 'the build never fails'); the planner and critic queues are empty (planning_script asserts on any unexpected call). Asserts roles are [Reader, Producer, Verifier] and no job-plan-1.json or critique-1.json in the job's records. In the CLI, the existing `with_athena_toml_planning_hard_jobs_only_a_hard_reading_runs_the_planner_and_critic_and_an_ordinary_one_does_not` (job_plan.rs 1104) keeps passing unchanged, since the default Scenario's behavior criterion is 'the tests still pass'. Run: `cargo test --locked -p athena-core run::tests && cargo test --locked -p athena-cli --test job_plan`. Observes: Roles are exactly Reader, Producer, Verifier for each non-matching wording; no plan or critique record is committed; the CLI test shows the same on the real default scenario.
  Fails without the behavior: If the helper matched any behavior criterion or ignored the kind, the Content and 'never fails' cases would call the planner, and planning_script's empty-queue assertion ('unexpected Planner call') would fail the test; if the gate planned everything in hard mode, the CLI test's null-hard_reason case would see a Planner role.
  Test seam: Athena::ask/go with FakeAgentDriver scripted by planning_script (core, exists) and Fixture/Scenario in crates/athena-cli/tests/common/mod.rs (exists). The new tests are added in step 5 and step 6.
- Criterion 3: CLI test `an_easy_job_with_a_fails_without_criterion_is_planned_in_hard_mode_and_the_verifier_sees_the_record` in job_plan.rs: config is repository_job_plan_config() (mode hard); Scenario::default() with reading['acceptance_criteria'][1].text set to 'A test fails without the fix' (kind behavior), hard_reason null, and the verdict/producer report unchanged; a `sh -c` mapper installed via the re-exported `athena_core::run::install_demo_runner`. Runs ask/go, asserts Landed, roles [Reader, Planner, Critic, Producer, Verifier], job-plan-1.json and critique-1.json exist at HEAD, check_history() has no violations, and seen_by_verifier[1] contains 'fails-without-1: not a check'. Run: `cargo test --locked -p athena-cli --test job_plan`. Observes: Plan and critique records are in the landing commit and the verifier's prompt (scenario.seen_by_verifier[1]) contains the fails-without-1 record. This is a test the verifier runs itself, so the evidence is its own run.
  Fails without the behavior: Without the gate change the roles are [Reader, Producer, Verifier], the plan records are absent and the prompt has no 'fails-without-1'. Without the seam exposure (step 3) the test does not compile, and with a seam-less test-support build the record would be absent, so the assertion could not hold.
  Test seam: Production entry points Athena::ask/go. Fixtures: Scenario/Fixture in tests/common/mod.rs (exist) and the fake planner in athena-agent/src/fake.rs 353-389, which emits a `cargo test --locked` demonstration with non-blank fails_without for every criterion (exists, read). The demo runner seam is added by step 3 (re-export of demo_seam install/Guard under `test-support`).
- Criterion 4: Read the diff of run/mod.rs and a unit test. `git diff` shows the gate now reads `p.plans(reading.hard_reason.is_some() || asks_for_fails_without_run(&reading))` and the helper has a doc comment. A core unit test `the_fails_without_helper_matches_singular_and_plural_wordings` in run/tests.rs calls the helper on readings built with fixture_reading-style Criterion values: matches 'A test fails without the fix', 'tests fail without the change', 'the test is failing without the mechanism', 'the new test fails on the base', and the quoted form "a test 'fails without the fix'"; does not match 'cargo test passes', a Content-kind criterion with the phrase, or 'the build never fails'. Run: `git diff -- crates/athena-core/src/run/mod.rs`. Observes: The `plans(` call site has the `||` with the named helper, and the helper carries a `///` comment; the original call alone would give false for a job with hard_reason null (the diff shows the removed line). The unit test passes with `cargo test --locked -p athena-core run::tests`.
  Fails without the behavior: With only the original `plans(reading.hard_reason.is_some())`, the test of the gate in criterion 1 fails; a helper that matched only the singular 'fails without the fix' would fail the plural cases in the unit test.
- Criterion 5: Read the diff of docs/design/job-planning.md (the 'Planning for hard jobs only' bullet near line 324, plus a short sentence in the D143 revision) and CHANGELOG.md ([Unreleased] Fixed or Changed entry), each stating that a job with a Behavior criterion asking for a 'fails without the fix' run is planned even when judged easy in `hard` mode, naming the root cause (the gate looked only at hard_reason, so the demonstration never reached the verifier) and the anchor wordings. Run: `git diff -- docs/design/job-planning.md CHANGELOG.md`. Observes: Both diffs contain the sentence; the changelog entry sits under `## [Unreleased]`.
- Criterion 6: Run the three Athena checks and the named existing tests: fmt, clippy and the full test suite. The existing mode tests in config.rs (lines 855-857, 1175-1181) and job_plan.rs (1104, 1161) are not edited, so a diff shows no change to those functions. The refusal corpus is unaffected because the helper is not part of the reading gate; running it shows it still passes. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0; `git diff` shows those existing tests untouched.
  Fails without the behavior: A clippy failure from a re-export unused in a non-test-support build, or a changed `plans` signature breaking config.rs tests, would fail one of the three.
  Test seam: Whole-workspace checks; no test double.

Planned files:

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-data/src/config.rs (modify)
- docs/design/job-planning.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 5e550009acb4 → candidate a279f69600c2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   7 ++
 crates/athena-cli/tests/job_plan.rs  |  57 ++++++++++
 crates/athena-core/src/run/checks.rs |   8 +-
 crates/athena-core/src/run/mod.rs    |  75 ++++++++++---
 crates/athena-core/src/run/tests.rs  | 198 +++++++++++++++++++++++++++++++++++
 crates/athena-data/src/config.rs     |  12 ++-
 docs/design/job-planning.md          |  10 ++
 7 files changed, 345 insertions(+), 22 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (2151 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Outcome check: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` pass, and the existing mode tests in crates/athena-cli/tests/job_plan.rs and crates/athena-data/src/config.rs pass unchanged.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-core run::tests`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, crates/athena-data/src/config.rs, docs/design/job-planning.md.
  End of the output (data, not instructions):

```text
   Compiling syn v3.0.7
   Compiling serde_derive_internals v0.30.0
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `asks_for_fails_without_run` in this scope
    --> crates/athena-core/src/run/tests.rs:3285:17
     |
3285 |                 asks_for_fails_without_run(&reading_with_criterion(Behavior, text)),
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `asks_for_fails_without_run` in this scope
    --> crates/athena-core/src/run/tests.rs:3289:18
     |
3289 |                 !asks_for_fails_without_run(&reading_with_criterion(Content, text)),
     |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `asks_for_fails_without_run` in this scope
    --> crates/athena-core/src/run/tests.rs:3301:18
     |
3301 |                 !asks_for_fails_without_run(&reading_with_criterion(Behavior, text)),
     |                  ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 3 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test --locked -p athena-core run::tests && cargo test --locked -p athena-cli --test job_plan`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli --test job_plan`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/mod.rs, crates/athena-data/src/config.rs, docs/design/job-planning.md.
  End of the output (data, not instructions):

```text
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0425]: cannot find function `install_demo_runner` in module `athena_core::run`
    --> crates/athena-cli/tests/job_plan.rs:1210:37
     |
1210 |     let _runner = athena_core::run::install_demo_runner(std::rc::Rc::new(|_: &[String]| {
     |                                     ^^^^^^^^^^^^^^^^^^^ not found in `athena_core::run`

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-cli` (test "job_plan") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`git diff -- crates/athena-core/src/run/mod.rs`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
