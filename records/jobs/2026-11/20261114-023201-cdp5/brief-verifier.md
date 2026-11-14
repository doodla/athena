# Job 20261114-023201-cdp5: verify "Make request misread an explicit yes/no field for critic and verifier"

## Request (verbatim)

> Stop Athena from reading a non-empty `request_reading_mismatch` as a misread when the agent's text says there is none. Read: in job 20261114-012353-pbvc the critic's verdict was accept, with no blocker or major finding, and its mismatch field began "None for the plan. One factual error…". end_round in crates/athena-core/src/run/planning.rs (about line 474) stops on any Some(..) before it looks at the verdict, so the job failed and was retried (20261114-013056-pkhp). judge in crates/athena-core/src/verify.rs (about line 625) does the same for the verifier: job 20261029-084645-sza7 failed with "the reading misread the request" when the field began "None for this change." Fix: in CriticOutput and the verifier output (crates/athena-context/src/outputs.rs), make the misread an explicit required yes/no field, with the explanation as a separate optional text. Remarks about the reading that are not a misread go in findings. Update the critic and verifier prompts and the schema text to match. Planning and judge act only on the yes/no field. Keep D93's rule that a real misread is a major finding. Tests: replay the recorded critique text from pbvc and the verifier text from sza7 through the new parse. Neither may stop planning or add the misread reason, and a real misread must still stop planning and fail verification. Update the record readers in display.rs and status.rs, and any fixtures that set the field. Add a CHANGELOG.md entry. Run cargo fmt --check, cargo clippy --all-targets -D warnings and the full cargo test.
> 
> Acceptance:
> - The change is made as described: A critic or verifier that writes text starting "None…" in the misread field is counted as a misread: an accepted plan was stopped and a verification failed
> - No unrelated behavior changes; the existing tests pass
> - fmt, clippy and test pass
> 
> Files: crates/athena-core/src/run/planning.rs, crates/athena-core/src/verify.rs, crates/athena-context/src/outputs.rs, yes/no, display.rs, status.rs, CHANGELOG.md

## Reading (accepted by the user)

Replace the free-text `request_reading_mismatch` in CriticOutput and VerifierOutput with a required boolean `request_misread` plus optional `request_misread_reason`. Planning end_round and verify judge act only on the boolean, so text starting "None…" (pbvc, sza7) no longer stops planning or fails verification. Prior jobs pbvc, pkhp and sza7 are addressed in the assumptions.

Assumptions:
- Code survey: I read crates/athena-context/src/outputs.rs (CriticOutput near line 539 and the verifier output near line 250 both hold `request_reading_mismatch: Option<String>`), crates/athena-core/src/run/planning.rs end_round (stops on any Some before the verdict), crates/athena-core/src/verify.rs judge (same for the verifier), and crates/athena-data/src/records.rs, where Verification (line 944) and the plan record store the same field as Option<String>. The records validation is `check_record_file` (records.rs:1455) returning `AnyRecord` (records.rs:1124); it accepts the record field as it is today. Design: the stored record field `request_reading_mismatch` is kept unchanged and is set only when the boolean is true, so records.rs needs no change and existing records still pass `check_record_file` and the records round-trip tests. Only the agent-facing output structs change. Entry points are the output parse in outputs.rs, then end_round in planning.rs and judge in verify.rs. The property is measured by replaying the recorded texts through the real parse and the real end_round and judge, not through stand-ins.
- Field meaning: `request_reading_mismatch` is documented in outputs.rs:538 as 'Set when the accepted reading misread the request, with how.' It fits as the stored record field once it is set only on a real misread. The agent-facing fields are new: `request_misread` (required bool) and `request_misread_reason` (optional text). Remarks about the reading that are not a misread go in findings. The live athena.toml value is not involved.
- Prior jobs: pbvc (planning stopped on 'None for the plan. One factual error…') and sza7 (verification failed on 'None for this change.') are fixed by the new parse. pkhp (rejected, 7 candidate files: defaults.toml, defaults.rs, athena-context lib.rs, run_pin_tests.rs, decisions.md, system-facts.md, CHANGELOG.md) is unrelated; its work is dropped. Its flaky drain test is not addressed here. sza7's candidate files (athena.toml, checks.rs, disk.rs, run/mod.rs) are dropped, and its status.rs part (3) is not part of this job.
- Consumers: none found for config. No athena.toml or defaults.toml change is needed. The prompt files crates/athena-context/prompts/critic.md and verifier.md are composed into the critic and verifier prompt sets. Critic has 298 tokens of headroom and the verifier (change) has 90 tokens left, so the verifier edit must stay within 90 tokens, which is the binding limit. Reader and other sets are untouched.
- D93's rule is kept: a real misread (`request_misread` true) is recorded as a major finding, so planning stops and verification fails. The new record is appended to docs/decisions.md as a new decision with a Provenance bullet. Existing entries are not edited.

