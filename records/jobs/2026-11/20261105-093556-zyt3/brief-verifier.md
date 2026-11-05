# Job 20261105-093556-zyt3: verify "Require a root-cause diagnosis and a base-failing test in bug-fix"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The request and reading described the wanted page content but did not require a diagnosis: why decid
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: symptom-fix-without-diagnosing-root-cause-in-bug-job (20261104-054711-kb90)
> Cause: The request and reading described the wanted page content but did not require a diagnosis: why decided proposals reach the published page, from which data source and state. Without a criterion for the root cause and a test that uses real de...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261104-054700-e6js (job 20261103-201526-t4qv); prevention it proposed: For any job that fixes wrong output, the reading must include a criterion that names the root cause: where the wrong items enter and why they are not excluded. It must also require a test built from real decided (approved and rejected) prop...
> Introspection 20261105-051344-q9fv (job 20261103-231808-3gkq); prevention it proposed: Two changes together. First, for bug-fix jobs, Athena runs the new tests on the base tree on a scratch copy before the producer edits code, and records the result. If a test passes on the base, the job must say the bug is not reproduced and...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Jobs 20261103-201526-t4qv and 20261103-231808-3gkq both moved a filter without diagnosing why decided proposals reached the page, because the reading never required a diagnosis. Fix it at the reading gate: a new check refuses a bug-fix code/tests reading that has no root-cause criterion and no criterion for a test that fails on the base tree, plus reader guidance. Audience: Athena's reader and verifier.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs: refusing_check (step! lines in order: check_prior_result_and_scope, check_criteria_evidence, check_code_survey, check_design_existing_code, check_governance_coverage, ...) is the entry point and the new check is added as one step! line after check_code_survey; it reads the request, class_of_work, criteria and title only, never the repository. crates/athena-core/tests/refusal_corpus.rs fails when a step! has no corpus case where that check refuses, so a corpus case is required. crates/athena-context/src/briefs.rs holds CODE_SURVEY_GUIDE, the model for the new reader guide. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched. The property is measured directly by calling check_reading_acceptance / refusing_check on real reading shapes (the 20261103-201526-t4qv reading) and asserting the check name that refuses.
- Goal: stop the class 'symptom fix without diagnosis' seen in jobs 20261103-201526-t4qv and 20261103-231808-3gkq (introspections 20261104-054700-e6js and 20261105-051344-q9fv). Fix at the origin: the reading gate that accepted readings with no root-cause criterion and a test that passes on the base. Both jobs were rejected; their candidate edits to xtask/src/decisions.rs, xtask/src/site.rs and CHANGELOG.md are not part of this job and stay dropped, and their findings (decided proposals still shown, test passing on base) are addressed by the gate requirement, not by re-fixing the decisions page.
- A bug-fix request is detected from the request text by whole-word matches such as fix, bug, wrong, broken, regression, fails, 'already', 'still shows'; the check applies only when the reading has a capability, no question, class code or tests, and at least one behavior criterion.
- The check requires two criteria: one containing 'root cause' that says where the wrong output enters and why it is not excluded, and one behavior criterion saying a named test fails on the base tree (without the change) and passes with it, using real recorded data shapes. Running the new tests on a scratch base copy (the first introspection's second proposal) is out of this job; only the reading gate and reader guidance change.
- Expected 8% of newly refused readings; the refusal rate rises intentionally because bug-fix readings now need a root-cause criterion. Replay stays under the 10% hold.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: a new unit test in crates/athena-core/src/reading.rs builds the reading for a request like 'The page lists proposals that were already decided; fix it' with criteria that only describe the wanted page content, and check_root_cause_diagnosis refuses it, naming the missing root-cause criterion and the missing base-failing test criterion; the same reading with both criteria is accepted. The refusal test fails on the base tree because the function does not exist and the reading is accepted.
2. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: a corpus case in crates/athena-core/tests/refusal_corpus.rs shaped like the 20261103-201526-t4qv reading is refused by check_root_cause_diagnosis, and the corpus coverage test passes with the new step! line in refusing_check.
3. [behavior] Run `cargo test -p athena-core`: a non-bug request (add a feature), a docs reading, a decline and a reading with a question are not refused by the new check.
4. [content] crates/athena-core/src/reading.rs doc comment on the new check names the root cause (reading did not require a diagnosis of where wrong items enter, nor a test that fails on the base), cites jobs 20261103-201526-t4qv and 20261103-231808-3gkq, and states 'expected 8% of newly refused readings'.
5. [content] crates/athena-context/src/briefs.rs gains a reader guide constant, composed beside CODE_SURVEY_GUIDE, telling the reader to add a root-cause criterion and a base-failing test criterion for bug-fix jobs; its brief test asserts the section contains it. The prompt files under the token budget are not edited.
6. [content] CHANGELOG.md has a Fixed entry that names the root cause and the new gate.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)

