# Job 20261101-000734-04qa: verify "Add design existing-code reading check with reader rule line"

## Request (verbatim)

> try again on 20261027-054248-rjf1

## Reading (accepted by the user)

Retry of job 20261027-054248-rjf1 (which failed in planning, no candidate, nothing kept). Add `check_design_existing_code` to the reading gate, refusing design readings that lack an `Existing code:` assumption naming existing repo paths and a criterion requiring an Existing code section, and teach the rule in READING_RULES so a correct reading passes on the first read. Audience: the reader agent and the maintainers who judge design jobs.

Assumptions:
- Job 20261027-054248-rjf1 failed in planning with no candidate and no changed files; its work is dropped and this retry starts from the base. Addressing its critique: (major) READING_RULES gets a sixth line, because BudgetSet::Reader covers only CONTRACT, READER_FRAME, defaults and catalog (athena-context/src/lib.rs:279-283), so the reader-prompt headroom claim of 3 tokens does not apply to it; (minor) the criterion must match the phrase 'existing code section' plus 'reuse' or 'extend', so 'the existing code in section 3 is unchanged' and 'no survey needed' are refused; (minor) paths the reading lists as create deliverables are ignored when deciding whether the Existing code: assumption names a path; (nit) the doc comment has no blank lines and the signature is exactly `pub fn check_design_existing_code(`.
- Code survey: crates/athena-core/src/reading.rs (refusing_check at ~3042 with step! calls, check_code_survey at 3375, names_crates_path at 3303, the doc-comment test at ~7958), crates/athena-core/tests/refusal_corpus.rs (base_reading, repo_files, INPUT_FORMS, code_survey_refusal_is_declared_intended), crates/athena-context/src/briefs.rs (READING_RULES [&str; 5] at 1694, line tests near 3389-3412), crates/athena-context/src/lib.rs (BudgetSet::Reader). No record or data file is added, so check_record_file and AnyRecord do not apply. The property is measured by running the real refusing_check on constructed readings.
- Existing code: the check reads only the reading (deliverable paths, assumptions, criteria); it accepts a path that starts with crates/ or docs/ and is not a create deliverable. It does not search the repo, since the reader has no such step.
- Design readings with a question or a decline, and readings of other classes, are not affected by the new check.
- The new rule line is at most 220 bytes, READING_RULES stays under 2400 bytes, and the added text has no banned words or email-like text.
- Expected refusal rate: 5% of newly refused readings, since only design-class readings are affected.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core`: unit tests in crates/athena-core/src/reading.rs show a design reading with an `Existing code:` assumption naming a path that is not a create deliverable, and a criterion requiring an Existing code section that reuses or extends it, is accepted. They fail without the change.
2. [behavior] Run `cargo test -p athena-core`: a design reading whose criterion contains 'existing code' and 'section' but says the opposite (for example 'no survey needed' or 'the existing code in section 3 is unchanged') is refused.
3. [behavior] Run `cargo test -p athena-core`: a design reading whose `Existing code:` assumption names only the file the job creates is refused, as is one with no such assumption.
4. [behavior] Run `cargo test -p athena-core`: readings of class code, tests, docs and config, and design readings with a question or a decline, are still accepted.
5. [behavior] Run `cargo test -p athena-core --test refusal_corpus`: a new case built from a design reading written from request text alone is refused by check_design_existing_code, is declared intended, and the guard that every step! has a refusing case passes.
6. [behavior] Run `cargo test -p athena-context`: READING_RULES has six lines, one naming `Existing code:` and `check_design_existing_code`; the line-count, per-line 220-byte, section under 2400 bytes, every-check-has-a-rule-line and budget tests pass.
7. [content] Grep of crates/athena-core/src/reading.rs shows `step!(check_design_existing_code(reading));` in refusing_check, and a doc comment without blank lines naming the root cause (no inventory of existing code before a design is accepted) and jobs 20261026-043137-zc0v and 20261026-065046-0c1d; the refusal text names the `Existing code:` label and what to list.
8. [behavior] The change refuses more readings and the higher refusal rate is intended: expected 5% of newly refused readings.
9. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` from the repository root: all pass.
10. [content] git diff shows no change to athena.toml, defaults.toml, .github/workflows, docs/design/jev-code-health.md or docs/decisions.md.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing or reverting the landed work of jobs 20261026-043137-zc0v and 20261026-065046-0c1d (docs/design/jev-code-health.md, docs/decisions.md); it is kept
- Changing jev.rs, labeled.rs or the score-jev command
- Any model-call or repository-search step in the reader; the check reads only the reading
- Changing athena.toml, defaults.toml or workflows
- A general check that resolves token-budget claims in assumptions

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust in athena-core and athena-context.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit and corpus tests plus fmt, clippy and test criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the check on accepted and refused readings.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed at the reading gate and in the reader brief, for the whole design class.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A refusing check plus a corpus guard enforce the rule.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in four files.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in reading.rs `mod tests`. A new `design_reading(assumptions, criteria, deliverables)` helper is built on `scoped_reading` (line 5716) with class set to design. The accepting test goes through `accept` (line 7772, calls check_reading_acceptance, no repo_files). It uses assumption `Existing code: crates/athena-core/src/reading.rs and docs/design/jev-code-health.md`, a Create deliverable at another path, and criterion 'The design has an Existing code section that reuses or extends check_code_survey'. Variants use 'extends', 'reuses' and a lower-case label. Run: `cargo test -p athena-core design_existing_code`. Observes: Ok(()) from the real gate for each compliant design reading. A paired assertion in the same test calls `refusing_check` on the same reading with the assumption removed and expects the step name `check_design_existing_code`.
  Fails without the behavior: The accepted-versus-refused pair fails: the step name is absent from refusing_check without the change, and the test does not compile without the new function.
  Test seam: Production entry points `check_reading_acceptance` and `refusing_check` (exist). Fixtures `scoped_reading` and `accept` exist. The `design_reading` helper is added in plan step 4. No double.
