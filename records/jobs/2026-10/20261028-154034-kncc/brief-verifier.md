# Job 20261028-154034-kncc: verify "Require a code survey in readings; declare the intended higher refusal"

## Request (verbatim)

> try again on 20261028-082300-y4yj

## Reading (accepted by the user)

Retry of job 20261028-082300-y4yj, which stopped at plan critique because the reading had no intended-refusal criterion, so the replay would hold the job. Same fix as before (a 'Code survey:' acceptance check, a reader-brief line and tests), now with a criterion saying the higher refusal rate is intended, and with the critic's minor findings addressed. Audience: the reader stage and the verifier.

Assumptions:
- Job 20261028-082300-y4yj blocker (no intended-refusal criterion, so the replay holds the job): fixed by criterion 2, which uses an INTENDED_REFUSAL_PHRASES phrase ('refuses more readings', 'higher refusal rate is intended').
- Job 20261028-082300-y4yj minor finding on signal C (any mention of records/ in a criterion): fixed by dropping signal C. A new-output requirement is triggered only by a create deliverable for a record or data file under crates/ or by a records.rs change, so a criterion saying 'records/ is not changed' does not trigger it.
- Job 20261028-082300-y4yj minor finding on single keyword evidence: fixed by requiring the survey to name check_record_file, AnyRecord, or a named round-trip test function or path, not a bare term such as 'allow-list'.
- Job 20261028-082300-y4yj nit on 'git diff --stat': fixed by phrasing the scope criterion as a file list that contains only the listed files, leaving the tool to the verifier.
- The root cause is the reading stage designing storage, override and test seams from a general idea of the feature without surveying the records validation and the code (introspections 20261028-055651-dhh0 and 20261028-082240-77as; jobs 20261028-055601-hg0n and 20261028-082151-67fa). The fix sits in check_reading_acceptance and the reader brief, where readings are accepted.
- The user's retry request takes precedence over the follow-up proposed in introspection 20261028-154006-ebdw, so this reading retries the original code-survey fix and does not build the separate intended-refusal check.
- The reader brief text lives in briefs.rs READING_RULES, outside the 2000-token reader prompt set, and the existing briefs token-budget test guards it. run/mod.rs changes only if the producer needs it, and the replay itself is not changed.

## Acceptance criteria

1. [content] crates/athena-core/src/reading.rs has a new pub fn check_code_survey, called from check_reading_acceptance. Its doc comment names the root cause: the reading stage designs storage, override and test seams from a general idea of the feature without surveying what the records validation accepts or where resolution and probing happen in the code. It cites jobs 20261028-055601-hg0n and 20261028-082151-67fa and introspections 20261028-055651-dhh0 and 20261028-082240-77as.
2. [content] The change refuses more readings: change readings of class code or tests with a behavior criterion and no 'Code survey:' assumption naming a crates/ path are refused, and a higher refusal rate is intended. The CHANGELOG.md entry and the doc comment of check_code_survey say so, so the replay gate treats the rise as intended.
3. [behavior] Run `cargo test -p athena-core`: a new unit test calls check_reading_acceptance with a reading of capability change, class_of_work code and one behavior criterion but no assumption starting 'Code survey:' naming a crates/ path. It is refused with an error naming the missing survey. The same reading with such an assumption is accepted. The test fails if the check_reading_acceptance call to check_code_survey is removed.
4. [behavior] Run `cargo test -p athena-core`: a unit test shows that a reading with a create deliverable for a record or data file under crates/ is refused unless the survey names check_record_file, AnyRecord, or a named round-trip test function or path. A survey that only mentions 'allow-list' in passing is refused. A criterion that mentions records/ only to exclude it does not trigger the requirement. Docs-only readings, readings with no capability and readings with only content criteria are still accepted, and the existing reading tests pass unchanged.
5. [content] The reader brief in crates/athena-context/src/briefs.rs tells the reader to write the 'Code survey:' assumption before giving behavior criteria. The survey covers the entry point and ordering of resolution and probing, the records validation allow-list, and how to measure the property directly. It also tells the reader to write an intended-refusal criterion when a request adds or tightens a refusal rule. The briefs token-budget tests pass under `cargo test -p athena-context`.
6. [content] CHANGELOG.md has an entry stating the root cause and the fix. The change's file list contains only crates/athena-core/src/reading.rs, crates/athena-context/src/briefs.rs, CHANGELOG.md and, if needed, crates/athena-core/src/run/mod.rs. athena.toml, defaults.toml and files under .github/workflows are not changed.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/mod.rs (modify)