## Out of scope

- xtask/src/decisions.rs, xtask/src/site.rs and the decisions page fix: candidates of jobs 20261103-201526-t4qv and 20261103-231808-3gkq are dropped, not continued
- Running new tests on a scratch base copy in the pipeline (drain/producer changes)
- athena.toml, defaults.toml, rust-toolchain.toml, .gitattributes, setup.sh and .github/ workflows
- Editing records/ or prompt files with no token headroom (reader, verifier prompts)
- Changing approve-work or reject-work behavior or record schemas

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in crates/athena-core and crates/athena-context.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests and a corpus case; fmt, clippy and test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed at the reading gate where the class originates.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by a refusal check, not prose alone.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the check on real reading shapes.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criterion checks the gate refuses the symptom-only reading.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check, one guide, tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments name cause and jobs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 5)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in `mod tests` of reading.rs. A `refuses_a_symptom_only_reading`: start from `code_reading(&[SURVEY], &[MODIFY_READING])`, one Behavior criterion that only describes page content ('Run `cargo test -p athena-core`: the page shows only undecided proposals'), request 'The page lists proposals that were already decided; fix it.'. Call `check_root_cause_diagnosis`, `refusing_check` and `check_reading_acceptance`; assert the first refusing step is `check_root_cause_diagnosis` and the message names the missing root-cause criterion and the missing base-failing test criterion. B `accepts_both_criteria`: add 'The root cause is named: decided proposals enter the page from the unfiltered list built in `crates/athena-core/src/reading.rs` and the filter does not exclude them' and Behavior 'The named test `decided_proposals_are_not_listed`, built from approved and rejected proposals, fails on the base tree and passes with the change'; Ok from the check and the gate. B2: location as plain text xtask/src/site.rs is accepted. C `refuses_copied_boilerplate`: criteria are the request's stock bullets verbatim; refused by the new check. D: no location refused; 'and/or' alone refused; base-failing criterion with no named test refused, including 'Run `cargo test`: tests fail without the change' (finding 4); dropping only the test criterion gives a message naming only that part. E `accepts_this_jobs_own_reading`. F `agrees_with_the_revert_run_predicate`: for each phrase of run/tests.rs:4700-4731 (positive 'A test fails without the fix', 'Tests fail without the change', 'the new test fails on the base', 'the demo test FAILS without it', 'A regression test that fails before the fix and passes after'; negative 'the build never fails', 'a failing check is reported to the user', 'cargo test passes') build a criterion 'The named test `decided_proposals_are_not_listed`: {phrase}' and assert the gate accepts it exactly when `crate::run::asks_for_failing_run_without_the_fix(phrase)`; 'the test fails before the change' is refused. G `bug_fix_signal`: request-prose cases (fix, fixes, fixing, bug, 'still shows', 'still showed', 'still lists') trigger; 'prefix', 'fixture', `fix_it` do not. H `continued_bug_fix`: an outcome-worded request ('Show only open proposals above the limit on the decisions page') is accepted with no attempt, refused with an attempt (rejected, `verdict` Some('fail'), finding 'decided proposals still shown; the new test passes on the base'), accepted with a rejected attempt whose findings and request name no symptom, and accepted when the attempt is `failed == true`. I `stored_bug_fix_reading_is_refused_by_check_reading_text`: `check_reading_text` on a class-code symptom-only reading with request containing 'fix' returns `ReadingCheck::Refuse`, and Accept with both criteria. Run: `cargo test -p athena-core root_cause_diagnosis`. Observes: A is refused naming both missing parts. B, B2, E accepted. C and D refused by `check_root_cause_diagnosis`; the bare `cargo test` criterion refused. F: gate accept set equals the revert-run predicate. G and H: the triggers and the continuation signal behave as listed. I: stored readings are refused by `check_reading_text`.
  Fails without the behavior: On the base tree `check_root_cause_diagnosis` does not exist, so the tests do not compile. Without the step! line `refusing_check` returns None and A, C, H are accepted. Without the location and not-a-request-bullet rules C and D fail. If the location rule accepted only crates/ or backticks B2 fails. With a private phrase list F fails. Without the non-cargo test-name rule the bare `cargo test` case in D is accepted.
  Test seam: Production `check_root_cause_diagnosis` (step 3), `refusing_check` and `check_reading_acceptance` and `check_reading_text` (exist, reading.rs:3164-3217, 3744), `asks_for_failing_run_without_the_fix` (exists, run/mod.rs:174; step 1 makes it `pub(crate)`). Fixtures `code_reading`, `accept`, `SURVEY`, `MODIFY_READING` in the reading.rs test module (exist); `EarlierAttempt` built with `..EarlierAttempt::default()` as refusal_corpus.rs:201-208 does. Values are passed as arguments; no double, no logic in fixtures.
