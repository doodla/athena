# Job 20261107-144804-5vht: verify "Require a measured stage timeline in readings of speed or delay requests"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The request named a fix (move the call earlier) and a timing figure, but nothing required the readin
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: speed-change-requested-without-checking-measured-stage-timeline (20261025-110542-wa68)
> Cause: The request named a fix (move the call earlier) and a timing figure, but nothing required the reading or planner to check that figure against the real stage order and durations before planning around it. The class already exists for a speed...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261025-110857-eqhd (job 20261025-110525-8hzb); prevention it proposed: For any request that claims a delay or speed-up, the reading stage must first produce a timeline from the code and the recorded run timings: each stage in order, its measured duration, and where the claimed delay sits. The reading must stat...
> Introspection 20261025-183009-xw7y (job 20261025-034733-c453); prevention it proposed: Add a measure-first step for any request to speed up a pipeline or check. Before the reading is accepted, Athena runs or reads a timing breakdown (compile versus run, per test binary or stage) and puts it in the reading. The reading must th...
> Introspection 20261025-190659-jvd7 (job 20261025-190606-eebk); prevention it proposed: Add a required measurement step to any request or reading whose goal is speed. Before the reading fixes a method, it must record a per-stage or per-binary timeline from a real run. It must also try each candidate lever, such as a different ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: no reading rule makes a request that claims a delay or speed-up (jobs 20261025-110525-8hzb, 20261025-034733-c453, 20261025-190606-eebk) be checked against the real stage order and measured durations. I will add a refusal check, `check_timing_claim`, beside `check_code_survey` and `check_records_measure`. It refuses a speed or delay reading that has no `Timeline:` assumption naming crates/ paths, stage order and measured or estimated durations. The three earlier jobs are not retried; their candidates are dropped.

Assumptions:
- Goal and audience: Athena's reader, planner and the user who writes speed requests. A request that claims a delay or speed-up with a timing figure must have the figure checked against the real stage order before planning.
- Code survey: crates/athena-core/src/reading/checks.rs (the ordered `step!` list at about line 539, where the new check is wired in after check_records_measure), crates/athena-core/src/reading/surveys.rs (check_code_survey at 199 and check_records_measure at 538 are the models for the lexical trigger and the error text), crates/athena-core/src/reading.rs (in-file tests for check order at about 8234 and for check_code_survey at 7240), crates/athena-core/tests/refusal_corpus.rs (the refusal corpus). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected.
- Existing code: crates/athena-core/src/reading/surveys.rs, crates/athena-core/src/reading/checks.rs and the helpers changes_code_with_behavior_criterion, has_phrase and straight_lowercase are reused; the new check extends them and does not duplicate them.
- Term: `timeline` means the `Timeline:` assumption listing each stage in order, its duration (measured from the code or recorded run timings, or labeled as an estimate), and the stage where the claimed delay sits.
- The trigger is lexical on the request and reading text: words like speed up, faster, delay, earlier, minutes or seconds with a stage name. A reading that names no such claim is unaffected. The check refuses more readings and the higher refusal rate is intended: expected 4% of newly refused readings. This is an estimate, not measured: 3 of about 1000 scored jobs recorded this failure class, plus other speed requests.
- Environment: none. The check works on the reading and repository snapshot only, with no CI, push, PR or records reads. The verifier replays past readings with the existing `replay`, which reads local records.
- Prompt files are not changed, because the reader prompt has 2 tokens left; the rule is enforced by mechanism in code, and the refusal message tells the reader what to add.
- Consumers: none found for config; no athena.toml or defaults.toml change is needed.

## Acceptance criteria

