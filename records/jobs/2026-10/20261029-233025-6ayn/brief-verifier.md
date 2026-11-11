# Job 20261029-233025-6ayn: verify "Refuse records-measure readings lacking a real-records criterion"

## Request (verbatim)

> Priority: high
> Measures and reports that Athena computes from its own records (throughput, keep rule, planning cost, refusal rates, check durations) have twice in two days been verified on fixture records alone and then shown wrong figures on the real records (2026-10-29: landing time 0.1 minutes, then 273.8 minutes, both against real landings of a few minutes). Require that any change to a measure computed from records includes a criterion run against this repository's real records with an expected range taken from an independent source (git log, recorded check durations), and give the verifier that real output.
> Acceptance:
> - The reader brief and the planner brief state the rule in one line each and stay within their size limits; the size test passes.
> - A test shows a reading for a change to a records-computed measure with no real-records criterion is refused at reading time naming the missing criterion.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/reading.rs, crates/athena-context/src/briefs.rs, CHANGELOG.md

## Reading (accepted by the user)

Add a reading-time check that refuses a code change to a measure computed from Athena's records unless a criterion runs it against this repository's real records with an expected range from an independent source, plus one-line rules in the reader and planner briefs. Audience: Athena maintainers and the verifier, who need the real output.

Assumptions:
- Code survey: read crates/athena-core/src/reading.rs (check_reading chain with step! calls, build_reading, size test helpers), crates/athena-core/src/reading/surveys.rs (check_history_fixture is the closest existing check: it fires on code readings with a behavior criterion and names the missing phrase in its error), and crates/athena-context/src/briefs.rs (reader and planner rule lines and the size test). The new check is called from the same chain as check_history_fixture; the property (a records-computed measure change lacking a real-records criterion) is measured directly from reading text: summary, criteria and deliverable paths. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are unaffected.
- Existing code: crates/athena-core/src/reading/surveys.rs check_history_fixture and changes_code_with_behavior_criterion are extended or mirrored; crates/athena-context/src/briefs.rs reader and planner rule lists get one line each.
- Environment: the check runs inside Athena's reader step on the producer's working copy snapshot, with read-only access to the reading text and no git, network or records access; it needs no origin, so with no origin, a non-GitHub origin or a GitHub origin it behaves identically. The tests run under cargo test with no records branch. The rule it enforces asks later jobs' criteria to run against this repository's real records, which the verifier sees as recorded check output; the producer working copy does not contain records/. No data or doc file is read by the new check.
- Because the request asks for the rule: the check refuses more readings and the higher refusal rate is intended: expected 3% of newly refused readings (estimate, not measured; few past jobs changed records-computed measures). The replay at verification measures the real share.
- The rule is detected by keywords for records-computed measures (throughput, keep rule, planning cost, refusal rate, check duration, landing time) in a code or tests reading's summary or criteria, and satisfied by a criterion naming real records and an independent source (git log or recorded check durations) with an expected range.
- Brief growth is tight: reader has 2 tokens left and planner 95, so the reader line must replace or shorten existing wording to stay within the size limit.

## Acceptance criteria

