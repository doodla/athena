# Job 20261107-224504-k8mh: verify "Add bug-evidence default and enforce root-cause criterion on bug"

## Request (verbatim)

> try again on 20261105-234147-88tw

## Reading (accepted by the user)

Retry of job 20261105-234147-88tw (dispatched, never verified, no findings, no candidate files): add a "bug-evidence" default to defaults.toml for bug, wrong-output and regression requests, and make build_reading add a criterion that the change names the root cause, shows it from readable data and shows why the symptom follows, before any fix. Audience: the verifier who rejects data-less root-cause claims at the final verdict. The whole prior scope is kept.

Assumptions:
- Prior job 20261105-234147-88tw: it recorded no findings and no candidate changes, so nothing is carried over. This retry redoes the same work from the base and keeps every deliverable of that job's scope (defaults.toml, reading.rs, defaults.rs, CHANGELOG.md, athena.toml, briefs.rs, defaults-loop.md). Out-of-scope items are also unchanged.
- Code survey: crates/athena-core/src/reading.rs. build_reading (around line 685) builds the Reading from the structured output, then calls ensure_changelog (line 842) before validate(). ensure_changelog (line 281) is the model: idempotent, pushes a DefaultApplied entry only if its id is absent. The new step goes beside it, runs before validate(), and is idempotent. build_reading does not receive the request text, so the step detects bug-type work from the reading's own title, summary and criteria, or the producer threads the request through. Also read crates/athena-data/src/defaults.rs, where defaults.toml is parsed and rendered, and crates/athena-context/src/briefs.rs, where the defaults text reaches the reader prompt.
- Code survey: no new record or data file is created under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected. Root-cause evidence is measured directly as a criterion in Reading.acceptance_criteria plus a defaults_applied entry with id bug-evidence, not through a stand-in.
- The reader prompt is at 1997 of 2000 tokens, and the new default renders into it, so the reader budget in athena.toml likely needs a raise. Only that budget value changes, and only if the budget check fails. Verifier prompt sets have 2 to 4 tokens of headroom and must not grow.
- The new step adds a criterion and refuses nothing, so it creates no new refusal rule and no replay refusal rate applies.
- Existing code: crates/athena-core/src/reading.rs (ensure_changelog pattern), crates/athena-data/src/defaults.rs, docs/design/defaults-loop.md. The new step extends the ensure_changelog pattern.

## Acceptance criteria

1. [content] defaults.toml has a new [[default]] with id bug-evidence for bug, wrong-output and regression requests. Its statement requires that the change names the root cause, shows it from data the job can read (records, code path, a failing case), and shows why the wrong output or metric change follows, before any fix. It says a guess labelled as an estimate does not meet it. Its source cites 20261104-054700-e6js, 20261105-051344-q9fv, 20261105-101121-qw9r and 20261105-205835-hk09. The existing root-cause default is not reworded.
2. [content] crates/athena-core/src/reading.rs has a step called next to ensure_changelog in build_reading that adds the root-cause-evidence criterion and a bug-evidence defaults_applied entry exactly once for bug, wrong-output, regression and slowdown readings, and nothing for other readings. A Grep for bug-evidence in that file shows it.
3. [behavior] Unit tests in crates/athena-core/src/reading.rs build a reading from a bug request and assert the criterion and defaults entry are present. They assert a non-bug reading gets neither, and that running the step twice adds them once. Run `cargo test -p athena-core`.
4. [behavior] A test in crates/athena-data/src/defaults.rs asserts bug-evidence parses from the repo defaults.toml and its first-sentence line renders. Run `cargo test -p athena-data`.
5. [behavior] `cargo fmt --check`, `cargo clippy --workspace` and `cargo test --workspace` pass. Existing tests keep their outcomes and the prompt token budget tests pass.
6. [content] Governance: defaults.toml is changed and named here. athena.toml changes only if the reader prompt budget check requires it, and then only that budget value. CHANGELOG.md has an entry that names the cause: bug jobs were rejected at the final verdict because no criterion demanded data-backed root-cause evidence. docs/design/defaults-loop.md mentions the new default.
7. [content] Standing defaults hold. outcome-check: a newcomer reading the new default and the criterion text can tell what evidence is required. root-cause: the change description states why bug jobs were rejected. tested: new tests pass. no-empty-success and thrash-stop apply unchanged. records/ is untouched.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-data/src/defaults.rs (modify)
- CHANGELOG.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-context/src/briefs.rs (modify)
- docs/design/defaults-loop.md (modify)

## Out of scope