## Out of scope

- Changing athena.toml, defaults.toml or workflows
- Changing any existing reading rule
- The replay feature from job 20261028-055601-hg0n
- The interrupted-retry takeover and disk changes from job 20261028-082151-67fa
- Changing the planner or critic prompts
- Detecting refusal-rule changes by a flag or by matching diffs
- Adding a switch to skip the new check
- The separate intended-refusal acceptance check proposed in introspection 20261028-154006-ebdw

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in athena-core and athena-context.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests call the real check_reading_acceptance; fmt, clippy and test must pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The check is added at reading acceptance, where the class of failure originates.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run cargo test and check the refusal happening.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced in code in check_reading_acceptance, not in prose only.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The doc comment and CHANGELOG entry name the cause and the jobs.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The behavior criteria run commands the verifier can run.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the reading rules and the reader brief.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff of reading.rs. Entry point: `check_reading_acceptance`. Content criterion, no seam needed. Run: `git diff -- crates/athena-core/src/reading.rs`. Observes: A `pub fn check_code_survey(reading: &Reading) -> Result<()>` exists. `check_reading_acceptance` calls it. Its doc comment names the root cause (the reading stage designs storage, override and test seams from a general idea of the feature without surveying what the records validation accepts or where resolution and probing happen), both job ids and both introspection ids.
- Criterion 2: Unit test calling the real `check_reading_acceptance` and the real `criteria_intend_higher_refusal`. The seam is the `Reading` value built by the existing `scoped_reading` helper and `Criterion` values; both exist today. No double. Also read `git diff -- CHANGELOG.md crates/athena-core/src/reading.rs` for the phrases 'refuses more readings' and 'higher refusal rate is intended'. Run: `cargo test --locked -p athena-core code_survey`. Observes: The refusal happens, and the intended-refusal phrases are recognized, so the replay would not hold the job.
  Fails without the behavior: Test `code_survey_refusal_is_declared_intended` fails if the doc comment, CHANGELOG or refusal behavior is missing. It asserts that `criteria_intend_higher_refusal` is true for the text of criterion 2 and that a code reading with a behavior criterion and no survey is refused through `check_reading_acceptance`.
- Criterion 3: Unit tests `code_survey_missing_is_refused_and_present_is_accepted` and `code_survey_tests_class_is_refused_too`. Entry point is the production `check_reading_acceptance(&reading, request, None, &[])`. Fixture: `scoped_reading(...)` with class_of_work set to "code", capability "change" (default), and one Behavior criterion that cites a command, so `check_criteria_evidence` passes. First case has no 'Code survey:' assumption. Further cases have an assumption without the prefix, a 'Code survey:' assumption without a crates/ path, and a valid one (accepted). The step that adds the tests builds the helper `code_reading(assumptions, deliverables, criteria)` in the tests module. The seam is plain data, with no double. Run: `cargo test --locked -p athena-core code_survey_missing`. Observes: The unsurveyed reading returns Err whose text contains 'Code survey:' and 'crates/'. The same reading with `Code survey: crates/athena-core/src/reading.rs ...` returns Ok.
  Fails without the behavior: Removing the `check_code_survey(reading)?` call from `check_reading_acceptance` makes the refusal assertion fail, because the reading is then accepted.