1. [content] crates/athena-context/src/briefs.rs has exactly one added rule line in the reader brief and one in the planner brief stating that a change to a records-computed measure needs a criterion run against real records with an expected range from an independent source; Grep the diff shows one line each.
2. [behavior] Run `cargo test -p athena-context`: the brief size test passes with both new lines, so the reader and planner briefs stay within their token budgets.
3. [behavior] Run `cargo test -p athena-core`: a new test builds a code reading whose summary changes a records-computed measure (for example landing time) with only fixture-based criteria and asserts check_reading returns an error naming the missing real-records criterion.
4. [behavior] Run `cargo test -p athena-core`: a companion test shows the same reading accepted once it adds a criterion naming real records, an independent source (git log or recorded check durations) and an expected range, and a reading not touching a records measure is unaffected.
5. [content] The new check's doc comment in crates/athena-core/src/reading/surveys.rs or reading.rs states that it refuses more readings, the higher rate is intended, and declares `expected 3% of newly refused readings`.
6. [content] CHANGELOG.md has a new entry describing the real-records criterion rule, citing the 2026-10-29 landing-time figures 0.1 and 273.8 minutes.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing athena.toml, defaults.toml or workflows
- Making the verifier itself fetch real records or changing verifier prompts
- Fixing the landing-time measure itself
- Changing the reader, planner token budgets in config

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests plus cargo test, clippy -D warnings and fmt.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Refusal and acceptance are checked by running tests, not by file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused check plus two brief lines and a changelog entry.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 4)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: The test is the evidence. READING_RULES.len() goes from 11 to 12 and exactly one line in the reader section names a records-computed measure, real records, an expected range and an independent source (count of matching lines asserted == 1). A new planner test asserts REAL_RECORDS_RULE appears exactly once in the Report. The diff of the two sub-module files supplements it; reader.rs also shows '+' lines for shortened entries, so the count rests on the assertions. Run: `cargo test -p athena-context the_reader_brief_names_each_reading_rule_in_one_short_line && git diff -U0 -- crates/athena-context/src/briefs/reader.rs crates/athena-context/src/briefs/planner.rs`. Observes: Both tests pass; the diff shows one new READING_RULES entry naming check_records_measure and one new REAL_RECORDS_RULE const plus its interpolation in the Report string. The rule text lives in briefs/reader.rs and briefs/planner.rs, not briefs.rs (briefs.rs:17-18).
  Fails without the behavior: With either line missing, the exactly-one assertions fail (zero reader lines match, zero Report occurrences).
  Test seam: Production entry points `reader_brief(&reader_inputs(&reading()))` and `planner_brief(&planner_inputs(None))` in the briefs.rs test module; both builders exist. The reader test exists (briefs.rs:3495-3517) and is updated; the planner test is added by plan steps 5-6. No double.
- Criterion 2: Run all athena-context tests, including the updated reader rule test (count 12, each line at most 220 characters, `section.len() < 3150` unchanged, baseline and new size in the comment, no-growth assertion) and the new planner test (rule at most 200 bytes, `report.len() <= REPORT_BASE + REAL_RECORDS_RULE.len() + 4`). Run: `cargo test -p athena-context`. Observes: All pass, so the reader and planner briefs stay within their limits with both lines present.
  Fails without the behavior: If the reader line is added without shortening others, the < 3150 or no-growth assertion fails. If the planner const is not interpolated, the once-in-Report assertion fails. Over 200 bytes fails the bound.
  Test seam: Production entry points `reader_brief` and `planner_brief` via `reader_inputs` and `planner_inputs` in the briefs.rs test module; they exist. Steps 5-6 add the assertions and the planner test. No double.
- Criterion 3: In the reading.rs test module build `code_reading(&[SURVEY], &[MODIFY_READING])` with the summary 'Fix the landing time shown by status' and a fixture-only Behavior criterion. Call `gate_refuser` and `accept(..).unwrap_err()`. Variants, each refused: the three parts in a Content criterion only; real records with no range; real records and a range with no independent source; summary written `landing-time`; each word (refusal counts, wasted work, `time spent per job`, `time_spent`, throughput review); a deliverable `crates/athena-core/src/status/throughput.rs` with no measure word; a status/planning.rs deliverable; a split refactor of status/throughput.rs; a reading saying the throughput section is unchanged; and `check_reading_text` on the same reading returning `ReadingCheck::Refuse` (the drain, plan and steward seam). Run: `cargo test -p athena-core records_measure_without_real_records_criterion_is_refused`. Observes: `gate_refuser` returns Some("check_records_measure"); the error names the trigger (phrase or path), `real records`, the independent sources, `expected range` and the word criterion.
  Fails without the behavior: Without the check or its `step!` line, `gate_refuser` is None and `unwrap_err()` panics.
  Test seam: Production entry points `refusing_check` (checks.rs:518), `check_reading_acceptance` (checks.rs:490) and `check_reading_text` (replay.rs:54). `check_records_measure` is added by plan step 2 and its `step!` line by step 3. Fixture: a `Reading` built by the existing helpers `code_reading`, `add_behavior`, `accept`, `gate_refuser` in the reading.rs test module (they exist; check their signatures when writing). The check is a pure function of the Reading; no double.