- Changing records/ or any owner file, including records/failure-classes/2026-11/20261104-054711-kb90.json
- Changing the verifier's final-verdict rejection logic
- Rewording the existing root-cause default
- Fixing the individual bug and slowdown jobs that were rejected

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion requires that the new default and criterion text make the required evidence clear to a newcomer.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The change description names why bug jobs were rejected: no criterion asked for data-backed root-cause evidence.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Reading and defaults tests are added; fmt, clippy and test are required to pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a reading-pipeline step, not prose alone.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria run cargo test to see the criterion added.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change that extends the ensure_changelog pattern.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The default text and docs are written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 4)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Parse the repo defaults.toml in an athena-data unit test and check the new entry's id, applies_to, key phrases and source. The same test asserts the `root-cause` statement equals its base text. Run: `cargo test -p athena-data bug_evidence`. Observes: `bug_evidence_default_parses_and_renders_its_first_sentence` passes. It asserts: applies_to is ["*"]; the first sentence contains 'Only for bug, wrong-output, regression or slowdown work'; the statement contains 'before any fix', 'root cause', 'data the job can read' and 'a guess labelled as an estimate does not meet this'; the source contains 20261104-054700-e6js, 20261105-051344-q9fv, 20261105-101121-qw9r and 20261105-205835-hk09; the `root-cause` statement is byte-identical to the base.
  Fails without the behavior: Without the defaults.toml entry `get("bug-evidence")` is None and the test fails; a reworded `root-cause` fails the equality assertion; a first sentence without the scope words fails.
  Test seam: Production `DefaultsModel::parse` and `get` (crates/athena-data/src/defaults.rs:36, 51) on the real file through the existing `REPO_DEFAULTS` const (defaults.rs:89). Exists; no double. The test is added by step 2.
- Criterion 2: Grep reading.rs for `bug-evidence`, and run a source-text test (the style of the existing include_str! tests in reading.rs) that takes the body of `build_reading` and asserts the call order. Run: `cargo test -p athena-core bug_evidence_step_is_called_beside_ensure_changelog`. Observes: Grep for `bug-evidence` in crates/athena-core/src/reading.rs shows the id constant and the entry. The test asserts that `build_reading`'s body contains `ensure_changelog(&mut reading)`, then `ensure_bug_evidence(&mut reading, request_texts)`, then `reading.validate()`, in that order, and that `ensure_bug_evidence`'s source returns early unless capability is `change` and when criteria are empty. A second test in this run, `bug_evidence_read_once_passes_the_earlier_request`, asserts that `read_once`'s source passes `attempt` request text into `build_reading`.
  Fails without the behavior: Removing the call from `build_reading`, or moving it after `validate()`, makes the order test fail. Dropping the earlier-request argument at the `build_reading` call fails the second test.
  Test seam: Reads the production source of reading.rs with `include_str!` and splits at `fn build_reading(` and `fn read_once(`, as existing source-text tests do. Exists in style; the new tests are added by step 7.
