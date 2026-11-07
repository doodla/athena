# Job 20261107-200727-xwss: verify "Make the reading check cover ontology terms that were misread"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: Athena turns request wording into a reading without checking it against the outcome the user will ob
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: ambiguous-term-misread-as-narrow-deliverable (20261014-082433-5rw0)
> Cause: Athena turns request wording into a reading without checking it against the outcome the user will observe. For verbs like publish, deploy or share, the real goal is a user-visible state, such as being reachable at a URL. Nothing requires a ...
> Seen 3 time(s) in 3 jobs. The introspections that record it:
> Introspection 20261014-082423-9m1q (job 20261014-082328-g6vy); prevention it proposed: Change reading generation so that every reading of a request is written as an observable outcome: what the user can do or see when the work is done. For delivery verbs such as publish, deploy, share or release, the reading must name the acc...
> Introspection 20261014-174922-2n9v (job 20261014-174815-7bf8); prevention it proposed: Add a step to reading acceptance. For any request that uses a figurative or structural term (ladder, layer, tier, publish, and similar), the reading must define the term in terms of what it ranks or does, and give an observable acceptance c...
> Introspection 20261017-174855-d7s1 (job 20261017-173508-0wtc); prevention it proposed: Keep a glossary of project terms and owner conventions in the repo. It would hold the terms above, each with its meaning and source, plus the rule to refer to Sri by name or as "they". Load it into every reading step. Add a check that flags...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: a reading is not checked against the outcome the user will observe. The delivery-verb, governance and structural-term checks already close the g6vy and 7bf8 cases, but 20261017-173508-0wtc (the Hermes misread) is still not caught. This job extends `check_defined_terms` so a request that uses an ambiguous name recorded in docs/ontology.toml (Hermes) is refused until the reading restates it in a `Term Hermes:` assumption. The audience is the Athena reading gate and Sri.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (`refusing_check` runs check_prior_result_and_scope, check_criteria_evidence, check_code_survey, check_history_fixture, check_design_existing_code, check_governance_coverage, check_outcome_state, check_environment_reach, check_defined_terms, then the boundary and reference checks; `check_defined_terms` reads `STRUCTURAL_TERMS`, `defined_terms` and `plain_request_prose`), crates/athena-core/tests/refusal_corpus.rs (CITED_REQUESTS, `delivery_verb_replay_over_the_cited_jobs`, `defined_terms_replay_over_the_cited_jobs`, which assert 1 of 3 and read CHANGELOG.md), and crates/athena-cli/tests/ontology.rs (validates docs/ontology.toml). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched. The property is measured directly by running `refusing_check` on the recorded request of 20261017-173508-0wtc with a reading that has no `Term Hermes:` line.
- Existing job state: the checks for the three jobs are on main already. Job 20261014-082328-g6vy (publish an explainer page) is covered by check_outcome_state and check_governance_coverage; its candidate files (pages.yml, xtask/src/site.rs, xtask/src/main.rs, xtask/tests/policy.rs, .gitignore, README.md, docs/design/explainer-site.md, docs/design/requirements-map.md, CHANGELOG.md) are not redone: the page work is dropped here and left to its own follow-up. Job 20261014-174815-7bf8 landed and passed, its findings were scope notes, and its work is kept untouched. Job 20261017-173508-0wtc was rejected for the Hermes definition; docs/ontology.toml on main now holds both Hermes senses, so that finding no longer applies, and this job keeps that file and its test and extends the reading check to cover the remaining class.
- Term ladder: not used by the request in its own prose; the word appears only in quoted introspection text that Athena wrote.
- The ambiguous-name list is a small constant in reading.rs, starting with Hermes (the one ontology term whose second sense caused a reading misread); a test requires every name on it to be a term in docs/ontology.toml, so the list cannot drift from the shared vocabulary.
- Matching follows the existing rules of check_defined_terms: whole words in plain prose, skipping quoted, backtick, path and generated lines. The error names the term and the repair `Term Hermes: <meaning of at least three words>`.
- Expected 2% of newly refused readings, an estimate: few requests name Hermes outside quotes or generated text. Past readings should be refused because the request now asks the reader to define an ambiguous recorded name before building on it.
- Environment: none; no CI workflow, push or pull request is involved, and the change reads only the request, the reading and a repo file included at compile time.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: a test replays the recorded request of 20261017-173508-0wtc through `refusing_check` with a reading that has no `Term Hermes:` assumption and asserts check_defined_terms refuses it naming Hermes; the same reading with a `Term Hermes:` line of three or more words is accepted. The test fails on the code before the change.
2. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: the replay over the three cited jobs now asserts check_defined_terms catches 2 of 3 (7bf8 and 0wtc), check_outcome_state still catches g6vy, and CHANGELOG.md states the same counts.
3. [behavior] Run `cargo test -p athena-core`: cases show the class, not one job: other requests using Hermes in plain prose are refused, while Hermes inside quotes, backticks, paths or Athena-generated lines (root-cause and approval requests) is not, and a request with no listed name is unchanged.
4. [behavior] Run `cargo test -p athena-core`: a test reads docs/ontology.toml and fails if any name on the ambiguous-name list in reading.rs is not a `[[term]]` name there.
5. [content] The doc comment of `check_defined_terms` in crates/athena-core/src/reading.rs names the root cause (readings are not checked against the outcome the user observes), cites jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc, states that it refuses more readings with `expected 2% of newly refused readings`, and no longer says the Hermes term is uncovered; the error text of the check is updated to cover ambiguous names.
6. [content] CHANGELOG.md has an entry naming the root cause and the three jobs, and says which checks cover which job.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass with no new warnings.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Publishing the explainer page or any pages.yml, xtask/src/site.rs, xtask/src/main.rs, xtask/tests/policy.rs, .gitignore, README.md, docs/design/explainer-site.md or docs/design/requirements-map.md work from job 20261014-082328-g6vy: dropped, not redone here
- Redoing the steward, roles and ladder work of job 20261014-174815-7bf8: kept as landed, no files of it change
- Rewriting docs/ontology.toml or crates/athena-cli/tests/ontology.rs from job 20261017-173508-0wtc: kept as they are on main
- Changes to athena.toml, defaults.toml, workflows, Cargo files or lint configuration
- Adding ontology terms not caused by a misread, or wiring the whole ontology into agent prompts
- Prompt files under the reader, verifier or other prompt sets (no token headroom)

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in athena-core and its tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New corpus and unit tests; fmt, clippy and test are criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Extends the existing reading gate where structural-term misreads originate, with a sync test against the ontology.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the replay and name the refusal and acceptance.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a check in refusing_check and a drift test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check extended, a few files.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Checks that the recorded 0wtc request is refused until the reading defines the term.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment and CHANGELOG name cause and jobs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Replay the recorded 0wtc request through the real gate with the file-only reading, once with no Term Hermes assumption, once with a three-or-more-word one, and once with a one- or two-word one. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: With no Term line the result is Some(("check_defined_terms", error)) and the error holds `Hermes` and `Term Hermes:`. With `Term Hermes: the studied self-improving agent, not part of Athena` the result is None. With `Term Hermes: agent` it is still refused.
  Fails without the behavior: Before the change refusing_check returns None for the 0wtc request with no Term Hermes line, so the expect-refused and Some(("check_defined_terms", _)) assertions in defined_terms_replay_over_the_cited_jobs fail. Without the three-word rule the two-word-meaning case would be accepted when it must be refused.
  Test seam: Production entry point: athena_core::reading::refusing_check(&reading, request, None, &[], Some(&repo_files())), pub and imported in refusal_corpus.rs (exists). Fixture: base_reading(request) (exists) builds the reading and the test sets reading.assumptions directly. The request is the recorded 0wtc text, moved from the CITED_REQUESTS tuple into `const REQUEST_0WTC` in plan step 2. Guards (capability set, no question, non-empty deliverables) hold in base_reading. No double.