- Criterion 4: Unit tests `code_survey_new_record_file_needs_named_evidence` and `code_survey_does_not_apply_outside_its_gate`, through `check_reading_acceptance`. Fixtures come from the `code_reading` helper added in the test step. Refused: a Create deliverable `crates/athena-data/src/example.json` with a survey that only says 'the allow-list' or only 'round-trip' in prose. Accepted: the same reading with a survey naming `check_record_file`, `AnyRecord`, or `records_round_trip_every_kind`. A new `crates/foo/Cargo.toml` create, a modify of an existing json, and a criterion 'records/ is not changed' do not trigger the requirement. Still accepted: class docs, capability None, question set, and content-only criteria. The existing reading tests are not edited, and `cargo test -p athena-core` runs them. Run: `cargo test --locked -p athena-core code_survey`. Observes: The refusal names the missing evidence (`check_record_file`, `AnyRecord` or a round-trip test). The accepted cases return Ok. All pre-existing reading tests pass unchanged.
  Fails without the behavior: Without the record-file branch, the 'allow-list only' case is accepted and `code_survey_new_record_file_needs_named_evidence` fails. Without the gate, the docs, no-capability, question and content-only cases are refused and `code_survey_does_not_apply_outside_its_gate` fails.
- Criterion 5: Unit tests in briefs.rs call the real `reader_brief` with the existing `reader_inputs` fixture. Also run `cargo test --locked -p athena-context` for the token-budget tests, and read `git diff -- crates/athena-context/src/briefs.rs`. Run: `cargo test --locked -p athena-context reader_brief`. Observes: The Reading rules section carries a 'Code survey:' line before the criteria guidance and a guide paragraph. The guide names the entry point and order of resolution and probing, the records validation allow-list, measuring the property directly, and writing a criterion saying the change 'refuses more readings' when a request adds or tightens a refusal rule. Every rule line is at most 220 bytes. The budget tests pass.
  Fails without the behavior: `the_reader_brief_tells_the_reader_to_write_a_code_survey` fails if the guide text is absent from `reader_brief`. `the_reader_brief_names_each_reading_rule_in_one_short_line` fails if the rule for `check_code_survey` is absent. Both go through the real `reader_brief(&reader_inputs(&reading()))`.
- Criterion 6: List the changed files and read the CHANGELOG diff with `git diff -- CHANGELOG.md`. Content criterion; the producer snapshot is a git repository, so these git commands are available. Run: `git status --short`. Observes: The changed files are crates/athena-core/src/reading.rs, crates/athena-context/src/briefs.rs and CHANGELOG.md. run/mod.rs, athena.toml, defaults.toml and .github/workflows are absent. The CHANGELOG entry states the root cause and the fix.
- Criterion 7: Run the three Athena checks. As separate whole commands: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: Fails if any new code is unformatted, triggers a clippy warning (for example a needless `clone` or an unused import in the new helpers), or breaks a test. That includes tests that count `READING_RULES` and the system-facts budget test.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 40 of 50 are newly refused by the candidate and were accepted by the base. The criteria state a higher refusal rate is intended, so the job is not held.

- 20261028-124903-wngz: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261028-103330-3d47: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261028-091836-kbnb: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261028-082300-y4yj: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261028-082151-67fa: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261028-081329-r26g: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261028-081326-zzxg: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261028-065915-a41d: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261028-065911-m6m5: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261028-055601-hg0n: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261028-035117-c5qb: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261028-034820-v3rf: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261028-033455-94t4: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261027-225544-q7fe: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261027-160815-k3wz: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261027-123701-77zh: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261027-122909-vvvm: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261027-085401-0qgf: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261027-082521-0sfz: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- 20261027-074457-bx8e: the reading was refused because it changes code with a behavior criterion but has no `Code survey:` assumption that names a crates/ path; the criteria were designed from a general idea of the feature 
- and 20 more (the full list is in replay.json)

## The change

Base 67a461cc108d → candidate 709fb8c32631. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |  12 ++
 crates/athena-context/src/briefs.rs |  50 ++++-
 crates/athena-core/src/reading.rs   | 357 +++++++++++++++++++++++++++++++++++-
 3 files changed, 414 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1577 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