- Criterion 2: Add cases to `reading_check_cases()` in crates/athena-core/tests/refusal_corpus.rs via `case()`, `base_reading()`, `repo_files()`, `root_cause_request()`, `earlier()` (all exist). Each has class code (docs case: docs; regression-tests case: tests), a `Code survey:` assumption naming crates/athena-core/src/reading.rs (in `repo_files()`), no report/summary wording. New input form 'bug-fix-request' (INPUT_FORMS 17 to 18). Cases: (1) `bug-fix-reading-without-diagnosis`, dash bullets, content-only criteria, Refuse; source is a reconstruction from the brief's description of 20261103-201526-t4qv, NOT its verbatim request (the snapshot has no records; stated in the case comment). (2) `bug-fix-reading-copying-the-stock-bullets`, number-dot bullets copying refusal_corpus.rs:227-228, Refuse. (3) `bug-fix-reading-with-root-cause-but-no-base-failing-test`, Refuse. (4) `introspection-fix-request-without-diagnosis`, prose of `root_cause_request(&[])` (refusal_corpus.rs:218; hestia.rs:434), Refuse. (5) `bug-fix-reading-with-diagnosis-and-base-failing-test`, backticked locations, Accept. (6) `bug-fix-reading-naming-an-xtask-path-in-plain-text`, Accept. (7) `feature-request-with-a-failure-bullet-needs-no-diagnosis`, star bullet 'fails when ...', Accept. (8) `feature-request-with-fixed-and-broken-words-needs-no-diagnosis`, Accept. (9) `feature-request-needs-no-diagnosis`, Accept. (10) `bug-fix-docs-reading-needs-no-diagnosis`, class docs, Accept. (11) `regression-tests-request-needs-no-diagnosis`, class tests, Accept. (12) `outcome-worded-first-attempt-is-accepted`, request 'Show only open proposals above the limit on the decisions page' (worded like HISTORY.txt:75), no attempt, Accept; documents the limit. (13) `outcome-worded-retry-of-a-rejected-bug-fix-needs-diagnosis`, same request, `attempt` = `earlier(...)` with `failed` false, `verdict` Some('fail'), finding 'decided proposals still shown; the new test passes on the base', reading accounts for the prior job as the existing retry cases do (refusal_corpus.rs:347-380), Refuse. (14) `still-showed-request-needs-diagnosis`, prose 'The page published on 3 Nov still showed decided proposals', Refuse. Existing forms (star-bullet, number-dot-parts, code-survey, retry) are carried by cases 2, 7 and 13. Add `root_cause_diagnosis_refusal_is_declared_intended` beside `design_existing_code_refusal_is_declared_intended` (CHANGELOG contains 'check_root_cause_diagnosis', 'expected 8% of newly refused readings', 'estimate', 'athena go', 'introspection'). Run: `cargo test -p athena-core --test refusal_corpus`. Observes: `corpus_cases_get_the_expected_outcome`, `every_acceptance_check_has_a_refusing_corpus_case` (passes only when a case is refused by `check_root_cause_diagnosis`), `the_wrapper_agrees_with_refusing_check_on_every_case` and `corpus_covers_the_listed_input_forms` pass. I cannot run it here and cite only case ids I planned; the producer and verifier run it.
  Fails without the behavior: Without the check, cases 1-4, 13 and 14 are accepted and `corpus_cases_get_the_expected_outcome` reports a mismatch. With the step! line but no refusing case, `every_acceptance_check_has_a_refusing_corpus_case` fails naming the check.
  Test seam: Production `refusing_check` through the corpus `run()` (exists, refusal_corpus.rs:660, which passes `c.attempt` and `c.referenced`). Fixtures `case()`, `base_reading()`, `repo_files()`, `root_cause_request()`, `earlier()` exist; new cases and the INPUT_FORMS change are added in step 6.