## Acceptance criteria

1. [content] In crates/athena-context/src/outputs.rs, CriticOutput and the verifier output each have a required bool `request_misread` and an optional `request_misread_reason`. The free-text `request_reading_mismatch` is gone from the agent output structs. The schema text matches (Grep outputs.rs).
2. [content] crates/athena-context/prompts/critic.md and verifier.md say the yes/no field is true only for a real misread, that a real misread is also a major finding, and that other remarks go in findings. The verifier prompt grows by at most 90 tokens.
3. [content] end_round in crates/athena-core/src/run/planning.rs and judge in crates/athena-core/src/verify.rs read only the boolean to decide on a misread. Neither tests the reason text for Some (Grep both files).
4. [behavior] Run `cargo test`. A test replays the recorded pbvc critique text ('None for the plan. One factual error…', verdict accept, no blocker or major finding) through the new parse and end_round. Planning is not stopped and no misread reason is added.
5. [behavior] Run `cargo test`. A test replays the recorded sza7 verifier text ('None for this change.') through the new parse and judge. The verdict is not failed for a misread and the reason 'the reading misread the request' is not added.
6. [behavior] Run `cargo test`. With `request_misread` true, planning still stops and verification fails with the misread reason, and a major finding is recorded (D93).
7. [content] The record readers in crates/athena-cli/src/display.rs and crates/athena-core/src/status.rs show a misread only when the stored field is set, and every fixture that set the old field is updated. Existing record round-trip tests still pass `check_record_file`.
8. [content] CHANGELOG.md has a new entry for this change, and docs/decisions.md has a new appended decision with a Provenance bullet.
9. [behavior] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and the full `cargo test` pass. Athena's recorded checks on the final tree count as evidence.

## Expected deliverables

- crates/athena-context/src/outputs.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/records.md (modify)

## Out of scope

- Changing the stored record field in crates/athena-data/src/records.rs: it stays Option<String>, so old records stay readable.
- The pkhp work (defaults.toml version-pins and hermetic-checks entries, reader budget raise, D181): dropped.
- The sza7 work (check_timeout_s, environment stop, status durations): dropped.
- The flaky drain test a_group_of_urgent_items_does_not_start_ahead_of_the_improvement_turn.
- Changing athena.toml, defaults.toml or workflows.
- Changing other planning or critic acceptance rules.

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria replay the recorded pbvc and sza7 texts and check a real misread still stops and fails.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: fmt, clippy and the full test run are required.
- changelog: (not in the defaults model) Here: CHANGELOG.md entry required.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cause is a free-text field read as a flag; the fix is an explicit boolean for both critic and verifier.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New tests in the `mod tests` of crates/athena-context/src/outputs.rs, with names containing `request_misread`. They call `critic_schema()` and `verdict_schema()` and assert that `properties.request_misread.type` is boolean, that `request_misread_reason` is present, that both are in `required`, and that `request_reading_mismatch` is absent. They also assert that `serde_json::from_value::<CriticOutput>` and `::<VerdictOutput>` refuse a JSON that still carries the old key, and that a JSON without `request_misread` is refused. The existing `sample_critique` (outputs.rs:692) and the VerdictOutput literal (outputs.rs:950) are updated to the new fields. Run: `cargo test -p athena-context request_misread`. Observes: The schema requires the boolean and the reason for both agents, the old key is gone, and an old-format output no longer parses.
  Fails without the behavior: With the old structs the schema holds `request_reading_mismatch` and no `request_misread`, so the property assertions fail, and the old-key refusal assertion fails because the old JSON would still parse.
  Test seam: Production `schema_of`, `critic_schema`, `verdict_schema` and serde parse in outputs.rs. No fake is used. The tests are added in step 1, to the existing `mod tests`.
