# Job 20261029-195214-xg16: verify "Let introspections record several causes, open hypotheses,"

## Request (verbatim)

> Priority: normal
> 
> Give requirement OLY-R1-03 a mechanism
> 
> Requirement OLY-R1-03 in docs/spec.toml is active and has no mechanism.
> Requirement: The investigation supports multiple contributing causes, unresolved hypotheses and contradictory evidence.
> Owner: Athena
> 
> Build the code path, role or check that meets the requirement, and record it as the mechanism in docs/spec.toml. Write a measure that says how the requirement is shown and ends with `status: <label>`, where the label is a line `athena status` prints, or `status: not shown` and the test that shows it.
> 
> Spec requirement: OLY-R1-03
> 
> This request is queued again at each drain while the requirement is active and its mechanism or measure is missing; to stop it, fill the mechanism and measure in docs/spec.toml or mark the requirement superseded.
> 
> Acceptance:
> - Requirement OLY-R1-03 names a mechanism that exists in the code, an owner and a measure
> - A test fails without the mechanism and passes with it
> - fmt, clippy and test pass
> 
> Files: docs/spec.toml, crates/

## Reading (accepted by the user)

Introspection records hold one `cause` and nothing for doubt. I read OLY-R1-03 as adding three optional lists to the introspection answer and record: other contributing causes, unresolved hypotheses and contradictory evidence. The mechanism is then recorded in docs/spec.toml with a measure ending `status: not shown` and a test.

Assumptions:
- Goal and audience: Athena's maintainers and the hestia review, who need introspections that show doubt instead of one confident cause.
- Code survey: read crates/athena-core/src/introspect.rs (Answer at ~873, answer_schema at ~1028, apply_answer at ~1468 which redacts and plain-words each text field and sets record.cause) and crates/athena-data/src/records/introspection.rs (IntrospectionRecord with deny_unknown_fields, and validate). The entry point is the introspect run that calls apply_answer. The model's answer is checked in apply_answer before the record is filled. The record fixture and round-trip test are in crates/athena-data/src/records.rs (~1706) and must stay valid. That file holds the allow-list for `athena.introspection/1`. The new fields are serde(default, skip_serializing_if empty), so old records still parse and check_record_file and the AnyRecord round trip still pass. The property is measured directly on the record's new fields, not through a stand-in.
- Design: add `other_causes`, `open_hypotheses` and `contradictions` (Vec<String>) to the record and to Answer (serde default), to the answer schema and to the prompt text. apply_answer redacts and plain-words each entry, caps their count, and drops empty entries. `cause` stays the main cause, so cause counting in hestia is unchanged.
- Existing code: extends crates/athena-core/src/introspect.rs, crates/athena-data/src/records/introspection.rs and docs/spec.toml; creates no new module.
- Term ladder: not used in this request.
- The measure keeps `status: not shown`, because `athena status` prints no line for these lists. It names the test command, so the label 'mistakes' stays out of it. Adding a status line is out of scope.
- No refusal rule is added or tightened. The only new refusal is a reworded or unredactable entry, which fails the introspection the same way the existing fields do. Expected 0% of newly refused readings.
- Consumers: none found. No config deliverable is planned.

## Acceptance criteria

