# Job 20261027-054230-8nz8: verify "Refuse delivery-verb readings that name no user-visible outcome"

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

Root cause: reading acceptance never compared a delivery verb (publish, deploy, share, release) with the state the user will observe, so jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc were read as narrow file deliverables. Add `check_outcome_state` to reading acceptance. When the request uses such a verb, it requires a criterion that names the observable state, and its error says what to add. Retries of the three earlier jobs are addressed by what the check enforces, not by redoing their changes.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs. The entry point is check_reading_acceptance, which calls refusing_check; refusing_check runs one step! line per check in order: prior result, criteria evidence, code survey, design existing code, governance coverage, boundary and repo files, references. The new check is added as one step! line, placed after check_criteria_evidence. crates/athena-core/tests/refusal_corpus.rs asserts that every step! has a corpus case in which it refuses. This change adds no record or data file under crates/, so check_record_file, AnyRecord and the round-trip test are not involved. The property is measured directly by calling check_outcome_state on a Reading and by running real request shapes through refusing_check.
- Code survey: crates/athena-context/src/briefs.rs holds READING_RULES, and a test asserts exactly 6 rules. The reader prompt has 3 tokens of headroom, so the brief is not changed. The rule reaches the reader through the refusal text, which Athena::read_for feeds into one re-read.
- Job 20261014-082328-g6vy: its governance-file miss is already prevented by check_governance_coverage. What remains is the underlying misread, publish read as 'write a page' rather than 'reachable at a URL'. Its candidate files (xtask site, pages.yml, docs, README, CHANGELOG and others) are not kept or redone by this job; they are listed in out_of_scope as dropped.
- Job 20261014-174815-7bf8: it landed and passed. Its findings about extra files and the A10-1 status are not this cause, and the files it changed are kept as they are and listed in out_of_scope. Its ladder/figurative-term finding is addressed only for delivery verbs. Defining ladder-style terms is left to the ontology work.
- Job 20261017-173508-0wtc: it was rejected for the Hermes definition and its relations in docs/ontology.toml. That is a content error in a file this job does not create, so the candidate is dropped and the glossary proposal is out of scope. The shared cause, a term misread as a narrow deliverable, is covered by the delivery-verb check.
- Goal and audience: the goal is that a reading states what the user will see when the work is done. The audience is the user who judges the result and the verifier who checks the criteria. Observable state words accepted by the check include a URL, reachable, served, live at, downloadable, installable, a tag, a published release, and visible to. The word list is small and deliberately conservative.
- The check applies only when the reading has a capability and deliverables, the same guard as the other checks. A request word counts only as a whole word, matched with has_phrase. Expected N% of newly refused readings is a low number, since delivery verbs are rare among requests.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`. A new unit test in crates/athena-core/src/reading.rs shows check_outcome_state refusing a reading for the request 'Publish a first explainer page on how firstmate handles work.' whose criteria only say a file exists. The error names the missing observable state, such as the URL or reachability. The test fails when the check is removed.
2. [behavior] Run `cargo test -p athena-core`. A unit test shows the same request accepted when a criterion names a reachable URL or served state. Further cases cover deploy, share and release, and show no refusal when the request has no delivery verb, for example 'Add a timeout setting'.
3. [content] Grep crates/athena-core/src/reading.rs shows a `step!(check_outcome_state(` line inside refusing_check, with a doc comment that names the root cause (a delivery verb read as a narrow deliverable, with no check against the state the user observes) and cites jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc.
4. [behavior] Run `cargo test -p athena-core --test refusal_corpus`. The corpus has a case in which check_outcome_state refuses and a matching accepting case. The corpus test still passes, including its rule that every step! has a refusing case.
5. [behavior] The change refuses more readings. Expected 5% of newly refused readings. A replay of past readings is expected to hold the job only above 10%. The higher refusal rate is intended: the check applies only to requests with a delivery verb whose reading names no observable state.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All three pass, including the briefs.rs test that asserts READING_RULES has 6 entries.
7. [content] CHANGELOG.md has an entry that names the root cause and the new check. No governance file (athena.toml, defaults.toml, workflows, toolchain or lint config) changes, and none is named in a deliverable.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Redoing job 20261014-082328-g6vy: xtask site, .github/workflows/pages.yml, docs/design/explainer-site.md, README.md, .gitignore, xtask/tests/policy.rs and requirements-map.md edits are dropped
- Job 20261014-174815-7bf8's landed files (steward, roles, ladder) are kept and not touched
- Job 20261017-173508-0wtc's docs/ontology.toml, ontology test and decisions entry are dropped; no glossary or ontology is built here
- Changing the reader prompt or READING_RULES in crates/athena-context/src/briefs.rs, which has 3 tokens of headroom
- Defining figurative terms such as ladder, layer or tier, beyond delivery verbs
- Changes to athena.toml, defaults.toml, .github/workflows or toolchain and lint configuration
- Enabling GitHub Pages or any deployment

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix sits in reading acceptance, where a delivery verb is turned into a reading, not in one job's output.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests and corpus cases fail without the check; fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that show the refusal and the acceptance happening.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced as a step! check, not prose.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check, with tests and corpus cases.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check requires an observable user outcome in the reading.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit test `check_outcome_state_refuses_a_publish_request_whose_criteria_only_say_a_file_exists` in the tests module of reading.rs. Run: `cargo test -p athena-core --lib check_outcome_state`. Observes: The test builds a reading with `governed_reading(&["docs/explainer.md"], &["docs/explainer.md exists"], &["the logo"])`, which is a Content criterion of an existing helper, and the request 'Publish a first explainer page on how firstmate handles work.' It asserts that `check_outcome_state` returns Err. The message contains 'publish', 'URL' and 'reachable'. It also asserts that `refusing_check(...)` returns `Some(("check_outcome_state", _))`.
  Fails without the behavior: Without check_outcome_state and its step! line, the module does not compile. If only the step! line is removed, `refusing_check` returns None for the file-only reading and the test fails on the assertion that it names `check_outcome_state`.
  Test seam: Entry points: `check_outcome_state(&Reading, &str, Option<&EarlierAttempt>)` and `refusing_check`. The fixture is the existing `governed_reading` helper (via `scoped_reading`, reading.rs:6157, 7013), passed in as arguments and with no double. The test calls production code. The function and the step! line are added by plan steps 1 and 2; the test is added by step 5.
- Criterion 2: Unit tests `check_outcome_state_accepts_a_reading_that_names_the_state`, `check_outcome_state_covers_deploy_share_and_release` and `check_outcome_state_leaves_other_requests_alone`. Run: `cargo test -p athena-core --lib check_outcome_state`. Observes: For publish, deploy, share and release (e.g. 'Deploy the site on every push', 'Share the status report with the team', 'Release the tool as a tagged version'): a file-only reading is refused. The same reading with a criterion such as 'The page is reachable at https://example.org/explainer' or 'The site is served at its URL' is accepted. A criterion with a tag, 'downloadable' or 'installable' is accepted for release. An `out_of_scope` item 'Publishing is not done here' is accepted. 'Add a timeout setting', 'Release notes need a March entry', 'Share of failed jobs is shown' and the mid-sentence 'verbs like publish, deploy or share' are accepted. A reading that declines, asks a question or lists no deliverable is accepted. A `delivery_verb` test over '- Publish', '* Deploy', '1. Share', '1) Release' and '(a) Publish' finds the verb.
  Fails without the behavior: If the state-word list or the verb scan were missing or always refused, the accept cases would fail. If the check always refused, the 'Add a timeout setting' and noun-use cases would fail. If it accepted file-only readings, the refuse half of each verb case would fail.
  Test seam: Entry points: `check_outcome_state` and the private `delivery_verb` and `names_observable_state`. The fixture is the `governed_reading` helper (exists). Requests and criteria are plain string arguments and nothing is doubled. The tests are added by step 5.
- Criterion 3: A source-reading unit test, in the style of `design_existing_code_refusal_is_declared_intended` at reading.rs:8619-8653. Run: `cargo test -p athena-core --lib check_outcome_state_doc_names_the_root_cause`. Observes: The test takes the `refusing_check` body from the `include_str!("reading.rs")` source and asserts it contains `step!(check_outcome_state(`. It takes the doc paragraph before `pub fn check_outcome_state(` (lines all starting `///`). It asserts the paragraph contains a delivery verb read as a narrow deliverable, no check against the state the user observes, 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc. The Grep tool can confirm the same text.
  Fails without the behavior: If the step! line were missing, the `acceptance_check_names`-style scan finds no `step!(check_outcome_state(`. If the doc comment lacks the cause sentence or any of the three job ids, the assertion on the doc text fails.
  Test seam: Entry point: `include_str!("reading.rs")` on production source, with no double. The test follows the existing pattern; the doc comment and step! line are added by steps 1 and 2; the test by step 5.
- Criterion 4: Run the whole refusal-corpus test file, with the new cases in `reading_check_cases()`. Cases the producer adds: `publish-reading-names-no-observable-state` (Refuse check_outcome_state), `publish-reading-names-a-reachable-url` (Accept), `delivery-verb-in-a-dash-bullet` and `delivery-verb-in-a-star-bullet` (Refuse), `delivery-verb-in-numbered-dot-parts`, `delivery-verb-in-numbered-paren-parts` and `delivery-verb-in-lettered-paren-parts` (Refuse), `retry-complaint-earlier-request-has-a-delivery-verb` (Refuse) and `retry-complaint-earlier-request-with-state-criterion` (Accept), `root-cause-request-names-the-verbs-mid-sentence` (Accept), and `release-notes-noun-use` (Accept). The existing case `brand-word-and-placeholder-are-not-repo-paths` stays Accept because a criterion naming a reachable GitHub URL is pushed onto its reading. Existing ids I read and reuse unchanged: `steward-three-dash-bullets`, `star-bullets`, `numbered-dot-parts`, `numbered-paren-parts-not-read-today`, `lettered-paren-parts-not-read-today`, `root-cause-cites-two-jobs-without-relying`, `retry-complaint-reading-names-the-job`, `request-names-a-governance-file-the-reading-ignores`. I have not run any of them; I can cite only what I read. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: Every case gets its expected outcome. The new refuse cases are refused by `check_outcome_state` as the first refusing check. The two retry cases use `c.attempt = Some(earlier(RETRIED_JOB, "Publish a first explainer page ..."))`. The test `every_acceptance_check_has_a_refusing_corpus_case` finds `check_outcome_state` among the step! names with at least one refusing case. `corpus_covers_the_listed_input_forms` accepts the new tags `delivery-verb`, `delivery-verb-mid-sentence` and `delivery-noun-use`, which are added to `INPUT_FORMS` (length 17 → 20). `an_extra_check_with_no_case_is_reported` and `no_check_bypasses_refusing_check` still hold.
  Fails without the behavior: Without the corpus cases, `every_acceptance_check_has_a_refusing_corpus_case` reports 'no corpus case in which ["check_outcome_state"] refuses' once the step! line exists. Without the check, `corpus_cases_get_the_expected_outcome` fails on the refuse cases, which return None.
  Test seam: Entry point: `refusing_check` (via `run(&Case)`, refusal_corpus.rs:660). Fixtures are the `case()`, `base_reading()`, `earlier()` and `deliverable()` helpers (exist). Each new case changes one field of `base_reading`. No double. Cases are added by step 6.
- Criterion 5: A source test in the style of `design_existing_code_refusal_is_declared_intended` (reading.rs:8619). The 5% share is an estimate; nothing here measured it. Run: `cargo test -p athena-core --lib check_outcome_state_refusal_is_declared_intended`. Observes: `replay_declaration(&[Criterion{text:"The change refuses more readings: expected 5% of newly refused readings", ..}], &[])` returns share Some(5). The doc paragraph of `check_outcome_state` contains the intended-refusal sentence. The whitespace-normalised `include_str!("../../../CHANGELOG.md")` contains 'check_outcome_state', 'higher refusal rate is intended' and 'expected 5% of newly refused readings'. The held limit is 5 + REPLAY_DECLARED_MARGIN_POINTS (5) = 10%.
  Fails without the behavior: If the doc comment lacks the sentence 'This check refuses more readings; the higher refusal rate is intended.' or the CHANGELOG lacks 'expected 5% of newly refused readings', the test fails. The replay itself runs at verification (`replay` in reading.rs, REPLAY_WINDOW 50, hold above 10% when nothing is declared) and I could not run it.
  Test seam: Entry points: `replay_declaration` (exists, reading.rs:3986) and `include_str!` of the production doc comment and CHANGELOG. No double. The test is added by step 5; the doc and CHANGELOG text by steps 1 and 7.
- Criterion 6: Run the three project checks. The `READING_RULES` count test in `crates/athena-context/src/briefs.rs` passes because that file is unchanged. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. The tests updated in reading.rs are `a_request_naming_a_governance_path_is_refused_until_it_is_covered` (named and excluded readings get a state criterion), `a_request_with_only_common_words_and_no_governance_path_is_accepted` (its `bare` reading gets a state criterion) and `a_request_naming_a_workflow_file_needs_it_listed_and_named` (the `with_workflow` reading gets a state criterion). In the corpus, `brand-word-and-placeholder-are-not-repo-paths` gets a state criterion.
  Fails without the behavior: If the producer breaks the briefs.rs `READING_RULES` test, formatting or clippy, or leaves an existing test that expects acceptance of a file-only 'Publish ...' reading, one of the three commands fails.
  Test seam: Entry points: the `cargo` commands on the workspace. The existing test files and helpers (`governed_reading`, `case()`) are used. No double. The test updates are in steps 4 and 6.
- Criterion 7: The CHANGELOG assertions are in the test of criterion 5. The diff listing shows the file set. Run: `cargo test -p athena-core --lib check_outcome_state_refusal_is_declared_intended && git diff --name-only`. Observes: The test asserts the normalised CHANGELOG contains 'check_outcome_state' and the root-cause sentence ('delivery verb' and 'no check against the state the user observes'). `git diff --name-only` lists only crates/athena-core/src/reading.rs, crates/athena-core/tests/refusal_corpus.rs, CHANGELOG.md and docs/decisions.md. It lists none of athena.toml, defaults.toml, .github/workflows/*, rust-toolchain.toml or setup.sh.
  Fails without the behavior: If the CHANGELOG entry lacks the root cause or the check name, the test fails. If a governance file is edited, `git diff --name-only` lists it and the verifier rejects the change.
  Test seam: Entry points: `include_str!` of CHANGELOG.md and `git diff`. No double. The CHANGELOG entry is written by step 7.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base b8153689b81d → candidate e1c8e2cfc0a2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  14 +
 crates/athena-core/src/reading.rs          | 413 ++++++++++++++++++++++++++++-
 crates/athena-core/tests/refusal_corpus.rs | 169 +++++++++++-
 docs/decisions.md                          |  44 +++
 4 files changed, 634 insertions(+), 6 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261027-054230-8nz8/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261027-054230-8nz8/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42359815672 for e1c8e2cfc0a2c9db3306c2cd1d02a9727aebb388)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42359815672 for e1c8e2cfc0a2c9db3306c2cd1d02a9727aebb388)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42359815672 for e1c8e2cfc0a2c9db3306c2cd1d02a9727aebb388)
- ci-result: `not a check` passed (Check results for e1c8e2cfc0a2c9db3306c2cd1d02a9727aebb388 come from the CI run https://github.com/doodla/athena/actions/runs/42359815672 (638s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All three pass, including the briefs.rs test that asserts READING_RULES has 6 entries.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --lib check_outcome_state`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
     |
8754 |             check_outcome_state(&stated, request, None)
     |             ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:8763:9
     |
8763 |         check_outcome_state(&declined, "Publish the status page.", None).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:8769:17
     |
8769 |         assert!(check_outcome_state(&elsewhere, "Publish the status page.", None).is_err());
     |                 ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:8786:13
     |
8786 |             check_outcome_state(&file_only, request, None)
     |             ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:8797:13
     |
8797 |             check_outcome_state(reading, PUBLISH_REQUEST, None).unwrap();
     |             ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:8807:17
     |
8807 |         assert!(check_outcome_state(&docs, complaint, Some(&earlier)).is_err());
     |                 ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:8808:17
     |
8808 |         assert!(check_outcome_state(&docs, complaint, None).is_ok());
     |                 ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:8814:9
     |
8814 |         check_outcome_state(&stated, complaint, Some(&earlier)).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `delivery_verb` in this scope
    --> crates/athena-core/src/reading.rs:8831:24
     |
8831 |             assert_eq!(delivery_verb(text), Some(verb), "{text}");
     |                        ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `delivery_verb` in this scope
    --> crates/athena-core/src/reading.rs:8840:24
     |
8840 |             assert_eq!(delivery_verb(text), None, "{text}");
     |                        ^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 14 previous errors
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --lib check_outcome_state_doc_names_the_root_cause`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
     |
8754 |             check_outcome_state(&stated, request, None)
     |             ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:8763:9
     |
8763 |         check_outcome_state(&declined, "Publish the status page.", None).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:8769:17
     |
8769 |         assert!(check_outcome_state(&elsewhere, "Publish the status page.", None).is_err());
     |                 ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:8786:13
     |
8786 |             check_outcome_state(&file_only, request, None)
     |             ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:8797:13
     |
8797 |             check_outcome_state(reading, PUBLISH_REQUEST, None).unwrap();
     |             ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:8807:17
     |
8807 |         assert!(check_outcome_state(&docs, complaint, Some(&earlier)).is_err());
     |                 ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:8808:17
     |
8808 |         assert!(check_outcome_state(&docs, complaint, None).is_ok());
     |                 ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_outcome_state` in this scope
    --> crates/athena-core/src/reading.rs:8814:9
     |
8814 |         check_outcome_state(&stated, complaint, Some(&earlier)).unwrap();
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `delivery_verb` in this scope
    --> crates/athena-core/src/reading.rs:8831:24
     |
8831 |             assert_eq!(delivery_verb(text), Some(verb), "{text}");
     |                        ^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `delivery_verb` in this scope
    --> crates/athena-core/src/reading.rs:8840:24
     |
8840 |             assert_eq!(delivery_verb(text), None, "{text}");
     |                        ^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 14 previous errors
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core --test refusal_corpus`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/reading.rs, docs/decisions.md.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.38s
     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 11 tests
test corpus_covers_the_listed_input_forms ... ok
test corpus_cases_get_the_expected_outcome ... FAILED
test no_check_bypasses_refusing_check ... ok
test plan_side::a_plan_case_that_expects_the_wrong_outcome_is_reported ... ok
test plan_side::open_question_cases_get_the_expected_outcome ... ok
test plan_side::plan_cases_get_the_expected_outcome ... ok
test plan_side::the_complete_ci_reuse_case_names_the_rule_the_search_finds ... ok
test an_extra_check_with_no_case_is_reported ... ok
test every_acceptance_check_has_a_refusing_corpus_case ... ok
test the_wrapper_agrees_with_refusing_check_on_every_case ... ok
test design_existing_code_refusal_is_declared_intended ... ok

failures:

---- corpus_cases_get_the_expected_outcome stdout ----

thread 'corpus_cases_get_the_expected_outcome' (31818) panicked at crates/athena-core/tests/refusal_corpus.rs:854:5:
publish-reading-names-no-observable-state (jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc, a delivery verb read as a narrow deliverable; rebuilt from the job briefs): expected Refuse("check_outcome_state"), got accepted
delivery-verb-in-a-dash-bullet (jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc, a delivery verb read as a narrow deliverable; rebuilt from the job briefs): expected Refuse("check_outcome_state"), got accepted
delivery-verb-in-a-star-bullet (jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc, a delivery verb read as a narrow deliverable; rebuilt from the job briefs): expected Refuse("check_outcome_state"), got accepted
delivery-verb-in-numbered-dot-parts (jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc, a delivery verb read as a narrow deliverable; rebuilt from the job briefs): expected Refuse("check_outcome_state"), got accepted
delivery-verb-in-numbered-paren-parts (jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc, a delivery verb read as a narrow deliverable; rebuilt from the job briefs): expected Refuse("check_outcome_state"), got accepted
delivery-verb-in-lettered-paren-parts (jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc, a delivery verb read as a narrow deliverable; rebuilt from the job briefs): expected Refuse("check_outcome_state"), got accepted
retry-complaint-earlier-request-has-a-delivery-verb (jobs 20261014-082328-g6vy, 20261014-174815-7bf8 and 20261017-173508-0wtc, a delivery verb read as a narrow deliverable; rebuilt from the job briefs): expected Refuse("check_outcome_state"), got accepted
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    corpus_cases_get_the_expected_outcome

test result: FAILED. 10 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

error: test failed, to rerun pass `-p athena-core --test refusal_corpus`
error: 1 target failed:
    `-p athena-core --test refusal_corpus`
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core --lib check_outcome_state_refusal_is_declared_intended`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test -p athena-core --lib check_outcome_state_refusal_is_declared_intended && git diff --name-only`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