- Criterion 2: Rewrite the prompt anchor rows in crates/athena-context/src/lib.rs (605 and 612) to the new sentences: `request_misread` true only for a real misread, a real misread is also a major finding, other remarks go in findings. Move the `M-mismatch` row of `REQUIRED_FIELDS` (verify.rs:1254) to anchor on `set request_misread true`, and move the test at verify.rs:3240 to replace that phrase. Then `git diff -- crates/athena-context/prompts/verifier.md` shows the net growth, which must stay under 90 tokens (270 bytes). The target is about 100 bytes, and the headroom in the facts file is 67 tokens. Run: `cargo test -p athena-context && cargo test -p athena-cli --test budgets && cargo test -p athena-core --lib verify`. Observes: The critic and verifier prompts carry the new wording. `check_verifier_prompt` accepts the shipped frame and the cap-change and cap-study fragments. The budgets test passes with the facts lines updated, and the verifier set stays at or under 1300.
  Fails without the behavior: The lib.rs anchor test fails if a prompt lacks the new wording. `check_verifier_prompt` refuses the frame with `M-mismatch` if the anchor phrase is missing. The budgets test fails if a prompt overflows or the facts lines are stale.
  Test seam: Production `compose`, `check_budget`, `check_verifier_prompt` and the prompt-anchor test in the context lib.rs, through `system_facts_budgets_match_computed` (budgets.rs:125) and the verify.rs prompt tests (verify.rs:3167-3290). All exist today. Step 8 edits the anchors and the facts lines.
- Criterion 3: Read the diff of `critic_round`, `end_round` and `judge`. `end_round` has a `request_misread: bool` parameter and an `if request_misread` branch. `critic_round` passes `critic_output.request_misread`. `judge` tests `output.request_misread`. No `if let Some(..)` or `.is_some()` test of `request_misread_reason` or of `request_reading_mismatch` decides either outcome. The reason is read only to build the message and the finding evidence. Run: `git diff -- crates/athena-core/src/run/planning.rs crates/athena-core/src/verify.rs`. Observes: Both decisions branch on the boolean. The reason text is used for wording only.
  Fails without the behavior: If either function still tested the reason text for Some, the criterion 4 and 5 replays would stop planning or fail verification.
  Test seam: Not applicable: a content criterion read from the diff. Criteria 4 and 5 give the behavior proof.
