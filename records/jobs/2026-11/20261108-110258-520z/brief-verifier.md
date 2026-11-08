# Job 20261108-110258-520z: verify "Refuse plans that leave caller and test impact to the producer"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: The planning stage is not required to find every existing caller and test that runs the code path it
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: plan-leaves-existing-test-impact-open-fails-planning-rounds (20261025-070501-16qb)
> Cause: The planning stage is not required to find every existing caller and test that runs the code path it changes, or to settle each open question before it hands over the plan. The risk list and open questions can say "producer to check", and t...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261025-070448-xbdw (job 20261025-055626-er96); prevention it proposed: Change the plan template and the planning checks. (1) For every production path the plan changes, the planner must list the existing callers and tests that run it (a search of the tests for the entry point and for output assertions) and say...
> Introspection 20261108-110243-93rh (job 20261026-193430-avnq); prevention it proposed: Before dispatching a move or split job, run a check that searches the test sources for reads of the file or symbols being moved (for example include_str! or fs reads of display.rs) and lists them in the reading as files the job must update....
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: plan admission (`plan_findings` in admission.rs) checks open questions but not risks, so a plan can pass with a risk whose mitigation is "producer to check" and no list of existing callers/tests that run the changed path (seen in jobs 20261025-055626-er96 and 20261026-193430-avnq). Fix at that gate and in the planner brief: a risk that defers to the producer is a major finding, and the planner must list the callers and tests of each changed path in `dependents`.

Assumptions:
- Goal and audience: the planner and the plan gate, so the planning rounds stop ending 'plan still needed revising' because caller and test impact was left for the producer; the user and verifier judge by the new refusal and its test.
- Code survey: read crates/athena-core/src/admission.rs (plan_findings, open_questions_findings, dependents_findings, dependents_unnamed; entry point plan_findings, called from crates/athena-core/src/run/planning.rs), crates/athena-data/src/records/plan.rs (PlanRisk has risk and mitigation strings; JobPlan.risks), crates/athena-core/tests/refusal_corpus.rs (open_question_cases and plan_findings tests) and crates/athena-context/prompts/planner.md. No record or data file is added, so check_record_file, AnyRecord and the round-trip test are unaffected. The property is measured directly by calling plan_findings on a JobPlan whose risk mitigation defers to the producer.
- Cause as it originates: open_questions_findings already forces open questions to be User:/Host: only, but a risk's mitigation is unchecked, and its doc comment says the caller listing 'stays prose in the planner brief'. Job 20261025-055626-er96 failed in plan round 3 for exactly this (risk 'Existing go tests would call real gh' left to the producer; CLI lifecycle tests asserting no 'warning' were missed). Job 20261026-193430-avnq failed because a move broke athena-core's include_str! read of display.rs, which no one searched for.
- Previous result, job 20261025-055626-er96: its work (early push in gh.rs, run.rs, land.rs, pr.rs, test_support.rs) was never produced (planning stopped, no candidate). Its findings do not apply to this change; they are the evidence for the cause. The early-push feature itself is dropped here and stays a separate job.
- Previous result, job 20261026-193430-avnq: its candidate (display.rs split into display/status_view.rs, CHANGELOG entry) was rejected because status.rs's include_str! test read display.rs. That split is dropped here: display.rs and status_view.rs are not touched. Its CHANGELOG entry is not kept; this job adds its own entry.
- The new rule: a risk whose mitigation (or risk text) defers to the producer, with phrases such as 'producer to check', 'producer should check', 'producer will check', 'producer to verify', 'left to the producer', is a major finding at location 'risks' naming it, sent back to the planner without a critic call. A mitigation that states a found answer with file and line passes.
- The planner brief gains one short line (planner has 175 tokens of headroom; stay within it): for every production path the plan changes, search the tests and other crates for callers, include_str!/fs reads of moved files, and output assertions, list them in dependents, and never defer them to the producer. No other prompt file changes.
- Refusal rule: expected 5% of newly refused plans. The higher refusal rate is intended, because those plans spent critic rounds on impact the planner could find by searching.

## Acceptance criteria

