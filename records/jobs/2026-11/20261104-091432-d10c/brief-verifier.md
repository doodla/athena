# Job 20261104-091432-d10c: verify "Count recorded fails-without revert runs as commands seen running"

## Request (verbatim)

> Count Athena's 'fails without the fix' runs as commands seen running. Since 64cb889 Athena runs each planned revert demonstration and shows it to the verifier under 'Checks Athena ran on this tree', but the run is not written to verification.json checks, so the verdict's seen-running rule (D43, crates/athena-core/src/verify.rs) does not count it. Job 20261104-081830-js1p was rejected only on this: the verifier quoted Athena's revert record fails-without-1 (both new tests failed on the base code, exit 101) and the criterion still came out unmet with commands_verified false. Record each demonstration run in the verification record's checks (name, command, exit code, output tail, marked as a revert run), and let a behavior criterion that cites that record's command, or its record name, count as seen running for the 'fails without the fix' part. Acceptance: a test where a criterion citing a recorded revert run is met with commands_verified true; a test where a criterion citing a command neither run nor recorded stays unseen.

## Reading (accepted by the user)

Retry of job 20261104-081830-js1p's follow-up need: Athena's revert demonstration (fails-without-N) is shown to the verifier but not recorded in verification.json checks, so judge() in verify.rs does not count it as seen running. Record each run as a check (name, command, exit code, output tail, marked as a revert run) and let a behavior criterion citing its command or record name count as seen running. The audience is the verifier verdict and the maintainers reading verification.json.

