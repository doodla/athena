# Job 20261109-075120-7zqw: verify "Refuse oversized multi-surface readings before dispatch"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Nothing before dispatch measures whether a request fits one producer run. This job touches a decisio
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: multi-surface-job-producer-makes-no-change-and-cannot-report (20261025-185414-aapc)
> Cause: Nothing before dispatch measures whether a request fits one producer run. This job touches a decision doc, the tool pins in athena.toml, the CI workflow, the check runner's output parsing in checks.rs and gh.rs, new unit tests and the chang...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261025-185359-yvbn (job 20261025-183644-t67q); prevention it proposed: Add a pre-dispatch size check and a no-change guard for the producer. (1) Before dispatch, count the distinct deliverable surfaces and criteria in the reading, and send any job over a set bound back to be split. (2) If the producer ends wit...
> Introspection 20261109-075105-011b (job 20261017-205554-zaj2); prevention it proposed: Add a check before dispatch that splits a request spanning separable surfaces (core snapshot and landing code with tests, versus config and changelog) into one job per change, ordered by dependency. Also require a producer that ends with an...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: no step in the reading gate (`refusing_check`, crates/athena-core/src/reading/checks.rs) measures whether a request fits one producer run. Jobs 20261025-183644-t67q (10 deliverables over docs, config, CI and code) and 20261017-205554-zaj2 (12 deliverables) ended with no change. I add a `check_job_size` step that sends back, to be split, any change reading spanning too many deliverables or separable surfaces. Both earlier attempts are addressed this way: their size problem no longer reaches dispatch, and neither left a candidate to keep or drop.

Assumptions:
- Code survey: crates/athena-core/src/reading/checks.rs (`refusing_check` runs the `step!` checks in order, and `check_scope_limit` is the nearest precedent), crates/athena-core/src/reading.rs (the gate tests, `gate_refuser`, the step-order tests) and crates/athena-core/src/reading/surveys.rs. The property is measured directly on `Reading.deliverables` and `Reading.acceptance_criteria`, not through a stand-in. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not involved.
- Goal and audience: the Athena operator and the gate's later readers; a job that is too large is stopped at reading time with a repair, instead of costing a producer run that ends with no change.
- Size bound: count deliverables other than the always-written CHANGELOG.md, and count distinct surfaces (docs, config such as athena.toml, .github/workflows, xtask, tests, and each crates/<crate>). Refuse a change reading above 8 deliverables or 4 surfaces. The exact numbers are a design choice, set so both failing jobs are refused. Study readings and readings with a question are skipped, as in check_scope_limit.
- The 'producer ends with no change and cannot report' half of the proposed prevention is out of this job. Pre-dispatch splitting prevents the class at its origin, and `run/producer.rs` already reports a no-change outcome.
- The retried jobs left no candidate (no recorded changes), so no earlier work is kept or dropped. Their deliverables (docs/decisions.md, athena.toml, ci.yml, xtask, athena-data config, the athena-git snapshot, and others) are not in this job's scope.
- Consumers: none found. No config file is a deliverable, and this job does not change athena.toml, defaults.toml or any workflow.
- Prompt files are untouched because the reader prompt has only 71 tokens of headroom; the refusal error message carries the repair text.

## Acceptance criteria