- Criterion 3: Unit tests drive the real `Athena::build_reading` through a new helper with bug requests, and call the step directly over positive and negative tables. They cover both tiers, the negated tier-1 entry, the standing-default criterion, retry by earlier request, the empty-criteria guard, idempotency, and that the added text trips no `refusing_check` step. Run: `cargo test -p athena-core bug_evidence`. Observes: All `bug_evidence_*` tests pass. (a) A change reading built from request 'Fix the bug where the report shows the wrong output' has exactly one content criterion starting 'Before any fix, the change states the root cause' and exactly one `bug-evidence` entry; `refusing_check` on it is None. (b) Tier 2 positives: 'Fix the regression in the parser', 'Investigate the slowdown in queue drain', 'Fix wrong-output in the status line', 'Fix the regression and add a regression test', 'Bugfix for the parser crash'. Negatives, which get neither criterion nor entry: 'Add a regression test for the parser', 'Add bug-evidence default and enforce root-cause criterion on bug', 'Throughput review: flag median slowdowns and queue a request', 'Fix the docs for the bug report form', 'Fix a typo in the regression section of the README', 'Describe the demo in README.md', 'Add debug logging', a study-capability bug request, a declined reading. (c) Tier 1: a change reading with a keyword-free request whose defaults_applied lists `bug-evidence` gains the criterion and no second entry. A tier-1 reading whose only root-cause criterion is 'root-cause: the change description names the root cause' still gains the step's criterion. Entries whose `how` is 'not applicable: this is not a bug job', 'does not fit: feature work', or 'n/a' on a feature request gain no criterion; a `how` containing 'in/a' as a substring of a word does not count as negated. (d) Retry: the request texts are ['try again on 20261105-234147-88tw', 'Fix the regression in the parser'] with a neutral title; the reading gains the criterion. The same with an earlier request 'Add a footer' and a neutral title gains nothing. (e) A bug request whose reading has an empty criteria list is still refused by `build_reading` with 'needs at least one acceptance criterion'. (f) Calling the step twice leaves the reading equal.
  Fails without the behavior: With no step, (a) finds zero root-cause criteria and zero entries. With bare `has_phrase` matching, the negative table fails on 'bug-evidence', 'regression test' and 'slowdowns'. With a fix word allowed anywhere, the docs and typo cases fail. If the reader-written root-cause criterion suppressed the step, the standing-default case in (c) finds no criterion. Without the whole-phrase negation check the negated cases gain a criterion. Without the earlier-request input, (d) finds nothing. Without the empty-criteria early return, (e) is accepted instead of refused. With a non-idempotent step, (f) finds duplicates. Without the capability gate, the study case gains the criterion.
  Test seam: Production entry point `Athena::build_reading` (reading.rs:685) through a new test helper `build_for_request(reading_json, request_texts, capabilities)` added in step 7. It copies `build_sized` (reading.rs:5606-5640): `crate::test_support::make_athena` on tempdirs, the shipped athena.toml and defaults.toml, `Redactor::new(&athena_gate::builtin_config())`, capabilities ["change","study"]. The reading JSON comes from the existing `sized_reading` (reading.rs:5580) with capability set to change; for (e) the criteria array is emptied. The step is also called directly on the existing `change_reading()` (reading.rs:6130). The real `build_reading`, `names_bug_work` and `refusing_check` run; no double holds the logic under test.
- Criterion 4: An athena-data unit test parses the repo defaults.toml and checks the entry and its rendered first-sentence line. Run: `cargo test -p athena-data bug_evidence`. Observes: `DefaultsModel::parse` succeeds on the real file. `render()` contains the exact line `- bug-evidence [*]: <first sentence>\n` and does not contain the statement's second sentence.
  Fails without the behavior: Without the entry the line is absent. If the first sentence had no '. ' break, the render would carry the whole statement and the second-sentence assertion fails.
  Test seam: Production `DefaultsModel::parse`, `get` and `render` (defaults.rs:36, 51, 58) on the real file through `REPO_DEFAULTS` (defaults.rs:89). Exists; no double.
- Criterion 5: Run the three Athena checks, the budget tests that compose the reader set from the real defaults.toml, the briefs and pin tests that quote the reader figure, and the refusal corpus. Existing tests keep their outcomes. Run: `cargo test --locked`. Observes: `cargo fmt --all --check` and `cargo clippy --all-targets --locked -- -D warnings` pass. `cargo test --locked` passes, including `cargo test -p athena-cli --test budgets` (`every_role_set_is_within_its_budget`, `system_facts_budgets_match_computed`), the run_pin_tests.rs:1624-1644 test now reading `BudgetSet::Reader.budget()`, the decision-entry structure tests, and `cargo test -p athena-core --test refusal_corpus` (no case added: the step refuses nothing). Also run `cargo test -p athena-context`.
  Fails without the behavior: If the reader budget were not raised, `check_budget(BudgetSet::Reader, ..)` fails in `every_role_set_is_within_its_budget`. If system-facts.md kept 1997 of 2000, `system_facts_budgets_match_computed` fails. If run_pin_tests.rs kept the literal 2000, its brief-text assertion fails. If D25 were edited instead of appending, the append-only check fails.
  Test seam: Existing tests: `computed_rows` and `check_system_facts` in crates/athena-cli/tests/budgets.rs build the sets from the real files; the `refusal_corpus` target drives `refusing_check`. No double. Nothing is added except the run_pin_tests.rs edit.
- Criterion 6: A test reads CHANGELOG.md, docs/decisions.md and docs/design/defaults-loop.md; the diff paths are inspected with git. Run: `cargo test -p athena-core bug_evidence_changelog_and_design_doc_name_the_cause`. Observes: The test passes. The CHANGELOG.md Unreleased item contains `bug-evidence`, states the cause (bug jobs were rejected at the final verdict because no criterion demanded data-backed root-cause evidence), and names the reader budget raise and each file outside the reading's list (lib.rs, run_pin_tests.rs, system-facts.md, decisions.md, context-engineer.md). The decisions.md tail holds an entry that names `bug-evidence`, D25, `crates/athena-context/src/lib.rs` and says the reader stays above 95%. defaults-loop.md has a section naming `bug-evidence`. `git diff` shows defaults.toml changed, no athena.toml and no records/ path.
  Fails without the behavior: Without the CHANGELOG item, the decision entry or the doc section the test fails.
  Test seam: Reads the three files with `include_str!` relative to crates/athena-core/src/reading.rs, as existing doc-pin tests do. Added by step 8. No double.
