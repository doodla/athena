# Job 20261028-185350-qjzx: verify "Check readings against boundary words and repo files before acceptance"

## Request (verbatim)

> try again on 20261027-054206-w1y3

## Reading (accepted by the user)

Retry of job 20261027-054206-w1y3 (dispatched, never verified, no findings recorded, so nothing to fix; the work is simply redone). Root cause: check_reading_acceptance in reading.rs validates only prior-result, evidence, survey and governance rules, never the reading's text against the boundary word list or its deliverables against files at HEAD. Add that deterministic check there, for the operator and later readers.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (check_reading_acceptance at ~2269 runs check_prior_result_and_scope, check_criteria_evidence, check_code_survey, check_governance_coverage in that order, and bails with a message on the first failure), crates/athena-context/src/briefs.rs (boundary_scan, boundary_words, boundary_fragments), and crates/athena-core/Cargo.toml (athena-context is already a dependency). The new check goes last in that function and measures the reading's text directly with boundary_scan and its deliverable paths against the snapshot's file list at HEAD. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not touched.
- Job 20261027-054206-w1y3 is addressed by redoing its reading: it had no recorded findings. Deliverables and out_of_scope are kept the same as that job's.
- check_reading_acceptance has no repo access today, so the producer passes it the set of paths at HEAD from the existing snapshot (athena_git::snapshot) and updates its callers; this may need a small change in lib.rs, which is why that file is listed.
- briefs.rs changes only if boundary_scan needs a small public helper (e.g. a hit-word accessor); the word list itself is not changed.
- Rule: a modify or delete deliverable for a path absent at HEAD is refused; a create deliverable for an existing path is refused. Paths are compared as repository-relative strings.
- Past readings: the rule is new and refuses readings that name absent paths or banned words; expected 5% of newly refused readings.

## Acceptance criteria