- Criterion 4: New tests in crates/athena-core/src/run/tests.rs next to the planning tests (step 11). The tests build `repo_with_planning(3)` and a `state` tempdir. They call `athena_with(&repo, &state, PASSES, crate::test_support::plenty_of_space, planning_script(vec![good_plan()], vec![critique], calls.clone()))` and then `finished_job`. The critique JSON is the pbvc critic output as the new agent writes it: verdict accept, findings with minor items only, `request_misread` false, and `request_misread_reason` starting 'None for the plan. One factual error' (recorded start from the request; the rest is reconstructed and labelled so in the comment, because records/ is absent). A second variant sets the reason to null. The tests assert `finished.state == Landed`, a non-empty `prompts_of(&calls, Role::Producer)`, and that the stored critique (from `Records::load(&athena.git, &athena.head().unwrap())`, `jobs[job.as_str()].critiques`) has `final_verdict` Accept and `request_reading_mismatch` None. They also assert that `finished.reason` lacks 'misread'. Run: `cargo test -p athena-core a_critique_whose_reason_starts_with_none_does_not_stop_planning`. Observes: Planning is not stopped, the producer runs, and no misread reason is added to the job or its critique.
  Fails without the behavior: An `end_round` or `critic_round` that tests the reason text for Some stops the job with 'the critic judged that the reading misread the request', makes no producer call, and stores the text. The three assertions on state, the producer call and the stored critique then fail. The test is a replay that maps the recorded text into the reason with the boolean false. It does not run the old code.
  Test seam: Production entry `Athena::go` through `athena_with` and `finished_job` into the real `critic_round`, `CriticOutput` parse, `critic_findings`, `plan_verdict` and `end_round`. The fake agent script `planning_script` (run/tests.rs:1204) only supplies the critic JSON. It exists today. The new tests are added in step 11.
- Criterion 5: Two tests. In crates/athena-core/src/verify.rs tests, `the_sza7_reason_with_request_misread_false_adds_nothing` builds a `VerdictOutput` with `serde_json::from_value` (pass, both criteria met, no findings, `request_misread` false, `request_misread_reason` 'None for this change.', the recorded sza7 text). It calls `judge_with(&out, &["cargo test".to_string()], &[check(true)])` with the existing helpers `output()` and `check()`. It asserts `final_verdict` Pass, empty `findings`, and no reason containing 'misread the request'. In run/tests.rs, `the_sza7_verifier_text_does_not_fail_verification` wraps `base_script()` in a closure like the one in `run_planned_behavior` (run/tests.rs:1267). The closure sets `request_misread_reason` on the verifier turn and leaves the boolean false. It runs `athena_with(&repo(), ...)` and `finished_job`, and asserts Landed, no misread reason in `finished.verification.unwrap().reasons`, and `request_reading_mismatch` None in the stored verification. Run: `cargo test -p athena-core the_sza7`. Observes: The verdict is not failed for a misread, and neither the reason 'the reading misread the request' nor a major finding is added.
  Fails without the behavior: A `judge` that tests the reason text for Some returns Fail, adds 'the reading misread the request' and a Major finding with the sza7 text, and `check_and_verify` stores the text. Each assertion above then fails. The test maps the recorded text into the reason with the boolean false, as its name and comment say.
  Test seam: Production `judge` (verify.rs:408) with the `output()` and `judge_with()` helpers (verify.rs:1935, 1958), which exist. For the second test, production `Athena::go` into `check_and_verify` (run/verification.rs, redaction at 350-377 and the stored field at 422), with a `base_script` wrapper that step 11 adds. The fake supplies only the verifier JSON.
- Criterion 6: Four tests plus one existing. In verify.rs, `a_true_request_misread_is_a_major_finding_and_the_verdict_is_not_pass` replaces the old test at verify.rs:2935. It sets `request_misread` true with a reason, and asserts Fail, a Major finding with source `FindingSource::Athena` whose evidence is the reason, and 'misread the request' in `reasons`. A sibling with no reason asserts the same with evidence 'no explanation given'. In run/tests.rs, `a_true_request_misread_stops_planning` runs `athena_with` and `planning_script` with an accept critique, `request_misread` true and a reason. It asserts Failed, a `finished.reason` containing 'the critic judged that the reading misread the request: <reason>', an empty producer prompt list, and a stored critique with `request_reading_mismatch` Some(reason). `a_true_request_misread_fails_verification` wraps the verifier turn as in criterion 5 with the boolean true, and asserts a Rejected state, the 'the reading misread the request' reason, a Major Athena finding, and the stored verification field Some(reason). The existing crates/athena-cli/tests/job_plan.rs test at line 636 still asserts 'misread the request: It reads the request as a refactor', with `critique_of` (line 462) mapping its `mismatch` argument to `request_misread` and the reason. Run: `cargo test -p athena-core a_true_request_misread && cargo test -p athena-cli --test job_plan a_reject_a_misread_reading`. Observes: With the boolean true, planning stops and verification fails, each with the misread reason, and the verification records a Major finding (D93). The critic path records no Athena finding.
  Fails without the behavior: A `judge` or `end_round` that ignored the boolean would let a true value pass verification or continue planning. The Fail, Failed and Rejected assertions would then fail. A `judge` that dropped the finding would fail the `any(Major)` assertion.
  Test seam: Same production entries as criteria 4 and 5. The `critique_of` helper and `Scenario` in job_plan.rs exist today. The job_plan.rs edit and the new tests are added in steps 10 and 11.
