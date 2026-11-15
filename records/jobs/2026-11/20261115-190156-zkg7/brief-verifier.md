# Job 20261115-190156-zkg7: verify "Show the producer's pre-verdict criteria check in introspection events"

## Request (verbatim)

> try again on 20261114-160745-8e6x

## Reading (accepted by the user)

Retry of job 20261114-160745-8e6x. The failure class "nothing in the stored event shows the unmet criterion checked before the verdict" is about the introspection Event built in `event_of_job`, which reads only the outcome reason and verification.json. The fix adds, for each criterion the verification marks unmet, the producer's stored pre-verdict status and evidence from `candidate.producer_report.criteria_check`. The audience is the introspection stage, whose prompt says it can read nothing else.

Assumptions:
- Job 20261114-160745-8e6x (failed at planning, no candidate, no files changed): the critic found the reading misread 'stored event' as the producer invocation detail and wrongly claimed a passing self-check is not stored. This reading addresses each finding: the root cause is now the introspection event input (`event_of_job`, `failure_details`); the passing check is acknowledged as already stored in `candidate.producer_report`; a test asserts on the event's `what` and fails on base; the producer.rs invocation-detail change, the `; redacted report: ` parse risk, the briefs.rs unchanged-file send-back and the detail-overwrite nit no longer apply because producer.rs, send_back.rs and briefs.rs are not changed.
- Code survey: read crates/athena-core/src/introspect.rs (`event_of_job` at ~1473 builds Event.what from title, outcome.reason and `failure_details(verification)`; `failure_details` ~984 lists unmet criteria, blocker/major findings and failed checks; tests `what_of`, `athena_with` ~2081 commit only outcome and verification; `an_event_carries_the_unmet_criterion_and_the_major_finding` ~2715), crates/athena-data/src/records.rs (`ReportedCriterion {criterion,status,evidence}`, `ProducerReport.criteria_check`, `Candidate.producer_report`), and the job records struct with `candidate: Option<Candidate>` (crates/athena-core/src/store.rs:37 and athena-gate jobs.rs). No new record or data file is added, so no records allow-list or AnyRecord round-trip change applies. Candidate is stored as a record file; the test must commit it like the Verification in `athena_with`.
- Consuming stage: the introspection stage, whose input is built in `event_of_job`. Matching of producer criteria to verification criteria uses the same normalisation as `criterion_key` in crates/athena-core/src/run/send_back.rs (reuse it by making it pub(crate), or an equivalent trim/lowercase compare). Evidence text is clipped with the existing `clip` and TEXT_CAP, and the count with DETAIL_CAP.
- Adds no refusal rule: expected 0% of newly refused readings.
- Existing code: crates/athena-core/src/introspect.rs `event_of_job`, `failure_details`, `clip`; crates/athena-data/src/records.rs `ProducerReport`; crates/athena-core/src/run/send_back.rs `criterion_key`.

## Acceptance criteria

1. [content] crates/athena-core/src/introspect.rs: `event_of_job` loads the job's stored candidate and, for each criterion the verification marks unmet, appends to `what` the producer's pre-verdict status and evidence taken from `candidate.producer_report.criteria_check` (clipped with the existing caps). A criterion the producer did not list is shown as having no recorded pre-verdict check. Nothing is added when the verification passed or no candidate/report is stored.
2. [behavior] A new test in crates/athena-core/src/introspect.rs, in the style of `an_event_carries_the_unmet_criterion_and_the_major_finding`, commits a Candidate whose producer_report marks criterion C as met with some evidence plus a failed Verification with C unmet. It calls `event_of_job` and asserts `what` contains C's pre-verdict status and evidence. It fails on the base tree and passes with the change; check with `cargo test -p athena-core introspect`.
3. [behavior] A second new test covers an unmet criterion absent from criteria_check, and a job with no stored candidate. It asserts `what` states no pre-verdict check was recorded and that the existing assertions on `what` (unmet list, major findings, failed checks) still hold; check with `cargo test -p athena-core introspect`.
4. [content] The root cause is named in the CHANGELOG.md entry (a new entry; docs/decisions.md and records/ are not edited): the self-check was already stored in candidate.producer_report, but the introspection event built in `event_of_job` never included it, so the stage that reports the failure could not see the check.
5. [behavior] Existing introspect and run_pin tests still pass: `cargo test` passes on the final tree, as does `cargo clippy --all-targets` and `cargo fmt --check`, as recorded by Athena's checks.

