# Job 20261029-195211-3ary: verify "Give OLY-5-07 a mechanism: steward proposals need general evidence"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-5-07 a mechanism
> 
> Requirement OLY-5-07 in docs/spec.toml is active and has no mechanism.
> Requirement: Athena favors the narrowest change that addresses a demonstrated general weakness, and a safeguard addresses a meaningful class of failure without imposing inappropriate universal rules.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-5-07
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-5-07 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Add a rule in the steward's validate_item (crates/athena-core/src/steward.rs) that refuses a non-blind-spot improvement proposal citing fewer than two distinct evidence references, so a one-off is not treated as a general weakness. Record it in docs/spec.toml as OLY-5-07's mechanism with a measure ending `status: not shown` and the test that shows it. The audience is the Athena owner and the verifier judging the spec row.

Assumptions:
- Code survey: read crates/athena-core/src/steward.rs (ProposedChange, validate_items, validate_item, check_reference), crates/athena-core/src/admission.rs, and crates/athena-core/src/hestia.rs (measure validation at ~1218). The entry point is validate_items, which calls validate_item per item. Source and evidence references are checked with check_reference, then the rung and request checks run. No record or data file under crates/ is added, so check_record_file, AnyRecord and the round-trip test are not affected. The property is measured directly by calling validate_item or validate_items in unit tests.
- Existing code: extends crates/athena-core/src/steward.rs (validate_item, whose evidence-required check already exists) and docs/spec.toml, whose OLY-5-06 row already points at steward.rs. Nothing new is created.
- Interpretation: 'demonstrated general weakness' is mechanized as two or more distinct evidence references, and 'narrowest change' is already served by the single-rung ProposedChange. The first is an assumption the owner may correct.
- Environment: none; no CI workflow, push or history read is involved.
- No governance file (athena.toml, defaults.toml, workflows) needs to change.

## Acceptance criteria