1. [content] crates/athena-core/src/admission.rs has a risks rule called from plan_findings (next to open_questions_findings) whose doc comment names the root cause: planning was not required to settle caller and test impact and let risks defer it to the producer; the rule is a major finding at location 'risks'.
2. [behavior] A unit test in crates/athena-core/src/admission.rs builds a JobPlan with a risk whose mitigation is 'producer to check existing go tests' and asserts plan_findings returns a major Athena finding at 'risks' naming it; the same test fails on the base commit and passes with the change. Run: cargo test -p athena-core --lib admission.
3. [behavior] The test also shows a risk whose mitigation states a settled answer with file and line (for example 'lifecycle.rs:2627 asserts no warning; GhEarly skips quietly') and an empty risks list add no finding, so the rule is not blanket. Run: cargo test -p athena-core --lib admission.
4. [behavior] crates/athena-core/tests/refusal_corpus.rs maps each input form of the new rule (each deferring phrase, the settled answer, mixed case, deferral in the risk text rather than the mitigation) to a case with its expected outcome, and passes. Run: cargo test -p athena-core --test refusal_corpus.
5. [content] crates/athena-context/prompts/planner.md requires the planner to list, for each changed production path, the existing callers and tests that run it (including include_str!/fs reads of moved files and output assertions) in dependents and never leave them to the producer; Grep shows the line.
6. [content] The change expects 5% of newly refused plans; the higher refusal rate is intended because those plans cost critic rounds on impact the planner can find by searching.
7. [content] CHANGELOG.md has an entry naming the root cause and the new risks rule, and the display.rs/status_view.rs split of job 20261026-193430-avnq is not present in the diff.
8. [behavior] cargo fmt --check, cargo clippy --workspace --all-targets and cargo test --workspace all pass, including the prompt token budget tests for the planner set.

## Expected deliverables

- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/prompts/planner.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/planning.rs (modify)

## Out of scope