- Criterion 7: status.rs `plan_state` (line 133) already reads the stored field, so step 9 updates its doc line and adds a unit test next to the fixtures at status.rs:2413 and 2577. The test shows that a critique with a final verdict of accept and the stored field None is accepted, that the same critique with the field Some is stopped, and that a revise stays stopped. In display.rs (line 1921) the logic is unchanged: it prints the misread line only for a stored value. A new assertion renders a critique with the stored field None and one with Some, and checks that the first has no 'misread the request' line. Every agent-JSON fixture is updated, and the record-shaped fixtures (`request_reading_mismatch: None` literals) and the invariant.rs JSON are left as they are. The round-trip tests of `check_record_file` (records.rs:1455) still pass. Run: `cargo test -p athena-core --lib status && cargo test -p athena-cli`. Observes: The readers show a misread only for a set stored field. The cli suite and the records round trips pass with the new fixtures.
  Fails without the behavior: A reader that showed a misread with the field unset would fail the new `plan_state` and display assertions. A fixture still carrying the old key would fail to parse and fail its cli or core test.
  Test seam: Production `plan_state`, the display renderer and `check_record_file`, through existing tests. The only new double is a `Critique` record with the stored field None or Some. Step 9 adds the tests.
- Criterion 8: Read `git diff -- CHANGELOG.md docs/decisions.md docs/records.md`. CHANGELOG.md gets an entry under `## [Unreleased]` / `### Fixed` (line 8 and 10). It names the root cause, the new fields, jobs 20261114-012353-pbvc, 20261114-013056-pkhp (the retry) and 20261029-084645-sza7, the unchanged stored field, the prompt and budget effect, and the dropped reason text when the boolean is false. docs/decisions.md gets `## D181 · …` appended after D180 (lines 5198-5225) in the D180 form: Case, Decision, Alternatives, Tradeoff, Reversal cost and a Provenance bullet with `Not measured:` for anything not run. It edits no earlier entry. docs/records.md gets one sentence at lines 97-99. Run: `cargo test -p xtask && cargo test -p athena-cli`. Observes: Both files carry the new entry with a Provenance bullet, and the existing decision-log and mirror checks pass.
  Fails without the behavior: The decision-log structure checks and the changelog tests flag a decision entry without its required bullets. Without the entries the criterion is unmet.
  Test seam: Not applicable for the diff read. The log structure is checked by existing tests (xtask policy tests and the cli tests), and they exist today.
- Criterion 9: Run the three Athena checks on the final tree. Athena's recorded checks on the same tree count as evidence. Also run `cargo test -p athena-core --test refusal_corpus`: the change touches no reading refusal or matching rule, so no corpus case is added or changed. The producer and verifier run it only as a guard. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: fmt, clippy and the full test run all pass.
  Fails without the behavior: A fixture still carrying the old key, a stale constructor of VerdictOutput (send_back.rs:462, verify.rs:1936, outputs.rs:950), a stale anchor or a stale budget line fails to compile or fails a test.
  Test seam: Not applicable: the repository's own checks.

Planned files:

- crates/athena-context/src/outputs.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/verify.rs (modify)
- crates/athena-core/src/run/verification.rs (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-context/prompts/verifier.md (modify)
- crates/athena-context/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- docs/records.md (modify)
- crates/athena-core/src/run/send_back.rs (modify)
- docs/system-facts.md (modify)
- crates/athena-core/src/scope_guard.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 947f3b9b274e → candidate 35cc6c5d59f5. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |   1 +
 crates/athena-agent/src/fake.rs            |   6 +-
 crates/athena-cli/src/display.rs           |  53 ++++++++
 crates/athena-cli/tests/common/mod.rs      |   3 +-
 crates/athena-cli/tests/drain.rs           |   3 +-
 crates/athena-cli/tests/introspection.rs   |   6 +-
 crates/athena-cli/tests/job_plan.rs        |   3 +-
 crates/athena-cli/tests/lifecycle.rs       |  15 ++-
 crates/athena-cli/tests/retry.rs           |   3 +-
 crates/athena-context/prompts/critic.md    |   2 +-
 crates/athena-context/prompts/verifier.md  |   4 +-
 crates/athena-context/src/lib.rs           |  11 +-
 crates/athena-context/src/outputs.rs       |  79 ++++++++++-
 crates/athena-core/src/run/planning.rs     |  17 ++-
 crates/athena-core/src/run/send_back.rs    |   3 +-
 crates/athena-core/src/run/tests.rs        | 210 ++++++++++++++++++++++++++++-
 crates/athena-core/src/run/verification.rs |  17 ++-
 crates/athena-core/src/run_pin_tests.rs    |   3 +-
 crates/athena-core/src/scope_guard.rs      |   3 +-
 crates/athena-core/src/status.rs           |  20 ++-
 crates/athena-core/src/verify.rs           |  96 +++++++++----
 docs/decisions.md                          |  32 +++++
 docs/records.md                            |   4 +-
 docs/system-facts.md                       |   4 +-
 24 files changed, 526 insertions(+), 72 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261114-023201-cdp5/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261114-023201-cdp5/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43578429832 for 35cc6c5d59f5d1ae09b25c146a44cb18c1718813)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43578429832 for 35cc6c5d59f5d1ae09b25c146a44cb18c1718813)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43578429832 for 35cc6c5d59f5d1ae09b25c146a44cb18c1718813)
- ci-result: `not a check` passed (Check results for 35cc6c5d59f5d1ae09b25c146a44cb18c1718813 come from the CI run https://github.com/doodla/athena/actions/runs/43578429832 (667s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and the full `cargo test` pass. Athena's recorded checks on the final tree count as evidence.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-context request_misread`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/fake.rs, crates/athena-cli/src/display.rs, crates/athena-context/prompts/critic.md, crates/athena-context/prompts/verifier.md, crates/athena-context/src/outputs.rs, crates/athena-core/src/run/planning.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/status.rs, crates/athena-core/src/verify.rs, docs/decisions.md, docs/records.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
   Compiling itoa v1.0.18
   Compiling cfg-if v1.0.5
   Compiling ref-cast v1.0.27
   Compiling toml_datetime v0.7.5+spec-1.1.0
   Compiling serde_spanned v1.1.2
   Compiling serde_derive_internals v0.30.0
   Compiling toml_parser v1.1.5+spec-1.1.0
   Compiling serde_derive v1.0.229
   Compiling ref-cast-impl v1.0.27
   Compiling thiserror-impl v2.0.21
   Compiling digest v0.10.7
   Compiling jiff-core v0.1.1
   Compiling winnow v0.7.15
   Compiling toml_writer v1.1.3+spec-1.1.0
   Compiling cpufeatures v0.2.17
   Compiling sha2 v0.10.9
   Compiling jiff v0.2.38
   Compiling schemars_derive v1.2.2
   Compiling toml v0.9.12+spec-1.1.0
   Compiling dyn-clone v1.0.20
   Compiling schemars v1.2.2
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
error[E0425]: cannot find function `misread_explanation` in this scope
   --> crates/athena-context/src/outputs.rs:917:13
    |
917 |             misread_explanation(Some("  reads it as a refactor ")),
    |             ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `misread_explanation` in this scope
   --> crates/athena-context/src/outputs.rs:920:20
    |
920 |         assert_eq!(misread_explanation(Some("  ")), "no explanation given");
    |                    ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0425]: cannot find function `misread_explanation` in this scope
   --> crates/athena-context/src/outputs.rs:921:20
    |