- Criterion 2: Replay the three CITED_REQUESTS through refusing_check; collect the jobs whose first refusing check is check_defined_terms and those caught by check_outcome_state. Then read CHANGELOG.md through include_str! and look for the counts. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: check_defined_terms catches exactly 20261014-174815-7bf8 and 20261017-173508-0wtc. check_outcome_state catches exactly 20261014-082328-g6vy. The whitespace-normalised changelog holds `check_defined_terms covers 2 of the 3 cited jobs` and `covers 1 of the 3 cited jobs` (for check_outcome_state), holds `are not caught by this check`, and does not hold `3 of the 3 cited jobs`.
  Fails without the behavior: Before the change `caught` in defined_terms_replay_over_the_cited_jobs is [7bf8] only, so assert_eq!(caught, [7bf8, 0wtc]) fails, and the 0wtc branch of delivery_verb_replay_over_the_cited_jobs, which now expects check_defined_terms, fails. Without the CHANGELOG edit the claim `check_defined_terms covers 2 of the 3 cited jobs` is not found.
  Test seam: refusing_check over CITED_REQUESTS with base_reading and repo_files() (all exist, refusal_corpus.rs about lines 478-614). The changelog is read with include_str!("../../../CHANGELOG.md") as at lines 541 and 601 (exists). The test edits are plan steps 2-3; the changelog edit is step 6.
