# Job 20261028-233235-jwp0: verify "Report per-rule reading refusal counts in status; flag rules over 30%"

## Request (verbatim)

> Priority: high
> Reading rules have multiplied (boundary words, governance files, previous result, criteria resting on the producer's report, size, repository files, code survey) and each refusal costs a reader call and time; on 2026-10-28 one retry needed three corrections to satisfy two rules in turn. Nothing measures how often each rule refuses a first reading or how many re-reads it causes. Add, per reading rule, the count of first readings it refused and of second readings it still refused over the last 50 readings, to athena status, and have the Hestia sweep flag any rule that refuses more than 30% of first readings, naming the rule and its most common refusal text, so the reader's brief or the rule can be fixed.
> Acceptance:
> - A test with fixture reading records shows status reports per-rule first and second refusal counts.
> - A test shows the sweep flags a rule refusing more than 30% of first readings and not one under it.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/status.rs, crates/athena-core/src/hestia.rs, CHANGELOG.md

## Reading (accepted by the user)

Add to `athena status` a per-reading-rule count of first readings refused and second readings still refused over the last 50 readings. Make the Hestia sweep flag any rule refusing more than 30% of first readings, naming the rule and its most common refusal text. Goal: measure which reading rules cause re-reads, so the reader brief or the rule can be fixed. Audience: the owner and the Hestia sweep.

Assumptions:
- Code survey: I read crates/athena-core/src/status.rs (StatusView, `status` at line 985), crates/athena-core/src/hestia.rs (`default_checks`, `run_sweep`, `repeated_causes`), crates/athena-core/src/reading.rs (rule functions named check_*, `ReadingCheck::Accept` and `Refuse(String)`) and crates/athena-core/src/run_pin_tests.rs (the refused-then-reread test at line 1855). Resolution order: the reader runs, rules run in turn, and the first refusal triggers one re-read whose brief carries the refusal text. A second refusal is the failure, and the reader does not run a third time. Reader invocations are recorded under `<job>/invocations/`. I found no stored refusal text or rule name per reading, so the producer must first find where the first refusal's text can be read back from records and add the smallest record field if there is none. A new record or field under crates/ must be covered by `check_record_file`, `AnyRecord` and the round-trip test. The property is measured directly from per-reading refusal data, not from a stand-in such as invocation counts alone.
- Rule identification: a refusal text is mapped to a rule by the check function name (for example `check_governance_coverage`) or by a stable text prefix. Text that matches no rule is counted under 'other'.
- Window: the last 50 readings means the 50 most recent jobs that have a reading, counted per job. A first refusal counts once per rule that refused it. A second refusal counts the rules that refused the re-read. The sweep rate is first refusals by that rule divided by first readings in the window.
- Fewer than a minimum number of readings in the window, for example 10, does not trigger a flag, so an empty or tiny window is never reported as healthy or as a failure. This is a design choice the producer states in docs or tests.
- Existing deliverables beyond the three named files (reading.rs, run/mod.rs, athena-data records.rs, a docs page) are possible only if the refusal data is not already recorded; they are listed with lower probability.
- The change adds no refusal rule. It only measures refusals, so no 'higher refusal rate is intended' or replay percentage criterion is needed. Governance files (athena.toml, defaults.toml and others) are untouched. Prompts are untouched; the reader prompt has 2 tokens of headroom.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-core status`. A test in crates/athena-core/src/status.rs builds fixture reading records where rule A refuses 3 first readings and 1 second reading, and rule B refuses 1 first reading. It asserts that the status output reports, per rule, the first-refusal count and the second-refusal count, over at most the last 50 readings.
2. [behavior] Run `cargo test -p athena-core status`. A test with fixture records of more than 50 readings asserts that only the most recent 50 are counted.
3. [behavior] Run `cargo test -p athena-core hestia`. A test in crates/athena-core/src/hestia.rs asserts that the sweep flags a rule refusing more than 30% of first readings. The flag names the rule and its most common refusal text. The same test asserts that a rule at or under 30% is not flagged.
4. [behavior] Run `cargo test -p athena-core hestia`. A test asserts that a window with no readings, or fewer than the stated minimum, produces no flag and no claim that the rules are healthy.
5. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass.
6. [content] CHANGELOG.md has a new entry. It names the per-rule first and second refusal counts in `athena status` and the Hestia sweep flag above 30%.
7. [content] The diff in crates/athena-core/src/status.rs shows each reading rule listed with its two counts. The diff in crates/athena-core/src/hestia.rs shows the 30% threshold as a named constant, not a bare number.
8. [content] If the diff adds a record type or field under crates/, it is covered by `check_record_file`, `AnyRecord` parsing and the round-trip test. Run `cargo test -p athena-data` to confirm.

## Expected deliverables

- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status/throughput.rs (modify)

## Out of scope

- Changing, adding or loosening any reading rule or its refusal text
- Editing the reader prompt or brief; the sweep only names which rule or brief to fix
- Automatically filing fix requests for flagged rules
- Editing athena.toml, defaults.toml or workflows
- Changing the retry limit of one re-read

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criteria require new tests plus fmt, clippy -D warnings and cargo test.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests on fixture reading records and check the reported counts and flags.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: An empty or tiny window gives no flag and is not reported as healthy.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The CHANGELOG entry and code docs are written for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The change measures rule refusals by rule rather than counting retries only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Unit tests in status.rs, plus an end-to-end test that commits fixture reader-invocation records and calls the real `Athena::status()`. Run: `cargo test -p athena-core status`. Observes: The fixture has these first readings. Rule A, `check_governance_coverage`, refuses 3 first readings and, in the following re-read, 1 second reading. Rule B, `check_code_survey`, refuses 1 first reading and its re-read is accepted. The rest are accepted first readings. `ReadingRuleStats::compute` returns, in order, A with first_refused 3, second_refused 1 and B with 1, 0. The window totals are right and `text()` prints each rule with both counts. `Athena::status()` over committed fixture invocation records returns the same under `StatusView.reading_rules`, and the field is absent when no reader invocation has a `reading_attempt`.
  Fails without the behavior: Without the field or `compute`, the counts are absent. If second readings were ignored, A would show 0 second refusals instead of 1. If rules were merged, A and B would not be reported separately.
  Test seam: Production entry points: `ReadingRuleStats::compute(&Records)` and `Athena::status()` (status.rs:985, exists). Fixtures: `Records` built from `JobRecords` with `Invocation`s made by `planning_cost_tests::invocation` (status.rs:1735, pub(crate), exists) with `reading_attempt` set. For the end-to-end case, `code_health::tests::set_up()` and `records_worktree_tests::commit_in_tree` (status.rs:1628, exist) inject them into a temporary git repository. Plan steps 1-2 add `reading_attempt` and `compute`. No logic sits in the fixtures, which hold only recorded facts. Step 4 also extends run_pin_tests.rs (the refused-then-read-again test at line 1855, with the scripted reader `reader_refused_then`) to show `read_once` writes the field.
- Criterion 2: A unit test with fixtures of more than 50 first readings, where the oldest are refused by one rule and the newest 50 are accepted. Run: `cargo test -p athena-core status`. Observes: With 55 first readings by `started_at`, the 5 oldest refused by rule X, `first_readings` is 50 and X is absent or 0. When 5 of the newest 50 are also refused, X shows exactly 5. The same fixture ordered by job id rather than by time pins the sort key.
  Fails without the behavior: A window that counts all readings would report X 5 refusals and 55 first readings.
  Test seam: `ReadingRuleStats::compute` over `Records` fixtures (as in criterion 1). `READING_WINDOW` is the named constant the test reads. The seam is added by plan step 2.
- Criterion 3: A hestia test builds a stats fixture of 20 first readings. Rule A refuses 7 (35%) with the same text 4 times and 3 other texts once each, B refuses 6 (exactly 30%), and C refuses 2. It calls `reading_rule_findings`, then runs the check through `run_sweep` and through `Athena::hestia_sweep_with` on committed fixture invocation records. Run: `cargo test -p athena-core hestia`. Observes: Exactly one finding comes out, under check id `reading-rule-refusals`, with subject A's rule name. Its evidence names A, 7 of 20 and 35%, and quotes the 4-times text. B (exactly 30%) and C produce no finding. A test also checks `REFUSAL_FLAG_PERCENT == 30`. In the full path the committed sweep record holds the finding and the check result `examined` 20, count 1.
  Fails without the behavior: If the comparison were `>=`, B would be flagged. If rates were not per rule, A would not be isolated. Without the check wired into `hestia_sweep_with`, the committed sweep would have no finding.
  Test seam: Production entry points: `reading_rule_findings(&ReadingRuleStats)` (pure, added by plan step 5), `ReadingRulesCheck` through `run_sweep` (hestia.rs:148, exists), and `Athena::hestia_sweep_with` (hestia.rs:2678, exists, refactored in step 5). Fixtures: `ReadingRuleStats` literals, and for the full path `athena_in(dir, state)` (hestia.rs:2871, exists, with `FakeAgentDriver`) plus `commit_in_tree` and `planning_cost_tests::invocation`. The test holds only recorded facts, and the 30% rule lives in the production function.
- Criterion 4: Hestia tests with zero first readings, 9 first readings all refused, and 10 first readings all refused. Run: `cargo test -p athena-core hestia`. Observes: With 0 or 9 first readings there are no findings and `examined` is 0. The evidence and any output contain no 'healthy', 'ok' or 'pass' claim. With 10 all-refused readings the rule is flagged, which pins the minimum as inclusive. `MIN_FIRST_READINGS` is a named constant. The full-path test shows the committed sweep for the backlog fixture of the existing test (hestia.rs:4693) has the check with count 0 and examined 0.
  Fails without the behavior: Without the minimum, 9 all-refused readings would flag. Reporting `examined` as 9 with 0 findings would let a reader take a tiny window as healthy.
  Test seam: `reading_rule_findings` and `ReadingRulesCheck::run` (added by step 5), with `ReadingRuleStats` literals; the full path uses `hestia_sweep_with` as in criterion 3.
- Criterion 5: Run the three project checks on the whole workspace, plus the refusal corpus as a regression check. The corpus is not changed, and it pins the rule names the new field records. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three pass. Existing tests touched by the new struct field compile and pass: hestia.rs:2936 and :4693 (3 ui lines), the run_pin_tests.rs tests at 1855 and 1915, and the refusal corpus. The corpus is also run on its own with `cargo test -p athena-core --test refusal_corpus`. I read these ids there: `corpus_cases_get_the_expected_outcome`, `the_wrapper_agrees_with_refusing_check_on_every_case`, `every_acceptance_check_has_a_refusing_corpus_case` and `no_check_bypasses_refusing_check`. I have not run it. It must still pass because `check_reading_acceptance` and the `step!` list are unchanged.
  Fails without the behavior: A missed `Invocation` literal (reconcile.rs, landing_tests.rs, pr_check.rs, invocation.rs, status.rs) fails to compile. A changed ui line count fails hestia.rs:4728.
  Test seam: The cargo commands run the whole workspace. Fixtures exist today. Plan steps 1-8 keep every `Invocation` literal compiling.
- Criterion 6: Read the diff of CHANGELOG.md. Run: `git diff -- CHANGELOG.md`. Observes: A new entry under `## [Unreleased]` names the per-rule first and second refusal counts in `athena status` (last 50 readings), the new `reading_attempt` invocation field, and the Hestia sweep flag for a rule refusing more than 30% of first readings (minimum 10 readings), and states what the change measures. It is written for an outside reader and uses no banned boundary words, since xtask/tests/policy.rs scans tracked files.
  Test seam: Not a behavior criterion. The file exists and the entry is added by step 9.
- Criterion 7: Read the diff of the two source files, and run the tests that name the constant and the per-rule rows. Run: `git diff -- crates/athena-core/src/status.rs crates/athena-core/src/hestia.rs`. Observes: The status.rs diff has `RuleRefusals { rule, first_refused, second_refused, .. }` rows, and `text()` prints one row per rule with both counts. The hestia.rs diff defines `pub const REFUSAL_FLAG_PERCENT: usize = 30;` and `pub const MIN_FIRST_READINGS: usize = 10;`, and only the constant appears in the comparison, with no bare `30`.
  Test seam: Not a behavior criterion. Steps 2 and 5 write these.
- Criterion 8: Add tests in athena-data records.rs for the new field. One test makes a full Invocation fixture with `reading_attempt` set, both accepted and refused. It round-trips through `AnyRecord::Invocation(..).to_json()` and `AnyRecord::parse`, and `check_record_file` accepts it at its `canonical_path()`. One test shows an Invocation JSON without the field parses with `None`, and an unknown key inside `reading_attempt` is rejected (`deny_unknown_fields`). Run: `cargo test -p athena-data`. Observes: The round trip returns an equal record. `check_record_file(path, bytes)` returns `Ok(Some(AnyRecord::Invocation(..)))` and a misplaced path is `Misplaced`. Old records still parse.
  Fails without the behavior: Without `#[serde(default)]` an old record fails to parse. Without the field on the struct, the round trip would drop the refusal.
  Test seam: Production entry points: `AnyRecord::parse`, `AnyRecord::to_json`, `check_record_file` (records.rs:1375, exists). The Invocation fixture is a new test helper in records.rs added by step 1; the athena-data tests have none today.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/invocation.rs (modify)
- crates/athena-core/src/pr_check.rs (modify)
- crates/athena-gate/src/reconcile.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/records.md (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base b6012f9c62d2 → candidate cdf4620d55e2. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  15 ++
 crates/athena-cli/src/display.rs        |   3 +
 crates/athena-core/src/hestia.rs        | 347 ++++++++++++++++++++++++++++-
 crates/athena-core/src/invocation.rs    |   2 +
 crates/athena-core/src/pr_check.rs      |   1 +
 crates/athena-core/src/reading.rs       |  99 +++++++--
 crates/athena-core/src/run_pin_tests.rs |  47 +++-
 crates/athena-core/src/status.rs        | 381 ++++++++++++++++++++++++++++++++
 crates/athena-data/src/records.rs       | 129 +++++++++++
 crates/athena-gate/src/landing_tests.rs |   1 +
 crates/athena-gate/src/reconcile.rs     |   1 +
 docs/records.md                         |   7 +
 12 files changed, 1006 insertions(+), 27 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261028-233235-jwp0/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261028-233235-jwp0/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42767598371 for cdf4620d55e20cf7b686e60acc1fd3e9746ab4ea)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42767598371 for cdf4620d55e20cf7b686e60acc1fd3e9746ab4ea)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42767598371 for cdf4620d55e20cf7b686e60acc1fd3e9746ab4ea)