- Criterion 7: A test checks that the `bug-evidence` statement and the criterion constant state the same evidence to a newcomer. The CHANGELOG test shows the cause. The diff shows records/ untouched. Run: `cargo test -p athena-core bug_evidence_criterion_and_default_state_the_same_evidence`. Observes: The criterion text and the repo default's statement both contain 'root cause', 'data the job can read', 'before any fix' and 'a guess labelled as an estimate does not meet this'. `git diff` shows no records/ path. tested is met by the new passing tests; no-empty-success and thrash-stop apply unchanged.
  Fails without the behavior: If the criterion or the default drops the data requirement or the estimate rule, the test fails.
  Test seam: Production constant `ROOT_CAUSE_EVIDENCE_CRITERION` and `DefaultsModel::parse` on `include_str!("../../../defaults.toml")`, the way `build_sized` loads it (reading.rs:5622). Added by step 8; no double.

Planned files:

- defaults.toml (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-data/src/defaults.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/system-facts.md (modify)
- docs/decisions.md (modify)
- docs/design/context-engineer.md (modify)
- CHANGELOG.md (modify)
- docs/design/defaults-loop.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 1db857ec0267 → candidate d76ba14d8e11. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  21 ++
 crates/athena-context/src/briefs.rs     |  31 ++
 crates/athena-context/src/lib.rs        |   2 +-
 crates/athena-core/src/reading.rs       | 561 +++++++++++++++++++++++++++++++-
 crates/athena-core/src/run_pin_tests.rs |   5 +-
 crates/athena-data/src/defaults.rs      |  37 +++
 defaults.toml                           |   6 +
 docs/decisions.md                       |  37 +++
 docs/design/context-engineer.md         |  15 +-
 docs/design/defaults-loop.md            |  43 +++
 docs/system-facts.md                    |   2 +-
 11 files changed, 749 insertions(+), 11 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261107-224504-k8mh/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261107-224504-k8mh/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42425049392 for d76ba14d8e11bd417ee8c262d3f4e5d2ca97ccc0)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42425049392 for d76ba14d8e11bd417ee8c262d3f4e5d2ca97ccc0)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42425049392 for d76ba14d8e11bd417ee8c262d3f4e5d2ca97ccc0)
- ci-result: `not a check` passed (Check results for d76ba14d8e11bd417ee8c262d3f4e5d2ca97ccc0 come from the CI run https://github.com/doodla/athena/actions/runs/42425049392 (608s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --workspace` and `cargo test --workspace` pass. Existing tests keep their outcomes and the prompt token budget tests pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-data bug_evidence`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/lib.rs, crates/athena-core/src/reading.rs, defaults.toml, docs/decisions.md, docs/design/context-engineer.md, docs/design/defaults-loop.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
   Compiling unicode-ident v1.0.26
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling generic-array v0.14.7
   Compiling libc v0.2.190
   Compiling syn v3.0.7
   Compiling zmij v1.0.23
   Compiling thiserror v2.0.21
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling serde v1.0.229
   Compiling getrandom v0.3.4
   Compiling cfg-if v1.0.5
   Compiling serde_json v1.0.151
   Compiling winnow v1.0.4
   Compiling digest v0.10.7
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling serde_spanned v1.1.2
   Compiling cpufeatures v0.2.17
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling itoa v1.0.18
   Compiling jiff-core v0.1.1
   Compiling memchr v2.8.3
   Compiling winnow v0.7.15
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling jiff v0.2.38
   Compiling toml v0.9.12+spec-1.1.0
   Compiling sha2 v0.10.9
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 15.88s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_data-62eb92b0c349858b)

running 1 test
test defaults::tests::bug_evidence_default_parses_and_renders_its_first_sentence ... FAILED

failures:

---- defaults::tests::bug_evidence_default_parses_and_renders_its_first_sentence stdout ----

thread 'defaults::tests::bug_evidence_default_parses_and_renders_its_first_sentence' (24042) panicked at crates/athena-data/src/defaults.rs:150:47:
called `Option::unwrap()` on a `None` value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    defaults::tests::bug_evidence_default_parses_and_renders_its_first_sentence

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 186 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-data --lib`
     Running tests/repo_config.rs (<job>/target/verify/debug/deps/repo_config-bf3c3e8d6e6af283)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-data --lib`