1. [content] Root cause is named in the CHANGELOG.md entry and in a doc comment: reading output was checked only for prior-result, evidence, survey and governance rules, not against the boundary word list or repo files at HEAD. The check is called from check_reading_acceptance in crates/athena-core/src/reading.rs, not in a prompt file.
2. [behavior] A unit test in reading.rs fails without the change and passes with it: a reading whose summary holds a boundary-scan word is refused and the error names the word. Run `cargo test -p athena-core reading`.
3. [behavior] A unit test shows a 'modify' or 'delete' deliverable for a path absent at HEAD is refused, a 'create' deliverable for an existing path is refused, and a reading with valid paths and clean text is accepted. Run `cargo test -p athena-core reading`.
4. [behavior] A test drives the reader path with the fake reader returning a bad reading and shows the reading is not accepted and the reason reaches the caller. Run `cargo test -p athena-core`.
5. [behavior] Existing reading tests, including prior-result and scope tests, still pass; `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass.
6. [behavior] The change refuses more readings and the higher refusal rate is intended: expected 5% of newly refused readings.
7. [content] CHANGELOG.md has an entry saying readings are now checked against the boundary words and the repo's files, written for an outside reader.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-context/src/briefs.rs (modify)

## Out of scope

- Re-implementing the disk-space hold, the hold-resume route, or the records-branch reader move from the earlier jobs
- Editing prompt files, which have almost no token headroom
- Semantic checks such as finding every reader of a moved path
- Changing athena.toml, defaults.toml or the workflows
- Changing the boundary word list itself

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and wiring tests; fmt, clippy and test must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced in code in check_reading_acceptance, not in prose.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed at the acceptance gate for all readings, not for one job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that show refusal happening.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt --check.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused check added to one function.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check by reading the diff. The CHANGELOG entry and the doc comment on the new check function both state the root cause. `git diff --stat` shows no prompt file changed. Run: `git diff -- CHANGELOG.md crates/athena-core/src/reading.rs`. Observes: The root-cause sentence appears in both places: readings were checked only for prior-result, evidence, survey and governance rules, never against the boundary words or the repo's files at HEAD. The diff shows the new check called from the body of `check_reading_acceptance`. No file under a prompts directory appears in the diff.
- Criterion 2: New unit test in reading.rs. It builds a reading with the existing `scoped_reading` helper, puts a boundary word in the summary (built from fragments, for example `["oper","ator"].concat()`, so the test file passes the scan), and calls the production `check_reading_acceptance(&reading, request, None, &[], None)`. No double is involved. It asserts the error contains the word and `summary`. It then loops over the title, an assumption, a criterion, an out_of_scope item and a fragment (a machine path fragment built from pieces). It asserts that a clean reading is accepted. Run: `cargo test -p athena-core reading`. Observes: The refusal text names the word and the field. The clean reading passes.
  Fails without the behavior: `check_reading_acceptance` returns Ok for the dirty reading, so the test's `unwrap_err()` panics and the test fails.
- Criterion 3: New unit test in reading.rs. It calls the production `check_reading_acceptance` with `Some(&files)`, a `BTreeSet` fixture holding `README.md` and `src/lib.rs`. The `scoped_reading` helper makes deliverables Modify, so the test sets `deliverables[i].action` directly. Cases: modify of `docs/missing.md` is refused, and the error names the path. Delete of `docs/missing.md` is refused. Create of `README.md` is refused. Modify of `README.md` plus create of `docs/new.md` with clean text is accepted. Modify of `src` (a directory with files) is accepted. An absent `CHANGELOG.md` modify is accepted. `None` for the file set skips the path rule. The fixture is a plain set passed in, so the rule under test is not inside the double. Run: `cargo test -p athena-core reading`. Observes: The error names each offending path and its action. The valid reading is accepted.
  Fails without the behavior: Without the path rule, absent modify and delete paths and an existing create path are all accepted, so the `unwrap_err()` calls panic.
- Criterion 4: New tests in run_pin_tests.rs. The entry point is `Athena::ask`, which runs `read_for`, `read_once`, `build_reading`, then `check_reading_acceptance` with the file set from `ls_tree` on the fixture repo's commit. The seam is the `Tweak` closure given to the existing `athena_with(dir, state, pinner, seen, tweak)`. It edits the fake `FakeTurn` structured reader output, and it exists today (see `reader_refused_then` and `reader_writes`). The repo fixture has `README.md`, `athena.toml` and `defaults.toml` at HEAD. Test A: the tweak puts a boundary word (built from fragments) into the summary on every read. It asserts `reading` is None, `reader_error` contains the word, the reader ran twice (the second brief has `REFUSAL_HEADING` and the word), and the committed job has an invocation record and no reading. Test B: the tweak sets a deliverable to `docs/missing.md` modify on the first read only. The second read is clean and accepted, and the second brief contains the path. Test C: both reads name the absent path, and the error reaches `reader_error`. The refusal logic stays in production code; the tweak only writes the reader's output. Run: `cargo test -p athena-core`. Observes: The reading is not accepted, the refusal text names the word or path and reaches the caller as `reader_error`, and a clean second reading is accepted.
  Fails without the behavior: Without the check, the fake reader's dirty reading is accepted. `proposal.reading` is Some and `reader_error` is None, so the asserts on `reading.is_none()` and on the error naming the word fail.
- Criterion 5: Run the three Athena checks as separate whole commands. The existing prior-result, scope, evidence, survey and governance tests in reading.rs keep passing through the extra `None` argument. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three pass. Prior-result and scope tests are unchanged except for the added `None` argument.
  Fails without the behavior: If the new rule broke an existing fixture (for example a CLI test whose fake reader names a path absent at HEAD), `cargo test --locked` would fail. The producer fixes those fixtures in this change.
- Criterion 6: The refusal tests of criteria 2 and 3 show the higher refusal rate happening. `git diff -- CHANGELOG.md` shows the entry says the refusal rate rises, that this is intended, and gives the estimate of 5% of newly refused readings, labelled as an estimate and not measured. Run: `cargo test -p athena-core reading`. Observes: The tests refuse readings the old gate accepted. The changelog text states the higher refusal rate is intended and quotes the 5% estimate.
  Fails without the behavior: Without the check, readings with boundary words or absent paths are accepted, so the refusal tests of criteria 2 and 3 fail.
- Criterion 7: Content check by reading the diff. The new `[Unreleased]` / `### Added` entry follows the style of the neighbouring entries, in plain words for an outside reader. Run: `git diff -- CHANGELOG.md`. Observes: The entry says readings are now refused if their text has a boundary word, or if a deliverable's action disagrees with the files at HEAD (modify or delete of an absent path, create of an existing one). It says where the check runs, and that it adds no model call. It has no boundary word and no email-like text.

Planned files:

- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-agent/src/fake.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 5% of newly refused readings (the job is held above the declared share plus 5 points).

## The change

Base 21ba791e0838 → candidate 6c8a4f638ffd. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  16 ++
 crates/athena-agent/src/fake.rs         |  31 ++-
 crates/athena-cli/tests/drain.rs        |  12 +
 crates/athena-cli/tests/steward.rs      |   3 +-
 crates/athena-core/src/reading.rs       | 428 +++++++++++++++++++++++++++++---
 crates/athena-core/src/run_pin_tests.rs | 122 +++++++++
 6 files changed, 578 insertions(+), 34 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1600 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Existing reading tests, including prior-result and scope tests, still pass; `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