921 |         assert_eq!(misread_explanation(None), "no explanation given");
    |                    ^^^^^^^^^^^^^^^^^^^ not found in this scope

error[E0560]: struct `outputs::VerdictOutput` has no field named `request_misread`
    --> crates/athena-context/src/outputs.rs:1013:13
     |
1013 |             request_misread: false,
     |             ^^^^^^^^^^^^^^^ `outputs::VerdictOutput` does not have this field
     |
     = note: available fields are: `request_reading_mismatch`

error[E0560]: struct `outputs::VerdictOutput` has no field named `request_misread_reason`
    --> crates/athena-context/src/outputs.rs:1014:13
     |
1014 |             request_misread_reason: None,
     |             ^^^^^^^^^^^^^^^^^^^^^^ `outputs::VerdictOutput` does not have this field
     |
     = note: available fields are: `request_reading_mismatch`

Some errors have detailed explanations: E0425, E0560.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `athena-context` (lib test) due to 5 previous errors
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test -p athena-context && cargo test -p athena-cli --test budgets && cargo test -p athena-core --lib verify`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 3 (`git diff -- crates/athena-core/src/run/planning.rs crates/athena-core/src/verify.rs`). Result: not run: not a plain cargo test command.
- fails-without-4: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 4 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core a_critique_whose_reason_starts_with_none_does_not_stop_planning`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/fake.rs, crates/athena-cli/src/display.rs, crates/athena-context/prompts/critic.md, crates/athena-context/prompts/verifier.md, crates/athena-context/src/outputs.rs, crates/athena-core/src/run/planning.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/status.rs, crates/athena-core/src/verify.rs, docs/decisions.md, docs/records.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0560]: struct `athena_context::outputs::VerdictOutput` has no field named `request_misread`
   --> crates/athena-core/src/run/send_back.rs:471:13
    |
471 |             request_misread: false,
    |             ^^^^^^^^^^^^^^^ `athena_context::outputs::VerdictOutput` does not have this field
    |
    = note: available fields are: `request_reading_mismatch`

error[E0560]: struct `athena_context::outputs::VerdictOutput` has no field named `request_misread_reason`
   --> crates/athena-core/src/run/send_back.rs:472:13
    |
472 |             request_misread_reason: None,
    |             ^^^^^^^^^^^^^^^^^^^^^^ `athena_context::outputs::VerdictOutput` does not have this field
    |
    = note: available fields are: `request_reading_mismatch`

error[E0560]: struct `athena_context::outputs::VerdictOutput` has no field named `request_misread`
    --> crates/athena-core/src/verify.rs:1954:13
     |
1954 |             request_misread: false,
     |             ^^^^^^^^^^^^^^^ `athena_context::outputs::VerdictOutput` does not have this field
     |
     = note: available fields are: `request_reading_mismatch`

error[E0560]: struct `athena_context::outputs::VerdictOutput` has no field named `request_misread_reason`
    --> crates/athena-core/src/verify.rs:1955:13
     |
1955 |             request_misread_reason: None,
     |             ^^^^^^^^^^^^^^^^^^^^^^ `athena_context::outputs::VerdictOutput` does not have this field
     |
     = note: available fields are: `request_reading_mismatch`

error[E0609]: no field `request_misread` on type `athena_context::outputs::VerdictOutput`
    --> crates/athena-core/src/verify.rs:2932:13
     |
2932 |         out.request_misread = misread;
     |             ^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `verdict`, `criteria`, `findings`, `request_reading_mismatch`, `summary`