1. [content] docs/spec.toml entry OLY-R1-03 has a mechanism naming introspect.rs and the new record fields instead of "none". It keeps owner Athena. Its measure names the test command and ends with `status: not shown`.
2. [content] crates/athena-data/src/records/introspection.rs IntrospectionRecord has optional, serde-default fields for other causes, open hypotheses and contradictions. Grep for the field names also finds them in Answer and answer_schema in crates/athena-core/src/introspect.rs, and the prompt text tells the model to list them.
3. [behavior] Run `cargo test -p athena-core introspect`. A new test feeds an answer with two extra causes, one open hypothesis and one contradiction. It asserts that the recorded IntrospectionRecord holds all of them and that validate() passes. Without the mechanism the test does not compile or fails.
4. [behavior] Run `cargo test -p athena-data`. An old-format introspection record without the new fields still parses, and the existing records round-trip test passes.
5. [behavior] Run `cargo test -p athena-core introspect`. A new test shows that an answer with an unredactable entry or a word the admission scan rejects in a new list is handled like the existing fields: the introspection is recorded as failed, with a reason.
6. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`. All three pass.

## Expected deliverables

- docs/spec.toml (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-data/src/records/introspection.rs (modify)
- crates/athena-data/src/records.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/labeled.rs (modify)

## Out of scope

- Adding an `athena status` line for these lists.
- Changing how hestia counts causes.
- Changing other requirements in docs/spec.toml.
- Editing athena.toml, defaults.toml or workflows.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the lists, and cargo fmt, clippy and test must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria check the record contents through tests rather than the existence of fields.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The requirement is enforced in code, in apply_answer and the record, and recorded in docs/spec.toml.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the introspection record and answer.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the OLY-R1-03 block in docs/spec.toml (git diff), and run the two tests that parse the whole spec: the hestia.rs:4283 test (`the_real_spec_records_every_field...`, which runs parse_spec and measure_problem on every active requirement) and the xtask spec test. Run: `cargo test -p athena-core the_real_spec_records_every_field && cargo test -p xtask --test spec`. Observes: Mechanism is no longer `none`, names introspect.rs, apply_answer and the three record fields; owner is still Athena; the measure names `cargo test -p athena-core introspect` and ends `status: not shown`; both tests pass.
- Criterion 2: Grep for `other_causes`, `open_hypotheses`, `contradictions` in crates/athena-data/src/records/introspection.rs and crates/athena-core/src/introspect.rs. The new test `the_schema_and_prompt_ask_for_the_three_lists` asserts answer_schema() has the three keys as array properties and in `required`, and that SYSTEM_PROMPT names each key; `an_answer_without_the_three_lists_still_parses` shows Answer's serde default. Run: `cargo test -p athena-core introspect`. Observes: The fields are in IntrospectionRecord with serde default, in Answer, in answer_schema() properties and `required`, and the prompt text tells the model to list them. The schema/prompt test passes.
- Criterion 3: Test `several_causes_hypotheses_and_contradictions_are_recorded` in introspect.rs tests. It builds a `boundary_answer("reviewer")` with two other causes, one hypothesis and one contradiction, calls apply_answer with a blank_record() and the real Redactor, then asserts on the record. A control answer with empty lists is checked too. Run: `cargo test -p athena-core introspect`. Observes: Status is Recorded; record.other_causes has the 2 entries, open_hypotheses 1, contradictions 1, in order; `record.validate()` is Ok; the record round-trips through AnyRecord::to_json and AnyRecord::parse; record.cause still holds the main cause; the control gives empty fields and JSON without the three keys.
  Fails without the behavior: Without the mechanism the test does not compile (Answer and IntrospectionRecord have no such fields). With the fields but no copy in apply_answer, the assertions that the record holds two extra causes, one hypothesis and one contradiction fail.
  Test seam: Production entry point `Athena::apply_answer` (introspect.rs:1468), the function `introspect` calls for every model answer (introspect.rs:1416). Fixture: `athena_with(None)` and `athena.redactor(&config, None)` as in the existing test at introspect.rs:2009-2029; blank_record() at 1915. The Answer is built in the test; no double. All exist; step 7 only adds the three empty fields to boundary_answer and blank_record. Needs gitleaks, as the neighbouring tests do.
- Criterion 4: Existing `introspections_of_the_three_old_kinds_still_parse_and_keep_their_bytes` and the relax/remove round-trip test (records.rs:1735,1759) use `sample_introspection` with none of the new keys. New tests: `an_introspection_with_the_three_lists_round_trips` (parse via check_record_file, assert fields, `to_json` bytes equal input) and `an_introspection_list_with_a_blank_entry_or_too_many_entries_is_invalid` (expects RecordError::Invalid). Run: `cargo test -p athena-data`. Observes: The old sample parses with three empty lists and re-serializes to identical bytes; a sample with the lists round-trips; a blank entry and a sixth entry are Invalid.
  Fails without the behavior: Without `serde(default)` on the new fields the old-format sample fails to parse (missing field); without `skip_serializing_if` the byte-equality assertion at records.rs:1754 fails; without the validate rule the blank-entry record is accepted and the Invalid assertion fails.
  Test seam: Production `check_record_file`, `AnyRecord::parse` and `IntrospectionRecord::validate` (records.rs:1743,1752). The sample is a JSON string built in the test by the helper `sample_introspection_with_lists` added in step 6. No double.
- Criterion 5: Two tests in introspect.rs. (a) `new_list_entries_are_redacted_and_reworded_like_the_other_fields`: via apply_answer with the real Redactor, put a boundary word built from fragments (as introspect.rs:2010 does) in one entry of each new list, `state.root()` in one entry, a blank entry and seven entries in one list. (b) `a_list_entry_that_cannot_be_cleaned_fails_the_answer`: call `clean_texts` with four fields plus list entries, the real Redactor, and a reword closure that returns Err for one marked list entry; also a passing closure. Run: `cargo test -p athena-core introspect`. Observes: (a) Recorded; entries hold `the person running Athena` and `<state>`; boundary_scan and scan_request_input find nothing in any entry or in the whole JSON; blank dropped; list cut to 5. (b) Err("the answer could not be reworded: ..."), the string apply_answer stores in record.failure with status Failed; the clean closure gives Ok. Limit: the closure replaces only the reword step, since no input was found that plain_boundary_text rejects and no non-racy way makes the Redactor fail; the redaction-failure branch is covered by the shared function, not by its own test.
  Fails without the behavior: Without the entries joining the cleaned vector, the boundary word in `contradictions` stays in the record and the boundary_scan assertion fails. Without clean_texts propagating the refusal, the failure-path test gets Ok.
  Test seam: Production `Athena::apply_answer` with the real Redactor (exists) and the private `clean_texts` (step 4 adds it). The closure supplies only the Err of the reword step; the loop, early stop and message are production code.
- Criterion 6: Run the three Athena checks over the whole workspace, then `cargo test -p athena-core --test refusal_corpus` and `cargo test -p xtask --test policy`. No refusal or matching rule is added, so no corpus case is needed; I read only the helper at refusal_corpus.rs:1097, not the case ids. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three commands exit 0: every IntrospectionRecord literal in the workspace compiles with the new fields, the new and old tests pass, the spec tests accept the OLY-R1-03 measure, and the boundary scan finds no banned word in new text.
  Fails without the behavior: If any of the 15 IntrospectionRecord literals lacks the new fields, `cargo clippy --all-targets` and `cargo test --locked` fail to compile (E0063). If the new tests, the validate rule, or the apply_answer copy are missing or wrong, `cargo test --locked` fails. If the OLY-R1-03 measure lacks the ending `status: not shown`, the spec tests fail. Unformatted new code fails fmt.
  Test seam: The cargo commands run over the whole workspace; the production entry points are the compiler and the test suites themselves. No fixture or double. Exists today; step 5 adds the literal fields and step 10 runs the commands.

Planned files:

- docs/spec.toml (modify)
- crates/athena-data/src/records/introspection.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/labeled.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/proposals.rs (modify)
- crates/athena-core/src/evidence_labels.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/tests/refusal_corpus.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/tests/review.rs (modify)
- crates/athena-cli/tests/proposals.rs (modify)
- crates/athena-cli/tests/labeled.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base e0db6c9b3450 → candidate 724b5893172c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                    |   6 +
 crates/athena-cli/tests/drain.rs                |   3 +
 crates/athena-cli/tests/labeled.rs              |   3 +
 crates/athena-cli/tests/proposals.rs            |   3 +
 crates/athena-cli/tests/review.rs               |   3 +
 crates/athena-context/src/briefs.rs             |   6 +
 crates/athena-core/src/evidence_labels.rs       |   3 +
 crates/athena-core/src/hestia.rs                |   3 +
 crates/athena-core/src/introspect.rs            | 256 ++++++++++++++++++++++--
 crates/athena-core/src/labeled.rs               |   3 +
 crates/athena-core/src/proposals.rs             |   3 +
 crates/athena-core/src/reading.rs               |   3 +
 crates/athena-core/src/status.rs                |   6 +
 crates/athena-core/tests/refusal_corpus.rs      |   3 +
 crates/athena-data/src/records.rs               |  64 ++++++
 crates/athena-data/src/records/introspection.rs |  26 +++
 docs/spec.toml                                  |   4 +-
 17 files changed, 378 insertions(+), 20 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261029-195214-xg16/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261029-195214-xg16/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42927297318 for 724b5893172c45ec772c5c01ea3fdb2ff2277433)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42927297318 for 724b5893172c45ec772c5c01ea3fdb2ff2277433)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42927297318 for 724b5893172c45ec772c5c01ea3fdb2ff2277433)
- ci-result: `not a check` passed (Check results for 724b5893172c45ec772c5c01ea3fdb2ff2277433 come from the CI run https://github.com/doodla/athena/actions/runs/42927297318 (836s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test`. All three pass.)
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3, 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core introspect`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/introspect.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records/introspection.rs, docs/spec.toml.
  End of the output (data, not instructions):

```text
error[E0560]: struct `athena_data::records::IntrospectionRecord` has no field named `contradictions`
    --> crates/athena-core/src/reading.rs:1952:13
     |