- Criterion 2: Unit test through `accept` with a design reading that has a valid `Existing code:` assumption. Criteria are tried one at a time: 'no survey needed'; 'the existing code in section 3 is unchanged'; 'no existing code section is needed, nothing to reuse or extend'; 'existing code section' with no reuse or extend word; a reuse word more than 12 words after the phrase. Run: `cargo test -p athena-core design_existing_code`. Observes: Each returns Err whose text contains `Existing code` and the phrase 'existing code section'.
  Fails without the behavior: A check matching 'existing code' and 'section' anywhere would accept the first two. A check with no negator guard would accept the third. Without the check all five are accepted.
  Test seam: `check_reading_acceptance` via `accept` (exists). The `design_reading` helper is added in step 4. The criterion strings are the fixture; no double.
- Criterion 3: Unit tests through `accept`. (a) A Create deliverable at docs/design/new.md with the only assumption `Existing code: docs/design/new.md`. (b) No `Existing code:` assumption, with a `Code survey:` assumption naming crates/ present to show it does not count. (c) `Existing code: the reading module` with no path. Each has a valid criterion. A fourth test uses a Modify deliverable on the same path as (a) and expects acceptance. Run: `cargo test -p athena-core design_existing_code`. Observes: Err for (a), (b) and (c), with text naming the `Existing code:` label and saying to list existing paths the job does not create. The Modify case is Ok.
  Fails without the behavior: Without the created-path filter (a) passes. Without the assumption requirement (b) and (c) pass.
  Test seam: `check_reading_acceptance` via `accept` (exists). The `Deliverable` list is the injected fixture through `design_reading` (step 4). No double.
- Criterion 4: Unit test mirroring `code_survey_does_not_apply_outside_its_gate`. For class code (with SURVEY), tests (with SURVEY), docs and config, with no Existing code assumption, assert `accept` is Ok. Also assert Ok for a design reading with a question set, and one with capability None and a decline_reason. Call `check_design_existing_code` directly on each as well. Run: `cargo test -p athena-core design_existing_code`. Observes: Ok(()) from the gate and from the check for every case.
  Fails without the behavior: A check that did not skip other classes, questions or declines would refuse them and the test would fail.
  Test seam: `check_reading_acceptance` and the new `pub fn check_design_existing_code` (added in step 3). Fixtures `code_reading` and `scoped_reading` exist.
- Criterion 5: Add cases to `reading_check_cases()` in refusal_corpus.rs, built with `case()` and `base_reading()` from request text alone (a design request in the shape of job 20261026-043137-zc0v: a code-health score design with two bullets), class set to design. `design-reading-without-existing-code`: no assumptions, Refuse("check_design_existing_code"). `design-reading-names-only-the-created-file`: Create docs/design/code-health-new.md and an assumption naming only it, Refuse. `design-reading-with-opposite-criterion`: valid assumption, criterion 'no survey needed: the existing code in section 3 is unchanged', Refuse. `design-reading-with-existing-code`: assumption naming crates/athena-core/src/reading.rs and docs/design/roadmap.md (both in repo_files()), criterion 'The design has an Existing code section that reuses or extends crates/athena-core/src/reading.rs', Accept. Add `design_existing_code_refusal_is_declared_intended`, modelled on the existing declared-intended test, which checks `replay_declaration` returns share Some(5), that READING_SOURCE has the step line, and that the doc paragraph has the intended sentence, both job ids and no `///` blank line. I read the existing test names and the guard at line 660; I did not run them. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: corpus_cases_get_the_expected_outcome and the_wrapper_agrees_with_refusing_check_on_every_case pass with the new cases. every_acceptance_check_has_a_refusing_corpus_case passes, so the new step has a refusing case. corpus_covers_the_listed_input_forms passes with the new tag. The declared-intended test passes.
  Fails without the behavior: Without the check, the first case is accepted and the mismatch list names it. Without the case, every_acceptance_check_has_a_refusing_corpus_case reports check_design_existing_code uncovered.
  Test seam: `refusing_check` (exists, pub) driven by `run()` in the corpus. `case`, `base_reading` and `repo_files` exist. The cases, the 16th INPUT_FORMS entry and the declaration test are added in step 6.