- Criterion 3: Unit tests in reading.rs and corpus cases, run together by the athena-core test run. The unit test defined_terms_refuses_an_ambiguous_ontology_name drives check_defined_terms through terms_error; the corpus cases drive refusing_check through case(). Run: `cargo test -p athena-core`. Observes: Refused: `Compare Hermes with Athena.`, `Hermes's skills differ.`, lowercase `hermes`, Hermes in an Acceptance bullet, an approved-work title that uses Hermes. Accepted: Hermes in double, curly, backtick and single quotes, a parenthesised example list, `docs/hermes.md`, `Hermes::new`, `hermeshub`, `Hermesian`, the copied ## Source section of an approval request, the introspection line of a root-cause request, and a request with no listed name (existing case structural-term-no-structural-word-is-accepted). A Hermes definition does not define ladder and the reverse. `Term hermes:` with three or more words defines it, with fewer does not.
  Fails without the behavior: Before the change every Hermes request below is accepted, so each Expected::Refuse("check_defined_terms") corpus case and each terms_error(..) expectation of Some fails. If quote, path, identifier or generated-line skipping were lost, the accept cases fail. If matching were over-broad, the hermeshub and Hermesian cases fail.
  Test seam: Production entry points: check_defined_terms (pub) through terms_error and terms_reading (reading.rs about 10532-10543, exist), and refusing_check through the corpus case() helper (refusal_corpus.rs about line 210, exists). Generated-text fixtures use the real generators generated_approval_request (about line 743) and root_cause_request (about line 249), both exist. New unit test: step 4. New corpus cases: step 3.
- Criterion 4: Parse the real ontology text and compare the two lists with its term names in both directions; also assert the lists are non-empty, every ungated reason has at least three words, and the controls report the injected faults. Run: `cargo test -p athena-core ambiguous_and_ungated_terms_cover_the_ontology`. Observes: For the real file the problem list is empty: Hermes is in AMBIGUOUS_NAMES, and planning, job, Athena, product versus system, landed and Sri are in UNGATED_TERMS. Each control returns a non-empty list naming the injected fault.
  Fails without the behavior: If a name were listed that is not an ontology term, or a [[term]] were added to docs/ontology.toml and put in neither list (or in both), the test fails. The controls show it can fail in each direction: a listed `Zeus` is reported as missing, an ontology list extended with `Zeus` is reported as unclassified, and a name in both lists is reported.
  Test seam: Private items AMBIGUOUS_NAMES and UNGATED_TERMS (both added in plan step 4, visible to the child tests module) and crate::mirrors::parse_ontology(&str) -> Result<Vec<OntologyTerm>, String> (pub, mirrors.rs:880, exists) with OntologyTerm.name (mirrors.rs:668). Fixture: the real docs/ontology.toml via include_str!("../../../docs/ontology.toml"), the same depth as the existing ../../../CHANGELOG.md include in reading.rs. The helper `ontology_problems_for(listed, ungated, ontology_names)` is new (step 4); it holds only set comparison and is passed the lists as arguments so the controls call it with injected data.
- Criterion 5: A content test over the `///` block above `pub fn check_defined_terms(` (helper defined_terms_doc, exists) plus a call to check_defined_terms on a Hermes request to read the error. The updated defined_terms_refusal_is_declared_intended requires `expected 2% of newly refused readings` in the doc and changelog and keeps `expected 3%` only for D164. Run: `cargo test -p athena-core ambiguous_name_doc_and_error_text`. Observes: The doc contains `readings are not checked against the outcome the user observes`, the three job ids, `covers 2 of the 3 cited jobs`, `expected 2% of newly refused readings`, and the stated one-sense limit; it does not contain `whose recorded request holds no word of the list`. The error contains `ambiguous name`, `Term Hermes:`, the wording that asks for which recorded sense applies and for docs/ontology.toml, has no newline, and starts `the reading was refused because`.
  Fails without the behavior: Without the rewrite the doc comment still says the Hermes term is not covered and has no `expected 2%` text, and the error says only `structural term`, so the contains and does-not-contain assertions fail.
  Test seam: Production entry point check_defined_terms (pub) and the source text via include_str!("reading.rs") (the pattern of the existing defined_terms_doc, exists). No double. New test: step 4; edited test: step 4.