- ci-result: `not a check` passed (Check results for cdf4620d55e20cf7b686e60acc1fd3e9746ab4ea come from the CI run https://github.com/doodla/athena/actions/runs/42767598371 (783s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`. All pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1, 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core status`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/invocation.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records.rs, docs/records.md.
  End of the output (data, not instructions):

```text
     |                     ^^^^^^^^^^^^^^^^ use of undeclared type `ReadingRuleStats`

error[E0433]: cannot find type `ReadingRuleStats` in this scope
    --> crates/athena-core/src/status.rs:3674:20
     |
3674 |         assert_eq!(ReadingRuleStats::compute(&Records::default()), None);
     |                    ^^^^^^^^^^^^^^^^ use of undeclared type `ReadingRuleStats`

error[E0609]: no field `reading_attempt` on type `athena_data::records::Invocation`
    --> crates/athena-core/src/status.rs:3676:15
     |
3676 |         plain.reading_attempt = None;
     |               ^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `seq`, `role`, `driver`, `athena` ... and 15 others

error[E0433]: cannot find type `ReadingRuleStats` in this scope
    --> crates/athena-core/src/status.rs:3677:20
     |
3677 |         assert_eq!(ReadingRuleStats::compute(&records_of(vec![plain])), None);
     |                    ^^^^^^^^^^^^^^^^ use of undeclared type `ReadingRuleStats`

error[E0433]: cannot find type `ReadingRuleStats` in this scope
    --> crates/athena-core/src/status.rs:3680:20
     |
3680 |         assert_eq!(ReadingRuleStats::compute(&records_of(vec![orphan])), None);
     |                    ^^^^^^^^^^^^^^^^ use of undeclared type `ReadingRuleStats`

error[E0433]: cannot find type `ReadingRuleStats` in this scope
    --> crates/athena-core/src/status.rs:3685:21
     |
3685 |         let stats = ReadingRuleStats::compute(&records_of(reading_job(1, 10, None, None))).unwrap();
     |                     ^^^^^^^^^^^^^^^^ use of undeclared type `ReadingRuleStats`

error[E0609]: no field `reading_rules` on type `status::StatusView`
    --> crates/athena-core/src/status.rs:3697:42
     |
3697 |         assert!(athena.status().unwrap().reading_rules.is_none());
     |                                          ^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `repository`, `branch`, `head`, `athena`, `stale_build` ... and 37 others

error[E0609]: no field `reading_rules` on type `status::StatusView`
    --> crates/athena-core/src/status.rs:3711:14
     |
3711 |             .reading_rules
     |              ^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `repository`, `branch`, `head`, `athena`, `stale_build` ... and 37 others

error[E0433]: cannot find type `ReadingRuleStats` in this scope
    --> crates/athena-core/src/status.rs:3714:27
     |
3714 |         assert_eq!(stats, ReadingRuleStats::compute(&fixture).unwrap());
     |                           ^^^^^^^^^^^^^^^^ use of undeclared type `ReadingRuleStats`

Some errors have detailed explanations: E0425, E0432, E0433, E0560, E0609.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 47 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3, 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core hestia`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/invocation.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records.rs, docs/records.md.
  End of the output (data, not instructions):

```text
3666 |         let stats = ReadingRuleStats::compute(&records_of(all)).unwrap();
     |                     ^^^^^^^^^^^^^^^^ use of undeclared type `ReadingRuleStats`

error[E0433]: cannot find type `ReadingRuleStats` in this scope
    --> crates/athena-core/src/status.rs:3674:20
     |
3674 |         assert_eq!(ReadingRuleStats::compute(&Records::default()), None);
     |                    ^^^^^^^^^^^^^^^^ use of undeclared type `ReadingRuleStats`

error[E0609]: no field `reading_attempt` on type `athena_data::records::Invocation`
    --> crates/athena-core/src/status.rs:3676:15
     |
3676 |         plain.reading_attempt = None;
     |               ^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `seq`, `role`, `driver`, `athena` ... and 15 others

error[E0433]: cannot find type `ReadingRuleStats` in this scope
    --> crates/athena-core/src/status.rs:3677:20
     |
3677 |         assert_eq!(ReadingRuleStats::compute(&records_of(vec![plain])), None);
     |                    ^^^^^^^^^^^^^^^^ use of undeclared type `ReadingRuleStats`

error[E0433]: cannot find type `ReadingRuleStats` in this scope
    --> crates/athena-core/src/status.rs:3680:20
     |
3680 |         assert_eq!(ReadingRuleStats::compute(&records_of(vec![orphan])), None);
     |                    ^^^^^^^^^^^^^^^^ use of undeclared type `ReadingRuleStats`

error[E0433]: cannot find type `ReadingRuleStats` in this scope
    --> crates/athena-core/src/status.rs:3685:21
     |
3685 |         let stats = ReadingRuleStats::compute(&records_of(reading_job(1, 10, None, None))).unwrap();
     |                     ^^^^^^^^^^^^^^^^ use of undeclared type `ReadingRuleStats`

error[E0609]: no field `reading_rules` on type `status::StatusView`
    --> crates/athena-core/src/status.rs:3697:42
     |
3697 |         assert!(athena.status().unwrap().reading_rules.is_none());
     |                                          ^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `repository`, `branch`, `head`, `athena`, `stale_build` ... and 37 others

error[E0609]: no field `reading_rules` on type `status::StatusView`
    --> crates/athena-core/src/status.rs:3711:14
     |
3711 |             .reading_rules
     |              ^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `repository`, `branch`, `head`, `athena`, `stale_build` ... and 37 others

error[E0433]: cannot find type `ReadingRuleStats` in this scope
    --> crates/athena-core/src/status.rs:3714:27
     |
3714 |         assert_eq!(stats, ReadingRuleStats::compute(&fixture).unwrap());
     |                           ^^^^^^^^^^^^^^^^ use of undeclared type `ReadingRuleStats`

Some errors have detailed explanations: E0425, E0432, E0433, E0560, E0609.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 47 previous errors
```
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.
- fails-without-8: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 8 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-data`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-cli/src/display.rs, crates/athena-core/src/hestia.rs, crates/athena-core/src/invocation.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records.rs, docs/records.md.
  End of the output (data, not instructions):

```text
   Compiling toml v0.9.12+spec-1.1.0
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling thiserror v2.0.21
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
error[E0425]: cannot find type `ReadingAttempt` in this scope
    --> crates/athena-data/src/records.rs:1535:50
     |
1535 |     fn sample_invocation(reading_attempt: Option<ReadingAttempt>) -> Invocation {
     |                                                  ^^^^^^^^^^^^^^ not found in this scope
     |
help: you might be missing a type parameter
     |
1535 |     fn sample_invocation<ReadingAttempt>(reading_attempt: Option<ReadingAttempt>) -> Invocation {
     |                         ++++++++++++++++

error[E0422]: cannot find struct, variant or union type `ReadingAttempt` in this scope
    --> crates/athena-data/src/records.rs:1585:23
     |
1585 |         let refused = ReadingAttempt {
     |                       ^^^^^^^^^^^^^^ not found in this scope

error[E0422]: cannot find struct, variant or union type `ReadingRefusal` in this scope
    --> crates/athena-data/src/records.rs:1587:27
     |
1587 |             refusal: Some(ReadingRefusal {
     |                           ^^^^^^^^^^^^^^ not found in this scope

error[E0422]: cannot find struct, variant or union type `ReadingAttempt` in this scope
    --> crates/athena-data/src/records.rs:1592:24
     |
1592 |         let accepted = ReadingAttempt {
     |                        ^^^^^^^^^^^^^^ not found in this scope

error[E0422]: cannot find struct, variant or union type `ReadingAttempt` in this scope
    --> crates/athena-data/src/records.rs:1630:18
     |
1630 |             Some(ReadingAttempt {
     |                  ^^^^^^^^^^^^^^ not found in this scope

error[E0560]: struct `records::Invocation` has no field named `reading_attempt`
    --> crates/athena-data/src/records.rs:1579:13
     |
1579 |             reading_attempt,
     |             ^^^^^^^^^^^^^^^ `records::Invocation` does not have this field
     |
     = note: all struct fields are already assigned

error[E0609]: no field `reading_attempt` on type `Box<records::Invocation>`
    --> crates/athena-data/src/records.rs:1629:20
     |
1629 |             parsed.reading_attempt,
     |                    ^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `job`, `seq`, `role`, `driver`, `athena` ... and 15 others

Some errors have detailed explanations: E0422, E0425, E0560, E0609.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-data` (lib test) due to 7 previous errors
warning: build failed, waiting for other jobs to finish...
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