1. [content] crates/athena-core/src/reading/checks.rs has `pub fn check_job_size(reading: &Reading) -> Result<()>` and one `step!(check_job_size(reading));` line in `refusing_check`. Its doc comment names the root cause (nothing before dispatch measured whether a request fits one producer run) and cites jobs 20261025-183644-t67q and 20261017-205554-zaj2.
2. [behavior] Run `cargo test -p athena-core`. A new test builds a change reading shaped like job 20261025-183644-t67q (docs, athena.toml, CI workflow, two crates files, xtask, tests, CHANGELOG) and one shaped like 20261017-205554-zaj2 (12 deliverables). `check_job_size` refuses both, and the error says to split the job by surface and dependency. The test fails when the `step!` line is removed.
3. [behavior] Run `cargo test -p athena-core`. Tests show the bound is general: a focused reading with a few deliverables in one or two surfaces passes. A reading with many deliverables in one crate is refused on count alone, and a study reading and a reading with a question are not refused.
4. [behavior] Run `cargo test -p athena-core`. The refusal corpus (crates/athena-core/tests/refusal_corpus.rs) has a case in which check_job_size is the refusing check, so the corpus test that every `step!` has a refusing case still passes. The existing gate-order tests still pass.
5. [content] The check refuses more readings and the higher refusal rate is intended: expected 6% of newly refused readings, an estimate and not a measurement. CHANGELOG.md has an entry that names the root cause and the new bound.
6. [behavior] Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`: all pass.

## Expected deliverables

- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Splitting or redoing job 20261025-183644-t67q (nextest adoption, docs/decisions.md, athena.toml, ci.yml, xtask) or job 20261017-205554-zaj2 (study snapshot isolation, Hermes Agent, the systems schema)
- A producer-side no-change guard or changes to run/producer.rs and run/send_back.rs
- Prompt files, athena.toml, defaults.toml and .github/workflows
- Automatic splitting of a request into several jobs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests plus a refusal corpus case; fmt, clippy and test are criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is a gate step at the origin, not a patch for either job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test on readings shaped like the two failed jobs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Source-text tests in the in-crate tests module of reading.rs, copying `scope_limit_has_one_step_line` and `scope_limit_doc_and_changelog_name_the_cause`. The tests are added in step 5. Run: `cargo test -p athena-core job_size_`. Observes: `job_size_has_one_step_line` asserts exactly one trimmed line of the `refusing_check` body equals `step!(check_job_size(reading));` and that checks.rs contains `pub fn check_job_size(reading: &Reading) -> Result<()>`. `job_size_doc_and_changelog_name_the_cause` collapses the doc comment above `check_job_size` and asserts it contains 'nothing before dispatch measured whether a request fits one producer run' and both job ids 20261025-183644-t67q and 20261017-205554-zaj2.
  Fails without the behavior: Without the function the tests do not compile. Without the step line, the one-line test fails. A doc without the root-cause sentence or the job ids fails the needle assertions.
  Test seam: Source text via `include_str!("reading/checks.rs")` as in the existing scope-limit tests in reading.rs (exists). The tests are added by plan step 5. No double.
- Criterion 2: Unit tests build two change readings with the helper `job_size_reading(paths)` (a wrapper on `governed_reading`, added in step 4). The t67q shape: docs/decisions.md, athena.toml, .github/workflows/ci.yml, crates/athena-core/src/checks.rs, crates/athena-core/src/gh.rs, crates/athena-core/tests/nextest.rs, xtask/src/main.rs, a tests file and CHANGELOG.md. The zaj2 shape: 12 deliverables over docs, athena.toml, ci.yml, xtask, athena-data and athena-git files, plus CHANGELOG.md. Each goes through the production gate with `gate_refuser`; a second test goes through `refusing_check`. Run: `cargo test -p athena-core job_size_`. Observes: `gate_refuser(&reading) == Some("check_job_size")` for both shapes. `check_job_size(&reading).unwrap_err().to_string()` contains 'one producer run', 'split', 'surface', 'dependency' and 'out_of_scope', and lists the groups found. `job_size_refusal_needs_the_step_line` asserts the gate result is `Some("check_job_size")` through `refusing_check`.
  Fails without the behavior: Without the `step!` line `gate_refuser` returns another refuser or None and `assert_eq!` fails. With the group bound raised to 4 both shapes are accepted and the test fails. Counting config and ci as one group also fails it.
  Test seam: Production entry points `athena_core::reading::refusing_check` (checks.rs:518), reached through `gate_refuser` (reading.rs:7049, exists), and `check_job_size` (step 2). Fixtures `governed_reading` (reading.rs:4401) and `limited` (reading.rs:8099) exist. `job_size_reading` is added in step 4. The fixture builds input data only; no double is injected.
- Criterion 3: `job_size_bound_is_general`, `job_size_accepts_landed_shapes` and `job_size_refuses_the_backstop_landed_shape` call `check_job_size` and `gate_refuser`. Refused: 31 non-changelog deliverables in one crate (count alone, one group); docs + athena.toml + a workflow + a crate file (groups alone, 4 files); the same list with `crates/athena-core/src/` as a directory deliverable; a root `.` deliverable; the 31 non-changelog paths of landed commit 6e7fd3a copied from HISTORY.txt lines 780-811 (3 groups, refused by count, so the figure in the doc is executed). Accepted: a focused docs + one crate file + CHANGELOG reading; exactly 30 deliverables in one crate; config, ci and code (3 groups); `./docs/a.md` and `docs/a.md` as one group; CHANGELOG.md counted neither as a deliverable nor as the docs group; a study reading with 40 deliverables; a reading with a question and 40 deliverables; path lists from landed commits a5c938c, 97d1e5c, d6d2ad1, dd4d76d, 6f1b5f1, 0cd0111 and eab5d6c. Run: `cargo test -p athena-core job_size_`. Observes: Refused readings return an error naming the crossed bound ('31 deliverables, bound 30' or '4 groups, bound 3') and listing the groups. Accepted readings return Ok, and `gate_refuser` is None for the in-bound change readings.
  Fails without the behavior: A check that tested only the two job shapes fails the count-alone case. A check that ignored groups fails the 4-file refusal. A check that ignored `capability` or `question` fails the skip cases. A check that counted CHANGELOG.md fails the at-the-bound case. A per-surface bound fails the landed-shape cases (dd4d76d, d6d2ad1).
  Test seam: Production entry points `check_job_size` (step 2) and `refusing_check`. Fixtures `governed_reading`, `scoped_reading` (reading.rs:3545) and `limited` exist; `reading.capability = Some("study")` and `reading.question = Some(..)` are set on the fixture as in the existing scope-limit unaffected cases. Landed path lists are inline test data copied from HISTORY.txt. No double.
- Criterion 4: Add `job_size_cases()` to refusal_corpus.rs and wire it into `cases()`. Input forms and case ids (each from the existing `case()` and `base_reading()` fixtures with `c.reading.deliverables` replaced, as `history_case` does): the t67q shape (4 groups) -> `job-size-four-groups-t67q` (new; Refuse check_job_size); the zaj2 shape (12 deliverables, 4 groups) -> `job-size-four-groups-zaj2` (new; Refuse); 31 files in one crate -> `job-size-over-count` (new; Refuse); a directory or root deliverable hiding groups -> `job-size-directory-deliverable` (new; Refuse); a landed 9-file one-crate shape from HISTORY.txt a5c938c -> `job-size-within-bounds-landed` (new; Accept); config, ci and code, 3 groups -> `job-size-three-groups-accepted` (new; Accept). A dash-bullet request form is covered by the existing `base_reading()` cases. Crate-relative paths and `./` prefixes are handled by `normalized_path` (checks.rs:775) and have a unit case in step 5. A retry complaint, brand word, environment variable or job-citing request is not read by this rule, which looks only at deliverable paths, so it needs no case. I read the corpus structure (`INPUT_FORMS`, `cases()`, `scope_limit_cases` near line 333) and cite only those names; I cannot run the tests. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: Each new refusing case yields `check_job_size` as the refuser through `refusing_check`; the accepting cases yield Accept. `every_acceptance_check_has_a_refusing_corpus_case` finds `step!(check_job_size` and a refusing case. `INPUT_FORMS` gains four tags (`job-size-four-groups`, `job-size-over-count`, `job-size-directory-deliverable`, `job-size-within-bounds`), length 52 to 56, checked by `corpus_covers_the_listed_input_forms`. The existing gate-order tests in reading.rs still pass: none pins the new second position.
  Fails without the behavior: Without the `step!` line the refusing cases are accepted and `corpus_cases_get_the_expected_outcome` fails. Without the cases, `every_acceptance_check_has_a_refusing_corpus_case` fails with 'no corpus case in which ["check_job_size"] refuses'.
  Test seam: Production entry point `athena_core::reading::refusing_check`, imported at refusal_corpus.rs:52 (exists). `job_size_cases()` and the four tags are added in step 6. No double.
- Criterion 5: A unit test pins the doc comment and the CHANGELOG item; a corpus test pins that the declaration parses to the stated share. Both use the accepted criterion 5 text unchanged: 'expected 6% of newly refused readings, an estimate and not a measurement'. Run: `cargo test -p athena-core job_size_ && cargo test -p athena-core --test refusal_corpus job_size_refusal_is_declared_intended`. Observes: The doc comment above `check_job_size` and the CHANGELOG.md [Unreleased] item containing `reading::check_job_size` both contain: the root-cause sentence, 'higher refusal rate is intended', 'expected 6% of newly refused readings', 'estimate', 'HISTORY.txt', 'only a reading that spans all four groups', 'more than 3 groups', '30 deliverables', '2 of 200', 'check_reading_size' and 'producer no-change guard'. The corpus test asserts `replay_declaration(&[criterion with that sentence], &[]).share == Some(6)`, modelled on `design_existing_code_refusal_is_declared_intended` (refusal_corpus.rs:2771).
  Fails without the behavior: A CHANGELOG item without the cause, the bound, the complements or the share fails the needle assertions. A doc without 'This check refuses more readings; the higher refusal rate is intended.' fails the pin. A share written as 4% fails `Some(6)`.
  Test seam: `include_str!("reading/checks.rs")` and `include_str!("../../../CHANGELOG.md")` as in the scope-limit doc test (exist). `athena_core::reading::replay_declaration` is public and used at refusal_corpus.rs:2773 (exists). Tests are added in steps 5 and 7. No double.
- Criterion 6: Run the three gate commands Athena runs on the result. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. This covers every existing test that reaches the gate (reading.rs gate tests, run/tests.rs, steward.rs, athena-cli drain.rs, the briefs test that lists reading checks).
  Fails without the behavior: A formatting slip, a clippy lint (an unused constant or helper), or an existing reading now refused as too large fails it.
  Test seam: The cargo commands themselves; nothing is injected.

Planned files:

- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 6% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 517b29a9d40a → candidate 68deeb6ca2b2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  23 +++
 crates/athena-core/src/reading.rs          | 304 +++++++++++++++++++++++++++++
 crates/athena-core/src/reading/checks.rs   | 124 +++++++++++-
 crates/athena-core/tests/refusal_corpus.rs | 163 +++++++++++++++-
 4 files changed, 612 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-075120-7zqw/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-075120-7zqw/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42588979913 for 68deeb6ca2b2cf1d7a763d23745c95b8ff9996cd)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42588979913 for 68deeb6ca2b2cf1d7a763d23745c95b8ff9996cd)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42588979913 for 68deeb6ca2b2cf1d7a763d23745c95b8ff9996cd)
- ci-result: `not a check` passed (Check results for 68deeb6ca2b2cf1d7a763d23745c95b8ff9996cd come from the CI run https://github.com/doodla/athena/actions/runs/42588979913 (787s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`: all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 2, 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core job_size_`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading/checks.rs.
  End of the output (data, not instructions):

```text
8573 |             assert!(check_job_size(&reading).is_err(), "root form {root:?}");
     |                     ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_job_size` in this scope
    --> crates/athena-core/src/reading.rs:8577:17
     |
8577 |         assert!(check_job_size(&reading).is_ok());
     |                 ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_job_size` in this scope
    --> crates/athena-core/src/reading.rs:8584:17
     |
8584 |         assert!(check_job_size(&with_ci).is_err());
     |                 ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_job_size` in this scope
    --> crates/athena-core/src/reading.rs:8588:17
     |
8588 |         assert!(check_job_size(&reading).is_ok());
     |                 ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_job_size` in this scope
    --> crates/athena-core/src/reading.rs:8591:17
     |
8591 |         assert!(check_job_size(&reading).is_ok());
     |                 ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_job_size` in this scope
    --> crates/athena-core/src/reading.rs:8594:17
     |
8594 |         assert!(check_job_size(&reading).is_ok());
     |                 ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_job_size` in this scope
    --> crates/athena-core/src/reading.rs:8597:17
     |
8597 |         assert!(check_job_size(&reading).is_ok());
     |                 ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_job_size` in this scope
    --> crates/athena-core/src/reading.rs:8633:21
     |
8633 |         let error = check_job_size(&reading).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_job_size` in this scope
    --> crates/athena-core/src/reading.rs:8636:17
     |
8636 |         assert!(check_job_size(&job_size_reading(&as_strs(&paths))).is_ok());
     |                 ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_job_size` in this scope
    --> crates/athena-core/src/reading.rs:8643:17
     |
8643 |         assert!(check_job_size(&reading).is_err());
     |                 ^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 13 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading/checks.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.43s
     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 24 tests
test ambiguous_name_changelog_and_decision ... ok
test approve_request_has_the_generators_shapes ... ok
test defined_terms_decision_and_changelog ... ok
test defined_terms_replay_over_the_cited_jobs ... ok
test delivery_verb_replay_over_the_cited_jobs ... ok
test design_existing_code_refusal_is_declared_intended ... ok
test environment_cache_scope_refusal_is_declared_intended ... ok
test corpus_covers_the_listed_input_forms ... ok
test corpus_cases_get_the_expected_outcome ... FAILED
test generated_request_has_the_generators_shapes ... ok
test environment_reach_refusal_is_declared_intended ... ok
test job_size_refusal_is_declared_intended ... FAILED
test no_check_bypasses_refusing_check ... ok
test plan_side::a_plan_case_that_expects_the_wrong_outcome_is_reported ... ok
test plan_side::an_impact_case_that_expects_the_wrong_outcome_is_reported ... ok
test plan_side::impact_cases_get_the_expected_outcome ... ok
test plan_side::open_question_cases_get_the_expected_outcome ... ok
test plan_side::plan_cases_get_the_expected_outcome ... ok
test plan_side::the_complete_ci_reuse_case_names_the_rule_the_search_finds ... ok
test queued_work_holds_a_proposed_request_whose_stored_reading_leaves_a_term_undefined ... ok
test an_extra_check_with_no_case_is_reported ... ok
test history_fixture_refusal_is_declared_intended ... ok
test every_acceptance_check_has_a_refusing_corpus_case ... ok
test the_wrapper_agrees_with_refusing_check_on_every_case ... ok

failures:

---- corpus_cases_get_the_expected_outcome stdout ----

thread 'corpus_cases_get_the_expected_outcome' (9594) panicked at crates/athena-core/tests/refusal_corpus.rs:2767:5:
job-size-four-groups-t67q (job 20261025-183644-t67q, nine deliverables over docs, config, CI and code; rebuilt from the job brief): expected Refuse("check_job_size"), got refused by check_governance_coverage: the reading was refused because the outcome needs governance files it does not cover: athena.toml (it is listed as a deliverable but no criterion names it); .github/workflows/ci.yml (it is listed as a deliverable but no criterion names it); list each missing path as a deliverable and name it in an acceptance criterion, or, if this job must not change it, name it in out_of_scope; admission refuses a governance change that is not both listed and named, and only after the producer has finished
job-size-four-groups-zaj2 (job 20261017-205554-zaj2, twelve deliverables over docs, config, CI and code; rebuilt from the job brief): expected Refuse("check_job_size"), got refused by check_governance_coverage: the reading was refused because the outcome needs governance files it does not cover: athena.toml (it is listed as a deliverable but no criterion names it); .github/workflows/ci.yml (it is listed as a deliverable but no criterion names it); list each missing path as a deliverable and name it in an acceptance criterion, or, if this job must not change it, name it in out_of_scope; admission refuses a governance change that is not both listed and named, and only after the producer has finished
job-size-over-count (the bounds of the check (3 groups, 30 deliverables); built for the check): expected Refuse("check_job_size"), got accepted
job-size-directory-deliverable (the bounds of the check (3 groups, 30 deliverables); built for the check): expected Refuse("check_job_size"), got refused by check_governance_coverage: the reading was refused because the outcome needs governance files it does not cover: athena.toml (it is listed as a deliverable but no criterion names it); list each missing path as a deliverable and name it in an acceptance criterion, or, if this job must not change it, name it in out_of_scope; admission refuses a governance change that is not both listed and named, and only after the producer has finished
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- job_size_refusal_is_declared_intended stdout ----

thread 'job_size_refusal_is_declared_intended' (9979) panicked at crates/athena-core/tests/refusal_corpus.rs:2928:5:
assertion failed: acceptance_check_names(reading_source()).contains(&"check_job_size".to_string())


failures:
    corpus_cases_get_the_expected_outcome
    job_size_refusal_is_declared_intended

test result: FAILED. 22 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s

error: test failed, to rerun pass `-p athena-core --test refusal_corpus`
error: 1 target failed:
    `-p athena-core --test refusal_corpus`
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core job_size_ && cargo test -p athena-core --test refusal_corpus job_size_refusal_is_declared_intended`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