- Criterion 4: Companion tests in the reading.rs test module. (1) The same landing-time reading plus a Behavior criterion naming real records, `history.json` commit times and an expected range: Ok and `gate_refuser(..) == None` with NO `Environment:` assumption. (2) A `git log` variant with an `Environment:` assumption: None. (3) The status/throughput.rs path reading with the satisfying criterion: accepted. (4) Unaffected and accepted: the default reading ('Change the reading rules.'), a docs reading naming landing time, a question, a decline, a content-only reading, a code reading on CI or drain throughput, a CI-wait reading ('reduce time spent waiting for CI'), a code reading with `crates/athena-data/src/records/throughput.rs` as deliverable and no measure word, a wrapped 'higher refusal rate is intended' sentence, and a criterion with 'expected 3% of newly refused readings'. (5) A step-line test pins one `step!(check_records_measure(reading));` line directly after `step!(check_history_fixture(reading));`, mirroring `history_fixture_has_one_step_line` (reading.rs:8152 area). Run: `cargo test -p athena-core records_measure`. Observes: Satisfied readings are accepted by the whole chain; unaffected readings are accepted; the step line is present once, in position.
  Fails without the behavior: If the check ignored the satisfying criterion, the accepted cases would be refused. If it ignored the carve-outs, the docs, question, decline, CI, records/throughput.rs and declaration cases would be refused.
  Test seam: Same as criterion 3; the assumption is a string pushed to `reading.assumptions` in the test. The step-line test reads the checks.rs source as the existing test does. No double.
- Criterion 5: A test modelled on `history_fixture_refusal_is_declared_intended` (refusal_corpus.rs:2919) reads the doc comment above `pub fn check_records_measure(` from the surveys source (via `reading_source()`, line 2673) and checks it, and calls `replay_declaration` on a criterion with the declaration. Run: `cargo test -p athena-core --test refusal_corpus records_measure_refusal_is_declared_intended`. Observes: The doc comment holds 'This check refuses more readings; the higher refusal rate is intended.', 'expected 3% of newly refused readings', 'estimate', the 2026-10-29 figures 0.1 and 273.8 minutes, and a Limits paragraph (lexical; unchanged-measure and split readings still refused; status.rs refusal counts caught by words only). `replay_declaration` returns share Some(3).
  Fails without the behavior: Missing declaration or limits make the needle assertions fail.
  Test seam: Production entry point `athena_core::reading::replay_declaration` and the source reader in refusal_corpus.rs (exists); the test is added by plan step 8. No double.
- Criterion 6: The same test collapses the whitespace of CHANGELOG.md (include_str!, as the existing declared-intended tests do) and checks the new item; the diff shows it under [Unreleased] / Changed. Run: `cargo test -p athena-core --test refusal_corpus records_measure_refusal_is_declared_intended && git diff -U0 -- CHANGELOG.md`. Observes: The item names `reading::check_records_measure`, states the rule, cites landing time 0.1 and 273.8 minutes on 2026-10-29, gives the history.json commit-time example with `status --snapshot` as the output compared against it, says splits of the figure modules also need the criterion, and contains 'higher refusal rate is intended' and 'expected 3% of newly refused readings'.
  Fails without the behavior: Without the item the changelog assertions fail.
  Test seam: CHANGELOG.md read with include_str! in refusal_corpus.rs; exists. No double.