Assumptions:
- Prior job 20261104-081830-js1p (rejected only on its criterion 3 'cited but not seen running'): its easy-job-planning work is not addressed here and is treated as already handled elsewhere. The repository now has asks_for_fails_without_run in crates/athena-core/src/run/mod.rs, so its minor and nit findings no longer apply to this request. Its candidate files CHANGELOG.md, crates/athena-cli/tests/job_plan.rs, run/mod.rs, run/tests.rs, athena-data config.rs and docs/design/job-planning.md are not deliverables here, except run/tests.rs, which is kept as a deliverable for the new tests.
- Code survey: crates/athena-core/src/verify.rs judge() builds `ran` from commands_run plus passing checks, but only when candidate_tree equals checks_tree. It then marks a behavior criterion commands_verified only if every cited command is in `ran`. crates/athena-core/src/run/checks.rs (around lines 327-354) runs each demonstration as a CheckSpec named fails-without-N and returns only a CheckResultView for the brief. verify.rs demonstration_view and demonstration_entry (around lines 975-1035) build that view with command 'not a check'. crates/athena-data/src/records.rs CheckRun (line 785) is the record type in verification.json. It gets a new serde-default, skip-if-false field such as `revert: bool`, like `reused`. This is an optional field on an existing record, so no new record file is added. The records round-trip tests in records.rs and check_record_file cover CheckRun. All CheckRun literals (checks.rs, status.rs, introspect.rs, routing.rs, reading.rs, send_back.rs, gh/tests.rs, cli tests) need the new field.
- Seen-running measurement: a revert run is recorded with its demonstration command (the group's argv text), exit code and output tail, and marked as a revert run. Its `passed` means the test failed as the criterion expected, so judge() adds its command and its record name to `ran` only for a revert-marked record whose run happened, regardless of the exit code being nonzero. A recorded run that did not happen ('not run') is never counted.
- The change is judged only on the existing tree equality rule: the revert record belongs to the same candidate tree. A cited name such as `fails-without-1` is accepted as a cited command for a behavior criterion only when a revert record of that name exists.
- Existing wiring in run/verification.rs and run/checks.rs is reused; how demonstrations are chosen, run or shown to the verifier (64cb889) is unchanged except that the run is also returned as a CheckRun.

## Acceptance criteria

1. [behavior] A judge() test in crates/athena-core/src/verify.rs: with a recorded revert-marked CheckRun (name fails-without-1, exit code 101) for the candidate tree, a behavior criterion citing that run's command is met with commands_verified true. Run `cargo test --locked -p athena-core verify`.
2. [behavior] A judge() test where the criterion cites the record name fails-without-1 gets commands_verified true and met true, and no 'cited but not seen running' finding appears. Run `cargo test --locked -p athena-core verify`.
3. [behavior] A judge() test where a behavior criterion cites a command that was neither run nor recorded (including a revert-marked record that did not run, or one from a different tree) stays commands_verified false, is forced unmet, and gets the Major 'cited but not seen running' finding. Run `cargo test --locked -p athena-core verify`.
4. [behavior] A run-level test in crates/athena-core/src/run/tests.rs shows the demonstration run is written to the verification record's checks with name, command, exit code, output tail and the revert marker, in the same pass that shows it to the verifier. Run `cargo test --locked -p athena-core run::`.
5. [behavior] CheckRun with the new field round-trips, and an older verification.json without the field still loads, with the marker false. Run `cargo test --locked -p athena-data`.
6. [behavior] The whole workspace still passes. Run `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check`.
7. [content] CHANGELOG.md has an entry, and a design doc (docs/design/job-planning.md or the one that covers D43) states in prose that recorded revert runs count as seen running for the 'fails without the fix' part only.
8. [content] Outcome check: a job like 20261104-081830-js1p, whose verifier cites the revert record fails-without-1, would get commands_verified true. This is shown by the test in the first criterion, which builds the same input shape (exit 101, failing on base).

## Expected deliverables

- crates/athena-core/src/verify.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-cli/tests/status_durations.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- Job 20261104-081830-js1p's candidate files crates/athena-cli/tests/job_plan.rs and crates/athena-data/src/config.rs: that work is dropped here, and so is its planning-gate change, which is not part of this request.
- Changing how demonstrations are chosen, run or shown to the verifier (landed in 64cb889).
- Counting any other non-command record as seen running, or loosening the D43 rule for criteria that do not cite a revert run.
- Changing the planner, critic, reader or verifier prompts.
- Governance files: athena.toml, defaults.toml, workflows.
- Adding a fails_without field to the reading's Criterion record.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests plus cargo test, clippy and fmt checks pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run judge() and the record round-trip tests, not wiring existence.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that a verifier citing the recorded revert run gets commands_verified true.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix counts the recorded run in the verdict rule, not just the one rejected job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced in judge() code with tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the record and the verdict rule.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and changelog written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test in verify.rs: a revert record (name fails-without-1, command cargo test --locked -p athena-core verify, exit 101, passed true, revert_for [2]) with candidate_tree and checks_tree both 'abc'; criteria[1] (number 2) cites the command text. The input shape of job 20261104-081830-js1p. Run: `cargo test --locked -p athena-core verify`. Observes: criteria[1].commands_verified and met are true, and no 'cited but not seen running' finding appears.
  Fails without the behavior: Without the per-criterion rule, exit 101 fails the `exit_code == Some(0)` filter (verify.rs 378), the command is unseen, commands_verified is false, met is forced false and the Major D43 finding appears.
  Test seam: Entry point judge() in crates/athena-core/src/verify.rs, called through the existing test helper judge_recorded (it takes recorded checks, candidate and checks trees, and cited commands). Fixture: a CheckRun literal from a new `revert_check(exit, for_criteria)` helper added in plan step 10. judge_recorded exists; the rule under test runs in judge, not in the fixture.
- Criterion 2: Unit test: the same fixture, criterion number 2 cites the record name 'fails-without-1'; both trees 'abc'. Run: `cargo test --locked -p athena-core verify`. Observes: commands_verified true, met true, and d43_finding(&j) is None.
  Fails without the behavior: The name is in neither commands_run nor any check command, so it is unseen; the criterion is forced unmet with the Major finding.
  Test seam: judge() via judge_recorded (exists) with the revert_check helper from step 10.
- Criterion 3: Unit tests, each ending with commands_verified false, met forced false and a Major 'cited but not seen running: `...`' finding at Severity::Major: (a) a command nobody ran or recorded; (b) a revert record with exit_code None, cited by command and by name; (c) the right record with checks_tree 'def', and one with None, while candidate_tree is 'abc'; (d) a name no record carries; (e) revert_for [3] with exit 101, criterion number 2 cites its command and name; (f) the finding-2 case: revert record with exit_code Some(0) and revert_for [3], cited by criterion 2 by command and by name, stays unseen. Positive companions: criterion 3 citing the [3] record is seen, and an exit-0 record with revert_for [2] is seen for criterion 2 (documented behavior). Run: `cargo test --locked -p athena-core verify`. Observes: Cases (a)-(f) give commands_verified false and the Major finding with the cited text; the companions give commands_verified true.
  Fails without the behavior: (e) fails if the rule is job-wide. (f) fails if the revert record reaches the job-wide `ran` through the filter at verify.rs 378, i.e. without the `continue` guard. (b) and (c) fail if the rule ignores the exit-code test or the tree gate. (a) and (d) already hold today; they guard against over-counting.
  Test seam: judge() via judge_recorded (exists) with revert_check from step 10. The tree gate is exercised through judge_recorded's existing checks_tree parameter.
- Criterion 4: Run-level tests in run/tests.rs. Test A: run_demonstration_job(repo_with_planning(3), demonstration_plan("cargo test --locked demo"), Some(README_TEST), None), then read job.finished.verification.unwrap().checks and find fails-without-1. Test B: the same with 'cargo test --locked && echo done' (not a plain command) stores no revert record. Test C: new helper run_citing_job("cargo test --locked demo") (reader criterion kind behavior, text 'A test fails without the fix', planner queue demonstration_plan_with_seam(..), the fake verifier's criteria[0].commands set to the cited text, commands_run empty), asserting the stored criterion is met with commands_verified true. Run: `cargo test --locked -p athena-core run::`. Observes: A: stored checks hold fails-without-1 with revert_for == [1], passed true, command [cargo,test,--locked,demo], exit_code Some(1) and an output_tail containing 'test demo ... FAILED'; the verifier prompt of the same job still contains 'fails-without-1: not a check' exactly once. B: no stored check has a non-empty revert_for and the prompt says 'not run'. C: the stored criterion is met and commands_verified is true, with no 'cited but not seen running' finding.
  Fails without the behavior: Without the change to demonstration_records and verification_stage, the stored checks lack the record and A's first assertion fails. Without the judge() rule, C comes out unmet with commands_verified false. An off-by-one between group criteria and judge's index+1 fails C.
  Test seam: Production entry point: Athena dispatch through the existing helpers run_demonstration_job (run/tests.rs 3069) and finished_job (1216). Fakes: the scripted agents (planning_script, base_script) and demo_seam::install (run/checks.rs, test-only), which only maps the argv to `sh -c`; scratch copy, revert, process and record building are production code. These exist. Step 11 adds run_citing_job, which wraps the script closure as run_hard_mode_job (3340-3363) does; the verdict logic stays in production judge().
- Criterion 5: Unit tests in records.rs: a CheckRun with revert_for [1, 2] serialises with the key, parses back equal, and validates inside a Baseline via AnyRecord::parse and check_record_file (model: a_reused_check_record_round_trips_and_validates). Another test serialises an ordinary record and asserts no 'revert_for' key, and parses JSON written without the field (model: a_check_run_written_before_reuse_was_marked_loads_as_not_reused). Run: `cargo test --locked -p athena-data`. Observes: The revert record round-trips and validates; the old shape loads with an empty revert_for; an ordinary record serialises as before.
  Fails without the behavior: Without #[serde(default, skip_serializing_if = "Vec::is_empty")], the old JSON fails to parse and the ordinary record gains a key.
  Test seam: CheckRun serde, AnyRecord::parse and check_record_file in crates/athena-data/src/records.rs; all exist, step 2 adds the tests.
- Criterion 6: Run the workspace gates, which include the admission test ci_reuse_replay_the_seen_commands_filter_turns_up_in_a_search_and_must_be_named (admission.rs 1078-1143, finds the unchanged filter line) and the refusal corpus. Run: `cargo test --locked && cargo clippy --locked --all-targets -- -D warnings && cargo fmt --check`. Observes: All tests pass, no clippy warning, no fmt diff.
  Fails without the behavior: A missed `revert_for: Vec::new()` in any CheckRun literal fails the build; editing the filter line fails the admission test.
  Test seam: Full workspace runs; no seam.
- Criterion 7: Read the diff of the two docs. CHANGELOG.md [Unreleased] Fixed: root cause (the revert run was shown but not recorded, so D43 never saw it), the fix, per-criterion scope, exit-0 behavior, the `passed` meaning and its departure from the reading's wording, and the PR body count. docs/design/job-planning.md: a paragraph after the 'fails without the fix' text (lines 332-341) saying in prose that a recorded revert run counts as seen running, by command or record name, only for the criteria whose planned demonstration names it; never for other criteria, never for a run that did not finish or is from another tree; a run that still passed without the fix counts for its own criteria and the verifier judges it; and within one criterion the code cannot tell the 'fails without' half from the 'passes with' half. Run: `git diff -- CHANGELOG.md docs/design/job-planning.md`. Observes: The Unreleased entry exists and the design paragraph states the scope that the criterion-3 tests enforce.
- Criterion 8: Outcome check: the tests of criteria 1 and 2 build the input shape of job 20261104-081830-js1p (revert fails-without-1, exit 101, cited by command and by name by a criterion in its group); criterion 4's run-level test C shows production writes that shape and the stored criterion is met. Run: `cargo test --locked -p athena-core verify`. Observes: commands_verified true and no D43 finding for that shape; before the change the same input was unmet with commands_verified false.
  Fails without the behavior: Without the rule the same input is unmet with commands_verified false and the Major finding.
  Test seam: judge() via judge_recorded (exists) with revert_check (step 10); the production path is shown by run_citing_job (step 11).

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run/close.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/gh/tests.rs (modify)
- crates/athena-cli/tests/status_durations.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base c4ee40199ec5 → candidate 9910e19f339c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  13 ++
 crates/athena-cli/tests/status_durations.rs |   1 +
 crates/athena-core/src/checks.rs            |   7 +
 crates/athena-core/src/introspect.rs        |   1 +
 crates/athena-core/src/reading.rs           |  24 +++
 crates/athena-core/src/routing.rs           |   1 +
 crates/athena-core/src/run/checks.rs        |  47 +++--
 crates/athena-core/src/run/close.rs         |  39 ++++-
 crates/athena-core/src/run/mod.rs           |   1 +
 crates/athena-core/src/run/send_back.rs     |   1 +
 crates/athena-core/src/run/tests.rs         |  92 +++++++++-
 crates/athena-core/src/run/verification.rs  |  12 +-
 crates/athena-core/src/status.rs            |   1 +
 crates/athena-core/src/verify.rs            | 259 +++++++++++++++++++++++++++-
 crates/athena-data/src/records.rs           |  43 +++++
 docs/design/job-planning.md                 |  12 ++
 16 files changed, 524 insertions(+), 30 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41823296862 for 9910e19f339c0d428127307ba189abaf995e0b71)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41823296862 for 9910e19f339c0d428127307ba189abaf995e0b71)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41823296862 for 9910e19f339c0d428127307ba189abaf995e0b71)