- Criterion 3: Unit test `root_cause_diagnosis_does_not_apply` in reading.rs calling `check_root_cause_diagnosis` and `check_reading_acceptance` with the same symptom-only criteria on: 'add a greeting'; a feature request whose bullet says '- `athena status` fails with a clear message when no job exists' and whose prose says a flag 'already' exists; the same bullet indented (trimmed, so a bullet); prose with 'a fixed list', 'reports broken links', 'a wrong value'; class tests 'Add regression tests for the parser'; docs-class 'Fix the typo in the guide'; a decline (`capability = None`); `question = Some(..)`; only content criteria; request with only 'prefix' and 'fixture'; a retry (attempt set) of a rejected feature job whose request and findings carry no symptom phrase. Each is Ok. The bug-fix reading from criterion 1 is refused in the same test. Run: `cargo test -p athena-core root_cause_diagnosis_does_not_apply`. Observes: Every non-bug-fix shape is accepted by the check and the gate; the bug-fix reading is refused.
  Fails without the behavior: Without the guards (capability, question, class, behavior criterion, prose-only trigger with bullets dropped, whole-word split, symptom-only continuation) the feature, docs, decline, question, indented-bullet, regression-tests, fixed/broken/wrong and feature-retry readings are refused.
  Test seam: Production `check_root_cause_diagnosis` and `check_reading_acceptance`; fixtures `code_reading`, `scoped_reading`, `accept` in the reading.rs test module (exist). Mutations are field assignments on `Reading` (`capability`, `question`, `class_of_work`) as the code-survey tests do near reading.rs:8250.
- Criterion 4: Test `root_cause_diagnosis_doc_comment_names_the_cause` in reading.rs takes `include_str!("reading.rs")`, splits at 'pub fn check_root_cause_diagnosis', takes the last `///` paragraph before it (as the existing test near reading.rs:8286 does for `check_code_survey`), joins it and asserts it contains 'did not require', both job ids, 'expected 8% of newly refused readings', 'estimate', 'asks_for_failing_run_without_the_fix', 'introspection', and the stated limit that an outcome-worded first attempt is not caught. A second assertion builds a `Reading` whose criterion 4 text is this job's sentence and asserts `replay_declaration_of(&reading).share == Some(8)`. Run: `cargo test -p athena-core root_cause_diagnosis_doc_comment`. Observes: The doc comment holds the root cause, both ids, the declared share with its estimate label, the shared predicate, the introspection scope and the limit; the replay parser reads 8 from the criterion.
  Fails without the behavior: Without the function there is a compile error; a comment lacking any string fails a contains assertion.
  Test seam: Production source text via `include_str!` and production `replay_declaration_of` (reading.rs:3979, exists). No double. Added in step 4.