1. [content] Root cause named in CHANGELOG.md and a new appended decision in docs/decisions.md with a Provenance bullet: nothing required a reading of a delay or speed-up request to check the claimed figure against the real stage order and durations. It cites jobs 20261025-110525-8hzb, 20261025-034733-c453 and 20261025-190606-eebk. No existing decisions.md entry is edited.
2. [content] crates/athena-core/src/reading/surveys.rs defines `pub fn check_timing_claim`, and crates/athena-core/src/reading/checks.rs calls it with `step!(check_timing_claim(...))` in the check list. The error text names the missing `Timeline:` assumption and gives an example.
3. [behavior] Run `cargo test -p athena-core`: a new unit test shows a reading for the 8hzb-style request (move a call earlier to save about 12 minutes) with no `Timeline:` assumption is refused by check_timing_claim. It passes when a `Timeline:` assumption names crates/ paths, stage order and durations. The refusal test fails if the step! line is removed. A second test shows a reading with no speed or delay claim is not refused.
4. [behavior] Run `cargo test -p athena-core`: the existing check-order test and the refusal corpus test still pass, with one new corpus case for the refused speed-claim reading.
5. [content] The change declares `expected 4% of newly refused readings` in a criterion or the doc comment of check_timing_claim, with limits of the lexical trigger stated, as done for check_records_measure.
6. [behavior] Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass, as recorded by Athena's checks on the final tree.

## Expected deliverables

- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Retrying the three earlier jobs' own changes: job 20261025-110525-8hzb (run.rs, gh.rs early pull request move), job 20261025-034733-c453 (nextest and test speed-up) and job 20261025-190606-eebk (test speed-up); their candidates are dropped. Job 20261025-190606-eebk's candidate changed CHANGELOG.md, crates/athena-core/src/checks.rs, crates/athena-gate/src/secrets.rs and docs/decisions.md; checks.rs and secrets.rs changes are dropped, and CHANGELOG.md and docs/decisions.md get only this job's new entries.
- Editing prompt files under crates/*/prompts (token headroom is 2 to 12 tokens for reader and planner)
- Changing athena.toml, defaults.toml or CI workflows
- Actually speeding up any pipeline or test suite
- Editing existing decisions.md or records entries

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is a reading-stage refusal rule where the unchecked figure enters, not a per-job workaround.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a refusal check in code, with tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests that fail without the step line.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: The new test reads CHANGELOG.md and docs/decisions.md through `include_str!` (the form at refusal_corpus.rs:3187) and collapses white space. It asserts the CHANGELOG entry holds `reading::check_timing_claim`, the three job ids, `Timeline:` and the root-cause sentence. It asserts docs/decisions.md holds a `## D190 · ` heading whose entry holds the three job ids, the root-cause sentence and a `- **Provenance:**` bullet. The verifier also runs `git diff --numstat -- docs/decisions.md`, which must show 0 deleted lines, so no existing entry is edited. Run: `cargo test -p athena-core --test refusal_corpus timing_claim_refusal_is_declared_intended`. Observes: The test passes only when both texts hold the cause and the three jobs; numstat shows added lines only for docs/decisions.md.
- Criterion 2: A unit test in reading.rs modeled on `records_measure_has_one_step_line` (reading.rs:8698-8722). It reads the source through `reading_source()` and asserts: exactly one `step!(check_timing_claim(reading, request));` line sits in the body of `refusing_check`, immediately after the `step!(check_records_measure(reading));` line; the source holds `pub fn check_timing_claim(reading: &Reading, request: &str) -> Result<()>`; the text above `pub fn check_reading_acceptance(` names [`check_timing_claim`]; and the error of a refusing fixture (from `check_timing_claim(..).unwrap_err()`) contains `` `Timeline:` ``, `for example`, `athena status` and `landing and rebuild`. Run: `cargo test -p athena-core timing_claim_has_one_step_line`. Observes: The step line, the public function and an error text naming the `Timeline:` assumption with an example are all present.
- Criterion 3: Unit tests in the reading.rs test module, next to the records-measure tests. (1) `timing_claim_without_timeline_is_refused_and_with_timeline_accepted`: build `code_reading(&[SURVEY], &[MODIFY_READING])` (reading.rs:7049-7081), set the summary to `Move the call earlier in the run to save about 12 minutes`, and use that sentence as the request. Assert `refuser_for(&reading, request) == Some("check_timing_claim")` (reading.rs:7300), that `check_timing_claim(&reading, request).unwrap_err()` equals the gate's error text, and that `check_reading_text(&reading, request)` is `Refuse` with the Timeline text. Then push a `Timeline:` assumption naming `crates/athena-core/src/run/mod.rs`, stages in order with durations and the claimed delay, and assert `refuser_for` is None, `accept` is Ok and `check_reading_text` is `Accept`. (2) `timing_claim_class_shapes_are_refused`: a table of request and summary pairs for the class, not the one job: the 8hzb shape, `Speed up the test suite` (c453 and eebk shape, no figure), `The checks are delayed by 12 minutes`, `This speeds up the tests`, `Reduce the CI wait from 10 minutes`, `Reduce time spent waiting for CI`, `speed-up`, `Speed Up`, `90s`, `500ms`, and the text of the real generator `crate::hestia::slowdown_request(25.0, 60.0, &[StageGrowth{stage:"producer", previous_minutes:25.0, minutes:45.0}])` (hestia.rs:2381, 2320). Each is refused by check_timing_claim. (3) `timing_claim_leaves_other_readings_alone`: no speed or delay claim (the default `code_reading`), `delayer` with a figure, `Shorten the reader prompt so the budget tests pass`, `Raise the throughput of the drain queue`, `the earlier attempt ran 40 minutes`, a docs class, a question, a decline, content-only criteria, a root-cause request from `crate::hestia::root_cause_request` whose quoted lines hold the words, and a reading with `expected 4% of newly refused readings` and a `crates/athena-core/src/reading/surveys.rs` deliverable. Each gives `refuser_for != Some("check_timing_claim")`; and the same declaration with `crates/athena-core/src/gh.rs` is refused. (4) `timing_claim_names_each_missing_timeline_part`: remove the path, the second figure, the order marker, the label and the claim word one at a time; each refuses and names the missing part. (5) `timing_claim_example_satisfies_the_check_and_names_real_files`: extract the example as at reading.rs:8543-8547, push it as an assumption and assert the gate gives None; split it with `sentences` and `path_tokens`, assert every `crates/` token exists under `Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")`, and assert `check_reading_references(&reading, Some(&files))` is Ok for a set made of those paths. Run: `cargo test -p athena-core timing_claim`. Observes: The 8hzb-style reading with a claim and no Timeline is refused by `check_timing_claim` with an error naming `Timeline:`; it is accepted once a Timeline holds the path, order, durations, label and claim word. The same holds for the other shapes of the class and the reading with no claim is untouched.
  Fails without the behavior: `timing_claim_without_timeline_is_refused_and_with_timeline_accepted` asserts `refuser_for(..) == Some("check_timing_claim")`. With the `step!` line removed the gate returns None for that reading and the assertion fails. With the function body made `Ok(())` the direct `unwrap_err()` fails. With the trigger narrowed to the three jobs' words, the table test `timing_claim_class_shapes_are_refused` fails on `delayed`, `speeds up` and `Reduce the CI wait`. `timing_claim_leaves_other_readings_alone` passes either way, so the check is not a blanket refusal.
  Test seam: Production entry points: `refusing_check` (reading/checks.rs:520) through `refuser_for`/`gate_refuser` (reading.rs:7294-7302), `check_reading_text` (reading/replay.rs:54) and a direct call of `check_timing_claim`. Fixtures `code_reading`, `SURVEY`, `MODIFY_READING`, `refuser_for` exist (read at reading.rs:7049-7081 and 7294-7302); `crate::hestia::slowdown_request` and `root_cause_request` are real generators, pub and callable. Injection: the test sets `summary`, `assumptions`, `deliverables` and `acceptance_criteria` on the `Reading` value and passes the request text as an argument; no fake is used and no logic sits in a double. The guard is `changes_code_with_behavior_criterion`, which `code_reading` meets (a behavior criterion, class code, a capability, no question). The test module and its helpers are added by plan step 5; the gate-level requests are chosen so no earlier check refuses them, and the direct call needs no such care.
- Criterion 4: Add `timing_claim_cases()` to `cases()` with the 20 cases and 14 new tags of decision 7, and change `ci-wait-time-spent-reading` to `Expected::Refuse("check_timing_claim")`. Run the whole corpus file, then the order tests with `cargo test -p athena-core has_one_step_line` (`records_measure_has_one_step_line` and `history_fixture_has_one_step_line` pin only the predecessors of their steps, so they stay green with the new line after `check_records_measure`). Case ids I read as the pattern: `records-measure-change-on-fixtures-only-job-landing-time`, `ci-wait-time-spent-reading`, `retry-complaint-reading-names-the-job`; no result was seen. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: All corpus tests pass, including `the_wrapper_agrees_with_refusing_check_on_every_case` over the new cases; the new refusing cases are refused by `check_timing_claim` by name and the accepting cases are accepted; the existing order tests pass.
  Fails without the behavior: `every_acceptance_check_has_a_refusing_corpus_case` (refusal_corpus.rs:2905) fails if the step line exists with no case in which `check_timing_claim` refuses. `corpus_covers_the_listed_input_forms` (2826) fails if a new tag is missing from INPUT_FORMS or not carried by a case (the array length 77 also fails to compile if miscounted). With the `step!` line removed the Refuse cases fail in `corpus_cases_get_the_expected_outcome`.
  Test seam: Production entry point: `refusing_check` through the corpus `run(c)` (refusal_corpus.rs:2770) and `check_reading_acceptance` (checks.rs:492). Fixtures: `case()` (241), `base_reading` (172), `lines` and the generators `hestia::root_cause_request` and `hestia::slowdown_request` (public, already used at 1334-1349). All exist; the `timing_case` closure and `timing_claim_cases()` are added by plan step 6.
- Criterion 5: Modeled on `records_measure_refusal_is_declared_intended` (refusal_corpus.rs:3138-3207). (a) `replay_declaration` of a Criterion holding `The change refuses more readings: expected 4% of newly refused readings` gives `share == Some(4)`. (b) The doc comment directly above `pub fn check_timing_claim(` holds: `This check refuses more readings; the higher refusal rate is intended.`, `expected 4% of newly refused readings`, `estimate`, `Limits:`, `no reading of negation`, `figures written as words`, `throughput`, `earlier attempt`, `presence, not that its figures are true` and the three job ids. (c) The whole doc wrapped in a Criterion parses to 4. (d) The CHANGELOG entry holds the same declaration. Run: `cargo test -p athena-core --test refusal_corpus timing_claim_refusal_is_declared_intended`. Observes: The declared share parses to 4 and the limits text is present in the doc comment and the changelog.
- Criterion 6: Athena's three checks run on the final tree: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`, recorded by Athena's checks. The producer runs the same three in its own tree before finishing (plan step 10). Run: `cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: If a fixture elsewhere carried a speed phrase with a figure in a gated code reading, or a test asserted `Accept` for the CI-wait reading, `cargo test --locked` would fail. The searches in decision 3 found only `ci-wait-time-spent-reading`, which the plan updates; the producer fixes any other fixture it finds by narrowing the trigger tables, and changes a fixture only when it is truly a speed request.
  Test seam: Production entry points are cargo's own; nothing is injected. The checks' environment is the snapshot with `--locked`; the new tests read only compile-time sources and `CARGO_MANIFEST_DIR/../..` paths.

Planned files:

- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 1 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 4% of newly refused readings (the job is held above the declared share plus 5 points).

- 20261114-052309-w3zt: the reading was refused because the request or reading claims a delay or a speed-up (the request says "When planning escalates the planner to planner_revision_effort (xhigh) on a revision round, scale

## The change

Base cffab2af9276 → candidate 9ca7271669af. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |   1 +
 crates/athena-core/src/reading.rs          | 409 +++++++++++++++++++++++++++++
 crates/athena-core/src/reading/checks.rs   |   6 +-
 crates/athena-core/src/reading/surveys.rs  | 405 ++++++++++++++++++++++++++++
 crates/athena-core/tests/refusal_corpus.rs | 400 +++++++++++++++++++++++++++-
 docs/decisions.md                          |  55 ++++
 6 files changed, 1272 insertions(+), 4 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261107-144804-5vht/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-144804-5vht/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43954493368 for 9ca7271669af56759d073334b537eee2c2fb7194)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43954493368 for 9ca7271669af56759d073334b537eee2c2fb7194)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43954493368 for 9ca7271669af56759d073334b537eee2c2fb7194)
- ci-result: `not a check` passed (Check results for 9ca7271669af56759d073334b537eee2c2fb7194 come from the CI run https://github.com/doodla/athena/actions/runs/43954493368 (1068s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy` and `cargo test`: all pass, as recorded by Athena's checks on the final tree.)
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core timing_claim`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/surveys.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
8963 |             check_timing_claim(reading, request).unwrap_or_else(|e| panic!("{label}: {e}"));
     |             ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:8972:17
     |
8972 |         assert!(check_timing_claim(&timing_reading("x"), handwritten).is_err());
     |                 ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:8987:13
     |
8987 |             check_timing_claim(&rule, MOVE_CALL_EARLIER).unwrap_or_else(|e| panic!("{path}: {e}"));
     |             ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:9010:17
     |
9010 |         assert!(check_timing_claim(&deleted, MOVE_CALL_EARLIER).is_err());
     |                 ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:9014:17
     |
9014 |         assert!(check_timing_claim(&undeclared, MOVE_CALL_EARLIER).is_err());
     |                 ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:9056:25
     |
9056 |             let error = check_timing_claim(&reading, MOVE_CALL_EARLIER)
     |                         ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:9080:21
     |
9080 |         let error = check_timing_claim(&bare, MOVE_CALL_EARLIER)
     |                     ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:9089:9
     |
9089 |         check_timing_claim(&several, MOVE_CALL_EARLIER).unwrap();
     |         ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:9096:9
     |
9096 |         check_timing_claim(&arrow, MOVE_CALL_EARLIER).unwrap();
     |         ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:9102:21
     |
9102 |         let error = check_timing_claim(&refused, MOVE_CALL_EARLIER)
     |                     ^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 14 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/surveys.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 25 tests
test approve_request_has_the_generators_shapes ... ok
test ambiguous_name_changelog_and_decision ... ok
test defined_terms_decision_and_changelog ... ok
test defined_terms_replay_over_the_cited_jobs ... ok
test corpus_covers_the_listed_input_forms ... ok
test corpus_cases_get_the_expected_outcome ... FAILED
test an_extra_check_with_no_case_is_reported ... ok
test delivery_verb_replay_over_the_cited_jobs ... ok
test design_existing_code_refusal_is_declared_intended ... ok
test generated_request_has_the_generators_shapes ... ok
test environment_reach_refusal_is_declared_intended ... ok
test no_check_bypasses_refusing_check ... ok
test plan_side::a_plan_case_that_expects_the_wrong_outcome_is_reported ... ok
test plan_side::an_impact_case_that_expects_the_wrong_outcome_is_reported ... ok
test plan_side::impact_cases_get_the_expected_outcome ... ok
test plan_side::open_question_cases_get_the_expected_outcome ... ok
test plan_side::plan_cases_get_the_expected_outcome ... ok
test plan_side::the_complete_ci_reuse_case_names_the_rule_the_search_finds ... ok
test environment_cache_scope_refusal_is_declared_intended ... ok
test queued_work_holds_a_proposed_request_whose_stored_reading_leaves_a_term_undefined ... ok
test history_fixture_refusal_is_declared_intended ... ok
test timing_claim_refusal_is_declared_intended ... FAILED
test records_measure_refusal_is_declared_intended ... ok
test every_acceptance_check_has_a_refusing_corpus_case ... ok
test the_wrapper_agrees_with_refusing_check_on_every_case ... ok

failures:

---- corpus_cases_get_the_expected_outcome stdout ----

thread 'corpus_cases_get_the_expected_outcome' (22790) panicked at crates/athena-core/tests/refusal_corpus.rs:3097:5:
ci-wait-time-spent-reading (a CI wait job; rebuilt from the job brief): expected Refuse("check_timing_claim"), got accepted
timing-claim-call-earlier-12-minutes-no-timeline (job 20261025-110525-8hzb, a call moved earlier to save about 12 minutes; rebuilt from the job brief): expected Refuse("check_timing_claim"), got accepted
timing-claim-timeline-without-durations (job 20261025-110525-8hzb, a call moved earlier to save about 12 minutes; rebuilt from the job brief): expected Refuse("check_timing_claim"), got accepted
timing-claim-speed-up-the-tests-star-bullet (jobs 20261025-034733-c453 and 20261025-190606-eebk, a test speed-up with no stage timeline; rebuilt from the job briefs): expected Refuse("check_timing_claim"), got accepted
timing-claim-reduce-ci-wait-numbered-part (jobs 20261025-034733-c453 and 20261025-190606-eebk, a test speed-up with no stage timeline; rebuilt from the job briefs): expected Refuse("check_timing_claim"), got accepted
timing-claim-delayed-and-speeds-up (the shapes of the speed class; rebuilt from the introspection prevention text): expected Refuse("check_timing_claim"), got accepted
timing-claim-hyphen-spelling-90s (the shapes of the speed class; rebuilt from the introspection prevention text): expected Refuse("check_timing_claim"), got accepted
timing-claim-declaration-on-pipeline-change (the shapes of the speed class; rebuilt from the introspection prevention text): expected Refuse("check_timing_claim"), got accepted
timing-claim-steward-slowdown-request (the steward's slowdown request; built by hestia::slowdown_request): expected Refuse("check_timing_claim"), got accepted
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- timing_claim_refusal_is_declared_intended stdout ----

thread 'timing_claim_refusal_is_declared_intended' (23205) panicked at crates/athena-core/tests/refusal_corpus.rs:3526:5:
assertion failed: acceptance_check_names(reading_source()).contains(&"check_timing_claim".to_string())


failures:
    corpus_cases_get_the_expected_outcome
    timing_claim_refusal_is_declared_intended

test result: FAILED. 23 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s

error: test failed, to rerun pass `-p athena-core --test refusal_corpus`
error: 1 target failed:
    `-p athena-core --test refusal_corpus`
```
- fails-without-6: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 6 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/surveys.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:8972:17
     |
8972 |         assert!(check_timing_claim(&timing_reading("x"), handwritten).is_err());
     |                 ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:8987:13
     |
8987 |             check_timing_claim(&rule, MOVE_CALL_EARLIER).unwrap_or_else(|e| panic!("{path}: {e}"));
     |             ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:9010:17
     |
9010 |         assert!(check_timing_claim(&deleted, MOVE_CALL_EARLIER).is_err());
     |                 ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:9014:17
     |
9014 |         assert!(check_timing_claim(&undeclared, MOVE_CALL_EARLIER).is_err());
     |                 ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:9056:25
     |
9056 |             let error = check_timing_claim(&reading, MOVE_CALL_EARLIER)
     |                         ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:9080:21
     |
9080 |         let error = check_timing_claim(&bare, MOVE_CALL_EARLIER)
     |                     ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:9089:9
     |
9089 |         check_timing_claim(&several, MOVE_CALL_EARLIER).unwrap();
     |         ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:9096:9
     |
9096 |         check_timing_claim(&arrow, MOVE_CALL_EARLIER).unwrap();
     |         ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_timing_claim` in this scope
    --> crates/athena-core/src/reading.rs:9102:21
     |
9102 |         let error = check_timing_claim(&refused, MOVE_CALL_EARLIER)
     |                     ^^^^^^^^^^^^^^^^^^ not found in this scope

   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 14 previous errors
warning: build failed, waiting for other jobs to finish...
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