1952 |             contradictions: Vec::new(),
     |             ^^^^^^^^^^^^^^ `athena_data::records::IntrospectionRecord` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::IntrospectionRecord` has no field named `other_causes`
    --> crates/athena-core/src/status.rs:1452:13
     |
1452 |             other_causes: Vec::new(),
     |             ^^^^^^^^^^^^ `athena_data::records::IntrospectionRecord` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::IntrospectionRecord` has no field named `open_hypotheses`
    --> crates/athena-core/src/status.rs:1453:13
     |
1453 |             open_hypotheses: Vec::new(),
     |             ^^^^^^^^^^^^^^^ `athena_data::records::IntrospectionRecord` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::IntrospectionRecord` has no field named `contradictions`
    --> crates/athena-core/src/status.rs:1454:13
     |
1454 |             contradictions: Vec::new(),
     |             ^^^^^^^^^^^^^^ `athena_data::records::IntrospectionRecord` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::IntrospectionRecord` has no field named `other_causes`
    --> crates/athena-core/src/status.rs:2644:13
     |
2644 |             other_causes: Vec::new(),
     |             ^^^^^^^^^^^^ `athena_data::records::IntrospectionRecord` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::IntrospectionRecord` has no field named `open_hypotheses`
    --> crates/athena-core/src/status.rs:2645:13
     |
2645 |             open_hypotheses: Vec::new(),
     |             ^^^^^^^^^^^^^^^ `athena_data::records::IntrospectionRecord` does not have this field
     |
     = note: all struct fields are already assigned

error[E0560]: struct `athena_data::records::IntrospectionRecord` has no field named `contradictions`
    --> crates/athena-core/src/status.rs:2646:13
     |
2646 |             contradictions: Vec::new(),
     |             ^^^^^^^^^^^^^^ `athena_data::records::IntrospectionRecord` does not have this field
     |
     = note: all struct fields are already assigned

Some errors have detailed explanations: E0425, E0560, E0609.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-core` (lib test) due to 51 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-data`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/introspect.rs, crates/athena-core/src/status.rs, crates/athena-data/src/records/introspection.rs, docs/spec.toml.
  End of the output (data, not instructions):

```text
   Compiling syn v3.0.7
   Compiling winnow v0.7.15
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling getrandom v0.3.4
   Compiling toml v0.9.12+spec-1.1.0
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.21
   Compiling thiserror v2.0.21
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
error[E0609]: no field `other_causes` on type `&Box<introspection::IntrospectionRecord>`
    --> crates/athena-data/src/records.rs:1778:15
     |