- Criterion 5: Add `pub const ROOT_CAUSE_GUIDE` to briefs.rs, compose it right after `CODE_SURVEY_GUIDE` in `reader_brief`, add the seventh `READING_RULES` line naming `check_root_cause_diagnosis` (at most 220 bytes). Update `the_reader_brief_names_each_reading_rule_in_one_short_line` (7 rules, new check name, raised bound with a comment). Add `the_reader_brief_tells_the_reader_to_diagnose_a_bug_fix`: take the reading-rules section of `reader_brief(&reader_inputs(&reading()))`; assert `section.contains(ROOT_CAUSE_GUIDE)`; assert the guide contains 'root cause', 'where the wrong', 'fails on the base tree', 'real', 'approved and rejected', 'backticked'. Glob crates/athena-context/prompts to confirm no prompt file changed. Run: `cargo test -p athena-context reader_brief`. Observes: The section carries the guide; 7 rule lines each at most 220 bytes; prompt files unchanged, so `system_facts_budgets_match_computed` still matches docs/system-facts.md.
  Fails without the behavior: Without the constant the new test does not compile; a brief that omits the guide fails `section.contains(ROOT_CAUSE_GUIDE)`.
  Test seam: Production `reader_brief` (exists, briefs.rs near 1970) with fixtures `reader_inputs(&reading())` (exist, near 3935). Constant and composition added in step 5.
- Criterion 6: Add a Fixed entry at the top of the `### Fixed` list under `## [Unreleased]` in CHANGELOG.md. It names the root cause (the reading gate accepted bug-fix readings requiring neither a diagnosis of where wrong output enters nor a test that fails on the base; jobs 20261103-201526-t4qv and 20261103-231808-3gkq), `check_root_cause_diagnosis`, the guide, the shared revert-run predicate, the bug-fix signal (prose words, 'still' phrases, retry of a bug-fix or symptom-finding job), that introspection fix requests are in scope, that an outcome-worded first attempt is not caught, that queued proposed jobs whose stored reading now fails are held until `athena go` re-reads them, 'the higher refusal rate is intended', 'expected 8% of newly refused readings' and, as a separate sentence, that it is an estimate. The corpus test `root_cause_diagnosis_refusal_is_declared_intended` reads CHANGELOG.md with `include_str!`, collapses white space as the existing declared-intended test does, and asserts the strings. Run: `cargo test -p athena-core --test refusal_corpus root_cause_diagnosis_refusal_is_declared_intended`. Observes: The changelog holds the gate name, the declared share with its estimate label, the held-jobs note and the introspection scope.
  Fails without the behavior: Without the entry the contains assertions fail.
  Test seam: Production file CHANGELOG.md via `include_str!`. Test added in step 6. No double.
- Criterion 7: Run the three Athena checks as separate commands on the final tree: fmt, then clippy, then the full test run. The full run covers athena-core, athena-context, the corpus, the run/tests.rs predicate tests (`the_fails_without_helper_matches_singular_and_plural_wordings`, run/tests.rs:4698), steward tests (unchanged), `system_facts_budgets_match_computed` and the xtask policy tests. New text uses no banned word or email-like string. Run: `cargo fmt --all --check`. Observes: Exit 0 for fmt; then `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked` exit 0.
  Fails without the behavior: A formatting slip, unused helper or clippy lint fails the first two; an existing test moved by the new rule line, the size bound or the visibility change fails the third.
  Test seam: Cargo commands on the whole workspace; no double.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 9 of 50 are newly refused by the candidate and were accepted by the base. The job is held for this.

Declared share: 8% of newly refused readings (the job is held above the declared share plus 5 points).
Reason given for the declared share (judge whether it explains why past readings should be refused): Run `cargo test -p athena-core`: a new unit test in crates/athena-core/src/reading.rs builds the reading for a request like 'The page lists proposals that were already decided; fix it' with criteria that only describe the wanted page content, and check_root_cause_diagnosis refuses it, naming the mis