- Criterion 7: Run the three Athena checks, then the corpus alone. New corpus cases: records-measure-change-on-fixtures-only-job-landing-time (Refuse), refusal-counts-status-from-5724053 (Refuse; summary 'Report per-rule reading refusal counts in status', deliverables as in HISTORY.txt lines 109-113), measure-module-path-no-measure-word (Refuse; deliverable crates/athena-core/src/status/throughput.rs), measure-module-split-refactor (Refuse), records-measure-named-as-unchanged (Refuse), records-measure-change-with-real-records-range-history-json (Accept, no assumption), ...-git-log (Accept, Environment assumption), refusal-rate-declaration-wrapped-line (Accept), docs-reading-naming-landing-time (Accept), ci-drain-throughput-reading (Accept), ci-wait-time-spent-reading (Accept), records-throughput-record-types-no-measure-word (Accept; deliverable crates/athena-data/src/records/throughput.rs). I did not run any of this and cite no result. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked && cargo test -p athena-core --test refusal_corpus`. Observes: All pass. `every_acceptance_check_has_a_refusing_corpus_case` finds check_records_measure refusing in the first case; `corpus_covers_the_listed_input_forms` passes with the new tags; `the_wrapper_agrees_with_refusing_check_on_every_case` passes. Existing cases keep their outcome: the measure phrases appear in refusal_corpus.rs only in declared-intended sentences, which are skipped.
  Fails without the behavior: Without the new corpus case `every_acceptance_check_has_a_refusing_corpus_case` fails; any broken existing reading test fails cargo test.
  Test seam: Production entry points `refusing_check` and `check_reading_acceptance` called by the corpus harness `run(c)` with new `Case` values built by the existing `case(...)` helper, `repo_files` set as the history cases do. Plan step 7 adds them. No double.

Planned files:

- crates/athena-core/src/reading/surveys.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/reading/checks.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs/reader.rs (modify)
- crates/athena-context/src/briefs/planner.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 2 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 3% of newly refused readings (the job is held above the declared share plus 5 points).

- 20261110-040834-t8ge: the reading was refused because it changes a measure computed from Athena's records (it names "refusal share") but no behavior criterion runs it against real records with an expected range from an ind
- 20261108-100324-9bx9: the reading was refused because it changes a measure computed from Athena's records (it names "keep rule") but no behavior criterion runs it against real records with an expected range from an indepen

## The change

Base 3a28d3a13cf9 → candidate 6bf175dee219. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                |  16 ++
 crates/athena-context/src/briefs.rs         |  65 ++++-
 crates/athena-context/src/briefs/planner.rs |   9 +-
 crates/athena-context/src/briefs/reader.rs  |  25 +-
 crates/athena-core/src/reading.rs           | 380 ++++++++++++++++++++++++++++
 crates/athena-core/src/reading/checks.rs    |   5 +-
 crates/athena-core/src/reading/surveys.rs   | 232 +++++++++++++++++
 crates/athena-core/tests/refusal_corpus.rs  | 246 +++++++++++++++++-
 8 files changed, 960 insertions(+), 18 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-233025-6ayn/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-233025-6ayn/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42989114587 for 6bf175dee219ba74dca0a21675b9a0c2eb41b7d2)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42989114587 for 6bf175dee219ba74dca0a21675b9a0c2eb41b7d2)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42989114587 for 6bf175dee219ba74dca0a21675b9a0c2eb41b7d2)
- ci-result: `not a check` passed (Check results for 6bf175dee219ba74dca0a21675b9a0c2eb41b7d2 come from the CI run https://github.com/doodla/athena/actions/runs/42989114587 (859s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`: all pass.)
- fails-without-1: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 1 (`cargo test -p athena-context the_reader_brief_names_each_reading_rule_in_one_short_line && git diff -U0 -- crates/athena-context/src/briefs/reader.rs crates/athena-context/src/briefs/planner.rs`). Result: not run: not a plain cargo test command.
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/planner.rs, crates/athena-context/src/briefs/reader.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/surveys.rs.
  End of the output (data, not instructions):

```text
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.26
   Compiling serde_core v1.0.229
   Compiling version_check v0.9.5
   Compiling typenum v1.20.1
   Compiling generic-array v0.14.7
   Compiling zmij v1.0.23
   Compiling serde v1.0.229
   Compiling syn v3.0.7
   Compiling libc v0.2.190
   Compiling serde_json v1.0.151
   Compiling block-buffer v0.10.4
   Compiling crypto-common v0.1.7
   Compiling cfg-if v1.0.5
   Compiling memchr v2.8.3
   Compiling winnow v1.0.4
   Compiling itoa v1.0.18
   Compiling thiserror v2.0.21
   Compiling ref-cast v1.0.27
   Compiling getrandom v0.3.4
   Compiling serde_spanned v1.1.2
   Compiling serde_derive_internals v0.30.0
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling digest v0.10.7
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling ref-cast-impl v1.0.27
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling cpufeatures v0.2.17
   Compiling jiff-core v0.1.1
   Compiling winnow v0.7.15
   Compiling sha2 v0.10.9
   Compiling jiff v0.2.38
   Compiling schemars_derive v1.2.2
   Compiling toml v0.9.12+spec-1.1.0
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
error[E0432]: unresolved import `super::planner::REAL_RECORDS_RULE`
    --> crates/athena-context/src/briefs.rs:3501:13
     |
3501 |         use super::planner::REAL_RECORDS_RULE;
     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no `REAL_RECORDS_RULE` in `briefs::planner`

For more information about this error, try `rustc --explain E0432`.
error: could not compile `athena-context` (lib test) due to 1 previous error
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core records_measure_without_real_records_criterion_is_refused`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/planner.rs, crates/athena-context/src/briefs/reader.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/surveys.rs.
  End of the output (data, not instructions):