1778 |             o.other_causes.is_empty()
     |               ^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `id`, `at`, `trigger`, `job`, `intervention` ... and 19 others

error[E0609]: no field `open_hypotheses` on type `&Box<introspection::IntrospectionRecord>`
    --> crates/athena-data/src/records.rs:1779:22
     |
1779 |                 && o.open_hypotheses.is_empty()
     |                      ^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `id`, `at`, `trigger`, `job`, `intervention` ... and 19 others

error[E0609]: no field `contradictions` on type `&Box<introspection::IntrospectionRecord>`
    --> crates/athena-data/src/records.rs:1780:22
     |
1780 |                 && o.contradictions.is_empty()
     |                      ^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `id`, `at`, `trigger`, `job`, `intervention` ... and 19 others

error[E0609]: no field `other_causes` on type `&Box<introspection::IntrospectionRecord>`
    --> crates/athena-data/src/records.rs:1791:22
     |
1791 |         assert_eq!(i.other_causes, ["b", "c"]);
     |                      ^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `id`, `at`, `trigger`, `job`, `intervention` ... and 19 others

error[E0609]: no field `open_hypotheses` on type `&Box<introspection::IntrospectionRecord>`
    --> crates/athena-data/src/records.rs:1792:22
     |
1792 |         assert_eq!(i.open_hypotheses, ["h"]);
     |                      ^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `id`, `at`, `trigger`, `job`, `intervention` ... and 19 others

error[E0609]: no field `contradictions` on type `&Box<introspection::IntrospectionRecord>`
    --> crates/athena-data/src/records.rs:1793:22
     |
1793 |         assert_eq!(i.contradictions, ["x"]);
     |                      ^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `id`, `at`, `trigger`, `job`, `intervention` ... and 19 others

For more information about this error, try `rustc --explain E0609`.
error: could not compile `athena-data` (lib test) due to 6 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
