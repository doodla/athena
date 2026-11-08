# Job 20261027-054239-v4kj: verify "Require a stated check method for subjective-quality outcomes"

## Request (verbatim)

> Priority: normal
> 
> Fix the root cause of a repeated failure: The system has no default for how to verify subjective quality such as how clear a page is to a newc
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: acceptance-method-unspecified-needs-user-answer (20261014-163954-85d6)
> Cause: The system has no default for how to verify subjective quality such as how clear a page is to a newcomer. Requests state the outcome without a verification method. Athena's reading step does not require an acceptance check for such outcomes...
> Seen 3 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261014-163945-t9c3 (job 20261014-163844-c2kt); prevention it proposed: Add a standing default: when a deliverable is a page or document whose quality is subjective, the acceptance check is an independent model reader that had no part in building it. It reads as a newcomer, rates the page 1 to 5, and blocks shi...
> Introspection 20261014-165951-resy (job 20261014-163844-c2kt); prevention it proposed: When a reading's acceptance criterion depends on a subjective or model-judged rating, admission should require the reading to state the check as a rubric before the build starts. It should name the reader persona and prompt, the rating dime...
> Introspection 20261015-064256-fmsv (job 20261015-064153-wekz); prevention it proposed: Add defaults that apply when a request has a quality outcome or a loop. (1) The outcome is the check behind the work, not the activity. Audience-facing quality is judged by a fresh model reader, with no part in building the work, who rates ...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the reading gate (`refusing_check` in crates/athena-core/src/reading/checks.rs) accepts a reading whose outcome is subjective quality, such as a page being clear to a newcomer, with no criterion saying how that is judged. This produced jobs 20261014-163844-c2kt and 20261015-064153-wekz, whose verifiers then failed or asked. I add a new `check_subjective_method` step that refuses such a reading until a criterion names an independent model reader, a persona, a 1–5 rating and a pass threshold, so the whole class is prevented at reading time.

Assumptions:
- Goal and audience: the owner and the verifier of future jobs. A reading that states a subjective outcome must also state how it is checked, so no job ends in 'needs user answer' or a verifier failure on an unverifiable outcome.
- Code survey: read crates/athena-core/src/reading/checks.rs (`refusing_check` runs ordered `step!` checks; a new check is one step line), crates/athena-core/src/reading/surveys.rs (`check_criteria_evidence` shows the shape of a check, error text and tests), crates/athena-core/tests/refusal_corpus.rs (every step needs a corpus case in which it refuses), and crates/athena-context/src/briefs.rs (reading rules list each check). No record or data file is added under crates/, so `check_record_file`, `AnyRecord` and the round-trip test do not apply. The property is measured directly by calling `check_subjective_method` on a built Reading and through `refusing_check`.
- Existing code: extends crates/athena-core/src/reading/surveys.rs and crates/athena-core/src/reading/checks.rs; reuses `has_phrase` and the `reading_with_criterion` test helper in crates/athena-core/src/reading.rs.
- The check triggers on a fixed lowercase whole-word list in the request or summary (for example newcomer, clear, readable, understandable, useful, easy to follow, quality of a page or explainer). It is satisfied by one criterion that has an independent or fresh reader/model, a rating scale and a pass threshold. The corpus tests tune this list.
- The reader prompt has 3 tokens of headroom, so no prompt or defaults.toml text is added; the mechanism is the check and its error message. The brief's reading-rules line in briefs.rs may gain one line, if its tests allow.
- Governance: athena.toml, defaults.toml and CI workflows are not changed; no criterion needs them.

## Acceptance criteria