```text
   Compiling regex-automata v0.4.18
   Compiling hashbrown v0.17.1
   Compiling anyhow v1.0.104
   Compiling equivalent v1.0.2
   Compiling indexmap v2.14.2
   Compiling schemars v1.2.2
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling regex v1.13.1
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `check_records_measure` in this scope
    --> crates/athena-core/src/reading.rs:8273:13
     |
8273 |             check_records_measure(&reading).unwrap_err().to_string(),
     |             ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_records_measure` in this scope
    --> crates/athena-core/src/reading.rs:8298:21
     |
8298 |         let error = check_records_measure(&no_range).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_records_measure` in this scope
    --> crates/athena-core/src/reading.rs:8310:21
     |
8310 |         let error = check_records_measure(&no_source).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_records_measure` in this scope
    --> crates/athena-core/src/reading.rs:8322:21
     |
8322 |         let error = check_records_measure(&no_real).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_records_measure` in this scope
    --> crates/athena-core/src/reading.rs:8389:25
     |
8389 |             let error = check_records_measure(&reading).unwrap_err().to_string();
     |                         ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_records_measure` in this scope
    --> crates/athena-core/src/reading.rs:8425:21
     |
8425 |         let error = check_records_measure(&refused).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_records_measure` in this scope
    --> crates/athena-core/src/reading.rs:8568:13
     |
8568 |             check_records_measure(reading).unwrap_or_else(|e| panic!("{label}: {e}"));
     |             ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 7 previous errors
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core records_measure`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-context/src/briefs/planner.rs, crates/athena-context/src/briefs/reader.rs, crates/athena-core/src/reading/checks.rs, crates/athena-core/src/reading/surveys.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find function `check_records_measure` in this scope
    --> crates/athena-core/src/reading.rs:8273:13
     |
8273 |             check_records_measure(&reading).unwrap_err().to_string(),
     |             ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_records_measure` in this scope
    --> crates/athena-core/src/reading.rs:8298:21
     |
8298 |         let error = check_records_measure(&no_range).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_records_measure` in this scope
    --> crates/athena-core/src/reading.rs:8310:21
     |
8310 |         let error = check_records_measure(&no_source).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_records_measure` in this scope
    --> crates/athena-core/src/reading.rs:8322:21
     |
8322 |         let error = check_records_measure(&no_real).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_records_measure` in this scope
    --> crates/athena-core/src/reading.rs:8389:25
     |
8389 |             let error = check_records_measure(&reading).unwrap_err().to_string();
     |                         ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_records_measure` in this scope
    --> crates/athena-core/src/reading.rs:8425:21
     |
8425 |         let error = check_records_measure(&refused).unwrap_err().to_string();
     |                     ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `check_records_measure` in this scope
    --> crates/athena-core/src/reading.rs:8568:13
     |
8568 |             check_records_measure(reading).unwrap_or_else(|e| panic!("{label}: {e}"));
     |             ^^^^^^^^^^^^^^^^^^^^^ not found in this scope

For more information about this error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 7 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo test -p athena-core --test refusal_corpus records_measure_refusal_is_declared_intended`). Result: not run: only the first few distinct commands are run.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-core --test refusal_corpus records_measure_refusal_is_declared_intended && git diff -U0 -- CHANGELOG.md`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked && cargo test -p athena-core --test refusal_corpus`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