- The early-push-to-PR-branch feature of job 20261025-055626-er96 (gh.rs, run.rs, land.rs, pr.rs, test_support.rs); changing ci.yml or the xtask fingerprint; merging the PR before verification passes; changing how land merges or waits for checks
- The display.rs split of job 20261026-193430-avnq: crates/athena-cli/src/display.rs and crates/athena-cli/src/display/status_view.rs are not changed or created; behavior or output format changes; splitting other files or raising limits; refactoring the status function; fixing status.rs's include_str! test
- Changing the critic prompt, the reader prompt, athena.toml or defaults.toml
- A mechanical repository-wide search for callers (the search stays the planner's; Athena checks only that risks do not defer it)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The rule and tests are Rust in athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A failing-without test plus refusal corpus cases; fmt, clippy and test must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced in plan_findings, not left as prose only.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed at plan admission and the planner brief where the cause originates, covering both jobs' failures.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run plan_findings on a deferring plan and check the finding.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One rule, one brief line, tests and a changelog entry.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The outcome is that plans deferring impact are refused before the critic, checked by the test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment names the cause for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check by Grep and Read, then run the admission tests. Grep admission.rs for `fn risks_findings`, `fn dependents_deferral_findings` and `fn defers_to_producer`. Read the `findings.extend(risks_findings(&plan.risks))` call after `open_questions_findings` in `plan_findings`, and read the doc comment. Run: `cargo test -p athena-core --lib admission`. Observes: The `risks_findings` doc comment names the root cause: planning was not required to settle caller and test impact and let risks and dependents defer it to the producer. It says the finding is Major at location 'risks', and lists what the rule cannot see (omission, passive, adverb, singular `test`). The call sits in `plan_findings` beside the open-questions call. The admission unit tests compile against it and pass.
- Criterion 2: Unit tests `a_risk_that_defers_to_the_producer_is_a_major_finding_naming_it` and `a_dependent_that_defers_to_the_producer_is_a_major_finding_naming_it` in admission.rs. Each builds a plan whose only variable part is the risks or the dependent, calls `plan_findings`, and filters to location 'risks' (or a location starting 'dependents, dependent'). Risk inputs: ('Existing go tests would call real gh', 'producer to check existing go tests'); 'the producer should check the callers', 'will verify other crates that read it', 'producer should also search the tests', 'producer to look for include_str! readers', 'producer checks existing callers', "producer's to verify the readers", 'left to the producer: which tests assert no warning'; a deferral in the risk text with a settled mitigation; two risks where only the second defers (numbering). Dependent inputs: reason 'producer to check for include_str! readers' and test 'producer to check existing tests'. Run: `cargo test -p athena-core --lib admission`. Observes: One finding per deferring risk or dependent: severity Major, source Athena, location Some('risks') or 'dependents, dependent N', summary containing the offending text and 'defers its impact to the producer'. The second of two risks is named 'Risk 2'.
  Fails without the behavior: On the base commit `plan_findings` has no risks or dependents-deferral rule, so the findings at 'risks' are empty and the `len() == 1` assertion fails for every deferring input.
  Test seam: Entry point: `plan_findings(&JobPlan, &Reading, &Policy, &dyn Fn(&str)->bool)` in admission.rs, the function called from run/planning.rs:216. Fixture: the module's existing `plan_findings_for` (admission.rs:870), which builds a JobPlan with a content criterion and an empty verifier Policy. Step 3 makes it delegate to a new `plan_findings_with(dependents, open_questions, risks)` and adds the wrappers `risk_findings_for` and `dependent_findings_for`. Nothing is doubled; the tests supply data only. `plan_findings` and `plan_findings_for` exist today; the wrappers are added by step 3.
- Criterion 3: Unit tests `a_settled_risk_and_an_empty_risks_list_add_no_finding` and a dependents counterpart in admission.rs, using the same helpers. Inputs that add no finding at 'risks' or 'dependents, dependent N': mitigation 'lifecycle.rs:2627 asserts no warning; GhEarly skips quietly'; 'The producer runs cargo test -p athena-core --lib admission after the edit'; 'the producer should check that cargo fmt --check passes'; 'the producer will verify with cargo test -p athena-cli --test budgets'; 'the producer must read the computed figure from the budgets test failure message'; 'the producer must confirm cargo fmt --check passes'; 'the producer should verify the existing tests pass'; 'updating the budget line is left to the producer'; an empty risks list; a dependent whose reason states a found answer ('status.rs:410 reads display.rs via include_str!; test status_reads_display updated in step 4'). Run: `cargo test -p athena-core --lib admission`. Observes: For each input the findings at the checked locations are empty. For the empty risks list the whole `plan_findings` result is empty, as before the rule.
  Fails without the behavior: A blanket implementation (any risk that mentions the producer, any non-empty list, or discovery verbs without the impact-object and procedural-marker conditions) produces a finding and fails these emptiness assertions. Refusing deferrals at all is criterion 2's failing-on-base direction.
  Test seam: Same as criterion 2: production `plan_findings` through `risk_findings_for` and `dependent_findings_for`, added by step 3. Nothing is doubled.
- Criterion 4: Add an impact table inside `mod plan_side` of crates/athena-core/tests/refusal_corpus.rs, after `open_question_cases` (line 2606), with the test `impact_cases_get_the_expected_outcome` and a wrong-expectation check like `a_plan_case_that_expects_the_wrong_outcome_is_reported` (line 2575). Map of input forms to case ids (I read the existing tables; I have not run the new cases). Refuse at 'risks': risk-producer-to-check (source: job 20261025-055626-er96, 'Existing go tests would call real gh'), risk-producer-should-check, risk-producer-will-check, risk-producer-to-verify, risk-producer-should-also-check, risk-producer-to-look-for, risk-producer-checks-callers (inflected), risk-possessive-producers-to-verify, risk-left-to-the-producer, risk-mixed-case-deferral ('PRODUCER To Check'), risk-bold-and-backtick-markup-deferral, risk-deferral-across-a-line-break, risk-deferral-in-the-risk-text (synthetic: written for this corpus, an include_str! reader of a moved file left to the producer, settled mitigation). Refuse at 'dependents, dependent 1': dependent-reason-defers, dependent-test-defers. Accept: risk-settled-answer-with-file-and-line, risk-producer-runs-a-command, risk-producer-should-check-that-fmt-passes, risk-producer-will-verify-with-cargo-test-budgets, risk-producer-must-read-the-figure, risk-producer-must-confirm-fmt, risk-producer-verifies-existing-tests-pass, risk-left-to-the-producer-without-impact, risk-empty-list, dependent-reason-states-found-answer. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: Every impact case gets its expected outcome: each Refuse case gives exactly one Major Athena finding at its location whose summary contains the stated text; each Accept case gives no finding. Ids are unique and the table holds both outcomes.
  Fails without the behavior: Without the rule every Refuse case gets zero findings and the table test fails. Without the impact-object and procedural-marker conditions the Accept cases 'should check that fmt passes' and 'will verify with cargo test ... budgets' fail. Without the narrowed verb list the 'must read' and 'must confirm' cases fail.
  Test seam: Entry point: `athena_core::admission::plan_findings`. Fixture: the corpus's existing `plan_side::plan_for(dependents)` plus `super::base_reading("")` with a content criterion, as `question_findings` does (refusal_corpus.rs:2668). The new `impact_findings(&ImpactCase)` helper sets `plan.risks` and the dependent's test and reason before the call. The harness shape exists; the table, case struct and helper are added by step 4. The matching logic stays in production code.
- Criterion 5: Content check: Grep planner.md for 'include_str!' and read the new line. Then run the budget tests. Run: `cargo test -p athena-cli --test budgets`. Observes: One line tells the planner to search the tests and other crates for the callers of every changed production path, for include_str! or fs reads of moved files and for output assertions, to list them in dependents, and never to leave them to the producer (Athena refuses a written deferral). The planner set stays within 1300 and docs/system-facts.md carries the recomputed `budget planner` figure.
- Criterion 6: Content check: Grep for '5% of newly refused plans' in admission.rs and CHANGELOG.md. Run: `cargo test -p athena-core --lib admission`. Observes: The `risks_findings` doc comment and the CHANGELOG entry each say the rule refuses more plans, expected 5% of newly refused plans (the reading's estimate, not a measurement), and that the higher refusal rate is intended because those plans cost critic rounds on impact the planner can find by searching. The admission tests compile and pass with the comment in place.
- Criterion 7: Read the new CHANGELOG.md entry under [Unreleased], then check the changed paths with git status and git diff. Run: `git status`. Observes: The entry names the root cause (planning was not required to settle caller and test impact, and risks and dependents could say 'producer to check', as in job 20261025-055626-er96), the two rules, and the limit: job 20261026-193430-avnq's unlisted reader is covered only by the planner.md prose line. It names briefs.rs `OPEN_QUESTION_RULE` as a follow-up. Changed paths are only admission.rs, refusal_corpus.rs, planner.md, planning.rs, CHANGELOG.md and docs/system-facts.md. display.rs is unchanged and display/status_view.rs does not exist.
- Criterion 8: Run the whole gate: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, then `cargo test --workspace`, which includes budgets.rs, the refusal corpus and the athena-context wording tests. Run: `cargo test --workspace`. Observes: All pass. Fake planner outputs in run/tests.rs, lifecycle.rs, job_plan.rs and introspection.rs carry `"risks": []` and no deferral text, so none is refused.
  Fails without the behavior: Without the system-facts update, `system_facts_budgets_match_computed` fails. Without the rule none of the new tests pass.
  Test seam: Whole-workspace run, no double. Existing tests that reach `plan_findings` indirectly with empty risks are the regression check.

Planned files:

- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/prompts/planner.md (modify)
- docs/system-facts.md (modify)
- crates/athena-core/src/run/planning.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).
Reason given for the declared share (judge whether it explains why past readings should be refused): The change expects 5% of newly refused plans; the higher refusal rate is intended because those plans cost critic rounds on impact the planner can find by searching.

## The change

Base efc7c2cfaf5d → candidate cb43d64a6016. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  16 ++
 crates/athena-context/prompts/planner.md   |   1 +
 crates/athena-context/src/lib.rs           |   5 +-
 crates/athena-core/src/admission.rs        | 352 ++++++++++++++++++++++++++++-
 crates/athena-core/src/run/planning.rs     |   4 +-
 crates/athena-core/tests/refusal_corpus.rs | 310 ++++++++++++++++++++++++-
 docs/system-facts.md                       |   2 +-
 7 files changed, 681 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-110258-520z/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-110258-520z/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42480667363 for cb43d64a60168ff062087ebff3a73be69f60b166)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42480667363 for cb43d64a60168ff062087ebff3a73be69f60b166)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42480667363 for cb43d64a60168ff062087ebff3a73be69f60b166)