1. [content] docs/spec.toml requirement OLY-5-07 has a mechanism that names validate_item in crates/athena-core/src/steward.rs (no longer "none"), owner "Athena", and a measure ending `status: not shown` that names the test, e.g. `cargo test -p athena-core steward`. Check with Grep on docs/spec.toml.
2. [behavior] validate_item in crates/athena-core/src/steward.rs rejects a repeated-failure or proposal item whose evidence holds fewer than two distinct references, with a reason that names the key and says one instance is not a general weakness. It accepts the same item with two distinct references. Blind spots are exempt. Run `cargo test -p athena-core steward`.
3. [behavior] A new steward unit test fails when the new check is removed: a single-evidence item is rejected with the new reason, and a duplicated reference counts once. It passes with the check. Run `cargo test -p athena-core steward`.
4. [behavior] Existing steward tests and fixtures that build valid items with one evidence reference are updated to cite two, and the whole workspace passes: `cargo fmt --check`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`.
5. [content] The newly refused share is expected at 0% of newly refused readings, because the rule applies only to steward proposals and not to readings or admission. The record-validation allow-list is untouched, since no record or data file is added.
6. [content] CHANGELOG.md has an entry for the new rule. The spec.toml status-measure validation in crates/athena-core/src/hestia.rs still accepts the measure.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/steward.rs (modify)
- CHANGELOG.md (modify)
- docs/enforcement.md (modify)

## Out of scope

- Changing admission, reading checks or the replay refusal rules
- Changing prompts or role definitions
- Editing other spec requirements

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit test, clippy and fmt required
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rule enforced in validate_item rather than prose
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the steward tests
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One check, one test, one spec row
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check by Grep of the OLY-5-07 row. `the_real_spec_passes` and `measure_problem` are regression checks only. Run: `Grep pattern 'id = "OLY-5-07"' with -A 6 on docs/spec.toml; then `cargo test -p xtask --test spec the_real_spec_passes`; then `cargo test -p athena-core hestia``. Observes: The mechanism is no longer `none`; it names `validate_item` and `crates/athena-core/src/steward.rs`. The owner is `Athena`. The measure is `cargo test -p athena-core steward; status: not shown`. The xtask spec test and the hestia tests still pass (`measure_problem`, hestia.rs ~1219-1229, accepts a label starting `not shown`).
- Criterion 2: Unit test in the steward.rs test module calls `validate_items` (public, called by `consult`), which calls the private `validate_item`. The real code runs with no double. Each case clones `item()` and replaces `evidence` and, for proposals, `kind`. Refused: a repeated-failure item with one job; a repeated-failure item with a job plus a path; a repeated-failure item with a job plus a commit (`abcdef1`); a proposal with one job. Each reason must contain the key, `not a general weakness` and `1 distinct`. Accepted: a repeated failure with two distinct jobs; a proposal with a job plus `docs/spec.toml`; a proposal with two paths other than its source; a blind spot with empty evidence. Run: `cargo test -p athena-core steward::tests::one_instance_is_not_a_general_weakness`. Observes: The one-instance and padded items land in `rejected` with the new reason and no change. The two-instance items and the blind spot land in `changes`.
  Fails without the behavior: Without the check, the one-evidence items are accepted, `rejected` stays empty, and the first `contains("not a general weakness")` assertion fails.
  Test seam: Entry point `validate_items` -> `validate_item` (steward.rs:932, 954), called from the in-file test module. Fixture `records_with_job` (exists, steward.rs:1970) is extended by step 3 to hold jobs k7q2 and k7q3, and `item()` (exists, 1982) is edited by step 3 to cite both. No double.
- Criterion 3: The same new test also builds: one job cited twice (once as `' 20261010-153012-k7q2 '`, once exact); a proposal whose evidence is a job plus a path equal to its own source (`records/jobs`); and a repeated failure with a job plus a made-up commit. Each must be refused with `1 distinct` in the reason. The producer comments out the check, runs the test and quotes the failing assertion output in the report, then restores the check and runs it again. Run: `cargo test -p athena-core steward::tests::one_instance_is_not_a_general_weakness`. Observes: With the check removed the test fails at its first refusal assertion. With it restored the test passes. The duplicate, own-source and padding cases show that raw entries are not counted.
  Fails without the behavior: Without the check these items are accepted and `rejected` is empty, so the test fails at its first assertion.
  Test seam: Same entry point and fixtures as criterion 2; the step-5 test adds these cases. No double.
- Criterion 4: Run the steward module, the CLI steward integration test file, then fmt, clippy and the whole test suite. Edited core tests: `a_valid_item_becomes_a_proposed_change` asserts both job references; `the_steward_output_parser_reads_the_five_brief_fields` gets a second job evidence object. Edited CLI tests (crates/athena-cli/tests/steward.rs): `repeated_failure_items` gives `proposal:brief-readme` evidence `jobs[0]` and `jobs[1]` (lines 177); the plain proposal in `a_missing_token_reaches_the_owner_with_its_brief_and_a_plain_gap_does_not` cites `BRIEF.md` and `docs/spec.toml` (line 561) while keeping source `README.md`; `bad:request` in `items_that_fail_validation_are_reported_and_nothing_is_queued_for_them` cites `jobs[0]` and `jobs[1]` (line 685) and the test asserts its reason names the key and `needs a request`; a fourth item `bad:evidence` (repeated failure, one job, with a request) is added and its reason must contain `not a general weakness`, so `rejected.len()` becomes 4. Run: `cargo test -p athena-core steward && cargo test -p athena-cli --test steward && cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All steward tests pass. In the CLI file, `scripted_output_with_a_real_gap_...` still queues both items with no rejections, `a_second_run_with_the_same_keys_queues_nothing_again` still sees 2 queued, and the missing-token test still queues its proposal. fmt, clippy and the workspace tests finish clean.
  Fails without the behavior: If `item()` kept one evidence reference, every core test using it as a valid item would fail with the new reason, such as `a_run_queues_at_most_twenty_items`. If the CLI proposals kept one reference, `scripted_output_with_a_real_gap_...` would find `report.rejected` non-empty.
  Test seam: Entry points: `validate_items` for the core tests; `steward::run` (via `run_steward` in the CLI file) with `steward_scenario(items)` feeding `scenario.steward = json!({"items": items})` (CLI tests steward.rs:125) for the CLI tests. Both exist today. The CLI items are built with the file's `item(...)` helper and the two real rejected jobs from `after_two_failures` (exists). The new `bad:evidence` item is added by step 6. No double holds the rule.