1. [content] crates/athena-core/src/reading/surveys.rs defines `pub fn check_subjective_method`, with a doc comment that names the root cause (no required verification method for subjective quality) and jobs 20261014-163844-c2kt and 20261015-064153-wekz; Grep for `fn check_subjective_method` finds it.
2. [content] crates/athena-core/src/reading/checks.rs `refusing_check` has exactly one new line `step!(check_subjective_method(...))`, and the check_reading_acceptance doc comment lists it.
3. [behavior] Run `cargo test -p athena-core`: a unit test in crates/athena-core/src/reading.rs builds a reading whose request or summary says a page must be clear to a newcomer and whose criteria give no method; `check_subjective_method` refuses it and the error names the missing independent reader, rating and threshold. The same test fails if the new step is removed from `refusing_check`.
4. [behavior] Run `cargo test -p athena-core`: a unit test shows a reading with the same subjective outcome and a criterion naming an independent model reader that had no part in building it, a newcomer persona, a 1 to 5 rating and a minimum score is accepted; a reading with no subjective wording (for example a plain refactor) is unaffected.
5. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: the corpus has new cases built from the shapes of jobs 20261014-163844-c2kt and 20261015-064153-wekz, expecting `Expected::Refuse("check_subjective_method")`, and at least one accepted counterpart; the corpus test that every step has a refusing case passes.
6. [content] Expected 5% of newly refused readings: the rule refuses more readings and the higher refusal rate is intended, but it triggers only on subjective-quality wording, so it must not refuse readings of ordinary code requests.
7. [content] Job 20261014-163844-c2kt's findings are addressed by moving the fix to admission: its newcomer.rs gate, prompt asset and ship-path wiring are dropped and not redone here (see out_of_scope). Job 20261015-064153-wekz's findings about `athena land --pr` and retry budget are dropped as a different cause. The check's error text tells the reader to state the method before the build.
8. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` from the repository root: all pass.
9. [content] CHANGELOG.md has an entry naming the root cause and the new check; Grep for `check_subjective_method` in CHANGELOG.md matches.

## Expected deliverables

- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/design/defaults-loop.md (modify)

## Out of scope

- Job 20261014-163844-c2kt candidate files are dropped, not kept: CHANGELOG.md newcomer entry, Cargo.lock, crates/athena-cap-study/Cargo.toml, crates/athena-cap-study/prompts/newcomer.md, crates/athena-cap-study/src/lib.rs, crates/athena-cap-study/src/newcomer.rs, docs/architecture.md, docs/design/explainer-site.md. Wiring a newcomer gate into the pages ship path.
- Job 20261015-064153-wekz candidate files are dropped, not kept: README.md, crates/athena-cli/src/display.rs, crates/athena-cli/src/drain.rs, crates/athena-cli/src/lib.rs, crates/athena-cli/src/main.rs, crates/athena-cli/tests/drain.rs, crates/athena-core/src/lib.rs, crates/athena-core/src/steward.rs, docs/decisions.md. Drain landing through `athena land --pr`.
- Changes to athena.toml, defaults.toml, CI workflows, lint configuration or prompt files
- Rewriting the firstmate explainer or calibrating a model reader against human ratings

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is at the reading gate where the missing method originates, not at either job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced as a refusing check in `refusing_check`, not prose.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests and refusal-corpus cases; fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test cases that show refusal and acceptance.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The new check requires the outcome's check to be named.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check, its tests and a changelog entry.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comment names the root cause and the jobs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check by test plus Grep. `subjective_method_refusal_is_declared_intended` (added in step 10) reads the shipped source with the existing `reading_source()` (refusal_corpus.rs:2114-2125 already include_str!s surveys.rs) and asserts the doc block above `pub fn check_subjective_method(` contains 'no required verification method for subjective quality', '20261014-163844-c2kt', '20261015-064153-wekz', 'This check refuses more readings; the higher refusal rate is intended.' and 'estimate'. Also Grep for `pub fn check_subjective_method` in crates/athena-core/src/reading/surveys.rs. Run: `cargo test -p athena-core --test refusal_corpus subjective_method_refusal_is_declared_intended`. Observes: The function exists with a doc comment that names the cause and both jobs; the test passes.
  Fails without the behavior: Without the function or its doc text the Grep finds nothing and the doc assertions fail.
  Test seam: Entry point: the source text of surveys.rs via `reading_source()` (exists, refusal_corpus.rs:2114; the test is added by step 10). No double.
- Criterion 2: Unit test `subjective_method_has_one_step_line` (reading.rs, step 9, modelled on `history_fixture_has_one_step_line` at reading.rs:7501) splits the `refusing_check` body of checks.rs and asserts exactly one line `step!(check_subjective_method(reading, request));` that sits after `check_reading_references`, and that the `check_reading_acceptance` doc text holds the link to `check_subjective_method`. Also read the candidate diff of checks.rs. Run: `cargo test -p athena-core subjective_method_has_one_step_line`. Observes: One step line and the doc link; the test passes.
  Fails without the behavior: The test fails with zero or two step lines or with no doc link.
  Test seam: Entry point: checks.rs source text through the same include-based source read that `history_fixture_has_one_step_line` uses (exists); the test is added by step 9.
- Criterion 3: Unit test in reading.rs. Build `terms_reading(&[])` (reading.rs:6614, a docs reading that passes every check) with the request 'Write an explainer page on how firstmate handles work that is clear to a newcomer.' and criteria that give no method. Call the production `refusing_check` through `refuser_for` (reading.rs:6609), `check_subjective_method` and `check_reading_acceptance`. Assert the refuser is Some("check_subjective_method"), the error starts 'the reading was refused because' and contains 'independent', 'persona', 'rating', 'threshold', 'behavior' and 'before the build', and has no newline. A second assertion uses a generated root-cause request whose own sentence asks for the same outcome: refused too. A third assertion: 'how clear the explainer page is to a first-time reader' is refused (clarity tier with an audience word). Run: `cargo test -p athena-core subjective_method_missing_is_refused_and_named`. Observes: The gate refuses under the check's own name and the error names the missing independent model reader, persona, 1 to 5 rating and pass threshold.
  Fails without the behavior: Without the `step!` line `refuser_for` returns None and the Some assert fails. Without the function, or with a trigger that misses 'clear to a newcomer', `unwrap_err()` panics. If the error lacks the missing parts the string asserts fail.
  Test seam: Production entry points `refusing_check`, `check_reading_acceptance` and `check_subjective_method`; fixtures `terms_reading` and `refuser_for` exist (reading.rs:6609-6618); function and step line are added by steps 2-5, the test by step 7. No double: the reading is a plain struct value built in the test.
- Criterion 4: Unit tests in reading.rs (step 7). (a) `subjective_method_stated_is_accepted`: the criterion-3 reading plus a Behavior criterion 'Run `athena audience`: an independent model reader that had no part in building the page reads it as a newcomer and rates it from 1 to 5; a rating below 4 blocks completion.' is accepted by `check_reading_acceptance(&reading, request, None, &[], None)`; further accepted variants: the defaults.toml wording 'a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion' (finding 1), '4/5', 'out of 5', 'at least 4', '4 or above', and a persona 'a new user' (finding 3). (b) `subjective_method_partial_is_refused`: variants drop one part at a time (independent reader, persona, scale, threshold); each is refused and the error names exactly the dropped part; a variant with the threshold word and the scale digits but no digit next to the threshold word is refused; a variant with the full method only in a Content criterion is refused. (c) `subjective_method_ignores_other_requests`: 'Refactor the page renderer into two functions', 'Clear the cache when the page reloads', 'Make each call site readable', 'Make the page output machine-readable', 'Make the JSON output human-readable', 'Improve the clarity of the verifier report', 'Make the error in the report clear to the operator', a quoted mention (Rename the "newcomer" page template) and a root-cause request whose cause line quotes the newcomer outcome are all accepted. Run: `cargo test -p athena-core subjective_method`. Observes: The full Behavior method (including the user's own default wording) is accepted, each partial or content-only method is refused naming the gap, ordinary and technical-clarity requests pass.
  Fails without the behavior: If the check demanded more than the four parts or detected none, the accept assertions fail. If the scale matcher lacked '<digit> of 5', the defaults-wording variant is refused and fails. If it ignored the Behavior kind, the content-only refusal fails. If the trigger ignored the artifact noun, the audience-word rule for clarity phrases, the hyphen boundary or quotes, the ordinary requests are refused. If it refused everything, the plain refactor fails.
  Test seam: Same production entry points as criterion 3; `Reading` built with existing `terms_reading`; request strings are inputs; no double. Tests added by step 7.
- Criterion 5: Step 10 adds `subjective_cases()` chained in `cases()` (refusal_corpus.rs:281), built with the existing `case(...)`, `with_behavior` and `base_reading`, run through `refusing_check` by `run` (refusal_corpus.rs:2039). Input forms handled and their cases (new tags appended to INPUT_FORMS, array length 41 grows by the number added, and `corpus_covers_the_listed_input_forms` at line 2097 requires each tag to have a case): subjective-outcome-no-method → Refuse cases subjective-explainer-page-no-method (source 20261014-163844-c2kt, rebuilt from the job brief), subjective-loop-page-no-method (shape of 20261015-064153-wekz: a loop improving the explainer page until it is easy to follow, rebuilt from the brief), subjective-root-cause-own-sentence-no-method (root-cause request helper with the sentence 'The explainer page must be clear to a newcomer.' in the request's own first paragraph), subjective-clear-to-first-time-reader-no-method; subjective-method-stated → Accept subjective-explainer-page-method-stated, subjective-loop-page-method-stated, and subjective-defaults-sentence-method-accepted (the Behavior criterion is 'Run `athena audience`: ' plus the outcome-check sentence from defaults.toml, extracted from `include_str!("../../../defaults.toml")` between 'For audience-facing quality' and 'blocks completion.', not retyped; finding 1), subjective-new-user-persona-method-stated (finding 3); subjective-word-without-artifact → Accept subjective-clear-the-cache, subjective-machine-readable-output, subjective-call-site-readable, subjective-plain-refactor; subjective-technical-clarity → Accept subjective-clarity-of-verifier-report and subjective-clear-to-operator-report (finding 2); subjective-quoted-or-generated → Accept subjective-quoted-newcomer and subjective-cited-in-root-cause-request (generated request whose cause and prevention text quote the newcomer outcome); subjective-method-partial-or-content-only → Refuse subjective-method-without-threshold, subjective-method-without-independent-reader, subjective-method-in-content-criterion-only. The existing `generated_root_cause_request` (refusal_corpus.rs:742) hard-codes its cause; step 10 moves its body into a helper taking the cause text, prevention text and sentence, and the old function calls it. Each Refuse case expects `Expected::Refuse("check_subjective_method")`. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: Every new Refuse case is refused by exactly check_subjective_method, the accepted counterparts pass, and the coverage tests pass. I cannot run the file; I cite only the ids this plan adds.
  Fails without the behavior: Without the step line the Refuse cases return accepted and `corpus_cases_get_the_expected_outcome` fails. `every_acceptance_check_has_a_refusing_corpus_case` fails if no case is refused by this check. `corpus_covers_the_listed_input_forms` fails if a new tag has no case. The defaults-sentence Accept case fails if the scale matcher lacks '<digit> of 5'.
  Test seam: Entry point `refusing_check` via corpus `run`; fixtures `case`, `with_behavior`, `base_reading`, `repo_files` and `generated_root_cause_request` exist; the cases, the tags, the defaults.toml include_str! and the helper split are added by step 10.
- Criterion 6: `subjective_method_refusal_is_declared_intended` mirrors `environment_reach_refusal_is_declared_intended` (refusal_corpus.rs:2265): `athena_core::reading::replay_declaration` on a Behavior criterion 'The change refuses more readings: expected 5% of newly refused readings' gives share Some(5); the collapsed CHANGELOG (`include_str!`, as at refusal_corpus.rs:2255) holds 'check_subjective_method', 'higher refusal rate is intended' and 'expected 5% of newly refused readings'; the doc comment calls the 5% an estimate. The narrow trigger is shown by the Accept corpus cases for ordinary and technical-clarity requests (criterion 5). Run: `cargo test -p athena-core --test refusal_corpus subjective_method_refusal_is_declared_intended`. Observes: The declared share parses as 5; the text is in the doc and the changelog; the ordinary-request cases are accepted by the corpus run.
  Fails without the behavior: If the changelog or doc omits the declared share, or the parser reads another number, the test fails. If the trigger is too wide, the accepted ordinary-request cases fail.
  Test seam: `replay_declaration` is public and exists (used at refusal_corpus.rs:2217); CHANGELOG read with `include_str!` as sibling tests do; test added by step 10.
- Criterion 7: Read the candidate diff's file list: it holds only the seven deliverables (surveys.rs, checks.rs, reading.rs, refusal_corpus.rs, CHANGELOG.md, briefs.rs, defaults-loop.md) and none of the dropped paths (crates/athena-cap-study/*, athena-cli/*, README.md, docs/architecture.md, docs/design/explainer-site.md, steward.rs, lib.rs, Cargo.lock). The behavioural part is the criterion-3 test's 'before the build' assertion on the error text; the CHANGELOG entry says the c2kt gate and the wekz land/retry findings are not redone here. Run: `cargo test -p athena-core subjective_method_missing_is_refused_and_named`. Observes: Only the seven deliverables changed; the refusal text tells the reader to state the method before the build.
  Fails without the behavior: If the error text lacked the instruction to state the method before the build, the unit-test assertion fails.
  Test seam: Same as criterion 3 for the text assertion (exists once steps 2-4 and 7 land); the file list is read from the candidate diff with `git diff` or `git status`.
- Criterion 8: Run the three checks Athena runs on the result, from the repository root. The test run includes the athena-context briefs tests (new rule count and byte bound), athena-core unit tests, the refusal corpus, athena-cli tests including tests/drain.rs and budgets (unchanged), and the xtask policy scans. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0 with no warnings and no failed test.
  Fails without the behavior: fmt fails on unformatted new code. clippy (-D warnings) fails on an unused constant or helper if the step line is missing. cargo test fails if the change is incomplete: `the_reader_brief_names_each_reading_rule_in_one_short_line` if the rules list and its test disagree; `every_acceptance_check_has_a_refusing_corpus_case` if the step has no refusing case; `corpus_covers_the_listed_input_forms` if a tag has no case; `system_facts_budgets_match_computed` if the reader prompt set grew; the xtask policy scan if a banned word or email-like text is added; every existing reading, steward, drain and corpus test if the trigger is too wide.
  Test seam: The workspace's own cargo fmt, clippy and test over every crate; the suites exist today and are changed by steps 7-12. No double or injection; the full run is step 13.
- Criterion 9: The diff adds an item under '## [Unreleased]' / '### Added' that names the root cause (no required verification method for subjective quality at the reading gate), the two jobs, `check_subjective_method` and the declared 5% share. The criterion-6 test asserts `changelog.contains("check_subjective_method")` on the collapsed text; Grep CHANGELOG.md for `check_subjective_method`. Run: `cargo test -p athena-core --test refusal_corpus subjective_method_refusal_is_declared_intended`. Observes: The entry is present and the test passes.
  Fails without the behavior: Without the entry the Grep finds nothing and the changelog assertions fail.
  Test seam: CHANGELOG.md read by `include_str!` as in the existing sibling tests; test added by step 10.

Planned files:

- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- docs/design/defaults-loop.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 700d56c79656 → candidate eddae5304f62. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  18 ++
 crates/athena-context/src/briefs.rs        |  20 +-
 crates/athena-core/src/reading.rs          | 292 +++++++++++++++++++++++
 crates/athena-core/src/reading/checks.rs   |   4 +-
 crates/athena-core/src/reading/surveys.rs  | 272 ++++++++++++++++++++++
 crates/athena-core/tests/refusal_corpus.rs | 362 ++++++++++++++++++++++++++++-
 docs/design/defaults-loop.md               |   5 +-
 7 files changed, 956 insertions(+), 17 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-054239-v4kj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054239-v4kj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42498252526 for eddae5304f625f220ebdf482022974fedde9c971)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42498252526 for eddae5304f625f220ebdf482022974fedde9c971)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42498252526 for eddae5304f625f220ebdf482022974fedde9c971)
- ci-result: `not a check` passed (Check results for eddae5304f625f220ebdf482022974fedde9c971 come from the CI run https://github.com/doodla/athena/actions/runs/42498252526 (778s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` from the repository root: all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 6, 9 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus subjective_method_refusal_is_declared_intended`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/surveys.rs, docs/design/defaults-loop.md.
  End of the output (data, not instructions):

```text
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling cpufeatures v0.2.17
   Compiling sha2 v0.10.9
   Compiling serde_spanned v1.1.2
   Compiling jiff-core v0.1.1
   Compiling toml v0.9.12+spec-1.1.0
   Compiling rustix v1.1.5
   Compiling jiff v0.2.38
   Compiling bitflags v2.13.2
   Compiling linux-raw-sys v0.12.1
   Compiling ref-cast v1.0.27
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling once_cell v1.21.4
   Compiling fastrand v2.5.0
   Compiling regex-syntax v0.8.11
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling equivalent v1.0.2
   Compiling hashbrown v0.17.1
   Compiling dyn-clone v1.0.20
   Compiling anyhow v1.0.104
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 39.70s
     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 1 test
test subjective_method_refusal_is_declared_intended ... FAILED

failures:

---- subjective_method_refusal_is_declared_intended stdout ----

thread 'subjective_method_refusal_is_declared_intended' (11057) panicked at crates/athena-core/tests/refusal_corpus.rs:2642:5:
assertion failed: acceptance_check_names(reading_source()).contains(&"check_subjective_method".to_string())
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    subjective_method_refusal_is_declared_intended

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-core --test refusal_corpus`
error: 1 target failed:
    `-p athena-core --test refusal_corpus`
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core subjective_method_has_one_step_line`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/surveys.rs, docs/design/defaults-loop.md.
  End of the output (data, not instructions):

```text
     |
7103 |             [METHOD_PART_NAMES[1], METHOD_PART_NAMES[3]].join("; ")
     |              ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `METHOD_PART_NAMES` in this scope
    --> crates/athena-core/src/reading.rs:7103:36
     |
7103 |             [METHOD_PART_NAMES[1], METHOD_PART_NAMES[3]].join("; ")
     |                                    ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:6882:9
     |
6882 |         check_subjective_method(reading, request)
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:6972:9
     |
6972 |         check_subjective_method(&reading, &root_cause("")).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:6981:13
     |
6981 |             check_subjective_method(&reading, SUBJECTIVE_REQUEST)
     |             ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:7019:13
     |
7019 |             check_subjective_method(&plain, request).unwrap();
     |             ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:7023:9
     |
7023 |         check_subjective_method(&asking, SUBJECTIVE_REQUEST).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:7026:9
     |
7026 |         check_subjective_method(&none, SUBJECTIVE_REQUEST).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:7030:9
     |
7030 |         check_subjective_method(&declined, SUBJECTIVE_REQUEST).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:7125:13
     |
7125 |             check_subjective_method(&reading, request).unwrap_or_else(|e| panic!("{request}: {e}"));
     |             ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 13 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3, 7 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core subjective_method_missing_is_refused_and_named`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/surveys.rs, docs/design/defaults-loop.md.
  End of the output (data, not instructions):

```text
     |
7103 |             [METHOD_PART_NAMES[1], METHOD_PART_NAMES[3]].join("; ")
     |              ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `METHOD_PART_NAMES` in this scope
    --> crates/athena-core/src/reading.rs:7103:36
     |
7103 |             [METHOD_PART_NAMES[1], METHOD_PART_NAMES[3]].join("; ")
     |                                    ^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:6882:9
     |
6882 |         check_subjective_method(reading, request)
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:6972:9
     |
6972 |         check_subjective_method(&reading, &root_cause("")).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:6981:13
     |
6981 |             check_subjective_method(&reading, SUBJECTIVE_REQUEST)
     |             ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:7019:13
     |
7019 |             check_subjective_method(&plain, request).unwrap();
     |             ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:7023:9
     |
7023 |         check_subjective_method(&asking, SUBJECTIVE_REQUEST).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:7026:9
     |
7026 |         check_subjective_method(&none, SUBJECTIVE_REQUEST).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:7030:9
     |
7030 |         check_subjective_method(&declined, SUBJECTIVE_REQUEST).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_subjective_method` in this scope
    --> crates/athena-core/src/reading.rs:7125:13
     |
7125 |             check_subjective_method(&reading, request).unwrap_or_else(|e| panic!("{request}: {e}"));
     |             ^^^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 13 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core subjective_method`). Result: not run: only the first few distinct commands are run.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core --test refusal_corpus`). Result: not run: only the first few distinct commands are run.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