- Criterion 6: A content test that reads CHANGELOG.md and docs/decisions.md with include_str! and checks the updated bullets and D165 for needles, and that the old claims it must not break are intact. Run: `cargo test -p athena-core --test refusal_corpus ambiguous_name_changelog_and_decision`. Observes: The whitespace-normalised changelog holds the root-cause sentence, the ids 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc, and states that check_outcome_state and check_governance_coverage cover 20261014-082328-g6vy while check_defined_terms covers 20261014-174815-7bf8 and 20261017-173508-0wtc. It no longer holds `expected 3%` in the check_defined_terms bullet and does not hold `3 of the 3 cited jobs`. D165 holds `**Alternatives:**`, `**Provenance:**`, `amends D164`, the same root-cause sentence, `expected 2% of newly refused readings` and the one-sense limit. D164's text is unchanged, so the earlier D164 test still passes.
  Fails without the behavior: Without the CHANGELOG edit and D165 the needles (the root-cause sentence, the three job ids, check_outcome_state, check_governance_coverage, check_defined_terms, `covers 2 of the 3 cited jobs`) are not found.
  Test seam: The two files themselves, included at compile time as at refusal_corpus.rs about lines 541, 601 and 618 (exist). Test added in plan step 3. No double.
- Criterion 7: Run the three Athena checks. Also run `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`; the workspace test run covers every crate that reaches check_reading_acceptance. Run: `cargo fmt --all --check`. Observes: All three exit 0 with no warning output. The athena-core unit and refusal_corpus tests, the athena-cli ontology and mirrors tests (unchanged files) and the athena-context briefs tests pass.
  Fails without the behavior: A formatting error, a clippy warning (an unused `structural_term` left after the rename, dead_code on AMBIGUOUS_NAMES or UNGATED_TERMS) or a broken existing test makes the respective command fail.
  Test seam: The cargo commands themselves; nothing to inject. UNGATED_TERMS is #[cfg(test)] and AMBIGUOUS_NAMES is used by non-test named_term, so neither is dead code.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 2% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base f4303dd387f3 → candidate e18b6b2f9359. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  43 ++--
 crates/athena-core/src/reading.rs          | 376 ++++++++++++++++++++++++++---
 crates/athena-core/tests/refusal_corpus.rs | 266 +++++++++++++++++---
 docs/decisions.md                          |  54 +++++
 4 files changed, 654 insertions(+), 85 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261107-200727-xwss/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-200727-xwss/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42412732186 for e18b6b2f93598d81202b8c26b3f7ac3221f68941)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42412732186 for e18b6b2f93598d81202b8c26b3f7ac3221f68941)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42412732186 for e18b6b2f93598d81202b8c26b3f7ac3221f68941)