- ci-result: `not a check` passed (Check results for 9910e19f339c0d428127307ba189abaf995e0b71 come from the CI run https://github.com/doodla/athena/actions/runs/41823296862 (458s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: The whole workspace still passes. Run `cargo test --locked`, `cargo clippy --locked --all-targets -- -D warnings` and `cargo fmt --check`.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 2, 3, 8 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-core verify`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/close.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/status.rs, crates/athena-core/src/verify.rs, crates/athena-data/src/records.rs, docs/design/job-planning.md.
  End of the output (data, not instructions):

```text
     |
3508 |         assert!(checks.iter().all(|c| c.revert_for.is_empty()), "{checks:?}");
     |                                         ^^^^^^^^^^ unknown field
     |
     = note: available fields are: `name`, `command`, `exit_code`, `passed`, `tests_ran` ... and 5 others

error[E0560]: struct `athena_data::records::CheckRun` has no field named `revert_for`
    --> crates/athena-core/src/status.rs:2857:13
     |
2857 |             revert_for: Vec::new(),
     |             ^^^^^^^^^^ `athena_data::records::CheckRun` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::CheckRun` has no field named `revert_for`
    --> crates/athena-core/src/verify.rs:1456:13
     |
1456 |             revert_for: Vec::new(),
     |             ^^^^^^^^^^ `athena_data::records::CheckRun` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::CheckRun` has no field named `revert_for`
    --> crates/athena-core/src/verify.rs:1988:13
     |
1988 |             revert_for: for_criteria.to_vec(),
     |             ^^^^^^^^^^ `athena_data::records::CheckRun` does not have this field
     |
     = note: available fields are: `duration`, `note`, `failed_tests`, `reused`

error[E0425]: cannot find function `revert_record` in this scope
    --> crates/athena-core/src/verify.rs:2163:22
     |
2163 |         let record = revert_record(&group, &run);
     |                      ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `demonstration_name` in this scope
    --> crates/athena-core/src/verify.rs:2165:33
     |
1013 | / fn demonstration_entry(
1014 | |     group: &DemonstrationGroup,
1015 | |     result: String,
1016 | |     exit_code: Option<i32>,
...    |
1040 | | }
     | |_- similarly named function `demonstration_entry` defined here
...
2165 |           assert_eq!(record.name, demonstration_name(&group));
     |                                   ^^^^^^^^^^^^^^^^^^
     |
help: a function with a similar name exists
     |
2165 -         assert_eq!(record.name, demonstration_name(&group));
2165 +         assert_eq!(record.name, demonstration_entry(&group));
     |

Some errors have detailed explanations: E0425, E0432, E0560, E0609.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 20 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-core run::`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/close.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/status.rs, crates/athena-core/src/verify.rs, crates/athena-data/src/records.rs, docs/design/job-planning.md.
  End of the output (data, not instructions):

```text
    --> crates/athena-core/src/run/tests.rs:3508:41
     |
3508 |         assert!(checks.iter().all(|c| c.revert_for.is_empty()), "{checks:?}");
     |                                         ^^^^^^^^^^ unknown field
     |
     = note: available fields are: `name`, `command`, `exit_code`, `passed`, `tests_ran` ... and 5 others

error[E0560]: struct `athena_data::records::CheckRun` has no field named `revert_for`
    --> crates/athena-core/src/status.rs:2857:13
     |
2857 |             revert_for: Vec::new(),
     |             ^^^^^^^^^^ `athena_data::records::CheckRun` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::CheckRun` has no field named `revert_for`
    --> crates/athena-core/src/verify.rs:1456:13
     |
1456 |             revert_for: Vec::new(),
     |             ^^^^^^^^^^ `athena_data::records::CheckRun` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::CheckRun` has no field named `revert_for`
    --> crates/athena-core/src/verify.rs:1988:13
     |
1988 |             revert_for: for_criteria.to_vec(),
     |             ^^^^^^^^^^ `athena_data::records::CheckRun` does not have this field
     |
     = note: available fields are: `duration`, `note`, `failed_tests`, `reused`

error[E0425]: cannot find function `revert_record` in this scope
    --> crates/athena-core/src/verify.rs:2163:22
     |
2163 |         let record = revert_record(&group, &run);
     |                      ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `demonstration_name` in this scope
    --> crates/athena-core/src/verify.rs:2165:33
     |
1013 | / fn demonstration_entry(
1014 | |     group: &DemonstrationGroup,
1015 | |     result: String,
1016 | |     exit_code: Option<i32>,
...    |
1040 | | }
     | |_- similarly named function `demonstration_entry` defined here
...
2165 |           assert_eq!(record.name, demonstration_name(&group));
     |                                   ^^^^^^^^^^^^^^^^^^
     |
help: a function with a similar name exists
     |
2165 -         assert_eq!(record.name, demonstration_name(&group));
2165 +         assert_eq!(record.name, demonstration_entry(&group));
     |

Some errors have detailed explanations: E0425, E0432, E0560, E0609.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 20 previous errors
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-data`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/checks.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/checks.rs, crates/athena-core/src/run/close.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/status.rs, crates/athena-core/src/verify.rs, crates/athena-data/src/records.rs, docs/design/job-planning.md.
  End of the output (data, not instructions):

```text
   Compiling libc v0.2.190
   Compiling syn v3.0.7
   Compiling serde_json v1.0.151
   Compiling serde v1.0.229
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling winnow v0.7.15
   Compiling getrandom v0.3.4
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling toml v0.9.12+spec-1.1.0
   Compiling thiserror v2.0.21
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
error[E0560]: struct `records::CheckRun` has no field named `revert_for`
    --> crates/athena-data/src/records.rs:4491:13
     |
4491 |             revert_for: Vec::new(),
     |             ^^^^^^^^^^ `records::CheckRun` does not have this field
     |
     = note: all struct fields are already assigned

error[E0609]: no field `revert_for` on type `records::CheckRun`
    --> crates/athena-data/src/records.rs:4543:24
     |
4543 |         assert!(parsed.revert_for.is_empty());
     |                        ^^^^^^^^^^ unknown field
     |
     = note: available fields are: `name`, `command`, `exit_code`, `passed`, `tests_ran` ... and 5 others

error[E0609]: no field `revert_for` on type `records::CheckRun`
    --> crates/athena-data/src/records.rs:4553:21
     |
4553 |         b.checks[1].revert_for = vec![1, 2];
     |                     ^^^^^^^^^^ unknown field
     |
     = note: available fields are: `name`, `command`, `exit_code`, `passed`, `tests_ran` ... and 5 others

error[E0609]: no field `revert_for` on type `records::CheckRun`
    --> crates/athena-data/src/records.rs:4563:32
     |
4563 |         assert!(back.checks[0].revert_for.is_empty());
     |                                ^^^^^^^^^^ unknown field
     |
     = note: available fields are: `name`, `command`, `exit_code`, `passed`, `tests_ran` ... and 5 others

error[E0609]: no field `revert_for` on type `records::CheckRun`
    --> crates/athena-data/src/records.rs:4564:35
     |
4564 |         assert_eq!(back.checks[1].revert_for, [1, 2]);
     |                                   ^^^^^^^^^^ unknown field
     |
     = note: available fields are: `name`, `command`, `exit_code`, `passed`, `tests_ran` ... and 5 others

Some errors have detailed explanations: E0560, E0609.
For more information about an error, try `rustc --explain E0560`.
error: could not compile `athena-data` (lib test) due to 5 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test --locked && cargo clippy --locked --all-targets -- -D warnings && cargo fmt --check`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