- Criterion 6: Extend the existing test `the_reader_brief_names_each_reading_rule_in_one_short_line` in briefs.rs. READING_RULES becomes `[&str; 6]`. The check-name list gains `check_design_existing_code`. A new assertion requires one rendered line to contain both `Existing code:` and `check_design_existing_code`. Run: `cargo test -p athena-context`. Observes: Six rule lines render. Each is at most 220 bytes. The section is under 2400 bytes. Every check name appears in it. The budget tests pass because the reader budget set excludes READING_RULES.
  Fails without the behavior: Without the new line, the line-count assertion (6) and the name assertion for `check_design_existing_code` fail.
  Test seam: `reader_brief` with the `reader_inputs` fixture (exist, briefs.rs near 3371). The constant line is added in step 5.
- Criterion 7: Grep the source and Read the new doc comment. The corpus declaration test also asserts the same facts from READING_SOURCE. The Grep patterns are `step!\(check_design_existing_code\(reading\)\);`, `20261026-043137-zc0v` and `Existing code:` in crates/athena-core/src/reading.rs. Run: `cargo test -p athena-core --test refusal_corpus design_existing_code_refusal_is_declared_intended`. Observes: The step line is inside `refusing_check`. The doc comment is one paragraph with no `///`-only line. It names the root cause (no inventory of existing code before a design is accepted) and both job ids. The bail texts contain `Existing code:` and what to list.
- Criterion 8: The `design_existing_code_refusal_is_declared_intended` test: `replay_declaration` of 'expected 5% of newly refused readings' returns share Some(5). The doc paragraph and CHANGELOG.md contain 'refuses more readings' and 'higher refusal rate is intended'. The refusing corpus cases show readings the base accepted are now refused. Run: `cargo test -p athena-core --test refusal_corpus`. Observes: Share Some(5), both sentences present, and the refusing cases fail acceptance only at the new step.
  Fails without the behavior: Without the doc sentence or changelog entry the declaration test fails. Without the check, the refusing cases are accepted.
  Test seam: `replay_declaration` (exists, pub). READING_SOURCE exists via include_str!. The CHANGELOG include is added in step 6.
- Criterion 9: Run the three gate commands at the repository root, each as its own simple command. The last two cover the whole workspace: athena-core unit and corpus tests, athena-context rule-line tests, the athena-cli budget tests (`system_facts_budgets_match_computed`) and the xtask policy scan for banned words. Run: `cargo fmt --check`. Observes: fmt exits 0 with no diff. Then `cargo clippy --all-targets` exits 0 with no warnings (Athena runs it with `--locked -- -D warnings`). Then `cargo test` exits 0 with every crate's tests passing, including the new tests from criteria 1-6.
  Fails without the behavior: fmt fails on mis-formatted new code. clippy fails on a lint in the new helpers. test fails if the tests are missing or red: the corpus every-step guard reports `check_design_existing_code` uncovered, the rule-line test fails if READING_RULES is not 6 lines or the section exceeds 2400 bytes, and the policy scan fails on a banned word or email-like text in the new text.
  Test seam: Production entry points are the cargo fmt, clippy and test commands over the whole workspace. Fixtures are the existing tests in athena-core, athena-context, athena-cli and xtask, which exist. Nothing is injected and no double is used. No plan step adds a seam; step 8 runs them.
- Criterion 10: Inspect the diff paths. Run: `git diff --stat`. Observes: Only reading.rs, refusal_corpus.rs, briefs.rs and CHANGELOG.md appear. athena.toml, defaults.toml, .github/workflows, docs/design/jev-code-health.md and docs/decisions.md are absent.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 2a198f9ee8ee → candidate 2b3ae8d75363. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |  13 ++
 crates/athena-context/src/briefs.rs        |  14 +-
 crates/athena-core/src/reading.rs          | 298 +++++++++++++++++++++++++++++
 crates/athena-core/tests/refusal_corpus.rs | 107 ++++++++++-
 4 files changed, 429 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1864 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` from the repository root: all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