error[E0609]: no field `request_misread_reason` on type `athena_context::outputs::VerdictOutput`
    --> crates/athena-core/src/verify.rs:2933:13
     |
2933 |         out.request_misread_reason = reason.map(String::from);
     |             ^^^^^^^^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `verdict`, `criteria`, `findings`, `request_reading_mismatch`, `summary`

Some errors have detailed explanations: E0560, E0609.
For more information about an error, try `rustc --explain E0560`.
error: could not compile `athena-core` (lib test) due to 6 previous errors
warning: build failed, waiting for other jobs to finish...
```
- fails-without-5: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 5 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test -p athena-core the_sza7`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/fake.rs, crates/athena-cli/src/display.rs, crates/athena-context/prompts/critic.md, crates/athena-context/prompts/verifier.md, crates/athena-context/src/outputs.rs, crates/athena-core/src/run/planning.rs, crates/athena-core/src/run/verification.rs, crates/athena-core/src/status.rs, crates/athena-core/src/verify.rs, docs/decisions.md, docs/records.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
error[E0560]: struct `athena_context::outputs::VerdictOutput` has no field named `request_misread`
   --> crates/athena-core/src/run/send_back.rs:471:13
    |
471 |             request_misread: false,
    |             ^^^^^^^^^^^^^^^ `athena_context::outputs::VerdictOutput` does not have this field
    |
    = note: available fields are: `request_reading_mismatch`

error[E0560]: struct `athena_context::outputs::VerdictOutput` has no field named `request_misread_reason`
   --> crates/athena-core/src/run/send_back.rs:472:13
    |
472 |             request_misread_reason: None,
    |             ^^^^^^^^^^^^^^^^^^^^^^ `athena_context::outputs::VerdictOutput` does not have this field
    |
    = note: available fields are: `request_reading_mismatch`

error[E0560]: struct `athena_context::outputs::VerdictOutput` has no field named `request_misread`
    --> crates/athena-core/src/verify.rs:1954:13
     |
1954 |             request_misread: false,
     |             ^^^^^^^^^^^^^^^ `athena_context::outputs::VerdictOutput` does not have this field
     |
     = note: available fields are: `request_reading_mismatch`

error[E0560]: struct `athena_context::outputs::VerdictOutput` has no field named `request_misread_reason`
    --> crates/athena-core/src/verify.rs:1955:13
     |
1955 |             request_misread_reason: None,
     |             ^^^^^^^^^^^^^^^^^^^^^^ `athena_context::outputs::VerdictOutput` does not have this field
     |
     = note: available fields are: `request_reading_mismatch`

error[E0609]: no field `request_misread` on type `athena_context::outputs::VerdictOutput`
    --> crates/athena-core/src/verify.rs:2932:13
     |
2932 |         out.request_misread = misread;
     |             ^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `verdict`, `criteria`, `findings`, `request_reading_mismatch`, `summary`

error[E0609]: no field `request_misread_reason` on type `athena_context::outputs::VerdictOutput`
    --> crates/athena-core/src/verify.rs:2933:13
     |
2933 |         out.request_misread_reason = reason.map(String::from);
     |             ^^^^^^^^^^^^^^^^^^^^^^ unknown field
     |
     = note: available fields are: `verdict`, `criteria`, `findings`, `request_reading_mismatch`, `summary`

Some errors have detailed explanations: E0560, E0609.
For more information about an error, try `rustc --explain E0560`.
error: could not compile `athena-core` (lib test) due to 6 previous errors
```
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`cargo test -p athena-core a_true_request_misread && cargo test -p athena-cli --test job_plan a_reject_a_misread_reading`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo test -p athena-core --lib status && cargo test -p athena-cli`). Result: not run: not a plain cargo test command.
- fails-without-8: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 8 (`cargo test -p xtask && cargo test -p athena-cli`). Result: not run: not a plain cargo test command.
- fails-without-9: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 9 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