- ci-result: `not a check` passed (Check results for e18b6b2f93598d81202b8c26b3f7ac3221f68941 come from the CI run https://github.com/doodla/athena/actions/runs/42412732186 (708s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`: all pass with no new warnings.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
test defined_terms_decision_and_changelog ... ok
test defined_terms_replay_over_the_cited_jobs ... FAILED
test delivery_verb_replay_over_the_cited_jobs ... FAILED
test design_existing_code_refusal_is_declared_intended ... ok
test environment_reach_refusal_is_declared_intended ... ok
test an_extra_check_with_no_case_is_reported ... ok
test generated_request_has_the_generators_shapes ... ok
test corpus_covers_the_listed_input_forms ... ok
test no_check_bypasses_refusing_check ... ok
test corpus_cases_get_the_expected_outcome ... FAILED
test plan_side::a_plan_case_that_expects_the_wrong_outcome_is_reported ... ok
test plan_side::open_question_cases_get_the_expected_outcome ... ok
test plan_side::the_complete_ci_reuse_case_names_the_rule_the_search_finds ... ok
test plan_side::plan_cases_get_the_expected_outcome ... ok
test queued_work_holds_a_proposed_request_whose_stored_reading_leaves_a_term_undefined ... ok
test history_fixture_refusal_is_declared_intended ... ok
test every_acceptance_check_has_a_refusing_corpus_case ... ok
test the_wrapper_agrees_with_refusing_check_on_every_case ... ok

failures:

---- ambiguous_name_changelog_and_decision stdout ----

thread 'ambiguous_name_changelog_and_decision' (15042) panicked at crates/athena-core/tests/refusal_corpus.rs:1182:9:
the changelog lacks "readings are not checked against the outcome the user observes, so structural terms stay undefined, and so do ambiguous ontology names such as Hermes"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- defined_terms_replay_over_the_cited_jobs stdout ----

thread 'defined_terms_replay_over_the_cited_jobs' (15197) panicked at crates/athena-core/tests/refusal_corpus.rs:589:5:
assertion `left == right` failed: caught ["20261014-174815-7bf8"] of 3
  left: ["20261014-174815-7bf8"]
 right: ["20261014-174815-7bf8", "20261017-173508-0wtc"]

---- delivery_verb_replay_over_the_cited_jobs stdout ----

thread 'delivery_verb_replay_over_the_cited_jobs' (15204) panicked at crates/athena-core/tests/refusal_corpus.rs:550:41:
the ontology request is refused

---- corpus_cases_get_the_expected_outcome stdout ----

thread 'corpus_cases_get_the_expected_outcome' (15045) panicked at crates/athena-core/tests/refusal_corpus.rs:2066:5:
ambiguous-name-hermes-undefined-in-0wtc-request (job 20261017-173508-0wtc, its recorded request, verbatim; the reading is the file-only reading `base_reading` builds): expected Refuse("check_defined_terms"), got accepted
ambiguous-name-possessive-and-lowercase-use-is-refused (the request in the brief of job 20261107-200727-xwss; rebuilt, not a recorded request): expected Refuse("check_defined_terms"), got accepted
ambiguous-name-lowercase-use-is-refused (the request in the brief of job 20261107-200727-xwss; rebuilt, not a recorded request): expected Refuse("check_defined_terms"), got accepted
ambiguous-name-in-acceptance-bullet-is-refused (the request in the brief of job 20261107-200727-xwss; rebuilt, not a recorded request): expected Refuse("check_defined_terms"), got accepted
ambiguous-name-approve-request-title-use-is-refused (this job, 20261107-200727-xwss; text built by the real generator): expected Refuse("check_defined_terms"), got accepted


failures:
    ambiguous_name_changelog_and_decision
    corpus_cases_get_the_expected_outcome
    defined_terms_replay_over_the_cited_jobs
    delivery_verb_replay_over_the_cited_jobs

test result: FAILED. 16 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.33s

error: test failed, to rerun pass `-p athena-core --test refusal_corpus`
error: 1 target failed:
    `-p athena-core --test refusal_corpus`
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find value `AMBIGUOUS_NAMES` in this scope
     --> crates/athena-core/src/reading.rs:11092:33
      |
11092 |         let listed: Vec<&str> = AMBIGUOUS_NAMES.to_vec();
      |                                 ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `UNGATED_TERMS` in this scope
     --> crates/athena-core/src/reading.rs:11093:42
      |
11093 |         let ungated: Vec<(&str, &str)> = UNGATED_TERMS.to_vec();
      |                                          ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `named_term` in this scope
     --> crates/athena-core/src/reading.rs:11102:24
      |
11102 |             assert_eq!(named_term(&name.to_lowercase()), Some(*name));
      |                        ^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 3 previous errors
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core ambiguous_and_ungated_terms_cover_the_ontology`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find value `AMBIGUOUS_NAMES` in this scope
     --> crates/athena-core/src/reading.rs:11092:33
      |
11092 |         let listed: Vec<&str> = AMBIGUOUS_NAMES.to_vec();
      |                                 ^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `UNGATED_TERMS` in this scope
     --> crates/athena-core/src/reading.rs:11093:42
      |
11093 |         let ungated: Vec<(&str, &str)> = UNGATED_TERMS.to_vec();
      |                                          ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `named_term` in this scope
     --> crates/athena-core/src/reading.rs:11102:24
      |
11102 |             assert_eq!(named_term(&name.to_lowercase()), Some(*name));
      |                        ^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 3 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core ambiguous_name_doc_and_error_text`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-core --test refusal_corpus ambiguous_name_changelog_and_decision`). Result: not run: only the first few distinct commands are run.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo fmt --all --check`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