```
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core bug_evidence_step_is_called_beside_ensure_changelog`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/lib.rs, crates/athena-core/src/reading.rs, defaults.toml, docs/decisions.md, docs/design/context-engineer.md, docs/design/defaults-loop.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
     |             ------------- unexpected argument #8 of type `&[&str]`
     |
note: method defined here
    --> crates/athena-core/src/reading.rs:685:8
     |
 685 |     fn build_reading(
     |        ^^^^^^^^^^^^^
help: remove the extra argument
     |
5665 -             capabilities,
5666 -             request_texts,
5665 +             capabilities,
     |

error[E0425]: cannot find function `names_bug_work` in this scope
    --> crates/athena-core/src/reading.rs:6303:21
     |
6303 |             assert!(names_bug_work(text), "{text}");
     |                     ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `names_bug_work` in this scope
    --> crates/athena-core/src/reading.rs:6318:22
     |
6318 |             assert!(!names_bug_work(text), "{text}");
     |                      ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `listing_is_negated` in this scope
    --> crates/athena-core/src/reading.rs:6421:18
     |
6421 |         assert!(!listing_is_negated("the cause is shown in/a log file"));
     |                  ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `ensure_bug_evidence` in this scope
    --> crates/athena-core/src/reading.rs:6465:9
     |
6465 |         ensure_bug_evidence(&mut reading, &["Fix the bug in the parser"]);
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `ensure_bug_evidence` in this scope
    --> crates/athena-core/src/reading.rs:6473:9
     |
6473 |         ensure_bug_evidence(&mut reading, &["Fix the bug in the parser"]);
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `ensure_bug_evidence` in this scope
    --> crates/athena-core/src/reading.rs:6477:9
     |
6477 |         ensure_bug_evidence(&mut reading, &["Fix the bug in the parser"]);
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `ensure_bug_evidence` in this scope
    --> crates/athena-core/src/reading.rs:6483:9
     |
6483 |         ensure_bug_evidence(&mut built, &["Fix the regression in the parser"]);
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0061, E0425.
For more information about an error, try `rustc --explain E0061`.
error: could not compile `athena-core` (lib test) due to 12 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core bug_evidence`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/lib.rs, crates/athena-core/src/reading.rs, defaults.toml, docs/decisions.md, docs/design/context-engineer.md, docs/design/defaults-loop.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
5666 |             request_texts,
     |             ------------- unexpected argument #8 of type `&[&str]`
     |
note: method defined here
    --> crates/athena-core/src/reading.rs:685:8
     |
 685 |     fn build_reading(
     |        ^^^^^^^^^^^^^
help: remove the extra argument
     |
5665 -             capabilities,
5666 -             request_texts,
5665 +             capabilities,
     |

error[E0425]: cannot find function `names_bug_work` in this scope
    --> crates/athena-core/src/reading.rs:6303:21
     |
6303 |             assert!(names_bug_work(text), "{text}");
     |                     ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `names_bug_work` in this scope
    --> crates/athena-core/src/reading.rs:6318:22
     |
6318 |             assert!(!names_bug_work(text), "{text}");
     |                      ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `listing_is_negated` in this scope
    --> crates/athena-core/src/reading.rs:6421:18
     |
6421 |         assert!(!listing_is_negated("the cause is shown in/a log file"));
     |                  ^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `ensure_bug_evidence` in this scope
    --> crates/athena-core/src/reading.rs:6465:9
     |
6465 |         ensure_bug_evidence(&mut reading, &["Fix the bug in the parser"]);
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `ensure_bug_evidence` in this scope
    --> crates/athena-core/src/reading.rs:6473:9
     |
6473 |         ensure_bug_evidence(&mut reading, &["Fix the bug in the parser"]);
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `ensure_bug_evidence` in this scope
    --> crates/athena-core/src/reading.rs:6477:9
     |
6477 |         ensure_bug_evidence(&mut reading, &["Fix the bug in the parser"]);
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `ensure_bug_evidence` in this scope
    --> crates/athena-core/src/reading.rs:6483:9
     |
6483 |         ensure_bug_evidence(&mut built, &["Fix the regression in the parser"]);
     |         ^^^^^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0061, E0425.
For more information about an error, try `rustc --explain E0061`.
error: could not compile `athena-core` (lib test) due to 12 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test --locked`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-core bug_evidence_changelog_and_design_doc_name_the_cause`). Result: not run: only the first few distinct commands are run.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test -p athena-core bug_evidence_criterion_and_default_state_the_same_evidence`). Result: not run: only the first few distinct commands are run.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