- ci-result: `not a check` passed (Check results for cb43d64a60168ff062087ebff3a73be69f60b166 come from the CI run https://github.com/doodla/athena/actions/runs/42480667363 (771s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --workspace --all-targets and cargo test --workspace all pass, including the prompt token budget tests for the planner set.)
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2, 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --lib admission`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/planner.md, crates/athena-core/src/admission.rs, crates/athena-core/src/run/planning.rs, docs/system-facts.md.
  End of the output (data, not instructions):

```text
   Compiling cpufeatures v0.2.17
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling winnow v0.7.15
   Compiling sha2 v0.10.9
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling jiff v0.2.38
   Compiling linux-raw-sys v0.12.1
   Compiling bitflags v2.13.2
   Compiling ref-cast v1.0.27
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling anyhow v1.0.104
   Compiling equivalent v1.0.2
   Compiling dyn-clone v1.0.20
   Compiling hashbrown v0.17.1
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0422]: cannot find struct, variant or union type `PlanRisk` in this scope
   --> crates/athena-core/src/admission.rs:876:39
    |
876 |             .map(|(risk, mitigation)| PlanRisk {
    |                                       ^^^^^^^^ not found in this scope

error[E0425]: cannot find type `PlanRisk` in this scope
   --> crates/athena-core/src/admission.rs:909:20
    |
909 |         risks: Vec<PlanRisk>,
    |                    ^^^^^^^^ not found in this scope
    |
help: you might be missing a type parameter
    |
906 |     fn plan_findings_with<PlanRisk>(
    |                          ++++++++++

Some errors have detailed explanations: E0422, E0425.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 2 previous errors
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/planner.md, crates/athena-core/src/admission.rs, crates/athena-core/src/run/planning.rs, docs/system-facts.md.
  End of the output (data, not instructions):

```text
test defined_terms_decision_and_changelog ... ok
test defined_terms_replay_over_the_cited_jobs ... ok
test delivery_verb_replay_over_the_cited_jobs ... ok
test design_existing_code_refusal_is_declared_intended ... ok
test environment_reach_refusal_is_declared_intended ... ok
test corpus_cases_get_the_expected_outcome ... ok
test corpus_covers_the_listed_input_forms ... ok
test generated_request_has_the_generators_shapes ... ok
test no_check_bypasses_refusing_check ... ok
test plan_side::a_plan_case_that_expects_the_wrong_outcome_is_reported ... ok
test plan_side::an_impact_case_that_expects_the_wrong_outcome_is_reported ... FAILED
test plan_side::impact_cases_get_the_expected_outcome ... FAILED
test plan_side::open_question_cases_get_the_expected_outcome ... ok
test plan_side::plan_cases_get_the_expected_outcome ... ok
test plan_side::the_complete_ci_reuse_case_names_the_rule_the_search_finds ... ok
test queued_work_holds_a_proposed_request_whose_stored_reading_leaves_a_term_undefined ... ok
test an_extra_check_with_no_case_is_reported ... ok
test history_fixture_refusal_is_declared_intended ... ok
test every_acceptance_check_has_a_refusing_corpus_case ... ok
test the_wrapper_agrees_with_refusing_check_on_every_case ... ok

failures:

---- plan_side::an_impact_case_that_expects_the_wrong_outcome_is_reported stdout ----

thread 'plan_side::an_impact_case_that_expects_the_wrong_outcome_is_reported' (4539) panicked at crates/athena-core/tests/refusal_corpus.rs:3029:9:
assertion `left == right` failed
  left: 1
 right: 2
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- plan_side::impact_cases_get_the_expected_outcome stdout ----

thread 'plan_side::impact_cases_get_the_expected_outcome' (4540) panicked at crates/athena-core/tests/refusal_corpus.rs:3010:9:
risk-producer-to-check (job 20261025-055626-er96: the risk and mitigation its plan passed the gate with): expected Refuse("defers its impact to the producer"), got []
risk-producer-should-check (synthetic: written for this corpus): expected Refuse("defers its impact to the producer"), got []
risk-producer-will-check (synthetic: written for this corpus): expected Refuse("defers its impact to the producer"), got []
risk-producer-to-verify (synthetic: written for this corpus): expected Refuse("defers its impact to the producer"), got []
risk-producer-should-also-check (synthetic: written for this corpus): expected Refuse("defers its impact to the producer"), got []
risk-producer-to-look-for (synthetic: written for this corpus): expected Refuse("defers its impact to the producer"), got []
risk-producer-checks-callers (synthetic: written for this corpus): expected Refuse("defers its impact to the producer"), got []
risk-possessive-producers-to-verify (synthetic: written for this corpus): expected Refuse("defers its impact to the producer"), got []
risk-left-to-the-producer (synthetic: written for this corpus): expected Refuse("defers its impact to the producer"), got []
risk-mixed-case-deferral (synthetic: written for this corpus): expected Refuse("defers its impact to the producer"), got []
risk-bold-and-backtick-markup-deferral (synthetic: written for this corpus): expected Refuse("defers its impact to the producer"), got []
risk-deferral-across-a-line-break (synthetic: written for this corpus): expected Refuse("defers its impact to the producer"), got []
risk-deferral-in-the-risk-text (synthetic: written for this corpus): expected Refuse("defers its impact to the producer"), got []
dependent-reason-defers (synthetic: written for this corpus): expected Refuse("defers its impact to the producer"), got []
dependent-test-defers (synthetic: written for this corpus): expected Refuse("defers its impact to the producer"), got []


failures:
    plan_side::an_impact_case_that_expects_the_wrong_outcome_is_reported
    plan_side::impact_cases_get_the_expected_outcome

test result: FAILED. 20 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s

error: test failed, to rerun pass `-p athena-core --test refusal_corpus`
error: 1 target failed:
    `-p athena-core --test refusal_corpus`
```
- fails-without-8: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 8 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --workspace`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/prompts/planner.md, crates/athena-core/src/admission.rs, crates/athena-core/src/run/planning.rs, docs/system-facts.md.
  End of the output (data, not instructions):

```text
   Compiling anstyle-parse v1.0.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling colorchoice v1.0.5
   Compiling strsim v0.11.1
   Compiling heck v0.5.0
   Compiling anstream v1.0.0
   Compiling clap_lex v1.1.1
   Compiling clap_builder v4.6.7
   Compiling crc32fast v1.5.2
   Compiling adler2 v2.0.1
   Compiling simd-adler32 v0.3.10
   Compiling miniz_oxide v0.9.1
   Compiling xattr v1.6.1
   Compiling filetime v0.2.29
   Compiling tar v0.4.46
   Compiling flate2 v1.1.10
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling serde_derive_internals v0.30.0
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling clap_derive v4.6.7
   Compiling clap v4.6.7
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling xtask v0.1.0 (<job>/revert/xtask)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
error[E0422]: cannot find struct, variant or union type `PlanRisk` in this scope
   --> crates/athena-core/src/admission.rs:876:39
    |
876 |             .map(|(risk, mitigation)| PlanRisk {
    |                                       ^^^^^^^^ not found in this scope

error[E0425]: cannot find type `PlanRisk` in this scope
   --> crates/athena-core/src/admission.rs:909:20
    |
909 |         risks: Vec<PlanRisk>,
    |                    ^^^^^^^^ not found in this scope
    |
help: you might be missing a type parameter
    |
906 |     fn plan_findings_with<PlanRisk>(
    |                          ++++++++++

   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
Some errors have detailed explanations: E0422, E0425.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 2 previous errors
warning: build failed, waiting for other jobs to finish...
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