- 20261105-205932-ppfv: the reading was refused because it is a bug-fix change (code) whose criteria describe the wanted result but do not require a diagnosis; it lacks a root-cause criterion that says 'root cause', says whe
- 20261104-153614-pzfw: the reading was refused because it is a bug-fix change (code) whose criteria describe the wanted result but do not require a diagnosis; it lacks a root-cause criterion that says 'root cause', says whe
- 20261104-124034-ysc4: the reading was refused because it is a bug-fix change (code) whose criteria describe the wanted result but do not require a diagnosis; it lacks a root-cause criterion that says 'root cause', says whe
- 20261104-091432-d10c: the reading was refused because it is a bug-fix change (code) whose criteria describe the wanted result but do not require a diagnosis; it lacks a root-cause criterion that says 'root cause', says whe
- 20261104-081830-js1p: the reading was refused because it is a bug-fix change (code) whose criteria describe the wanted result but do not require a diagnosis; it lacks a root-cause criterion that says 'root cause', says whe
- 20261104-054754-b4f5: the reading was refused because it is a bug-fix change (code) whose criteria describe the wanted result but do not require a diagnosis; it lacks a root-cause criterion that says 'root cause', says whe
- 20261103-231510-20z0: the reading was refused because it is a bug-fix change (code) whose criteria describe the wanted result but do not require a diagnosis; it lacks a root-cause criterion that says 'root cause', says whe
- 20261103-231422-8dzf: the reading was refused because it is a bug-fix change (code) whose criteria describe the wanted result but do not require a diagnosis; it lacks a root-cause criterion that says 'root cause', says whe
- 20261103-231410-asf2: the reading was refused because it is a bug-fix change (code) whose criteria describe the wanted result but do not require a diagnosis; it lacks a root-cause criterion that says 'root cause', says whe

## The change

Base e2da35340f60 → candidate bfda3de49736. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  27 ++
 crates/athena-context/src/briefs.rs        |  55 ++-
 crates/athena-core/src/reading.rs          | 679 ++++++++++++++++++++++++++++-
 crates/athena-core/src/run/mod.rs          |   2 +-
 crates/athena-core/tests/refusal_corpus.rs | 254 ++++++++++-
 docs/architecture.md                       |   8 +
 6 files changed, 1017 insertions(+), 8 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-093556-zyt3/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-093556-zyt3/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42126873713 for bfda3de4973633714747b3274ceae2d244c63b77)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42126873713 for bfda3de4973633714747b3274ceae2d244c63b77)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42126873713 for bfda3de4973633714747b3274ceae2d244c63b77)