- Criterion 5: Content check from the code. The rule's only call site is `validate_items` in `consult` (steward.rs ~1288); reading.rs and admission.rs do not call it. No record or data file is added, so the record allow-list is untouched. No refusal_corpus case reaches steward evidence validation, so no input form here maps to a corpus case id and none is added; the corpus runs as a regression. Input forms the rule handles: one job; a repeated job (padded or exact); job plus path; job plus made-up commit; proposal with its source echoed; proposal with two distinct paths; blind spot with no evidence. The first six are covered by the step-5 unit test, the last by the existing blind-spot tests and the same test. Expected share of newly refused readings: 0% (an estimate from the code, not a measurement). Run: `cargo test -p athena-core --test refusal_corpus`. Observes: The corpus passes unchanged. `git diff --stat` lists only docs/spec.toml, steward.rs, crates/athena-cli/tests/steward.rs, CHANGELOG.md and docs/enforcement.md.
- Criterion 6: Grep CHANGELOG.md for the new bullet under [Unreleased] / Added naming OLY-5-07, `validate_item`, the two-instance rule, jobs-only for repeated failures, the exempt blind spots and the unmechanized clause. Run the hestia tests that call `measure_problem` and the xtask tests, which check the new enforcement row's test names. Run: `cargo test -p athena-core hestia && cargo test -p xtask`. Observes: The CHANGELOG bullet exists. `measure_problem` accepts `cargo test -p athena-core steward; status: not shown`. `every_test_named_in_the_enforcement_map_exists` passes with row M34.

Planned files:

- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- docs/spec.toml (modify)
- CHANGELOG.md (modify)
- docs/enforcement.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 0% of newly refused readings (the job is held above the declared share plus 5 points).
Reason given for the declared share (judge whether it explains why past readings should be refused): The newly refused share is expected at 0% of newly refused readings, because the rule applies only to steward proposals and not to readings or admission. The record-validation allow-list is untouched, since no record or data file is added.

## The change

Base f9d65a3ddf7c → candidate 600054c02fbc. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                       |   6 ++
 crates/athena-cli/tests/steward.rs |  28 +++++-
 crates/athena-core/src/steward.rs  | 187 +++++++++++++++++++++++++++++++++----
 docs/enforcement.md                |   1 +
 docs/spec.toml                     |   4 +-
 5 files changed, 204 insertions(+), 22 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-195211-3ary/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-195211-3ary/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42917522061 for 600054c02fbc6ad8f8c79be694a29d3b14c02937)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42917522061 for 600054c02fbc6ad8f8c79be694a29d3b14c02937)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42917522061 for 600054c02fbc6ad8f8c79be694a29d3b14c02937)
- ci-result: `not a check` passed (Check results for 600054c02fbc6ad8f8c79be694a29d3b14c02937 come from the CI run https://github.com/doodla/athena/actions/runs/42917522061 (904s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Existing steward tests and fixtures that build valid items with one evidence reference are updated to cite two, and the whole workspace passes: `cargo fmt --check`, `cargo clippy --workspace --all-targets`, `cargo test --workspace`.)
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2, 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core steward::tests::one_instance_is_not_a_general_weakness`, exit code 101. Result: the named test command failed without the fix (exit 101).
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/steward.rs, docs/enforcement.md, docs/spec.toml.
  End of the output (data, not instructions):

```text
   Compiling syn v3.0.7
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling serde_derive_internals v0.30.0
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling ref-cast v1.0.27
   Compiling schemars_derive v1.2.2
   Compiling thiserror v2.0.21
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 48.62s
     Running unittests src/lib.rs (<job>/target/verify/debug/deps/athena_core-18f5333b8c967527)

running 1 test
test steward::tests::one_instance_is_not_a_general_weakness ... FAILED

failures:

---- steward::tests::one_instance_is_not_a_general_weakness stdout ----

thread 'steward::tests::one_instance_is_not_a_general_weakness' (12384) panicked at crates/athena-core/src/steward.rs:2291:13:
accepted RepeatedFailure with [StewardRefOutput { kind: Job, value: "20261010-153012-k7q2" }]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    steward::tests::one_instance_is_not_a_general_weakness

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1308 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p athena-core --lib`
     Running tests/external_calls_guard.rs (<job>/target/verify/debug/deps/external_calls_guard-a2cb0acbd401486e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/refusal_corpus.rs (<job>/target/verify/debug/deps/refusal_corpus-a935feb820f2850a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 23 filtered out; finished in 0.00s

error: 1 target failed:
    `-p athena-core --lib`
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test -p athena-core steward && cargo test -p athena-cli --test steward && cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