## Expected deliverables

- crates/athena-core/src/introspect.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/send_back.rs (modify)

## Out of scope

- crates/athena-core/src/run/producer.rs, crates/athena-core/src/run/tests.rs and crates/athena-context/src/briefs.rs: the prior job's planned files, dropped (no candidate existed); no change to producer invocation detail
- Prompt edits (reader.md, producer.md, verifier.md), the reader token budget, crates/athena-context/src/lib.rs, crates/athena-core/src/verify.rs, crates/athena-cli/tests/budgets.rs
- docs/decisions.md, docs/records.md, docs/design/* edits and anything under records/ or owner files
- A new repair loop, changing PRODUCER_SEND_BACKS, or changing the verifier or its verdict rules
- Any new refusal rule

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy, test criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix at the introspection event builder, not at one job.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Tests assert on the event's `what`, the consuming stage's input.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Read the diff for the three content claims. (a) `event_of_job` passes `job_records.candidate.as_ref().and_then(|c| c.producer_report.as_ref())` into `failure_details`. (b) The block is built only after the `Verdict::Pass` early return, uses `clip(.., TEXT_CAP)` and `.take(DETAIL_CAP)`, and writes the `no pre-verdict check recorded` phrase for an unmatched criterion. (c) Nothing is added for a passing verification, and the existing 'lists no unmet criterion' return is untouched. Then run `cargo test -p athena-core introspect` to show the behavior holds. Run: `git diff crates/athena-core/src/introspect.rs crates/athena-core/src/run/send_back.rs`. Observes: The diff shows exactly those pieces, plus `criterion_key` moved to introspect.rs and imported by send_back.rs. The test run (criteria 2 and 3) shows the effect.
- Criterion 2: New unit test in the introspect.rs test module, in the style of `an_event_carries_the_unmet_criterion_and_the_major_finding` (2715). It commits a Candidate whose `producer_report.criteria_check` has `{criterion: 'Tests call event_of_job itself', status: 'met', evidence: 'ran the new test and it passed'}` and a failed Verification with that criterion unmet (via `criterion(text, false)`, whose verifier evidence is 'evidence that must not be copied'). It calls `event_of_job` and asserts: `what` contains the criterion, the producer's status and the evidence text; the old assertions still hold (`Unmet acceptance criteria (1):`, the major finding line); `what` does not contain 'must not be copied' (the verifier's evidence) or 'Failed checks'. A second case stores the producer's criterion as `[behavior] Tests call event_of_job itself.` and asserts the same match through `criterion_key`. Run: `cargo test -p athena-core introspect::tests::an_unmet_criterion_shows_the_producers_pre_verdict_check`. Observes: The test passes with the change. `what` shows, under the unmet list, the criterion with the producer's status 'met' and the producer's evidence.
  Fails without the behavior: On the base tree `event_of_job` never reads `job_records.candidate`, so `what` has no text from `candidate.producer_report`. The asserts on the producer's status 'met' and the evidence 'ran the new test and it passed' fail. The `[behavior] ... .` variant case fails for the same reason.
  Test seam: Production entry point `Athena::event_of_job` (introspect.rs:1473), called from the in-module test. Fixture: a real temporary git repository holding committed record files, built by the test helper `athena_with` (introspect.rs:2081, exists). Step 4 adds `athena_with_candidate(verification, candidate)`, with `athena_with` delegating to it with `None`, and a `what_of_with` helper. The Candidate is committed as `AnyRecord::Candidate(Box::new(c))`, written to `<job dir>/candidate.json` (records.rs:1326-1328) and read back by `Records::load` (store.rs:207). It is a /1 record with no base_fingerprint, as records.rs:1230-1235 requires. Nothing is faked; the logic under test is production code.
- Criterion 3: Two new tests. (a) `an_unmet_criterion_with_no_pre_verdict_check_says_so`: a Candidate whose `criteria_check` lists only other criteria (second case: empty list; third case: `producer_report: None`) and a failed Verification with criterion C unmet; assert `what` contains C and 'no pre-verdict check recorded', and does not contain the other criteria's evidence. (b) `a_job_without_a_candidate_says_no_pre_verdict_check_was_recorded`: no Candidate committed (via `athena_with`), failed Verification with C unmet, a major finding and, in a second case, a failing check; assert `what` contains 'no pre-verdict check recorded' and still contains 'Unmet acceptance criteria (1):', '- C', the 'major: ... at ...' line and 'Failed checks'. The existing exact-equality test `a_job_without_verification_or_without_failures_keeps_the_old_what` (2782-2791) stays unchanged to show a missing or passing verification adds nothing. A fourth test, `the_producers_pre_verdict_evidence_is_capped`, gives 11 unmet criteria with matching entries whose evidence is 'e' x 1200 and asserts no run of 401 'e' and exactly DETAIL_CAP producer lines. Run: `cargo test -p athena-core introspect::tests::an_unmet_criterion_with_no_pre_verdict_check_says_so`. Observes: The statement appears for an unlisted criterion, for a candidate without a report, and for a job with no candidate. The old assertions on `what` hold.
  Fails without the behavior: On the base tree `what` never contains 'no pre-verdict check recorded', so the asserts on that phrase fail in all three cases: criterion absent from `criteria_check`, empty `criteria_check` or `producer_report: None`, and no stored candidate.
  Test seam: Production entry point `Athena::event_of_job` via `athena_with` (exists, introspect.rs:2081) for the no-candidate job, and via `athena_with_candidate` (added in plan step 4) for the other cases. Same real-git-repository fixture as criterion 2; the Candidate is a real record value with `producer_report` None or with a list of other criteria.
- Criterion 4: Read the diff of CHANGELOG.md and `git status` for the other changed paths. The diff must show one new bullet at the top of `### Fixed` under `## [Unreleased]` (CHANGELOG.md:21). The bullet states that the producer's self-check was already stored in `candidate.producer_report`, that the introspection event built in `event_of_job` read only the outcome reason and `verification.json`, so the stage that reports the failure could not see the check, and that the passing check was already stored. It names `failure_details`, the moved `criterion_key`, the caps, the matching limit, how the criterion 1/3 conflict was resolved, 'no refusal rule added (0% newly refused)', and the tests. `git status` must list no change under docs/decisions.md, docs/design or records/. Run: `git diff CHANGELOG.md`. Observes: The entry names the root cause and the fix. No other doc or record file is in the change.
- Criterion 5: Run the checks Athena runs, one command each: `cargo fmt --all --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. The test run covers: the introspect unit tests that exist today (introspect.rs:2715-2830, which now also carry the no-candidate line where they have an unmet criterion); `run_pin_tests` and the send_back.rs test modules, which drive `unmet_criteria` and `left_to_athena` and so the moved `criterion_key`; and the end-to-end introspection tests of the CLI crate. In that CLI test the fake producer reports every criterion as met with evidence 'checked by the producer' (athena-cli/tests/common/mod.rs:385, 592-605), so the new block reads '...marked met; its evidence: checked by the producer', which holds none of the asserted-absent strings. Run: `cargo test --locked`. Observes: All three commands exit 0. The 2, 3 and 4 new tests run in the same pass and pass.
  Fails without the behavior: A regression guard, so it fails when the behavior is broken. Dropping the `use crate::introspect::criterion_key;` line, or leaving `criterion_key` private, is a compile error in send_back.rs. Changing the moved normalisation breaks `a_criterion_reported_with_only_cosmetic_differences_counts_as_reported` and `a_missing_unmet_or_different_criterion_is_still_listed` (run_pin_tests.rs:1076, 1102) and the left_to_athena_tests (send_back.rs:576-612). A block added to a passing or missing verification breaks the exact-equality test at introspect.rs:2782-2791. Text containing 'Failed checks', 'trailing space' or 'the tests still pass' breaks introspect.rs:2715-2745 and athena-cli/tests/introspection.rs:421-471. A lint or format slip fails the two other check commands.
  Test seam: Production entry points that existing tests already reach, all existing: (1) `Athena::event_of_job` through `athena_with` and `what_of` in the introspect.rs test module (2081, 2705), with a real git repository fixture; (2) `crate::run::unmet_criteria` and `left_to_athena` (send_back.rs:18, 37), through the `report` and `accepted` helpers in run_pin_tests.rs:1063 and send_back.rs:543-570, which build a real `ProducerOutput` and `AcceptedReading` and inject nothing fake; (3) a whole job and `introspect_job` (run/mod.rs:435) in athena-cli/tests/introspection.rs:421-471, with `setup(scenario, &model)` and `run_job` injecting a fake model and a fake producer driver (common/mod.rs). The seam exists today; no step adds it.

Planned files:

- crates/athena-core/src/introspect.rs (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 0b8daffd6aa0 → candidate 55acbcd53869. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |   1 +
 crates/athena-core/src/introspect.rs    | 244 +++++++++++++++++++++++++++++++-
 crates/athena-core/src/run/send_back.rs |  24 +---
 3 files changed, 240 insertions(+), 29 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261115-190156-zkg7/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261115-190156-zkg7/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43753902885 for 55acbcd53869f30e6b98ae00a944fd152f5c3662)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43753902885 for 55acbcd53869f30e6b98ae00a944fd152f5c3662)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43753902885 for 55acbcd53869f30e6b98ae00a944fd152f5c3662)
- ci-result: `not a check` passed (Check results for 55acbcd53869f30e6b98ae00a944fd152f5c3662 come from the CI run https://github.com/doodla/athena/actions/runs/43753902885 (860s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Existing introspect and run_pin tests still pass: `cargo test` passes on the final tree, as does `cargo clippy --all-targets` and `cargo fmt --check`, as recorded by Athena's checks.)
- fails-without-2: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 2 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core introspect::tests::an_unmet_criterion_shows_the_producers_pre_verdict_check`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/introspect.rs, crates/athena-core/src/run/send_back.rs.
  End of the output (data, not instructions):

```text
   Compiling regex v1.13.1
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find type `ProducerReport` in this scope
    --> crates/athena-core/src/introspect.rs:2770:59
     |
2770 |     fn producer_report(entries: &[(&str, &str, &str)]) -> ProducerReport {
     |                                                           ^^^^^^^^^^^^^^ not found in this scope

error[E0422]: cannot find struct, variant or union type `ProducerReport` in this scope
    --> crates/athena-core/src/introspect.rs:2771:9
     |
2771 |         ProducerReport {
     |         ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find type `ProducerReport` in this scope
    --> crates/athena-core/src/introspect.rs:2791:38
     |
2791 |     fn candidate_with(report: Option<ProducerReport>) -> Candidate {
     |                                      ^^^^^^^^^^^^^^ not found in this scope
     |
help: you might be missing a type parameter
     |
2791 |     fn candidate_with<ProducerReport>(report: Option<ProducerReport>) -> Candidate {
     |                      ++++++++++++++++

error[E0425]: cannot find value `NO_PRE_VERDICT_CHECK` in this scope
    --> crates/athena-core/src/introspect.rs:2846:36
     |
2846 |             assert!(!what.contains(NO_PRE_VERDICT_CHECK), "{what}");
     |                                    ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `NO_PRE_VERDICT_CHECK` in this scope
    --> crates/athena-core/src/introspect.rs:2856:35
     |
2856 |             assert!(what.contains(NO_PRE_VERDICT_CHECK), "{what}");
     |                                   ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `NO_PRE_VERDICT_CHECK` in this scope
    --> crates/athena-core/src/introspect.rs:2865:31
     |
2865 |         assert!(what.contains(NO_PRE_VERDICT_CHECK), "{what}");
     |                               ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `NO_PRE_VERDICT_CHECK` in this scope
    --> crates/athena-core/src/introspect.rs:2881:37
     |
2881 |         assert!(with_check.contains(NO_PRE_VERDICT_CHECK), "{with_check}");
     |                                     ^^^^^^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0422, E0425.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 7 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core introspect::tests::an_unmet_criterion_with_no_pre_verdict_check_says_so`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/introspect.rs, crates/athena-core/src/run/send_back.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0425]: cannot find type `ProducerReport` in this scope
    --> crates/athena-core/src/introspect.rs:2770:59
     |
2770 |     fn producer_report(entries: &[(&str, &str, &str)]) -> ProducerReport {
     |                                                           ^^^^^^^^^^^^^^ not found in this scope

error[E0422]: cannot find struct, variant or union type `ProducerReport` in this scope
    --> crates/athena-core/src/introspect.rs:2771:9
     |
2771 |         ProducerReport {
     |         ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find type `ProducerReport` in this scope
    --> crates/athena-core/src/introspect.rs:2791:38
     |
2791 |     fn candidate_with(report: Option<ProducerReport>) -> Candidate {
     |                                      ^^^^^^^^^^^^^^ not found in this scope
     |
help: you might be missing a type parameter
     |
2791 |     fn candidate_with<ProducerReport>(report: Option<ProducerReport>) -> Candidate {
     |                      ++++++++++++++++

error[E0425]: cannot find value `NO_PRE_VERDICT_CHECK` in this scope
    --> crates/athena-core/src/introspect.rs:2846:36
     |
2846 |             assert!(!what.contains(NO_PRE_VERDICT_CHECK), "{what}");
     |                                    ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `NO_PRE_VERDICT_CHECK` in this scope
    --> crates/athena-core/src/introspect.rs:2856:35
     |
2856 |             assert!(what.contains(NO_PRE_VERDICT_CHECK), "{what}");
     |                                   ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `NO_PRE_VERDICT_CHECK` in this scope
    --> crates/athena-core/src/introspect.rs:2865:31
     |
2865 |         assert!(what.contains(NO_PRE_VERDICT_CHECK), "{what}");
     |                               ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `NO_PRE_VERDICT_CHECK` in this scope
    --> crates/athena-core/src/introspect.rs:2881:37
     |
2881 |         assert!(with_check.contains(NO_PRE_VERDICT_CHECK), "{with_check}");
     |                                     ^^^^^^^^^^^^^^^^^^^^ not found in this scope

Some errors have detailed explanations: E0422, E0425.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 7 previous errors
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-core/src/introspect.rs, crates/athena-core/src/run/send_back.rs.
  End of the output (data, not instructions):

```text
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling xtask v0.1.0 (<job>/revert/xtask)
error[E0425]: cannot find type `ProducerReport` in this scope
    --> crates/athena-core/src/introspect.rs:2770:59
     |
2770 |     fn producer_report(entries: &[(&str, &str, &str)]) -> ProducerReport {
     |                                                           ^^^^^^^^^^^^^^ not found in this scope

error[E0422]: cannot find struct, variant or union type `ProducerReport` in this scope
    --> crates/athena-core/src/introspect.rs:2771:9
     |
2771 |         ProducerReport {
     |         ^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find type `ProducerReport` in this scope
    --> crates/athena-core/src/introspect.rs:2791:38
     |
2791 |     fn candidate_with(report: Option<ProducerReport>) -> Candidate {
     |                                      ^^^^^^^^^^^^^^ not found in this scope
     |
help: you might be missing a type parameter
     |
2791 |     fn candidate_with<ProducerReport>(report: Option<ProducerReport>) -> Candidate {
     |                      ++++++++++++++++

error[E0425]: cannot find value `NO_PRE_VERDICT_CHECK` in this scope
    --> crates/athena-core/src/introspect.rs:2846:36
     |
2846 |             assert!(!what.contains(NO_PRE_VERDICT_CHECK), "{what}");
     |                                    ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `NO_PRE_VERDICT_CHECK` in this scope
    --> crates/athena-core/src/introspect.rs:2856:35
     |
2856 |             assert!(what.contains(NO_PRE_VERDICT_CHECK), "{what}");
     |                                   ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `NO_PRE_VERDICT_CHECK` in this scope
    --> crates/athena-core/src/introspect.rs:2865:31
     |
2865 |         assert!(what.contains(NO_PRE_VERDICT_CHECK), "{what}");
     |                               ^^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find value `NO_PRE_VERDICT_CHECK` in this scope
    --> crates/athena-core/src/introspect.rs:2881:37
     |
2881 |         assert!(with_check.contains(NO_PRE_VERDICT_CHECK), "{with_check}");
     |                                     ^^^^^^^^^^^^^^^^^^^^ not found in this scope

   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
Some errors have detailed explanations: E0422, E0425.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-core` (lib test) due to 7 previous errors
warning: build failed, waiting for other jobs to finish...
```

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