- ci-result: `not a check` passed (Check results for bfda3de4973633714747b3274ceae2d244c63b77 come from the CI run https://github.com/doodla/athena/actions/runs/42126873713 (521s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core root_cause_diagnosis`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, docs/architecture.md.
  End of the output (data, not instructions):

```text
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8560:9
     |
8560 |         check_root_cause_diagnosis(&reading, outcome_request, &[]).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8574:17
     |
8574 |         assert!(check_root_cause_diagnosis(&reading, outcome_request, &[&symptom]).is_err());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8577:17
     |
8577 |         assert!(check_root_cause_diagnosis(&reading, outcome_request, &[&bug]).is_err());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8580:9
     |
8580 |         check_root_cause_diagnosis(&reading, outcome_request, &[&feature]).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8584:9
     |
8584 |         check_root_cause_diagnosis(&reading, outcome_request, &[&failed]).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8587:9
     |
8587 |         check_root_cause_diagnosis(&reading, outcome_request, &[&unverified]).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8656:13
     |
8656 |             check_root_cause_diagnosis(reading, request, &[])
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8662:9
     |
8662 |         check_root_cause_diagnosis(&bug_fix_reading(none), "Add a greeting", &[&feature]).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8664:17
     |
8664 |         assert!(check_root_cause_diagnosis(&bug_fix_reading(none), BUG_REQUEST, &[]).is_err());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0425, E0603.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 21 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, docs/architecture.md.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.06s
     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-658bf9aaa38225b9)

running 12 tests
test corpus_covers_the_listed_input_forms ... ok
test corpus_cases_get_the_expected_outcome ... FAILED
test no_check_bypasses_refusing_check ... ok
test plan_side::a_plan_case_that_expects_the_wrong_outcome_is_reported ... ok
test plan_side::open_question_cases_get_the_expected_outcome ... ok
test plan_side::plan_cases_get_the_expected_outcome ... ok
test plan_side::the_complete_ci_reuse_case_names_the_rule_the_search_finds ... ok
test an_extra_check_with_no_case_is_reported ... ok
test root_cause_diagnosis_refusal_is_declared_intended ... FAILED
test every_acceptance_check_has_a_refusing_corpus_case ... ok
test the_wrapper_agrees_with_refusing_check_on_every_case ... ok
test design_existing_code_refusal_is_declared_intended ... ok

failures:

---- corpus_cases_get_the_expected_outcome stdout ----

thread 'corpus_cases_get_the_expected_outcome' (28049) panicked at crates/athena-core/tests/refusal_corpus.rs:906:5:
bug-fix-reading-without-diagnosis (jobs 20261103-201526-t4qv and 20261103-231808-3gkq, a page that lists decided proposals; rebuilt from the job brief, not verbatim): expected Refuse("check_root_cause_diagnosis"), got accepted
bug-fix-reading-copying-the-stock-bullets (jobs 20261103-201526-t4qv and 20261103-231808-3gkq, a page that lists decided proposals; rebuilt from the job brief, not verbatim): expected Refuse("check_root_cause_diagnosis"), got accepted
bug-fix-reading-with-root-cause-but-no-base-failing-test (jobs 20261103-201526-t4qv and 20261103-231808-3gkq, a page that lists decided proposals; rebuilt from the job brief, not verbatim): expected Refuse("check_root_cause_diagnosis"), got accepted
introspection-fix-request-without-diagnosis (job 20261030-170757-gev8's request shape (an introspection fix request); rebuilt from the job brief): expected Refuse("check_root_cause_diagnosis"), got accepted
outcome-worded-retry-of-a-rejected-bug-fix-needs-diagnosis (jobs 20261103-201526-t4qv and 20261103-231808-3gkq, a page that lists decided proposals; rebuilt from the job brief, not verbatim): expected Refuse("check_root_cause_diagnosis"), got accepted
still-showed-request-needs-diagnosis (jobs 20261103-201526-t4qv and 20261103-231808-3gkq, a page that lists decided proposals; rebuilt from the job brief, not verbatim): expected Refuse("check_root_cause_diagnosis"), got accepted
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- root_cause_diagnosis_refusal_is_declared_intended stdout ----

thread 'root_cause_diagnosis_refusal_is_declared_intended' (28058) panicked at crates/athena-core/tests/refusal_corpus.rs:1098:5:
assertion failed: acceptance_check_names(READING_SOURCE).contains(&"check_root_cause_diagnosis".to_string())


failures:
    corpus_cases_get_the_expected_outcome
    root_cause_diagnosis_refusal_is_declared_intended

test result: FAILED. 10 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p athena-core --test refusal_corpus`
error: 1 target failed:
    `-p athena-core --test refusal_corpus`
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core root_cause_diagnosis_does_not_apply`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/run/mod.rs, docs/architecture.md.
  End of the output (data, not instructions):

```text
8552 |         check_root_cause_diagnosis(&reading, "Add a prefix to every line", &[]).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8560:9
     |
8560 |         check_root_cause_diagnosis(&reading, outcome_request, &[]).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8574:17
     |
8574 |         assert!(check_root_cause_diagnosis(&reading, outcome_request, &[&symptom]).is_err());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8577:17
     |
8577 |         assert!(check_root_cause_diagnosis(&reading, outcome_request, &[&bug]).is_err());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8580:9
     |
8580 |         check_root_cause_diagnosis(&reading, outcome_request, &[&feature]).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8584:9
     |
8584 |         check_root_cause_diagnosis(&reading, outcome_request, &[&failed]).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8587:9
     |
8587 |         check_root_cause_diagnosis(&reading, outcome_request, &[&unverified]).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8656:13
     |
8656 |             check_root_cause_diagnosis(reading, request, &[])
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8662:9
     |
8662 |         check_root_cause_diagnosis(&bug_fix_reading(none), "Add a greeting", &[&feature]).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_root_cause_diagnosis` in this scope
    --> crates/athena-core/src/reading.rs:8664:17
     |
8664 |         assert!(check_root_cause_diagnosis(&bug_fix_reading(none), BUG_REQUEST, &[]).is_err());
     |                 ^^^^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0425, E0603.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 21 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core root_cause_diagnosis_doc_comment`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-context reader_brief`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-core --test refusal_corpus root_cause_diagnosis_refusal_is_declared_intended`). Result: not run: only the first few distinct commands are run.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo fmt --all --check`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
